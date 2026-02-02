# Nostr Nations - AI Roadmap (Phase 2)

## Overview

This document outlines the AI system planned for Phase 2 development. AI opponents will allow single-player games and fill empty slots in multiplayer matches.

## Design Principles

### 1. Deterministic Behavior

AI decisions must be deterministic for replay:
- Use Cashu randomness for any probabilistic choices
- Same game state + same random seed = same AI decision
- AI actions recorded as normal Nostr events

### 2. Fair Play

AI operates under same rules as human players:
- No map visibility cheating (respects fog of war)
- No production cheating (builds at normal rate)
- No combat cheating (same formulas)
- Uses same Cashu randomness for combat

### 3. Configurable Difficulty

Multiple difficulty levels via:
- Decision quality (search depth)
- Bonus/penalties to yields
- Aggression tuning
- Mistake frequency

## AI Architecture

### Component Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      AI Controller                           │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │  Perception │  │   Memory    │  │   Decision Engine   │ │
│  │   System    │  │   System    │  │                     │ │
│  │             │  │             │  │  ┌───────────────┐  │ │
│  │ - Visible   │  │ - Last seen │  │  │   Evaluator   │  │ │
│  │   tiles     │  │   enemy pos │  │  └───────────────┘  │ │
│  │ - Known     │  │ - Threat    │  │  ┌───────────────┐  │ │
│  │   enemies   │  │   levels    │  │  │   Planner     │  │ │
│  │ - Resource  │  │ - Past      │  │  └───────────────┘  │ │
│  │   locations │  │   actions   │  │  ┌───────────────┐  │ │
│  └─────────────┘  └─────────────┘  │  │   Executor    │  │ │
│                                     │  └───────────────┘  │ │
│                                     └─────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Perception System

What the AI knows (fog of war respected):
- Explored map areas
- Currently visible units and cities
- Resource locations discovered
- Own units, cities, resources

### Memory System

What the AI remembers:
- Last known enemy positions
- Enemy military strength estimates
- Threat assessments per region
- Historical diplomatic actions

### Decision Engine

#### State Evaluator

Scores current game state:
```rust
struct StateEvaluation {
    military_strength: f32,
    economic_strength: f32,
    scientific_progress: f32,
    territorial_control: f32,
    diplomatic_standing: f32,
    victory_progress: HashMap<VictoryType, f32>,
}
```

#### Strategic Planner

Long-term goals:
- Victory path selection
- Expansion targets
- Research priorities
- Alliance strategies

#### Tactical Executor

Turn-by-turn decisions:
- Unit movement orders
- Combat targeting
- City production
- Diplomatic actions

## AI Subsystems

### Military AI

#### Unit Management

```rust
trait MilitaryAI {
    /// Decide what to do with a military unit
    fn get_unit_orders(&self, unit: &Unit, state: &GameState) -> UnitOrders;
    
    /// Evaluate a potential attack
    fn evaluate_attack(&self, attacker: &Unit, target: AttackTarget) -> f32;
    
    /// Find optimal defensive positions
    fn find_defensive_positions(&self, region: &Region) -> Vec<HexCoord>;
}

enum UnitOrders {
    Move(Vec<HexCoord>),
    Attack(UnitId),
    Fortify,
    Garrison(CityId),
    Explore,
    Heal,
}
```

#### Threat Assessment

```rust
struct ThreatLevel {
    immediate_threat: f32,    // Enemy units nearby
    strategic_threat: f32,    // Enemy military potential
    economic_threat: f32,     // Enemy growth rate
}

impl MilitaryAI {
    fn assess_threat(&self, player: PlayerId, state: &GameState) -> ThreatLevel;
}
```

#### Combat Decisions

When to attack:
- Favorable odds (>60% win chance)
- Strategic value of target
- Acceptable losses

When to retreat:
- Unfavorable odds
- Unit critically damaged
- Better position available

### Economic AI

#### City Management

```rust
trait EconomicAI {
    /// Choose what to produce in a city
    fn choose_production(&self, city: &City, state: &GameState) -> ProductionItem;
    
    /// Assign citizens to tiles
    fn assign_citizens(&self, city: &City) -> Vec<HexCoord>;
    
    /// Evaluate city site for settling
    fn evaluate_city_site(&self, coord: HexCoord, state: &GameState) -> f32;
}
```

