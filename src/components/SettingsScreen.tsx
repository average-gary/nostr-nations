import React, { useCallback } from 'react';
import {
  useSettingsStore,
  DEFAULT_SETTINGS,
  selectAudioSettings,
  selectDisplaySettings,
  selectGamePreferences,
} from '@/stores/settingsStore';

interface SettingsScreenProps {
  onBack: () => void;
}

const SettingsScreen: React.FC<SettingsScreenProps> = ({ onBack }) => {
  const audio = useSettingsStore(selectAudioSettings);
  const display = useSettingsStore(selectDisplaySettings);
  const game = useSettingsStore(selectGamePreferences);

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
  } = useSettingsStore();

  const handleResetToDefaults = useCallback(() => {
    resetToDefaults();
  }, [resetToDefaults]);

  return (
    <div className="h-full w-full flex flex-col items-center justify-center bg-gradient-game">
      {/* Title */}
      <h1 className="text-5xl font-header text-secondary text-glow mb-10 animate-fade-in">
        Settings
      </h1>

      {/* Settings Container */}
      <div className="w-full max-w-2xl px-8 animate-slide-up">
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
              onChange={(value) => setAutoSaveInterval(parseInt(value, 10) as 0 | 1 | 5 | 10)}
            />
            <TurnTimerSetting
              value={game.turnTimer}
              onChange={setTurnTimer}
            />
          </SettingsSection>
        </div>

        {/* Buttons */}
        <div className="flex justify-between mt-6">
          <button
            onClick={handleResetToDefaults}
            className="px-6 py-3 text-lg font-header rounded-lg transition-all duration-200 hover:scale-105 focus:outline-none focus:ring-2 focus:ring-danger bg-danger/20 hover:bg-danger/30 border-2 border-danger text-danger"
          >
            Reset to Defaults
          </button>
          <button
            onClick={onBack}
            className="px-6 py-3 text-lg font-header rounded-lg transition-all duration-200 hover:scale-105 focus:outline-none focus:ring-2 focus:ring-secondary bg-primary hover:bg-primary-600 border-2 border-secondary text-foreground"
          >
            Back to Menu
          </button>
        </div>
      </div>

      {/* Version info */}
      <div className="absolute bottom-4 left-4 text-foreground-dim text-sm">
        Version 0.1.0
      </div>
    </div>
  );
};

// Section component for grouping settings
interface SettingsSectionProps {
  title: string;
  children: React.ReactNode;
}

const SettingsSection: React.FC<SettingsSectionProps> = ({ title, children }) => {
  return (
    <div className="mb-6 last:mb-0">
      <div className="panel-header text-lg">{title}</div>
      <div className="panel-content space-y-4">{children}</div>
    </div>
  );
};

// Volume slider component
interface VolumeSliderProps {
  label: string;
  value: number;
  onChange: (value: number) => void;
  disabled?: boolean;
}

const VolumeSlider: React.FC<VolumeSliderProps> = ({
  label,
  value,
  onChange,
  disabled = false,
}) => {
  const percentage = Math.round(value * 100);

  return (
    <div className={`flex flex-col ${disabled ? 'opacity-50' : ''}`}>
      <div className="flex justify-between items-center mb-2">
        <label className="text-foreground text-sm font-medium">{label}</label>
        <span className="text-foreground-muted text-sm font-mono">{percentage}%</span>
      </div>
      <input
        type="range"
        min="0"
        max="100"
        value={percentage}
        onChange={(e) => onChange(parseInt(e.target.value, 10) / 100)}
        disabled={disabled}
        className="w-full h-2 bg-background rounded-lg appearance-none cursor-pointer disabled:cursor-not-allowed slider"
      />
    </div>
  );
};

// Toggle switch component
interface ToggleProps {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}

const Toggle: React.FC<ToggleProps> = ({ label, checked, onChange }) => {
  return (
    <div className="flex justify-between items-center">
      <label className="text-foreground text-sm font-medium">{label}</label>
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
  );
};

// Select dropdown component
interface SelectOptionProps {
  label: string;
  value: string;
  options: { value: string; label: string }[];
  onChange: (value: string) => void;
}

const SelectOption: React.FC<SelectOptionProps> = ({
  label,
  value,
  options,
  onChange,
}) => {
  return (
    <div className="flex justify-between items-center">
      <label className="text-foreground text-sm font-medium">{label}</label>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="px-3 py-2 bg-background border border-primary-700 rounded-lg text-foreground text-sm focus:outline-none focus:ring-2 focus:ring-secondary cursor-pointer"
      >
        {options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </div>
  );
};

// Turn timer setting component with enable/disable
interface TurnTimerSettingProps {
  value: number | null;
  onChange: (value: number | null) => void;
}

const TurnTimerSetting: React.FC<TurnTimerSettingProps> = ({ value, onChange }) => {
  const isEnabled = value !== null;
  const timerValue = value ?? 60;

  const handleToggle = () => {
    if (isEnabled) {
      onChange(null);
    } else {
      onChange(60); // Default to 60 seconds
    }
  };

  const handleValueChange = (newValue: string) => {
    const parsed = parseInt(newValue, 10);
    if (!isNaN(parsed) && parsed > 0) {
      onChange(parsed);
    }
  };

  return (
    <div className="space-y-3">
      <div className="flex justify-between items-center">
        <label className="text-foreground text-sm font-medium">Turn Timer (Multiplayer)</label>
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
        <div className="flex justify-between items-center ml-4">
          <label className="text-foreground-muted text-sm">Seconds per turn</label>
          <select
            value={timerValue.toString()}
            onChange={(e) => handleValueChange(e.target.value)}
            className="px-3 py-2 bg-background border border-primary-700 rounded-lg text-foreground text-sm focus:outline-none focus:ring-2 focus:ring-secondary cursor-pointer"
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
  );
};

export default SettingsScreen;
