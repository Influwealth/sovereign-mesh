use crate::CapsuleId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapsuleGraph {
    pub capsule_id: CapsuleId,
    pub nodes: Vec<CapsuleNode>,
    pub edges: Vec<CapsuleEdge>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapsuleNode {
    pub id: String,
    pub node_type: String,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapsuleEdge {
    pub from: String,
    pub to: String,
    pub relation: String,
}

pub fn load_graph_for_capsule(_id: &CapsuleId) -> Option<CapsuleGraph> {
    // TODO: parse graph.yaml for the requested capsule.
    // TODO: expose dependencies to scheduler placement and policy checks.
    // TODO: validate graph signatures before trusting mesh routing hints.
    None
}
