//! Metric Exporter
//!
//! Export metrics to various formats.

use crate::telemetry::metrics::MetricSnapshot;

/// Export format
#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Json,
    Prometheus,
    Csv,
}

/// Metric exporter
pub struct MetricExporter;

impl MetricExporter {
    /// Export snapshot to string
    pub fn export(&self, snapshot: &MetricSnapshot, format: ExportFormat) -> String {
        match format {
            ExportFormat::Json => self.to_json(snapshot),
            ExportFormat::Prometheus => self.to_prometheus(snapshot),
            ExportFormat::Csv => self.to_csv(snapshot),
        }
    }

    fn to_json(&self, snapshot: &MetricSnapshot) -> String {
        format!(
            r#"{{"timestamp":{},"escape_rate":{:.4},"efficiency":{:.4},"avg_latency_ms":{},"total_tokens":{},"total_generations":{}}"#,
            snapshot.timestamp,
            snapshot.escape_rate,
            snapshot.efficiency,
            snapshot.avg_latency_ms,
            snapshot.total_tokens,
            snapshot.total_generations
        )
    }

    fn to_prometheus(&self, snapshot: &MetricSnapshot) -> String {
        format!(
            r#"# HELP gzmo_escape_rate Fraction of stuck patterns escaped
# TYPE gzmo_escape_rate gauge
gzmo_escape_rate {:.4}
# HELP gzmo_efficiency Quality per cost unit
# TYPE gzmo_efficiency gauge
gzmo_efficiency {:.4}
# HELP gzmo_avg_latency_ms Average generation latency
# TYPE gzmo_avg_latency_ms gauge
gzmo_avg_latency_ms {}
"#,
            snapshot.escape_rate,
            snapshot.efficiency,
            snapshot.avg_latency_ms
        )
    }

    fn to_csv(&self, snapshot: &MetricSnapshot) -> String {
        format!(
            "timestamp,escape_rate,efficiency,avg_latency_ms,total_tokens,total_generations\n{},{:.4},{:.4},{},{},{}\n",
            snapshot.timestamp,
            snapshot.escape_rate,
            snapshot.efficiency,
            snapshot.avg_latency_ms,
            snapshot.total_tokens,
            snapshot.total_generations
        )
    }
}

impl Default for MetricExporter {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_snapshot() -> MetricSnapshot {
        MetricSnapshot {
            timestamp: 12345,
            temperature_diversity_correlation: 0.5,
            escape_rate: 0.8,
            efficiency: 12.5,
            avg_latency_ms: 500,
            total_tokens: 10000,
            total_generations: 100,
        }
    }

    #[test]
    fn json_export_valid() {
        let exporter = MetricExporter::default();
        let json = exporter.export(&make_snapshot(), ExportFormat::Json);
        assert!(json.contains("timestamp"));
        assert!(json.contains("escape_rate"));
    }

    #[test]
    fn prometheus_export_valid() {
        let exporter = MetricExporter::default();
        let prom = exporter.export(&make_snapshot(), ExportFormat::Prometheus);
        assert!(prom.contains("# HELP"));
        assert!(prom.contains("gzmo_escape_rate"));
    }

    #[test]
    fn csv_export_valid() {
        let exporter = MetricExporter::default();
        let csv = exporter.export(&make_snapshot(), ExportFormat::Csv);
        let lines: Vec<_> = csv.lines().collect();
        assert_eq!(lines.len(), 2); // header + data
    }
}