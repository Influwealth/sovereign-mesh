use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    pub version: u32,
    pub capsule: String,
    pub default_decision: String,
    pub caller_methods: BTreeMap<String, Vec<String>>,
    pub escalation_methods: Vec<String>,
    pub mutation_methods: Vec<String>,
    pub query_methods: Vec<String>,
    pub quantum_methods: Vec<String>,
    pub compliance: Option<String>,
    pub require_audit: bool,
    pub allow_probabilistic_quantum: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyLoadError {
    pub message: String,
}

impl PolicyLoadError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

pub fn parse_policy(input: &str) -> Result<Policy, PolicyLoadError> {
    let mut policy = Policy {
        version: 0,
        capsule: String::new(),
        default_decision: String::new(),
        caller_methods: BTreeMap::new(),
        escalation_methods: Vec::new(),
        mutation_methods: Vec::new(),
        query_methods: Vec::new(),
        quantum_methods: Vec::new(),
        compliance: None,
        require_audit: false,
        allow_probabilistic_quantum: false,
    };

    let lines: Vec<&str> = input.lines().collect();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index].trim();
        if line.is_empty() || line.starts_with('#') {
            index += 1;
            continue;
        }

        if let Some(value) = line.strip_prefix("version:") {
            policy.version = value.trim().parse().unwrap_or(0);
        } else if let Some(value) = line.strip_prefix("capsule:") {
            policy.capsule = value.trim().to_string();
        } else if let Some(value) = line.strip_prefix("default_decision:") {
            policy.default_decision = value.trim().to_string();
        } else if line == "callers:" {
            index = parse_callers(&lines, index + 1, &mut policy.caller_methods);
            continue;
        } else if line == "escalation:" {
            index = parse_named_methods(&lines, index + 1, "methods:", &mut policy.escalation_methods);
            continue;
        } else if line == "mutation:" {
            index = parse_named_methods(&lines, index + 1, "allowed_methods:", &mut policy.mutation_methods);
            continue;
        } else if line == "queries:" {
            index = parse_named_methods(&lines, index + 1, "allowed_methods:", &mut policy.query_methods);
            continue;
        } else if line == "quantum:" {
            index = parse_named_methods(&lines, index + 1, "allowed_methods:", &mut policy.quantum_methods);
            continue;
        } else if line == "risk:" {
            index = parse_risk(&lines, index + 1, &mut policy);
            continue;
        } else if line == "compliance:" {
            policy.compliance = Some("placeholder".to_string());
        } else if let Some(value) = line.strip_prefix("placeholder:") {
            policy.compliance = Some(value.trim().trim_matches('"').to_string());
        }

        index += 1;
    }

    if policy.capsule.is_empty() {
        return Err(PolicyLoadError::new("policy capsule is required"));
    }
    if policy.default_decision.is_empty() {
        return Err(PolicyLoadError::new("policy default_decision is required"));
    }

    Ok(policy)
}

fn parse_callers(
    lines: &[&str],
    mut index: usize,
    caller_methods: &mut BTreeMap<String, Vec<String>>,
) -> usize {
    let mut current_caller: Option<String> = None;
    while index < lines.len() {
        let raw = lines[index];
        let line = raw.trim();
        if line.is_empty() {
            index += 1;
            continue;
        }
        if !raw.starts_with(' ') {
            break;
        }
        if raw.starts_with("  ") && !raw.starts_with("    ") && line.ends_with(':') {
            let caller = line.trim_end_matches(':').to_string();
            caller_methods.entry(caller.clone()).or_default();
            current_caller = Some(caller);
        } else if line.starts_with("- ") {
            if let Some(caller) = &current_caller {
                caller_methods
                    .entry(caller.clone())
                    .or_default()
                    .push(line.trim_start_matches("- ").to_string());
            }
        }
        index += 1;
    }
    index
}

fn parse_named_methods(
    lines: &[&str],
    mut index: usize,
    marker: &str,
    methods: &mut Vec<String>,
) -> usize {
    let mut in_methods = false;
    while index < lines.len() {
        let raw = lines[index];
        let line = raw.trim();
        if line.is_empty() {
            index += 1;
            continue;
        }
        if !raw.starts_with(' ') {
            break;
        }
        if line == marker {
            in_methods = true;
        } else if in_methods && line.starts_with("- ") {
            methods.push(line.trim_start_matches("- ").to_string());
        } else if in_methods && !line.starts_with("- ") {
            in_methods = false;
        }
        index += 1;
    }
    index
}

fn parse_risk(lines: &[&str], mut index: usize, policy: &mut Policy) -> usize {
    while index < lines.len() {
        let raw = lines[index];
        let line = raw.trim();
        if line.is_empty() {
            index += 1;
            continue;
        }
        if !raw.starts_with(' ') {
            break;
        }
        if let Some(value) = line.strip_prefix("require_audit:") {
            policy.require_audit = parse_bool(value);
        } else if let Some(value) = line.strip_prefix("allow_probabilistic_quantum:") {
            policy.allow_probabilistic_quantum = parse_bool(value);
        }
        index += 1;
    }
    index
}

fn parse_bool(value: &str) -> bool {
    matches!(value.trim(), "true" | "True" | "TRUE")
}

#[cfg(test)]
mod tests {
    use super::parse_policy;

    #[test]
    fn parses_prediction_policy() {
        let input = include_str!("../../sdk/examples/prediction_capsule/policy.yaml");
        let policy = parse_policy(input).expect("policy should parse");

        assert_eq!(policy.version, 1);
        assert_eq!(policy.capsule, "capsule.prediction.v1");
        assert_eq!(policy.default_decision, "deny");
        assert_eq!(policy.mutation_methods, vec!["predict"]);
        assert_eq!(policy.query_methods, vec!["health"]);
        assert_eq!(policy.quantum_methods, vec!["predict"]);
        assert!(policy.require_audit);
        assert!(!policy.allow_probabilistic_quantum);
    }
}
