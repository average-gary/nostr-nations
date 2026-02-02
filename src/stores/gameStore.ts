import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';
import type {
  GameState,
  GamePhase,
  Player,
  Unit,
  City,
  HexTile,
  Selection,
  Notification,
  NotificationPayload,
  HexCoord,
  GameSettings,
  TileVisibility,
} from '@/types/game';

interface GameStore {
  // Game state
  gameState: GameState | null;
  isLoading: boolean;
  error: string | null;

  // UI state
  selection: Selection;
  notifications: Notification[];
  isPaused: boolean;

  // Camera state (for 3D view)
  cameraPosition: { x: number; y: number; z: number };
  cameraZoom: number;

  // Actions - Game lifecycle
  initializeGame: (settings?: Partial<GameSettings>) => void;
  loadGame: (gameId: string) => Promise<void>;
  saveGame: () => Promise<void>;
  endTurn: () => void;

  // Actions - Selection
  selectTile: (coord: HexCoord) => void;
  selectUnit: (unitId: string) => void;
  selectCity: (cityId: string) => void;
  clearSelection: () => void;

  // Actions - Units
  moveUnit: (unitId: string, destination: HexCoord) => void;
  attackWithUnit: (unitId: string, targetCoord: HexCoord) => void;

  // Actions - Cities
  setProduction: (cityId: string, productionItem: string) => void;
  foundCity: (unitId: string) => void;

  // Actions - Fog of War
  updateVisibility: (playerId: string) => void;
  revealTile: (coord: HexCoord) => void;
  revealArea: (center: HexCoord, radius: number) => void;

  // Actions - Camera
  setCameraPosition: (position: { x: number; y: number; z: number }) => void;
  setCameraZoom: (zoom: number) => void;

  // Actions - Notifications
  addNotification: (notification: Omit<Notification, 'id' | 'timestamp' | 'dismissed'>) => void;
  addNotificationFromBackend: (payload: NotificationPayload) => void;
  dismissNotification: (notificationId: string) => void;
  clearNotifications: () => void;
  removeNotification: (notificationId: string) => void;

  // Actions - UI
  setPaused: (paused: boolean) => void;
  setError: (error: string | null) => void;
}

// Helper to generate unique IDs
const generateId = () => Math.random().toString(36).substring(2, 11);

// Create default game state for new games
const createDefaultGameState = (settings?: Partial<GameSettings>): GameState => {
  const defaultSettings: GameSettings = {
    mapSize: 'medium',
    playerCount: 2,
    victoryConditions: ['domination', 'science'],
    turnTimer: null,
    ...settings,
  };

  const players: Player[] = [
    {
      id: 'player-1',
      name: 'Player 1',
      color: 'blue',
      civilization: 'rome',
      gold: 100,
      science: 0,
      culture: 0,
      isHuman: true,
    },
    {
      id: 'player-2',
      name: 'AI Player',
      color: 'red',
      civilization: 'greece',
      gold: 100,
      science: 0,
      culture: 0,
      isHuman: false,
    },
  ];

  // Generate a simple starter map (would be replaced by Rust backend)
  const map: HexTile[] = [];
  const mapRadius = 5;

  for (let q = -mapRadius; q <= mapRadius; q++) {
    for (let r = -mapRadius; r <= mapRadius; r++) {
      if (Math.abs(q + r) <= mapRadius) {
        map.push({
          coord: { q, r },
          terrain: Math.random() > 0.3 ? 'grassland' : 'plains',
          features: Math.random() > 0.7 ? ['forest'] : [],
          resource: null,
          improvement: null,
          owner: null,
          // Start with all tiles hidden - fog of war will reveal them
          visibility: 'hidden',
        });
      }
    }
  }

  const units: Unit[] = [
    {
      id: 'unit-1',
      type: 'settler',
      owner: 'player-1',
      position: { q: 0, r: 0 },
      health: 100,
      maxHealth: 100,
      movement: 2,
      maxMovement: 2,
      strength: 0,
      promotions: [],
      canAct: true,
    },
    {
      id: 'unit-2',
      type: 'warrior',
      owner: 'player-1',
      position: { q: 1, r: 0 },
      health: 100,
      maxHealth: 100,
      movement: 2,
      maxMovement: 2,
      strength: 8,
      promotions: [],
      canAct: true,
    },
  ];

  return {
    phase: 'playing',
    turn: 1,
    currentPlayer: 'player-1',
    players,
    map,
    units,
    cities: [],
    settings: defaultSettings,
  };
};

