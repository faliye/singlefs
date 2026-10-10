# 删除批第 33 批 S 合入的代码：代码轮第一轮判决（2026-10-07）

<!-- doc-lint:not-numbers K1 K2 K3 S1 S2 S3 S4 S5 S6 S7 S8 E1 E2 E4 E6 E7 F1 F2 F3 Z1 Z2 Z3 F-KEY F-ID-b F-W H-b H-V -->

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / diff / 背景材料 | `research/prompts/_m3-batch33s-code-r1-body.md`、`_m3-batch33s-code-r1-diff.md`、`_m3-batch33s-code-r1-background.md` | — |
| 开工快照 | `research/prompts/m3-batch33s-code-r1-snapshot/crates-sha256.txt`（18 份）；冻结副本 `~/.local/share/singlefs/work/m3-batch33s-code-r1/tree/`（S 合后、根环线 10x 已合） | 腿跑着的时候主 agent 没改这 18 份；第 34 批 U 只在副本里改、补丁没打 |
| 云端正推（Sonnet） | `research/prompts/m3-batch33s-code-r1-sonnet-output.md`（238 行，sha256 30500483…），模型 `…-sonnet-model/` | — |
| 云端攻方（Opus） | `research/prompts/m3-batch33s-code-r1-opus-output.md`（sha256 c65128ed…），模型 `…-opus-model/` | — |
| 引文 | 两份腿报告 | `cite-check.py --root <冻结副本> --root . --background <背景材料>`：正推「核了 7 处引文，对上 7 处，没判 0 处」；攻方「核了 25 处引文，对上 25 处，没判 0 处」 |
| 主 agent 复跑：正推 | 模型拷进会话草稿目录 `m3b33s-sonnet-model/`，`SINGLEFS_REPO=<冻结副本> SINGLEFS_MAIN_REPO=<仓根> rerun.sh ~/.local/share/singlefs/work/m3b33s-sonnet-rerun`，再 `compare.sh <rerun.out> <expected.out>` | 「相同」；`0.7 guard_model	cases 9918 disagree-with-truth {'old': 1133, 'new': 0}`、`0.8 cmp_mut 头三行	old 3121 new 3152;names added 51 removed 20;changed rows 30;` |
| 主 agent 复跑：攻方 | 模型拷进会话草稿目录 `m3b33s-opus-model/`，在空目录 `~/.local/share/singlefs/work/m3b33s-opus-rerun/` 里 `SINGLEFS_REPO=<仓根> rerun.sh`，再 `compare.sh <summary.txt> <expected-summary.txt>` | `COMPARE same lines=35`；第二节引的行原样都在 |
| 主 agent 现查 | 冻结副本 `crates/singlefs-checker/src/walk.rs` 第 1230–1240 行（每版走完按当时的折叠表把落点折成单元）、第 1304–1316 行（读回第 1 条那一份时改写折叠表）；主工作区 `crates/mutations.tsv` 第 2983 行替换文引 `page_placement`，`grep -n page_placement crates/singlefs-checker/src/walk.rs` 0 命中 | 坐实 |

## 二、逐格判决

