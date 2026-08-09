# zabal-cli

LoopHouse CLI. Auto-generate ZABAL hackathon submission documents straight from your Git commit history.

Instead of writing up what you built at the end of a hackathon, `zabal` reads the commits you already
made and turns them into a submission document.

## Install

Requires a Rust toolchain (`rustup`, which provides `cargo`) and a C compiler, since `git2`
builds libgit2 from source. On Linux and macOS the system compiler is enough. On Windows use
either the MSVC build tools or a MinGW-w64 toolchain with the `x86_64-pc-windows-gnu` target;
both are known to work.

```bash
git clone https://github.com/imanafrikah-base-eth/zabal-cli.git
cd zabal-cli
cargo build --release
```

The binary lands at `target/release/zabal`. To install it onto your PATH:

```bash
cargo install --path .
```

## Usage

```
zabal <COMMAND>
```

Run the commands from inside the repository you want to write up. The repo is discovered by
walking upwards, so any subdirectory works.

### `zabal init`

Initialize a project tracker with your ZABAL handle. This writes a small `.zabal.json` in the
project root; `preview` and `submit` pick your handle up from it automatically.

| Flag | Alias | Default | Description |
| --- | --- | --- | --- |
| `--username` | `-u` | required | Your ZABAL handle, e.g. `@username` |

```bash
zabal init --username @yourhandle
```

### `zabal preview`

Show what would be submitted, without writing anything to disk.

| Flag | Alias | Default | Description |
| --- | --- | --- | --- |
| `--days` | `-d` | `7` | Number of days of commit history to scan |

```bash
zabal preview --days 14
```

```
ZABAL preview: last 14 days
Handle: @yourhandle
------------------------------------------------------------
12 commits, 34 files touched, +910 / -120 lines, across 5 active days
------------------------------------------------------------

2026-08-08
  bcff658  Add zabal-cli scaffold                          +186/-0
```

### `zabal submit`

Generate the submission document from recent commits.

| Flag | Alias | Default | Description |
| --- | --- | --- | --- |
| `--days` | `-d` | `7` | Number of days of commit history to scan |
| `--format` | `-f` | `markdown` | Output format: `markdown` or `json` |
| `--output` | `-o` | `zabal-submission.<ext>` | Write to a specific path |

```bash
zabal submit --days 7 --format markdown
```

The Markdown document opens with a summary table (commits, files touched, lines added and
removed, active days, contributors), then lists what shipped grouped by day, then credits
contributors. The JSON format carries the same information plus the full per-commit records,
for feeding into other tooling.

## Project status

Week 1 MVP is in place: scanning, both output formats, the tracker, and an integration test
suite all work end to end. Verified on Rust 1.97.1 (`x86_64-pc-windows-gnu`): `cargo build`,
`cargo test` (11 passing), `cargo clippy` and `cargo fmt --check` are all clean.

| Module | Responsibility |
| --- | --- |
| `src/main.rs` | Argument parsing and command dispatch |
| `src/lib.rs` | Library surface, so the modules are testable |
| `src/parser.rs` | `scan_commits`: walks the repo with `git2`, collects commits and diff stats |
| `src/generator.rs` | `generate_submission`: renders commits as Markdown or JSON |
| `src/ui.rs` | Terminal output for `preview`, plus the `.zabal.json` tracker |
| `tests/integration.rs` | Builds throwaway repos with controlled commit times and asserts behavior |

## Tests

```bash
cargo test
```

The tests create temporary Git repositories with commits at fixed timestamps, so they exercise
the time window logic without depending on whatever history happens to exist locally.

## Dependencies

| Crate | Purpose |
| --- | --- |
| `clap` | Command line argument parsing (derive API) |
| `git2` | Reading commit history from the local repository |
| `chrono` | Date math for the `--days` window |
| `serde` / `serde_json` | JSON output format |

## License

MIT
