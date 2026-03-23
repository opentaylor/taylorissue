import { Download } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { usePlatform } from "@/hooks/use-platform";
import { useReleases } from "@/hooks/use-releases";
import type { Platform } from "@/types/github";
import logoImg from "@images/logo.png";
import overviewImg from "@images/overview.png";

const ALL_PLATFORMS: Platform[] = ["mac-arm", "mac-intel", "win-x64", "win-arm"];

export function HeroSection() {
  const { t } = useTranslation();
  const { platform: detectedPlatform } = usePlatform();
  const { getAllPlatformDownloads, loading } = useReleases();

  const platformDownloads = getAllPlatformDownloads();

  return (
    <section className="relative overflow-hidden pt-10 pb-12 sm:pt-14 sm:pb-16">
      <div className="pointer-events-none absolute inset-0 -z-10 bg-[radial-gradient(ellipse_60%_50%_at_50%_-10%,var(--color-primary)/0.08,transparent)]" />

      <div className="mx-auto flex max-w-4xl flex-col items-center px-4 text-center sm:px-6">
        <img
          src={logoImg}
          alt="TaylorIssue"
          className="mx-auto w-full max-w-[200px]"
          loading="eager"
        />

        <h1 className="mt-5 text-3xl font-bold tracking-tight sm:text-4xl">
          {t("hero.title")}
        </h1>

        <p className="mt-4 text-base font-medium text-muted-foreground">
          {t("hero.subtitle")}
          <br className="hidden sm:inline" /> {t("hero.subtitleLine2")}
        </p>

        <p className="mt-3 max-w-2xl text-base text-muted-foreground/80">
          {t("hero.description")}
        </p>

        <div className="mt-5 flex flex-wrap items-center justify-center gap-3">
          {ALL_PLATFORMS.map((p, i) => {
            const asset = platformDownloads[i];
            const isDetected = p === detectedPlatform;
            const label = t(`platform.${p}`);

            if (asset) {
              return (
                <a key={p} href={asset.url}>
                  <Button
                    size="lg"
                    variant={isDetected ? "default" : "outline"}
                    disabled={loading}
                  >
                    <Download className="size-4" />
                    {label}
                  </Button>
                </a>
              );
            }

            return (
              <a key={p} href="#download">
                <Button
                  size="lg"
                  variant={isDetected ? "default" : "outline"}
                  disabled={loading}
                >
                  <Download className="size-4" />
                  {loading ? t("hero.loading") : label}
                </Button>
              </a>
            );
          })}
        </div>
      </div>

      <div className="mx-auto mt-8 max-w-3xl px-4 sm:px-6">
        <div className="overflow-hidden rounded-xl border shadow-2xl shadow-primary/5">
          <img
            src={overviewImg}
            alt="TaylorIssue overview"
            className="w-full"
            loading="eager"
          />
        </div>
      </div>
    </section>
  );
}
