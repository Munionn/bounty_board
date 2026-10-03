export type BountyStatus = 'open' | 'claimed' | 'closed'

/** Matches `BountyResponse` from GET /api/bounties */
export type Bounty = {
  id: string
  on_chain_id: number
  poster_wallet: string
  claimer_wallet: string | null
  pda_address: string
  title: string
  description: string
  amount_lamports: number
  status: BountyStatus
  submission_uri: string | null
  tx_signature: string | null
  created_at: string
  updated_at: string
}
