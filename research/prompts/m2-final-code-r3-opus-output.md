# m2-final-code-r3 云端攻方（Opus）报告：Z13、Z16、Z17

立场：找反例。代码一律读冻结副本 `/tmp/claude-1000/m2-final-code-r3/tree/crates/`，kb 一律读 `/tmp/claude-1000/m2-final-code-r3/kb-snapshot/`，行号取那份文件自己的。
副本工作区 `/tmp/claude-1000/m2-final-code-r3-opus/`（`tree/` 是冻结副本的拷贝，`target/` 在旁边）；用例与模型最后放 `research/prompts/m2-final-code-r3-opus-model/`。
开工 2026-09-24 23:4x UTC（2026-09-25 08:4x JST）。开工时 `ps` 看到另一个会话的 `cargo test … second_transaction_supplement_three_random_history`（pid 2244926），没有性能测量在跑。

（「各格判定一览」在全部跑完之后追加在文末「各格判定一览（汇总）」一节；分段落盘，先写的在前。）

**复跑**：`bash research/prompts/m2-final-code-r3-opus-model/rerun.sh <空的工作目录>`。每个文件的 sha256 在 `research/prompts/m2-final-code-r3-opus-model/SHA256SUMS`（70 个文件），非日志的 8 个抄在文末「复跑与文件指纹」一节。

## Z17　15 个测试二进制重钉的数：按条款独立算

**做法**：写了一个不读实现、不跑实现的 Python 模型 `model/z17_independent.py`（模型目录见文末），只照三处条款：
D3 已定项 10 ⑤ 的 bump 次序（kb 快照 `03-空间分配.md` 第 200 行）、D8 已定项 14 的实现取值（`08-核心索引结构.md` 第 385、390、394、395 行）、D3 已定项 7 的「释放只改写不删」。
两块盘对称（同槽两份），分配记录树重写集照条款走固定点（只重写内容变了的叶与全部祖先）。复跑：`python3 z17_independent.py`（纯算，1 秒内）。

**原样输出**（2026-09-24 23:5x UTC 这一次）：

```
根层（4 GiB × 2）	2
A 单元数	12
A 落点	[50180, 50240, 50242, 50244, 50245, 50246, 50247, 50248, 50249, 50250, 50251, 50252]
A 分配记录树节点	[('alloc', (0, 0, 61)), ('alloc', (0, 1, 61)), ('alloc', (1, 0, 0)), ('alloc', (1, 1, 0)), ('alloc', 'root')]
A 树表落点	50252
A 单元写次数（两盘）	24
B 落点	[('data', 50182), ('extent_root', 50253), ('inode_leaf', 50254), ('inode_root', 50256), ((0, 0, 61), 50257), ((0, 1, 61), 50258), ((1, 0, 0), 50259), ((1, 1, 0), 50260), ('root', 50261), ('accounting', 50262), ('mapping', 50263), ('tree_table', 50264)]
B 之后分配记录条数（两盘）	52
B 之后已分配槽 / defer 槽 / 空闲槽	(31, 15, 211937)
B 之后空闲字节	3472375808
B 映射条目（码 1 一条 + 码 2 / 码 3：extent 根、inode 叶、inode 根、记账根、分配记录树节点）	10
建 13280 个 inode：分配记录树重写节点	[('alloc', (0, 0, 61)), ('alloc', (0, 0, 62)), ('alloc', (0, 1, 61)), ('alloc', (0, 1, 62)), ('alloc', (1, 0, 0)), ('alloc', (1, 1, 0)), ('alloc', 'root')]
建 13280 个 inode：非容器角色数 / 总角色数	(11, 68)
建 13280 个 inode：叶容器落点首末	(50254, 50366)
B / 写行 / 暖机 / 暖机 / C 各占槽	[14, 10, 8, 8, 14]
被抛弃根独占的槽（逐盘）	54
去掉只被 C 引用的之后	40
回退写行 D（影子账开）落点与节点	[('instance_table', 50368), ((0, 0, 61), 50370), ((0, 0, 62), 50371), ((0, 1, 61), 50372), ((0, 1, 62), 50373), ((1, 0, 0), 50374), ((1, 1, 0), 50375), ('root', 50376), ('accounting', 50377), ('mapping', 50378), ('tree_table', 50379)]
回退写行 D（影子账开）占槽	12
回退写行 D（影子账关）落点与节点	[('instance_table', 50304), ((0, 0, 61), 50306), ((0, 1, 61), 50307), ((1, 0, 0), 50308), ((1, 1, 0), 50309), ('root', 50310), ('accounting', 50311), ('mapping', 50312), ('tree_table', 50313)]
回退写行 D（影子账关）占槽	10
两种「全空段」读法分歧的段起点	[50304]
```

**逐项对钉的数**（测试行号取冻结副本那一份，`grep -nF` 现取）：

