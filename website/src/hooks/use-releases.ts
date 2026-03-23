import { useState, useEffect } from "react";
import type { GitHubRelease, Platform, DownloadAsset } from "@/types/github";

const API_URL =
  "https://api.github.com/repos/opentaylor/taylorissue/releases";

const ASSET_PATTERNS: Record<Platform, RegExp> = {
  "mac-arm": /aarch64\.dmg$/,
  "mac-intel": /x64\.dmg$/,
  "win-x64": /x64-setup\.exe$/,
  "win-arm": /arm64-setup\.exe$/,
  unknown: /\.dmg$/,
};

const PLATFORM_LABELS: Record<Platform, string> = {
  "mac-arm": "macOS (Apple)",
  "mac-intel": "macOS (Intel)",
  "win-x64": "Windows (x64)",
  "win-arm": "Windows (ARM)",
  unknown: "Unknown",
};

function findAssetForPlatform(
  release: GitHubRelease,
  platform: Platform
): DownloadAsset | null {
  const pattern = ASSET_PATTERNS[platform];
  const asset = release.assets.find((a) => pattern.test(a.name));
  if (!asset) return null;
  return {
    platform,
    label: PLATFORM_LABELS[platform],
    fileName: asset.name,
    url: asset.browser_download_url,
    size: asset.size,
  };
}

export function useReleases() {
  const [releases, setReleases] = useState<GitHubRelease[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function fetchReleases() {
      try {
        const res = await fetch(API_URL);
        if (!res.ok) throw new Error(`GitHub API returned ${res.status}`);
        const data: GitHubRelease[] = await res.json();
        if (!cancelled) {
          setReleases(
            data.filter((r) => !r.draft && !r.prerelease)
          );
        }
      } catch (err) {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : "Failed to fetch releases");
        }
      } finally {
        if (!cancelled) setLoading(false);
      }
    }

    fetchReleases();
    return () => {
      cancelled = true;
    };
  }, []);

  const latestRelease = releases[0] ?? null;

  function getDownloadUrl(platform: Platform): DownloadAsset | null {
    if (!latestRelease) return null;
    return findAssetForPlatform(latestRelease, platform);
  }

  function getAllDownloads(): DownloadAsset[] {
    if (!latestRelease) return [];
    return getDownloadsForRelease(latestRelease);
  }

  function getDownloadsForRelease(release: GitHubRelease): DownloadAsset[] {
    const platforms: Platform[] = ["mac-arm", "mac-intel", "win-x64", "win-arm"];
    return platforms
      .map((p) => findAssetForPlatform(release, p))
      .filter((a): a is DownloadAsset => a !== null);
  }

  function getAllPlatformDownloads(): (DownloadAsset | null)[] {
    const platforms: Platform[] = ["mac-arm", "mac-intel", "win-x64", "win-arm"];
    if (!latestRelease) return platforms.map(() => null);
    return platforms.map((p) => findAssetForPlatform(latestRelease, p));
  }

  return {
    releases,
    latestRelease,
    loading,
    error,
    getDownloadUrl,
    getAllDownloads,
    getDownloadsForRelease,
    getAllPlatformDownloads,
  };
}
