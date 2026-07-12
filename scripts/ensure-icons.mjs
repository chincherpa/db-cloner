// Generates the Tauri icon set on first run (icons are derived artifacts and
// not checked into git). Runs automatically via beforeDevCommand/beforeBuildCommand.
import { existsSync } from "node:fs";
import { execSync } from "node:child_process";

if (existsSync("src-tauri/icons/icon.ico")) {
  process.exit(0);
}
console.log("App-Icons fehlen — generiere sie…");
execSync("node scripts/gen-icon.mjs .tauri-icon.png", { stdio: "inherit" });
execSync("tauri icon .tauri-icon.png -o src-tauri/icons", { stdio: "inherit" });
console.log("Icons erstellt unter src-tauri/icons/");
