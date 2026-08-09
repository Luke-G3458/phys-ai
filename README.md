# Tauri Boilerplate

> A reusable Tauri 2 application boilerplate.

<!-- template-only:start -->
## Create a project

Create a local project with a fresh Git history:

```bash
bun create Luke-G3458/tauri-boilerplate my-app
cd my-app
bun run setup
```

To create the project on GitHub first, mark this repository as a GitHub template and run:

```bash
gh repo create my-app \
  --template Luke-G3458/tauri-boilerplate \
  --private \
  --clone

cd my-app
bun run setup
```
<!-- template-only:end -->

## Development

```bash
bun run tauri dev
```

Run all static checks:

```bash
bun run check
```
