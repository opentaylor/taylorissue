import { invoke, Channel } from "@tauri-apps/api/core"
import { useConfigStore } from "@/stores/config-store"

export async function tauriInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(command, args)
}

export function createChannel<T>(onMessage: (data: T) => void): Channel<T> {
  const channel = new Channel<T>()
  channel.onmessage = onMessage
  return channel
}

export function getAppConfig(): Record<string, unknown> {
  const cfg = useConfigStore.getState().getAppConfig()
  return {
    provider: cfg.provider,
    base_url: cfg.baseUrl,
    api_key: cfg.apiKey,
    model: cfg.model,
    workspace_path: cfg.openclawDir,
  }
}
