import { SoundEffect, MusicTrack, SOUND_PATHS, MUSIC_PATHS } from './sounds'

/**
 * Core audio manager class using Web Audio API
 * Implements singleton pattern for global audio control
 */
export class AudioManager {
  private static instance: AudioManager
  private audioContext: AudioContext | null = null
  private masterGainNode: GainNode | null = null
  private musicGainNode: GainNode | null = null
  private sfxGainNode: GainNode | null = null

  private masterVolume: number = 1.0
  private musicVolume: number = 0.7
  private sfxVolume: number = 1.0
  private musicEnabled: boolean = true
  private sfxEnabled: boolean = true

  private currentMusic: AudioBufferSourceNode | null = null
  private currentMusicTrack: MusicTrack | null = null
  private audioBufferCache: Map<string, AudioBuffer> = new Map()
  private loadingPromises: Map<string, Promise<AudioBuffer | null>> = new Map()

  private constructor() {
    // Private constructor for singleton pattern
  }

  /**
   * Get the singleton instance of AudioManager
   */
  static getInstance(): AudioManager {
    if (!AudioManager.instance) {
      AudioManager.instance = new AudioManager()
    }
    return AudioManager.instance
  }

  /**
   * Initialize the audio context and gain nodes
   * Must be called after user interaction due to browser autoplay policies
   */
  async initialize(): Promise<void> {
    if (this.audioContext) {
      // Already initialized, just resume if suspended
      if (this.audioContext.state === 'suspended') {
        await this.audioContext.resume()
      }
      return
    }

    try {
      this.audioContext = new AudioContext()

      // Create gain nodes for volume control
      this.masterGainNode = this.audioContext.createGain()
      this.musicGainNode = this.audioContext.createGain()
      this.sfxGainNode = this.audioContext.createGain()

      // Connect gain nodes: music/sfx -> master -> destination
      this.musicGainNode.connect(this.masterGainNode)
      this.sfxGainNode.connect(this.masterGainNode)
      this.masterGainNode.connect(this.audioContext.destination)

      // Apply initial volumes
      this.updateGainNodes()

      // Handle browser autoplay policies
      if (this.audioContext.state === 'suspended') {
        await this.audioContext.resume()
      }
    } catch (error) {
      console.error('Failed to initialize AudioManager:', error)
      throw error
    }
  }

  /**
   * Update gain node values based on current volume settings
   */
  private updateGainNodes(): void {
    if (!this.masterGainNode || !this.musicGainNode || !this.sfxGainNode) {
      return
    }

    const currentTime = this.audioContext?.currentTime ?? 0

    this.masterGainNode.gain.setValueAtTime(this.masterVolume, currentTime)
    this.musicGainNode.gain.setValueAtTime(
      this.musicEnabled ? this.musicVolume : 0,
      currentTime
    )
    this.sfxGainNode.gain.setValueAtTime(
      this.sfxEnabled ? this.sfxVolume : 0,
      currentTime
    )
  }

  /**
   * Load an audio file and return its AudioBuffer
   */
  private async loadAudioBuffer(path: string): Promise<AudioBuffer | null> {
    if (!this.audioContext) {
      console.warn('AudioManager not initialized')
      return null
    }

    // Return cached buffer if available
    const cached = this.audioBufferCache.get(path)
    if (cached) {
      return cached
    }

    // Return existing loading promise if one exists
    const existingPromise = this.loadingPromises.get(path)
    if (existingPromise) {
      return existingPromise
    }

    // Start loading
    const loadPromise = (async () => {
      try {
        const response = await fetch(path)
        if (!response.ok) {
          console.warn(
            `Failed to load audio file: ${path} (${response.status})`
          )
          return null
        }

        const arrayBuffer = await response.arrayBuffer()
        const audioBuffer =
          await this.audioContext!.decodeAudioData(arrayBuffer)

        // Cache the buffer
        this.audioBufferCache.set(path, audioBuffer)
        this.loadingPromises.delete(path)

        return audioBuffer
      } catch (error) {
        console.warn(`Error loading audio file ${path}:`, error)
        this.loadingPromises.delete(path)
        return null
      }
    })()

    this.loadingPromises.set(path, loadPromise)
    return loadPromise
  }

  /**
   * Set master volume (0.0 - 1.0)
   */
  setMasterVolume(volume: number): void {
    this.masterVolume = Math.max(0, Math.min(1, volume))
    this.updateGainNodes()
  }

  /**
   * Set music volume (0.0 - 1.0)
   */
  setMusicVolume(volume: number): void {
    this.musicVolume = Math.max(0, Math.min(1, volume))
    this.updateGainNodes()
  }

  /**
   * Set sound effects volume (0.0 - 1.0)
   */
  setSfxVolume(volume: number): void {
    this.sfxVolume = Math.max(0, Math.min(1, volume))
    this.updateGainNodes()
  }

  /**
   * Enable or disable music
   */
  setMusicEnabled(enabled: boolean): void {
    this.musicEnabled = enabled
    this.updateGainNodes()

    if (!enabled && this.currentMusic) {
      this.stopMusic()
    }
  }

