import React, { useState, useMemo } from 'react'

// Types
type VictoryType =
  | 'domination'
  | 'science'
  | 'economic'
  | 'diplomatic'
  | 'score'
  | 'time'

interface PlayerRanking {
  rank: number
  name: string
  civilization: string
  score: number
  isWinner: boolean
}

interface GameStatistics {
  totalTurns: number
  citiesFounded: number
  unitsTrained: number
  technologiesResearched: number
  wondersBuilt: number
  warsWon: number
  warsLost: number
}

interface DominationDetails {
  conqueredCapitals: string[]
  totalCapitals: number
}

interface ScienceDetails {
  spaceshipParts: { name: string; completed: boolean }[]
  launchTurn: number
}

interface EconomicDetails {
  totalGoldAccumulated: number
  peakGoldPerTurn: number
  tradeRoutes: number
}

interface DiplomaticDetails {
  totalVotes: number
  votesReceived: number
  resolutionsPassed: number
}

interface ScoreDetails {
  landScore: number
  populationScore: number
  techScore: number
  wonderScore: number
  militaryScore: number
}

// Mock data
const MOCK_STATISTICS: GameStatistics = {
  totalTurns: 324,
  citiesFounded: 8,
  unitsTrained: 156,
  technologiesResearched: 52,
  wondersBuilt: 4,
  warsWon: 3,
  warsLost: 1,
}

const MOCK_RANKINGS: PlayerRanking[] = [
  {
    rank: 1,
    name: 'Player 1',
    civilization: 'Roman Empire',
    score: 2847,
    isWinner: true,
  },
  {
    rank: 2,
    name: 'Alexander',
    civilization: 'Greek Empire',
    score: 2341,
    isWinner: false,
  },
  {
    rank: 3,
    name: 'Cleopatra',
    civilization: 'Egyptian Kingdom',
    score: 2156,
    isWinner: false,
  },
  {
    rank: 4,
    name: 'Cyrus',
    civilization: 'Persian Empire',
    score: 1892,
    isWinner: false,
  },
  {
    rank: 5,
    name: 'Genghis Khan',
    civilization: 'Mongol Horde',
    score: 1654,
    isWinner: false,
  },
]

const MOCK_DOMINATION: DominationDetails = {
  conqueredCapitals: ['Athens', 'Thebes', 'Memphis', 'Persepolis'],
  totalCapitals: 5,
}

const MOCK_SCIENCE: ScienceDetails = {
  spaceshipParts: [
    { name: 'Booster', completed: true },
    { name: 'Cockpit', completed: true },
    { name: 'Stasis Chamber', completed: true },
    { name: 'Engine', completed: true },
    { name: 'Fuel Cells', completed: true },
  ],
  launchTurn: 318,
}

const MOCK_ECONOMIC: EconomicDetails = {
  totalGoldAccumulated: 125847,
  peakGoldPerTurn: 892,
  tradeRoutes: 12,
}

const MOCK_DIPLOMATIC: DiplomaticDetails = {
  totalVotes: 24,
  votesReceived: 15,
  resolutionsPassed: 8,
}

const MOCK_SCORE: ScoreDetails = {
  landScore: 520,
  populationScore: 680,
  techScore: 890,
  wonderScore: 400,
  militaryScore: 357,
}

// Props
interface VictoryScreenProps {
  isOpen: boolean
  victoryType: VictoryType
  isVictory: boolean
  winnerId: number
  winnerName: string
  winnerCivilization: string
  finalTurn: number
  onMainMenu: () => void
  onContinuePlaying: () => void
}

const VICTORY_LABELS: Record<VictoryType, string> = {
  domination: 'Domination Victory',
  science: 'Science Victory',
  economic: 'Economic Victory',
  diplomatic: 'Diplomatic Victory',
  score: 'Score Victory',
  time: 'Time Victory',
}

