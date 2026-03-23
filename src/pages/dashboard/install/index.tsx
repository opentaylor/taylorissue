import { useState, useRef } from "react"
import { useTranslation } from "react-i18next"
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardFooter,
  CardAction,
} from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Progress } from "@/components/ui/progress"
import { ErrorDialog } from "@/components/error-dialog"
import {
  ChainOfThought,
  ChainOfThoughtContent,
  ChainOfThoughtHeader,
  ChainOfThoughtStep,
  ChainOfThoughtSearchResults,
  ChainOfThoughtSearchResult,
} from "@/components/chain-of-thought"
import {
  MonitorIcon,
  CheckCircle2Icon,
  SettingsIcon,
  ShieldCheckIcon,
  DownloadIcon,
  LoaderIcon,
  XCircleIcon,
  RotateCcwIcon,
  RocketIcon,
  GitBranchIcon,
  HexagonIcon,
} from "lucide-react"
import { useInstallStore } from "@/stores/workflows/install-store"
import type { WorkflowStepStatus } from "@/types/workflow"

const STEP_KEYS = [
  "detectEnv",
  "installGit",
  "installNode",
  "installOpenClaw",
  "configure",
  "verify",
] as const

type StepKey = (typeof STEP_KEYS)[number]

const STEP_ICONS: Record<StepKey, typeof MonitorIcon> = {
  detectEnv: MonitorIcon,
  installGit: GitBranchIcon,
  installNode: HexagonIcon,
  installOpenClaw: DownloadIcon,
  configure: SettingsIcon,
  verify: ShieldCheckIcon,
}

function StepDetailBadges({ details }: { details: string[] }) {
  return (
    <ChainOfThoughtSearchResults>
      {details.map((detail) => (
        <ChainOfThoughtSearchResult key={detail}>
          {detail}
        </ChainOfThoughtSearchResult>
      ))}
    </ChainOfThoughtSearchResults>
  )
}

function StepStatusIcon({ status }: { status: WorkflowStepStatus }) {
  if (status === "active") {
    return <LoaderIcon className="size-3.5 animate-spin text-primary" />
  }
  if (status === "complete") {
    return <CheckCircle2Icon className="size-3.5 text-success" />
  }
  if (status === "error") {
    return <XCircleIcon className="size-3.5 text-destructive" />
  }
  return null
}

export default function InstallPage() {
  const { t } = useTranslation()
  const {
    steps,
    isRunning,
    isComplete,
    errorInfo,
    progressValue,
    start,
    reset,
  } = useInstallStore()

  const [showErrorDialog, setShowErrorDialog] = useState(false)
  const prevErrorInfo = useRef(errorInfo)
  if (errorInfo && errorInfo !== prevErrorInfo.current) {
    setShowErrorDialog(true)
  }
  prevErrorInfo.current = errorInfo

  const hasError = errorInfo !== null
  const hasStarted =
    isRunning || isComplete || hasError || steps.some((s) => s.status !== "pending")

  const errorStepLabel = errorInfo?.stepId
    ? t(`page.install.steps.${errorInfo.stepId}.label`, { defaultValue: "" })
    : ""

  return (
    <div className="flex flex-col gap-4 px-4 py-6 lg:px-6">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <DownloadIcon className="size-4" />
            {t("page.install.card.title")}
          </CardTitle>
          <CardDescription>
            {t("page.install.card.description")}
          </CardDescription>
          <CardAction>
            {isComplete ? (
              <Badge variant="secondary" className="h-9 rounded-lg px-2.5 text-success">
                {t("page.install.status.complete")}
              </Badge>
            ) : hasError ? (
              <Badge variant="secondary" className="h-9 rounded-lg px-2.5 text-destructive">
                {t("page.install.status.error")}
              </Badge>
            ) : isRunning ? (
              <Badge variant="secondary" className="h-9 rounded-lg px-2.5">
                <LoaderIcon className="size-3.5 animate-spin" />
              </Badge>
            ) : null}
          </CardAction>
        </CardHeader>

        <CardContent className="flex flex-col gap-4">
          {hasStarted && (
            <Progress value={progressValue}>
              <span className="text-base font-medium text-muted-foreground">
                {t("page.install.progress", {
                  current: steps.filter((s) => s.status === "complete").length,
                  total: steps.length,
                })}
              </span>
              <span className="ml-auto text-base tabular-nums text-muted-foreground">
                {progressValue}%
              </span>
            </Progress>
          )}

          {hasStarted && (
            <ChainOfThought defaultOpen>
              <ChainOfThoughtHeader label={t("page.install.chainOfThought")} />
              <ChainOfThoughtContent>
                {steps.map((step) => {
                  if (step.status === "pending") return null

                  const stepKey = step.id as StepKey
                  return (
                    <ChainOfThoughtStep
                      key={step.id}
                      icon={STEP_ICONS[stepKey]}
                      label={
                        <span className="flex items-center gap-2">
                          {t(`page.install.steps.${stepKey}.label`)}
                          <StepStatusIcon status={step.status} />
                        </span>
                      }
                      description={
                        step.status === "active"
                          ? t(`page.install.steps.${stepKey}.active`)
                          : step.status === "complete"
                            ? t(`page.install.steps.${stepKey}.complete`)
                            : step.status === "error"
                              ? step.error
                              : undefined
                      }
                      status={
                        step.status === "error"
                          ? "active"
                          : step.status === "complete"
                            ? "complete"
                            : step.status === "active"
                              ? "active"
                              : "pending"
                      }
                    >
                      {step.status === "complete" && step.details && (
                        <StepDetailBadges details={step.details} />
                      )}
                    </ChainOfThoughtStep>
                  )
                })}
              </ChainOfThoughtContent>
            </ChainOfThought>
          )}

          {!hasStarted && (
            <div className="flex flex-col items-center gap-3 py-6 text-center">
              <div className="flex size-12 items-center justify-center rounded-full bg-muted">
                <RocketIcon className="size-6 text-muted-foreground" />
              </div>
              <p className="text-base text-muted-foreground">
                {t("page.install.ready")}
              </p>
            </div>
          )}
        </CardContent>

        <CardFooter className="flex items-center gap-2">
          {!hasStarted && (
            <Button size="lg" onClick={() => void start()}>
              {t("page.install.startButton")}
            </Button>
          )}
          {hasError && (
            <Button variant="outline" size="lg" onClick={reset}>
              <RotateCcwIcon data-icon="inline-start" />
              {t("page.install.resetButton")}
            </Button>
          )}
          {isRunning && (
            <span className="text-base text-muted-foreground">
              {t("page.install.status.pleaseWait")}
            </span>
          )}
        </CardFooter>
      </Card>

      <ErrorDialog
        open={!!errorInfo && showErrorDialog}
        title={t("page.install.error.title")}
        message={errorInfo?.message ?? ""}
        stepLabel={errorStepLabel ? t("page.install.error.failedStep", { step: errorStepLabel }) : undefined}
        closeLabel={t("page.install.error.close")}
        onClose={() => setShowErrorDialog(false)}
      />
    </div>
  )
}
