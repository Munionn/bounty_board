import { useCallback, useEffect, useState } from 'react'
import { Link, useNavigate, useParams } from 'react-router-dom'
import {
  deleteBounty,
  getBounty,
  listSubmissions,
  updateBounty,
  updateBountyStatus,
} from '../../api/bounties'
import { useAuth } from '../../auth/AuthContext'
import { shortWallet } from '../../auth/wallet'
import { formatSol } from '../../lib/sol'
import type { Bounty } from '../../types/BountyType'
import type { Submission } from '../../types/Submission'
import { BountyChat } from './BountyChat'
import { BountySubmissionForm } from './BountySubmissionForm'
import './BountyDetailPage.css'

export function BountyDetailPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const { token, wallet } = useAuth()

  const [bounty, setBounty] = useState<Bounty | null>(null)
  const [submissions, setSubmissions] = useState<Submission[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [busy, setBusy] = useState(false)
  const [actionError, setActionError] = useState<string | null>(null)

  const load = useCallback(async () => {
    if (!id) return
    setLoading(true)
    setError(null)
    try {
      const [bountyData, submissionData] = await Promise.all([
        getBounty(token, id),
        listSubmissions(token, id).catch(() => [] as Submission[]),
      ])
      setBounty(bountyData)
      setSubmissions(submissionData)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load bounty')
      setBounty(null)
    } finally {
      setLoading(false)
    }
  }, [id, token])

  useEffect(() => {
    void load()
  }, [load])

  const isPoster = Boolean(bounty && wallet && wallet === bounty.poster_wallet)
  const isClaimer = Boolean(bounty && wallet && wallet === bounty.claimer_wallet)
  const isTaken =
    Boolean(bounty?.claimer_wallet) ||
    bounty?.status === 'claimed' ||
    bounty?.status === 'closed'
  const canClaim = Boolean(
    bounty?.status === 'open' && !bounty.claimer_wallet && wallet && !isPoster,
  )
  const canSubmit = Boolean(bounty?.status === 'claimed' && isClaimer && token && wallet)
  const canApprove = bounty?.status === 'claimed' && isPoster
  const canCancel = bounty?.status === 'open' && isPoster
  const canChat = Boolean(
    bounty?.status === 'claimed' && wallet && (isPoster || isClaimer) && token,
  )

  async function run(action: () => Promise<Bounty | void>) {
    if (!token || !wallet) return
    setBusy(true)
    setActionError(null)
    try {
      const result = await action()
      if (result) {
        setBounty(result)
        const nextSubs = await listSubmissions(token, result.id).catch(() => [] as Submission[])
        setSubmissions(nextSubs)
      }
    } catch (err) {
      if (err instanceof Error && err.message.includes('409')) {
        setActionError('This bounty was already claimed by someone else.')
      } else {
        setActionError(err instanceof Error ? err.message : 'Action failed')
      }
    } finally {
      setBusy(false)
    }
  }

  function onClaim() {
    if (!bounty) return
    void run(() =>
      updateBounty(token, bounty.id, {
        claimer_wallet: wallet!,
        status: 'claimed',
      }),
    )
  }

  function onApprove() {
    if (!bounty) return
    void run(() => updateBountyStatus(token, bounty.id, 'closed'))
  }

  function onCancel() {
    if (!bounty) return
    void run(async () => {
      await deleteBounty(token, bounty.id)
      navigate('/bountypage', { replace: true })
    })
  }

  async function refreshAfterSubmission(submissionUri?: string | null) {
    if (!bounty || !token) return
    if (submissionUri) {
      const updated = await updateBounty(token, bounty.id, { submission_uri: submissionUri })
      setBounty(updated)
    }
    setSubmissions(await listSubmissions(token, bounty.id))
  }

  return (
    <main className="bounty-detail">
      <div className="bounty-detail__nav">
        <Link to="/bountypage" className="bounty-detail__back">
          ← Back to bounties
        </Link>
      </div>

      {loading ? <p className="bounty-detail__status">Loading…</p> : null}
      {error ? <p className="bounty-detail__error">{error}</p> : null}

      {!loading && bounty ? (
        <article className="bounty-detail__panel">
          <header className="bounty-detail__header">
            <div>
              <p className="bounty-detail__eyebrow">
                {isPoster ? 'Your bounty' : isClaimer ? 'Your assignment' : 'Bounty detail'}
              </p>
              <h1 className="bounty-detail__title">{bounty.title}</h1>
            </div>
            <span className={`bounty-detail__status-chip bounty-detail__status-chip--${bounty.status}`}>
              {bounty.status}
            </span>
          </header>

          <section className="bounty-detail__section">
            <h2>Description</h2>
            <p className="bounty-detail__description">
              {bounty.description?.trim() || 'No description provided.'}
            </p>
          </section>

          <section className="bounty-detail__section">
            <h2>Details</h2>
            <dl className="bounty-detail__facts">
              <div>
                <dt>Reward</dt>
                <dd>{formatSol(bounty.amount_lamports)} SOL</dd>
              </div>
              <div>
                <dt>Poster (employer)</dt>
                <dd title={bounty.poster_wallet}>{shortWallet(bounty.poster_wallet)}</dd>
              </div>
              <div>
                <dt>Worker (consumer)</dt>
                <dd title={bounty.claimer_wallet ?? undefined}>
                  {bounty.claimer_wallet ? shortWallet(bounty.claimer_wallet) : '—'}
                </dd>
              </div>
              <div>
                <dt>PDA</dt>
                <dd title={bounty.pda_address}>{bounty.pda_address}</dd>
              </div>
              <div>
                <dt>On-chain ID</dt>
                <dd>{bounty.on_chain_id}</dd>
              </div>
              <div>
                <dt>Created</dt>
                <dd>{new Date(bounty.created_at).toLocaleString()}</dd>
              </div>
              <div>
                <dt>Updated</dt>
                <dd>{new Date(bounty.updated_at).toLocaleString()}</dd>
              </div>
              {bounty.tx_signature ? (
                <div>
                  <dt>Tx signature</dt>
                  <dd title={bounty.tx_signature}>{shortWallet(bounty.tx_signature)}</dd>
                </div>
              ) : null}
            </dl>
          </section>

          <section className="bounty-detail__section">
            <BountyChat
              bountyId={bounty.id}
              token={token}
              wallet={wallet}
              enabled={canChat}
            />
          </section>

          <section className="bounty-detail__section">
            <h2>Updates under bounty</h2>
            {canSubmit ? (
              <BountySubmissionForm
                bountyId={bounty.id}
                token={token!}
                wallet={wallet!}
                onSubmitted={refreshAfterSubmission}
              />
            ) : null}
            {submissions.length === 0 ? (
              <p className="bounty-detail__muted">
                {canSubmit
                  ? 'No updates posted yet. Use the form above to deliver work.'
                  : 'No worker updates yet.'}
              </p>
            ) : (
              <ul className="bounty-detail__subs">
                {submissions.map((sub) => (
                  <li key={sub.id}>
                    <header className="bounty-detail__sub-meta">
                      <span title={sub.submitter_wallet}>
                        {sub.submitter_wallet === wallet ? 'You' : shortWallet(sub.submitter_wallet)}
                      </span>
                      <time dateTime={sub.created_at}>
                        {new Date(sub.created_at).toLocaleString()}
                      </time>
                    </header>
                    {sub.submission_uri ? (
                      <a href={sub.submission_uri} target="_blank" rel="noreferrer">
                        {sub.submission_uri}
                      </a>
                    ) : null}
                    {sub.note ? <p>{sub.note}</p> : null}
                  </li>
                ))}
              </ul>
            )}
          </section>

          <div className="bounty-detail__actions">
            {canClaim ? (
              <button type="button" className="btn btn--primary" disabled={busy} onClick={onClaim}>
                Claim bounty
              </button>
            ) : null}
            {!canClaim && isTaken && !isPoster && !isClaimer ? (
              <p className="bounty-detail__muted">
                This bounty is already assigned
                {bounty.claimer_wallet ? ` to ${shortWallet(bounty.claimer_wallet)}` : ''}.
              </p>
            ) : null}
            {canApprove ? (
              <button
                type="button"
                className="btn btn--primary"
                disabled={busy}
                onClick={onApprove}
              >
                Approve & close
              </button>
            ) : null}
            {canCancel ? (
              <button type="button" className="btn btn--ghost" disabled={busy} onClick={onCancel}>
                Cancel bounty
              </button>
            ) : null}
          </div>

          {actionError ? <p className="bounty-detail__error">{actionError}</p> : null}
        </article>
      ) : null}
    </main>
  )
}
