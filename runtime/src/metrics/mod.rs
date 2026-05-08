use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{TcpListener, ToSocketAddrs};
use std::sync::{Mutex, OnceLock};

static GLOBAL_METRICS: OnceLock<MetricsRegistry> = OnceLock::new();

pub fn global_metrics() -> &'static MetricsRegistry {
    GLOBAL_METRICS.get_or_init(MetricsRegistry::default)
}

#[derive(Debug, Default)]
pub struct MetricsRegistry {
    scheduler_decisions_total: Mutex<BTreeMap<String, u64>>,
    capsule_executions_total: Mutex<BTreeMap<(String, String), u64>>,
    capsule_execution_duration_seconds: Mutex<BTreeMap<String, Vec<f64>>>,
    pending_requests: Mutex<BTreeMap<String, i64>>,
    recent_executions: Mutex<Vec<String>>,
}

impl MetricsRegistry {
    pub fn scheduler_decision(&self, lane: &str) {
        increment_string_counter(&self.scheduler_decisions_total, lane);
    }

    pub fn capsule_execution(&self, capsule_id: &str, result: &str) {
        if let Ok(mut counters) = self.capsule_executions_total.lock() {
            let key = (capsule_id.to_string(), result.to_string());
            *counters.entry(key).or_insert(0) += 1;
        }
        if let Ok(mut recent) = self.recent_executions.lock() {
            recent.push(format!("capsule_id={} result={}", capsule_id, result));
            if recent.len() > 50 {
                recent.remove(0);
            }
        }
    }

    pub fn observe_execution_duration(&self, capsule_id: &str, seconds: f64) {
        if let Ok(mut histograms) = self.capsule_execution_duration_seconds.lock() {
            histograms
                .entry(capsule_id.to_string())
                .or_default()
                .push(seconds);
        }
    }

    pub fn set_pending_requests(&self, lane: &str, value: i64) {
        if let Ok(mut gauges) = self.pending_requests.lock() {
            gauges.insert(lane.to_string(), value);
        }
    }

    pub fn recent_executions(&self) -> Vec<String> {
        self.recent_executions
            .lock()
            .map(|recent| recent.clone())
            .unwrap_or_default()
    }

    pub fn render_prometheus(&self) -> String {
        let mut output = String::new();
        output.push_str("# TYPE scheduler_decisions_total counter\n");
        if let Ok(counters) = self.scheduler_decisions_total.lock() {
            for (lane, value) in counters.iter() {
                output.push_str(&format!(
                    "scheduler_decisions_total{{lane=\"{}\"}} {}\n",
                    lane, value
                ));
            }
        }

        output.push_str("# TYPE capsule_executions_total counter\n");
        if let Ok(counters) = self.capsule_executions_total.lock() {
            for ((capsule_id, result), value) in counters.iter() {
                output.push_str(&format!(
                    "capsule_executions_total{{capsule_id=\"{}\",result=\"{}\"}} {}\n",
                    capsule_id, result, value
                ));
            }
        }

        output.push_str("# TYPE capsule_execution_duration_seconds histogram\n");
        if let Ok(histograms) = self.capsule_execution_duration_seconds.lock() {
            for (capsule_id, values) in histograms.iter() {
                let sum: f64 = values.iter().sum();
                output.push_str(&format!(
                    "capsule_execution_duration_seconds_count{{capsule_id=\"{}\"}} {}\n",
                    capsule_id,
                    values.len()
                ));
                output.push_str(&format!(
                    "capsule_execution_duration_seconds_sum{{capsule_id=\"{}\"}} {}\n",
                    capsule_id, sum
                ));
            }
        }

        output.push_str("# TYPE pending_requests gauge\n");
        if let Ok(gauges) = self.pending_requests.lock() {
            for (lane, value) in gauges.iter() {
                output.push_str(&format!(
                    "pending_requests{{lane=\"{}\"}} {}\n",
                    lane, value
                ));
            }
        }

        output
    }
}

pub struct PrometheusExporter {
    pub port: u16,
}

impl PrometheusExporter {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub fn serve_once(&self, registry: &MetricsRegistry) -> std::io::Result<()> {
        let addr = ("127.0.0.1", self.port)
            .to_socket_addrs()?
            .next()
            .expect("loopback address should resolve");
        let listener = TcpListener::bind(addr)?;
        let (mut stream, _) = listener.accept()?;
        let mut buffer = [0_u8; 1024];
        let _ = stream.read(&mut buffer)?;
        let body = registry.render_prometheus();
        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: text/plain; version=0.0.4\r\ncontent-length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes())?;
        stream.flush()
    }
}

pub fn execution_started(capsule_id: &str) {
    global_metrics().capsule_execution(capsule_id, "started");
}

pub fn execution_completed(capsule_id: &str) {
    global_metrics().capsule_execution(capsule_id, "completed");
}

pub fn execution_failed(capsule_id: &str) {
    global_metrics().capsule_execution(capsule_id, "failed");
}

fn increment_string_counter(counter: &Mutex<BTreeMap<String, u64>>, key: &str) {
    if let Ok(mut values) = counter.lock() {
        *values.entry(key.to_string()).or_insert(0) += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::MetricsRegistry;

    #[test]
    fn renders_prometheus_metrics() {
        let registry = MetricsRegistry::default();
        registry.scheduler_decision("standard");
        registry.capsule_execution("capsule.test.v1", "completed");
        registry.observe_execution_duration("capsule.test.v1", 0.5);
        registry.set_pending_requests("standard", 2);

        let rendered = registry.render_prometheus();

        assert!(rendered.contains("scheduler_decisions_total"));
        assert!(rendered.contains("capsule_executions_total"));
        assert!(rendered.contains("pending_requests"));
    }
}
