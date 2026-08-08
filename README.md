# zabal-cli

LoopHouse CLI. Auto-generate ZABAL hackathon submission documents straight from your Git commit history.

Instead of writing up what you built at the end of a hackathon, `zabal` reads the commits you already
made and turns them into a submission document.

## Install

Requires a Rust toolchain (`rustup`, which provides `cargo`).

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

### `zabal init`

Initialize a project tracker with your ZABAL handle.

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

### `zabal submit`

Generate the submission document from recent commits.

| Flag | Alias | Default | Description |
| --- | --- | --- | --- |
| `--days` | `-d` | `7` | Number of days of commit history to scan |
| `--format` | `-f` | `markdown` | Output format: `markdown` or `json` |

```bash
zabal submit --days 7 --format markdown
```

## Project status

This is the CLI scaffold. The argument surface is complete and parses correctly; the three
subcommands currently print a placeholder rather than doing the work.

Planned module layout:

| Module | Responsibility |
| --- | --- |
| `src/main.rs` | Argument parsing and command dispatch (done) |
| `src/parser.rs` | Walk the repo with `git2` and collect commits in the requested window |
| `src/generator.rs` | Render collected commits as Markdown or JSON |
| `src/ui.rs` | Terminal output for `preview` and `init` |

## Dependencies

| Crate | Purpose |
| --- | --- |
| `clap` | Command line argument parsing (derive API) |
| `git2` | Reading commit history from the local repository |
| `chrono` | Date math for the `--days` window |
| `serde` / `serde_json` | JSON output format |

## License

MIT
