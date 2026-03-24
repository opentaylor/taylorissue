import * as React from "react"
import { useTranslation } from "react-i18next"

import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle,
} from "@/components/ui/dialog"
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarProvider,
} from "@/components/ui/sidebar"
import { Button } from "@/components/ui/button"
import {
  Field,
  FieldDescription,
  FieldGroup,
  FieldLabel,
} from "@/components/ui/field"
import { Input } from "@/components/ui/input"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
} from "@/components/ui/select"
import { SettingsIcon, ServerIcon, SlidersHorizontalIcon } from "lucide-react"
import { useConfigStore } from "@/stores/config-store"
import { useTheme } from "next-themes"
import { getCurrentWindow, type Theme } from "@tauri-apps/api/window"

const LANGUAGE_LABELS: Record<string, string> = {
  "zh-CN": "简体中文",
  "en-US": "English",
}

function GeneralSettings() {
  const { t, i18n } = useTranslation()
  const { language, setLanguage } = useConfigStore()
  const { theme, setTheme } = useTheme()

  const themeLabels: Record<string, string> = {
    system: t("settings.themeSystem"),
    light: t("settings.themeLight"),
    dark: t("settings.themeDark"),
  }

  const handleSave = (e: React.FormEvent) => {
    e.preventDefault()
  }

  return (
    <form id="general-settings" onSubmit={handleSave} className="flex flex-1 flex-col">
      <FieldGroup className="flex-1">
        <Field>
          <FieldLabel htmlFor="language">{t("settings.language")}</FieldLabel>
          <Select
            value={language}
            onValueChange={(v) => {
              if (v === "zh-CN" || v === "en-US") {
                setLanguage(v)
                i18n.changeLanguage(v)
              }
            }}
          >
            <SelectTrigger id="language">
              <span data-slot="select-value" className="flex flex-1 text-left">
                {LANGUAGE_LABELS[language] ?? language}
              </span>
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="zh-CN">简体中文</SelectItem>
              <SelectItem value="en-US">English</SelectItem>
            </SelectContent>
          </Select>
          <FieldDescription>{t("settings.languageDescription")}</FieldDescription>
        </Field>
        <Field>
          <FieldLabel htmlFor="theme">{t("settings.theme")}</FieldLabel>
          <Select value={theme} onValueChange={(v) => {
            if (!v) return
            setTheme(v)
            const tauriTheme: Theme | null = v === "system" ? null : (v as Theme)
            getCurrentWindow().setTheme(tauriTheme)
          }}>
            <SelectTrigger id="theme">
              <span data-slot="select-value" className="flex flex-1 text-left">
                {themeLabels[theme ?? "system"]}
              </span>
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="system">{t("settings.themeSystem")}</SelectItem>
              <SelectItem value="light">{t("settings.themeLight")}</SelectItem>
              <SelectItem value="dark">{t("settings.themeDark")}</SelectItem>
            </SelectContent>
          </Select>
          <FieldDescription>{t("settings.themeDescription")}</FieldDescription>
        </Field>
      </FieldGroup>
      <div className="flex justify-end pt-4">
        <Button type="submit" size="lg">{t("settings.save")}</Button>
      </div>
    </form>
  )
}

function AdvancedSettings() {
  const { t } = useTranslation()
  const { openclawDir, setOpenclawDir } = useConfigStore()

  const handleSave = (e: React.FormEvent) => {
    e.preventDefault()
  }

  return (
    <form id="advanced-settings" onSubmit={handleSave} className="flex flex-1 flex-col">
      <FieldGroup className="flex-1">
        <Field>
          <FieldLabel htmlFor="openclaw-dir">{t("settings.openclawDir")}</FieldLabel>
          <Input
            id="openclaw-dir"
            placeholder="~/.openclaw"
            value={openclawDir}
            onChange={(e) => setOpenclawDir(e.target.value)}
          />
          <FieldDescription>{t("settings.openclawDirDescription")}</FieldDescription>
        </Field>
      </FieldGroup>
      <div className="flex justify-end pt-4">
        <Button type="submit" size="lg">{t("settings.save")}</Button>
      </div>
    </form>
  )
}

