# 第 37 批代码轮第三轮判决（2026-10-09）

<!-- doc-lint:not-numbers C1 C2 C3 C4 D1 D2 D3 D4 -->

第三轮，这一批最后一轮。被判的文件（第二轮判决第三节的改法，逐块见改动全文 `research/prompts/_m3-formal-code-b37-r3-diff.md`）：
- `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs`（内联单测 `every_criterion_has_a_positive_control_and_every_control_is_red`）；
- `crates/mutations.tsv`（第二轮追加的两行）。

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 改动全文 / 背景材料 | `research/prompts/_m3-formal-code-b37-r3-body.md`、`_m3-formal-code-b37-r3-diff.md`、`_m3-formal-code-b37-r3-background.md` | — |
| 冻结副本与快照 | `~/.local/share/singlefs/work/m3-formal-code-b37-r3/tree/`（清单同目录 336 行）；`research/prompts/m3-formal-code-b37-r3-snapshot/` | 腿引文读冻结副本 |
| 云端辩方（Sonnet） | `research/prompts/m3-formal-code-b37-r3-sonnet-output.md`（sha256 151b31db…），模型 `…-sonnet-model/` | 主 agent 在会话草稿副本里跑 `tools/rerun.sh <冻结副本> <仓>/research/results`：「与 rerun-output.txt 逐字相同」 |
| 云端攻方（Opus） | `research/prompts/m3-formal-code-b37-r3-opus-output.md`（sha256 90dbb15b…），模型 `…-opus-model/` | 主 agent 在会话草稿副本里跑 `tools/rerun.sh <第三轮冻结副本> <第二轮冻结副本> <仓>`：「✓ 模型输出与 expected-output.txt 逐字相同（113 行）」 |
| 引文 | 两份腿报告 | `cite-check.py` 两份合跑、`--root` 冻结副本与仓根：58 / 58 对上 |

## 二、逐格判决

| 格 | 辩方 | 攻方 | 判决 |
|---|---|---|---|
| D1 14 个单元是 `replay.sh` 里正式跑的全部 | 站住：18 行起装置的驱动去掉 `root_choice` 三行剩 15，703 与 706 同一单元 | 没打中（先当打中过一条「r2 直接建表、两族一起跑」，查清杀伤界驱动编的是 `--root` 指的冻结根后撤回） | 站住 |
| D1 `replay.sh` 之外登记的正式跑 | 打中：第 5 次重跑登记比对对象表第 4–7 行（`research/prompts/e165-r5-prereg.md` 第 669–672 行）是 r2 直接建表合法、对抗各偶、奇数一份，「无复跑行」，单元表没有；J7 那一臂谓词换成 `compares_the_two_record_tables()` 的 5 种改坏 14 个单元都不变、那一跑丢 J7 对照 | — | **采**。主 agent 现查第 669–672 行坐实；现跑第二轮单测加 `\| Self::J7cUnmountMarkerWhenNothingIsApplied => run_profile.compares_the_two_record_tables(),` 得 `1 passed`（第四节）。改法见第三节第 1 条 |
| D1 14 份集合对产物 | 站住 | 没打中 | 站住；r1 三个单元注释点的是 2026-10-04 产物、`replay.sh` 拿 2026-10-01 那三份比，两边 `name=control` 行逐字节相同，不改 |
| D1 产物与今天的装置 | 站住（只到判据集合一级） | 单测这一侧没打中；另有 10 份产物出来之后没在新编的二进制上复跑过，是登记一侧的余项 | 站住；余项记账（第三节第 3 条） |
| D1-乙 同组单元集合相同 | — | 弱打中（两步）：手写的档或奇偶写错成同组另一个，单测照过 | 不采：要先把单测自己写错，再碰上集合恰好相同；记一句 |
| D2 族这一维 | 站住：70 个挪族改坏新单测抓 66，另 4 个等价 | — | 站住 |
| D2 第二轮判决「那 9 个红得对」 | 站不住一半：那句只对第二轮改前的五档写法成立，逐单元写法下这 9 个 0 红；其中 5 个是上面 D1 那一处缺口，4 个与 C1-乙 同类 | — | **采**。第二轮判决第二节 C2 行那一句更正为：「对第二轮改前的五档单测成立；逐单元单测在这 9 个上 0 红，其中 5 个改变一条没有复跑行的正式跑（第三轮补单元后判红），4 个只改变 r3 档的合法一族、r3 没有合法一族的正式跑（与 C1-乙 同类）」。第二轮判决原文不改 |
| D2 C1-乙、C3-丙 判成不归这一批 | 站住；C3-丙 另有 `replay.sh` 的 E165 行 `exact` 逐字节比整份产物兜着（整轮门禁里是否实际跑到，推的） | — | 站住 |
| D3-甲 `main` 那一次调用 | — | 打中：单测直接调 `positive_control_readings`，不经 `main` 第 17769 行；那一行把 `&selection.families` 换成 `&[Family::LegalHistory]` 或把档换成 `RunProfile::R1`，单测照过，正式跑 9 或 12 条对照集变 | **不在这一批改，记账**。理由：这一行改坏会改 `replay.sh` 里每一条 E165 正式跑的 `name=control` 行，`replay.sh` 那几行判 `exact`，提交时的实验复跑逐字节比会红；改法（单元改用驱动命令行字面经 `parse_the_command_line` 解、与 `main` 共用一个函数）要动装置的非测试代码，且是第三轮新冒出来的零轮形态，照「第三轮之后停」不再为它开一轮 |
| D3 判据表与对照循环、照产物钉、`--only`、杀伤界模式、线程数 | — | 没打中 | 站住 |
| D4 两行变异 | 站住 | 没打中 | 站住 |
| D4 挂钟 | 站住：改后单测 10.95 秒 | 站住 | 站住；补四个单元之后 14.99 秒（第四节） |

