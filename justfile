set windows-shell := ["cmd", "/c"]

dev:
		bun run tauri dev

build:
		bun run tauri build

test:
		cd src-tauri && cargo test --workspace 2>&1

setup:
		bun run setup

check:
		bun run check
