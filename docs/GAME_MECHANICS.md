# Game Mechanics Reference

Nostr Nations is a turn-based 4X strategy game inspired by Civilization. This document details the core game mechanics.

## Turn System

### Turn Order

1. Players take turns in order (Player 0, Player 1, Player 2, ...)
2. When all players have taken their turn, the turn number increments
3. Eliminated players are skipped

### Turn Phases

Each player's turn consists of:

1. **Start of Turn**
   - Cities process yields (food, production, gold, science, culture)
   - Research progress applied
   - Unit movement points restored
   - Healing occurs for resting/fortified units

2. **Player Actions** (in any order)
   - Move units
   - Attack enemies
   - Found cities
   - Build improvements
   - Set production
   - Research technologies
   - Diplomacy actions

3. **End Turn**
   - Player explicitly ends their turn
   - Victory conditions checked
   - Turn passes to next player

### Simultaneous Turns (Multiplayer)

In multiplayer, players can act simultaneously during a turn. The turn only advances when all players have ended their turn.

---

## Combat System

### Combat Strength

Each unit has base combat strength. Effective strength is calculated as:

```
Effective Strength = Base Strength × (Health / 100)
```

| Unit      | Base Strength         |
| --------- | --------------------- |
| Warrior   | 8                     |
| Archer    | 5 (melee), 7 (ranged) |
| Spearman  | 11                    |
| Swordsman | 14                    |
| Knight    | 20                    |
| Musketman | 24                    |
| Rifleman  | 34                    |
| Infantry  | 50                    |
| Tank      | 60                    |

### Combat Resolution

Damage is calculated using strength ratios:

```
Ratio = Attacker Strength / Defender Strength
Base Damage = 30

Defender Damage = Base Damage × sqrt(Ratio) × Random(0.8-1.2)
Attacker Damage = Base Damage / sqrt(Ratio) × Random(0.8-1.2)
```

**Examples:**

- Equal strength (1:1): ~30 damage to each
- 2:1 advantage: ~42 to defender, ~21 to attacker
- 1:2 disadvantage: ~21 to defender, ~42 to attacker

### Combat Modifiers

#### Terrain Defense Bonuses

| Terrain/Feature          | Defense Bonus |
| ------------------------ | ------------- |
| Flat (Grassland, Plains) | 0%            |
| Hills                    | +25%          |
| Forest                   | +25%          |
| Jungle                   | +25%          |
| Forest on Hills          | +50%          |

#### Fortification

Units can fortify to gain defense:

- Turn 1: +25%
- Turn 2+: +50% (max)

Fortification is lost when the unit moves or attacks.

#### Promotions

| Promotion         | Effect                      |
| ----------------- | --------------------------- |
| Shock I/II/III    | +15/20/25% vs melee units   |
| Drill I/II/III    | +15/20/25% in rough terrain |
| Accuracy I/II/III | +15/20/25% ranged attacks   |
| Barrage I/II/III  | +15/20/25% vs cities        |
| Cover I/II        | +25/50% defense vs ranged   |

### Ranged Combat

Ranged units (Archers, Crossbows, Artillery):

- Can attack from 2+ tiles away
- **Do not receive counter-attack damage**
- Cannot capture cities (must use melee)

### City Combat

Cities have inherent combat strength and health:

- **Base Strength**: 10 (increases with buildings)
- **Base Health**: 200 (increases with walls)
- **Buildings**: Walls (+50 HP, +5 strength), Castle (+75 HP, +8 strength)

Cities cannot be captured until their health reaches 0. Ranged attacks can damage cities but cannot capture them.

---

## City Mechanics

### Founding Cities

- Settlers can found cities on valid land tiles
- Cannot found within 3 tiles of another city
- First city becomes the capital

### City Territory

Initial territory: city center + 6 adjacent tiles (7 total)

Territory expands via culture accumulation:

```
Culture Needed = 10 + (Tiles² × 2)
```

### Population Growth

Citizens consume 2 food per turn. Surplus food is stored toward growth.

**Food needed for next citizen:**

```
Food = 15 + 6 × (Population - 1) + Population^1.8
```

| Population | Food Needed |
| ---------- | ----------- |
| 1 → 2      | ~16         |
| 2 → 3      | ~25         |
| 3 → 4      | ~35         |
| 5 → 6      | ~55         |
| 10 → 11    | ~120        |

### Citizen Assignment

Each citizen can:

- **Work a tile**: Gather yields from one owned tile
- **Become a specialist**: Provide fixed yields

**Specialist Yields:**

| Specialist | Yields        |
| ---------- | ------------- |
| Scientist  | +3 Science    |
| Engineer   | +2 Production |
| Merchant   | +2 Gold       |
| Artist     | +3 Culture    |

### Production

Cities produce items using accumulated production (hammers).

