use crate::ingress::IngressMessage;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionPriority {
    Low,
    Normal,
    High,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduleDecision {
    pub priority: ExecutionPriority,
    pub lane: String,
}

pub trait Scheduler {
    fn schedule(&self, ingress: &IngressMessage) -> ScheduleDecision;
}

#[derive(Clone, Debug, Default)]
pub struct DeterministicScheduler;

impl Scheduler for DeterministicScheduler {
    fn schedule(&self, ingress: &IngressMessage) -> ScheduleDecision {
        let priority = if ingress.method.starts_with("query_") {
            ExecutionPriority::Low
        } else if ingress.method.starts_with("govern_") {
            ExecutionPriority::High
        } else {
            ExecutionPriority::Normal
        };

        ScheduleDecision {
            priority,
            lane: "default-sovereign-lane".to_string(),
        }
    }
}
