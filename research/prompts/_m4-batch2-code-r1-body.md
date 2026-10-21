# 里程碑四第 2 批：代码轮第一轮正文（2026-10-13）

<!-- doc-lint:not-numbers T1 T2 T3 T4 T5 T6 T7 T8 T9 G1 G2 G3 G5 G9 S1 S4 S9 K-b K-c K-n I-1.13 I-4.4 I-3.1 I-3.11 I-1.11 I-1.12 I-4.2 I-5.1 I-1.9 I-1.8 -->

## 一、题面

这一轮攻里程碑四第 2 批四份补丁，已合进工作树 `~/.local/share/singlefs/work/m4-predev`（分支 `m4-predev`，基线是主仓 `ce1457c3` 加主仓暂存树）。四份补丁各照一份判决改，互不撞文件；合入后验证六个包 2088 条测试全绿、clippy 与 build 过（`~/.local/share/singlefs/work/merge-and-verify/2026-10-13-1125423/`）。

| 补丁 | 关的 | 落点（产品代码） |
|---|---|---|
| 四·2 扫描重建对比（`m4-rebuild-writer` + `m4-rebuild-writer-2`） | `.claude/kb/milestone/04-fourth-txn.md`「四·1 题面」：只读扫描重建三张派生表、与遍历方向逐项比，新不变量 I-1.13；用户 2026-10-13 定 GPU 覆盖表进、内核不置位 | `crates/singlefs-checker/src/rebuild.rs`（新）、`walk.rs`、`image.rs`、`lib.rs`；`crates/singlefs-checker-tier/src/crash_judge_gpu.rs` 覆盖表 |
| 一·2 快照格式层（`m4-snapshot-format-writer`） | 判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1（码 14 / 15 两条新种类条目、一一配对）、第七节岔路 1（上限 10，用户定） | `crates/singlefs-core/src/records.rs`（三个常量）、`recovery.rs`（配对判定 `snapshot_tree_table_entries_paired_one_to_one`、`RebuiltVersionWithItsSnapshotEntries`）、`mounted_read.rs` 第 437 行一处 match |
| 二·2 GC 纯函数（`m4-gc-plan-writer`） | 判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1（logged_ops 定宽条目 10 + 34 = 44、一节点 369）、G9（区域扫描四类）、G2（`choose_region` / `plan_batch` 两层纯函数） | `crates/singlefs-format/src/lib.rs` 四个常量；`crates/singlefs-core/src/logged_operations.rs`、`region_scan.rs`、`compaction_plan.rs`（三个新模块），`lib.rs` 三行 |
| 二·2 块设备 discard 口子（`m4-discard-writer`） | 同一判决第三节 G5（discard 在发布返回后、可再分配谓词为真时整段发；崩溃点重放当「范围写零」枚举两态）；D26（后台整理与放置回收） 未定项 9 | `crates/singlefs-core/src/block_device.rs`（trait `discard_at`、`DiscardOutcome`、文件后端打洞、`O_DIRECT` 后端） |

改动全文：`research/prompts/_m4-batch2-code-r1-diff.md`（四份 `crates.patch` 与 `mutations-append.tsv` 原样拼接）。实现员报告：`~/.local/share/singlefs/work/m4-rebuild-writer-2/report.md`、`m4-snapshot-format-writer/report.md`、`m4-gc-plan-writer/report.md`、`m4-discard-writer/report.md`（各目录下 `patch/report.md` 同一份）。

## 二、实现今天的样子（主 agent 的观测，2026-10-13）

