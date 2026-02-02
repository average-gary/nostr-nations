import { useCallback, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

/**
 * Hook for checking if running in Tauri environment
 */
export function useIsTauri(): boolean {
  const [isTauri, setIsTauri] = useState(false);

  useEffect(() => {
    // Check if we're in a Tauri environment
    setIsTauri(typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window);
  }, []);

  return isTauri;
}

/**
 * Hook for invoking Tauri commands with loading and error states
 */
export function useTauriCommand<T, Args extends unknown[] = []>(
  command: string
) {
  const [data, setData] = useState<T | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const isTauri = useIsTauri();

  const execute = useCallback(
    async (...args: Args): Promise<T | null> => {
      if (!isTauri) {
        console.warn(`Tauri not available, skipping command: ${command}`);
        return null;
      }

      setIsLoading(true);
      setError(null);

      try {
        const result = await invoke<T>(command, args[0] as Record<string, unknown>);
        setData(result);
        return result;
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        setError(errorMessage);
        console.error(`Tauri command "${command}" failed:`, err);
        return null;
      } finally {
        setIsLoading(false);
      }
    },
    [command, isTauri]
  );

  return { data, isLoading, error, execute };
}

/**
 * Hook for listening to Tauri events
 */
export function useTauriEvent<T>(
  eventName: string,
  handler: (payload: T) => void
) {
  const isTauri = useIsTauri();

  useEffect(() => {
    if (!isTauri) return;

    let unlisten: UnlistenFn | undefined;

    const setupListener = async () => {
      try {
        unlisten = await listen<T>(eventName, (event) => {
          handler(event.payload);
        });
      } catch (err) {
        console.error(`Failed to listen to event "${eventName}":`, err);
      }
    };

    setupListener();

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [eventName, handler, isTauri]);
}

/**
 * Game-specific Tauri commands
 */
export function useGameCommands() {
  const isTauri = useIsTauri();

  const newGame = useCallback(
    async (settings: {
      mapSize: string;
      playerCount: number;
      victoryConditions: string[];
    }) => {
      if (!isTauri) {
        console.warn('Tauri not available, using mock data');
        return null;
      }
      return invoke('new_game', { settings });
    },
    [isTauri]
  );

  const loadGame = useCallback(
    async (saveId: string) => {
      if (!isTauri) return null;
      return invoke('load_game', { saveId });
    },
    [isTauri]
  );

  const saveGame = useCallback(
    async (gameState: unknown) => {
      if (!isTauri) return null;
      return invoke('save_game', { gameState });
    },
    [isTauri]
  );

  const endTurn = useCallback(async () => {
    if (!isTauri) return null;
    return invoke('end_turn');
  }, [isTauri]);

  const moveUnit = useCallback(
    async (unitId: string, destination: { q: number; r: number }) => {
      if (!isTauri) return null;
      return invoke('move_unit', { unitId, destination });
    },
    [isTauri]
  );

  const foundCity = useCallback(
    async (unitId: string) => {
      if (!isTauri) return null;
      return invoke('found_city', { unitId });
    },
    [isTauri]
  );

  const setProduction = useCallback(
    async (cityId: string, productionItem: string) => {
      if (!isTauri) return null;
      return invoke('set_production', { cityId, productionItem });
    },
    [isTauri]
  );

  const researchTechnology = useCallback(
    async (techId: string) => {
      if (!isTauri) return null;
      return invoke('research_technology', { techId });
    },
    [isTauri]
  );

  return {
    newGame,
    loadGame,
    saveGame,
    endTurn,
    moveUnit,
    foundCity,
    setProduction,
    researchTechnology,
  };
}

/**
 * Hook for Nostr-specific commands
 */
export function useNostrCommands() {
  const isTauri = useIsTauri();

  const getPublicKey = useCallback(async () => {
    if (!isTauri) return null;
    return invoke<string>('nostr_get_public_key');
  }, [isTauri]);

  const signEvent = useCallback(
    async (eventData: unknown) => {
      if (!isTauri) return null;
      return invoke('nostr_sign_event', { eventData });
    },
    [isTauri]
  );

  const publishEvent = useCallback(
    async (signedEvent: unknown) => {
      if (!isTauri) return null;
      return invoke('nostr_publish_event', { signedEvent });
    },
    [isTauri]
  );

  const subscribeToGame = useCallback(
    async (gameId: string) => {
      if (!isTauri) return null;
      return invoke('nostr_subscribe_game', { gameId });
    },
    [isTauri]
  );

  return {
    getPublicKey,
    signEvent,
    publishEvent,
    subscribeToGame,
  };
}