| 测试里钉的 | 行 | 独立算的 | 对得上 |
|---|---|---|---|
| 第一个事务 12 个单元、50180, 50240, 50242, 50244, 50245..=50252 | `first_transaction_step_five_publish.rs:405` | A 落点同一串 | 是 |
| 段序列 `(31, "24+2+1+2", 16_777_223)` | `first_transaction_step_five_publish.rs:363` | 单元写 12 × 2 = 24 | 是（24 这一项；闭式里 2^24 + 7 的 7 来自另三段，没动） |
| 树表槽 50252 | `first_transaction_step_six_recovery.rs:77`、`:188` | A 树表落点 50252 | 是 |
| B 的落点 50253..50264、`3_472_375_808`、`"24+2+1+2"` | `second_transaction_step_one_overwrite.rs:154`、`:281`、`:94` | B 落点、空闲 211937 × 16384 | 是 |
| 一次建 13280 个 inode：67 − 11、七个分配记录树节点、68 个单元 | `second_transaction_parallel_line_three_many_inodes.rs:467` | (11, 68)、叶 61 与叶 62 各两块盘 + 第 1 层两个 + 根 | 是 |
| 被抛弃根独占 54、去掉 C 之后 40 | `second_transaction_supplement_two_admission_formula.rs:218`、`second_transaction_step_four_rollback.rs:1130` | 14 + 10 + 8 + 8 + 14 = 54；54 − 14 = 40 | 是 |
| 影子账开着多写 2 | `second_transaction_supplement_two_admission_formula.rs:243` | D 开 12 槽、关 10 槽 | 是 |

**结论**：挑的四组（加 step_one_overwrite 一组）逐项与条款独立推出来的相同，**没打中**。测试注释写的来源（「B 14、写行 10、暖机 8 + 8、C 14」「叶 61 与叶 62」）与独立推的路径一致。
推翻它的现象：有人按条款字面换一种「全空段」读法（见下）或换 bump 次序里同层的排法，重算出不同的槽号。

**顺带看到的两处（不改结论）**：

1. kb 条款里第一个事务的落点例子还是八个单元的旧布局，与这一轮重钉的测试说反话：`03-空间分配.md` 第 212 行 `| t8 | 树表单元（码 2） | — | 50248 | 0 |`（同表第 209 行 t5 分配记录树 50245），`19-块指针的结构与宽度预算.md` 第 201 行写「t8 树表单元 50248（无树归属，序号 0）」；按 D8 已定项 14 推出来树表在 50252、分配记录树五个节点占 50245..50249、出生序号 0..4。背景材料「已知」清单点了 `layout/01-first-txn.md`、E142 装置与 `first_transaction_regions.rs`，**没点这两条已定项里的例子**——同一族，去向该是书记员随 E142 重跑一起改。
2. 「全空段」两种读法在回退那一段分歧：D3 已定项 10 ① 字面是「段内 64 槽都没有未释放分配记录」，实现（`allocator.rs` `segment_is_empty` 看 `used_per_segment`，隔离位另算进开段条件）把被影子账隔离的槽也算「不空」。影子账开着时段 [50304, 50368) 只装着被隔离的槽：字面读法下它是全空段、D 从 50304 开段再绕开隔离位 bump，实现读法下跳到 [50368, 50432)。两种读法下 D 都落进叶 62、多写的都是两片叶，所以 `admission_formula` 钉的「2」不受影响；受影响的只有具体槽号。这是 D3 那一项的读法，不在这一轮的格里，只记线索。

## Z16　取号之前的预演与真发走同一段

**装置**：冻结副本的拷贝 `/tmp/claude-1000/m2-final-code-r3-opus/tree/`，副本专用补丁 `copy-only.patch`（模型目录里）只加环境变量开关，不设时与冻结副本同行为：
`SINGLEFS_R3_OPUS_UNIT_AREA_SLOTS` 改历史执行器 240 槽那一档的单元区宽；`SINGLEFS_R3_OPUS_REPORT` 让 `establish_instance` 把预演与真发逐次比一遍打到 stderr（**含断言跳过的「有一份被隔离」那一格**，不改行为）；`SINGLEFS_R3_OPUS_DRY_RUN_READS` 是试改（见下）。
用例 `r3_opus_z16.rs`，用仓里自己的历史执行器（每一步之后池级 checker、理想模型对拍、panic 接住）与故障注入层。

### Z16-a　坏一份被换下的单元 + 回退到环里最旧的根：预演放行、真发在取号之后被拒（**打中，已知 C542；后果比欠账写的多一样**）

**扫的范围**（用户决定的那几步放开：覆盖写次数、盘宽；回退目标固定取环里最旧，因为这一格只有「这一串自己的根把比它旧的有效根全盖掉」才走得到，见 `mount.rs` 第 1215 行那段注释）：
- 对照：单元区 240..=384 槽每 4 槽一档（37 档）× 覆盖写 18..=35 次 = 666 段，不注入：358 段回退做成、308 段回退在取号之前被 `PlacementRefusedBeforeAcquisitionMountAdmissionUndecided(NoFreeSlotOnAnyDevice)` 拒，预演与真发逐次相同。
- 注入：对照里回退做成的每一段，取它回退那次写行发布释放的每一个槽，在盘 1 上把那个槽起的读一律报错（覆盖写之后、回退之前开），各跑一段：2619 段。

