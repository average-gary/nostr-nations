import React, { useRef, useMemo, useEffect } from 'react'
import { useFrame } from '@react-three/fiber'
import * as THREE from 'three'
import type { HexTile, TileVisibility, HexCoord } from '@/types/game'
import {
  fogOfWarVertexShader,
  fogOfWarFragmentShader,
  defaultFogUniforms,
  FogOfWarUniforms,
} from '@/shaders/fogOfWar'

// Hex layout constants (must match HexMap)
const HEX_SIZE = 1
const FOG_HEIGHT = 0.01 // Slightly above hex tiles

interface FogOfWarProps {
  tiles: HexTile[]
  /** Optional custom fog intensity (0-1) */
  fogIntensity?: number
  /** Whether to animate fog edges */
  animated?: boolean
}

/**
 * OPTIMIZATION: Pre-computed visibility values
 * Avoids switch statement overhead on every tile
 */
const VISIBILITY_VALUES: Record<TileVisibility, number> = {
  hidden: 0.0,
  explored: 0.5,
  visible: 1.0,
}

function visibilityToFloat(visibility: TileVisibility): number {
  return VISIBILITY_VALUES[visibility] ?? 1.0
}

/**
 * OPTIMIZATION: Cached position calculations
 */
const positionCache = new Map<string, [number, number, number]>()

function axialToWorld(coord: HexCoord): [number, number, number] {
  const key = `${coord.q},${coord.r}`
  let pos = positionCache.get(key)
  if (!pos) {
    const x = HEX_SIZE * (3 / 2) * coord.q
    const z = HEX_SIZE * Math.sqrt(3) * (coord.r + coord.q / 2)
    pos = [x, FOG_HEIGHT, z]
    positionCache.set(key, pos)
  }
  return pos
}

/**
 * OPTIMIZATION: Shared hex fog geometry
 * Created once and reused
 */
let sharedHexFogGeometry: THREE.BufferGeometry | null = null

function getSharedHexFogGeometry(): THREE.BufferGeometry {
  if (!sharedHexFogGeometry) {
    const shape = new THREE.Shape()
    const corners: [number, number][] = []

    // Generate hex corner points (pointy-top orientation)
    for (let i = 0; i < 6; i++) {
      const angle = (Math.PI / 3) * i - Math.PI / 6
      // Slightly larger than actual hex for overlap/no gaps
      const x = HEX_SIZE * 1.02 * Math.cos(angle)
      const y = HEX_SIZE * 1.02 * Math.sin(angle)
      corners.push([x, y])
    }

    // Create shape from corners
    shape.moveTo(corners[0][0], corners[0][1])
    for (let i = 1; i < 6; i++) {
      shape.lineTo(corners[i][0], corners[i][1])
    }
    shape.closePath()

    // Create flat geometry from shape
    sharedHexFogGeometry = new THREE.ShapeGeometry(shape)

    // Rotate to lie flat on XZ plane
    sharedHexFogGeometry.rotateX(-Math.PI / 2)
  }
  return sharedHexFogGeometry
}

/**
 * OPTIMIZATION: Reusable typed arrays for instance data
 * Pre-allocate based on expected max tiles to avoid GC pressure
 */
let positionsArray: Float32Array | null = null
let visibilitiesArray: Float32Array | null = null
let lastTileCount = 0

function getInstanceArrays(tileCount: number): {
  positions: Float32Array
  visibilities: Float32Array
} {
  // Only reallocate if tile count increases significantly
  if (!positionsArray || !visibilitiesArray || tileCount > lastTileCount) {
    // Allocate with some buffer to avoid frequent reallocations
    const allocSize = Math.ceil(tileCount * 1.5)
    positionsArray = new Float32Array(allocSize * 3)
    visibilitiesArray = new Float32Array(allocSize)
    lastTileCount = allocSize
  }
  return { positions: positionsArray, visibilities: visibilitiesArray }
}

/**
 * FogOfWar component with optimizations:
 * - Shared geometry instance
 * - Reusable typed arrays to reduce GC pressure
 * - Memoized shader material
 * - Efficient visibility updates (only update changed tiles)
 * - Early bailout when no fog needed
 * - React.memo for shallow comparison
 */
