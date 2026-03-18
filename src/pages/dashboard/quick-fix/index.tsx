import { useTranslation } from "react-i18next"
import { useEffect } from "react"
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
import { Textarea } from "@/components/ui/textarea"
import {
  ChainOfThought,
  ChainOfThoughtContent,
  ChainOfThoughtHeader,
  ChainOfThoughtStep,
  ChainOfThoughtSearchResults,
  ChainOfThoughtSearchResult,
} from "@/components/chain-of-thought"
import {
  SearchIcon,
  CheckCircle2Icon,
  LoaderIcon,
  XCircleIcon,
  WrenchIcon,
  ScanSearchIcon,
  ActivityIcon,
  WifiIcon,
  SparklesIcon,
  FileSearchIcon,
  StethoscopeIcon,
  CircleCheckIcon,
  FileTextIcon,
  ShieldCheckIcon,
} from "lucide-react"
import {
  CUSTOM_FIX_STEP_KEYS,
  useRepairStore,
} from "@/stores/workflows/repair-store"
import type { WorkflowStepStatus } from "@/types/workflow"

const STEP_KEYS = [
  "checkGateway",
  "checkConfig",
  "checkModelRequest",
  "runDoctor",
] as const

type StepKey = (typeof STEP_KEYS)[number]

const STEP_ICONS: Record<StepKey, typeof ScanSearchIcon> = {
  checkGateway: ActivityIcon,
  checkConfig: FileTextIcon,
  checkModelRequest: WifiIcon,
  runDoctor: ShieldCheckIcon,
}

