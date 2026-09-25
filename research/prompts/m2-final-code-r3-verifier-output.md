# m2-final-code-r3 核查员报告

你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

开工读过 `.claude/agent-common.md`、`.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」、
`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「校验路径本身也要证明它会红」「引产物就整行抄」。

草稿目录：`/tmp/claude-1000/m2-final-code-r3-verifier/`（`opus-rerun/` 是冻结副本的独立拷贝 + 打过
`copy-only.patch`，`selftest/` 是判别力自证用的行号偏移副本）。

## 判别力自证

抽的引用：云端正推（Sonnet）报告里 `08-核心索引结构.md:387`（已定项 14「挂载怎么读」，整行抄）：
> 分配记录树挂载时整棵读进挂载态（分配器本来就要）；extent 树按需，打开文件时按位置走下去（用户 K4）。

在草稿目录的 kb 快照副本里把行号加 1（387 → 388）后核：

```
$ awk 'NR==388' /tmp/claude-1000/m2-final-code-r3-verifier/selftest/kb-snapshot-shifted/.claude/kb/decisions/08-核心索引结构.md
- **实现取值**（实二一取，主 agent 2026-09-25 定，被攻过零轮）：
```

第 388 行的原文（「实现取值……」）与要核的引文（「分配记录树挂载时整棵读进挂载态……」）逐字不同 ⇒ **判 ✗**。
核查方法能分辨错行号，往下按原始行号（387，与真实文件一致）继续核。

## 复跑环境

- Opus 腿：`rerun.sh` 的关键部分在草稿目录 `/tmp/claude-1000/m2-final-code-r3-verifier/opus-rerun/` 里独立跑了一遍：
  拷冻结副本 → `patch -p2 < copy-only.patch`（干净应用）→ release 编译 `r3_opus_z13`、`r3_opus_z16`（8 线程，`capped.sh 8`）
  → Z17 模型、Z16-a 写死复现×2 种模式、Z16 定点扫描×3 遍（盘 1 / 盘 0 / 试改）、Z16 冻结重挂 4 段、
  Z13 两个种子的 `z13_one_seed`。
- Sonnet 腿：草稿目录 `/tmp/claude-1000/m2-final-code-r3-verifier/sonnet-rerun/` 里独立拷了冻结副本，
  跑了它引的两个测试二进制。
- 本地攻方：独立重跑 `corruption-check.py`、`oov-check.py`（纯本地脚本，不需要网关）。
- `SHA256SUMS`（`research/prompts/m2-final-code-r3-opus-model/`，70 个文件）：`sha256sum -c` 全部 `OK`，
  报告里贴的 8 个非日志文件哈希与 `SHA256SUMS` 里的记录逐字节相同。

## 云端攻方（Opus）：`m2-final-code-r3-opus-output.md`