**Building Costs:**

| Building   | Cost | Effect                            |
| ---------- | ---- | --------------------------------- |
| Monument   | 40   | +2 Culture                        |
| Granary    | 60   | +2 Food, keeps 50% food on growth |
| Library    | 75   | +1 Science per 2 pop              |
| Barracks   | 75   | +15 XP for trained units          |
| Walls      | 75   | +50 HP, +5 city strength          |
| Market     | 100  | +25% Gold                         |
| Aqueduct   | 120  | +2 Food, keeps 40% food on growth |
| University | 160  | +33% Science (requires Library)   |
| Bank       | 200  | +25% Gold (requires Market)       |
| Factory    | 300  | +25% Production                   |
| Hospital   | 400  | +5 Food (requires Aqueduct)       |

**Unit Costs:**

| Unit      | Cost |
| --------- | ---- |
| Warrior   | 40   |
| Settler   | 80   |
| Worker    | 50   |
| Archer    | 40   |
| Spearman  | 56   |
| Swordsman | 75   |
| Knight    | 120  |
| Musketman | 150  |

---

## Technology Tree Overview

Technologies are organized by era. Each has prerequisites and unlocks.

### Ancient Era (20-55 cost)

```
Agriculture ─┬─► Pottery ─────► Calendar
             │                      │
             ├─► Animal Husbandry ─►│ The Wheel
             │           │          │
             └─► Archery │          └─► Trapping
                         │
Mining ─────────────────►├─► Masonry
                         │
Sailing                  └─► Bronze Working
```

**Key Unlocks:**

- Agriculture: Farm improvement
- Pottery: Granary
- Archery: Archer unit
- Bronze Working: Spearman, Barracks
- Masonry: Walls, Quarry

### Classical Era (85-150 cost)

```
Sailing ──────► Optics ──────────────────────────────────►
                                                          │
Writing ───────► Philosophy ──► Education ──────────────►│
    │                              │                      │
    └─► Drama           Mathematics                       │
                            │                             │
Bronze Working ──► Iron Working ──────────────────────────┤
                                                          │
Animal Husbandry ──► Horseback Riding ───────────────────►│
                                                          │
                   Currency ◄── Mathematics               │
```

**Key Unlocks:**

- Iron Working: Swordsman
- Horseback Riding: Horseman
- Philosophy: Temple
- Mathematics: Catapult, Courthouse
- Currency: Market

### Medieval Era (275-400 cost)

```
Civil Service ──► Chivalry ──► Banking
     │               │
Education ──────────►│
                     │
Iron Working ──► Steel ──► Longswordsman
     │
     └─► Machinery ──► Crossbow, Lumber Mill
                │
Optics ──► Compass ──► Caravel
                │
Mathematics + Machinery ──► Physics ──► Trebuchet
```

**Key Unlocks:**

- Chivalry: Knight, Castle
- Education: University
- Steel: Longswordsman
- Banking: Bank

### Renaissance Era (485-620 cost)

```
Compass + Education ──► Astronomy ──► Navigation
                                          │
Machinery ──► Printing Press ──► Acoustics │
                    │                      │
Physics + Steel ──► Gunpowder ──► Metallurgy
                        │             │
                   Chemistry       Lancer, Cannon
                        │
Banking + Printing ──► Economics
```

**Key Unlocks:**

- Gunpowder: Musketman
- Metallurgy: Cannon, Lancer
- Astronomy: Ocean crossing
- Economics: Workshop

### Industrial Era (700-950 cost)

```
Economics ──► Industrialization ──► Steam Power
                      │                   │
Scientific Theory ◄───┴───► Electricity ──┤
        │                        │        │
        └─► Biology ──► Hospital │        │
                                 │        │
Metallurgy ──► Rifling ──► Military Science
                   │               │
                   └─► Dynamite ◄──┤
                           │       │
                      Artillery    Cavalry
```

**Key Unlocks:**

- Rifling: Rifleman
- Industrialization: Factory
- Steam Power: Ironclad
- Dynamite: Artillery

### Modern Era (1040-1500 cost)

```
Steam Power ──► Replaceable Parts ──► Combustion
                        │                  │
Dynamite ──► Ballistics │                Tank
        │        │      │
        └─► Flight ◄────┘
               │
Telegraph ──► Electronics ──► Radar ──► Rocketry ──► Spaceflight
                                │
                           Nuclear Fission
```

**Key Unlocks:**

- Replaceable Parts: Infantry
- Combustion: Tank
- Flight: Fighter
- Radar: Battleship, Bomber
- Rocketry: Rocket Artillery
- Spaceflight: Spaceship parts

---

## Victory Conditions

### Domination Victory

Control all original capital cities.

- Capture every player's original capital
- Immediate victory when achieved

### Science Victory

Build and launch a spaceship.

**Required Parts (5 total):**

