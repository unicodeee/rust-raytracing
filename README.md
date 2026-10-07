# Rust Ray Tracer

Ray tracing 3D renderer built from scratch with Rust lang. (in progress)

[Rebuild pictures](docs/render-progression.py).

## 1. One ball.

![One flat red ball on black, rendered from commit 92c1cda](docs/renders/01-ball.png)

## 2. More balls.

![Three flat colored balls, rendered from commit 9c35c6a](docs/renders/02-balls.png)

## 3. Add plane. Balls get floor.

![Three balls above a plane, rendered from commit 97975ad](docs/renders/03-plane.png)

## 4. Add lights.

![Two lights illuminate the balls and floor, rendered from commit 5a11d03](docs/renders/04-lights.png)

## 5. Add ambient shading.

![Ambient shading brightens the balls, rendered from commit 327dfe3](docs/renders/05-ambient.png)

## 6. Make shiny. Blinn–Phong shading.

![Shaded balls with highlights, rendered from commit 83037a0](docs/renders/06-shading.png)

## 7. Add shadows.

![Four shaded balls cast shadows onto the floor, rendered from commit 55a7b8e](docs/renders/07-shadows.png)

## Run it.

Need Rust. Run:

```sh
cargo run --release
```

Four views. Four PNGs: `output_front.png`, `output_right.png`, `output_back.png`, `output_left.png`.

## To be continued... 😆
