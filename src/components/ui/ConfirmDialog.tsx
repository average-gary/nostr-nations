import React from 'react'
import Modal from './Modal'

export type ConfirmVariant = 'primary' | 'danger'

interface ConfirmDialogProps {
  isOpen: boolean
  onConfirm: () => void
  onCancel: () => void
  title: string
  message: string
  confirmText?: string
  cancelText?: string
  variant?: ConfirmVariant
}

const CONFIRM_BUTTON_CLASSES: Record<ConfirmVariant, string> = {
  primary: 'bg-secondary hover:bg-secondary-600 text-background',
  danger: 'bg-danger hover:bg-red-600 text-white',
}

/**
 * Confirmation dialog component built on top of Modal.
 * Useful for destructive actions or important confirmations.
 */
const ConfirmDialog: React.FC<ConfirmDialogProps> = ({
  isOpen,
  onConfirm,
  onCancel,
  title,
  message,
  confirmText = 'Confirm',
  cancelText = 'Cancel',
  variant = 'primary',
}) => {
  const confirmButtonClass = CONFIRM_BUTTON_CLASSES[variant]

  return (
    <Modal
      isOpen={isOpen}
      onClose={onCancel}
      title={title}
      size="sm"
      closeOnBackdrop={false}
    >
      <div className="space-y-4">
        {/* Message */}
        <p className="text-sm text-foreground-muted">{message}</p>

        {/* Actions */}
        <div className="flex justify-end gap-3 pt-2">
          <button
            onClick={onCancel}
            className="rounded bg-background px-4 py-2 text-sm font-medium text-foreground-muted transition-colors hover:bg-background-lighter hover:text-foreground"
          >
            {cancelText}
          </button>
          <button
            onClick={onConfirm}
            className={`rounded px-4 py-2 text-sm font-semibold transition-colors ${confirmButtonClass}`}
          >
            {confirmText}
          </button>
        </div>
      </div>
    </Modal>
  )
}

export default ConfirmDialog
