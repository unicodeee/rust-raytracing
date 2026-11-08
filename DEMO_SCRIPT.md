# PolyMesh Demo Script — 3:25 maximum

Use this as both the screen plan and narration. Keep the terminal, editor, and
rendered images ready before recording so no time is lost waiting for builds.

## 0:00–0:20 — Goal and assets

**Screen:** repository tree with `src/polymesh.rs`, `objects/`, and
`docs/renders/` expanded.

**Say:** “This update adds an indexed PolyMesh shape, linear ray/triangle
intersection, interpolated vertex normals, a triangle-only OBJ reader, and a
three-angle vase scene. The object folder contains the supplied capuchino and
vase files plus hard and smooth pyramid fixtures.”

## 0:20–0:55 — Data structure

**Screen:** `Triangle` and `PolyMesh` near the top of `src/polymesh.rs`.

**Say:** “A triangle stores three position indices and three normal indices.
They are independent, so the same position can select one face normal on one
triangle and a different normal on another. PolyMesh stores the position,
normal, and triangle arrays and implements Shape.”

**Screen:** briefly show `triangle_data`.

**Say:** “This iterator fetches all three positions and normals for every
triangle.”

## 0:55–1:35 — Intersection and smooth shading

**Screen:** `Triangle::hit`, then `impl Shape for PolyMesh`.

**Say:** “Each triangle uses Moller-Trumbore intersection. Beta and gamma give
the barycentric weights, with alpha equal to one minus beta minus gamma. The
whole-mesh hit function linearly calls each triangle, shortens the maximum
distance, and returns the closest hit. Only then are the three vertex normals
interpolated.”

**Screen:** `Hit` in `src/shapes.rs`, then the shadow code in `src/scene.rs`.

**Say:** “A hit keeps an interpolated shading normal and the triangle’s true
geometric normal. Lighting uses the smooth normal; new ray origins use the
geometric normal to avoid self-intersection. The shadow-terminator check
handles the case where those normals disagree about the light.”

**Screen:** `pyramid_hard.png`, then `pyramid_smooth.png`.

**Say:** “These have identical geometry. Separate face normals create hard
edges; shared vertex normals create the smooth result.”

## 1:35–2:05 — OBJ reader

**Screen:** `load_obj` and `parse_corner` in `src/polymesh.rs`.

**Say:** “The reader supports positions, normals, and triangle faces. It reads
both double-slash normal syntax and files that also contain ignored texture
indices. Quads are rejected. File normals are preserved; when they are
missing, the loader computes corner normals using the lecture’s 35-degree
crease rule.”

**Screen:** `capuchino_obj.png`.

**Say:** “This is the supplied 33,120-vertex, 50,000-triangle OBJ. It is shown
at preview resolution because this assignment intentionally uses linear search.”

## 2:05–2:40 — Scene construction

**Screen:** README’s “How the vase scene was made” section, then
`tools/generate_vase_scene.py`.

**Say:** “The detailed vase is the updated Tinkercad export: 36,944 vertices
and 73,884 triangles. The renderer converts its Z-up coordinates to Y-up,
centers and scales it in code, and generates its missing crease-aware normals.
The committed script makes the table, backdrop, and low-poly reference
geometry repeatable. The original OBJ is not edited.”

## 2:40–3:10 — Final scene

**Screen:** show `vase_front.png`, `vase_left.png`, and `vase_right.png`.

**Say:** “The vase and table are loaded from OBJ, lit by warm key, cool fill,
and rim lights, and rendered from three camera angles. The smooth vase and hard
table demonstrate both normal styles in one scene.”

## 3:10–3:25 — Verification

**Screen:** terminal with `cargo test` and Clippy results already visible.

**Say:** “All tests pass, including the real supplied OBJ counts, independent
normal indices, 35-degree crease handling, barycentric interpolation,
geometric normals, bounding-box rejection, and quad rejection. Each image also
reports its render time in hours, minutes, and seconds. Clippy passes with
warnings treated as errors.”
