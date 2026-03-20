import { Download } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { useReleases } from "@/hooks/use-releases";
import { AppleIcon, WindowsIcon } from "@/components/platform-icons";

const RELEASES_URL = "https://github.com/opentaylor/taylorissue/releases";

function formatSize(bytes: number) {
  return (bytes / 1024 / 1024).toFixed(1) + " MB";
}

export function DownloadSection() {
  const { t } = useTranslation();
  const { latestRelease, getAllDownloads, loading } = useReleases();
  const downloads = getAllDownloads();

  const version = latestRelease?.name ?? "v0.0.3";

  const macDownloads = downloads.filter((d) => d.platform.startsWith("mac"));
  const winDownloads = downloads.filter((d) => d.platform.startsWith("win"));

  return (
    <section
      id="download"
      className="scroll-mt-16 bg-muted/40 py-20 sm:py-28"
    >
      <div className="mx-auto max-w-4xl px-4 sm:px-6">
        <div className="mx-auto max-w-2xl text-center">
          <h2 className="text-2xl font-bold tracking-tight sm:text-3xl">
            {t("download.title")}
          </h2>
          <p className="mt-3 text-base text-muted-foreground">
            {t("download.subtitle", { version })}
          </p>
        </div>

        {loading ? (
          <div className="mt-12 text-center text-base text-muted-foreground">
            {t("download.loading")}
          </div>
        ) : (
          <div className="mt-12 grid gap-6 sm:grid-cols-2">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <AppleIcon className="size-5" />
                  {t("download.macOS")}
                </CardTitle>
              </CardHeader>
              <CardContent className="flex flex-col gap-3">
                {macDownloads.length > 0 ? (
                  macDownloads.map((d) => (
                    <a
                      key={d.platform}
                      href={d.url}
                      className="flex items-center gap-3 rounded-lg border px-4 py-3 transition-colors hover:bg-muted"
                    >
                      <div className="min-w-0 flex-1">
                        <p className="text-base font-medium">
                          {t(`platform.${d.platform}`)}
                        </p>
                        <p className="truncate text-base text-muted-foreground">
                          {d.fileName} &middot; {formatSize(d.size)}
                        </p>
                      </div>
                      <Download className="size-4 shrink-0 text-muted-foreground" />
                    </a>
                  ))
                ) : (
                  <p className="text-base text-muted-foreground">
                    {t("download.noMac")}
                  </p>
                )}
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <WindowsIcon className="size-5" />
                  {t("download.windows")}
                </CardTitle>
              </CardHeader>
              <CardContent className="flex flex-col gap-3">
                {winDownloads.length > 0 ? (
                  winDownloads.map((d) => (
                    <a
                      key={d.platform}
                      href={d.url}
                      className="flex items-center gap-3 rounded-lg border px-4 py-3 transition-colors hover:bg-muted"
                    >
                      <div className="min-w-0 flex-1">
                        <p className="text-base font-medium">
                          {t(`platform.${d.platform}`)}
                        </p>
                        <p className="truncate text-base text-muted-foreground">
                          {d.fileName} &middot; {formatSize(d.size)}
                        </p>
                      </div>
                      <Download className="size-4 shrink-0 text-muted-foreground" />
                    </a>
                  ))
                ) : (
                  <p className="text-base text-muted-foreground">
                    {t("download.noWin")}
                  </p>
                )}
              </CardContent>
            </Card>
          </div>
        )}

        <div className="mt-8 text-center">
          <a href={RELEASES_URL} target="_blank" rel="noopener noreferrer">
            <Button variant="outline" size="lg">
              {t("download.viewAll")}
            </Button>
          </a>
        </div>
      </div>
    </section>
  );
}
