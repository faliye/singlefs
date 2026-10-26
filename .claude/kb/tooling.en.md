# Tooling and Environment Facts

[中文](tooling.md) · **English** · [日本語](tooling.ja.md)

**tooling.md records "what the current state of tools used in this project is, and how to use them."**
Rules do not go into tooling.md; they belong in [CLAUDE.md](../../CLAUDE.md) or `.claude/rules/`.

## Local LLM (Not Dispatched as a Leg in Three-Way Argumentation)

Three-way argumentation only dispatches two cloud legs plus the main agent verdict; local model legs are not dispatched. The repo's gateway-calling scripts and word corruption checkers (`ask-local.sh`, `ask-local-selftest.sh`, `local-repro.sh`, `local-repro-tuned.sh`, `corruption-check.py`, `corruption-summary.py`, `oov-check.py`, `en-words.txt`) were removed on 2026-10-11. If needed again, use `git log --diff-filter=D -- scripts/agents/ask-local.sh` to find the commit that removed them, and retrieve them from the commit prior to that.

| Item | Value |
|---|---|
| Entry point | The local model gateway on this machine (OpenAI-compatible; its address and directory stay out of the repository) |
| Authentication | Request header `X-Api-Key`; the value is in private configuration on this machine and stays out of the repository |
| Model | `local` → `Qwen3-Next-80B-A3B-Thinking-AWQ-4bit`, max_model_len 262144 |
| Invocation script | No longer in repo (retrieval method in opening paragraph of this section) |
| Budget | Do not pass "max_tokens" or "thinking_token_budget" |

When using it:

- Prompts must always be in English, without markdown emphasis (`**bold**:` ); do not rely on `presence_penalty` or `repetition_penalty` to fix Chinese repetition.
- Specific field names, struct names, and mechanism descriptions given by it are always treated as unverified clues; if verification fails, discard the entire item.
- Explicitly state and forbid the most common category errors in the prompt: "the same system across two media" must not be answered as "two different systems"; "width is determined elsewhere" must explicitly state "what is deferred is the width, not the field itself."
- When it answers "cannot give an example", run a positive control: using the exact same prompt, swap only the queried subject for one whose answer is known to be "exists"; only if an example can be given this time does the "cannot give an example" count as valid.

## Performance Figures in the Repo Hold Only on This Machine; Citations Must Include the Machine Measurement Basis

**Scope**: Wall-clock time, throughput, milliseconds per publish, gate duration, cold mount seconds, parallel speedup ratio—anything involving time and rate.

**How to do it**:

- When citing a performance figure, include the **machine measurement basis** (which machine, what disk, what load, idle or busy); figures without a measurement basis must not enter judgment criteria.
- When using local numbers to establish **thresholds, upper bounds, or timeouts**, explicitly state "this is a local value; it must be remeasured on other machines."
- When a figure that holds across machines is needed, either measure on two or more machines, or phrase it conditionally ("calculating according to X yields Y; that tier cannot be measured locally and remains a hypothesis").

The full ring zeroing time in the gate tier, the cold restart recovery time, the wall-clock time for two streams in crash-point replay, E152 (Per-Milestone File Performance Against Six File Systems) and all comparison numbers with the six file systems are all local machine numbers (C486 (55 seconds of full ring zero-fill and 17 seconds of cold reopen unmonitored)).

## Calibrating Timeout Floors on an Idle Machine Mistakes "Machine Busy with Other Work" for "Mutation Not Caught"

- Concurrency multiplied by the number of cores occupied by a single unit under test must not exceed core count. cargo-mutants runs on this repo using `--jobs 1`.
- Timeout floors must be calibrated against a loaded machine; if unable to calibrate accurately, do not set a timeout.
- Entries hitting the timeout are listed separately and not merged into caught, unred, or invalid columns; when reporting counts, report all four numbers.
- Floor values must carry the machine measurement basis (see section "Performance Figures in the Repo Hold Only on This Machine; Citations Must Include the Machine Measurement Basis").

## Mutation Counts Drift with Source Code; Never Pin Them Down in the KB

