# m3-batch10z-lm-code-r1 正推腿的模型与日志

都建在冻结副本 `~/.local/share/singlefs/work/m3-batch10z-lm-code-r1/tree/` 的拷贝上，不进仓内 crates。拷贝与编译目录交回前已删，下面写怎么重建。

- `src/mount-triref-and-diff-arm.diff`：加在冻结副本 `crates/singlefs-core/src/mount.rs` 上的参照臂。`triref` 是把上一轮正推腿的 tri 臂（`research/prompts/m3-batch10z-jk-code-r1-sonnet-model/src/probe-arms-on-mount.diff`）逐支重写；`SINGLEFS_PROBE_ARM=diff` 同一次调用里先跑入库实现、再跑 `triref`，打 `TRIDIFF … equal=… extra_in_impl=… missing_in_impl=…`（并进 R 那一槽的落点集合差）与 `TRIDESC`（点名的打包单元读成什么）；`narrow` 是第十二块之前的窄取法。
- `src/s2_probe.rs.txt`：放进拷贝 `crates/singlefs-harness/tests/s2_probe.rs`。原件是实现员量法副本里的 s2 探针（上一轮攻方 s2 探针），我加了 `S2_ACTIONS`、故障 `rlin1`、`rlin2`、`qracct`、`qlin`、`qall`（差异 `src/s2_probe-additions.diff`）。`src/probe_jk.rs.txt` 是上一轮正推腿的探针原样。
- `src/lm-gpu-ledger-tests-T1-T5.diff`：加在拷贝 `crates/singlefs-harness/src/gpu_memory_ledger.rs` 测试模块里的五个用例（T1 poll 当口新记的没等成借据不销、T2 早记丢了加重起后旧设备仍占卡不多给、T3 一张卡没有设备不挡后面的卡、T4 丢了而没有没等成借据的卡、T5 随机操作串）。
- `scan-model/`：把第十三块 checker 档两条扫源码用例逐字搬出来（`src/main.rs.txt` 改名 `main.rs` 编），喂任意一份源文件，报 GREEN 或 RED。
- `scripts/`：`piece.sh`（经内存包装起一件探针）、`batch.sh`（并行起一批，每件写自己的 `.rc`）、`jobs-*.txt`（各批件表）、`gen1.py`、`patch-ref.py`（把参照臂加进拷贝的脚本）。
- `mut/`：`drv.py`（变异驱动，3 个工人各用自己的拷贝与编译目录）、`core-rows.json`、`gpu-rows.json`（实现员交的变异行，抄自 `research/prompts/_m3-batch10z-lm-code-r1-diff.md`）、`extra-core.py`、`gpu-h.json`（我加的变异行）、`scanmut.py`、`scan-false-red.py`、`chk.py`、`chk2.py`、`gpu-lm-tests.py`、`gpu-lm-t5.py`、`logs-*`（每个变异一份原始输出）。
- `logs/`：`mutation-summary.txt`（53 行变异汇总）、`tridiff-summary.txt`、`scanmut.txt`、`cargo-check-A2-A3-A7.txt`、`scan-false-red.txt`、`gpu-lm-tests-*.txt`、`gpu-lm-t5-on-mutants.txt`、两份基线。`runs/<批>/<件>.log`：每一件探针的原始输出。

重建：`rsync -a <冻结副本>/ tree/`；`python3 scripts/patch-ref.py`（改 `tree/crates/singlefs-core/src/mount.rs`，再手工把 `TRIDESC` 那一段按 `src/mount-triref-and-diff-arm.diff` 补上，或直接 `patch` 那份 diff）；拷 `src/s2_probe.rs.txt`、`src/probe_jk.rs.txt` 进 `crates/singlefs-harness/tests/`；`CARGO_TARGET_DIR=… bash scripts/capped.sh 15 cargo test --release --offline -p singlefs-harness --test s2_probe --test probe_jk --no-run`；`scripts/piece.sh` 里的二进制路径与工作目录要改成自己的。
