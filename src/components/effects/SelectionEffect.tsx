import React, { useRef, useMemo, useEffect } from 'react'
import { useFrame } from '@react-three/fiber'
import * as THREE from 'three'

export interface SelectionEffectProps {
  position: [number, number, number]
  radius?: number
  active: boolean
  color?: string
}

/**
 * Selection highlight effect with subtle orbiting particles
 * Continuous animation while selected
 */
const SelectionEffect: React.FC<SelectionEffectProps> = ({
  position,
  radius = 0.6,
  active,
  color = '#ffd700',
}) => {
  const pointsRef = useRef<THREE.Points>(null)
  const timeRef = useRef(0)

  const particleCount = 12

  // Create geometry and material
  const { geometry, material } = useMemo(() => {
    const positions = new Float32Array(particleCount * 3)
    const colors = new Float32Array(particleCount * 3)
    const sizes = new Float32Array(particleCount)

    const baseColor = new THREE.Color(color)
    const accentColor = new THREE.Color('#ffffff')

    for (let i = 0; i < particleCount; i++) {
      const i3 = i * 3
      const angle = (i / particleCount) * Math.PI * 2

      // Initial positions in a ring
      positions[i3] = Math.cos(angle) * radius
      positions[i3 + 1] = 0.1
      positions[i3 + 2] = Math.sin(angle) * radius

      // Alternate between base color and accent
      const particleColor = i % 3 === 0 ? accentColor : baseColor
      colors[i3] = particleColor.r
      colors[i3 + 1] = particleColor.g
      colors[i3 + 2] = particleColor.b

      sizes[i] = 0.06 + (i % 2) * 0.04
    }

    const geo = new THREE.BufferGeometry()
    geo.setAttribute('position', new THREE.BufferAttribute(positions, 3))
    geo.setAttribute('color', new THREE.BufferAttribute(colors, 3))
    geo.setAttribute('size', new THREE.BufferAttribute(sizes, 1))

    const mat = new THREE.PointsMaterial({
      size: 0.1,
      vertexColors: true,
      transparent: true,
      opacity: 0.8,
      sizeAttenuation: true,
      blending: THREE.AdditiveBlending,
      depthWrite: false,
    })

    return { geometry: geo, material: mat }
  }, [color, radius])

  // Reset time when becoming active
  useEffect(() => {
    if (active) {
      timeRef.current = 0
    }
  }, [active])

  // Cleanup
  useEffect(() => {
    return () => {
      geometry.dispose()
      material.dispose()
    }
  }, [geometry, material])

  // Animation loop
  useFrame((_, delta) => {
    if (!active || !pointsRef.current) return

    timeRef.current += delta

    const posAttr = geometry.getAttribute('position') as THREE.BufferAttribute
    const sizeAttr = geometry.getAttribute('size') as THREE.BufferAttribute
    const time = timeRef.current

    for (let i = 0; i < particleCount; i++) {
      const baseAngle = (i / particleCount) * Math.PI * 2

      // Orbit around the center
      const orbitSpeed = 0.8
      const angle = baseAngle + time * orbitSpeed

      // Slight vertical oscillation
      const verticalOffset = Math.sin(time * 2 + i * 0.5) * 0.1

      // Slight radius variation for more organic feel
      const radiusVariation = radius + Math.sin(time * 1.5 + i) * 0.05

      posAttr.setXYZ(
        i,
        Math.cos(angle) * radiusVariation,
        0.1 + verticalOffset,
        Math.sin(angle) * radiusVariation
      )

      // Pulse size
      const sizePulse = 0.06 + Math.sin(time * 3 + i * 0.8) * 0.03
      sizeAttr.setX(i, sizePulse)
    }

    posAttr.needsUpdate = true
    sizeAttr.needsUpdate = true

    // Subtle opacity pulse
    material.opacity = 0.6 + Math.sin(time * 2) * 0.2
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

export default React.memo(SelectionEffect)
