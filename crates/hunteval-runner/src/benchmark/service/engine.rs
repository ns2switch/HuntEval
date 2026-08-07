use std::{fmt, path::Path, thread};

use hunteval_domain::{BenchmarkAttemptId, BenchmarkCellId, DeploymentId, RunId, UtcTimestamp};
use time::OffsetDateTime;

use crate::benchmark::{BenchmarkCellStatus, BenchmarkEventKind, BenchmarkJournal, BenchmarkState};

use super::{
    BenchmarkCellExecutor, BenchmarkExecutionPlan, BenchmarkRunOptions, BenchmarkRunSummary,
    BenchmarkServiceError, CellExecutionFailure, ComparisonEligibility, RetryPolicy,
    storage::{store_definition, verify_definition},
    types::cell_state_map,
};

pub struct BenchmarkService<'a> {
    executor: &'a dyn BenchmarkCellExecutor,
}

impl fmt::Debug for BenchmarkService<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BenchmarkService")
            .finish_non_exhaustive()
    }
}

impl<'a> BenchmarkService<'a> {
    #[must_use]
    pub const fn new(executor: &'a dyn BenchmarkCellExecutor) -> Self {
        Self { executor }
    }

    pub fn run(
        &self,
        plan: &BenchmarkExecutionPlan,
        output_root: &Path,
        options: BenchmarkRunOptions,
    ) -> Result<BenchmarkRunSummary, BenchmarkServiceError> {
        validate_options(options)?;
        validate_plan(plan)?;
        let mut journal = BenchmarkJournal::open(output_root, plan.definition.id.clone())?;
        let timestamp = now()?;
        if journal.state().is_none() {
            store_definition(output_root, &plan.definition)?;
            journal.append(timestamp, BenchmarkEventKind::BenchmarkStarted)?;
            for cell in plan.definition.cells()? {
                journal.append(
                    timestamp,
                    BenchmarkEventKind::CellQueued {
                        cell_id: cell.cell_id,
                    },
                )?;
            }
        } else {
            verify_definition(output_root, &plan.definition)?;
            journal.interrupt_running(timestamp)?;
        }
        self.execute_selected(plan, output_root, options, &mut journal)?;
        summarize(journal.state().as_ref())
    }

    pub fn status(
        output_root: &Path,
        definition: &hunteval_domain::BenchmarkDefinition,
    ) -> Result<BenchmarkRunSummary, BenchmarkServiceError> {
        verify_definition(output_root, definition)?;
        let journal = BenchmarkJournal::open(output_root, definition.id.clone())?;
        summarize(journal.state().as_ref())
    }

    pub fn compare(
        output_root: &Path,
        definition: &hunteval_domain::BenchmarkDefinition,
        left: &DeploymentId,
        right: &DeploymentId,
    ) -> Result<ComparisonEligibility, BenchmarkServiceError> {
        verify_definition(output_root, definition)?;
        let journal = BenchmarkJournal::open(output_root, definition.id.clone())?;
        let state = journal.state().ok_or(BenchmarkServiceError::MissingState)?;
        Ok(ComparisonEligibility::evaluate(
            definition,
            &state,
            output_root,
            left,
            right,
        )?)
    }

