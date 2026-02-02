import React, { useCallback, useMemo, useState } from 'react'

// Types
type RelationshipStatus = 'Friendly' | 'Neutral' | 'Hostile' | 'War' | 'Allied'

interface Treaty {
  id: string
  type: 'OpenBorders' | 'DefensivePact' | 'ResearchAgreement' | 'TradeRoute'
  name: string
  turnsRemaining: number | null
}

interface DiplomaticEvent {
  id: string
  turn: number
  description: string
  type: 'positive' | 'neutral' | 'negative'
}

interface Civilization {
  id: number
  name: string
  leaderName: string
  traits: string[]
  relationshipScore: number
  relationshipStatus: RelationshipStatus
  color: string
  treaties: Treaty[]
  diplomaticHistory: DiplomaticEvent[]
}

interface TradeOffer {
  goldPerTurn: number
  lumpSumGold: number
  resources: string[]
  cities: string[]
}

interface TradeDeal {
  offer: TradeOffer
  request: TradeOffer
}

// Mock data for civilizations
const MOCK_CIVILIZATIONS: Civilization[] = [
  {
    id: 1,
    name: 'Roman Empire',
    leaderName: 'Augustus Caesar',
    traits: ['Expansionist', 'Militaristic'],
    relationshipScore: 45,
    relationshipStatus: 'Neutral',
    color: '#8B0000',
    treaties: [
      {
        id: 't1',
        type: 'OpenBorders',
        name: 'Open Borders',
        turnsRemaining: 12,
      },
    ],
    diplomaticHistory: [
      {
        id: 'e1',
        turn: 42,
        description: 'Established Open Borders agreement',
        type: 'positive',
      },
      {
        id: 'e2',
        turn: 38,
        description: 'First contact established',
        type: 'neutral',
      },
    ],
  },
  {
    id: 2,
    name: 'Greek City-States',
    leaderName: 'Alexander',
    traits: ['Scientific', 'Cultural'],
    relationshipScore: 78,
    relationshipStatus: 'Friendly',
    color: '#4169E1',
    treaties: [
      {
        id: 't2',
        type: 'ResearchAgreement',
        name: 'Research Agreement',
        turnsRemaining: 8,
      },
      {
        id: 't3',
        type: 'DefensivePact',
        name: 'Defensive Pact',
        turnsRemaining: null,
      },
    ],
    diplomaticHistory: [
      {
        id: 'e3',
        turn: 50,
        description: 'Signed Research Agreement',
        type: 'positive',
      },
      {
        id: 'e4',
        turn: 45,
        description: 'Formed Defensive Pact',
        type: 'positive',
      },
      {
        id: 'e5',
        turn: 30,
        description: 'First contact established',
        type: 'neutral',
      },
    ],
  },
  {
    id: 3,
    name: 'Egyptian Kingdom',
    leaderName: 'Cleopatra',
    traits: ['Wonder Builder', 'Trade-focused'],
    relationshipScore: 92,
    relationshipStatus: 'Allied',
    color: '#FFD700',
    treaties: [
      {
        id: 't4',
        type: 'OpenBorders',
        name: 'Open Borders',
        turnsRemaining: null,
      },
      {
        id: 't5',
        type: 'DefensivePact',
        name: 'Defensive Pact',
        turnsRemaining: null,
      },
      {
        id: 't6',
        type: 'TradeRoute',
        name: 'Trade Route',
        turnsRemaining: null,
      },
    ],
    diplomaticHistory: [
      { id: 'e6', turn: 55, description: 'Alliance formed', type: 'positive' },
      {
        id: 'e7',
        turn: 40,
        description: 'Mutual trade agreement established',
        type: 'positive',
      },
      {
        id: 'e8',
        turn: 25,
        description: 'First contact established',
        type: 'neutral',
      },
    ],
  },
  {
    id: 4,
    name: 'Persian Empire',
    leaderName: 'Cyrus',
    traits: ['Aggressive', 'Wealthy'],
    relationshipScore: -25,
    relationshipStatus: 'Hostile',
    color: '#800080',
    treaties: [],
    diplomaticHistory: [
      {
        id: 'e9',
        turn: 48,
        description: 'Denounced our civilization',
        type: 'negative',
      },
      {
        id: 'e10',
        turn: 35,
        description: 'Border dispute began',
        type: 'negative',
      },
      {
        id: 'e11',
        turn: 20,
        description: 'First contact established',
        type: 'neutral',
      },
    ],
  },
  {
    id: 5,
    name: 'Mongol Horde',
    leaderName: 'Genghis Khan',
    traits: ['Warmonger', 'Nomadic'],
    relationshipScore: -80,
    relationshipStatus: 'War',
    color: '#2F4F4F',
    treaties: [],
    diplomaticHistory: [
      { id: 'e12', turn: 52, description: 'War declared!', type: 'negative' },
      {
        id: 'e13',
        turn: 51,
        description: 'Ultimatum rejected',
        type: 'negative',
      },
      {
        id: 'e14',
        turn: 50,
        description: 'Troops massing on border',
        type: 'negative',
      },
      {
        id: 'e15',
        turn: 15,
        description: 'First contact established',
        type: 'neutral',
      },
    ],
  },
]