  /**
   * Enable or disable sound effects
   */
  setSfxEnabled(enabled: boolean): void {
    this.sfxEnabled = enabled
    this.updateGainNodes()
  }

  /**
   * Play a sound effect
   */
  async playSound(soundId: SoundEffect): Promise<void> {
    if (!this.sfxEnabled) {
      return
    }

    if (!this.audioContext || !this.sfxGainNode) {
      console.warn('AudioManager not initialized, attempting to initialize...')
      await this.initialize()
    }

    const path = SOUND_PATHS[soundId]
    if (!path) {
      console.warn(`Unknown sound effect: ${soundId}`)
      return
    }

    const buffer = await this.loadAudioBuffer(path)
    if (!buffer) {
      return
    }

    try {
      const source = this.audioContext!.createBufferSource()
      source.buffer = buffer
      source.connect(this.sfxGainNode!)
      source.start(0)
    } catch (error) {
      console.error(`Error playing sound ${soundId}:`, error)
    }
  }

  /**
   * Play a music track (stops any currently playing music)
   */
  async playMusic(trackId: MusicTrack): Promise<void> {
    if (!this.musicEnabled) {
      return
    }

    // Don't restart if same track is already playing
    if (this.currentMusicTrack === trackId && this.currentMusic) {
      return
    }

    if (!this.audioContext || !this.musicGainNode) {
      console.warn('AudioManager not initialized, attempting to initialize...')
      await this.initialize()
    }

    // Stop current music
    this.stopMusic()

    const path = MUSIC_PATHS[trackId]
    if (!path) {
      console.warn(`Unknown music track: ${trackId}`)
      return
    }

    const buffer = await this.loadAudioBuffer(path)
    if (!buffer) {
      return
    }

    try {
      const source = this.audioContext!.createBufferSource()
      source.buffer = buffer
      source.loop = true
      source.connect(this.musicGainNode!)
      source.start(0)

      this.currentMusic = source
      this.currentMusicTrack = trackId

      // Clean up when music ends (if not looping or stopped)
      source.onended = () => {
        if (this.currentMusic === source) {
          this.currentMusic = null
          this.currentMusicTrack = null
        }
      }
    } catch (error) {
      console.error(`Error playing music ${trackId}:`, error)
    }
  }

  /**
   * Stop the currently playing music
   */
  stopMusic(): void {
    if (this.currentMusic) {
      try {
        this.currentMusic.stop()
      } catch {
        // Ignore errors if already stopped
      }
      this.currentMusic = null
      this.currentMusicTrack = null
    }
  }

  /**
   * Fade out the current music over a duration (in seconds)
   */
  fadeOutMusic(duration: number): void {
    if (!this.currentMusic || !this.musicGainNode || !this.audioContext) {
      return
    }

    const currentTime = this.audioContext.currentTime
    const currentVolume = this.musicGainNode.gain.value

    // Fade to 0
    this.musicGainNode.gain.setValueAtTime(currentVolume, currentTime)
    this.musicGainNode.gain.linearRampToValueAtTime(0, currentTime + duration)

    // Stop and restore volume after fade
    const musicToStop = this.currentMusic
    setTimeout(() => {
      if (this.currentMusic === musicToStop) {
        this.stopMusic()
      }
      // Restore the gain node volume
      if (this.musicGainNode) {
        this.musicGainNode.gain.setValueAtTime(
          this.musicEnabled ? this.musicVolume : 0,
          this.audioContext?.currentTime ?? 0
        )
      }
    }, duration * 1000)
  }

  /**
   * Suspend the audio context (e.g., when app loses focus)
   */
  suspend(): void {
    if (this.audioContext && this.audioContext.state === 'running') {
      this.audioContext.suspend()
    }
  }

  /**
   * Resume the audio context (e.g., when app regains focus)
   */
  resume(): void {
    if (this.audioContext && this.audioContext.state === 'suspended') {
      this.audioContext.resume()
    }
  }

  /**
   * Get the current audio context state
   */
  getState(): AudioContextState | null {
    return this.audioContext?.state ?? null
  }

  /**
   * Check if audio is initialized
   */
  isInitialized(): boolean {
    return this.audioContext !== null
  }

  /**
   * Preload a set of sounds for faster playback
   */
  async preloadSounds(sounds: SoundEffect[]): Promise<void> {
    const promises = sounds.map((sound) => {
      const path = SOUND_PATHS[sound]
      return this.loadAudioBuffer(path)
    })
    await Promise.all(promises)
  }

  /**
   * Preload music tracks
   */
  async preloadMusic(tracks: MusicTrack[]): Promise<void> {
    const promises = tracks.map((track) => {
      const path = MUSIC_PATHS[track]
      return this.loadAudioBuffer(path)
    })
    await Promise.all(promises)
  }

  /**
   * Clear the audio buffer cache
   */
  clearCache(): void {
    this.audioBufferCache.clear()
  }
}

// Export singleton instance for convenience
export const audioManager = AudioManager.getInstance()
