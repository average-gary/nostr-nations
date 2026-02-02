import React from 'react'

export type ProgressVariant = 'primary' | 'success' | 'warning' | 'danger'

interface ProgressBarProps {
  progress: number
  label?: string
  showPercentage?: boolean
  variant?: ProgressVariant
  className?: string
  animated?: boolean
}

const VARIANT_CLASSES: Record<ProgressVariant, string> = {
  primary: 'bg-secondary',
  success: 'bg-success',
  warning: 'bg-warning',
  danger: 'bg-danger',
}

/**
 * Progress bar component with animated fill and color variants.
 */
const ProgressBar: React.FC<ProgressBarProps> = ({
  progress,
  label,
  showPercentage = false,
  variant = 'primary',
  className = '',
  animated = true,
}) => {
  // Clamp progress between 0 and 100
  const clampedProgress = Math.max(0, Math.min(100, progress))
  const variantClass = VARIANT_CLASSES[variant]

  return (
    <div className={`w-full ${className}`}>
      {/* Label and percentage header */}
      {(label || showPercentage) && (
        <div className="mb-1 flex justify-between text-sm">
          {label && <span className="text-foreground-muted">{label}</span>}
          {showPercentage && (
            <span className="font-mono text-foreground">
              {Math.round(clampedProgress)}%
            </span>
          )}
        </div>
      )}

      {/* Progress track */}
      <div
        className="h-2 overflow-hidden rounded bg-background"
        role="progressbar"
        aria-valuenow={clampedProgress}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-label={label ?? 'Progress'}
      >
        {/* Progress fill */}
        <div
          className={`
            h-full
            ${variantClass}
            ${animated ? 'transition-all duration-300 ease-out' : ''}
          `}
          style={{ width: `${clampedProgress}%` }}
        />
      </div>
    </div>
  )
}

export default ProgressBar
