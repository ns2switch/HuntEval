#!/usr/bin/env python3
"""Build, package, install, and smoke-test one native R8 candidate."""

from __future__ import annotations

import argparse
import gzip
import hashlib
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tarfile
import tempfile
import zipfile

import r8_platform

BINARIES = [
    "hunteval",
    "hunteval-duckdb-worker",
    "hunteval-commercial-worker",
    "hunteval-reference-deployment",
    "hunteval-reference-tool",
    "hunteval-fixture-tool",
]


def fail(message: str) -> None:
    raise SystemExit(f"error: {message}")


def run(arguments: list[str], **kwargs: object) -> subprocess.CompletedProcess:
    result = subprocess.run(arguments, check=False, **kwargs)
    if result.returncode != 0:
        fail(f"command failed with exit code {result.returncode}: {arguments[0]}")
    return result


def digest(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_json(path: pathlib.Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def stage(root: pathlib.Path, suffix: str) -> None:
    (root / "bin").mkdir(parents=True)
    (root / "docs").mkdir()
    (root / "examples/contracts/v1.0").mkdir(parents=True)
    for name in BINARIES:
        source = pathlib.Path("target/release") / f"{name}{suffix}"
        if source.is_symlink() or not source.is_file():
            fail(f"native release binary is missing: {source}")
        shutil.copy2(source, root / "bin" / source.name)
    for version in ("v0.3", "v0.4", "v0.5", "v0.6", "v0.7", "v0.8", "v0.9", "v1.0"):
        shutil.copytree(
            pathlib.Path("schemas") / version,
            root / "schemas" / version,
            ignore=shutil.ignore_patterns("ground-truth.schema.json"),
        )
    shutil.copytree("taxonomies", root / "taxonomies")
    for name in ("LICENSE", "README.md", "SECURITY.md"):
        shutil.copy2(name, root / name)
    shutil.copy2("docs/OFFICIAL_BENCHMARK_CARD.md", root / "docs/OFFICIAL_BENCHMARK_CARD.md")
    for name in ("cloud-mvp-benchmark.yaml", "scoring-profile-balanced.yaml"):
        shutil.copy2(pathlib.Path("examples") / name, root / "examples" / name)
    for name in (
        "compatibility-matrix.json",
        "interface-freeze-manifest.json",
        "migration-inventory.json",
        "official-benchmark-pack.json",
        "platform-target-matrix.json",
        "release-interface-inventory.json",
    ):
        source = pathlib.Path("examples/contracts/v1.0") / name
        shutil.copy2(source, root / "examples/contracts/v1.0" / name)


def tar_archive(source: pathlib.Path, output: pathlib.Path) -> None:
    with output.open("wb") as raw:
        with gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=0) as compressed:
            with tarfile.open(fileobj=compressed, mode="w") as archive:
                for path in sorted(source.rglob("*")):
                    if not path.is_file() or path.is_symlink():
                        continue
                    relative = pathlib.PurePosixPath("hunteval") / path.relative_to(source).as_posix()
                    info = archive.gettarinfo(str(path), arcname=str(relative))
                    info.uid = 0
                    info.gid = 0
                    info.uname = ""
                    info.gname = ""
                    info.mtime = 0
                    info.mode = 0o755 if relative.parts[1] == "bin" else 0o644
                    with path.open("rb") as content:
                        archive.addfile(info, content)


def zip_archive(source: pathlib.Path, output: pathlib.Path) -> None:
    with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        for path in sorted(source.rglob("*")):
            if not path.is_file() or path.is_symlink():
                continue
            relative = pathlib.PurePosixPath("hunteval") / path.relative_to(source).as_posix()
            info = zipfile.ZipInfo(str(relative), date_time=(1980, 1, 1, 0, 0, 0))
            mode = 0o755 if relative.parts[1] == "bin" else 0o644
            info.external_attr = (mode & 0xFFFF) << 16
            info.compress_type = zipfile.ZIP_DEFLATED
            archive.writestr(info, path.read_bytes(), compresslevel=9)


def checksums(root: pathlib.Path) -> None:
    entries = []
    for path in sorted(root.rglob("*")):
        if path.is_file() and path.name not in {"SHA256SUMS.json", "verification.json"}:
            entries.append({"path": path.relative_to(root).as_posix(), "sha256": digest(path)})
    write_json(root / "SHA256SUMS.json", {"schema_version": "1.0", "files": entries})
    write_json(root / "verification.json", {"schema_version": "1.0", "verified": True, "file_count": len(entries)})


def sign_candidate_manifest(output: pathlib.Path, revision_epoch: str) -> None:
    ssh_keygen = shutil.which("ssh-keygen")
    if ssh_keygen is None:
        fail("ssh-keygen is required for the native candidate rehearsal")
    with tempfile.TemporaryDirectory() as directory:
        key = pathlib.Path(directory) / "rehearsal-key"
        run([ssh_keygen, "-q", "-t", "ed25519", "-N", "", "-C", "hunteval-r8-native-rehearsal", "-f", str(key)])
        public_key = key.with_suffix(".pub").read_text(encoding="utf-8").strip()
        fingerprint_output = run(
            [ssh_keygen, "-lf", str(key.with_suffix(".pub")), "-E", "sha256"],
            capture_output=True,
            text=True,
        ).stdout
        fields = fingerprint_output.split()
        if len(fields) < 2:
            fail("cannot resolve rehearsal signing-key fingerprint")
        policy = pathlib.Path(directory) / "policy.json"
        repository = os.environ.get("GITHUB_REPOSITORY", "ns2switch/HuntEval")
        ref = os.environ.get("GITHUB_REF", "refs/heads/main")
        workflow = "r8-native-candidate"
        write_json(policy, {
            "schema_version": "1.0",
            "signer_identity": "hunteval-r8-native-rehearsal",
            "namespace": "hunteval-release",
            "public_key": public_key,
            "key_fingerprint": fields[1],
            "repository": repository,
            "ref": ref,
            "workflow": workflow,
            "valid_from_epoch": 0,
            "valid_until_epoch": 4102444800,
            "revoked_fingerprints": [],
        })
        artifact = output / "release-manifest.json"
        run([sys.executable, "scripts/r8_sign.py", "sign", "--artifact", str(artifact), "--key", str(key), "--policy", str(policy), "--output", str(output / "signature")])
        run([
            sys.executable, "scripts/r8_sign.py", "verify", "--artifact", str(artifact),
            "--signature-root", str(output / "signature"), "--repository", repository,
            "--ref", ref, "--workflow", workflow, "--at-epoch", revision_epoch,
        ])


def build(output: pathlib.Path, target: str, runner: str) -> None:
    entry = r8_platform.assert_host(target)
    if runner != entry["runner"]:
        fail("declared native runner does not match the platform matrix")
    if not output.is_absolute() or output == pathlib.Path(output.anchor) or output.exists():
        fail("output must be a new absolute directory")
    dirty = run(["git", "status", "--porcelain"], capture_output=True, text=True).stdout
    if dirty and os.environ.get("HUNTEVAL_RELEASE_ALLOW_DIRTY") != "1":
        fail("native candidate requires a clean worktree")
    revision = run(["git", "rev-parse", "--verify", "HEAD"], capture_output=True, text=True).stdout.strip()
    epoch = run(["git", "show", "-s", "--format=%ct", "HEAD"], capture_output=True, text=True).stdout.strip()
    rust_version = os.environ.get("HUNTEVAL_RUST_VERSION", "1.93.1")
    run(["cargo", "build", "--workspace", "--release", "--locked"])
    output.mkdir(parents=True)
    with tempfile.TemporaryDirectory() as directory:
        temporary = pathlib.Path(directory)
        package = temporary / "hunteval"
        stage(package, entry["binary_suffix"])
        files = [path.relative_to(package).as_posix() for path in sorted(package.rglob("*")) if path.is_file()]
        cli = package / "bin" / f"hunteval{entry['binary_suffix']}"
        with (output / "secret-scan.json").open("w", encoding="utf-8") as scan:
            run([str(cli), "system", "secret-scan", "--root", str(package), "--format", "json", "--", *files], stdout=scan)
        extension = ".tar.gz" if entry["archive_format"] == "tar.gz" else ".zip"
        archive_name = f"hunteval-rc-{revision[:12]}-{target}{extension}"
        archive = output / archive_name
        (tar_archive if entry["archive_format"] == "tar.gz" else zip_archive)(package, archive)
        metadata = temporary / "cargo-metadata.json"
        with metadata.open("w", encoding="utf-8") as stream:
            run(["cargo", "metadata", "--locked", "--format-version", "1"], stdout=stream)
        run([
            sys.executable, "scripts/r8_supply_chain.py", "build", "--package-root", str(package),
            "--output", str(output / "evidence"), "--metadata", str(metadata), "--revision", revision,
            "--target", target, "--rust-toolchain", rust_version, "--epoch", epoch,
        ])
        run([
            sys.executable,
            "-m",
            "pip",
            "wheel",
            "--disable-pip-version-check",
            "--no-deps",
            "--no-build-isolation",
            "--wheel-dir",
            str(output),
            "./sdk/python",
        ])
        wheels = sorted(output.glob("hunteval_sdk-*.whl"))
        if len(wheels) != 1:
            fail("native candidate must contain exactly one Python SDK wheel")
        run([sys.executable, "scripts/ci/check-python-wheel.py", str(wheels[0])])
        installed = temporary / "installed"
        run([sys.executable, "scripts/r8_install.py", "install", "--archive", str(archive), "--destination", str(installed)])
        run([sys.executable, "scripts/r8_install.py", "verify", "--root", str(installed), "--target", target])
        run([str(installed / "bin" / f"hunteval{entry['binary_suffix']}"), "--help"], stdout=subprocess.DEVNULL)
    write_json(output / "release-metadata.json", {
        "schema_version": "1.0", "revision": revision, "rust_toolchain": rust_version,
        "target": target, "runner": runner, "production_published": False,
    })
    write_json(output / "native-platform-evidence.json", {
        "schema_version": "1.0", "target": target, "runner": runner,
        "matrix_sha256": digest(r8_platform.MATRIX),
        "archive": archive_name, "archive_sha256": digest(archive),
        "native_smoke_passed": True, "production_published": False,
    })
    root_artifacts = [
        archive,
        output / "native-platform-evidence.json",
        output / "secret-scan.json",
        output / "evidence/release-manifest.json",
    ]
    root_artifacts.extend(sorted(output.glob("hunteval_sdk-*.whl")))
    write_json(output / "release-manifest.json", {
        "schema_version": "1.0",
        "release_status": "candidate",
        "production_published": False,
        "revision": revision,
        "target": target,
        "artifacts": [
            {"path": path.relative_to(output).as_posix(), "sha256": digest(path)}
            for path in root_artifacts
        ],
    })
    sign_candidate_manifest(output, epoch)
    checksums(output)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", required=True)
    parser.add_argument("--target", required=True)
    parser.add_argument("--runner", required=True)
    args = parser.parse_args()
    build(pathlib.Path(args.output), args.target, args.runner)


if __name__ == "__main__":
    main()
