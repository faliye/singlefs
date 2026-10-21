# m4-c860-r1 判决：扫描重建认「删除中」凭哪一侧（主 agent，2026-10-20）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 -->

正文 `research/prompts/_m4-c860-r1-body.md`，背景材料 `_m4-c860-r1-background.md`（sha256 0293d539c65fa94fd48afd13f3ec139a630eb9f3b946bb1e51c8d78dc24c517c），开工快照 `m4-c860-r1-snapshot/`（`kb-sha256.txt` 与 `tree-sha256.txt`，后者记被判的四份代码：`rebuild.rs`、`walk.rs`、扫描重建内核、那份 harness 用例）。三份腿报告：正推（Sonnet）`research/prompts/m4-c860-r1-sonnet-output.md`（272 行，sha256 38bbc0e47200f8747fff8bb92b518cf8dd294d2d39eb064c544b83db1ae61e97）；攻方（Opus）第一次抽样 `m4-c860-r1-opus-output.md`（205 行，sha256 ee174484d387062611ceb22fa7eeb73970fb17ad3fe98afb37911c97e0988ea9）；攻方第二次抽样 `m4-c860-r1-opus-s2-output.md`（sha256 3254bd569817f11a566a88b032b5ead49fd0b8ae53e214cc71f1be8f5e29c33c）。第二次抽样照「没打中要再抽一次」派，禁读第一次的报告与模型。

## 一、核查

