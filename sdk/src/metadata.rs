#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapsuleMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub authors: Vec<String>,
    pub runtime: String,
    pub backend: Option<String>,
    pub wasm_target: String,
    pub wasm_module: String,
    pub updates: Vec<String>,
    pub queries: Vec<String>,
    pub state_schema: String,
    pub subsidy_class: Option<String>,
    pub quantum_enabled: bool,
    pub quantum_provider: String,
    pub quantum_model: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetadataError {
    pub message: String,
}

impl MetadataError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

pub fn parse_capsule_metadata(input: &str) -> Result<CapsuleMetadata, MetadataError> {
    let mut section = String::new();
    let mut metadata = CapsuleMetadata {
        id: String::new(),
        name: String::new(),
        version: String::new(),
        description: String::new(),
        authors: Vec::new(),
        runtime: String::new(),
        backend: None,
        wasm_target: String::new(),
        wasm_module: String::new(),
        updates: Vec::new(),
        queries: Vec::new(),
        state_schema: String::new(),
        subsidy_class: None,
        quantum_enabled: false,
        quantum_provider: String::new(),
        quantum_model: String::new(),
    };

    for raw_line in input.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            section = line.trim_matches(['[', ']']).to_string();
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();

        match (section.as_str(), key) {
            ("capsule", "id") => metadata.id = parse_string(value),
            ("capsule", "name") => metadata.name = parse_string(value),
            ("capsule", "version") => metadata.version = parse_string(value),
            ("capsule", "description") => metadata.description = parse_string(value),
            ("capsule", "authors") => metadata.authors = parse_string_array(value),
            ("capsule", "runtime") => metadata.runtime = parse_string(value),
            ("capsule", "backend") => metadata.backend = Some(parse_string(value)),
            ("capsule", "wasm_target") => metadata.wasm_target = parse_string(value),
            ("capsule", "wasm_module") => metadata.wasm_module = parse_string(value),
            ("entrypoints", "updates") => metadata.updates = parse_string_array(value),
            ("entrypoints", "queries") => metadata.queries = parse_string_array(value),
            ("state", "schema") => metadata.state_schema = parse_string(value),
            ("governance", "subsidy_class") => metadata.subsidy_class = Some(parse_string(value)),
            ("quantum", "enabled") => metadata.quantum_enabled = parse_bool(value),
            ("quantum", "provider") => metadata.quantum_provider = parse_string(value),
            ("quantum", "model") => metadata.quantum_model = parse_string(value),
            _ => {}
        }
    }

    if metadata.id.is_empty() {
        return Err(MetadataError::new("capsule.id is required"));
    }
    if metadata.name.is_empty() {
        return Err(MetadataError::new("capsule.name is required"));
    }
    if metadata.version.is_empty() {
        return Err(MetadataError::new("capsule.version is required"));
    }
    if metadata.wasm_module.is_empty() {
        return Err(MetadataError::new("capsule.wasm_module is required"));
    }

    Ok(metadata)
}

fn parse_string(value: &str) -> String {
    value.trim().trim_matches('"').to_string()
}

fn parse_bool(value: &str) -> bool {
    matches!(value.trim(), "true" | "True" | "TRUE")
}

fn parse_string_array(value: &str) -> Vec<String> {
    value
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(parse_string)
        .filter(|item| !item.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::parse_capsule_metadata;

    #[test]
    fn parses_prediction_capsule_metadata() {
        let input = include_str!("../examples/prediction_capsule/capsule.toml");
        let metadata = parse_capsule_metadata(input).expect("metadata should parse");

        assert_eq!(metadata.id, "capsule.prediction.v1");
        assert_eq!(metadata.name, "Prediction Capsule");
        assert_eq!(metadata.updates, vec!["predict"]);
        assert_eq!(metadata.queries, vec!["health"]);
        assert!(metadata.quantum_enabled);
        assert_eq!(metadata.quantum_provider, "mock");
    }
}
