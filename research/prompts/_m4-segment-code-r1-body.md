# 里程碑四预开发整段：代码轮第一轮正文（2026-10-16）

<!-- doc-lint:not-numbers T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 S1 S2 S3 S4 S5 S6 S7 S8 S9 G1 G2 G3 G4 G5 G6 G7 G8 G9 K-b K-c K-n P1 P2 P3 P4 N2 N3 N5 N6 V-1 -->

## 一、题面

这一轮攻里程碑四预开发整段的合入：写段三波、修复批三簇、写段收尾件，已全部打进工作树 `~/.local/share/singlefs/work/m4-predev`（分支 `m4-predev`，基线 master `c1e9b324`）。第 2 批四份补丁已经过代码轮 `m4-batch2-code-r1`（判决 `research/prompts/m4-batch2-code-r1-main-verification.md`），它们之后又被改过的地方照样在射程里。合入后验证全绿：`✓ 测试全绿（213 个测试二进制，共 2303 条测试）`，fmt、clippy、build 过（`~/.local/share/singlefs/work/merge-and-verify/2026-10-15-2116125/`）；形式化轻型绿（78 verified）。变异轻型红（`~/.local/share/singlefs/work/m4-verify-logs/mutation-quick.log` 的计数行：`计数：抓到 530 条、没红 5 条、无效 0 条、内存撞顶 3 条、超时 4 条`，另有 60 行基线不绿，其中 57 行撞 4G 内存上限），红的那几行在单独重跑，归验证段的修复批，不算这一轮打中。

被判的产品代码（相对 master 的改动全文 `research/prompts/_m4-segment-code-r1-diff.md`）：

| 文件 | 主要来自（补丁目录在 `~/.local/share/singlefs/work/<名>/patch/`，报告是其中的 `report.md`） | 照的判决 |
|---|---|---|
| `crates/singlefs-core/src/transaction.rs`（+2949） | 丙 1 `m4-snapshot-publish-writer`、丙 2 `m4-snapshot-release-writer`、丁 2 `m4-gc-publish-writer`、丁 3 `m4-gc-intent-writer`、调查员 `m4-compaction-remount-investigate` | 快照轮 S1–S4、S8；GC 轮 G3、G4、G5、G8 |
| `crates/singlefs-core/src/mounted_session.rs`（+1057） | 丁 2、丁 3、簇甲 `m4-fix-core-harness-writer` | G4、G5、G8；合取 7 |
| `crates/singlefs-core/src/recovery.rs`（+691） | 第 2 批、丙 1、丙 2、丁 3、簇甲 | S1、S2、S4；G1；I-2.5 次序 |
| `crates/singlefs-core/src/allocator.rs`（+354）、`admission.rs`（+71） | 丁 2、丁 3、写段收尾件 `m4-write-tail-writer` | G4（R 排他、签发 / 停机、K-c）、合取 7、`empty_segments` 记录口径 |
| `crates/singlefs-core/src/records.rs`（+311）、`code_two_tree.rs`（+93）、`unlink.rs`、`write_accounting.rs`、`mount.rs`（+202）、`mounted_read.rs`、`lib.rs` | 丙 1、丙 2、丁 3 | S1、S3、S4、S8；`TransactionUnit::LoggedOperationsRoot` |
| `crates/singlefs-core/src/block_device.rs`（+496） | 第 2 批 discard、丁 1 `m4-gc-fix-writer` | G5 |
| `crates/singlefs-core/src/region_scan.rs`、`compaction_plan.rs`、`logged_operations.rs`（新） | 第 2 批、丁 1、丁 2、写段收尾件、Verus 件 `m4-compaction-verus-writer` | G1、G2、G9 |
| `crates/singlefs-format/src/lib.rs`（+44） | 第 2 批 | G1 |
| `crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs`（新）、`lib.rs` | Verus 件 | G7 V-1、P1–P4 |
| `crates/singlefs-reference-filesystem/src/lib.rs`（+242） | 丙 2、丙 3 `m4-snapshot-model-writer` | S7 |

判决原文：快照轮 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节（S1–S9）与第七节用户定案；GC 轮 `research/prompts/m4-gc-r1-main-verification.md` 第三节（G1–G9）与第七节用户定案；第 2 批代码轮判决第二节的去向。两份判决写回的决策：快照一侧 D5（快照 / 空间记账机制）、D8（核心索引结构）、D16（发布语义）；GC 一侧 D26（后台整理与放置回收）、D19（块指针的结构与宽度预算），discard 在崩溃点重放里的态照 D13（验证路线）。用户在这一段定的读法（调度记录 `records/2026-10-13-里程碑四预开发.md` 第三节原话）：建快照算非空；扫描重建第 4 批各定一条读法；合取 7 现在就接；I-9.34 取乙；三处都认、父链断了报不适用；照字节比、搬根算非空。

