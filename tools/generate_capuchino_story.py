#!/usr/bin/env python3
"""Build a two-character capuchino story OBJ."""

from math import hypot
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "objects" / "capuchino.obj"
OUTPUT = ROOT / "objects" / "capuchino_murder_scene.obj"
EXPRESSIONS = ROOT / "objects" / "capuchino_murder_expressions.obj"


def load_obj(path):
    vertices, normals, faces = [], [], []
    for line in path.read_text().splitlines():
        words = line.split()
        if not words:
            continue
        if words[0] == "v":
            vertices.append(tuple(map(float, words[1:4])))
        elif words[0] == "vn":
            normals.append(tuple(map(float, words[1:4])))
        elif words[0] == "f":
            face = []
            for word in words[1:4]:
                fields = word.split("/")
                face.append((int(fields[0]) - 1, int(fields[2]) - 1))
            faces.append(tuple(face))
    return vertices, normals, faces


def front_facing(point):
    x, y, z = point
    return -x, y, -z


def standing(point):
    x, y, z = front_facing(point)
    return 0.82 * x - 0.75, 0.82 * y + 0.82, 0.82 * z


def fallen(point):
    x, y, z = front_facing(point)
    return 0.72 * y + 1.05, -0.72 * x + 0.70, 0.72 * z + 0.12


def standing_normal(normal):
    return front_facing(normal)


def fallen_normal(normal):
    x, y, z = front_facing(normal)
    return y, -x, z


def append_instance(output, source, point_transform, normal_transform):
    source_vertices, source_normals, source_faces = source
    vertex_offset = len(output[0])
    normal_offset = len(output[1])
    output[0].extend(point_transform(point) for point in source_vertices)
    output[1].extend(normal_transform(normal) for normal in source_normals)
    output[2].extend(
        tuple((vertex + vertex_offset, normal + normal_offset) for vertex, normal in face)
        for face in source_faces
    )


def append_mesh(output, source):
    vertex_offset = len(output[0])
    normal_offset = len(output[1])
    output[0].extend(source[0])
    output[1].extend(source[1])
    output[2].extend(
        tuple((vertex + vertex_offset, normal + normal_offset) for vertex, normal in face)
        for face in source[2]
    )


def append_quad(output, points, normal):
    vertex = len(output[0])
    normal_index = len(output[1])
    output[0].extend(points)
    output[1].append(normal)
    output[2].extend(
        [
            ((vertex, normal_index), (vertex + 1, normal_index), (vertex + 2, normal_index)),
            ((vertex, normal_index), (vertex + 2, normal_index), (vertex + 3, normal_index)),
        ]
    )


def append_bar(output, start, end, z, width, transform):
    x0, y0 = start
    x1, y1 = end
    length = hypot(x1 - x0, y1 - y0)
    px = -(y1 - y0) * width / (2.0 * length)
    py = (x1 - x0) * width / (2.0 * length)
    points = [
        transform((x0 + px, y0 + py, z)),
        transform((x0 - px, y0 - py, z)),
        transform((x1 - px, y1 - py, z)),
        transform((x1 + px, y1 + py, z)),
    ]
    normal = standing_normal((0.0, 0.0, 1.0))
    if transform is fallen:
        normal = fallen_normal((0.0, 0.0, 1.0))
    append_quad(output, points, normal)


def append_expressions(output):
    append_bar(output, (-0.38, 0.30), (-0.08, 0.17), 0.92, 0.075, standing)
    append_bar(output, (0.08, 0.17), (0.38, 0.30), 0.92, 0.075, standing)
    append_bar(output, (-0.20, -0.06), (0.0, -0.15), 0.92, 0.06, standing)
    append_bar(output, (0.0, -0.15), (0.20, -0.06), 0.92, 0.06, standing)

    for center_x in (-0.23, 0.23):
        append_bar(
            output,
            (center_x - 0.09, 0.17),
            (center_x + 0.09, 0.35),
            0.92,
            0.065,
            fallen,
        )
        append_bar(
            output,
            (center_x - 0.09, 0.35),
            (center_x + 0.09, 0.17),
            0.92,
            0.065,
            fallen,
        )
    append_bar(output, (-0.20, -0.10), (0.20, -0.10), 0.92, 0.065, fallen)


def write_obj(path, data):
    vertices, normals, faces = data
    lines = ["# Two capuchino characters in a stylized defeat scene."]
    lines.extend(f"v {x:.8f} {y:.8f} {z:.8f}" for x, y, z in vertices)
    lines.extend(f"vn {x:.8f} {y:.8f} {z:.8f}" for x, y, z in normals)
    for face in faces:
        corners = " ".join(f"{v + 1}//{n + 1}" for v, n in face)
        lines.append(f"f {corners}")
    path.write_text("\n".join(lines) + "\n")


def main():
    source = load_obj(SOURCE)
    output = ([], [], [])
    expressions = ([], [], [])
    append_instance(output, source, standing, standing_normal)
    append_instance(output, source, fallen, fallen_normal)
    append_expressions(expressions)
    append_mesh(output, expressions)
    write_obj(OUTPUT, output)
    write_obj(EXPRESSIONS, expressions)
    print(f"created {OUTPUT} with {len(output[0])} vertices and {len(output[2])} triangles")


if __name__ == "__main__":
    main()