**原样汇总**（`logs/z16-targeted-240-384.tsv`，`awk` 数出来）：

```
666 controls; 2619 faulted runs
   1558 NewFinding[CheckerViolations	Applied:Mounted	true	true
     54 NewFinding[CheckerViolations	Applied:Mounted	true	false
     16 NewFinding[ModelDisagreement	Refused:MountError::Publish(PublishError::PlacementRefused(NoFreeSlotOnAnyDevice))	-	-
    991 Completed	Applied:Mounted	q=false	same=true
```

（列：结局、回退那一步、这一串有没有一份被隔离、预演与真发逐次相同否。）
- 16 段：**预演放行、真发在取号之后被落点拒**。落在单元区 312 / 316 / 324 / 336 槽 × 覆盖写 24 次，坏的槽是写行那次换下的分配记录树节点或记账、映射单元（312 档 50245..=50248；316、324 档 50306..=50309；336 档 50312..=50315）。
- 54 段：有一份被隔离、预演与真发取的落点不同，但回退照样做成（断言按设计跳过这一格，`mount.rs` 第 1763 行 `if let (Some(planned), false) = (&placements_planned, some_copy_was_quarantined) {`）。
- 1558 + 54 段的 `CheckerViolations`（I-3.11，部分带 I-3.1）是装置造出来的，不算：注入的读错只在实现那一侧，checker 读镜像时那一份读得出且对得上，按 I-3.1 / I-3.11 的隔离豁免字面（「每一份都读得出且对得上，照旧判违例」）判违例。

**写死复现与后果**（`z16_rollback_refused_after_acquisition_burns_instance_generations_by_hand`，历史执行器停在第一次模型对不上之后接着手动走；原样输出，312 档那一格，其余两格同形，日志 `logs/z16-hand-*.log`）：

```
R3OPUS-HAND prefix ending=Completed last=Some(Refused { member: "PublishError::PlacementRefused(NoFreeSlotOnAnyDevice)" })
R3OPUS-HAND rollback attempt 0 to (instance 1, txg 3): to_acquire before=InstanceGeneration(3) after=InstanceGeneration(4) writes+barriers issued=28 result=Publish(PublishSequenceFailed { cause: PlacementRefused { unit: TreeTable, refusal: NoFreeSlotOnAnyDevice }, writes_of_persisted_publishes: [WritesByStructureKind { counted: {AllocationRecordTreeNode: WriteCallsAndBytes { write_calls: 10, written_bytes: 163840 }, AccountingTreeNode: WriteCallsAndByt
R3OPUS-HAND rollback attempt 1 to (instance 1, txg 3): to_acquire before=InstanceGeneration(4) after=InstanceGeneration(4) writes+barriers issued=0 result=RollbackTargetNotACandidate { target: RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(3) }, exclusion: NotInRing }
R3OPUS-HAND writable attempt 0: to_acquire before=InstanceGeneration(4) after=InstanceGeneration(4) writes+barriers issued=0 result=PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { instance_to_acquire: InstanceGeneration(4), publish_index: 0, warm_up_publishes_planned: 1, unit: TreeTable, refusal: NoFreeSlotOnAnyDevice }
```

历史：第一个文件 → 可写挂载（实例 2）→ 空内容覆盖写 24 次（第 24 次已被落点拒：池本来就满了）→ 盘 1 槽 50245 持久读不出 → 管理员回退到环里最旧的根 (实例 1, txg 3)。每一步许可它的条款：回退目标在候选集里（D16 已定项 1）；读不出按对不上处置、那份的记录留在已分配（D19 已定项 5）；预演不做释放前读盘核（D18 已定项 11「可写挂载的顺序」，kb 快照 `18-块里携带什么信息.md` 第 314 行「预演不做释放前读盘核，差别见 C542（取号前预演不读盘核，坏盘加回退到最老根会烧号）」）。

**这是 C542**（`checks-owed.md` 第 475 行，用户 2026-09-24 定记欠账、不改），不是新形态。量出来多的一样：C542 那一栏写「真发在取号之后被落点拒绝，烧掉一个实例代号，不丢数据」；实测拒的是**暖机**那一次，**写行那次发布已经落盘**（28 次写与屏障，`writes_of_persisted_publishes` 一项），它的根盖掉了回退目标那一槽——之后再试回退报 `NotInRing`，管理员没法重试同一条回退；盘上留下的是一条没暖机覆盖两块盘的新实例根（实例 3）。「不丢数据」照旧成立（写行那一版就是回退目标那一版加一行），但「只烧一个号」少说了「回退已经半生效、目标根离环」。C542 那一栏「造这一格」要的用例，这三格可以直接拿去（单元区 312 槽、覆盖写 24 次、盘 1 槽 50245）。