## 二、实现今天的样子（主 agent 的观测，2026-10-16）

- 冻结副本 `~/.local/share/singlefs/work/m4-segment-code-r1/repo/`（工作树此刻的 `crates/`、`Cargo.toml`、`Cargo.lock`）。被判文件与 `crates/mutations.tsv` 的 sha256 在 `research/prompts/m4-segment-code-r1-snapshot/crates-sha256.txt`（21 行）。
- 读到的入口（冻结副本行号）：`transaction.rs` 第 7220 行 `publish_create_snapshot`、第 7447 行 `publish_relocation_batch`、第 7494 行 `publish_compaction_step`、第 6855 行 `plan_rewriting_the_paths_to_the_moved_entries`、第 3511 行 `placements_to_release_via_mapping_except_the_units_kept_for_the_snapshots`、第 4762 行 `deadlist_entries_the_destruction_of_a_snapshot_releases`、第 4819 行 `deadlist_changes_of_this_publish`、第 5001 行 `snapshot_lifetime_verdict`、第 4727 行 `next_side_of_the_snapshot`、第 5015 行 `LoggedOperationsRootOrigin`、第 8440 行合取 7 拒成 `PublishError::EmptyClusterSegmentsBelowTheFloor`；`mounted_session.rs` 第 111–117 行 `CreateSnapshot` / `DestroySnapshot`、第 1026 行合取 7 被拒不推抬 F、第 1286 行 `run_compaction`、第 1786 行 discard 调用点；`allocator.rs` 第 821 行 `take_the_segments_to_discard`、第 850 行 `empty_segments`；`admission.rs` 第 856 行合取 7 判定；`mount.rs` 第 7312 行 `mount_writable_with_compaction_switches`、第 7595 行 `RollbackError::SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses`；`recovery.rs` 第 5931 行 `read_the_first_file_of_a_snapshot`、第 5204 行 `deadlist_entries_parsed`；`records.rs` 第 559 行 `previous_snapshot_txg`；`region_scan.rs` 第 204 行 `scan_region`、第 183 行 `TwoRecordsMovableUnderOneMappingKey`；`compaction_plan.rs` 第 88 行 `choose_region`、第 221 行 `plan_batch`、第 332 行 `CompactionSwitches`；`unlink.rs` 第 112 行 `deadlist_entries_per_snapshot_destruction_batch`。
- 实现员取、主 agent 认、被攻过零轮的读法（各份报告「停下交主 agent」一节，判定在调度记录第三节对应的交回行）：
  - 快照：码 14 的 `previous_snapshot_txg` 写前驱链；建快照那次事务号取上一版 + 1；`mount_writable` 不换入口；deadlist 节点空间按普通分配；下一侧是活头时 hi 取这一批自己的 txg；删快照批大小与 unlink 共用开关；C300 写前拒；有树而文件集合为空的一版上删快照放行（丁 2 改）。
  - GC：`segs0` 沿用记录口径；`B_gc` 数整理的每一次发布（签发与删意图都算）；丢意图也发一次发布删掉盘上那条；整理意图、不合格段表、常驻隔离集归位到 `MountedSession::compaction`；装着还没清扫的孤儿头的段不发 discard；同一份扫描里两条能搬的记录映射 key 相同时整次拒（盘上走得到：读盘重建时留存给快照的映射条目）、`plan_batch` 开头断言；`empty_segments` 改记录口径增量维护；合取 7 被拒不推抬 F。丁 3 报告「交主 agent 的读法」一节另有 12 条。
  - 调查员修法：中央映射树规划只按 key 进出定重写哪些节点，整理搬单元 key 不变、位置变，`plan_rewriting_the_paths_to_the_moved_entries` 把装着搬走 key 的叶连同到根整条标重写；多出来的重写准入 `ckpt_cost` 罩不罩得住没量。
- 已知、归验证段修复批或欠账、报出来不算打中的：变异轻型的红行（第一节）；GPU 扫描重建内核没有（C821（GPU 判器没有扫描重建内核））；扫描重建对删除中与覆盖写变短的镜像报不适用（C827（扫描重建对删除中与覆盖写变短的镜像报不适用））；`fallocate` 报不支持那一支没有用例；判器 WGSL 与 GPU 对拍（checker 档，不在射程）。它们的后果超出这里写的范围时算打中。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### T1　建快照那次发布
`transaction.rs` 第 7220 行 `publish_create_snapshot` 与 `SnapshotStepOfThePublish::CreateSnapshot` 的各处分支（第 4679、10819–10842、10960–11048 行一带）、`mounted_session.rs` 的 `CreateSnapshot`：逐样对快照轮 S1、S2 与用户定案「建快照算非空」——独立发布、从水位连发两个树号、码 14 / 15 一对、活头 inode 树条目 `previous_snapshot_txg` 换成 S.txg、第 11 个写前拒；事务号取上一版 + 1（零轮）在回退、根环、准入与 D16（发布语义） 的判定里会不会和别的发布撞号；发布把上一版码 14 / 15 原样续上，覆盖写、unlink、整理一批、抬 F 空发布这几类发布各续没续；S1 配对判定（第 2 批代码轮 T1 打中后补的两条用例与变异）在建快照之后的每一版上还成立不成立。

