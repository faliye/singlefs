# 第 37 批代码轮第二轮判决（2026-10-09）

<!-- doc-lint:not-numbers B1 B2 B3 B4 C1 C2 C3 C4 -->

被判的文件（第一轮判决第三节的改法，逐块见改动全文 `research/prompts/_m3-formal-code-b37-r2-diff.md`）：
- `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs`（内联单测 `every_criterion_has_a_positive_control_and_every_control_is_red`）；
- `crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs`（三处注释）；
- `crates/singlefs-checker/src/image.rs`（第 212 行注释）；
- `crates/mutations.tsv`（本批四行）。

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 改动全文 / 背景材料 | `research/prompts/_m3-formal-code-b37-r2-body.md`、`_m3-formal-code-b37-r2-diff.md`、`_m3-formal-code-b37-r2-background.md` | — |
| 冻结副本与快照 | `~/.local/share/singlefs/work/m3-formal-code-b37-r2/tree/`（清单同目录 336 行）；`research/prompts/m3-formal-code-b37-r2-snapshot/` | 腿引文读冻结副本 |
| 云端辩方（Sonnet） | `research/prompts/m3-formal-code-b37-r2-sonnet-output.md`（sha256 508f4322…），模型 `…-sonnet-model/` | 主 agent 在会话草稿副本里复跑（第四节） |
| 云端攻方（Opus） | `research/prompts/m3-formal-code-b37-r2-opus-output.md`（sha256 ffe24282…），模型 `…-opus-model/` | 主 agent 在会话草稿副本里复跑（第四节） |
| 引文 | 两份腿报告 | `cite-check.py` 两份合跑、`--root` 冻结副本与仓根：115 / 115 对上 |

## 二、逐格判决

| 格 | 辩方 | 攻方 | 判决 |
|---|---|---|---|
| C1 五份写死集合对登记 | 站住：18、28、30、29、28 条与登记原句、`is_judged_under`、正式跑逐档相等 | 没打中 | 站住，但见下一行：按档对得上，族没对上 |
| C1 族这一维（辩方 C1 够不着一处；攻方 C1-甲、C3-甲） | 打中：五档都给全族，`Criterion::family` 的族过滤没人钉；35 条各换族的 70 个改坏里 66 个改正式跑对照集，单测抓 18 | 打中：五个调用四个取三族，`main` 起不出这个组合（第 17388、17416、17419 行），正式跑每条一族；70 个挪族改坏里单测放过、正式跑少对照的 48 个，族过滤与 `is_judged_in_every_family` 6 个里 3 个；一行坏法第 6193 行 `=> Family::LegalHistory,` | **两腿一致，采**。主 agent 现跑第一轮改后的单测加这一行坏法坐实放过（第四节）。改法照攻方甲：逐个核正式跑的单元（第三节第 1 条） |
| C1-乙 r3 档写 29 条、正式跑 r3 只出 8 条 | — | 打中（原有）：r3 唯一的正式跑只跑对抗一族，产物第 379 行 `control_readings=80`，同一份产物第 209 行在 A 域照判 J6Qn、J7 | **现查坐实，不归这一批**：对抗一族的正式跑在 A 域都判 J6Qn、J7，它们的对照只在合法一族那一跑里有；r2 有同档的合法一族正式跑，r3 没有，r3 登记第 424 行只登记对抗一族那一跑、靠「去掉 Q3 几行之后与 r2 逐字节相同」担。这是 E165（恢复重放有界穷举与变异杀伤界）登记的事，进交用户表；改后单测照今天的产物钉 r3 对抗 8 条，不替登记补 |
| C2 B1-甲、乙、丙 | 站住；替并集判辩护只剩代价两点（调用次数相同、不用手写集合），「抓得到某档少跑」站不住；并集判在 13 个正式跑集合不变的改坏上 0 红、新写法 9 红，过不过严由主 agent 定 | — | 站住。那 9 个是登记语义被改的改坏，红是对的，不算过严 |
| C3-乙 反盘次序、奇偶另一种 | — | 打中（造的写法）：单测只取偶数、默认盘次序，臂上多一个看盘次序或奇偶的条件 12 种全放过、8 种让正式跑少对照 | **采**：逐单元核时盘次序反报的两条、奇数版本的两条各是一个单元，第三节第 1 条一并罩上 |
| C3-丙 读数一级 | — | 打中（读码，原有）：单测只比判据集合，一条判据少掉一部分读数、剩一个红的就过；r5 奇数版本下 C8a、C8c 的 C 域读数删掉没有单测红 | **不归这一批**，记账（第三节第 3 条）：读数一级在第一轮改前就没人钉，钉它要逐单元写死 (判据, 量在哪) 的列表，范围比这一批大 |
| C3-丁 按标签比 | — | 打中（弱，两步） | **采**：集合改按 `Criterion` 比，另加一条标签互不相同 |
| C3 只加不删、空集、scan、r4、挂钟 | — | 没打中 | 站住 |
| C4 四行锚点、红在哪、第 3497 行只有 nff 杀得掉 | 站住 | 没打中 | 站住 |
| C4 变异表罩不住的（辩方 C4 漏；攻方 C4-甲） | 打中：删 r1、r2 直接建表、r3 任一调用新单测不红，无行守 | 打中：r2、r3 两个调用没有一行变异只靠它判红；族、盘次序与奇偶、读数层三类变异表里一行都没有 | **采一部分**：补族这一维两行（第三节第 2 条）；盘次序与奇偶那一类今天的臂一个都不看这两个旋钮，要新写条件才改得坏，不补；读数层随第 3 条记账 |
| C4 三处注释 | 站住 | 没打中 | 站住 |

