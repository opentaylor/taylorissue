import { create } from "zustand"
import {
  clearConversation,
  fetchAgents,
  fetchConversation,
  sendChat,
  syncMessages,
} from "@/services/api/message"
import type { AgentEntry, ChatMessage, StoredMessage } from "@/types/chat"

type ConversationMap = Record<string, ChatMessage[]>

function storedToChat(message: StoredMessage): ChatMessage {
  return {
    id: message.id,
    from: message.role,
    content: message.content,
    timestamp: message.timestamp,
  }
}

function chatToStored(message: ChatMessage): StoredMessage {
  return {
    id: message.id,
    role: message.from,
    content: message.content,
    timestamp: message.timestamp,
  }
}

interface MessageState {
  agents: AgentEntry[]
  agentsLoading: boolean
  selectedAgentId: string | null
  conversations: ConversationMap
  loadedAgentIds: Set<string>
  isTyping: boolean
  lastError: string | null
  initialize: () => Promise<void>
  selectAgent: (agentId: string) => void
  ensureConversation: (
    agentId: string,
    buildGreeting: (agent: AgentEntry) => string,
  ) => Promise<void>
  sendMessage: (agentId: string, text: string) => Promise<void>
  clearAgentConversation: (
    agentId: string,
    buildGreeting: (agent: AgentEntry) => string,
  ) => Promise<void>
  clearError: () => void
}

export const useMessageStore = create<MessageState>((set, get) => ({
  agents: [],
  agentsLoading: true,
  selectedAgentId: null,
  conversations: {},
  loadedAgentIds: new Set<string>(),
  isTyping: false,
  lastError: null,

  initialize: async () => {
    set({ agentsLoading: true })
    try {
      const agents = await fetchAgents()
      set({
        agents,
        agentsLoading: false,
        selectedAgentId: get().selectedAgentId ?? agents[0]?.id ?? null,
      })
    } catch {
      set({ agents: [], agentsLoading: false })
    }
  },

  selectAgent: (selectedAgentId) => set({ selectedAgentId }),

  ensureConversation: async (agentId, buildGreeting) => {
    if (get().loadedAgentIds.has(agentId)) {
      return
    }

    const agent = get().agents.find((item) => item.id === agentId)

    try {
      const storedMessages = await fetchConversation(agentId)
      const messages = storedMessages.map(storedToChat)
      const nextMessages =
        messages.length > 0 || !agent
          ? messages
          : [
              {
                id: `greeting-${agentId}`,
                from: "assistant" as const,
                content: buildGreeting(agent),
                timestamp: Date.now() - 1000,
              },
            ]

      set((state) => ({
        conversations: { ...state.conversations, [agentId]: nextMessages },
        loadedAgentIds: new Set(state.loadedAgentIds).add(agentId),
      }))
    } catch {
      if (!agent) {
        set((state) => ({
          loadedAgentIds: new Set(state.loadedAgentIds).add(agentId),
        }))
        return
      }

      set((state) => ({
        conversations: {
          ...state.conversations,
          [agentId]: [
            {
              id: `greeting-${agentId}`,
              from: "assistant",
              content: buildGreeting(agent),
              timestamp: Date.now() - 1000,
            },
          ],
        },
        loadedAgentIds: new Set(state.loadedAgentIds).add(agentId),
      }))
    }
  },

  sendMessage: async (agentId, text) => {
    const trimmed = text.trim()
    if (!trimmed || get().isTyping) {
      return
    }

    const userMessage: ChatMessage = {
      id: `user-${Date.now()}`,
      from: "user",
      content: trimmed,
      timestamp: Date.now(),
    }
    const assistantMessageId = `assistant-${Date.now()}`
    const assistantMessage: ChatMessage = {
      id: assistantMessageId,
      from: "assistant",
      content: "",
      timestamp: Date.now(),
    }

    const previousMessages = get().conversations[agentId] || []

    set((state) => ({
      conversations: {
        ...state.conversations,
        [agentId]: [...previousMessages, userMessage, assistantMessage],
      },
      isTyping: true,
    }))

    const apiMessages = [...previousMessages, userMessage].map((message) => ({
      role: message.from,
      content: message.content,
    }))

    let fullContent = ""

    try {
      await sendChat(agentId, apiMessages, (chunk) => {
        fullContent += chunk
        set((state) => ({
          conversations: {
            ...state.conversations,
            [agentId]: (state.conversations[agentId] || []).map((message) =>
              message.id === assistantMessageId
                ? { ...message, content: fullContent }
                : message,
            ),
          },
        }))
      })
    } catch (err) {
      const errMsg = err instanceof Error ? err.message : "Unknown error"
      set({ lastError: errMsg || "Error getting response. Check API connection." })
      set((state) => ({
        conversations: {
          ...state.conversations,
          [agentId]: (state.conversations[agentId] || []).filter(
            (message) => message.id !== assistantMessageId,
          ),
        },
      }))
    } finally {
      set({ isTyping: false })
    }

    if (fullContent) {
      const finalAssistantMessage: ChatMessage = {
        ...assistantMessage,
        content: fullContent,
        timestamp: Date.now(),
      }

      void syncMessages(
        agentId,
        [userMessage, finalAssistantMessage].map(chatToStored),
      ).catch(() => {})
    } else {
      void syncMessages(agentId, [chatToStored(userMessage)]).catch(() => {})
    }
  },

  clearError: () => set({ lastError: null }),

  clearAgentConversation: async (agentId, buildGreeting) => {
    const agent = get().agents.find((item) => item.id === agentId)
    void clearConversation(agentId).catch(() => {})

    set((state) => ({
      isTyping: false,
      conversations: {
        ...state.conversations,
        [agentId]:
          agent == null
            ? []
            : [
                {
                  id: `greeting-${Date.now()}`,
                  from: "assistant",
                  content: buildGreeting(agent),
                  timestamp: Date.now(),
                },
              ],
      },
    }))
  },
}))
