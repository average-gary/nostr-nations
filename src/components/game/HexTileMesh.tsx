import React, { useMemo, useRef, useCallback } from 'react'
import * as THREE from 'three'
import { ThreeEvent } from '@react-three/fiber'
import type { HexTile, HexCoord } from '@/types/game'

interface HexTileMeshProps {
  tile: HexTile
  position: [number, number, number]
  isSelected: boolean
  onClick: (coord: HexCoord) => void
}

const HEX_SIZE = 1
const HEX_HEIGHT = 0.2

/**
 * OPTIMIZATION: Shared geometry instance for all hex tiles
 * Created once and reused by all HexTileMesh components
 * This dramatically reduces memory usage and GPU draw calls
 */
let sharedHexGeometry: THREE.BufferGeometry | null = null

function getSharedHexGeometry(): THREE.BufferGeometry {
  if (!sharedHexGeometry) {
    const shape = new THREE.Shape()
    const corners: [number, number][] = []

    // Generate hex corner points
    for (let i = 0; i < 6; i++) {
      const angle = (Math.PI / 3) * i - Math.PI / 6
      const x = HEX_SIZE * Math.cos(angle)
      const y = HEX_SIZE * Math.sin(angle)
      corners.push([x, y])
    }

    // Create shape from corners
    shape.moveTo(corners[0][0], corners[0][1])
    for (let i = 1; i < 6; i++) {
      shape.lineTo(corners[i][0], corners[i][1])
    }
    shape.closePath()

    // Extrude to create 3D hex
    const extrudeSettings = {
      depth: HEX_HEIGHT,
      bevelEnabled: false,
    }

    sharedHexGeometry = new THREE.ExtrudeGeometry(shape, extrudeSettings)
    sharedHexGeometry.rotateX(-Math.PI / 2)
  }
  return sharedHexGeometry
}

/**
 * OPTIMIZATION: Shared geometry instances for sub-meshes
 * Reduces memory and GPU overhead
 */
let sharedRingGeometry: THREE.RingGeometry | null = null
let sharedResourceSphereGeometry: THREE.SphereGeometry | null = null
let sharedImprovementBoxGeometry: THREE.BoxGeometry | null = null

function getSharedRingGeometry(): THREE.RingGeometry {
  if (!sharedRingGeometry) {
    // OPTIMIZATION: Reduced segments from default (8 -> 6 to match hex shape)
    sharedRingGeometry = new THREE.RingGeometry(
      HEX_SIZE * 0.8,
      HEX_SIZE * 0.9,
      6
    )
  }
  return sharedRingGeometry
}

function getSharedResourceSphereGeometry(): THREE.SphereGeometry {
  if (!sharedResourceSphereGeometry) {
    // OPTIMIZATION: Reduced segments from 16 to 8 (still looks smooth)
    sharedResourceSphereGeometry = new THREE.SphereGeometry(0.15, 8, 8)
  }
  return sharedResourceSphereGeometry
}

function getSharedImprovementBoxGeometry(): THREE.BoxGeometry {
  if (!sharedImprovementBoxGeometry) {
    sharedImprovementBoxGeometry = new THREE.BoxGeometry(0.2, 0.2, 0.2)
  }
  return sharedImprovementBoxGeometry
}

/**
 * OPTIMIZATION: Shared material instances for common materials
 * Materials are expensive to create; sharing them reduces GPU memory
 */
const sharedMaterials: Map<
  string,
  THREE.MeshStandardMaterial | THREE.MeshBasicMaterial
> = new Map()

function getSharedBasicMaterial(color: string): THREE.MeshBasicMaterial {
  const key = `basic-${color}`
  let material = sharedMaterials.get(key) as THREE.MeshBasicMaterial
  if (!material) {
    material = new THREE.MeshBasicMaterial({ color, side: THREE.DoubleSide })
    sharedMaterials.set(key, material)
  }
  return material
}

function getSharedStandardMaterial(color: string): THREE.MeshStandardMaterial {
  const key = `standard-${color}`
  let material = sharedMaterials.get(key) as THREE.MeshStandardMaterial
  if (!material) {
    material = new THREE.MeshStandardMaterial({ color })
    sharedMaterials.set(key, material)
  }
  return material
}

// Terrain color lookup (pre-computed for performance)
const TERRAIN_COLORS: Record<string, string> = {
  grassland: '#48bb78',
  plains: '#c6a969',
  desert: '#ecc94b',
  tundra: '#a0aec0',
  snow: '#e2e8f0',
  coast: '#63b3ed',
  ocean: '#3182ce',
  mountain: '#4a5568',
}

const FEATURE_COLORS: Record<string, string> = {
  forest: '#276749',
  jungle: '#22543d',
  marsh: '#68d391',
}

const RESOURCE_COLORS: Record<string, string> = {
  wheat: '#f6e05e',
  cattle: '#9f7aea',
  fish: '#4299e1',
  iron: '#718096',
  horses: '#ed8936',
  coal: '#1a202c',
  oil: '#2d3748',
  gold: '#d69e2e',
  gems: '#b794f4',
  marble: '#e2e8f0',
  stone: '#a0aec0',
}

/**
 * OPTIMIZATION: Memoized terrain color calculation
 */
function getTerrainColor(terrain: string, features: string[]): string {
  // Check features first (they override base terrain)
  for (const feature of features) {
    if (FEATURE_COLORS[feature]) {
      return FEATURE_COLORS[feature]
    }
  }
  return TERRAIN_COLORS[terrain] ?? '#718096'
}

function getResourceColor(resourceType: string): string {
  return RESOURCE_COLORS[resourceType] ?? '#718096'
}