**四句**：
1. 分不分辨臂：这一轮没有跑前的臂；它分得开「预演读盘核」与「预演不读」两种做法（下面量了）。
2. 系统看不看得到：看得到——持久坏扇区在取号之前就读得出「读不出」，预演选择不读。只有「预演读时好、真发读时坏」的瞬时故障看不到，那一半任何做法都没法在取号之前判。
3. 满足的分句：D18 已定项 11「这次挂载要发的写行与暖机在分配器的副本上预演取得到全部落点」——预演取得到、真发取不到；条款同一句已经写明「差别见 C542」，所以按字面不算违反条款，是条款自认的缺口。
4. 改法在打中的格上还中不中：C542 没给改法（用户定不改）。我在副本上试了一个（下一段）。

**试改（只在我的模型上量过、被攻过零轮）**：预演也按 `ReleaseChecksumCheck::ReadEveryReplacedCopyBeforeReleasingIt` 读盘核（`SINGLEFS_R3_OPUS_DRY_RUN_READS=1`，改 `mount.rs` 两处）。三格原样输出（`logs/z16-hand-fix-reads-*.log`）：

```
R3OPUS-HAND rollback attempt 0 to (instance 1, txg 3): to_acquire before=InstanceGeneration(3) after=InstanceGeneration(3) writes+barriers issued=0 result=PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { instance_to_acquire: InstanceGeneration(3), publish_index: 1, warm_up_publishes_planned: 1, unit: TreeTable, refusal: NoFreeSlotOnAnyDevice }
R3OPUS-HAND rollback attempt 0 to (instance 2, txg 4): to_acquire before=InstanceGeneration(3) after=InstanceGeneration(3) writes+barriers issued=0 result=PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { instance_to_acquire: InstanceGeneration(3), publish_index: 1, warm_up_publishes_planned: 1, unit: TreeTable, refusal: NoFreeSlotOnAnyDevice }
R3OPUS-HAND rollback attempt 0 to (instance 2, txg 5): to_acquire before=InstanceGeneration(3) after=InstanceGeneration(3) writes+barriers issued=0 result=PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { instance_to_acquire: InstanceGeneration(3), publish_index: 1, warm_up_publishes_planned: 2, unit: TreeTable, refusal: NoFreeSlotOnAnyDevice }
```

三格都改成在取号之前拒、号不涨、一个写都不发（量过）。代价（推的，没量）：C542 那一栏写的「挪动已有故障注入用例的注入点」——预演多读每个换下单元两份，读调用序号全变；读放大是每次可写挂载多读写行那一串换下单元的字节。瞬时故障那一半它修不了（推的）。

### Z16-b　其余几种形态（**没打中**）

| 形态 | 扫了什么 | 结果 |
|---|---|---|
| 带文件、小容量（第二轮 Z12 偏薄那一半） | 上面 666 段对照：单元区 240..=384 槽 × 覆盖写 18..=35 × 回退到最旧 | 预演与真发逐次相同 666 / 666；被拒的 308 段都拒在取号之前 |
| 回退到环里最旧的根 | 同上（回退目标就取最旧） | 同上 |
| 回退之后一路坏一份单元（持久） | 上面 2619 段注入 | 除 Z16-a 的 16 段外，被拒的一段都没有；预演与真发不同的 54 段都是「有一份被隔离」、断言按设计跳过、回退做成 |
| 冻结着一次失败时挂载 | `z16_mount_after_a_frozen_publish_by_hand`：前缀（第一个文件 → 挂载 → 覆盖写 3 次 → 挂载）之后本进程覆盖写一次、第 k 次写或屏障报错（k = 1..=40）→ 同进程再发被拒 → 放手 → 同一批盘可写挂载 → 覆盖写 → 池级 checker；4 GiB 与 384 槽两档 × 写 / 屏障两种 = 160 段 | 冻结的都在同进程第二次发布时报 `PublishFrozenAfterAWriteFailureIsNotResentYet`；160 段重挂全做成、之后覆盖写做成、checker 全绿；预演行 328 条、逐次不同 0 条 |

原样计数（`logs/z16-frozen2-*.log`，每个文件 `grep -c 'R3OPUS-DRYRUN.*same=false'` 都是 0；各 82 条预演行）。k 越过这次发布的写数时注入落在后面的重挂里、第一次覆盖写照成，那几段不算冻结格（表里 160 段包含它们：写 18 + 屏障 72 段属于这一种）。

