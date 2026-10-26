# m4-validate-code-r1：里程碑四验证段代码轮第一轮正文（2026-10-22）

<!-- doc-lint:not-numbers T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 K1 K2 K3 K4 K5 K6 K7 K8 K9 K10 K11 K12 R1 R2 R3 R4 R5 R6 R7 R8 -->

## 一、题面

这一轮攻里程碑四预开发这一段合进主工作区、还没被任何代码轮快照罩住的产品代码（`crates/singlefs-core/`、`crates/singlefs-format/`、`crates/singlefs-reference-filesystem/` 的 `src/`）。都已在 HEAD 841b3beab 里。来源三处，清单由事务员倒推（`~/.local/share/singlefs/work/m4-code-round-inventory/report.md`，sha256 9c65bb44…，逐 hunk 的来源在它 4.3 节）：

| 来源 | 关的 | 落点（产品代码） |
|---|---|---|
| 截断落地六·2：截甲（core）、截丙（模型与 RefFS），连同修复批四的甲四、乙四 | 判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1–K12 | `crates/singlefs-core/src/truncate.rs`（新）、`extent_tree.rs`、`write_request_split.rs`、`mounted_read.rs`、`recovery.rs`、`mount.rs`、`admission.rs`、`logged_operations.rs`；`crates/singlefs-reference-filesystem/src/lib.rs` |
| 截庚：inode 记录截断字段 | 判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R1–R6 | `records.rs`、`system_configuration.rs`（incompat 位 2）、`make_filesystem.rs`、`inode_tree.rs`、`lib.rs`、`recovery.rs` |
| 10-18 同步 master 与合回时解的冲突 | 无条款：只核解冲突没有丢掉任一侧的行为 | `crates/singlefs-format/src/lib.rs`（删 `NEW_POOL_FILE_CREATION_ACCOUNTING_ROWS`）、`mount.rs`、`recovery.rs` 的几处 |

改动全文：`research/prompts/_m4-validate-code-r1-diff.md`，三份：
- `code-round.diff`：13 份文件、81 个 hunk，能拆清来源的部分。
- `code-round-upper-bound.diff`：`logged_operations.rs`、`mount.rs`、`recovery.rs`、`crates/singlefs-reference-filesystem/src/lib.rs` 四份。这四份在 `m4-segment-code-r3` 那一轮快照时的原样找不回来，这一段的改动拆不出来，给的是上界：里面包含 `m4-segment-code-r1` 到 `r3` 已经攻过的部分。
- `code-round-publish-lookup-covered.diff`：`transaction.rs`、`mounted_session.rs` 两份 19 个 hunk。这两份在 HEAD 上与 `m4-publish-lookup-code-r2` 的开工快照逐字节相同，但那一轮的材料只带了七·6 的 diff，这里的截断那几处没被看过。

`crates/singlefs-core/src/block_device.rs` 只动了注释，不进这一轮。

## 二、实现今天的样子（主 agent 的观测，2026-10-22）

