import { Outlet, useLocation, useNavigate } from "react-router"
import { useEffect } from "react"
import { AppSidebar } from "@/components/app-sidebar"
import { SiteHeader } from "@/components/site-header"
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar"
import { useConfigStore } from "@/stores/config-store"

export default function DashboardLayout() {
  const { pathname } = useLocation()
  const navigate = useNavigate()
  const apiKey = useConfigStore((s) => s.modelConfig.apiKey)

  useEffect(() => {
    if (!apiKey && pathname !== "/") {
      navigate("/", { replace: true })
    }
  }, [apiKey, pathname, navigate])

  return (
    <SidebarProvider
      className="max-h-screen overflow-hidden"
      style={
        {
          "--header-height": "calc(var(--spacing) * 12)",
        } as React.CSSProperties
      }
    >
      <AppSidebar />
      <SidebarInset className="min-h-0">
        <SiteHeader />
        <div className="flex flex-1 flex-col min-h-0 overflow-y-auto overscroll-none no-scrollbar">
          <div className="@container/main flex flex-1 flex-col gap-2">
            <Outlet />
          </div>
        </div>
      </SidebarInset>
    </SidebarProvider>
  )
}
