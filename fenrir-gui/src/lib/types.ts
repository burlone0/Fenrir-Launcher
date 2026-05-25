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
  | "OnlineFixMelonLoader"
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

export interface FenrirConfig {
  general: {
    library_db: string;
    prefix_dir: string;
    runtime_dir: string;
  };
  scan: {
    game_dirs: string[];
    auto_scan: boolean;
  };
  privacy: {
    fetch_metadata: boolean;
    fetch_covers: boolean;
    metadata_source: string;
  };
  defaults: {
    runtime: string;
    enable_dxvk: boolean;
    enable_vkd3d: boolean;
    esync: boolean;
    fsync: boolean;
  };
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
