//! Basic Modulation Example
//!
//! Demonstrates state generator and parameter mapping.

use gzmo_core_clean::modulation::{StateGenerator, ParameterMapper};

fn main() {
    println!("=== Basic Modulation Example ===\n");

    // Create state generator with seed
    let mut gen = StateGenerator::new(0.506);

    // Create parameter mapper
    let mapper = ParameterMapper::default();

    println!("Running 10 steps of parameter modulation:\n");

    for i in 0..10 {
        // Advance generator
        let (x, y, z) = gen.step();

        // Map to LLM parameters
        let params = mapper.map_state(&gen);

        println!(
            "Step {}: x={:.4}, y={:.4}, z={:.4} | temp={:.2}, tokens={}, top_p={:.2}",
            i + 1,
            x, y, z,
            params.temperature,
            params.max_tokens,
            params.top_p
        );
    }

    println!("\nModulation complete!");
}