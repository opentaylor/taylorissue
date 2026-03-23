import { useState, useEffect } from "react";
import type { PlatformInfo } from "@/types/github";

interface NavigatorUAData {
  getHighEntropyValues(hints: string[]): Promise<{ architecture?: string }>;
}

async function detectArchitecture(): Promise<"arm" | "x86" | null> {
  try {
    const uaData = (navigator as unknown as { userAgentData?: NavigatorUAData })
      .userAgentData;
    if (uaData) {
      const hints = await uaData.getHighEntropyValues(["architecture"]);
      if (hints.architecture === "arm") return "arm";
      if (hints.architecture === "x86") return "x86";
    }
  } catch {
    // Client Hints unavailable or rejected
  }

  try {
    const canvas = document.createElement("canvas");
    const gl = canvas.getContext("webgl2") || canvas.getContext("webgl");
    if (gl) {
      const ext = gl.getExtension("WEBGL_debug_renderer_info");
      if (ext) {
        const renderer = gl.getParameter(
          ext.UNMASKED_RENDERER_WEBGL
        ) as string;
        if (/Apple M\d|Apple GPU/i.test(renderer)) return "arm";
        if (/Intel/i.test(renderer)) return "x86";
      }
    }
  } catch {
    // WebGL unavailable
  }

  return null;
}

function getInitialPlatform(): { os: "mac" | "win" | null; info: PlatformInfo } {
  const ua = navigator.userAgent.toLowerCase();

  if (ua.includes("mac")) {
    return {
      os: "mac",
      info: {
        platform: "mac-arm",
        label: "macOS (Apple Silicon)",
        icon: "apple" as const,
      },
    };
  }

  if (ua.includes("win")) {
    const isArm = ua.includes("arm") || ua.includes("aarch64");
    return {
      os: "win",
      info: isArm
        ? { platform: "win-arm", label: "Windows (ARM)", icon: "windows" as const }
        : { platform: "win-x64", label: "Windows (x64)", icon: "windows" as const },
    };
  }

  return {
    os: null,
    info: { platform: "unknown", label: "your platform", icon: "unknown" as const },
  };
}

export function usePlatform(): PlatformInfo {
  const [info, setInfo] = useState<PlatformInfo>(() => getInitialPlatform().info);

  useEffect(() => {
    const { os } = getInitialPlatform();
    if (!os) return;

    detectArchitecture().then((arch) => {
      if (os === "mac") {
        setInfo(
          arch === "x86"
            ? { platform: "mac-intel", label: "macOS (Intel)", icon: "apple" as const }
            : { platform: "mac-arm", label: "macOS (Apple Silicon)", icon: "apple" as const }
        );
      } else if (os === "win") {
        setInfo(
          arch === "arm"
            ? { platform: "win-arm", label: "Windows (ARM)", icon: "windows" as const }
            : { platform: "win-x64", label: "Windows (x64)", icon: "windows" as const }
        );
      }
    });
  }, []);

  return info;
}
