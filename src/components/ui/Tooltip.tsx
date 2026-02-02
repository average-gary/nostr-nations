import React, { useState, useRef, useCallback, useEffect } from 'react'

export type TooltipPosition = 'top' | 'bottom' | 'left' | 'right'

interface TooltipProps {
  content: React.ReactNode
  position?: TooltipPosition
  delay?: number
  children: React.ReactNode
  className?: string
}

const POSITION_CLASSES: Record<TooltipPosition, string> = {
  top: 'bottom-full left-1/2 -translate-x-1/2 mb-2',
  bottom: 'top-full left-1/2 -translate-x-1/2 mt-2',
  left: 'right-full top-1/2 -translate-y-1/2 mr-2',
  right: 'left-full top-1/2 -translate-y-1/2 ml-2',
}

const ARROW_CLASSES: Record<TooltipPosition, string> = {
  top: 'top-full left-1/2 -translate-x-1/2 border-t-background-lighter border-x-transparent border-b-transparent',
  bottom:
    'bottom-full left-1/2 -translate-x-1/2 border-b-background-lighter border-x-transparent border-t-transparent',
  left: 'left-full top-1/2 -translate-y-1/2 border-l-background-lighter border-y-transparent border-r-transparent',
  right:
    'right-full top-1/2 -translate-y-1/2 border-r-background-lighter border-y-transparent border-l-transparent',
}

/**
 * Hover tooltip component with configurable position and delay.
 */
const Tooltip: React.FC<TooltipProps> = ({
  content,
  position = 'top',
  delay = 200,
  children,
  className = '',
}) => {
  const [isVisible, setIsVisible] = useState(false)
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const containerRef = useRef<HTMLDivElement>(null)

  const showTooltip = useCallback(() => {
    timeoutRef.current = setTimeout(() => {
      setIsVisible(true)
    }, delay)
  }, [delay])

  const hideTooltip = useCallback(() => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current)
      timeoutRef.current = null
    }
    setIsVisible(false)
  }, [])

  // Cleanup timeout on unmount
  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current)
      }
    }
  }, [])

  const positionClass = POSITION_CLASSES[position]
  const arrowClass = ARROW_CLASSES[position]

  return (
    <div
      ref={containerRef}
      className={`relative inline-block ${className}`}
      onMouseEnter={showTooltip}
      onMouseLeave={hideTooltip}
      onFocus={showTooltip}
      onBlur={hideTooltip}
    >
      {children}

      {/* Tooltip */}
      {isVisible && (
        <div
          className={`
            absolute z-50 ${positionClass}
            animate-fade-in whitespace-nowrap rounded bg-background-lighter px-2
            py-1 text-xs text-foreground
            shadow-lg
          `}
          role="tooltip"
        >
          {content}
          {/* Arrow */}
          <div
            className={`absolute border-4 ${arrowClass}`}
            aria-hidden="true"
          />
        </div>
      )}
    </div>
  )
}

export default Tooltip
