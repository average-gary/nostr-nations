import React, { useRef, useMemo, useCallback } from 'react'
import { ThreeEvent } from '@react-three/fiber'
import * as THREE from 'three'
import { Text } from '@react-three/drei'
import type { City } from '@/types/game'

interface CityMeshProps {
  city: City
  position: [number, number, number]
  isSelected: boolean
  onClick: (cityId: string) => void
  /** Whether this city is on an explored (but not visible) tile */
  dimmed?: boolean
}

// Player color lookup (pre-computed)
const PLAYER_COLORS: Record<string, string> = {
  'player-1': '#3182ce',
  'player-2': '#e53e3e',
  'player-3': '#38a169',
  'player-4': '#d69e2e',
}

function getPlayerColor(playerId: string): string {
  return PLAYER_COLORS[playerId] ?? '#718096'
}

/**
 * OPTIMIZATION: Shared geometry instances for city buildings
 * These are reused across all city components
 */
const sharedGeometries: Map<string, THREE.BufferGeometry> = new Map()

// OPTIMIZATION: Get or create shared geometry by key
function getSharedGeometry<T extends THREE.BufferGeometry>(
  key: string,
  createFn: () => T
): T {
  let geometry = sharedGeometries.get(key) as T
  if (!geometry) {
    geometry = createFn()
    sharedGeometries.set(key, geometry)
  }
  return geometry
}

// OPTIMIZATION: Pre-created shared geometries for common elements
function getOctahedronGeometry(): THREE.OctahedronGeometry {
  return getSharedGeometry(
    'octahedron-0.15',
    () => new THREE.OctahedronGeometry(0.15)
  )
}

function getSmallCircleGeometry(): THREE.CircleGeometry {
  return getSharedGeometry(
    'circle-0.15',
    () => new THREE.CircleGeometry(0.15, 12)
  ) // Reduced from 16 to 12
}

function getProductionCylinderGeometry(): THREE.CylinderGeometry {
  return getSharedGeometry(
    'prod-cylinder',
    () => new THREE.CylinderGeometry(0.08, 0.08, 0.15, 6)
  ) // Reduced from 8 to 6
}

/**
 * OPTIMIZATION: Shared materials for common colors
 */
const sharedMaterials: Map<
  string,
  THREE.MeshBasicMaterial | THREE.MeshStandardMaterial
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

/**
 * CityMesh - Individual city with optimizations:
 * - Shared geometry instances
 * - Shared materials for common colors
 * - useCallback for event handlers
 * - Memoized calculations
 * - React.memo for shallow comparison
 */
