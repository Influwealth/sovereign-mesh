use sovereign_mesh_sdk::{
    capsule, emit_event, query, update, CapsuleResponse, KeyValueStableState, StableState,
};

#[capsule]
pub mod dames_legacy_account {
    use super::*;

    const PROFILE_PREFIX: &str = "youth_profile:";

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct YouthProfile {
        pub youth_id: String,
        pub display_name: String,
        pub guardian_contact: String,
        pub community: String,
        pub updated_at: u64,
    }

    impl YouthProfile {
        pub fn new(
            youth_id: impl Into<String>,
            display_name: impl Into<String>,
            guardian_contact: impl Into<String>,
            community: impl Into<String>,
            updated_at: u64,
        ) -> Self {
            Self {
                youth_id: youth_id.into(),
                display_name: display_name.into(),
                guardian_contact: guardian_contact.into(),
                community: community.into(),
                updated_at,
            }
        }

        fn encode(&self) -> Vec<u8> {
            format!(
                "{}|{}|{}|{}|{}",
                self.youth_id,
                self.display_name,
                self.guardian_contact,
                self.community,
                self.updated_at
            )
            .into_bytes()
        }

        fn decode(bytes: &[u8]) -> Option<Self> {
            let raw = core::str::from_utf8(bytes).ok()?;
            let parts: Vec<&str> = raw.split('|').collect();
            if parts.len() != 5 {
                return None;
            }

            Some(Self {
                youth_id: parts[0].to_string(),
                display_name: parts[1].to_string(),
                guardian_contact: parts[2].to_string(),
                community: parts[3].to_string(),
                updated_at: parts[4].parse().ok()?,
            })
        }

        fn to_json(&self) -> String {
            format!(
                "{{\"youth_id\":\"{}\",\"display_name\":\"{}\",\"guardian_contact\":\"{}\",\"community\":\"{}\",\"updated_at\":{}}}",
                self.youth_id,
                self.display_name,
                self.guardian_contact,
                self.community,
                self.updated_at
            )
        }
    }

    #[derive(Clone, Debug, Default)]
    pub struct DamesLegacyAccountCapsule {
        stable_state: KeyValueStableState,
    }

    impl DamesLegacyAccountCapsule {
        pub fn new() -> Self {
            Self {
                stable_state: KeyValueStableState::new(),
            }
        }

        #[update]
        pub fn register_youth(&mut self, profile: YouthProfile) -> CapsuleResponse {
            let key = profile_key(&profile.youth_id);
            if self.stable_state.get(&key).is_some() {
                return CapsuleResponse::rejected("youth profile already exists");
            }

            self.stable_state.set(&key, profile.encode());
            let _event = emit_event("youth.registered", &profile.to_json());

            CapsuleResponse::ok(profile.to_json().into_bytes())
        }

        #[update]
        pub fn update_profile(&mut self, profile: YouthProfile) -> CapsuleResponse {
            let key = profile_key(&profile.youth_id);
            if self.stable_state.get(&key).is_none() {
                return CapsuleResponse::rejected("youth profile does not exist");
            }

            self.stable_state.set(&key, profile.encode());
            let _event = emit_event("youth.updated", &profile.to_json());

            CapsuleResponse::ok(profile.to_json().into_bytes())
        }

        #[query]
        pub fn get_profile(&self, youth_id: &str) -> CapsuleResponse {
            match self
                .stable_state
                .get(&profile_key(youth_id))
                .and_then(|bytes| YouthProfile::decode(&bytes))
            {
                Some(profile) => CapsuleResponse::ok(profile.to_json().into_bytes()),
                None => CapsuleResponse::rejected("youth profile not found"),
            }
        }
    }

    pub fn profile_key(youth_id: &str) -> String {
        format!("{}{}", PROFILE_PREFIX, youth_id)
    }

    pub fn capsule_update_entry(
        capsule: &mut DamesLegacyAccountCapsule,
        method: &str,
        profile: YouthProfile,
    ) -> CapsuleResponse {
        // TODO: replace typed local dispatch with generated WASM ABI routing.
        match method {
            "register_youth" => capsule.register_youth(profile),
            "update_profile" => capsule.update_profile(profile),
            _ => CapsuleResponse::rejected("unsupported update method"),
        }
    }

    pub fn capsule_query_entry(
        capsule: &DamesLegacyAccountCapsule,
        method: &str,
        youth_id: &str,
    ) -> CapsuleResponse {
        // TODO: let the runtime authorize query visibility through policy.yaml.
        match method {
            "get_profile" => capsule.get_profile(youth_id),
            _ => CapsuleResponse::rejected("unsupported query method"),
        }
    }
}