- 冻结副本：`~/.local/share/singlefs/work/m4-validate-code-r1/repo/`。它是 `git archive HEAD` 导出的 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default`。主工作区里别的会话暂存的 9 份产品代码（`allocator.rs`、`transaction.rs` 等）不是这一轮的，所以不取主工作区。被判文件与 `crates/mutations.tsv` 的 sha256 在 `research/prompts/m4-validate-code-r1-snapshot/crates-sha256.txt`（18 行）。
- 读到的事实（行号按冻结副本现查）：
  - `crates/singlefs-format/src/lib.rs` 的 `INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BIT: u8 = 0x04`，文档写「读者收只带位 1 的（0x02）与位 1、位 2 都带的（0x06）两种；不带位 2 的池里截断字段必须全零」。
  - `crates/singlefs-core/src/system_configuration.rs` 的 `InodeRecordTruncationFieldsOfThePool { AllZero, Carried }` 与 `WRITTEN_BY_THIS_WRITER = Carried`。
  - 合回那一处删了 `NEW_POOL_FILE_CREATION_ACCOUNTING_ROWS`。
  - `truncate.rs` 是新文件，695 行。
- 这一段的验证情况：这几份产品代码在 2026-10-21 之前的几趟合入后验证与轻型三条上都跑过，判决不重复它们。验证段里还没打进主工作区的三份补丁（修红两份、`m4-i315-r3` 定案）只改测试代码，不动这一轮的快照。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### T1　截断意图条目与 logged_ops
`logged_operations.rs` 的截断种类（码 2）的 `to_bytes` 与 `parse`、`truncate.rs` 签条目那一处，逐字段对判决 K1：条目宽 44，value 34 里 inode 号、对象出生代、目标 size 三段，补零非零拒收；全池活截断条目至多一条。K11 单一路径与 K12 的两个边界在哪一行实现：N = S 只改记录；⌈N ÷ 净荷⌉ × 净荷溢出 u64 时，在任何写之前拒成类型化错误。

### T2　先落目标、分批与最后一批
`truncate.rs`、`transaction.rs` 的 `publish_size_change` 与 `PublishPlan`、`mounted_session.rs`，对判决 K2：意图那次一次发布写 size = N、签条目、按 K3 重写尾单元；每批只从尾部放 K 个单元，共用 `UnlinkBatchSize`，不碰 inode 记录；最后一批放完剩下的（可以 0 个），同一次发布撤条目。中途崩溃、重挂续做时，批的边界能不能被重算错：多放、少放、放到 N 之前的单元。

### T3　尾单元重写与 extent 计划
`truncate.rs`、`extent_tree.rs`、`write_request_split.rs` 的 `data_unit_count_of_a_sequential_write`，对判决 K3：单元 k 的声明长度不等于 `N − k × 净荷` 才重写，截到 0 留一个声明长度 0 的单元；`roles_replaced_via_mapping` 去掉「含 Data(0) 就当整份」那一判之后，覆盖写与截断各自换下哪几个角色。净荷边界上的 N（恰为整单元、差 1、0）各判什么。

### T4　快照下截断
`transaction.rs` 截断那几批经 `deadlist_changes_of_this_publish` 判生死，对判决 K4 与 D5（快照 / 空间记账机制） 已定项 13：被快照引用的尾部单元进 deadlist，不被引用的放回空闲；重写前的单元 k 在快照下怎么判。

### T5　变长、洞的标签与读
`extent_tree.rs` 的 `ExtentUpperLeafEntry` 新标签值与 `find_upper_leaf_entry`、`lower_children_of_a_file_without_holes`，`mounted_read.rs` 读洞那一路，`recovery.rs` 的 `extents_of_the_first_file` 与 `walk_to_file_in_the_unit_area_starting_at`，对判决 K5 与 D8（核心索引结构） 已定项 14、16：变长不立意图、一次发布做完；末单元补零重写；之后到 N 都是洞，不写单元；洞在盘上声明。读者读到洞交全零；冷读与挂载态读同判；不认得新标签的旧读者怎么办。

### T6　inode 记录截断字段与 incompat 位 2
`records.rs` 的 `InodeRecord`（`to_bytes`、`parse`、`truncation_field_fits_the_pool`）、`system_configuration.rs`、`make_filesystem.rs`、`transaction.rs` 的 `inode_record_of_file_version`，对判决 R1、R2、R5 与 D15（格式冻结政策） 已定项 4：
- 偏移 120 起两段 8 字节（T、N_w），108–119 与 136–139 恒 0。
- 什么时候写：意图那次写；各批不碰；变长与 N = S 照抄；整份写清零；unlink 照抄。
- 回退随记录取 R_old。
- 位 2 由 mkfs 起就置上，读者收 0x02 与 0x06 两种，不带位 2 时截断字段全零。

格里要问的：五处构造 `InodeRecord` 是不是每一处都照 R2；位 2 的登记在 D15（格式冻结政策） 已定项 4 那张表里有没有（`.claude/kb/feature-bits.md` 与门禁 code-source-discipline 的 feature-bits 格判两处一致）。

### T7　恢复与扫描重建在核心层的读法
`recovery.rs` 的 `rebuild_version_in_the_unit_area_starting_at`、`rebuild_version_with_its_snapshot_entries_in_the_unit_area_starting_at`，对判决 K6 与 R3、R4：
- 认领数据单元的三条：对象出生代相同；偏移在 ⌈size ÷ 净荷⌉.max(1) 之内；写序。
- 第四条：T ≠ 0 时，单元号 ≥ ⌈N_w ÷ 净荷⌉.max(1) 且出生 txg < T 的不认领。
- 核心层与池级 checker 的 `rebuild.rs` 是两套实现，判同一件事。核心层这一侧有没有漏条件、条件之间是不是合取。

### T8　回退
`mount.rs` 的 `roll_back_by_a_forward_publish`、`version_to_publish_the_rollback_after`、`is_a_unit_taken_from_the_rollback_target`，对判决 K1 末句「管理员回退时 logged_ops 按种类分来源：截断条目取 R_old 那一版」与 R5「字段随记录取 R_old，不另写；回退不许把现行版本的字段并回 R_old 的记录」：截断进行到一半时回退，回退目标那一版的截断条目与记录字段是否成对。

### T9　准入
`admission.rs` 的 `RingWindowOfTheReading` 改动、`mounted_session.rs` 的 `refusal_is_short_of_space`，对判决 K10 与 D28（挂载期承诺量） 已定项 1：截断的意图、各批、最后一批当删除类，不判式子那一判、照判落点，被落点拒时推抬 F；变长当普通写。意图那次每盘新占槽数是多少，有没有超过 W 里那一项 24。

### T10　RefFS 的截断
`crates/singlefs-reference-filesystem/src/lib.rs` 的 `Operation::Truncate`、`VisibleState`、`OperationNotAccepted`、`CrashNotAccepted`，对判决 K8：缩短与变长的可见内容，崩溃之后允许集合里收哪几种中间态。RefFS 是语义参照，不引实现；它与理想模型、实现三方对拍时，截断那一格三方判的是不是同一件事。

### T11　合回时解的冲突
`crates/singlefs-format/src/lib.rs` 删掉 `NEW_POOL_FILE_CREATION_ACCOUNTING_ROWS`；`mount.rs`、`recovery.rs` 里归到「解冲突」的 hunk（事务员报告 4.3 节逐个标了，`recovery.rs` 有 3 个 hunk 混着别的会话的行）。逐个核：取了哪一侧、另一侧的行为丢没丢、被删的常量还有没有人用（全仓 grep）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | T1–T11 | 核「代码做的是不是判决与条款说的那件事」：逐格对判决原文（K1–K12、R1–R6）与冻结副本的行，每条结论写文件:行号；T11 只核解冲突有没有丢行为 |
| 云端攻方（Opus） | T1–T11 | 在冻结副本的拷贝上造反例：合法镜像被拒或坏镜像被放行；截断批的边界算错；回退把字段或条目配错；洞被读成非零或被扫描重建认领；位 2 的两种池混读；准入放过超额。每个打中给可复现的用例或变异（改哪一行、哪条断言红）。上界那份 diff 里已被 `m4-segment-code-r1` 到 `r3` 攻过的角度不重复：那三轮攻的是聚簇段与整理选段（判决 `research/prompts/m4-segment-code-r3-main-verification.md`） |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出。量法是拷冻结副本，不在冻结副本里改。拷编译目录当种子之前先 `find crates -type f -exec touch {} +`，让改过的文件都重编。起 cargo 之前先 `eval "$(python3 /home/fy5090/code/singlefs/scripts/hosts/rocksdb-prebuilt.py env)"`，让 RocksDB 链本机预编库。
- 每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 8G 的上限，同一时刻只起一条 cargo；编出来的测试二进制也经它跑。线程上限照派发提示，用自己的编译目录。前台命令预计超过 4 分钟的拆成小段。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试，不起 GPU。
