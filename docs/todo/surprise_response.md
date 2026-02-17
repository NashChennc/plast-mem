# Surprise Response / Free Energy Minimization (TODO)

## Core Insight

**Active learning** and **free energy minimization** are two sides of the same coin:

- **Free Energy Minimization** (the objective): When prediction fails (high surprise), reduce future prediction error
- **Active Learning** (the action): What strategy to employ to achieve that objective

```
High Surprise (prediction failure)
    │
    ▼
┌───────────────────────────┐
│  Free Energy Minimization │  ← The objective framework
│  (Minimize surprise)      │
└───────────────┬───────────┘
                │
    ┌───────────┼────────────┬───────────────┐
    ▼           ▼            ▼               ▼
┌──────────┐ ┌──────────┐ ┌────────────┐ ┌─────────────┐
│ Update   │ │Reinforce │ │   Active   │ │   Self      │
│ Model    │ │ Memory   │ │ Exploration│ │ Reflection  │
│(Internal)│ │(Storage) │ │  (External)│ │             │
└──────────┘ └──────────┘ └────────────┘ └─────────────┘
```

## Design: Unified Surprise Response

### Surprise Levels

| Level | Range | Response Strategy | Actions |
|-------|-------|-------------------|---------|
| **Normal** | 0.0 - 0.65 | Standard processing | Update Event Model |
| **Boundary** | 0.65 - 0.85 | Segmentation | Create new Episode + extract facts |
| **CognitiveDissonance** | 0.85 - 0.90 | Deep learning | + Deep extraction + belief update |
| **ParadigmShift** | 0.90+ | Active exploration | + Flashbulb + connect prior + follow-up questions |

**Note**: Flashbulb threshold is **0.90** (not 0.85).

### Data Model

```rust
// In episodic_memory table
pub struct EpisodicMemory {
    // ... existing fields ...

    /// Why was this episode surprising? (LLM-generated explanation)
    pub surprise_explanation: Option<String>,

    /// What strategy was used to minimize free energy
    pub surprise_response: Option<SurpriseResponse>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SurpriseResponse {
    /// Detected surprise level
    pub level: SurpriseLevel,

    /// Explanation of why the prediction failed
    pub explanation: String,

    /// Belief updates derived from this surprise
    pub belief_updates: Vec<BeliefUpdate>,

    /// Actions taken
    pub actions: Vec<ResponseAction>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum SurpriseLevel {
    Normal(f32),
    Boundary(f32),
    CognitiveDissonance(f32),
    ParadigmShift(f32),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BeliefUpdate {
    pub prior_assumption: String,
    pub new_understanding: String,
    pub confidence_delta: f32,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ResponseAction {
    /// Mark as flashbulb memory (never forget)
    FlashbulbMarked,

    /// Extracted additional semantic facts
    DeepExtraction { fact_count: usize },

    /// Created links to prior memories
    ConnectedPrior { episode_ids: Vec<Uuid> },

    /// Generated follow-up questions for clarification
    GeneratedQuestions { questions: Vec<String> },

    /// Self-reflection on cognitive blind spots
    SelfReflection { reflection: String },
}
```

### Job: SurpriseResponseJob

```rust
// crates/worker/src/jobs/surprise_response.rs

pub struct SurpriseResponseJob {
    pub episode_id: Uuid,
    pub surprise_signal: f32,
}

#[async_trait]
impl Job for SurpriseResponseJob {
    async fn run(&self) -> Result<(), AppError> {
        let episode = EpisodicMemory::find_by_id(self.episode_id).await?;
        let level = SurpriseLevel::from_signal(self.surprise_signal);

        // 1. Analyze: Why was this surprising?
        let analysis = analyze_surprise(&episode, self.surprise_signal).await?;

        // 2. Update: Minimize free energy based on level
        let actions = match level {
            SurpriseLevel::Normal(_) => vec![],

            SurpriseLevel::Boundary(_) => {
                vec![ResponseAction::DeepExtraction {
                    fact_count: extract_facts(&episode, 10).await?.len()
                }]
            }

            SurpriseLevel::CognitiveDissonance(_) => {
                let mut actions = vec![
                    ResponseAction::DeepExtraction {
                        fact_count: extract_facts(&episode, 20).await?.len()
                    },
                ];

                // Update beliefs
                for update in &analysis.belief_updates {
                    update_semantic_fact(update).await?;
                }

                actions
            }

            SurpriseLevel::ParadigmShift(_) => {
                let mut actions = vec![
                    ResponseAction::FlashbulbMarked,
                    ResponseAction::DeepExtraction {
                        fact_count: extract_facts(&episode, 30).await?.len()
                    },
                    ResponseAction::SelfReflection {
                        reflection: generate_reflection(&episode).await?
                    },
                ];

                // Active exploration: connect to prior memories
                let related = find_related_memories(&episode, 5).await?;
                if !related.is_empty() {
                    actions.push(ResponseAction::ConnectedPrior {
                        episode_ids: related
                    });
                }

                // Generate follow-up questions
                let questions = generate_exploration_questions(&episode).await?;
                if !questions.is_empty() {
                    actions.push(ResponseAction::GeneratedQuestions { questions });
                }

                actions
            }
        };

        // 3. Persist response
        let response = SurpriseResponse {
            level,
            explanation: analysis.explanation,
            belief_updates: analysis.belief_updates,
            actions,
        };

        episode.update_surprise_response(response).await?;

        Ok(())
    }
}
```

