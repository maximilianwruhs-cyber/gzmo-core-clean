//! Real LLM Connection Demo
//!
//! This connects to an actual llama.cpp server and shows real learning.
//! Requires: llama.cpp server running on localhost:8000

use gzmo_core_clean::feedback::{
    RepetitionDetector, OutputEvaluator, StrategyLearner, LearningStrategy, Experience,
};
use gzmo_core_clean::modulation::{StateGenerator, ParameterMapper};
use gzmo_core_clean::gateway::{LlmClient, LlmRequest};

#[tokio::main]
async fn main() {
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║   REAL LLM CONNECTION DEMO                             ║");
    println!("║   Connecting to localhost:8000 (llama.cpp)           ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    // Check if server is running
    let client = match LlmClient::new(
        "http://localhost:8000/v1/chat/completions",
        "not-needed",
        "local-model"
    ).with_timeout(5).send(LlmRequest {
        system_prompt: None,
        user_prompt: "Hi".to_string(),
        params: gzmo_core_clean::modulation::LLMParameters {
            temperature: 0.7,
            max_tokens: 10,
            top_p: 0.9,
        },
    }).await {
        Ok(_) => {
            println!("✅ Connected to LLM server at localhost:8000\n");
            LlmClient::local_llamacpp("local-model")
        }
        Err(e) => {
            println!("❌ Could not connect to LLM server: {}", e);
            println!("\nPlease start llama.cpp server:");
            println!("  ./server -m model.gguf --port 8000");
            println!("\n⚠️  Falling back to simulation mode...\n");
            return run_simulation().await;
        }
    };

    // Real learning loop with actual LLM
    let mut generator = StateGenerator::new(0.506);
    let mapper = ParameterMapper::default();
    let mut detector = RepetitionDetector::with_config(5, 0.75, 0.90, 2);
    let mut evaluator = OutputEvaluator::new();
    let mut learner = StrategyLearner::with_config(
        100,
        0.1,
        LearningStrategy::BestAverage,
    );

    let prompt = "Generate a creative sentence about artificial intelligence.";
    
    println!("Running 10 real generations with learning...\n");
    println!("{:<4} {:<10} {:<12} {:<10} {}",
        "Iter", "Temp", "State", "Score", "LLM Output (first 50 chars)");
    println!("{}", "─".repeat(90));

    for i in 0..10 {
        // Get parameters
        generator.step();
        let mut params = mapper.map_state(&generator);
        
        // Apply learning
        if let Some(rec) = learner.recommend("demo") {
            params.temperature = rec.temperature;
        }
        
        // Call REAL LLM
        let request = LlmRequest {
            system_prompt: Some("You are creative.".to_string()),
            user_prompt: prompt.to_string(),
            params,
        };
        
        let start = std::time::Instant::now();
        match client.send(request).await {
            Ok(response) => {
                let latency = start.elapsed();
                
                // Detect pattern
                let state = detector.add_output(&response.text);
                
                // Evaluate
                let quality = evaluator.evaluate(&response.text, latency, response.tokens_used);
                
                // Record experience
                let is_stuck = state.needs_exploration();
                learner.record(Experience {
                    temperature: params.temperature,
                    max_tokens: params.max_tokens,
                    quality_score: quality.score(),
                    success: !is_stuck,
                    task_type: "demo".to_string(),
                    timestamp: i,
                });
                
                // Show result
                let preview: String = response.text.chars().take(50).collect();
                println!("{:>4} {:.2} {:<12} {:.2} {}",
                    i,
                    params.temperature,
                    format!("{:?}", state),
                    quality.score(),
                    preview.replace('\n', " ")
                );
            }
            Err(e) => {
                println!("{:>4} {:.2} {:<12} {} {}",
                    i, params.temperature, "ERROR", 0.0, e);
            }
        }
    }
    
    // Show learned parameters
    println!("\n{}", "═".repeat(90));
    if let Some(rec) = learner.recommend("demo") {
        println!("🎯 Learned optimal temperature: {:.2} (confidence: {:.2})",
            rec.temperature, rec.confidence);
        println!("   Based on {} real LLM calls", rec.sample_count);
    }
    
    if let Some(stats) = learner.stats("demo") {
        println!("\n📊 Real learning statistics:");
        println!("   Success rate: {:.1}%", stats.success_rate * 100.0);
        println!("   Average quality: {:.2}", stats.avg_quality_score);
    }
}

async fn run_simulation() {
    println!("SIMULATION MODE (no LLM server)\n");
    println!("This shows the mechanics, but to see REAL learning:");
    println!("1. Download llama.cpp");
    println!("2. Get a model (e.g., llama-3.2-1b-instruct.gguf)");
    println!("3. Run: ./server -m model.gguf --port 8000");
    println!("4. Re-run this demo\n");
    
    // Quick simulation
    println!("Simulated output showing what would happen:");
    for i in 0..5 {
        println!("  Iter {}: Temp={:.2}, Score={:.2}", i, 0.7 + i as f32 * 0.1, 0.8);
    }
}
