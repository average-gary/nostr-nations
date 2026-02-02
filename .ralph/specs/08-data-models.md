# Nostr Nations - Data Models

## Overview

This document defines the core data structures used throughout the game. All structures are designed to be serializable for Nostr events and deterministic for replay validation.

## Game State

### Root Game State

```rust
pub struct GameState {
    /// Unique game identifier
    pub id: GameId,
    
    /// Game configuration
    pub settings: GameSettings,
    
    /// Current turn number
    pub turn: u32,
    
    /// Index of player whose turn it is
    pub current_player: PlayerId,
    
    /// All players in the game
    pub players: Vec<Player>,
    
    /// The game map
    pub map: Map,
    
    /// All units in the game
    pub units: HashMap<UnitId, Unit>,
    
    /// All cities in the game
    pub cities: HashMap<CityId, City>,
    
    /// Diplomatic relationships
    pub diplomacy: DiplomacyState,
    
    /// Random seed for deterministic replay
    pub seed: [u8; 32],
    
    /// Chain of event IDs for validation
    pub event_chain: Vec<EventId>,
}

pub type GameId = String;
pub type PlayerId = u8;
pub type UnitId = u64;
pub type CityId = u64;
pub type EventId = String;
```

### Game Settings

```rust
pub struct GameSettings {
    /// Display name for the game
    pub name: String,
    
    /// Map dimensions
    pub map_size: MapSize,
    
    /// Number of players (2-4)
    pub player_count: u8,
    
    /// Enabled victory conditions
    pub victory_conditions: VictoryConditions,
    
    /// Turn time limit in seconds (0 = unlimited)
    pub turn_timer: u32,
    
    /// Maximum turns (0 = unlimited)
    pub max_turns: u32,
    
    /// Allow technology trading
    pub tech_trading: bool,
    
    /// Game start era
    pub starting_era: Era,
}

#[derive(Clone, Copy)]
pub enum MapSize {
    Duel,      // 40x25
    Small,     // 60x38
    Standard,  // 80x50
    Large,     // 100x63
    Huge,      // 120x75
}

impl MapSize {
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            MapSize::Duel => (40, 25),
            MapSize::Small => (60, 38),
            MapSize::Standard => (80, 50),
            MapSize::Large => (100, 63),
            MapSize::Huge => (120, 75),
        }
    }
}

pub struct VictoryConditions {
    pub domination: bool,
    pub science: bool,
    pub economic: bool,
    pub diplomatic: bool,
    pub score: bool,
}
```

## Player

```rust
pub struct Player {
    /// Player index (0-3)
    pub id: PlayerId,
    
    /// Nostr public key
    pub pubkey: String,
    
    /// Display name
    pub name: String,
    
    /// Civilization
    pub civilization: Civilization,
    
    /// Player color
    pub color: PlayerColor,
    
    /// Current gold in treasury
    pub gold: i32,
    
    /// Gold income per turn
    pub gold_per_turn: i32,
    
    /// Science output per turn
    pub science_per_turn: i32,
    
    /// Culture output per turn
    pub culture_per_turn: i32,
    
    /// Currently researching
    pub current_research: Option<TechId>,
    
    /// Research progress on current tech
    pub research_progress: u32,
    
    /// Completed technologies
    pub technologies: HashSet<TechId>,
    
    /// Capital city ID
    pub capital: Option<CityId>,
    
    /// Has this player been eliminated?
    pub eliminated: bool,
    
    /// Tiles this player has explored
    pub explored_tiles: HashSet<HexCoord>,
    
    /// Score components
    pub score: Score,
}

pub struct PlayerColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub struct Score {
    pub population: u32,
    pub land: u32,
    pub techs: u32,
    pub wonders: u32,
    pub cities: u32,
    pub total: u32,
}

pub struct Civilization {
    pub id: String,
    pub name: String,
    pub leader_name: String,
    pub unique_unit: UnitType,
    pub unique_building: BuildingType,
    pub ability: CivAbility,
}
```

## Map

### Hex Coordinate System

