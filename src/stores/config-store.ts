import { create } from "zustand"
import { persist } from "zustand/middleware"
import type { ModelConfig, AppConfig } from "@/types/config"

interface ConfigState {
  language: "zh-CN" | "en-US"
  modelConfig: ModelConfig
  openclawDir: string
  setLanguage: (language: "zh-CN" | "en-US") => void
  setModelConfig: (config: Partial<ModelConfig>) => void
  setOpenclawDir: (dir: string) => void
  getAppConfig: () => AppConfig
}

export const useConfigStore = create<ConfigState>()(
  persist(
    (set, get) => ({
      language: "zh-CN",
      modelConfig: {
        provider: "openai",
        baseUrl: "https://aihubmix.com/v1",
        apiKey: "",
        model: "gpt-4.1",
      },
      openclawDir: "~/.openclaw",

      setLanguage: (language) => set({ language }),
      setModelConfig: (config) =>
        set((state) => ({
          modelConfig: { ...state.modelConfig, ...config },
        })),
      setOpenclawDir: (openclawDir) => set({ openclawDir }),

      getAppConfig: () => {
        const state = get()
        return {
          provider: state.modelConfig.provider,
          baseUrl: state.modelConfig.baseUrl,
          apiKey: state.modelConfig.apiKey,
          model: state.modelConfig.model,
          openclawDir: state.openclawDir,
        }
      },
    }),
    {
      name: "taylor-config",

      version: 1,
      migrate: (persistedState) => persistedState as any,
      partialize: (state) => ({
        language: state.language,
        modelConfig: state.modelConfig,
        openclawDir: state.openclawDir,
      }),
    },
  ),
)
