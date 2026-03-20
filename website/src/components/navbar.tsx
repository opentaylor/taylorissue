import { Star, Sun, Moon, Monitor, Languages } from "lucide-react";
import { useTheme } from "next-themes";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { useGitHubStars } from "@/hooks/use-github-stars";

const GITHUB_URL = "https://github.com/opentaylor/taylorissue";

function ThemeToggle() {
  const { theme, setTheme } = useTheme();

  const next: Record<string, string> = {
    system: "light",
    light: "dark",
    dark: "system",
  };
  const Icon = theme === "dark" ? Moon : theme === "light" ? Sun : Monitor;

  return (
    <Button
      variant="ghost"
      size="icon"
      onClick={() => setTheme(next[theme ?? "system"])}
      aria-label="Toggle theme"
    >
      <Icon className="size-4" />
    </Button>
  );
}

function LanguageToggle() {
  const { i18n } = useTranslation();
  const isZh = i18n.language.startsWith("zh");

  return (
    <Button
      variant="ghost"
      size="icon"
      onClick={() => i18n.changeLanguage(isZh ? "en-US" : "zh-CN")}
      aria-label="Switch language"
    >
      <Languages className="size-4" />
    </Button>
  );
}

function StarBadge() {
  const stars = useGitHubStars();

  return (
    <a
      href={GITHUB_URL}
      target="_blank"
      rel="noopener noreferrer"
      className="inline-flex items-center gap-1.5 rounded-lg border px-2.5 py-1 text-base font-medium transition-colors hover:bg-muted"
    >
      <Star className="size-3.5 fill-amber-400 text-amber-400" />
      {stars !== null ? stars : "—"}
    </a>
  );
}

export function Navbar() {
  const { t } = useTranslation();

  return (
    <header className="sticky top-0 z-50 w-full border-b bg-background/80 backdrop-blur-md">
      <div className="relative mx-auto flex h-14 max-w-6xl items-center justify-end px-4 sm:px-6">
        <nav className="absolute inset-x-0 hidden items-center justify-center gap-6 text-base font-medium text-muted-foreground md:flex">
          <a
            href="#features"
            className="transition-colors hover:text-foreground"
          >
            {t("nav.features")}
          </a>
          <a
            href="#download"
            className="transition-colors hover:text-foreground"
          >
            {t("nav.download")}
          </a>
          <a
            href="#changelog"
            className="transition-colors hover:text-foreground"
          >
            {t("nav.changelog")}
          </a>
        </nav>

        <div className="relative z-10 flex items-center gap-1">
          <StarBadge />
          <LanguageToggle />
          <ThemeToggle />
        </div>
      </div>
    </header>
  );
}
