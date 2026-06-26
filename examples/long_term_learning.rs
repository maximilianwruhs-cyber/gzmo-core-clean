//! Long-Term Learning Demonstration
//!
//! Shows how the system improves over 100+ iterations
//! by accumulating experiences and converging to optimal parameters.

use gzmo_core_clean::feedback::{
    RepetitionDetector, OutputEvaluator, StrategyLearner, LearningStrategy, Experience,
};
use gzmo_core_clean::modulation::{StateGenerator, ParameterMapper};
use std::time::Duration;

fn main() {
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║   LONG-TERM LEARNING: 100+ ITERATIONS                  ║");
    println!("║   Watching improvement converge over time              ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    // Initialize components
    let mut generator = StateGenerator::new(0.506);
    let mapper = ParameterMapper::default();
    let mut detector = RepetitionDetector::with_config(5, 0.75, 0.90, 2);
    let mut evaluator = OutputEvaluator::new();
    let mut learner = StrategyLearner::with_config(
        500,
        0.1,
        LearningStrategy::BestAverage,
    );

    // Track metrics over time
    let mut total_score = 0.0;
    let mut stuck_count = 0;
    let mut escape_count = 0;
    let mut checkpoints: Vec<(usize, f32, f64, f64, f64)> = Vec::new();

    println!("Learning Phase 1: Exploration (0-30)");
    println!("{}", "─".repeat(60));

    for i in 0..100 {
        // Step 1: Get base parameters from generator
        generator.step();
        let base_params = mapper.map_state(&generator);
        
        // Step 2: Apply learning (with exploration)
        let (temp, source) = if let Some(rec) = learner.recommend("task") {
            // Exploit: use learned temp with small noise
            let noise = (rand::random::<f32>() - 0.5) * 0.2;
            let t = (rec.temperature + noise).clamp(0.3, 1.5);
            (t, format!("learned({:.2})", rec.confidence))
        } else {
            // Explore: try random temps to gather data
            let t = 0.3 + rand::random::<f32>() * 1.0;
            (t, "explore".to_string())
        };
        
        // Step 3: Simulate generation
        let (output, quality) = simulate_generation(temp);
        let latency = Duration::from_millis(200 + (rand::random::<u64>() % 300));
        
        // Step 4: Detect repetition
        let state = detector.add_output(&output);
        let is_stuck = state.needs_exploration();
        let was_stuck = detector.current_state().needs_exploration();
        
        // Step 5: Evaluate quality
        let metrics = evaluator.evaluate(&output, latency, 100);
        let mut score = metrics.score();
        
        // Penalize stuck outputs
        if is_stuck {
            score *= 0.5;
            stuck_count += 1;
        }
        total_score += score;
        
        // Track escapes
        if was_stuck && !is_stuck {
            escape_count += 1;
        }
        
        // Step 6: Record experience (CRITICAL: always record)
        learner.record(Experience {
            temperature: temp,
            max_tokens: 1000,
            quality_score: score,
            success: !is_stuck,
            task_type: "task".to_string(),
            timestamp: i as u64,
        });
        
        // Record checkpoint every 10 iterations
        let rec_conf = learner.recommend("task").map(|r| r.confidence).unwrap_or(0.0);
        let rec_temp = learner.recommend("task").map(|r| r.temperature).unwrap_or(temp);
        
        if i % 10 == 0 || i == 99 {
            checkpoints.push((i, temp, score, rec_conf, rec_temp as f64));
            
            if i <= 30 || (i > 30 && i <= 60 && i % 15 == 0) || i == 99 {
                let status = if is_stuck { "STUCK" } else { "OK  " };
                println!("Iter {:>3} | Temp: {:.2} | Score: {:.2} | {} | {}",
                    i, temp, score, status, source);
            }
        }
        
        if i == 30 {
            println!("\nLearning Phase 2: Convergence (30-60)");
            println!("{}", "─".repeat(60));
        }
        if i == 60 {
            println!("\nLearning Phase 3: Optimization (60-100)");
            println!("{}", "─".repeat(60));
        }
    }

    // Final results
    println!("\n{}", "═".repeat(60));
    println!("FINAL RESULTS");
    println!("{}", "═".repeat(60));
    
    let avg_score = total_score / 100.0;
    let escape_rate = if stuck_count > 0 {
        (escape_count as f64 / stuck_count as f64) * 100.0
    } else {
        0.0
    };
    
    println!("\n📊 OVERALL STATISTICS:");
    println!("   Total Iterations:      100");
    println!("   Average Score:         {:.2}", avg_score);
    println!("   Stuck Patterns:        {}", stuck_count);
    println!("   Escapes:               {} ({:.1}%)", escape_count, escape_rate);
    
    // Progression table
    println!("\n📈 LEARNING PROGRESSION:");
    println!("   Iter | Used  | Score | Conf | Learned");
    println!("   {}", "─".repeat(45));
    for (iter, used, score, conf, learned) in &checkpoints {
        println!("   {:>4} | {:.2} | {:.2}  | {:.2} | {:.2}", 
            iter, used, score, conf, learned);
    }
    
    // Final recommendation
    if let Some(rec) = learner.recommend("task") {
        println!("\n🎯 LEARNED OPTIMAL PARAMETERS:");
        println!("   Optimal Temperature:     {:.2}", rec.temperature);
        println!("   Confidence:              {:.2}", rec.confidence);
        println!("   Based on {} samples", rec.sample_count);
        
        // Analyze convergence
        let optimal = 0.45f32; // Best temp in simulation
        let diff = (rec.temperature - optimal).abs();
        
        if diff < 0.15 {
            println!("\n✅ SUCCESSFUL CONVERGENCE!");
            println!("   Learned temp {:.2} is close to optimal {:.2}", 
                rec.temperature, optimal);
        } else {
            println!("\n⚠️  Converged to {:.2} (optimal is ~{:.2})", 
                rec.temperature, optimal);
        }
    } else {
        println!("\n❌ No recommendation learned");
    }
    
    // Stats
    if let Some(stats) = learner.stats("task") {
        println!("\n📚 LEARNING STATISTICS:");
        println!("   Samples:        {}", stats.total_samples);
        println!("   Success Rate:   {:.1}%", stats.success_rate * 100.0);
        println!("   Avg Quality:    {:.2}", stats.avg_quality_score);
        println!("   Avg Temp:       {:.2}", stats.avg_temperature);
    }
    
    // Improvement
    if checkpoints.len() >= 3 {
        let early = checkpoints[1].2;
        let mid = checkpoints[checkpoints.len()/2].2;
        let late = checkpoints.last().unwrap().2;
        
        println!("\n🚀 QUALITY TREND:");
        println!("   Early (iter ~10):  {:.2}", early);
        println!("   Middle (iter ~50): {:.2}", mid);
        println!("   Late (iter ~100):  {:.2}", late);
        
        if late > early {
            let improvement = ((late - early) / early) * 100.0;
            println!("   Improvement:       +{:.1}%", improvement);
        } else {
            let decline = ((early - late) / early) * 100.0;
            println!("   Decline:           -{:.1}%", decline);
        }
    }
    
    println!("\n✅ Long-term learning demonstration complete.");
}

/// Simulate generation quality based on temperature
fn simulate_generation(temp: f32) -> (String, f64) {
    // Quality curve:
    // - 0.3-0.6: High quality (0.9-1.0) <- OPTIMAL
    // - 0.6-0.9: Medium (0.7-0.8)
    // - 0.9-1.2: Low (0.4-0.6)
    // - 1.2+: Very low (0.2-0.4)
    
    let base_quality = if temp < 0.5 {
        0.95
    } else if temp < 0.7 {
        0.80
    } else if temp < 1.0 {
        0.60
    } else if temp < 1.3 {
        0.40
    } else {
        0.25
    };
    
    let noise = (rand::random::<f64>() - 0.5) * 0.15;
    let quality = (base_quality + noise).clamp(0.0, 1.0);
    
    // Generate varied output
    let output = format!("Generation at temp {:.2} with quality {:.2}", temp, quality);
    
    (output, quality)
}