- 冻结副本 `~/.local/share/singlefs/work/m4-batch2-code-r1/repo/`（工作树合入第 2 批之后的 `crates/`、`Cargo.toml`、`Cargo.lock`）。被判文件与 `crates/mutations.tsv` 的 sha256 在 `research/prompts/m4-batch2-code-r1-snapshot/crates-sha256.txt`（16 行）。
- 产品代码改动（`git diff --stat`，不含未跟踪的新文件）：`block_device.rs` +429/−?、`recovery.rs` +199、`format/src/lib.rs` +44、`records.rs` +12、`mounted_read.rs` 6 行、`core/src/lib.rs` 3 行；新文件 `logged_operations.rs`、`region_scan.rs`、`compaction_plan.rs`（核心）、`rebuild.rs`（池级 checker）。
- 读到的事实：`records.rs` 第 438、441、444 行三个常量（码 14、码 15、`SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION = 10`）；`recovery.rs` 第 2327–2342 行四条拒绝文案、第 2411 行 `snapshot_tree_table_entries_paired_one_to_one`；`format/src/lib.rs` 第 190、194、197、201 行四个 logged_ops 常量，各带 `format-const:` 文档；`logged_operations.rs` 第 19 行种类码 1 = 整理、第 56 行 `CompactionRegion`、第 75 行 `of_one_cluster_segment`、第 227 行 `parse`；`region_scan.rs` 第 101 行 `AllocationInRegion`（Movable / Pinned / Quarantined / Dead / Inconsistent）、第 144 行 `RegionScanRefusal`、第 167 行 `scan_region`；`compaction_plan.rs` 第 45 行 `choose_region`、第 133 行 `plan_batch`；`block_device.rs` 第 43 行 `DiscardOutcome`、第 193 行默认 `discard_at`、第 246 行 `DiscardLanding`。
- 实现员自定、主 agent 接受、被攻过零轮的（各份报告「停下交主 agent」一节，判定在 `records/2026-10-13-里程碑四预开发.md`）：区域扫描判类优先级钉住 → 隔离 → 读头 → 类标签 → key 与位置、隔离槽仍有映射条目指着时按普通单元判、跨 R 边界的记录拒而不分类、跨度不是 1 / 2 判不一致、只看 R 那块盘的记录与槽号；greedy 同分取起点槽号最小的段；条目宽 ≠ 44 一律拒、种类码 0 保留；码 3 的 key 从单元头反查时实例代号取写序里那一个；discard 归 `StepKind::ZeroFill`、录制器只记 Applied、长度 0 交 Applied、计数放录制器与两个后端、故障注入按写序号排 discard；扫描重建的常量表 `walk::INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED` 按整份 judgements 查、「映射条目改指一份没分配的副本」镜像的期望添 I-1.13。
- 已知、记作欠账或归第 3 批、报出来不算打中的：选段资格写成「至少一个活槽」而判决 G2 是「R 里至少一个能搬的单元」（派发写错，第 3 批改）；GPU 与 CPU 对拍对 I-1.13「不比」还没接进 `crash_judge_conclusions.rs`（第 3 批）；`TreeTableEntry.previous_snapshot_txg` 字段与四处构造点还没加（第 3 批）；宿主重跑的内存盘交 Applied 而虚机 `O_DIRECT` 交 Unsupported（产品路径发 discard 时定）；用默认实现的包装会把 discard 吞成 Unsupported（同上）；GPU 没有扫描重建内核（C821（GPU 判器没有扫描重建内核））；扫描重建对删除中与覆盖写变短的镜像报不适用（C827（扫描重建对删除中与覆盖写变短的镜像报不适用））；常驻隔离集由谁维护（第 3 批）；G3 落地后合法搬迁镜像上 I-1.13 会红（推的，第 3 批补条款）。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### T1　快照条目的配对判定钉得住没有
`recovery.rs` 第 2411 行起 `snapshot_tree_table_entries_paired_one_to_one` 判四样（条数相等、每条码 15 的头 ID 恰是一条码 14 的树 ID 且那条码 14 只被这一条配上、码 14 至多 10 条、码 14 的头 ID 是活头 inode 树号）。逐样对判决 S1 原文与 D8（核心索引结构） 已定项 9 / D5（快照 / 空间记账机制） 已定项 14：四样是不是 S1 要的全部；两条码 15 指同一条码 14、码 14 比码 15 多一条、第 11 条、头 ID 指到别的头这几种镜像各落在哪一条拒绝文案上；`tree_table_entries_each_kind_at_most_once` 对别的种类照旧判「同一种树两条」时，码 14 / 15 的放行有没有把别的种类也放过去。6 条用例（`tree_table_entries_carry_the_previous_snapshot_txg_and_admit_snapshot_kinds`）覆没覆盖这几种镜像，8 行变异是不是各自只靠一条用例红。