function ModelService() {
  const { t } = useTranslation()
  const { modelConfig, setModelConfig } = useConfigStore()
  const [testing, setTesting] = React.useState(false)
  const [testResult, setTestResult] = React.useState<"success" | "failed" | null>(null)

  const handleTest = async () => {
    setTesting(true)
    setTestResult(null)
    await new Promise((resolve) => setTimeout(resolve, 1500))
    setTestResult(modelConfig.baseUrl && modelConfig.apiKey ? "success" : "failed")
    setTesting(false)
  }

  return (
    <div className="flex flex-1 flex-col">
      <FieldGroup className="flex-1">
        <Field>
          <FieldLabel htmlFor="provider">{t("settings.provider")}</FieldLabel>
          <Select
            value={modelConfig.provider}
            onValueChange={(v) => {
              if (v === "openai" || v === "anthropic") {
                setModelConfig({ provider: v })
              }
            }}
          >
            <SelectTrigger id="provider">
              <span data-slot="select-value" className="flex flex-1 text-left">
                {modelConfig.provider === "anthropic" ? "Anthropic" : "OpenAI"}
              </span>
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="openai">OpenAI</SelectItem>
              <SelectItem value="anthropic">Anthropic</SelectItem>
            </SelectContent>
          </Select>
          <FieldDescription>{t("settings.providerDescription")}</FieldDescription>
        </Field>
        <Field>
          <FieldLabel htmlFor="api-url">{t("settings.apiUrl")}</FieldLabel>
          <Input
            id="api-url"
            placeholder="https://api.example.com"
            value={modelConfig.baseUrl}
            onChange={(e) => setModelConfig({ baseUrl: e.target.value })}
          />
          <FieldDescription>{t("settings.apiUrlDescription")}</FieldDescription>
        </Field>
        <Field>
          <FieldLabel htmlFor="api-key">{t("settings.apiKey")}</FieldLabel>
          <Input
            id="api-key"
            type="password"
            placeholder="sk-..."
            value={modelConfig.apiKey}
            onChange={(e) => setModelConfig({ apiKey: e.target.value })}
          />
          <FieldDescription>{t("settings.apiKeyDescription")}</FieldDescription>
        </Field>
        <Field>
          <FieldLabel htmlFor="model">{t("settings.model")}</FieldLabel>
          <Input
            id="model"
            placeholder="gpt-4o"
            value={modelConfig.model}
            onChange={(e) => setModelConfig({ model: e.target.value })}
          />
          <FieldDescription>{t("settings.modelDescription")}</FieldDescription>
        </Field>
      </FieldGroup>
      <div className="flex items-center gap-2 pt-4">
        {testResult === "success" && (
          <span className="text-base text-success">{t("settings.testSuccess")}</span>
        )}
        {testResult === "failed" && (
          <span className="text-base text-destructive">{t("settings.testFailed")}</span>
        )}
        <div className="ml-auto">
          <Button
            type="button"
            variant="outline"
            size="lg"
            onClick={handleTest}
            disabled={testing || !modelConfig.baseUrl}
          >
            {testing ? t("settings.testing") : t("settings.test")}
          </Button>
        </div>
      </div>
    </div>
  )
}

type SettingsTab = "general" | "modelService" | "advanced"

const contentMap: Record<SettingsTab, () => React.ReactNode> = {
  general: () => <GeneralSettings />,
  modelService: () => <ModelService />,
  advanced: () => <AdvancedSettings />,
}

export function SettingsDialog({
  open,
  onOpenChange,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
}) {
  const { t } = useTranslation()
  const [active, setActive] = React.useState<SettingsTab>("general")

  const navItems = [
    { key: "general" as const, name: t("settings.general"), icon: <SettingsIcon /> },
    { key: "advanced" as const, name: t("settings.openclawSettings"), icon: <SlidersHorizontalIcon /> },
    { key: "modelService" as const, name: t("settings.modelService"), icon: <ServerIcon /> },
  ]

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent className="overflow-hidden p-0 md:max-h-[420px] md:max-w-[700px] lg:max-w-[800px]">
        <DialogTitle className="sr-only">{t("settings.title")}</DialogTitle>
        <DialogDescription className="sr-only">
          {t("settings.title")}
        </DialogDescription>
        <SidebarProvider className="items-start">
          <Sidebar collapsible="none" className="hidden md:flex">
            <SidebarContent>
              <SidebarGroup>
                <SidebarGroupContent>
                  <SidebarMenu>
                    {navItems.map((item) => (
                      <SidebarMenuItem key={item.key}>
                        <SidebarMenuButton
                          isActive={item.key === active}
                          onClick={() => setActive(item.key)}
                        >
                          {item.icon}
                          <span>{item.name}</span>
                        </SidebarMenuButton>
                      </SidebarMenuItem>
                    ))}
                  </SidebarMenu>
                </SidebarGroupContent>
              </SidebarGroup>
            </SidebarContent>
          </Sidebar>
          <main className="flex h-[420px] flex-1 flex-col overflow-hidden">
            <header className="flex h-16 shrink-0 items-center gap-2">
              <div className="flex items-center gap-2 px-4">
                <Breadcrumb>
                  <BreadcrumbList>
                    <BreadcrumbItem className="hidden md:block">
                      <BreadcrumbLink href="#">{t("settings.title")}</BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator className="hidden md:block" />
                    <BreadcrumbItem>
                      <BreadcrumbPage>
                        {navItems.find((i) => i.key === active)?.name}
                      </BreadcrumbPage>
                    </BreadcrumbItem>
                  </BreadcrumbList>
                </Breadcrumb>
              </div>
            </header>
            <div className="flex flex-1 flex-col gap-4 overflow-y-auto p-4 pt-0">
              {contentMap[active]()}
            </div>
          </main>
        </SidebarProvider>
      </DialogContent>
    </Dialog>
  )
}
