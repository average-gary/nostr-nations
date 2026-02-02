import React, { useRef, useMemo, useCallback } from 'react'
import { useFrame, ThreeEvent } from '@react-three/fiber'
import * as THREE from 'three'
import { Text } from '@react-three/drei'
import type { Unit } from '@/types/game'

interface UnitMeshProps {
  unit: Unit
  position: [number, number, number]
  isSelected: boolean
  onClick: (unitId: string) => void
}

const UNIT_HEIGHT = 0.5

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

// Unit shape types
type UnitShapeType = 'cylinder' | 'cone' | 'box'

const UNIT_SHAPES: Record<string, UnitShapeType> = {
  settler: 'box',
  warrior: 'cylinder',
  swordsman: 'cylinder',
  archer: 'cone',
  scout: 'cone',
}

function getUnitShape(unitType: string): UnitShapeType {
  return UNIT_SHAPES[unitType] ?? 'cylinder'
}

/**
 * OPTIMIZATION: Shared geometry instances for all unit types
 * Created once and reused across all unit components
 */
const sharedGeometries: Map<string, THREE.BufferGeometry> = new Map()

function getSharedUnitGeometry(shape: UnitShapeType): THREE.BufferGeometry {
  let geometry = sharedGeometries.get(shape)
  if (!geometry) {
    switch (shape) {
      case 'cylinder':
        // OPTIMIZATION: Reduced radial segments from 16 to 12
        geometry = new THREE.CylinderGeometry(0.25, 0.25, 0.5, 12)
        break
      case 'cone':
        // OPTIMIZATION: Reduced radial segments from 16 to 12
        geometry = new THREE.ConeGeometry(0.25, 0.5, 12)
        break
      case 'box':
        geometry = new THREE.BoxGeometry(0.4, 0.4, 0.4)
        break
    }
    sharedGeometries.set(shape, geometry)
  }
  return geometry
}

// OPTIMIZATION: Shared geometries for UI elements
let sharedHealthBgGeometry: THREE.PlaneGeometry | null = null
let sharedSelectionRingGeometry: THREE.RingGeometry | null = null
let sharedMovementIndicatorGeometry: THREE.SphereGeometry | null = null

function getHealthBgGeometry(): THREE.PlaneGeometry {
  if (!sharedHealthBgGeometry) {
    sharedHealthBgGeometry = new THREE.PlaneGeometry(0.6, 0.08)
  }
  return sharedHealthBgGeometry
}

function getSelectionRingGeometry(): THREE.RingGeometry {
  if (!sharedSelectionRingGeometry) {
    // OPTIMIZATION: Reduced segments from 32 to 16
    sharedSelectionRingGeometry = new THREE.RingGeometry(0.4, 0.5, 16)
  }
  return sharedSelectionRingGeometry
}

function getMovementIndicatorGeometry(): THREE.SphereGeometry {
  if (!sharedMovementIndicatorGeometry) {
    // OPTIMIZATION: Low poly sphere for small indicator
    sharedMovementIndicatorGeometry = new THREE.SphereGeometry(0.08, 6, 6)
  }
  return sharedMovementIndicatorGeometry
}

/**
 * OPTIMIZATION: Shared materials for common colors
 */
const sharedMaterials: Map<string, THREE.MeshBasicMaterial> = new Map()

function getSharedBasicMaterial(color: string): THREE.MeshBasicMaterial {
  const key = `basic-${color}`
  let material = sharedMaterials.get(key)
  if (!material) {
    material = new THREE.MeshBasicMaterial({ color, side: THREE.DoubleSide })
    sharedMaterials.set(key, material)
  }
  return material
}

/**
 * UnitMesh - Individual unit with optimizations:
 * - Shared geometry instances (single allocation per unit type)
 * - Shared materials for UI elements
 * - useCallback for event handlers
 * - Memoized calculations
 * - Conditional animation (only when selected)
 * - React.memo for shallow comparison
 */