```rust
/// Axial coordinates for hex grid (offset odd-q)
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct HexCoord {
    pub q: i32,  // column
    pub r: i32,  // row
}

impl HexCoord {
    /// Get all 6 neighboring hexes
    pub fn neighbors(&self) -> [HexCoord; 6] {
        let offset = if self.q % 2 == 0 { 0 } else { 1 };
        [
            HexCoord { q: self.q + 1, r: self.r - 1 + offset },
            HexCoord { q: self.q + 1, r: self.r + offset },
            HexCoord { q: self.q, r: self.r + 1 },
            HexCoord { q: self.q - 1, r: self.r + offset },
            HexCoord { q: self.q - 1, r: self.r - 1 + offset },
            HexCoord { q: self.q, r: self.r - 1 },
        ]
    }
    
    /// Distance to another hex
    pub fn distance(&self, other: &HexCoord) -> u32 {
        let cube1 = self.to_cube();
        let cube2 = other.to_cube();
        ((cube1.0 - cube2.0).abs() 
         + (cube1.1 - cube2.1).abs() 
         + (cube1.2 - cube2.2).abs()) as u32 / 2
    }
    
    fn to_cube(&self) -> (i32, i32, i32) {
        let x = self.q;
        let z = self.r - (self.q - (self.q & 1)) / 2;
        let y = -x - z;
        (x, y, z)
    }
}
```

### Map Structure

```rust
pub struct Map {
    /// Map dimensions
    pub width: u32,
    pub height: u32,
    
    /// All tiles indexed by coordinate
    pub tiles: HashMap<HexCoord, Tile>,
    
    /// Does the map wrap east-west?
    pub wrap_x: bool,
}

pub struct Tile {
    /// Position
    pub coord: HexCoord,
    
    /// Base terrain
    pub terrain: Terrain,
    
    /// Feature on tile (forest, jungle, etc.)
    pub feature: Option<Feature>,
    
    /// Resource present
    pub resource: Option<Resource>,
    
    /// Improvement built
    pub improvement: Option<Improvement>,
    
    /// Road/railroad
    pub road: Option<Road>,
    
    /// Owning player (via city borders)
    pub owner: Option<PlayerId>,
    
    /// Owning city
    pub city_id: Option<CityId>,
    
    /// River edges (6 possible edges)
    pub river_edges: [bool; 6],
}
```

