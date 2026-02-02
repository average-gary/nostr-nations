import React, { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useIsTauri } from '@/hooks/useTauri';

// Types for game configuration matching Rust backend
export type MapSize = 'duel' | 'small' | 'standard' | 'large' | 'huge';
export type Difficulty = 'settler' | 'chieftain' | 'prince' | 'king' | 'emperor' | 'deity';
export type GameSpeed = 'quick' | 'normal' | 'epic' | 'marathon';
export type CivilizationId = 'rome' | 'greece' | 'egypt' | 'china' | 'india' | 'persia' | 'aztec' | 'inca';

interface CreateGameOptions {
  name: string;
  player_name: string;
  civilization: string;
  map_size: string;
  difficulty: string;
  game_speed: string;
  seed: string | null;
}

interface GameStateResponse {
  game_id: string;
  phase: string;
  turn: number;
  current_player: number;
  player_count: number;
  map_width: number;
  map_height: number;
}

interface NewGameWizardProps {
  onCancel: () => void;
  onGameCreated: (gameId: string) => void;
}

interface CivilizationInfo {
  id: CivilizationId;
  name: string;
  leader: string;
  description: string;
  ability: string;
}

// Civilization data
const CIVILIZATIONS: CivilizationInfo[] = [
  {
    id: 'rome',
    name: 'Rome',
    leader: 'Augustus Caesar',
    description: 'The eternal city rises to dominate the known world.',
    ability: 'All roads lead to Rome - Free roads in city radius',
  },
  {
    id: 'greece',
    name: 'Greece',
    leader: 'Alexander',
    description: 'Birthplace of democracy and philosophy.',
    ability: 'Hellenic League - City-state influence bonus',
  },
  {
    id: 'egypt',
    name: 'Egypt',
    leader: 'Cleopatra',
    description: 'Ancient land of pyramids and pharaohs.',
    ability: 'Gift of the Nile - Bonus food from flood plains',
  },
  {
    id: 'china',
    name: 'China',
    leader: 'Qin Shi Huang',
    description: 'The Middle Kingdom with ancient wisdom.',
    ability: 'Dynastic Cycle - Bonus to wonder production',
  },
  {
    id: 'india',
    name: 'India',
    leader: 'Gandhi',
    description: 'A land of spiritual enlightenment and diversity.',
    ability: 'Satyagraha - Faith and happiness bonus',
  },
  {
    id: 'persia',
    name: 'Persia',
    leader: 'Cyrus',
    description: 'Empire of roads and royal gardens.',
    ability: 'Satrapies - Golden age bonuses',
  },
  {
    id: 'aztec',
    name: 'Aztec',
    leader: 'Montezuma',
    description: 'Warriors of the sun god.',
    ability: 'Sacrificial Captives - Culture from kills',
  },
  {
    id: 'inca',
    name: 'Inca',
    leader: 'Pachacuti',
    description: 'Masters of the mountain terrain.',
    ability: 'Great Andean Road - No movement penalty on hills',
  },
];

// Configuration option data
const MAP_SIZES: { value: MapSize; label: string; description: string }[] = [
  { value: 'duel', label: 'Duel', description: '40x24 - Perfect for 1v1 matches' },
  { value: 'small', label: 'Small', description: '56x36 - Quick games, 2-4 players' },
  { value: 'standard', label: 'Standard', description: '80x52 - Balanced experience' },
  { value: 'large', label: 'Large', description: '104x64 - Epic scale' },
  { value: 'huge', label: 'Huge', description: '128x80 - Massive world' },
];

const DIFFICULTIES: { value: Difficulty; label: string; description: string }[] = [
  { value: 'settler', label: 'Settler', description: 'For new players learning the game' },
  { value: 'chieftain', label: 'Chieftain', description: 'A relaxed challenge' },
  { value: 'prince', label: 'Prince', description: 'Fair and balanced - no bonuses' },
  { value: 'king', label: 'King', description: 'AI receives minor advantages' },
  { value: 'emperor', label: 'Emperor', description: 'A serious challenge' },
  { value: 'deity', label: 'Deity', description: 'Near impossible - for experts only' },
];

