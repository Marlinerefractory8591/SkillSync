#!/usr/bin/env python3
"""
SkillSync Automated Multi-Platform Release Builder & Packager
Compiles, packages, and verifies desktop distributions for macOS (DMG/App) and Windows (MSI/EXE).
Generates cryptographic SHA-256 verification manifests.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys


def compute_sha256(file_path: Path) -> str:
    sha = hashlib.sha256()
    with open(file_path, "rb") as f:
        for chunk in iter(lambda: f.read(65536), b""):
            sha.update(chunk)
    return sha.hexdigest()


def run_command(cmd, cwd=None, env=None):
    print(f"==> Running: {' '.join(cmd) if isinstance(cmd, list) else cmd}")
    res = subprocess.run(cmd, cwd=cwd, shell=isinstance(cmd, str), env=env, text=True, capture_output=True)
    if res.returncode != 0:
        print(f"Error output:\n{res.stderr}", file=sys.stderr)
        raise RuntimeError(f"Command failed with code {res.returncode}: {cmd}")
    return res.stdout


def get_project_version(root_dir: Path) -> str:
    pkg_json = root_dir / "package.json"
    if pkg_json.exists():
        try:
            data = json.loads(pkg_json.read_text(encoding="utf-8"))
            return data.get("version", "1.0.0")
        except Exception:
            pass
    return "1.0.0"


def package_release(root_dir: Path, output_dir: Path, target_os: str = "auto", skip_build: bool = False):
    current_system = platform.system().lower()
    current_target = {"darwin": "macos", "windows": "windows"}.get(current_system)
    if current_target is None:
        raise RuntimeError(
            "Release packages can only be built on macOS or Windows. "
            "Use the GitHub Actions release workflow for both platforms."
        )
    if target_os == "all":
        raise RuntimeError(
            "A local host cannot reliably produce signed native installers for both macOS and Windows. "
            "Use the multi-platform GitHub Actions release workflow."
        )
    requested_target = current_target if target_os == "auto" else target_os
    if requested_target != current_target:
        raise RuntimeError(
            f"Requested {requested_target} installer on {current_system}. "
            "Run this command on that platform or use the release workflow."
        )
    version = get_project_version(root_dir)
    print(f"Building SkillSync {requested_target} release v{version} on {current_system}...")

    output_dir.mkdir(parents=True, exist_ok=True)
    release_artifacts = []

    # 1. Build web frontend assets
    if not skip_build:
        print("==> Step 1/3: Building frontend web assets (Vite + React)...")
        run_command(["npm", "run", "build"], cwd=root_dir)

    # 2. Package Tauri binary & installers
    tauri_target_dir = root_dir / "src-tauri" / "target" / "release"
    bundle_dir = tauri_target_dir / "bundle"

    if not skip_build:
        print(f"==> Step 2/3: Invoking Tauri compiler for {current_system}...")
        run_command(["npm", "run", "tauri", "build"], cwd=root_dir)

    # 3. Collect and organize release artifacts
    print("==> Step 3/3: Harvesting and verifying release artifacts...")
    
    # macOS Bundle harvesting (.dmg, .app)
    macos_dir = output_dir / "macos"
    macos_dir.mkdir(parents=True, exist_ok=True)
    
    dmg_src_dir = bundle_dir / "dmg"
    if dmg_src_dir.exists():
        for dmg in dmg_src_dir.glob("*.dmg"):
            dest = macos_dir / dmg.name
            shutil.copy2(dmg, dest)
            release_artifacts.append(dest)
            print(f"  [macOS DMG] -> {dest.name}")

    macos_app_dir = bundle_dir / "macos"
    if macos_app_dir.exists():
        for app in macos_app_dir.glob("*.app"):
            archive_name = f"{app.stem}_{version}_macos.tar.gz"
            dest_archive = macos_dir / archive_name
            print(f"  [macOS APP Bundle Archive] -> {dest_archive.name}")
            subprocess.run(
                ["tar", "-czf", str(dest_archive), "-C", str(app.parent), app.name],
                check=True
            )
            release_artifacts.append(dest_archive)

    # Windows Bundle harvesting (.msi, .exe)
    windows_dir = output_dir / "windows"
    windows_dir.mkdir(parents=True, exist_ok=True)

    msi_src_dir = bundle_dir / "msi"
    if msi_src_dir.exists():
        for msi in msi_src_dir.glob("*.msi"):
            dest = windows_dir / msi.name
            shutil.copy2(msi, dest)
            release_artifacts.append(dest)
            print(f"  [Windows MSI] -> {dest.name}")

    nsis_src_dir = bundle_dir / "nsis"
    if nsis_src_dir.exists():
        for exe in nsis_src_dir.glob("*.exe"):
            dest = windows_dir / exe.name
            shutil.copy2(exe, dest)
            release_artifacts.append(dest)
            print(f"  [Windows NSIS Setup] -> {dest.name}")

    expected_artifacts = {
        "macos": ("DMG", any(artifact.suffix == ".dmg" for artifact in release_artifacts)),
        "windows": (
            "MSI and NSIS EXE",
            any(artifact.suffix == ".msi" for artifact in release_artifacts)
            and any(artifact.suffix == ".exe" for artifact in release_artifacts),
        ),
    }
    expected_name, is_complete = expected_artifacts[requested_target]
    if not is_complete:
        raise RuntimeError(
            f"Missing expected {expected_name} installer(s) for {requested_target}. "
            "No release manifest was generated."
        )

    # Generate Manifest & Checksums
    checksum_lines = []
    manifest_files = {}

    for artifact in release_artifacts:
        rel_path = artifact.relative_to(output_dir).as_posix()
        sha = compute_sha256(artifact)
        size = artifact.stat().st_size
        manifest_files[rel_path] = {
            "sha256": sha,
            "size_bytes": size,
            "filename": artifact.name,
        }
        checksum_lines.append(f"{sha}  {rel_path}")

    # Write SHA256SUMS.txt
    sums_file = output_dir / "SHA256SUMS.txt"
    sums_file.write_text("\n".join(checksum_lines) + "\n", encoding="utf-8")

    # Write RELEASE-MANIFEST.json
    manifest = {
        "product": "SkillSync",
        "version": version,
        "algorithm": "sha256",
        "host_platform": current_system,
        "artifact_count": len(release_artifacts),
        "files": manifest_files,
    }
    manifest_file = output_dir / "RELEASE-MANIFEST.json"
    manifest_file.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")

    print("\n" + "=" * 60)
    print(f"Release packaging complete for SkillSync v{version}!")
    print(f"Artifacts output: {output_dir}")
    print(f"Manifest written: {manifest_file.name} ({len(release_artifacts)} artifacts)")
    print("=" * 60 + "\n")

    return manifest


def main():
    parser = argparse.ArgumentParser(description="SkillSync Multi-Platform Release Packager")
    parser.add_argument(
        "--root",
        type=Path,
        default=Path(__file__).resolve().parents[1],
        help="Root repository directory",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path(__file__).resolve().parents[1] / "dist-release",
        help="Target output directory for installers and checksums",
    )
    parser.add_argument(
        "--platform",
        choices=["auto", "macos", "windows", "all"],
        default="auto",
        help="Target platform packaging mode",
    )
    parser.add_argument(
        "--skip-build",
        action="store_true",
        help="Skip compilation and harvest existing build artifacts",
    )

    args = parser.parse_args()
    manifest = package_release(
        root_dir=args.root.resolve(),
        output_dir=args.output.resolve(),
        target_os=args.platform,
        skip_build=args.skip_build,
    )
    print(json.dumps(manifest, indent=2))


if __name__ == "__main__":
    main()