const VICTORY_DESCRIPTIONS: Record<VictoryType, string> = {
  domination: 'Conquered all enemy capitals through military might',
  science: 'Successfully launched the spaceship to Alpha Centauri',
  economic: 'Achieved unparalleled economic dominance',
  diplomatic: 'Elected World Leader by the United Nations',
  score: 'Achieved the highest civilization score',
  time: 'Highest score when time limit reached',
}

const VictoryScreen: React.FC<VictoryScreenProps> = ({
  isOpen,
  victoryType,
  isVictory,
  winnerId,
  winnerName,
  winnerCivilization,
  finalTurn,
  onMainMenu,
  onContinuePlaying,
}) => {
  const [showDetailedStats, setShowDetailedStats] = useState(false)

  // Adjust rankings to reflect winner
  const rankings = useMemo(() => {
    return MOCK_RANKINGS.map((r, index) => ({
      ...r,
      isWinner: index === 0 && isVictory,
      // Use winnerId to identify the player in the rankings
      name: index === 0 || r.rank === winnerId ? winnerName : r.name,
      civilization: index === 0 ? winnerCivilization : r.civilization,
    }))
  }, [winnerName, winnerCivilization, isVictory, winnerId])

  if (!isOpen) return null

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/90">
      <div
        className={`relative flex h-[95vh] w-[95vw] max-w-5xl flex-col overflow-hidden rounded-lg border-2 shadow-2xl ${
          isVictory
            ? 'border-yellow-500/50 bg-gradient-to-b from-yellow-900/20 via-background-light to-background-light'
            : 'border-red-900/50 bg-gradient-to-b from-red-900/20 via-background-light to-background-light'
        }`}
      >
        {/* Victory/Defeat Banner */}
        <div
          className={`relative overflow-hidden px-6 py-8 text-center ${
            isVictory
              ? 'bg-gradient-to-r from-yellow-900/30 via-yellow-800/40 to-yellow-900/30'
              : 'bg-gradient-to-r from-red-900/30 via-red-800/40 to-red-900/30'
          }`}
        >
          {/* Animated background stripes for victory */}
          {isVictory && (
            <div className="absolute inset-0 opacity-10">
              <div className="absolute inset-0 animate-pulse bg-[repeating-linear-gradient(45deg,transparent,transparent_10px,rgba(255,215,0,0.1)_10px,rgba(255,215,0,0.1)_20px)]" />
            </div>
          )}

          {/* Banner text */}
          <div className="relative">
            <h1
              className={`font-header text-5xl font-bold tracking-wider ${
                isVictory ? 'text-yellow-400' : 'text-red-400'
              } animate-pulse`}
            >
              {isVictory ? 'VICTORY!' : 'DEFEAT'}
            </h1>

            {/* Victory type */}
            <div className="mt-4">
              <span
                className={`rounded-full border px-6 py-2 text-lg font-semibold ${
                  isVictory
                    ? 'border-yellow-500/50 bg-yellow-900/30 text-yellow-300'
                    : 'border-red-500/50 bg-red-900/30 text-red-300'
                }`}
              >
                {VICTORY_LABELS[victoryType]}
              </span>
            </div>

            <p className="mt-3 text-foreground-muted">
              {VICTORY_DESCRIPTIONS[victoryType]}
            </p>
          </div>
        </div>

        {/* Winner Info */}
        <div className="border-b border-primary-700 bg-background px-6 py-4">
          <div className="flex items-center justify-center gap-8">
            {/* Winner Badge */}
            <div className="flex items-center gap-4">
              <div
                className={`flex h-16 w-16 items-center justify-center rounded-full border-2 ${
                  isVictory
                    ? 'border-yellow-500 bg-yellow-900/30'
                    : 'border-red-500 bg-red-900/30'
                }`}
              >
                {isVictory ? (
                  <TrophyIcon className="h-8 w-8 text-yellow-400" />
                ) : (
                  <DefeatIcon className="h-8 w-8 text-red-400" />
                )}
              </div>
              <div>
                <div className="text-xl font-bold text-foreground">
                  {winnerName}
                </div>
                <div className="text-foreground-muted">
                  {winnerCivilization}
                </div>
              </div>
            </div>

            {/* Divider */}
            <div className="h-12 w-px bg-primary-700" />

            {/* Final Turn */}
            <div className="text-center">
              <div className="text-3xl font-bold text-secondary">
                {finalTurn}
              </div>
              <div className="text-sm text-foreground-muted">Final Turn</div>
            </div>
          </div>
        </div>

        {/* Main Content - Scrollable */}
        <div className="flex-1 overflow-y-auto p-6">
          <div className="space-y-6">
            {/* Victory-Specific Content */}
            <VictoryTypeContent
              victoryType={victoryType}
              isVictory={isVictory}
            />

            {/* Game Statistics Summary */}
            <div className="rounded-lg border border-primary-700 bg-background p-4">
              <div className="mb-4 flex items-center justify-between">
                <h3 className="font-header text-lg text-secondary">
                  Game Statistics
                </h3>
                <button
                  onClick={() => setShowDetailedStats(!showDetailedStats)}
                  className="flex items-center gap-1 rounded border border-primary-700 px-3 py-1 text-sm text-foreground-muted transition-colors hover:bg-background-lighter hover:text-foreground"
                >
                  {showDetailedStats ? 'Hide Details' : 'Show Details'}
                  <ChevronIcon
                    className={`h-4 w-4 transition-transform ${showDetailedStats ? 'rotate-180' : ''}`}
                  />
                </button>
              </div>

              <div className="grid grid-cols-3 gap-4 md:grid-cols-6">
                <StatBox
                  label="Turns Played"
                  value={MOCK_STATISTICS.totalTurns}
                  icon={<TurnIcon className="h-5 w-5" />}
                />
                <StatBox
                  label="Cities Founded"
                  value={MOCK_STATISTICS.citiesFounded}
                  icon={<CityIcon className="h-5 w-5" />}
                />
                <StatBox
                  label="Units Trained"
                  value={MOCK_STATISTICS.unitsTrained}
                  icon={<UnitIcon className="h-5 w-5" />}
                />
                <StatBox
                  label="Techs Researched"
                  value={MOCK_STATISTICS.technologiesResearched}
                  icon={<TechIcon className="h-5 w-5" />}
                />
                <StatBox
                  label="Wonders Built"
                  value={MOCK_STATISTICS.wondersBuilt}
                  icon={<WonderIcon className="h-5 w-5" />}
                />
                <StatBox
                  label="Wars Won/Lost"
                  value={`${MOCK_STATISTICS.warsWon}/${MOCK_STATISTICS.warsLost}`}
                  icon={<WarIcon className="h-5 w-5" />}
                />
              </div>

              {/* Detailed Stats */}
              {showDetailedStats && (
                <div className="mt-4 border-t border-primary-700 pt-4">
                  <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
                    <DetailedStatBox
                      label="Land Controlled"
                      value="2,847 tiles"
                    />
                    <DetailedStatBox label="Population" value="42.5 million" />
                    <DetailedStatBox
                      label="Total Gold Earned"
                      value="125,847 gold"
                    />
                    <DetailedStatBox label="Trade Routes" value="12 active" />
                    <DetailedStatBox label="Great People" value="8 earned" />
                    <DetailedStatBox label="Policies Adopted" value="24" />
                    <DetailedStatBox label="Resources Controlled" value="18" />
                    <DetailedStatBox label="Battles Won" value="87" />
                  </div>
                </div>
              )}
            </div>

            {/* Player Rankings */}
            <div className="rounded-lg border border-primary-700 bg-background p-4">
              <h3 className="mb-4 font-header text-lg text-secondary">
                Final Rankings
              </h3>
              <div className="space-y-2">
                {rankings.map((player) => (
                  <div
                    key={player.rank}
                    className={`flex items-center gap-4 rounded-lg p-3 ${
                      player.isWinner
                        ? 'border border-yellow-500/30 bg-yellow-900/20'
                        : 'bg-background-lighter'
                    }`}
                  >
                    {/* Rank */}
                    <div
                      className={`flex h-10 w-10 items-center justify-center rounded-full font-bold ${
                        player.rank === 1
                          ? 'bg-yellow-500 text-background'
                          : player.rank === 2
                            ? 'bg-gray-400 text-background'
                            : player.rank === 3
                              ? 'bg-amber-700 text-foreground'
                              : 'bg-background text-foreground-muted'
                      }`}
                    >
                      {player.rank}
                    </div>

                    {/* Player Info */}
                    <div className="flex-1">
                      <div
                        className={`font-semibold ${player.isWinner ? 'text-yellow-400' : 'text-foreground'}`}
                      >
                        {player.name}
                        {player.isWinner && (
                          <span className="ml-2 text-xs text-yellow-500">
                            (YOU)
                          </span>
                        )}
                      </div>
                      <div className="text-sm text-foreground-muted">
                        {player.civilization}
                      </div>
                    </div>

                    {/* Score */}
                    <div className="text-right">
                      <div className="text-xl font-bold text-secondary">
                        {player.score.toLocaleString()}
                      </div>
                      <div className="text-xs text-foreground-muted">
                        points
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>

        {/* Action Buttons */}
        <div className="flex items-center justify-center gap-4 border-t border-primary-700 bg-background px-6 py-4">
          <button
            onClick={onMainMenu}
            className="rounded border border-primary-700 bg-background-lighter px-6 py-3 font-semibold text-foreground transition-colors hover:bg-background-lighter/80"
          >
            Main Menu
          </button>

          <button
            onClick={onContinuePlaying}
            className={`rounded px-6 py-3 font-semibold transition-colors ${
              isVictory
                ? 'bg-yellow-600 text-background hover:bg-yellow-500'
                : 'bg-secondary text-background hover:bg-secondary-600'
            }`}
          >
            One More Turn
          </button>

          <button
            onClick={() => console.log('View Replay - Coming Soon')}
            className="rounded border border-primary-700 bg-background-lighter px-6 py-3 font-semibold text-foreground-muted transition-colors hover:bg-background-lighter/80"
            title="Coming Soon"
          >
            View Replay
          </button>
        </div>
      </div>
    </div>
  )
}

// Victory Type Specific Content
interface VictoryTypeContentProps {
  victoryType: VictoryType
  isVictory: boolean
}

const VictoryTypeContent: React.FC<VictoryTypeContentProps> = ({
  victoryType,
  isVictory,
}) => {
  switch (victoryType) {
    case 'domination':
      return <DominationContent isVictory={isVictory} />
    case 'science':
      return <ScienceContent isVictory={isVictory} />
    case 'economic':
      return <EconomicContent isVictory={isVictory} />
    case 'diplomatic':
      return <DiplomaticContent isVictory={isVictory} />
    case 'score':
    case 'time':
      return <ScoreContent isVictory={isVictory} />
    default:
      return null
  }
}

const DominationContent: React.FC<{ isVictory: boolean }> = ({ isVictory }) => (
  <div className="rounded-lg border border-primary-700 bg-background p-4">
    <h3 className="mb-4 flex items-center gap-2 font-header text-lg text-secondary">
      <SwordIcon className="h-5 w-5" />
      {isVictory ? 'Capitals Conquered' : 'Enemy Conquest'}
    </h3>
    <div className="grid grid-cols-2 gap-3 md:grid-cols-4">
      {MOCK_DOMINATION.conqueredCapitals.map((capital, index) => (
        <div
          key={capital}
          className="flex items-center gap-2 rounded border border-red-500/30 bg-red-900/20 p-3"
        >
          <div className="flex h-8 w-8 items-center justify-center rounded-full bg-red-600 text-sm font-bold text-white">
            {index + 1}
          </div>
          <span className="font-semibold text-foreground">{capital}</span>
        </div>
      ))}
    </div>
    <div className="mt-3 text-center text-foreground-muted">
      {MOCK_DOMINATION.conqueredCapitals.length} of{' '}
      {MOCK_DOMINATION.totalCapitals} capitals conquered
    </div>
  </div>
)

const ScienceContent: React.FC<{ isVictory: boolean }> = ({ isVictory }) => (
  <div className="rounded-lg border border-primary-700 bg-background p-4">
    <h3 className="mb-4 flex items-center gap-2 font-header text-lg text-secondary">
      <RocketIcon className="h-5 w-5" />
      {isVictory ? 'Spaceship Completed' : 'Spaceship Progress'}
    </h3>
    <div className="grid grid-cols-5 gap-3">
      {MOCK_SCIENCE.spaceshipParts.map((part) => (
        <div
          key={part.name}
          className={`flex flex-col items-center rounded border p-3 ${
            part.completed
              ? 'border-cyan-500/30 bg-cyan-900/20'
              : 'border-gray-500/30 bg-gray-900/20'
          }`}
        >
          <div
            className={`h-8 w-8 rounded-full ${part.completed ? 'bg-cyan-500' : 'bg-gray-600'}`}
          >
            {part.completed && (
              <CheckIcon className="h-8 w-8 p-1 text-background" />
            )}
          </div>
          <span
            className={`mt-2 text-center text-sm ${part.completed ? 'text-cyan-400' : 'text-foreground-muted'}`}
          >
            {part.name}
          </span>
        </div>
      ))}
    </div>
    <div className="mt-3 text-center text-cyan-400">
      Launched on Turn {MOCK_SCIENCE.launchTurn}
    </div>
  </div>
)

const EconomicContent: React.FC<{ isVictory: boolean }> = ({ isVictory }) => (
  <div className="rounded-lg border border-primary-700 bg-background p-4">
    <h3 className="mb-4 flex items-center gap-2 font-header text-lg text-secondary">
      <GoldIcon className="h-5 w-5 text-yellow-400" />
      {isVictory ? 'Economic Dominance' : 'Economic Summary'}
    </h3>
    <div className="grid grid-cols-3 gap-4">
      <div className="rounded border border-yellow-500/30 bg-yellow-900/20 p-4 text-center">
        <div className="text-3xl font-bold text-yellow-400">
          {MOCK_ECONOMIC.totalGoldAccumulated.toLocaleString()}
        </div>
        <div className="text-sm text-foreground-muted">Total Gold Earned</div>
      </div>
      <div className="rounded border border-yellow-500/30 bg-yellow-900/20 p-4 text-center">
        <div className="text-3xl font-bold text-yellow-400">
          {MOCK_ECONOMIC.peakGoldPerTurn}
        </div>
        <div className="text-sm text-foreground-muted">Peak Gold/Turn</div>
      </div>
      <div className="rounded border border-yellow-500/30 bg-yellow-900/20 p-4 text-center">
        <div className="text-3xl font-bold text-yellow-400">
          {MOCK_ECONOMIC.tradeRoutes}
        </div>
        <div className="text-sm text-foreground-muted">Trade Routes</div>
      </div>
    </div>
  </div>
)

const DiplomaticContent: React.FC<{ isVictory: boolean }> = ({ isVictory }) => (
  <div className="rounded-lg border border-primary-700 bg-background p-4">
    <h3 className="mb-4 flex items-center gap-2 font-header text-lg text-secondary">
      <DiplomacyIcon className="h-5 w-5 text-blue-400" />
      {isVictory ? 'World Leader Elected' : 'UN Voting Results'}
    </h3>
    <div className="grid grid-cols-3 gap-4">
      <div className="rounded border border-blue-500/30 bg-blue-900/20 p-4 text-center">
        <div className="text-3xl font-bold text-blue-400">
          {MOCK_DIPLOMATIC.votesReceived}/{MOCK_DIPLOMATIC.totalVotes}
        </div>
        <div className="text-sm text-foreground-muted">Votes Received</div>
      </div>
      <div className="rounded border border-blue-500/30 bg-blue-900/20 p-4 text-center">
        <div className="text-3xl font-bold text-blue-400">
          {Math.round(
            (MOCK_DIPLOMATIC.votesReceived / MOCK_DIPLOMATIC.totalVotes) * 100
          )}
          %
        </div>
        <div className="text-sm text-foreground-muted">Vote Share</div>
      </div>
      <div className="rounded border border-blue-500/30 bg-blue-900/20 p-4 text-center">
        <div className="text-3xl font-bold text-blue-400">
          {MOCK_DIPLOMATIC.resolutionsPassed}
        </div>
        <div className="text-sm text-foreground-muted">Resolutions Passed</div>
      </div>
    </div>
  </div>
)

const ScoreContent: React.FC<{ isVictory: boolean }> = ({ isVictory }) => (
  <div className="rounded-lg border border-primary-700 bg-background p-4">
    <h3 className="mb-4 flex items-center gap-2 font-header text-lg text-secondary">
      <ScoreIcon className="h-5 w-5 text-purple-400" />
      {isVictory ? 'Score Breakdown' : 'Final Score'}
    </h3>
    <div className="space-y-3">
      <ScoreBar
        label="Land"
        value={MOCK_SCORE.landScore}
        max={1000}
        color="bg-green-500"
      />
      <ScoreBar
        label="Population"
        value={MOCK_SCORE.populationScore}
        max={1000}
        color="bg-blue-500"
      />
      <ScoreBar
        label="Technology"
        value={MOCK_SCORE.techScore}
        max={1000}
        color="bg-cyan-500"
      />
      <ScoreBar
        label="Wonders"
        value={MOCK_SCORE.wonderScore}
        max={1000}
        color="bg-yellow-500"
      />
      <ScoreBar
        label="Military"
        value={MOCK_SCORE.militaryScore}
        max={1000}
        color="bg-red-500"
      />
    </div>
    <div className="mt-4 border-t border-primary-700 pt-3 text-center">
      <span className="text-foreground-muted">Total Score: </span>
      <span className="text-2xl font-bold text-purple-400">
        {Object.values(MOCK_SCORE)
          .reduce((a, b) => a + b, 0)
          .toLocaleString()}
      </span>
    </div>
  </div>
)

// Reusable Components
interface StatBoxProps {
  label: string
  value: string | number
  icon: React.ReactNode
}

const StatBox: React.FC<StatBoxProps> = ({ label, value, icon }) => (
  <div className="rounded border border-primary-700 bg-background-lighter p-3 text-center">
    <div className="mb-1 flex justify-center text-secondary">{icon}</div>
    <div className="text-xl font-bold text-foreground">{value}</div>
    <div className="text-xs text-foreground-muted">{label}</div>
  </div>
)

interface DetailedStatBoxProps {
  label: string
  value: string
}

const DetailedStatBox: React.FC<DetailedStatBoxProps> = ({ label, value }) => (
  <div className="rounded bg-background-lighter p-2">
    <div className="text-sm text-foreground">{value}</div>
    <div className="text-xs text-foreground-muted">{label}</div>
  </div>
)

interface ScoreBarProps {
  label: string
  value: number
  max: number
  color: string
}

const ScoreBar: React.FC<ScoreBarProps> = ({ label, value, max, color }) => (
  <div className="flex items-center gap-3">
    <div className="w-24 text-sm text-foreground-muted">{label}</div>
    <div className="h-4 flex-1 overflow-hidden rounded-full bg-background-lighter">
      <div
        className={`h-full transition-all ${color}`}
        style={{ width: `${(value / max) * 100}%` }}
      />
    </div>
    <div className="w-12 text-right text-sm font-semibold text-foreground">
      {value}
    </div>
  </div>
)

// Icon Components
const TrophyIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M19 5h-2V3H7v2H5c-1.1 0-2 .9-2 2v1c0 2.55 1.92 4.63 4.39 4.94.63 1.5 1.98 2.63 3.61 2.96V19H7v2h10v-2h-4v-3.1c1.63-.33 2.98-1.46 3.61-2.96C19.08 12.63 21 10.55 21 8V7c0-1.1-.9-2-2-2zM5 8V7h2v3.82C5.84 10.4 5 9.3 5 8zm14 0c0 1.3-.84 2.4-2 2.82V7h2v1z" />
  </svg>
)

const DefeatIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z" />
  </svg>
)

const ChevronIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg
    className={className}
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth={2}
  >
    <path strokeLinecap="round" strokeLinejoin="round" d="M19 9l-7 7-7-7" />
  </svg>
)

const TurnIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M11.99 2C6.47 2 2 6.48 2 12s4.47 10 9.99 10C17.52 22 22 17.52 22 12S17.52 2 11.99 2zM12 20c-4.42 0-8-3.58-8-8s3.58-8 8-8 8 3.58 8 8-3.58 8-8 8zm.5-13H11v6l5.25 3.15.75-1.23-4.5-2.67z" />
  </svg>
)

const CityIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M15 11V5l-3-3-3 3v2H3v14h18V11h-6zm-8 8H5v-2h2v2zm0-4H5v-2h2v2zm0-4H5V9h2v2zm6 8h-2v-2h2v2zm0-4h-2v-2h2v2zm0-4h-2V9h2v2zm0-4h-2V5h2v2zm6 12h-2v-2h2v2zm0-4h-2v-2h2v2z" />
  </svg>
)

const UnitIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M12 2L4 5v6.09c0 5.05 3.41 9.76 8 10.91 4.59-1.15 8-5.86 8-10.91V5l-8-3z" />
  </svg>
)

const TechIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M9.4 16.6L4.8 12l4.6-4.6L8 6l-6 6 6 6 1.4-1.4zm5.2 0l4.6-4.6-4.6-4.6L16 6l6 6-6 6-1.4-1.4z" />
  </svg>
)

const WonderIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M12 2L4.5 20.29l.71.71L12 18l6.79 3 .71-.71z" />
  </svg>
)

const WarIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M7 5h10v2h-1.73l-2.77 8H9.5l-2.77-8H5V5h2z" />
  </svg>
)

const SwordIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M6.92 5H5l4 10-3.5 3.5 1.42 1.42L9 18.5V17h.5L14 7V5h-1.92l-1.96 4.88L6.92 5z" />
    <path d="M17.08 5H19l-4 10 3.5 3.5-1.42 1.42L15 18.5V17h-.5L10 7V5h1.92l1.96 4.88L17.08 5z" />
  </svg>
)

const RocketIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M9.19 6.35c-2.04 2.29-3.44 5.58-3.57 5.89L2 10.69l4.05-4.05c.47-.47 1.15-.68 1.81-.55l1.33 1.33v-.07zm5.46 6.46l1.33 1.33c.13.66-.08 1.34-.55 1.81L11.38 20l-1.55-3.62c.31-.13 3.6-1.53 5.89-3.57h-.07zm4.93-10.49c.01.18.02.36.02.55 0 4.41-2.23 8.39-5.85 10.75l-4.86-4.86C11.27 5.13 15.25 2.9 19.66 2.9c.19 0 .37.01.55.02l-.63-.63-1 1.94 2.63 2.63 1.94-1-1.57-.54zM5.5 20c-2.5-2.5-2.5-5 0-7.5l2 2c-1.5 1.5-1.5 3.5 0 5l-2 .5zm15-15l-.5 2c-1.5-1.5-3.5-1.5-5 0l-2-2c2.5-2.5 5-2.5 7.5 0z" />
  </svg>
)

const GoldIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <circle cx="12" cy="12" r="10" />
  </svg>
)

const DiplomacyIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z" />
  </svg>
)

const ScoreIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zM9 17H7v-7h2v7zm4 0h-2V7h2v10zm4 0h-2v-4h2v4z" />
  </svg>
)

const CheckIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg
    className={className}
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth={3}
  >
    <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
  </svg>
)

export default VictoryScreen
