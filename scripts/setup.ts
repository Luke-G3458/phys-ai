import { basename, join } from "node:path";

type PackageJson = {
  name: string;
  version: string;
  description?: string;
  [key: string]: unknown;
};

type TauriConfig = {
  productName?: string;
  version?: string;
  identifier: string;
  [key: string]: unknown;
};

const root = process.cwd();
const args = new Set(Bun.argv.slice(2));
const useDefaults = args.has("--yes") || args.has("-y");
const skipChecks = args.has("--skip-checks");

if (args.has("--help") || args.has("-h")) {
  console.log(`Usage: bun run setup [options]

Options:
  -y, --yes        Accept all derived defaults
  --skip-checks    Skip cargo check, lint, and frontend build
  -h, --help       Show this help
`);
  process.exit(0);
}

function titleCase(value: string): string {
  return value
    .split(/[-_\s]+/)
    .filter(Boolean)
    .map((word) => word[0]!.toUpperCase() + word.slice(1))
    .join(" ");
}

function slugify(value: string): string {
  return value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

function ask(label: string, defaultValue: string): string {
  if (useDefaults) {
    console.log(`${label}: ${defaultValue}`);
    return defaultValue;
  }

  const answer = prompt(`${label} (${defaultValue})`);
  return answer?.trim() || defaultValue;
}

function assertValid(condition: boolean, message: string): asserts condition {
  if (!condition) {
    console.error(`\nSetup failed: ${message}`);
    process.exit(1);
  }
}

async function read(path: string): Promise<string> {
  return Bun.file(join(root, path)).text();
}

async function write(path: string, contents: string): Promise<void> {
  await Bun.write(join(root, path), contents);
}

function replaceTomlField(
  contents: string,
  section: string,
  field: string,
  value: string | string[],
): string {
  const sectionPattern = new RegExp(
    `(\\[${section.replace(/[.*+?^${}()|[\\]\\\\]/g, "\\\\$&")}\\][\\s\\S]*?)(?=\\n\\[|$)`,
  );
  const match = contents.match(sectionPattern);
  assertValid(Boolean(match), `Could not find [${section}] in src-tauri/Cargo.toml.`);

  const fieldPattern = new RegExp(`^${field}\\s*=.*$`, "m");
  assertValid(
    fieldPattern.test(match![1]!),
    `Could not find ${field} in [${section}] in src-tauri/Cargo.toml.`,
  );
  const updatedSection = match![1]!.replace(fieldPattern, `${field} = ${JSON.stringify(value)}`);

  return contents.replace(match![1]!, updatedSection);
}

async function run(command: string[]): Promise<void> {
  console.log(`\n$ ${command.join(" ")}`);
  const process = Bun.spawn(command, {
    cwd: root,
    stdin: "inherit",
    stdout: "inherit",
    stderr: "inherit",
  });
  const exitCode = await process.exited;
  assertValid(exitCode === 0, `${command.join(" ")} exited with code ${exitCode}.`);
}

const packagePath = "package.json";
const tauriPath = "src-tauri/tauri.conf.json";
const cargoPath = "src-tauri/Cargo.toml";
const mainPath = "src-tauri/src/main.rs";

const packageJson = JSON.parse(await read(packagePath)) as PackageJson;
const tauriConfig = JSON.parse(await read(tauriPath)) as TauriConfig;
let cargoToml = await read(cargoPath);
let mainRs = await read(mainPath);

const folderSlug = slugify(basename(root)) || packageJson.name;
const defaultDisplayName = titleCase(folderSlug);
const identifierParts = tauriConfig.identifier.split(".");
const identifierPrefix =
  identifierParts.length > 1 ? identifierParts.slice(0, -1).join(".") : "com.example";

console.log("Configure this Tauri project. Press Enter to accept any default.\n");

const displayName = ask("Display name", defaultDisplayName);
const slug = ask("Project slug", slugify(folderSlug));
const identifier = ask("Bundle identifier", `${identifierPrefix}.${slug}`);
const description = ask(
  "Description",
  packageJson.name === "tauri-boilerplate"
    ? `${displayName} desktop application`
    : packageJson.description || `${displayName} desktop application`,
);

const currentAuthor = cargoToml.match(/^authors\s*=\s*\["([^"]*)"\]/m)?.[1] || "";
const author = ask("Author", currentAuthor || "Your Name");

assertValid(displayName.length > 0, "Display name cannot be empty.");
assertValid(
  !/[\/\\:*?"<>|]/.test(displayName),
  'Display name cannot contain / \\ : * ? " < > or |.',
);
assertValid(
  /^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$/.test(slug),
  "Project slug must start with a letter and contain only lowercase letters, numbers, and hyphens.",
);
assertValid(
  /^[A-Za-z0-9-]+(?:\.[A-Za-z0-9-]+)+$/.test(identifier),
  "Bundle identifier must use reverse-DNS form, such as com.example.my-app.",
);

const oldLibName = cargoToml.match(/\[lib\][\s\S]*?^name\s*=\s*"([^"]+)"/m)?.[1];
assertValid(Boolean(oldLibName), "Could not find the Rust library name.");
const rustLibName = `${slug.replaceAll("-", "_")}_lib`;

packageJson.name = slug;
packageJson.description = description;
await write(packagePath, `${JSON.stringify(packageJson, null, 2)}\n`);

tauriConfig.productName = displayName;
tauriConfig.version = packageJson.version;
tauriConfig.identifier = identifier;
await write(tauriPath, `${JSON.stringify(tauriConfig, null, 2)}\n`);

cargoToml = replaceTomlField(cargoToml, "package", "name", slug);
cargoToml = replaceTomlField(cargoToml, "package", "version", packageJson.version);
cargoToml = replaceTomlField(cargoToml, "package", "description", description);
cargoToml = replaceTomlField(cargoToml, "package", "authors", [author]);
cargoToml = replaceTomlField(cargoToml, "lib", "name", rustLibName);
await write(cargoPath, cargoToml);

mainRs = mainRs.replace(new RegExp(`\\b${oldLibName}::run\\(\\)`), `${rustLibName}::run()`);
assertValid(
  mainRs.includes(`${rustLibName}::run()`),
  `Could not update the Rust entry point from ${oldLibName} to ${rustLibName}.`,
);
await write(mainPath, mainRs);

const indexHtml = (await read("index.html")).replace(
  /<title>.*?<\/title>/,
  `<title>${displayName}</title>`,
);
await write("index.html", indexHtml);

const readme = (await read("README.md"))
  .replace(/^# .*$/m, `# ${displayName}`)
  .replace(/^> .*$/m, `> ${description}`)
  .replace(/\n?<!-- template-only:start -->[\s\S]*?<!-- template-only:end -->\n?/m, "\n");
await write("README.md", readme);

const agents = (await read("AGENTS.md"))
  .replace(/^# Agents file for .*$/m, `# Agents file for ${displayName}`)
  .replace(
    /^This is a boilerplate for tauri 2\.0 apps, prewired with$/m,
    `${displayName} is a Tauri 2.0 app prewired with`,
  );
await write("AGENTS.md", agents);

console.log(`
Configured ${displayName}:
  package:    ${slug}
  Rust lib:   ${rustLibName}
  identifier: ${identifier}
`);

await run(["bun", "install"]);

if (!skipChecks) {
  await run(["cargo", "check", "--manifest-path", "src-tauri/Cargo.toml"]);
  await run(["bun", "run", "lint"]);
  await run(["bun", "run", "build"]);
}

console.log(`
Setup complete. Start the app with:

  bun run tauri dev
`);