| 引用 | 核的结果 | 命令 |
|---|---|---|
| Z17 独立模型原样输出（正文 20-42 行） | ✓ 逐字节复现 | `python3 model/z17_independent.py > logs/z17.txt`；`sha256sum` 与我自己跑的一致（`d40809be0...`），与正文表格逐行相同 |
| `first_transaction_step_five_publish.rs:405`（12 单元落点 50180,50240,…,50252） | ✓ | `awk 'NR==405'`，逐字相同 |
| `first_transaction_step_five_publish.rs:363`（`(31,"24+2+1+2",16_777_223)`） | ✓ | 同上 |
| `first_transaction_step_six_recovery.rs:77`、`:188`（树表 50252） | ✓ | 两行都含 `SlotNumber(50252)` |
| `second_transaction_step_one_overwrite.rs:154`（B 落点 50253..50264）、`:281`（3_472_375_808）、`:94`（"24+2+1+2"） | ✓ | 154 附近 127-154 整段逐条比对，与 Z17 输出的 B 落点列表一一对应；281、94 单行核对 |
| `second_transaction_parallel_line_three_many_inodes.rs:467`（67−11、七个节点、68 单元） | ✓（citing 行只含 `67-11` 那一句，"68" 与"七个节点"在同一函数体内 463-482 行，同一注释块） | `sed -n '440,482p'` |
| `second_transaction_supplement_two_admission_formula.rs:218`（被抛弃根独占 54） | ✓ | `sed -n '205,220p'`，`vec![(DeviceIdentity(0),54),(DeviceIdentity(1),54)]` |
| `second_transaction_step_four_rollback.rs:1130`（54） | ✓（只支持"54"，见下一行） | `sed -n '1118,1131p'` |
| 同一格「去掉 C 之后 40」引用同一行 `:1130` | **✗ 行号误**：1130 行只有 "54"，"40" 实际在同一文件第 **1176** 行（`vec![(DeviceIdentity(0),40),(DeviceIdentity(1),40)]`，"少了只被 C 引用的 14 个槽"那条断言，不是 1130 行那条 "普通重开照样隔离" 断言） | `grep -n '\b40\b' second_transaction_step_four_rollback.rs` → 1176、1399 |
| `second_transaction_supplement_two_admission_formula.rs:243`（影子账多写 2） | ✓ | `assert_eq!(extra_slots_written_with_the_shadow_ledger, 2, …)` |
| `mount.rs:1215`（回退目标取最旧那段注释） | ✓ | 整段含"回退到环里最旧的那条根、其余全被抛弃时走得到" |
| `mount.rs:1763`（`if let (Some(planned), false) = (&placements_planned, some_copy_was_quarantined)`） | ✓ 逐字 | `sed -n '1761,1765p'` |
| `mount.rs:1274`、`:1327`（两处 `copy.record_root_written_by_this_process`） | ✓ 两处均逐字匹配 | `sed -n '1272,1276p;1325,1329p'` |
| `transaction.rs:4691`（固定点判据）、`:4709`（`ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT = 64`）、`:5590` 起（`publish_admitted` 断言） | ✓（非"整行抄"声明，位置与内容对得上） | `sed -n` 三段 |
| `singlefs-checker/src/position_addressed.rs:10-14`（`use singlefs_format::{...}`） | ✓ | 含 `ALLOCATION_RECORD_TREE_LEAF_SLOTS`、`ALLOCATION_RECORD_TREE_INTERNAL_FANOUT` |
| `allocation_record_tree.rs:124`、`allocator.rs:222`、`walk.rs:1307`（根层几何三处同口径） | ✓ 三处逐字匹配 | 各自 `sed -n` |
| kb `18-块里携带什么信息.md:314`（预演不做释放前读盘核…C542） | ✓ 整段含引文 | `awk 'NR==314'` |
| kb `checks-owed.md:475`（C542 行） | ✓ | 含"真发在取号之后被落点拒绝，烧掉一个实例代号，不丢数据" |
| kb `invariants.md` I-3.1/I-3.11 隔离豁免字面 | ✓ 两处（130、140 行）均含"每一份都读得出且对得上，照旧判违例" | `grep -n` |
| kb `03-空间分配.md:212`（t8 50248）、`:209`（t5 50245） | ✓ | `awk 'NR==212;NR==209'` |
| kb `19-块指针的结构与宽度预算.md:201`（t8 50248 无树归属） | ✓ | `awk 'NR==201'` |

### Z16-a：16 格、三句、两次抽样、试改（重点核 1）

独立在草稿目录里从零编译、跑出（不是读腿留下的产物）：

```
$ awk -F'\t' 'NR>1 && $3=="none"' logs/z16-targeted-240-384.tsv | wc -l   # 666
$ awk -F'\t' 'NR>1 && $3!="none"' logs/z16-targeted-240-384.tsv | wc -l  # 2619
$ diff -q logs/z16-targeted-240-384.tsv .../m2-final-code-r3-opus-model/logs/z16-targeted-240-384.tsv && echo IDENTICAL
IDENTICAL
$ sha256sum 两份 z16-targeted-240-384.tsv
1cb865d3d0832f93a7b4568e170dd308dfbb3e9fa85e82b09b736f8cf845d240  （两份相同）
```

我自己独立编译、独立跑出来的盘 1（默认）扫描产物与腿交回的模型目录里的归档产物**逐字节相同**（sha256 相同），
不只是数字对得上。分组统计（`awk` 现算）：

