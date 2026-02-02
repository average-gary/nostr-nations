# Nostr Nations - UI Design

## Overview

The UI is built with React for interface components and Three.js/WebGL for the hex map renderer. Communication with the Bevy backend occurs through Tauri's IPC system.

## Screen Flow

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Splash    │───►│  Main Menu  │───►│  Game Setup │
└─────────────┘    └─────────────┘    └─────────────┘
                         │                   │
                         │                   ▼
                         │            ┌─────────────┐
                         │            │   Lobby     │
                         │            │  (P2P Join) │
                         │            └─────────────┘
                         │                   │
                         ▼                   ▼
                   ┌─────────────┐    ┌─────────────┐
                   │ Load Game   │───►│  Game View  │
                   └─────────────┘    └─────────────┘
                                            │
                         ┌──────────────────┼──────────────────┐
                         ▼                  ▼                  ▼
                   ┌───────────┐     ┌───────────┐     ┌───────────┐
                   │ Tech Tree │     │ Diplomacy │     │  Victory  │
                   └───────────┘     └───────────┘     └───────────┘
```

## Main Menu

### Components

```
┌────────────────────────────────────────────────────────┐
│                                                        │
│                    NOSTR NATIONS                       │
│                                                        │
│              ┌────────────────────────┐                │
│              │      New Game          │                │
│              └────────────────────────┘                │
│              ┌────────────────────────┐                │
│              │      Join Game         │                │
│              └────────────────────────┘                │
│              ┌────────────────────────┐                │
│              │      Load Game         │                │
│              └────────────────────────┘                │
│              ┌────────────────────────┐                │
│              │      Settings          │                │
│              └────────────────────────┘                │
│              ┌────────────────────────┐                │
│              │        Exit            │                │
│              └────────────────────────┘                │
│                                                        │
│  Version 1.0.0                     Nostr: connected    │
└────────────────────────────────────────────────────────┘
```

### New Game Flow

1. **Game Settings Screen**
   - Map size selection
   - Number of players (2-4)
   - Victory conditions (checkboxes)
   - Turn timer (optional)
   - Game name

2. **Player Setup**
   - Civilization selection
   - Color selection
   - Starting position preference

3. **Lobby (if multiplayer)**
   - Show QR code for joining
   - List connected players
   - Chat functionality
   - Start when all ready

### Join Game Flow

1. **Scan QR Code** or enter connection string
2. **Connecting** indicator
3. **Lobby** (waiting for host)

## Game View

### Main Game Layout

```
┌─────────────────────────────────────────────────────────────────┐
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │                        Top Bar                               │ │
│ │  Gold: 234  Science: 45  Culture: 12  Turn: 42  [End Turn]  │ │
│ └─────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│ │                                                             │ │
│ │                                                             │ │
│ │                                                             │ │
│ │                    HEX MAP CANVAS                           │ │
│ │                    (Three.js WebGL)                         │ │
│ │                                                             │ │
│ │                                                             │ │
│ │                                                             │ │
│ │                                                             │ │
│ ├───────────┐                                   ┌─────────────┤ │
│ │  Minimap  │                                   │  Selection  │ │
│ │           │                                   │    Panel    │ │
│ │   ┌───┐   │                                   │             │ │
│ │   │   │   │                                   │  [Unit]     │ │
│ │   └───┘   │                                   │  HP: 80/100 │ │
│ └───────────┘                                   └─────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │  [Tech] [Civics] [Diplomacy] [Military] [Cities] [Leaders]  │ │
│ └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Top Bar Components

- **Resources Display**: Gold, Science, Culture, Faith
- **Turn Counter**: Current turn number
- **Turn Indicator**: Whose turn / time remaining
- **End Turn Button**: Prominent action button
- **Menu Button**: Access settings, save, quit

### Hex Map Renderer

#### Three.js Implementation

