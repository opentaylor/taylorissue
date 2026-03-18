import { useTranslation } from "react-i18next"
import { useLocation } from "react-router"
import { Separator } from "@/components/ui/separator"
import { SidebarTrigger } from "@/components/ui/sidebar"

const pageTitleKeys: Record<string, string> = {
  "/": "page.dashboard.title",
  "/install": "page.install.title",
  "/uninstall": "page.uninstall.title",
  "/quick-fix": "page.quickFix.title",
  "/skill": "page.skill.title",
  "/message": "page.message.title",
}

export function SiteHeader() {
  const { t } = useTranslation()
  const { pathname } = useLocation()
  const titleKey = pageTitleKeys[pathname] ?? "page.dashboard.title"

  return (
    <header className="flex h-(--header-height) shrink-0 items-center gap-2 border-b transition-[width,height] ease-linear group-has-data-[collapsible=icon]/sidebar-wrapper:h-(--header-height)">
      <div className="flex w-full items-center gap-1 px-4 lg:gap-2 lg:px-6">
        <SidebarTrigger className="-ml-1" />
        <Separator
          orientation="vertical"
          className="mx-2 h-4 data-vertical:self-auto"
        />
        <h1 className="text-base font-medium">{t(titleKey)}</h1>
      </div>
    </header>
  )
}
