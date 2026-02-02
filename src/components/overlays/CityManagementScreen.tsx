import React, { useCallback, useMemo, useState } from 'react'

// Types
type ProductionCategory = 'All' | 'Units' | 'Buildings' | 'Wonders'
type CityFocus = 'food' | 'production' | 'gold' | 'science'
type TerrainType =
  | 'plains'
  | 'grassland'
  | 'hills'
  | 'forest'
  | 'mountain'
  | 'water'
  | 'desert'

interface TileYields {
  food: number
  production: number
  gold: number
  science: number
  culture: number
}

interface CityTile {
  id: number
  q: number
  r: number
  terrain: TerrainType
  yields: TileYields
  isWorked: boolean
  hasImprovement: boolean
  improvementName?: string
}

interface ProductionItem {
  id: string
  name: string
  type: 'unit' | 'building' | 'wonder'
  cost: number
  turnsToComplete: number
  description: string
  icon?: string
}

interface QueuedItem extends ProductionItem {
  queuePosition: number
}

interface CityBuilding {
  id: string
  name: string
  effect: string
  maintenance: number
}

interface CityData {
  id: number
  name: string
  population: number
  growthProgress: number
  growthThreshold: number
  turnsToGrow: number
  yields: TileYields
  defense: number
  happiness: number
  currentProduction: ProductionItem | null
  productionProgress: number
  productionQueue: QueuedItem[]
  tiles: CityTile[]
  buildings: CityBuilding[]
  gold: number
}

// Mock data
const MOCK_PRODUCTION_ITEMS: ProductionItem[] = [
  {
    id: 'warrior',
    name: 'Warrior',
    type: 'unit',
    cost: 40,
    turnsToComplete: 4,
    description: 'Basic melee unit',
  },
  {
    id: 'archer',
    name: 'Archer',
    type: 'unit',
    cost: 50,
    turnsToComplete: 5,
    description: 'Basic ranged unit',
  },
  {
    id: 'settler',
    name: 'Settler',
    type: 'unit',
    cost: 80,
    turnsToComplete: 8,
    description: 'Founds new cities',
  },
  {
    id: 'worker',
    name: 'Worker',
    type: 'unit',
    cost: 60,
    turnsToComplete: 6,
    description: 'Improves tiles',
  },
  {
    id: 'scout',
    name: 'Scout',
    type: 'unit',
    cost: 30,
    turnsToComplete: 3,
    description: 'Fast exploration unit',
  },
  {
    id: 'spearman',
    name: 'Spearman',
    type: 'unit',
    cost: 55,
    turnsToComplete: 5,
    description: 'Anti-cavalry unit',
  },
  {
    id: 'granary',
    name: 'Granary',
    type: 'building',
    cost: 60,
    turnsToComplete: 6,
    description: '+2 Food, +1 Food from Wheat',
  },
  {
    id: 'library',
    name: 'Library',
    type: 'building',
    cost: 75,
    turnsToComplete: 7,
    description: '+1 Science per 2 Citizens',
  },
  {
    id: 'barracks',
    name: 'Barracks',
    type: 'building',
    cost: 75,
    turnsToComplete: 7,
    description: '+15 XP for new units',
  },
  {
    id: 'market',
    name: 'Market',
    type: 'building',
    cost: 100,
    turnsToComplete: 10,
    description: '+25% Gold',
  },
  {
    id: 'monument',
    name: 'Monument',
    type: 'building',
    cost: 40,
    turnsToComplete: 4,
    description: '+2 Culture',
  },
  {
    id: 'walls',
    name: 'Walls',
    type: 'building',
    cost: 75,
    turnsToComplete: 7,
    description: '+5 Defense',
  },
  {
    id: 'stonehenge',
    name: 'Stonehenge',
    type: 'wonder',
    cost: 185,
    turnsToComplete: 18,
    description: '+5 Faith, Free Monument',
  },
  {
    id: 'pyramids',
    name: 'Pyramids',
    type: 'wonder',
    cost: 185,
    turnsToComplete: 18,
    description: '+2 Worker charges',
  },
  {
    id: 'great_library',
    name: 'Great Library',
    type: 'wonder',
    cost: 220,
    turnsToComplete: 22,
    description: 'Free Technology',
  },
]