#### Production Priority

Decision factors:
1. Immediate military need?
2. Growth vs production balance
3. Building prerequisites
4. Wonder racing

#### Expansion Strategy

When to build settlers:
- Happy capacity available
- Good city sites available
- Not under military threat
- Can protect the settler

Where to settle:
- Resource access
- Defensive terrain
- Growth potential
- Strategic position

### Diplomacy AI

#### Personality System

```rust
struct AIPersonality {
    aggression: f32,        // 0.0 = peaceful, 1.0 = warlike
    trustworthiness: f32,   // Likelihood to honor deals
    greed: f32,             // Willingness to accept unfair deals
    forgiveness: f32,       // How quickly grudges fade
}
```

#### Relationship Management

```rust
trait DiplomacyAI {
    /// Evaluate a trade proposal
    fn evaluate_deal(&self, deal: &TradeProposal) -> DealEvaluation;
    
    /// Decide whether to declare war
    fn consider_war(&self, target: PlayerId, state: &GameState) -> bool;
    
    /// Generate counter-proposal
    fn counter_propose(&self, deal: &TradeProposal) -> Option<TradeProposal>;
}

struct DealEvaluation {
    fair_value: bool,
    strategic_value: f32,
    trust_factor: f32,
    accept: bool,
}
```

### Research AI

#### Tech Selection

```rust
trait ResearchAI {
    /// Choose next technology to research
    fn choose_research(&self, state: &GameState) -> TechId;
    
    /// Evaluate a technology's value
    fn evaluate_tech(&self, tech: &Technology, state: &GameState) -> f32;
}
```

Factors:
- Units unlocked (military need)
- Buildings unlocked (economic value)
- Prerequisites for desired tech
- Era advancement
- Victory condition progress

## Difficulty Levels

### Settler (Easy)

- AI makes suboptimal decisions
- -25% to AI yields
- Passive/defensive behavior
- Accepts unfair trade deals
- Rarely declares war

### Chieftain

- Basic strategic decisions
- No yield modifiers
- Moderate aggression
- Fair trade evaluation

### Warlord

- Good tactical decisions
- No yield modifiers
- Strategic alliances
- Opportunistic warfare

### Prince (Normal)

- Competent all-around
- No yield modifiers
- Balanced aggression
- Smart diplomacy

### King

- Strong decisions
- +10% to AI yields
- Coordinated attacks
- Strategic wonder racing

### Emperor

- Excellent decisions
- +20% to AI yields
- Multi-front warfare
- Tech beelining

### Immortal

- Near-optimal decisions
- +30% to AI yields
- Aggressive early game
- Perfect micro-management

### Deity

- Optimal decisions
- +50% to AI yields
- Extra starting units
- Relentless pressure

## Implementation Phases

### Phase 2.1: Basic AI

- Random valid moves
- Basic production (units when threatened, buildings otherwise)
- Simple tech selection (first available)
- No diplomacy (auto-reject all)

### Phase 2.2: Tactical AI

- Pathfinding for units
- Combat evaluation
- Defensive positioning
- City site evaluation

### Phase 2.3: Strategic AI

- Long-term planning
- Victory path selection
- Expansion strategy
- Research priorities

### Phase 2.4: Diplomatic AI

- Trade evaluation
- Alliance formation
- War declaration logic
- Personality system

### Phase 2.5: Polish

- Difficulty tuning
- Personality variety
- Edge case handling
- Performance optimization

## Testing Strategy

### Unit Tests

- Individual decision functions
- State evaluation accuracy
- Combat odds calculation

### Integration Tests

- Full turn execution
- Multi-turn scenarios
- Victory achievement

### Playtesting

- Human vs AI games
- AI vs AI tournaments
- Difficulty balance verification

## Performance Considerations

### Turn Time Budget

AI should complete turn in <5 seconds:
- Parallel evaluation where possible
- Caching of repeated calculations
- Early termination of low-value searches

### Memory Usage

- Don't store full game states
- Incremental memory updates
- Garbage collection between turns

## Future Enhancements

### Machine Learning (Phase 3+)

Potential ML applications:
- Opening strategies learned from games
- Combat outcome prediction
- Player behavior modeling
- Adaptive difficulty

### Mod Support

Allow custom AI:
- Lua scripting for AI personalities
- Exposed evaluation functions
- Custom victory conditions