```typescript
// Map renderer setup
const renderer = new THREE.WebGLRenderer({ antialias: true });
const scene = new THREE.Scene();
const camera = new THREE.OrthographicCamera(...);

// Hex geometry (reusable)
const hexGeometry = createHexGeometry(HEX_SIZE);

// Instanced rendering for performance
const hexInstances = new THREE.InstancedMesh(
  hexGeometry,
  hexMaterial,
  MAX_VISIBLE_HEXES
);
```

#### Map Features

- **Pan**: Click and drag to pan
- **Zoom**: Scroll wheel / pinch to zoom
- **Tile Selection**: Click to select
- **Unit Selection**: Click on unit
- **Movement Preview**: Show valid moves
- **Attack Preview**: Show attackable targets
- **Fog of War**: Shader-based darkness

#### Visual Elements

- **Terrain Textures**: Distinct per terrain type
- **Resources**: Icons on tiles
- **Units**: Sprites with health bars
- **Cities**: Growing sprites with population
- **Borders**: Colored territory edges
- **Roads/Railroads**: Connecting lines
- **Rivers**: Edge decorations

### Selection Panel

Shows details of selected item:

**Unit Selected:**
```
┌─────────────────────────┐
│  WARRIOR                │
│  ════════════════════   │
│  HP: 80/100  ████████░░ │
│  Movement: 2/2          │
│  Strength: 8            │
│                         │
│  Promotions:            │
│  • Shock I              │
│                         │
│  [Fortify] [Skip] [Del] │
└─────────────────────────┘
```

**City Selected:**
```
┌─────────────────────────┐
│  ROME (Capital)         │
│  ════════════════════   │
│  Population: 12         │
│  Growth: 5 turns        │
│                         │
│  Producing: Swordsman   │
│  Progress: ████░░ 60%   │
│  4 turns remaining      │
│                         │
│  [Change Production]    │
│  [Manage Citizens]      │
└─────────────────────────┘
```

**Tile Selected:**
```
┌─────────────────────────┐
│  GRASSLAND (Hills)      │
│  ════════════════════   │
│  Yields:                │
│  • Food: 2              │
│  • Production: 2        │
│                         │
│  Resource: Iron         │
│  Improvement: Mine      │
│                         │
│  Owner: Rome            │
└─────────────────────────┘
```

### Minimap

- Shows entire map at small scale
- Current view indicated by rectangle
- Click to jump to location
- Color-coded by owner
- Toggle visibility layers

### Bottom Bar

Quick access buttons:
- **Tech Tree**: Open technology view
- **Diplomacy**: Open diplomacy screen
- **Military Overview**: List all units
- **City List**: List all cities
- **Leaders**: View other players

## Technology Tree Screen

### Layout

```
┌─────────────────────────────────────────────────────────────────┐
│  Technology Tree                              [Research: 45/t]  │
├─────────────────────────────────────────────────────────────────┤
│  [Ancient] [Classical] [Medieval] [Renaissance] [Industrial]   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│    ┌─────┐        ┌─────┐        ┌─────┐                       │
│    │Agri │───────►│Pott │───────►│Writ │                       │
│    │cult │        │ery  │        │ing  │                       │
│    └─────┘        └─────┘        └─────┘                       │
│        │              │              │                          │
│        ▼              │              ▼                          │
│    ┌─────┐            │         ┌─────┐                        │
│    │Min- │────────────┴────────►│Cal- │                        │
│    │ing  │                      │endar│                        │
│    └─────┘                      └─────┘                        │
│                                                                 │
│  Selected: Writing (55 science)                                 │
│  Unlocks: Library, Open Borders                                 │
│  Turns: 2                                [Set Research]         │
└─────────────────────────────────────────────────────────────────┘
```

### Interaction

- Click tech to select
- Double-click to set as research target
- Grayed out = unavailable
- Green border = available
- Blue = currently researching
- Completed = full color

## Diplomacy Screen

### Leader Overview