const CityMesh: React.FC<CityMeshProps> = ({
  city,
  position,
  isSelected,
  onClick,
  dimmed = false,
}) => {
  const groupRef = useRef<THREE.Group>(null)
  const [hovered, setHovered] = React.useState(false)

  // OPTIMIZATION: Memoize color calculations
  const baseColor = useMemo(() => getPlayerColor(city.owner), [city.owner])
  const color = useMemo(
    () => (dimmed ? '#4a5568' : baseColor),
    [dimmed, baseColor]
  )

  // OPTIMIZATION: Memoize size calculations
  const { sizeMultiplier, buildingHeight, baseSize } = useMemo(
    () => ({
      baseSize: 0.3,
      sizeMultiplier: 1 + Math.log10(city.population + 1) * 0.3,
      buildingHeight: 0.4 + city.population * 0.05,
    }),
    [city.population]
  )

  // OPTIMIZATION: Memoize adjusted position
  const adjustedPosition = useMemo<[number, number, number]>(
    () => [position[0], position[1] + 0.3, position[2]],
    [position]
  )

  // OPTIMIZATION: Memoize geometries that depend on size
  const cityBaseGeometry = useMemo(
    () =>
      new THREE.CylinderGeometry(
        0.6 * sizeMultiplier,
        0.7 * sizeMultiplier,
        0.2,
        6
      ),
    [sizeMultiplier]
  )

  const mainBuildingGeometry = useMemo(
    () =>
      new THREE.BoxGeometry(
        baseSize * sizeMultiplier,
        buildingHeight,
        baseSize * sizeMultiplier
      ),
    [baseSize, sizeMultiplier, buildingHeight]
  )

  const selectionRingGeometry = useMemo(
    () =>
      new THREE.RingGeometry(0.7 * sizeMultiplier, 0.85 * sizeMultiplier, 16), // Reduced from 32 to 16
    [sizeMultiplier]
  )

  // OPTIMIZATION: Get shared geometries for small elements
  const capitalGeometry = useMemo(() => getOctahedronGeometry(), [])
  const badgeCircleGeometry = useMemo(() => getSmallCircleGeometry(), [])
  const productionGeometry = useMemo(() => getProductionCylinderGeometry(), [])

  // OPTIMIZATION: Get shared materials
  const grayMaterial = useMemo(() => getSharedStandardMaterial('#718096'), [])
  const selectionMaterial = useMemo(() => getSharedBasicMaterial('#d69e2e'), [])
  const badgeMaterial = useMemo(() => getSharedBasicMaterial(color), [color])
  const productionMaterial = useMemo(
    () => getSharedStandardMaterial('#ed8936'),
    []
  )

  // OPTIMIZATION: useCallback for event handlers
  const handleClick = useCallback(
    (e: ThreeEvent<MouseEvent>) => {
      e.stopPropagation()
      onClick(city.id)
    },
    [onClick, city.id]
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

  // OPTIMIZATION: Memoize material props
  const baseMaterialProps = useMemo(
    () => ({
      color,
      emissive: isSelected ? '#d69e2e' : hovered ? color : '#000000',
      emissiveIntensity: isSelected ? 0.5 : hovered ? 0.3 : 0,
    }),
    [color, isSelected, hovered]
  )

  const mainBuildingMaterialProps = useMemo(
    () => ({
      color: '#4a5568',
      emissive: isSelected ? '#d69e2e' : '#000000',
      emissiveIntensity: isSelected ? 0.2 : 0,
    }),
    [isSelected]
  )

  // OPTIMIZATION: Memoize position calculations
  const mainBuildingPosition = useMemo<[number, number, number]>(
    () => [0, buildingHeight / 2 + 0.2, 0],
    [buildingHeight]
  )

  const capitalPosition = useMemo<[number, number, number]>(
    () => [0, buildingHeight + 0.5, 0],
    [buildingHeight]
  )

  const namePosition = useMemo<[number, number, number]>(
    () => [0, buildingHeight + 0.8, 0],
    [buildingHeight]
  )

  const badgeGroupPosition = useMemo<[number, number, number]>(
    () => [0.4 * sizeMultiplier, buildingHeight + 0.4, 0],
    [sizeMultiplier, buildingHeight]
  )

  const productionPosition = useMemo<[number, number, number]>(
    () => [-0.4 * sizeMultiplier, 0.3, 0.4 * sizeMultiplier],
    [sizeMultiplier]
  )

  // OPTIMIZATION: Memoize building visibility flags
  const showMediumBuildings = city.population >= 3
  const showLargeBuildings = city.population >= 6
  const hasProduction = !!city.production.item

  // OPTIMIZATION: Shared geometries for secondary buildings
  const smallBuilding1Geometry = useMemo(
    () =>
      getSharedGeometry(
        'building-0.15-0.4',
        () => new THREE.BoxGeometry(0.15, 0.4, 0.15)
      ),
    []
  )
  const smallBuilding2Geometry = useMemo(
    () =>
      getSharedGeometry(
        'building-0.15-0.3',
        () => new THREE.BoxGeometry(0.15, 0.3, 0.15)
      ),
    []
  )
  const smallBuilding3Geometry = useMemo(
    () =>
      getSharedGeometry(
        'building-0.12-0.5',
        () => new THREE.BoxGeometry(0.12, 0.5, 0.12)
      ),
    []
  )
  const smallBuilding4Geometry = useMemo(
    () =>
      getSharedGeometry(
        'building-0.1-0.2',
        () => new THREE.BoxGeometry(0.1, 0.2, 0.1)
      ),
    []
  )

  return (
    <group ref={groupRef} position={adjustedPosition}>
      {/* City base - OPTIMIZATION: uses memoized geometry */}
      <mesh
        position={[0, 0.1, 0]}
        geometry={cityBaseGeometry}
        onClick={handleClick}
        onPointerOver={handlePointerOver}
        onPointerOut={handlePointerOut}
      >
        <meshStandardMaterial {...baseMaterialProps} />
      </mesh>

      {/* Main building - OPTIMIZATION: uses memoized geometry and props */}
      <mesh position={mainBuildingPosition} geometry={mainBuildingGeometry}>
        <meshStandardMaterial {...mainBuildingMaterialProps} />
      </mesh>

      {/* Secondary buildings - OPTIMIZATION: shared geometries and materials */}
      {showMediumBuildings && (
        <>
          <mesh
            position={[-0.3, 0.3, 0.2]}
            geometry={smallBuilding1Geometry}
            material={grayMaterial}
          />
          <mesh
            position={[0.3, 0.25, -0.2]}
            geometry={smallBuilding2Geometry}
            material={grayMaterial}
          />
        </>
      )}

      {showLargeBuildings && (
        <>
          <mesh
            position={[-0.2, 0.35, -0.3]}
            geometry={smallBuilding3Geometry}
            material={grayMaterial}
          />
          <mesh
            position={[0.25, 0.2, 0.25]}
            geometry={smallBuilding4Geometry}
            material={grayMaterial}
          />
        </>
      )}

      {/* Capital star indicator - OPTIMIZATION: shared geometry */}
      {city.isCapital && (
        <mesh
          position={capitalPosition}
          rotation={[0, Math.PI / 4, 0]}
          geometry={capitalGeometry}
        >
          <meshStandardMaterial
            color="#d69e2e"
            emissive="#d69e2e"
            emissiveIntensity={0.5}
          />
        </mesh>
      )}

      {/* Selection ring - OPTIMIZATION: memoized geometry */}
      {isSelected && (
        <mesh
          position={[0, 0.05, 0]}
          rotation={[-Math.PI / 2, 0, 0]}
          geometry={selectionRingGeometry}
          material={selectionMaterial}
        />
      )}

      {/* City name label */}
      <Text
        position={namePosition}
        fontSize={0.25}
        color="#f7fafc"
        anchorX="center"
        anchorY="middle"
        outlineWidth={0.02}
        outlineColor="#1a202c"
      >
        {city.name}
      </Text>

      {/* Population badge - OPTIMIZATION: shared geometry and material */}
      <group position={badgeGroupPosition}>
        <mesh geometry={badgeCircleGeometry} material={badgeMaterial} />
        <Text
          position={[0, 0, 0.01]}
          fontSize={0.15}
          color="#f7fafc"
          anchorX="center"
          anchorY="middle"
        >
          {city.population}
        </Text>
      </group>

      {/* Production indicator - OPTIMIZATION: shared geometry and material */}
      {hasProduction && (
        <mesh
          position={productionPosition}
          geometry={productionGeometry}
          material={productionMaterial}
        />
      )}
    </group>
  )
}

/**
 * OPTIMIZATION: React.memo with custom comparison
 * Prevents re-renders when props haven't meaningfully changed
 */
export default React.memo(CityMesh, (prevProps, nextProps) => {
  // Quick reference checks
  if (prevProps.isSelected !== nextProps.isSelected) return false
  if (prevProps.dimmed !== nextProps.dimmed) return false
  if (prevProps.onClick !== nextProps.onClick) return false

  // Position comparison
  if (
    prevProps.position[0] !== nextProps.position[0] ||
    prevProps.position[1] !== nextProps.position[1] ||
    prevProps.position[2] !== nextProps.position[2]
  )
    return false

  // City comparison - check fields that affect rendering
  const prevCity = prevProps.city
  const nextCity = nextProps.city
  if (prevCity.id !== nextCity.id) return false
  if (prevCity.name !== nextCity.name) return false
  if (prevCity.owner !== nextCity.owner) return false
  if (prevCity.population !== nextCity.population) return false
  if (prevCity.isCapital !== nextCity.isCapital) return false
  if (!!prevCity.production.item !== !!nextCity.production.item) return false

  return true // Props are equal, skip re-render
})
