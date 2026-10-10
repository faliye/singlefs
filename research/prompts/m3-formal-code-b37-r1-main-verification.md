# 第 37 批代码轮第一轮判决（2026-10-08）

<!-- doc-lint:not-numbers B1 B2 B3 B4 -->

被判的文件（第 37 批，主 agent 自己写的几处，逐块见改动全文 `research/prompts/_m3-formal-code-b37-r1-diff.md`）：
- `crates/singlefs-checker/src/image.rs`（第 211–212 行 I-4.1 常量的文档注释）；
- `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs`（内联单测 `every_criterion_has_a_positive_control_and_every_control_is_red`）；
- `crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs`（两条内联单测的钉值）；
- `crates/mutations.tsv`（本批两行）。

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 改动全文 / 背景材料 | `research/prompts/_m3-formal-code-b37-r1-body.md`、`_m3-formal-code-b37-r1-diff.md`、`_m3-formal-code-b37-r1-background.md` | — |
| 冻结副本与快照 | `~/.local/share/singlefs/work/m3-formal-code-b37-r1/tree/`（清单同目录 336 行）；文件快照 `research/prompts/m3-formal-code-b37-r1-snapshot/files.sha256`、kb 快照同目录 `kb-sha256.txt` | 两腿引文都读冻结副本；写判决时主工作区那几份已被这一轮的改法改过（第三节），腿引的行号对冻结副本 |
| 云端正推（Sonnet） | `research/prompts/m3-formal-code-b37-r1-sonnet-output.md`（sha256 905539af…），模型 `…-sonnet-model/` | 主 agent 在会话草稿副本里跑 `tools/rerun.sh <冻结副本>`（不带 harness 那一段），输出与模型目录 `rerun-output.txt` 对应几段逐字相同 |
| 云端攻方（Opus） | `research/prompts/m3-formal-code-b37-r1-opus-output.md`（sha256 437331bd…），模型 `…-opus-model/` | 主 agent 在会话草稿副本里跑 `tools/rerun.sh`，退 0（脚本自己逐字比报告里贴的输出），第 17–19 行与「每档各自」两行与报告同 |
| 引文 | 两份腿报告 | `cite-check.py` 两份合跑、`--root` 冻结副本与仓根：102 / 102 对上 |

harness 档那一半（正推跑的 `new_pool_file_creation_publish` 9 通过）主 agent 没复跑，判决不引它的数。

## 二、逐格判决

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| B1 三档并起来罩全表 | 一致：35 行没有一行三档都不判 | 一致（没打中那一节：交叉标签、三档都不判的行都排除） | 站住 |
| B1 每一行的对照在判它的那一档里 | 一致：读数只出自判它的那一档（靠 `is_judged_under` 与出行处共用谓词） | 打中（甲）：并集只问每条判据在三档合起来的读数里有没有一条（第 18316 行 `.any`）。`is_judged_under` 只在 `positive_control_readings` 里用，正式跑判不判走 `main` 的判定行（第 8139、8207–8260 行），不经它；它在一档少一条臂时那一档照判、不跑对照，V2 只管跑了不红（第 17779 行），并集里别的档的读数把缺口补上，单测照过。模型扫 230 种只删不加的写法，单测放过且正式跑有缺口 177 种 | **采攻方**。两腿不冲突：正推核的是「今天的读数出自哪一档」，攻方核的是「单测能不能抓到某一档少跑」；后者是这条单测要防的。主 agent 复跑攻方模型坐实 177 这个数，坏法 `ALimit => !run_profile.judges_j8()` 由第三节变异行在改后的单测上现证 |
| B1 r1、r2 两段 | 一致：两段没动，r2「不判拆行」仍成立 | 打中（乙，原有）：r1 段读数为空也照过；r2 段读数一条都不判红 | **采攻方**，原有缺口，一起改 |
| B1 档与正式跑对不对得上 | 部分对得上：r3 加两份表都取的那一档驱动里没有，正式跑带 `--table both` 的全配 r2（`research/scripts/replay.sh` 第 741、744、747 行） | 打中（丙）：同一事实；另指出第一档 r3 直接建表多余 | **两腿一致，采**：改成正式跑真用的五档 |
| B2 25 | 一致 | 一致（没打中） | 站住 |
| B2 三种写 | 一致 | 一致（没打中） | 站住 |
| B2 别处旧值 | 打中文字：第 2446 行文档注释仍写 23；第 2444 行「第 187–188 行」落后 | 打中同一处，另有第 1460 行同一句 | **两腿一致，采**：主 agent 现查 `crates/singlefs-harness/tests/new_pool_file_creation_publish.rs` 第 189 行 `let warm_up_operation_count;`、第 201 行赋值 |
| B3 函数名 | 一致 | 一致 | 站住 |
| B3 注释漏写 | 「只列四种的两种，不算错」 | 打中（原有）：漏「锚点认不定、所选根 txg 没有后继」两种（`walk.rs` 第 9017、9021 行） | **采**：补全；正推的「不算错」与攻方的「读的人会以为锚点认不定时判红」不冲突，补上不改行为 |
| B4 两行新变异 | 一致：锚点各一次、红在断言上（推演） | 一致：红在第 18317 行断言 | 站住；改法落地后这两行锚的测试代码不在了，整行换掉（第三节） |
| B4 r3 直接建表一档被改掉 | 等价变异，造不出 | 同 | 改法去掉了这一档，不再有这一格 |
| B4 `is_judged_under` 各臂无变异 | — | 打中：表里没有一行锚在它的臂上 | **采**：补两行 |
| B4 报数次序无变异 | 打中：只有 nff 这条单测管 | 打中同一处，给了一行（原文命中 1 次） | **两腿一致，采** |