### Terrain and Features

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Terrain {
    Grassland,
    Plains,
    Desert,
    Tundra,
    Snow,
    Coast,
    Ocean,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Feature {
    Hills,
    Mountains,
    Forest,
    Jungle,
    Marsh,
    Oasis,
    FloodPlains,
    Ice,
}

impl Terrain {
    pub fn base_yields(&self) -> Yields {
        match self {
            Terrain::Grassland => Yields { food: 2, ..Default::default() },
            Terrain::Plains => Yields { food: 1, production: 1, ..Default::default() },
            Terrain::Desert => Yields::default(),
            Terrain::Tundra => Yields { food: 1, ..Default::default() },
            Terrain::Snow => Yields::default(),
            Terrain::Coast => Yields { food: 1, gold: 1, ..Default::default() },
            Terrain::Ocean => Yields { food: 1, ..Default::default() },
        }
    }
    
    pub fn movement_cost(&self) -> u32 {
        match self {
            Terrain::Coast | Terrain::Ocean => 1,  // Naval only
            _ => 1,
        }
    }
}

impl Feature {
    pub fn yield_modifier(&self) -> Yields {
        match self {
            Feature::Hills => Yields { production: 1, ..Default::default() },
            Feature::Forest => Yields { food: -1, production: 1, ..Default::default() },
            Feature::Jungle => Yields { production: -1, ..Default::default() },
            Feature::Marsh => Yields { food: -1, ..Default::default() },
            Feature::Oasis => Yields { food: 3, gold: 1, ..Default::default() },
            Feature::FloodPlains => Yields { food: 2, ..Default::default() },
            _ => Yields::default(),
        }
    }
    
    pub fn movement_cost(&self) -> u32 {
        match self {
            Feature::Hills => 2,
            Feature::Forest | Feature::Jungle => 2,
            Feature::Marsh => 3,
            Feature::Mountains => u32::MAX,  // Impassable
            _ => 1,
        }
    }
    
    pub fn defense_bonus(&self) -> i32 {
        match self {
            Feature::Hills | Feature::Forest | Feature::Jungle => 25,
            Feature::Marsh => -10,
            _ => 0,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct Yields {
    pub food: i32,
    pub production: i32,
    pub gold: i32,
    pub science: i32,
    pub culture: i32,
}
```

### Resources

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Resource {
    // Strategic
    Iron,
    Horses,
    Coal,
    Oil,
    Uranium,
    
    // Luxury
    Gold,
    Silver,
    Gems,
    Pearls,
    Silk,
    Dyes,
    Spices,
    Incense,
    Wine,
    Furs,
    Ivory,
    Marble,
    
    // Bonus
    Wheat,
    Cattle,
    Sheep,
    Deer,
    Fish,
    Whales,
    Crabs,
    Stone,
    Copper,
    Salt,
}

impl Resource {
    pub fn category(&self) -> ResourceCategory {
        match self {
            Resource::Iron | Resource::Horses | Resource::Coal | 
            Resource::Oil | Resource::Uranium => ResourceCategory::Strategic,
            
            Resource::Gold | Resource::Silver | Resource::Gems |
            Resource::Pearls | Resource::Silk | Resource::Dyes |
            Resource::Spices | Resource::Incense | Resource::Wine |
            Resource::Furs | Resource::Ivory | Resource::Marble => ResourceCategory::Luxury,
            
            _ => ResourceCategory::Bonus,
        }
    }
    
    pub fn yield_bonus(&self) -> Yields {
        // Returns additional yields when improved
        match self {
            Resource::Wheat | Resource::Cattle => Yields { food: 1, ..Default::default() },
            Resource::Iron | Resource::Coal => Yields { production: 1, ..Default::default() },
            Resource::Gold | Resource::Silver => Yields { gold: 2, ..Default::default() },
            _ => Yields::default(),
        }
    }
}

pub enum ResourceCategory {
    Strategic,
    Luxury,
    Bonus,
}
```

### Improvements and Roads

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Improvement {
    Farm,
    Mine,
    Plantation,
    Pasture,
    Camp,
    Quarry,
    FishingBoats,
    OilWell,
    LumberMill,
    TradingPost,
    Fort,
    Academy,       // Great Scientist
    Manufactory,   // Great Engineer
    CustomsHouse,  // Great Merchant
    Landmark,      // Great Artist
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Road {
    Road,
    Railroad,
}
```

## Units

```rust
pub struct Unit {
    /// Unique unit ID
    pub id: UnitId,
    
    /// Owning player
    pub owner: PlayerId,
    
    /// Unit type
    pub unit_type: UnitType,
    
    /// Current position
    pub position: HexCoord,
    
    /// Current health (0-100)
    pub health: u32,
    
    /// Remaining movement points (x10 for precision)
    pub movement: u32,
    
    /// Experience points
    pub experience: u32,
    
    /// Promotions earned
    pub promotions: Vec<Promotion>,
    
    /// Is unit fortified?
    pub fortified: bool,
    
    /// Turns until fortification bonus (0-2)
    pub fortify_turns: u32,
    
    /// Is unit embarked (on water)?
    pub embarked: bool,
    
    /// Has unit used its action this turn?
    pub has_acted: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UnitType {
    // Civilian
    Settler,
    Worker,
    GreatScientist,
    GreatEngineer,
    GreatMerchant,
    GreatArtist,
    GreatGeneral,
    
    // Ancient Military
    Warrior,
    Archer,
    Spearman,
    Slinger,
    Chariot,
    Galley,
    Catapult,
    
    // Classical Military
    Swordsman,
    CompositeBow,
    Horseman,
    Pikeman,
    Trireme,
    Ballista,
    
    // Medieval Military
    Longswordsman,
    Crossbow,
    Knight,
    Halberdier,
    Caravel,
    Trebuchet,
    
    // Renaissance Military
    Musketman,
    Lancer,
    Frigate,
    Cannon,
    
    // Industrial Military
    Rifleman,
    GatlingGun,
    Cavalry,
    Ironclad,
    Artillery,
    
    // Modern Military
    Infantry,
    MachineGun,
    Tank,
    Battleship,
    RocketArtillery,
    Fighter,
    Bomber,
}

impl UnitType {
    pub fn stats(&self) -> UnitStats {
        match self {
            UnitType::Warrior => UnitStats {
                combat_strength: 8,
                ranged_strength: 0,
                range: 0,
                movement: 2,
                cost: 40,
                category: UnitCategory::Melee,
            },
            UnitType::Archer => UnitStats {
                combat_strength: 5,
                ranged_strength: 7,
                range: 2,
                movement: 2,
                cost: 40,
                category: UnitCategory::Ranged,
            },
            UnitType::Settler => UnitStats {
                combat_strength: 0,
                ranged_strength: 0,
                range: 0,
                movement: 2,
                cost: 106,
                category: UnitCategory::Civilian,
            },
            // ... more units
            _ => UnitStats::default(),
        }
    }
}

pub struct UnitStats {
    pub combat_strength: u32,
    pub ranged_strength: u32,
    pub range: u32,
    pub movement: u32,
    pub cost: u32,
    pub category: UnitCategory,
}

pub enum UnitCategory {
    Civilian,
    Melee,
    Ranged,
    Cavalry,
    Naval,
    Siege,
    Air,
}

#[derive(Clone, Copy)]
pub enum Promotion {
    // Melee
    ShockI,
    ShockII,
    ShockIII,
    DrillI,
    DrillII,
    DrillIII,
    
    // Ranged
    AccuracyI,
    AccuracyII,
    AccuracyIII,
    BarrageI,
    BarrageII,
    BarrageIII,
    
    // General
    Medic,
    March,
    Blitz,
    Logistics,
}
```

## Cities

```rust
pub struct City {
    /// Unique city ID
    pub id: CityId,
    
    /// Owning player
    pub owner: PlayerId,
    
    /// City name
    pub name: String,
    
    /// City location
    pub position: HexCoord,
    
    /// Population count
    pub population: u32,
    
    /// Food stored toward next population
    pub food_stored: u32,
    
    /// Is this the capital?
    pub is_capital: bool,
    
    /// City health (defense)
    pub health: u32,
    pub max_health: u32,
    
    /// City combat strength
    pub combat_strength: u32,
    
    /// Buildings constructed
    pub buildings: HashSet<BuildingType>,
    
    /// Current production item
    pub production: Option<ProductionItem>,
    
    /// Production progress (hammers)
    pub production_progress: u32,
    
    /// Production queue
    pub production_queue: Vec<ProductionItem>,
    
    /// Tiles being worked by citizens
    pub worked_tiles: HashSet<HexCoord>,
    
    /// Specialist assignments
    pub specialists: Specialists,
    
    /// Tiles owned by this city
    pub territory: HashSet<HexCoord>,
    
    /// Culture accumulated (for border expansion)
    pub culture: u32,
}

#[derive(Clone)]
pub enum ProductionItem {
    Unit(UnitType),
    Building(BuildingType),
    Wonder(WonderType),
    Project(ProjectType),
}

impl ProductionItem {
    pub fn cost(&self) -> u32 {
        match self {
            ProductionItem::Unit(ut) => ut.stats().cost,
            ProductionItem::Building(bt) => bt.cost(),
            ProductionItem::Wonder(wt) => wt.cost(),
            ProductionItem::Project(pt) => pt.cost(),
        }
    }
}

pub struct Specialists {
    pub scientists: u32,
    pub engineers: u32,
    pub merchants: u32,
    pub artists: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuildingType {
    Monument,
    Granary,
    Library,
    Barracks,
    Walls,
    Market,
    Aqueduct,
    University,
    Bank,
    Factory,
    Hospital,
    // ... more buildings
}

impl BuildingType {
    pub fn cost(&self) -> u32 {
        match self {
            BuildingType::Monument => 40,
            BuildingType::Granary => 60,
            BuildingType::Library => 75,
            BuildingType::Barracks => 75,
            BuildingType::Walls => 75,
            BuildingType::Market => 100,
            BuildingType::Aqueduct => 120,
            BuildingType::University => 160,
            BuildingType::Bank => 200,
            BuildingType::Factory => 300,
            BuildingType::Hospital => 400,
        }
    }
    
    pub fn effects(&self) -> BuildingEffects {
        match self {
            BuildingType::Granary => BuildingEffects {
                food: 2,
                food_kept_on_growth: 0.5,
                ..Default::default()
            },
            BuildingType::Library => BuildingEffects {
                science_per_2_pop: 1,
                ..Default::default()
            },
            // ... more effects
            _ => BuildingEffects::default(),
        }
    }
}

#[derive(Default)]
pub struct BuildingEffects {
    pub food: i32,
    pub production: i32,
    pub gold: i32,
    pub science: i32,
    pub culture: i32,
    pub defense: i32,
    pub xp_bonus: u32,
    pub food_kept_on_growth: f32,
    pub gold_modifier: f32,
    pub science_modifier: f32,
    pub production_modifier: f32,
    pub science_per_2_pop: i32,
}
```

## Diplomacy State

```rust
pub struct DiplomacyState {
    /// Relationships between all pairs of players
    pub relationships: HashMap<(PlayerId, PlayerId), Relationship>,
    
    /// Active treaties
    pub treaties: Vec<Treaty>,
    
    /// Pending proposals
    pub proposals: Vec<DiplomaticProposal>,
}

pub struct Relationship {
    pub player_a: PlayerId,
    pub player_b: PlayerId,
    pub status: DiplomaticStatus,
    pub war_weariness: u32,
    pub turns_at_war: u32,
    pub turns_at_peace: u32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DiplomaticStatus {
    War,
    Hostile,
    Neutral,
    Friendly,
    Allied,
}

pub struct Treaty {
    pub id: u64,
    pub treaty_type: TreatyType,
    pub parties: Vec<PlayerId>,
    pub start_turn: u32,
    pub duration: u32,  // 0 = permanent
    pub terms: TreatyTerms,
}

#[derive(Clone, Copy)]
pub enum TreatyType {
    Peace,
    OpenBorders,
    DefensivePact,
    ResearchAgreement,
    TradeAgreement,
    NonAggression,
}

pub struct TreatyTerms {
    pub gold_payments: HashMap<PlayerId, i32>,
    pub gold_per_turn: HashMap<PlayerId, i32>,
    pub resources: HashMap<PlayerId, Vec<Resource>>,
    pub cities: HashMap<PlayerId, Vec<CityId>>,
}

pub struct DiplomaticProposal {
    pub id: u64,
    pub from: PlayerId,
    pub to: PlayerId,
    pub proposal_type: ProposalType,
    pub terms: TreatyTerms,
    pub turn_proposed: u32,
}

pub enum ProposalType {
    Treaty(TreatyType),
    Trade,
    Denouncement,
    Declaration(DiplomaticStatus),
}
```

## Technology

```rust
pub type TechId = String;

pub struct Technology {
    pub id: TechId,
    pub name: String,
    pub era: Era,
    pub cost: u32,
    pub prerequisites: Vec<TechId>,
    pub unlocks: TechUnlocks,
    pub quote: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Era {
    Ancient,
    Classical,
    Medieval,
    Renaissance,
    Industrial,
    Modern,
}

pub struct TechUnlocks {
    pub units: Vec<UnitType>,
    pub buildings: Vec<BuildingType>,
    pub wonders: Vec<WonderType>,
    pub improvements: Vec<Improvement>,
    pub abilities: Vec<String>,
}
```

## Actions (for Nostr Events)

```rust
/// All possible player actions
pub enum GameAction {
    // Unit actions
    MoveUnit { unit_id: UnitId, path: Vec<HexCoord> },
    AttackUnit { attacker_id: UnitId, defender_id: UnitId },
    AttackCity { attacker_id: UnitId, city_id: CityId },
    FoundCity { settler_id: UnitId, name: String },
    Fortify { unit_id: UnitId },
    Sleep { unit_id: UnitId },
    Delete { unit_id: UnitId },
    Embark { unit_id: UnitId },
    Disembark { unit_id: UnitId },
    
    // Worker actions
    BuildImprovement { unit_id: UnitId, improvement: Improvement },
    BuildRoad { unit_id: UnitId, road: Road },
    RemoveFeature { unit_id: UnitId },
    Repair { unit_id: UnitId },
    
    // City actions
    SetProduction { city_id: CityId, item: ProductionItem },
    BuyItem { city_id: CityId, item: ProductionItem },
    AssignCitizen { city_id: CityId, tile: HexCoord },
    UnassignCitizen { city_id: CityId, tile: HexCoord },
    SetSpecialist { city_id: CityId, specialist_type: SpecialistType, count: u32 },
    SellBuilding { city_id: CityId, building: BuildingType },
    
    // Research
    SetResearch { tech_id: TechId },
    
    // Diplomacy
    ProposeTreaty { to: PlayerId, treaty_type: TreatyType, terms: TreatyTerms },
    AcceptProposal { proposal_id: u64 },
    RejectProposal { proposal_id: u64 },
    CancelTreaty { treaty_id: u64 },
    DeclareWar { target: PlayerId },
    
    // Turn management
    EndTurn,
}

/// Result of validating/executing an action
pub struct ActionResult {
    pub success: bool,
    pub error: Option<String>,
    pub events: Vec<GameEvent>,
    pub random_needed: bool,
}

/// Events generated by actions (for UI updates)
pub enum GameEvent {
    UnitMoved { unit_id: UnitId, from: HexCoord, to: HexCoord },
    UnitDamaged { unit_id: UnitId, damage: u32, new_health: u32 },
    UnitDestroyed { unit_id: UnitId },
    UnitCreated { unit: Unit },
    CityFounded { city: City },
    CityGrew { city_id: CityId, new_population: u32 },
    CityProduced { city_id: CityId, item: ProductionItem },
    TechResearched { player_id: PlayerId, tech_id: TechId },
    TileRevealed { player_id: PlayerId, coord: HexCoord },
    BordersExpanded { city_id: CityId, new_tiles: Vec<HexCoord> },
    TreatyProposed { proposal: DiplomaticProposal },
    TreatyAccepted { treaty: Treaty },
    WarDeclared { attacker: PlayerId, defender: PlayerId },
    Victory { player_id: PlayerId, victory_type: VictoryType },
}
```
