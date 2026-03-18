import * as React from "react"
import { useTranslation } from "react-i18next"

import { NavMain } from "@/components/nav-main"
import { NavSecondary } from "@/components/nav-secondary"
import { SettingsDialog } from "@/components/settings-dialog"
import {
  Sidebar,
  SidebarContent,
  SidebarHeader,
  SidebarRail,
} from "@/components/ui/sidebar"
import {
  DownloadIcon,
  Trash2Icon,
  WrenchIcon,
  RocketIcon,
  HeartIcon,
  Settings2Icon,
  LayoutDashboardIcon,
} from "lucide-react"

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  const { t } = useTranslation()
  const [settingsOpen, setSettingsOpen] = React.useState(false)

  const navMain = [
    { title: t("sidebar.overview"), url: "/", icon: <LayoutDashboardIcon /> },
    { title: t("sidebar.install"), url: "/install", icon: <DownloadIcon /> },
    { title: t("sidebar.repair"), url: "/quick-fix", icon: <WrenchIcon />, className: "!text-primary" },
    { title: t("sidebar.uninstall"), url: "/uninstall", icon: <Trash2Icon /> },
    { title: t("sidebar.use"), url: "/message", icon: <RocketIcon /> },
    {
      title: t("sidebar.raise"),
      url: "/skill",
      icon: <HeartIcon />,
      items: [
        { title: t("sidebar.mySkills"), url: "/skill/my" },
        { title: t("sidebar.marketplace"), url: "/skill/marketplace" },
      ],
    },
  ]

  const navSecondary = [
    { title: t("sidebar.settings"), url: "#", icon: <Settings2Icon /> },
  ]

  return (
    <>
      <Sidebar className="border-r-0" {...props}>
        <SidebarHeader>
          <div className="flex items-center gap-2 px-2 py-1.5">
            <img
              src="/logo.png"
              alt="Logo"
              className="size-14"
            />
            <div className="flex min-w-0 flex-col">
              <span className="truncate font-semibold text-base leading-tight">{t("app.name")}</span>
              <span className="truncate text-xs text-muted-foreground">{t("app.subtitle")}</span>
            </div>
          </div>
          <NavMain items={navMain} />
        </SidebarHeader>
        <SidebarContent>
          <NavSecondary
            items={navSecondary}
            className="mt-auto"
            onItemClick={(title) => {
              if (title === t("sidebar.settings")) {
                setSettingsOpen(true)
              }
            }}
          />
        </SidebarContent>
        <SidebarRail />
      </Sidebar>
      <SettingsDialog open={settingsOpen} onOpenChange={setSettingsOpen} />
    </>
  )
}
