# Screenshots

This directory holds GUI screenshots used in the README and user docs.
Screenshots are not committed yet -- this file documents what needs to be
captured and where each image is referenced.

## What to capture

Run `npm run tauri dev` in `fenrir-gui/`, set up a populated library (use
sample games or seed the DB manually), and grab the following:

| Filename | What it shows | Where it's used |
|----------|--------------|-----------------|
| `library.png` | Library view with 6-10 games in the grid, one selected showing the detail panel. Mix of statuses (Ready, Configured, Detected) and stores (Steam, GOG, Epic). | [README.md](../../../README.md#screenshots) |
| `scan.png` | ScanView with results phase showing high-confidence games and a couple needs-confirmation entries. | docs/user (future GUI guide) |
| `runtime-manager.png` | RuntimeManager with at least one installed runtime and the available-runtimes panel open. | docs/user (future GUI guide) |
| `settings.png` | Settings screen with the General/Scan/Defaults/Privacy sections visible. | docs/user (future GUI guide) |
| `log-viewer.png` | LogViewer with a sample game log scrolled to show a few lines of Wine output. | [docs/user/troubleshooting.md](../../user/troubleshooting.md) |
| `onboarding.png` | OnboardingWizard mid-flow, showing one of the steps (scan path or runtime install). | [docs/user/installation.md](../../user/installation.md) |
| `melonloader.png` | (Optional) Configure dialog showing winetricks `dotnetdesktop6` install in progress. | [docs/user/troubleshooting/melonloader-games.md](../../user/troubleshooting/melonloader-games.md) |

## Conventions

- **Format:** PNG, lossless. Don't use JPEG for UI screenshots -- text gets
  fuzzy.
- **Theme:** dark theme by default (matches Fenrir's defaults). Take a
  `library-light.png` if we ever want to showcase light mode.
- **Resolution:** capture at 1920×1080 or whatever the GUI window renders at
  natively. Don't upscale.
- **Window chrome:** include the Tauri window decorations (title bar) so it's
  obvious this is a desktop app, not a web page.
- **Demo data:** use fake but plausible game names. Avoid identifiable real
  cracked-game names in screenshots -- "Game A", "Game B" or use fully legit
  games (free demos, FOSS games like SuperTuxKart for a Wine demo, etc.).
- **File size:** if a PNG ends up over 500 KB after capture, run it through
  `oxipng -o 4` or `pngquant` to compress. Don't commit multi-MB images.

## Adding new screenshots

1. Capture the image, name it according to the table above.
2. Drop the file in this directory.
3. The Markdown references already exist -- they'll start rendering as soon
   as the file is in place.
4. If you add a screenshot for a new feature/screen not in the table, add a
   row above and reference it from the relevant doc.

## Animated GIFs

If a static screenshot doesn't capture a feature (e.g. scan progress, runtime
download), use a short GIF (under 5 seconds, under 2 MB). Name with `.gif`
extension and add it to the table. Tools: `peek`, `byzanz-record`, or
`ffmpeg` from an X11 screen recording.
