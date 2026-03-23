import { useTranslation } from "react-i18next";
import { Separator } from "@/components/ui/separator";
import logoImg from "@images/logo.png";

const GITHUB_URL = "https://github.com/opentaylor/taylorissue";
const RELEASES_URL = "https://github.com/opentaylor/taylorissue/releases";
const ISSUES_URL = "https://github.com/opentaylor/taylorissue/issues";

export function Footer() {
  const { t } = useTranslation();

  return (
    <footer className="border-t bg-muted/30">
      <div className="mx-auto max-w-6xl px-4 py-8 sm:px-6">
        <div className="grid gap-8 sm:grid-cols-3">
          <div>
            <div className="flex items-center gap-2.5">
              <img
                src={logoImg}
                alt="TaylorIssue"
                className="size-7 rounded-lg"
              />
              <span className="text-base font-semibold">TaylorIssue</span>
            </div>
            <p className="mt-3 text-base leading-relaxed text-muted-foreground">
              {t("footer.description")}
            </p>
          </div>

          <div>
            <h4 className="text-base font-semibold">{t("footer.links")}</h4>
            <ul className="mt-3 space-y-2 text-base text-muted-foreground">
              <li>
                <a
                  href={GITHUB_URL}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="transition-colors hover:text-foreground"
                >
                  GitHub
                </a>
              </li>
              <li>
                <a
                  href={RELEASES_URL}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="transition-colors hover:text-foreground"
                >
                  {t("footer.releases")}
                </a>
              </li>
              <li>
                <a
                  href={ISSUES_URL}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="transition-colors hover:text-foreground"
                >
                  {t("footer.reportIssue")}
                </a>
              </li>
            </ul>
          </div>

          <div>
            <h4 className="text-base font-semibold">
              {t("footer.acknowledgements")}
            </h4>
            <ul className="mt-3 space-y-2 text-base text-muted-foreground">
              <li>
                Zhang Zhi &mdash; {t("footer.maintainer")}
              </li>
              <li>
                Liu Yan &mdash; {t("footer.maintainer")}
              </li>
              <li>
                Chen Gong &mdash; {t("footer.sponsor")}
              </li>
            </ul>
          </div>
        </div>

        <Separator className="my-8" />

        <p className="text-center text-base text-muted-foreground">
          &copy; {new Date().getFullYear()} OpenTaylor.{" "}
          <a
            href={`${GITHUB_URL}/blob/main/LICENSE`}
            target="_blank"
            rel="noopener noreferrer"
            className="underline underline-offset-2 hover:text-foreground"
          >
            {t("footer.license")}
          </a>
        </p>
      </div>
    </footer>
  );
}
