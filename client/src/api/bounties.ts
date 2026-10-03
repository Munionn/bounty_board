import { apiJson } from './client'
import type { Bounty, BountyStatus } from '../types/BountyType'
import type { Submission } from '../types/Submission'

export type ListBountiesParams = {
  status?: BountyStatus | 'all'
  poster?: string
  claimer?: string
  limit?: number
  offset?: number
}

export type CreateBountyInput = {
  on_chain_id: number
  poster_wallet: string
  pda_address: string
  title: string
  description?: string
  amount_lamports: number
  tx_signature?: string | null
}

export type UpdateBountyInput = {
  title?: string
  description?: string
  claimer_wallet?: string
  amount_lamports?: number
  status?: BountyStatus
  submission_uri?: string
  tx_signature?: string
}

function toQuery(params: ListBountiesParams): string {
  const q = new URLSearchParams()
  if (params.status && params.status !== 'all') q.set('status', params.status)
  if (params.poster) q.set('poster', params.poster)
  if (params.claimer) q.set('claimer', params.claimer)
  if (params.limit != null) q.set('limit', String(params.limit))
  if (params.offset != null) q.set('offset', String(params.offset))
  const s = q.toString()
  return s ? `?${s}` : ''
}

export function listBounties(token: string | null, params: ListBountiesParams = {}) {
  return apiJson<Bounty[]>(`/api/bounties${toQuery(params)}`, {}, token)
}

export function getBounty(token: string | null, id: string) {
  return apiJson<Bounty>(`/api/bounties/${id}`, {}, token)
}

export function createBounty(token: string | null, body: CreateBountyInput) {
  return apiJson<Bounty>(
    '/api/bounties',
    { method: 'POST', body: JSON.stringify(body) },
    token,
  )
}

export function updateBounty(token: string | null, id: string, body: UpdateBountyInput) {
  return apiJson<Bounty>(
    `/api/bounties/${id}`,
    { method: 'PUT', body: JSON.stringify(body) },
    token,
  )
}

export function updateBountyStatus(token: string | null, id: string, status: BountyStatus) {
  return apiJson<Bounty>(
    `/api/bounties/${id}/status`,
    { method: 'PATCH', body: JSON.stringify({ status }) },
    token,
  )
}

export function deleteBounty(token: string | null, id: string) {
  return apiJson<void>(`/api/bounties/${id}`, { method: 'DELETE' }, token)
}

export function createSubmission(
  token: string | null,
  bountyId: string,
  body: { submitter_wallet: string; submission_uri?: string; note?: string },
) {
  return apiJson<Submission>(
    `/api/bounties/${bountyId}/submissions`,
    { method: 'POST', body: JSON.stringify(body) },
    token,
  )
}

export function listSubmissions(token: string | null, bountyId: string) {
  return apiJson<Submission[]>(`/api/bounties/${bountyId}/submissions`, {}, token)
}
