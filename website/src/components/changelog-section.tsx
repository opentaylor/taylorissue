import Markdown from "react-markdown";
import { useTranslation } from "react-i18next";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { useReleases } from "@/hooks/use-releases";
import type { GitHubRelease } from "@/types/github";

const PLACEHOLDER_BODY = "See the assets to download this version and install.";

interface ChangelogEntry {
  version: string;
  date: string;
  body: string;
}

const FALLBACK_V010_ITEM_KEYS = [
  "install",
  "repair",
  "uninstall",
  "chat",
  "skills",
  "model",
] as const;

function releaseToEntry(release: GitHubRelease): ChangelogEntry | null {
  if (!release.body || release.body.trim() === PLACEHOLDER_BODY) return null;

  return {
    version: release.name,
    date: release.published_at,
    body: release.body,
  };
}

function formatDate(dateStr: string) {
  const d = new Date(dateStr);
  return d.toLocaleDateString("en-US", {
    year: "numeric",
    month: "long",
    day: "numeric",
  });
}

const markdownComponents = {
  h3: ({ children, ...props }: React.ComponentProps<"h3">) => (
    <h4 className="mt-4 mb-2 text-base font-semibold" {...props}>
      {children}
    </h4>
  ),
  ul: ({ children, ...props }: React.ComponentProps<"ul">) => (
    <ul className="space-y-1.5" {...props}>
      {children}
    </ul>
  ),
  li: ({ children, ...props }: React.ComponentProps<"li">) => (
    <li
      className="text-base leading-relaxed text-muted-foreground"
      {...props}
    >
      <span className="mr-1.5 text-primary">&bull;</span>
      {children}
    </li>
  ),
  p: ({ children, ...props }: React.ComponentProps<"p">) => (
    <p className="mt-2 text-base leading-relaxed text-muted-foreground" {...props}>
      {children}
    </p>
  ),
  strong: ({ children, ...props }: React.ComponentProps<"strong">) => (
    <strong className="font-semibold text-foreground" {...props}>
      {children}
    </strong>
  ),
  a: ({ children, ...props }: React.ComponentProps<"a">) => (
    <a
      className="text-primary underline underline-offset-2 hover:text-primary/80"
      target="_blank"
      rel="noopener noreferrer"
      {...props}
    >
      {children}
    </a>
  ),
  code: ({ children, ...props }: React.ComponentProps<"code">) => (
    <code
      className="rounded bg-muted px-1.5 py-0.5 text-sm"
      {...props}
    >
      {children}
    </code>
  ),
};

function ChangelogEntryCard({ entry }: { entry: ChangelogEntry }) {
  return (
    <div className="flex gap-6">
      <div className="hidden w-28 shrink-0 pt-2 text-right text-base text-muted-foreground sm:block">
        {formatDate(entry.date)}
      </div>

      <div className="relative flex flex-col">
        <div className="absolute top-3.5 -left-[5px] size-2.5 rounded-full border-2 border-primary bg-background" />
        <div className="absolute top-6 -left-px h-full w-px bg-border" />
      </div>

      <div className="pb-12 pl-4">
        <div className="flex flex-wrap items-center gap-2">
          <Badge variant="outline" className="h-9 px-3 text-base">
            {entry.version}
          </Badge>
          <span className="text-base text-muted-foreground sm:hidden">
            {formatDate(entry.date)}
          </span>
        </div>

        <div className="mt-2 [&>*:first-child]:mt-2">
          <Markdown components={markdownComponents}>{entry.body}</Markdown>
        </div>
      </div>
    </div>
  );
}

export function ChangelogSection() {
  const { t } = useTranslation();
  const { releases } = useReleases();

  const entries: ChangelogEntry[] = [];
  const seenVersions = new Set<string>();

  for (const release of releases) {
    const entry = releaseToEntry(release);
    if (entry) {
      entries.push(entry);
      seenVersions.add(entry.version);
    }
  }

  if (!seenVersions.has("v0.1.0")) {
    const fallbackBody = [
      `### ${t("changelog.newFeatures")}`,
      "",
      ...FALLBACK_V010_ITEM_KEYS.map((k) => `- ${t(`changelog.items.${k}`)}`),
    ].join("\n");

    entries.unshift({
      version: "v0.1.0",
      date: "2026-03-21",
      body: fallbackBody,
    });
  }

  return (
    <section id="changelog" className="scroll-mt-16 py-20 sm:py-28">
      <div className="mx-auto max-w-3xl px-4 sm:px-6">
        <div className="mx-auto max-w-2xl text-center">
          <h2 className="text-2xl font-bold tracking-tight sm:text-3xl">
            {t("changelog.title")}
          </h2>
          <p className="mt-3 text-base text-muted-foreground">
            {t("changelog.subtitle")}
          </p>
        </div>

        <Separator className="my-10" />

        <div className="pl-2">
          {entries.map((entry) => (
            <ChangelogEntryCard key={entry.version} entry={entry} />
          ))}
        </div>

      </div>
    </section>
  );
}
