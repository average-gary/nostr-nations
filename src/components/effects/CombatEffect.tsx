import React, { useMemo, useCallback, useState, useEffect } from 'react'
import * as THREE from 'three'
import ParticleSystem, { ParticleConfig } from './ParticleSystem'

export type DamageType = 'melee' | 'ranged' | 'siege' | 'magic'

export interface CombatEffectProps {
  position: [number, number, number]
  intensity?: number
  damageType?: DamageType
  onComplete?: () => void
}

// Color palettes for different damage types
const DAMAGE_COLORS: Record<DamageType, string[]> = {
  melee: ['#ff4444', '#ff6666', '#cc2222', '#ff8888'],
  ranged: ['#ffcc00', '#ffdd44', '#ff9900', '#ffee66'],
  siege: ['#ff6600', '#ff8833', '#cc4400', '#ffaa55'],
  magic: ['#aa44ff', '#cc66ff', '#8822cc', '#dd88ff'],
}

/**
 * Combat explosion/hit effect
 * Burst of particles on impact with color based on damage type
 */
const CombatEffect: React.FC<CombatEffectProps> = ({
  position,
  intensity = 1,
  damageType = 'melee',
  onComplete,
}) => {
  const [active, setActive] = useState(true)

  // Reset when position changes (new effect)
  useEffect(() => {
    setActive(true)
  }, [position[0], position[1], position[2]])

  const config = useMemo<ParticleConfig>(() => {
    const baseCount = 30
    const count = Math.floor(baseCount * intensity)
    const speed = 2 * intensity

    return {
      count,
      lifetime: 0.8,
      velocityMin: new THREE.Vector3(-speed, 0.5, -speed),
      velocityMax: new THREE.Vector3(speed, speed * 1.5, speed),
      colors: DAMAGE_COLORS[damageType],
      sizeMin: 0.05 * intensity,
      sizeMax: 0.15 * intensity,
      gravity: 3,
      fadeOut: true,
      shrink: true,
      spreadRadius: 0.1,
      blending: THREE.AdditiveBlending,
    }
  }, [intensity, damageType])

  const handleComplete = useCallback(() => {
    setActive(false)
    onComplete?.()
  }, [onComplete])

  return (
    <ParticleSystem
      position={position}
      config={config}
      active={active}
      onComplete={handleComplete}
    />
  )
}

export default React.memo(CombatEffect)
