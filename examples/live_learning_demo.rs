//! Live Self-Improving Loop Demonstration
//!
//! Shows the self-improving loop running continuously with simulated outputs
//! to demonstrate the learning and parameter adjustment in real-time.

use gzmo_core_clean::feedback::{
    RepetitionDetector, OutputEvaluator, StrategyLearner, LearningStrategy, Experience,
};
use gzmo_core_clean::modulation::{StateGenerator, ParameterMapper};
use std::time::Duration;

fn main() {
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║   LIVE SELF-IMPROVING LOOP DEMONSTRATION               ║");
    println!("║   Real-time parameter modulation based on detection    ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    // Initialize components
    let mut generator = StateGenerator::new(0.506);
    let mapper = ParameterMapper::default();
    let mut detector = RepetitionDetector::with_config(10, 0.75, 0.90, 3);
    let mut evaluator = OutputEvaluator::new();
    let mut learner = StrategyLearner::with_config(
        1000,
        0.1,
        LearningStrategy::EpsilonGreedy { epsilon: 0.2 },
    );

    // Simulated outputs: some repetitive, some novel
    let simulated_outputs = vec![
        "The quick brown fox jumps over the lazy dog.",
        "Machine learning is a subset of artificial intelligence.",
        "The quick brown fox jumps over the lazy dog.",  // REPETITION
        "The quick brown fox jumps over the lazy dog.",  // STUCK PATTERN
        "Neural networks mimic the structure of biological neurons.",
        "The quick brown fox jumps over the lazy dog.",  // STILL STUCK
        "Deep learning uses multiple layers for feature extraction.", // NOVEL - BREAKTHROUGH
        "Transformers have revolutionized natural language processing.",
        "The quick brown fox jumps over the lazy dog.",  // Trying to repeat
        "Reinforcement learning optimizes through trial and error.",
        "Computer vision enables machines to interpret visual data.",
        "Generative adversarial networks consist of two competing networks.",
    ];

    let mut current_params = mapper.map_state(&generator);
    let mut stuck_count = 0;
    let mut escape_count = 0;
    let mut total_score = 0.0;

    println!("Starting {} iterations...\n", simulated_outputs.len());
    println!("{:<4} {:<12} {:<10} {:<10} {:<8} {:<12} {}",
        "Iter", "Temp", "State", "Score", "Stuck", "Action", "Output Preview");
    println!("{}", "─".repeat(100));

    for (i, output) in simulated_outputs.iter().enumerate() {
        // Step 1: Advance generator
        generator.step();
        
        // Step 2: Check current pattern state
        let was_stuck = detector.current_state().needs_exploration();
        let pattern_state_before = format!("{:?}", detector.current_state());
        
        // Step 3: Detect repetition in this output
        let state = detector.add_output(*output);
        let is_stuck = state.needs_exploration();
        
        // Step 4: Evaluate quality
        let quality = evaluator.evaluate(output, Duration::from_millis(500), 100);
        let score = quality.score();
        total_score += score;
        
        // Step 5: Record experience
        let success = !is_stuck;
        learner.record(Experience {
            temperature: current_params.temperature,
            max_tokens: current_params.max_tokens,
            quality_score: score,
            success,
            task_type: "demo".to_string(),
            timestamp: i as u64,
        });
        
        // Step 6: Adjust parameters based on state
        let mut action = "Normal".to_string();
        if is_stuck && !was_stuck {
            stuck_count += 1;
        }
        
        if state.needs_exploration() {
            // Boost temperature to escape repetition
            let base_params = mapper.map_state(&generator);
            current_params.temperature = (base_params.temperature + 0.4).min(1.5);
            current_params.max_tokens = (base_params.max_tokens as f32 * 1.2) as u32;
            action = "BOOST TEMP".to_string();
        } else {
            // Use learned recommendation if available
            if let Some(rec) = learner.recommend("demo") {
                current_params.temperature = rec.temperature;
                action = format!("Learned({:.2})", rec.confidence);
            } else {
                current_params = mapper.map_state(&generator);
            }
        }
        
        // Step 7: Track escapes
        if was_stuck && !is_stuck {
            escape_count += 1;
            action = "ESCAPED!".to_string();
        }
        
        // Display results
        let state_str = format!("{:?}", state);
        let preview = if output.len() > 40 {
            format!("{}...", &output[..40])
        } else {
            output.to_string()
        };
        
        println!("{:<4} {:<12.2} {:<10} {:<10.2} {:<8} {:<12} {}",
            i + 1,
            current_params.temperature,
            state_str,
            score,
            if is_stuck { "YES" } else { "no" },
            action,
            preview
        );
        
        // Small delay for visual effect
        std::thread::sleep(Duration::from_millis(100));
    }
    
    println!("{}", "─".repeat(100));
    
    // Final statistics
    let avg_score = total_score / simulated_outputs.len() as f64;
    let escape_rate = if stuck_count > 0 {
        (escape_count as f64 / stuck_count as f64) * 100.0
    } else {
        0.0
    };
    
    println!("\n╔════════════════════════════════════════════════════════╗");
    println!("║   FINAL STATISTICS                                     ║");
    println!("╠════════════════════════════════════════════════════════╣");
    println!("║  Total Iterations:     {:<30} ║", simulated_outputs.len());
    println!("║  Stuck Patterns:       {:<30} ║", stuck_count);
    println!("║  Escapes:              {:<30} ║", escape_count);
    println!("║  Escape Rate:          {:<29.1}% ║", escape_rate);
    println!("║  Average Score:        {:<30.2} ║", avg_score);
    println!("╚════════════════════════════════════════════════════════╝");
    
    // Show learned recommendation
    if let Some(rec) = learner.recommend("demo") {
        println!("\n🎯 LEARNED OPTIMAL PARAMETERS:");
        println!("   Recommended Temperature: {:.2}", rec.temperature);
        println!("   Recommended Max Tokens:  {}", rec.max_tokens);
        println!("   Confidence:              {:.2}", rec.confidence);
        println!("   Based on {} samples", rec.sample_count);
    }
    
    // Show learning stats
    if let Some(stats) = learner.stats("demo") {
        println!("\n📊 LEARNING STATISTICS:");
        println!("   Total Samples:    {}", stats.total_samples);
        println!("   Success Rate:     {:.1}%", stats.success_rate * 100.0);
        println!("   Avg Quality:      {:.2}", stats.avg_quality_score);
        println!("   Avg Temperature:  {:.2}", stats.avg_temperature);
    }
    
    println!("\n✅ Self-improving loop demonstration complete.");
    println!("   The system detected repetitions, adjusted parameters,");
    println!("   learned from experience, and improved over time.");
}
