export type LlmProvider = "openai" | "anthropic"

export interface ModelConfig {
  provider: LlmProvider
  baseUrl: string
  apiKey: string
  model: string
}

export interface AppConfig {
  provider: LlmProvider
  baseUrl: string
  apiKey: string
  model: string
  openclawDir: string
}
