export interface GitHubAsset {
  name: string;
  browser_download_url: string;
  size: number;
  download_count: number;
}

export interface GitHubRelease {
  tag_name: string;
  name: string;
  published_at: string;
  body: string;
  html_url: string;
  draft: boolean;
  prerelease: boolean;
  assets: GitHubAsset[];
}

export type Platform = "mac-arm" | "mac-intel" | "win-x64" | "win-arm" | "unknown";

export interface PlatformInfo {
  platform: Platform;
  label: string;
  icon: "apple" | "windows" | "unknown";
}

export interface DownloadAsset {
  platform: Platform;
  label: string;
  fileName: string;
  url: string;
  size: number;
}
