import { useEffect, useState } from 'react'
import type { FormEvent } from 'react'
import { createPortal } from 'react-dom'
import { useNavigate } from 'react-router-dom'
import { createBounty } from '../../api/bounties'
import { useAuth } from '../../auth/AuthContext'
import { makeOnChainId, makePendingPda, solToLamports } from '../../lib/sol'
import './dialog.css'

export function PostBountyForm() {
  const navigate = useNavigate()
  const { token, wallet } = useAuth()
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)

  function close() {
    navigate('/bountypage')
  }

  useEffect(() => {
    const prev = document.body.style.overflow
    document.body.style.overflow = 'hidden'

    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') navigate('/bountypage')
    }

    window.addEventListener('keydown', onKey)
    return () => {
      document.body.style.overflow = prev
      window.removeEventListener('keydown', onKey)
    }
  }, [navigate])

  async function onSubmit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault()
    setError(null)

    if (!token || !wallet) {
      setError('Connect your wallet first.')
      return
    }

    const form = e.currentTarget
    const title = (form.elements.namedItem('title') as HTMLInputElement).value.trim()
    const description = (form.elements.namedItem('description') as HTMLTextAreaElement).value.trim()
    const amountSol = Number((form.elements.namedItem('amount') as HTMLInputElement).value)

    if (!title) {
      setError('Title is required.')
      return
    }
    if (!Number.isFinite(amountSol) || amountSol <= 0) {
      setError('Enter a reward greater than 0 SOL.')
      return
    }

    setBusy(true)
    try {
      await createBounty(token, {
        on_chain_id: makeOnChainId(),
        poster_wallet: wallet,
        pda_address: makePendingPda(wallet),
        title,
        description,
        amount_lamports: solToLamports(amountSol),
        tx_signature: null,
      })
      navigate('/bountypage', { replace: true })
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create bounty')
    } finally {
      setBusy(false)
    }
  }

  return createPortal(
    <div
      className="dialog-backdrop"
      role="presentation"
      onClick={(e) => {
        if (e.target === e.currentTarget && !busy) close()
      }}
    >
      <div
        className="dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="post-bounty-title"
      >
        <header className="dialog__header">
          <h2 id="post-bounty-title" className="dialog__title">
            Post a bounty
          </h2>
          <button
            type="button"
            className="dialog__icon-btn"
            aria-label="Close"
            onClick={close}
            disabled={busy}
          >
            ×
          </button>
        </header>

        <form className="dialog__body" onSubmit={onSubmit}>
          <div className="dialog__field">
            <label htmlFor="bounty-title">Title</label>
            <input
              id="bounty-title"
              name="title"
              type="text"
              required
              placeholder="What needs to be done?"
              autoFocus
              disabled={busy}
            />
          </div>

          <div className="dialog__field">
            <label htmlFor="bounty-description">Description</label>
            <textarea
              id="bounty-description"
              name="description"
              rows={5}
              placeholder="Spell out scope, acceptance criteria, and links."
              disabled={busy}
            />
          </div>

          <div className="dialog__field">
            <label htmlFor="bounty-amount">Reward (SOL)</label>
            <input
              id="bounty-amount"
              name="amount"
              type="number"
              min="0.01"
              step="0.01"
              required
              placeholder="0.5"
              disabled={busy}
            />
          </div>

          {error ? <p className="dialog__error">{error}</p> : null}

          <div className="dialog__actions">
            <button type="button" className="btn btn--ghost" onClick={close} disabled={busy}>
              Cancel
            </button>
            <button type="submit" className="btn btn--primary" disabled={busy}>
              {busy ? 'Posting…' : 'Post bounty'}
            </button>
          </div>
        </form>

        <footer className="dialog__footer">
          Saved off-chain for now. On-chain escrow (Devnet) can be attached after you confirm a
          wallet transaction.
        </footer>
      </div>
    </div>,
    document.body,
  )
}
