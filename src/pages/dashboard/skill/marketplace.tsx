import { useEffect, useRef, useState } from "react"
import { useTranslation } from "react-i18next"
import { useSkillStore } from "@/stores/skill-store"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { ErrorDialog } from "@/components/error-dialog"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import {
  CheckCircleIcon,
  ExternalLinkIcon,
  LoaderIcon,
  SearchIcon,
} from "lucide-react"
import { PageLoading } from "./shared"

const DEFAULT_QUERIES = ["automation", "database", "web", "devops", "writing"]

interface InstallResult {
  skillName: string
  ok: boolean
  outputs: string[]
}

function InstallSuccessDialog({
  result,
  onOpenChange,
}: {
  result: InstallResult | null
  onOpenChange: (open: boolean) => void
}) {
  const { t } = useTranslation()

  return (
    <Dialog open={Boolean(result)} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <CheckCircleIcon className="size-5 text-green-500" />
            {t("page.skill.installSuccess")}
          </DialogTitle>
          <DialogDescription>{result?.skillName}</DialogDescription>
        </DialogHeader>

        {result && result.outputs.length > 0 && (
          <div className="flex flex-col gap-2">
            <p className="text-base font-medium">
              {t("page.skill.installLog")}
            </p>
            <div className="max-h-60 overflow-y-auto rounded-md bg-muted p-3">
              {result.outputs.map((line, i) => (
                <p key={i} className="font-mono text-sm leading-relaxed">
                  {line}
                </p>
              ))}
            </div>
          </div>
        )}

        <DialogFooter>
          <Button
            variant="outline"
            size="lg"
            onClick={() => onOpenChange(false)}
          >
            {t("page.skill.close")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

export default function MarketplacePage() {
  const { t } = useTranslation()
  const { hubResults, hubLoading, installingHubSkills, searchHub, installHubSkill } =
    useSkillStore()

  const [query, setQuery] = useState("")
  const [hasSearched, setHasSearched] = useState(false)
  const [successResult, setSuccessResult] = useState<InstallResult | null>(null)
  const [errorResult, setErrorResult] = useState<InstallResult | null>(null)
  const didInit = useRef(false)

  useEffect(() => {
    if (didInit.current) return
    didInit.current = true
    if (hubResults.length === 0) {
      const pick = DEFAULT_QUERIES[Math.floor(Math.random() * DEFAULT_QUERIES.length)]
      searchHub(pick)
    }
  }, [hubResults.length, searchHub])

  const handleSearch = () => {
    if (query.trim()) {
      setHasSearched(true)
      searchHub(query.trim())
    }
  }

  const handleInstall = async (slug: string) => {
    const result = await installHubSkill(slug)
    if (result) {
      const entry = { skillName: slug, ok: result.ok, outputs: result.outputs }
      if (result.ok) {
        setSuccessResult(entry)
      } else {
        setErrorResult(entry)
      }
    }
  }

  return (
    <div className="flex flex-col gap-4 px-4 py-6 lg:px-6">
      <div className="flex items-center gap-2">
        <div className="relative flex-1">
          <SearchIcon className="absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            id="hub-search"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") handleSearch()
            }}
            placeholder={t("page.skill.hubSearchPlaceholder")}
            className="h-9 pl-8"
          />
        </div>
        <Button
          size="lg"
          onClick={handleSearch}
          disabled={hubLoading || !query.trim()}
        >
          {hubLoading ? (
            <LoaderIcon className="size-4 animate-spin" />
          ) : (
            t("page.skill.search")
          )}
        </Button>
      </div>

      {hubLoading ? (
        <PageLoading text={t("page.skill.loadingMarketplace")} />
      ) : hubResults.length === 0 && hasSearched ? (
        <div className="py-8 text-center text-base text-muted-foreground">
          {t("page.skill.hubNoResults")}
        </div>
      ) : hubResults.length > 0 ? (
        <>
          {!hasSearched && (
            <p className="text-base text-muted-foreground">
              {t("page.skill.hubRecommended")}
            </p>
          )}
          <div className="overflow-hidden rounded-lg border">
            <Table className="table-fixed">
              <TableHeader>
                <TableRow>
                  <TableHead className="px-4">
                    {t("page.skill.skillName")}
                  </TableHead>
                  <TableHead className="w-20 text-center">
                    {t("page.skill.homepage")}
                  </TableHead>
                  <TableHead className="w-24 text-center">
                    {t("page.skill.actions")}
                  </TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {hubResults.map((skill) => (
                  <TableRow key={skill.slug}>
                    <TableCell className="overflow-hidden px-4">
                      <a
                        href={`https://clawhub.ai/skills/${skill.slug}`}
                        target="_blank"
                        rel="noreferrer"
                        className="block truncate text-base font-medium hover:underline"
                      >
                        {skill.name}
                      </a>
                      <p className="mt-0.5 truncate text-base text-muted-foreground">
                        {skill.summary}
                      </p>
                    </TableCell>
                    <TableCell className="text-center">
                      <a
                        href={`https://clawhub.ai/skills/${skill.slug}`}
                        target="_blank"
                        rel="noreferrer"
                        className="inline-flex items-center gap-1 text-base text-muted-foreground hover:text-primary"
                      >
                        {t("page.skill.link")}
                        <ExternalLinkIcon className="size-3" />
                      </a>
                    </TableCell>
                    <TableCell className="text-center">
                      <Button
                        variant="default"
                        size="lg"
                        disabled={installingHubSkills.has(skill.slug)}
                        onClick={() => handleInstall(skill.slug)}
                      >
                        {installingHubSkills.has(skill.slug) ? (
                          <LoaderIcon className="size-4 animate-spin" />
                        ) : (
                          t("page.skill.install")
                        )}
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </div>
        </>
      ) : null}

      <InstallSuccessDialog
        result={successResult}
        onOpenChange={(open) => {
          if (!open) setSuccessResult(null)
        }}
      />

      <ErrorDialog
        open={!!errorResult}
        title={t("page.skill.installFailed")}
        message={errorResult?.outputs.join("\n") ?? ""}
        stepLabel={errorResult?.skillName}
        closeLabel={t("page.skill.close")}
        onClose={() => setErrorResult(null)}
      />
    </div>
  )
}
