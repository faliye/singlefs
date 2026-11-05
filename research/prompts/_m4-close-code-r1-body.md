# m4-close-code-r1：里程碑四收口这一段代码轮第一轮正文（2026-10-30）

<!-- doc-lint:not-numbers T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 S2 S3 S4 S5 S6 S7 K2 K3 K5 K9 K11 R2 R3 R4 R5 R6 R7 P1 P2 P3 P4 P5 P6 P7 Z1 Z3 Z5 Y5 G7 F2 -->

## 一、题面

这一轮攻 m4-validate-code-r3（判决 `research/prompts/m4-validate-code-r3-main-verification.md`）之后、本会话合进主工作区的产品代码改动。射程是 `crates/singlefs-core/`、`crates/singlefs-format/`、`crates/singlefs-replay-prefix/`、`crates/singlefs-reference-filesystem/` 的 `src/`。来源 14 份补丁，都已打进主工作区、没提交。每份补丁压着的条款、改到的文件与行数、报告自己写的「推的」「没核」「停下交主 agent」，整理在事务员的事实表 `~/.local/share/singlefs/work/m4-close-code-round-inventory/result.md`（822 行，sha256 994d24e0…，按补丁目录一节；条款整句抄自各份交回报告，带报告行号）。腿开工先读那一份。

| 来源（补丁目录，`~/.local/share/singlefs/work/` 下） | 关的 | 落点（产品代码） | 执行代码 |
|---|---|---|---|
| m4-multi-inode-size-writer | 判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S2、S3、S4、S6；`research/prompts/m4-trunc-r1-main-verification.md` 第三节 K2、K3、K5、K11；C145 第三条；D8（核心索引结构） 已定项 16 第 2 条 | core：`transaction.rs`、`truncate.rs`、`recovery.rs`、`extent_tree.rs`（只改注释） | 有 |
| m4-partial-write-writer | 判决 `research/prompts/m4-partial-write-r1-main-verification.md` 第二、三、五节（R2–R7）；D4（校验和位置） 已定项 3、4 | core：`mounted_session.rs`、`transaction.rs`、`truncate.rs`、`write_request_split.rs` | 有 |
| m4-partial-write-model-writer | 同一份判决第二、五节；`research/prompts/m4-partial-write-r1-sonnet-output.md` R7.1 | core：`transaction.rs`；RefFS：`lib.rs` | 有 |
| m4-publish-trigger-writer-2 | 判决 `research/prompts/m4-publish-trigger-r1-main-verification.md` 第二节（P1–P7）；D16（发布语义） 已定项 10、17 | core：`journal.rs`、`mounted_session.rs`、`system_configuration.rs`、`transaction.rs` | 有 |
| m4-exit5-block-device-writer | 出口五块设备信任边界三条（事实表该节） | core：`block_device.rs` | 有 |
| m4-exit5-codec-writer、m4-exit5-codec-writer-2 | 出口五编解码与记录格式一簇 146 个 fn；D13（验证路线） 已定项 14；判决 `research/prompts/m4-exit5-relocation-proof-r3-main-verification.md` Z5 | core：`bytes.rs`、`checksum.rs`、`pointer.rs`、`unit.rs`、`root_record.rs`、`records.rs`、`logged_operations.rs`、`journal.rs` | 有 |
| m4-exit5-container-trees-writer | 出口五容器小树一簇 67 个 fn | core：`container_tree.rs`、`deleted_inodes.rs`、`inode_tree.rs`、`lineage_tree.rs`、`retirement_tree.rs`、`tombstone_tree.rs` | 有 |
| m4-close-journal-preallocation-writer | E197（扫环胶水内存随盘数） 的分配器峰值预测按恰好项数算 | core：`journal.rs` | 有 |
| m4-exit5-codec-tighten-writer | 码 2、码 3 写者的前提收严；必红世界换行 | core：`unit.rs`、`journal.rs` | 没有（只改规格与证明） |
| m4-exit5-protocol-writer、m4-exit5-compaction-floor-writer-2 | 判决 `research/prompts/m4-exit5-relocation-proof-r3-main-verification.md` 第二节 Z1、Z3；D13（验证路线） 已定项 18；用户 2026-10-26 定「整理的发布只带持久 F」 | replay-prefix：`gc_ring_interval_proofs.rs`（新）、`compaction_plan_proofs.rs`、`gc_crash_proofs.rs`、`lib.rs` | 没有 |
| m4-exit5-spec-base-writer、m4-exit5-format-constant-consistency-writer | 同一份判决第三节「层与射程」「抽象状态」；用户 2026-10-26 定「core 直验，连胶水一起证」；D13（验证路线） 已定项 5 | core：`lib.rs`、`address.rs`、`system_configuration.rs`、`verified_model.rs`（新） | 没有 |

