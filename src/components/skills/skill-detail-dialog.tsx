import { useTranslation } from "react-i18next"
import type { Skill } from "@/types/skills"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { ExternalLinkIcon } from "lucide-react"

interface SkillDetailDialogProps {
  skill: Skill | null
  onOpenChange: (open: boolean) => void
  onUninstall: (name: string) => void
}

export function SkillDetailDialog({
  skill,
  onOpenChange,
  onUninstall,
}: SkillDetailDialogProps) {
  const { t } = useTranslation()

  const hasMissing =
    skill &&
    (skill.missing.bins.length > 0 ||
      skill.missing.env.length > 0 ||
      skill.missing.config.length > 0 ||
      skill.missing.os.length > 0)

  return (
    <Dialog open={Boolean(skill)} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>
            {skill?.emoji && <span className="mr-1.5">{skill.emoji}</span>}
            {skill?.name}
          </DialogTitle>
          <DialogDescription>{skill?.description}</DialogDescription>
        </DialogHeader>

        {skill && (
          <div className="flex flex-col gap-3">
            <div className="flex items-center gap-2">
              <span className="text-base text-muted-foreground">
                {t("page.skill.source")}:
              </span>
              <Badge variant="secondary">
                {skill.bundled
                  ? t("page.skill.bundled")
                  : t("page.skill.personal")}
              </Badge>
            </div>

            <div className="flex items-center gap-2">
              <span className="text-base text-muted-foreground">
                {t("page.skill.status")}:
              </span>
              <Badge variant={skill.eligible ? "default" : "outline"}>
                {skill.eligible
                  ? t("page.skill.eligible")
                  : t("page.skill.notEligible")}
              </Badge>
            </div>

            {skill.homepage && (
              <div className="flex items-center gap-2">
                <span className="text-base text-muted-foreground">
                  {t("page.skill.homepage")}:
                </span>
                <a
                  href={skill.homepage}
                  target="_blank"
                  rel="noreferrer"
                  className="inline-flex items-center gap-1 text-base text-primary hover:underline"
                >
                  {new URL(skill.homepage).hostname}
                  <ExternalLinkIcon className="size-3" />
                </a>
              </div>
            )}

            {hasMissing && (
              <div className="flex flex-col gap-1.5">
                <span className="text-base text-muted-foreground">
                  {t("page.skill.missingRequirements")}:
                </span>
                <div className="flex flex-wrap gap-1.5">
                  {skill.missing.bins.map((b) => (
                    <Badge key={b} variant="destructive">{b}</Badge>
                  ))}
                  {skill.missing.env.map((e) => (
                    <Badge key={e} variant="outline">{e}</Badge>
                  ))}
                  {skill.missing.config.map((c) => (
                    <Badge key={c} variant="outline">{c}</Badge>
                  ))}
                  {skill.missing.os.map((o) => (
                    <Badge key={o} variant="outline">{o}</Badge>
                  ))}
                </div>
              </div>
            )}

            {skill.install.length > 0 && (
              <div className="flex flex-col gap-1.5">
                <span className="text-base text-muted-foreground">
                  {t("page.skill.installHints")}:
                </span>
                <ul className="list-inside list-disc text-base">
                  {skill.install.map((inst) => (
                    <li key={inst.id}>{inst.label}</li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        )}

        <DialogFooter>
          {skill && !skill.bundled && (
            <Button
              variant="destructive"
              size="lg"
              onClick={() => {
                onUninstall(skill.name)
                onOpenChange(false)
              }}
            >
              {t("page.skill.uninstall")}
            </Button>
          )}
          <Button variant="outline" size="lg" onClick={() => onOpenChange(false)}>
            {t("page.skill.close")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
