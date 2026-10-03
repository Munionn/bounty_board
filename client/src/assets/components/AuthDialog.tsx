import { useEffect, useState } from 'react'
import { createPortal } from 'react-dom'
import { useLocation, useNavigate } from 'react-router-dom'
import { useAuth } from '../../auth/AuthContext'
import { getAuthNext } from '../../auth/RequireAuth'
import { getPhantom, signInWithPhantom } from '../../auth/wallet'
import './dialog.css'
import './AuthDialog.css'

export function AuthDialog() {
  const navigate = useNavigate()
  const location = useLocation()
  const { setSession } = useAuth()
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [hasPhantom, setHasPhantom] = useState(false)

  const next = getAuthNext(location.state)

  const intentCopy =
    next === '/postbounty'
      ? 'Connect your wallet to post a bounty.'
      : next === '/bountypage'
        ? 'Connect your wallet to browse bounties.'
        : 'Sign a message with your wallet to log in.'

  function close() {
    navigate('/')
  }

  useEffect(() => {
    setHasPhantom(Boolean(getPhantom()))
    const prev = document.body.style.overflow
    document.body.style.overflow = 'hidden'

    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') navigate('/')
    }

    window.addEventListener('keydown', onKey)
    return () => {
      document.body.style.overflow = prev
      window.removeEventListener('keydown', onKey)
    }
  }, [navigate])

  async function onConnect() {
    setBusy(true)
    setError(null)
    try {
      const session = await signInWithPhantom()
      setSession(session.token, session.wallet)
      navigate(next, { replace: true })
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Sign-in failed')
    } finally {
      setBusy(false)
    }
  }

  return createPortal(
    <div
      className="dialog-backdrop"
      role="presentation"
      onClick={(e) => {
        if (e.target === e.currentTarget) close()
      }}
    >
      <div
        className="dialog dialog--auth"
        role="dialog"
        aria-modal="true"
        aria-labelledby="auth-dialog-title"
      >
        <header className="dialog__header">
          <h2 id="auth-dialog-title" className="dialog__title">
            Connect wallet
          </h2>
          <button
            type="button"
            className="dialog__icon-btn"
            aria-label="Close"
            onClick={close}
          >
            ×
          </button>
        </header>

        <div className="dialog__body auth-dialog__body">
          <p className="auth-dialog__lede">
            {intentCopy} No transaction and no gas — just proof you own the address.
          </p>

          <button
            type="button"
            className="auth-wallet-btn"
            onClick={onConnect}
            disabled={busy || !hasPhantom}
          >
            <span className="auth-wallet-btn__mark" aria-hidden="true" />
            <span className="auth-wallet-btn__copy">
              <strong>{busy ? 'Waiting for Phantom…' : 'Phantom'}</strong>
              <small>{hasPhantom ? 'Browser extension' : 'Not detected — install Phantom'}</small>
            </span>
          </button>

          {!hasPhantom ? (
            <a
              className="auth-dialog__install"
              href="https://phantom.app/"
              target="_blank"
              rel="noreferrer"
            >
              Get Phantom
            </a>
          ) : null}

          {error ? <p className="auth-dialog__error">{error}</p> : null}
        </div>

        <footer className="dialog__footer">
          By continuing you agree this app may verify your wallet signature and create a profile
          for that address.
        </footer>
      </div>
    </div>,
    document.body,
  )
}
