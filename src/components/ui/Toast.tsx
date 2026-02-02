import React, { useEffect, useState, useCallback } from 'react'

export type ToastType = 'info' | 'success' | 'warning' | 'error'

interface ToastProps {
  type: ToastType
  message: string
  duration?: number
  onClose: () => void
  title?: string
}

const TOAST_STYLES: Record<
  ToastType,
  {
    bgClass: string
    borderClass: string
    iconBgClass: string
  }
> = {
  info: {
    bgClass: 'bg-blue-900/90',
    borderClass: 'border-blue-500',
    iconBgClass: 'bg-blue-600',
  },
  success: {
    bgClass: 'bg-green-900/90',
    borderClass: 'border-green-500',
    iconBgClass: 'bg-green-600',
  },
  warning: {
    bgClass: 'bg-yellow-900/90',
    borderClass: 'border-yellow-500',
    iconBgClass: 'bg-yellow-600',
  },
  error: {
    bgClass: 'bg-red-900/90',
    borderClass: 'border-red-500',
    iconBgClass: 'bg-red-600',
  },
}

const DEFAULT_DURATION = 5000

/**
 * Toast notification component with auto-dismiss and manual close.
 */
const Toast: React.FC<ToastProps> = ({
  type,
  message,
  duration = DEFAULT_DURATION,
  onClose,
  title,
}) => {
  const [isExiting, setIsExiting] = useState(false)
  const [progress, setProgress] = useState(100)

  const styles = TOAST_STYLES[type]
  const shouldAutoDismiss = duration > 0

  const handleClose = useCallback(() => {
    setIsExiting(true)
    setTimeout(() => {
      onClose()
    }, 300)
  }, [onClose])

  // Auto-dismiss timer
  useEffect(() => {
    if (!shouldAutoDismiss) return

    const startTime = Date.now()
    const endTime = startTime + duration

    const progressInterval = setInterval(() => {
      const now = Date.now()
      const remaining = Math.max(0, endTime - now)
      const progressPercent = (remaining / duration) * 100
      setProgress(progressPercent)

      if (remaining <= 0) {
        clearInterval(progressInterval)
      }
    }, 50)

    const dismissTimer = setTimeout(() => {
      handleClose()
    }, duration)

    return () => {
      clearInterval(progressInterval)
      clearTimeout(dismissTimer)
    }
  }, [duration, handleClose, shouldAutoDismiss])

  return (
    <div
      className={`
        relative w-80 overflow-hidden rounded-lg border-l-4 shadow-lg backdrop-blur-sm
        ${styles.bgClass} ${styles.borderClass}
        ${isExiting ? 'animate-slide-out-right' : 'animate-slide-in-right'}
        transition-all duration-200
      `}
      role="alert"
    >
      <div className="flex items-start gap-3 p-3">
        {/* Icon */}
        <div
          className={`
            flex h-8 w-8 shrink-0 items-center justify-center rounded-full
            ${styles.iconBgClass}
          `}
        >
          <ToastIcon type={type} />
        </div>

        {/* Content */}
        <div className="min-w-0 flex-1">
          {title && (
            <h4 className="truncate font-header text-sm font-semibold text-white">
              {title}
            </h4>
          )}
          <p
            className={`line-clamp-3 text-xs text-gray-200 ${title ? 'mt-0.5' : ''}`}
          >
            {message}
          </p>
        </div>

        {/* Close button */}
        <button
          onClick={handleClose}
          className="shrink-0 rounded p-1 text-gray-400 transition-colors hover:bg-white/10 hover:text-white"
          aria-label="Close notification"
        >
          <CloseIcon />
        </button>
      </div>

      {/* Progress bar for auto-dismiss */}
      {shouldAutoDismiss && (
        <div className="absolute bottom-0 left-0 h-0.5 w-full bg-white/20">
          <div
            className="h-full bg-white/50 transition-all duration-100 ease-linear"
            style={{ width: `${progress}%` }}
          />
        </div>
      )}
    </div>
  )
}

interface ToastIconProps {
  type: ToastType
}

const ToastIcon: React.FC<ToastIconProps> = ({ type }) => {
  switch (type) {
    case 'info':
      return (
        <svg
          className="h-4 w-4 text-white"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
          />
        </svg>
      )
    case 'success':
      return (
        <svg
          className="h-4 w-4 text-white"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M5 13l4 4L19 7"
          />
        </svg>
      )
    case 'warning':
      return (
        <svg
          className="h-4 w-4 text-white"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
          />
        </svg>
      )
    case 'error':
      return (
        <svg
          className="h-4 w-4 text-white"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M6 18L18 6M6 6l12 12"
          />
        </svg>
      )
  }
}

const CloseIcon: React.FC = () => (
  <svg
    className="h-4 w-4"
    fill="none"
    viewBox="0 0 24 24"
    stroke="currentColor"
  >
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      d="M6 18L18 6M6 6l12 12"
    />
  </svg>
)

export default Toast
