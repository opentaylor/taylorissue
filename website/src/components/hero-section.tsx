import { Download, ExternalLink } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { usePlatform } from "@/hooks/use-platform";
import { useReleases } from "@/hooks/use-releases";
import { HeroAnimation } from "@/components/hero-animation";
import overviewImg from "@images/overview.png";

const GITHUB_URL = "https://github.com/opentaylor/taylorissue";

export function HeroSection() {
  const { t } = useTranslation();
  const { platform } = usePlatform();
  const { getDownloadUrl, loading } = useReleases();

  const download = getDownloadUrl(platform);
  const platformLabel = t(`platform.${platform}`);

  return (
    <section className="relative overflow-hidden pt-16 pb-16 sm:pt-24 sm:pb-24">
      <div className="pointer-events-none absolute inset-0 -z-10 bg-[radial-gradient(ellipse_60%_50%_at_50%_-10%,var(--color-primary)/0.08,transparent)]" />

      <div className="mx-auto flex max-w-4xl flex-col items-center px-4 text-center sm:px-6">
        <HeroAnimation />

        <h1 className="mt-8 text-3xl font-bold tracking-tight sm:text-4xl">
          {t("hero.title")}
        </h1>

        <p className="mt-4 text-base font-medium text-muted-foreground">
          {t("hero.subtitle")}
          <br className="hidden sm:inline" /> {t("hero.subtitleLine2")}
        </p>

        <p className="mt-3 max-w-2xl text-base text-muted-foreground/80">
          {t("hero.description")}
        </p>

        <div className="mt-8 flex flex-col items-center gap-3 sm:flex-row sm:gap-4">
          {download ? (
            <a href={download.url}>
              <Button size="lg">
                <Download className="size-4" />
                {t("hero.downloadFor", { platform: platformLabel })}
              </Button>
            </a>
          ) : (
            <a href="#download">
              <Button size="lg" disabled={loading}>
                <Download className="size-4" />
                {loading ? t("hero.loading") : t("hero.download")}
              </Button>
            </a>
          )}

          <a href={GITHUB_URL} target="_blank" rel="noopener noreferrer">
            <Button variant="outline" size="lg">
              <ExternalLink className="size-4" />
              {t("hero.viewOnGitHub")}
            </Button>
          </a>
        </div>
      </div>

      <div className="mx-auto mt-16 max-w-3xl px-4 sm:px-6">
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