const generateMockTiles = (): CityTile[] => {
  const tiles: CityTile[] = []
  const terrains: TerrainType[] = [
    'plains',
    'grassland',
    'hills',
    'forest',
    'desert',
    'water',
  ]
  let id = 0

  // Generate hex tiles in a radius of 3 around center (0,0)
  for (let q = -3; q <= 3; q++) {
    for (let r = -3; r <= 3; r++) {
      if (Math.abs(q + r) <= 3) {
        const terrain = terrains[Math.floor(Math.random() * terrains.length)]
        const baseYields = getTerrainYields(terrain)
        tiles.push({
          id: id++,
          q,
          r,
          terrain,
          yields: baseYields,
          isWorked: q === 0 && r === 0 ? true : Math.random() > 0.6,
          hasImprovement: Math.random() > 0.7,
          improvementName: Math.random() > 0.5 ? 'Farm' : 'Mine',
        })
      }
    }
  }
  return tiles
}

const getTerrainYields = (terrain: TerrainType): TileYields => {
  const yields: Record<TerrainType, TileYields> = {
    plains: { food: 1, production: 1, gold: 0, science: 0, culture: 0 },
    grassland: { food: 2, production: 0, gold: 0, science: 0, culture: 0 },
    hills: { food: 0, production: 2, gold: 0, science: 0, culture: 0 },
    forest: { food: 1, production: 1, gold: 0, science: 0, culture: 0 },
    mountain: { food: 0, production: 0, gold: 0, science: 0, culture: 0 },
    water: { food: 1, production: 0, gold: 1, science: 0, culture: 0 },
    desert: { food: 0, production: 0, gold: 0, science: 0, culture: 0 },
  }
  return yields[terrain]
}

const MOCK_CITY: CityData = {
  id: 1,
  name: 'New Athens',
  population: 5,
  growthProgress: 12,
  growthThreshold: 24,
  turnsToGrow: 4,
  yields: { food: 12, production: 8, gold: 6, science: 4, culture: 3 },
  defense: 15,
  happiness: 3,
  currentProduction: MOCK_PRODUCTION_ITEMS[6], // Granary
  productionProgress: 24,
  productionQueue: [
    { ...MOCK_PRODUCTION_ITEMS[0], queuePosition: 1 },
    { ...MOCK_PRODUCTION_ITEMS[7], queuePosition: 2 },
  ],
  tiles: generateMockTiles(),
  buildings: [
    { id: 'monument', name: 'Monument', effect: '+2 Culture', maintenance: 0 },
    { id: 'granary', name: 'Granary', effect: '+2 Food', maintenance: 1 },
  ],
  gold: 156,
}

// Props
interface CityManagementScreenProps {
  isOpen: boolean
  onClose: () => void
  cityId: number
}

