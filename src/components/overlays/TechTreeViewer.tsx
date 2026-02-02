import React, { useCallback, useMemo, useState } from 'react'
import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'

// Types
type TechStatus = 'researched' | 'researching' | 'available' | 'locked'
type Era =
  | 'Ancient'
  | 'Classical'
  | 'Medieval'
  | 'Renaissance'
  | 'Industrial'
  | 'Modern'

interface TechUnlock {
  type: 'unit' | 'building' | 'improvement'
  name: string
}

interface Technology {
  id: string
  name: string
  era: Era
  cost: number
  prerequisites: string[]
  unlocks: TechUnlock[]
  description: string
}

interface TechTreeState {
  technologies: Technology[]
  researchedTechs: string[]
  currentResearch: string | null
  researchProgress: number
  sciencePerTurn: number
}

interface TechTreeStore extends TechTreeState {
  setCurrentResearch: (techId: string | null) => void
  addResearchedTech: (techId: string) => void
  setResearchProgress: (progress: number) => void
}

// Mock data for the tech tree
const MOCK_TECHNOLOGIES: Technology[] = [
  // Ancient Era
  {
    id: 'agriculture',
    name: 'Agriculture',
    era: 'Ancient',
    cost: 20,
    prerequisites: [],
    unlocks: [
      { type: 'building', name: 'Granary' },
      { type: 'improvement', name: 'Farm' },
    ],
    description: 'The foundation of civilization.',
  },
  {
    id: 'pottery',
    name: 'Pottery',
    era: 'Ancient',
    cost: 35,
    prerequisites: ['agriculture'],
    unlocks: [{ type: 'building', name: 'Shrine' }],
    description: 'Enables storage and religious structures.',
  },
  {
    id: 'animal_husbandry',
    name: 'Animal Husbandry',
    era: 'Ancient',
    cost: 35,
    prerequisites: ['agriculture'],
    unlocks: [
      { type: 'unit', name: 'Scout' },
      { type: 'improvement', name: 'Pasture' },
    ],
    description: 'Domestication of animals for work and food.',
  },
  {
    id: 'mining',
    name: 'Mining',
    era: 'Ancient',
    cost: 35,
    prerequisites: [],
    unlocks: [{ type: 'improvement', name: 'Mine' }],
    description: 'Extract resources from the earth.',
  },
  {
    id: 'bronze_working',
    name: 'Bronze Working',
    era: 'Ancient',
    cost: 55,
    prerequisites: ['mining'],
    unlocks: [
      { type: 'unit', name: 'Spearman' },
      { type: 'building', name: 'Barracks' },
    ],
    description: 'Forge bronze weapons and armor.',
  },
  {
    id: 'writing',
    name: 'Writing',
    era: 'Ancient',
    cost: 55,
    prerequisites: ['pottery'],
    unlocks: [{ type: 'building', name: 'Library' }],
    description: 'Record knowledge for future generations.',
  },

  // Classical Era
  {
    id: 'iron_working',
    name: 'Iron Working',
    era: 'Classical',
    cost: 120,
    prerequisites: ['bronze_working'],
    unlocks: [{ type: 'unit', name: 'Swordsman' }],
    description: 'Superior weapons from iron ore.',
  },
  {
    id: 'mathematics',
    name: 'Mathematics',
    era: 'Classical',
    cost: 120,
    prerequisites: ['writing'],
    unlocks: [
      { type: 'unit', name: 'Catapult' },
      { type: 'building', name: 'Courthouse' },
    ],
    description: 'The language of science.',
  },
  {
    id: 'currency',
    name: 'Currency',
    era: 'Classical',
    cost: 120,
    prerequisites: ['writing'],
    unlocks: [{ type: 'building', name: 'Market' }],
    description: 'Standardized medium of exchange.',
  },
  {
    id: 'construction',
    name: 'Construction',
    era: 'Classical',
    cost: 120,
    prerequisites: ['mining', 'animal_husbandry'],
    unlocks: [
      { type: 'building', name: 'Colosseum' },
      { type: 'improvement', name: 'Lumber Mill' },
    ],
    description: 'Advanced building techniques.',
  },

  // Medieval Era
  {
    id: 'engineering',
    name: 'Engineering',
    era: 'Medieval',
    cost: 250,
    prerequisites: ['mathematics', 'construction'],
    unlocks: [
      { type: 'building', name: 'Aqueduct' },
      { type: 'improvement', name: 'Fort' },
    ],
    description: 'Complex machinery and infrastructure.',
  },
  {
    id: 'steel',
    name: 'Steel',
    era: 'Medieval',
    cost: 275,
    prerequisites: ['iron_working'],
    unlocks: [{ type: 'unit', name: 'Knight' }],
    description: 'Stronger than iron, lighter than bronze.',
  },
  {
    id: 'education',
    name: 'Education',
    era: 'Medieval',
    cost: 275,
    prerequisites: ['mathematics', 'currency'],
    unlocks: [{ type: 'building', name: 'University' }],
    description: 'Formal institutions of learning.',
  },

  // Renaissance Era
  {
    id: 'gunpowder',
    name: 'Gunpowder',
    era: 'Renaissance',
    cost: 450,
    prerequisites: ['steel', 'engineering'],
    unlocks: [{ type: 'unit', name: 'Musketman' }],
    description: 'Revolutionary weapon technology.',
  },
  {
    id: 'printing_press',
    name: 'Printing Press',
    era: 'Renaissance',
    cost: 400,
    prerequisites: ['education'],
    unlocks: [{ type: 'building', name: 'Print Shop' }],
    description: 'Mass production of written works.',
  },
  {
    id: 'astronomy',
    name: 'Astronomy',
    era: 'Renaissance',
    cost: 450,
    prerequisites: ['education'],
    unlocks: [{ type: 'building', name: 'Observatory' }],
    description: 'Understanding the cosmos.',
  },

  // Industrial Era
  {
    id: 'industrialization',
    name: 'Industrialization',
    era: 'Industrial',
    cost: 700,
    prerequisites: ['gunpowder', 'printing_press'],
    unlocks: [
      { type: 'building', name: 'Factory' },
      { type: 'unit', name: 'Infantry' },
    ],
    description: 'Mass production and mechanization.',
  },
  {
    id: 'electricity',
    name: 'Electricity',
    era: 'Industrial',
    cost: 800,
    prerequisites: ['industrialization'],
    unlocks: [{ type: 'building', name: 'Power Plant' }],
    description: 'Harness the power of electrons.',
  },
  {
    id: 'railroad',
    name: 'Railroad',
    era: 'Industrial',
    cost: 750,
    prerequisites: ['industrialization'],
    unlocks: [{ type: 'improvement', name: 'Railroad' }],
    description: 'Rapid land transportation.',
  },

  // Modern Era
  {
    id: 'computers',
    name: 'Computers',
    era: 'Modern',
    cost: 1200,
    prerequisites: ['electricity'],
    unlocks: [{ type: 'building', name: 'Research Lab' }],
    description: 'Digital computation and information.',
  },
  {
    id: 'rocketry',
    name: 'Rocketry',
    era: 'Modern',
    cost: 1500,
    prerequisites: ['computers'],
    unlocks: [{ type: 'unit', name: 'Rocket Artillery' }],
    description: 'Propulsion beyond the atmosphere.',
  },
  {
    id: 'internet',
    name: 'Internet',
    era: 'Modern',
    cost: 1800,
    prerequisites: ['computers'],
    unlocks: [{ type: 'building', name: 'Broadcast Tower' }],
    description: 'Global connectivity.',
  },
]

