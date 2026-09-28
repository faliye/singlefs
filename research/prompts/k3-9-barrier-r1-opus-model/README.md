# k3-9-barrier-r1 云端攻方原型（W1）

副本上的原型，不是入库装置。冻结副本 `/tmp/claude-1000/k3-9-barrier-r1/tree/crates/`（sha256 清单 `research/prompts/k3-9-barrier-r1-snapshot/crates-sha256.txt`）。

## 复跑

1. 建副本：`mkdir -p R && cp Cargo.toml Cargo.lock R/ && mkdir -p R/.cargo && cp .cargo/config.toml R/.cargo/ && rsync -a /tmp/claude-1000/k3-9-barrier-r1/tree/crates R/`（在仓根跑）。
2. 打补丁：`patch -d R -p0` 不适用（diff 是绝对路径），按 `patches/transaction.rs.diff`（设 `K39_DROP_UNIT_RECORD_BARRIER` 时跳过第 1058 行那道屏障）、`patches/walk.rs.diff`（设 `K39_CHECKER_VERIFIES_NAMED` 时攻方自提的 checker 改法生效）、`patches/workspace-Cargo.toml.diff`（工作区加 `crates/k39-barrier-attack`）手工施加；把 `crate/` 拷成 `R/crates/k39-barrier-attack/`。
3. 编：`cd R && nice -n 19 bash <仓根>/research/scripts/capped.sh 10 cargo build --release --offline -p k39-barrier-attack`。
4. 跑：`scripts/run-arm.sh <control|mutated> <日志名> <原型参数…>`（草稿目录路径写死在脚本里，换目录要改第 5 行），外面套 `bash <仓根>/research/scripts/run-with-memory-cap.sh 16G bash <仓根>/research/scripts/capped.sh 10`。批次脚本 `scripts/batch1.sh`–`batch8.sh` 是这一轮实际跑的次序。
   - 最短复现（第一截、零故障、池级 checker I-3.10 红）：`K39_DROP_UNIT_RECORD_BARRIER=1 k39-barrier-attack stage1 floorreuse4 384 16 32`，看 `CHECKER-RED-FEW-WITHHELD` 与 `TALLY` 行；不设环境变量（带屏障）同一命令 I-3.10 为 0。
   - 第二、三截（崩溃镜像上可写挂载、挂载那一段层 0 全量展开）：`stage2 <场景> 384 16 <随机数> <第一截状态数> <none|remount|publish>`。

## 原型参数

`survey 384`；`stage1 <场景> 384 <全枚举段长上限> <随机掩码数>`；`full1 <场景> 384`（`enumerate_layer0_selecting_versions_observing_each_state`、每段都展开）；`stage2 …`；`faults <场景> 384 16 32`。
场景：`first` `over` `seq2` `seq3` `over_i2` `rollback` `overmany<K>`（小盘上第 20 次覆盖写被准入拒，没用上）`floorreuse<K>`。
