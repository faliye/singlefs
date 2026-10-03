# 第 17 批代码轮第一轮判决（2026-10-04）

<!-- doc-lint:not-numbers K1 K2 K3 K4 M1 H1 f1 f2 -->

被判的文件：`crates/singlefs-checker/src/image.rs`、`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`、`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl`、`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl`、`crates/singlefs-checker-tier/src/crash_judge_gpu.rs`、`crates/singlefs-core/src/transaction.rs`（一句注释），测试 `crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs`、`crates/singlefs-harness/tests/checker_known_bad_images.rs`、`crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs`、`crates/singlefs-harness/tests/tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs`、`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`，`crates/mutations.tsv` 17-1 追加的 8 行。

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 改动全文 / 背景材料 | `research/prompts/_m3-formal-code-b17-r1-body.md`、`_m3-formal-code-b17-r1-diff.md`、`_m3-formal-code-b17-r1-background.md` | — |
| 开工快照 | `research/prompts/m3-formal-code-b17-r1-snapshot/`（crates 274 份，冻结副本 `/tmp/claude-1000/m3-formal-code-b17-r1/tree/`） | crates 无输出；kb 三份对不上：`.claude/kb/invariants.md`、`.claude/kb/decisions/08-核心索引结构.md`、`.claude/kb/experiments/171-三oracle合谋测试.md`——开工之后 18-3 写回与删除与并发那条线第二次写回改了它们（旧串新串在 `/tmp/claude-1000/m3-formal-18-3/report.md` 与 `/tmp/claude-1000/m3-writeback2/scribe-{a,b}/spec.md`）；正推读到 `invariants.md` 第 314 行中途被改（加「宽不到 124 整表不判」），照改后的对，与代码一致 |
| 云端正推（Sonnet） | `research/prompts/m3-formal-code-b17-r1-sonnet-output.md`（256 行，sha256 be9afd1b…） | 引文 |
| 云端攻方（Opus） | `research/prompts/m3-formal-code-b17-r1-opus-output.md`（202 行，sha256 1f1b2281…），模型 `…-opus-model/` | 主 agent 在会话草稿 `b17-rerun/` 跑 `tools/rerun.sh`（`RERUN_RC=0`）：`summary.txt` 各行与它的 `out/counts.txt` 同（runA1 `cells=40 holds=18 violated=22`、runB1 `cells=41 holds=22 violated=19`、runA-f1 / f2、runB-f1 / f2、A2c 三臂 24 / 24 红、H1 三臂 24 / 24 判成立、runE、runC1 两行）；8 行变异 `cargo_rc=101` 点名用例红；改法两臂 harness 目标全过 |
| 引文 | 两份腿报告 | `cite-check.py <两份报告> --root <冻结副本> --root . --background <背景材料>`：「✓ 核了 62 处引文，对上 62 处，没判 0 处」 |

## 二、逐格判决

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| K1 I-9.27 | CPU 与条款逐句一致；GPU 推同判；M1 只红 I-9.27；**证据缺口**：种类 2 自等、写 0 七种、种类 9 / 10、多条种类 2 放宽四处它自加的变异在 harness 两个目标里都不红 | GPU 推同判 | 站住；四处证据缺口进下一批（各补一张坏镜像或一条用例，照 `checker_known_bad_images.rs` 的写法） |
| K2 I-5.5 | min 改 max 不红（夹具每盘只一个数据单元） | **打中（复跑坐实）**：mkfs 之后只可写挂载 k 次再写第一个文件，k ≥ 10 时 U、U + 1（mkfs 的实例表，实例 2 写行时 COW 走）过了回收门槛，实现照 D3（空间分配） 已定项 8 第 1 条把数据放在 U，I-5.5 误红（runA1 40 格红 22，默认环与 128 MiB 环都是 k = 9 → 10 翻转）；同一次挂载里推零单元发布 z ≥ 22 也误红（runB1 41 格红 19）；「写死从 U + 4 起扫」的坏实现 H1 在这段历史上被判成立 | **打中，病根在条款**：D3 已定项 8 射程「t1 = 单元区起始槽号 U 加 4」（用户 2026-10-04 定的那一类）与已定项 8 第 1 条（起点最小的空槽对）在回收之后不能同真；判法照字面判没错。**交用户**（第五节第 1 行）。两腿都没打中的：A2c 那一形 24 / 24 红；非默认环、零长文件只在这一形上红；两遍读环不一致 76 格隔离数恒 0 |
| K3 覆盖表断言 | 开 `gpu` 特性 `cargo check` 漏、重、乱序、越界都编不过；重复登记那一支报的话说成「漏」 | — | 站住；报错措辞进下一批改一句 |
| K4 变异 | 8 / 8 副本上红，第 1605、1809 行仍红 | 8 / 8 红，第 2、3、4 行是同一变异体点名不同用例；8 行碰不到 K2 那一格 | 站住 |
| 另：WGSL | 正推用独立小程序跑了 naga 校验，三个内核都过、改坏一个标识符会红 | — | 记着；正式的 naga 用例归提交时 checker 档 |

## 三、去向

1. 17-1、17-4 的改动站住，只有 I-5.5 的判据随用户定的条款改（第五节第 1 行），改法并进下一批实现员（连同 GPU 内核那一处，GPU 同样写死 U + 4）。
2. I-9.27 四处证据缺口、K3 报错措辞、I-5.5 的 min / max 取样点（夹具加一块盘多个数据单元）进下一批。
3. 提交时：checker 档 GPU 对拍两条新用例、naga 那三条、8 行变异整表。

## 四、复跑核对

见第一节表。核不动：checker 档、GPU 没跑；崩溃落在第一个文件版本中途那一形（闭式 16 777 223 态，超单段上限）没跑。

## 五、交用户表（零轮）

| # | 形态 | 来源 | 量过没有 | 去向 |
|---|---|---|---|---|
| 1 | I-5.5 管的那一类与判据怎么改（今天的条款在 k ≥ 10 次只暖机的挂载之后误红） | 攻方 | 误红量过（副本，主 agent 复跑同）；改法 f1、f2 在副本上量过，收窄到实例 1 那一版是推的 | 弹窗交用户 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D3（空间分配） 已定项 8 | 不影响 | 2026-10-04 不受影响：射程与第 1 条在回收之后不能同真，交用户定之前不改 |
| D8（核心索引结构） 已定项 8 | 不影响 | 2026-10-04 不受影响：头 ID 关系判据站住 |