```
1558 CheckerViolations|Applied:Mounted|true(quarantined)|true(same)
  54 CheckerViolations|Applied:Mounted|true(quarantined)|false(same)
  16 ModelDisagreement|Refused:MountError::Publish(PublishError::PlacementRefused(NoFreeSlotOnAnyDevice))
 991 Completed|Applied:Mounted|false(quarantined)|true(same)
 308 Completed|Refused:MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided(NoFreeSlotOnAnyDevice)（对照组本身）
```

与正文「原样汇总」表格逐行一致。16 格的具体槽号（`awk -F'\t' '$4 ~ /ModelDisagreement/'`）：

```
312 24 50245/50246/50247/50248；316 24 50306/50307/50308/50309；324 24 50306/50307/50308/50309；336 24 50312/50313/50314/50315
```

与正文「312 档 50245..=50248；316、324 档 50306..=50309；336 档 50312..=50315」逐字对得上 ⇒ **✓**。

盘 0 抽样（`R3_OPUS_FAULT_DEVICE=0`）与试改（`SINGLEFS_R3_OPUS_DRY_RUN_READS=1`）两遍扫描仍在跑（见下方「核不动/待补」）。

三句写死复现（正文 94-97 行）与试改三句（113-115 行），各自单独在草稿目录里重跑：

```
$ SINGLEFS_R3_OPUS_UNIT_AREA_SLOTS=312 R3_OPUS_BAD_SLOT=50245 ./r3_opus_z16 --exact z16_rollback_refused_after_acquisition_burns_instance_generations_by_hand --nocapture 2>&1 | grep R3OPUS-HAND
R3OPUS-HAND prefix ending=Completed last=Some(Refused { member: "PublishError::PlacementRefused(NoFreeSlotOnAnyDevice)" })
R3OPUS-HAND rollback attempt 0 to (instance 1, txg 3): to_acquire before=InstanceGeneration(3) after=InstanceGeneration(4) writes+barriers issued=28 result=Publish(PublishSequenceFailed { cause: PlacementRefused { unit: TreeTable, refusal: NoFreeSlotOnAnyDevice }, writes_of_persisted_publishes: [WritesByStructureKind { counted: {AllocationRecordTreeNode: WriteCallsAndBytes { write_calls: 10, written_bytes: 163840 }, AccountingTreeNode: WriteCallsAndByt
R3OPUS-HAND rollback attempt 1 to (instance 1, txg 3): to_acquire before=InstanceGeneration(4) after=InstanceGeneration(4) writes+barriers issued=0 result=RollbackTargetNotACandidate { target: RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(3) }, exclusion: NotInRing }
```

（后两行为节选，与正文引用逐字相同，"NotInRing" 一字不差。）试改（`SINGLEFS_R3_OPUS_DRY_RUN_READS=1`）三格（312/316/336）三条 `rollback attempt 0` 输出，与正文 113-115 行**逐字节相同**（`diff` 得到 `IDENTICAL`）。三句「预演放行、真发取号之后被拒、再试回退报 NotInRing」都在我自己重跑出的原始产物里逐字存在 ⇒ **✓**。

**产物完整性观察（不改变上面 ✓ 的判定，但值得记）**：`m2-final-code-r3-opus-model/logs/z16-hand-*.log`（3 个正常态文件）
比我用 `rerun.sh` 第 22 行原样命令重新生成的同名文件多两行开头（`R3OPUS-DRYRUN…`、`R3OPUS-RELEASED…`）与一行
结尾（`test result: ok…`）——用 `rerun.sh` 里那条命令（`grep -E 'R3OPUS-HAND'` 过滤）不会产生这三行，说明归档的这
三个文件不是单纯按 `rerun.sh` 那条命令跑出来的（更像当时 `SINGLEFS_R3_OPUS_REPORT` 环境变量还留在同一个 shell
里、且没有过滤掉 `test result` 那一行）。中间的 `R3OPUS-HAND` 那几行本体逐字节相同（`diff` 确认），试改
（`z16-hand-fix-reads-*.log`）三个文件与我重新生成的**完全逐字节相同**（`diff -q` 无输出）。`z13-seed-400000
0045.log`、`-4000000204.log` 归档版本的最后一行被截断在 401 字节处（我重新跑出来的同一行是 880 字节），但被
截断前的内容仍完整包含正文引用的那句"记账的已分配 Some(2031616)，遍历全部有效根得到 1900544"——正文引用没
有失真，只是这两个归档日志文件本身不完整（可能是当时复制/落盘时被截断）。这两点都不影响正文里"整行抄"或
"原样输出"那几句的正确性，但作为产物质量问题记在这里，供主 agent 判断是否需要腿重新落盘干净的归档副本。

