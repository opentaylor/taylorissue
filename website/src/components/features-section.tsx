import {
  Download as InstallIcon,
  Wrench,
  Trash2,
  MessageSquare,
  Puzzle,
  Settings2,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
} from "@/components/ui/card";
import type { LucideIcon } from "lucide-react";

const FEATURES: { key: string; icon: LucideIcon }[] = [
  { key: "install", icon: InstallIcon },
  { key: "repair", icon: Wrench },
  { key: "uninstall", icon: Trash2 },
  { key: "chat", icon: MessageSquare },
  { key: "skills", icon: Puzzle },
  { key: "model", icon: Settings2 },
];

export function FeaturesSection() {
  const { t } = useTranslation();

  return (
    <section id="features" className="scroll-mt-16 py-20 sm:py-28">
      <div className="mx-auto max-w-6xl px-4 sm:px-6">
        <div className="mx-auto max-w-2xl text-center">
          <h2 className="text-2xl font-bold tracking-tight sm:text-3xl">
            {t("features.title")}
          </h2>
          <p className="mt-3 text-base text-muted-foreground">
            {t("features.subtitle")}
          </p>
        </div>

        <div className="mt-14 grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
          {FEATURES.map((f) => (
            <Card
              key={f.key}
              className="transition-shadow hover:shadow-lg hover:shadow-primary/5"
            >
              <CardHeader>
                <div className="mb-2 flex size-9 items-center justify-center rounded-lg bg-primary/10 text-primary">
                  <f.icon className="size-5" />
                </div>
                <CardTitle>{t(`features.${f.key}.title`)}</CardTitle>
                <CardDescription>
                  {t(`features.${f.key}.description`)}
                </CardDescription>
              </CardHeader>
            </Card>
          ))}
        </div>
      </div>
    </section>
  );
}