**第一个问句「共用代码之后这条断言还剩什么判别力」**（推的，按代码读，没做变异）：
- 两边共用 `prepare_the_version_publish` / `prepare_the_row_publish_on_a_version_without_file` / `prepare_the_publish_without_units` 与同一张计划，断言只剩判「两边的**输入**与**胶水**一样」：进来的分配器（取号不碰它）、预演循环与真发循环的次序与 txg、零单元发布之后记根（`record_root_written_by_this_process`）、释放前读盘核（设计上不同，已跳过）。
- 断言只比槽号（`placements_taken_by` 交回 `(角色, 槽)`），不比分配代、出生序号、记账行；预演里某次计划的 txg 或 counter 写错而槽号不变，它看不见——但那种错也不改变「取不取得到」，对准入无害。
- 预演里那两处 `copy.record_root_written_by_this_process`（`mount.rs` 第 1274、1327 行）对断言是死的：它们只在树表 0 条的一版上跑，而那一版之后的暖机都是零单元发布、不取任何落点，拷贝上回收了什么都不会再被发出去。删掉它们断言不会红（推的）。
- 所以这条断言今天能判出的只剩「将来有人让预演循环与真发循环分叉」（多推 / 少推一次暖机、计划函数换一个）。

**没做的形态**：分配记录树「在这一次长一层」——根层由池几何定死（D8 已定项 14），不随记录多少变；能变的是根之下多出一个第 1 层节点。4 GiB 盘上第 1 层第 1 格从槽 137228 起，要先占掉 8.7 万槽才走得到，没造；小盘（根在第 1 层）上多出一片叶就是根多一个孩子，上面 240..=384 槽的扫描都走过（叶 62 从槽 50344 起）。

## Z13　分配记录树按位置寻址

用例 `r3_opus_z13.rs`，同样在冻结副本的拷贝上跑、每一步之后池级 checker。根层公式、W、扇出那几样算术归本地攻方，这里不碰。

### Z13-a　缺席即全空闲：叶先有、回退后缺席、再长出来（**没打中**）

- **历史**（`z13_leaf_absent_then_regrown_after_rollback`，4 GiB 两块盘）：第一个文件 → 可写挂载 → 覆盖写 n1 次 → 回退到环里第 k 新的根 → 覆盖写 m 次 → 可写挂载 → 覆盖写 → 冷启动读回。**用户决定的 k、m 整段放开**：n1 = 4..=14，k = 0..=min(23, n1 + 3)，m = 0..=12，共 1859 段。
- **结果**（8 个分片原样末行各一条，合计）：1859 段全 `Completed`、回退全 `Applied`，没有 checker 违例、模型对不上与 panic。
- **走没走到那一格**（`z13_regrow_coverage_probe` 取 5 格，每一步之后从盘上读所选根下盘 0 的记录落在哪几片叶）：叶 62 在第 2 步（第二次覆盖写）长出来；n1 = 4、k = 3 那一格回退目标是第 1 步之后的根，那一版里叶 62 缺席（`1:{61}`），回退那次写行发布按那一版的节点起算、又把叶 62 长出来（影子账把被抛弃的槽隔离，写行的落点被推到叶 62）。原样一行：`R3OPUS-Z13-PROBE n1=4 k=3 m=6 ending=true leaves=S:{61} 0:{61} 1:{61} 2:{61, 62} 3:{61, 62} …`（全文 `logs/z13-regrow-probe.log`）。
- **一片叶能不能被删空**（推的，按代码读）：记录只在 `make_room_for_record_on_device` 里删（`allocator.rs` 第 619 行 `records.retain(|record| !(record.device == device && record.slot == record_slot));`），删的是新落点罩住的已回收记录，而新落点的记录落进同一片叶（两槽单元起在偶数槽、W = 812 偶数，同一次分配的起点与被罩的那条同叶）。所以带文件的一路上一片叶只会在「换到一个更旧的版本」（回退）时变成缺席，上面扫的就是这一路。

### Z13-b　树表 0 条那一版：实例表多片 + 回退 + 再长（**没打中**）

- **历史**（`without_file::z13_without_file_versions_rollback_and_regrow`，4 GiB）：mkfs → 可写挂载 → 取号之后崩溃 a 次（a ∈ {0, 368, 369, 738, 1200, 23985}：一片、两片的边界、四片、66 片）→ 可写挂载 m1 ∈ {1, 3, 6} 次 → 回退到环里第 j 新的根（j = 0..7）→ 可写挂载 m2 ∈ {0, 1, 3, 6} 次；每次挂载之后池级 checker。共 576 格。
- **结果**：504 格一路做成、checker 全绿，72 格环里没有第 j 条根（`no-such-ring-root`，不算）。没有一次挂载被拒、没有 panic。这一路上每次挂载都过 `establish_instance` 那条「预演与真发逐次相同」断言（产品代码里 panic 就会记成 PANIC），所以 Z16「树表 0 条那一版写行、66 片实例表」那一格在回退形态上也算扫过。

### Z13-c　固定点（推的，按代码读；随机历史见 Z13-d）

