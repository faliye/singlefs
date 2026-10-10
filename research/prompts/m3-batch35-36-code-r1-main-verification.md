# 删除批第 35 批 W（core 修补）与第 36 批 V（GPU）合入的代码：代码轮第一轮判决（2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 W1 W2 W3 W4 W5 W6 W7 V1 V2 V3 V4 V5 V6 V7 H1 H2 H3 H4 F1 F2 P Pm R -->

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / diff / 背景材料 | `research/prompts/_m3-batch35-36-code-r1-body.md`、`_m3-batch35-36-code-r1-diff.md`、`_m3-batch35-36-code-r1-background.md` | — |
| 开工快照 | `research/prompts/m3-batch35-36-code-r1-snapshot/crates-sha256.txt`（20 份）、`kb-sha256.txt`；冻结副本 `~/.local/share/singlefs/work/m3-batch35-36-code-r1/tree/`（V 合后） | 腿跑着的时候主 agent 没改 20 份 crates 文件。**主 agent 违规改了快照里的一份 kb**：攻方腿还在跑时给 `.claude/kb/decisions/23-journal的角色与格式.md` 第 392 行加了一句「只除去现行账里仍分配着的槽（……）」，别的字没动；两条腿引的 D23 都是背景材料里冻结的那一版，攻方附录引的 D23 已定项 14 原文没有落在这一句上（腿只引了 F2 那一段与「记录有缺口时拒可写」那一句） |
| 云端正推（Sonnet） | `research/prompts/m3-batch35-36-code-r1-sonnet-output.md`（189 行，sha256 7af313fd…），模型 `…-sonnet-model/` | — |
| 云端攻方（Opus） | `research/prompts/m3-batch35-36-code-r1-opus-output.md`（246 行，sha256 9aa08c66…），模型 `…-opus-model/` | — |
| 引文 | 两份腿报告 | `cite-check.py --root <冻结副本> --root . --background <背景材料>`：正推「核了 101 处引文，对上 101 处，没判 0 处」；攻方「核了 22 处引文，对上 22 处，没判 0 处」 |
| 主 agent 复跑：正推 | 模型拷进会话草稿目录 `m3b3536-sonnet-model/`，`SINGLEFS_REPO=<冻结副本> SINGLEFS_MAIN_REPO=<仓根> rerun.sh ~/.local/share/singlefs/work/m3b3536-sonnet-rerun`，再 `compare.sh <复跑> <它的 expected>` | 「static.txt 全同（10 行）」「baseline.txt 全同（8 行）」「mutations.txt 全同（49 行）」「combo.txt 全同（1 行）」；原样有 `ROW 3 rc=0 GREEN(mutation survived) name=第33批T C694 的 C 选法退成只认带末条标志的记录（last）` 与 `COMBO rc=101` |
| 主 agent 复跑：攻方 | 模型拷进会话草稿目录 `m3b3536-opus-model/`，`SINGLEFS_REPO=<冻结副本> SINGLEFS_TOOLS=<仓根> THREADS=15 rerun.sh ~/.local/share/singlefs/work/m3b3536-opus-rerun`，再 `compare.sh <summary.txt>` | 「COMPARE same lines=1511」；原样有 `cbd2-ow-k5-base	TALLY kind=ow vis=root M1=Ok r_units=overwritten cells=9` 与 `cbd2-ow-k5-all3	TALLY kind=ow vis=root M1=Ok r_units=kept cells=22` |
| 主 agent 上卡 | V 合后主工作区对拍文件「test result: ok. 159 passed; 0 failed; 2 ignored」，`CRASH_JUDGE_GPU_MISMATCH` 0 行（日志 `~/.local/share/singlefs/work/m3-batch36/gpu-agree-after-v.log`） | V1、V3、V4 在登记的对拍流上两边同判 |

两条腿的数都是腿副本上量的，不进 kb。

## 二、逐格判决