export const useGameStore = create<GameStore>()(
  subscribeWithSelector((set, get) => ({
    // Initial state
    gameState: null,
    isLoading: false,
    error: null,
    selection: { type: 'none', id: null, coord: null },
    notifications: [],
    isPaused: false,
    cameraPosition: { x: 0, y: 10, z: 10 },
    cameraZoom: 1,

    // Game lifecycle actions
    initializeGame: (settings) => {
      set({
        gameState: createDefaultGameState(settings),
        isLoading: false,
        error: null,
        selection: { type: 'none', id: null, coord: null },
        notifications: [],
      });

      // Initialize fog of war - reveal tiles around starting units
      const newState = get().gameState;
      if (newState) {
        // Update visibility for human player
        get().updateVisibility('player-1');
      }
    },

    loadGame: async (gameId: string) => {
      set({ isLoading: true, error: null });
      try {
        // TODO: Call Tauri backend to load game
        console.log('Loading game:', gameId);
        // For now, just create a new game
        set({
          gameState: createDefaultGameState(),
          isLoading: false,
        });
      } catch (error) {
        set({
          error: error instanceof Error ? error.message : 'Failed to load game',
          isLoading: false,
        });
      }
    },

    saveGame: async () => {
      const { gameState } = get();
      if (!gameState) return;

      set({ isLoading: true });
      try {
        // TODO: Call Tauri backend to save game
        console.log('Saving game...');
        set({ isLoading: false });
      } catch (error) {
        set({
          error: error instanceof Error ? error.message : 'Failed to save game',
          isLoading: false,
        });
      }
    },

    endTurn: () => {
      const { gameState } = get();
      if (!gameState) return;

      // Reset unit movement and actions
      const refreshedUnits = gameState.units.map((unit) => ({
        ...unit,
        movement: unit.maxMovement,
        canAct: true,
      }));

      set({
        gameState: {
          ...gameState,
          turn: gameState.turn + 1,
          units: refreshedUnits,
        },
      });

      // Add turn notification
      get().addNotification({
        type: 'info',
        title: `Turn ${gameState.turn + 1}`,
        message: 'A new turn has begun.',
      });
    },

    // Selection actions
    selectTile: (coord) => {
      set({
        selection: {
          type: 'tile',
          id: `${coord.q},${coord.r}`,
          coord,
        },
      });
    },

    selectUnit: (unitId) => {
      const { gameState } = get();
      const unit = gameState?.units.find((u) => u.id === unitId);
      set({
        selection: {
          type: 'unit',
          id: unitId,
          coord: unit?.position ?? null,
        },
      });
    },

    selectCity: (cityId) => {
      const { gameState } = get();
      const city = gameState?.cities.find((c) => c.id === cityId);
      set({
        selection: {
          type: 'city',
          id: cityId,
          coord: city?.position ?? null,
        },
      });
    },

    clearSelection: () => {
      set({ selection: { type: 'none', id: null, coord: null } });
    },

    // Unit actions
    moveUnit: (unitId, destination) => {
      const { gameState } = get();
      if (!gameState) return;

      const movedUnit = gameState.units.find((u) => u.id === unitId);
      if (!movedUnit) return;

      const updatedUnits = gameState.units.map((unit) =>
        unit.id === unitId
          ? {
              ...unit,
              position: destination,
              movement: Math.max(0, unit.movement - 1),
              canAct: unit.movement > 1,
            }
          : unit
      );

      set({
        gameState: { ...gameState, units: updatedUnits },
      });

      // Update fog of war when unit moves
      get().updateVisibility(movedUnit.owner);
    },

    attackWithUnit: (unitId, targetCoord) => {
      const { gameState } = get();
      if (!gameState) return;

      // TODO: Implement combat logic via Tauri backend
      console.log('Attack:', unitId, 'at', targetCoord);
    },

    // City actions
    setProduction: (cityId, productionItem) => {
      const { gameState } = get();
      if (!gameState) return;

      const updatedCities = gameState.cities.map((city) =>
        city.id === cityId
          ? {
              ...city,
              production: {
                ...city.production,
                item: productionItem,
                progress: 0,
              },
            }
          : city
      );

      set({
        gameState: { ...gameState, cities: updatedCities },
      });
    },

    foundCity: (unitId) => {
      const { gameState } = get();
      if (!gameState) return;

      const settler = gameState.units.find(
        (u) => u.id === unitId && u.type === 'settler'
      );
      if (!settler) return;

      const newCity: City = {
        id: generateId(),
        name: `City ${gameState.cities.length + 1}`,
        owner: settler.owner,
        position: settler.position,
        population: 1,
        isCapital: gameState.cities.filter((c) => c.owner === settler.owner).length === 0,
        production: {
          item: null,
          progress: 0,
          total: 0,
          turnsRemaining: 0,
        },
        buildings: [],
        tiles: [settler.position],
        yields: {
          food: 2,
          production: 1,
          gold: 1,
          science: 0,
          culture: 0,
        },
      };

      set({
        gameState: {
          ...gameState,
          cities: [...gameState.cities, newCity],
          units: gameState.units.filter((u) => u.id !== unitId),
        },
      });

      get().addNotification({
        type: 'production',
        title: 'City Founded!',
        message: `${newCity.name} has been founded.`,
      });

      // Update visibility around the new city
      get().updateVisibility(settler.owner);
    },

    // Fog of War actions
    updateVisibility: (playerId) => {
      const { gameState } = get();
      if (!gameState) return;

      // Get all units and cities owned by this player
      const playerUnits = gameState.units.filter((u) => u.owner === playerId);
      const playerCities = gameState.cities.filter((c) => c.owner === playerId);

      // Define sight ranges for different unit types
      const getSightRange = (unitType: string): number => {
        switch (unitType) {
          case 'scout':
            return 3;
          case 'archer':
            return 2;
          default:
            return 2;
        }
      };

      // City sight range
      const citySightRange = 2;

      // Calculate hex distance
      const hexDistance = (a: HexCoord, b: HexCoord): number => {
        return Math.max(
          Math.abs(a.q - b.q),
          Math.abs(a.r - b.r),
          Math.abs(a.q + a.r - b.q - b.r)
        );
      };

      // Create a set of visible tile keys
      const visibleTileKeys = new Set<string>();

      // Add tiles visible from units
      playerUnits.forEach((unit) => {
        const sightRange = getSightRange(unit.type);
        gameState.map.forEach((tile) => {
          if (hexDistance(unit.position, tile.coord) <= sightRange) {
            visibleTileKeys.add(`${tile.coord.q},${tile.coord.r}`);
          }
        });
      });

      // Add tiles visible from cities
      playerCities.forEach((city) => {
        gameState.map.forEach((tile) => {
          if (hexDistance(city.position, tile.coord) <= citySightRange) {
            visibleTileKeys.add(`${tile.coord.q},${tile.coord.r}`);
          }
        });
      });

      // Update tile visibility
      const updatedMap = gameState.map.map((tile) => {
        const key = `${tile.coord.q},${tile.coord.r}`;
        const isNowVisible = visibleTileKeys.has(key);

        let newVisibility: TileVisibility;
        if (isNowVisible) {
          newVisibility = 'visible';
        } else if (tile.visibility === 'visible') {
          // Was visible, now explored
          newVisibility = 'explored';
        } else {
          // Keep current state (hidden or explored)
          newVisibility = tile.visibility;
        }

        return tile.visibility !== newVisibility
          ? { ...tile, visibility: newVisibility }
          : tile;
      });

      set({
        gameState: { ...gameState, map: updatedMap },
      });
    },

    revealTile: (coord) => {
      const { gameState } = get();
      if (!gameState) return;

      const updatedMap = gameState.map.map((tile) =>
        tile.coord.q === coord.q && tile.coord.r === coord.r
          ? { ...tile, visibility: 'visible' as TileVisibility }
          : tile
      );

      set({
        gameState: { ...gameState, map: updatedMap },
      });
    },

    revealArea: (center, radius) => {
      const { gameState } = get();
      if (!gameState) return;

      const hexDistance = (a: HexCoord, b: HexCoord): number => {
        return Math.max(
          Math.abs(a.q - b.q),
          Math.abs(a.r - b.r),
          Math.abs(a.q + a.r - b.q - b.r)
        );
      };

      const updatedMap = gameState.map.map((tile) =>
        hexDistance(center, tile.coord) <= radius
          ? { ...tile, visibility: 'visible' as TileVisibility }
          : tile
      );

      set({
        gameState: { ...gameState, map: updatedMap },
      });
    },

    // Camera actions
    setCameraPosition: (position) => set({ cameraPosition: position }),
    setCameraZoom: (zoom) => set({ cameraZoom: Math.max(0.5, Math.min(3, zoom)) }),

    // Notification actions
    addNotification: (notification) => {
      const newNotification: Notification = {
        ...notification,
        id: generateId(),
        timestamp: Date.now(),
        dismissed: false,
      };
      set((state) => ({
        notifications: [...state.notifications, newNotification],
      }));
    },

    addNotificationFromBackend: (payload: NotificationPayload) => {
      const newNotification: Notification = {
        id: generateId(),
        type: payload.notification_type,
        title: payload.title,
        message: payload.message,
        timestamp: Date.now(),
        dismissed: false,
        icon: payload.icon,
        durationMs: payload.duration_ms,
        action: payload.action
          ? {
              actionType: payload.action.action_type,
              label: payload.action.label,
              data: payload.action.data,
            }
          : undefined,
      };
      set((state) => ({
        notifications: [...state.notifications, newNotification],
      }));
    },

    dismissNotification: (notificationId) => {
      set((state) => ({
        notifications: state.notifications.map((n) =>
          n.id === notificationId ? { ...n, dismissed: true } : n
        ),
      }));
    },

    removeNotification: (notificationId) => {
      set((state) => ({
        notifications: state.notifications.filter((n) => n.id !== notificationId),
      }));
    },

    clearNotifications: () => set({ notifications: [] }),

    // UI actions
    setPaused: (paused) => set({ isPaused: paused }),
    setError: (error) => set({ error }),
  }))
);

// Selectors for commonly accessed state slices
export const selectCurrentPlayer = (state: GameStore) =>
  state.gameState?.players.find((p) => p.id === state.gameState?.currentPlayer);

export const selectSelectedUnit = (state: GameStore) =>
  state.selection.type === 'unit'
    ? state.gameState?.units.find((u) => u.id === state.selection.id)
    : null;

export const selectSelectedCity = (state: GameStore) =>
  state.selection.type === 'city'
    ? state.gameState?.cities.find((c) => c.id === state.selection.id)
    : null;

export const selectSelectedTile = (state: GameStore) =>
  state.selection.type === 'tile' && state.selection.coord
    ? state.gameState?.map.find(
        (t) =>
          t.coord.q === state.selection.coord!.q &&
          t.coord.r === state.selection.coord!.r
      )
    : null;
