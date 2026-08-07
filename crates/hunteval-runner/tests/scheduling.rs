use hunteval_domain::{AgentId, TaskId};
use hunteval_runner::{ScheduledTask, deterministic_schedule};

fn item(
    sequence: u64,
    agent: &str,
    task: &str,
) -> Result<ScheduledTask, Box<dyn std::error::Error>> {
    Ok(ScheduledTask {
        ready_sequence: sequence,
        agent_id: AgentId::new(agent)?,
        task_id: TaskId::new(task)?,
    })
}

#[test]
fn scheduling_permutations_have_one_deterministic_order() -> Result<(), Box<dyn std::error::Error>>
{
    let first = vec![
        item(2, "worker-b", "task-2")?,
        item(1, "worker-a", "task-1")?,
        item(2, "worker-a", "task-3")?,
    ];
    let mut second = first.clone();
    second.reverse();
    assert_eq!(
        deterministic_schedule(first),
        deterministic_schedule(second)
    );
    Ok(())
}
