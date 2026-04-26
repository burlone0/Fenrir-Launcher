import { invoke } from "@tauri-apps/api/core";
import type { Game, Runtime, GitHubRelease } from "./types";

// --- Games ---
export const listGames = () => invoke<Game[]>("list_games");
export const getGame = (id: string) => invoke<Game>("get_game", { id });
export const addGame = (path: string) => invoke<Game>("add_game", { path });
export const confirmGame = (query: string) => invoke<Game>("confirm_game", { query });
export const configureGame = (id: string, clean: boolean) =>
  invoke<void>("configure_game", { id, clean });
export const launchGame = (id: string) => invoke<void>("launch_game", { id });
export const deleteGame = (id: string) => invoke<void>("delete_game", { id });

// --- Scan ---
export const scanDirectory = (path?: string) =>
  invoke<void>("scan_directory", { path: path ?? null });

// --- Runtimes ---
export const listRuntimes = () => invoke<Runtime[]>("list_runtimes");
export const availableRuntimes = (kind: "proton-ge" | "wine-ge") =>
  invoke<GitHubRelease[]>("available_runtimes", { kind });
export const installRuntime = (version: string) =>
  invoke<void>("install_runtime", { version });
export const setDefaultRuntime = (id: string) =>
  invoke<void>("set_default_runtime", { id });
