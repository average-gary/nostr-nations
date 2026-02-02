import React, { useMemo, useCallback, useState, useEffect, useRef } from 'react'
import { useFrame } from '@react-three/fiber'
import * as THREE from 'three'
import ParticleSystem, { ParticleConfig } from './ParticleSystem'

export interface TechResearchedEffectProps {
  position: [number, number, number]
  onComplete?: () => void
}

/**
 * Technology completion effect
 * Blue/cyan "data" particles spiraling upward
 */
const TechResearchedEffect: React.FC<TechResearchedEffectProps> = ({
  position,
  onComplete,
}) => {
  const [active, setActive] = useState(true)
  const spiralRef = useRef<THREE.Points>(null)
  const spiralTimeRef = useRef(0)

  // Reset when position changes
  useEffect(() => {
    setActive(true)
    spiralTimeRef.current = 0
  }, [position[0], position[1], position[2]])

  // Burst particle config
  const burstConfig = useMemo<ParticleConfig>(
    () => ({
      count: 40,
      lifetime: 1.5,
      velocityMin: new THREE.Vector3(-0.5, 2, -0.5),
      velocityMax: new THREE.Vector3(0.5, 4, 0.5),
      colors: ['#00bfff', '#00ffff', '#1e90ff', '#87ceeb', '#4169e1'],
      sizeMin: 0.04,
      sizeMax: 0.12,
      gravity: -1, // Float upward
      fadeOut: true,
      shrink: false,
      spreadRadius: 0.3,
      blending: THREE.AdditiveBlending,
    }),
    []
  )

  // Spiral "data stream" geometry
  const { spiralGeometry, spiralMaterial, spiralData } = useMemo(() => {
    const count = 30
    const positions = new Float32Array(count * 3)
    const colors = new Float32Array(count * 3)
    const sizes = new Float32Array(count)

    const techColors = [
      new THREE.Color('#00bfff'),
      new THREE.Color('#00ffff'),
      new THREE.Color('#ffffff'),
      new THREE.Color('#1e90ff'),
    ]

    // Store initial phase for each particle
    const phases = new Float32Array(count)

    for (let i = 0; i < count; i++) {
      const i3 = i * 3

      // Initial positions along spiral
      const t = i / count
      const angle = t * Math.PI * 4 // Two full rotations
      const radius = 0.2 + t * 0.3
      const height = t * 2

      positions[i3] = Math.cos(angle) * radius
      positions[i3 + 1] = height
      positions[i3 + 2] = Math.sin(angle) * radius

      const color = techColors[Math.floor(Math.random() * techColors.length)]
      colors[i3] = color.r
      colors[i3 + 1] = color.g
      colors[i3 + 2] = color.b

      sizes[i] = 0.06 + Math.random() * 0.06
      phases[i] = Math.random() * Math.PI * 2
    }

    const geo = new THREE.BufferGeometry()
    geo.setAttribute('position', new THREE.BufferAttribute(positions, 3))
    geo.setAttribute('color', new THREE.BufferAttribute(colors, 3))
    geo.setAttribute('size', new THREE.BufferAttribute(sizes, 1))

    const mat = new THREE.PointsMaterial({
      size: 0.12,
      vertexColors: true,
      transparent: true,
      opacity: 1,
      sizeAttenuation: true,
      blending: THREE.AdditiveBlending,
      depthWrite: false,
    })

    return { spiralGeometry: geo, spiralMaterial: mat, spiralData: { phases } }
  }, [])

  // Animate spiral
  useFrame((_, delta) => {
    if (!active || !spiralRef.current) return

    spiralTimeRef.current += delta
    const time = spiralTimeRef.current

    const posAttr = spiralGeometry.getAttribute(
      'position'
    ) as THREE.BufferAttribute
    const sizeAttr = spiralGeometry.getAttribute(
      'size'
    ) as THREE.BufferAttribute
    const count = posAttr.count

    for (let i = 0; i < count; i++) {
      const t = i / count
      const phase = spiralData.phases[i]

      // Animated spiral that rises and expands
      const angle = t * Math.PI * 4 + time * 3 + phase
      const baseRadius = 0.2 + t * 0.3
      const radius = baseRadius + Math.sin(time * 2 + i) * 0.1

      // Rise faster over time
      const height = (t * 2 + time * 1.5) % 3

      posAttr.setXYZ(
        i,
        Math.cos(angle) * radius,
        height,
        Math.sin(angle) * radius
      )

      // Pulsing size
      const sizePulse = 0.06 + Math.sin(time * 5 + phase) * 0.03
      sizeAttr.setX(i, sizePulse)
    }

    posAttr.needsUpdate = true
    sizeAttr.needsUpdate = true

    // Fade out after duration
    const fadeStart = 1.2
    const duration = 1.8
    if (time > fadeStart) {
      const fadeProgress = (time - fadeStart) / (duration - fadeStart)
      spiralMaterial.opacity = Math.max(0, 1 - fadeProgress)
    }
  })

  // Cleanup
  useEffect(() => {
    return () => {
      spiralGeometry.dispose()
      spiralMaterial.dispose()
    }
  }, [spiralGeometry, spiralMaterial])

  const handleComplete = useCallback(() => {
    setActive(false)
    onComplete?.()
  }, [onComplete])

  if (!active) return null

  return (
    <group position={position}>
      {/* Burst particles */}
      <ParticleSystem
        position={[0, 0, 0]}
        config={burstConfig}
        active={active}
        onComplete={handleComplete}
      />

      {/* Spiral data stream */}
      <points
        ref={spiralRef}
        geometry={spiralGeometry}
        material={spiralMaterial}
      />
    </group>
  )
}

export default React.memo(TechResearchedEffect)