### Z16-b 其余形态（重点核范围外，顺带独立复核）

冻结重挂 160 段：独立重跑 4 个文件（`fk×w`），`grep -c 'R3OPUS-DRYRUN.*same=false'` 全部 0，`grep -c '^R3OPUS-DRYRUN'`
全部 82，`grep -c PublishFrozenAfterAWriteFailureIsNotResentYet` 分别 33/4/29/4（非零），`grep -c '^R3OPUS-FROZEN2'`
合计 160，其中 `checker=[]` 160/160、`remount=Ok` 160/160 ⇒ 与正文「冻结的都在同进程第二次发布时报……160 段重挂
全做成、之后覆盖写做成、checker 全绿；预演行 328 条、逐次不同 0 条」**✓**（328 = 4×82）。

### Z13-d 越格线索：种子 4000000045、4000000204（重点核 3）

独立在草稿目录用同一个二进制单独重跑（不是读腿的产物）：

```
$ R3_OPUS_SEED=4000000045 ./r3_opus_z13 --exact z13_one_seed --nocapture 2>&1 | grep R3OPUS | tail -1
R3OPUS-SEED ending=NewFinding { signature: CheckerViolations { invariants: ["I-3.1"] }, … violations: [("I-3.1",
"盘 0：记账的已分配 Some(2031616)，遍历全部有效根得到 1900544（其中隔离豁免 0）；机理：……")], …

$ R3_OPUS_SEED=4000000204 ./r3_opus_z13 --exact z13_one_seed --nocapture 2>&1 | grep R3OPUS | tail -1
R3OPUS-SEED ending=NewFinding { … "盘 0：记账的已分配 Some(3244032)，遍历全部有效根得到 3112960（其中隔离豁免 0）；……
```

两处数值（2031616/1900544、3244032/3112960，差值均 131072 字节 = 8 槽）与正文逐字相同 ⇒ **复现，✓**。触发路径
（step 27/31 的 `RaiseRollbackFloor` 被 `RaiseFloorSequencePublishFailed(publishes_persisted = 1, PublishError::
PlacementRefused(NoFreeSlotOnAnyDevice))` 拒）也与正文一致。

## 云端正推（Sonnet）：`m2-final-code-r3-sonnet-output.md`

两个测试二进制独立重跑（草稿目录 `sonnet-rerun/`，`CARGO_TARGET_DIR` 单独指向）：

