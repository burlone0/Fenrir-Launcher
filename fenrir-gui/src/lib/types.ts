export interface Game {
  id: string;
  title: string;
  executable: string;
  install_dir: string;
  store_origin: StoreOrigin;
  crack_type: CrackType | null;
  prefix_path: string;
  runtime_id: string | null;
  status: GameStatus;
  play_time: number;
  last_played: string | null;
  added_at: string;
  user_overrides: unknown | null;
}

export type StoreOrigin = "Steam" | "GOG" | "Epic" | "Unknown";

export type CrackType =
  | "OnlineFix"
  | "DODI"
  | "FitGirl"
  | "Scene"
  | "GOGRip"
  | "SteamRip"
  | "SmokeAPI"
  | "Unsteam"
  | "Unknown";

export type GameStatus =
  | "Detected"
  | "Configured"
  | "Ready"
  | "Broken"
  | "NeedsConfirmation";

export interface Runtime {
  id: string;
  runtime_type: RuntimeType;
  version: string;
  path: string;
  source: RuntimeSource;
  is_default: boolean;
}

export type RuntimeType = "Wine" | "Proton" | "ProtonGE" | "WineGE";
export type RuntimeSource = "System" | "Steam" | "Downloaded";

export interface ClassifiedGame {
  path: string;
  title: string;
  store_origin: StoreOrigin;
  crack_type: CrackType | null;
  confidence: number;
  signature_name: string;
}

export interface ScanProgress {
  current: number;
  total: number;
  path: string;
}

export interface ScanDonePayload {
  high_confidence: ClassifiedGame[];
  needs_confirmation: ClassifiedGame[];
  total: number;
}

export interface GitHubRelease {
  tag_name: string;
  assets: GitHubAsset[];
}

export interface GitHubAsset {
  name: string;
  browser_download_url: string;
  size: number;
}
