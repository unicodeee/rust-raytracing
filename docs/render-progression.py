#!/usr/bin/env python3
"""Rebuild the README pictures from the original commits. Needs Git and Cargo."""

from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile


# Source commit, original output, saved picture. Keep each historical scene intact.
STAGES = (
    ("92c1cda", "output.png", "01-ball.png"),
    ("9c35c6a", "output_front.png", "02-balls.png"),
    ("97975ad", "output_front.png", "03-plane.png"),
    ("5a11d03", "output_front.png", "04-lights.png"),
    ("327dfe3", "output_front.png", "05-ambient.png"),
    ("83037a0", "output_front.png", "06-shading.png"),
    ("55a7b8e", "output_front.png", "07-shadows.png"),
)


def main():
    root = Path(__file__).resolve().parents[1]
    pictures = root / "docs" / "renders"
    pictures.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="rust-render-history-") as temp:
        scratch = Path(temp)
        for commit, original, saved in STAGES:
            scene = scratch / commit
            scene.mkdir()
            archive = scratch / f"{commit}.tar"
            subprocess.run(
                ["git", "archive", "--format=tar", f"--output={archive}", commit],
                cwd=root, check=True,
            )
            with tarfile.open(archive) as source:
                source.extractall(scene, filter="data")

            # The first sphere predates Cargo.toml. Use the project's first
            # manifest and lockfile, without changing any historical Rust code.
            if not (scene / "Cargo.toml").exists():
                for filename in ("Cargo.toml", "Cargo.lock"):
                    data = subprocess.check_output(
                        ["git", "show", f"2818ddf:{filename}"], cwd=root,
                    )
                    (scene / filename).write_bytes(data)

            print(f"Rendering {saved} from {commit}", flush=True)
            # Archived files have old timestamps. Clear this package's build
            # artifacts so Cargo cannot reuse a different stage's executable.
            subprocess.run(
                ["cargo", "clean", "--release", "--package", "rust-learn",
                 "--offline", "--locked", "--target-dir", str(scratch / "target")],
                cwd=scene, check=True,
            )
            subprocess.run(
                ["cargo", "run", "--release", "--offline", "--locked",
                 "--target-dir", str(scratch / "target")],
                cwd=scene, check=True,
            )
            shutil.copyfile(scene / original, pictures / saved)


if __name__ == "__main__":
    main()
