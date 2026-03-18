import { tauriInvoke, createChannel, getAppConfig } from "@/services/api/core/context"
import type { AgentEntry, StoredMessage } from "@/types/chat"

export function fetchAgents() {
  return tauriInvoke<AgentEntry[]>("list_agents", { config: getAppConfig() })
}

export function fetchConversation(agentId: string) {
  return tauriInvoke<StoredMessage[]>("get_conversation", {
    agentId,
    config: getAppConfig(),
  })
}

export function syncMessages(agentId: string, messages: StoredMessage[]) {
  return tauriInvoke<{ ok: boolean }>("append_conversation", {
    agentId,
    request: { messages },
    config: getAppConfig(),
  })
}

export function clearConversation(agentId: string) {
  return tauriInvoke<{ ok: boolean }>("clear_conversation", {
    agentId,
    config: getAppConfig(),
  })
}

export async function sendChat(
  agentId: string,
  messages: { role: string; content: string }[],
  onChunk: (text: string) => void,
): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    let sseError: string | null = null

    const channel = createChannel<Record<string, unknown>>((data) => {
      if (typeof data.error === "string") {
        sseError = data.error
      } else if (typeof data.content === "string") {
        onChunk(data.content)
      }
      if (data.done) {
        if (sseError) {
          reject(new Error(sseError))
        } else {
          resolve()
        }
      }
    })

    tauriInvoke("message_chat", {
      agentId,
      request: { config: getAppConfig(), messages, operator_name: "Operator" },
      onEvent: channel,
    }).catch(reject)
  })
}
