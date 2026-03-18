import { tauriInvoke, createChannel, getAppConfig } from "@/services/api/core/context"
import type { StepEventData } from "@/types/workflow"

export type { StepEventData }

export type UninstallSSEEvent =
  | { type: "step"; data: StepEventData }
  | { type: "done" }

export async function startUninstall(
  selectedOptions: string[],
  onEvent: (event: UninstallSSEEvent) => void,
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

    tauriInvoke("start_uninstall", {
      request: {
        config: getAppConfig(),
        selected_options: selectedOptions,
      },
      onEvent: channel,
    }).catch(reject)
  })
}
