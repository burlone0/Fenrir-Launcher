import { listen } from "@tauri-apps/api/event";
import type { ScanProgress, ScanDonePayload, Game } from "./types";

export const onScanProgress = (cb: (p: ScanProgress) => void) =>
  listen<ScanProgress>("scan:progress", (e) => cb(e.payload));

export const onScanDone = (cb: (p: ScanDonePayload) => void) =>
  listen<ScanDonePayload>("scan:done", (e) => cb(e.payload));

export const onConfigureStep = (cb: (step: string) => void) =>
  listen<{ step: string }>("configure:step", (e) => cb(e.payload.step));

export const onConfigureDone = (cb: (game: Game) => void) =>
  listen<{ game: Game }>("configure:done", (e) => cb(e.payload.game));

export const onLaunchStarted = (cb: (gameId: string) => void) =>
  listen<{ game_id: string }>("launch:started", (e) => cb(e.payload.game_id));

export const onLaunchEnded = (
  cb: (p: { game_id: string; exit_code: number; play_time_secs: number }) => void
) =>
  listen<{ game_id: string; exit_code: number; play_time_secs: number }>(
    "launch:ended",
    (e) => cb(e.payload)
  );

export const onDownloadProgress = (
  cb: (p: { bytes_received: number; total_bytes: number }) => void
) =>
  listen<{ bytes_received: number; total_bytes: number }>(
    "download:progress",
    (e) => cb(e.payload)
  );

export const onDownloadDone = (cb: (version: string) => void) =>
  listen<{ version: string }>("download:done", (e) => cb(e.payload.version));
