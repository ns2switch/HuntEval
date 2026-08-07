use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Hard limits enforced by the trusted runner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BudgetLimits {
    pub messages: u64,
    pub tool_calls: u64,
    pub tokens: u64,
}

/// Monotonic usage observed by the runner.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BudgetUsage {
    pub messages: u64,
    pub tool_calls: u64,
    pub tokens: u64,
}

/// Checked, monotonic budget ledger.
#[derive(Debug, Clone)]
pub struct BudgetLedger {
    limits: BudgetLimits,
    usage: BudgetUsage,
}

impl BudgetLedger {
    #[must_use]
    pub const fn new(limits: BudgetLimits) -> Self {
        Self {
            limits,
            usage: BudgetUsage {
                messages: 0,
                tool_calls: 0,
                tokens: 0,
            },
        }
    }

    pub fn charge_message(&mut self) -> Result<(), BudgetError> {
        charge(
            &mut self.usage.messages,
            1,
            self.limits.messages,
            "messages",
        )
    }

    pub fn charge_tool_call(&mut self) -> Result<(), BudgetError> {
        charge(
            &mut self.usage.tool_calls,
            1,
            self.limits.tool_calls,
            "tool_calls",
        )
    }

    pub fn charge_tokens(&mut self, amount: u64) -> Result<(), BudgetError> {
        charge(&mut self.usage.tokens, amount, self.limits.tokens, "tokens")
    }

    #[must_use]
    pub const fn usage(&self) -> BudgetUsage {
        self.usage
    }
}

fn charge(
    value: &mut u64,
    amount: u64,
    limit: u64,
    resource: &'static str,
) -> Result<(), BudgetError> {
    let next = value
        .checked_add(amount)
        .ok_or(BudgetError::Overflow { resource })?;
    if next > limit {
        return Err(BudgetError::Exceeded { resource, limit });
    }
    *value = next;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BudgetError {
    #[error("{resource} budget exceeded (limit {limit})")]
    Exceeded { resource: &'static str, limit: u64 },
    #[error("{resource} usage overflow")]
    Overflow { resource: &'static str },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_charge_without_mutating_usage() {
        let mut ledger = BudgetLedger::new(BudgetLimits {
            messages: 1,
            tool_calls: 0,
            tokens: 2,
        });
        assert!(ledger.charge_message().is_ok());
        assert!(matches!(
            ledger.charge_message(),
            Err(BudgetError::Exceeded { .. })
        ));
        assert_eq!(ledger.usage().messages, 1);
        assert!(ledger.charge_tokens(3).is_err());
        assert_eq!(ledger.usage().tokens, 0);
    }
}