Write the commands to count them in the body text rather than just writing the count:

```
cargo mutants --list --package singlefs-checker | wc -l
cargo mutants --list --package singlefs-core | wc -l
```

To cite a specific count, phrase it as an event sentence together with the measurement date ("measured N entries on <date>"), not as a present-tense status sentence.

## When Modifying a Reference, Grep the Entire Repo to Update All References

After correcting a phrasing, run `grep -rn` across the entire repo before claiming "all modified." The scope includes all subdirectories of `kb/`, `research/prompts/` (background material fed to three-way argumentation), source code comments under `scripts/` and `research/`, `records/`, and gate scripts. If unable to count completely, state "modified these few places, have not audited the full repo."

## A Number Must Never Be Computed from a Configuration That Itself Judges as Illegal

- Before calculating an upper or lower bound, first confirm that the configuration is legal under this project's judgment.
- When withdrawing a figure, write the invalidation reason as the actual one (missing input, illegal configuration, wrong measurement basis), not as a property that other retained configurations also share.

## Unsettled Items Must Always Be Numbered Entries; Headings Must Always Declare Item Count

Scripts extracting by `^\d+\.` only recognize numbered entries; the item count check in the kb-shape slot of `.claude/gate.d/doc-decisions.sh` only triggers when the heading declares "N undecided items".

## Where Verification Artifacts Go: Four Locations, Each Managing Its Own Stage

| Location | What goes here | Who reads it | Committed to repo |
|---|---|---|---|
| `.claude/kb/experiments.md` | Conclusions, measurement basis, criteria, reproduction commands, **decisive lines of figures** | Model retrieval | Yes |
| `records/` | Narrative of each round: what was settled, what was overturned, rationale | Human read-through | Yes |
| `scripts/` | Experiment code and scripts | Person running it | Yes (`.gitignore` only excludes `target/` and rare oversized files) |
| `research/results/<experiment>-<date>.out` | Full raw output | Line-by-line comparison during next rerun | Yes; rounds earlier than latest are archived into git history upon next commit (gate doc-experiments archive-past-rounds slot) |

- Decisive figures must be pasted into the KB body.
- Each raw output must prove itself to be left by the run that produced it: the N in the closing line `E7RESULT name=done emitted=N` must equal the number of `E7RESULT` lines in the file; if mismatched, the entire round is voided. Check during archival:
  `for f in research/results/*.out; do echo "$(grep -c '^E7RESULT' $f) vs $(grep 'name=done' $f)"; done`

## Kernel Source Code: Complete Tree Available Locally, No Need for WebFetch

| Item | Value |
|---|---|
| Path | `~/kbuild/linux-om` (1.8 GB) |
| Version | **Linux 7.2.0** (`VERSION = 7` / `PATCHLEVEL = 2` / `SUBLEVEL = 0` in `Makefile`) |
| What is present | Full `.c`/`.h` text, including `fs/jbd2/`, `fs/xfs/`, `fs/btrfs/`, `fs/erofs/`, `fs/ext4/`, `fs/f2fs/`, `drivers/nvme/`, `drivers/md/`, `crypto/`, `Documentation/` |
| **What is NOT present** | **`fs/bcachefs/` is not in this tree** (see [decisions.md](decisions.md) D7 (Merging into Linux Mainline)). `/usr/src/linux-headers-6.17.0-1028-oem/fs/bcachefs/` has only `Kconfig` + `Makefile`, **no source code** |

- Inspect source code using `~/kbuild/linux-om`, not `/usr/src/linux-*-headers-*` (which only contains `Kconfig` and `Makefile`).
- Tag the specific version number for whichever tree is cited; do not mix conclusions from two versions. Citations in the KB marked "fresh lookup Linux 6.17" come from the GitHub `v6.17` tag; this machine does not have that version.
- OpenZFS has no tree on this machine; fetch fresh from `https://raw.githubusercontent.com/openzfs/zfs/master/...`; record the retrieval date when citing, as line numbers are not permanent.

## Triggering Work on Schedule: Register with remind.sh

