import React from 'react';
import { useGameStore } from '@/stores/gameStore';

const Minimap: React.FC = () => {
  const gameState = useGameStore((state) => state.gameState);
  const setCameraPosition = useGameStore((state) => state.setCameraPosition);

  if (!gameState) return null;

  // Calculate map bounds
  const tiles = gameState.map;
  const minQ = Math.min(...tiles.map((t) => t.coord.q));
  const maxQ = Math.max(...tiles.map((t) => t.coord.q));
  const minR = Math.min(...tiles.map((t) => t.coord.r));
  const maxR = Math.max(...tiles.map((t) => t.coord.r));

  const width = maxQ - minQ + 1;
  const height = maxR - minR + 1;

  const handleClick = (e: React.MouseEvent<HTMLDivElement>) => {
    const rect = e.currentTarget.getBoundingClientRect();
    const x = ((e.clientX - rect.left) / rect.width) * width + minQ;
    const y = ((e.clientY - rect.top) / rect.height) * height + minR;

    // Convert to world coordinates and update camera
    setCameraPosition({ x: x * 1.5, y: 10, z: y * 1.732 + 10 });
  };

  // Get terrain color
  const getTerrainColor = (terrain: string): string => {
    const colors: Record<string, string> = {
      grassland: '#48bb78',
      plains: '#d69e2e',
      desert: '#ecc94b',
      tundra: '#a0aec0',
      snow: '#f7fafc',
      coast: '#4299e1',
      ocean: '#2b6cb0',
      mountain: '#718096',
    };
    return colors[terrain] ?? '#718096';
  };

  return (
    <div className="panel w-40 h-32">
      <div
        className="w-full h-full relative cursor-pointer"
        onClick={handleClick}
        title="Click to move camera"
      >
        {/* Render minimap tiles */}
        {tiles.map((tile) => {
          const x = ((tile.coord.q - minQ) / width) * 100;
          const y = ((tile.coord.r - minR) / height) * 100;
          const tileWidth = 100 / width;
          const tileHeight = 100 / height;

          return (
            <div
              key={`${tile.coord.q},${tile.coord.r}`}
              className="absolute"
              style={{
                left: `${x}%`,
                top: `${y}%`,
                width: `${tileWidth}%`,
                height: `${tileHeight}%`,
                backgroundColor: tile.owner
                  ? getPlayerColor(tile.owner)
                  : getTerrainColor(tile.terrain),
                opacity: tile.visibility === 'hidden' ? 0.3 : 1,
              }}
            />
          );
        })}

        {/* Render units as dots */}
        {gameState.units.map((unit) => {
          const x = ((unit.position.q - minQ) / width) * 100;
          const y = ((unit.position.r - minR) / height) * 100;

          return (
            <div
              key={unit.id}
              className="absolute w-2 h-2 rounded-full border border-white"
              style={{
                left: `${x}%`,
                top: `${y}%`,
                backgroundColor: getPlayerColor(unit.owner),
                transform: 'translate(-50%, -50%)',
              }}
            />
          );
        })}

        {/* Render cities as squares */}
        {gameState.cities.map((city) => {
          const x = ((city.position.q - minQ) / width) * 100;
          const y = ((city.position.r - minR) / height) * 100;

          return (
            <div
              key={city.id}
              className="absolute w-3 h-3 border-2 border-white"
              style={{
                left: `${x}%`,
                top: `${y}%`,
                backgroundColor: getPlayerColor(city.owner),
                transform: 'translate(-50%, -50%)',
              }}
            />
          );
        })}

        {/* Current view indicator (placeholder rectangle) */}
        <div className="absolute border-2 border-secondary opacity-70 pointer-events-none"
          style={{
            left: '30%',
            top: '30%',
            width: '40%',
            height: '40%',
          }}
        />
      </div>
    </div>
  );
};

// Helper function to get player color
function getPlayerColor(playerId: string): string {
  const colors: Record<string, string> = {
    'player-1': '#3182ce',
    'player-2': '#e53e3e',
    'player-3': '#38a169',
    'player-4': '#d69e2e',
  };
  return colors[playerId] ?? '#718096';
}

export default Minimap;
