import { useEffect, useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import { useSkillStore } from "@/stores/skill-store"
import type { Skill } from "@/types/skills"
import { SkillDetailDialog } from "@/components/skills/skill-detail-dialog"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { DownloadIcon, LoaderIcon } from "lucide-react"
import { PageLoading, EmptyState } from "./shared"

export default function MySkillsPage() {
  const { t } = useTranslation()
  const {
    skills,
    skillsLoading,
    uninstallingSkills,
    fetchSkills,
    uninstallSkill,
  } = useSkillStore()

  const [selectedSkill, setSelectedSkill] = useState<Skill | null>(null)

  useEffect(() => {
    fetchSkills()
  }, [fetchSkills])

  const eligibleSkills = useMemo(
    () => skills.filter((s) => s.eligible),
    [skills],
  )

  return (
    <div className="flex flex-col gap-4 px-4 py-6 lg:px-6">
      {skillsLoading ? (
        <PageLoading text={t("page.skill.loadingMySkills")} />
      ) : eligibleSkills.length === 0 ? (
        <EmptyState
          icon={<DownloadIcon className="size-10" />}
          title={t("page.skill.noInstalledSkills")}
          description={t("page.skill.noInstalledSkillsDesc")}
        />
      ) : (
        <div className="overflow-hidden rounded-lg border">
          <Table className="table-fixed">
            <TableHeader>
              <TableRow>
                <TableHead className="px-4">
                  {t("page.skill.skillName")}
                </TableHead>
                <TableHead className="w-24 text-center">
                  {t("page.skill.source")}
                </TableHead>
                <TableHead className="w-24 text-center">
                  {t("page.skill.status")}
                </TableHead>
                <TableHead className="w-28 text-center">
                  {t("page.skill.actions")}
                </TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {eligibleSkills.map((skill) => (
                <TableRow key={skill.name}>
                  <TableCell className="overflow-hidden px-4">
                    <button
                      type="button"
                      className="block truncate text-left text-base font-medium hover:underline"
                      onClick={() => setSelectedSkill(skill)}
                    >
                      {skill.emoji && (
                        <span className="mr-1.5">{skill.emoji}</span>
                      )}
                      {skill.name}
                    </button>
                    <p className="mt-0.5 truncate text-base text-muted-foreground">
                      {skill.description}
                    </p>
                  </TableCell>
                  <TableCell className="text-center">
                    <Badge
                      variant="secondary"
                      className="h-9 rounded-lg px-2.5"
                    >
                      {skill.bundled
                        ? t("page.skill.bundled")
                        : t("page.skill.personal")}
                    </Badge>
                  </TableCell>
                  <TableCell className="text-center">
                    <Badge variant="default" className="h-9 rounded-lg px-2.5">
                      {t("page.skill.eligible")}
                    </Badge>
                  </TableCell>
                  <TableCell className="text-center">
                    {!skill.bundled && (
                      <Button
                        variant="outline"
                        size="lg"
                        disabled={uninstallingSkills.has(skill.name)}
                        onClick={() => uninstallSkill(skill.name)}
                      >
                        {uninstallingSkills.has(skill.name) ? (
                          <LoaderIcon className="size-4 animate-spin" />
                        ) : (
                          t("page.skill.uninstall")
                        )}
                      </Button>
                    )}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      )}

      <SkillDetailDialog
        skill={selectedSkill}
        onOpenChange={(open) => {
          if (!open) setSelectedSkill(null)
        }}
        onUninstall={uninstallSkill}
      />
    </div>
  )
}