// Zustand store for tech tree
const useTechTreeStore = create<TechTreeStore>((set) => ({
  technologies: MOCK_TECHNOLOGIES,
  researchedTechs: ['agriculture', 'mining'],
  currentResearch: 'pottery',
  researchProgress: 15,
  sciencePerTurn: 5,

  setCurrentResearch: (techId) =>
    set({ currentResearch: techId, researchProgress: 0 }),
  addResearchedTech: (techId) =>
    set((state) => ({
      researchedTechs: [...state.researchedTechs, techId],
      currentResearch:
        state.currentResearch === techId ? null : state.currentResearch,
    })),
  setResearchProgress: (progress) => set({ researchProgress: progress }),
}))

// Props
interface TechTreeViewerProps {
  isOpen: boolean
  onClose: () => void
}

// Era order for column layout
const ERA_ORDER: Era[] = [
  'Ancient',
  'Classical',
  'Medieval',
  'Renaissance',
  'Industrial',
  'Modern',
]

const ERA_COLORS: Record<Era, string> = {
  Ancient: 'border-amber-600',
  Classical: 'border-blue-500',
  Medieval: 'border-gray-400',
  Renaissance: 'border-purple-500',
  Industrial: 'border-orange-500',
  Modern: 'border-cyan-400',
}