### LLM Interface

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SurpriseAnalysisOutput {
    /// Natural language explanation of why this was surprising
    pub explanation: String,

    /// Belief updates (if any)
    pub belief_updates: Vec<BeliefUpdateOutput>,

    /// Suggested exploration strategy
    pub suggested_strategy: ExplorationStrategy,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BeliefUpdateOutput {
    pub prior_assumption: String,
    pub new_understanding: String,
    pub confidence_delta: f32,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub enum ExplorationStrategy {
    None,
    DeepExtraction,
    ConnectPrior,
    AskFollowUpQuestions,
    SelfReflect,
}

const SURPRISE_ANALYSIS_PROMPT: &str = r#"
You are analyzing a surprising event in a conversation to help minimize prediction error (free energy minimization).

The event had a surprise score of {surprise}/1.0, where:
- 0.0-0.65: Normal (within expectations)
- 0.65-0.85: Boundary (new event, but understandable)
- 0.85-0.90: Cognitive Dissonance (challenges existing beliefs)
- 0.90+: Paradigm Shift (fundamentally changes understanding)

Analyze this episode:
{episode_summary}

Provide:
1. **Explanation**: Why was this surprising? What prediction failed?
2. **Belief Updates**: What assumptions were challenged? (if any)
3. **Strategy**: How should the system respond to minimize future prediction error?

Rules:
- Be specific about what was unexpected
- Only suggest belief updates if the surprise actually challenges prior understanding
- For high surprises (>0.85), suggest active exploration strategies
"#;
```

## Integration Points

### 1. Episode Creation Flow

```rust
// In creation.rs
pub async fn create_episode(
    // ...
    surprise_signal: f32,
) -> Result<EpisodicMemory, AppError> {
    let episode = // ... create episode

    // Trigger surprise response for high-surprise episodes
    if surprise_signal >= 0.85 {
        SurpriseResponseJob::enqueue(episode.id, surprise_signal).await?;
    }

    Ok(episode)
}
```

### 2. Semantic Memory Integration

Belief updates feed into Semantic Memory:

```rust
// Belief update -> Semantic Fact
async fn update_semantic_fact(update: &BeliefUpdate) -> Result<(), AppError> {
    // Mark prior assumption as challenged (in Phase 2: set invalid_at)
    // Insert new understanding as active fact
    SemanticFact::upsert(SemanticFact {
        subject: "assistant".to_string(),
        predicate: "believes".to_string(),
        object: update.new_understanding.clone(),
        // ...
    }).await
}
```

### 3. Flashbulb Memory Integration

Automatic flashbulb marking for paradigm shifts:

```rust
// In flashbulb_memory.rs (future)
pub async fn auto_mark_flashbulb(
    episode_id: Uuid,
    surprise: f32
) -> Result<(), AppError> {
    if surprise >= 0.90 {
        EpisodicMemory::update()
            .set(is_flashbulb, true)
            .filter(id.eq(episode_id))
            .exec()
            .await?;
    }
    Ok(())
}
```

## Implementation Plan

### Phase 1: Foundation (MVP)

- [ ] Add `surprise_explanation` and `surprise_response` columns to `episodic_memory`
- [ ] Create `SurpriseResponse` data structures
- [ ] Implement basic `SurpriseResponseJob` (levels + explanation only)
- [ ] Trigger on `surprise >= 0.85`

### Phase 2: Belief Updates

- [ ] Implement belief extraction in LLM prompt
- [ ] Connect to Semantic Memory (store beliefs as facts)
- [ ] Add `belief_updates` field tracking

### Phase 3: Active Exploration

- [ ] Implement `find_related_memories()` for `ParadigmShift` level
- [ ] Add `episode_relation` table for explicit memory links
- [ ] Generate and queue follow-up questions
- [ ] Self-reflection capability

### Phase 4: Optimization

- [ ] Tune thresholds based on observed distribution
- [ ] Batch processing for multiple high-surprise episodes
- [ ] Cost-aware response (skip LLM calls if API budget exceeded)

## Relationship to Other Memory Types

| Memory Type | Connection |
|-------------|------------|
| **Episodic** | Surprise is computed at episode creation; response enriches episode metadata |
| **Flashbulb** | High surprise (`>= 0.90`) automatically triggers flashbulb marking |
| **Semantic** | Belief updates become semantic facts; deep extraction populates facts |
| **Procedural** | Self-reflection may generate behavioral rules ("I should ask when...") |

## Open Questions

1. **Threshold calibration**: Should thresholds be adaptive based on user's baseline surprise distribution?
2. **Cost management**: How to balance thoroughness vs. API cost for high-frequency interactions?
3. **False positives**: How to distinguish genuine paradigm shifts from noise/outliers?
4. **User feedback**: Should users be able to correct surprise responses (e.g., "this wasn't actually important")?

## References

- [Nemori](https://arxiv.org/abs/2508.03341) — Predict-Calibrate principle, Free-Energy Principle
- [Event Segmentation Theory](https://doi.org/10.1037/0033-2909.133.2.273) — Zacks, J. M., Speer, N. K., Swallow, K. M., Braver, T. S., & Reynolds, J. R. (2007). Event perception: A mind-brain perspective. *Psychological Bulletin*, 133(2), 273-293.
- [Active Inference](https://doi.org/10.1162/neco_a_00912) — Friston, K. J., FitzGerald, T., Rigoli, F., Schwartenbeck, P., & Pezzulo, G. (2017). Active inference: A process theory. *Neural Computation*, 29(1), 1-49.

## What We Don't Do

- **No explicit world model table**: World model is implicit in the collection of semantic facts
- **No online policy learning**: Response strategies are hard-coded by surprise level (could be learned in future)
- **No counterfactual simulation**: We don't simulate "what if" scenarios to test belief updates
