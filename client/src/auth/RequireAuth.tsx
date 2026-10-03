import type { ReactNode } from 'react'
import { Link, Navigate, useLocation, useNavigate } from 'react-router-dom'
import { useAuth } from '../auth/AuthContext'

export type AuthNext = '/bountypage' | '/postbounty' | '/'

type LocationAuthState = {
  next?: string
}

export function getAuthNext(state: unknown): string {
  const next = (state as LocationAuthState | null)?.next
  if (!next) return '/'
  if (next === '/bountypage' || next === '/postbounty') return next
  if (next.startsWith('/bounty/')) return next
  return '/'
}

/** Redirects to /login (keeping intended destination) when logged out. */
export function RequireAuth({ children }: { children: ReactNode }) {
  const { isAuthenticated } = useAuth()
  const location = useLocation()

  if (!isAuthenticated) {
    return (
      <Navigate
        to="/login"
        replace
        state={{ next: location.pathname } satisfies LocationAuthState}
      />
    )
  }

  return children
}

export function AuthGateLink({
  to,
  className,
  children,
}: {
  to: AuthNext
  className?: string
  children: ReactNode
}) {
  const { isAuthenticated } = useAuth()
  const navigate = useNavigate()

  if (isAuthenticated) {
    return (
      <Link to={to} className={className}>
        {children}
      </Link>
    )
  }

  return (
    <button
      type="button"
      className={className}
      onClick={() => navigate('/login', { state: { next: to } satisfies LocationAuthState })}
    >
      {children}
    </button>
  )
}
