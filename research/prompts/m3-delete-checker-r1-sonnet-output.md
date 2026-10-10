# m3-delete-checker-r1 云端正推腿报告（Sonnet，格 K1、K3）

日期 2026-10-05。背景材料 `research/prompts/_m3-delete-checker-r1-background.md`，正文 `research/prompts/_m3-delete-checker-r1-body.md`。冻结副本 `/tmp/claude-1000/m3-delete-checker-r1/tree/`。引代码写冻结副本里的文件:行号；引 kb 写 kb 文件自己的行号。每一句引文都在写进报告前用 `grep -nF` 对过原文（行里带「」的整行不引，改写成转述并标「转述」）。

## 0 开工核对与读法约定

- 冻结副本校验与 `nlink` 零命中的原样输出在第 10 节。
- 没编译、没起任何测试：重型测试不跑，冻结副本里也没有 target 目录。凡写「推的」都是读代码推出来的，没造镜像；造镜像是攻方腿（K2）的活。
- 读过主 agent 的前置调查 `/tmp/claude-1000/m3-prep-3-3/report.md` 当线索。每处事实自己在冻结副本里重查；与它不同的地方标「与前置调查不同」。
- 候选名沿用正文第一节表（F1 的 A / B / 丙，F2 的 甲 / 乙 / 丙，F3 的 甲 / 乙1 / 乙2 / 丙）。我自己提的新候选一律标「零轮」。
- 「字面 / 扩读 / 缩读」的判法：定义原句里的每个分句，候选是不是照它判；多判了定义没说的算扩读，少判了定义说的算缩读；某一分句在候选下永远为假或永远为真的，记「某句落空」。

## 1 先立一张「合法历史会造出哪些盘面」的目录

后面每个候选「合法镜像上会不会误红」都拿这张表比。每行的出处是冻结副本里的代码。

