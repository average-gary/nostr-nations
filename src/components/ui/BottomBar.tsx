import React from 'react';

interface BottomBarButtonProps {
  label: string;
  icon: React.ReactNode;
  onClick: () => void;
  shortcut?: string;
}

const BottomBarButton: React.FC<BottomBarButtonProps> = ({
  label,
  icon,
  onClick,
  shortcut,
}) => (
  <button
    onClick={onClick}
    className="flex flex-col items-center gap-1 px-4 py-2 hover:bg-background-lighter rounded transition-colors duration-200 group"
    title={shortcut ? `${label} (${shortcut})` : label}
  >
    <span className="text-foreground-muted group-hover:text-secondary transition-colors">
      {icon}
    </span>
    <span className="text-xs text-foreground-dim group-hover:text-foreground transition-colors">
      {label}
    </span>
  </button>
);

const BottomBar: React.FC = () => {
  const handleTechTree = () => console.log('Open Tech Tree');
  const handleCivics = () => console.log('Open Civics');
  const handleDiplomacy = () => console.log('Open Diplomacy');
  const handleMilitary = () => console.log('Open Military');
  const handleCities = () => console.log('Open Cities');
  const handleLeaders = () => console.log('Open Leaders');

  return (
    <div className="h-16 bg-background-light border-t border-primary-700 flex items-center justify-center gap-2">
      <BottomBarButton
        label="Tech"
        icon={<TechIcon />}
        onClick={handleTechTree}
        shortcut="T"
      />
      <BottomBarButton
        label="Civics"
        icon={<CivicsIcon />}
        onClick={handleCivics}
        shortcut="C"
      />
      <BottomBarButton
        label="Diplomacy"
        icon={<DiplomacyIcon />}
        onClick={handleDiplomacy}
        shortcut="D"
      />
      <BottomBarButton
        label="Military"
        icon={<MilitaryIcon />}
        onClick={handleMilitary}
        shortcut="M"
      />
      <BottomBarButton
        label="Cities"
        icon={<CitiesIcon />}
        onClick={handleCities}
        shortcut="V"
      />
      <BottomBarButton
        label="Leaders"
        icon={<LeadersIcon />}
        onClick={handleLeaders}
        shortcut="L"
      />
    </div>
  );
};

// Simple SVG icons
const TechIcon: React.FC = () => (
  <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
  </svg>
);

const CivicsIcon: React.FC = () => (
  <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4" />
  </svg>
);

const DiplomacyIcon: React.FC = () => (
  <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
  </svg>
);

const MilitaryIcon: React.FC = () => (
  <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
  </svg>
);

const CitiesIcon: React.FC = () => (
  <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4" />
  </svg>
);

const LeadersIcon: React.FC = () => (
  <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
  </svg>
);

export default BottomBar;
