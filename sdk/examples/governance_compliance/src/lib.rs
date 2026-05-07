use sovereign_mesh_sdk::{
    capsule, emit_event, query, update, CapsuleResponse, KeyValueStableState, StableState,
};

#[capsule]
pub mod governance_compliance {
    use super::*;

    const SUBSIDY_PREFIX: &str = "governance:subsidy_class:";
    const COMPLIANCE_PREFIX: &str = "governance:compliance_flag:";

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct GovernanceState {
        pub capsule_id: String,
        pub subsidy_class: Option<String>,
        pub requires_manual_review: bool,
    }

    impl GovernanceState {
        fn to_json(&self) -> String {
            let subsidy = self
                .subsidy_class
                .clone()
                .unwrap_or_else(|| "unset".to_string());
            format!(
                "{{\"capsule_id\":\"{}\",\"subsidy_class\":\"{}\",\"requires_manual_review\":{}}}",
                self.capsule_id, subsidy, self.requires_manual_review
            )
        }
    }

    #[derive(Clone, Debug, Default)]
    pub struct GovernanceComplianceCapsule {
        stable_state: KeyValueStableState,
    }

    impl GovernanceComplianceCapsule {
        pub fn new() -> Self {
            Self {
                stable_state: KeyValueStableState::new(),
            }
        }

        #[update]
        pub fn set_subsidy_class_for_capsule(
            &mut self,
            capsule_id: &str,
            subsidy_class: &str,
        ) -> CapsuleResponse {
            self.stable_state.set(
                &subsidy_key(capsule_id),
                subsidy_class.as_bytes().to_vec(),
            );

            let payload = format!(
                "{{\"capsule_id\":\"{}\",\"subsidy_class\":\"{}\"}}",
                capsule_id, subsidy_class
            );
            let _event = emit_event("governance.subsidy_class_updated", &payload);

            CapsuleResponse::ok(payload.into_bytes())
        }

        #[update]
        pub fn set_compliance_flag_for_capsule(
            &mut self,
            capsule_id: &str,
            requires_manual_review: bool,
        ) -> CapsuleResponse {
            self.stable_state.set(
                &compliance_key(capsule_id),
                requires_manual_review.to_string().into_bytes(),
            );

            let payload = format!(
                "{{\"capsule_id\":\"{}\",\"requires_manual_review\":{}}}",
                capsule_id, requires_manual_review
            );
            let _event = emit_event("governance.compliance_flag_updated", &payload);

            CapsuleResponse::ok(payload.into_bytes())
        }

        #[query]
        pub fn get_capsule_governance_state(&self, capsule_id: &str) -> CapsuleResponse {
            let subsidy_class = self
                .stable_state
                .get(&subsidy_key(capsule_id))
                .and_then(|bytes| String::from_utf8(bytes).ok());
            let requires_manual_review = self
                .stable_state
                .get(&compliance_key(capsule_id))
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .map(|value| value == "true")
                .unwrap_or(false);

            let state = GovernanceState {
                capsule_id: capsule_id.to_string(),
                subsidy_class,
                requires_manual_review,
            };

            CapsuleResponse::ok(state.to_json().into_bytes())
        }
    }

    pub fn subsidy_key(capsule_id: &str) -> String {
        format!("{}{}", SUBSIDY_PREFIX, capsule_id)
    }

    pub fn compliance_key(capsule_id: &str) -> String {
        format!("{}{}", COMPLIANCE_PREFIX, capsule_id)
    }

    pub fn capsule_update_entry(
        capsule: &mut GovernanceComplianceCapsule,
        method: &str,
        capsule_id: &str,
        value: &str,
    ) -> CapsuleResponse {
        // TODO: enforce governance role at runtime before dispatching these updates.
        match method {
            "set_subsidy_class_for_capsule" => {
                capsule.set_subsidy_class_for_capsule(capsule_id, value)
            }
            "set_compliance_flag_for_capsule" => {
                capsule.set_compliance_flag_for_capsule(capsule_id, value == "true")
            }
            _ => CapsuleResponse::rejected("unsupported governance update method"),
        }
    }

    pub fn capsule_query_entry(
        capsule: &GovernanceComplianceCapsule,
        method: &str,
        capsule_id: &str,
    ) -> CapsuleResponse {
        match method {
            "get_capsule_governance_state" => {
                capsule.get_capsule_governance_state(capsule_id)
            }
            _ => CapsuleResponse::rejected("unsupported governance query method"),
        }
    }
}
