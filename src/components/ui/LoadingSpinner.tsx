import React from 'react'

export type SpinnerSize = 'sm' | 'md' | 'lg'

interface LoadingSpinnerProps {
  size?: SpinnerSize
  color?: string
  className?: string
}

const SIZE_CLASSES: Record<SpinnerSize, string> = {
  sm: 'w-4 h-4 border-2',
  md: 'w-8 h-8 border-3',
  lg: 'w-16 h-16 border-4',
}

/**
 * Reusable loading spinner component with customizable size and color.
 */
const LoadingSpinner: React.FC<LoadingSpinnerProps> = ({
  size = 'md',
  color,
  className = '',
}) => {
  const sizeClass = SIZE_CLASSES[size]
  const borderColor = color ?? 'border-t-secondary'

  return (
    <div
      className={`
        ${sizeClass}
        border-primary-600
        ${borderColor}
        animate-spin
        rounded-full
        ${className}
      `}
      role="status"
      aria-label="Loading"
    >
      <span className="sr-only">Loading...</span>
    </div>
  )
}

export default LoadingSpinner
