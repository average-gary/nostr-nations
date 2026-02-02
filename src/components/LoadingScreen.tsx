import React from 'react';

const LoadingScreen: React.FC = () => {
  return (
    <div className="h-full w-full flex flex-col items-center justify-center bg-gradient-game">
      {/* Logo/Title */}
      <h1 className="text-6xl font-header text-secondary text-glow mb-8 animate-fade-in">
        NOSTR NATIONS
      </h1>

      {/* Loading spinner */}
      <div className="relative w-16 h-16 mb-8">
        <div className="absolute inset-0 border-4 border-primary-600 rounded-full animate-spin border-t-secondary" />
      </div>

      {/* Loading text */}
      <p className="text-foreground-muted text-lg animate-pulse">
        Loading...
      </p>

      {/* Version info */}
      <div className="absolute bottom-4 left-4 text-foreground-dim text-sm">
        Version 0.1.0
      </div>
    </div>
  );
};

export default LoadingScreen;