### T2　释放分流与 deadlist
第 3511 行 `placements_to_release_via_mapping_except_the_units_kept_for_the_snapshots`、第 5001 行 `snapshot_lifetime_verdict`、第 4819 行 `deadlist_changes_of_this_publish`、第 8753–8771 行 deadlist 条目落点与角色、`recovery.rs` 第 5204 行读回：对 S3 原文——切口、按出生 txg 与快照 txg 比（`birth == S.txg` 那一格归哪边）、只有六个用户可见角色问寿命判定（第 4958 行）；deadlist 种类 8、key 43 的字节与 D8（核心索引结构） 的码 2 节点布局；deadlist 节点空间按普通分配（零轮）会不会让 deadlist 自己的节点被下一次释放判成「快照留着」或反过来；两个快照之间出生的单元在中间那个快照删掉时去哪一侧（第 4727 行 `next_side_of_the_snapshot`，下一侧是活头时 hi 取这一批 txg，零轮）。

### T3　删快照意图、批与续做
`mounted_session.rs` 第 117 行 `DestroySnapshot`、第 4762 行 `deadlist_entries_the_destruction_of_a_snapshot_releases`、`unlink.rs` 第 112 行批大小：对 S4 原文——意图写在码 14 条目 flags 位 0、一批放多少条、最后一批去掉码 14 / 15、崩在两批之间的可写挂载续做到空；批大小与 unlink 共用开关（零轮）；有树而文件集合为空的一版上删快照放行（丁 2 改）之后，删快照与 unlink、整理一批同一次挂载里交错时谁先谁后；销毁中的快照在读快照、回退拒、建新快照（上限 10 数不数它）三处各被当成什么。

### T4　回退拒与读快照
`mount.rs` 第 7595、8262、8341 行回退拒、`recovery.rs` 第 5931 行 `read_the_first_file_of_a_snapshot` 与映射条目留存：对 S8 原文「现行版或目标版树表里有码 14 / 15 就拒」——两版各怎么取，销毁中的快照算不算；读快照走的是快照那一版的 inode 树还是留存的映射条目，映射条目被整理搬走（T5）之后读快照读到哪；第 2 批代码轮 T2 打中的五个读树表的读者（`key_width_for_kind`、`user_visible_tree_root_pointers` 等）今天对码 14 / 15、码 8、码 12 各放行还是拒。

### T5　一批搬迁的事务形态（G3）
第 7447 行 `publish_relocation_batch`、第 6855 行 `plan_rewriting_the_paths_to_the_moved_entries`、第 5976–6050 行与第 6794–6800 行映射单元的去向：对 G3 用户定案「映射是唯一权威」——一批里每个 key 的目的落点、映射条目、源落点释放是不是同一次发布、崩在发布前后各读到哪一份；被快照钉住的单元搬走之后快照还读得到（G3 与 S3 的交叉）；调查员修法在搬的单元落在多片叶、同一片叶里还有别的 key 进出、映射树只有一片叶这几种形状下重写的节点集合对不对；多出来的重写有没有被 `ckpt_cost` 准入罩住（没量）——给一个越过准入的取样点或说明为什么越不过。

### T6　整理意图上盘与挂载续做
`TransactionUnit::LoggedOperationsRoot` 的各处（`transaction.rs` 第 2344、2576、2698、3676、5015、9923–9950 行，`recovery.rs` 第 3348、3435 行，`write_accounting.rs` 第 96、305 行，`admission.rs` 第 1490 行）、第 7494 行 `publish_compaction_step`、`mount.rs` 第 7312 行：对 G1 字段表与 G8——签发、每一批、达成、丢意图各写什么、删什么；挂载恢复把盘上那条意图的 R 设成排他并续做；`segs0` 记录口径与字段表原文「记账第 11 项占用口径」不一致（零轮，丁 3 自报）会让哪一个停机或准入判定翻面；`B_gc` 数每一次发布（零轮）；丢意图也发一次发布（零轮）；logged_ops 根恒为零的旧镜像与带意图的新镜像在只读挂载、可写挂载、回退三条路上各被当成什么。

