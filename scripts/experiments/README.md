# scripts/experiments

实验装置的 crate：`Cargo.toml` 在这一层，一个实验一个或几个 `e<编号>_<简称>.rs` 目标，自带清单或多份源码的实验各占一层子目录。复跑入口是 `replay.sh`（在仓根跑，`bash scripts/experiments/replay.sh <实验号>`）。

- 编译与单测：`cargo test --manifest-path scripts/experiments/Cargo.toml`，经 `scripts/hosts/run-with-memory-cap.sh`；变异在这一层起：`cd scripts/experiments && bash ../mutation/mutate.sh <bin 名> <源文件> ../mutation/tables/<表>`。
- 每一份 `.rs` 都要是某个目标，或经 `#[path]`、`mod`、`include!` 被引到；没有着落的由门禁 code-source-discipline 的 crate-sources-reachable 格判红。
- 产物不放这里：写进 `research/results/`，编译目录 `target/` 不进 git。
- 计时实验（`replay.sh` 复跑表里判据写 `timing` 的行）的收尾行写成 `E7RESULT name=done emitted=N loadavg1=<1 分钟负载> cpus=<在线 CPU 数>`，两个字段装置从 `/proc/loadavg` 与 `available_parallelism` 取、不进 `emitted`；`loadavg1 / cpus` 超过 `REPLAY_TIMING_LOAD_RATIO`（默认 0.5）的那一行 `replay.sh` 记「本次不判」，没有这两个字段的判红，存量登记在 `.claude/gate.d/timing-load-field-lag.tsv`（只缩不涨）。约定全文在 `replay.sh` 文件头「计时行的负载关」。
- `replay.sh` 按行复用：每一行按装置源码、复跑表那一行、留存产物与工具链算一把键（`scripts/gate/admission.py replay-row-inputs`），判定库（`<git common-dir>/singlefs-unit-verdicts/replay-row/`）里键相同、判过的那一行不跑、记「复用」；`--full` 只记不读。留存产物已归档的行逐行报「已归档、只跑不比」。约定全文在 `replay.sh` 文件头「按行复用」。
