import { createBrowserRouter, Navigate } from "react-router"
import DashboardLayout from "@/layouts/DashboardLayout"
import DashboardHome from "@/pages/dashboard"
import InstallPage from "@/pages/dashboard/install"
import UninstallPage from "@/pages/dashboard/uninstall"
import QuickFixPage from "@/pages/dashboard/quick-fix"
import MySkillsPage from "@/pages/dashboard/skill/my-skills"
import MarketplacePage from "@/pages/dashboard/skill/marketplace"

import MessagePage from "@/pages/dashboard/message"

export const router = createBrowserRouter([
  {
    path: "/",
    element: <DashboardLayout />,
    children: [
      { index: true, element: <DashboardHome /> },
      { path: "install", element: <InstallPage /> },
      { path: "quick-fix", element: <QuickFixPage /> },
      { path: "uninstall", element: <UninstallPage /> },
      { path: "message", element: <MessagePage /> },
      { path: "skill/my", element: <MySkillsPage /> },
      { path: "skill/marketplace", element: <MarketplacePage /> },
      { path: "skill", element: <Navigate to="/skill/my" replace /> },
    ],
  },
])