When a user needs a session to start work automatically at a scheduled time, only use `bash scripts/agents/remind.sh start --every <minutes> --message <reminder content> --once` (spawned via Bash `run_in_background`, exits at the scheduled time to wake up the session; rule `.claude/rules/timer-only-remind.md`, syntax in `.claude/skills/remind/SKILL.md`); do not use CronCreate, and do not write `sleep` in Bash solely for scheduling. In the reminder message, specify the path containing the instructions; the first step uses `set -o noclobber` to exclusively create an "already started" flag; if the flag already exists, do nothing (`remind.sh` may send duplicate reminders upon restart).

- Reminders only live in the current session; inform the user that the session must remain open and the machine cannot sleep.

## Rust Toolchain

| Item | Value |
|---|---|
| Version | rustc 1.98.0 / cargo 1.98.0 |
| Installation method | rustup, `--profile minimal --default-toolchain stable`, installed in host `~/.cargo` |
| Extra targets | `x86_64-unknown-linux-musl`, experiment binaries (packages under `scripts/experiment-apparatus/`) compiled static to run in VM's busybox initramfs; checker-tier real device binary `new_pool_file_creation_on_device` does not use it (checker tier compiles RocksDB C++ by default, host lacks musl C/C++ compiler), instead uses `--target x86_64-unknown-linux-gnu` with `+crt-static` to build static-pie, command in `.claude/gate.d/checker-tier-qemu-device-streams.sh` |
| Components | `rustfmt` + `clippy` (`--profile minimal` excludes them, install separately) |
| **PATH** | rustup shims symlinked in the per-user executable directory on PATH (cargo / rustc / rustup / rustdoc / rustfmt / cargo-fmt / cargo-clippy); gates and scripts launched via `bash -c`, do not read the shell startup files, PATH not dependent on editing dotfiles |
| Release overflow checks | Root `Cargo.toml` and `scripts/experiment-apparatus/Cargo.toml` `[profile.release]` both set `overflow-checks = true`; profile only recognized at workspace root, writing in member manifests has no effect |

## RocksDB and snappy Prebuilt Libraries: Built Once per Host, Values Passed Only Through the Process Environment

The C++ of librocksdb-sys (`librocksdb.a` and `libsnappy.a`) is built only once per host, and every build directory links to this one copy through environment variables instead of building it again in each build directory. The tool is `scripts/hosts/rocksdb-prebuilt.py`; its usage is written in the file header (it does not recognize `--help` and exits with 2). Run every subcommand at the repo root; without `--root R` it means the repo the script lives in.

| Subcommand | What it does |
|---|---|
| `key` | Prints this host's current key (its components) and the prebuilt library directory; builds nothing |
| `env` | When the prebuilt library exists and matches, prints four `export` lines (for `eval`); when it does not, exits with 1 and prints nothing to standard output |
| `build` | Builds the prebuilt library once if it is absent, under a lock: when several processes arrive together only one builds, and the later ones wait for the lock, see that it is built and exit with 0; it starts a real cargo and is heavy work, started through `bash scripts/hosts/capped.sh <N>` |
| `install` | Runs `build`, then writes the three environment sources of this host, changes the current environment of the user-level systemd manager, and finally runs `check`; also heavy work |
| `check` | Judges this host: the dependencies of the two workspaces (`crates/singlefs-checker-tier/Cargo.toml` and `scripts/experiment-apparatus/Cargo.toml`) match, the prebuilt library exists and matches, the values in the three environment sources and in the current process environment all agree with the prebuilt library, and the cargo configuration does not set these four variables |
| `audit-target <build directory>…` | Judges build directories: whether the values the librocksdb-sys build script received that time are this host's prebuilt library, whether `out/` holds any C++ built on the spot, and how many different values were seen under the same key; exits with 77 (not judged this time) when it finds no build script record at all |
| `entry [--cargo C]` | The light judgment called at the start of every entry that starts cargo (who calls it and what to do when values are missing are written in the paragraph "Entries that start cargo") |