### T2　读侧兜底与还没加的字段
`mounted_read.rs` 第 437 行那处 match 与 `recovery.rs` 里读树表的每一处（`_ =>` 兜底在哪）：码 14 / 15 在挂载读路径上被当成什么——放行、拒、还是当成某棵既有的树读它的根？`RebuiltVersionWithItsSnapshotEntries` 今天有没有消费者；`previous_snapshot_txg` 字段还没加时，码 14 条目那 8 字节在今天的 `TreeTableEntry::parse` 里落在哪个字段、会不会被别的判定读成别的量。

### T3　logged_ops 条目的格式
`logged_operations.rs` 第 195 行 `to_bytes`、第 227 行 `parse`，`format/src/lib.rs` 第 190–201 行四个常量：字节偏移与判决 G1 的字段表逐个对（种类 2 + 签发 txg 8 = key 10；value 34 装 R 与 c0 / segs0 这些的宽度各多少、剩下预留多少）；宽 ≠ 44 一律拒、种类码 0 当保留、`CompactionRegion::of_one_cluster_segment` 拒不对齐，这三条拒绝与 D26（后台整理与放置回收） 已定项 2「零格式位：不新增任何盘上字段」按「树里一条普通 key 的 value」读相容不相容；一节点 369 与 D8（核心索引结构） 码 2 节点内部布局（基础字节数、条目宽 44）算得出来算不出来；round-trip 用例有没有钉字节偏移的绝对值，还是只比 `to_bytes` 再 `parse` 回来相等（`.claude/singlefs-ai-sop/rules/test-discipline.md`「只让多条臂互相比」）。

### T4　区域扫描的四类判定
`region_scan.rs` 第 167–219 行 `scan_region` 与第 101 行 `AllocationInRegion`：判类优先级钉住 → 隔离 → 读头 → 类标签 → key 与位置，每一步对判决 G9 原文；隔离槽仍有映射条目指着时按普通单元往下判（自定）会不会把一个「释放前核出校验和对不上」的单元判成 Movable 搬走；头读不出而没有条目指着判「不一致」不判「已死」，这与 G9「已死」的定义（映射里没有它）冲不冲突；只看 R 那块盘、只比槽号（K-n）在两盘不对称时漏判哪一类；跨 R 边界的记录拒而不分类（C411（恢复不判分配记录的跨度与单元类别） 的域）——拒整个扫描还是拒那一条。7 条用例与变异覆没覆盖优先级的每一次交叉。

### T5　选段与计划
`compaction_plan.rs` 第 45 行 `choose_region`、第 133 行 `plan_batch`：资格「至少一个活槽」（已知与 G2 不同、第 3 批改）在这一版上会不会选出一个活槽全是已死或不一致单元的 R，让 `plan_batch` 交空批而 R 还排他着；排开装着豁免单元的段、开放段（`cluster_segments`）与活意图的 R 三条各在哪一行；greedy「活槽最少者先」的分数怎么算（按记录还是按 `scan_region` 的分类）、同分取起点槽号最小；`plan_batch` 只交（映射 key, 源落点）、目的落点不在这里取（G2 原文）；批大小 `CompactionBatchUnits` 与一批里同一个 key 两份副本怎么排。

### T6　discard 口子的 trait 与两个后端
`block_device.rs` 第 189–204 行默认 `discard_at`、文件后端打洞、`O_DIRECT` 后端：默认实现「核过对齐与越界再交 Unsupported」——越界与没对齐的请求交错误还是交 Unsupported，与写零那一路的判定一致不一致；文件后端 `PunchingUnsupported` 交 Unsupported 不退回写 0（与 `ZeroFillLanding::PunchHoleOrWriteZeroBytesIfUnsupported` 的差别），报别的错照常交回，`EINTR` 原样重打有没有上限；长度 0 交 Applied 并计一次；`DiscardLanding` 的测试开关只供测试、平常路径认不认；`O_DIRECT` 交 Unsupported 并计数（第一版不走 `BLKDISCARD`）。14 条新用例与 19 行变异里有 5 行没证红（harness 2、checker 档 3）：没证的那几行各靠谁红。

