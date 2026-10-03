import { useEffect, useRef, useState } from 'react'
import type { FormEvent } from 'react'
import { getChatFileDownloadUrl, uploadChatAttachment } from '../../api/chat'
import { useBountyChat } from '../../hooks/useBountyChat'
import { shortWallet } from '../../auth/wallet'
import type { ChatAttachment } from '../../types/ChatMessage'
import './BountyChat.css'

type Props = {
  bountyId: string
  token: string | null
  wallet: string | null
  enabled: boolean
}

function formatBytes(size: number | null | undefined) {
  if (size == null) return ''
  if (size < 1024) return `${size} B`
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`
  return `${(size / (1024 * 1024)).toFixed(1)} MB`
}

export function BountyChat({ bountyId, token, wallet, enabled }: Props) {
  const { messages, status, error, sendMessage } = useBountyChat({
    bountyId,
    token,
    wallet,
    enabled,
  })
  const [draft, setDraft] = useState('')
  const [selectedFiles, setSelectedFiles] = useState<File[]>([])
  const [uploading, setUploading] = useState(false)
  const [sendError, setSendError] = useState<string | null>(null)
  const listRef = useRef<HTMLDivElement>(null)
  const fileInputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    const node = listRef.current
    if (!node) return
    node.scrollTop = node.scrollHeight
  }, [messages])

  async function downloadAttachment(file: ChatAttachment, preview = false) {
    if (!token) return
    try {
      const response = await getChatFileDownloadUrl(token, bountyId, file.file_id, preview)
      window.open(response.url, '_blank', 'noopener,noreferrer')
    } catch (err) {
      setSendError(err instanceof Error ? err.message : 'Could not open attachment.')
    }
  }

  async function onSubmit(e: FormEvent) {
    e.preventDefault()
    setSendError(null)

    const trimmed = draft.trim()
    if (!trimmed && selectedFiles.length === 0) return
    if (!token) {
      setSendError('Sign in to send attachments.')
      return
    }

    setUploading(true)
    try {
      const fileIds: string[] = []
      for (const file of selectedFiles) {
        const ready = await uploadChatAttachment(token, bountyId, file)
        fileIds.push(ready.file_id)
      }

      const ok = sendMessage(trimmed, fileIds)
      if (!ok) {
        setSendError('Could not send message. Check your connection.')
        return
      }

      setDraft('')
      setSelectedFiles([])
      if (fileInputRef.current) fileInputRef.current.value = ''
    } catch (err) {
      setSendError(err instanceof Error ? err.message : 'Attachment upload failed.')
    } finally {
      setUploading(false)
    }
  }

  if (!enabled) {
    return (
      <section className="bounty-chat">
        <h2>Chat</h2>
        <p className="bounty-chat__muted">
          Chat opens after the bounty is claimed, for the employer and assigned worker only.
        </p>
      </section>
    )
  }

  const canSend =
    status === 'open' && !uploading && (draft.trim().length > 0 || selectedFiles.length > 0)

  return (
    <section className="bounty-chat">
      <div className="bounty-chat__header">
        <h2>Chat</h2>
        <span className={`bounty-chat__status bounty-chat__status--${status}`}>{status}</span>
      </div>

      <div className="bounty-chat__messages" ref={listRef}>
        {messages.length === 0 ? (
          <p className="bounty-chat__muted">No messages yet. Say hello.</p>
        ) : (
          messages.map((message) => {
            const mine = message.sender_wallet === wallet
            return (
              <article
                key={message.id}
                className={`bounty-chat__message${mine ? ' bounty-chat__message--mine' : ''}`}
              >
                <header>
                  <span title={message.sender_wallet}>
                    {mine ? 'You' : shortWallet(message.sender_wallet)}
                  </span>
                  <time dateTime={message.created_at}>
                    {new Date(message.created_at).toLocaleString()}
                  </time>
                </header>
                {message.body ? <p>{message.body}</p> : null}
                {message.attachments.length > 0 ? (
                  <ul className="bounty-chat__attachments">
                    {message.attachments.map((file) => (
                      <li key={file.file_id}>
                        <button
                          type="button"
                          className="bounty-chat__attachment-link"
                          onClick={() => void downloadAttachment(file)}
                        >
                          {file.original_filename}
                          {file.size_bytes ? ` (${formatBytes(file.size_bytes)})` : ''}
                        </button>
                        {file.extracted_text_preview ? (
                          <p className="bounty-chat__attachment-preview">{file.extracted_text_preview}</p>
                        ) : null}
                        {file.has_preview ? (
                          <button
                            type="button"
                            className="bounty-chat__attachment-preview-btn"
                            onClick={() => void downloadAttachment(file, true)}
                          >
                            Open preview
                          </button>
                        ) : null}
                      </li>
                    ))}
                  </ul>
                ) : null}
              </article>
            )
          })
        )}
      </div>

      {error ? <p className="bounty-chat__error">{error}</p> : null}
      {sendError ? <p className="bounty-chat__error">{sendError}</p> : null}

      <form className="bounty-chat__composer" onSubmit={(e) => void onSubmit(e)}>
        {selectedFiles.length > 0 ? (
          <ul className="bounty-chat__pending-files">
            {selectedFiles.map((file) => (
              <li key={`${file.name}-${file.size}-${file.lastModified}`}>
                {file.name} ({formatBytes(file.size)})
              </li>
            ))}
          </ul>
        ) : null}

        <div className="bounty-chat__composer-row">
          <input
            ref={fileInputRef}
            type="file"
            className="bounty-chat__file-input"
            multiple
            disabled={status !== 'open' || uploading}
            onChange={(e) => setSelectedFiles(Array.from(e.target.files ?? []))}
          />
          <textarea
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            placeholder="Write a message…"
            rows={2}
            disabled={status !== 'open' || uploading}
          />
          <button type="submit" className="btn btn--primary" disabled={!canSend}>
            {uploading ? 'Uploading…' : 'Send'}
          </button>
        </div>
      </form>
    </section>
  )
}
