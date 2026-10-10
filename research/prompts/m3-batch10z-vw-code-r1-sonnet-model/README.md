# m3-batch10z-vw-code-r1 正推腿的模型与日志

都建在冻结副本 `~/.local/share/singlefs/work/m3-batch10z-vw-code-r1/tree/`（被判文件 sha256 对 `research/prompts/m3-batch10z-vw-code-r1-snapshot/crates-sha256.txt` 338 行全 OK）的拷贝上，不进仓内 crates。拷贝与编译目录交回前已删。上一轮的模型在 `research/prompts/m3-batch10z-tu-code-r1-sonnet-model/`，本轮的探针与脚本由它改来（路径换成本轮草稿目录 `~/.local/share/singlefs/work/m3-batch10z-vw-code-r1-sonnet/`）。

## 目录

- `src/p10z_probe.rs.txt`：随机历史探针（放进拷贝 `crates/singlefs-harness/tests/p10z_probe.rs`）。本轮改的一处：`describe_ns` 里报「被宽护卡住」之后另调 `session.ways_out_of_the_wide_protection(&pool.devices, s)` 取预演结局与出路；拷贝里 `mounted_session.rs` 要把 `RemainderOfTheUserChange` 与 `rehearse_the_push_then_the_remainder_of_the_change` 放成 `pub`（`T` 动作直接调预演），`allocator.rs` 加 `src/core-allocator-debug.diff` 的访问器。
- `src/atk-pqrs/`：上一轮攻方 pqrs 原型（`atk_part*.rs`）加我的 `atk_part5.rs`，把读 `ways_out()`、`pushing_the_root_ring` 的几处改成用新方法（`pool.session().ways_out_of_the_wide_protection(&pool.devices, s)`）。`src/atk-tu/`：上一轮攻方 tu 原型 8 份，`atk_tu_part1.rs` 里的垫片 `atk_register_ways_out` / `atk_ways`、`atk_tu_part4.rs` 里一处 `atk_register_stuck`、`part2/6/7` 里读 `ways_out()` / `pushing_the_root_ring` 的几处改读垫片。拼法：冻结副本的 `raise_floor_takes_…table.rs` 后接这些文件，成一个新的 harness 测试目标（名字不带 layer0）。
- `src/vw/part9_k14.rs`、`part10_k3.rs`、`part10_k3b.rs`：本轮新写的测试——K1-4（报错那一步与要出路那一步碰了什么：计数盘、分配器克隆计数；读错注入；盘表不对）、K1-3（不动 `(实例代号, txg)` 的状态变化）、K1-2（记对了没有）、K3（推完之后那一段与走空能不能分开）。接在 `atk_tu_part*.rs` 之后拼进同一个目标。需要量法副本（`tree-k14`）里的两处改动：`unlink.rs` 的 `DeletionSwitches` 手写 `Clone` 带全局计数 `ALLOCATOR_CLONES`、`mounted_session.rs` 的 `VW_SKIP_REPORT` 开关（置着时 `wide_protection_at_the_ceiling` 一进来就交 `NotWhatStopsTheAdmission`）。
- `src/gpu-variants.tsv`、`gpu-variants2.tsv`（上一轮的 49 个卡池模块变体）、`gpu-variants3.tsv`（本轮新增 53 个）。
- `scripts/`：`gen_runs.py`（从上一轮探针日志的 `S2` 行读出批次）、`piece.sh` / `par4.sh`（起探针，一件 3G）、`arun.sh` / `par_atk.sh`（起攻方原型的测试，`TGT=k14` 选编译目录）、`sum_new.py` / `jk_holds.sh`（汇总）、`mutworker.py`（逐行施加变异表行或我自己的行文件，跑点名的 `cargo test`，结果追加到结果文件）；其余是上一轮留下的脚本，本轮没用。
- `scan/`：`vw_extract3.py`（从源文件 `mod tests {` 机械抽出「读自己源文件的用例」与闭包，拼成独立小程序，`include_str!` 换成读 `SRC`）、`vw_extract4.py`（加 `only=1 nomain=1 extra=名字`，抽纯函数用）、`replay23.py`（把变异行套到小程序上）、`replay_mut_vw3.py`（上一轮判法子分支变异）、`replay_new_mut.py`（本轮新分支变异）、`gen_crash_variants_vw.py`、`gen_more_variants.py`（整份源文件变体）、`corpus-main.rs.txt` / `corpus-Cargo.toml.txt`（真 Rust 语料对拍，`proc-macro2 = 1.0.106` + `span-locations`）、`corpus-files.txt`（语料文件清单）、`hide_tests.rs`（宏包住测试模块的 rustc 小程序）。
- `logs/`：报告引的日志与结果。`done-*.txt` 是 `mutworker.py` 的结果（名、结局、红的用例、耗时）；`spec-*.txt` 是每件的环境变量；`rows-*.tsv` 是施加的变异行；`gpu-variants-all.txt`、`crash-variants-main.txt`、`old-mut-*.txt`、`old-scan-rows.txt` 是扫源码小程序的输出；`corpus-1.txt`、`corpus-neg.txt` 是语料对拍。

## 重建

1. `rsync -a --exclude target <冻结副本>/ tree-a/`；`patch crates/singlefs-core/src/allocator.rs < src/core-allocator-debug.diff`；把 `RemainderOfTheUserChange` 与 `rehearse_…` 放成 `pub`；拷 `src/p10z_probe.rs.txt` 成 `crates/singlefs-harness/tests/p10z_probe.rs`；`cargo test --release --offline -p singlefs-harness --test p10z_probe --no-run`，`scripts/par4.sh` 起批（清单用 `scripts/gen_runs.py` 从上一轮日志生成）。
2. 攻方原型：`cat <冻结副本的 raise_floor_….rs> src/atk-pqrs/atk_part*.rs > crates/singlefs-harness/tests/atk_pqrs_vw.rs`；`atk-tu` 同法成 `atk_tu_vw.rs`（再接 `src/vw/part9_k14.rs`、`part10_k3b.rs`）；`scripts/par_atk.sh` 起测试。
3. 变异：三份干净副本 `tree-m1..3`，`python3 scripts/mutworker.py <树> <编译目录> <行文件> <结果文件> <预算秒>`。
4. 扫源码：`python3 scan/vw_extract3.py <源文件> <文件名> <输出 main.rs> skip=…`，`rustc --edition 2021 -O`；`SRC=<变体文件> <二进制> [用例名…]`。