## 三、去向

主 agent 当场改（主工作区；第二轮改后那份单测副本在 `~/.local/share/singlefs/work/m3-formal-37b-before/e165_replay_bounded_enumeration.rs.after-r2`）：

1. E165 那条单测的单元表加四个单元（共 18 个）：r2 合法、r2 对抗、奇数版本的 r2 合法、奇数版本的 r2 对抗，期望集取各自 2026-10-04 重跑产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-{legal-family,adversarial-family,odd-marker-legal-family,odd-marker-adversarial-family}.out` 的 `name=control` 集合（17、7、17、7 条），注释写明它们在比对对象表第 4–7 行、`replay.sh` 里没有复跑行。
2. `crates/mutations.tsv` 追加一行「E165 J7 五条只在两份表都取时跑对照（r2 直接建表合法一族那一跑判了没对照）」；`crates-mutation-rows.py static-check` 整表 `CHECKED 3495 322 10`。
3. 记账（不在这一批改，写进 `records/2026-10-01-里程碑三形式化接入.md`）：`main` 第 17769 行那一次调用单测罩不到（D3-甲，今天靠 `replay.sh` 逐字节比）；10 份产物出来之后没在新编的二进制上复跑过（攻方 D1 余项）。

## 四、复跑核对

主 agent 现跑（`SINGLEFS_HEAVY_TESTS=user-request`，日志 `~/.local/share/singlefs/work/m3-formal-37d-run/`，副本跑完已删）：

- 坐实 D1 那一处：第二轮改后的单测（14 个单元）加 `| Self::J7cUnmountMarkerWhenNothingIsApplied => run_profile.compares_the_two_record_tables(),`，`test result: ok. 1 passed; 0 failed`（`r2-test-with-j7-mutation.log`）。
- 改后单测（18 个单元）`1 passed; 0 failed; … finished in 14.99s`（`e165.log`）；`cargo fmt -p singlefs-checker-tier -- --check` 退 0；clippy `-D warnings` 退 0。
- 新那一行 `prove-red.sh`：「✓ 点名 1 条：跑了 1 条，跳过 0 条，跑的都抓到了」，红在「r2 合法 那个单元跑出对照的判据与那条正式跑产物里的那一份不同」（`prove-red-logs/001.log`）。

## 五、交用户表（零轮）

| 条 | 形态 | 轮数 |
|---|---|---|
| 第三节第 1、2 条 | 四个单元与一行变异是辩方提的收严，主 agent 照它写进单测 | 零轮；第三轮之后停，不再开一轮 |
| D3-甲 | `main` 那一次调用的改坏单测罩不到；改法（单元改用驱动命令行字面、与 `main` 共用一个函数）要动装置的非测试代码 | 零轮，记账，下一批 E165 的活再判 |
| C1-乙（第二轮判决交用户那一条） | r3 档的 J6Qn、J7 在 A 域判了、同档没有对照 | 条款的事，交用户；不挡拍树 |

## 回看决策

不涉及决策：这一批只改 E165（恢复重放有界穷举与变异杀伤界）装置的内联单测与变异表，不改任何条款，也不改任何正式跑的判定与产物。