1. Cockpit
2. Fuel Tanks
3. Thrusters
4. Life Support
5. Stasis Chamber

Each part costs 1500 production. Requires Spaceflight technology.

### Economic Victory

Accumulate 20,000 gold.

- Gold must be in treasury (not spent)
- Immediate victory when achieved

### Diplomatic Victory

Win a United Nations vote.

- Requires more than half of all votes
- Players vote for a candidate
- Held periodically (configurable)

### Score Victory

Highest score when turn limit is reached.

**Score Components:**

- Population: 2 points per citizen
- Land: 1 point per owned tile
- Wonders: 25 points each
- Technologies: 5 points each
- Future techs: 10 points each

**Default turn limit:** 500 turns

---

## Diplomacy System

### Relationship Scores

Each player pair has a relationship score (-100 to +100):

| Score Range | Status                       |
| ----------- | ---------------------------- |
| -100 to -50 | Hostile (war likely)         |
| -49 to 49   | Neutral                      |
| 50+         | Friendly (treaties possible) |

### Treaties

| Treaty             | Requirement   | Effect                                     |
| ------------------ | ------------- | ------------------------------------------ |
| Peace              | At war        | 10-turn ceasefire                          |
| Open Borders       | Score ≥ 50    | Units can pass through territory           |
| Trade Agreement    | Score ≥ 50    | Gold bonus for both                        |
| Research Agreement | Score ≥ 50    | Science bonus for both                     |
| Defensive Pact     | Allied status | Automatic war declaration if ally attacked |

### War

**Declaring War:**

- Breaks all existing treaties
- Relationship score -40
- Enables combat between players

**Making Peace:**

- Requires mutual agreement
- Relationship score +20
- 10-turn mandatory peace period

### War Weariness

Accumulates each turn at war, causing happiness penalties. Slowly decreases during peace.

---

## Map Generation

Maps are procedurally generated using a seeded RNG.

### Map Sizes

| Size     | Dimensions | Recommended Players |
| -------- | ---------- | ------------------- |
| Duel     | 40×25      | 2                   |
| Small    | 52×32      | 3-4                 |
| Standard | 66×42      | 4-6                 |
| Large    | 80×52      | 6-8                 |
| Huge     | 104×64     | 8+                  |

### Terrain Types

| Terrain   | Food | Production | Gold | Movement Cost |
| --------- | ---- | ---------- | ---- | ------------- |
| Grassland | 2    | 0          | 0    | 1             |
| Plains    | 1    | 1          | 0    | 1             |
| Desert    | 0    | 0          | 0    | 1             |
| Tundra    | 1    | 0          | 0    | 1             |
| Snow      | 0    | 0          | 0    | 1             |
| Coast     | 1    | 0          | 1    | 1             |
| Ocean     | 1    | 0          | 0    | 1             |

### Features

| Feature   | Yield Modifier | Defense             | Movement    |
| --------- | -------------- | ------------------- | ----------- |
| Forest    | +1 Production  | +25%                | +1          |
| Jungle    | +1 Food        | +25%                | +1          |
| Hills     | +1 Production  | +25%                | +1          |
| Mountains | Impassable     | -                   | -           |
| Rivers    | -              | +25% when defending | +1 to cross |

### Resources

**Strategic:** Horses, Iron, Coal, Oil, Uranium  
**Luxury:** Gold, Silver, Gems, Spices, etc.  
**Bonus:** Wheat, Cattle, Fish, etc.

---

## Tile Improvements

Workers can build improvements on tiles to increase yields.

| Improvement  | Terrain           | Yields        | Tech Required    |
| ------------ | ----------------- | ------------- | ---------------- |
| Farm         | Grassland, Plains | +1 Food       | Agriculture      |
| Mine         | Hills             | +1 Production | Mining           |
| Pasture      | Resource tiles    | Varies        | Animal Husbandry |
| Plantation   | Resource tiles    | Varies        | Calendar         |
| Camp         | Resource tiles    | Varies        | Trapping         |
| Quarry       | Stone, Marble     | +1 Production | Masonry          |
| Lumber Mill  | Forest            | +1 Production | Machinery        |
| Trading Post | Any               | +1 Gold       | None             |
| Fort         | Any               | +50% Defense  | None             |

Roads reduce movement cost and connect cities for trade.

---

## Cashu Randomness

Combat and map generation use Cashu blinded signatures for verifiable randomness.

### How It Works

1. Player creates a blinded message
2. Cashu mint signs without seeing content
3. Player unblinds to get deterministic random value
4. Anyone can verify the signature

### Applications

- **Combat**: ±20% damage variance
- **Map generation**: Seeded terrain placement
- **Exploration**: Goody hut outcomes

### Offline Fallback

For local games, a deterministic PRNG provides consistent (but biasable) randomness.
