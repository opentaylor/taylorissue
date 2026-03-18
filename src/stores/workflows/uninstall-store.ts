import { create } from "zustand"
import { startUninstall } from "@/services/api/uninstall"
import type { WorkflowStepStatus } from "@/types/workflow"

export const OPTION_KEYS = [
  "stopServices",
  "removePackage",
  "deleteWorkspace",
  "deleteConfig",
  "deleteData",
] as const

export type OptionKey = (typeof OPTION_KEYS)[number]

export const REQUIRED_OPTIONS: OptionKey[] = ["stopServices", "removePackage"]
export const DEFAULT_CHECKED: OptionKey[] = [
  "stopServices",
  "removePackage",
  "deleteConfig",
  "deleteData",
]
export const DANGEROUS_OPTIONS: OptionKey[] = ["deleteWorkspace"]

interface UninstallStep {
  id: string
  status: WorkflowStepStatus
  details?: string[]
  error?: string
}

interface ErrorInfo {
  stepId: string
  message: string
}

interface UninstallWorkflowState {
  selectedOptions: Set<OptionKey>
  steps: UninstallStep[]
  isRunning: boolean
  isComplete: boolean
  errorInfo: ErrorInfo | null
  progressValue: number
  toggleOption: (key: OptionKey) => void
  start: () => Promise<void>
  reset: () => void
  clearError: () => void
}

export const useUninstallStore = create<UninstallWorkflowState>((set, get) => ({
  selectedOptions: new Set(DEFAULT_CHECKED),
  steps: [],
  isRunning: false,
  isComplete: false,
  errorInfo: null,
  progressValue: 0,

  toggleOption: (key) => {
    if (REQUIRED_OPTIONS.includes(key)) {
      return
    }

    set((state) => {
      const next = new Set(state.selectedOptions)
      if (next.has(key)) {
        next.delete(key)
      } else {
        next.add(key)
      }
      return { selectedOptions: next }
    })
  },

  start: async () => {
    set({
      steps: [],
      isRunning: true,
      isComplete: false,
      errorInfo: null,
      progressValue: 0,
    })

    try {
      await startUninstall(
        Array.from(get().selectedOptions),
        (event) => {
          if (event.type === "step") {
            const { step_id, status, details, error } = event.data

            const nextSteps = (() => {
              const existing = get().steps.some((step) => step.id === step_id)
              if (!existing && status === "active") {
                return [...get().steps, { id: step_id, status, details, error }]
              }

              return get().steps.map((step) =>
                step.id === step_id
                  ? { ...step, status, details, error }
                  : step,
              )
            })()

            const completedCount = nextSteps.filter(
              (step) => step.status === "complete",
            ).length
            const progressValue =
              nextSteps.length === 0
                ? 0
                : Math.round((completedCount / nextSteps.length) * 100)

            set({ steps: nextSteps, progressValue })

            if (status === "error") {
              set({
                isRunning: false,
                errorInfo: { stepId: step_id, message: error || "Unknown error" },
              })
            }
            return
          }

          const hasFailedStep = get().steps.some((step) => step.status === "error")
          set({
            isRunning: false,
            isComplete: !hasFailedStep,
            progressValue: hasFailedStep ? get().progressValue : 100,
          })
        },
      )
    } catch (error) {
      set({
        isRunning: false,
        errorInfo: {
          stepId: "",
          message: error instanceof Error ? error.message : "Unknown error",
        },
      })
    }
  },

  reset: () =>
    set({
      selectedOptions: new Set(DEFAULT_CHECKED),
      steps: [],
      isRunning: false,
      isComplete: false,
      errorInfo: null,
      progressValue: 0,
    }),

  clearError: () => set({ errorInfo: null }),
}))
