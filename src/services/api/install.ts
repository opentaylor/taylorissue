import { tauriInvoke, createChannel, getAppConfig } from "@/services/api/core/context"
import type { StepEventData } from "@/types/workflow"

export type { StepEventData }

export type InstallSSEEvent =
  | { type: "step"; data: StepEventData }
  | { type: "done" }

export async function startInstall(
  onEvent: (event: InstallSSEEvent) => void,
): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    const channel = createChannel<Record<string, unknown>>((data) => {
      const event = data.event as string
      if (event === "step") {
        onEvent({ type: "step", data: data.data as StepEventData })
      } else if (event === "done") {
        onEvent({ type: "done" })
        resolve()
      }
    })

    tauriInvoke("start_install", {
      config: getAppConfig(),
      onEvent: channel,
    }).catch(reject)
  })
}
