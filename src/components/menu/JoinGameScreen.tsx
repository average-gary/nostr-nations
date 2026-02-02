import React, { useState, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useIsTauri } from '@/hooks/useTauri'

// Types for ticket info returned from scanning
interface TicketInfo {
  node_id: string
  game_id: string | null
  expires_at: string | null
}

interface JoinGameScreenProps {
  onBack: () => void
  onConnected: () => void
}

type TabMode = 'scan' | 'manual'

const JoinGameScreen: React.FC<JoinGameScreenProps> = ({
  onBack,
  onConnected,
}) => {
  const isTauri = useIsTauri()

  // Tab state
  const [activeTab, setActiveTab] = useState<TabMode>('manual')

  // Manual entry state
  const [ticketInput, setTicketInput] = useState('')

  // Shared state
  const [ticketInfo, setTicketInfo] = useState<TicketInfo | null>(null)
  const [isScanning, setIsScanning] = useState(false)
  const [isConnecting, setIsConnecting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Handle QR code scan (placeholder for real camera API)
  const handleQrScan = useCallback(
    async (qrData: string) => {
      setIsScanning(true)
      setError(null)
      setTicketInfo(null)

      try {
        if (isTauri) {
          const result = await invoke<TicketInfo>('scan_qr_code', {
            qr_data: qrData,
          })
          setTicketInfo(result)
        } else {
          // Mock for development without Tauri
          await new Promise((resolve) => setTimeout(resolve, 500))
          setTicketInfo({
            node_id: 'mock-node-' + qrData.substring(0, 8),
            game_id: 'mock-game-123',
            expires_at: new Date(Date.now() + 3600000).toISOString(),
          })
        }
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err)
        setError(errorMessage)
      } finally {
        setIsScanning(false)
      }
    },
    [isTauri]
  )

  // Handle manual ticket validation
  const handleValidateTicket = async () => {
    if (!ticketInput.trim()) {
      setError('Please enter a connection ticket')
      return
    }

    setIsScanning(true)
    setError(null)
    setTicketInfo(null)

    try {
      if (isTauri) {
        const result = await invoke<TicketInfo>('scan_qr_code', {
          qr_data: ticketInput.trim(),
        })
        setTicketInfo(result)
      } else {
        // Mock for development without Tauri
        await new Promise((resolve) => setTimeout(resolve, 500))
        setTicketInfo({
          node_id: 'node-' + ticketInput.substring(0, 8),
          game_id: ticketInput.includes('game') ? 'game-456' : null,
          expires_at: new Date(Date.now() + 3600000).toISOString(),
        })
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      setError(errorMessage)
    } finally {
      setIsScanning(false)
    }
  }

  // Handle peer connection
  const handleConnect = async () => {
    if (!ticketInfo) return

    setIsConnecting(true)
    setError(null)

    try {
      const ticket = ticketInput.trim()

      if (isTauri) {
        await invoke('connect_peer', { ticket })
        onConnected()
      } else {
        // Mock for development without Tauri
        console.log('Connecting to peer with ticket:', ticket)
        await new Promise((resolve) => setTimeout(resolve, 1000))
        onConnected()
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      setError(errorMessage)
      setIsConnecting(false)
    }
  }

  // Reset state when switching tabs
  const handleTabChange = (tab: TabMode) => {
    setActiveTab(tab)
    setError(null)
    setTicketInfo(null)
    setTicketInput('')
  }

  // Format expiration time
  const formatExpiration = (isoString: string | null): string => {
    if (!isoString) return 'No expiration'

    const date = new Date(isoString)
    const now = new Date()
    const diff = date.getTime() - now.getTime()

    if (diff <= 0) return 'Expired'

    const minutes = Math.floor(diff / 60000)
    const hours = Math.floor(minutes / 60)

    if (hours > 0) {
      return `Expires in ${hours}h ${minutes % 60}m`
    }
    return `Expires in ${minutes}m`
  }

  // Truncate long strings
  const truncateString = (str: string, maxLength: number = 20): string => {
    if (str.length <= maxLength) return str
    return str.substring(0, maxLength - 3) + '...'
  }

  const renderTabs = () => (
    <div className="mb-6 flex border-b border-primary-700">
      <button
        onClick={() => handleTabChange('scan')}
        className={`
          flex-1 px-4 py-3 font-header text-lg transition-all duration-200
          ${
            activeTab === 'scan'
              ? 'border-b-2 border-secondary bg-primary-900/30 text-secondary'
              : 'text-foreground-muted hover:bg-primary-900/20 hover:text-foreground'
          }
        `}
      >
        Scan QR Code
      </button>
      <button
        onClick={() => handleTabChange('manual')}
        className={`
          flex-1 px-4 py-3 font-header text-lg transition-all duration-200
          ${
            activeTab === 'manual'
              ? 'border-b-2 border-secondary bg-primary-900/30 text-secondary'
              : 'text-foreground-muted hover:bg-primary-900/20 hover:text-foreground'
          }
        `}
      >
        Enter Ticket
      </button>
    </div>
  )

  const renderScanMode = () => (
    <div className="space-y-6">
      {/* Camera preview placeholder */}
      <div className="relative mx-auto aspect-square max-w-sm overflow-hidden rounded-lg border-2 border-primary-700 bg-background-light">
        <div className="absolute inset-0 flex flex-col items-center justify-center p-6 text-center">
          {/* Camera icon placeholder */}
          <svg
            xmlns="http://www.w3.org/2000/svg"
            className="mb-4 h-16 w-16 text-foreground-dim"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={1.5}
              d="M3 9a2 2 0 012-2h.93a2 2 0 001.664-.89l.812-1.22A2 2 0 0110.07 4h3.86a2 2 0 011.664.89l.812 1.22A2 2 0 0018.07 7H19a2 2 0 012 2v9a2 2 0 01-2 2H5a2 2 0 01-2-2V9z"
            />
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={1.5}
              d="M15 13a3 3 0 11-6 0 3 3 0 016 0z"
            />
          </svg>

          <p className="mb-2 text-foreground-muted">Camera Preview</p>
          <p className="text-sm text-foreground-dim">
            Point your camera at a QR code to scan
          </p>

          {/* Scanning corners overlay */}
          <div className="pointer-events-none absolute inset-8">
            <div className="absolute left-0 top-0 h-8 w-8 border-l-2 border-t-2 border-secondary" />
            <div className="absolute right-0 top-0 h-8 w-8 border-r-2 border-t-2 border-secondary" />
            <div className="absolute bottom-0 left-0 h-8 w-8 border-b-2 border-l-2 border-secondary" />
            <div className="absolute bottom-0 right-0 h-8 w-8 border-b-2 border-r-2 border-secondary" />
          </div>
        </div>

        {isScanning && (
          <div className="absolute inset-0 flex items-center justify-center bg-black/50">
            <div className="flex flex-col items-center gap-3">
              <span className="inline-block h-8 w-8 animate-spin rounded-full border-4 border-secondary border-t-transparent" />
              <span className="text-foreground">Scanning...</span>
            </div>
          </div>
        )}
      </div>

      <p className="text-center text-sm text-foreground-dim">
        Camera access may require permissions. If the camera doesn't appear, try
        using the "Enter Ticket" tab instead.
      </p>

      {/* Demo button for testing without real camera */}
      {!ticketInfo && (
        <div className="text-center">
          <button
            onClick={() => handleQrScan('demo-ticket-abc123')}
            disabled={isScanning}
            className="rounded-lg bg-primary-700 px-4 py-2
                       text-sm text-foreground-muted
                       transition-all duration-200
                       hover:bg-primary-600 hover:text-foreground
                       disabled:cursor-not-allowed disabled:opacity-50"
          >
            Simulate QR Scan (Demo)
          </button>
        </div>
      )}
    </div>
  )

  const renderManualMode = () => (
    <div className="space-y-6">
      <div>
        <label className="mb-2 block text-sm text-foreground-muted">
          Connection Ticket
        </label>
        <textarea
          value={ticketInput}
          onChange={(e) => setTicketInput(e.target.value)}
          placeholder="Paste the connection ticket here..."
          rows={4}
          className="w-full resize-none rounded-lg border-2 border-primary-700 bg-background-light
                     px-4 py-3 font-mono text-sm text-foreground
                     placeholder-foreground-dim transition-colors focus:border-secondary focus:outline-none"
          disabled={isScanning || isConnecting}
        />
        <p className="mt-2 text-sm text-foreground-dim">
          Ask the host for their connection ticket and paste it above.
        </p>
      </div>

      {!ticketInfo && (
        <button
          onClick={handleValidateTicket}
          disabled={!ticketInput.trim() || isScanning}
          className="flex w-full items-center justify-center gap-2 rounded-lg
                     border-2 border-secondary bg-primary px-6
                     py-3
                     font-header text-lg
                     text-foreground transition-all duration-200 hover:bg-primary-600 disabled:cursor-not-allowed disabled:opacity-50"
        >
          {isScanning ? (
            <>
              <span className="inline-block h-5 w-5 animate-spin rounded-full border-2 border-foreground border-t-transparent" />
              Validating...
            </>
          ) : (
            'Validate Ticket'
          )}
        </button>
      )}
    </div>
  )

  const renderTicketInfo = () => {
    if (!ticketInfo) return null

    return (
      <div className="mt-6 rounded-lg border-2 border-secondary bg-background-light p-6">
        <h3 className="mb-4 font-header text-lg text-secondary">
          Ticket Information
        </h3>

        <div className="space-y-3">
          <div className="flex items-start justify-between gap-4">
            <span className="text-sm text-foreground-dim">Node ID</span>
            <span
              className="break-all text-right font-mono text-sm text-foreground"
              title={ticketInfo.node_id}
            >
              {truncateString(ticketInfo.node_id, 32)}
            </span>
          </div>

          <div className="flex items-center justify-between gap-4">
            <span className="text-sm text-foreground-dim">Game ID</span>
            <span className="font-mono text-sm text-foreground">
              {ticketInfo.game_id
                ? truncateString(ticketInfo.game_id, 20)
                : 'N/A'}
            </span>
          </div>

          <div className="flex items-center justify-between gap-4">
            <span className="text-sm text-foreground-dim">Expiration</span>
            <span
              className={`font-header text-sm ${
                ticketInfo.expires_at &&
                new Date(ticketInfo.expires_at) < new Date()
                  ? 'text-danger'
                  : 'text-secondary'
              }`}
            >
              {formatExpiration(ticketInfo.expires_at)}
            </span>
          </div>
        </div>

        <button
          onClick={handleConnect}
          disabled={
            isConnecting ||
            Boolean(
              ticketInfo.expires_at &&
              new Date(ticketInfo.expires_at) < new Date()
            )
          }
          className="mt-6 flex w-full items-center justify-center gap-2 rounded-lg
                     border-2 border-secondary bg-secondary px-6
                     py-3 font-header
                     text-lg text-background
                     transition-all duration-200 hover:border-secondary-600 hover:bg-secondary-600 disabled:cursor-not-allowed disabled:opacity-50"
        >
          {isConnecting ? (
            <>
              <span className="inline-block h-5 w-5 animate-spin rounded-full border-2 border-background border-t-transparent" />
              Connecting...
            </>
          ) : (
            'Connect'
          )}
        </button>

        <button
          onClick={() => {
            setTicketInfo(null)
            setTicketInput('')
            setError(null)
          }}
          disabled={isConnecting}
          className="mt-3 w-full rounded-lg px-4 py-2 font-header text-sm
                     text-foreground-muted
                     transition-all duration-200
                     hover:bg-primary-900/30 hover:text-foreground
                     disabled:cursor-not-allowed disabled:opacity-50"
        >
          Clear and try another ticket
        </button>
      </div>
    )
  }

  return (
    <div className="fixed inset-0 z-50 flex animate-fade-in items-center justify-center bg-black/80">
      <div className="max-h-[90vh] w-full max-w-lg overflow-hidden rounded-xl border-2 border-primary-700 bg-background shadow-2xl">
        {/* Header */}
        <div className="border-b border-primary-700 bg-primary-900/50 px-6 py-4">
          <h1 className="text-center font-header text-2xl text-secondary">
            Join Game
          </h1>
        </div>

        {/* Content */}
        <div className="max-h-[calc(90vh-140px)] overflow-y-auto p-6">
          {renderTabs()}

          {error && (
            <div className="mb-6 rounded-lg border-2 border-danger bg-danger/20 px-4 py-3">
              <p className="text-sm text-danger">{error}</p>
            </div>
          )}

          {activeTab === 'scan' ? renderScanMode() : renderManualMode()}

          {renderTicketInfo()}
        </div>

        {/* Footer */}
        <div className="border-t border-primary-700 bg-primary-900/50 px-6 py-4">
          <button
            onClick={onBack}
            disabled={isConnecting}
            className="rounded-lg border-2 border-primary-700 bg-background-light px-6
                       py-2 font-header text-lg text-foreground
                       transition-all duration-200
                       hover:border-primary-500 hover:bg-background-lighter
                       disabled:cursor-not-allowed disabled:opacity-50"
          >
            Back
          </button>
        </div>
      </div>
    </div>
  )
}

export default JoinGameScreen
