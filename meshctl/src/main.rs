use sovereign_mesh_runtime::{EdgeCompatibility, Request, SubsidyClass};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("mcp") {
        run_mcp_server();
        return;
    }

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
        "metrics" => render_metrics(json),
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

fn render_metrics(json: bool) -> String {
    if json {
        return "{\"scheduler_decisions_total\":0,\"capsule_executions_total\":0,\"pending_requests\":0}".to_string();
    }

    table(&[
        ("Scheduler Decisions", "0"),
        ("Capsule Executions", "0"),
        ("Pending Requests", "0"),
    ])
}

fn render_help(json: bool) -> String {
    if json {
        return "{\"commands\":[\"status\",\"graph\",\"schedule\",\"capsule <id>\",\"governance\",\"metrics\"]}"
            .to_string();
    }

    "Commands: status | graph | schedule | capsule <id> | governance | metrics".to_string()
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

fn run_mcp_server() {
    use std::io::{self, BufRead, Write};

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let Ok(line) = line else {
            break;
        };
        if line.trim().is_empty() {
            continue;
        }

        let response = handle_mcp_request(&line);
        let _ = writeln!(stdout, "{}", response);
        let _ = stdout.flush();
    }
}

fn handle_mcp_request(input: &str) -> String {
    let id = json_field(input, "id").unwrap_or_else(|| "null".to_string());
    if input.contains("\"method\":\"initialize\"") {
        return format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":{},\"result\":{{\"protocolVersion\":\"2024-11-05\",\"serverInfo\":{{\"name\":\"sovereign_mesh\",\"version\":\"0.1.0\"}},\"capabilities\":{{\"tools\":{{}}}}}}}}",
            id
        );
    }
    if input.contains("\"method\":\"tools/list\"") {
        return format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":{},\"result\":{{\"tools\":[{}]}}}}",
            id,
            [
                tool_json("meshctl_status", "Return Sovereign Mesh operator status."),
                tool_json("meshctl_graph", "Return capsule graph overview."),
                tool_json("meshctl_schedule", "Return deterministic scheduler lane decision."),
                tool_json("meshctl_capsule", "Inspect one capsule by id."),
                tool_json("meshctl_governance", "Return governance operator status."),
                tool_json("meshctl_metrics", "Return runtime metric summary.")
            ]
            .join(",")
        );
    }
    if input.contains("\"method\":\"tools/call\"") {
        let name = json_string_field(input, "name").unwrap_or_default();
        let text = match name.as_str() {
            "meshctl_status" => render_status(false),
            "meshctl_graph" => render_graph(false),
            "meshctl_schedule" => render_schedule(false),
            "meshctl_capsule" => render_capsule("capsule.unknown", false),
            "meshctl_governance" => render_governance(false),
            "meshctl_metrics" => render_metrics(false),
            _ => "unknown meshctl MCP tool".to_string(),
        };
        return format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":{},\"result\":{{\"content\":[{{\"type\":\"text\",\"text\":\"{}\"}}]}}}}",
            id,
            json_escape(&text)
        );
    }

    format!(
        "{{\"jsonrpc\":\"2.0\",\"id\":{},\"result\":{{\"content\":[{{\"type\":\"text\",\"text\":\"unsupported MCP request\"}}]}}}}",
        id
    )
}

fn tool_json(name: &str, description: &str) -> String {
    format!(
        "{{\"name\":\"{}\",\"description\":\"{}\",\"inputSchema\":{{\"type\":\"object\",\"properties\":{{}}}}}}",
        name, description
    )
}

fn json_field(input: &str, field: &str) -> Option<String> {
    let needle = format!("\"{}\":", field);
    let start = input.find(&needle)? + needle.len();
    let rest = &input[start..];
    let end = rest.find([',', '}']).unwrap_or(rest.len());
    Some(rest[..end].trim().to_string())
}

fn json_string_field(input: &str, field: &str) -> Option<String> {
    let needle = format!("\"{}\":\"", field);
    let start = input.find(&needle)? + needle.len();
    let rest = &input[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn json_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
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

    #[test]
    fn metrics_json_snapshot() {
        let output = render_cli(&["metrics".to_string(), "--json".to_string()]);
        assert!(output.contains("scheduler_decisions_total"));
    }
}
