# Header SVG Design Prompt

Create a GitHub profile header as a deterministic SVG, not a raster image.

## Intent

Build a dark, futuristic engineering map for Yu Sun / bioinformatist. The image
should feel like a control-room dashboard and a technical constellation: precise,
high-signal, and visually rich without becoming a generic skills badge wall.

## Required Text

- Center focus: `Rust`
- Main identity line: `Industrial Systems / Bioinformatics Research / Financial Engineering`
- Supporting line: `open source · data infrastructure · reproducible engineering`
- Bioinformatics must be visually prominent even if it occupies a corner.

## Layout

- Canvas: 1280x420.
- Dark background with subtle grid, scanlines, and faint technical traces.
- Center: a large Rust core node, lower-middle or near center.
- Three primary clusters around the core:
  - `Industrial Systems`: Linux, NixOS, Containers, CI/CD, Web Backend.
  - `Bioinformatics Research`: R, Python, RNA-seq, Bioconductor, pipelines.
  - `Financial Engineering`: time-series, PostgreSQL, TimescaleDB, Qdrant,
    decision systems.
- Add a smaller `Open Source` strip with `burn`, `rfd`, and `cargo-pgo`.

## Visual Style

- Geeky and polished, with neon cyan, electric violet, amber, and bio-green
  accents.
- Avoid a one-color palette.
- Use crisp vector shapes, thin strokes, and readable text.
- Do not include skill icons copied from third-party brands. Use text labels and
  abstract glyphs/nodes instead.
- Avoid emojis, stock-art motifs, and generic corporate illustration.

## README Constraints

- The SVG must render well on GitHub in light and dark mode.
- Keep all text inside the 1280x420 viewport.
- Do not rely on external fonts, scripts, remote images, or CSS files.
- Prefer inline SVG definitions and simple animations only if GitHub rendering
  remains safe.
