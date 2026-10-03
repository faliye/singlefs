# 删除批（三·2 unlink）合入的代码：代码轮第一轮正文（2026-10-03）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 K6 L1 L2 L3 L4 L5 L6 L7 L8 F1 F2 X1 W N S -->

## 一、题面

里程碑三 三·2 第 2–8 条（`.claude/kb/milestone/03-third-txn.md`「三　删除（新事务）」作业点表三·2 那一行）由 D-5c2 一线十位实现员接力写成，2026-10-03 一次合进主仓 `crates/`（调度记录 `records/2026-10-01-删除与并发事务.md` 批 8 至批 10）。这一轮判：代码做的是不是条款说的，哪一格会错。

被判的改动全文在 `research/prompts/_m3-unlink-code-r1-diff.md`（60 份文件，冻结副本 `/tmp/claude-1000/m3-unlink-code-r1/tree/`）：

| # | 改动 | 主要落点 | 压着的条款（零轮的标出） |
|---|---|---|---|
| L1 | unlink 单文件：走空 + 意图 + 从尾部分批；意图那次 nlink = 0、`deleted_inodes` 加 key、墓碑开放容器追加；删除批只重写被截掉那一片下段叶与祖先；最后一批删 inode 记录与 key 同一次发布 | `crates/singlefs-core/src/unlink.rs`（第 166 行 `plan_the_extent_tree_after_a_deletion_batch`）、`transaction.rs`、`deleted_inodes.rs`、`tombstone_tree.rs`、`inode_tree.rs`、`records.rs` | D8（核心索引结构） 已定项 5、6、16 第 1、2、7 条；D18（块里携带什么信息） 已定项 24；只改尾叶的 extent 分支与重写角色集合钉绝对值被攻过零轮 |
| L2 | 读者与写入口、删空：`PoolVersion` / `RebuiltVersion` / `PreviousVersion` 第三个成员「有树、文件集合为空」；写入口拒 nlink = 0；删完之后只许 `publish_new_inodes`（size 0）、写行与空发布 | `recovery.rs`、`mounted_read.rs`、`transaction.rs` | D8（核心索引结构） 已定项 16 第 4、5 条（K-U5 X1） |
| L3 | 续做：`mount_writable` 写行与暖机之后把队列走到空（`mount.rs` 第 4320 行 `continue_the_deletion_queue_until_it_is_empty`）；会话里每次用户改动先走空；任何一版 `deleted_inodes` 至多一条 key；崩溃注入 `WritableMountPhase::Continuation` 进 `ALL` | `mount.rs`、`mounted_session.rs`、`crates/singlefs-checker-tier/src/crash_injection.rs` | D8（核心索引结构） 已定项 16 第 6 条；用户 2026-10-01 定案 F1（会话里每次用户改动先走空，第 6 条增补被攻零轮） |
| L4 | 删除类发布被落点那一判以空间拒时推抬 F 再试：会话那一处（`mounted_session.rs` 第 536 行 `publish_the_queue_until_it_is_empty_raising_the_floor_for_space`）与挂载续做那一处；每一次被拒的删除类发布各算一次准入（`mount.rs` 第 2207 行 `PUBLISHES_PER_ADMISSION_AT_MOST = 8`）；推满仍不够，会话交回 `NoSpaceAfterRaisingTheFloor`、挂载交回 `MountError::Publish`，已发成的留在盘上 | `mounted_session.rs`、`mount.rs` | **用户定案，条款还没写回 kb（零轮）**：2026-10-02「推抬 F 再试（推荐）」，2026-10-03「甲 每次被拒各算一次（推荐）」「甲 同样推抬 F 再试（推荐）」（原话在调度记录同日条目）；D16（发布语义） 已定项 1「准入」那一行 |
| L5 | 回退新根挑树：extent、inode、`deleted_inodes` 取 R_old 的，别的种类随现行；回退那次写一条谱系记录；标签 6 的 n_live 按新根现算 | `transaction.rs`、`lineage_tree.rs` | D23（journal 的角色与格式） 已定项 14；D18（块里携带什么信息） 已定项 24 |
| L6 | 准入：环窗口 = (3S + 1) × W、W = 24 + Σ_ℓ min(N_ℓ, 2·(K + 26))，只住内存、每块盘各扣一份（`admission.rs` 第 383 行 `ring_window_on_one_device`）；墓碑预付标签 6；标签 4 恒 0；删除类发布不判式子那一判、落点那一判照判 | `admission.rs`、`write_accounting.rs`、`allocator.rs` | D28（挂载期承诺量） 已定项 1、4；D3（空间分配） 已定项 2、17（用户定案 F2）；D5（快照 / 空间记账机制） 已定项 4 |
| L7 | 墓碑树与谱系树根满长高、只供测试的容量旋钮、读侧 key 宽；池级 checker 照遍历四棵新树、单元算进已有统计 | `container_tree.rs`、`recovery.rs`、`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker/src/lib.rs`、`crates/singlefs-format/src/lib.rs` | D8（核心索引结构） 已定项 6；D18（块里携带什么信息） 已定项 24 第 3 条；D13（验证路线） 已定项 5（checker 不调 `singlefs_core`） |
| L8 | 模型与随机历史：模型删除与删后答「没有文件」；随机历史加 unlink 与建 inode（`history.rs` 第 429 行起）；执行器比较链按「有树」挑版本、续做里推的抬 F 按 txg 排进去；Q-T 那 4 条照今天的结局改钉改名 | `crates/singlefs-harness/src/model.rs`、`history.rs`、`model_comparison.rs`、`reference_filesystem_translation.rs`、`crates/singlefs-harness/tests/random_histories.rs` | `.claude/rules/verification.md`「功能正确性靠模型对拍」；用户 2026-10-03 定案「乙 照现值改钉并改名」「甲 下一棒加建 inode」 |

