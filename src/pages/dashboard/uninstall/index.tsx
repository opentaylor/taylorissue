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
import { Checkbox } from "@/components/ui/checkbox"
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
  CheckCircle2Icon,
  FileXIcon,
  FolderXIcon,
  LoaderIcon,
  PackageXIcon,
  SquareIcon,
  Trash2Icon,
  XCircleIcon,
} from "lucide-react"
import {
  DANGEROUS_OPTIONS,
  OPTION_KEYS,
  REQUIRED_OPTIONS,
  type OptionKey,
  useUninstallStore,
} from "@/stores/workflows/uninstall-store"
import type { WorkflowStepStatus } from "@/types/workflow"

const OPTION_ICONS: Record<OptionKey, typeof Trash2Icon> = {
  stopServices: SquareIcon,
  removePackage: PackageXIcon,
  deleteWorkspace: FolderXIcon,
  deleteConfig: FileXIcon,
  deleteData: Trash2Icon,
}

const STEP_ICONS: Record<string, typeof Trash2Icon> = {
  stopServices: SquareIcon,
  removePackage: PackageXIcon,
  deleteWorkspace: FolderXIcon,
  deleteConfig: FileXIcon,
  deleteData: Trash2Icon,
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

export default function UninstallPage() {
  const { t } = useTranslation()
  const {
    selectedOptions,
    steps,
    isRunning,
    isComplete,
    errorInfo,
    progressValue,
    toggleOption,
    start,
    reset,
    clearError,
  } = useUninstallStore()

  const hasError = errorInfo !== null
  const hasStarted =
    isRunning || isComplete || hasError || steps.some((s) => s.status !== "pending")

  const errorStepLabel = errorInfo?.stepId
    ? t(`page.uninstall.steps.${errorInfo.stepId}.label`, { defaultValue: "" })
    : ""

  return (
    <div className="flex flex-col gap-4 px-4 py-6 lg:px-6">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Trash2Icon className="size-4" />
            {t("page.uninstall.card.title")}
          </CardTitle>
          <CardDescription>
            {t("page.uninstall.card.description")}
          </CardDescription>
          <CardAction>
            {isComplete ? (
              <Badge variant="secondary" className="h-9 rounded-lg px-2.5 text-success">
                {t("page.uninstall.status.complete")}
              </Badge>
            ) : hasError ? (
              <Badge variant="secondary" className="h-9 rounded-lg px-2.5 text-destructive">
                {t("page.uninstall.status.error")}
              </Badge>
            ) : isRunning ? (
              <Badge variant="secondary" className="h-9 rounded-lg px-2.5">
                <LoaderIcon className="size-3.5 animate-spin" />
              </Badge>
            ) : null}
          </CardAction>
        </CardHeader>

        <CardContent className="flex flex-col gap-4">
          {!hasStarted && (
            <div className="flex flex-col gap-3">
              <p className="text-base font-medium">
                {t("page.uninstall.selectItems")}
              </p>
              <div className="flex flex-col gap-2">
                {OPTION_KEYS.map((key) => {
                  const isRequired = REQUIRED_OPTIONS.includes(key)
                  const isDangerous = DANGEROUS_OPTIONS.includes(key)
                  const isChecked = selectedOptions.has(key)
                  const Icon = OPTION_ICONS[key]

                  return (
                    <label
                      key={key}
                      className="flex cursor-pointer items-center gap-3 rounded-lg border p-3 transition-colors hover:bg-muted/50 has-data-[state=checked]:border-primary/30 has-data-[state=checked]:bg-primary/5 data-[disabled]:cursor-not-allowed data-[disabled]:opacity-60"
                      data-disabled={isRequired || undefined}
                    >
                      <Checkbox
                        checked={isChecked}
                        onCheckedChange={() => toggleOption(key)}
                        disabled={isRequired}
                      />
                      <Icon className="size-4 text-muted-foreground" />
                      <div className="flex flex-1 flex-col gap-0.5">
                        <span className="text-base font-medium">
                          {t(`page.uninstall.options.${key}.label`)}
                        </span>
                        <span className="text-base text-muted-foreground">
                          {t(`page.uninstall.options.${key}.description`)}
                        </span>
                      </div>
                      {isRequired && (
                        <Badge variant="outline" className="h-9 rounded-lg px-2.5 text-base">
                          {t("page.uninstall.required")}
                        </Badge>
                      )}
                      {isDangerous && (
                        <Badge variant="destructive" className="h-9 rounded-lg px-2.5 text-base">
                          {t("page.uninstall.caution")}
                        </Badge>
                      )}
                    </label>
                  )
                })}
              </div>
            </div>
          )}

          {hasStarted && (
            <Progress value={progressValue}>
              <span className="text-base font-medium text-muted-foreground">
                {t("page.uninstall.progress", {
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
              <ChainOfThoughtHeader
                label={t("page.uninstall.chainOfThought")}
              />
              <ChainOfThoughtContent>
                {steps.map((step) => {
                  if (step.status === "pending") return null

                  return (
                    <ChainOfThoughtStep
                      key={step.id}
                      icon={STEP_ICONS[step.id] || Trash2Icon}
                      label={
                        <span className="flex items-center gap-2">
                          {t(`page.uninstall.steps.${step.id}.label`)}
                          <StepStatusIcon status={step.status} />
                        </span>
                      }
                      description={
                        step.status === "active"
                          ? t(`page.uninstall.steps.${step.id}.active`)
                          : step.status === "complete"
                            ? t(`page.uninstall.steps.${step.id}.complete`)
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
        </CardContent>

        <CardFooter className="flex items-center gap-2">
          {!hasStarted && (
            <Button variant="destructive" size="lg" onClick={() => void start()}>
              {t("page.uninstall.startButton")}
            </Button>
          )}
          {hasError && (
            <Button variant="outline" size="lg" onClick={reset}>
              {t("page.uninstall.resetButton")}
            </Button>
          )}
          {isRunning && (
            <span className="text-base text-muted-foreground">
              {t("page.uninstall.status.pleaseWait")}
            </span>
          )}
        </CardFooter>
      </Card>

      <ErrorDialog
        open={!!errorInfo}
        title={t("page.uninstall.error.title")}
        message={errorInfo?.message ?? ""}
        stepLabel={errorStepLabel ? t("page.uninstall.error.failedStep", { step: errorStepLabel }) : undefined}
        closeLabel={t("page.uninstall.error.close")}
        onClose={clearError}
      />
    </div>
  )
}
