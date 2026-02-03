import React, { useCallback } from 'react'
import {
  useSettingsStore,
  selectAudioSettings,
  selectDisplaySettings,
  selectGamePreferences,
} from '@/stores/settingsStore'

interface SettingsScreenProps {
  onBack: () => void
}

const SettingsScreen: React.FC<SettingsScreenProps> = ({ onBack }) => {
  const audio = useSettingsStore(selectAudioSettings)
  const display = useSettingsStore(selectDisplaySettings)
  const game = useSettingsStore(selectGamePreferences)

  const {
    setMasterVolume,
    setMusicVolume,
    setSfxVolume,
    setMusicEnabled,
    setSfxEnabled,
    setShowGrid,
    setShowYieldIcons,
    setFullscreen,
    setAutoSaveInterval,
    setTurnTimer,
    resetToDefaults,
  } = useSettingsStore()

  const handleResetToDefaults = useCallback(() => {
    resetToDefaults()
  }, [resetToDefaults])

  return (
    <div className="bg-gradient-game flex h-full w-full flex-col items-center justify-center">
      {/* Title */}
      <h1 className="text-glow mb-10 animate-fade-in font-header text-5xl text-secondary">
        Settings
      </h1>

      {/* Settings Container */}
      <div className="w-full max-w-2xl animate-slide-up px-8">
        <div className="panel max-h-[60vh] overflow-y-auto">
          {/* Audio Settings */}
          <SettingsSection title="Audio">
            <VolumeSlider
              label="Master Volume"
              value={audio.masterVolume}
              onChange={setMasterVolume}
            />
            <VolumeSlider
              label="Music Volume"
              value={audio.musicVolume}
              onChange={setMusicVolume}
              disabled={!audio.musicEnabled}
            />
            <VolumeSlider
              label="Sound Effects Volume"
              value={audio.sfxVolume}
              onChange={setSfxVolume}
              disabled={!audio.sfxEnabled}
            />
            <Toggle
              label="Music Enabled"
              checked={audio.musicEnabled}
              onChange={setMusicEnabled}
            />
            <Toggle
              label="Sound Effects Enabled"
              checked={audio.sfxEnabled}
              onChange={setSfxEnabled}
            />
          </SettingsSection>

          {/* Display Settings */}
          <SettingsSection title="Display">
            <Toggle
              label="Show Grid Overlay"
              checked={display.showGrid}
              onChange={setShowGrid}
            />
            <Toggle
              label="Show Yield Icons"
              checked={display.showYieldIcons}
              onChange={setShowYieldIcons}
            />
            <Toggle
              label="Fullscreen"
              checked={display.fullscreen}
              onChange={setFullscreen}
            />
          </SettingsSection>

          {/* Game Settings */}
          <SettingsSection title="Game">
            <SelectOption
              label="Auto-save Interval"
              value={game.autoSaveInterval.toString()}
              options={[
                { value: '0', label: 'Disabled' },
                { value: '1', label: 'Every Turn' },
                { value: '5', label: 'Every 5 Turns' },
                { value: '10', label: 'Every 10 Turns' },
              ]}
              onChange={(value) =>
                setAutoSaveInterval(parseInt(value, 10) as 0 | 1 | 5 | 10)
              }
            />
            <TurnTimerSetting value={game.turnTimer} onChange={setTurnTimer} />
          </SettingsSection>
        </div>

        {/* Buttons */}
        <div className="mt-6 flex justify-between">
          <button
            onClick={handleResetToDefaults}
            className="rounded-lg border-2 border-danger bg-danger/20 px-6 py-3 font-header text-lg text-danger transition-all duration-200 hover:scale-105 hover:bg-danger/30 focus:outline-none focus:ring-2 focus:ring-danger"
          >
            Reset to Defaults
          </button>
          <button
            onClick={onBack}
            className="rounded-lg border-2 border-secondary bg-primary px-6 py-3 font-header text-lg text-foreground transition-all duration-200 hover:scale-105 hover:bg-primary-600 focus:outline-none focus:ring-2 focus:ring-secondary"
          >
            Back to Menu
          </button>
        </div>
      </div>

      {/* Version info */}
      <div className="absolute bottom-4 left-4 text-sm text-foreground-dim">
        Version 0.1.0
      </div>
    </div>
  )
}

// Section component for grouping settings
interface SettingsSectionProps {
  title: string
  children: React.ReactNode
}

