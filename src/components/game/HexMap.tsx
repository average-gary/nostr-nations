import React, { useMemo, useRef, useCallback } from 'react'
import { useThree } from '@react-three/fiber'
import * as THREE from 'three'
import { useGameStore } from '@/stores/gameStore'
import type { HexTile, Unit, City, HexCoord } from '@/types/game'
import HexTileMesh from './HexTileMesh'
import UnitMesh from './UnitMesh'
import CityMesh from './CityMesh'
import FogOfWar from './FogOfWar'

interface HexMapProps {
  tiles: HexTile[]
  units: Unit[]
  cities: City[]
}

// Hex layout constants
const HEX_SIZE = 1

// Convert axial coordinates to world position
export function axialToWorld(coord: HexCoord): [number, number, number] {
  const x = HEX_SIZE * (3 / 2) * coord.q
  const z = HEX_SIZE * Math.sqrt(3) * (coord.r + coord.q / 2)
  return [x, 0, z]
}

/**
 * OPTIMIZATION: Memoized position cache to avoid recalculating world positions
 * Uses a WeakMap-style key pattern for efficient lookup
 */
const positionCache = new Map<string, [number, number, number]>()

function getCachedPosition(coord: HexCoord): [number, number, number] {
  const key = `${coord.q},${coord.r}`
  let pos = positionCache.get(key)
  if (!pos) {
    pos = axialToWorld(coord)
    positionCache.set(key, pos)
  }
  return pos
}

/**
 * OPTIMIZATION: Split store selectors for granular updates
 * Each selector only triggers re-renders when its specific value changes
 */
const useSelectTile = () => useGameStore((state) => state.selectTile)
const useSelectUnit = () => useGameStore((state) => state.selectUnit)
const useSelectCity = () => useGameStore((state) => state.selectCity)
const useSelectionCoord = () => useGameStore((state) => state.selection.coord)
const useSelectionType = () => useGameStore((state) => state.selection.type)
const useSelectionId = () => useGameStore((state) => state.selection.id)

/**
 * HexMap - Main map component with optimizations:
 * - Memoized position calculations
 * - useCallback for event handlers
 * - Frustum culling for viewport optimization
 * - Granular store selectors to minimize re-renders
 */