// Available resources for trading
const AVAILABLE_RESOURCES = [
  'Iron',
  'Horses',
  'Gold',
  'Silk',
  'Spices',
  'Wine',
  'Ivory',
]

// Props
interface DiplomacyScreenProps {
  isOpen: boolean
  onClose: () => void
  selectedPlayerId?: number
}

const STATUS_COLORS: Record<RelationshipStatus, string> = {
  Allied: 'bg-blue-500',
  Friendly: 'bg-green-500',
  Neutral: 'bg-yellow-500',
  Hostile: 'bg-orange-500',
  War: 'bg-red-600',
}

const STATUS_TEXT_COLORS: Record<RelationshipStatus, string> = {
  Allied: 'text-blue-400',
  Friendly: 'text-green-400',
  Neutral: 'text-yellow-400',
  Hostile: 'text-orange-400',
  War: 'text-red-400',
}

const DiplomacyScreen: React.FC<DiplomacyScreenProps> = ({
  isOpen,
  onClose,
  selectedPlayerId,
}) => {
  const [selectedCiv, setSelectedCiv] = useState<Civilization | null>(
    selectedPlayerId
      ? MOCK_CIVILIZATIONS.find((c) => c.id === selectedPlayerId) || null
      : null
  )
  const [showTradeDialog, setShowTradeDialog] = useState(false)
  const [tradeDeal, setTradeDeal] = useState<TradeDeal>({
    offer: { goldPerTurn: 0, lumpSumGold: 0, resources: [], cities: [] },
    request: { goldPerTurn: 0, lumpSumGold: 0, resources: [], cities: [] },
  })

  // Handle backdrop click
  const handleBackdropClick = useCallback(
    (e: React.MouseEvent) => {
      if (e.target === e.currentTarget) {
        onClose()
      }
    },
    [onClose]
  )

  // Get relationship score color
  const getScoreColor = (score: number): string => {
    if (score >= 75) return 'text-blue-400'
    if (score >= 50) return 'text-green-400'
    if (score >= 0) return 'text-yellow-400'
    if (score >= -50) return 'text-orange-400'
    return 'text-red-400'
  }

  // Diplomatic actions
  const handleDeclareWar = useCallback(() => {
    if (selectedCiv) {
      console.log(`Declaring war on ${selectedCiv.name}`)
      // Would call backend API here
    }
  }, [selectedCiv])

  const handleProposePeace = useCallback(() => {
    if (selectedCiv) {
      console.log(`Proposing peace to ${selectedCiv.name}`)
      // Would call backend API here
    }
  }, [selectedCiv])

  const handleOpenBorders = useCallback(() => {
    if (selectedCiv) {
      console.log(`Proposing Open Borders with ${selectedCiv.name}`)
      // Would call backend API here
    }
  }, [selectedCiv])

  const handleDefensivePact = useCallback(() => {
    if (selectedCiv) {
      console.log(`Proposing Defensive Pact with ${selectedCiv.name}`)
      // Would call backend API here
    }
  }, [selectedCiv])

  const handleResearchAgreement = useCallback(() => {
    if (selectedCiv) {
      console.log(`Proposing Research Agreement with ${selectedCiv.name}`)
      // Would call backend API here
    }
  }, [selectedCiv])

  const handleSubmitTrade = useCallback(() => {
    if (selectedCiv) {
      console.log(`Submitting trade deal to ${selectedCiv.name}:`, tradeDeal)
      // Would call backend API here
      setShowTradeDialog(false)
      setTradeDeal({
        offer: { goldPerTurn: 0, lumpSumGold: 0, resources: [], cities: [] },
        request: { goldPerTurn: 0, lumpSumGold: 0, resources: [], cities: [] },
      })
    }
  }, [selectedCiv, tradeDeal])

  // Check if treaty exists
  const hasTreaty = useCallback(
    (type: Treaty['type']): boolean => {
      return selectedCiv?.treaties.some((t) => t.type === type) || false
    },
    [selectedCiv]
  )

  // Sort civilizations by relationship status
  const sortedCivs = useMemo(() => {
    const statusOrder: Record<RelationshipStatus, number> = {
      Allied: 0,
      Friendly: 1,
      Neutral: 2,
      Hostile: 3,
      War: 4,
    }
    return [...MOCK_CIVILIZATIONS].sort(
      (a, b) =>
        statusOrder[a.relationshipStatus] - statusOrder[b.relationshipStatus]
    )
  }, [])

  if (!isOpen) return null

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/75"
      onClick={handleBackdropClick}
    >
      <div className="relative flex h-[90vh] w-[95vw] max-w-6xl overflow-hidden rounded-lg border border-primary-700 bg-background-light shadow-2xl">
        {/* Left Sidebar - Civilization List */}
        <div className="flex w-72 flex-col border-r border-primary-700 bg-background">
          <div className="border-b border-primary-700 px-4 py-3">
            <h3 className="font-header text-lg text-secondary">
              Known Civilizations
            </h3>
          </div>
          <div className="flex-1 overflow-y-auto">
            {sortedCivs.map((civ) => (
              <button
                key={civ.id}
                onClick={() => setSelectedCiv(civ)}
                className={`w-full border-b border-primary-700/50 p-3 text-left transition-colors hover:bg-background-lighter ${
                  selectedCiv?.id === civ.id ? 'bg-background-lighter' : ''
                }`}
              >
                <div className="flex items-center gap-3">
                  {/* Color indicator */}
                  <div
                    className="h-8 w-8 rounded-full border-2 border-white/20"
                    style={{ backgroundColor: civ.color }}
                  />
                  <div className="min-w-0 flex-1">
                    <div className="truncate font-semibold text-foreground">
                      {civ.name}
                    </div>
                    <div className="truncate text-sm text-foreground-muted">
                      {civ.leaderName}
                    </div>
                  </div>
                  {/* Status indicator */}
                  <div
                    className={`h-3 w-3 rounded-full ${STATUS_COLORS[civ.relationshipStatus]}`}
                    title={civ.relationshipStatus}
                  />
                </div>
              </button>
            ))}
          </div>
        </div>

        {/* Main Panel */}
        <div className="flex flex-1 flex-col">
          {/* Header */}
          <div className="flex items-center justify-between border-b border-primary-700 bg-background px-6 py-4">
            <h2 className="font-header text-2xl text-secondary">Diplomacy</h2>
            <button
              onClick={onClose}
              className="rounded p-2 transition-colors hover:bg-background-lighter"
              title="Close"
            >
              <CloseIcon className="h-6 w-6 text-foreground" />
            </button>
          </div>

          {/* Content */}
          <div className="flex-1 overflow-y-auto p-6">
            {selectedCiv ? (
              <div className="space-y-6">
                {/* Civilization Header */}
                <div className="flex items-start gap-6">
                  {/* Leader Portrait Placeholder */}
                  <div
                    className="flex h-32 w-32 items-center justify-center rounded-lg border-2 border-primary-700"
                    style={{ backgroundColor: selectedCiv.color + '40' }}
                  >
                    <LeaderIcon className="h-16 w-16 text-foreground-muted" />
                  </div>

                  <div className="flex-1">
                    <h3 className="font-header text-2xl text-foreground">
                      {selectedCiv.name}
                    </h3>
                    <p className="text-lg text-foreground-muted">
                      Leader: {selectedCiv.leaderName}
                    </p>
                    <div className="mt-2 flex flex-wrap gap-2">
                      {selectedCiv.traits.map((trait) => (
                        <span
                          key={trait}
                          className="rounded-full border border-primary-700 bg-background px-3 py-1 text-xs text-foreground-muted"
                        >
                          {trait}
                        </span>
                      ))}
                    </div>
                  </div>

                  {/* Relationship Status */}
                  <div className="text-right">
                    <div
                      className={`text-2xl font-bold ${getScoreColor(selectedCiv.relationshipScore)}`}
                    >
                      {selectedCiv.relationshipScore > 0 ? '+' : ''}
                      {selectedCiv.relationshipScore}
                    </div>
                    <div
                      className={`text-lg font-semibold ${STATUS_TEXT_COLORS[selectedCiv.relationshipStatus]}`}
                    >
                      {selectedCiv.relationshipStatus}
                    </div>
                  </div>
                </div>

                {/* Active Treaties */}
                <div className="rounded-lg border border-primary-700 bg-background p-4">
                  <h4 className="mb-3 font-header text-lg text-secondary">
                    Active Treaties & Agreements
                  </h4>
                  {selectedCiv.treaties.length > 0 ? (
                    <div className="space-y-2">
                      {selectedCiv.treaties.map((treaty) => (
                        <div
                          key={treaty.id}
                          className="flex items-center justify-between rounded bg-background-lighter p-2"
                        >
                          <div className="flex items-center gap-2">
                            <TreatyIcon className="h-5 w-5 text-secondary" />
                            <span className="text-foreground">
                              {treaty.name}
                            </span>
                          </div>
                          <span className="text-sm text-foreground-muted">
                            {treaty.turnsRemaining !== null
                              ? `${treaty.turnsRemaining} turns remaining`
                              : 'Permanent'}
                          </span>
                        </div>
                      ))}
                    </div>
                  ) : (
                    <p className="text-foreground-muted">
                      No active agreements
                    </p>
                  )}
                </div>

                {/* Diplomatic Actions */}
                <div className="rounded-lg border border-primary-700 bg-background p-4">
                  <h4 className="mb-3 font-header text-lg text-secondary">
                    Diplomatic Actions
                  </h4>
                  <div className="flex flex-wrap gap-3">
                    {selectedCiv.relationshipStatus === 'War' ? (
                      <ActionButton
                        onClick={handleProposePeace}
                        variant="peace"
                        icon={<PeaceIcon className="h-5 w-5" />}
                      >
                        Propose Peace
                      </ActionButton>
                    ) : (
                      <ActionButton
                        onClick={handleDeclareWar}
                        variant="war"
                        icon={<WarIcon className="h-5 w-5" />}
                      >
                        Declare War
                      </ActionButton>
                    )}

                    <ActionButton
                      onClick={() => setShowTradeDialog(true)}
                      variant="neutral"
                      icon={<TradeIcon className="h-5 w-5" />}
                      disabled={selectedCiv.relationshipStatus === 'War'}
                    >
                      Trade Deal
                    </ActionButton>

                    <ActionButton
                      onClick={handleOpenBorders}
                      variant="neutral"
                      icon={<BordersIcon className="h-5 w-5" />}
                      disabled={
                        selectedCiv.relationshipStatus === 'War' ||
                        hasTreaty('OpenBorders')
                      }
                    >
                      Open Borders
                    </ActionButton>

                    <ActionButton
                      onClick={handleDefensivePact}
                      variant="neutral"
                      icon={<ShieldIcon className="h-5 w-5" />}
                      disabled={
                        selectedCiv.relationshipStatus === 'War' ||
                        selectedCiv.relationshipStatus === 'Hostile' ||
                        hasTreaty('DefensivePact')
                      }
                    >
                      Defensive Pact
                    </ActionButton>

                    <ActionButton
                      onClick={handleResearchAgreement}
                      variant="neutral"
                      icon={<ResearchIcon className="h-5 w-5" />}
                      disabled={
                        selectedCiv.relationshipStatus === 'War' ||
                        selectedCiv.relationshipStatus === 'Hostile' ||
                        hasTreaty('ResearchAgreement')
                      }
                    >
                      Research Agreement
                    </ActionButton>
                  </div>
                </div>

                {/* Diplomatic History */}
                <div className="rounded-lg border border-primary-700 bg-background p-4">
                  <h4 className="mb-3 font-header text-lg text-secondary">
                    Recent Diplomatic History
                  </h4>
                  <div className="space-y-2">
                    {selectedCiv.diplomaticHistory.map((event) => (
                      <div
                        key={event.id}
                        className="flex items-center gap-3 rounded bg-background-lighter p-2"
                      >
                        <HistoryIcon
                          className={`h-4 w-4 ${
                            event.type === 'positive'
                              ? 'text-green-400'
                              : event.type === 'negative'
                                ? 'text-red-400'
                                : 'text-foreground-muted'
                          }`}
                        />
                        <span className="text-sm text-foreground-muted">
                          Turn {event.turn}:
                        </span>
                        <span className="text-sm text-foreground">
                          {event.description}
                        </span>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            ) : (
              <div className="flex h-full items-center justify-center">
                <p className="text-lg text-foreground-muted">
                  Select a civilization to view diplomatic options
                </p>
              </div>
            )}
          </div>
        </div>

        {/* Trade Dialog */}
        {showTradeDialog && selectedCiv && (
          <TradeDialog
            civilization={selectedCiv}
            tradeDeal={tradeDeal}
            setTradeDeal={setTradeDeal}
            onSubmit={handleSubmitTrade}
            onClose={() => setShowTradeDialog(false)}
          />
        )}
      </div>
    </div>
  )
}

// Action Button Component
interface ActionButtonProps {
  onClick: () => void
  variant: 'war' | 'peace' | 'neutral'
  icon: React.ReactNode
  disabled?: boolean
  children: React.ReactNode
}

const ActionButton: React.FC<ActionButtonProps> = ({
  onClick,
  variant,
  icon,
  disabled,
  children,
}) => {
  const variantStyles = {
    war: 'border-red-600 text-red-400 hover:bg-red-900/30',
    peace: 'border-green-600 text-green-400 hover:bg-green-900/30',
    neutral: 'border-primary-700 text-foreground hover:bg-background-lighter',
  }

  return (
    <button
      onClick={onClick}
      disabled={disabled}
      className={`flex items-center gap-2 rounded border px-4 py-2 transition-colors ${
        variantStyles[variant]
      } ${disabled ? 'cursor-not-allowed opacity-50' : ''}`}
    >
      {icon}
      {children}
    </button>
  )
}

// Trade Dialog Component
interface TradeDialogProps {
  civilization: Civilization
  tradeDeal: TradeDeal
  setTradeDeal: React.Dispatch<React.SetStateAction<TradeDeal>>
  onSubmit: () => void
  onClose: () => void
}

const TradeDialog: React.FC<TradeDialogProps> = ({
  civilization,
  tradeDeal,
  setTradeDeal,
  onSubmit,
  onClose,
}) => {
  const updateOffer = (field: keyof TradeOffer, value: number | string[]) => {
    setTradeDeal((prev) => ({
      ...prev,
      offer: { ...prev.offer, [field]: value },
    }))
  }

  const updateRequest = (field: keyof TradeOffer, value: number | string[]) => {
    setTradeDeal((prev) => ({
      ...prev,
      request: { ...prev.request, [field]: value },
    }))
  }

  const toggleResource = (side: 'offer' | 'request', resource: string) => {
    setTradeDeal((prev) => {
      const current = prev[side].resources
      const updated = current.includes(resource)
        ? current.filter((r) => r !== resource)
        : [...current, resource]
      return {
        ...prev,
        [side]: { ...prev[side], resources: updated },
      }
    })
  }

  return (
    <div className="absolute inset-0 flex items-center justify-center bg-black/50">
      <div className="w-full max-w-2xl rounded-lg border border-primary-700 bg-background-light p-6 shadow-xl">
        <div className="mb-4 flex items-center justify-between">
          <h3 className="font-header text-xl text-secondary">
            Trade with {civilization.name}
          </h3>
          <button
            onClick={onClose}
            className="rounded p-1 transition-colors hover:bg-background-lighter"
          >
            <CloseIcon className="h-5 w-5 text-foreground" />
          </button>
        </div>

        <div className="grid grid-cols-2 gap-6">
          {/* Our Offer */}
          <div className="rounded border border-primary-700 bg-background p-4">
            <h4 className="mb-3 font-semibold text-green-400">We Offer</h4>

            <div className="space-y-4">
              <div>
                <label className="mb-1 block text-sm text-foreground-muted">
                  Gold per Turn
                </label>
                <input
                  type="number"
                  min={0}
                  value={tradeDeal.offer.goldPerTurn}
                  onChange={(e) =>
                    updateOffer('goldPerTurn', parseInt(e.target.value) || 0)
                  }
                  className="w-full rounded border border-primary-700 bg-background-lighter px-3 py-2 text-foreground"
                />
              </div>

              <div>
                <label className="mb-1 block text-sm text-foreground-muted">
                  Lump Sum Gold
                </label>
                <input
                  type="number"
                  min={0}
                  value={tradeDeal.offer.lumpSumGold}
                  onChange={(e) =>
                    updateOffer('lumpSumGold', parseInt(e.target.value) || 0)
                  }
                  className="w-full rounded border border-primary-700 bg-background-lighter px-3 py-2 text-foreground"
                />
              </div>

              <div>
                <label className="mb-2 block text-sm text-foreground-muted">
                  Resources
                </label>
                <div className="flex flex-wrap gap-2">
                  {AVAILABLE_RESOURCES.map((resource) => (
                    <button
                      key={resource}
                      onClick={() => toggleResource('offer', resource)}
                      className={`rounded px-2 py-1 text-xs transition-colors ${
                        tradeDeal.offer.resources.includes(resource)
                          ? 'bg-green-600 text-white'
                          : 'bg-background-lighter text-foreground-muted hover:bg-background-lighter/80'
                      }`}
                    >
                      {resource}
                    </button>
                  ))}
                </div>
              </div>
            </div>
          </div>

          {/* Their Offer */}
          <div className="rounded border border-primary-700 bg-background p-4">
            <h4 className="mb-3 font-semibold text-yellow-400">We Request</h4>

            <div className="space-y-4">
              <div>
                <label className="mb-1 block text-sm text-foreground-muted">
                  Gold per Turn
                </label>
                <input
                  type="number"
                  min={0}
                  value={tradeDeal.request.goldPerTurn}
                  onChange={(e) =>
                    updateRequest('goldPerTurn', parseInt(e.target.value) || 0)
                  }
                  className="w-full rounded border border-primary-700 bg-background-lighter px-3 py-2 text-foreground"
                />
              </div>

              <div>
                <label className="mb-1 block text-sm text-foreground-muted">
                  Lump Sum Gold
                </label>
                <input
                  type="number"
                  min={0}
                  value={tradeDeal.request.lumpSumGold}
                  onChange={(e) =>
                    updateRequest('lumpSumGold', parseInt(e.target.value) || 0)
                  }
                  className="w-full rounded border border-primary-700 bg-background-lighter px-3 py-2 text-foreground"
                />
              </div>

              <div>
                <label className="mb-2 block text-sm text-foreground-muted">
                  Resources
                </label>
                <div className="flex flex-wrap gap-2">
                  {AVAILABLE_RESOURCES.map((resource) => (
                    <button
                      key={resource}
                      onClick={() => toggleResource('request', resource)}
                      className={`rounded px-2 py-1 text-xs transition-colors ${
                        tradeDeal.request.resources.includes(resource)
                          ? 'bg-yellow-600 text-white'
                          : 'bg-background-lighter text-foreground-muted hover:bg-background-lighter/80'
                      }`}
                    >
                      {resource}
                    </button>
                  ))}
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Actions */}
        <div className="mt-6 flex justify-end gap-3">
          <button
            onClick={onClose}
            className="rounded border border-primary-700 px-4 py-2 text-foreground transition-colors hover:bg-background-lighter"
          >
            Cancel
          </button>
          <button
            onClick={onSubmit}
            className="rounded bg-secondary px-6 py-2 font-semibold text-background transition-colors hover:bg-secondary-600"
          >
            Submit Deal
          </button>
        </div>
      </div>
    </div>
  )
}

// Icon Components
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

const LeaderIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z" />
  </svg>
)

const TreatyIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm-5 14H7v-2h7v2zm3-4H7v-2h10v2zm0-4H7V7h10v2z" />
  </svg>
)

const WarIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M7 5h10v2h-1.73l-2.77 8H9.5l-2.77-8H5V5h2zm5.5 12c1.1 0 2 .9 2 2s-.9 2-2 2-2-.9-2-2 .9-2 2-2z" />
  </svg>
)

const PeaceIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z" />
  </svg>
)

const TradeIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z" />
  </svg>
)

const BordersIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M3 5v14c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2H5c-1.11 0-2 .9-2 2zm12 4c0 1.66-1.34 3-3 3s-3-1.34-3-3 1.34-3 3-3 3 1.34 3 3zm-9 8c0-2 4-3.1 6-3.1s6 1.1 6 3.1v1H6v-1z" />
  </svg>
)

const ShieldIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4zm-2 16l-4-4 1.41-1.41L10 14.17l6.59-6.59L18 9l-8 8z" />
  </svg>
)

const ResearchIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M13 3v4h7l-8 12-8-12h7V3h2zm-2 14h2v2h-2v-2z" />
  </svg>
)

const HistoryIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M13 3c-4.97 0-9 4.03-9 9H1l3.89 3.89.07.14L9 12H6c0-3.87 3.13-7 7-7s7 3.13 7 7-3.13 7-7 7c-1.93 0-3.68-.79-4.94-2.06l-1.42 1.42C8.27 19.99 10.51 21 13 21c4.97 0 9-4.03 9-9s-4.03-9-9-9zm-1 5v5l4.28 2.54.72-1.21-3.5-2.08V8H12z" />
  </svg>
)

export default DiplomacyScreen
