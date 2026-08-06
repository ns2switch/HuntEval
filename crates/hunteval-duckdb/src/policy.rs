use std::{collections::BTreeSet, ops::ControlFlow};

use sqlparser::{
    ast::{
        Expr, ObjectName, Query, Select, SelectFlavor, SetExpr, Statement, TableFactor, Value,
        ValueWithSpan, Visit, Visitor,
    },
    dialect::DuckDbDialect,
    parser::Parser,
};
use thiserror::Error;

/// Deny-by-default SQL policy for the supported read-only query subset.
#[derive(Debug, Clone)]
pub struct SqlPolicy {
    allowed_tables: BTreeSet<String>,
}

impl SqlPolicy {
    #[must_use]
    pub fn new(allowed_tables: impl IntoIterator<Item = String>) -> Self {
        Self {
            allowed_tables: allowed_tables.into_iter().collect(),
        }
    }

    /// Parses and structurally validates exactly one parameterized `SELECT`.
    pub fn validate(&self, sql: &str, parameter_count: usize) -> Result<(), SqlPolicyError> {
        let statements =
            Parser::parse_sql(&DuckDbDialect {}, sql).map_err(|_| SqlPolicyError::InvalidSyntax)?;
        if statements.len() != 1 {
            return Err(SqlPolicyError::MultipleStatements);
        }
        let statement = statements
            .first()
            .ok_or(SqlPolicyError::MultipleStatements)?;
        if !matches!(statement, Statement::Query(_)) {
            return Err(SqlPolicyError::ReadOnlyQueryRequired);
        }

        let mut visitor = PolicyVisitor {
            allowed_tables: &self.allowed_tables,
            placeholders: 0,
        };
        if let ControlFlow::Break(error) = statement.visit(&mut visitor) {
            return Err(error);
        }
        if visitor.placeholders != parameter_count {
            return Err(SqlPolicyError::ParameterCountMismatch);
        }
        Ok(())
    }
}

struct PolicyVisitor<'a> {
    allowed_tables: &'a BTreeSet<String>,
    placeholders: usize,
}

impl Visitor for PolicyVisitor<'_> {
    type Break = SqlPolicyError;

    fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<Self::Break> {
        let is_plain_select = matches!(query.body.as_ref(), SetExpr::Select(_));
        if !is_plain_select
            || query.with.is_some()
            || query.fetch.is_some()
            || !query.locks.is_empty()
            || query.for_clause.is_some()
            || query.settings.is_some()
            || query.format_clause.is_some()
            || !query.pipe_operators.is_empty()
        {
            return ControlFlow::Break(SqlPolicyError::UnsupportedQueryShape);
        }
        ControlFlow::Continue(())
    }

    fn pre_visit_select(&mut self, select: &Select) -> ControlFlow<Self::Break> {
        if !select.optimizer_hints.is_empty()
            || select.select_modifiers.is_some()
            || select.top.is_some()
            || select.exclude.is_some()
            || select.into.is_some()
            || !select.lateral_views.is_empty()
            || select.prewhere.is_some()
            || !select.connect_by.is_empty()
            || !select.cluster_by.is_empty()
            || !select.distribute_by.is_empty()
            || !select.sort_by.is_empty()
            || !select.named_window.is_empty()
            || select.qualify.is_some()
            || select.value_table_mode.is_some()
            || select.flavor != SelectFlavor::Standard
        {
            return ControlFlow::Break(SqlPolicyError::UnsupportedQueryShape);
        }
        ControlFlow::Continue(())
    }

    fn pre_visit_table_factor(&mut self, table: &TableFactor) -> ControlFlow<Self::Break> {
        match table {
            TableFactor::Table {
                args,
                with_hints,
                version,
                with_ordinality,
                partitions,
                json_path,
                sample,
                index_hints,
                ..
            } if args.is_none()
                && with_hints.is_empty()
                && version.is_none()
                && !with_ordinality
                && partitions.is_empty()
                && json_path.is_none()
                && sample.is_none()
                && index_hints.is_empty() =>
            {
                ControlFlow::Continue(())
            }
            _ => ControlFlow::Break(SqlPolicyError::TableFunctionRejected),
        }
    }

    fn pre_visit_relation(&mut self, relation: &ObjectName) -> ControlFlow<Self::Break> {
        let name = relation.to_string().to_ascii_lowercase();
        if self.allowed_tables.contains(&name) {
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(SqlPolicyError::UnknownTable)
        }
    }

    fn pre_visit_expr(&mut self, expression: &Expr) -> ControlFlow<Self::Break> {
        if let Expr::Function(function) = expression {
            let name = function.name.to_string().to_ascii_lowercase();
            const ALLOWED: &[&str] = &[
                "avg",
                "coalesce",
                "count",
                "length",
                "lower",
                "max",
                "min",
                "substr",
                "substring",
                "sum",
                "upper",
            ];
            if !ALLOWED.contains(&name.as_str()) {
                return ControlFlow::Break(SqlPolicyError::UnknownFunction);
            }
        }
        ControlFlow::Continue(())
    }

    fn pre_visit_value(&mut self, value: &ValueWithSpan) -> ControlFlow<Self::Break> {
        if matches!(value.value, Value::Placeholder(_)) {
            self.placeholders += 1;
        }
        ControlFlow::Continue(())
    }
}

/// Stable SQL policy rejection reasons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum SqlPolicyError {
    #[error("SQL syntax is invalid")]
    InvalidSyntax,
    #[error("exactly one SQL statement is required")]
    MultipleStatements,
    #[error("a read-only SELECT query is required")]
    ReadOnlyQueryRequired,
    #[error("SQL query shape is outside the supported subset")]
    UnsupportedQueryShape,
    #[error("SQL table functions are not allowed")]
    TableFunctionRejected,
    #[error("SQL query references an unknown table")]
    UnknownTable,
    #[error("SQL query references a function outside the allowlist")]
    UnknownFunction,
    #[error("SQL placeholder count does not match bound parameters")]
    ParameterCountMismatch,
}