各位实现员的报告（读法与量）：`/tmp/claude-1000/m3-unlink-body/report.md`、`report-2.md` 至 `report-8.md`。

## 二、实现今天的样子（主 agent 的观测，2026-10-03）

- 冻结副本 `/tmp/claude-1000/m3-unlink-code-r1/tree/`，被判 60 份文件的清单 `research/prompts/m3-unlink-code-r1-snapshot/crates-sha256.txt`（主仓与冻结副本逐文件相同）。
- 合入验证（在与主仓逐文件相同的副本 `/tmp/claude-1000/m3-unlink-body/tree-2` 上，输出 `/tmp/claude-1000/m3-unlink-body/r8-main/`）：harness 档 117 个测试二进制全绿；随机历史慢档 `test result: ok. 15 passed; 0 failed`；`reuse_after_raising_the_floor` 慢档 13 过；clippy 红只在补丁没碰的四份别的会话的文件。checker 档（崩溃注入、层 0、变异整表）归提交时，没跑。
- 已知、不用再攻的：① 合并后 `crates/singlefs-checker-tier/src/crash_injection.rs` 主仓那套 `FloorRaisesInTheMountWrites::of_the_mount_output` 按挂载写表末尾找抬 F 的界，看不见挂载续做里推的抬 F（第十位报告第六节，进下一批）；② `crates/mutations.tsv` 改名后的第 478、481 行在今天的取样点上不红（取样点不敏感，下一批补窄盘用例）；③ 删完之后建 inode、第一个文件不能直接写，是 D8（核心索引结构） 已定项 16 第 5 条与 K-U5 定的，不是缺陷。
- E168（删除借保留量的环窗口） 第一段的装置用的是补丁表的删除路径（每批整片重写 extent 下段），不是 L1 的实现；它的数不拿来判这一轮。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　分批与幂等（L1）
只改尾叶那一支与 K-R3-1 逐字对不对（截在叶边界、截到只剩一个单元、上段那一串）；最后一批三件事同一次发布；D8 已定项 16 第 7 条五条幂等判据各自的用例是不是真判那一条（①b 两份盘逐字节相同、③ 每个写点截断重开）；墓碑容器号与谱系容器号的取号。

### K2　续做、走空与推抬 F（L3、L4）
挂载续做与会话走空是不是同一个函数；「每一次被拒的删除类发布各算一次准入」在续做那一趟里怎么数（实现员写「有一步落了盘就从 0 数」）；推满仍不够时盘上留下的几版是不是条款允许的中间态；`deleted_inodes` 至多一条 key 的拒绝是不是类型化、不归空间不够；崩溃注入 Continuation 那一段认得出续做的写。

### K3　准入（L6）
环窗口与墓碑预付两式逐项对条款（S、K、N_ℓ 的取法；H 从墓碑树根码 2 头现读；41 先取整再乘；按副本之和记）；删除类发布绕开式子那一判而落点那一判照判，落点那一判要的槽数怎么算、会不会比一次删除发布真要写的多出很多而误拒；标签 4 恒 0 与 I-3.4 的读法。

### K4　回退、删空与读者（L2、L5）
回退挑树的种类表；谱系记录与标签 6 现算；第三个成员的判定只看那四样、不看墓碑树与谱系树；从意图提交起只读挂载、冷走读、模型三处都答「没有文件」；写入口拒 nlink = 0 的每一条路。

### K5　新树与 checker（L7）
根长高与内部条目 120 字节、类型段；只供测试的旋钮会不会漏进落盘；池级 checker 遍历四棵新树用的是不是自己的算法（D13 已定项 5），单元有没有漏算进 I-3.1、I-3.11 那几项统计。

### K6　模型与随机历史（L8）
模型删除那一臂与实现在哪种历史上会分叉；执行器按「有树」挑版本之后，还有没有一种版本被比较链跳过；建 inode 只接在第一个文件删完之后，比重表换表会不会让别的组的旧钉值悄悄失效；Q-T 改钉后的 4 条用例名与断言说的是不是同一件事。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1–K6 | 逐条核代码做的是不是条款说的：每一格从条款原文推到 `crates/` 里的行，再推到钉它的测试与变异行 |
| 云端攻方（Opus） | K1–K6，重点 K2、K3、K6 | 在 `crates/` 的拷贝上造反例：续做与推抬 F 在小盘上留下 checker 不认的盘面、准入误拒或误放、模型与实现分叉的随机历史 |

## 五、跑前写死的判据

- 两腿都没打中 ⇒ 站住（一次抽样，「没打中」照 `.claude/rules/three-way-inference.md` 再抽一次才记）。
- 打中 ⇒ 主 agent 现查坐实，改法进下一批；第二轮攻改法落地与这一轮站住的。
- 腿提的新改法算零轮；L4 那几句用户定案还没写回 kb，算零轮。

交付：腿的报告写 `research/prompts/m3-unlink-code-r1-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-unlink-code-r1-<腿名>-model/`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。