改动全文：`research/prompts/_m4-close-code-r1-diff.md`，按补丁的打入次序逐份收这 14 份补丁里落在射程内的 hunk（取自各份 `patch/crates.patch`，不另拼）。同一份文件被几份补丁改过的 9 份（`transaction.rs`、`truncate.rs`、`mounted_session.rs`、`journal.rs`、`unit.rs`、`system_configuration.rs`、`lib.rs`、`verified_model.rs`、`gc_ring_interval_proofs.rs`），后一份补丁的上下文是前一份打完的样子，次序见事实表「同一份产品文件被几份补丁改过」。

## 二、实现今天的样子（主 agent 的观测，2026-10-30）

- 冻结副本：`~/.local/share/singlefs/work/m4-close-code-r1/repo/`，取全部 14 份补丁打完之后的主工作区（`crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default`、`.claude/gate.d/verus-pins.tsv`、`.claude/gate.d/verus-trust-list.tsv`）。主工作区里另有别的会话没提交的改动（池级 checker、harness、checker 档、单元图与 `mount.rs` 这类），冻结副本照原样带着，它们不在这一轮的射程里；腿引到的行落在不是这 14 份补丁改的 hunk 上时，写明「不在这一轮的 diff 里」。被判文件的 sha256 在 `research/prompts/m4-close-code-r1-snapshot/crates-sha256.txt`。
- 这一段的验证情况（主 agent 现跑的，记在 `records/2026-10-13-里程碑四预开发.md`）：
  - 形式化轻型在主工作区：core「成项 618 个…各项 verified 之和 711、0 errors（登记 711）」，replay-prefix 154；
  - 变异轻型：journal 预分配那一趟「抓到 302 条、没红 0 条、无效 0 条」；
  - 合入后验证：前几趟编译、测试、fmt、clippy 的红已逐条归属，判决不重复它们。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### T1　多 inode 改 size：别的 inode 走第一个文件那一套
`transaction.rs` 的 `publish_size_change`、`publish_truncation_step`、`plan_the_files_other_than_the_first_after_this_publish`、`ChangeOfAFileOtherThanTheFirst::SizeChange`、`TruncationBatch` / `TruncationFinalBatch` 带 `inode`；`truncate.rs` 的 `plan_the_lower_segment_after_a_size_change`；`recovery.rs` 单元数核对改按 `file.tail` 与新拒因 `RECORD_WITHOUT_AN_EXTENT_ENTRY_HAS_A_SIZE`。对判决 S2、S3、S4、S6 与截断判决 K2、K3、K5、K11，核两样。一是第一个文件与别的 inode 是不是同一条路径、同一份判法（单元数按号取、标签 0/3/4、粘住的标签 0）。二是实现员照条款推出来、写明「供三方核」的四处写法是不是条款推得出（事实表该节「停下交主 agent」末条，报告 R:244、R:245 与之后两处）：
- unlink 的批截带洞的别的文件时，尾部标签换回 1、2；
- N = S = 0、上段里本来没有它的 inode 只改记录、不写条目；
- 冷恢复对 nlink = 0 的记录同判「有记录、没有条目、size > 0」；
- 第四处见事实表。

