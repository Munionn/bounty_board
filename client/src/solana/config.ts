/** Solana cluster settings used by the client (Devnet). */
export const SOLANA_CLUSTER = import.meta.env.VITE_SOLANA_CLUSTER ?? 'devnet'
export const SOLANA_RPC_URL =
  import.meta.env.VITE_SOLANA_RPC_URL ?? 'https://api.devnet.solana.com'
export const BOUNTY_PROGRAM_ID =
  import.meta.env.VITE_BOUNTY_PROGRAM_ID ?? '4LSnV5QYxrXohJUHzQFUuj2YfQH3bWAscWUyQVuk4ELg'
