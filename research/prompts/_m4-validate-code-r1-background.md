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

### 小节清单：`.claude/kb/feature-bits.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # feature bit 记账表 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （feature 标题之下、第一个下级标题之前的正文：第 2-24 行） | 抄 | 开篇正文，与顶上标题合成行区间 |

### 小节清单：`.claude/kb/decisions/05-snapshot-space-accounting-mechanism.en.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D5 Snapshot / Space Accounting Mechanism —— Settled | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D5 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### Settled Items | 抄 | 默认全抄（脚本生成） |
| #### Settled Item 1: Which Dimensions Are in Dimension Tuple | 抄 | 随父节带到 |
| #### Settled Item 2: What "Generation" Takes, When Old Generations Can Be Discarded | 抄 | 随父节带到 |
| #### Settled Item 3: Carrier Form of Accounting | 抄 | 随父节带到 |
| #### Settled Item 4: Which Statistics Exist | 抄 | 随父节带到 |
| #### Settled Item 5: Widths of Accounting Key Segments | 抄 | 随父节带到 |
| #### Settled Item 6: Placement and Field Table of Sparse Side Table | 抄 | 随父节带到 |
| #### Settled Item 7: Whether "Allocated Bytes" Includes Fixed Structures Written by mkfs | 抄 | 随父节带到 |
| #### Settled Item 8: Which Rows Written to Accounting Tree on New-Pool File Creation | 抄 | 随父节带到 |
| #### Settled Item 9: Which ID Identifies Writable Head, Who Owns `previous_snapshot_txg` | 抄 | 随父节带到 |
| #### Settled Item 10: Statistics Tag Registry and Two Reserved Values | 抄 | 随父节带到 |
| #### Settled Item 11: Whether to Build Deadlist Carrier Day-1 | 抄 | 随父节带到 |
| #### Settled Item 12: Form of Deadlist Entries | 抄 | 随父节带到 |
| #### Settled Item 13: Which Mechanism to Adopt for Lifecycle Determination | 抄 | 随父节带到 |
| #### Settled Item 14: Necessary and Sufficient Condition of Reference Interval | 抄 | 随父节带到 |
| #### Settled Item 15: Whether Background Relocation Modifies `birth` | 抄 | 随父节带到 |
| #### Settled Item 16: Which Side Deadlist Merges to Upon Snapshot Destruction | 抄 | 随父节带到 |
| #### Settled Item 17: Four Premises; Crossing Any One Invalidates It | 抄 | 随父节带到 |
| #### Settled Item 18: Three Clauses to Add for Clones / Writable Snapshots | 抄 | 随父节带到 |
| #### Settled Item 19: Runtime Form of Liveness Judgment and Cost of Snapshot Destruction | 抄 | 随父节带到 |
| #### Settled Item 20: Whether Pending-Deletion Structures Count as Live or Dead | 抄 | 随父节带到 |
| #### Settled Item 21: Whether Snapshot List Needs an Enumerable Persistent Structure | 抄 | 随父节带到 |
| ## Version History | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/05-スナップショット-空間会計メカニズム.ja.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D5 スナップショット / 空間会計メカニズム —— 確定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D5 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 確定項目 | 抄 | 默认全抄（脚本生成） |
| #### 確定項目 1: 次元タプルにどの次元を含めるか | 抄 | 随父节带到 |
| #### 確定項目 2: 「世代」に何を採用し、旧世代をいつ破棄できるか | 抄 | 随父节带到 |
| #### 確定項目 3: 会計のキャリア形態 | 抄 | 随父节带到 |
| #### 確定項目 4: どの統計量が存在するか | 抄 | 随父节带到 |
| #### 確定項目 5: 会計キー各セグメントの幅 | 抄 | 随父节带到 |
| #### 確定項目 6: スパースサイドテーブルの配置先とフィールド表 | 抄 | 随父节带到 |
| #### 確定項目 7: 「割り当て済みバイト」に mkfs が書き込んだ固定構造を含むか | 抄 | 随父节带到 |
| #### 確定項目 8: 新規プールでのファイル作成時に会計ツリーへ書き込む行 | 抄 | 随父节带到 |
| #### 確定項目 9: 書き込み可能ヘッドをどのIDで識別し、`previous_snapshot_txg` は誰に帰属するか | 抄 | 随父节带到 |
| #### 確定項目 10: 統計量タグ登録表と二つの予約値 | 抄 | 随父节带到 |
| #### 確定項目 11: deadlist のキャリアを day-1 に作成するか | 抄 | 随父节带到 |
| #### 確定項目 12: deadlist エントリの形態 | 抄 | 随父节带到 |
| #### 確定項目 13: ライフサイクル判定にどのメカニズムを採用するか | 抄 | 随父节带到 |
| #### 確定項目 14: 参照区間の必要十分条件 | 抄 | 随父节带到 |
| #### 確定項目 15: バックグラウンド再配置で `birth` を変更するか | 抄 | 随父节带到 |
| #### 確定項目 16: スナップショット破棄時に deadlist をどちら側へマージするか | 抄 | 随父节带到 |
| #### 確定項目 17: 4つの前提、いずれかを超過すると無効化する | 抄 | 随父节带到 |
| #### 確定項目 18: クローン / 書き込み可能スナップショットに追加すべき3条 | 抄 | 随父节带到 |
| #### 確定項目 19: 生死判定の実行時形態とスナップショット破棄のコスト | 抄 | 随父节带到 |
| #### 确定項目 20: 削除待ち構造を生存とみなすか死亡とみなすか | 抄 | 随父节带到 |
| #### 確定項目 21: スナップショット一覧に列挙可能な永続構造が必要か | 抄 | 随父节带到 |
| ## 履歴バージョン | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/05-快照-空间记账机制.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D5 快照 / 空间记账机制 —— 已定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D5 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已定项 | 不抄 | 分项索引表，用 --extra 按行区间取 |
| #### 已定项 1：维度元组里有哪几维 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 2：「代」取什么，旧代什么时候可以丢 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 3：记账的载体形态 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 4：统计量有哪几个 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 5：记账 key 各段的宽度 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 6：稀疏旁表的落点与字段表 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 7：「已分配字节」含不含 mkfs 写下的固定结构 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 8：新池新建文件记账树里写哪几行 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 9：可写头由哪个 ID 标识，`previous_snapshot_txg` 归谁 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 10：统计量标签登记表与两个保留值 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 11：deadlist 的载体 day-1 建不建 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 12：deadlist 条目的形态 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 13：生命周期判定取哪套机制 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 14：引用区间的充要条件 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 15：后台搬迁改不改 `birth` | 抄 | 默认全抄（脚本生成） |
| #### 已定项 16：销毁快照时 deadlist 往哪一侧并 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 17：四条前提，越过任何一条它就失效 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 18：克隆 / 可写快照要加三条 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 19：生死判定的运行时形态与销毁快照的代价 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 20：待删除结构算活的还是死的 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 21：快照列表要不要一个可枚举的持久结构 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |

### 小节清单：`.claude/kb/decisions/08-core-index-structure.en.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D8 Core Index Structure —— Settled | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D8 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### Settled Items | 抄 | 默认全抄（脚本生成） |
| #### Settled Item 1: Accounting Expressed as Idempotent Full Values, Batch Generational Replacement | 抄 | 随父节带到 |
| #### Settled Item 2: Node Size 16 KiB, Pinned as Constant | 抄 | 随父节带到 |
| #### Settled Item 3: Key Layout —— Decoupling Rename from Indexing | 抄 | 随父节带到 |
| #### Settled Item 4: Node Size —— Upper Bound Determined by Recovery Time, Value Set by Experiment | 抄 | 随父节带到 |
| #### Settled Item 5: Whiteout Deletion —— Accepted, but Batch Deletions Must Follow Intent Log | 抄 | 随父节带到 |
| #### Settled Item 6: Inode Tree Leaves Are Code 3 Packed Record Units, Record Fixed Length 140, Published (Inode Number, Object Birth Generation) Not Reused | 抄 | 随父节带到 |
| #### Settled Item 7: Write Buffer Entry `seq` —— 4 Bytes, Residing in Value | 抄 | 随父节带到 |
| #### Settled Item 8: Field Table of Tree Table Entries | 抄 | 随父节带到 |
| #### Settled Item 9: Registry of Tree Kind Codes | 抄 | 随父节带到 |
| #### Settled Item 10: First-Version Accounting Lands Directly on Leaves, `seq` Assignment Rule | 抄 | 随父节带到 |
| #### Settled Item 11: Code 2 Payload Layout and Key Comparison Order | 抄 | 随父节带到 |
| #### Settled Item 12: Coupling Between Write Buffer Front-End and Node Message Buffer —— One Level Permitted Per Tree | 抄 | 随父节带到 |
| #### Settled Item 13: Index Tree Updates Perform State Folding —— Intermediate States of Same Key Excluded from Leaves in Same Publish | 抄 | 随父节带到 |
| #### Settled Item 14: One Btree Implementation + Multiple Independent Keyspaces + Two Front-Ends, Derived Trees May Be Heterogeneous | 抄 | 随父节带到 |
| #### Settled Item 15: Three Hard Requirements for Write Buffer Entries | 抄 | 随父节带到 |
| #### Settled Item 16: First-Version Unlink: Creates deleted_inodes Only, Batched from Tail, Resumes to Empty During Mount | 抄 | 随父节带到 |
| ## Version History | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/08-コアインデックス構造.ja.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D8 コアインデックス構造 —— 確定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D8 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 確定項目 | 抄 | 默认全抄（脚本生成） |
| #### 確定項目 1：会計は冪等完全値を取り、一括世代交代とする | 抄 | 随父节带到 |
| #### 確定項目 2：ノードサイズは 16 KiB とし、定数として固定する | 抄 | 随父节带到 |
| #### 確定項目 3：キー配置 —— リネームとインデックスは分離して処理する | 抄 | 随父节带到 |
| #### 確定項目 4：ノードサイズ —— 上限はリカバリ時間によって定め、実験によって値を決める | 抄 | 随父节带到 |
| #### 確定項目 5：whiteout 削除 —— 許容するが、バッチ削除は必ず intent log を経由する | 抄 | 随父节带到 |
| #### 確定項目 6：inode ツリーのリーフはコード 3 パッキングレコードユニットとし、レコードは固定長 140、パブリッシュ済みの (inode 番号, オブジェクト誕生世代) は再利用しない | 抄 | 随父节带到 |
| #### 確定項目 7：write buffer エントリ付属の `seq` —— 4 バイト、value に格納 | 抄 | 随父节带到 |
| #### 確定項目 8：ツリーテーブルエントリのフィールド表 | 抄 | 随父节带到 |
| #### 確定項目 9：ツリー種別コードの登録表 | 抄 | 随父节带到 |
| #### 確定項目 10：第1版の会計はリーフへ直接書き込み、seq の値生成規則 | 抄 | 随父节带到 |
| #### 確定項目 11：コード 2 のペイロード配置とキー比較順序 | 抄 | 随父节带到 |
| #### 確定項目 12：write buffer フロントエンドとノードメッセージバッファの連携 —— 1 つのツリーには 1 レベルのみ許可 | 抄 | 随父节带到 |
| #### 確定項目 13：インデックスツリー更新で状態縮小を行う —— 同一パブリッシュ内で同一キーの中間状態はリーフへ書き出さない | 抄 | 随父节带到 |
| #### 確定項目 14：1 組の btree 実装 + 複数の独立 keyspace + 2 つのフロントエンド、派生ツリーは異種構造を許容 | 抄 | 随父节带到 |
| #### 確定項目 15：write buffer エントリの3つの厳格要件 | 抄 | 随父节带到 |
| #### 確定項目 16：第1版の unlink：deleted_inodes のみを立て、末尾からバッチ処理し、マウント内で空になるまで継続実行する | 抄 | 随父节带到 |
| ## 履歴バージョン | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/08-核心索引结构.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D8 核心索引结构 —— 已定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D8 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已定项 | 不抄 | 分项索引表，用 --extra 按行区间取 |
| #### 已定项 1：记账取幂等完整值，整批换代 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 2：节点大小 16 KiB，钉成常量 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 3：key 布局 —— 改名与索引分开处理 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 4：节点大小 —— 上限由恢复时间定，靠实验取值 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 5：whiteout 删除 —— 接受，但批量删除必须走 intent log | 抄 | 默认全抄（脚本生成） |
| #### 已定项 6：inode 树的叶是码 3 打包记录单元，记录定长 140，已发布的 (inode 号, 对象出生代) 不复用 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 7：write buffer 条目的 `seq` —— 4 字节，住 value | 抄 | 默认全抄（脚本生成） |
| #### 已定项 8：树表条目的字段表 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 9：树的种类码的登记表 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 10：第一版记账直落叶，`seq` 的取值规则 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 11：码 2 的载荷布局与 key 比较序 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 12：write buffer 前端与节点消息缓冲区的衔接 —— 一棵树只准一级 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 13：索引树更新做状态折叠 —— 同一次发布里同一个 key 的中间状态不落叶 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 14：一套 btree 实现 + 多个独立 keyspace + 两个前端，派生树可以异构 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 15：write buffer 条目的三条硬要求 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 16：第一版 unlink：只立 deleted_inodes、从尾部分批、挂载里续做到空 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |

### 小节清单：`.claude/kb/decisions/15-format-freeze-policy.en.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D15 (Format Freeze Policy) — Settled | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D15 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### Settled Items | 抄 | 默认全抄（脚本生成） |
| #### Settled Item 1: Target of Item 7 is Unsettled Items, Not a Decision List | 抄 | 随父节带到 |
| #### Settled Item 2: Tree Table Made an Open List from Day-1 | 抄 | 随父节带到 |
| #### Settled Item 3: `spec_hash` Invariably Computed from Complete Projection of kb Field Tables, Never Switched to Rust Structs | 抄 | 随父节带到 |
| #### Settled Item 4: Bit Allocation Registry for Feature Bits | 抄 | 随父节带到 |
| #### Settled Item 5: Freezing is a Commit Arbitrated by Gates, Not a Verbal Statement | 抄 | 随父节带到 |
| #### Settled Item 6: Checklist That Must Be Completed Before Freeze | 抄 | 随父节带到 |
| #### Settled Item 7: Freezing Split into Four Layers, Plus Three Independent Freeze Components Outside Layers | 抄 | 随父节带到 |
| #### Settled Item 8: What Freedoms Are Lost After Freezing | 抄 | 随父节带到 |
| #### Settled Item 9: Post-Freeze Change Judgment Table | 抄 | 随父节带到 |
| #### Settled Item 10: Bit Count, Accounting, and Allocation Rights of Feature Bits | 抄 | 随父节带到 |
| #### Settled Item 11: Two Gates That Can Go Red Paired with Freezing | 抄 | 随父节带到 |
| ## Historical Versions | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/15-フォーマット凍結ポリシー.ja.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D15 (フォーマット凍結ポリシー) — 確定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D15 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 確定項目 | 抄 | 默认全抄（脚本生成） |
| #### 確定項目 1：第 7 項の判定対象は未確定項目であり決定リストではない | 抄 | 随父节带到 |
| #### 確定項目 2：ツリーテーブルを day-1 から開放リストとする | 抄 | 随父节带到 |
| #### 確定項目 3：`spec_hash` は kb フィールド表の完全射影から常に算出し、Rust 構造体へ切り替えない | 抄 | 随父节带到 |
| #### 確定項目 4：feature bit のビット割り当て登録表 | 抄 | 随父节带到 |
| #### 確定項目 5：凍結はゲート仲裁を受けるコミットであり言葉の表明ではない | 抄 | 随父节带到 |
| #### 確定項目 6：凍結前に完了必須のチェックリスト | 抄 | 随父节带到 |
| #### 確定項目 7：凍結は 4 層に分割し、4 層外に独立した 3 つの凍結コンポーネントを設ける | 抄 | 随父节带到 |
| #### 確定項目 8：凍結後に失われる自由 | 抄 | 随父节带到 |
| #### 確定項目 9：凍結後の変更判定表 | 抄 | 随父节带到 |
| #### 確定項目 10：feature bit のビット数・会計・割り当て権 | 抄 | 随父节带到 |
| #### 確定項目 11：凍結に伴い整備すべき赤判定検査 | 抄 | 随父节带到 |
| ## 履歴バージョン | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/15-格式冻结政策.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D15 格式冻结政策 —— 已定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D15 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已定项 | 不抄 | 分项索引表，用 --extra 按行区间取 |
| #### 已定项 1：第 7 项的对象是未定分项，不是决策名单 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 2：树表 day-1 做成开放列表 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 3：`spec_hash` 恒对 kb 字段表的完整投影求，不切到 Rust 结构体 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 4：feature bit 的位分配登记表 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 5：冻结是一次受门禁仲裁的提交，不是一句话 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 6：冻结前必须完成的清单 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 7：冻结分四层，四层之外另有三个独立冻结组件 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 8：冻结之后失去哪些自由 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 9：冻结后的改动判定表 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 10：feature bit 的位数、记账与分配权 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 11：冻结要配的两条会红的检查 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |

### 小节清单：`.claude/kb/decisions/28-mount-time-commitments.en.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D28 Mount-Time Commitments —— Settled | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D28 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### Settled Items | 抄 | 默认全抄（脚本生成） |
| #### Settled Item 1: Authoritative form of admission inequality | 抄 | 随父节带到 |
| #### Settled Item 2: In-flight synthesis of admission control | 抄 | 随父节带到 |
| #### Settled Item 3: Space reservation for instance switching | 抄 | 随父节带到 |
| #### Settled Item 4: Checkpoint reserve pool inclusion in admission | 抄 | 随父节带到 |
| ## Historical Versions | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/28-マウント時コミットメント量.ja.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D28 マウント時コミットメント量 —— 確定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D28 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 確定項目 | 抄 | 默认全抄（脚本生成） |
| #### 確定項目 1: 受入不等式の権威形式 | 抄 | 随父节带到 |
| #### 確定項目 2: 受入のインフライト合成 | 抄 | 随父节带到 |
| #### 確定項目 3: インスタンス切り替え時の空間予約 | 抄 | 随父节带到 |
| #### 確定項目 4: checkpoint 保持プールの受入算入 | 抄 | 随父节带到 |
| ## 履歴 | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/28-挂载期承诺量.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D28 挂载期承诺量 —— 已定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D28 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已定项 | 不抄 | 分项索引表，用 --extra 按行区间取 |
| #### 已定项 1：准入不等式的权威形式 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 2：准入的在飞合成 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 3：实例切换的预留 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 4：checkpoint 保留池进准入 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |

**出处 `.claude/kb/feature-bits.md:1-24`（整段抄，未转述）**

```markdown
# feature bit 记账表

**中文** · [English](feature-bits.en.md) · [日本語](feature-bits.ja.md)

三张位图（incompat / compat_ro / compat）各 256 位，住系统配置。每分出去一位就在这里写一行，说清它叫什么、哪个格式版本哪次提交引进来、什么状态、语义是什么。

**位号的唯一登记位不在这个文件里，在 D15（格式冻结政策） 已定项 4 的登记表**（`.claude/kb/decisions/15-格式冻结政策.md`）。「第几位归谁」以那张表为准；两处对不上时改 `.claude/kb/feature-bits.md`，不改那张表。门禁 code-source-discipline 的 feature-bits 格（`.claude/gate.d/code-source-discipline.sh`）逐位比对两处，对不上判红。

分配权是「谁先合并谁先占」，不是审批制（D15（格式冻结政策） 已定项 10）。**位一旦用过不许回收**：回收之后重新分配给新特性，旧镜像那一位会被新代码读成「新特性已启用」，文件系统会真的做错事（例如把不存在的加密特征当存在，去找不存在的密钥）。退役的位状态写「退役」，语义永久锁定为退役前最后一次使用的那一句，不许改写。

<!-- feature-bits:table -->
一行一位，七列照 D15（格式冻结政策） 已定项 10 定下的列序。「语义一句话」要逐字包含 D15（格式冻结政策） 已定项 4 登记表里同一位的「含义」，门禁 code-source-discipline 的 feature-bits 格按这一条比对。「名称」写代码里那个常量名，代码侧没有常量的写 `—`。

| 位号 | 类别 | 名称 | 引入版本 | 引入 commit | 状态 | 语义一句话 |
|---|---|---|---|---|---|---|
| 0 | incompat | INCOMPAT_RETIRED_FIRST_SSD_LINE_WITH_ROLLBACK_WITNESS_BIT | 格式版本 1 | 53c9f38 | 退役 | 第一条纯 SSD 布局线；mkfs 起就置上，不认识这一位的读者挂不上（D12（目标介质） 已定项 1「每套布局一个 incompat 位」） |
| 1 | incompat | INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT | 格式版本 1 | 45d49aaf | 在用 | 第一条纯 SSD 布局线（系统配置带回退下界 F、根记录带卸载记号、没有回退见证与回退行）；mkfs 起就置上，不认识这一位的读者挂不上；位 0 退役之后读者见到位 0 一律拒挂 |

**compat_ro 与 compat 两张位图一位都没分，incompat 分了位 0（退役）与位 1，别的位一位都没分**（出处：D15（格式冻结政策） 已定项 4 登记表）。

**代码侧引用了哪几位**：`crates/singlefs-core/src/system_configuration.rs` 的 `INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT = 0x02`，位 1；同一份文件的 `INCOMPAT_RETIRED_FIRST_SSD_LINE_WITH_ROLLBACK_WITNESS_BIT = 0x01` 是退役的位 0，只在测试与注释里用，留着让「用过的位不回收」写进代码；`SUPPORTED_INCOMPAT_BITS` 由位 1 合成，不是位掩码字面量，门禁 code-source-discipline 的 feature-bits 格解不出位号，把它列进成功那句的「没判位号的」名单。`crates/singlefs-checker/src/lib.rs` 判 incompat 用它自己的具名常量 `INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT = 0x02`（第 219 行）。

**位图的字节序**：位图小端，位 n 住第 n div 8 个字节的第 n mod 8 低位（D22（单元原子性怎么合成） 已定项 13）。

```

**出处 `.claude/kb/decisions/05-snapshot-space-accounting-mechanism.en.md:1-6`（整段抄，未转述）**

```markdown
## D5 Snapshot / Space Accounting Mechanism —— Settled

[中文](05-快照-空间记账机制.md) · **English** · [日本語](05-スナップショット-空間会計メカニズム.ja.md)

D5 (Snapshot / Space Accounting Mechanism) governs two matters: block lifecycle determination (birth generation + deadlist, settled item 13 to 21) and space accounting (statistics list, carrier, key width, rows written out on new-pool file creation, settled item 1 to 11); deadlist entry form in settled item 12. Accounting is incrementally maintained on the commit path, not traversed post-hoc, which is the entire value of D5 (Snapshot / Space Accounting Mechanism). Out of scope: authoritative recording point for the predicate "whether a placement currently belongs to active version" is D3 (Space Allocation) settled item 1, D5 (Snapshot / Space Accounting Mechanism) producing only one half—placements entering deadlist are referenced solely by snapshots and not in live tree; authoritative form of acceptance inequality in D28 (Mount-Time Commitments) settled item 1, D5 (Snapshot / Space Accounting Mechanism) only governing who maintains each term in formula (settled item 4 "Formula item-by-item comparison" table); which mechanism for multi-writable-head in D6 (Snapshot Implementation Model) settled item 1, livelist and its shared tree in D6 (Snapshot Implementation Model) settled item 2 / settled item 3; fields and width of tree table entries in D8 (Core Index Structure) settled item 8; central mapping key in D19 (Block Pointer Structure and Width Budget) settled item 6; `seq` of accounting entries in D8 (Core Index Structure) settled item 7. The O(1) judgment property of D5 (Snapshot / Space Accounting Mechanism) has been invalidated by D6 (Snapshot Implementation Model) adopting multi-writable-head; what restores it is the sparse side table in settled item 6; four premises and invalidation modes in settled item 17.

```

**出处 `.claude/kb/decisions/05-snapshot-space-accounting-mechanism.en.md:7-463`（整段抄，未转述）**

````markdown
### Settled Items

| # | Item | Settlement |
|---|---|---|
| 1 | **Which dimensions are in dimension tuple** | Beyond tree ID only add "device" dimension, key written as `(statistic, tree_id, device, generation)` **Status: Settled.** **Progress: Implemented.** |
| 2 | **What "generation" takes, when old generations can be discarded** | Generation takes checkpoint number, recorded in row key; each root's accounting tree holds only rows of that root's generation, keeping no multiple generations; inode number watermark step row excepted **Status: Settled.** **Progress: Implemented.** |
| 3 | **Carrier form of accounting** | Resides in a btree with independent keyspace, entry is "(statistic, dimension tuple, generation) → full value", goes through write buffer front-end, hangs under parent pointer via copy-on-write **Status: Settled.** **Progress: Implemented.** |
| 4 | **Which statistics exist** | Twelve in active use, numbered up to fourteen (items 8 / 9 withdrawn, numbers not reused); six carrying device dimension, one carrying tree dimension **Status: Settled.** **Progress: Implemented.** |
| 5 | **Widths of accounting key segments** | `(tag 2, tree_id 8, device 4, generation 8)` ⇒ key 22 bytes; value 8 plus seq 4 ⇒ entry 34 bytes **Status: Settled.** **Progress: Implemented.** |
| 6 | **Placement and field table of sparse side table** | Forms independent tree registered day-1; entry = central mapping key 27 + count 4 = 31 bytes wide; maintained at commit, deletion follows intention mechanism **Status: Settled.** **Progress: Implemented.** |
| 7 | **Whether "allocated bytes" includes fixed structures written by mkfs** | Excluded: allocated = sum of placements in allocation record tree, "capacity" in acceptance = unit area size **Status: Settled.** **Progress: Implemented.** |
| 8 | **Which rows written to accounting tree on new-pool file creation** | Statistics with values write one row, those without objects write no rows; new-pool file creation writes 15 rows **Status: Settled.** **Progress: Implemented.** |
| 9 | **Which ID identifies writable head, who owns `previous_snapshot_txg`** | Head ID = tree ID of that head's inode tree; predecessor snapshot number written in that head's inode tree tree table entry and each code 14 snapshot entry, others write 0 and 0 = not applicable **Status: Settled.** **Progress: Implemented.** |
| 10 | **Statistics tag registry and two reserved values** | Tag code = row number in settled item 4 table; device segment without device dimension writes 0xFFFF_FFFF, tree ID segment without tree dimension writes 0 **Status: Settled.** **Progress: Implemented.** |
| 11 | **Whether to build deadlist carrier day-1** | Built: forms independent tree registered into tree table day-1, kind code 8, tree ID 65543, root pointer zero **Status: Settled.** **Progress: Implemented.** |
| 12 | **Form of deadlist entries** | key = head tree ID 8 + epoch 8 + central mapping key 27 = 43 bytes, value empty; entries bucketed by epoch, handover does not move entries **Status: Settled.** **Progress: Unimplemented.** |
| 13 | **Which mechanism to adopt for lifecycle determination** | Takes birth generation + deadlist: compares `previous_snapshot_txg` on deletion, hands over completely on snapshot creation, merges to newer side without full scan on snapshot destruction **Status: Settled.** **Progress: Unimplemented.** |
| 14 | **Necessary and sufficient condition of reference interval** | `birth(b) ≤ S.txg < death(b)`, left-closed right-open; three quantities each defined; default to deadlist when judgment inputs unavailable **Status: Settled.** **Progress: Unimplemented.** |
| 15 | **Whether background relocation modifies `birth`** | Does not modify: relocation is not a publication, `birth` travels with unit as-is **Status: Settled.** **Progress: Unimplemented.** |
| 16 | **Which side deadlist merges to upon snapshot destruction** | Takes Convention B (merges to next newer side); not "snapshot deadlist never rewritten", just exceptions one order of magnitude fewer **Status: Settled.** **Progress: Unimplemented.** |
| 17 | **Four premises; crossing any one invalidates it** | Four premises written strictly item-by-item; introducing second live reference to same block must reopen D5 (Snapshot / Space Accounting Mechanism); triggered once, converged form is D5 (Snapshot / Space Accounting Mechanism) primary + sparse side table auxiliary **Status: Settled.** **Progress: No code object.** |
| 18 | **Three clauses to add for clones / writable snapshots** | Each head holds own predecessor snapshot number and deadlist, initialized from origin; branch point snapshots and heads with live snapshots of their own forbidden from destruction; destroying a head requires livelist **Status: Settled.** **Progress: Unimplemented.** |
| 19 | **Runtime form of liveness judgment and cost of snapshot destruction** | Query side table first, decrement count if count > 1, otherwise `birth > previous_snapshot_txg` releases immediately, remainder enters deadlist; snapshot destruction O(deadlist entry count + bucket count) **Status: Settled.** **Progress: Unimplemented.** |
| 20 | **Whether pending-deletion structures count as live or dead** | Count as live reference: `death` = physical batch checkpoint, judgment naturally occurs at moment of death **Status: Settled.** **Progress: Implemented.** |
| 21 | **Whether snapshot list needs an enumerable persistent structure** | Required: an enumerable persistent structure must exist on disk whose elements are precisely all current live snapshots **Status: Settled.** **Progress: Implemented.** |

#### Settled Item 1: Which Dimensions Are in Dimension Tuple

**Settlement**: Beyond tree ID only add "device" dimension, accounting key written as `(statistic, tree_id, device, generation)`. Whether device dimension is carried is directly given by formula in D28 (Mount-Time Commitments) settled item 1: terms taking each disk's own value carry it, whole-pool commitment terms (recorded as sum of all replicas, deducted amortized to each disk) do not carry it.

**Scope**: Dimension tuple is not fully utilized by every statistic; which terms use which dimension is in settled item 4 list; two reserved values in settled item 10, segment widths in settled item 5. This item does not settle acceptance inequality itself (D28 (Mount-Time Commitments) settled item 1), nor count of statistics (settled item 4).

**Basis**:

- E71 (Accounting Key Entry Count and Width): Entry count computed by per-item dimensioning is one to two orders of magnitude below "dimensions fully used" upper bound; what exhausts budget first is product of `tree_count × device_count`, not any of the four independent variables.
- E97 (Entry Encoding of Accounting and Allocation Records): After each root holds only one generation (criterion 10 fifth rerun, `research/results/e97-entry-encoding-single-generation-2026-10-20.out`, node bytes count leaves only), accounting tree heights for the four key widths (7 / 14 / 19 / 22 bytes) are identical at each of the four grid points registered in E97 (Entry Encoding of Accounting and Allocation Records) (tree count × device count = 64 × 8, 1024 × 64, 8192 × 8, 8192 × 64) with 0 step-row levels (narrow read 1, 2, 2, 2; wide read 2, 3, 3, 3); between grid points this is not guaranteed: among the 524 288 points with tree count 1 to 8192 and device count 1 to 64 under the same geometry, the four widths give different tree heights at 17 440 points for narrow read and 92 663 points for wide read; with today's row set in `crates/` (3 pool-level rows + 6 rows per disk + one step-row level), 7-byte and 22-byte keys give different tree heights on 79 to 141 disks. Under one-generation accounting the whole batch is rewritten, and the cost is the number of leaves written, which follows entry width, not tree height: `1024 × 64` narrow read 22 bytes vs 7 bytes 1.667× (81 920 vs 49 152 bytes), wide read 2.000×; `64 × 8` narrow read is one leaf for all four widths (1.000×). Segment width does have a cost, small in absolute terms under narrow read (largest cell 22 bytes 507 904 bytes, 2.3% of budget).
- User decision, original wording in history.

**Owed**: None.

#### Settled Item 2: What "Generation" Takes, When Old Generations Can Be Discarded

**Settlement**: Generation takes checkpoint number (checkpoint_txg + 1 on each publish, D16 (Publish Semantics) settled item 6, hence generation follows publish rather than 5-second window), recorded in row key. **Each root's accounting tree holds only rows of that root's generation, keeping no multiple generations**: "accounting rolls back with root" is provided by each root copy-on-write branching an accounting tree; rolling back to whichever root reads accounting of that generation. Exception is step rows for inode number watermark statistic: kept continuously with root (see scope clause "step rows for inode number watermark").

**Scope**:

- Generation dimension must still be recorded in row key: accounting entries are idempotent full values (D8 (Core Index Structure) settled item 1), same statistic written repeatedly; scan rebuild relies on it to order new versus old for same statistic under merge rules of settled item 3 / settled item 10. "Accounting rolls back with root" does not rely on it: if accounting had only one in-place overwritten latest value, mounting old root yields new accounting and old tree, instantly turning I-3.1 (allocated statistics match) red; with each root copy-on-write branching an accounting tree, mounting whichever root reads accounting of that generation.
- Two rider clauses: generation segment width **≥ 48 bits** (under 2785 publishes/sec, 32 bits wraps in 17.8 days; width arithmetic of E71 (Accounting Key Entry Count and Width) recalculated accordingly); **cross-publish merging no longer viable** for accounting entries (generation entered key), so each publish rewrites all rows of this generation in full; row count written is actual paid value, not upper bound.
- **Step rows for inode number watermark** (C2aL form of W3): On each publish where inode number watermark changes, accounting leaves a step row (g, H) for this statistic, where g is txg of that publish and H is watermark post-publish; step row COW-persists with root indefinitely; administrator rollback carries from cur (active version prior to rollback) instead of taking from R_old; "each root's accounting tree holds only rows of that root's generation" does not apply to step rows. Inode number watermark at rollback takes only roots not discarded by crash recovery (D23 (Role and Format of Journal) settled item 14 "watermark" slot); rollback does not allow watermark to jump forward, so step rows written only on publishes creating inodes or clones; clone head step rows divided by (statistic, tree), clone creation leaving a row for new head. Step rows cross-checked by equality with cur watermark carried in lineage records (D18 (Information Carried in a Block) settled item 24 clause 5). I-3.13 (accounting row generation segment equals txg of this version) likewise inapplicable to step rows: exception recorded here; invariant list explicitly specifies "step rows of inode number watermark statistic not judged", pool-level checker (`crates/singlefs-checker/src/walk.rs` judging section recognizing step rows via `is_an_inode_number_watermark_step` and skipping) already honors this exception. Cost: 34 bytes per row; units extra written per publish: step rows 0 up to row 462, +2 from row 463, +4 from row 36 074 (calculated by planner model of three-party legs, cross-checked against real planner probes, unmeasured on admission harness).

**Basis**:

- E54 (How Many Generations of Accounting Generations to Keep): "What generation takes" and "how many generations kept" are orthogonal issues—no-discard arm plus incremental discard rule matches keep-last-K-generations cell-for-cell across 60 cells; when not discarding, slope of key count over checkpoint count equals statistic count, whereas keeping K generations slope is 0.
- Main agent inspection on 2026-09-23: No reader requires accounting across multiple generations simultaneously—checker reads accounting rows taking values by (statistic, device), ignoring generation segment (`crates/singlefs-checker/src/walk.rs` accounting tree branch); recovery reads only accounting tree of selected root (`crates/singlefs-core/src/recovery.rs` accounting tree root section); write path writes 15 rows of current generation per publish, never point-deleting (`crates/singlefs-core/src/transaction.rs` `accounting_entry_count(2)`).
- User decision 2026-09-23: Each root carries only its own generation (milestone "overwrite, release, rollback and reuse" closure table row 56), original wording in history.
- User decision, original wording in history.
- Step rows for inode number watermark: W3 verdict `research/prompts/m3-w3-r2-main-verification.md` Section 2 attacker clauses (C2aL strongest form leaves step rows on every publish where watermark changes, granting exception to I-3.13 (accounting row generation segment equals txg of this version) and D5 (Snapshot / Space Accounting Mechanism) settled item 2), `research/prompts/m3-w3-r3-main-verification.md` Section 2 clone head C2aL row (leaves row on clone creation) and Section 3 cost table; verbatim definition of C2aL in second round attacker report `m3-w3-r2-opus-output.md` "**C2aL strongest form** (proposed by me)" row; user decision 2026-10-08 Y2 "C2aL: accounting leaves step rows (recommended)", original wording in history. No experiment: numbers measured on three-party leg models and repository copies, no experiment number established.

**Owed**: None.

#### Settled Item 3: Carrier Form of Accounting

**Settlement**: Accounting resides in a btree with independent keyspace, entry is "(statistic, dimension tuple, generation) → full value".

| # | What Is Settled |
|---|---|
| 1 | Carrier is key in btree, not scalar group in system configuration nor fixed-length area on disk—fixed-length areas freeze "which statistics exist" into format, whereas D12 (Target Media) settled open layout branches per media, each line potentially carrying distinct statistical dimensions; under btree key form **adding new statistical dimensions does not modify format** (bcachefs 2024 shape turning counters into btree keys) |
| 2 | key = (statistic type tag, dimension tuple, generation number); value = full value of that statistic in that generation |
| 3 | Merge rules classified by statistic: non-monotonic statistics "latter wins" (latter determined by seq), monotonic statistics take maximum (items 12, 13, 14 of settled item 4). Taking the maximum is used only to order multiple on-disk values of the same key during scan rebuild (settled item 2 scope: "scan rebuild ordering old and new values of the same statistic by the merge rules of settled item 3 / settled item 10 depends on it"); when the front-end drains to leaves and when a key is read with front-end and leaf coexisting, the maximum is not taken; per D8 (Core Index Structure) settled item 13: apply in arrival order starting from the leaf's current value, and a tombstone is a delete. Point-deleted tombstones do not participate in taking maximum; folding tombstones into max as numerical values causes point-deleted generations to remain indefinitely |
| 4 | Goes through write buffer front-end, entries carrying `seq`—D8 (Core Index Structure) settled write buffer covering three trees "reverse index / LRU / accounting", seq being 3rd hard requirement (sorting discards temporal order, "latter" must be carried by entry itself). First version accounting directly lands on leaves without write buffer, `seq` uniformly 1 (D8 (Core Index Structure) settled item 10), field unchanged |
| 5 | Entire accounting tree copy-on-write hangs under parent pointer (D22 (How Unit Atomicity Is Synthesized) settled item 3) |

**Scope**: Buys "adding statistical dimensions without format changes" and discriminative power of I-3.1 (allocated statistics match), paying in rising key counts (each statistic × each generation occupying an entry), multiplier unmeasured; D8 (Core Index Structure) explicitly states inferring "how much space accounting occupies" is forbidden before measurement. "Tag → merge rule" is on-disk semantics, statistics registry established as independent frozen component (D15 (Format Freeze Policy)); registry itself in settled item 10. If any category of accounting truly cannot achieve idempotence, that category must possess an independent recomputation path sharing no verdict code with runtime.

**Basis**:

- E14 (Discriminative Power of I-3.1: Idempotent Full Value vs Incremental Delta): Under idempotent full value form four categories of semantic bugs caught 3/3, incremental delta form 0/4—full value for value is prerequisite for I-3.1 (allocated statistics match) retaining discriminative power.
- E38 (Accounting Structure COW vs In-Place): In-place form requires mkfs reserving fixed locations for all nodes on disk, COW constantly 0; hardest basis on COW side.
- Conjunction of three prerequisites: D8 (Core Index Structure) settled item 1 settles value semantics, D22 (How Unit Atomicity Is Synthesized) settled item 3 settles landing method, D21 (Boundary Between Authoritative and Derived States) settles requirement of root coverage and participation in format freeze; `.claude/rules/fs-design.md` Slot 1 settles incremental maintenance on transaction commit path. User decision, original wording in history.
- How tombstones fold in row 3 of the settlement table: verdict `research/prompts/m3-close-c693-r2-main-verification.md` section 2 V4 cell (narrowed to "how a front-end tombstone folds against the leaf's current value from the previous publish" and merged into D8 (Core Index Structure) settled item 13, both legs agree) and `research/prompts/m3-close-c693-r3-main-verification.md` section 2 T4 cell (narrowed and merged into D8 (Core Index Structure) settled item 13, upheld); the revision follows the forward leg `research/prompts/m3-close-c693-r2-sonnet-output.md` section "V4.5 clauses and owed checks to change".

**Owed**: C383 (Intermediate state folding of same key in same publish lacks check).

#### Settled Item 4: Which Statistics Exist

**Settlement**: Twelve statistics in active use, numbered up to fourteen (items 8 / 9 withdrawn, numbers not reused), six carrying device dimension (items 1 / 2 / 3 / 5 / 10 / 11), one carrying tree dimension (item 12). Completeness of list is judged relative to acceptance inequality in D28 (Mount-Time Commitments) settled item 1—every term appearing in formula must be a maintained statistic, rejecting any that appears in no verdict; terms unable to do so must explain identity and rationale in "Formula item-by-item comparison" table.

Two additional rules take effect alongside settlement:

- **Item 2 (free bytes) must be maintained independently, never computed on the fly as `capacity − allocated`.** I-5.2 (free space statistics match) verbatim reads "free space statistics == total space − allocated space"; if free space were calculated this way, that invariant becomes a tautology with zero discriminative power; both numbers must follow two non-shared add/subtract paths.
- **Lifecycle determination by design requires no aggregate statistics.** The O(1) property in settled item 19 is built on "liveness determination solely using `ptr.birth` and `head.previous_snapshot_txg`, without querying accounting tree"; adding a statistic to accounting tree for liveness determination destroys that proven property.

**Formula Item-by-Item Comparison** (Authoritative text of formula in D28 (Mount-Time Commitments) settled item 1; this table is sole comparison location between formula and statistics list, gate doc-registries admission-terms cell comparing both sides item-by-item):

<!-- gate:admission-terms -->

| Term in Formula | Who Maintains It |
|---|---|
| 容量 | **Exception**: Device geometry, not accounting statistic—read from device and system configuration, retrieved on each mount, accounting does not maintain it |
| 已分配 | Item 1 (Allocated bytes) |
| 不可回收 | Item 3 (Unreclaimable bytes) |
| 挂载期承诺量 | **Exception**: In-memory only, not persisted, recomputed on each mount (D28 (Mount-Time Commitments) settled item 3), not a published statistic, hence excluded from statistics list; computed per device, discarded on crash (pool-level checkpoint reserve pool listed separately, D28 (Mount-Time Commitments) settled item 4) |
| 待删占用 | Item 4 (Pending deletion but intention incomplete usage) |
| 已承诺预留 | Item 6 (Committed reserve volume (including tombstones)) |
| 被抛弃根独占量 | **Exception**: In-memory derived quantity—value = slots in allocator actively isolated set: slots referenced solely by abandoned roots (slots referenced by abandoned roots − slots referenced by roots in rollback candidate set, lower bound 0; slots referenced by each root read from its allocation record tree without full pool traversal), plus pre-isolated slots during writable mount (D23 (Role and Format of Journal) settled item 14 "pre-isolation during writable mount"), excluding only slots occupied by allocation pointers; computed at each mount and each floor F raise, cleared when abandoned roots overwritten in rotation (D28 (Mount-Time Commitments) settled item 1 ninth term); not a published statistic, excluded from statistics list |
| checkpoint 保留池 | **Exception**: Pool-level member of mount-time commitments, computed on the fly from sum of record tree heights at that moment + accounting nodes (D28 (Mount-Time Commitments) settled item 4), recomputed on each publish, in-memory only, recorded as sum of all replicas and amortized per device, not a published statistic |
| 环窗口 | **Exception**: Backstop for new writes in deletion batches, (root ring slot count 3 × S + 1) × per-batch upper bound W (D28 (Mount-Time Commitments) settled item 1); in-memory only, excluded from accounting rows, recomputed from geometry on each publish, same category as checkpoint reserve pool, not a published statistic |

None of the five exceptions are "noted now to fill later": device capacity, mount-time commitments, abandoned root exclusive volume, checkpoint reserve pool, and ring window do not belong in accounting reconstructed post-crash; building statistics for them equates to persisting quantities recomputed on each mount. Completeness of list is judged solely on terms pointing to "Item N" in table.

**Statistics List**:

| # | Statistic | Who Needs It | Carries Device Dimension | Carries Tree Dimension |
|---|---|---|---|---|
| 1 | Allocated bytes | Allocation, acceptance, I-3.1 (allocated statistics match) | **Carried** | Not carried |
| 2 | Free bytes | Acceptance, checker | **Carried** | Not carried |
| 3 | Unreclaimable bytes | Acceptance, zoned line allocation, user-visible df | **Carried** | Not carried |
| 4 | Pending deletion but intention incomplete usage | Acceptance (I-3.4 (available space deducts pending deletion usage)); constant 0 reserved slot: unreleased units of deleting inodes already in "allocated" (D28 (Mount-Time Commitments) settled item 1) | Not carried (pool-level commitment) | Not carried |
| 5 | Defer queue pending release (by generation) | Defer window, allocation, acceptance | **Carried** | Not carried |
| 6 | Committed reserve volume (including tombstones) | Acceptance (D3 (Space Allocation) settled item 15 + settled item 2); contains only tombstone share per live object, retired record share per inode record, and pool-wide resident margin (tombstone tree 2 + H_tombstone + 1 slots, retired tree 2 + H_retired + 1 slots, H being respective interior levels of active version; formula in D3 (Space Allocation) settled item 2), excluding ring window | Not carried | Not carried |
| 7 | Extension point quota used | Acceptance (D21 (Boundary Between Authoritative and Derived States) settled item 17 clause 4; C21 (extension point pointer escaping accounting)) | Not carried | Not carried |
| 8 | **Withdrawn · This number no longer used** | Formerly settled "per-tree exclusive bytes". Basis for withdrawal: independent censuses by two legs of three-party review across 14 verdict paths + 66 invariants + 129 owed checks / 6 paths yielded **zero hits**, and settled item 4 explicitly commands "liveness judgment does not inspect accounting tree"—not omitted, but blocked; sole consumer "snapshot usage" corresponds to user-visible feature marked **unsettled** in [pitfalls.md](../pitfalls.md). Thus **first version does not provide per-snapshot usage, `df` reporting pool-wide volume only**. Should per-snapshot quota be implemented in future, being a runtime acceptance read quantity, it instantly reverts to `.claude/rules/fs-design.md` Slot 1 "accounting is transaction byproduct", while key encoding is frozen at Layer 3 per D15 (Format Freeze Policy), requiring decision before freeze; no owed checks across repository for this slot | — | — |
| 9 | **Withdrawn · This number no longer used** | Formerly settled "per-tree shared bytes", withdrawn simultaneously with Item 8 on identical grounds | — | — |
| 10 | Fragmentation `F = runs / L` runs: **count of contiguous runs of free slots in unit area** | Compaction trigger verdict (D26 (Background Compaction and Placement Reclamation) settled item 6) | **Carried** (physical metric, one row per device) | Not carried |
| 11 | Completely empty cluster segment count | Compaction trigger verdict + allocation self-sufficiency (same as Item 10) | **Carried** (physical metric—**corollary**, basis in D26 (Background Compaction and Placement Reclamation) settled item 6) | Not carried |
| 12 | Inode number watermark (next available inode number) | Inode number allocation (D8 (Core Index Structure) settled item 6: inode numbers monotonic without reuse, next number must answer immediately when queried) | Not carried | **Carried** (maintained for writable heads only) |
| 13 | Sweep watermark (txg up to which releases confirmed unreadable by full sweep) | Garbage sweep acceptance and tombstone / retired record reclamation gate (C113 (no input for active version judgment of multi-version units during scan rebuild) Settlement P4 / P5) | Not carried (pool-wide quantity: advances only after sweep covers all devices) | Not carried |
| 14 | Generation of most recent root destruction | Lower bound of tombstone reclamation gate (C113 (no input for active version judgment of multi-version units during scan rebuild) Settlement P5: gate = max(death generation, this item)) | Not carried | Not carried |

**Scope**:

- Whether device dimension is carried is directly determined by formula in D28 (Mount-Time Commitments) settled item 1; key dimension tuple is not fully used by every statistic; terms not divided by tree use reserved tree ID 0 as "pool-wide / unassigned" row (settled item 10). Items 10 and 11 are not in that formula; device dimension inclusion is a corollary derived from dimensional arguments in D26 (Background Compaction and Placement Reclamation) settled item 6, requiring verification before modeling.
- Item 12 is maintained only for writable heads; publishes modifying watermark leave a step row, COW-persisting with root indefinitely without being rewritten on every publish (settled item 2 grants exception to rule that each root's accounting tree holds only rows of its own generation rewritten each publish; see settled item 2 scope clause "step rows for inode number watermark"); read-only trees (snapshots) lack this row; carrying tree dimension, tree ID segment remains in key. Reading active value of a statistic = taking row with maximum generation under that dimension tuple; empty signifies never maintained, not 0.
- Items 13 / 14 enter list because "reclamation judgment requires them", both monotonic quantities; sweep watermark advances only when full sweep cycle completes, progress unpersisted mid-cycle, rewritten with each publish only after an advance occurs. First version reclaims no tombstones and performs no sweep; these two items unrecorded in first version (D18 (Information Carried in a Block) settled item 24).
- List is open: single-disk and zoned lines of D12 (Target Media) are not yet opened, and basis of settled item 3 acknowledges verbatim that "each line may possess distinct statistical dimensions", Item 3 being advance extension for zoned; opening lines reopens list.
- Item 12 resides in accounting. Administrator rollback is a forward publish while mounted (D23 (Role and Format of Journal) settled item 14): accounting statistics of that publish computed on the fly by publish path from adjusted allocator rather than loaded from R_old accounting; inode number watermark takes max(active version in-memory, readable from ring and not discarded by crash recovery per instance table) (D23 (Role and Format of Journal) settled item 14 "watermark" slot), without rewinding or re-issuing numbers (`research/prompts/m2-rollback-forward-r3-main-verification.md` Section 2 K1; `research/prompts/m2-rollback-forward-r2-main-verification.md` Section 2 G2). When crash recovery falls back to older root, watermark loads with accounting of selected root, re-issuing numbers issued in abandoned timeline; disposition is narrowing uniqueness: D8 (Core Index Structure) settled item 6 changed to "published (inode number, object birth generation) not reused", watermark remaining here as before (`research/prompts/c143-r3-main-verification.md` Section 4).

**Basis**:

- E71 (Accounting Key Entry Count and Width): After each root holds only one generation (first rerun, `research/results/e71-accounting-keys-single-generation-2026-10-20.out`), the entry count ratio grows with `tree_count × device_count` and is not a constant: across the 15 cells at 30-byte width, wide-read arm vs narrow-read arm is 1.00 to 128.47×, upper-bound arm (dimensions fully used) vs narrow-read arm 1.00 to 283.52×; the `tree_count 64, device_count 8` cell is 14.53× and 28.27×. Cells exceeding 1% of `T_dirty` (21 474 836 bytes): of 45 cells over three widths, upper-bound arm 5 cells (9 under old basis), wide-read arm 3 cells (5 under old basis), narrow-read arm 0 cells; at the `1024 × 64` cell, upper-bound arm does not exceed at 22 and 30 bytes under the new basis (12 976 128, 17 694 720 bytes) and exceeds at 50 bytes (29 491 200), wide-read arm exceeds at none of the three widths under the new basis (5 879 808, 8 017 920, 13 363 200), while under the old basis only 50 bytes exceeded (26 726 400); at the `8192 × 64` cell wide-read arm still exceeds under the new basis at today's 34-byte entry width (the width implemented in `crates/`). `statistic_count × tree_count × device_count` is upper bound of one generation rather than entry count (inode number watermark step rows excepted: persisting with root, entry count grows with publishes modifying watermark, unconstrained by this upper bound).
- E97 (Entry Encoding of Accounting and Allocation Records): One-generation basis (criterion 10 fifth rerun, counted in dirty nodes). The tightest narrow-read cells are `8192 × 8` and `8192 × 64` (tied): 19-byte arm (verdict arm) 458 752 bytes, 2.1% of budget; 22-byte arm (today's key width in `crates/`) 507 904 bytes, 2.3%; the old-basis `8192 × 64` cell, 19-byte arm 11 419 648 bytes, 53%, was narrow read, not wide read. Wide read: `1024 × 64` takes 42% (22-byte arm 46%) and `8192 × 8` takes 47% (22-byte arm 53%), within bound; `8192 × 64` takes 337% (22-byte arm 375%), over bound (all three cells over bound under the old basis), and back-solving that cell yields no feasible entry width. This is one of the cost-side bases for the "which items divided by tree" column: today only item 12 carries the tree dimension, maintained only for writable heads, paying the upper bound of the narrow-read tier; wide read is unaffordable only in the largest cell; the trade-off itself rests on "who needs it" (this item's completeness criterion).
- E93 (Placement and Fragmentation Under Aging): `F = runs / L` incrementally maintainable, incremental values matching full-scan audit point-for-point across all arms × workloads × seeds × sample points, qualifying compaction trigger metric for list inclusion.
- Three-party review: Items 8 / 9 withdrawal verified by two legs independently with zero hits; user decision, original wording in history.
- Item 4 constant 0, Item 6 excluding ring window: `research/prompts/m3-delete-r3-main-verification.md` Section 3 G1-Tag 4 (zero rounds), G1-Tombstone upfront two rows, user decision K-R3-2 "resident deductions (recommended)", original wording in history; Items 13, 14 unrecorded in first version: tombstone verdict `research/prompts/m3-tomb-r1-main-verification.md` Section 3 W3 reclamation row.

**Owed**: C122 (Allocation record entry count measurement basis after eliminating point-deletion); C143 (Inode number watermark rewinding to re-issue numbers after crash recovery falls back to old root); C326 (Whether E93 figures remain quotable after runs measurement basis change).

#### Settled Item 5: Widths of Accounting Key Segments

**Settlement**: `(statistic type tag 2, tree ID 8, device 4, generation 8)`, value = full value 8 bytes, `seq` 4 bytes resides in value (D8 (Core Index Structure) settled item 7) ⇒ key 22 bytes, entry 34 bytes. Generation segment stores full checkpoint number, modulo forbidden.

| Segment | Width | Grade of Basis |
|---|---|---|
| 统计量类型标签 | **2** | **Free choice**: Twelve in active use, numbered to fourteen, 1 byte suffices; 2 chosen for extension margin (settled item 4 declares list open, D12 (Target Media) does not cap media variations); the extra 1 byte drops accounting leaf capacity from 491 to 477 entries (arithmetic on the `crates/` geometry, 2.9% fewer). Whether extra nodes are written depends on whether the accounting tree's total row count (3 + 6 × disk count + step-row count) crosses the threshold of one full leaf: on two disks with 463 to 476 step-row levels, a 21-byte key fits one leaf while a 22-byte key takes two leaves plus a root, writing 2 more nodes per publish |
| 树 ID | **8** | **Free choice**: No existing settled clause bounds upper limit (D6 (Snapshot Implementation Model) settles each writable head has own tree, snapshot count unbounded), using narrow width to fabricate design cap forbidden |
| 设备 | **4** | **Follows D19 (Block Pointer Structure and Width Budget) settled item 4**, same coordinate system |
| 代 | **8** | **Directly given by settled clause**: Rider clause of settled item 2 requires generation segment ≥ 48 bits, and `checkpoint_txg` in D22 (How Unit Atomicity Is Synthesized) settled item 7 is 8 bytes |
| value（完整值） | **8** | Statistics are byte counts; 256 TiB utilizes only 48 bits |

**Scope**: These five segments do not constitute a key capable of determining merge results—accounting entries are idempotent full values, same statistic written multiple times within single checkpoint, `(tag, tree, device, generation)` non-unique within a checkpoint; "latter" in "latter wins" determined by `seq`; with `seq` residing in value, ordering contract of these five segments remains untouched. User decision is "land with this shape first, adjust when writing code".

**Basis**:

- E97 (Entry Encoding of Accounting and Allocation Records): Under the one-generation basis, tree heights for four tiers of keys (7 / 14 / 19 / 22 bytes) are identical at the four registered grid points with 0 step-row levels and not identical everywhere between grid points, and the cost curve is not flat—when the whole batch is rewritten, the number of leaves written follows entry width (leaf fan-out 1087 / 740 / 603 / 542; `1024 × 64` wide read 5 046 272 / 7 405 568 / 9 076 736 / 10 092 544 bytes, narrow read 49 152 / 65 536 / 65 536 / 81 920 bytes); generation segment modulo 2³² wraps in 17 days, modulo 2⁴⁸ in 3204 years, any finite modulo merely postponing broken key ordering until wraparound (modulo 256 yields 255 inverted pairs at generation 257, wrapping in 91 ms), serving as actionable basis for "modulo forbidden".
- E44 (Local Measurement and Cost of Sequence Bit-Width): 48 bits lasts 3202 years, same magnitude as rider clause requiring generation segment ≥ 48 bits.
- E73 (Fanout Cost of Node Key Ranges): Calculated with 22-byte keys, fanout drops at most one step after node carries key ranges, tree height increasing zero levels.
- User decision, original wording in history.

**Owed**: C162 (Recalculating four experiment figures after seq enters value).

#### Settled Item 6: Placement and Field Table of Sparse Side Table

**Settlement**: One sentence per each of four questions—(1) Carrier is an independent keyspace btree, registered into tree table day-1 (new-pool file creation adds one 200-byte tree table entry, root pointer zero); (2) Key takes central mapping key (D19 (Block Pointer Structure and Width Budget) settled item 6: Code 1 is 27 bytes, 25 bytes of Code 2 / Code 3 padded with trailing zeros to 27, D19 (Block Pointer Structure and Width Budget) settled item 10), without version dimension—birth txg in mapping key is already version identity; value is 4-byte reference count, entry width 31 bytes; (3) Incrementally maintained on commit, falling under `.claude/rules/fs-design.md` Slot 1 (liveness determination of cross-head shared blocks must read it); (4) Follows intention mechanism of D8 (Core Index Structure) settled item 5, deletions batched.

**Scope**: Modifies bytes of new-pool file creation—sparse side table entry in tree table (200 bytes), entry itself writing zero rows on new-pool file creation. It answers only four questions: "where it resides, entry shape, who maintains it, whether it uses intention mechanism"; which mechanism cross-head sharing adopts is in D6 (Snapshot Implementation Model) settled item 1, side table repairing premise crossed in settled item 17. Remaining question from D26 (Background Compaction and Placement Reclamation) settled item 5 (what deadlist entries hold) concluded alongside this item: entries recorded by version identity, landing as field table in settled item 12. User decision did not go through three-party review, refutable.

**Basis**:

- E52 (Operating Costs of Two Multi-Writable-Head Mechanisms): Side table entry counts across two cells are 1 000 and 50 000 rows respectively, scale indeed sparse.
- E12 (Batched Sequential Appends vs Non-Batched Random Page RMW): Almost free when fitting in cache; when exceeding cache, approximately 1 device I/O per deletion.
- E146 (Width and Cost of Livelist Entries Identified by Mapping Key): Bill paid for day-1 registration of isomorphic structures (one tree table entry, plus one unit if empty tree requires root node).
- User decision, original wording in history.

**Owed**: C236 (Sparse side table settled as mandatory lacks any placement)—settled half resolved, red-turning check remains owed.

#### Settled Item 7: Whether "Allocated Bytes" Includes Fixed Structures Written by mkfs

**Settlement**: Excluded. "Allocated bytes" = sum of placements in allocation record tree; "capacity" in acceptance inequality = unit area size (system configuration "unit area start slot" to disk end). Fixed structures (system configuration slots, root ring, journal ring) are neither in capacity nor counted as allocated; "total space" of I-5.2 (free space statistics match) likewise.

**Scope**: Settles measurement basis of these two quantities only; does not settle acceptance inequality itself (D28 (Mount-Time Commitments) settled item 1), nor unit area start slot assignment (D3 (Space Allocation) settled item 10).

**Basis**:

- E142 (Dry Run of New-Pool File Creation): Allocated for new-pool file creation is 17 16-KiB slots, fixed structures absent by a single byte, matching `crates/` implementation field-by-field identically.
- User decision and pre-coding general audit, original wording in history.

**Owed**: None.

#### Settled Item 8: Which Rows Written to Accounting Tree on New-Pool File Creation

**Settlement**: Rule is statistics with values write one row, **statistics without objects write no rows** (reading empty signifies never maintained, not equal to 0). New-pool file creation writes **15** rows, keys in ascending order of `(statistic tag 2, tree ID 8, device 4, generation 8)`, tag = row number in settled item 4 table; device segment without device dimension writes 0xFFFF_FFFF, tree ID segment without tree dimension writes 0, generation = checkpoint_txg 3:

| Row | Statistic (Settled Item 4 Row Number) | Key | Value |
|---|---|---|---|
| 1–2 | 已分配字节（1） | (1, 0, 0, 3)、(1, 0, 1, 3) | 278 528 per disk |
| 3–4 | 空闲字节（2） | (2, 0, 0, 3)、(2, 0, 1, 3) | 3 472 605 184 per disk (projection for 4 GiB image) |
| 5–6 | 不可回收字节（3） | (3, 0, 0, 3)、(3, 0, 1, 3) | 0 |
| 7 | 待删除但意图未完成占用（4） | (4, 0, 0xFFFF_FFFF, 3) | 0 |
| 8–9 | defer 队列待释放（5） | (5, 0, 0, 3)、(5, 0, 1, 3) | 16 384 (first file version rewriting tree table replaces mkfs generation 0 tree table unit, D3 (Space Allocation) settled item 7) |
| 10 | 已承诺预留量（6） | (6, 0, 0xFFFF_FFFF, 3) | 196 760 (formula in D3 (Space Allocation) settled item 2, tree table having `deleted_inodes` entry, 2 × ((2 + 0 + 1 + 2 + 0 + 1) × 16384 + 41 × 1 + 35 × 1)) |
| 11–12 | 碎片度 runs（10） | (10, 0, 0, 3)、(10, 0, 1, 3) | 4 per disk |
| 13–14 | 全空聚簇段数（11） | (11, 0, 0, 3)、(11, 0, 1, 3) | 3310 per disk (projection for 4 GiB image) |
| 15 | inode 号水位（12） | (12, 65537, 0xFFFF_FFFF, 3) | 2 |

**Scope**: Modifies bytes of new-pool file creation: yes—accounting tree unit entry count (15), declared length (15 × 34 = 510), and contents of 15 records. Four statistics required by acceptance inequality (unreclaimable, pending deletion, committed reserve, defer pending release) write one row each day-1 to preserve defense line "empty != 0, empty = never maintained": if these four write zero rows day-1, first mount reads empty; with acceptance inequality (D3 (Space Allocation) settled item 4 / D28 (Mount-Time Commitments) settled item 2) needing to read them, judging by "empty = never maintained" refuses writable mount, while judging by "empty reads as 0" dismantles defense; writing one row each is sole approach preserving defense while passing first mount, cost being 6 rows × 34 bytes = 204 bytes. Free bytes and completely empty cluster segment count rows vary with image size, 4 GiB being projection for milestone Step 0. Sweep watermark and generation of most recent root destruction possess no objects at this time, writing zero rows. User decision did not go through three-party review, refutable.

**Basis**:

- E142 (Dry Run of New-Pool File Creation): Keys and values of 15 rows written to bytes field-by-field; 22nd run matches `crates/` region-by-region identically (29 regions `equal=29 unequal=0`, row 10 being 98 386 on both sides); accounting tree unit declared length and entry width on that page.
- User decision and pre-coding general audit, original wording in history.

**Owed**: None.

#### Settled Item 9: Which ID Identifies Writable Head, Who Owns `previous_snapshot_txg`

**Settlement**: Writable head ID = tree ID of that head's inode tree. Accounting tree dimension, livelist shared tree key prefix, and tree table entry `previous_snapshot_txg` ownership all use it. `previous_snapshot_txg` written in two places: in that head's inode tree tree table entry, and in each code 14 snapshot entry (= live head value at capture moment); other entries write 0 and 0 = not applicable; non-zero value written in inode tree entry does not exceed txg of root referencing this tree table (I-9.25 (value of previous_snapshot_txg in tree table entry)); new head without snapshots also writes 0 (snapshot txg ≥ 1, mkfs txg 0 lacks user-visible state, preventing collisions). Non-inode trees locate owning head via "head ID" in tree table entry: tree table entry carves 8 bytes from reserved space to store it (reserved is 76 bytes, D8 (Core Index Structure) settled item 8), value rule being per-head structures write tree ID of head they serve, pool-wide structures write 0—inode tree entries write own tree ID (65537 on new-pool file creation), extent tree entries write head they serve (65537), `deleted_inodes`, tombstone tree, and retired tree write inode tree tree ID (65537), lineage tree writes 0 (pool-wide), logged_ops writes 0 (the compaction intention entry field table is settled, see D26 (Background Compaction and Placement Reclamation) settled item 2; C364 (compaction intent record fields lack field-by-field reconciliation) still owes the field-by-field reconciliation of the field table against the implementation; the field-whitelist judging of I-4.4 (intent only describes target state) is implemented), allocation record tree / accounting tree / livelist shared tree / sparse side table tree / deadlist tree write 0 (unassigned), central mapping tree excluded from tree table.

**Scope**: Settles only "who identifies a head" and "which entry predecessor snapshot number is written to"; does not settle fields and width of tree table entries (D8 (Core Index Structure) settled item 8), nor how `previous_snapshot_txg` propagates in destruction cascade (settled item 13). Read-only snapshot Code 14 entry head ID = captured live head inode tree tree ID; Code 15 entry head ID = paired Code 14 tree ID, predecessor (`previous_snapshot_txg`) written as 0.

**Basis**:

- E142 (Dry Run of New-Pool File Creation): Head ID and `previous_snapshot_txg` for Tree Table Version 1's 11 entries (version prior to adding retired tree) written byte-by-byte, matching `crates/` implementation at the time (24th run on 2026-10-05); with retired tree added implementation reached 12 entries, harness catching up in 25th run (2026-10-13) with 12 entries, watermark 65549, and 29 primary regions identical byte-by-byte.
- User decision and pre-coding general audit, original wording in history.
- Verdict `research/prompts/m3-blanks-r1-main-verification.md` Section 2 head ID row: old definition sentence contradicted its own five examples writing 0 and failed to judge new trees, revised to per-head structures writing their head and pool-wide structures writing 0.
- User decision 2026-10-01: Lineage tree head ID writes 0, pool-wide (`m3-blanks-r1-forks.md` fork sheet "lineage tree head ID" row, original wording in history).
- Verdict `research/prompts/m4-snapshot-r1-main-verification.md` Section 3 S1 (Code 14 entry head ID = live head inode tree tree ID, `previous_snapshot_txg` = live head value at capture moment; Code 15 entry head ID = paired Code 14 tree ID, predecessor 0) and S5 (predecessor chain of I-3.7 (live snapshots enumerable) requires Code 14 carrying predecessor).

**Owed**: None.

#### Settled Item 10: Statistics Tag Registry and Two Reserved Values

**Settlement**: Statistics tag registry is table in settled item 4—code = row number (1 to 14; items 8 / 9 retired without reuse; 0 reserved as invalid); merge rules per row follow settled item 3 (monotonic quantities take maximum: items 12 / 13 / 14; remainder latter wins); device and tree dimensions follow table. For statistics without device dimension, device segment of key takes reserved value 0xFFFF_FFFF; tree ID segment without tree dimension takes 0.

**Scope**: Registry governs "tag code → which statistic, how merged, which dimensions used"; does not govern what older reader does upon encountering unrecognized tag—that slot is compat tier issue, unhandled by any row in D15 (Format Freeze Policy) verdict table.

**Basis**:

- E142 (Dry Run of New-Pool File Creation): Two reserved values in 15 row keys of new-pool file creation (device segment 0xFFFF_FFFF, tree ID segment 0) written byte-by-byte, matching `crates/` implementation identically.
- User decision and pre-coding general audit, original wording in history.

**Owed**: C71 (Unrecognized statistics tag silently ignored); C117 (Device segment of statistics without device dimension lacks assigned value)—value settled, red-turning check remains owed.

#### Settled Item 11: Whether to Build Deadlist Carrier Day-1

**Settlement**: Built. Deadlist forms an independent keyspace tree, **registered into tree table day-1**—tree kind code **8** (registry in D8 (Core Index Structure) settled item 9), tree ID **65543**, root pointer zero (no root yet), birth txg 3, head ID 0 (unassigned). Tree Table Version 1 together with five zero-root trees of D18 (Information Carried in a Block) settled item 24 totals **12 entries**, tree ID watermark post-publish **65549**. New-pool file creation writes zero deadlist entries (no releases).

**Scope**: Modifies bytes of new-pool file creation—tree table Version 1 adds deadlist entry (together with five zero-root trees of D18 (Information Carried in a Block) settled item 24 totals 12 entries, declared length 12 × 200 = 2400; was 11 entries and 2200 before retired tree added), tree ID watermark post-publish 65549 (was 65548 before retired tree added; root record and journal new-root segment one copy each). Settles only whether and when carrier is built; entry shape is in settled item 12. User decision did not go through three-party review, refutable.

**Basis**:

- E146 (Width and Cost of Livelist Entries Identified by Mapping Key): Livelist shared tree is isomorphic precedent; costs of day-1 registration versus lazy creation documented there; lazy creation introduces extra path "build tree upon first snapshot creation", which must create tree, write tree table entry, and elevate tree ID watermark in single publish.
- E142 (Dry Run of New-Pool File Creation): Tree Table Version 1's 11 entries (including four zero-root trees such as `deleted_inodes`) and tree ID watermark 65548 written field-by-field, matching `crates/` implementation at the time (24th run on 2026-10-05); subsequently added retired tree, implementation reaching 12 entries and watermark 65549; harness caught up in 25th run (2026-10-13), 12 entries, watermark 65549, 29 primary regions identical byte-by-byte.
- User decision, original wording in history.

**Owed**: None.

#### Settled Item 12: Form of Deadlist Entries

**Settlement**: Adopts "C-Interval (with upper bound)".

| Item | What Is Settled |
|---|---|
| 条目 | key = head tree ID 8 + epoch 8 + central mapping key 27 = 43 bytes, value empty (key is fact). Identity is central mapping key; epoch = head's `previous_snapshot_txg` at moment block killed; bucketing is this segment, no separate bucket ID field |
| 哪一份 | Snapshot X portion = entries with epoch in [prev(X).txg, X.txg); head portion = [prev(head).txg, ∞). Snapshot creation advances prev(head) only, leaving existing entries unmoved—settled item 13 "hands over completely to new snapshot" refers to this, moving no entries |
| 销毁 S | Writes intention (head, lo = S.txg, hi, P = prev(S).txg): hi takes snapshot txg when next side is snapshot, or open checkpoint number writing intention when next side is head (taking maximum published number makes interval empty, never releasing entries); prev(S) passed to next side at that moment within transaction writing intention, forbidden from splitting across subsequent publishes; subsequently releases entries in [lo, hi) with birth > P in batches without modifying keys of any entries; each action verified for entry presence before applying (D8 (Core Index Structure) settled item 5 hard requirement clause 4); cascades require no ordering, head destruction intention awaiting completion of earlier cascades on same head before touching entries |
| 下一侧怎么找 | Tree table entries lack links between preceding and succeeding snapshots; at moment intention is written, retrieved from derived "head → live snapshot chain" built by scanning tree table at mount: zero bytes, losing it merely induces slowdown (D21 (Boundary Between Authoritative and Derived States) derived state criterion) |
| 按头 / 按快照删一段 | Prefix (head) / (head, epoch in [prev(X).txg, X.txg)) segment |
| 树表字节 | 0; reserved 76 bytes retained for other subscribers |
| 改新池新建文件的字节 | No—new-pool file creation deletes nothing, writing zero entries |

**Scope**: Destroyed snapshots cease pinning blocks from moment intention is written, serving as exception to settled item 20 "pending deletion structures count as live references"; from which moment destroyed writable heads cease counting and when branch point gate `children` decrements lack clauses. Entry shape settled, red-turning check remains unbuilt.

First version destruction intention written on destroyed snapshot's own Code 14 tree table entry: occupies flags bit 0 only (destroying), writing zero bytes to reserved space; lo = entry birth txg, P = entry predecessor, hi retrieved fresh from derived chain per batch (next Code 14 birth txg, or publish txg writing intention when next side is head), storing no resumption point (freed entries represent progress); unhoused in logged_ops. Destruction serialized: single invocation = drain first + intention + all batches, completing before taking other changes (user decision). Snapshot deletion follows unlink process with equal admission, granting no exception to "drain first" (D8 (Core Index Structure) settled item 16 clause 6) (user decision). Next side retrieved from derived chain ordered by (birth txg, tree ID) from tree table.

**Basis**:

- Three rounds of three-party verdicts: `research/prompts/d5-item12-r1-main-verification.md`, `d5-item12-r2-main-verification.md`, `d5-item12-r3-main-verification.md` Section 4 each. C (point-read approach) eliminated across two rounds, A eliminated at U2; C-Interval second round weakest interpretation (intention interval [S.txg, ∞)) over-released in 6 steps, violating D8 (Core Index Structure) settled item 5 "intentions declare target state, never delta", tightened to bounded interval; third round over-released in 7 steps under "plan batch by start-of-batch state without re-verification upon application", violating idempotency of same requirement, tightened to verification upon application. B and D under-released under arbitrary ordering in second round, failing all five aspects in third round once ordered, paying in cascade key rewrites; across five destruction geometries C-Interval had 0 rows, B D(S), D K (`out-geometry.jsonl`).
- E94 (Touch Surface of Moving Shared Placements): Recording by placement forces rewriting deadlist row-by-row on every relocation; recording by version identity maintains single root for central mapping, deadlist rewrites constantly zero.
- Verdict `research/prompts/m4-snapshot-r1-main-verification.md` Section 3 S4 (intention carrier takes Code 14 entry itself, occupying flags bit 0 only, leaving reserved unwritten; lo and P on entry, hi taken fresh per batch from derived chain, no resumption point stored, unhoused in logged_ops) and Section 7 user decision (Fork 2 destruction serialized, Fork 4 snapshot deletion follows unlink process), original wording in history.
- User decision, original wording in history.

**Owed**: C145 (Deadlist lacks on-disk placement)—entry shape settled, checker half remains owed; C336 (Cannot find snapshot next side at moment intention is written).

#### Settled Item 13: Which Mechanism to Adopt for Lifecycle Determination

**Settlement**: Adopts birth generation + deadlist model (ZFS birth txg + deadlist).

- Each block carries **birth generation** (birth txg).
- On deletion, block generation is compared against **live head record `previous_snapshot_txg`** (preceding snapshot generation): `birth > previous_snapshot_txg` **releases immediately**, otherwise **appended to live head's deadlist**.
- On snapshot creation, **live head deadlist is completely handed over to new snapshot**, live head replacing with an empty one (settled item 12: handover moves no entries—entries bucketed by epoch, snapshot creation advancing prev(head) only).
- On snapshot destruction, its deadlist is **merged to next newer side**, filtering entries releasable by `birth > prev(S).txg`, **without full disk scan** (which side merged to in settled item 16).
- **`prev` must propagate**: destroyed side's `prev(S).txg` is inherited by receiving side, passed to next side at that moment within transaction writing intention, forbidden from splitting across subsequent publishes (splitting caused all three candidates to over-release in 5 steps). Without propagation, subsequent generations filter against original `prev`, entries with small `birth` never satisfying `birth > prev`, producing unbounded leaks.

**Scope**: Reference criterion and definition of three quantities in settled item 14, runtime form and cost measurement basis in settled item 19, applicable premises in settled item 17. Interlocking with D9 (Encryption): birth txg **cannot** serve as encryption nonce—blocks in same transaction share it, causing nonce reuse. D5 (Snapshot / Space Accounting Mechanism) itself left only crash replay unrun during live measurement: deadlist append and cascade merge are **non-idempotent** (multiset addition, appending same entry twice double-frees on destruction), double-freeing pointing toward data loss; proven "structural zero in data loss direction" covers only steady-state `death` rule evaluations; two crash-point replay streams write zero deadlist entries, leaving this cell at zero coverage, forbidding claiming risk paths of D5 (Snapshot / Space Accounting Mechanism) verified until run (E13 (Four Risk Paths of D5) Path 3).

**Basis**:

- E12 (Batched Sequential Appends vs Non-Batched Random Page RMW): Across 200,000 deletions, device I/O on birth generation side constantly 2 across three cache tiers; global reference count jumps to ~1 per deletion when counter table exceeds cache, transition being steep; block-layer independent verification (in-VM `/sys/block/vda/stat`) ratio across nine cells uniformly 1.00; removing `O_DIRECT` zeroes block-layer readings while program counters hold, confirming discriminative power of verification path.
- E94 (Touch Surface of Moving Shared Placements): Without `prev` propagation entries pinned permanently, mutation M5b verified red at model layer.
- External implementations (clues, unverified in this project): btrfs qgroup traverses extent refcount + reverse index, paying +76% commit time, +1347% transaction wait, −25% write throughput (measurement basis is "qgroup traversing reverse index", not "pure refcount"); bcachefs accounting maintained by triggers on key commit, `check_allocations` rebuilding via full run traversal, validating "incremental maintenance + traversal audit" pattern rather than this mechanism (bcachefs lacks deadlist), and its traversal audit shares same add/subtract code with runtime, failing to catch semantic errors ([prior-art.md](../prior-art.md) 7.11); copying that shape forbidden in this project.

**Owed**: C195 (Prev propagation during destruction cascade only documented in model revision notes).

#### Settled Item 14: Necessary and Sufficient Condition of Reference Interval

**Settlement**: **Block b referenced by snapshot S ⟺ `birth(b) ≤ S.txg < death(b)`**, left-closed right-open.

| Quantity | Definition | What It Is **Not** |
|---|---|---|
| `birth(b)` | Checkpoint number where block was **published** | Not open txg when write request issued—number re-issued post-crash |
| `death(b)` | Checkpoint number where last live reference unlinked **and unlinking published**; ∞ if still in live tree | Not moment `rm` returns (D8 (Core Index Structure) follows batched intentions), nor moment space becomes reallocatable (D16 (Publish Semantics) settled item 15) |
| `S.txg` | **Last published txg captured by S**; when snapshot creation is independent publish (publish without data mutations), S.txg is `checkpoint_txg` of that publish, same value written to live head inode tree entry `previous_snapshot_txg` and snapshot entry birth txg | Not "txg where creation operation resided" |

Three boundaries and consequences of misconfiguration:

| Boundary | Verdict | Consequence of Misconfiguration |
|---|---|---|
| `birth == S.txg` | S **references** it | Writing `birth < S.txg` judges "newer than snapshot", releasing immediately, causing **data loss** |
| `death == S.txg` | S **does not reference** | Writing `S.txg ≤ death` hangs on snapshot not needing it, causing **space leak + misattribution** |
| `birth == death` | Unreferenced by any snapshot, releasable immediately | Accounting correct, but **physical space forbidden from reuse within this checkpoint** (D16 (Publish Semantics) settled item 15) |

**Conservative default discipline: When any input to judgment is unavailable or uncertain, default must be "enter deadlist".**

**Scope**: Condition equals reachability only under total order of txg (unidirectional linear history); handling beyond premises in settled item 17; under this `death` definition, false deaths under branching are provably zero—truth being true indicates block alive on that root's chain, failing "deleted across all branches", `death = ∞`, interval judgment necessarily true. Rationale for conservative default is asymmetry of failure directions: leaks detectable by I-3.6 (deadlist tight bound) (**not** I-3.1 (allocated statistics match)—[invariants/05-空间记账.md](../../../crates/singlefs-checker/invariants/05-空间记账.md) explicitly notes "misattribution invisible to it... total ledger remains balanced"), reclaimable, damage bounded; loss irreversible, degrading to "checksum mismatch, cause unknown" once space reused, losing even attribution. Second definition for `S.txg` is mechanistic: `birth` resolution reaches checkpoint only; taking "txg of creation operation" makes condition flip between `birth ≤ S.txg` and `birth < S.txg` depending on whether snapshot point sits at checkpoint start or end, definition requiring stability against implementation shifts.

**Basis**:

- E27 (Three Modelable Risk Paths of D5): Consequences of misconfiguring three boundaries indeed asymmetric, not three independent typos—writing right boundary `S.txg ≤ death` hits both `death == S.txg` and `birth == death`, single typo inducing two leaks; failure directions opposite (tight left boundary loses data in 1 place, closed right boundary leaks in 2 places, zero cross-over).
- E18 (Whether D5 Scraps Under Multiple Forks / Multi-Writable-Heads): Under this `death` definition, linear, single fork, and 8 writable heads across three tiers all yielded 0 false deaths.
- Verdict `research/prompts/m4-snapshot-r1-main-verification.md` Section 3 S2: Snapshot creation adopts independent publish, S.txg taking `checkpoint_txg` of that publish; reading S.txg and prev(head) from same source yielded 0 violations on attacker model.

**Owed**: None.

#### Settled Item 15: Whether Background Relocation Modifies `birth`

**Settlement**: Does not modify. Background compaction moving unit from one placement to another **is not "publishing a new version"**—logical content unchanged by a single byte, only physical location shifted; under settled item 14 definition of `birth(b)` as "checkpoint number where block was **published**", relocation generates no new publication event, `birth` traveling with unit as-is.

**Scope**: Governs only whether `birth` travels with move; does not govern which structures relocation touches (D26 (Background Compaction and Placement Reclamation) settled item 5). Deadlist entries recorded by version identity rather than placement and carrying no location hints downstream from same rationale: recording by placement forces rewriting deadlist row-by-row on every move, hints buying nothing (releases universally routed through mapping, D19 (Block Pointer Structure and Width Budget) settled item 5 Hard Rule 1) while leaving implementation open to double-freeing; entries formatted as field table in settled item 12. Deletion path retrieves birth tree from pointer without reading unit header (D19 (Block Pointer Structure and Width Budget) settled item 7).

**Basis**:

- E94 (Touch Surface of Moving Shared Placements): If relocation reassigns `birth`, reassignment violations equal moves referenced by snapshots—erroneous on every unit moved while referenced; leaving `birth` unchanged yields constantly 0 violations.

**Owed**: None.

#### Settled Item 16: Which Side Deadlist Merges to Upon Snapshot Destruction

**Settlement**: Adopts Convention B—merges deadlist to **next newer side** upon snapshot destruction.

**Scope**: Convention B is **not** "snapshot deadlist never rewritten"—target `dsl_deadlist_merge(&ds_next->ds_deadlist, ...)` in destruction cascade involves `ds_next` which is itself a snapshot; rationale for Convention B is exceptions being an order of magnitude fewer, not nonexistent. Difference between two conventions is **frequency of rewriting published snapshot deadlists**, not whether rewritten—alternative convention appends to published snapshot deadlist on every kill, whereas Convention B rewrites once during destruction cascade. Tight bound `birth ≤ prev(S).txg` in I-3.6 (deadlist tight bound) written under Convention B; `prev` propagation in cascade in settled item 13; branch point lacking unique next side hence forbidden from destruction in settled item 18.

**Basis**:

- No experiment: Difference between conventions is frequency of rewriting published snapshot deadlists; project has not measured this frequency nor possesses second measurable quantity; clue is production implementation—ZFS implements Convention B (`module/zfs/dsl_destroy.c`: `/* Merge our deadlist into next's and free it. */`), unverified in this project.

**Owed**: None.

#### Settled Item 17: Four Premises; Crossing Any One Invalidates It

**Settlement**: Four premises—(1) Linear history (no forks); (2) Single writable head; (3) Each block at most one live reference in live tree; (4) Snapshot points aligned with checkpoint boundaries (D16 (Publish Semantics) settled item 14). **Any feature introducing "second live reference to same block" must reopen D5 (Snapshot / Space Accounting Mechanism).** This reopening clause triggered once and converged: D6 (Snapshot Implementation Model) settled item 1 adopted "one tree per head", cross-head shared blocks constituting second live reference; ⇒ converged following correct shape provided by D5 (Snapshot / Space Accounting Mechanism): **D5 (Snapshot / Space Accounting Mechanism) primary + sparse side table auxiliary (ZFS BRT shape)**—not replacing D5 (Snapshot / Space Accounting Mechanism) with global refcounts; placement and field table of side table in settled item 6, three accompanying requirements in settled item 18.

**Scope**:

- Premise 3 mechanistically incompatible with reflink / dedup / block cloning: with second live reference, removing one does not constitute death; criterion inspecting only `birth` and `previous_snapshot_txg` with both references sharing identical birth judges "immediate release" on first removal, leaving second reference dangling. ZFS lacked reflink for twenty years; 2023 block cloning repaired it by **adding separate reference-counted table**, not extending birth/death (clue, unverified in this project).
- Invalidation mode after crossing Premises 1 and 2 is **unbounded leaks + accounting degrading to reference counting**, not data loss: maintaining "whether last live reference" requires per-block branch coverage information, namely reference counts, exactly what D5 (Snapshot / Space Accounting Mechanism) sought to eliminate. **O(1) property of D5 (Snapshot / Space Accounting Mechanism) rests in this state**, repaired by sparse side table.
- ~100× cost reduction in "D5 (Snapshot / Space Accounting Mechanism) + sparse side table" arm serves only as **unverified order-of-magnitude indicator, forbidden from being written into verdicts as verified gain**: its sole originating arm lacked failure clauses and positive controls; block-layer independent verification and fault injection self-checks of Arm A / Arm B never performed on it; usage requires supplementing failure clauses and positive controls before rerunning.
- Extrapolation boundary of side table scale: probes utilized in-memory global bitmap at 1 bit/block (~128 MB/TB on real file systems), overestimating side table performance on large disks; replacing with genuine sparse structures requires retesting. Locality also measured: switching deletion sequence from uniform random to sequential reduced reference counting 8× tier from 174 998 to 391 I/Os (448×); with real-workload locality, advantage of D5 (Snapshot / Space Accounting Mechanism) noticeably smaller than primary table magnitude, **mandatory to quote alongside main conclusion**. Checker cost and crash consistency outside scope of that arm.

**Basis**:

- E18 (Whether D5 Scraps Under Multiple Forks / Multi-Writable-Heads): False liveness 8.19% on single fork, 29.40% on 8 writable heads; under 8 writable heads correct rule matches "record no death" bit-for-bit—trading "reclaim zero blocks" for zero data loss.
- E12 (Batched Sequential Appends vs Non-Batched Random Page RMW): Median device I/O across 200,000 deletions: D5 (Snapshot / Space Accounting Mechanism) constantly 2 across three tiers, global reference count 512 / ~200 208 / ~350 042, D5 (Snapshot / Space Accounting Mechanism) + sparse side table 508 / ~1 956 / ~3 466; reference counting almost free when counter table fits in cache, representing valid result rather than bug—advantage condition of D5 (Snapshot / Space Accounting Mechanism) is "counter table exceeds cache", not unconditionally true.
- E52 (Operating Costs of Two Multi-Writable-Head Mechanisms): Side table entry counts across two cells in converged shape are 1 000 and 50 000 rows.

**Owed**: None.

#### Settled Item 18: Three Clauses to Add for Clones / Writable Snapshots

**Settlement**: Three clauses added after Premises 1 and 2 crossed. With D6 (Snapshot Implementation Model) settled item 1 adopting one tree per head, these three are mandatory, not for reference.

1. Each writable head maintains own `previous_snapshot_txg` and deadlist; **`previous_snapshot_txg` of new clone must initialize to origin snapshot txg**—setting to 0 causes deleting inherited old block in clone to compute "immediate release", causing **data loss**.
2. **Branch point snapshots forbidden from destruction** (Convention B lacks unique next side at branch point). Criterion is O(1) child count `children > 1` → reject, `children` **counting writable heads cloned from this snapshot**. **Second gate: writable heads with live snapshots of their own forbidden from destruction**—destroying a writable head requires destroying all snapshots created by it first; origin snapshot attached during cloning does not count as its own.
3. **Destroying a writable head requires second structure** (livelist shape: ALLOC/FREE pairs, grouped by birth txg, incremental, condensable). Without it, "never traversing post-hoc" is false on this path—what needs releasing are blocks with `birth > origin.txg`, scattered across tree.

**Scope**:

- Second gate buys literal truth of clause 3 "what needs releasing are blocks with `birth > origin.txg`": at moment of destruction, head's own snapshots all destroyed, `previous_snapshot_txg` rewound to clone time via `prev` propagation of settled item 13. Without this gate, `previous_snapshot_txg` at head destruction could exceed clone time; blocks with `birth ≤ previous_snapshot_txg` still referenced by own snapshots cannot release, while destination for these blocks undefined across repository—merging into nearest snapshot deadlist releases block still referenced by clone upon subsequent destruction; consuming with livelist leaves blocks permanently unreleased once `prev` inherited earlier (C269 (blocks left when destroying writable head lack destination), cleared).
- **Cost, knowingly accepted**: Repository lacks promote; while snapshot of a head is cloned and clone survives, snapshot cannot delete, hence head cannot delete (`children` of branch point gate counts clones). Isomorphic to D6 (Snapshot Implementation Model) Criterion 5 "evading via feature absence is not satisfaction", this is second such constraint in repository.
- Livelist key, width, and shared tree in D6 (Snapshot Implementation Model) settled item 2 / settled item 3.
- Two corresponding ZFS gates in local `fs-refs/zfs/module/zfs/dsl_destroy.c`: lines 717–723 verbatim "Can't delete a head dataset if there are snapshots of it. (Except if the only snapshots are from the branch we cloned from.)", and `dsl_destroy_snapshot_check_impl`: `/* Can't delete a branch point. */`—clue, unverified in this project.

**Basis**:

- E146 (Width and Cost of Livelist Entries Identified by Mapping Key): Width and cost of second structure required by clause 3 measured after entry identity takes central mapping key; bill paid for day-1 registration documented there.
- Three-party review: Second destruction gate established in Round 2 of three-round confrontation of D6 (Snapshot Implementation Model) settled item 2; user decision, original wording in history.

**Owed**: C265 (Writable head destruction release boundary fixed to clone time)—clause supplemented, red-turning check remains owed; C270 (Whether fork point gate child count counts clones)—likewise.

#### Settled Item 19: Runtime Form of Liveness Judgment and Cost of Snapshot Destruction

**Settlement**:

```
judge(ptr):
    birth = ptr.birth              // Pointer inline field, zero extra I/O
    p     = head.previous_snapshot_txg     // Scalar in live head header, resident in memory
    n     = sparse_side_table[mapping_key(ptr)]  // Cross-head sharing count, settled item 6 (3); treated as 1 if no entry
    if n > 1:       decrement count, neither release nor enter deadlist (another head still references it)
    elif birth > p: release immediately -> enter defer queue (not directly into free table, see D16 (Publish Semantics) settled item 15)
    else:           append to head.deadlist
```

**Cost of destroying snapshot is O(deadlist entry count + bucket count), not O(full disk)**, provided deadlist is **bucketed by birth txg, bucket boundary = snapshot txg** (epoch segment of settled item 12 is this bucketing); otherwise filtering on destruction degrades to full table scan.

**Scope**: Reduction proof—suppose deletion occurs at checkpoint `d`, P being newest snapshot at this moment. (⇐) If `birth ≤ P.txg`: P published, so `P.txg < d = death`, thus `birth ≤ P.txg < death`, exists S referencing it. (⇒) If exists S such that `birth ≤ S.txg < d`, then S existed at deletion, so `S.txg ≤ P.txg`, thus `birth ≤ P.txg`. ∎ Relies on three facts: total order of txg, **judgment necessarily occurring at moment of death**, and querying sparse side table first for cross-head shared blocks; third does not violate O(1), side table being single point query with key as central mapping key. Violating any removes O(1): multi-writable-head of D6 (Snapshot Implementation Model) violates first (handled in settled item 17), batched deletions of D8 (Core Index Structure) violates second (handled in settled item 20). Reduction is **proof, not measurement**, proving "judgment remains O(1) after restoring Premise 3".

**Basis**:

- E12 (Batched Sequential Appends vs Non-Batched Random Page RMW): Reading two scalars with zero extra device I/O constantly yields 2 device I/Os across three cache tiers, whereas one refcount per block jumps by an order of magnitude depending on cache fit, establishing cost differential of this form relative to refcounts.

**Owed**: None.

#### Settled Item 20: Whether Pending-Deletion Structures Count as Live or Dead

**Settlement**: Pending-deletion structures (intention keys, deleting queue) **count as live references**. Thus `death` = checkpoint of physical batch, judgment naturally occurring at moment of death, **preserving O(1) without introducing new invariants**. Exception: destroyed snapshots cease pinning blocks from moment intention is written (settled item 12).

**Scope**: D8 (Core Index Structure) settled unbounded deletions follow "write intention → batching → resumable"; judgment therefore occurs at physical batch while logical death occurred earlier, breaking second premise of settled item 19 reduction, this item being selected disposition. Cost is user-visible phenomenon: snapshots created after `rm` returns **pin** unreleased extents, `df` recovering only after snapshot death, partially covered by I-3.4 (available space deducts pending deletion usage) and I-4.7 (deleting inodes accounted for) invariants (both implemented in checker; see status columns of I-3.4 (available space deducts pending deletion usage) row in `crates/singlefs-checker/invariants/05-空间记账.md` and I-4.7 (deleting inodes accounted for) row in `06-事务与崩溃一致性.md`). Alternative path (treating pending-deletion structures as dead) destroys O(1): correct deadlist belongs to "first snapshot created after logical death", which does not yet exist when intention is written, requiring querying snapshot list during batch execution. From which moment destroyed writable head ceases counting as live reference, and whether branch point gate `children` decrements upon writing intention or completion, lack clauses. "Live reference" in settled items 14 and 17 refers to live tree references; read-only snapshots are not live trees.

**Basis**:

- No experiment: Difference between two paths is structural property of "when judgment occurs", not measurable cost; selection directly determined by second premise of settled item 19 reduction proof, without measurable quantity.
- Verdict `research/prompts/m4-snapshot-r1-main-verification.md` Section 3 S9: "Live reference" in settled items 14, 17 versus settled item 20 are two measurement bases; exception sentence revised to "ceases pinning blocks".

**Owed**: C337 (No clause for when destroyed head ceases to count as live reference).

#### Settled Item 21: Whether Snapshot List Needs an Enumerable Persistent Structure

**Settlement**: Required. An enumerable persistent structure must exist on disk whose elements are precisely all current live snapshots (neither more nor less), invariant being I-3.7 (live snapshots enumerable).

**Scope**: Requires only "existence and enumerability", without settling which tree it resides in or entry shape. Scanning checker (second direction required by D13 (Verification Route) settled item 12) requires three items to judge attribution: `birth` inlined in pointer (available), deadlist itself being scannable self-describing tree (available), **txg list of all roots** (available: the set of code 14 entries in the tree table without the being-destroyed mark). Checker for I-3.7 (live snapshots enumerable) implemented.

**Basis**:

- No experiment: Format existence requirement with no measurable quantity; discriminative power provable only by implementing checker for I-3.7 (live snapshots enumerable), which is implemented.

**Owed**: C144 (Interval labeling of snapshot lineage lacks carrier)—account for "txg list of all roots" recorded under its name.

````

**出处 `.claude/kb/decisions/05-snapshot-space-accounting-mechanism.en.md:464-467`（整段抄，未转述）**

```markdown
## Version History

D5 (Snapshot / Space Accounting Mechanism) historical entries are collected in [decisions-history.md](../decisions-history.md).

```

**出处 `.claude/kb/decisions/05-スナップショット-空間会計メカニズム.ja.md:1-6`（整段抄，未转述）**

```markdown
## D5 スナップショット / 空間会計メカニズム —— 確定

[中文](05-快照-空间记账机制.md) · [English](05-snapshot-space-accounting-mechanism.en.md) · **日本語**

D5 (スナップショット / 空間会計メカニズム) は二つの事項を管轄する：ブロックのライフサイクル判定（誕生世代番号 + deadlist、確定項目 13 から 21）と空間会計（統計量リスト、キャリア、キー幅、新規プールでのファイル作成時に書き出される行、確定項目 1 から 11）；deadlist エントリの形態は確定項目 12 にある。会計はコミットパス上で増分維持されるものであり、事後トラバースによって得られるものではない。これが D5 (スナップショット / 空間会計メカニズム) の全価値である。管轄外：「ある配置先が現行バージョンに属するか否か」という述語の権威記録位置は D3 (領域割り当て) 確定項目 1 であり、D5 (スナップショット / 空間会計メカニズム) はその半分のみを供給する——deadlist に入った配置先はスナップショットからのみ参照され、生存ツリーには存在しない；受入不等式の権威形式は D28 (マウント期コミットメント量) 確定項目 1 にあり、D5 (スナップショット / 空間会計メカニズム) は式中の各項を誰が維持するかのみを管轄する（確定項目 4 の「式各項対照」表）；複数書き込み可能ヘッドでどのメカニズムを採用するかは D6 (スナップショット実現モデル) 確定項目 1、livelist とその共有ツリーは D6 (スナップショット実現モデル) 確定項目 2 / 確定項目 3；ツリーテーブルエントリのフィールドと幅は D8 (コアインデックス構造) 確定項目 8；中央マッピングキーは D19 (ブロックポインタの構造と幅の予算) 確定項目 6；会計エントリの `seq` は D8 (コアインデックス構造) 確定項目 7。D5 (スナップショット / 空間会計メカニズム) の O(1) 判定性質は D6 (スナップショット実現モデル) が複数書き込み可能ヘッドを採用したことで失効しており、それを回復したのが確定項目 6 のスパースサイドテーブルである；4つの前提と失効形態は確定項目 17 にある。

```

**出处 `.claude/kb/decisions/05-スナップショット-空間会計メカニズム.ja.md:7-463`（整段抄，未转述）**

````markdown
### 確定項目

| # | 項目 | 決定事項 |
|---|---|---|
| 1 | **次元タプルにどの次元を含めるか** | ツリーIDの他に「デバイス」次元のみを追加し、キーは `(統計量, ツリーID, デバイス, 世代)` と表記 **状態: 確定。** **進捗: 実装済み。** |
| 2 | **「世代」に何を採用し、旧世代をいつ破棄できるか** | 世代には checkpoint 番号を採用し、行キーに記録する；各ルートの会計ツリーはそのルートの世代の行のみを収容し、複数世代を保持しない；inode 番号高水位の段差行は例外とする **状態: 確定。** **進捗: 実装済み。** |
| 3 | **会計のキャリア形態** | 独立キースペースの btree に常駐し、エントリは「(統計量, 次元タプル, 世代) → 完全値」とし、write buffer フロントエンドを経由してコピーオンライト方式で親ポインタ下にぶら下がる **状態: 確定。** **進捗: 実装済み。** |
| 4 | **どの統計量が存在するか** | 現役12項目、番号は14まで（第8 / 9項は撤回済み、番号は再利用しない）；デバイス次元付き6項目、ツリー次元付き1項目 **状態: 確定。** **進捗: 実装済み。** |
| 5 | **会計キー各セグメントの幅** | `(タグ 2, ツリーID 8, デバイス 4, 世代 8)` ⇒ キー 22 バイト；値 8 に seq 4 を加え ⇒ エントリ 34 バイト **状態: 確定。** **進捗: 実装済み。** |
| 6 | **スパースサイドテーブルの配置先とフィールド表** | 自ら一ツリーを形成し day-1 に登録；エントリ = 中央マッピングキー 27 + カウント 4 = 幅 31 バイト；コミット時に維持し、削除は意図メカニズムに従う **状態: 確定。** **進捗: 実装済み。** |
| 7 | **「割り当て済みバイト」に mkfs が書き込んだ固定構造を含むか** | 含まない: 割り当て済み = 割り当てレコードツリー内の配置先の総和、受入の「容量」= ユニット領域サイズ **状態: 確定。** **進捗: 実装済み。** |
| 8 | **新規プールでのファイル作成時に会計ツリーへ書き込む行** | 値のある統計量は1行書き込み、対象のない統計量は行を書かない；新規プールでのファイル作成時は15行書き込む **状態: 確定。** **進捗: 実装済み。** |
| 9 | **書き込み可能ヘッドをどのIDで識別し、`previous_snapshot_txg` は誰に帰属するか** | ヘッドID = そのヘッドの inode ツリーのツリーID；前駆スナップショット番号はそのヘッドの inode ツリーのツリーテーブルエントリおよび各コード14スナップショットエントリに記録し、その他は 0 を書き 0 = 該当なしとする **状態: 確定。** **進捗: 実装済み。** |
| 10 | **統計量タグ登録表と二つの予約値** | タグコード = 確定項目 4 の表の行番号；デバイス次元を持たないデバイスセグメントには 0xFFFF_FFFF、ツリー次元を持たないツリーIDセグメントには 0 を書き込む **状態: 確定。** **進捗: 実装済み。** |
| 11 | **deadlist のキャリアを day-1 に作成するか** | 作成する: 独立キースペースのツリーとして day-1 にツリーテーブルへ登録、種類コード 8、ツリーID 65543、ルートポインタゼロ **状態: 確定。** **進捗: 実装済み。** |
| 12 | **deadlist エントリの形態** | キー = ヘッドのツリーID 8 + エポック 8 + 中央マッピングキー 27 = 43 バイト、値は空；エントリはエポックによりバケット分割され、引き渡し時にエントリを移動しない **状態: 確定。** **進捗: 未実装。** |
| 13 | **ライフサイクル判定にどのメカニズムを採用するか** | 誕生世代番号 + deadlist を採用: 削除時に `previous_snapshot_txg` と比較し、スナップショット作成時に全体を引き渡し、スナップショット破棄時は全盤スキャンを行わずにより新しい側へマージする **状態: 確定。** **進捗: 未実装。** |
| 14 | **参照区間の必要十分条件** | `birth(b) ≤ S.txg < death(b)`、左閉右開；3つの量にそれぞれ定義あり；判定入力が取得できない場合は既定で deadlist 入りとする **状態: 確定。** **進捗: 未実装。** |
| 15 | **バックグラウンド再配置で `birth` を変更するか** | 変更しない: 再配置はパブリッシュではなく、`birth` はユニットと共にそのまま保持される **状態: 確定。** **進捗: 未実装。** |
| 16 | **スナップショット破棄時に deadlist をどちら側へマージするか** | 規約 B を採用（次のより新しい側へマージ）；「スナップショットの deadlist は決して書き換えない」という意味ではなく、例外が1桁少ないだけである **状態: 確定。** **進捗: 未実装。** |
| 17 | **4つの前提、いずれかを超過すると無効化する** | 4つの前提を逐条固定；同一ブロックへの第2の生存参照を導入する際は必ず D5 (スナップショット / 空間会計メカニズム) を再開；既に1度トリガーされ、収束形態は D5 (スナップショット / 空間会計メカニズム) 主 + スパースサイドテーブル従 **状態: 確定。** **進捗: 対象なし。** |
| 18 | **クローン / 書き込み可能スナップショットに追加すべき3条** | 各ヘッドが独自に前駆スナップショット番号と deadlist を保持し origin に従い初期化；分岐点スナップショットおよび自身の生存スナップショットを持つヘッドは破棄禁止；ヘッド破棄には livelist が必要 **状態: 確定。** **進捗: 未実装。** |
| 19 | **生死判定の実行時形態とスナップショット破棄のコスト** | まずサイドテーブルへ問い合わせ、カウント > 1 なら減算、それ以外は `birth > previous_snapshot_txg` で即時解放、残余は deadlist へ；スナップショット破棄は O(deadlist エントリ数 + バケット数) **状態: 確定。** **進捗: 未実装。** |
| 20 | **削除待ち構造を生存とみなすか死亡とみなすか** | 生存参照とみなす: `death` = 物理バッチの checkpoint、判定は必然的に死亡の瞬間に発生する **状態: 確定。** **進捗: 実装済み。** |
| 21 | **スナップショット一覧に列挙可能な永続構造が必要か** | 必要: ディスク上に現在の全生存スナップショットを正確に要素とする列挙可能な永続構造が存在しなければならない **状態: 確定。** **進捗: 実装済み。** |

#### 確定項目 1: 次元タプルにどの次元を含めるか

**決定事項**: ツリーIDの他に「デバイス」次元のみを追加し、会計キーは `(統計量, ツリーID, デバイス, 世代)` と表記する。デバイス次元を含めるかどうかは D28 (マウント期コミットメント量) 確定項目 1 の式から直接導かれる：各ディスク自身の値をとる項目は含め、プール全体コミットメント量の項目（全レプリカ合算で記録し、各ディスクへ按分控除）は含めない。

**適用範囲**: 次元タプルはすべての統計量で完全に使い切るわけではなく、どの項目がどの次元を用いるかは確定項目 4 のリストにある；二つの予約値は確定項目 10、各セグメントの幅は確定項目 5 にある。本項目は受入不等式自体（D28 (マウント期コミットメント量) 確定項目 1）も統計量の個数（確定項目 4）も決定しない。

**根拠**:

- E71 (会計キーのエントリ数と幅): 項目別次元計算によるエントリ数は「次元完全使用」の上限より1〜2桁低く、予算を先に圧迫するのは `ツリー数 × デバイス数` の積であり、4つの独立変数のいずれでもない。
- E97 (会計と割り当てレコードのエントリ符号化): 各ルートが 1 世代のみを収容するようになった後（判定基準 10 の第 5 回再実行、`research/results/e97-entry-encoding-single-generation-2026-10-20.out`、ノードバイトはリーフのみ計数）、4 段階のキー幅（7 / 14 / 19 / 22 バイト）の会計ツリー高は E97 (会計と割り当てレコードのエントリ符号化) に登録された 4 つの格子点（ツリー数 × デバイス数 = 64 × 8、1024 × 64、8192 × 8、8192 × 64）で段差行 0 段のとき各点で互いに同じ（狭域読み取り 1、2、2、2；広域読み取り 2、3、3、3）であり、格子点の間では保証されない：同じ幾何でツリー数 1 〜 8192、デバイス数 1 〜 64 の 524 288 点のうち、狭域読み取りでは 17 440 点、広域読み取りでは 92 663 点で 4 段階のツリー高が異なる；`crates/` の今日の行集合（プールレベル 3 行 + ディスクごと 6 行 + 1 段の段差行）では 79 〜 141 台のディスクで 7 バイトと 22 バイトのキーのツリー高が異なる。1 世代の計測条件では一括再書き込みとなり、コストは書き出すリーフ数でエントリ幅に従い、ツリー高ではない：`1024 × 64` の狭域読み取りで 22 バイト対 7 バイトは 1.667 倍（81 920 対 49 152 バイト）、広域読み取りで 2.000 倍；`64 × 8` の狭域読み取りでは 4 段階ともリーフ 1 枚（1.000 倍）。セグメント幅にはコストがあり、狭域読み取りでは絶対値が小さい（最大のセルで 22 バイト 507 904 バイト、予算の 2.3%）。
- ユーザー決定、原文は変更履歴にある。

**未済**: なし。

#### 確定項目 2: 「世代」に何を採用し、旧世代をいつ破棄できるか

**決定事項**: 世代には checkpoint 番号を採用し（パブリッシュごとに checkpoint_txg + 1、D16 (パブリッシュセマンティクス) 確定項目 6、したがって世代は5秒ウィンドウではなくパブリッシュに伴って進む）、行キーに記録する。**各ルートの会計ツリーはそのルートの世代の行のみを収容し、複数世代を保持しない**:「会計がルートと共にロールバックする」ことは、各ルートが独自に会計ツリーをコピーオンライト分岐させることによって保証され、どのルートへロールバックしてもその世代の会計が読み出される。例外は inode 番号高水位統計量の段差行であり、ルートと共に永続保持される（適用範囲「inode 番号高水位の段差行」の条項参照）。

**適用範囲**:

- 世代次元は依然として行キーに記録する必要がある：会計エントリは冪等な完全値であり（D8 (コアインデックス構造) 確定項目 1）、同一の統計量が何度も書き込まれるため、スキャン再構築時に確定項目 3 / 確定項目 10 のマージ規則に従って同一統計量の新旧を整列するためにこれを見る必要がある。「会計がルートと共にロールバックする」ことはこれに依存しない：もし会計が最新の値をインプレース上書きする1コピーしか持たない場合、旧ルートをマウントした後に会計は新しくツリーは古い状態となり、I-3.1 (割り当て統計が一致する) が即座に赤判定となる；各ルートが独自に会計ツリーをコピーオンライト分岐させることで、マウントされたルートに対応する世代の会計が読み出される。
- 二つの付帯条項：世代セグメント幅 **≥ 48 ビット**（2785 パブリッシュ/秒において 32 ビットは 17.8 日でラップアラウンド、E71 (会計キーのエントリ数と幅) の幅計算はこれに従い再計算）；会計エントリの**パブリッシュ跨ぎのマージはもはや不可能**（世代がキーに入ったため）、したがって各パブリッシュごとにその世代の行をすべて再書き込みし、書き込み行数は上限ではなく実払い値となる。
- **inode 番号高水位の段差行**（W3 の C2aL 形態）：会計は inode 番号高水位が変更された各パブリッシュにおいて、この統計量に対して段差行 (g, H) を残す。g はそのパブリッシュの txg、H はそのパブリッシュ後の高水位である；段差行はルートと共にコピーオンライトされ、永続保持される。管理者ロールバック時は cur（ロールバック前の現行版）から引き継ぎ、R_old からは取得しない；「各ルートの会計ツリーはそのルートの世代の行のみを収容する」は段差行には適用されない。ロールバック時の inode 番号高水位はクラッシュリカバリで放棄されなかったルートからのみ取得され（D23 (journal の役割とフォーマット) 確定項目 14「高水位」スロット）、ロールバックによって高水位が前進することはないため、段差行は inode 作成およびクローン作成のパブリッシュでのみ書き込まれる；クローンヘッドの段差行は (統計量, ツリー) で分割され、クローン作成時に新ヘッドへ1行残される。段差行と系譜レコードが保持する cur 高水位は「等しい」として相互検証される（D18 (ブロックに含める情報) 確定項目 24 第5条）。I-3.13 (会計行の世代セグメントがこの版の txg と等しい) も段差行には適用されない：例外はここに明記され、不変条件リストの同条項には「inode 番号高水位統計量の段差行は判定しない」と記述されており、プールレベル checker（`crates/singlefs-checker/src/walk.rs` の該当判定箇所が `is_an_inode_number_watermark_step` により段差行を認識してスキップ）はすでにこの例外に従い処理している。コスト: 1行 34 バイト；パブリッシュごとの追加書き込みユニット数は、段差行 462 行目までは 0、463 行目から +2、36 074 行目から +4（三方脚のプランナーモデルによる計算、実プランナーのプローブと突合済み、受入装置では未計測）。

**根拠**:

- E54 (会計の「世代」は何世代保持すべきか): 「世代に何を採用するか」と「何世代残すか」は直交する問題である——破棄しないアームに増分破棄ルールを加えたものは、最新 K 世代のみを残すアームと 60 セルにわたりセルごとに一致する；破棄しない場合、checkpoint 数に対するキー数の傾きは統計量数に等しく、K 世代残す場合の傾きは 0 である。
- メイン agent 2026-09-23 の現地調査: 複数世代の会計を同時に必要とする読者は存在しない——checker は会計行を (統計量, デバイス) で読み取り、世代セグメントは読まない（`crates/singlefs-checker/src/walk.rs` の会計ツリー分岐）；リカバリは選択されたルートの会計ツリーのみを読む（`crates/singlefs-core/src/recovery.rs` の会計ツリーのルート部分）；書き込みパスはパブリッシュごとに現行世代の 15 行を書き込み、ポイント削除は一切行わない（`crates/singlefs-core/src/transaction.rs` の `accounting_entry_count(2)`）。
- ユーザー決定 2026-09-23: 各ルートは自身の世代のみを保持する（マイルストーン「上書き・解放・ロールバック・再利用」収束表第 56 行）、原文は変更履歴にある。
- ユーザー決定、原文は変更履歴にある。
- inode 番号高水位の段差行: W3 判定 `research/prompts/m3-w3-r2-main-verification.md` 第2節攻撃側の各条項（C2aL 最強形は高水位が変更されたパブリッシュごとに段差行を残し、I-3.13 (会計行の世代セグメントがこの版の txg と等しい) および D5 (スナップショット / 空間会計メカニズム) 確定項目 2 に例外を設ける）、`research/prompts/m3-w3-r3-main-verification.md` 第2節クローンヘッド C2aL 行（クローン作成時に行を残す）および第3節コスト表；C2aL の原文定義は第2ラウンド攻撃側報告 `m3-w3-r2-opus-output.md` 内「**C2aL 最強形**（私が提示したもの）」の行にある；ユーザー決定 2026-10-08 Y2「C2aL: 会計に段差行を残す（推奨）」、原文は変更履歴にある。実験なし: 数値は三方脚のモデルおよびリポジトリ複製プロトタイプ上で計測、実験番号は未設定。

**未済**: なし。

#### 確定項目 3: 会計のキャリア形態

**決定事項**: 会計は独立キースペースの btree に常駐し、エントリは「(統計量, 次元タプル, 世代) → 完全値」とする。

| # | 決定された内容 |
|---|---|
| 1 | キャリアは btree 内のキーであり、システム構成内のスカラー群でもディスク上の固定長領域でもない——固定長領域は「どの統計量が存在するか」をフォーマットに固定してしまうが、D12 (対象メディア) はメディアに応じてレイアウト分岐を開放することを確定しており、各系統で統計次元が異なる可能性がある；btree キー形態であれば**新しい統計次元を追加してもフォーマットを変更しない**（bcachefs が2024年にカウンタを btree キーにした形態） |
| 2 | キー = (統計量タイプタグ, 次元タプル, 世代番号)；値 = その統計量のその世代における完全値 |
| 3 | マージ規則は統計量ごとに分類: 非単調統計量は「後勝ち」（後者は seq により決定）、単調統計量は最大値をとる（確定項目 4 の第12、13、14項）。最大値をとるのはスキャン再構築時にディスク上の同一キーの複数の値を順序づけるためだけである（確定項目 2 の適用範囲：「スキャン再構築が確定項目 3 / 確定項目 10 のマージ規則で同一統計量の新旧を順序づけるにはこれを見る必要がある」）；フロントエンドをリーフまでドレインするとき、およびキーを読む際にフロントエンドとリーフが併存するときは、いずれも最大値をとらず、D8 (コアインデックス構造) 確定項目 13 に従う：リーフ上の現在値を起点に到着順に適用し、墓石は削除である。ポイント削除の墓石は最大値取得に参加しない；墓石を数値として max に畳み込むとポイント削除された世代が永続残留することになる |
| 4 | write buffer フロントエンドを経由し、エントリ自身が `seq` を保持——D8 (コアインデックス構造) は write buffer が「逆インデックス / LRU / 会計」の3種をカバーすることを確定しており、seq はその第3の硬性要求である（ソートによって時間順序が破棄されるため、「後者」はエントリ自身が保持しなければならない）。第1版の会計は葉へ直接配置され write buffer を経由せず、`seq` は一律 1（D8 (コアインデックス構造) 確定項目 10）、フィールドは不変 |
| 5 | 会計ツリー全体がコピーオンライト方式で親ポインタ下に配置される（D22 (ユニット原子性をどう合成するか) 確定項目 3） |

**適用範囲**:「フォーマット変更なしの統計次元追加」と I-3.1 (割り当て統計が一致する) の判定力を獲得し、キー数の増加（各統計量 × 各世代が1エントリを占有）という代償を支払う。倍率は未計測であり、D8 (コアインデックス構造) は計測前にこれをもって「会計がどれだけの領域を占めるか」を推定することを禁止している。「タグ → マージ規則」はディスク上セマンティクスであり、統計量登録表は独立した凍結コンポーネントとして設立される（D15 (フォーマット凍結ポリシー)）；登録表自体は確定項目 10 にある。もしある種の会計がどうしても冪等性を達成できない場合、その種別の会計は実行時と判定コードを共有しない独立した再計算パスを持たなければならない。

**根拠**:

- E14 (I-3.1 の判別力: 冪等な完全値 vs 増分 Δ): 同一ツリー同一負荷に4種のセマンティックバグを注入したところ、冪等な完全値形態では I-3.1 (割り当て統計が一致する) が 3/3 を検知し死角ゼロ、増分 Δ 形態では 0/4——値を完全値とすることが I-3.1 (割り当て統計が一致する) が判別力を保つ前提である。
- E38 (会計構造の COW かインプレースか): インプレース形態では mkfs が全ノードの固定位置をディスク上に予約する必要があり、COW は常に 0 である；これは COW 側の最も強固な根拠である。
- 3つの前提条件の連言: D8 (コアインデックス構造) 確定項目 1 が値のセマンティクスを決定、D22 (ユニット原子性をどう合成するか) 確定項目 3 がディスク永続化手法を決定、D21 (権威態と派生態の分界) がルートによる被覆とフォーマット凍結への参加を決定；`.claude/rules/fs-design.md` 第1枠がトランザクションコミットパス上での増分維持を決定。ユーザー決定、原文は変更履歴にある。
- 確定表第 3 行の墓石の畳み込み方：判定 `research/prompts/m3-close-c693-r2-main-verification.md` 第 2 節 V4 のセル（「フロントエンドの墓石をリーフ上の前回パブリッシュの現在値に対してどう畳み込むか」に縮めて D8 (コアインデックス構造) 確定項目 13 に統合、両側一致）と `research/prompts/m3-close-c693-r3-main-verification.md` 第 2 節 T4 のセル（題を縮めて D8 (コアインデックス構造) 確定項目 13 に統合、維持）；改め方は順推側 `research/prompts/m3-close-c693-r2-sonnet-output.md`「V4.5 変更すべき条項、未済の検査」の節に従う。

**未済**: C383 (同一パブリッシュ内での同キー中間状態の折りたたみ検査欠落)。

#### 確定項目 4: どの統計量が存在するか

**決定事項**: 現役の統計量は12項目、番号は14まで（第8 / 9項は撤回済み、番号は再利用しない）、デバイス次元付き6項目（第1 / 2 / 3 / 5 / 10 / 11項）、ツリー次元付き1項目（第12項）。リストの完全性は D28 (マウント期コミットメント量) 確定項目 1 の受入不等式に対して判定される——式に現れる各項は必ず維持される統計量でなければならず、いかなる判定にも現れないものは含めない；それが不可能な項目は「式各項対照」表においてその項目が何であり、なぜかを記述しなければならない。

決定事項と共に発効する付帯条項が2件ある：

- **第2項（空きバイト数）は独立に維持されなければならず、`容量 − 割り当て済み` からその場で算出することは許されない。** I-5.2 (空き領域統計が一致する) の条文は文字通り「空き領域統計 == 総領域 − 割り当て済み領域」であり、空き領域をこのように算出するとその不変条件は恒真式となり判別力はゼロとなる；二つの数値はコードを共有しない加減算パスを通らなければならない。
- **ライフサイクル判定は設計上いかなる集約統計量も必要としない。** 確定項目 19 の O(1) 形態は「生死判定は `ptr.birth` と `head.previous_snapshot_txg` のみを用い、会計ツリーを検索しない」という点に立脚している；生死判定のために会計ツリーに統計量を追加することは、証明されたその性質を破壊することになる。

**式各項対照**（式の権威正文は D28 (マウント期コミットメント量) 確定項目 1 にある；本表は式と統計量リストの間の唯一の対照位置であり、ゲート doc-registries の admission-terms セルが両者を逐項照合する）：

<!-- gate:admission-terms -->

| 式中の項目 | 誰がそれを維持するか |
|---|---|
| 容量 | **例外**: デバイス幾何形状であり、会計統計量ではない——デバイスとシステム構成から読み取り、マウントごとに取得し、会計はこれを維持しない |
| 已分配 | 第1項（割り当て済みバイト） |
| 不可回收 | 第3項（回収不能バイト） |
| 挂载期承诺量 | **例外**: メモリ内にのみ存在しディスクに永続化されず、マウントごとに再計算（D28 (マウント期コミットメント量) 確定項目 3）、パブリッシュ済み統計量ではないため統計量リストには入らない；デバイスごとに計算されクラッシュで破棄（プールレベルの checkpoint 保持プールは単独計上、D28 (マウント期コミットメント量) 確定項目 4） |
| 待删占用 | 第4項（削除待ちだが意図未完了の占有） |
| 已承诺预留 | 第6項（コミット済み予約量（墓石を含む）） |
| 被抛弃根独占量 | **例外**: メモリ内にのみ常駐する派生量——値 = アロケータが実際に隔離した集合のスロット数：放棄されたルートからのみ参照されるスロット（放棄されたルートが参照するスロット − ロールバック候補集合内のルートが参照するスロット、下限0；各ルートが参照するスロットはその割り当てレコードツリーから読み出し、プール全体の全走査は行わない）、および書き込み可能マウント時の事前隔離スロット（D23 (journal の役割とフォーマット) 確定項目 14「書き込み可能マウント時の事前隔離」）を加え、アロケータポインタが位置するスロットのみを除外；マウントごとおよび F 引き上げごとに算出され、放棄されたルートがローテーションで上書きされた際にクリア（D28 (マウント期コミットメント量) 確定項目 1 第9項）；パブリッシュ済み統計量ではないため統計量リストには入らない |
| checkpoint 保留池 | **例外**: マウント期コミットメント量のプールレベルメンバー、当時の各レコードツリーの高さの合計 + 会計ノードからその場で算出（D28 (マウント期コミットメント量) 確定項目 4）、パブリッシュごとに再計算、メモリ常駐のみ、全レプリカ合算で記録しデバイスごとに按分控除、パブリッシュ済み統計量ではない |
| 环窗口 | **例外**: 削除各バッチの新書き込みに対する安全マージン、(ルートリングスロット数 3 × S + 1) × 各バッチ上限 W（D28 (マウント期コミットメント量) 確定項目 1）；メモリ常駐のみ、会計行に入らず、パブリッシュごとに幾何形状から算出、checkpoint 保持プールと同種であり、パブリッシュ済み統計量ではない |

5つの例外はいずれも「後で補うために一時記録しておく」ものではない：デバイス容量、マウント期コミットメント量、放棄ルート排他占容量、checkpoint 保持プール、リングウィンドウはクラッシュ後に再構築すべき会計に含まれず、これらに統計量を設けることはマウントごとに再計算される量に対して永続会計を行うことに等しい。リストの完全性は表内で「第 N 項」を指す項目に対してのみ判定される。

**統計量リスト**:

| # | 統計量 | 誰がそれを必要とするか | デバイス次元の有無 | ツリー次元の有無 |
|---|---|---|---|---|
| 1 | 割り当て済みバイト | 割り当て、受入、I-3.1 (割り当て統計が一致する) | **あり** | なし |
| 2 | 空きバイト数 | 受入、checker | **あり** | なし |
| 3 | 回収不能バイト | 受入、zoned 系統の割り当て、ユーザー可視の df | **あり** | なし |
| 4 | 削除待ちだが意図未完了の占有 | 受入（I-3.4 (利用可能領域からの削除待ち占有控除)）；恒常的0の予約枠: 削除中の inode の未解放ユニットは元々「割り当て済み」に含まれる（D28 (マウント期コミットメント量) 確定項目 1） | なし（プール全体コミットメント量） | なし |
| 5 | defer キュー解放待ち（世代別） | defer ウィンドウ、割り当て、受入 | **あり** | なし |
| 6 | コミット済み予約量（墓石を含む） | 受入（D3 (領域割り当て) 確定項目 15 + 確定項目 2）；各生存オブジェクトの墓石割り当て分、各 inode レコードの退役レコード割り当て分、およびプール全体の常駐マージン（墓石ツリー 2 + H_墓石 + 1 スロット、退役ツリー 2 + H_退役 + 1 スロット、H は各現行版の内部層数；計算式は D3 (領域割り当て) 確定項目 2 参照）のみを含み、リングウィンドウは含まない | なし | なし |
| 7 | 拡張ポイントクォータ使用量 | 受入（D21 (権威態と派生態の分界) 確定項目 17 第4条；C21 (拡張ポイントポインタが会計から逸脱)） | なし | なし |
| 8 | **撤回済み・この番号は再利用しない** | かつて「ツリー別排他バイト」と定めた。撤回根拠: 三方論証の2脚による独立全数調査（14種の判定パス + 66条の不変条件 + 129条の未済の検査 / 6パス）で**ヒットゼロ**、かつ確定項目 4 自らが「生死判定は会計ツリーを検索しない」と明命している——記載漏れではなく、阻止されたものである；唯一の消費者「スナップショット使用量」は [pitfalls.md](../pitfalls.md) で**未決**とマークされたユーザー可視機能に対応する。これにより**第1版ではスナップショットごとの使用量を提供せず、`df` はプール全体の量のみを報告する**。将来 per-snapshot クォータを実装する場合、それは実行時受入が読み取る量となり、即座に `.claude/rules/fs-design.md`「会計はトランザクションの副産物」の第1枠へ回帰し、キー符号化は D15 (フォーマット凍結ポリシー) 第3層で凍結されるため、凍結前に決定する必要がある；この枠に全リポジトリで未済の検査はない | — | — |
| 9 | **撤回済み・この番号は再利用しない** | かつて「ツリー別共有バイト」と定めた。第8項と同時に撤回、同一の根拠 | — | — |
| 10 | 断片化度 `F = runs / L` の runs: **ユニット領域空きスロットの連続区間数** | 整理のトリガー判定（D26 (バックグラウンド整理と配置回収) 確定項目 6） | **あり**（物理量、デバイスごとに1行） | なし |
| 11 | 完全空クラスタセグメント数 | 整理のトリガー判定 + 割り当て自立性（第10項と同等） | **あり**（物理量——**推論**、根拠は D26 (バックグラウンド整理と配置回収) 確定項目 6 参照） | なし |
| 12 | inode 番号高水位（次の利用可能 inode 番号） | inode 番号割り当て（D8 (コアインデックス構造) 確定項目 6: inode 番号は単調非再利用、次の番号は問い合わせの瞬間に回答が存在しなければならない） | なし | **あり**（書き込み可能ヘッドのみ維持） |
| 13 | 清掃高水位（どの txg までの解放が全清掃により読み取り不能と確認されたか） | ガベージ清掃の受入および墓石 / 退役レコードの回収ゲート（C113 (スキャン再構築時に複数バージョンユニットの現行バージョン判定に入力がない) 決定事項 P4 / P5） | なし（プール全体量: 全デバイスをカバーする清掃完了時にのみ前進） | なし |
| 14 | 直近のルート破棄世代番号 | 墓石回収ゲートの下限（C113 (スキャン再構築時に複数バージョンユニットの現行バージョン判定に入力がない) 決定事項 P5: ゲート = max(死亡世代番号, 本項目)） | なし | なし |

**適用範囲**:

- デバイス次元の有無は D28 (マウント期コミットメント量) 確定項目 1 の式から直接与えられ、キーの次元タプルはすべての統計量で完全に使われるわけではない；ツリーごとに分割されない項目は予約ツリーID 0 を「プール全体 / 帰属なし」行として用いる（確定項目 10）。第10、11項はその式に含まれておらず、デバイス次元の有無は D26 (バックグラウンド整理と配置回収) 確定項目 6 の次元論述に基づく推論であり、モデリングに用いる前に再確認が必要である。
- 第12項は書き込み可能ヘッドに対してのみ維持され、高水位が変更されたパブリッシュ時に段差行を1行残し、ルートと共にコピーオンライトされ永続保持され、毎回のパブリッシュで再書き込みされない（確定項目 2 の、各ルートの会計ツリーはその世代の行のみを収容しパブリッシュごとに全行書き直すという規則に対して例外を設けている。確定項目 2 適用範囲「inode 番号高水位の段差行」参照）。読み取り専用ツリー（スナップショット）はこの行を持たない；ツリー次元を持つため、ツリーIDセグメントはキー内に残る。統計量の現行値を読み取る = その次元タプルの下で世代番号が最大の行を取得する。空は一度も維持されていないことを示し、0 ではない。
- 第13 / 14項がリストに入る理由は「回収判定がそれを必要とする」ためであり、共に単調量である；清掃高水位は全清掃が完了した時にのみ前進し、中途の進捗は永続化されず、一度前進した後に毎回のパブリッシュで再書き込みされる。第1版では墓石を回収せず清掃もないため、これら2項目は第1版では登録されない（D18 (ブロックに含める情報) 確定項目 24）。
- リストは非閉鎖的である: D12 (対象メディア) の単一ディスク系統および zoned 系統は未着手であり、確定項目 3 の根拠は「各系統が独自の統計次元を持ち得る」と認めており、第3項は zoned が先行して組み込まれたものである；系統開始時にリストは再開される。
- 第12項は会計に常駐する。管理者ロールバックはマウント中の前方パブリッシュである（D23 (journal の役割とフォーマット) 確定項目 14）: そのパブリッシュの会計統計量は調整後のアロケータに従ってパブリッシュパスがその場で計算し、R_old の会計からはロードしない。inode 番号高水位は max(メモリ内の現行版, リングから読み取れインスタンス表に従いクラッシュリカバリで放棄されなかったルートのもの) を取得し（D23 (journal の役割とフォーマット) 確定項目 14「高水位」スロット）、後退せず再発番もしない（`research/prompts/m2-rollback-forward-r3-main-verification.md` 第2節 K1；`research/prompts/m2-rollback-forward-r2-main-verification.md` 第2節 G2）。クラッシュリカバリで古いルートへ着地した場合、高水位は選択されたルートの会計と共にロードされ、放棄されたタイムラインで発番済みの番号が再発番される；対処は一意性の狭小化であり、D8 (コアインデックス構造) 確定項目 6 は「パブリッシュ済みの (inode 番号, オブジェクト誕生世代) は再利用しない」に変更され、高水位は従前通りここに常駐する（`research/prompts/c143-r3-main-verification.md` 第4節）。

**根拠**:

- E71 (会計キーのエントリ数と幅): 各ルートが 1 世代のみを収容するようになった後（初回の再実行、`research/results/e71-accounting-keys-single-generation-2026-10-20.out`）、エントリ数の比は `ツリー数 × デバイス数` に伴って増え、定数ではない：30 バイト幅の 15 セルで広域読み取りアームは狭域読み取りアームの 1.00 〜 128.47 倍、上限アーム（次元完全使用）は狭域読み取りアームの 1.00 〜 283.52 倍であり、`ツリー数 64、デバイス数 8` のセルでは 14.53 倍と 28.27 倍である。`T_dirty` の 1%（21 474 836 バイト）を超えるセル：3 種の幅の 45 セルのうち上限アーム 5 セル（旧計測条件では 9 セル）、広域読み取りアーム 3 セル（旧計測条件では 5 セル）、狭域読み取りアーム 0 セル；`1024 × 64` のセルでは、上限アームは 22 と 30 バイト幅で新計測条件では超えず（12 976 128、17 694 720 バイト）、50 バイトで超える（29 491 200）、広域読み取りアームは 3 種の幅いずれも新計測条件では超えず（5 879 808、8 017 920、13 363 200）、旧計測条件では 50 バイトのみ超えていた（26 726 400）；`8192 × 64` のセルでは広域読み取りアームは今日の 34 バイトのエントリ幅（`crates/` の実装幅）で新計測条件でもなお超える。`統計量数 × ツリー数 × デバイス数` は1世代の上限でありエントリ数ではない（inode 番号高水位の段差行は例外: ルートと共に保持され、エントリ数は高水位が変更されたパブリッシュ数に伴い増加し、この上限には拘束されない）。
- E97 (会計と割り当てレコードのエントリ符号化): 1 世代の計測条件（判定基準 10 の第 5 回再実行、ダーティノード計数）。狭域読み取りで最も厳しいセルは `8192 × 8` と `8192 × 64`（同率）：19 バイトアーム（判定アーム）458 752 バイトで予算の 2.1%、22 バイトアーム（`crates/` の今日のキー幅）507 904 バイトで 2.3%；旧計測条件の `8192 × 64` のセルの 19 バイトアーム 11 419 648 バイト、53% は狭域読み取りであり広域読み取りではない。広域読み取りでは `1024 × 64` が 42%（22 バイトアーム 46%）、`8192 × 8` が 47%（22 バイトアーム 53%）で上限内、`8192 × 64` が 337%（22 バイトアーム 375%）で上限超過（旧計測条件では 3 セルとも超過）であり、そのセルは逆算しても実現可能なエントリ幅がない。これが「どの項目をツリー別に分割するか」の列のコスト側根拠の一つである：今日ツリー次元を持つのは第 12 項のみで、書き込み可能ヘッドのためだけに維持され、支払うのは狭域読み取りの段の上限であり、広域読み取りが払えないのは最大のセルのみ；取捨そのものは「誰がそれを必要とするか」に立脚する（本項の完全性判定基準）。
- E93 (エージング下での配置と断片化度): `F = runs / L` は増分維持が可能であり、増分値と全走査監査は全アーム × 負荷 × シード × サンプリング点において点ごとに一致し、整理のトリガー量がリストに入り得ることが証明された。
- 三方論証: 第8 / 9項の撤回時、2本の脚が独立して全数調査を行いヒットゼロ；ユーザー決定、原文は変更履歴にある。
- 第4項は恒常的0、第6項はリングウィンドウを含まない: `research/prompts/m3-delete-r3-main-verification.md` 第3節 G1-タグ 4（ゼロラウンド）、G1-墓石前払い2行、ユーザー決定 K-R3-2「常駐控除項目（推奨）」、原文は変更履歴にある；第13、14項は第1版では登録しない: 墓石判定 `research/prompts/m3-tomb-r1-main-verification.md` 第3節 W3 回収行。

**未済**: C122 (割り当てレコードのポイント削除廃止後のエントリ数計測条件)；C143 (クラッシュリカバリで旧ルートに着地した後に inode 番号高水位が後退し再発番される)；C326 (runs の計測条件変更後に E93 の数値を引用可能か)。

#### 確定項目 5: 会計キー各セグメントの幅

**決定事項**: `(統計量タイプタグ 2, ツリーID 8, デバイス 4, 世代 8)`、値 = 完全値 8 バイト、`seq` 4 バイトは値に常駐（D8 (コアインデックス構造) 確定項目 7）⇒ キー 22 バイト、エントリ 34 バイト。世代セグメントは完全な checkpoint 番号を格納し、剰余算は禁止する。

| セグメント | 幅 | 根拠の等級 |
|---|---|---|
| 統計量タイプタグ | **2** | **自由選択**: 現役12項目、番号は14までであり 1 バイトで十分；2 を採用したのは拡張マージンのため（確定項目 4 はリストが非閉鎖的であると自認し、D12 (対象メディア) はメディア系統数を制限しない）；増えた 1 バイトで会計リーフの容量は 491 件から 477 件に落ちる（`crates/` の幾何での算術、2.9% 減）。ノードを余分に書くかどうかは会計ツリーの総行数（3 + 6 × ディスク数 + 段差行数）がリーフ 1 枚が満杯になる閾値を越えるかどうかで決まる：2 台のディスクで段差行が 463 〜 476 段のとき、21 バイトキーはリーフ 1 枚、22 バイトキーはリーフ 2 枚とルート 1 つで、パブリッシュごとに 2 ノード多く書く |
| ツリーID | **8** | **自由選択**: 上限を拘束する既定条項が一つも存在しない（D6 (スナップショット実現モデル) は書き込み可能ヘッドごとに独自のツリーを持ち、スナップショット数に上限はないと決定）、狭い幅を採用して設計上の上限を捏造することは禁止 |
| デバイス | **4** | **D19 (ブロックポインタの構造と幅の予算) 確定項目 4 に従う**、同一の座標系 |
| 世代 | **8** | **既定条項から直接与えられる**: 確定項目 2 の付帯条項は世代セグメント ≥ 48 ビットを要求しており、D22 (ユニット原子性をどう合成するか) 確定項目 7 の `checkpoint_txg` は 8 バイト |
| 値（完全値） | **8** | 統計量はバイトカウントであり、256 TiB でも 48 ビットしか使用しない |

**適用範囲**: これら5セグメントはマージ結果を確定できるキーを構成しない——会計エントリは冪等な完全値であり、同一の統計量が同一 checkpoint 内で何度も書き込まれるため、`(タグ, ツリー, デバイス, 世代)` は checkpoint 内で一意ではない。「後勝ち」の「後者」は `seq` によって決定される；`seq` が値に常駐するため、これら5セグメントのソート契約は変更されない。ユーザー決定は「まずこの形態で着工し、コード記述時に調整する」である。

**根拠**:

- E97 (会計と割り当てレコードのエントリ符号化): 1 世代の計測条件では 4段階のキー（7 / 14 / 19 / 22 バイト）のツリー高は登録された 4 つの格子点で段差行 0 段のとき互いに同じであり、格子点の間では至る所で同じとは限らず、コスト曲線は平坦ではない——一括再書き込み時に書き出すリーフ数はエントリ幅に従う（リーフのファンアウト 1087 / 740 / 603 / 542；`1024 × 64` 広域読み取り 5 046 272 / 7 405 568 / 9 076 736 / 10 092 544 バイト、狭域読み取り 49 152 / 65 536 / 65 536 / 81 920 バイト）；世代セグメントの mod 2³² のラップアラウンド周期は 17 日、mod 2⁴⁸ は 3204 年であり、いかなる有限の剰余もキー順序の崩壊をラップアラウンドの瞬間まで先送りするに過ぎない（mod 256 では 257 世代で 255 個の逆序ペアが発生し、91 ミリ秒で一巡する）。これが「剰余算禁止」の実行可能な根拠である。
- E44 (シーケンス番号ビット幅のローカル実測とコスト): 48 ビットは 3202 年持ちこたえ、世代セグメント ≥ 48 ビットの付帯条項と同一の桁数である。
- E73 (ノードキー区間のファンアウトコスト): 22 バイトのキーで計算した場合、ノードがキー区間を保持した後のファンアウト低下は最大1段階にとどまり、ツリー高は1層も増加しなかった。
- ユーザー決定、原文は変更履歴にある。

**未済**: C162 (seq が値に入った後に4つの実験数値を再計算)。

#### 確定項目 6: スパースサイドテーブルの配置先とフィールド表

**決定事項**: 4つの問いにそれぞれ1文で回答——① キャリアは自ら独立キースペースの btree を形成し、day-1 にツリーテーブルへ登録（新規プールでのファイル作成時に 200 バイトのツリーテーブルエントリが1件増加、ルートポインタはゼロ）；② キーは中央マッピングキーを採用（D19 (ブロックポインタの構造と幅の予算) 確定項目 6: コード1は 27 バイト、コード2 / コード3の 25 バイトは末尾をゼロ埋めして 27 バイトとする、D19 (ブロックポインタの構造と幅の予算) 確定項目 10）、バージョン次元は持たない——マッピングキー内の誕生 txg がすでにバージョン識別子である；値は 4 バイトの参照カウントであり、エントリ幅は 31 バイト；③ コミット時に増分維持され、`.claude/rules/fs-design.md` 第1枠に該当（ヘッド間共有ブロックの生死判定がこれを読み取る）；④ D8 (コアインデックス構造) 確定項目 5 の意図メカニズムに従い、削除はバッチ分割される。

**適用範囲**: 新規プールでのファイル作成時のバイト数を変更——ツリーテーブル内のスパースサイドテーブルのエントリ（200 バイト）、エントリ自体は新規プールでのファイル作成時に1行も書き込まれない。これは「どこに常駐し、エントリはどのような形態か、誰が維持し、意図メカニズムに従うか」の4問にのみ回答する；ヘッド間共有がどのメカニズムを採用するかは D6 (スナップショット実現モデル) 確定項目 1 にあり、サイドテーブルは確定項目 17 の超過された前提を修復するものである。D26 (バックグラウンド整理と配置回収) 確定項目 5 に残されていた問い（deadlist エントリに何を格納するか）は本項目と共に収束した：エントリはバージョン識別子によって記録され、フィールド表への着地は確定項目 12 にある。ユーザー決定は三方論証を経ておらず、覆され得る。

**根拠**:

- E52 (二つの複数書き込み可能ヘッドメカニズムの運用コスト): サイドテーブルのエントリ数は2セルでそれぞれ 1 000 および 50 000 行であり、規模は確かにスパースである。
- E12 (バッチ順次追記 vs 非バッチランダムページ RMW): キャッシュに収まる際はほぼ無料、収まらない際は削除ごとに約1回のデバイス I/O。
- E146 (マッピングキーで識別される livelist エントリの幅とコスト): 同型構造の day-1 登録で支払うべきコスト（1件のツリーテーブルエントリ、空ツリーがルートノードを要する場合はさらに1ユニット）。
- ユーザー決定、原文は変更履歴にある。

**未済**: C236 (スパースサイドテーブルが必須と確定されたが配置先が一切存在しない)——立項側の半分は返済済み、赤となる検査は依然として未済。

#### 確定項目 7: 「割り当て済みバイト」に mkfs が書き込んだ固定構造を含むか

**決定事項**: 含まない。「割り当て済みバイト」= 割り当てレコードツリー内の配置先の総和；受入不等式内の「容量」= ユニット領域サイズ（システム構成の「ユニット領域開始スロット番号」からディスク末尾まで）。固定構造（システム構成スロット、ルートリング、journal リング）は容量にも含まれず、割り当て済みにも算入されない；I-5.2 (空き領域統計が一致する) の「総空間」も同様である。

**適用範囲**: これら二つの量の計測条件のみを決定し、受入不等式自体（D28 (マウント期コミットメント量) 確定項目 1）もユニット領域開始スロット番号の設定（D3 (領域割り当て) 確定項目 10）も決定しない。

**根拠**:

- E142 (新規プールでのファイル作成のドライラン): 新規プールでのファイル作成の割り当て済みは 17 個の 16 KiB スロットであり、固定構造は1バイトも含まれず、フィールドごとに `crates/` 実装と突合して完全一致。
- ユーザー決定およびコーディング前の総監査、原文は変更履歴にある。

**未済**: なし。

#### 確定項目 8: 新規プールでのファイル作成時に会計ツリーへ書き込む行

**決定事項**: 規則は値のある統計量は1行書き込み、**対象のない統計量は行を書かない**（空の読み取りは一度も維持されていないことを示し、0 とは等しくない）。新規プールでのファイル作成時は **15** 行を書き込み、キーは `(統計量タグ 2, ツリーID 8, デバイス 4, 世代 8)` の昇順とし、タグ = 確定項目 4 の表の行番号、デバイス次元を持たないデバイスセグメントには 0xFFFF_FFFF、ツリー次元を持たないツリーIDセグメントには 0、世代 = checkpoint_txg 3 とする：

| 行 | 統計量（確定項目 4 の行番号） | キー | 値 |
|---|---|---|---|
| 1–2 | 已分配字节（1） | (1, 0, 0, 3)、(1, 0, 1, 3) | 各ディスク 278 528 |
| 3–4 | 空闲字节（2） | (2, 0, 0, 3)、(2, 0, 1, 3) | 各ディスク 3 472 605 184（4 GiB イメージの予測値） |
| 5–6 | 不可回收字节（3） | (3, 0, 0, 3)、(3, 0, 1, 3) | 0 |
| 7 | 待删除但意图未完成占用（4） | (4, 0, 0xFFFF_FFFF, 3) | 0 |
| 8–9 | defer 队列待释放（5） | (5, 0, 0, 3)、(5, 0, 1, 3) | 16 384（最初のファイルバージョンがツリーテーブルを書き換える際、mkfs の第0版ツリーテーブルユニットを交替、D3 (領域割り当て) 確定項目 7） |
| 10 | 已承诺预留量（6） | (6, 0, 0xFFFF_FFFF, 3) | 196 760（D3 (領域割り当て) 確定項目 2 の計算式、ツリーテーブルに `deleted_inodes` エントリが存在、2 × ((2 + 0 + 1 + 2 + 0 + 1) × 16384 + 41 × 1 + 35 × 1)） |
| 11–12 | 碎片度 runs（10） | (10, 0, 0, 3)、(10, 0, 1, 3) | 各ディスク 4 |
| 13–14 | 全空聚簇段数（11） | (11, 0, 0, 3)、(11, 0, 1, 3) | 各ディスク 3310（4 GiB イメージの予測値） |
| 15 | inode 号水位（12） | (12, 65537, 0xFFFF_FFFF, 3) | 2 |

**適用範囲**: 新規プールでのファイル作成のバイト数を変更する: はい——会計ツリーユニットのエントリ数（15）、宣言長（15 × 34 = 510）および 15 件のレコード内容。受入不等式が必要とする4項目の統計量（回収不能、削除待ち占有、コミット済み予約、defer 解放待ち）は day-1 に各1行書き込まれ、これは「空は 0 と等しくない、空 = 未維持」という防衛線を維持するためである：もしこれら4項目を day-1 に1行も書かない場合、初回マウント時に読み取られるのは空となり、受入不等式（D3 (領域割り当て) 確定項目 4 / D28 (マウント期コミットメント量) 確定項目 2）がこれらを読み取る際、「空 = 未維持」で判定すれば書き込み可能マウントを拒絶すべきとなり、「空を 0 として読む」と判定すれば防衛線を撤廃することになる；各1行書くことが、防衛線を維持しつつ初回マウントを通過させる唯一の手法であり、コストは 6行 × 34バイト = 204バイト である。空きバイト数と完全空クラスタセグメント数の2行の値はイメージサイズに応じて変動し、4 GiB はマイルストーンステップ 0 の予測値である。清掃高水位と直近のルート破棄世代番号の2項目はこの時点で対象が存在しないため、1行も書かれない。ユーザー決定は三方論証を経ておらず、覆され得る。

**根拠**:

- E142 (新規プールでのファイル作成のドライラン): 15行のキーと値がフィールドごとにバイト列として書き出され、第22回実行において `crates/` と領域ごとに突合して完全一致（29領域 `equal=29 unequal=0`、第10行は両側とも 98 386）；会計ツリーユニットの宣言長とエントリ幅はそのページにある。
- ユーザー決定およびコーディング前の総監査、原文は変更履歴にある。

**未済**: なし。

#### 確定項目 9: 書き込み可能ヘッドをどのIDで識別し、`previous_snapshot_txg` は誰に帰属するか

**決定事項**: 書き込み可能ヘッドの ID = そのヘッドの inode ツリーのツリーID。会計のツリー次元、livelist 共有ツリーのキープレフィックス、ツリーテーブルエントリの `previous_snapshot_txg` の帰属の3箇所すべてでこれを用いる。`previous_snapshot_txg` は2箇所に記録される: そのヘッドの inode ツリーのツリーテーブルエントリ内、および各コード14スナップショットエントリ内（= キャプチャ瞬間の生存ヘッドの値）；その他のエントリには 0 を書き、0 = 該当なしとする；inode ツリーエントリに書かれた非0の値は、このツリーテーブルを参照するルートの txg を超えない（I-9.25 (ツリーテーブルエントリの previous_snapshot_txg の値)）；スナップショットをまだ取得していない新規ヘッドも 0 を書き込む（スナップショットの txg ≥ 1、mkfs の txg 0 にはユーザー可視状態が存在せず衝突しない）。非 inode ツリーはツリーテーブルエントリ内の「ヘッドID」によって自身がどのヘッドに帰属するかを特定する: ツリーテーブルエントリは予約領域から 8 バイトを切り出してこれを格納し（予約領域は 76 バイト、D8 (コアインデックス構造) 確定項目 8）、値の規則はヘッドごとの構造にはそれがサービスを提供するヘッドのツリーIDを書き、プール全体で1つの構造には 0 を書く——inode ツリーエントリは自身のツリーIDを書き（新規プールでのファイル作成時は 65537）、extent ツリーエントリはそれがサービスを提供するヘッド（65537）を書き、`deleted_inodes`、墓石ツリーおよび退役ツリーは inode ツリーのツリーID（65537）を書き、系譜ツリーは 0 を書き（プール全体で1つ）、logged_ops は 0 を書き（整理意図エントリのフィールド表は確定済み、D26 (バックグラウンド整理と配置回収) 確定項目 2 を参照；C364 (整理インテントレコードのフィールドが逐一突き合わせされていない) はフィールド表と実装の逐一突き合わせが未済、I-4.4 (インテントは目標状態のみを記述する) のフィールド白リスト判定は実装済み）、割り当てレコードツリー / 会計ツリー / livelist 共有ツリー / スパースサイドテーブルツリー / deadlist ツリーは 0 を書き（帰属なし）、中央マッピングツリーはツリーテーブルに入らない。

**適用範囲**:「誰がヘッドを識別するか」および「前駆スナップショット番号をどのエントリに記録するか」のみを決定し、ツリーテーブルエントリ自体のフィールドと幅（D8 (コアインデックス構造) 確定項目 8）も、破棄カスケード内で `previous_snapshot_txg` がどのように伝播するか（確定項目 13）も決定しない。読み取り専用スナップショットのコード14エントリのヘッドID = キャプチャされた生存ヘッドの inode ツリーのツリーID、コード15エントリのヘッドID = ペアリングされたコード14エントリのツリーID、前駆（`previous_snapshot_txg`）には 0 を書き込む。

**根拠**:

- E142 (新規プールでのファイル作成のドライラン): ツリーテーブル第1版の11エントリ（退役ツリー追加前のバージョン）のヘッドIDと `previous_snapshot_txg` がバイトごとに書き出され、当時の `crates/` 実装と突合して完全一致（2026-10-05 第24回実行）；退役ツリー追加後の実装は12エントリとなり、テスト装置は第25回実行（2026-10-13）で追従、ツリーテーブル12エントリ、高水位 65549、主要29領域がバイト単位ですべて一致。
- ユーザー決定およびコーディング前の総監査、原文は変更履歴にある。
- 判定 `research/prompts/m3-blanks-r1-main-verification.md` 第2節ヘッドID行: 旧定義文は自身が列挙した5つの 0 書き込み例と矛盾し新しいツリーを判定できなかったため、ヘッドごと構造はそのヘッドを書き、プール全体構造は 0 を書くよう修正。
- ユーザー決定 2026-10-01: 系譜ツリーのヘッドIDは 0 を書き、プール全体で1つ（分岐シート `m3-blanks-r1-forks.md` の「系譜ツリーのヘッドID」行、原文は変更履歴にある）。
- 判定 `research/prompts/m4-snapshot-r1-main-verification.md` 第3節 S1（コード14エントリのヘッドID = 生存ヘッドの inode ツリーのツリーID、`previous_snapshot_txg` = キャプチャ瞬間の生存ヘッドの値；コード15エントリのヘッドID = ペアリングされたコード14のツリーID、前駆 0）および S5（I-3.7 (生存スナップショットが列挙可能) の前駆チェーンにはコード14が前駆を保持することが必要）。

**未済**: なし。

#### 確定項目 10: 統計量タグ登録表と二つの予約値

**決定事項**: 統計量タグの登録表は確定項目 4 の表そのものである——コード = 行番号（1 から 14；第8 / 9項は退役済みで再利用しない；0 は無効値として予約）、各行のマージ規則は確定項目 3 に従い（単調量は最大値をとる: 第12 / 13 / 14項；その他は後勝ち）、デバイス次元およびツリー次元の有無は表に従う。デバイス次元を持たない統計量は、キーのデバイスセグメントに予約値 0xFFFF_FFFF をとる；ツリー次元を持たないツリーIDセグメントには 0 をとる。

**適用範囲**: 登録表が管轄するのは「タグコード → どの統計量か、どうマージするか、どの次元を用いるか」であり、古いリーダーが未知のタグを読み取った際にどうすべきかは管轄しない——その枠は compat レベルの問題であり、D15 (フォーマット凍結ポリシー) の判定表にはこれを処理する行が存在しない。

**根拠**:

- E142 (新規プールでのファイル作成のドライラン): 新規プールでのファイル作成の15行のキー内の二つの予約値（デバイスセグメント 0xFFFF_FFFF、ツリーIDセグメント 0）がバイトごとに書き出され、`crates/` 実装と突合して完全一致。
- ユーザー決定およびコーディング前の総監査、原文は変更履歴にある。

**未済**: C71 (未知の統計量タグが暗黙に無視される)；C117 (デバイス次元を持たない統計量のデバイスセグメントに値が設定されていない)——値は確定済み、赤となる検査は依然として未済。

#### 確定項目 11: deadlist のキャリアを day-1 に作成するか

**決定事項**: 作成する。deadlist は独立キースペースのツリーを形成し、**day-1 にツリーテーブルへ登録される**——ツリーの種類コード **8**（D8 (コアインデックス構造) 確定項目 9 の登録表）、ツリーID **65543**、ルートポインタゼロ（まだルートなし）、誕生 txg 3、ヘッドID 0（帰属なし）。ツリーテーブル第1版は D18 (ブロックに含める情報) 確定項目 24 の5本のゼロルートツリーと合わせて合計 **12エントリ**、ツリーID高水位はパブリッシュ後 **65549**。新規プールでのファイル作成時は deadlist エントリを1件も書き込まない（解放なし）。

**適用範囲**: 新規プールでのファイル作成のバイト数を変更する——ツリーテーブル第1版に deadlist が1件追加（D18 (ブロックに含める情報) 確定項目 24 の5本のゼロルートツリーと合わせて合計12エントリ、宣言長 12 × 200 = 2400；退役ツリー追加前は11エントリ、2200）、ツリーID高水位はパブリッシュ後 65549（退役ツリー追加前は 65548；ルートレコードと journal 新ルートセグメントに各1コピー）。本項目はキャリアを作成するか否か、いつ作成するかのみを決定し、エントリの形態は確定項目 12 にある。ユーザー決定は三方論証を経ておらず、覆され得る。

**根拠**:

- E146 (マッピングキーで識別される livelist エントリの幅とコスト): livelist 共有ツリーは同型の先例であり、day-1 登録と遅延作成がそれぞれ支払うべきコストはそのページにある；遅延作成は「最初のスナップショット作成時にツリーを作成する」という追加パスを生じさせ、そのパスは単一パブリッシュ内でツリー作成、ツリーテーブルエントリ書き込み、ツリーID高水位引き上げを行わなければならない。
- E142 (新規プールでのファイル作成のドライラン): ツリーテーブル第1版の11エントリ（`deleted_inodes` 等の4本のゼロルートツリーを含む）とツリーID高水位 65548 がフィールドごとにバイト列として書き出され、当時の `crates/` 実装と突合して完全一致（2026-10-05 第24回実行）；その後退役ツリーが追加され実装は12エントリ、高水位 65549 となった；テスト装置は第25回実行（2026-10-13）で追従、ツリーテーブル12エントリ、高水位 65549、主要29領域がバイト単位ですべて一致。
- ユーザー決定、原文は変更履歴にある。

**未済**: なし。

#### 確定項目 12: deadlist エントリの形態

**決定事項**:「丙-区間（上限付き）」を採用する。

| 項目 | 決定された内容 |
|---|---|
| 条目 | キー = ヘッドのツリーID 8 + エポック 8 + 中央マッピングキー 27 = 43 バイト、値は空（キーそのものが事実）。識別子は中央マッピングキー；エポック = ブロックが破棄された瞬間のヘッドの `previous_snapshot_txg` であり、バケット分割はこのセグメントそのものであり、別途バケット番号フィールドは設けない |
| 哪一份 | スナップショット X の分 = エポックが [prev(X).txg, X.txg) に入るエントリ；ヘッドの分 = [prev(ヘッド).txg, ∞)。スナップショット作成時は prev(ヘッド) を前進させるのみで、既存エントリは1件も移動しない——確定項目 13 の「新スナップショットへ全体を引き渡す」とはこのことを指しており、エントリの移動は行わない |
| 销毁 S | 意図 (ヘッド, lo = S.txg, hi, P = prev(S).txg) を書き込む: hi は次の側がスナップショットの場合はその txg、ヘッドの場合は意図を書き込む開放 checkpoint の番号をとる（パブリッシュ済みの最大番号をとると区間が空になりエントリが永遠に解放されない）；prev(S) は意図を書き込むトランザクション内でその瞬間の次の側へ渡され、その後のパブリッシュへ分割することは許されない；その後バッチごとに [lo, hi) 内の birth > P であるエントリを解放し、いかなるエントリのキーも変更しない；バッチ内の各操作は適用前に削除対象エントリの存在を確認する（D8 (コアインデックス構造) 確定項目 5 硬性要求第4条）；カスケード間に順序は不要であり、ヘッド破棄意図は同一ヘッド上の先行カスケード完了を待ってからエントリを操作する |
| 下一侧怎么找 | ツリーテーブルエントリには前後スナップショットのリンクがないため、意図書き込みの瞬間にマウント時ツリーテーブル走査で構築した「ヘッド → 生存スナップショットチェーン」派生態から取得: ゼロバイト、喪失しても速度低下のみ（D21 (権威態と派生態の分界) の派生態判定基準） |
| 按头 / 按快照删一段 | プレフィックス (ヘッド) / (ヘッド, エポックが [prev(X).txg, X.txg) に入る) 区間 |
| 树表字节 | 0；予約 76 バイトは他の予約者のために残す |
| 改新池新建文件的字节 | いいえ——新規プールでのファイル作成では何も削除せず、エントリは1件も書き込まれない |

**適用範囲**: 破棄されたスナップショットは意図を書き込んだ瞬間からブロックを固定しなくなり、これは確定項目 20「削除待ち構造は生存参照とみなす」の例外である；破棄された書き込み可能ヘッドがどの瞬間から算入されなくなるか、分岐点ゲートの `children` がいつ減算されるかは条項が存在しない。エントリ形態は確定したが、赤となる検査はまだない。

第1版の破棄意図は破棄されるスナップショット自身のコード14ツリーテーブルエントリに書き込まれる: flags のビット 0（破棄中）のみを占有し、予約領域には1バイトも書き込まない；lo = エントリの誕生 txg、P = エントリの前駆、hi はバッチごとに派生チェーンからその場で取得（次のコード14の誕生 txg、次の側がヘッドの場合は意図を書き込んだパブリッシュの txg）、再開ポイントは保持しない（エントリが解放されること自体が進捗）；logged_ops には常駐しない。破棄は直列化される: 1回の呼び出し = まずドレイン + 意図 + 全バッチ、削除が完了してから他の変更を受け付ける（ユーザー決定）。スナップショット削除は unlink のプロセスに従い同等の受入を行い、「まずドレイン」（D8 (コアインデックス構造) 確定項目 16 第6条）に対する例外は設けない（ユーザー決定）。次の側はツリーテーブルから (誕生 txg, ツリーID) 順に並んだ派生チェーンから取得する。

**根拠**:

- 3ラウンドの三方判定: `research/prompts/d5-item12-r1-main-verification.md`、`d5-item12-r2-main-verification.md`、`d5-item12-r3-main-verification.md` それぞれの第4節。丙（ポイント読み出し法）は2ラウンドで脱落、甲は U2 で脱落；丙-区間の第2ラウンド最弱解釈（意図区間 [S.txg, ∞)）は 6 ステップで過剰解放となり、D8 (コアインデックス構造) 確定項目 5「意図は目標状態を宣言し、決して増分を宣言しない」に抵触、上限付きへ厳格化；第3ラウンドは「バッチ開始時の状態に基づいて計画し適用時に再確認しない」場合においてのみ 7 ステップで重複解放となり、同要求の冪等性に抵触、適用時再確認へ厳格化。乙および丁は第2ラウンドで任意順序下において解放漏れが発生、順序付け後も第3ラウンドで5つの面すべて不成立、代償はカスケードキーの書き換えであり、5つの破棄幾何形状において丙-区間は 0 件、乙は D(S)、丁は K（`out-geometry.jsonl`）。
- E94 (共有配置先移動の接触面): 配置先による記録では再配置ごとに deadlist を逐条再書き込みする必要があるが、バージョン識別子による記録では中央マッピングは常に単一ルート、deadlist の再書き込みは常にゼロである。
- 判定 `research/prompts/m4-snapshot-r1-main-verification.md` 第3節 S4（意図キャリアはコード14エントリ自身を採用し、flags ビット 0 のみを占有、予約は不書き込み、lo と P はエントリ上に保持、hi はバッチごとに派生チェーンからその場で取得、再開点は非保持、logged_ops には非常駐）および第7節ユーザー決定（分岐2 破棄直列化、分岐4 スナップショット削除は unlink のプロセスに準拠）、原文は変更履歴にある。
- ユーザー決定、原文は変更履歴にある。

**未済**: C145 (deadlist がディスク上に配置先を持たない)——エントリ形態は確定済み、checker 側の半分が依然として未済；C336 (意図書き込みの瞬間にスナップショットの次の側が見つからない)。

#### 確定項目 13: ライフサイクル判定にどのメカニズムを採用するか

**決定事項**: 誕生世代番号 + deadlist モデル（ZFS の birth txg + deadlist）を採用する。

- 各ブロックは**誕生世代番号**（birth txg）を自ら保持する。
- 削除時、ブロックの世代番号と**生存ヘッドレコードの `previous_snapshot_txg`**（直前のスナップショットの世代番号）を比較する: `birth > previous_snapshot_txg` であれば**即座に解放**、そうでなければ**生存ヘッド自身の deadlist へ追記**する。
- スナップショット作成時、**生存ヘッドの deadlist 全体を新スナップショットへ引き渡し**、生存ヘッドは空のものと交替する（確定項目 12: 引き渡し時にエントリは移動しない——エントリはエポックによりバケット分割されており、スナップショット作成時は prev(ヘッド) を前進させるのみである）。
- スナップショット破棄時、その deadlist を**次のより新しい側へマージ**し、`birth > prev(S).txg` に従い解放可能エントリをフィルタリングし、**全盤スキャンは行わない**（どちら側へマージするかは確定項目 16 にある）。
- **`prev` は伝播させなければならない**: 破棄された側の `prev(S).txg` は引き受ける側へ継承され、意図を書き込むトランザクション内でその瞬間の次の側へ渡され、その後のパブリッシュへ分割することは許されない（分割した場合、3つの候補すべてで 5 ステップ過剰解放が発生した）。伝播させないと、代々のスナップショットが自身の元の `prev` でフィルタリングを行うことになり、`birth` の小さなエントリは永遠に `birth > prev` が成立せず、無制限なリークとなる。

**適用範囲**: 参照判定基準と3つの量の定義は確定項目 14、実行時形態とコストの計測条件は確定項目 19、適用前提は確定項目 17 にある。D9 (暗号化) との連動: birth txg は暗号化 nonce として**使用できない**——同一トランザクション内のブロックがこれを共有するため、nonce が重複することになる。D5 (スナップショット / 空間会計メカニズム) 自体の実測ではクラッシュ再生の1セルのみが未実行のままである: deadlist の追記とカスケードマージは**非冪等**であり（多重集合の加算、同一エントリを2回追記すると破棄時に二重解放となる）、二重解放はデータ損失方向であり、証明済みの「データ損失方向は構造的にゼロ」は定常状態における `death` 規則評価のみをカバーしている；クラッシュポイント再生の2ストリームは deadlist エントリを1件も書き込まず、このセルは依然としてカバレッジゼロであり、実行されるまで D5 (スナップショット / 空间会計メカニズム) のリスクパスが検証済みであると主張することは許されない（E13 (D5 の四つのリスクパス) パス 3）。

**根拠**:

- E12 (バッチ順次追記 vs 非バッチランダムページ RMW): 20万回の削除において、誕生世代番号側のデバイス I/O は3段階のキャッシュにおいて常に 2 回、全域参照カウントはカウンタ表がキャッシュに収まらない際に削除あたり約 1 回へと跳ね上がり、転換点は急峻である；ブロック層独立検証（VM 内 `/sys/block/vda/stat`）の9セルにおける比率はすべて 1.00 であり、`O_DIRECT` を外すとブロック層の読み取り値はゼロとなる一方プログラムカウンタは不変であり、この検証パスは判別力を持つ。
- E94 (共有配置先移動の接触面): `prev` が伝播しない場合エントリは恒久的に固定され、ミューテーション M5b がモデル層で赤となることが検証された。
- 他社事例（手がかり、本プロジェクトでは未検証）: btrfs qgroup は extent refcount + 逆インデックス走査を採用し、コストはコミット時間 +76%、トランザクション待ち +1347%、書き込みスループット −25%（計測条件は「qgroup が逆インデックスを走査」、純粋な参照カウントではない）；bcachefs の会計はトリガーによりキーコミット時に維持され、`check_allocations` はフル実行走査により再構築され、裏付けているのは「増分維持 + 走査検証」というパターンであって本メカニズムではない（bcachefs は deadlist を持たない）。かつその走査検証は実行時と同一の加減算コードを共用しており、「セマンティクス自体の誤記述」を検出できない（[prior-art.md](../prior-art.md) 7.11）。本エンジニアリングにおいてその形態をそのまま模倣することは許されない。

**未済**: C195 (破棄カスケード時の prev 伝播が実験のモデル改定記録にのみ記述されている)。

#### 確定項目 14: 参照区間の必要十分条件

**決定事項**: **ブロック b がスナップショット S から参照されている ⟺ `birth(b) ≤ S.txg < death(b)`**、左閉右開。

| 量 | 定義 | 何で**ない**か |
|---|---|---|
| `birth(b)` | ブロックが**パブリッシュ**された checkpoint 番号 | 書き込み要求発行時の開放 txg ではない——クラッシュ後に同番号は再発番される |
| `death(b)` | 最後の生存参照が外され、**かつその切り離しがパブリッシュ**された checkpoint 番号；生存ツリー内に依然としてある場合は ∞ | `rm` が復帰した瞬間ではない（D8 (コアインデックス構造) は意図バッチに従う）、領域が再割り当て可能となる瞬間でもない（D16 (パブリッシュセマンティクス) 確定項目 15） |
| `S.txg` | **S がキャプチャした最後のパブリッシュ済み txg**；スナップショット作成が独立パブリッシュ（データ変更を伴わないパブリッシュ）の場合、S.txg はそのパブリッシュの `checkpoint_txg` であり、同一の値が生存ヘッドの inode ツリーエントリの `previous_snapshot_txg` およびスナップショットエントリの誕生 txg に書き込まれる | 「作成操作が存在した txg」ではない |

3つの境界と誤設定時の結果：

| 境界 | 判定 | 誤設定時の結果 |
|---|---|---|
| `birth == S.txg` | S はそれを**参照**する | `birth < S.txg` と記述すると「スナップショットより新しい」と判定され即時解放、**データ損失** |
| `death == S.txg` | S は**参照しない** | `S.txg ≤ death` と記述するとそれを不要とするスナップショットにぶら下がり、**領域リーク + 帰属エラー** |
| `birth == death` | いかなるスナップショットも参照せず、即時解放可能 | 会計上は正しいが、**物理領域はこの checkpoint 内で再利用してはならない**（D16 (パブリッシュセマンティクス) 確定項目 15） |

**保守的既定規律: 判定のいかなる入力も取得できないか不確定な場合、既定は必ず「deadlist 入り」としなければならない。**

**適用範囲**: この条件は txg が全順序（単方向線形履歴）である場合にのみ到達可能性と同値であり、前提を超過した後の対応は確定項目 17 にある；この `death` 定義によれば、分岐下における誤死亡判定は恒常的にゼロであり、これは証明可能である——truth が真であることはブロックがそのルートのチェーン上で死んでいないことを示し、「全分岐で削除済み」を満たさず、`death = ∞` となり、区間判定は必然的に真となる。保守的既定の理由は二つの方向の非対称性にある: リークは I-3.6 (deadlist の厳密境界) により検出可能であり（I-3.1 (割り当て統計が一致する) では**ない**——[invariants/05-空间记账.md](../../../crates/singlefs-checker/invariants/05-空间记账.md) は「帰属エラーはこれに見えない……総勘定は一致したまま」と明記している）、回収可能で被害は有界である；損失は不可逆であり、領域がいったん再利用されると「チェックサム不一致、原因不明」へと退化し、原因帰属すら失われる。`S.txg` が第2の定義をとることはメカニズム的である: `birth` の解像度は checkpoint までであり、「作成操作が存在した txg」をとると、条件が「スナップショット時点を checkpoint の先頭に置くか末尾に置くか」に応じて `birth ≤ S.txg` と `birth < S.txg` の間を揺れ動くことになり、定義は不変条件が実装位置によって揺らがないものを選択しなければならない。

**根拠**:

- E27 (D5 のモデル化可能な三つのリスクパス): 3つの境界を誤った際の結果は確かに非対称であり、独立した3箇所の誤記ではない——右境界を `S.txg ≤ death` と記述すると `death == S.txg` と `birth == death` の2行に同時にヒットし、1箇所の誤記が2箇所のリークを引き起こす；二つの誤りの失敗方向は正反対である（左境界を厳格にするとデータ損失1箇所のみ、右境界を閉じるとリーク2箇所のみ、クロスオーバーゼロ）。
- E18 (D5 は多分岐 / 複数書き込み可能ヘッド下で破綻するか): この `death` 定義に従えば、線形、単一分岐、8書き込み可能ヘッドの3段階において誤死亡判定はすべて 0 であった。
- 判定 `research/prompts/m4-snapshot-r1-main-verification.md` 第3節 S2: スナップショット作成は独立パブリッシュを採用し、S.txg はそのパブリッシュの `checkpoint_txg` を取得；攻撃側モデルにおいて S.txg と prev(ヘッド) を同源とする解釈は違反 0 件。

**未済**: なし。

#### 確定項目 15: バックグラウンド再配置で `birth` を変更するか

**決定事項**: 変更しない。バックグラウンド整理があるユニットをある配置先から別の配置先へ移動することは、**「新バージョンをパブリッシュすること」ではない**——論理内容は1バイトも変化しておらず、物理位置が移動しただけである；確定項目 14 が `birth(b)` に与えた定義文「ブロックが**パブリッシュ**された checkpoint 番号」に従い、再配置は新たなパブリッシュイベントを発生させず、`birth` はユニットと共にそのまま保持される。

**適用範囲**: `birth` が移動に伴うか否かのみを管轄し、再配置自体がどの構造に触れるかは管轄しない（D26 (バックグラウンド整理と配置回収) 確定項目 5）。deadlist エントリが配置先ではなくバージョン識別子で記録され位置ヒントを持たないことも、同一の理由の下流にある: 配置先による記録は再配置ごとに deadlist を逐条再書き込みさせ、ヒントを保持しても何も得られず（解放は一律マッピングを経由、D19 (ブロックポインタの構造と幅の予算) 確定項目 5 硬性規則 1）、むしろ二重解放を引き起こす余地を実装に残してしまう；エントリのフィールド表着地は確定項目 12 にある。削除側はポインタから誕生ツリーを取得し、ユニットヘッダを余分に読み取る必要はない（D19 (ブロックポインタの構造と幅の予算) 確定項目 7）。

**根拠**:

- E94 (共有配置先移動の接触面): もし再配置で `birth` を再割り当てした場合、再割り当ての違反数はスナップショットから参照されている移動数に正確に一致する——スナップショットから参照されているユニットを1つ移動するごとに1回誤る；`birth` を変更しなければ違反は恒常的に 0 である。

**未済**: なし。

#### 確定項目 16: スナップショット破棄時に deadlist をどちら側へマージするか

**決定事項**: 規約 B を採用する——スナップショット破棄時はその deadlist を**次のより新しい側**へマージする。

**適用範囲**: 規約 B は「スナップショットの deadlist は決して書き換えない」というわけでは**ない**——破棄カスケードの対象 `dsl_deadlist_merge(&ds_next->ds_deadlist, ...)` において `ds_next` 自体がスナップショットであり、規約 B を採用する理由は例外が1桁少ないためであり、例外がないわけではない。二つの規約の差異は**パブリッシュ済みスナップショットの deadlist を書き換える頻度**であり、書き換えるか否かではない——別の規約では毎回の kill でパブリッシュ済みスナップショットの deadlist へ追記する必要があるが、規約 B では破棄カスケード時に1回書き換えるのみである。I-3.6 (deadlist の厳密境界) の厳密境界 `birth ≤ prev(S).txg` は規約 B に従って書かれている；カスケード内で `prev` をどう伝播させるかは確定項目 13、分岐点は一意な次の側を持たないため破棄禁止であることは確定項目 18 にある。

**根拠**:

- 実験なし: 二つの規約の差は「パブリッシュ済みスナップショットの deadlist を書き換える頻度」であり、本エンジニアリングではこの頻度を計測しておらず、第2の定量可能な量も存在しない；手がかりは現役実装にある——ZFS が実装しているのは規約 B である（`module/zfs/dsl_destroy.c` 現地確認: `/* Merge our deadlist into next's and free it. */`）、本プロジェクトでは未検証。

**未済**: なし。

#### 確定項目 17: 4つの前提、いずれかを超過すると無効化する

**決定事項**: 4つの前提——① 線形履歴（分岐なし）；② 単一の書き込み可能ヘッド；③ 各ブロックは生存ツリー内で最大1つの生存参照を持つ；④ スナップショット時点が checkpoint 境界と整列している（D16 (パブリッシュセマンティクス) 確定項目 14）。**「同一ブロックへの第2の生存参照」を導入するいかなる機能も、事前に D5 (スナップショット / 空間会計メカニズム) を再開しなければならない。** この再開条項はすでに1度トリガーされ、収束している: D6 (スナップショット実現モデル) 確定項目 1 が「ヘッドごとに独自のツリー」を採用し、ヘッド間共有ブロックこそが第2の生存参照となった；⇒ D5 (スナップショット / 空間会計メカニズム) 自らが提示した正しい形態に従い収束: **D5 (スナップショット / 空間会計メカニズム) 主 + スパースサイドテーブル従（ZFS の BRT 形態）**——D5 (スナップショット / 空間会計メカニズム) を全域参照カウントに置き換えるのではなく、サイドテーブルの配置先とフィールド表は確定項目 6、3つの連帯要求は確定項目 18 にある。

**適用範囲**:

- 前提 3 は reflink / dedup / ブロッククローンとメカニズム的に非互換である: 第2の生存参照がある場合、そのうちの1つを外しても death は構成されないが、判定基準は `birth` と `previous_snapshot_txg` のみを参照し二つの参照の birth は完全に同一であるため、最初の除去で「即時解放」と判定され、もう一方の参照が宙吊りとなる。ZFS は20年間 reflink を持たず、2023年の block cloning での修復方法は**参照カウント付きの別表を追加すること**であり、birth/death の拡張ではなかった（手がかり、本プロジェクトでは未検証）。
- 前提 1、2 を超過した後の失効形態は**無制限なリーク + 会計の参照カウントへの退化**であり、データ損失ではない:「最後の生存参照であるか否か」の維持に必要なのはまさに各ブロックの分岐網羅情報、すなわち参照カウントであり、それこそが D5 (スナップショット / 空間会計メカニズム) が省こうとしたものである。**D5 (スナップショット / 空間会計メカニズム) の O(1) 性質はこの状態に置かれており**、それを補完するのがスパースサイドテーブルである。
- 「D5 (スナップショット / 空間会計メカニズム) + スパースサイドテーブル」アームがコストを約100分の1に圧縮したことは、**検証待ちのオーダー指標としてのみ扱うべきであり、検証済みの利益として判定に記述することは禁止される**: その唯一の出所であるアームには失敗条項も陽性対照も存在せず、アーム A / アーム B のブロック層独立検証や故障注入自己検査はこれに対して行われていない；これを用いるにはまず失敗条項と陽性対照を補完して再実行する必要がある。
- サイドテーブル規模の外挿境界: 探索ではメモリ全域ビットマップ、ブロックあたり 1 ビット（実ファイルシステム上で約 128 MB/TB）を用いており、大ディスク上でのサイドテーブルの挙動を過大評価しているため、真正なスパース構造に置き換えて再測定しなければならない。局所性もすでに測定されている: 削除シーケンスを一様ランダムから順次に変更すると、参照カウント 8× クラスは 174 998 回の I/O から 391 回へと激減（448倍）；実負荷が局所性を持つ場合、D5 (スナップショット / 空間会計メカニズム) の優位性は主表の桁数より著しく小さくなり、**主結論と共に引用することが必須である**。checker コストとクラッシュ整合性はそのアームの管轄外である。

**根拠**:

- E18 (D5 は多分岐 / 複数書き込み可能ヘッド下で破綻するか): 単一分岐時の生存判定誤り（生存と判定したが実際は死亡）は 8.19%、8書き込み可能ヘッド時は 29.40% であり、8書き込み可能ヘッドクラスの正しい規則は「death を一切記録しない」とビット単位で一致——「1ブロックも回収しない」ことと引き換えにデータ損失ゼロを達成している。
- E12 (バッチ順次追記 vs 非バッチランダムページ RMW): 20万回削除のデバイス I/O 中央値において、D5 (スナップショット / 空間会計メカニズム) の3クラスは常に 2 回、全域参照カウントは 512 / 約 200 208 / 約 350 042 回、D5 (スナップショット / 空間会計メカニズム) + スパースサイドテーブルは 508 / 約 1 956 / 約 3 466 回；カウンタ表がキャッシュに収まる場合参照カウントはほぼ無料であり、これは正当な結果であってバグではない——D5 (スナップショット / 空間会計メカニズム) の優位条件は「カウンタ表がキャッシュに収まらないこと」であり、無条件に成立するわけではない。
- E52 (二つの複数書き込み可能ヘッドメカニズムの運用コスト): 収束形態におけるサイドテーブルのエントリ数は2セルでそれぞれ 1 000 および 50 000 行である。

**未済**: なし。

#### 確定項目 18: クローン / 書き込み可能スナップショットに追加すべき3条

**決定事項**: 前提 1、2 が超過された後に3条を追加する。D6 (スナップショット実現モデル) 確定項目 1 がヘッドごとに1ツリーを採用した後、これら3条は必須事項であり参考事項ではない。

1. 各書き込み可能ヘッドは独自に `previous_snapshot_txg` と deadlist を1部ずつ保持し、**新クローンの `previous_snapshot_txg` は必ず origin スナップショットの txg で初期化されなければならない**——0 に設定すると、クローン内で継承した古いブロックを削除した際に「即時解放」と算出され、**データ損失**となる。
2. **分岐点スナップショットは破棄を禁止する**（規約 B は分岐点において一意な次の側を持たない）。判定基準は O(1) 子カウント `children > 1` → 拒絶であり、`children` は**このスナップショットからクローンされた書き込み可能ヘッドも算入する**。**もう一つのゲート: 自身の生存スナップショットを持つ書き込み可能ヘッドは破棄を禁止する**——書き込み可能ヘッドを破棄するには、まずそれが作成した全スナップショットを破棄しなければならない；クローン時に依拠した origin スナップショットは自身のものとはみなさない。
3. **書き込み可能ヘッドの破棄には第2の構造が必要である**（livelist 形態: ALLOC/FREE ペア、birth txg 別グループ化、増分可能、凝縮可能）。これを認めない場合、このパスにおける「事後トラバースを行わない」は偽りとなる——解放すべきは `birth > origin.txg` であるブロック群であり、全ツリーに散乱している。

**適用範囲**:

- 第2のゲートが獲得するのは、第3条の「解放すべきは `birth > origin.txg` であるブロック群である」が文字通り成立することである: 破棄の瞬間にヘッド自身のスナップショットはすべて破棄されており、`previous_snapshot_txg` は確定項目 13 の `prev` 伝播に沿ってクローン時点まで後退している。このゲートがない場合、ヘッド破棄時の `previous_snapshot_txg` はそのクローン時点より大きくなり得、`birth ≤ previous_snapshot_txg` であるブロック群は依然として自身のスナップショットから参照されているため解放できず、これらブロック群の行き先はリポジトリ全体で定義されていない——「直近のスナップショットの deadlist にマージ」とすると、その後のそのスナップショット破棄時にクローンが依然として参照しているブロックを解放してしまい、「livelist に伴い消費」とすると、`prev` がより古い時点へ継承された後に永久に解放されなくなる（C269 (書き込み可能ヘッド破棄時に残されたブロックに行き先がない)、返済済み）。
- **代償、了解の上で受容**: 本リポジトリには promote が存在しないため、あるヘッドのスナップショットがクローンされそのクローンが生存している場合、そのスナップショットは削除できず、したがってこのヘッドも削除できない（分岐点ゲートの `children` はクローンをカウントする）。これは D6 (スナップショット実現モデル) 判定基準 5 の「機能欠落によって回避されたものは満足ではない」と同型であり、本リポジトリで2番目のこのような制約である。
- livelist 自身のキー、幅および共有ツリーは D6 (スナップショット実現モデル) 確定項目 2 / 確定項目 3 にある。
- ZFS の2つの対応するゲートはローカルの `fs-refs/zfs/module/zfs/dsl_destroy.c` にある: 第717–723行の原文「Can't delete a head dataset if there are snapshots of it. (Except if the only snapshots are from the branch we cloned from.)」、および `dsl_destroy_snapshot_check_impl`: `/* Can't delete a branch point. */`——手がかり、本プロジェクトでは未検証。

**根拠**:

- E146 (マッピングキーで識別される livelist エントリの幅とコスト): 第3条が要求する第2の構造において、エントリ識別子に中央マッピングキーを採用した後の幅とコストは計測済みであり、day-1 登録で支払うべきコストもそのページにある。
- 三方論証: 第2の破棄ゲートは D6 (スナップショット実現モデル) 確定項目 2 の3ラウンド対抗の第2ラウンドで導出された；ユーザー決定、原文は変更履歴にある。

**未済**: C265 (書き込み可能ヘッド破棄の解放境界が固定のクローン時点と記述されている)——条項は補充済み、赤となる検査は依然として未済；C270 (分岐点ゲートの子カウントがクローンをカウントするか否か)——同上。

#### 確定項目 19: 生死判定の実行時形態とスナップショット破棄のコスト

**決定事項**:

```
judge(ptr):
    birth = ptr.birth              // ポインタ内包フィールド、追加I/Oゼロ回
    p     = head.previous_snapshot_txg     // 生存ヘッド頭部のスカラー、メモリ常駐
    n     = スパースサイドテーブル[マッピング key(ptr)]  // ヘッド間共有カウント、確定項目 6 ③；エントリなしは 1 扱い
    if n > 1:       カウントを 1 減算、解放もせず deadlist にも入れない（別のヘッドが依然として参照中）
    elif birth > p: 即時解放 → defer キューへ投入（空き領域表への直接投入ではない、D16 (パブリッシュセマンティクス) 確定項目 15 参照）
    else:           head.deadlist へ追加
```

**スナップショット破棄のコストは O(deadlist エントリ数 + バケット数) であり、O(全盤) ではない**。前提として deadlist は **birth txg 別にバケット分割され、バケット境界 = スナップショット txg** でなければならない（確定項目 12 のエポックセグメントがこのバケット分割である）。そうでなければ破棄時のフィルタリングは全表スキャンへと退化する。

**適用範囲**: 帰約証明——削除が checkpoint `d` で発生し、P をその瞬間の最新スナップショットとする。(⇐) もし `birth ≤ P.txg` ならば: P はパブリッシュ済みであるため `P.txg < d = death`、よって `birth ≤ P.txg < death` であり、それを参照する S が存在する。(⇒) もし `birth ≤ S.txg < d` を満たす S が存在するならば、S は削除時にすでに存在していたため `S.txg ≤ P.txg`、よって `birth ≤ P.txg` である。∎ これは3つの事項を用いている: txg の全順序、**判定は必ず死亡の瞬間に発生すること**、およびヘッド間共有ブロックはまずスパースサイドテーブルへ問い合わせること；3番目は O(1) を破壊せず、サイドテーブルはキーを中央マッピングキーとする1回のポイント検索である。いずれか一つでも破綻すれば O(1) は失われる: D6 (スナップショット実現モデル) の複数書き込み可能ヘッドは1番目を破綻させ（確定項目 17 で対応）、D8 (コアインデックス構造) のバッチ削除は2番目を破綻させる（確定項目 20 で対応）。帰約は**証明であり、計測ではない**。これは「前提 3 を回復した後、判定は依然として O(1) であること」を証明している。

**根拠**:

- E12 (バッチ順次追記 vs 非バッチランダムページ RMW): 2つのスカラーを読み出し追加のデバイス I/O がゼロ回である側は3段階のキャッシュにおいて常に 2 回のデバイス I/O であるのに対し、ブロックごとに1つの参照カウントを持つ側はカウンタ表がキャッシュに収まるか否かで1桁跳ね上がる。これが参照カウントに対する本形態のコスト差である。

**未済**: なし。

#### 确定項目 20: 削除待ち構造を生存とみなすか死亡とみなすか

**決定事項**: 削除待ち構造（意図キー、削除中キュー）は**生存参照とみなす**。したがって `death` = 物理バッチの checkpoint となり、判定は必然的に死亡の瞬間に発生し、**O(1) は維持され、いかなる新しい不変条件も追加する必要がない**。例外: 破棄されたスナップショットは意図を書き込んだ瞬間からブロックを固定しなくなる（確定項目 12）。

**適用範囲**: D8 (コアインデックス構造) は無制限な削除が「意図書き込み → バッチ分割 → 再開可能」に従うことを確定しており、したがって判定は物理バッチの瞬間に発生するが、論理的死亡はずっと前に起きており、確定項目 19 の帰約の第2の前提が破綻するため、本項目はその対応策である。代償としてユーザー可視の現象が生じる: `rm` 復帰後に作成されたスナップショットは未解放の extent 群を**固定**し、`df` はスナップショットが死亡して初めて回復する。これを部分的にカバーするのが I-3.4 (利用可能領域からの削除待ち占有控除) および I-4.7 (削除中 inode に台帳が存在する) の二つの不変条件である（checker は両方とも実装済み、`crates/singlefs-checker/invariants/05-空间记账.md` の I-3.4 (利用可能領域からの削除待ち占有控除) 行および `06-事务与崩溃一致性.md` の I-4.7 (削除中 inode に台帳が存在する) 行の状態列参照）。別の道（削除待ち構造を死亡とみなす）は O(1) を破壊する: 正しい deadlist は「論理的死亡後に作成された最初のスナップショット」に帰属するが、そのスナップショットは意図書き込み時点では存在せず、バッチ実行時にスナップショットリストを検索せざるを得ない。破棄された書き込み可能ヘッドがどの時点から生存参照とみなされなくなるか、分岐点ゲートの `children` が意図書き込み時に減算されるか完了時に減算されるかは条項が存在しない。「生存参照」は確定項目 14、17 では生存ツリー参照を指し、読み取り専用スナップショットは生存ツリーではない。

**根拠**:

- 実験なし: 二つの道の差は「判定がどの瞬間に発生するか」という構造的性質であり、計測可能なコストではない；どちらを選択するかは確定項目 19 の帰約証明の第2の前提から直接導出され、計測可能な量はない。
- 判定 `research/prompts/m4-snapshot-r1-main-verification.md` 第3節 S9:「生存参照」は確定項目 14、17 と確定項目 20 で二つの計測条件であり、例外文は「ブロックを固定しなくなる」に修正。

**未済**: C337 (破棄されたヘッドがいつ生存参照とみなされなくなるかの条項がない)。

#### 確定項目 21: スナップショット一覧に列挙可能な永続構造が必要か

**決定事項**: 必要である。ディスク上に現在の全生存スナップショットを正確に（過不足なく）要素とする列挙可能な永続構造が存在しなければならず、不変条件は I-3.7 (生存スナップショットが列挙可能) である。

**適用範囲**:「存在しかつ列挙可能であること」のみを要求し、どのツリーに常駐しエントリがどのような形態であるかは決定しない。走査式 checker（D13 (検証ルート) 確定項目 12 が要求する第2の方向）が帰属を判定するには3つのものが必要である: `birth` がポインタ内に内包されていること（あり）、deadlist 自体が走査可能な自己記述ツリーであること（あり）、**全ルートの txg リスト**（あり：ツリーテーブルの破棄中マークを持たないコード 14 エントリの集合）。I-3.7 (生存スナップショットが列挙可能) の checker は実装済み。

**根拠**:

- 実験なし: これはフォーマット上の存在性要求であり、定量可能な量はない；その判別力は I-3.7 (生存スナップショットが列挙可能) の checker 実装によってのみ証明され得るが、その checker は実装済みである。

**未済**: C144 (スナップショット系譜の区間ラベルにキャリアがない)——「全ルートの txg リスト」の枠の勘定はその名義に計上されている。

````

**出处 `.claude/kb/decisions/05-スナップショット-空間会計メカニズム.ja.md:464-467`（整段抄，未转述）**

```markdown
## 履歴バージョン

D5 (スナップショット / 空間会計メカニズム) の履歴エントリは [decisions-history.md](../decisions-history.md) に集約されている。

```

**出处 `.claude/kb/decisions/05-快照-空间记账机制.md:1-6`（整段抄，未转述）**

```markdown
## D5 快照 / 空间记账机制 —— 已定

**中文** · [English](05-snapshot-space-accounting-mechanism.en.md) · [日本語](05-スナップショット-空間会計メカニズム.ja.md)

D5（快照 / 空间记账机制） 管两件事：块的生命周期判定（诞生代号 + deadlist，已定项 13 到 21）与空间记账（统计量清单、载体、key 宽度、新池新建文件写出哪几行，已定项 1 到 11）；deadlist 条目的形态在已定项 12。记账是在提交路径上增量维护出来的，不是事后遍历出来的，这是 D5（快照 / 空间记账机制） 的全部价值。不管的：「某个落点此刻属不属于现行版本」这个谓词的权威记录点是 D3（空间分配） 已定项 1，D5（快照 / 空间记账机制） 只出其中一半——进了 deadlist 的落点只被快照引用、不在活树里；准入不等式的权威形式在 D28（挂载期承诺量） 已定项 1，D5（快照 / 空间记账机制） 只管式子里每一项由谁维护（已定项 4 的「式子逐项对照」表）；多可写头取哪条机制在 D6（快照实现模型） 已定项 1，livelist 与它的共享树在 D6（快照实现模型） 已定项 2 / 已定项 3；树表条目的字段与宽度在 D8（核心索引结构） 已定项 8；中央映射 key 在 D19（块指针的结构与宽度预算） 已定项 6；记账条目的 `seq` 在 D8（核心索引结构） 已定项 7。D5（快照 / 空间记账机制） 那条 O(1) 判定性质已因 D6（快照实现模型） 取多可写头而失效，把它补回来的是已定项 6 的稀疏旁表；四条前提与失效形态在已定项 17。

```

**出处 `.claude/kb/decisions/05-快照-空间记账机制.md:33-46`（整段抄，未转述）**

```markdown
#### 已定项 1：维度元组里有哪几维

**定案**：除树 ID 之外只加「设备」一维，记账 key 写作 `(统计量, 树 ID, 设备, 代)`。带不带设备维由 D28（挂载期承诺量） 已定项 1 那个式子直接给：取每块盘自己的值的那几项带，全池承诺量那几项（按全部副本之和记、摊到每块盘扣）不带。

**射程**：维度元组不是每个统计量都用满，哪几项用哪一维在已定项 4 的清单里；两个保留值在已定项 10、各段宽度在已定项 5。这一项不定准入不等式本身（D28（挂载期承诺量） 已定项 1），也不定统计量的个数（已定项 4）。

**依据**：

- E71（记账 key 的条目数与宽度）：按分项计维算出的条目数比「维度用满」那个上界低一到两个数量级，先撑爆预算的是 `树数 × 设备数` 这个乘积，不是四个自变量中的任何一个。
- E97（记账与分配记录的条目编码）：每条根只装一代之后（判据 10 第五次重跑，`research/results/e97-entry-encoding-single-generation-2026-10-20.out`，节点字节只数叶），四档 key 宽（7 / 14 / 19 / 22 字节）的记账树高在 E97（记账与分配记录的条目编码） 登记的四个网格点（树数 × 设备数 = 64 × 8、1024 × 64、8192 × 8、8192 × 64）上、台阶行 0 级时每点彼此相同（窄读 1、2、2、2；宽读 2、3、3、3），格点之间不保证：同一套几何下树数 1 到 8192、设备数 1 到 64 的 524 288 个点里，窄读有 17 440 点、宽读有 92 663 点四档树高不同；`crates/` 今天的行集（池级 3 行 + 每盘 6 行 + 一级台阶行）在 79 到 141 块盘上 7 字节与 22 字节 key 的树高不同。一代口径整批重写，代价是写出的叶数、随条目宽走，不是树高：`1024 × 64` 窄读 22 字节对 7 字节 1.667 倍（81 920 对 49 152 字节），宽读 2.000 倍；`64 × 8` 窄读四档都是一片叶（1.000 倍）。段宽是有代价的，窄读下绝对值小（最大一格 22 字节 507 904 字节，占预算 2.3%）。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/05-快照-空间记账机制.md:47-66`（整段抄，未转述）**

```markdown
#### 已定项 2：「代」取什么，旧代什么时候可以丢

**定案**：代取 checkpoint 号（每次发布 checkpoint_txg + 1，D16（发布语义） 已定项 6，所以代按发布走、不按 5 秒窗口走），记在行 key 里。**每条根的记账树只装这条根那一代的行，不留多代**：「账跟着根回退」由每条根各自 COW 一棵记账树给出，回退到哪条根就读到哪一代的账。例外是 inode 号水位那一统计量的台阶行：随根一直留着（射程「inode 号水位的台阶行」那一条）。

**射程**：

- 代这一维仍要记在行 key 里：记账条目是幂等完整值（D8（核心索引结构） 已定项 1），同一个统计量会被写很多次，扫描重建按已定项 3 / 已定项 10 的合并规则排同一个统计量的新旧要看它。「账跟着根回退」不靠它：账若只有一份就地覆盖的最新值，挂上旧根之后账是新的、树是旧的，I-3.1（已分配统计对得上） 当场判红；每条根各自 COW 一棵记账树之后，挂上哪条根读到的就是哪一代的账。
- 两条骑手条款：代段宽度 **≥ 48 位**（2785 发布/秒下 32 位 17.8 天回绕，E71（记账 key 的条目数与宽度） 的宽度算术要按此重算）；记账条目的**跨发布合并不再可行**（代进了 key），所以每次发布把这一代的行全写一遍，写的行数是实付值，不是上界。
- **inode 号水位的台阶行**（W3 的 C2aL 形态）：记账在 inode 号水位变了的每次发布给这一统计量留一行台阶行 (g, H)，g 是那次发布的 txg，H 是那次发布之后的水位；台阶行随根 COW、一直留着，管理员回退那次从 cur（回退之前的现行那一版）带、不取 R_old 的；「每条根的记账树只装这条根那一代的行」对台阶行不适用。回退那次的 inode 号水位只取没被崩溃恢复抛弃的根（D23（journal 的角色与格式） 已定项 14「水位」那一格），回退不让水位跳升，所以台阶行只在建 inode 与建克隆的发布写；克隆头的台阶行按 (统计量, 树) 分，建克隆那次给新头留一行。台阶行与谱系记录带的 cur 水位按「等于」互核（D18（块里携带什么信息） 已定项 24 第 5 条）。I-3.13（记账行代段等于这一版的 txg） 对台阶行同样不适用：例外写在这里，不变量清单里这一条的条文已写「inode 号水位那一统计量的台阶行不判」，池级 checker（`crates/singlefs-checker/src/walk.rs` 判它的那一段以 `is_an_inode_number_watermark_step` 认出台阶行、跳过）已按这条例外办。代价：一行 34 字节；每次发布多写的单元，台阶行到第 462 行为止是 0，第 463 行起 +2，第 36 074 行起 +4（三方腿的规划器模型算的，对着真规划器的探针核过，没在入库装置上量）。

**依据**：

- E54（记账的「代」要保留几代）：「代取什么」与「留几代」是两个正交问题——不丢那一臂加一条增量丢弃规则之后与只留最近 K 代在 60 个格上逐格相同；不丢时 key 数对 checkpoint 数的斜率等于统计量数，留 K 代时斜率是 0。
- 主 agent 2026-09-23 现查：没有一个读者同时要多代的账——checker 读记账行按（统计量、设备）取值、不读代那一段（`crates/singlefs-checker/src/walk.rs` 走记账树那一支）；恢复只读所选根那棵记账树（`crates/singlefs-core/src/recovery.rs` 读记账树根那一段）；写路径每次发布写当前代的 15 行、从不点删（`crates/singlefs-core/src/transaction.rs` 的 `accounting_entry_count(2)`）。
- 用户定案 2026-09-23：每条根只带自己那一代（里程碑「覆盖写、释放、回退与复用」收口表第 56 行），原话在变更史。
- 用户定案，原话在变更史。
- inode 号水位的台阶行：W3 判决 `research/prompts/m3-w3-r2-main-verification.md` 第二节攻方那几条（C2aL 最强形每次水位变了的发布都留台阶行，要给 I-3.13（记账行代段等于这一版的 txg） 与 D5（快照 / 空间记账机制） 已定项 2 开例外）、`research/prompts/m3-w3-r3-main-verification.md` 第二节克隆头 C2aL 那一行（建克隆那次留行）与第三节代价表；C2aL 的逐字定义在第二轮攻方报告 `m3-w3-r2-opus-output.md` 里「**C2aL 最强形**（我给的）」那一行；用户定案 2026-10-08 Y2「C2aL：记账留台阶行（推荐）」，原话在变更史。无实验：数在三方腿的模型与仓副本原型上量，没立实验号。

**欠**：无。

```

**出处 `.claude/kb/decisions/05-快照-空间记账机制.md:67-89`（整段抄，未转述）**

```markdown
#### 已定项 3：记账的载体形态

**定案**：记账住一棵独立 keyspace 的 btree，条目是「(统计量, 维度元组, 代) → 完整值」。

| # | 定的是什么 |
|---|---|
| 1 | 载体是 btree 里的 key，不是系统配置里的一组标量、也不是盘上一块定长区域——定长区域把「有哪些统计量」焊进格式，而 D12（目标介质） 已定按介质开放布局分支、每条线可能各有统计维度；btree key 形态下**加新统计维度不改格式**（bcachefs 2024 年把计数器做成 btree key 的那个形态） |
| 2 | key = (统计量类型标签, 维度元组, 代号)；value = 该统计量在该代的完整值 |
| 3 | 合并规则按统计量分：非单调统计量「后者胜」（后者由 seq 定），单调统计量取最大值（已定项 4 的第 12、13、14 三项）。取最大只用来排扫描重建时盘上同一个 key 的多个值（已定项 2 射程：「扫描重建按已定项 3 / 已定项 10 的合并规则排同一个统计量的新旧要看它」）；前端排空到叶、读一个 key 时前端与叶并存，都不取最大，按 D8（核心索引结构） 已定项 13：以叶上现值为起点按到达序应用，墓碑就是删。点删的墓碑不参加取最大，把墓碑当数值折进 max 会让被点删的那一代永远留下 |
| 4 | 走 write buffer 前端，条目自带 `seq`——D8（核心索引结构） 已定 write buffer 覆盖「反向索引 / LRU / 记账」三类，seq 是它的第 3 条硬要求（排序丢弃时间序，「后者」必须由条目自己携带）。第一版记账直落叶、不走 write buffer，`seq` 一律 1（D8（核心索引结构） 已定项 10），字段不变 |
| 5 | 整棵记账树按 COW 挂在父指针下（D22（单元原子性怎么合成） 已定项 3） |

**射程**：买到的是「加统计维度不改格式」与 I-3.1（已分配统计对得上） 的判别力，付出的是 key 数量上升（每个统计量 × 每一代各占一条），倍数没量过，D8（核心索引结构） 明写量出来之前不许拿它推「记账占多少空间」。「标签 → 合并规则」是盘上语义，统计量登记表立成独立冻结组件（D15（格式冻结政策））；登记表本身在已定项 10。若某一类账确实做不到幂等，那一类必须另有一条与运行时不共享判定代码的独立重算路径。

**依据**：

- E14（I-3.1 的判别力：幂等完整值 vs 增量 Δ）：幂等完整值形态下四类语义 bug 抓到 3/3，增量 Δ 形态 0/4——value 取完整值是 I-3.1（已分配统计对得上） 还有判别力的前提。
- E38（记账结构 COW 还是原地）：原地形态要 mkfs 在盘上预留全部节点的固定位置，COW 恒为 0；这是 COW 那一侧最硬的依据。
- 三个前置的合取：D8（核心索引结构） 已定项 1 定 value 的语义，D22（单元原子性怎么合成） 已定项 3 定落盘方式，D21（权威态与派生态的分界） 定它必须被根覆盖、参与格式冻结；`.claude/rules/fs-design.md` 第一格定它在事务提交路径上增量维护。用户定案，原话在变更史。
- 定案表第 3 行墓碑怎么折：判决 `research/prompts/m3-close-c693-r2-main-verification.md` 第二节 V4 一格（缩成「前端墓碑对叶上上一次发布的现值怎么折」并进 D8（核心索引结构） 已定项 13，两腿一致）与 `research/prompts/m3-close-c693-r3-main-verification.md` 第二节 T4 一格（缩题并进 D8（核心索引结构） 已定项 13 站住）；改法照正推腿 `research/prompts/m3-close-c693-r2-sonnet-output.md`「V4.5 要动的条款、欠账」那一节。

**欠**：C383（同一次发布里同 key 的中间态折叠没有检查）。

```

**出处 `.claude/kb/decisions/05-快照-空间记账机制.md:90-153`（整段抄，未转述）**

```markdown
#### 已定项 4：统计量有哪几个

**定案**：统计量在用十二个、编号到十四（第 8 / 9 项已撤回，编号不复用），带设备维六项（第 1 / 2 / 3 / 5 / 10 / 11 项）、带树维一项（第 12 项）。清单的完备性相对 D28（挂载期承诺量） 已定项 1 那个准入不等式来判——式子里出现的每一项都必须是被维护的统计量，没出现在任何一条判定里的就不要；做不到的那一项要在「式子逐项对照」表里写出它是谁、为什么。

另有两条随定案一起生效：

- **第 2 项（空闲字节）必须独立维护，不许由 `容量 − 已分配` 现算。** I-5.2（空闲统计对得上） 逐字就是「空闲统计 == 总空间 − 已分配空间」，空闲若是这么算出来的，那条不变量是恒真式、判别力为零；两个数要走两条不共享的加减路径。
- **生命周期判定按设计不要任何聚合统计量。** 已定项 19 的 O(1) 形态建立在「生死判定只用 `ptr.birth` 与 `head.previous_snapshot_txg`，不查记账树」上；往记账树里加一条给生死判定用的统计量，就是破坏那个已经证明过的性质。

**式子逐项对照**（式子的权威正文在 D28（挂载期承诺量） 已定项 1；这张表是式子与统计量清单之间唯一的对照位，门禁 doc-registries 的 admission-terms 格拿两边逐项比）：

<!-- gate:admission-terms -->

| 式子里的项 | 谁维护它 |
|---|---|
| 容量 | **例外**：设备几何，不是记账统计量——它从设备与系统配置读，每次挂载现取，记账不维护它 |
| 已分配 | 第 1 项（已分配字节） |
| 不可回收 | 第 3 项（不可回收字节） |
| 挂载期承诺量 | **例外**：只住内存、不落盘、每次挂载重算（D28（挂载期承诺量） 已定项 3），不是已发布统计量，所以不进统计量清单；它按设备算，崩溃即丢（池级的 checkpoint 保留池单列一项，D28（挂载期承诺量） 已定项 4） |
| 待删占用 | 第 4 项（待删除但意图未完成占用） |
| 已承诺预留 | 第 6 项（已承诺预留量（含墓碑）） |
| 被抛弃根独占量 | **例外**：只住内存的派生量——取值 = 分配器实际隔离的集合的槽数：只被被抛弃根引用的槽（被抛弃根引用的槽 − 回退候选集里的根引用的槽，下界 0；各条根引用的槽从它的分配记录树读，不遍历全池），加可写挂载时预隔离的槽（D23（journal 的角色与格式） 已定项 14「可写挂载时的预隔离」），只除去分配位置着的槽；每次挂载与每次抬 F 时算出，被抛弃的根被轮转覆写时清零（D28（挂载期承诺量） 已定项 1 第九项）；不是已发布统计量，不进统计量清单 |
| checkpoint 保留池 | **例外**：挂载期承诺量的池级成员，按当时各记录树的高之和 + 记账节点现算（D28（挂载期承诺量） 已定项 4），每次发布重算、只住内存、按全部副本之和记而逐设备摊扣，不是已发布统计量 |
| 环窗口 | **例外**：删除各批新写的兜底，(根环槽数 3 × S + 1) × 每批上界 W（D28（挂载期承诺量） 已定项 1）；只住内存、不进记账行、每次发布按几何现算，与 checkpoint 保留池同类，不是已发布统计量 |

五处例外都不是「先记着以后补」：设备容量、挂载期承诺量、被抛弃根独占量、checkpoint 保留池与环窗口都不在崩溃后要重建的那份账里，给它们建统计量等于给一个每次挂载都重算的量做持久记账。清单的完备性只对表里指到「第 N 项」的那几项判。

**统计量清单**：

| # | 统计量 | 谁要它 | 带不带设备维 | 带不带树维 |
|---|---|---|---|---|
| 1 | 已分配字节 | 分配、准入、I-3.1（已分配统计对得上） | **带** | 不带 |
| 2 | 空闲字节 | 准入、checker | **带** | 不带 |
| 3 | 不可回收字节 | 准入、zoned 线的分配、用户可见的 df | **带** | 不带 |
| 4 | 待删除但意图未完成占用 | 准入（I-3.4（可用空间扣待删占用））；恒 0 的保留位：删除中 inode 没释放的单元本来就在「已分配」里（D28（挂载期承诺量） 已定项 1） | 不带（全池承诺量） | 不带 |
| 5 | defer 队列待释放（按代） | defer 窗口、分配、准入 | **带** | 不带 |
| 6 | 已承诺预留量（含墓碑） | 准入（D3（空间分配） 已定项 15 + 已定项 2）；只含每个活对象的墓碑份额、每条 inode 记录的退役记录份额与全池常驻余量（墓碑树 2 + H_墓碑 + 1 槽、退役树 2 + H_退役 + 1 槽，H 是各自现行那一版的内部层数；式子见 D3（空间分配） 已定项 2），不含环窗口 | 不带 | 不带 |
| 7 | 扩展点配额已用量 | 准入（D21（权威态与派生态的分界） 已定项 17 第四条；C21（扩展点指向逃出记账）） | 不带 | 不带 |
| 8 | **已撤回·此编号不再使用** | 曾定「每树独占字节」。撤回依据：三方论证两腿独立普查（14 类判定路径 + 66 条不变量 + 129 条欠账 / 6 条路径）**零命中**，且已定项 4 自己明令「生死判定不查记账树」——不是漏写，是被堵死；唯一的消费者「快照用量」对应 [pitfalls.md](../pitfalls.md) 里标着**未决**的用户可见特性。由此**第一版不提供每快照用量，`df` 只报全池的量**。将来若要做 per-snapshot 配额，它是运行时准入要读的量，当场回到 `.claude/rules/fs-design.md`「记账是事务的副产品」第一格，而 key 编码按 D15（格式冻结政策） 第 3 层冻结、要做就得在冻结前决定；这一格全仓没有欠账 | — | — |
| 9 | **已撤回·此编号不再使用** | 曾定「每树共享字节」，与第 8 项同一次撤回、同一依据 | — | — |
| 10 | 碎片度 `F = runs / L` 的 runs：**单元区空闲槽的连续段数** | 整理的触发判定（D26（后台整理与放置回收） 已定项 6） | **带**（物理量，逐设备一行） | 不带 |
| 11 | 全空聚簇段数 | 整理的触发判定 + 分配自持（同第 10 项） | **带**（物理量——**推论**，依据见 D26（后台整理与放置回收） 已定项 6） | 不带 |
| 12 | inode 号水位（下一个可用 inode 号） | inode 号分配（D8（核心索引结构） 已定项 6：inode 号单调不复用，下一个号在被问到那一刻要有答案） | 不带 | **带**（只为可写头维护） |
| 13 | 清扫水位（截至哪个 txg 的释放已由整轮清扫确认不可读） | 垃圾清扫的准入与墓碑 / 退役记录的回收门（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P4 / P5） | 不带（全池量：一轮清扫覆盖全部设备才推进） | 不带 |
| 14 | 最近一次根销毁的代号 | 墓碑回收门的下限（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P5：门 = max(死亡代号, 这一项)） | 不带 | 不带 |

**射程**：

- 带不带设备维由 D28（挂载期承诺量） 已定项 1 的式子直接给，key 的维度元组不是每个统计量都用满；不按树分的那几项用保留树 ID 0 当「全池 / 无归属」行（已定项 10）。第 10、11 两项不在那个式子里，带不带设备维是按 D26（后台整理与放置回收） 已定项 6 的量纲论述填的推论，要拿去建模先复核。
- 第 12 项只为可写头维护，水位变了的那次发布留一行台阶行，随根 COW 一直留着、不随每次发布重写（已定项 2 每条根的记账树只装这条根那一代的行、每次发布把这一代的行全写一遍，对它开了例外，见已定项 2 射程「inode 号水位的台阶行」那一条），只读树（快照）没有这一行；它带树维，所以树 ID 段仍然留在 key 里。读一个统计量的现行值 = 取该维度元组下代号最大的那一行，空表示从未维护过、不是 0。
- 第 13 / 14 项进清单的理由是「回收判定要它」，都是单调量；清扫水位只在整轮清扫完成时推进，中途进度不持久化，有过一次推进之后才随每次发布重写。第一版不回收墓碑、没有清扫，这两项第一版不登记（D18（块里携带什么信息） 已定项 24）。
- 清单不封闭：D12（目标介质） 的单盘线与 zoned 线还没开，而已定项 3 的依据逐字承认「每条线可能各有统计维度」，第 3 项就是 zoned 提前伸进来的那一个；开线时清单要重开。
- 第 12 项住记账。管理员回退是挂着时的一次向前发布（D23（journal 的角色与格式） 已定项 14）：那次发布的记账统计量由发布路径照调整之后的分配器现算、不从 R_old 那棵账载入，inode 号水位取 max(现行那一版内存里的, 环里读得出、按实例表没被崩溃恢复抛弃的根的)（D23（journal 的角色与格式） 已定项 14「水位」那一格），不退回去、不重发号（`research/prompts/m2-rollback-forward-r3-main-verification.md` 第二节 K1；`research/prompts/m2-rollback-forward-r2-main-verification.md` 第二节 G2）。崩溃恢复落到较旧的根时水位随所选根那棵账载入，被抛弃时间线里已发过的号会被重发；处置是收窄唯一性，D8（核心索引结构） 已定项 6 改成「已发布的 (inode 号, 对象出生代) 不复用」，水位照旧住这里（`research/prompts/c143-r3-main-verification.md` 第四节）。

**依据**：

- E71（记账 key 的条目数与宽度）：每条根只装一代之后（第一次重跑，`research/results/e71-accounting-keys-single-generation-2026-10-20.out`），条目数比随 `树数 × 设备数` 涨，不是一个常数：30 字节宽度 15 格里宽读臂对窄读臂 1.00 到 128.47 倍、上界臂（维度用满）对窄读臂 1.00 到 283.52 倍，`树数 64、设备数 8` 那一格是 14.53 倍与 28.27 倍。过 `T_dirty` 的 1%（21 474 836 字节）的格：三种宽度 45 格里上界臂 5 格（旧口径 9 格）、宽读臂 3 格（旧口径 5 格）、窄读臂 0 格；`1024 × 64` 一格，上界臂在 22 与 30 字节宽度新口径不过（12 976 128、17 694 720 字节）、50 字节过（29 491 200），宽读臂三种宽度新口径都不过（5 879 808、8 017 920、13 363 200），旧口径只有 50 字节过（26 726 400）；`8192 × 64` 一格宽读臂在今天的 34 字节条目宽（`crates/` 的实现宽度）下新口径仍过。`统计量数 × 树数 × 设备数` 是一代的上界不是条目数（inode 号水位的台阶行除外：它随根留着，条目数随水位变过的发布数增长，不受这个上界管）。
- E97（记账与分配记录的条目编码）：一代口径（判据 10 第五次重跑，脏节点计）。窄读最紧的一格是 `8192 × 8` 与 `8192 × 64`（并列）：19 字节臂（判决臂）458 752 字节占预算 2.1%，22 字节臂（`crates/` 今天的 key 宽）507 904 字节占 2.3%；旧口径 `8192 × 64` 那一格 19 字节臂 11 419 648 字节、53%，那是窄读不是宽读。宽读 `1024 × 64` 占 42%（22 字节臂 46%）、`8192 × 8` 占 47%（22 字节臂 53%）不过界，`8192 × 64` 占 337%（22 字节臂 375%）过界（旧口径三格都过界），且那一格反解没有可行的条目宽。这是「哪几项按树分」那一列的代价侧依据之一：今天只有第 12 项带树维、只为可写头维护，付的是窄读那一档的上界，宽读只在最大一格付不起；取舍本身站在「谁要它」上（本项完备性判据）。
- E93（老化下的放置与碎片度）：`F = runs / L` 可增量维护、增量值与全扫审计在全部臂 × 负载 × 种子 × 采样点逐点相等，整理的触发量因此进得了清单。
- 三方论证：第 8 / 9 项撤回时两条腿各自独立普查、零命中；用户定案，原话在变更史。
- 第 4 项恒 0、第 6 项不含环窗口：`research/prompts/m3-delete-r3-main-verification.md` 第三节 G1-标签 4（零轮）、G1-墓碑预付两行，用户定案 K-R3-2「常驻扣项（推荐）」，原话在变更史；第 13、14 项第一版不登记：墓碑判决 `research/prompts/m3-tomb-r1-main-verification.md` 第三节 W3 回收那一行。

**欠**：C122（分配记录不点删之后的条目数口径）；C143（inode 号水位在崩溃恢复落到旧根后退回去重发）；C326（runs 换口径之后 E93 的数还能不能引）。

```

**出处 `.claude/kb/decisions/05-快照-空间记账机制.md:154-176`（整段抄，未转述）**

```markdown
#### 已定项 5：记账 key 各段的宽度

**定案**：`(统计量类型标签 2, 树 ID 8, 设备 4, 代 8)`，value = 完整值 8 字节，`seq` 4 字节住 value（D8（核心索引结构） 已定项 7）⇒ key 22 字节、条目 34 字节。代段存完整 checkpoint 号，不许取模。

| 段 | 宽 | 依据的等级 |
|---|---|---|
| 统计量类型标签 | **2** | **自由选择**：在用十二个、编号到十四，1 字节就够；取 2 是留扩展余量（已定项 4 自陈清单不封闭、D12（目标介质） 不封顶介质套数）；多出的 1 字节让记账叶的容量从 491 条掉到 477 条（`crates/` 几何的算术，少 2.9%）。多不多写节点看记账树总行数（3 + 6 × 盘数 + 台阶行条数）跨没跨过一片叶装满的门槛：两块盘上台阶行 463 到 476 级时，21 字节 key 一片叶、22 字节 key 两片叶加一个根，每次发布多写 2 个节点 |
| 树 ID | **8** | **自由选择**：上界一条已定条款都指不到（D6（快照实现模型） 定每个可写头一棵自己的树、快照数无上界），不许拿一个窄宽度去制造一个设计上限 |
| 设备 | **4** | **随 D19（块指针的结构与宽度预算） 已定项 4**，同一个坐标系 |
| 代 | **8** | **已定条款直接给的**：已定项 2 的骑手条款要求代段 ≥ 48 位，而 D22（单元原子性怎么合成） 已定项 7 的 `checkpoint_txg` 就是 8 字节 |
| value（完整值） | **8** | 统计量是字节计数，256 TiB 也只用到 48 位 |

**射程**：这五段不构成一个定得下合并结果的 key——记账条目是幂等完整值，同一个统计量在同一个 checkpoint 内会被写很多次，`(标签, 树, 设备, 代)` 在一个 checkpoint 内不唯一，「后者胜」的「后者」要靠 `seq` 定；`seq` 住 value，这五段的排序契约不动。用户定案是「先按这个形态落地，写代码时再调整」。

**依据**：

- E97（记账与分配记录的条目编码）：一代口径下四档 key（7 / 14 / 19 / 22 字节）的树高在登记的四个网格点上、台阶行 0 级时彼此相同，格点之间不处处相同，代价曲线不是平的——整批重写时写出的叶数随条目宽走（叶扇出 1087 / 740 / 603 / 542；`1024 × 64` 宽读 5 046 272 / 7 405 568 / 9 076 736 / 10 092 544 字节，窄读 49 152 / 65 536 / 65 536 / 81 920 字节）；代段模 2³² 的回绕期 17 天、模 2⁴⁸ 3204 年，而任何有限模都只是把破 key 序推迟到回绕那一刻（模 256 在 257 代出 255 个逆序对、91 毫秒绕完一圈），这是「不许取模」的可执行依据。
- E44（序号位宽的本机实测与代价）：48 位撑 3202 年，与代段 ≥ 48 位那条骑手条款是同一个量级。
- E73（节点 key 区间的扇出代价）：按 22 字节的 key 算，节点带 key 区间之后扇出最多掉一格、树高一层没涨。
- 用户定案，原话在变更史。

**欠**：C162（seq 进 value 之后四个实验的数要重算）。

```

**出处 `.claude/kb/decisions/05-快照-空间记账机制.md:177-191`（整段抄，未转述）**

```markdown
#### 已定项 6：稀疏旁表的落点与字段表

**定案**：四问各一句——① 载体自成一棵 keyspace 的 btree，day-1 注册进树表（新池新建文件多一条 200 字节的树表条目，根指针为零）；② key 取中央映射 key（D19（块指针的结构与宽度预算） 已定项 6：码 1 是 27 字节，码 2 / 码 3 的 25 字节末尾补零到 27，D19（块指针的结构与宽度预算） 已定项 10），不带版本维——映射 key 里的出生 txg 已经是版本身份；value 是 4 字节引用计数，条目 31 字节一宽；③ 提交时增量维护，落 `.claude/rules/fs-design.md` 第一格（跨头共享块的生死判定要读它）；④ 走 D8（核心索引结构） 已定项 5 的意图机制，删除分批。

**射程**：改新池新建文件的字节——树表里稀疏旁表那一条条目（200 字节），条目本身新池新建文件一条都不写。它只答「住哪、条目长什么样、谁维护、走不走意图机制」四问；跨头共享取哪条机制在 D6（快照实现模型） 已定项 1，旁表补的是已定项 17 那条被越过的前提。D26（后台整理与放置回收） 已定项 5 剩的那一问（deadlist 条目装什么）随这一项收口：条目按版本身份记，落成字段表在已定项 12。用户定案没有走三方论证，可推翻。

**依据**：

- E52（两种多可写头机制的运营代价）：旁表条目数在两格上分别是 1 000 与 50 000 条，规模确实是稀疏的。
- E12（攒批的顺序追加 vs 不攒批的随机页读改写）：装得进缓存时几乎免费，装不下时每次删除约 1 次设备 I/O。
- E146（livelist 条目按映射 key 定身份之后的宽度与代价）：同型结构 day-1 注册要付的那笔账（一条树表条目，空树若要根节点再加一个单元）。
- 用户定案，原话在变更史。

**欠**：C236（稀疏旁表被定为必做却没有任何落点）——立项那一半已还，会红的检查仍欠。

```

**出处 `.claude/kb/decisions/05-快照-空间记账机制.md:192-204`（整段抄，未转述）**

```markdown
#### 已定项 7：「已分配字节」含不含 mkfs 写下的固定结构

**定案**：不含。「已分配字节」= 分配记录树里落点之和；准入不等式里的「容量」= 单元区大小（系统配置「单元区起始槽号」到盘尾）。固定结构（系统配置槽、根环、journal 环）既不在容量里也不算已分配；I-5.2（空闲统计对得上） 的「总空间」同此。

**射程**：只定这两个量的口径，不定准入不等式本身（D28（挂载期承诺量） 已定项 1），也不定单元区起始槽号取多少（D3（空间分配） 已定项 10）。

**依据**：

- E142（新池新建文件的干跑）：新池新建文件的已分配是 17 个 16 KiB 槽，固定结构一个字节都不在其中，逐字段与 `crates/` 实装比对全等。
- 用户定案与写代码前的总审核，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/05-快照-空间记账机制.md:205-229`（整段抄，未转述）**

```markdown
#### 已定项 8：新池新建文件记账树里写哪几行

**定案**：规则是有值的统计量就写一行，**没有对象的统计量不写行**（读空表示从未维护，不等于 0）。新池新建文件写 **15** 行，key 按 `(统计量标签 2, 树 ID 8, 设备 4, 代 8)` 升序，标签 = 已定项 4 那张表的行号，不带设备维的设备段写 0xFFFF_FFFF、不带树维的树 ID 段写 0，代 = checkpoint_txg 3：

| 行 | 统计量（已定项 4 的行号） | key | 值 |
|---|---|---|---|
| 1–2 | 已分配字节（1） | (1, 0, 0, 3)、(1, 0, 1, 3) | 每盘 278 528 |
| 3–4 | 空闲字节（2） | (2, 0, 0, 3)、(2, 0, 1, 3) | 每盘 3 472 605 184（4 GiB 镜像的预想） |
| 5–6 | 不可回收字节（3） | (3, 0, 0, 3)、(3, 0, 1, 3) | 0 |
| 7 | 待删除但意图未完成占用（4） | (4, 0, 0xFFFF_FFFF, 3) | 0 |
| 8–9 | defer 队列待释放（5） | (5, 0, 0, 3)、(5, 0, 1, 3) | 16 384（第一个文件版本重写树表时把 mkfs 那片第 0 版树表单元换下，D3（空间分配） 已定项 7） |
| 10 | 已承诺预留量（6） | (6, 0, 0xFFFF_FFFF, 3) | 196 760（D3（空间分配） 已定项 2 的式子，树表有 `deleted_inodes` 条目，2 × ((2 + 0 + 1 + 2 + 0 + 1) × 16384 + 41 × 1 + 35 × 1)） |
| 11–12 | 碎片度 runs（10） | (10, 0, 0, 3)、(10, 0, 1, 3) | 每盘 4 |
| 13–14 | 全空聚簇段数（11） | (11, 0, 0, 3)、(11, 0, 1, 3) | 每盘 3310（4 GiB 镜像的预想） |
| 15 | inode 号水位（12） | (12, 65537, 0xFFFF_FFFF, 3) | 2 |

**射程**：改新池新建文件的字节：是——记账树单元的条目数（15）、声明长度（15 × 34 = 510）与 15 条记录的内容。准入不等式要的四项统计量（不可回收、待删占用、已承诺预留、defer 待释放）day-1 各写一行，为的是保住「空不等于 0，空 = 从未维护」那道防线：四项若 day-1 一行都不写，第一次挂载读到的就是空，而准入不等式（D3（空间分配） 已定项 4 / D28（挂载期承诺量） 已定项 2）要读它们，按「空 = 从未维护」判就该拒绝可写挂载、按「空按 0 读」判就把防线撤了；各写一行是唯一既保防线、又让第一次挂载过得去的走法，代价是 6 行 × 34 字节 = 204 字节。空闲字节与全空聚簇段数两行的值随镜像大小变，4 GiB 是里程碑步 0 的预想。清扫水位与最近一次根销毁的代号两项此时还没有对象，一行都不写。用户定案没有走三方论证，可推翻。

**依据**：

- E142（新池新建文件的干跑）：15 行的 key 与值逐字段写成字节；第二十二次跑与 `crates/` 逐区域比对全等（29 个区域 `equal=29 unequal=0`，第 10 行两边都是 98 386）；记账树单元的声明长度与条目宽度都在那一页。
- 用户定案与写代码前的总审核，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/05-快照-空间记账机制.md:230-245`（整段抄，未转述）**

```markdown
#### 已定项 9：可写头由哪个 ID 标识，`previous_snapshot_txg` 归谁

**定案**：可写头的 ID = 该头 inode 树的树 ID。记账的树维、livelist 共享树的 key 前缀、树表条目的 `previous_snapshot_txg` 归属三处都用它。`previous_snapshot_txg` 写在两处：该头 inode 树那条树表条目里，与每条码 14 快照条目里（= 捕获那一刻活头的值）；其余条目写 0 且 0 = 不适用；inode 树那条条目写的非 0 值不高于引用这张树表的根的 txg（I-9.25（树表条目的 previous_snapshot_txg 取值））；还没打过快照的新头也写 0（快照的 txg ≥ 1，mkfs 的 txg 0 没有用户可见状态，不会撞）。非 inode 树靠树表条目里的「头 ID」找到它归哪个头：树表条目从预留里切 8 字节存它（预留是 76 字节，D8（核心索引结构） 已定项 8），值的规则是每头一份的结构写它服务的那个头的树 ID，全池一份的写 0——inode 树条目写自己的树 ID（新池新建文件是 65537），extent 树条目写它服务的那个头（65537），`deleted_inodes`、墓碑树与退役树写 inode 树的树 ID（65537），谱系树写 0（全池一份），logged_ops 写 0（整理意图条目的字段表已定，见 D26（后台整理与放置回收） 已定项 2；C364（整理意图记录的字段没有逐字段对账） 仍欠字段表与实现的逐字段对账，I-4.4（意图只描述目标态） 的字段白名单判法已实现），分配记录树 / 记账树 / livelist 共享树 / 稀疏旁表树 / deadlist 树写 0（无归属），中央映射树不进树表。

**射程**：只定「谁标识一个头」与「前驱快照号写在哪条条目上」，不定树表条目本身的字段与宽度（D8（核心索引结构） 已定项 8），也不定 `previous_snapshot_txg` 在销毁级联里怎么传（已定项 13）。只读快照的码 14 条目的头 ID = 被捕获的活头 inode 树的树 ID，码 15 条目的头 ID = 配对的码 14 条目的树 ID、前驱（`previous_snapshot_txg`）写 0。

**依据**：

- E142（新池新建文件的干跑）：树表第 1 版 11 条条目（退役树加进之前的一版）的头 ID 与 `previous_snapshot_txg` 逐字段写成字节，与当时的 `crates/` 实装比对全等（2026-10-05 第二十四次跑）；退役树加进之后实装是 12 条，装置在第二十五次跑（2026-10-13）追平，树表 12 条、水位 65549，主点 29 个区域逐字节全相同。
- 用户定案与写代码前的总审核，原话在变更史。
- 判决 `research/prompts/m3-blanks-r1-main-verification.md` 第二节头 ID 那一行：旧定义句与它自己列的五棵写 0 的例子矛盾、判不出新树，改成每头一份写它的头、全池一份写 0。
- 用户定案 2026-10-01：谱系树的头 ID 写 0、全池一份（岔路单 `m3-blanks-r1-forks.md` 的「谱系树的头 ID」一行，原话在变更史）。
- 判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1（码 14 条目的头 ID = 活头 inode 树的树 ID、`previous_snapshot_txg` = 捕获那一刻活头的值；码 15 条目的头 ID = 配对码 14 的树 ID、前驱 0）与 S5（I-3.7（活快照可枚举） 的前驱链要码 14 带前驱）。

**欠**：无。

```

**出处 `.claude/kb/decisions/05-快照-空间记账机制.md:246-258`（整段抄，未转述）**

```markdown
#### 已定项 10：统计量标签登记表与两个保留值

**定案**：统计量标签的登记表就是已定项 4 那张表——码 = 行号（1 到 14；第 8 / 9 项退役不复用；0 保留为无效），每行的合并规则按已定项 3（单调量取最大值：第 12 / 13 / 14 项；其余后者胜）、带不带设备维与树维照表。不带设备维的统计量，key 的设备段取保留值 0xFFFF_FFFF；不带树维的树 ID 段取 0。

**射程**：登记表管的是「标签码 → 哪个统计量、怎么合并、用哪几维」，不管一个老读者读到不认识的标签时该怎么办——那一格是 compat 档位的问题，D15（格式冻结政策） 的判定表没有一行处理它。

**依据**：

- E142（新池新建文件的干跑）：新池新建文件 15 行 key 里两个保留值（设备段 0xFFFF_FFFF、树 ID 段 0）逐字节写出，与 `crates/` 实装比对全等。
- 用户定案与写代码前的总审核，原话在变更史。

**欠**：C71（不认识的统计量标签静默忽略）；C117（不带设备维的统计量的设备段没有取值）——取值已定，会红的检查仍欠。

```

**出处 `.claude/kb/decisions/05-快照-空间记账机制.md:259-272`（整段抄，未转述）**

```markdown
#### 已定项 11：deadlist 的载体 day-1 建不建

**定案**：建。deadlist 自成一棵 keyspace 的树，**day-1 注册进树表**——树的种类码 **8**（D8（核心索引结构） 已定项 9 的登记表）、树 ID **65543**、根指针零（还没有根）、诞生 txg 3、头 ID 0（无归属）。树表第 1 版连同 D18（块里携带什么信息） 已定项 24 的五棵零根树共 **12 条**，树 ID 水位发布后 **65549**。新池新建文件一条 deadlist 条目都不写（没有释放）。

**射程**：改新池新建文件的字节——树表第 1 版多 deadlist 一条（连同 D18（块里携带什么信息） 已定项 24 的五棵零根树共 12 条，声明长度 12 × 200 = 2400；退役树加进之前是 11 条、2200），树 ID 水位发布后 65549（退役树加进之前 65548；根记录与 journal 新根段各一份）。这一项只定载体建不建、什么时候建，条目长什么样在已定项 12。用户定案没有走三方论证，可推翻。

**依据**：

- E146（livelist 条目按映射 key 定身份之后的宽度与代价）：livelist 共享树是同型的先例，day-1 注册与惰性创建各要付什么在那一页；惰性创建会多出一条「第一次建快照时建树」的路径，那条路径要在一次发布里建树、写树表条目、抬树 ID 水位。
- E142（新池新建文件的干跑）：树表第 1 版 11 条（含 `deleted_inodes` 等四棵零根树）与树 ID 水位 65548 逐字段写成字节，与当时的 `crates/` 实装比对全等（2026-10-05 第二十四次跑）；此后加了退役树，实装已是 12 条、水位 65549；装置在第二十五次跑（2026-10-13）追平，树表 12 条、水位 65549，主点 29 个区域逐字节全相同。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/05-快照-空间记账机制.md:273-299`（整段抄，未转述）**

```markdown
#### 已定项 12：deadlist 条目的形态

**定案**：取「丙-区间（带上界）」。

| 项 | 定的是什么 |
|---|---|
| 条目 | key = 头的树 ID 8 + 纪元 8 + 中央映射 key 27 = 43 字节，value 空（key 即事实）。身份就是中央映射 key；纪元 = 块被杀那一刻头的 `previous_snapshot_txg`，分桶就是这一段，不另设桶号字段 |
| 哪一份 | 快照 X 那一份 = 纪元落在 [prev(X).txg, X.txg) 的条目；头那一份 = [prev(头).txg, ∞)。建快照只推进 prev(头)，已有条目一条不动——已定项 13 那句「整个交接给新快照」说的就是这件事，不搬条目 |
| 销毁 S | 写意图 (头, lo = S.txg, hi, P = prev(S).txg)：hi 在下一侧是快照时取它的 txg，是头时取写意图那个开放 checkpoint 的号（取已发布的最大号会让区间为空、条目永远不放）；prev(S) 在写意图的那个事务里传给那一刻的下一侧，不许拆到以后的发布；之后按批放掉 [lo, hi) 里 birth > P 的条目，不改任何条目的键；一批里每个动作施加前先核它要删的条目还在（D8（核心索引结构） 已定项 5 硬要求第 4 条）；级联之间不要次序，销毁头的意图等同一个头上更早的级联做完才动条目 |
| 下一侧怎么找 | 树表条目没有前后快照的链接，写意图那一刻从挂载时扫树表建的「头 → 活快照链」派生态取：零字节，丢了只是慢（D21（权威态与派生态的分界） 的派生态判据） |
| 按头 / 按快照删一段 | 前缀（头）/（头, 纪元落在 [prev(X).txg, X.txg)）一段 |
| 树表字节 | 0；预留 76 字节留给别的认购者 |
| 改新池新建文件的字节 | 否——新池新建文件不删任何东西，一条条目都不写 |

**射程**：被销毁的快照从写意图那一刻起不再钉住块，这是已定项 20「待删除结构算作活引用」的例外；被销毁的可写头从哪一刻起不算、分叉点闸的 `children` 何时减，没有条款。条目形态定了，会红的检查还没有。

第一版的销毁意图写在被销毁快照自己的码 14 树表条目上：只占 flags 位 0（销毁中），预留一个字节不写；lo = 条目的诞生 txg，P = 条目的前驱，hi 每批从派生链现取（下一条码 14 的诞生 txg，下一侧是头时取写意图那次发布的 txg），不存续做点（条目被放掉就是进度）；不住 logged_ops。销毁串行：一次调用 = 先走空 + 意图 + 全部批，删完再接别的改动（用户定案）。删快照照 unlink 的过程做、同等准入，不对「先走空」（D8（核心索引结构） 已定项 16 第 6 条）开例外（用户定案）。下一侧从树表按（诞生 txg, 树 ID）排的派生链取。

**依据**：

- 三轮三方判决：`research/prompts/d5-item12-r1-main-verification.md`、`d5-item12-r2-main-verification.md`、`d5-item12-r3-main-verification.md` 各自第四节。丙（点读法）两轮判出局、甲在 U2 出局；丙-区间第二轮的最弱读法（意图区间 [S.txg, ∞)）6 步多放，撞 D8（核心索引结构） 已定项 5「意图声明目标状态，绝不声明增量」，收严成带上界；第三轮只在「一批按批开始时的状态计划、施加时不复核」下 7 步重复放，撞同一条硬要求的幂等，收严成施加时复核。乙与丁 第二轮在任意次序下漏放、带次序之后第三轮五个面都不中，代价是级联改键，五个销毁几何上丙-区间 0 条、乙 D(S)、丁 K（`out-geometry.jsonl`）。
- E94（搬共享落点的触碰面）：按落点记会让每一次搬迁都要逐条重写 deadlist，按版本身份记时中央映射恒一根、deadlist 重写恒零。
- 判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S4（意图载体取码 14 条目自身，只占 flags 位 0、预留不写，lo 与 P 在条目上、hi 每批从派生链现取、不存续做点，不住 logged_ops）与第七节用户定案（岔路 2 销毁串行、岔路 4 删快照照 unlink 的过程做），原话在变更史。
- 用户定案，原话在变更史。

**欠**：C145（deadlist 在盘上没有落点）——条目形态已定，checker 那一半仍欠；C336（写意图那一刻找不到快照的下一侧）。

```

**出处 `.claude/kb/decisions/05-快照-空间记账机制.md:300-319`（整段抄，未转述）**

```markdown
#### 已定项 13：生命周期判定取哪套机制

**定案**：取诞生代号 + deadlist 模型（ZFS 的 birth txg + deadlist）。

- 每个块自带**诞生代号**（birth txg）。
- 删除时把块的代号与**活头记录的 `previous_snapshot_txg`**（上一个快照的代号）比：`birth > previous_snapshot_txg` 就**直接释放**，否则**追加进活头自己的 deadlist**。
- 建快照时把**活头的 deadlist 整个交接给新快照**，活头换一个空的（已定项 12：交接不搬条目——条目按纪元分段，建快照只推进 prev(头)）。
- 销毁快照时把它的 deadlist **合并到下一个更新的那一侧**，并按 `birth > prev(S).txg` 过滤出可释放的条目，**不做全盘扫描**（往哪一侧并在已定项 16）。
- **`prev` 要跟着传递**：被销毁那一侧的 `prev(S).txg` 由承接的一侧继承下去，在写意图的那个事务里就传给那一刻的下一侧，不许拆到以后的发布（拆开时三条候选都 5 步多放）。不传递则代代都拿自己原来的 `prev` 去过滤，`birth` 小的条目永远等不到 `birth > prev` 成立，那是无界泄漏。

**射程**：引用判据与三个量的定义在已定项 14，运行时形态与代价口径在已定项 19，适用前提在已定项 17。与 D9（加密） 的连锁：birth txg **不能**当加密 nonce——同一事务里的块共享它，nonce 会重复。D5（快照 / 空间记账机制） 自己那次实测只剩崩溃重放一格没跑：deadlist 的追加与级联合并**不幂等**（多重集相加，同一条目追加两次就会在销毁时双重释放），而双重释放是数据丢失方向，已证的「数据丢失方向结构性为零」只覆盖稳态下的 `death` 规则取值；崩溃点重放的两条流一条 deadlist 条目都不写，这一格仍然零覆盖，在它跑起来之前不许说 D5（快照 / 空间记账机制） 的风险路径验过了（E13（D5 的四条风险路径） 路径 3）。

**依据**：

- E12（攒批的顺序追加 vs 不攒批的随机页读改写）：20 万次删除里，诞生代号这一侧的设备 I/O 在三档缓存上恒为 2 次，全域引用计数在计数器表装不下缓存时跳到约每次删除 1 次，转折是陡的；块层独立校验（虚机内 `/sys/block/vda/stat`）九个格子比值全部 1.00，摘掉 `O_DIRECT` 后块层读数归零而程序计数器不变，这条校验路径有判别力。
- E94（搬共享落点的触碰面）：`prev` 不传递时条目会被永久钉住，变异 M5b 在模型层验过这条会红。
- 别家记录（线索，未在本项目验证）：btrfs qgroup 走 extent refcount + 反向索引遍历，代价是提交时间 +76%、事务等待 +1347%、写吞吐 −25%（口径是「qgroup 走反向索引」，不是「纯引用计数」）；bcachefs 的记账由触发器在 key 提交时维护、`check_allocations` 用全量遍历重建，印证的是「增量维护 + 遍历校验」这个模式，不是这套机制（bcachefs 没有 deadlist），而且它那套遍历校验与运行时共用同一段加减代码，抓不到「语义本身写错」（[prior-art.md](../prior-art.md) 7.11），本工程不许照搬那个形态。

**欠**：C195（销毁级联时 prev 传递只写在实验的模型修订记录里）。

```

**出处 `.claude/kb/decisions/05-快照-空间记账机制.md:320-349`（整段抄，未转述）**

```markdown
#### 已定项 14：引用区间的充要条件

**定案**：**块 b 被快照 S 引用 ⟺ `birth(b) ≤ S.txg < death(b)`**，左闭右开。

| 量 | 定义 | **不是**什么 |
|---|---|---|
| `birth(b)` | 块被**发布**的那个 checkpoint 号 | 不是写请求发出时所在的开放 txg——崩溃后该号会被重发 |
| `death(b)` | 最后一个活引用被摘掉、**且该摘除被发布**的 checkpoint 号；仍在活树里则为 ∞ | 不是 `rm` 返回的时刻（D8（核心索引结构） 走意图分批），也不是空间变得可再分配的时刻（D16（发布语义） 已定项 15） |
| `S.txg` | **S 捕获的最后一个已发布 txg**；建快照是独立发布（不带数据变更的一次发布）时，S.txg 就是那次发布的 `checkpoint_txg`，同一个值写进活头 inode 树条目的 `previous_snapshot_txg` 与快照条目的诞生 txg | 不是「创建操作所在的 txg」 |

三条边界与取错的后果：

| 边界 | 判定 | 取错的后果 |
|---|---|---|
| `birth == S.txg` | S **引用**它 | 写成 `birth < S.txg` 就判为「比快照新」、立即释放，**数据丢失** |
| `death == S.txg` | S **不引用** | 写成 `S.txg ≤ death` 就挂在不需要它的快照上，**空间泄漏 + 归属错** |
| `birth == death` | 任何快照都不引用，可立即释放 | 记账上对，但**物理空间不许在本 checkpoint 内复用**（D16（发布语义） 已定项 15） |

**保守默认纪律：判据的任何一个输入取不到或不确定时，默认必须是「进 deadlist」。**

**射程**：这个条件只在 txg 全序（单向线性历史）时等价于可达性，越过前提之后怎么办在已定项 17；按这条 `death` 定义，分叉下判死实活恒为零，这是可证的——truth 为真说明块在该根的链上没死，不满足「所有分支都删过」，`death = ∞`，区间判定必为真。保守默认那条的理由是两个方向不对称：泄漏可被 I-3.6（deadlist 紧界） 检出（**不是** I-3.1（已分配统计对得上）——[invariants/05-空间记账.md](../../../crates/singlefs-checker/invariants/05-空间记账.md) 明写「归属错误对它不可见……总账依然平」）、可回收、伤害有上界；丢失不可逆，一旦空间被复用就退化成「校验和不匹配、原因不明」，连归因都没了。`S.txg` 取第二种定义是机制性的：`birth` 的分辨率只到 checkpoint，取「创建操作所在 txg」会让条件随「快照点放在 checkpoint 头还是尾」在 `birth ≤ S.txg` 与 `birth < S.txg` 之间跳，而定义要选让不变量不随实现位置摆动的那个。

**依据**：

- E27（D5 三条可建模的风险路径）：三条边界取错的后果确实不对称，而且不是三处独立的笔误——右边界写成 `S.txg ≤ death` 同时命中 `death == S.txg` 与 `birth == death` 两行，一个笔误造成两处泄漏；两种取错的失败方向相反（写严左边界只丢数据 1 处、写闭右边界只泄漏 2 处，零交叉）。
- E18（D5 在多分叉 / 多可写头下是否报废）：按这条 `death` 定义，线性、一次分叉、8 个可写头三档的判死实活全部是 0。
- 判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S2：建快照取独立发布，S.txg 取那次发布的 `checkpoint_txg`；攻方模型上 S.txg 与 prev(头) 同源的读法 0 违例。

**欠**：无。

```

**出处 `.claude/kb/decisions/05-快照-空间记账机制.md:350-361`（整段抄，未转述）**

```markdown
#### 已定项 15：后台搬迁改不改 `birth`

**定案**：不改。后台整理把一个单元从一个落点搬到另一个落点，**不是「发布一个新版本」**——逻辑内容一个字节没变，只是物理位置换了；按已定项 14 给 `birth(b)` 的定义句「块被**发布**的那个 checkpoint 号」，搬迁不产生新的发布事件，`birth` 原样跟着单元走。

**射程**：只管 `birth` 跟不跟着搬，不管搬迁本身要触碰哪些结构（D26（后台整理与放置回收） 已定项 5）。deadlist 条目按版本身份记、不按落点记、也不带位置提示，是同一条理由的下游：按落点记会让每一次搬迁都要逐条重写 deadlist，带提示则买不到任何东西（释放一律经映射，D19（块指针的结构与宽度预算） 已定项 5 硬规则 1），反而给实现留了一条会双重释放的路；条目落成字段表在已定项 12。删除侧从指针取出生树、不必多读单元头（D19（块指针的结构与宽度预算） 已定项 7）。

**依据**：

- E94（搬共享落点的触碰面）：搬迁若重新指派 `birth`，重指派的违例数恰等于被快照引用的搬动数——每搬一个被快照引用的单元就错一次；`birth` 不改则违例恒 0。

**欠**：无。

```

**出处 `.claude/kb/decisions/05-快照-空间记账机制.md:362-373`（整段抄，未转述）**

```markdown
#### 已定项 16：销毁快照时 deadlist 往哪一侧并

**定案**：取约定 B——销毁快照时把它的 deadlist 并到**下一个更新的那一侧**。

**射程**：约定 B **不是**「快照的 deadlist 永不改写」——销毁级联的目标 `dsl_deadlist_merge(&ds_next->ds_deadlist, ...)` 里 `ds_next` 本身就是快照，取约定 B 的理由是例外少一个数量级，不是没有例外。两种约定的差别是**改写已发布快照 deadlist 的频率**，不是改不改写——另一种约定每一次 kill 都要往一个已发布快照的 deadlist 里追加，约定 B 只在销毁级联时改写一次。I-3.6（deadlist 紧界） 那条紧界 `birth ≤ prev(S).txg` 是按约定 B 写的；级联里 `prev` 怎么传在已定项 13，分叉点没有唯一的下一侧、因此不许销毁在已定项 18。

**依据**：

- 无实验：两种约定的差别是「改写已发布快照 deadlist 的频率」，本工程没有量过这个频率，也没有第二个可量的量；线索是现役实现——ZFS 实现的就是约定 B（现查 `module/zfs/dsl_destroy.c`：`/* Merge our deadlist into next's and free it. */`），未在本项目验证。

**欠**：无。

```

**出处 `.claude/kb/decisions/05-快照-空间记账机制.md:374-392`（整段抄，未转述）**

```markdown
#### 已定项 17：四条前提，越过任何一条它就失效

**定案**：四条前提——① 线性历史（无分叉）；② 单一可写头；③ 每个块在活树里至多一个活引用；④ 快照点与 checkpoint 边界对齐（D16（发布语义） 已定项 14）。**任何引入「同一块的第二个活引用」的特性，必须先重开 D5（快照 / 空间记账机制）。** 这条重开条款已经被触发过一次并已收口：D6（快照实现模型） 已定项 1 取「每头一棵自己的树」，而跨头共享的块就是第二个活引用；⇒ 按 D5（快照 / 空间记账机制）自己给出的正确形态收口：**D5（快照 / 空间记账机制）为主 + 稀疏旁表为辅（ZFS 的 BRT 形态）**——不是把 D5（快照 / 空间记账机制） 换成全域引用计数，旁表的落点与字段表在已定项 6，三条连带要求在已定项 18。

**射程**：

- 前提 3 与 reflink / dedup / 块克隆机制上不相容：有第二个活引用时摘掉其中一个不构成 death，而判据只看 `birth` 与 `previous_snapshot_txg`、两个引用的 birth 完全相同，第一次摘除就判「立即释放」，另一个引用悬空。ZFS 二十年没有 reflink，它 2023 年的 block cloning 补的方式是**另加一张带引用计数的表**，不是扩展 birth/death（线索，未在本项目验证）。
- 越过前提 1、2 之后的失效形态是**无界泄漏 + 记账退化到引用计数**，不是数据丢失：维护「是不是最后一个活引用」需要的正是每块的分支覆盖信息，也就是引用计数，而那正是 D5（快照 / 空间记账机制） 想省掉的东西。**D5（快照 / 空间记账机制） 的 O(1) 性质就处在这个状态**，补回它的是稀疏旁表。
- 「D5（快照 / 空间记账机制） + 稀疏旁表」那条臂把代价压回去的约 100 倍只能当**待验的量级指示，不许当作已验证的收益写进任何判据**：它的唯一来源那条臂没有失败条款、也没有阳性对照，臂 A / 臂 B 那套块层独立校验与故障注入自检没有对它做过，要用它先补失败条款与阳性对照再重跑。
- 旁表规模的外推边界：探测用的是内存全域位图、每块 1 bit（真实文件系统上约 128 MB/TB），高估了旁表在大盘上的表现，换成真正的稀疏结构必须重测。局部性也已量出：删除序列从均匀随机换成顺序，引用计数 8× 档从 174 998 次 I/O 掉到 391 次（448 倍），真实负载有局部性时 D5（快照 / 空间记账机制） 的优势显著小于主表那个数量级，**必须与主结论一起引用**。checker 代价与崩溃一致性不在那条臂的射程里。

**依据**：

- E18（D5 在多分叉 / 多可写头下是否报废）：一次分叉时判活实死 8.19%、8 个可写头时 29.40%，而 8 个可写头那一档正确规则与「根本不记 death」逐位相同——它用「一块都不回收」换零数据丢失。
- E12（攒批的顺序追加 vs 不攒批的随机页读改写）：20 万次删除的设备 I/O 中位数，D5（快照 / 空间记账机制） 三档恒 2 次，全域引用计数 512 / 约 200 208 / 约 350 042 次，D5（快照 / 空间记账机制） + 稀疏旁表 508 / 约 1 956 / 约 3 466 次；计数器表装得下缓存时引用计数几乎免费，那是合法结果不是实现 bug——D5（快照 / 空间记账机制） 的优势条件是「计数器表装不下缓存」，不是无条件成立。
- E52（两种多可写头机制的运营代价）：收口形态里旁表的条目数在两格上分别是 1 000 与 50 000 条。

**欠**：无。

```

**出处 `.claude/kb/decisions/05-快照-空间记账机制.md:393-414`（整段抄，未转述）**

```markdown
#### 已定项 18：克隆 / 可写快照要加三条

**定案**：前提 1、2 被越过之后要加三条。D6（快照实现模型） 已定项 1 取每头一棵树之后，这三条是必做，不是备查。

1. 每个可写头各持一份 `previous_snapshot_txg` 与一份 deadlist，**新克隆的 `previous_snapshot_txg` 必须初始化为 origin 快照的 txg**——置 0 则克隆里删一个继承来的老块会算出「立即释放」，**数据丢失**。
2. **分叉点快照不许销毁**（约定 B 在分叉点没有唯一的下一侧）。判据是 O(1) 子计数 `children > 1` → 拒绝，`children` **把从这个快照克隆出去的可写头也算上**。**另一道闸：还有自己活快照的可写头不许销毁**——要销毁一个可写头，先销毁它自己打出的全部快照；它克隆时依附的那个 origin 快照不算它自己的。
3. **销毁一个可写头需要第二个结构**（livelist 形态：ALLOC/FREE 配对、按 birth txg 分组、可增量、可 condense）。不承认它，「永不事后遍历」在这条路径上就是假的——要释放的是 `birth > origin.txg` 的那批块，散落全树。

**射程**：

- 第二道闸买到的是第 3 条那句「要释放的是 `birth > origin.txg` 的那批块」按字面成立：销毁那一刻头自己的快照已全部销毁，`previous_snapshot_txg` 已沿已定项 13 的 `prev` 传递退回克隆时刻。没有这道闸时，销毁头那一刻 `previous_snapshot_txg` 可以高于它的克隆时刻，`birth ≤ previous_snapshot_txg` 那批块还被它自己的快照引用、不能放，而这批块的去处全仓没有定义——填成「并进最近的快照的 deadlist」会在之后销毁那个快照时放掉一个克隆还在引用的块，填成「随 livelist 消耗掉」会在 `prev` 被继承到更早之后永远放不掉（C269（销毁可写头时留下的块没有去处），已还清）。
- **代价，知情接受**：本仓没有 promote，所以一个头的某个快照被克隆过、那个克隆还活着时，那个快照删不掉，这个头也就删不掉（分叉点闸的 `children` 数克隆）。这与 D6（快照实现模型） 判据 5 那句「用功能缺失回避掉的，不是满足」同形，是本仓第二条这样的限制。
- livelist 自己的 key、宽度与共享树在 D6（快照实现模型） 已定项 2 / 已定项 3。
- ZFS 的两道对应闸在本机 `fs-refs/zfs/module/zfs/dsl_destroy.c`：第 717–723 行逐字「Can't delete a head dataset if there are snapshots of it. (Except if the only snapshots are from the branch we cloned from.)」，以及 `dsl_destroy_snapshot_check_impl`：`/* Can't delete a branch point. */`——线索，未在本项目验证。

**依据**：

- E146（livelist 条目按映射 key 定身份之后的宽度与代价）：第 3 条要的那个第二结构，条目身份取中央映射 key 之后的宽度与代价已量，day-1 注册要付的那笔账也在那一页。
- 三方论证：第二道销毁闸是 D6（快照实现模型） 已定项 2 三轮对抗第二轮打出来的；用户定案，原话在变更史。

**欠**：C265（销毁可写头的释放边界写成了固定的克隆时刻）——条款已补，会红的检查仍欠；C270（分叉点闸的子计数数不数克隆）——同此。

```

**出处 `.claude/kb/decisions/05-快照-空间记账机制.md:415-438`（整段抄，未转述）**

````markdown
#### 已定项 19：生死判定的运行时形态与销毁快照的代价

**定案**：

```
判定(ptr):
    birth = ptr.birth              // 指针内联字段，零次额外 I/O
    p     = head.previous_snapshot_txg     // 活头头部一个标量，常驻内存
    n     = 稀疏旁表[映射 key(ptr)]        // 跨头共享计数，已定项 6 ③；没有条目按 1 算
    if n > 1:       计数减一，既不释放也不进 deadlist（另一个头还引用着它）
    elif birth > p: 立即释放 → 进 defer 队列（不是直接进空闲表，见 D16（发布语义） 已定项 15）
    else:           追加进 head.deadlist
```

**销毁快照的代价是 O(deadlist 条目数 + 桶数)，不是 O(全盘)**，前提是 deadlist **按 birth txg 分桶、桶边界 = 快照 txg**（已定项 12 的纪元段就是这个分桶），否则销毁时的过滤退化成全表扫描。

**射程**：归约证明——设删除发生在 checkpoint `d`，P 为此刻最新的快照。(⇐) 若 `birth ≤ P.txg`：P 已发布，所以 `P.txg < d = death`，于是 `birth ≤ P.txg < death`，存在 S 引用它。(⇒) 若存在 S 使 `birth ≤ S.txg < d`，则 S 在删除时已存在，所以 `S.txg ≤ P.txg`，于是 `birth ≤ P.txg`。∎ 它用了三件事：txg 全序、**判定必须发生在死亡那一刻**，以及跨头共享块先问稀疏旁表；第三件不破 O(1)，旁表是一次点查、key 就是中央映射 key。哪一条破了 O(1) 就没了：D6（快照实现模型） 的多可写头破第一条（处置在已定项 17），D8（核心索引结构） 的分批删除破第二条（处置在已定项 20）。归约是**证明，不是测量**，它证的是「补回前提 3 之后，判定仍然是 O(1)」。

**依据**：

- E12（攒批的顺序追加 vs 不攒批的随机页读改写）：读两个标量、零次额外设备 I/O 这一侧在三档缓存上恒 2 次设备 I/O，而每块一个引用计数的那一侧随计数器表装不装得下缓存跳一个数量级，这是这条形态相对引用计数的代价差。

**欠**：无。

````

**出处 `.claude/kb/decisions/05-快照-空间记账机制.md:439-451`（整段抄，未转述）**

```markdown
#### 已定项 20：待删除结构算活的还是死的

**定案**：待删除结构（意图 key、删除中队列）**算作活引用**。于是 `death` = 物理批次的 checkpoint，判定天然发生在死亡那一刻，**O(1) 保住，且不需要新增任何不变量**。例外：被销毁的快照从写意图那一刻起不再钉住块（已定项 12）。

**射程**：D8（核心索引结构） 已定无界删除走「写意图 → 分批 → 可续做」，判定因此发生在物理批次那一刻，而逻辑死亡早得多，已定项 19 归约的第二个前提被破，这一项是取的处置。代价是一个用户可见现象：`rm` 返回之后建的快照会**钉住**那批尚未释放的 extent，`df` 要等快照死了才回来，部分覆盖它的是 I-3.4（可用空间扣待删占用） 与 I-4.7（删除中 inode 有账） 两条不变量（checker 两条都已实现，见 `crates/singlefs-checker/invariants/05-空间记账.md` 的 I-3.4（可用空间扣待删占用） 一行与 `06-事务与崩溃一致性.md` 的 I-4.7（删除中 inode 有账） 一行的状态列）。另一条路（把待删除结构算作已死）会破坏 O(1)：正确的 deadlist 属于「逻辑死亡之后建的第一个快照」，而那个快照在写意图时还不存在，只能在批次执行时查快照列表。被销毁的可写头从哪一刻起不算活引用、分叉点闸的 `children` 在写意图时减还是做完时减，没有条款。「活引用」在已定项 14、17 里指活树引用，只读快照不是活树。

**依据**：

- 无实验：两条路的差别是「判定发生在哪一刻」这个结构性质，不是可量的代价；选哪一条由已定项 19 归约证明的第二件前提直接判出来，没有可量的量。
- 判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S9：「活引用」在已定项 14、17 与已定项 20 是两个口径，例外句改成「不再钉住块」。

**欠**：C337（被销毁的头何时不算活引用没有条款）。

```

**出处 `.claude/kb/decisions/05-快照-空间记账机制.md:452-463`（整段抄，未转述）**

```markdown
#### 已定项 21：快照列表要不要一个可枚举的持久结构

**定案**：要。盘上必须存在一个可枚举的持久结构，其元素恰为当前全部活快照（既不多也不少），不变量是 I-3.7（活快照可枚举）。

**射程**：只要求「存在且可枚举」，不定它住哪棵树、条目长什么样。扫描式 checker（D13（验证路线） 已定项 12 要求的第二个方向）要判归属需要三样：`birth` 内联在指针里（有）、deadlist 自身是可扫描的自描述树（有）、**全部根的 txg 列表**（有：树表里不带销毁中记号的码 14 条目集合）。I-3.7（活快照可枚举） 的 checker 已实现。

**依据**：

- 无实验：这是格式上的一条存在性要求，没有可量的量；它的判别力只能由 I-3.7（活快照可枚举） 的 checker 实现来证，那条 checker 已实现。

**欠**：C144（快照谱系的区间标号没有载体）——「全部根的 txg 列表」那一格的账就记在它名下。

```

**出处 `.claude/kb/decisions/08-core-index-structure.en.md:1-6`（整段抄，未转述）**

```markdown
## D8 Core Index Structure —— Settled

[中文](08-核心索引结构.md) · **English** · [日本語](08-コアインデックス構造.ja.md)

D8 (Core Index Structure) governs what the core index looks like: a single btree implementation paired with multiple independent keyspaces and two front-ends, node size and its upper bound criteria, key layout, whiteout deletion and batching of unbounded operations, inode tree shape and on-disk field table of inode records, write buffer entry `seq`, tree table entry field table and tree kind code registry, Code 2 index node payload layout and key comparison order, coupling between front-end and node message buffers, and state folding of the same key within the same publish. **Does not presuppose B-tree**, research in [prior-art.md](../prior-art.md) Section 5. Out of scope: accounting statistic carrier, key, and merge rules in D5 (Snapshot / Space Accounting Mechanism); whether keys add a snapshot dimension under multi-writable-head and snapshot implementation model in D6 (Snapshot Implementation Model); message buffer ε layout axis in D11 (Whether to Reserve Message Buffer in Index Nodes); unit header, unit type codes, and packing record unit loading discipline in D18 (Information Carried in a Block); pointer fields and width in D19 (Block Pointer Structure and Width Budget); publish semantics and checkpoint boundaries in D16 (Publish Semantics); format freeze layers in D15 (Format Freeze Policy).

```

**出处 `.claude/kb/decisions/08-core-index-structure.en.md:7-512`（整段抄，未转述）**

```markdown
### Settled Items

**D8 (Core Index Structure) settled items use a single numbering sequence**, never reordered.

| # | Item | Settlement |
|---|---|---|
| 1 | **How accounting is expressed as idempotent full value** | Accounting takes idempotent full value, avoiding incremental Δ; each root carries rows of its own generation (batch generational replacement, no multiple generations), step rows staying with root **Status: Settled.** **Progress: Implemented.** |
| 2 | **What node size takes** | 16 KiB: mkfs writes into system configuration, implementation follows constant, not computed by device characteristics **Status: Settled.** **Progress: Implemented.** |
| 3 | **Key layout** | key = `(locality_id, inode, offset)`, `locality_id` inherited from parent directory at creation, intentionally not updated on rename **Status: Settled.** **Progress: Implemented.** |
| 4 | **What bounds node size upper limit** | Upper bound determined by recovery time, not recoverability; value chosen where lookup cost is lowest under "rescan cost ≤ target line", relying on experiment **Status: Settled.** **Progress: Implemented.** |
| 5 | **Whiteout deletion** | Accepts tombstones; operations potentially modifying unbounded extents write intention first then advance in batches; first-version unlink intention resides in `deleted_inodes` (settled item 16) **Status: Settled.** **Progress: Implemented.** |
| 6 | **On-disk field table of inode records** | Inode tree leaves are Code 3 packed record units (Type 2, 233 fixed-length 140-byte records per leaf), internal nodes Code 2, entries 120 bytes, no buffer; published (inode number, object birth generation) not reused **Status: Settled.** **Progress: Implemented.** |
| 7 | **Write buffer entry inherent `seq`** | Width 4 bytes, resides in value not key, scope covers three trees: accounting / reverse index / LRU; upper bound of `T_dirty` is thus mount-time judgment **Status: Settled.** **Progress: Implemented.** |
| 8 | **Tree table entry: tree ID allocation, encoding, tree kinds, and contents beyond root pointer** | Entries total 200 bytes, tree ID 8 leading; snapshots enter tree table; tree ID watermark takes max across all root records in root ring; unknown flags bits reject entry **Status: Settled.** **Progress: Unimplemented.** |
| 9 | **Registry of tree kind codes** | Code 0 none, 1..8 are eight existing trees, 9..15 are deleted_inodes, tombstone tree, lineage tree, logged_ops, retired tree, snapshot inode root, snapshot extent root; new trees append only without code reuse **Status: Settled.** **Progress: Implemented.** |
| 10 | **Whether first-version accounting uses write buffer front-end; `seq` assignment rule** | First-version accounting directly lands on accounting tree leaves without front-end; `seq` = Nth write of same key into tree within single publish, starting at 1, constantly 1 in first version **Status: Settled.** **Progress: Implemented.** |
| 11 | **Code 2 index node payload layout, kind-to-entry-width table, key comparison order** | Code 2 header self-describing, base 86 bytes, entry area continuously arrayed with fixed width starting from header end including reserved space; key is first key_width bytes of entry **Status: Settled.** **Progress: Implemented.** |
| 12 | **Coupling between write buffer front-end and node message buffer** | Tree permitted only one level: trees using front-end have ε constantly 0, writes uniformly enter front-end, drained to leaves before publish; neither enabled in first version **Status: Settled.** **Progress: Unimplemented.** |
| 13 | **Whether index tree updates perform state folding** | When same key modified multiple times in same publish, landing on leaves occurs once, writing final state E directly; intermediate states not written into any node, E recognized by arrival order **Status: Settled.** **Progress: Implemented.** |
| 14 | **Overall index structure** | One btree implementation, multiple independent keyspaces, two front-ends (write buffer, key cache); derived allocation record tree and extent tree may adopt alternative key-space addressed structures **Status: Settled.** **Progress: Implemented.** |
| 15 | **Three hard requirements for write buffer entries** | Node cache must be scan-resistant; entries must be idempotent full values, never incremental Δ; entries must carry inherent sequence number `seq`, being a format requirement **Status: Settled.** **Progress: Unimplemented.** |
| 16 | **How first-version unlink deletes** | First-version unlink: creates `deleted_inodes` only, batched from tail by 144 units, resumes to empty during mount **Status: Settled.** **Progress: Implemented.** |

#### Settled Item 1: Accounting Expressed as Idempotent Full Values, Batch Generational Replacement

**Settlement**: **Accounting entries are uniformly "full value of a certain statistic in a certain generation"**, overwriting whole cell on write; **no incremental Δ**. Merge rules categorized by statistic (D5 (Snapshot / Space Accounting Mechanism) settled item 3 row 3, registry in D5 (Snapshot / Space Accounting Mechanism) settled item 10): non-monotonic statistics "latter wins" (latter determined by `seq`), monotonic statistics take maximum. Taking maximum is likewise idempotent, commutative, and associative; argumentation of this item (idempotent full value outperforming incremental Δ) remains intact. Batch generational replacement: each root's accounting tree holds only rows of that root's generation, keeping no multiple generations (except step rows for inode number watermark staying with root, D5 (Snapshot / Space Accounting Mechanism) settled item 2); key count does not monotonically accumulate with generation count, normally 15 rows on a two-disk pool excluding step rows.

**Scope**: Settles semantics of accounting entry values; carrier (accounting residing in an independent keyspace btree, entries being "(statistic, dimension tuple, generation) → full value") in D5 (Snapshot / Space Accounting Mechanism). ⚠️ **Key count does not monotonically accumulate with generations**: each root carries rows of its own generation only (D5 (Snapshot / Space Accounting Mechanism) settled item 2, C548 (Accounting tree batch generational replacement per publish conflicts with one row per statistic per generation)), normally 15 rows on two-disk pool excluding step rows, splitting only at 80 disks and above. ⚠️ **Backstop**: If any category of accounting truly cannot achieve idempotence (this item does not prejudge whether such category exists), that category must possess an independent recomputation path sharing **no verdict code** with runtime. ⚠️ Concurrently alters validity condition for Cost 1 of D11 (Whether to Reserve Message Buffer in Index Nodes)—criterion for that cost is "whether buffer holds idempotent full value or incremental Δ"; this item pins answer to idempotent side ⇒ **Cost 1 does not hold in this project**; Cost 2 of D11 (Whether to Reserve Message Buffer in Index Nodes) (degradation of reconstructibility) unaffected.

**Basis**:

- E14 (Discriminative Power of I-3.1: Idempotent Full Value vs Incremental Delta): Injecting four categories of semantic bugs into same tree under same workload, I-3.1 (allocated statistics match) caught all with zero blind spots under idempotent full value form, catching zero under incremental Δ form—under Δ form accounting itself is authoritative content, checker merely replaying identical Δ log with identical summation function, comparing against itself.
- User decision 2026-08-29: Adopt idempotent full value, accepting rise in key count (original wording in history).

**Owed**: C15 (Write buffer mixing in Δ).

#### Settled Item 2: Node Size 16 KiB, Pinned as Constant

**Settlement**: **Node size 16 KiB. mkfs writes 16 KiB into system configuration, implementation follows constant, not computed by device characteristics**—a constant in format, not a formula.
<!-- format-const: NODE_BYTES = 16384 -->

**Scope**: ⚠️ **Registering `format-const` does not equal freezing this number** (2026-08-31 user decision: "this number needs to be adjustable"). That marker ensures only that **kb and experiment source code write identical number**; modifying it follows ordinary process of D8 (Core Index Structure); written into system configuration, it remains an mkfs parameter. ⚠️ **It currently pins only a subset of source code**: gate stage "format constants synchronized between kb and experiment source code" (`.claude/gate.d/code-source-discipline.sh` format-constants cell) recognizes form `const NAME = integer_literal`; `16 * 1024` in `e56_epsilon.rs`, array forms in `e43_extension_point_budget.rs` and `e52_head_mechanisms.rs` unrecognized. ⚠️ **16 KiB is a choice on CPU cache axis, not device I/O axis**: E7 (Offline Index Harness) measuring device I/O showed small nodes winning on both ends with opposite directions; measuring different resources, they do not conflict. Upper bound dimension belongs to settled item 4; once recovery time half of E1 (Node Size vs Recoverability) yields upper bound tighter than 16 KiB, this item reopens.

**Basis**:

- E20 (CPU Cache Cost of Widening Pointers): On sole working set exceeding L3, 16 KiB is minimum point; in same experiment 40-byte pointer header pays far higher relative cost on 4 KiB node than 16 KiB node, large nodes absorbing pointer widening; on same curve 8 KiB anomaly stable across six rounds and unexplained ⇒ before explaining it, writing node size as "computed by device characteristics" is forbidden.
- E6 (Encryption Algorithm Selection): Gains of encryption / checksum units essentially saturate after 16 KiB.
- E73 (Fanout Cost of Node Key Ranges): Tree height did not increase by single level after Code 2 header carried key ranges; 16 KiB needs no reopening.
- User decision 2026-08-30: Pin 16 KiB as constant (original wording in history).

**Owed**: C56 (Unregistered format constants still drift), C386 (Node size conclusion owes rerun under final pointer width, unnumbered).

#### Settled Item 3: Key Layout —— Decoupling Rename from Indexing

**Settlement**: **key = `(locality_id, inode, offset)`**, `locality_id` inherited from parent directory at creation, **intentionally not updated on rename**. offset segment is **file byte offset** (offset of first byte of this unit in file; Nth unit starting from 0 is n × payload capacity 32634).

- Naming follows dirent tree (`parent, name` → inode); renaming deletes one and adds one, **O(1)**.
- Data keyed by inode, completely decoupled from name.
- Scan locality derives from inherited `locality_id`; after rename **locality slowly degrades while correctness is unaffected**.
- Degradation repaired by resident background compaction promised in D3 (Space Allocation), not an added burden.

**Batch rename**: `mv large_dir somewhere_else` under this design **modifies only one dirent**. Three situations would turn it back into unbounded operation, which must be blocked from day one:

1. **`locality_id` must never be updated by rename.** It is **hint, not part of correctness**—can be erroneous, can be stale.
2. **Rename overwriting existing target = one deletion**, which must proceed through intent log of settled item 5 (whiteout deletion).
3. **Background reclustering must be incremental, interruptible, and resumable**, stoppable at any time.

**Corollary (usable as review criterion)**: **Any design change causing rename to touch descendants is rejected unconditionally.**

**Inode record must store a copy of `locality_id`**: It is both head segment of data key and never updated by rename; naming tree value holds inode only, and unit header 5-tuple lacks it; thus on disk only key itself carries it—readers opening a renamed file must retrieve it somewhere, otherwise data after rename could only be discovered by scan. Placement is settled item 6 (on-disk field table of inode records); this value inherited from parent directory per this item; first version lacking directories, "what to take when no parent directory" is settled simultaneously—first version takes 0, registered in record field table of settled item 6. During scan rebuild, head segment of reconstructed keys is assigned uniformly for all keys of this object following inode record > surviving extent key > 0 (2026-09-16 user decision; field addition proposal of D18 (Information Carried in a Block) settled item 3 rejected).

**Scope**: Governs data key encoding and rename avoiding descendants; does not govern naming tree keys (dirent tree keyed by `(parent_inode, name)`, see settled item 5). Scan locality need not reside in key order: can reside in key order (full path indexing, rename O(subtree depth)), in allocator (placing subdirectory data near parent directory), or in secondary index (still expensive, requiring extra consistency maintenance); this project adopts "locality prefix inherited at birth and unupdated on rename" (better, free renames). ⚠️ "It is a hint and can be wrong" authorizes **disbelieving its semantics**, not **omitting its storage**: as address head segment its value is exact and definitional. ⚠️ **Traversal locality figures of E9 (Impact of Key Encoding on Traversal Locality) hold under single measurement basis only, citations must include measurement basis**: **interleaved creation** (`interleave`, proxy for aging) **plus `cache_leaves = 8`**; gains of `locality_id` are **new-tree gains**, not steady-state gains; realism of rename intensity uncalibrated—curve usable, absolute scale unusable. ⚠️ Traversal advantages of full path indexing also measured, but **using it to substitute judgment on D9 (Encryption) leakage surface and D1 (Data Mobility / Reverse Index) stability is forbidden**—E9 (Impact of Key Encoding on Traversal Locality) says zero words on those. ⚠️ "Average ~2700 keys rewritten per rename" **lacks measurement basis** (untraceable which rename tier, which seed), marked unverified; ext4 Orlov allocator "traversing kernel tree 30% faster" in same table likewise unverified—lacking source and measurement basis in kb, must be supplemented before use. ⚠️ 3-segment key was once incomplete after D6 (Snapshot Implementation Model) adopted multi-writable-head (two writable heads each writing same `(locality_id, inode, offset)` would land in same key slot, later overwriting earlier); resolved on 2026-08-30 when D6 (Snapshot Implementation Model) settled item 1 adopted "one tree per head"—with one tree per head, 3-segment key is unique within tree, **two heads no longer landing in same key slot**. ⚠️ **Nonce obligation remains**: D9 (Encryption) settled item 4 settled nonce stored per extent; **generation must still guarantee no collision across two writable heads** (candidate form partitioning nonce space by head); no longer obtained free from identity uniqueness, otherwise I-6.1 (nonce not reused) shifts from "guaranteed by identity uniqueness" to "guaranteed by generator" with no one guaranteeing it; obligation is corollary, unverified by three-party review. ⚠️ Whether background reclustering moves physical location only or re-encodes keys, how batched during re-encoding, and where progress resumes from are undefined across repository.

**Basis**:

- E9 (Impact of Key Encoding on Traversal Locality): Comparing four arms on same directory tree and same LRU, `locality_id` prefix issues fewer device reads than keying by inode, worthwhile; during depth-first creation three real arms match bit-for-bit; when cache holds whole tree all four arms equalize and effect vanishes; gains decay to near zero with rename frequency.
- Mechanistic argument (written at settlement on 2026-08-25, original text in history): Full path indexing in this project pays three additional costs BetrFS avoids—separator key being path expands sensitive information identified in D9 (Encryption) settled item 5 from one tree to all path-keyed trees; referencer identity being path invalidates all reverse index entries of D1 (Data Mobility / Reverse Index) upon rename; lifting requirements on key encoding contest same key space with D9 (Encryption) nonce carrier and D6 (Snapshot Implementation Model) snapshot ID placement.
- User decision 2026-09-16: Scan rebuild assigns identical head segment to all extent keys of an object (original wording in history).
- E157 (Counting Models for Two Parallel Line Clauses) measured: node reads locating a page, tree height and leaf counts, byte range restoration upon capacity shift—both arms equal point-by-point across all sample points; sole differentiator is AAD anchor cell: under file byte offset N keys compute correctly, under unit sequence number only 1 computes correctly.
- D9 (Encryption) settled item 6 verbatim reads "anchor offset = in-object start offset when extent first written, reader computes `key.offset − ptr.extent_off` on the fly", while settled item 12 clause 2 of same decision specifies AAD field composition as day-1 permanent contract ⇒ clause inherently presupposed this unit.

**Owed**: C346 (Background reclustering undefined).

#### Settled Item 4: Node Size —— Upper Bound Determined by Recovery Time, Value Set by Experiment

**Settlement**: Large nodes save seeks and yield shallower trees, but **rescan cost scales linearly with node size**, hence **not copying 128K–256K**. **Node size trades "recovery time", not "data safety"; target line should be set on recovery time. Value determined by E1 (Node Size vs Recoverability): choosing lowest lookup cost tier under prerequisite "rescan cost ≤ target line".** Three constraints:

1. **Format parameter at mkfs, not mount parameter.** Automatic selection occurs once at mkfs. Changing it = **rebuilding index**, not rebuilding file system (D21 (Boundary Between Authoritative and Derived States) settled index is derived state, Layer 2 freeze downgraded to local cache format).
2. **Default policy must be deterministic and auditable**, and mkfs must output "why this value was chosen".
3. **Tests pin "simulated device parameters", not node size itself**—fixing a set of device tiers allowing automatic rules to run normally, **testing selection rule itself**. Sole requirement: identical tier must produce identical choice.

**Scope**: ⚠️ **Criterion is not "full disk scan rebuild success rate ≥ target line"**—permanent loss from corrupt index constantly 0 ⇒ success rate dimension **unrelated** to node size; using it as criterion clamps no upper bound. ⚠️ **Target line itself remains unsettled**; it is a recovery time budget (how quickly destroyed portion must be rescanned back), uncomputable by this item alone; remaining half of E1 (Node Size vs Recoverability) owes mkfs, checker, and a tool定点-corrupting images. ⚠️ Prerequisite: index reconstructible from unit self-description—E28 (Whether Plaintext Mapping Layer Can Be Rebuilt Solely from Unit Self-Description) measured **holding on metadata side, failing on data block side** (deficit exactly equal to total data blocks, owing D18 (Information Carried in a Block) settled item 3); ⚠️ **"Changing it = rebuilding index" revokes qualitative severity, not cost**: rebuilding entire tree to new size is full-scale whole-disk I/O, merely without data loss; furthermore revocation has unfulfilled prerequisite—before data block self-description is patched, "equivalent to rebuilding file system" still holds on data block side. Bridging basis is index tree rebuild and plaintext mapping layer rebuild **read identical input** (each unit carrying "who I am, who I belong to"); ⚠️ this step is deductive, E28 (Whether Plaintext Mapping Layer Can Be Rebuilt Solely from Unit Self-Description) having tested plaintext mapping layer instance only; this half-sentence represents entire rationale for D11 (Whether to Reserve Message Buffer in Index Nodes) being "unpostponable". ⚠️ **First phase includes byte-dominant end in scope only**: D12 (Target Media) settled item 5 settled first layout line as pure SSD ⇒ conclusion holds, **support shifting from two ends to one end**; seek-dominant end re-enters scope once HDD layout line opens. ⚠️ **Automatic node size selection criterion must include pointer width** (E20 (CPU Cache Cost of Widening Pointers): large nodes absorb pointer widening, direction opposite to rescan cost), absent prior to E20 (CPU Cache Cost of Widening Pointers). ⚠️ **"Opposite directions by device characteristics" rationale does not hold**: both optimal directions identical, both favoring smaller; node size remains mkfs parameter, retaining media-dependent adjustment requires separate basis.

**Basis**:

- E29 (Blast Radius of Corrupting Single Node): Permanent loss from corrupted index constantly 0 (D21 (Boundary Between Authoritative and Derived States) settled index is derived state, loss merely induces slowdown); loss from corrupted data unit unrelated to node size; only rescan cost scales linearly with node size (rescan volume of Level 1 node exactly equals fanout)—hence upper bound can be set on recovery time only, "blast radius" terminology corrected by it.
- E7 (Offline Index Harness): **Under fixed memory budget large nodes are inferior across all three axes** (measuring io/op, bytes/op, point lookup reads/op); mechanism is larger nodes permit fewer nodes in same memory, hit rate dropping faster than gained by "reading more each time"; seek-dominant end points same direction, large nodes remaining inferior ⇒ "not copying 128K–256K" supported on both byte-dominant and seek-dominant ends.
- User decision 2026-09-06: Measurement basis rewritten to "upper bound determined by recovery time" (original wording in history).

**Owed**: C386 (Node size conclusion owes rerun under final pointer width, unnumbered).

#### Settled Item 5: Whiteout Deletion —— Accepted, but Batch Deletions Must Follow Intent Log

**Settlement**: Deleting large directories or truncating large files generates massive tombstones, inevitably blowing up single transaction. **Approach (bcachefs verified shape)**: Operations **potentially modifying unbounded extents** (truncate, collapse, deleting large directories) **write intention into logged_ops tree first, then advance in batches**; recovery post-crash resumes incomplete intentions. Deleting inodes enter `deleted_inodes` tree separately, reclaimed by background—user `rm` returning does not mean all extents released. First-version unlink writes zero entries to logged_ops tree: intention is inode record nlink = 0 plus `deleted_inodes` entry (settled item 16). logged_ops tree registered day-1 with zero root alongside `deleted_inodes` during first file version; field table settled (the compaction intention entry field table of D26 (Background Compaction and Placement Reclamation) settled item 2), while which version taken on rollback and state classifications are deferred to landing of first writer (compaction) (C364 (compaction intent record fields lack field-by-field reconciliation)). Intentions divided into two classes: **committed class** (unlink; truncate, delete large directory) must resume to completion, draining to empty before writable mount returns; **hint class** (compaction) can pause, terminate early under stop predicate, unresumed on disk-side line, not blocking mount; resumption entry points segregated by tree, sharing publish path. Settled item 5 clause 2, three-state table, and C1 (Intention resumption made regular path) literally written for committed class, separated by class upon compaction landing (C621 (Deletion clauses touched when compaction lands lack out-of-list checks)).

**Hard requirements**: Unbounded operations forbidden from single transactions; intentions must be **idempotent**; whiteouts cleared only when node rewriting merges bsets, **no other clearing paths permitted**.

Intention itself is an ordinary key riding on same transaction layer, possessing three states post-power-loss:

| Power Loss Timing | Observed Post-Recovery | Resolution |
|---|---|---|
| Before intention commit | No intention | Operation never occurred, user side received no success |
| Intention committed, work half done | Intention present | **Resumed** on recovery, completed portion re-run idempotently |
| Work completed, intention undeleted | Intention present | Resumes once finding nothing to do, then deletes intention |

Impossible to observe "intention gone but target state unattained"—**deleting intention must commit in same transaction as final batch of work**.

1. **Intentions declare target state, never delta.** "Truncate file X to N bytes" can be re-executed; "delete 1000 more extents" cannot.
2. **Resumption is regular action run on every recovery, not repair tool** (otherwise creating code paths "unrun in normal times, run first time during disaster").
3. **Every intermediate state must be legal file system state**—"file with half extents deleted" must itself be legal.
4. **Each action in batch verifies target presence before applying** (concurring with D5 (Snapshot / Space Accounting Mechanism) settled item 12)—literal idempotence: planning batch per intention based on start-of-batch state without re-verification upon application causes overlapping cascade intervals to free same block twice, decrementing statistics twice and turning I-3.1 (allocated statistics match) red.

**Range rebuild vs itemized update: both must exist with explicit criteria** (`.claude/rules/fs-design.md` "do not pursue panaceas"):

| Approach | Cost | Applicability |
|---|---|---|
| Itemized update | K × random access | Affected keys sparsely scattered |
| Range rebuild | N × sequential access | Affected keys contiguous in key order |

**Real switch is not K/N, but "whether affected keys are contiguous"**: Contiguous ⇒ range rebuild can discard or rewrite whole nodes; non-contiguous ⇒ itemized only. **Yielding layout rule: place things dying together contiguously in key order.**

- dirent keyed by `(parent_inode, name)` → deleting whole directory is range operation
- extent keyed by `(inode, offset)` → truncation is range operation
- keys of same snapshot carry identical snapshot prefix → deleting snapshot can become range operation

**This structurally eliminates whiteout bloat**: range deletions discard whole nodes directly without inserting N tombstones.

**Scope**: Naive crossover point is `K/N > sequential_cost / random_cost` (~1% HDD, ~10% SSD). ⚠️ These numbers are **deductive, not measured**, referring to **latency**, requiring measurement via E4 (Range Rebuild vs Itemized Update Crossover Point) with QEMU. ⚠️ **First phase takes SSD tier only** (D12 (Target Media) settled item 5: first layout line is pure SSD). ⚠️ Write amplification crossover point measured layout and node size as independent variables rather than media, unable to disprove "~1% HDD, ~10% SSD" latency statement. Per five requirements of `.claude/rules/fs-design.md`, this branch still owes: computable formula for criterion, test switch forcing specified branch, crash-point replay run on both branches, runtime visibility of taken path. ⚠️ **Switching criterion forbidden from using quantities shifting mid-operation such as "current fill rate"**—criteria use inputs stable throughout operation. **Interaction with D3 (Space Allocation) (easily overlooked)**: space not yet truly released while intention incomplete; space accounting must tally "pending deletion incomplete", otherwise "deleted massive data yet `df` unmoved" manifests.

**Basis**:

- E30 (Write Amplification Half of Crossover Point): **Write amplification half measured**—measuring crossover point between contiguous and scattered entry layouts across 4 KiB / 16 KiB nodes under write amplification basis, differing by over two orders of magnitude—switch is layout and node size, not media.
- User decision 2026-09-16: Add 4th hard requirement "verify target presence before applying batch" (original wording in history).
- Three-party `research/prompts/m3-intent-r1-main-verification.md`: Intentions classified into committed and hint classes (derived independently by two legs, one round); logged_ops tree zero-root registration is user decision K-I1 (original wording in history; attacked zero rounds).

**Owed**: C2 (Unbounded operations made single transactions).

#### Settled Item 6: Inode Tree Leaves Are Code 3 Packed Record Units, Record Fixed Length 140, Published (Inode Number, Object Birth Generation) Not Reused
<!-- doc-lint:not-numbers P1 P2 P3 P4 P5 P6 P7 P8 P9 -->

**Settlement**: Inode tree = Code 2 internal nodes (derived state, no message buffer, entries 120 bytes, fanout 135) + Code 3 packed record units as leaves (packed record Type 2, authoritative unit, 233 140-byte records per leaf); root constantly Code 2; container index is internal nodes of this tree; published (inode number, object birth generation) not reused; next number is a statistic merged by maximum in accounting. Code 2 root grows a level when full—new root 1 Code 2, split right half 1 Code 2 (right half has no container identity); inode tree and tombstone tree (D18 (Information Carried in a Block) settled item 24) share this rule; growing publish adds 2 16-KiB nodes × replica count. Before root grows, code in `crates/singlefs-core/src/inode_tree.rs` rejects via `MoreLeafContainersThanOneRootNodeHolds` (deduced units, zero rounds).

| Component | What It Is | State Tier |
|---|---|---|
| Leaf | One 32 KiB Code 3 packed record unit (D18 (Information Carried in a Block) settled item 11), packed record Type 2, housing 233 140-byte records; carries class identity segment, record count, record width, payload checksum; payload integrity evidence on scan path (C104 (Payload integrity of Code 1 / 2 on scan path lacks self-contained evidence) hole on Code 2) absent for it | Authoritative state (unit) |
| Internal Node | Code 2 index node. Header carries key range (min / max 8 bytes each; D18 (Information Carried in a Block) settled item 2 slot for inode tree settles "carried": identical rationale to accounting tree—single node picked up self-proves covered range; key is inode number not filename, sensitivity argument out of reach). **Header is block header, falling into D15 (Format Freeze Policy) Layer 1 (frozen), same slot as accounting tree node header (C102 (Index node header lacks reserved area, adding fields is not compat)); payload layout (entry 120, ε = 0) falls into Layer 4, non-permanent** | Derived state, rebuilt from leaves upon loss |
| Root | Constantly Code 2 index node, even when tree has single leaf or tree is empty (zero entries, key range written as [0, 0], 0 being invalid inode number). Rationale: D19 (Block Pointer Structure and Width Budget) pointers lack class and size of referenced object; if root pointed by tree table altered class as tree grew, readers reading 32 KiB per 16 KiB experience silent misread | Derived state |
| Container Index | Internal nodes of this tree, not separate tree: physical pointers reside in internal nodes only, dirent tree referencing inode numbers. Updating a record = copy-on-write leaf + all ancestors (D23 (Role and Format of Journal) settled item 1 A) | — |

**Internal node entry = separator key 8 + identity reference 26 + child pointer 86 = 120 bytes** (fanout calculated by header 131 as ⌊(16384 − 131) / 120⌋ = 135). Identity reference is 26 bytes allocated in D18 (Information Carried in a Block) settled item 11 for "other trees referencing container": birth tree 8 + packed record type 2 + container number 8 + container birth generation 8. **Type segment doubles as "what child node is"**: 0 ⇒ child node is Code 2 index node (container number, birth generation segments constantly 0; 0 in registry remains invalid in container header, distinguished by hosting structure), 2 ⇒ child node is Code 3 container (reading two 16-KiB slots); other values judge entry corrupted. Legal sets partitioned by tree: inode tree {0, 2} (I-9.2 (Entry identity matches child header)), tombstone tree {0, 1} (I-9.23 (Tombstone tree entry identity matches child header)), lineage tree {0, 5} (I-9.17 (Lineage tree entry identity matches child header)), retired tree {0, 3} (invariant established alongside batch 50 and retired tree implementation, verdict `research/prompts/m3-retire-r1-main-verification.md` Section 3 clause 8), internal entries of four trees sharing identical shape. Reader thereby knows how many slots to read; AAD five segments of Code 3 edge (tag deduced from type ≠ 0, remaining four segments copied from entry) originate entirely from lookup path, D9 (Encryption) A6 (AAD expected values do not come from plaintext side) requiring no tree kind on this edge; Code 2 edge (internal node pointing to internal node) AAD remains C107 (Index node AAD identity lacks placement). Birth tree carried per entry because clone heads (D6 (Snapshot Implementation Model) settled item 1) share origin leaves, single internal node mixing entries with distinct birth trees; after D19 (Block Pointer Structure and Width Budget) settled item 7 added birth tree to child pointer header, birth tree in identity reference **remains retained**—both coexist, occupying space regardless of encryption, not repurposed; birth tree in identity reference of type segment 0 entries constantly 0, shaped like container number and birth generation segments; Code 3 edge AAD birth tree takes identity reference copy, not child pointer copy. **Separator key** = minimum key when child created; on publish where left borrows from right, modified to minimum key of borrowed right child's current version; entries monotonically ascend by separator key; lookup takes last entry with separator key ≤ target key.
<!-- format-const: INODE_INTERNAL_ENTRY = 120 -->

**Internal nodes reserve no message buffer, inode tree ε = 0**—tree axis supplement to D11 (Whether to Reserve Message Buffer in Index Nodes) settled item 6 (layout axis "all layouts share identical ε" untouched). Branch variable is predicate in D11 (Whether to Reserve Message Buffer in Index Nodes) settled item 4: **whether pending flush messages of this tree can be recomputed solely from authoritative state (units (with write sequence) + instance table + ancestor table + tombstone units)**. If yes ⇒ reserve buffer (ε = 0.65); if no ⇒ do not reserve. Inode tree cannot: attribute updates (new values of mtime / nlink / mode) lack authoritative carrier before reaching leaves; buffering into derived nodes violates D21 (Boundary Between Authoritative and Derived States) Hard Constraint 6. extent / dirent / reverse index can (three categories all 0 in "unit + accounting" row of E59 (Can Buffered Messages Be Recomputed Solely from Units)); accounting tree judged capable per D21 (Boundary Between Authoritative and Derived States) "recomputable offline by traversal", but column in E59 (Can Buffered Messages Be Recomputed Solely from Units) is oracle, unverified. Falls into D15 (Format Freeze Policy) Layer 4 (derived state), not format branch. **Tier judged by recomputability, not word "tree"**: internal nodes recomputable from leaves ⇒ derived state; leaves housing uncomputable values ⇒ authoritative state. Accounting is settled instance of same criterion, authoritative recording point in D21 (Boundary Between Authoritative and Derived States).

**Loading discipline classified by record type**: "One container per generation" and "close open containers before snapshot publish" hold only for packed record Type 1 (tombstones) reclaimed by generation; Type 2 records die individually, containers reclaimed by B-tree split / merge, single leaf surviving many generations and rewritten repeatedly, snapshots merely sharing it. **Death divided into three layers**: (1) Death of Type 2 records writes no tombstone, death reflected by active leaf version, active version selection resting on C113 (no input for active version judgment of multi-version units during scan rebuild); (2) Leaf container identity disappearance (emptied and unlinked, right half absorbed during merge, old identity upon cross-head first COW rebirth) writes retired record: packed record Type 3, carrying death txg, housed in dedicated retired tree (D18 (Information Carried in a Block) settled item 11); retired record writers append only: multiple entries across administrator rollbacks permitted for same identity, without deduplication or overwrite; death txg = checkpoint_txg of publish where identity disappeared; readers and checkers recognize retired records on this version lineage only; (3) Whole-object deletion reflected by whole-object tombstone written during unlink intention, effective from intention publish (D18 (Information Carried in a Block) settled item 24 clause 2, settled item 16 clause 1). File shortening (overwrite into fewer units, truncation) writes no new records, data unit liveness judged during scan rebuild by reconstructed inode record ("Rebuild" section).

**Container identity requires no allocator**: Inode numbers monotonic on each timeline (administrator rollback new root takes R_old version inode tree, watermark taking max(active version in-memory, readable from ring and not discarded by crash recovery per instance table); crash recovery returns to selected root tree and watermark; subsequent issued numbers exceed every key in tree) ⇒ insertion always lands on rightmost leaf ⇒ splitting occurs on rightmost leaf only, **split point taken at end**—left half retains all original records and identity; right half starts from new record triggering split, **right half container number = inode number of that new record**, birth generation = checkpoint_txg of current publish, **birth tree = tree executing this split** (split is new creation, distinct from "COW rewrite identity unchanged"); first container number = inode number of first record. Internal nodes likewise overflow at right end, splitting at end, right half being Code 2 with no container identity. Merging: **both left-absorbs-right and left-borrows-from-right implemented** (right-absorbs-left or right-borrows-from-left breaks "record number ≥ container number"), retaining left half identity; absorbed right half follows COW release path, writing a retired record for its identity in same publish; when left borrows from right, borrowed count evenly split (difference between two pieces post-borrow ≤ 1), right piece retaining identity without writing retired record, its separator key updated to minimum key of its current version; merge timing is write-path policy (unit test / model differential), excluded from format. **Type 2 containers with record count 0 unpersisted to disk** (must be merged before publish; when sole leaf emptied, unpersisted, root becoming zero entries); emptied and unlinked containers write retired record for identity in same publish. **Rebirth upon cross-head first COW** (C113 (no input for active version judgment of multi-version units during scan rebuild) Settlement P6): new identity = (this head tree ID, Type 2, original container number, current publish generation)—retains container number, swapping birth tree and birth generation, writing retired record for old identity. Rebirth applies to Type 2 only: Type 1 containers owned exclusively by writer tree, clone excluding them from container index (D18 (Information Carried in a Block) settled item 11), never undergoing cross-head COW. Two distinct assertions coexist: "container identity unchanged when COW rewritten" (content same, position changed) and "left half identity unchanged post-split" (content reduced, identity unchanged). Type 1 container numbers still monotonically allocated per C103 (Container index lacks placement), two rules classified by record type.

**Key and inode number**: key = inode number 8 bytes, uniqueness scope (tree ID, inode), bit width 64, declared lifecycle total creations upper bound 2^63. **Published (inode number, object birth generation) not reused** (precept of D6 (Snapshot Implementation Model) Criterion 9): reuse within `deleted_inodes` background reclamation window causes extent key 3-segments of new and old objects to match verbatim. ⚠️ **The word "published" is not phrasing alignment, but self-consistency of current design**: watermark resides in accounting (D5 (Snapshot / Space Accounting Mechanism) settled item 4 Item 12); accounting of abandoned checkpoint discarded with it ⇒ **every ordinary crash** causes inode numbers issued in that generation to be reissued. Administrator rollback is forward publish while mounted, inode number watermark of that publish taking max(active version in-memory, readable from ring and not discarded by crash recovery per instance table), issued numbers un-reissued (D23 (Role and Format of Journal) settled item 14); when crash recovery lands on older root, watermark loads with accounting of selected root, numbers issued in abandoned timeline reissued, while post-recovery new object birth generations all exceed txg of every root and record readable from ring (D23 (Role and Format of Journal) settled item 14 Note 3)—how many faults required to collide with birth generation of published object in abandoned timeline unmeasured in crash recovery form, see C143 (Inode number watermark rewinding to re-issue numbers after crash recovery falls back to old root); externally exposed object identity = (inode number, object birth generation); structures recognizing objects by inode number must carry birth generation or roll back with root. Extent key 3-segments matching verbatim in `deleted_inodes` reclamation window refers to reuse on same timeline (old object published, extents awaiting background reclamation in tree), remaining forbidden; after crash recovery reissuing same number, new object extents land in extent tree of selected root, old object extents absent from tree alongside abandoned root—**relying on extent tree and `deleted_inodes` both following selected root**; administrator rollback publish new root takes R_old inode tree and extent tree, `deleted_inodes` likewise taking R_old version (otherwise objects deleted after R_old returning with R_old inode tree reclaimed again by background; main agent settled 2026-09-26, deduced, attacked zero rounds); `deleted_inodes` shape (settled item 16) honors these two; on disk both separated by object birth generation in unit 5-tuple (D9 (Encryption) settled item 6). **Inode records remain in inode tree before reclamation completes (nlink = 0), `deleted_inodes` storing numbers only**.

**Next inode number is a statistic in accounting** (D5 (Snapshot / Space Accounting Mechanism) settled item 4 Item 12 "inode number watermark"): key = (statistic, tree ID, device segment, generation), without device dimension, value = next available number, **merge rule takes maximum** (idempotent, commutative, associative); entry carries seq (settled item 15 clause 3 is format requirement, one entry format per write buffer tree), merely **merge result does not depend on it**. Selected over "latter wins" due to txg reissue post-crash: when larger and smaller values appear in same generation segment, maximum yields larger (wastes numbers, avoids reusing numbers); "uncommitted equals unhappened" holds for regular mount path only, scan rebuild inspecting disk. "Tag → merge rule" is on-disk semantics, statistics registry established as independent frozen component (D15 (Format Freeze Policy)). Watermark rows maintained for writable heads only, written per step-row rules: publishes modifying watermark leave a step row, COW-persisting with root indefinitely (D5 (Snapshot / Space Accounting Mechanism) settled item 2 granting exception to rule that each root's accounting tree holds rows of its own generation rewritten each publish; see D5 (Snapshot / Space Accounting Mechanism) settled item 2 scope clause "step rows for inode number watermark"); read-only trees lack this row; reading active value = taking row with maximum generation under dimension tuple; empty ⇒ never maintained, not 0. Clone heads: new head watermark = runtime watermark at origin moment (including unpersisted in write buffer); clone creation publish must write this watermark row for new tree; two heads allocating independently subsequently may intersect numbers, uniqueness relying on (tree ID, inode).

**Record field table (Fixed length 140, little-endian; alignment basis is in-record offset; Nth record in container starts at 136 + 140n (Code 3 header 107 + nonce / MAC / algorithm type reserved space 29, D18 (Information Carried in a Block) settled item 16) unaligned to 8, read unaligned)**
<!-- format-const: INODE_RECORD_BYTES = 140 -->
<!-- format-const: INODE_LEAF_RECORDS = 233 -->

| Offset | Field | Width |
|---|---|---|
| 0 | inode | 8 |
| 8 | Object birth generation (checkpoint_txg of creation publish; retains day-1 contract of 5-tuple / AAD after inode numbers un-reused without discriminating reuse, matched field-by-field against each data unit header during scan) | 8 |
| 16 | locality_id (0 in first version without parent directory) | 8 |
| 24 | mode / uid / gid / nlink, 4 each | 16 |
| 40 | size / blocks / rdev, 8 each | 24 |
| 64 | atime / mtime / ctime seconds, 8 each (three fields share single caller-supplied value for unlink intention, copied by each batch) | 24 |
| 88 | Modification count (checkpoint_txg of publish containing last modification; commit anchor = birth generation of leaf container header; multiple modifications in same window land only final version on disk—internal nodes unbuffered; unlink intention and batches count as modifications writing current publish checkpoint_txg, not copying previous version; creation counts as modification, modification count not lower than object birth generation at offset 8, I-9.26 (Inode Modification Count Between Birth Generation and Container Birth Generation)) | 8 |
| 96 | atime / mtime / ctime nanoseconds, 4 each | 12 |
| 108 | Padding (constantly 0) | 4 |
| 112 | flags (constantly 0) | 8 |
| 120 | Reserved (constantly 0) | 20 |

**Write time (three second fields starting offset 64, three nanosecond fields starting offset 96) takes parameters of publish writing this record, not system clock**: Each publish invocation carries write time, identical parameters across two publishes matching byte-for-byte (reproducible). All three second fields write parameter seconds; all three nanosecond fields write parameter nanoseconds; parameters carrying seconds only, writing 0. **Creation** (new-pool file creation, creating inode) writes creation publish parameters; **content modification** (overwrite, sequential write) rewriting record writes current publish parameters, not copying previous version; **unlink intention** writes caller-supplied time for this unlink, three second fields (atime, mtime, ctime) sharing single value, subsequent batches copying value written at intention; **publishes not modifying this record**—other records modified in same leaf, this record rewritten incidentally, rollbacks—copy previous version value of this record.

**`blocks` is 512-byte blocks of logical size**: `blocks` = ⌈`size` ÷ 512⌉, rounded up by file length, **does not represent allocated space**, not calculated by data units allocated to object. New-pool file creation record 3000 bytes ⇒ 6; newly created inode with unwritten data ⇒ 0.

Omits extent pointers (D14 (Dual-Track Large and Small Files / Persistent and Temporary) settled item 3 inline threshold 0), btime excluded from record. **flags / padding / reserved three segments share identical policy: constantly 0, readers encountering non-zero ⇒ record EIO (object-level, not rejecting whole container)**—forward compatibility policy is not corruption detection (payload checksum still passes), instantiated as I-9.7 (Record three segments constantly zero), outside abort chain of I-1.7 (Packing container legality and evaluation order). Assigning semantics to any bit / byte in future judged incompat per D15 (Format Freeze Policy) "altering meaning of existing field", retired bytes permanently scrapped. **No compat extension path inside record**: inode attributes have semantic dependencies, unreachable by first row of D15 (Format Freeze Policy) verdict table, records rewritten repeatedly causing older readers to drop unrecognized content. Reserved 20 non-load-bearing: any assignment inside record is incompat, width merely deciding whether record width modified together. **Overflow path judged per attribute**: attributes without semantic dependencies take new keyspace (D15 (Format Freeze Policy) verdict table granting compat), dependent ones taking flags bits / reserved (incompat); attribute keyspace lacks placement (xattr postponed). **Record width must equal width assigned to Type 2 in registry**, inequality judged corrupt (inserted after "class identity segment" and before I-1.2 (Block header write sequence published) in I-1.7 (Packing container legality and evaluation order), requiring header fields only); width modification is incompat. Record format, Code 3 class identity segment usage for Type 2, inode tree key encoding are permanent bytes: **established as independent frozen component** (C9 (Stealthily modifying format post-freeze) tracked by component), freeze moment being first image writing Type 2 container; D15 (Format Freeze Policy) four layers untouched; Type 1 record format (width 40, D18 (Information Carried in a Block) settled item 24) independent.

**Invariants**: I-9 (inode tree structure) category, fourteen invariants, see [invariants/09-inode树结构.md](../../../crates/singlefs-checker/invariants/09-inode树结构.md); status column uniformly carries prerequisite "tree kind identifiable (settled item 8)", deducible on first version image from birth tree in Type 2 container header. nlink == dirent entry count not established here—gate (dirent tree existence) lacks input, established when dirent tree goes live. Sole file in first version takes inode 1 (0 invalid), nlink = 1, excluded from orphan reclamation. Write path disciplines beyond split / merge identity propagation and merge direction are operational assertions, verified via unit test / model differential and crash-point replay.

**Rebuild**: When internal nodes lost, collect packed record Type 2 containers, group by (birth tree, container number, container birth generation), active version selection among multi-versions of same identity = C113 (no input for active version judgment of multi-version units during scan rebuild); identities identified by retired records in retired tree excluded; retired records carry death txg, filtered by lineage (same interpretation as I-9.18 (Numbers in deletion queue have tombstones on lineage) for tombstones); trees following active state (retired tree, tombstone tree, lineage tree) select version solely per C113 (no input for active version judgment of multi-version units during scan rebuild) Settlement P2; per-leaf key range computed from records, internal node key ranges computed from leaves; artifact lands in D18 (Information Carried in a Block) settled item 5 Level 1 (read-only mount), Level 2 witness nonexistent. **Extent tree rebuild**: objects recognized by (tree ID, inode), inspecting this head's extent tree only; all extent keys of this object (surviving and recovered) encoded with identical head segment, value assigned as inode record > surviving extent key > 0; when record intact, reconstructed view matches record value; misassignment loses locality only. **Data unit liveness**: judged by reconstructed inode record object birth generation and unit count derived from size, file shortening writing no new records (Interpretation Z). I-9.9 (Locality three-value cross-check) judged by side post-rebuild: outputs "undecidable" if one side encoded from other.

**New-pool file creation writes**: one Code 2 root node (header carries key range [1, 1]; one entry: key 1, Type 2, birth tree, container number 1, birth generation 3, pointer); one Code 3 leaf (Type 2, container number 1, birth generation 3, record count 1, record width 140); one record (inode 1, birth generation 3, locality 0, nlink 1, modification count 3, three segments 0); one accounting row "inode number watermark = 2".

**Scope**: **Inode numbers not reused on same timeline**—"timeline" denotes ancestor chain of selected root: includes segment discarded by administrator rollback (rollback publish watermark taking max, issued numbers un-reissued), excludes segment discarded by crash recovery (post-crash landing on older root, issued numbers in discarded segment reissuable). **Uniqueness scope is within single published history**—each container number is inode number upon initial insertion of some record; numbers reissuable post-crash landing on older root (administrator rollback publish watermark taking max, un-reissued, D23 (Role and Format of Journal) settled item 14), hence two containers in same tree may share number, separated by container birth generation (right half birth generation = txg of publish triggering split, while new instance txg post-recovery exceeds every root and record txg readable from ring, D23 (Role and Format of Journal) settled item 14 Note 3); "split → merge → split again" within single publish cannot produce two versions of same identity. Cross-crash does not hold: checkpoint numbers reissued post-crash, discarded and retried versions matching 4-segments verbatim with distinct content, both present on disk—same exposure as data unit 5-tuples and Code 2 node (tree, level, birth generation) (root records distinguished by instance generation of D16 (Publish Semantics) Rider 1; other unit classes distinguished by write sequence (instance generation, transaction number) in class identity segment—containers add no separate instance generation, 4 bytes in write sequence being it); regular path judged by D3 (Space Allocation) settled item 1 predicate (allocation records), scan rebuild path judged by P2 / P3 rules of C113 (no input for active version judgment of multi-version units during scan rebuild). ⚠️ **Basis judging it safe is isomorphic to tree ID rule**: orphan and live instance generations never match (D18 (Information Carried in a Block) settled item 11 "voiding checkpoint must change instance generation"), scan rebuild gates selecting new by write sequence. ⚠️ **Remaining doubt**: statement on statistics merge rule "'uncommitted equals unhappened' holds for regular mount path only, **scan rebuild inspects disk**" reads opposite—governs "watermark taking maximum in generation segment rather than latter wins", distinct from yet proximate to "whether unissued numbers can be reissued". ⚠️ **Rebuild scope is single-head image**: with clone heads, birth tree in shared leaf header is origin (I-1.3 (Block header tree ID consistency) C110 (Block header consistency reading for cross-head sharing and post-encryption metadata) slot); claiming by birth tree fractures clone head tree—multi-head image attribution requires clone ancestor table (E90 (Tree ID Entering AAD and Cross-Head Sharing) second entry); multi-head image inode tree rebuild lacks input prior to implementation. ⚠️ Fill rate has no lower bound: `⌈n/233⌉` is fresh-tree value; 1e6 sequential creations deleting 90% unmerged yields 9.98× space amplification; widening records shrinks blast radius (160 ⇒ 204 records), trade-off belonging to C100 (Inode container loss is permanent loss uncovered by redundancy clauses). ⚠️ **Nine items formerly blocking "new-pool file creation writes" (Milestone 1), (1) (2) settled, remainder recorded**: (1) tree table entry settled by settled item 8; (2) accounting entry seq in value, width 4 bytes (settled item 7); (3) statistic tag values (C71 (Unrecognized statistics tag silently ignored)); (4) device segment value for statistics without device dimension, accounting row initialization for new trees, active reading method (C117 (Device segment of statistics without device dimension lacks assigned value)); (5) whether Code 3 containers carry extension points (C108 (Extension point attribution lacks judgment for packed record units)); (6) base bytes of Code 2 node header (concluded by settled item 11); (7) `deleted_inodes` shape (settled item 16); (8) (9) spec files for record format and statistics registry frozen components (C119 (Frozen components lack spec files)). ⚠️ Rebirth of container identity upon cross-head first COW belongs to E90 (Tree ID Entering AAD and Cross-Head Sharing) second entry. ⚠️ **`blocks` does not serve external `st_blocks`**: stat `st_blocks` reporting (computed on the fly from extent tree during stat) and SEEK_HOLE / SEEK_DATA moved to POSIX milestone. `blocks` rounded up by file length alters bytes of new-pool file creation: 8 bytes at offset 48 of that record changed from 64 to 6. ⚠️ **Write time unjudgeable on single image**: value is publish parameter, unobservable on single image—O1 (In-memory model differential (HashMap ideal file system)) model and differential omit write time, O2 (Independent parser + checker) judges single image only, O3 (Independent specification executor) projection excludes inode tree; judged by harness test cases pinned by write path (overwrite, sequential write, creating inode, unlink, one each); multi-unit sequential write, rollback, and incidentally rewritten records in same leaf lack cases across three cells (C677 (Write time lacks test cases across three write path cells)).

⚠️ **Two costs of death divided into three layers**: Tombstones take effect from intention publish; during unlink scan rebuild cannot read nlink = 0 record and remaining data units, rebuild view at crash points omitting objects with published intention (E178 (Leaf Container and Record Death Accounting) paragraphs 2, 3); upon borrow right piece retains identity, container number post-borrow potentially not exceeding left piece maximum key; third sentence of I-9.4 (Container number does not exceed minimum key) (left container max key < right container number) deleted alongside this form, invariant list and checker updated accordingly (2026-10-09 batch 50); cost is "right container number assigned too small yet exceeding left neighbor upon split" unblocked by criteria on such images, leaving `inode_tree` unit test only.

**Basis**:

- E98 (Geometry of Inode Records and Inode Tree): Inode record on-disk shape—packing into Code 3 avoids D21 (Boundary Between Authoritative and Derived States) Hard Constraint 6; residing in index nodes triggers it (11 of 14 fields uncomputable); modification count alone cannot order two versions in same window via checkpoint_txg, hence internal nodes unbuffered, only final version landing on disk within same publish window.
- E103 (Update Cost of Inode Packing Path 1): Path 1 (independent container index + inode tree referencing by identity) read cost exceeds container index merged into internal nodes, adding extra unit read while single write cost identical—container index does not form separate tree.
- E104 (Active Version Judgment of Scan Rebuild): Without recognizing retired records, dead inodes in retired containers resurrect; model showed "rebirth taking minimum inode number as container number collides", hence rebirth retains container number, swapping birth tree and birth generation only.
- User decision 2026-09-05 following four rounds of three-party review (materials `_d8-item6-path.md`, `_d8-item6-r2-amended.md`, `_d8-item6-r3-conclusion.md`, `_d8-item6-r4-conclusion.md`, `_d8-item6-r5-landing.md`, outputs `research/prompts/d8-item6-r{1,2,3,4}-{sonnet,opus}-output.md` and `research/results/d8-item6-r{1,2,3,4}-local.out`).
- User decision 2026-09-12 (C297 (Dual birth trees in inode internal entry lack settled handling), two rounds of three-party review): Birth tree in identity reference and child pointer coexist, original wording in history.
- User decision 2026-09-16 (C143 (Inode number watermark rewinding to re-issue numbers after crash recovery falls back to old root), three rounds of three-party review): Uniqueness narrowed to (inode number, object birth generation), original wording in history.
- User decision 2026-09-23: `blocks` rounded up by file length, not representing allocated space; `st_blocks` moved to POSIX milestone (main agent proposal, C480 (How inode record blocks is computed lacks clause across repository)), original wording in history.
- User decision 2026-10-01: K-R3-3 "write root growth", `research/prompts/m3-delete-r3-main-verification.md` Section 3 G6-Tombstone tree root full (original wording in history).
- Verdict `research/prompts/c673-clauses-r1-main-verification.md` Section 2 K3: Write time values across write paths (both legs agree, attacker measured on replica, main agent rerun confirmed), unjudgeable by all three oracles.
- User decision 2026-10-03: Deletion steps count as modifications ("count as modification, update implementation"), time fields taking "B intention carries time, all three updated together", original wording in history; code round 3rd verdict `research/prompts/m3-unlink-code-r3-main-verification.md` Section 2 n3 row (narrowing "ctime updated along" without re-querying user, offset 64 writing identical value across three slots).
- User decision 2026-10-05: Scope adds "inode numbers not reused on same timeline" ("write it"), verdict `research/prompts/m3-delete-checker-r2-main-verification.md` Section 2 T15 row, original wording in history.
- E176 (Capacity and Write Costs of Inode Tree Leaf Merge Candidates): Merging yields doubled capacity (Q1r, unstable under load); extra leaf rewrites for left-absorbs-right and left-borrows-from-right non-inverting under position basis, negligible under content basis—both merge and borrow implemented; borrowing to bare lower bound shifts extra rewrites from "negligible" to "non-inverting"—borrowing evenly splits.
- E177 (Resurrection Counts When Omitting Tombstones Per Tree): Emptied leaf containers unpersisted to disk writing nothing resurrect inode tree under kb active-version scan rebuild—container identity disappearance requires separate carrier (Layer 2 of death divided into three layers).
- E178 (Leaf Container and Record Death Accounting): With retired records carrying death txg and filtered by lineage, container resurrections and false deaths under borrowing, absorbing, and administrator rollback all 0; file shortening judging liveness via reconstructed inode record yields 0 resurrections and false deaths; tombstones effective from intention miss 1 record and remaining units during unlink, omitting extra objects at crash points; borrowing swapping right piece identity writes one extra Code 3, Code 2, and retired record each time compared to retaining identity.
- Three-party verdicts `research/prompts/m3-leaf-death-r1-main-verification.md` (Section 2 K1: "death reflected by active leaf version" modified literally rather than overturned; K3: evenly split borrow), `research/prompts/m3-leaf-death-r2-main-verification.md` (Section 2: retired records carry death txg, active-following trees select version per P2 only; separator key takes child current minimum key; evenly split borrow maintained), `research/prompts/m3-leaf-death-r3-main-verification.md` (Sections 2, 5).
- User decision 2026-10-08: "Implement borrowing alongside M3" (`m3-3-5-forks.md` M row); container identity disappearance writes retired record A, file shortening adopts Interpretation Z, tombstones effective from intention, borrow separator key takes child current minimum key and deletes third sentence of I-9.4 (Container number does not exceed minimum key) (`m3-leaf-death-forks.md`, verdict `research/prompts/m3-leaf-death-r3-main-verification.md` Section 5). Original wording in history.
- Retired record writers append only, death txg takes identity disappearance publish, readers and checkers recognize lineage records only: verdict `research/prompts/m3-retire-r1-main-verification.md` Section 2 K2 "which publish death txg takes" and "how many times same identity retires" rows (both attacker samplings confirmed: death txg taking unlink intention or final version birth generation falsely judges death; writer "omit if exists" or "overwrite by identity" resurrects) and Section 3 clauses 3, 4; user decision "build upon pool creation (recommended)", original wording in history.

**Owed**: C100 (Inode container loss is permanent loss uncovered by redundancy clauses), C102 (Index node header lacks reserved area, adding fields is not compat), C103 (Container index lacks placement), C104 (Payload integrity of Code 1 / 2 on scan path lacks self-contained evidence), C106 (Open container count upper bound), C107 (Index node AAD identity lacks placement), C110 (Block header consistency reading for cross-head sharing and post-encryption metadata), C113 (No input for active version judgment of multi-version units during scan rebuild), C115 (Inode attribute overflow path lacks placement), C116 (Leaf container split / merge discipline lacks failing checks), C119 (Frozen components lack spec files), C152 (Scan rebuild recheck of reissued uncommitted numbers post-narrowing), C347 (No clause for what clone head claims when ancestor table unreadable), C677 (Write time lacks test cases across three write path cells).

#### Settled Item 7: Write Buffer Entry `seq` —— 4 Bytes, Residing in Value

**Settlement**: **`seq` width 4 bytes, resides in value not key**, scope covering three write buffer trees: accounting / reverse index / LRU; **one entry format per tree, identical width across all three**. Pinned width reverses causality: **upper bound of `T_dirty` is henceforth a mount-time judgment** (4 bytes = 32-bit window, corresponding to ~136 GiB at 34 bytes/entry); D16 (Publish Semantics) settled item 5 statement "`T_dirty` is tunable value, not format constant" holds as before, altered by possessing format-determined upper bound. **Placing another txg in seq duplicates segment already in key**, forbidden. Two riders:

1. **Reverse index and LRU key definitions must likewise bound deduplication window by `T_dirty`** (generation in key, or equivalent). Basis for 4 bytes sufficiency is "entries to deduplicate for same key belong to same publish window", deduced on accounting tree from generation being final key segment per D5 (Snapshot / Space Accounting Mechanism) settled item 5; key layouts of those two trees absent across repository; without fulfilling this rider, 4 bytes lacks basis for them.
2. **Cross-batch insertion order for same key must align with `seq`**, forbidden from remaining implicit assumption—same key in btree layer is "latter write wins", not "larger `seq` wins"; under leaf-partitioned parallel flush physical arrival sequence of two batches unconstrained by `seq`.

**Scope**: Settles width and placement of this segment in entry; does not settle other segments (accounting key `(statistic type tag, tree ID, device, generation)` four segments in D5 (Snapshot / Space Accounting Mechanism) settled item 5). ⚠️ This segment is not orthogonal to others: those four segments are **non-unique within single checkpoint** (idempotent full values written repeatedly); deduplication rule "latter wins" verbatim requires "latter" carried by entry itself ⇒ without it, four segments combined do not form a key determining merge result. Established in **D8 (Core Index Structure) rather than D5 (Snapshot / Space Accounting Mechanism)** because write buffer covers three categories "reverse index / LRU / accounting"; placing in accounting forces other two trees to re-settle individually. ⚠️ Accounting entries consequently change from 30 bytes (key 22 + value 8) to **34**; figures in E97 (Entry Encoding of Accounting and Allocation Records) / E71 (Accounting Key Entry Count and Width) / E54 (How Many Generations of Accounting Generations to Keep) / E99 (Deduplication of Writebuffer Entries and Seq) require recalculation. ⚠️ "Inode number watermark" accounting entry carries `seq` as usual (settled item 15 clause 3 is format requirement), merely merge result takes maximum, independent of `seq`. ⚠️ Value assignment rules (starting value, counting per publish) in settled item 10; coupling between front-end and node buffer in settled item 12.

**Basis**:

- E99 (Deduplication of Writebuffer Entries and Seq): Measured how many entries to deduplicate for same key within single publish window and required bits—window demand falls within 32 bits, 4 bytes suffices.
- E17 (Write Buffer Merge Limit): First version omitted entry sequence numbers, two arms mismatching truth immediately, compelling decidable form "entry must carry inherent `seq`"; leaf-partitioned parallel measurement showed physical landing unconstrained by `seq`, Rider 2 requiring explicit documentation.
- Literal conflicts among three settled clauses (three-party review, main agent direct check): D5 (Snapshot / Space Accounting Mechanism) settled item 2 verbatim "no lookup needed ⇒ point deletion not scan; touched keys = statistic count"—with `seq` in key multiple rows exist under same prefix, point deletion failing; settled item 6 verbatim "entry carries seq as usual... merely **merge result does not depend on it**"—monotonic statistics taking maximum requires two meeting, whereas with `seq` in key they form distinct keys, never meeting; D5 (Snapshot / Space Accounting Mechanism) settled item 5 key definition omits `seq`, "latter determined by `seq`" self-consistent only when `seq` excluded from key. Main agent fourth point: `seq` in key renders key unconstructible—folding leaves single entry per generation; readers querying `(tag, tree, device, generation)` unaware of `seq`, point lookup degrading to prefix scan.
- User decision 2026-09-06: Width takes 4 bytes reversing causality, scope expanded to three trees, `seq` residing in value (original wording in history).

**Owed**: C161 (Two buffer levels coexist with undefined coupling), C162 (Recalculating four experiment figures after seq enters value), C345 (Staleness detection of reverse index / LRU lacks temporal clues), C416 (Deduplication window bounded by T_dirty rider unchecked).

#### Settled Item 8: Field Table of Tree Table Entries

**Settlement**: **Tree table entries total 200 bytes** = **tree ID 8** (leading) + entry length 2 + tree kind 2 + flags 2 + root pointer 86 + `previous_snapshot_txg` 8 + birth txg 8 + **head ID 8** + reserved 76. Per level ⌊(16384 − 131) / 200⌋ = ⌊16253 / 200⌋ = **81** trees, two-level capped at 81² = 6561 trees. ⚠️ **Denominator measurement basis fixed here**: 16384 minus **Code 2 header including reserved space** (131 is tree table unit's own Code 2 header, key width 8 ⇒ 115 + 16, D18 (Information Carried in a Block) settled item 16 / settled item 18; entry area begins at end of header including reserved space, settled item 11), subtracting no other numbers.

- **(1) Snapshots enter tree table**, two entries per read-only snapshot: Code 14 (snapshot inode root) and Code 15 (snapshot extent root); root pointer copies verbatim two roots of live head at snapshot creation; **birth txg 8 bytes**, measurement basis is **txg of checkpoint where snapshot completed (published)**, not birth generation of root node (which is its last COW generation, uncomputable by this tree). Provides carrier for I-3.7 (live snapshots enumerable).
- **(2) Tree ID watermark resides in root record, taking max across all root records in root ring**. Verbatim form: `new_watermark = max(all root records in root ring for this field, highest tree ID issued in this checkpoint + 1)`, **rewritten with root record on every publish**. "All root records in root ring" reads as all self-proven roots in root ring (including abandoned timelines), merged with this field carried in new-root segments of all journal records passing self-proof in ring (D23 (Role and Format of Journal) settled item 15): when root slot is unreadable while record remains, recovery rebuilds root from record, record watermark being that root's watermark; journal records overwritten in ring rotation only after corresponding tree written and root lands on disk. Slot where both root and record unreadable (dual media failure) uncovered, recorded in C342 (Tree ID watermark rewinds to reissue when root unreadable). Issuing first file version beyond mkfs watermark, thirteen trees allocate sequentially from watermark in order of settled item 11 65536..65548 (extent, inode, allocation record, accounting, central mapping, livelist, sparse side table, deadlist, `deleted_inodes`, tombstone tree, lineage tree, logged_ops, retired tree). ⚠️ **The word "cumulative" is load-bearing**: reading as "highest issued in this checkpoint" (non-cumulative) reissues tree IDs as roots rotate out of ring, where **reissued IDs belong to published trees**, directly hitting D6 (Snapshot Implementation Model) Criterion 9 ⇒ invalidating entire settlement.
- **(3) `previous_snapshot_txg` one copy per head, residing in tree table entry** (8 bytes). It is a **scalar per head**, not per unit, answering "which snapshot this head branched from", unrelated to unit content ⇒ per D1 (Data Mobility / Reverse Index) settled item 4 criteria, excluded from unit self-contained section. Tree table entry carries **two mutually independent 8-byte txg fields**—birth txg (own publish generation) and `previous_snapshot_txg` (predecessor snapshot generation), defined for both live head and snapshot entries.
- **(4) Compatibility tier adopts "tree table entry carries inherent length", rejecting "index node each entry carries length"**: former allows older readers to **skip** unrecognized tree; latter allows older readers to **unpack** nodes of new tree with **zero consumers**—rejected not for high cost, but for **failing to buy required capabilities**.
- **(a) Older writers encountering unrecognized accounting rows** (limited to "tree ID unrecognized, statistic tag recognized"): **writer must carry such rows to new generation as-is** (value unchanged, generation updated; writers with front-ends treat as front-end entries, drained to leaves on publish; first-version writers lack front-end (settled item 10), landing directly on leaves, on-disk layout identical, reading correctly). Step rows do not change generation segment (D5 (Snapshot / Space Accounting Mechanism) settled item 2 scope "step rows for inode number watermark"); when the watermark changed in the previous version's publish, the same row is both a current row and a step row.
- **(b) Remaining 15 flags bits**: **unknown bits rejected unconditionally—entry EIO** (object-level, not rejecting whole pool). **Rejects** "ignore unknown bits and interpret per known semantics", which is silent misinterpretation. Paired with two disciplines: **changes requiring older reader visibility use system configuration incompat bits, forbidden from per-entry flags**; **checker forbidden from "skipping"**—I-3.1 (allocated statistics match) verbatim "allocated space statistics == sum traversing **all** references" is unqualified; checker skipping unrecognized tree truncates traversal sum, producing false positive red rather than missed detection; correct behavior upon unrecognized tree is **reporting "incomplete traversal" and halting**.
- **Ordering contract and nesting levels**: **entries sorted in ascending order of tree ID; nesting levels uncapped by format**. Rationale for tree ID ordering: universal key used in 5-tuples, accounting keys, container indices; (2) ensures strict monotonicity with creation order. Rationale for no cap: capping introduces extra failure mode, cap shifting with entry width.
- **Tie-breaker takes `(birth txg, tree ID)`, zero bytes**: D16 (Publish Semantics) settled using `(txg, snapshot_id)` to break ties; moving (2) to root record ensures tree ID **strictly ascends with creation order** (cumulative watermark, non-decreasing, crash burning numbers without rewinding—I-7.8 (Root record tree ID watermark not below pool max tree ID) making this decidable) ⇒ `(birth txg, tree ID)` forms total order consistent with creation order, tree ID already present in entry.
- **Head ID 8 bytes value assignment**: inode tree entry writes own tree ID (65537), extent tree entry writes head it serves (65537), `deleted_inodes`, tombstone tree, and retired tree write inode tree tree ID (65537), lineage tree writes 0 (pool-wide), logged_ops writes 0 (the compaction intention entry field table is settled, see D26 (Background Compaction and Placement Reclamation) settled item 2; C364 (compaction intent record fields lack field-by-field reconciliation) still owes the field-by-field reconciliation of the field table against the implementation; the field-whitelist judging of I-4.4 (intent only describes target state) is implemented), allocation record tree / accounting tree / livelist shared tree / sparse side table tree / deadlist tree write 0 (unassigned, identical convention as D5 (Snapshot / Space Accounting Mechanism) "pool-wide / unassigned" rows); central mapping tree excluded from tree table. See D5 (Snapshot / Space Accounting Mechanism) settled item 9. ⚠️ 65537 in parentheses is value on version issuing from mkfs watermark 65536 (new-pool file creation); criterion is relational, not literal: inode tree entry head ID equals own tree ID; extent tree, `deleted_inodes`, tombstone tree, retired tree entries head ID equals inode tree tree ID in same table; lineage tree, logged_ops, allocation record tree, accounting tree, livelist shared tree, sparse side table tree, deadlist tree write 0. On versions where tree ID does not issue from mkfs watermark (elevated watermark, or clone head inode tree), head ID follows inode tree tree ID in table, no longer 65537 (D5 (Snapshot / Space Accounting Mechanism) settled item 9 "writable head ID = tree ID of that head's inode tree"). Judged by I-9.27 (Tree table entry head ID value).
- **Reserved 76 bytes**: Known subscribers arranged by "one byte range one meaning", **registered individually in tree table entry reserve subscription table** (gate doc-registries tree-table-reserve cell computing sum: total subscriptions must not exceed 76, each pointing to open owed check or settled item). Sum 48, remaining 28.

<!-- gate:tree-table-reserve total=76 -->

| Subscriber | Hosting Entry | Width | Source |
|---|---|---|---|
| livelist per-head counter | Head entry | 8 | C271 (When livelist FREE is recorded undefined) |
| Second destruction gate live snapshot counter | Head entry | 8 | C272 (Second destruction gate lacks O(1) check) |
| origin link / clone time | Head entry | 8 | C271 (When livelist FREE is recorded undefined) |
| Snapshot lineage interval label | Snapshot entry | 16 | C144 (Snapshot lineage interval labeling lacks carrier) |
| `children` (child snapshot counter) | Snapshot entry | 8 | D5 (Snapshot / Space Accounting Mechanism) settled item 18 |

⚠️ **Inode number watermark excluded from table**: C143 (Inode number watermark rewinding to re-issue numbers after crash recovery falls back to old root) settled on 2026-09-16 "do not move watermark, watermark stays in accounting" (D5 (Snapshot / Space Accounting Mechanism) settled item 4 Item 12) ⇒ subscribes zero bytes here.

**Modifies bytes of new-pool file creation: Yes**—tree table unit t12 entry width field (200), declared length (12 × 200 = 2400), and arrangement of 12 entries.

**Scope**: ⚠️ **Scope of (a) reaches "tree ID unrecognized" only; prescription for other half is opposite**: unrecognized **statistic tags** governed by C71 (Unrecognized statistics tag silently ignored), which verbatim requires "unrecognized statistic tags encountered at mount **must reject writable mount per compat_ro**"—(a) neither covers nor overturns it. Mechanistic basis for separation: fixed key width in D5 (Snapshot / Space Accounting Mechanism) settled item 5 ⇒ writers unfamiliar with tag **semantics** still partition segments at fixed offsets; mechanism holds via settled item 1 "accounting entries take idempotent full values", rows movable even if opaque to carrier, writer unable to modify unrecognized tree ⇒ values remain unchanged. Cost of non-carrying: D5 (Snapshot / Space Accounting Mechanism) settled item 2 settles each root accounting tree holds current generation rows rewritten in full ⇒ rows of unrecognized trees un-rewritten, vanishing next publish. ⚠️ **Narrow exposure**: only one statistic carries tree dimension (D5 (Snapshot / Space Accounting Mechanism) settled item 4 Item 12, maintained for writable heads only) ⇒ trees not writable heads have zero rows; adding another tree-dimensioned statistic scales uncarried losses by tree count. ⚠️ **(b) rejection avoiding data loss relies on (4) entry inherent length**: entry EIO but **position preserved** ⇒ writer knows "tree exists there unreadable by me" ⇒ avoids cross-tree deadlist merges; without length field next entry unlocatable, deduction failing. ⚠️ **(b) shares identical policy with two other flags fields in project**: D18 (Information Carried in a Block) "offset 7 is 8 flag bits... readers encountering non-zero reject... project leaves no 'ignore unfamiliar bits' path for these 8 bits"; settled item 6 "flags / padding / reserved share identical policy: constantly 0, readers encountering non-zero ⇒ **record EIO**", instantiated as I-9.7 (Record three segments constantly zero) ⇒ three flags fields unified into single rule, eliminating need to explain discrepancies. ⚠️ **"Bits" and "kinds" are two separate axes**: D15 (Format Freeze Policy) settled item 2 compat (new keyspace merely hides new trees from older readers) governs **kinds**, (b) governs **bits**—adding bits to tree presumed known alters semantics, not additive. ⚠️ **Uncapped nesting levels cost performance cliff**, observable at runtime per `.claude/rules/fs-design.md` Hard Requirement 4: "COW tree table units per publish = level count" must be readable at runtime—wrong levels raise no errors, undetectable without observation tools. ⚠️ **Table delivered by (1) insufficient**: "txg list of all roots" named in D5 (Snapshot / Space Accounting Mechanism) predated D6 (Snapshot Implementation Model) settled item 1 (one tree per head), D6 (Snapshot Implementation Model) counterexample shattering its premise (mainline txg=10 creates writable snapshot X, writes block in X at txg=30, mainline txg=35 creates snapshot Y; D5 (Snapshot / Space Accounting Mechanism) condition judges "Y references it" while Y cannot see it) ⇒ required object is not "one txg scalar per root", but **lineage**; birth txg 8 bytes remains indispensable (I-3.5 (Precision of reference intervals) enumerating all snapshot roots and live heads in txg ascending order, I-3.6 (deadlist tight bound) requiring `prev(S).txg`), what is owed is **layering labeling above it** (nested interval labeling involves two numbers rewritten with lineage shifts, distinct format property from "birth txg written once immutable"). Among three consumers of same premise, settlement fulfills I-3.7 (live snapshots enumerable) only: decidable form of I-3.5 (Precision of reference intervals) premised on "all roots ordered in single total order by txg", permanently false once lineage branches; generational reclamation in D18 (Information Carried in a Block) follows same interval test, over-judging ⇒ under-reclaiming ⇒ leaking. ⚠️ **(3) patches half-slot of live head header only**: D5 (Snapshot / Space Accounting Mechanism) verbatim "each writable head maintains one `previous_snapshot_txg` **and one deadlist**", "snapshot creation **hands over live head deadlist completely to new snapshot**, live head replacing with empty one" ⇒ snapshot entries must also point to deadlist, yet on-disk deadlist placement absent across repository. ⚠️ **Sole uncovered slot of C-2 judged non-error**: checkpoint allocating tree ID 42 writes unit header with 42, crashing before root commits ⇒ watermark lost with root ⇒ next mount reissues 42, orphan units with 42 remaining on disk; judged non-error because orphan and live instance generations never match (D18 (Information Carried in a Block) settled item 11 "voiding checkpoint must change instance generation"), I-1.2 (Block header write sequence published) verbatim "unjudged published must be uncommitted torn transaction garbage or abandoned timeline residue", scan rebuild gates and orphan reclamation ignoring tree ID, sharing 5-tuples between orphans and live versions being normal (C113 (No input for active version judgment of multi-version units during scan rebuild) Settlement P1 adding 10-byte write sequence for this), D6 (Snapshot Implementation Model) Criterion 9 rationale stating "keys in tree become visible" while orphans belong to no tree. Scope of D6 (Snapshot Implementation Model) Criterion 9 narrowed to "**published** snapshot identifiers monotonic without reclamation" (2026-09-06 user decision). ⚠️ **Inode number watermark yields separate answer**: D5 (Snapshot / Space Accounting Mechanism) settled item 4 Item 12 verbatim "**carried** (maintained for writable heads only)" ⇒ scalar per head, scaling with head count, while root record is fixed slot ⇒ C-2 unmovable; 2026-09-16 settled not moving watermark, narrowing uniqueness to "published (inode number, object birth generation) not reused" (settled item 6). ⚠️ **Tree table capacity calculated with disparate entry widths in two locations**: E79 (Root Record Capacity) and D22 (How Unit Atomicity Is Synthesized) settled item 7 reported trees per level under alternative model (treating tree table entry as "tree ID 8 + root pointer 59" = 67 bytes), direction unchanged (indirection unavoidable, nesting beyond one level), threshold lower than D22 (How Unit Atomicity Is Synthesized) text. ⚠️ Two known subscriptions in reserved space (deadlist placement, snapshot lineage nested interval labeling) **unnamed as fields**—naming settles unsettled items prematurely. Ordering contract ((1) tree ID strictly ascending, (2) co-issued number order) judged by I-9.16 (Tree table entries strictly ascending by tree ID and co-issued number order): rebuild and cold traversal reject before reading any tree, pool-level checker judging duplicate copy (Audit A3b Q2 and tree table ordering implementation, user settled 2026-09-27 establishing and synchronizing invariant). First version permits at most 10 concurrently live read-only snapshots (user decision, constant `SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION`; single node tree table capped at 34). Snapshots named by Code 14 tree numbers (first version lacks directories). Administrator rollback rejected unconditionally when live snapshots present (Code 14 / 15 entries in active or target tree table) (user decision, D23 (Role and Format of Journal) settled item 14).

**Basis**:

- E112 (Older Writer Encountering Unrecognized Tree): Pre-run fixed criteria and acceptance clauses evaluated silent outcomes and carriage costs across six cells—`Skip + Carry` yielded zero silent outcomes among mountable cells; `Ignore` misinterpreted record encoding; `Refuse` failed to mount all, incompatible with compat tier of D15 (Format Freeze Policy) settled item 2; carriage cost wrote two extra rows per generation, pre-set invalidation threshold unreached.
- E113 (Unrecognized Tree · Arm Set Completion): Arm set completed to 5 flags × 3 accounting with pre-set lexicographical selection; `RejectEntry` never lost across all scan points; `Skip` lost at Stage 1 (invisible snapshots caused deadlist merge to misidentify neighbors); `Ignore` lost at Stage 2; attacker leg's per-bit tiering `PerBit` lost for lacking third behavior ⇒ (b) revised from "skip entry" to "entry EIO"; accounting dimension selected `Carry` over `Exempt` (at the time `Exempt` required exception on D5 (Snapshot / Space Accounting Mechanism) settled item 2 K-generation, cost occupying zero cells; on 2026-09-23 K-generation point deletion withdrawn, `Exempt` losing object).
- User decision 2026-09-06: Four questions (snapshots entering tree table, tree ID watermark in root record, `previous_snapshot_txg` per head, compat tier taking entry inherent length), (a) (b), ordering contract and nesting levels, tie-breaker, reserved space 32 bytes (original wording in history).
- User decision 2026-09-23: (2) max includes watermark carried in ring record new-root segment (C342 (Tree ID watermark rewinds to reissue when root unreadable) prerequisite question; C511 (How birth generation connects after rolling back to version without files) Step 3 implementation writer chose this side, user concurred same day: "journal logs exist precisely to ensure complete tree writes", original wording in history); issuing sequence settled by main agent same day per format constant sequence, attacked zero rounds.
- User decision 2026-09-14: Tree ID moved to entry head, root pointer 83 → 86, reserved space carves 8 bytes for head ID (original wording in history; basis being settled item 11 Code 2 entry key is first key_width bytes, non-leading tree ID requiring exception to universal decode rule).
- User decision 2026-09-16: Entry widened from 148 to 200, reserved space 76, paying in trees per unit dropping from 109 to 81, two-level cap dropping from 11881 to 6561, user knowingly accepting (original wording in history).
- Verdict `research/prompts/c673-clauses-r1-main-verification.md` Section 2 K1: Head ID reads intra-table relationship, not literal 65537—literal reading falsely flagged 4 entries on valid high-watermark history while passing flawed implementation hardcoding 65537 (attacker measured on replica, main agent rerun confirmed); judged by I-9.27 (Tree table entry head ID value), evaluated only when tree table has entries.
- Retired tree head ID writes inode tree tree ID, tree table declared length 12 × 200 = 2400: verdict `research/prompts/m3-retire-r1-main-verification.md` Section 3 clause 1 (tree table entry head ID writes inode tree tree ID, isomorphic to tombstone tree, attacked zero rounds); user decision "build upon pool creation (recommended)", original wording in history.
- Verdict `research/prompts/m4-snapshot-r1-main-verification.md` Section 3 S1 (two new tree kind entries per snapshot, A eliminated; single node tree table capped at 34), S8 (rollback rejected when snapshots present) and Section 7 user decision (Fork 1 upper bound 10, Fork 3 reject if snapshots present; Fork 5 snapshots named by Code 14 tree numbers settled by main agent), original wording in history.

**Owed**: C144 (Snapshot lineage interval labeling lacks carrier), C145 (Deadlist lacks on-disk placement), C147 (Tree ID watermark invariant missing), C148 (Widening tree table entries lowers indirection threshold to Level 2), C149 (Loosening birth measurement basis by one tier post-root-slot-reissue), C156 (Record encoding misinterpretation lacks invariant), C158 (Checker skipping unrecognized trees flags false positives), C159 (E112 arm set missing three arms and balance omits unmountable costs), C342 (Tree ID watermark rewinds to reissue when root unreadable), C675 (At most one tree of same kind in tree table lacks invariant), C865 (unknown flags bits in a tree table entry make the whole tree table unreadable), C866 (F2 rollback does not tell lost bytes from rejected content).

#### Settled Item 9: Registry of Tree Kind Codes

**Settlement**: Value assignment for 2 bytes of "tree kind" in tree table entry has sole registration location in this settled item 9 registry; **new trees append only without code reuse**, 0 reserved as "none".

| Code | Tree | Source |
|---|---|---|
| 0 | None (unused in tree table entries) | — |
| 1 | extent tree | D4 (Checksum Location) settled item 7 |
| 2 | inode tree | Settled item 6 |
| 3 | allocation record tree | D3 (Space Allocation) settled item 3 |
| 4 | accounting tree | D5 (Snapshot / Space Accounting Mechanism) settled item 3 |
| 5 | central mapping tree | D19 (Block Pointer Structure and Width Budget) settled item 5 |
| 6 | livelist shared tree | D6 (Snapshot Implementation Model) settled item 2 |
| 7 | sparse side table tree | D5 (Snapshot / Space Accounting Mechanism) settled item 6 |
| 8 | deadlist tree | D5 (Snapshot / Space Accounting Mechanism) settled item 11 |
| 9 | `deleted_inodes` tree | Settled item 16 |
| 10 | tombstone tree | D18 (Information Carried in a Block) settled item 24 |
| 11 | lineage tree | D18 (Information Carried in a Block) settled item 24 |
| 12 | logged_ops tree | Settled item 5 |
| 13 | retired tree | D18 (Information Carried in a Block) settled item 11 |
| 14 | snapshot inode root | Settled item 8 |
| 15 | snapshot extent root | Settled item 8 |

**12** tree table entries written on new-pool file creation follow this table (Codes 1..4, 6..13 one entry each; central mapping tree root resides in root record, excluded from tree table, D19 (Block Pointer Structure and Width Budget) settled item 11; Code 5 registered as usual); `TREE_KIND_*` constants in E142 (Dry Run of New-Pool File Creation) comprise eight values for Codes 1..8 (Codes 9..15 absent); `crates/singlefs-core/src/records.rs` and checker `walk.rs` use identical constant group (Code 5 excluded from tree table, absent there).

**Scope**: Registers value assignment for "tree kind" segment only; tree ID is separate quantity, domains disjoint (tree ID ≥ 65536, values in settled item 11). Distinct from unit class registry in D18 (Information Carried in a Block) settled item 11, forming two concepts (user rejected unification). Code 8 (deadlist) shares precedent with livelist shared tree and sparse side table—all structures "read on snapshot destruction while destruction forbids full disk scan"; day-1 registration eliminates "initial tree creation" path of lazy creation upon first snapshot; entry shape settled separately in D5 (Snapshot / Space Accounting Mechanism) settled item 12. **Modifies bytes of new-pool file creation: Yes**—adding Code 8 expanded tree table from 6 to 7 entries, tree ID watermark shifting from 65543 to 65544 (tree IDs numbered from 65536, deadlist being 65543).

**Basis**:

- No experiment: Code values are value assignment conventions without measurable quantity; registry settled by user on 2026-09-13 (Codes 0..7) and 2026-09-14 (adding Code 8), both unverified by three-party review, refutable; original wording in history.
- Codes 9..12: Verdict `research/prompts/m3-blanks-r1-main-verification.md` Section 2 kind code row (derived independently by both legs); disjoint domains of tree ID and kind codes settled by user 2026-10-01 (`m3-blanks-r1-forks.md` fork sheet "kind code and tree ID disjointness" row, original wording in history).
- Verdict `research/prompts/c673-clauses-r1-main-verification.md` Section 2 K2: Tree table entry count on new-pool file creation follows active implementation value (11 entries at verdict, 12 entries post-retired tree, `crates/singlefs-format/src/lib.rs` `NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES`).
- Code 13 retired tree, new-pool file creation writing 12 entries: verdict `research/prompts/m3-retire-r1-main-verification.md` Section 3 clause 1 (kind code appended, tree table placed last, registered with zero root day-1, value following tombstone and lineage precedent, attacked zero rounds); user decision "build upon pool creation (recommended)", original wording in history.
- Code 14 snapshot inode root, Code 15 snapshot extent root: verdict `research/prompts/m4-snapshot-r1-main-verification.md` Section 3 S1 (two new tree kind entries, appended without reuse; A eliminated, both legs agreed, specific fields zero rounds).

**Owed**: C315 (Tree kind codes lack registry).

#### Settled Item 10: First-Version Accounting Lands Directly on Leaves, `seq` Assignment Rule

**Settlement**: **First-version accounting lands directly on accounting tree leaves without write buffer front-end**; D5 (Snapshot / Space Accounting Mechanism) settled item 3 clause 3 reads "front-end is subsequent optimization". **`seq` = Nth write of same `(tag, tree, device, generation)` into tree within single publish** (not count of runtime modifications to that statistic), **starting at 1**; first version prohibits interleaved COW during insertion (fixed point of D19 (Block Pointer Structure and Width Budget) settled item 12: each Code 2 / Code 3 written once per checkpoint per instance, single COW at publish end) ⇒ each key written into tree once per publish ⇒ **constantly 1**; statistics taking maximum merge rule written identically.

**Scope**: Settles first-version path and `seq` counting; does not settle `seq` width and placement (settled item 7), nor coupling between front-end and node buffer (settled item 12, neither enabled in first version). Whether "constantly 1" holds after relaxing fixed point and enabling front-end or node buffer is governed by settled item 13 state folding.

**Basis**:

- No experiment: First-version front-end usage is implementation scheduling choice with no measurable quantity; settled by user on 2026-09-13 during pre-coding general audit, unverified by three-party review, refutable; original wording in history.

**Owed**: None.

#### Settled Item 11: Code 2 Payload Layout and Key Comparison Order

**Settlement**: Code 2 node header self-describing (reserved 2 followed by entry_count 2 + entry_width 2; key_width 1 precedes key range; offset table in D18 (Information Carried in a Block) settled item 18), base bytes 81 → 86; header including 29 bytes reserved space = `115 + 2 × key_width`. Entry area starts from end of header including reserved space (`115 + 2 × key_width`), **fixed-width entries continuously arrayed**, **declared length = entry_count × entry_width**; free bytes trailing entries constantly 0, participating in payload CRC. Payload CRC covers `[86 + 2 × key_width, 16384)` (starting at end of plaintext header, identical measurement basis as 105 for Code 1 and 107 for Code 3). **Key in entry is uniformly first key_width bytes of entry**—"read entry width → slice entry by entry width → take first key_width bytes as key" forms universal decode rule holding across all Code 2 trees, scan rebuild path avoiding consulting table "which position key occupies in entry for this tree". Total order of keys compares field-by-field as unsigned integers, left to right; **on-disk little-endian storage does not constitute memcmp order**. Tree IDs numbered from 65536 (minimum value unreachable by 2-byte kind field, domains permanently disjoint): extent 65536, inode 65537, allocation record 65538, accounting 65539, central mapping 65540, livelist 65541, sparse side table 65542, deadlist 65543, `deleted_inodes` 65544, tombstone tree 65545, lineage tree 65546, logged_ops 65547, retired tree 65548; mkfs watermark 65536, post-first-file-version 65549.

**Modifies bytes of new-pool file creation: Yes**—entry area starting points of six Code 2 units (`115 + 2 × key_width`: inode tree root 131, extent 163, allocation record 135, accounting 159, mapping 169, tree table 131), declared lengths, and tree table entry field order; tree ID fields (tree ID and head ID in tree table entries, tree ID in Code 2 header, tree ID segment of accounting key) and tree ID watermark shift accordingly, following tree IDs starting at 65536.

**Multi-level Code 2 tree growth and contraction** (accounting tree, central mapping tree, and trees using Code 2 btree per settled item 14): (1) Nodes created by splitting within single publish land in same segment as other units, entire path from root to leaf COW-rewritten normally, adding zero write sequence steps or barriers; (2) Leaves split from middle when full; (3) Contraction unlinks empty nodes only, decreasing height by one level when root left with single child, avoiding "merge below half" and borrowing entries; (4) Separator keys maintained concurrently: depressed when inserting below leftmost separator key, right child separator key updated to its new minimum key after entries shift between siblings; decidable per level: separator key ≤ child node header minimum key, and > left sibling child node header maximum key (key range of D18 (Information Carried in a Block) settled item 18); (5) Tree height = level in root node Code 2 header + 1.

- **Delete before insert within single publish**: Keys present in previous version and absent in current version deleted in ascending key order first, then new keys inserted in ascending key order; dictates shape only, not write sequence (writing final state, settled item 13).
- **Separator keys corrupted in tree reconstructed from disk**: Rebuild verifies only components forming tree (level, non-empty nodes, internal entry width, no duplicate references), omitting separator keys; when subsequent publish fails to delete per separator key, planning returns, writable mount rejected before taking ticket, on-disk state unchanged. Subsequent disposition (repair, read-only mount) unsettled.
- **Leaf entry width unjudged by tree reader**: Tree reader slices keys only; leaf entries narrower than field table reported independently by entry decoders; internal entries narrower than key + 86 reported by tree reader.

**Internal node entry = tree key + child pointer 86**: accounting tree 22 + 86 = 108, central mapping tree 27 + 86 = 113. Allocation record tree and extent tree adopt alternative key-space addressed structures per settled item 14, structures evaluated in three-party `m2-keyspace-r1`; when implemented as Code 2 btrees their internal entries are 96 and 110 (extent 110 settled earlier by user on 2026-09-24).

<!-- format-const: ACCOUNTING_INTERNAL_ENTRY_BYTES = 108 -->
<!-- format-const: CENTRAL_MAPPING_INTERNAL_ENTRY_BYTES = 113 -->

**Scope**: Governs Code 2 payload layout and key comparison; does not govern field offsets in header (D18 (Information Carried in a Block) settled item 18), nor entry contents beyond key (individual tree items). Sole entry requiring reordering due to "key leading" is tree table entry, restructured with tree ID leading (settled item 8). "Fixed-width entries continuously arrayed" and "declared length = entry_count × entry_width" hold unmodified after central mapping entry returned to fixed width 55 (D19 (Block Pointer Structure and Width Budget) settled item 10)—entry width field having single u16, these statements fail under dual widths. Both settlements added on 2026-09-14 were user popup decisions, unverified by three-party review, refutable. ⚠️ Splitting, contraction, split point, and write sequence attacked zero rounds, three-party prototype sampling once; "merge below half" and "split at end upon append" matched chosen pair zero-to-zero across crash histories, rejected on user trade-off rather than safety. "Dedicated publish for splitting" eliminated: structural-only publishes displace "last 4 distinct rollback states" of D16 (Publish Semantics) settled item 1. Inode tree leaves are Code 3 packed record units, not Code 2 nodes: (2) "split from middle when full" and (3) "omit merge below half and borrowing" govern Code 2 nodes, reaching zero leaves of inode tree; leaf end-splitting, left-absorbs-right, and left-borrows-from-right (evenly split) follow settled item 6.

**Basis**:

- E145 (Costs of Code 2 Self-Describing Header and Mapping Tree Key Width): Measured additions for Code 2 self-describing header and mapping tree key width—widening to 86 degraded neither fanout, tree height, nor header decoding; self-description worthwhile.
- User decision 2026-09-13: Code 2 header self-describing, base 86, entry area start, payload CRC coverage, key comparison order (original wording in history).
- User decision 2026-09-14: Entry key uniformly first key_width bytes; "fixed-width entries continuously arrayed" and "declared length = entry_count × entry_width" hold unmodified (original wording in history).
- User decision 2026-09-24: Extent tree internal entry 110 bytes (original wording in history); E157 (Counting Models for Two Parallel Line Clauses) conclusions invariant across sample points 110 and 136.
- No experiment: Splitting, contraction, split point, write sequence chosen by user after three-party prototype (`research/prompts/m2-treesplit-r1-opus-model/`) tied zero-to-zero across identical crash histories, no experiment number established; separator key concurrent maintenance settled by main agent hitting issues in that round (prototype t1.out: retaining D8 (Core Index Structure) settled item 6 "separator key immutable post-creation" red across all 24 cells, concurrent maintenance 0 across all 24 cells), verdict `research/prompts/m2-treesplit-r1-main-verification.md`.
- User decision 2026-09-24: Write sequence same-segment normal COW, unlink empty nodes only with height drop when root has single child, split from middle, internal entry = tree key + child pointer 86 (original wording in history).
- `m2-treesplit-implementer-report.md` Section 5 B1, B5, B6.
- User decision 2026-10-01: Tree IDs shifted starting at 65536, disjoint from kind codes (`m3-blanks-r1-forks.md` fork sheet "kind code and tree ID disjointness" row, original wording in history).
- Verdict `research/prompts/m3-blanks-r1-main-verification.md` Section 2 kind code row: new tree kind codes take 9..12 (derived independently by both legs).
- Retired tree tree ID 65548 and post-first-publish watermark 65549: verdict `research/prompts/m3-retire-r1-main-verification.md` Section 3 clause 1 and Section 5 (values following precedent, attacked zero rounds); user decision "build upon pool creation (recommended)", original wording in history.

**Owed**: None (C306 (Code 2 header non-self-describing key width) cleared on 2026-10-11, see owed checks cleared table).

#### Settled Item 12: Coupling Between Write Buffer Front-End and Node Message Buffer —— One Level Permitted Per Tree

**Settlement**: Three trees using write buffer front-end (accounting / reverse index / LRU) have **ε constantly 0, internal nodes reserving no message buffer**; writes on these three trees **uniformly enter front-end**—including accounting rows carried by older writers per settled item 8 (a): writers with front-ends treat as front-end entries, first-version writers lacking front-ends (settled item 10) landing directly on leaves, on-disk layouts identical, reading correctly; **drained to leaves before publish**, comparing arrival order within same window for same key; front-end entries of publish set drained in same publish, cut per transaction, non-spanning across publishes (fsync = single full run publish). Other trees reserve buffers per tree axis of D11 (Whether to Reserve Message Buffer in Index Nodes) settled item 6. **Neither level enabled in first version** (settled item 10, D11 (Whether to Reserve Message Buffer in Index Nodes) settled item 7), altering zero bytes of new-pool file creation.

**Checks**: Write path of front-end trees exposes front-end entry points only (type layer); runtime assertion—every logical mutation (key → value) on leaves of front-end trees within a checkpoint corresponds to an entry entering leaves this checkpoint, counting both mid-window flushes and publish-time drains, preserved across instance switch reissues; rebalancing, index rebuild, corruption recovery rebuild three exceptions each paired with "values read for each key before and after unchanged". Can be runtime only: on-disk state cannot distinguish draining via front-end from direct leaf writes. Three literal assertion forms (recognizing publish-time drains only, recognizing current attempt only, counting physical new leaves) falsely alert on valid histories (`c161-r3-model-output.txt` `G3_publish_only` / `G3_attempt` / `G3_physical` rows).

**Scope**: **Cost**: Drain-to-leaf writeout per publish is 3.4–10.7× that of "drain-to-root-buffer" forms (4.8–10.6× under overwrite workload, third paragraph of `part` column in same artifact); entries in publish set must land on leaves within current publish, unbatchable per budget; front-end holds an order of magnitude fewer entries under same dirty page budget—`T_dirty` calculation measurement basis absent. **Benefit**: Front-end trees eliminate cross-level coexistence of same key, shortest read paths; staleness detection (C52 (Staleness treated as legal)) lacks objects on these three trees. **Uneliminated but unselected**: A-R4 (front-end lands directly on leaves, node buffer accepts non-front-end writes only) and B-D (front-end enters root buffer pushing down level-by-level, ordering by position) hit once on collapse; their staleness detection requires per-message publish numbers (conflicting with settled item 7 "prohibiting duplicate txg"), node layouts entering D15 (Format Freeze Policy) Layer 4. **Consequences**: how node buffers relocate during rebalancing (scope is all buffered trees, this item sparing front-end three only); fold starting point of tombstones against the leaf's current value (settled item 13); temporal clues for reverse index / LRU; branch variables of D11 (Whether to Reserve Message Buffer in Index Nodes) settled item 6 tree axis and phrasing of settled item 5.

**Basis**:

- No experiment: Measures whether design form hit by discriminator arms on valid histories, based on three rounds of three-party review (`research/prompts/c161-r1-main-verification.md`, `c161-r2-main-verification.md`, `c161-r3-main-verification.md` Section 4 each)—this form unhit across all shapes tested in three rounds (out-of-order flush, K-generation point deletion, split and merge, partial publish, crash mid-drain, deletion, height 4, index rebuild, instance switch, administrator rollback); eliminated B + seq global monotonic hit twice (counter unpersisted; counter reloaded from selected root upon instance switch), requiring 8-byte seq and persisted counter. User settled in closing popup on 2026-09-16, original wording in history.

**Owed**: C161 (Two buffer levels coexist with undefined coupling), C345 (Staleness detection of reverse index / LRU lacks temporal clues), C348 (T_dirty calculation lacks measurement basis), C349 (Node message buffer relocation during rebalancing lacks clauses), C383 (Intermediate state folding of same key in same publish lacks check).

#### Settled Item 13: Index Tree Updates Perform State Folding —— Intermediate States of Same Key Excluded from Leaves in Same Publish

**Settlement**: When same key on an index tree is modified multiple times in same publish (A → B → C → D → E), landing on leaves occurs once: entry for this key in leaf is written directly as final state E, A–D written into no nodes. "Same cycle" denotes single publish: identical `checkpoint_txg`, fsync-triggered publishes likewise being single publish (D16 (Publish Semantics) settled item 6). **E takes last modification by arrival order in same publish, not by value**; when E is tombstone, folded result is tombstone; treating tombstone as numerical value taking maximum alongside A–D is forbidden (C383 (Intermediate state folding of same key in same publish lacks check) measured naive folding losing point deletions 16 / 16 across four candidates, `b.out`). **The fold starts from the value this key's leaf holds from the previous publish (none if absent)**: draining = starting from it, apply this publish's front-end entries for the key in arrival order, tombstone = delete; the starting point counts before A–D and is not an intermediate state within the same publish. Once the front-end is enabled, the "previous generation leaving" of an accounting row lands as a tombstone, aimed at exactly this starting point. Arrival order carried in memory by entries themselves (settled item 15 clause 3), not relying on relative positions post-sorting by key.

**Scope**: **All leaf-landing paths across all index trees**. Beyond first-version direct-to-leaf (settled item 10), future write buffer front-end drains (settled item 12: landing compares arrival order within same window, resulting final entry being E), node message buffers (D11 (Whether to Reserve Message Buffer in Index Nodes) settled item 6 tree axis), and interleaved COW during insertion (permitted by D3 (Space Allocation) settled item 5, unadopted in first version by D19 (Block Pointer Structure and Width Budget) settled item 12) all land on leaves per this rule. **First version satisfies it already**: fixed point of D19 (Block Pointer Structure and Width Budget) settled item 12 causes settled item 10 to deduce "each key written into tree once per publish", `seq` constantly 1; settled item 13 elevates this from fixed-point corollary to independent clause, holding after relaxing fixed point, altering zero bytes of new-pool file creation. Implementation has no object today: `publish_overwrite` in `crates/singlefs-core/src/transaction.rs` performs single publish per overwrite (txg and transaction number incremented by one), possessing no path where "multiple transactions modify same key in same publish"; test cases packing two objects in same checkpoint modify distinct keys. ⚠️ **Two unanswered questions**: (1) Destination of A–D outside tree ungoverned here—when A–D are data unit versions from overwriting same logical block, per D16 (Publish Semantics) settled item 2 loss window and "one transaction one unit" slicing discipline, A–D are separate transactions, each possessing on-disk data unit and journal record (D23 (Role and Format of Journal) settled item 7: one record one transaction number); these units referenced by no root, outside I-7.4 (Blocks of recent K generations not reused) protection; whether to write, how to free within same publish, and how accounting computes lack governing clauses in decision text, owed checks, or invariant lists; (2) On-disk `seq` of settled item 7 constantly 1 under folding (expanding settled item 10 "constantly 1" to all versions); with E recognized via in-memory arrival order, whether on-disk 4-byte `seq` still serves as basis for `T_dirty` upper bound (statement "upper bound of `T_dirty` is henceforth mount-time judgment" in settled item 7) unanswered. ⚠️ "Monotonic statistics do not decrease within same publish, hence both methods yield identical number" is main agent deduction, unverified by checks.

**Basis**:

- No experiment: User settled "this is a decision, record it, temporarily unnecessary to implement and verify", opening no three-party review, creating no experiment, possessing no measurable quantity; user settlement 2026-09-19 and confirmation same day that "same cycle = single publish", original wording in history. Three pillars supporting omitting A–D are existing clauses (main agent verified 2026-09-19): D16 (Publish Semantics) settled item 4 and D23 (Role and Format of Journal) settled item 14 clause 5 (application unit is single publish; when legal prefix halts between two transactions of a publish, that publish is unapplied in full, replay exposing states on publish boundaries only), D16 (Publish Semantics) settled item 6 (fsync-triggered publish is also single publish, fsync-confirmed states landing on publish boundaries), D16 (Publish Semantics) settled item 14 (snapshot publish points aligned with checkpoint boundaries, pinned after all data changes in checkpoint, no snapshot referencing A–D).
- The fold-start sentence and how tombstones fold, merged into settled item 13: verdict `research/prompts/m3-close-c693-r2-main-verification.md` section 2 V4 cell (narrowed and merged into settled item 13, upheld, both legs agree) and `research/prompts/m3-close-c693-r3-main-verification.md` section 2 T4 cell (upheld; the check uses a drain differential, and the judgment that step H strictly increases goes with the per-tree split in milestone fourteen); the revision follows the forward leg `research/prompts/m3-close-c693-r2-sonnet-output.md` section "V4.5 clauses and owed checks to change". Forward-leg model `research/prompts/m3-close-c693-r2-sonnet-model/out/v4_fold_checks.out` lines 1 and 3: after 20 publishes on a history where a single head creates an inode twice, the rebuild-landing final state has 18 rows, while naive max-taking has 35 rows with 17 stale (round two verdict verification item 4, main agent rerun identical=1).

**Owed**: C383 (Intermediate state folding of same key in same publish lacks check).

#### Settled Item 14: One Btree Implementation + Multiple Independent Keyspaces + Two Front-Ends, Derived Trees May Be Heterogeneous

**Settlement**: **One btree implementation + multiple independent keyspaces + two front-ends, governing authoritative trees and trees following splitting; derived allocation record tree and extent tree (D21 (Boundary Between Authoritative and Derived States) settled items 5, 9) may adopt alternative key-space addressed structures, formats potentially differing from Code 2 btree.**

| Layer | Function |
|---|---|
| 独立 keyspace | Independent tree per data category (extent / inode / dirent / reverse index / free space / accounting ...), mutually isolated |
| 日志结构节点 | New keys appended not inserted in node, periodically rearranged in memory; sequential writes, writeback under read-only lock |
| **write buffer** 前端 | High-frequency small updates (reverse index, LRU, accounting) batched, sorted, deduplicated, and inserted in batches |
| **key cache** 前端 | Repeated reads/writes of same key set (allocation info) routed through hash table, avoiding full tree lookup |

**Structure of two derived trees**:

- **Allocation record tree**: Position-addressed by absolute slot numbers. Each leaf covers fixed range of slots `[k × W, (k + 1) × W)`, W even (two-slot data units never straddle leaves), and W ≤ leaf entry capacity 812 (one record covers at least one slot, leaf always housing them); records land on leaves by starting slot. Ascending levels position-addressed, child covered range computed from position in parent. **Range without records = completely free**: leaf unwritten, parent slot left empty (User K1). Checker verifies per node "this node covers range prescribed by position" and "record end slot does not exceed end slot of hosting leaf".
- **Extent tree**: Two segments, both position-addressed. Upper segment position-addressed by inode number; lower segment one tree per file, position-addressed by data unit number (leaf covers 144 units, internal fanout 147), holes represented by absence. **Files with single data unit omit lower segment**: upper leaf entry houses data pointer directly; entry carries tag byte distinguishing "no unit / lower root pointer / inline data pointer" (User K2). **Upper segment canonical form**: upper segment shape determined solely by set of inodes possessing entries, write and delete sharing this rule: nodes formed by union of root paths (chain from leaf to root) of inodes possessing entries, root level taking lowest level covering maximum number, taking Level 0 zero-entry root when set empty; complete rewrite rules in settled item 16 clause 8.
- **Reading at mount**: Allocation record tree read into mount state in full at mount (allocator inherently requires); extent tree on-demand, descended by position upon opening file (User K4).
- **Implementation choices** (Option 1 in Implementation Round 2, settled by main agent 2026-09-25, attacked zero rounds):
  - **Allocation record tree**:
    - Leaf width W = 812 (taking leaf entry capacity (16384 − 135) ÷ 20 itself), internal entry 96 bytes (key 10 + child pointer 86), fanout 169.
    <!-- format-const: ALLOCATION_RECORD_TREE_LEAF_SLOTS = 812 -->
    <!-- format-const: ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES = 96 -->
    <!-- format-const: ALLOCATION_RECORD_TREE_INTERNAL_FANOUT = 169 -->
    - Level L node covers W × 169^L slots.
    - Internal nodes and root write occupied slots only: one internal entry (key 10 + child pointer 86) per child, key taking start of range covered by child position (disk where child resides, first slot of range), sorted by key; childless slots consume zero entries; readers identify children by key; slot numbers off child level grid points, disks outside pool, or identified children unassigned to parent point to no child (`crates/singlefs-core/src/allocation_record_tree.rs` `child_named_by_entry_key`). Root node header key range covers whole key space, smallest_key = (0, 0), largest_key = (0xFFFFFFFF, 2^48 − 1).
    - Root covers whole key space, partitioned by disk; root level takes minimum R ≥ 1 such that Σ_disk ⌈disk_slots ÷ (W × 169^(R−1))⌉ ≤ 169. At 4 GiB × 2, root at Level 2, tree height 3.
  - **Extent tree**:
    - Upper leaf entry 113 bytes = key 24 ((0, inode, 0)) + tag 1 + payload 88. Tag 0 all zeros, reader-recognized only, writer unwritten; Tag 1 is lower root node pointer 86 bytes plus 2 zero bytes; Tag 2 is data pointer 88 bytes for single data unit.
    <!-- format-const: EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES = 113 -->
    - Upper leaf covers 143 inodes, internal entry 110 bytes (key 24 + child pointer 86), fanout 147.
    - Upper leaf node header key range per D18 (Information Carried in a Block) settled item 2 scope writes range covered by leaf position, omitting keys of first and last entries: closed interval, kth leaf (from 0) smallest_key = (0, 143k, 0), largest_key = (0, 143k + 142, 2^64 − 1).
    <!-- format-const: EXTENT_TREE_UPPER_LEAF_INODES = 143 -->
    <!-- format-const: EXTENT_TREE_INTERNAL_ENTRY_BYTES = 110 -->
    <!-- format-const: EXTENT_TREE_INTERNAL_FANOUT = 147 -->
    - Lower leaf covers 144 units, lower key byte offset = unit_number × 32634. Internal entries for upper and lower segments take key as start of range covered by child position (`crates/singlefs-core/src/extent_tree.rs` `build_extent_internal_entry`).
    <!-- format-const: EXTENT_TREE_LOWER_LEAF_DATA_UNITS = 144 -->
  - **Publish path fixed point**: guess rewrite set → perform releases on allocator copy (leaves dirtied by releases merged into rewrite set first) → obtain placements → calculate dirty nodes → re-run if absent from set. Returns `AllocationRecordTreeRewriteSetDidNotSettle` before landing if unconverged by Round 64; 64 lacks basis, recorded in new owed check.
  - **On-disk slots**: takes whole on-disk slots, discarding fractional slots at disk end; write and read sides share single definition (`crates/singlefs-core/src/allocator.rs` `absolute_slot_count_of_device`), identical value across both sides of allocation record tree root level (Option 6 in Implementation Round 2, settled by main agent 2026-09-25, attacked zero rounds; test case `half_a_slot_past_a_root_level_boundary_leaves_the_writer_and_the_reader_on_the_same_root_level_and_every_tree_written_reads_back`).
  - **Reported reads when opening file**: `ExtentTreeReadsAtOpen` counts extent tree nodes read, not block layer device reads; when hints stale and falling back via central mapping, adds 2 device reads per hop, numbers differing (inspected in Implementation Round 2, main agent adopted measurement basis without extra device reads, attacked zero rounds; test cases `opening_a_file_whose_lower_extent_segment_has_two_levels_reports_as_many_node_reads_as_the_block_layer_counts` pinning equal cell, `a_leaf_of_a_two_level_lower_extent_segment_with_a_stale_location_hint_reads_back_every_data_unit_through_the_central_mapping` pinning unequal cell, both in `crates/singlefs-harness/tests/tree_nodes_and_the_central_mapping_on_mounted_read.rs`).
- Nodes of both trees carry Code 2 headers per D18 (Information Carried in a Block) settled item 18 (tree ID, level, key range, birth generation); tree height read per D8 (Core Index Structure) settled item 11 "tree height = root header level + 1".

**Scope**: ⚠️ Heterogeneity granted only to derived, reconstructible trees: allocation record tree and extent tree; accounting tree (authoritative), central mapping tree, tree table, instance table, and inode tree follow single Code 2 btree (splitting and contraction of D8 (Core Index Structure) settled item 11)—inode tree possesses Code 2 root and internal nodes only, governed by settled item 11; leaves are Code 3 packed record units, end-splitting, left-absorbs-right, and left-borrows-from-right (evenly split) following settled item 6, unreachable by settled item 11 "split from middle when full" and "omit merge below half and borrowing". Heterogeneous structures still require: individual node self-description (which tree, level, key range, generation, D18 (Information Carried in a Block)), detectable corruption (D21 (Boundary Between Authoritative and Derived States) settled item 11), every new write sequence entering crash-point replay crash points, checker possessing dedicated judging code. Structures of those two evaluated in three-party `m2-keyspace-r1`. ⚠️ "Log-structured nodes" denotes **in-memory arrangement**—on disk each landing is whole-unit COW rewrite per single fixed-width entry array of settled item 11 (declared length exactly equals entry_count × entry_width), not multi-segment appended bsets. ⚠️ **Write buffer front-end unimplemented in first version** (settled item 10: accounting lands directly on leaves); coupling with node message buffer in settled item 12; three hard requirements in settled item 15. Index structures may vary while transaction layers must not per `.claude/rules/fs-design.md`, this item settling index side only. ⚠️ Exact values of leaf width W, upper fanout, and upper leaf entry field table unwritten in clauses chosen by implementation writer per above rules and detailed in handover, supplemented by main agent into this section post-settlement. ⚠️ Three new write sequences (writing chain of nodes at position 0 in same publish upon multi-level growth, releasing chain upon multi-level drop, swapping inline with lower segment) must enter crash-point replay, checker judgments implemented alongside code. ⚠️ Four items in this section attacked zero rounds; crash history of "global position-addressed + inline" un-scanned (user scheduled full run crash unified at end). ⚠️ Rejected candidates: global key-ordered (upper segment requires splitting), per-file root in inode record (modifies frozen inode record), global position-addressed non-inlined (falsely reports full at 14625th small file in single mount, violating D3 (Space Allocation) settled item 9 clause 1)—rationale in verdict Section 3.

**Basis**:

- No experiment: Single btree implementation with multiple keyspaces possesses no measurable quantity, based on inspection of `BCH_BTREE_IDS()` macro in `linux-6.17/fs/bcachefs/bcachefs_format.h` on 2026-08-29 (counting `x(` and `BTREE_IS_write_buffer` lines, source pinned in `~/code/fs-refs/`): bcachefs utilizes **one btree implementation + 21 trees**, of which **5** use write buffer (`lru` / `backpointers` / `deleted_inodes` / `rebalance_work` / `accounting`); superior to "different structures per access pattern" in **requiring one structure tuned correctly**—heterogeneous solutions imply N implementations, N crash consistency bugs, N checkers. prior-art.md bcachefs entry follows this text.
- User decision 2026-09-24: Derived trees may be heterogeneous (original wording in history); extra implementation, crash consistency, and checker covered by crash-point replay streams and checker respectively, no longer blocked by "one structure tuned correctly".
- No experiment: Three-party prototype (`research/prompts/m2-keyspace-r1-opus-model/`) measured across identical histories and cost models, unnumbered; selection settled by user, verdict `research/prompts/m2-keyspace-r1-main-verification.md`.
- User decision 2026-09-24: K1, K2, K4 (original wording in history).
- Implementation Round 1 report `m2-keyspace-implementer-report.md`: implementation writer values accepted by main agent, attacked zero rounds.
- Implementation Round 2 report `m2-e26-implementer-report.md` Section 6 Q3, Q4: on-disk slot count and file opening read count values accepted by main agent, attacked zero rounds.
- E142 (Dry Run of New-Pool File Creation) 15th run Paragraph 5 (`e142-r15-step5-runner-report.md` Section 4): allocation record tree internal and root cells, extent upper leaf key range cell—variant arm of independent harness matched crates byte-for-byte under this reading only; represents attribution, unverified on harness primary reading, rerun following updates documented later.
- Verdict `research/prompts/m3-multi-file-unlink-r1-main-verification.md` Section 3 "upper segment canonical form": upper segment shape determined solely by set of inodes possessing entries, write and delete sharing single rule (both legs derived identical rule; non-decreasing rule yields two shapes for same set).

**Owed**: C211 (Node landing form lacks text).

#### Settled Item 15: Three Hard Requirements for Write Buffer Entries

**Settlement**:

1. **Sorting destroys locality rather than creating it ⇒ node cache must be scan-resistant** (2Q / LRU-K or similar). After sorting by leaf, each flush constitutes a sequential scan covering entire leaf space, whereas **LRU hit rate on "repeated sequential scans larger than cache" is exactly 0**. **This is hard requirement, not recommendation.**
2. **Entry shape must be idempotent full value, never incremental Δ**. Accounting tree binds tightest—"freed 4096 bytes" naturally expresses as Δ, disposition in settled item 1. Project diverges from bcachefs here: its accounting tree adopts delta shape (`disk_accounting_format.h` verbatim "updates are _deltas_").
3. **Entries must carry inherent sequence number (seq), being a format requirement**. Sorting by key on flush **discards temporal order**, while deduplication rule is "latter wins" ⇒ **"latter" must be carried by entry itself**; unstable sorting leaves relative order of same-key entries undefined, rendering deduplication non-deterministic.

**Scope**: Governs entries and cache policies on write buffer path; width and placement in settled item 7, value assignment in settled item 10, coupling with node message buffer in settled item 12. ⚠️ Requirement 2 and Requirement 3 are distinct: idempotence governs "repeated application without error"; sequence number governs "deterministic overwriting". ⚠️ **Known cost** (bcachefs text): write buffer updates are unordered and eventually consistent; flush discards temporal order by key sorting; sort-merge-sweep is **inherently single-threaded**, its efficiency being critical path under multi-threaded workloads. Statement has two interpretations yielding opposite measurements—**thread interpretation** (merge CPU throughput insufficient for multi-threaded loads) **does not hold**: single-thread merge throughput assuming 3 entries per 4 KiB user write far exceeds disk bandwidth, leaf-partitioning enabling parallelism; **I/O interpretation** (tree insertion I/O of flush is critical path) **holds, being largest of three items**. ⇒ **What truly bottlenecks critical path is D9 (Encryption) encryption, not merging**; ⚠️ E17 (Write Buffer Merge Limit) measured merging only, omitting tree insertion and I/O.

**Basis**:

- E7 (Offline Index Harness): Project io/op matched bit-for-bit across cache-to-tree ratios of 2% / 6% / 25%; intervention confirmed preserving leaf aggregation while shuffling group processing order reduced reads on 25% cache tier, matching 100% cache tier before and after shuffle (negative control)—LRU hit rate on "repeated sequential scans larger than cache" is exactly 0, confirming Requirement 1 as hard requirement; same harness showed flush tree insertion I/O far exceeding encryption and merging, confirming I/O interpretation.
- E14 (Discriminative Power of I-3.1: Idempotent Full Value vs Incremental Delta): Under incremental Δ form I-3.1 (allocated statistics match) discriminative power dropped to zero, catching all under idempotent full value form—Requirement 2 pinned by it.
- E17 (Write Buffer Merge Limit): First version omitted `seq`, two arms mismatching truth immediately, compelling decidable form of Requirement 3; single-threaded merge throughput and leaf-partitioned parallel multiplier disproved thread interpretation of "sort-merge-sweep inherently single-threaded".

**Owed**: C15 (Write buffer mixing in Δ).

#### Settled Item 16: First-Version Unlink: Creates deleted_inodes Only, Batched from Tail, Resumes to Empty During Mount

**Settlement**:

1. **Intention**: Intention publish writes inode N record with nlink = 0 (size unchanged), adds key = N (8 bytes, no value, no object birth generation) to `deleted_inodes`, appends tombstone to tombstone tree open container (D18 (Information Carried in a Block) settled item 24); creates no logged_ops (K-U1 L4).
2. **Batching**: For any inode N, batched from tail by data unit count, each batch releasing K trailing units of N, updating size to "remaining units × payload capacity", remaining units taking N units in extent tree (0 if absent, final batch only); K product takes `EXTENT_TREE_LOWER_LEAF_DATA_UNITS` (144, free choice), test knob shrinking for tests; deletion batch rewrites only truncated lower leaf of N and ancestors, plus N chain in upper segment (K-R3-1, new extent branch, satisfying five requirements of `.claude/rules/fs-design.md`); final batch releases remaining units of N, deletes inode record of N, deletes `deleted_inodes` key, removes N entry from upper segment (writing zero Tag 0), in same publish; sole path, omitting fast-path completion. Intention, each batch, and each resumed batch occupy separate transaction numbers, equaling next of current instance (same sequence as regular publishes).
3. **Tree creation**: `deleted_inodes` registered with zero root during first file version publish; derived state: queue = set of numbers in inode tree with nlink = 0 (`research/prompts/m3-delete-r2-main-verification.md` Section 3 V7).
4. **Readers and write entry points**: `InodeRecord` nlink read by `parse` and written by `to_bytes`; fresh record entry points reject "nlink = 0" and "record absent"; read-only mounts, cold traversals, and model reads answer "no file" from intention commit onward; write entry points reject inodes with nlink = 0 (`research/prompts/m3-delete-r2-main-verification.md` Section 2 row 1, Section 3 V6).
5. **Deleted to empty**: "Tree exists with empty file set" is third member of `RebuiltVersion` / `PreviousVersion` / `PoolVersion`, distinct from `WithoutFile`; judgment = tree table has entries ∧ inode tree zero-entry root (key range `[0, 0]`, per settled item 6) ∧ extent tree zero-entry root (key range per position) ∧ `deleted_inodes` root pointer zeroed; omitting tombstone tree, lineage tree, and retired tree. Following full deletion, permitted operations are `publish_new_inodes` (size 0), writing rows, and empty publishes (K-U5 X1) (`research/prompts/m3-delete-r3-main-verification.md` Section 3 G5). Overwrites, sequential writes, and unlinks on emptied version rejected as existing "no file version to change" refusal (`UserChangeRefused::NoFileVersionToChange`). Version deleting first file while other files survive classified as `WithFile`: version whose inode tree retains any record or extent tree retains entries carries files, regardless of inode 1 presence; third member denotes genuinely empty file set.
6. **Resumption**: `mount_writable` drains queue to empty after writing rows and warm-up, before returning; excluded from pre-ticket dry run; in-session queue read fresh from active version, unbuffered; crash injection adds `WritableMountPhase::Continuation` (`research/prompts/m3-delete-r2-main-verification.md` Section 3 V3). Every user mutation in session (including unlink, write, create inode) drains active `deleted_inodes` queue to empty before executing; mount resumption unchanged; single unlink call = drain + intention + all batches (sharing drain with resumption). When deletion publish (intention and each batch) rejected by placement for space during session drain, prompts chain of floor F raises and reclamation before retrying; each rejected deletion publish counts as one admission (at most `PUBLISHES_PER_ADMISSION_AT_MOST` = 8 publishes per admission, `crates/singlefs-core/src/mount.rs`); returning refusal "no space after raising floor" (`UserChangeRefused::NoSpaceAfterRaisingTheFloor`) if still unplaceable when exhausted, successful publishes remaining on disk. Mount resumption draining queue rejected for space likewise prompts floor raises before retrying, budget matching session drain; if unplaceable when exhausted, mount returns writable as usual with queue retained; subsequent user mutations drain queue first, reporting insufficient space if blocked, normal unmount completing normally via slots reclaimed from available section of D16 (Publish Semantics) settled item 1 "normal unmount" row (same settlement as D16 (Publish Semantics) settled item 1 "admission" row). Any version holds at most one key in `deleted_inodes`; branch exceeding single root capacity rejects as typed error, not space exhaustion.
7. **Five idempotency criteria**: (1) Equal plans; (1b) Resuming same batch from identical crash image on separate mounts yields byte-for-byte identical disks (modification count of intention and batches is checkpoint_txg of current publish; intention second fields supplied by caller sharing single value, batches copying, resumed batches taking no new time); (2) Resuming post-deletion issues zero writes; (3) Truncating at every write point and resuming to completion yields final logical content matching non-crashed final state, checker all green on every crash state; (4) Accounting tags 4, 6 match values computed fresh from this image. "Logical content" includes inode entries (number, object birth generation, size, nlink), extent unit sequence set and content checksums, `deleted_inodes` key set, tombstone multiset (with death txg), lineage multiset, tags 12 / 4 / 6, reader answers; excludes instance numbers, txg in root and unit headers, modification counts, placements, unit header birth generations and write sequences, per-disk tags 1 / 2 / 5 / 10 / 11. Harness layer pins absolute values for rewrite role sets per publish (zero rounds) (`research/prompts/m3-delete-r2-main-verification.md` Section 3 V1; `research/prompts/m3-delete-r3-main-verification.md` Section 3 G2). Distinct operation sequences not required to match final logical content (e.g. distinct deletion orders yielding different tombstone death txgs), requiring upper segment shape determined solely by set of inodes with entries, checker all green on every crash state.
8. **Upper segment canonical form**: Upper segment shape determined solely by set of inodes possessing entries, write and delete sharing single rule: nodes formed by union of root paths (chain from leaf to root) of inodes possessing entries; root level taking lowest level covering maximum number; taking Level 0 slot 0 zero-entry root-leaf when set empty; non-root nodes forbidden from zero entries. Within single publish, nodes in new shape along chain of numbers entering/exiting set or modifying entries and newly appearing nodes rewritten, others copied, nodes present in previous version absent from new shape retired; growth copies old root as node below root, reduction copies slot 0 node at reduced level of previous version as root, retiring old root and nodes unlinked above ridge line (extent tree in settled item 14 writes identical shape, `research/prompts/m3-multi-file-unlink-r1-main-verification.md` Section 3 "upper segment canonical form").

**Scope**: First-version deletes any inode by number, no directories, no truncate (user settled 2026-09-28 single file, by inode number, no directories, no truncate; any number relaxed by multi-file deletion verdict); writing data following first file deletion (K-U5, recorded in owed checks) rejects inodes with object birth generation later than inode 1 tombstone death txg only, other pre-existing files writable as usual; retrieving this death txg recognizes tombstones on active parent chain only: "parent chain" defined per I-9.18 (Numbers in deletion queue have tombstones on lineage), ascending lineage records from active version, excluding death txgs in open intervals (T_old, T_rb) abandoned by rollbacks on parent chain, retaining lineage records outside parent chain, taking maximum; unjudged while inode 1 record survives; reading relies on tombstones un-reclaimed in first version (D18 (Information Carried in a Block) settled item 24), requiring alternative reading when reclamation lands (C618 (Cannot create file with data after deletion)); crash stream attached after `crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs` unlinks its 3-unit file (K-U6 F1), truncation to 0 establishing separate stream. Extent branch modifying trailing leaf only, write publishes judged by post-publish tags, and rewrite role sets pinned absolutely attacked zero rounds; clause 6 supplements (session mutation draining first, at most one key per version) attacked zero rounds.

**Basis**:

- No experiment: Numbers measured on three-party replicas (three deletion verdicts); admission harness "deletion borrowing ring window of reserve volume" Step 1 completed on patched replica (2026-10-03), Step 2 awaiting rerun on real implementation deletion batches.
- Three-party verdicts `research/prompts/m3-delete-r1-main-verification.md`, `research/prompts/m3-delete-r2-main-verification.md`, `research/prompts/m3-delete-r3-main-verification.md`.
- User decision 2026-10-01: K-U1, K-U5, K-U6, K-R3-1 (original wording in history).
- Verdict `research/prompts/m3-blanks-r1-main-verification.md` Section 2 queue bound and interface row: rolling back to intention returns number to queue; only two arms ("session mutation drains first" and "resumption post-rollback") preserved queue length and write counts under unrestricted user actions.
- User decision 2026-10-01: Session mutations drain first (`m3-blanks-r1-forks.md` fork sheet "resumption points in session" row, original wording in history).
- User decision 2026-10-02: Session drain rejected for space prompts floor F raise retry; 2026-10-03: each rejected deletion publish counts as one admission, mount resumption rejected for space prompts floor raise retry, mount returns writable per D16 (Publish Semantics) settled item 1 if unplaceable when exhausted, deletion publishes occupy separate transaction numbers (original wording in history).
- Code round verdict `research/prompts/m3-unlink-code-r1-main-verification.md` Section 2 K2 row (resumption, draining, and floor raises) and Section 4 (resolution when mount resumption exhausted handed to user); implementation: session drain floor raise is `publish_the_queue_until_it_is_empty_raising_the_floor_for_space` in `crates/singlefs-core/src/mounted_session.rs`, transaction numbers in `publish_deletion_step` in `crates/singlefs-core/src/transaction.rs`.
- Main agent adopted Q-L 2026-10-02 (`records/2026-10-01-删除与并发事务.md` "16 Batch 8"): overwrites, sequential writes, and unlinks on emptied version reject as existing refusal.
- User decision 2026-10-03: "Unmount must succeed", normal unmount post-exhaustion succeeds via slots freed by D16 (Publish Semantics) settled item 1 "normal unmount" row, single surviving disk taking "A GA (recommended)" (original wording in history); design verdict `research/prompts/m3-unmount-after-short-continuation-r1-main-verification.md`, code round 3rd verdict `research/prompts/m3-unlink-code-r3-main-verification.md` Section 2 n1 rows.
- User decision 2026-10-03: Deletion steps count as modifications ("count as modification, update implementation"), time fields taking "B intention carries time, all three updated together" (original wording in history); code round 3rd verdict `research/prompts/m3-unlink-code-r3-main-verification.md` Section 2 n3 row.
- Verdict `research/prompts/m3-multi-file-unlink-r1-main-verification.md` Section 3 "deleting any number", "upper segment canonical form", "deleting first file" and Section 5 user decision: steps for deleting any inode, upper canonical form, K-U5 taking "β temporal tier" (user decision original wording in history).
- Code round verdict `research/prompts/m3-close-w5-code-r1-main-verification.md` Section 2 W4 and Section 3 clause 1: β taking tombstone death txg recognizes active parent chain tombstones only (attacker measured: delete inode 1, create inode 3, rollback pre-delete, delete inode 1 again, rollback to inode 3 creation, writing inode 3 succeeds, pool checker zero red).

**Owed**: C618 (Cannot create file with data after deletion), C619 (Tombstone tree root growth publish lacks crash stream), C614 (Tombstone tree misselection on rollback and deleted object lacking tombstones unjudgeable on single image); C118 (`deleted_inodes` tree shape lacks placement) cleared.

```

**出处 `.claude/kb/decisions/08-core-index-structure.en.md:513-516`（整段抄，未转述）**

```markdown
## Version History

D8 (Core Index Structure) historical entries are collected in [decisions-history.md](../decisions-history.md).

```

**出处 `.claude/kb/decisions/08-コアインデックス構造.ja.md:1-6`（整段抄，未转述）**

```markdown
## D8 コアインデックス構造 —— 確定

[中文](08-核心索引结构.md) · [English](08-core-index-structure.en.md) · **日本語**

D8 (コアインデックス構造) はコアインデックスがどのような形状を持つかを管轄する：1組の btree 実装に複数の独立キースペースと2つのフロントエンドを組み合わせ、ノードサイズとその上限判拠、キー配置、whiteout 削除と無界操作のバッチ分割、inode ツリーの形態とディスク上の inode レコードフィールド表、write buffer エントリの `seq`、ツリーテーブルエントリのフィールド表とツリー種別コード登録表、コード 2 インデックスノードのペイロード配置とキー比較順序、フロントエンドとノードメッセージバッファの連携、同一パブリッシュ内での同一キーの状態縮小を定める。**B-tree を前提としない**。調査は [prior-art.md](../prior-art.md) 第5節を参照。管轄外：会計統計量のキャリア、キーおよびマージ規則は D5 (スナップショット / 空間会計メカニズム)；複数書き込み可能ヘッド下でキーにスナップショット次元を追加するか否か、スナップショットの実装モデルは D6 (スナップショット実現モデル)；メッセージバッファの ε 配置軸は D11 (インデックスノードにメッセージバッファを残すか)；ユニットヘッダ、ユニット種別コード、パッキングレコードユニットの装填規律は D18 (ブロックに含める情報)；ポインタのフィールドと幅は D19 (ブロックポインタの構造と幅の予算)；パブリッシュセマンティクスとチェックポイント境界は D16 (パブリッシュセマンティクス)；フォーマット凍結の階層は D15 (フォーマット凍結ポリシー) にある。

```

**出处 `.claude/kb/decisions/08-コアインデックス構造.ja.md:7-512`（整段抄，未转述）**

```markdown
### 確定項目

**D8 (コアインデックス構造) の項目には単一の番号体系のみを用い**、並べ替えない。

| # | 項目 | 決定事項 |
|---|---|---|
| 1 | **会計をどのように冪等完全値として表現するか** | 会計は冪等完全値を取り、増分 Δ は用いない；各ルートはその世代の行のみを収容し（一括世代交代、複数世代を残さない）、段差行はルートに随伴して残す **状態: 確定。** **進捗: 実装済み。** |
| 2 | **ノードサイズをいくつにするか** | 16 KiB：mkfs でシステム構成に書き込み、実装は定数に従い、デバイス特性からは算出しない **状態: 確定。** **進捗: 実装済み。** |
| 3 | **キー配置** | key = `(locality_id, inode, offset)`、`locality_id` は作成時に親ディレクトリから継承し、リネーム時には意図的に更新しない **状態: 確定。** **進捗: 実装済み。** |
| 4 | **ノードサイズの上限は何によって決まるか** | 上限はリカバリ時間によって定まり、復旧可能性によっては定まらない；「再スキャンコスト ≤ 目標ライン」の制約下で検索コストが最も低いグレードを実験により選択する **状態: 確定。** **進捗: 実装済み。** |
| 5 | **whiteout 削除** | 墓石を受け入れる；無界な数のエクステントを変更し得る操作はまず意図を書いてからバッチ分割で推進する；第1版の unlink の意図は `deleted_inodes` に入る（確定項目 16） **状態: 確定。** **進捗: 実装済み。** |
| 6 | **inode レコードのディスク上フィールド表** | inode ツリーのリーフはコード 3 パッキングレコードユニット（タイプ 2、1リーフあたり固定長 140 バイトのレコード 233 件）、内部ノードはコード 2、エントリ 120 バイト、バッファなし；パブリッシュ済みの (inode 番号, オブジェクト誕生世代) は再利用しない **状態: 確定。** **進捗: 実装済み。** |
| 7 | **write buffer エントリ付属の `seq`** | 幅 4 バイト、キーではなく value に格納し、射程は会計 / 逆引きインデックス / LRU の3ツリーをカバーする。これにより `T_dirty` の上限はマウント時判定となる **状態: 確定。** **進捗: 実装済み。** |
| 8 | **ツリーテーブルエントリ：ツリー ID の割り当てとエンコード、ツリー種別、ルートポインタ以外の内容** | エントリ合計 200 バイト、ツリー ID は先頭 8 バイト；スナップショットはツリーテーブルに入る；ツリー ID 高水位はルートリング内の全ルートレコードにおける当該フィールドの max を取る；未知の flags ビットは当該エントリを拒絶する **状態: 確定。** **進捗: 未実装。** |
| 9 | **ツリー種別コードの登録表** | コード 0 はなし、1..8 は8つの既存ツリー、9..15 は deleted_inodes、墓石ツリー、系譜ツリー、logged_ops、退役ツリー、スナップショット inode ルート、スナップショット extent ルート；新規ツリーは追加のみとしコードは再利用しない **状態: 確定。** **進捗: 実装済み。** |
| 10 | **第1版の会計が write buffer フロントエンドを通るか否か；`seq` の値生成規則** | 第1版の会計は直接会計ツリーのリーフに書き込み、フロントエンドは通らない；`seq` = 単一パブリッシュ内で同一キーがツリーに書き込まれた回数（1から開始）、第1版では常に 1 **状態: 確定。** **進捗: 実装済み。** |
| 11 | **コード 2 インデックスノードのペイロード配置、種別からエントリ幅への対応表、キー比較順序** | コード 2 ヘッダは自己記述、基本 86 バイト、エントリ領域は予約領域を含むヘッダ末尾から固定幅で連続配列、キーはエントリの先頭 key 幅バイト **状態: 確定。** **進捗: 実装済み。** |
| 12 | **write buffer フロントエンドとノードメッセージバッファの連携** | 1つのツリーには1レベルのみ許可：フロントエンドを通る3ツリーは ε が常に 0、書き込みは一律フロントエンドに入り、パブリッシュ前にリーフへドレインする；第1版では両レベルとも無効 **状態: 確定。** **進捗: 未実装。** |
| 13 | **インデックスツリー更新で状態縮小を行うか否か** | 単一パブリッシュ内で同一キーが複数回変更された場合、リーフへの書き込みは1回のみ行い、最終状態 E を直接書き込む。中間状態はいかなるノードにも書き込まれず、E は到達順序により認識する **状態: 確定。** **進捗: 実装済み。** |
| 14 | **インデックスの全体構造** | 1組の btree 実装、複数の独立キースペース、2つのフロントエンド（write buffer、key cache）；派生状態の割り当てレコードツリーと extent ツリーはキー空間によりアドレッシングされる別構造を採用可能 **状態: 確定。** **進捗: 実装済み。** |
| 15 | **write buffer エントリの3つの厳格要件** | ノードキャッシュは耐スキャン性を備えなければならない；エントリは冪等完全値でなければならず増分 Δ は不可；エントリは自前のシーケンス番号 `seq` を持たねばならず、これはフォーマット要件である **状態: 確定。** **進捗: 未実装。** |
| 16 | **第1版 unlink の削除方式** | 第1版 unlink：`deleted_inodes` のみを立て、末尾から 144 ユニット単位でバッチ処理し、マウント内で空になるまで継続実行する **状態: 確定。** **進捗: 実装済み。** |

#### 確定項目 1：会計は冪等完全値を取り、一括世代交代とする

**決定事項**：**会計エントリは一律に「ある世代における特定の統計量の完全値」とし**、書き込み時にはセル全体を上書きする；**増分 Δ は用いない**。マージ規則は統計量ごとに分類される（D5 (スナップショット / 空間会計メカニズム) 確定項目 3 第 3 行、登録表は D5 (スナップショット / 空間会計メカニズム) 確定項目 10 を参照）：非単調統計量は「後勝ち」（後は `seq` によって決定される）、単調統計量は最大値を取る。最大値を取ることも同様に冪等、可換、結合的であり、本項の論証（冪等完全値が増分 Δ に勝る）は影響を受けない。一括世代交代：各ルートの会計ツリーはそのルート自身の世代の行のみを収容し、複数世代を保持しない（inode 番号高水位の段差行がルートに随伴して保持される点を除き、D5 (スナップショット / 空間会計メカニズム) 確定項目 2）；キー数は世代数に伴って単調蓄積することはなく、2ディスクのプール上では段差行を除き平常時 15 行である。

**対象範囲**：本項が定めるのは会計エントリの値のセマンティクスである；キャリア（会計が独立キースペースの btree に常駐し、エントリが「(統計量, 次元タプル, 世代) → 完全値」であること）は D5 (スナップショット / 空間会計メカニズム) にある。⚠️ **キー数は世代数に伴って単調蓄積しない**：各ルートは自身の世代の行のみを保持し（D5 (スナップショット / 空間会計メカニズム) 確定項目 2、C548 (会計ツリーのパブリッシュごと一括世代交代が統計量ごと世代ごと1行と矛盾)）、2ディスクのプール上では段差行を除き平常時 15 行であり、80 ディスク以上になって初めて分割される。⚠️ **フォールバック**：ある種の会計でどうしても冪等性を達成できない場合（本項はそのような分類が存在するかを事前予判しない）、その分類はランタイムと**判定コードを共有しない**独立した再計算パスを別途持たなければならない。⚠️ 本項は同時に D11 (インデックスノードにメッセージバッファを残すか) のコスト 1 の成立条件を変更する——そのコストの判拠は「バッファに置くのが冪等完全値か増分 Δ か」であり、本項はその答えを冪等側に固定する ⇒ **コスト 1 は本プロジェクトにおいて成立しない**；D11 (インデックスノードにメッセージバッファを残すか) のコスト 2（再構築性の低下）は影響を受けない。

**根拠**：

- E14 (I-3.1 の判別力：冪等完全値 vs 増分 Δ)：同一ツリー同一負荷で4種のセマンティックバグを注入したところ、冪等完全値形態では I-3.1 (割り当て済み統計の合致) がすべてを捕捉しブラインドスポットゼロであったのに対し、増分 Δ 形態では1件も捕捉できなかった——Δ 形態では会計自体が権威的内容であり、checker は同一の累積関数で同一の Δ ログを再生するしかなく、自身と比較しているに過ぎない。
- ユーザー決定 2026-08-29：冪等完全値を採用し、キー数の増加を受け入れる（原語は変更履歴）。

**未済**: C15 (write buffer への Δ 混入)。

#### 確定項目 2：ノードサイズは 16 KiB とし、定数として固定する

**決定事項**：**ノードサイズは 16 KiB とする。mkfs は 16 KiB をシステム構成に書き込み、実装は定数に従い、デバイス特性からは算出しない**——フォーマット内では1つの定数であり、計算式ではない。
<!-- format-const: NODE_BYTES = 16384 -->

**対象範囲**：⚠️ **`format-const` の登録はこの数値を凍結することを意味しない**（2026-08-31 ユーザー決定：「この数値は調整可能である必要がある」）。そのマーカーは単に **kb と実験ソースコードが同一の数値を書いていること**を保証するに過ぎず、その変更は通常通り D8 (コアインデックス構造) のプロセスに従う；システム構成に書き込まれるが、依然として mkfs パラメータである。⚠️ **現在固定できているのはソースコードの一部のみである**：ゲート段階の「フォーマット定数の kb と実験ソースコード間の同期」（`.claude/gate.d/code-source-discipline.sh` の format-constants セル）の形態は `const 名 = 整数リテラル` であり、`e56_epsilon.rs` の `16 * 1024`、`e43_extension_point_budget.rs` および `e52_head_mechanisms.rs` の配列形式は認識されない。⚠️ **16 KiB は CPU キャッシュ軸上の選択であり、デバイス I/O 軸上のものではない**：E7 (オフラインインデックス harness) でデバイス I/O を計測した際、小さいノードが両端で勝ったが方向は逆であり、両者は異なるリソースを計測しているため矛盾しない。上限の次元は確定項目 4 に属する；E1 (ノードサイズ vs 復旧可能性) のリカバリ時間の半分が 16 KiB より厳しい上限を示した場合、本項は再審議となる。

**根拠**：

- E20 (ポインタ拡幅の CPU キャッシュコスト)：L3 を超える唯一のワーキングセットにおいて 16 KiB が最小ポイントであった；同一実験内で 40 バイトのポインタヘッダが 4 KiB ノードで支払う相対コストは 16 KiB ノードより遥かに高く、大ノードがポインタ拡幅を吸収する；同一曲線上で 8 KiB の異常値が6回にわたり安定して発生し説明されていない ⇒ それを説明しきる前にノードサイズを「デバイス特性に従って計算」と書くことは禁じられる。
- E6 (暗号化アルゴリズム選定)：暗号化 / チェックサムユニットの利得は 16 KiB 以降ほぼ飽和する。
- E73 (ノードキー区間のファンアウトコスト)：コード 2 ヘッダにキー区間を持たせた後もツリー高は1段も増加せず、16 KiB を再検討する必要はない。
- ユーザー決定 2026-08-30：16 KiB を定数として固定（原語は変更履歴）。

**未済**: C56 (未登録のフォーマット定数が漂流する)、C386 (ノードサイズ結論が最終ポインタ幅による再実行を欠く、番号なし)。

#### 確定項目 3：キー配置 —— リネームとインデックスは分離して処理する

**決定事項**：**key = `(locality_id, inode, offset)`**、`locality_id` は作成時に親ディレクトリから継承し、**リネーム時には意図的に更新しない**。offset セグメントは**ファイルバイトオフセット**である（このユニットの先頭バイトのファイル内オフセットであり、0 から始まる第 n ユニットは n × 正味ペイロード容量 32634）。

- 命名は dirent ツリー（`parent, name` → inode）に従い、リネームは1件削除して1件追加するだけであり、**O(1)** である
- データは inode に従ってキーが編成され、名前とは完全に切り離される
- scan の局所性は継承された `locality_id` に由来し、リネーム後**局所性は緩やかに劣化するが、正当性は影響を受けない**
- 劣化は D3 (領域割り当て) で約束済みの常駐バックグラウンド整理によって修復され、新規の負担ではない

**一括リネーム**：この設計において `mv 大ディレクトリ 別場所` は **dirent を1件変更するだけ**である。3箇所においてそれが再び無界操作へと逆戻りする恐れがあり、最初から塞いでおかなければならない：

1. **`locality_id` はリネームによって更新されてはならない。** それは**ヒントであり、正当性の一部ではない**——誤りであってもよく、古くなっていてもよい。
2. **既存のターゲットを上書きするリネーム = 1回の削除**であり、必ず確定項目 5 (whiteout 削除) の intent log を通らねばならない。
3. **バックグラウンドの再クラスタリングは増分的、中断可能、再開可能でなければならず**、いつでも停止して取りやめることができなければならない。

**推論（レビュー判拠として利用可能）**：**リネームが子孫に触れるようないかなる設計変更も、一律に否決する。**

**inode レコードは `locality_id` のコピーを1部保持しなければならない**：それはデータキーの先頭セグメントでありながらリネームで決して更新されず、命名ツリーの値には inode しかなく、データユニットヘッダの5タプル内にも存在しないため、ディスク上ではキー自身のみがそれを保持している——読者がリネームされたファイルをオープンする際、どこかからそれを取得できなければならず、さもなければリネーム後のデータはスキャンによってしか見つけられなくなる。配置先は確定項目 6 (inode レコードのディスク上フィールド表) である；この値は本項に従い親ディレクトリから継承されるが、第1版にはディレクトリが存在しないため、「親ディレクトリがない場合に何を取るか」も合わせて定める必要がある——第1版では 0 を取り、確定項目 6 のレコードフィールド表に登録する。スキャン再構築時に再編成されるキー先頭セグメントは、inode レコード > 残存 extent キー > 0 の優先順位に従ってそのオブジェクトの全キーに同一の先頭セグメントを割り当てる（2026-09-16 ユーザー決定、D18 (ブロックに含める情報) 確定項目 3 のフィールド追加提案は却下）。

**対象範囲**：データキーの編成方法およびリネームが子孫に波及しない方法を管轄し、命名ツリー自体のキーは管轄外である（dirent ツリーは `(parent_inode, name)` で編成、確定項目 5 参照）。scan の局所性は必ずしもキー順序に常駐する必要はない：キー順序に常駐させる（フルパスインデックス、リネームは O(部分木深度)）、アロケータに常駐させる（子ディレクトリのデータを親ディレクトリ近傍に配置）、セカンダリインデックスに常駐させる（依然高価で整合性の維持が余計に必要）ことが考えられるが、本プロジェクトでは「誕生時に継承しリネームで更新しない locality プレフィックス」を採用する（リネームが無料であり優れている）。⚠️ 「ヒントであり誤りでもよい」が許容するのは**その意味を信じないこと**であり、**そのビットを保存しないこと**ではない：アドレス先頭セグメントとしてその値は正確かつ定義的である。⚠️ **E9 (キーエンコードが走査局所性に与える影響) の走査局所性の数値は単一の計測条件の下でのみ成立し、引用時は必ず計測条件を併記しなければならない**：**インターリーブ作成**（`interleave`、経年劣化の代理）**に `cache_leaves = 8` を加えた条件**；`locality_id` の利得は**新しいツリーでの利得**であり、定常状態での利得ではない。リネーム強度の現実性は較正されていない——曲線は使用可能だが、絶対目盛は使用不能である。⚠️ フルパスインデックスの走査優位性も測定されたが、**それを D9 (暗号化) の漏洩面や D1 (データ移動性 / 逆引きインデックス) の安定性の判断に流用することは禁じられる**——E9 (キーエンコードが走査局所性に与える影響) はそれら2点について一言も言及していない。⚠️ 「リネーム1回あたり平均約 2700 個のキーを書き直す」という数値は**計測条件を欠いており**（どのリネーム階層か、どのシードかが追跡不能）、未検証と標記される；同表の ext4 Orlov アロケータの「カーネルツリー走査が 30% 高速」も同様に未検証である——kb 内に出典も計測条件もなく、使用前に補完を要する。⚠️ 3セグメントキーは D6 (スナップショット実現モデル) が複数書き込み可能ヘッドを採用した後に一時不完全となったが（2つの書き込み可能ヘッドがそれぞれ同一の `(locality_id, inode, offset)` を書き込むと同一のキースロットに落ち、後書きが先書きを上書きする）、2026-08-30 に D6 (スナップショット実現モデル) 確定項目 1 が「各ヘッドに自前のツリー1本」を採用したことで解消された——ヘッドごとに1本のツリーがあるため、3セグメントキーはツリー内で一意となり、**2つのヘッドが同一のキースロットに落ちることはなくなる**。⚠️ **nonce に関する義務は依然として存在する**：D9 (暗号化) 確定項目 4 で nonce はエクステントごとに保存すると確定しており、**生成時には2つの書き込み可能ヘッド間で衝突しないことを依然として保証しなければならない**（候補形態はヘッドごとに nonce 空間を分割すること）。これは識別子の一意性から自動的に得られるものではなくなり、さもなければ I-6.1 (nonce の非再利用) は「識別子の一意性により保証される」から「生成器により保証される」へと変わり、誰も保証しなくなる；この義務は推論であり、三者検証を経ていない。⚠️ バックグラウンド再クラスタリングが物理位置のみを移動するのかキーを再編成するのか、再編成時のバッチ分割方法、進捗の再開位置はリポジトリ全体で定義されていない。

**根拠**：

- E9 (キーエンコードが走査局所性に与える影響)：同一ディレクトリツリー同一 LRU 上で4本のアームを比較したところ、`locality_id` プレフィックスは inode 単位のキー編成よりもデバイス読み取り回数が少なく、実施価値がある；depth-first 作成時には3つの実アームがビット単位で一致し、キャッシュがツリー全体を収容できる場合は4アームすべてが同等となり効果は消失する；利得はリネーム回数とともにほぼゼロに減衰する。
- メカニズム論証（2026-08-25 決定時に記述、本文は変更履歴）：フルパスインデックスは本プロジェクトにおいて BetrFS が支払う必要のない3つの追加コストを支払うことになる——区切りキーがパスそのものであると D9 (暗号化) 確定項目 5 が認定した機密情報を1本のツリーからパスでキー編成される全ツリーへと拡大させてしまう、参照元識別子がパスであると D1 (データ移動性 / 逆引きインデックス) の逆引きインデックスエントリがリネームに伴いすべて無効化される、キーエンコードに対する lifting の追加要件が D9 (暗号化) の nonce キャリアおよび D6 (スナップショット実現モデル) のスナップショット ID 位置と同一のキー空間を奪い合う。
- ユーザー決定 2026-09-16：スキャン再構築時、1つのオブジェクトの全エクステントキーに同一の先頭セグメントを割り当てる（原語は変更履歴）。
- E157 (並行線1、2の条項の計数モデル) の計測：1ページを特定するためのノード読み取り回数、ツリー高とリーフ数、容量変更後のバイト区間復元セルにおいて、2アームは全サンプリングポイントで点ごとに一致し区別がつかなかった；唯一区別できたのは AAD アンカーのセルであり、ファイルバイトオフセット下では N 個のキーがすべて正しく計算されたのに対し、ユニット番号下では1個しか正しく計算されなかった。
- D9 (暗号化) 確定項目 6 の原文「アンカーオフセット = 当該エクステントが初めて書き出された際のオブジェクト内開始オフセット、読者は `key.offset − ptr.extent_off` をその場で計算」、および同一決定の確定項目 12 第2項が AAD フィールド構成は day-1 恒久契約であると明記 ⇒ 条項はもともとこの単位を前提としていた。

**未済**: C346 (バックグラウンド再クラスタリングが未定義)。

#### 確定項目 4：ノードサイズ —— 上限はリカバリ時間によって定め、実験によって値を決める

**決定事項**：大きなノードは seek を節約しツリーを浅くするが、**再スキャンコストはノードサイズに比例して線形に拡大する**ため、**128K–256K を無批判に踏襲しない**。**ノードサイズが交換するのは「リカバリ時間」であり「データ安全性」ではないため、目標ラインはリカバリ時間に設定すべきである。値の選択は E1 (ノードサイズ vs 復旧可能性) に依存する：「再スキャンコスト ≤ 目標ライン」の前提下で検索コストが最も低いグレードを選択する。** 3つの制約：

1. **これは mkfs 時のフォーマットパラメータであり、マウントパラメータではない。** 自動選択は mkfs 時に1回だけ発生する。これを変更する = **インデックスの再構築**であり、ファイルシステムの再構築ではない（D21 (権威態と派生態の境界) でインデックスは派生状態と決定済みであり、凍結レイヤ 2 はローカルキャッシュフォーマットに格下げされている）。
2. **デフォルトポリシーは決定的かつ調査可能でなければならず**、mkfs は「なぜこの値を選択したか」を出力しなければならない。
3. **テストが固定するのは「模擬デバイスのパラメータ」であり、ノードサイズ自体ではない**——固定された一連のデバイスタスクにおいて自動ルールを通常通り実行させ、**選択ルール自体もテスト対象とする**。要件は1つのみ：同一グレードでは同一の値を得なければならない。

**対象範囲**：⚠️ **判拠は「全ディスクスキャン再構築成功率 ≥ 目標ライン」ではない**——インデックス破損の恒久損失は常に 0 である ⇒ 成功率の次元はノードサイズと**無関係**であり、それを判拠にしても上限を絞り込むことはできない。⚠️ **目標ライン自体は未定である**。それはリカバリ時間予算（どれだけの時間内に破損部分を再スキャンして復元するか）であり、本項単独では算出できない；E1 (ノードサイズ vs 復旧可能性) の残りの半分は依然として mkfs、checker、およびイメージをピンポイント破壊できるツールを欠いている。⚠️ 前提条件：インデックスはユニットの自己記述から再構築可能である——E28 (平文マッピング層はユニット自己記述のみで再構築可能か) の実測では**メタデータ側で成立し、データブロック側で成立しなかった**（不足数はデータブロック総数に正確に一致、D18 (ブロックに含める情報) 確定項目 3 の未済）；⚠️ **「変更する = インデックス再構築」が取り消すのは定性的な性質であり、コストではない**：新サイズによるツリー全体の再構築はフルスケールの全ディスク I/O であり、データが失われないだけである；さらにこの取り消しには未達の前提がある——データブロックの自己記述が補完されるまで、「ファイルシステム再構築に等しい」はデータブロック側で依然成立する。ブリッジの根拠は、インデックスツリー再構築と平文マッピング層再構築が**同一の入力を読み取る**ことである（各ユニットが自前で「私は誰か、誰に属するか」を保持している）。⚠️ この一歩は推論であり、E28 (平文マッピング層はユニット自己記述のみで再構築可能か) は平文マッピング層のその1インスタンスのみを測定した；D11 (インデックスノードにメッセージバッファを残すか) が「先延ばしにできない」とした全理由はまさにこの半文にある。⚠️ **第1期ではバイト主導の側のみが射程内にある**：D12 (対象メディア) 確定項目 5 で第1の配置ラインは純 SSD と確定 ⇒ 結論は不変、**支柱は両端から一端へと変わる**；シーク主導の側は HDD 配置ラインが開始されて初めて再び射程に入る。⚠️ **ノードサイズの自動選択判拠にはポインタ幅の項目を含めなければならない**（E20 (ポインタ拡幅の CPU キャッシュコスト)：大ノードはポインタ拡幅を吸収し、方向は再スキャンコストと逆）、E20 (ポインタ拡幅の CPU キャッシュコスト) 以前には存在しなかった。⚠️ **「デバイス特性に応じて逆方向を取る」という理由は成立しない**：両端の最適方向は同一であり、いずれも小さめである；ノードサイズは依然として mkfs パラメータであり、メディアに応じた調整を残すには別の根拠が必要である。

**根拠**：

- E29 (ノード1個破損の爆発半径)：インデックス破損の恒久損失は常に 0（D21 (権威態と派生態の境界) でインデックスは派生状態と確定、消失しても遅くなるだけ）、データユニット破損の損失はノードサイズと無関係であり、再スキャンコストのみがノードサイズに比例して線形拡大する（第1層ノードの再スキャン量はファンアウトに正確に一致）——したがって上限はリカバリ時間によってのみ定めることができ、「爆発半径」という表現はこれによって是正された。
- E7 (オフラインインデックス harness)：**固定メモリ予算下では大ノードは3軸すべてで劣る**（io/op、バイト/回、ポイントクエリ読み取り/回を計測）。メカニズムは、ノードが大きいほど同一メモリに収容できるノード数が減り、ヒット率の低下が「1回の読み取り量の増加」による利得を上回るためである；シーク主導の側も方向は同一で大ノードは依然として劣る ⇒ 「128K–256K を無批判に踏襲しない」はバイト主導とシーク主導の両端で支持された。
- ユーザー決定 2026-09-06：計測条件を「上限はリカバリ時間によって定める」に書き換え（原語は変更履歴）。

**未済**: C386 (ノードサイズ結論が最終ポインタ幅による再実行を欠く、番号なし)。

#### 確定項目 5：whiteout 削除 —— 許容するが、バッチ削除は必ず intent log を経由する

**決定事項**：大ディレクトリの削除や大ファイルの切り詰めは大量の墓石を生成し、単一のトランザクションに詰め込むと確実にパンクする。**手法（bcachefs で検証済みの形態）**：**無界な数のエクステントを変更し得る**操作（truncate、collapse、大ディレクトリ削除）は、**まず意図を logged_ops ツリーに書き込み、その後にバッチ分割で推進する**；クラッシュ後のリカバリ時に未完了の意図を継続実行する。削除処理中の inode は単独で `deleted_inodes` ツリーに入り、バックグラウンドで回収される——ユーザーの `rm` が返ることは、全エクステントが解放済みであることを意味しない。第1版の unlink は logged_ops ツリーにエントリを書き込まない：意図とは inode レコードの nlink = 0 に `deleted_inodes` の1件を加えたものである（確定項目 16）。logged_ops ツリーは最初のファイルバージョン作成時に `deleted_inodes` と共にゼロルート登録され、フィールド表は確定済み（D26 (バックグラウンド整理と配置回収) 確定項目 2 の整理意図エントリのフィールド表）、ロールバック時にどの版を取るか、状態の区別は最初のライター（整理）の着工時に定める（C364 (整理インテントレコードのフィールドが逐一突き合わせされていない)）。意図は2種に分かれる：**コミット型**（unlink；truncate、大ディレクトリ削除）は最後まで継続実行しなければならず、書き込み可能マウントが返される前に空になるまで進める；**ヒント型**（整理）は一時停止可能で、停止述語の下で早期回収でき、ディスクに渡すライン上では継続実行せず、マウントを妨げない；継続実行のエントリはツリーごとに分かれ、パブリッシュパスを共用する。確定項目 5 第2条と三状態表、C1 (意図継続実行の通常パス化) の字面はコミット型に従って書かれており、整理着工時に分類して書き分ける（C621 (整理着工時に触れる削除条項にリスト外の検査がない)）。

**厳格要件**：無界規模の操作を一律に単一トランザクションにしてはならない；意図は必ず**冪等**でなければならない；whiteout はノード書き換えによる bset マージ時のみクリアされ、**他のクリアパスを許容してはならない**。

意図自体は通常のキーであり、同一のトランザクション層に乗るため、電源断後に生じる状態は3つのみである：

| 電源断のタイミング | リカバリ後に見える状態 | どのように収拾するか |
|---|---|---|
| 意図コミット前 | 意図なし | 操作は一切発生しておらず、ユーザー側にも成功は返っていない |
| 意図コミット済み、作業半ば | 意図あり | リカバリ時に**継続実行**し、完了済みの部分は冪等に再実行される |
| 作業完了、意図未削除 | 意図あり | 継続実行して作業が残っていないことを確認し、意図を削除する |

「意図が消えたのに目標状態が達成されていない」という事態は生じ得ない——**意図の削除は最後のバッチ作業と同一トランザクション内でコミットされなければならない**。

1. **意図は目標状態を宣言し、増分は決して宣言しない。**「ファイル X を N バイトに切り詰める」は反復実行できるが、「さらに 1000 エクステント削除する」はできない。
2. **継続実行はリカバリごとに毎回実行される通常動作であり、修復手段ではない**（さもなければ「普段は走らず、真の障害時に初めて走る」コードパスが生じてしまう）。
3. **中間状態はすべて正当なファイルシステム状態でなければならない**——「エクステントの半分が削除されたファイル」自体が正当でなければならない。
4. **バッチ内の各アクションは適用時に削除対象がまだ存在するかをまず検証する**（D5 (スナップショット / 空間会計メカニズム) 確定項目 12 に付随）——冪等の字面：バッチ開始時の状態に基づいて複数の意図に対してそれぞれ計画を算出し、適用時に再検証しない場合、重複する2つの区間のカスケードにより同一ブロックが2回解放され、統計量が2回減算されて I-3.1 (割り当て済み統計の合致) が赤となる。

**範囲再構築 vs 項目ごと変更：双方が必要であり、明示的な判拠を持たねばならない**（`.claude/rules/fs-design.md`「万能の解決策を追求しない」）：

| アプローチ | コスト | 適用対象 |
|---|---|---|
| 項目ごと変更 | K × ランダムアクセス | 影響を受けるキーが疎に散布している |
| 範囲再構築 | N × シーケンシャルアクセス | 影響を受けるキーがキー順序上で連続している |

**真のスイッチは K/N ではなく、「影響を受けるキーが連続しているか否か」である**：連続している ⇒ 範囲再構築によりノード全体を破棄または書き換え可能；不連続 ⇒ 項目ごとに処理するしかない。**これより配置規則が得られる：共に破棄されるものはキー順序上で隣接させる。**

- dirent は `(parent_inode, name)` でキー編成 → ディレクトリ全体の削除は範囲操作となる
- extent は `(inode, offset)` でキー編成 → truncate は範囲操作となる
- 同一スナップショットのキーは同一の snapshot プレフィックスを保持 → スナップショット削除が範囲操作となる余地がある

**これは付随して構造的に whiteout の肥大化を解消する**：範囲削除はノード全体を直接破棄するため、N 個の墓石を挿入する必要がない。

**対象範囲**：素朴な交差点は `K/N > シーケンシャルコスト / ランダムコスト` である（機械式ディスクで約 1%、SSD で約 10%）。⚠️ これら2つの数値は**推論であり実測ではない**。**所要時間**について述べており、E4 (範囲再構築 vs 項目ごと変更の交差点) で計測する必要があり、その半分は QEMU を要する。⚠️ **第1期では SSD のグレードのみを採用する**（D12 (対象メディア) 確定項目 5：第1の配置ラインは純 SSD）。⚠️ ライトアンプリフィケーションの半分で測定された交差点の独立変数はメディアではなく「配置」と「ノードサイズ」であり、これをもって「機械式ディスク約 1%、SSD 約 10%」を反証することはできない——それは所要時間を述べているためである。`.claude/rules/fs-design.md` の5つの要件に従い、この分岐には未済が残る：判拠を計算可能な式として書くこと、指定分岐を強制できるテストスイッチを残すこと、2つの分岐でそれぞれクラッシュポイント再生を実行すること、ランタイムでどちらを通ったか確認できること。⚠️ **切り替え判拠に「現在の充填率」のような操作途中で変動する量を含めてはならない**——判拠には操作期間中に安定している入力のみを使用できる。**D3 (領域割り当て) との相互作用（見落としやすい）**：意図未完了の間、それらの領域はまだ実際には解放されておらず、領域統計は「削除待ちだが未完了」を計上しなければならず、さもなければ「大量に削除したのに `df` が動かない」事態が発生する。

**根拠**：

- E30 (交差点のライトアンプリフィケーション側の計測)：**ライトアンプリフィケーション側は計測済み**——ライトアンプリフィケーションの計測条件の下で、エントリ連続とエントリ散布の2種の配置における 4 KiB / 16 KiB ノードでの交差点を計測したところ、両者は2桁以上異なっていた——スイッチは配置とノードサイズであり、メディアではない。
- ユーザー決定 2026-09-16：第4の厳格要件「バッチ適用時に削除対象がまだ存在するかまず検証」を追加（原語は変更履歴）。
- 三者レビュー `research/prompts/m3-intent-r1-main-verification.md`：意図をコミット型とヒント型に分類（2系統が独立して導出、1巡）；logged_ops ツリーのゼロルート登録はユーザー決定 K-I1（原語は変更履歴；0巡攻撃済み）。

**未済**: C2 (無界操作の単一トランザクション化)。

#### 確定項目 6：inode ツリーのリーフはコード 3 パッキングレコードユニットとし、レコードは固定長 140、パブリッシュ済みの (inode 番号, オブジェクト誕生世代) は再利用しない
<!-- doc-lint:not-numbers P1 P2 P3 P4 P5 P6 P7 P8 P9 -->

**決定事項**：inode ツリー = コード 2 内部ノード（派生状態、メッセージバッファを残さない、エントリ 120 バイト、ファンアウト 135）+ コード 3 パッキングレコードユニットのリーフ（パッキングレコードタイプ 2、権威状態ユニット、1リーフあたり 140 バイトのレコード 233 件）；ルートは常にコード 2；コンテナインデックスはこのツリーの内部ノードそのものである；パブリッシュ済みの (inode 番号, オブジェクト誕生世代) は再利用せず、次の番号は会計内の最大値マージを取る統計量である。コード 2 ルートが満杯になると1層高くなる——新ルートにコード 2 が1個、分割された右半分にコード 2 が1個（右半分はコンテナ識別子を持たない）。inode ツリーと墓石ツリー（D18 (ブロックに含める情報) 確定項目 24）はこの条項を共用する；高くなるパブリッシュでは 16 KiB ノード × レプリカ数が2個多くなる。ルートが高くなる前のコードは `crates/singlefs-core/src/inode_tree.rs` の `MoreLeafContainersThanOneRootNodeHolds` で拒絶する（推論されたユニット数、0巡）。

| コンポーネント | 実体 | 状態種別 |
|---|---|---|
| リーフ | 1個の 32 KiB コード 3 パッキングレコードユニット（D18 (ブロックに含める情報) 確定項目 11）、パッキングレコードタイプ 2、140 バイトのレコード 233 件を収容；クラス識別子セグメント、レコード数、レコード幅、ペイロードチェックサムを自前で保持し、スキャンパス上のペイロード完全性の証拠（C104 (スキャンパス上のコード 1 / 2 のペイロード完全性に自己完結した証拠がない) によるコード 2 の欠落）はこれに対して存在しない | 権威状態（ユニット） |
| 内部ノード | コード 2 インデックスノード。ヘッダにキー区間（min / max 各 8 バイト、D18 (ブロックに含める情報) 確定項目 2 が inode ツリーに残したセルはこれにより「保持」と決定：理由は会計ツリーと同一——1ノードを単独で拾い上げてもカバレッジセグメントを自証でき、キーはファイル名ではなく inode 番号であり機密性論証が及ばないため）。**ヘッダはブロックヘッダであり、D15 (フォーマット凍結ポリシー) 第1層（凍結）に該当し、会計ツリーノードヘッダと同一セル（C102 (インデックスノードヘッダに予約領域がなくフィールド追加が compat でない)）；ペイロード配置（エントリ 120、ε = 0）は第4層に該当し、恒久的ではない** | 派生状態、消失時はリーフから再構築 |
| ルート | 常にコード 2 インデックスノードであり、ツリーのリーフが1枚のみであっても、ツリーが空（エントリ0件、キー区間に [0, 0] を書き込み、0 は無効 inode 番号）であっても同様。理由：D19 (ブロックポインタの構造と幅の予算) のポインタは指示対象のクラスとサイズを保持せず、ツリーテーブルポインタが指すルートのクラスがツリーの成長に伴って変化すると、読者が 16 KiB のつもりで 32 KiB を読みサイレント誤読となる | 派生状態 |
| コンテナインデックス | このツリーの内部ノードそのものであり、別個のツリーを立てない：物理ポインタは内部ノードにのみ常駐し、dirent ツリーが参照するのは inode 番号である。1件のレコード更新 = COW リーフ + 全祖先（D23 (ジャーナルの役割とフォーマット) 確定項目 1 甲） | — |

**内部ノードエントリ = 区切りキー 8 + 識別子参照 26 + 子ポインタ 86 = 120 バイト**（ファンアウトはヘッダ 131 で計算すると ⌊(16384 − 131) / 120⌋ = 135）。識別子参照は D18 (ブロックに含める情報) 確定項目 11 が「他のツリーがコンテナを参照する」ために与えた26バイトそのものである：誕生ツリー 8 + パッキングレコードタイプ 2 + コンテナ番号 8 + コンテナ誕生世代 8。**タイプセグメントは「子ノードが何であるか」を兼ねる**：0 ⇒ 子ノードはコード 2 インデックスノード（コンテナ番号、誕生世代の2セグメントは常に 0；登録表の 0 はコンテナヘッダ内では依然として無効であり、2箇所の意味は所在構造によって区別される）、2 ⇒ 子ノードはコード 3 コンテナ（16 KiB スロットを2個読み取る）；その他の値はエントリ破損と判定する。正当な集合はツリーごとに異なる：inode ツリー {0, 2}（I-9.2 (エントリ識別子と子ヘッダの合致)）、墓石ツリー {0, 1}（I-9.23 (墓石ツリーエントリ識別子と子ヘッダの合致)）、系譜ツリー {0, 5}（I-9.17 (系譜ツリーエントリ識別子と子ヘッダの合致)）、退役ツリー {0, 3}（不変条件は第50バッチと退役ツリーの実装に伴い設定、判決 `research/prompts/m3-retire-r1-main-verification.md` 第3節第8条）、4ツリーの内部エントリは同一形態である。読者はこれにより何個のスロットを読むべきかを知る；コード 3 の側の AAD 5セグメント（ラベルはタイプ ≠ 0 より導出、残り4セグメントはエントリをそのまま複製）はすべて探索パスから得られ、D9 (暗号化) の A6 (AAD 期待値が平文側に由来しない) はこのエッジにおいてツリー種別を必要としない；コード 2 の側のエッジ（内部ノードが内部ノードを指す）の AAD は依然として C107 (インデックスノードの AAD 識別子に配置先がない) である。誕生ツリーがエントリごとに保持されるのは、クローンヘッド（D6 (スナップショット実現モデル) 確定項目 1）が origin のリーフを共有し、同一の内部ノード内に誕生ツリーの異なるエントリが混在するためである；D19 (ブロックポインタの構造と幅の予算) 確定項目 7 で子ポインタヘッダにも誕生ツリーを持たせた後も、識別子参照内の誕生ツリーは**従来通り保持される**——2箇所が並存し、暗号化の有効無効にかかわらず領域を占有し、他目的に流用されない；タイプセグメント 0 のエントリの識別子参照内の誕生ツリーは常に 0 であり、コンテナ番号、誕生世代の2セグメントと同形である；コード 3 の側の AAD の誕生ツリーは識別子参照の側を取り、子ポインタの側は取らない。**区切りキー** = その子が作成された際の最小キーであり、左が右から借用したパブリッシュでは借用された右の子の現行バージョンの最小キーに変更される；エントリは区切りキーに従って単調増加し、検索は区切りキー ≤ 目標キーを満たす最後のエントリを取得する。
<!-- format-const: INODE_INTERNAL_ENTRY = 120 -->

**内部ノードにはメッセージバッファを残さず、inode ツリーの ε = 0 とする**——これは D11 (インデックスノードにメッセージバッファを残すか) 確定項目 6 のツリー軸に対する補足である（配置軸「全配置で同一の ε グループを共有」は変更なし）。分岐変数は D11 (インデックスノードにメッセージバッファを残すか) 確定項目 4 の述語である：**このツリーの下方フラッシュ待ちメッセージが権威状態（ユニット（書き込み順序付き）+ インスタンス表 + 祖先表 + 墓石ユニット）のみから再計算可能か否か**。可能 ⇒ バッファを残す（ε = 0.65）；不可 ⇒ 残さない。inode ツリーは不可である：属性更新（mtime / nlink / mode の新値）はリーフに到達するまで権威状態のキャリアを持たず、派生状態ノードにバッファリングすると D21 (権威態と派生態の境界) 厳格制約 6 に抵触する。extent / dirent / 逆引きインデックスは可能である（E59 (バッファメッセージはユニットのみから再計算可能か) の「ユニット + 会計」行で3分類すべてが 0）；会計ツリーは D21 (権威態と派生態の境界)「オフラインで走査により再計算可能」に従い可能と判定されるが、E59 (バッファメッセージはユニットのみから再計算可能か) のその列は神託であり未実測である。これは D15 (フォーマット凍結ポリシー) 第4層（派生状態）に該当し、フォーマット分岐ではない。**状態種別は再計算可能性により判定し、「ツリー」という単語では判定しない**：あるツリーの内部ノードがそのリーフから再計算可能 ⇒ 派生状態；リーフが計算で復元できない値を収容している場合 ⇒ 権威状態。会計は同一判拠の確定済みインスタンスであり、権威記録ポイントは D21 (権威態と派生態の境界) にある。

**装填規律はレコードタイプごとに分かれる**：「1コンテナ1世代」と「スナップショットパブリッシュ前にオープンコンテナを閉じる」は世代単位で回収されるパッキングレコードタイプ 1（墓石）にのみ成立する；タイプ 2 のレコードは1件ずつ死亡し、コンテナは B ツリーの分割 / マージによって回収され、1枚のリーフが多数の世代を生き延びて反復的に書き直され、スナップショットはそれを共有するに過ぎない。**死亡は3層に分かれる**：① タイプ 2 のレコードの死亡は墓石を書かず、現行リーフバージョンによって具現化され、現行バージョンの選択は C113 (スキャン再構築時に複数世代ユニットの現行バージョン判定に入力がない) に依存する；② リーフコンテナの識別子が消失した場合（削除により空になって除去、マージ時に吸収された右半分、ヘッド跨ぎの初回 COW 再生時の旧識別子）、退役レコードを書き込む：パッキングレコードタイプ 3、死亡 txg を保持し、専用の退役ツリー（D18 (ブロックに含める情報) 確定項目 11）に常駐する；退役レコードのライターは追記のみ行う：同一識別子が管理者ロールバックを跨いで複数存在することがあり、重複排除や上書きはしない；死亡 txg = 識別子が消失したパブリッシュの checkpoint_txg；読者および checker は死亡 txg が現行バージョンの系譜上にある退役レコードのみを認識する；③ オブジェクト全体の削除は unlink の意図時に書き込まれるオブジェクト全体の墓石により具現化され、意図のパブリッシュから有効となる（D18 (ブロックに含める情報) 確定項目 24 第2条、確定項目 16 第1条）。ファイルの縮小（上書きによるユニット削減、truncate）は新レコードを書き込まず、データユニットの生死はスキャン再構築時に再構築された inode レコードに従って判定する（「再構築」の段落）。

**コンテナ識別子にアロケータは不要である**：inode 番号は各タイムライン上で単調である（管理者ロールバックの新ルートは R_old バージョンの inode ツリーを取り、高水位は max(現行版メモリ内, リング内から読み出せインスタンス表でクラッシュリカバリにより破棄されていないルート) を取る；クラッシュリカバリは選択されたルートのツリーとその高水位に戻る；いずれの後も発行される番号はそのツリー内の全キーより大きい）⇒ 挿入は常に最右のリーフに落ちる ⇒ 分割は最右リーフでのみ発生し、**分割点は末尾を取る**——左半分は元の全レコードと識別子を保持し、右半分は分割の契機となった新レコードから開始し、**右半分のコンテナ番号 = その新レコードの inode 番号**、誕生世代 = 今回のパブリッシュの checkpoint_txg、**誕生ツリー = 今回の分割を実行したツリー**（分割は新規作成であり、「COW 書き直しで識別子不変」とは別事象である）；最初のコンテナの番号 = 最初のレコードの inode 番号。内部ノードも同様に右端でオーバーフローし末尾で分割され、右半分はコード 2 でありコンテナ識別子を持たない。マージ：**左による右の吸収と左による右からの借用の双方を行う**（右による左の吸収や右による左からの借用は「レコード番号 ≥ コンテナ番号」を破る）、左半分の識別子を保持する；吸収された右半分は COW 解放パスを通り、同一パブリッシュ内でその識別子に対して退役レコードを1件書き込む；左が右から借用する際の借用件数は均等配分とする（借用後の2枚の件数差は1以下）、右半分は識別子を保持し退役レコードを書かず、その区切りキーを現行バージョンの最小キーに変更する；マージの契機は書き込みパスのポリシー（単体テスト / モデル突き合わせ）であり、フォーマットには含めない。**レコード数が 0 のタイプ 2 コンテナはディスクに書き出さない**（パブリッシュ前に必ずマージされねばならない；唯一のリーフが削除により空になった場合はディスクに書かず、ルートがゼロエントリとなる）、削除により空になり除去されたコンテナは同一パブリッシュ内でその識別子に対して退役レコードを1件書き込む。**ヘッド跨ぎ初回 COW の再生**（C113 (スキャン再構築時に複数世代ユニットの現行バージョン判定に入力がない) 確定 P6）：新識別子 = (自ヘッドツリー ID, タイプ 2, 元コンテナ番号, 今回のパブリッシュ世代番号)——コンテナ番号を保持し、誕生ツリーと誕生世代のみを変更し、同時に旧識別子に退役レコードを書き込む。再生はタイプ 2 にのみ適用される：タイプ 1 コンテナはライターツリーが排他的に占有し、クローンは自身のコンテナインデックスに取り込まず（D18 (ブロックに含める情報) 確定項目 11）、ヘッド跨ぎの COW は行わない。「コンテナが COW 書き直された際に識別子は不変」（内容同一、位置変更）と「分割後に左半分の識別子は不変」（内容削減、識別子不変）という2つの異なるアサーションが並存する。タイプ 1 のコンテナ番号は依然として C103 (コンテナインデックスの配置先がない) の単調割り当てに従い、2つの規則はレコードタイプごとに分かれる。

**キーと inode 番号**：key = inode 番号 8 バイト、一意性スコープは (ツリー ID, inode)、ビット幅 64、ライフサイクル内の作成総数上限として 2^63 を宣言。**パブリッシュ済みの (inode 番号, オブジェクト誕生世代) は再利用しない**（D6 (スナップショット実現モデル) 判拠 9 の戒律）：`deleted_inodes` のバックグラウンド回収ウィンドウ内での再利用は新旧オブジェクトの extent キー3セグメントを完全に一致させてしまう。⚠️ **「パブリッシュ済みの」という言葉は言い回しの統一のためではなく、現行設計の自己無撞着性を保つためである**：高水位は会計に常駐し（D5 (スナップショット / 空間会計メカニズム) 確定項目 4 第12項）、破棄されたチェックポイントの会計はそれに随伴して失われる ⇒ **通常のクラッシュのたびに**その世代で発行された inode 番号が再発行されることになる。管理者ロールバックはマウント中の前方パブリッシュであり、そのパブリッシュの inode 番号高水位は max(現行版メモリ内, リング内から読み出せインスタンス表でクラッシュリカバリにより破棄されていないルート) を取り、発行済みの番号は再発行しない（D23 (ジャーナルの役割とフォーマット) 確定項目 14）；クラッシュリカバリで比較的古いルートに落ちた際、高水位は選択されたルートの会計と共にロードされ、破棄されたタイムラインで発行済みの番号は再発行され得るが、リカバリ後の全新規オブジェクトの誕生世代はリング内から読み出せる全ルートおよびレコードの txg より大きくなる（D23 (ジャーナルの役割とフォーマット) 確定項目 14 注3）——破棄されたタイムライン内のパブリッシュ済みオブジェクトの誕生世代と衝突するのに何重の故障が必要かはクラッシュリカバリ形態で未計測であり、C143 (inode番号高水位がクラッシュリカバリで旧ルートに退行した後に再発行される) を参照；外部に公開されるオブジェクト識別子 = (inode 番号, オブジェクト誕生世代) であり、inode 番号でオブジェクトを識別する全構造は誕生世代を付帯するかルートと共にロールバックしなければならない。`deleted_inodes` 回収ウィンドウ内で extent キー3セグメントが完全一致するという文は同一タイムライン内での再利用（旧オブジェクトがパブリッシュ済みで、そのエクステントがツリー内でバックグラウンド回収待ち）を指しており、従来通り禁止される；クラッシュリカバリ後に同一番号を再発行した場合、新オブジェクトのエクステントは選択されたルートの extent ツリーに入り、旧オブジェクトのエクステントは破棄されたルートと共にツリー内に存在しない——**これは extent ツリーと `deleted_inodes` が共に選択されたルートに随伴することに依存している**；管理者ロールバックのパブリッシュでの新ルートは R_old の inode ツリーと extent ツリーを取り、`deleted_inodes` も R_old バージョンのものを取る（さもなければ R_old 後に削除され R_old の inode ツリーと共に戻ったオブジェクトがバックグラウンド回収で再度回収されてしまう；メイン agent 2026-09-26 決定、推論、0巡攻撃済み）；`deleted_inodes` の形態（確定項目 16）はこの2条を守る；ディスク上で両者を分離するのはユニット5タプル内のオブジェクト誕生世代である（D9 (暗号化) 確定項目 6）。**inode レコードは回収完了まで inode ツリー内に残り（nlink = 0）、`deleted_inodes` は番号のみを保存する**。

**次の inode 番号は会計内の統計量の1つである**（D5 (スナップショット / 空間会計メカニズム) 確定項目 4 第12項「inode 番号高水位」）：key = (統計量, ツリー ID, デバイスセグメント, 世代)、デバイス次元は持たず、value = 次の利用可能番号、**マージ規則は最大値を取る**（冪等、可換、結合的）；エントリは同様に seq を保持するが（確定項目 15 第3条はフォーマット要件であり、write buffer ツリー1本につき1組のエントリフォーマット）、**マージ結果はそれに依存しない**。「後勝ち」ではなくこれを選択した理由は、クラッシュ後に txg が再発行されるセルにある：同一世代セグメント内で大小2つの値が前後して出現した場合、最大値を取って大きい方を採用する（番号の浪費、再利用の回避）；「未パブリッシュは発生しなかったものと同等」は通常マウントパスにのみ成立し、スキャン再構築が見るのはディスクである。「ラベル → マージ規則」はディスク上セマンティクスであり、統計量登録表は独立した凍結コンポーネントとして確立される（D15 (フォーマット凍結ポリシー)）。高水位行は書き込み可能ヘッドに対してのみ維持され、段差行の規則に従って書き込まれる：高水位が変動したパブリッシュで段差行を1行残し、ルートの COW に随伴して保持し続ける（D5 (スナップショット / 空間会計メカニズム) 確定項目 2 で各ルートの会計ツリーはその世代の行のみを収容しパブリッシュごとに全行を書き直すが、これに例外を設ける。D5 (スナップショット / 空间会計メカニズム) 確定項目 2 の対象範囲「inode 番号高水位の段差行」を参照）、読み取り専用ツリーにはこの行はない；現行値の読み取り = 当該次元タプル下で世代番号が最大の行を取得、空 ⇒ 一度も維持されていない、0 ではない。クローンヘッド：新ヘッドの高水位 = origin のその時点のランタイム高水位（write buffer 内にあり未フラッシュのものを含む）、クローン作成時のパブリッシュで新ツリーのためにこの高水位行を書き出さねばならない；2つのヘッドはその後それぞれ割り当てを行うため番号は交差し、一意性は (ツリー ID, inode) に依存する。

**レコードフィールド表（固定長 140、リトルエンディアン；アライメント計測条件はレコード内オフセットであり、コンテナ内第 n 件の開始位置 136 + 140n（コード 3 ヘッダ 107 + nonce / MAC / アルゴリズムタイプ予約領域 29、D18 (ブロックに含める情報) 確定項目 16）は 8 バイト境界に整列せず、非整列読み取りを行う）**
<!-- format-const: INODE_RECORD_BYTES = 140 -->
<!-- format-const: INODE_LEAF_RECORDS = 233 -->

| オフセット | フィールド | 幅 |
|---|---|---|
| 0 | inode | 8 |
| 8 | オブジェクト誕生世代（作成時のパブリッシュの checkpoint_txg；inode 番号非再利用後は再利用判別の責務を負わず、5タプル / AAD の day-1 契約に従って保持され、スキャン時に当該オブジェクトの各データユニットヘッダとフィールドごとに対照） | 8 |
| 16 | locality_id（第1版で親ディレクトリがない場合は 0） | 8 |
| 24 | mode / uid / gid / nlink、各 4 | 16 |
| 40 | size / blocks / rdev、各 8 | 24 |
| 64 | atime / mtime / ctime 秒、各 8（unlink の意図時に呼び出し元が与えた単一の値を3つで共用、各バッチでそのまま複製） | 24 |
| 88 | 変更カウント（最後の変更が存在するパブリッシュの checkpoint_txg；コミットアンカー = リーフコンテナヘッダの誕生世代番号；同一パブリッシュウィンドウ内の複数回変更は最終版のみがディスクに落ちる——内部ノードはバッファリングしない；unlink の意図時および各バッチも変更と見なし、今回のパブリッシュの checkpoint_txg を書き、前版をそのまま複製しない；作成も1回の変更と見なすため、変更カウントはレコードオフセット 8 のオブジェクト誕生世代以上となる、I-9.26 (inode 変更カウントが誕生世代とコンテナ誕生世代番号の間にある)） | 8 |
| 96 | atime / mtime / ctime ナノ秒、各 4 | 12 |
| 108 | パディング（常に 0） | 4 |
| 112 | flags（常に 0） | 8 |
| 120 | 予約（常に 0） | 20 |

**書き込み時間（オフセット 64 からの3つの秒フィールド、オフセット 96 からの3つのナノ秒フィールド）は当該レコードを書き込んだパブリッシュのパラメータを取り、システムクロックは取らない**：パブリッシュ呼び出しごとに1つの書き込み時間を渡し、同一パラメータの2回のパブリッシュはバイト単位で一致する（再現性）。3つの秒フィールドはいずれもパラメータの秒数を書く；3つのナノ秒フィールドはパラメータのナノ秒部分を書くが、パラメータは秒数のみを保持するため 0 を書く。**新規作成**（新規プールでのファイル作成、新規 inode 作成）は作成時のパブリッシュパラメータを書く；**内容書き換え**（上書き、シーケンシャルライト）で当該レコードを書き直す際は今回のパブリッシュパラメータを書き、前版の値を複製しない；**unlink の意図時**は呼び出し元が今回の unlink に随伴して与えた時間を書き、3つの時間秒（atime、mtime、ctime）で単一の値を共用し、各バッチは意図時に書かれた値をそのまま複製する；**当該レコードの内容を変更しないパブリッシュ**——同一リーフ内の他レコードの変更に伴う巻き添え書き直し、ロールバック——は当該レコードの前版の値をそのまま複製する。

**`blocks` は論理長における 512 バイトブロック数である**：`blocks` = ⌈`size` ÷ 512⌉、ファイル長から切り上げで計算し、**割り当てられた空間を表すものではなく**、当該オブジェクトにいくつのデータユニットが割り当てられたかで計算しない。新規プールでのファイル作成における当該レコードは 3000 バイト ⇒ 6；新規作成されまだデータが書き込まれていない inode ⇒ 0。

エクステントポインタは配置せず（D14 (双軌（大小ファイル / 永続一時）) 確定項目 3 インライン閾値 0）、btime はレコードに入らない。**flags / パディング / 予約の3セグメントは同一ポリシー：常に 0、読者が非 0 に遭遇した場合 ⇒ 当該レコードは EIO（オブジェクト単位、コンテナ全体は拒絶しない）**——前方互換性ポリシーは破損検出ではなく（ペイロードチェックサムは通常通り通過）、I-9.7 (レコードの3セグメントが常にゼロ) として設定され、I-1.7 (パッキングコンテナの正当性と判定順序) のアボートチェーン外にある。将来的に任意のビット / バイトに意味を付与する場合は D15 (フォーマット凍結ポリシー)「既存フィールドの意味変更」に従い incompat と判定し、退役したバイトは永久に廃棄される。**レコード内に compat 拡張パスは存在しない**：inode 属性はすべてセマンティック依存関係を持ち、D15 (フォーマット凍結ポリシー) 判定表の第1行は適用できず、かつレコードは反復的に書き直され、古い読者は未知の内容を破棄してしまう。予約 20 は耐荷重を持たない：レコード内のいかなる意味付与も incompat であり、幅はその際にレコード幅まで変更するか否かを決めるに過ぎない。**オーバーフロー路線は属性ごとに個別に判定する**：セマンティック依存のない属性はキースペース追加（D15 (フォーマット凍結ポリシー) 判定表で compat）、依存のあるものは flags ビット / 予約（incompat）とする；属性キースペースの配置先はない（xattr は延期）。**レコード幅は登録表がタイプ 2 に与えた幅に等しくなければならず**、不等時は破損と判定する（I-1.7 (パッキングコンテナの正当性と判定順序) 内で「クラス識別子セグメント」の後、I-1.2 (ブロックヘッダ書き込み順序がパブリッシュ済み) の前に1ステップ挿入、ヘッダフィールドのみを要する）；幅の変更は incompat である。レコードフォーマット、コード 3 クラス識別子のタイプ 2 に対する用法、inode ツリーのキーエンコードは恒久バイトである：**独立した凍結済みコンポーネントとして確立する**（C9 (冻结后偷改格式) はコンポーネント単位で記録）、凍結時点はタイプ 2 コンテナを初めて書き出したイメージである；D15 (フォーマット凍結ポリシー) の4層は動かさない；タイプ 1 のレコードフォーマット（幅 40、D18 (ブロックに含める情報) 確定項目 24）はそれぞれ独立である。

**不変条件**：I-9 (inode树结构) クラス、14条、[invariants/09-inode树结构.md](../../../crates/singlefs-checker/invariants/09-inode树结构.md) を参照；状態列には一律に「ツリーの種別が識別可能であること（確定項目 8）」という前提が付き、第1版イメージ上ではタイプ 2 コンテナヘッダ内の誕生ツリーからどのツリーが inode ツリーであるかを逆算できる。nlink == dirent 件数はここでは確立しない——そのゲート（dirent ツリーが存在すること）に入力がなく、dirent ツリー運用開始時に同時に確立する。第1版の唯一のファイルは inode 1（0 は無効）、nlink = 1 を取り、孤児回収には入らない。分割 / マージの識別子継承およびマージ方向以外の書き込みパス規律は操作アサーションであり、単体テスト / モデル突き合わせおよびクラッシュポイント再生を通る。

**再構築**：内部ノードが消失した場合、パッキングレコードタイプ 2 のコンテナを収集し、(誕生ツリー, コンテナ番号, コンテナ誕生世代) でグループ化し、同一識別子の複数バージョンから現行を選択する = C113 (スキャン再構築時に複数世代ユニットの現行バージョン判定に入力がない)；退役ツリー内に退役レコードによって指名された識別子は収集せず、退役レコードは死亡 txg を保持し系譜に従ってフィルタリングする（I-9.18 (削除キュー内の番号に系譜上の墓石が存在する) の墓石に対する読み出しと同一）、退役ツリー、墓石ツリー、系譜ツリーの現行に随伴するこれら数ツリーは C113 (スキャン再構築時に複数世代ユニットの现行バージョン判定に入力がない) 確定 P2 に従ってのみバージョンを選択する；各リーフのキー範囲はレコードから計算し、内部ノードのキー区間はリーフから計算する；成果物は D18 (ブロックに含める情報) 確定項目 5 のレベル 1（読み取り専用マウント）に該当し、レベル 2 の見届け人は存在しない。**extent ツリーの再構築**：オブジェクトは (ツリー ID, inode) で識別し、自ヘッドの extent ツリーのみを見る；当該オブジェクトの全エクステントキー（残存したものと復元されたもの）に同一の先頭セグメントを割り当て、値は inode レコード > 残存 extent キー > 0 とし、レコードが無事な場合は再構築ビュー内の値がレコード内の値となる；誤った値を取っても局所性が失われるだけである。**データユニットの生死**：再構築された inode レコードのオブジェクト誕生世代と size から換算されるユニット数により判定し、ファイル縮小は新レコードを書かない（読み方 Z）。I-9.9 (locality 三値対照) は再構築後に側ごとに判定する：一方が他方から編成されたものである場合は「判定不能」を出力する。

**新規プールでのファイル作成が書き込む内容**：コード 2 ルートノード1個（ヘッダにキー区間 [1, 1]；エントリ1件：key 1、タイプ 2、誕生ツリー、コンテナ番号 1、誕生世代 3、ポインタ）；コード 3 リーフ1個（タイプ 2、コンテナ番号 1、誕生世代 3、レコード数 1、レコード幅 140）；レコード1件（inode 1、誕生世代 3、locality 0、nlink 1、変更カウント 3、3セグメント 0）；会計内に「inode 番号高水位 = 2」の1行。

**対象範囲**：**同一タイムライン上で inode 番号は再利用しない**——「タイムライン」とは選択されたルートの祖先チェーンを指す：管理者ロールバックで破棄された区間を含み（ロールバック時のパブリッシュの水位は max を取り、その区間で発行された番号は再発行しない）、クラッシュリカバリで破棄された区間は含まない（クラッシュリカバリで旧ルートに落ちた後、破棄された区間で発行された番号は再発行され得る）。**一意性の射程はパブリッシュ済み履歴の範囲内である**——各コンテナ番号はいずれかのレコードが初めて挿入された際の inode 番号である；番号はクラッシュリカバリで旧ルートに落ちた後に再発行され得るため（管理者ロールバック時のパブリッシュの水位は max を取り再発行しない、D23 (ジャーナルの役割とフォーマット) 確定項目 14）、同一ツリー内で2つのコンテナが同一番号を持ち得るが、それらを分離するのはコンテナ誕生世代である（右半分の誕生世代 = 分割契機となったパブリッシュの txg であり、リカバリ後の新インスタンスの全 txg はリング内から読み出せる全ルートおよびレコードの txg より大きい、D23 (ジャーナルの役割とフォーマット) 確定項目 14 注3）。同一パブリッシュ内での「分割 → マージ → 再分割」でも同一識別子の2版は生成されない。クラッシュ跨ぎでは成立しない：クラッシュ後にチェックポイント番号が再発行され得、破棄された版と再試行された版は4セグメントが逐字一致し、内容が異なり、双方がディスク上に存在する——これはデータユニットの5タプル、コード 2 ノードの (ツリー, 階層, 誕生世代番号) と同一の露出である（ルートレコードは D16 (パブリッシュセマンティクス) ライダー条項 1 のインスタンス世代番号で識別、他ユニットクラスはクラス識別子セグメント内の書き込み順序 (インスタンス世代番号, トランザクション番号) で識別——コンテナは単独でインスタンス世代番号を追加せず、書き込み順序内の4バイトがそれである）。通常パスは D3 (領域割り当て) 確定項目 1 の述語（割り当てレコード）で判定し、スキャン再構築パスは C113 (スキャン再構築時に複数世代ユニットの現行バージョン判定に入力がない) 確定の P2 / P3 規則で判定する。⚠️ **安全と判定する根拠はツリー ID の条項と同源である**：孤児と生存版のインスタンス世代番号は必ず一致しない（D18 (ブロックに含める情報) 確定項目 11「1チェックポイントを無効化する際は必ずインスタンス世代番号を変更する」）、スキャン再構築の2つのゲートは書き込み順序に従って新しい方を選択する。⚠️ **残存する疑問点**：統計量マージ規則を説明する際「『未パブリッシュは発生しなかったものと同等』は通常マウントパスにのみ成立し、**スキャン再構築が見るのはディスクである**」という一文は逆方向のように読める——それは「水位が1世代セグメント内で後勝ちではなく最大値を取る」ことを管轄しており、「未パブリッシュの番号を再発行してよいか」とは同一命題ではないが、両者は非常に近接している。⚠️ **再構築の射程は単一ヘッドイメージである**：クローンヘッドが存在する場合、共有リーフヘッダ内の誕生ツリーは origin である（I-1.3 (ブロックヘッダツリー ID の一致) の C110 (ヘッド跨ぎ共有と暗号化後のメタデータクラスブロックヘッダ一致性読み出し) のセル）、誕生ツリーで識別するとクローンヘッドのツリーが不完全になる——複数ヘッドイメージの帰属分割にはクローン祖先表が必要であり（E90 (ツリー ID の AAD 導入とヘッド跨ぎ共有) 第2筆）、着工前には複数ヘッドイメージの inode ツリー再構築に入力がない。⚠️ 充填率に下限はない：`⌈n/233⌉` は新規ツリーの値であり、1e6 件順次作成して 90% 削除しマージしない場合は 9.98 倍の空間増幅となる；レコード拡幅により爆発半径を同時に縮小できるが（160 ⇒ 204件）、このトレードオフは C100 (inode コンテナの消失が恒久損失であり冗長条項がカバーしていない) に属する。⚠️ **当時「新規プールでのファイル作成が書き込む内容」を妨げていた9項目（マイルストーン1）、① ② は確定済み、残りは記録の通り**：① ツリーテーブルエントリは確定項目 8 で決定済み；② 会計エントリの seq は value に常駐し幅 4 バイト（確定項目 7）；③ 統計量ラベル値（C71 (未知の統計量ラベルのサイレント無視)）；④ デバイス次元を持たない統計量のデバイスセグメントの値、新規ツリーの会計行初期化、現行値の読み出し方（C117 (デバイス次元を持たない統計量のデバイスセグメントに値がない)）；⑤ コード 3 コンテナが拡張ポイントを持つか否か（C108 (パッキングレコードユニットに対する拡張ポイント帰属判定がない)）；⑥ コード 2 ノードヘッダの基本バイト数（確定項目 11 で収束済み）；⑦ `deleted_inodes` の形態（確定項目 16）；⑧ ⑨ レコードフォーマットと統計量登録表の2つの凍結コンポーネントの spec ファイル（C119 (凍結コンポーネントに spec ファイルがない)）。⚠️ ヘッド跨ぎ初回 COW 時にコンテナ識別子を再生させるか否かのもう半分は E90 (ツリー ID の AAD 導入とヘッド跨ぎ共有) 第2筆に属する。⚠️ **`blocks` は外部の `st_blocks` を担わない**：stat が報告する `st_blocks` をどう与えるか（stat 時に extent ツリーから割り当てユニットをその場で計算）、SEEK_HOLE / SEEK_DATA は POSIX 面を実装するマイルストーンに移管する。`blocks` をファイル長切り上げに変更したことで新規プールでのファイル作成のバイトが変更：当該レコードオフセット 48 の 8 バイトが 64 から 6 に変更。⚠️ **書き込み時間は単一イメージから判定できない**：値はパブリッシュパラメータであり、単一イメージ内からパラメータが何であったかは見えない——O1 (メモリモデル突き合わせ（HashMap 理想ファイルシステム）) のモデルと突き合わせは書き込み時間を持たず、O2 (独立パーサー + checker) は単一イメージのみを判定し、O3 (独立仕様実行器) の射影は inode ツリーに入らない；これを判定するのは書き込みパスに従って厳格に記述された harness テストケース（上書き、シーケンシャルライト、inode 作成、unlink 各1本）であり、複数ユニットシーケンシャルライト、ロールバック、同一リーフ内の巻き添え書き直しの3セルはテストケースを欠く（C677 (書き込み時間が3セルの書き込みパスでテストケースを欠く)）。

⚠️ **死亡3層化の2つのコスト**：墓石は意図時から発効し、unlink 途中のスキャン再構築はその nlink = 0 のレコードと残存データユニットを読み取れず、クラッシュポイント上の再構築ビューに意図パブリッシュ済みのオブジェクトが余分に現れることはない（E178 (リーフコンテナとレコード死亡の計数) 第2、3段）；借用時に右半分は識別子を保持し、借用後にそのコンテナ番号は左半分の最大キー以下となり得る。I-9.4 (コンテナ番号が最小キーを超えない) 第3文（左コンテナの最大キー < 右コンテナ番号）はこの形態に伴い削除され、不変条件リストと checker は修正済み（2026-10-09 第50バッチ）；コストは「分割時に右半分のコンテナ番号を小さく与えすぎたが左隣の番号よりは大きい」という種類のイメージを阻止する判拠がなく、`inode_tree` 単体テストのみが残る点である。

**根拠**：

- E98 (inode レコードと inode ツリーの幾何構造)：inode レコードのディスク上形態——コード 3 へのパッキングは D21 (権威態と派生態の境界) 厳格制約 6 をトリガーせず、インデックスノードへの常駐はトリガーする（14フィールド中11フィールドが計算復元不能）；変更カウントは checkpoint_txg 単独では同一ウィンドウ内の2バージョンを順序付けできないため、内部ノードはバッファリングせず、同一パブリッシュウィンドウ内では最終版のみがディスクに落ちる。
- E103 (inode パッキングアプローチ①の更新コスト)：アプローチ①（独立コンテナインデックス + inode ツリーの識別子参照）の読み取りコストはコンテナインデックスを内部ノードに統合した場合より高く、さらにユニット読み取りが1回増えるが、単一書き込みは不変である——コンテナインデックスは別個のツリーを立てない。
- E104 (スキャン再構築の現行バージョン判定)：退役レコードを認識しない場合、退役コンテナ内の死亡した inode が復活する；モデルにより「再生時に最小 inode 番号をコンテナ番号とすると衝突する」ことが示されたため、再生はコンテナ番号を保持し、誕生ツリーと誕生世代のみを変更する。
- 4巡の三者論証後のユーザー決定 2026-09-05（資料 `_d8-item6-path.md`、`_d8-item6-r2-amended.md`、`_d8-item6-r3-conclusion.md`、`_d8-item6-r4-conclusion.md`、`_d8-item6-r5-landing.md`、4巡3脚の出力 `research/prompts/d8-item6-r{1,2,3,4}-{sonnet,opus}-output.md` および `research/results/d8-item6-r{1,2,3,4}-local.out`）。
- ユーザー決定 2026-09-12（C297 (inode 内部エントリ内に誕生ツリーが2部存在し処理が未確定)、2巡三者）：識別子参照内の誕生ツリーと子ポインタ内のものが2部並存、原語は変更履歴。
- ユーザー決定 2026-09-16（C143 (inode番号高水位がクラッシュリカバリで旧ルートに退行した後に再発行される)、3巡三者）：一意性を (inode 番号, オブジェクト誕生世代) に絞り込み、原語は変更履歴。
- ユーザー決定 2026-09-23：`blocks` はファイル長切り上げとし割り当て空間を表さない、`st_blocks` は POSIX 面実装マイルストーンへ移管（メイン agent の提案、C480 (inode レコードの blocks の計算規則がリポジトリ全体にない)）、原語は変更履歴。
- ユーザー決定 2026-10-01：K-R3-3「ルート伸長を記述」、`research/prompts/m3-delete-r3-main-verification.md` 第3節 G6-墓石ツリールート満杯（原語は変更履歴）。
- 判決 `research/prompts/c673-clauses-r1-main-verification.md` 第2節 K3：書き込み時間の各書き込みパスでの値（2脚一致、攻撃側がレプリカで計測済み、メイン agent が再実行し確定）、3つのオラクルはいずれも判定不能。
- ユーザー決定 2026-10-03：削除ステップを変更と見なす（「変更と見なし実装修正」）、時間フィールドは「乙 意図が時間を保持、3セル同一変更」を採用、原語は変更履歴；コード巡第3巡判決 `research/prompts/m3-unlink-code-r3-main-verification.md` 第2節 n3 行（「ctime が追従変更」の絞り込みはユーザーへの再確認なし、レコードオフセット 64 からの3セルに同一値を書き込み）。
- ユーザー決定 2026-10-05：対象範囲に「同一タイムライン上で inode 番号を再利用しない」を追加（「書け」）、判決 `research/prompts/m3-delete-checker-r2-main-verification.md` 第2節 T15 行、原語は変更履歴。
- E176 (inode ツリーリーフマージ3候補の容量と書き込みコスト)：マージにより容量が倍増（Q1r、負荷に対して不安定）、左による右の吸収と左による右からの借用で多重書き換えされるリーフは位置記法では反転せず、内容記法では無視可能——マージと借用の双方を実施；借用が下限に丁度達した際、内容記法の多重書き換えは「無視可能」から「反転なし」へ変化——借用は均等配分とする。
- E177 (ツリーごとの墓石不書込による復活数)：削除により空になったリーフコンテナをディスクに書かず何も書かない場合、kb に従い現行を選択するスキャン再構築の読み方下で inode ツリーが復活する——コンテナ識別子の消失には別個のキャリアが必要（死亡3層化の第②層）。
- E178 (リーフコンテナとレコード死亡の計数)：退役レコードが死亡 txg を保持し系譜でフィルタリングされる場合、借用と吸収、管理者ロールバックの履歴においてコンテナの復活と誤判定死亡は共に 0；ファイル縮小を再構築された inode レコードに従い生死判定する場合も復活と誤判定死亡は共に 0；墓石が意図時から発効する場合 unlink 途中で1レコードと残存ユニットが漏れるがクラッシュポイントで余分なオブジェクトは出ない；借用時に右半分の識別子を変更すると識別子保持に比べ毎回コード 3、コード 2 各1個と退役レコード1件が余計に書き込まれる。
- 三者判決 `research/prompts/m3-leaf-death-r1-main-verification.md`（第2節 K1：「死亡は現行リーフバージョンにより具現化」は全否定せず字面修正；K3：均等借用）、`research/prompts/m3-leaf-death-r2-main-verification.md`（第2節：退役レコードは死亡 txg を保持、現行随伴ツリーは P2 に従ってのみバージョン選択；区切りキーは子の現行版の最小キーを採用；均等借用維持）、`research/prompts/m3-leaf-death-r3-main-verification.md`（第2、5節）。
- ユーザー決定 2026-10-08：「M3 で借用も合わせて実施」（分岐票 `m3-3-5-forks.md` M 行）；コンテナ識別子消失時に退役レコード・甲を書き込み、ファイル縮小は読み方 Z を採用、墓石は意図時から発効、借用時の区切りキーは子の現行版最小キーを採用し I-9.4 (コンテナ番号が最小キーを超えない) 第3文を削除（分岐票 `m3-leaf-death-forks.md`、判決 `research/prompts/m3-leaf-death-r3-main-verification.md` 第5節）。原語は変更履歴。
- 退役レコードのライターは追記のみ、死亡 txg は識別子消失時のパブリッシュを採用、読者と checker は系譜上のみを認識：判決 `research/prompts/m3-retire-r1-main-verification.md` 第2節 K2「死亡 txg はいつのものを取るか」「同一識別子の退役回数」の2行（2回の攻撃側サンプリングでいずれも確定：死亡 txg を unlink 意図時または最終版の誕生世代番号とすると誤判定死亡、ライターが「既存なら追記しない」または「識別子で上書き」とすると復活）および第3節第3、4条；ユーザー決定「プール開設時に作成（推奨）」、原語は変更履歴。

**未済**: C100 (inode コンテナの消失が恒久損失であり冗長条項がカバーしていない)、C102 (インデックスノードヘッダに予約領域がなくフィールド追加が compat でない)、C103 (コンテナインデックスの配置先がない)、C104 (スキャンパス上のコード 1 / 2 のペイロード完全性に自己完結した証拠がない)、C106 (オープンコンテナ数の上限)、C107 (インデックスノードの AAD 識別子に配置先がない)、C110 (ヘッド跨ぎ共有と暗号化後のメタデータクラスブロックヘッダ一致性読み出し)、C113 (スキャン再構築時に複数世代ユニットの現行バージョン判定に入力がない)、C115 (inode 属性のオーバーフロー路線に配置先がない)、C116 (リーフコンテナの分割 / マージ規律に失敗する検査がない)、C119 (凍結コンポーネントに spec ファイルがない)、C152 (絞り込み後の未パブリッシュ番号再発行のスキャン再構築再検証)、C347 (祖先表が読み出せない場合にクローンヘッドが何を認識するかの条項がない)、C677 (書き込み時間が3セルの書き込みパスでテストケースを欠く)。

#### 確定項目 7：write buffer エントリ付属の `seq` —— 4 バイト、value に格納

**決定事項**：**`seq` は幅 4 バイト、キーではなく value に格納し**、射程は会計 / 逆引きインデックス / LRU の3本の write buffer ツリーをカバーし、**1ツリーにつき1組のエントリフォーマット、3ツリーで同一幅とする**。幅が確定したことで因果関係が逆転する：**`T_dirty` の上限はこれ以降マウント時判定となる**（4 バイト = 32 ビットウィンドウ、34 バイト/エントリ換算で約 136 GiB に対応）；D16 (パブリッシュセマンティクス) 確定項目 5 の「`T_dirty` は調整可能値でありフォーマット定数ではない」はそのまま成立し、フォーマットで規定される上限が追加された点のみが変更される。**seq 内にさらに txg を格納することはキー内に既にあるセグメントの二重コピーであり**、禁じられる。2つのライダー条項：

1. **逆引きインデックスおよび LRU のキー決定時、重複排除ウィンドウが同様に `T_dirty` で区切られるようにしなければならない**（世代をキーに含める、または他の同等手段）。4バイトで足りる根拠は「同一キーの重複排除待ちエントリは必然的に同一パブリッシュウィンドウに属する」ことであり、その条項は会計ツリーにおいて D5 (スナップショット / 空間会計メカニズム) 確定項目 5 のキー末尾セグメントが「世代」であることから導出された；両ツリーのキー配置はリポジトリ全体で一言も言及されておらず、このライダー条項を履行しない限り両ツリーにおいて4バイトの根拠はない。
2. **同一キーのバッチ間挿入順序は必ず `seq` と同一方向でなければならず**、暗黙の前提として放置してはならない——btree 層の同一キーは「後書き勝ち」であり「`seq` が大きい方が勝ち」ではない。一方、リーフ分割による並行 flush 下では2つのバッチの物理着地順序は `seq` の制約を受けない。

**対象範囲**：本項が定めるのはエントリ内の当該セグメントの幅と配置先であり、他セグメントは定めない（会計キーの `(統計量タイプタグ, ツリーID, デバイス, 世代)` 4セグメントは D5 (スナップショット / 空間会計メカニズム) 確定項目 5 にある）。⚠️ 当該セグメントは他セグメントと直交しているわけではない：それら4セグメントは**同一チェックポイント内で一意ではない**（冪等完全値は複数回書き込まれる）、重複排除規則「後勝ち」は「後」がエントリ自身によって保持されることを逐字的に要求する ⇒ これを定めなければ、4セグメントを合わせてもマージ結果を確定できるキーとはならない。本項が **D5 (スナップショット / 空間会計メカニズム) ではなく D8 (コアインデックス構造) に設定された**理由は、write buffer が「逆引きインデックス / LRU / 会計」の3分類をカバーし、会計側に設定すると残り2ツリーでそれぞれ再確認が必要になるためである。⚠️ 会計エントリはこれにより 30 バイト（key 22 + value 8）から **34** バイトとなり、E97 (会計と割り当てレコードのエントリエンコード) / E71 (会計キーのエントリ数と幅) / E54 (会計の「世代」は何世代保持すべきか) / E99 (write buffer エントリと seq の重複排除) の4ページの数値を再計算する必要がある。⚠️ 「inode 番号高水位」の会計エントリも同様に `seq` を保持するが（確定項目 15 第3条はフォーマット要件）、そのマージ結果は最大値を取り、`seq` に依存しない。⚠️ 値生成規則（何から開始し単一パブリッシュ内でどう数えるか）は確定項目 10；フロントエンドとノードバッファの連携は確定項目 12 にある。

**根拠**：

- E99 (write buffer エントリと seq の重複排除)：まさに同一キーの重複排除待ちエントリが1パブリッシュウィンドウ内にどれだけ存在し得るか、重複排除に何ビット必要かを計測——ウィンドウ要件は 32 ビット以内に収まり、4バイトで十分。
- E17 (write buffer マージ上限)：第1版でエントリ番号を欠落させ、2アームが真値と即座に不一致となり、「エントリは自前の `seq` を持たねばならない」という判定可能形態を迫られた；リーフ分割並行の実測により並行 flush 下で物理着地順序が `seq` の制約を受けないことも判明したため、ライダー条項 2 は明記が必須となった。
- 3つの確定条項の文面衝突（三者論証、メイン agent が各条をその場で調査）：D5 (スナップショット / 空間会計メカニズム) 確定項目 2「探索不要 ⇒ ポイント削除であり走査ではない；接触キー数 = 統計量数」——`seq` がキーに入ると同一プレフィックス下に複数行が存在し、ポイント削除が成立しなくなる；確定項目 6「エントリは同様に seq を保持する……ただし**マージ結果はそれに依存しない**」——単調統計量の最大値取得は2つの合流を要求するが、`seq` がキーに入ると異なるキーとなり決して合流しない；D5 (スナップショット / 空間会計メカニズム) 確定項目 5 のキー定義自体に `seq` は含まれず、「後は `seq` で決定」は `seq` がキーに入らない場合のみ自己整合する。メイン agent の補足第4条：`seq` がキーに入るとキーを構築不能にする——縮小後に同一世代で1行のみ残るため、読者が `(タグ, ツリー, デバイス, 世代)` の値を検索しようとした際に `seq` が不明となり、ポイントクエリがプレフィックススキャンに劣化する。
- ユーザー決定 2026-09-06：幅を 4 バイトとし因果を逆転、射程を3ツリーに拡大、`seq` は value に格納（原語は変更履歴）。

**未済**: C161 (バッファ2レベル並存で連携未定)、C162 (seq の value 格納に伴い4実験の数値を再計算)、C345 (逆引きインデックス / LRU の陳腐化検出に時間手がかりがない)、C416 (重複排除ウィンドウが T_dirty で区切られるライダー条項が未検査)。

#### 確定項目 8：ツリーテーブルエントリのフィールド表

**決定事項**：**ツリーテーブルエントリ合計 200 バイト** = **ツリー ID 8**（先頭）+ エントリ長 2 + ツリー種別 2 + flags 2 + ルートポインタ 86 + `previous_snapshot_txg` 8 + 誕生 txg 8 + **ヘッド ID 8** + 予約 76。各層 ⌊(16384 − 131) / 200⌋ = ⌊16253 / 200⌋ = **81** ツリー、2層上限 81² = 6561 ツリー。⚠️ **分母の計測条件はここに固定**：16384 から**予約領域を含むコード 2 ヘッダ**を引く（131 はツリーテーブルユニット自体のコード 2 ヘッダ、key 幅 8 ⇒ 115 + 16、D18 (ブロックに含める情報) 確定項目 16 / 確定項目 18；エントリ領域開始位置は予約領域を含むヘッダ末尾、確定項目 11）、他の数値は引かない。

- **① スナップショットはツリーテーブルに入る**。各読み取り専用スナップショットにつき2件のエントリ：コード 14（スナップショット inode ルート）とコード 15（スナップショット extent ルート）。ルートポインタはスナップショット作成時の生存ヘッドの2つのルートをそのまま複製；**誕生 txg 8 バイト**、計測条件は**スナップショット完了（パブリッシュ）のチェックポイントの txg** であり、ルートノードの誕生世代番号ではない（それは最後の COW の番号であり、ツリー自体からは算出不能）。I-3.7 (生存スナップショットの列挙可能性) にキャリアを提供する。
- **② ツリー ID 高水位はルートレコードに常駐し、ルートリング内の全ルートレコードにおける当該フィールドの max を取る**。逐字形態：`新水位 = max(ルートリング内全ルートレコードの当該フィールド, 今回チェックポイントで発行された最高ツリーID + 1)`、**パブリッシュごとにルートレコードに随伴して書き直し**。「ルートリング内の全ルートレコード」とはルートリング内の自証に成功した全ルート（破棄されたタイムライン上のものも含む）を指し、リング内で自証を通過した journal レコードの新ルートセグメントが保持する当該フィールドをマージする（D23 (ジャーナルの役割とフォーマット) 確定項目 15）：ルートが存在するスロットが読み出せずそのレコードが存在する場合、リカバリはレコードからルートを再構築するため、レコードが保持する水位はそのルートの水位となる；journal レコードは対応するツリーが書き込まれルートがディスクに落ちた後にリング周回で上書きされる。ルートとそのレコードの双方が読み出せない（二重メディア障害）セルはカバーできず、C342 (ツリーID高水位がルート読出不能時に退行して再発行される) に記録。「累計」という言葉は重要である：「今回チェックポイントで発行された最高番号」（非累計）と解釈すると、ツリー ID はルートがリングを周回して追い出される際に再発行され、**その時点で再発行された番号はパブリッシュ済みツリーに属する**ため D6 (スナップショット実現モデル) 判拠 9 に正面から抵触 ⇒ 決定事項全体が不成立となる。mkfs 以外の水位から最初のファイルバージョンを発行する際、13本のツリーの番号は水位から連番で発行され、順序は確定項目 11 の一連の 65536..65548（extent、inode、割り当てレコード、会計、中央マッピング、livelist、スパースサイドテーブル、deadlist、`deleted_inodes`、墓石ツリー、系譜ツリー、logged_ops、退役ツリー）に従う。
- **③ `previous_snapshot_txg` はヘッドごとに1部、ツリーテーブルエントリに常駐**（8 バイト）。**各ヘッドにつき1つのスカラー**でありユニットごとに1部ではなく、「このヘッドがどのスナップショットから分岐したか」に答えるものであり、ユニット自身の内容とは無関係 ⇒ D1 (データ移動性 / 逆引きインデックス) 確定項目 4 の判拠に基づきユニットの自己完結セグメントには入らない。1件のツリーテーブルエントリ上に**独立した2つの 8 バイト txg フィールド**が存在する——誕生 txg（自身がパブリッシュされた番号）と `previous_snapshot_txg`（前駆スナップショットの番号）であり、生存ヘッドエントリとスナップショットエントリの双方に定義がある。
- **④ 互換性レベルは「ツリーテーブルエントリ自前の長さ」を採用し、「インデックスノードのエントリごとの自前長さ」は採用しない**：前者は古い読者が未知のツリーを**スキップ**することを可能にする；後者は古い読者が新ツリーのノードを**展開**できるようにするが、**消費者が存在しない**——コストが高いと判定されたのではなく、**必要なものを得られない**と判定されたためである。
- **㊀ 未知の会計行に対する古いライターの挙動**（「ツリー ID が未知、統計量ラベルが既知」の側に限定）：**ライターはこのような行をそのまま新世代へ引き継がねばならない**（値は不変、世代番号のみ変更；フロントエンドを持つライターはフロントエンドエントリとして書きパブリッシュに伴いリーフへドレイン、第1版ライターはフロントエンドを持たず（確定項目 10）直接リーフへ落とす、ディスク上配置は同一で誤読なし）。段差行は世代セグメントを変えない（D5 (スナップショット / 空間会計メカニズム) 確定項目 2 の適用範囲「inode 番号高水位の段差行」）；前の版のそのパブリッシュで高水位が変わった場合、同じ行が現行行であり段差行でもある。
- **㊁ flags の残り 15 ビット**：**未知のビットは一律に拒絶——当該エントリは EIO**（オブジェクト単位、プール全体は拒絶しない）。「未知ビットを無視して既知セマンティクスで解釈を継続」は採用しない。それはサイレント誤読である。2つの規律を付帯：**古い読者に見せる必要がある変更はシステム構成の incompat ビットを通し、エントリごとの flags に詰め込んではならない**；**checker は「スキップ」を行ってはならない**——I-3.1 (割り当て済み統計の合致)「割り当て済み空間統計 == **全**参照を実際に走査して得られた合計」には限定がなく、checker が未知のツリーをスキップすると走査合計が不足し偽陽性の赤と判定される。未知ツリーに遭遇した際の正しい挙動は**「完全走査不能」を報告して停止**することである。
- **ソート契約とネスト階層数**：**エントリはツリー ID 昇順で整列；ネスト階層数にフォーマット上の上限は設けない**。ツリー ID によるソートの理由：5タプル、会計キー、コンテナインデックスの至る所で使用されるキーであり、② により作成順序に従って厳格に単調増加するため。上限を設けない理由：上限の設定は障害モードの増加に等しく、その上限もエントリ幅に伴って変動するため。
- **タイブレーカーは `(誕生 txg, ツリー ID)` を採用、ゼロバイト**：D16 (パブリッシュセマンティクス) で並列打破に `(txg, スナップショット id)` を用いると決定済みであり、② がルートレコード常駐に変更されたことでツリー ID は**作成順序に従って厳格に単調増加**する（水位累計、増加のみで減少しない、クラッシュ時は欠番のみでロールバックしない——I-7.8 (ルートレコードツリー ID 高水位が全プール最大ツリー ID 以上) により判定可能）⇒ `(誕生 txg, ツリー ID)` は作成順序と一致する全順序であり、ツリー ID はもともとエントリ内に存在する。
- **ヘッド ID 8 バイトの値**：inode ツリーエントリは自前のツリー ID（65537）、extent ツリーエントリはそれがサービスするヘッド（65537）、`deleted_inodes`、墓石ツリー、退役ツリーは inode ツリーのツリー ID（65537）、系譜ツリーは 0（プール全体で1部）、logged_ops は 0（整理意図エントリのフィールド表は確定済み、D26 (バックグラウンド整理と配置回収) 確定項目 2 を参照；C364 (整理インテントレコードのフィールドが逐一突き合わせされていない) はフィールド表と実装の逐一突き合わせが未済、I-4.4 (インテントは目標状態のみを記述する) のフィールド白リスト判定は実装済み）、割り当てレコードツリー / 会計ツリー / livelist 共有ツリー / スパースサイドテーブルツリー / deadlist ツリーは 0（帰属なし、D5 (スナップショット / 空間会計メカニズム) が「全プール / 帰属なし」行に定めた規約と同一値）；中央マッピングツリーはツリーテーブルに入らない。D5 (スナップショット / 空間会計メカニズム) 確定項目 9 を参照。⚠️ 括弧内の 65537 はツリー ID が mkfs 水位 65536 から発行された版（新規プールでのファイル作成）での値であり、判拠は関係性であってリテラル値ではない：inode ツリーエントリのヘッド ID は自前のツリー ID に等しい；extent ツリー、`deleted_inodes`、墓石ツリー、退役ツリーエントリのヘッド ID は同一ツリーテーブル内の inode ツリーエントリのツリー ID に等しい；系譜ツリー、logged_ops、割り当てレコードツリー、会計ツリー、livelist 共有ツリー、スパースサイドテーブルツリー、deadlist ツリーは 0 を書く。ツリー ID が mkfs 水位から発行された版でない（水位引き上げ済み、またはクローンヘッドの inode ツリー）場合、ヘッド ID はその表内の inode ツリーのツリー ID に追従し、65537 ではなくなる（D5 (スナップショット / 空間会計メカニズム) 確定項目 9「書き込み可能ヘッドの ID = そのヘッドの inode ツリーのツリー ID」）。これを判定するのは I-9.27 (ツリーテーブルエントリのヘッド ID の値) である。
- **予約 76 バイト**：判明している予約者は「1セグメントのバイトは1つの意味のみを持つ」に従って並べられ、**ツリーテーブルエントリ予約の予約者一覧に1件ずつ登録される**（ゲート doc-registries の tree-table-reserve セルが合計を計算：予約合計は 76 を超えてはならず、表内の各項目は未解決の未済の検査または確定項目を指していなければならない）。合計 48、残り 28。

<!-- gate:tree-table-reserve total=76 -->

| 予約者 | どのエントリに常駐するか | 幅 | 出典 |
|---|---|---|---|
| livelist 各ヘッドカウンタ | ヘッドエントリ | 8 | C271 (livelist の FREE の記録タイミングが未定義) |
| 第2破棄ゲートの生存スナップショット数 | ヘッドエントリ | 8 | C272 (第2破棄ゲートに O(1) の検査方法がない) |
| origin リンク / クローン時刻 | ヘッドエントリ | 8 | C271 (livelist の FREE の記録タイミングが未定義) |
| スナップショット系譜の区間ラベル | スナップショットエントリ | 16 | C144 (スナップショット系譜の区間ラベルにキャリアがない) |
| `children`（子スナップショット数） | スナップショットエントリ | 8 | D5 (スナップショット / 空間会計メカニズム) 確定項目 18 |

⚠️ **inode 番号高水位は表内にない**：C143 (inode番号高水位がクラッシュリカバリで旧ルートに退行した後に再発行される) の 2026-09-16 決定「水位は移動せず、従来通り会計に常駐」（D5 (スナップショット / 空間会計メカニズム) 確定項目 4 第12項）⇒ ここのバイトは予約しない。

**新規プールでのファイル作成のバイト変更：あり**——ツリーテーブルユニット t12 のエントリ幅フィールド（200）、宣言長（12 × 200 = 2400）、および12件のエントリの配置。

**対象範囲**：⚠️ **㊀ の射程は「ツリー ID が未知」の半分に留まり、もう半分の処方は逆である**：**統計量ラベル**が未知の半分は C71 (未知の統計量ラベルのサイレント無視) が管轄し、「マウント時に未知の統計量ラベルに遭遇した場合は**compat_ro に従い書き込み可能マウントを拒絶しなければならない**」と逐字要求している——㊀ はそれをカバーも覆しもしない。両者を分離できるメカニズム根拠：D5 (スナップショット / 空間会計メカニズム) 確定項目 5 のキー固定幅 ⇒ ラベルの**意味**を知らないライターでも固定オフセットに従って各セグメントを切り出すことができる；成立するメカニズムは確定項目 1「会計エントリは冪等完全値を取る」であり、行の値が運搬者にとって不透明であっても運搬可能であり、ライターはそのツリーを知らなければ変更できない ⇒ 値はもともと不変である。運搬しない場合のコスト：D5 (スナップショット / 空間会計メカニズム) 確定項目 2 で各ルートの会計ツリーはその世代の行のみを収容しパブリッシュごとに全行を書き直すと決定 ⇒ ライターが未知のツリーの行は書き直されず次回のパブリッシュで消失する。⚠️ **露出面は極めて狭い**：ツリー次元を持つ統計量は1つのみ（D5 (スナップショット / 空間会計メカニズム) 確定項目 4 第12項、書き込み可能ヘッドに対してのみ維持）⇒ 書き込み可能ヘッド以外のツリーは1行も持たない；ツリー次元を持つ統計量をさらにもう1つ追加すると、非運搬による損失量はツリー数に比例して増大する。⚠️ **㊁ の拒絶でデータが失われないことは ④ のエントリ自前長さに依存している**：エントリは EIO となるが**位置は残る** ⇒ ライターは「そこに読めないツリーが存在する」ことを認識 ⇒ それを跨ぐ deadlist マージを行わない；長さフィールドがなければ次のエントリを特定できず、この推導は成立しない。⚠️ **㊁ は本プロジェクトにおける他の2つの flags フィールドで確定したポリシーと同一である**：D18 (ブロックに含める情報)「オフセット 7 は 8 個の flag ビット……読者は非 0 ビットに遭遇した場合一律拒絶……本プロジェクトはこれら 8 ビットに未知ビット無視の経路を残さない」；確定項目 6「flags / パディング / 予約の3セグメントは同一ポリシー：常に 0、読者が非 0 に遭遇した場合 ⇒ **当該レコードは EIO**」、I-9.7 (レコードの3セグメントが常にゼロ) として設定済み ⇒ 3つの flags フィールドは1条に統合され、なぜ異なるのかを説明する必要はなくなる。⚠️ **「ビット」と「種別」は2本の軸である**：D15 (フォーマット凍結ポリシー) 確定項目 2 の compat（新規キースペースは古い読者にとって新ツリーが見えないだけ）は**種別**を管轄し、㊁ は**ビット**を管轄する——古い読者が知っているつもりのツリーにビットが追加された場合、そのセマンティクスは変更されており、それは追加ではない。⚠️ **ネスト階層数に上限を設けないことの代償は性能の崖である**。`.claude/rules/fs-design.md` 厳格要件 4 に従い可観測でなければならない：「パブリッシュごとの COW ツリーテーブルユニット数 = 階層数」をランタイムで読み出せる必要がある——誤った階層に進んでもエラーは出ず、観測手段がなければ永久に発見できない。⚠️ **① が提供するテーブルは不十分である**：D5 (スナップショット / 空間会計メカニズム) が点名した「全ルートの txg リスト」は D6 (スナップショット実現モデル) 確定項目 1（ヘッドごとに自前のツリー1本）より前に書かれたものであり、D6 (スナップショット実現モデル) は反例によってその前提を打ち破った（メインライン txg=10 で書き込み可能スナップショット X 作成、X 内で txg=30 でブロック書き込み、メインライン txg=35 でスナップショット Y 作成、D5 (スナップショット / 空間会計メカニズム) の条件では「Y がそれを参照」と判定されるが Y からそれは全く見えない）⇒ 必要なのは「各ルートに1つの txg スカラー」ではなく**系譜**である；誕生 txg の 8 バイトは依然不可欠であるが（I-3.5 (参照区間の正確性) は txg 昇順で全スナップショットルートと生存ヘッドを列挙する必要があり、I-3.6 (deadlist の緊密境界) は `prev(S).txg` を必要とする）、不足しているのは**その上にさらにラベル階層を追加すること**である（ネスト区間ラベルは2つの数値であり、系譜の変化に伴い遡って書き換える必要があり、「誕生 txg は一度書いたら動かない」とは全く異なるフォーマット性質を持つ）。同一前提の3つの消費者の中で決定事項が実現したのは I-3.7 (生存スナップショットの列挙可能性) の1つのみである：I-3.5 (参照区間の正確性) の判定可能形式は「全ルートが txg に従い単一の全順序に並ぶ」ことに基づいており、系譜がツリーを形成した後は恒偽となる；D18 (ブロックに含める情報) の世代間回収は同一の区間テストを通り、方向は過剰判定 ⇒ 回収過少 ⇒ リークである。⚠️ **③ は生存ヘッドヘッダの半セルを補完したに過ぎない**：D5 (スナップショット / 空間会計メカニズム)「各書き込み可能ヘッドは `previous_snapshot_txg` **と deadlist 1部**を保持する」「スナップショット作成時に**生存ヘッドの deadlist を新スナップショットへ丸ごと引き渡し**、生存ヘッドは空のものに交換する」⇒ スナップショットエントリも deadlist を指し示すことができなければならないが、deadlist のディスク上配置先はリポジトリ全体に一箇所もない。⚠️ **丙-2 で唯一カバーできないセルはエラーではないと判定される**：あるチェックポイントがツリー ID 42 を割り当て、42 を持つユニットヘッダを書き出したが、ルートが書けないままクラッシュした場合 ⇒ 水位はそのルートと共に失われる ⇒ 次回マウント時に 42 が再発行され、ディスク上に 42 を持つ孤児ユニットが残存する；これをエラーと見なさない根拠は、孤児と生存版のインスタンス世代番号が必ず一致しないこと（D18 (ブロックに含める情報) 確定項目 11「1チェックポイントを無効化する際は必ずインスタンス世代番号を変更する」）、I-1.2 (ブロックヘッダ書き込み順序がパブリッシュ済み)「パブリッシュ済みと判定されないものは未パブリッシュの不完全トランザクショントラッシュまたは破棄されたタイムラインの残存物である」、スキャン再構築の2つのゲートと孤児回収条件がいずれもツリー ID を見ないこと、孤児と生存版が5タプルを共有することはもともと常態であること（C113 (スキャン再構築時に複数世代ユニットの現行バージョン判定に入力がない) 確定 P1 が 10 バイトの書き込み順序を追加したのはまさにこのため）、D6 (スナップショット実現モデル) 判拠 9 の理由文が「ツリー内のキーが可視化される」と述べており孤児はいかなるツリー内にも存在しないことである。これにより D6 (スナップショット実現モデル) 判拠 9 の射程はその理由文に従って「**パブリッシュ済み**のスナップショット識別子は単調で回収されない」へと絞り込まれる（2026-09-06 ユーザー決定、同一回）。⚠️ **inode 番号高水位は別回答を与える**：D5 (スナップショット / 空間会計メカニズム) 確定項目 4 第12項「**保持**（書き込み可能ヘッドに対してのみ維持）」⇒ 各書き込み可能ヘッドに1つのスカラーであり件数はヘッド数に伴い増加する一方、ルートレコードも固定長スロットである ⇒ 丙-2 は同様に移行できない；2026-09-16 の決定で水位は移動せず、一意性を「パブリッシュ済みの (inode 番号, オブジェクト誕生世代) は再利用しない」に絞り込んだ（確定項目 6）。⚠️ **ツリーテーブル容量が2箇所で異なるエントリ幅で計算されている**：E79 (ルートレコードの容量) と D22 (ユニット原子性をどう合成するか) 確定項目 7 が報告する各層のツリー数は別モデルである（ツリーテーブルエントリを「ツリー ID 8 + ルートポインタ 59」= 67 バイトとしている）。方向は不変（間接層は回避できず、1層を超えるとさらにネスト）であるが、閾値は D22 (ユニット原子性をどう合成するか) 本文の記述より低くなる。⚠️ 予約領域内の判明している2件の予約（deadlist の配置先、スナップショット系譜のネスト区間ラベル）は**敢えてフィールドとして命名していない**——命名は2つの未定事項を一括確定することに等しく、それは2つの決定である。ソート契約（① ツリー ID 厳格昇順、② 同時発行番号順）は I-9.16 (ツリーテーブルエントリがツリー ID 厳格昇順かつ同時発行番号順) により判定：再構築とコールドリードはツリーを読む前に拒絶し、プールレベル checker は別途判定する（実審 A3b Q2 およびツリーテーブルソート実装、ユーザー 2026-09-27 不変条件設定および同期決定）。第1版で同時に生存する読み取り専用スナップショットは最大 10 個（ユーザー決定、定数 `SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION`；ツリーテーブル単一ノード上限は 34 個）。スナップショットはコード 14 エントリのツリー番号で指定される（第1版にはディレクトリがない）。生存スナップショット（現行版または対象版のツリーテーブルにコード 14 / 15 エントリが存在）がある場合、管理者ロールバックは一律拒絶される（ユーザー決定、D23 (ジャーナルの役割とフォーマット) 確定項目 14）。

**根拠**：

- E112 (古いライターが未知のツリーに遭遇した場合)：実行前に固定された判拠と受入条項下での6セルのサイレント結末と運搬コスト——`Skip + Carry` はマウント可能なものの中でサイレント結末ゼロ、`Ignore` のセルはレコードが誤ったエンコードで解釈され、`Refuse` は一切マウントできず D15 (フォーマット凍結ポリシー) 確定項目 2 の compat グレードと相容れない；運搬コストは世代ごとに2行余分に書くだけであり、実行前に定めた破棄閾値はトリガーされなかった。
- E113 (未知のツリー・アームセット補完)：アームセットを 5 flags × 3 会計に補完し選択法を実行前固定の辞書順に変更した後、`RejectEntry` は全スキャンポイントで一度も負けず、`Skip` は第1関門で敗北（不可視スナップショットにより deadlist マージが隣人を誤認）、`Ignore` は第2関門で敗北、攻撃側が提案したビット単位グレード `PerBit` は第3の挙動を持たないことで敗北——㊁ はこれにより「当該エントリをスキップ」から「当該エントリ EIO」へと再決定された；会計の次元は `Exempt` ではなく `Carry` を採用（当時 `Exempt` は D5 (スナップショット / 空間会計メカニズム) 確定項目 2 の K 世代で例外を設ける必要があり、そのコストは1セルも占有していなかった；2026-09-23 K 世代ポイント削除が撤回され、`Exempt` アームは対象を喪失）。
- ユーザー決定 2026-09-06：4つの質問（スナップショットのツリーテーブル格納、ツリー ID 高水位のルートレコード常駐、`previous_snapshot_txg` の各ヘッド1部保持、互換性グレードの自前エントリ長採用）、㊀ ㊁、ソート契約とネスト階層数、タイブレーカー、予約領域 32 バイト採用（原語は変更履歴）。
- ユーザー決定 2026-09-23：② の max にリング内レコードの新ルートセグメントが保持する水位もマージ（C342 (ツリーID高水位がルート読出不能時に退行して再発行される) 前提の質問；C511 (ファイルなし版へロールバック後の誕生世代の接続) ステップ 3 の実装担当が先にこちらを選択し、ユーザーが同日追認、原語「journal ログはまさにツリーが完全に書き込まれることを保証するためのものである」）、原語は変更履歴；番号発行順序はメイン agent が同日フォーマット定数順序に従って決定、0巡攻撃済み。
- ユーザー決定 2026-09-14：ツリー ID をエントリ先頭へ移動、ルートポインタ 83 → 86、予約領域から 8 バイトを切り出してヘッド ID とした（原語は変更履歴；根拠は確定項目 11 がコード 2 エントリ内でキーは一律先頭 key 幅バイトと決定したことであり、ツリー ID が先頭にないと汎用デコード規則に例外を追加せざるを得なくなるため）。
- ユーザー決定 2026-09-16：エントリを 148 から 200 へ拡幅、予約 76、代償としてユニットあたり 109 ツリーから 81 ツリーへ減少、2層上限が 11881 から 6561 へ低下、ユーザーは承知の上で受入（原語は変更履歴）。
- 判決 `research/prompts/c673-clauses-r1-main-verification.md` 第2節 K1：ヘッド ID は同表関係を読み、リテラル 65537 は読まない——リテラル読みは正当な高水位履歴で誤って4条を赤とし、さらにヘッド ID に 65537 をハードコードした不正実装を通してしまう（攻撃側がレプリカで計測済み、メイン agent が再実行し確定）；これを判定する I-9.27 (ツリーテーブルエントリのヘッド ID の値) は表に従って判定し、ツリーテーブルにエントリがある場合にのみ判定する。
- 退役ツリーのヘッド ID に inode ツリーのツリー ID を書き込み、ツリーテーブル宣言長 12 × 200 = 2400：判決 `research/prompts/m3-retire-r1-main-verification.md` 第3節第1条（ツリーテーブルエントリのヘッド ID に inode ツリーのツリー ID を書き込み、墓石ツリーと同等、0巡攻撃済み）；ユーザー決定「プール開設時に作成（推奨）」、原語は変更履歴。
- 判決 `research/prompts/m4-snapshot-r1-main-verification.md` 第3節 S1（各スナップショットにつき新種別のツリーテーブルエントリ2件、甲は脱落；単一ノードツリーテーブル上限 34）、S8（スナップショット存在時のロールバック拒絶）および第7節ユーザー決定（分岐 1 上限 10、分岐 3 スナップショット存在時は拒絶；分岐 5 スナップショットはコード 14 のツリー番号で指定はメイン agent が決定）、原語は変更履歴。

**未済**: C144 (スナップショット系譜の区間ラベルにキャリアがない)、C145 (deadlist にディスク上配置先がない)、C147 (ツリーID高水位の不変条件が未作成)、C148 (ツリーテーブルエントリ拡幅により間接層閾値が2層に圧縮)、C149 (ルートスロット再発行後に birth 計測条件が1段緩和)、C156 (レコードエンコード誤読の不変条件がない)、C158 (checker が未知ツリーをスキップすると偽陽性を判定)、C159 (E112 のアームセットが3条不足し天秤にマウント不能のコストが含まれない)、C342 (ツリーID高水位がルート読出不能時に退行して再発行される)、C675 (ツリーテーブル内に同一種別のツリーが高々1本であることの不変条件がない)、C865 (ツリーテーブルエントリの未知 flags ビットでツリーテーブル全体が読めなくなる)、C866 (F2 のロールバックがバイト消失と内容拒否を区別しない)。

#### 確定項目 9：ツリー種別コードの登録表

**決定事項**：ツリーテーブルエントリにおける「ツリーの種別」の 2 バイトの値の唯一の登録場所は、確定項目 9 の本節の登録表である；**新規ツリーは追加のみとしコードを再利用しない**。0 は「なし」として予約される。

| コード | ツリー | 出典 |
|---|---|---|
| 0 | なし（ツリーテーブルエントリは使用しない） | — |
| 1 | extent ツリー | D4 (チェックサム位置) 確定項目 7 |
| 2 | inode ツリー | 確定項目 6 |
| 3 | 割り当てレコードツリー | D3 (領域割り当て) 確定項目 3 |
| 4 | 会計ツリー | D5 (スナップショット / 空間会計メカニズム) 確定項目 3 |
| 5 | 中央マッピングツリー | D19 (ブロックポインタの構造と幅の予算) 確定項目 5 |
| 6 | livelist 共有ツリー | D6 (スナップショット実現モデル) 確定項目 2 |
| 7 | スパースサイドテーブルツリー | D5 (スナップショット / 空间会計メカニズム) 確定項目 6 |
| 8 | deadlist ツリー | D5 (スナップショット / 空間会計メカニズム) 確定項目 11 |
| 9 | `deleted_inodes` ツリー | 確定項目 16 |
| 10 | 墓石ツリー | D18 (ブロックに含める情報) 確定項目 24 |
| 11 | 系譜ツリー | D18 (ブロックに含める情報) 確定項目 24 |
| 12 | logged_ops ツリー | 確定項目 5 |
| 13 | 退役ツリー | D18 (ブロックに含める情報) 確定項目 11 |
| 14 | スナップショット inode ルート | 確定項目 8 |
| 15 | スナップショット extent ルート | 確定項目 8 |

新規プールでのファイル作成で書き出される **12** 件のツリーテーブルエントリはこの表に従う（コード 1..4、6..13 各1件；中央マッピングツリーのルートはルートレコードに常駐しツリーテーブルには入らない、D19 (ブロックポインタの構造と幅の予算) 確定項目 11；コード 5 は登録のみ）；E142 (新規プールファイル作成のドライラン) の `TREE_KIND_*` 定数はコード 1..8 の8つの値である（コード 9..15 は持たない）；`crates/singlefs-core/src/records.rs` と checker の `walk.rs` は同一組の定数を使用する（コード 5 はツリーテーブルに入らないためそこには存在しない）。

**対象範囲**：本項は「ツリーの種別」セグメントの値のみを登録し、ツリー ID は別の量であり両者の値域は交差しない（ツリー ID ≥ 65536、値は確定項目 11 にある）。D18 (ブロックに含める情報) 確定項目 11 のユニットクラス登録表とは別表であり、別概念である（ユーザーは統合を採用しない）。コード 8（deadlist）は livelist 共有ツリー、スパースサイドテーブルと同一の先例である——いずれも「スナップショット破棄時に読み取る必要があり、破棄において全ディスクスキャンが許されない」構造であり、day-1 にツリーテーブルに登録しておくことで最初のスナップショット作成時に遅延作成するよりも「初回ツリー作成」パスを1本削減できる；エントリ形態は別途 D5 (スナップショット / 空間会計メカニズム) 確定項目 12 に設定。**新規プールでのファイル作成のバイト変更：あり**——コード 8 の追加によりツリーテーブルが 6 件から 7 件に変更、ツリー ID 高水位が 65543 から 65544 に変更（ツリー ID は 65536 から開始され、deadlist は 65543）。

**根拠**：

- 実験なし：コード値は規約であり、計測可能な量はない；登録表はユーザーにより 2026-09-13 決定（コード 0..7）、2026-09-14 コード 8 追加を決定。いずれも三者論証を経ておらず覆し可能、原語は変更履歴。
- コード 9..12：判決 `research/prompts/m3-blanks-r1-main-verification.md` 第2節種別コード行（2脚が個別に導出）；ツリー ID と種別コードの値域が交差しないことはユーザー決定 2026-10-01 で採用（分岐票 `m3-blanks-r1-forks.md` の「種別コードとツリー ID の分離」行、原語は変更履歴）。
- 判決 `research/prompts/c673-clauses-r1-main-verification.md` 第2節 K2：新規プールでのファイル作成が書き出すツリーテーブルエントリ数は実装の現行値に従う（判決当時 11 件、退役ツリー追加後 12 件、`crates/singlefs-format/src/lib.rs` の `NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES`）。
- コード 13 退役ツリー、新規プールでのファイル作成で 12 件書き出し：判決 `research/prompts/m3-retire-r1-main-verification.md` 第3節第1条（種別コード追加登録、ツリーテーブル末尾に配置、day-1 ゼロルート登録、値は墓石ツリーおよび系譜ツリーの先例に従う、0巡攻撃済み）；ユーザー決定「プール開設時に作成（推奨）」、原語は変更履歴。
- コード 14 スナップショット inode ルート、コード 15 スナップショット extent ルート：判決 `research/prompts/m4-snapshot-r1-main-verification.md` 第3節 S1（ツリーテーブルエントリに2種の新種別を採用、追加登録、コード非再利用；甲脱落で2脚一致、具体フィールド0巡）。

**未済**: C315 (ツリー種別コードに登録表がない)。

#### 確定項目 10：第1版の会計はリーフへ直接書き込み、seq の値生成規則

**決定事項**：**第1版の会計は直接会計ツリーのリーフへ書き込み、write buffer フロントエンドは通らない**；D5 (スナップショット / 空間会計メカニズム) 確定項目 3 第3条は「フロントエンドは後続の最適化」と解釈する。**`seq` = 同一の `(タグ, ツリー, デバイス, 世代)` が1回のパブリッシュ内でツリーに書き込まれた回数**（ランタイムで当該統計量が変更された回数ではない）、**1 から開始**；第1版は挿入中 COW を禁止（D19 (ブロックポインタの構造と幅の予算) 確定項目 12 の固定点：各コード 2 / コード 3 は1インスタンス1チェックポイントにつき1回のみ書き出され、パブリッシュ末尾で一括 COW）⇒ 各キーはパブリッシュごとにツリーへ1回のみ書き込まれる ⇒ **常に 1**；マージ規則が最大値を取る統計量も同様に書き込む。

**対象範囲**：本項が定めるのは第1版がどのパスを通るかと `seq` の数え方であり、`seq` の幅や配置先（確定項目 7）、フロントエンドとノードバッファの連携（確定項目 12、第1版は両レベルとも無効）は定めない。固定点の緩和、フロントエンドまたはノードバッファの開放後に「常に 1」が依然成立するか否かは確定項目 13 の状態縮小が引き続き管轄する。

**根拠**：

- 実験なし：第1版がフロントエンドを通るか否かは実装スケジュールの選択であり、計測可能な量はない；ユーザーにより 2026-09-13 コーディング前の総合レビューで決定、三者論証を経ておらず覆し可能、原語は変更履歴。

**未済**: なし。

#### 確定項目 11：コード 2 のペイロード配置とキー比較順序

**決定事項**：コード 2 ノードヘッダは自己記述（予約 2 の後にエントリ数 2 + エントリ幅 2、key 幅 1 は key 区間の前に配置、オフセット表は D18 (ブロックに含める情報) 確定項目 18 を参照）、基本バイト数は 81 → 86；29 バイトの予約領域を含めたヘッダ = `115 + 2 × key 幅`。エントリ領域は予約領域を含むヘッダ末尾（`115 + 2 × key 幅`）から開始し、**固定幅エントリが連続配列される**、**宣言長 = エントリ数 × エントリ幅**；エントリ以降の空きバイトは常に 0 でありペイロード CRC に参加する。ペイロード CRC の範囲は `[86 + 2 × key 幅, 16384)`（開始位置は平文ヘッダ末尾であり、コード 1 の 105、コード 3 の 107 と同一の計測条件）。**エントリ内のキーは一律にエントリの先頭 key 幅 バイトとする**——「エントリ幅を読み取る → エントリ幅に従ってエントリを切り出す → 先頭 key 幅 バイトをキーとして取得する」は全コード 2 ツリーに対して成立する汎用デコード規則であり、スキャン再構築パスにおいて「このツリーのキーがエントリの何番目に並んでいるか」という別表を引く必要がない。キーの全順序はフィールドごとの符号なし整数として左から右へと比較し、**ディスク上のリトルエンディアン格納は memcmp 順序を構成しない**。ツリー ID は 65536 から連番で割り当てられる（種別フィールド 2 バイトが取り得ない最小値、両者の値域は決して交差しない）：extent 65536、inode 65537、割り当てレコード 65538、会計 65539、中央マッピング 65540、livelist 65541、スパースサイドテーブル 65542、deadlist 65543、`deleted_inodes` 65544、墓石ツリー 65545、系譜ツリー 65546、logged_ops 65547、退役ツリー 65548；mkfs 時の水位は 65536、最初のファイルバージョン作成後は 65549。

**新規プールでのファイル作成のバイト変更：あり**——6個のコード 2 ユニットのエントリ領域開始位置（`115 + 2 × key 幅`：inode ツリールート 131、extent 163、割り当てレコード 135、会計 159、マッピング 169、ツリーテーブル 131）、宣言長、およびツリーテーブルエントリのフィールド順序；ツリー ID フィールド（ツリーテーブルエントリ内のツリー ID とヘッド ID、コード 2 ヘッダ内のツリー ID、会計キーのツリー ID セグメント）およびツリー ID 高水位の値もツリー ID が 65536 から開始されることに伴い変更される。

**多層コード 2 ツリーの伸長・収縮手順**（会計ツリー、中央マッピングツリー、および確定項目 14 に従い引き続きコード 2 btree を使用するツリー）：① 1回のパブリッシュ内で分割・新設されたノードは他ユニットと同一セグメントに配置され、ルートからリーフまでのパス全体が通常通り COW され、書き込み順序ステップやバリアは追加されない；② リーフに収容しきれなくなった場合は中央で分割する；③ 収縮は空ノードの除去のみ行い、ルートに子が1つのみ残った場合は階層を1段下げ、「半分未満でのマージ」やエントリ借用は行わない；④ 区切りキーは追従して維持する：最左の区切りキーより下に挿入された場合はそれを押し下げ、エントリが兄弟間で移動した後は右の子の区切りキーをその新たな最小キーに変更する；層ごとに判定可能：区切りキー ≤ 子ノードヘッダの最小キー、かつ > 左隣の子ノードヘッダの最大キー（D18 (ブロックに含める情報) 確定項目 18 のキー区間）；⑤ ツリー高 = ルートノードコード 2 ヘッダ内の階層 + 1。

- **1回のパブリッシュ内では削除を先に行い挿入を後に行う**：前版にあり現行版にないキーをまずキー昇順で削除し、その後現行版で新たに現れたキーをキー昇順で挿入する；形状のみを決定し、書き込み順序は決定しない（書き込まれるのは最終状態である、確定項目 13）。
- **ディスクから再構築されたツリーの区切りキーが破損していた場合**：再構築はツリーを構成可能な要素（階層、ノードが空でないこと、内部エントリ幅、重複参照がないこと）のみを検証し、区切りキーは検証しない；次回のパブリッシュで区切りキーに従って削除できなかった場合は計画を返却し、書き込み可能マウントは番号取得前に拒絶され、ディスク上は変更されない。このようなディスクに遭遇した後にどうするか（修復、読み取り専用マウント）は未定。
- **リーフエントリ幅はツリー読み取り側では判定しない**：ツリー読み取り側はキーを切り出すのみであり、リーフエントリがフィールド表より狭いことはエントリを展開する側が各自報告する；内部エントリが key + 86 より狭いことはツリー読み取り側が報告する。

**内部ノードのエントリ = 自ツリーキー + 子ポインタ 86**：会計ツリー 22 + 86 = 108、中央マッピングツリー 27 + 86 = 113。割り当てレコードツリーと extent ツリーは確定項目 14 に従いキー空間によりアドレッシングされる別構造を採用し、構造は三者 `m2-keyspace-r1` で判定；コード 2 btree に従って構築される場合、それらの内部エントリは 96 および 110 である（extent 110 はユーザーが 2026-09-24 に事前決定）。

<!-- format-const: ACCOUNTING_INTERNAL_ENTRY_BYTES = 108 -->
<!-- format-const: CENTRAL_MAPPING_INTERNAL_ENTRY_BYTES = 113 -->

**対象範囲**：本項はコード 2 ノードのペイロード配置とキー比較順序を管轄し、ヘッダ内の各フィールドのオフセット（D18 (ブロックに含める情報) 確定項目 18）や各ツリーのエントリ内にキー以外に何が含まれるか（各ツリー自前の項目）は管轄しない。「キー先頭」により再配置が必要となった唯一の対象はツリーテーブルエントリであり、本条に従いツリー ID 先頭に変更済みである（確定項目 8）。「固定幅エントリの連続配列」と「宣言長 = エントリ数 × エントリ幅」は中央マッピングエントリが固定幅 55 に戻ったため変更不要（D19 (ブロックポインタの構造と幅の予算) 確定項目 10）——エントリ幅フィールドは u16 が1つしかなく、クラスごとに2つの幅を持つ場合はこれら2文を維持できない。2026-09-14 に補完された2つの決定はいずれもユーザーポップアップ決定であり、三者論証を経ておらず覆し可能。⚠️ 分割、収縮、分割点、書き込み順序の4項目は0巡攻撃済みであり、三者プロトタイプは1回のみサンプリングを実施；「半分未満でのマージ」と「追記時の末尾切断」はクラッシュ履歴上で選択された2項目と 0 対 0 であり、非選択の理由はユーザーの取捨選択であって安全性ではない。「分割のための単独パブリッシュ」は却下：構造変更のみのパブリッシュは D16 (パブリッシュセマンティクス) 確定項目 1 の「直近4つのロールバック可能な異なる状態」を圧迫してしまう。inode ツリーのリーフはコード 3 パッキングレコードユニットでありコード 2 ノードではない：「多層コード 2 ツリーの伸長・収縮手順」における ② の「リーフに収容しきれなくなった場合は中央で分割」および ③ の「半分未満でのマージやエントリ借用は行わない」はコード 2 ノードを管轄し、inode ツリーのリーフ層には及ばない；リーフ層の末尾分割、左による右の吸収、左による右からの借用（均等借用）は確定項目 6 に従う。

**根拠**：

- E145 (コード 2 自己記述ヘッダとマッピングツリーキー幅のコスト)：まさにコード 2 自己記述ヘッダに追加すべきセグメントとマッピングツリーキー幅のコストを計測——86 に増加させた後もファンアウト、ツリー高、ヘッダ解釈の各節でグレード低下はなく、自己記述は採用価値がある。
- ユーザー決定 2026-09-13：コード 2 ヘッダ自己記述、基本 86、エントリ領域開始位置、ペイロード CRC 対象範囲、キー比較順序（原語は変更履歴）。
- ユーザー決定 2026-09-14：エントリ内のキーは一律に先頭 key 幅 バイトとする；「固定幅エントリの連続配列」と「宣言長 = エントリ数 × エントリ幅」はそのまま成立（原語は変更履歴）。
- ユーザー決定 2026-09-24：extent ツリー内部エントリ 110 バイト（原語は変更履歴）；E157 (並行線1、2の条項の計数モデル) は 110、136 の2つのサンプリングポイントで結論が反転しない。
- 実験なし：分割、収縮、分割点、書き込み順序の4項目は三者プロトタイプ（`research/prompts/m2-treesplit-r1-opus-model/`）が同一のクラッシュ履歴上で 0 対 0 であった後、ユーザーにより選択されたものであり、実験番号は設定されていない；区切りキーの追従維持はその巡でメイン agent がヒットに基づき決定（プロトタイプ t1.out：D8 (コアインデックス構造) 確定項目 6 の「区切りキーは作成後不変」を踏襲すると 24 セルすべて赤、追従維持で 24 セルすべて 0）、判決 `research/prompts/m2-treesplit-r1-main-verification.md`。
- ユーザー決定 2026-09-24：書き込み順序は同セグメント通常 COW、空ノードのみ除去しルートに子が1つの場合は階層低下、中央分割、内部エントリ = 自ツリーキー + 子ポインタ 86（原語は変更履歴）。
- `m2-treesplit-implementer-report.md` 第5節 B1、B5、B6。
- ユーザー決定 2026-10-01：ツリー ID は全体を 65536 からに移動し、種別コードの値域と交差しない（分岐票 `m3-blanks-r1-forks.md` の「種別コードとツリー ID の分離」行、原語は変更履歴）。
- 判決 `research/prompts/m3-blanks-r1-main-verification.md` 第2節種別コード行：新ツリーの種別コードは 9..12 を採用（2脚が個別に導出）。
- 退役ツリーのツリー ID 65548 および初回パブリッシュ後の水位 65549：判決 `research/prompts/m3-retire-r1-main-verification.md` 第3節第1条および第5節（値は先例に従う、0巡攻撃済み）；ユーザー決定「プール開設時に作成（推奨）」、原語は変更履歴。

**未済**: なし（C306 (コード 2 ヘッダが key 幅を自己記述しない) は 2026-10-11 に解決済み、未済の検査解消一覧を参照）。

#### 確定項目 12：write buffer フロントエンドとノードメッセージバッファの連携 —— 1 つのツリーには 1 レベルのみ許可

**決定事項**：write buffer フロントエンドを通る3ツリー（会計 / 逆引きインデックス / LRU）は **ε が常に 0 であり、内部ノードにメッセージバッファを残さない**；これら3ツリーへの書き込みは**一律フロントエンドに入る**——確定項目 8 ㊀ のような古いライターが運搬する会計行を含む：フロントエンドを持つライターはそれをフロントエンドエントリとして書き、第1版ライターはフロントエンドを持たず（確定項目 10）直接リーフへ落とす、ディスク上配置は同一で誤読なし；**パブリッシュ前にフロントエンドをリーフへドレインする**。リーフへの書き出し時には同一キーに対して同一ウィンドウ内の到達順序を比較する；パブリッシュセットのフロントエンドエントリは同一パブリッシュ内でドレインされ、トランザクション単位で分割され、パブリッシュを跨がない（fsync = 1回のフル実行パブリッシュ）。その他のツリーは D11 (インデックスノードにメッセージバッファを残すか) 確定項目 6 のツリー軸に従ってバッファを残すか否かを決定する。**第1版では両レベルとも無効**（確定項目 10、D11 (インデックスノードにメッセージバッファを残すか) 確定項目 7）、新規プールでのファイル作成のバイトは変更しない。

**検査**：フロントエンドツリーの書き込みパスはフロントエンドエントリのみを公開する（タイプ層）；ランタイムアサーション——1つのチェックポイントにおいてフロントエンドツリーのリーフ上の各論理変更（キー → 値）が、このチェックポイントでリーフに入ったフロントエンドエントリの1つに対応すること。ウィンドウ途中で flush されたものとパブリッシュ時にドレインされたものの双方がカウントされ、インスタンス切り替えによる再発行時にも保持される；再編成、インデックス再構築、破損リカバリ再構築の3例外にはそれぞれ「前後の各キーから読み出される値が不変」という条項を付帯する。これはランタイムでのみ可能である：ディスク上ではフロントエンドドレイン経由かリーフ直接書き込みかを区別できない。字面通りに書かれた3種のアサーション（パブリッシュ時ドレインのみを認める、今回の試行のみを認める、物理的新リーフ数による）は正当な履歴において誤報を出す（`c161-r3-model-output.txt` の `G3_publish_only` / `G3_attempt` / `G3_physical` 行）。

**対象範囲**：**コスト**：パブリッシュごとにリーフへドレインする書き出し量は「ルートバッファへのドレイン」形態の 3.4–10.7 倍（上書き負荷で 4.8–10.6 倍、同一成果物 `part` 列の第3段）であり、パブリッシュセットのエントリはすべて今回のパブリッシュでリーフに落とさねばならず、予算に応じたバッチ分割ができない；同一のダーティページ予算下でフロントエンドに収容できるエントリ数は1桁少なくなる——`T_dirty` を何に基づいて計算するかの計測条件がない。**得られたもの**：フロントエンドツリー上で同一キーが階層を跨いで並存することがなくなり、読み取り順序が最短となる；陳腐化検出（C52 (陳腐化を正当と見なす)）はこれら3ツリーにおいて対象を持たない。**脱落していないが非選択のもの**：甲-R4（フロントエンドは直接リーフへ落とし、ノードバッファはフロントエンドを通らない書き込みのみを受け入れる）および乙-D（フロントエンドはルートバッファに入り階層ごとに下押しされ、位置により新旧を決定）はいずれも崩壊において1回のみヒットされた。それらの陳腐化検出はメッセージごとにパブリッシュ番号を保持する必要があり（確定項目 7「txg の二重保持禁止」と衝突）、ノード配置は D15 (フォーマット凍結ポリシー) 第4層に入る。**連動事項**：再編成時にノードバッファをどう移動するか（射程はバッファを残すすべてのツリーであり、本項はフロントエンドの3ツリーを省いただけである）；墓石とリーフ上の現在値の縮小起点（確定項目 13）；逆引きインデックス / LRU の時間手がかり；D11 (インデックスノードにメッセージバッファを残すか) 確定項目 6 のツリー軸の分岐変数と確定項目 5 の表現。

**根拠**：

- 実験なし：このセルが計測するのは設計形態が正当な履歴上で識別アームにヒットされるか否かであり、根拠は3巡の三者検証（`research/prompts/c161-r1-main-verification.md`、`c161-r2-main-verification.md`、`c161-r3-main-verification.md` 各自第4節）である——この形態は3巡で試行された全形状（順不同 flush、K世代ポイント削除、分割とマージ、部分パブリッシュ、ドレイン途中のクラッシュ、削除、ツリー高 4、インデックス再構築、インスタンス切り替え、管理者ロールバック）でヒットされなかった；脱落した乙 + seq 全プール単調は2巡でヒットされ（カウンタがディスクに落ちない；インスタンス切り替え時にカウンタが選択ルートから再ロードされる）、さらに 8 バイト seq とディスク常駐カウンタを要する。ユーザー 2026-09-16 最終ポップアップ決定、原語は変更履歴。

**未済**: C161 (バッファ2レベル並存で連携未定)、C345 (逆引きインデックス / LRU の陳腐化検出に時間手がかりがない)、C348 (T_dirty を何で計算するかの計測条件がない)、C349 (再編成時にノードメッセージバッファをどう移動するかの条項がない)、C383 (同一パブリッシュ内での同キー中間状態の折りたたみ検査欠落)。

#### 確定項目 13：インデックスツリー更新で状態縮小を行う —— 同一パブリッシュ内で同一キーの中間状態はリーフへ書き出さない

**決定事項**：1本のインデックスツリー上の同一キーが同一パブリッシュ内で複数回変更された場合（A → B → C → D → E）、リーフへの書き込みは1回のみ行う：リーフ内のこのキーのエントリは最終状態 E を直接書き込み、A–D はいかなるノードにも書き込まない。「同一周期」とは1回のパブリッシュである：同一の `checkpoint_txg` であり、fsync がトリガーするものも1回のパブリッシュである（D16 (パブリッシュセマンティクス) 確定項目 6）。**E は同一パブリッシュ内の到達順序に従って最後の変更を取得し、値によっては取得しない**；E が墓石である場合、縮小の結果は墓石そのものであり、墓石を数値と見なして A–D と共に最大値を取ることは禁じられる（C383 (同一パブリッシュ内での同キー中間状態の折りたたみ検査欠落) の実測でこのような素朴な縮小は4候補中 16 / 16 でポイント削除を喪失、`b.out`）。**縮小の起点はリーフ上でこのキーが前回のパブリッシュで残した現在値（なければ無し）である**：ドレイン = それを起点に、今回のパブリッシュにおける当該キーのフロントエンドエントリを到達順に順次適用し、墓石 = 削除；起点は A–D より前に数え、同一パブリッシュ内の中間状態の一つとは数えない。フロントエンドを有効にした後、会計行の「前世代の離脱」は墓石として落ち、それが向かうのがこの起点である。到達順序はメモリ内でエントリ自身が保持し（確定項目 15 第3条）、キーソート後の相対位置には依存しない。

**対象範囲**：**全インデックスツリーの全リーフ書き込みパス**。第1版の直接リーフ書き込み（確定項目 10）のほか、将来開放される write buffer フロントエンドドレイン（確定項目 12：リーフ書き込み時に同一キーに対して同一ウィンドウ内の到達順序を比較し、比較された最後の1件が E となる）、ノードメッセージバッファ（D11 (インデックスノードにメッセージバッファを残すか) 確定項目 6 のツリー軸）、挿入中 COW（D3 (領域割り当て) 確定項目 5 で許容され、D19 (ブロックポインタの構造と幅の予算) 確定項目 12 で第1版では採用しないとした実装）はいずれもこの規則に従ってリーフに書き込む。**第1版はすでにこれを満たしている**：D19 (ブロックポインタの構造と幅の予算) 確定項目 12 の固定点により確定項目 10 は「各キーはパブリッシュごとにツリーへ1回のみ書き込まれる」、`seq` は常に 1 を導出する；確定項目 13 はこれを固定点の推論から独立条項へ引き上げたものであり、固定点緩和後も同様に成立し、新規プールでのファイル作成のバイトは変更しない。実装において本日は対象が存在しない：`crates/singlefs-core/src/transaction.rs` の `publish_overwrite` による1回の上書きは1回のパブリッシュ（txg、トランザクション番号が各1増加）であり、「同一パブリッシュ内で複数のトランザクションが同一キーを変更する」パスは存在しない；同一チェックポイント内に2つのオブジェクトを収容するテストケースは異なるキーを変更している。⚠️ **未回答の2項目**：① A–D のツリー外での行方は本項の管轄外である——A–D が同一論理ブロックが複数回上書きされて得られたデータユニットバージョンである場合、D16 (パブリッシュセマンティクス) 確定項目 2 の損失ウィンドウと分割規律「1トランザクション1ユニット」に従い、A–D はそれぞれ1トランザクションであり、それぞれディスクに落ちたデータユニット1個と journal レコード1件を持つ（D23 (ジャーナルの役割とフォーマット) 確定項目 7：1レコード1トランザクション番号）。これら複数のユニットはルート参照を持たず、I-7.4 (直近 K 世代ブロックの未再利用) の保護下にもない。これらを書き込むべきか、書き込んだ場合に同一パブリッシュ内でどう解放するか、会計はどう計算するかについて、決定本文、未済の検査一覧、不変条件リストに管轄する条項がない；② 確定項目 7 のディスク上 `seq` は縮小下で常に 1 となり（確定項目 10 の「常に 1」が第1版から全バージョンへ拡大）、E の認識にはメモリ内の到達順序を用いるため、ディスク上のその 4 バイトの `seq` を依然として `T_dirty` 上限の根拠とするか否か（確定項目 7 の決定事項内「`T_dirty` の上限はこれ以降マウント時判定となる」の一文）について未回答である。⚠️ 「単調統計量は同一パブリッシュ内で減少しないため、2つの取得法で同一の数値が得られる」はメイン agent の推論であり、検査が存在しない。

**根拠**：

- 実験なし：ユーザーが「これは決定事項の記録であり、当面作業および検証は不要」と決定し、三者検証は行われず、実験も作成されず、計測可能な量も存在しない；ユーザー 2026-09-19 の決定と同日の「同一周期 = 1回のパブリッシュ」の確認、原語は変更履歴。A–D を書き出さなくてよい3つの支点は既存の条項である（メイン agent 2026-09-19 その場調査）：D16 (パブリッシュセマンティクス) 確定項目 4 および D23 (ジャーナルの役割とフォーマット) 確定項目 14 第5条（適用の単位は1回のパブリッシュであり、正当プレフィックスが1回のパブリッシュの2トランザクション間で停止した場合そのパブリッシュ全体が適用されないため、replay で得られるのはパブリッシュ境界上の状態のみである）、D16 (パブリッシュセマンティクス) 確定項目 6（fsync がトリガーするものも1回のパブリッシュであり、fsync により確認された状態はすべてパブリッシュ境界上に位置する）、D16 (パブリッシュセマンティクス) 確定項目 14（スナップショットパブリッシュポイントはチェックポイント境界にアラインされ、当該チェックポイントの全データ変更の後に固定されるため、A–D を参照するスナップショットは存在しない）。
- 縮小起点の一文と墓石の縮小方法を確定項目 13 に統合：判定 `research/prompts/m3-close-c693-r2-main-verification.md` 第 2 節 V4 のセル（題を縮めて確定項目 13 に統合、維持、両側一致）と `research/prompts/m3-close-c693-r3-main-verification.md` 第 2 節 T4 のセル（維持；検査はドレインの突き合わせを採り、段差 H が厳密に増加するという判定はマイルストーン十四のツリー分割とともに行う）；改め方は順推側 `research/prompts/m3-close-c693-r2-sonnet-output.md`「V4.5 変更すべき条項、未済の検査」の節に従う。順推側モデル `research/prompts/m3-close-c693-r2-sonnet-model/out/v4_fold_checks.out` 第 1、3 行：単一ヘッドが inode を 2 回作成する履歴で 20 回パブリッシュした後、再構築の配置先による最終状態は 18 行、素朴な最大値取りは 35 行で 17 行が陳腐（第 2 ラウンド判定の検証第 4 条、メイン agent 再実行 identical=1）。

**未済**: C383 (同一パブリッシュ内で同一キーの中間状態縮小に検査がない)。

#### 確定項目 14：1 組の btree 実装 + 複数の独立 keyspace + 2 つのフロントエンド、派生ツリーは異種構造を許容

**決定事項**：**1組の btree 実装 + 複数の独立キースペース + 2つのフロントエンドにより、権威状態のツリーおよび分割に従って構築されるツリーを管轄する；派生状態の割り当てレコードツリーと extent ツリー（D21 (権威態と派生態の境界) 確定項目 5、9）はキー空間によりアドレッシングされる別構造を採用可能であり、フォーマットはコード 2 btree と異なってよい。**

| レイヤ | 役割 |
|---|---|
| 独立キースペース | データ分類ごとに1本のツリー（extent / inode / dirent / 逆引きインデックス / 空き領域 / 会計 …）、相互干渉なし |
| ログ構造化ノード | ノード内の新キーは挿入せず追記し、メモリ内で定期的に再整列；書き込み順序、読み取り専用ロック下でのライトバックが可能 |
| **write buffer** フロントエンド | 高頻度の小規模更新（逆引きインデックス、LRU、会計）を一括集約しソート・重複排除後にバッチ挿入 |
| **key cache** フロントエンド | 同一バッチのキーが反復的に読み書きされるもの（割り当て情報）はハッシュテーブルを通し、ツリー全体探索を行わない |

**2本の派生ツリーの構造**：

- **割り当てレコードツリー**：絶対スロット番号に従って位置アドレッシング。各リーフは固定範囲のスロット `[k × W, (k + 1) × W)` をカバーし、W は偶数を取る（2スロットのデータユニットが決してリーフを跨がない）、かつ W ≤ リーフエントリ容量 812（1レコードが少なくとも1スロットをカバーし、リーフは常に収容可能）；レコードは開始スロットに従ってリーフに配置。上方の各層は位置に従ってアドレッシングされ、子ノードがどの区間をカバーするかは親ノード内でのその位置から算出される。**レコードが存在しない区間 = 完全空き**：そのリーフは書き込まれず、親ノードのそのセルは空のまま残される（ユーザー K1）。checker はノードごとに「このノードがカバーするのはその位置が規定する区間そのものであること」および「レコードの末尾スロットが所在リーフの末尾スロットを越えないこと」を検証する。
- **extent ツリー**：2段構成であり、いずれも位置に従ってアドレッシングされる。上段は inode 番号の位置に従ってアドレッシング；下段は1ファイルにつき1本であり、データユニット番号の位置に従ってアドレッシング（リーフは 144 ユニットをカバー、内部ファンアウト 147）、ホールは欠落として表現される。**データユニットが1つのみのファイルは下段を構築しない**：上段リーフエントリ内にそのデータポインタを直接配置；エントリは「ユニットなし / 下段ルートポインタ / インラインデータポインタ」を区別する1バイトのラベルを保持する（ユーザー K2）。**上段の正規形**：上段の形状はエントリを持つ inode 番号の集合によってのみ決定され、書き込みと削除で同一の規則を共用する：ノードは各エントリ保持 inode のルートパス（そのリーフからルートまでの一連）の和集合を取り、ルート階層は最大番号をカバー可能な最低層を取り、集合が空の場合は階層 0 のゼロエントリルートを取る；完全な書き換え規則は確定項目 16 第8条にある。
- **マウント時の読み込み方法**：割り当てレコードツリーはマウント時に全体をマウント状態へ読み込む（アロケータがもともと必要とする）；extent ツリーはオンデマンドとし、ファイルオープン時に位置に従って辿る（ユーザー K4）。
- **実装値**（実二一で採用、メイン agent 2026-09-25 決定、0巡攻撃済み）：
  - **割り当てレコードツリー**：
    - リーフ幅 W = 812（リーフエントリ容量 (16384 − 135) ÷ 20 そのものを採用）、内部エントリ 96 バイト（key 10 + 子ポインタ 86）、ファンアウト 169。
    <!-- format-const: ALLOCATION_RECORD_TREE_LEAF_SLOTS = 812 -->
    <!-- format-const: ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES = 96 -->
    <!-- format-const: ALLOCATION_RECORD_TREE_INTERNAL_FANOUT = 169 -->
    - 階層 L のノードは W × 169^L 個のスロットをカバー。
    - 内部ノードおよびルートは子を持つセルのみを書き込む：各子につき1件の内部エントリ（key 10 + 子ポインタ 86）、エントリの key はその子が位置に従ってカバーする区間の開始点（子が位置するディスク, その区間の最初のスロット番号）を取り、key に従って整列、子のないセルはエントリを占有しない；読者はこの key に従って子を認識し、スロット番号が子の階層のグリッド上にない、ディスクがプール内にない、または認識された子がこの親ノードの傘下にない場合、このエントリはいかなる子も指さない（`crates/singlefs-core/src/allocation_record_tree.rs` の `child_named_by_entry_key`）。ルートノードヘッダのキー区間はキー空間全体をカバーし、smallest_key = (0, 0)、largest_key = (0xFFFFFFFF, 2^48 − 1)。
    - ルートはキー空間全体をカバーし、ディスクごとに分流；ルート層は Σ_ディスク ⌈ディスク上スロット数 ÷ (W × 169^(R−1))⌉ ≤ 169 を満たす最小の R ≥ 1 を取る。4 GiB × 2 の場合、ルートは第2層、ツリー高 3 となる。
  - **extent ツリー**：
    - 上段リーフエントリ 113 バイト = key 24（(0, inode, 0)）+ ラベル 1 + ペイロード 88。ラベル 0 は全ゼロであり読者のみが認識しライターは書かない；ラベル 1 は下段ルートのノードポインタ 86 バイトに 2 バイトのゼロを加えたもの；ラベル 2 はその1データユニットのデータポインタ 88 バイト。
    <!-- format-const: EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES = 113 -->
    - 上段リーフは 143 個の inode をカバー、内部エントリ 110 バイト（key 24 + 子ポインタ 86）、ファンアウト 147。
    - 上段リーフノードヘッダのキー区間は D18 (ブロックに含める情報) 確定項目 2 の対象範囲段落に従いこのリーフが位置に従ってカバーする区間を書き、リーフ内の最初と最後の内部エントリの key は書かない：閉区間であり、第 k 枚（0から開始）の smallest_key = (0, 143k, 0)、largest_key = (0, 143k + 142, 2^64 − 1)。
    <!-- format-const: EXTENT_TREE_UPPER_LEAF_INODES = 143 -->
    <!-- format-const: EXTENT_TREE_INTERNAL_ENTRY_BYTES = 110 -->
    <!-- format-const: EXTENT_TREE_INTERNAL_FANOUT = 147 -->
    - 下段リーフは 144 個のユニットをカバー、下段 key のバイトオフセット = ユニット番号 × 32634。上段と下段の内部エントリはいずれも、key にその子が位置に従ってカバーする区間の開始点を取る（`crates/singlefs-core/src/extent_tree.rs` の `build_extent_internal_entry`）。
    <!-- format-const: EXTENT_TREE_LOWER_LEAF_DATA_UNITS = 144 -->
  - **パブリッシュパスの固定点**：書き換えセットを推測 → アロケータコピー上で解放を実行（解放によってダーティとなったリーフはまず書き換えセットにマージ）→ 配置先を取得 → ダーティノードを計算 → 集合内になければマージしてやり直し。第 64 巡でも収束しない場合、ディスクへのいかなる書き込みの前に `AllocationRecordTreeRewriteSetDidNotSettle` を返す；64 に根拠はなく、新設の未済の検査を参照。
  - **ディスク上スロット数**：ディスク上の完全スロット数を採用し、ディスク末尾の1スロットに満たない端数は計算に入れない；書き込み側と読み取り側で定義は1箇所のみ（`crates/singlefs-core/src/allocator.rs` の `absolute_slot_count_of_device`）、割り当てレコードツリールート層の両側で同一の数値（実二六で採用、メイン agent 2026-09-25 決定、0巡攻撃済み；テストケース `half_a_slot_past_a_root_level_boundary_leaves_the_writer_and_the_reader_on_the_same_root_level_and_every_tree_written_reads_back`）。
  - **ファイルオープン時に自己申告される読み取り数**：`ExtentTreeReadsAtOpen` が数えるのは読み取った extent ツリーノードの数であり、ブロック層のデバイス読み取り回数ではない；ヒントが古く中央マッピングを経由してロールバックした場合、1ホップごとにデバイス読み取りが2回増加し、2つの数値は一致しない（実二六で検出、メイン agent 2026-09-25 この計測条件を承認、デバイス読み取り回数は別途追加せず、0巡攻撃済み；テストケース `opening_a_file_whose_lower_extent_segment_has_two_levels_reports_as_many_node_reads_as_the_block_layer_counts` が一致セルを固定、`a_leaf_of_a_two_level_lower_extent_segment_with_a_stale_location_hint_reads_back_every_data_unit_through_the_central_mapping` が不一致セルを固定、いずれも `crates/singlefs-harness/tests/tree_nodes_and_the_central_mapping_on_mounted_read.rs`）。
- 両ツリーのノードは同様に D18 (ブロックに含める情報) 確定項目 18 のコード 2 ヘッダ（ツリー ID、階層、キー区間、誕生世代番号）を保持し、ツリー高は D8 (コアインデックス構造) 確定項目 11「ツリー高 = ルートノードヘッダ階層 + 1」に従って読み取る。

**対象範囲**：⚠️ 異種構造が許容されるのは派生状態で再構築可能な2本のツリーのみである：割り当てレコードツリーと extent ツリー；会計ツリー（権威状態）、中央マッピングツリー、ツリーテーブル、インスタンス表、inode ツリーは1組のコード 2 btree に従って構築される（D8 (コアインデックス構造) 確定項目 11 の分割と収縮）——inode ツリーはルートと内部ノードのみがコード 2 であり、確定項目 11 が管轄するのはこの階層である；リーフはコード 3 パッキングレコードユニットであり、リーフ階層の末尾分割、左による右の吸収、左による右からの借用（均等借用）は確定項目 6 に従い、確定項目 11 の「リーフに収容しきれなくなった場合は中央で分割」および「半分未満でのマージやエントリ借用は行わない」は及ばない。異種構造においても以下は依然として必須である：単一ノードの自己記述（どのツリーか、どの階層か、どのキー区間を管轄するか、第何世代か、D18 (ブロックに含める情報)）、破損の検出可能性（D21 (権威態と派生態の境界) 確定項目 11）、各新規書き込み順序のクラッシュポイント再生クラッシュポイントへの導入、checker における別個の判定コード。両ツリーの構造は三者 `m2-keyspace-r1` で判定。⚠️ 「ログ構造化ノード」とは**メモリ内の配置**を指す——ディスク上への各書き出しはユニット全体の COW 書き換えであり、確定項目 11 の単一固定幅エントリ配列に従って書き込まれ（宣言長はエントリ数 × エントリ幅に正確に一致）、複数セグメントの追記 bset ではない。⚠️ **write buffer フロントエンドは第1版では実装されない**（確定項目 10：会計は直接会計ツリーのリーフへ書き出し）；フロントエンドとノードメッセージバッファの連携は確定項目 12、エントリが満たすべき3つの厳格要件は確定項目 15 にある。インデックス構造は多様であってよいがトランザクション層は多様であってはならないという規律は `.claude/rules/fs-design.md` にあり、本項はインデックス側のみを定める。⚠️ リーフ幅 W の具体値、上段ファンアウト、上段リーフエントリの完全フィールド表など、条項に記載のないものは実装担当が上記各条に従って選択し、完了報告に明記し、メイン agent が確定後に本節へ追記する。⚠️ 3種の新規書き込み順序（多層高くなる際に位置 0 の一連のノードが同一パブリッシュで書き込まれる、多層低くなる際に同一の一連が一度に解放される、インラインと下段の相互切り替え）はクラッシュポイント再生へ導入する必要があり、checker の判定は実装に伴い作成される。⚠️ 本節の4項目はいずれも0巡攻撃済みである；「グローバル位置アドレッシング＋インライン」のクラッシュ履歴は未走査（ユーザーがフル実行クラッシュは最後に一括実行と決定）。⚠️ 却下された候補：グローバルキーソート（上段の分割が必要）、ファイルごと1本のルートを inode レコードに格納（凍結済みの inode レコードの変更が必要）、グローバル位置アドレッシングでインラインなし（1回のマウント内で14625個目の小ファイルまで連続作成すると偽の満杯を報告し、D3 (領域割り当て) 確定項目 9 第1条に違反）——理由は判決第3節を参照。

**根拠**：

- 実験なし：1組の btree 実装に複数 keyspace を組み合わせる形態には計測可能な量はない。根拠は 2026-08-29 に `linux-6.17/fs/bcachefs/bcachefs_format.h` の `BCH_BTREE_IDS()` マクロをその場で調査した結果である（`x(` 行および `BTREE_IS_write_buffer` を持つ行を計数、ソースコードは `~/code/fs-refs/` に固定）：bcachefs は**1組の btree 実装 + 21本のツリー**を使用し、うち **5本** が write buffer を通る（`lru` / `backpointers` / `deleted_inodes` / `rebalance_work` / `accounting`）；「アクセスパターンに応じて異なる構造を選択」に比べ**調整すべき構造が1組のみである**という利点がある——異種構造案は N 組の実装、N 組のクラッシュ整合性バグ、N 組の checker を意味する。prior-art.md の bcachefs の行はこの段落を正とする。
- ユーザー決定 2026-09-24：派生ツリーは異種構造を許容（原語は変更履歴）；増加する実装、クラッシュ整合性、checker はクラッシュポイント再生ストリームと checker が個別にカバーし、「調整すべき構造が1組のみ」を理由に阻止しない。
- 実験なし：三者プロトタイプ（`research/prompts/m2-keyspace-r1-opus-model/`）が同一の履歴とコストモデル上で計測、実験番号は未設定、どれを選択するかはユーザーが決定、判決 `research/prompts/m2-keyspace-r1-main-verification.md`。
- ユーザー決定 2026-09-24：K1、K2、K4（原語は変更履歴）。
- 実二一報告 `m2-keyspace-implementer-report.md`：実装担当の値選択をメイン agent が承認、0巡攻撃済み。
- 実二六報告 `m2-e26-implementer-report.md` 第6節 Q3、Q4：ディスク上スロット数とファイルオープン時の自己申告読み取り数の2つの値選択をメイン agent が承認、0巡攻撃済み。
- E142 (新規プールファイル作成のドライラン) 第15回実行第5段落（`e142-r15-step5-runner-report.md` 第4節）：割り当てレコードツリー内部およびルートノードの2セル、extent 上段リーフキー区間のセルにおいて、独立装置のバリアントアームのみがこの読み出し方で crates とバイト単位で一致した；これは帰因であり、装置の主読み出し方で検証されたわけではない。主読み出し方をこれら各セルに従って修正した再実行は後続で行われる。
- 判決 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第3節「上段の正規形」：上段の形状はエントリを持つ inode 番号の集合によってのみ決定され、書き込みと削除で同一の規則を共用する（2脚が個別に同一規則を導出；上昇のみで下降しない規則下では同一集合から2種の形状が生じてしまう）。

**未済**: C211 (ノードのディスク書き出し形態に関する本文がない)。

#### 確定項目 15：write buffer エントリの3つの厳格要件

**決定事項**：

1. **ソートは局所性を創造するのではなく破壊する ⇒ ノードキャッシュは耐スキャン性を備えなければならない**（2Q / LRU-K など）。リーフごとのソート後の各 flush はリーフ空間全体をカバーするシーケンシャルスキャンであり、**LRU は「反復的な、キャッシュを超えるシーケンシャルスキャン」に対するヒット率が正確に 0 となる**。**これは厳格要件であり、推奨ではない。**
2. **エントリ形態は必ず冪等完全値でなければならず、増分 Δ は許されない**。会計のツリーが最も厳格である——「4096バイトを解放した」の自然な表現は Δ であるが、処置は確定項目 1 の決定事項を参照。本プロジェクトはこれにより bcachefs と袂を分かつ：その accounting ツリーは delta 形態である（`disk_accounting_format.h` 原文「updates are _deltas_」）。
3. **エントリは自前のシーケンス番号（seq）を持たねばならず、これはフォーマット要件である**。flush 時のキーソートは**時間順序を破棄する**が、重複排除規則は「後勝ち」である ⇒ **「後」はエントリ自身によって保持されなければならない**；不安定ソートを用いた場合、同一キーのエントリの相対順序は未定義となり、重複排除結果は不確定となる。

**対象範囲**：3条は write buffer パス上のエントリとキャッシュポリシーを管轄し、幅と配置先は確定項目 7、値生成規則は確定項目 10、ノードメッセージバッファとの連携は確定項目 12 にある。⚠️ 要件 2 と要件 3 は異なる2つの要件である：冪等は「反復適用でエラーが出ないこと」を管轄し、シーケンス番号は「どちらがどちらを上書きするかが確定的であること」を管轄する。⚠️ **既知のコスト**（bcachefs 原文）：write buffer の更新は順不同かつ結果整合的であり、flush 時のキーソートにより時間順序が破棄され、この sort-merge-sweep は**本質的にシングルスレッド**であり、マルチスレッド負荷下でその効率がクリティカルパスとなる。この文には2つの解釈があり、実測結果は相反する——**スレッド解釈**（マージの CPU スループットがマルチスレッド負荷を支えきれない）は**不成立**：4 KiB のユーザー書き込みごとに3エントリ書き込む場合のシングルスレッドマージスループットはディスク帯域幅を遥かに上回り、リーフ分割により並列化も可能である；**I/O 解釈**（flush のツリー挿入 I/O がクリティカルパス）は**成立し、かつ3項目中最大である**。⇒ **クリティカルパス上で真にボトルネックとなるのは D9 (暗号化) の暗号化であり、マージではない**；⚠️ E17 (write buffer マージ上限) はマージの区間のみを計測しており、ツリー挿入と I/O は含まない。

**根拠**：

- E7 (オフラインインデックス harness)：本プロジェクト側の io/op はキャッシュがツリーの 2% / 6% / 25% を占める3段階でビット単位で同一であり、同リーフ集約を不変のままグループの処理順序をシャッフルするだけで 25% キャッシュ段階の読み取り回数を削減できることが介入実験で確認された。100% キャッシュ段階ではシャッフル前後で完全に同一であった（陰性対照）——LRU は「反復的な、キャッシュを超えるシーケンシャルスキャン」に対するヒット率が正確に 0 となるため、要件 1 は厳格要件である；同一装置により flush のツリー挿入 I/O が換算すると暗号化やマージより遥かに大きいことが計測され、I/O 解釈が成立した。
- E14 (I-3.1 の判別力：冪等完全値 vs 増分 Δ)：増分 Δ 形態下では I-3.1 (割り当て済み統計の合致) の判別力はゼロに帰し、冪等完全値形態下ではすべてを捕捉した——要件 2 はこれによって固定された。
- E17 (write buffer マージ上限)：第1版で `seq` を欠落させ、2アームが真値と即座に不一致となり、要件 3 の判定可能形態を迫られた；同一実験のシングルスレッドマージスループットとリーフ分割並列倍率により、「sort-merge-sweep は本質的にシングルスレッド」というスレッド解釈は不成立と判定された。

**未済**: C15 (write buffer への Δ 混入)。

#### 確定項目 16：第1版の unlink：deleted_inodes のみを立て、末尾からバッチ処理し、マウント内で空になるまで継続実行する

**決定事項**：

1. **意図**：意図のパブリッシュでは inode N のレコードを nlink = 0（size は不変）として書き込み、`deleted_inodes` に key = N（8 バイト、value なし、オブジェクト誕生世代なし）を追加し、墓石ツリーのオープンコンテナに墓石を1件追加する（D18 (ブロックに含める情報) 確定項目 24）；logged_ops は立てない（K-U1 L4）。
2. **バッチ分割**：任意の inode N に対し、末尾からデータユニット数単位でバッチ処理を行い、各バッチで N の末尾 K 個のユニットを解放し、size を「残存ユニット数 × 正味ペイロード容量」に変更する。残存ユニット数は N の extent ツリー内のユニット数を取得する（存在しない場合は 0、最後のバッチのみ）；製品コードの K は `EXTENT_TREE_LOWER_LEAF_DATA_UNITS`（144、自由選択）を採用し、テスト時はテスト専用ノブにより小さく絞る；削除バッチは N の切り詰められたその1枚の下段リーフとその祖先、および上段の N の一連のみを書き直す（K-R3-1、extent 書き込みの新規分岐であり、`.claude/rules/fs-design.md` の5つの厳格要件を通過する必要がある）；最後のバッチは N の残存全ユニットを解放し、N の inode レコードを削除し、`deleted_inodes` のその key を削除し、N のエントリを上段から除去し（ラベル 0 は書かない）、同一パブリッシュとする；この1つの経路のみとし、一度で完了する高速経路は設けない。意図のパブリッシュ、各バッチ、および継続実行の各バッチはそれぞれ1つのトランザクション番号を占有し、自インスタンスの次の番号に等しい（通常パブリッシュと同一の一連番号）。
3. **ツリー構築**：`deleted_inodes` は最初のファイルバージョン作成時のパブリッシュでゼロルート登録される；派生状態である：キュー = inode ツリー内の nlink = 0 の番号集合（`research/prompts/m3-delete-r2-main-verification.md` 第3節 V7）。
4. **読者と書き込みエントリ**：`InodeRecord` の nlink は `parse` で読み出し、`to_bytes` で構造体内の値を書き込む；レコードを新規作成するエントリは「nlink = 0」および「レコードが存在しない」を拒絶する；意図のコミット以降、読み取り専用マウント、コールドリード、モデル読み出しは「ファイルなし」と回答する；書き込みエントリは nlink = 0 の inode を拒絶する（`research/prompts/m3-delete-r2-main-verification.md` 第2節第1行、第3節 V6）。
5. **完全削除**：「ツリーが存在し、ファイル集合が空」は `RebuiltVersion` / `PreviousVersion` / `PoolVersion` の第3のメンバーであり、`WithoutFile` を借用しない；判定 = ツリーテーブルにエントリが存在 ∧ inode ツリーがゼロエントリルート（キー区間 `[0, 0]`、確定項目 6 参照）∧ extent ツリーがゼロエントリルート（キー区間は位置の規定に従う）∧ `deleted_inodes` ルートポインタがゼロ；墓石ツリー、系譜ツリー、退役ツリーは見ない。完全削除後は `publish_new_inodes`（size 0）、行の書き込み、および空パブリッシュのみが可能である（K-U5 X1）（`research/prompts/m3-delete-r3-main-verification.md` 第3節 G5）。完全削除された版上での上書き、シーケンシャルライト、unlink は既存の「変更可能なファイルバージョンが存在しない」という拒絶（`UserChangeRefused::NoFileVersionToChange`）として拒絶される。最初のファイルを削除完了し他ファイルが残存している版は `WithFile` に帰属する：inode ツリーにいずれかのレコードが存在するか extent ツリーにエントリが存在する版はファイルが存在する版であり、inode 1 の有無は問わない；第3のメンバーはファイル集合が真に空であることのみを指す。
6. **継続実行**：`mount_writable` は行の書き込みとウォームアップの後、制御を戻す前にキューを空になるまで進める；番号取得前のリハーサルには入らない；セッション内のキューは毎回現行版から読み出し、キャッシュしない；クラッシュ注入に `WritableMountPhase::Continuation` を追加（`research/prompts/m3-delete-r2-main-verification.md` 第3節 V3）。セッション内の各ユーザー変更（unlink、書き込み、inode 作成を含む）は、まず現行版の `deleted_inodes` キューを空になるまで進め、その後に自身の処理を行う；マウント時の継続実行は従来通り；unlink の1回の呼び出し = 空まで進行 + 意図 + 全バッチ（継続実行と同一の空まで進行を共用）。セッションの空まで進行時、削除系パブリッシュ（意図時およびバッチ分割の各バッチ）が配置先判定により空間不足で拒絶された場合、F の引き上げと回収を連続して試行する；拒絶された各削除系パブリッシュはそれぞれ1回の受入としてカウントされる（1回の受入につき最大 `PUBLISHES_PER_ADMISSION_AT_MOST` = 8 回のパブリッシュ、`crates/singlefs-core/src/mount.rs`）；上限まで試行しても配置できない場合、「F 引き上げ後も空間不足」の拒絶（`UserChangeRefused::NoSpaceAfterRaisingTheFloor`）を返し、すでにパブリッシュに成功したものはディスク上に残る。マウント継続実行の空まで進行時に削除系パブリッシュが配置先判定により空間不足で拒絶された場合も、同様に F 引き上げを試行し、予算計測条件はセッションの空まで進行と同一である；上限まで試行しても配置できない場合、マウントは通常通り書き込み可能として制御を戻し、キューは残される。その後のセッション内での各ユーザー変更はまず空まで進行を試み、進まない場合は空間不足を報告し、正常アンマウントは通常通り成功する。これは D16 (パブリッシュセマンティクス) 確定項目 1「正常アンマウント」行のその場で利用可能な回収スロットに依存している（D16 (パブリッシュセマンティクス) 確定項目 1「受入」行と同一定案）。いかなる版においても `deleted_inodes` は最大1件の key のみを持つ；複数のルートに収容可能な分岐は型付きエラーとして拒絶され、空間不足ではない。
7. **冪等の5つの判拠**：① 計画の一致；①b 同一のクラッシュイメージからそれぞれ再マウントして同一バッチを継続実行した場合、両ディスクがバイト単位で一致すること（意図時および各バッチの変更カウントは今回のパブリッシュの checkpoint_txg；意図時の3つの時間秒は呼び出し元から与えられ単一の値を共用、各バッチでそのまま複製し、継続実行のバッチは別個に時間を取得しない）；② 削除完了後に再継続実行しても一切書き込みを発行しない；③ 各書き込みポイントで切断し、再開して最後まで継続実行した場合、最終状態の論理的内容がクラッシュしない場合の最終状態と一致し、各クラッシュ状態の checker がすべて緑であること；④ 判定対象の各状態の会計ラベル 4、6 がそのバージョンのイメージからその場で計算された値に一致すること。「論理的内容」には inode 各条（番号、オブジェクト誕生世代、size、nlink）、extent ユニット番号集合と内容チェックサム、`deleted_inodes` key 集合、墓石レコード多重集合（死亡 txg を含む）、系譜レコード多重集合、ラベル 12 / 4 / 6、読者の回答が含まれる；インスタンス番号、ルートおよびユニットヘッダ内の txg、変更カウント、配置先、ユニットヘッダの誕生世代番号と書き込み順序、ディスクごとラベル 1 / 2 / 5 / 10 / 11 は含まれない。さらに harness 層における各パブリッシュの書き換えロール集合を絶対値で固定（0巡）（`research/prompts/m3-delete-r2-main-verification.md` 第3節 V1；`research/prompts/m3-delete-r3-main-verification.md` 第3節 G2）。異なる操作シーケンスの最終状態において論理的内容の一致は要求されず（例えば異なる削除順序では墓石の死亡 txg が異なる）、上段の形状がエントリを持つ inode 番号の集合によってのみ決定されること、各クラッシュ状態の checker がすべて緑であることが要求される。
8. **上段の正規形**：extent ツリー上段の形状はエントリを持つ inode 番号の集合によってのみ決定され、書き込みと削除で同一の規則を共用する：ノードは各エントリ保持 inode のルートパス（そのリーフからルートまでの一連）の和集合を取る；ルート階層は最大番号をカバー可能な最低層を取る；集合が空の場合は階層 0 の第 0 セルにゼロエントリのルート兼リーフを1個配置する；ルート以外のノードでゼロエントリは許されない。1回のパブリッシュにおいて、集合に出入りした、またはエントリが変更された番号の一連上で新形状に残るノードおよび新たに出現したノードを書き直し、他はそのまま複製し、前版に存在し新形状に存在しないものは外す；高くなる際は旧ルートをそのまま複製してルート配下のノードとし、低くなる際は前版で低下後階層の第 0 ノードをそのまま複製してルートとし、旧ルートおよび背骨線上方で外されたノードを交代させる（確定項目 14 の extent ツリーの条項と同一形状、`research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第3節「上段の正規形」）。

**対象範囲**：第1版は inode 番号による任意の inode の削除、ディレクトリなし、truncate なし（ユーザー 2026-09-28 の決定は単一ファイル、inode 番号による、ディレクトリなし、truncate なしであったが、任意番号は複数ファイル削除の判決により開放された）；最初のファイルを削除完了した後（K-U5、未済の検査に記録）、inode 番号によるデータ書き込みは inode 1 の墓石死亡 txg より遅いオブジェクト誕生世代を持つ inode のみを拒絶し、削除前から存在していた他ファイルは通常通り書き込み可能である；この死亡 txg の取得は現行バージョンの親チェーン上の墓石のみを認識する：「親チェーン」は I-9.18 (削除キュー内の番号に系譜上の墓石が存在する) の定義に従い、現行版から系譜レコードに沿って遡り、この親チェーン上のロールバックにより切り捨てられた一連の (T_old, T_rb) 開区間内の死亡 txg のみを除外する。親チェーン上にない系譜レコードは除外せず、その上で最大値を取る；inode 1 のレコードが存在する場合は判定しない；この読み方は第1版で墓石を回収しないこと（D18 (ブロックに含める情報) 確定項目 24）に依存しており、墓石回収の実装時に読み方を変更する必要がある（C618 (全削除後にデータ付きファイルを再作成できない)）；クラッシュストリームは `crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs` のストリームの後に接続してその 3 ユニットファイルを unlink し（K-U6 F1）、0 への truncate は別ストリームとして確立する。末尾リーフのみを変更する extent 分岐、書き込み系パブリッシュをパブリッシュ後のラベルで判定すること、書き換えロール集合の絶対値固定は0巡攻撃済み；第6条で補完されたセッション内の各ユーザー変更前の空まで進行、およびいかなる版でも最大1件の key も0巡攻撃済み。

**根拠**：

- 実験なし：数値はすべて三者検証脚のレプリカプロトタイプ上で計測（削除に関する3件の判決）、リポジトリ組み込み装置「削除が保持量を借用するリングウィンドウ」の第1段落はパッチ表適用済みレプリカ上で実行完了（2026-10-03）、第2段落は真の実装の削除バッチ上で再実行が必要であり、未実行。
- 三者判決 `research/prompts/m3-delete-r1-main-verification.md`、`research/prompts/m3-delete-r2-main-verification.md`、`research/prompts/m3-delete-r3-main-verification.md`。
- ユーザー決定 2026-10-01：K-U1、K-U5、K-U6、K-R3-1（原語は変更履歴）。
- 判決 `research/prompts/m3-blanks-r1-main-verification.md` 第2節キュー上限とインターフェース行：意図の版へのロールバックは番号をキューに戻すが、ユーザーアクションを開放した履歴において「セッション内の各ユーザー変更前に空まで進行」と「ロールバック後の継続実行」の2アームのみがキュー長とキュー存在時の書き込み回数の2項目を守り抜いた。
- ユーザー決定 2026-10-01：セッション内の各ユーザー変更前に空まで進行（分岐票 `m3-blanks-r1-forks.md` の「セッション内の継続実行ポイント」行、原語は変更履歴）。
- ユーザー決定 2026-10-02：セッション空まで進行が空間拒絶された場合 F 引き上げを試行；2026-10-03：拒絶された各削除系パブリッシュは各1回の受入としてカウント、マウント継続実行が空間拒絶された場合も同様に F 引き上げを試行、上限到達でも配置不能な場合は D16 (パブリッシュセマンティクス) 確定項目 1 に従い書き込み可能として成立、削除系パブリッシュは各1トランザクション番号を占有（原語は変更履歴）。
- コード巡判決 `research/prompts/m3-unlink-code-r1-main-verification.md` 第2節 K2 行（継続実行、空まで進行および F 引き上げ）および第4節（マウント継続実行の上限到達未配置の結末をユーザーへ提示）；実装：セッション空まで進行の F 引き上げは `crates/singlefs-core/src/mounted_session.rs` の `publish_the_queue_until_it_is_empty_raising_the_floor_for_space`、トランザクション番号は `crates/singlefs-core/src/transaction.rs` の `publish_deletion_step`。
- メイン agent 2026-10-02 Q-L 受入決定（スケジューリング記録 `records/2026-10-01-删除与并发事务.md`「十六 バッチ 8」）：完全削除された版上での上書き、シーケンシャルライト、unlink は既存の拒絶として拒絶。
- ユーザー決定 2026-10-03：「アンマウントは必ず成功させねばならない」、継続実行の上限到達未配置後の正常アンマウントは D16 (パブリッシュセマンティクス) 確定項目 1「正常アンマウント」行が開放した回収スロットにより成立させる、単一ディスク残存の質問は「甲 GA（推奨）」を採用（原語は変更履歴）；設計巡判決 `research/prompts/m3-unmount-after-short-continuation-r1-main-verification.md`、コード巡第3巡判決 `research/prompts/m3-unlink-code-r3-main-verification.md` 第2節 n1 各行。
- ユーザー決定 2026-10-03：削除ステップを変更と見なす（「変更と見なし実装修正」）、時間フィールドは「乙 意図が時間を保持、3セル同一変更」を採用（原語は変更履歴）；コード巡第3巡判決 `research/prompts/m3-unlink-code-r3-main-verification.md` 第2節 n3 行。
- 判決 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第3節「任意番号の削除」「上段の正規形」「最初のファイル削除」および第5節ユーザー決定：任意 inode 削除の各ステップ、上段正規形、K-U5 は「β 時系列順」を採用（ユーザー決定原語は変更履歴）。
- コード巡判決 `research/prompts/m3-close-w5-code-r1-main-verification.md` 第2節 W4 および第3節第1条：β の墓石死亡 txg 取得は現行版の親チェーン上の墓石のみを認識（攻撃側が計測済み：inode 1 削除、inode 3 作成、削除前へロールバック、再度 inode 1 削除、inode 3 作成の版へロールバック、inode 3 の書き込み成功、プールレベル checker 赤ゼロ）。

**未済**: C618 (全削除後にデータ付きファイルを再作成できない)、C619 (墓石ツリールート伸長時のパブリッシュにクラッシュストリームがない)、C614 (ロールバック時の墓石ツリー誤取得およびオブジェクト全削除後の墓石なしが単一イメージで判定できない)；C118 (`deleted_inodes` ツリーの形態に配置先がない) は解決済み。

```

**出处 `.claude/kb/decisions/08-コアインデックス構造.ja.md:513-516`（整段抄，未转述）**

```markdown
## 履歴バージョン

D8 (コアインデックス構造) の履歴項目は [decisions-history.md](../decisions-history.md) に集約されている。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:1-6`（整段抄，未转述）**

```markdown
## D8 核心索引结构 —— 已定

**中文** · [English](08-core-index-structure.en.md) · [日本語](08-コアインデックス構造.ja.md)

D8（核心索引结构） 管核心索引长什么样：一套 btree 实现配多个独立 keyspace 与两个前端、节点大小与它的上限判据、key 布局、whiteout 删除与无界操作的分批、inode 树的形态与 inode 记录的盘上字段表、write buffer 条目的 `seq`、树表条目的字段表与树的种类码登记表、码 2 索引节点的载荷布局与 key 比较序、前端与节点消息缓冲区的衔接、同一次发布里同一个 key 的状态折叠。**不预设 B-tree**，调查见 [prior-art.md](../prior-art.md) 第五节。不管的：记账统计量的载体、key 与合并规则在 D5（快照 / 空间记账机制）；多可写头下 key 要不要加一维快照、快照的实现模型在 D6（快照实现模型）；消息缓冲区的 ε 布局轴在 D11（索引节点要不要留消息缓冲区）；单元头、单元类型码与打包记录单元的装载纪律在 D18（块里携带什么信息）；指针的字段与宽度在 D19（块指针的结构与宽度预算）；发布语义与 checkpoint 边界在 D16（发布语义）；格式冻结的层次在 D15（格式冻结政策）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:30-42`（整段抄，未转述）**

```markdown
#### 已定项 1：记账取幂等完整值，整批换代

**定案**：**记账条目一律是「某个统计量在某一代的完整值」**，写入时整格覆盖；**不用增量 Δ**。合并规则按统计量分（D5（快照 / 空间记账机制） 已定项 3 第 3 行、登记表见 D5（快照 / 空间记账机制） 已定项 10）：非单调统计量「后者胜」（后者由 `seq` 定），单调统计量取最大值。取最大值同样幂等、可交换、可结合，本项的论证（幂等完整值胜过增量 Δ）不受影响。整批换代：每条根的记账树只装这条根那一代的行，不留多代（除 inode 号水位的台阶行随根保留外，D5（快照 / 空间记账机制） 已定项 2）；key 数量不随代数单调累积，两块盘的池上除台阶行外常态 15 行。

**射程**：它定的是记账条目 value 的语义；载体（记账住一棵独立 keyspace 的 btree、条目是「(统计量, 维度元组, 代) → 完整值」）在 D5（快照 / 空间记账机制）。⚠️ **key 数量不随代数单调累积**：每条根只留自己那一代的行（D5（快照 / 空间记账机制） 已定项 2，C548（记账树每次发布整批换代与每统计量每代一条不符）），两块盘的池上除台阶行外常态 15 行，80 块盘起才分裂。⚠️ **兜底**：若某一类账确实做不到幂等（本项不预判有没有这一类），那一类必须另有一条与运行时**不共享判定代码**的独立重算路径。⚠️ 它同时改变 D11（索引节点要不要留消息缓冲区） 代价 1 的成立条件——该代价的判据是「缓冲里放的是幂等完整值还是增量 Δ」，本项把答案钉在幂等那一侧 ⇒ **代价 1 在本工程不成立**；D11（索引节点要不要留消息缓冲区） 的代价 2（可重建性退化）不受影响。

**依据**：

- E14（I-3.1 的判别力：幂等完整值 vs 增量 Δ）：同一棵树同一负载注入四类语义 bug，幂等完整值形态下 I-3.1（已分配统计对得上） 全抓、零盲区，增量 Δ 形态下一条都没抓到——Δ 形态下账本身就是权威内容，checker 只能拿同一个累加函数重放同一份 Δ 日志，它在和自己比。
- 用户定案 2026-08-29：取幂等完整值，接受 key 数量上升（原话在变更史）。

**欠**：C15（write buffer 混入 Δ）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:43-58`（整段抄，未转述）**

```markdown
#### 已定项 2：节点大小 16 KiB，钉成常量

**定案**：**节点大小 16 KiB。mkfs 把 16 KiB 写进系统配置，实现按常量走，不按设备特性算**——格式里是一个常量，不是一条公式。
<!-- format-const: NODE_BYTES = 16384 -->

**射程**：⚠️ **登记 `format-const` 不等于冻结这个数**（2026-08-31 用户定案：「这个数需要可以调整」）。那条标记只保证 **kb 与实验源码写的是同一个数**，改它照常走 D8（核心索引结构） 的流程；它写进系统配置，仍然是 mkfs 参数。⚠️ **它现在只钉得住一部分源码**：门禁阶段「格式常量在 kb 与实验源码之间同步」（`.claude/gate.d/code-source-discipline.sh` 的 format-constants 格）的形态是 `const 名 = 整数字面量`，`e56_epsilon.rs` 的 `16 * 1024`、`e43_extension_point_budget.rs` 与 `e52_head_mechanisms.rs` 的数组形式都不认。⚠️ **16 KiB 是 CPU 缓存轴上的选择，不是设备 I/O 轴上的**：E7（离线索引 harness） 量设备 I/O 时说小节点两端都赢、方向相反，两者量的是不同资源，不矛盾。上限那一维归已定项 4；E1（节点大小 vs 可恢复性） 的恢复时间那一半一旦给出比 16 KiB 更紧的上限，本项重开。

**依据**：

- E20（指针变宽的 CPU 缓存代价）：唯一超出 L3 的工作集上 16 KiB 是最小点；同一实验里 40 字节的指针头部在 4 KiB 节点付出的相对代价远高于 16 KiB 节点，大节点吸收指针变宽；同一条曲线上 8 KiB 的反常六轮稳定且无人解释 ⇒ 在解释掉它之前不许把节点大小写成「按设备特性算」。
- E6（加密算法选型）：加密 / 校验单元的增益在 16 KiB 之后基本饱和。
- E73（节点 key 区间的扇出代价）：码 2 头带上 key 区间之后树高一格都没涨，16 KiB 不用重开。
- 用户定案 2026-08-30：16 KiB 钉成常量（原话在变更史）。

**欠**：C56（没登记的格式常量仍会漂）、C386（节点大小结论欠一次按最终指针宽度的复跑，没有编号）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:59-89`（整段抄，未转述）**

```markdown
#### 已定项 3：key 布局 —— 改名与索引分开处理

**定案**：**key = `(locality_id, inode, offset)`**，`locality_id` 在创建时从父目录继承，**改名时故意不更新**。offset 段是**文件字节偏移**（这个单元第一个字节在文件里的偏移，第 n 个单元从 0 起是 n × 净荷容量 32634）。

- 命名走 dirent 树（`parent, name` → inode），改名就是删一条加一条，**O(1)**
- 数据按 inode 编 key，与名字彻底解耦
- scan 局部性来自继承的 `locality_id`，改名后**局部性缓慢退化，正确性不受影响**
- 退化由 D3（空间分配） 已承诺的常驻后台整理修复，不是新增负担

**批量改名**：`mv 大目录 别处` 在这个设计下**只改一条 dirent**。三处会把它重新变成无界操作，必须一开始就堵死：

1. **`locality_id` 永远不许被改名更新。** 它是**提示，不是正确性的一部分**——可以是错的、可以过期。
2. **改名覆盖已存在的目标 = 一次删除**，必须走已定项 5（whiteout 删除）的 intent log。
3. **后台重新聚簇必须是增量、可中断、可续做的**，且随时可以停掉不做。

**推论（可当 review 判据用）**：**任何让 rename 触及后代的设计改动，一律否决。**

**inode 记录必须存一份 `locality_id`**：它既是数据 key 的首段、又永不随改名更新，而命名树的 value 只有 inode、数据单元头的五元组里也没有它，所以盘上只有 key 自己带着它——读者打开一个改过名的文件时必须有地方拿到它，否则改名之后数据只能靠扫描才找得到。落点是已定项 6（inode 记录的盘上字段表）；这个值按本项从父目录继承，而第一版没有目录，所以「没有父目录时取什么」也要一起定——第一版取 0，登记在已定项 6 的记录字段表。扫描重建时重编出来的 key 首段按 inode 记录 > 幸存 extent key > 0 给这个对象的全部 key 编同一个首段（2026-09-16 用户定案，D18（块里携带什么信息） 已定项 3 那条加字段的提议不收）。

**射程**：它管数据 key 怎么编与改名怎么不触及后代，不管命名树自己的 key（dirent 树按 `(parent_inode, name)` 编，见已定项 5）。scan 局部性不一定要住在 key 序里：可以住 key 序（全路径索引，改名是 O(子树深度)）、住分配器（把子目录的数据放到父目录附近）、住第二索引（仍然贵，且要多维护一份一致性），本工程取「出生时继承、改名不更新的 locality 前缀」（较好，改名免费）。⚠️ 「它是提示、可以是错的」准许的是**不信它的含义**，不是**不存它的位**：作为地址首段它的取值精确且定义性。⚠️ **E9（key 编码对遍历局部性的影响） 的遍历局部性数只在一个口径下成立，引用必须连口径一起引**：**交错创建**（`interleave`，老化的代理）**加上 `cache_leaves = 8`**；`locality_id` 的收益是**新树的收益**，不是稳态收益，改名强度的现实性未校准——曲线可用，绝对刻度不可用。⚠️ 全路径索引的遍历优势也量出来了，但**不许拿它去替 D9（加密） 泄漏面与 D1（数据可移动性 / 反向索引） 稳定性做判断**——E9（key 编码对遍历局部性的影响） 对那两样一个字没说。⚠️ 「每次改名平均重写约 2700 个 key」这个数**缺口径**（查不到是哪一档改名、哪个种子），标为未核实；同表里 ext4 Orlov 分配器「遍历内核树快 30%」也是未核实——kb 内无出处无口径，要用先补。⚠️ 三段 key 在 D6（快照实现模型） 取多可写头之后曾经不完整（两个可写头各写一次同一个 `(locality_id, inode, offset)` 会落在同一个 key 槽、后写覆盖先写），2026-08-30 D6（快照实现模型） 已定项 1 取「每头一棵自己的树」之后消解——每头一棵树之下三段 key 在树内唯一，**两个头不再落进同一个 key 槽**。⚠️ **nonce 那条义务仍在**：D9（加密） 已定项 4 已定 nonce 每 extent 存，**生成时仍必须保证两个可写头之间不撞**（候选形态是按头划分 nonce 空间），它不再由身份唯一性免费得到，否则 I-6.1（nonce 不重用） 从「由身份唯一性保证」变成「由生成器保证」而没人保证它；该义务是推论，未走三方论证。⚠️ 后台重新聚簇是只搬物理位置还是重编 key、重编时怎么分批、进度从哪续做，全仓没有定义。

**依据**：

- E9（key 编码对遍历局部性的影响）：同一棵目录树同一个 LRU 上四条臂对比，`locality_id` 前缀比按 inode 编 key 少发设备读，值得做；depth-first 创建时三条真实臂逐位相同，缓存装得下整棵树时四条臂全部相等、效应消失；收益随改名次数衰减到几乎为零。
- 机制论证（2026-08-25 定案时写下，原文在变更史）：全路径索引在本工程还要多付三条 BetrFS 不必付的代价——分隔 key 就是路径会把 D9（加密） 已定项 5 认定的敏感信息从一棵树扩到所有按路径编 key 的树、引用方身份是路径会让 D1（数据可移动性 / 反向索引） 的反向索引条目随改名全部失效、lifting 对 key 编码的额外要求与 D9（加密） 的 nonce 载体和 D6（快照实现模型） 的快照 ID 位置抢同一段 key 空间。
- 用户定案 2026-09-16：扫描重建时给一个对象的全部 extent key 编同一个首段（原话在变更史）。
- E157（并行线一两条条款的计数模型） 量出：定位一页的节点读取次数、树高与叶片数、换容量后字节区间还原那一格，两臂在全部取样点上逐点相等、分不开；唯一分得开的是 AAD 锚点那一格，文件字节偏移下 N 个 key 全算对、单元序号下只有 1 个算对。
- D9（加密） 已定项 6 逐字「锚点偏移 = 该 extent 首次写出时的对象内起始偏移，读者现算 `key.offset − ptr.extent_off`」，而同一条决策已定项 12 第 2 项写明 AAD 字段组成是 day-1 永久契约 ⇒ 条款本来就预设了这个单位。

**欠**：C346（后台重新聚簇没有定义）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:90-107`（整段抄，未转述）**

```markdown
#### 已定项 4：节点大小 —— 上限由恢复时间定，靠实验取值

**定案**：大节点省 seek、树更浅，但**重扫代价随节点大小线性放大**，所以**不照抄 128K–256K**。**节点大小换的是「恢复时间」，不是「数据安全」，目标线该定在恢复时间上。取值靠 E1（节点大小 vs 可恢复性） 定：在「重扫代价 ≤ 目标线」的前提下取查找代价最低的那档。** 三条约束：

1. **它是 mkfs 时的格式参数，不是挂载参数。** 自动选取只发生在 mkfs 一次。改它 = **重建索引**，不是重建文件系统（D21（权威态与派生态的分界） 已定索引是派生态、冻结第 2 层降级为本地缓存格式）。
2. **默认策略必须是确定性的、可查的**，且 mkfs 要把「为什么选了这个值」打出来。
3. **测试钉死的是「模拟设备的参数」，不是节点大小本身**——固定一组设备档位让自动规则照常跑，**连选取规则本身也一起被测**。要求只有一条：同一档位必须得到同一取值。

**射程**：⚠️ **判据不是「全盘扫描重建成功率 ≥ 目标线」**——索引坏的永久损失恒 0 ⇒ 成功率这一维与节点大小**无关**，拿它当判据夹不出任何上限。⚠️ **目标线本身仍未定**，它是一个恢复时间预算（多久之内要把被破坏的那部分重扫回来），本项自己算不出来；E1（节点大小 vs 可恢复性） 剩下那一半仍欠 mkfs、checker 与一个能定点破坏镜像的工具。⚠️ 前置：索引可从单元自描述重建——E28（明文映射层能不能只靠单元自描述重建） 实测**元数据侧成立、数据块侧不成立**（缺口恰等于数据块总数，欠 D18（块里携带什么信息） 已定项 3）；⚠️ **「改它 = 重建索引」取消的是定性，不是代价**：整棵树按新尺寸重建是一次满量级的全盘 I/O，只是不丢数据；而且这个取消有未满足的前置——数据块自描述补上之前，「等于重建文件系统」在数据块那一侧仍然成立。桥接依据是索引树重建与明文映射层重建**读同一份输入**（每个单元自带「我是谁、我属于谁」），⚠️ 这一步是推理，E28（明文映射层能不能只靠单元自描述重建） 只测了明文映射层那一个实例；D11（索引节点要不要留消息缓冲区） 「不能拖」的全部理由就是这半句。⚠️ **第一期只有字节主导那一端在射程内**：D12（目标介质） 已定项 5 定第一条布局线是纯 SSD ⇒ 结论不变，**支撑从两端变一端**；寻道主导那一端要等 HDD 布局线开出来才重新在射程里。⚠️ **节点大小的自动选取判据里必须含指针宽度这一项**（E20（指针变宽的 CPU 缓存代价）：大节点吸收指针变宽，方向与重扫代价相反），E20（指针变宽的 CPU 缓存代价） 之前没有。⚠️ **「按设备特性取相反方向」这个理由不成立**：两端最优方向相同，都偏小；节点大小仍是 mkfs 参数，要保留按介质调档需另找依据。

**依据**：

- E29（坏一个节点的爆炸半径）：索引坏的永久损失恒为 0（D21（权威态与派生态的分界） 已定索引是派生态、丢了只是慢），数据单元坏的损失与节点大小无关，只有重扫代价随节点大小线性放大（第 1 层节点的重扫量恰等于扇出）——所以上限只能定在恢复时间上，「爆炸半径」这个说法被它纠正。
- E7（离线索引 harness）：**固定内存预算下大节点三个轴全都更差**（量的是 io/op、字节/次、点查读/次），机制是节点越大、同样的内存装得下的节点越少，命中率掉得比「每次读更多」赚回来的更快；寻道主导那一端方向相同，大节点仍差 ⇒ 「不照抄 128K–256K」在字节主导与寻道主导两端都得到支持。
- 用户定案 2026-09-06：口径改写成「上限由恢复时间定」（原话在变更史）。

**欠**：C386（节点大小结论欠一次按最终指针宽度的复跑，没有编号）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:108-153`（整段抄，未转述）**

```markdown
#### 已定项 5：whiteout 删除 —— 接受，但批量删除必须走 intent log

**定案**：删大目录、截断大文件会产生海量墓碑，塞进一个事务必然爆掉。**做法（bcachefs 已验证的形态）**：凡是**可能修改无界数量 extent** 的操作（truncate、collapse、删大目录），**先把意图写进 logged_ops 树，再分批推进**；崩溃后恢复时续做未完成的意图。删除中的 inode 单独进 `deleted_inodes` 树，由后台回收——用户的 `rm` 返回，不等于所有 extent 都已释放。第一版 unlink 不往 logged_ops 树写条目：意图就是 inode 记录 nlink = 0 加 `deleted_inodes` 那一条（已定项 16）。logged_ops 树在第一个文件版本那次与 `deleted_inodes` 一起零根注册，字段表已定（D26（后台整理与放置回收） 已定项 2 的整理意图条目字段表），回退时取哪一版、态别留到第一个写者（整理）落地时定（C364（整理意图记录的字段没有逐字段对账））。意图分两类：**承诺类**（unlink；truncate、删大目录）必须续做到底，可写挂载交回之前走到空；**提示类**（整理）可暂停、可在停机谓词下提前收、交给盘那条线上不续做，不挡挂载；续做入口按树分、共用发布路径。已定项 5 第 2 条与三态表、C1（意图续做做成常规路径） 的字面按承诺类写，整理落地时按类分开写（C621（整理落地时要动的删除条款没有清单外的检查））。

**硬要求**：无界规模的操作一律不许做成单个事务；意图必须**幂等**；whiteout 只在节点重写合并 bset 时清除，**不许有别的清除路径**。

意图本身就是一条普通 key，骑在同一个事务层上，断电后只有三种状态：

| 断电时机 | 恢复后看到 | 怎么收场 |
|---|---|---|
| 意图提交之前 | 没有意图 | 操作从未发生，用户那边也没返回成功 |
| 意图已提交、工作做了一半 | 意图还在 | 恢复时**续做**，已做完的部分被幂等地重跑 |
| 工作做完、意图未删 | 意图还在 | 续做一遍发现无事可做，然后删掉意图 |

不可能出现「意图没了但目标状态没达成」——**删除意图必须与最后一批工作在同一个事务里提交**。

1. **意图声明目标状态，绝不声明增量。**「把文件 X 截到 N 字节」可以重复执行，「再删 1000 个 extent」不能。
2. **续做是每次恢复都跑的常规动作，不是修复手段**（否则会出现「平时不跑、真出事时才第一次跑」的代码路径）。
3. **中间状态必须每一个都是合法的文件系统状态**——「删了一半 extent 的文件」本身就得合法。
4. **一批里每个动作施加时先核它要删的东西还在**（随 D5（快照 / 空间记账机制） 已定项 12）——幂等的字面：一批按批开始时的状态为几个意图各算一份计划、施加时不复核，两个区间重叠的级联会把同一块放两次，统计量减两次、I-3.1（已分配统计对得上） 红。

**范围重建 vs 逐项修改：两条都要有，且要有显式判据**（`.claude/rules/fs-design.md`「不追求万能方案」）：

| 走法 | 代价 | 适用 |
|---|---|---|
| 逐项修改 | K × 随机访问 | 受影响的 key 稀疏散布 |
| 范围重建 | N × 顺序访问 | 受影响的 key 在 key 序上连续 |

**真正的开关不是 K/N，是「受影响的 key 连不连续」**：连续 ⇒ 范围重建可整节点丢弃或重写；不连续 ⇒ 只能逐项。**由此得到一条布局规则：让一起死的东西在 key 序上挨在一起。**

- dirent 按 `(parent_inode, name)` 编 key → 删整个目录是范围操作
- extent 按 `(inode, offset)` 编 key → 截断是范围操作
- 同一快照的 key 带同一 snapshot 前缀 → 删快照有机会变成范围操作

**这顺带从结构上解掉了 whiteout 膨胀**：范围删除直接丢弃整个节点，不用插 N 个墓碑。

**射程**：朴素的交叉点是 `K/N > 顺序代价/随机代价`（机械盘约 1%、SSD 约 10%）。⚠️ 这两个数是**推理，不是实测**，说的是**耗时**，要靠 E4（范围重建 vs 逐项修改的交叉点） 量，那一半要 QEMU。⚠️ **第一期只取 SSD 那一档**（D12（目标介质） 已定项 5：第一条布局线是纯 SSD）。⚠️ 写放大那一半量到的交叉点，自变量是「布局」与「节点大小」而不是介质，不能拿它去证伪「机械盘约 1%、SSD 约 10%」——那说的是耗时。按 `.claude/rules/fs-design.md` 的五条要求，这个分支还欠：判据写成可计算的式子、留一个能强制走指定分支的测试开关、两条分支各自跑崩溃点重放、运行时能看出走了哪条。⚠️ **切换判据里不许出现「当前填充率」这类操作中途会变的量**——判据只能用操作期间稳定的输入。**与 D3（空间分配） 的交互（容易漏）**：意图未完成期间那些空间还没真的释放，空间统计必须把「待删除但未完成」计进去，否则会出现「我删了一大堆，`df` 却不动」。

**依据**：

- E30（交叉点的写放大那一半）：**写放大那一半已量**——在写放大口径下量出条目连续与条目散布两种布局在 4 KiB / 16 KiB 节点上的交叉点，两者相差两个数量级以上——开关是布局与节点大小，不是介质。
- 用户定案 2026-09-16：加第 4 条硬要求「一批施加时先核要删的东西还在」（原话在变更史）。
- 三方 `research/prompts/m3-intent-r1-main-verification.md`：意图分承诺类与提示类（两腿独立得出，一轮）；logged_ops 树零根注册是用户定案 K-I1（原话在变更史；被攻过零轮）。

**欠**：C2（无界操作做成单事务）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:154-234`（整段抄，未转述）**

```markdown
#### 已定项 6：inode 树的叶是码 3 打包记录单元，记录定长 140，已发布的 (inode 号, 对象出生代) 不复用
<!-- doc-lint:not-numbers P1 P2 P3 P4 P5 P6 P7 P8 P9 -->

**定案**：inode 树 = 码 2 内部节点（派生态，不留消息缓冲区，条目 120 字节、扇出 135）+ 码 3 打包记录单元做叶（打包记录类型 2，权威态单元，一叶 233 条 140 字节记录）；根恒为码 2；容器索引就是这棵树的内部节点；已发布的 (inode 号, 对象出生代) 不复用，下一个号是记账里一条取最大值合并的统计量。码 2 根装满时长一层——新根 1 个码 2、分出的右半 1 个码 2（右半没有容器身份），inode 树与墓碑树（D18（块里携带什么信息） 已定项 24）共用这一条；长高那次发布多 2 个 16 KiB 节点 × 副本数。根长高之前代码在 `crates/singlefs-core/src/inode_tree.rs` 的 `MoreLeafContainersThanOneRootNodeHolds` 拒（推的单元数，零轮）。

| 部件 | 是什么 | 态别 |
|---|---|---|
| 叶 | 一个 32 KiB 码 3 打包记录单元（D18（块里携带什么信息） 已定项 11），打包记录类型 2，装 233 条 140 字节记录；自带类身份段、记录数、记录宽、载荷校验和，扫描路径上的载荷完整性证据（C104（扫描路径上码 1 / 2 的载荷完整性无自包含证据） 对码 2 的洞）对它不存在 | 权威态（单元） |
| 内部节点 | 码 2 索引节点。头带 key 区间（min / max 各 8 字节，D18（块里携带什么信息） 已定项 2 留给 inode 树的那一格由此定「带」：理由与记账树同一条——一个节点单独捡起来能自证覆盖段，key 是 inode 号不是文件名、敏感性论证够不到）。**头是块头，落 D15（格式冻结政策） 第 1 层（冻结），与记账树节点头同一格（C102（索引节点头没有预留区，加字段不是 compat））；载荷布局（条目 120、ε = 0）落第 4 层，不永久** | 派生态，丢了从叶重建 |
| 根 | 恒为码 2 索引节点，哪怕树只有一片叶，哪怕树是空的（零条目，key 区间写 [0, 0]，0 是无效 inode 号）。理由：D19（块指针的结构与宽度预算） 的指针不带被指对象的类与大小，树表指针指向的根若类会随树长大而变，读者按 16 KiB 读 32 KiB 是静默误读 | 派生态 |
| 容器索引 | 就是这棵树的内部节点，不另立树：物理指针只住内部节点，dirent 树引用的是 inode 号。更新一条记录 = COW 叶 + 全部祖先（D23（journal 的角色与格式） 已定项 1 甲） | — |

**内部节点条目 = 分隔 key 8 + 身份引用 26 + 子指针 86 = 120 字节**（扇出按头 131 算 ⌊(16384 − 131) / 120⌋ = 135）。身份引用就是 D18（块里携带什么信息） 已定项 11 给「别的树引用容器」的那 26 字节：出生树 8 + 打包记录类型 2 + 容器号 8 + 容器出生代 8。**类型段兼作「子节点是什么」**：0 ⇒ 子节点是码 2 索引节点（容器号、出生代两段恒 0；登记表里的 0 在容器头里仍是无效，两处含义靠所在结构区分），2 ⇒ 子节点是码 3 容器（读两个 16 KiB 槽）；其余值判条目损坏。合法集合按树分：inode 树 {0, 2}（I-9.2（条目身份与子头相符））、墓碑树 {0, 1}（I-9.23（墓碑树条目身份与子头相符））、谱系树 {0, 5}（I-9.17（谱系树条目身份与子头相符））、退役树 {0, 3}（不变量随第 50 批与退役树的实现一起立，判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 8 条），四棵树的内部条目同一形态。读者由此知道读几个槽；码 3 那条边的 AAD 五段（标签由类型 ≠ 0 推出，其余四段照抄条目）全部来自查找路径，D9（加密） 的 A6（AAD 期望值不来自明文侧）在这条边上不需要树的种类；码 2 那条边（内部节点指内部节点）的 AAD 仍是 C107（索引节点的 AAD 身份无落点）。出生树逐条目携带，是因为克隆头（D6（快照实现模型） 已定项 1）共享 origin 的叶，同一个内部节点里会混着出生树不同的条目；D19（块指针的结构与宽度预算） 已定项 7 让子指针头部也带出生树之后，身份引用里的出生树**照旧保留**——两份并存，加密开不开都占位、不挪作他用；类型段 0 条目身份引用里的出生树恒 0，与容器号、出生代两段同形；码 3 那条边 AAD 的出生树取身份引用那份，不取子指针那份。**分隔 key** = 该孩子创建时的最小 key，左从右借的那次发布改成被借的右孩子这一版的最小 key；条目按分隔 key 单调递增，查找取最后一个分隔 key ≤ 目标 key 的条目。
<!-- format-const: INODE_INTERNAL_ENTRY = 120 -->

**内部节点不留消息缓冲区，inode 树的 ε = 0**——这是对 D11（索引节点要不要留消息缓冲区） 已定项 6 的树轴增补（布局轴「全部布局共用同一组 ε」不动）。分支变量是 D11（索引节点要不要留消息缓冲区） 已定项 4 那条谓词：**这棵树待下刷的消息能不能只从权威态（单元（带写序）+ 实例表 + 祖先表 + 墓碑单元）重算**。能 ⇒ 留缓冲（ε = 0.65）；不能 ⇒ 不留。inode 树不能：属性更新（mtime / nlink / mode 的新值）在到达叶之前没有权威态载体，缓冲进派生态节点就撞 D21（权威态与派生态的分界） 硬约束 6。extent / dirent / 反向索引能（E59（缓冲消息能不能只从单元重算） 的「单元 + 记账」一行三类全 0）；记账树按 D21（权威态与派生态的分界）「离线可由遍历重算」判能，但 E59（缓冲消息能不能只从单元重算） 那一列是神谕、未实测。它落 D15（格式冻结政策） 第 4 层（派生态），不是格式分支。**态别按重算性判，不按「树」这个词判**：一棵树的内部节点可从它的叶重算 ⇒ 派生态；叶若装着算不回来的值 ⇒ 权威态。记账是同一条判据的已定实例，权威记录点在 D21（权威态与派生态的分界）。

**装载纪律按记录类型分**：「一容器一个代际」与「快照发布前先关闭开放容器」只对按代际回收的打包记录类型 1（墓碑）成立；类型 2 的记录逐条死亡，容器由 B 树分裂 / 合并回收，一片叶活很多代、反复重写，快照只是共享它。**死亡分三层**：① 类型 2 的记录死亡不写墓碑，死亡由现行叶版本体现，择现行版本压在 C113（扫描重建时多版单元的现行版本判定无输入）；② 叶容器的身份消失（删空摘掉、合并时被吸收的右半、跨头首次 COW 重生时的旧身份）写退役记录：打包记录类型 3，带死亡 txg，住专用的退役树（D18（块里携带什么信息） 已定项 11）；退役记录的写者只追加：同一身份跨管理员回退可以有多条，不去重、不覆盖；死亡 txg = 身份消失那次发布的 checkpoint_txg；读者与 checker 只认死亡 txg 在这一版谱系上的退役记录；③ 对象整个删除由 unlink 意图那次写的整对象墓碑体现，从意图那次发布起生效（D18（块里携带什么信息） 已定项 24 第 2 条、已定项 16 第 1 条）。文件变短（覆盖写成更少的单元、截短）不写新记录，数据单元的死活在扫描重建时按重建出的 inode 记录判（「重建」一段）。

**容器身份不需要分配器**：inode 号在每一条时间线上单调（管理员回退的新根取 R_old 那一版的 inode 树、水位取 max(现行那一版内存里的, 环里读得出、按实例表没被崩溃恢复抛弃的根的)；崩溃恢复回到所选根的树与它的水位；两种之后发的号都大于那棵树里每一个 key）⇒ 插入永远落在最右的叶 ⇒ 分裂只发生在最右叶，**分裂点取末尾**——左半保留全部原有记录与身份，右半从触发分裂的那条新记录起，**右半容器号 = 那条新记录的 inode 号**，出生代 = 本次发布的 checkpoint_txg，**出生树 = 执行这次分裂的那棵树**（分裂是新建，与「COW 重写身份不变」是两件事）；第一个容器的号 = 第一条记录的 inode 号。内部节点同样在右端溢出、末尾分裂，右半是码 2、没有容器身份。合并：**左吸收右与左从右借两样都做**（右吸收左或右从左借会破「记录号 ≥ 容器号」），保留左半身份；被吸收的右半走 COW 释放路径，同一次发布给它的身份写一条退役记录；左从右借时借的条数取均分（借完两片条数相差不超过 1），右片保留身份、不写退役记录，它的分隔 key 改成它这一版的最小 key；合并时机是写路径策略（单测 / 模型对拍），不进格式。**记录数为 0 的类型 2 容器不落盘**（发布前必须合并掉；唯一的叶被删空时是不落盘、根变成零条目），删空摘掉的容器同一次发布给它的身份写一条退役记录。**跨头首次 COW 的重生**（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P6）：新身份 = (本头树 ID, 类型 2, 原容器号, 本次发布代号)——保留容器号、只换出生树与出生代，同时给旧身份写退役记录。重生只对类型 2：类型 1 容器归写者树独占、克隆不收进自己的容器索引（D18（块里携带什么信息） 已定项 11），从不跨头 COW。两条不同的断言并存：「容器被 COW 重写时身份不变」（内容同、位置变）与「分裂后左半身份不变」（内容少了、身份不变）。类型 1 的容器号仍按 C103（容器索引无落点） 那套单调分配，两条规则按记录类型分。

**key 与 inode 号**：key = inode 号 8 字节，唯一性范围 (树 ID, inode)，位宽 64，声明生命周期创建总数上界 2^63。**已发布的 (inode 号, 对象出生代) 不复用**（D6（快照实现模型） 判据 9 的戒律）：`deleted_inodes` 后台回收窗口里复用会让新旧对象的 extent key 三段逐字相同。⚠️ **「已发布的」这四个字不是为了对齐措辞，是让现行设计自洽**：水位住记账（D5（快照 / 空间记账机制） 已定项 4 第 12 项），被抛弃 checkpoint 的那笔账随它一起丢 ⇒ **每一次普通崩溃**都会让那一代发过的 inode 号被重发。管理员回退是挂着时的一次向前发布，那次发布的 inode 号水位取 max(现行那一版内存里的, 环里读得出、按实例表没被崩溃恢复抛弃的根的)，已发过的号不重发（D23（journal 的角色与格式） 已定项 14）；崩溃恢复落到较旧的根时水位随所选根那棵账载入，被抛弃时间线里已发过的号会被重发，而恢复之后每一个新对象的出生代都大于环里读得出的每一条根与记录的 txg（D23（journal 的角色与格式） 已定项 14 注 3）——要几个故障才撞得上被抛弃时间线里已发布对象的出生代，崩溃恢复那一形没量过，见 C143（inode 号水位在崩溃恢复落到旧根后退回去重发）；对外暴露的对象身份 = (inode 号, 对象出生代)，凡按 inode 号认对象的结构都要带上出生代或随根回退。`deleted_inodes` 回收窗口里 extent key 三段逐字相同那一句说的是同一条时间线里的复用（旧对象已发布、它的 extent 在树里等后台回收），照旧被禁；崩溃恢复之后重发同一个号，新对象的 extent 落在所选根那棵 extent 树里，旧对象的 extent 随被抛弃的根一起不在树里——**这靠 extent 树与 `deleted_inodes` 都随所选根**；管理员回退那次发布的新根取 R_old 的 inode 树与 extent 树，`deleted_inodes` 也取 R_old 那一版的（否则 R_old 之后删掉、随 R_old 的 inode 树回来的对象会被后台回收再收一次；主 agent 2026-09-26 定，推的，被攻过零轮）；`deleted_inodes` 的形态（已定项 16）守这两条；盘上分开两者的是单元五元组里的对象出生代（D9（加密） 已定项 6）。**inode 记录在回收完成前留在 inode 树里（nlink = 0），`deleted_inodes` 只存号**。

**下一个 inode 号是记账里的一条统计量**（D5（快照 / 空间记账机制） 已定项 4 第 12 项「inode 号水位」）：key = (统计量, 树 ID, 设备段, 代)，不带设备维，value = 下一个可用号，**合并规则取最大值**（幂等、交换、结合）；条目照样带 seq（已定项 15 第 3 条是格式要求，一棵 write buffer 树一套条目格式），只是**合并结果不依赖它**。选它而不是「后者胜」的理由是崩溃后 txg 重发那一格：同一个代段里先后出现大小两个值，取最大值给大的（浪费号、不复用号）；「未发布的等于没发生」只对常规挂载路径成立，扫描重建看的是盘。「标签 → 合并规则」是盘上语义，统计量登记表立成独立冻结组件（D15（格式冻结政策））。水位行只为可写头维护，照台阶行的规矩写：水位变了的那次发布留一行台阶行，随根 COW 一直留着（D5（快照 / 空间记账机制） 已定项 2 每条根的记账树只装这条根那一代的行、每次发布把这一代的行全写一遍，对它开了例外，见 D5（快照 / 空间记账机制） 已定项 2 射程「inode 号水位的台阶行」那一条），只读树没有这一行；读现行值 = 取该维度元组下代号最大的那一行，空 ⇒ 从未维护过、不是 0。克隆头：新头的水位 = origin 那一刻的运行时水位（含还在 write buffer 里没落盘的那条），建克隆那次发布必须为新树写出这条水位行；两个头此后各自分配，号会相交，唯一性靠 (树 ID, inode)。

**记录字段表（定长 140，小端；对齐口径是记录内偏移，容器内第 n 条起点 136 + 140n（码 3 头 107 + nonce / MAC / 算法类型预留位 29，D18（块里携带什么信息） 已定项 16）不按 8 对齐，按非对齐读）**
<!-- format-const: INODE_RECORD_BYTES = 140 -->
<!-- format-const: INODE_LEAF_RECORDS = 233 -->

| 偏移 | 字段 | 宽 |
|---|---|---|
| 0 | inode | 8 |
| 8 | 对象出生代（创建那次发布的 checkpoint_txg；inode 号不复用后它不再承担复用判别，按五元组 / AAD 的 day-1 契约保留，扫描时与该对象每个数据单元头逐字段对照） | 8 |
| 16 | locality_id（第一版无父目录取 0） | 8 |
| 24 | mode / uid / gid / nlink，各 4 | 16 |
| 40 | size / blocks / rdev，各 8 | 24 |
| 64 | atime / mtime / ctime 秒，各 8（unlink 的意图那次三个共写调用方给的一个值，各批照抄） | 24 |
| 88 | 改动计数（最后一次改动所在发布的 checkpoint_txg；提交锚 = 叶容器头的诞生代号；同一发布窗口内多次改动只有最后一版落盘——内部节点不缓冲；unlink 的意图那次与每一批也算改动，写本次发布的 checkpoint_txg，不照抄上一版；创建算一次改动，所以改动计数不低于记录偏移 8 的对象出生代，I-9.26（inode 改动计数落在出生代与容器诞生代号之间）） | 8 |
| 96 | atime / mtime / ctime 纳秒，各 4 | 12 |
| 108 | 填充（恒 0） | 4 |
| 112 | flags（恒 0） | 8 |
| 120 | 预留（恒 0） | 20 |

**写入时间（偏移 64 起三个秒字段、偏移 96 起三个纳秒字段）取写这条记录的那一次发布的参数，不取系统时钟**：每次发布调用带一个写入时间，同一个参数两次发布逐字节相同（可复现）。三个秒字段都写参数的秒数；三个纳秒字段写参数的纳秒部分，参数只带秒数，所以写 0。**新建**（新池新建文件、新建 inode）写创建那次发布的参数；**改写内容**（覆盖写、顺序写）重写这条记录时写这一次发布的参数，不照抄上一版的值；**unlink 的意图那次**写调用方随这次 unlink 给的时间，三个时间秒（atime、mtime、ctime）共一个值，各批照抄意图那次写的值；**不改这条记录内容的发布**——同一片叶里别的记录改动、这条被连带重写，回退——照抄这条记录上一版的值。

**`blocks` 是逻辑长度的 512 字节块数**：`blocks` = ⌈`size` ÷ 512⌉，按文件长度向上取整，**不表示分到的空间**，不按这个对象分到几个数据单元算。新池新建文件那条记录 3000 字节 ⇒ 6；新建、还没写过数据的 inode ⇒ 0。

不放 extent 指针（D14（双轨（大小文件 / 持久临时）） 已定项 3 内联阈值 0），btime 不进记录。**flags / 填充 / 预留三段同一政策：恒 0，读者遇到非 0 ⇒ 该记录 EIO（对象级，不拒整个容器）**——前向兼容政策不是损坏检出（载荷校验和照样过），落成 I-9.7（记录三段恒零），在 I-1.7（打包容器合法与判定顺序） 的中止链之外。将来给任一位 / 任一字节赋义按 D15（格式冻结政策）「改变已有字段含义」判 incompat，退役的字节永久报废。**记录内不存在 compat 扩展路径**：inode 属性都有语义依赖，D15（格式冻结政策） 判定表第一行够不着，且记录被反复重写、旧读者会把不认识的内容丢掉。预留 20 不承重：记录内任何赋义都是 incompat，宽度只决定那次要不要连记录宽一起改。**溢出路线按属性逐个判档**：无语义依赖的属性走新增 keyspace（D15（格式冻结政策） 判定表给 compat），有依赖的走 flags 位 / 预留（incompat）；属性 keyspace 没有落点（xattr 被推迟）。**记录宽必须等于登记表给类型 2 的宽**，不等判损坏（I-1.7（打包容器合法与判定顺序） 里在「类身份段」之后、I-1.2（块头写序已发布） 之前插一步，只需头字段）；改宽是 incompat。记录格式、码 3 类身份段对类型 2 的用法、inode 树的 key 编码是永久字节：**立成一个独立的已冻结组件**（C9（冻结后偷改格式） 按组件记），冻结时刻是第一个写出类型 2 容器的镜像；不动 D15（格式冻结政策） 的四层；类型 1 的记录格式（宽 40，D18（块里携带什么信息） 已定项 24）各自独立。

**不变量**：I-9（inode 树结构） 一类，十四条，见 [invariants/09-inode树结构.md](../../../crates/singlefs-checker/invariants/09-inode树结构.md)；状态列统一带前置「树的种类可识别（已定项 8）」，第一版镜像上可由类型 2 容器头里的出生树反推哪棵树是 inode 树。nlink == dirent 条数不在这里立——它的门（dirent 树存在）没有输入，随 dirent 树上线时一起立。第一版的唯一文件取 inode 1（0 无效），nlink = 1，不进孤儿回收。分裂 / 合并的身份传递与合并方向以外的写路径纪律是操作断言，走单测 / 模型对拍与崩溃点重放。

**重建**：内部节点丢了，收集打包记录类型 2 的容器，按 (出生树, 容器号, 容器出生代) 分组，同身份多版的择现行 = C113（扫描重建时多版单元的现行版本判定无输入）；退役树里有退役记录指认的身份不收，退役记录带死亡 txg、按谱系滤（与 I-9.18（删除队列里的号有谱系上的墓碑） 对墓碑同一读法），退役树、墓碑树、谱系树这几棵随现行的树只按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 择版本；每叶 key 范围由记录算，内部节点的 key 区间由叶算；产物落 D18（块里携带什么信息） 已定项 5 的级 1（只读挂载），级 2 的见证者不存在。**extent 树的重建**：对象按 (树 ID, inode) 认、只看本头的 extent 树；这个对象的全部 extent key（幸存的与找回的）编同一个首段，取值 inode 记录 > 幸存 extent key > 0，记录完好时重建视图里的值就是记录里的值；取错只丢局部性。**数据单元的死活**：按重建出的 inode 记录的对象出生代与 size 折成的单元数判，文件变短不写新记录（读法 Z）。I-9.9（locality 三值对照） 在重建之后按侧判：一侧是从另一侧编出来的就输出「不可判定」。

**新池新建文件写**：一个码 2 根节点（头带 key 区间 [1, 1]；一条条目：key 1、类型 2、出生树、容器号 1、出生代 3、指针）；一个码 3 叶（类型 2、容器号 1、出生代 3、记录数 1、记录宽 140）；一条记录（inode 1、出生代 3、locality 0、nlink 1、改动计数 3、三段 0）；记账里一条「inode 号水位 = 2」。

**射程**：**同一条时间线上 inode 号不复用**——「时间线」指所选根的祖先链：含管理员回退抛弃的那一段（回退那次发布的水位取 max，那一段里发过的号不重发），不含崩溃恢复抛弃的那一段（崩溃恢复落到较旧的根之后，被抛弃那一段里发过的号可以重发）。**唯一性的射程是一条已发布历史内**——每个容器号都是某条记录首次插入时的 inode 号；号在崩溃恢复落到较旧的根之后可以重发（管理员回退那次发布的水位取 max、不重发，D23（journal 的角色与格式） 已定项 14），所以同一棵树里两个容器可以同号，分开它们的是容器出生代（右半的出生代 = 触发分裂那次发布的 txg，而恢复之后新实例的每一个 txg 都大于环里读得出的每一条根与记录的 txg，D23（journal 的角色与格式） 已定项 14 注 3），同一次发布内「分裂 → 合并 → 再分裂」也造不出同身份两版。跨崩溃不成立：崩溃后 checkpoint 号会被重发，被抛弃那版与重试那版四段逐字相同、内容不同、都在盘上——这与数据单元的五元组、码 2 节点的 (树, 层级, 诞生代号) 是同一种暴露（根记录靠 D16（发布语义） 骑手条款 1 的实例代号分辨，别的单元类靠类身份段里的写序 (实例代号, 事务号) 分辨——容器不单独加实例代号，写序里那 4 字节就是它），常规路径由 D3（空间分配） 已定项 1 的谓词（分配记录）判，扫描重建路径按 C113（扫描重建时多版单元的现行版本判定无输入） 定案的 P2 / P3 规则判。⚠️ **判它安全的依据与树 ID 那条同源**：孤儿与活版的实例代号必不相等（D18（块里携带什么信息） 已定项 11「作废一个 checkpoint 必须换实例代号」），扫描重建的两道闸按写序择新。⚠️ **残留的疑点**：讲统计量合并规则那一句「『未发布的等于没发生』只对常规挂载路径成立，**扫描重建看的是盘**」读起来像是反方向的——它管的是「水位在一个代段里取最大值而不是后者胜」，与「未发布过的号可不可以重发」不是同一个命题，但两者挨得很近。⚠️ **重建的射程是单头镜像**：有克隆头时共享叶头里的出生树是 origin（I-1.3（块头树 ID 一致） 的 C110（跨头共享与加密后元数据类的块头一致性读法） 那一格），按出生树认领会把克隆头的树认残——多头镜像的归属划分要克隆祖先表（E90（树 ID 进 AAD 与跨头共享） 第二笔），落地前多头镜像的 inode 树重建没有输入。⚠️ 填充率没有下界：`⌈n/233⌉` 是新建树的值，1e6 顺序创建再删 90% 不合并是 9.98 倍空间放大；加宽记录能同时缩小爆炸半径（160 ⇒ 204 条），这笔取舍归 C100（inode 容器丢失是永久损失而没有任何冗余条款覆盖它）。⚠️ **当时挡着「新池新建文件写」的九样（里程碑一），① ② 已定案，其余照录**：① 树表条目已由已定项 8 定；② 记账条目的 seq 住 value、宽 4 字节（已定项 7）；③ 统计量标签值（C71（不认识的统计量标签静默忽略））；④ 不带设备维的统计量的设备段取值、新建树的记账行初始化、读现行值的读法（C117（不带设备维的统计量的设备段没有取值））；⑤ 码 3 容器带不带扩展点（C108（扩展点归属对打包记录单元无判定））；⑥ 码 2 节点头的基础字节数（已由已定项 11 收口）；⑦ `deleted_inodes` 的形态（已定项 16）；⑧ ⑨ 记录格式与统计量登记表两个冻结组件的 spec 文件（C119（冻结组件没有 spec 文件））。⚠️ 跨头首次 COW 时容器身份要不要重生的另一半归 E90（树 ID 进 AAD 与跨头共享） 第二笔。⚠️ **`blocks` 不承担对外的 `st_blocks`**：stat 报的 `st_blocks` 怎么给（stat 时从 extent 树现算分到的单元）、SEEK_HOLE / SEEK_DATA，挪到做 POSIX 面的里程碑。`blocks` 按文件长度向上取整改新池新建文件的字节：那条记录偏移 48 的 8 字节从 64 变 6。 ⚠️ **写入时间单镜像判不出**：取值是发布参数，单个镜像里看不出参数是什么——O1（内存模型对拍（HashMap 理想文件系统）） 的模型与对拍不带写入时间，O2（独立解析器 + checker） 只判单个镜像，O3（独立规约执行器） 的投影不进 inode 树；判它的是按写路径钉写的 harness 用例（覆盖写、顺序写、建 inode、unlink 各一条），多单元顺序写、回退、同一片叶里被连带重写的别的记录三格没有用例（C677（写入时间在三格写路径上没有用例））。

⚠️ **死亡分三层的两笔代价**：墓碑从意图那次生效，unlink 途中扫描重建读不到那条 nlink = 0 的记录与剩下的数据单元，崩溃点上重建视图里不多出意图已发布的对象（E178（叶容器与记录死亡的计数） 第二、三段）；借时右片保留身份，借之后它的容器号可以不大于左片的最大 key，I-9.4（容器号不超最小 key） 第三句（左容器的最大 key < 右容器号）随这一形删掉，不变量清单与 checker 已跟着改（2026-10-09 第 50 批）；代价是「分裂时右半容器号给小了、仍大于左邻号」那一类镜像上没有判据拦，只剩 `inode_tree` 单测。

**依据**：

- E98（inode 记录与 inode 树的几何）：inode 记录的盘上形态——打包进码 3 不触发 D21（权威态与派生态的分界） 硬约束 6、住索引节点触发（14 个字段里 11 个算不回来）；改动计数单靠 checkpoint_txg 排不了同窗口内两个版本，所以内部节点不缓冲、同一发布窗口只有最后一版落盘。
- E103（inode 打包路 ① 的更新代价）：路 ①（独立容器索引 + inode 树按身份引用）的读代价比容器索引并进内部节点高、还多一次单元读，而单次写不变——容器索引不另立树。
- E104（扫描重建的现行版本判定）：不认退役记录时退役容器里已死的 inode 会复活；模型打出「重生取最小 inode 号当容器号会撞车」，所以重生保留容器号、只换出生树与出生代。
- 四轮三方论证后用户定案 2026-09-05（材料 `_d8-item6-path.md`、`_d8-item6-r2-amended.md`、`_d8-item6-r3-conclusion.md`、`_d8-item6-r4-conclusion.md`、`_d8-item6-r5-landing.md`，四轮三腿输出 `research/prompts/d8-item6-r{1,2,3,4}-{sonnet,opus}-output.md` 与 `research/results/d8-item6-r{1,2,3,4}-local.out`）。
- 用户定案 2026-09-12（C297（inode 内部条目里出生树两份没有定处理），两轮三方）：身份引用里的出生树与子指针里的那份两份并存，原话在变更史。
- 用户定案 2026-09-16（C143（inode 号水位在崩溃恢复落到旧根后退回去重发），三轮三方）：唯一性收窄到 (inode 号, 对象出生代)，原话在变更史。
- 用户定案 2026-09-23：`blocks` 按文件长度向上取整、不表示分到的空间，`st_blocks` 挪到做 POSIX 面的里程碑（主 agent 的提议，C480（inode 记录的 blocks 怎么算全仓没有条款）），原话在变更史。
- 用户定案 2026-10-01：K-R3-3「写根长高」，`research/prompts/m3-delete-r3-main-verification.md` 第三节 G6-墓碑树根满（原话在变更史）。
- 判决 `research/prompts/c673-clauses-r1-main-verification.md` 第二节 K3：写入时间在各写路径上的取值（两腿同，攻方在副本上量过、主 agent 复跑坐实），三条 oracle 都判不到。
- 用户定案 2026-10-03：删除步算改动（「算改动，改实现」）、时间字段取「乙 意图带时间、三格同改」，原话在变更史；代码轮第三轮判决 `research/prompts/m3-unlink-code-r3-main-verification.md` 第二节 n3 那一行（收窄掉「ctime 跟着改」没回问用户，记录偏移 64 起三格写同一个值）。
- 用户定案 2026-10-05：射程加「同一条时间线上 inode 号不复用」（「写吧」），判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T15 那一行，原话在变更史。
- E176（inode 树叶合并三候选的容量与写代价）：合并买到成倍容量（Q1r，对负载不稳定），左吸收右与左从右借多重写的叶片按位置记法不翻面、按内容记法可忽略——合并与借都做；借到刚好够下限时内容记法的多重写由「可忽略」变「不翻」——借取均分。
- E177（逐树不写墓碑的复活数）：删空的叶容器不落盘、什么都不写时，照 kb 择现行的扫描重建读法下 inode 树复活——容器身份消失要另有载体（死亡分三层的第 ② 层）。
- E178（叶容器与记录死亡的计数）：退役记录带死亡 txg、按谱系滤时，借与吸收、管理员回退的历史上容器复活与误判死亡都是 0；文件变短按重建出的 inode 记录判死活时复活与误判死亡都是 0；墓碑从意图那次生效时 unlink 途中漏 1 条记录与剩下的单元、崩溃点上不多出对象；借时给右片换身份每次比保留身份多写码 3、码 2 各一个与一条退役记录。
- 三方判决 `research/prompts/m3-leaf-death-r1-main-verification.md`（第二节 K1：「死亡由现行叶版本体现」不整条推翻、改字面；K3：均分借）、`research/prompts/m3-leaf-death-r2-main-verification.md`（第二节：退役记录带死亡 txg、随现行的树只按 P2 择版本；分隔 key 取孩子这一版的最小 key；均分借维持）、`research/prompts/m3-leaf-death-r3-main-verification.md`（第二、五节）。
- 用户定案 2026-10-08：「M3 连借一起做」（岔路单 `m3-3-5-forks.md` M 那一行）；容器身份消失写退役记录·甲、文件变短取读法 Z、墓碑从意图那次生效、借时分隔 key 取孩子这一版的最小 key 并删 I-9.4（容器号不超最小 key） 第三句（岔路单 `m3-leaf-death-forks.md`，判决 `research/prompts/m3-leaf-death-r3-main-verification.md` 第五节）。原话在变更史。
- 退役记录的写者只追加、死亡 txg 取身份消失那次发布、读者与 checker 只认谱系上的：判决 `research/prompts/m3-retire-r1-main-verification.md` 第二节 K2 的「死亡 txg 取哪次」「同一身份退役几次」两行（两次攻方抽样都坐实：死亡 txg 取 unlink 意图那次或最后一版的诞生代号会误判死亡，写者「已有就不追加」或「按身份覆盖」会复活）与第三节第 3、4 条；用户定案「开池就建（推荐）」，原话在变更史。

**欠**：C100（inode 容器丢失是永久损失而没有任何冗余条款覆盖它）、C102（索引节点头没有预留区，加字段不是 compat）、C103（容器索引无落点）、C104（扫描路径上码 1 / 2 的载荷完整性无自包含证据）、C106（开放容器数上界）、C107（索引节点的 AAD 身份无落点）、C110（跨头共享与加密后元数据类的块头一致性读法）、C113（扫描重建时多版单元的现行版本判定无输入）、C115（inode 属性的溢出路线无落点）、C116（叶容器的分裂 / 合并纪律没有会失败的检查）、C119（冻结组件没有 spec 文件）、C152（收窄后未发布号重发的扫描重建复核）、C347（祖先表读不出时克隆头认领什么没有条款）、C677（写入时间在三格写路径上没有用例）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:235-252`（整段抄，未转述）**

```markdown
#### 已定项 7：write buffer 条目的 `seq` —— 4 字节，住 value

**定案**：**`seq` 宽 4 字节，住 value 不住 key**，射程覆盖记账 / 反向索引 / LRU 三棵 write buffer 树，**一棵树一套条目格式、三棵同宽**。宽度定死之后因果倒过来：**`T_dirty` 的上界从此是挂载时判定**（4 字节 = 32 位窗口，按 34 字节/条对应约 136 GiB）；D16（发布语义） 已定项 5 那句「`T_dirty` 是可调值，不是格式常量」原样成立，改的是它多了一个由格式定的上界。**在 seq 里再放一份 txg 是把 key 里已有的段抄第二遍**，不许。两条骑手：

1. **反向索引与 LRU 定 key 时必须让去重窗口同样被 `T_dirty` 界住**（代进 key，或别的等效办法）。4 字节够用的依据是「同一个 key 的待去重条目必然同属一个发布窗口」，而那一条在记账树上由 D5（快照 / 空间记账机制） 已定项 5 的 key 末段是「代」推出来；那两棵树的 key 布局全仓一个字都没有，不兑现这条骑手，4 字节在它们身上没有依据。
2. **同一个 key 的跨批次插入顺序必须与 `seq` 同向**，不许留成隐含假设——btree 层同 key 是「谁后写谁赢」，不是「谁 `seq` 大谁赢」，而按叶分区的并行 flush 下两个批次的物理落地先后不受 `seq` 约束。

**射程**：它定的是条目里这一段的宽度与落点，不定别的段（记账 key 的 `(统计量类型标签, 树 ID, 设备, 代)` 四段在 D5（快照 / 空间记账机制） 已定项 5）。⚠️ 这一段不是与别段正交的：那四段在**同一个 checkpoint 内不唯一**（幂等完整值会被写很多次），去重规则「后者胜」逐字要求「后者」由条目自己携带 ⇒ 不定它，那四段合起来还不是一个定得下合并结果的 key。它**立在 D8（核心索引结构） 不立在 D5（快照 / 空间记账机制）**，因为 write buffer 覆盖「反向索引 / LRU / 记账」三类，立在记账那边会让另外两棵树各自重问一遍。⚠️ 记账条目因此从 30 字节（key 22 + value 8）变 **34**，E97（记账与分配记录的条目编码） / E71（记账 key 的条目数与宽度） / E54（记账的「代」要保留几代） / E99（write buffer 条目与 seq 的去重） 四页的数要重算。⚠️ 「inode 号水位」那条记账条目照样带 `seq`（已定项 15 第 3 条是格式要求），只是它的合并结果取最大值、不依赖 `seq`。⚠️ 取值规则（从几起、一次发布里怎么数）在已定项 10；前端与节点缓冲怎么衔接在已定项 12。

**依据**：

- E99（write buffer 条目与 seq 的去重）：量的正是同一个 key 的待去重条目在一个发布窗口内能有多少条、去重要几位——窗口需求落在 32 位以内，4 字节够。
- E17（write buffer 合并上限）：第一版漏了条目序号，两条臂与真值当场对不上，逼出「条目必须自带 `seq`」这个可判定形态；按叶分区可并行的实测同时说明并行 flush 下物理落地先后不受 `seq` 约束，骑手 2 因此必须显式写下来。
- 三条已定条款的字面冲突（三方论证，主 agent 逐条现查）：D5（快照 / 空间记账机制） 已定项 2 逐字「不用找 ⇒ 是点删不是扫描；触及的 key 数 = 统计量数」——`seq` 进 key 之后同一前缀下有多行，点删当场不成立；已定项 6 逐字「条目照样带 seq……只是**合并结果不依赖它**」——单调统计量取最大值要求两条相遇，而 `seq` 进 key 之后它们是不同的 key、根本不会相遇；D5（快照 / 空间记账机制） 已定项 5 的 key 定义本身不含 `seq`，而「后者由 `seq` 定」只有在 `seq` 不进 key 时才自洽。主 agent 补的第四条：`seq` 进 key 会让 key 不可构造——折叠后同一代只留一条，读者想查 `(标签, 树, 设备, 代)` 的值时不知道 `seq`，点查退化成前缀扫。
- 用户定案 2026-09-06：宽度取 4 字节并把因果倒过来、射程扩到三棵树、`seq` 住 value（原话在变更史）。

**欠**：C161（缓冲两级并存而衔接没定）、C162（seq 进 value 之后四个实验的数要重算）、C345（反向索引 / LRU 的陈旧检出没有时间线索）、C416（去重窗口被 T_dirty 界住那条骑手没人查）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:253-297`（整段抄，未转述）**

```markdown
#### 已定项 8：树表条目的字段表

**定案**：**树表条目合计 200 字节** = **树 ID 8**（打头）+ 条目长度 2 + 树的种类 2 + flags 2 + 根指针 86 + `previous_snapshot_txg` 8 + 诞生 txg 8 + **头 ID 8** + 预留 76。每层 ⌊(16384 − 131) / 200⌋ = ⌊16253 / 200⌋ = **81** 棵，二层封顶 81² = 6561 棵。⚠️ **分母的口径写死在这里**：16384 减**含预留位的码 2 头**（131 是树表单元自己的码 2 头，key 宽 8 ⇒ 115 + 16，D18（块里携带什么信息） 已定项 16 / 已定项 18；条目区起点是含预留位的头末尾，已定项 11），不减别的数。

- **① 快照进树表**，每个只读快照两条条目：码 14（快照 inode 根）与码 15（快照 extent 根），根指针逐字抄建快照那一版活头的两个根；**诞生 txg 8 字节**，口径是**快照完成（发布）那一次 checkpoint 的 txg**，不是根节点的诞生代号（那是它最后一次 COW 的号，这棵树自己算不出来）。它给 I-3.7（活快照可枚举） 一个载体。
- **② 树 ID 水位住根记录，取根环里全部根记录该字段的 max**。逐字形态：`新水位 = max(根环里全部根记录的该字段, 本次 checkpoint 发出的最高树 ID + 1)`，**每次发布随根记录重写**。「根环里全部根记录」读作根环里全部自证过的根（被抛弃时间线上的也算），并上环里全部自证通过的 journal 记录新根段带的该字段（D23（journal 的角色与格式） 已定项 15）：根所在的槽读不出而它那条记录还在时，恢复本来就从记录把那条根重建出来，记录带的水位就是那条根的水位；journal 记录要等它对应的树写好、根落盘之后才会被绕环盖掉。根与它那条记录都读不出（双重介质故障）的那一格盖不住，记在 C342（树 ID 水位在根读不出时退回去重发）。从 mkfs 之外的水位发第一个文件版本时，十三棵树的号从水位起连号发，次序照已定项 11 那一组 65536..65548（extent、inode、分配记录、记账、中央映射、livelist、稀疏旁表、deadlist、`deleted_inodes`、墓碑树、谱系树、logged_ops、退役树）。⚠️ **「累计」这两个字是承重的**：读成「本 checkpoint 发出的最高号」（不累计）时，树 ID 会随根轮出环被重发，而**那时被重发的号属于已发布的树**，D6（快照实现模型） 判据 9 正面命中 ⇒ 整条定案不成立。
- **③ `previous_snapshot_txg` 每头一份，住树表条目**（8 字节）。它是**每个头一个标量**不是每个单元一份，答的是「这个头从哪个快照分出来」，与单元自己的内容无关 ⇒ 按 D1（数据可移动性 / 反向索引） 已定项 4 的判据它进不了单元的自包段。一条树表条目上有**两个各自独立的 8 字节 txg 字段**——诞生 txg（这条自己发布的号）与 `previous_snapshot_txg`（它前驱那个快照的号），对活头条目与快照条目都有定义。
- **④ 兼容档取「树表条目自带长度」这一层，不取「索引节点每条目自带长度」那一层**：前者让旧读者**跳过**一棵不认识的树；后者让旧读者**解开**新树的节点，而**没有消费者**——它不是被判成本高，是被判**买不到要的东西**。
- **㊀ 旧写者对不认识的记账行**（限「树 ID 不认识、统计量标签认识」这一半）：**写者必须把这类行原样带到新代**（值不变、只换代号；有前端的写者把这类行当前端条目写、随发布排空到叶，第一版写者没有前端（已定项 10）、直落叶，盘上布局相同、读不错）。台阶行不换代段（D5（快照 / 空间记账机制） 已定项 2 射程「inode 号水位的台阶行」），上一版那次发布水位变了时同一行既是现行行又是台阶行。
- **㊁ flags 其余 15 位**：**未知位一律拒收——该条目 EIO**（对象级，不拒整个池）。**不取**「忽略未知位、继续按已知语义解释」，那是静默误读。配两条纪律：**必须被旧读者看见的改动走系统配置 incompat 位，不许塞进每条目的 flags**；**checker 不许走「跳过」**——I-3.1（已分配统计对得上） 逐字「已分配空间统计 == 实际遍历**所有**引用得到的和」无限定，checker 跳过一棵不认识的树，遍历和就短一截，判出来的是假阳性红而不是漏检，它遇到不认识的树的正确行为是**报「无法完整遍历」并停**。
- **排序契约与嵌套层数**：**条目按树 ID 升序排；嵌套层数不设格式上限**。排序取树 ID 的理由：它是五元组、记账 key、容器索引里到处用的那个键，且 ② 让它随创建序严格递增。不设上限的理由：设上限等于多一个失败模式，而那个上限还随条目宽变。
- **平局判别子取 `(诞生 txg, 树 ID)`，零字节**：D16（发布语义） 已定要用 `(txg, 快照 id)` 打破并列，而 ② 改住根记录之后树 ID **严格随创建序递增**（水位累计、只增不减，崩溃只烧号不回退——I-7.8（根记录树 ID 水位不低于全池最大树 ID) 把这一条变成可判的）⇒ `(诞生 txg, 树 ID)` 就是与创建序一致的全序，而树 ID 本来就在条目里。
- **头 ID 8 字节的取值**：inode 树条目写自己的树 ID（65537），extent 树条目写它服务的那个头（65537），`deleted_inodes`、墓碑树与退役树写 inode 树的树 ID（65537），谱系树写 0（全池一份），logged_ops 写 0（整理意图条目的字段表已定，见 D26（后台整理与放置回收） 已定项 2；C364（整理意图记录的字段没有逐字段对账） 仍欠字段表与实现的逐字段对账，I-4.4（意图只描述目标态） 的字段白名单判法已实现），分配记录树 / 记账树 / livelist 共享树 / 稀疏旁表树 / deadlist 树写 0（无归属，与 D5（快照 / 空间记账机制） 给「全池 / 无归属」行的约定同一个数）；中央映射树不进树表。见 D5（快照 / 空间记账机制） 已定项 9。 ⚠️ 括号里的 65537 是树 ID 从 mkfs 水位 65536 起发的那一版（新池新建文件）上的取值，判据是关系、不是字面值：inode 树条目的头 ID 等于它自己的树 ID；extent 树、`deleted_inodes`、墓碑树、退役树条目的头 ID 等于同一张树表里 inode 树条目的树 ID；谱系树、logged_ops、分配记录树、记账树、livelist 共享树、稀疏旁表树、deadlist 树写 0。树 ID 不是从 mkfs 水位起发的那一版（水位已被抬过，或一个克隆头的 inode 树）上，头 ID 随这张表里 inode 树的树 ID 走，不再是 65537（D5（快照 / 空间记账机制） 已定项 9「可写头的 ID = 该头 inode 树的树 ID」）。判它的是 I-9.27（树表条目的头 ID 取值）。
- **预留 76 字节**：已知认购者按「一段字节只有一个意思」排，**逐条登记在树表条目预留的认购表里**（门禁 doc-registries 的 tree-table-reserve 格按它算和：认购合计不许超过 76，表里每条要指得到一个还开着的欠账或一条已定分项）。合计 48，余 28。

<!-- gate:tree-table-reserve total=76 -->

| 认购者 | 住哪条条目 | 宽度 | 出处 |
|---|---|---|---|
| livelist 每头计数器 | 头条目 | 8 | C271（livelist 的 FREE 在什么时刻记没有定义） |
| 第二道销毁闸的活快照计数 | 头条目 | 8 | C272（第二道销毁闸没有 O(1) 的检查法） |
| origin 链接 / 克隆时刻 | 头条目 | 8 | C271（livelist 的 FREE 在什么时刻记没有定义） |
| 快照谱系的区间标号 | 快照条目 | 16 | C144（快照谱系的区间标号没有载体） |
| `children`（子快照计数） | 快照条目 | 8 | D5（快照 / 空间记账机制） 已定项 18 |

⚠️ **inode 号水位不在表里**：C143（inode 号水位在崩溃恢复落到旧根后退回去重发） 2026-09-16 定案「不搬水位，水位照旧住记账」（D5（快照 / 空间记账机制） 已定项 4 第 12 项）⇒ 它不认购这里的字节。

**改新池新建文件的字节：是**——树表单元 t12 的条目宽字段（200）、声明长度（12 × 200 = 2400）与 12 条条目的排布。

**射程**：⚠️ **㊀ 的射程只到「树 ID 不认识」那一半，另一半的处方是相反的**：不认识**统计量标签**那一半由 C71（不认识的统计量标签静默忽略） 管，而它逐字要求「挂载时遇到不认识的统计量标签**必须按 compat_ro 拒绝可写挂载**」——㊀ 不覆盖也不推翻它。两半分得开的机制依据：D5（快照 / 空间记账机制） 已定项 5 的 key 定宽 ⇒ 不认识标签**含义**的写者照样能按固定偏移切出各段；成立的机制是已定项 1「记账条目取幂等完整值」，一条行的值对搬运者不透明也搬得动，而写者不认识那棵树就改不了它 ⇒ 值本来就不变。不搬的代价：D5（快照 / 空间记账机制） 已定项 2 定每条根的记账树只装这条根那一代的行、每次发布全写一遍 ⇒ 写者不认识的那棵树的行没人重写，下一次发布就消失。⚠️ **暴露面很窄**：只有一个统计量带树维（D5（快照 / 空间记账机制） 已定项 4 第 12 项，只为可写头维护）⇒ 不是可写头的树一行都没有；再加一个带树维的统计量，不搬的丢失量就按树数长。⚠️ **㊁ 拒收不丢数据依赖 ④ 的条目自带长度**：条目 EIO 但**位置还在** ⇒ 写者知道「那儿有一棵树我读不了」⇒ 不做跨它的 deadlist 合并；没有长度字段就定位不到下一条，这条推导不成立。⚠️ **㊁ 与本工程对另外两个 flags 字段已定的政策是同一条**：D18（块里携带什么信息）「偏移 7 是 8 个 flag 位……读者遇到非 0 位一律拒收……本工程不给这 8 位留「忽略陌生位」的通道」；已定项 6「flags / 填充 / 预留三段同一政策：恒 0，读者遇到非 0 ⇒ **该记录 EIO**」，已落成 I-9.7（记录三段恒零）⇒ 三个 flags 字段并成一条，不用再解释它们为什么不同。⚠️ **「位」和「种类」是两根轴**：D15（格式冻结政策） 已定项 2 那条 compat（新增 keyspace 对旧读者只是看不到新树）管的是**种类**，㊁ 管的是**位**——一棵旧读者自以为认识的树被加了位，它的语义变了，那不是加法。⚠️ **嵌套层数不设上限的代价是性能悬崖**，按 `.claude/rules/fs-design.md` 硬要求 4 必须可观测：「每次发布 COW 的树表单元数 = 层数」要能在运行时读出来——走错层数不报错，没有观测手段就永远发现不了。⚠️ **① 交付的那张表不够用**：D5（快照 / 空间记账机制) 点名的「全部根的 txg 列表」写在 D6（快照实现模型） 已定项 1（每头一棵自己的树）之前，而 D6（快照实现模型） 用一个反例打穿了它的前提（主线 txg=10 建可写快照 X、在 X 里于 txg=30 写一个块、主线 txg=35 建快照 Y，按 D5（快照 / 空间记账机制） 的条件判「Y 引用它」而 Y 根本看不见它）⇒ 要的不是「每个根一个 txg 标量」，是**谱系**；诞生 txg 这 8 字节仍然非有不可（I-3.5（引用区间的精确性） 要按 txg 升序枚举全部快照根与活头、I-3.6（deadlist 紧界） 要 `prev(S).txg`），欠的是**在它之上再加一层标号**（嵌套区间标号是两个数，而且随谱系变化要回头改写，与「诞生 txg 写一次不再动」是完全不同的格式性质）。同一个前提的三个消费者里定案只兑现了 I-3.7（活快照可枚举） 一个：I-3.5（引用区间的精确性） 的可判定形式建在「全部根按 txg 排成单一全序」上，谱系成树之后那一条恒假；D18（块里携带什么信息） 的代际回收走同一个区间测试，方向是过判 ⇒ 少回收 ⇒ 泄漏。⚠️ **③ 只补了活头头部的半格**：D5（快照 / 空间记账机制） 逐字「每个可写头各持一份 `previous_snapshot_txg` **与一份 deadlist**」「建快照时把**活头的 deadlist 整个交接给新快照**，活头换一个空的」⇒ 快照条目也要能指到一份 deadlist，而 deadlist 的盘上落点全仓一处都没有。⚠️ **丙-2 唯一盖不住的那一格判为不是错误**：某个 checkpoint 分配了树 ID 42、写出带 42 的单元头，根没写成就崩了 ⇒ 水位随那个根一起丢 ⇒ 下次挂载重发 42，而盘上留着带 42 的孤儿单元；判它不构成错误的依据是孤儿与活版的实例代号必不相等（D18（块里携带什么信息） 已定项 11「作废一个 checkpoint 必须换实例代号」）、I-1.2（块头写序已发布） 逐字「判不成已发布者必是未发布的撕裂事务垃圾或被抛弃时间线的残留」、扫描重建的两道闸与孤儿回收条件都不看树 ID、孤儿与活版共用五元组本来就是常态（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P1 加 10 字节写序正是为此）、D6（快照实现模型） 判据 9 的理由句说的是「树里的 key 变成可见」而孤儿不在任何树里。由此 D6（快照实现模型） 判据 9 的射程按它自己的理由句收窄成「**已发布**的快照标识单调不回收」（2026-09-06 用户定案，同一次）。⚠️ **inode 号水位另给答案**：D5（快照 / 空间记账机制） 已定项 4 第 12 项逐字「**带**（只为可写头维护）」⇒ 每个可写头一个标量、条数随头数长，而根记录也是定长槽 ⇒ 丙-2 同样搬不过去；2026-09-16 定案不搬水位，把唯一性收窄到「已发布的 (inode 号, 对象出生代) 不复用」（已定项 6）。⚠️ **树表容量在两处按不同条目宽算**：E79（根记录的容量） 与 D22（单元原子性怎么合成） 已定项 7 报的每层棵数是另一个模型（它把树表条目当成「树 ID 8 + 根指针 59」= 67 字节），方向不变（间接层躲不开、超过一层再嵌一层），只是门槛比 D22（单元原子性怎么合成） 正文写的低。⚠️ 预留里那两笔已知认购（deadlist 的落点、快照谱系的嵌套区间标号）**没有顺手命名成字段**——命名等于把两个未定项一并定了，那是两个决定。排序契约（① 树 ID 严格升序，② 合发号次序）由 I-9.16（树表条目按树 ID 严格升序且合发号次序） 判：重建与冷走读在读任何一棵树之前拒，池级 checker 另判一份（实审 A3b Q2 与树表排序实现，用户 2026-09-27 定立不变量并同步）。第一版同时活着的只读快照至多 10 个（用户定案，常量 `SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION`；树表单节点封顶 34 个）。快照按码 14 条目的树号点名（第一版没有目录）。有活快照（现行版或目标版树表里有码 14 / 15 条目）时管理员回退一律拒（用户定案，D23（journal 的角色与格式） 已定项 14）。

**依据**：

- E112（旧写者遇到不认识的树）：跑前写死判据与接受条款下六格的静默结局与搬运代价——`Skip + Carry` 在挂得上的那些里零静默结局，`Ignore` 那一格记录被按错编码解释，`Refuse` 全挂不上且与 D15（格式冻结政策） 已定项 2 的 compat 档不相容；搬运代价每代只多写两行，跑前定的作废阈值未触发。
- E113（不认识的树·补齐臂集）：臂集补成 5 flags × 3 记账、选法改成跑前写死的字典序之后，`RejectEntry` 在全部扫描点里一次都没输过，`Skip` 在第 1 关就输（隐形快照使 deadlist 合并认错邻居）、`Ignore` 输在第 2 关、攻击腿提的按位分档 `PerBit` 输在它没有第三种行为——㊁ 由此从「跳过该条目」重定为「该条目 EIO」；记账那一维取 `Carry` 不取 `Exempt`（当时 `Exempt` 要在 D5（快照 / 空间记账机制） 已定项 2 的 K 代上开例外，那笔代价一个格子都没占；2026-09-23 K 代点删撤回，`Exempt` 臂失去对象）。
- 用户定案 2026-09-06：四问（快照进树表、树 ID 水位住根记录、`previous_snapshot_txg` 每头一份、兼容档取条目自带长度）、㊀ ㊁、排序契约与嵌套层数、平局判别子、预留取 32 字节（原话在变更史）。
- 用户定案 2026-09-23：② 的 max 也并上环里记录新根段带的水位（C342（树 ID 水位在根读不出时退回去重发） 前置那一问；C511（回退到无文件那一版之后诞生代怎么接） 第 3 步的实现员先选了这一边，用户同日认下，原话「journal 日志就是为了保证树能完整写入的」），原话在变更史；发号次序是主 agent 同日照格式常量次序定的，被攻过零轮。
- 用户定案 2026-09-14：树 ID 挪到条目最前、根指针 83 → 86、预留里切 8 字节做头 ID（原话在变更史；依据是已定项 11 定码 2 条目里 key 一律是条目的前 key 宽 个字节，树 ID 不打头就要给通用解码规则加例外）。
- 用户定案 2026-09-16：条目从 148 加宽到 200、预留 76，代价是每单元从 109 棵降到 81 棵、二层封顶从 11881 降到 6561，用户知情接受（原话在变更史）。
- 判决 `research/prompts/c673-clauses-r1-main-verification.md` 第二节 K1：头 ID 读同表关系、不读字面 65537——字面读法在一段合法的高水位历史上误红 4 条，还放行头 ID 写死 65537 的坏实现（攻方在副本上量过，主 agent 复跑坐实）；判它的 I-9.27（树表条目的头 ID 取值） 按表判，树表有条目时才判。
- 退役树的头 ID 写 inode 树的树 ID、树表声明长度 12 × 200 = 2400：判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 1 条（树表条目的头 ID 写 inode 树的树 ID，同墓碑树，被攻过零轮）；用户定案「开池就建（推荐）」，原话在变更史。
- 判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1（每个快照两条新种类的树表条目，甲出局；单节点树表封顶 34）、S8（有快照时回退拒）与第七节用户定案（岔路 1 上限 10、岔路 3 有快照就拒；岔路 5 快照按码 14 的树号点名是主 agent 定的），原话在变更史。

**欠**：C144（快照谱系的区间标号没有载体）、C145（deadlist 在盘上没有落点）、C147（树 ID 水位的不变量还没有）、C148（树表条目变宽把间接层门槛压到二层）、C149（根槽重发之后 birth 口径松一格）、C156（记录编码误读没有不变量）、C158（checker 跳过不认识的树会判假阳性）、C159（E112 的臂集缺三条且天平不含挂不上的代价）、C342（树 ID 水位在根读不出时退回去重发）、C675（同一种树在树表里至多一条没有不变量）、C865（树表条目未知 flags 位让整张树表读不下去）、C866（F2 退回不分字节丢了与内容拒收）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:298-334`（整段抄，未转述）**

```markdown
#### 已定项 9：树的种类码的登记表

**定案**：树表条目里「树的种类」那 2 字节的取值，唯一登记位是已定项 9 这一节的登记表；**新树只追加不重用码**，0 保留为「无」。

| 码 | 树 | 出处 |
|---|---|---|
| 0 | 无（树表条目不用它） | — |
| 1 | extent 树 | D4（校验和位置） 已定项 7 |
| 2 | inode 树 | 已定项 6 |
| 3 | 分配记录树 | D3（空间分配） 已定项 3 |
| 4 | 记账树 | D5（快照 / 空间记账机制） 已定项 3 |
| 5 | 中央映射树 | D19（块指针的结构与宽度预算） 已定项 5 |
| 6 | livelist 共享树 | D6（快照实现模型） 已定项 2 |
| 7 | 稀疏旁表树 | D5（快照 / 空间记账机制） 已定项 6 |
| 8 | deadlist 树 | D5（快照 / 空间记账机制） 已定项 11 |
| 9 | `deleted_inodes` 树 | 已定项 16 |
| 10 | 墓碑树 | D18（块里携带什么信息） 已定项 24 |
| 11 | 谱系树 | D18（块里携带什么信息） 已定项 24 |
| 12 | logged_ops 树 | 已定项 5 |
| 13 | 退役树 | D18（块里携带什么信息） 已定项 11 |
| 14 | 快照 inode 根 | 已定项 8 |
| 15 | 快照 extent 根 | 已定项 8 |

新池新建文件写出的 **12** 条树表条目照此表（码 1..4、6..13 各一条；中央映射树的根住根记录、不进树表，D19（块指针的结构与宽度预算） 已定项 11；码 5 照登记）；E142（新池新建文件的干跑） 的 `TREE_KIND_*` 常量是码 1..8 那八个值（码 9..15 它没有）；`crates/singlefs-core/src/records.rs` 与 checker 的 `walk.rs` 用同一组常量（码 5 不进树表，那里没有它）。

**射程**：它只登记「树的种类」这一段的取值，树 ID 是另一个量，两个量的值域不相交（树 ID ≥ 65536，取值在已定项 11）。与 D18（块里携带什么信息） 已定项 11 的单元类登记表是两张表、两个概念（用户不取合并）。码 8（deadlist）与 livelist 共享树、稀疏旁表同一个先例——都是「销毁快照时要读、而销毁不许全盘扫描」的结构，day-1 注册进树表比第一次建快照时惰性创建少一条「首次建树」路径；条目形态另立 D5（快照 / 空间记账机制） 已定项 12。**改新池新建文件的字节：是**——加码 8 那一次让树表从 6 条变 7 条、树 ID 水位从 65543 变 65544（树 ID 从 65536 起编，deadlist 是 65543）。

**依据**：

- 无实验：码值是取值约定，没有可量的量；登记表由用户 2026-09-13 定案（码 0..7）、2026-09-14 定案加码 8，两次都没有走三方论证、可推翻，原话在变更史。
- 码 9..12：判决 `research/prompts/m3-blanks-r1-main-verification.md` 第二节种类码那一行（两腿各自得出）；树 ID 与种类码值域不相交由用户定案 2026-10-01 取（岔路单 `m3-blanks-r1-forks.md` 的「种类码与树 ID 的错开」一行，原话在变更史）。
- 判决 `research/prompts/c673-clauses-r1-main-verification.md` 第二节 K2：新池新建文件写出的树表条目数按实现现值（判决当时 11 条，加退役树之后 12 条，`crates/singlefs-format/src/lib.rs` 的 `NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES`）。
- 码 13 退役树、新池新建文件写 12 条：判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 1 条（种类码追加登记、树表排最后、day-1 零根注册，取值照墓碑树与谱系树的先例，被攻过零轮）；用户定案「开池就建（推荐）」，原话在变更史。
- 码 14 快照 inode 根、码 15 快照 extent 根：判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1（取两条新种类的树表条目，追加登记、不重用码；甲出局两腿一致，具体字段零轮）。

**欠**：C315（树的种类码没有登记表）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:335-346`（整段抄，未转述）**

```markdown
#### 已定项 10：第一版记账直落叶，`seq` 的取值规则

**定案**：**第一版记账直落记账树叶，不走 write buffer 前端**；D5（快照 / 空间记账机制） 已定项 3 第 3 条读成「前端是后续优化」。**`seq` = 同一个 `(标签, 树, 设备, 代)` 在一次发布里被写进树的第几次**（不是运行期那个统计量被改的次数），**从 1 起**；第一版禁边插边 COW（D19（块指针的结构与宽度预算） 已定项 12 的固定点：每个码 2 / 码 3 一个实例一个 checkpoint 只写出一次，发布末尾一次性 COW）⇒ 每个 key 每次发布只写进树一次 ⇒ **恒 1**；合并规则取最大值的统计量也照写。

**射程**：它定的是第一版走哪条路与 `seq` 怎么数，不定 `seq` 的宽度与落点（已定项 7）、也不定前端与节点缓冲的衔接（已定项 12，第一版两级都不开）。放开固定点、开前端或节点缓冲之后「恒 1」还成不成立，由已定项 13 的状态折叠接着管。

**依据**：

- 无实验：第一版走不走前端是实现排期的选择，没有可量的量；由用户 2026-09-13 在写代码前的总审核里定案，没有走三方论证、可推翻，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:347-380`（整段抄，未转述）**

```markdown
#### 已定项 11：码 2 的载荷布局与 key 比较序

**定案**：码 2 节点头自描述（预留 2 之后是条目数 2 + 条目宽 2，key 宽 1 排在 key 区间之前，偏移表见 D18（块里携带什么信息） 已定项 18），基础字节数 81 → 86；含 29 字节预留位后头 = `115 + 2 × key 宽`。条目区从含预留位的头末尾（`115 + 2 × key 宽`）起，**定宽条目连续排列**，**声明长度 = 条目数 × 条目宽**；条目之后的空闲字节恒 0 且参与载荷 CRC。载荷 CRC 覆盖 `[86 + 2 × key 宽, 16384)`（起点是明文头末尾，口径与码 1 的 105、码 3 的 107 同）。**条目里 key 一律是条目的前 key 宽 个字节**——「读条目宽 → 按条目宽切条目 → 取前 key 宽 个字节当 key」是一条对全部码 2 树都成立的通用解码规则，扫描重建路径上不用另查一张「这棵树的 key 在条目里排第几」的表。key 的全序按逐字段无符号整数、字段自左向右比较，**盘上小端存储不构成 memcmp 序**。树 ID 从 65536 起编（种类字段 2 字节取不到的最小值，两个量的值域永不相交）：extent 65536、inode 65537、分配记录 65538、记账 65539、中央映射 65540、livelist 65541、稀疏旁表 65542、deadlist 65543、`deleted_inodes` 65544、墓碑树 65545、谱系树 65546、logged_ops 65547、退役树 65548；mkfs 时水位 65536，第一个文件版本那次之后 65549。

**改新池新建文件的字节：是**——六个码 2 单元的条目区起点（`115 + 2 × key 宽`：inode 树根 131、extent 163、分配记录 135、记账 159、映射 169、树表 131）、声明长度，以及树表条目的字段次序；树 ID 字段（树表条目里的树 ID 与头 ID、码 2 头里的树 ID、记账 key 的树 ID 段）与树 ID 水位的值也变，随树 ID 从 65536 起编。

**多层码 2 树怎么长、怎么收**（记账树、中央映射树，以及照已定项 14 仍用码 2 btree 的树）：① 一次发布里分裂新建的节点与别的单元落在同一段，整条根到叶的路径照常 COW，不加写序步骤、不加屏障；② 叶装不下时从中间切；③ 收缩只摘空节点，根只剩一个孩子时降高一层，不做「低于一半合并」与借条目；④ 分隔 key 跟着维护：插到最左分隔 key 之下时把它压低，条目在兄弟间挪动之后把右边那个孩子的分隔 key 改成它新的最小 key；逐层可判：分隔 key ≤ 孩子节点头的最小 key，且 > 左邻孩子节点头的最大 key（D18（块里携带什么信息） 已定项 18 的 key 区间）；⑤ 树高 = 根节点码 2 头里的层级 + 1。

- **一次发布里先删后插**：上一版有、这一版没有的 key 先按 key 升序删，再按 key 升序插这一版新有的；只决定形状，不决定写序（写的是最终态，已定项 13）。
- **从盘上重建的树分隔 key 坏了**：重建只核拼得成树的几样（层级、节点不空、内部条目宽、不重复引用），不核分隔 key；下一次发布按分隔 key 删不掉时规划交回，可写挂载在取号之前拒，盘上不变。遇到这种盘之后怎么办（修复、只读挂载）没定。
- **叶条目宽不由读树判**：读树只切 key，叶条目窄于字段表由解条目的一方各报各的；内部条目窄于 key + 86 由读树报。

**内部节点的条目 = 本树 key + 子指针 86**：记账树 22 + 86 = 108、中央映射树 27 + 86 = 113。分配记录树与 extent 树按已定项 14 另起按 key 空间寻址的结构，结构在三方 `m2-keyspace-r1` 判；照码 2 btree 做时它们的内部条目是 96 与 110（extent 110 是用户 2026-09-24 早先定的）。

<!-- format-const: ACCOUNTING_INTERNAL_ENTRY_BYTES = 108 -->
<!-- format-const: CENTRAL_MAPPING_INTERNAL_ENTRY_BYTES = 113 -->

**射程**：它管码 2 节点的载荷怎么排与 key 怎么比，不管头里各字段的偏移（D18（块里携带什么信息） 已定项 18）、也不管每棵树的条目里除 key 之外还有什么（各树自己的分项）。唯一因「key 打头」需要重排的是树表条目，已按这条改成树 ID 打头（已定项 8）。「定宽条目连续排列」与「声明长度 = 条目数 × 条目宽」在中央映射条目回到定宽 55 之后不用动（D19（块指针的结构与宽度预算） 已定项 10）——条目宽字段只有一个 u16，按类两宽时这两句顶不住。两条 2026-09-14 补的定案都是用户弹窗定案，没有走三方论证，可推翻。⚠️ 分裂、收缩、分裂点、写序四样被攻过零轮，三方原型只抽了一次样；「低于一半合并」与「追加时末尾切」在崩溃历史上与选中的那两样零对零，没选的理由是用户的取舍，不是安全性。「分裂单开一次发布」出局：只改结构的发布挤掉 D16（发布语义） 已定项 1 的「最近 4 个可退到的不同状态」。inode 树的叶是码 3 打包记录单元，不是码 2 节点：「多层码 2 树怎么长、怎么收」里 ② 的「叶装不下时从中间切」与 ③ 的「不做低于一半合并与借条目」管码 2 节点，够不着 inode 树的叶层；叶层的末尾分裂、左吸收右与左从右借（均分借）照已定项 6。

**依据**：

- E145（码 2 自描述头与映射树 key 宽的代价）：量的正是码 2 头自描述要加的那几段与映射树 key 宽的代价——加到 86 之后扇出、树高、解头那几节都不掉档，自描述值得做。
- 用户定案 2026-09-13：码 2 头自描述、基础 86、条目区起点、载荷 CRC 覆盖范围、key 比较序（原话在变更史）。
- 用户定案 2026-09-14：条目里 key 一律是前 key 宽 个字节；「定宽条目连续排列」与「声明长度 = 条目数 × 条目宽」原样成立（原话在变更史）。
- 用户定案 2026-09-24：extent 树内部条目 110 字节（原话在变更史）；E157（并行线一两条条款的计数模型） 在 110、136 两个取样点上结论都不翻面。
- 无实验：分裂、收缩、分裂点、写序四样是三方原型（`research/prompts/m2-treesplit-r1-opus-model/`）在同一批崩溃历史上零对零之后由用户选的，没立实验号；分隔 key 跟着维护是那一轮主 agent 按打中定（原型 t1.out：沿用 D8（核心索引结构） 已定项 6 的「分隔 key 创建后不变」24 格全红、跟着维护 24 格全 0），判决 `research/prompts/m2-treesplit-r1-main-verification.md`。
- 用户定案 2026-09-24：写序同段照常 COW、只摘空节点且根剩一个孩子时降高、从中间切、内部条目 = 本树 key + 子指针 86（原话在变更史）。
- `m2-treesplit-implementer-report.md` 第五节 B1、B5、B6。
- 用户定案 2026-10-01：树 ID 整体挪到 65536 起，与种类码的值域不相交（岔路单 `m3-blanks-r1-forks.md` 的「种类码与树 ID 的错开」一行，原话在变更史）。
- 判决 `research/prompts/m3-blanks-r1-main-verification.md` 第二节种类码那一行：新树的种类码取 9..12（两腿各自得出）。
- 退役树的树 ID 65548 与第一次发布后的水位 65549：判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 1 条与第五节（取值照先例，被攻过零轮）；用户定案「开池就建（推荐）」，原话在变更史。

**欠**：无（C306（码 2 头不自描述 key 宽） 2026-10-11 已还清，见欠账表已还清表）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:381-394`（整段抄，未转述）**

```markdown
#### 已定项 12：write buffer 前端与节点消息缓冲区的衔接 —— 一棵树只准一级

**定案**：走 write buffer 前端的三棵树（记账 / 反向索引 / LRU）**ε 恒 0、内部节点不留消息缓冲区**；这三棵树上的写**一律进前端**——含已定项 8 ㊀ 那类旧写者搬运的记账行：有前端的写者把它当前端条目写，第一版写者没有前端（已定项 10）、直落叶，盘上布局相同、读不错；**发布前把前端排空到叶**，落叶时同 key 比同一窗口的到达序；发布集的前端条目在同一次发布里排空、按事务切，不跨发布（fsync = 一次全量发布）。其余树按 D11（索引节点要不要留消息缓冲区） 已定项 6 的树轴留不留缓冲。**第一版两级都不开**（已定项 10、D11（索引节点要不要留消息缓冲区） 已定项 7），不改新池新建文件的字节。

**检查**：前端树的写路径只暴露前端入口（类型层）；运行时断言——一个 checkpoint 里前端树叶上的每一处逻辑改动（key → 值）都对得上一条这个 checkpoint 进过叶的前端条目，窗口中途 flush 的与发布时排空的都算、实例切换重发时保留；重组、索引重建、损坏恢复重建三个例外各配一条「前后每个 key 读出来的值不变」。它只能是运行时的：盘上分不出经前端排空与直写落叶。按字面写的三种断言（只认发布时排空的、只认这一次尝试的、按物理新叶数）在合法历史上会误报（`c161-r3-model-output.txt` 的 `G3_publish_only` / `G3_attempt` / `G3_physical` 行）。

**射程**：**代价**：每次发布排空到叶的写出是「排空到根缓冲」那类形态的 3.4–10.7 倍（覆盖写负载 4.8–10.6 倍，同一产物 `part` 列的第三段），发布集的条目都得在这次发布里落叶、不能按预算分批；同一份脏页预算下前端装的条目少一个量级——`T_dirty` 按什么算没有口径。**买到的**：前端树上没有同 key 跨级并存，读的次序最短；陈旧检出（C52（陈旧当成合法））在这三棵树上没有对象。**没出局但没选的**：甲-R4（前端直落叶、节点缓冲只收不走前端的写）与乙-D（前端进根缓冲逐层下推、按位置定新旧）各只在塌缩上被打中一次，它们的陈旧检出要每条消息带发布号（与已定项 7「不许再放一份 txg」冲突），节点布局进 D15（格式冻结政策） 第 4 层。**连带**：重组时节点缓冲怎么搬（射程是全部留缓冲的树，这一项只省了前端三棵）；墓碑与叶上现值的折叠起点（已定项 13）；反向索引 / LRU 的时间线索；D11（索引节点要不要留消息缓冲区） 已定项 6 树轴的分支变量与已定项 5 的措辞。

**依据**：

- 无实验：这一格量的是设计形态在合法历史上会不会被分辨臂打中，依据是三轮三方（`research/prompts/c161-r1-main-verification.md`、`c161-r2-main-verification.md`、`c161-r3-main-verification.md` 各自第四节）——这一形在三轮试过的全部形状（乱序 flush、K 代点删、分裂与合并、部分发布、排空做到一半崩溃、删除、树高 4、索引重建、实例切换、管理员回退）上没被打中；出局的乙 + seq 全池单调两轮被打中（计数器不落盘；实例切换时计数器从所选根重新载入），还要 8 字节 seq 与落盘计数器。用户 2026-09-16 收尾弹窗定案，原话在变更史。

**欠**：C161（缓冲两级并存而衔接没定）、C345（反向索引 / LRU 的陈旧检出没有时间线索）、C348（T_dirty 按什么算没有口径）、C349（重组时节点消息缓冲区怎么搬没有条款）、C383（同一次发布里同 key 的中间态折叠没有检查）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:395-407`（整段抄，未转述）**

```markdown
#### 已定项 13：索引树更新做状态折叠 —— 同一次发布里同一个 key 的中间状态不落叶

**定案**：一棵索引树上的同一个 key 在同一次发布里被改了多次（A → B → C → D → E）时，落叶只做一次：叶里这个 key 的条目直接写成最终态 E，A–D 不写进任何节点。「同一周期」是一次发布：同一个 `checkpoint_txg`，fsync 触发的也是一次发布（D16（发布语义） 已定项 6）。**E 按同一次发布里的到达序取最后一次改动，不按值取**；E 是墓碑时，折叠的结果就是墓碑，不许把墓碑当数值与 A–D 一起取最大（C383（同一次发布里同 key 的中间态折叠没有检查） 实测这种朴素折叠四条候选 16 / 16 丢点删，`b.out`）。**折叠的起点是叶上这个 key 在上一次发布留下的现值（没有则无）**：排空 = 以它为起点、按到达序依次应用本次发布里该 key 的前端条目，墓碑 = 删；起点算在 A–D 之前，不算同一次发布里的一个中间态。开前端之后，记账行的「上一代离开」落成墓碑，对着的就是这个起点。到达序在内存里由条目自己带着（已定项 15 第 3 条），不靠按 key 排序之后的相对位置。

**射程**：**全部索引树的全部落叶路径**。第一版的直落叶（已定项 10）之外，以后开的 write buffer 前端排空（已定项 12：落叶时同 key 比同一窗口的到达序，比出来的最后一条就是 E）、节点消息缓冲区（D11（索引节点要不要留消息缓冲区） 已定项 6 的树轴）、边插边 COW（D3（空间分配） 已定项 5 允许、D19（块指针的结构与宽度预算） 已定项 12 第一版不取的那条实现）都照这个规矩落叶。**第一版已经满足它**：D19（块指针的结构与宽度预算） 已定项 12 的固定点让已定项 10 推出「每个 key 每次发布只写进树一次」、`seq` 恒 1；已定项 13 把这件事从固定点的推论提成独立条款，放开固定点之后照样成立，不改新池新建文件的字节。实现今天没有对象：`crates/singlefs-core/src/transaction.rs` 的 `publish_overwrite` 一次覆盖写就是一次发布（txg、事务号各加一），没有「同一次发布里几个事务改同一个 key」的路径；同一个 checkpoint 里装两个对象的用例改的是不同的 key。⚠️ **没答的两条**：① A–D 在树之外的去向不归它管——A–D 是同一个逻辑块被覆盖写几次得到的数据单元版本时，按 D16（发布语义） 已定项 2 的丢失窗口与切分纪律「一事务一单元」，A–D 各是一个事务，各有一个落了盘的数据单元与一条 journal 记录（D23（journal 的角色与格式） 已定项 7：一条记录一个事务号），这几个单元没有根引用、也不在 I-7.4（近 K 代块未被复用） 的保护里，它们要不要写、写了在同一次发布里怎么释放、记账怎么算，决策正文、欠账表、不变量清单里都没有管它的条款；② 已定项 7 的盘上 `seq` 在折叠之下恒 1（已定项 10 的「恒 1」从第一版扩到所有版本），而认 E 用的是内存里的到达序，盘上那 4 字节 `seq` 还拿不拿来当 `T_dirty` 上界的依据（已定项 7 定案里「`T_dirty` 的上界从此是挂载时判定」那一句），没答。⚠️ 「单调统计量在同一次发布里不减、所以两种取法给同一个数」是主 agent 的推论，没有检查。

**依据**：

- 无实验：用户定「这个是决策 记录 暂时不需要做和验证」，没开三方、没建实验，也没有可量的量；用户 2026-09-19 定案与同日对「同一周期 = 一次发布」的确认，原话在变更史。A–D 可以不落的三条支点是已有条款（主 agent 2026-09-19 现查）：D16（发布语义） 已定项 4 与 D23（journal 的角色与格式） 已定项 14 第五条（施加的单位是一次发布，合法前缀停在一次发布的两个事务之间时那次发布整体不施加，所以 replay 出来的只有发布边界上的状态）、D16（发布语义） 已定项 6（fsync 触发的也是一次发布，被 fsync 确认过的状态都落在发布边界上）、D16（发布语义） 已定项 14（快照发布点对齐 checkpoint 边界、钉在这个 checkpoint 全部数据变更之后，没有快照引用 A–D）。
- 折叠起点一句与墓碑怎么折并进已定项 13：判决 `research/prompts/m3-close-c693-r2-main-verification.md` 第二节 V4 一格（缩题并进已定项 13 站住，两腿一致）与 `research/prompts/m3-close-c693-r3-main-verification.md` 第二节 T4 一格（站住；检查取排空对拍，台阶 H 严格递增那一判随里程碑十四的分树做）；改法照正推腿 `research/prompts/m3-close-c693-r2-sonnet-output.md`「V4.5 要动的条款、欠账」那一节。正推腿模型 `research/prompts/m3-close-c693-r2-sonnet-model/out/v4_fold_checks.out` 第 1、3 行：单头两次建 inode 的历史上 20 次发布之后，重建落法终态 18 行，朴素取最大 35 行、17 行陈旧（第二轮判决核查第 4 条主 agent 复跑 identical=1）。

**欠**：C383（同一次发布里同 key 的中间态折叠没有检查）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:408-462`（整段抄，未转述）**

```markdown
#### 已定项 14：一套 btree 实现 + 多个独立 keyspace + 两个前端，派生树可以异构

**定案**：**一套 btree 实现 + 多个独立 keyspace + 两个前端，管权威态的树与照分裂做的树；派生态的分配记录树与 extent 树（D21（权威态与派生态的分界） 已定项 5、9）可以另起按 key 空间寻址的结构，格式可以与码 2 btree 不同。**

| 层 | 做什么 |
|---|---|
| 独立 keyspace | 每类数据一棵树（extent / inode / dirent / 反向索引 / 空闲空间 / 记账 …），互不干扰 |
| 日志结构节点 | 节点内新 key 追加不插入，内存里定期重排；写顺序、可在只读锁下回写 |
| **write buffer** 前端 | 高频小更新（反向索引、LRU、记账）批量攒起来排序去重后批量插入 |
| **key cache** 前端 | 同一批 key 反复读写的（分配信息）走哈希表，不做全树查找 |

**两棵派生树的结构**：

- **分配记录树**：按绝对槽号按位置寻址。每片叶罩一段固定的槽 `[k × W, (k + 1) × W)`，W 取偶数（两槽数据单元永远不跨叶），且 W ≤ 叶条目容量 812（一条记录至少罩一个槽，叶永远装得下）；记录按起点槽落叶。往上各层按位置寻址，子节点罩哪一段由它在父节点里的位置算出。**没有记录的一段 = 全空闲**：那片叶不写，父节点那一格留空（用户 K1）。checker 逐节点核「这个节点罩的就是它的位置规定的那一段」与「记录的末槽不越过它所在叶的末槽」。
- **extent 树**：两段，都按位置寻址。上段按 inode 号的位置寻址；下段一个文件一棵，按数据单元号的位置寻址（叶罩 144 个单元，内部扇出 147），洞就是缺席。**只有一个数据单元的文件不建下段**：上段叶条目里直接放那个数据指针；条目带一个标签字节区分「没有单元 / 下段根指针 / 内联数据指针」（用户 K2）。**上段规范形**：上段形状只由有条目 inode 的号集合定，写与删共用这一条规则：节点取各有条目 inode 的根路径（从它那片叶到根那一串）之并，根层级取罩得住最大号的最低一层，集合为空时取层级 0 的零条目根；完整的重写规则在已定项 16 第 8 条。
- **挂载怎么读**：分配记录树挂载时整棵读进挂载态（分配器本来就要）；extent 树按需，打开文件时按位置走下去（用户 K4）。
- **实现取值**（实二一取，主 agent 2026-09-25 定，被攻过零轮）：
  - **分配记录树**：
    - 叶宽 W = 812（取叶条目容量 (16384 − 135) ÷ 20 本身），内部条目 96 字节（key 10 + 子指针 86），扇出 169。
    <!-- format-const: ALLOCATION_RECORD_TREE_LEAF_SLOTS = 812 -->
    <!-- format-const: ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES = 96 -->
    <!-- format-const: ALLOCATION_RECORD_TREE_INTERNAL_FANOUT = 169 -->
    - 层级 L 的节点罩 W × 169^L 个槽。
    - 内部节点与根只写有孩子的格：每个孩子一条内部条目（key 10 + 子指针 86），条目的 key 取这个孩子按位置罩的那一段的起点（孩子所在的盘, 那一段的第一个槽号），按 key 排，没有孩子的格不占条目；读者按这个 key 认孩子，槽号不落在孩子那一层的格点上、盘不在池里、或认出的孩子不在这个父节点名下的，这条条目不指任何孩子（`crates/singlefs-core/src/allocation_record_tree.rs` 的 `child_named_by_entry_key`）。根节点头的 key 区间罩整个 key 空间，smallest_key = (0, 0)，largest_key = (0xFFFFFFFF, 2^48 − 1)。
    - 根罩整个 key 空间、按盘分流；根层取最小的 R ≥ 1，使 Σ_盘 ⌈盘上槽数 ÷ (W × 169^(R−1))⌉ ≤ 169。4 GiB × 2 时根在第 2 层、树高 3。
  - **extent 树**：
    - 上段叶条目 113 字节 = key 24（(0, inode, 0)）+ 标签 1 + 载荷 88。标签 0 全零，只有读者认、写者不写；标签 1 是下段根的节点指针 86 字节加 2 字节零；标签 2 是那一个数据单元的数据指针 88 字节。
    <!-- format-const: EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES = 113 -->
    - 上段叶罩 143 个 inode，内部条目 110 字节（key 24 + 子指针 86），扇出 147。
    - 上段叶节点头的 key 区间照 D18（块里携带什么信息） 已定项 2 射程段写这片叶按位置罩的那一段，不写叶里第一条与最后一条条目的 key：闭区间，第 k 片（从 0 起）smallest_key = (0, 143k, 0)，largest_key = (0, 143k + 142, 2^64 − 1)。
    <!-- format-const: EXTENT_TREE_UPPER_LEAF_INODES = 143 -->
    <!-- format-const: EXTENT_TREE_INTERNAL_ENTRY_BYTES = 110 -->
    <!-- format-const: EXTENT_TREE_INTERNAL_FANOUT = 147 -->
    - 下段叶罩 144 个单元，下段 key 的字节偏移 = 单元号 × 32634。上段与下段的内部条目，key 都取这个孩子按位置罩的那一段的起点（`crates/singlefs-core/src/extent_tree.rs` 的 `build_extent_internal_entry`）。
    <!-- format-const: EXTENT_TREE_LOWER_LEAF_DATA_UNITS = 144 -->
  - **发布路径的固定点**：猜重写集 → 在分配器拷贝上走释放（释放弄脏的叶先并进重写集）→ 取落点 → 算脏节点 → 不在集合里就并进去重来。第 64 轮还没收拢，就在任何落盘之前返回 `AllocationRecordTreeRewriteSetDidNotSettle`；64 没有依据，见新立的欠账。
  - **盘上槽数**：取盘上整槽数，盘尾不足一槽的零头不算；写侧与读侧只有一处定义（`crates/singlefs-core/src/allocator.rs` 的 `absolute_slot_count_of_device`），分配记录树根层两侧同一个数（实二六取，主 agent 2026-09-25 定，被攻过零轮；用例 `half_a_slot_past_a_root_level_boundary_leaves_the_writer_and_the_reader_on_the_same_root_level_and_every_tree_written_reads_back`）。
  - **打开文件时自报的读数**：`ExtentTreeReadsAtOpen` 数的是读了几个 extent 树节点，不是块层的设备读次数；提示过期、经中央映射回退时每跳多 2 次设备读，两个数不相等（实二六查出，主 agent 2026-09-25 认这个口径、不另加设备读数，被攻过零轮；用例 `opening_a_file_whose_lower_extent_segment_has_two_levels_reports_as_many_node_reads_as_the_block_layer_counts` 钉相等的那一格，`a_leaf_of_a_two_level_lower_extent_segment_with_a_stale_location_hint_reads_back_every_data_unit_through_the_central_mapping` 钉不等的那一格，都在 `crates/singlefs-harness/tests/tree_nodes_and_the_central_mapping_on_mounted_read.rs`）。
- 两棵树的节点照样带 D18（块里携带什么信息） 已定项 18 的码 2 头（树 ID、层级、key 区间、诞生代号），树高照 D8（核心索引结构） 已定项 11「树高 = 根节点头层级 + 1」读。

**射程**：⚠️ 异构只放给派生态、可重建的两棵树：分配记录树与 extent 树；记账树（权威态）、中央映射树、树表、实例表、inode 树照一套码 2 btree 做（D8（核心索引结构） 已定项 11 的分裂与收缩）——inode 树只有根与内部节点是码 2，已定项 11 管的是这一层；叶是码 3 打包记录单元，叶层的末尾分裂、左吸收右与左从右借（均分借）照已定项 6，已定项 11 的「叶装不下时从中间切」与「不做低于一半合并与借条目」够不着它。异构结构照样要：单个节点自描述（哪棵树、哪一层、管哪一段 key、第几代，D18（块里携带什么信息））、损坏可检出（D21（权威态与派生态的分界） 已定项 11）、每一种新写序进崩溃点重放崩溃点、checker 另有一套判它的代码。那两棵的结构在三方 `m2-keyspace-r1` 判。⚠️ 「日志结构节点」说的是**内存里的排布**——盘上每次落盘是整单元 COW 重写，按已定项 11 的单一定宽条目数组写（声明长度恰等于条目数 × 条目宽），不是多段追加的 bset。⚠️ **write buffer 前端第一版不实装**（已定项 10：记账直落记账树叶）；前端与节点消息缓冲区怎么衔接在已定项 12，条目要满足的三条硬要求在已定项 15。索引结构可以多样、事务层不许多样那条纪律在 `.claude/rules/fs-design.md`，本项只定索引这一侧。⚠️ 叶宽 W 的具体取值、上段扇出、上段叶条目的完整字段表，条款没写到的由实现员照上面几条取、交回里写明，主 agent 定后补进这一节。⚠️ 三种新写序（长高多层时位置 0 那一串节点同一次发布写、矮下去多层时同一串一起释放、内联与下段互换）要进崩溃点重放，checker 判定随实现一起做。⚠️ 这一节的四项都被攻过零轮；「全局按位置＋内联」的崩溃历史没扫（用户定全量崩溃最后统一跑）。⚠️ 被否掉的候选：全局按 key 排（上段要分裂）、每文件一棵根住 inode 记录（要改已冻结的 inode 记录）、全局按位置不内联（一次挂载里连续建到第 14625 个小文件就假性报满，违反 D3（空间分配） 已定项 9 第 1 条）——理由在判决第三节。

**依据**：

- 无实验：一套 btree 实现配多 keyspace 的形态没有可量的量，依据是 2026-08-29 现查 `linux-6.17/fs/bcachefs/bcachefs_format.h` 的 `BCH_BTREE_IDS()` 宏（数 `x(` 行与带 `BTREE_IS_write_buffer` 的行，源码固定在 `~/code/fs-refs/`）：bcachefs 用**一套 btree 实现 + 21 棵树**，其中 **5 棵**走 write buffer（`lru` / `backpointers` / `deleted_inodes` / `rebalance_work` / `accounting`）；比「按访问模式选不同结构」好在**只有一套结构要调对**——异构方案意味着 N 套实现、N 套崩溃一致性 bug、N 套 checker。prior-art.md 的 bcachefs 那一行以这一段为准。
- 用户定案 2026-09-24：派生树可以异构（原话在变更史）；多出来的那套实现、崩溃一致性与 checker 由崩溃点重放流与 checker 各自覆盖，不再拿「只有一套结构要调对」挡。
- 无实验：三方原型（`research/prompts/m2-keyspace-r1-opus-model/`）在同一批历史与代价模型上量过，没立实验号，选哪一个由用户定，判决 `research/prompts/m2-keyspace-r1-main-verification.md`。
- 用户定案 2026-09-24：K1、K2、K4（原话在变更史）。
- 实二一报告 `m2-keyspace-implementer-report.md`：实现员取值，主 agent 认，被攻过零轮。
- 实二六报告 `m2-e26-implementer-report.md` 第六节 Q3、Q4：盘上槽数与打开文件自报读数两条取值，主 agent 认，被攻过零轮。
- E142（新池新建文件的干跑） 第十五次跑第五段（`e142-r15-step5-runner-report.md` 第四节）：分配记录树内部与根节点那两格、extent 上段叶 key 区间那一格，独立装置的变体臂只有这一种读法逐字节对上 crates；这是归因，不是在装置主读法上验过，主读法按这几格改完的重跑在后面。
- 判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节「上段规范形」：上段形状只由有条目 inode 的号集合定，写与删共用一条规则（两腿各自推出同一条规则；只升不降那条规则下同一个集合会出两种形状）。

**欠**：C211（节点落盘形态没有正文）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:463-480`（整段抄，未转述）**

```markdown
#### 已定项 15：write buffer 条目的三条硬要求

**定案**：

1. **排序会摧毁局部性而不是创造它 ⇒ 节点缓存必须是抗扫描的**（2Q / LRU-K 之类）。按叶排序之后每次 flush 就是一次覆盖整个叶子空间的顺序扫描，而 **LRU 对「反复的、大于缓存的顺序扫描」命中率恰好为 0**。**这是硬要求，不是建议。**
2. **条目形态必须是幂等完整值，不许是增量 Δ**。记账那一棵咬得最紧——「释放了 4096 字节」的自然表达就是 Δ，处置见已定项 1 的定案。本工程因此与 bcachefs 分道：其 accounting 树是 delta 形态（`disk_accounting_format.h` 原文「updates are _deltas_」）。
3. **条目必须自带序号（seq），这是格式要求**。flush 时按 key 排序**丢弃时间序**，而去重规则是「后者胜」⇒ **「后者」必须由条目自己携带**；用不稳定排序时同 key 条目的相对次序未定义，去重结果因此不确定。

**射程**：三条管的是 write buffer 这条路上的条目与缓存策略，宽度与落点在已定项 7、取值规则在已定项 10、与节点消息缓冲区的衔接在已定项 12。⚠️ 要求 2 与要求 3 是两条不同的要求：幂等管「重复应用不出错」，序号管「谁覆盖谁是确定的」。⚠️ **已知代价**（bcachefs 原文）：write buffer 的更新无序且最终一致，flush 时按 key 排序丢弃时间序，且这个 sort-merge-sweep **本质单线程**，在多线程负载下它的效率是关键路径。这句有两种读法，实测结果相反——**线程读法**（合并的 CPU 吞吐撑不住多线程负载）**不成立**：单线程合并吞吐按每 4 KiB 用户写 3 条条目算远超盘的带宽，按叶分区还可并行；**I/O 读法**（flush 的树插入 I/O 是关键路径）**成立，且是三项里最大的一项**。⇒ **关键路径上真正卡住的是 D9（加密） 的加密，不是合并**；⚠️ E17（write buffer 合并上限） 只量了合并这一段，不含树插入与 I/O。

**依据**：

- E7（离线索引 harness）：本工程这一侧的 io/op 在缓存占树 2% / 6% / 25% 三档上逐位相同，干预证实保持同叶聚合不变、只打乱组的处理顺序就能让 25% 缓存档的读次数下降，100% 缓存档打乱前后完全相同（阴性对照）——LRU 对「反复的、大于缓存的顺序扫描」命中率恰好为 0，所以要求 1 是硬要求；同一装置量出 flush 的树插入 I/O 折算下来远大于加密与合并，I/O 读法成立。
- E14（I-3.1 的判别力：幂等完整值 vs 增量 Δ）：增量 Δ 形态下 I-3.1（已分配统计对得上） 判别力归零，幂等完整值形态下全抓——要求 2 由它钉住。
- E17（write buffer 合并上限）：第一版漏了 `seq`，两条臂与真值当场对不上，逼出要求 3 的可判定形态；同一实验的单线程合并吞吐与按叶分区的并行倍数把「sort-merge-sweep 本质单线程」的线程读法判为不成立。

**欠**：C15（write buffer 混入 Δ）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:481-512`（整段抄，未转述）**

```markdown
#### 已定项 16：第一版 unlink：只立 deleted_inodes、从尾部分批、挂载里续做到空

**定案**：

1. **意图**：意图那次发布把 inode N 的记录写成 nlink = 0（size 不变），`deleted_inodes` 加 key = N（8 字节、无 value、不带对象出生代），往墓碑树开放容器追加一条墓碑（D18（块里携带什么信息） 已定项 24）；不立 logged_ops（K-U1 L4）。
2. **分批**：对任一 inode N，从尾部按数据单元个数分批，每批释放 N 尾部 K 个单元、size 改成「剩下单元数 × 净荷容量」，剩下单元数取 N 在 extent 树里的单元数（没有的取 0，只有最后一批）；K 产品取 `EXTENT_TREE_LOWER_LEAF_DATA_UNITS`（144，自由选取），测试由只供测试的旋钮压小；删除批只重写 N 被截掉那一片下段叶与它的祖先，加上段里 N 那一串（K-R3-1，extent 写法的新分支，要过 `.claude/rules/fs-design.md` 五条硬要求）；最后一批释放 N 剩下全部单元、删 N 的 inode 记录、删 `deleted_inodes` 那条 key、把 N 的条目从上段去掉（不写标签 0），同一次发布；只有这一条路，不设一次做完的快路。意图那次、每一批与续做的每一批各占一个事务号，等于本实例的下一个（与普通发布同一套编号）。
3. **建树**：`deleted_inodes` 在第一个文件版本那次发布零根注册；它是派生态：队列 = inode 树里 nlink = 0 的号集合（`research/prompts/m3-delete-r2-main-verification.md` 第三节 V7）。
4. **读者与写入口**：`InodeRecord` 的 nlink 由 `parse` 读、`to_bytes` 写结构里的值；现造记录的入口拒「nlink = 0」与「没有这条记录」；从意图提交起只读挂载、冷走读、模型读回答「没有文件」；写入口拒 nlink = 0 的 inode（`research/prompts/m3-delete-r2-main-verification.md` 第二节第 1 行、第三节 V6）。
5. **删空**：「有树、文件集合为空」是 `RebuiltVersion` / `PreviousVersion` / `PoolVersion` 的第三个成员，不借 `WithoutFile`；判定 = 树表有条目 ∧ inode 树零条目根（key 区间 `[0, 0]`，照已定项 6）∧ extent 树零条目根（key 区间按位置规定）∧ `deleted_inodes` 根指针置零；不看墓碑树、谱系树与退役树。删完之后只能做 `publish_new_inodes`（size 0）、写行与空发布（K-U5 X1）（`research/prompts/m3-delete-r3-main-verification.md` 第三节 G5）。删空的一版上覆盖写、顺序写、unlink 拒成已有的「没有文件版本可改」那一种拒绝（`UserChangeRefused::NoFileVersionToChange`）。删完第一个文件、别的文件还在的一版归 `WithFile`：inode 树还有任一记录或 extent 树还有条目的一版是带文件的一版，不管 inode 1 在不在；第三个成员只指文件集合真的空。
6. **续做**：`mount_writable` 在写行与暖机之后、交回之前把队列走到空；不进取号前预演；会话里的队列每次从现行那一版读、不缓存；崩溃注入加 `WritableMountPhase::Continuation`（`research/prompts/m3-delete-r2-main-verification.md` 第三节 V3）。会话里每一次用户改动（含 unlink、写、建 inode）先把现行那一版的 `deleted_inodes` 队列走空，再做它自己；挂载续做照旧；unlink 一次调用 = 走空 + 意图 + 全部批（与续做共用同一个走空）。会话走空时删除类发布（意图那次与分批的每一批）被落点那一判以空间拒，推一串抬 F、回收后再试；每一次被拒的删除类发布各算一次准入（一次准入最多 `PUBLISHES_PER_ADMISSION_AT_MOST` = 8 次发布，`crates/singlefs-core/src/mount.rs`）；推满仍放不下交回「抬 F 之后仍没空间」的拒绝（`UserChangeRefused::NoSpaceAfterRaisingTheFloor`），已发成的留在盘上。挂载续做走空时删除类发布被落点那一判以空间拒，同样推抬 F 再试，预算口径与会话走空相同；推满仍放不下时挂载照样交回可写、队列留着，之后会话里每次用户改动先走空、走不动报空间不够，正常卸载照常做成，靠的是 D16（发布语义） 已定项 1「正常卸载」那一行当场可用的那一截回收出来的槽（与 D16（发布语义） 已定项 1「准入」那一行同一定案）。任何一版里 `deleted_inodes` 至多一条 key；多于一个根装得下那一支拒成类型化错误、不是空间不够。
7. **幂等的判据五条**：① 计划相等；①b 同一份崩溃镜像各自重挂载续做同一批，两份盘逐字节相同（意图那次与每一批的改动计数是本次发布的 checkpoint_txg；意图那次的三个时间秒由调用方给、共一个值，各批照抄，续做的几批不另取时间）；② 删完再续做一个写都不发；③ 每个写点截断、重开续做到底，终态逻辑内容与不崩溃的终态相同、每个崩溃态 checker 全绿；④ 每个被判状态的记账标签 4、6 等于从这一版镜像现算的值。「逻辑内容」含 inode 每条（号、对象出生代、size、nlink）、extent 单元序号集与内容校验和、`deleted_inodes` key 集、墓碑记录多重集（含死亡 txg）、谱系记录多重集、标签 12 / 4 / 6、读者的答案；不含实例号、根与单元头里的 txg、改动计数、落点、单元头的诞生代号与写序、逐盘标签 1 / 2 / 5 / 10 / 11。另加 harness 层每种发布的重写角色集合钉绝对值（零轮）（`research/prompts/m3-delete-r2-main-verification.md` 第三节 V1；`research/prompts/m3-delete-r3-main-verification.md` 第三节 G2）。不同操作序列的终态不要求逻辑内容相同（例如不同删序的墓碑死亡 txg 不同），要求上段形状只由有条目 inode 的号集合定、每个崩溃态 checker 全绿。
8. **上段规范形**：extent 树上段形状只由有条目 inode 的号集合定，写与删共用一条规则：节点取各有条目 inode 的根路径（从它那片叶到根那一串）之并；根层级取罩得住最大号的最低一层；集合为空时取层级 0 第 0 格一个零条目根兼叶；非根节点不许零条目。一次发布里，进出集合或条目变了的号那一串上留在新形状里的节点与新出现的节点重写，别的照抄，上一版有而新形状没有的换下；长高那次旧根照抄成根之下的节点，降低那次上一版里降后层级的第 0 个节点照抄成根，旧根与脊线上方摘掉的节点换下（已定项 14 的 extent 树一条写同一个形状，`research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节「上段规范形」）。

**射程**：第一版按 inode 号删任一 inode、无目录、不带 truncate（用户 2026-09-28 定的是单文件、按 inode 号、无目录、不带 truncate，任一号由多文件删除的判决放开）；删完第一个文件之后（K-U5，记欠账）按 inode 号写数据只拒对象出生代晚于 inode 1 墓碑死亡 txg 的 inode，删之前就有的别的文件照常能写；取这个死亡 txg 只认这一版父链上的墓碑：「父链」照 I-9.18（删除队列里的号有谱系上的墓碑） 的定义，从这一版沿谱系记录往上走，只排掉被这条父链上的回退抛下的那几段 (T_old, T_rb) 开区间里的死亡 txg，不在父链上的谱系记录不排，再取最大；inode 1 的记录还在时不判；这一读法依赖墓碑第一版不回收（D18（块里携带什么信息） 已定项 24），墓碑回收落地时要换读法（C618（删完之后不能再建带数据的文件））；崩溃流接在 `crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs` 那条流后面 unlink 它的 3 单元文件（K-U6 F1），截断成 0 另立一条流。只改尾叶的 extent 分支、写入类发布按发布之后的标签判、重写角色集合钉绝对值被攻过零轮；第 6 条增补的会话里每次用户改动先走空、与任何一版至多一条 key，也被攻零轮。

**依据**：

- 无实验：数都在三方腿的副本原型上量（三份删除判决），入库装置「删除借保留量的环窗口」第一段已在套补丁表的副本上跑完（2026-10-03），第二段要在真实现的删除批上重跑、还没跑。
- 三方判决 `research/prompts/m3-delete-r1-main-verification.md`、`research/prompts/m3-delete-r2-main-verification.md`、`research/prompts/m3-delete-r3-main-verification.md`。
- 用户定案 2026-10-01：K-U1、K-U5、K-U6、K-R3-1（原话在变更史）。
- 判决 `research/prompts/m3-blanks-r1-main-verification.md` 第二节队列上界与接口那一行：回退到意图那一版会把号带回队列，只有「会话里每次用户改动先走空」与「回退之后续做」两个臂在放开用户动作的历史上队列长度与有队列时的写次数两项都守住。
- 用户定案 2026-10-01：会话里每一次用户改动先走空（岔路单 `m3-blanks-r1-forks.md` 的「会话里的续做点」一行，原话在变更史）。
- 用户定案 2026-10-02：会话走空被空间拒推抬 F 再试；2026-10-03：每一次被拒的删除类发布各算一次准入、挂载续做被空间拒同样推抬 F 再试、推满仍放不下照 D16（发布语义） 已定项 1 做成可写、删除类发布各占一个事务号（原话在变更史）。
- 代码轮判决 `research/prompts/m3-unlink-code-r1-main-verification.md` 第二节 K2 那一行（续做、走空与推抬 F）与第四节（挂载续做推满仍放不下的结局交用户）；实现：会话走空推抬 F 是 `crates/singlefs-core/src/mounted_session.rs` 的 `publish_the_queue_until_it_is_empty_raising_the_floor_for_space`，事务号是 `crates/singlefs-core/src/transaction.rs` 的 `publish_deletion_step`。
- 主 agent 2026-10-02 定 Q-L 照收（调度记录 `records/2026-10-01-删除与并发事务.md`「十六　批 8」）：删空的一版上覆盖写、顺序写、unlink 拒成已有的那一种拒绝。
- 用户定案 2026-10-03：「卸载必须做成」，续做推满仍放不下之后正常卸载靠 D16（发布语义） 已定项 1「正常卸载」那一行放开的那一截回收出来的槽做成，单盘幸存那一问取「甲 GA（推荐）」（原话在变更史）；设计轮判决 `research/prompts/m3-unmount-after-short-continuation-r1-main-verification.md`，代码轮第三轮判决 `research/prompts/m3-unlink-code-r3-main-verification.md` 第二节 n1 各行。
- 用户定案 2026-10-03：删除步算改动（「算改动，改实现」），时间字段取「乙 意图带时间、三格同改」（原话在变更史）；代码轮第三轮判决 `research/prompts/m3-unlink-code-r3-main-verification.md` 第二节 n3 那一行。
- 判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节「删任一号」「上段规范形」「删第一个文件」与第五节用户定案：删任一 inode 的各步、上段规范形、K-U5 取「β 时序级」（用户定案原话在变更史）。
- 代码轮判决 `research/prompts/m3-close-w5-code-r1-main-verification.md` 第二节 W4 与第三节第 1 条：β 取墓碑死亡 txg 只认这一版父链上的墓碑（攻方量过：删 inode 1、建 inode 3、回退到删之前、再删 inode 1、回退到建 inode 3 那一版，写 inode 3 做成，池级 checker 零红）。

**欠**：C618（删完之后不能再建带数据的文件）、C619（墓碑树根长高那次发布没有崩溃流）、C614（回退时墓碑树取错与删完对象无墓碑单镜像判不了）；C118（`deleted_inodes` 树的形态无落点） 还清。

```

**出处 `.claude/kb/decisions/15-format-freeze-policy.en.md:1-6`（整段抄，未转述）**

```markdown
## D15 (Format Freeze Policy) — Settled

[中文](15-格式冻结政策.md) · **English** · [日本語](15-フォーマット凍結ポリシー.ja.md)

D15 (Format Freeze Policy) governs two questions: **when it is permissible to state "format v1 frozen"**, and **what channel newly added items follow after freezing**. It is the common prerequisite for D11 (Index Nodes Message Buffers) / D12 (Target Media) / D14 (Dual-Track File Sizes and Lifetime) — the current value of the cost "format branch permanent" is zero, but infinite after freezing, and the true cost of those decisions is defined by it. Out of scope: what value each individual field should settle on belongs to its respective decision; before the first external user emerges, the format is soft, and that policy resides in `.claude/rules/format-evolution.md`; the semantics of each bit in `feature-bits.md` belongs to the decision claiming that bit, while D15 (Format Freeze Policy) only governs how bit numbers are allocated and whether they can be reclaimed.

```

**出处 `.claude/kb/decisions/15-format-freeze-policy.en.md:7-233`（整段抄，未转述）**

````markdown
### Settled Items

Items in D15 (Format Freeze Policy) have only a single set of numbers without reordering.

| # | Item | Settlement |
|---|---|---|
| 1 | **Specific handling of item 7 in pre-freeze checklist** | Item 7 does not write a list of decisions, but is written as a single predicate — each unsettled item across the repository declares whether it "touches the format"; if a freeze request intersects with it, judge red; default padding width is 0 **Status: Settled.** **Progress: No code object.** |
| 2 | **Whether the tree table is an open list** | Yes, made an open list from day-1 **Status: Settled.** **Progress: Implemented.** |
| 3 | **Source of `spec_hash`** | No switching. Invariably computed against `format-spec/<component>.toml`; that toml file is a complete projection of the kb field table, including row ordering and non-scalar on-disk semantic rules **Status: Settled.** **Progress: Unimplemented.** |
| 4 | **Bit allocation registry for feature bits** | Registry resides in settled item 4; incompat bit 1 = first pure-SSD layout line (the version with F and unmount mark), set upon mkfs, bit 0 retired; compat_ro and compat have zero bits allocated **Status: Settled.** **Progress: Implemented.** |
| 5 | **What the action of freezing constitutes** | Freezing = committing a file `.claude/kb/format-frozen/<component>.toml`; humans only decide whether to initiate it, while the machine decides whether the request holds **Status: Settled.** **Progress: Unimplemented.** |
| 6 | **Checklist that must be completed before freeze** | Seven prerequisites, re-judged upon each freeze; values for each item are taken fresh from the settlement line of the corresponding item, no manual copying allowed **Status: Settled.** **Progress: No code object.** |
| 7 | **How many layers freezing is split into, and what lies outside the four layers** | The four layers have strict dependency ordering (system configuration and block header, pointer field layout, key encoding, index node internal layout), plus three independent freeze components **Status: Settled.** **Progress: No code object.** |
| 8 | **What freedoms are lost after freezing** | Width, existence, and meaning of frozen fields can no longer change, only additions permitted; checker judgments on frozen portions also become compatibility contracts **Status: Settled.** **Progress: No code object.** |
| 9 | **Post-freeze change judgment table** | Nine categories of changes judged for compat / compat_ro / incompat and whether to bump, with the criterion being what old readers will do **Status: Settled.** **Progress: No code object.** |
| 10 | **Bit count, accounting, and allocation rights of feature bits** | Three bitmaps of 256 bits each stored in system configuration; allocation right is "first merged, first claimed", bit numbers strictly monotonic; once used, bits may never be reclaimed **Status: Settled.** **Progress: Implemented.** |
| 11 | **Two gates that can go red paired with freezing** | Format freeze contract (`spec_hash` comparison) and backward compatibility regression (golden images); image bodies do not enter git, checklist is append-only **Status: Settled.** **Progress: Unimplemented.** |

#### Settled Item 1: Target of Item 7 is Unsettled Items, Not a Decision List

**Settlement**: Three rules.

1. **The target is every still unsettled item across the whole repository, not a decision list.** Each unsettled item must carry a verdict statement — **touches the format or not**: whether its answer will modify any on-disk bytes or alter the **interpretation measurement basis** of existing bytes (the measurement basis aspect reflects the pattern of C97 (changing field semantics without altering width produces no red gate anywhere in repository): width does not change by a single byte, but semantics change). Three possible values: **touches / does not touch / boundary unsettled (treated as touches)**; **unwritten is treated as touches**.
2. **Judgment**: When a freeze request is initiated, any item judged as "touches" or not yet judged triggers a red gate. To freeze in batches, the request must **list each item by name and explicitly accept the risk** (pattern following settled item 7 "risks of batch freezing must be explicitly accepted"); **omitting any item triggers a red gate**.
3. **Default padding width is 0**, with the burden of proof on the party proposing non-zero width; it must pass the format branching criterion in `.claude/rules/fs-design.md` (verbatim: "whether a format branch is worthwhile depends on whether it can turn 'used incorrectly' into 'fails to mount'").

Framing determines interpretation (`.claude/singlefs-ai-sop/skills/decide/SKILL.md` hard requirement 7: phrasing of tentative items must have only one reading): instead of "pinning values across six cells item by item" (content of each cell lives in its respective decision; duplicating it creates dual records of the same fact), we adopt "defining the judgment rules for item 7"; item 7 is **re-judged upon each freeze**, because the checklist is a pre-commit check triggered by commits and runs with every freeze request.

**Scope**: Default padding width 0 governs **reserving placeholders for features settled as not done**; it does not govern **general reserved areas** within structures — general reserved areas are governed by the first row of settled item 9 judgment table (new field in reserved area / TLV tail => compat without bump) and C102 (inode header lacks reserved space making field additions non-compat), which have different criteria; conflating the two would deduce "reserve nothing anywhere", whereas C102 (inode header lacks reserved space making field additions non-compat) is repaying an owed check in the opposite direction. What elevates to a rule is solely the procedure of "default 0 + inverted burden of proof"; any entry reaching the branch "v1 does not include this feature" must pass the format branching criterion again on its own. Predicates are not partitioned by layer: exiting freeze is judged by structure, not by layer (scope of settled item 7), and membership is incomplete — the three independent freeze components in settled item 7 do not belong to any of the four layers => layer-partitioned predicates are undefined on them; partitioning by the binary predicate "touches format or not" avoids both defects and assumes no independent adjudicability between layers. The cell for D26 (Background Compaction and Placement Reclamation) settled item 4 remains owned by D26 (Background Compaction and Placement Reclamation) itself; D15 (Format Freeze Policy) has no authority to answer on its behalf. Judgment inputs are always fetched fresh and never cached here: `grep -c '未定\*\*' .claude/kb/decisions.md` counts how many unsettled items remain across the repository (item list line tails are currently formatted as "—— **未定** · 未实现", no longer ending with "未定**"), and `bash .claude/gate.d/doc-decisions.sh --check decision-items-sync` judges whether that list is a faithful projection of the body.

**Evidence**: No experiment: what is settled is the target and default values of gate judgment rules, with no quantifiable metric. Supporting it are two facts — list patterns leak entries (D22 (Synthesizing Unit Atomicity) settled item 9 is the **field table of system configuration**, and system configuration is Layer 1; D19 (Block Pointer Structure & Width Budget) settled item 6 is the **field table of central mapping entries**, residing in Layer 2 — neither was on that six-decision list, so the list had zero say when freezing those two layers), and relying on a "format-level" tag fails to catch anything (registry location of IDs is in the title line of each decision body, not on the index page; `grep -rn 格式级 .claude/gate.d/ .claude/scripts/` yields zero hits => zero objects, zero implementation). Settled by main agent based on three-way verification; an invalid argument is recorded in decisions history to avoid reusing it.

**Owed checks**: C62 (settlement values of items branch in kb); C102 (inode header lacks reserved space making field additions non-compat).

#### Settled Item 2: Tree Table Made an Open List from Day-1

**Settlement**: The tree table is made an open list from day-1. Evidence lies in the judgment table of settled item 9: under an open list, "adding a keyspace" means old readers merely **do not see the new tree** (compat grade, graceful degradation); hardcoding tree counts means old readers **cannot read the tree table** (incompat, immediate breakage). **Open lists strictly dominate**, with the only cost being an extra table length field plus a variable-length table section in the tree table unit — identical to the rationale in settled item 10 "reserves should lean generous". => Adding keyspaces is an explicitly stated expansion objective of this project, not an accident, so that row in the judgment table is pinned as **compat**.

**Scope**: What is settled is "tree table is an open list", regardless of where tree tables reside — tree tables reside in the root record indirection layer (D22 (Synthesizing Unit Atomicity) settled item 7 verbatim "tree roots invariably reside in tree table units, never flattened into root slots", settled item 12 settles "system configuration does not carry its own tree table unit pointer"; D21 (Boundary Between Authoritative and Derived States) groups root record + tree table unit + instance table unit together into "root"). It **must be incorporated when Layer 1 freezes**, and cannot wait until adding a tree reveals it was omitted; freeze layer membership for the tree table unit itself remains owed.

**Evidence**:

- E142 (new-pool file creation dry run): The tree table unit written during new-pool file creation declares entries by count (0 entries in mkfs version, 11 entries in transaction version, representing values before adding retired trees; 12 entries in transaction version after adding retired trees), table length is data rather than compile-time constant — the open list form holds on written bytes.
- User settlement, verbatim text in decisions history.

**Owed checks**: C246 (tree table unit lacks freeze layer membership).

#### Settled Item 3: `spec_hash` Invariably Computed from Complete Projection of kb Field Tables, Never Switched to Rust Structs

**Settlement**: `spec_hash` is invariably computed against `format-spec/<component>.toml`. That toml file is a **complete projection of the kb field table** — containing **row ordering** (on-disk sequence) and **non-scalar on-disk semantic rules** (merge rules for component 2 in settled item 7 are encoded as data), not merely the collection of registered scalar constants. What is added once code exists is **not changing the hash source**, but two gates: (1) **equality check** between spec and code (comparing both width and ordering); (2) golden image regression (settled item 11 rule 2).

Four reasons:

1. **Directionality**. D13 (Verification Route) settled item 5 verbatim: "**checker and implementation share only one artifact: a constants module generated from kb field tables**, with the generator reading from kb and both sides only consuming, no manual edits permitted by anyone" => authority resides in kb, while Rust structs are **downstream consumers**; computing hashes against structs anchors the fingerprint of a permanent contract on a derivative artifact.
2. **Hash is a change detector, not an equality check between two artifacts.** Computing hashes on structs cannot catch "structs inconsistent with kb"; it alarms only when structs **change**; what genuinely catches that class of error is an equality check, which requires the spec side to exist => switching across not only gains nothing extra, but tears down the opposite side required for comparison.
3. **Language binding**. D17 (Implementation Layering & Third-Party Pipeline) settled item 3 verbatim: "Verdicts must be independently verifiable by third parties, otherwise 'conformance' degrades into 'we claim you conform'". Anchoring fingerprints on Rust source text prevents third-party pipelines from recomputing them.
4. **Feasibility**. Spec files for the three components can be drafted immediately, allowing hash mechanisms to take effect long before the first freeze; whereas the freeze moment for component 1 in settled item 7 is the first image writing a Type 2 container => switching would be forced into the identical commit as the first real freeze, with zero burn-in period.

**Scope**: Projecting only scalars into the spec is insufficient; row ordering must be included, and equality checks must compare sequence — the unit header in [layout/01-first-txn.md](../layout/01-first-txn.md) lists six adjacent fields of width 8 in row order (the 5-tuple's tree ID / object ID / object birth txg / anchor offset, plus birth generation and fsid); transposing any two during serialization leaves all scalar constants and `DATA_UNIT_HEADER_BYTES` unchanged while on-disk bytes become completely misaligned, falling into the "changing field semantics" row of settled item 9 judgment table. The boundary in D13 (Verification Route) settled item 5 stating "generator emits only scalar values" governs shared artifacts between checker and implementation, whereas `format-spec/<component>.toml` is a **gate artifact** with zero runtime consumers => outside that boundary's scope, and a field table with row ordering contains neither branching nor arithmetic. **Explicit statement of what is uncaught**: hashes do not establish binding between spec and actually written bytes — if the first golden image was itself written by faulty code, regression will treat that faulty version as baseline; what stops that is checker as an independent parser and the conformance suite judge, not `spec_hash`. Specs can only be written manually, with no existing mechanism projecting kb field tables to toml. The **magnitude of risk** in batch freezing where "early frozen layers omit reserved fields" cannot be quantified; it depends on yet non-existent concrete layouts in D8 (Core Index Structure) / D11 (Index Nodes Message Buffers), so only the stop-loss rule in settled item 7 can be provided, while residual risk remains unquantifiable.

**Evidence**:

- E142 (new-pool file creation dry run): Six adjacent fields of width 8 in the unit header are arranged sequentially in written bytes; swapping any two while preserving all scalars is constructible — projecting only scalars in spec cannot safeguard field ordering.
- Three-way verification: the reverse deduction leg eliminated the branch "spec projects only scalars" according to pre-execution failed clauses; settlement adopts the tightened form (spec must include row ordering), which demands one extra guarantee and subtracts nothing. Settled by main agent.

**Owed checks**: C227 (golden image baseline written by its own code version); C226 (spec manually copied without generation from field tables); C119 (freeze components lack spec files).

#### Settled Item 4: Bit Allocation Registry for Feature Bits

**Settlement**: Three bitmaps of 256 bits each (three bitmaps of settled item 10); the sole registry for bit numbers is the table in this section. "Refuse mount on unrecognized bits" must apply even to the default layout, so the first layout line claims a bit from mkfs onward — avoiding the alternative "first line claims no bit" (which would allow readers unfamiliar with SSD layout to mount).

| bitmap | bit | meaning | provenance |
|---|---|---|---|
| incompat | 0 | First pure-SSD layout line | D12 (Target Media) settled item 5; D12 (Target Media) settled item 1 "one incompat bit per layout"; retired: the version with rollback witness and system configuration lacking F, readers reject as unrecognized bit (user 2026-09-26) |
| incompat | 1 | First pure-SSD layout line (system configuration carries rollback floor F, root record carries unmount mark, lacks rollback witness and rollback row) | D12 (Target Media) settled item 5; user 2026-09-26 settled "switch incompat bit, cannot mount" |
| incompat | 2..255 | Unallocated; append one bit for each new layout line | D12 (Target Media) settled item 1 |
| compat_ro | 0..255 | Unallocated | — |
| compat | 0..255 | Unallocated | — |

**Scope**: Governs only how bit numbers are split and the assignment of the first bit; semantic definitions of each bit belong to the decision claiming that bit. Bytes written in new-pool file creation: the 96 bytes of feature bits in system configuration change from all zeros to incompat bit 1 set to 1; bitmap is little-endian, bit 1 is the 2nd bit counting from the least significant bit of the first byte (D22 (Synthesizing Unit Atomicity) settled item 13). This item did not go through three-way verification; both main agent recommendation and user choice can be overturned.

**Evidence**:

- E142 (new-pool file creation dry run): The 96 bytes of feature bits in system configuration are written per this item, closing its blank G12, and [layout/01-first-txn.md](../layout/01-first-txn.md) updated synchronously.
- User settlement, verbatim text in decisions history; pre-settlement cell text also in decisions history, including its same-day entry "alters bytes: no" — that statement has been overturned by this settlement.
- User settlement 2026-09-26: bit 0 retired, layout identity switched to bit 1, old images (with rollback witness, system configuration without F) cannot mount (three-way verdict `research/prompts/m2-rollback-forward-r3-main-verification.md` section 2 K7: "reject writable mount if witness table non-empty" eliminated; read-only recovery of 13 corruption depths had 4 falling into rolled-back versions; altering system configuration format version number cannot block it as current readers do not verify version numbers), verbatim in decisions history; no experiment: old readers failing to mount on new bit measured on prototype, new readers rejecting old bit is derived; "altering system configuration format version number cannot block it as current readers do not verify version numbers" is obsolete: starting 2026-09-27, readers refuse to mount the whole pool on any system configuration format version other than 1 (settled 2026-10-21, D22 (Synthesizing Unit Atomicity) settled item 9).

**Owed checks**: C183 (one incompat bit per layout while on-disk bytes show no difference).

#### Settled Item 5: Freezing is a Commit Arbitrated by Gates, Not a Verbal Statement

**Settlement**: The **action** is committing `.claude/kb/format-frozen/<component>.toml` to the repository (containing `version`, `frozen_at_commit`, `spec_hash`). **Who judges**: not a human "feeling ready" — once the gate detects that the current diff touches this directory, it switches automatically to a stricter suite of pre-checks (checklist in settled item 6), permitting the commit to pass only if all are satisfied. **Humans only decide "whether to initiate a freeze now", while machines decide "whether this freeze request holds"**, isomorphic with `.claude/singlefs-ai-sop/rules/show-me-test.md` "do not differentiate by source, differentiate only by evidence".

**Scope**: That stage does not run ordinarily (running only when the diff touches that directory), so attaching it never slows daily commits; **it is destined to trigger 0 times for long periods, which is not a flaw but correct behavior** — sharing the same honesty as gates explicitly listing unimplemented stages. ⚠️ It is commit-triggered, whereas the freeze moment for component 1 in settled item 7 is "the first image writing a Type 2 container", which is an execution rather than a commit => pre-freeze checklist has no execution path for component 1, and the gate will never ignite for it.

**Evidence**: No experiment: settles gate trigger timing and arbitration ownership, with no quantifiable metric; format aligns with existing "project local stage" mounting points in shared `gate.sh`. Settled by main agent.

**Owed checks**: C225 (component freeze moment bypasses gate triggers).

#### Settled Item 6: Checklist That Must Be Completed Before Freeze

**Settlement**: Seven prerequisites, judged item by item upon freeze request initiation, re-judged upon each freeze.

| # | Item | What happens if freeze proceeds without it |
|---|---|---|
| 1 | For the component to freeze, its field table in kb has no row marked "unsettled", "envisioned", "semi-settled", or "placeholder" | Freezing an evolving design; subsequent answers to open items will be retroactively claimed as part of v1. ⚠️ This and item 7 are two distinct rulers: item 1 measures completeness of **already written tables**, while item 7 measures whether **unwritten items** touch it |
| 2 | I-1 (block self-describing) and I-2 (checksum) category checkers are implemented and pass green | "Checker is specification" — without checkers, "what v1 is" exists only in documents, and subsequent modifications cannot be mechanically adjudicated |
| 3 | Four reserved fields of D9 (Encryption) settled item 12 can be asserted in checkers as "present with width matching reservation" | Omitting even one forces incompat patching post-freeze, and existing old images permanently lack this field |
| 4 | I-3.1 (allocated accounting matches) is implemented and passed N>=5 rounds all green | The sole verification method for accounting correctness is this comparison; without it, whether frozen ledger fields are correctly designed remains completely untested |
| 5 | Model differential and crash-point replay each pass at least one round, **running against the exact format fingerprint about to be frozen** | Otherwise v1 describes a format never touched by automated verification |
| 6 | C4 (project rule checklists inconsistent) and C8 (gate scope cannot be evaluated) in [checks-owed.md](../checks-owed.md) are implemented | Post-freeze rule divergence or gates slowing down without measurement leads to incidents in higher-risk phases |
| 7 | Every still unsettled item across repository carries the verdict "touches format or not", and none judges "touches" or remains unjudged — unless the freeze request explicitly enumerates them and accepts the risk | This chain mutually defines itself; silently freezing with "unsettled" items equates to pretending decisions are already made. Judgment rules in settled item 1 |

⚠️ **Reviewing every item in the checklist requires taking values fresh from the settlement line of corresponding items; manual copying is forbidden.** Freezing is irreversible, while copying an erroneous number looks nowhere awkward in prose => **open the settlement line of that item verbatim during review**; machine-checked forms and prerequisites are tracked in C62 (settlement values of items branch in kb); until that check is implemented, this step relies on humans.

**Executable forms of items 1 and 7**: Item 1 scans the field table of the component to freeze — the table in [layout/01-first-txn.md](../layout/01-first-txn.md) listing fields sequentially by on-disk order is its ready form; scanning its **"width" and "status" columns**, encountering **any** of `未定` / `预想` / `半定` / `占位` triggers a red gate. Item 7 scans the "touches format or not" verdict in unsettled items across the repository. It splits into two halves: the **presence check** is the second ruler of the blocking-verdict cell in `.claude/gate.d/doc-decisions.sh`, sharing parsing, authoritative lists, independent reporting, and distinct remedies with the first ruler (modifying new-pool file creation bytes), running regularly; the **rejection check** (judging "touches" or unwritten triggers red gate unless listed explicitly with accepted risk) happens upon freeze request initiation, whose prerequisite is C225 (component freeze moment bypasses gate triggers) — freeze requests have no execution path for component 1. Neither item **creates a duplicate checklist** — `.claude/singlefs-ai-sop/rules/kb-discipline.md`: only one authoritative record permitted per fact.

**Scope**: Wordlist for item 1 is written into clauses without leaving script authors to guess — terms used in byte tables today are "envisioned" and "semi-settled" (the extent tree node row verbatim "envisioned 162 = 86 + 2 x 24 + 28", status column "semi-settled"; whether it carries key ranges is unsettled, 162 with, 114 without, a 48-byte difference shifting the entry start offset), precisely the evolving design item 1 intends to intercept; a check scanning only "unsettled" would pass it, allowing a Layer 4 freeze request to retroactively affirm a 48-byte unresolved measurement basis into v1. The checklist validates whether prerequisites are complete, not whether the content of any item is correct.

**Evidence**:

- E142 (new-pool file creation dry run): The field table item 1 scans is the very table written and checked row by row; the words "envisioned" and "semi-settled" in "width" and "status" columns possess concrete objects.
- Settled by main agent (narrowing of item 1 vocabulary in 2026 detailed in decisions history); judgment rules for item 7 reside in settled item 1.

**Owed checks**: C62 (settlement values of items branch in kb); C8 (gate scope cannot be evaluated) (the other half of checklist item 6, C4 (project rule checklists inconsistent), has been repaid).

#### Settled Item 7: Freezing Split into Four Layers, Plus Three Independent Freeze Components Outside Layers

**Settlement**: The four layers have strict dependency ordering.

```
Layer 1: System configuration + block header (self address / generation / tree ID / fsid / magic)
    ↓
Layer 2: Pointer field layout (D4 checksum + D9 MAC / nonce / ciphertext checksum)
    ↓
Layer 3: Key encoding for each keyspace (D8 locality_id / inode / offset)
    ↓
Layer 4: Index node internal layout (node size rules, bset format, D11 message buffers)
```

Rationale for dependency direction: Layer 1 is the physical premise that "metadata blocks must be self-describing", upon which all subsequent layers rest; Layer 2 is pinned by D9 (Encryption) MAC / nonce / ciphertext checksum being "impossible to insert after the fact", and must precede tree structures relying on it; once key encodings in Layer 3 are settled, organizing these keys within nodes has a foundation for discussion.

**Risks of batch freezing must be explicitly accepted**: If an early frozen layer omits fields, subsequent work must patch via incompat, equivalent to burning design margins prematurely. There is only one stop-loss rule: **Layers 1 and 2 must include all four D9 (Encryption) reserved fields prior to freeze**, as they happen to be fields known to be needed eventually, which cannot be deferred to Layers 3 or 4.

**Independent freeze components outside the four layers**: Three classes of permanent bytes do not belong to any of the four layers; each forms an independent component, with `spec_hash` in C9 (covertly modifying format after freeze) recorded per component, keeping the four layers untouched:

- 1. **Payload record format for packed record units**, registered by packed record type: Type 2's 140-byte inode record table (D8 (Core Index Structure) settled item 6), freeze moment being the first image writing a Type 2 container; Type 1 (width 40, D18 (Information Carried in a Block) settled item 24) is mutually independent, not frozen together with Type 2.
- 2. **Statistics registry** (tag value + dimension + merge rules, D5 (Snapshot and Space Accounting Mechanism) settled item 3 / 4) — merge rules are on-disk semantics; two readers applying different rules to the same bytes will compute divergent values; C71 (silently ignoring unrecognized statistics tags) shields only unrecognized tags, not recognizing tags while applying wrong rules.
- 3. **journal record format** (record header, named items, payload checksum coverage, transaction boundary): pre-freeze requirements no earlier than Layer 2 and placement granularity; freeze moment is the commit submitting its toml file; new record formats follow "new record type + system configuration log-incompat bit", full text in D16 (Publish Semantics) settled item 3.

**Scope**: The four-layer diagram is not rearranged: D21 (Boundary Between Authoritative and Derived States) settled item 10 changed status classification to per-structure judgments; in Layer 4, Code 2 nodes of the accounting tree are authoritative state while internal nodes of other trees are derived state, so no layer is pure; thus the unit of exiting freeze is a structure rather than a layer (placement of each structure, its state category, and whether it exits freeze are registered in `.claude/kb/freeze-layer-membership.md`, checked by gate doc-decisions freeze-layer-membership cell); spec files for the three components do not exist; freeze moment for component 1 is an execution rather than a commit, so the gate in settled item 5 does not ignite for it. No clauses yet exist for which layer tree table units and instance table units belong to.

**Evidence**: No experiment: layering and dependency ordering are derived from sorting "which elements cannot be retrofitted later", with no quantifiable metric; target of the stop-loss rule (four reserved fields of D9 (Encryption)) resides in D9 (Encryption) settled item 12. Settled by main agent and user, verbatim in decisions history.

**Owed checks**: C119 (freeze components lack spec files); C225 (component freeze moment bypasses gate triggers); C246 (tree table unit lacks freeze layer membership).

#### Settled Item 8: What Freedoms Are Lost After Freezing

**Settlement**:

| Freedoms during soft format phase | What they become after freezing |
|---|---|
| Tearing down and rebuilding structures | Width / existence / meaning of frozen fields can no longer change; additions only, retired fields follow "retain read support, cease writing" |
| Correcting text directly upon finding flawed designs without compatibility | Can still be overturned at decision record level, but **on-disk byte compatibility transitions from zero cost to non-zero** |
| mkfs automatic selection rules can switch algorithms anytime | Selection rules for frozen components are also pinned; changing algorithms causes default discrepancies between old and new images, requiring a new format version number |
| Modifying formats directly upon disappointing experiment outcomes | New policies must be introduced via new feature bits, retaining interpretability of old values |
| Checker freely modified alongside formats | **For frozen portions, checker judgments become de facto compatibility contracts** — silently altering checker interpretations of frozen fields is as perilous as silently altering formats |

**Scope**: Only states which actions cease to be valid post-freeze, without adjudicating specific change grades — which belongs to settled item 9. Before the first external user emerges, format remains soft, and none of this table takes effect prior to that point (`.claude/rules/format-evolution.md`).

**Evidence**: No experiment: itemized expansion of the constraint "external permanent commitments cannot be altered", with no quantifiable metric. The final row (checker judgments become contracts) shares provenance with "checker is specification" in D13 (Verification Route). Settled by main agent.

**Owed checks**: None.

#### Settled Item 9: Post-Freeze Change Judgment Table

**Settlement**:

| Change type | Old reader behavior | Grade | Bump | Rationale |
|---|---|---|---|---|
| Adding fields in reserved space / TLV tail | Ignored, continues normal operation | compat | No | Pure addition without semantic dependency; safety net of "ignoring unfamiliar fields" suffices |
| Adding fields whose omission causes old readers to miscompute **another existing field** | **Miscomputes, potentially corrupting data** | incompat | Yes | Once semantics couple, safety nets fail |
| Expanding existing field width | Parsed with old width, out-of-bounds read / misalignment | incompat | Yes | Width is part of layout; offset calculations in old code break completely |
| Changing existing field meaning | Interpreted with old meaning, producing erroneous behavior **without error** | incompat, with **old position abandoned without reuse** | Yes | Reusing positions leaves old and new readers unable to distinguish, the most dangerous class. Two examples (D18 (Information Carried in a Block) settled item 11): adding a unit class (registry codes 4–15) alters the interpretation measurement basis of class identity segments; giving meaning to the constant-0 flag bit at common prefix offset 7 (old readers reject non-zero bits) |
| Adding record types (new key types) | Read-only traversals may misjudge; compaction / write may delete misunderstood records | compat_ro | Yes | **"Read-only safety" and "writable safety" are independent guarantees** that must be judged separately. ⚠️ Packed record types inside packed record units **do not follow this row** (D18 (Information Carried in a Block) settled item 11): skipping a tombstone or inode record alters read answers, defaulting to incompat; only types proven at registration that "skipping changes no read answer" demote to compat_ro, and compat_ro readers must not execute scan-rebuilds |
| Adding a keyspace | If tree table is open list day-1: simply does not see new tree; if tree count hardcoded: fails to read tree table | **compat** (settled item 2 makes tree table an open list day-1). Reserving open tree table -> compat; omitted -> incompat | Yes | **"Whether tree table is expandable" must be settled at freeze**, cannot wait until adding trees reveals omission |
| Changing sorting order | Binary search / traversal results incorrect **without error** | incompat | Yes | Sort order is index identity; reordering log-structured nodes directly relies on its invariance |
| Changing checksum / MAC algorithm | D9 (Encryption) algorithm type field reserved -> unfamiliar types fail explicitly | Reserved -> compat_ro; unreserved -> incompat | Yes | Item 4 of D9 (Encryption) settled item 12 is precisely the channel prepared for this change |
| Retiring a field | Unaffected, old readers continue interpreting old way | compat, with **parsing code preserved permanently** | No | Any field committed to disk must remain interpretable as long as a single old image exists |

**Scope**: Judges "what old readers will do", not "whether this change ought to be made". The first row (additions in reserved space are compat) holds only where real reserved space exists; inode headers carry only 2 bytes of reserved space, accommodating few fields.

**Evidence**:

- E59 (can buffered messages be recomputed solely from units): Skipping a tombstone record causes deleted keys to fully resurrect, resurrection count precisely matching deletion count — the quantified basis for "packed record types do not follow compat_ro row, defaulting to incompat".
- Settled by main agent, row-by-row rationales documented in table.

**Owed checks**: C102 (inode header lacks reserved space making field additions non-compat); C97 (changing field semantics without altering width produces no red gate anywhere in repository).

#### Settled Item 10: Bit Count, Accounting, and Allocation Rights of Feature Bits

**Settlement**:

- **Bit count**: Three bitmaps of **256 bits** each (`u64[4]`), stored in system configuration. Bits are far cheaper than field widths (3x32 bytes negligible for system configuration), **reserves should lean generous**.
- **Accounting**: `.claude/kb/feature-bits.md` lists `bit | category | name | introduced version | introduced commit | status | one-line semantics`; the sole registry for bit numbers is the table in settled item 4.
- **Allocation right is "first merged, first claimed", not an approval hierarchy**. Gate enforcement: bit numbers strictly monotonic, no gap-skipping reservations, no code referencing non-existent bit numbers — mechanical arbitration.
- **Once used, bits may never be reclaimed.** Rationale is **stricter** than non-recycling of numbers in [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md): numbers are retrieval keys whose reuse misleads humans or models; **feature bits are physical bits that existed on disk**; reallocating them to a new feature means an old image mounted on new code misinterprets the bit as "new feature X enabled" — **the file system would perform real incorrect actions** (e.g., treating non-existent encryption characteristics as present and searching for non-existent keys). Retired bits are marked "meaning permanently locked to semantics of last use prior to retirement", and never assigned to new features.

**Scope**: Governs how bits are divided, recorded, and whether they can be reclaimed; does not govern semantics of individual bits (belonging to decisions claiming them).

**Evidence**: No experiment: bit counts and non-reclamation are deduced from the physical fact that "old images will be read by new code", with no quantifiable metric; 256-bit space overhead is negligible for system configuration, so this cell does not optimize for frugality. Settled by main agent.

**Owed checks**: C183 (one incompat bit per layout while on-disk bytes show no difference).

#### Settled Item 11: Two Gates That Can Go Red Paired with Freezing

**Settlement**:

1. **Format freeze contract**: Record `spec_hash` for each frozen component; gate recomputes and compares; divergence requires current diff to **simultaneously** satisfy "`feature-bits.md` has new rows or `version` is explicitly altered" and "corresponding entry in `decisions.md` modified in same commit". Missing either triggers a red gate.
2. **Backward compatibility regression (golden images)**: Whenever a component version changes, generate and archive an image using **code of that era**; gate reads all historical images using **current code**, executes checker, and compares expected content digests. Image bodies **do not enter git** (`.claude/singlefs-ai-sop/rules/command-safety.md`: test images must not reside in repository); repository commits only the manifest `.claude/kb/golden-images.toml`; images are placed in `${SINGLEFS_GOLDEN_DIR:-$HOME/.cache/singlefs-golden}`. The manifest is **append-only** — deleting regression for any version equates to silently abandoning compatibility commitments, requiring formal decision records.

**Scope**: Neither is yet implemented; numbers are in checks-owed table. Rule 2 compares only "whether today's code understands historical images", not "whether the original image was written correctly"; Rule 1 anchors on declarations rather than bytes; neither shields against "the initial golden image itself being produced by erroneous code".

**Evidence**: No experiment: settles what two gate checks should judge; discrimination self-proof awaits implementation; failure modes they guard against (covertly modifying formats, quietly abandoning compatibility commitments) are tracked in C9 (covertly modifying format after freeze) and C227 (golden image baseline written by its own code version). Settled by main agent.

**Owed checks**: C9 (covertly modifying format after freeze); C227 (golden image baseline written by its own code version).

````

**出处 `.claude/kb/decisions/15-format-freeze-policy.en.md:234-237`（整段抄，未转述）**

```markdown
## Historical Versions

Historical entries for D15 (Format Freeze Policy) are centralized in [decisions-history.md](../decisions-history.md).

```

**出处 `.claude/kb/decisions/15-フォーマット凍結ポリシー.ja.md:1-6`（整段抄，未转述）**

```markdown
## D15 (フォーマット凍結ポリシー) — 確定

[中文](15-格式冻结政策.md) · [English](15-format-freeze-policy.en.md) · **日本語**

D15 (フォーマット凍結ポリシー) は二つの問題を所管する：**いつ「format v1 frozen」と宣言できるか**、および**凍結後に新要素を追加する際にどの流路を経由するか**。本決定は D11 (インデックスノード内のメッセージバッファ要否) / D12 (対象メディア) / D14 (大小ファイル・永続/一時デュアルトラック) に共通する前提である——「フォーマット分岐が恒久化する」というコストの現在価値はゼロであるが、凍結後は無限大となるため、これら各決定の真の代償は本決定により定義される。対象外とする事項：各フィールド自体に定めるべき具体的な値は各決定の所管に属する；最初の外部ユーザーが現れるまでフォーマットは流動的であり、その運用方針は `.claude/rules/format-evolution.md` にある；`feature-bits.md` における各ビットのセマンティクスは当該ビットを要求する決定に属し、D15 (フォーマット凍結ポリシー) はビット番号の割り振り方針と回収可否のみを所管する。

```

**出处 `.claude/kb/decisions/15-フォーマット凍結ポリシー.ja.md:7-233`（整段抄，未转述）**

````markdown
### 確定項目

D15 (フォーマット凍結ポリシー) の項目番号体系は単一であり、番号の振り直しは行わない。

| # | 項目 | 確定内容 |
|---|---|---|
| 1 | **凍結前チェックリスト第 7 項の具体的処置** | 第 7 項は決定リストを列挙せず、単一の判定式とする——リポジトリ内の未確定項目ごとに「フォーマット変更要否」を自ら宣言し、凍結要求がそれと交差すれば赤判定；デフォルトパディング幅は 0 **状態：確定。** **進捗：対象なし。** |
| 2 | **ツリーテーブルは開放リストか否か** | 開放リストとする、day-1 から開放リストとして実装 **状態：確定。** **進捗：実装済み。** |
| 3 | **`spec_hash` の算出元** | 切り替えない。常に `format-spec/<コンポーネント>.toml` に対して算出し、当該 toml は行順および非スカラーのディスク上セマンティクス規則を含む kb フィールド表の完全射影とする **状態：確定。** **進捗：未実装。** |
| 4 | **feature bit のビット割り当て登録表** | 登録表は確定項目 4 に配置；incompat ビット 1 = 最初の純 SSD レイアウト系統（F およびアンマウント記号を含む版）、mkfs 時点から設定、ビット 0 は廃止；compat_ro、compat は 1 ビットも割り当てない **状態：確定。** **進捗：実装済み。** |
| 5 | **凍結という動作の定義** | 凍結 = リポジトリに `.claude/kb/format-frozen/<コンポーネント>.toml` をコミットすること；人間は開始するか否かのみを決定し、機械が要求の成立可否を判定する **状態：確定。** **進捗：未実装。** |
| 6 | **凍結前に完了必須のチェックリスト** | 7 項目の前提条件とし、凍結ごとに再判定；各項目の値は対応する小項目の確定行からその都度取得し、手作業の転記を禁止する **状態：確定。** **進捗：対象なし。** |
| 7 | **凍結階層の分割と4層外の独立要素** | 4層には厳格な依存関係（システム構成とブロックヘッダ、ポインタフィールド配置、key 符号化、インデックスノード内部レイアウト）が存在し、別途3つの独立凍結コンポーネントを設ける **状態：確定。** **進捗：対象なし。** |
| 8 | **凍結後に失われる自由** | 凍結済みフィールドの幅・存在・意味は変更不可となり追加のみ許容；凍結箇所に対する checker の判定も互換性規約となる **状態：確定。** **進捗：対象なし。** |
| 9 | **凍結後の変更判定表** | 9 分類の変更それぞれについて旧リーダーの挙動を基準に compat / compat_ro / incompat および bump の要否を判定 **状態：確定。** **進捗：対象なし。** |
| 10 | **feature bit のビット数・会計・割り当て権** | 3 つのビットマップ（各 256 ビット）をシステム構成に格納；割り当て権は「先行マージ優先」、ビット番号は厳格に単調増加；一度使用したビットは再利用を禁止 **状態：確定。** **進捗：実装済み。** |
| 11 | **凍結に伴い整備すべき赤判定検査** | フォーマット凍結規約（`spec_hash` 照合）および後方互換性リグレッション（ゴールデンイメージ）；イメージ本体は git に含めず、リストは追加のみ許容 **状態：確定。** **進捗：未実装。** |

#### 確定項目 1：第 7 項の判定対象は未確定項目であり決定リストではない

**確定内容**：3 条。

1. **対象はリポジトリ内の未だ未確定であるすべての項目であり、決定リストではない。** 未確定項目ごとに判定文言——**フォーマット変更要否**を付与する：その結論がディスク上のバイト列を変更するか、あるいは既存バイト列の**解釈計測条件**を変更するか否か（計測条件の側面は C97 (改字段含义而不改宽度，全仓没有一处会红) の形態を指す：幅は 1 バイトも変わらないが意味が変化する場合）。取り得る値は 3 つ：**変更あり / 変更なし / 境界未確定 (変更ありとみなす)**；**未記載は変更ありとみなす**。
2. **判定**：凍結要求の発行時、「変更あり」または未判定の項目があれば赤判定とする。段階的凍結を行う場合は、要求内で**全項目を個別に列挙してリスクを明示的に受容**しなければならず（確定項目 7 の「段階的凍結のリスクは明示的に受容されなければならない」を踏襲）、**列挙漏れがあれば赤判定**とする。
3. **パディング幅のデフォルトは 0** とし、非ゼロの幅を確保しようとする側に立証責任を課し、`.claude/rules/fs-design.md` のフォーマット分岐判定基準（原文ママ：「フォーマット分岐に価値があるかは、『誤用』を『マウント不可』に変えられるか否かによる」）を通過させなければならない。

問いの立て方が読み方を規定する (`.claude/singlefs-ai-sop/skills/decide/SKILL.md` 必須要件 7：未確定項目の問いは一意に読めなければならない)：「6 つのスロットの値を個別に固定する」方式（各スロットの内容は各決定内に存在するため、複製は同一事実の二重記録となる）は採らず、「第 7 項の判定規則を定める」方式を採る；チェックリストはコミット契機の前置検査であり凍結要求ごとに実行されるため、第 7 項は**凍結ごとに再判定**される。

**射程**：デフォルトパディング幅 0 の規定が所管するのは**実装しないことが確定した機能のためのプレースホルダ確保**であり、構造体内の**汎用予約領域**は所管しない——汎用予約領域は確定項目 9 判定表の第 1 行（予約領域 / TLV 末尾へのフィールド追加 ⇒ compat かつ bump なし）および C102 (索引节点头没有预留区，加字段不是 compat) が所管し、判定基準が異なる；両者を混同すると「どこにも予約を残さない」という結論に至るが、C102 (索引节点头没有预留区，加字段不是 compat) は現在その逆方向の未済の検査を清算中である。規則として昇格するのは「デフォルト 0 + 立証責任の転換」という手続きのみであり、「v1 にはこの機能を含めない」という分岐に達した項目は、各自でフォーマット分岐基準を再度通過する必要がある。述語は層ごとに分割しない：凍結からの除外は構造単位で判定され、層単位ではないためであり（確定項目 7 の射程）、かつ構成要素が揃っていない——確定項目 7 における 3 つの独立凍結コンポーネントは 4 層のいずれにも属さない ⇒ 層単位の述語はそれらに対して定義できない；「フォーマット変更要否」という二値述語で分割することでこれら双方の破綻を回避でき、層間の独立判定可能性を前提としない。D26 (バックグラウンド整理と配置回収) 確定項目 4 のセルは従来どおり D26 (バックグラウンド整理と配置回収) 自らが保持し、D15 (フォーマット凍結ポリシー) が代行して回答する権限はない。判定入力は常に現行状態から取得し、ここに複製を保持しない：`grep -c '未定\*\*' .claude/kb/decisions.md` によりリポジトリ内の未確定項目残数を数え（項目リスト行末は現在「—— **未定** · 未实现」の形式であり「未定**」で終わらない）、`bash .claude/gate.d/doc-decisions.sh --check decision-items-sync` で当該リストが本文の忠実な射影であることを判定する。

**根拠**：実験なし：ゲート判定規則の対象とデフォルト値を定めるものであり、定量可能な数値は存在しない。これを支えるのは 2 つの事実である——リスト形式には漏れが生じ得ること（D22 (ユニット原子性の合成方法) 確定項目 9 は**システム構成のフィールド表**であり、システム構成は第 1 層である；D19 (ブロックポインタの構造と幅予算) 確定項目 6 は**中央マッピングエントリのフィールド表**であり、第 2 層に位置する——いずれも 6 決定のリストには含まれず、これら 2 層を凍結する際にリストは一切関与できない）、および「フォーマットレベル」タグに依存した網羅は不可能であること（番号の登録場所は各決定本文のタイトル行であり索引ページではないため、`grep -rn 格式级 .claude/gate.d/ .claude/scripts/` はヒット 0 件 ⇒ 対象 0、実装 0）。メイン agent による確定、三方検証に基づく；成立しなかった論拠は再利用防止のため変更履歴に記録している。

**未済の検査**：C62 (分项定案值在 kb 里分叉)；C102 (索引节点头没有预留区，加字段不是 compat)。

#### 確定項目 2：ツリーテーブルを day-1 から開放リストとする

**確定内容**：ツリーテーブルは day-1 から開放リストとして構成する。根拠は確定項目 9 の判定表にある：開放リスト下では「新規 keyspace の追加」は旧リーダーにとって単に**新規ツリーが見えない**だけであり（compat グレード、グレースフルデグラデーション）；ツリー数をハードコードすると**ツリーテーブルを読み出せない**状態となる（incompat、即時破損）。**開放リストが厳格に優位**であり、その代償はツリーテーブルユニット内のテーブル長フィールドと可変長テーブル部分の追加にとどまる——これは確定項目 10 の「予約は広めに確保する」と同一の論理である。⇒ keyspace の追加は本プロジェクトにおいて明記された拡張目標であり偶発的事象ではないため、判定表の当該行のグレードは **compat** として固定される。

**射程**：定めるのは「ツリーテーブルが開放リストであること」であり、ツリーテーブルの配置先とは無関係である——ツリーテーブルはルートレコードの間接参照層に位置する（D22 (ユニット原子性の合成方法) 確定項目 7 原文ママ「ツリーのルートは常にツリーテーブルユニットに格納し、ルートスロットにフラット展開しない」、確定項目 12「システム構成は自前のツリーテーブルユニットポインタを持たない」；D21 (権威態と派生態の境界) はルートレコード + ツリーテーブルユニット + インスタンス表ユニットを一括して「ルート」に分類）。これは**第 1 層の凍結時に組み込まれなければならず**、ツリーを追加する段になって未確保であることが判明する事態は許容されない；なおツリーテーブルユニット自体の凍結層帰属は依然として未済である。

**根拠**：

- E142 (新規プールでのファイル作成のドライラン)：新規プールでのファイル作成で書き出されるツリーテーブルユニットはエントリ数で宣言され（mkfs 版は 0 エントリ、トランザクション版は 11 エントリであり、退役ツリー追加前の現行値；退役ツリー追加後のトランザクション版は 12 エントリ）、テーブル長はコンパイル時定数ではなくデータである——開放リストの形態は書き出されたバイト列において成立している。
- ユーザーによる確定、発言原文は変更履歴にある。

**未済の検査**：C246 (树表单元没有冻结层归属)。

#### 確定項目 3：`spec_hash` は kb フィールド表の完全射影から常に算出し、Rust 構造体へ切り替えない

**確定内容**：`spec_hash` は常に `format-spec/<コンポーネント>.toml` に対して算出する。当該 toml は **kb フィールド表の完全射影**であり——**行順**（ディスク上の順序）および**非スカラーのディスク上セマンティクス規則**（確定項目 7 のコンポーネント ② のマージ規則をデータ形式で記述）を含み、登録済みのスカラー定数の単なる集合ではない。コード実装後に新たに追加されるのは**ハッシュ元の変更ではなく**、2 つの検査である：① spec とコード間の**同等性検査**（幅と順序の双方を比較）；② ゴールデンイメージリグレッション（確定項目 11 第 2 条）。

4 つの理由：

1. **方向性**。D13 (検証ルート) 確定項目 5 原文ママ：「**checker と実装の間で共有されるのは唯一つ：kb のフィールド表から生成された定数モジュールのみであり**、ジェネレータが kb から読み取り双方は消費のみ行い、何人による手作業の変更も禁止する」⇒ 権威は kb にあり、Rust 構造体は**下流の消費者**である；構造体に対してハッシュを算出することは、恒久的な規約の指紋を派生品に固定することに他ならない。
2. **ハッシュは変更検知器であり、二つの成果物間の同等性検査ではない。** 構造体に対してハッシュを算出しても「構造体と kb の不一致」は捕捉できず、構造体が**変更された**際に警告を発するのみである；その種の誤りを真に捕捉するのは同等性検査であり、同等性検査は spec 側の存在を前提とする ⇒ 切り替えを行っても付加価値が得られないばかりか、照合すべき対向側を解体してしまう。
3. **言語バインディング**。D17 (実装階層とサードパーティパイプライン) 確定項目 3 原文ママ：「判定は第三者が独自に再検証可能でなければならず、さもなければ『適合』は『当方が適合と主張している』状態に劣化する」。指紋を Rust ソースコードのテキストに固定すると、サードパーティパイプラインによる再計算が不可能となる。
4. **実現性**。3 つのコンポーネントの spec ファイルは直ちに記述可能であり、ハッシュ機構を最初の凍結より遥か以前から有効化できる；一方で確定項目 7 のコンポーネント ① の凍結契機はタイプ 2 コンテナを書き出す最初のイメージである ⇒ 切り替え作業が最初の実凍結と同一コミットに強制され、試行運用期間がゼロとなる。

**射程**：spec がスカラーのみを射影するのでは不十分であり、行順を含めなければならず、同等性検査も順序を比較しなければならない——[layout/01-first-txn.md](../layout/01-first-txn.md) のユニットヘッダには幅 8 の隣接フィールドが 6 つ行順に並んでおり（5つ組のツリー ID / オブジェクト ID / オブジェクト誕生代 / アンカーオフセット、ならびに誕生代番号、fsid）、シリアライズ時に任意の 2 つを入れ替えても全スカラーおよび `DATA_UNIT_HEADER_BYTES` は一切不変のままディスク上のバイト列が完全に狂い、確定項目 9 判定表の「既存フィールドの意味変更」の行に該当することになる。D13 (検証ルート) 確定項目 5 の「ジェネレータはスカラー値のみを出力する」という境界規定は checker と実装の間の共有物を所管しており、`format-spec/<コンポーネント>.toml` は**ゲート成果物**であり実行時の消費者は存在しない ⇒ 当該境界の射程外であり、行順を持つフィールド表には分岐も算術も存在しない。**検知不能な事項の明記**：spec と実際に書き出されたバイト列との間において、ハッシュは直接の拘束力を持たない——最初のゴールデンイメージ自体が誤ったコードで出力された場合、リグレッションは誤った版を基準として扱うことになる；これを阻止するのは独立したパーサである checker および適合性スイートの判定プログラムであり、`spec_hash` ではない。spec は手書きするほかなく、kb フィールド表から toml へ射影する自動機構は存在しない。段階的凍結における「早期凍結層でのフィールド確保漏れ」の**リスク規模**を定量化することはできず、未だ存在しない D8 (コアインデックス構造) / D11 (インデックスノード内のメッセージバッファ要否) の具体的レイアウトに依存するため、確定項目 7 の損切り規則を提示できるのみであり、残留リスクは計測不能である。

**根拠**：

- E142 (新規プールでのファイル作成のドライラン)：ユニットヘッダにおける幅 8 の隣接する 6 フィールドは書き出されたバイト列上で行ごとに整列しており、任意の 2 つを入れ替えても全スカラーを不変に保つ状態は構成可能である——spec がスカラーのみを射影する形態ではフィールド順序を担保できない。
- 三方論証：事前定義された失敗条項に基づき、逆推論レグが「spec はスカラーのみを射影する」という方針を除外した。確定内容は厳格化された形態（spec は行順を含まなければならない）を採用しており、新形態は旧形態よりも追加要件を課すのみで要件を一切削減していない。メイン agent による確定。

**未済の検査**：C227 (黄金镜像的基准是它自己那份代码写的)；C226 (spec 是手抄的，没有从字段表生成)；C119 (冻结组件没有 spec 文件)。

#### 確定項目 4：feature bit のビット割り当て登録表

**確定内容**：3 つのビットマップ（各 256 ビット、確定項目 10 の 3 種のビットマップ）；ビット番号の唯一の登録場所は本節の登録表とする。「認識不能なビットでのマウント拒絶」はデフォルトレイアウトに対しても機能させる必要があるため、第 1 の系統が独自に 1 ビットを占有し mkfs 時点から設定する——「第 1 系統はビットを消費しない」方式は採らない（さもなければ SSD レイアウトを認識しないリーダーでもマウント可能となってしまうため）。

| bitmap | bit | 意味 | 出典 |
|---|---|---|---|
| incompat | 0 | 最初の純 SSD レイアウト系統 | D12 (対象メディア) 確定項目 5；D12 (対象メディア) 確定項目 1「レイアウトごとに 1 つの incompat ビット」；廃止：ロールバック証人を含みシステム構成に F を欠く版であり、リーダーは一律に未認識ビットとして拒絶する（ユーザー 2026-09-26） |
| incompat | 1 | 最初の純 SSD レイアウト系統（システム構成にロールバック下限 F を含み、ルートレコードにアンマウント記号を含み、ロールバック証人およびロールバック行を持たない） | D12 (対象メディア) 確定項目 5；ユーザー 2026-09-26 決定「incompat ビットを変更し、マウント不可とする」 |
| incompat | 2..255 | 未割り当て；レイアウト系統が新設されるごとに追加 | D12 (対象メディア) 確定項目 1 |
| compat_ro | 0..255 | 未割り当て | — |
| compat | 0..255 | 未割り当て | — |

**射程**：ビット番号の分割方法と最初のビットの帰属のみを定め、各ビットのセマンティクスは当該ビットを要求する決定に属する。新規プールでのファイル作成におけるバイト列：システム構成の feature bits 領域（96 バイト）は全ゼロから incompat ビット 1 のセット（1）へと変化する；ビットマップはリトルエンディアンであり、ビット 1 は第 1 バイトの最下位から数えて第 2 ビットである（D22 (ユニット原子性の合成方法) 確定項目 13）。本項目は三方論証を経ておらず、メイン agent の推奨およびユーザーの選択はいずれも覆され得る。

**根拠**：

- E142 (新規プールでのファイル作成のドライラン)：システム構成の feature bits 領域（96 バイト）が本項目に従って書き出され、空白領域 G12 が解消され、[layout/01-first-txn.md](../layout/01-first-txn.md) が同期更新された。
- ユーザーによる確定、発言原文は変更履歴にある；確定前の当該セルの原文も変更履歴にあり、当時の記述「バイト変更：否」を含んでいたが、その記述は本項目の確定により覆された。
- ユーザー確定 2026-09-26：ビット 0 は廃止されレイアウト識別はビット 1 に切り替わり、旧イメージ（ロールバック証人を含みシステム構成に F を欠く）はマウント不可となる（三方判定 `research/prompts/m2-rollback-forward-r3-main-verification.md` 第 2 節 K7：「証人テーブルが空でなければ書き込み可能マウントを拒絶する」方針は除外、読み出し専用リカバリの破損深度 13 パターンのうち 4 パターンがロールバックされた版に該当；システム構成フォーマットバージョン番号の変更では阻止できず、現行リーダーはバージョン番号を検証しないため）、原文は変更履歴にある；実験なし：新ビットに対する旧リーダーのマウント不可はプロトタイプで計測済み、旧ビットに対する新リーダーの拒絶は論理的導出；「システム構成フォーマットバージョン番号の変更では阻止できず、現行リーダーはバージョン番号を検証しない」との記述は陳腐化：2026-09-27 以降、リーダーはシステム構成フォーマットバージョンが 1 でない場合プール全体のマウントを拒否する（2026-10-21 決定、D22 (ユニット原子性の合成方法) 確定項目 9）。

**未済の検査**：C183 (每套布局一个 incompat 位而盘上差不出字节)。

#### 確定項目 5：凍結はゲート仲裁を受けるコミットであり言葉の表明ではない

**確定内容**：**アクション**はリポジトリに `.claude/kb/format-frozen/<コンポーネント>.toml`（内容：`version`、`frozen_at_commit`、`spec_hash`）をコミットすることである。**誰が判定するか**：人間が「準備ができたと感じた」から発効するのではない——ゲートは今回の差分が当該ディレクトリに触れたことを検知すると、より厳格な前置検査群（確定項目 6 のチェックリスト）へと自動的に切り替わり、すべてを満たした場合にのみ当該コミットの通過を許可する。**「今凍結を開始するか否か」のみを人間が決定し、「今回の凍結要求が成立するか否か」は機械が決定する**のであり、`.claude/singlefs-ai-sop/rules/show-me-test.md` の「出所によって区別せず、根拠によってのみ区別する」と同型である。

**射程**：当該ステージは通常時には実行されず（差分が当該ディレクトリに触れた時のみ実行）、日常のコミットを遅延させない；**長期間にわたりトリガー回数が 0 回となる宿命にあるが、これは欠陥ではなく正しい状態である**——ゲートが未実装ステージを明示的に列挙することと同種の誠実さである。⚠️ コミット契機であるのに対し、確定項目 7 のコンポーネント ① の凍結契機は「タイプ 2 コンテナを書き出す最初のイメージ」であり、これは実行であってコミットではない ⇒ 凍結前チェックリストはコンポーネント ① に対して実行経路を持たず、ゲートがそれに対して発火することはない。

**根拠**：実験なし：ゲートトリガーの契機と仲裁権の帰属を定めるものであり、定量可能な数値は存在しない；共有 `gate.sh` に既存の「プロジェクトローカルステージ」マウントポイントと整合させた。メイン agent による確定。

**未済の検査**：C225 (组件的冻结时刻绕开门禁触发)。

#### 確定項目 6：凍結前に完了必須のチェックリスト

**確定内容**：7 項目の前提条件であり、凍結要求の発行時に項目ごとに判定し、凍結ごとに再判定する。

| # | 項目 | 実施せずに凍結した場合の帰結 |
|---|---|---|
| 1 | 今回凍結対象となるコンポーネントのフィールド表において、kb 内のいかなる行にも「未定」「構想」「半定」「プレースホルダ」の記載がないこと | 変更途中の設計を凍結することになり、以降未確定項目の回答が補填されるたびに v1 の一部として追認されてしまう。⚠️ 本項目と第 7 項は異なる 2 つの尺度である：第 1 項は**既に書き出された表**の充足度を測り、第 7 項は**未だ書かれていない項目群**がそれに抵触するかを測る |
| 2 | I-1 (ブロック自己記述)、I-2 (チェックサム) 系（ブロック自己記述、チェックサム）の checker が実装済みかつ緑通過していること | 「checker こそが仕様」——checker がなければ「v1 とは何か」は文書内にのみ存在することになり、後続の変更を機械的に判定できなくなる |
| 3 | D9 (暗号化) 確定項目 12 の 4 つの予約フィールドについて、checker 上で「存在し幅が予約値に合致している」とアサート可能であること | 1 つでも確保を怠ると凍結後に incompat でパッチを当てるほかなく、既存の旧イメージは恒久的に当該フィールドを欠損する |
| 4 | I-3.1 (割り当て済み統計の合致) が実装済みであり、N>=5 回のフル実行ですべて緑通過していること | 会計の正当性を検証する唯一の手段がこの対比である；通過していなければ、凍結された台帳フィールドの設計が正しかったか否か一切検証されていないことになる |
| 5 | モデル突き合わせおよびクラッシュポイント再生がそれぞれ最低 1 回 PASS しており、**かつ実行対象が凍結直前のフォーマット指紋そのものであること** | さもなければ v1 が記述するフォーマットはいかなる自動検証も経ていないことになる |
| 6 | [checks-owed.md](../checks-owed.md) の C4 (项目规则清单不一致) および C8 (门禁范围判不出来) が清算済みであること | 凍結後に規則の乖離やゲートの遅延が放置されれば、よりリスクの高いフェーズにおいて障害へと直結する |
| 7 | リポジトリ内の未だ未確定であるすべての項目に「フォーマット変更要否」の判定が付与されており、「変更あり」または未判定の項目が 1 つも存在しないこと——今回の凍結要求において全件を個別に列挙しリスクを明示的に受容した場合を除く | この連鎖は相互に定義されており、「未確定」を残したまま暗黙裏に凍結することは既に決定したと詐称することと同義である。判定規則は確定項目 1 を参照 |

⚠️ **チェックリスト内の各項目の再確認において、値は常に対応項目の確定行からその都度取得しなければならず、手作業の転記を禁止する。** 凍結は不可逆であり、数値を誤記してもテキスト上いかなる違和感も生じないためである ⇒ **再確認時は当該項目の確定行を逐字確認すること**；機械検査の形態と前提条件は C62 (分项定案值在 kb 里分叉) に記録されており、それが実装されるまでは本工程は人手に依存する。

**第 1 項および第 7 項の実行可能形態**：第 1 項は今回凍結対象のフィールド表を走査する——[layout/01-first-txn.md](../layout/01-first-txn.md) のディスク上順序で行ごとに列挙された表がその既定形態であり、その**「幅」列および「状態」列**を走査し、`未定` / `预想` / `半定` / `占位` の**いずれか**が現れれば直ちに赤判定とする。第 7 項はリポジトリ内の未確定項目における「フォーマット変更要否」判定を走査する。これは 2 つに分割される：**記載の有無**についてはゲートステージ `.claude/gate.d/doc-decisions.sh` の blocking-verdict セルの第 2 の尺度であり、第 1 の尺度（新規プールでのファイル作成バイトの変更有無）と同一の解析・同一の権威リストを共有し、それぞれ数値を報告し個別の解決策を提示し常時稼働する；**赤判定**の側面（「変更あり」または未記載なら赤判定、全件列挙の上リスク明示受容の場合を除く）は凍結要求発行時の処理であり、前提は C225 (组件的冻结时刻绕开门禁触发) である——凍结要求という契機はコンポーネント ① に対して実行経路を持たない。いずれの項目も**第 2 のリストを別途作成しない**——`.claude/singlefs-ai-sop/rules/kb-discipline.md`：同一事実に対する権威ある記録は 1 箇所のみ許容される。

**射程**：第 1 項の用語集は条文内に規定し、スクリプト作成者の推測に委ねない——バイト表で現行使用されている語は「**預想**」および「**半定**」である（extent ツリーノードの行は原文ママ「**預想 162** = 86 + 2 × 24 + 28」、状態列「半定」、key 区間を含めるか否かは未定であり、含めれば 162、含めなければ 114 であり 48 バイトの差が生じエントリ領域の開始位置もずれる）、それこそが第 1 項が阻止すべき「変更途中の設計」に他ならず、「未定」のみを走査する方式では誤って通過させてしまい、第 4 層の凍結要求が 48 バイトの未確定計測条件を v1 として追認してしまうことになる。チェックリストが判定するのは前提条件の充足有無であり、いかなる項目の内容の当否も判定しない。

**根拠**：

- E142 (新規プールでのファイル作成のドライラン)：第 1 項が走査すべきフィールド表とは、それが行ごとに出力し行ごとに検証した表そのものであり、「幅」列および「状態」列における「預想」「半定」という 2 語は実在の対象を有している。
- メイン agent による確定（第 1 項の語彙が 2026 年に絞り込まれた経緯は変更履歴にある）；第 7 項の判定規則は確定項目 1 にある。

**未済の検査**：C62 (分项定案值在 kb 里分叉)；C8 (门禁范围判不出来)（チェックリスト第 6 項のもう半分である C4 (项目规则清单不一致) は清算済み）。

#### 確定項目 7：凍結は 4 層に分割し、4 層外に独立した 3 つの凍結コンポーネントを設ける

**確定内容**：4 層には厳格な依存順序が存在する。

```
第 1 層：システム構成 + ブロックヘッダ（自アドレス / generation / ツリー ID / fsid / magic）
    ↓
第 2 層：ポインタフィールドレイアウト（D4 チェックサム + D9 の MAC / nonce / 暗号文チェックサム）
    ↓
第 3 層：各 keyspace の key 符号化（D8 の locality_id / inode / offset）
    ↓
第 4 層：インデックスノード内部レイアウト（ノードサイズ規則、bset 形式、D11 のメッセージバッファ）
```

依存順序の理由：第 1 層は「メタデータブロックは自己記述でなければならない」という物理的前提であり、後続のすべての層はその上に構築される；第 2 層は D9 (暗号化) の MAC / nonce / 暗号文チェックサムが「事後追加不能」であることにより固定され、これに依存するツリー構造に先行しなければならない；第 3 層の key 符号化が定まって初めて、ノード内部でこれら key をどう編制するかの議論基盤が成立する。

**段階的凍結のリスクは明示的に受容されなければならない**：早期に凍結された層でフィールド確保を怠った場合、後続は incompat によるパッチ適用を余儀なくされ、全体設計時に確保すべきであった余力を前倒しで喪失することに等しい。損切り規則は唯一つである：**第 1 層および第 2 層の凍結前に D9 (暗号化) の全 4 つの予約フィールドを内包していなければならない**。これらは「いずれ必要となることが明白な」フィールドであり、第 3 層・第 4 層での補填を期待することはできないためである。

**4 層外の独立凍結コンポーネント**：4 層のいずれにも属さない恒久的なバイト列が 3 つ存在し、それぞれを独立したコンポーネントとして立脚させ、C9 (冻结后偷改格式) の `spec_hash` はコンポーネント単位で記録し、4 層側には影響を与えない：

- ① **パック記録ユニットのペイロードレコード形式**、パック記録種別ごとに登録：種別 2 の 140 バイト inode レコード表（D8 (コアインデックス構造) 確定項目 6）、凍結契機は種別 2 コンテナを書き出す最初のイメージ；種別 1 のもの（幅 40、D18 (ブロックに含める情報) 確定項目 24）は相互に独立しており、種別 2 と一括凍結は行わない。
- ② **統計量登録表**（タグ値 + 次元 + マージ規則、D5 (スナップショット・領域会計メカニズム) 確定項目 3 / 4）——マージ規則はディスク上セマンティクスであり、2 つのリーダーが同一のバイト列に対し異なる規則を適用すれば異なる値を算出してしまう；C71 (不认识的统计量标签静默忽略) は「未認識タグ」のみを保護し、「タグは認識するが誤った規則を適用する」事態は防げない。
- ③ **journal 記録形式**（レコードヘッダ、指定項目、ペイロードチェックサムの適用範囲、トランザクション境界）：凍結前提条件は第 2 層および配置先粒度セルより先行せず、凍結契機は当該 toml を提出するコミットとし、新規レコード形式は「新規レコード種別 + システム構成 log-incompat ビット」を採用する。全文は D16 (パブリッシュセマンティクス) 確定項目 3 にある。

**射程**：4 層の図は再編しない：D21 (権威態と派生態の境界) 確定項目 10 は状態区分を構造単位の判定へと変更した；第 4 層において会計ツリーのコード 2 ノードは権威態、他のツリーの内部ノードは派生態であり、純粋な層は存在しないため、凍結除外の単位は構造体であり層ではない（各構造体がどの層に属し、状態区分が何であり、凍結から除外されるか否かは `.claude/kb/freeze-layer-membership.md` に登録され、ゲート doc-decisions の freeze-layer-membership セルが判定する）；3 つのコンポーネントの spec ファイルは存在しない；コンポーネント ① の凍結契機は実行でありコミットではないため、確定項目 5 の遮断ゲートはそれに対して発火しない。ツリーテーブルユニットおよびインスタンス表ユニットがどの層に属するかは未だ条項が存在しない。

**根拠**：実験なし：層の分割と依存順序は「後から追加できない要素はどれか」から導出した順序付けであり、定量可能な数値は存在しない；損切り規則の対象（D9 (暗号化) の 4 つの予約フィールド）は D9 (暗号化) 確定項目 12 にある。メイン agent およびユーザーによる確定、原文は変更履歴にある。

**未済の検査**：C119 (冻结组件没有 spec 文件)；C225 (组件的冻结时刻绕开门禁触发)；C246 (树表单元没有冻结层归属)。

#### 確定項目 8：凍結後に失われる自由

**確定内容**：

| 流動フォーマット期の自由 | 凍結後に何へと変化するか |
|---|---|
| 構造を「解体して再構築」 | 凍結済みフィールドの幅 / 存在 / 意味は変更不可；追加のみ許容され、廃止フィールドは「読み出しサポート維持、書き込み停止」とする |
| 設計の誤りを発見時に互換性を残さず本文を直接修正 | 決定記録レベルでは依然として覆すことが可能であるが、**ディスク上バイト列レベルでは互換性コストがゼロから非ゼロへと転換する** |
| mkfs の自動選択規則でアルゴリズムを随時変更 | 凍結済みコンポーネントの選択規則自体も固定される；アルゴリズム変更により新旧イメージのデフォルト値が乖離するため、新規フォーマットバージョン番号の下で表現されなければならない |
| 実験結果が芳しくない場合にフォーマットを直接改訂 | 新規 feature bit を通じて新規ポリシーを導入し、旧値の解釈能力を保持しなければならない |
| フォーマットに合わせて checker を自由に変更 | **凍結部分について、checker の判定は事実上の互換性規約となる**——凍結フィールドに対する checker の解釈を密かに改変することは、フォーマットを密かに改変することと同等に危険である |

**射程**：凍結後にいかなる行為が非正当となるかを述べるにとどまり、個別の具体的変更がいずれのグレードに属するかは判定しない——それは確定項目 9 の所管である。最初の外部ユーザーが現れるまでフォーマットは流動的であり、本表はそれ以前には一切効力を持たない (`.claude/rules/format-evolution.md`)。

**根拠**：実験なし：「対外的な恒久公約は変更できない」という制約を項目ごとに展開したものであり、定量可能な数値は存在しない。最終行の規定（checker の判定も規約となる）は D13 (検証ルート) の「checker こそが仕様」と同源である。メイン agent による確定。

**未済の検査**：なし。

#### 確定項目 9：凍結後の変更判定表

**確定内容**：

| 変更分類 | 旧リーダーはどう振る舞うか | グレード | bump | 理由 |
|---|---|---|---|---|
| 予約領域 / TLV 末尾へのフィールド追加 | 無視し、正常動作を継続 | compat | 否 | セマンティクス依存のない純粋な追加であり、「未知のフィールドを無視する」セーフティネットで十分であるため |
| フィールド追加であり、その欠落により旧リーダーが**他の既存フィールド**を誤計算する場合 | **誤計算し、データを破損する恐れがある** | incompat | 是 | セマンティクスが結合した場合、セーフティネットは破綻するため |
| 既存フィールド幅の拡張 | 旧幅で解析し、範囲外読み出し / 位置ずれ | incompat | 是 | 幅はレイアウトの一部であり、旧コードのオフセット計算が完全に狂うため |
| 既存フィールドの意味の変更 | 旧解釈で処理し、**エラーを出さずに**不正な挙動を発生 | incompat、かつ**旧位置を廃棄し再利用禁止** | 是 | 位置の再利用は新旧リーダーの識別を不能とし、最も危険な分類であるため。実例 2 件（D18 (ブロックに含める情報) 確定項目 11）：ユニット種別の追加（登録表コード 4–15）は種別識別セグメントの解釈計測条件を変更する；共通プレフィックスオフセット 7 の常に 0 である flag ビットへの意味付与（旧リーダーは非ゼロビットを一律拒絶） |
| レコード種別の追加（新規 key 種別） | 読み出し専用トラバースが誤判定の恐れ；整理 / 書き込みが理解不能なレコードを誤削除する恐れ | compat_ro | 是 | **「読み出し専用の安全性」と「書き込み可能の安全性」は独立した二重の保証**であり、個別に判定しなければならないため。⚠️ パック記録ユニット内のパック記録種別は**本行に従わない**（D18 (ブロックに含める情報) 確定項目 11）：墓石系または inode 系のレコードのスキップは読み出し専用の回答を変化させるためデフォルトは incompat とする；登録時に「スキップしても読み出し専用の回答が一切不変である」と証明された種別のみ compat_ro へ降格し、compat_ro リーダーはスキャン再構築の実行を禁止する |
| keyspace の追加 | ツリーテーブルが day-1 から開放リストの場合：新規ツリーが見えないのみ；ツリー数がハードコードの場合：ツリーテーブルを読み出せない | **compat**（確定項目 2 によりツリーテーブルは day-1 から開放リスト化）。開放ツリーテーブルを確保済み → compat；未確保 → incompat | 是 | **「ツリーテーブルが拡張可能か否か」は凍結時に決定されなければならず**、ツリー追加時に未確保であることが判明する事態は許容されないため |
| ソート順序の変更 | 二分探索 / トラバース結果の誤り、**エラーなし** | incompat | 是 | ソート順序はインデックスの本質であり、ログ構造ノードの並べ替えは順序の不変性に直結するため |
| チェックサム / MAC アルゴリズムの変更 | D9 (暗号化) のアルゴリズム種別フィールドが確保済み → 未認識の種別に遭遇時は明示的に失敗 | 確保済み → compat_ro；未確保 → incompat | 是 | D9 (暗号化) 確定項目 12 第 4 項はこの種の変更のために用意された流路であるため |
| フィールドの廃止 | 影響を受けず、旧リーダーは旧方式で解釈を継続 | compat、かつ**パース用コードを恒久保持** | 否 | 一度ディスクに書き込まれたフィールドは、旧イメージが 1 つでも存在する限り恒久的に解釈可能でなければならないため |

**射程**：「旧リーダーがどう振る舞うか」を判定するものであり、「その変更を行うべきか否か」は判定しない。第 1 行の規定（予約領域への追加は compat）は真に予約領域が存在する場合にのみ成立し、インデックスノードヘッダには 2 バイトの予約領域しかなく、格納可能なフィールドは極めて限られる。

**根拠**：

- E59 (缓冲消息能不能只从单元重算)：墓石系レコードを 1 つスキップすると削除済み key がすべて復活し、復活数は削除回数に正確に一致する——「パック記録種別は compat_ro の行に従わずデフォルトは incompat である」という文言の定量的根拠。
- メイン agent による確定、行ごとの理由は表内に記載。

**未済の検査**：C102 (索引节点头没有预留区，加字段不是 compat)；C97 (改字段含义而不改宽度，全仓没有一处会红)。

#### 確定項目 10：feature bit のビット数・会計・割り当て権

**確定内容**：

- **ビット数**：3 つのビットマップ（各 **256 ビット**、`u64[4]`）をシステム構成に格納。ビットはフィールド幅より遥かに低コストであり（3×32 バイトはシステム構成にとって無視可能）、**予約は広めに確保する**。
- **会計**：`.claude/kb/feature-bits.md` に `ビット番号 | カテゴリ | 名称 | 導入バージョン | 導入コミット | 状態 | セマンティクス一言` を列挙；ビット番号の唯一の登録場所は確定項目 4 の表とする。
- **割り当て権は「先行マージ優先」とし、承認制としない**。ゲートによる強制：ビット番号は厳格に単調増加し、番号の飛び越し予約を禁止し、表に存在しないビット番号を参照するコードを禁止——機械的仲裁を行う。
- **一度使用したビットは再利用を禁止する。** 根拠は [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md) の番号非回収規定より**さらに硬い**：番号は検索キーであり、回収による誤導は「文書を読む人間またはモデル」にとどまる；**feature bit はディスク上に物理的に実在したビットであり**、回収して新機能に再割り当てした場合、旧イメージを新コードでマウントした際に当該ビットが「新機能 X が有効」と誤読され——**ファイルシステムが実際に誤作動を起こす**（例：存在しない暗号化特徴を存在すると誤認し、存在しない鍵を探索する）。廃止されたビットは「廃止直前の最終使用セマンティクスに恒久固定」と表記し、新機能への付与を禁止する。

**射程**：ビットの分割・記録・回収可否を所管し、各ビットのセマンティクス（要求した決定に属する）は所管しない。

**根拠**：実験なし：ビット数および再利用禁止は「旧イメージが新コードにより読み出される」という物理的事実から導出したものであり、定量可能な数値は存在しない；256 ビットの空間コストはシステム構成にとって無視可能であり、本セルは節約競争を行わない。メイン agent による確定。

**未済の検査**：C183 (每套布局一个 incompat 位而盘上差不出字节)。

#### 確定項目 11：凍結に伴い整備すべき赤判定検査

**確定内容**：

1. **フォーマット凍結規約**：凍結済みコンポーネントごとに `spec_hash` を記録；ゲートは再計算して照合し、不一致時は今回の差分が「`feature-bits.md` に新規行が存在するか、または `version` が明示的に変更されている」ことと「`decisions.md` の対応条目が同一コミット内で変更されている」ことの**双方**を満たすことを要求する。いずれか一方でも欠落すれば赤判定とする。
2. **後方互換性リグレッション（ゴールデンイメージ）**：あるコンポーネントのバージョン変更ごとに**当時のコード**を用いてイメージを生成・アーカイブし、ゲートは**現行コード**を用いて全履歴イメージを読み出し、checker を実行し、期待されるコンテンツダイジェストを照合する。イメージ本体は **git に含めない** (`.claude/singlefs-ai-sop/rules/command-safety.md`：テストイメージのリポジトリ内格納を禁止)；リポジトリにはマニフェスト `.claude/kb/golden-images.toml` のみをコミットする；イメージは `${SINGLEFS_GOLDEN_DIR:-$HOME/.cache/singlefs-golden}` に配置する。マニフェストは**追加のみ許容**される——特定バージョンのリグレッション削除はその互換性公約を密かに放棄することに等しく、正式な決定記録を経なければならない。

**射程**：双方が未実装であり、番号は未済の検査表にある。第 2 項は「現行コードが過去のイメージを理解できるか」のみを比較し、「当時のイメージが正しく書き出されていたか」は比較しない；第 1 項は宣言に固定されバイト列には固定されない；双方は「最初のゴールデンイメージ自体が誤ったコードで出力された」事態を防ぐことはできない。

**根拠**：実験なし：2 つのゲート検査がいかなる判定を行うべきかを定めるものであり、判別力の自己証明は実装を待つ必要がある；これらが防ぐべき障害形態（フォーマットの不正改変、互換性公約の密かな放棄）は C9 (冻结后偷改格式) および C227 (黄金镜像的基准是它自己那份代码写的) にそれぞれ 1 条存在する。メイン agent による確定。

**未済の検査**：C9 (冻结后偷改格式)；C227 (黄金镜像的基准是它自己那份代码写的)。

````

**出处 `.claude/kb/decisions/15-フォーマット凍結ポリシー.ja.md:234-237`（整段抄，未转述）**

```markdown
## 履歴バージョン

D15 (フォーマット凍結ポリシー) の履歴項目は [decisions-history.md](../decisions-history.md) に集約されている。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:1-6`（整段抄，未转述）**

```markdown
## D15 格式冻结政策 —— 已定

**中文** · [English](15-format-freeze-policy.en.md) · [日本語](15-フォーマット凍結ポリシー.ja.md)

D15（格式冻结政策） 管两个问题：**什么时候允许说「format v1 frozen」**，以及**冻结之后新增东西走什么通道**。它是 D11（索引节点要不要留消息缓冲区） / D12（目标介质） / D14（双轨（大小文件 / 持久临时）） 的共同前置——「格式分支永久」这条成本的当期值为零、冻结后为无穷，那几条决策的真实代价由它定义。不管的：每一条字段本身该定什么值归各自那条决策；第一个外部用户出现之前格式是软的，那条政策在 `.claude/rules/format-evolution.md`；`feature-bits.md` 里每一位的语义归申请那一位的决策，D15（格式冻结政策） 只管位号怎么分、能不能回收。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:25-40`（整段抄，未转述）**

```markdown
#### 已定项 1：第 7 项的对象是未定分项，不是决策名单

**定案**：三条。

1. **对象是全仓每一条仍未定的分项，不是一份决策名单。** 每条未定分项要带一句判定——**动不动格式**：它的答案会不会改动任何盘上字节，或改动已有字节的**解释口径**（口径这半是 C97（改字段含义而不改宽度，全仓没有一处会红） 的形态：宽度一个字节不变而含义变了）。三个取值：**动 / 不动 / 界不定（视同动）**；**没写过视同动**。
2. **判定**：冻结请求发起时，凡判「动」或没判过的，判红。要分批冻结就在请求里**逐条列名并显式接受风险**（形态照已定项 7 那条「分批冻结的风险必须显式接受」），**列漏一条判红**。
3. **留白宽度默认 0**，举证责任在要留非零宽度的一方，必须过 `.claude/rules/fs-design.md` 的格式分支判据（逐字：「一条格式分支值不值得，看它能不能把『用错了』变成『挂不上』」）。

问法先定读法（`.claude/singlefs-ai-sop/skills/decide/SKILL.md` 硬要求 7：待定项的问法必须只有一种读法）：不取「逐条钉死六格的取值」（每一格的内容住在各自那条决策里，复制一份就是同一事实两处记录），取「定第 7 项的判定规则」；第 7 项是**逐次冻结重判**，因为清单是 commit 触发的前置检查，每一次冻结请求都跑一遍。

**射程**：留白宽度默认 0 这一条管的是**给一个已定不做的特性占位**，不管结构里的**通用预留区**——通用预留区由已定项 9 判定表第一行（新增字段在预留区 / TLV 尾部 ⇒ compat 不 bump）与 C102（索引节点头没有预留区，加字段不是 compat） 管，判据不同；把两者混成一条会推出「哪儿都别留」，而 C102（索引节点头没有预留区，加字段不是 compat） 正在往相反方向还账。升成规则的只有「默认 0 + 举证责任倒置」这套程序，任何一条走到「v1 不含此特性」那一支的条目，都要自己再过一遍格式分支判据。谓词不按层切：退出冻结按结构判、不按层判（已定项 7 的射程），且成员不齐——已定项 7 里那三个独立冻结组件不属于四层里的任何一层 ⇒ 按层切的谓词在它上面没有定义；按「动不动格式」这个二值谓词切，这两条毛病都碰不到，而且不假设层与层之间可以独立判定。D26（后台整理与放置回收） 已定项 4 那一格照旧由 D26（后台整理与放置回收） 自己拥有，D15（格式冻结政策） 无权替它答。判定输入一律现取，不在这里存副本：`grep -c '未定\*\*' .claude/kb/decisions.md` 数全仓还有几条未定分项（分项清单行末现为「—— **未定** · 未实现」这一形，不再以「未定**」收尾），`bash .claude/gate.d/doc-decisions.sh --check decision-items-sync` 判那张清单是不是正文的忠实投影。

**依据**：无实验：定的是门禁判定规则的对象与默认值，没有可量的量。撑它的是两条事实——名单形态漏得到（D22（单元原子性怎么合成） 已定项 9 是**系统配置的字段表**，而系统配置就是第 1 层；D19（块指针的结构与宽度预算）已定项 6 是**中央映射条目的字段表**，落在第 2 层——两条都不在那份六条名单里，冻这两层时名单一个字都说不上话），以及靠一个「格式级」标签去兜是兜不住的（编号的登记位在各决策正文的标题行、不在索引页，`grep -rn 格式级 .claude/gate.d/ .claude/scripts/` 零命中 ⇒ 零对象、零实现）。主 agent 定案，按三方论证；一条不成立的论据写在变更史里，免得再用。

**欠**：C62（分项定案值在 kb 里分叉）；C102（索引节点头没有预留区，加字段不是 compat）。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:41-53`（整段抄，未转述）**

```markdown
#### 已定项 2：树表 day-1 做成开放列表

**定案**：树表 day-1 就做成开放列表。依据在已定项 9 那张判定表里：开放列表下「新增一棵 keyspace」对旧读者只是**看不到新树**（档次 compat，优雅降级）；树数量硬编码则是**读不出树表**（incompat，直接坏）。**开放列表严格占优**，代价只是树表单元里多一个表长字段加一段可变长表——与已定项 10「预留要往宽了留」是同一条理由。⇒ 加 keyspace 是本工程明写的扩展目标，不是意外，判定表里那一行的档次因此钉死为 **compat**。

**射程**：定的是「树表是开放列表」，与树表住哪无关——树表住根记录的间接层（D22（单元原子性怎么合成） 已定项 7 逐字「树根一律住树表单元，不平铺进根槽」、已定项 12 定「系统配置不带自己的树表单元指针」；D21（权威态与派生态的分界） 把根记录 + 树表单元 + 实例表单元一并划进「根」）。它**必须在第 1 层冻结时就做进去**，不能等到要加树才发现没留；而树表单元自己的冻结层归属仍欠。

**依据**：

- E142（新池新建文件的干跑）：新池新建文件写出的树表单元按条数声明（mkfs 那版 0 条、事务那版 11 条，是退役树加进之前的现值；退役树加进之后事务那版 12 条），表长是数据不是编译期常数——开放列表这个形态在写出来的字节上成立。
- 用户定案，原话在变更史。

**欠**：C246（树表单元没有冻结层归属）。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:54-73`（整段抄，未转述）**

```markdown
#### 已定项 3：`spec_hash` 恒对 kb 字段表的完整投影求，不切到 Rust 结构体

**定案**：`spec_hash` 恒对 `format-spec/<组件>.toml` 求。那份 toml 是 **kb 字段表的完整投影**——含**行序**（盘上顺序）与**非标量的盘上语义规则**（已定项 7 里组件 ② 的合并规则以数据形式写进去），不是已登记的那批标量常量的集合。有代码之后新增的**不是换哈希源**，是两道检查：① spec 与代码之间的**相等性检查**（宽度与顺序都比）；② 黄金镜像回归（已定项 11 第 2 条）。

四条理由：

1. **方向**。D13（验证路线） 已定项 5 逐字：「**checker 与实现之间只共享一样东西：一份由 kb 的字段表生成的常量模块**，生成器从 kb 读、两边都只消费，任何人不许手改」⇒ 权威在 kb，Rust 结构体是**下游消费者**；对结构体求哈希，是把永久契约的指纹锚在派生物上。
2. **哈希是变更探测器，不是两份工件的相等性检查。** 对结构体求哈希抓不到「结构体与 kb 不一致」，它只在结构体**变化**时报警；真正抓那一类错的是相等性检查，而相等性检查要求 spec 侧存在 ⇒ 切过去不但没多买到东西，还把要比对的另一侧拆掉了。
3. **语言绑定**。D17（实现分层与第三方管道） 已定项 3 逐字：「判定必须能被第三方自己复核，否则『合规』就退化成『我们说你合规』」。指纹锚在 Rust 源码文本上，第三方管道复算不了它。
4. **落地**。三个组件的 spec 文件写得出来，哈希机制可以远早于第一次冻结开始生效；而已定项 7 里组件 ① 的冻结时刻是第一个写出类型 2 容器的镜像 ⇒ 切换会被逼到与第一次真实冻结同一个提交里完成，零试跑期。

**射程**：spec 只投影标量是不够的，必须含行序，相等性检查也要比顺序——[layout/01-first-txn.md](../layout/01-first-txn.md) 的单元头逐行列着六个宽度都是 8 的相邻字段（五元组的树 ID / 对象 ID / 对象出生代 / 锚点偏移，以及诞生代号、fsid），把序列化时任意两个对调，所有标量与 `DATA_UNIT_HEADER_BYTES` 一个都不变、盘上字节全错位，落进已定项 9 判定表「改变已有字段含义」那一行。D13（验证路线） 已定项 5 那条「生成器只发射标量值」的边界管的是 checker 与实现之间的共享物，而 `format-spec/<组件>.toml` 是**门禁工件**、运行期一个消费者都没有 ⇒ 不在那条边界的射程里，而且一张带行序的字段表既没有分支也没有算术。**抓不到的明说**：spec 与实际写出的字节之间，哈希不构成绑定——第一份黄金镜像若本身就是错代码写出来的，回归会把错的那份当基准；堵它的是 checker 这个独立解析器与一致性套件的判定程序，不是 `spec_hash`。spec 只能手写，没有任何从 kb 字段表投影到 toml 的机制。分批冻结「早冻层漏留字段」的**风险大小**给不出量化形式，它依赖尚不存在的 D8（核心索引结构） / D11（索引节点要不要留消息缓冲区） 具体布局，只能给出已定项 7 那条止损规则，残余风险无法量。

**依据**：

- E142（新池新建文件的干跑）：单元头那六个宽度都是 8 的相邻字段在写出来的字节里逐行排着，对调任意两个而全部标量不变这件事是可构造的——spec 只投影标量罩不住字段顺序。
- 三方论证：反推腿按跑前写死的失败条款判「spec 只投影标量」那条臂出局，定案取的是收严后的形态（spec 必须含行序），新形态比旧形态多要求一样东西、没少要求任何东西。主 agent 定案。

**欠**：C227（黄金镜像的基准是它自己那份代码写的）；C226（spec 是手抄的，没有从字段表生成）；C119（冻结组件没有 spec 文件）。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:74-95`（整段抄，未转述）**

```markdown
#### 已定项 4：feature bit 的位分配登记表

**定案**：三个 bitmap 各 256 位（已定项 10 的三张位图），位号的唯一登记位是这一节的登记表。「不认识不许挂」要对默认布局也生效，所以第一条线自己占一位、mkfs 起就置上——不取「第一条线不占位」那条（那样不认识 SSD 布局的读者照样能挂）。

| bitmap | 位 | 含义 | 出处 |
|---|---|---|---|
| incompat | 0 | 第一条纯 SSD 布局线 | D12（目标介质） 已定项 5；D12（目标介质） 已定项 1「每套布局一个 incompat 位」；退役：带回退见证、系统配置不带 F 的那一版，读者一律按不认识的位拒（用户 2026-09-26） |
| incompat | 1 | 第一条纯 SSD 布局线（系统配置带回退下界 F、根记录带卸载记号、没有回退见证与回退行） | D12（目标介质） 已定项 5；用户 2026-09-26 定「换 incompat 位，挂不上」 |
| incompat | 2..255 | 未分配；每开一条布局线追加一位 | D12（目标介质） 已定项 1 |
| compat_ro | 0..255 | 未分配 | — |
| compat | 0..255 | 未分配 | — |

**射程**：只定位号怎么分与第一位的归属，每一位的语义归申请那一位的决策。新池新建文件的字节：系统配置 feature bits 那 96 字节从全 0 变成 incompat 位 1 置 1，位图小端、位 1 是第一个字节从最低位数起的第 2 位（D22（单元原子性怎么合成） 已定项 13）。这一项没有走三方论证，主 agent 的推荐与用户的选择都可推翻。

**依据**：

- E142（新池新建文件的干跑）：系统配置 feature bits 那 96 字节按这一项写出来，它的空白 G12 随之收口，[layout/01-first-txn.md](../layout/01-first-txn.md) 同步改。
- 用户定案，原话在变更史；定案之前那一格的原文也在变更史，含它当天写的「改字节：否」——那句已被这一项的定案推翻。
- 用户定案 2026-09-26：位 0 退役、布局身份换成位 1，旧镜像（带回退见证、系统配置不带 F）挂不上（三方判决 `research/prompts/m2-rollback-forward-r3-main-verification.md` 第二节 K7：「见证表非空就拒可写挂载」出局，只读恢复 13 个改坏深度里 4 个落到被回退掉的那一版；改系统配置格式版本号挡不住，今天的读者不核版本号），原话在变更史；无实验：旧读者对新位挂不上在原型上量过，新读者对旧位拒是推的；「改系统配置格式版本号挡不住，今天的读者不核版本号」这一句已过时：2026-09-27 起读者遇系统配置格式版本不是 1 整池拒绝挂载（2026-10-21 定，D22（单元原子性怎么合成） 已定项 9）。

**欠**：C183（每套布局一个 incompat 位而盘上差不出字节）。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:96-105`（整段抄，未转述）**

```markdown
#### 已定项 5：冻结是一次受门禁仲裁的提交，不是一句话

**定案**：**动作**是向仓库提交 `.claude/kb/format-frozen/<组件>.toml`（内容含 `version`、`frozen_at_commit`、`spec_hash`）。**谁判**：不是人「觉得可以了」就生效——门禁一旦发现本次 diff 触碰了这个目录，自动切换到一套更严的前置检查（清单在已定项 6），全部满足才允许该 commit 通过。**人只决定「现在要不要发起冻结」，机器决定「这次冻结请求成不成立」**，与 `.claude/singlefs-ai-sop/rules/show-me-test.md`「不按来源区分，只按证据区分」同构。

**射程**：那个阶段平时不跑（只在本次 diff 触碰这个目录时才跑），因此挂上去也不拖慢日常提交；**它注定常年触发 0 次，这不是缺陷，是正确状态**——与门禁显式列出未实现阶段是同一种诚实。⚠️ 它是 commit 触发的，而已定项 7 里组件 ① 的冻结时刻是「第一个写出类型 2 容器的镜像」，那是一次运行、不是一次提交 ⇒ 冻结前清单对组件 ① 没有执行路径，门禁永远不会为它点火。

**依据**：无实验：定的是门禁触发时机与仲裁归属，没有可量的量；形态与共享 `gate.sh` 已有的「项目本地阶段」挂载点对齐。主 agent 定案。

**欠**：C225（组件的冻结时刻绕开门禁触发）。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:106-132`（整段抄，未转述）**

```markdown
#### 已定项 6：冻结前必须完成的清单

**定案**：七项前置，冻结请求发起时逐项判，逐次冻结重判。

| # | 项 | 不做它冻结会怎样 |
|---|---|---|
| 1 | 本次要冻的组件，它的字段表在 kb 里没有一行写着「未定」「预想」「半定」「占位」 | 冻结的是一个还在动的设计；此后每次补答一个待定项都会被追认成 v1 的一部分。⚠️ 它与第 7 项是两把尺子：第 1 项量**已经写下来的那张表**齐不齐，第 7 项量**还没写下来的那些分项**动不动它 |
| 2 | I-1（块自描述）、I-2（校验和） 类（块自描述、校验和）checker 已实现且跑绿 | 「checker 即规范」——没有 checker，「v1 是什么」只存在于文档里，后续改动无法被机械判定 |
| 3 | D9（加密） 已定项 12 的四项预留字段在 checker 里可断言「存在且宽度符合预留」 | 漏留一项，冻结后只能靠 incompat 打补丁，且已有旧镜像永远缺这个字段 |
| 4 | I-3.1（已分配统计对得上） 已实现且跑过 N≥5 轮全绿 | 记账正确性唯一的验证手段就是这条对照；没跑过，冻结的账本字段是不是设计对了完全没被检验 |
| 5 | 模型对拍与崩溃点重放 各至少跑过一轮 PASS，**且跑的正是即将冻结的那份格式指纹** | 否则 v1 描述的是一份从未被任何自动化验证碰过的格式 |
| 6 | [checks-owed.md](../checks-owed.md) 的 C4（项目规则清单不一致） 与 C8（门禁范围判不出来） 已实现 | 冻结后规则脱节、门禁越跑越慢却没人量化，会在风险更高的阶段变成事故 |
| 7 | 全仓每一条仍未定的分项都带着「动不动格式」那句判定，且没有一条判「动」或没判过——除非本次冻结请求把它们逐条列名并显式接受风险 | 这条链互相定义，沉默地带着「待定」冻结等同于假装它已经决定了。判定规则见已定项 1 |

⚠️ **清单里每一项的复核，取值一律从对应分项的定案行现取，不许手抄。** 冻结是不可逆的，而抄错一个数在文本上没有任何地方看起来别扭 ⇒ **复核时逐字打开那条分项的定案行**；机检形态与它的前置记在 C62（分项定案值在 kb 里分叉），在那条实现之前这一步靠人。

**第 1 项与第 7 项的可执行形态**：第 1 项扫本次要冻组件的字段表——[layout/01-first-txn.md](../layout/01-first-txn.md) 那张按盘上顺序逐行列字段的表就是它的现成形态，扫它的**「宽度」列与「状态」列**，出现 `未定` / `预想` / `半定` / `占位` **任一**即判红。第 7 项扫全仓未定分项里那句「动不动格式」的判定。它拆成两半：**写没写**那一半是门禁阶段 `.claude/gate.d/doc-decisions.sh` 的 blocking-verdict 格的第二把尺，与那一道的第一把尺（改不改新池新建文件的字节）共用同一次解析、同一份权威清单，各报各的数、各带各的出路，常跑；**判红**那一半（凡判「动」或没写过即判红，除非逐条列名并显式接受风险）是冻结请求发起时的事，前置是 C225（组件的冻结时刻绕开门禁触发）——冻结请求这个触发对组件 ① 没有执行路径。两项都**不另建第二份清单**——`.claude/singlefs-ai-sop/rules/kb-discipline.md`：同一事实只许一处权威记录。

**射程**：第 1 项的词表写进条款、不留给写脚本的人猜——字节表今天用的词是「**预想**」与「**半定**」（extent 树节点那一行逐字「**预想 162** = 86 + 2 × 24 + 28」、状态列「半定」，带不带 key 区间还没定，带 162、不带 114，差 48 字节，条目区起点跟着挪），那正是第 1 项要拦的「还在动的设计」，而按只扫「未定」的形态它一定放行，第 4 层的冻结请求就能把一个 48 字节的未决口径追认成 v1。清单判的是前置齐不齐，判不了任何一条分项的内容对不对。

**依据**：

- E142（新池新建文件的干跑）：第 1 项要扫的那张字段表就是它逐行写出来又逐行核过的那张，「宽度」列与「状态」列上「预想」「半定」这两个词有真实对象。
- 主 agent 定案（第 1 项词表 2026 年那次收窄的经过在变更史），第 7 项的判定规则在已定项 1。

**欠**：C62（分项定案值在 kb 里分叉）；C8（门禁范围判不出来）（清单第 6 项的另一半 C4（项目规则清单不一致） 已还清）。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:133-162`（整段抄，未转述）**

````markdown
#### 已定项 7：冻结分四层，四层之外另有三个独立冻结组件

**定案**：四层有严格依赖顺序。

```
第 1 层：系统配置 + 块头（自身地址 / generation / 树 ID / fsid / magic）
    ↓
第 2 层：指针字段布局（D4 校验和 + D9 的 MAC / nonce / 密文校验和）
    ↓
第 3 层：各 keyspace 的 key 编码（D8 的 locality_id / inode / offset）
    ↓
第 4 层：索引节点内部布局（节点大小规则、bset 形态、D11 的消息缓冲区）
```

依赖方向的理由：第 1 层是「元数据块必须自描述」的物理前提，后面所有层都建在它之上；第 2 层被 D9（加密） 的 MAC / nonce / 密文校验和「事后加不进去」钉死，必须先于依赖它的树结构；第 3 层的 key 编码定了，节点内部怎么组织这些 key 才有讨论基础。

**分批冻结的风险必须显式接受**：早冻的层若漏留字段，后面只能走 incompat 打补丁，等价于提前烧掉一次本该整体设计时留的余量。止损规则只有一条：**第 1、2 层冻结前必须已包含 D9（加密） 全部四项预留**，因为它们恰好是「已知迟早要用」的字段，不能指望第 3、4 层再补。

**四层之外的独立冻结组件**：有三样永久字节不属于四层里的任何一层，各立成一个组件，C9（冻结后偷改格式） 的 `spec_hash` 按组件记，四层不动——

- ① **打包记录单元的载荷记录格式**，按打包记录类型登记：类型 2 的 140 字节 inode 记录表（D8（核心索引结构） 已定项 6），冻结时刻是第一个写出类型 2 容器的镜像；类型 1 的（宽 40，D18（块里携带什么信息） 已定项 24）各自独立，不因类型 2 一起冻结。
- ② **统计量登记表**（标签值 + 维度 + 合并规则，D5（快照 / 空间记账机制） 已定项 3 / 4）——合并规则是盘上语义，两个读者对同一批字节用不同规则会算出不同的值，C71（不认识的统计量标签静默忽略） 只罩「不认识的标签」，罩不到「认识标签但用错规则」。
- ③ **journal 记录格式**（记录头、点名项、载荷校验和的覆盖范围、事务边界）：冻结前置不早于第 2 层与落点粒度那一格，冻结时刻是提交它那份 toml 的 commit，新记录格式走「新记录类型 + 系统配置 log-incompat 位」，全文在 D16（发布语义） 已定项 3。

**射程**：四层图不重排：D21（权威态与派生态的分界） 已定项 10 把态别改成逐结构判，第 4 层里记账树的码 2 节点是权威态、别的树的内部节点是派生态，没有一层是纯的，所以退出冻结的单位是一个结构、不是一层（每个结构落哪一层、态别是什么、退不退出冻结，登记在 `.claude/kb/freeze-layer-membership.md`，门禁 doc-decisions 的 freeze-layer-membership 格判）；三个组件的 spec 文件都不存在；组件 ① 的冻结时刻是一次运行不是一次提交，已定项 5 那道闸对它不点火。树表单元与实例表单元落哪一层还没有条款。

**依据**：无实验：分层与依赖顺序是从「哪一样事后加不进去」推出来的排序，没有可量的量；止损规则那一条的对象（D9（加密） 四项预留）在 D9（加密） 已定项 12。主 agent 与用户定案，原话在变更史。

**欠**：C119（冻结组件没有 spec 文件）；C225（组件的冻结时刻绕开门禁触发）；C246（树表单元没有冻结层归属）。

````

**出处 `.claude/kb/decisions/15-格式冻结政策.md:163-180`（整段抄，未转述）**

```markdown
#### 已定项 8：冻结之后失去哪些自由

**定案**：

| 软格式期的自由 | 冻结后变成什么 |
|---|---|
| 结构「拆了重做」 | 已冻结字段的宽度 / 存在性 / 含义不能再改；只能新增，退役字段走「保留读支持、停止写」 |
| 发现设计错了直接改正文不留兼容 | 决策记录层面仍可推翻，但**盘上字节层面从 0 兼容成本变成非零** |
| mkfs 自动选取规则可随时换算法 | 已冻结组件的选取规则本身也被钉死；换算法会让新旧镜像默认值不同，必须体现在新格式版本号下 |
| 实验结果不理想就直接改格式重定 | 只能通过新 feature bit 引入新策略，并保留旧值的解释能力 |
| checker 可以跟着格式自由改 | **对已冻结部分，checker 的判断变成事实上的兼容性契约**——悄悄改 checker 对冻结字段的解释，和悄悄改格式一样危险 |

**射程**：只说冻结之后哪些动作不再合法，不说某一次具体改动该判哪一档——那归已定项 9。第一个外部用户出现之前格式是软的，这张表在那之前一条都不生效（`.claude/rules/format-evolution.md`）。

**依据**：无实验：它是「对外的永久承诺改不掉」这条约束的逐条展开，没有可量的量。最后一行那条（checker 的判断也成契约）与 D13（验证路线） 的「checker 即规范」同源。主 agent 定案。

**欠**：无。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:181-205`（整段抄，未转述）**

```markdown
#### 已定项 9：冻结后的改动判定表

**定案**：

| 改动类型 | 旧读者会怎样 | 哪档 | bump | 理由 |
|---|---|---|---|---|
| 新增字段在预留区 / TLV 尾部 | 忽略，继续正常工作 | compat | 否 | 没有语义依赖的纯加法，「忽略陌生字段」这条安全网够用 |
| 新增字段，缺了会让旧读者对**另一个已有字段**算错 | **算错，可能损坏数据** | incompat | 是 | 语义一旦耦合，安全网失效 |
| 扩大已有字段宽度 | 按旧宽度解析，越界读 / 错位 | incompat | 是 | 宽度是布局的一部分，旧代码的偏移计算全错 |
| 改变已有字段含义 | 按旧含义解读，**不报错**地产生错误行为 | incompat，且**弃用旧位置不复用** | 是 | 位置复用会让新旧读者无法区分，是最危险的一类。两个实例（D18（块里携带什么信息） 已定项 11）：新增一个单元类（登记表码 4–15）改的是类身份段的解析口径；给共同前缀偏移 7 里恒 0 的 flag 位赋义（旧读者对非 0 位一律拒收） |
| 新增记录类型（新 key 类型） | 只读遍历可能误判；整理 / 写可能误删看不懂的记录 | compat_ro | 是 | **「只读安全」与「可写安全」是两条独立保证**，必须分开判。⚠️ 打包记录单元内的打包记录类型**不走这一行**（D18（块里携带什么信息） 已定项 11）：跳过一个墓碑类或 inode 类记录会改变只读答案，默认档是 incompat；只有登记时能证明「跳过它不改变任何只读答案」的类型才降到 compat_ro，且 compat_ro 读者不许跑扫描重建 |
| 新增一棵 keyspace | 若树表 day-1 就是开放列表：只是看不到新树；若树数量硬编码：读不出树表 | **compat**（已定项 2 定树表 day-1 做成开放列表）。留了开放树表 → compat；没留 → incompat | 是 | **「树表是否可扩展」必须在冻结时决定**，不能等要加树才发现没留 |
| 改变排序规则 | 二分查找 / 遍历结果错，**不报错** | incompat | 是 | 排序规则是索引本体，日志结构节点的重排直接依赖它不变 |
| 改变校验和 / MAC 算法 | D9（加密） 的算法类型字段已留 → 遇到不认识的类型只能显式失败 | 已留 → compat_ro；未留 → incompat | 是 | D9（加密） 已定项 12 第 4 项正是为这类改动准备的通道 |
| 退役一个字段 | 不受影响，旧读者继续按旧方式解释 | compat，且**永久保留解析代码** | 否 | 落过盘的字段，只要还有一份旧镜像存在就必须一直能被解释 |

**射程**：判的是「旧读者会怎样」，不判「这个改动该不该做」。第一行那条（预留区里的新增是 compat）只在真有预留区时成立，索引节点头只有 2 字节预留，装得下的字段很少。

**依据**：

- E59（缓冲消息能不能只从单元重算）：跳过一个墓碑类记录会让已删的 key 全数复活、复活数恰等于删除次数——「打包记录类型不走 compat_ro 那一行、默认档是 incompat」这半句的量。
- 主 agent 定案，逐行的理由写在表里。

**欠**：C102（索引节点头没有预留区，加字段不是 compat）；C97（改字段含义而不改宽度，全仓没有一处会红）。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:206-220`（整段抄，未转述）**

```markdown
#### 已定项 10：feature bit 的位数、记账与分配权

**定案**：

- **位数**：三个 bitmap 各 **256 位**（`u64[4]`），存系统配置。位比字段宽度便宜得多（3×32 字节对系统配置可忽略），**预留要往宽了留**。
- **记账**：`.claude/kb/feature-bits.md` 列 `位号 | 类别 | 名称 | 引入版本 | 引入 commit | 状态 | 语义一句话`；位号的唯一登记位是已定项 4 那张表。
- **分配权是「谁先合并谁先占」，不是审批制**。门禁强制：位号严格递增、不许跳号占位、不许有代码引用了表里不存在的位号——机械仲裁。
- **位一旦用过不许回收。** 理由比 [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md) 编号不回收**更硬**：编号是检索键，回收误导的是「读文档的人或模型」；**feature bit 是磁盘上真实存在过的物理位**，回收后重新分配给新特性，一个旧镜像挂到新代码上这一位会被误读成「新特性 X 已启用」——**文件系统会真的做错事**（例如把不存在的加密特征当存在，去找不存在的密钥）。退役的位标为「含义永久锁定为退役前最后一次使用的语义」，不许赋予新特性。

**射程**：管位怎么分、怎么记、能不能回收，不管每一位的语义（归申请那一位的决策）。

**依据**：无实验：位数与不回收都是从「旧镜像会被新代码读到」这条物理事实推出来的，没有可量的量；256 位的空间代价对系统配置可忽略，这一格不比谁省。主 agent 定案。

**欠**：C183（每套布局一个 incompat 位而盘上差不出字节）。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:221-233`（整段抄，未转述）**

```markdown
#### 已定项 11：冻结要配的两条会红的检查

**定案**：

1. **格式冻结契约**：每个已冻结组件记 `spec_hash`；门禁重算并比对，不同则要求本次 diff **同时**满足「`feature-bits.md` 有新增行或 `version` 被显式改动」与「`decisions.md` 对应条目在同一 commit 里被改动」。缺一判红。
2. **向后兼容回归（黄金镜像）**：每次某组件版本变化时用**当时的代码**生成一份镜像并归档，门禁用**当前代码**去读全部历史镜像、跑 checker、比对期望内容摘要。镜像本体**不进 git**（`.claude/singlefs-ai-sop/rules/command-safety.md`：测试镜像不许放仓库内），仓库只提交清单 `.claude/kb/golden-images.toml`；镜像放 `${SINGLEFS_GOLDEN_DIR:-$HOME/.cache/singlefs-golden}`。清单**只增不删**——删掉某版本的回归等于悄悄放弃对它的兼容承诺，必须走决策记录。

**射程**：两条都还没实现，编号在欠账表里。第 2 条只比「今天的代码读不读得懂历史镜像」，不比「当初那份镜像写对没有」；第 1 条锚在声明上、不锚在字节上，两条都罩不到「第一份黄金镜像本身就是错代码写出来的」。

**依据**：无实验：定的是两道门禁检查该判什么，判别力自证要等它们有实现；它们要罩的失败形态（偷改格式、悄悄放弃兼容承诺）在 C9（冻结后偷改格式） 与 C227（黄金镜像的基准是它自己那份代码写的） 里各有一条。主 agent 定案。

**欠**：C9（冻结后偷改格式）；C227（黄金镜像的基准是它自己那份代码写的）。

```

**出处 `.claude/kb/decisions/28-mount-time-commitments.en.md:1-6`（整段抄，未转述）**

```markdown
## D28 Mount-Time Commitments —— Settled

[中文](28-挂载期承诺量.md) · **English** · [日本語](28-マウント時コミットメント量.ja.md)

D28 (Mount-Time Commitments) governs the **admission / acceptance cell**: space committed to itself during a writable mount period (not written to disk, recomputed on each mount), and how allocation admission readings are taken. Out of scope: writable mount admission conjunction is in D2 (RAID Stripe Policy) Settled Item 13; authoritative sentence for checkpoint reserve pool is in D23 (Journal Role & Format) "Deadlock 2 and spurious ENOSPC represent distinct failure modes" section, here only settling how it enters admission (Settled Item 4); admission inequality and in-flight synthesis originally resided in D3 (Space Allocation) Settled Items 4 / 6, where pointers remain while numbers stay untouched.

```

**出处 `.claude/kb/decisions/28-mount-time-commitments.en.md:7-141`（整段抄，未转述）**

````markdown
### Settled Items

| # | Item | Settlement |
|---|---|---|
| 1 | **Authoritative form of admission inequality** | Available computed per-device, each disk satisfied independently: five items take this disk's value, pending deletion usage (const 0), committed reserve, and reserve pool amortized across disks as sum of all replicas, deducting per-disk resident ring window **Status: Settled.** **Progress: Implemented.** |
| 2 | **In-flight synthesis of admission control** | Admission reading = published statistics − in-flight approved; releases within window grant no credit; "unwinding itself" includes tombstones **Status: Settled.** **Progress: Implemented.** |
| 3 | **Space reservation for instance switching** | (N_switch + 1) × single switch worst-case volume (instance table chain rewrite + warm-up empty publish), computed per-device **Status: Settled.** **Progress: Implemented.** |
| 4 | **Checkpoint reserve pool inclusion in admission** | Reserve pool computed dynamically from current structures, no constant upper bound, counted as mount-time commitment, recorded by sum of all replicas, amortized and deducted into Settled Item 1 per device **Status: Settled.** **Progress: Implemented.** |

#### Settled Item 1: Authoritative form of admission inequality

<!-- gate:admission-formula -->

```
available(d) = capacity(d) − allocated(d) − unreclaimable(d) − mount_commitments(d) − abandoned_root_exclusive(d) − pending_delete_usage ÷ replicas − committed_reserve ÷ replicas − checkpoint_reserve ÷ replicas − ring_window(d)
```

**Settlement**: Allocation admission computes available space by this formula. It is the executable form of "any operation increasing space usage calculates beforehand how much is needed to unwind itself in the worst case" (D3 (Space Allocation) Item 3).

- **Computed per-device, each disk satisfied independently** (per-device conjunction): The formula computes one instance per disk d; admission requires every disk to suffice, refusing if a single disk is insufficient without subsidizing from other disks' surplus. The first five items (including switch reservation in mount-time commitments, abandoned root exclusive volume) take the disk's own value—two replicas land on separate disks, and one disk filling cannot be subsidized by another disk; the latter three items—pending deletion usage, committed reserve, checkpoint reserve pool—are pool-level commitment volumes, **recorded in bytes as sum of all replicas** (physical bytes, identical measurement basis as "allocated" in D5 (Snapshot / Space Accounting Mechanism) Settled Item 7), amortized to each disk deducting "replica sum ÷ replica count". This demarcation decides which statistics in D5 (Snapshot / Space Accounting Mechanism) Settled Item 4 carry device dimensions. "Demand" is measured in disk bytes, identical per disk (same measurement basis as per-disk physical bytes of "allocated" in D5 (Snapshot / Space Accounting Mechanism) Settled Item 7).
- **Mount-time commitments** (Item 5): Constant during mount period, in-memory only, not written to disk, recomputed on each mount, amortized per device. Two members: instance switch reservation (Settled Item 3, amortized per device) and checkpoint reserve pool (Settled Item 4, pool-level volume, recomputed on each publish, item 8 in formula, amortized and deducted per device).
- **Checkpoint reserve pool** (Item 8): Pool-level member of mount-time commitments, recomputed dynamically from sum of record tree heights + accounting nodes, recomputed on each publish, no constant upper bound, in-memory only; recorded by sum of all replicas, deducting "replica sum ÷ replica count" per disk (each block written at fixed points lands across two columns per D2 (RAID Stripe Policy) Settled Item 6, one on each disk); ensures checkpoint's own fixed points can be written out.
- **Abandoned root exclusive volume** (Item 9): Value = slot count of allocator's actually isolated set: slots referenced solely by abandoned roots (slots referenced by abandoned roots − slots referenced by roots in rollback candidate set, lower bound 0), plus slots pre-isolated during writable mount: placement entries named by unapplied journal records above high-water mark, placement entries in accounting of that publish, placement entries in accounting of roots in root ring newer than selected version, valid per instance table, and readable (including placements dynamically recomputed from central mapping when accounting unreadable per D23 (Journal Role & Format) Settled Item 14 "Pre-isolation during writable mount"), as well as fallback resurrection placements merged into that slot when newer root accounting unreadable but records intact per D16 (Publish Semantics) Settled Item 1 "Root slot read corrupted this time" row (rollback target R_old older than selected version, placements in its accounting) and broad protection placements (released placements in current accounting), regardless of whether referenced by roots in rollback candidate set, excluding only slots occupied by allocation bit (including released slots still within defer window; counted in "allocated", not counted twice here, unisolated at mount time, `crates/singlefs-core/src/allocator.rs` `DeviceFreeMap::isolate_the_slots_not_allocated`); allocator supplements isolation for unreleased portions upon reclaiming them during this mount via two paths: recording root within mount (`PoolAllocator::record_root_written_by_this_process`), raising F reclamation sequence (`PoolAllocator::reclaim_released_up_to_isolating_what_the_isolation_still_protects`) (D23 (Journal Role & Format) Settled Item 14 "Pre-isolation during writable mount"; verdicts `research/prompts/m3-batch10z-lm-code-r1-main-verification.md` section two K1-3, `research/prompts/m3-batch10z-no-code-r1-main-verification.md` section two K1-3), matching the allocator's actually isolated set; slots referenced by each root are read from its allocation record tree, without on-disk fields; computed on each mount and each raising of F, in-memory only, per device, cleared when abandoned root is overwritten by rotation (C762 (Isolation added for abandoned roots not recorded in root ring table never released on raising F) literal mismatch with item 9: abandoned roots read from disk and unrecorded in root ring table during raising F have supplemental isolation that is never released even after overwriting its slot, unfillable on reclamation, isolating whole segment, double-deducted with "allocated"); abandoned roots are produced solely by crash recovery (administrator rollback is forward publish during mount, abandoning no roots, D23 (Journal Role & Format) Settled Item 14). When shadow accounting is active, allocation record tree writes an extra leaf per disk, admission reading difference between sides = isolated slots + these nodes (report 21 "stop and hand over to main agent" item 8).
- **Pending deletion usage**: Constant 0 reserved bit—unreleased units of inodes undergoing deletion are already in "allocated", counting again would double-deduct.
- **Ring window** (resident deduction item): Safety margin for new writes across deletion batches, ring window = (root ring slot count 3 × S + 1) × W, S is slots per region in system configuration (S = 4 / 8 / 16 gives ring window 13W / 25W / 49W); same class as checkpoint reserve pool, in-memory only, not entering accounting rows, recomputed from geometry on each publish, deducted per disk. W is upper bound on new slots occupied per disk during one deletion-type publish, W = 24 + (6 + H_retire) + Σ_ℓ min(N_ℓ, 2·(K + 26)), K is deletion batch size, N_ℓ is combined positions covered by allocation record tree level ℓ across two disks, H_retire is internal level count of retirement tree (read from retirement tree root node code 2 header, 0 for zero root or root only); 6 + H_retire is upper bound on newly written retirement units in one publish: spanning at most two retirement containers, 2 × 2 + 1 + H_retire + 1 (two containers 2 slots each, root 1, internal levels H_retire, right half 1 from root splitting), regardless of cascading merges; this upper bound holds when deletion batches modify only trailing leaves (D8 (Core Index Structure) Settled Item 16). When deletion queue is non-empty, the check prior to writable mount counts ring window as available.

**Wiring** (Experiment 25, zero rounds attacked):

- Publish path judged in `transaction::prepare_the_version_publish`: after computing shape of this publish, before disk reading verification and allocator mutation.
- Writable mount judged in `mount::establish_instance`: before acquisition and dry run, demand at this moment recorded as 0 per disk.
- Rejection returns before any write: publish path reports `PublishError::SpaceAdmissionRefused`, writable mount reports `MountError::SpaceAdmissionRefusedBeforeAcquisition`.
- Demand = all new slots written by this publish (ordinary allocation plus fixed points written this time, excluding slots replaced this time); judgment conditions unchanged: publishes without ordinary allocation (empty publish, write row, warm-up, raising F) are not judged (user 2026-09-25 settled ND+A1n, attacked 1 round, safety design round 3 verdict `research/prompts/m2-safety-r3-main-verification.md`); deletion-type publishes (intent, batches, resume batches in mount) and administrator rollback do not judge formula, placement judgment for deletion-type publishes judged as usual; lineage containers (D18 (Information Carried in a Block) Settled Item 24) belong to fixed points, entering ckpt_cost Σ list. Reading takes allocator's count at that moment.
- ckpt_cost computed per Settled Item 4 Σ list.
- When publish invoked via mounted session (`crates/singlefs-core/src/mounted_session.rs`, Experiment 5), admission rejection or placement rejection causes session to push empty publish raising F per D16 (Publish Semantics) Settled Item 1 "Admission" row, then retry; writable mount pushes after row write and warm-up in `mount::establish_instance`.

**Scope**: Whether each term in formula is a maintained statistic is compared item by item in D5 (Snapshot / Space Accounting Mechanism) Settled Item 4 "Formula item-by-item comparison" table (sole comparison location; five exceptions—device capacity, mount-time commitments, abandoned root exclusive volume, checkpoint reserve pool, and ring window—are also written there); D5 (Snapshot / Space Accounting Mechanism) Settled Item 4 statistics list is judged complete relative to this formula. Statistics in formula read published values (mount-time commitments, abandoned root exclusive volume, checkpoint reserve pool, and ring window are in-memory quantities recomputed at mount or publish, not belonging here), yet admission occurs mid-window; reading synthesis belongs to Settled Item 2. Pending deletion usage constant 0, W upper bound, administrator rollback skipping formula, lineage containers entering Σ list, and ring window counted as available prior to mount with non-empty deletion queue attacked zero rounds; deletion-type publishes skipping formula attacked 1 round. The premise "deletion batch modifies trailing leaves only" failed on pre-batch 50 empty deletion paths: when emptying a leaf, `crates/singlefs-core/src/inode_tree.rs` `remove_record_from_leaf_containers` rewrote every leaf to its right by position; two sample replicas measured new slot counts exceeding W (100 MiB disk W = 36, emptying occupied 45 / 85 slots per disk; 4 GiB disk 30 leaves 62 slots, 135 leaves 272 slots); batch 50 restored this premise with M3 deletion path redesign: when emptying or absorbing, pieces shifted in position with unchanged content are copied from previous version (`InodeLeafContainerOrigin::CarriedFrom`), rewriting only leaves and ancestors with changed content; across 20-leaf and 40-leaf geometries on publish path, absorbing second leaf into first wrote 14 slots per disk identically (`m3-batch50-m3b-implementer-report.md`).

**Basis**: Why each term is in formula—

| Item | Why It Is in Formula |
|---|---|
| Capacity, **Allocated** | Audited object of I-3.1 (Allocated statistics tally correctly) |
| **Unreclaimable** | On zoned media, physical space pinned by active snapshot is zone-granularity while referenced blocks are block-granularity; [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md) verbatim requires "establish a separate unreclaimable quantity" |
| **Pending Deletion Usage** | I-3.4 (Available space deducts pending deletion usage); D5 (Snapshot / Space Accounting Mechanism) and D8 (Core Index Structure) batched deletion settles "pending deletion structures count as live references" |
| **Committed Reserve** | D3 (Space Allocation) Settled Item 15; D3 (Space Allocation) Settled Item 2 verbatim "tombstone is part of unwinding itself, counted into that quantity" |
| **Mount-Time Commitments** | User settlement (C126 (Worst-case volume for switch reservation has no measurement basis) round 1): switch reservation not written to disk, computed per device, cannot fit into "committed reserve" cell |
| **Abandoned Root Exclusive Volume** | User settlement (C318 (Units isolated by shadow accounting not included in admission inequality)): after C314 (Crash recovery falling back to old root allows reuse of units referenced by abandoned root) adopts shadow accounting, units referenced solely by abandoned root are free in current accounting, yet allocator must not take them until abandoned root departs root ring; they are neither in "allocated" nor in "defer pending release", overestimating availability if not deducted. Measurement basis see `research/prompts/m2-step45-code-r3-main-verification.md` section six 1 |
| **Checkpoint Reserve Pool** | User settlement (Settled Item 4) |
| **Ring Window** | User settlement K-R3-2 (Resident deduction item): safety margin for deletion batches does not enter committed reserve, established as separate term in formula, recomputed from geometry |

Experiments:

- E19 (Spurious ENOSPC under Defer Window): Checkpoint deadlocks when reserve pool does not enter admission—item 8.
- E82 (In-Flight Synthesis of Admission Control): Reading published values alone causes intra-window overselling—this formula must pair with Settled Item 2.
- E141 (Self-Proof of Mount Admission for Switch Reservation): Per-device judgment separates from pool-level scalar judgment on pools where one disk is short 1 block but sum of two disks suffices—per-device summation in parentheses.
- E150 (Rollback Reuse of Units Referenced by Abandoned Roots): Available space with item 9 equals slots allocator can issue; without it, isolated units are over-reported—item 9.
- User settlement 2026-09-23: Admission per-device conjunction (C355 (Checkpoint reserve pool deducted at pool level while fixed points land across two columns) selects Option D); pending deletion usage and committed reserve recorded by sum of all replicas (C375 (No terms on whether pending deletion usage and committed reserve recorded by replica or by sum), round 3 three-way writeback: single-replica reading immediately falsifies this formula and I-3.4 (Available space deducts pending deletion usage)); verdict `research/prompts/c355-c363-r3-main-verification.md` sections two, three, five; verbatim words and overturned 2026-09-13 words in changelog.
- Experiment 25 report `m2-e25-implementer-report.md`.
- Deleting "defer pending release", changing demand to all newly written slots: user 2026-09-25 settlement (verbatim words in changelog), safety design round 2 verdict `research/prompts/m2-safety-r2-main-verification.md` ("defer pending release" double-deducts with "allocated"), round 3 verdict `research/prompts/m2-safety-r3-main-verification.md`.
- Ring window, pending deletion usage const 0, deletion publishes and administrator rollback skipping formula, lineage containers entering Σ list, check prior to mount with non-empty deletion queue: `research/prompts/m3-delete-r3-main-verification.md` section three G1-how deletion publishes judge, G1-tag 4, G1-W upper bound, G1-window position, G1-remount mid-crash, G3-rollback six rows; W upper bound formula in defense report `m3-delete-r3-sonnet-output.md` section 3.1; user settlements K-R3-1 "trailing leaves only (recommended)", K-R3-2 "resident deduction item (recommended)", verbatim words in changelog.
- Root count in ring window reads root ring slot count 3 × S: `research/prompts/m3-blanks-r1-main-verification.md` section two root count in root ring row (1 round).
- Ring window W adds 6 + H_retire, emptying path rewriting each right piece premise fails: verdict `research/prompts/m3-retire-r1-main-verification.md` section two K4 "max retirement records per publish" and "24 slots in ring window W" rows, section three items 6, 7 (upper bound formula deduced, 0 rounds; emptying exceeding W measured on both sample replicas); user settlements "do both (recommended)" and "merge into M3 (recommended)", verbatim words in changelog.
- Item 9 value expanded to allocator's actually isolated set (adding unapplied records above high-water mark, slots unreferenced by roots in rollback candidate set): verdict `research/prompts/m3-c694-r2-main-verification.md` section two "K1 clause gap localization" row, section three item 3 (`crates/singlefs-core/src/admission.rs` `AdmissionReading::of_allocator` treats entire `device_map.isolated_slots()` as item 9), attacked zero rounds; pre-isolation no longer deducts slots referenced by roots in rollback candidate set: `research/prompts/m3-batch33t-code-r1-main-verification.md` section two "t1 C candidate exemption" and "t1 C blind spot" rows (same pattern as root ring track `research/prompts/m3-batch10y-code-r1-main-verification.md` K4-1), attacked zero rounds; pre-isolation further merges newer readable root accounting and formula-recomputed placements: verdict `research/prompts/m3-batch35-36-code-r1-main-verification.md` section two H1, H3 rows and section three item 3; user settlement 2026-10-08 "fix all four patterns" (verbatim words in changelog), measured solely on attacker model, attacked zero rounds.
- "Demand" measured in disk bytes, identical per disk: user settlement (verbatim words in changelog).

**Owed**: C70 (Cross-statistical sum unmonitored): write vector equation per this formula; C87 (Synthesized value of admission reading undefined).

#### Settled Item 2: In-flight synthesis of admission control

**Settlement**: Admission reading = published statistics − in-flight approved reservations; releases generated within window grant no credit (admission-side form of D16 (Publish Semantics) Settled Item 15); "how much needed to unwind itself in worst case" includes tombstone portion (executable reading of D3 (Space Allocation) Settled Item 2).

- In-flight overlay is pure in-memory derived state: cleared by definition after crash (all in-flight transactions voided), not written to disk, zero format bits.
- Upon instance switch reconstructing memory state, overlay recomputed from replay results: applying record registers named code 1 units as allocated; code 2 / 3 units written after selected root are unpublished per published predicate (D18 (Information Carried in a Block) Settled Item 11), unregistered, rewritten by redo; failed checkpoint fixed point allocations and transaction allocations with sequence > W voided, re-admitted upon redo.

**Scope**: The switch sentence simultaneously settles spatial reading of C284 (Applying a record has undefined behavior on pointer layer) for the switch cell; remainder of pointer layer remains under C284 (Applying a record has undefined behavior on pointer layer).

**Basis**:

- E82 (In-Flight Synthesis of Admission Control): Three intuitive implementations each produce countable violations—reading published values alone causes intra-window overselling; recording intra-window releases as available consumes blocks defer should hold (D16 (Publish Semantics) Settled Item 15 silently bypassed); omitting tombstones deadlocks all deletions on full disk.
- Switch sentence: User settlement following round 1 three-way argument for C126 (Worst-case volume for switch reservation has no measurement basis) (`research/prompts/c126-r1-main-verification.md`), verbatim words in changelog.

**Owed**: C87 (Synthesized value of admission reading undefined): Copy three red examples from E82 (In-Flight Synthesis of Admission Control) into admission path unit tests and crash-point replay; make overlay a test switch with forced zeroing (after zeroing, concurrent admission must oversell, check must catch—bidirectional proof); comment in `crates/singlefs-core/src/admission.rs` states "in-flight overlay unimplemented" (in-flight synthesis of admission reading lacks code today).

#### Settled Item 3: Space reservation for instance switching

**Settlement**: Reservation = (N_switch + 1) × single switch worst-case volume, computed per-device; single switch worst-case volume = instance table chain rewrite + warm-up empty publish.

- **Chain rewrite**: Replica count × 32768 × max(1, ⌈(rows0 + N_switch) / rows_per_piece⌉) bytes. Replica count = 2 (two position entry slots (two position entries) of instance table unit pointer, D22 (Synthesizing Unit Atomicity) Settled Item 7; two copies land on two disks, so per-device calculation is 32768 × piece count each); rows_per_piece = ⌊(32768 − 135) / row_width⌋ − 1 (chain pointer record is always last entry of a piece, D18 (Information Carried in a Block) Settled Item 11; row width 88 ⇒ 369 rows); rows0 = rows read at mount plus rows to write during row writing (new instance − max(selected root instance, 1), known at mount; every writable mount writes a row).
- **Warm-up**: At most R empty publishes × dynamically computed c_max (R = 3, c_max computed per publish per Settled Item 4); this half of reservation is (N_switch + 1) × R × c_max; initial runnable target mirrors two disks, empty publish c_max blocks occupying 1 on each disk, so each disk adds one share.
- **N_switch + 1 shares**: N_switch = 3 (D23 (Journal Role & Format) Settled Item 14); the extra share is metadata for the row-writing publish—row-writing is new instance's first publish, preceded by no empty publish raising F; user data redo in same publish follows admission, returning ENOSPC if insufficient. N_switch shares form sequentially accumulated upper bound: continuous switching requires previous root unpublished, reconstructed state originating from same root; previous chain page does not exist in reconstructed state, practically not accumulating.
- **Fixed point redo and data unit rewrite for seq > W require no extra space**: Switching = intra-mount recovery (D23 (Journal Role & Format) Settled Item 14), reconstructed state originating from selected root + records applied up to W; failed checkpoint fixed point units and seq > W transaction units never entered published allocation records, nor registered by application (switch sentence in Settled Item 2) ⇒ In reconstructed state they are free, redone in identical space using checkpoint reserve pool; seq > W user data allocation failure returns ENOSPC to caller per D23 (Journal Role & Format) Settled Item 14, without blocking switch.
- **Publish path uses rows0 + 1**: Switch reservation in publish admission computed by next writable mount's rows0 (this mount's rows0 + 1, next mount inevitably writes a row); writable mount's own judgment follows this mount's rows0 as usual (user 2026-09-26 settled, attacker measured on replica: reaching 366 instance table rows crashed, remount rejected 8 → 0; attacked zero rounds).
- **`df`**: Available space already deducts this item, `df` reports available (D3 (Space Allocation) Settled Item 9 Item 1); failure to deduct causes spurious ENOSPC (D16 (Publish Semantics) Settled Item 1).

**Scope**: Formula written assuming "two replicas land on separate disks", unambiguous only for two-disk pools (initial runnable target of D2 (RAID Stripe Policy) Settled Item 9); **unsettled which two disks replicas land on with three or more disks**, whether mount judgment requires "any two", "two with most free space", or "two allocator selects", nor which disk empty publishes land on. Version 1 does not run on zoned media, where rows0 is unbounded (C121 (Instance table chain length unbounded on zoned media)); on non-zoned media, rows0 is constrained by row reclamation, which requires "no read failure ∧ all disks online" and row deletion requires writable mount; near-full disk + bad sectors may trap pool in "reservation unobtainable ⇒ read-only" absorbing state (deduced, extra piece only every 369 rows); every writable mount writes a row, row reclamation requiring every root ring slot read successfully and self-proven (D18 (Information Carried in a Block) Settled Item 11 "Reclamation", cleaner `crates/singlefs-core/src/instance_table_sweep.rs` skips deletion accordingly); when a root slot repeatedly fails to read, returns all zeros, or fails self-proof on reread, rows only increase without deletion, this absorbing state no longer requiring near-full disks (C597 (Instance table rows increase without deletion, conditions for row deletion have no clause)). D16 (Publish Semantics) Settled Item 5 specifies T_dirty only as trigger condition, unsettled whether checkpoint open during redo blocks writers, a measurement basis gap in D16 (Publish Semantics).

**Basis**:

- E141 (Self-Proof of Mount Admission for Switch Reservation): Mount admission per this formula flips at quota between two pools differing by 1 block; when reservation set to 0, both writable; pool-level scalar cannot detect short disk; seven error patterns separated by sample points; sample points for per-disk quota and piece boundaries under new-pool file creation geometry on that page, constructed for two disks only.
- E148 (Commit Fixed Points Recomputed from Two Record Trees): Measurement basis for dynamically computed c_max, new-pool file creation scale numbers on that page.
- Three-way: C126 (Worst-case volume for switch reservation has no measurement basis) round 1 (`research/prompts/c126-r1-main-verification.md`), chain rewrite formula corrected by two cloud legs; extra share from user settlement after three rounds of three-way for C329 (Empty publish raising F prior to row-writing publish lacks check); warm-up half coupled with user settlement for D16 (Publish Semantics) Settled Item 8.
- Publish path uses rows0 + 1: User settlement, verbatim words in changelog; safety design round 3 verdict `research/prompts/m2-safety-r3-main-verification.md` section two "two structural divergences" row and section three.

**Owed**: C126 (Worst-case volume for switch reservation has no measurement basis): implementation differential and placement for three or more disks; C121 (Instance table chain length unbounded on zoned media); C597 (Instance table rows increase without deletion, conditions for row deletion have no clause).

#### Settled Item 4: Checkpoint reserve pool inclusion in admission

**Settlement**: Checkpoint reserve pool sets no constant upper bound, computed dynamically from current structures, sizing to whatever pool requires.

- **Form**: ckpt_cost = Σ (each record tree current height) + accounting tree nodes per publish + 1 (tree table), recomputed on each publish from tree heights at that moment; current height of each code 2 tree = level read from root node code 2 header at publish moment + 1 (level field in D18 (Information Carried in a Block) Settled Item 18), requiring no separate in-memory copy; how allocation record tree and extent tree heights are read per key-space geometry follows three-way `m2-keyspace-r1`. Unit is 16 KiB metadata block, multiplied by 16384 when substituted into byte formula of Settled Item 1 ("allocated" in formula is "allocated bytes", "capacity" is unit area size, D5 (Snapshot / Space Accounting Mechanism) Settled Item 7); runtime-computed quantity, no on-disk field.
  - Allocation record tree term computed by K1: 1 + Σ_disk Σ_{level L = 0 … root_level − 1} (positions covered by this disk's unit area [s, e) at level L = ⌊(e − 1) ÷ S_L⌋ − ⌊s ÷ S_L⌋ + 1, S_L = 812 × 169^L); root_level is level read from root node code 2 header (version with 0 tree table entries having written row takes pool geometry root_level), reading no state of this version, independent of ckpt_cost, permanently an upper bound: fixed point of leaves modified in one publish cannot converge (new segments opened when open segment cannot fit, falling back without full empty segments, replacing nodes placed elsewhere long ago; audit A4c measured 3–4 leaves modified on one disk), only entire tree serves as reliable upper bound; cost grows linearly with unit area: two 4 GiB disks 529 nodes, each disk deducting ~2.7% extra (~3.2% for 1 TiB). Central mapping tree term enters Σ, computed by existing deletion/insertion formula, deletion/insertion count taking "allocation record tree term + accounting tree node count + lineage container, lineage tree root each 1 key and lineage tree internal levels each 1 key written extra during administrator rollback publish + retirement container, retirement tree root each 1 key and retirement tree internal levels each 1 key" (retirement keys follow ckpt_cost retirement item, verdict `research/prompts/m3-retire-r1-main-verification.md` section six R2, deduced, zero rounds) (lineage container is 32 KiB unit, 1 key, counted by unit count, not block count), varying with allocation record tree term formula;
  - Accounting tree does not enter Σ, listed separately by nodes per publish;
  - Tree table listed separately, 1 unit per publish (D16 (Publish Semantics) Settled Item 9: rewritten on each publish, has no height);
  - Lineage container listed separately, computed by single lineage container COW with ancestors: container 2 blocks + lineage tree root 1 block + lineage tree internal level count; 3 blocks for lineage tree with zero root or root only, 4 blocks when growing 1 level (written during administrator rollback, D18 (Information Carried in a Block) Settled Item 24; single lineage container COW attacked zero rounds, including ancestors settled by user 2026-10-02, unattacked);
  - Retirement container listed separately, computed by single retirement container COW with ancestors: container 2 blocks + retirement tree root 1 block + retirement tree internal level count; 3 blocks for retirement tree with zero root or root only (written in final batch, D18 (Information Carried in a Block) Settled Item 11 type 3; user settlement "do both (recommended)", combined with retirement item of D3 (Space Allocation) Settled Item 2 tag 6, attacked zero rounds);
  - Instance table chain does not enter ckpt_cost, its overhead belonging to switch reservation of Settled Item 3.
- **Attribution**: Counted as mount-time commitments, entering Settled Item 1 formula (separately listed item 8), landing on latter three items side of Settled Item 1 demarcation: recorded by sum of all replicas (ckpt_cost × 16384 × replica count), each device deducting "replica sum ÷ replica count".
- Empty publish reserve pool 10 + 7 c_max in D16 (Publish Semantics) Settled Item 1 and switch reservation in D16 (Publish Semantics) Settled Item 8 both take identical dynamically computed c_max.

**Scope**: Authoritative sentence for reserve pool itself (`reserve ≥ ckpt_cost`, distinction between deadlock 2 and spurious ENOSPC) is in D23 (Journal Role & Format) Settled Item 24, here settling only how it enters admission. Clauses contain no constants: splitting/merging, stripe membership table, livelist did not enter E148 (Commit Fixed Points Recomputed from Two Record Trees) model, its 9 blocks being model numbers. Retirement container item covers only transient state of single publish (newly written retirement container and ancestors in one publish); permanently grown slots prepaid by retirement item of D3 (Space Allocation) Settled Item 2 tag 6, not in ckpt_cost; combining both under "delete → rollback" loop still exceeds, recorded in C617 (Tombstones left by rollback after deletion lack prepaid coverage).

**Basis**:

- E148 (Commit Fixed Points Recomputed from Two Record Trees): Fixed points during clustered commit blocks form a constant independent of user block count; when user blocks random or fully scattered, no available upper bound exists—hence dynamically computed, clauses containing no constants.
- E19 (Spurious ENOSPC under Defer Window): Checkpoint does not deadlock when reservation precisely equals ckpt_cost—reserve pool must enter admission.
- User settlement: No constant upper bound, dynamically computed, verbatim words in changelog.
- User settlement 2026-09-23: Per-device conjunction (C355 (Checkpoint reserve pool deducted at pool level while fixed points land across two columns) selects Option D), verbatim words and overturned 2026-09-13 words in changelog; verdict `research/prompts/c355-c363-r3-main-verification.md` section five question 2.
- User settlement 2026-09-24: Tree height read dynamically from root node header (verbatim words in changelog; three-way `research/prompts/m2-treesplit-r1-main-verification.md` T5).
- `research/prompts/c363b-r1-main-verification.md` section three V2-2, section four item 4.
- User settlement: Allocation record tree term replaced with K1 (entire tree), 2026-09-27 popup chose "K1 entire tree"; cost and candidate formula table in `m2-rev-a4c-implementer-report.md` section two "Candidate formulas" (K0 not upper bound, audit measured 3–4 leaves modified on one disk across three scenarios, fixed points non-convergent).
- User settlement 2026-10-02: Lineage container item "Option B including ancestors (recommended)", central mapping tree deletion/insertion count "Option B included together (recommended)", verbatim words in changelog; dispatch record `records/2026-10-01-删除与并发事务.md` "Sixteen Batch 8".
- Retirement container item: Verdict `research/prompts/m3-retire-r1-main-verification.md` section two K4 rows P-a, P-b and section six "do both" (listing separately like lineage, oversells even without rollback, measured on replica during first sampling, reproduced by main agent; combined pattern attacked zero rounds); user settlement "do both (recommended)", verbatim words from two popups in changelog.

**Owed**: C83 (Commit fixed points unanswered): record fixed point rounds after implementation, judging red if exceeding dynamically computed volume at that time; allocation record tree formula per K1 already written into `admission.rs` (audit A4d, `m2-rev-a4d-implementer-report.md`), three design issues handed to main agent recorded as owed: C573 (Fixed point upper bound omitted extra deletions/insertions from publish with ordinary allocation), C574 (Landing check fails to verify two-slot alignment and same-slot across disks), C575 (Single error variant holds payloads of two distinct criteria), C576 (Three test cases red due to premature admission rejection, remedy options open).

````

**出处 `.claude/kb/decisions/28-mount-time-commitments.en.md:142-145`（整段抄，未转述）**

```markdown
## Historical Versions

Historical entries for D28 (Mount-Time Commitments) are centralized in [decisions-history.md](../decisions-history.md).

```

**出处 `.claude/kb/decisions/28-マウント時コミットメント量.ja.md:1-6`（整段抄，未转述）**

```markdown
## D28 マウント時コミットメント量 —— 確定

[中文](28-挂载期承诺量.md) · [English](28-mount-time-commitments.en.md) · **日本語**

D28 (マウント時コミットメント量) は**受入（アドミッション）枠**を管轄する：書き込み可能マウントがマウント期間中に自身へコミットする領域（ディスクに永続化せず、マウントごとに再計算）、および割り当て受入における数値の読み取り方法。管轄外：書き込み可能マウントの受入連言条件は D2 (RAID ストライプ方針) 確定項目 13；checkpoint 保持プールの権威ある記述は D23 (ジャーナルの役割と形式)「デッドロック2と見かけの ENOSPC の異種性」節にあり、ここでは受入への算入方法のみを定める（確定項目 4）；受入不等式とインフライト合成の原規定は D3 (領域割り当て) 確定項目 4 / 6 にあり、そこにはポインタのみを残し番号は維持する。

```

**出处 `.claude/kb/decisions/28-マウント時コミットメント量.ja.md:7-141`（整段抄，未转述）**

````markdown
### 確定項目

| # | 項目 | 決定事項 |
|---|---|---|
| 1 | **受入不等式の権威形式** | 利用可能領域をデバイスごとに計算し各ディスクが個別に満足：5項はこのディスク自体の値をとり、削除待ち占有（恒常的に0）、コミット済み予約、保持プールは全レプリカ合算値として各ディスクへ按分控除、さらに各ディスクの常駐リングウィンドウを控除 **状態: 確定。** **進捗: 実装済み。** |
| 2 | **受入のインフライト合成** | 受入読み取り値 = パブリッシュ済み統計量 − インフライト承認済み予約；ウィンドウ内の解放はクレジットを付与しない；「自身の巻き戻し」は墓標を含む **状態: 確定。** **進捗: 実装済み。** |
| 3 | **インスタンス切り替え時の空間予約** | (N_switch + 1) × 1回切り替えの最悪ケース量（インスタンス表チェーン書き直し + ウォームアップ空パブリッシュ）、デバイスごとに算出 **状態: 確定。** **進捗: 実装済み。** |
| 4 | **checkpoint 保持プールの受入算入** | 保持プールは当時の構造からその都度算出し定数上限を設けず、マウント時コミットメント量として計上、全レプリカ合算値として記録しデバイスごとに按分して確定項目 1 へ控除算入 **状態: 確定。** **進捗: 実装済み。** |

#### 確定項目 1: 受入不等式の権威形式

<!-- gate:admission-formula -->

```
利用可能(d) = 容量(d) − 割り当て済み(d) − 回収不能(d) − マウント時コミットメント量(d) − 放棄ルート排他占容量(d) − 削除待ち占有 ÷ レプリカ数 − コミット済み予約 ÷ レプリカ数 − checkpoint 保持プール ÷ レプリカ数 − リングウィンドウ(d)
```

**決定事項**: 割り当て受入はこの式に従って利用可能領域を計算する。「空間占有を増加させるいかなる操作も、受付前に最悪ケースで自身を巻き戻すために必要な量を計算する」（D3 (領域割り当て) 項目 3）の実行可能形式である。

- **デバイスごとに計算し、各ディスクが個別に満足**（デバイスごとの連言）：式は各ディスク d ごとに個別に計算され、受入にはすべてのディスクが足りていることが必要であり、1台でも不足すれば拒絶され、他のディスクの余剰で補うことはしない。先頭5項（マウント時コミットメント量内の切り替え予約、放棄ルート排他占容量を含む）はそのディスク自身の値をとる——2つのレプリカはそれぞれ別ディスクへ配置され、1台の満杯を他で補填できない；後続3項——削除待ち占有、コミット済み予約、checkpoint 保持プール——はプール全体のコミットメント量であり、**全レプリカ合算のバイト数として記録**（物理バイト、D5 (スナップショット / 空間記帳メカニズム) 確定項目 7 の「割り当て済み」と同一の計測条件）、各ディスクへ「レプリカ合算 ÷ レプリカ数」を按分控除する。この境界線が D5 (スナップショット / 空間記帳メカニズム) 確定項目 4 でどの統計量にデバイス次元を持たせるかを決定する。「要求量」はディスクバイト単位で計上され各ディスク同一（D5 (スナップショット / 空間記帳メカニズム) 確定項目 7 の各ディスク物理バイト「割り当て済み」と同一の計測条件）。
- **マウント時コミットメント量**（第5項）：マウント期間中の定数、メモリ上にのみ存在しディスクに書かず、マウントごとに再計算されデバイスごとに按分。内訳は2つ：インスタンス切り替え予約（確定項目 3、デバイス按分）および checkpoint 保持プール（確定項目 4、プール全体量、パブリッシュごとにその都度計算、式中では第8項として単独掲記しデバイス按分控除）。
- **checkpoint 保持プール**（第8項）：マウント時コミットメント量のプール級構成員であり、当時の各レコードツリーの高さの合計 + 会計ノードからその都度計算、パブリッシュごとに再計算、定数上限なし、メモリ上にのみ常駐；全レプリカ合算値として記録し、各ディスクへ「レプリカ合算 ÷ レプリカ数」を控除（固定点で書き出される各ブロックは D2 (RAID ストライプ方針) 確定項目 6 に従い2列に配置先を持ち、各ディスクへ1部ずつ配置）；checkpoint 自身の固定点が書き出せることを保証する。
- **放棄ルート排他占容量**（第9項）：値 = アロケータが実際に隔離している集合のスロット数：放棄されたルートのみから参照されるスロット（放棄ルート参照スロット − ロールバック候補集合内のルート参照スロット、下限0）、および書き込み可能マウント時に事前隔離されたスロット：高水位より上で適用されていないジャーナルレコードが指定する配置先、そのパブリッシュの会計内の配置先、ルートリング内で選択版より新しくインスタンス表上有効で読み出し可能なルートの会計内の配置先（会計読み出し不能時に D23 (ジャーナルの役割と形式) 確定項目 14「書き込み可能マウント時の事前隔離」に従い中央マッピングから再計算された配置先も含む）、およびその新しいルートの会計が読めずレコードが揃っている場合に D16 (パブリッシュセマンティクス) 確定項目 1「ルートスロットの読み取り破損」に従いそのスロットへ併合されるロールバック復活分（選択版より古いロールバック目標 R_old の会計内配置先）と広域保護分（現行会計内の解放済み配置先）、これらがロールバック候補集内のルートから参照されているかを問わず合算し、割り当て位置に存在するスロット（解放済みで defer ウィンドウ内にあるものを含む；これらは「割り当て済み」に計上され二重計算を避けるためマウント時点では隔離しない、`crates/singlefs-core/src/allocator.rs` の `DeviceFreeMap::isolate_the_slots_not_allocated`）のみを除外；本マウント中にそれらを回収する際、アロケータは未解放分を補充隔離する。2経路あり：マウント内ルート記録時（`PoolAllocator::record_root_written_by_this_process`）、F 引き上げ系列の回収（`PoolAllocator::reclaim_released_up_to_isolating_what_the_isolation_still_protects`）（D23 (ジャーナルの役割と形式) 確定項目 14「書き込み可能マウント時の事前隔離」；判定 `research/prompts/m3-batch10z-lm-code-r1-main-verification.md` 第2節 K1-3、`research/prompts/m3-batch10z-no-code-r1-main-verification.md` 第2節 K1-3）、アロケータの実隔離集合と同一値；各ルート参照スロットはその割り当てレコードツリーから読みディスク上のフィールドは追加しない；マウントごとおよび F 引き上げごとに計算されメモリのみに常駐、デバイスごとに計算、放棄ルートがローテーションで上書きされた際にゼロクリアされる（C762 (F引き上げによるルートリング表未記録の放棄ルートに対する補充隔離が解放されない問題) の形態は第9項の字面と不一致：F 引き上げ時にディスクから読み出されルートリング表に未記録の放棄ルートについて、補充された隔離はそのスロットを書き潰しても解放されず、回収時に補填不能となり全区間隔離が継続し「割り当て済み」と二重控除）；放棄ルートはクラッシュリカバリでのみ生成される（管理者ロールバックはマウント中の前進パブリッシュでありルートを放棄しない、D23 (ジャーナルの役割と形式) 確定項目 14）。シャドウ会計が有効な場合、割り当てレコードツリーは両ディスクで葉を1枚ずつ多く書き、受入読み取り値の差異 = 隔離スロット + これらノード（実験21報告「停止しメイン agent へ引き渡し」第8条）。
- **削除待ち占有**: 恒常的に0の予約枠——削除進行中 inode の未解放ユニットは元より「割り当て済み」に含まれており、再度計上すると重複控除となる。
- **リングウィンドウ**（常駐控除項目）：削除各バッチの新書き込みに対する安全マージン、リングウィンドウ = (ルートリングスロット数 3 × S + 1) × W、S はシステム構成の区ごとのスロット数（S = 4 / 8 / 16 のときリングウィンドウ 13W / 25W / 49W）；checkpoint 保持プールと同種でありメモリ上にのみ常駐、会計行へ入らず、パブリッシュごとに幾何形状から算出し各ディスクへこの分を控除。W は1回の削除系パブリッシュでディスクごとに新規占有されるスロット数の上限値、W = 24 + (6 + H_退役) + Σ_ℓ min(N_ℓ, 2·(K + 26))、K は削除バッチサイズ、N_ℓ は割り当てレコードツリー第 ℓ 層が2枚のディスク合計でカバーする位置数、H_退役 は退役ツリーの内部層数（退役ツリーのルートノードコード2ヘッダから読み出し、ルートなしまたはルートのみのときは0）；6 + H_退役 は1回のパブリッシュで新しく書かれる退役ユニット上限：最大2枚の退役コンテナに跨がり、2 × 2 + 1 + H_退役 + 1（コンテナ2スロット×2、ルート1、内部層 H_退役、ルート満杯での成長による右半分1）、マージ連鎖の有無を問わない；この上限は削除バッチが末尾の葉のみを変更する場合に成立（D8 (コアインデックス構造) 確定項目 16）。削除キューが非空のとき、書き込み可能マウント前の判定はリングウィンドウを動用可能と見なす。

**配線**（実験25、攻撃0ラウンド）：

- パブリッシュパスは `transaction::prepare_the_version_publish` 内で判定：今回のパブリッシュ形状を算定後、ディスク読み出し照合とアロケータ変更の前。
- 書き込み可能マウントは `mount::establish_instance` 内で判定：番号取得とドライランの前、この瞬間の要求量はディスクごとに0と記録。
- 拒否時はあらゆる書き込みの前に復帰：パブリッシュパスは `PublishError::SpaceAdmissionRefused`、書き込み可能マウントは `MountError::SpaceAdmissionRefusedBeforeAcquisition` を返す。
- 要求量 = 今回のパブリッシュで新しく書かれる全スロット（通常割り当て + 今回書き出される固定点、今回の差し替えスロットは加算しない）；判定要否は従前通り：通常割り当てのないパブリッシュ（空パブリッシュ、行書き込み、ウォームアップ、F 引き上げ）は判定しない（ユーザー 2026-09-25 決定 ND+A1n、攻撃1ラウンド、安全性設計ラウンド3判定 `research/prompts/m2-safety-r3-main-verification.md`）；削除系パブリッシュ（意図、各バッチ、マウント内継続バッチ）および管理者ロールバックも式を判定しない（削除系パブリッシュの配置先判定は通常通り実施）；系譜コンテナ（D18 (ブロックに含める情報) 確定項目 24）は固定点に帰属し ckpt_cost の Σ リストへ入る。読み取り値はその瞬間のアロケータのカウント値をとる。
- ckpt_cost は確定項目 4 の Σ リストに従い計算。
- マウントセッション経由のパブリッシュ（`crates/singlefs-core/src/mounted_session.rs`、実験5）時、受入拒否または配置先拒否はセッションが D16 (パブリッシュセマンティクス) 確定項目 1「受入」行に従い F 引き上げの空パブリッシュを発行し再試行；書き込み可能マウントは `mount::establish_instance` での行書き込みおよびウォームアップ後に発行。

**適用範囲**: 式中の各項が維持される統計量であるかどうかは、D5 (スナップショット / 空間記帳メカニズム) 確定項目 4 の「式各項対照」表に逐項記載されている（唯一の対照位置であり、5つの例外——デバイス容量、マウント時コミットメント量、放棄ルート排他占容量、checkpoint 保持プール、リングウィンドウ——もそこに記載）；D5 (スナップショット / 空間記帳メカニズム) 確定項目 4 の統計量リストはこの式に対して完全であると判定。式中の統計量はパブリッシュ済み数値を読み取るが（マウント時コミットメント量、放棄ルート排他占容量、checkpoint 保持プール、リングウィンドウはマウント時またはパブリッシュごとに計算されるメモリ量でありこれに含まれない）、受入はウィンドウの中間で発生するため、読み取り値の合成手法は確定項目 2 に帰属する。削除待ち占有の恒常的0、W 上限、管理者ロールバックの式判定スキップ、系譜コンテナの Σ リスト算入、削除キュー非空時のマウント前リングウィンドウ動用可能見なしは攻撃0ラウンド；削除系パブリッシュの式判定スキップは攻撃1ラウンド。「削除バッチが末尾の葉のみを変更する」という前提はバッチ50以前の全削除パスでは不成立であった：葉を1枚空にする際 `crates/singlefs-core/src/inode_tree.rs` の `remove_record_from_leaf_containers` がその右側の全葉を位置順に再書き込みしており、サンプリングで新占有スロット数が W を超過することが測定された（100 MiB ディスク W = 36 で全削除時にディスクあたり 45 / 85 スロット占有；4 GiB ディスク 30枚の葉で 62 スロット、135枚で 272 スロット）；バッチ50で M3 削除パス改修に伴いこの前提を回復：空化・吸収時に右側で位置移動し内容不変の断片は前バージョンをそのままコピー（`InodeLeafContainerOrigin::CarriedFrom`）、内容変更の葉と祖先のみ再書き込み；パブリッシュパス上の20枚・40枚の幾何形状下で第1葉が第2葉を吸収する際、各ディスクの新規書き込みは等しく14スロットとなった（`m3-batch50-m3b-implementer-report.md`）。

**根拠**: 各項が式中に存在する理由——

| 項目 | 式中に存在する理由 |
|---|---|
| 容量、**割り当て済み** | I-3.1（割り当て済み統計の整合性）の監査対象 |
| **回収不能** | zoned メディアにおいてアクティブスナップショットが固定する物理領域は zone 粒度であり参照ブロック数はブロック粒度であるため、[invariants/README.md](../../../crates/singlefs-checker/invariants/README.md) が逐字で「別個の回収不能量を設ける」ことを要求 |
| **削除待ち占有** | I-3.4（利用可能領域からの削除待ち占有控除）；D5 (スナップショット / 空間記帳メカニズム) および D8 (コアインデックス構造) のバッチ削除で「削除待ち構造は生存参照とみなす」と確定 |
| **コミット済み予約** | D3 (領域割り当て) 確定項目 15；D3 (領域割り当て) 確定項目 2 が逐字で「墓標は自身の巻き戻しの一部であり、その量へ算入」と規定 |
| **マウント時コミットメント量** | ユーザー決定（C126 (インスタンス切り替え予約の最悪ケース量に計測条件なし) 第1ラウンド）：インスタンス切り替え予約はディスクに永続化せずデバイスごとに算出し、「コミット済み予約」枠には収まらない |
| **放棄ルート排他占容量** | ユーザー決定（C318 (シャドウ会計で隔離されたユニットが受入不等式に未算入)）：C314 (クラッシュリカバリで旧ルートへ後退した際放棄ルート参照ユニットを再利用可能) でシャドウ会計採用後、放棄ルートのみから参照されるユニットは現行会計上は空きだがアロケータはそれらの使用を禁止されており、放棄ルートがルートリングを離脱するまで解放されない；これらは「割り当て済み」にも「defer 解放待ち」にもなく、控除しなければ利用可能領域を過大評価する。計測条件は `research/prompts/m2-step45-code-r3-main-verification.md` 第6節 1 参照 |
| **checkpoint 保持プール** | ユーザー決定（確定項目 4） |
| **リングウィンドウ** | ユーザー決定 K-R3-2（常駐控除項目）：削除各バッチの新書き込みに対する安全マージンはコミット済み予約へ入れず、式中に別項目を立て幾何形状から算出 |

実験:

- E19 (defer ウィンドウ下での見かけの ENOSPC): 保持プールを受入へ算入しないと checkpoint が行き詰まる——第8項。
- E82 (受入のインフライト合成): パブリッシュ済み数値のみを読み取ると同ウィンドウ内で過剰販売が生じる——本式は確定項目 2 と連動必須。
- E141 (切り替え予約のマウント受入自己証明): デバイス別判定とプール級スカラー判定は、1台が1ブロック不足し2台合算では十分なプールで挙動が分かれる——括弧内でデバイスごとに合算。
- E150 (放棄ルート参照ユニットのロールバック再利用): 第9項を含む利用可能領域はアロケータが発行可能なスロット数と一致し、含まなければ隔離ユニットを過大計上——第9項。
- ユーザー決定 2026-09-23: 受入はデバイスごとの連言（C355 (checkpoint保持プールはプール単位で控除するが固定点は各ブロック2列に配置される問題) で丁を選択）；削除待ち占有とコミット済み予約は全レプリカ合算で記録（C375 (削除待ち占有とコミット済み予約を個別記録か合算記録かの条項欠落)、第3ラウンド三方書き戻し：1レプリカ読み取りでは本式および I-3.4（利用可能領域からの削除待ち占有控除）が直ちに偽となる）；判定 `research/prompts/c355-c363-r3-main-verification.md` 第2、3、5節、原文および覆された2026-09-13原文は変更履歴にある。
- 実験25報告 `m2-e25-implementer-report.md`。
- 「defer 解放待ち」の削除、要求量を新書き込み全スロットへ変更：ユーザー 2026-09-25 決定（原文は変更履歴）、安全性設計ラウンド2判定 `research/prompts/m2-safety-r2-main-verification.md`（「defer 解放待ち」と「割り当て済み」の二重控除）、ラウンド3判定 `research/prompts/m2-safety-r3-main-verification.md`。
- リングウィンドウ、削除待ち占有恒常0、削除系パブリッシュと管理者ロールバックの式判定スキップ、系譜コンテナの Σ リスト算入、削除キュー非空時のマウント前判定：`research/prompts/m3-delete-r3-main-verification.md` 第3節 G1-削除系パブリッシュ判定法、G1-ラベル4、G1-W 上限、G1-ウィンドウ位置、G1-削除中間クラッシュ再マウント、G3-ロールバックの6行；W 上限式は弁護側報告 `m3-delete-r3-sonnet-output.md` 3.1節；ユーザー決定 K-R3-1「末尾の葉のみ変更（推奨）」、K-R3-2「常駐控除項目（推奨）」、原文は変更履歴。
- リングウィンドウ内のルート数はルートリングスロット数 3 × S を読み取り：`research/prompts/m3-blanks-r1-main-verification.md` 第2節ルートリング内ルート数行（1ラウンド）。
- リングウィンドウ W へ 6 + H_退役 の加算、全削除パス右側葉再書き込み前提の不成立：判定 `research/prompts/m3-retire-r1-main-verification.md` 第2節 K4「1パブリッシュあたり最大何条の退役レコードか」「リングウィンドウ W 内の24スロット」行および第3節第6、7条（上限式は推論、0ラウンド；全削除での W 超過は2回ともサンプリングで確認）；ユーザー決定「双方実施（推奨）」「M3 改修へ合流（推奨）」、原文は変更履歴。
- 第9項の値をアロケータ実隔離集合へ拡張（高水位より上で未適用のレコード指定、ロールバック候補集合外のルート参照スロットを追加）：判定 `research/prompts/m3-c694-r2-main-verification.md` 第2節「K1条項空白同定」行および第3節第3条（`crates/singlefs-core/src/admission.rs` の `AdmissionReading::of_allocator` が `device_map.isolated_slots()` 全体を第9項として扱う）、攻撃0ラウンド；事前隔離部分からロールバック候補集合内ルート参照スロットの控除を廃止：`research/prompts/m3-batch33t-code-r1-main-verification.md` 第2節「t1 C 候補免除」および「t1 C 盲点」行（ルートリング系 `research/prompts/m3-batch10y-code-r1-main-verification.md` K4-1 と同型）、攻撃0ラウンド；事前隔離へ新しく読み取り可能なルートの会計とマッピング再計算分を合算：判定 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第2節 H1、H3 行および第3節第3条、ユーザー決定 2026-10-08「4形態すべて修正」（原文は変更履歴）、攻撃側モデル上でのみ測定、攻撃0ラウンド。
- 「要求量」はディスクバイト単位で計上、各ディスク同一：ユーザー決定（原文は変更履歴）。

**未済**: C70 (統計量横断の合計監視欠落): 本式に従いベクトル等式を記述； C87 (受入読み取りの合成値未定義)。

#### 確定項目 2: 受入のインフライト合成

**決定事項**: 受入読み取り値 = パブリッシュ済み統計量 − インフライト承認済み予約；ウィンドウ内で発生した解放はクレジットを付与しない（D16 (パブリッシュセマンティクス) 確定項目 15 の受入側形態）；「最悪ケースで自身を巻き戻すために必要な量」は墓標分を含む（D3 (領域割り当て) 確定項目 2 の実行可能解釈）。

- インフライト overlay は純粋なメモリ派生状態：クラッシュ後は定義上ゼロクリア（インフライトトランザクションは全破棄）、ディスクへ書かずフォーマットビット0。
- インスタンス切り替え時のメモリ状態再構築において overlay はリプレイ結果から再計算：レコード適用により指定されたコード1ユニットを割り当て済みとして登録；選択ルート以降に書き出されたコード2 / 3ユニットはパブリッシュ済み述語（D18 (ブロックに含める情報) 確定項目 11）に従い未パブリッシュとして未登録、redo により再書き込み；失敗した checkpoint の固定点割り当ておよびシーケンス > W のトランザクション割り当ては破棄、redo 時に再度受入を実施。

**適用範囲**: 切り替えの記述は同時に C284 (レコード適用がポインタ層で何を行うかの定義欠落) の切り替え枠における空間側解釈を確定；ポインタ層の他部分は依然 C284 (レコード適用がポインタ層で何を行うかの定義欠落) の管轄。

**根拠**:

- E82 (受入のインフライト合成): 3つの直感的実装がそれぞれ可算の違反を生む——パブリッシュ済み数値のみの読み取りは同ウィンドウ内過剰販売、ウィンドウ内解放を利用可能へ計上すると defer が抑止すべきブロックを消費（D16 (パブリッシュセマンティクス) 確定項目 15 が暗黙に迂回される）、墓標の忘却はディスク満杯後の削除全デッドロック。
- 切り替えの記述: C126 (インスタンス切り替え予約の最悪ケース量に計測条件なし) 第1ラウンド三方論証（`research/prompts/c126-r1-main-verification.md`）後のユーザー決定、原文は変更履歴。

**未済**: C87 (受入読み取りの合成値未定義): E82 (受入のインフライト合成) の3つの赤例を受入パスの単体テストおよびクラッシュポイント再生へ転記、overlay を強制ゼロクリア可能なテストスイッチとして構築（クリア後並行受入で過剰販売発生、検査が検出することを確認——双方向検証）；`crates/singlefs-core/src/admission.rs` のコメントに「インフライト overlay は未実装」（受入読み取り値のインフライト合成は現状コードなし）と明記。

#### 確定項目 3: インスタンス切り替え時の空間予約

**決定事項**: 予約量 = (N_switch + 1) × 1回切り替えの最悪ケース量、デバイスごとに算出；1回切り替えの最悪ケース量 = インスタンス表チェーン書き直し + ウォームアップ空パブリッシュ。

- **チェーン書き直し**: レプリカ数 × 32768 × max(1, ⌈(rows0 + N_switch) / 断片あたり行数⌉) バイト。レプリカ数 = 2（インスタンス表ユニットポインタの2つの位置エントリ、D22 (ユニット原子性の合成手法) 確定項目 7；2部が2枚のディスクへ落ちるためデバイスごとに 32768 × 断片数を計上）；断片あたり行数 = ⌊(32768 − 135) / 行幅⌋ − 1（チェーンポインタレコードは常に断片の末尾、D18 (ブロックに含める情報) 確定項目 11；行幅 88 ⇒ 369 行）；rows0 = マウント時に読み取った行数 + 行書き込みで書く行数（新インスタンス − max(選択ルートのインスタンス, 1)、マウント時に既知；書き込み可能マウントごとに必ず行書き込み）。
- **ウォームアップ**: 最大 R 回の空パブリッシュ × その都度算出 c_max（R = 3、c_max は確定項目 4 に従いパブリッシュごとに算出）、予約内のこの半分は (N_switch + 1) × R × c_max；初期実行可能目標は2枚のディスクが相互ミラーであり、空パブリッシュの c_max ブロックは各ディスクへ1部ずつ占有するため、デバイスごとに1部ずつ加算。
- **N_switch + 1 部**: N_switch = 3（D23 (ジャーナルの役割と形式) 確定項目 14）；余分の1部は行書き込みパブリッシュのメタデータ用——行書き込みは新インスタンスの初回パブリッシュであり、事前に F 引き上げ空パブリッシュを発行しない；同パブリッシュ内のユーザーデータ redo は通常通り受入を経て不足時 ENOSPC を返却。N_switch 部は逐次累加上限：連続切り替えの前提は前回のルートが未パブリッシュで再構築状態が同一ルートから生じることであり、前回のチェーンページは再構築状態に存在せず、実際には累積しない。
- **固定点 redo および seq > W のデータユニット再書き込みは追加領域不要**: 切り替え = マウント内リカバリ（D23 (ジャーナルの役割と形式) 確定項目 14）、再構築状態は選択ルート + W まで適用されたレコードから生成；失敗した checkpoint の固定点ユニットおよび seq > W トランザクションのユニットはパブリッシュ済み割り当てレコードへ一度も入っておらず、適用でも登録されない（確定項目 2 の切り替え記述） ⇒ 再構築状態においてそれらは空き領域であり、checkpoint 保持プールを用いて同一空間上で再実行；seq > W のユーザーデータ割り当て失敗は D23 (ジャーナルの役割と形式) 確定項目 14 に従い呼び出し元へ ENOSPC を返し、切り替えを阻止しない。
- **パブリッシュパスは rows0 + 1 を使用**: パブリッシュ受入内の切り替え予約は次回書き込み可能マウントの rows0 で計算（本マウントの rows0 + 1、次回マウントは必ず1行書くため）；書き込み可能マウント自身の判定は本マウントの rows0 に従う（ユーザー 2026-09-26 決定、攻撃側がレプリカ上で測定：インスタンス表行数が366の枠でクラッシュ後再マウント拒否 8 → 0；攻撃0ラウンド）。
- **`df`**: 利用可能領域は既に本項目を控除済み、`df` は利用可能領域を報告（D3 (領域割り当て) 確定項目 9 項目 1）；未控除時は見かけの ENOSPC となる（D16 (パブリッシュセマンティクス) 確定項目 1）。

**適用範囲**: 式は「2つのレプリカがそれぞれ別ディスクに配置される」前提で記述され、2台ディスクプールにおいてのみ曖昧さがない（D2 (RAID ストライプ方針) 確定項目 9 の初期実行可能目標）；**3台以上においてレプリカがどの2台に落ちるかは未定**、マウント判定が「任意の2台」「空き最多の2台」「アロケータが選択する2台」のいずれを要求するか未確定、空パブリッシュの配置先ディスクも未定。第1版は zoned メディア上では動作せず、zoned では rows0 が無界（C121 (zoned 上でインスタンス表チェーン長が無界)）；非 zoned では rows0 は行回収により抑制されるが、回収には「読み出し失敗なし ∧ 全ディスクオンライン」が必要かつ行削除には書き込み可能マウントが必要であり、満杯近接 + 不良セクタのプールでは「予約獲得不能 ⇒ 読み取り専用」が吸収状態となる可能性がある（推論、369行ごとに1断片増加）；書き込み可能マウントごとに行が書かれ、行回収はルートリング全スロットの読み出し成功と自己証明後に行われるため（D18 (ブロックに含める情報) 確定項目 11「回収」、クリーナー `crates/singlefs-core/src/instance_table_sweep.rs` はこれに従い削除しない）、1つのルートスロットが継続して読めない、全ゼロを返す、または再読でも自己証明を通過しない場合に行は単調増加し、この吸収状態は満杯近接を必要としない（C597 (インスタンス表行が削除されず単調増加する問題)）。D16 (パブリッシュセマンティクス) 確定項目 5 は T_dirty をトリガー条件と記述するのみで、redo 期間中に開放された checkpoint がライターをブロックするか否かは記述しておらず、D16 (パブリッシュセマンティクス) の計測条件空白である。

**根拠**:

- E141 (切り替え予約のマウント受入自己証明): 本式によるマウント受入は、1ブロックのみ異なる2プールにおいて割当枠で反転し、予約を0にすると双方書き込み可能、プール級スカラーでは不足ディスクを判定不能、7つの誤実装がサンプリング点で識別可能；新規プールでのファイル作成の幾何形状における各ディスク割当枠と断片境界のサンプリングはそのページにあり、2台ディスクのみ構築。
- E148 (2つのレコードツリーから再計算されるコミット固定点): その都度算出される c_max の計測条件、新規プールでのファイル作成規模の数値はそのページ。
- 三方: C126 (インスタンス切り替え予約の最悪ケース量に計測条件なし) 第1ラウンド（`research/prompts/c126-r1-main-verification.md`）、チェーン書き直し式は2本のクラウド脚が修正；余分の1部は C329 (行書き込みパブリッシュ前にF引き上げ空パブリッシュを発行する検査の欠落) の3ラウンド三方後のユーザー決定；ウォームアップの半分は D16 (パブリッシュセマンティクス) 確定項目 8 を伴うユーザー決定。
- パブリッシュパスで rows0 + 1 を使用: ユーザー決定、原文は変更履歴；安全性設計ラウンド3判定 `research/prompts/m2-safety-r3-main-verification.md` 第2節「2つの構造的分岐」行および第3節。

**未済**: C126 (インスタンス切り替え予約の最悪ケース量に計測条件なし): 実装側の突き合わせおよび3台以上ディスクでの配置先；C121 (zoned 上でインスタンス表チェーン長が無界)；C597 (インスタンス表行が削除されず単調増加する問題)。

#### 確定項目 4: checkpoint 保持プールの受入算入

**決定事項**: checkpoint 保持プールは定数上限を設けず、当時の構造からその都度算出し、プール規模に応じて伸縮する。

- **形態**: ckpt_cost = Σ (各レコードツリーの現在の高さ) + 会計ツリーのパブリッシュあたりノード数 + 1 (ツリーテーブル)、当時のツリー高さからパブリッシュごとに再計算；各コード2ツリーの現在の高さ = パブリッシュの瞬間にそのルートノードコード2ヘッダから読み出した階層 + 1（D18 (ブロックに含める情報) 確定項目 18 の階層フィールド）、メモリ上に別コピーを保持しない；キースペースに応じた割り当てレコードツリーとエクステントツリーの高さの読み取り方法は三方 `m2-keyspace-r1` に従う。単位は 16 KiB メタデータブロックであり、確定項目 1 のバイト単位式へ代入する際は 16384 を乗算（式中の「割り当て済み」は「割り当て済みバイト」、「容量」はユニット領域サイズ、D5 (スナップショット / 空间記帳メカニズム) 確定項目 7）；実行時に計算される量でありディスク上にフィールドはない。
  - 割り当てレコードツリー項は K1 に従い計算: 1 + Σ_ディスク Σ_{階層 L = 0 … ルート階層 − 1} (このディスクのユニット領域 [s, e) が階層 L でカバーする位置数 = ⌊(e − 1) ÷ S_L⌋ − ⌊s ÷ S_L⌋ + 1, S_L = 812 × 169^L)；ルート階層はルートノードコード2ヘッダから読み出した階層（ツリーテーブル0エントリ、行書き込み済み版はプール幾何形状のルート階層を採用）、現版の状態を読まず ckpt_cost に依存せず恒常的に上限値：1回のパブリッシュで変更される葉数の不動点は収束しない（開放セグメントに収まらず新セグメント開設、完全空セグメントへの後退なし、過去に他所へ配置されたノードの差し替え；監査 A4c で1ディスクあたり3–4葉変更を測定）、信頼できる上限はツリー全体のみ；コストはユニット領域に比例して増加：4 GiB ディスク2台で 529 ノード、各ディスク約 2.7% 余分に控除（1 TiB で約 3.2%）。中央マッピングツリー項は Σ に入り既存の削除・挿入式に従い計算、削除・挿入数は「割り当てレコードツリー項 + 会計ツリーノード数 + 管理者ロールバックパブリッシュで余分に書かれる系譜コンテナ、系譜ツリーのルート各1キーおよび系譜ツリー内部層各1キー + 退役コンテナ、退役ツリーのルート各1キーおよび退役ツリー内部層各1キー」（退役キーは ckpt_cost の退役項に連動、判定 `research/prompts/m3-retire-r1-main-verification.md` 第6節 R2、推論、0ラウンド）（系譜コンテナは 32 KiB ユニット、1キー、ユニット数で計算しブロック数ではない）、割り当てレコードツリー項の式変更に連動；
  - 会計ツリーは Σ に含めず単独計上、パブリッシュあたりノード数による；
  - ツリーテーブルは単独計上、パブリッシュあたり 1 ユニット（D16 (パブリッシュセマンティクス) 確定項目 9：パブリッシュごとに再書き込み、高さなし）；
  - 系譜コンテナは単独計上、祖先を含む1回の系譜コンテナ COW として計算：コンテナ 2 ブロック + 系譜ツリールート 1 ブロック + 系譜ツリー内部層数；ルートなしまたはルートのみの系譜ツリーは 3 ブロック、1層成長時は 4 ブロック（管理者ロールバック時に書き込み、D18 (ブロックに含める情報) 確定項目 24；祖先を含む系譜コンテナ COW 1回としての計算は攻撃0ラウンド、祖先連動はユーザー 2026-10-02 決定、攻撃なし）；
  - 退役コンテナは単独計上、祖先を含む1回の退役コンテナ COW として計算：コンテナ 2 ブロック + 退役ツリールート 1 ブロック + 退役ツリー内部層数；ルートなしまたはルートのみの退役ツリーは 3 ブロック（最終バッチで書き込み、D18 (ブロックに含める情報) 确定項目 11 タイプ3；ユーザー決定「双方実施（推奨）」、D3 (領域割り当て) 確定項目 2 ラベル6の退役項と合算した形態は攻撃0ラウンド）；
  - インスタンス表チェーンは ckpt_cost に含めず、そのオーバーヘッドは確定項目 3 の切り替え予約に帰属。
- **帰属**: マウント時コミットメント量として計上され、確定項目 1 の式に入り（第8項として単独掲記）、確定項目 1 の境界線の後続3項側へ着地：全レプリカ合算値として記録（ckpt_cost × 16384 × レプリカ数）、各デバイスへ「レプリカ合算 ÷ レプリカ数」を控除。
- D16 (パブリッシュセマンティクス) 確定項目 1 の空パブリッシュ保持プール 10 + 7 c_max および D16 (パブリッシュセマンティクス) 確定項目 8 の切り替え予約は同一のその都度算出される c_max を採用。

**適用範囲**: 保持プール自体（`reserve ≥ ckpt_cost`、デッドロック2と見かけの ENOSPC の識別）の権威ある記述は D23 (ジャーナルの役割と形式) 確定項目 24 にあり、ここでは受入への算入方法のみを定める。条項内に定数は記載しない：分割マージ、ストライプメンバー表、livelist はいずれも E148 (2つのレコードツリーから再計算されるコミット固定点) のモデルに入っておらず、その 9 ブロックはモデル値である。退役コンテナ項は1回のパブリッシュの過渡状態のみをカバー（1回のパブリッシュで新しく書かれる退役コンテナとその祖先）；恒久的に増加するスロットは D3 (領域割り当て) 確定項目 2 ラベル6の退役項により前払いされ ckpt_cost には含まれない；「削除 → ロールバック」ループ下で双方が合算しても超過する問題は C617 (削除後ロールバックによる墓標の前払い不足) に記録。

**根拠**:

- E148 (2つのレコードツリーから再計算されるコミット固定点): クラスタ化コミットブロックにおいて固定点はユーザーブロック数に依存しない定数だが、ユーザーブロックがランダムまたは完全分散している場合利用可能な上限は存在しない——したがってその都度算出し、条項に定数を記載しない。
- E19 (defer ウィンドウ下での見かけの ENOSPC): 予約が正確に ckpt_cost と一致するとき checkpoint は行き詰まらない——保持プールは受入へ算入必須。
- ユーザー決定: 定数上限を設けずその都度算出、原文は変更履歴。
- ユーザー決定 2026-09-23: デバイスごとの連言（C355 (checkpoint保持プールはプール単位で控除するが固定点は各ブロック2列に配置される問題) で丁を選択）、原文および覆された2026-09-13原文は変更履歴；判定 `research/prompts/c355-c363-r3-main-verification.md` 第5節第2問。
- ユーザー決定 2026-09-24: ツリーの高さはルートノードヘッダからその都度読み取り（原文は変更履歴；三方 `research/prompts/m2-treesplit-r1-main-verification.md` T5）。
- `research/prompts/c363b-r1-main-verification.md` 第3節 V2-2、第4節第4条。
- ユーザー決定: 割り当てレコードツリー項を K1（ツリー全体）へ変更、2026-09-27 ポップアップで「K1 ツリー全体」を選択；コストと候補式一覧は `m2-rev-a4c-implementer-report.md` 第2節「候補式」（K0 は上限値でなく実測3シナリオで1ディスクあたり3–4葉変更、不動点非収束）。
- ユーザー決定 2026-10-02: 系譜コンテナ項「乙 祖先連動計算（推奨）」、中央マッピングツリー削除・挿入数「乙 一括合算（推奨）」、原文は変更履歴；ディスパッチ記録 `records/2026-10-01-削除与并发事务.md`「十六 バッチ 8」。
- 退役コンテナ項: 判定 `research/prompts/m3-retire-r1-main-verification.md` 第2節 K4 の P-a、P-b 行および第6節「双方実施」（系譜と同様に単独計上、ロールバックなしでも過剰販売、初サンプリングでレプリカ上測定、メイン agent が再実行再現；合算形態は攻撃0ラウンド）；ユーザー決定「双方実施（推奨）」、2回のポップアップ原文は変更履歴。

**未済**: C83 (コミット固定点への回答欠落): 実装後固定点ラウンド数を記録し、当時の算出量を超過した場合赤と判定；割り当てレコードツリーの K1 式は `admission.rs` へ実装済み（監査 A4d、`m2-rev-a4d-implementer-report.md`）、メイン agent へ引き渡された3設計課題を未済として記録：C573 (通常割り当てを伴うパブリッシュでの追加削除挿入が固定点上限から欠落)、C574 (配置チェックにおける2スロットアライメントおよび各ディスク同スロット照合の欠落)、C575 (単一エラー列挙子への2種の判定基準ペイロード同居)、C576 (早期受入拒否による3テストケース連鎖赤の改修法選択待ち)。

````

**出处 `.claude/kb/decisions/28-マウント時コミットメント量.ja.md:142-145`（整段抄，未转述）**

```markdown
## 履歴

D28 (マウント時コミットメント量) の履歴エントリは [decisions-history.md](../decisions-history.md) に集約されている。

```

**出处 `.claude/kb/decisions/28-挂载期承诺量.md:1-6`（整段抄，未转述）**

```markdown
## D28 挂载期承诺量 —— 已定

**中文** · [English](28-mount-time-commitments.en.md) · [日本語](28-マウント時コミットメント量.ja.md)

D28（挂载期承诺量） 管**准入这一格**：可写挂载在挂载期间承诺给自己的空间（不落盘、每次挂载重算），以及分配准入怎么读数。不管的：可写挂载的准入合取在 D2（RAID 条带策略） 已定项 13；checkpoint 保留池的权威句在 D23（journal 的角色与格式）「死锁 2 与假性 ENOSPC 是两种故障」一节，这里只定它怎么进准入（已定项 4）；准入不等式与在飞合成原住 D3（空间分配） 已定项 4 / 6，那两处只留指针、编号不动。

```

**出处 `.claude/kb/decisions/28-挂载期承诺量.md:16-73`（整段抄，未转述）**

````markdown
#### 已定项 1：准入不等式的权威形式

<!-- gate:admission-formula -->

```
可用(d) = 容量(d) − 已分配(d) − 不可回收(d) − 挂载期承诺量(d) − 被抛弃根独占量(d) − 待删占用 ÷ 副本数 − 已承诺预留 ÷ 副本数 − checkpoint 保留池 ÷ 副本数 − 环窗口(d)
```

**定案**：分配准入按这个式子算可用。它是「任何增加空间占用的操作，进门前先算最坏情况下解开自己需要多少」（D3（空间分配） 第 3 条）的可执行形式。

- **逐设备算，每块盘各自满足**（逐设备合取）：式子对每块盘 d 各算一份，准入要每一块盘都够，一块盘不够就拒，不拿别的盘的富余补。前五项（含挂载期承诺量里的切换预留、被抛弃根独占量）取这块盘自己的值——两份副本各落一块盘，一块盘满了不能靠别的盘补；后三项——待删占用、已承诺预留、checkpoint 保留池——是全池的承诺量，**按全部副本之和记字节**（物理字节，与 D5（快照 / 空间记账机制） 已定项 7 的「已分配」同一口径），摊到每块盘各扣「副本之和 ÷ 副本数」。这个分界决定 D5（快照 / 空间记账机制） 已定项 4 里哪几个统计量要带设备维。「需求」按盘字节计、每盘相同（与 D5（快照 / 空间记账机制） 已定项 7 的「已分配」逐盘物理字节同口径）。
- **挂载期承诺量**（第五项）：挂载期间常量、只住内存、不落盘、每次挂载重算，按设备分摊。成员两个：实例切换的预留（已定项 3，按设备摊）与 checkpoint 保留池（已定项 4，全池的量、每次发布现算，式子里单列第八项、逐设备摊扣）。
- **checkpoint 保留池**（第八项）：挂载期承诺量的池级成员，按当时各记录树的高之和 + 记账节点现算，每次发布重算、不设常数上限、只住内存；按全部副本之和记，摊到每块盘扣「副本之和 ÷ 副本数」（固定点写出的每一块按 D2（RAID 条带策略） 已定项 6 落两列，每块盘各落一份）；它保证 checkpoint 自己的固定点写得出去。
- **被抛弃根独占量**（第九项）：取值 = 分配器实际隔离的集合的槽数：只被被抛弃根引用的槽（被抛弃根引用的槽 − 回退候选集里的根引用的槽，下界 0），加可写挂载时预隔离的槽：水位之上没施加的 journal 记录点名的落点、那次发布的账里的落点、根环里比所选那一版新、按实例表有效、读得出的根的账里的落点（账读不出时照 D23（journal 的角色与格式） 已定项 14「可写挂载时的预隔离」从中央映射现算出来的落点也在内），以及那条较新的根账读不出而记录齐时照 D16（发布语义） 已定项 1「根槽这一次读坏」那一行并进它那一槽的回退复活那一份（回退目标 R_old 比所选那一版旧，它账里的落点）与宽护那一份（现行账里已释放的落点），不论它们有没有被回退候选集里的根引用，只除去分配位置着的槽（含已释放而还在 defer 窗口里的；那些算在「已分配」里，不在这里再算一次，挂载那一刻不隔离，`crates/singlefs-core/src/allocator.rs` 的 `DeviceFreeMap::isolate_the_slots_not_allocated`）；这次挂载里回收它们时分配器照还没解开的那一份补隔离，有两条路：挂载内记根时（`PoolAllocator::record_root_written_by_this_process`）、抬 F 那一串的回收（`PoolAllocator::reclaim_released_up_to_isolating_what_the_isolation_still_protects`）（D23（journal 的角色与格式） 已定项 14「可写挂载时的预隔离」；判决 `research/prompts/m3-batch10z-lm-code-r1-main-verification.md` 第二节 K1-3、`research/prompts/m3-batch10z-no-code-r1-main-verification.md` 第二节 K1-3），与分配器实际隔离的集合是同一个数；各条根引用的槽从它的分配记录树读、不加盘上字段；每次挂载与每次抬 F 时算出、只住内存、按设备算，被抛弃的根被轮转覆写时清零（C762（抬 F 给根环表没记着的被抛弃根补的隔离一直不放） 那一形与第九项字面不符：抬 F 那一串读盘得来、根环表没记着的被抛弃根，补的隔离写掉它那一槽也不放，回收时补不上、照旧整段隔离，与「已分配」双扣）；被抛弃的根只由崩溃恢复造出（管理员回退是挂着时的一次向前发布，不抛弃任何根，D23（journal 的角色与格式） 已定项 14）。影子账开着时，分配记录树两块盘各多写一片叶，准入读数两边之差 = 被隔离的槽 + 这几个节点（实二一报告「停下交主 agent」第 8 条）。
- **待删占用**：恒 0 的保留位——删除中 inode 没释放的单元本来就在「已分配」里，再记一次是重复扣。
- **环窗口**（常驻扣项）：删除各批新写的兜底，环窗口 = (根环槽数 3 × S + 1) × W，S 是系统配置每区槽数（S = 4 / 8 / 16 时环窗口 13W / 25W / 49W）；与 checkpoint 保留池同类，只住内存、不进记账行，每次发布按几何现算，每块盘各扣这一份。W 是一次删除类发布每盘新占槽数的上界，W = 24 + (6 + H_退役) + Σ_ℓ min(N_ℓ, 2·(K + 26))，K 是删除批大小，N_ℓ 是分配记录树第 ℓ 层两块盘合计罩到的位置数，H_退役 是退役树的内部层数（从退役树根节点码 2 头现读，零根与只有根时 0）；6 + H_退役 是一次发布新写的退役单元上界：至多跨两片退役容器，2 × 2 + 1 + H_退役 + 1（两片容器各 2 槽、根 1、内部层 H_退役、根满长高多出的右半 1），不论合并连不连锁；这个上界在删除批只改尾叶时成立（D8（核心索引结构） 已定项 16）。删除队列非空时，可写挂载前那一判把环窗口算作可动用。

**接线**（实二五，被攻过零轮）：

- 发布路径在 `transaction::prepare_the_version_publish` 里判：算定这次发布的形状之后、读盘核与动分配器之前。
- 可写挂载在 `mount::establish_instance` 里判：取号与预演之前，这一刻的需求逐盘记 0。
- 拒了在任何写之前返回：发布路径报 `PublishError::SpaceAdmissionRefused`，可写挂载报 `MountError::SpaceAdmissionRefusedBeforeAcquisition`。
- 需求 = 这次发布新写的全部槽（普通分配加这次写出的固定点，不加这次换下的槽）；判不判照旧：没有普通分配的发布（空发布、写行、暖机、抬 F）不判（用户 2026-09-25 定 ND+A1n，被攻过一轮，安全设计轮第三轮判决 `research/prompts/m2-safety-r3-main-verification.md`）；删除类发布（意图、各批、挂载里的续做批）与管理员回退也不判式子，删除类发布的落点那一判照判；谱系容器（D18（块里携带什么信息） 已定项 24）归固定点、进 ckpt_cost 的 Σ 名单。读数取分配器此刻的计数。
- ckpt_cost 照已定项 4 的 Σ 名单算。
- 发布经挂着的会话调（`crates/singlefs-core/src/mounted_session.rs`，实五）时，准入拒或落点被拒由会话照 D16（发布语义） 已定项 1「准入」那一行推抬 F 的空发布、再重发；可写挂载在 `mount::establish_instance` 写行与暖机之后推。

**射程**：式子里每一项是不是被维护的统计量，逐项对照写在 D5（快照 / 空间记账机制） 已定项 4 的「式子逐项对照」表里（那里是唯一的对照位，五处例外——设备容量、挂载期承诺量、被抛弃根独占量、checkpoint 保留池与环窗口——也写在那里）；D5（快照 / 空间记账机制） 已定项 4 的统计量清单相对这个式子判完备。式子里的统计量读的是已发布值（挂载期承诺量、被抛弃根独占量、checkpoint 保留池与环窗口是挂载时或每次发布现算的内存量，不在此列），准入却发生在窗口中间，读数怎么合成归已定项 2。待删占用恒 0、W 上界、管理员回退不判式子与谱系容器进 Σ 名单、删除队列非空时挂载前把环窗口算作可动用被攻过零轮；删除类发布不判式子被攻过一轮。「删除批只改尾叶」这个前提在第 50 批之前的删空路径上不成立：删空一片叶时 `crates/singlefs-core/src/inode_tree.rs` 的 `remove_record_from_leaf_containers` 曾按位置重写它右边的每一片，两次抽样副本都量到删空那次新占的槽数超过 W（100 MiB 盘 W = 36、删空那次每盘新占 45 / 85 槽；4 GiB 盘 30 片叶 62 槽、135 片 272 槽），第 50 批随 M3 改删除路径已恢复这个前提：删空、吸收一片时右边挪了位置、内容没变的片照抄上一版（`InodeLeafContainerOrigin::CarriedFrom`），只重写内容变了的叶与祖先；发布路径上 20 片与 40 片两种几何下第一片吸收第二片那一次每盘新写同为 14 槽（`m3-batch50-m3b-implementer-report.md`）。

**依据**：每一项为什么在式子里——

| 项 | 为什么在式子里 |
|---|---|
| 容量、**已分配** | I-3.1（已分配统计对得上） 的被审计对象 |
| **不可回收** | zoned 上一个活快照钉住的物理空间是 zone 粒度而引用的块数是块粒度，[invariants/README.md](../../../crates/singlefs-checker/invariants/README.md) 逐字要求「另立一个不可回收量」 |
| **待删占用** | I-3.4（可用空间扣待删占用）；D5（快照 / 空间记账机制） 与 D8（核心索引结构） 分批删除定「待删除结构算作活引用」 |
| **已承诺预留** | D3（空间分配） 已定项 15；D3（空间分配） 已定项 2 逐字「墓碑就是解开自己的一部分，计入那个量」 |
| **挂载期承诺量** | 用户定案（C126（切换预留的最坏量没有口径） 第一轮）：实例切换的预留不落盘、按设备算，塞不进「已承诺预留」那一格 |
| **被抛弃根独占量** | 用户定案（C318（影子账隔离的单元没进准入不等式））：C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元） 取影子账之后，只被被抛弃的根引用的单元在当前账里是空闲的、分配器却不许拿它们，直到被抛弃的根离开根环；它们既不在「已分配」里也不在「defer 待释放」里，不扣就高估可用。取值口径见 `research/prompts/m2-step45-code-r3-main-verification.md` 第六节 1 |
| **checkpoint 保留池** | 用户定案（已定项 4） |
| **环窗口** | 用户定案 K-R3-2（常驻扣项）：删除各批新写的兜底不进已承诺预留，在式子里另立一项、按几何现算 |

实验：

- E19（defer 窗口下的假性 ENOSPC）：保留池不进准入时 checkpoint 会卡死——第八项。
- E82（准入的在飞合成）：只读已发布值会同窗超卖——这个式子要配已定项 2。
- E141（切换预留的挂载准入自证）：按设备判与按池级标量判，在一块盘短 1 块而两块盘加起来够的池上分得开——括号里按设备求和。
- E150（回退复用被抛弃的根引用的单元）：带第九项的可用等于分配器发得出的槽数，不带就多报被隔离的单元——第九项。
- 用户定案 2026-09-23：准入逐设备合取（C355（checkpoint 保留池池级扣而固定点每块要落两列） 取丁）；待删占用与已承诺预留按全部副本之和记（C375（待删占用与已承诺预留按份还是按和记没有条款），第三轮三方写回：一份副本那一读法让这个式子与 I-3.4（可用空间扣待删占用） 当场为假）；判决 `research/prompts/c355-c363-r3-main-verification.md` 第二、三、五节，原话与被推翻的那句 2026-09-13 原话在变更史。
- 实二五报告 `m2-e25-implementer-report.md`。
- 删「defer 待释放」、需求改成新写的全部槽：用户 2026-09-25 定案（原话在变更史），安全设计轮第二轮判决 `research/prompts/m2-safety-r2-main-verification.md`（「defer 待释放」与「已分配」重复扣）、第三轮判决 `research/prompts/m2-safety-r3-main-verification.md`。
- 环窗口、待删占用恒 0、删除类发布与管理员回退不判式子、谱系容器进 Σ 名单、删除队列非空时挂载前那一判：`research/prompts/m3-delete-r3-main-verification.md` 第三节 G1-删除类发布怎么判、G1-标签 4、G1-W 上界、G1-窗口位置、G1-崩在删除中间的重挂载、G3-回退六行；W 上界的式子在辩方报告 `m3-delete-r3-sonnet-output.md` 3.1 节；用户定案 K-R3-1「只改尾叶（推荐）」、K-R3-2「常驻扣项（推荐）」，原话在变更史。
- 环窗口里的根数读根环槽数 3 × S：`research/prompts/m3-blanks-r1-main-verification.md` 第二节根环里的根数那一行（一轮）。
- 环窗口 W 加 6 + H_退役、删空路径重写右边每一片的前提不成立：判决 `research/prompts/m3-retire-r1-main-verification.md` 第二节 K4 的「一次发布至多几条退役记录」「环窗口 W 里那 24 槽」两行与第三节第 6、7 条（上界式子是推的、零轮；删空那次超 W 两次抽样都在副本上量到）；用户定案「两样都做（推荐）」「并进 M3 改（推荐）」，原话在变更史。
- 第九项的取值扩成分配器实际隔离的集合（加水位之上没施加的记录点名、不被回退候选集里的根引用的槽）：判决 `research/prompts/m3-c694-r2-main-verification.md` 第二节「K1 条款空白定位」那一行与第三节第 3 条（`crates/singlefs-core/src/admission.rs` 的 `AdmissionReading::of_allocator` 把 `device_map.isolated_slots()` 整个当第九项），被攻过零轮；预隔离那部分不再扣回退候选集里的根引用的槽：`research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节「t1 C 的候选豁免」与「t1 C 的盲区」两行（与根环线 `research/prompts/m3-batch10y-code-r1-main-verification.md` K4-1 同一形），被攻过零轮；预隔离再并上较新可读根的账与从映射现算的那一份：判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 H1、H3 两行与第三节第 3 条，用户定案 2026-10-08「四形都修」（原话在变更史），只在攻方模型上量过、被攻过零轮。
- 「需求」按盘字节计、每盘相同：用户定案（原话在变更史）。

**欠**：C70（跨统计量的和没人盯）：按这个式子写那条向量等式；C87（准入读的合成值无定义）。

````

**出处 `.claude/kb/decisions/28-挂载期承诺量.md:74-89`（整段抄，未转述）**

```markdown
#### 已定项 2：准入的在飞合成

**定案**：准入读数 = 已发布统计量 − 在飞已批准的预留；窗口内产生的释放不发信用（D16（发布语义） 已定项 15 的准入侧形态）；「最坏情况下解开自己需要多少」含墓碑那一笔（D3（空间分配） 已定项 2 的可执行读法）。

- 在飞 overlay 是纯内存派生态：崩溃后按定义清零（在飞事务全部作废），不落盘、零格式位。
- 实例切换重建内存态时 overlay 按重放结果重算：施加一条记录把它点名的码 1 单元登记成已分配；所选根之后写出的码 2 / 3 单元按已发布谓词（D18（块里携带什么信息） 已定项 11）未发布、不登记、由重做重写；失败那次 checkpoint 的固定点分配与号 > W 事务的分配作废、重做时重新准入。

**射程**：切换那一句同时定了 C284（施加一条记录在指针层上做什么没有定义） 空间侧在切换这一格的读法；指针层其余部分仍归 C284（施加一条记录在指针层上做什么没有定义）。

**依据**：

- E82（准入的在飞合成）：三种直觉写法各出一类可数违例——只读已发布值同窗超卖、把窗口内释放记成可用会吃进 defer 该扣住的块（D16（发布语义） 已定项 15 被静默绕过）、忘了墓碑盘满后删除全数死锁。
- 切换那一句：C126（切换预留的最坏量没有口径） 第一轮三方论证（`research/prompts/c126-r1-main-verification.md`）之后的用户定案，原话在变更史。

**欠**：C87（准入读的合成值无定义）：E82（准入的在飞合成） 的三个红样例照抄进准入路径的单测与崩溃点重放，overlay 做成可强制清零的测试开关（清零后并发准入必须超卖、检查必须抓到——双向证）；`crates/singlefs-core/src/admission.rs` 的注释写明「在飞 overlay 没有实现」（准入读数的在飞合成今天没有代码）。

```

**出处 `.claude/kb/decisions/28-挂载期承诺量.md:90-111`（整段抄，未转述）**

```markdown
#### 已定项 3：实例切换的预留

**定案**：预留 = (N_switch + 1) × 一次切换的最坏量，按设备算；一次切换的最坏量 = 实例表链重写 + 暖机空发布。

- **链重写**：副本数 × 32768 × max(1, ⌈(rows0 + N_switch) / 每片行数⌉) 字节。副本数 = 2（实例表单元指针的两个位置条目，D22（单元原子性怎么合成） 已定项 7；两份落两块盘，所以按设备各算一份 32768 × 片数）；每片行数 = ⌊(32768 − 135) / 行宽⌋ − 1（链指针记录恒为一片的最后一条，D18（块里携带什么信息） 已定项 11；行宽 88 ⇒ 369 行）；rows0 = 挂载时读到的行数加写行那次要写的行数（新实例 − max(所选根的实例, 1)，挂载时就知道；每次可写挂载都写行）。
- **暖机**：至多 R 次空发布 × 现算 c_max（R = 3，c_max 按已定项 4 每次发布现算），预留里这一半是 (N_switch + 1) × R × c_max；第一个可运行目标两块盘互为镜像，空发布的 c_max 块每块盘各占一份，所以按设备各加一份。
- **N_switch + 1 份**：N_switch = 3（D23（journal 的角色与格式） 已定项 14）；多的一份给写行那次发布的元数据——写行那次是新实例的第一次发布，之前不推抬 F 的空发布，同一次发布里的用户数据重做照走准入、不够返回 ENOSPC。N_switch 那几份是逐次累加的上界：连续切换的前提是上一次的根没发布、重建态从同一个根来，上一次的链页在重建态里不存在，实际不累加。
- **固定点重做与号 > W 的数据单元重写不另要空间**：切换 = 挂载内一次恢复（D23（journal 的角色与格式） 已定项 14），重建态从所选根 + 施加到 W 的记录来；失败那次 checkpoint 的固定点单元与号 > W 事务的单元从没进过已发布的分配记录、也不被施加登记（已定项 2 的切换那一句）⇒ 重建态里它们是空闲的，重做在同一片空间上再做一遍，用的是 checkpoint 的保留池；号 > W 的用户数据分配失败按 D23（journal 的角色与格式） 已定项 14 向调用者返回 ENOSPC，不阻塞切换。
- **发布路径用 rows0 + 1**：发布准入里的切换预留按下一次可写挂载的 rows0 算（这次挂载的 rows0 + 1，下一次挂载必写一行）；可写挂载自己判时照旧按这次的 rows0（用户 2026-09-26 定，攻方在副本上量过：实例表行数到 366 那一格崩了再挂被拒 8 → 0；被攻过零轮）。
- **`df`**：可用已经扣掉这一项，`df` 报可用（D3（空间分配） 已定项 9 第 1 条）；没扣时出的是假性 ENOSPC（D16（发布语义） 已定项 1）。

**射程**：式子按「两份副本各落一块盘」写，只对两块盘的池没有歧义（D2（RAID 条带策略） 已定项 9 的第一个可运行目标）；**三块盘以上时两份落哪两块盘没写**，挂载判定该要求「任意两块」「空闲最多的两块」还是「分配器会选的那两块」都没定，空发布落哪块盘同样没写。第一版不跑 zoned，zoned 上 rows0 无界（C121（zoned 上实例表链长无上界））；非 zoned 上 rows0 由行回收压住，但回收要「无读失败 ∧ 全盘在线」且删行要可写挂载，近满盘 + 坏扇区的池上「预留拿不到 ⇒ 只读」可能成吸收态（推的，每 369 行才多一片）；每次可写挂载都写行、行回收要根环每个槽都读成功且自证过之后（D18（块里携带什么信息） 已定项 11「回收」那一条，清扫器 `crates/singlefs-core/src/instance_table_sweep.rs` 照它不删），一个根槽持续读不出、读回全零或重读仍自证不过时行只增不减，这个吸收态不再要求近满盘（C597（实例表行只增不删，删行的条件没有条款））。D16（发布语义） 已定项 5 只写 T_dirty 是触发条件、没写重做期间开放的 checkpoint 会不会堵住写者，那是 D16（发布语义） 的口径空白。

**依据**：

- E141（切换预留的挂载准入自证）：挂载准入按这个式子，在只差 1 块的两个池上于份额处翻，预留置 0 时都可写，池级标量判不出短的那块盘，七个错法各有取样点分得开；新池新建文件几何下每块盘的份额与各片边界的取样都在那一页，只建了两块盘。
- E148（提交固定点按两棵记录树重算）：c_max 的现算口径，新池新建文件规模的数在那一页。
- 三方：C126（切换预留的最坏量没有口径） 第一轮（`research/prompts/c126-r1-main-verification.md`），链重写的式子由两条云端腿改正；多的那一份来自 C329（写行那次发布之前推抬 F 的空发布没有检查） 三轮三方之后的用户定案；暖机那一半连带 D16（发布语义） 已定项 8 的用户定案。
- 发布路径用 rows0 + 1：用户定案，原话在变更史；安全设计轮第三轮判决 `research/prompts/m2-safety-r3-main-verification.md` 第二节「两处结构性分叉」那一行与第三节。

**欠**：C126（切换预留的最坏量没有口径）：实现侧的对拍与三块盘以上的落点；C121（zoned 上实例表链长无上界）；C597（实例表行只增不删，删行的条件没有条款）。

```

**出处 `.claude/kb/decisions/28-挂载期承诺量.md:112-141`（整段抄，未转述）**

```markdown
#### 已定项 4：checkpoint 保留池进准入

**定案**：checkpoint 保留池不设常数上限，按当时的结构现算，池多大它就多大。

- **形态**：ckpt_cost = Σ（每棵记录树当前的高）+ 记账树每发布的节点数 + 1（树表），每次发布按当时的树高重算；每棵码 2 树当前的高 = 发布那一刻从它的根节点码 2 头里现读的层级 + 1（D18（块里携带什么信息） 已定项 18 的层级字段），不用内存里另存一份；按 key 空间定形状的分配记录树与 extent 树的高怎么读，随三方 `m2-keyspace-r1`。单位是 16 KiB 元数据块，代进已定项 1 那条按字节写的式子时乘 16384（式子里的「已分配」是「已分配字节」、「容量」是单元区大小，D5（快照 / 空间记账机制） 已定项 7）；它是运行时算出的量，盘上没有字段。
  - 分配记录树这一项按 K1 算：1 + Σ_盘 Σ_{层级 L = 0 … 根层级 − 1}（这块盘单元区 [s, e) 在层级 L 上罩到的位置数 = ⌊(e − 1) ÷ S_L⌋ − ⌊s ÷ S_L⌋ + 1，S_L = 812 × 169^L）；根层级是根节点码 2 头里现读的层级（树表 0 条、写过行的那一版取池几何的根层级），不读这一版的状态、不依赖 ckpt_cost，恒是上界：一次发布改几片叶的不动点收不了口（开放段装不下开新段、没有全空段回落、换下很久以前落在别处的节点，实审 A4c 量到一块盘改 3–4 片叶），拿得准的上界只有整棵树；代价随单元区线性涨：两块 4 GiB 盘 529 个节点，每块盘多扣约 2.7%（1 TiB 约 3.2%）。中央映射树这一项进 Σ，按它现有的删插式子算，删插数取「分配记录树这一项 + 记账树节点数 + 管理员回退那次发布多写的谱系容器、谱系树根各一把和谱系树内部层每层一把 + 退役容器、退役树根各一把和退役树内部层每层一把」（退役那几把随 ckpt_cost 的退役项，判决 `research/prompts/m3-retire-r1-main-verification.md` 第六节 R2，推的，零轮）（谱系容器是一个 32 KiB 单元、一把 key，按单元数算、不按块数），随分配记录树这一项的式子变而跟着变；
  - 记账树不进 Σ，单列，按每发布的节点数；
  - 树表单列一项，每次发布 1 个单元（D16（发布语义） 已定项 9：每次发布重写它，它没有高）；
  - 谱系容器单列一项，按一次谱系容器 COW 连祖先算：容器 2 块 + 谱系树根 1 块 + 谱系树内部层数，零根与只有根的谱系树 3 块、长高一层 4 块（管理员回退那次写它，D18（块里携带什么信息） 已定项 24；按一次谱系容器 COW 算被攻过零轮，连祖先算是用户 2026-10-02 定的，没攻过）；
  - 退役容器单列一项，按一次退役容器 COW 连祖先算：容器 2 块 + 退役树根 1 块 + 退役树内部层数，零根与只有根的退役树 3 块（最后一批写它，D18（块里携带什么信息） 已定项 11 类型 3；用户定案「两样都做（推荐）」，与 D3（空间分配） 已定项 2 标签 6 的退役项合起来这一形被攻过零轮）；
  - 实例表链不进 ckpt_cost，它的开销归已定项 3 的切换预留。
- **归属**：算挂载期承诺量，进已定项 1 的式子（单列第八项），落在已定项 1 分界的后三项那一侧：按全部副本之和记（ckpt_cost × 16384 × 副本数），逐设备各扣「副本之和 ÷ 副本数」。
- D16（发布语义） 已定项 1 的推空保留池 10 + 7 c_max 与 D16（发布语义） 已定项 8 的切换预留都按同一个现算的 c_max 取。

**射程**：保留池本身（`reserve ≥ ckpt_cost`、死锁 2 与假性 ENOSPC 的区分）的权威句在 D23（journal 的角色与格式） 已定项 24，这里只定它怎么进准入。条款里不写常数：分裂合并、条带成员表、livelist 都没进 E148（提交固定点按两棵记录树重算） 的模型，它的 9 块是模型数。退役容器那一项只罩一次发布的暂态（一次发布新写的退役容器与它的祖先）；永久长出的槽由 D3（空间分配） 已定项 2 标签 6 的退役项预付，不在 ckpt_cost 里；「删 → 回退」环下两样合起来仍超，记在 C617（删后回退留下的墓碑没有预付罩着）。

**依据**：

- E148（提交固定点按两棵记录树重算）：聚簇提交块时固定点是与用户块数无关的常数，用户块随机或全散布时没有可用上界——所以现算、条款里不写常数。
- E19（defer 窗口下的假性 ENOSPC）：预留恰等于 ckpt_cost 时 checkpoint 就不卡死——保留池必须进准入。
- 用户定案：不设常数上限、现算，原话在变更史。
- 用户定案 2026-09-23：逐设备合取（C355（checkpoint 保留池池级扣而固定点每块要落两列） 取丁），原话与它推翻的那句 2026-09-13 原话在变更史；判决 `research/prompts/c355-c363-r3-main-verification.md` 第五节第 2 题。
- 用户定案 2026-09-24：树高从根节点头现读（原话在变更史；三方 `research/prompts/m2-treesplit-r1-main-verification.md` T5）。
- `research/prompts/c363b-r1-main-verification.md` 第三节 V2-2、第四节第 4 条。
- 用户定案：分配记录树这一项换成 K1（整棵树），2026-09-27 弹窗选「K1 整棵树」；代价与候选式子表在 `m2-rev-a4c-implementer-report.md` 第二节「候选式子」（K0 不是上界、实测三种情形一块盘改 3–4 片叶、不动点不收敛）。
- 用户定案 2026-10-02：谱系容器那一项「乙 连祖先算（推荐）」，中央映射树那一项的删插数「乙 一并算上（推荐）」，原话在变更史；调度记录 `records/2026-10-01-删除与并发事务.md`「十六　批 8」。
- 退役容器那一项：判决 `research/prompts/m3-retire-r1-main-verification.md` 第二节 K4 的 P-a、P-b 两行与第六节「两样都做」（照谱系单列一项单做，不回退也超卖，第一次抽样在副本上量到、主 agent 复跑重现；两样合起来那一形被攻过零轮）；用户定案「两样都做（推荐）」，两次弹窗的原话在变更史。

**欠**：C83（提交固定点没人回答）：实现后记固定点轮数，超过当时现算的量判红；分配记录树按 K1 的式子已写进 `admission.rs`（实审 A4d，`m2-rev-a4d-implementer-report.md`），交主 agent 的三处设计问题记欠：C573（固定点上界没算有普通分配那次发布多出的删插）、C574（落得下那一判不核两槽对齐与各盘同槽）、C575（一个错误成员装两种判据的载荷）、C576（三条用例被准入先拒连带红，改法待选）。

```

**出处 `.claude/kb/decisions/05-快照-空间记账机制.md:8-32`（整段抄，未转述）**

```markdown

| # | 分项 | 定案 |
|---|---|---|
| 1 | **维度元组里有哪几维** | 除树 ID 之外只加「设备」一维，key 写作 `(统计量, 树 ID, 设备, 代)` **状态：已定。** **进度：已实现。** |
| 2 | **「代」取什么，旧代什么时候可以丢** | 代取 checkpoint 号、记在行 key 里；每条根的记账树只装这条根那一代的行，不留多代；inode 号水位的台阶行除外 **状态：已定。** **进度：已实现。** |
| 3 | **记账的载体形态** | 住一棵独立 keyspace 的 btree，条目是「(统计量, 维度元组, 代) → 完整值」，走 write buffer 前端、按 COW 挂父指针下 **状态：已定。** **进度：已实现。** |
| 4 | **统计量有哪几个** | 在用十二个、编号到十四（第 8 / 9 项已撤回，编号不复用）；带设备维六项、带树维一项 **状态：已定。** **进度：已实现。** |
| 5 | **记账 key 各段的宽度** | `(标签 2, 树 ID 8, 设备 4, 代 8)` ⇒ key 22 字节；value 8 加 seq 4 ⇒ 条目 34 字节 **状态：已定。** **进度：已实现。** |
| 6 | **稀疏旁表的落点与字段表** | 自成一棵树 day-1 注册；条目 = 中央映射 key 27 + 计数 4 = 31 字节一宽；提交时维护，删除走意图机制 **状态：已定。** **进度：已实现。** |
| 7 | **「已分配字节」含不含 mkfs 写下的固定结构** | 不含：已分配 = 分配记录树里落点之和，准入里的「容量」= 单元区大小 **状态：已定。** **进度：已实现。** |
| 8 | **新池新建文件记账树里写哪几行** | 有值的统计量写一行、没有对象的不写行；新池新建文件写 15 行 **状态：已定。** **进度：已实现。** |
| 9 | **可写头由哪个 ID 标识，`previous_snapshot_txg` 归谁** | 头 ID = 该头 inode 树的树 ID；前驱快照号写在该头 inode 树那条树表条目与每条码 14 快照条目里，其余写 0 且 0 = 不适用 **状态：已定。** **进度：已实现。** |
| 10 | **统计量标签登记表与两个保留值** | 标签码 = 已定项 4 那张表的行号；不带设备维的设备段写 0xFFFF_FFFF、不带树维的树 ID 段写 0 **状态：已定。** **进度：已实现。** |
| 11 | **deadlist 的载体 day-1 建不建** | 建：自成一棵树 day-1 注册进树表，种类码 8、树 ID 65543、根指针零 **状态：已定。** **进度：已实现。** |
| 12 | **deadlist 条目的形态** | key = 头的树 ID 8 + 纪元 8 + 中央映射 key 27 = 43 字节，value 空；条目按纪元分段，交接不搬条目 **状态：已定。** **进度：未实现。** |
| 13 | **生命周期判定取哪套机制** | 取诞生代号 + deadlist：删除时比 `previous_snapshot_txg`，建快照整个交接，销毁快照并到更新的那一侧、不做全盘扫描 **状态：已定。** **进度：未实现。** |
| 14 | **引用区间的充要条件** | `birth(b) ≤ S.txg < death(b)`，左闭右开；三个量各有定义；判据的输入取不到时默认进 deadlist **状态：已定。** **进度：未实现。** |
| 15 | **后台搬迁改不改 `birth`** | 不改：搬迁不是一次发布，`birth` 原样跟着单元走 **状态：已定。** **进度：未实现。** |
| 16 | **销毁快照时 deadlist 往哪一侧并** | 取约定 B（并到下一个更新的那一侧）；它不是「快照的 deadlist 永不改写」，只是例外少一个数量级 **状态：已定。** **进度：未实现。** |
| 17 | **四条前提，越过任何一条它就失效** | 四条前提逐条写死；引入同一块的第二个活引用必须先重开 D5（快照 / 空间记账机制）；已触发一次，收口形态是 D5（快照 / 空间记账机制） 为主 + 稀疏旁表为辅 **状态：已定。** **进度：无对象。** |
| 18 | **克隆 / 可写快照要加三条** | 每头各持一份前驱快照号与 deadlist 且按 origin 初始化；分叉点快照与还有自己活快照的头不许销毁；销毁一个头要 livelist **状态：已定。** **进度：未实现。** |
| 19 | **生死判定的运行时形态与销毁快照的代价** | 先问旁表，计数大于 1 就减计数，否则 `birth > previous_snapshot_txg` 即释放、余者进 deadlist；销毁快照 O(deadlist 条数 + 桶数) **状态：已定。** **进度：未实现。** |
| 20 | **待删除结构算活的还是死的** | 算活引用：`death` = 物理批次的 checkpoint，判定天然发生在死亡那一刻 **状态：已定。** **进度：已实现。** |
| 21 | **快照列表要不要一个可枚举的持久结构** | 要：盘上必须存在一个可枚举的持久结构，其元素恰为当前全部活快照 **状态：已定。** **进度：已实现。** |

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:8-29`（整段抄，未转述）**

```markdown

**D8（核心索引结构）的分项只有一套编号**，不重排。

| # | 分项 | 定案 |
|---|---|---|
| 1 | **记账怎么表达成幂等完整值** | 记账取幂等完整值，不用增量 Δ；每条根装自己那一代的行（整批换代，不留多代），台阶行随根留 **状态：已定。** **进度：已实现。** |
| 2 | **节点大小取多少** | 16 KiB：mkfs 写进系统配置，实现按常量走，不按设备特性算 **状态：已定。** **进度：已实现。** |
| 3 | **key 布局** | key = `(locality_id, inode, offset)`，`locality_id` 创建时从父目录继承、改名时故意不更新 **状态：已定。** **进度：已实现。** |
| 4 | **节点大小的上限由什么定** | 上限由恢复时间定，不由可恢复性定；取值在「重扫代价 ≤ 目标线」下取查找代价最低的那档，靠实验 **状态：已定。** **进度：已实现。** |
| 5 | **whiteout 删除** | 接受墓碑；可能修改无界数量 extent 的操作先写意图再分批推进；第一版 unlink 的意图住 `deleted_inodes`（已定项 16） **状态：已定。** **进度：已实现。** |
| 6 | **inode 记录的盘上字段表** | inode 树叶是码 3 打包记录单元（类型 2，一叶 233 条定长 140 字节记录），内部节点码 2、条目 120 字节、无缓冲；(inode 号, 对象出生代) 发布后不复用 **状态：已定。** **进度：已实现。** |
| 7 | **write buffer 条目自带的 `seq`** | 宽 4 字节、住 value 不住 key，射程覆盖记账 / 反向索引 / LRU 三棵树，`T_dirty` 的上界因此是挂载时判定 **状态：已定。** **进度：已实现。** |
| 8 | **树表条目：树 ID 的分配与编码、树的种类、根指针以外还有什么** | 条目合计 200 字节、树 ID 8 打头；快照进树表；树 ID 水位取根环全部根记录该字段的 max；未知 flags 位拒收该条目 **状态：已定。** **进度：未实现。** |
| 9 | **树的种类码的登记表** | 码 0 无、1..8 是八棵既有的树、9..15 是 deleted_inodes、墓碑树、谱系树、logged_ops、退役树、快照 inode 根、快照 extent 根；新树只追加不重用码 **状态：已定。** **进度：已实现。** |
| 10 | **第一版记账走不走 write buffer 前端；`seq` 的取值规则** | 第一版记账直落记账树叶、不走前端；`seq` = 同一个 key 在一次发布里被写进树的第几次，从 1 起，第一版恒 1 **状态：已定。** **进度：已实现。** |
| 11 | **码 2 索引节点的载荷布局、种类到条目宽的表、key 比较序** | 码 2 头自描述、基础 86 字节，条目区从含预留位的头末尾起定宽连续排列，key 是条目的前 key 宽 个字节 **状态：已定。** **进度：已实现。** |
| 12 | **write buffer 前端与节点消息缓冲区的衔接** | 一棵树只准一级：走前端的三棵树 ε 恒 0、写一律进前端、发布前排空到叶；第一版两级都不开 **状态：已定。** **进度：未实现。** |
| 13 | **索引树更新做不做状态折叠** | 同一次发布里同一个 key 改了多次时落叶只做一次、直接写最终态 E，中间态不写进任何节点，E 按到达序认 **状态：已定。** **进度：已实现。** |
| 14 | **索引的总体结构** | 一套 btree 实现、多个独立 keyspace、两个前端（write buffer、key cache）；派生态的分配记录树与 extent 树可以另起按 key 空间寻址的结构 **状态：已定。** **进度：已实现。** |
| 15 | **write buffer 条目的三条硬要求** | 节点缓存必须抗扫描；条目必须是幂等完整值、不许是增量 Δ；条目必须自带序号 `seq`，这是格式要求 **状态：已定。** **进度：未实现。** |
| 16 | **第一版 unlink 怎么删** | 第一版 unlink：只立 `deleted_inodes`，从尾部按 144 单元分批，挂载里续做到空 **状态：已定。** **进度：已实现。** |

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:8-24`（整段抄，未转述）**

```markdown

D15（格式冻结政策）的分项只有一套编号，不重排。

| # | 分项 | 定案 |
|---|---|---|
| 1 | **冻结前清单第 7 项的具体处置** | 第 7 项不写决策名单，写成一句判定——全仓每条未定分项自己声明「动不动格式」，冻结请求与它相交即判红；留白宽度默认 0 **状态：已定。** **进度：无对象。** |
| 2 | **树表是不是开放列表** | 是，day-1 就做成开放列表 **状态：已定。** **进度：已实现。** |
| 3 | **`spec_hash` 的来源** | 不切换。恒对 `format-spec/<组件>.toml` 求，那份 toml 是 kb 字段表的完整投影，含行序与非标量的盘上语义规则 **状态：已定。** **进度：未实现。** |
| 4 | **feature bit 的位分配登记表** | 登记表住已定项 4；incompat 位 1 = 第一条纯 SSD 布局线（带 F 与卸载记号那一版），mkfs 起就置上，位 0 退役；compat_ro、compat 一位都没分 **状态：已定。** **进度：已实现。** |
| 5 | **冻结这个动作是什么** | 冻结 = 提交一份 `.claude/kb/format-frozen/<组件>.toml`，人只决定发不发起，机器决定这次请求成不成立 **状态：已定。** **进度：未实现。** |
| 6 | **冻结前必须完成的清单** | 七项前置，逐次冻结重判；每一项的取值从对应分项的定案行现取，不许手抄 **状态：已定。** **进度：无对象。** |
| 7 | **冻结分几层，四层之外还有什么** | 四层有严格依赖顺序（系统配置与块头、指针字段布局、key 编码、索引节点内部布局），另有三个独立冻结组件 **状态：已定。** **进度：无对象。** |
| 8 | **冻结之后失去哪些自由** | 已冻结字段的宽度、存在性、含义不能再改，只能新增；对已冻结部分 checker 的判断也成了兼容性契约 **状态：已定。** **进度：无对象。** |
| 9 | **冻结后的改动判定表** | 九类改动各自判 compat / compat_ro / incompat 与 bump 与否，判据是旧读者会怎样 **状态：已定。** **进度：无对象。** |
| 10 | **feature bit 的位数、记账与分配权** | 三个 bitmap 各 256 位存系统配置；分配权是「谁先合并谁先占」，位号严格递增；位一旦用过不许回收 **状态：已定。** **进度：已实现。** |
| 11 | **冻结要配的两条会红的检查** | 格式冻结契约（`spec_hash` 比对）与向后兼容回归（黄金镜像），镜像本体不进 git、清单只增不删 **状态：已定。** **进度：未实现。** |

```

**出处 `.claude/kb/decisions/28-挂载期承诺量.md:8-15`（整段抄，未转述）**

```markdown

| # | 分项 | 定案 |
|---|---|---|
| 1 | **准入不等式的权威形式** | 可用逐设备算、每块盘各自满足：五项取这块盘的值，待删占用（恒 0）、已承诺预留、保留池按全部副本之和摊到每块盘，另扣每盘的常驻环窗口 **状态：已定。** **进度：已实现。** |
| 2 | **准入的在飞合成** | 准入读数 = 已发布统计量 − 在飞已批准；窗口内释放不发信用；「解开自己」含墓碑 **状态：已定。** **进度：已实现。** |
| 3 | **实例切换的预留** | (N_switch + 1) × 一次切换的最坏量（实例表链重写 + 暖机空发布），按设备算 **状态：已定。** **进度：已实现。** |
| 4 | **checkpoint 保留池进准入** | 保留池按当时的结构现算、不设常数上限，算挂载期承诺量，按全部副本之和记、逐设备摊扣进已定项 1 **状态：已定。** **进度：已实现。** |

```
