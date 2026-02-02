import React, { useCallback, useMemo, useState } from 'react'

// Types
type UnitType = 'Land' | 'Naval' | 'Air' | 'Civilian'
type UnitStatus = 'Idle' | 'Fortified' | 'Moving' | 'Combat'
type SortOption = 'Name' | 'Type' | 'Health' | 'Location'
type FilterOption = 'All' | UnitType

interface MilitaryUnit {
  id: number
  name: string
  type: UnitType
  health: number
  maxHealth: number
  location: string
  coordinates: { x: number; y: number }
  movementPoints: number
  maxMovementPoints: number
  status: UnitStatus
  experience: number
  promotions: string[]
  maintenanceCost: number
}

// Mock data
const MOCK_UNITS: MilitaryUnit[] = [
  {
    id: 1,
    name: 'Warrior',
    type: 'Land',
    health: 100,
    maxHealth: 100,
    location: 'Rome',
    coordinates: { x: 12, y: 8 },
    movementPoints: 2,
    maxMovementPoints: 2,
    status: 'Idle',
    experience: 15,
    promotions: ['Shock I'],
    maintenanceCost: 1,
  },
  {
    id: 2,
    name: 'Spearman',
    type: 'Land',
    health: 75,
    maxHealth: 100,
    location: 'Rome',
    coordinates: { x: 12, y: 8 },
    movementPoints: 0,
    maxMovementPoints: 2,
    status: 'Fortified',
    experience: 30,
    promotions: ['Drill I', 'Drill II'],
    maintenanceCost: 2,
  },
  {
    id: 3,
    name: 'Archer',
    type: 'Land',
    health: 100,
    maxHealth: 100,
    location: 'Field (15, 10)',
    coordinates: { x: 15, y: 10 },
    movementPoints: 1,
    maxMovementPoints: 2,
    status: 'Moving',
    experience: 5,
    promotions: [],
    maintenanceCost: 2,
  },
  {
    id: 4,
    name: 'Swordsman',
    type: 'Land',
    health: 45,
    maxHealth: 100,
    location: 'Near Athens',
    coordinates: { x: 20, y: 15 },
    movementPoints: 0,
    maxMovementPoints: 2,
    status: 'Combat',
    experience: 50,
    promotions: ['Shock I', 'Shock II', 'Cover I'],
    maintenanceCost: 3,
  },
  {
    id: 5,
    name: 'Horseman',
    type: 'Land',
    health: 100,
    maxHealth: 100,
    location: 'Milan',
    coordinates: { x: 18, y: 6 },
    movementPoints: 4,
    maxMovementPoints: 4,
    status: 'Idle',
    experience: 0,
    promotions: [],
    maintenanceCost: 3,
  },
  {
    id: 6,
    name: 'Catapult',
    type: 'Land',
    health: 80,
    maxHealth: 100,
    location: 'Rome',
    coordinates: { x: 12, y: 8 },
    movementPoints: 2,
    maxMovementPoints: 2,
    status: 'Idle',
    experience: 10,
    promotions: [],
    maintenanceCost: 4,
  },
  {
    id: 7,
    name: 'Trireme',
    type: 'Naval',
    health: 100,
    maxHealth: 100,
    location: 'Ostia Harbor',
    coordinates: { x: 10, y: 12 },
    movementPoints: 4,
    maxMovementPoints: 4,
    status: 'Idle',
    experience: 20,
    promotions: ['Boarding Party I'],
    maintenanceCost: 3,
  },
  {
    id: 8,
    name: 'Galley',
    type: 'Naval',
    health: 60,
    maxHealth: 100,
    location: 'Mediterranean',
    coordinates: { x: 8, y: 18 },
    movementPoints: 0,
    maxMovementPoints: 3,
    status: 'Moving',
    experience: 35,
    promotions: ['Coastal Raider I', 'Supply'],
    maintenanceCost: 2,
  },
  {
    id: 9,
    name: 'Scout',
    type: 'Land',
    health: 100,
    maxHealth: 100,
    location: 'Exploring (25, 3)',
    coordinates: { x: 25, y: 3 },
    movementPoints: 3,
    maxMovementPoints: 3,
    status: 'Moving',
    experience: 40,
    promotions: ['Survivalism I', 'Survivalism II'],
    maintenanceCost: 0,
  },
  {
    id: 10,
    name: 'Fighter',
    type: 'Air',
    health: 100,
    maxHealth: 100,
    location: 'Rome',
    coordinates: { x: 12, y: 8 },
    movementPoints: 10,
    maxMovementPoints: 10,
    status: 'Idle',
    experience: 0,
    promotions: [],
    maintenanceCost: 5,
  },
  {
    id: 11,
    name: 'Bomber',
    type: 'Air',
    health: 85,
    maxHealth: 100,
    location: 'Milan',
    coordinates: { x: 18, y: 6 },
    movementPoints: 8,
    maxMovementPoints: 8,
    status: 'Idle',
    experience: 25,
    promotions: ['Air Targeting I'],
    maintenanceCost: 6,
  },
  {
    id: 12,
    name: 'Worker',
    type: 'Civilian',
    health: 100,
    maxHealth: 100,
    location: 'Near Rome',
    coordinates: { x: 13, y: 9 },
    movementPoints: 2,
    maxMovementPoints: 2,
    status: 'Idle',
    experience: 0,
    promotions: [],
    maintenanceCost: 0,
  },
  {
    id: 13,
    name: 'Settler',
    type: 'Civilian',
    health: 100,
    maxHealth: 100,
    location: 'Field (22, 11)',
    coordinates: { x: 22, y: 11 },
    movementPoints: 1,
    maxMovementPoints: 2,
    status: 'Moving',
    experience: 0,
    promotions: [],
    maintenanceCost: 0,
  },
  {
    id: 14,
    name: 'Knight',
    type: 'Land',
    health: 100,
    maxHealth: 100,
    location: 'Venice',
    coordinates: { x: 16, y: 4 },
    movementPoints: 3,
    maxMovementPoints: 4,
    status: 'Idle',
    experience: 60,
    promotions: ['Charge', 'Shock I', 'Shock II'],
    maintenanceCost: 5,
  },
  {
    id: 15,
    name: 'Caravel',
    type: 'Naval',
    health: 100,
    maxHealth: 100,
    location: 'Atlantic',
    coordinates: { x: 2, y: 20 },
    movementPoints: 5,
    maxMovementPoints: 5,
    status: 'Moving',
    experience: 10,
    promotions: [],
    maintenanceCost: 3,
  },
]

