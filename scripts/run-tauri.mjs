import { execFileSync, spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const args = process.argv.slice(2);
const isBuild = args[0] === "build";
const isMacOS = process.platform === "darwin";
const projectRoot = fileURLToPath(new URL("../", import.meta.url));
// Call the JavaScript entry point directly. `node_modules/.bin/tauri` is a
// POSIX shell shim on Windows checkouts, so passing it to Node produces a
// syntax error before Tauri can start.
const tauriCli = join(
  projectRoot,
  "node_modules",
  "@tauri-apps",
  "cli",
  "tauri.js",
);
const env = { ...process.env };

const setMacOSSigningEnvironment = () => {
  if (!isMacOS || !isBuild || env.TAURI_SIGNING_PRIVATE_KEY) return;

  const localKeyPath = join(homedir(), ".tauri", "skillsync.key");
  if (existsSync(localKeyPath)) {
    // Tauri's bundler reads the private key content from this variable. The
    // key file itself remains outside the repository in the user's account.
    env.TAURI_SIGNING_PRIVATE_KEY = readFileSync(localKeyPath, "utf8");
  }

  if (
    !env.TAURI_SIGNING_PRIVATE_KEY_PASSWORD &&
    env.TAURI_SIGNING_PRIVATE_KEY
  ) {
    try {
      env.TAURI_SIGNING_PRIVATE_KEY_PASSWORD = execFileSync(
        "security",
        [
          "find-generic-password",
          "-a",
          "skillsync",
          "-s",
          "SkillSync updater signing password",
          "-w",
        ],
        { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] },
      ).trim();
    } catch {
      // The official CI secrets remain the authoritative release path. A
      // missing local Keychain item is reported by the signing command below.
    }
  }
};

const run = (commandArgs) => {
  const result = spawnSync(process.execPath, [tauriCli, ...commandArgs], {
    cwd: projectRoot,
    env,
    stdio: "inherit",
  });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
};

setMacOSSigningEnvironment();
run(args);

if (isMacOS && isBuild && env.TAURI_SIGNING_PRIVATE_KEY) {
  const updaterPackage = join(
    projectRoot,
    "src-tauri",
    "target",
    "release",
    "bundle",
    "macos",
    "SkillSync.app.tar.gz",
  );

  if (existsSync(updaterPackage)) {
    run(["signer", "sign", updaterPackage]);
  }
}