const CityManagementScreen: React.FC<CityManagementScreenProps> = ({
  isOpen,
  onClose,
  cityId,
}) => {
  // State
  const [cityName, setCityName] = useState(MOCK_CITY.name)
  const [isEditingName, setIsEditingName] = useState(false)
  const [selectedCategory, setSelectedCategory] =
    useState<ProductionCategory>('All')
  const [productionQueue, setProductionQueue] = useState<QueuedItem[]>(
    MOCK_CITY.productionQueue
  )
  const [tiles, setTiles] = useState<CityTile[]>(MOCK_CITY.tiles)
  const [selectedBuilding, setSelectedBuilding] = useState<string | null>(null)
  const [currentFocus, setCurrentFocus] = useState<CityFocus | null>(null)
  const [draggedItem, setDraggedItem] = useState<number | null>(null)

  // Memoized values
  const cityData = useMemo(
    () => ({
      ...MOCK_CITY,
      id: cityId,
      name: cityName,
      productionQueue,
      tiles,
    }),
    [cityId, cityName, productionQueue, tiles]
  )

  const filteredProductionItems = useMemo(() => {
    if (selectedCategory === 'All') return MOCK_PRODUCTION_ITEMS
    return MOCK_PRODUCTION_ITEMS.filter(
      (item) => item.type === selectedCategory.toLowerCase().slice(0, -1)
    )
  }, [selectedCategory])

  const totalYields = useMemo(() => {
    const workedTiles = tiles.filter((t) => t.isWorked)
    return workedTiles.reduce(
      (acc, tile) => ({
        food: acc.food + tile.yields.food,
        production: acc.production + tile.yields.production,
        gold: acc.gold + tile.yields.gold,
        science: acc.science + tile.yields.science,
        culture: acc.culture + tile.yields.culture,
      }),
      { food: 2, production: 2, gold: 0, science: 0, culture: 0 } // Base yields
    )
  }, [tiles])

  // Handlers
  const handleBackdropClick = useCallback(
    (e: React.MouseEvent) => {
      if (e.target === e.currentTarget) {
        onClose()
      }
    },
    [onClose]
  )

  const handleNameSubmit = useCallback(() => {
    setIsEditingName(false)
  }, [])

  const handleAddToQueue = useCallback(
    (item: ProductionItem) => {
      if (productionQueue.length >= 5) return
      const newItem: QueuedItem = {
        ...item,
        queuePosition: productionQueue.length + 1,
      }
      setProductionQueue([...productionQueue, newItem])
    },
    [productionQueue]
  )

  const handleRemoveFromQueue = useCallback((index: number) => {
    setProductionQueue((prev) =>
      prev
        .filter((_, i) => i !== index)
        .map((item, i) => ({
          ...item,
          queuePosition: i + 1,
        }))
    )
  }, [])

  const handleDragStart = useCallback((index: number) => {
    setDraggedItem(index)
  }, [])

  const handleDragOver = useCallback(
    (e: React.DragEvent, index: number) => {
      e.preventDefault()
      if (draggedItem === null || draggedItem === index) return

      setProductionQueue((prev) => {
        const newQueue = [...prev]
        const [removed] = newQueue.splice(draggedItem, 1)
        newQueue.splice(index, 0, removed)
        return newQueue.map((item, i) => ({ ...item, queuePosition: i + 1 }))
      })
      setDraggedItem(index)
    },
    [draggedItem]
  )

  const handleDragEnd = useCallback(() => {
    setDraggedItem(null)
  }, [])

  const handleTileClick = useCallback((tileId: number) => {
    setTiles((prev) =>
      prev.map((tile) =>
        tile.id === tileId ? { ...tile, isWorked: !tile.isWorked } : tile
      )
    )
  }, [])

  const handlePurchase = useCallback(() => {
    if (!cityData.currentProduction) return
    const cost = Math.ceil(
      (cityData.currentProduction.cost - cityData.productionProgress) * 2
    )
    console.log(
      `Purchasing ${cityData.currentProduction.name} for ${cost} gold`
    )
  }, [cityData])

  const handleSellBuilding = useCallback(() => {
    if (!selectedBuilding) return
    console.log(`Selling building: ${selectedBuilding}`)
    setSelectedBuilding(null)
  }, [selectedBuilding])

  const handleFocusChange = useCallback((focus: CityFocus) => {
    setCurrentFocus((prev) => (prev === focus ? null : focus))
  }, [])

  if (!isOpen) return null

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/80"
      onClick={handleBackdropClick}
    >
      <div className="relative flex h-[95vh] w-[98vw] flex-col overflow-hidden rounded-lg border border-primary-700 bg-background-light shadow-2xl">
        {/* Header */}
        <div className="flex items-center justify-between border-b border-primary-700 bg-background px-6 py-4">
          <div className="flex items-center gap-6">
            {/* City Name */}
            {isEditingName ? (
              <input
                type="text"
                value={cityName}
                onChange={(e) => setCityName(e.target.value)}
                onBlur={handleNameSubmit}
                onKeyDown={(e) => e.key === 'Enter' && handleNameSubmit()}
                className="rounded border border-primary-700 bg-background-lighter px-3 py-1 font-header text-2xl text-secondary outline-none focus:border-secondary"
                autoFocus
              />
            ) : (
              <button
                onClick={() => setIsEditingName(true)}
                className="font-header text-2xl text-secondary hover:text-secondary-600"
                title="Click to edit name"
              >
                {cityName}
              </button>
            )}

            {/* Population */}
            <div className="flex items-center gap-2">
              <PopulationIcon className="h-6 w-6 text-green-400" />
              <span className="text-xl font-semibold text-foreground">
                {cityData.population}
              </span>
            </div>

            {/* Growth Progress */}
            <div className="flex items-center gap-3">
              <div className="w-32">
                <div className="mb-1 flex justify-between text-xs text-foreground-muted">
                  <span>Growth</span>
                  <span>{cityData.turnsToGrow} turns</span>
                </div>
                <div className="h-2 overflow-hidden rounded-full bg-background">
                  <div
                    className="h-full bg-green-500 transition-all duration-300"
                    style={{
                      width: `${(cityData.growthProgress / cityData.growthThreshold) * 100}%`,
                    }}
                  />
                </div>
              </div>
              <span className="text-sm text-foreground-muted">
                {cityData.growthProgress}/{cityData.growthThreshold}
              </span>
            </div>
          </div>

          {/* Close Button */}
          <button
            onClick={onClose}
            className="rounded p-2 transition-colors hover:bg-background-lighter"
            title="Close"
          >
            <CloseIcon className="h-6 w-6 text-foreground" />
          </button>
        </div>

        {/* Main Content */}
        <div className="flex flex-1 overflow-hidden">
          {/* Left Panel - Production */}
          <div className="flex w-80 flex-col border-r border-primary-700 bg-background">
            {/* Current Production */}
            <div className="border-b border-primary-700 p-4">
              <h3 className="mb-3 text-sm font-semibold uppercase text-foreground-muted">
                Current Production
              </h3>
              {cityData.currentProduction ? (
                <div className="rounded border border-primary-700 bg-background-lighter p-3">
                  <div className="mb-2 flex items-center justify-between">
                    <span className="font-semibold text-foreground">
                      {cityData.currentProduction.name}
                    </span>
                    <span className="rounded bg-primary-700 px-2 py-0.5 text-xs text-foreground-muted">
                      {cityData.currentProduction.type}
                    </span>
                  </div>
                  <div className="mb-1 flex justify-between text-xs text-foreground-muted">
                    <span>
                      {cityData.productionProgress}/
                      {cityData.currentProduction.cost}
                    </span>
                    <span>
                      {Math.ceil(
                        (cityData.currentProduction.cost -
                          cityData.productionProgress) /
                          totalYields.production
                      )}{' '}
                      turns
                    </span>
                  </div>
                  <div className="h-2 overflow-hidden rounded-full bg-background">
                    <div
                      className="h-full bg-orange-500 transition-all duration-300"
                      style={{
                        width: `${(cityData.productionProgress / cityData.currentProduction.cost) * 100}%`,
                      }}
                    />
                  </div>
                </div>
              ) : (
                <div className="rounded border border-dashed border-primary-700 p-3 text-center text-foreground-muted">
                  No production selected
                </div>
              )}
            </div>

            {/* Production Queue */}
            <div className="border-b border-primary-700 p-4">
              <h3 className="mb-3 text-sm font-semibold uppercase text-foreground-muted">
                Production Queue ({productionQueue.length}/5)
              </h3>
              <div className="flex flex-col gap-2">
                {productionQueue.map((item, index) => (
                  <div
                    key={`${item.id}-${index}`}
                    draggable
                    onDragStart={() => handleDragStart(index)}
                    onDragOver={(e) => handleDragOver(e, index)}
                    onDragEnd={handleDragEnd}
                    className={`flex cursor-move items-center justify-between rounded border border-primary-700 bg-background-lighter p-2 transition-colors hover:border-secondary ${
                      draggedItem === index ? 'opacity-50' : ''
                    }`}
                  >
                    <div className="flex items-center gap-2">
                      <span className="text-xs text-foreground-muted">
                        {index + 1}.
                      </span>
                      <span className="text-sm text-foreground">
                        {item.name}
                      </span>
                    </div>
                    <button
                      onClick={() => handleRemoveFromQueue(index)}
                      className="text-red-400 hover:text-red-300"
                    >
                      <CloseIcon className="h-4 w-4" />
                    </button>
                  </div>
                ))}
                {productionQueue.length === 0 && (
                  <div className="text-center text-sm text-foreground-muted">
                    Queue empty
                  </div>
                )}
              </div>
            </div>

            {/* Category Filters */}
            <div className="border-b border-primary-700 p-4">
              <div className="flex gap-2">
                {(
                  [
                    'All',
                    'Units',
                    'Buildings',
                    'Wonders',
                  ] as ProductionCategory[]
                ).map((category) => (
                  <button
                    key={category}
                    onClick={() => setSelectedCategory(category)}
                    className={`rounded px-3 py-1 text-xs font-medium transition-colors ${
                      selectedCategory === category
                        ? 'bg-secondary text-background'
                        : 'bg-background-lighter text-foreground-muted hover:text-foreground'
                    }`}
                  >
                    {category}
                  </button>
                ))}
              </div>
            </div>

            {/* Available Production Items */}
            <div className="flex-1 overflow-y-auto p-4">
              <h3 className="mb-3 text-sm font-semibold uppercase text-foreground-muted">
                Available to Build
              </h3>
              <div className="flex flex-col gap-2">
                {filteredProductionItems.map((item) => (
                  <button
                    key={item.id}
                    onClick={() => handleAddToQueue(item)}
                    disabled={productionQueue.length >= 5}
                    className="flex flex-col rounded border border-primary-700 bg-background-lighter p-3 text-left transition-colors hover:border-secondary disabled:cursor-not-allowed disabled:opacity-50"
                  >
                    <div className="mb-1 flex items-center justify-between">
                      <span className="font-semibold text-foreground">
                        {item.name}
                      </span>
                      <span className="rounded bg-primary-700 px-2 py-0.5 text-xs text-foreground-muted">
                        {item.type}
                      </span>
                    </div>
                    <div className="mb-1 flex items-center gap-3 text-xs text-foreground-muted">
                      <span className="flex items-center gap-1">
                        <ProductionIcon className="h-3 w-3 text-orange-400" />
                        {item.cost}
                      </span>
                      <span>{item.turnsToComplete} turns</span>
                    </div>
                    <p className="text-xs text-foreground-muted">
                      {item.description}
                    </p>
                  </button>
                ))}
              </div>
            </div>
          </div>

          {/* Center Panel - City Tiles */}
          <div className="flex flex-1 flex-col items-center justify-center bg-background-lighter p-4">
            <h3 className="mb-4 text-sm font-semibold uppercase text-foreground-muted">
              City Tiles
            </h3>
            <div className="relative">
              {/* Hex Grid */}
              <svg viewBox="-250 -250 500 500" className="h-[500px] w-[500px]">
                {tiles.map((tile) => {
                  const x = tile.q * 45 + tile.r * 22.5
                  const y = tile.r * 39
                  const isCenter = tile.q === 0 && tile.r === 0
                  return (
                    <g
                      key={tile.id}
                      onClick={() => !isCenter && handleTileClick(tile.id)}
                    >
                      <HexTile
                        x={x}
                        y={y}
                        terrain={tile.terrain}
                        isWorked={tile.isWorked}
                        isCenter={isCenter}
                      />
                      {/* Yields */}
                      <g transform={`translate(${x}, ${y})`}>
                        {tile.yields.food > 0 && (
                          <text
                            x="-12"
                            y="-5"
                            fontSize="10"
                            fill="#4ade80"
                            textAnchor="middle"
                          >
                            {tile.yields.food}F
                          </text>
                        )}
                        {tile.yields.production > 0 && (
                          <text
                            x="12"
                            y="-5"
                            fontSize="10"
                            fill="#fb923c"
                            textAnchor="middle"
                          >
                            {tile.yields.production}P
                          </text>
                        )}
                        {tile.yields.gold > 0 && (
                          <text
                            x="0"
                            y="10"
                            fontSize="10"
                            fill="#fbbf24"
                            textAnchor="middle"
                          >
                            {tile.yields.gold}G
                          </text>
                        )}
                        {isCenter && (
                          <text
                            x="0"
                            y="3"
                            fontSize="12"
                            fill="#f9fafb"
                            textAnchor="middle"
                            fontWeight="bold"
                          >
                            City
                          </text>
                        )}
                      </g>
                    </g>
                  )
                })}
              </svg>
            </div>
            <div className="mt-4 flex gap-4 text-sm">
              <div className="flex items-center gap-2">
                <div className="h-4 w-4 rounded bg-green-500/50" />
                <span className="text-foreground-muted">Worked</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="h-4 w-4 rounded border border-primary-700 bg-background" />
                <span className="text-foreground-muted">Unworked</span>
              </div>
            </div>
          </div>

          {/* Right Panel - City Info */}
          <div className="flex w-72 flex-col border-l border-primary-700 bg-background">
            {/* Yields Summary */}
            <div className="border-b border-primary-700 p-4">
              <h3 className="mb-3 text-sm font-semibold uppercase text-foreground-muted">
                City Yields
              </h3>
              <div className="flex flex-col gap-2">
                <YieldRow
                  icon={<FoodIcon className="h-5 w-5 text-green-400" />}
                  label="Food"
                  value={totalYields.food}
                  surplus={totalYields.food - cityData.population * 2}
                />
                <YieldRow
                  icon={<ProductionIcon className="h-5 w-5 text-orange-400" />}
                  label="Production"
                  value={totalYields.production}
                />
                <YieldRow
                  icon={<GoldIcon className="h-5 w-5 text-yellow-400" />}
                  label="Gold"
                  value={totalYields.gold}
                />
                <YieldRow
                  icon={<ScienceIcon className="h-5 w-5 text-blue-400" />}
                  label="Science"
                  value={totalYields.science}
                />
                <YieldRow
                  icon={<CultureIcon className="h-5 w-5 text-purple-400" />}
                  label="Culture"
                  value={totalYields.culture}
                />
              </div>
            </div>

            {/* Buildings */}
            <div className="flex-1 overflow-y-auto border-b border-primary-700 p-4">
              <h3 className="mb-3 text-sm font-semibold uppercase text-foreground-muted">
                Buildings ({cityData.buildings.length})
              </h3>
              <div className="flex flex-col gap-2">
                {cityData.buildings.map((building) => (
                  <button
                    key={building.id}
                    onClick={() =>
                      setSelectedBuilding(
                        selectedBuilding === building.id ? null : building.id
                      )
                    }
                    className={`rounded border p-2 text-left transition-colors ${
                      selectedBuilding === building.id
                        ? 'border-secondary bg-secondary/10'
                        : 'border-primary-700 bg-background-lighter hover:border-secondary'
                    }`}
                  >
                    <div className="flex items-center justify-between">
                      <span className="font-medium text-foreground">
                        {building.name}
                      </span>
                      {building.maintenance > 0 && (
                        <span className="text-xs text-red-400">
                          -{building.maintenance}g
                        </span>
                      )}
                    </div>
                    <p className="text-xs text-foreground-muted">
                      {building.effect}
                    </p>
                  </button>
                ))}
                {cityData.buildings.length === 0 && (
                  <p className="text-center text-sm text-foreground-muted">
                    No buildings constructed
                  </p>
                )}
              </div>
            </div>

            {/* City Stats */}
            <div className="p-4">
              <h3 className="mb-3 text-sm font-semibold uppercase text-foreground-muted">
                City Stats
              </h3>
              <div className="flex flex-col gap-2">
                <div className="flex items-center justify-between">
                  <span className="flex items-center gap-2 text-foreground-muted">
                    <DefenseIcon className="h-4 w-4" />
                    Defense
                  </span>
                  <span className="font-medium text-foreground">
                    {cityData.defense}
                  </span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="flex items-center gap-2 text-foreground-muted">
                    <HappinessIcon className="h-4 w-4 text-green-400" />
                    Happiness
                  </span>
                  <span
                    className={`font-medium ${cityData.happiness >= 0 ? 'text-green-400' : 'text-red-400'}`}
                  >
                    {cityData.happiness >= 0 ? '+' : ''}
                    {cityData.happiness}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Bottom Bar */}
        <div className="flex items-center justify-between border-t border-primary-700 bg-background px-6 py-3">
          <div className="flex gap-3">
            {/* Purchase Button */}
            <button
              onClick={handlePurchase}
              disabled={!cityData.currentProduction}
              className="rounded border border-yellow-600 bg-yellow-900/50 px-4 py-2 text-yellow-400 transition-colors hover:bg-yellow-900 disabled:cursor-not-allowed disabled:opacity-50"
            >
              <span className="flex items-center gap-2">
                <GoldIcon className="h-4 w-4" />
                Purchase (
                {cityData.currentProduction
                  ? Math.ceil(
                      (cityData.currentProduction.cost -
                        cityData.productionProgress) *
                        2
                    )
                  : 0}
                g)
              </span>
            </button>

            {/* Sell Building Button */}
            <button
              onClick={handleSellBuilding}
              disabled={!selectedBuilding}
              className="rounded border border-red-600 bg-red-900/50 px-4 py-2 text-red-400 transition-colors hover:bg-red-900 disabled:cursor-not-allowed disabled:opacity-50"
            >
              Sell Building
            </button>
          </div>

          {/* Focus Buttons */}
          <div className="flex items-center gap-2">
            <span className="mr-2 text-sm text-foreground-muted">Focus:</span>
            {(['food', 'production', 'gold', 'science'] as CityFocus[]).map(
              (focus) => (
                <button
                  key={focus}
                  onClick={() => handleFocusChange(focus)}
                  className={`rounded p-2 transition-colors ${
                    currentFocus === focus
                      ? 'bg-secondary text-background'
                      : 'bg-background-lighter text-foreground-muted hover:text-foreground'
                  }`}
                  title={`Focus on ${focus}`}
                >
                  {focus === 'food' && <FoodIcon className="h-5 w-5" />}
                  {focus === 'production' && (
                    <ProductionIcon className="h-5 w-5" />
                  )}
                  {focus === 'gold' && <GoldIcon className="h-5 w-5" />}
                  {focus === 'science' && <ScienceIcon className="h-5 w-5" />}
                </button>
              )
            )}
          </div>
        </div>
      </div>
    </div>
  )
}