C145 第三条：`deadlist_changes_of_this_publish` 删快照放块那一处新加的 `assert!(released_entry.is_some(), …)`，在合法历史里会不会被触发。

### T2　部分写 core 一侧：读盘、逐格读回、失败成员
`transaction.rs`、`mounted_session.rs`、`truncate.rs`、`write_request_split.rs`，对判决 R2–R7 与 D4（校验和位置） 已定项 3、4。逐处核以下几样：
- 读旧净荷改成读盘（`read_back_the_old_payload_of_a_data_unit`，截断尾单元一起改）；
- 两份副本一份读错时的判法；
- 部分写的切分（`write_request_split.rs`）在净荷边界、跨单元、写到尾的取样点上切得对不对；
- 新错误成员在落盘之前交回。

### T3　部分写进模型与 RefFS
`crates/singlefs-reference-filesystem/src/lib.rs` 与 `transaction.rs` 的那一处，对判决第二、五节与正推腿 R7.1。实现员报告写了 F2 的读法：正推 R7.4 的推翻条件是「一份读错仍成功」与「读不回就失败」会不会冲突，这一批照正推读成「两份都读不回才失败」、钉成成功，没改判。格里问两件事：这个读法是不是 D4（校验和位置） 已定项 4 推得出；RefFS 的允许集合里部分写的中间态与实现、理想模型判的是不是同一件事。

### T4　发布触发
`journal.rs`、`mounted_session.rs`、`system_configuration.rs`、`transaction.rs`，对判决 P1–P7、D16（发布语义） 已定项 10、17 与用户定案（触发是时间与脏量；P2 是 journal 字节）。核以下几样：
- 触发判据只在操作期间不变的输入上分支；
- 有只供测试的强制开关；
- 屏障与进门预判的次序；
- 报错之后并进冻结的那一次的处理。

另外，报告写「D16 已定项 10「同一 inode 的写折叠」管不到两个号」（R:66）。格里问：两个号的写攒在一次发布里，判决与已定项推不推得出今天的做法。

### T5　块设备信任边界
`block_device.rs` 38 个 fn 进 `verus!`、特质带 ghost 操作日志、`discard_at` 默认实现证完、D 类 8 个与打洞段照信任清单登记并配差分测试。核两样：
- 执行代码这次改写之后，与改前是不是同一个行为（对齐与越界的判法、零填分块长度、错误种类与次序）；
- 信任清单里登记的那几样，差分测试罩不罩得住它们声称的规格。

`check_aligned_and_in_range`、`length_of`、`zero_fill_chunk_length` 在本机负载 80 时各自超过 600 秒验不出，负载 1 时几十秒内验绿。腿不必复验这一点；要是读出它们的证明对 SMT 的负担有异常，写出来。

### T6　编解码与记录格式的执行代码改写
`bytes.rs`、`checksum.rs`、`pointer.rs`、`unit.rs`、`root_record.rs`、`records.rs`、`logged_operations.rs`、`journal.rs`。为了进 `verus!`，闭包与迭代器写成下标循环，`collect` 写成逐项 push，算术改成带 ensures 的写法。两份报告有「同义改写逐处列表」（codec-writer-2 报告第二节）。核三样：
- 每一处是不是同义：边界、溢出、错误分支、读者拒收的条件与次序一个不少；
- 往返契约（写者写出的读者收下、读回相同）在格式常量的边界上取不取得到；
- 码 2、码 3 写者新收严的前提（`build_index_node`「条目宽不窄于 key 宽」、`build_packed_unit`「记录宽不是 0」）在每一个执行调用点是不是都成立，有没有一条合法写路径会碰到它（codec-tighten 报告 R:6）。

