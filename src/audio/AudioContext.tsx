import {
  createContext,
  useContext,
  useEffect,
  useCallback,
  useState,
  ReactNode,
} from 'react'
import { AudioManager } from './AudioManager'
import { SoundEffect, MusicTrack } from './sounds'
import { useSettingsStore, selectAudioSettings } from '../stores/settingsStore'

/**
 * Audio context value interface
 */
interface AudioContextValue {
  /** Whether audio has been initialized */
  isInitialized: boolean
  /** Initialize the audio system (requires user interaction) */
  initialize: () => Promise<void>
  /** Play a sound effect */
  playSound: (sound: SoundEffect) => void
  /** Play a music track */
  playMusic: (track: MusicTrack) => void
  /** Stop the current music */
  stopMusic: () => void
  /** Fade out the current music */
  fadeOutMusic: (duration?: number) => void
  /** Suspend audio playback */
  suspend: () => void
  /** Resume audio playback */
  resume: () => void
}

const AudioReactContext = createContext<AudioContextValue | null>(null)

/**
 * Props for the AudioProvider component
 */
interface AudioProviderProps {
  children: ReactNode
  /** Whether to auto-initialize on first user interaction (default: true) */
  autoInitialize?: boolean
}

/**
 * Audio provider component that integrates with the settings store
 * and provides audio functionality to the component tree
 */
export function AudioProvider({
  children,
  autoInitialize = true,
}: AudioProviderProps) {
  const [isInitialized, setIsInitialized] = useState(false)
  const audioSettings = useSettingsStore(selectAudioSettings)
  const audioManager = AudioManager.getInstance()

  // Sync audio manager with settings store whenever settings change
  useEffect(() => {
    audioManager.setMasterVolume(audioSettings.masterVolume)
    audioManager.setMusicVolume(audioSettings.musicVolume)
    audioManager.setSfxVolume(audioSettings.sfxVolume)
    audioManager.setMusicEnabled(audioSettings.musicEnabled)
    audioManager.setSfxEnabled(audioSettings.sfxEnabled)
  }, [audioSettings, audioManager])

  // Initialize audio
  const initialize = useCallback(async () => {
    if (isInitialized) {
      return
    }

    try {
      await audioManager.initialize()
      setIsInitialized(true)
    } catch (error) {
      console.error('Failed to initialize audio:', error)
    }
  }, [isInitialized, audioManager])

  // Auto-initialize on first user interaction
  useEffect(() => {
    if (!autoInitialize || isInitialized) {
      return
    }

    const handleUserInteraction = () => {
      initialize()
      // Remove listeners after first interaction
      document.removeEventListener('click', handleUserInteraction)
      document.removeEventListener('keydown', handleUserInteraction)
      document.removeEventListener('touchstart', handleUserInteraction)
    }

    document.addEventListener('click', handleUserInteraction)
    document.addEventListener('keydown', handleUserInteraction)
    document.addEventListener('touchstart', handleUserInteraction)

    return () => {
      document.removeEventListener('click', handleUserInteraction)
      document.removeEventListener('keydown', handleUserInteraction)
      document.removeEventListener('touchstart', handleUserInteraction)
    }
  }, [autoInitialize, isInitialized, initialize])

  // Handle visibility change (suspend/resume audio when tab is hidden/visible)
  useEffect(() => {
    const handleVisibilityChange = () => {
      if (document.hidden) {
        audioManager.suspend()
      } else {
        audioManager.resume()
      }
    }

    document.addEventListener('visibilitychange', handleVisibilityChange)

    return () => {
      document.removeEventListener('visibilitychange', handleVisibilityChange)
    }
  }, [audioManager])

  // Handle window focus/blur (additional fallback for visibility)
  useEffect(() => {
    const handleBlur = () => {
      audioManager.suspend()
    }

    const handleFocus = () => {
      audioManager.resume()
    }

    window.addEventListener('blur', handleBlur)
    window.addEventListener('focus', handleFocus)

    return () => {
      window.removeEventListener('blur', handleBlur)
      window.removeEventListener('focus', handleFocus)
    }
  }, [audioManager])

  // Playback functions
  const playSound = useCallback(
    (sound: SoundEffect) => {
      audioManager.playSound(sound)
    },
    [audioManager]
  )

  const playMusic = useCallback(
    (track: MusicTrack) => {
      audioManager.playMusic(track)
    },
    [audioManager]
  )

  const stopMusic = useCallback(() => {
    audioManager.stopMusic()
  }, [audioManager])

  const fadeOutMusic = useCallback(
    (duration: number = 1.0) => {
      audioManager.fadeOutMusic(duration)
    },
    [audioManager]
  )

  const suspend = useCallback(() => {
    audioManager.suspend()
  }, [audioManager])

  const resume = useCallback(() => {
    audioManager.resume()
  }, [audioManager])

  const contextValue: AudioContextValue = {
    isInitialized,
    initialize,
    playSound,
    playMusic,
    stopMusic,
    fadeOutMusic,
    suspend,
    resume,
  }

  return (
    <AudioReactContext.Provider value={contextValue}>
      {children}
    </AudioReactContext.Provider>
  )
}

/**
 * Hook to access the audio context
 * Must be used within an AudioProvider
 */
export function useAudioContext(): AudioContextValue {
  const context = useContext(AudioReactContext)

  if (!context) {
    throw new Error('useAudioContext must be used within an AudioProvider')
  }

  return context
}

/**
 * Optional hook that returns null if used outside AudioProvider
 * Useful for components that may or may not have audio available
 */
export function useAudioContextOptional(): AudioContextValue | null {
  return useContext(AudioReactContext)
}
