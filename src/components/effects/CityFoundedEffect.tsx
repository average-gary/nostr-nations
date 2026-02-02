import React, { useMemo, useCallback, useState, useEffect, useRef } from 'react'
import { useFrame } from '@react-three/fiber'
import * as THREE from 'three'
import ParticleSystem, { ParticleConfig } from './ParticleSystem'

export interface CityFoundedEffectProps {
  position: [number, number, number]
  onComplete?: () => void
}

/**
 * City founding celebration effect
 * Upward rising gold particles with sparkle effect
 */
const CityFoundedEffect: React.FC<CityFoundedEffectProps> = ({
  position,
  onComplete,
}) => {
  const [active, setActive] = useState(true)
  const sparkleRef = useRef<THREE.Points>(null)
  const sparkleTimeRef = useRef(0)

  // Reset when position changes
  useEffect(() => {
    setActive(true)
    sparkleTimeRef.current = 0
  }, [position[0], position[1], position[2]])

  // Main rising particle config
  const risingConfig = useMemo<ParticleConfig>(
    () => ({
      count: 50,
      lifetime: 2,
      velocityMin: new THREE.Vector3(-0.3, 1.5, -0.3),
      velocityMax: new THREE.Vector3(0.3, 3, 0.3),
      colors: ['#ffd700', '#ffec8b', '#ffa500', '#fff8dc', '#daa520'],
      sizeMin: 0.08,
      sizeMax: 0.2,
      gravity: -0.5, // Negative gravity = particles float upward
      fadeOut: true,
      shrink: false,
      spreadRadius: 0.5,
      blending: THREE.AdditiveBlending,
    }),
    []
  )

  // Sparkle geometry and material (separate effect)
  const { sparkleGeometry, sparkleMaterial } = useMemo(() => {
    const count = 20
    const positions = new Float32Array(count * 3)
    const colors = new Float32Array(count * 3)
    const sizes = new Float32Array(count)

    const goldColors = [
      new THREE.Color('#ffd700'),
      new THREE.Color('#ffec8b'),
      new THREE.Color('#ffffff'),
    ]

    for (let i = 0; i < count; i++) {
      const i3 = i * 3
      const angle = (i / count) * Math.PI * 2
      const radius = 0.3 + Math.random() * 0.4

      positions[i3] = Math.cos(angle) * radius
      positions[i3 + 1] = Math.random() * 0.5
      positions[i3 + 2] = Math.sin(angle) * radius

      const color = goldColors[Math.floor(Math.random() * goldColors.length)]
      colors[i3] = color.r
      colors[i3 + 1] = color.g
      colors[i3 + 2] = color.b

      sizes[i] = 0.05 + Math.random() * 0.1
    }

    const geo = new THREE.BufferGeometry()
    geo.setAttribute('position', new THREE.BufferAttribute(positions, 3))
    geo.setAttribute('color', new THREE.BufferAttribute(colors, 3))
    geo.setAttribute('size', new THREE.BufferAttribute(sizes, 1))

    const mat = new THREE.PointsMaterial({
      size: 0.15,
      vertexColors: true,
      transparent: true,
      opacity: 1,
      sizeAttenuation: true,
      blending: THREE.AdditiveBlending,
      depthWrite: false,
    })

    return { sparkleGeometry: geo, sparkleMaterial: mat }
  }, [])

  // Animate sparkles
  useFrame((_, delta) => {
    if (!active || !sparkleRef.current) return

    sparkleTimeRef.current += delta

    const posAttr = sparkleGeometry.getAttribute(
      'position'
    ) as THREE.BufferAttribute
    const sizeAttr = sparkleGeometry.getAttribute(
      'size'
    ) as THREE.BufferAttribute
    const count = posAttr.count

    for (let i = 0; i < count; i++) {
      const baseAngle = (i / count) * Math.PI * 2
      const time = sparkleTimeRef.current

      // Spiral upward motion
      const angle = baseAngle + time * 2
      const radius = 0.3 + Math.sin(time * 3 + i) * 0.2
      const height = (time * 0.5 + i * 0.1) % 2

      posAttr.setXYZ(
        i,
        Math.cos(angle) * radius,
        height,
        Math.sin(angle) * radius
      )

      // Twinkle effect
      const twinkle = 0.5 + Math.sin(time * 10 + i * 2) * 0.5
      sizeAttr.setX(i, 0.1 * twinkle)
    }

    posAttr.needsUpdate = true
    sizeAttr.needsUpdate = true

    // Fade out sparkle material over time
    const fadeStart = 1.5
    if (sparkleTimeRef.current > fadeStart) {
      const fadeProgress = (sparkleTimeRef.current - fadeStart) / 0.5
      sparkleMaterial.opacity = Math.max(0, 1 - fadeProgress)
    }
  })

  // Cleanup
  useEffect(() => {
    return () => {
      sparkleGeometry.dispose()
      sparkleMaterial.dispose()
    }
  }, [sparkleGeometry, sparkleMaterial])

  const handleComplete = useCallback(() => {
    setActive(false)
    onComplete?.()
  }, [onComplete])

  if (!active) return null

  return (
    <group position={position}>
      {/* Rising particles */}
      <ParticleSystem
        position={[0, 0, 0]}
        config={risingConfig}
        active={active}
        onComplete={handleComplete}
      />

      {/* Sparkle ring */}
      <points
        ref={sparkleRef}
        geometry={sparkleGeometry}
        material={sparkleMaterial}
      />
    </group>
  )
}

export default React.memo(CityFoundedEffect)
