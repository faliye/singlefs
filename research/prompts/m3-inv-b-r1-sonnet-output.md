# m3-inv-b-r1 正推腿（Sonnet）报告：B1–B7（2026-09-30）

立场：正推。从已定条款与 `crates/` 今天的实现推「应该是什么」，再与底稿 `research/prompts/c395-invariants-draft.md` 逐格比。
读的是 HEAD fcc4d340 的 `crates/`（与冻结副本 `/tmp/claude-1000/m3-ring-r1/tree/crates/` 逐字节相同：`diff -rq crates <冻结副本>/crates` 无输出；`git status --short crates | wc -l` 输出 `0`）。
没编译、没跑 checker、没造任何镜像：坏镜像都是设计，「推得只红它」是照代码读出来的，写作「推的」。跑了两个纯 Python 小模型（B3、B7），落在 `research/prompts/m3-inv-b-r1-sonnet-model/`。
引 kb 条款写 kb 文件自己的行号，都是这一次 `grep -nF` 现取的。

## 先说四条对全轮有用的发现

1. **底稿主表第 3 行的下界那一半在合法镜像上误红，出局**（B3）：管理员回退「复活」的单元诞生代号不变，再落一次抬 F，候选集里就会有一条比回退发布早、又不引用它的根。抽象模型 16125 条合法历史里含回退的 379 条踩中，不含回退的 0 条。上界那一半站得住，但它不是 I-3.5 ② 的条款，是「单元先于根」推出来的，建议另立。
2. **B2 该按「末条标志」认记录，不按「计数器最大」**：恢复施加的就是带末条标志那一条的新根段（`crates/singlefs-core/src/recovery.rs:3200`），条款只钉这一条；今天实现让一次发布的每条记录都带同一份新根段，那一句只写在代码注释里，条款没有。
3. **码 2 那一类在 C289 定之前「跳过并写明」，不能用 `part_of_the_range_not_judged`**：它让每份干净镜像整条报不适用（`crates/singlefs-checker/src/image.rs:159` 与 `into_report`），I-1.11 的先例是写进状态列、不登记（B1）。
4. **B7 收窄后的守卫在模型里零误红**（43302 个情形），今天的整条不适用守卫只判 31 个合法情形、抓 186 个非法情形，收窄后判 1562、抓 9372（B7）。

## B1　指针出生树与实例表指针（C294）

### 应该是什么

指针头部偏移 34 的出生树、42 的出生 txg，78 起的实例代号与尾段，按 D19 已定项 8「携带被指单元映射 key 除类标签外的各段」，在被指单元头里各有一份。今天 checker 的覆盖是四块：