const FogOfWar: React.FC<FogOfWarProps> = ({
  tiles,
  fogIntensity = 1.0,
  animated = true,
}) => {
  const meshRef = useRef<THREE.InstancedMesh>(null)
  const materialRef = useRef<THREE.ShaderMaterial>(null)

  // OPTIMIZATION: Track previous visibility state to only update changed tiles
  const prevVisibilitiesRef = useRef<Map<string, TileVisibility>>(new Map())

  // OPTIMIZATION: Use shared hex geometry
  const hexGeometry = useMemo(() => getSharedHexFogGeometry(), [])

  // OPTIMIZATION: Memoize shader uniforms with stable reference
  const uniforms = useMemo<FogOfWarUniforms>(
    () => ({
      ...defaultFogUniforms,
      uFogIntensity: { value: fogIntensity },
    }),
    [fogIntensity]
  )

  // OPTIMIZATION: Memoize shader material (expensive to create)
  const shaderMaterial = useMemo(() => {
    return new THREE.ShaderMaterial({
      vertexShader: fogOfWarVertexShader,
      fragmentShader: fogOfWarFragmentShader,
      uniforms: uniforms as unknown as { [uniform: string]: THREE.IUniform },
      transparent: true,
      depthWrite: false,
      blending: THREE.NormalBlending,
      side: THREE.DoubleSide,
    })
  }, [uniforms])

  // OPTIMIZATION: Check if any fog is needed before processing
  const hasAnyFog = useMemo(
    () => tiles.some((tile) => tile.visibility !== 'visible'),
    [tiles]
  )

  // OPTIMIZATION: Calculate instance data with reusable arrays
  const instanceData = useMemo(() => {
    if (!hasAnyFog) {
      return {
        positions: new Float32Array(0),
        visibilities: new Float32Array(0),
        count: 0,
      }
    }

    const { positions, visibilities } = getInstanceArrays(tiles.length)

    tiles.forEach((tile, i) => {
      const [x, y, z] = axialToWorld(tile.coord)

      // Adjust Y based on tile features (hills, etc.)
      const heightOffset = tile.features.includes('hills') ? 0.3 : 0

      const idx = i * 3
      positions[idx] = x
      positions[idx + 1] = y + heightOffset
      positions[idx + 2] = z
      visibilities[i] = visibilityToFloat(tile.visibility)
    })

    return {
      positions: positions.subarray(0, tiles.length * 3),
      visibilities: visibilities.subarray(0, tiles.length),
      count: tiles.length,
    }
  }, [tiles, hasAnyFog])

  // OPTIMIZATION: Update instance attributes efficiently
  useEffect(() => {
    if (!meshRef.current || !hasAnyFog) return

    const mesh = meshRef.current
    const geometry = mesh.geometry

    // Create or update position attribute
    let positionAttribute = geometry.getAttribute(
      'instancePosition'
    ) as THREE.InstancedBufferAttribute
    if (!positionAttribute || positionAttribute.count !== instanceData.count) {
      positionAttribute = new THREE.InstancedBufferAttribute(
        instanceData.positions,
        3
      )
      geometry.setAttribute('instancePosition', positionAttribute)
    } else {
      positionAttribute.set(instanceData.positions)
      positionAttribute.needsUpdate = true
    }

    // Create or update visibility attribute
    let visibilityAttribute = geometry.getAttribute(
      'instanceVisibility'
    ) as THREE.InstancedBufferAttribute
    if (
      !visibilityAttribute ||
      visibilityAttribute.count !== instanceData.count
    ) {
      visibilityAttribute = new THREE.InstancedBufferAttribute(
        instanceData.visibilities,
        1
      )
      geometry.setAttribute('instanceVisibility', visibilityAttribute)
    } else {
      visibilityAttribute.set(instanceData.visibilities)
      visibilityAttribute.needsUpdate = true
    }

    // Update instance count
    mesh.count = instanceData.count
  }, [instanceData, hasAnyFog])

  // OPTIMIZATION: Incremental visibility updates (only update changed tiles)
  useEffect(() => {
    if (!meshRef.current || !hasAnyFog) return

    const geometry = meshRef.current.geometry
    const visibilityAttr = geometry.getAttribute(
      'instanceVisibility'
    ) as THREE.InstancedBufferAttribute
    if (!visibilityAttr) return

    const prevVisibilities = prevVisibilitiesRef.current
    let hasChanges = false

    tiles.forEach((tile, index) => {
      const key = `${tile.coord.q},${tile.coord.r}`
      const prevVis = prevVisibilities.get(key)

      // Only update if visibility changed
      if (prevVis !== tile.visibility) {
        visibilityAttr.setX(index, visibilityToFloat(tile.visibility))
        prevVisibilities.set(key, tile.visibility)
        hasChanges = true
      }
    })

    // Only trigger GPU update if something changed
    if (hasChanges) {
      visibilityAttr.needsUpdate = true
    }
  }, [tiles, hasAnyFog])

  // OPTIMIZATION: Conditional animation frame (skip if not animated)
  useFrame((state) => {
    if (materialRef.current && animated && hasAnyFog) {
      materialRef.current.uniforms['uTime'].value = state.clock.elapsedTime
    }
  })

  // OPTIMIZATION: Early bailout when no fog needed
  if (!hasAnyFog) {
    return null
  }

  return (
    <instancedMesh
      ref={meshRef}
      args={[hexGeometry, shaderMaterial, instanceData.count]}
      frustumCulled={false}
    >
      <primitive object={hexGeometry} attach="geometry" />
      <primitive object={shaderMaterial} attach="material" ref={materialRef} />
    </instancedMesh>
  )
}

