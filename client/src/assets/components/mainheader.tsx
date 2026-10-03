import { Link } from 'react-router-dom'
import { useAuth } from '../../auth/AuthContext'
import { AuthGateLink } from '../../auth/RequireAuth'
import { shortWallet } from '../../auth/wallet'
import './mainheader.css'

export function MainHeader() {
  const { isAuthenticated, wallet, logout } = useAuth()

  return (
    <header className="site-header">
      <Link className="site-header__mark" to="/">
        <span className="site-header__dot" aria-hidden="true" />
        <span>Bounty Board</span>
      </Link>
      <div className="site-header__actions">
        {isAuthenticated && wallet ? (
          <>
            <AuthGateLink to="/bountypage" className="btn btn--text">
              Browse
            </AuthGateLink>
            <AuthGateLink to="/postbounty" className="btn btn--ghost">
              Post
            </AuthGateLink>
            <span className="site-header__wallet" title={wallet}>
              {shortWallet(wallet)}
            </span>
            <button type="button" className="btn btn--text" onClick={logout}>
              Log out
            </button>
          </>
        ) : (
          <>
            <Link to="/login" className="btn btn--text">
              Log in
            </Link>
            <Link to="/login" className="btn btn--primary">
              Sign up
            </Link>
          </>
        )}
      </div>
    </header>
  )
}