## 三、去向

主 agent 当场改（主工作区；第一轮改后那份单测副本在 `~/.local/share/singlefs/work/m3-formal-37b-before/e165_replay_bounded_enumeration.rs.after-r1`）：

1. E165 那条单测改成逐个核正式跑的 14 个单元（`research/scripts/replay.sh` 第 703、706 行同一单元，另 710、738、741、744、747、750、753、763、855、858、861、864、867 行各一个）：每个单元一族一档、一种建表法、一种盘次序、一种标记奇偶，调 `positive_control_readings(&[族], 档, …)`，断言跑出对照的判据集合等于那条驱动 2026-10-04 产物里 `name=control` 的判据集合（写死成字面：11、7、4、19、9、6、19、6、8、16、8、4、16、8 条），读数全红；14 个单元并起来罩全表；集合按 `Criterion` 比，另断言判据标签互不相同。主 agent 另照登记推了一遍各单元的集合：合法一族基础 11 条，对抗一族 7 条，多盘 4 条，两份表都取的加 T 两条，J6Qn 与 J7 归合法一族，拆行归对抗一族的 r3、r5，J8 四条归合法一族的 r5；与产物取出来的逐单元相同。修 C1 族这一维、C3-乙、C3-丁。
2. `crates/mutations.tsv` 追加两行：「E165 对抗一族八条挪进合法一族（对抗一族的正式跑没有它们的对照，几族并起来判时照过）」「E165 T_table、T_row 只在合法一族跑对照（对抗、多盘两族带两份表的正式跑没有它们的对照）」。`crates-mutation-rows.py static-check` 整表 `CHECKED 3494 322 10`。
3. 记账（不在这一批改）：E165 的单测只钉到判据一级，读数一级（每条判据量了哪几个域、几个读数）没人钉（攻方 C3-丙 第 1、2 条）；r3 档的 J6Qn、J7 在 A 域判了、同档没有对照（攻方 C1-乙）。两条写进 `records/2026-10-01-里程碑三形式化接入.md`，后一条交用户。

验证见第四节。

## 四、复跑核对

两腿模型在会话草稿副本里复跑：攻方「✓ 模型输出与 expected-output.txt 逐字相同（93 行）」，辩方「与 rerun-output.txt 逐字相同」。主 agent 现跑（`SINGLEFS_HEAVY_TESTS=user-request`，日志都在 `~/.local/share/singlefs/work/m3-formal-37c-run/`，副本跑完已删）：

- 坐实族这一维：第一轮改后那份单测（`…/m3-formal-37b-before/e165_replay_bounded_enumeration.rs.after-r1`）加 `| Self::J6QnSplitByI812 => Family::LegalHistory,`，`test result: ok. 1 passed; 0 failed`（`r1-test-with-family-mutation.log`）。正式跑那一半（对抗一族的正式跑 `control_readings=0`）主 agent 没现跑装置，依据是攻方引的第 7758 行族过滤与模型拿 15 条驱动产物核过的对照集（模型复跑逐字同），推的。
- 改后单测（逐单元）`1 passed; 0 failed`（`e165.log`），`cargo fmt -p singlefs-checker-tier -- --check` 退 0，clippy `-D warnings` 退 0。
- 锚 E165 的五行变异（第 3383、3407、3496 行与这一轮追加的两行）`prove-red.sh`：「✓ 点名 5 条：跑了 5 条，跳过 0 条，跑的都抓到了」（`prove-red.log`），各红在「<单元> 那个单元跑出对照的判据与那条正式跑产物里的那一份不同」：r5 合法、两份表都取的 r2 合法、r1 合法、r1 合法（挪族之后合法一族多跑八条）、两份表都取的 r2 对抗（`prove-red-logs/001–005.log`）。第 3497 行（nff）这一轮没动，沿用第一轮的证红。

## 五、交用户表（零轮）

| 条 | 形态 | 轮数 |
|---|---|---|
| 第三节第 1 条 | 逐单元写死期望集的写法是攻方提的收严，主 agent 照它写进单测 | 零轮；开第三轮（最后一轮）攻它 |
| 第三节第 2 条 | 两行变异 | 零轮；并进第三轮 |
| C1-乙 | r3 档的 J6Qn、J7 在 A 域判了、同档没有对照：要么给 r3 补一条合法一族的正式跑，要么在 E165 登记里写明 r3 档这几条的对照由 r2 档那几跑担 | 条款的事，交用户；不挡拍树（E171 不用 E165） |

## 回看决策

不涉及决策：这一批只改 E165（恢复重放有界穷举与变异杀伤界）与 nff 两个装置的内联单测、checker 的一句注释与变异表，不改任何条款，也不改任何正式跑的判定与产物。
