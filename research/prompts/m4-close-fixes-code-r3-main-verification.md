# m4-close-fixes-code-r3 判决：r2 判决第四节与 P2a，第三轮，最终轮（2026-11-09）

<!-- doc-lint:not-numbers T1 T2 T3 T4 P1 P2 P2a P3 B1 B4 B8 F1 F2 Q5 Q6 Q8 -->

被判的文件（开工快照 `research/prompts/m4-close-fixes-code-r3-snapshot/crates-sha256.txt`）：`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/compaction_plan.rs`、`crates/singlefs-core/src/instance_table_sweep.rs`、`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/mounted_session.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/transaction.rs`。

## 一、核查

- 腿：辩方 `research/prompts/m4-close-fixes-code-r3-sonnet-output.md`，攻方 `research/prompts/m4-close-fixes-code-r3-opus-output.md`。两腿各抽一次，「没打中」不支撑结论。
- 攻方模型复跑任务 11382：`name=attack_fixes_r3_summary target_as_expected=yes`，观测 `~/.local/share/singlefs/work/m4-close-fixes-code-r3-opus/rerun-1/observations.txt`。下面引的读数都是这一趟的；模型是腿的副本装置，读数不进 kb。
- 这是这一题第三轮，照 `.claude/rules/three-way-inference.md` 之后不再开轮；新冒出的零轮形态进第四节。

## 二、逐格判

| 格 | 判 |
|---|---|
| 「按字面全接上」 | 打中（两腿一致，模型量实）：`rotation_read reads_of_the_damaged_copy=2 other=0 outcome=NotNeeded still_damaged=true`（`transaction.rs:679`）；`floor_raise … reads_of_the_damaged_copy=4 other=0`（`mount.rs:1767`，`mount.rs:5484` 会话抬 F 也走）。实现员「写入口够不着账」的依据不成立（`transaction.rs:376` 已有同形的账）。`mount.rs:1809`、`:1998`、`:2170`、`:2241`、`:2525`、`:2826`、`:3128` 攻方与辩方只核了一部分。水位那一读（`watermark_read rolled_back=true reads_of_the_damaged_copy=1 other=1`）已接，打中的是没有变异行 |
| P2a 多写 | 没打中（读码、一次抽样）。量到的：测量补丁上 15 个点不动点逐个相同，产品代码上 1 条历史 |
| P2a 仍撞 64 | 机制量实、可达没量实：改空闲图造的盘上 `fallback sweep` 185 次里 5 次 `outcome=did_not_settle_limit_64`（每片叶 2 个空槽）；用户操作那一段 `natural` 写到 `NoSpaceAfterRaisingTheFloor` 没走到这个形状。进第四节。`transaction.rs:13773` 注释「全部取样点」宽了，只有第一段 16 个点 |
| 映射 key 不等就拒 | 拒合法的盘没打中：`legal_disks histories=768 roots_judged=47062 key_set_refusals=0 other_refusals=0`（一次采样，没枚举崩溃状态）。key 集合相等、位置对换的盘照收（`swapped_positions mount=accepted read=ok same_as_the_file=true`，读对），进第四节 |
| 随机历史 8 条分歧 | 未判：`endings_not_completed=8`，`ModelDisagreement` 7 条 `根带的 F`（`OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable`）、1 条 `会话状态`（`PublishOverwrite`），`endings_naming_the_key_set=0`。是不是这批或别的会话带来的没查，派调查员，判决不引它下结论 |
| 六处接了线没变异行、写行两处盲区、用例文档点名的行不在表里（`unit_read_…rs:2149`、`:2229`）、改名 8 行与锚点 4 行没证红 | 打中（轻）。现读过 `:2149` |
| 账与 D4 第 73 行 | 打中（轻）：`mounted_session.rs:228`「再记、再试一次」与 D4 第 73 行字面对不上，kb 写回 |
| 未定项清单 | 打中：r2 判决第五节要登的攒批在 `.claude/kb/checks-owed.md` 没有（现查），两腿列了 10 到 16 条没去向。收进第五节 |
| 数字 | 打中（轻）：「变异都证过红」窄了 12 行；任务 11334 退 1 正文没写（缩域判不绿，用例本身 `violations=0`）；`29 轮` 的日志随副本删了，引文在 `records/2026-10-13-里程碑四预开发.md` |
| 检查单 | 走不到、类型与名字、不兼容、判定库、多机、多线程成立；带 `--item` 不成立（上一格）；新崩溃流推不出算打中：坏盘 × 部分落盘只判 `recover`，可写挂载那半没判 |