**Where the prebuilt library lives**: `${SINGLEFS_ROCKSDB_PREBUILT_STORE:-~/.local/share/singlefs/rocksdb-prebuilt}/librocksdb-sys-<version>/<first 16 characters of the key>/lib/{librocksdb.a,libsnappy.a}`; `manifest.json` in the same directory records the components of the key and the sha256 of the two libraries, and `complete` is written last (only with it is the build counted as done); it is not placed in a directory that is cleared at boot. `SINGLEFS_ROCKSDB_PREBUILT_STORE` is used only by the self-test to switch to a temporary directory. The key is made of the spelling of the rocksdb dependency in the two workspaces, the versions and checksums of the related packages in the two `Cargo.lock` files, `rustc -vV`, the real paths and versions of c++ / cc / ar, the instruction sets handed to C++, and the way of building; when any of them changes, the key changes and a rebuild is needed.

**Where the values come from**: the values of the four variables `ROCKSDB_LIB_DIR`, `ROCKSDB_STATIC`, `SNAPPY_LIB_DIR` and `SNAPPY_STATIC` are written in three environment sources, all written by `install`:

| Location | Role |
|---|---|
| `~/.config/singlefs/rocksdb-prebuilt.env` | The only source of values: four `export` lines plus `unset ROCKSDB_COMPILE` and `unset SNAPPY_COMPILE`; do not edit by hand, rerun `install` after any change |
| The first block of `~/.bashrc` (three lines, from `# >>> singlefs rocksdb-prebuilt >>>` to `# <<< singlefs rocksdb-prebuilt <<<`) | Sources that env file; it sits before the interactive check, so interactive shells, sessions started from them and non-interactive commands started over ssh all read it |
| `~/.config/environment.d/60-singlefs-rocksdb-prebuilt.conf` | Read by units that the user-level systemd manager starts (long jobs started by `scripts/hosts/detached-run.sh`); `install` also runs `systemctl --user set-environment` so the manager that is running now takes the values at once |

**The four variables go only through the process environment**: do not write them into the `[env]` of `.cargo/config.toml` or of any other layer of cargo configuration (`check` judges red when it finds them there; the reason is in the comment at the top of `.cargo/config.toml`). Never set `ROCKSDB_COMPILE` or `SNAPPY_COMPILE`: setting them to 1 builds the C++ on the spot in the build directory.

**Entries that start cargo**: the gate stages, the crash and mutation drivers, `merge-and-verify.sh`, the commands handed over by `admission.py` and the second host's side all call `entry` first. It judges only three things: this host has the library installed (the prebuilt library exists and matches, without computing the sha256 of the two libraries), the four values in the current process environment agree with the prebuilt library, and neither `ROCKSDB_COMPILE` nor `SNAPPY_COMPILE` is set; it does not judge the three environment sources (that is the job of `check`). When a value is missing or does not match, the entry refuses on the spot, starts no cargo at all and writes out the way out for each failed item. The way out: when the current process has no values, open a new session (a login shell started after `install` reads the first block of `~/.bashrc`), or put `eval "$(python3 scripts/hosts/rocksdb-prebuilt.py env)"` in front of the command; on a host where it is not installed yet, first run `bash scripts/hosts/capped.sh <N> python3 scripts/hosts/rocksdb-prebuilt.py install`. When the tree to be built has no workspace that depends on rocksdb (librocksdb-sys cannot be built; the small repos of gate samples and self-tests are mostly like this), `entry` does not judge and exits with 0.

## VM Benchmark Harness (Used by E7 (Offline Index Harness))

| Item | Value |
|---|---|
| Script | `bash scripts/devices/vm-bench.sh <static-binary>` / `--selftest` |
| What it is | Project's own VM apparatus: attaches virtio disk to VM (sparse file in `TMPDIR`) and bundles binary into initramfs |
| Integrity gate | Benchmark program's closing line reports `name=done emitted=N`; count caught by host must equal N, otherwise entire round voided |
| Default spec | 2048M memory / 4 vCPUs / 4096M disk / 900s timeout, all overridable via environment variables |

