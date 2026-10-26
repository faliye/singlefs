# m4-segment-code-r2 判决：里程碑四预开发整段代码轮第二轮（主 agent，2026-10-16）

<!-- doc-lint:not-numbers R1 R2 R3 R4 R5 R6 R7 R8 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 F1 F2 F3 F4 F5 F6 F7 F8 F9 F10 H1 H2 H3 H4 H5 H6 G3 G4 G5 S3 -->

被判的产品代码与第一轮同一组 20 份（冻结副本 `~/.local/share/singlefs/work/m4-segment-code-r2/repo/`，sha256 在 `m4-segment-code-r2-snapshot/crates-sha256.txt` 21 行），这一轮改过的是 `crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/mounted_session.rs`、`crates/singlefs-core/src/compaction_plan.rs`、`crates/singlefs-reference-filesystem/src/lib.rs`。正文 `research/prompts/_m4-segment-code-r2-body.md`，背景材料 `_m4-segment-code-r2-background.md`，复核的判决 `research/prompts/m4-segment-code-r1-main-verification.md`。

开工快照：`research/prompts/m4-segment-code-r2-snapshot/`

## 一、核查

- 快照：两条腿各自在冻结副本根上 `sha256sum -c` 21 行全 OK（两份报告开头原样）。腿跑着的时候主 agent 只往工作树打了测试代码与 checker 档的补丁（修复批二丁、丙二），被判的 20 份产品代码文件没动。
- cite-check（以工作树为根）：辩方 `✓ 核了 65 处引文、29 处裸行号，对上 65 处，没判 29 处（其中裸行号没有引文可比 29 处；逐处列在上面）`；攻方 `✓ 核了 37 处引文、6 处裸行号，对上 37 处，没判 6 处（其中裸行号没有引文可比 6 处；逐处列在上面）`。
- 复跑（各在新的草稿目录里，经 `run-with-memory-cap.sh 8G`）：辩方 `rerun.sh` → `name=rerun_total same=10 differ=0`（`~/.local/share/singlefs/work/m4-segment-code-r2-main-rerun-defense.log`）；攻方 `rerun.sh` → `name=rerun_total same=15 differ=0`（`…-main-rerun-attack.log`）。
- 主 agent 现查的行：`.claude/kb/decisions/23-journal的角色与格式.md` 第 378 行「**管理员回退是挂着时的一次向前发布**」那一段在（辩方 R1 依据）；`crates/singlefs-harness/src/model.rs` 第 2750 行一带回退那一支 `relocations_carried.extend(` 在（攻方 R1 根因的锚点）。

## 二、判定与去向

| 格 | 判定 | 依据 | 去向 |
|---|---|---|---|
| R1 回退之后的映射 | F1 站得住：49 个回退场景第一轮核心 24 格红、108 条违例，修后 0 格（辩方，量的）；写死形状 240 格、随机历史 2924 段没红（攻方，量的）。判决 F1 字面「中央映射树取目标版的」与 D23（journal 的角色与格式） 第 378 行那一段（回退不从 R_old 那棵账载入）冲突，F1 现在的取法（映射树照现行版 COW、位置与目标版不同的 key 所在路径重写）与 D23 相容、仍守 G3「映射是唯一权威」（辩方）。**打中（模型一侧，两腿独立）**：整理搬过根单元之后回退，理想模型把现行版「最后一次搬它」的记录并进回退那一版（`model.rs:2750` 一带），抬 F 上限算得比实现低——攻方 4 GiB 盘 310 段中 3 段、小盘 496 段中 12 段，改模型那一处后 0 段；辩方种子 950093 第 18 步 `RaiseRollbackFloor` 模型 9、实现 11 是同一处 | 两腿；复跑 `model-rollback-relocations-before-and-after.txt`、`f1-r2-core.txt` 相同 | H1；判决第一轮 F1 的读法照 D23 改写（H6，归阶段同步） |
| R2 腾空 R 与删意图 | F2 站不全：**打中（两腿）**——计划里要搬的单元全是这次发布本来就重写的节点、一个没搬而 R 还不空时发出「没搬、意图照抄」的一批，模型判不合（辩方种子 940015、940105、960004；攻方种子下标 297、6318；第一轮核心上同样复现）；**打中（攻方，后果轻）**：一份副本连读两次读不出、下一遍又读得出时，两条「核不过就照抄意图」的退路在产品代码里走得到，代码注释「只有测试开关才走得到」不成立；「最后一批删意图核不过、照抄再发」一支没有强制用例（辩方） | 两腿；复跑 `seeds.txt`、`mount-transient-reads.txt` 相同 | 用户 2026-10-16 定「规划时补足」→ H4；退路与注释 → H2 |
| R3 `segs0` | 攻方没打中；辩方：F3 照 D26（后台整理与放置回收） 字段表字面，把停机谓词「全局差值」的基线挤低一格（`the_completed_intent_yielded(3310, 3310)=false`、`(3310, 3309)=true`） | 辩方 T6 一节 | 用户 2026-10-16 定「取签发之前」→ H3；D26 字段表那一句归阶段同步 |
| R4 强制开关 | 站得住（两腿）；辩方：开关挂写入口，只有直接造写入口的测试够得着 | 两腿 | 无 |
| R5 空池快照 | 核心与模型 115 格对上（攻方），F5 探针第一轮核心 6/6 不合、修后全绿（辩方）；RefFS 对「空池上建快照」与核心对「删完文件之后」两形口径不同、不误判（攻方） | 两腿 | 无 |
| R6 RefFS 允许集合 | F6 翻译层 20 行探针没找到核心能落到而 RefFS 判不允许的形状（辩方）；**打中（攻方，RefFS 单元层，翻译今天喂不到，推的）**：恢复读出的内容对不上任何允许状态时，快照集合退回最初那一份，之后建的快照记成没有、读它判红 | 两腿 | H5 |
| R7 开着的形状 | 「一个没搬而 R 还不空」造出（即 R2 第一处）；`ckpt_cost` 余量 528–534 槽越不过（攻方，量的），默认几何罩得住（辩方，算术）；零个全空段、合法镜像触发 `TwoRecordsMovableUnderOneMappingKey` 两腿都造不出合法镜像 | 两腿 | 后两形与更大池的 `ckpt_cost` 上界记欠账 |
| R8 第一轮没打中的六格 | 攻方换小盘、新比重、瞬时读错三面，T1、T2、T3、T7、T9、T10 没有新的红；连同第一轮，两次抽样都没打中 | 攻方 | 记「没打中」（两次） |
| 第一轮判决复核 | T4 坏在判据、F9 够得着（同一实现同一段历史第一轮 checker 红、修后绿，六个必红与已知坏镜像目标 309 条全绿）；T7、T8、T10 三处不改代码的判定站得住（依据 `.claude/kb/decisions/03-空间分配.md` 第 146 行、G5 字面不要求补发、可写挂载拒成 `UnitMalformed` 而冷启动读得出同一条根）；F4、F5、F6、F7 站得住 | 辩方 | 无 |

