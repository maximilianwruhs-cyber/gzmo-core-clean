# GZMO Clean Core

From-scratch architecture for GZMO without theatrical language.

## Design Principles

1. **No Theatrical Language** — Every name describes actual behavior
2. **Explicit Over Implicit** — All parameters visible and configurable  
3. **Measurable Outcomes** — Every decision has attached metrics
4. **Minimal Indirection** — Direct data flow, no theatrical layers

## Architecture

### Module Structure

```
gzmo-core-clean/
├── src/
│   ├── lib.rs              # Clean re-exports
│   ├── modulation/         # Deterministic parameter generation (was "chaos")
│   │   ├── state_generator.rs    # Lorenz ODE solver
│   │   ├── parameter_mapper.rs   # State → LLM param mapping
│   │   └── tempo.rs              # Adaptive timing
│   ├── feedback/           # Closed-loop optimization
│   │   ├── detector.rs           # Repetition detection
│   │   ├── evaluator.rs          # Quality metrics
│   │   ├── learner.rs            # Strategy optimization
│   │   └── queue.rs              # Delayed mutation queue
│   ├── pedagogy/           # Simplified 2-agent system
│   │   ├── evaluator.rs          # Student state assessment
│   │   ├── tutor.rs              # Socratic response generator
│   │   └── session.rs            # Session management
│   ├── storage/            # 2-layer storage
│   │   ├── vault.rs              # SQLite structured storage
│   │   ├── vectors.rs            # Qdrant semantic search
│   │   └── dedup.rs              # Binary duplicate detection
│   ├── etl/                # Nightly batch processing
│   │   ├── extract.rs            # LLM extraction
│   │   ├── verify.rs             # Confidence filtering
│   │   └── promote.rs            # KG/vault insertion
│   ├── skills/             # Function registry
│   │   ├── registry.rs           # Skill registration
│   │   ├── dispatch.rs           # Execution dispatcher
│   │   └── builtin.rs            # Built-in functions
│   ├── gateway/            # LLM API abstraction
│   │   ├── client.rs             # HTTP client with retries
│   │   ├── routing.rs            # Multi-model fallback
│   │   └── cache.rs              # Response caching
│   ├── config/             # Explicit configuration
│   │   ├── mod.rs                # Config loading
│   │   ├── validation.rs         # Parameter validation
│   │   └── defaults.rs           # Documented defaults
│   ├── telemetry/          # Observability
│   │   ├── metrics.rs            # Metric collection
│   │   ├── exporter.rs           # Export to stdout/file
│   │   └── dashboard.rs          # Real-time display
│   └── cli/                # Command-line interface
│       ├── args.rs               # Argument parsing
│       ├── repl.rs               # Interactive mode
│       └── commands.rs           # Subcommand handlers
```

## Usage

### Basic Parameter Modulation

```rust
use gzmo_core_clean::modulation::StateGenerator;

let mut gen = StateGenerator::new(0.506);
let temp = gen.map_to_range(0.3, 1.2);
println!("Temperature: {}", temp);
```

### Feedback Loop

```rust
use gzmo_core_clean::feedback::{RepetitionDetector, PatternState};

let mut detector = RepetitionDetector::new();

let state = detector.add_output("some text");
if state.needs_exploration() {
    println!("Increasing temperature for exploration");
}
```

### Pedagogy Session

```rust
use gzmo_core_clean::pedagogy::{Session, SessionConfig};

let mut session = Session::new(SessionConfig {
    subject: "calculus".to_string(),
    student_id: "student_001".to_string(),
    max_interactions: 10,
    initial_level: KnowledgeLevel::Developing,
});

let result = session.interact("What is a derivative?");
```

## CLI Usage

```bash
# Interactive mode
gzmo-clean

# Run main loop with configuration
gzmo-clean --config config.toml run

# Start pedagogy session
gzmo-clean pedagogy --subject rust

# Show telemetry
gzmo-clean telemetry

# Run ETL batch job
gzmo-clean etl

# Run self-improving loop
gzmo-clean self-improve
```

## Configuration

See `src/config/defaults.rs` for all default values.

```toml
[modulation]
sigma = 10.0
rho = 28.0
beta = 2.667
temp_min = 0.3
temp_max = 1.2

[feedback]
history_window = 10
similarity_threshold = 0.85
exploration_boost = 0.5

[storage]
vault_path = "data/vault.db"
qdrant_url = "http://localhost:6333"
```

## Key Differences from Original GZMO

| Aspect | Original (Theatrical) | Clean (Honest) |
|--------|----------------------|----------------|
| **Names** | "Thought Cabinet", "Dream", "Chaos" | "ParameterQueue", "NightlyETL", "StateGenerator" |
| **Layers** | 4-layer memory | 2-layer storage |
| **Agents** | 4-agent pedagogy | 2-agent system |
| **Feedback** | Open loop | Closed loop with metrics |
| **Magic Numbers** | Hardcoded | Configurable with validation |
| **Lifecycle** | 6-variant enum | Binary (Duplicate/Novel) |
| **Death/Rebirth** | Boolean theater | Capacity-based throttling |
| **Energy** | Meaningless formula | Work-based calculation |
| **Discovery** | "Spark", "Dream" | Explicit extract/verify/promote |

## License

MIT OR Apache-2.0
