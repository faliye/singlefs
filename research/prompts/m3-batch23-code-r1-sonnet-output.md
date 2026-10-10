# m3-batch23-code-r1 正推腿（Sonnet）报告

日期 2026-10-06。轮 m3-batch23-code-r1（删除批第 23 批合入的代码，代码轮第一轮），立场正推，格 K1（N1–N8）与 K3（13 份每份一格）。
引文约定：crates 的文件在冻结副本 `/tmp/claude-1000/m3-batch23-code-r1/tree/` 里，行号是那份副本里 `grep -nF` 现取的；kb 条款在仓里 `.claude/kb/invariants.md`，行号同样现取。
做法：先从 kb 条款与 `walk.rs`（CPU 实现，今天的 `crates/`）推出 GPU 与测试「应该是什么」，再与合入的代码逐格比；CPU 侧的预期红集用 harness 档探针在冻结副本的拷贝上现算（GPU 不跑）。

## 0 材料核对

冻结副本与快照清单（`cd tree && sha256sum -c …/crates-sha256.txt`，原样输出）：

```text
crates/mutations.tsv: OK
crates/singlefs-checker/src/walk.rs: OK
crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs: OK
crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl: OK
crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl: OK
crates/singlefs-checker-tier/src/crash_judge_checker.wgsl: OK
crates/singlefs-checker-tier/src/crash_judge_common.wgsl: OK
crates/singlefs-checker-tier/src/crash_judge_dispatch.rs: OK
crates/singlefs-checker-tier/src/crash_judge_gpu.rs: OK
crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs: OK
crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs: OK
crates/singlefs-harness/tests/checker_known_bad_images.rs: OK
crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs: OK
```

模型目录 `research/prompts/m3-batch23-code-r1-sonnet-model/`：`probes/zs_probe.rs`（CPU 侧探针，10 个测试，共 23 张镜像上的判定）、`anchors.py`（25 行变异的锚点核对）、`mutate_one.py`（把表里的行施加到副本再跑）、`rerun.sh`、`expected-summary.txt`（456 行，sha256 `ff56b1f4…`）。
`rerun.sh <空目录> [步名…]` 在两个互不相干的空目录里各从头跑过一遍（编译 6G、跑 4G，经 `run-with-memory-cap.sh`，线程 7），第二遍 `summary.txt` 与 `expected-summary.txt` 逐字节相同：

```text
SUMMARY_MATCHES lines=456
ff56b1f4104f2ae0dcebb4c7a27a3d347a63210547419841f674387483fd55f8  summary.txt
ff56b1f4104f2ae0dcebb4c7a27a3d347a63210547419841f674387483fd55f8  expected-summary.txt
```

（前台一次放不下，每遍分两次调用：先 `anchors n6 t13 t9 t10 fb t3b t1 lb q17 n5 tests bin`，再 `mut-n5 mut-n8 summary`；不是 `rerun.sh` 不带步名的一次跑。）

## 1 K1 · N1：GPU 八条由内核置位（走读内核 I-9.17、I-9.23；扫描内核 I-9.24、I-9.18–I-9.21、I-9.29）

### 1.1 走读内核：I-9.17（谱系树）与 I-9.23（墓碑树）

条款（`.claude/kb/invariants.md` 304、310 行）两条同形：类型段 ∈ {0, 记录类型}；类型段 0 ⇒ 子单元是码 2、身份引用出生树 / 容器号 / 出生代三段为 0；类型段 = 记录类型 ⇒ 子头四段与身份引用逐字相等、子指针出生树相等、类标签 3；其余判该条目损坏。「身份不符单独判定，不再记走读失败」（`.claude/kb/invariants.md:304`）
CPU 与 GPU 逐支对（CPU 读了 `walk.rs` 2067–2287 行，GPU 读了 `crash_judge_checker.wgsl` 2891–3010 行）：

- 选哪条不变量：GPU 按记录类型选，「var identity_invariant = INVARIANT_I_9_23;」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2909`），谱系树改 I-9.17；CPU 由调用处传入（`walk.rs` 1992–2011 行）。下标 74 = I-9.17、80 = I-9.23 与 `IMPLEMENTED_INVARIANTS` 次序一致（见 1.5）。
- 类型段 0：CPU 「let entry_holds = birth_tree == 0」（`crates/singlefs-checker/src/walk.rs:2130`）；GPU 「w_judge(identity_invariant, is_zero64(rd64(n.base, o + 8u))」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2950`）。孩子「是码 2」那一小句两边都在读不出时放过：CPU 「.is_none_or(|class| class == 2)」（`crates/singlefs-checker/src/walk.rs:2133`），GPU 「let child_is_a_code_two_node_or_unreadable = child_unit.version == NONE || rd8(child_unit.base, 6u) == 2u;」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2949`）。判过身份之后才看层级：CPU 「if node.level <= 1 {」（`crates/singlefs-checker/src/walk.rs:2191`），GPU 「if (n.level <= 1u) {」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2952`）（层级 ≤ 1 记走读失败、不往下走），两边先判后记。
- 类型段不认得：CPU 先登记引用再判红，孩子的形状由这一层定，「let span_in_slots = if node.level >= 2 { 1 } else { 2 };」（`crates/singlefs-checker/src/walk.rs:2162`）；GPU 同形，「w_note_pointer_references(child, span_of_the_unregistered_child);」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2982`），随后 「w_judge(identity_invariant, false);」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2983`）。
- 类型段 = 记录类型：CPU 「let container_entry_holds = unit[6] == 3」（`crates/singlefs-checker/src/walk.rs:2260`）（容器头四段、`header.1 == record_type`、子指针出生树）；GPU 「let identity_holds = rd8(s.base, 6u) == 3u」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2994`），判在「走没走过」之前：「w_judge(identity_invariant, identity_holds);」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2997`）。`header.1 == record_type` 在 GPU 里并进 `rd16(s.base, 51u) == entry_type`（这一支里 `entry_type == record_type`），等价。
- 读不出容器：CPU 与 GPU 都记走读失败、不判身份（CPU 2243–2253 行，GPU 2987–2991 行）。

结论：四支同判同位。预期红集用 CPU 探针现算（第 4 节），T1（谱系条目容器号加 1）红 I-9.17，T2（类型段 7）红 I-9.23，T3（类型段 0）红 I-9.23 并连带走读失败的 I-4.8、I-7.2、I-7.4（条款写「再因为层级 1 的节点里出现类型段 0 记走读失败」）。
推翻它的现象：某个合法流上 GPU 在 I-9.17 / I-9.23 位上与 CPU 不同；或 GPU 对一个坏条目置位而 CPU 不红（或反过来）。本轮没有跑 GPU，这一格是读码结论。

### 1.2 扫描内核：删除态扫描里的 I-9.24，与墓碑 / 谱系那一遍里的 I-9.18、I-9.19、I-9.20、I-9.21、I-9.29

读了 `walk.rs` 8281–8302、8602–8769（队列与 I-9.24）、9040–9923（墓碑 / 谱系那一遍）、10557–10593 行，与 `crash_judge_checker_scan.wgsl` 2320–2364、2585–3064 行逐句对。

