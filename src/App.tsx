import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useGameStore } from '@/stores/gameStore'
import MainMenu from '@/components/MainMenu'
import GameView from '@/components/GameView'
import LoadingScreen from '@/components/LoadingScreen'
import SettingsScreen from '@/components/SettingsScreen'
import NewGameWizard from '@/components/menu/NewGameWizard'
import JoinGameScreen from '@/components/menu/JoinGameScreen'
import LoadGameScreen from '@/components/menu/LoadGameScreen'

type Screen =
  | 'loading'
  | 'menu'
  | 'newgame'
  | 'joingame'
  | 'loadgame'
  | 'game'
  | 'lobby'
  | 'settings'

function App() {
  const [currentScreen, setCurrentScreen] = useState<Screen>('loading')
  const [_activeGameId, setActiveGameId] = useState<string | null>(null)
  const { initializeGame } = useGameStore()

  useEffect(() => {
    // Simulate initial loading
    const timer = setTimeout(() => {
      setCurrentScreen('menu')
    }, 1500)

    return () => clearTimeout(timer)
  }, [])

  const handleOpenNewGameWizard = () => {
    setCurrentScreen('newgame')
  }

  const handleCancelNewGame = () => {
    setCurrentScreen('menu')
  }

  const handleGameCreated = (gameId: string) => {
    setActiveGameId(gameId)
    // Initialize the frontend game store as well
    initializeGame()
    setCurrentScreen('game')
  }

  const handleBackToMenu = async () => {
    // End the game on the backend to allow starting a new one
    try {
      await invoke('end_game')
    } catch (e) {
      // Ignore error if no game was active
      console.debug('No active game to end:', e)
    }
    setActiveGameId(null)
    setCurrentScreen('menu')
  }

  const handleJoinGame = () => {
    setCurrentScreen('joingame')
  }

  const handleLoadGame = () => {
    setCurrentScreen('loadgame')
  }

  const handleGameJoined = (gameId: string) => {
    setActiveGameId(gameId)
    initializeGame()
    setCurrentScreen('game')
  }

  const handleGameLoaded = (gameId: string) => {
    setActiveGameId(gameId)
    initializeGame()
    setCurrentScreen('game')
  }

  const handleOpenSettings = () => {
    setCurrentScreen('settings')
  }

  return (
    <div className="bg-gradient-game h-screen w-screen overflow-hidden">
      {currentScreen === 'loading' && <LoadingScreen />}

      {currentScreen === 'menu' && (
        <MainMenu
          onNewGame={handleOpenNewGameWizard}
          onJoinGame={handleJoinGame}
          onLoadGame={handleLoadGame}
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

      {currentScreen === 'joingame' && (
        <JoinGameScreen
          onBack={handleBackToMenu}
          onConnected={handleGameJoined}
        />
      )}

      {currentScreen === 'loadgame' && (
        <LoadGameScreen
          onBack={handleBackToMenu}
          onGameLoaded={handleGameLoaded}
        />
      )}

      {currentScreen === 'settings' && (
        <SettingsScreen onBack={handleBackToMenu} />
      )}

      {currentScreen === 'game' && <GameView onBackToMenu={handleBackToMenu} />}
    </div>
  )
}

export default App
