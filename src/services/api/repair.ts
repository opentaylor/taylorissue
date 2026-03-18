import { tauriInvoke, createChannel, getAppConfig } from "@/services/api/core/context"
import type { StepEventData } from "@/types/workflow"

export type { StepEventData }

export interface DoneEventData {
  session_id?: string
}

export type RepairSSEEvent =
  | { type: "step"; data: StepEventData }
  | { type: "done"; data: DoneEventData }

export async function startRepair(
  onEvent: (event: RepairSSEEvent) => void,
): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    const channel = createChannel<Record<string, unknown>>((data) => {
      const event = data.event as string
      if (event === "step") {
        onEvent({ type: "step", data: data.data as StepEventData })
      } else if (event === "done") {
        onEvent({ type: "done", data: data.data as DoneEventData })
        resolve()
      }
    })

    tauriInvoke("start_repair", {
      config: getAppConfig(),
      onEvent: channel,
    }).catch(reject)
  })
}

export async function fixStep(
  sessionId: string,
  stepId: string,
  issueDescription: string,
  onEvent: (event: RepairSSEEvent) => void,
): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    const channel = createChannel<Record<string, unknown>>((data) => {
      const event = data.event as string
      if (event === "step") {
        onEvent({ type: "step", data: data.data as StepEventData })
      } else if (event === "done") {
        onEvent({ type: "done", data: data.data as DoneEventData })
        resolve()
      }
    })

    tauriInvoke("fix_step", {
      request: {
        config: getAppConfig(),
        session_id: sessionId,
        step_id: stepId,
        issue_description: issueDescription,
      },
      onEvent: channel,
    }).catch(reject)
  })
}

export async function startCustomFix(
  problem: string,
  onEvent: (event: RepairSSEEvent) => void,
): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    const channel = createChannel<Record<string, unknown>>((data) => {
      const event = data.event as string
      if (event === "step") {
        onEvent({ type: "step", data: data.data as StepEventData })
      } else if (event === "done") {
        onEvent({ type: "done", data: data.data as DoneEventData })
        resolve()
      }
    })

    tauriInvoke("start_custom_fix", {
      request: {
        config: getAppConfig(),
        problem,
      },
      onEvent: channel,
    }).catch(reject)
  })
}