    fn execute_selected(
        &self,
        plan: &BenchmarkExecutionPlan,
        output_root: &Path,
        options: BenchmarkRunOptions,
        journal: &mut BenchmarkJournal,
    ) -> Result<(), BenchmarkServiceError> {
        let state = journal.state().ok_or(BenchmarkServiceError::MissingState)?;
        let states = cell_state_map(&state);
        let selected = plan
            .definition
            .cells()?
            .into_iter()
            .filter(|cell| should_execute(states.get(&cell.cell_id), options.retry))
            .collect::<Vec<_>>();
        let runs_root = output_root.join("runs");
        for batch in selected.chunks(options.jobs) {
            let mut work = Vec::with_capacity(batch.len());
            for cell in batch {
                let attempt_number = states
                    .get(&cell.cell_id)
                    .map_or(1, |state| state.attempt_ids.len() + 1);
                let attempt_id = attempt_id(cell.cell_id, attempt_number)?;
                let run_id = run_id(cell.cell_id, attempt_number)?;
                journal.append(
                    now()?,
                    BenchmarkEventKind::AttemptStarted {
                        cell_id: cell.cell_id,
                        attempt_id: attempt_id.clone(),
                    },
                )?;
                work.push((cell.clone(), attempt_id, run_id));
            }
            let results = thread::scope(|scope| {
                let handles = work
                    .iter()
                    .map(|(cell, attempt, run)| {
                        let runs_root = runs_root.clone();
                        (
                            cell.cell_id,
                            attempt.clone(),
                            scope.spawn(move || {
                                self.executor.execute(cell, attempt, run, &runs_root)
                            }),
                        )
                    })
                    .collect::<Vec<_>>();
                handles
                    .into_iter()
                    .map(|(cell_id, attempt, handle)| {
                        let result = match handle.join() {
                            Ok(result) => result,
                            Err(_) => Err(CellExecutionFailure::validated("executor_panicked")),
                        };
                        (cell_id, attempt, result)
                    })
                    .collect::<Vec<_>>()
            });
            let mut failed = false;
            for (cell_id, attempt_id, result) in results {
                match result {
                    Ok(execution) => {
                        journal.complete_attempt(
                            now()?,
                            cell_id,
                            attempt_id,
                            execution.run_id,
                            &execution.result_path,
                        )?;
                    }
                    Err(error) => {
                        failed = true;
                        journal.append(
                            now()?,
                            BenchmarkEventKind::AttemptFailed {
                                cell_id,
                                attempt_id,
                                reason_code: error.reason_code,
                            },
                        )?;
                    }
                }
            }
            if failed && options.fail_fast {
                break;
            }
        }
        let state = journal.state().ok_or(BenchmarkServiceError::MissingState)?;
        if !selected.is_empty()
            && state.cells.iter().all(|cell| {
                matches!(
                    cell.status,
                    BenchmarkCellStatus::Completed | BenchmarkCellStatus::NonComparable
                )
            })
        {
            journal.append(now()?, BenchmarkEventKind::BenchmarkCompleted)?;
        }
        Ok(())
    }
}

fn should_execute(
    state: Option<&crate::benchmark::BenchmarkCellState>,
    retry: RetryPolicy,
) -> bool {
    match state {
        Some(cell) if cell.status == BenchmarkCellStatus::Pending => true,
        Some(cell) if cell.status == BenchmarkCellStatus::Failed => match retry {
            RetryPolicy::Failed => true,
            RetryPolicy::Interrupted => {
                cell.reason_code.as_deref() == Some("controller_interrupted")
            }
            RetryPolicy::None => false,
        },
        _ => false,
    }
}

fn validate_options(options: BenchmarkRunOptions) -> Result<(), BenchmarkServiceError> {
    if options.jobs == 0 || options.jobs > 256 {
        Err(BenchmarkServiceError::InvalidOptions)
    } else {
        Ok(())
    }
}

fn validate_plan(plan: &BenchmarkExecutionPlan) -> Result<(), BenchmarkServiceError> {
    plan.definition.validate()?;
    let deployments_complete = plan
        .definition
        .deployments
        .iter()
        .all(|item| plan.deployments.contains_key(&item.id));
    let episodes_complete = plan
        .definition
        .episodes
        .iter()
        .all(|item| plan.episodes.contains_key(&item.id));
    if deployments_complete && episodes_complete {
        Ok(())
    } else {
        Err(BenchmarkServiceError::IncompletePlan)
    }
}

fn now() -> Result<UtcTimestamp, BenchmarkServiceError> {
    UtcTimestamp::new(OffsetDateTime::now_utc()).map_err(|_| BenchmarkServiceError::InvalidOptions)
}

fn attempt_id(
    cell_id: BenchmarkCellId,
    number: usize,
) -> Result<BenchmarkAttemptId, BenchmarkServiceError> {
    BenchmarkAttemptId::new(format!("attempt-{}-{number}", cell_id))
        .map_err(|_| BenchmarkServiceError::InvalidIdentifier)
}

fn run_id(cell_id: BenchmarkCellId, number: usize) -> Result<RunId, BenchmarkServiceError> {
    RunId::new(format!("run-{}-{number}", cell_id))
        .map_err(|_| BenchmarkServiceError::InvalidIdentifier)
}

fn summarize(state: Option<&BenchmarkState>) -> Result<BenchmarkRunSummary, BenchmarkServiceError> {
    let state = state.ok_or(BenchmarkServiceError::MissingState)?;
    let count = |status| {
        state
            .cells
            .iter()
            .filter(|cell| cell.status == status)
            .count()
    };
    Ok(BenchmarkRunSummary {
        total: state.cells.len(),
        completed: count(BenchmarkCellStatus::Completed),
        failed: count(BenchmarkCellStatus::Failed),
        pending: count(BenchmarkCellStatus::Pending) + count(BenchmarkCellStatus::Running),
        non_comparable: count(BenchmarkCellStatus::NonComparable),
    })
}
