import { create } from "zustand"
import { startInstall } from "@/services/api/install"
import type { WorkflowStepStatus } from "@/types/workflow"

type InstallStepKey = "detectEnv" | "installGit" | "installNode" | "installOpenClaw" | "configure" | "startGateway" | "verify"

export interface InstallStep {
  id: InstallStepKey
  status: WorkflowStepStatus
  details?: string[]
  error?: string
}

interface ErrorInfo {
  stepId: string
  message: string
}

const STEP_KEYS: InstallStepKey[] = [
  "detectEnv",
  "installGit",
  "installNode",
  "installOpenClaw",
  "configure",
  "startGateway",
  "verify",
]

function createInitialSteps(): InstallStep[] {
  return STEP_KEYS.map((id) => ({ id, status: "pending" }))
}

interface InstallWorkflowState {
  steps: InstallStep[]
  isRunning: boolean
  isComplete: boolean
  errorInfo: ErrorInfo | null
  progressValue: number
  start: () => Promise<void>
  reset: () => void
  clearError: () => void
}

export const useInstallStore = create<InstallWorkflowState>((set, get) => ({
  steps: createInitialSteps(),
  isRunning: false,
  isComplete: false,
  errorInfo: null,
  progressValue: 0,

  start: async () => {
    set({
      steps: createInitialSteps(),
      isRunning: true,
      isComplete: false,
      errorInfo: null,
      progressValue: 0,
    })

    try {
      await startInstall((event) => {
        if (event.type === "step") {
          const { step_id, status, details, error } = event.data
          const nextSteps = get().steps.map((step) =>
            step.id === step_id
              ? { ...step, status, details, error }
              : step,
          )
          const completedCount = nextSteps.filter(
            (step) => step.status === "complete",
          ).length

          set({
            steps: nextSteps,
            progressValue: Math.round((completedCount / nextSteps.length) * 100),
          })

          if (status === "error") {
            set({
              errorInfo: { stepId: step_id, message: error || "Unknown error" },
              isRunning: false,
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
      })
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
      steps: createInitialSteps(),
      isRunning: false,
      isComplete: false,
      errorInfo: null,
      progressValue: 0,
    }),

  clearError: () => set({ errorInfo: null }),
}))