const SettingsSection: React.FC<SettingsSectionProps> = ({
  title,
  children,
}) => {
  return (
    <div className="mb-6 last:mb-0">
      <div className="panel-header text-lg">{title}</div>
      <div className="panel-content space-y-4">{children}</div>
    </div>
  )
}

// Volume slider component
interface VolumeSliderProps {
  label: string
  value: number
  onChange: (value: number) => void
  disabled?: boolean
}

const VolumeSlider: React.FC<VolumeSliderProps> = ({
  label,
  value,
  onChange,
  disabled = false,
}) => {
  const percentage = Math.round(value * 100)

  return (
    <div className={`flex flex-col ${disabled ? 'opacity-50' : ''}`}>
      <div className="mb-2 flex items-center justify-between">
        <label className="text-sm font-medium text-foreground">{label}</label>
        <span className="font-mono text-sm text-foreground-muted">
          {percentage}%
        </span>
      </div>
      <input
        type="range"
        min="0"
        max="100"
        value={percentage}
        onChange={(e) => onChange(parseInt(e.target.value, 10) / 100)}
        disabled={disabled}
        className="slider h-2 w-full cursor-pointer appearance-none rounded-lg bg-background disabled:cursor-not-allowed"
      />
    </div>
  )
}

// Toggle switch component
interface ToggleProps {
  label: string
  checked: boolean
  onChange: (checked: boolean) => void
}

const Toggle: React.FC<ToggleProps> = ({ label, checked, onChange }) => {
  return (
    <div className="flex items-center justify-between">
      <label className="text-sm font-medium text-foreground">{label}</label>
      <button
        type="button"
        role="switch"
        aria-checked={checked}
        onClick={() => onChange(!checked)}
        className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-secondary focus:ring-offset-2 focus:ring-offset-background ${
          checked ? 'bg-secondary' : 'bg-background-lighter'
        }`}
      >
        <span
          className={`inline-block h-4 w-4 transform rounded-full bg-foreground transition-transform duration-200 ${
            checked ? 'translate-x-6' : 'translate-x-1'
          }`}
        />
      </button>
    </div>
  )
}

// Select dropdown component
interface SelectOptionProps {
  label: string
  value: string
  options: { value: string; label: string }[]
  onChange: (value: string) => void
}

const SelectOption: React.FC<SelectOptionProps> = ({
  label,
  value,
  options,
  onChange,
}) => {
  return (
    <div className="flex items-center justify-between">
      <label className="text-sm font-medium text-foreground">{label}</label>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="cursor-pointer rounded-lg border border-primary-700 bg-background px-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-secondary"
      >
        {options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </div>
  )
}

// Turn timer setting component with enable/disable
interface TurnTimerSettingProps {
  value: number | null
  onChange: (value: number | null) => void
}

const TurnTimerSetting: React.FC<TurnTimerSettingProps> = ({
  value,
  onChange,
}) => {
  const isEnabled = value !== null
  const timerValue = value ?? 60

  const handleToggle = () => {
    if (isEnabled) {
      onChange(null)
    } else {
      onChange(60) // Default to 60 seconds
    }
  }

  const handleValueChange = (newValue: string) => {
    const parsed = parseInt(newValue, 10)
    if (!isNaN(parsed) && parsed > 0) {
      onChange(parsed)
    }
  }

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <label className="text-sm font-medium text-foreground">
          Turn Timer (Multiplayer)
        </label>
        <button
          type="button"
          role="switch"
          aria-checked={isEnabled}
          onClick={handleToggle}
          className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-secondary focus:ring-offset-2 focus:ring-offset-background ${
            isEnabled ? 'bg-secondary' : 'bg-background-lighter'
          }`}
        >
          <span
            className={`inline-block h-4 w-4 transform rounded-full bg-foreground transition-transform duration-200 ${
              isEnabled ? 'translate-x-6' : 'translate-x-1'
            }`}
          />
        </button>
      </div>
      {isEnabled && (
        <div className="ml-4 flex items-center justify-between">
          <label className="text-sm text-foreground-muted">
            Seconds per turn
          </label>
          <select
            value={timerValue.toString()}
            onChange={(e) => handleValueChange(e.target.value)}
            className="cursor-pointer rounded-lg border border-primary-700 bg-background px-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-secondary"
          >
            <option value="30">30 seconds</option>
            <option value="60">60 seconds</option>
            <option value="90">90 seconds</option>
            <option value="120">2 minutes</option>
            <option value="180">3 minutes</option>
            <option value="300">5 minutes</option>
          </select>
        </div>
      )}
    </div>
  )
}

export default SettingsScreen
