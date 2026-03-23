import { useState, useEffect, useRef, useCallback } from "react"
import { useTranslation } from "react-i18next"
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import {
  Field,
  FieldDescription,
  FieldGroup,
  FieldLabel,
} from "@/components/ui/field"
import { Input } from "@/components/ui/input"
import { Button } from "@/components/ui/button"
import { FreeApiHelpDialog } from "@/components/free-api-help-dialog"
import { useConfigStore } from "@/stores/config-store"

const TOTAL_FRAMES = 145
const FPS = 24
const framePaths = Array.from(
  { length: TOTAL_FRAMES },
  (_, i) => `/hero-frames/frame_${String(i + 1).padStart(4, "0")}.png`,
)

function HeroAnimation() {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const imagesRef = useRef<HTMLImageElement[]>([])
  const frameRef = useRef(0)
  const rafRef = useRef<number>(0)
  const lastTimeRef = useRef(0)

  const draw = useCallback((timestamp: number) => {
    if (!lastTimeRef.current) lastTimeRef.current = timestamp
    const elapsed = timestamp - lastTimeRef.current

    if (elapsed >= 1000 / FPS) {
      lastTimeRef.current = timestamp - (elapsed % (1000 / FPS))
      const canvas = canvasRef.current
      const ctx = canvas?.getContext("2d")
      const img = imagesRef.current[frameRef.current]
      if (canvas && ctx && img?.complete && img.naturalWidth > 0) {
        canvas.width = img.naturalWidth
        canvas.height = img.naturalHeight
        ctx.clearRect(0, 0, canvas.width, canvas.height)
        ctx.drawImage(img, 0, 0)
      }
      frameRef.current = (frameRef.current + 1) % TOTAL_FRAMES
    }
    rafRef.current = requestAnimationFrame(draw)
  }, [])

  useEffect(() => {
    const imgs = framePaths.map((src) => {
      const img = new Image()
      img.src = src
      return img
    })
    imagesRef.current = imgs
    rafRef.current = requestAnimationFrame(draw)
    return () => cancelAnimationFrame(rafRef.current)
  }, [draw])

  return (
    <canvas
      ref={canvasRef}
      className="h-full w-full object-contain"
    />
  )
}

export default function DashboardHome() {
  const { t } = useTranslation()
  const { modelConfig, setModelConfig } = useConfigStore()
  const [showCredentialsDialog, setShowCredentialsDialog] = useState(false)

  const maintainers = [
    t("page.dashboard.meta.maintainers.items.zhang"),
  ]
  const acknowledgements = [t("page.dashboard.meta.acknowledgements.items.chen")]

  return (
    <div className="flex flex-col gap-4 px-4 py-6 lg:px-6">
      <Card>
        <CardContent className="grid gap-4 lg:grid-cols-[minmax(0,1.2fr)_minmax(280px,0.8fr)]">
          <section className="flex flex-col justify-center gap-2">
            <p className="text-lg font-semibold">{t("app.name")}</p>
            <p className="text-base">{t("page.dashboard.hero.title")}</p>
            <p className="text-base text-muted-foreground">
              {t("page.dashboard.hero.description")}
            </p>
          </section>

          <section>
            <div className="mx-auto flex max-w-xs items-center justify-center overflow-hidden rounded-2xl">
              <HeroAnimation />
            </div>
          </section>
        </CardContent>
      </Card>

      <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_360px]">
        <Card>
          <CardHeader>
            <CardTitle>{t("page.dashboard.config.title")}</CardTitle>
            <CardDescription>{t("page.dashboard.config.description")}</CardDescription>
          </CardHeader>

          <CardContent>
            <FieldGroup>
              <Field>
                <FieldLabel htmlFor="dashboard-api-base">
                  {t("page.dashboard.config.apiBaseLabel")}
                </FieldLabel>
                <Input
                  id="dashboard-api-base"
                  placeholder="https://api.example.com"
                  value={modelConfig.baseUrl}
                  onChange={(e) => setModelConfig({ baseUrl: e.target.value })}
                />
                <FieldDescription>
                  {t("page.dashboard.config.apiBaseDescription")}
                </FieldDescription>
              </Field>

              <Field>
                <FieldLabel htmlFor="dashboard-api-key">
                  {t("page.dashboard.config.apiKeyLabel")}
                </FieldLabel>
                <Input
                  id="dashboard-api-key"
                  type="password"
                  placeholder="sk-..."
                  value={modelConfig.apiKey}
                  onChange={(e) => setModelConfig({ apiKey: e.target.value })}
                />
                <FieldDescription>
                  {t("page.dashboard.config.apiKeyDescription")}
                </FieldDescription>
              </Field>

              <Field>
                <FieldLabel htmlFor="dashboard-model">
                  {t("page.dashboard.config.modelLabel")}
                </FieldLabel>
                <Input
                  id="dashboard-model"
                  placeholder="gpt-4o"
                  value={modelConfig.model}
                  onChange={(e) => setModelConfig({ model: e.target.value })}
                />
                <FieldDescription>
                  {t("page.dashboard.config.modelDescription")}
                </FieldDescription>
              </Field>
            </FieldGroup>
          </CardContent>

          <CardFooter>
            <Button
              type="button"
              variant="link"
              className="active:scale-100 px-0 no-underline hover:no-underline"
              onClick={() => setShowCredentialsDialog(true)}
            >
              {t("page.dashboard.links.credentials")}
            </Button>
          </CardFooter>
        </Card>

        <Card>
          <CardContent className="space-y-5">
            <section className="space-y-1">
              <h3 className="font-medium">{t("page.dashboard.meta.licenseLabel")}</h3>
              <p className="text-base text-muted-foreground">
                {t("page.dashboard.meta.licenseValue")}
              </p>
            </section>

            <section className="space-y-2">
              <h3 className="font-medium">
                {t("page.dashboard.meta.maintainers.label")}
              </h3>
              <div className="space-y-2 text-base text-muted-foreground">
                {maintainers.map((maintainer) => (
                  <p key={maintainer}>{maintainer}</p>
                ))}
              </div>
            </section>

            <section className="space-y-2">
              <h3 className="font-medium">
                {t("page.dashboard.meta.acknowledgements.label")}
              </h3>
              <div className="space-y-2 text-base text-muted-foreground">
                {acknowledgements.map((item) => (
                  <p key={item}>{item}</p>
                ))}
              </div>
            </section>

            <section className="space-y-1">
              <h3 className="font-medium">{t("page.dashboard.meta.releaseDateLabel")}</h3>
              <p className="text-base text-muted-foreground">
                {t("page.dashboard.meta.releaseDateValue")}
              </p>
            </section>
          </CardContent>
        </Card>
      </div>

      <FreeApiHelpDialog
        open={showCredentialsDialog}
        onOpenChange={setShowCredentialsDialog}
      />
    </div>
  )
}