// Props
interface MilitaryOverviewProps {
  isOpen: boolean
  onClose: () => void
  onSelectUnit?: (unitId: number) => void
}

const MilitaryOverview: React.FC<MilitaryOverviewProps> = ({
  isOpen,
  onClose,
  onSelectUnit,
}) => {
  const [filter, setFilter] = useState<FilterOption>('All')
  const [sortBy, setSortBy] = useState<SortOption>('Name')
  const [searchQuery, setSearchQuery] = useState('')
  const [statsCollapsed, setStatsCollapsed] = useState(false)

  // Filter and sort units
  const filteredUnits = useMemo(() => {
    let units = [...MOCK_UNITS]

    // Apply filter
    if (filter !== 'All') {
      units = units.filter((u) => u.type === filter)
    }

    // Apply search
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase()
      units = units.filter(
        (u) =>
          u.name.toLowerCase().includes(query) ||
          u.location.toLowerCase().includes(query)
      )
    }

    // Apply sort
    units.sort((a, b) => {
      switch (sortBy) {
        case 'Name':
          return a.name.localeCompare(b.name)
        case 'Type':
          return a.type.localeCompare(b.type)
        case 'Health':
          return b.health / b.maxHealth - a.health / a.maxHealth
        case 'Location':
          return a.location.localeCompare(b.location)
        default:
          return 0
      }
    })

    return units
  }, [filter, sortBy, searchQuery])

  // Calculate statistics
  const stats = useMemo(() => {
    const landUnits = MOCK_UNITS.filter((u) => u.type === 'Land').length
    const navalUnits = MOCK_UNITS.filter((u) => u.type === 'Naval').length
    const airUnits = MOCK_UNITS.filter((u) => u.type === 'Air').length
    const civilianUnits = MOCK_UNITS.filter((u) => u.type === 'Civilian').length

    const militaryUnits = MOCK_UNITS.filter((u) => u.type !== 'Civilian')
    const totalHealth = militaryUnits.reduce((sum, u) => sum + u.health, 0)
    const totalMaxHealth = militaryUnits.reduce(
      (sum, u) => sum + u.maxHealth,
      0
    )
    const avgHealth =
      militaryUnits.length > 0
        ? Math.round((totalHealth / totalMaxHealth) * 100)
        : 0

    const inCombat = MOCK_UNITS.filter((u) => u.status === 'Combat').length
    const totalMaintenance = MOCK_UNITS.reduce(
      (sum, u) => sum + u.maintenanceCost,
      0
    )

    // Calculate military strength (simplified formula)
    const strengthScore = militaryUnits.reduce((sum, u) => {
      const healthPercent = u.health / u.maxHealth
      const baseStrength = u.type === 'Air' ? 30 : u.type === 'Naval' ? 20 : 15
      const promotionBonus = u.promotions.length * 5
      return sum + Math.round(baseStrength * healthPercent + promotionBonus)
    }, 0)

    return {
      total: MOCK_UNITS.length,
      land: landUnits,
      naval: navalUnits,
      air: airUnits,
      civilian: civilianUnits,
      avgHealth,
      inCombat,
      maintenance: totalMaintenance,
      strength: strengthScore,
    }
  }, [])

  // Handle backdrop click
  const handleBackdropClick = useCallback(
    (e: React.MouseEvent) => {
      if (e.target === e.currentTarget) {
        onClose()
      }
    },
    [onClose]
  )

  // Handle unit selection
  const handleUnitClick = useCallback(
    (unitId: number) => {
      onSelectUnit?.(unitId)
    },
    [onSelectUnit]
  )

  // Handle unit actions
  const handleFortify = useCallback((unitId: number, e: React.MouseEvent) => {
    e.stopPropagation()
    console.log('Fortify unit:', unitId)
  }, [])

  const handleSleep = useCallback((unitId: number, e: React.MouseEvent) => {
    e.stopPropagation()
    console.log('Sleep unit:', unitId)
  }, [])

  const handleDelete = useCallback((unitId: number, e: React.MouseEvent) => {
    e.stopPropagation()
    console.log('Delete unit:', unitId)
  }, [])

  if (!isOpen) return null

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/75"
      onClick={handleBackdropClick}
    >
      <div className="relative flex h-[90vh] w-[95vw] max-w-6xl flex-col overflow-hidden rounded-lg border border-primary-700 bg-background-light shadow-2xl">
        {/* Header */}
        <div className="flex items-center justify-between border-b border-primary-700 bg-background px-6 py-4">
          <div className="flex items-center gap-6">
            <h2 className="font-header text-2xl text-secondary">
              Military Overview
            </h2>

            {/* Strength Score */}
            <div className="flex items-center gap-2 rounded-lg bg-red-900/30 px-4 py-2">
              <StrengthIcon className="h-5 w-5 text-red-400" />
              <span className="text-lg font-bold text-red-400">
                {stats.strength}
              </span>
              <span className="text-sm text-red-300">Military Strength</span>
            </div>
          </div>

          {/* Unit Count Summary */}
          <div className="flex items-center gap-4">
            <span className="text-foreground-muted">
              {stats.total} Units:{' '}
              <span className="text-amber-400">{stats.land} Land</span>,{' '}
              <span className="text-blue-400">{stats.naval} Naval</span>,{' '}
              <span className="text-cyan-400">{stats.air} Air</span>,{' '}
              <span className="text-gray-400">{stats.civilian} Civilian</span>
            </span>

            {/* Close Button */}
            <button
              onClick={onClose}
              className="rounded p-2 transition-colors hover:bg-background-lighter"
              title="Close"
            >
              <CloseIcon className="h-6 w-6 text-foreground" />
            </button>
          </div>
        </div>

        {/* Filter/Sort Controls */}
        <div className="flex items-center gap-4 border-b border-primary-700/50 bg-background-light px-6 py-3">
          {/* Filter by Type */}
          <div className="flex items-center gap-2">
            <label className="text-sm text-foreground-muted">Filter:</label>
            <select
              value={filter}
              onChange={(e) => setFilter(e.target.value as FilterOption)}
              className="rounded border border-primary-700 bg-background px-3 py-1 text-sm text-foreground focus:border-secondary focus:outline-none"
            >
              <option value="All">All Types</option>
              <option value="Land">Land</option>
              <option value="Naval">Naval</option>
              <option value="Air">Air</option>
              <option value="Civilian">Civilian</option>
            </select>
          </div>

          {/* Sort by */}
          <div className="flex items-center gap-2">
            <label className="text-sm text-foreground-muted">Sort:</label>
            <select
              value={sortBy}
              onChange={(e) => setSortBy(e.target.value as SortOption)}
              className="rounded border border-primary-700 bg-background px-3 py-1 text-sm text-foreground focus:border-secondary focus:outline-none"
            >
              <option value="Name">Name</option>
              <option value="Type">Type</option>
              <option value="Health">Health</option>
              <option value="Location">Location</option>
            </select>
          </div>

          {/* Search Box */}
          <div className="flex flex-1 items-center gap-2">
            <SearchIcon className="h-4 w-4 text-foreground-muted" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search units..."
              className="flex-1 rounded border border-primary-700 bg-background px-3 py-1 text-sm text-foreground placeholder:text-foreground-muted focus:border-secondary focus:outline-none"
            />
          </div>

          {/* Toggle Stats */}
          <button
            onClick={() => setStatsCollapsed(!statsCollapsed)}
            className="flex items-center gap-1 rounded border border-primary-700 bg-background px-3 py-1 text-sm text-foreground-muted transition-colors hover:bg-background-lighter hover:text-foreground"
          >
            <ChartIcon className="h-4 w-4" />
            Stats
            <ChevronIcon
              className={`h-4 w-4 transition-transform ${statsCollapsed ? '' : 'rotate-180'}`}
            />
          </button>
        </div>

        {/* Statistics Panel (Collapsible) */}
        {!statsCollapsed && (
          <div className="grid grid-cols-4 gap-4 border-b border-primary-700/50 bg-background px-6 py-4">
            <div className="rounded-lg border border-primary-700 bg-background-light p-3">
              <div className="text-xs text-foreground-muted">Units by Type</div>
              <div className="mt-2 space-y-1">
                <div className="flex justify-between text-sm">
                  <span className="text-amber-400">Land</span>
                  <span className="text-foreground">{stats.land}</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-blue-400">Naval</span>
                  <span className="text-foreground">{stats.naval}</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-cyan-400">Air</span>
                  <span className="text-foreground">{stats.air}</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-gray-400">Civilian</span>
                  <span className="text-foreground">{stats.civilian}</span>
                </div>
              </div>
            </div>

            <div className="rounded-lg border border-primary-700 bg-background-light p-3">
              <div className="text-xs text-foreground-muted">
                Average Health
              </div>
              <div className="mt-2 flex items-center gap-2">
                <div className="h-2 flex-1 overflow-hidden rounded-full bg-background">
                  <div
                    className={`h-full transition-all ${
                      stats.avgHealth >= 70
                        ? 'bg-green-500'
                        : stats.avgHealth >= 40
                          ? 'bg-yellow-500'
                          : 'bg-red-500'
                    }`}
                    style={{ width: `${stats.avgHealth}%` }}
                  />
                </div>
                <span className="text-lg font-bold text-foreground">
                  {stats.avgHealth}%
                </span>
              </div>
            </div>

            <div className="rounded-lg border border-primary-700 bg-background-light p-3">
              <div className="text-xs text-foreground-muted">
                Units in Combat
              </div>
              <div className="mt-2 flex items-center gap-2">
                <CombatIcon className="h-6 w-6 text-red-400" />
                <span className="text-2xl font-bold text-red-400">
                  {stats.inCombat}
                </span>
              </div>
            </div>

            <div className="rounded-lg border border-primary-700 bg-background-light p-3">
              <div className="text-xs text-foreground-muted">
                Maintenance Cost
              </div>
              <div className="mt-2 flex items-center gap-2">
                <GoldIcon className="h-6 w-6 text-yellow-400" />
                <span className="text-2xl font-bold text-yellow-400">
                  {stats.maintenance}
                </span>
                <span className="text-sm text-foreground-muted">/turn</span>
              </div>
            </div>
          </div>
        )}

        {/* Unit List */}
        <div className="flex-1 overflow-auto p-4">
          {filteredUnits.length === 0 ? (
            <div className="flex h-full items-center justify-center text-foreground-muted">
              No units found
            </div>
          ) : (
            <div className="grid gap-3">
              {filteredUnits.map((unit) => (
                <UnitCard
                  key={unit.id}
                  unit={unit}
                  onClick={() => handleUnitClick(unit.id)}
                  onFortify={(e) => handleFortify(unit.id, e)}
                  onSleep={(e) => handleSleep(unit.id, e)}
                  onDelete={(e) => handleDelete(unit.id, e)}
                />
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

// Unit Card Component
interface UnitCardProps {
  unit: MilitaryUnit
  onClick: () => void
  onFortify: (e: React.MouseEvent) => void
  onSleep: (e: React.MouseEvent) => void
  onDelete: (e: React.MouseEvent) => void
}

const UnitCard: React.FC<UnitCardProps> = ({
  unit,
  onClick,
  onFortify,
  onSleep,
  onDelete,
}) => {
  const healthPercent = (unit.health / unit.maxHealth) * 100
  const healthColor =
    healthPercent >= 70
      ? 'bg-green-500'
      : healthPercent >= 40
        ? 'bg-yellow-500'
        : 'bg-red-500'

  const typeColor = {
    Land: 'text-amber-400 border-amber-400/30 bg-amber-900/20',
    Naval: 'text-blue-400 border-blue-400/30 bg-blue-900/20',
    Air: 'text-cyan-400 border-cyan-400/30 bg-cyan-900/20',
    Civilian: 'text-gray-400 border-gray-400/30 bg-gray-900/20',
  }[unit.type]

  const statusColor = {
    Idle: 'text-gray-400',
    Fortified: 'text-blue-400',
    Moving: 'text-green-400',
    Combat: 'text-red-400',
  }[unit.status]

  const statusIcon = {
    Idle: <IdleIcon className="h-4 w-4" />,
    Fortified: <FortifyIcon className="h-4 w-4" />,
    Moving: <MovingIcon className="h-4 w-4" />,
    Combat: <CombatIcon className="h-4 w-4" />,
  }[unit.status]

  return (
    <div
      onClick={onClick}
      className="flex cursor-pointer items-center gap-4 rounded-lg border border-primary-700 bg-background p-4 transition-all hover:border-secondary hover:bg-background-lighter"
    >
      {/* Unit Type Icon Placeholder */}
      <div
        className={`flex h-12 w-12 items-center justify-center rounded-lg border ${typeColor}`}
      >
        <UnitTypeIcon type={unit.type} className="h-6 w-6" />
      </div>

      {/* Name and Type */}
      <div className="min-w-[140px]">
        <div className="font-semibold text-foreground">{unit.name}</div>
        <div className={`text-sm ${typeColor.split(' ')[0]}`}>{unit.type}</div>
      </div>

      {/* Health Bar */}
      <div className="min-w-[120px]">
        <div className="mb-1 flex justify-between text-xs text-foreground-muted">
          <span>Health</span>
          <span>
            {unit.health}/{unit.maxHealth}
          </span>
        </div>
        <div className="h-2 overflow-hidden rounded-full bg-background-lighter">
          <div
            className={`h-full transition-all ${healthColor}`}
            style={{ width: `${healthPercent}%` }}
          />
        </div>
      </div>

      {/* Location */}
      <div className="min-w-[140px]">
        <div className="text-xs text-foreground-muted">Location</div>
        <div className="text-sm text-foreground">{unit.location}</div>
      </div>

      {/* Movement Points */}
      <div className="min-w-[80px]">
        <div className="text-xs text-foreground-muted">Movement</div>
        <div className="text-sm text-foreground">
          {unit.movementPoints}/{unit.maxMovementPoints}
        </div>
      </div>

      {/* Status */}
      <div className={`flex min-w-[100px] items-center gap-1 ${statusColor}`}>
        {statusIcon}
        <span className="text-sm">{unit.status}</span>
      </div>

      {/* Experience/Promotions */}
      <div className="min-w-[120px]">
        <div className="text-xs text-foreground-muted">
          XP: {unit.experience}
        </div>
        {unit.promotions.length > 0 ? (
          <div className="flex flex-wrap gap-1">
            {unit.promotions.slice(0, 2).map((promo, i) => (
              <span
                key={i}
                className="rounded bg-purple-900/50 px-1 text-xs text-purple-300"
              >
                {promo}
              </span>
            ))}
            {unit.promotions.length > 2 && (
              <span className="text-xs text-foreground-muted">
                +{unit.promotions.length - 2}
              </span>
            )}
          </div>
        ) : (
          <div className="text-xs text-foreground-muted">No promotions</div>
        )}
      </div>

      {/* Action Buttons */}
      <div className="ml-auto flex items-center gap-2">
        {unit.type !== 'Civilian' && unit.status !== 'Fortified' && (
          <button
            onClick={onFortify}
            className="rounded border border-blue-500/50 bg-blue-900/30 p-2 text-blue-400 transition-colors hover:bg-blue-900/50"
            title="Fortify"
          >
            <FortifyIcon className="h-4 w-4" />
          </button>
        )}
        <button
          onClick={onSleep}
          className="rounded border border-gray-500/50 bg-gray-900/30 p-2 text-gray-400 transition-colors hover:bg-gray-900/50"
          title="Sleep"
        >
          <SleepIcon className="h-4 w-4" />
        </button>
        <button
          onClick={onDelete}
          className="rounded border border-red-500/50 bg-red-900/30 p-2 text-red-400 transition-colors hover:bg-red-900/50"
          title="Delete"
        >
          <DeleteIcon className="h-4 w-4" />
        </button>
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

const StrengthIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M12 2L4 5v6.09c0 5.05 3.41 9.76 8 10.91 4.59-1.15 8-5.86 8-10.91V5l-8-3zm6 9.09c0 4-2.55 7.7-6 8.83-3.45-1.13-6-4.82-6-8.83V6.31l6-2.25 6 2.25v4.78z" />
  </svg>
)

const SearchIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg
    className={className}
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth={2}
  >
    <circle cx="11" cy="11" r="8" />
    <path d="M21 21l-4.35-4.35" />
  </svg>
)

const ChartIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zM9 17H7v-7h2v7zm4 0h-2V7h2v10zm4 0h-2v-4h2v4z" />
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

const CombatIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M19.78 3.22l-2-2-9.5 9.5-4.5-4.5-2 2 6.5 6.5 11.5-11.5zm-2.06 12.56l-6.44-6.44 1.42-1.42 6.44 6.44-1.42 1.42zm-8.28 2.5l-1.42 1.42-2.12-2.12 1.42-1.42 2.12 2.12z" />
  </svg>
)

const GoldIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <circle cx="12" cy="12" r="10" />
  </svg>
)

const IdleIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg
    className={className}
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth={2}
  >
    <circle cx="12" cy="12" r="10" />
  </svg>
)

const FortifyIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4z" />
  </svg>
)

const MovingIcon: React.FC<{ className?: string }> = ({ className }) => (
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
      d="M17 8l4 4-4 4M3 12h18"
    />
  </svg>
)

const SleepIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M12.34 2.02C6.59 1.82 2 6.42 2 12c0 5.52 4.48 10 10 10 3.71 0 6.93-2.02 8.66-5.02-7.51-.25-12.09-8.43-8.32-14.96z" />
  </svg>
)

