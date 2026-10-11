# m3-batch10z-tu-code-r1 正推腿的模型与日志

都建在冻结副本 `~/.local/share/singlefs/work/m3-batch10z-tu-code-r1/tree/`（被判文件 sha256 对 `research/prompts/m3-batch10z-tu-code-r1-snapshot/crates-sha256.txt` 338 行全 OK）的拷贝上，不进仓内 crates。拷贝与编译目录交回前已删，清单与大小在报告末尾。上一轮的模型在 `research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/`，本轮的探针由它改来。

## 目录

- `src/p10z_probe.rs.txt`：放进拷贝 `crates/singlefs-harness/tests/p10z_probe.rs`。动作字母同上一轮（`o n s g h` 小写不推环，`W N S G H` 大写被拒时当场读报告、真推环、再试，`P` 显式推环，`F` 抬 F，`X` 卸载再挂，`K` 崩了再挂，`J` 往根环表注入 C 槽，`b` 回退），本轮加：`D` / `d` unlink 最近建的 inode（大写被拒时推环再试）、`E` unlink 第一个文件、`T` 直接调预演并核会话 / 盘不动（`T_KIND=walk|change`、`T_UNITS`）、`Z` 读错注入（没造成）；环境变量 `WAY1=1`（被拒后不推环、卸载再挂再试）、`S2_PERSIST`、`P_FAIL_AT`、`J_OFFSETS`、`J_SLOTS`、`J_OVERLAP`、`H_STEP`、`S2_SLOTS`。每条「被宽护卡住」的输出带 `reh=`（A 放得下 / R 推完仍被拒 / P 推环被拒）、`rel=`（预演报的放出来数）、`CMP[…]`（预演对真推环再试的对不上处，空 = 对得上）。
- `src/core-allocator-debug.diff`、`src/core-mounted-session-debug.diff`：对冻结副本的拷贝所做的改动（只给探针和差分测试用，不进库）：`allocator.rs` 放出一个读保护集的访问器；`mounted_session.rs` 把 `RemainderOfTheUserChange` 与预演函数放成 `pub`、预演里加计时 / 计块数的 `eprintln!`、末尾加差分测试模块 `sonnet_k5_differential`。
- `src/atk_part5.rs.txt`、`src/atk_part-patch.diff`：攻方上一轮原型（`research/prompts/m3-batch10z-pqrs-code-r1-opus-model/`）的两处补丁——把 `Stuck` 改成 `Stuck:A/R:<预演拒的停因>`；`atk_part5` 是我加的两条（两槽 C、被抛弃根加 C 上长写被卡），拼法见 `scripts/assemble_atk.sh`。
- `src/scan-gpu-main.rs.txt`、`src/scan-crash-main.rs.txt`（加 `scan-*-Cargo.toml.txt`）：从冻结副本源码机械抽出的两条扫源码判法，读 `GPU_SRC` / `CRASH_SRC` 指的源码，可再读变异清单（`VARIANTS`、`VARIANT_DIR`、`DEPTH_CASES`）。`src/gpu-variants*.tsv`、`scripts/gen_crash_variants.py`、`src/depth-cases.txt`、`src/depth-expect.json` 是喂它们的变体。
- `src/replay-gpu_model.rs.txt`、`src/replay-crash_model.rs.txt`：把判法连同第二十一块的真用例体抽成可编的小程序（`include_str!` 换成读环境变量指的文件）；`scripts/replay_scan.py` 套改动全文的变异行、`scripts/replay_mut.py` 套我造的变异。
- `src/cfg_shadow.rs`、`cfg_stop.rs`、`struct_std.rs`、`type_wgpu2.rs`、`type_std2.rs`：K4 的 rustc 小程序（`logs/final/rustc-shadow.txt` 是输出）。
- `scripts/`：`piece.sh` / `par4.sh` 起探针（一件 3G、线程 1，经内存包装）、`sum_new.py` / `jk_holds.sh` / `stuck_by_fault.sh` / `notwide_hold.sh` / `son_check.sh` / `depth_check.py` 汇总日志、`rows.py`（改动全文变异行套拷贝）、`mut2.py` / `mut3.py` / `mut3_body.py`（变异驱动）、`probe_mut.sh` / `piece-pm.sh` / `par4pm.sh`（把一个变异编进探针跑）。脚本里的草稿目录路径要换成自己的。
- `logs/final/`：报告引的日志；`logs/rows/`：改动全文变异行重跑；`logs/mut/`：变异日志。

## 重建

1. `rsync -a <冻结副本>/ tree-a/`；拷 `src/p10z_probe.rs.txt` 成 `tree-a/crates/singlefs-harness/tests/p10z_probe.rs`；按两份 `*-debug.diff` 手工改 `tree-a` 的 `allocator.rs`、`mounted_session.rs`。
2. `cd tree-a && CARGO_TARGET_DIR=<目录> bash scripts/run-with-memory-cap.sh 12G bash scripts/capped.sh 15 cargo test --release --offline -p singlefs-harness --test p10z_probe --no-run`。
3. `bash scripts/par4.sh < 清单` 跑探针（清单每行 `日志名 环境变量…`）。
4. 扫源码模型：`src/scan-*-Cargo.toml.txt` 改名 `Cargo.toml`、`src/scan-*-main.rs.txt` 改名 `src/main.rs`，`cargo build --release --offline`；`replay-*.rs.txt` 直接 `rustc --edition 2021 -O`。