### T7　录制、两态与设备日志（harness 与 checker 档，产品代码之外）
`crates/singlefs-harness/src/` 的 `RecordedOperationKind::Discard`、`segments.rs` 归 `StepKind::ZeroFill`、`memory_pool.rs` 整段清零、`crates/singlefs-checker-tier/src/device_log.rs` 期望侧投 Discard：录制器只记 Applied、Unsupported 只计数——「不支持就一个字节没动」这一前提在哪一层保证；discard 与 mkfs 清环在种类串里同名 `zero_fill`，哪一道检查会因此分不出来；持久态读回 0、没持久读回旧字节这两态是不是 G5 原文说的两态，有没有第三态（设备不保证读回 0 时 Applied 之后读回什么）；设备日志判器期望侧投 Discard 之后，宿主重跑（`SparseBlockDevice` 交 Applied）与虚机（`O_DIRECT` 交 Unsupported）在今天的流上为什么都是 0 条、下一批会在哪一行判红。

### T8　扫描重建与 I-1.13 的不适用口径（池级 checker）
`crates/singlefs-checker/src/walk.rs` 的 `judge_scan_rebuild_agrees_with_the_walked_index` 与常量表 `INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED`（I-1.11、I-1.12、I-4.2、I-5.1）、`rebuild.rs` 三张派生表：四·1 题面的三个必红世界在常量表之下还红不红（① 映射条目指错落点、② 单元头 key 清零、③ 逆向）；常量表按整份 judgements 查（别的候选根上判的违例也算）会把哪些合法镜像判成不适用；`ScanRebuildNotDecided` 11 个成员里哪几格真实写路径走得到（C827（扫描重建对删除中与覆盖写变短的镜像报不适用））；「映射条目改指一份没分配的副本」镜像上 I-1.13 的违例是扫描重建真的对不上还是 I-3.1 / I-3.11 的连带；输入禁区（只读 `ImageReader` 的设备、几何、候选槽，实例表行从最新根读）有没有被绕过。

### T9　GPU 覆盖表进、内核不置位
`crates/singlefs-checker-tier/src/crash_judge_gpu.rs` 覆盖表 `POOL_CHECKER_COVERED_INVARIANT_INDEXES` 收下标 92、内核不置位（用户 2026-10-13 定，C821（GPU 判器没有扫描重建内核））：CPU 一侧 `crash_judge_conclusions.rs` 的判到位第 92 位为 1、GPU 恒 0，「不比」没接上之前哪几条对拍用例会红、红在哪一条断言；覆盖表的编译期断言（93 项、第 k 项是 k）在带 `gpu` 特性编时过（主 agent 实测 `cargo check --features gpu` 退 0）——它证明的是什么、证明不了什么；I-4.4（意图只描述目标态） 先例那句「进覆盖表但内核不置位」在今天的判器里是怎么落的，I-1.13 照抄它有没有抄错一格。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | T1–T9 | 核「代码做的是不是判决与题面说的那件事」：逐格对判决原文（S1、G1、G2、G5、G9、四·1 题面）与冻结副本的行，每条结论写文件:行号；T7 只核 G5 的两态与「不支持一字不动」的前提 |
| 云端攻方（Opus） | T1–T9 | 在冻结副本的拷贝上造反例：合法镜像被拒或坏镜像被放行、分类把该搬的判死或把不该搬的判 Movable、空批与排他、默认 discard 把越界吞成 Unsupported、必红世界被常量表吃掉、对拍红在没预期的那一条；每个打中给可复现的用例或变异（改哪一行、哪条断言红）；已知清单里的不算打中，但它们的后果超出清单写的范围时算 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；量法拷冻结副本，不在冻结副本里改。拷编译目录当种子之前先 `find crates -type f -exec touch {} +`，让改过的文件都重编。
- 每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 8G 的上限，同一时刻只起一条 cargo；编出来的测试二进制也经它跑；线程上限 8；用自己的编译目录。前台命令预计超过 4 分钟的拆成小段。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试，不起 GPU，不碰 `crates/singlefs-checker-tier/src/crash_amplification*` 与 WGSL。
