# FGP Manager App

Desktop companion app for managing local FGP daemons. Built with **Tauri**, **SvelteKit**, and **TypeScript** and designed as a small popover window that can start/stop daemons, show status, and open the marketplace view.

## Tech stack

- Tauri 2 (Rust backend + webview)
- SvelteKit + TypeScript (frontend)
- Tailwind CSS for styling

## Project layout

```
app/
├── src/            # Svelte routes + UI
├── src-tauri/      # Tauri Rust commands & config
├── static/         # Static assets
└── vite.config.js  # Vite configuration
```

## Development

```bash
pnpm install
pnpm dev
```

## Building

```bash
pnpm install
pnpm build
pnpm tauri build
```

## Useful commands

- `pnpm dev` - Run the SvelteKit dev server
- `pnpm tauri dev` - Run the Tauri desktop shell in dev mode
- `pnpm tauri build` - Produce a production desktop build

## Notes

- The frontend relies on Tauri commands like `list_daemons`, `start_daemon`, and `stop_daemon` from the Rust backend in `src-tauri/`.
- Marketplace is rendered in a separate Tauri window opened from the main popover UI.
