# Web Note

A simple web-based Markdown note-taking app. `.md` is the single source of truth.

- Embedded image
- Wikilink
- Backlinks

## Usage

```sh
web-note-server # Default notes directory: Notes/
web-note-server --notes-dir path/to/notes
web-note-server --port 5393
web-note-server --help # More info
```

<http://localhost:5393>

Notes directory:

```text
Notes/
├── Inbox/         # Default notebook
│   └── <id>.md
├── <notebook>/    # More notebook
│   └── <id>.md
├── .assets/
│   └── <hash>.<ext>
├── .trash/
└── .cache/        # Can be added to .gitignore
    └── index.db
```

Frontmatter:

```yaml
---
id: <nanoID>
created: <RFC 3339>
updated: <RFC 3339>
tags:
  - <tag>
---
```

## Development

```sh
# BE
cargo run -p web-note-server
# or with Swagger UI at /swagger-ui
cargo run -p web-note-server --features swagger

# FE
pnpm --dir web install
pnpm --dir web dev

# Test
cargo test

# Formatter & linter
cargo fmt
cargo clippy
pnpx @biomejs/biome check --write
```

Generate `openapi.json` after the API changes:

```sh
cargo run -p web-note-server -- --openapi > openapi.json
pnpm --dir web gen:api

pnpx @biomejs/biome format --write openapi.json
```

Release build:

```sh
pnpm --dir web build
cargo build --release -p web-note-server
```

Cross-compile a statically linked Linux x86_64 binary (musl) with [cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild). Requires `zig`, which is needed to build the bundled SQLite:

```sh
rustup target add x86_64-unknown-linux-musl
pnpm --dir web build
cargo zigbuild --release -p web-note-server --target x86_64-unknown-linux-musl
```

The output is `target/x86_64-unknown-linux-musl/release/web-note-server`. It has no glibc dependency, so it can be copied to and run on any Linux x86_64 host.

## Tech Stack

- BE
  - rusqlite
  - axum
  - utoipa
  - blake3
  - nanoid
- FE
  - Solid.js
  - SolidUI
  - CodeMirror
  - Phosphoricons
