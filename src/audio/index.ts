// Core audio manager
export { AudioManager, audioManager } from './AudioManager'

// Sound and music definitions
export { SoundEffect, MusicTrack, SOUND_PATHS, MUSIC_PATHS } from './sounds'

// React hooks
export {
  useAudio,
  useUISounds,
  useGameSounds,
  useNotificationSounds,
} from './useAudio'

// React context
export {
  AudioProvider,
  useAudioContext,
  useAudioContextOptional,
} from './AudioContext'