- 收拢条件是 `holding_records == nodes_after && changed.is_subset(&rewritten_by_the_plan)`（`transaction.rs` 第 4691 行，树表 0 条那一路第 1197 行同一句）。产品路径上每轮只多分配、不删叶（上一条），`nodes_after` 与重写集只增不减，上界是池里的节点数，64 轮（第 4709 行）在 4 GiB 盘上远用不完。
- 「收拢到一个与真发不同的重写集」由 `publish_admitted` 里那条断言兜（第 5590 行起，`nodes_holding_records(&allocation_geometry, &allocation_records)` 与 `changed ⊆ rewritten`），被隔离那一格记录留在已分配、叶的集合不变，只会让变了的更少。上面 Z16 与 Z13 全部扫描（4 GiB 与 240..=384 槽，合计 5000 段以上）没有一次在这条断言上 panic。
- 没造 `ReuseWindow::ForcedToZero` 那一格：只供测试的开关，C549 已立账。

### Z13-e　checker 那份几何是不是真的独立（推的，按代码读）

- checker 的 `position_addressed.rs` 只从格式常量模块取 `ALLOCATION_RECORD_TREE_LEAF_SLOTS`、`ALLOCATION_RECORD_TREE_INTERNAL_FANOUT`（第 10–14 行的 `use singlefs_format::{…}`），key 字节、格的区间、根层、孩子判定、叶判定都自己写，没有 `use singlefs_core`。
- 根层的输入三处同口径：写者 `UNIT_AREA_START_SLOT + device_map.unit_area_slots()`（`allocation_record_tree.rs` 第 124 行）= `device_bytes / 16384`（`allocator.rs` 第 222 行 `unit_area_slots_of_device` 的定义），读者 `/ SLOT_BYTES`（第 146 行），checker `self.reader.device_bytes(*device).unwrap_or(0) / SLOT_BYTES`（`walk.rs` 第 1307 行）。盘字节数不是 16384 的整数倍时三处都向下取，一致。
- 不独立的只有「读条款的方式」：两份都是同一句 Σ⌈槽数 ÷ S_{R−1}⌉ ≤ 169 的直译，条款句子写错两边一起错——这是 D13 已定项 5 允许的形态（只共享格式常量），不算打中。
- 两份都**不判**根的孩子格落不落在那块盘的槽数之内（根的孩子只核「槽号在格点上、盘在池里」）：盘上被改坏成一条罩着盘末尾之外的格，读者与 checker 都照走下去，接住它的是叶里记录的单元区判定。这一格只有坏盘面走得到，不是合法历史，只记线索。

### Z13-d　随机历史（仓里自己的执行器，每一步之后池级 checker，种子与门禁那一批不同；两次抽样）

原样末行（`logs/z13-campaign-*.log`、`logs/z13-campaign2-*.log`）：

```
R3OPUS-CAMPAIGN weights=broad width=4g first_seed=3000000000 seeds=64 ops=40 new_findings=0 known_red=4 most_records=842 secs=11.7
R3OPUS-CAMPAIGN weights=rollback width=4g first_seed=3000000000 seeds=64 ops=40 new_findings=0 known_red=2 most_records=670 secs=5.9
R3OPUS-CAMPAIGN weights=reuse width=384 first_seed=3000000000 seeds=64 ops=60 new_findings=0 known_red=3 most_records=708 secs=17.9
R3OPUS-CAMPAIGN weights=unitwall width=256 first_seed=3000000000 seeds=32 ops=100 new_findings=0 known_red=0 most_records=462 secs=3.3
R3OPUS-CAMPAIGN weights=wall width=4g first_seed=3000000000 seeds=16 ops=150 new_findings=0 known_red=1 most_records=1358 secs=21.9
R3OPUS-CAMPAIGN weights=broad width=4g first_seed=4000000000 seeds=640 ops=40 new_findings=0 known_red=23 most_records=884 secs=100.9
R3OPUS-CAMPAIGN weights=rollback width=4g first_seed=4000000000 seeds=640 ops=40 new_findings=0 known_red=9 most_records=824 secs=151.8
R3OPUS-CAMPAIGN weights=reuse width=384 first_seed=4000000000 seeds=640 ops=60 new_findings=0 known_red=24 most_records=714 secs=205.9
R3OPUS-CAMPAIGN weights=unitwall width=256 first_seed=4000000000 seeds=320 ops=100 new_findings=0 known_red=0 most_records=466 secs=32.4
R3OPUS-CAMPAIGN weights=wall width=4g first_seed=4000000000 seeds=96 ops=200 new_findings=0 known_red=4 most_records=1434 secs=165.1
R3OPUS-CAMPAIGN weights=reuse width=240 first_seed=4000000000 seeds=320 ops=60 new_findings=1 known_red=4 most_records=454 secs=55.0
```