const TechTreeViewer: React.FC<TechTreeViewerProps> = ({ isOpen, onClose }) => {
  const {
    technologies,
    researchedTechs,
    currentResearch,
    researchProgress,
    sciencePerTurn,
    setCurrentResearch,
  } = useTechTreeStore()

  const [selectedTech, setSelectedTech] = useState<Technology | null>(null)

  // Calculate tech status
  const getTechStatus = useCallback(
    (tech: Technology): TechStatus => {
      if (researchedTechs.includes(tech.id)) return 'researched'
      if (currentResearch === tech.id) return 'researching'
      if (
        tech.prerequisites.every((prereq) => researchedTechs.includes(prereq))
      )
        return 'available'
      return 'locked'
    },
    [researchedTechs, currentResearch]
  )

  // Group technologies by era
  const techsByEra = useMemo(() => {
    const grouped: Record<Era, Technology[]> = {
      Ancient: [],
      Classical: [],
      Medieval: [],
      Renaissance: [],
      Industrial: [],
      Modern: [],
    }
    technologies.forEach((tech) => {
      grouped[tech.era].push(tech)
    })
    return grouped
  }, [technologies])

  // Get current research info
  const currentResearchTech = useMemo(
    () => technologies.find((t) => t.id === currentResearch),
    [technologies, currentResearch]
  )

  const turnsRemaining = useMemo(() => {
    if (!currentResearchTech || sciencePerTurn === 0) return Infinity
    const remaining = currentResearchTech.cost - researchProgress
    return Math.ceil(remaining / sciencePerTurn)
  }, [currentResearchTech, researchProgress, sciencePerTurn])

  // Handle research button click
  const handleSetResearch = useCallback(
    async (techId: string) => {
      try {
        await invoke('set_research', { tech_id: techId })
        setCurrentResearch(techId)
        setSelectedTech(null)
      } catch (error) {
        console.error('Failed to set research:', error)
        // Still update local state for demo purposes
        setCurrentResearch(techId)
        setSelectedTech(null)
      }
    },
    [setCurrentResearch]
  )

  // Handle backdrop click
  const handleBackdropClick = useCallback(
    (e: React.MouseEvent) => {
      if (e.target === e.currentTarget) {
        onClose()
      }
    },
    [onClose]
  )

  // Status colors
  const getStatusStyles = (status: TechStatus): string => {
    switch (status) {
      case 'researched':
        return 'bg-green-900/80 border-green-500 text-green-100'
      case 'researching':
        return 'bg-blue-900/80 border-blue-400 text-blue-100 animate-pulse'
      case 'available':
        return 'bg-white/10 border-white/50 text-white hover:bg-white/20'
      case 'locked':
        return 'bg-gray-800/50 border-gray-600 text-gray-500'
    }
  }

  if (!isOpen) return null

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/75"
      onClick={handleBackdropClick}
    >
      <div className="relative flex h-[90vh] w-[95vw] flex-col overflow-hidden rounded-lg border border-primary-700 bg-background-light shadow-2xl">
        {/* Header */}
        <div className="flex items-center justify-between border-b border-primary-700 bg-background px-6 py-4">
          <h2 className="font-header text-2xl text-secondary">
            Technology Tree
          </h2>

          {/* Current Research Progress */}
          {currentResearchTech && (
            <div className="flex items-center gap-4">
              <div className="flex flex-col items-end">
                <span className="text-sm text-foreground-muted">
                  Researching
                </span>
                <span className="text-lg font-semibold text-blue-400">
                  {currentResearchTech.name}
                </span>
              </div>
              <div className="w-48">
                <div className="mb-1 flex justify-between text-xs text-foreground-muted">
                  <span>
                    {researchProgress} / {currentResearchTech.cost}
                  </span>
                  <span>{turnsRemaining} turns</span>
                </div>
                <div className="h-2 overflow-hidden rounded-full bg-background">
                  <div
                    className="h-full bg-blue-500 transition-all duration-300"
                    style={{
                      width: `${(researchProgress / currentResearchTech.cost) * 100}%`,
                    }}
                  />
                </div>
              </div>
              <div className="flex items-center gap-1 text-blue-400">
                <span className="font-mono text-lg">+{sciencePerTurn}</span>
                <ScienceIcon className="h-5 w-5" />
              </div>
            </div>
          )}

          {/* Close Button */}
          <button
            onClick={onClose}
            className="rounded p-2 transition-colors hover:bg-background-lighter"
            title="Close"
          >
            <CloseIcon className="h-6 w-6 text-foreground" />
          </button>
        </div>

        {/* Tech Tree Content */}
        <div className="flex-1 overflow-auto p-4">
          <div className="flex min-w-max gap-4">
            {ERA_ORDER.map((era) => (
              <div key={era} className="flex min-w-[200px] flex-col">
                {/* Era Header */}
                <div
                  className={`mb-4 border-b-2 px-4 py-2 text-center ${ERA_COLORS[era]}`}
                >
                  <span className="font-header text-lg text-foreground">
                    {era}
                  </span>
                </div>

                {/* Technologies in this Era */}
                <div className="flex flex-col gap-3">
                  {techsByEra[era].map((tech) => {
                    const status = getTechStatus(tech)
                    return (
                      <TechCard
                        key={tech.id}
                        tech={tech}
                        status={status}
                        isSelected={selectedTech?.id === tech.id}
                        onClick={() => setSelectedTech(tech)}
                        statusStyles={getStatusStyles(status)}
                      />
                    )
                  })}
                </div>
              </div>
            ))}
          </div>

          {/* Connection Lines SVG Overlay */}
          <TechConnections
            technologies={technologies}
            techsByEra={techsByEra}
            getTechStatus={getTechStatus}
          />
        </div>

        {/* Tech Details Panel */}
        {selectedTech && (
          <TechDetailsPanel
            tech={selectedTech}
            status={getTechStatus(selectedTech)}
            researchProgress={
              currentResearch === selectedTech.id ? researchProgress : 0
            }
            onClose={() => setSelectedTech(null)}
            onResearch={() => handleSetResearch(selectedTech.id)}
          />
        )}
      </div>
    </div>
  )
}

