import { useTheme } from "next-themes";
import { Navbar } from "@/components/navbar";
import { HeroSection } from "@/components/hero-section";
import { FeaturesSection } from "@/components/features-section";
import { DownloadSection } from "@/components/download-section";
import { ChangelogSection } from "@/components/changelog-section";
import { Footer } from "@/components/footer";
import { Particles } from "@/components/ui/particles";

export default function App() {
  const { resolvedTheme } = useTheme();
  const particleColor = resolvedTheme === "dark" ? "#ffffff" : "#000000";

  return (
    <div className="relative min-h-screen">
      <Particles
        className="fixed inset-0 -z-10"
        quantity={80}
        staticity={30}
        ease={60}
        size={0.5}
        color={particleColor}
      />
      <Navbar />
      <main>
        <HeroSection />
        <FeaturesSection />
        <DownloadSection />
        <ChangelogSection />
      </main>
      <Footer />
    </div>
  );
}
