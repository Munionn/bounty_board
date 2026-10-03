export function apiUrl(path: string): string {
  const base = String(import.meta.env.VITE_API_BASE_URL ?? '')
    .trim()
    .replace(/\/$/, '')
  const normalized = path.startsWith('/') ? path : `/${path}`
  return `${base}${normalized}`
}

export class ApiError extends Error {
  status: number

  constructor(status: number, message: string) {
    super(message)
    this.name = 'ApiError'
    this.status = status
  }
}

export async function apiFetch(path: string, init: RequestInit = {}, token?: string | null) {
  const headers = new Headers(init.headers)
  if (!headers.has('Content-Type') && init.body) {
    headers.set('Content-Type', 'application/json')
  }
  if (token) {
    headers.set('Authorization', `Bearer ${token}`)
  }

  const res = await fetch(apiUrl(path), { ...init, headers })
  if (!res.ok) {
    let detail = `${res.status} ${res.statusText}`
    try {
      const text = await res.text()
      if (text) detail = text
    } catch {
      /* ignore */
    }
    throw new ApiError(res.status, detail)
  }
  return res
}

export async function apiJson<T>(path: string, init: RequestInit = {}, token?: string | null): Promise<T> {
  const res = await apiFetch(path, init, token)
  if (res.status === 204) return undefined as T
  return (await res.json()) as T
}
