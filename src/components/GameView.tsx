import React, { Suspense, useCallback } from 'react'
import { Canvas } from '@react-three/fiber'
import { OrbitControls } from '@react-three/drei'
import { useGameStore, selectCurrentPlayer } from '@/stores/gameStore'
import TopBar from './ui/TopBar'
import BottomBar from './ui/BottomBar'
import SelectionPanel from './ui/SelectionPanel'
import Minimap from './ui/Minimap'
import HexMap from './game/HexMap'
import { NotificationContainer } from './hud'
import type { Notification } from '@/types/game'

interface GameViewProps {
  onBackToMenu: () => void
}

const GameView: React.FC<GameViewProps> = ({ onBackToMenu }) => {
  const gameState = useGameStore((state) => state.gameState)
  const currentPlayer = useGameStore(selectCurrentPlayer)
  const endTurn = useGameStore((state) => state.endTurn)
  const selectTile = useGameStore((state) => state.selectTile)

  // Handle notification actions (e.g., navigate to location)
  const handleNotificationAction = useCallback(
    (notification: Notification) => {
      if (notification.action) {
        const { actionType, data } = notification.action

        switch (actionType) {
          case 'goto_location':
            // Navigate camera to specific hex coordinates
            if (data?.['q'] !== undefined && data?.['r'] !== undefined) {
              selectTile({ q: data['q'] as number, r: data['r'] as number })
              // Could also trigger camera pan here if setCameraPosition is enhanced
            }
            break
          case 'open_city':
            // Select a city
            if (data?.['cityId']) {
              useGameStore.getState().selectCity(data['cityId'] as string)
            }
            break
          case 'select_unit':
            // Select a unit
            if (data?.['unitId']) {
              useGameStore.getState().selectUnit(data['unitId'] as string)
            }
            break
          default:
            console.log('Unknown notification action:', actionType, data)
        }
      }
    },
    [selectTile]
  )

  if (!gameState) {
    return (
      <div className="flex h-full w-full items-center justify-center">
        <p className="text-foreground-muted">No game loaded</p>
      </div>
    )
  }

  return (
    <div className="flex h-full w-full flex-col">
      {/* Top bar - resources and turn info */}
      <TopBar
        gold={currentPlayer?.gold ?? 0}
        science={currentPlayer?.science ?? 0}
        culture={currentPlayer?.culture ?? 0}
        turn={gameState.turn}
        onEndTurn={endTurn}
        onMenu={onBackToMenu}
      />

      {/* Main game area */}
      <div className="relative flex-1">
        {/* 3D Canvas for hex map */}
        <Canvas
          camera={{ position: [0, 10, 10], fov: 60 }}
          className="h-full w-full"
        >
          <Suspense fallback={null}>
            <ambientLight intensity={0.5} />
            <directionalLight position={[10, 10, 5]} intensity={1} />
            <HexMap
              tiles={gameState.map}
              units={gameState.units}
              cities={gameState.cities}
            />
            <OrbitControls
              enablePan={true}
              enableZoom={true}
              enableRotate={true}
              maxPolarAngle={Math.PI / 3}
              minPolarAngle={Math.PI / 6}
              maxDistance={30}
              minDistance={5}
            />
          </Suspense>
        </Canvas>

        {/* UI Overlays */}
        <div className="absolute bottom-4 left-4">
          <Minimap />
        </div>

        <div className="absolute bottom-4 right-4">
          <SelectionPanel />
        </div>
      </div>

      {/* Bottom bar - quick actions */}
      <BottomBar />

      {/* Notification toasts */}
      <NotificationContainer
        position="top-right"
        onNotificationAction={handleNotificationAction}
      />
    </div>
  )
}

export default GameView
