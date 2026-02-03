import React from 'react'
import {
  useGameStore,
  selectSelectedUnit,
  selectSelectedCity,
  selectSelectedTile,
} from '@/stores/gameStore'
import type { Unit, City, HexTile } from '@/types/game'

const SelectionPanel: React.FC = () => {
  const selection = useGameStore((state) => state.selection)
  const selectedUnit = useGameStore(selectSelectedUnit)
  const selectedCity = useGameStore(selectSelectedCity)
  const selectedTile = useGameStore(selectSelectedTile)

  if (selection.type === 'none') {
    return null
  }

  return (
    <div className="panel w-64 animate-fade-in">
      {selection.type === 'unit' && selectedUnit && (
        <UnitPanel unit={selectedUnit} />
      )}
      {selection.type === 'city' && selectedCity && (
        <CityPanel city={selectedCity} />
      )}
      {selection.type === 'tile' &&
        selectedTile &&
        !selectedUnit &&
        !selectedCity && <TilePanel tile={selectedTile} />}
    </div>
  )
}

interface UnitPanelProps {
  unit: Unit
}

const UnitPanel: React.FC<UnitPanelProps> = ({ unit }) => {
  const foundCity = useGameStore((state) => state.foundCity)

  const healthPercent = (unit.health / unit.maxHealth) * 100
  const healthColor =
    healthPercent > 66
      ? 'bg-success'
      : healthPercent > 33
        ? 'bg-warning'
        : 'bg-danger'

  return (
    <>
      <div className="panel-header uppercase tracking-wider">{unit.type}</div>
      <div className="panel-content space-y-3">
        {/* Health bar */}
        <div>
          <div className="mb-1 flex justify-between text-sm">
            <span className="text-foreground-muted">HP</span>
            <span className="font-mono">
              {unit.health}/{unit.maxHealth}
            </span>
          </div>
          <div className="h-2 overflow-hidden rounded bg-background">
            <div
              className={`h-full ${healthColor} transition-all duration-300`}
              style={{ width: `${healthPercent}%` }}
            />
          </div>
        </div>

        {/* Stats */}
        <div className="grid grid-cols-2 gap-2 text-sm">
          <StatItem
            label="Movement"
            value={`${unit.movement}/${unit.maxMovement}`}
          />
          <StatItem label="Strength" value={unit.strength.toString()} />
        </div>

        {/* Promotions */}
        {unit.promotions.length > 0 && (
          <div>
            <span className="text-sm text-foreground-muted">Promotions:</span>
            <ul className="ml-2 text-sm">
              {unit.promotions.map((promo) => (
                <li key={promo} className="text-foreground-dim">
                  {promo}
                </li>
              ))}
            </ul>
          </div>
        )}

        {/* Actions */}
        <div className="flex flex-wrap gap-2 border-t border-primary-700 pt-2">
          {unit.type === 'settler' && (
            <ActionButton onClick={() => foundCity(unit.id)}>
              Found City
            </ActionButton>
          )}
          <ActionButton onClick={() => console.log('Fortify')}>
            Fortify
          </ActionButton>
          <ActionButton onClick={() => console.log('Skip')}>Skip</ActionButton>
        </div>
      </div>
    </>
  )
}

interface CityPanelProps {
  city: City
}

const CityPanel: React.FC<CityPanelProps> = ({ city }) => {
  const productionPercent =
    city.production.total > 0
      ? (city.production.progress / city.production.total) * 100
      : 0

  return (
    <>
      <div className="panel-header uppercase tracking-wider">
        {city.name}
        {city.isCapital && (
          <span className="ml-2 text-xs text-secondary">(Capital)</span>
        )}
      </div>
      <div className="panel-content space-y-3">
        {/* Population */}
        <div className="text-center">
          <span className="font-header text-3xl text-secondary">
            {city.population}
          </span>
          <p className="text-sm text-foreground-muted">Population</p>
        </div>

        {/* Production */}
        <div>
          <div className="mb-1 flex justify-between text-sm">
            <span className="text-foreground-muted">Producing</span>
            <span>{city.production.item ?? 'Nothing'}</span>
          </div>
          {city.production.item && (
            <>
              <div className="h-2 overflow-hidden rounded bg-background">
                <div
                  className="h-full bg-secondary transition-all duration-300"
                  style={{ width: `${productionPercent}%` }}
                />
              </div>
              <p className="mt-1 text-right text-xs text-foreground-dim">
                {city.production.turnsRemaining} turns
              </p>
            </>
          )}
        </div>

        {/* Yields */}
        <div className="grid grid-cols-2 gap-2 text-sm">
          <StatItem label="Food" value={city.yields.food.toString()} />
          <StatItem
            label="Production"
            value={city.yields.production.toString()}
          />
          <StatItem label="Gold" value={city.yields.gold.toString()} />
          <StatItem label="Science" value={city.yields.science.toString()} />
        </div>

        {/* Actions */}
        <div className="flex flex-wrap gap-2 border-t border-primary-700 pt-2">
          <ActionButton onClick={() => console.log('Change Production')}>
            Production
          </ActionButton>
          <ActionButton onClick={() => console.log('Manage Citizens')}>
            Citizens
          </ActionButton>
        </div>
      </div>
    </>
  )
}

interface TilePanelProps {
  tile: HexTile
}

const TilePanel: React.FC<TilePanelProps> = ({ tile }) => {
  return (
    <>
      <div className="panel-header uppercase tracking-wider">
        {tile.terrain}
        {tile.features.length > 0 && (
          <span className="ml-1 text-foreground-dim">
            ({tile.features.join(', ')})
          </span>
        )}
      </div>
      <div className="panel-content space-y-3">
        {/* Yields would go here based on terrain */}
        <div className="text-sm text-foreground-muted">
          <p>Yields:</p>
          <ul className="ml-2">
            <li>Food: {tile.terrain === 'grassland' ? 2 : 1}</li>
            <li>Production: {tile.features.includes('hills') ? 2 : 0}</li>
          </ul>
        </div>

        {/* Resource */}
        {tile.resource && (
          <div className="text-sm">
            <span className="text-foreground-muted">Resource: </span>
            <span className="text-secondary">{tile.resource.type}</span>
          </div>
        )}

        {/* Improvement */}
        {tile.improvement && (
          <div className="text-sm">
            <span className="text-foreground-muted">Improvement: </span>
            <span>{tile.improvement}</span>
          </div>
        )}

        {/* Coordinates */}
        <div className="text-xs text-foreground-dim">
          Coordinates: ({tile.coord.q}, {tile.coord.r})
        </div>
      </div>
    </>
  )
}

interface StatItemProps {
  label: string
  value: string
}

const StatItem: React.FC<StatItemProps> = ({ label, value }) => (
  <div className="flex justify-between">
    <span className="text-foreground-muted">{label}</span>
    <span className="font-mono">{value}</span>
  </div>
)

interface ActionButtonProps {
  children: React.ReactNode
  onClick: () => void
}

const ActionButton: React.FC<ActionButtonProps> = ({ children, onClick }) => (
  <button
    onClick={onClick}
    className="rounded bg-primary px-3 py-1 text-sm transition-colors duration-200 hover:bg-primary-600"
  >
    {children}
  </button>
)

export default SelectionPanel
