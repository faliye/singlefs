# m3-close-b19-e165-r3：C650 剩下那一族的新判据，第三轮（最终轮，2026-11-05）

## 一、题面

前两轮判决：`research/prompts/m3-close-b19-e165-r1-main-verification.md`、`research/prompts/m3-close-b19-e165-r2-main-verification.md`。第二轮定了以下几件：

- 形甲造不出：两条 `checkpoint_txg` 相同、非 0、实例代号不同的根，不会同时进择根。
- C971（多处读坏时可写挂载写出倒挂的新实例根）那一形落在 D22（单元原子性怎么合成） 已定项 8 射程「双故障形不在承诺里（用户 2026-09-28 定不罩）」里。
- 按原判据「改坏择根序这一族要红」，C650 剩下那一句没满足（记了一次输）。病根是「判别子观测不到」，处方是改判据，改完是新一轮。

第二轮给的新判据（零轮）：

> 三条择根变体由 Verus 必红世界杀，v1826 另由 KL 杀；承诺内的合法写者造不出让它们红的盘面，等价于「同一非 0 txg 至多一条根进择根」，这一句做成读者侧逐槽穷举的入库用例（攻方第二份模型那一条做成 harness 档用例）留档。

三条变体：v1823 择根核比较不看实例代号，v1825 择根核 txg 平局时实例代号低者赢，v1826 择根核以实例代号为主、txg 只破平局。

## 二、实现今天的样子（主 agent 的观测，2026-11-05）

方案按 `crates/` 今天的实现来谈。

- 读者侧落点核：`crates/singlefs-core/src/recovery.rs` 第 1865–1873 行，`read_root_ring_slot` 解出的根若不在 `root_sits_in_the_ring_slot_the_formula_gives` 给的槽里，就交 `NotSelfVerified`；`crates/singlefs-core/src/root_ring.rs` 第 188–193 行，txg 0 只认第 0 代那几槽（`genesis_root_ring_slots`），别的 txg 只认 `target_for_publish` 那一槽（读码）。
- 择根：`recovery.rs:1888` `choose_root` 把每槽摊成择新键，交 `crates/singlefs-replay-prefix/src/lib.rs:3989` `choose_root_slot`（读码）。
- 实例表清扫读根环那一遍（`crates/singlefs-core/src/instance_table_sweep.rs` 第 461 行起 `every_root_of_the_ring_self_verified`）不调落点核（读码，第二轮登成线索）。
- 第二轮攻方模型 `research/prompts/m3-close-b19-e165-r2-opus-model/e165_attack_form_a_reader.rs` 在主 agent 复跑（任务 7660）里量到：产品读者上，S = 4、8、16 时同一非 0 txg 至多一槽被收；对照臂去掉落点核之后是 26、50、98。整行在第二轮判决第一节。
- 现有的读者侧测试：`crates/singlefs-harness/tests/root_ring_readers_take_a_root_outside_its_formula_slot_for_a_bad_read.rs`，`root_ring.rs` 里的单测 `a_root_sits_only_in_the_ring_slot_its_txg_targets`（第二轮辩方说只取了一个点，推的）。

## 三、要回答的问题

**Q1（攻）**：

1. 新判据的等价那一句（「承诺内的合法写者造不出让它们红的盘面」⟺「同一非 0 txg 至多一条根进择根」）两个方向都成立吗？有没有一种盘面满足后一句而三条变体里某一条与基线择得不同（看 v1826：实例更高而 txg 更低的那一形）？有的话，新判据要改成什么？
2. 留档用例只罩读者侧，判别力够不够？写者侧「txg 0 的根只有 mkfs 写」要不要另一条用例？哪一种改坏能同时躲过 Verus 必红世界与这条用例？
3. 第二轮那几个读数（第二轮判决第一节）能不能核：给出复跑的整条命令，与它该打出的那几行。

**Q2（辩，复核第二轮判决）**：

1. 「C971 那一形在承诺外」引 D22 第 185 行，引得对不对：那一句的「每盘各坏一槽系统配置」含不含「读回全零」「读得出而自证不过」两种读数？
2. 第二轮记的那一次输、判的病根（判别子观测不到），站不站得住？
3. 第二轮的「去向」表里，有没有漏掉的、或越过证据下的？

前提观测都在前两轮判决与腿报告里；没量过的写「推的」。