| 格 | 正推 | 攻方（一次抽样） | 判决 |
|---|---|---|---|
| K1 / K2 W1 K4-1 窄读 | 一致：K4-1 变异红；豁免与 `isolated` 去重的键只取起点槽不带跨度（推的一形：R 的 2 槽新单元与现行账一条 1 槽记录同起点时整个被豁免，旧代码同病）；`allocator.rs` 第 1105 行 `protected.extend(...)` 没有用例也没有变异行；D23 第 392 行没写「除现行账里仍分配着的槽」与 D28 第 27 行不一致 | 字面那一形没打中（记根、抬 F 两条回收路都按没解开的 C 补隔离，读码） | **采**；`protected.extend` 那一句的用例由删除线第 37 批 X 补了（4 GiB 池上造出、删掉证红）；D23 那一句主 agent 已补（见第一节违规那一行）；键不带跨度那一形进下一批先造盘面、坐实再改 |
| K2 W1 / W2 H1 | — | **打中**（双故障）：R 的根读得出、元数据单元读不出 ⇒ 走读失败、F2 退回 P；R 的记录又全读不出 ⇒ C 与 C_ACCOUNT 都空，影子账只管被抛弃的根、不管 R；M1 写行根落盘之前写掉 R 的 14 份单元，池级 checker 红 I-2.1、I-4.8、I-7.2、I-7.4（ow 每个 k 22 格中 9、sw3 中 10）；根槽与全部记录都藏（R 在 M1 那一刻整个看不见）那一形也写掉，判别子观测不到 | **坐实**（复跑原样同上）。进下一批照攻方 `C_NEWER_ROOTS`（C 并上根环里比所选那一版新、按实例表有效、读得出的根自己的账）加 `C_MAPPING`（账读不出时照被抛弃根那一份从中央映射现算，收窄成只在那次发布读得出的记录不齐——缺序号或缺末条——时才现算、才拒）。R 整个看不见那一形不修（判别子观测不到），记欠账 |
| K2 W3 H2 | — | **打中**（双故障）：F1 按 F2 退之前的水位判，退到 P 之后不再判，按 D23 已定项 14 字面该拒的 12 格都挂上、之后 R 的数据单元被写掉；F1_ANY 本身没打中（单份坏 182 格 0 拒 0 写掉） | **坐实**。进下一批照攻方 `F1_AFTER_F2`：F2 退了之后按退到的那一版再判一次 F1 |
| K2 W2 H3 | — | **打中**（三故障）：缺末条、账上节点读不出 ⇒ C_ACCOUNT 什么都不加交 F1，F1 只看读得出的记录点名的数据单元、不拒（根藏 12 格、根读得出另 12 格） | **坐实**。随 `C_MAPPING` 收窄那一形修（攻方量到三臂合用 0 格，照被抛弃根那一份不收窄会误拒入库 7 条） |
| K1 / K2 W4 H4 | 一致；规则没说两处：c_见证 落在所选那次发布自己的某条记录上时不退、退到的根不按实例表滤 | **打中**（双故障）：R 的单元加 P 那次轮换写的较新系统配置槽读回全零 ⇒ c_见证 落到 Pm，恢复、只读挂载、可写挂载三处跳过已确认、单元完好的 P 落到 Pm（4 / 4 组）；F2 字面许可、在用户接受的那一形之外 | **坐实**。进下一批照攻方 `F2_NEWEST`：退到 [见证到的那一版, 所选根) 里读得出、不被所选根实例表抛弃的最新一条根；实例表读不出时只认恰好相等的那条；见证记录读不出照旧不退（攻方量到 4 组都落 P、F2 两份入库用例 11 条全过）。正推那两处「规则没说」随它写定：退到的根按实例表滤；c_见证 落在所选那次发布自己的记录上时所选那一版算见证过、不退。GPU 恢复内核同改 |
| K1 W5、W6 | 一致 | — | **采** |
| K1 W7、V7 变异 | **冲突**：第 3163 行（第 33 批 T「C 选法退成只认带末条标志的记录（last）」）在 C_ACCOUNT 之下存活、与 C_ACCOUNT 变异同打才红；第 3075、3116、3117 三行原文被 V 改没 | — | **坐实**。第 3075、3116、3117 三行删除线第 37 批 X 已重取锚点（它全表 static-check `CHECKED 3257 315 9`）；第 3163 行是取样点不敏感（`.claude/rules/mutation-sampling.md` 第三类），进下一批补一个分得开的取样点（C_ACCOUNT 读不到账、末条读不出的那一形） |
| K1 V1 GPU F2 | 一致（读码），建模差两处：GPU「读错」只建模槽距未对齐、c_见证 = 0 的守卫写法 | 没打中（读码） | **采**；建模差两处随 `F2_NEWEST` 改 GPU 时一并对齐 |
| K1 / K2 V2 H-S1-fold | 一致；规则没说折叠表 | 抽象模型：最终折叠表在 165 888 个字面要判的落点里漏判 96 个、比改前少漏 912 个，这一形 I-2.1 已红；真盘面没造 | **采**；漏判记欠账（只在 I-2.1 已红的坏镜像上，且比改前少） |
| K1 V3、V4 | 一致（读码加独立 CRC 验证 `crc(A‖B) = (crc(A‖0) ^ crc(0)) ^ crc(B)`；并成一个调用点等价） | 没打中（读码） | **采**（根环线 2026-10-08 逐条看过同认） |
| K1 V5、V6 | 一致；`gpu_unit_checks.rs` 里 `lease_card_waiting_for_room` 成死代码 | — | **采**；死代码归根环线删 |
| K3 | 20 份每份一格 | — | **采** |