const UnitMesh: React.FC<UnitMeshProps> = ({
  unit,
  position,
  isSelected,
  onClick,
}) => {
  const groupRef = useRef<THREE.Group>(null)
  const meshRef = useRef<THREE.Mesh>(null)

  // OPTIMIZATION: Only run animation frame when selected
  useFrame((state) => {
    if (isSelected && meshRef.current) {
      meshRef.current.position.y =
        UNIT_HEIGHT + Math.sin(state.clock.elapsedTime * 3) * 0.05
    }
  })

  const [hovered, setHovered] = React.useState(false)

  // OPTIMIZATION: Memoize color and shape calculations
  const color = useMemo(() => getPlayerColor(unit.owner), [unit.owner])
  const shape = useMemo(() => getUnitShape(unit.type), [unit.type])

  // OPTIMIZATION: Get shared geometry for unit type
  const unitGeometry = useMemo(() => getSharedUnitGeometry(shape), [shape])

  // OPTIMIZATION: Memoize health calculations
  const healthPercent = useMemo(
    () => unit.health / unit.maxHealth,
    [unit.health, unit.maxHealth]
  )
  const healthColor = useMemo(
    () =>
      healthPercent > 0.66
        ? '#48bb78'
        : healthPercent > 0.33
          ? '#ed8936'
          : '#f56565',
    [healthPercent]
  )

  // OPTIMIZATION: Memoize health bar position (depends on health)
  const healthBarFillPosition = useMemo<[number, number, number]>(
    () => [-0.3 * (1 - healthPercent), UNIT_HEIGHT + 0.5, 0.001],
    [healthPercent]
  )

  // OPTIMIZATION: Memoize health bar fill geometry (depends on health)
  const healthBarFillGeometry = useMemo(
    () => new THREE.PlaneGeometry(0.6 * healthPercent, 0.06),
    [healthPercent]
  )

  // Adjusted position (raised above hex)
  const adjustedPosition = useMemo<[number, number, number]>(
    () => [position[0], position[1] + 0.3, position[2]],
    [position]
  )

  // OPTIMIZATION: Get shared geometries
  const healthBgGeometry = useMemo(() => getHealthBgGeometry(), [])
  const selectionRingGeometry = useMemo(() => getSelectionRingGeometry(), [])
  const movementGeometry = useMemo(() => getMovementIndicatorGeometry(), [])

  // OPTIMIZATION: Get shared materials
  const healthBgMaterial = useMemo(() => getSharedBasicMaterial('#1a202c'), [])
  const healthFillMaterial = useMemo(
    () => getSharedBasicMaterial(healthColor),
    [healthColor]
  )
  const selectionMaterial = useMemo(() => getSharedBasicMaterial('#d69e2e'), [])
  const movementMaterial = useMemo(() => getSharedBasicMaterial('#48bb78'), [])

  // OPTIMIZATION: useCallback for event handlers
  const handleClick = useCallback(
    (e: ThreeEvent<MouseEvent>) => {
      e.stopPropagation()
      onClick(unit.id)
    },
    [onClick, unit.id]
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
  const unitMaterialProps = useMemo(
    () => ({
      color: hovered ? '#ffffff' : color,
      emissive: isSelected ? '#d69e2e' : color,
      emissiveIntensity: isSelected ? 0.5 : hovered ? 0.3 : 0.1,
    }),
    [hovered, color, isSelected]
  )

  // OPTIMIZATION: Memoize label text
  const labelText = useMemo(() => unit.type.toUpperCase(), [unit.type])

  // OPTIMIZATION: Show movement indicator flag
  const showMovementIndicator = unit.canAct && unit.movement > 0

  return (
    <group ref={groupRef} position={adjustedPosition}>
      {/* Unit mesh - OPTIMIZATION: uses shared geometry */}
      <mesh
        ref={meshRef}
        position={[0, UNIT_HEIGHT, 0]}
        geometry={unitGeometry}
        onClick={handleClick}
        onPointerOver={handlePointerOver}
        onPointerOut={handlePointerOut}
      >
        <meshStandardMaterial {...unitMaterialProps} />
      </mesh>

      {/* Health bar background - OPTIMIZATION: shared geometry and material */}
      <mesh
        position={[0, UNIT_HEIGHT + 0.5, 0]}
        geometry={healthBgGeometry}
        material={healthBgMaterial}
      />

      {/* Health bar fill */}
      <mesh
        position={healthBarFillPosition}
        geometry={healthBarFillGeometry}
        material={healthFillMaterial}
      />

      {/* Selection ring - OPTIMIZATION: shared geometry and material */}
      {isSelected && (
        <mesh
          position={[0, 0.05, 0]}
          rotation={[-Math.PI / 2, 0, 0]}
          geometry={selectionRingGeometry}
          material={selectionMaterial}
        />
      )}

      {/* Movement indicator - OPTIMIZATION: shared geometry and material */}
      {showMovementIndicator && (
        <mesh
          position={[0.35, UNIT_HEIGHT + 0.3, 0]}
          geometry={movementGeometry}
          material={movementMaterial}
        />
      )}

      {/* Unit type label - OPTIMIZATION: only rendered when needed */}
      {(hovered || isSelected) && (
        <Text
          position={[0, UNIT_HEIGHT + 0.7, 0]}
          fontSize={0.2}
          color="#f7fafc"
          anchorX="center"
          anchorY="middle"
        >
          {labelText}
        </Text>
      )}
    </group>
  )
}

/**
 * OPTIMIZATION: React.memo with custom comparison
 * Prevents re-renders when props haven't meaningfully changed
 */
export default React.memo(UnitMesh, (prevProps, nextProps) => {
  // Quick reference checks
  if (prevProps.isSelected !== nextProps.isSelected) return false
  if (prevProps.onClick !== nextProps.onClick) return false

  // Position comparison
  if (
    prevProps.position[0] !== nextProps.position[0] ||
    prevProps.position[1] !== nextProps.position[1] ||
    prevProps.position[2] !== nextProps.position[2]
  )
    return false

  // Unit comparison - check fields that affect rendering
  const prevUnit = prevProps.unit
  const nextUnit = nextProps.unit
  if (prevUnit.id !== nextUnit.id) return false
  if (prevUnit.type !== nextUnit.type) return false
  if (prevUnit.owner !== nextUnit.owner) return false
  if (prevUnit.health !== nextUnit.health) return false
  if (prevUnit.maxHealth !== nextUnit.maxHealth) return false
  if (prevUnit.canAct !== nextUnit.canAct) return false
  if (prevUnit.movement !== nextUnit.movement) return false

  return true // Props are equal, skip re-render
})
