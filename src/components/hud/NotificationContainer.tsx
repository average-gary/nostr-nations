import React, { useCallback, useMemo } from 'react';
import { useGameStore } from '@/stores/gameStore';
import { useTauriEvent } from '@/hooks/useTauri';
import type { Notification, NotificationPayload } from '@/types/game';
import NotificationToast from './NotificationToast';

// Maximum number of notifications visible at once
const MAX_VISIBLE_NOTIFICATIONS = 5;

// Event name from backend
const NOTIFICATION_EVENT = 'notification';

interface NotificationContainerProps {
  /**
   * Position of the notification stack on screen
   * @default 'top-right'
   */
  position?: 'top-right' | 'top-left' | 'bottom-right' | 'bottom-left';
  /**
   * Callback when a notification action is triggered
   */
  onNotificationAction?: (notification: Notification) => void;
}

/**
 * Container component that manages and displays notification toasts.
 * Listens to Tauri backend events and displays notifications in a stack.
 */
const NotificationContainer: React.FC<NotificationContainerProps> = ({
  position = 'top-right',
  onNotificationAction,
}) => {
  const notifications = useGameStore((state) => state.notifications);
  const addNotificationFromBackend = useGameStore((state) => state.addNotificationFromBackend);
  const dismissNotification = useGameStore((state) => state.dismissNotification);
  const removeNotification = useGameStore((state) => state.removeNotification);

  // Listen for backend notification events
  const handleBackendNotification = useCallback(
    (payload: NotificationPayload) => {
      addNotificationFromBackend(payload);
    },
    [addNotificationFromBackend]
  );

  useTauriEvent<NotificationPayload>(NOTIFICATION_EVENT, handleBackendNotification);

  // Get visible notifications (not dismissed, limited to MAX_VISIBLE)
  const visibleNotifications = useMemo(() => {
    return notifications
      .filter((n) => !n.dismissed)
      .slice(-MAX_VISIBLE_NOTIFICATIONS);
  }, [notifications]);

  // Handle dismiss - marks as dismissed then removes after animation
  const handleDismiss = useCallback(
    (id: string) => {
      dismissNotification(id);
      // Remove from store after animation completes
      setTimeout(() => {
        removeNotification(id);
      }, 100);
    },
    [dismissNotification, removeNotification]
  );

  // Handle notification action click
  const handleAction = useCallback(
    (notification: Notification) => {
      if (onNotificationAction) {
        onNotificationAction(notification);
      }
      // Dismiss after action
      handleDismiss(notification.id);
    },
    [handleDismiss, onNotificationAction]
  );

  // Position classes for the container
  const positionClasses = {
    'top-right': 'top-20 right-4',
    'top-left': 'top-20 left-4',
    'bottom-right': 'bottom-20 right-4',
    'bottom-left': 'bottom-20 left-4',
  };

  // Stack direction based on position
  const stackClasses = {
    'top-right': 'flex-col',
    'top-left': 'flex-col',
    'bottom-right': 'flex-col-reverse',
    'bottom-left': 'flex-col-reverse',
  };

  if (visibleNotifications.length === 0) {
    return null;
  }

  return (
    <div
      className={`
        fixed z-50 pointer-events-none
        ${positionClasses[position]}
      `}
      role="region"
      aria-label="Notifications"
      aria-live="polite"
    >
      <div className={`flex ${stackClasses[position]} gap-2`}>
        {visibleNotifications.map((notification) => (
          <div key={notification.id} className="pointer-events-auto">
            <NotificationToast
              notification={notification}
              onDismiss={handleDismiss}
              onAction={handleAction}
            />
          </div>
        ))}
      </div>
    </div>
  );
};

export default NotificationContainer;
