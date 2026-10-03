export type ChatAttachment = {
  file_id: string
  original_filename: string
  content_type: string | null
  size_bytes: number | null
  position: number
  caption: string | null
  status: string
  extracted_text_preview?: string | null
  has_preview: boolean
}

export type ChatMessage = {
  id: string
  bounty_id: string
  chat_id: string
  sender_wallet: string
  body: string
  attachments: ChatAttachment[]
  created_at: string
}

export type ClientWsEvent =
  | {
      type: 'send_message'
      body?: string
      file_ids?: string[]
      client_message_id?: string
    }
  | { type: 'ping' }

export type ServerWsEvent =
  | { type: 'connected'; chat_id: string; bounty_id: string }
  | { type: 'history'; messages: ChatMessage[] }
  | { type: 'message_created'; message: ChatMessage }
  | {
      type: 'message_ack'
      client_message_id?: string | null
      message: ChatMessage
    }
  | { type: 'pong' }
  | {
      type: 'error'
      code: string
      message: string
      client_message_id?: string | null
    }
