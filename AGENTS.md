# Safe Keep

Tauri 2.x + Rust + Vue 3 + Vite photo/video backup desktop app.

## Commands

| Action     | Command          | Notes                                                  |
| ---------- | ---------------- | ------------------------------------------------------ |
| dev        | `pnpm dev`       | Vite on port **1420** (strict)                         |
| build      | `pnpm build`     | runs `vue-tsc --build` then `vite build`               |
| type-check | `pnpm ts:check`  | `vue-tsc --noEmit --skipLibCheck`                      |
| test       | `pnpm test:unit` | Vitest (jsdom). No tests exist yet.                    |
| lint       | `pnpm lint`      | sequential: oxlint → eslint → stylelint → lint-staged  |
| format     | `pnpm format`    | Prettier (no semi, single quotes, trailingComma: none) |

Pre-commit hook runs: `ts:check` → `lint:lint-staged`. Commits must pass conventional-commitlint (extended types: feat/fix/docs/style/refactor/perf/test/ci/chore/revert/workflow/mod/wip/types/release).

## Structure

- `src/` — Vue 3 frontend (entry: `main.ts`, router, views, locales, composables, styles)
- `src-tauri/` — Rust backend (minimal: only `lib.rs` + `main.rs`, no custom commands yet)
- `@/` path alias → `./src/`
- `types/` contains auto-generated `auto-imports.d.ts` and `components.d.ts` (gitignored, rebuilt on dev)

## Framework quirks

- **Auto-imports**: Vue, Vue Router, and Pinia APIs are globally auto-imported (no explicit imports needed)
- **Auto-components**: Element Plus components + `icon-*` local icons are auto-registered; `icon-*` icons come from `src/assets/icons/` SVG files
- **UnoCSS**: Wind4 preset + Attributify with `un-` prefix and `prefixedOnly: true` (use `un-` prefix for attributify mode)
- **SCSS**: `src/styles/variables.module.scss` is globally injected via Vite `additionalData`
- **i18n**: vue-i18n, flat key structure (`page.key`), zh-CN default with English fallback
- **Theme**: `useTheme` composable, persisted in `localStorage`, controlled via `data-theme` on `<html>`
- **Element Plus style**: full import in dev (`VITE_USE_ALL_ELEMENT_PLUS_STYLE=true`), on-demand in prod
- **Tauri dev**: `beforeDevCommand` = `pnpm dev`, `beforeBuildCommand` = `pnpm build`, frontend served at `http://localhost:1420`

## Coding conventions

- **Filenames**: `kebab-case` (two-or-more-words.ts, never `twoOrMoreWords.ts` or `two_or_more.ts`)
- **Variables/functions**: `camelCase` (never snake_case or PascalCase for variables/functions)
- **Reusability**: extract shared logic into composables, utilities, or helpers; avoid code duplication
- **Components**: PascalCase (Vue convention), filenames still `kebab-case`

## Rust backend (src-tauri)

- `tauri 2.x`, no custom IPC commands defined yet (only `core:default` capability)
- `tauri-plugin-log` loaded in debug builds only
- Planned modules (from feature spec) are not yet implemented: scanner, copier, database, hasher, etc.