- 已知红全是清单第 0 条（增补 2 收口表第 43 行）。分配记录多于 812 条（多叶）的版本在 4 GiB 的几档都走到了（最多 1434 条）。
- **Z13 这一格两次抽样都没打中**：没有一条落在分配记录树的位置核（I-1.1）、I-3.10、读树报错或 panic 上。
- **越出这一格的一条线索**（最后一行那一档，240 槽小盘、偏向抬 F 之后复用）：新签名 `CheckerViolations { invariants: ["I-3.1"] }`，种子 4000000045、4000000204 两段。逐步复看（`z13_one_seed`，`logs/z13-seed-4000000045.log`、`-4000000204.log`）两段同形：池已被写满（覆盖写报 `PlacementRefused(NoFreeSlotOnAnyDevice)`）之后抬 F，报 `MountError::RaiseFloorSequencePublishFailed(publishes_persisted = 1, PublishError::PlacementRefused(NoFreeSlotOnAnyDevice))`——**抬 F 那一串落了一条带新 F 的根、第二条取不到落点**；之后 I-3.1：记账的已分配比遍历全部有效根多 8 槽（131072 字节；第一段 `记账的已分配 Some(2031616)，遍历全部有效根得到 1900544`，第二段 `Some(3244032)` 对 `3112960`）。它没被已知红第 0 条接走：那一条要「F 落在回退留下的空档里」。机理我没追（不在我的格里），只交线索：复跑 `R3_OPUS_SEED=4000000045 R3_OPUS_WEIGHTS=reuse R3_OPUS_WIDTH=240 <r3_opus_z13 二进制> --exact z13_one_seed --nocapture`。与分配记录树按位置寻址有没有关系没判：没在第二轮冻结树上复跑同一个种子。

### Z16-a 补：第二次抽样与试改的整轮扫描

同一批 666 段对照、2619 段注入，各跑一遍（`logs/z16-targeted-dev0-240-384.tsv`、`logs/z16-targeted-fixreads-240-384.tsv`）：

| 扫描 | 结局 × 回退那一步 × 有无隔离 × 预演与真发逐次同否 | 段数 |
|---|---|---|
| 注入换到盘 0（第二次抽样） | 模型对不上 × `MountError::Publish(PlacementRefused(NoFreeSlotOnAnyDevice))`（取号之后被拒） | 16（与盘 1 同一批格：312 档 50245..=50248、316 / 324 档 50306..=50309、336 档 50312..=50315） |
| 同上 | 回退做成 × 有隔离 × 不同 | 54 |
| 同上 | 回退做成 × 有隔离 × 相同（checker 判的是装置造出的 I-3.11 / I-3.1，见上） | 1558 |
| 同上 | 做完 × 无隔离 × 相同 | 991 |
| 盘 1 注入 + 试改（预演读盘核） | 做完 × `PlacementRefusedBeforeAcquisitionMountAdmissionUndecided(NoFreeSlotOnAnyDevice)`（取号之前拒） | 16（同一批格） |
| 同上 | 回退做成 × 有隔离 × **相同** | 1612（= 1558 + 54：原先预演与真发不同的 54 段全变成相同） |
| 同上 | 做完 × 无隔离 × 相同 | 991 |

两次抽样打中同一批 16 格；试改在这 16 格上全改成取号之前拒（量过），在其余 2603 段上没多拒一段、也没少做成一段（量过，只在我的副本上）。

## 各格判定一览（汇总）

| 格 | 判定 | 依据一句 |
|---|---|---|
| Z17 | **没打中** | 独立模型只照 D3 已定项 10 ⑤、D8 已定项 14、D3 已定项 7 推，第一个事务 12 个单元 50180 / 50240..=50252、树表 50252、B 的 50253..=50264 与空闲 3472375808、建 13280 个 inode 的 67 − 11 与七个节点、被抛弃根 54 / 40、影子账开着多写 2，逐项与测试钉的相同。顺带：D3 已定项 10 第 212 行、D19 第 201 行的第一个事务落点例子仍是八个单元的旧布局（50248），背景「已知」清单没点它们 |
| Z16 | **打中一格，已知 C542**；后果比欠账那一栏多一样 | 单元区 312 / 316 / 324 / 336 槽、覆盖写 24 次、坏一份写行要换下的单元、回退到环里最旧的根：预演放行，真发取号之后写行已落盘、暖机被落点拒，号烧掉一个，回退目标那一槽被写行的根盖掉（之后再试回退 `NotInRing`）。两次抽样同一批 16 格。试改「预演也读盘核」在 16 格上改成取号之前拒（只在我的模型上量过、被攻过零轮） |
| Z16 其余形态 | **没打中** | 带文件小容量 666 段、回退到最旧、冻结之后重挂 160 段，预演与真发逐次相同；「长一层」没造 |
| Z13 | **没打中**（两次抽样） | 叶缺席再长 1859 段、树表 0 条那一路（到 66 片实例表）加回退 576 格、随机历史 11 档，没有一条落在分配记录树的位置核、读树或 panic 上；checker 几何只共享格式常量 |
| 越格线索 | 交主 agent | 240 槽小盘上抬 F 那一串落一条根、第二条取不到落点之后 I-3.1 记账多 8 槽（种子 4000000045、4000000204），不在已知红清单里，机理没追 |

**打中那一格的四句**见 Z16-a。**推翻条件**：Z16-a——有人在冻结副本上用同一段历史（`rerun.sh` 里的写死复现）跑出取号之前被拒，或号不涨；Z17——有人按同一组条款换一种读法（全空段、同层排法）推出别的槽号且那种读法才是条款本意；Z13——换一组种子或更大的盘面扫出分配记录树的位置核红。