const GAME_SPEEDS: { value: GameSpeed; label: string; description: string }[] = [
  { value: 'quick', label: 'Quick', description: '33% faster - shorter games' },
  { value: 'normal', label: 'Normal', description: 'Standard pacing' },
  { value: 'epic', label: 'Epic', description: '50% longer - more strategic depth' },
  { value: 'marathon', label: 'Marathon', description: '3x longer - ultimate experience' },
];

type WizardStep = 'basic' | 'civilization' | 'settings' | 'review';

const STEPS: WizardStep[] = ['basic', 'civilization', 'settings', 'review'];

const NewGameWizard: React.FC<NewGameWizardProps> = ({ onCancel, onGameCreated }) => {
  const isTauri = useIsTauri();

  // Wizard state
  const [currentStep, setCurrentStep] = useState<WizardStep>('basic');
  const [isCreating, setIsCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Form state
  const [gameName, setGameName] = useState('New Game');
  const [playerName, setPlayerName] = useState('Player');
  const [selectedCiv, setSelectedCiv] = useState<CivilizationId>('rome');
  const [mapSize, setMapSize] = useState<MapSize>('standard');
  const [difficulty, setDifficulty] = useState<Difficulty>('prince');
  const [gameSpeed, setGameSpeed] = useState<GameSpeed>('normal');
  const [randomSeed, setRandomSeed] = useState('');
  const [useRandomSeed, setUseRandomSeed] = useState(false);

  // Validation
  const isBasicValid = gameName.trim().length > 0 && playerName.trim().length > 0;
  const isCivSelected = selectedCiv !== null;

  const canProceed = useCallback(() => {
    switch (currentStep) {
      case 'basic':
        return isBasicValid;
      case 'civilization':
        return isCivSelected;
      case 'settings':
        return true;
      case 'review':
        return true;
      default:
        return false;
    }
  }, [currentStep, isBasicValid, isCivSelected]);

  const goToStep = (step: WizardStep) => {
    setError(null);
    setCurrentStep(step);
  };

  const goNext = () => {
    const currentIndex = STEPS.indexOf(currentStep);
    if (currentIndex < STEPS.length - 1) {
      goToStep(STEPS[currentIndex + 1]);
    }
  };

  const goBack = () => {
    const currentIndex = STEPS.indexOf(currentStep);
    if (currentIndex > 0) {
      goToStep(STEPS[currentIndex - 1]);
    } else {
      onCancel();
    }
  };

  const handleCreateGame = async () => {
    setIsCreating(true);
    setError(null);

    try {
      const options: CreateGameOptions = {
        name: gameName.trim(),
        player_name: playerName.trim(),
        civilization: selectedCiv,
        map_size: mapSize,
        difficulty: difficulty,
        game_speed: gameSpeed,
        seed: useRandomSeed && randomSeed.trim() ? randomSeed.trim() : null,
      };

      if (isTauri) {
        const result = await invoke<GameStateResponse>('create_game', { options });
        onGameCreated(result.game_id);
      } else {
        // Mock for development without Tauri
        console.log('Creating game with options:', options);
        await new Promise((resolve) => setTimeout(resolve, 1000));
        onGameCreated('mock-game-id');
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      setIsCreating(false);
    }
  };

  const renderStepIndicator = () => (
    <div className="flex items-center justify-center gap-2 mb-8">
      {STEPS.map((step, index) => {
        const isActive = step === currentStep;
        const isPast = STEPS.indexOf(currentStep) > index;

        return (
          <React.Fragment key={step}>
            <button
              onClick={() => isPast && goToStep(step)}
              disabled={!isPast}
              className={`
                w-10 h-10 rounded-full flex items-center justify-center
                font-header text-lg transition-all duration-200
                ${isActive
                  ? 'bg-secondary text-background border-2 border-secondary'
                  : isPast
                    ? 'bg-primary-600 text-foreground border-2 border-secondary cursor-pointer hover:bg-primary-500'
                    : 'bg-background-light text-foreground-dim border-2 border-primary-700'
                }
              `}
            >
              {index + 1}
            </button>
            {index < STEPS.length - 1 && (
              <div
                className={`w-12 h-1 rounded ${
                  isPast ? 'bg-secondary' : 'bg-primary-700'
                }`}
              />
            )}
          </React.Fragment>
        );
      })}
    </div>
  );

  const renderBasicStep = () => (
    <div className="space-y-6">
      <h2 className="text-3xl font-header text-secondary text-center mb-8">
        Game Setup
      </h2>

      <div className="space-y-4">
        <div>
          <label className="block text-foreground-muted text-sm mb-2">
            Game Name
          </label>
          <input
            type="text"
            value={gameName}
            onChange={(e) => setGameName(e.target.value)}
            placeholder="Enter game name..."
            className="w-full px-4 py-3 bg-background-light border-2 border-primary-700
                       rounded-lg text-foreground placeholder-foreground-dim
                       focus:outline-none focus:border-secondary transition-colors"
            maxLength={50}
          />
        </div>

        <div>
          <label className="block text-foreground-muted text-sm mb-2">
            Your Name
          </label>
          <input
            type="text"
            value={playerName}
            onChange={(e) => setPlayerName(e.target.value)}
            placeholder="Enter your name..."
            className="w-full px-4 py-3 bg-background-light border-2 border-primary-700
                       rounded-lg text-foreground placeholder-foreground-dim
                       focus:outline-none focus:border-secondary transition-colors"
            maxLength={30}
          />
        </div>
      </div>
    </div>
  );

  const renderCivilizationStep = () => (
    <div className="space-y-6">
      <h2 className="text-3xl font-header text-secondary text-center mb-4">
        Choose Your Civilization
      </h2>

      <div className="grid grid-cols-2 gap-4 max-h-[400px] overflow-y-auto pr-2">
        {CIVILIZATIONS.map((civ) => (
          <button
            key={civ.id}
            onClick={() => setSelectedCiv(civ.id)}
            className={`
              p-4 rounded-lg text-left transition-all duration-200
              border-2
              ${selectedCiv === civ.id
                ? 'bg-primary-600 border-secondary shadow-lg scale-[1.02]'
                : 'bg-background-light border-primary-700 hover:border-primary-500'
              }
            `}
          >
            <div className="flex items-start gap-3">
              <div className={`
                w-12 h-12 rounded-full flex items-center justify-center
                text-2xl font-header
                ${selectedCiv === civ.id ? 'bg-secondary text-background' : 'bg-primary-700 text-secondary'}
              `}>
                {civ.name.charAt(0)}
              </div>
              <div className="flex-1 min-w-0">
                <h3 className="font-header text-lg text-foreground truncate">
                  {civ.name}
                </h3>
                <p className="text-foreground-muted text-sm truncate">
                  {civ.leader}
                </p>
              </div>
            </div>
            <p className="mt-2 text-foreground-dim text-xs line-clamp-2">
              {civ.description}
            </p>
            <div className="mt-2 px-2 py-1 bg-background/50 rounded text-xs text-secondary">
              {civ.ability}
            </div>
          </button>
        ))}
      </div>
    </div>
  );

  const renderSettingsStep = () => (
    <div className="space-y-6">
      <h2 className="text-3xl font-header text-secondary text-center mb-4">
        Game Settings
      </h2>

      {/* Map Size */}
      <div>
        <label className="block text-foreground-muted text-sm mb-3">
          Map Size
        </label>
        <div className="grid grid-cols-5 gap-2">
          {MAP_SIZES.map((size) => (
            <button
              key={size.value}
              onClick={() => setMapSize(size.value)}
              className={`
                px-3 py-2 rounded-lg text-center transition-all duration-200
                border-2
                ${mapSize === size.value
                  ? 'bg-primary-600 border-secondary'
                  : 'bg-background-light border-primary-700 hover:border-primary-500'
                }
              `}
            >
              <span className="block font-header text-sm">{size.label}</span>
            </button>
          ))}
        </div>
        <p className="mt-2 text-foreground-dim text-sm text-center">
          {MAP_SIZES.find((s) => s.value === mapSize)?.description}
        </p>
      </div>

      {/* Difficulty */}
      <div>
        <label className="block text-foreground-muted text-sm mb-3">
          Difficulty
        </label>
        <div className="grid grid-cols-6 gap-2">
          {DIFFICULTIES.map((diff) => (
            <button
              key={diff.value}
              onClick={() => setDifficulty(diff.value)}
              className={`
                px-2 py-2 rounded-lg text-center transition-all duration-200
                border-2
                ${difficulty === diff.value
                  ? 'bg-primary-600 border-secondary'
                  : 'bg-background-light border-primary-700 hover:border-primary-500'
                }
              `}
            >
              <span className="block font-header text-xs">{diff.label}</span>
            </button>
          ))}
        </div>
        <p className="mt-2 text-foreground-dim text-sm text-center">
          {DIFFICULTIES.find((d) => d.value === difficulty)?.description}
        </p>
      </div>

      {/* Game Speed */}
      <div>
        <label className="block text-foreground-muted text-sm mb-3">
          Game Speed
        </label>
        <div className="grid grid-cols-4 gap-2">
          {GAME_SPEEDS.map((speed) => (
            <button
              key={speed.value}
              onClick={() => setGameSpeed(speed.value)}
              className={`
                px-3 py-2 rounded-lg text-center transition-all duration-200
                border-2
                ${gameSpeed === speed.value
                  ? 'bg-primary-600 border-secondary'
                  : 'bg-background-light border-primary-700 hover:border-primary-500'
                }
              `}
            >
              <span className="block font-header text-sm">{speed.label}</span>
            </button>
          ))}
        </div>
        <p className="mt-2 text-foreground-dim text-sm text-center">
          {GAME_SPEEDS.find((s) => s.value === gameSpeed)?.description}
        </p>
      </div>

      {/* Random Seed (Advanced) */}
      <div className="pt-4 border-t border-primary-700">
        <div className="flex items-center gap-3 mb-3">
          <input
            type="checkbox"
            id="use-seed"
            checked={useRandomSeed}
            onChange={(e) => setUseRandomSeed(e.target.checked)}
            className="w-5 h-5 rounded bg-background-light border-primary-700
                       text-secondary focus:ring-secondary focus:ring-offset-background"
          />
          <label htmlFor="use-seed" className="text-foreground-muted text-sm cursor-pointer">
            Use custom seed (for reproducible maps)
          </label>
        </div>
        {useRandomSeed && (
          <input
            type="text"
            value={randomSeed}
            onChange={(e) => setRandomSeed(e.target.value)}
            placeholder="Enter seed value..."
            className="w-full px-4 py-2 bg-background-light border-2 border-primary-700
                       rounded-lg text-foreground placeholder-foreground-dim
                       focus:outline-none focus:border-secondary transition-colors text-sm"
            maxLength={32}
          />
        )}
      </div>
    </div>
  );

  const renderReviewStep = () => {
    const selectedCivInfo = CIVILIZATIONS.find((c) => c.id === selectedCiv);

    return (
      <div className="space-y-6">
        <h2 className="text-3xl font-header text-secondary text-center mb-4">
          Review Your Game
        </h2>

        <div className="bg-background-light rounded-lg p-6 border-2 border-primary-700">
          <div className="grid grid-cols-2 gap-6">
            {/* Left Column */}
            <div className="space-y-4">
              <div>
                <span className="text-foreground-dim text-sm">Game Name</span>
                <p className="text-foreground font-header text-lg">{gameName}</p>
              </div>

              <div>
                <span className="text-foreground-dim text-sm">Player Name</span>
                <p className="text-foreground font-header text-lg">{playerName}</p>
              </div>

              <div>
                <span className="text-foreground-dim text-sm">Civilization</span>
                <div className="flex items-center gap-3 mt-1">
                  <div className="w-10 h-10 rounded-full bg-secondary flex items-center justify-center text-background font-header text-xl">
                    {selectedCivInfo?.name.charAt(0)}
                  </div>
                  <div>
                    <p className="text-foreground font-header">{selectedCivInfo?.name}</p>
                    <p className="text-foreground-muted text-sm">{selectedCivInfo?.leader}</p>
                  </div>
                </div>
              </div>
            </div>

            {/* Right Column */}
            <div className="space-y-4">
              <div>
                <span className="text-foreground-dim text-sm">Map Size</span>
                <p className="text-foreground font-header text-lg capitalize">{mapSize}</p>
              </div>

              <div>
                <span className="text-foreground-dim text-sm">Difficulty</span>
                <p className="text-foreground font-header text-lg capitalize">{difficulty}</p>
              </div>

              <div>
                <span className="text-foreground-dim text-sm">Game Speed</span>
                <p className="text-foreground font-header text-lg capitalize">{gameSpeed}</p>
              </div>

              {useRandomSeed && randomSeed && (
                <div>
                  <span className="text-foreground-dim text-sm">Seed</span>
                  <p className="text-foreground font-mono text-sm">{randomSeed}</p>
                </div>
              )}
            </div>
          </div>

          {/* Civilization Ability */}
          <div className="mt-6 pt-4 border-t border-primary-700">
            <span className="text-foreground-dim text-sm">Unique Ability</span>
            <p className="text-secondary font-header mt-1">{selectedCivInfo?.ability}</p>
          </div>
        </div>

        {error && (
          <div className="bg-danger/20 border-2 border-danger rounded-lg px-4 py-3">
            <p className="text-danger text-sm">{error}</p>
          </div>
        )}
      </div>
    );
  };

  const renderCurrentStep = () => {
    switch (currentStep) {
      case 'basic':
        return renderBasicStep();
      case 'civilization':
        return renderCivilizationStep();
      case 'settings':
        return renderSettingsStep();
      case 'review':
        return renderReviewStep();
      default:
        return null;
    }
  };

  return (
    <div className="fixed inset-0 bg-black/80 flex items-center justify-center z-50 animate-fade-in">
      <div className="bg-background rounded-xl shadow-2xl w-full max-w-3xl max-h-[90vh] overflow-hidden border-2 border-primary-700">
        {/* Header */}
        <div className="px-6 py-4 border-b border-primary-700 bg-primary-900/50">
          <h1 className="text-2xl font-header text-secondary text-center">
            New Game
          </h1>
        </div>

        {/* Content */}
        <div className="p-6 overflow-y-auto max-h-[calc(90vh-140px)]">
          {renderStepIndicator()}
          {renderCurrentStep()}
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-primary-700 bg-primary-900/50 flex justify-between">
          <button
            onClick={goBack}
            disabled={isCreating}
            className="px-6 py-2 rounded-lg font-header text-lg
                       bg-background-light border-2 border-primary-700 text-foreground
                       hover:bg-background-lighter hover:border-primary-500
                       disabled:opacity-50 disabled:cursor-not-allowed
                       transition-all duration-200"
          >
            {currentStep === 'basic' ? 'Cancel' : 'Back'}
          </button>

          {currentStep === 'review' ? (
            <button
              onClick={handleCreateGame}
              disabled={isCreating || !canProceed()}
              className="px-8 py-2 rounded-lg font-header text-lg
                         bg-secondary border-2 border-secondary text-background
                         hover:bg-secondary-600 hover:border-secondary-600
                         disabled:opacity-50 disabled:cursor-not-allowed
                         transition-all duration-200 flex items-center gap-2"
            >
              {isCreating ? (
                <>
                  <span className="animate-spin inline-block w-5 h-5 border-2 border-background border-t-transparent rounded-full" />
                  Creating...
                </>
              ) : (
                'Start Game'
              )}
            </button>
          ) : (
            <button
              onClick={goNext}
              disabled={!canProceed()}
              className="px-8 py-2 rounded-lg font-header text-lg
                         bg-primary border-2 border-secondary text-foreground
                         hover:bg-primary-600
                         disabled:opacity-50 disabled:cursor-not-allowed
                         transition-all duration-200"
            >
              Next
            </button>
          )}
        </div>
      </div>
    </div>
  );
};

export default NewGameWizard;
