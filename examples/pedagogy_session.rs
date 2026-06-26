//! Pedagogy Session Example
//!
//! Demonstrates Socratic tutoring session.

use gzmo_core_clean::pedagogy::{Session, SessionConfig, SessionResult};
use gzmo_core_clean::pedagogy::evaluator::KnowledgeLevel;

fn main() {
    println!("=== Pedagogy Session Example ===\n");

    // Create a new learning session
    let mut session = Session::new(SessionConfig {
        subject: "calculus".to_string(),
        student_id: "student_001".to_string(),
        max_interactions: 5,
        initial_level: KnowledgeLevel::Novice,
        llm_endpoint: "http://localhost:8000/v1/chat/completions".to_string(),
        llm_model: "local-model".to_string(),
    });

    println!("Session ID: {}", session.id);
    println!("Subject: {}\n", session.subject);

    // Simulate student interactions using sync method
    let student_inputs = vec![
        "What is a derivative?",
        "I don't understand the concept of limits.",
        "So dx/dy is the rate of change?",
    ];

    for input in student_inputs {
        println!("Student: {}", input);

        let result = session.interact_sync(input);

        match result {
            SessionResult::Response { text, meta, remaining_interactions } => {
                println!("Tutor: {}", text);
                println!("  [Difficulty: {:.1}%, Load: {}, Remaining: {}]\n",
                    meta.difficulty * 100.0,
                    meta.cognitive_load,
                    remaining_interactions
                );
            }
            SessionResult::SessionEnded => {
                println!("Session has ended.\n");
                break;
            }
        }
    }

    // Show session statistics
    let stats = session.stats();
    println!("Session Statistics:");
    println!("  Total interactions: {}", stats.total_interactions);
    println!("  Tutor questions: {}", stats.tutor_questions);
    println!("  Average cognitive load: {}", stats.avg_cognitive_load);
    println!("  Average latency: {}ms", stats.avg_latency_ms);
    println!("  Total LLM calls: {}", stats.total_llm_calls);
}