/**
 * HexTileMesh - Individual hex tile with optimizations:
 * - Shared geometry instances (single allocation for all tiles)
 * - Shared material instances where possible
 * - useCallback for event handlers
 * - Memoized calculations
 * - React.memo for shallow comparison
 */
const HexTileMesh: React.FC<HexTileMeshProps> = ({
  tile,
  position,
  isSelected,
  onClick,
}) => {
  const meshRef = useRef<THREE.Mesh>(null)

  // OPTIMIZATION: Use shared geometry instead of creating per-instance
  const geometry = useMemo(() => getSharedHexGeometry(), [])

  // OPTIMIZATION: Memoize sub-mesh geometries
  const ringGeometry = useMemo(() => getSharedRingGeometry(), [])
  const resourceGeometry = useMemo(() => getSharedResourceSphereGeometry(), [])
  const improvementGeometry = useMemo(
    () => getSharedImprovementBoxGeometry(),
    []
  )

  // OPTIMIZATION: Memoize height offset calculation
  const heightOffset = useMemo(
    () => (tile.features.includes('hills') ? 0.3 : 0),
    [tile.features]
  )

  const adjustedPosition = useMemo<[number, number, number]>(
    () => [position[0], position[1] + heightOffset, position[2]],
    [position, heightOffset]
  )

  // OPTIMIZATION: Memoize terrain color
  const color = useMemo(
    () => getTerrainColor(tile.terrain, tile.features),
    [tile.terrain, tile.features]
  )

  // Handle hover state
  const [hovered, setHovered] = React.useState(false)

  // OPTIMIZATION: useCallback for event handlers to prevent recreation
  const handleClick = useCallback(
    (e: ThreeEvent<MouseEvent>) => {
      e.stopPropagation()
      onClick(tile.coord)
    },
    [onClick, tile.coord]
  )

  const handlePointerOver = useCallback((e: ThreeEvent<PointerEvent>) => {
    e.stopPropagation()
    setHovered(true)
    document.body.style.cursor = 'pointer'
  }, [])

  const handlePointerOut = useCallback(() => {
    setHovered(false)
    document.body.style.cursor = 'default'
  }, [])

  // OPTIMIZATION: Memoize material properties based on state
  const materialProps = useMemo(
    () => ({
      color: hovered ? '#ffffff' : color,
      emissive: isSelected ? '#d69e2e' : hovered ? color : '#000000',
      emissiveIntensity: isSelected ? 0.5 : hovered ? 0.3 : 0,
      toneMapped: true,
    }),
    [hovered, color, isSelected]
  )

  // OPTIMIZATION: Memoize resource color if present
  const resourceColor = useMemo(
    () => (tile.resource ? getResourceColor(tile.resource.type) : null),
    [tile.resource]
  )

  // OPTIMIZATION: Get shared materials for sub-meshes
  const selectionMaterial = useMemo(() => getSharedBasicMaterial('#d69e2e'), [])
  const improvementMaterial = useMemo(
    () => getSharedStandardMaterial('#718096'),
    []
  )

  return (
    <group position={adjustedPosition}>
      {/* Main hex mesh */}
      <mesh
        ref={meshRef}
        geometry={geometry}
        onClick={handleClick}
        onPointerOver={handlePointerOver}
        onPointerOut={handlePointerOut}
      >
        <meshStandardMaterial {...materialProps} />
      </mesh>

      {/* Selection indicator ring - OPTIMIZATION: uses shared geometry */}
      {isSelected && (
        <mesh
          position={[0, HEX_HEIGHT + 0.01, 0]}
          rotation={[-Math.PI / 2, 0, 0]}
          geometry={ringGeometry}
          material={selectionMaterial}
        />
      )}

      {/* Resource indicator - OPTIMIZATION: uses shared geometry and memoized color */}
      {tile.resource && resourceColor && (
        <mesh position={[0, HEX_HEIGHT + 0.3, 0]} geometry={resourceGeometry}>
          <meshStandardMaterial
            color={resourceColor}
            emissive={resourceColor}
            emissiveIntensity={0.3}
          />
        </mesh>
      )}

      {/* Improvement indicator - OPTIMIZATION: uses shared geometry and material */}
      {tile.improvement && (
        <mesh
          position={[0.3, HEX_HEIGHT + 0.15, 0.3]}
          geometry={improvementGeometry}
          material={improvementMaterial}
        />
      )}
    </group>
  )
}

/**
 * OPTIMIZATION: React.memo with custom comparison
 * Only re-renders when tile data, selection state, or position actually changes
 */
export default React.memo(HexTileMesh, (prevProps, nextProps) => {
  // Shallow comparison for most props
  if (prevProps.isSelected !== nextProps.isSelected) return false

  // Position array comparison
  if (
    prevProps.position[0] !== nextProps.position[0] ||
    prevProps.position[1] !== nextProps.position[1] ||
    prevProps.position[2] !== nextProps.position[2]
  )
    return false

  // Tile comparison - check relevant fields
  const prevTile = prevProps.tile
  const nextTile = nextProps.tile
  if (prevTile.terrain !== nextTile.terrain) return false
  if (prevTile.visibility !== nextTile.visibility) return false
  if (
    prevTile.coord.q !== nextTile.coord.q ||
    prevTile.coord.r !== nextTile.coord.r
  )
    return false
  if (prevTile.features.length !== nextTile.features.length) return false
  if (prevTile.resource?.type !== nextTile.resource?.type) return false
  if (prevTile.improvement !== nextTile.improvement) return false

  // onClick is a callback - compare by reference
  if (prevProps.onClick !== nextProps.onClick) return false

  return true // Props are equal, skip re-render
})