- Capturing result lines does not anchor to line start (console first line has BIOS escape sequences before it); count is verified via the integrity gate.
- Sequential write throughput of raw devices inside VM is unstable; it is not used as input for any judgment.
- When batch run scripts void a round, record the tail of that round's output as well.

---

## Constructing Devices with Different Geometries: No Real Hardware Required

This machine has all the means to construct heterogeneous device pools; [decisions.md](decisions.md) D12 (Target Media) settled item 3 "where device-level geometry belongs, whether heterogeneous pools are supported" can be physically verified.

### QEMU Route (Recommended: No root needed, directly hooks into existing harness)

`zoned.zone_size` and `logical_block_size` are attached to **`nvme-ns` (namespaces)**, not to controllers; multiple namespaces under the same controller can have different geometries.

Physically tested and verified (QEMU 8.2.2, exit code 0, all three devices successfully constructed):

```
-device nvme,serial=A,id=nv0
-device nvme-ns,drive=d1,bus=nv0,nsid=1,zoned=true,zoned.zone_size=64M,logical_block_size=4096,physical_block_size=4096
-device nvme-ns,drive=d2,bus=nv0,nsid=2,zoned=true,zoned.zone_size=128M,logical_block_size=4096,physical_block_size=4096
-device nvme-ns,drive=d3,bus=nv0,nsid=3,logical_block_size=512,physical_block_size=512
```

A single command simultaneously constructs: **heterogeneous zoned pool** (64M vs 128M zones), **512e/4Kn mixed use**, and **zoned and regular mixed use**.

- `logical_block_size` must be paired with `physical_block_size`; providing only the former is rejected: `logical_block_size > physical_block_size not supported`.
- Whether guest kernels see three different geometries has not been verified: requires entering guest to read `/sys/block/*/queue/{logical_block_size,chunk_sectors}` and `blkzone report`.

### Host-Side Route (Requires root, advantage is not needing to start a VM)

| Method | What to construct | Key parameters |
|---|---|---|
| null_blk | Zoned device, memory-backed | `zoned=1 zone_size=<MB> zone_capacity zone_nr_conv zone_max_open`. Module parameters are global; **per-device geometry must go through configfs** (`/sys/kernel/config` mounted on this machine) |
| zloop | **File-backed** zoned device | Module at `drivers/block/zloop.ko.zst`, present on this machine |
| scsi_debug | Arbitrary sector size + ZBC | `sector_size=<bytes> zbc=host-managed zone_cap_mb zone_nr_conv` |

Relevant configs for running kernel 6.17.0-1028-oem: `CONFIG_BLK_DEV_ZONED=y`,
`CONFIG_BLK_DEV_NULL_BLK=m`, `CONFIG_BLK_DEV_ZONED_LOOP=m`, `CONFIG_SCSI_DEBUG=m`.

⚠️ The only real block device on this machine is `nvme0n1` (3.6T, 512/512, non-rotational)—
**no real rotational disks, no real 4Kn disks, no real zoned disks**; all related figures can only be simulated, and citations must explicitly include this measurement basis.

## Outcome Classification of scripts/mutation/mutate.sh

- Outcomes: `✅ [caught]` (turned red) / `❌ [not red]` (test blind spot) / `⏭ mutation caused compilation failure (invalid mutation, excluded from blind spot and not counted as hit)`; additionally `⏱ [timeout]`, `🧱 [out of memory]`, packaging un-queueable, squeezed out by total memory limit, insufficient VRAM, and anchor invalidation (`⚓ [corrupted anchor]`; lines where original text match is not exactly once are skipped and not entered into verdict library, while other lines run normally, entire round exits 3), each listed separately (all outcomes see `scripts/mutation/mutate.sh` file header and `--selftest`).
- When changing judgment tools, regress against the full corpus with known answers; do not only run the newly added items.

## How to Stage and Commit When Multiple Sessions Share One Repository

When the gate turns red, first determine whose changes caused the failure before acting; never edit someone else's in-flight files just to turn the gate green. Rules in upstream `session-wrapup.md` section 4; this repo's practices:

| Task | How to do it |
|---|---|
| Stage only own hunks | `scripts/gate/stage-mine.py --match my_marker --foreign others_marker file...`, first `--dry-run` to see which hunks it picked |
| Index / Staging area is also shared | Before staging, verify `git diff --cached --name-only` is empty; after staging, verify list and per-file cached diff before committing, then run pathless `git commit` |
| Do NOT use `git commit -- <path>` | When paths are given, git commits their working-tree content, not the index / staging area |
| Both sides edited the same line | When stage-mine reports "hunk entering staging area contains line matching --foreign" and single line cannot be split: if the other party only made reversible substitutions (renames) in that file, apply reverse substitution to the other party's edits in working copy, diff against `git show HEAD:file` to verify only own edits remain, then put into staging area via `git hash-object -w` + `git update-index --cacheinfo 100644,<sha>,<path>`, leaving working copy untouched |
| Commit on a worktree | On worktree (at HEAD), replay all changes to commit via script → compare file-by-file against main working directory (differences should belong entirely to the other party; use `git -c core.quotePath=false` during comparison, otherwise non-ASCII paths are escaped and comparison tools report all different) → run entire gate in worktree and commit → in main repo run `git update-ref refs/heads/master <new_commit> <old_HEAD>` then `git read-tree <new_commit>`, leaving working copy untouched and other party's changes intact |
| Inspect which changes are NOT part of this commit | `git -c core.quotepath=false status --short`; non-ASCII paths are escaped in octal by default, causing grep for Chinese keywords to miss matches |
| Changes from both sides intertwined in same file | Bundle all in a single commit, state clearly in commit message (splitting commits causes doc-registries experiment-refs slot and doc-experiments experiment-orphans slot to turn red) |

## Replay and Mutation Scripts Must Be Run Under scripts/experiment-apparatus/

- `scripts/mutation/mutate.sh` runs under `scripts/experiment-apparatus/`: `cd scripts/experiment-apparatus && bash ../mutation/mutate.sh <bin-name> <source-file> ../mutation/tables/<mutation table>`.
- `replay.sh` only recognizes experiment numbers with E (`E` followed by digits); bare digits exit with code 2.
- Each `replay.sh` run builds into its own build directory at `${TMPDIR:-/tmp}/singlefs-replay-build-<pid>-XXXXXX/target/` and deletes it afterwards (if the caller sets `CARGO_TARGET_DIR`, it builds there and does not delete it); research binaries start from `$REPLAY_RELEASE_DIRECTORY` (`$CARGO_TARGET_DIR/release/`); `scripts/experiment-apparatus/target/` no longer holds binaries built by replays.

## Gate Failure Patterns When Batch Creating / Finalizing Sub-Items

After batch creating or settling a batch of sub-items, several slots in the gate will turn red in the following patterns prior to commit; when drafting, follow the "Prescribed Format" column:

