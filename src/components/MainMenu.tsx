import React from 'react';
import { useIsTauri } from '@/hooks/useTauri';

interface MainMenuProps {
  onNewGame: () => void;
  onJoinGame: () => void;
  onLoadGame: () => void;
  onSettings: () => void;
  onExit: () => void;
}

const MainMenu: React.FC<MainMenuProps> = ({
  onNewGame,
  onJoinGame,
  onLoadGame,
  onSettings,
  onExit,
}) => {
  const isTauri = useIsTauri();

  return (
    <div className="h-full w-full flex flex-col items-center justify-center bg-gradient-game">
      {/* Title */}
      <h1 className="text-7xl font-header text-secondary text-glow mb-16 animate-fade-in">
        NOSTR NATIONS
      </h1>

      {/* Menu buttons */}
      <div className="flex flex-col gap-4 animate-slide-up">
        <MenuButton onClick={onNewGame}>New Game</MenuButton>
        <MenuButton onClick={onJoinGame}>Join Game</MenuButton>
        <MenuButton onClick={onLoadGame}>Load Game</MenuButton>
        <MenuButton onClick={onSettings}>Settings</MenuButton>
        {isTauri && (
          <MenuButton onClick={onExit} variant="danger">
            Exit
          </MenuButton>
        )}
      </div>

      {/* Footer */}
      <div className="absolute bottom-4 left-0 right-0 flex justify-between px-8">
        <span className="text-foreground-dim text-sm">Version 0.1.0</span>
        <ConnectionStatus />
      </div>
    </div>
  );
};

interface MenuButtonProps {
  children: React.ReactNode;
  onClick: () => void;
  variant?: 'primary' | 'danger';
}

const MenuButton: React.FC<MenuButtonProps> = ({
  children,
  onClick,
  variant = 'primary',
}) => {
  const baseClasses =
    'w-64 py-4 px-6 text-xl font-header rounded-lg transition-all duration-200 hover:scale-105 focus:outline-none focus:ring-2 focus:ring-secondary';

  const variantClasses = {
    primary: 'bg-primary hover:bg-primary-600 border-2 border-secondary text-foreground',
    danger: 'bg-danger/20 hover:bg-danger/30 border-2 border-danger text-danger',
  };

  return (
    <button
      className={`${baseClasses} ${variantClasses[variant]}`}
      onClick={onClick}
    >
      {children}
    </button>
  );
};

const ConnectionStatus: React.FC = () => {
  // TODO: Implement actual Nostr connection status
  const isConnected = false;

  return (
    <div className="flex items-center gap-2 text-sm">
      <span
        className={`w-2 h-2 rounded-full ${
          isConnected ? 'bg-success' : 'bg-foreground-dim'
        }`}
      />
      <span className="text-foreground-dim">
        Nostr: {isConnected ? 'connected' : 'disconnected'}
      </span>
    </div>
  );
};

export default MainMenu;
