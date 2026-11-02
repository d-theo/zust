# Zust : the improved z.sh

`zust` lets you jump to any directory from anywhere in the terminal by typing just
its name, or part of its name. It is a Rust rewrite of
[z.sh](https://github.com/rupa/z) for Linux / WSL, with a small selection TUI.

```console
$ z proj        # Enter: cd to the best directory containing "proj"
$ z proj<Tab>   # Tab: show the 10 best matches and pick one
    1  /home/hane/projets
    2  /home/hane/work/proj-api
    3  /tmp/proj-test
```

Ranking is based on **frecency**: a mix of access frequency (how many times you
visited the directory) and recency (how long ago the last visit was).

## Installation

### Prebuilt binary (recommended)

Static binaries for Linux / WSL (no dependencies, any distribution) are
published on the [Releases](https://github.com/d-theo/zust/releases) page:
`zust-x86_64-linux` (most PCs, WSL) and `zust-aarch64-linux` (ARM).

```sh
mkdir -p ~/.local/bin
curl -fL -o ~/.local/bin/zust \
  https://github.com/d-theo/zust/releases/latest/download/zust-x86_64-linux
chmod +x ~/.local/bin/zust
```

`~/.local/bin` must be on your `PATH`. Each binary has a `.sha256` file next to it
for verification.

### From source

Requirement: a Rust toolchain (`cargo`).

```sh
cargo install --path .
```

The binary is installed to `~/.cargo/bin/zust`; that directory must be on your `PATH`.

### Shell setup

Then add this at the end of `~/.bashrc`:

```sh
eval "$(zust init bash)"
```

and open a new terminal (or run `source ~/.bashrc`).

Only **bash** is supported for now (`zust init zsh` returns an error).

If `z.sh` is already installed, its `z` alias is replaced automatically. Its
history (`~/.z`) is not imported: the `zust` database starts empty and fills up
as you navigate.

## Usage

There is nothing to do to feed the database: every directory change (`cd`,
`pushd`, `popd`, `cd -`, or any other way) is recorded automatically when the
prompt is displayed.

| Command                  | Effect                                              |
|--------------------------|-----------------------------------------------------|
| `z`                      | `cd ~`                                              |
| `z -`                    | `cd -` (previous directory)                         |
| `z <existing directory>` | `cd` straight to that directory                     |
| `z word…` + Enter        | `cd` to the best matching directory                 |
| `z word…` + Tab          | open the TUI with the 10 best matches ([configurable](#configuration)) |

If no directory matches, `z` does nothing and returns exit code 1.

### Matching rules

- matching is **case-insensitive**;
- every word must appear in the path, **in the given order**;
- the **last word** must appear in the **last segment** of the path.

Examples with the directories `/work/api/src`, `/src/api/work`, `/home/proj` and `/home/proj/docs`:

| Query          | Matches             | Why                                                    |
|----------------|---------------------|--------------------------------------------------------|
| `z proj`       | `/home/proj`        | `/home/proj/docs` is excluded: `proj` is not in `docs` |
| `z work src`   | `/work/api/src`     | `/src/api/work` is excluded: words are out of order    |
| `z PROJ`       | `/home/proj`        | case-insensitive                                       |

Recorded directories that no longer exist are skipped and removed from the
database automatically.

### The TUI (Tab)

The TUI is drawn below the prompt, without clearing the screen.

| Key                     | Action                                  |
|-------------------------|-----------------------------------------|
| `↑` / `↓`, `Tab`        | move the selection (wraps around)       |
| `Enter`                 | go to the selected directory            |
| `1` … `9`               | go straight to choice 1…9               |
| `0`                     | go straight to the 10th choice          |
| `Esc`, `q`, `Ctrl-C`    | cancel                                  |

The choice takes effect immediately, with no need to press Enter a second time.
If there is only one candidate, the TUI is skipped and you go there directly.

Digit keys only reach the first 10 choices; use the arrows for the rest. The list
is cut to fit the terminal height.

## Ranking (frecency)

```
score = count × weight(time since last access)
```

| Last access         | Default weight |
|---------------------|----------------|
| less than an hour   | 4              |
| less than a day     | 2              |
| less than a week    | 0.5            |
| older               | 0.25           |

The score is computed on each search; it is not stored. The weights can be
changed with `ZUST_WEIGHTS` (see [Configuration](#configuration)).

**Aging**: when the sum of all `count` values exceeds 10,000, every `count` is
multiplied by 0.9 and entries whose `count` drops below 1 are removed. Directories
you stop visiting eventually disappear.

## Configuration

zust is configured with environment variables. Set them in `~/.bashrc`, before
or after the `eval` line:

```sh
export ZUST_TUI_CHOICES=20
export ZUST_WEIGHTS="8,2,0.5,0.1"
```

| Variable           | Default              | Meaning                                                    |
|--------------------|----------------------|------------------------------------------------------------|
| `ZUST_TUI_CHOICES` | `10`                 | number of choices shown by the TUI, from 1 to 99           |
| `ZUST_WEIGHTS`     | `4,2,0.5,0.25`       | frecency weights `hour,day,week,older`: 4 numbers ≥ 0       |
| `ZUST_DATA`        | `~/.zust`            | database file (see [Data](#data))                          |

Raising the `hour` weight favors directories you just visited; raising `older`
favors directories you visited often, even long ago. A weight of `0` ranks that
age bucket last.

An invalid value prints a warning and falls back to the default.

## Data

The database is a plain text file, `~/.zust` by default, one line per directory:

```
path;count;last_access
/home/hane;12;1778353961
```

- `count`: number of visits (may become fractional after aging);
- `last_access`: Unix timestamp in seconds.

Paths may contain `;` (lines are split from the right). Invalid lines are
ignored. Writes are atomic (temporary file, then `rename`).

The `ZUST_DATA` environment variable selects a different file:

```sh
export ZUST_DATA=~/.local/share/zust/db
```

To start over: `rm ~/.zust`.

## CLI reference

These subcommands are called by the integration script; you normally don't type
them yourself.

| Command                  | Purpose                                                          |
|--------------------------|------------------------------------------------------------------|
| `zust init bash`         | print the bash integration script                                |
| `zust add <path>`        | record a visit: `count += 1`, `last_access = now`                |
| `zust query <words…>`    | print the best path to stdout                                    |
| `zust tui <words…>`      | show the TUI on `/dev/tty`, then print the chosen path to stdout |

A `--` before the arguments is accepted (`zust add -- "$PWD"`).

Exit codes: `0` success, `1` no match / cancelled / error, `2` invalid usage.

## How it works

A program cannot change the current directory of the shell that launched it.
So `zust` only computes a path and prints it to stdout; the `z` shell function
does the `cd`.

The script generated by `zust init bash` installs:

- a `PROMPT_COMMAND` hook that calls `zust add "$PWD"` when the directory changed;
- the `z` function;
- Tab completion on `z` that runs `zust tui`. The TUI is drawn on `/dev/tty`
  because stdout is captured by the completion. After the choice, the script
  sends the terminal a status request (`\e[5n`); its reply (`\e[0n`) is bound to
  `accept-line`, which submits the `z <path>` line without an extra Enter.

Code layout:

| File               | Contents                                                   |
|--------------------|------------------------------------------------------------|
| `src/main.rs`      | CLI parsing, subcommands                                   |
| `src/config.rs`    | `Config`: settings read from environment variables         |
| `src/dir.rs`       | `Dir` (path, count, last access) and frecency computation  |
| `src/store.rs`     | `DirStore`: reading, writing and aging the database        |
| `src/selector.rs`  | `DirSelector`: word matching and frecency sorting          |
| `src/display.rs`   | selection TUI (crossterm) and result output                |
| `src/shell.rs`     | bash integration script                                    |

Design decisions are described (in French) in [`design/`](design/):
[initial need](design/initiative.md), [decisions](design/adr.md),
[architecture](design/archi.md).

## Development

```sh
cargo build
cargo test
```

The integration test `tests/tab_tui.rs` drives a real interactive bash in a
pseudo-terminal (via `tests/tab_tui.py`) to check that `z proj` + Tab + a choice
actually changes directory. It requires `python3` and `bash`.

### Releasing

1. Bump `version` in `Cargo.toml` (and run `cargo build` to update `Cargo.lock`), commit.
2. Tag and push: `git tag v0.2.0 && git push origin v0.2.0`.

The [release workflow](.github/workflows/release.yml) runs the tests, builds
static musl binaries for x86_64 and aarch64, and publishes them with their
checksums to a GitHub Release. It fails if the tag does not match the
`Cargo.toml` version.
