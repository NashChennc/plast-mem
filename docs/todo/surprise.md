# Surprise Detection (TODO)

Surprise measures prediction error—the core mechanism of Event Segmentation Theory. It quantifies how unexpected an event is relative to prior expectations.

## Measurement

Surprise is scored on a `0.0` to `1.0` scale:

| Score | Interpretation | Example |
|-------|---------------|---------|
| 0.0 | Fully expected, no new information | "Got it" / "Understood" |
| 0.3 | Minor information gain | "I see" / "Makes sense" |
| 0.7 | Significant pivot or revelation | "Wait, what?" / "That's different" |
| 1.0 | Complete surprise, model-breaking | "I had no idea" / "That changes everything" |

## Data Model

```rust
pub struct EventSignificance {
    /// Prediction error / surprise (0.0 ~ 1.0)
    /// 0 = fully expected, no new information
    /// 1 = complete surprise, model-breaking
    pub surprise: f32,
}
```

## LLM Detection

The extraction prompt assesses surprise during event segmentation:

```
Evaluate the surprise (0-1) of this conversation:
- Does it contain unexpected information?
- Does it change prior understanding or plans?
- Does it involve new goals, constraints, or outcomes?

Reply with a single number.
```

## Integration with Memory

### FSRS Stability Boost

Surprise affects the initial stability of EpisodicMemory:

```rust
let initial_stability = base_stability * (1.0 + surprise * 0.5);
// surprise 1.0 -> stability × 1.5 (easier to retain)
// surprise 0.0 -> stability × 1.0 (normal decay)
```

Rationale: surprising events contain more learning value and should be retained longer.

### Display Logic

Surprise is used in tool result formatting to determine detail inclusion:

| Rank | Surprise Score | Include Details? | Rationale |
|------|----------------|------------------|-----------|
| 1-2 | ≥ 0.7 | ✅ | Top relevant + high surprise = key moment |
| 1-2 | < 0.7 | ❌ | Top relevant but routine information |
| 3-5 | any | ❌ | Context references, summaries suffice |

## Distinction from Valence

Surprise is preferred over emotional valence (positive/negative) for memory systems:

| Aspect | Valence | Surprise |
|--------|---------|----------|
| Relevance to memory | Low (positive/negative doesn't imply importance) | High (surprise indicates learning opportunity) |
| Detection cost | Requires sentiment analysis | Single scale, objective |
| EST alignment | Weak | Strong (directly implements prediction error) |

## Use Cases

1. **FSRS Initial Stability**: Higher surprise = higher initial stability
2. **Display**: High-surprise memories get full details in tool results

## Thresholds Reference

| Threshold | Usage | Rationale |
|-----------|-------|-----------|
| ≥ 0.7 | Key moment in tool result | Worth showing details |