// Tech Card Component
interface TechCardProps {
  tech: Technology
  status: TechStatus
  isSelected: boolean
  onClick: () => void
  statusStyles: string
}

const TechCard: React.FC<TechCardProps> = ({
  tech,
  status,
  isSelected,
  onClick,
  statusStyles,
}) => {
  return (
    <button
      onClick={onClick}
      data-tech-id={tech.id}
      className={`
        relative rounded-lg border-2 p-3 text-left transition-all duration-200
        ${statusStyles}
        ${isSelected ? 'scale-105 ring-2 ring-secondary' : ''}
        ${status !== 'locked' ? 'cursor-pointer' : 'cursor-not-allowed'}
      `}
    >
      <div className="mb-1 flex items-center gap-2">
        <TechIcon className="h-6 w-6" />
        <span className="text-sm font-semibold">{tech.name}</span>
      </div>
      <div className="flex items-center gap-1 text-xs opacity-75">
        <ScienceIcon className="h-4 w-4" />
        <span>{tech.cost}</span>
      </div>

      {/* Status indicator */}
      {status === 'researched' && (
        <div className="absolute right-1 top-1">
          <CheckIcon className="h-4 w-4 text-green-400" />
        </div>
      )}
      {status === 'researching' && (
        <div className="absolute right-1 top-1">
          <ResearchingIcon className="h-4 w-4 text-blue-400" />
        </div>
      )}
      {status === 'locked' && (
        <div className="absolute right-1 top-1">
          <LockIcon className="h-4 w-4 text-gray-500" />
        </div>
      )}
    </button>
  )
}

// Tech Details Panel
interface TechDetailsPanelProps {
  tech: Technology
  status: TechStatus
  researchProgress: number
  onClose: () => void
  onResearch: () => void
}