- L0 没有删除：inode 的 nlink 是 1，队列空。队列空时树表里 `deleted_inodes` 条目的根指针置零、一个节点都不写：「队列 = inode 树里 nlink = 0 的号集合（派生态，第 3 条）。队列空时树表条目的根指针置零、一个节点都不写；」（`crates/singlefs-core/src/deleted_inodes.rs:2`）
- L1 意图（inode N）：记录写成 nlink = 0、size 照抄、数据单元与 extent 树不动、队列 {N}。「nlink: NLINK_OF_AN_INODE_BEING_DELETED,」（`crates/singlefs-core/src/transaction.rs:5879`） 与 「size: record.size,」（`crates/singlefs-core/src/transaction.rs:5880`）
- L2 不是最后一批的一批：size = 剩下单元数 × 净荷容量，nlink 仍 0，留下的单元数 ≥ 1。分批判据在 「if data_units_remaining <= data_units_per_batch {」（`crates/singlefs-core/src/unlink.rs:120`），size 的算式从 「size_after_the_batch_in_bytes: data_units_kept」（`crates/singlefs-core/src/unlink.rs:128`） 起。
- L3 最后一批：删记录、从队列删号、放掉剩下全部单元，同一次发布。「UnlinkStep::FinalBatch { .. } => DeletionStepOfThePublish::FinalBatch { inode },」（`crates/singlefs-core/src/transaction.rs:5949`） 队列的变化：「pub fn deleted_inodes_after_this_publish(self, before: &[InodeNumber]) -> Vec<InodeNumber> {」（`crates/singlefs-core/src/transaction.rs:3991`）
- L4 空 inode 的意图：第一版只有 inode 1 写过数据，别的号剩余单元数取 0：「let data_units_remaining = if inode.0 == FIRST_INODE_NUMBER {」（`crates/singlefs-core/src/transaction.rs:5927`） 新建的 inode 长度 0：「size: 0,」（`crates/singlefs-core/src/transaction.rs:5810`） 所以 L1 之后下一发布就是最后一批。
- L5 清空过的文件的意图：把内容覆盖成空，文件长度 0 但仍有 **一个** 声明长度 0 的数据单元。单元数的算式：「content_length_in_bytes.div_ceil(payload_capacity).max(1)」（`crates/singlefs-core/src/write_request_split.rs:61`） 测试钉住：「assert_eq!(emptied.data_pointers.len(), 1, "空内容也写一个数据单元");」（`crates/singlefs-harness/tests/core_review_geometry_back_chain_and_empty_inode.rs:652`） 与 「assert_eq!(emptied_file.inode_record().size, 0, "长度 0");」（`crates/singlefs-harness/tests/core_review_geometry_back_chain_and_empty_inode.rs:659`） 在这样的文件上发意图，size 照抄（transaction.rs:5880）= 0，单元数 1。这一版是 nlink = 0、size = 0、1 个单元（推的：`publish_unlink_intent` 只要求记录 nlink ≠ 0，不看 size）。
- L6 管理员回退：新根的 inode 树、extent 树与 `deleted_inodes` 根指针一起取 R_old 的：「已定项 14：新根的 inode 树、extent 树与 `deleted_inodes` 的根指针取 R_old 的；墓碑树、谱系树、logged_ops 随现行那一版）。」（`crates/singlefs-core/src/mount.rs:5801`） 所以候选集里每一版三者自洽。
- L7 删空的一版：inode 树、extent 树零条目根，`deleted_inodes` 零根指针（D8 已定项 16 第 5 条，`.claude/kb/decisions/08-核心索引结构.md:468`，转述）。

由此得到三条后面反复用的事实：

1. 可达的合法盘面里，`deleted_inodes` 里的号集合恒等于 inode 树里 nlink = 0 的号集合（D8 已定项 16 第 3 条逐字：「3. **建树**：`deleted_inodes` 在第一个文件版本那次发布零根注册；它是派生态：队列 = inode 树里 nlink = 0 的号集合（`research/prompts/m3-delete-r2-main-verification.md` 第三节 V7）。」（`.claude/kb/decisions/08-核心索引结构.md:466`））。
2. 「有数据单元、没有 inode 记录」的盘面不可达：最后一批在同一次发布里删记录、删 key、放单元（L3）。
3. 「nlink = 0、size 与单元数按 ⌈size ÷ 净荷容量⌉ 对不上」的盘面 **可达**（L5：0 对 1）。这一条是本报告最要紧的一个发现，F3 展开。

推翻第 1 条的现象：任何一条由 `publish_unlink_*` 发出的版本里 Q 与 nlink = 0 集合不等（K2 放开扫描的活）。推翻第 2 条：任何合法历史里出现 extent 上段叶有数据条目而 inode 树无该号记录的版本。推翻第 3 条：L5 那一版在 harness 上发不出来（读 `publish_unlink_intent` 的 `find(... nlink != 0)`，没有 size 的条件）。

## 2 F1　I-4.7「处于删除中的 inode」指谁

### 2.1 从定义推

定义整行：「| I-4.7 | 删除中 inode 有账 | 任一处于删除中的 inode，要么在删除中队列里有条目，要么其 extent 已全部释放——两者皆非的中间态不存在 | 未实现 |」（`.claude/kb/invariants.md:183`）

句子是「任一〔处于删除中〕的 inode，要么〔分句一：在队列里有条目〕，要么〔分句二：其 extent 已全部释放〕」。前件「处于删除中」定义原句没有解释；决策那边给了操作定义：

- D8 已定项 6 末句（转述）：inode 记录在回收完成前留在 inode 树里、nlink = 0，`deleted_inodes` 只存号（`.claude/kb/decisions/08-核心索引结构.md:173`）。
- D8 已定项 16 第 3 条：队列是派生态，等于 nlink = 0 的号集合（引文在第 1 节事实 1）。
- D8 已定项 16 第 4 条：从意图提交起读者答「没有文件」（`.claude/kb/decisions/08-核心索引结构.md:467`，转述）。

也就是说在 D8 的设计里「删除中」的标记是 **nlink = 0**，队列是它的派生。读者只看到「没有文件」，标记之外的「记录已删、单元还在」不叫删除中——那个盘面在设计里不可达（第 1 节事实 2）。

### 2.2 逐候选（判法 / 字面度 / 合法镜像误红 / 能否单红 / 要改 kb）

| 候选 | 判法一句 | 对定义的字面度 | 合法镜像误红 | 坏镜像能不能只红自己 | 要改 kb |
|---|---|---|---|---|---|
| A | 每条 nlink = 0 的记录：号在 `deleted_inodes` 里，或该号在 extent 上段叶没有数据条目 | 前件取 D8 的操作定义，两分句都照判，**字面**。但分句二在可达盘面上从不单独起作用：L1、L2、L4、L5 的号都在队列里，L3、L7 没有 nlink = 0 的记录；它只放过 {nlink = 0、不在队列、无单元}，而 D8 已定项 16 第 3 条禁止这个盘面（队列 = nlink = 0 集合）。即 I-4.7 字面比 D8 弱 | 否：L0–L7 全不红 | 红集 {nlink = 0 ∧ 不在队列 ∧ 有单元} ⊂ I-9.5 反方向（C708）的红集 {nlink = 0 ∧ 不在队列}。I-9.5 带反方向时 A 的任何坏镜像同红 I-9.5，单红做不到；I-9.5 仍只有正方向（kb 今天的字面）时，checker 里别的条都不读 nlink（零命中见第 10 节），A 的镜像只红 I-4.7（推的） | 183 行写明前件；若 I-9.5 加反方向，183 行状态列写 A 嵌在 I-9.5 里 |
| B（ii）「活记录」取 nlink ≥ 1 | 每个在 extent 上段叶有数据条目（LowerSegmentRoot 或 InlineDataUnit）的号：inode 树有它 nlink ≥ 1 的记录，或号在队列里 | **扩读**：把「删除中」扩到「记录已没了、单元还在」，而 D8 的设计里这是泄漏、不是中间态。**分句二落空**：选号的条件就是有单元，「extent 已全部释放」永远为假。「活记录」取 nlink ≥ 1 与代码里 `n_live` 同义：「副本数 × ((2 + H + 1) × 16384 + 41 × n_live)，H 是这次之后墓碑树的内部层数，n_live 是这次之后 inode 树里 nlink ≥ 1 的记录数」（`crates/singlefs-core/src/transaction.rs:4154`） | 否（L1–L7 无「有单元、无活记录、不在队列」） | 红集 = A 的红集 ∪ 孤儿（无记录、有单元）。孤儿那一块与 I-9.5 两个方向都不交；I-9.10 对没有记录的号明写不判：「if let Some(record_birth) = walk.inode_object_birth.get(inode) {」（`crates/singlefs-checker/src/walk.rs:8369`） 所以孤儿镜像可只红 I-4.7（推的）。A 那一块仍同红 I-9.5 | 183 行写明前件与「有单元」的口径 |
| B（i）「活记录」取「记录根本不在」 | 同 B（ii），但前件只取无记录的号 | 同上，扩读更多；分句一对它也是死分句（I-9.5 正方向要求队列里的号有记录，无记录的号不可能在队列里） | 否 | 红集 = 孤儿，与 I-9.5、I-9.10 都不交，**可单红**（推的） | 同上 |
| 丙 = A ∪ B | nlink = 0 的记录按 A 判；有单元而无记录的号直接判红 | 两分句都有活的定义域（分句二活在 A 的那一块），但前件是 D8 操作定义的并集，比定义字面宽 | 否 | 用孤儿镜像可单红；A 那一块同红 I-9.5 | 同 B |

### 2.3 这一格的推论

- 照定义的 **字面** 推，落在 A（前件取 D8 的操作定义）。
- 照三·3 验收「每条只红自己」推，落在 B 或丙：只有孤儿那一块不与 I-9.5 相交。
- 两个目标在 I-9.5 带反方向时互相冲突，**定义本身分不出来**；A 另有一个文字上的缝：I-4.7 的分句二放过 C708 举的那一形（空 inode、nlink = 0、不在队列，`.claude/kb/checks-owed.md:563` 转述），所以 C708 的反方向收不进 I-4.7，只能在 I-9.5。
- 取不下来的原因是一个前提：I-9.5 反方向进不进（C708 要求先写进 kb）。交用户，K3 第 1 行。

推翻本节的现象：① 在 harness 上造出 A 的坏镜像且 I-9.5 不红（说明红集不嵌套，前提是读错 nlink 偏移或 I-9.5 没带反方向）；② 合法历史里出现「有单元、无活记录、不在队列」的版本（B 与丙会误红，翻回 A）；③ 出现 {nlink = 0、不在队列、无单元} 的合法盘面（则 D8 第 3 条「队列 = 集合」被推翻，A 的分句二才有定义域）。

## 3 F2　I-4.4 在第一版判什么

### 3.1 从定义推

定义整行：「| I-4.4 | 意图只描述目标态 | 任一未完成的意图 key，其字段只描述目标状态，不含任何增量字段（"再删 N 个"一类） | 未实现 |」（`.claude/kb/invariants.md:179`） 压着的 D8 已定项 5 第 1 条（转述，该行带「」不整行引）：意图声明目标状态、绝不声明增量（`.claude/kb/decisions/08-核心索引结构.md:122`）。第一版意图的载体，D8 已定项 16 第 1 条逐字：「1. **意图**：意图那次发布把 inode N 的记录写成 nlink = 0（size 不变），`deleted_inodes` 加 key = N（8 字节、无 value、不带对象出生代），往墓碑树开放容器追加一条墓碑（D18（块里携带什么信息） 已定项 24）；不立 logged_ops（K-U1 L4）。」（`.claude/kb/decisions/08-核心索引结构.md:464`）

第一版意图有三处载体，没有一处装得下「再删 N 个」：

- `deleted_inodes` 里的 key：8 字节、无 value，条目宽被 I-1.10 钉死。checker 今天已判种类 9 的条目宽：「.judge("I-1.10", entry_width == entry_field_table_bytes, || {」（`crates/singlefs-checker/src/walk.rs:1912`） 登记宽是 8：「TREE_KIND_DELETED_INODES => Some(DELETED_INODES_ENTRY_BYTES),」（`crates/singlefs-checker/src/walk.rs:213`）
- nlink = 0 的 inode 记录：140 字节全是定长状态字段，预留段恒 0 由 I-9.7 判：「record[108..140].iter().all(|byte| *byte == 0),」（`crates/singlefs-checker/src/walk.rs:2340`） 唯一随批进度变的字段是 size（和随它走的 blocks）。
- 墓碑：整对象区间（D18 已定项 24，本轮不判）。

所以「字段只描述目标状态」在第一版由格式本身保证（I-1.10 加 I-9.7），I-4.4 没有自己独有的、别的条判不到的对象，除非借用 size 那一格或造一条定义里没有的约束。

### 3.2 逐候选

| 候选 | 判法一句 | 对定义的字面度 | 合法镜像误红 | 坏镜像能不能只红自己 | 要改 kb |
|---|---|---|---|---|---|
| 甲 | （1）每版 `deleted_inodes` 条目宽 = 8；（2）每版树表里 logged_ops（种类 12）条目根指针全零 | （1）就是 I-1.10 在种类 9 上的同一个谓词，零新增判力；（2）定义说「字段」，不说「那棵树必须是空的」，是第一版写者行为（D8 已定项 5 与 16 第 1 条「不立 logged_ops」）搬进了不变量：**扩读**。D8 已定项 5 自己写了 logged_ops 的字段表留到第一个写者（整理）落地时定（转述，`.claude/kb/decisions/08-核心索引结构.md:108`），写者一落地（2）就在合法镜像上红 | 现在：否（第一版不写 logged_ops）。整理或截断一落地：会 | （1）不能：改宽的镜像同红 I-1.10。（2）可能，但镜像带来的连带要攻方量：走读对种类 12 一出 `key_schema_for_tree_kind` 的 None 就返回（「let Some(schema) = key_schema_for_tree_kind(kind) else {」（`crates/singlefs-checker/src/walk.rs:1816`）），不记引用，指到已记账单元的非零根会让 I-3.1 一类按引用判的条跟着变（推的）。实现一侧恢复路径已把种类 12 的非零根按「树的种类没登记」拒（「let key_width = key_width_for_kind(entry.kind).ok_or(RecoveryFailure::UnitMalformed {」（`crates/singlefs-core/src/recovery.rs:4899`），推的，没跑），这一形的归属更像「树种类登记」类不变量，不像 I-4.4 | 179 行写清判什么；整理落地那天删（2） |
| 乙 | 第一版对 I-4.4 报「不适用」并写理由：载体没有可判的字段 | **字面**（量词「任一意图 key 的字段」在第一版是空域，空真）。与上一轮 Sonnet 的判法同向：`research/prompts/m3-delete-r1-sonnet-output.md:49`（转述：该行写 I-4.4 在这个形态上恒真） | 否（恒不适用） | 无坏镜像可言：判不出红。三·3 验收「坏镜像各一、`crates/mutations.tsv` 各一行」对它做不到 | 179 行状态列；`.claude/kb/milestone/03-third-txn.md:17`（出口 1 点名 I-4.4 实现）与 `.claude/kb/milestone/03-third-txn.md:85`（三·3 验收）要改措辞，**归用户** |
| 丙 | nlink = 0 记录的 size 等于由单元数现算的值（落到记录字段上） | 定义的「字段」可以读到记录上（D8 已定项 16 第 1 条把 nlink = 0 的记录也列为意图的一部分），size 是唯一会漂成「增量」的字段（把「还剩多少」写成差值），**可读成字面**。但判式与 F3 的乙1 是 **同一个谓词、同一组镜像** | 与 F3 乙1 同：式子写成 ⌈size ÷ 净荷容量⌉ 时在 L5 上误红（见 4.2） | 与乙1 同：size 与 blocks 一起改的镜像只红这一条。同一个谓词挂两个名字，则一份镜像同红 I-4.4 与 I-4.6（若 I-4.6 取乙1），不能各自单红；只能二选一宿主 | 179 行写明判 size；182 行同步 |

### 3.3 零轮新候选

- 丁（零轮）：宿主放 I-4.4（取丙），I-4.6 取甲（只做合取），乙1 的式子不重复写。与「宿主放 I-4.6、I-4.4 取乙」对称；谁当宿主的区别只是名字，红集相同。一份镜像仍只红一条的前提是另一条不再判这个谓词。
- 戊（零轮）：甲的（2）改成「报部分没判」而不是红（logged_ops 有根时字段表未定，说明写没判到的是什么）。去掉了整理落地时的误红，代价是 I-4.4 在任何镜像上都不会红，验收的坏镜像仍做不到。

### 3.4 这一格的推论

- 照定义字面推到 **乙**。乙与出口 1 的「实现」二字冲突，按正文第五节「改出口 1 措辞的候选一律交用户」，交用户。
- 不改出口措辞的两条路是甲与丙，各有代价：甲多判的（2）是定义没说的约束且会过期；丙把 F3 乙1 的谓词搬了个名字。
- 交用户时带 K3 第 2 行。

推翻本节的现象：① 第一版出现任何能让 `deleted_inodes` 条目或 nlink = 0 记录携带增量的写路径（例如条目宽 > 8 却被 I-1.10 放过）——那时乙的「空域」前提不成立；② 用户在出口措辞里接受「恒不适用也算实现」，则乙直接成立、甲丙都不需要。

## 4 F3　I-4.6 判什么

### 4.1 从定义推

定义整行：「| I-4.6 | 中间态其余不变量成立 | 存在未完成意图的镜像上，其余全部不变量仍然成立（分批之间的中间态是合法状态） | 未实现 |」（`.claude/kb/invariants.md:182`）

压着它的决策与判决（都带「」，不整行引，转述）：D8 已定项 5 第 3 条要求中间状态每一个都是合法的文件系统状态（`.claude/kb/decisions/08-核心索引结构.md:124`）；D8 已定项 16 第 2 条定每批 size 改成「剩下单元数 × 净荷容量」（`.claude/kb/decisions/08-核心索引结构.md:465`）；设计轮第一轮判决 U5 一行写明：池级 checker 不判单元数对得上 size，进三·3，I-4.6 光靠今天的 checker 判不出这一格（`research/prompts/m3-delete-r1-main-verification.md:52`）。

所以 I-4.6 有两层来历：定义字面是「其余各条的合取」（甲）；决策链点名要补的缺口是「size 与单元数」（乙）。

### 4.2 乙的式子要先改：⌈size ÷ 净荷容量⌉ 在可达盘面 L5 上误红

正文与前置调查写的式子是 ⌈size ÷ 净荷容量⌉。实现的除法多一个下界。写侧：「一次顺序写请求要写几个数据单元：内容长度除净荷容量向上取整，长度为 0 时是 1」（`crates/singlefs-core/src/write_request_split.rs:56`） 与 「content_length_in_bytes.div_ceil(payload_capacity).max(1)」（`crates/singlefs-core/src/write_request_split.rs:61`） 恢复路径拿同一个函数判第一个文件：「第一个文件的数据指针个数等于 inode size 按净荷容量除出来的单元数（D4（校验和位置） 已定项 5；与写侧切分同一条除法）——」（`crates/singlefs-core/src/recovery.rs:4565`） 判式在 「!= data_unit_count_of_a_sequential_write(inode_record.size)」（`crates/singlefs-core/src/recovery.rs:4572`）

- 内容覆盖成空的文件：size = 0，数据单元 1 个（第 1 节 L5 的测试行）。
- 在它上面发意图：记录 nlink = 0、size = 0、单元 1 个（推的：`publish_unlink_intent` 只找 nlink ≠ 0 的记录，size 照抄）。这一版按 ⌈0 ÷ 32634⌉ = 0 对 1，**乙1 在合法的 unlink 上误红**；乙2 还会在活文件（nlink = 1、size 0、1 个单元）上误红。
- 修正式：有数据的号（extent 上段叶条目是 LowerSegmentRoot 或 InlineDataUnit，枚举见 「ExtentUpperLeafEntryView::LowerSegmentRoot { inode, pointer } => {」（`crates/singlefs-checker/src/walk.rs:2998`） 与 「ExtentUpperLeafEntryView::InlineDataUnit { inode, pointer } => {」（`crates/singlefs-checker/src/walk.rs:3006`））单元数 = max(1, ⌈size ÷ 32634⌉)；没有数据的号（L4 的空 inode）size = 0 且单元数 = 0。批后 size = 留下单元数 × 32634，修正式下恒成立（留下 ≥ 1）。
- checker 自己有净荷容量常量（不调 core）：「pub fn data_unit_payload_capacity_in_bytes() -> u64 {」（`crates/singlefs-checker/src/position_addressed.rs:25`）
- kb 里没有 size 与单元数的条款（第 10 节 grep 零命中）：「长度 0 取 1 个单元」只在代码和测试里。式子要先写进 kb（D4、D8 或 I-4.6 那一行本身），否则 checker 的式子只能从 core 的注释抄，等于没有独立来源。

推翻 4.2 的现象：L5 在 harness 上发不出来；或 `data_unit_count_of_a_sequential_write` 不是第一版唯一的切分入口且存在 size = 0 零单元的活文件（那时修正式的「取 1」要分情况）。

### 4.3 逐候选

| 候选 | 判法一句 | 对定义的字面度 | 合法镜像误红 | 坏镜像能不能只红自己 | 要改 kb |
|---|---|---|---|---|---|
| 甲（字面） | 有未完成意图的版本上，除 I-4.6 外已实现的每一条都不红 | **字面**。但「其余全部」含 I-4.4、I-4.7、I-9.5、I-3.4 这四个兄弟条 | 没有独立误红：它红当且仅当别的条红 | 做不到：红永远伴随别条红。兄弟条的坏镜像也同红它（见 4.4） | 182 行状态列写明；三·3 验收「每条只红自己」（`.claude/kb/milestone/03-third-txn.md:85`）对它与兄弟条不成立，**归用户** |
| 甲'（零轮） | 同甲，但「其余」读成除这五条之外的已实现条 | **缩读**：把「全部」收窄，为的是兄弟条的坏镜像不再被它拖红 | 同甲 | 自己仍不能单红；兄弟条的镜像不再同红它 | 182 行文字改「其余」的范围 |
| 乙1（式子取修正式） | nlink = 0 的记录：该号单元数等于修正式 | **缩读**（只判中间态记录，不判「其余」），但补的正是 U5 点名的缺口 | 修正式下否（L1–L5 全过：L2 批后 size 恰为整倍数，L1 与 L5 走 max(1, ⌈⌉)，L4 走无数据支） | **可单红**（推的）：改 size 并同步 blocks（只有 I-9.15 读 size，「let size_in_bytes = read_u64(&record, INODE_RECORD_SIZE_OFFSET);」（`crates/singlefs-checker/src/walk.rs:2346`）），队列与 nlink 不动，别的条不读这两个字段 | 182 行写式子；补 size 与单元数的 kb 条款 |
| 乙2 | 每条记录都判修正式 | **扩读**：活文件不是「中间态」，式子是 D4 与恢复路径已有的结构性质，checker 没有登记过的条承接它，挂在 I-4.6 名下名不副实 | 修正式下否；沿用 ⌈⌉ 式在 L5 的活文件上误红 | 可单红（推的） | 同乙1；更合适另立 I-9.x（零轮） |
| 丙 | 甲或甲' 加乙1 | 字面加补缺口 | 同乙1 | 乙1 那一份镜像可单红；甲的部分不能 | 两处都改 |

### 4.4 「触发」与兄弟条：甲不可能让五条都单红

甲要先判「存在未完成意图」。三种取法：T1 队列非空；T2 有 nlink = 0 的记录；T3 二者之一。五条里兄弟条的坏镜像（推的，各自最自然的一份）：

- I-9.5 正方向（队列里有号、记录 nlink ≠ 0 或没有记录）：T1 真，T2 假。
- I-9.5 反方向（nlink = 0 而不在队列）与 I-4.7 的 A 镜像：T1 假，T2 真。
- I-4.7 的孤儿镜像（无记录、有单元、队列空）：T1、T2 都假，甲不触发。

没有一种触发取法让 I-9.5 的两个方向都不触发甲。所以甲（字面）下 I-9.5 至少一个方向的坏镜像同红 I-4.6，三·3 验收对 I-9.5 落空；甲' 才消得掉。别的已实现条里任何一条的坏镜像若造在含意图的版本上也同红甲，这是它的定义，不算缺陷，但 harness 里 `verdicts_that_differ_outside` 一类的断言要把 I-4.6 列进例外。

### 4.5 这一格的推论

- 能单红、能挂变异的内容只有乙1（修正式）；乙2 是扩读。
- 甲要不要加、加字面还是加甲'，是「字面忠实」对「三·3 验收」的取舍，定义推不出来。交用户，K3 第 3 行。

推翻本节的现象：① 合法历史里出现 nlink = 0 的记录单元数与修正式不等（K2 放开批大小、文件单元数、崩在哪批、回退到哪版扫一遍）；② 找到一种触发取法使 I-9.5 的两个方向、I-4.7 的 A 镜像都不触发甲（则甲字面可行）；③ 乙1 的镜像除了 I-4.6 还红了别的条（说明有条在读 size 或 blocks 之外的联动字段）。

## 5 F4　I-9.5、I-4.7、I-4.6 共用的三件

### 5.1 F4-a　判哪几版

事实（冻结副本，现查）：

- 走读先走最新根，再走候选集里每条更早的根，最后走「只由记录施加出来、没有根槽」的几版。候选条件：「*index == newest_index || (!abandoned && !below_floor && !outside_the_window);」（`crates/singlefs-checker/src/walk.rs:8099`） 记录施加出来的版本：「let versions_applied_only_by_records = versions_applied_only_by_records(」（`crates/singlefs-checker/src/walk.rs:8147`）
- 按版判的先例：I-9.25 拿每一版自己的 txg 判自己的树表：「let version_txg = self.version_being_walked.checkpoint_txg();」（`crates/singlefs-checker/src/walk.rs:1961`） 按最新根判的先例：记账行只留最新根那一份：「let accounting = walk.accounting.clone();」（`crates/singlefs-checker/src/walk.rs:8026`）
- 与本题最近的条款 I-9.24（同一个 `deleted_inodes` 的对象）明写每个被判的版本各判一次、不遍历版本之间的历史（转述，行带「」：`.claude/kb/invariants.md:311`）
- 主走读对共享单元只走一遍，第二次读到同一落点 `read_index_node` 交回 None，条目拿不到：「if !self.visited_units.insert(placement) {」（`crates/singlefs-checker/src/walk.rs:1266`） 每进一版只清几张按版的小表：「fn begin_walking_a_version(&mut self) {」（`crates/singlefs-checker/src/walk.rs:973`） 所以逐版判需要给「落点 → 号集 / 记录」加缓存，或另走一遍只读不判的扫描。

| 候选 | 判法一句 | 字面度 | 合法镜像误红 | 单红 | 实现与 kb |
|---|---|---|---|---|---|
| 只最新根 | 最新根那一版判一次 | 定义不限版数，字面不排除。D8 已定项 16 第 6 条以「任何一版」说话（转述：`.claude/kb/decisions/08-核心索引结构.md:469`）；I-9.24 明写每个被判的版本各判一次 | 否 | 同 | 免缓存，一次扫描 |
| 候选集每版加记录施加出来的几版 | 每一版各判一次，按树表落点去重 | 与 I-9.24、I-9.25、I-4.8 同族；管理员回退取 R_old 的三棵树自洽（L6），候选集每一版本来就是某次发布的后态，**不会在合法历史上误红**（推的，对着第 1 节目录） | 否：L0–L7 都自洽；被抛弃的线、F 之下、窗口之外的根不在候选集里（8099 的过滤） | 同 | 版本数 ≤ 根环槽数 + 几版记录施加的；根环 12 / 24 / 48 条（S = 4 / 8 / 16，环窗口 13W / 25W / 49W 即根环槽数 3S 再加 1，转述：`.claude/kb/decisions/28-挂载期承诺量.md:29`）；共享叶用缓存；每版读 inode 树全部记录与一个 `deleted_inodes` 根，推的，没量过 |

为什么不会因为「只判最新根」漏掉写者造出来的坏版本：D8 已定项 16 第 7 条 ③ 要求每个写点截断、重开续做，每个崩溃态 checker 全绿（转述：`.claude/kb/decisions/08-核心索引结构.md:470`）。每个发布出来的版本在崩溃放量里都当过最新根（推的：崩在这一版之后、下一版之前的状态，最新根就是它）。只最新根漏的是非写者造成的坏（位腐、手改镜像）和「管理员回退到的目标版本本身是坏的」——后一种只有逐版判才看得见，而回退正是 D8 已定项 6 点名的路径。

推翻：候选集里某一版在合法历史上不满足 I-9.5 / I-4.7 / 修正式（则候选集读法误红）；或用户把这几条定成「只判现行态」。

### 5.2 F4-b　`deleted_inodes` 与 inode 树怎么配

I-9.5 原文：「| I-9.5 | 删除队列里的号仍在树里 | `deleted_inodes` 里的每个 inode 号在同一棵树的 inode 树里存在且记录的 nlink = 0（记录在回收完成前留在 inode 树里，`deleted_inodes` 只存号） | 未实现（三·3 实现；`deleted_inodes` 的形态已定在 D8（核心索引结构） 已定项 16） |」（`.claude/kb/invariants.md:292`） 「同一棵树」要有盘上的联系。`deleted_inodes` 的条目只有 8 字节 key，没有树 ID 段：「const DELETED_INODES_ENTRY_BYTES: usize = 8;」（`crates/singlefs-checker/src/walk.rs:96`） 联系只剩树表条目偏移 116 的头 ID：「const TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET: usize = 8 + 2 + 2 + 2 + 86 + 8 + 8;」（`crates/singlefs-checker/src/walk.rs:3410`） D8 已定项 8 写明 `deleted_inodes` 的头 ID 等于 inode 树的树 ID（转述：`.claude/kb/decisions/08-核心索引结构.md:254`），I-9.27 判它（`.claude/kb/invariants.md:314`，转述）。

- 经头 ID 配：字面；多头时头 ID 恰好选中一棵 inode 树（I-9.27 在多于一条种类 2 条目时放宽成「等于其中某一条」，所以头 ID 是唯一能分清的东西）。头 ID 指不到任何 inode 树时 I-9.27 已红，I-9.5 再红一次会让一份镜像同红两条：这一版 **不判**，记为判不了（见 F4-c）。
- 「同表里唯一那条种类 2 条目」的捷径：第一版（一头一棵 inode 树）与头 ID 配法结果相同，多头时不同；它不是定义，是第一版的巧合，不取。
- 同一张表里种类 9 条目多于一条（C675 欠着，没有不变量）：逐条各按自己的头 ID 配。

这一格不需要交用户。推翻：合法树表里出现头 ID 不等于任何 inode 树 ID 的 `deleted_inodes` 条目且 I-9.27 不红（则 I-9.27 与本节的前提不一致）。

### 5.3 F4-c　读不全时报什么

- 报「部分没判」的条件，kb 写得很准（转述，行带「」：`.claude/kb/invariants.md:6`）：判到的那一部分都对得上、射程里另有一部分这张镜像上判不了，说明逐条列出没判到的是什么；它不是红也不是成立。
- 代码：「PartiallyJudged(String),」（`crates/singlefs-checker/src/image.rs:39`） 两种机制的出口：违例之后先报部分没判（「} else if let Some(unjudged) = self.partially_judged.get(invariant) {」（`crates/singlefs-checker/src/image.rs:282`）），再报 `part_of_the_range_not_judged`（「} else if let Some(reason) = self.part_of_the_range_not_judged.get(invariant) {」（`crates/singlefs-checker/src/image.rs:284`）），最后才是成立（「} else if self.evaluated.get(invariant).copied().unwrap_or(0) > 0 {」（`crates/singlefs-checker/src/image.rs:286`））。两种在没有违例时都 **压过「成立」**；区别只在报告里：前者是第四种判定、带逐条说明；后者出来是「不适用」带一句理由。层 0 把两者都算进「不适用」那一格（`.claude/kb/invariants.md:6` 同一行），GPU 判器只记违例位，所以对「只红自己」与各层计数没有差别。
- 先例：I-9.28 用 `partially_judged`（用户 2026-10-04 定第四种判定）；`part_of_the_range_not_judged` 在 walk.rs 里有 12 处调用（grep 见第 10 节）。别的判法静默跳过读不出的部分是 C701 的题面，C701 写明下一轮判要不要都改（`.claude/kb/checks-owed.md:556`，转述）。

定义推不出该取哪个：两者都满足「不适用必须带理由，不许静默算成立」。凡「这一版判到的部分都对得上、另有几版判不了」的形，字面最贴的是第四种判定；取它的代价是改 `.claude/kb/invariants.md` 第 6 行末句「今天只有 I-9.28 报它」与 C701 那一行，并按 C701 的要求给判别力自证（把这一形改回静默跳过必须红）。交用户（或带 C701 那一问一起答），K3 第 4 行。

推翻：用户答 C701 时定「只有 I-9.28 用第四种」。

## 6 F5　I-3.4

### 6.1 从定义推

I-3.4 整行带「」，不整行引，转述它的三句：① 待删中 inode 没释放的单元计在记账已分配里（释放只进 defer）；② 记账标签 4 恒 0；③ 可用空间统计因此已扣除待删占用，准入式子另扣环窗口常驻项（`.claude/kb/invariants.md:148`）。决策侧：D28 已定项 1 同一句理由（转述，行带「」：`.claude/kb/decisions/28-挂载期承诺量.md:28`），D5 已定项 4 统计量表第 4 行（转述：`.claude/kb/decisions/05-快照-空间记账机制.md:119`），第三轮判决 G1-标签 4 一行（转述：`research/prompts/m3-delete-r3-main-verification.md:40`）。

三句各自在镜像上能不能判：

| 句 | 镜像上判得出吗 | 今天谁在判 | 单红 |
|---|---|---|---|
| ① 单元计在已分配里、不进 defer | 判得出，但是 I-3.1 与 I-3.11 的特例：少算进 allocated → I-3.1 红（「judgements.judge("I-3.1", allocated == Some(walked), || {」（`crates/singlefs-checker/src/walk.rs:8514`））；错记进 defer → I-3.11 红（「matches!((allocated, deferred), (Some(allocated), Some(deferred)) if deferred.checked_add(referenced_by_the_newest_root) == Some(allocated)),」（`crates/singlefs-checker/src/walk.rs:8551`）） | I-3.1、I-3.11 | 另立一个判法必被前两条蕴含，**不可能单红** |
| ② 标签 4 恒 0 | 判得出；行进了 `self.accounting`（键是 统计量 × 设备，「.insert((read_u16(row, 0), read_u32(row, 10)), read_u64(row, 22));」（`crates/singlefs-checker/src/walk.rs:1873`）），查 (4, 设备段 0xFFFF_FFFF) 即可；写侧每次发布取同一常量：「crate::admission::PENDING_DELETE_OF_THE_FIRST_VERSION.0」（`crates/singlefs-core/src/transaction.rs:7931`） 常量定义：「pub const PENDING_DELETE_OF_THE_FIRST_VERSION: BytesSummedOverAllReplicas =」（`crates/singlefs-core/src/admission.rs:196`） | 没人读（checker 的统计量常量只有 1、2、5、12，见 walk.rs:149–156） | 可单红：别的条不读这一行（推的） |
| ③ 可用已扣除 | 前半「因此」由 ① 加 I-5.2（「judgements.judge("I-5.2", matches!((free, allocated, capacity), (Some(free), Some(allocated), Some(capacity)) if free.checked_add(allocated) == Some(capacity)), || {」（`crates/singlefs-checker/src/walk.rs:8530`））推出，镜像上没有「可用」那一行；后半环窗口「只住内存、不进记账行」：「| 环窗口 | **例外**：删除各批新写的兜底，(根环槽数 3 × S + 1) × 每批上界 W（D28（挂载期承诺量） 已定项 1）；只住内存、不进记账行、每次发布按几何现算，与 checkpoint 保留池同类，不是已发布统计量 |」（`.claude/kb/decisions/05-快照-空间记账机制.md:108`） | 前半同 ①；后半镜像上无对象，归准入对拍（D28 的 C355、C375 欠账线） | 后半判不了 |

### 6.2 F5-a　落哪几句

- 「三句都落」：① 是被蕴含的重复，③ 前半同，③ 后半没有镜像对象 ⇒ 多出来的两句要么恒与前面同红、要么恒不适用，三·3 要的「坏镜像只红自己」只有 ② 做得到。**扩读**。
- 「只落标签 4 恒 0」：对定义是 **缩读**，但被缩掉的 ①③ 在镜像上本来就由 I-3.1 / I-3.11 / I-5.2 判；缩的部分要在状态列写明归谁。
- 推到：**只落 ②**，I-3.4 状态列写「① 归 I-3.1、I-3.11；③ 前半归 I-5.2 加 ①，后半是内存量、归准入对拍」。改 kb 一处：`.claude/kb/invariants.md:148`。这一格不必交用户。

推翻：能造出一份只违反 ① 而 I-3.1、I-3.11 都不红的镜像（则 ① 有独立判力，要落）。

### 6.3 F5-b　标签 4 那一行缺了判红还是不适用

- 前提：写侧每次发布都写这一行（`PoolWideAccountingRow::ALL` 三行里的待删占用，装行处：「let mut entries: Vec<AccountingEntry> = PoolWideAccountingRow::ALL」（`crates/singlefs-core/src/transaction.rs:4777`））；D5 已定项 8 射程要求这四项统计量 day-1 各写一行，为的是守住空与 0 不是一回事这道防线（转述，行带「」：`.claude/kb/decisions/05-快照-空间记账机制.md:216`）；现行值的读法：「- 第 12 项只为可写头维护，随每次发布重写（已定项 2：每条根的记账树只装这条根那一代的行，每次发布把这一代的行全写一遍），只读树（快照）没有这一行；它带树维，所以树 ID 段仍然留在 key 里。读一个统计量的现行值 = 取该维度元组下代号最大的那一行，空表示从未维护过、不是 0。」（`.claude/kb/decisions/05-快照-空间记账机制.md:134`）
- 先例分两派：day-1 的逐盘行缺了一律判红（I-3.1 要 `Some(walked)`：8514；I-5.2 要三个 `Some`：8530）；只为可写头维护的水位行缺了报不适用：「"最新的根下面没有记账树、或记账里没有 inode 号水位那一行",」（`crates/singlefs-checker/src/walk.rs:8356`） 标签 4 是全池行、day-1 写、每版都写，属前一派。
- 判红是 **扩读**：「恒 0」说的是值，「行在」是另一条性质（今天没有任何不变量判一棵记账树的行集），挂在 I-3.4 名下它成了这一行在场的唯一把关人。不适用是 **缩读**：行不在时 I-3.4 无话可说。
- 推到判红（与 day-1 先例一致），但这是政策格，翻面观测和代价见 K3 第 5 行。
- 推翻：合法镜像上出现记账树走全而缺这一行的版本（则判红误红，改不适用）。

### 6.4 F5-c　判哪几版

- 每一版的记账树各装自己那一代的行，已有循环对走全了的树逐版判 I-3.13：「let accounting_version_txg = self.version_being_walked.checkpoint_txg();」（`crates/singlefs-checker/src/walk.rs:1856`） 同一处加标签 4 零额外读盘；共享树表的几版（零单元发布照抄树表）只判一次，内容相同。
- 只最新根：沿用 I-3.1 / I-3.11 / I-5.2 的 `accounting` 快照（walk.rs:8026）。那三条必须与走读的范围对齐才这么做；标签 4 比的是常量，没有对齐的要求。
- 回退那次发布的记账统计量由发布路径现算、不从 R_old 载入（D5 已定项 4 射程，转述：`.claude/kb/decisions/05-快照-空间记账机制.md:137`）⇒ 旧版本里的坏行不会传到将来，逐版判多出来的只是「看见过去的写者错」，而崩溃放量里那一版当最新根时本来就判过。
- 推到：逐版（与 I-3.13 同处、零成本、与 I-9.25 同族），但价值有限；政策格，K3 第 6 行。
- 推翻：发现某条发布路径把旧版本的标签 4 行原样继承到新版本（则旧行里的坏值会传下去，逐版就有独立价值，只最新根反而不够）。

## 7 K3　取不下来的格：岔路单行（候选 / 翻面观测 / 够判条件 / 代价）

代价里「推的，没量过」是读代码推的，没造镜像、没起扫描。版本数上界：根环槽数 3S，S = 4 / 8 / 16 即 12 / 24 / 48 条（D28 已定项 1 环窗口那一行，第 5.1 节引过），再加几版只由记录施加出来的。

| 候选（各自定义） | 翻面观测 | 够判条件 | 代价 |
|---|---|---|---|
| **1　F1　I-4.7 的「处于删除中」**。A = nlink = 0 的记录：号在 `deleted_inodes` 里，或该号 extent 上段叶没有数据条目；B = 号在 extent 上段叶有数据条目（LowerSegmentRoot 或 InlineDataUnit）而 inode 树没有 nlink ≥ 1 的记录：号必须在 `deleted_inodes` 里（孤儿记红）；丙 = A ∪ B | ① I-9.5 反方向（C708）进不进 I-9.5：进 ⇒ A 的坏镜像必同红 I-9.5（红集嵌套，逻辑推出，在 harness 上造一份 A 镜像读红集即坐实），A 当不了「只红自己」，翻向 B 或丙；不进 ⇒ A 字面且可单红，C708 仍欠着。② 合法历史（放开文件单元数、批大小、崩在哪批、哪一版、回退到哪一版）里出现一版「有单元、无活记录、不在队列」⇒ B、丙误红，翻回 A | 用户答 ①；攻方放开扫描对 ② 零命中（一次抽样，不支撑结论，再抽一次） | A：读 nlink（记录偏移 36）与每版队列号集、inode 记录集。B 另加每版 extent 上段叶「有数据」号集（不下下段）。丙 = 两者之和。主走读对共享单元只走一遍，需缓存或另走只读扫描。扫描次数见 F4-a 行。改 `.claude/kb/invariants.md:183`（写明前件）。推的，没量过 |
| **2　F2　I-4.4 第一版判什么**。甲 = 每版 `deleted_inodes` 条目宽 = 8 且 logged_ops 条目根指针全零；乙 = 第一版报不适用并写理由；丙 = nlink = 0 记录的 size 与单元数的关系（与 F3 乙1 同一谓词）。零轮：丁 = 丙的宿主放 I-4.4、I-4.6 取甲；戊 = 甲的 logged_ops 一半改报部分没判 | 用户是否接受 I-4.4 第一版恒不适用（出口 1 的「实现」二字）：接受 ⇒ 乙；不接受 ⇒ 甲或丙。另：第一个写 logged_ops 的写者（整理、截断）若在本里程碑内落地 ⇒ 甲的零根一半在合法镜像上红，翻离甲 | 用户答出口 1 措辞；不接受时再答甲与丙取一（别让 I-4.4、I-4.6 判同一个谓词） | 乙：0 行判定代码，改 `.claude/kb/invariants.md:179`、`.claude/kb/milestone/03-third-txn.md:17`、`.claude/kb/milestone/03-third-txn.md:85` 三处。甲：（1）零新增判力（I-1.10 同一谓词），（2）每版读树表种类 12 条目的根指针，整理落地那天要删。丙：同 F3 乙1，少一个独立谓词。GPU 判器同批（排期问题，不在本轮） |
| **3　F3　I-4.6 判什么**。甲 = 有未完成意图的版本上其余已实现条都不红（字面，含另四条兄弟条）；甲' = 同，但「其余」不含这五条里另四条；乙1 = nlink = 0 记录的号：有数据者单元数 = max(1, ⌈size ÷ 32634⌉)，无数据者 size = 0 且单元数 0；乙2 = 同式对每条记录；丙 = 甲或甲' 加乙1 | 用户在 182 行字面「其余全部」与三·3 验收「每条只红自己」之间取哪个：取字面 ⇒ 甲，I-9.5 至少一个方向的坏镜像同红 I-4.6（4.4 节）；取验收 ⇒ 甲' 或只做乙1。乙部分：放开扫描里出现 nlink = 0 记录的单元数偏离修正式一次 ⇒ 乙1 误红，式子重写 | 用户答字面对验收；乙1 的式子先有 kb 条款（今天只在 `crates/singlefs-core/src/write_request_split.rs:61` 与恢复路径里） | 乙1：每版每个 nlink = 0 的号取单元数（LowerSegmentRoot 要下到下段叶数记录，`extent_tree_without_judging`（walk.rs:4011）要加按号计数的变体），净荷容量 checker 自有一份。甲 / 甲'：`check_pool_image` 末尾一个汇总，按「有没有违例」读各条（`Judgements` 在 into_report 才定结论），不增读盘。乙2 另立 I-9.x 要再加一条。改 182 行与三·3 验收。推的，没量过 |
| **4　F4-c　读不全时报什么**。`partially_judged`（第四种判定，逐条列没判到的版本）；`part_of_the_range_not_judged`（没违例时出「不适用」带一句理由） | C701 那一问的答案：别的判法是否也改报部分没判。答「也改」⇒ 新条跟着用第四种；答「只 I-9.28」⇒ 新条与 I-3.5 等一致 | 用户答 C701（`.claude/kb/checks-owed.md:556`）；不影响任何坏镜像与层 0 计数 | 第四种：改 `.claude/kb/invariants.md:6` 末句与 C701 行，按 C701 要求加判别力自证（把这一形改回静默跳过必须红，多一行变异）。另一种：零 kb 改动。GPU 侧都是 0 |
| **5　F5-b　标签 4 缺行**。判红 = 记账树走全而没有 (4, 0, 0xFFFF_FFFF) 那一行即违例；不适用 = 该版 I-3.4 不评估，理由写「没有这一行」 | 合法镜像上出现记账树走全而缺这一行的版本 ⇒ 判红误红，翻向不适用。读代码推：每次发布都写（`PoolWideAccountingRow::ALL`），应无 | 攻方放开扫描零命中，且 `accounting_entries_of_this_publish` 全部调用点读过 | 都挂已有循环，零额外读盘。判红让 I-3.4 兼任「行在场」的把关人（扩读）；不适用留一个洞。推的 |
| **6　F5-c　标签 4 判几版**。只最新根的记账；每一版走全了的记账树各判 | 旧版本的标签 4 行有没有可能被后续发布继承：D5 已定项 4 射程写回退那次发布现算、不从 R_old 载入 ⇒ 不继承；发现任一路径继承旧行 ⇒ 逐版才有独立价值 | 读 `accounting_entries_of_this_publish` 全部调用点确认不继承 | 最新根：复用 `accounting` 快照，零新增。逐版：挂在 I-3.13 同一循环（walk.rs:1857 一带），零额外读盘，共享树表的几版只判一次 |
| **7　F4-a　判几版**。只最新根；候选集每版加记录施加出来的几版 | 候选集里有合法的一版不满足 I-9.5 / I-4.7 / 修正式 ⇒ 翻向只最新根；或用户定这几条只判现行态 | 攻方放开扫描对「合法候选版是否都自洽」零命中 | 最新根：一次扫描，免缓存。候选集：版本数次（≤ 12 / 24 / 48 + 几版）扫描，共享叶用缓存（键 设备 × 槽 × 校验和），每版读 inode 树全部记录与一个 `deleted_inodes` 根；只有队列非空或有 nlink = 0 记录的版本才需要再下 extent 扫描（推的）。没量过。正推已推到候选集，这一行供复核 |

## 8 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| F1 | 取不下来，交用户 | A 照 D8 操作定义是字面，但红集嵌在 I-9.5 反方向里单红不了；B / 丙 能单红但是扩读、分句二落空。前提是 I-9.5 反方向进不进 |
| F2 | 推到乙，必交用户 | 第一版三处载体没有可装增量的字段；甲（1）重复 I-1.10，甲（2）是扩读且整理落地就过期；丙与 F3 乙1 同谓词；乙要改出口 1 |
| F3 | 乙1（修正式）是核；甲 / 甲' 取舍交用户 | 式子 ⌈size ÷ 净荷容量⌉ 在可达盘面 L5（清空过的文件被 unlink）上误红，要改 max(1, …) 加无数据支；甲字面使 I-9.5 至少一个方向同红 I-4.6 |
| F4-a | 推到候选集加记录施加的版本 | 与 I-9.24、I-9.25 同族，合法历史上不误红；只最新根漏的是非写者造成的坏和回退目标本身坏 |
| F4-b | 推到经头 ID 配，配不上记判不了 | 字面；再红一次会和 I-9.27 双红 |
| F4-c | 取不下来（政策格） | 两种都不违反「不适用必须带理由」；带 C701 一起答 |
| F5-a | 推到只落标签 4 恒 0 | ① 被 I-3.1、I-3.11 蕴含，③ 前半被 I-5.2 加 ① 蕴含，后半是内存量；状态列写明归谁 |
| F5-b | 推到判红（政策格） | day-1 全池行，先例判红；但让 I-3.4 兼管行在场 |
| F5-c | 推到逐版（政策格） | 零成本、与 I-3.13 同处；价值有限，回退不继承旧行 |

## 9 与前置调查不同的地方

- F3 乙的式子：前置调查与正文写 ⌈size ÷ 净荷容量⌉，我查到实现的除法带下界 1 且有测试钉住，式子要改（4.2 节）。
- F3 甲：前置调查说甲永远伴随别条红；我多推了一步——触发条件取 T1 / T2 / T3 都挡不住 I-9.5 两个方向，甲字面使兄弟条的验收也落空（4.4 节）。
- F1：前置调查指出 A 与 I-9.5 反方向嵌套，我同意；另补一点：I-4.7 的分句二放过 C708 举的那一形（空 inode、nlink = 0、不在队列），所以 C708 收不进 I-4.7（2.3 节）。
- F4-c：前置调查建议 `part_of_the_range_not_judged`（零 kb 改动）；我推不出该取哪个，两种都合规，当政策格交出去（5.3 节）。
- F2、F5-a、F4-b：与前置调查同向，各自重查过出处。

## 10 现查命令与原样输出

```
$ cd /tmp/claude-1000/m3-delete-checker-r1/tree && sha256sum -c /home/fy5090/code/singlefs/research/prompts/m3-delete-checker-r1-snapshot/crates-sha256.txt | grep -vc ': OK$'
0
$ ... | wc -l  （校验文件总行数）
8
$ grep -rn nlink /tmp/claude-1000/m3-delete-checker-r1/tree/crates/singlefs-checker/src | wc -l
0
$ grep -c 'judgements.part_of_the_range_not_judged(' crates/singlefs-checker/src/walk.rs  （冻结副本）
12
$ grep -rn '声明长度为 0\|声明长度 0 的\|长度 0 时是 1\|空内容也写一个' .claude/kb --include=*.md | wc -l
0
$ grep -c '^| I-.*已实现' .claude/kb/invariants.md
69
$ grep -nF 'pub const IMPLEMENTED_INVARIANTS' crates/singlefs-checker/src/image.rs  （冻结副本）
83:pub const IMPLEMENTED_INVARIANTS: [&str; 69] = [
```

kb 里 `.claude/kb/milestone/03-third-txn.md` 在快照之后改过（`sha256sum -c research/prompts/m3-delete-checker-r1-snapshot/kb-sha256.txt` 该行 FAILED，其余四份 OK）；这份文件我只引行号作指向，行号是现查当前文件的。快照的 `crates-sha256.txt` 只列 8 份文件（checker 四份、GPU 判器、`transaction.rs`、`unlink.rs`、`records.rs`）；`recovery.rs`、`mount.rs`、`admission.rs`、`deleted_inodes.rs`、`write_request_split.rs` 与 harness 测试没有哈希，我引的是冻结副本里的版本。

## 11 没做什么

- 没造任何镜像、没编译、没起 harness 或别的测试：本轮格是 K1、K3（推读与代价），造误红与漏判的镜像是攻方腿 K2 的格。凡写「推的」都未跑。
- 不替主 agent 采纳或判决。F2 乙、F3 甲 / 甲'、F1 的 I-9.5 反方向，按正文第五节或本报告第 7 节归用户。
- 没碰标签 6 的现算（D8 已定项 16 第 7 条 ④ 也点了标签 6，不在五条不变量里）、墓碑与谱系系列 I-9.17–I-9.24、GPU 判器覆盖表、`IMPLEMENTED_INVARIANTS` 追加次序（正文划在这一轮之外）。
- 「recovery 对 logged_ops 非零根拒成『树的种类没登记』」只读了代码路径（`recovery.rs:4899` 的 `ok_or`），没起恢复。
- 没量判几版的扫描次数、没量缓存命中；版本数上界取自 D28 的环窗口系数（S = 4 / 8 / 16）。
- 没查 `crates/mutations.tsv` 的行格式，K3 的代价里只写「各一行」。
- 没建模型目录（没有脚本）。
- 提的新候选（甲'、丁、戊、乙2 另立 I-9.x）都是零轮，没被攻过。
