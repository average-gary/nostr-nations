/**
 * Core game type definitions for Nostr Nations
 */

// Coordinate types
export interface HexCoord {
  q: number;
  r: number;
}

export interface CubeCoord {
  x: number;
  y: number;
  z: number;
}

// Terrain types
export type TerrainType =
  | 'plains'
  | 'grassland'
  | 'desert'
  | 'tundra'
  | 'snow'
  | 'coast'
  | 'ocean'
  | 'mountain';

export type TerrainFeature =
  | 'forest'
  | 'jungle'
  | 'marsh'
  | 'hills'
  | 'river'
  | 'oasis';

// Resource types
export type ResourceType =
  | 'wheat'
  | 'cattle'
  | 'fish'
  | 'iron'
  | 'horses'
  | 'coal'
  | 'oil'
  | 'gold'
  | 'gems'
  | 'marble'
  | 'stone';

export interface Resource {
  type: ResourceType;
  quantity: number;
}

// Tile definition
export interface HexTile {
  coord: HexCoord;
  terrain: TerrainType;
  features: TerrainFeature[];
  resource: Resource | null;
  improvement: string | null;
  owner: PlayerId | null;
  visibility: TileVisibility;
}

export type TileVisibility = 'hidden' | 'explored' | 'visible';

// Player types
export type PlayerId = string;

export type PlayerColor = 'blue' | 'red' | 'green' | 'yellow';

export interface Player {
  id: PlayerId;
  name: string;
  color: PlayerColor;
  civilization: CivilizationType;
  gold: number;
  science: number;
  culture: number;
  isHuman: boolean;
  nostrPubkey?: string;
}

export type CivilizationType =
  | 'rome'
  | 'greece'
  | 'egypt'
  | 'china'
  | 'india'
  | 'persia'
  | 'aztec'
  | 'inca';

// Unit types
export type UnitType =
  | 'settler'
  | 'warrior'
  | 'archer'
  | 'swordsman'
  | 'horseman'
  | 'catapult'
  | 'scout'
  | 'worker';

export interface Unit {
  id: string;
  type: UnitType;
  owner: PlayerId;
  position: HexCoord;
  health: number;
  maxHealth: number;
  movement: number;
  maxMovement: number;
  strength: number;
  promotions: string[];
  canAct: boolean;
}

// City types
export interface City {
  id: string;
  name: string;
  owner: PlayerId;
  position: HexCoord;
  population: number;
  isCapital: boolean;
  production: CityProduction;
  buildings: string[];
  tiles: HexCoord[];
  yields: CityYields;
}

export interface CityProduction {
  item: string | null;
  progress: number;
  total: number;
  turnsRemaining: number;
}

export interface CityYields {
  food: number;
  production: number;
  gold: number;
  science: number;
  culture: number;
}

// Technology types
export interface Technology {
  id: string;
  name: string;
  era: TechnologyEra;
  cost: number;
  prerequisites: string[];
  unlocks: string[];
}

export type TechnologyEra =
  | 'ancient'
  | 'classical'
  | 'medieval'
  | 'renaissance'
  | 'industrial'
  | 'modern';

// Diplomacy types
export type DiplomaticStatus =
  | 'unknown'
  | 'neutral'
  | 'friendly'
  | 'allied'
  | 'hostile'
  | 'war';

export interface DiplomaticRelation {
  playerId: PlayerId;
  status: DiplomaticStatus;
  treaties: Treaty[];
}

export interface Treaty {
  type: TreatyType;
  turnsRemaining: number;
}

export type TreatyType =
  | 'open_borders'
  | 'defensive_pact'
  | 'research_agreement'
  | 'trade_agreement';

// Game state types
export type GamePhase = 'setup' | 'playing' | 'ended';

export type VictoryCondition =
  | 'domination'
  | 'science'
  | 'culture'
  | 'diplomatic';

export interface GameSettings {
  mapSize: 'small' | 'medium' | 'large';
  playerCount: number;
  victoryConditions: VictoryCondition[];
  turnTimer: number | null;
}

export interface GameState {
  phase: GamePhase;
  turn: number;
  currentPlayer: PlayerId;
  players: Player[];
  map: HexTile[];
  units: Unit[];
  cities: City[];
  settings: GameSettings;
}

// Selection types
export type SelectionType = 'none' | 'tile' | 'unit' | 'city';

export interface Selection {
  type: SelectionType;
  id: string | null;
  coord: HexCoord | null;
}

// Action types (for commands sent to backend)
export interface GameAction {
  type: GameActionType;
  payload: Record<string, unknown>;
}

export type GameActionType =
  | 'move_unit'
  | 'attack'
  | 'found_city'
  | 'build_improvement'
  | 'set_production'
  | 'research_tech'
  | 'propose_trade'
  | 'declare_war'
  | 'end_turn';

// Notification types - aligned with backend NotificationPayload
export interface Notification {
  id: string;
  type: NotificationType;
  title: string;
  message: string;
  timestamp: number;
  dismissed: boolean;
  icon?: string;
  durationMs?: number;
  action?: NotificationAction;
}

// Matches backend NotificationType enum (snake_case from Rust serde)
export type NotificationType =
  | 'info'
  | 'success'
  | 'warning'
  | 'error'
  | 'achievement'
  | 'diplomacy'
  | 'research'
  | 'production'
  | 'combat';

// Action that can be triggered from a notification
export interface NotificationAction {
  actionType: string;
  label: string;
  data?: Record<string, unknown>;
}

// Backend notification payload structure (from Tauri events)
export interface NotificationPayload {
  notification_type: NotificationType;
  title: string;
  message: string;
  icon?: string;
  duration_ms?: number;
  action?: {
    action_type: string;
    label: string;
    data?: Record<string, unknown>;
  };
}