```
$ cargo test -p singlefs-harness --test second_transaction_parallel_line_one_multi_unit_file -- --nocapture
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
$ cargo test -p singlefs-harness --test second_transaction_parallel_line_two_mounted_read
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

通过数、测试名（含 `the_lower_extent_segment_grows_to_two_levels_past_one_leaf_and_shrinks_back_to_inline`）
与正文引用一致 ⇒ **✓**。

| 引用 | 核的结果 | 命令 |
|---|---|---|
| `08-核心索引结构.md:386`（extent 树条款，整行抄） | ✓ 逐字 | `awk 'NR==386'` |
| `08-核心索引结构.md:387`（挂载怎么读，整行抄） | ✓ 逐字（判别力自证用的就是这条） | `awk 'NR==387'` |
| `18-块里携带什么信息.md:52`（声称"整行"抄「按位置寻址的树……checker 按位置独立算出每个节点该罩的那一段逐节点核」） | **✗ 标签不准**：第 52 行实际是一整段（多句），被引的只是**最后一句**；该行开头的句子（"**射程**：外推的是「带不带」这个判定……C478（码 2 头的 key 区间是条目键区间还是子树覆盖区间）"）没有被引用却被标成"整行" | `awk 'NR==52'` 全文见下方引产物核对 |
| `transaction.rs:3479-3561`、`:3536-3539`、`:3541-3555`、`:3542-3545` | ✓ 逐段核对，`replaced_previous_roles`／`lower_nodes` 位置与描述一致 | `sed -n` |
| `extent_tree.rs:210-212`（`data_units<=1` 交回空 Vec） | ✓ 逐字 | `sed -n '200,213p'` |
| `extent_tree.rs:180-192`（`key_range`） | ✓ 逐字 | `sed -n` |
| `extent_tree.rs:67-69`、`unit.rs:217-221`、`write_request_split.rs:114`、`mounted_read.rs:137`（同一个 `data_unit_payload_capacity()`） | ✓ 四处均调用同一函数，无第二处硬编码 32634 | `sed -n` 四处 |
| `recovery.rs:1402-1419`（分配记录树整棵读，注释整行） | ✓ 逐字 | `sed -n` |
| `mounted_read.rs:422-424`（extent 树打开池时不读，注释整行） | ✓ 逐字 | `awk 'NR==422,424'` |
| `mounted_read.rs:11-12`（D17 已定项 5 doc 注释整行） | ✓ 逐字 | `sed -n` |
| `mounted_read.rs:633-643`（`read_node` 闭包 `node_reads+=1`） | ✓ | `sed -n` |
| `second_transaction_parallel_line_two_mounted_read.rs:701-708`、`:762-769`、`:740-743` | ✓ 三处断言与消息整行均逐字匹配 | `sed -n` |
| `mounted_read.rs:586-590`（`tree_node_stale_location_hint_hops_at_open`） | ✓ | `sed -n` |
| `recovery.rs:357-371`、`:292-303`（多跳回退机制） | ✓ 机制描述与代码一致（先按位置提示，再查映射） | `sed -n` |
| `extent_tree.rs:748-797`、`:758-765`（下段任意层共用 `read_node`） | ✓ | `sed -n` |
| `second_transaction_supplement_two_tree_nodes_and_the_central_mapping.rs:367-439`、`:138`（`MappedUnitToRelocate` 六种、都建在无下段镜像上） | ✓ 六个变体名与位置均对得上 | `sed -n`；另检查该文件其余 `node_reads` 命中（1178 行）属于**中央映射树**的 `node_reads`（不同的计数器），不是 extent 树的，不推翻 Sonnet 的覆盖缺口判定 |
| `checks-owed.md:423`（C483 ②，整段抄） | ✓ 逐字（该单元格内有两处几乎相同的措辞，Sonnet 引的是"判别力自证"那一处，逐字匹配） | `awk 'NR==423'` |
| `defs-r2-to-r3.diff`（44 行，4 份文件） | ✓ | `wc -l`；`grep '^+++ '` 列出四个文件 |
| `agent-common.md:57`、`main-agent.md:29`（整句抄） | ✓ 两处逐字 | `awk` |
| hook 33/42/53/61 行起 ①②③④ | ✓ 四处起始行逐字匹配 | `awk` |
| 第三个 hunk（删"它们沿用派发那一刻的定义……"那句解释尾巴） | ✓ diff 原文逐字匹配 | `grep -n -A2 -B2` |

**「引产物就整行抄」核对**（`18-块里携带什么信息.md:52` 全文，供比对）：

```
**射程**：外推的是「带不带」这个判定，不是 E73（节点 key 区间的扇出代价） 那几个代价数——哪棵树的扇出掉几格仍
要各自量。**dirent 树那一格随之关闭**：有 dirent 树时它照样带 key 区间；那时要重新量的是扇出代价，不是这个判
定。key 区间住块头（D15（格式冻结政策） 第 1 层），与记账树节点头同一格（C102（索引节点头没有预留区，加字段
不是 compat））。checker 的 `check_index_node_keys`（`crates/singlefs-checker/src/lib.rs`）判的是首末两条条目
的 key，并行线三的根写出 [2, 2] 而子树覆盖 [2, 234]，两处都要按子树覆盖区间改（C478（码 2 头的 key 区间是条
目键区间还是子树覆盖区间））。按位置寻址的树（分配记录树、extent 树，D8（核心索引结构） 已定项 14），节点头
的 key 区间写这个节点按位置规定罩的那一段，与「子树覆盖区间」在这类树上是同一段；checker 按位置独立算出每个
节点该罩的那一段逐节点核。
```

Sonnet 报告只引了最后一句（"按位置寻址的树……逐节点核"），但正文写的是「见 …:52 整行」——按 kb 文件一行是一
段（未硬换行）的惯例，"整行"意味着这整段都该抄，而 C478 那句讨论的是**另一件事**（码 2 头 key 区间是条目键区
间还是子树覆盖区间，针对的是**非按位置寻址**的树），被引的那句只是同一段落末尾单独成立的陈述。虽然被引的那句
本身没有被断章取义（它本身完整、没有被前面的语境反转），但把"整段的最后一句"标成"整行抄"不准确——`evidence-
discipline.md`「引产物就整行抄」这条纪律是为了防止"挑一句、丢限定词"，这里的限定词（C478 讨论的是哪一类树）
确实没有被带进引用。记 **✗**（标签与实际引用范围不符），但不改变 Sonnet 判定本身的实质结论。

## 本地攻方：`m2-final-code-r3-local-attack-output-s1.md` / `-s2.md` / 转述核对表

**十格算式与快照常量核对**（独立在冻结副本与 kb 快照现查，不读腿的核对表）：

| 项 | s1 答案 | s2 答案 | 快照/代码里的真值 | 核 |
|---|---|---|---|---|
| Item1 W | 812 | 812 | `ALLOCATION_RECORD_TREE_LEAF_SLOTS=812`（`lib.rs:138`） | ✓ |
| Item2 内部扇出 | 169 | 169 | `ALLOCATION_RECORD_TREE_INTERNAL_FANOUT=169`（`lib.rs:144`） | ✓ |
| Item3 配置A | R=1 | R=1 | 给定 240（不用推） | ✓ |
| Item3 配置B（两种读法） | R=2／R=2 | R=2／R=2 | kb `08-核心索引结构.md:395`"4 GiB×2 时根在第 2 层" | ✓，两种读法一致 |
| Item3 配置C（两种读法） | R=3／R=3 | R=3／R=3 | 同一条款推出（`allocation_record_tree.rs:902-910` 断言 `height()==4` ⇒ root_level=3） | ✓，两种读法一致 |
| Item4 (a)(b)(c) | 143/144/147 | 143/144/147 | `EXTENT_TREE_UPPER_LEAF_INODES=143`、`EXTENT_TREE_LOWER_LEAF_DATA_UNITS=144`、`EXTENT_TREE_INTERNAL_FANOUT=147`（`lib.rs:161/154/151`） | ✓ |
| Item5 145 单元 | root level 1／height 2 | 同 | `extent_tree.rs:1130-1146` 测试断言（level 0 两个 + level 1 一个，即 root level=1） | ✓ |

十格全部与冻结副本/kb 快照的真值一致，两种槽数读法（总槽数 vs 单元区槽数）在三种盘面上给出的 R 值也都一致
（配置 A 平凡相同，B/C 两种读法都得到同一个 R）。

**提示未泄题核对**：读 `m2-final-code-r3-local-attack.md` 全文（219 行），逐条与派发的 5 个 item 对照，未出现
812/169/143/144/147 或任何 R 值/根层/树高的具体数字，`### 有意省略的清单` 一节所列五处答案确认不在提示正文
里 ⇒ **✓**。

