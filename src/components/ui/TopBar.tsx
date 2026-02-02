import React from 'react';

interface TopBarProps {
  gold: number;
  science: number;
  culture: number;
  turn: number;
  onEndTurn: () => void;
  onMenu: () => void;
}

const TopBar: React.FC<TopBarProps> = ({
  gold,
  science,
  culture,
  turn,
  onEndTurn,
  onMenu,
}) => {
  return (
    <div className="h-14 bg-background-light border-b border-primary-700 flex items-center justify-between px-4">
      {/* Left section - Resources */}
      <div className="flex items-center gap-4">
        <ResourceDisplay icon="coins" label="Gold" value={gold} color="text-yellow-400" />
        <ResourceDisplay icon="flask" label="Science" value={science} color="text-blue-400" />
        <ResourceDisplay icon="masks" label="Culture" value={culture} color="text-purple-400" />
      </div>

      {/* Center section - Turn info */}
      <div className="flex items-center gap-4">
        <span className="text-foreground-muted">Turn</span>
        <span className="text-xl font-mono font-bold text-secondary">{turn}</span>
      </div>

      {/* Right section - Actions */}
      <div className="flex items-center gap-3">
        <button
          onClick={onEndTurn}
          className="px-6 py-2 bg-secondary hover:bg-secondary-600 text-background font-semibold rounded transition-colors duration-200 animate-pulse-gold"
        >
          End Turn
        </button>
        <button
          onClick={onMenu}
          className="p-2 hover:bg-background-lighter rounded transition-colors duration-200"
          title="Menu"
        >
          <MenuIcon />
        </button>
      </div>
    </div>
  );
};

interface ResourceDisplayProps {
  icon: string;
  label: string;
  value: number;
  color: string;
}

const ResourceDisplay: React.FC<ResourceDisplayProps> = ({
  icon,
  label,
  value,
  color,
}) => {
  // Simple icon representations
  const iconMap: Record<string, React.ReactNode> = {
    coins: <span className="text-yellow-400">$</span>,
    flask: <span className="text-blue-400">S</span>,
    masks: <span className="text-purple-400">C</span>,
  };

  return (
    <div className="flex items-center gap-2 px-3 py-1 bg-background rounded" title={label}>
      <span className="text-lg">{iconMap[icon]}</span>
      <span className={`font-mono font-medium ${color}`}>{value}</span>
    </div>
  );
};

const MenuIcon: React.FC = () => (
  <svg
    className="w-6 h-6 text-foreground"
    fill="none"
    stroke="currentColor"
    viewBox="0 0 24 24"
  >
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      d="M4 6h16M4 12h16M4 18h16"
    />
  </svg>
);

export default TopBar;
