/**
 * Fog of War Shader for Nostr Nations
 *
 * This shader handles three visibility states:
 * - Unexplored (0.0): Completely hidden/black
 * - Explored (0.5): Dimmed/greyed out, shows terrain but not units
 * - Visible (1.0): Full color, shows everything
 *
 * Uses instanced rendering for efficient handling of large maps.
 */

export const fogOfWarVertexShader = /* glsl */ `
  // Per-instance attributes
  attribute vec3 instancePosition;
  attribute float instanceVisibility;

  // Varyings to pass to fragment shader
  varying float vVisibility;
  varying vec2 vUv;
  varying vec3 vWorldPosition;

  void main() {
    vVisibility = instanceVisibility;
    vUv = uv;

    // Transform position with instance offset
    vec3 transformed = position + instancePosition;
    vWorldPosition = transformed;

    gl_Position = projectionMatrix * modelViewMatrix * vec4(transformed, 1.0);
  }
`;

export const fogOfWarFragmentShader = /* glsl */ `
  uniform float uTime;
  uniform float uFogIntensity;
  uniform vec3 uUnexploredColor;
  uniform vec3 uExploredColor;
  uniform float uEdgeSoftness;

  varying float vVisibility;
  varying vec2 vUv;
  varying vec3 vWorldPosition;

  // Noise function for organic fog edges
  float hash(vec2 p) {
    return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
  }

  float noise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    f = f * f * (3.0 - 2.0 * f);

    float a = hash(i);
    float b = hash(i + vec2(1.0, 0.0));
    float c = hash(i + vec2(0.0, 1.0));
    float d = hash(i + vec2(1.0, 1.0));

    return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
  }

  // Fractal Brownian Motion for more organic noise
  float fbm(vec2 p) {
    float value = 0.0;
    float amplitude = 0.5;
    float frequency = 1.0;

    for (int i = 0; i < 4; i++) {
      value += amplitude * noise(p * frequency);
      amplitude *= 0.5;
      frequency *= 2.0;
    }

    return value;
  }

  void main() {
    // Calculate distance from hex center for edge effects
    vec2 centeredUv = vUv - 0.5;
    float distFromCenter = length(centeredUv) * 2.0;

    // Add animated noise to fog edges
    vec2 noiseCoord = vWorldPosition.xz * 0.5 + uTime * 0.1;
    float noiseValue = fbm(noiseCoord);

    // Smooth visibility transitions with noise
    float edgeNoise = noiseValue * uEdgeSoftness;

    // Determine fog state based on visibility
    // 0.0 = unexplored (black), 0.5 = explored (dimmed), 1.0 = visible (no fog)

    float unexploredThreshold = 0.25;
    float exploredThreshold = 0.75;

    float opacity = 0.0;
    vec3 fogColor = uUnexploredColor;

    if (vVisibility < unexploredThreshold) {
      // Unexplored - full black fog
      opacity = uFogIntensity;
      fogColor = uUnexploredColor;

      // Add subtle animated texture to unexplored areas
      float animatedNoise = fbm(vWorldPosition.xz * 0.3 + uTime * 0.05);
      opacity *= (0.9 + animatedNoise * 0.1);

    } else if (vVisibility < exploredThreshold) {
      // Explored - dimmed, grey fog
      float exploredFactor = (vVisibility - unexploredThreshold) / (exploredThreshold - unexploredThreshold);
      opacity = uFogIntensity * 0.6 * (1.0 - exploredFactor * 0.3);
      fogColor = mix(uUnexploredColor, uExploredColor, exploredFactor);

      // Softer edges for explored areas
      float edgeFade = smoothstep(0.8, 1.0, distFromCenter + edgeNoise * 0.2);
      opacity *= (1.0 - edgeFade * 0.3);

    } else {
      // Visible - no fog (but keep slight edge fade for transitions)
      float visibleFactor = (vVisibility - exploredThreshold) / (1.0 - exploredThreshold);
      opacity = uFogIntensity * 0.1 * (1.0 - visibleFactor);
      fogColor = uExploredColor;

      // Very soft transition at edges only
      float edgeFade = smoothstep(0.9, 1.0, distFromCenter);
      opacity *= edgeFade;
    }

    // Hex edge darkening for definition
    float hexEdge = smoothstep(0.85, 1.0, distFromCenter);
    float edgeDarkening = hexEdge * 0.15;

    // Final color with edge darkening
    vec3 finalColor = fogColor * (1.0 - edgeDarkening);

    // Discard nearly transparent fragments for performance
    if (opacity < 0.01) {
      discard;
    }

    gl_FragColor = vec4(finalColor, opacity);
  }
`;

/**
 * Shader uniforms type definition
 */
export interface FogOfWarUniforms {
  uTime: { value: number };
  uFogIntensity: { value: number };
  uUnexploredColor: { value: [number, number, number] };
  uExploredColor: { value: [number, number, number] };
  uEdgeSoftness: { value: number };
}

/**
 * Default uniform values
 */
export const defaultFogUniforms: FogOfWarUniforms = {
  uTime: { value: 0 },
  uFogIntensity: { value: 1.0 },
  uUnexploredColor: { value: [0.02, 0.02, 0.05] }, // Near black with slight blue tint
  uExploredColor: { value: [0.15, 0.15, 0.18] },   // Dark grey with slight blue
  uEdgeSoftness: { value: 0.3 },
};

/**
 * Convert hex color to RGB array for shader uniforms
 */
export function hexToRgb(hex: string): [number, number, number] {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  if (!result) {
    return [0, 0, 0];
  }
  return [
    parseInt(result[1], 16) / 255,
    parseInt(result[2], 16) / 255,
    parseInt(result[3], 16) / 255,
  ];
}
