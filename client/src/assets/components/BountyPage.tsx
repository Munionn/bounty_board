import { useCallback, useEffect, useRef, useState } from 'react'
import { listBounties } from '../../api/bounties'
import { useAuth } from '../../auth/AuthContext'
import { AuthGateLink } from '../../auth/RequireAuth'
import type { Bounty, BountyStatus } from '../../types/BountyType'
import { BountyElement } from './BountyElement'
import './BountyPage.css'

type Filter = BountyStatus | 'all'

const PAGE_SIZE = 5

const FILTERS: { id: Filter; label: string }[] = [
  { id: 'all', label: 'All' },
  { id: 'open', label: 'Open' },
  { id: 'claimed', label: 'Claimed' },
  { id: 'closed', label: 'Closed' },
]

export function BountyPage() {
  const { token, wallet } = useAuth()
  const [bounties, setBounties] = useState<Bounty[]>([])
  const [filter, setFilter] = useState<Filter>('all')
  const [offset, setOffset] = useState(0)
  const [hasMore, setHasMore] = useState(true)
  const [initialLoading, setInitialLoading] = useState(true)
  const [loadingMore, setLoadingMore] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const requestId = useRef(0)
  const sentinelRef = useRef<HTMLDivElement | null>(null)

  const fetchPage = useCallback(
    async (pageOffset: number, replace: boolean) => {
      const id = ++requestId.current
      if (replace) {
        setInitialLoading(true)
        setError(null)
      } else {
        setLoadingMore(true)
      }

      try {
        const data = await listBounties(token, {
          status: filter,
          limit: PAGE_SIZE,
          offset: pageOffset,
        })
        if (id !== requestId.current) return

        setBounties((prev) => (replace ? data : [...prev, ...data]))
        setOffset(pageOffset + data.length)
        setHasMore(data.length === PAGE_SIZE)
      } catch (err) {
        if (id !== requestId.current) return
        setError(err instanceof Error ? err.message : 'Failed to load bounties')
        if (replace) setBounties([])
      } finally {
        if (id === requestId.current) {
          setInitialLoading(false)
          setLoadingMore(false)
        }
      }
    },
    [token, filter],
  )

  useEffect(() => {
    setOffset(0)
    setHasMore(true)
    void fetchPage(0, true)
  }, [fetchPage])

  useEffect(() => {
    const node = sentinelRef.current
    if (!node || !hasMore || initialLoading) return

    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0]?.isIntersecting && hasMore && !loadingMore && !initialLoading) {
          void fetchPage(offset, false)
        }
      },
      { rootMargin: '200px' },
    )

    observer.observe(node)
    return () => observer.disconnect()
  }, [fetchPage, hasMore, initialLoading, loadingMore, offset])

  function onUpdated(next: Bounty) {
    setBounties((prev) => prev.map((b) => (b.id === next.id ? next : b)))
  }

  function onRemoved(id: string) {
    setBounties((prev) => prev.filter((b) => b.id !== id))
  }

  function onRefresh() {
    setOffset(0)
    setHasMore(true)
    void fetchPage(0, true)
  }

  return (
    <main className="bounty-page">
      <header className="bounty-page__header">
        <div className="bounty-page__heading">
          <h1 className="bounty-page__title">Bounties</h1>
          <p className="bounty-page__lede">
            Claim open work, submit results, and close payouts when done.
          </p>
        </div>
        <AuthGateLink to="/postbounty" className="btn btn--primary">
          Post a bounty
        </AuthGateLink>
      </header>

      <div className="bounty-page__toolbar" role="tablist" aria-label="Filter by status">
        {FILTERS.map((item) => (
          <button
            key={item.id}
            type="button"
            role="tab"
            aria-selected={filter === item.id}
            className={
              filter === item.id
                ? 'bounty-page__chip bounty-page__chip--active'
                : 'bounty-page__chip'
            }
            onClick={() => setFilter(item.id)}
          >
            {item.label}
          </button>
        ))}
        <button type="button" className="bounty-page__chip" onClick={onRefresh}>
          Refresh
        </button>
      </div>

      {error ? <p className="bounty-page__error">{error}</p> : null}

      {initialLoading ? (
        <ul className="bounty-page__list" aria-busy="true" aria-label="Loading bounties">
          {Array.from({ length: PAGE_SIZE }).map((_, i) => (
            <li key={i}>
              <div className="bounty-skeleton">
                <div className="bounty-skeleton__line bounty-skeleton__line--title" />
                <div className="bounty-skeleton__line" />
                <div className="bounty-skeleton__line bounty-skeleton__line--short" />
                <div className="bounty-skeleton__meta">
                  <span />
                  <span />
                  <span />
                </div>
              </div>
            </li>
          ))}
        </ul>
      ) : null}

      {!initialLoading && !error && bounties.length === 0 ? (
        <p className="bounty-page__status">No bounties in this filter. Post one to get started.</p>
      ) : null}

      {!initialLoading && bounties.length > 0 ? (
        <ul className="bounty-page__list">
          {bounties.map((bounty) => (
            <li key={bounty.id}>
              <BountyElement
                bounty={bounty}
                wallet={wallet}
                token={token}
                onUpdated={onUpdated}
                onRemoved={onRemoved}
              />
            </li>
          ))}
        </ul>
      ) : null}

      <div ref={sentinelRef} className="bounty-page__sentinel" aria-hidden="true" />

      {loadingMore ? (
        <div className="bounty-page__more" aria-live="polite">
          <div className="bounty-skeleton bounty-skeleton--compact">
            <div className="bounty-skeleton__line bounty-skeleton__line--title" />
            <div className="bounty-skeleton__line bounty-skeleton__line--short" />
          </div>
          <p className="bounty-page__status">Loading more…</p>
        </div>
      ) : null}

      {!initialLoading && !loadingMore && hasMore && bounties.length > 0 ? (
        <div className="bounty-page__more">
          <button
            type="button"
            className="btn btn--ghost"
            onClick={() => void fetchPage(offset, false)}
          >
            Load more
          </button>
        </div>
      ) : null}

      {!initialLoading && !hasMore && bounties.length > 0 ? (
        <p className="bounty-page__status bounty-page__end">You’ve reached the end.</p>
      ) : null}
    </main>
  )
}
