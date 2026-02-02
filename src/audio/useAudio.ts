import { useCallback, useEffect, useRef } from 'react'
import { AudioManager } from './AudioManager'
import { SoundEffect, MusicTrack } from './sounds'
import { useSettingsStore, selectAudioSettings } from '../stores/settingsStore'

/**
 * React hook for audio playback and control
 * Integrates with the settings store for volume and enabled state
 */
export function useAudio() {
  const audioManager = useRef<AudioManager>(AudioManager.getInstance())
  const audioSettings = useSettingsStore(selectAudioSettings)
  const initialized = useRef(false)

  // Sync audio manager with settings store
  useEffect(() => {
    const manager = audioManager.current
    manager.setMasterVolume(audioSettings.masterVolume)
    manager.setMusicVolume(audioSettings.musicVolume)
    manager.setSfxVolume(audioSettings.sfxVolume)
    manager.setMusicEnabled(audioSettings.musicEnabled)
    manager.setSfxEnabled(audioSettings.sfxEnabled)
  }, [audioSettings])

  // Initialize audio context on first user interaction
  const initializeAudio = useCallback(async () => {
    if (initialized.current) {
      return
    }

    try {
      await audioManager.current.initialize()
      initialized.current = true
    } catch (error) {
      console.error('Failed to initialize audio:', error)
    }
  }, [])

  /**
   * Play a sound effect
   * Will attempt to initialize audio context if not already done
   */
  const playSound = useCallback(
    async (sound: SoundEffect) => {
      if (!initialized.current) {
        await initializeAudio()
      }
      audioManager.current.playSound(sound)
    },
    [initializeAudio]
  )

  /**
   * Play a music track
   * Will attempt to initialize audio context if not already done
   */
  const playMusic = useCallback(
    async (track: MusicTrack) => {
      if (!initialized.current) {
        await initializeAudio()
      }
      audioManager.current.playMusic(track)
    },
    [initializeAudio]
  )

  /**
   * Stop the currently playing music
   */
  const stopMusic = useCallback(() => {
    audioManager.current.stopMusic()
  }, [])

  /**
   * Fade out the current music
   */
  const fadeOutMusic = useCallback((duration: number = 1.0) => {
    audioManager.current.fadeOutMusic(duration)
  }, [])

  /**
   * Suspend audio (call when app loses focus)
   */
  const suspendAudio = useCallback(() => {
    audioManager.current.suspend()
  }, [])

  /**
   * Resume audio (call when app regains focus)
   */
  const resumeAudio = useCallback(() => {
    audioManager.current.resume()
  }, [])

  /**
   * Preload sounds for faster playback
   */
  const preloadSounds = useCallback(
    async (sounds: SoundEffect[]) => {
      if (!initialized.current) {
        await initializeAudio()
      }
      await audioManager.current.preloadSounds(sounds)
    },
    [initializeAudio]
  )

  /**
   * Preload music tracks
   */
  const preloadMusic = useCallback(
    async (tracks: MusicTrack[]) => {
      if (!initialized.current) {
        await initializeAudio()
      }
      await audioManager.current.preloadMusic(tracks)
    },
    [initializeAudio]
  )

  return {
    playSound,
    playMusic,
    stopMusic,
    fadeOutMusic,
    suspendAudio,
    resumeAudio,
    preloadSounds,
    preloadMusic,
    initializeAudio,
    isInitialized: initialized.current,
  }
}

/**
 * Hook for UI sound effects (convenience wrapper)
 * Provides commonly used UI sounds
 */
export function useUISounds() {
  const { playSound } = useAudio()

  const playClick = useCallback(() => {
    playSound(SoundEffect.ButtonClick)
  }, [playSound])

  const playHover = useCallback(() => {
    playSound(SoundEffect.ButtonHover)
  }, [playSound])

  const playMenuOpen = useCallback(() => {
    playSound(SoundEffect.MenuOpen)
  }, [playSound])

  const playMenuClose = useCallback(() => {
    playSound(SoundEffect.MenuClose)
  }, [playSound])

  return {
    playClick,
    playHover,
    playMenuOpen,
    playMenuClose,
  }
}

/**
 * Hook for game sound effects (convenience wrapper)
 */
export function useGameSounds() {
  const { playSound } = useAudio()

  const playUnitSelect = useCallback(() => {
    playSound(SoundEffect.UnitSelect)
  }, [playSound])

  const playUnitMove = useCallback(() => {
    playSound(SoundEffect.UnitMove)
  }, [playSound])

  const playUnitAttack = useCallback(() => {
    playSound(SoundEffect.UnitAttack)
  }, [playSound])

  const playUnitDeath = useCallback(() => {
    playSound(SoundEffect.UnitDeath)
  }, [playSound])

  const playCityFounded = useCallback(() => {
    playSound(SoundEffect.CityFounded)
  }, [playSound])

  const playBuildingComplete = useCallback(() => {
    playSound(SoundEffect.BuildingComplete)
  }, [playSound])

  const playTechResearched = useCallback(() => {
    playSound(SoundEffect.TechResearched)
  }, [playSound])

  const playTurnStart = useCallback(() => {
    playSound(SoundEffect.TurnStart)
  }, [playSound])

  const playVictory = useCallback(() => {
    playSound(SoundEffect.Victory)
  }, [playSound])

  const playDefeat = useCallback(() => {
    playSound(SoundEffect.Defeat)
  }, [playSound])

  return {
    playUnitSelect,
    playUnitMove,
    playUnitAttack,
    playUnitDeath,
    playCityFounded,
    playBuildingComplete,
    playTechResearched,
    playTurnStart,
    playVictory,
    playDefeat,
  }
}

/**
 * Hook for notification sounds
 */
export function useNotificationSounds() {
  const { playSound } = useAudio()

  const playInfo = useCallback(() => {
    playSound(SoundEffect.NotificationInfo)
  }, [playSound])

  const playWarning = useCallback(() => {
    playSound(SoundEffect.NotificationWarning)
  }, [playSound])

  const playError = useCallback(() => {
    playSound(SoundEffect.NotificationError)
  }, [playSound])

  return {
    playInfo,
    playWarning,
    playError,
  }
}
