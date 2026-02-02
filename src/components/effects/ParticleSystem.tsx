import React, { useRef, useMemo, useEffect } from 'react'
import { useFrame } from '@react-three/fiber'
import * as THREE from 'three'

export interface ParticleConfig {
  /** Number of particles to spawn */
  count: number
  /** Particle lifetime in seconds */
  lifetime: number
  /** Base velocity range [min, max] */
  velocityMin: THREE.Vector3
  velocityMax: THREE.Vector3
  /** Particle color(s) - can be single color or array for random selection */
  colors: string[]
  /** Size range [min, max] */
  sizeMin: number
  sizeMax: number
  /** Gravity multiplier (negative = upward) */
  gravity?: number
  /** Whether particles should fade out */
  fadeOut?: boolean
  /** Whether particles should shrink over time */
  shrink?: boolean
  /** Spread radius for initial position */
  spreadRadius?: number
  /** Blend mode for particles */
  blending?: THREE.Blending
}

export interface ParticleSystemProps {
  position: [number, number, number]
  config: ParticleConfig
  active: boolean
  onComplete?: () => void
}

interface ParticleData {
  positions: Float32Array
  velocities: Float32Array
  colors: Float32Array
  sizes: Float32Array
  lifetimes: Float32Array
  maxLifetimes: Float32Array
}

/**
 * GPU-efficient particle system using THREE.Points
 * Supports configurable particle behavior for various effects
 */
const ParticleSystem: React.FC<ParticleSystemProps> = ({
  position,
  config,
  active,
  onComplete,
}) => {
  const pointsRef = useRef<THREE.Points>(null)
  const particleDataRef = useRef<ParticleData | null>(null)
  const hasCompletedRef = useRef(false)
  const startTimeRef = useRef<number | null>(null)

  // Parse colors once
  const parsedColors = useMemo(() => {
    return config.colors.map((c) => new THREE.Color(c))
  }, [config.colors])

  // Initialize particle data
  const { geometry, material } = useMemo(() => {
    const { count, sizeMin, sizeMax, lifetime, spreadRadius = 0 } = config

    const positions = new Float32Array(count * 3)
    const velocities = new Float32Array(count * 3)
    const colors = new Float32Array(count * 3)
    const sizes = new Float32Array(count)
    const lifetimes = new Float32Array(count)
    const maxLifetimes = new Float32Array(count)

    for (let i = 0; i < count; i++) {
      const i3 = i * 3

      // Random initial position within spread radius
      const angle = Math.random() * Math.PI * 2
      const radius = Math.random() * spreadRadius
      positions[i3] = Math.cos(angle) * radius
      positions[i3 + 1] = 0
      positions[i3 + 2] = Math.sin(angle) * radius

      // Random velocity within range
      velocities[i3] =
        config.velocityMin.x +
        Math.random() * (config.velocityMax.x - config.velocityMin.x)
      velocities[i3 + 1] =
        config.velocityMin.y +
        Math.random() * (config.velocityMax.y - config.velocityMin.y)
      velocities[i3 + 2] =
        config.velocityMin.z +
        Math.random() * (config.velocityMax.z - config.velocityMin.z)

      // Random color from palette
      const color =
        parsedColors[Math.floor(Math.random() * parsedColors.length)]
      colors[i3] = color.r
      colors[i3 + 1] = color.g
      colors[i3 + 2] = color.b

      // Random size
      sizes[i] = sizeMin + Math.random() * (sizeMax - sizeMin)

      // Random lifetime variation (80-100% of max)
      const maxLife = lifetime * (0.8 + Math.random() * 0.2)
      lifetimes[i] = maxLife
      maxLifetimes[i] = maxLife
    }

    particleDataRef.current = {
      positions,
      velocities,
      colors,
      sizes,
      lifetimes,
      maxLifetimes,
    }

    const geo = new THREE.BufferGeometry()
    geo.setAttribute('position', new THREE.BufferAttribute(positions, 3))
    geo.setAttribute('color', new THREE.BufferAttribute(colors, 3))
    geo.setAttribute('size', new THREE.BufferAttribute(sizes, 1))

    const mat = new THREE.PointsMaterial({
      size: 0.1,
      vertexColors: true,
      transparent: true,
      opacity: 1,
      sizeAttenuation: true,
      blending: config.blending ?? THREE.AdditiveBlending,
      depthWrite: false,
    })

    return { geometry: geo, material: mat }
  }, [config, parsedColors])

  // Reset particles when becoming active
  useEffect(() => {
    if (active) {
      hasCompletedRef.current = false
      startTimeRef.current = null

      // Reset particle data
      const data = particleDataRef.current
      if (data) {
        for (let i = 0; i < config.count; i++) {
          data.lifetimes[i] = data.maxLifetimes[i]
        }
      }
    }
  }, [active, config.count])

  // Cleanup
  useEffect(() => {
    return () => {
      geometry.dispose()
      material.dispose()
    }
  }, [geometry, material])

  // Animation loop
  useFrame((state, delta) => {
    if (!active || !pointsRef.current || !particleDataRef.current) return

    if (startTimeRef.current === null) {
      startTimeRef.current = state.clock.elapsedTime
    }

    const data = particleDataRef.current
    const { gravity = 0, fadeOut = true, shrink = false } = config
    const posAttr = geometry.getAttribute('position') as THREE.BufferAttribute
    const sizeAttr = geometry.getAttribute('size') as THREE.BufferAttribute
    const colorAttr = geometry.getAttribute('color') as THREE.BufferAttribute

    let allDead = true

    for (let i = 0; i < config.count; i++) {
      if (data.lifetimes[i] <= 0) continue

      allDead = false
      const i3 = i * 3

      // Update lifetime
      data.lifetimes[i] -= delta

      const lifeRatio = Math.max(0, data.lifetimes[i] / data.maxLifetimes[i])

      // Update position
      data.positions[i3] += data.velocities[i3] * delta
      data.positions[i3 + 1] += data.velocities[i3 + 1] * delta
      data.positions[i3 + 2] += data.velocities[i3 + 2] * delta

      // Apply gravity
      data.velocities[i3 + 1] -= gravity * delta

      posAttr.setXYZ(
        i,
        data.positions[i3],
        data.positions[i3 + 1],
        data.positions[i3 + 2]
      )

      // Fade out (modify alpha via color intensity)
      if (fadeOut) {
        const originalColor = parsedColors[Math.floor(i % parsedColors.length)]
        colorAttr.setXYZ(
          i,
          originalColor.r * lifeRatio,
          originalColor.g * lifeRatio,
          originalColor.b * lifeRatio
        )
      }

      // Shrink
      if (shrink) {
        sizeAttr.setX(i, data.sizes[i] * lifeRatio)
      }
    }

    posAttr.needsUpdate = true
    if (fadeOut) colorAttr.needsUpdate = true
    if (shrink) sizeAttr.needsUpdate = true

    // Check completion
    if (allDead && !hasCompletedRef.current) {
      hasCompletedRef.current = true
      onComplete?.()
    }
  })

  if (!active) return null

  return (
    <points
      ref={pointsRef}
      position={position}
      geometry={geometry}
      material={material}
    />
  )
}

export default React.memo(ParticleSystem)