```
┌─────────────────────────────────────────────────────────────────┐
│  Diplomacy                                                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────────┐  ┌──────────────────┐                    │
│  │  [Avatar]        │  │  [Avatar]        │                    │
│  │  Player 2        │  │  Player 3        │                    │
│  │  FRIENDLY        │  │  HOSTILE         │                    │
│  │                  │  │                  │                    │
│  │  [Propose Deal]  │  │  [Propose Deal]  │                    │
│  │  [Denounce]      │  │  [Declare War]   │                    │
│  └──────────────────┘  └──────────────────┘                    │
│                                                                 │
│  Active Treaties:                                               │
│  • Open Borders with Player 2 (15 turns)                       │
│  • Research Agreement with Player 3 (8 turns)                  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Trade Deal Screen

```
┌─────────────────────────────────────────────────────────────────┐
│  Trade with Player 2                                            │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────────────┐  ┌──────────────────────┐            │
│  │  Your Offer          │  │  Their Offer         │            │
│  │                      │  │                      │            │
│  │  • 10 Gold/Turn      │  │  • Iron              │            │
│  │  • Open Borders      │  │  • 50 Gold           │            │
│  │                      │  │                      │            │
│  │  [Add Item ▼]        │  │  [Request Item ▼]    │            │
│  └──────────────────────┘  └──────────────────────┘            │
│                                                                 │
│  Deal Status: [FAIR]                                            │
│                                                                 │
│  [Cancel]                          [Propose Deal]               │
└─────────────────────────────────────────────────────────────────┘
```

## City Screen

### Production Management

```
┌─────────────────────────────────────────────────────────────────┐
│  Rome - Population 12                                           │
├─────────────────────────────────────────────────────────────────┤
│  ┌────────────────────────────────┐  ┌────────────────────────┐ │
│  │  City View                     │  │  Production Queue      │ │
│  │  [3D city visualization]       │  │                        │ │
│  │                                │  │  1. Swordsman (4t)     │ │
│  │                                │  │  2. Library (8t)       │ │
│  │                                │  │  3. Walls (6t)         │ │
│  │                                │  │                        │ │
│  │                                │  │  [Add to Queue]        │ │
│  └────────────────────────────────┘  └────────────────────────┘ │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │  Yields:  Food: 24  Prod: 18  Gold: 12  Science: 8         │ │
│  └────────────────────────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │  Buildings: Granary, Monument, Library, Barracks           │ │
│  └────────────────────────────────────────────────────────────┘ │
│  [Manage Citizens]  [Buy with Gold]  [Demolish Building]        │
└─────────────────────────────────────────────────────────────────┘
```

## Notifications

### Turn Notifications

Displayed when:
- Research completed
- Production completed
- City grows
- Unit needs orders
- Diplomatic proposal received
- Combat results

### Notification Queue

```
┌─────────────────────────────────────┐
│  Research Complete!                 │
│  You have discovered Writing.       │
│  [View Tech Tree]  [Choose Next]    │
└─────────────────────────────────────┘
```

## Responsive Design

### Breakpoints

- **Desktop**: Full layout as shown
- **Tablet**: Collapsible side panels
- **Mobile Light Client**: Simplified touch UI

### Touch Controls

- Tap: Select
- Double tap: Action
- Pinch: Zoom
- Two-finger drag: Pan
- Long press: Context menu

## Theme and Style

### Color Palette

- **Primary**: Deep blue (#1a365d)
- **Secondary**: Gold accent (#d69e2e)
- **Background**: Dark slate (#1a202c)
- **Text**: Off-white (#f7fafc)
- **Success**: Green (#48bb78)
- **Danger**: Red (#f56565)
- **Warning**: Orange (#ed8936)

### Typography

- **Headers**: Serif font (Cinzel or similar)
- **Body**: Sans-serif (Inter or similar)
- **Monospace**: For numbers, codes

### Player Colors

Default civilization colors:
1. Blue (#3182ce)
2. Red (#e53e3e)
3. Green (#38a169)
4. Yellow (#d69e2e)
