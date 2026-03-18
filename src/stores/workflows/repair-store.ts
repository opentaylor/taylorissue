import { create } from "zustand"
import {
  fixStep,
  startCustomFix,
  startRepair,
} from "@/services/api/repair"
import type { WorkflowStepStatus } from "@/types/workflow"

export const REPAIR_STEP_KEYS = [
  "checkGateway",
  "checkConfig",
  "checkModelRequest",
  "runDoctor",
] as const

export type RepairStepKey = (typeof REPAIR_STEP_KEYS)[number]

export const CUSTOM_FIX_STEP_KEYS = [
  "analyze",
  "diagnose",
  "fix",
  "verify",
] as const

export type CustomFixStepKey = (typeof CUSTOM_FIX_STEP_KEYS)[number]

interface RepairStep {
  id: string
  status: WorkflowStepStatus
  details?: string[]
  error?: string
  hasIssue?: boolean
  fixing?: boolean
  fixed?: boolean
  fixError?: string
  fixDetails?: string[]
}

interface CustomFixStep {
  id: string
  status: WorkflowStepStatus
  details?: string[]
  error?: string
}

interface RepairWorkflowState {
  steps: RepairStep[]
  isRunning: boolean
  isComplete: boolean
  sessionId: string | null
  progressValue: number
  issueCount: number
  customText: string
  customFixSteps: CustomFixStep[]
  isCustomFixing: boolean
  customFixDone: boolean
  customFixError: boolean
  setCustomText: (text: string) => void
  startScan: () => Promise<void>
  rescan: () => Promise<void>
  fixSingleStep: (stepId: string) => Promise<void>
  resetScan: () => void
  startCustom: () => Promise<void>
  resetCustom: () => void
}

export const useRepairStore = create<RepairWorkflowState>((set, get) => ({
  steps: [],
  isRunning: false,
  isComplete: false,
  sessionId: null,
  progressValue: 0,
  issueCount: 0,
  customText: "",
  customFixSteps: [],
  isCustomFixing: false,
  customFixDone: false,
  customFixError: false,

  setCustomText: (customText) => set({ customText }),

  startScan: async () => {
    set({
      steps: [],
      isRunning: true,
      isComplete: false,
      sessionId: null,
      progressValue: 0,
      issueCount: 0,
    })

    try {
      await startRepair((event) => {
        if (event.type === "step") {
          const { step_id, status, details, error, has_issue } = event.data

          const nextSteps = (() => {
            const existing = get().steps.some((step) => step.id === step_id)
            if (!existing && status === "active") {
              return [
                ...get().steps,
                { id: step_id, status, details, error, hasIssue: has_issue },
              ]
            }

            return get().steps.map((step) =>
              step.id === step_id
                ? {
                    ...step,
                    status,
                    details,
                    error,
                    hasIssue: has_issue ?? step.hasIssue,
                  }
                : step,
            )
          })()

          const completedCount = nextSteps.filter(
            (step) => step.status === "complete",
          ).length

          set({
            steps: nextSteps,
            progressValue:
              nextSteps.length === 0
                ? 0
                : Math.round((completedCount / nextSteps.length) * 100),
            issueCount: nextSteps.filter(
              (step) => step.hasIssue || step.status === "error",
            ).length,
          })
          return
        }

        const hasFailedStep = get().steps.some((step) => step.status === "error")
        set({
          isRunning: false,
          isComplete: !hasFailedStep,
          sessionId: event.data.session_id ?? get().sessionId,
          progressValue: hasFailedStep ? get().progressValue : 100,
        })
      })
    } catch {
      set({ isRunning: false })
    }
  },

  rescan: async () => {
    get().resetScan()
    await get().startScan()
  },

  fixSingleStep: async (stepId) => {
    const sessionId = get().sessionId
    const step = get().steps.find((item) => item.id === stepId)
    if (!sessionId || !step) {
      return
    }

    const issueDescription = [step.error, ...(step.details || [])]
      .filter(Boolean)
      .join("\n")

    set((state) => ({
      steps: state.steps.map((item) =>
        item.id === stepId
          ? {
              ...item,
              fixing: true,
              fixError: undefined,
              fixDetails: undefined,
            }
          : item,
      ),
    }))

    try {
      await fixStep(
        sessionId,
        stepId,
        issueDescription,
        (event) => {
          if (event.type !== "step") {
            return
          }

          const { status, details, error } = event.data
          set((state) => ({
            steps: state.steps.map((item) => {
              if (item.id !== stepId) {
                return item
              }
              if (status === "complete") {
                return {
                  ...item,
                  fixing: false,
                  fixed: true,
                  fixDetails: details,
                  hasIssue: false,
                  status: "complete",
                  error: undefined,
                }
              }
              if (status === "error") {
                return {
                  ...item,
                  fixing: false,
                  fixError: error || "Fix failed",
                }
              }
              return item
            }),
          }))
        },
      )
    } catch (error) {
      set((state) => ({
        steps: state.steps.map((item) =>
          item.id === stepId
            ? {
                ...item,
                fixing: false,
                fixError: error instanceof Error ? error.message : "Fix failed",
              }
            : item,
        ),
      }))
    }
  },

  resetScan: () =>
    set({
      steps: [],
      isRunning: false,
      isComplete: false,
      sessionId: null,
      progressValue: 0,
      issueCount: 0,
    }),

  startCustom: async () => {
    const problem = get().customText.trim()
    if (!problem) {
      return
    }

    set({
      isCustomFixing: true,
      customFixDone: false,
      customFixError: false,
      customFixSteps: [],
    })

    try {
      await startCustomFix(problem, (event) => {
        if (event.type === "step") {
          const { step_id, status, details, error } = event.data
          const nextSteps = (() => {
            const existing = get().customFixSteps.some(
              (step) => step.id === step_id,
            )
            if (!existing && status === "active") {
              return [...get().customFixSteps, { id: step_id, status, details, error }]
            }

            return get().customFixSteps.map((step) =>
              step.id === step_id
                ? { ...step, status, details, error }
                : step,
            )
          })()

          set({ customFixSteps: nextSteps })

          if (status === "error") {
            set({ isCustomFixing: false, customFixError: true })
          }
          return
        }

        const hasFailedStep = get().customFixSteps.some(
          (step) => step.status === "error",
        )
        set({
          isCustomFixing: false,
          customFixDone: !hasFailedStep,
          customFixError: hasFailedStep,
        })
      })
    } catch {
      set({ isCustomFixing: false, customFixError: true })
    }
  },

  resetCustom: () =>
    set({
      customText: "",
      customFixSteps: [],
      isCustomFixing: false,
      customFixDone: false,
      customFixError: false,
    }),
}))