`crc32_castagnoli` 留在未解释函数背后（D13（验证路线） 已定项 14），不在这一格判。

### T7　容器小树的执行代码改写
`container_tree.rs`、`deleted_inodes.rs`、`inode_tree.rs`、`lineage_tree.rs`、`retirement_tree.rs`、`tombstone_tree.rs`，67 个 fn。核两样：
- 改写前后行为同不同：分裂、合并、借位、删空叶、按 key 定位，以及 `from_children` 这类构造在空输入或越界时交什么；
- 照实现员定义第 6 步「走得到的分支在任何落盘之前交回一个类型化错误」：有没有改成 `assert!`、`unreachable!` 或静默截断的地方，在合法镜像或坏盘输入上走得到。

### T8　只动规格与证明的五份：规格说的是不是条款
`gc_ring_interval_proofs.rs`（区间版根环协议、G7 `theorem_compaction_keeps_what_the_user_sees` 连崩溃、`lemma_the_reclaim_threshold_leaves_the_released_run_unreferenced`），以及：
- `compaction_plan_proofs.rs`、`gc_crash_proofs.rs`；
- `verified_model.rs` 与 `address.rs` 的 newtype；
- codec-tighten 用 `hide` 换的两行必红世界。

空真与必红世界由门禁判，这一格不重复。只问 `requires` / `ensures` 与判决 Z1、Z3 和用户定案（整理的发布只带持久 F；core 直验，连胶水一起证）是不是同一件事：
- 有没有一条 ensures 比条款弱、一条 requires 比实现的调用点强，使证明成立而条款没被证；
- spec 里的 int 与 core 的 u64 之间的桥（判决 Y5，报告写「这一批没做」）缺着，会让哪一条结论在实现上不成立。

### T9　journal 读者预分配
`journal.rs` 的 `JournalRecord::parse`：读点名项从 `Vec::new()` 改成 `Vec::with_capacity(named_count)`。核两样：`named_count` 在调用到这一行之前已经被约束（坏盘输入给一个大项数时不会先按它分配一大块）；循环不变式与 `with_capacity` 的 vstd 规格是不是仍然证得出同一份结论。

### T10　测试与变异钉得住这一段的执行代码改动吗
对 T1、T2、T4、T6、T7 里的执行代码改动，在 `crates/mutations.tsv` 里找点名它的变异行。格里问：把改动改回改前、或改成另一种错法（同义改写里漏一个边界、错误分支换序），有没有一行变异会红；红不了的，补哪一个取样点。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | T1–T10 | 核「代码做的是不是判决与条款说的那件事」：逐格对判决原文、用户定案与冻结副本的行，每条结论写文件:行号；T1 那四处「供三方核」的写法逐处判推得出、推不出、条款没说 |
| 云端攻方（Opus） | T1–T10 | 在冻结副本的拷贝上造反例：合法镜像被拒或坏镜像被放行；别的 inode 的改 size 与第一个文件判得不一样；部分写一份读错时被误判；发布触发在中途变了分支；同义改写漏一个边界或换了错误次序；容器树在合法镜像上走到 panic；规格比条款弱。每个打中给可复现的用例或变异（改哪一行、哪条断言红）。执行代码改动多，先攻 T6、T7（行数最多：编解码 +6482 行、容器小树 +5423 行），再攻 T1、T2、T4 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出。量法是拷冻结副本，不在冻结副本里改。几棵内容不同的树共用编译目录之前先把 `crates/` 下的源文件时间碰新，或每棵树各用一个编译目录。起 cargo 之前先 `eval "$(python3 /home/fy5090/code/singlefs/scripts/hosts/rocksdb-prebuilt.py env)"`。
- 每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 8G 的上限，同一时刻只起一条 cargo；编出来的测试二进制也经它跑。线程上限照派发提示，用自己的编译目录，跑完删。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试，不起 GPU，不起 Verus。
