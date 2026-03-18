export interface AgentEntry {
  id: string
  name: string
  title: string
  reports_to: string | null
  direct_reports: string[]
  soul_path: string | null
  color: string
  emoji: string
  model: string | null
  description: string
}

export interface StoredMessage {
  id: string
  role: "user" | "assistant"
  content: string
  timestamp: number
}

export interface ChatMessage {
  id: string
  from: "user" | "assistant"
  content: string
  timestamp: number
}

