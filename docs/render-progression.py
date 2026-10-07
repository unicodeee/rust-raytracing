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
    ("43908ae", "output_front.png", "08-colored-lights.png"),
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

            # Use the existing yellow and purple lights for the final picture.
            # Keep the historical geometry, camera, and shading unchanged.
            if saved == "08-colored-lights.png":
                renderer = scene / "src" / "render.rs"
                code = renderer.read_text()
                before = "    let lights = vec![light1, light2];\n    // let lights = vec![yellow_light, purple_light];"
                after = "    // let lights = vec![light1, light2];\n    let lights = vec![yellow_light, purple_light];"
                if code.count(before) != 1:
                    raise RuntimeError("Expected historical light selection was not found")
                renderer.write_text(code.replace(before, after))

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
