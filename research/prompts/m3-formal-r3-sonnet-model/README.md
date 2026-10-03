# m3-formal-r3 辩方腿（Sonnet）的模型与复跑

冻结副本 `/tmp/claude-1000/m3-formal-r3/tree/`（sha256 清单 `research/prompts/m3-formal-r3-snapshot/crates-sha256.txt`，212 份全 OK）。

## 一、S5 版本序（译本，只数选根器，不是 Rust 上量的）
`python3 s5_version_order_model_v2.py > s5_out_v2.txt`，纯 Python，秒级。输出表就是报告第二节的表。

## 二、S6 顶格与 S1 其余字段（E165 装置副本，只加计数、不改任何判据）
1. 拷冻结副本到草稿目录，`python3 patch_e165.py <副本>/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs`，再 `python3 e165/patch_e165_precontrol.py` 同一个文件。
   加的东西：`judge_the_fixed_point` 里按 (前提真假, 第一次是否落在 u64 最大值, 第二次结局种类) 数；每个域的 verdict 行之后打一行 `S6SPLIT` 并清零；环境变量 `S1_MARKER=1` 时时间线上每一版的根带卸载记号，`judge_a_legal_state` 里数「施加至少一条」的态里返回的根与所选根在非新根段字段上的差（`S1SPLIT`）。
2. 变异版：`apply_mutation.py <副本> <crates/mutations.tsv 行号>` 套一行（原文必须恰好命中一次），`apply_mutation.py <副本> restore` 还原。装置只重编一次（`cargo build --release --locked --offline -p singlefs-checker-tier --bin e165_replay_bounded_enumeration`），`e165/build-variants.sh` 一口气编 8 个变异版。
3. 跑：`e165/batch-e165.sh`（light 档，合法一族与对抗一族各一遍，4 线程）；整条经 `run-with-memory-cap.sh 6G` 与 `capped.sh 4`（`e165/master.sh`、`e165/master-retry2.sh`）。
4. 汇总：`python3 s6_tabulate_v2.py <runs 目录> > s6_table.tsv`。`extract/` 里是每个运行的 S6SPLIT、S1SPLIT 与域合计行，`extract/sha256-full-outputs.txt` 是完整输出的 sha256。
5. 装置没动判据的证明：打了补丁的基线跑 `--bounds light --family legal`，876 行 `E7RESULT` 与 `research/results/e165-replay-bounded-enumeration-2026-10-01-light-legal-family.out` 的 876 行逐行相同（`diff` 无输出）。
6. `s6_table.tsv` 的第 4 列（当前判据下的违例数）与装置自己的 `j6_premise_holds_violations` 在全部 126 行上相等（`awk` 核过无不等）。

## 三、Verus（冻结核的删判表与 S2 补强原型）
- `verus/make_judge_deletions.py` 在冻结核 `crates/singlefs-replay-prefix/src/lib.rs` 上给 13 道 break 判各做一份「条件前加 `false &&`」的变体；`verus/run-verus-batch.sh <目录…>` 逐个 `verus --crate-type=lib --edition=2021 --crate-name singlefs_replay_prefix --no-cheating src/lib.rs`（Verus 0.2026.09.20.aef82ed）。结果 `verus/summary-variants.txt`。
- 原型：`verus/make_proto1.py` + `verus/make_proto2.py` 把冻结核改成 `verus/proto2-lib.rs`：`m2_holds` 加「施加的计数器从链首起逐一连续」，锚点写成 spec，主循环带链条不变量；`verus/proto2.verus.out` 是「14 verified, 0 errors」。同一张删判表在原型上的结果 `verus/summary-variants2.txt`，冒烟与两行必红世界 `verus/summary-variants3.txt`（`verus/make_proto2_extra.py` 造）。
- 这些都在草稿目录的副本上做，没有改 `crates/`，也没有并进第 4 批的活。
- 七·5 空真样本：`verus/make_vacuity_variants.py` 造两组：`summary-variants4.txt`（`replay_core` 的 `requires` 加 `false`：仍是 14 verified, 0 errors，判绿四条件全过）与 `summary-variants5.txt`（加一个真调 `replay_core` 的见证函数：基线 15 verified, 0 errors，空真版 14 verified, 1 errors，原始输出 `verus/vacuous_requires_false_plus_witness.verus.out`）。