**转述核对表逐条核**：`Fact 1` 至 `Fact 13`、`Item 1` 至 `Item 5` 的「原文文件:行」独立重新 `sed -n` 核对
（`lib.rs:11-15/62-66/98-99/122-123/136-161/284-285`、`allocator.rs:217-223/1436-1442`、
`allocation_record_tree.rs:85-129/885-911`、`extent_tree.rs:79-101`、`background-final.md:102`）**全部逐字匹配**
腿自己贴出的 grep 产物，也匹配我独立重新执行的 grep（未见任何一处行号错位或摘句）；「多出来的限定词」两条
（Fact 1 的"两个独立命名恰好同值"、Fact 11 整条）与「有意省略」五处均如实标注，未发现藏着未标注的额外限定或
遗漏的省略 ⇒ **✓**。

**运行记录复算**：

```
$ wc -w output-s1.md output-s2.md          # 467 / 333，与运行记录逐字一致
$ python3 research/scripts/corruption-check.py output-s1.md
绿 … cjk=0 words=325 fffd=0 … 各项复读/落单/粘连/自复读计数均为 0
$ python3 research/scripts/oov-check.py output-s1.md
绿 … 生词=1  生词: miscalculating
$ python3 research/scripts/corruption-check.py output-s2.md
绿 … cjk=0 words=212 fffd=0 …
$ python3 research/scripts/oov-check.py output-s2.md
绿 … 生词=2  生词: miscalculating
```

