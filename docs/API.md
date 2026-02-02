# Tauri API Documentation

This document describes the IPC interface between the React frontend and the Rust backend.

## Commands

Commands are invoked from the frontend using `@tauri-apps/api/core`:

```typescript
import { invoke } from '@tauri-apps/api/core'

const result = await invoke('command_name', { arg1: 'value' })
```

### Game Lifecycle Commands

#### `create_game`

Create a new game session.

**Parameters:**

```typescript
{
  options: {
    name: string;           // Game name
    player_name: string;    // Local player name
    civilization: string;   // Civilization ID
    map_size: string;       // "duel" | "small" | "standard" | "large" | "huge"
    difficulty: string;     // "settler" | "chieftain" | "prince" | "king" | "emperor" | "deity"
    game_speed: string;     // "quick" | "normal" | "epic" | "marathon"
    seed?: string;          // Optional seed for deterministic map generation
  }
}
```

**Returns:**

```typescript
{
  game_id: string
  phase: string // "Setup" | "Playing" | "Ended"
  turn: number
  current_player: number
  player_count: number
  map_width: number
  map_height: number
}
```

**Example:**

```typescript
const gameState = await invoke('create_game', {
  options: {
    name: 'My Game',
    player_name: 'Player 1',
    civilization: 'rome',
    map_size: 'standard',
    difficulty: 'prince',
    game_speed: 'normal',
  },
})
```

---

#### `join_game`

Join an existing game.

**Parameters:**

```typescript
{
  player_name: string
  civilization: string
}
```

**Returns:** `GameStateResponse` (same as `create_game`)

---

#### `start_game`

Start the game (transitions from Setup to Playing phase).

**Parameters:** None

**Returns:** `GameStateResponse`

**Events Emitted:**

- `game_state_updated` - Full state update
- `turn_event` - Turn started notification
- `notification` - Game started message

---

#### `get_game_state`

Get the current game state.

**Parameters:** None

**Returns:** `GameStateResponse`

---

#### `end_turn`

End the current player's turn.

**Parameters:** None

**Returns:** `GameStateResponse`

**Events Emitted:**

- `turn_event` - Turn ended for current player
- `turn_event` - Turn started for next player (if new turn)
- `game_state_updated` - State update
- `notification` - "Your Turn" if local player's turn

---

### Game Action Commands

#### `move_unit`

Move a unit along a path.

**Parameters:**

```typescript
{
  unit_id: number;
  path: [number, number][];  // Array of [q, r] hex coordinates
}
```

**Returns:**

```typescript
{
  success: boolean;
  message?: string;
  effects: string[];  // List of effects that occurred
}
```

**Example:**

```typescript
const result = await invoke('move_unit', {
  unit_id: 1,
  path: [
    [5, 3],
    [5, 4],
    [6, 4],
  ],
})
```

---

#### `attack_unit`

Attack an enemy unit.

**Parameters:**

```typescript
{
  attacker_id: number
  defender_id: number
  random: number // Random value for combat (0.0 - 1.0)
}
```

**Returns:** `ActionResult`

**Events Emitted:**

- `combat_resolved` - Detailed combat results
- `game_state_updated` - Partial update with changed units
- `notification` - Combat outcome message

---

#### `found_city`

Found a new city with a settler unit.

**Parameters:**

```typescript
{
  settler_id: number
  name: string
}
```

**Returns:** `ActionResult`

---

#### `build_improvement`

Build a tile improvement with a worker unit.

**Parameters:**

```typescript
{
  unit_id: number
  improvement: string // "farm" | "mine" | "pasture" | "plantation" | "camp" | "quarry" | "lumber_mill" | "trading_post" | "fort"
}
```

**Returns:** `ActionResult`

---

#### `set_research`

Set the current research target.

**Parameters:**

```typescript
{
  tech_id: string
}
```

**Returns:** `ActionResult`

---

### Network Commands

#### `connect_peer`

Connect to a peer using a connection ticket.

**Parameters:**

```typescript
{
  ticket: string // Base64-encoded connection ticket
}
```

**Returns:**

```typescript
{
  connected: boolean;
  peer_count: number;
  ticket?: string;
}
```

**Events Emitted:**

- `network_event` - Peer connected
- `notification` - Connection success/failure

---

#### `disconnect_peer`

Disconnect from a peer.

**Parameters:**

```typescript
{
  peer_id: string
}
```

**Returns:** `ConnectionStatus`

**Events Emitted:**

- `network_event` - Peer disconnected
- `notification` - Disconnect confirmation

---

#### `get_connection_ticket`

Generate a connection ticket for others to connect.

**Parameters:** None

**Returns:** `string` (Base64-encoded ticket)

---

#### `scan_qr_code`

Parse a QR code containing a connection ticket.

**Parameters:**

```typescript
{
  qr_data: string // Raw QR code data (may include "nn:" prefix)
}
```

**Returns:**

```typescript
{
  node_id: string;
  addresses: string[];
  game_id?: string;
  expires_at: number;  // Unix timestamp
}
```

---

## Events

Events are emitted from the backend and can be listened to in the frontend:

```typescript
import { listen } from '@tauri-apps/api/event'

const unlisten = await listen('event_name', (event) => {
  console.log(event.payload)
})

// Clean up when done
unlisten()
```

### `game_state_updated`

Emitted when the game state changes.

**Payload:**

