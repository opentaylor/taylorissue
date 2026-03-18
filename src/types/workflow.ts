export type WorkflowStepStatus = "pending" | "active" | "complete" | "error"

export interface StepEventData {
  step_id: string
  status: Extract<WorkflowStepStatus, "active" | "complete" | "error">
  description?: string
  details?: string[]
  error?: string
  has_issue?: boolean
}

export interface WorkflowDoneEventData {
  session_id?: string
}
