import { useCallback, useEffect, useRef, useState } from 'react'
import { chatWebSocketUrl } from '../api/chat'
import type { ChatMessage, ServerWsEvent } from '../types/ChatMessage'

type ChatStatus = 'idle' | 'connecting' | 'open' | 'closed' | 'error'

type Options = {
  bountyId: string | undefined
  token: string | null
  wallet: string | null
  enabled: boolean
}

function parseServerEvent(raw: string): ServerWsEvent | null {
  try {
    return JSON.parse(raw) as ServerWsEvent
  } catch {
    return null
  }
}

function dedupeMessages(list: ChatMessage[]): ChatMessage[] {
  const seen = new Map<string, ChatMessage>()
  for (const message of list) {
    seen.set(message.id, message)
  }
  return Array.from(seen.values())
}

/** Merge a server message, drop matching optimistic row, dedupe by id. */
function mergeMessage(
  list: ChatMessage[],
  message: ChatMessage,
  clientMessageId?: string | null,
): ChatMessage[] {
  const withoutOptimistic = clientMessageId
    ? list.filter((item) => item.id !== clientMessageId)
    : list

  const index = withoutOptimistic.findIndex((item) => item.id === message.id)
  if (index === -1) return [...withoutOptimistic, message]

  const next = [...withoutOptimistic]
  next[index] = message
  return next
}

export function useBountyChat({ bountyId, token, wallet, enabled }: Options) {
  const [messages, setMessages] = useState<ChatMessage[]>([])
  const [status, setStatus] = useState<ChatStatus>('idle')
  const [error, setError] = useState<string | null>(null)
  const [chatId, setChatId] = useState<string | null>(null)
  const wsRef = useRef<WebSocket | null>(null)

  useEffect(() => {
    if (!enabled || !bountyId || !token || !wallet) {
      setStatus('idle')
      setMessages([])
      setChatId(null)
      setError(null)
      return
    }

    let cancelled = false
    setStatus('connecting')
    setError(null)

    const ws = new WebSocket(chatWebSocketUrl(bountyId, token))
    wsRef.current = ws

    ws.onopen = () => {
      if (cancelled) {
        ws.close()
        return
      }
      setStatus('open')
    }

    ws.onmessage = (event) => {
      if (cancelled) return

      const payload = parseServerEvent(String(event.data))
      if (!payload) {
        setError('Received invalid chat payload from server.')
        return
      }

      switch (payload.type) {
        case 'connected':
          setChatId(payload.chat_id)
          break
        case 'history':
          setMessages(dedupeMessages(payload.messages))
          break
        case 'message_ack':
          setMessages((prev) =>
            mergeMessage(prev, payload.message, payload.client_message_id),
          )
          break
        case 'message_created':
          // Sender already receives message_ack; skip duplicate broadcast for own messages.
          setMessages((prev) => {
            if (payload.message.sender_wallet === wallet) {
              if (prev.some((item) => item.id === payload.message.id)) return prev
            }
            return mergeMessage(prev, payload.message)
          })
          break
        case 'error':
          setError(payload.message)
          break
        case 'pong':
          break
        default:
          break
      }
    }

    ws.onerror = () => {
      if (cancelled) return
      setStatus('error')
      setError('Chat connection failed.')
    }

    ws.onclose = () => {
      if (cancelled) return
      setStatus('closed')
      if (wsRef.current === ws) wsRef.current = null
    }

    return () => {
      cancelled = true
      ws.close()
      if (wsRef.current === ws) wsRef.current = null
    }
  }, [bountyId, enabled, token, wallet])

  const sendMessage = useCallback(
    (body: string, fileIds: string[] = []) => {
      const trimmed = body.trim()
      if (!trimmed && fileIds.length === 0) return false
      if (!wallet || !bountyId) return false

      const clientMessageId = crypto.randomUUID()
      const optimistic: ChatMessage = {
        id: clientMessageId,
        bounty_id: bountyId,
        chat_id: chatId ?? '',
        sender_wallet: wallet,
        body: trimmed,
        attachments: [],
        created_at: new Date().toISOString(),
      }

      setMessages((prev) => [...prev, optimistic])

      const event = {
        type: 'send_message' as const,
        body: trimmed,
        file_ids: fileIds,
        client_message_id: clientMessageId,
      }

      const socket = wsRef.current
      if (!socket || socket.readyState !== WebSocket.OPEN) {
        setError('Chat is not connected.')
        return false
      }

      socket.send(JSON.stringify(event))
      return true
    },
    [bountyId, chatId, wallet],
  )

  return {
    messages,
    status,
    error,
    chatId,
    sendMessage,
  }
}
