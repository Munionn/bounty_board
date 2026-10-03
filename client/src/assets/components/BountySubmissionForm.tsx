import { useState } from 'react'
import type { FormEvent } from 'react'
import { createSubmission } from '../../api/bounties'

type Props = {
  bountyId: string
  token: string
  wallet: string
  onSubmitted: (submissionUri?: string | null) => Promise<void> | void
  compact?: boolean
}

export function BountySubmissionForm({
  bountyId,
  token,
  wallet,
  onSubmitted,
  compact = false,
}: Props) {
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)

  async function onSubmit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault()

    const form = e.currentTarget
    const submission_uri = (form.elements.namedItem('uri') as HTMLInputElement).value.trim()
    const note = (form.elements.namedItem('note') as HTMLTextAreaElement).value.trim()

    if (!submission_uri && !note) {
      setError('Add a delivery link or write a note for the employer.')
      return
    }

    setBusy(true)
    setError(null)
    try {
      await createSubmission(token, bountyId, {
        submitter_wallet: wallet,
        submission_uri: submission_uri || undefined,
        note: note || undefined,
      })

      form.reset()
      await onSubmitted(submission_uri || null)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Could not post update')
    } finally {
      setBusy(false)
    }
  }

  return (
    <form
      className={compact ? 'bounty-card__submit' : 'bounty-detail__submit'}
      onSubmit={onSubmit}
    >
      <p className={compact ? 'bounty-card__muted' : 'bounty-detail__muted'}>
        Post your delivery link or write an update for the employer.
      </p>
      <label>
        Delivery link (optional)
        <input
          name="uri"
          type="url"
          placeholder="https://… or ipfs://…"
          disabled={busy}
        />
      </label>
      <label>
        Message for employer
        <textarea
          name="note"
          rows={compact ? 2 : 3}
          placeholder="Describe what you delivered or ask a question…"
          disabled={busy}
        />
      </label>
      <button type="submit" className="btn btn--primary" disabled={busy}>
        {busy ? 'Posting…' : 'Post under bounty'}
      </button>
      {error ? (
        <p className={compact ? 'bounty-card__error' : 'bounty-detail__error'}>{error}</p>
      ) : null}
    </form>
  )
}
