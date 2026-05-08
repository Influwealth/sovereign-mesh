use sovereign_mesh_runtime::{EdgeCompatibility, Request, SubsidyClass};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let output = render_cli(&args);
    println!("{}", output);
}

pub fn render_cli(args: &[String]) -> String {
    let json = args.iter().any(|arg| arg == "--json");
    let command = args
        .iter()
        .find(|arg| !arg.starts_with("--"))
        .map(String::as_str)
        .unwrap_or("status");

    match command {
        "status" => render_status(json),
        "graph" => render_graph(json),
        "schedule" => render_schedule(json),
        "capsule" => {
            let id = args
                .iter()
                .skip_while(|arg| arg.as_str() != "capsule")
                .nth(1)
                .map(String::as_str)
                .unwrap_or("unknown");
            render_capsule(id, json)
        }
        "governance" => render_governance(json),
        _ => render_help(json),
    }
}

fn render_status(json: bool) -> String {
    if json {
        return "{\"status\":\"ok\",\"runtime\":\"sovereign-mesh\",\"mode\":\"local\"}".to_string();
    }

    table(&[
        ("Runtime", "sovereign-mesh"),
        ("Status", "ok"),
        ("Mode", "local"),
    ])
}

fn render_graph(json: bool) -> String {
    if json {
        return "{\"graph\":\"placeholder\",\"capsules\":[\"prediction\",\"dames_legacy_account\",\"governance_compliance\"]}".to_string();
    }

    table(&[
        ("Graph", "placeholder"),
        ("Capsules", "prediction,dames_legacy_account,governance_compliance"),
    ])
}

fn render_schedule(json: bool) -> String {
    let req = Request::new(
        "meshctl-trace",
        "meshctl",
        "capsule.meshctl.v1",
        "status",
        Vec::new(),
    )
    .with_scheduling(SubsidyClass::Standard, EdgeCompatibility::Eligible, None);
    let decision = sovereign_mesh_runtime::schedule(&req)
        .expect("local placeholder scheduling should not fail");

    if json {
        return format!(
            "{{\"lane\":\"{}\",\"edge_eligible\":{}}}",
            decision.lane, decision.edge_eligible
        );
    }

    table(&[
        ("Lane", decision.lane.as_str()),
        ("Edge Eligible", bool_text(decision.edge_eligible)),
    ])
}

fn render_capsule(id: &str, json: bool) -> String {
    if json {
        return format!(
            "{{\"capsule_id\":\"{}\",\"status\":\"registered-placeholder\"}}",
            id
        );
    }

    table(&[("Capsule", id), ("Status", "registered-placeholder")])
}

fn render_governance(json: bool) -> String {
    if json {
        return "{\"governance\":\"placeholder\",\"role\":\"governance-admin\"}".to_string();
    }

    table(&[
        ("Governance", "placeholder"),
        ("Required Role", "governance-admin"),
    ])
}

fn render_help(json: bool) -> String {
    if json {
        return "{\"commands\":[\"status\",\"graph\",\"schedule\",\"capsule <id>\",\"governance\"]}"
            .to_string();
    }

    "Commands: status | graph | schedule | capsule <id> | governance".to_string()
}

fn table(rows: &[(&str, &str)]) -> String {
    let mut output = String::from("FIELD                 VALUE\n");
    output.push_str("--------------------  ------------------------------\n");
    for (field, value) in rows {
        output.push_str(&format!("{:<20}  {}\n", field, value));
    }
    output.trim_end().to_string()
}

fn bool_text(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

#[cfg(test)]
mod tests {
    use super::render_cli;

    #[test]
    fn status_human_snapshot() {
        let output = render_cli(&["status".to_string()]);
        assert!(output.contains("Runtime"));
        assert!(output.contains("sovereign-mesh"));
    }

    #[test]
    fn schedule_json_snapshot() {
        let output = render_cli(&["schedule".to_string(), "--json".to_string()]);
        assert_eq!(output, "{\"lane\":\"standard\",\"edge_eligible\":true}");
    }

    #[test]
    fn capsule_human_snapshot() {
        let output = render_cli(&[
            "capsule".to_string(),
            "capsule.prediction.v1".to_string(),
        ]);
        assert!(output.contains("capsule.prediction.v1"));
    }
}
