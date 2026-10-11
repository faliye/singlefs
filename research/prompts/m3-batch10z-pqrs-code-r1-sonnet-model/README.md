# m3-batch10z-pqrs-code-r1 正推腿的模型与日志

都建在冻结副本 `~/.local/share/singlefs/work/m3-batch10z-pqrs-code-r1/tree/`（被判文件 sha256 对 `research/prompts/m3-batch10z-pqrs-code-r1-snapshot/crates-sha256.txt` 336 行全 OK）的拷贝上，不进仓内 crates。拷贝与编译目录交回前已删，清单与大小在报告末尾。

## 目录

- `src/p10z_probe.rs.txt`：放进拷贝 `crates/singlefs-harness/tests/p10z_probe.rs`（起点是上一轮 `research/prompts/m3-batch10z-no-code-r1-sonnet-model/src/s2_probe.rs.txt`）。动作（`S2_ACTIONS` 或随机字母表 `S2_ALPHABET`）：`o` 覆盖写、`n` 建 inode、`s` 顺序写 2 个数据单元、`g` 顺序写把文件长一个单元、`h` 顺序写把文件长 `H_STEP` 个单元（`H_STEP` 默认 40）；大写 `W` `N` `S` `G` `H` 同上，但被「推满仍不够」拒时当场记 `NS[…]`、独立重算、显式推环、再写一次，一行输出；`P` 显式推环；`F` 准入抬 F；`U` 卸载；`X` 卸载再可写挂载；`K` 不卸载崩了再挂；`J` 往根环表注入 C 槽（`J_OFFSETS`、`J_SLOTS`、`J_OVERLAP`、`J_NOISO`）；`b` 回退。环境变量：`S2_SLOTS`（单元区槽数，默认 384）、`S2_PREFIXES`、`S2_MID`、`S2_RS`、`S2_FAULTS`、`S2_PERSIST`、`S2_NOCHECK`、`S2_CRASH_BEFORE_ROOT`、`P_FAIL_AT`、`P_CRASH`、`P_CRASH_STRIDE`、`TRACE`。输出行：`DIAG`（每次被拒、推环、再挂）、`PUDIAG`（保护集里有没挡的槽）、`CRASHDIAG`（推环里某个镜像不过）、`S2 label=…`（每段历史一行）。
- `src/core-debug-edits.diff`：拷贝上对 `allocator.rs` 的改动，只加一个 `pub fn debug_placements_the_isolation_still_protects`（读保护集）。行为不变，不进库。
- `src/control-early-release.diff.txt`：K2-1 阳性对照用的改动（推环入口一开始清掉 C 记着的槽的隔离位），只在编译副本里。
- `src/scan-gpu-main.rs.txt`、`src/scan-crash-main.rs.txt` 与各自的 `*-Cargo.toml.txt`：把三条 GPU 用例与崩溃放量那条的测试体从冻结副本源码机械抽出，`include_str!` 换成读文件（环境变量 `GPU_SRC`、`CRASH_SRC` 指向要判的源码），报 GREEN 或 RED。
- `src/shadow-a.rs.txt` 到 `shadow-e.rs.txt`、`src/k5-legal-card-pool-snippets.rs.txt`：rustc 单独编的小程序（名字遮蔽、合法写法）。
- `scripts/`：`piece.sh`（经内存包装起一件探针）、`piece-ctl.sh`（同，指向对照编译目录）、`repro.sh`（报告引的关键探针各跑一遍，日志落 `logs/final/`）、`mut.py`（在 `tree-m` 上套 25 个变异、编、跑入库用例文件）、`mut_names.py`（同，列全红的用例名，多一个 `accessorZero`）、`lib_accessor.sh`（访问器恒交 0 只跑库内单元测试）、`rows.py`（把改动全文里实现员的变异行逐行套到 `tree-m`）、`row24_ignored.py`（行 1524 加 `--include-ignored`）、`variants.py`、`variants_rows.py`（喂扫源码模型）、`summ2.py`、`tersum.py`、`summ_jx.py`、`count_reports.py`、`count_notwide.py`、`count_stuck_by_file.py`、`count_needs_vs_pushed.py`（日志汇总）。
- `logs/`：`final/`（报告直接引的日志，`summaries.txt`、`summaries-k3-2.txt` 是汇总）、`sweeps/`（随机历史批次，`a*` `b*` `c*` `d*` 是 K1/K3 的主批，`f*` 是顺序写把文件长大的批，`h*` `i*` `g*` 是其他槽数）、`mut-1.log` 到 `mut-5-*.log`（变异）、`rows-*.log`（实现员变异行）、`scan-variants.txt`。

## 重建

1. `rsync -a <冻结副本>/ tree-a/`；拷 `src/p10z_probe.rs.txt` 成 `tree-a/crates/singlefs-harness/tests/p10z_probe.rs`；`patch -p0` 或手工按 `src/core-debug-edits.diff` 改 `tree-a/crates/singlefs-core/src/allocator.rs`。
2. `cd tree-a && CARGO_TARGET_DIR=<目录> bash scripts/run-with-memory-cap.sh 12G bash scripts/capped.sh 15 cargo test --release --offline -p singlefs-harness --test p10z_probe --no-run`。
3. `scripts/*.sh|py` 里的 `W=`、`$HOME/.local/share/singlefs/work/m3-batch10z-pqrs-code-r1-sonnet` 要改成自己的草稿目录；`bash scripts/repro.sh` 重跑 `logs/final/`。
4. 变异：`rsync -a <冻结副本>/ tree-m/`，`python3 scripts/mut.py <变异名…>`（名字见脚本里的 `mut('名字', …)`）。
5. 扫源码模型：`src/scan-*-Cargo.toml.txt` 改名 `Cargo.toml`、`src/scan-*-main.rs.txt` 改名 `src/main.rs`，`cargo build --release --offline`。