| 那一段 | 被指的类 | 今天判不判 | 依据 |
|---|---|---|---|
| 出生 txg、实例代号、尾段 | 码 1 / 码 2 / 码 3（走读到的） | 判，挂 I-1.2 遍历方向 | 「fn judge_birth_identity_of_a_referenced_unit(」（`crates/singlefs-checker/src/walk.rs:482`），调用点 825、1591、1682、1979 |
| 出生树（偏移 34） | inode 内部→叶容器 | 判，由 I-9.2 罩住 | 「&& child.birth_tree == identity.0;」（`crates/singlefs-checker/src/walk.rs:1512`）加头四段与身份引用相等 |
| 出生树（偏移 34） | 中央映射树根 | 只把它当 I-1.3 的期望树 | 「tree: parse_node_pointer(mapping_root_pointer).birth_tree,」（`crates/singlefs-checker/src/walk.rs:958`） |
| 出生树，其余全部 | 树表条目根指针、码 2 子指针、extent 数据指针、实例表指针 | **不判** | 全仓 `birth_tree` 命中里没有别的与头比的用法（事实表第七节，我又跑了 `grep -n birth_tree` 核 `walk.rs` 的命中：390、958、1076–1086、1512、1522–1527、2921） |
| 出生 txg / 实例 / 尾段 | 根记录偏移 170 的实例表指针与链指针 | **不判** | 「打包记录类型 4 的实例表单元不进这两格」（`crates/singlefs-checker/src/walk.rs:480`） |

条款能推到哪一步：

- 码 3 与码 1：「出生树取类身份段里的出生树（码 3 偏移 43），出生 txg 取诞生代号（码 3 偏移 73；码 1、码 2 同取诞生代号）」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:143`）；被快照共享的码 1 单元头里的树 ID 是出生树：「打包记录单元（码 3）与被快照共享的数据单元头里的树 ID 是出生树」（`.claude/kb/invariants.md:26`）；不共享时它同时是引用树，两种读法同值。所以码 1、码 3 的指针偏移 34 = 头偏移 43，条款推得出（码 1 的字段落点偏移 43 取自 I-1.11 状态列的读法，D19 那句只点了码 3）。
- 码 2：头里取哪个字段没定，I-1.11 的先例是「码 2 的出生树随 C289（码 2 的出生树取哪个字段没写） 不判」（`.claude/kb/invariants.md:34`）。现有坏镜像已把码 2 映射根那一格钉成只红 I-1.3：映射根指针的出生树改成 16、头仍说 15（`crates/singlefs-harness/tests/checker_known_bad_images.rs:2953` 起的用例）。码 2 一判，它就多红一条。
- 实例表指针：「实例代号 4 + 出生序号 4 照填被指单元头里的值、不拿去查映射」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:182`），出生 txg 与出生树由已定项 7 的指针头部定；实例表单元身份四元组是「(出生树 0, 类型 4, 容器号 = 片序号从 0 起, 出生代 0)」（`.claude/kb/decisions/18-块里携带什么信息.md:304`）。

### 措辞与判法（建议）

- **立一条新的**（建议号 I-1.12，下一个没用过的号，`grep -rnF 'I-1.12' .claude/kb` 零命中）：「任一被走读的指针，偏移 34 的出生树等于被指单元头里的出生树：码 1、码 3 取头偏移 43；码 2 的出生树取哪个字段随 C289 定，不判。」不挂 I-1.2：I-1.2 的名字与谓词说的是写序已发布，出生树不进谓词；且挂上去后，出生树坏镜像会在 I-1.2 的既有「只红它」用例里多一个红点。
- **实例表指针的出生 txg / 实例 / 尾段挂 I-1.2 遍历方向**：把状态列里「按本行的例外不进这一条」（`.claude/kb/invariants.md:25`）收窄成「例外只免谓词那一半」，条款本行的例外句原文就只说不走谓词：「例外：打包记录类型 4 的实例表单元不走这条谓词，它的现行版本由根记录持有的物理指针唯一确定。」（`.claude/kb/invariants.md:25`）。I-4.2 那一行同样的话是谓词半，不用改。
- **inode 内部→叶容器那一类不重判**：I-9.2 已经把指针、身份引用、头三方钉成相等，重判会让 I-9.2 既有坏镜像多红一格。
- **码 2 的跳过不用 `part_of_the_range_not_judged`**：它的注释说判了的那一部分成立不等于整条成立、没有违例时整条改报不适用（`crates/singlefs-checker/src/image.rs:157`，取值在 `into_report` 里），每份镜像都有码 2 指针，登记它等于这条新不变量永远报不适用。写进状态列就是了。

### checker 怎么判（对 `crates/` 的改动）

1. `crates/singlefs-checker/src/walk.rs:482` 的函数拆成两半：指针身份与头相等（出生 txg、实例、尾段）和已发布谓词。前一半加一个入参说要不要判出生树；谓词半不动。
2. 出生树：在数据单元调用点（`walk.rs:1979`）与实例表链（`walk_instance_table_chain`，`walk.rs:1780`，读到 `unit` 之后、`judge_packed_container` 之后）判；分类直接复用今天就有的 `birth_tree_in_the_header_as_the_mapping_key_names_it`（`walk.rs:390`）：`AtOffsetFortyThree` 那一支判，`FieldStillOpenUnderC289` 那一支什么都不做。
3. 实例表：链上每一片、根记录偏移 170 那一条，用头里的 73、93、103 与指针的 42、78、82 比。core 两边取自同一变量，合法镜像上恒等：`build_instance_table_chain` 里指针与单元同用 `birth_sequence`（`crates/singlefs-core/src/transaction.rs:1595` 一带），mkfs 那一片同用 `instance_table_sequence`（`crates/singlefs-core/src/make_filesystem.rs:548`）。

### 坏镜像（设计，没造没跑）

- 数据指针出生树：新池新建文件那份，`EXTENT_ROOT`（槽 50240，`checker_known_bad_images.rs:101`）里第 0 条 extent 记录的数据指针出生树，偏移 = 163 + 24 + 1 + 34 = 222（从「const EXTENT_RECORD_DATA_POINTER_BIRTH_TXG_OFFSET: usize = 163 + 24 + 1 + 42;」（`crates/singlefs-harness/tests/checker_known_bad_images.rs:588`）减 8），11 改 16（池里发过、低于水位的号，理由同 2953 那份），`mutate_unit(..., true)` 沿引用链重封。I-1.3 比头树 ID 与 extent 树（`walk.rs:1980` 起），I-1.11 比映射条目自己的字节，都不读这 8 字节 ⇒ 推得只红新条。
- 实例表指针出生 txg：txg 3 那条根（`NEW_POOL_FILE_CREATION_ROOT_SLOT`，`checker_known_bad_images.rs:1624`）偏移 170 + 42 = 212 的 0 改 1，重封根记录校验和（偏移 138）⇒ 推得只红 I-1.2；出生树改在 170 + 34 = 204 的 0 改 16 ⇒ 只红新条。
- 变异（判定恒真）：把新条的 `judge("I-1.12", header_birth_tree == pointer.birth_tree, …)` 换成 `true`；另一条「实例表链不调用」，靠上面第二份镜像红。

## B2　journal 记录新根段与根（底稿主表第 2 行）

### 应该是什么

- 施加一条记录 = 换四个字段：「施加一条记录 = 把所选根的这四个字段换成记录里的，不需要树语义。」（`.claude/kb/decisions/23-journal的角色与格式.md:443`）；新根段 188 字节 = 树表指针 86 + 映射根指针 86 + 水位 8 + F 8，段起点在记录偏移 99（`crates/singlefs-checker/src/lib.rs:703` 的「const JOURNAL_NEW_ROOT_SEGMENT_OFFSET: usize = 99;」）。
- 恢复施加的是带末条标志那一条：「if !record_ends_its_publish(record) {」（`crates/singlefs-core/src/recovery.rs:3200`）之后才施加，注释写「施加末条那一份」（`crates/singlefs-core/src/recovery.rs:3209`）。锚点也是标志：「不取读得出的同 txg 记录里 jsn 最大的那条」（`.claude/kb/decisions/23-journal的角色与格式.md:367`，用户 2026-09-24 读法乙）。
- 一次发布切成几条记录时，前几条的新根段该写什么，条款没有：D23 已定项 17 只说共享的提交内生块「只在最后一条记录里点名」（`.claude/kb/decisions/23-journal的角色与格式.md:475`）。今天实现让每条都带同一份：「每条都带整次发布的新根段（已定项 15）。」（`crates/singlefs-core/src/transaction.rs:6728`），这一句只在代码注释里，D23 已定项 15 没有。

### 与底稿比

- **冲突（认哪条记录）**：底稿写「且 J 是同一块盘上同 (实例代号, checkpoint_txg) 的记录里计数器最大的那一条」（`research/prompts/c395-invariants-draft.md:20`），条款读法乙写「不取读得出的同 txg 记录里 jsn 最大的那条」（`.claude/kb/decisions/23-journal的角色与格式.md:367`）。两句在一次发布的末条读不出、而根在案时给出不同的对象：按计数器最大会拿前几条去比，而条款没有规定前几条的新根段。
- **冲突（偏移，过时）**：底稿「树表单元指针 95、映射根指针 181、树 ID 水位 267」「记录偏移 275」「F 在 307 字节头内」（`research/prompts/c395-invariants-draft.md:20`），今天是 99 / 185 / 271 / 279、头 311：「pub const JOURNAL_HEADER_BYTES: u64 = 311;」（`crates/singlefs-format/src/lib.rs:200`）。根记录一侧不变：树表指针 36、水位 122、F 130、映射根 256（`.claude/kb/decisions/22-单元原子性怎么合成.md` 已定项 7 的偏移行）。
- 一致：「一次发布切成多条记录时，前几条的新根段该写什么没有条款」（`research/prompts/c395-invariants-draft.md:20`）今天仍对；「根槽没落盘、只由记录施加出来的那一版，它的 F 没有根可对，这一条不管」也对。

### 措辞（建议号 I-8.10）

「某块盘 journal 环里自证过的记录 J，带末条标志（记录标志位 0 = 1），且这块盘上同 (实例代号, checkpoint_txg) 带末条标志的记录只有这一条；根环里任一自证过、实例代号与 checkpoint_txg 都与 J 相同的根 R，J 的新根段四段与 R 的同名四段逐字节相同：段[0..86] = R[36..122]，段[86..172] = R[256..342]，段[172..180] = R[122..130]，段[180..188] = R[130..138]。」

- 同 (实例, txg) 有两条以上不同内容的根：不判，归 C88（`.claude/kb/checks-owed.md:88`）；末条标志不唯一：归 I-8.9，不判。
- 只判带标志那一条，前几条不判：条款没写前几条该带什么，今天代码带同一份是实现选择，不进不变量。若要进，是条款变更（D23 已定项 15 加一句「一次发布的每条记录带同一份新根段」），交用户，不是这一轮该采的。
- 挂在 I-8 类新立，不挂 I-8.9：I-8.9 判序号与标志，这条判内容。

### checker 怎么判

`ScannedJournalRecord` 已带 `new_root_segment`、`record_flags_byte`、`checkpoint_txg`、实例（`crates/singlefs-checker/src/walk.rs:4893`、4899、4907），`scanned_journal_records_by_device`（`walk.rs:5066`）逐盘交回；根字节在 `RootView::record_bytes`（`crates/singlefs-checker/src/lib.rs:307` 起），`valid_roots`（`crates/singlefs-checker/src/image.rs:613`）交 (区域, 槽, 根)。缺的只是判定：`new_rollback_floor`、`new_tree_identifier_watermark` 解出来没人用（`grep -n 'new_rollback_floor\|new_tree_identifier_watermark' crates/singlefs-checker/src/*.rs` 只命中 `lib.rs:750`、751、833、834）。

合法镜像上恒等，因为三处装记录与根的地方用的是同一份计划值：零单元发布 `new_rollback_floor: plan.rollback_floor,`（`crates/singlefs-core/src/transaction.rs:1300`）对 `rollback_floor: plan.rollback_floor,`（`crates/singlefs-core/src/transaction.rs:1313`），写行发布 1822 对 1842，带文件发布 6770 对 6784；卸载与准入抬 F 那一串走同一条 `publish_version_with_unmount_marker`，F 也取自计划。

### 坏镜像（设计，没造没跑）

新池新建文件那份，计数器 3 那条记录（txg 3，环里最后一条：新池新建文件写 w1、w4、t9 三条，D23 已定项 17 的「w1、w4、t9 三条记录」），两盘各一份，记录偏移 279 起的 F 0 改 1，重封头校验和（偏移 46，罩整条 4096；载荷从 311 起，F 在头内，载荷 CRC 不动）。最后一条没有下一条的反向链 ⇒ I-8.6 不动；`new_rollback_floor` 别处不读 ⇒ 推得只红新条。**改根一侧不行**：F 改在根上会让 I-7.12 一起红（「不低于这块盘根槽里每一条自证过的根带的 F」，`.claude/kb/invariants.md:63`），所以坏镜像取记录一侧。变异（判定恒真）：新条里逐字节相等的比较结果换成 `true`。

## B3　诞生代号的下界与点名项（底稿主表第 3、4 行）

### 3a　单元头诞生代号：上界成立，下界出局

条款：I-3.5 ② 的原文是「对任一块 b 与任一根 R（快照根或活头根）」枚举后的 R_first / R_prev（`.claude/kb/invariants.md:137`），它说的是快照根与活头，不是根环里的候选根。D5 的区间定义：「块 b 被快照 S 引用 ⟺ `birth(b) ≤ S.txg < death(b)`」（`.claude/kb/decisions/05-快照-空间记账机制.md:309`）。把候选根当快照 S 用，底稿自己标了「推的」，条款不支持这一步。

**上界能由别处推出来**：`birth(b)` 是「块被发布的那个 checkpoint 号」（`.claude/kb/decisions/05-快照-空间记账机制.md:313`），持久顺序是单元先于根：「一次发布的持久顺序恒为 COW 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）→ 系统配置槽 → 屏障」（`.claude/kb/decisions/16-发布语义.md:179`）。所以根 R 引用的单元，诞生代号 ≤ R 的 checkpoint_txg，与 I-3.5 无关。实例切换照旧的单元：头里是在飞 checkpoint 的号 X，重发那次的 txg Y ≥ max(环里全部根 txg, 全部记录 txg) + 1（`.claude/kb/decisions/23-journal的角色与格式.md:369`，「⚠️ **checkpoint_txg 也一样**」那一句）≥ X，上界照样成立（切换重发今天没有实现，`.claude/kb/decisions/23-journal的角色与格式.md:409` 写明留给后面的里程碑，所以这一格只有条款推演、没有镜像）。

**下界在合法镜像上误红**：回退是挂着时的一次向前发布，「复活」把目标那一版里被现行账记成已释放的用户可见单元改回已分配、分配代写回目标那一版账里的（`.claude/kb/decisions/23-journal的角色与格式.md:382`），单元头一个字节不动，诞生代号仍是老的。之后抬 F 落在回退目标与回退发布之间，候选集里就有一条 txg 大于该单元诞生代号、又不引用它的根，`R_prev` 落在它上面。史例（抬 F 取第 4 新的非空根，`.claude/kb/invariants.md:60` 的准入抬 F 上限）：根 A(3) 是第一个文件版本，B(4) C(5) D(6) 各覆盖写，E(7) 回退到 A；非空根 5 条（txg 是抽象编号，真实历史里写行与暖机的空发布会占号），第 4 新的是 B(4)，F 抬到 4，A 出候选集；A 的单元 u（诞生代号 3）被 E 引用，R_first = E(7)、R_prev = D(6)，`3 > 6` 不成立 ⇒ 判红，而这份镜像合法。
- 模型复跑：`python3 research/prompts/m3-inv-b-r1-sonnet-model/b3_bounds_model.py` 输出末尾 `含回退：下界违例 379 条，上界违例 0 条` 与 `不含回退：下界违例 0 条，上界违例 0 条`（合法历史 16125 条，操作串长 ≤ 6；模型只有根、单元、回退、抬 F，不含实例表与被抛弃时间线，是抽象的，不是实现）；最短反例 `('W','W','W',('R',0),('F',4))`。
- 落到实现：历史生成器已有回退与抬 F 两个操作（`crates/singlefs-harness/src/history.rs:359`、361），每一步跑 checker（`crates/singlefs-harness/tests/random_histories.rs:425` 的 `PerStepChecker::Run`），所以按上面史例摆五次覆盖写、一次 `RollBackWhileMounted(RingRoot)`、一次 `RaiseRollbackFloor` 就能造；我没跑。
- **I-3.5 自己的 ① 连续无洞同样被回退复活破坏**（A 与 E 引用 u、B..D 不引用，Reach(u) 有洞）。这是条款之间的冲突，推的、没走三方，交主 agent：I-3.5 状态列该写明射程收在快照与活头，回退复活之后候选集里的 Reach 允许有洞。

结论：底稿「下界 `birth(b) > R_prev.txg`」出局（五判据第一条：一条措辞在任一合法镜像上判红 ⇒ 出局）；修不了，因为复活的单元与没复活的单元在盘上分不开（分配代、头、指针都相同）。上界保留，**另立**，建议号 I-4.9：「回退候选集里不是最新的每一条根 R，它引用的单元（按落点认，读落点上单元头里的诞生代号，头校验和过）诞生代号 ≤ R 的 checkpoint_txg。」最新根那一条归 I-4.2，剔掉它，否则 I-4.2 那份既有坏镜像（`crates/singlefs-harness/tests/checker_known_bad_images.rs:1162`，数据单元头与指针一起抬到 4、最新根 txg 3）会多红一条。

I-4.9 的实现与坏镜像：

- 判：在 `judge_release_generation_and_tree_table_birth`（`crates/singlefs-checker/src/walk.rs:3395`）里，候选集每条非最新根已经各走一遍 `references_of_root`（`walk.rs:2793`），对它的每个落点读头，用 `birth_txg_in_a_usable_unit_header`（`crates/singlefs-checker/src/walk.rs:3523`）取诞生代号与 `checkpoint_txg` 比。代价两处：① 深度今天由最新根有没有已释放记录定（`walk.rs:3421` 的「let depth = if newest」），新条要求每条候选根都走到叶，层 0 每个崩溃状态都多走一遍；② `RootReferences.placements` 只存 (盘, 槽)（`walk.rs:2722`），没有校验和，同 I-3.10 只认头校验和：一个已被合法复用的落点（I-7.4 红的那种）会让两条一起红，这是真违例的重复报告，不是误红。候选集只剩一条根时同 I-3.9 一样报不适用（`walk.rs:3404` 一带）。
- 坏镜像（覆盖写之后那份，A 的数据单元 `DATA_UNIT`＝槽 50180，B 之后只有 A 的根引用它）：与既有 I-4.2 那份同形（`crates/singlefs-harness/tests/checker_known_bad_images.rs:1162`）四处一起改：单元头偏移 75 的 3 改 4；`EXTENT_ROOT` 里数据指针的出生 txg 3 改 4；A 的映射叶里那条码 1 key 的出生 txg 用 `rewrite_the_data_unit_mapping_key_birth_txg(bytes, 3, 4)`（`checker_known_bad_images.rs:605`）；A 的两片分配记录叶里未释放记录的分配代 3 改 4（`mutate_allocation_leaves` 加 `rewrite_unreleased_allocation_generation`）。**底稿这一份少改了映射 key**：I-1.11 是 2026-09-27 加的，映射 key 不跟着改会多红 I-1.11，底稿 2026-09-24 写时还没有。I-4.2 按 4 ≤ 最新根 4 成立，I-1.2、I-3.10、I-1.11 各自对得上，A（txg 3）是非最新根、3 < 4 ⇒ 推得只红 I-4.9。变异（判定恒真）：`birth <= root_txg` 的比较换成 `true`。
- 底稿另一份「下界」坏镜像（B 的数据单元 4 改 3）随下界出局，不造。

### 3b　点名项与被指单元头（底稿主表第 4 行）

- 条款推得出：点名项的字节 [28..55) 就是映射 key（类标签 1 + 出生树 8 + 出生 txg 8 + 尾段 10，与 27 字节映射 key 同形，`NamedEntryView::mapping_key` 在 `crates/singlefs-checker/src/lib.rs:720`），「重放从点名项直接凑出映射 key」（`.claude/kb/decisions/23-journal的角色与格式.md:472`），而映射 key 与单元头相符是 I-1.11 已判的东西（`.claude/kb/invariants.md:34`）。校验和不相符的位置条目不判：「记录点名的每一项自带校验和」（`.claude/kb/decisions/23-journal的角色与格式.md:474`），落点被合法复用之后单元头就不是它点名的那个单元了。
- 措辞（建议号 I-8.11）：「任一自证过的 journal 记录里的任一点名项的每条位置条目，若它指着的那一份单元的整单元校验和与位置条目里的相符，则单元头里的类标签、诞生代号、key 尾段与点名项偏移 28、37、45 的同名字段逐字节相同（码 1 尾段 = 写序实例代号 4 + 事务号 6，码 2 / 码 3 尾段 = 实例代号 4 + 出生序号 4 + 补零 2，补零恒 0）；出生树（点名项偏移 29）只判码 1、码 3，等于头偏移 43，码 2 随 C289 不判。校验和不相符的位置条目不判。」
- **底稿这一句不能整条照抄**：底稿的措辞把「出生树」一起收进去、没分码 2（`research/prompts/c395-invariants-draft.md:22`），码 2 那一段要跳过，理由同 B1。
- checker：`scanned_journal_records_of_device`（`crates/singlefs-checker/src/walk.rs:4918`）只留点名项的位置条目（`walk.rs:4913` 的 `named_entry_locations`），要把 `NamedEntryView` 的其余字段也留下（`lib.rs:708`），照 `walk_central_mapping_entries`（`walk.rs:987`）的读法比；两处不同：映射条目校验和对不上是 I-2.1 违例，点名项对不上是跳过。
- 坏镜像（覆盖写之后那份）：计数器 4（发布 B，环里最后一条）里类标签 1 那个点名项（数据单元，槽 50182）偏移 37 的出生 txg 4 改 3，两盘各一份，先重算载荷 CRC（偏移 95，罩点名项区），再重封头校验和（偏移 46）。单元没动，位置条目校验和仍相符 ⇒ 判到它；没有下一条的反向链要改；I-2.5 只读位置条目的顺序 ⇒ 推得只红 I-8.11。**底稿写的载荷 CRC 偏移 91 是错的**：91 今天是反向链，载荷校验和在 95（「const JOURNAL_PAYLOAD_CHECKSUM_OFFSET: usize = 95;」，`crates/singlefs-checker/src/lib.rs:702`）。变异：比较结果换成 `true`。
- 与底稿比：判法一致；「`walk.rs` 一处都没读 `named`」（`research/prompts/c395-invariants-draft.md:22`）过时，2026-09-27 起读点名项的位置条目判 I-2.5（`crates/singlefs-checker/src/walk.rs:4971`），出生树 / 出生 txg / key 尾段仍不读；「点名项的诞生代号 = 这条记录的 checkpoint_txg 推不出」仍对（切换照旧那一格，同 3a）。

### 3c　底稿推的两处

① 把候选根当快照 S 用：**上界成立、下界不成立**，成立的那一半也不是 I-3.5 的条款（3a）。② 实例切换照旧的数据单元：上界 X ≤ Y 成立（3a 里 `.claude/kb/decisions/23-journal的角色与格式.md:369` 的下界推出）；下界方面，R_prev 是切换前最后一条落盘的根，txg ≤ X − 1（被重发的是「最后已发布 + 1」那个在飞 checkpoint；根已落盘则切换按读盘选出的根接着走，不重发，`.claude/kb/decisions/23-journal的角色与格式.md:409`）。但下界那一半整体已出局，这一格没有再讨论的对象；切换重发今天没有实现，没有镜像。

## B4　计数类字段（底稿主表第 5、7、8 行）

### 4a　`previous_snapshot_txg`（条目偏移 100）

- 条款：「`previous_snapshot_txg` 只写在该头 inode 树那条树表条目里，其余条目写 0 且 0 = 不适用」（`.claude/kb/decisions/05-快照-空间记账机制.md:225`）。core 恒写 0：「writer.put_u64(0); // previous_snapshot_txg：0 = 不适用」（`crates/singlefs-core/src/records.rs:296`），读出即丢（`records.rs:329`）。
- **进（建议号 I-9.17）**：「任一走读到树表的根（候选集里每条）的树表里，树的种类不是 inode 树（种类码 ≠ 2）的条目，previous_snapshot_txg == 0。」
- **不进，交用户 / 记欠账**：inode 树那条「0，或 1 ≤ p ≤ 这条根的 checkpoint_txg」的上界半。条款只说 inode 树条目可非 0、快照 txg ≥ 1；「≤ 根 txg」是从 S.txg 的定义（`.claude/kb/decisions/05-快照-空间记账机制.md:315`）加「根只记已存在的快照」推的，底稿自己写了「是推的」（`research/prompts/c395-invariants-draft.md:23`）。第一版没有快照（`grep -rn 'create_snapshot\|clone_head' crates/singlefs-core/src` 零命中），条目恒 0，这半句今天没有对象；跟 D5 已定项 21 的快照列表一起定。
- checker：`references_of_root`（`crates/singlefs-checker/src/walk.rs:2863` 一带的树表条目循环）已按 200 字节切条目、读了种类（偏移 10）与诞生 txg（偏移 108），加读偏移 100。今天判定恒真（合法镜像全 0），价值是抓 core 把别的数写错位到偏移 100。
- 坏镜像：新池新建文件那份，树表（`TREE_TABLE`＝槽 50252，`checker_known_bad_images.rs:107`）第 0 条（extent 树，种类 1）偏移 `tree_table_entry_offset(0) + 100`（`checker_known_bad_images.rs:552`，即 231）0 改 3，`mutate_unit(TREE_TABLE, …, true)`；I-9.14 只读偏移 108，I-9.16 读树 ID，推得只红。变异：判定恒真。
- 底稿：措辞一致；**槽号 50248 过时**，今天树表是 50252（`crates/singlefs-harness/tests/checker_known_bad_images.rs:107`）。

### 4b　记账代

- 条款：「每条根的记账树只装这条根那一代的行，不留多代」（`.claude/kb/decisions/05-快照-空间记账机制.md:47`）与四个等号里的「记账代 = checkpoint 号」（`.claude/kb/decisions/16-发布语义.md:156`）；空发布同样重写：「记账树存在时就写——空发布也是发布」（`.claude/kb/decisions/16-发布语义.md:218`）。
- 措辞（建议号 I-3.13）：「走读到的任一版（候选根，以及由记录施加出来的那几版）的记账树里，每一行的代段（记账 key 偏移 14，8 字节）等于这一版的 checkpoint_txg。」
- 合法镜像上恒真：core 每次发布写的行「generation: txg,」（`crates/singlefs-core/src/transaction.rs:4114`，每块盘那六行在 4125）；带文件的一版上的空发布走 `publish_version`，不走零单元发布（`crates/singlefs-core/src/mount.rs:2818` 的 `publish_empty_after`：`WithFile` 分支 `publish_version`，`WithoutFile` 分支才 `publish_without_units`），零单元发布只在树表 0 条的一版，那一版没有记账树。**底稿担心的「走错」今天不存在**（`research/prompts/c395-invariants-draft.md:25`）。没核：回退那次发布落盘的记账行代段（`mount.rs:4931` 的 `version_to_publish_the_rollback_after` 在内存里把现行版本的 `generation` 原样带过），我没读到它落盘前是否按新 txg 重写，交攻方腿。
- checker：记账行读在 `walk_tree_table_entry` 里（「.insert((read_u16(row, 0), read_u32(row, 10)), read_u64(row, 22));」，`crates/singlefs-checker/src/walk.rs:1346`），不读偏移 14；走读要带上这一版的 txg：根记录里在偏移 28（`walk.rs:5534`），记录施加出来的那一版在 `version.checkpoint_txg`。只判走全了的树。
- 坏镜像：新池新建文件那份，记账树（`ACCOUNTING_ROOT`＝槽 50250，`checker_known_bad_images.rs:106`）15 行里挑 key 不在首尾的一行（行起点 159 + 34 × 下标，`checker_known_bad_images.rs:3008`），代段（行偏移 + 14）3 改 2，头里的 key 区间不动 ⇒ I-1.1 不红；I-3.1、I-5.2、I-9.6 只按 (统计量, 设备) 取值 ⇒ 推得只红。变异：判定恒真。
- 底稿：一致；**槽号 50246 过时**，今天 50250。

### 4c　inode 改动计数（记录偏移 88）

- 条款：「改动计数（最后一次改动所在发布的 checkpoint_txg；提交锚 = 叶容器头的诞生代号；同一发布窗口内多次改动只有最后一版落盘——内部节点不缓冲）」（`.claude/kb/decisions/08-核心索引结构.md:187`）；叶容器每次发布重写、换新诞生代号：「开放容器在它所属代际内每次要发布时被重写，新单元、新诞生代号、身份不变。」（`.claude/kb/decisions/18-块里携带什么信息.md:294`）。
- **进上界（建议号 I-9.18）**：「任一走读到的 inode 叶容器（码 3，打包记录类型 2）里每条记录：改动计数（记录偏移 88）≤ 这片容器头的诞生代号（偏移 73）。」不是等于：容器因别的记录改动而重写时，这条没改的记录改动计数小于容器诞生代号是合法的。回退整批取目标版的容器，两个数同属那个年代，不破；分裂 / 合并重写的容器诞生代号是这次发布的 txg，也不破。
- **不进下界，交用户**：「对象出生代（偏移 8）≤ 改动计数」要「创建算一次改动」。条款只有一个例子：新池新建文件的记录「改动计数 3」（`.claude/kb/decisions/08-核心索引结构.md:201` 那一行）；core 建对象时写「let change_count = txg.0;」（`crates/singlefs-core/src/transaction.rs:4844`），上一行的注释说建出来那次就是这次发布。合法镜像上不会误红，但条款不支撑，一句话补进 D8 已定项 6 偏移 88 那一行即可（「创建也算一次改动」），补了再进。
- checker：`walk_inode_leaf_container`（`crates/singlefs-checker/src/walk.rs:1568`）逐条记录那一段（1601 起）读了偏移 0、8、40、48、108..140，加读偏移 88 与容器头偏移 73（外面已在读偏移 43、53、61）。
- 坏镜像：新池新建文件那份，`INODE_LEAF`（槽 50242，`checker_known_bad_images.rs:102`）第 0 条记录（记录起点 136）偏移 136 + 88 的 3 改 4（> 容器诞生代号 3），形同既有 I-9.10 那份（`crates/singlefs-harness/tests/checker_known_bad_images.rs:1027`，偏移 136 + 8 改 2）；I-9.10 只比对象出生代，不读 88 ⇒ 推得只红。变异：判定恒真。
- 底稿：**「`walk_inode_root` 已逐条读记录」过时**，今天逐条读记录的是 `walk_inode_leaf_container`（`walk.rs:1568`），`walk_inode_root`（`walk.rs:1405`）只处理根。

## 续做说明

上一条腿撞限额停在 B4c 之后，已落 147 行（B1–B4，cite-check 40 处全对）。这一条核过现场后不重写前面，从 B5 起追加。B1–B4 的引文与行号没有再核第二遍，只跑了末尾的 cite-check。下面 B5–B7 与判定一览是这一条写的。

## B5　实例表行 T（底稿主表第 9 行）

### 应该是什么

- 行字段：「`kind` = 0 行记录：`kind 1 | 实例代号 4 | 所选根的 checkpoint_txg 8 | 属于该实例的最大已施加事务号 W 8 | flags 1」（`.claude/kb/decisions/18-块里携带什么信息.md:299`），T 在行偏移 5，checker 已按这个偏移解：「published_checkpoint_txg: u64::from_le_bytes(row[5..13]」（`crates/singlefs-checker/src/walk.rs:3749`）。
- T 写什么：恢复在所选根那一版表上给每个实例写行，所选根那个实例写「所选根那个实例写 (i, **重放之后那个根的 checkpoint_txg**」（`.claude/kb/decisions/18-块里携带什么信息.md:306`），中间实例写 (i, 0, 0)。代码同形：「if row_instance == previous_row.instance.0 {」（`crates/singlefs-core/src/mount.rs:3093`）那一支取 `selected_root_txg`，它来自「selected_root_txg: effective_root.checkpoint_txg,」（`crates/singlefs-core/src/mount.rs:4518`），另一支写 (i, 0, 0)。
- 新实例第一次发布的 txg：「新实例（普通挂载、切换）的第一次发布的 checkpoint_txg ≥ max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1」（`.claude/kb/decisions/23-journal的角色与格式.md:369`）。代码：「checkpoint_txg_after_the_version(highest_ring_txg.max(highest_record_txg))」（`crates/singlefs-core/src/mount.rs:1114`）。
- 写行与第一个新根同一次发布：「按实例代号唯一、后写覆盖，只在恢复与实例切换时写」（`.claude/kb/decisions/18-块里携带什么信息.md:305`）那一行接着写「与第一个新根同一次发布」。
- checker 今天：一根一张表只读到最新根那一张的行（`judge_instance_table_rows`，`crates/singlefs-checker/src/walk.rs:1874`，只判唯一与低于挂载根实例）；别的根的表有现成读法：「fn instance_table_rows_of_root_without_judging(」（`crates/singlefs-checker/src/walk.rs:3762`，读 `record[170..256]` 那条指针）。T 的值今天有三处用：判被抛弃（`walk.rs:3818`）、谓词（`walk.rs:4587`）、记录施加（`walk.rs:5608`），没有一处比 T 与根的 txg。

### (a)　T < 引用这张表的根的 txg：进（建议号 I-3.12）

推导：任何引用表 X 的根 R，R 是写 X 那次发布 P 或它之后的根，或回退发布（向前发布，引用现行版本的表）。X 里每一行的 T：恢复写的 = 重放之后那个根的 txg ≤ 环里读得出的记录与根的最大 txg（`mount.rs:4518` 与 `mount.rs:1114` 同一批输入），P 的 txg 是它加一（`mount.rs:1114`），故 T < P.txg ≤ R.txg；中间实例写 0 < 任何根 txg。实例切换写的行（D18 已定项 11 的四条，`.claude/kb/decisions/18-块里携带什么信息.md:307` 起）取值都是被重发那次的在飞号 X 之前的根 txg 或 0，重发 txg Y ≥ X（`.claude/kb/decisions/23-journal的角色与格式.md:369`），所以上式对切换也成立；切换今天没有实现，这一句只有条款推演、没有镜像。

措辞：「走读到的（根环里自证过的）任一根 R，它指着的实例表的每一行 (i, T, W) 满足 T < R 的 checkpoint_txg。」链上某一片读不出的根不判。

- 挂法：新立，不并进 I-3.8。I-3.8 的射程是「最新根那一张表」（`judge_instance_table_rows` 只在挂载根那一张上调），并进去要改它的入参；新立一条按根循环读表即可。
- checker：`valid_roots`（`crates/singlefs-checker/src/image.rs:613`）交的每个根用 `instance_table_rows_of_root_without_judging(reader, &root.record_bytes)` 读一遍（返回 None 跳过），逐行 `T < root.checkpoint_txg`。多条根共用同一张表时结果相同，不必去重（省读可按表单元落点缓存）。合法镜像上恒真，由上面推导；空表（mkfs 那片）没有行、判定空成立，报不适用要看那条判据是否规定「一行都没有」的写法（照 I-3.9 的写法，没有可判的行时报不适用）。
- 坏镜像（设计，没造没跑）：`image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record`（`crates/singlefs-harness/tests/checker_known_bad_images.rs:4213`）的 `image`，写行发布 txg 6 写下行 (1, 5, 3)，暖机 txg 7。用 `instance_table_chain_of_root`（同文件 5236 行那样的读法）取表的槽，行记录起点 136 + 88 × 下标，T 在 +5：把 5 改成 6（= 引用它的 txg 6 那条根自己的号，正好卡在严格小于上），`mutate_unit_of(..., true)` 沿引用链重封到根记录。T 只在三处别的地方用（上面三处）：实例 1 在 (5, 6] 里没有根也没有单元（6 是实例 2 的写行发布），所以被抛弃、谓词、记录施加都不变 ⇒ 推得只红新条。变异：新条判定换成 `true`；再加一条把 `<` 换成 `<=` 的变异，这份镜像卡在等号上，`<=` 会放过它。
- 与底稿比：措辞一致；底稿的依据 QT1、QT2、QT4、QT3 里 QT4（回退写的行）已删（D18 已定项 11 写「管理员回退是挂着时的一次向前发布，不写行」，`.claude/kb/decisions/18-块里携带什么信息.md:305`），(a) 今天的依据是上面的 306 行与 D23 369 行，结论没变。底稿「T 的精确值推不出」仍对：T 等于重放之后那个根的 txg，那条根常已轮出根环，所以只判上界。

### (b)　同一实例的行在候选集两张表里逐字节相同：不立

- 条款没有这句。最接近的一句是「后来的恢复只给 [新的所选根实例, 新实例) 写行、不回头改旧行，所以表上允许有多行 W ≠ 0」（`.claude/kb/decisions/18-块里携带什么信息.md:306`），它说的是不回头改**更低实例的**旧行；对 [新的所选根实例, 新实例) 里的实例，新恢复照样写新行，条款给这些行的是「按实例代号唯一、后写覆盖」（`.claude/kb/decisions/18-块里携带什么信息.md:305`）。
- 能造出的差异（推的，没造）：恢复 X（实例 2）选实例 1 的根 A5，重放到记录 6，写 (1, 6, w)；实例 2 发根 7、8。恢复 Y（实例 3）里实例 2 的根槽读不出，选同一条 A5，记录 6 也读不出，写 (1, 5, 0)。条款明写这条路：「恢复恒选最新可读且自证通过的根，它可以不是最新发布过的根（槽坏了就退一格」（`.claude/kb/decisions/18-块里携带什么信息.md:306`）与「槽恢复后被抛弃的根回到回退候选集」（`.claude/kb/decisions/18-块里携带什么信息.md:310`）。两张表同实例的行 (1, 6, w) 与 (1, 5, 0) 不同。
- 试着把这个差异塞进「按最新根的表判仍然有效」的候选集里，我没造出反例（有效的根要么早于 T、要么其实例的行没被后来的恢复改写），但那是一个要按谱系做的归纳，条款没有写、我也没证完；判据第一条是「推不出条款支撑的那一半 ⇒ 不进不变量」，(b) 不进。攻方腿可以用「Y 那次读不出的根槽后来又读得出、且 Y 选的根不是最新根」那一格去攻它，也可以证它。
- 顺手：行的 flags 一字节今天没人读（`parse_instance_table_row` 只解 1..21，`crates/singlefs-checker/src/walk.rs:3746` 起），条款「flags 1（第一版恒 0、非 0 拒收」在 `.claude/kb/decisions/18-块里携带什么信息.md:299`，实现的读者拒收（`crates/singlefs-core/src/instance_table.rs:59` 一带），checker 一侧没有会红的检查。这不在 B5 的格里，写在这里交主 agent 判要不要另记一条欠账。

### 推翻条件

- (a) 被推翻：找到一份合法镜像，某条根引用的表里有一行 T ≥ 该根 txg。最可能的来路是实例切换（今天没实现）或恢复取的「重放之后那个根」txg 大于第一次发布的 txg（要 `mount.rs:1114` 少算了记录 txg）。
- (b) 不立被推翻：证明候选集（按最新根的表判有效）里两张表的同实例行恒相等。


## B6　三个计数器的取值（底稿 ② 表）

### 6a　系统配置槽世代号：奇偶与槽号进，「两槽都是 1」交用户

条款：「槽世代号从 1 起、每写一次 +1，下一次写的槽 = 世代号 mod 2，择槽取校验和过且世代号最大的（逐盘计：取号、发布末尾的轮换、取号全或无失败时的回卷三种系统配置写都写这块盘上自证过的槽里最大的世代号 + 1）」（`.claude/kb/decisions/22-单元原子性怎么合成.md:331`）。代码同形：写入的世代号是这块盘上自证过的槽里最大的加一（`crates/singlefs-core/src/transaction.rs:364` 一带的 `.max()` 与 `+ 1`），落槽「(slot_generation % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * spacing,」（`crates/singlefs-core/src/transaction.rs:409`）。mkfs 两槽都写 1：「pub const SYSTEM_CONFIGURATION_GENERATION_AT_MKFS: u64 = 1;」（`crates/singlefs-core/src/make_filesystem.rs:46`），字节表写「| m4 | mkfs | 系统配置，两槽同内容、世代号 1、实例代号 0」（`.claude/kb/layout/01-first-txn.md:59`）。

推出：一块盘上凡是本条写出的槽，世代号 g ≥ 2 时落在槽 g mod 2；写入总是「最大自证世代号 + 1」进另一个槽（最大那槽的奇偶恰是槽号，加一之后落到另一槽），所以两槽都自证过时相差恰为 1，只有 mkfs 那一刻两槽同为 1。撕裂写不改这个式子：撕的槽读不出，下一次仍从自证的最大值加一，又落回撕的那一槽。

- 措辞（建议号 I-7.15，`grep -rnF 'I-7.15' .claude/kb` 零命中）：「每块盘上，任一可择（自证过、池级字段在读者收的范围里）的系统配置槽 i，世代号 ≥ 2 时世代号 mod 2 == i；两个槽都可择时，世代号相差不超过 1。」
- 用「不超过 1」不用「都是 1 或恰差 1」：mkfs 那一刻的 (1, 1) 差 0 也落在里面，这一句只靠 D22 已定项 16（`.claude/kb/decisions/22-单元原子性怎么合成.md:331`），不靠字节表。「相等只许都是 1」那一半依据只有 `.claude/kb/layout/01-first-txn.md:59` 一行，决策正文没有；要不要把「mkfs 两槽都写 1」写进 D22 已定项 16 是条款变更，交用户（判据第四条）。
- 带越界值的槽（`CarriesAValueThisReaderRefuses`，`crates/singlefs-checker/src/image.rs:380`）自证过但没有 `view`，不进这一条（它归 I-7.13 整池拒绝）。
- checker：**底稿说要改的那一处今天已经有了**。底稿写「`verified_system_configuration_slots` … 槽 0 无效时丢了槽号；要改成带槽号交回」，今天 `system_configuration_slot_readings`（`crates/singlefs-checker/src/image.rs:456`）逐盘交回 `[槽 0, 槽 1]` 两个 `SystemConfigurationSlotReading`，槽序就是数组下标，`Usable(view, _)` 里 `view.slot_generation`（`crates/singlefs-checker/src/lib.rs:173`，取自偏移 147，`lib.rs:280`）。新条直接消费它，不用改交回形状。
- 坏镜像（设计，没造没跑）：新池新建文件那份，盘 0 两槽世代号是 (4, 5)（`.claude/kb/layout/01-first-txn.md:66` 与 `:81`：暖机槽 0 世代 4，事务后槽 1 世代 5）。两份，各只红一半：
  - 差值那一半：槽 1（偏移 = 槽距，取 `parameters().geometry.fixed_structure_slot_spacing`，不写死 4096）世代号 5 → 7，奇偶仍对、差 3；用 `mutate_system_configuration_slot`（`crates/singlefs-harness/tests/checker_known_bad_images.rs:438`）写偏移 147 并重封（偏移 155）。择槽仍选槽 1，别处不读世代号 ⇒ 推得只红新条。
  - 奇偶那一半：槽 0 的 4 → 5、槽 1 的 5 → 6，两槽都错位、差 1 ⇒ 只红奇偶那一半；择槽仍选槽 1（内容没动）。
  - 变异：新条里 `mod 2` 那一比与「差不超过 1」各换成 `true` 两条，一条一份镜像。
- 与底稿比：措辞底稿是「要么两槽都是 1，要么相差恰为 1」，我改成「不超过 1」，理由上面；「≥ 2 才判奇偶」一致；「推不出：两槽都是 1」一致（依据只在字节表）。底稿槽号偏移与「丢槽号」两处过时。

### 6b　根记录所在区域 = txg mod R、槽 = (txg div R) mod S：进（建议号 I-7.16）

条款：「**跨区轮转**（区域 = `txg mod R`）；区域内槽位 = `(txg div R) mod S`」（`.claude/kb/decisions/22-单元原子性怎么合成.md:57`）；「**四个等号**：fsync = 提前发布、区域 = txg mod R、一个 txg 一个号、记账代 = checkpoint 号」（`.claude/kb/decisions/16-发布语义.md:156`）。代码：写根只有一个入口 `target_for_publish`：「region: checkpoint_txg.0 % ROOT_RING_REGIONS,」（`crates/singlefs-core/src/root_ring.rs:114`）与「slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % slots_per_region.count(),」（`crates/singlefs-core/src/root_ring.rs:115`），三处调用（`crates/singlefs-core/src/mount.rs:1833`、2299、2429）加分配器一处（`crates/singlefs-core/src/allocator.rs:1649`），`grep -rn 'target_for_publish\|RootRingSlot {' crates/*/src` 里别的写根路径只有 mkfs 那一处种第 0 代。

第 0 代：mkfs 把第 0 代根种进全部区域：「(0..ROOT_RING_REGIONS).map(|region| RootRingSlot { region, slot: 0 })」（`crates/singlefs-core/src/make_filesystem.rs:610`）与 D22 已定项 8「**mkfs 要把第 0 代根种进全部根环区域**」（`.claude/kb/decisions/22-单元原子性怎么合成.md:176`）；所以 txg 为 0 的根在区域 0、1、2 的槽 0 都合法，只判槽 == 0。

- 措辞：「根环里任一自证过的根 R，它所在的槽位是 (区域 r, 槽 s)：checkpoint_txg ≥ 1 时 r == txg mod R 且 s == (txg div R) mod S；checkpoint_txg == 0 时 s == 0，不判 r。」
- 合法镜像上恒真：写根只走 `target_for_publish`，且 R、S 是池里恒定的（S 由 mkfs 定：`ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = 8`，`crates/singlefs-format/src/lib.rs:256`，R = 3）。轮出、撕裂、设备失而复得都不改一条根住哪个位置（位置由它自己的 txg 定，读盘时 C88 的双根平局不同 txg，不干这一条）。实例切换重发是新 txg 的普通发布，落它自己的槽。
- checker：`valid_roots`（`crates/singlefs-checker/src/image.rs:613`）已交 (区域, 槽, 根)，根的 txg 在 `RootView.checkpoint_txg`（`crates/singlefs-checker/src/lib.rs:346`，根记录偏移 28）；R、S 在 `PoolGeometry`（`root_slot_positions`，`image.rs:593`）。只是一个循环，没有新输入。
- 坏镜像（设计，没造没跑）：新池新建文件那份，`root_slots()`（`crates/singlefs-harness/tests/checker_known_bad_images.rs:142`：区域 r 在盘 [0, 1, 0][r]，槽距 4096，每区 8 槽）。这份里根环是：区域 0 槽 0 是第 0 代、槽 1 是 txg 3；区域 1 槽 0 是 txg 1；区域 2 槽 0 是 txg 2。把 txg 1 那条根（区域 1 槽 0，盘 1）整槽原样复制到区域 2 槽 3（盘 0，mkfs 清零过的空槽）：自证校验和只罩槽内字节、不含位置（`check_root_slot`，`crates/singlefs-checker/src/lib.rs:318`），照样过；1 mod 3 ≠ 2 ⇒ 红。多出来的与 txg 1 同内容，走读引用同一批单元；C88 那条「同 `(实例代号, checkpoint_txg)` 的自证合法根 ≥ 2」没实现，不跟着红 ⇒ 推得只红新条（推的）。它同时错区域与槽（两个都不对）；要分开就各造一份：槽错区域对用 txg 3 那条（区域 0 槽 1）复制到区域 0 槽 5，区域错槽对用 txg 3 复制到区域 1 槽 1，但复制到最新根会让择新出现同 txg 双根，要先核。变异：新条比较换成 `true`。
- 与底稿比：措辞一致（第 0 代只判槽号），底稿的镜像现在仍成立（每区 8 槽、区域 2 在盘 0 上，与今天 `root_slots()` 相符）。底稿说这条「就是 C85 里『区域 = txg mod R』那个等号的会红检查」：`.claude/kb/checks-owed.md:85` 写着「四个等号还没写进」，同一行还写「「同一 txg 不许两条根」在设备失而复得下会误伤，正确判据要带实例代号」，新条不判同 txg 双根，这一点不冲突。

### 6c　journal 记录自身的 txg：同实例不减进（建议号 I-8.12），换实例严格大于不进

- 条款：每次发布加一（`.claude/kb/decisions/16-发布语义.md:156` 的「一个 txg 一个号」）；一次发布可切成几条记录，共享一个 txg；新实例第一次发布 ≥ 环里读得出的最大 txg + 1（`.claude/kb/decisions/23-journal的角色与格式.md:369`）。
- 同实例内按计数器排，txg 不减：一个实例里发布串行、一个事务失败即实例切换（不在同实例里重发），管理员回退是向前发布、拿新 txg，所以同实例内 txg 只增或（同一次发布的几条记录）相等。措辞：「同一块盘 journal 环里自证过的记录，同一实例代号的记录按 jsn 计数器从小到大，checkpoint_txg 不减。」只比同实例的记录，不管中间有没有别的实例的记录或环里缺号。
- **换实例严格大于不进**：底稿这一半的依据是 D23 那一句，但那一句写的是「环里全部自证通过的记录」，只对挂载那一刻读得出的记录成立。C88 写「设备失而复得会让根环里并存两条同 checkpoint_txg 的自证合法根」「回退后新时间线复用了旧计数」（`.claude/kb/checks-owed.md:88`）：一块盘回归带回的旧记录，新实例挂载时看不见它，新实例的 txg 可以不大于它；这一格底稿标「只推了、没核」，我也没核。合法镜像上可能误红 ⇒ 判据第一条，出局；留给攻方腿。
- checker：`ScannedJournalRecord` 已有 `instance` 与 `checkpoint_txg`（`crates/singlefs-checker/src/walk.rs:4893` 起），`scanned_journal_records_of_device`（`walk.rs:4918`）按计数器收成 `BTreeMap`；同实例过滤后一遍比较，与 I-8.7（实例内事务号不重号）同一遍扫。
- 坏镜像（设计，没造没跑）：覆盖写之后那份，计数器 4（发布 B，环里最后一条）两盘各一份，记录偏移 26 的 4 → 0，重封头校验和（偏移 46，罩整条 4096；参照 `set_transaction_number_of_journal_record`，`crates/singlefs-harness/tests/checker_known_bad_images.rs:390`）。取 0 不取 2 / 3：(1, 2)、(1, 3) 在根环里有根，改成它们会让 I-8.10 与 I-8.9 跟着动，(1, 0) 没有根、也不是别的记录的号；最后一条没有下一条的反向链 ⇒ 推得只红新条；这份镜像上实例表没有行，「只由记录施加的版本」不会拿 (1, 0) 当施加对象（没核）。变异：比较换成 `true`。
- 与底稿比：同实例不减一致；换实例严格大于那一半不进（上）；底稿说的「设备失而复得只推了、没核」一致。

### 推翻条件

- 6a 被推翻：找到一块盘的合法历史里两槽自证世代号差 ≥ 2，或世代号 ≥ 2 的槽 i 上 g mod 2 ≠ i。最可能：取号回卷（「取号全或无失败时的回卷」）写的不是最大自证世代号加一（`crates/singlefs-core/src/transaction.rs:364` 一带没有别的取值路径，我没逐条追回卷的调用点）。
- 6b 被推翻：找到 `target_for_publish` 之外的写根路径，或 R、S 可以随时间变。
- 6c 同实例不减被推翻：同一实例里出现 txg 回退的合法记录序列（例如管理员回退把 txg 也退了）；D23 已定项 14 写回退是向前发布，拿新 txg。


## B7　I-3.9 的空当（底稿「① I-3.9 那一处空当」）

### 应该是什么

- 判据：「一条已释放记录判得了，当且仅当 checkpoint_txg 大于 L 的每条候选根都走得全」（`research/prompts/c395-invariants-draft.md:41`）。L 是候选集里还引用这个落点的最新一条根，T 是 L 之后第一条候选根；释放代必须落在 (L, T]（`.claude/kb/invariants.md:142`，I-3.9 一行）。
- 今天的守卫：「if !scanned.iter().all(|root| root.references.is_complete) {」（`crates/singlefs-checker/src/walk.rs:3279`），任一候选根走不全就整条报不适用并返回。`is_complete` 在 `references_of_root`（`walk.rs:2793`）里被置假的路径有：分配记录树读不出（`walk.rs:2834`）、树表读不出或树表条目窄（`walk.rs:2839` 起）、某棵树的根读不出（`walk.rs:2872`）、映射根读不出（`walk.rs:2896`）等。
- 推：一条根 R 若 txg ≤ L，它不改 L（L 是最大值）也不改 T（T 取 L 之后的根）；所以 R 走不全与否对这条记录无关。R 若 txg > L 而走不全，R 可能藏着对这个落点的引用，真 L 会更大，T 也跟着变，所以不能判。收窄后的守卫「比 L 晚的每条候选根都走得全才判这一条记录」不会让合法镜像上判红：走得全的根看得见全部引用，比看得见的 L 晚的根既然都走得全，就没有藏着的引用能把真 L 抬到看得见的 L 之上。
- 模型：`python3 research/prompts/m3-inv-b-r1-sonnet-model/b7_guard_model.py`（`nice -n 19` 跑，这一次跑）：

```
情形数: 43302
今天（任一根走不全整条不判）: 合法 g 判了 31 次、误红 0；非法 g 判了 186 次、抓到 186
不设守卫: 合法 g 判了 5764 次、误红 1640；非法 g 判了 34584 次、抓到 32944
收窄（比 L 晚的根都走得全才判）: 合法 g 判了 1562 次、误红 0；非法 g 判了 9372 次、抓到 9372
```

模型是抽象的（候选根 6 条、每条根走得全 / 走不全、走不全的根看得见的引用取真引用的任一子集），不是实现：它证明收窄后的判据在这个抽象里零误红，判得到的合法情形从 31 涨到 1562；不设守卫会在 1640 个合法情形上误红。

### 实现改哪里（对 `crates/` 今天的实现）

`judge_release_generations`（`crates/singlefs-checker/src/walk.rs:3257`）：去掉 3279 那个整条返回的守卫；在每条记录算出 `last_referencing_txg`（`walk.rs:3289`）之后加：`scanned` 里 txg 大于它的根只要有一条 `!is_complete`，这一条记录跳过、记数；循环结束时数大于 0 就调 `judgements.part_of_the_range_not_judged("I-3.9", …)`（`crates/singlefs-checker/src/image.rs:159`，没有违例时整条改报不适用、判出违例照报）。这里用 `part_of_the_range_not_judged` 是对的（跟 B1 的码 2 那一处不同）：它只在真有候选根走不全的镜像上触发，干净镜像上不触发。`witnessed_records == 0` 那一支照留。

### 底稿造法过时之处

- 底稿「A 那棵分配记录树（槽 50245，B 之后只有 A 的根引用它）两盘各翻一个载荷字节 … A 的落点照样数得全：分配记录树那条根指针在读节点之前就记进了 `placements`」。今天分配记录树多层：两块盘的叶 61（50245、50246）、第 1 层节点 0（50247、50248）与根（50249）（`crates/singlefs-harness/tests/checker_known_bad_images.rs:105` 与 `:2650` 一带的常量与注释），`allocation_record_tree_without_judging`（`walk.rs:3020`）整棵读、节点读不出就交回 `None`（如 `walk.rs:3045`），已收的节点指针不交回（`walk.rs:2834` 只置 `is_complete = false`）。翻叶 50245，A 的 `placements` 少四个节点（第 1 层两个、叶两个），这四个单元的已释放记录（两盘各一条，8 条）在 I-3.9 里没有引用过它们的根、被跳过；判得到的是剩下的 24 − 8 = 16 条（推的）。所以底稿的「落点数得全」在今天要改：**翻映射根**。
- 底稿「B 的分配记录树（槽 50253）里 16 条已释放记录」：B 的两片叶今天是 50257、50258（`checker_known_bad_images.rs:95`），已有的 I-3.9 用例改 24 条（`checker_known_bad_images.rs:1295` 的 `rewrite_released_generations(bytes, 4, 3)`，断言「A 的十二个单元、两盘各一条」）。
- 底稿「`image_after_the_overwrite`」的名字今天仍在。

### 正例：走不全的根不比 L 晚（应判红）

覆盖写之后那份：

1. 同已有 I-3.9 那份（`checker_known_bad_images.rs:1284` 的 `known_bad_images_after_the_overwrite`）：B 的两片叶里 24 条已释放记录的释放代 4 → 3，`mutate_allocation_leaves` 重封并补链。
2. 让 A（txg 3）走不全：A 的映射根（`MAPPING_ROOT` 一类，A 独有的那一个，槽 50251，`checker_known_bad_images.rs:2939`）两盘各一份都翻一个载荷字节，直接 `write`，不重封、不沿链补校验和。A 的映射根指针在读节点之前记进 `placements`（`walk.rs:2817` 的 `references.note(&mapping_root)`），根之下没有节点，映射条目指的单元与树表走读记的是同一批（`walk.rs:2814` 的注释）⇒ A 的 `placements` 不少（推的，造的时候要核：翻前翻后 A 的 `placements` 集合相同）。
3. 该判什么：I-3.9 **违例**：还引用这些落点的最新根 L = A（txg 3），比 L 晚的根只有 B（txg 4）、走得全 ⇒ T = 4，释放代 3 不在 (3, 4] 里。今天报不适用（`walk.rs:3279`）。
4. 同一份镜像上一起红的会有 I-2.1、I-7.4、I-4.8（A 的走读读到坏单元，位置条目校验和对不上）——造法带来的，不是 I-3.9 的误报；用例断言精确的红集合，不写成「只红 I-3.9」（照实：不满足「只红它」，原因是造 `is_complete = false` 必须让一个单元读不出，读不出本身由那三条说话，这一格按判据第二条写「未实现 / 一起红」的写法）。
5. 变异：把新守卫改回「任一根走不全就整条返回」，这份镜像上 I-3.9 由违例变成不适用。

### 对照格（底稿说还缺的）：走不全的根比 L 晚、落点藏在它读不出的节点下（不该判）：造得出，推的

底稿说现成镜像造不出，因为覆盖写重写整份文件，没有跨两条以上根共享、又在第三条根被释放的单元。今天造法：新池新建文件（A，txg 3）之后，推一次带文件的空发布 E（txg 4），再覆盖写 B'（txg 5）。空发布参照 `crates/singlefs-harness/tests/multi_record_publishes_of_transaction_zero.rs:284` 那样直接调 `publish_version`（`file: None`、`new_inode_records: &[]`）；带文件的一版空发布重写固定点单元（记账树存在，`crates/singlefs-core/src/mount.rs:2818` 一带 `publish_empty_after` 的注释），不换 extent 树与数据单元，所以数据单元 u 被 A、E 共享，B' 覆盖写时在 txg 5 释放它。

- E 独有的树表单元翻一个载荷字节（两盘各一份，不重封）：`references_of_root` 读不出树表就在记完树表指针之后返回（`walk.rs:2847` 起），E 的 `placements` 里没有 u，`is_complete` 假。
- u 的已释放记录（释放代 5，合法：真 L = E = 4，真 T = B' = 5，5 ∈ (4, 5]）：看得见的 L 是 A（txg 3），比 L 晚的根里 E 走不全 ⇒ **收窄后不判**。不设守卫时按看得见的算：T = E（4），(3, 4] 不含 5 ⇒ 判红，是合法镜像上的误红。这份镜像要看到的是：新守卫下 I-3.9 不是违例（不适用），把守卫整个删掉之后它变成违例。这份镜像本身有别的红（E 的树表读不出：I-2.1、I-7.4、I-4.8），只判 I-3.9 一格，不写「全绿」。
- 不能跳过的一点：造之前先核 E 与 A 共享的确是 u（比 A、E 的 extent 树根指针相同），不共享则这份镜像没有意义；这一步没做，推的。
- 变异：删掉守卫（不管后来的根走不走得全都判）⇒ 这份镜像上 I-3.9 由不适用变违例。

### 与底稿比

判据一致（底稿说法与我推的一致，模型证过零误红）；造法里的槽号、树形、条数过时（上）；「还缺的对照格」造得出。

### 推翻条件

- 收窄的守卫被推翻：找到一份合法镜像，某条已释放记录在守卫放行（比看得见的 L 晚的根都走得全）时仍判红。要么 `is_complete` 在某条路径上对一个走不全的根仍置真（漏了一种读不出的形状），要么候选集之外的根（F 之下、被抛弃）引用了这个落点而判据本来就只看候选集。
- 造法被推翻：翻映射根之后 A 的 `placements` 变了（少了单元），那份正例就不是「L 与 T 都对」，改翻别的 A 独有、不引用别的单元的节点。


## 判定一览

判定取「一致 / 冲突 / 规则没说」：一致 = 底稿这一格与今天条款、代码相符；冲突 = 两句并排见各节；规则没说 = 条款推不出（进不了不变量或交用户）。「推翻」列写什么现象会推翻这一格。

| 格 | 判定 | 一句话 | 推翻它的现象 |
|---|---|---|---|
| B1 出生树（数据指针等） | 规则没说（码 2） / 一致（码 1、码 3） | 新立 I-1.12，码 1、码 3 取头偏移 43，码 2 随 C289 不判，写进状态列、不登记 `part_of_the_range_not_judged` | 码 1 的头偏移 43 读法有反例，或 I-9.2 之外的指针出生树在合法镜像上与头不等 |
| B1 实例表指针 | 冲突（I-1.2 状态列写「不进这一条」，条款例外句只说不走谓词） | 出生 txg / 实例 / 尾段挂 I-1.2 遍历方向，把状态列收窄成只免谓词半 | 实例表单元的头与指针在合法镜像上不等（core 两边同变量） |
| B2 新根段与根 | 冲突 | 认「带末条标志那一条」，不认「计数器最大」；四段逐字节相同，新立 I-8.10；偏移过时（99 / 185 / 271 / 279，头 311） | 一次发布的末条读不出而根在案的合法镜像上，按末条标志判红 |
| B3a 诞生代号下界 | 冲突（出局） | 下界在回退复活后的合法镜像上误红（模型 379 条）；上界成立，另立 I-4.9 | 找到复活的单元诞生代号被重写、使下界成立的条款 |
| B3b 点名项 | 一致（判法）/ 冲突（偏移、载荷 CRC 偏移 91 → 95、「walk.rs 没读 named」过时） | 新立 I-8.11，出生树只判码 1、码 3 | 校验和相符的位置条目所指单元头与点名项字段在合法镜像上不等 |
| B3c 底稿推的两处 | 规则没说 | 把候选根当快照 S：上界成立、下界不成立；实例切换照旧：切换今天没实现，只有条款推演 | 切换实现后按 D23 369 行的下界推演出反例 |
| B4a previous_snapshot_txg | 一致（措辞）/ 冲突（槽号 50248 → 50252） | 非 inode 树条目恒 0 进（I-9.17）；inode 树 ≤ 根 txg 那半不进，交用户 | 条款补出 inode 树条目的取值 |
| B4b 记账代 | 一致（措辞）/ 冲突（槽号 50246 → 50250） | I-3.13；「走错」今天不存在；回退发布落盘行的代段没核 | 回退发布写的记账行代段不等于新 txg |
| B4c inode 改动计数 | 一致（上界）/ 规则没说（下界） | 改动计数 ≤ 容器诞生代号进（I-9.18）；下界要「创建算一次改动」，条款没有，交用户 | 合法镜像上一条记录改动计数大于所在容器诞生代号 |
| B5(a) 实例表行 T | 一致 | 新立 I-3.12：T < 引用这张表的根的 txg；依据改成 D18 306 行与 D23 369 行（QT4 已删） | 找到一行 T ≥ 引用它的根 txg 的合法镜像 |
| B5(b) 同实例行逐字节相同 | 规则没说（不立） | 条款没有；能造出两张表同实例行不同的历史（根槽读不出退一格）；候选集内是否恒同没证完，不进 | 证明候选集内两张表同实例行恒等 |
| B6 世代号 | 一致（奇偶）/ 规则没说（两槽都是 1） | 新立 I-7.15：g ≥ 2 时 g mod 2 == 槽号，两槽差不超过 1；「相等只许都是 1」依据只在字节表，是否进 D22 已定项 16 交用户；底稿说要改的交回形状今天已有 | 取号回卷写的不是最大自证世代号加一 |
| B6 根记录位置 | 一致 | 新立 I-7.16：区域 = txg mod R、槽 = (txg div R) mod S，第 0 代只判槽 0 | 找到 `target_for_publish` 之外的写根路径 |
| B6 journal 记录 txg | 一致（同实例不减）/ 规则没说（换实例严格大于） | 新立 I-8.12 同实例不减；换实例严格大于在设备失而复得下没核，不进 | 同实例里合法的 txg 回退序列 |
| B7 守卫收窄 | 一致 | 比 L 晚的候选根都走得全才判；模型 43302 情形零误红；造法过时（树多层、槽号、条数），翻映射根替代翻叶 | 守卫放行的合法镜像上判红 |
| B7 对照格 | 规则没说（底稿说造不出） | 造得出：A → 空发布 E → 覆盖写 B'，翻 E 独有的树表；没造没跑 | E 与 A 不共享数据单元 u |

## 没做什么

- 没编译、没跑 checker、没造任何镜像：所有「推得只红它」都是照代码读出来的，写作推的；两个纯 Python 小模型（B3、B7）在 `research/prompts/m3-inv-b-r1-sonnet-model/`，B7 这一次重跑输出与上一条腿落盘的相同。
- 没跑变异表、没跑 gate；变异行只写了「判定恒真」的替换点，没有写进 `crates/mutations.tsv`。
- 没有判别的腿的格（B1–B7 全归我，没有别的腿的格）；没替主 agent采纳任何一条：建议号（I-1.12、I-8.10、I-8.11、I-4.9、I-9.17、I-3.13、I-9.18、I-3.12、I-7.15、I-7.16、I-8.12）只是 `grep -rnF` 核过没用过的下一个号。
- 没读禁读清单里的文件（攻方腿产出与模型目录、交接摘要 opus 那份、并行几轮的产物）。
- 没核：B4b 回退发布落盘的记账行代段（`mount.rs:4931` 一带）；B6a 取号回卷的每个调用点；B6b 复制根到最新根会不会让择新出现同 txg 双根；B6c 换实例严格大于在设备失而复得下的真实可达性；B5(b) 按谱系的归纳。
- B1–B4 的引文是上一条腿写的，这一条只跑了 cite-check（末行见下），没有逐条重新对读。
