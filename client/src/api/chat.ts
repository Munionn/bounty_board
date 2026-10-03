import { apiJson } from './client'

export type RegisterChatFileInput = {
  original_filename: string
  content_type: string
  size_bytes: number
}

export type RegisterChatFileResponse = {
  file_id: string
  status: string
  upload_url: string
  upload_expires_at: string
  object_key: string
}

export type ChatFileStatus = {
  file_id: string
  status: 'pending' | 'processing' | 'ready' | 'failed' | 'deleted'
  original_filename: string
  content_type: string | null
  size_bytes: number | null
  extracted_text_preview?: string | null
  has_preview: boolean
  uploaded_at?: string | null
}

export type ChatFileDownload = {
  url: string
  expires_at: string
  file_id: string
  original_filename: string
  content_type: string | null
  size_bytes: number | null
  preview: boolean
}

export type PublishAttachmentInput = {
  body: string
  client_message_id?: string
}

/** Build a WebSocket URL for bounty chat (JWT passed as query param). */
export function chatWebSocketUrl(bountyId: string, token: string): string {
  const path = `/api/bounties/${bountyId}/chat/ws`
  const params = new URLSearchParams({ token })

  const apiBase = String(import.meta.env.VITE_API_BASE_URL ?? '')
    .trim()
    .replace(/\/$/, '')

  if (apiBase) {
    const wsBase = apiBase.replace(/^http/i, 'ws')
    return `${wsBase}${path}?${params.toString()}`
  }

  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
  return `${protocol}//${window.location.host}${path}?${params.toString()}`
}

export function registerChatFile(
  token: string,
  bountyId: string,
  body: RegisterChatFileInput,
) {
  return apiJson<RegisterChatFileResponse>(
    `/api/bounties/${bountyId}/chat/files`,
    { method: 'POST', body: JSON.stringify(body) },
    token,
  )
}

export async function uploadToPresignedUrl(
  uploadUrl: string,
  file: File,
  contentType: string,
) {
  const res = await fetch(uploadUrl, {
    method: 'PUT',
    headers: { 'Content-Type': contentType },
    body: file,
  })
  if (!res.ok) {
    throw new Error(`Direct upload failed (${res.status})`)
  }
}

export function completeChatFileUpload(
  token: string,
  bountyId: string,
  fileId: string,
  publish?: PublishAttachmentInput,
) {
  return apiJson<ChatFileStatus>(
    `/api/bounties/${bountyId}/chat/files/${fileId}/complete`,
    {
      method: 'POST',
      body: JSON.stringify({ publish: publish ?? null }),
    },
    token,
  )
}

export function getChatFileStatus(token: string, bountyId: string, fileId: string) {
  return apiJson<ChatFileStatus>(
    `/api/bounties/${bountyId}/chat/files/${fileId}/status`,
    {},
    token,
  )
}

export async function waitForChatFileReady(
  token: string,
  bountyId: string,
  fileId: string,
  options: { timeoutMs?: number; intervalMs?: number } = {},
) {
  const timeoutMs = options.timeoutMs ?? 60000
  const intervalMs = options.intervalMs ?? 750
  const deadline = Date.now() + timeoutMs

  while (Date.now() < deadline) {
    const status = await getChatFileStatus(token, bountyId, fileId)
    if (status.status === 'ready') return status
    if (status.status === 'failed') {
      throw new Error('Attachment processing failed.')
    }
    await new Promise((resolve) => setTimeout(resolve, intervalMs))
  }

  throw new Error('Attachment processing timed out.')
}

export function getChatFileDownloadUrl(
  token: string,
  bountyId: string,
  fileId: string,
  preview = false,
) {
  const q = preview ? '?preview=true' : ''
  return apiJson<ChatFileDownload>(
    `/api/bounties/${bountyId}/chat/files/${fileId}/download${q}`,
    {},
    token,
  )
}

export async function uploadChatAttachment(
  token: string,
  bountyId: string,
  file: File,
): Promise<ChatFileStatus> {
  const contentType = file.type || 'application/octet-stream'
  const registered = await registerChatFile(token, bountyId, {
    original_filename: file.name,
    content_type: contentType,
    size_bytes: file.size,
  })

  await uploadToPresignedUrl(registered.upload_url, file, contentType)
  await completeChatFileUpload(token, bountyId, registered.file_id)
  return waitForChatFileReady(token, bountyId, registered.file_id)
}