// Hex Tile Component
interface HexTileProps {
  x: number
  y: number
  terrain: TerrainType
  isWorked: boolean
  isCenter: boolean
}

const HexTile: React.FC<HexTileProps> = ({
  x,
  y,
  terrain,
  isWorked,
  isCenter,
}) => {
  const terrainColors: Record<TerrainType, string> = {
    plains: '#d4a574',
    grassland: '#4ade80',
    hills: '#a8a29e',
    forest: '#166534',
    mountain: '#78716c',
    water: '#3b82f6',
    desert: '#fcd34d',
  }

  const hexPath = 'M0,-25 L22,-12.5 L22,12.5 L0,25 L-22,12.5 L-22,-12.5 Z'

  return (
    <g transform={`translate(${x}, ${y})`}>
      <path
        d={hexPath}
        fill={terrainColors[terrain]}
        stroke={isCenter ? '#f9fafb' : isWorked ? '#4ade80' : '#374151'}
        strokeWidth={isCenter ? 3 : isWorked ? 2 : 1}
        opacity={isWorked || isCenter ? 1 : 0.6}
        className={!isCenter ? 'cursor-pointer hover:opacity-100' : ''}
      />
    </g>
  )
}

// Yield Row Component
interface YieldRowProps {
  icon: React.ReactNode
  label: string
  value: number
  surplus?: number
}

