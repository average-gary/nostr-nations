# Nostr Nations - Game Mechanics

## Overview

Nostr Nations is a 4X (explore, expand, exploit, exterminate) turn-based strategy game for 2-4 players. This document defines the core gameplay mechanics.

## Map System

### Hex Grid

The game uses a hexagonal tile system:
- **Pointy-top orientation**: Hexes have points at top and bottom
- **Offset coordinates**: Odd-q vertical layout
- **Wrap options**: Cylindrical (east-west wrap) or flat

### Map Sizes

| Size | Dimensions | Tiles | Typical Duration |
|------|------------|-------|------------------|
| Duel | 40 x 25 | 1,000 | 30-60 min |
| Small | 60 x 38 | 2,280 | 1-2 hours |
| Standard | 80 x 50 | 4,000 | 2-4 hours |
| Large | 100 x 63 | 6,300 | 4-6 hours |
| Huge | 120 x 75 | 9,000 | 6+ hours |

### Terrain Types

| Terrain | Movement Cost | Defense Bonus | Yield |
|---------|--------------|---------------|-------|
| Grassland | 1 | 0% | 2 Food |
| Plains | 1 | 0% | 1 Food, 1 Production |
| Desert | 1 | 0% | - |
| Tundra | 1 | 0% | 1 Food |
| Snow | 1 | 0% | - |
| Hills | 2 | +25% | +1 Production |
| Mountains | Impassable | - | - |
| Forest | 2 | +25% | 1 Food, 1 Production |
| Jungle | 2 | +25% | 2 Food |
| Marsh | 3 | -10% | 1 Food |
| Coast | 1 (naval) | 0% | 1 Food, 1 Gold |
| Ocean | 1 (naval) | 0% | 1 Food |

### Resources

**Strategic Resources** (required for units/buildings):
- Iron - Swordsmen, Knights
- Horses - Cavalry units
- Coal - Factories, Ironclads
- Oil - Tanks, Aircraft
- Uranium - Nuclear units

**Luxury Resources** (happiness bonus):
- Gold, Silver, Gems, Pearls
- Silk, Dyes, Spices, Incense
- Wine, Furs, Ivory, Marble

**Bonus Resources** (yield bonus):
- Wheat, Cattle, Sheep, Deer
- Fish, Whales, Crabs
- Stone, Copper, Salt

### Map Generation

Procedurally generated from Cashu seed:
1. Generate landmass shapes (continents/islands)
2. Apply terrain types based on latitude/moisture
3. Place mountains and hills
4. Add forests, jungles, marshes
5. Distribute resources (balanced for fairness)
6. Place starting locations (equidistant)

## Units

### Unit Categories

**Civilian Units:**
- Settler - Founds cities
- Worker - Improves tiles
- Great People - Special abilities

**Military Units:**

| Era | Melee | Ranged | Cavalry | Naval | Siege |
|-----|-------|--------|---------|-------|-------|
| Ancient | Warrior, Spearman | Archer, Slinger | Chariot | Galley | Catapult |
| Classical | Swordsman, Pikeman | Composite Bow | Horseman | Trireme | Ballista |
| Medieval | Longswordsman, Halberdier | Crossbow | Knight | Caravel | Trebuchet |
| Renaissance | Musketman | - | Lancer | Frigate | Cannon |
| Industrial | Rifleman | Gatling Gun | Cavalry | Ironclad | Artillery |
| Modern | Infantry | Machine Gun | Tank | Battleship | Rocket Artillery |

### Unit Stats

Each unit has:
- **Combat Strength**: Base attack/defense power
- **Ranged Strength**: Attack power for ranged units
- **Movement Points**: Tiles per turn
- **Range**: Attack range for ranged units
- **Health**: 100 HP (damage reduces effectiveness)

### Movement

- Units have movement points (MP) per turn
- Terrain costs MP to enter
- Roads reduce movement cost to 1/3
- Railroads allow unlimited movement on network
- Zone of Control: Enemy units in adjacent tiles reduce MP
- Embarking: Land units can board transports or swim (with tech)

### Combat System

#### Melee Combat

When units fight adjacent:
1. Calculate attacker strength (base + modifiers)
2. Calculate defender strength (base + terrain + modifiers)
3. Request Cashu randomness
4. Apply combat formula
5. Both units take damage based on strength ratio

```
Combat Formula:
damage_to_defender = 30 * (attacker_str / defender_str) * random_factor
damage_to_attacker = 30 * (defender_str / attacker_str) * random_factor

random_factor = 0.8 + (cashu_random * 0.4)  // Range: 0.8 to 1.2
```

#### Ranged Combat

Ranged units attack without retaliation:
1. Check range (1-2 tiles typically)
2. Calculate ranged strength vs defender
3. Request Cashu randomness
4. Defender takes damage, attacker takes none

#### Combat Modifiers

| Modifier | Bonus |
|----------|-------|
| Fortified | +25% |
| Hills defense | +25% |
| Forest/Jungle defense | +25% |
| River crossing attack | -25% |
| Flanking (unit per adjacent ally) | +10% |
| Great General nearby | +15% |
| Health (per 10 HP lost) | -10% |
| Promotions | Varies |

### Experience and Promotions

Units gain XP from combat:
- Win combat: 5 XP
- Lose combat: 3 XP
- Kill unit: +5 XP bonus

Promotion thresholds: 10, 30, 60, 100, 150...

**Promotion Trees:**
- Shock (melee) - Bonus vs melee
- Drill (melee) - Rough terrain bonus
- Accuracy (ranged) - Bonus vs units
- Barrage (ranged) - Bonus vs cities
- Mobility (cavalry) - Movement bonus
- Logistics (all) - Extra attacks