四条命令的退出码与打印内容与运行记录表格逐字一致 ⇒ **✓**。

## 计数与没做什么（本节先写非扫描部分，扫描核对独立成节，见下）

**引用与产物核对计数（不含仍在跑的盘 0/试改两遍扫描，见下一节补齐）**：

- 云端攻方（Opus）：核了 24 处文件:行/产物引用 + 1 处 SHA256SUMS 整体校验 + 1 次 Z17 模型完整重跑 + 3 组
  写死复现重跑 + 1 组冻结重挂完整重跑（160 段）+ 2 个种子完整重跑。**✓ 25 处**（含 SHA256SUMS 校验和逐字节相
  同的 tsv 重跑）；**✗ 1 处**（`second_transaction_step_four_rollback.rs:1130` 被同时当成"40"这个数的出处，
  实际"40"在同文件第 1176 行）。
- 云端正推（Sonnet）：核了 28 处文件:行引用 + 2 个测试二进制完整重跑（20 个测试全过，测试名逐一核对）。
  **✓ 27 处**；**✗ 1 处**（`18-块里携带什么信息.md:52`"整行抄"标签与实际只引末句不符）。
- 本地攻方 + 转述核对表：核了 13 条 Fact + 5 条 Item 的文件:行引用（独立重新 grep，不采信腿自带的 grep 产
  物）、10 格算式与源码常量比对、1 次提示全文未泄题核对、4 条运行记录（词数×2、`corruption-check.py`×2、
  `oov-check.py`×2 共 6 次独立重跑）。**✓ 全部**（13+5+10+1+6 = 35 处均对得上，无 ✗）。

## 没做什么

- 不判任何一条推论打中成不成立、该不该采纳；不核推理本身，只核引用、产物与复跑（本报告开头那句同样适用）。
- Z13 的完整扫描（叶缺席再长 1859 段、树表 0 条一路 576 格、随机历史 11 档合计 2896 段）未独立重跑：这三组
  不在派发提示的「重点核」清单里，只对越格线索的两个种子（`z13_one_seed`）做了独立重跑（见上）。
- Z16-a 的盘 0 抽样与试改扫描（`z16-targeted-dev0-240-384.tsv`、`z16-targeted-fixreads-240-384.tsv`）在草稿
  目录里仍在跑（背景任务，nice 19，8 线程）；已核对**归档产物**（`sha256sum -c` 通过、tsv 行数 3286、16 格槽
  号与盘 1 完全相同）与盘 1（默认）的**独立现场重跑**（逐字节与归档相同），盘 0 与试改两遍的独立现场重跑等
  后台任务结束后另行补一段（见下）。
- 未核 Z13-a/b/c/e 的具体测试断言（叶缺席、树表 0 条、固定点、checker 独立性）逐行数字，只核了 Z13-e 引用的
  三处代码位置（position_addressed.rs、allocation_record_tree.rs、allocator.rs、walk.rs）与 Z13-d 的越格线索：
  这些不在「重点核」清单里，Opus 报告里的具体断言数字（1859、576、2896 等）也未逐一复算。
- 未跑 `checklist-filled.md` 与 `_m2-final-code-r3-checklist.md` 的差异排查之外的背景材料一致性核对（两份文
  件除小节标题里 `./` 前缀这一处纯路径写法差异外，未见抽样比对之外的其它差异；未做逐段全量 diff）。
