use hunteval_resilience::{FaultEvent, FaultSchedule};

/// Deterministic cursor over a precomputed logical fault schedule.
#[derive(Debug, Clone)]
pub struct FaultController {
    events: Vec<FaultEvent>,
    next: usize,
}

impl FaultController {
    #[must_use]
    pub fn new(mut schedule: FaultSchedule) -> Self {
        schedule
            .events
            .sort_by_key(|event| (event.logical_sequence, event.attempt));
        Self {
            events: schedule.events,
            next: 0,
        }
    }

    /// Consume faults scheduled at this boundary. Skipped boundaries are discarded.
    pub fn at_boundary(&mut self, sequence: u64) -> Vec<FaultEvent> {
        while self
            .events
            .get(self.next)
            .is_some_and(|event| event.logical_sequence < sequence)
        {
            self.next += 1;
        }
        let start = self.next;
        while self
            .events
            .get(self.next)
            .is_some_and(|event| event.logical_sequence == sequence)
        {
            self.next += 1;
        }
        self.events[start..self.next].to_vec()
    }
}
