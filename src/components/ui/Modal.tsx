import React, { useEffect, useRef, useCallback } from 'react'

export type ModalSize = 'sm' | 'md' | 'lg' | 'full'

interface ModalProps {
  isOpen: boolean
  onClose: () => void
  title?: string
  size?: ModalSize
  closeOnBackdrop?: boolean
  closeOnEsc?: boolean
  children: React.ReactNode
  className?: string
}

const SIZE_CLASSES: Record<ModalSize, string> = {
  sm: 'max-w-sm',
  md: 'max-w-md',
  lg: 'max-w-2xl',
  full: 'max-w-full mx-4',
}

/**
 * Reusable modal component with backdrop, ESC key support, and focus trap.
 */
const Modal: React.FC<ModalProps> = ({
  isOpen,
  onClose,
  title,
  size = 'md',
  closeOnBackdrop = true,
  closeOnEsc = true,
  children,
  className = '',
}) => {
  const modalRef = useRef<HTMLDivElement>(null)
  const previousActiveElement = useRef<HTMLElement | null>(null)

  // Handle ESC key
  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      if (closeOnEsc && event.key === 'Escape') {
        onClose()
      }
    },
    [closeOnEsc, onClose]
  )

  // Focus trap
  const handleTabKey = useCallback((event: KeyboardEvent): void => {
    if (event.key !== 'Tab' || !modalRef.current) return

    const focusableElements = modalRef.current.querySelectorAll<HTMLElement>(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    )

    const firstElement = focusableElements[0]
    const lastElement = focusableElements[focusableElements.length - 1]

    if (event.shiftKey && document.activeElement === firstElement) {
      event.preventDefault()
      lastElement?.focus()
    } else if (!event.shiftKey && document.activeElement === lastElement) {
      event.preventDefault()
      firstElement?.focus()
    }
  }, [])

  // Setup and cleanup
  useEffect(() => {
    if (!isOpen) return

    // Store the currently focused element
    previousActiveElement.current = document.activeElement as HTMLElement

    // Add event listeners
    document.addEventListener('keydown', handleKeyDown)
    document.addEventListener('keydown', handleTabKey)

    // Prevent body scroll
    document.body.style.overflow = 'hidden'

    // Focus the modal
    modalRef.current?.focus()

    return () => {
      document.removeEventListener('keydown', handleKeyDown)
      document.removeEventListener('keydown', handleTabKey)
      document.body.style.overflow = ''

      // Restore focus to previous element
      previousActiveElement.current?.focus()
    }
  }, [isOpen, handleKeyDown, handleTabKey])

  if (!isOpen) return null

  const sizeClass = SIZE_CLASSES[size]

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      role="dialog"
      aria-modal="true"
      aria-labelledby={title ? 'modal-title' : undefined}
    >
      {/* Backdrop */}
      <div
        className="absolute inset-0 animate-fade-in bg-black/70 backdrop-blur-sm"
        onClick={closeOnBackdrop ? onClose : undefined}
        aria-hidden="true"
      />

      {/* Modal content */}
      <div
        ref={modalRef}
        tabIndex={-1}
        className={`
          relative w-full ${sizeClass}
          animate-scale-in rounded-lg border border-primary-700 bg-background-light
          shadow-2xl
          ${className}
        `}
      >
        {/* Header */}
        {title && (
          <div className="flex items-center justify-between border-b border-primary-700 px-4 py-3">
            <h2
              id="modal-title"
              className="font-header text-lg font-semibold text-foreground"
            >
              {title}
            </h2>
            <button
              onClick={onClose}
              className="rounded p-1 text-foreground-muted transition-colors hover:bg-background-lighter hover:text-foreground"
              aria-label="Close modal"
            >
              <CloseIcon />
            </button>
          </div>
        )}

        {/* Body */}
        <div className={`p-4 ${!title ? 'pt-10' : ''}`}>
          {/* Close button when no title */}
          {!title && (
            <button
              onClick={onClose}
              className="absolute right-2 top-2 rounded p-1 text-foreground-muted transition-colors hover:bg-background-lighter hover:text-foreground"
              aria-label="Close modal"
            >
              <CloseIcon />
            </button>
          )}
          {children}
        </div>
      </div>
    </div>
  )
}

const CloseIcon: React.FC = () => (
  <svg
    className="h-5 w-5"
    fill="none"
    stroke="currentColor"
    viewBox="0 0 24 24"
  >
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      d="M6 18L18 6M6 6l12 12"
    />
  </svg>
)

export default Modal