| 格 | 正推 | 攻方（一次抽样） | 判决 |
|---|---|---|---|
| K1 / K2 S1 allrbmap 键 | 一致：CPU、GPU 六把键同取读回落点；合法状态判定集合不变（读码）；一形没有坏镜像钉：两份都读不出、第 0 条落点早被走过（改前判「别的版本先走过」，今天 `NotWalkable`，只在 I-2.1 已红的镜像上） | **打中 H-S1-fold**：I-3.5 折叠表在走读中途被改写，CPU 每版走完就按当时的表折成单元，GPU 判时按最终的表认；txg 5 那张树表里 extent 根指针第 0 条位置条目校验和坏时 CPU 多红 I-3.5、GPU 不红；放开用户步骤扫三种历史 12 格，CPU 4 格红、GPU 取法 0 格；CPU 自己随根环次序翻面 | **坐实（坏镜像上的 CPU / GPU 分叉，I-2.1 已红）**：复跑原样 `ATTACK_ARM arm=A-newer-version-pointer-garbled lazy_fold=false red=["I-2.1", "I-4.8", "I-7.4", "I-3.5"]` 与 `… lazy_fold=true red=["I-2.1", "I-4.8", "I-7.4"] i35=holds`，扫描 `ATTACK_SCAN … snapshot_i35=violated lazy_i35=holds` 4 行；`KNOWN_BAD_LAZY_FOLD test result: ok. 203 passed`。按条款字面 I-3.5 判的是落点，GPU 的结局对；进下一批照攻方 F1（CPU 判 I-3.5 时按全部走完之后的折叠表取每版的单元，零轮） |
| K1 S1 第 25 批 H-b、第 28 批 H-V、第 30 批 F-W 结局 | harness `checker_known_bad_images --include-ignored` 203 passed | 副本上换 GPU 取法跑同一份 203 passed | **采** |
| K1 / K2 S2 守卫 | 一致：守卫与真值 9918 格逐个相同（改前 1133 格不同） | 没打中：20 412 个边界取样 0 处错（改前 1392）；**打中 H-S2-tail**：空段右端那一槽 g（g + 1 写过）按 32 KiB 读，GPU 守卫拒，CPU 读出全零 16 KiB 加 g + 1 那一槽；数据指针第 0 条改指这一槽、校验和取那 32 KiB 的 CRC 时 CPU 判对得上、红 I-1.6 / I-2.4 / I-7.2，GPU 推红 I-2.1；Z1、Z3 都碰不到，往下读的路径与恢复内核的读经同一个 `read_at`（推的） | **坐实 CPU 一侧**（复跑原样 `ATTACK_ARM arm=S2-crc-of-zeros-then-next-slot lazy_fold=false red=["I-1.6", "I-2.4", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.2", "I-7.4"]`），GPU 一侧读码推的、要上卡坐实。正推第 3 条（GPU `read_at` 只读起点是落点或整段在空段里的区间，「16 KiB 节点加下一槽」「空槽加下一已写槽前 16 KiB」两形 CPU 判对得上、GPU 判读不到）是同一族、不是 S 引入。进下一批：`read_at` 遇到跨空段终点的读，GPU 按第 32 批攻方 Z2（拿下一个落点那一版的检查字与全零段的 CRC 拼出整段的 CRC）判，做不出的那一支记判不了、回落 CPU；三形各一条对拍流 |
| K1 S3 P2 | 一致：三处调用两处与合成前同义、一处是用户定的改动；两份都没自证过 CPU、GPU 同判 | 没打中（读码） | **采** |
| K1 S4 P3 | 一致：E1、E2、E4、E6、I-4.9 各有坏镜像，变异下点名那条红 | — | **采** |
| K1 / K2 S5 删除流进放量与 GPU | 一致：三处 oracle 判法逐支同；空表时复用键不变；`JUDGE_TABLES_VERSION` 与判法闭包一起变 | 没打中：GPU oracle 对 CPU oracle 穷举 367 416 格 0 处不同，复用键截行 898 128 格不改判定 | **采**（复跑 `S5_ROW name=cpu_vs_gpu_oracle cases=367416 disagreements=0 first=None`）；正推测试尖度一处进下一批：GPU 对拍用例 `…deleted_flow_against_a_version_table_without_a_file_from_the_intent_on` 加断言 CPU 一个违例都没有（今天写错身份时 CPU、GPU 一起判违例仍一致、用例仍绿） |
| K1 S6 C744 ① | 部分一致：流有；「两种版本同在」的状态存不存在没被断言；流不是 kb 字面写的那一条 | — | **采**，进下一批：那条流加断言「两种版本同在」的状态至少一个 |
| K1 S7 钉值 | 一致：现量 320015、382259、337872，过 naga 校验 | — | **采** |
| K1 S8 变异表 | **冲突**：第 2983 行（第 10k 批攻方 H-b）原文仍恰好命中一次，替换文引 S 删掉的 `page_placement`，套用后编不过（`error[E0425]: cannot find value page_placement in this scope`），门禁会记无效、判红；拿 S 删掉的每个标识符扫全表替换文只有这一行中 | — | **坐实，主 agent 已改**：替换文换成 `if !self.visited_units.insert((pointer.locations[0].device, pointer.locations[0].slot)) {`（照原意：「走过」按第 0 条位置条目认），`prove-red.sh` 点名那一条「抓到」（日志 `~/.local/share/singlefs/work/m3-batch33/row2983-copy/prove-red-logs/001.log`）；改前副本 `~/.local/share/singlefs/work/m3-batch33/mutations-before-row2983-fix.tsv` |
| K3 | 18 份各一格：17 份一致或部分一致，`crates/mutations.tsv` 冲突（同 S8，已改）；放量用例（`crash_amplification_of_the_unlink_after_the_multi_record_publish_stream_is_clean`）没编没跑，归提交时 | — | **采** |

## 三、去向

1. 删除线下一批（walk.rs 与 GPU 判器，和 U 之后的那一批并簇）：H-S1-fold 照攻方 F1 改 CPU、GPU 那一侧不改，补 A 臂对拍流（预期 CPU、GPU 都不红 I-3.5）与变异；H-S2-tail 与正推第 3 条那两形：`read_at` 跨空段终点照 Z2 判、做不出的记判不了回落 CPU，三形各一条对拍流与变异；正推第 4 条那一形（两份都读不出、第 0 条落点早被走过）补坏镜像；删完那条对拍流加「CPU 无违例」断言；C744 ① 那条流加「两种版本同在的状态至少一个」断言。F1、Z2 被攻过零轮。
2. 治理欠账（记进 `records/2026-09-16-subagent拆分提案.md`，归治理那一批）：实现员按名字整行换锚点时只核原文命中，不核替换文里引的标识符还在；`crates-mutation-rows.py static-check` 判不出这一类（原文命中一次就绿），要么实现员交回前「拿删掉的每个标识符扫全表替换文」，要么静态检查认出替换文引了源码里已没有的标识符。
3. 不做：攻方「误红 / 漏判归哪一类」——两处都只在 I-2.1 已红的坏镜像上，归类不改去向。

## 四、交用户

不交：打中的两处都有腿给的改法，没有岔路。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 16 | 不影响 | 2026-10-07 不受影响：删除流进崩溃放量与 GPU 照第 4 条读法，两路 oracle 穷举 0 处不同 |
| D8（核心索引结构） 已定项 6 | 不影响 | 2026-10-07 不受影响：打中的是 checker 认单元身份用哪个时刻的折叠表、GPU 读跨空段终点的区间，内部条目形态不改 |