```typescript
{
  game_id: string;
  phase: string;
  turn: number;
  current_player: number;
  player_count: number;
  map_dimensions: [number, number];
  is_full_update: boolean;

  // Partial update fields (only present if is_full_update is false)
  changed_units?: {
    id: number;
    owner: number;
    unit_type: string;
    position: [number, number];
    health: number;
    movement_remaining: number;
    is_destroyed: boolean;
  }[];
  changed_cities?: {
    id: number;
    owner: number;
    name: string;
    position: [number, number];
    population: number;
    health: number;
  }[];
  changed_tiles?: {
    position: [number, number];
    improvement?: string;
    road?: string;
    owner?: number;
  }[];
}
```

---

### `turn_event`

Emitted for turn lifecycle events.

**Payload:**

```typescript
{
  event_type: "turn_started" | "turn_ended" | "player_turn";
  turn: number;
  player_id: number;
  player_name: string;
  previous_turn?: number;
  is_local_player: boolean;
}
```

---

### `combat_resolved`

Emitted when combat is resolved.

**Payload:**

```typescript
{
  attacker: {
    unit_id: number
    owner_id: number
    owner_name: string
    unit_type: string
    health_before: number
    health_after: number
    strength: number
  }
  defender: {
    // Same structure as attacker
  }
  results: {
    defender_damage: number
    attacker_damage: number
    defender_destroyed: boolean
    attacker_destroyed: boolean
    attacker_xp: number
    defender_xp: number
    was_ranged: boolean
  }
  position: [number, number]
  timestamp: number
}
```

---

### `network_event`

Emitted for P2P networking events.

**Payload:**

```typescript
{
  event_type: "peer_connected" | "peer_disconnected" | "sync_complete" | "sync_started" | "connection_error";
  peer_id?: string;
  peer_name?: string;
  peer_count: number;
  error_message?: string;
  sync_progress?: number;  // 0-100
}
```

---

### `notification`

Emitted for user-facing notifications.

**Payload:**

```typescript
{
  notification_type: "info" | "success" | "warning" | "error" | "achievement" | "diplomacy" | "research" | "production" | "combat";
  title: string;
  message: string;
  icon?: string;
  duration_ms?: number;  // null = user must dismiss
  action?: {
    action_type: string;
    label: string;
    data?: any;
  };
}
```

---

## Frontend Usage Examples

### Using the useTauri Hook

```typescript
import { useTauriCommand, useTauriEvent } from '../hooks/useTauri';

function GameComponent() {
  // Command with loading state
  const { data, isLoading, error, execute } = useTauriCommand<GameState>('get_game_state');

  // Listen to events
  useTauriEvent<TurnEventPayload>('turn_event', (payload) => {
    if (payload.is_local_player && payload.event_type === 'player_turn') {
      console.log('Your turn!');
    }
  });

  useEffect(() => {
    execute();
  }, []);

  if (isLoading) return <Loading />;
  if (error) return <Error message={error} />;

  return <GameView state={data} />;
}
```

### Creating a Game

```typescript
import { invoke } from '@tauri-apps/api/core'

async function createNewGame() {
  try {
    const state = await invoke('create_game', {
      options: {
        name: 'Epic Battle',
        player_name: 'Commander',
        civilization: 'greece',
        map_size: 'large',
        difficulty: 'king',
        game_speed: 'normal',
      },
    })

    console.log('Game created:', state.game_id)
  } catch (err) {
    console.error('Failed to create game:', err)
  }
}
```

### Handling Combat

```typescript
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

// Listen for combat results
await listen('combat_resolved', (event) => {
  const { attacker, defender, results } = event.payload

  if (results.defender_destroyed) {
    showNotification(`Victory! ${defender.unit_type} destroyed!`)
  } else {
    showNotification(`Combat: dealt ${results.defender_damage} damage`)
  }
})

// Initiate attack
async function attackUnit(attackerId: number, defenderId: number) {
  // In real implementation, random comes from Cashu proof
  const random = Math.random()

  const result = await invoke('attack_unit', {
    attacker_id: attackerId,
    defender_id: defenderId,
    random,
  })

  if (!result.success) {
    showError(result.message)
  }
}
```

### Network Connection

```typescript
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

// Get ticket for others to connect
const ticket = await invoke<string>('get_connection_ticket')
displayQRCode(ticket)

// Connect to a peer
async function connectToPeer(ticket: string) {
  try {
    const status = await invoke('connect_peer', { ticket })
    console.log(`Connected! ${status.peer_count} peers online`)
  } catch (err) {
    console.error('Connection failed:', err)
  }
}

// Monitor network status
await listen('network_event', (event) => {
  switch (event.payload.event_type) {
    case 'peer_connected':
      console.log(`Peer joined: ${event.payload.peer_name}`)
      break
    case 'sync_complete':
      console.log('Game state synchronized')
      break
    case 'connection_error':
      console.error('Network error:', event.payload.error_message)
      break
  }
})
```

## Error Handling

Commands may throw errors with the following structure:

```typescript
interface AppError {
  type:
    | 'NoActiveGame'
    | 'GameAlreadyActive'
    | 'InvalidState'
    | 'NetworkError'
    | 'SerializationError'
  message: string
}
```

Always wrap `invoke` calls in try-catch:

```typescript
try {
  const result = await invoke('command', params)
} catch (error) {
  // error is typically a string with the error message
  console.error('Command failed:', error)
}
```