- 快照：`tree-sha256.txt` 四份全部 OK。`kb-sha256.txt` 里 `.claude/kb/decisions/08-核心索引结构.md` 对不上，是开工后别的会话改的（三条腿都报了）。判决要用的那几行现查还在：第 484 行「1. **意图**：意图那次发布把 inode N 的记录写成 nlink = 0」，第 486 行「它是派生态：队列 = inode 树里 nlink = 0 的号集合」。
- cite-check（`--root` 仓根、`--background` 背景材料，三份报告一起）：`✓ 核了 37 处引文、19 处裸行号，对上 37 处，没判 19 处（其中裸行号没有引文可比 19 处；逐处列在上面）`。判决要用的裸行号逐处现查过：`crates/singlefs-checker/src/rebuild.rs:777`「if tombstoned_objects.contains(&(inode, object_birth)) {」，`crates/singlefs-checker-tier/src/crash_judge_checker_scan_rebuild.wgsl:1347`「if (scan_rebuild_record_is_of_an_object_being_deleted(sw(row + 2u), sw(row + 3u))) {」，`crates/singlefs-harness/tests/scan_rebuild_reads_lineage_and_retirement_records.rs:244`（不出现「inode 记录集合」与「每个 inode 的数据单元集合」那句断言），`crates/mutations.tsv:4958`（「扫描重建 M5 退役记录：认删除中的对象那一判恒不命中…」），`crates/singlefs-checker/src/walk.rs:14801`（`INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED`）。
- 复跑：攻方第一次抽样的模型目录拷到 `~/.local/share/singlefs/work/m4-c860-r1-verify/`，冻结副本取主工作区的拷贝（四份被判代码的 sha256 与快照相同），经 16G 包装跑 `tools/rerun.sh`，exit 0：`name=rerun_table_diff_lines 0`、`C860_RANDOM_SUMMARY histories=124 steps=30 states=3824 states_with_an_object_being_deleted=42 disagreements=0`、`C860_RANDOM_SUMMARY histories=155 steps=30 states=4780 states_with_an_object_being_deleted=8 disagreements=0`，与报告第 82、83 行逐字相同。攻方第二次抽样与正推的模型没复跑（各要现编一份 checker 档或 RocksDB）；它们的数在判定里标「腿报的」。
- 主 agent 补量（三条腿都没量）：在复跑那棵树里（攻方补丁带读法开关 `C860_READING`），harness 与池级 checker 全部测试两趟。`C860_READING=0`（甲）2244 过、0 败、80 ignored；`C860_READING=2`（攻方的丙）2242 过、2 败：`a_queue_number_whose_inode_record_has_nlink_one_reddens_the_deletion_queue_invariant`（`crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs:1001`，期望 `["I-9.5", "I-9.19"]`、实际多一个 `I-1.13`）与 `erasing_the_retirement_record_reddens_the_scan_rebuild_after_the_right_leaf_retired`（第 336 行那句按甲写死的断言）。日志 `~/.local/share/singlefs/work/m4-c860-r1-verify/run/readings-{0,2}.log`。
- 两腿编号不同要先对齐：攻方的丙是「乙，再加两侧那条记录 nlink 都是 0」；正推的丙是「两侧都不去记录、只去单元与映射」，正推的丁是「遍历方向按 nlink = 0 自己认、两侧集合要相等」（正推报告第 76–90 行）。下文的甲、乙、丙一律按攻方的定义。

## 二、判定与去向

| 格 | 判定 | 依据 | 去向 |
|---|---|---|---|
| K1 候选认法 | 三条腿都写清了甲、乙，攻方两次抽样各自提出丙。另有一处各读法共用的：扫描一侧被墓碑压着的对象，数据单元一律不认领，不看记录在不在、nlink 多少（`rebuild.rs:777`） | 三腿 | 交用户第 1 行 |
| K2 合法盘误红 | 三次抽样都没打中：甲、乙、丙在合法盘上零误红。攻方第一次：17 张固定镜像加 279 段随机历史（8604 个状态，50 个带删除中对象，复跑逐字相同）；第二次：14 张固定镜像、10 种比重 × 124 段 × 40 步共 50697 个状态（193 个带删除中对象）、一段回退的 65548 个崩溃状态（腿报的）；正推：14 个合法世界、1736 个随机历史状态（腿报的）。缺口：删除写流本身的崩溃中间态没枚举（第二次抽样估一段历史 6.7×10⁷ 到 6×10⁸ 个状态） | 三腿；主 agent 复跑 | 用户已定放量枚举（见第四节） |
| K2 已有用例 | 乙、丙都让第 336 行那句断言转红（正推量了乙、主 agent 量了丙）；丙另让 `a_queue_number_whose_inode_record_has_nlink_one…` 那条多红一个 I-1.13。两条都是断言按甲写死，不是读法误红：nlink 1 的活记录压着墓碑，I-1.13 跟着红是这条不变量该有的判别力 | 正推第 144 行；主 agent 补量 | 改法里两条断言跟着改 |
| K3 坏盘放过 | 攻方两次抽样各自打中同两形。（一）C860 镜像：甲判成立，乙、丙判违例；同一镜像上 I-9.35、I-3.9、I-3.11 红（三腿一致）。（二）墓碑坏了或多一条、压在活文件上：甲、乙都把这个活文件整个移出比对、判成立，丙判违例；第二次抽样再叠上 M5 必红世界那一招（挪那个文件一条映射的落点），甲、乙下 I-1.13 仍成立、丙红（腿报的）。这一形另由 I-9.18、I-9.19、I-9.20 兜住 | 攻方两次；正推第 165–178 行同形，表述是「由别的不变量拦」 | 交用户第 1 行 |
| K4 文意 | 三腿一致：M5 原句与 I-1.13 定义句都没写由哪一侧认；甲、乙照字面都过。丙把括注「墓碑压着的对象还有记录」收窄到 D8 已定项 16 第 1、3 条那一态（nlink = 0）。M5 必红世界在三种读法下都红，乙、丙不改它的判法（攻方两次都量过） | 三腿 | — |
| K5 改法与代价 | 乙：`rebuild.rs` 扫描一侧去记录那一步挪到 `walk.rs` 遍历方向那一循环，GPU 内核那两个函数同步，约 20 行（推的）。丙：乙之外两侧各核 nlink = 0，约再多 50 行，多半是新用例（推的）。都要改第 336 行与 `pool_checker_judges_the_deletion_state…:1001` 两句断言（丙两条、乙一条），C860 新用例去掉 `#[ignore]`，`mutations.tsv` 第 4958 行随读法改写判别力 | 三腿；主 agent 补量 | 交实现员 |

## 三、改法（被攻过零轮）

- **C1 乙**：扫描一侧只在遍历方向也有同号、同对象出生代的记录时才算删除中；两侧都去掉它的记录、单元与映射。修 K3（一），不修（二）。
- **C2 丙**（攻方两次抽样各自提出）：C1 之外两侧那条记录 nlink 都要是 0（D8 已定项 16 第 1 条意图写 nlink = 0，第 3 条队列 = nlink = 0 的号集合）。修 K3（一）、（二）两形。被攻过零轮：只在攻方的模型与主 agent 补量的那两趟 harness 上量过；改完之后的代码轮攻它，删除写流崩溃中间态的放量枚举也判它。

## 四、交用户

岔路单 `research/prompts/m4-c860-r1-forks.md` 一行：甲、乙、丙三选一。主 agent 的推荐是丙：三次抽样里乙、丙在合法盘上都零误红，丙多修 K3（二）那一形；代价多的部分是新用例，已有用例只多一句断言跟着改。不选乙的理由：一条坏墓碑能把一个活文件整个移出 I-1.13 的比对，连 M5 自己的必红世界都被遮掉（第二次抽样量的，主 agent 没复跑）；别的不变量兜住这一形，但 I-1.13 自己的判别力在这一形上为零。

用户 2026-10-20 已定：删除写流本身的崩溃状态要放量枚举。放在读法定案、CPU 与 GPU 都改完之后，走登记的崩溃放量全量（删除相关 8 个分项，两台、GPU、写库两段），开跑前先数状态数、报预计挂钟。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 16 | 不影响 | 2026-10-20 不受影响：只引它第 1、3 条对「删除中」是 nlink = 0 的那一态的定义，条款不改 |
| D18（块里携带什么信息） 已定项 24 | 不影响 | 2026-10-20 不受影响：墓碑每个对象一条照旧，这一轮只改池级 checker 怎么用墓碑认删除中 |

## 用户定案

2026-10-20 弹窗，用户选「丙：两侧都要有记录且 nlink 为 0 (Recommended)」。照第三节 C2 改池级 checker 与 GPU 内核，两条按甲写死的断言跟着改，C860 新用例去掉 `#[ignore]`。改完之后的代码轮三方攻它，删除写流崩溃中间态的放量枚举判它。