## 三、去向

1. 删除线下一批 core（用户 2026-10-08 定「四形都修」）：`C_NEWER_ROOTS`、`C_MAPPING`（收窄）、`F1_AFTER_F2`、`F2_NEWEST`（含退到的根按实例表滤、c_见证 落在所选那次发布自己的记录上不退）；第 3163 行补取样点；键不带跨度那一形先造盘面；`reuse_after_raising_the_floor.rs` 第 1164 行那条同形用例期望改成不红（主 agent 单跑红在第 1177 行「抬 F 之前恰好 I-7.4 红：[]」）。
2. 删除线下一批 GPU：恢复内核照 `F2_NEWEST` 同改，对齐建模差两处；改完主 agent 上卡跑对拍文件。
3. kb：D23 已定项 14 写进四条改法；D28 已定项 1 第九项随 C 的集合（加较新可读根的账、映射现算那一份）改；`.claude/kb/checks-owed.md` 第 735 行旧用例名改新名；新立欠账：R 在 M1 那一刻整个看不见（根槽与全部记录都读回全零）时它的单元没人护（判别子观测不到）、V2 最终折叠表漏判 96 / 165 888。
4. 不做：R 整个看不见那一形的修法（观测不到判别子）。

四条改法都只在攻方模型上量过、被攻过零轮；C694 / F2 这一题已过设计两轮、代码两轮，照 `.claude/rules/three-way-inference.md`「第三轮之后停」，零轮改法交了用户。

## 四、交用户

| 岔路 | 用户定案 |
|---|---|
| 多故障四形（H1–H4）拍 E171 最终冻结树之前修不修 | 2026-10-08「四形都修」（候选：修三形、H3 记欠账 / 四形都修 / 先拍树、四形都记欠账；弹窗写明都在单故障承诺之外、改法零轮、代价推的） |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D23（journal 的角色与格式） 已定项 14 | 备料 | 2026-10-08 不受影响：C、F1、F2 再收严四处（第三节第 1 条），下一批写回 |
| D28（挂载期承诺量） 已定项 1 | 备料 | 2026-10-08 不受影响：第九项随 C 的集合扩，下一批写回 |
| D22（单元原子性怎么合成） 已定项 8 | 不影响 | 2026-10-08 不受影响：四形都在单故障承诺之外，用户定修，承诺本身不改 |
