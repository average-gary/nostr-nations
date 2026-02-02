import React, { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useIsTauri } from '@/hooks/useTauri';

// Types for saved game data
interface SavedGame {
  id: string;
  name: string;
  saveDate: string;
  turn: number;
  civilization: string;
  mapSize: string;
}

interface LoadGameScreenProps {
  onBack: () => void;
  onGameLoaded: (gameId: string) => void;
}

// Mock data for development without Tauri
const MOCK_SAVES: SavedGame[] = [
  {
    id: 'save-1',
    name: 'Roman Empire Campaign',
    saveDate: '2026-01-28T14:30:00Z',
    turn: 42,
    civilization: 'Rome',
    mapSize: 'standard',
  },
  {
    id: 'save-2',
    name: 'Greek Conquest',
    saveDate: '2026-01-25T09:15:00Z',
    turn: 87,
    civilization: 'Greece',
    mapSize: 'large',
  },
  {
    id: 'save-3',
    name: 'Egyptian Dynasty',
    saveDate: '2026-01-20T18:45:00Z',
    turn: 156,
    civilization: 'Egypt',
    mapSize: 'huge',
  },
];

const LoadGameScreen: React.FC<LoadGameScreenProps> = ({ onBack, onGameLoaded }) => {
  const isTauri = useIsTauri();

  const [saves, setSaves] = useState<SavedGame[]>([]);
  const [selectedSave, setSelectedSave] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isDeleting, setIsDeleting] = useState(false);
  const [isLoadingGame, setIsLoadingGame] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [deleteConfirmId, setDeleteConfirmId] = useState<string | null>(null);

  // Fetch saved games on mount
  const fetchSaves = useCallback(async () => {
    setIsLoading(true);
    setError(null);

    try {
      if (isTauri) {
        const result = await invoke<SavedGame[]>('list_saved_games');
        setSaves(result);
      } else {
        // Mock for development without Tauri
        await new Promise((resolve) => setTimeout(resolve, 500));
        setSaves(MOCK_SAVES);
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
    } finally {
      setIsLoading(false);
    }
  }, [isTauri]);

  useEffect(() => {
    fetchSaves();
  }, [fetchSaves]);

  const handleLoadGame = async () => {
    if (!selectedSave) return;

    setIsLoadingGame(true);
    setError(null);

    try {
      if (isTauri) {
        const result = await invoke<{ game_id: string }>('load_game', { saveId: selectedSave });
        onGameLoaded(result.game_id);
      } else {
        // Mock for development without Tauri
        console.log('Loading game with saveId:', selectedSave);
        await new Promise((resolve) => setTimeout(resolve, 1000));
        onGameLoaded(selectedSave);
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      setIsLoadingGame(false);
    }
  };

  const handleDeleteSave = async (saveId: string) => {
    setIsDeleting(true);
    setError(null);

    try {
      if (isTauri) {
        await invoke('delete_saved_game', { saveId });
      } else {
        // Mock for development without Tauri
        await new Promise((resolve) => setTimeout(resolve, 300));
      }

      // Remove from local state
      setSaves((prev) => prev.filter((s) => s.id !== saveId));
      if (selectedSave === saveId) {
        setSelectedSave(null);
      }
      setDeleteConfirmId(null);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
    } finally {
      setIsDeleting(false);
    }
  };

  const formatDate = (dateString: string): string => {
    const date = new Date(dateString);
    return date.toLocaleDateString(undefined, {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const formatMapSize = (size: string): string => {
    return size.charAt(0).toUpperCase() + size.slice(1);
  };

  const renderSavesList = () => {
    if (isLoading) {
      return (
        <div className="flex items-center justify-center h-64">
          <div className="flex flex-col items-center gap-4">
            <span className="animate-spin inline-block w-8 h-8 border-4 border-secondary border-t-transparent rounded-full" />
            <span className="text-foreground-muted">Loading saved games...</span>
          </div>
        </div>
      );
    }

    if (saves.length === 0) {
      return (
        <div className="flex items-center justify-center h-64">
          <div className="text-center">
            <div className="text-6xl mb-4 opacity-50">
              <span className="text-foreground-dim">[ ]</span>
            </div>
            <h3 className="text-xl font-header text-foreground-muted mb-2">
              No Saved Games
            </h3>
            <p className="text-foreground-dim text-sm">
              Start a new game to create your first save.
            </p>
          </div>
        </div>
      );
    }

    return (
      <div className="space-y-3 max-h-[400px] overflow-y-auto pr-2">
        {saves.map((save) => (
          <div
            key={save.id}
            onClick={() => setSelectedSave(save.id)}
            className={`
              p-4 rounded-lg cursor-pointer transition-all duration-200
              border-2
              ${selectedSave === save.id
                ? 'bg-primary-600 border-secondary shadow-lg'
                : 'bg-background-light border-primary-700 hover:border-primary-500'
              }
            `}
          >
            <div className="flex items-start justify-between gap-4">
              <div className="flex-1 min-w-0">
                <h3 className="font-header text-lg text-foreground truncate">
                  {save.name}
                </h3>
                <div className="mt-2 grid grid-cols-2 gap-x-6 gap-y-1 text-sm">
                  <div className="flex items-center gap-2">
                    <span className="text-foreground-dim">Saved:</span>
                    <span className="text-foreground-muted">{formatDate(save.saveDate)}</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="text-foreground-dim">Turn:</span>
                    <span className="text-foreground-muted">{save.turn}</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="text-foreground-dim">Civilization:</span>
                    <span className="text-secondary">{save.civilization}</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="text-foreground-dim">Map:</span>
                    <span className="text-foreground-muted">{formatMapSize(save.mapSize)}</span>
                  </div>
                </div>
              </div>

              {/* Delete button */}
              <div className="flex-shrink-0">
                {deleteConfirmId === save.id ? (
                  <div className="flex items-center gap-2">
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleDeleteSave(save.id);
                      }}
                      disabled={isDeleting}
                      className="px-3 py-1 rounded text-sm font-header
                                 bg-danger text-white
                                 hover:bg-danger/80
                                 disabled:opacity-50 disabled:cursor-not-allowed
                                 transition-colors"
                    >
                      {isDeleting ? 'Deleting...' : 'Confirm'}
                    </button>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        setDeleteConfirmId(null);
                      }}
                      disabled={isDeleting}
                      className="px-3 py-1 rounded text-sm font-header
                                 bg-background-light border border-primary-700 text-foreground-muted
                                 hover:bg-background-lighter
                                 disabled:opacity-50 disabled:cursor-not-allowed
                                 transition-colors"
                    >
                      Cancel
                    </button>
                  </div>
                ) : (
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      setDeleteConfirmId(save.id);
                    }}
                    className="p-2 rounded text-foreground-dim
                               hover:bg-danger/20 hover:text-danger
                               transition-colors"
                    title="Delete save"
                  >
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      className="h-5 w-5"
                      fill="none"
                      viewBox="0 0 24 24"
                      stroke="currentColor"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                      />
                    </svg>
                  </button>
                )}
              </div>
            </div>
          </div>
        ))}
      </div>
    );
  };

  return (
    <div className="fixed inset-0 bg-black/80 flex items-center justify-center z-50 animate-fade-in">
      <div className="bg-background rounded-xl shadow-2xl w-full max-w-2xl max-h-[90vh] overflow-hidden border-2 border-primary-700">
        {/* Header */}
        <div className="px-6 py-4 border-b border-primary-700 bg-primary-900/50">
          <h1 className="text-2xl font-header text-secondary text-center">
            Load Game
          </h1>
        </div>

        {/* Content */}
        <div className="p-6">
          {error && (
            <div className="mb-4 bg-danger/20 border-2 border-danger rounded-lg px-4 py-3">
              <p className="text-danger text-sm">{error}</p>
            </div>
          )}

          {renderSavesList()}
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-primary-700 bg-primary-900/50 flex justify-between">
          <button
            onClick={onBack}
            disabled={isLoadingGame}
            className="px-6 py-2 rounded-lg font-header text-lg
                       bg-background-light border-2 border-primary-700 text-foreground
                       hover:bg-background-lighter hover:border-primary-500
                       disabled:opacity-50 disabled:cursor-not-allowed
                       transition-all duration-200"
          >
            Back
          </button>

          <button
            onClick={handleLoadGame}
            disabled={!selectedSave || isLoadingGame || isLoading}
            className="px-8 py-2 rounded-lg font-header text-lg
                       bg-secondary border-2 border-secondary text-background
                       hover:bg-secondary-600 hover:border-secondary-600
                       disabled:opacity-50 disabled:cursor-not-allowed
                       transition-all duration-200 flex items-center gap-2"
          >
            {isLoadingGame ? (
              <>
                <span className="animate-spin inline-block w-5 h-5 border-2 border-background border-t-transparent rounded-full" />
                Loading...
              </>
            ) : (
              'Load Game'
            )}
          </button>
        </div>
      </div>
    </div>
  );
};

export default LoadGameScreen;
