import { useEffect, useState } from 'react';
import { useGameStore } from '@/stores/gameStore';
import MainMenu from '@/components/MainMenu';
import GameView from '@/components/GameView';
import LoadingScreen from '@/components/LoadingScreen';
import SettingsScreen from '@/components/SettingsScreen';
import NewGameWizard from '@/components/menu/NewGameWizard';

type Screen = 'loading' | 'menu' | 'newgame' | 'game' | 'lobby' | 'settings';

function App() {
  const [currentScreen, setCurrentScreen] = useState<Screen>('loading');
  const [activeGameId, setActiveGameId] = useState<string | null>(null);
  const { initializeGame } = useGameStore();

  useEffect(() => {
    // Simulate initial loading
    const timer = setTimeout(() => {
      setCurrentScreen('menu');
    }, 1500);

    return () => clearTimeout(timer);
  }, []);

  const handleOpenNewGameWizard = () => {
    setCurrentScreen('newgame');
  };

  const handleCancelNewGame = () => {
    setCurrentScreen('menu');
  };

  const handleGameCreated = (gameId: string) => {
    setActiveGameId(gameId);
    // Initialize the frontend game store as well
    initializeGame();
    setCurrentScreen('game');
  };

  const handleBackToMenu = () => {
    setActiveGameId(null);
    setCurrentScreen('menu');
  };

  const handleOpenSettings = () => {
    setCurrentScreen('settings');
  };

  return (
    <div className="h-screen w-screen bg-gradient-game overflow-hidden">
      {currentScreen === 'loading' && <LoadingScreen />}

      {currentScreen === 'menu' && (
        <MainMenu
          onNewGame={handleOpenNewGameWizard}
          onJoinGame={() => console.log('Join game clicked')}
          onLoadGame={() => console.log('Load game clicked')}
          onSettings={handleOpenSettings}
          onExit={() => console.log('Exit clicked')}
        />
      )}

      {currentScreen === 'newgame' && (
        <NewGameWizard
          onCancel={handleCancelNewGame}
          onGameCreated={handleGameCreated}
        />
      )}

      {currentScreen === 'settings' && (
        <SettingsScreen onBack={handleBackToMenu} />
      )}

      {currentScreen === 'game' && (
        <GameView onBackToMenu={handleBackToMenu} />
      )}
    </div>
  );
}

export default App;
