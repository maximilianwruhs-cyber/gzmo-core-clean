//! Telemetry Dashboard Example
//!
//! Demonstrates metric collection and dashboard display.

use gzmo_core_clean::telemetry::{Telemetry, Dashboard};
use gzmo_core_clean::telemetry::exporter::{MetricExporter, ExportFormat};

fn main() {
    println!("=== Telemetry Dashboard Example ===\n");

    // Create telemetry collector
    let mut telemetry = Telemetry::new();
    let mut dashboard = Dashboard::new();

    // Simulate some activity
    for i in 0..5 {
        // Simulate generation
        telemetry.record_generation(100 + i * 10, 500 + i * 50);

        // Record some parameter correlations
        telemetry.record_correlation("temperature", 0.7 + i as f64 * 0.05, 0.8);

        // Occasionally record stuck/escape
        if i == 2 {
            telemetry.record_stuck();
        }
        if i == 3 {
            telemetry.record_escape();
        }

        // Update dashboard
        let events = dashboard.update(&telemetry);

        println!("\n--- Update {} ---", i + 1);
        for event in events {
            match event {
                gzmo_core_clean::telemetry::dashboard::DashboardEvent::MetricUpdate(_) => {
                    println!("Metrics updated");
                }
                gzmo_core_clean::telemetry::dashboard::DashboardEvent::Alert(msg) => {
                    println!("ALERT: {}", msg);
                }
                gzmo_core_clean::telemetry::dashboard::DashboardEvent::Status(msg) => {
                    println!("Status: {}", msg);
                }
            }
        }
    }

    // Show dashboard
    println!("\n=== Final Dashboard ===");
    println!("{}", dashboard.render());

    // Export metrics
    let exporter = MetricExporter::default();
    let snapshot = telemetry.snapshot();

    println!("\n=== JSON Export ===");
    println!("{}", exporter.export(&snapshot, ExportFormat::Json));

    println!("\n=== Prometheus Export ===");
    println!("{}", exporter.export(&snapshot, ExportFormat::Prometheus));
}