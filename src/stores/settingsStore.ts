import { create } from 'zustand';
import { persist } from 'zustand/middleware';

/**
 * Audio settings for the game
 */
export interface AudioSettings {
  /** Master volume (0.0 - 1.0) */
  masterVolume: number;
  /** Music volume (0.0 - 1.0) */
  musicVolume: number;
  /** Sound effects volume (0.0 - 1.0) */
  sfxVolume: number;
  /** Whether music is enabled */
  musicEnabled: boolean;
  /** Whether sound effects are enabled */
  sfxEnabled: boolean;
}

/**
 * Display settings for the game
 */
export interface DisplaySettings {
  /** Show grid overlay on the map */
  showGrid: boolean;
  /** Show yield icons on tiles */
  showYieldIcons: boolean;
  /** Fullscreen mode */
  fullscreen: boolean;
}

/**
 * Game-related settings
 */
export interface GamePreferences {
  /** Auto-save interval in turns (0 = disabled) */
  autoSaveInterval: 0 | 1 | 5 | 10;
  /** Turn timer in seconds for multiplayer (null = disabled) */
  turnTimer: number | null;
}

/**
 * Complete settings state
 */
export interface Settings {
  audio: AudioSettings;
  display: DisplaySettings;
  game: GamePreferences;
}

/**
 * Default settings values
 */
export const DEFAULT_SETTINGS: Settings = {
  audio: {
    masterVolume: 0.8,
    musicVolume: 0.7,
    sfxVolume: 0.8,
    musicEnabled: true,
    sfxEnabled: true,
  },
  display: {
    showGrid: true,
    showYieldIcons: true,
    fullscreen: false,
  },
  game: {
    autoSaveInterval: 5,
    turnTimer: null,
  },
};

/**
 * Settings store interface
 */
interface SettingsStore {
  settings: Settings;

  // Audio actions
  setMasterVolume: (volume: number) => void;
  setMusicVolume: (volume: number) => void;
  setSfxVolume: (volume: number) => void;
  setMusicEnabled: (enabled: boolean) => void;
  setSfxEnabled: (enabled: boolean) => void;

  // Display actions
  setShowGrid: (show: boolean) => void;
  setShowYieldIcons: (show: boolean) => void;
  setFullscreen: (fullscreen: boolean) => void;

  // Game actions
  setAutoSaveInterval: (interval: 0 | 1 | 5 | 10) => void;
  setTurnTimer: (seconds: number | null) => void;

  // Utility actions
  resetToDefaults: () => void;
  updateSettings: (partial: Partial<Settings>) => void;
}

/**
 * Clamp a value between min and max
 */
const clamp = (value: number, min: number, max: number): number => {
  return Math.max(min, Math.min(max, value));
};

/**
 * Settings store with localStorage persistence
 */
export const useSettingsStore = create<SettingsStore>()(
  persist(
    (set, get) => ({
      settings: { ...DEFAULT_SETTINGS },

      // Audio actions
      setMasterVolume: (volume) => {
        set((state) => ({
          settings: {
            ...state.settings,
            audio: {
              ...state.settings.audio,
              masterVolume: clamp(volume, 0, 1),
            },
          },
        }));
      },

      setMusicVolume: (volume) => {
        set((state) => ({
          settings: {
            ...state.settings,
            audio: {
              ...state.settings.audio,
              musicVolume: clamp(volume, 0, 1),
            },
          },
        }));
      },

      setSfxVolume: (volume) => {
        set((state) => ({
          settings: {
            ...state.settings,
            audio: {
              ...state.settings.audio,
              sfxVolume: clamp(volume, 0, 1),
            },
          },
        }));
      },

      setMusicEnabled: (enabled) => {
        set((state) => ({
          settings: {
            ...state.settings,
            audio: {
              ...state.settings.audio,
              musicEnabled: enabled,
            },
          },
        }));
      },

      setSfxEnabled: (enabled) => {
        set((state) => ({
          settings: {
            ...state.settings,
            audio: {
              ...state.settings.audio,
              sfxEnabled: enabled,
            },
          },
        }));
      },

      // Display actions
      setShowGrid: (show) => {
        set((state) => ({
          settings: {
            ...state.settings,
            display: {
              ...state.settings.display,
              showGrid: show,
            },
          },
        }));
      },

      setShowYieldIcons: (show) => {
        set((state) => ({
          settings: {
            ...state.settings,
            display: {
              ...state.settings.display,
              showYieldIcons: show,
            },
          },
        }));
      },

      setFullscreen: (fullscreen) => {
        set((state) => ({
          settings: {
            ...state.settings,
            display: {
              ...state.settings.display,
              fullscreen,
            },
          },
        }));

        // Actually toggle fullscreen in the browser/Tauri
        if (fullscreen) {
          document.documentElement.requestFullscreen?.().catch(() => {
            // Fullscreen request failed, revert state
            set((state) => ({
              settings: {
                ...state.settings,
                display: {
                  ...state.settings.display,
                  fullscreen: false,
                },
              },
            }));
          });
        } else {
          document.exitFullscreen?.().catch(() => {
            // Exit fullscreen failed, but we don't need to revert
          });
        }
      },

      // Game actions
      setAutoSaveInterval: (interval) => {
        set((state) => ({
          settings: {
            ...state.settings,
            game: {
              ...state.settings.game,
              autoSaveInterval: interval,
            },
          },
        }));
      },

      setTurnTimer: (seconds) => {
        set((state) => ({
          settings: {
            ...state.settings,
            game: {
              ...state.settings.game,
              turnTimer: seconds,
            },
          },
        }));
      },

      // Utility actions
      resetToDefaults: () => {
        const currentFullscreen = get().settings.display.fullscreen;

        set({ settings: { ...DEFAULT_SETTINGS } });

        // If we were in fullscreen, exit it
        if (currentFullscreen) {
          document.exitFullscreen?.().catch(() => {});
        }
      },

      updateSettings: (partial) => {
        set((state) => ({
          settings: {
            ...state.settings,
            ...partial,
            audio: {
              ...state.settings.audio,
              ...(partial.audio || {}),
            },
            display: {
              ...state.settings.display,
              ...(partial.display || {}),
            },
            game: {
              ...state.settings.game,
              ...(partial.game || {}),
            },
          },
        }));
      },
    }),
    {
      name: 'nostr-nations-settings',
      version: 1,
    }
  )
);

// Selectors
export const selectAudioSettings = (state: SettingsStore) => state.settings.audio;
export const selectDisplaySettings = (state: SettingsStore) => state.settings.display;
export const selectGamePreferences = (state: SettingsStore) => state.settings.game;

// Computed selectors for effective volumes
export const selectEffectiveMusicVolume = (state: SettingsStore) => {
  const { masterVolume, musicVolume, musicEnabled } = state.settings.audio;
  return musicEnabled ? masterVolume * musicVolume : 0;
};

export const selectEffectiveSfxVolume = (state: SettingsStore) => {
  const { masterVolume, sfxVolume, sfxEnabled } = state.settings.audio;
  return sfxEnabled ? masterVolume * sfxVolume : 0;
};
