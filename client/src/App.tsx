import { BrowserRouter, Route, Routes, useLocation } from 'react-router-dom'
import './App.css'
import { AuthProvider, useAuth } from './auth/AuthContext'
import { AuthGateLink, RequireAuth } from './auth/RequireAuth'
import { AuthDialog } from './assets/components/AuthDialog'
import { BountyDetailPage } from './assets/components/BountyDetailPage'
import { BountyPage } from './assets/components/BountyPage'
import { MainHeader } from './assets/components/mainheader'
import { PostBountyForm } from './assets/components/PostBountyForm'

function HomePage() {
  return (
    <main className="page">
      <section className="hero" aria-label="Bounty Board">
        <div className="hero__visual" aria-hidden="true">
          <span className="hero__orb hero__orb--a" />
          <span className="hero__orb hero__orb--b" />
        </div>
        <div className="hero__content">
          <h1 className="hero__brand">
            <AuthGateLink to="/bountypage">Bounty Board</AuthGateLink>
          </h1>
          <p className="hero__headline">Post work. Claim rewards. Settle on-chain.</p>
          <p className="hero__lede">
            A clean workspace for Solana bounties — publish tasks, track submissions, and pay out
            when the work lands.
          </p>
          <div className="hero__actions">
            <AuthGateLink to="/bountypage" className="btn btn--primary">
              Browse bounties
            </AuthGateLink>
            <AuthGateLink to="/postbounty" className="btn btn--ghost">
              Post a bounty
            </AuthGateLink>
          </div>
        </div>
      </section>
    </main>
  )
}

function AppRoutes() {
  const { pathname } = useLocation()
  const { isAuthenticated } = useAuth()
  const showPostDialog = pathname === '/postbounty' && isAuthenticated
  const showAuthDialog = pathname === '/login'

  return (
    <>
      <Routes>
        <Route path="/" element={<HomePage />} />
        <Route
          path="/bountypage"
          element={
            <RequireAuth>
              <BountyPage />
            </RequireAuth>
          }
        />
        <Route
          path="/bounty/:id"
          element={
            <RequireAuth>
              <BountyDetailPage />
            </RequireAuth>
          }
        />
        <Route
          path="/postbounty"
          element={
            <RequireAuth>
              <BountyPage />
            </RequireAuth>
          }
        />
        <Route path="/login" element={<HomePage />} />
      </Routes>
      {showPostDialog ? <PostBountyForm /> : null}
      {showAuthDialog ? <AuthDialog /> : null}
    </>
  )
}

function App() {
  return (
    <BrowserRouter>
      <AuthProvider>
        <MainHeader />
        <AppRoutes />
      </AuthProvider>
    </BrowserRouter>
  )
}

export default App
