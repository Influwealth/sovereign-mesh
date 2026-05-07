use crate::ingress::IngressMessage;
use crate::{EdgeCompatibility, ExecutionError, Request, SubsidyClass};

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
    pub edge_eligible: bool,
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
            edge_eligible: false,
        }
    }
}

pub fn schedule(req: &Request) -> Result<ScheduleDecision, ExecutionError> {
    // TODO: add locality-aware placement once runtime nodes publish capabilities.
    // TODO: account for smart-city data proximity when capsules touch municipal signals.
    // TODO: add multi-node load balancing across trusted sovereign mesh lanes.
    let lane = match req.subsidy_class {
        SubsidyClass::Youth => "youth-priority",
        SubsidyClass::Underserved => "underserved-priority",
        SubsidyClass::Standard => "standard",
        SubsidyClass::Enterprise => "enterprise",
    }
    .to_string();

    let priority = match req.subsidy_class {
        SubsidyClass::Youth | SubsidyClass::Underserved => ExecutionPriority::High,
        SubsidyClass::Standard => ExecutionPriority::Normal,
        SubsidyClass::Enterprise => ExecutionPriority::Normal,
    };

    Ok(ScheduleDecision {
        priority,
        lane,
        edge_eligible: matches!(req.edge_compatibility, EdgeCompatibility::Eligible),
    })
}

#[cfg(test)]
mod tests {
    use super::schedule;
    use crate::{EdgeCompatibility, Request, SubsidyClass};

    fn request_for(subsidy_class: SubsidyClass, edge_compatibility: EdgeCompatibility) -> Request {
        Request::new(
            "trace-test",
            "agent-test",
            "capsule.test.v1",
            "run",
            Vec::new(),
        )
        .with_scheduling(subsidy_class, edge_compatibility, None)
    }

    #[test]
    fn youth_uses_youth_priority_lane() {
        let decision = schedule(&request_for(
            SubsidyClass::Youth,
            EdgeCompatibility::NotEligible,
        ))
        .expect("schedule should succeed");

        assert_eq!(decision.lane, "youth-priority");
        assert!(!decision.edge_eligible);
    }

    #[test]
    fn standard_uses_standard_lane() {
        let decision = schedule(&request_for(
            SubsidyClass::Standard,
            EdgeCompatibility::NotEligible,
        ))
        .expect("schedule should succeed");

        assert_eq!(decision.lane, "standard");
        assert!(!decision.edge_eligible);
    }

    #[test]
    fn enterprise_uses_enterprise_lane() {
        let decision = schedule(&request_for(
            SubsidyClass::Enterprise,
            EdgeCompatibility::NotEligible,
        ))
        .expect("schedule should succeed");

        assert_eq!(decision.lane, "enterprise");
    }

    #[test]
    fn edge_compatible_request_is_edge_eligible() {
        let decision = schedule(&request_for(
            SubsidyClass::Standard,
            EdgeCompatibility::Eligible,
        ))
        .expect("schedule should succeed");

        assert!(decision.edge_eligible);
    }
}