## 三、交实现员（最后一批）

1. 读接全：`transaction.rs:679`（写入口轮换那一读，读到的片随下一批换）、`mount.rs:1767` 一族，穷举 `mount.rs:1809`、`:1998`、`:2170`、`:2241`、`:2525`、`:2826`、`:3128`，读别的版本、无东西可换的只计数不搬，报告逐条写。
2. 变异行：六处接线、写行两处、`wgsl:663` 之外本批新增的；`unit_read_…rs:2149`、`:2229` 文档点名的行补上或改对；改名 8 行、锚点 4 行证红。
3. 注释写实：`transaction.rs:13773` 一带「全部取样点」、`allocator.rs:1867` 悬空依据、`allocator.rs:1986`「都走它」。
4. 用例：坏盘 × 部分落盘之后接着可写挂载。

## 四、交用户（零轮或条款空白）

| 项 | 量到的 | 状态 |
|---|---|---|
| 固定点上界 64 与回落形状 | 只在造的盘上撞（5/185），`natural` 没走到；U 从 2¹⁴ 到 2²⁴ P2a 轮数都是 4（E206 报告）。候选：撞上界报成空间不够让会话推抬 F；回落时取落点优先已在重写集里的叶。都零轮 | 登欠账或另立题 |
| key 位置对换的盘 | 收下、读对 | 零轮 |
| B8 的判据 | 峰值来自一次整文件替换，正比于这次改动（`~/.local/share/singlefs/work/e206-b8-data-units-peak/result.md`）；按存储大小也超标（步数/U 24.65）。候选：判据改成按这次碰到的单元算，是新一轮，要写修订 | 要用户定 |
| P2a 的净步数 | 小点多 936 到 1692 步、5 个点 `costs_more`，大点净省（`files_30000` 26.6 亿）；登记 Q5 没写「净多」按点还是按全部判 | 要用户读 |
| 六个读别的版本的读 | 计数够得着、搬够不着，D4 没写 | 随第三节第 1 条，只计数 |

## 五、延后（去向）

- 攒批会话读不到待发写：登 `.claude/kb/checks-owed.md`（kb 写回那一批）。
- 重试尾巴、实现员自定的 `RefusedNotForSpace`、GPU 与 CPU 差的一形（`crash_judge_recovery.wgsl:846`）、多片实例表链用例：登同一处。
- kb 写回（D4 第 71、73 行，C946，映射 key 拒判，P2a 定案）在第三节那一批之后。
- 不再开第四轮。

## 这一轮被判过的文件

开工快照 `research/prompts/m4-close-fixes-code-r3-snapshot/crates-sha256.txt` 登的七份，这一轮攻 r2 判决第四节与 P2a 的修补在它们里的改动，逐格判见「二、逐格判」：

- crates/singlefs-core/src/allocator.rs：r2 判决第四节与 P2a 的修补在这份里的改动，随「按字面全接上」与 P2a 两格一起攻
- crates/singlefs-core/src/compaction_plan.rs：r2 判决第四节与 P2a 的修补在这份里的改动，随 P2a 两格一起攻
- crates/singlefs-core/src/instance_table_sweep.rs：r2 判决第四节与 P2a 的修补在这份里的改动，随 P2a 两格一起攻
- crates/singlefs-core/src/mount.rs：抬 F 与会话抬 F 读坏副本的入口（`mount.rs:1767`、`mount.rs:5484`）、映射 key 不等就拒
- crates/singlefs-core/src/mounted_session.rs：账与 D4 第 73 行「再记、再试一次」（`mounted_session.rs:228`）
- crates/singlefs-core/src/recovery.rs：r2 判决第四节与 P2a 的修补在这份里的改动，随「按字面全接上」一格一起攻
- crates/singlefs-core/src/transaction.rs：轮换读坏副本（`transaction.rs:679`）与回退扫描取样点（`transaction.rs:13773`）

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D4（校验和位置） 已定项 4 | 不影响 | 2026-11-09 不受影响：字面不改，kb 写回照实现 |
| D21（权威态与派生态的分界） 已定项 10 | 不影响 | 2026-11-09 不受影响：映射 key 那一判照用户定案实现 |
| D8（核心索引结构） 已定项 16 | 不影响 | 2026-11-09 不受影响：B8 判据交用户，条款没改 |