const HexMap: React.FC<HexMapProps> = ({ tiles, units, cities }) => {
  const groupRef = useRef<THREE.Group>(null)

  // OPTIMIZATION: Use split selectors to avoid re-renders from unrelated state changes
  const selectTile = useSelectTile()
  const selectUnit = useSelectUnit()
  const selectCity = useSelectCity()
  const selectionCoord = useSelectionCoord()
  const selectionType = useSelectionType()
  const selectionId = useSelectionId()

  // Get camera for frustum culling
  const { camera } = useThree()

  // OPTIMIZATION: Create lookup maps for O(1) access by position
  const unitsByPosition = useMemo(() => {
    const map = new Map<string, Unit>()
    units.forEach((unit) => {
      const key = `${unit.position.q},${unit.position.r}`
      map.set(key, unit)
    })
    return map
  }, [units])

  const citiesByPosition = useMemo(() => {
    const map = new Map<string, City>()
    cities.forEach((city) => {
      const key = `${city.position.q},${city.position.r}`
      map.set(key, city)
    })
    return map
  }, [cities])

  // OPTIMIZATION: Create tile lookup map for O(1) visibility checks
  const tilesByPosition = useMemo(() => {
    const map = new Map<string, HexTile>()
    tiles.forEach((tile) => {
      const key = `${tile.coord.q},${tile.coord.r}`
      map.set(key, tile)
    })
    return map
  }, [tiles])

  // OPTIMIZATION: useCallback to prevent recreating handler on every render
  const handleTileClick = useCallback(
    (coord: HexCoord) => {
      const key = `${coord.q},${coord.r}`

      // Check if there's a unit at this position
      const unit = unitsByPosition.get(key)
      if (unit) {
        selectUnit(unit.id)
        return
      }

      // Check if there's a city at this position
      const city = citiesByPosition.get(key)
      if (city) {
        selectCity(city.id)
        return
      }

      // Otherwise select the tile
      selectTile(coord)
    },
    [unitsByPosition, citiesByPosition, selectUnit, selectCity, selectTile]
  )

  // OPTIMIZATION: Memoize selection check to avoid recalculating for each tile
  const selectedTileKey = useMemo(() => {
    return selectionCoord ? `${selectionCoord.q},${selectionCoord.r}` : null
  }, [selectionCoord])

  // OPTIMIZATION: Frustum culling - only render tiles within camera view
  // This uses a simple bounding box check based on camera frustum
  const visibleTiles = useMemo(() => {
    // Create frustum from camera
    const frustum = new THREE.Frustum()
    const projScreenMatrix = new THREE.Matrix4()
    projScreenMatrix.multiplyMatrices(
      camera.projectionMatrix,
      camera.matrixWorldInverse
    )
    frustum.setFromProjectionMatrix(projScreenMatrix)

    // Filter tiles that are within the frustum (with padding for smooth loading)
    const padding = 5 // Extra tiles around viewport
    return tiles.filter((tile) => {
      const [x, , z] = getCachedPosition(tile.coord)
      // Simple sphere check for performance (actual hex is contained within)
      const point = new THREE.Vector3(x, 0, z)
      const sphere = new THREE.Sphere(point, HEX_SIZE * padding)
      return frustum.intersectsSphere(sphere)
    })
  }, [tiles, camera.projectionMatrix, camera.matrixWorldInverse])

  // OPTIMIZATION: Filter visible units using memoized tile lookup
  const visibleUnits = useMemo(() => {
    return units.filter((unit) => {
      const key = `${unit.position.q},${unit.position.r}`
      const tile = tilesByPosition.get(key)
      return tile?.visibility === 'visible'
    })
  }, [units, tilesByPosition])

  // OPTIMIZATION: Filter visible cities using memoized tile lookup
  const visibleCities = useMemo(() => {
    return cities.filter((city) => {
      const key = `${city.position.q},${city.position.r}`
      const tile = tilesByPosition.get(key)
      return tile && tile.visibility !== 'hidden'
    })
  }, [cities, tilesByPosition])

  // OPTIMIZATION: Memoize city dimmed state lookup
  const getCityDimmed = useCallback(
    (cityPosition: HexCoord): boolean => {
      const key = `${cityPosition.q},${cityPosition.r}`
      const tile = tilesByPosition.get(key)
      return tile?.visibility === 'explored'
    },
    [tilesByPosition]
  )

  // OPTIMIZATION: useCallback for city/unit click handlers
  const handleCityClick = useCallback(
    (cityId: string) => {
      selectCity(cityId)
    },
    [selectCity]
  )

  const handleUnitClick = useCallback(
    (unitId: string) => {
      selectUnit(unitId)
    },
    [selectUnit]
  )

  return (
    <group ref={groupRef}>
      {/* Render hex tiles - OPTIMIZATION: only renders tiles in viewport */}
      {visibleTiles.map((tile) => {
        const key = `${tile.coord.q},${tile.coord.r}`
        return (
          <HexTileMesh
            key={`tile-${key}`}
            tile={tile}
            position={getCachedPosition(tile.coord)}
            isSelected={key === selectedTileKey}
            onClick={handleTileClick}
          />
        )
      })}

      {/* Render cities (only on visible/explored tiles) */}
      {visibleCities.map((city) => (
        <CityMesh
          key={`city-${city.id}`}
          city={city}
          position={getCachedPosition(city.position)}
          isSelected={selectionType === 'city' && selectionId === city.id}
          onClick={handleCityClick}
          dimmed={getCityDimmed(city.position)}
        />
      ))}

      {/* Render units (only on visible tiles - never show on explored) */}
      {visibleUnits.map((unit) => (
        <UnitMesh
          key={`unit-${unit.id}`}
          unit={unit}
          position={getCachedPosition(unit.position)}
          isSelected={selectionType === 'unit' && selectionId === unit.id}
          onClick={handleUnitClick}
        />
      ))}

      {/* Fog of war overlay - rendered last for proper blending */}
      <FogOfWar tiles={tiles} fogIntensity={0.95} animated={true} />
    </group>
  )
}

// OPTIMIZATION: React.memo to prevent re-renders when parent re-renders with same props
export default React.memo(HexMap)