function StepStatusIcon({
  status,
  hasIssue,
  fixing,
  fixed,
}: {
  status: WorkflowStepStatus
  hasIssue?: boolean
  fixing?: boolean
  fixed?: boolean
}) {
  if (fixing) {
    return <LoaderIcon className="size-3.5 animate-spin text-primary" />
  }
  if (fixed) {
    return <CheckCircle2Icon className="size-3.5 text-success" />
  }
  if (status === "active") {
    return <LoaderIcon className="size-3.5 animate-spin text-primary" />
  }
  if (status === "error") {
    return <XCircleIcon className="size-3.5 text-destructive" />
  }
  if (status === "complete" && hasIssue) {
    return <XCircleIcon className="size-3.5 text-warning" />
  }
  if (status === "complete") {
    return <CheckCircle2Icon className="size-3.5 text-success" />
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

type CustomFixStepKey = (typeof CUSTOM_FIX_STEP_KEYS)[number]

const CUSTOM_FIX_ICONS: Record<CustomFixStepKey, typeof SearchIcon> = {
  analyze: FileSearchIcon,
  diagnose: StethoscopeIcon,
  fix: WrenchIcon,
  verify: CircleCheckIcon,
}

export default function QuickFixPage() {
  const { t } = useTranslation()

  const {
    steps,
    isRunning,
    isComplete,
    sessionId,
    progressValue,
    issueCount,
    startScan,
    rescan,
    fixSingleStep,
    customText,
    setCustomText,
    customFixSteps,
    isCustomFixing,
    customFixDone,
    customFixError,
    startCustom,
    resetCustom,
  } = useRepairStore()

  const hasStarted =
    isRunning || isComplete || steps.some((s) => s.status !== "pending")

  useEffect(() => {
    void startScan()
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  const customFixHasStarted =
    isCustomFixing ||
    customFixDone ||
    customFixSteps.some((s) => s.status !== "pending")

  return (
    <div className="flex flex-col gap-4 px-4 py-6 lg:px-6">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <ScanSearchIcon className="size-4" />
            {t("page.quickFix.scan.title")}
          </CardTitle>
          <CardDescription>
            {t("page.quickFix.scan.description")}
          </CardDescription>
          <CardAction>
            {isComplete && issueCount === 0 ? (
              <Badge variant="secondary" className="h-9 rounded-lg px-2.5 text-success">
                {t("page.quickFix.scan.noIssues")}
              </Badge>
            ) : isComplete && issueCount > 0 ? (
              <Badge variant="secondary" className="h-9 rounded-lg px-2.5 text-warning">
                {t("page.quickFix.scan.foundIssues", { count: issueCount })}
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
                {t("page.quickFix.scan.progress")}
              </span>
              <span className="ml-auto text-base tabular-nums text-muted-foreground">
                {progressValue}%
              </span>
            </Progress>
          )}

          {hasStarted && (
            <ChainOfThought defaultOpen>
              <ChainOfThoughtHeader
                label={t("page.quickFix.scan.chainOfThought")}
              />
              <ChainOfThoughtContent>
                {steps.map((step) => {
                  if (step.status === "pending") return null

                  const showFixButton =
                    isComplete &&
                    !isRunning &&
                    (step.hasIssue || step.status === "error") &&
                    !step.fixed &&
                    !!sessionId

                  return (
                    <ChainOfThoughtStep
                      key={step.id}
                      icon={STEP_ICONS[step.id as StepKey] || ScanSearchIcon}
                      label={
                        <span className="flex items-center gap-2">
                          {t(`page.quickFix.steps.${step.id}.label`)}
                          <StepStatusIcon
                            status={step.status}
                            hasIssue={step.hasIssue}
                            fixing={step.fixing}
                            fixed={step.fixed}
                          />
                        </span>
                      }
                      description={
                        step.fixing
                          ? t("page.quickFix.issues.fixing")
                          : step.fixed
                            ? t("page.quickFix.issues.fixed")
                            : step.fixError
                              ? step.fixError
                              : step.status === "active"
                                ? t(`page.quickFix.steps.${step.id}.active`)
                                : step.status === "complete"
                                  ? t(`page.quickFix.steps.${step.id}.complete`)
                                  : step.status === "error"
                                    ? step.error
                                    : undefined
                      }
                      status={
                        step.fixing
                          ? "active"
                          : step.fixed
                            ? "complete"
                            : step.status === "error"
                              ? "active"
                              : step.status === "complete"
                                ? "complete"
                                : step.status === "active"
                                  ? "active"
                                  : "pending"
                      }
                    >
                      {step.fixed && step.fixDetails && (
                        <StepDetailBadges details={step.fixDetails} />
                      )}
                      {!step.fixed && step.status === "complete" && step.details && (
                        <StepDetailBadges details={step.details} />
                      )}
                      {showFixButton && (
                        <div>
                          <Button
                            size="lg"
                            disabled={step.fixing}
                            onClick={() => fixSingleStep(step.id)}
                          >
                            {t("page.quickFix.issues.fix")}
                          </Button>
                        </div>
                      )}
                    </ChainOfThoughtStep>
                  )
                })}
              </ChainOfThoughtContent>
            </ChainOfThought>
          )}
        </CardContent>

        <CardFooter className="flex items-center gap-2">
          {(isComplete || (!isRunning && hasStarted)) && (
            <Button size="lg" onClick={() => void rescan()}>
              {t("page.quickFix.scan.rescanButton")}
            </Button>
          )}
          {isRunning && (
            <span className="text-base text-muted-foreground">
              {t("page.quickFix.scan.progress")}
            </span>
          )}
        </CardFooter>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <SparklesIcon className="size-4" />
            {t("page.quickFix.custom.title")}
          </CardTitle>
          <CardDescription>
            {t("page.quickFix.custom.description")}
          </CardDescription>
          {customFixDone && !customFixError && (
            <CardAction>
              <Badge variant="secondary" className="h-9 px-2.5 text-success">
                {t("page.quickFix.custom.done")}
              </Badge>
            </CardAction>
          )}
          {customFixError && (
            <CardAction>
              <Badge variant="secondary" className="h-9 rounded-lg px-2.5 text-destructive">
                {t("page.quickFix.status.error")}
              </Badge>
            </CardAction>
          )}
        </CardHeader>

        <CardContent className="flex flex-col gap-4">
          <Textarea
            placeholder={t("page.quickFix.custom.placeholder")}
            value={customText}
            onChange={(e) => setCustomText(e.target.value)}
            disabled={isCustomFixing}
            className="min-h-24 resize-none"
          />

          {customFixHasStarted && (
            <ChainOfThought defaultOpen>
              <ChainOfThoughtHeader
                label={t("page.quickFix.custom.chainOfThought")}
              />
              <ChainOfThoughtContent>
                {customFixSteps.map((step) => {
                  if (step.status === "pending") return null
                  const stepKey = step.id as CustomFixStepKey
                  return (
                    <ChainOfThoughtStep
                      key={step.id}
                      icon={CUSTOM_FIX_ICONS[step.id as CustomFixStepKey] || FileSearchIcon}
                      label={
                        <span className="flex items-center gap-2">
                          {t(`page.quickFix.custom.steps.${stepKey}.label`)}
                          {step.status === "active" && (
                            <LoaderIcon className="size-3.5 animate-spin text-primary" />
                          )}
                          {step.status === "complete" && (
                            <CheckCircle2Icon className="size-3.5 text-success" />
                          )}
                          {step.status === "error" && (
                            <XCircleIcon className="size-3.5 text-destructive" />
                          )}
                        </span>
                      }
                      description={
                        step.status === "active"
                          ? t(`page.quickFix.custom.steps.${stepKey}.active`)
                          : step.status === "complete"
                            ? t(
                                `page.quickFix.custom.steps.${stepKey}.complete`
                              )
                            : step.status === "error"
                              ? step.error
                              : undefined
                      }
                      status={
                        step.status === "complete"
                          ? "complete"
                          : step.status === "active"
                            ? "active"
                            : step.status === "error"
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
          {!isCustomFixing && !customFixDone && !customFixError && (
            <Button size="lg" onClick={() => void startCustom()} disabled={!customText.trim()}>
              {t("page.quickFix.custom.fixButton")}
            </Button>
          )}
          {(customFixDone || customFixError) && !isCustomFixing && (
            <Button size="lg" onClick={resetCustom}>
              {t("page.quickFix.custom.otherIssue")}
            </Button>
          )}
          {isCustomFixing && (
            <span className="text-base text-muted-foreground">
              {t("page.quickFix.issues.fixing")}
            </span>
          )}
        </CardFooter>
      </Card>
    </div>
  )
}
