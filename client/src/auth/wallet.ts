import bs58 from 'bs58'

export type PhantomProvider = {
  isPhantom?: boolean
  publicKey?: { toBase58(): string; toString(): string }
  connect: (opts?: { onlyIfTrusted?: boolean }) => Promise<{ publicKey: { toBase58(): string } }>
  signMessage: (
    message: Uint8Array,
    display?: 'utf8' | 'hex',
  ) => Promise<{ signature: Uint8Array }>
  on?: (event: string, handler: () => void) => void
  off?: (event: string, handler: () => void) => void
}

declare global {
  interface Window {
    solana?: PhantomProvider
    phantom?: { solana?: PhantomProvider }
  }
}

export function getPhantom(): PhantomProvider | null {
  const provider = window.phantom?.solana ?? window.solana
  if (provider?.isPhantom) return provider
  return provider ?? null
}

export function shortWallet(address: string) {
  return `${address.slice(0, 4)}…${address.slice(-4)}`
}

type NonceResponse = {
  wallet: string
  message: string
  nonce: string
}

type VerifyResponse = {
  token: string
  wallet: string
}

export async function signInWithPhantom(): Promise<VerifyResponse> {
  const phantom = getPhantom()
  if (!phantom) {
    throw new Error('Phantom wallet not found. Install Phantom and refresh.')
  }

  const connected = await phantom.connect()
  const wallet = connected.publicKey.toBase58()

  const nonceRes = await fetch(`/api/auth/nonce?wallet=${encodeURIComponent(wallet)}`)
  if (!nonceRes.ok) throw new Error('Failed to fetch sign-in challenge')
  const { message } = (await nonceRes.json()) as NonceResponse

  const encoded = new TextEncoder().encode(message)
  const signed = await phantom.signMessage(encoded, 'utf8')
  const signature = bs58.encode(signed.signature)

  const verifyRes = await fetch('/api/auth/verify', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ wallet, message, signature }),
  })
  if (!verifyRes.ok) throw new Error('Signature verification failed')

  return (await verifyRes.json()) as VerifyResponse
}