### T7　R 排他、签发与停机、合取 7
`allocator.rs` 的 R 排他与签发 / 停机谓词、第 850 行 `empty_segments`、`admission.rs` 第 856 行、`transaction.rs` 第 8440 行、`mounted_session.rs` 第 1026、1286 行、`compaction_plan.rs` 第 332 行 `CompactionSwitches`：对 G4 用户定案（R 本地谓词、签发前判、无产出就停）与合取 7 的条款——R 排他在分配、抬 F、回退、unlink 释放四条路上各有没有漏口；K-c 丢意图的判定（一个槽既钉住又在隔离集里时）；`empty_segments` 增量维护在分配、释放、整理搬迁、回退、挂载重建五处各加减对不对（与全扫重算比）；合取 7 被拒不推抬 F（零轮）会不会让一块盘永远凑不够 `N₀` 而整理又因为合取 7 起不来；`run_compaction` 一次调用至多签几条、发布后至多一批（G8 甲）。

### T8　discard 调用者（G5）
`mounted_session.rs` 第 1786 行、`allocator.rs` 第 821、1891 行：对 G5「发布返回之后、可再分配谓词为真时整段发」——整段的界、谓词在发 discard 那一刻与下一次分配之间会不会变假；装着还没清扫的孤儿头的段不发（零轮）——「还没清扫」怎么判、清扫之后补不补发；discard 交 Unsupported、交错误、长度 0 三种结局各让调用方做什么；discard 与同一次挂载里的回退、整理意图续做交错时，被 discard 的段有没有可能还被某一版根引用（变异轻型里 `crates/mutations.tsv` 第 4453 行那条超时的改法就在这里，超时不算打中，但攻方可以给一条不超时的反例）。

### T9　区域扫描与计划的修订、Verus 核
`region_scan.rs` 第 204 行起、`compaction_plan.rs` 第 88、221 行与末尾两条对拍单测、`crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs`：第 2 批代码轮 T4、T5 打中之后的改法（判类次序钉住 → 隔离 → 读头 → 类标签 → key 与位置，隔离槽一律 `Quarantined`；`RegionChoice` 三成员、`PlannedBatch` 三成员、扫描拒过的段不再选）有没有把打中的反例关掉、有没有新开口子；`TwoRecordsMovableUnderOneMappingKey` 整次拒在留存给快照的映射条目上会不会让一个含快照的段永远整理不了；P1 只证了五条资格、没证最优，P2 的前提由扫描先拒保证——核与核心层代码之间还差哪一条没对上（对拍单测的输入域多大）。

### T10　恢复读侧
`recovery.rs` 的改动（I-2.5 挪到盖映射位置之前那一处、logged_ops 根读回、码 14 / 15 与 deadlist 树的读者、留存映射条目）：I-2.5 提前之后，合法搬迁镜像（映射条目指着目的落点、源落点还没被覆盖）与合法快照镜像（留存的映射条目）会不会被它拒；映射条目值逆序的拒；读回的 logged_ops 条目与 `logged_operations.rs` 第 317 行起的拒绝（不升序、多于一条整理意图）在恢复里拒的是这一版还是整个挂载。

### T11　RefFS 与模型
`crates/singlefs-reference-filesystem/src/lib.rs` 的快照三操作与允许集合、harness 模型（`crates/singlefs-harness/src/model.rs` 一带，测试代码，只核口径）：对 S7——建快照、删快照、读快照在 RefFS 里的允许集合与核心层的崩溃语义（建快照那次发布崩在中途、删快照崩在两批之间）是否同口径；模型「非空」的判法与核心层（建快照算非空、搬根算非空）是否同口径——变异轻型里 `crates/mutations.tsv` 第 4346 行（「非空」不看快照条目）没红，攻方可以给一条会红的输入。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | T1–T11 | 核「代码做的是不是判决与用户定案说的那件事」：逐格对快照轮 S1–S9、GC 轮 G1–G9 的判决原文、第七节用户定案与冻结副本的行，每条结论写文件:行号；零轮读法逐条判它与判决原文相容不相容 |
| 云端攻方（Opus） | T1–T11 | 在冻结副本的拷贝上造反例：合法镜像被拒或坏镜像被放行、该留给快照的单元被释放或不该留的被留、搬迁后映射与快照读到旧落点、意图续做卡死或重复做、R 排他与合取 7 互锁、discard 打到还被引用的段、扫描拒让段永远整理不了、RefFS 与核心层口径不一；每个打中给可复现的用例或变异（改哪一行、哪条断言红）；零轮读法每条至少攻一次；已知清单里的不算打中，后果超出清单写的范围时算 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；量法拷冻结副本，不在冻结副本里改。拷编译目录当种子之前先 `find crates -type f -exec touch {} +`，让改过的文件都重编。
- 每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 8G 的上限，同一时刻只起一条 cargo；编出来的测试二进制也经它跑；用自己的编译目录。前台命令预计超过 4 分钟的拆成小段。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试，不起 GPU，不碰 `crates/singlefs-checker-tier/`。
