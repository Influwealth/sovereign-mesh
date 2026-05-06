#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CallerClass {
    Agent,
    Partner,
    SovereignUser,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IngressMessage {
    pub trace_id: String,
    pub caller_id: String,
    pub caller_class: CallerClass,
    pub capsule_id: String,
    pub method: String,
    pub payload: Vec<u8>,
}

impl IngressMessage {
    pub fn validate(&self) -> Result<(), String> {
        if self.trace_id.trim().is_empty() {
            return Err("trace_id is required".to_string());
        }
        if self.caller_id.trim().is_empty() {
            return Err("caller_id is required".to_string());
        }
        if self.capsule_id.trim().is_empty() {
            return Err("capsule_id is required".to_string());
        }
        if self.method.trim().is_empty() {
            return Err("method is required".to_string());
        }
        Ok(())
    }
}
