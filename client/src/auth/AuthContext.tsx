import { createContext, useContext, useEffect, useState } from 'react'
import type { ReactNode } from 'react'
import { apiFetch, ApiError } from '../api/client'

const TOKEN_KEY = 'bb_auth_token'
const WALLET_KEY = 'bb_auth_wallet'

type AuthContextValue = {
  token: string | null
  wallet: string | null
  isAuthenticated: boolean
  ready: boolean
  setSession: (token: string, wallet: string) => void
  logout: () => void
}

const AuthContext = createContext<AuthContextValue | null>(null)

export function AuthProvider({ children }: { children: ReactNode }) {
  const [token, setToken] = useState<string | null>(() => localStorage.getItem(TOKEN_KEY))
  const [wallet, setWallet] = useState<string | null>(() => localStorage.getItem(WALLET_KEY))
  const [ready, setReady] = useState(false)

  useEffect(() => {
    if (token) localStorage.setItem(TOKEN_KEY, token)
    else localStorage.removeItem(TOKEN_KEY)
  }, [token])

  useEffect(() => {
    if (wallet) localStorage.setItem(WALLET_KEY, wallet)
    else localStorage.removeItem(WALLET_KEY)
  }, [wallet])

  useEffect(() => {
    let cancelled = false

    async function validate() {
      if (!token) {
        if (!cancelled) setReady(true)
        return
      }

      try {
        const res = await apiFetch('/api/auth/me', {}, token)
        const me = (await res.json()) as { wallet_address: string }
        if (!cancelled) {
          setWallet(me.wallet_address)
          setReady(true)
        }
      } catch (err) {
        if (!cancelled) {
          if (err instanceof ApiError && (err.status === 401 || err.status === 404)) {
            setToken(null)
            setWallet(null)
          }
          setReady(true)
        }
      }
    }

    void validate()
    return () => {
      cancelled = true
    }
  }, [token])

  function setSession(nextToken: string, nextWallet: string) {
    setToken(nextToken)
    setWallet(nextWallet)
  }

  function logout() {
    setToken(null)
    setWallet(null)
  }

  return (
    <AuthContext.Provider
      value={{
        token,
        wallet,
        isAuthenticated: Boolean(token && wallet),
        ready,
        setSession,
        logout,
      }}
    >
      {children}
    </AuthContext.Provider>
  )
}

export function useAuth() {
  const ctx = useContext(AuthContext)
  if (!ctx) throw new Error('useAuth must be used within AuthProvider')
  return ctx
}