- 未核 `_m2-final-code-r3-diff.md`、`_m2-final-code-r3-appendix.md` 的内容与冻结副本是否一致（这两份不在四条
  腿报告引用的关键路径上，未被要求核）。
- 未跑门禁、未跑重型测试、未跑全量 `cargo test`；所有 `cargo` 调用都限定到具体测试二进制、`--release`（Opus
  用例）或默认 `test` profile（Sonnet 用例）、线程上限 8（`capped.sh 8`）。

## Z16-a 盘 0 抽样 / 试改扫描：独立重跑结果（补齐上文「待补」）

盘 0 抽样（`R3_OPUS_FAULT_DEVICE=0`）已在草稿目录独立重新编译、重新跑完：

```
$ diff -q logs/z16-targeted-dev0-240-384.tsv .../m2-final-code-r3-opus-model/logs/z16-targeted-dev0-240-384.tsv
（无输出，IDENTICAL）
$ sha256sum 两份 z16-targeted-dev0-240-384.tsv
1cb865d3d0832f93a7b4568e170dd308dfbb3e9fa85e82b09b736f8cf845d240（两份相同，且与盘 1/默认那一遍的 sha256 完全
相同——两块盘在这个两盘对称的池里注入同一形状的故障，产物逐字节相同，这与正文「两次抽样打中同一批 16 格」的
说法一致，而且比"同一批格"这个说法更强：不仅是同一批格，是完全相同的整份 tsv）
```

⇒ **✓**（盘 0 抽样也打中同一批 16 格，且是逐字节的强等价，不只是逐格等价）。

试改扫描（`SINGLEFS_R3_OPUS_DRY_RUN_READS=1`）：本报告落盘时仍在草稿目录后台跑（`nice -n 19`，未设超时、未中
途杀掉，遵照「长活可以等」）。**归档产物**（`m2-final-code-r3-opus-model/logs/z16-targeted-fixreads-240-384.tsv`，
`sha256sum -c` 已通过）已核对：3286 行（666 控制 + 2619 注入），16 格（312/316/324/336 档、槽号与前两遍完全相
同）全部从 `NewFinding[ModelDisagreement...]` 变成 `PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`，
其余 2603 段（1558+54+991=2603）结局不变 ⇒ 与正文「试改在 16 格上全改成取号之前拒，在其余 2603 段上没多拒一
段、也没少做成一段」逐字对得上。**这一条基于归档产物（已通过 sha256 完整性核对）判定 ✓；独立现场重跑仍在跑，
若结果与归档不同会在主 agent 下一次读取本报告前更新（本报告分段追加，若重跑跑完且不一致会另起一段说明）。**

## 补记：试改扫描独立现场重跑已完成（原「仍在跑」现已补齐）

后台任务在交回之后跑完，独立现场重跑的 `logs/z16-targeted-fixreads-240-384.tsv` 与归档产物核对：

```
$ diff -q logs/z16-targeted-fixreads-240-384.tsv .../m2-final-code-r3-opus-model/logs/z16-targeted-fixreads-240-384.tsv
（无输出，IDENTICAL）
$ sha256sum 两份 z16-targeted-fixreads-240-384.tsv
387da053fe37cf70643b4663ece82cc84aefc0128d989d72bd5357e378eaa798（两份相同）
$ awk -F'\t' '($1==312||$1==316||$1==324||$1==336) && $2==24' logs/z16-targeted-fixreads-240-384.tsv | grep -c "PlacementRefusedBeforeAcquisitionMountAdmissionUndecided"
16
```

**逐字节相同**，且这 16 格（312/316/324/336 档、覆盖写 24 次）全部改判成
`PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`（取号之前拒）⇒ 上文「基于归档产物判定 ✓」现已由
独立现场重跑坐实为**逐字节相同的 ✓**，不再是仅凭归档的推定。至此 Z16-a 的三遍扫描（盘 1 默认 / 盘 0 / 试改）
全部在草稿目录里从零编译、独立重跑过，三份产物与归档逐字节相同（sha256 一致），无一处不符。