const TechDetailsPanel: React.FC<TechDetailsPanelProps> = ({
  tech,
  status,
  researchProgress,
  onClose,
  onResearch,
}) => {
  return (
    <div className="absolute bottom-0 left-0 right-0 border-t border-primary-700 bg-background p-6">
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <div className="mb-2 flex items-center gap-3">
            <TechIcon className="h-8 w-8 text-secondary" />
            <div>
              <h3 className="font-header text-xl text-foreground">
                {tech.name}
              </h3>
              <span className="text-sm text-foreground-muted">
                {tech.era} Era
              </span>
            </div>
          </div>
          <p className="mb-4 text-foreground-muted">{tech.description}</p>

          {/* Prerequisites */}
          {tech.prerequisites.length > 0 && (
            <div className="mb-3">
              <span className="text-sm font-semibold text-foreground-muted">
                Requires:{' '}
              </span>
              <span className="text-sm text-foreground">
                {tech.prerequisites.join(', ')}
              </span>
            </div>
          )}

          {/* Unlocks */}
          <div className="flex flex-wrap gap-2">
            {tech.unlocks.map((unlock, i) => (
              <div
                key={i}
                className="rounded-full border border-primary-700 bg-background-lighter px-3 py-1 text-xs"
              >
                <span className="text-foreground-muted">{unlock.type}: </span>
                <span className="text-foreground">{unlock.name}</span>
              </div>
            ))}
          </div>
        </div>

        <div className="flex flex-col items-end gap-3">
          {/* Cost */}
          <div className="flex items-center gap-2">
            <ScienceIcon className="h-5 w-5 text-blue-400" />
            <span className="font-mono text-lg text-blue-400">
              {status === 'researching'
                ? `${researchProgress}/${tech.cost}`
                : tech.cost}
            </span>
          </div>

          {/* Action Button */}
          {status === 'available' && (
            <button
              onClick={onResearch}
              className="rounded bg-secondary px-6 py-2 font-semibold text-background transition-colors hover:bg-secondary-600"
            >
              Research
            </button>
          )}
          {status === 'researching' && (
            <span className="rounded border border-blue-500 bg-blue-900/50 px-4 py-2 text-blue-400">
              Researching...
            </span>
          )}
          {status === 'researched' && (
            <span className="rounded border border-green-500 bg-green-900/50 px-4 py-2 text-green-400">
              Completed
            </span>
          )}
          {status === 'locked' && (
            <span className="rounded border border-gray-600 bg-gray-800/50 px-4 py-2 text-gray-500">
              Locked
            </span>
          )}

          <button
            onClick={onClose}
            className="text-sm text-foreground-muted hover:text-foreground"
          >
            Close Details
          </button>
        </div>
      </div>
    </div>
  )
}

// Connection Lines Component
interface TechConnectionsProps {
  technologies: Technology[]
  techsByEra: Record<Era, Technology[]>
  getTechStatus: (tech: Technology) => TechStatus
}

const TechConnections: React.FC<TechConnectionsProps> = (_props) => {
  // This is a simplified version - in a real implementation,
  // you'd calculate actual positions from DOM elements
  // For now, we'll rely on CSS positioning or skip the lines

  return null // Connection lines would need DOM measurements
}

// Icon Components
const ScienceIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M13 3v4h7l-8 12-8-12h7V3h2zm-2 14h2v2h-2v-2z" />
  </svg>
)

const TechIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" />
  </svg>
)

const CloseIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg
    className={className}
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth={2}
  >
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      d="M6 18L18 6M6 6l12 12"
    />
  </svg>
)

const CheckIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z" />
  </svg>
)

const LockIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M18 8h-1V6c0-2.76-2.24-5-5-5S7 3.24 7 6v2H6c-1.1 0-2 .9-2 2v10c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V10c0-1.1-.9-2-2-2zm-6 9c-1.1 0-2-.9-2-2s.9-2 2-2 2 .9 2 2-.9 2-2 2zm3.1-9H8.9V6c0-1.71 1.39-3.1 3.1-3.1 1.71 0 3.1 1.39 3.1 3.1v2z" />
  </svg>
)

const ResearchingIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M12 4V1L8 5l4 4V6c3.31 0 6 2.69 6 6 0 1.01-.25 1.97-.7 2.8l1.46 1.46C19.54 15.03 20 13.57 20 12c0-4.42-3.58-8-8-8zm0 14c-3.31 0-6-2.69-6-6 0-1.01.25-1.97.7-2.8L5.24 7.74C4.46 8.97 4 10.43 4 12c0 4.42 3.58 8 8 8v3l4-4-4-4v3z" />
  </svg>
)

export default TechTreeViewer
