import { useState } from "react";
import { ChevronDown, Download } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { useReleases } from "@/hooks/use-releases";
import { AppleIcon, WindowsIcon } from "@/components/platform-icons";
import type { GitHubRelease, DownloadAsset } from "@/types/github";

function formatSize(bytes: number) {
  return (bytes / 1024 / 1024).toFixed(1) + " MB";
}

function ReleaseCard({
  release,
  downloads,
  t,
}: {
  release: GitHubRelease;
  downloads: DownloadAsset[];
  t: (key: string, opts?: Record<string, string>) => string;
}) {
  const macDownloads = downloads.filter((d) => d.platform.startsWith("mac"));
  const winDownloads = downloads.filter((d) => d.platform.startsWith("win"));

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-center text-lg">
          {release.name}
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="flex flex-col items-center gap-6 lg:flex-row lg:justify-center lg:gap-12">
          <div className="flex flex-col items-center gap-3">
            <div className="flex items-center gap-2 text-base font-medium">
              <AppleIcon className="size-5" />
              {t("download.macOS")}
            </div>
            {macDownloads.length > 0 ? (
              <div className="flex items-center justify-center gap-3">
                {macDownloads.map((d) => (
                  <a
                    key={d.platform}
                    href={d.url}
                    className="flex w-44 items-center justify-center gap-2 rounded-lg border px-4 py-2.5 transition-colors hover:bg-muted"
                  >
                    <span className="text-base">
                      {t(`arch.${d.platform}`)}
                    </span>
                    <span className="text-sm text-muted-foreground">
                      {formatSize(d.size)}
                    </span>
                    <Download className="size-3.5 shrink-0 text-muted-foreground" />
                  </a>
                ))}
              </div>
            ) : (
              <p className="text-base text-muted-foreground">
                {t("download.noMac")}
              </p>
            )}
          </div>

          <div className="flex flex-col items-center gap-3">
            <div className="flex items-center gap-2 text-base font-medium">
              <WindowsIcon className="size-5" />
              {t("download.windows")}
            </div>
            {winDownloads.length > 0 ? (
              <div className="flex items-center justify-center gap-3">
                {winDownloads.map((d) => (
                  <a
                    key={d.platform}
                    href={d.url}
                    className="flex w-44 items-center justify-center gap-2 rounded-lg border px-4 py-2.5 transition-colors hover:bg-muted"
                  >
                    <span className="text-base">
                      {t(`arch.${d.platform}`)}
                    </span>
                    <span className="text-sm text-muted-foreground">
                      {formatSize(d.size)}
                    </span>
                    <Download className="size-3.5 shrink-0 text-muted-foreground" />
                  </a>
                ))}
              </div>
            ) : (
              <p className="text-base text-muted-foreground">
                {t("download.noWin")}
              </p>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

export function DownloadSection() {
  const { t } = useTranslation();
  const { releases, getDownloadsForRelease, loading } = useReleases();
  const [visibleCount, setVisibleCount] = useState(2);

  const visibleReleases = releases.slice(0, visibleCount);
  const hasMore = visibleCount < releases.length;

  return (
    <section
      id="download"
      className="scroll-mt-16 py-12 sm:py-16"
    >
      <div className="mx-auto max-w-4xl px-4 sm:px-6">
        <div className="mx-auto max-w-2xl text-center">
          <h2 className="text-2xl font-bold tracking-tight sm:text-3xl">
            {t("download.title")}
          </h2>
          <p className="mt-3 text-base text-muted-foreground">
            {t("download.subtitle", {
              version: releases[0]?.name ?? "v0.1.0",
            })}
          </p>
        </div>

        {loading ? (
          <div className="mt-8 text-center text-base text-muted-foreground">
            {t("download.loading")}
          </div>
        ) : (
          <div className="mt-8 flex flex-col gap-6">
            {visibleReleases.map((release) => (
              <ReleaseCard
                key={release.tag_name}
                release={release}
                downloads={getDownloadsForRelease(release)}
                t={t}
              />
            ))}
          </div>
        )}

        {hasMore && (
          <div className="mt-8 text-center">
            <Button
              variant="outline"
              size="lg"
              onClick={() => setVisibleCount((c) => c + 2)}
            >
              <ChevronDown className="size-4" />
              {t("download.more")}
            </Button>
          </div>
        )}
      </div>
    </section>
  );
}
