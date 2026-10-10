# 根环实现第 10z 批第三块：代码轮第一轮判决（2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 G2 S4 S6 S9 -->

正文 `research/prompts/_m3-batch10z-c-code-r1-body.md`。两腿报告：
- 正推：`research/prompts/m3-batch10z-c-code-r1-sonnet-output.md`，sha256 a651d015…8aa，224 行；
- 攻方：`research/prompts/m3-batch10z-c-code-r1-opus-output.md`，sha256 99110a99…5bc，254 行。

## 一、核查

| 项 | 结论 |
|---|---|
| 开工快照 | 冻结副本 `~/.local/share/singlefs/work/m3-batch10z-c-code-r1/tree/` 对 `research/prompts/m3-batch10z-c-code-r1-snapshot/crates-sha256.txt` `sha256sum -c` 全过 |
| 引文 | `cite-check.py` 两份报告合跑，冻结副本与仓根各一次：`✓ 核了 67 处引文，对上 67 处，没判 0 处` |
| 主 agent 现读 | 冻结副本 `crates/singlefs-checker-tier/src/crash_amplification.rs` 第 3079–3090 行：没有卡报 Out of Memory 时先 `assert_eq!(stored_in_order.blocks_waiting(), 0, "领走的块每一块都写进了库")`，这一句不看叫停；下一句才写 `!work.blocks.any_left() || stop_is_requested()` |
| 复跑 | 草稿目录 `~/.local/share/singlefs/work/m3-batch10z-c-code-r1-main/rerun.sh`，两份模型各在一份冻结副本拷贝上跑，rc 全 0：攻方 `ATTACK k1_5=stop-in-the-closing-round … ended=AssertPanicked("领走的块每一块都写进了库（等着的 4 块）")`、`k1_4=cross-group same_group=false … y_told_out_of_memory=false`、`k1_4=genuine-panic-widened told_after_waiting=true told_at_the_failure=false` 与 `output-models.txt` 逐字相同；正推 `STOPSIM stop_from_round=Some(2) … assertion_failures=["round 2: blocks_waiting()=9 != 0…`、`DIRECTED device lost, ledger not asked yet (window): ledger_room=19550 truth=16990` 与 `logs/` 里的相同 |
| 腿的数 | 都是腿的模型或副本上的数，不进 kb |

被判的源文件：`crates/singlefs-harness/src/gpu_memory_ledger.rs`、`crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs`、`crates/singlefs-checker-tier/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs`。

## 二、逐格判决

| 格 | 两腿 | 判 |
|---|---|---|
| K1-1 G2 甲 | 两腿：「见到的那一份」零命中；上一轮两腿模型在冻结副本上重跑多给一律 0。正推：设备在卡上丢了、账还没被问到的那一下读数多给最多一份驱动（`ledger_room=19550 truth=16990`），之后每条借的路都经重起判，不会真建上缓冲；攻方 8000 段账放行而放不下 0 次、那一下最多 2319 MiB | 站住；那一下窗口不是这一批带进来的，借之前必经重起判，记下不改 |
| K1-2 S6 | 两腿：被拒之后每次再来都判。正推：每丢一次、重起一次多扣一份 D，k=5 时账 6990、真 19550，封顶在这个进程占着的 | 站住；累加的少给是甲之下的代价，记下 |
| K1-3 对拍用例 | 正推读码做到 | 站住 |
| K1-4 同块几段都报到再认 | 攻方：同一设备上不同组的两段（同一测试进程里几条用例共用卡池设备），一段因设备丢了先失败、先认，认不出显存不够（`y_told_out_of_memory=false`，同组对照 true），wgpu 的丢失原因里本来写着显存不够，可达性推的；真 panic 在设备没丢时发生、之后同块别段撞上致命的 Out of Memory，等齐再认把它说成显存不够（`told_after_waiting=true`，改前 false）。正推：后一形与原设计同向，等齐把窗口拉宽；挂住两腿都没造出（攻方 280 组全收齐） | **打中**（两形），并进下一批 |
| K1-5 收尾再起一轮 | **两腿都量到**：叫停落在收尾再起的那一轮里，上一轮判完的块还等在按块序写库那一层，第 3081 行断言不看叫停、panic，这一趟判红，判完的块不入库（攻方等着 4 块，正推等着 9 块）。攻方随机 6000 段里 9 段，全在叫停之后。两腿：`outcome.cards` 每轮追加，同一张卡在 `cards_used` 里算两遍（只影响汇总行）。判两遍、漏判、无限再起两腿都没造出 | **打中**，违反 K2-2「判红」；并进下一批 |
| K2-2 其余 | 正推：设备丢了那一下多给被重起判纠正之后，那张卡这一趟不用，单卡机器整趟本次未跑（读码推的）；一张卡报过 Out of Memory 整趟不再用，是设计 | 记下 |
| K3 判别力 | 攻方 15 处改坏 harness 档 7 处照绿，三处是真毛病：`notify_all` 改成 `notify_one` 三段以上挂住；范围里接到的挪到等完之后再记（退回第 10z 批第一块那一形，同组 2000 次只认出 294 / 1035 次）；先领块再问叫停。正推 7 条实质项照绿（M01、M05、M10、M13、M18、M30、M31），另写 `probes/killers.rs` 五条草稿用例对这七条全红；checker 档第 ① 段外层循环（`crash_amplification.rs` 第 2927–3148 行）与第 D 条接线没有用例也没有变异行 | **打中**，补用例与变异 |

## 三、下一批（第 10z 批第五块）

一簇，都在 `gpu_memory_ledger.rs`、`crash_amplification.rs`、`gpu_unit_checks.rs` 与 `crates/mutations.tsv`：
1. K1-5：叫停时第 3081 行那一句不判红，判完的块照块序入库、没判的留给下一趟；`cards_used` 同一张卡只算一次。用攻方 `attack_10zc_rounds.rs` 的 `stop-in-the-closing-round` 四格与正推 `rounds_sim4.rs` 的 `STOPSIM` 当用例。
2. K1-4：同一设备上不同组的一段因设备丢了失败时，按丢失原因（含显存不够）认；真 panic 发生时设备没丢的，不因之后别段的 Out of Memory 说成显存不够。用攻方 `attack_10zc_endings.rs` 那几格当用例。
3. K3：两腿列的照绿项各补会红的用例与变异行（正推 `probes/killers.rs` 可直接改成入库用例），`notify_all`、范围里接到的当场记、先问叫停再领块三处必补；第 ① 段外层循环与第 D 条接线能抽成纯函数的抽出来测。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D24（后台重活能不能卸给 GPU） 已定项 2 | 备料 | 2026-10-08 不受影响：判的是显存账与第 ① 段派活的实现与测试，三条硬约束不变 |
