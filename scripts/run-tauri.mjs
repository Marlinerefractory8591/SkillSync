import { execFileSync, spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const args = process.argv.slice(2);
const isBuild = args[0] === "build";
const isMacOS = process.platform === "darwin";
const projectRoot = fileURLToPath(new URL("../", import.meta.url));
const tauriCli = join(projectRoot, "node_modules", ".bin", "tauri");
const env = { ...process.env };

const setMacOSSigningEnvironment = () => {
  if (!isMacOS || !isBuild || env.TAURI_SIGNING_PRIVATE_KEY) return;

  const localKeyPath = join(homedir(), ".tauri", "skillsync.key");
  if (!env.TAURI_SIGNING_PRIVATE_KEY_PATH && existsSync(localKeyPath)) {
    env.TAURI_SIGNING_PRIVATE_KEY_PATH = localKeyPath;
  }

  if (
    !env.TAURI_SIGNING_PRIVATE_KEY_PASSWORD &&
    env.TAURI_SIGNING_PRIVATE_KEY_PATH
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

if (isMacOS && isBuild && env.TAURI_SIGNING_PRIVATE_KEY_PATH) {
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
