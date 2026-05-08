#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeConfig {
    pub deepflex_endpoint: String,
    pub metrics_port: u16,
    pub resource_classes: Vec<ResourceClass>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResourceClass {
    Cpu,
    Gpu,
    Quantum,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            deepflex_endpoint: std::env::var("DEEPFLEX_ENDPOINT")
                .unwrap_or_else(|_| "http://127.0.0.1:8181".to_string()),
            metrics_port: std::env::var("SOVEREIGN_MESH_METRICS_PORT")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(9095),
            resource_classes: vec![ResourceClass::Cpu],
        }
    }
}
