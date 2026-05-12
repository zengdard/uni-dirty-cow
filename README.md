# uni-dirty-cow

Small Rust proof-of-concept used for a university lab report on CVE-2016-5195,
commonly known as Dirty COW.

## Safety notice

This project is for controlled lab and research use only. The program attempts
to exercise a historical Linux kernel race condition and writes a replacement
first line for `/etc/passwd` when the race lands. Do not run it on systems you
do not own or administer, and do not run it outside an isolated test VM.

## Repository layout

- `src/whynot.rs` contains the Rust proof-of-concept.
- `docs/` contains the accompanying report artifacts.
- `Cargo.toml` defines the Rust package metadata and dependencies.

## Build

```sh
cargo build
```

For a release binary:

```sh
cargo build --release
```

For the Linux x64 lab binary used by this repo:

```sh
docker run --rm --platform linux/amd64 \
  -u "$(id -u):$(id -g)" \
  -e CARGO_HOME=/volume/.cargo-home \
  -v "$PWD:/volume" \
  -w /volume \
  clux/muslrust:stable \
  cargo build --release --target x86_64-unknown-linux-musl
cp target/x86_64-unknown-linux-musl/release/whynot ./whynot
```

The resulting binaries are written under `target/`, which is intentionally not
tracked by git. The root-level `whynot` file is the rebuilt Linux x64 release
artifact.

## Run

Only run the binary in a disposable Linux VM prepared for this lab.

```sh
cargo run --release
```
