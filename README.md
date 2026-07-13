# xodr-qcr

An experimental LLM AI Rust rewrite of the [ASAM OpenDRIVE Quality
Checker](https://github.com/asam-ev/qc-opendrive) (`qc-opendrive`). Written
initially by `Hy3 (free)`. Inspired by the distribution and usage simplicity
of other useful single binary Rust tools such as ripgrep, uv, ruff, etc.

It checks OpenDRIVE `.xodr` files for quality issues — 26 checkers covering
basic well-formedness, schema validation, semantic, geometry, performance, and
smoothness rules.

## Run

```sh
# Print the text report to stdout
xodr-qcr path/to/file.xodr

# Also write the .xqar XML report
xodr-qcr path/to/file.xodr -o report.xqar

# Suppress the text report
xodr-qcr path/to/file.xodr -q -o report.xqar
```

Exit code is `1` if any ERROR-level issue is found, otherwise `0`.

### Build from source

```sh
cargo build --release
# binary: target/release/xodr-qcr
```

## Download

Prebuilt binaries for Linux, macOS, and Windows are attached to each
[GitHub release](https://github.com/wo9xr4d/xodr-qcr/releases).

## License

MPL-2.0