const DeleteIcon: React.FC<{ className?: string }> = ({ className }) => (
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
      d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
    />
  </svg>
)

const UnitTypeIcon: React.FC<{ type: UnitType; className?: string }> = ({
  type,
  className,
}) => {
  switch (type) {
    case 'Land':
      return (
        <svg className={className} viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2C8.14 2 5 5.14 5 9c0 5.25 7 13 7 13s7-7.75 7-13c0-3.86-3.14-7-7-7zm0 9.5c-1.38 0-2.5-1.12-2.5-2.5s1.12-2.5 2.5-2.5 2.5 1.12 2.5 2.5-1.12 2.5-2.5 2.5z" />
        </svg>
      )
    case 'Naval':
      return (
        <svg className={className} viewBox="0 0 24 24" fill="currentColor">
          <path d="M20 21c-1.39 0-2.78-.47-4-1.32-2.44 1.71-5.56 1.71-8 0C6.78 20.53 5.39 21 4 21H2v2h2c1.38 0 2.74-.35 4-.99 2.52 1.29 5.48 1.29 8 0 1.26.65 2.62.99 4 .99h2v-2h-2zM3.95 19H4c1.6 0 3.02-.88 4-2 .98 1.12 2.4 2 4 2s3.02-.88 4-2c.98 1.12 2.4 2 4 2h.05l1.89-6.68c.08-.26.06-.54-.06-.78s-.34-.42-.6-.5L20 10.62V6c0-1.1-.9-2-2-2h-3V1H9v3H6c-1.1 0-2 .9-2 2v4.62l-1.29.42c-.26.08-.48.26-.6.5s-.15.52-.06.78L3.95 19z" />
        </svg>
      )
    case 'Air':
      return (
        <svg className={className} viewBox="0 0 24 24" fill="currentColor">
          <path d="M21 16v-2l-8-5V3.5c0-.83-.67-1.5-1.5-1.5S10 2.67 10 3.5V9l-8 5v2l8-2.5V19l-2 1.5V22l3.5-1 3.5 1v-1.5L13 19v-5.5l8 2.5z" />
        </svg>
      )
    case 'Civilian':
      return (
        <svg className={className} viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z" />
        </svg>
      )
  }
}

export default MilitaryOverview
