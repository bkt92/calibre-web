# Calibre-Web Rust Rewrite

This repository contains the Rust rewrite of Calibre-Web, a web application for browsing, reading, and downloading eBooks from a Calibre library.

## Repository Structure

```
calibre-web/
├── calibre-web-rust/     # Rust implementation (in progress)
│   ├── src/              # Source code
│   ├── migrations/       # Database migrations
│   ├── tests/            # Test suites
│   └── Cargo.toml        # Rust dependencies
├── legacy/               # Original Python/Flask implementation
│   ├── cps/              # Calibre-Web Python source
│   ├── cps.py            # Entry point
│   └── ...               # Python dependencies
└── docs/                 # Documentation
    ├── superpowers/      # Design specs and implementation plans
    └── ...               # Other documentation
```

## Rust Implementation Status

**Phase 1 & 2** (Foundation + Core Features): In Planning

See `docs/superpowers/plans/2025-03-29-calibre-web-rust-rewrite-phase1-2.md` for the complete implementation plan.

### Architecture

- **Web Framework**: Axum 0.7+ with Tokio async runtime
- **Database**: PostgreSQL as single source of truth (SQLx 0.7+)
- **Calibre Integration**: Import/export/sync with Calibre SQLite
- **Templates**: Tera 0.20+ (Jinja2-compatible)
- **Sessions**: Encrypted cookies (no Redis/database)
- **Caching**: Moka 0.12+ (in-memory)

## Legacy Python Implementation

The original Python/Flask implementation is preserved in the `legacy/` directory for reference and gradual migration.

### Running Legacy Version

```bash
cd legacy
python cps.py
```

Access at `http://localhost:8083` (default credentials: `admin` / `admin123`)

## Documentation

- **Design Spec**: `docs/superpowers/specs/2025-03-29-calibre-web-rust-rewrite-design.md`
- **Sync Strategy**: `docs/superpowers/specs/2025-03-29-calibre-sync-strategy.md`
- **Implementation Plan**: `docs/superpowers/plans/2025-03-29-calibre-web-rust-rewrite-phase1-2.md`

## Development

### Prerequisites

- Rust 1.70+ (`rustup` recommended)
- PostgreSQL 14+
- Node.js 18+ (for frontend assets)

### Setup Rust Environment

```bash
cd calibre-web-rust
cargo build
```

### Running Tests

```bash
cd calibre-web-rust
cargo test
```

## Contributing

Please see `CONTRIBUTING.md` in the `legacy/` directory for contribution guidelines.

## License

GPL v3 - See `legacy/LICENSE` for details.