## 三、去向

主 agent 当场改（主工作区，改前四份副本在 `~/.local/share/singlefs/work/m3-formal-37b-before/`）：

1. E165 那条单测改成正式跑用的五档各自核：r1（合法、对抗两族）、r2、两份表都取的 r2、r3、r5（三族）。每档断言「跑出对照的判据集合 = 登记给这一档的那一份」（字面写死：基础 18 条；J6Qn 与多盘 5 条在 r2、r3、r5；J7 五条在 r2、r3；J6Qn 拆行在 r3、r5；J8 四条在 r5；T 两条只在两份表都取的 r2），且这一档读数全红；五档并起来罩 `Criterion::ALL`。修 B1-甲、乙、丙。
2. nff 装置第 1460、2444 行「该文件第 187–188 行」改成「第 189、201 行」，第 2446 行 23 改成 25 并补一句 C651 的来由。修 B2 文字。
3. `image.rs` 第 212 行补「锚点认不定、所选根 txg 没有后继」。修 B3。
4. `crates/mutations.tsv`：本批两行（第 3383、3407 行）锚的测试代码改掉了，整行换成锚在被测的 `is_judged_under` 上：第 3383 行「E165 基础 18 条在 r5 档判了不跑对照（几档并起来判时被 r3 的读数补上）」、第 3407 行「E165 T_table、T_row 在两份表都取的 r2 档判了不跑对照（几档并起来判时被 r3 的读数补上）」；追加第 3496 行「E165 基础 18 条在 r1 档不跑对照（r1 段读数为空也照过）」、第 3497 行「报数次序里已删 inode 树与墓碑树对调（各种之和不变，core 的自检照绿）」。`crates-mutation-rows.py static-check` 整表 `CHECKED 3492 322 10`。

验证（`SINGLEFS_HEAVY_TESTS=user-request`，日志 `~/.local/share/singlefs/work/m3-formal-37b-run/`）：E165 那条单测 `1 passed; 0 failed`；nff 装置 `17 passed; 0 failed`；clippy `-D warnings` 过；fmt 先报这条单测排版，`rustfmt` 之后 `cargo fmt -p singlefs-checker-tier -- --check` 0 处（只动这条单测一段）。四行变异证红见第四节。

## 四、复跑核对

两腿模型复跑见第一节表。另外主 agent 现跑两样（`SINGLEFS_HEAVY_TESTS=user-request`，日志都在 `~/.local/share/singlefs/work/m3-formal-37b-run/`，副本跑完已删）：

- 坐实 B1-甲：改前那份单测（并集写法，`~/.local/share/singlefs/work/m3-formal-37b-before/` 那份装置）加坏法 `| Self::ALimit => !run_profile.judges_j8(),`，`test result: ok. 1 passed; 0 failed`（`old-union-test-with-alimit-mutation.log`）。改前单测确实放过这一坏法。正式跑「判了没对照」那一半主 agent 没现跑装置，依据是攻方引的 `main` 判定行（第 8139、8207–8260 行）与 `is_judged_under` 只在 `positive_control_readings` 里用（cite-check 对上），推的。
- 四行变异 `prove-red.sh`：「✓ 点名 4 条：跑了 4 条，跳过 0 条，跑的都抓到了」（`prove-red.log`）。前三行红在改后单测第 18377 行「r5 档 / 两份表都取的 r2 档 / r1 档跑出对照的判据与登记给这一档的那一份不同」，第四行红在 nff 装置第 1971 行 `publish_writes_line_gives_the_total_then_every_kind_in_report_order_with_zeros`（`prove-red-logs/001–004.log`）。

## 五、交用户表（零轮）

| 条 | 形态 | 轮数 |
|---|---|---|
| 第三节第 1 条 | 五档各自写死期望集的写法是攻方提的收严，主 agent 照它写进单测；被攻过零轮 | 零轮；照「打中的写回代码，再攻一轮」开第二轮 m3-formal-code-b37-r2 攻改后的单测与四行变异 |
| 第三节第 4 条 | 四行变异替换与追加 | 零轮，证红见第四节；并进第二轮 |

## 回看决策

不涉及决策：这一批只改 E165（恢复重放有界穷举与变异杀伤界）与 nff 两个装置的内联单测、checker 的一句注释与变异表，不改任何条款，也不改任何正式跑的判定与产物。
