# Contributing to GameRunner

## How to Help

### Code Contributions

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run `make test-all && make lint`
5. Submit a pull request

### Where to Start

- **Good first issues**: UI polish, theme creation, accessibility improvements
- **Rust work**: Wine version management, bottle configuration, Steam VDF parsing
- **Compatibility database**: Submit game compatibility reports, add entries to `compat-db/entries/`
- **Documentation**: Improve docs, write tutorials, record demos

### Development Setup

```bash
git clone https://github.com/user/gamerunner.git
cd gamerunner
make bootstrap
make dev
```

### Project Conventions

**Rust:**
- Follow standard Rust idioms (`cargo clippy` must pass)
- Modules under `src-tauri/src/` are backend logic
- `src-tauri/src/commands/` are Tauri command handlers (thin wrappers around modules)
- Use `thiserror` for error types, `tracing` for logging
- Tests go in the same file as the module (`#[cfg(test)]`)

**TypeScript / React:**
- TypeScript strict mode (no `any` without good reason)
- Components in `ui/src/components/` are reusable; views in `ui/src/views/` are page-level
- Use Tailwind CSS for styling; avoid inline styles
- Hooks in `ui/src/hooks/` wrap Tauri `invoke()` calls
- Run `cd ui && npm run lint` before committing

**Commits:**
- Conventional commits preferred (`feat:`, `fix:`, `docs:`, `test:`)
- Keep commits focused; one logical change per commit

## Compatibility Database

The compatibility DB lives in `compat-db/entries/`. Each file is a JSON object matching `schema.json`.

To add a game entry:
1. Create `compat-db/entries/<steam_app_id>.json`
2. Fill out all fields per the schema
3. Submit a PR

## Testing

```bash
cargo test                 # Rust unit tests
cd ui && npm run test      # Frontend tests
```

Integration tests require a real Wine installation and Steam client. These run in CI but can also be run locally with Wine installed.

## Architecture

See `/docs/architecture.md` and the plan at `.claude/plans/` for the full design.