## Cities

### Founding

- Settler unit founds city on current tile
- Minimum 4 tiles from other cities
- City claims surrounding tiles (workable radius)

### City Attributes

- **Population**: Number of citizens (workers)
- **Health**: Defense points (starts at 200)
- **Combat Strength**: City's defense rating
- **Production**: Hammers per turn
- **Food**: Food per turn (growth)
- **Gold**: Gold per turn
- **Science**: Science per turn
- **Culture**: Culture per turn (border expansion)

### Citizen Management

Each citizen works one tile:
- Automatically assigned to best yields
- Can be manually assigned
- Specialists work buildings instead of tiles

**Specialist Types:**
- Scientist: +3 Science
- Engineer: +2 Production
- Merchant: +3 Gold
- Artist: +3 Culture

### City Growth

```
Food surplus = food_produced - food_consumed
food_consumed = population * 2

Turns to grow = food_needed / food_surplus
food_needed = 15 + (population * 6) + (population^1.5)
```

### Production Queue

Cities produce one item at a time:
- Units
- Buildings
- Wonders
- Projects

```
Turns to complete = remaining_production / production_per_turn
```

### Buildings

Buildings provide bonuses to cities:

| Building | Cost | Effect | Requires |
|----------|------|--------|----------|
| Monument | 40 | +2 Culture | - |
| Granary | 60 | +2 Food, food stored on growth | Pottery |
| Library | 75 | +1 Science per 2 pop | Writing |
| Barracks | 75 | +15 XP to units | Bronze Working |
| Walls | 75 | +5 City Defense | Masonry |
| Market | 100 | +25% Gold | Currency |
| Aqueduct | 120 | 40% food kept on growth | Engineering |
| University | 160 | +33% Science | Education |
| Bank | 200 | +25% Gold | Banking |
| Factory | 300 | +25% Production | Industrialization |
| Hospital | 400 | +5 Food | Biology |

### Wonders

World Wonders (one per game):
- Pyramids: +2 Workers, Workers build 25% faster
- Great Library: Free technology
- Colosseum: +4 Happiness
- Machu Picchu: +25% Gold from trade routes
- Oxford University: Free technology
- Eiffel Tower: +5 Happiness
- Manhattan Project: Enables nuclear weapons

National Wonders (one per civilization):
- National College: +3 Science, +50% in city
- Heroic Epic: +15% combat for all units
- National Treasury: +8 Gold

## Resources and Economy

### Yields

Every tile produces yields:
- **Food**: City growth
- **Production**: Building/unit construction
- **Gold**: Treasury income
- **Science**: Technology research
- **Culture**: Border expansion, policies

### Gold Economy

Income:
- Tile yields
- Buildings (Markets, Banks)
- Trade routes
- Resource sales

Expenses:
- Unit maintenance (1-5 gold/turn per unit)
- Building maintenance (1-3 gold/turn per building)
- Road maintenance

### Trade Routes

- Connect cities for bonus gold
- Require roads/railroads or harbors
- Can trade with other players (mutual benefit)

## Fog of War

### Visibility States

- **Unexplored**: Never seen (black)
- **Explored**: Previously seen, shows terrain (gray)
- **Visible**: Currently seen by a unit (full color)

### Vision Range

- Most units: 2 tiles
- Scouts: 3 tiles
- Naval units: 2 tiles
- Hills: +1 tile vision
- Mountains block line of sight

### Information Hiding

Players only see:
- Their own units and cities
- Enemy units in visible tiles
- Terrain they've explored
- Resources they've discovered (requires tech)

Hidden information (encrypted in Nostr events):
- Enemy city production
- Enemy gold/science
- Unexplored map areas
- Enemy diplomacy with others

## Turn Structure

### Turn Phases

1. **Start of Turn**
   - Receive income (gold, science, culture)
   - City food calculated
   - Unit healing occurs
   - Triggered events process

2. **Action Phase**
   - Move units
   - Initiate combat
   - Manage cities
   - Conduct diplomacy
   - Choose research

3. **End of Turn**
   - Production completes
   - Cities grow/shrink
   - Borders expand
   - Research completes
   - Turn passes to next player

### Simultaneous vs Sequential Turns

**Sequential (Default):**
- Players take turns in order
- See results of previous player's actions
- Better for competitive play

**Simultaneous (Optional):**
- All players move at once
- Actions resolved at end of round
- Faster gameplay, more chaos

## Victory Conditions

### Domination Victory

Control all original capitals:
- Each player starts with one capital
- Capture by moving unit into enemy capital
- Last player with their capital wins

### Science Victory

Complete the space program:
1. Research all techs in Modern era
2. Build Apollo Program (project)
3. Build spaceship parts:
   - SS Booster (x3)
   - SS Cockpit
   - SS Engine
   - SS Stasis Chamber
4. Launch spaceship

### Economic Victory

Accumulate wealth:
- Reach 20,000 gold in treasury
- Or: Control 50% of luxury resources
- Or: Build all economic wonders

### Diplomatic Victory

Win world leader election:
- Build United Nations wonder
- Hold election every 10 turns
- Need majority of all delegates
- Delegates from city-states and players

### Score Victory

After turn limit (300/400/500 turns):
- Score based on:
  - Population (5 per citizen)
  - Land (1 per tile)
  - Techs (5 per tech)
  - Wonders (20 per wonder)
  - Cities (10 per city)
- Highest score wins