| Gate | Failure Pattern | Prescribed Format |
|---|---|---|
| first-txn-hooks slot in doc-registries (linking three files) | Splits unsettled item entries by `\| N \|` line; last row swallows all subsequent `#### Undecided item N` body text into its own entry; if body text says "Changes new-pool file creation bytes: Yes (", the last row judged "No" is also treated as "Yes" | Use canonical form for index row: `Changes new-pool file creation bytes: Yes (date, basis: ...)` and format body subsection as `**Changes new-pool file creation bytes**: Yes (...)` |
| item-ref-status slot in doc-decisions (reference attribution) | In-line bare "Settled item k" is attributed to nearest preceding `D<n> (short name)` on the same line | After adding sub-items, run item-ref-status slot of doc-decisions to restore decision names to flipped bare references; when referencing sub-items of other decisions within decision text, repeat own decision name on same line |
| doc-lint | Review numbers like F followed by digits are treated as unregistered numbers; appending comma annotations after short name triggers "name mismatch" | Write "Finding 1", "Proposal 2"; start separate parentheses for notes, written as "Number (short name) (clause drafted)", do not cram into short name parentheses |
| table-shape slot in doc-registries (owed check table shape) | New 6-column owed check row appended to end of file, which is where 4-column "cleared" table lives; an entry in pending table starts with "Cleared (...)" or "Written off"; or blank line inserted inside pending table | Append after last row of first table: before first 5-bar `\|---\|` separator, after last row starting with `\| C` having ≥ 7 vertical bars; when fully cleared, move to cleared table with 4 columns; for partially cleared items, prefix with "Half cleared", "Decision settled, checks still owed", "Prerequisite cleared"; no blank lines inside table |
| segment-registry slot in doc-registries (segment sequence registry) | `--selftest` requires segment sequence string for "mkfs root seeding" row to appear exactly once across entire `layout/01-first-txn.md`; duplicate occurrence elsewhere turns red | Write other references with spaces as "4 + 1 + 1 + 1 + 4" |
| Three changelog slots in doc-decisions gate (entry-added, shape, history-loss) | "(Part N)", quick-reference two lines, and section-top status quo line | Format and validation rules in `.claude/rules/changelog-format.md`; whether counts and numbers are lost is judged by history-loss slot; run `scripts/kb/history-consolidate.py loss-check` before staging changelog consolidation |
| format-constants slot in code-source-discipline (format constants) | After changing value in KB tag, every same-named `const` in `scripts/experiment-apparatus/**/*.rs` and `crates/**/*.rs` must follow (`research/` contains only frozen evidence, scanned but exempted individually via `.claude/format-constants-exclude`); old value string in `stale=` column must never appear in KB body | Do not add strings that still legitimately appear elsewhere (`base = 95`) into `stale=` |
| clippy shadow_unrelated | Second `let reader = ...` / `let segments = ...` in same test function turns red; regex replacement after rustfmt reformatting misses second half | Give each reader / segment / report distinct name (`extent_reader`, `warm_up_segments`, `control_report`) |
| naming-lint | "Letter + digit" treated as single letter: read_u48 turns red, test name starting with a_ turns red; scans all .rs in repo, including model source code written by cloud legs in `research/prompts` | Write `read_six_byte_unsigned`; `research/prompts` in `.claude/naming-lint-exclude` (evidence directory, not recycled); leg models written to other directories scanned as usual |
| checker-tier-crash-point-replay (crash-point replay full run) | Test cases marked `#[ignore]` living in `crates/singlefs-checker-tier/tests/` (settled item 15 of D13 (Verification Route)); `cargo test -p singlefs-checker-tier` reporting several ignored tests is expected; harness tier `cargo test -p singlefs-harness` cannot reach them | `--full` flag in gate runs each test in release mode against reuse verdict (the quick tier does not run the full run); when full run is in progress, changing `CARGO_TARGET_DIR` allows compiling other tests concurrently |
| Mutation proof | Multi-line call matching entire string "not hit" after rustfmt | Locate line containing key string, delete through line ending in `);`; restore after modification and `assert` file byte-for-byte identical to original |
| Checker new verdict | Verdict that only triggers after object passed same check will never turn red | Pair each new verdict with a known bad image (pattern of `checker_known_bad_images.rs`), never rely solely on clean images passing green |
| Batch targeted replacement | Writing disk item-by-item leaves half-baked changes when one anchor fails to match | Use `scripts/kb/replace-batch.py`: verify all replacements match exactly 1 time in memory before writing to disk |

## grep and find in This Machine's Claude Code Sessions Are Not GNU Versions

`type grep` and `type find` show both are shell functions: `grep` is backed by ugrep, and `find` is backed by bfs (errors begin with `ugrep: error` and `bfs: error` respectively). Two behaviors differ from GNU:

| Invocation | Result | Change to |
|---|---|---|
| `grep -o` with bounded repetition like `.\{0,90\}` on non-ASCII/multibyte lines | `exceeds complexity limits`, exit code 2, outputs nothing | Use Python index slicing: `line[max(0, i - 90):i + 90]` |
| `find <dir> -newermt '-40 minutes'` | `Invalid timestamp`, relative time not accepted | Use ISO date: `-newermt '<YYYY-MM-DD>'` |

Both failure modes appear as "command produced no output"; do not interpret this as "no matches found" when chained in pipes or after `||`.