/**
 * Hook to calculate tile visibility based on unit sight ranges
 * This would typically be used by the game store to update tile visibility
 */
export function useVisibilityCalculation() {
  /**
   * Calculate which tiles should be visible based on unit positions
   * @param tiles All map tiles
   * @param unitPositions Positions of player's units
   * @param sightRange Default sight range for units
   * @returns Map of tile keys to new visibility states
   */
  const calculateVisibility = (
    tiles: HexTile[],
    unitPositions: HexCoord[],
    sightRange: number = 2
  ): Map<string, TileVisibility> => {
    const visibilityMap = new Map<string, TileVisibility>()

    // Helper to calculate hex distance
    const hexDistance = (a: HexCoord, b: HexCoord): number => {
      return Math.max(
        Math.abs(a.q - b.q),
        Math.abs(a.r - b.r),
        Math.abs(a.q + a.r - b.q - b.r)
      )
    }

    // Process each tile
    tiles.forEach((tile) => {
      const key = `${tile.coord.q},${tile.coord.r}`

      // Check if any unit can see this tile
      let isVisible = false
      for (const unitPos of unitPositions) {
        if (hexDistance(tile.coord, unitPos) <= sightRange) {
          isVisible = true
          break
        }
      }

      if (isVisible) {
        visibilityMap.set(key, 'visible')
      } else if (tile.visibility === 'visible') {
        // Was visible, now explored
        visibilityMap.set(key, 'explored')
      } else {
        // Keep current state (hidden or explored)
        visibilityMap.set(key, tile.visibility)
      }
    })

    return visibilityMap
  }

  return { calculateVisibility }
}

/**
 * Utility to get visible tiles only (for rendering units/cities)
 */
export function getVisibleTiles(tiles: HexTile[]): HexTile[] {
  return tiles.filter((tile) => tile.visibility === 'visible')
}

/**
 * OPTIMIZATION: Create a tile lookup map for O(1) access
 * Use this when you need to check multiple tiles frequently
 */
export function createTileLookup(tiles: HexTile[]): Map<string, HexTile> {
  const map = new Map<string, HexTile>()
  for (const tile of tiles) {
    map.set(`${tile.coord.q},${tile.coord.r}`, tile)
  }
  return map
}

/**
 * Utility to check if a specific tile is visible
 * OPTIMIZATION: For single lookups. For multiple lookups, use createTileLookup instead
 */
export function isTileVisible(tiles: HexTile[], coord: HexCoord): boolean {
  // Note: This is O(n) - for frequent calls, use createTileLookup for O(1)
  const tile = tiles.find((t) => t.coord.q === coord.q && t.coord.r === coord.r)
  return tile?.visibility === 'visible'
}

/**
 * Utility to check tile visibility with pre-built lookup map (O(1))
 */
export function isTileVisibleFast(
  tileLookup: Map<string, HexTile>,
  coord: HexCoord
): boolean {
  const tile = tileLookup.get(`${coord.q},${coord.r}`)
  return tile?.visibility === 'visible'
}

/**
 * Utility to check if a tile has been explored (visible or explored)
 */
export function isTileExplored(tiles: HexTile[], coord: HexCoord): boolean {
  const tile = tiles.find((t) => t.coord.q === coord.q && t.coord.r === coord.r)
  return tile?.visibility !== 'hidden'
}

/**
 * Utility to check tile explored status with pre-built lookup map (O(1))
 */
export function isTileExploredFast(
  tileLookup: Map<string, HexTile>,
  coord: HexCoord
): boolean {
  const tile = tileLookup.get(`${coord.q},${coord.r}`)
  return tile?.visibility !== 'hidden'
}

/**
 * OPTIMIZATION: React.memo to prevent unnecessary re-renders
 * Only re-renders when tiles array, fog intensity, or animation setting changes
 */
const MemoizedFogOfWar = React.memo(FogOfWar, (prevProps, nextProps) => {
  // Shallow comparison for primitive props
  if (prevProps.fogIntensity !== nextProps.fogIntensity) return false
  if (prevProps.animated !== nextProps.animated) return false

  // For tiles, check if the array reference changed
  // The parent component should ensure tiles array is memoized
  if (prevProps.tiles !== nextProps.tiles) {
    // If references differ, check if content actually changed
    // This is a shallow length check for performance
    if (prevProps.tiles.length !== nextProps.tiles.length) return false

    // For same-length arrays, assume content changed if reference changed
    // (parent should use useMemo to prevent unnecessary reference changes)
    return false
  }

  return true
})

export default MemoizedFogOfWar