两腿分歧：没有。R1 辩方判「F1 站得住」、攻方打中的在模型一侧，说的是两处；R2 的「一个没搬而 R 还不空」两腿各自造出。

## 三、改法（都被攻过零轮，进修复批三，第三轮攻它们）

- **H1**（`crates/singlefs-harness/src/model.rs`，测试代码）：回退那一支不把现行版的搬根记录并进回退那一版，只取目标版的。验收：攻方 4 GiB 盘与小盘那两批随机历史 0 段不合；辩方种子 950093 第 18 步模型与实现同为一个数；变异一行证红。
- **H2**（`transaction.rs` / `mounted_session.rs`）：两条「核不过就照抄意图」的退路照实写注释（瞬时读错走得到）；用故障注入的瞬时读错强制走进两条退路各一条用例（含「最后一批删意图核不过、照抄再发」那一支），断言之后的整理照常收；变异各一行证红。
- **H3**（用户 2026-10-16 定「取签发之前」）：意图里的 `segs0` 记签发那次发布之前的全空聚簇段数；停机谓词「全局差值」相对它比。核心、模型同改；用例：签发那次发布开新段时，意图要真多腾出一段才算有产出；变异一行证红。D26 字段表那一句归阶段同步。
- **H4**（用户 2026-10-16 定「规划时补足」）：计划挑单元时跳过这次发布本来就重写的节点，补上 R 里真要搬的单元，一批至少搬一个；R 里只剩这次发布本来就重写的节点时照 F2 同发布删意图。验收：两腿五个种子与模型逐版对上；变异一行证红。
- **H5**（`crates/singlefs-reference-filesystem/src/lib.rs`）：恢复读出的内容对不上任何允许状态时，不退回最初的快照集合，按恢复不被接受处置（与读出内容对不上的那一判同一出口）；RefFS 单测钉这一形；变异一行证红。
- **H6**（阶段同步）：第一轮判决 F1 一条的读法照 D23（journal 的角色与格式） 第 378 行那一段改写进 D26 已定项 2 的射程：回退的映射照现行版 COW、位置与目标版不同的 key 所在路径重写。

修复批三另并进的（写段已交下来、不是这一轮的打中）：GPU 判器内核与池级 checker「孩子一律挂父单元」同形那一处（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 挂父单元那一段，丙二交的）；`.claude/gate.d/cpu-judging-sites.tsv` 第 50 行说明的旧落点（丁交的）。变异第 4491 行行名、I-3.5 / I-5.1 条文归阶段同步。

## 四、交给下一批的实现员（按文件聚簇）

| 簇 | 改法 | 文件 |
|---|---|---|
| 甲三 核心 | H2、H3（核心一侧）、H4 | `transaction.rs`、`mounted_session.rs`、`compaction_plan.rs`、`allocator.rs`（H3 要动时）与 harness 用例 |
| 乙三 模型与 RefFS | H1、H3（模型一侧）、H5 | `crates/singlefs-harness/src/model.rs`、`crates/singlefs-reference-filesystem/src/lib.rs` 与它们的单测 |
| 丁三 checker 档 | GPU 内核挂父单元同形一处 | `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 与对拍用例 |

甲三与乙三对 H3、H4 要同口径：派发各写对方的读法，不等对方交回。

## 五、交用户（已定）

| # | 题 | 用户定案（2026-10-16 弹窗） |
|---|---|---|
| 1 | 意图里的 `segs0` 取哪一刻 | 「取签发之前」 |
| 2 | 一批要搬的单元全是本来就重写的节点、一个没搬而 R 还不空 | 「规划时补足」 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D26（后台整理与放置回收） 已定项 2 | 支撑 | 2026-10-16 改了：H3 的 `segs0` 取签发之前、H6 的回退映射读法，归阶段同步写进字段表与射程 |
| D26（后台整理与放置回收） 已定项 1 | 不影响 | 2026-10-16 不受影响：H4 改的是计划挑单元，触发与停机判据不动 |
| D23（journal 的角色与格式） | 支撑 | 2026-10-16 不受影响：F1 的取法照它，条款不改 |
| D5（快照 / 空间记账机制） 已定项 14 | 不影响 | 2026-10-16 不受影响：H5 改的是 RefFS 的允许集合 |
