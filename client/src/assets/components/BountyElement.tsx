import { useState } from 'react'
import { Link } from 'react-router-dom'
import {
  deleteBounty,
  listSubmissions,
  updateBounty,
  updateBountyStatus,
} from '../../api/bounties'
import { shortWallet } from '../../auth/wallet'
import { formatSol } from '../../lib/sol'
import type { Bounty } from '../../types/BountyType'
import { BountySubmissionForm } from './BountySubmissionForm'
import './BountyElement.css'

type Props = {
  bounty: Bounty
  wallet: string | null
  token: string | null
  onUpdated: (bounty: Bounty) => void
  onRemoved: (id: string) => void
}

export function BountyElement({ bounty, wallet, token, onUpdated, onRemoved }: Props) {
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const isPoster = Boolean(wallet && wallet === bounty.poster_wallet)
  const isClaimer = Boolean(wallet && wallet === bounty.claimer_wallet)
  const isTaken =
    Boolean(bounty.claimer_wallet) ||
    bounty.status === 'claimed' ||
    bounty.status === 'closed'
  const canClaim = Boolean(
    bounty.status === 'open' && !bounty.claimer_wallet && wallet && !isPoster,
  )
  const canSubmit = Boolean(bounty.status === 'claimed' && isClaimer && token && wallet)
  const canApprove = bounty.status === 'claimed' && isPoster
  const canCancel = bounty.status === 'open' && isPoster

  async function run(action: () => Promise<Bounty | void>) {
    if (!token || !wallet) return
    setBusy(true)
    setError(null)
    try {
      const result = await action()
      if (result) onUpdated(result)
    } catch (err) {
      if (err instanceof Error && err.message.includes('409')) {
        setError('This bounty was already claimed by someone else.')
      } else {
        setError(err instanceof Error ? err.message : 'Action failed')
      }
    } finally {
      setBusy(false)
    }
  }

  function onClaim() {
    void run(() =>
      updateBounty(token, bounty.id, {
        claimer_wallet: wallet!,
        status: 'claimed',
      }),
    )
  }

  function onApprove() {
    void run(() => updateBountyStatus(token, bounty.id, 'closed'))
  }

  function onCancel() {
    void run(async () => {
      await deleteBounty(token, bounty.id)
      onRemoved(bounty.id)
    })
  }

  async function refreshAfterSubmission(submissionUri?: string | null) {
    if (!token) return
    if (submissionUri) {
      const updated = await updateBounty(token, bounty.id, { submission_uri: submissionUri })
      onUpdated(updated)
    } else {
      await listSubmissions(token, bounty.id)
    }
  }

  return (
    <article className="bounty-card">
      <div className="bounty-card__top">
        <h2 className="bounty-card__title">
          <Link to={`/bounty/${bounty.id}`}>{bounty.title}</Link>
        </h2>
        <span className={`bounty-card__status bounty-card__status--${bounty.status}`}>
          {bounty.status}
        </span>
      </div>

      <p className="bounty-card__desc">
        {bounty.description?.trim() || 'No description provided.'}
      </p>

      <div className="bounty-card__meta">
        <span>{formatSol(bounty.amount_lamports)} SOL</span>
        <span title={bounty.poster_wallet}>poster {shortWallet(bounty.poster_wallet)}</span>
        {bounty.claimer_wallet ? (
          <span title={bounty.claimer_wallet}>worker {shortWallet(bounty.claimer_wallet)}</span>
        ) : null}
        <span title={bounty.pda_address}>PDA {shortWallet(bounty.pda_address)}</span>
      </div>

      {bounty.submission_uri ? (
        <p className="bounty-card__submission">
          Latest delivery:{' '}
          <a href={bounty.submission_uri} target="_blank" rel="noreferrer">
            {bounty.submission_uri}
          </a>
        </p>
      ) : null}

      {canSubmit ? (
        <BountySubmissionForm
          bountyId={bounty.id}
          token={token!}
          wallet={wallet!}
          onSubmitted={refreshAfterSubmission}
          compact
        />
      ) : null}

      <div className="bounty-card__actions">
        <Link to={`/bounty/${bounty.id}`} className="btn btn--ghost">
          View details
        </Link>
        {canClaim ? (
          <button type="button" className="btn btn--primary" disabled={busy} onClick={onClaim}>
            Claim
          </button>
        ) : null}
        {!canClaim && isTaken && !isPoster && !isClaimer ? (
          <span className="bounty-card__muted">Already claimed</span>
        ) : null}
        {canApprove ? (
          <button type="button" className="btn btn--primary" disabled={busy} onClick={onApprove}>
            Approve & close
          </button>
        ) : null}
        {canCancel ? (
          <button type="button" className="btn btn--ghost" disabled={busy} onClick={onCancel}>
            Cancel bounty
          </button>
        ) : null}
      </div>

      {error ? <p className="bounty-card__error">{error}</p> : null}
    </article>
  )
}
