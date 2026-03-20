import { useMemo } from "react";
import type { PlatformInfo } from "@/types/github";

export function usePlatform(): PlatformInfo {
  return useMemo(() => {
    const ua = navigator.userAgent.toLowerCase();

    if (ua.includes("mac")) {
      // Apple Silicon detection: Chrome 92+ and Safari 17+ expose architecture.
      // On older browsers, default to ARM since most modern Macs are Apple Silicon.
      const isIntel =
        ua.includes("intel") ||
        (ua.includes("x86_64") && !ua.includes("arm"));

      return isIntel
        ? { platform: "mac-intel", label: "macOS (Intel)", icon: "apple" as const }
        : { platform: "mac-arm", label: "macOS (Apple Silicon)", icon: "apple" as const };
    }

    if (ua.includes("win")) {
      const isArm = ua.includes("arm") || ua.includes("aarch64");
      return isArm
        ? { platform: "win-arm", label: "Windows (ARM)", icon: "windows" as const }
        : { platform: "win-x64", label: "Windows (x64)", icon: "windows" as const };
    }

    return { platform: "unknown", label: "your platform", icon: "unknown" as const };
  }, []);
}
