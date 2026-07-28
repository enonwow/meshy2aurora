/**
 * Product limits for the local Animation Studio project layer.
 *
 * These are deliberately independent from Aurora's 65,535 index-entry
 * binary boundary. The timeline DOM has a separate virtualization cap.
 */
export const ANIMATION_STUDIO_PRODUCT_LIMITS_V1 = {
  maxAuthoredClips: 64,
  maxKeyframesPerClip: 10_000,
  maxTotalKeyframes: 100_000,
  maxTimelineDomMarkers: 2_000,
  maxDurationSeconds: 86_400,
  maxAbsoluteTrackValue: 1_000_000,
} as const;