const YieldRow: React.FC<YieldRowProps> = ({ icon, label, value, surplus }) => (
  <div className="flex items-center justify-between">
    <span className="flex items-center gap-2 text-foreground-muted">
      {icon}
      {label}
    </span>
    <div className="flex items-center gap-2">
      <span className="font-medium text-foreground">{value}</span>
      {surplus !== undefined && (
        <span
          className={`text-xs ${surplus >= 0 ? 'text-green-400' : 'text-red-400'}`}
        >
          ({surplus >= 0 ? '+' : ''}
          {surplus})
        </span>
      )}
    </div>
  </div>
)

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

const PopulationIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z" />
  </svg>
)

const FoodIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M18.06 22.99h1.66c.84 0 1.53-.64 1.63-1.46L23 5.05l-5 2.3V20c0 1.66 1.34 3 3.06 2.99zM1 21.99h15.03c.54 0 .99-.45.99-.99v-1H1v2zM1 13v3c0 .55.45 1 1 1h13c.55 0 1-.45 1-1v-3H1zm7-6L4.5 7H1v5h15V7h-4.51L8 7z" />
  </svg>
)

const ProductionIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M22 9V7h-2V5c0-1.1-.9-2-2-2H4c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2v-2h2v-2h-2v-2h2v-2h-2V9h2zm-4 10H4V5h14v14zM6 13h5v4H6zm6-6h4v3h-4zM6 7h5v5H6zm6 4h4v6h-4z" />
  </svg>
)

const GoldIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <circle cx="12" cy="12" r="10" />
  </svg>
)

const ScienceIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M13 3v4h7l-8 12-8-12h7V3h2zm-2 14h2v2h-2v-2z" />
  </svg>
)

const CultureIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M12 3L1 9l4 2.18v6L12 21l7-3.82v-6l2-1.09V17h2V9L12 3zm6.82 6L12 12.72 5.18 9 12 5.28 18.82 9zM17 15.99l-5 2.73-5-2.73v-3.72L12 15l5-2.73v3.72z" />
  </svg>
)

const DefenseIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4zm0 10.99h7c-.53 4.12-3.28 7.79-7 8.94V12H5V6.3l7-3.11v8.8z" />
  </svg>
)

const HappinessIcon: React.FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M11.99 2C6.47 2 2 6.48 2 12s4.47 10 9.99 10C17.52 22 22 17.52 22 12S17.52 2 11.99 2zM12 20c-4.42 0-8-3.58-8-8s3.58-8 8-8 8 3.58 8 8-3.58 8-8 8zm3.5-9c.83 0 1.5-.67 1.5-1.5S16.33 8 15.5 8 14 8.67 14 9.5s.67 1.5 1.5 1.5zm-7 0c.83 0 1.5-.67 1.5-1.5S9.33 8 8.5 8 7 8.67 7 9.5 7.67 11 8.5 11zm3.5 6.5c2.33 0 4.31-1.46 5.11-3.5H6.89c.8 2.04 2.78 3.5 5.11 3.5z" />
  </svg>
)

export default CityManagementScreen