- 版本清单：CPU 候选集每条根加由记录施加出来的几版，「let versions_to_scan_the_deletion_state_of: Vec<VersionToScanTheDeletionStateOf<'_>> =」（`crates/singlefs-checker/src/walk.rs:10557`）；GPU `judge_tombstones_and_the_lineage_of_the_versions` 循环 `c_candidate_count` 条候选根与 `c_version_count` 个记录版，树表指针取根偏移 36 与记录新根段打头，txg 取 `root.txg` / `record.txg`（`crash_judge_checker_scan.wgsl` 3054–3064 行）。CPU 按 (树表, txg) 去重只扫一次，GPU 不去重——违例位是并集，跨版本表按 (树, 号) 去重，不改结果；「CPU 按树表单元去重只扫一次，违例位是并集，这里不去重。」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2064`）。
- I-9.24：CPU 读队列为号集，「queue.len() <= 1,」（`crates/singlefs-checker/src/walk.rs:8627`）（队列读不出记判不了，不红）；GPU 读得出才判，读法同 CPU（根全零是空队列；层级 0、key 宽 8、条目宽 ≥ 8）：「|| read.queue.entry_width < DELETED_INODES_ENTRY_BYTES) {」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2336`），判「不同的号至多一个」：「fn d_deletion_queue_holds_at_most_one_key(read: DeletionQueueRead) -> bool {」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2344`）（所有条目都等于第一条 ⇔ 集合至多一个元素）。每条 `deleted_inodes` 条目都判、在配对之前：「if (queue_read.readable && !d_deletion_queue_holds_at_most_one_key(queue_read)) {」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2362`）。
- I-9.18：CPU 要求队列里每个号 N 有（对象出生代 = N 的记录的、整对象、死亡 txg 在谱系上）的墓碑，N 没有记录判不了：「judgements.judge(invariant, has_a_whole_object_tombstone_on_the_lineage, || {」（`crates/singlefs-checker/src/walk.rs:9716`）；GPU 同条件下置位 「violate(INVARIANT_I_9_18);」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2984`），N 没有记录 `continue`、读不出的一概 `return`。
- I-9.19：CPU 在谱系上的墓碑、同号同出生代的记录 nlink = 0：「record.nlink == 0,」（`crates/singlefs-checker/src/walk.rs:9770`）；GPU 置位 「violate(INVARIANT_I_9_19);」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2913`），前提 `lineage_holds` 与「在谱系上」：「if (!lineage_holds || !d_txg_is_on_the_lineage(rd64(base, o + DELETION_TOMBSTONE_RECORD_DEATH_TXG_OFFSET), version_txg)) {」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2909`）。
- I-9.10 墓碑那一个等号（按号认、不看谱系）：CPU 「judgements.judge("I-9.10", record.object_birth == tombstone.object_birth, || {」（`crates/singlefs-checker/src/walk.rs:9740`）；GPU 「violate(INVARIANT_I_9_10);」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2907`），在「谱系」那一句 `continue` 之前，故不看谱系。
- I-9.20 管全部墓碑的几句：CPU 「tombstone.object_birth <= tombstone.death_txg && tombstone.death_txg <= version_txg;」（`crates/singlefs-checker/src/walk.rs:9809`）；GPU 「if (!le64(rd64(base, o + DELETION_TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET), death_txg) || !le64(death_txg, version_txg)) {」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2847`）。整对象：CPU 「self.range_start < self.range_end」（`crates/singlefs-checker/src/walk.rs:9072`），GPU 「return lt64(range_start, range_end) && is_zero64(range_start) && eq64(range_end, vec2<u32>(NONE, NONE));」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2811`）（`NONE` 是全 1）。
- I-9.20 不重号（只判在谱系上的，谱系接成链才判）：CPU 用集合 「objects_on_the_lineage.insert((tombstone.object, tombstone.object_birth));」（`crates/singlefs-checker/src/walk.rs:9835`）；GPU 对叶序里每条在谱系上的墓碑，查它之前在谱系上的同 (号, 出生代) 的墓碑，置位 「violate(INVARIANT_I_9_20);」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2886`）。集合 `insert` 返回假 ⇔ 此前有同键的在谱系上的墓碑，等价。
- I-9.21（没装旋钮那一支）：CPU 容器至少两片时沿叶序容器号严格递增、非最右每片记录数 == 815：「judgements.judge(invariant, record_count == TOMBSTONE_CONTAINER_RECORDS, || {」（`crates/singlefs-checker/src/walk.rs:9884`）；GPU 「if (d_container_record_count(container_index) != DELETION_TOMBSTONE_CONTAINER_RECORDS) {」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2826`），容器号取容器头（`d_container_number`），不沿根条目读。
- 谱系链与「在谱系上」：CPU 按 T_rb 从大到小过一遍、T_rb 大于此刻的跳过、最后 [0, 此刻] 也在链上：「if record.rollback_txg > txg_walked_to {」（`crates/singlefs-checker/src/walk.rs:9143`）；GPU 同序，「if (lt64(txg_walked_to, rollback_txg)) {」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2770`），最后 `return le64(txg, txg_walked_to)`。「T_old ≥ T_rb、同一 T_rb 两条」接不成链：CPU `lineage_chain_of_a_version` 9123–9138 行，GPU `d_read_the_lineage_of_a_version` 末两个判（`crash_judge_checker_scan.wgsl` 2753–2761 行）。
- I-9.29：CPU 各版 inode 记录与谱系上的墓碑收进 (树 ID, 号) → 出生代集合，收齐后每个号 `births.len() == 1`：「judgements.judge(ONE_OBJECT_BIRTH_PER_INODE_NUMBER, births.len() == 1, || {」（`crates/singlefs-checker/src/walk.rs:9911`）；GPU 边收边比，同键第二个不同的出生代当场置位：「violate(INVARIANT_I_9_29);」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2788`）（第三个与第一个比，集合大小 > 1 ⇔ 出现过与首个不同的值，等价）。墓碑那一侧的键用墓碑树的头 ID，CPU 同（`walk.rs` 9576–9591 行）。

一处顺序上的小差别，不改结果：GPU 的 I-9.18 先读队列、再读 inode 记录、再数墓碑树条目，CPU 把每棵 inode 树的记录都先读好（`walk.rs` 9448–9459 行）；两边读不出一律不置位。

结论：逐句同判同位；GPU 比 CPU 多的只有表容量（1.4）。推翻它的现象：对拍里出现一个状态在 I-9.18–I-9.21、I-9.24、I-9.29 位上 GPU 与 CPU 不同，且不是 1.4 那六处判不了。

### 1.3 I-9.22 恒不置位

依据是 GPU 手里没有旋钮值、层 0 与放量只走旧入口 `check_pool_image`（旧入口当没装旋钮，I-9.22 恒报不适用）。代码里：「I-9.22 恒不置位：它只在池级 checker 带旋钮值的入口上判」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:163`）；kb：「GPU 判器第 23 批补，GPU 恒不置位、层 0 恒不适用」（`.claude/kb/invariants.md:309`）。CPU 探针里 I-9.22 在 T9（旋钮压到一片 1 条）和 T13 上都是 `NotApplicable`，与「GPU 不置位」一致（第 4 节原样输出）。
覆盖表 `crates/singlefs-checker-tier/src/crash_judge_gpu.rs:83` 仍写满 0..=82（下标 79 在表里、位恒 0），`crash_judge_gpu.rs` 里有一道编译期断言管「表长 = `IMPLEMENTED_INVARIANTS` 长、第 k 项是 k」；用户 2026-10-04 定「改回要求全覆盖」、没有「内核还没判」那一张表，写在 「用户 2026-10-04 定」（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:78`） 所在的那一行。I-9.22 与 I-4.4 是「登记了位、内核恒不置」的两条，kb 309 行写明了前者。
推翻它的现象：某条对拍流用 `check_pool_image_given_the_tombstone_container_capacity_knob` 带旋钮值判 CPU（那时 CPU 会红 I-9.22 而 GPU 不置位）。今天的对拍只走旧入口（`compare_on_every_state` 取 `check_pool_image`），所以不触发。

### 1.4 超上限记判不了（站点号 55–60）

五类容量，超了都先写 `undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (站点 << 8)`、读函数交回 `false`（调用方当「读不出」不判），不判成立：

- 站点 55、56：读一棵墓碑 / 谱系树时待走条目栈满（`C_R_MAX_PENDING` = 512）：「if (!d_push_container_tree_entries_in_reverse(root, &pending, 55u)) {」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2638`）、「if (!d_push_container_tree_entries_in_reverse(child, &pending, 56u)) {」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2657`）。
- 站点 57：一棵树的容器多于 256 片：「undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (57u << 8u);」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2669`）。
- 站点 58：一版谱系记录多于 256 条：「undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (58u << 8u);」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2721`）。
- 站点 59：跨版本表 (树, 号) 多于 2048 个：「undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (59u << 8u);」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2794`）。
- 站点 60：两两比不重号的墓碑总数多于 4096：「undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (60u << 8u);」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2862`），常量 「const DELETION_MAX_TOMBSTONES_COMPARED_PAIRWISE: u32 = 4096u;」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2600`）。

和已有站点不撞（`grep -n "<< 8u" ` 与 `grep -n "pending, 5" ` 现数：走读内核 40–46；扫描内核 47、48、49、52、53、54 写成 `<< 8u` 字面，50、51 是 `d_push_container_tree_entries_in_reverse` 的调用参数；新增 55、56 是同一函数的调用参数，57–60 是 `<< 8u` 字面）。
每个站点的上限都低于合法数据能到的量（一片谱系容器装 2039 条，上限 256 条；对拍 `expand_up_to` 一次不允许判不了：`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 第 5 行「一个状态都不许判不了」），所以放量路径遇到会整趟停下报站点，不会静默放过。
推翻它的现象：把某个表的上限压到合法流装得下之下，对拍里出现「判不了」的状态；或在合法流上这六处之一被触发。

### 1.5 下标与草稿区算术

位 ↔ 不变量按 `IMPLEMENTED_INVARIANTS` 数组次序（83 项）。我用一个一次性脚本把 `image.rs` 里的 `pub const … = "I-x.y"` 常量按数组次序展开，数出下标（输出原样）：

```text
len 83
40 I-9.4
43 I-9.10
74 I-9.17
75 I-9.18
76 I-9.19
77 I-9.20
78 I-9.21
79 I-9.22
80 I-9.23
81 I-9.24
82 I-9.29
```

GPU 内核里的常量：「const INVARIANT_I_9_4: u32 = 40u;」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:89`）、「const INVARIANT_I_9_10: u32 = 43u;」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:95`）、「const INVARIANT_I_9_17: u32 = 74u;」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:165`）、「const INVARIANT_I_9_29: u32 = 82u;」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:181`），75–81 中间依次连号（`crash_judge_checker_common.wgsl` 165–181 行），与上表一一对上。

五张表依次排在 `D_INODE_RECORDS` 之后，起点逐段相加得到 `SCRATCH_END`，再得每个状态的字节数与派活条数（原样）：

```text
D_INODE_RECORDS = 290544
+7*1024 -> D_DATA_UNIT_COUNTS = 297712
+3*1024 -> D_CONTAINERS_OF_A_CONTAINER_TREE = 300784
+5*256 -> D_LINEAGE_RECORDS = 302064
+4*256 -> D_OBJECT_BIRTHS = 303088
+6*2048 -> SCRATCH_END = 315376
bytes_per_state = 1261504
2_147_483_644 // bytes_per_state = 1702
12*1024**3 // bytes_per_state = 10213
```

代码里写的：「const D_INODE_RECORDS: u32 = 290544u;」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:789`）、「const D_INODE_RECORD_WORDS: u32 = 7u;」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:785`）、「const D_DATA_UNIT_COUNTS: u32 = 297712u;」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:796`）、「const D_CONTAINERS_OF_A_CONTAINER_TREE: u32 = 300784u;」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:804`）、「const D_LINEAGE_RECORDS: u32 = 302064u;」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:812`）、「const D_OBJECT_BIRTHS: u32 = 303088u;」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:820`）、「const SCRATCH_END: u32 = 315376u;」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:823`）；Rust 一侧 「pub const SCRATCH_WORDS: usize = 315_376;」（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:129`），派活单测 「assert_eq!(states_per_scratch_lane(BINDING_LIMIT), 1702);」（`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:882`）、「assert_eq!(states_per_dispatch(BINDING_LIMIT, 12 * 1024), 10_213);」（`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:889`），上限 「const BINDING_LIMIT: u64 = 2_147_483_644;」（`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:876`）。算出来与写的逐个相等。
`SCRATCH_WORDS` 与 `SCRATCH_END` 相等由 `crash_judge_gpu.rs` 里的单测 `scratch_words_given_to_each_state_reach_the_scratch_end_of_the_kernels` 核，这一条是 checker 档包 `singlefs-checker-tier` 的 `--lib` 测试，按规矩我不跑；我用 `grep` 直接对：两处都是 315376（见上两条引文）。

这一节推翻它的现象：`SCRATCH_END` 与 `SCRATCH_WORDS` 不等（那条单测红），或某张表的起点与上一张表的终点有重叠。

### 1.6 N1 小结

一致：八条置位、I-9.22 恒不置位、六处超上限判不了、草稿区算术。推翻它的现象见 1.1–1.5 各节末句。注意（第 9 节）：GPU 不跑，所以 GPU 一侧对拍是否真绿、那 18 行 GPU 变异是否真红、驱动编译这个扫描内核要多久，都是提交时才有的数。

## 2 K1 · N2：I-9.4 第四句并进下标 40、I-9.10 墓碑那一个等号并进下标 43

- 条款：「同一片打包记录类型 2 容器里记录号沿记录序严格递增」（`.claude/kb/invariants.md:291`）；「墓碑那一个等号 2026-10-05 删除线第 22 批实现，GPU 判器第 23 批补」（`.claude/kb/invariants.md:297`）。
- I-9.4 第四句：CPU 在 `walk_inode_leaf_container` 里逐条记录判 「self.judgements.judge("I-9.4", inode > previous_inode, || {」（`crates/singlefs-checker/src/walk.rs:2503`），上一条记录号随循环带；GPU 同位置、同条件（`r > 0u`）：「w_judge(INVARIANT_I_9_4, lt64(rd64(p.base, packed_record_offset(p, r - 1u)), inode));」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2641`），在 `w_visit` 之后、`w_judge_packed_container` 判过 `packed.ok` 之后，与 CPU「第一次走到这片叶 + 容器合法才判记录」同。第二、三句（容器号沿叶序、最小最大 key）原本就用 min / max 取每片叶的范围（`crash_judge_checker.wgsl` 2679–2686 行），记录乱序不影响它们。
- 文档：「I-9.4 第四句（同一片 inode 叶容器里记录号严格递增）走读内核一并判、并进下标 40」（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:77`）。
- I-9.10 墓碑那一个等号：见 1.2；GPU 置位的是 「violate(INVARIANT_I_9_10);」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2907`），常量是下标 43。CPU 在读不全、配不上时不记判不了，GPU 同（`return` / `continue` 不置位）。
- CPU 探针 T10（1、2 号两条记录对调后的镜像）：`violated=["I-9.4"]`，只红 I-9.4（第 4 节原样输出）。
结论：一致。推翻它的现象：合法的单叶或多叶镜像上 GPU 的 I-9.4 位比 CPU 多红（比如把「前一条记录号」跨叶带），或乱序叶上 GPU 少红。

## 3 K1 · N3：零条目根只在有条目时核条目宽；size 偏移合成一处定义

条款与判决：m3-batch20-code-r1 判决第三节第 2 条：「2. N5：GPU 恢复内核放过零条目 extent 上段根（与 `recovery.rs` 删完那一支同判）；对拍补一条删到最后一批的流；配变异。」（`research/prompts/m3-batch20-code-r1-main-verification.md:36`）。

- 实现的「应该是什么」：core 的 extent 上段读法只在逐条 `ExtentUpperLeafEntry::parse` 时核条目宽——「ExtentUpperLeafEntry::parse(entry_bytes).map_err(|malformed| match malformed {」（`crates/singlefs-core/src/extent_tree.rs:933`）；零条目只许上段根兼叶——「if header.entries.is_empty() && !(is_the_root_of_the_tree && expected_level == 0) {」（`crates/singlefs-core/src/extent_tree.rs:711`）。inode 树根同：「// 零条目的 inode 树根是 unlink 删掉树里最后一条记录之后的那一版」（`crates/singlefs-core/src/recovery.rs:2637`），宽度只在逐条解条目时核（`recovery.rs` 2643 行 `inode_internal_entry_parsed_and_judged(entry)?;`）。
- GPU 改后：「if (n.entry_count > 0u && n.entry_width < EXTENT_UPPER_LEAF_ENTRY_BYTES) {」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:2379`）；「if (inode_root.entry_count > 0u && inode_root.entry_width < CONTAINER_TREE_ENTRY_BYTES) {」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3230`）。上段 `judge_extent_node` 里零条目只许根兼叶那一判原本就有：「if (n.entry_count == 0u && !(is_the_root_of_the_tree && expected_level == 0u)) {」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:2333`）。有条目而窄的根两边仍都拒（GPU 的 `entry_count > 0u` 为真时走原判）。
- 常量：「const EXTENT_UPPER_LEAF_ENTRY_BYTES: u32 = 113u;」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:110`），与 `extent_tree.rs` 的 `EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES`（113）同值（注释写明）；`CONTAINER_TREE_ENTRY_BYTES` 120 已有。
- size 偏移合一：共用那一半一处 「const INODE_RECORD_SIZE_OFFSET: u32 = 40u;」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:141`）；使用处 「let size = rd64(p.base, ro + INODE_RECORD_SIZE_OFFSET);」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2645`）、「rd64(container.base, record + INODE_RECORD_SIZE_OFFSET), rd64(container.base, record + INODE_RECORD_OBJECT_BIRTH_OFFSET))) {」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2222`）、「inode_size = rd64(leaf.base, ro + INODE_RECORD_SIZE_OFFSET);」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3287`）。旧名 `DELETION_INODE_RECORD_SIZE_OFFSET` 全仓现数 0 处（`grep -rn DELETION_INODE_RECORD_SIZE_OFFSET crates | wc -l` 原样输出 `0`）。
- 对象出生代偏移同样进了共用那一半：「const INODE_RECORD_OBJECT_BIRTH_OFFSET: u32 = 8u;」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:139`）；但走读内核还有两处写死的 `ro + 8u`：「w_judge(INVARIANT_I_9_26, le64(rd64(p.base, ro + 8u), change_count)」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2651`）、「let birth = rd64(p.base, ro + 8u);」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2654`）（以及 `blocks` 的 `48u`、改动计数的 `88u`）——不改结果，是「一处定义」没做到底的小出入，见第 9 节。
- CPU 探针 T11 / T12：删到最后一批之后，把零条目的 extent 上段根（条目宽 113 → 24）/ inode 树根（120 → 8）改窄、重封，core 冷走读答「没有文件」，池级 checker 红 I-1.10（原样）：

```text
ZS_PROBE LB_clean recovery=NoFile
ZS_PROBE LB_clean violated=[]
ZS_PROBE T11_extent_root_width_24 recovery=NoFile
ZS_PROBE T11_extent_root_width_24 violated=["I-1.10"]
ZS_PROBE T12_inode_root_width_8 recovery=NoFile
ZS_PROBE T12_inode_root_width_8 violated=["I-1.10"]
```

这一步同时回答了「窄根时恢复答什么」：不是走读失败，是 `NoFile`，所以 GPU 不加 `entry_count > 0u` 就会与 CPU 分叉（`crates/mutations.tsv` 2828、2829 行变异名里的说法）。
结论：一致。推翻它的现象：对拍里 T11 / T12 两条流上 GPU 的恢复结局不是「没有文件」；或某个有条目而宽度窄的根上 GPU 放行。

## 4 K1 · N4：对拍新加的 13 条，每条覆盖的是不是它名字说的那一判

方法：GPU 不跑；对每条用例的「改坏动作」，在 harness 档探针里对同一动作造镜像、跑 CPU 池级 checker，看红集里有没有名字说的那一条、有没有别的（别的红了，对拍里 GPU 也得同红，不是缺陷）。探针造的是「第一批那一版」「新池 + 空 inode」「删到最后一批」「回退之后」几份最终镜像，与 GPU 用例的流在最后一个根槽写之后是同一动作同一镜像形；对拍还会在每个中间崩溃状态上比，那一半我没有（第 9 节）。

| # | 用例（`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 行） | 改坏动作 | 探针标签 | CPU 红集（原样） | 名字说的那一判 |
|---|---|---|---|---|---|
| 1 | 5964 `…lineage_root_entry_names_another_container_than_its_child` | 谱系树根第一条条目身份引用的容器号加 1 | T1_container_plus_one | `["I-9.17"]` | I-9.17 ✓ |
| 2 | 5976 `…tombstone_root_entry_carries_an_unregistered_type` | 墓碑树根第一条条目类型段 1 → 7 | T2_entry_type_seven | `["I-9.23"]` | I-9.23 ✓ |
| 3 | 5987 `…type_zero_carries_an_identity` | 同一条类型段 1 → 0（出生树非 0） | T3_entry_type_zero | `["I-4.8", "I-7.2", "I-7.4", "I-9.23"]` | I-9.23 ✓（后三条是层级 1 里出现类型段 0 记的走读失败） |
| 4 | 5999 `…queued_inode_has_no_tombstone_on_the_lineage` | 墓碑对象 ID 1 → 0 | T4_object_zero | `["I-9.18"]` | I-9.18 ✓ |
| 5 | 6010 `…tombstone_on_the_lineage_names_a_live_object` | 队列里 1 号记录 nlink 0 → 1 | T5_nlink_one | `["I-9.5", "I-9.19"]` | I-9.19 ✓（I-9.5 同红，用例只断言 I-9.19） |
| 6 | 6019 `…tombstone_dies_before_its_object_is_born` | 墓碑死亡 txg → 0 | T6_death_txg_zero | `["I-9.20"]` | I-9.20 管全部墓碑那几句 ✓ |
| 7 | 6030 `…tombstone_on_the_lineage_is_written_twice` | 同一条墓碑再追加一份 | T7_written_twice | `["I-9.20"]` | I-9.20 不重号那一句 ✓（出生代 ≤ 死亡 txg 仍成立，只有不重号能红） |
| 8 | 6039 `…tombstone_names_another_object_birth_than_the_record` | 墓碑对象出生代加 1 | T8_object_birth_plus_one | `["I-9.10", "I-9.18", "I-9.29"]` | I-9.29 + I-9.10 ✓（一条用例断言两条；I-9.18 同红） |
| 9 | 6050 `…non_rightmost_tombstone_container_is_not_full` | 旋钮压到一片 1 条，unlink 两个空 inode | T9 | `["I-9.21"]` | I-9.21（旧入口当没装旋钮）✓ |
| 10 | 6061 `…records_of_an_inode_leaf_are_out_of_order` | 同一片叶里 1、2 号两条记录对调 | T10_swapped | `["I-9.4"]` | I-9.4 第四句 ✓（第一、二、三句不红，只有第四句能红） |
| 11 | 6073 `…empty_extent_root_narrower_than_its_field_table_holds_no_file` | 删到最后一批后零条目 extent 上段根条目宽 113 → 24 | T11_extent_root_width_24 | `["I-1.10"]`，core 恢复 `NoFile` | I-1.10 + 恢复答没有文件 ✓ |
| 12 | 6086 `…empty_inode_root_narrower_than_its_field_table_holds_no_file` | 同上，inode 树根 120 → 8 | T12_inode_root_width_8 | `["I-1.10"]`，core 恢复 `NoFile` | 同上 ✓ |
| 13 | 6100 `…on_every_state_of_an_unlink_rolled_back_and_unlinked_again` | 不改字节：删 → 回退 → 再删 | T13 | `[]`；I-9.17–I-9.20、I-9.23、I-9.24、I-9.29 `Holds`，I-9.21、I-9.22 `NotApplicable` | 合法流九条一条不红 ✓ |

原样输出里相关的几行（探针 `ZS_PROBE …`，`rerun.sh` 的 `summary.txt` 抽出）：

```text
ZS_PROBE N6_after_mkfs violated=[]
ZS_PROBE N6_full_persisted violated=[]
ZS_PROBE T13 violated=[]
ZS_PROBE T9 violated=["I-9.21"]
ZS_PROBE T10_clean violated=[]
ZS_PROBE T10_swapped violated=["I-9.4"]
ZS_PROBE FB_clean violated=[]
ZS_PROBE T2_entry_type_seven violated=["I-9.23"]
ZS_PROBE T3_entry_type_zero violated=["I-4.8", "I-7.2", "I-7.4", "I-9.23"]
ZS_PROBE T4_object_zero violated=["I-9.18"]
ZS_PROBE T5_nlink_one violated=["I-9.5", "I-9.19"]
ZS_PROBE T6_death_txg_zero violated=["I-9.20"]
ZS_PROBE T7_written_twice violated=["I-9.20"]
ZS_PROBE T8_object_birth_plus_one violated=["I-9.10", "I-9.18", "I-9.29"]
ZS_PROBE T3b_type_zero_zero_identity_child_is_a_container violated=["I-4.8", "I-7.2", "I-7.4"]
ZS_PROBE T1_clean violated=[]
ZS_PROBE T1_container_plus_one violated=["I-9.17"]
ZS_PROBE LB_clean recovery=NoFile
ZS_PROBE LB_clean violated=[]
ZS_PROBE T11_extent_root_width_24 recovery=NoFile
ZS_PROBE T11_extent_root_width_24 violated=["I-1.10"]
ZS_PROBE T12_inode_root_width_8 recovery=NoFile
ZS_PROBE T12_inode_root_width_8 violated=["I-1.10"]
ZS_PROBE Q17 violated=["I-9.5", "I-9.24"]
ZS_PROBE N5_clean violated=[]
ZS_PROBE N5_instance_table_broken violated=["I-2.1", "I-4.8", "I-7.2", "I-7.4"]
```

T13 与 T9 的九条逐条判定（原样；T9 里 I-9.21 是 `Violated`、其余八条没有红）：

```text
ZS_PROBE T13 I-9.17 Holds
ZS_PROBE T13 I-9.18 Holds
ZS_PROBE T13 I-9.19 Holds
ZS_PROBE T13 I-9.20 Holds
ZS_PROBE T13 I-9.21 NotApplicable
ZS_PROBE T13 I-9.22 NotApplicable
ZS_PROBE T13 I-9.23 Holds
ZS_PROBE T13 I-9.24 Holds
ZS_PROBE T13 I-9.29 Holds
ZS_PROBE T9 I-9.17 NotApplicable
ZS_PROBE T9 I-9.18 Holds
ZS_PROBE T9 I-9.19 Holds
ZS_PROBE T9 I-9.20 Holds
ZS_PROBE T9 I-9.21 Violated
ZS_PROBE T9 I-9.22 NotApplicable
ZS_PROBE T9 I-9.23 Holds
ZS_PROBE T9 I-9.24 Holds
ZS_PROBE T9 I-9.29 Holds
```

队列 {1, 7} 那条（`…when_the_deletion_queue_names_an_inode_without_a_record`，用例位置 `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:5482`，多断言 I-9.24）：改动是条目数 +1、声明长度 +8、头里最大 key 改 7；探针 `Q17` 红集 `["I-9.5", "I-9.24"]`，I-9.29 `Holds`，与「7 号没有记录：I-9.5 正方向红，队列里两个不同的号：I-9.24 红」一致，I-9.18 对「没有记录」那一形判不了、不红。
断言形式：「fn assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(」（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:1773`） 要 CPU 红集含名字说的那条、且 GPU 与 CPU 在每个对拍的状态上整份逐位相同（`mismatches.is_empty()`）；多条的 「fn assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(」（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:5648`） 逐条要含。新加的 13 条里 T8 用后者，T13 单列，其余用前者；改过的旧用例队列 {1, 7} 那条也用后者。合法流 T13 的断言单列（「for invariant in ["I-9.18", "I-9.19", "I-9.20", "I-9.24", "I-9.29"] {」（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:6117`） 要成立过，另有九条不红的断言）。
文件头也写明了「不屏蔽」：「每一条不变量的违例位（`IMPLEMENTED_INVARIANTS` 全部，不屏蔽）要与 CPU 池级 checker 逐位相同」（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:5`）。

结论：13 条各自覆盖名字里的那一判；没有一条的改坏动作只红别的条。一处名实差：用例 3（类型段 0）覆盖的是「身份引用三段为 0」那一半，条款的另一半「子单元是码 2」这条路在 CPU 与 GPU 上都测不到，见第 9 节 L1。正文把 I-9.29 与 I-9.10 各数一条（`research/prompts/_m3-batch23-code-r1-body.md` 16 行），实际是同一条用例断言两条，`#[test]` 现数新加 13 个（`awk` 在 diff 里数 `^+#[test]` = 13），不是 14。
推翻它的现象：GPU 能跑时，某条用例 CPU 红集不含名字里那一条，或两侧在某个中间状态上逐位不同。

## 5 K1 · N5：CPU 的 I-9.29 在最新根实例表读不出时报部分没判、一个号都不比

条款：「最新根的实例表读不出、被崩溃恢复抛弃的根滤不掉时记判不了」（`.claude/kb/invariants.md:316`）。
实现：

- 新枚举 「enum AbandonedRootsFilteringOfTheCandidateSet {」（`crates/singlefs-checker/src/walk.rs:9033`）；判断在候选集那一段之前：「let abandoned_roots_filtering = match instance_table_rows_of_root_without_judging(」（`crates/singlefs-checker/src/walk.rs:10254`）（`Some` ⇒ 滤过了，`None` ⇒ 没滤过）；传进墓碑 / 谱系那一遍：「abandoned_roots_filtering,」（`crates/singlefs-checker/src/walk.rs:10589`）；那一遍收尾按枚举分支，没滤过的一支 「judgements.partially_judged(」（`crates/singlefs-checker/src/walk.rs:9366`） 一个号都不比（`walk.rs` 9361–9374 行）。
- 候选集自己的滤法仍用主走读沿链判下来的行：「let abandoned_by_the_newest_roots_table = |root: &crate::RootView| {」（`crates/singlefs-checker/src/walk.rs:10247`）；注释写明这两个读法的关系：「// 上面那张表是主走读沿链判下来的行，链断在哪一片就只有那一片之前的行：读不全时滤不掉的根认不出来。」（`crates/singlefs-checker/src/walk.rs:10252`）。

「读不出」的判法与候选集滤法对不对得上——我读两边的读法比出的结论：

- `instance_table_rows_of_root_without_judging`（`crates/singlefs-checker/src/walk.rs:5467`）逐片要求 `check_unit` 通过且是码 3、身份四元组 = (0, 4, 片序号, 0)、记录宽与声明长度合法、非末条的记录首字节是 0、末条是合法链指针记录。
- 主走读的 `walk_instance_table_chain`（`walk.rs` 2701–2802 行）沿同一条链读、把各片的行推进 `self.instance_table_rows`（`crates/singlefs-checker/src/walk.rs:2822`）。它每一片的门比前者松或相等：`judge_unit_header` 在载荷 CRC 不对时仍交回 `true`（「if !payload_crc_holds {」（`crates/singlefs-checker/src/walk.rs:1239`）），而 `check_unit` 此时拒（`crates/singlefs-checker/src/lib.rs:419`）；链指针记录畸形时主走读先把这一片推进 `pages` 再停，前者交回 `None`。
- 最新根是 `walk_root(…, true)` 第一个走的东西（`walk.rs` 1381 行起，实例表链在树表之前），`visited_units` 此时是空的，所以主走读那边不会因「走过了」而丢一片。
- 所以：前者给出 `Some` ⇒ 主走读也读全了同样的行，滤法没缺行（不会把滤不全的情形误当成滤全了）；前者给出 `None` 而主走读其实读全了——只出现在上面两种「主走读更松」的镜像上，结果是 I-9.29 多报一次部分没判，不会误红。这是我读出来的，没有造镜像逐形验。

原样复跑（同一次 `rerun.sh`，`out/tests.txt`、`out/mut-n5.txt`）：

```text
exit=0 checker_known_bad_images
test an_abandoned_root_naming_another_object_… ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 102 filtered out
MUT_ROWS 2 prefix=第23批 X I-9.29
MUT 第23批 X I-9.29：最新根的实例表读不出时当它滤过了被抛弃的根，照旧跨版 exit=101 test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 102 filtered out
MUT 第23批 X I-9.29：候选集滤不掉被抛弃的根时照样跨版本比、再记部分没判（ exit=101 test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 102 filtered out
```

测试 ① 实例表读得出时 I-9.29 `Holds`（不被误报判不了），② 两份都翻坏后报部分没判且说明以「最新根的实例表读不出」打头；两条 X 变异（读不出当成滤过、违例压过部分没判）都被它抓红。
另有一张我自己造的镜像（`N5_instance_table_broken`：干净的新池新建文件镜像上把最新根的实例表单元两份各翻一字节，池里没有任何被抛弃的根，原样）：

```text
ZS_PROBE N5_clean I-9.29 Holds
ZS_PROBE N5_clean violated=[]
ZS_PROBE N5_instance_table_broken I-9.29 PartiallyJudged
ZS_PROBE N5_instance_table_broken I-9.29 text=最新根的实例表读不出（链上有一片读不出或解不开）：回退候选集里滤不掉被崩溃恢复抛弃的根，收到的 1 个号一个都没比
ZS_PROBE N5_instance_table_broken violated=["I-2.1", "I-4.8", "I-7.2", "I-7.4"]
```

结论：判不了的条件与候选集的滤法对得上（只有「多报」一个方向的差）；与 kb 字面有一处读法差——条款是「读不出、被抛弃的根滤不掉时」，实现是「读不出」就报，不看环里有没有被抛弃的根（上面这张镜像：环里一条被抛弃的根都没有，仍报部分没判）。「读不出就等于滤不掉」是测试注释指到的「第 22 批 C 实现员报告『交主 agent 的设计判断』第 5 条」的读法（我没有读那份报告），条款没有把这一步写出来，交主 agent 定；方向是保守的（不红）。
GPU 侧还没有这一支（第 24 批）：见第 9 节 L2，那里有 CPU 算出的 GPU 期望。
推翻它的现象：一份镜像上最新根的实例表读得出（`Some`）而候选集里仍混着没被滤掉的被抛弃的根，I-9.29 因此误红；或实例表读不出时 I-9.29 红。

## 6 K1 · N6：`layer0_quick_tier_matches_the_full_tally_shape` 的分组

分组与计数（我用脚本从测试文件里数各个数组的项数，原样）：

```text
only_under_the_new_root 26
under_the_new_root_or_through_the_record_the_next_mount_applies_first 1
never_comparable_on_this_stream 6
never_has_an_inode_being_deleted_on_this_stream 3
never_has_a_tombstone_or_lineage_record_on_this_stream 6
not_applicable_in_the_first_version 1
judged_once_a_record_is_on_disk 2
judged_once_a_record_and_the_root_of_its_publish_are_on_disk 1
judged_once_a_record_of_the_file_publish_is_on_disk 1
judged_once_a_device_holds_two_records_of_one_instance 1
judged_only_when_the_replay_applies_a_record 1
judged_unless_every_device_has_one_choosable_slot_written_by_mkfs 1
not_every_state_total 50 every_state 33
```

和函数文档里写的数一致：「/// 那 26 条只在这些状态上评估」（`crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs:98`）、「/// 其余 33 条每个状态都评估」（`crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs:121`）、「/// 删除与回退那十条（I-4.7、I-4.6、I-4.4 与墓碑、谱系那七条 I-9.17–I-9.23）在这条流上一个状态都评估不到」（`crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs:120`）（3 + 6 + 1 = 10 条）；I-4.4 单列：「let not_applicable_in_the_first_version = [INTENT_DESCRIBES_ONLY_THE_TARGET_STATE];」（`crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs:210`）。

依据（正推）：这条流上只有 txg 3 那一版的树表有条目（别的版本的树表是空的），所以——

- I-9.5、I-3.4：要 txg 3 的根已落才评估（树表条目、`deleted_inodes`、记账树）；I-9.24：队列空也报成立，要 `deleted_inodes` 条目在；I-9.29：要 inode 记录收出号（第一个文件的 1 号）；四条进「根已落」组。记录已落而根没落的状态，由记录施加出来的那一版不进版本清单（要实例表里有一行记着恢复施加到过 txg 3，这条流上唯一一次挂载在 txg 3 之前）：「record.checkpoint_txg <= row.published_checkpoint_txg」（`crates/singlefs-checker/src/walk.rs:7484`）（`versions_applied_only_by_records`）。
- I-4.7、I-4.6：没有 nlink = 0 的记录；I-9.18：队列空就不判；I-9.17、I-9.19–I-9.23：墓碑树、谱系树都是零根（树表条目的根指针全零，`read_index_node` 对全零指针返回 `None`，不进走读），删除态扫描读出的墓碑容器是空的，I-9.21 要至少两片，I-9.22 在旧入口恒不适用；I-4.4 第一版恒不适用。

现算（CPU 探针 `N6_full_persisted` = 新池新建文件写完、全部持久那个状态的干净镜像；`N6_after_mkfs` = mkfs 刚写完；原样）：

```text
ZS_PROBE N6_full_persisted I-3.4 Holds
ZS_PROBE N6_full_persisted I-4.4 NotApplicable
ZS_PROBE N6_full_persisted I-4.6 NotApplicable
ZS_PROBE N6_full_persisted I-4.7 NotApplicable
ZS_PROBE N6_full_persisted I-9.10 Holds
ZS_PROBE N6_full_persisted I-9.17 NotApplicable
ZS_PROBE N6_full_persisted I-9.18 NotApplicable
ZS_PROBE N6_full_persisted I-9.19 NotApplicable
ZS_PROBE N6_full_persisted I-9.20 NotApplicable
ZS_PROBE N6_full_persisted I-9.21 NotApplicable
ZS_PROBE N6_full_persisted I-9.22 NotApplicable
ZS_PROBE N6_full_persisted I-9.23 NotApplicable
ZS_PROBE N6_full_persisted I-9.24 Holds
ZS_PROBE N6_full_persisted I-9.29 Holds
ZS_PROBE N6_full_persisted I-9.4 Holds
ZS_PROBE N6_full_persisted I-9.5 Holds
ZS_PROBE N6_full_persisted violated=[]
```

`N6_after_mkfs`：被盯的 16 条里 `NotApplicable` 现数 16 条（另有 `violated=[]` 一行）。

这是单个状态的判定，不是计数：I-9.5、I-3.4、I-9.24、I-9.29 在根已落那个状态 `Holds`（被评估过），mkfs 刚写完全是 `NotApplicable`，与「只在根已落的 9 个状态上评估」同向；I-4.7、I-4.6、I-9.18、I-9.17、I-9.19–I-9.23、I-4.4 在根已落的状态上 `NotApplicable`（评估数 0）。9 个状态各自的计数、记录已落根没落那几个状态上是不是 0，要跑层 0（checker 档，按规矩我不跑），正文也写明「推的，提交时核」。「const ROOT_PERSISTED_STATES: u64 = 9;」（`crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs:536`）。
结论：分组的依据站得住；计数部分只核到单个状态。推翻它的现象：层 0 快档日志里 `I-9.24` / `I-9.29` 的评估数不是 9，或 `I-9.18`–`I-9.23` 的评估数不是 0。

## 7 K1 · N7：E142 导出建盘前试分配

读：新函数 「fn free_maps_of_the_two_devices_fit_in_memory(」（`crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs:148`），每槽至多 4 字节的常量 「const FREE_MAP_BYTES_PER_UNIT_AREA_SLOT_AT_MOST: u64 = 4;」（`crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs:140`）。数一遍真实的空闲图（`allocator.rs` 的 `DeviceFreeMap::with_unit_area_start`）：三张 `Vec<bool>`（已分配、隔离、扣住）每槽 3 字节，四张 `Vec<u64>` 每 64 槽一个（段内已分配数、隔离数、扣住数、两槽都没挡的槽对数）每槽 0.5 字节，共 3.5 ≤ 4，常量不低估（如 「allocated: vec![false; usize::try_from(unit_area_slots).expect("单元区槽数")],」（`crates/singlefs-core/src/allocator.rs:387`））。
调用位置在建内存盘之前：「if let Err(refusal) = free_maps_of_the_two_devices_fit_in_memory(」（`crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs:208`），建盘在后面的 「let mut devices = two_recording_memory_devices(device_bytes, &stream);」（`crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs:216`）；分配不出打原因、退 「std::process::exit(EXIT_CODE_OF_A_REFUSED_COMMAND_LINE);」（`crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs:213`）。文件头退出码一段已把这一种写进去：「//! 两块盘内存里的单元区空闲图在这台机器上分配不出（例：`--device-bytes 18446744073709551615`，建盘之前判），」（`crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs:20`）。
原样复跑（`rerun.sh` 的 `bin` 步；编译 6G、跑 4G；`device_bytes` 对应 2⁶⁴−1、0、2⁵⁰、2⁴⁰）：

```text
build exit=0
BIN device_bytes=18446744073709551615 exit=1 stdout_lines=0 stderr_first=e142_new_pool_file_creation_write_dump 每盘 18446744073709551615 字节：单元区 1125899906792447 槽，内存里的空闲图每盘至多要 4 ×
BIN device_bytes=0 exit=1 stdout_lines=0 stderr_first=e142_new_pool_file_creation_write_dump 每盘 0 字节 mkfs 不收：mkfs：JournalRingTooLargeForDevice { ring_bytes: 805306368, device_bytes: 0 }
BIN device_bytes=1125899906842624 exit=1 stdout_lines=0 stderr_first=e142_new_pool_file_creation_write_dump 每盘 1125899906842624 字节：单元区 68719426560 槽，内存里的空闲图每盘至多要 4 × 68719426
BIN device_bytes=1099511627776 exit=0 stdout_lines=43 stderr_first=
```

2⁶⁴−1 退 1、stdout 零行、stderr 打原因（以前是 abort 退 134）；0 仍是 mkfs 拒、退 1；2⁴⁰（1 TiB）照常跑完（43 行 stdout）。2⁵⁰ 在这台机器上也被试分配拒掉（退 1）。
**试分配不是按算术判**，这台机器（`/proc/sys/vm/overcommit_memory` = 0）上试分配通过的中间档会在后面真分配时撞内存。我另跑了三档（编译在同一个副本里，4G 上限；原样输出，抄在下面）：

```text
== --device-bytes 8796093022208
run-with-memory-cap: 撞了内存上限 4G（scope singlefs-memory-cap-956772-949330074 的 Result=oom-kill，命令退出码 137）
== --device-bytes 35184372088832
run-with-memory-cap: 撞了内存上限 4G（scope singlefs-memory-cap-956851-1155929756 的 Result=oom-kill，命令退出码 137）
== --device-bytes 70368744177664
run-with-memory-cap: 撞了内存上限 4G（scope singlefs-memory-cap-957020-2882830348 的 Result=oom-kill，命令退出码 137）
```

2⁴³、2⁴⁵、2⁴⁶ 字节每盘试分配通过、后面真分配把 4G 撞满（包装退 250，命令自己退 137）。这就是正文说的「E142 的拒绝判据随机器 overcommit 变，第 24 批改成按算术判」，不记打中。
结论：N7 一致——2⁶⁴−1 退 1 不退 134。推翻它的现象：`--device-bytes 18446744073709551615` 退 134，或 2⁴⁰ 这类合法档被拒。

## 8 K1 · N8：用例改名与变异行

改名：旧名在全 `crates/` 里 `grep` 现数 0（`grep -rn the_version_before_the_unmount_released_in_either_warm_up crates` 无输出；全仓 `.rs`、`.tsv`、`.md`、`.sh` 里剩下的只有 `research/prompts/` 下归档的冻结物）。新名：「fn after_a_crashed_unmount_of_a_narrow_pool_overwritten_three_times_the_next_writable_mount_warming_up_twice_writes_no_unit_into_a_slot_released_by_any_version_after_the_root_device_one_alone_recovers_to(」（`crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:693`）；两处 `[…]` 文档链接（610、626 行）已改成新名，原先「名字没改：变异表里两行按这个名字点名」那行注释已删。
连带的变异行：`crates/mutations.tsv` 里点名这条的三行（2755、2756、2819 行）测试名列都是新名：「删除批第 21 批 N3：可写挂载在第一次暖机之后就放开重建分配器时扣住的那一截」（`crates/mutations.tsv:2755`）；「删除批第 21 批 N3：可写挂载只在真发那一处第一次暖机之后就放开」（`crates/mutations.tsv:2756`）；「删除批第 22 批 N5：可写挂载重建分配器时当场放开到 min(F_生效, max(每块盘上最新有效根的 txg 最小者」（`crates/mutations.tsv:2819`）。
按 `rerun.sh` 的 `anchors` 步核全部 25 行（追加 21 行：2823–2843 行；替换 4 行：1938、2755、2756、2819 行）：六段齐、原文在源文件里恰好命中一次、替换文与原文不同、测试函数存在：

```text
ANCHOR_RESULT rows=25 ok=25
exit=0
```

三行改名变异与 N5 两行 X 变异在副本里施加后点名的测试都红（`out/mut-n8.txt` 原样；N5 两行见第 5 节）：

```text
MUT_ROWS 2 prefix=删除批第 21 批 N3：可写挂载
MUT 删除批第 21 批 N3：可写挂载在第一次暖机之后就放开重建分配器时扣住的那一截 exit=101 test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 6 filtered out
MUT 删除批第 21 批 N3：可写挂载只在真发那一处第一次暖机之后就放开、取号之前的 exit=101 test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 6 filtered out
MUT_ROWS 1 prefix=删除批第 22 批 N5：可写挂载重建
MUT 删除批第 22 批 N5：可写挂载重建分配器时当场放开到 min(F_生效, m exit=101 test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 6 filtered out
```

正文写「连带 3 行变异；变异表追加 21 行、替换 4 行」，现数追加 21（2823–2843）、替换 4（1938、2755、2756、2819），对。GPU 那 18 行追加里每一行要 GPU 才红，没跑，见第 9 节。
改名后的测试单跑绿（`rerun.sh` 的 `tests` 步，第 5 节输出里第二段：`1 passed; 0 failed`，过滤串只命中它一个）。
结论：一致、变异行全。推翻它的现象：改名连带漏了一行（`grep` 旧名在 `crates/` 里还有命中），或某行锚点命中数不是 1。

## 9 线索与小出入（不记打中；每条写依据与推翻条件）

### L1 · I-9.17 / I-9.23 类型段 0 那一支，「子单元是码 2」在孩子是码 3 容器时测不到（CPU 与 GPU 同形，第 22 批起就有，第 23 批 GPU 照抄）

- 条款：「0 ⇒ 子单元是码 2（根长高之后，D8（核心索引结构） 已定项 6），身份引用的出生树、容器号、出生代三段为 0」（`.claude/kb/invariants.md:310`）。
- CPU：「let child_class = read_unit_without_judging(self.reader, &child, node_bytes())」（`crates/singlefs-checker/src/walk.rs:2128`）；读法 「fn read_unit_without_judging(」（`crates/singlefs-checker/src/walk.rs:3911`） 用位置条目里的整单元校验和比 `node_bytes()`（16384）字节：「.filter(|content| crc32_castagnoli_table(content) == location.checksum)」（`crates/singlefs-checker/src/walk.rs:3919`）。码 3 容器是 32768 字节、位置条目的校验和罩整单元，读 16384 字节永远对不上 ⇒ `None` ⇒ 「.is_none_or(|class| class == 2)」（`crates/singlefs-checker/src/walk.rs:2133`） 放过。GPU 同：「let child_unit = read_unit_via_locations(child, NODE_BYTES);」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2948`）；「let child_is_a_code_two_node_or_unreadable = child_unit.version == NONE || rd8(child_unit.base, 6u) == 2u;」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2949`）。
- 现算（探针 `T3b`：第一批那一版镜像上墓碑树根第一条条目类型段写 0、身份引用三段清零，孩子仍是码 3 容器；原样）：

```text
ZS_PROBE T3b_type_zero_zero_identity_child_is_a_container I-9.23 Holds
ZS_PROBE T3b_type_zero_zero_identity_child_is_a_container violated=["I-4.8", "I-7.2", "I-7.4"]
```

  条款说这一条该红 I-9.23；CPU 报 `Holds`，红的只有层级 1 里出现类型段 0 记的走读失败（I-4.8、I-7.2、I-7.4）。整张镜像仍红，被漏掉的是 I-9.23 这一格。GPU 与 CPU 同判，所以对拍看不出，T3 用例（出生树非 0）覆盖的是身份三段那一半。
- 来源：判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T7 写「重合小句照 I-9.2 先例留作合取」（28 行）；I-9.2 那边孩子是按期望类型定长读的，这里读长是 16 KiB，对码 3 孩子读不全。
- 严重度：低（镜像整体已红）。推翻它的现象：改读 32768 字节、或判决明说「读不出（含读长不够）的孩子这一小句不判」；若 T3b 在今天的 `crates/` 上 I-9.23 红，这条线索作废。归主 agent 现查坐实。

### L2 · GPU 侧还没有 N5 那一支（正文已划去第 24 批，不记打中），CPU 算出的 GPU 期望

N5 的测试 ② 那张镜像（最新根的实例表两份都翻坏，被抛弃的 C 同号不同出生代）上：CPU（有 N5）I-9.29 报部分没判、不红；GPU 没有「滤不掉就不比」，会像 N5 的第一条 X 变异那样直接跨版本比。我把那条变异施加到副本、取测试的失败信息当作 GPU 会给的结果（原样）：

```text
最新根的实例表（槽 50368）读不出、候选集滤不掉被抛弃的 C：I-9.29 要报部分没判，实际 Violated("inode 树（树 65537）里 inode 1 在回退候选集里对应不止一个对象出生代：对象出生代 3（根（实例 1、txg 3）的 inode 记录）、对象出生代 4（根（实例 2、txg 8）的 inode 记录）")
```

即 GPU 在该镜像上会置 I-9.29 位（下标 82），CPU 不红，对拍逐位比时这一格会分叉。kb 「GPU 判器第 23 批补」（`.claude/kb/invariants.md:316`） 写「第 23 批补」，字面上这一支没补完。归第 24 批。推翻它的现象：GPU 的候选集在实例表读不出时已经另有处理（我读 `crash_judge_checker.wgsl` / `…_scan.wgsl` 没有找到这样一支）。

### L3 · 小出入

1. 走读内核还有写死的对象出生代偏移 `ro + 8u` 两处：「w_judge(INVARIANT_I_9_26, le64(rd64(p.base, ro + 8u), change_count)」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2651`）、「let birth = rd64(p.base, ro + 8u);」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2654`）（共用那一半已有 「const INODE_RECORD_OBJECT_BIRTH_OFFSET: u32 = 8u;」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:139`））；不改结果，「一处定义」没做到底。
2. 扫描内核删除态扫描那一节的头注释只列 I-9.5、I-3.4、I-4.7、I-4.6：「I-9.5、I-3.4、I-4.7、I-4.6 在回退候选集里每条根与由记录施加出来的那几版上各重读一遍树表与几棵树（只读）」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2062`）；这一节现在还判 I-9.24。文件头注释（1–4 行）已更新。
3. 正文 N4 一格把 I-9.29 与 I-9.10 各数一条（`research/prompts/_m3-batch23-code-r1-body.md` 16 行），实际一条用例断言两条，新加用例数 13 对，列举出来是 14。
4. GPU 18 行变异（`crates/mutations.tsv` 2823–2840 行）和 GPU 对拍 13 条都需要显卡，我没跑；这些行「会红」只有读码推导（每行的锚点命中数核过，第 8 节）。变异 3（不登记引用）在 T2 流上会不会红，取决于那片容器的两槽还有没有别的版本引用；T2 的改动用重封把所有引用者的校验和一起换了，我推的是红（容器只经被改的墓碑树根被引用），没跑。
5. 驱动编译这个扫描内核的时间没有数：新增的函数在扫描内核里的调用点数（现数，含定义行）：

```text
d_read_inode_records( 5
d_read_containers_of_a_container_tree( 4
d_read_the_lineage_of_a_version( 2
d_read_deletion_queue( 3
d_txg_is_on_the_lineage( 5
d_note_object_birth( 3
```

   `crash_judge_checker.wgsl` 里有一段注释说显卡上的程序没有真的函数调用、驱动的编译器把每个调用点各展开一份，并给了走读内核冷编 523 秒的前科：「原来五处走根、一处走版本各展开一份时，走读内核展开后 242 万记号，本机（驱动 595 版）冷编 523 秒」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3055`）。扫描内核没有这样的记录；`d_read_inode_records` 现有 4 个调用点。提交时的 GPU 档要留意编译时间与常驻内存。
6. N5 在 kb 字面（合取）与实现（析取）之间的读法差见第 5 节。

## 10 K3：被判的 13 份文件，每份一格

| # | 文件（`crates/` 下） | 落的 N | 判定 | 一句话 |
|---|---|---|---|---|
| 1 | `mutations.tsv` | N8、N1–N7 各自的变异 | 一致 | 追加 21 行（2823–2843）、替换 4 行（1938、2755、2756、2819）；25 行锚点各命中 1 次、替换不同于原文、点名的测试函数存在；3 行改名连带全；N5 两行与 N8 三行副本里施加后红；GPU 18 行与 X 的 E142 一行未跑（后者是 bin 内单测，checker 档） |
| 2 | `singlefs-checker/src/walk.rs` | N5 | 一致（保守方向有一处读法差） | 枚举 + 最新根实例表 `…without_judging` 判滤没滤全，`None` 则 I-9.29 不比、报部分没判；复跑 N5 测试绿、两行变异红；与 kb 字面有「读不出」对「读不出且滤不掉」的读法差（第 5 节） |
| 3 | `singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs` | N7 | 一致 | 建盘前试分配，2⁶⁴−1 退 1；2⁴⁰ 照跑；中间档（2⁴³ 起）仍撞内存，归第 24 批 |
| 4 | `…/src/crash_judge_checker_common.wgsl` | N1 | 一致 | 九个新下标 74–82 与 `IMPLEMENTED_INVARIANTS` 次序一一对上；五张表的起点逐段相加到 315376，与 Rust 一侧相等 |
| 5 | `…/src/crash_judge_checker_scan.wgsl` | N1、N2、N3 | 一致 | I-9.24 / I-9.18–I-9.21 / I-9.29 / I-9.10 墓碑那一个等号逐句同 CPU；六处超上限记判不了；size / 出生代偏移取共用常量；小出入：节头注释未列 I-9.24 |
| 6 | `…/src/crash_judge_checker.wgsl` | N1、N2 | 一致 | I-9.17 / I-9.23 四支同判同位；I-9.4 第四句同位置同条件；小出入：两处 `ro + 8u` 写死；线索 L1 与 CPU 同形 |
| 7 | `…/src/crash_judge_common.wgsl` | N3 | 一致 | 零条目的 extent 上段根与 inode 树根只在有条目时核宽，与 core 逐条解条目才核宽同；CPU 探针恢复答 `NoFile` |
| 8 | `…/src/crash_judge_dispatch.rs` | N1（单测常量） | 一致 | 1797 → 1702、10 782 → 10 213，用 `2_147_483_644 // (315376 × 4)` 与 `12 GiB // …` 算出相等 |
| 9 | `…/src/crash_judge_gpu.rs` | N1 | 一致 | `SCRATCH_WORDS` = 315 376 = `SCRATCH_END`；覆盖表仍 0..=82；文档把 I-9.22 恒不置位、I-9.4 第四句并进 40 写明 |
| 10 | `…/tests/crash_enumeration_new_pool_file_creation_stream.rs` | N6 | 一致（单状态核过，计数待层 0） | 分组项数 26 / 10 / 33 与文档的数对；全持久状态与 mkfs 状态的判定与分组同向；9 个状态的计数要跑层 0 |
| 11 | `…/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` | N4 | 一致 | 13 条用例各自的改坏动作在 CPU 探针上红的就是名字里那一条；合法流九条不红；GPU 一侧未跑 |
| 12 | `singlefs-harness/tests/checker_known_bad_images.rs` | N5 | 一致 | 新测试 ① 滤得掉时 I-9.29 `Holds`、② 读不出时部分没判；绿，两条 X 变异都红 |
| 13 | `singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs` | N8 | 一致 | 旧名 `crates/` 里 0 处，新名 693 行；过滤串只命中它一个，绿；三行变异施加后红 |

13 份与 `research/prompts/m3-batch23-code-r1-snapshot/crates-sha256.txt` 的清单逐份对上（第 0 节）。

## 11 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 N1 GPU 八条置位 | 一致 | 走读 I-9.17 / I-9.23 四支、扫描 I-9.24 / I-9.18–I-9.21 / I-9.29 逐句同 CPU；I-9.22 恒不置位；六处超上限记判不了；草稿区 298 736 → 315 376 算术对 |
| K1 N2 I-9.4 第四句 / I-9.10 墓碑等号 | 一致 | 并进下标 40 / 43；同位置同条件 |
| K1 N3 零条目根与 size 偏移 | 一致 | GPU 恢复与 core 同判，CPU 探针答 `NoFile`；size 偏移一处定义，旧名 0 处；小出入：出生代偏移还有两处写死 |
| K1 N4 对拍 13 条 | 一致 | 每条的改坏动作在 CPU 上红名字里那一条（探针原样）；合法流不红；GPU 一侧未跑 |
| K1 N5 I-9.29 部分没判 | 一致（读法差一处） | 滤法对得上、只有保守方向的差；与 kb 字面「读不出且滤不掉」的读法差交主 agent；GPU 侧第 24 批 |
| K1 N6 分组 | 一致（单状态核） | 26 / 33 / 10 的项数对，全持久与 mkfs 状态的判定与分组同向；9 状态计数待层 0 |
| K1 N7 E142 退出码 | 一致 | 2⁶⁴−1 退 1；中间档试分配通过后撞内存，已划去第 24 批 |
| K1 N8 改名与变异行 | 一致 | 25 行锚点各命中 1 次；3 行改名连带全；复跑变异红 |
| K3 13 份 | 每份一格 | 全部「一致」，小出入与线索见第 9 节 |
| 线索 L1 | 不记打中 | I-9.23 类型段 0 的「码 2」子句对码 3 孩子测不到（CPU = GPU，第 22 批起） |
| 线索 L2 | 已划去第 24 批 | GPU 缺 N5 那一支，CPU 算出的 GPU 期望已给 |

## 12 没做什么

- 没跑 GPU（无论对拍、变异还是驱动编译）：GPU 一侧是读 wgsl、用 CPU 探针算期望；GPU 那 13 条对拍、18 行 GPU 变异、E142 单测 `the_largest_device_bytes_parse…`（checker 档包里的 bin 单测）、`SCRATCH_WORDS` 单测，都归提交时。
- 没跑层 0 与任何重型测试；N6 的 9 个状态计数只核了单个状态的判定。
- 没造镜像逐形验「实例表读不出 / 读得出」两个读法在所有形上的关系（第 5 节是读码结论）。
- 不判攻方腿的格（K2），不替主 agent 采纳或出判决；L1、L2、第 5 节的读法差、第 9 节小出入交主 agent 现查。
- 没读攻方腿的报告与模型（`research/prompts/m3-batch23-code-r1-opus-*`、`/tmp/claude-1000/m3-batch23-code-r1-opus/`），没读本轮判决文件。
- 自己建的编译目录与仓副本在 `/tmp/claude-1000/m3-batch23-code-r1-sonnet/`（`work/`、`target/`、`run*/`），交回前删除；报告引用的草稿证据（`rep/`、`e142-*.err`）随目录一并删，其内容已按原样抄进本报告。

