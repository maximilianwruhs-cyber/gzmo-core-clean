//! Feedback Loop Example
//!
//! Demonstrates repetition detection and exploration adjustment.

use gzmo_core_clean::feedback::{RepetitionDetector, PatternState};
use std::time::Duration;

fn main() {
    println!("=== Feedback Loop Example ===\n");

    let mut detector = RepetitionDetector::new();

    // Simulate outputs that become repetitive
    let outputs = vec![
        "The cat sat on the mat.",          // Novel
        "The quick brown fox jumps.",       // Novel
        "The cat sat on the mat.",          // Duplicate
        "The cat sat on the mat.",          // Loop
        "Something completely different.",  // Novel (escape)
    ];

    println!("Processing outputs with repetition detection:\n");

    for (i, output) in outputs.iter().enumerate() {
        let state = detector.add_output(output);

        let action = match state {
            PatternState::Novel => "Process normally",
            PatternState::Similar => "Slight adjustment",
            PatternState::Stuck => "Increase exploration",
            PatternState::Loop => "HIGH exploration boost",
        };

        println!(
            "Output {}: \"{}\" -> {:?}",
            i + 1,
            if output.len() > 30 { &output[..30] } else { output },
            state
        );
        println!("  Action: {}\n", action);
    }

    println!("Detection history: {} items", detector.history_len());
    println!("Final state: {:?}", detector.current_state());
}