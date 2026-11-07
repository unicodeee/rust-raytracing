#!/usr/bin/env python3
"""Generate the small OBJ assets used by the PolyMesh vase scene."""

from math import cos, pi, sin
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OBJECTS = ROOT / "objects"


def write_obj(name, vertices, faces, comment):
    lines = [f"# {comment}"]
    lines += [f"v {x:.6f} {y:.6f} {z:.6f}" for x, y, z in vertices]
    lines += [f"f {a + 1} {b + 1} {c + 1}" for a, b, c in faces]
    (OBJECTS / name).write_text("\n".join(lines) + "\n")


def build_vase(segments=16):
    # A Blender-style screw/lathe profile: (height, radius).
    profile = [
        (-0.25, 0.52),
        (-0.05, 0.62),
        (0.35, 0.48),
        (0.75, 0.55),
        (1.15, 0.76),
        (1.55, 0.62),
        (1.85, 0.36),
        (2.00, 0.34),
    ]
    vertices = []
    for y, radius in profile:
        for segment in range(segments):
            angle = 2.0 * pi * segment / segments
            vertices.append((radius * cos(angle), y, radius * sin(angle)))

    faces = []
    for ring in range(len(profile) - 1):
        for segment in range(segments):
            next_segment = (segment + 1) % segments
            a = ring * segments + segment
            b = ring * segments + next_segment
            c = (ring + 1) * segments + next_segment
            d = (ring + 1) * segments + segment
            faces += [(a, c, b), (a, d, c)]

    bottom = len(vertices)
    vertices.append((0.0, profile[0][0], 0.0))
    for segment in range(segments):
        next_segment = (segment + 1) % segments
        faces.append((bottom, segment, next_segment))
    return vertices, faces


def add_quad(vertices, faces, a, b, c, d):
    # Vertices are duplicated per face so generated normals remain hard.
    start = len(vertices)
    vertices.extend([a, b, c, d])
    faces.extend([(start, start + 1, start + 2), (start, start + 2, start + 3)])


def add_box(vertices, faces, minimum, maximum):
    x0, y0, z0 = minimum
    x1, y1, z1 = maximum
    add_quad(vertices, faces, (x1, y0, z0), (x1, y1, z0), (x1, y1, z1), (x1, y0, z1))
    add_quad(vertices, faces, (x0, y0, z1), (x0, y1, z1), (x0, y1, z0), (x0, y0, z0))
    add_quad(vertices, faces, (x0, y1, z0), (x0, y1, z1), (x1, y1, z1), (x1, y1, z0))
    add_quad(vertices, faces, (x0, y0, z1), (x0, y0, z0), (x1, y0, z0), (x1, y0, z1))
    add_quad(vertices, faces, (x1, y0, z1), (x1, y1, z1), (x0, y1, z1), (x0, y0, z1))
    add_quad(vertices, faces, (x0, y0, z0), (x0, y1, z0), (x1, y1, z0), (x1, y0, z0))


def build_table():
    vertices, faces = [], []
    add_box(vertices, faces, (-2.5, -0.55, -1.5), (2.5, -0.25, 1.5))
    for x0, x1 in [(-2.25, -1.95), (1.95, 2.25)]:
        for z0, z1 in [(-1.25, -0.95), (0.95, 1.25)]:
            add_box(vertices, faces, (x0, -2.0, z0), (x1, -0.55, z1))
    return vertices, faces


def build_backdrop():
    vertices, faces = [], []
    # Floor normal points up; wall normal points toward the cameras (-Z).
    add_quad(vertices, faces, (-30.0, -2.0, -20.0), (30.0, -2.0, -20.0), (30.0, -2.0, 2.8), (-30.0, -2.0, 2.8))
    add_quad(vertices, faces, (-30.0, -2.0, 2.8), (30.0, -2.0, 2.8), (30.0, 20.0, 2.8), (-30.0, 20.0, 2.8))
    return vertices, faces


def main():
    OBJECTS.mkdir(exist_ok=True)
    write_obj("vase_scene_vase.obj", *build_vase(), "Lathed low-poly vase; smooth normals are generated on load.")
    write_obj("vase_scene_table.obj", *build_table(), "Table assembled from five hard-edged boxes.")
    write_obj("vase_scene_backdrop.obj", *build_backdrop(), "Floor and vertical studio backdrop.")


if __name__ == "__main__":
    main()
