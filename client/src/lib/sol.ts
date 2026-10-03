export const LAMPORTS_PER_SOL = 1_000_000_000

export function solToLamports(sol: number): number {
  return Math.round(sol * LAMPORTS_PER_SOL)
}

export function lamportsToSol(lamports: number): number {
  return lamports / LAMPORTS_PER_SOL
}

export function formatSol(lamports: number): string {
  return lamportsToSol(lamports).toLocaleString(undefined, { maximumFractionDigits: 4 })
}

/** Temporary off-chain placeholders until post_bounty tx is wired. */
export function makePendingPda(wallet: string): string {
  return `pending-${wallet.slice(0, 8)}-${Date.now()}`
}

export function makeOnChainId(): number {
  return Date.now()
}