**试改各修哪一格**：

| 改法 | Z16-a 那 16 格 | 其余 2603 段 | 瞬时故障（预演读时好、真发读时坏） |
|---|---|---|---|
| 预演也读盘核（我的） | 修（量过） | 不变（量过） | 不修（推的） |
| C542 现状（不改） | 不修 | — | 不修 |

## 没打中的形状（试过什么、取样范围）

- Z16：带文件小容量对照 666 段（单元区 240..=384 每 4 槽 × 覆盖写 18..=35，回退到最旧）；持久坏一份单元 2619 段 × 两块盘；冻结之后重挂 160 段（4 GiB、384 槽 × 写 / 屏障 × k = 1..=40）；第一个事务起单盘 384 槽槽级扫描 384 段（`logs/z16-384-dev1/`，8 段有隔离、预演与真发都相同）。
- Z13：叶 62 缺席再长 1859 段（n1 4..=14、k 0..=min(23, n1+3)、m 0..=12）；树表 0 条一路 576 格（a ∈ {0, 368, 369, 738, 1200, 23985}、m1 ∈ {1, 3, 6}、j 0..7、m2 ∈ {0, 1, 3, 6}）；随机历史 11 档 2896 段（种子基 3000000000、4000000000）。
- Z17：四组数加 step_one_overwrite 一组，一次独立推导。Z17 是纯算，同一模型复跑结果不变，第二次抽样不适用。

## 这条腿自己的限度

- 副本上的数不算入库装置上的数；Z16-a 的复现与试改要在入库装置上重做才能引。
- Z16 的注入层是「那一槽的读一律报错」，而 checker 读的是镜像本身，所以隔离之后 checker 恒判 I-3.1 / I-3.11 违例（装置造出的，不算）；想让 checker 也看见坏的那一份，要在镜像上真改坏字节，我没做。
- Z16「分配记录树长一层」（4 GiB 上第 1 层第 1 格要占 8.7 万槽）没造。
- Z17 模型与实现同样按「全空段 = 段里没有被占着的槽（含隔离）」读，这一步与实现共用了读法；按条款字面另一读法算出的只有回退写行那一次槽号不同（50304 起 vs 50368 起），测试钉的数都不受影响。
- 越格那条 I-3.1 线索只看了两个种子的逐步结局，机理、是否实二一引入都没判。
- 干到一半收到主 agent 两条消息：一条是共用约束第 57 行检出 hook 加了第五种拒绝（我的写都在 `research/prompts/` 与 `/tmp`，没受影响，照旧做）；一条是整点询问，已写 `/tmp/claude-1000/m2-final-code-r3-opus/progress.md`。

## 没做什么

- 不判 Z14、Z15、Z18（正推腿）与算术格（本地攻方）；没跑任何层 0、门禁整轮或全量测试，只编、跑了自己的两个测试二进制（`r3_opus_z13`、`r3_opus_z16`，release）。
- 没改主工作区的 `crates/`；改动只在 `/tmp/claude-1000/m2-final-code-r3-opus/tree/`（补丁 `copy-only.patch`）。
- 没替主 agent 采纳试改；没在第二轮冻结树上复跑越格线索的种子。

## 复跑与文件指纹

复跑：`bash research/prompts/m2-final-code-r3-opus-model/rerun.sh <空的工作目录>`（拷冻结副本 → `patch -p2` 打副本补丁 → 放用例 → 编 → 依次跑 Z17、Z16 写死复现与试改、Z16 三遍扫描、冻结重挂、Z13 各段；全程约 40 分钟，8 线程）。
模型目录 `research/prompts/m2-final-code-r3-opus-model/` 共 70 个文件，`SHA256SUMS` 里逐个列着；非日志的 8 个：

```
248e48c67f4d8bf2ace0f8e391d4ff002e1b5cf679d9a76ed115e7708d3405c4  ./copy-only.patch
c72fd0dcca08d05f813e05e398efb9dccf10d3ed732368a1074238da40a2cd89  ./model/z17_independent.py
3c7c4159099f239a2cbe174a6d4a648431560edf318b20090bbcf0d3ac05f8f5  ./rerun.sh
2e7ca0e3cada483058ec6ac4800335f1801d7055f5de61df328eb1dded0999ce  ./run_z16_controls.sh
f75d2dd04d04fbe3b1c2d73d7fb127abebe5cc31ba49d6fb8aa43275045fec70  ./run_z16_sweep.sh
730208a4941edbe67f3a374f157b278fd7dd31727aadce911acfe5f895590788  ./tests/r3_opus_z13.rs
c0a80cb79ae36c505331c74c6986b0fba325223187e102ccfe962a6d429c1dd5  ./tests/r3_opus_z16.rs
a0564d270a683f934cf258b3818297c94bb173dd98d66f65952da0eff7c2a586  ./z16_targeted.py
```
