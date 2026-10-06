// Keep every shipped MaoMao target on the same public version.
// Tauri, npm, Cargo and Xcode each carry a version because their build tools
// require it; this check prevents a platform from silently shipping an older
// number than the rest of the product.

import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(fileURLToPath(new URL("..", import.meta.url)));
const repo = resolve(root, "..");
const read = (file) => readFileSync(resolve(root, file), "utf8");
const json = (file) => JSON.parse(read(file));

const versions = {
  "macOS GitHub": readFileSync(resolve(repo, "NotchBuddy/project.yml"), "utf8")
    .match(/CFBundleShortVersionString: "([^"]+)"/)?.[1],
  "macOS App Store": readFileSync(resolve(repo, "NotchBuddy/project.yml"), "utf8")
    .match(/CFBundleShortVersionString: "([^"]+)"[\s\S]*CFBundleShortVersionString: "([^"]+)"/)?.[2],
  "Windows/Linux Tauri": json("src-tauri/tauri.conf.json").version,
  "Windows/Linux npm": json("package.json").version,
  "Windows/Linux npm lock": json("package-lock.json").version,
  "Windows/Linux Cargo": read("Cargo.toml").match(/^version = "([^"]+)"/m)?.[1],
};

const missing = Object.entries(versions).filter(([, value]) => !value);
if (missing.length) {
  console.error(`Could not read version from: ${missing.map(([name]) => name).join(", ")}`);
  process.exit(1);
}

const unique = [...new Set(Object.values(versions))];
if (unique.length !== 1) {
  console.error("MaoMao release versions are out of sync:");
  for (const [name, version] of Object.entries(versions)) console.error(`  ${name}: ${version}`);
  process.exit(1);
}

console.log(`MaoMao version ${unique[0]} is aligned across macOS, Windows and Linux.`);
