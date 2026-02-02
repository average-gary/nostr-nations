import React, { useEffect, useState, useCallback } from 'react';
import type { Notification, NotificationType } from '@/types/game';

interface NotificationToastProps {
  notification: Notification;
  onDismiss: (id: string) => void;
  onAction?: (notification: Notification) => void;
}

/**
 * Styling configuration for each notification type
 */
const NOTIFICATION_STYLES: Record<
  NotificationType,
  {
    bgClass: string;
    borderClass: string;
    iconBgClass: string;
    icon: React.ReactNode;
  }
> = {
  info: {
    bgClass: 'bg-blue-900/90',
    borderClass: 'border-blue-500',
    iconBgClass: 'bg-blue-600',
    icon: <InfoIcon />,
  },
  success: {
    bgClass: 'bg-green-900/90',
    borderClass: 'border-green-500',
    iconBgClass: 'bg-green-600',
    icon: <SuccessIcon />,
  },
  warning: {
    bgClass: 'bg-yellow-900/90',
    borderClass: 'border-yellow-500',
    iconBgClass: 'bg-yellow-600',
    icon: <WarningIcon />,
  },
  error: {
    bgClass: 'bg-red-900/90',
    borderClass: 'border-red-500',
    iconBgClass: 'bg-red-600',
    icon: <ErrorIcon />,
  },
  achievement: {
    bgClass: 'bg-purple-900/90',
    borderClass: 'border-purple-500',
    iconBgClass: 'bg-purple-600',
    icon: <AchievementIcon />,
  },
  diplomacy: {
    bgClass: 'bg-indigo-900/90',
    borderClass: 'border-indigo-500',
    iconBgClass: 'bg-indigo-600',
    icon: <DiplomacyIcon />,
  },
  research: {
    bgClass: 'bg-cyan-900/90',
    borderClass: 'border-cyan-400',
    iconBgClass: 'bg-cyan-600',
    icon: <ResearchIcon />,
  },
  production: {
    bgClass: 'bg-amber-900/90',
    borderClass: 'border-amber-500',
    iconBgClass: 'bg-amber-600',
    icon: <ProductionIcon />,
  },
  combat: {
    bgClass: 'bg-rose-900/90',
    borderClass: 'border-rose-500',
    iconBgClass: 'bg-rose-600',
    icon: <CombatIcon />,
  },
};

// Default duration in milliseconds
const DEFAULT_DURATION = 5000;

const NotificationToast: React.FC<NotificationToastProps> = ({
  notification,
  onDismiss,
  onAction,
}) => {
  const [isExiting, setIsExiting] = useState(false);
  const [progress, setProgress] = useState(100);

  const styles = NOTIFICATION_STYLES[notification.type] || NOTIFICATION_STYLES.info;
  const duration = notification.durationMs ?? DEFAULT_DURATION;
  const shouldAutoDismiss = duration > 0;

  const handleDismiss = useCallback(() => {
    setIsExiting(true);
    // Wait for exit animation before removing
    setTimeout(() => {
      onDismiss(notification.id);
    }, 300);
  }, [notification.id, onDismiss]);

  const handleClick = useCallback(() => {
    if (notification.action && onAction) {
      onAction(notification);
    }
  }, [notification, onAction]);

  // Auto-dismiss timer
  useEffect(() => {
    if (!shouldAutoDismiss) return;

    const startTime = Date.now();
    const endTime = startTime + duration;

    // Progress bar update interval
    const progressInterval = setInterval(() => {
      const now = Date.now();
      const remaining = Math.max(0, endTime - now);
      const progressPercent = (remaining / duration) * 100;
      setProgress(progressPercent);

      if (remaining <= 0) {
        clearInterval(progressInterval);
      }
    }, 50);

    // Dismiss timer
    const dismissTimer = setTimeout(() => {
      handleDismiss();
    }, duration);

    return () => {
      clearInterval(progressInterval);
      clearTimeout(dismissTimer);
    };
  }, [duration, handleDismiss, shouldAutoDismiss]);

  return (
    <div
      className={`
        relative w-80 overflow-hidden rounded-lg border-l-4 shadow-lg backdrop-blur-sm
        ${styles.bgClass} ${styles.borderClass}
        ${isExiting ? 'animate-toast-exit' : 'animate-toast-enter'}
        ${notification.action ? 'cursor-pointer hover:brightness-110' : ''}
        transition-all duration-200
      `}
      onClick={notification.action ? handleClick : undefined}
      role={notification.action ? 'button' : 'alert'}
      tabIndex={notification.action ? 0 : undefined}
      onKeyDown={
        notification.action
          ? (e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                handleClick();
              }
            }
          : undefined
      }
    >
      <div className="flex items-start gap-3 p-3">
        {/* Icon */}
        <div
          className={`
            flex h-8 w-8 shrink-0 items-center justify-center rounded-full
            ${styles.iconBgClass}
          `}
        >
          {styles.icon}
        </div>

        {/* Content */}
        <div className="min-w-0 flex-1">
          <h4 className="font-header text-sm font-semibold text-white truncate">
            {notification.title}
          </h4>
          <p className="mt-0.5 text-xs text-gray-200 line-clamp-2">
            {notification.message}
          </p>
          {notification.action && (
            <span className="mt-1 inline-block text-xs font-medium text-secondary-300 hover:text-secondary-200">
              {notification.action.label}
            </span>
          )}
        </div>

        {/* Dismiss button */}
        <button
          onClick={(e) => {
            e.stopPropagation();
            handleDismiss();
          }}
          className="shrink-0 rounded p-1 text-gray-400 hover:bg-white/10 hover:text-white transition-colors"
          aria-label="Dismiss notification"
        >
          <CloseIcon />
        </button>
      </div>

      {/* Progress bar for auto-dismiss */}
      {shouldAutoDismiss && (
        <div className="absolute bottom-0 left-0 h-0.5 bg-white/20 w-full">
          <div
            className="h-full bg-white/50 transition-all duration-100 ease-linear"
            style={{ width: `${progress}%` }}
          />
        </div>
      )}
    </div>
  );
};

// Icon components
function InfoIcon() {
  return (
    <svg className="h-4 w-4 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
      />
    </svg>
  );
}

function SuccessIcon() {
  return (
    <svg className="h-4 w-4 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M5 13l4 4L19 7"
      />
    </svg>
  );
}

function WarningIcon() {
  return (
    <svg className="h-4 w-4 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
      />
    </svg>
  );
}

function ErrorIcon() {
  return (
    <svg className="h-4 w-4 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M6 18L18 6M6 6l12 12"
      />
    </svg>
  );
}

function AchievementIcon() {
  return (
    <svg className="h-4 w-4 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.143L13 21l-2.286-6.857L5 12l5.714-2.143L13 3z"
      />
    </svg>
  );
}

function DiplomacyIcon() {
  return (
    <svg className="h-4 w-4 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"
      />
    </svg>
  );
}

function ResearchIcon() {
  return (
    <svg className="h-4 w-4 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z"
      />
    </svg>
  );
}

function ProductionIcon() {
  return (
    <svg className="h-4 w-4 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z"
      />
    </svg>
  );
}

function CombatIcon() {
  return (
    <svg className="h-4 w-4 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M12 21v-1m4.95-4.95l.707.707M3 12l2.5-2.5L8 12l-2.5 2.5L3 12zm18 0l-2.5-2.5L16 12l2.5 2.5L21 12z"
      />
    </svg>
  );
}

function CloseIcon() {
  return (
    <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
    </svg>
  );
}

export default NotificationToast;
