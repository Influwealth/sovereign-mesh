use crate::ingress::{CallerClass, IngressMessage};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolicyDecision {
    Allow,
    Deny(String),
    Escalate(String),
}

pub trait PolicyEngine {
    fn evaluate(&self, ingress: &IngressMessage) -> PolicyDecision;
}

#[derive(Clone, Debug)]
pub struct StaticPolicyEngine {
    pub allow_agents: bool,
    pub allow_partners: bool,
    pub allow_sovereign_users: bool,
    pub escalation_methods: Vec<String>,
}

impl Default for StaticPolicyEngine {
    fn default() -> Self {
        Self {
            allow_agents: true,
            allow_partners: false,
            allow_sovereign_users: true,
            escalation_methods: Vec::new(),
        }
    }
}

impl PolicyEngine for StaticPolicyEngine {
    fn evaluate(&self, ingress: &IngressMessage) -> PolicyDecision {
        if self.escalation_methods.iter().any(|method| method == &ingress.method) {
            return PolicyDecision::Escalate(format!(
                "method {} requires governance review",
                ingress.method
            ));
        }

        let allowed = match ingress.caller_class {
            CallerClass::Agent => self.allow_agents,
            CallerClass::Partner => self.allow_partners,
            CallerClass::SovereignUser => self.allow_sovereign_users,
        };

        if allowed {
            PolicyDecision::Allow
        } else {
            PolicyDecision::Deny(format!("caller class denied for method {}", ingress.method))
        }
    }
}
