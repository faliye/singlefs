**出处 `.claude/kb/invariants.md:1-8`（整段抄，未转述）**

```markdown
# 不变量清单

**checker 是 invariants.md 的可执行形式。** invariants.md 每加一条，checker 就要加一个检查。
两者不同步的 commit 一律不收（见 [format-evolution.md](../rules/format-evolution.md)「硬约束」）。

每条不变量必须写成**可判定**的形式——能对着一个镜像回答"成立/不成立"。
写不成可判定形式的，说明还没想清楚，不许进 invariants.md。

```

**出处 `.claude/kb/invariants.md:9-20`（整段抄，未转述）**

```markdown
## 状态

**新池新建文件阶段（2026-09-14 起）。** 磁盘格式已落成字节：mkfs、新池新建文件的写路径与冷启动恢复在 `crates/singlefs-core`；checker 在 `crates/singlefs-checker`，与实现只共享常量模块（D13（验证路线） 已定项 5）。
池级 checker（`walk::check_pool_image`）判 65 条（数它的命令：`grep -c '^| I-.*已实现' .claude/kb/invariants.md`，与 `crates/singlefs-checker/src/image.rs` 的 `IMPLEMENTED_INVARIANTS` 长度相等；第一版 23 条，2026-09-16 里程碑「覆盖写、释放、回退与复用」步 3 加 I-3.8（实例表行唯一且低于挂载根），2026-09-17 步 6 加 I-7.4（近 K 代块未被复用） 与 I-4.8（近 K 代根校验和自洽），2026-09-18 加 I-3.9（释放代落在停止引用它的那一格区间里）、I-9.14（树表条目的诞生 txg 跨根不变） 与 I-5.4（分配记录罩住的槽互不相交），2026-09-21 加 I-1.8（归并后版本全序）、I-7.3（环健康性）、I-8.6（反向链算法） 与 I-8.7（实例内事务号不重号），2026-09-22 加 I-1.10（码 2 条目宽等于字段表宽）、I-9.6（水位大于两处最大号）、I-9.12（分隔 key 落在孩子区间之外）、I-1.2（块头写序已发布） 与 I-4.2（无被引用未提交块），2026-09-23 加 I-8.8（前缀里的事务不被切开） 与 I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号），2026-09-24 加 I-7.9（回退下界 F 不高于抬 F 的上限）、I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉）、I-3.11（已分配减 defer 等于最新根走读） 与 I-8.9（一次发布的记录序号连续且只有末条带标志），2026-09-25 加过 I-7.10（已停用·回退见证表删了） 与 I-7.11（已停用·见证择根删了）、2026-09-26 随回退见证删掉退役，同日加 I-7.12（系统配置 F 不低于同盘根上的 F），2026-09-27 加 I-1.11（映射 key 与单元头相符），同日加 I-7.13（系统配置池级字段在读者收的范围里） 与 I-9.16（树表条目按树 ID 严格升序且合发号次序），2026-09-28 加 I-7.14（系统配置本盘设备号等于盘在池里的身份），2026-10-02 加里程碑三二·1 那一批 16 条（I-1.12（指针出生树等于被指单元头）、I-3.12（实例表行 T 低于引用它的根的 txg）、I-3.13（记账行代段等于这一版的 txg）、I-4.1（重放后树可遍历）、I-4.9（非最新候选根引用的单元不晚于它）、I-7.5（根槽按判定宽度对齐）、I-7.15（各盘所选系统配置槽设备数相同）、I-7.16（系统配置槽世代号与槽号的关系）、I-7.17（根槽位置等于 txg 算出的位置）、I-8.1（环几何够大）、I-8.5（tail 槽有效集非空）、I-8.10（记录新根段与同代根相同）、I-8.11（点名项与被指单元头相同）、I-8.12（同实例记录的 txg 不减）、I-9.25（树表条目的 previous_snapshot_txg 取值）、I-9.26（inode 改动计数落在出生代与容器诞生代号之间））），状态列写「已实现」的就是这 65 条：每条配一份改坏了正好触发它的镜像（`crates/singlefs-harness/tests/checker_known_bad_images.rs`），层 0 的每个崩溃状态都跑它。
I-7.7（系统配置实例代号不低于根环） 2026-09-14 按 C322（取号那一步的屏障怎么放没有条款） 定案改成 ① ②，层 0 里 0 个状态判红；I-3.1（已分配统计对得上） 与 I-7.8（根记录树 ID 水位不低于全池最大树 ID） 的 checker 读法 2026-09-14 由用户收尾弹窗定甲，写在各自那一行。
其余条目仍写「未实现」：它们要覆盖写、释放、回退与复用（释放、重用、残留）、journal 一侧的那一类检查、快照或加密，新池新建文件的镜像上判不出东西。

违例位是 128 位（4 个 u32 字，`crates/singlefs-checker-tier/src/crash_judge_gpu.rs` 的 `POOL_CHECKER_VIOLATED_BIT_CAPACITY`），位 i 对 `IMPLEMENTED_INVARIANTS` 的第 i 条（`pool_checker_violated_bits` 读成 u128）；今天用了 65 位，`POOL_CHECKER_COVERED_INVARIANT_INDEXES` 列满 65 个下标，编译期断言它与 `IMPLEMENTED_INVARIANTS` 等长、按下标次序一条不缺（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 11 行判定的位宽加宽与 GPU 内核补齐）。

<!-- invariant-count -->
现共 104 条在用（编号至 I-9.27（树表条目的头 ID 取值），另有六条退役，编号不回收）。这一句是当前条数的权威登记位；「## 历史版本」里那几句「现共 N 条在用」记的是当时的数，不跟着改。

```

**出处 `.claude/kb/invariants.md:21-47`（整段抄，未转述）**

```markdown
## I-1 块自描述 —— 类
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-1.1 | 块头自述逻辑地址 | 任一单元，其头记录的**类内身份**与实际引用一致——数据单元的身份是五元组（锚点偏移即它的逻辑地址）；索引节点是树 ID + 层级（+ 该树已定携带的 key 区间，D18（块里携带什么信息）已定项 2 口径）；打包记录单元（码 3）是（出生树 ID, 打包记录类型, 容器号, 容器出生代）四元组，「父级」按打包记录类型分——类型 1 走容器索引的身份引用（C103（容器索引无落点）），类型 2 走 inode 树内部节点条目的身份引用（I-9.2（条目身份与子头相符）），**类型 4（实例表）走根记录的物理指针，不属于任何树、不进容器索引**。**自证单元（journal 记录 / 根槽 / 系统配置槽 / 类型 4 实例表单元）显式豁免**：无父引用无树身份，自证纪律在 I-7（根环）/ I-8（journal）两类与 D22（单元原子性怎么合成）已定项 21；多层码 2 树（记账树、中央映射树）的索引节点，身份里的层级与 key 区间按引用它的父条目核：层级 = 父 − 1、区间落在父条目给的那一段、内部节点区间是子树覆盖区间；按位置寻址的两棵树（分配记录树、extent 树），节点头里的层级与 key 区间等于它的位置规定的那一层、那一段；内部条目的 key 是某个孩子那一段的起点，孩子按位置严格递增；分配记录叶里每条记录的起点与末槽都落在叶罩的那一段里；extent 上段叶条目的 inode 号、下段叶记录的单元号落在叶罩的那一段里，上段叶条目的标签只能是 0、1、2（标签 0 载荷全零、标签 1 载荷末 2 字节为零）；根之下没有空节点（这一半 checker 由实二五补）。 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判；`walk::Walk::walk_code_two_subtree`；坏镜像用例 `the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root`；`crates/singlefs-checker/src/position_addressed.rs`） |
| I-1.2 | 块头写序已发布 | 任一单元，其头记录的写序 (i, n)（D18（块里携带什么信息）已定项 7 / 11 的字段）与诞生代号 b 按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的全局已发布谓词判成已发布：i < 挂载根实例且无该实例的行 ⇒ 已发布，有行 (i, T_pub, W) ⇒ 码 1 要 b ≤ T_pub ∨ n ≤ W、码 2 / 3 要 b ≤ T_pub（所选根那个实例的 T_pub = 重放之后那个根的 txg，严格介于所选根的实例与新实例之间、又不是切换照旧事务写序主人的实例写 (i, 0, 0)，切换的所选根按 D23（journal 的角色与格式） 已定项 14 取被重发的在飞 checkpoint 所基于的根；W = **属于实例 i 的**、被这次重放施加的最大事务号，切换时是被照旧的、写序属于 i 的最大事务号；同一次恢复写出的那批行里只有所选根那个实例的 W 可能非 0，同一次切换写出的那批里只有被照旧事务的写序主人的 W 可能非 0，表上允许有多行非 0，它们来自不同次的恢复或切换；2026-09-14 用户定案，C330（中间实例那一行的 T_pub 取所选根的 txg） 三轮三方）；i == 挂载根实例 ⇒ b ≤ 挂载根的 checkpoint_txg；i > 挂载根实例 ⇒ 判损坏。**例外**：打包记录类型 4 的实例表单元不走这条谓词，它的现行版本由根记录持有的物理指针唯一确定。**判不成已发布者必是未发布的撕裂事务垃圾或被抛弃时间线的残留**——这是扫描方向独立于遍历方向判它的依据（D13（验证路线） 已定项 12 双方向要求）；2026-09-05 前它只比诞生代号与当前 txg，崩溃后 txg 重发让那条判据对同 txg 两条时间线失效（E104（扫描重建的现行版本判定）：无实例表时崩溃世界复活 6） | 已实现（2026-09-22，池级 checker `walk::check_pool_image` 的 `judge_birth_identity_of_a_referenced_unit`，**只判遍历方向那一半**：走读到的每个被引用单元，头里的诞生代号 b、写序的实例代号 i 与尾段（码 1 是写序的事务号 n、码 2 / 码 3 是出生序号）要与**引用它的那条指针**记的相同——不同则「头记录的写序」不是这个单元的真实身份，谓词判的就是假输入；口径同恢复路径今天的前置检查（`crates/singlefs-core/src/recovery.rs` 五处报 I-1.2（块头写序已发布） 的 `InvariantViolated` 里「出生身份与指针不符」那三处；「诞生于根之后」那两处是谓词那一半，归 I-4.2（无被引用未提交块）），checker 按字段表自己解一份。⚠️ **扫描方向那一半（「任一单元」都按谓词判、i > 挂载根实例 ⇒ 判损坏）对着一份崩溃镜像判不了，没写**：① 判不成已发布本身不是违例——本行判据自己写着那是未发布的撕裂事务垃圾或被抛弃时间线的残留；② 「i > 挂载根实例 ⇒ 判损坏」只在恢复刚发布完新根的那一刻成立，而上一次挂载取了号、写出单元、一个根都没发布就崩时，盘上合法地留着 i > 最新根实例的在飞单元，那一支今天由 I-7.7（系统配置实例代号不低于根环） ① 按各盘系统配置的实例代号兜着。谓词本身照旧在用：`PublishedPredicate` 过滤 I-1.8（归并后版本全序） 的射程、判 I-4.2（无被引用未提交块）。打包记录类型 4 的实例表单元按本行的**例外**只免谓词那一半：根记录偏移 170 的实例表指针与链上每一片，指针里的出生 txg、实例代号、尾段与被指单元头里的值相等，仍挂这一条的遍历方向（已实现：`walk::Walk` 的 `judge_birth_identity_against_the_pointer`，被引用单元与实例表链上每一片都判；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 3 行）。坏镜像是 extent 树根头里的出生序号 +1、引用它的指针一个字节不动（`crates/singlefs-harness/tests/checker_known_bad_images.rs`）；层 0 每个崩溃状态都跑它，走读一个被引用单元都读不到的状态报「不适用」） |
| I-1.3 | 块头树 ID 一致 | 任一单元，其头记录的所属树 ID 与实际引用它的树一致。中央映射树不进树表，「实际引用它的树」取根记录（或记录新根段）里中央映射树根指针带的出生树，不写死 15：回退到无文件那一版之后再发第一个文件版本，八棵树从水位起重新发号（D8（核心索引结构） 已定项 8 ②），映射树的号随之变（主 agent 2026-09-23 定，C511（回退到无文件那一版之后诞生代怎么接） 第 3 步实现员查出写死 15 的读法在合法的回退历史上误红）。自证单元（含类型 4 实例表单元）与 I-1.1（块头自述逻辑地址）同口径显式豁免。⚠️ 打包记录单元（码 3）与被快照共享的数据单元头里的树 ID 是出生树，跨头共享时与引用树不同，那一情形的判法随 D9（加密） 那次「绑出生快照」的重开（C110（跨头共享与加密后元数据类的块头一致性读法））。这一射程覆盖码 3 全部打包记录类型，含类型 2（inode 树的叶，D8（核心索引结构） 已定项 6）：克隆头共享 origin 的叶时，叶头的出生树是 origin | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-1.4 | 块头 fsid 一致 | 任一单元，其头记录的 fsid == 系统配置 fsid（D18（块里携带什么信息）已定项 7 的字段；加密开启后与 KDF 绑定冗余，接受）。scrub 半边「本卷全部单元归属指向本卷」靠它才写得出（C74（无加密期的跨卷错向写没防线）补救的另一半） journal 记录头 2026-09-14 起也带 fsid 8（系统配置 fsid 低 8 字节，D23（journal 的角色与格式） 已定项 4）：fsid 不符的记录整条不进重放前缀、按断链处理（E142（新池新建文件的干跑） 第七次跑记的 G25） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-1.5 | 系统配置记管道身份 | 系统配置记录写出该镜像的管道身份（实现标识 + 版本），且非空。**字段 2026-09-14 用户定案有了**（D22（单元原子性怎么合成） 已定项 9）：自举头段「写入者身份 20」= 实现标识 16 字节 ASCII 零补齐 + 版本 4 字节 u32，第一版写 `singlefs-rs` / 1；实现标识全零判红 | 未实现（条款已写、检查仍欠） |
| I-1.6 | 类标签登记且三处相等 | 任一单元，共同前缀偏移 6 的单元类型标签 ∈ D18（块里携带什么信息） 已定项 11 的登记表（码 0 与未登记码即判损坏）；偏移 7 的 8 个 flag 位第一版恒 0，非 0 一律拒收（前向兼容政策，D15（格式冻结政策） 判定表）；数据单元（码 1）五元组首字节 == 偏移 6；打包记录单元（码 3）类身份段首字节 == 偏移 6 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-1.7 | 打包容器合法与判定顺序 | 任一打包记录单元（码 3）：记录数 × 记录宽 ≤ 声明长度且声明长度 ≤ 32768 − 136（含 29 字节预留位的头，D18（块里携带什么信息） 已定项 16；2026-09-14 预留位从 28 加到 29，`cap(136, 40) = ⌊32632 / 40⌋ = 815` 与按 135 算同值，40 是第一版墓碑记录宽，D18（块里携带什么信息） 已定项 24），**先于**载荷校验和判（D23（journal 的角色与格式） 已定项 1 的 B 条同一纪律）；判定顺序恒为头校验和 → 类身份段 → 记录宽 == 登记表为该打包记录类型登记的宽（类型 2 是 140，D8（核心索引结构） 已定项 6；类型 1（墓碑）是 40、类型 5（谱系）是 16，D18（块里携带什么信息） 已定项 24；不等判损坏，只需头字段）→ I-1.2（块头写序已发布） / I-1.3（块头树 ID 一致） / I-1.4（块头 fsid 一致） → 边界 → 载荷校验和 → 逐条记录，任一步失败即中止；打包记录类型 4 的两种 kind 记录另按 D18（块里携带什么信息） 已定项 11 的实例表字段表判，不套用「记录宽 == 登记表的宽」那一步（一片记录宽恒 88，条数上限由片容量给——D18（块里携带什么信息） 已定项 11 2026-09-13 从 64 改宽）（对象级的判定——I-9.7（记录三段恒零）、I-9.8（记录 inode 等于 key）——在这条中止链之外，只让那条记录 EIO）；容器头声明一种打包记录类型、一棵出生树；打包记录类型 ∈ 第二级登记表，未登记类型的容器其物理指针不得出现在任何回收 / 复用操作的输入集合 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判；类型 1、类型 5 两种记录宽未实现，三·3 实现） |
| I-1.8 | 归并后版本全序 | 码 1 / 3 的可读已发布单元按「类身份段全部字段含写序」归并成组（字段表在 D18（块里携带什么信息） 已定项 7（码 1）与已定项 11（码 3）；归并键取这些字段但去掉载荷校验和那一段——它正是 ①「同一组的成员载荷相同」要比的那个量，进了键 ① 就恒真）之后，同 key 的各组两两全序键不等——码 1 的键是写序，码 3 的键是 (诞生代号, 实例代号, 事务号) 三元——诞生代号打头（同一实例连着两个 checkpoint 都不分配新事务号时两版写序逐字节相同），事务号留在末位做同一 checkpoint 内的平局破除（固定点会迭代，同一容器在一个 checkpoint 里不保证只写一次）；同一组的成员载荷相同（码 3 由载荷校验和判、码 1 由载荷 CRC 判），不同即判损坏、不择（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3 副本归并，射程 w = 2；码 2 不进这条） | 已实现（2026-09-21，池级 checker `walk::check_pool_image` 判 ① 同一组的成员载荷相同；② 同 key 的各组两两全序键不等**只判码 3**——码 1 的键停着等 C464（I-1.8 对码 1 的全序键写窄了） 定案，checker 里显式写成 `TotalOrderKey::UndecidedForDataUnit`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-1.9 | 清扫水位内无可读垃圾 | 清扫水位（D5（快照 / 空间记账机制） 已定项 4 第 13 项）以内不存在可读垃圾；其外的可读垃圾每个 key 都被更大写序的可见版本或可见墓碑压住（管理员回退之后的镜像不在此列，见 C615（扫盘重建对挂着时的回退没有输入），出处 `research/prompts/m3-tomb-r1-main-verification.md` 第二节；C113（扫描重建时多版单元的现行版本判定无输入） 定案 P4；扫描重建的产物未晋升期间清扫禁止，这条在重建之后失效） | 未实现 |
| I-1.10 | 码 2 条目宽等于字段表宽 | 任一码 2 索引节点，头里自述的**条目宽**（D18（块里携带什么信息） 已定项 7 的自描述三段之一：key 宽 1 + 条目数 2 + 条目宽 2，偏移表在 D18（块里携带什么信息） 已定项 18、载荷布局在 D8（核心索引结构） 已定项 11）**等于**它那棵树登记的条目字段表宽度：extent 叶记录 112、inode 内部条目 120、分配记录 20、记账条目 34、记账树内部节点 108（key 22 + 子指针 86）。条目宽是**盘上读来的值**，`index_node_view` 只判了它 ≥ key 宽（extent 24、inode 8、分配记录 10、记账 22），够不着「装得下整条记录」——不判就按字段表的固定偏移切，切片当场越界（增补 3 第 6 件 panic 面普查第二族，C476（盘上内容可控时走得到的 panic 有 21 处） R1 / R3 / R4 / R13）。条目宽为 0 时条目数必须为 0（`声明长度 = 条目数 × 条目宽` 在宽 0 时恒成立，挡不住它），否则判该节点损坏（2026-09-27 主 agent 定，池级判定在 A3-checker-2 里做）。判定顺序与 I-1.7（打包容器合法与判定顺序） 同一纪律：**先于**按字段表解条目。⚠️ **中央映射树不判**：它的条目宽由 C307（映射树两种 key 宽怎么装进一棵定宽 key 的树） 定，那一条还开着；**树表条目（200 字节）两边都没写**，而走读照样按固定偏移切树表节点的条目——要不要把射程扩到它，账在 C476（盘上内容可控时走得到的 panic 有 21 处） | 已实现（2026-09-22，池级 checker `crates/singlefs-checker` 的 `walk::walk_tree_table_entry`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；条目宽为 0 时条目数非 0 那一条 2026-09-27 实审 A3-checker-2 落地：`index_node_view_judging_a_zero_entry_width`（`crates/singlefs-checker/src/walk.rs:747`）在解码失败理由是 `Verdict::EntryWidthZeroWithEntries`、且这棵树登记了条目字段表（`EntryFieldTableRegistration::Registered`）时报违例；树表条目与中央映射树不在射程内，只记走读失败；坏镜像 `crates/singlefs-harness/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs`） |
| I-1.11 | 映射 key 与单元头相符 | 任一中央映射条目，key 的类标签属于 D18（块里携带什么信息） 已定项 11 的登记表，且等于它的位置条目指的单元头里的类标签；key 里的出生 txg、实例代号、尾段（码 1 是写序的事务号 6 字节；码 2 / 码 3 是出生序号 4 字节，其后 2 字节补零恒 0）分别等于那个单元头里的诞生代号、写序实例代号、事务号或出生序号；key 里的出生树等于那个单元头里的出生树（码 1 / 码 3 取类身份段偏移 43，码 2 取哪个字段随 C289（码 2 的出生树取哪个字段没写） 定）。不等判损坏。 | 已实现（2026-09-27，池级 checker `walk::check_pool_image`，判定在 `crates/singlefs-checker/src/walk.rs` 的 `walk_central_mapping_entries`；key 各段的偏移按 D19（块指针的结构与宽度预算） 已定项 6 / 已定项 10 的字段表另写一份，不用实现的解析；判到的射程：类标签在登记表里且等于被指单元头的，出生 txg / 实例代号 / 尾段等于单元头的，码 2 / 码 3 尾段后 2 字节补零恒 0，码 1 / 码 3 的出生树等于单元头偏移 43，码 2 的出生树随 C289（码 2 的出生树取哪个字段没写） 不判；映射根全零（无文件的一版）时报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（映射里数据单元那条 key 的出生 txg 3 → 4）与 `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs`。编号已落到代码：`crates/singlefs-checker/src/image.rs:38` 的常量 `MAPPING_KEY_MATCHES_THE_UNIT_HEADER` 现在写的值就是这个编号本身，I-1.11（映射 key 与单元头相符）） |
| I-1.12 | 指针出生树等于被指单元头 | 任一被走读的指针，偏移 34 的出生树等于被指单元头里的出生树：码 1、码 3 取头偏移 43；码 2 的出生树取哪个字段随 C289（码 2 的出生树取哪个字段没写） 定，不判（只写进状态列）。指针各段落点依据 D19（块指针的结构与宽度预算） 已定项 8（指针携带被指单元映射 key 除类标签外的各段）。inode 内部条目到叶容器那一类由 I-9.2（条目身份与子头相符） 罩住，不重判。 | 已实现（2026-10-02，池级 checker `walk::check_pool_image` 走读里的 `judge_pointer_birth_tree_against_the_unit_header`，只在两处调用：实例表链上每一片、extent 数据单元；码 1、码 3 取头偏移 43，码 2 不判，码 2 那一支不登记成「判了一部分」，那会让每份干净镜像整条报不适用；没有走到这两处指针的镜像报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；对应的账是 C294（指针里的 key 各段与被指单元头相等没有不变量）；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 3 行） |

> I-1（块自描述） 类的适用范围是**任一单元**（数据与元数据都在内，
> D20（承重面：单元的原子性与自包含） 口径），判定字段的落点与宽度在
> [decisions.md](decisions.md) D18（块里携带什么信息） 已定项 7 的字段表（码 1 / 2）与已定项 11 的类身份段表（码 3）；
> 自证单元（journal 记录 / 根槽 / 系统配置槽）对 I-1.1（块头自述逻辑地址） 与
> I-1.3（块头树 ID 一致） 显式豁免，豁免的依据是它们的自证纪律另有落点
> （I-7（根环） / I-8（journal） 两类与 D22（单元原子性怎么合成） 已定项 21）。
> ⚠️ **豁免行是穷举的，按 D18（块里携带什么信息） 已定项 11 的登记表**：码 1 / 2 / 3 在字段表里有类身份段，码 16 / 17 / 18 在豁免行里点名，
> 其余码即判红（I-1.6（类标签登记且三处相等））——这是「穷举的显式分支」作用在不变量射程上。

```

**出处 `.claude/kb/invariants.md:48-82`（整段抄，未转述）**

```markdown
## I-7 根环 —— 类

**依赖 D22（单元原子性怎么合成），形态未定。**
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-7.1 | 根槽有效集非空 | 全部根槽位中，**自证校验和通过**的记录集合 S 非空 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-7.2 | 最新根可完整遍历 | S 中代号最大者，其 checkpoint 树根指针出发按 D4（校验和位置） 递归校验，能走完全部权威态（[decisions.md](decisions.md) D21（权威态与派生态的分界） 定义：单元 + 记账），无一处失配 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-7.3 | 环健康性 | **环健康性**：S 中除代号最大者外，至少还存在一条更早代号的记录。不存在即判红——它意味着某次提交把上一代直接覆盖了，轮换逻辑已失效，**下一次撕裂将无路可退**。**例外只有一个**：S 全部是第 0 代（mkfs 把第 0 代种进全部区域，D22（单元原子性怎么合成）已定项 8）时判绿，崩溃后回退到这个态的镜像同样判绿。⚠️ 射程：种子没被覆盖完之前，它们自己就充当「更早代号的记录」，连续两代落进同一槽的轮换 bug 要等 R × S 次发布把种子全部覆盖之后才会被这一条抓到 | 已实现（2026-09-21，池级 checker `walk::check_pool_image`，例外（自证过的根全部是第 0 代）照判据原样实现；坏镜像建在 mkfs 刚写完那份上（三条第 0 代根的 checkpoint_txg 一起改成 1），例外那一支另有一份阳性对照；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-7.4 | 近 K 代块未被复用 | 回退候选集里每一个根（按实例表判仍然有效 ∧ txg ≥ F_生效，D23（journal 的角色与格式） 已定项 14）所引用的块，其物理范围均未被重新分配给其他对象、也未被清扫抹头。**「K 代」2026-09-13 起就指这个候选集**：平时回退下界 F 不动时是整个根环，盘紧时抬 F 缩到最近 4 个可退到的状态、且每块盘上最新的持久有效根都在内（D16（发布语义） 已定项 1）；原「K 是运行时策略、下限 ≥ 2」作废。[decisions.md](decisions.md) D22（单元原子性怎么合成）：**根环深度是块重用延迟的上界**（D22（单元原子性怎么合成）同时明令块复用的界不能是格式常量），不满足则环里的旧根是假的回退候选。「有效」按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3 的回退候选集判：(i, T) 有效 ⟺ 实例表无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti——被抛弃时间线的根不在候选集里，这只管回退目标的选择；被抛弃时间线今天只由崩溃恢复造出（管理员回退是挂着时的一次向前发布，不抛弃任何根，D23（journal 的角色与格式） 已定项 14）。它们引用的块在离开根环之前同样不许重新分配、不许抹头（2026-09-13 用户定案，C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元） 取影子账，D23（journal 的角色与格式） 已定项 14；留着的理由是第一轮判决 H6：崩溃恢复落到旧根之后，被抛弃的根引用的单元被复用、之后恢复落在它上面读不出）。⚠️ F_生效 按 D16（发布语义） 已定项 1「生效」取根上与系统配置里读得出的最大值；checker 今天只读根上带的 F，带 F 的根全坏时会在合法状态上误红这一条与别的按回退候选集判的几条（第三轮判决 Y7，推的，没实现），欠 C556（checker 与层 0 不读系统配置里的 F）。崩溃后 defer 队列的内存态丢失，接手的是清扫准入「释放代 ≤ max(F_生效, 环里最旧有效根)」那条闸（谓词由 D16（发布语义） 已定项 1 给，2026-09-13；有效性按实例表判），保护宽度随之变 | 已实现（2026-09-17，池级 checker `walk::check_pool_image`，按每条回退候选集里更早的根各判一格：走读它引用的单元时校验和对不上或头用不了，就是它指着的块被复用或抹头；坏镜像是 mkfs 树表单元（只有第 0 代根还引用）改内容重封；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） 被抛弃、还在根环里的根也判（2026-09-27，用户定「checker 现在就补被抛弃根那一半」，代码审阅第 7 条）：最新根的实例表有行时，根环里每条被判抛弃的根（有行 (i, Ti, ·) 且 T > Ti）各用一份自己的走读走一遍，只看读回来对不对——走出 I-2.1（校验和与内容匹配） 违例或走读失败就按这条根判红，判定丢掉、引用不进 I-3.1（已分配统计对得上） 的并集（`walk::judge_blocks_referenced_by_abandoned_roots`，用例在 `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs`）；根环有读不出的槽且最新根的实例表有行时，没有违例就整条报不适用、不报成立；C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 那一形在这一半上照实红，等 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 修好转绿。 |
| I-7.5 | 根槽按判定宽度对齐 | 每个根槽的**起始偏移**是判定宽度的整数倍，判定宽度 = max(声明的 `physical_block_size`, 声明的 `io_min`)（`io_min` 为 0 视为没声明）；mkfs 在任何写之前先拒不按这个宽度对齐的几何，非常规几何由 mkfs 拒、不再是合法镜像。**逐槽判，不是只判第 0 个**——偏移半个槽的槽照样跨 2 个原子单元（E41（根环槽几何 2×4×256） 实测：对齐时风险槽数 1，偏移 256 时是 2），撕裂会连带打掉相邻的槽而没有任何东西报警 | 已实现（2026-10-02，池级 checker `judge_root_slots_aligned_to_the_decision_width`（`crates/singlefs-checker/src/image.rs`）逐槽判：判定宽度取各盘所选系统配置声明的 max(`physical_block_size`, `io_min`) 里最大的（异构池取最大，`research/prompts/m3-inv-a-r1-main-verification.md` 第四节用户定案第 1 条），是声明值、注明「声明值，未探测」，探测比对仍欠 C388（checker 的判定宽度不自己探测）；mkfs 在任何写之前拒不按这个宽度对齐的几何（`crates/singlefs-core/src/make_filesystem.rs` 的 `first_slot_not_aligned_to_the_decision_width`）；异构池一份与逐槽判的坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 5 行） ⚠️ **射程注（2026-09-13，D2（RAID 条带策略） 已定项 15）**：`io_min` 大于 `physical_block_size` 时按 `io_min` 判——已定项 17 收窄后根槽与系统配置槽在它的射程内。 |
| I-7.6 | 根环区域落在互不相同的盘上 | R 个区域**在 mkfs 时逐区域写下的设备身份**（D2（RAID 条带策略） 已定项 7 用户定案逐字「归属不再按 `(r × P) mod devs` 算，mkfs 时逐区域把设备身份写下来」）取满 min(R, devs) 个不同值，且没有一块盘背超过 ⌈R / devs⌉ 个区域（鸽笼下界，D22（单元原子性怎么合成） 已定项 14，2026-09-13 用户定案）：devs ≥ R 时就是两两不同；devs < R 时不拒绝挂载，容错承诺按盘数降级（E87（固定结构的放置） 量过的 2 盘 R = 3：掉盘 1 最坏 1 代、掉盘 0 最坏 2 代）。取值不满或某盘超过下界即**拒绝挂载**——名义容错 F 个失败域而实际区域挤在同一块盘上，是 E47（根环失败域的损失） 实测过的形态（`even_span` 在 18 个几何格里只有 3/18 真的落到 4 块不同盘） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-7.7 | 系统配置实例代号不低于根环 | 可写挂载独占打开成功的那个集合里（它至少是过半，过半是准入门槛不是写入范围），每块盘的系统配置实例代号按它两槽里全部自证过（校验和过且 fsid 与本池相同）的槽读，取号用同一个读法（2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款） 三轮三方）：① fsid 与本池相同的每个根记录、journal 记录、单元写序的实例代号 ≤ 各盘系统配置实例代号的最大者，任一根环区域、journal 记录槽、单元头读不出时这一半报「不适用」；② 各盘的实例代号（每盘取它全部自证过的槽里最大的那个）不等时，较大者不出现在任何根记录、journal 记录、单元写序里，镜像记不下这次挂载独占打开的集合时报「不适用」，第一版两盘就是全部盘。取号没做完、全或无失败回卷期间的代号不等由 ② 判成立，不另开例外（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的取号规则）。只在 devs ≥ R 时根环才是系统配置之外的独立见证，第一版 R = 3 > devs = 2 时两路见证的失败域是同一对盘 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） ⚠️ **2026-09-14 按 C322（取号那一步的屏障怎么放没有条款） 用户定案改成 ① ②**：checker 读每盘两槽里全部自证过的系统配置，扫描方向加读 journal 环与三类单元头的写序；层 0 全量 262165 个状态 I-7.7（系统配置实例代号不低于根环） 违例 0（旧定义「各盘择到的相等且 ≥ 根环」下取号只落一块盘的 2 个状态判红）；坏镜像三份：没人引用的实例 2 孤儿（①）、journal 环里实例 2 的记录（①）、盘 0 最大号 2 而孤儿带 2（②） |
| I-7.8 | 根记录树 ID 水位不低于全池最大树 ID | **根环里全部根记录的「树 ID 水位」字段取 max，必须 > 该池中出现过的最大树 ID。**「出现过的」由 checker 全盘扫描算出：全部单元头的索引节点类身份段里那个树 ID（D18（块里携带什么信息） 已定项 7），并上树表条目里的树 ID；**不许与运行时共用同一段代码**（D13（验证路线） 已定项 5）。取 max 的范围是「根环里**全部**根记录」不是「回退候选集」——被抛弃时间线的根仍然承载它那一代发过的号，形态与 D23（journal 的角色与格式） 已定项 14 给 checkpoint_txg 取值那句同源。**判别力**：抓两类——① 某次发布没更新水位；② 水位被实现成「本 checkpoint 发出的最高号」而不是**累计高水位**（后者的镜像是连发 R + 1 个 checkpoint 各建一棵树，第一棵的号会随根轮出环而被重发）。⚠️ **只许写成不等式**：水位是「下一个可用号」的上界，写成等式会把正常镜像判红 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） 崩溃孤儿两条流（第一个文件版本那次发布的每个前缀 → 可写挂载 → 挂载之后六种动作）在 `crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs`；中间实例行（T = 0）不排除的坏镜像是 `checker_known_bad_images.rs` 的 `published_nodes_behind_an_intermediate_row_still_count_against_the_tree_identifier_watermark`（实例 2 的根槽全清零之后实例 3 写中间实例行 (2, 0, 0)，根环水位压低之后只红 I-7.8（根记录树 ID 水位不低于全池最大树 ID）；回退行那一道随回退行删掉，D18（块里携带什么信息） 已定项 11）；实现在 `walk::written_after_its_instance_was_last_applied` ⚠️ **checker 读法（2026-09-14 用户收尾弹窗定甲；2026-09-24 主 agent 按代码三方 `research/prompts/m2-wave3-code-r1-main-verification.md` 第四节第 1 条补第二道，被攻过零轮）**：崩在发布之前的那个事务写出的孤儿节点不算「出现过」。扫描方向不数两类码 2 节点：① 诞生代号超过根环里最大 checkpoint_txg 的；② 写序实例 i 在最新根指着的实例表里有行 (i, Ti, ·)、Ti > 0、而诞生代号 > Ti 的——行说实例 i 只被施加到 Ti，之后写的没发布过；下一个实例第一次发布取的 txg 可以等于孤儿的诞生代号（D23（journal 的角色与格式） 已定项 14 注 3 取等号），只靠 ① 挡不住（第一个文件版本崩在记录落盘之前、再可写挂载的两条流上 130 格在合法状态上判红）。Ti = 0 的行（恢复写的中间实例行）不排除：中间实例发布过的号仍要算（D8（核心索引结构） 已定项 8 ②）。节点头 fsid 不是本池的照旧数。按字面数孤儿，层 0 全量 262165 个状态里 261891 个判红（水位 11 而盘上已有树 ID 11–15 的孤儿节点），见 records 2026-09-13-总审核 十一·七 ⚠️ **判法补一句（已实现：`walk::check_pool_image` 末尾在取 max 那一格之后再判最新那条根自己的水位）**：另判最新那条根的水位不低于全池最大树 ID。只取根环里全部根的 max 判不到把最新那条根的水位压低的镜像：攻方量到 checker 49 条 0 违例，而 core 的 `recover` 报 I-7.8（根记录树 ID 水位不低于全池最大树 ID） 失败；core 重发树 11..18 的状态 checker 也 0 违例。用攻方那两份坏镜像钉住（只压低最新那条根的水位；建一次树之后经会话入口覆盖写 24 次把发树那条根轮出去、再把环里每条根的水位压到 11，只红它），被攻过零轮（`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 9、10 行） |
| I-7.9 | 回退下界 F 不高于抬 F 的上限 | **抬 F 的那一条根**（它带的 F 比同一实例里 txg 比它小的最新那条根带的高；新实例的第一条根没有同实例的前一条，不算抬——它带的是恢复算出的 F_生效；管理员回退是现行实例里的一次向前发布，带的 F 与前一条相同，不算抬；一次抬 F 推的几次空发布只有第一条算）根记录里那个回退下界 F（D22（单元原子性怎么合成） 已定项 7 的字段），按这条根 flags 里的**卸载记号**（D22（单元原子性怎么合成） 已定项 7 flags 位 0）分两支判：**带卸载记号的**，F 不高于根环里 txg 比它小的最新那条根的 txg（正常卸载把 F 抬到现行那一版的 txg、不另判上限，D16（发布语义） 已定项 1「抬 F 的上限」）；**不带的**（准入抬 F），F 不高于**用它之前的根算出的** D16（发布语义） 已定项 1 的准入抬 F 上限：min(每块盘上最新的持久有效根的 txg, 第 4 新的非空持久有效根的 txg)，非空有效根不足 4 个时取最旧有效根的 txg。有效 = 按那条根自己指着的实例表判仍然有效 ∧ txg ≥ 抬之前的 F；「之前的根」取根环里 txg 比它小的那些。只判抬 F 的那一条，不在后来每张镜像上重算：F 抬过之后再回退，后来的根会把当初撑着 F 的非空根判成无效，现算的上限会低于 F，而 F 不回落（用户定案 2026-09-24，理由见实二报告：按「有效根不要求 txg ≥ F」的读法，固定用例 `rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version` 与两个随机种子在合法状态上判红）；非空 = 它的树表里 inode 树、extent 树的根指针与它前一条有效根（按 txg 排）树表里的不同——比的是树表里这两棵树的条目，不比树表单元自己的落点（D16（发布语义） 已定项 1 表格后那段「「非空」从盘上怎么认」）；数「第 4 新的非空」时去重：从新到旧数，一条非空根只有在它的 (inode 树根指针, extent 树根指针) 与每一条比它新、已计入的根都不同时才计入（同一段）。⚠️ 它判的是一张镜像上 F 有没有抬过头，不判 F 该不该抬（那是准入的事，D28（挂载期承诺量） 已定项 1）。⚠️ **卸载记号盘上没有别的东西核它**：准入入口误打记号、F 又抬过头，按带记号那一支放过（第三轮判决第二节 K6：(b) 丢的正是「准入抬过头却带了卸载记号」那一格）——这一格归层 0 录制流另核「卸载记号只由卸载入口打」（C557（卸载记号只由卸载入口打没有检查）），不归这一条。⚠️ F 写进了系统配置而带新 F 的根没落盘或坏了、恢复后生效值怎么取的那一类，归层 0 与 C556（checker 与层 0 不读系统配置里的 F），不归这一条。算上限要比的有效根树表读不出时，它空不空判不了、不猜：F 不高于把这些根全算空得出的上限就成立，高于全算非空得出的上限就违例，落在两者之间这条根不判 | 已实现，两支（不带记号那一支 2026-09-24、带卸载记号那一支 2026-09-26）：池级 checker `walk::check_pool_image` 的 `judge_rollback_floor_raises_against_their_ceilings`，根环里每条抬 F 的根各判一格，按根记录读出的卸载记号分支（checker 读成 `UnmountMarkerView`）。**不带记号那一支**：坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant`（步 5 那段历史上抬到上限 11 与之后再发一版都成立、抬到 12 只红它）；固定用例 `rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version` 与随机历史各档不红；层 0 `crash_enumeration_fixed_script_stream.rs` 抬过 F 的三条流要求真被评估过；`crates/mutations.tsv` 7 条变异（判法拿掉、上限取大、第 4 新改第 3 新、有效按新 F 取、不限同一实例、一条都不认、「4 个不同状态」不去重）；用户 2026-09-24 定义，里程碑「覆盖写、释放、回退与复用」收口表第 26 行。**带卸载记号那一支**：坏镜像同一文件的 `marked_root_raising_the_floor_above_the_previous_root_reddens_only_the_rollback_floor_invariant`（正常卸载之后的镜像按带记号那一支判 6 ≤ 6 成立、按准入那一支会红；把那条带记号的根的 F 改成 7 只红它）；变异「checker 把每条根都读成不带卸载记号」「带记号那一支把不高于前一条根的 txg 判成低于」；记号的写读在 `crates/singlefs-core/src/root_record.rs`（写者照 `unmount_marker` 写 flags 位 0、读者拒记号以外的位），写者、读者各有一条变异（实二） |
| I-7.10 | 已停用·回退见证表删了 | **此编号不再使用**——2026-09-26 管理员回退改成挂着时的一次向前发布，系统配置里的回退见证表删掉（D23（journal 的角色与格式） 已定项 14、D22（单元原子性怎么合成） 已定项 9）⇒ 这一格**无对象**。曾要求：每块盘两槽里自证过的系统配置槽，槽内偏移 481 起的回退见证表解得开（条数不超过 R × S − 1、条目严格升序、目标实例代号小于新实例代号、条数之后全 0），同一个新实例代号在各盘各槽里记的回退目标相同 | 不适用 |
| I-7.11 | 已停用·见证择根删了 | **此编号不再使用**——2026-09-26 回退见证表删掉（D23（journal 的角色与格式） 已定项 14），择根不再跳过被见证抛弃的根 ⇒ 这一格**无对象**。曾要求：所选根是根环里自证过、不被回退见证表抛弃的根里 (txg, 实例代号) 最大的那一条；根环里至少一条自证过的根不被见证表抛弃；见证表里新实例代号不大于所选根实例代号的每一条，所选根指着的实例表罩得住它 | 不适用 |
| I-7.12 | 系统配置 F 不低于同盘根上的 F | 每块盘分开判：这块盘两槽里自证过（校验和过且 fsid 与本池相同，与取号同一读法，D18（块里携带什么信息） 已定项 11）的系统配置槽中回退下界 F（D22（单元原子性怎么合成） 已定项 9 的字段）的最大值，不低于这块盘根槽里每一条自证过的根带的 F（D22（单元原子性怎么合成） 已定项 7 的字段）。抬 F 那一串先把新 F 写进每块盘的系统配置、过一道屏障，才发第一条带新 F 的根（D16（发布语义） 已定项 1「抬 F 那一串」），所以任何合法状态上一条根带的 F 都已经在它所在那块盘的系统配置里。某块盘两槽都自证不过时，那块盘不判；一块盘都判不了时整条报「不适用」。判别力：把「先写系统配置」挪到第一条根之后，崩在两者之间的状态必须红 | 已实现（2026-09-26，池级 checker `walk::check_pool_image` 的 `judge_system_configuration_floor_against_the_roots_on_each_device`：每块有自证过的系统配置槽的盘判一格，那块盘上一条根都没有时那一格成立；两槽都自证不过的盘不判，一块盘都判不了时整条报「不适用」。坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `one_device_whose_system_configuration_floor_is_below_a_root_on_it_reddens_only_the_system_configuration_floor_invariant`：步 5 那段历史上抬到上限 11 的镜像一条都不红、I-7.12（系统配置 F 不低于同盘根上的 F） 真被评估过且成立，盘 1 两槽的 F 改回 0 只红它、红在盘 1 txg 16 那条带 11 的根上；变异「不比系统配置里的 F（每块盘恒成立）」「checker 不读系统配置里的 F（读成 0）」（实二）。条款仍是主 agent 2026-09-26 按 SysPre 的写序推的，没三方；层 0 每个崩溃状态都跑池级 checker，按判法每个状态都评估得到它（推的，层 0 没跑） |
| I-7.13 | 系统配置池级字段在读者收的范围里 | 任一自证过（magic 与整槽校验和对，`check_system_configuration_slot`）、incompat 位认得的系统配置槽（字段住 D22（单元原子性怎么合成） 已定项 9 的字段表，读者收的界在 D22（单元原子性怎么合成） 已定项 2 与已定项 16）：格式版本 = 1（`SYSTEM_CONFIGURATION_FORMAT_VERSION_THIS_CHECKER_READS`）、加密类型 = 0（第一版恒关，`SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFF`）、固定结构槽距 ≥ 4096 字节（`FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES`）且槽 1 整槽落在根环基址之前、`physical_block_size` ∈ [457（`ROOT_RECORD_BYTES`）, 槽距]、journal 环长 ÷ 4096 ÷ F（`JOURNAL_SAFETY_FACTOR`）≥ 1 且环末端不越过单元区起始槽号（`geometry_of` 的 `fixed_structure_slot_spacing_lies_in_the_format_range` / `physical_block_size_fits_a_root_slot` / `journal_ring_bytes_lie_in_the_supported_range`）。任一盘任一槽不满足其中一项即判红：checker 报违例、不作保，且该池其余不变量一律报「不适用」，与实现整池拒绝挂载一致（用户 2026-09-27 定系统配置越界整池拒）。**判的字段与 core 收同一张表**：格式版本、加密类型、槽距（上界用同一槽自述的根环起点）、根槽宽、环长、单元区起始槽号（偏移 417）= 环长现算的起点且在 64 槽段边界上、journal 环起点（偏移 325）与根环起点（偏移 371）等于第一版常量，共八项（D22（单元原子性怎么合成） 已定项 16 第 1 条）。R（区域数）与 S（每区槽数）越界不在这一条里，仍归「这一槽不可择」 | 已实现（2026-09-27，池级 checker `crates/singlefs-checker/src/image.rs` 的常量 `SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS` 登记这个编号、列进 `IMPLEMENTED_INVARIANTS`；判定 `judge_system_configuration_values_the_reader_accepts` 在 `walk::check_pool_image` 里调用，任一盘任一槽带越界值时该池其余不变量整批报「不适用」并提前返回。**八项 checker 与 core 两边都判**：checker 解槽时判格式版本与加密类型（`crates/singlefs-checker/src/lib.rs`），其余六项在 `geometry_of` 判——先判 journal 环起点（偏移 325）= `JOURNAL_RING_START_SLOT`（1024）、根环起点（偏移 371）= `ROOT_RING_BASE_SLOT`（64）（两个起点先于槽距判：槽距上界按同一槽自述的根环起点算），再判槽距、根槽宽、环长，最后判单元区起始槽号（偏移 417）= 同一槽自述的环起点 + ⌈环长 ÷ 16384⌉（`unit_area_start_slot_is_the_slot_after_the_journal_ring`）且是 `CLUSTER_SEGMENT_SLOTS`（64）的整数倍；新加的四判各报一个 `Verdict` 成员（`JournalRingStartSlotNotTheFirstVersionConstant`、`RootRingBaseSlotNotTheFirstVersionConstant`、`UnitAreaStartNotTheSlotAfterTheJournalRing`、`UnitAreaStartOffTheClusterSegmentBoundary`），都归 `value_refused_by_this_reader`（整池不作保）；常量取自 `singlefs-format`，算式 checker 自己写。core 一侧 `crates/singlefs-core/src/system_configuration.rs` 的 `parse_slot` 读出偏移 325、371 两个字段，`crates/singlefs-core/src/recovery.rs` 的 `system_configuration_values_this_reader_accepts` 按格式版本、加密类型、槽距（上界用同一槽自述的根环起点，`largest_fixed_structure_slot_spacing_under_the_root_ring_base`）、根槽宽、根环起点、journal 环起点、环长、单元区起点的次序判，不收的值报 `SystemConfigurationValueOutsideWhatThisReaderAccepts` 的一个成员（两个起点是 `RootRingBaseNotTheFirstVersionSlot`、`JournalRingStartNotTheFirstVersionSlot`），经 `RecoveryFailure::SystemConfigurationValueRefused` 整池拒。坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs`：`a_system_configuration_slot_whose_encryption_type_is_on_reddens_only_its_own_invariant_and_every_other_is_not_applicable`（盘 1 槽 0 加密类型改成 1、重算整槽校验和，只红 I-7.13（系统配置池级字段在读者收的范围里）、其余不变量全报不适用），`each_system_configuration_geometry_field_the_reader_refuses_reddens_only_its_own_invariant_naming_the_refused_field`（几何字段各一格，各只红 I-7.13（系统配置池级字段在读者收的范围里）、违例说明点名被拒的那个成员）。两边收同一张表由 `crates/singlefs-harness/tests/system_configuration_fields_core_and_checker_refuse_the_same_table.rs` 钉住：新池新建文件之后的镜像上两块盘四个系统配置槽改同一个几何字段、整槽校验和重封，五格各一条用例，每格 core 的只读与可写挂载都拒成同一个 `SystemConfigurationValueRefused` 值、两块盘逐字节不变，checker 报 I-7.13（系统配置池级字段在读者收的范围里） 违例、其余不变量报不适用；根环起点写 0 那一格两边点名的成员不同（core 先判槽距上界、报槽距，checker 先判根环起点、报根环起点），照两边各自的判定次序钉（用户 2026-09-27 定「读字段，不等于常量就整池拒」，`records/2026-09-24-里程碑二收尾调度.md` 第 293 行；代码三方第二轮判决 `research/prompts/m2-closeout-code-r2-main-verification.md` 第一节「core 与 checker 对系统配置几何字段的读法」那一格）|
| I-7.14 | 系统配置本盘设备号等于盘在池里的身份 | 每块盘两个系统配置槽里，每个自证过（magic 与整槽校验和对）、fsid 与本池相同的槽，自举头里的本盘设备号等于这块盘在池里的身份。字段住 D22（单元原子性怎么合成） 已定项 9 的系统配置字段表（自举头「本盘设备号 4」）；实现一侧是 D18（块里携带什么信息） 已定项 11 里挂载核盘表那一句：可写挂载比盘表（`crates/singlefs-core/src/mount.rs` 的 `device_table_disagreeing_with`，`OwnDeviceNumberDiffersFromTheIdentityHandedIn`）。别的池的槽归 I-1.4（块头 fsid 一致） 判，不在这一条里 | 已实现（2026-09-28，池级 checker `crates/singlefs-checker/src/image.rs` 的常量 `SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY` 登记这个编号、列进 `IMPLEMENTED_INVARIANTS`；判定 `judge_own_device_number_is_the_device_identity` 在 `walk::check_pool_image` 里调用，每块盘每个自证过、fsid 属于本池的系统配置槽判一次，本盘设备号与 `ImageReader::devices` 给的这块盘的身份比。坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs`：两块盘四个系统配置槽的本盘设备号互换，`devices_whose_own_device_numbers_are_swapped_redden_only_the_own_device_number_invariant` 钉只红 I-7.14（系统配置本盘设备号等于盘在池里的身份），这份坏镜像也在 `the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target` 的清单里；变异表 `crates/mutations.tsv` 里对着这一判的三行（判定恒真、池级走读不调这一判、坏镜像写回原值）改法 A 的 `prove-red.sh` 各抓到（`research/prompts/m2-impl-r2-fixes-a-implementer-report.md` 第 77–78 行）。合法历史上不判违例：门禁 harness-model-differential-and-scenarios 随机历史那一趟 I-7.14（系统配置本盘设备号等于盘在池里的身份） 判绿 4155 次、不适用 0 次（同一报告第 189 行）；层 0 快档与全量没跑）|
| I-7.15 | 各盘所选系统配置槽设备数相同 | 各盘所选的系统配置槽（自证过、世代号最大者）里的设备数相同（D22（单元原子性怎么合成） 已定项 9 的字段）。不要求等于盘表：降级镜像（掉了一块盘）里 checker 手里的盘数小于设备数是合法的。形态抄 I-1.4（块头 fsid 一致） 的逐盘循环；core 的 `choose_system_configuration` 在几块盘的设备数对不上时报 `SystemConfigurationsDisagree`、整池挂不上，两边同表。 | 已实现（2026-10-02，池级 checker `walk::check_pool_image` 逐盘把各盘所选槽的设备数同第一块盘所选槽的比；坏镜像是盘 1 两个槽的设备数改成 3 并重封，只红它，在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；对应的账是 C466（几块盘的系统配置设备数对不上时 checker 判不出）；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 7 行） |
| I-7.16 | 系统配置槽世代号与槽号的关系 | 每块盘上，任一可择的（自证过、池级字段在读者收的范围里）系统配置槽 i：世代号 ≥ 2 时世代号 mod 2 == i；两个槽都可择时世代号相差不超过 1，相等时只许都是 1（mkfs 写下的两槽世代号都是 1，D22（单元原子性怎么合成） 已定项 16）。带越界值的槽归 I-7.13（系统配置池级字段在读者收的范围里） 整池拒，不进这一条。 | 已实现（2026-10-02，池级 checker `judge_system_configuration_slot_generations_follow_the_rotation`（`crates/singlefs-checker/src/image.rs`）：奇偶一格、差值一格，差值那一格取「两槽都是 mkfs 写的 1，或相差恰为 1」；差值与奇偶各一份只红它的坏镜像，在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 7 行与第四节用户定案第 1 条） |
| I-7.17 | 根槽位置等于 txg 算出的位置 | 根环里任一自证过的根所在的槽位 (区域 r, 槽 s)：`checkpoint_txg` ≥ 1 时 r == txg mod R 且 s == (txg div R) mod S；`checkpoint_txg` == 0（mkfs 把第 0 代种进全部区域，D22（单元原子性怎么合成） 已定项 8）时只判 s == 0，不判 r。公式是 D22（单元原子性怎么合成） 已定项 16 的定案。根环补洞判决要的「槽位与 txg 对得上」就是这一条（`research/prompts/m3-ring-r1-main-verification.md` 第二节第 6 行）。 | 已实现（2026-10-02，池级 checker `judge_root_records_live_in_the_slot_their_txg_names`（`crates/singlefs-checker/src/image.rs`）：根环里每条自证过的根逐条判，第 0 代只判槽 0；公式同 core 的 `target_for_publish`（`crates/singlefs-core/src/root_ring.rs`）；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 8 行） |

> ⚠️ **I-7.3（环健康性） 的价值在于它不需要真的崩溃一次才能发现 bug。**
> 轮换逻辑写错（例如连续两代落到同一槽位）在**下一次正常提交后的 checker 扫描**里就会被抓到，
> 而不是等到某次撕裂真的发生、发现无处可退时才暴露。
> 这与「校验和会不会红」是两类判别力：前者查机制活着没有，后者查数据对不对。

> ⚠️ **I-7（根环） 类验不了「失败域独立」这个假设。** 根环的多区域布局依赖
> 「盘首与盘尾的物理故障 / 掉电撕裂事件统计独立」，而对着镜像判不出这一条——
> 它只能验证「机制按设计跑了」，不能验证「设计的假设是真的」。
> 该假设**本机无证据**，见 [decisions.md](decisions.md) D22（单元原子性怎么合成） 已定项 2。

```

**出处 `.claude/kb/invariants.md:83-124`（整段抄，未转述）**

```markdown
## I-8 journal —— 类

**依赖 D23（journal 的角色与格式），形态已定（定长环 + tail 住系统配置槽），checker 未实现。**
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-8.1 | 环几何够大 | 每块盘每个自证过的系统配置槽里三个字段各占一格：journal 在飞记录数上限 = ⌊环长 ÷ 4096 ÷ F⌋，journal 最坏占用 = 在飞上限 × 4096，安全系数 F = `JOURNAL_SAFETY_FACTOR`（格式常量，checker 与 core 都判等于，不是下界），任一不等判红；另判一格：环里同一实例、同一 `checkpoint_txg` 的自证记录（同一次发布）不多于在飞记录数上限条。`环大小 ≥ F × 任一事务的最坏 journal 占用` 是这几格的推论，不另判。mkfs 时算、mount 时复算比对，core 挂载同判。三格任一不等时与 I-7.13（系统配置池级字段在读者收的范围里） 同口径整池不作保：checker 报这一条违例，其余不变量一律报「不适用」，与实现整池拒绝挂载一致；I-7.13（系统配置池级字段在读者收的范围里） 与这一条同时不成立时走 I-7.13（系统配置池级字段在读者收的范围里） 那一支（只红 I-7.13（系统配置池级字段在读者收的范围里），这一条报不适用）；同一次发布记录数那一格不触发整池不作保。 | 已实现（2026-10-02，池级 checker：三个字段等式在 `judge_journal_ring_geometry_fields_agree`（`crates/singlefs-checker/src/image.rs`），每块盘每个可择的系统配置槽各判三格：F 等于 `JOURNAL_SAFETY_FACTOR`，在飞上限等于环长 ÷ 4096 ÷ 该槽里的 F 取整（F 为 0 判红、不除），最坏占用等于在飞上限 × 4096（乘法溢出判红）；带越界值的槽归 I-7.13（系统配置池级字段在读者收的范围里）、不可择的槽不读几何；同一次发布的记录数不多于在飞上限那一格在 `walk::judge_records_of_one_publish_within_the_in_flight_record_limit`，取所选槽记的上限。F 那一格 2026-10-03 从「F ≥ 2」改成等于格式常量，三格不等的整池不作保那一支加在 `walk::check_pool_image` 里 I-7.13（系统配置池级字段在读者收的范围里） 那一支之后、I-8.5（tail 槽有效集非空） 之前（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 3、4 行）。坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（`each_ring_geometry_and_root_slot_alignment_bad_image_reddens_only_its_own_invariant`：F 取 1、2、4 与只改没被择的那一槽 F 各一份；`journal_ring_geometry_fields_that_disagree_redden_only_the_ring_geometry_invariant_and_every_other_is_not_applicable`：三格不等时其余不变量报不适用）。core 挂载同判：`crates/singlefs-core/src/recovery.rs` 的 `journal_ring_geometry_fields_recompute_from_the_ring_length`，放在 `system_configuration_values_this_reader_accepts` 的最后。在飞上限那一格复算取槽里的 F、core 取格式常量，F 不等于格式常量时两边都已因 F 那一格拒，不影响判红与否；对应的账是 C28（环几何不变量未实现）；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 4 行） ⚠️ **口径注（D28（挂载期承诺量） 已定项 4）**：ckpt_cost 按当时各记录树的高现算、挂载期内会长；实现上最坏占用 = 在飞上限 × 4096，由环长与 F 倒算，取的是上界、不随树高变（`research/prompts/m3-inv-a-r1-main-verification.md` 回看决策那一行）。 |
| I-8.2 | 记录头不跨原子单元 | 任一 journal 记录的头完整落在一个 `physical_block_size` 单元内。该宽度**运行时探测**，不许硬编码 | 未实现 |
| I-8.3 | 重放前缀严格连续 | 被重放的记录集合，其 `jsn` 严格连续（`jsn == expected`，断号即止；链从所选根覆盖的最后一条记录之后接，跨实例边界即停，记录水位按实例代号为主比——D23（journal 的角色与格式） 已定项 14 的注，2026-09-13），且整段落在在飞记录数上限之内。⚠️ **这一条不是前缀纪律的全部**：D23（journal 的角色与格式） 已定项 14 的前缀口径共五条，这里只落了其中三条半——**「提交标记齐全的事务才施加」与「施加前逐项验证点名单元」两条不在这条判据里**，而两者的处境不同：前者 2026-09-23 已落成 I-8.8（前缀里的事务不被切开），checker 判定与崩溃点重放那一半仍欠，记在 C37（事务与记录的对应无定义）；后者**已经有了**，只是不住这一条——它住 D16（发布语义） 已定项 7，两条故障注入自证 2026-09-14 在层 0 做出（C76（发布时序没有权威落点））。⇒ 照这一条实现恢复施加逻辑的人会漏掉那两道防线，而它们正是挡住「半个事务」与「点名单元已损坏的记录」的 | 未实现 ⚠️ **判据补一句（2026-09-13，D16（发布语义） 已定项 4）**：前缀不得停在一次发布的中间——一次发布整体施加或整体不施加。 ⚠️ **再补一句（2026-09-14，D23（journal 的角色与格式） 已定项 18 用户定案）**：「在飞记录数上限」的语义就是**这个前缀最多 N 条**（N = 环槽数 ÷ F），不是 XFS 那种「最后 N 条内算撕裂、之外算损坏」；**任一校验和不过的记录一律断链即止**——停在它之前那一条，第一版不区分撕裂与损坏、不因此拒绝挂载。 |
| I-8.4 | 重放幂等 | 任一 journal 记录被重放两次与重放一次，产生的读出内容相同（等价关系取「读出的内容相同」，不比扇区、实例代号与行） | 未实现（实现在途：取 R-换，在崩溃注入的二次崩溃上判恢复加写行那一整段；被攻过零轮，实现时先在崩溃注入上量误红；`research/prompts/m3-inv-a-r1-main-verification.md` 第四节用户定案第 2 条） |
| I-8.5 | tail 槽有效集非空 | 池里至少有一块盘的两个系统配置槽里有一个槽自证过（magic 与整槽校验和对）：tail 住这些槽（D23（journal 的角色与格式） 已定项 3），这个集合就是全部 tail 见证者。全池皆废判红（mkfs 没做完、写到系统配置之前失败的盘面也是全池皆废，照判红，故障注入的已知红清单认这一类，用户 2026-10-03 定）；单盘全废不判违例，走旁路报告：另报哪块盘少于两槽自证过、可写挂载会拒。对着只读恢复判。自证过而不可择的槽（R / S 越界、incompat 位不认得）与带越界值的槽都算自证过；I-7.13（系统配置池级字段在读者收的范围里） 带越界值整池拒、或 I-8.1（环几何够大） 三格不等整池不作保的那一支里，这一条不判、报「不适用」。 | 已实现（2026-10-02，池级 checker `judge_some_device_carries_a_self_verified_system_configuration`（`crates/singlefs-checker/src/image.rs`）：把「不自证」（magic 或整槽校验和不对）与「自证过而不可择」分成两类，后一类计入自证过（`slot_verifies_itself`），别的不变量在全池皆废时照旧报不适用；单盘全废走旁路报告 `system_configuration_redundancy_report`，不进违例通道；自证过而不可择的四槽一条都不红的一份坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（`self_verified_system_configuration_slots_no_reader_can_choose_still_witness_the_tail`）；对应的账是 C461（checker 判「挂不上的镜像」整片报不适用）；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 6 行与第四节用户定案第 3 条） |
| I-8.6 | 反向链算法 | 任一 journal 记录的反向链 = CRC32C(**本实例内逻辑前一条**记录的 311 字节头，`header_csum` 那 32 字节按零参与)；**本实例写出的第一条记录反向链恒 0**（D23（journal 的角色与格式） 已定项 10 / 已定项 19，2026-09-13 用户定案，链按实例算是 2026-09-14 用户定案，头宽随 D23（journal 的角色与格式） 已定项 4 现行 311）。不等的记录不进重放前缀。jsn 的计数器为 0 的记录判违例：它没有本实例内的逻辑前一条，反向链无从定义（D23（journal 的角色与格式） 已定项 18 的落点式 `(计数器 − 1) mod 槽数 × 4096` 从计数器 1 起；用户 2026-09-27 定）。⚠️ 链按实例算，所以跨实例边界的两条记录之间**不判**这一条——实例边界由实例代号挡着（I-8.3（重放前缀严格连续)） | 已实现（2026-09-21，池级 checker `walk::check_pool_image`，逐盘判：计数器 1 与跨实例边界的第一条判反向链恒 0，同实例前一条在盘上时判链值，前一条不在盘上不判；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 里环还空着的 4 个状态报不适用，其余每个崩溃状态都判）；计数器 0 判违例（2026-09-27，`walk.rs` 的 `ExpectedBackChain::CounterZeroHasNoLogicalPredecessor`，用例 `a_journal_record_whose_counter_is_zero_reddens_the_back_chain_invariant` 在 `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs`） |
| I-8.7 | 实例内事务号不重号 | 判环里全部自证过的记录，同一实例按 `jsn` 的计数器看：① 按计数器相邻的两条非 0 记录，事务号不减（事务号 0 是不承载事务的空发布记录，不进这个比较）；② 一个事务号的记录在计数器上连续，被别的记录（含事务号 0）隔开之后不再出现。D23（journal 的角色与格式） 已定项 7 逐字：「事务号按实例计数、从 1 起」「记录按事务号顺序追加……这是实例表行的 W 能当精确前缀的依据」；同一项定一个事务可以跨多条记录，所以相邻两条可以同号。 | 已实现（2026-09-21 立时判「非 0 事务号严格递增」；2026-09-23 按 D23（journal 的角色与格式） 已定项 7 改成不减加同号连续。池级 checker `walk::check_pool_image` 的 `judge_transaction_numbers_per_instance`：逐盘按实例判，同一实例按计数器相邻的两条非 0 记录，号不等时后一条严格大于前一条（①），号相等时两条之间不许坐着同一实例的别的记录（②，事务号 0 的也算；中间只有读不出的槽或别的实例的记录不判红）；「被别的非 0 号隔开之后又出现」一定伴着一处 ① 红，不另判；同一实例非 0 的记录不足两条时报不适用。坏镜像三份（交错两份、事务号倒退）在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，每份只红这一条，违例说明标出红的是 ① 还是 ②） |
| I-8.8 | 前缀里的事务不被切开 | 判环里全部自证过的记录：③ 同一实例同一事务号的一组记录里，**带提交标记的至多一条，且它是这一组计数器最大的那一条**；提交标记那一字节只许 0 或 1；④ **一组里一条提交标记都没有时，这个实例在它后面不许再有任何记录**（事务号 0 的空发布也算；别的实例的记录不算）：D23（journal 的角色与格式） 已定项 7 逐字写「一个事务在发出单元写之后失败即实例切换、号更大的记录一条都不追加」，实例切换之后这个实例不再写任何记录。它与 I-8.7（实例内事务号不重号） 的 ① ② 合起来，保证 D23（journal 的角色与格式） 已定项 7 原话那个恢复算法「丢掉提交标记还没出现的那个事务的全部记录」**恰好丢掉、且只丢掉**被崩溃切开的那个尾巴；`crates/singlefs-core/src/recovery.rs` 今天那一步写成「按 jsn 扫到第一条不带提交标记的记录就停」，只对一事务一条成立，一事务跨多条记录时要改，归并行线一。四条里任一不成立，恢复要么丢掉一个已经提交的事务、要么施加半个事务，而盘上没有任何痕迹。⚠️ **今天可达镜像上它恒真**：第一版一条记录一个事务、`is_commit` 全仓没有一处写 false，所以 ③ ④ 今天都不会被可达历史违反。**判别力只能靠坏镜像**，坏镜像不做这条就是恒真的检查——恒真的检查比没有检查更糟。⚠️ **它管的是条款允许的形态，不是第一版的形态**：D23（journal 的角色与格式） 已定项 7 定的是「可以跨多条，记录头带事务边界字段」，而第一版按 C310（事务切分纪律与记录数口径打架） 一条记录一个事务。I-8.8（前缀里的事务不被切开） 挡的是**一事务跨多条记录落地那一天**。⚠️ **它只判一份镜像上的记录流自洽，不判「施加的结果等于整数个事务」**：后者要拿「这次事务该有的全部效果」当 oracle，一份镜像上判不出来（形态同 I-4.3（提交原子） 那一行），归层 0 崩溃点重放。 | 已实现（2026-09-23，池级 checker `walk::check_pool_image` 的 `judge_commit_markers_per_transaction`：逐盘，一组 = 同一块盘上实例代号与事务号都相同的非 0 记录；③ 两条以上的一组里带提交标记的至多一条、且是计数器最大那条；提交标记字节每条记录都判（事务号 0 的也算），不是 0 或 1 即红；④ 一条提交标记都没有的组，后面同实例还有任何一条记录（事务号 0 的空发布也算）即红。没有 ③ 的对象也没有 ④ 的对象 ⇒ 报不适用，提交标记字节单独成立不算成立；第一版一事务一条、每条都带提交标记，层 0 每个崩溃状态都判、全部报不适用。阳性对照（一事务两条、提交标记在 jsn 最大那条，I-8.7（实例内事务号不重号） 与 I-8.8（前缀里的事务不被切开） 都成立）与四份坏镜像（提交标记放错位置、没提交的事务后面还有号更大的记录、没提交的事务后面跟一条空发布、提交标记字节写成 2，每份只红这一条）在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，违例说明逐条标出红的是哪条判据） |
| I-8.9 | 一次发布的记录序号连续且只有末条带标志 | 同一实例、同一 `checkpoint_txg` 的记录（一次发布写出的全部记录）：「本次发布内序号」依次是 1..N、与 jsn 同步（jsn 连号时序号也连号），记录标志位 0 恰好只在序号 N 那一条上为 1，记录标志其余位为 0；序号 0、一次发布之内跳号、没有末条或多于一条末条，都是违例。只判读得出的记录，读不出的那条不判（D23（journal 的角色与格式） 已定项 4、17，用户 2026-09-24 定，`records/2026-09-24-里程碑二收尾调度.md` 第三节「序号与末条标志的不变量」） | 已实现（`crates/singlefs-checker/src/walk.rs` 的 `judge_publish_ordinals_and_last_record_flags`；阳性对照与七份坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`） |
| I-8.10 | 记录新根段与同代根相同 | 某块盘 journal 环里自证过的记录 J，带末条标志（记录标志位 0 = 1），且这块盘上同（实例代号, `checkpoint_txg`）带末条标志的记录只有这一条；根环里任一自证过、实例代号与 `checkpoint_txg` 都与 J 相同的根 R，J 的新根段四段与 R 的同名四段逐字节相同：新根段 [0..86) = R[36..122)、[86..172) = R[256..342)、[172..180) = R[122..130)、[180..188) = R[130..138)（树表单元指针、中央映射树根指针、树 ID 水位、回退下界 F，D23（journal 的角色与格式） 已定项 15）。只认带末条标志那一条；同（实例, txg）有两条以上不同内容的根不判，归 C88（根环的时间线判别未实现）；末条标志不唯一归 I-8.9（一次发布的记录序号连续且只有末条带标志）。 | 已实现（2026-10-02，池级 checker `walk::judge_new_root_segment_of_the_last_record_against_the_roots`：逐盘按 (实例, txg) 分组，带末条标志恰一条才判，与根环里同 (实例, txg) 的每条自证过的根比四段，一对都没比到报不适用；坏镜像取记录一侧，把 F 改一位并重封头校验和，另有树表指针、映射根指针、树 ID 水位各一份，在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 4 行） |
| I-8.11 | 点名项与被指单元头相同 | 任一自证过的 journal 记录里任一点名项的每条位置条目：若它指着的那一份单元的整单元校验和与位置条目里的相符，则单元头里的类标签、诞生代号、key 尾段与点名项偏移 28、37、45 的同名字段逐字节相同（码 1 尾段 = 写序实例代号 4 + 事务号 6，码 2 / 码 3 尾段 = 实例代号 4 + 出生序号 4 + 补零 2，补零恒 0）；出生树（点名项偏移 29）只判码 1、码 3，等于头偏移 43，码 2 随 C289（码 2 的出生树取哪个字段没写） 不判。校验和不相符的位置条目不判（落点被合法复用之后，单元头就不是它点名的那个单元了）。 | 已实现（2026-10-02，池级 checker `walk::judge_named_entries_against_the_unit_headers`：点名项每条位置条目，所指那一份整单元校验和相符才比，不符不判；出生树只判码 1、码 3；一条都没比到报不适用；类标签、出生树、写序实例代号、key 尾段各一份只红它的坏镜像，在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 2 行） |
| I-8.12 | 同实例记录的 txg 不减 | 同一块盘 journal 环里自证过的记录，同一实例代号的记录按 jsn 计数器从小到大，`checkpoint_txg` 不减。换实例后严格大于不判（设备失而复得下没核）。 | 已实现（2026-10-02，池级 checker `walk::judge_journal_record_txg_per_instance`：逐盘、逐实例按计数器看 txg 不减，换实例不判，没有可比的一对时报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 9 行） |

> ⚠️ **D23（journal 的角色与格式）的死锁 3 全靠 I-8.1（环几何够大），而 I-8.1（环几何够大） 的 checker 未实现。**
> 「写进本清单」不等于「有东西在拦」——取定长环那条论证依赖的是**拦得住**，不是**写下来**。
> 欠检查见 [checks-owed.md](checks-owed.md) C28（环几何不变量未实现）。

> ⚠️ **I-8.4（重放幂等） 是被实验逼出来的，不是设计时想到的。**
> [experiments.md](experiments.md) E24（恢复算法：先信 tail 会不会丢记录）测出：
> 全环扫描式恢复虽然不丢数据，却会把**陈旧 tail 以来的记录全部重放一遍**
> （窗口为空时仍重放满环 64 条）。没有幂等，「陈旧 tail 只是多做功」
> 就变成「重复施加已生效的记录」。

> ⚠️ **I-8.3（重放前缀严格连续） 与 I-8.1（环几何够大） 不许互相顶替。**
> E24（恢复算法：先信 tail 会不会丢记录）测到窗口撑爆环时（100 条 > 64 槽）
> **两种恢复算法一起丢**——恢复算法救不了几何。

> ⚠️ **I-8.4（重放幂等） 的射程有一条已实测的边界（E78（重放的起点），2026-09-02）**：
> 幂等管「同一条记录重复施加」，**管不到「旧记录的点名块已被合法复用」**——
> 陈旧 tail 的重放窗口远长于 I-7.4（近 K 代块未被复用）扣块的窗口，
> 窗口内记录的点名块换过内容后，施加时验证（E77（发布的持久顺序）证明它是承重步骤）会失配。
> 「陈旧 tail 只是多做功」因此只在不验证或有重放下界时成立；
> 重放下界的两条出路（根带 jsn 水位 / 盲放加尾删）见
> [checks-owed.md](checks-owed.md) C77（重放起点未定义）。

```

**出处 `.claude/kb/invariants.md:125-135`（整段抄，未转述）**

```markdown
## I-2 校验和 —— 类
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-2.1 | 校验和与内容匹配 | 任一被引用的块，其校验和与内容匹配 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判；2026-09-17 起「被引用」按 I-3.1（已分配统计对得上） 那个候选集里的根算：F 之下的根引用的单元可以已被回收复用，不在它们上判——不抬 F 就回收复用时 A 的根还在候选集里、这一条红，是 C22（刚释放的块立即重分配） 的必红） |
| I-2.2 | 校验和算法全盘一致 | 校验和算法标识在整个文件系统内一致。**字段 2026-09-14 用户定案有了**（D22（单元原子性怎么合成） 已定项 9）：系统配置自举头段「校验和算法标识 1」，登记表 0 = 无效、1 = CRC-32C，第一版写 1；读到 0 或未登记的码一律拒绝挂载。算法的四个参数写死在 D18（块里携带什么信息） 已定项 17 | 未实现（条款已写、检查仍欠） |
| I-2.3 | 补齐字节为零且参与校验和 | 任一数据单元（码 1）或打包记录单元（码 3，D18（块里携带什么信息） 已定项 11），其声明长度（净荷字节数；补齐起点 = 头 + 声明长度）之后到单元末尾的补齐字节**恒为 0**，且校验和 / MAC 覆盖**整个单元含补齐**（[decisions.md](decisions.md) D4（校验和位置） 已定项 2：不足一个单元的 extent 补齐到 32 KiB；码 3 由头校验和（到头末尾）与载荷校验和（头之后到单元末尾）两段合起来覆盖整个单元）。补齐字节非零、或校验和只覆盖声明长度，一律判红。索引节点（码 2）从 2026-09-13 起同样在射程内（D18（块里携带什么信息） 已定项 18 用户定案）：声明长度 = 条目区字节数，之后到节点末尾的补齐字节恒为 0，载荷 CRC 覆盖 [86 + 2 × key 宽, 16384)——起点是明文头末尾，含 29 字节预留位、条目区与其后的补齐区，口径与码 1 的 105、码 3 的 107 同（预留位 2026-09-14 从 28 加到 29，含预留位的头是 115 + 2 × key 宽）。⚠️ **加密开启时判的是解密后的明文，不是盘上字节**（2026-09-07 用户定案，还清 C189（I-2.3 在加密开启时判明文还是盘上字节没定））：补齐在盘上是 `E(0)` 而不是 0，按盘上字节判必然全红；checker 本来就要解整单元，取值同一步就有。**与 D18（块里携带什么信息） 已定项 7 的载荷 CRC 方向相反是对的**——载荷 CRC 要给无密钥一侧用，所以算盘上字节；这一条只有 checker 用，所以判明文 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-2.4 | 头校验和覆盖范围 | 任一单元的头校验和覆盖从偏移 0 到该类明文头的最后一个字节，端点写死：码 1 到偏移 104、码 2 到偏移 85 + 2 × key 宽、码 3 到偏移 106（D18（块里携带什么信息） 已定项 7 / 已定项 11 / 已定项 18 的字段表）；头校验和字段自身按 0 参与。**29 字节 nonce / MAC / 算法类型预留位虽然算进头（已定项 16）、也是明文，但不在头校验和覆盖内**——MAC 在头校验和之后才算得出来，罩不住它；单元头的这 29 字节与指针头部的 MAC 16 + nonce 12（D19（块指针的结构与宽度预算） 已定项 3）由同一条规则守：加密关着时恒 0、读者遇到非 0 一律判该结构损坏（已定项 17；指针头部那一半 2026-09-27 主 agent 定，池级判定在 A3-checker-2 里做），加密开启后由 MAC 自己罩（2026-09-14 用户定案把预留位从 28 加到 29，多的那 1 字节是算法类型）。指针头部的 extent 偏移 2 字节第一版恒 0（第一版指针指整个单元），读者遇到非 0 判该指针所在的结构损坏，与 MAC / nonce 同一处置、与加密开不开无关（D19（块指针的结构与宽度预算） 已定项 3；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 11 行）。根记录里的指针偏移非 0 按这条根自证不过处置：core 退回上一条根（靠 journal 重放可以回到同一版，挂载照样成功），checker 红这一条而 core 挂载成功，两边各判各的、不矛盾（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 6 行）。加密开启（系统配置级、全有全无，I-6.4（无未加密块））后覆盖范围 = I-6.2（明文头字段白名单） 该类白名单里的全部明文头字段，密文段由 MAC 罩。改动覆盖区内任一字节必须失配、改动净荷字节不许失配 | 已实现（`crates/singlefs-checker` 的 `check_unit`：三类的端点各自写死，里程碑步 2 / 步 3 验收里改坏头任一字节判红，2026-09-14；池级走读在 `walk::judge_unit_header`（`crates/singlefs-checker/src/walk.rs:674` 起）另判单元头偏移 4 的格式版本 = 1（walk.rs:722，不认得就判该结构损坏、不按今天的字段表往下解）与 29 字节预留位全 0（walk.rs:758）；走读跟随的每条指针在 `judge_pointer_mac_and_nonce_are_zero`（`crates/singlefs-checker/src/image.rs:699`）判头部 MAC 16 + nonce 12 全 0，四个入口分别是码 2 节点指针（walk.rs:815）、inode 内部条目子指针（walk.rs:1485）、实例表链指针（walk.rs:1811）、extent 数据指针（walk.rs:1961），只判不断、指针指向的单元照样走下去，2026-09-27 实审 A3-checker-2；坏镜像 `crates/singlefs-harness/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs`）。**extent 偏移那一半已实现**（checker 的指针判定 `judge_pointer_mac_and_nonce_are_zero` 加这 2 字节，走读跟随的每条指针都判；core 每个读盘上指针头部的读者都判 extent 偏移非 0、判该结构损坏：树表条目的树根指针、inode 内部条目的子指针、extent 叶记录的数据指针、extent 上段叶条目的下段根指针与内联数据指针、extent 树内部条目的子指针、分配记录树内部条目的子指针、实例表链指针各一处，加原有的 journal 新根段、码 2 内部条目、根记录三处（`crates/singlefs-core/src/pointer.rs` 的 `read_judging_the_extent_offset_from`；`grep -rn 'Pointer::read_from' crates/singlefs-core/src/` 只剩 `pointer.rs` 自己的单测一处）；影子账认单元的两个读者（`allocation_record_tree::node_pointers_as_far_as_readable`、`recovery::instance_table_page_pointers_as_far_as_readable`）今天也判，已定改回不判（影子账要多隔离、不漏隔离），改动在下一批；坏镜像遍历偏移 32、33，core 每类读者各一条用例在 `crates/singlefs-harness/tests/pointer_extent_offset_that_is_not_zero_is_refused_by_every_tree_reader.rs`；对应的账是 C581（extent 偏移非 0 没有判据）；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 11 行与 `research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 5 行） |
| I-2.5 | 位置条目按设备身份升序 | 任一块指针、中央映射条目的 value、journal 点名项里的位置条目数组按设备身份严格升序（D19（块指针的结构与宽度预算） 已定项 11、D23（journal 的角色与格式） 已定项 17，2026-09-13 用户定案）；两条位置条目设备身份相同或逆序，一律判损坏。⚠️ **射程（2026-09-14 补）：整条 86 字节全零的指针表示「注册了但还没有根」，整条豁免升序判定**——它内部那两条位置条目都是 (设备 0, 槽 0, 校验和 0)，按字面正是「设备身份相同」那一档。今天写出全零指针的有四处：树表里 livelist 树、稀疏旁表树与 deadlist 树那三条条目（D6（快照实现模型） 已定项 2、D5（快照 / 空间记账机制） 已定项 6 与 D5（快照 / 空间记账机制） 已定项 11 各自定案的「day-1 注册进树表，根指针为零」），以及 mkfs 与暖机的根记录里那个还没发布过的中央映射树根指针。非全零的指针不享这条豁免。 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |

```

**出处 `.claude/kb/invariants.md:136-170`（整段抄，未转述）**

```markdown
## I-3 空间记账 —— 类

**依赖 D5（快照 / 空间记账机制）。**
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-3.1 | 已分配统计对得上 | 已分配空间统计 == 实际遍历所有引用得到的和 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） ⚠️ **checker 读法（2026-09-14 用户收尾弹窗定甲）**：「实际遍历所有引用」按根环里全部有效根的引用取并集——新池新建文件之后第 0 版树表单元只被更早的根引用、仍占着空间（已释放、在 defer 队列里），只按最新根算会少 1 槽。**2026-09-17 起「有效根」= 回退候选集里的根**：按最新根指着的实例表有效（无那个实例的行，或有行 (i, Ti, Wi) 且 T ≤ Ti）且 txg ≥ F_生效（D16（发布语义） 已定项 1「生效」）：checker 自己算，取各块盘上最新的有效根所带 F 的最大值，再与池里自证过、fsid 与本池相同的系统配置槽带的 F 取大（`walk::check_pool_image` 里的 `effective_rollback_floor`，C556（checker 与层 0 不读系统配置里的 F） 已还清）——被抛弃时间线的根引用的单元由影子账隔离、F 之下的根引用的已释放单元已可再分配，都不在当前账里（里程碑「覆盖写、释放、回退与复用」步 4 / 步 5，`crates/singlefs-checker/src/walk.rs`）。⚠️ **最新根带的 F 低于 F_生效（两者不等）时这一格不判、报「不适用」（实二的读法，主 agent 2026-09-26 暂认，最终代码三方专攻这一格）**：先把新 F 写进系统配置之后、带新 F 的根落盘之前，或带新 F 的根全读不出，最新根的「已分配」是按它自己那个较低的 F 记的，还算着 F_生效 之下那些根引用的槽，与按 F_生效 取的候选集并集不可比，照判会在合法状态上红；另一种读法（并集按最新根自己的 F 取、别的几条按 F_生效 取）条款没选。坏镜像 `checker_known_bad_images.rs` 的 `every_root_carrying_the_raised_floor_unreadable_keeps_the_candidate_invariants_green_because_the_checker_reads_the_system_configuration_floor` 钉住这一格报不适用，`crates/mutations.tsv` 里把这道例外拿掉的那一行（实二）让它红。F_生效 之下仍在 defer 的槽：F_生效 之下、没被抛弃的根引用着，候选集那一遍没走到，而最新根的账里罩住它的记录是已释放、释放代高于 F_生效 的（还在 defer 里，抬 F 还不许回收），并进遍历得到的和（收口表第 43 行取丁-defer，用户 2026-09-28 定，被攻过一轮，判决 `research/prompts/abandoned-floor-r1-main-verification.md` M1；`walk::deferred_slots_referenced_only_below_the_floor_per_device`，用例 `raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs`）；⚠️ 它会不会盖住「该回收而没回收」那一类错，没拿变异试过。隔离的记录：已分配而没有任何根引用的记录，按单元豁免——它罩住的那个单元只要有任何一份副本 checker 自己读出来读不出或校验和对不上，这个单元在每块盘上的那几条记录都豁免；每一份都读得出且对得上，照旧判违例（用户 2026-09-25 定「核出坏的才豁免」，按单元的读法是主 agent 同日定的，D19（块指针的结构与宽度预算） 已定项 5 的隔离）。 中央映射条目指的物理范围算进遍历得到的引用，与树里指向同一单元的引用同 (设备, 起点槽, 跨度) 时算一个（2026-09-27，`research/prompts/m2-rev-b1-implementer-report.md` 第六节第 2 条）。由记录施加出来的那一版照 D23（journal 的角色与格式） 已定项 15「实例表单元指针照所选根」，走它施加在其上那一版（环里同实例、txg 比它小的根里最大的一条，不管在不在候选集里）的实例表链与根记录持有的分配记录树（`walk::walk_version_applied_only_by_records`，主 agent 2026-09-27 定，被攻过零轮）。⚠️ **已知缺口**：环里同实例、txg 更小的根一条都读不出时，那一版的实例表与根记录持有的分配记录树走不到，照旧漏数；「隔一条也是同一张实例表」是按「一个实例写行之后每一版照抄同一张实例表」推的，没量（`research/prompts/m2-rev-b2-implementer-report.md` 第七节第 2 条）。 |
| I-3.2 | 已停用·方向反了 | **此编号不再使用**——它写成「≤ 引用它的**最新**快照的代号」，方向反了：紧的约束是最**旧**的那个。旧写法被 I-3.5（引用区间的精确性） 的子判据 ② 严格蕴含，且检不出任何一类真实错误 | 不适用 |
| I-3.3 | 已停用·归属未定义 | **此编号不再使用**——「所属快照」未定义，而两种归属约定给出不同的比较符。由 I-3.6（deadlist 紧界） 取代 | 不适用 |
| I-3.5 | 引用区间的精确性 | **引用区间的精确性**。**内容**：对任一块 b 与任一根 R（快照根或活头根），**「b 从 R 可达」这个可达性谓词，必须等于记账所用的那个谓词**——两者是同一个定义，不是两份。**当前形态下该谓词的表达式**是 `birth(b) ≤ R.txg < death(b)`（[decisions.md](decisions.md) D5（快照 / 空间记账机制） 形态）；**换快照模型时表达式要重写，内容不变**。可判定形式（与 I-3.1（已分配统计对得上） 共用同一趟全盘遍历）：按 txg 升序枚举全部快照根与活头得 `Reach(b)`，则 ①② 合成一条**出现规则**：取回退候选集（I-7.4（近 K 代块未被复用） 用的那一份，加由记录施加出来、根槽从没落盘的那几版）按 txg 升序，同 txg 的几条并成一条（R⁻ 侧引用集合取并，R 侧逐条各判一次），相邻两条 (R⁻, R) 为一对，逐对判。R 引用而 R⁻ 不引用的每个落点 b（第一次出现与再次出现都判，`Reach(b)` 在 txg 序上是否连续不再单判）满足 (a) `birth(b) > R⁻.txg`；或 (b) b 落在 R 树表里某棵用户可见树（今天 extent 树、inode 树，`deleted_inodes` 树根非全零之后加它）的子树里，那棵树的根指针非全零、出生 txg 严格小于 R⁻.txg，且那棵子树里每个读得出的单元头的诞生代号都不大于那个出生 txg（读不出头的单元不计入）。`birth(b)` 取 b 的单元头里的诞生代号；b 自己的头读不出时这一个落点不判，读不出归 I-1.1（块头自述逻辑地址） 与 I-2.1（校验和与内容匹配）。**缺口**：R.txg ≠ R⁻.txg + 1 时这一对之间有洞，分两种处置——最新根的实例表里有一行 (R⁻ 的实例, T = R⁻.txg)、R 的实例比 R⁻ 的大、两者之间每个实例的行都是 (j, 0, 0)，即实例表行证明 R 所在的实例直接接在 R⁻ 那一版上的，这一对照常判（gap-row）；不是的，用户可见树子树里的落点不判，别的落点（树表、记账树、分配记录树、中央映射树的单元与实例表单元）仍照 (a) 判 `birth(b) > R⁻.txg`。R 或 R⁻ 的引用集合数不全（走读读不出，或走到没有条款的形状）时整对报不适用；没有一对判到任何落点时整条报不适用并带理由（同 I-4.9（非最新候选根引用的单元不晚于它） 候选集只剩一条根的写法）；③ 若 b 不被活头引用，则 b **恰好**出现在一个 deadlist 中，且该 deadlist 的归属区间 = `(Reach(b) 中最晚者.txg, 其下一个根.txg]`；④ 若 b 被活头引用，则 b 不出现在任何 deadlist 中。⚠️ **射程**：缺口的来源是新实例第一条根的 txg 取根环与 journal 里见过的最大 txg 加一（`crates/singlefs-core/src/mount.rs` 取号那一步），所以写行那次发布的记录已落而根槽没落、多记录发布只落一部分之后再可写挂载，必出现缺口；另有恢复丢掉尾巴记录、根槽写失败重发跳一格、根槽读不出、根槽被环盖掉而记录也被盖掉（短 journal 环）。**盲区**：缺口右边那一对里，落在用户可见树子树里的新写单元（新叶）的诞生代号被改小到 R⁻.txg 以下，判不出；整条用户可见链（树根指针出生 txg 与树里单元的诞生代号）一起改小到 R⁻.txg 以下的坏镜像同样判不出：盘上与一次合法回退逐项同形，判别子是回退那次发布写的谱系记录（D18（块里携带什么信息） 已定项 24），`crates/` 今天没有。**谱系记录落地之后**，(b) 收成「R 的谱系树里有一条记录 (T_rb, T_old)，R⁻.txg < T_rb ≤ R.txg，且那棵用户可见树的根指针出生 txg ≤ T_old；谱系树读不全时该对不适用」，这一形与缺口条款并存（`research/prompts/m3-birth-floor-r2-main-verification.md` 第二节第 9 行）。**判别力**：诞生代号改小到恰好等于 R⁻.txg 的新写单元必须红；最新根的 inode 树指回已丢的旧叶、分配记录与映射条目一起改平必须红；回退根槽被环盖掉、记录也盖掉之后的合法镜像不许红。**推翻条件**：任一合法镜像（池级 checker 与层 0 其余判定都不红）上红。 | 未实现（实现进下一批）：出现规则取按树继承下界加 β 出现规则（用户 2026-10-02 定 ① 取 β）加缺口处置；缺口处置用户 2026-10-03 定 gap 加 N3（判决里的局部标号，只放过用户可见树里的单元）加 gap-row 两样都加。**被攻过几轮**：按树继承下界加 β 加整对不适用的缺口，在合法镜像上两轮没红（各一次抽样）；gap 加 N3 加 gap-row 的合成形、同 txg 几条并成一条、`deleted_inodes` 非全零才算继承、读不出头的单元不计入这三处措辞、谱系记录落地之后的区间形，都是零轮，终轮不再开轮，实现时再攻。实现挂在池级 checker 的主走读上（每版落点集合快照、用户可见树标签、压得住的判、新判定函数、位置键折叠、`IMPLEMENTED_INVARIANTS` 加一条、用例与变异），I-3.1（已分配统计对得上） 在短环上洞里的误红归下一批调查，与这一条共用同一个「候选集有洞」判法（`research/prompts/m3-birth-floor-r3-main-verification.md` 第二节第 4、5 行与第四节）。② 的上界半（`birth(b) ≤ R_first.txg`）另立 I-4.9（非最新候选根引用的单元不晚于它）（`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 1 行）。⚠️ **射程注（2026-09-13）**：跨头共享的块（稀疏旁表计数 > 1，D5（快照 / 空间记账机制） 已定项 6）不由 deadlist 单独管，判据 ③ / ④ 对它们不适用；它们的生死由旁表计数与各头的可达性合起来判，那条不变量还没立。 |
| I-3.6 | deadlist 紧界 | 任一 deadlist(S) 中的块满足 `birth ≤ prev(S).txg`（约定 B 下的紧界）。判别力：抓「kill 时该走直接释放却进了 deadlist」这一类（泄漏方向） | 未实现 |
| I-3.4 | 可用空间扣待删占用 | 待删中 inode 没释放的单元计在记账「已分配」里（释放只进 defer），记账标签 4（待删除但意图未完成占用）恒 0，可用空间统计因此已扣除待删占用；准入式子另扣环窗口常驻项（D28（挂载期承诺量） 已定项 1）。来源 `research/prompts/m3-delete-r3-main-verification.md` 第三节判标签 4 那一格，被攻过零轮 | 未实现 |
| I-3.7 | 活快照可枚举 | 盘上存在一个可枚举的持久结构，其元素恰为当前全部活快照（既不多也不少） | 未实现 |
| I-3.8 | 实例表行唯一且低于挂载根 | 实例表 kind 0 行按实例代号唯一；行只在回收条件成立后删（整轮清扫对池中每一个落点都得出判定、其中没有该实例的未发布单元、那次清扫里没有读失败 ∧ 全部设备在线 ∧ 根环里没有该实例发布的根、且根环每个槽都读成功）；行的实例代号 < 挂载根实例（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2） | 已实现（2026-09-16，`crates/singlefs-checker/src/walk.rs` 的 `judge_instance_table_rows`：行按实例代号唯一、每行实例代号低于最新根的实例、链指针记录在末尾；回收那一半没有输入，第一版行只增不删，删行的条件没人判） |
| I-3.9 | 释放代落在停止引用它的那一格区间里 | 任一带已释放标志的分配记录，它的释放代必须落在区间 `(最后一条仍引用这个落点的有效根的 checkpoint_txg, 最早不再引用它的有效根的 checkpoint_txg]` 内；根环没有洞时这个区间只有一个值，等同于「等于最早不再引用它的那条根的 txg」。若每一条有效根都还引用它，它不许带已释放标志（D3（空间分配） 已定项 7（释放时条目不删、改写 value：跨度段最高位置 1、后 8 字节从分配代换成释放代）；C374（释放代与树表诞生 txg 只有验收断言盯着））。环上有洞时（被施加的 journal 记录做的释放写着一个根槽从没写过的 txg，实测 2026-09-18 残留记录那条层 0 流的 12 个状态）按区间判，按「等于」判会在合法镜像上判红；已经回收、还没被复用的已释放记录跳过不判（见证它释放的根已不在候选集里，定位不了那个区间），全是这一类时整条报不适用并带理由。判别力：把发布 B 那次的释放代从 4 写成 3 必须红（最早不再引用那十个落点的有效根是 B、txg 4）。 | 已实现（2026-09-18，`crates/singlefs-checker` 的 `walk::judge_release_generation_and_tree_table_birth`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `each_release_generation_and_tree_table_birth_bad_image_reddens_only_its_own_invariant_after_the_overwrite`；层 0 每个崩溃状态都判）。**判法收窄在实现途中**：今天任一候选根走不全就整条报不适用；收窄成只在比最后引用它的那条根晚的候选根都走得全时才判这一条已释放记录，攻方造出的对照格镜像（抬 F 的空发布之后再覆盖写，把最后一次空发布那条根的树表翻坏）钉进用例（`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 10 行） |
| I-3.10 | 已分配记录的分配代等于它罩住的单元的诞生代号 | 任一**未带已释放标志**的分配记录，它的分配代（D3（空间分配） 已定项 3：`value = 分配代`；盘上编码见 D3（空间分配） 已定项 7）等于它罩住的那个单元头里的**诞生代号**（D18（块里携带什么信息） 已定项 7 的类身份段）——记录与单元是同一次发布写出来的，两个数因此同源。⚠️ **射程**：① 只管未释放的记录，已释放的那一半归 I-3.9（释放代落在停止引用它的那一格区间里）；② 记录罩住多个槽时（数据单元跨两槽）取起点槽那个单元头；③ **那个槽上读不出可用的单元头时这一条没有对象**（报不适用，不报违例）——读不出本身由 I-1.1（块头自述逻辑地址） 与 I-2.1（校验和与内容匹配） 管；④ **只读回退候选集里那几版的分配记录**（候选根的分配记录树、树表 0 条那一版根记录直接持有的那一片、由记录施加出来的那几版），**再加下一次挂载会先施加的那一版**：与最新根同实例、checkpoint_txg = 最新根的 txg + 1、带提交标记、根环里没有它的根的那条记录，新根段里树表指着的分配记录树（主 agent 2026-09-24 按代码三方 `research/prompts/m2-wave3-code-r1-main-verification.md` 第四节第 2 条定，被攻过零轮；只接这一步——恢复连着施加几条记录时第二条及以后那几版读不到，欠在欠账表）；不读被实例表判抛弃的根与低于 F 的根：低于 F 的根那棵账里未释放的记录罩住的槽，在 F 抬过之后可以被合法回收、换成诞生代号不同的单元，照「任一」去判会在合法镜像上红（主 agent 2026-09-23 定，被攻过零轮）。⚠️ 「再加下一次挂载会先施加的那一版」那一段的同源前提，靠 D16（发布语义） 已定项 7 持久顺序里单元与记录之间那道屏障撑着：拿掉那道屏障、又复用回收槽时，记录已持久而复用槽上仍是旧单元，恢复按点名验证拒施加、结局正确，这一条却在那个槽上红（2026-09-28 主 agent 复跑 floorreuse4：删屏障 271 个状态红 48、带屏障 73 个状态 0，`research/prompts/k3-9-barrier-r1-main-verification.md` 第一节）；用户 2026-09-28 定屏障维持现状、checker 不改，欠在 C592（单独拿掉单元写与记录间那道屏障今天没判据会红）。**它拦的是什么**：复用一个已回收的落点时改写了那条分配记录、却把上一次的代留着（2026-09-22 增补 3 第 1 件的代码三方第一轮攻方腿造出这条变异，快档与大档都判不出，因为全仓没有一条不变量管已分配记录的分配代） | 已实现（2026-09-23，待代码三方与崩溃验证；池级 checker `walk::judge_allocation_generations_against_unit_births`，读法照射程 ④，同一条记录（盘、槽、代）只判一次，射程 ③ 按记录逐条跳过、一条都没判到才整条报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） 下一次挂载会先施加的那一版由 `walk::tree_table_pointer_of_the_version_the_next_mount_applies_first` 并进读集；它的坏镜像（发布 B 的记录已落、根槽没落，B 那棵分配记录树里一条未释放记录的分配代改成诞生代号 + 1）是 `checker_known_bad_images.rs` 的 `an_allocation_generation_past_its_unit_birth_in_the_version_only_its_journal_record_carries_reddens_the_allocation_generation_invariant_before_the_mount` |
| I-3.11 | 已分配减 defer 等于最新根走读 | 逐盘：被判镜像上记账树里「已分配」那一行（记账第 1 项）减「defer 待释放」那一行（第 5 项），等于从最新有效根走读到的、这块盘上被引用的槽数。统计量的标签与行形态是 D5（快照 / 空间记账机制） 已定项 4 与已定项 10 的那几行，读的是镜像里的记账行、不读内存里的分配器；走读与 I-3.1（已分配统计对得上） 同一个走法，只取最新有效根。⚠️ 它补的是 I-3.1（已分配统计对得上） 与 I-5.2（空闲统计对得上） 都够不着的那一种：写者把活单元记成已释放、放进 defer，已分配与空闲两个和照样对得上（alloc-basis 第三轮判决 `research/prompts/alloc-basis-r3-main-verification.md` 的 Z10）。⚠️ **判别力自证在造出来的基底上做**：E156（alloc-basis 四条岔路的代价数） 第 4 次重跑在造出来的基底 β_syn（第 5 项为 0）上再记 1 槽 defer，I-3.11（已分配减 defer 等于最新根走读） 判红、去掉「减 defer」判绿（`research/results/e156-alloc-basis-counts-2026-09-26-r2.out` 的 `E7RESULT name=q7c1_not_subtracting_defer basis=beta_syn k=1 real_check_is_red=true without_subtracting_defer_is_red=false flips_red_to_green=true`）。可达状态里第 5 项能不能为 0 没量到：挂载时回退（回退写行那次发布）今天没有了，同一份产物里挂着回退那一版第 5 项是 356 槽（`E7RESULT name=u13a_rollback_to_oldest_candidate overwrites=72 target_txg=52 txg_before=75 txg_after=76 instance_before=1 instance_after=1 instance_table_rows_before=0 instance_table_rows_after=0 item1_slots=374 item5_slots=356 i311=holds structure_holds=true`）。隔离的记录：已分配而没有任何根引用的记录，按单元豁免——它罩住的那个单元只要有任何一份副本 checker 自己读出来读不出或校验和对不上，这个单元在每块盘上的那几条记录都豁免；每一份都读得出且对得上，照旧判违例（用户 2026-09-25 定「核出坏的才豁免」，按单元的读法是主 agent 同日定的，D19（块指针的结构与宽度预算） 已定项 5 的隔离）。 | 已实现（池级 checker `walk::check_pool_image`：逐盘判「defer 待释放 + 最新根走读 == 已分配」，最新根走读取 `walk_root(最新根)` 刚走完那一刻、与 I-3.1（已分配统计对得上） 同一个走法；最新根下面没有记账树时报不适用。坏镜像 `live_unit_recorded_as_deferred_reddens_only_the_allocated_minus_deferred_invariant`（盘 0 defer 行 +1 / −1 槽，只红它，I-3.1（已分配统计对得上） 与 I-5.2（空闲统计对得上） 照旧真被评估过且成立）；判别力自证在造出来的基底上 `on_a_synthetic_base_with_an_empty_defer_queue_one_deferred_slot_reddens_the_allocated_minus_deferred_invariant`（两盘 defer 挪回 0 之后再记一槽）；`crates/mutations.tsv` 5 条变异（判定恒真、不减第 5 项、`==` 放宽成 `≥` / `≤`、拿全部候选版本那一份走读比）；用户 2026-09-23 定采纳：alloc-basis 岔路单第 12 行（岔路 7）） |
| I-3.12 | 实例表行 T 低于引用它的根的 txg | 根环里任一自证过的根 R，它指着的实例表的每一行 (i, T, W) 满足 T < R 的 `checkpoint_txg`；链上某一片读不出的根不判。依据 D18（块里携带什么信息） 已定项 11（恢复写的行，所选根那个实例的 T 是重放之后那个根的 txg，中间实例写 0）与 D23（journal 的角色与格式） 已定项 14（新实例第一次发布的 `checkpoint_txg` 大于环里全部根与记录的号）。射程是每条根各自指着的那张表，不是只有挂载根那一张（那一张归 I-3.8（实例表行唯一且低于挂载根））；同一实例的行在两张表里是否逐字节相同不判，条款没有这一句。 | 已实现（2026-10-02，池级 checker `walk::judge_instance_table_rows_against_the_roots_referencing_them`：每条自证过的根各沿自己的实例表链判，链上有一片读不出的根跳过，一行都没判到时报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 6 行） |
| I-3.13 | 记账行代段等于这一版的 txg | 走读到的任一版（候选根，以及由记录施加出来的那几版）的记账树里，每一行的代段（记账 key 偏移 14，8 字节）等于这一版的 `checkpoint_txg`，只判走全了的树。依据 D5（快照 / 空间记账机制） 与 D16（发布语义） 已定项 6 的「记账代 = checkpoint 号」（每条根的记账树只装这条根那一代的行）。 | 已实现（2026-10-02，池级 checker 走读里的 `walk_tree_table_entry` 对每一版的记账树判，版本含由记录施加出来的，只判走全了的树；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 5 行） |

> ⚠️ **I-3.5（引用区间的精确性）四条子判据双侧夹死 与 I-3.1（已分配统计对得上） 正交，两条都必须有。** I-3.1（已分配统计对得上） 是标量总账（已分配 == 遍历和），
> 而**归属错误对它不可见**——块被挂到错的 deadlist 上时，它仍然被恰好一处引用，总账依然平。
> 要检出归属错，必须有**按快照分箱的向量等式**。
> 这与 [decisions.md](decisions.md) D14（双轨（大小文件 / 持久临时）） 已定项 6 推导 D-3（多轨兼容可判据） 里「标量等式 → 按轨道分箱的向量等式，
> 判别力净增」是同一个形状。
>
> ⚠️ **数据丢失方向只在空间被重新分配之前可检出。** 该进 deadlist 却被释放的块，
> 一旦其空间被复用，症状就退化成 I-2.1（校验和与内容匹配） 校验和不匹配，即「损坏，原因不明」。
> **所以这类检查必须在崩溃点重放 harness 里紧跟着删除动作跑，不能只在全量里跑。**
>
> ⚠️ I-3.1（已分配统计对得上） 是**唯一允许全盘遍历的记账检查**——它是 checker，不是运行时路径。
> 运行时记账必须增量维护（D5（快照 / 空间记账机制）），checker 才用遍历来验证增量维护有没有出错。
> 这条对照关系是本工程验证记账正确性的核心手段。

```

**出处 `.claude/kb/invariants.md:171-189`（整段抄，未转述）**

```markdown
## I-4 事务与崩溃一致性 —— 类
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-4.1 | 重放后树可遍历 | 任一崩溃点重放后，所选根（重放之后那一版）指向的树全部可完整遍历。起点含介质损坏的镜像（前缀停在一条点名单元两份没都验过的记录上）报不适用。另三处报不适用：所选根那次发布读得出的记录里带末条标志的多于一条（锚点认哪一条条款没写，实现在任何写之前拒挂，重放不出一版）；所选根的 checkpoint_txg 是 u64 的最大值（接在它后面的发布取不到 txg）；所选根之上一次发布都不施加（重放之后那一版就是所选根，可不可遍历归 I-7.2（最新根可完整遍历））——没有记录压在最新根之上的镜像上这一条因此恒报不适用、不报成立。重放用的读环按 (实例代号, 计数器) 建键，两块盘同一计数器坐着不同实例的记录时两条都留，与实现恢复同键（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 2、8 行）。 | 已实现（2026-10-02，池级 checker `walk::judge_trees_of_the_replayed_version_are_traversable`：R-独，checker 自己照 D23（journal 的角色与格式） 已定项 14 的前缀规则复刻只读重放（`replay_the_next_mount_would_do`），施加出来的那一版（所选根换上末条记录的新根段四段，实例表指针与分配记录树根照所选根）另起一份走读，走读断了判红；在飞上限按环长 ÷ 4096 ÷ `JOURNAL_SAFETY_FACTOR` 现算。读环键 2026-10-03 改成 (实例代号, 计数器)（`records_the_replay_reads`），反向链那一关改按 (实例代号, 计数器 − 1) 找前一条（`record_back_chain_lets_it_into_the_replay_prefix`），消掉两块盘同一计数器坐着不同实例的记录时与实现分开的形态（推为第一版两盘池上不可达）；`records_as_the_recovery_reads_them` 与 `record_can_enter_a_replay_prefix` 两处同改按 (实例代号, 计数器) 建键，实现进下一批（主 agent 2026-10-03 定）。坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（重放六条规则各一份、两块盘同计数器不同实例两份）；层 0 里与 core 的 `effective_root` 逐状态差分今天没有能跑的入口，要先开口子，进下一批清单（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 8、9 行）；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 2 行与第四节用户定案） |
| I-4.2 | 无被引用未提交块 | 任一崩溃点重放后，不存在被引用但未提交的块 | 已实现（2026-09-22，池级 checker `walk::check_pool_image` 的 `judge_birth_identity_of_a_referenced_unit`：走读到的每个被引用单元，头里的诞生代号与写序（D18（块里携带什么信息） 已定项 7 单元头字段表里的字段）按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的已发布谓词判成已发布——**「未提交」取的就是「按那个谓词判不成已发布」**，谓词与 I-1.8（归并后版本全序） 扫描方向共用同一份 `PublishedPredicate`。打包记录类型 4 的实例表单元按 I-1.2（块头写序已发布） 那一行的例外不走谓词，不进这一条。坏镜像是把数据单元头里的诞生代号与引用它的 extent 记录里那条数据指针的诞生代号一起抬到最新根的 txg 之上——指针仍与头一致 ⇒ I-1.2（块头写序已发布） 不跟着红，分得开两条的就是这一份（`crates/singlefs-harness/tests/checker_known_bad_images.rs`）；层 0 每个崩溃状态都跑它。⚠️ **「任一崩溃点重放后」那半句没实现**：池级 checker 判的是手上这一份镜像、不跑恢复，判的是「被引用 ⇒ 已发布」这个态上的性质；重放之后再判一遍归层 0） |
| I-4.3 | 提交原子 | 提交是原子的：不存在"部分生效"的事务 | 未实现（它要判的是 D23（journal 的角色与格式） 已定项 7 那两个事务边界字段——事务号与提交标记——施加之后的效果。**对着一份镜像判不出来，没硬写**：「部分生效」要拿「这次事务该有的全部效果」当 oracle 才分得出来，而一份镜像上任何状态都自洽——同一份字节既可能是「事务整体没生效的旧态」，也可能是「生效了一半」，池级 checker 手上没有第二个输入。判它的是层 0 崩溃点重放加 oracle：`.claude/kb/milestone/02-second-txn.md` 第 486 / 487 行那两条（崩在第二条记录之前 ⇒ 整个事务不施加、文件是旧长度；把提交标记提前到第一条记录 ⇒ 恢复施加了半个事务、I-4.3（提交原子） 红）说的就是这条路。形态同 C116（叶容器的分裂 / 合并纪律没有会失败的检查）：操作断言，不进池级 checker，恒真的检查比没有检查更糟） |
| I-4.4 | 意图只描述目标态 | 任一未完成的意图 key，其字段只描述目标状态，不含任何增量字段（"再删 N 个"一类） | 未实现 |
| I-4.5 | 已停用·需两镜像 | **此编号不再使用**——幂等要比对两个镜像，单镜像判不了；判据落在 [experiments.md](experiments.md) E3（意图日志的断电语义） 判据③ | 不适用 |
| I-4.8 | 近 K 代根校验和自洽 | 任一崩溃点重放后，从回退候选集（D23（journal 的角色与格式） 已定项 14：按实例表判仍然有效 ∧ txg ≥ F_生效）里**任一**根出发遍历，所有块的校验和均与其父指针记录的一致（候选集的深度由 D16（发布语义） 已定项 1 的回退下界 F 定：平时整个根环，盘紧时缩到最近 4 个可退到的状态，每块盘上最新的持久有效根都在内）。**判别力**：抓「本事务内释放的块被重新分配并写入」这一类——那会让旧根指向的块被写入新内容，**使旧根成为假的回退候选**，而只验最新那一代根是发现不了的 | 已实现（2026-09-17，池级 checker `walk::check_pool_image`，最新根与候选集里每条更早的根各判一格：从它出发的遍历有单元对不上或读不出就红；坏镜像是 mkfs 树表单元抹成零；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-4.6 | 中间态其余不变量成立 | 存在未完成意图的镜像上，其余全部不变量仍然成立（分批之间的中间态是合法状态） | 未实现 |
| I-4.7 | 删除中 inode 有账 | 任一处于删除中的 inode，要么在删除中队列里有条目，要么其 extent 已全部释放——两者皆非的中间态不存在 | 未实现 |
| I-4.9 | 非最新候选根引用的单元不晚于它 | 回退候选集里不是最新的每一条根 R，它引用的单元（按落点认，读落点上单元头里的诞生代号，头校验和过）的诞生代号 ≤ R 的 `checkpoint_txg`。最新那条根归 I-4.2（无被引用未提交块），不在这一条里。依据 D16（发布语义） 已定项 7（单元先于根落盘）。 | 已实现（2026-10-02，池级 checker `walk::judge_unit_birth_against_an_older_candidate_root` 判走读里每个被引用的单元，共享子树里别的版本先走过的单元由 `merge_placements_below_a_unit_walked_by_an_earlier_version` 补判；候选集只剩一条根时报不适用，同 I-3.9（释放代落在停止引用它的那一格区间里）；坏镜像沿 I-4.2（无被引用未提交块） 那一份的造法，映射 key 也要跟着改，在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 1 行） |

> ⚠️ **编号只往上加，不回收、不重编。** 回收一个编号会让别处的旧引用悄悄指向另一条陈述，
> 而检索只端出其中一条，不会告诉你它换了意思（`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 4 条「矛盾比空白更糟」）。
> 退役的编号留一行说明去向，比留一个无解释的空档安全。

```

**出处 `.claude/kb/invariants.md:190-212`（整段抄，未转述）**

```markdown
## I-5 分配器 —— 类
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-5.1 | 物理范围不重叠 | 不存在被两个不同引用同时占用的物理范围（除非是显式的多副本） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） 引用按版判（2026-09-27，`research/prompts/m2-rev-b1-implementer-report.md` 第六节第 1 条、`research/prompts/m2-rev-b2-implementer-report.md` 第八节）：同一版（根环里的一条根，或由记录施加出来的一版）里同一个 (设备, 槽) 被两条指针引用判红，(设备, 槽) 取自位置条目的设备 ID 与物理偏移（D19（块指针的结构与宽度预算） 已定项 4），一条指针自己的两条位置条目同落点算一个引用（那一格归 I-2.5（位置条目按设备身份升序））（`walk::Walk::note_references_of_a_pointer_followed_by_the_walk`）；中央映射条目指的物理范围算进引用集合；别的版本走过的共享子树不重走，按缓存把它下面的落点并进这一版再比、不读盘（`walk::Walk::merge_placements_below_a_unit_walked_by_an_earlier_version`）。坏镜像在 `crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs`。 |
| I-5.2 | 空闲统计对得上 | 空闲空间统计 == 总空间 − 已分配空间 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-5.3 | 报出的空闲都兑现得了 | `df` 报给用户的空闲 ≤ 当下可分配 + 能在有界步数内放回的扣住量（defer 窗口里、回退下界以下的释放）。运行时的等价形态：`df` 报出 s 字节空闲时写 s 字节必须成功；删掉 s 字节之后同样大小的写在有界步数内成功（[decisions.md](decisions.md) D3（空间分配） 已定项 9，用户定案）。界随 D16（发布语义） 已定项 1 的形态定。判别力：把「放回扣住量」那一步摘掉，近满盘上删一个对象再写同样大小必须由绿转红 | 未实现 ⚠️ **口径注（2026-09-13，D28（挂载期承诺量） 已定项 4）**：`df` 要扣的 checkpoint 保留池是每次发布现算的量，发布即重算，近满盘上报出的空闲会随树高变小。 |
| I-5.4 | 分配记录罩住的槽互不相交 | 任何一条有效根的分配记录树里，同一块盘上任意两条分配记录（不论已分配、已释放、释放代是否已低于回退下界）罩住的槽区间 [槽号, 槽号 + 跨度) 互不相交。复用一个已回收的落点时，新记录罩住的槽上别的已回收记录要随这次分配一起删掉；留着它，下一次可写挂载重建分配器时两条记录罩住同一个槽 | 已实现（2026-09-18，`crates/singlefs-checker` 的 `walk::judge_allocation_records_disjoint`，按回退候选集里每条根判；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `an_allocation_record_whose_span_covers_the_next_record_reddens_only_the_disjointness_invariant`；层 0 每个崩溃状态都判，同日在新代码上全量：第二条流 2,104,413 个状态里评估 1,842,252 个、违例 0，其余 262,161 个报不适用，第一条流 262,165 个状态里评估 4 个、违例 0（`m2-wave2-crash-verifier-report.md`「主 agent 要的两个数」）。同日立，依据 `research/prompts/m2-wave2-code-r1-main-verification.md` 第二节第 1 行：复用时跨度变了留下重叠记录，下一次可写挂载在重建分配器时 panic，而 I-5.1（物理范围不重叠） 只管树里的引用、判不出记录之间的重叠） |
| I-5.5 | 从 mkfs 水位建树那一版的 t1 落点 | 走读到的任一版 V（候选集里每条根，含由记录施加出来的那几版）若是从 mkfs 水位第一次建树的那一版——① V 的树表非空，且种类 1（extent 树）条目的树 ID 等于 65536；② V 的树表里每条条目的诞生 txg（条目偏移 108）都等于 V 的 `checkpoint_txg`——则 V 引用的数据单元里，每块盘上落点槽号最小的那一个，槽号等于 U + 4，U 是所选系统配置偏移 417 的单元区起始槽号（默认 768 MiB 环 U = 50176，所以是 50180）。V 没有数据单元时不判；不满足 ①② 的版本不判。这一类含新池新建文件，也含 mkfs 之后重挂、实例 1、2、3 从 mkfs 水位写的第一个文件（用户 2026-10-04 定案，原话在决策变更史 D3（空间分配） 节）。只判 t1，t2–t12 不判（D3（空间分配） 已定项 10 射程没有「偏离即不合法」）。依据 D3（空间分配） 已定项 8 射程与已定项 10 ③ ④。 | 未实现（C673（三 oracle 合谋测试的补 oracle 改法没做） ⑥） |

> ⚠️ **I-3.1（已分配统计对得上） 与 I-5.2（空闲统计对得上） 的等式在 zoned 布局上有一个已知缺口（[decisions.md](decisions.md) D12（目标介质） 定了多布局之后才出现）。**
> zoned 设备上一个 zone 里只要还有一块活数据就不能 reset ⇒
> **一个活快照钉住的物理空间是 zone 粒度，而它引用的块数是块粒度**。
> 于是「已分配 == 遍历所有引用得到的和」在那套布局上不成立，
> 除非把「已分配」明确定义为「被引用」，**并另立一个「不可回收」量**。
>
> 连带一条对用户可见的要求：**报给用户的「这个快照占了多少」不许被定义成引用块数之和**——
> 那个数在 zoned 上会**系统性地低于**删掉该快照真正能拿回的空间，
> 而 [pitfalls.md](pitfalls.md) 第 12 条骂的正是这类「`df` 输出对用户没有意义」。
>
> **本缺口尚未有对应条目**，补它需要先定义「不可回收空间」这个量。

```

**出处 `.claude/kb/invariants.md:213-262`（整段抄，未转述）**

```markdown
## I-6 加密不变量 —— 类

**依赖 D9（加密），17 个分项全部已定。**
**这些检查在加密未开启时应当整类跳过并显式报告为「不适用」，不许静默通过。**

<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-6.1 | nonce 不重用 | 加密开启时，全盘任意两个**已写出**的块（不论当前是否仍被引用），其 (密钥代号, nonce) 二元组不相同 | 未实现 |
| I-6.2 | 明文头字段白名单 | 加密开启时，任一单元的明文头字段集合 ⊆ 它那一类的白名单，**两张，按 D18（块里携带什么信息）已定项 11 登记表的码分**（码 1 数据单元类；码 2 / 3 元数据类；D18（块里携带什么信息）已定项 7 收口）——共同前缀两类都许：{magic / 格式版本 / flags（含单元类标签位）/ 头校验和 / **nonce 代号** / MAC}；元数据类**仅此而已**（五元组 / key 区间等身份字段一律进密文）；数据单元类另许 {fsid / 载荷 CRC}。⚠️ **2026-09-07 用户定案（D18（块里携带什么信息） 已定项 13，臂庚）按两处收窄**：① 码 1 的 {五元组 / 诞生代号 / 写序} 从「另许」里去掉，就地变密文；② **`声明长度` 从共同前缀里去掉**，三类一起变密文——它给出一个装得下的小文件的精确字节数，是收窄前泄漏面里最重的一项。字段的**位置与宽度一个都没动**，变的只是「盘上这几个字节是明文还是密文」 | 未实现 |
| I-6.3 | MAC 与密文匹配 | 加密开启时，任一被引用的块，其 MAC 与密文及关联数据匹配；**关联数据的每个字段由 checker 从权威树的 key 现算，不许从指针或明文侧读回**（需要密钥在场） | 未实现 |
| I-6.4 | 无未加密块 | 系统配置声明加密时，不存在算法类型字段标为「未加密」的数据块或元数据块 | 未实现 |
| I-6.5 | nonce 水位单调 | 记录在案的 nonce 水位 > 全盘出现过的最大 nonce（崩溃后不重用的可判定形式） | 未实现 |
| I-6.6 | MAC 长度一致 | 任一指针记录的 MAC 长度与系统配置声明的一致 | 未实现 |
| I-6.7 | 副本共用 nonce | 同一份数据的多个副本，其指针记录的 nonce 相同（nonce 未绑物理位置的可判定形式） | 未实现 |
| I-6.8 | 密文校验和匹配 | 加密开启时，任一被引用的块，其指针记录的密文校验和与盘上密文匹配（**不需要密钥**） | 未实现 |
| I-6.9 | 密文过而 MAC 败判身份错配 | 加密开启且密钥在场时，任一被引用的块，若**密文校验和通过而 MAC 失配**，判红并归类为「身份错配」——**不许归类为介质损坏，不许触发副本修复或投毒** | 未实现 |
| I-6.10 | 已停用·明文映射层不存在了 | **此编号不再使用**——2026-09-07 D9（加密） 已定项 5 改取「只做 scrub」，不设明文映射层、不设明文反向索引 ⇒ 这一格**无对象**。曾要求：加密开启且密钥在场时，checker 从权威侧重建出的明文映射层与反向索引与盘上明文副本逐条相同、差异判红不许就地修复 | 不适用 |

> ⚠️ **I-6.1（nonce 不重用） 只在「不论是否被引用」这个范围下才有判别力。** nonce 重用最危险的一批数据
> 恰恰是崩溃遗留的孤儿块——已写盘、未进树、因而不被引用。
> 把检查范围写成「被引用的块」，等于在设计上排除掉最该查的那一批。
> 因此 checker 必须两阶段：① 遍历树可达块；② **额外全盘扫描已分配物理区间**，
> 靠块的明文头识别「这是一个已写出的 singlefs 块」并取出其 nonce 纳入去重集合。
>
> ⚠️ **阶段 ② 对 D9（加密） 已定项 4（nonce 载体）提出一条硬约束：nonce 的代号部分必须放在明文头，
> 无密钥可读，不能整体埋进密文。** 埋进密文的话 checker 没有密钥就连「这块用过哪个 nonce」
> 都判断不出来，I-6.1（nonce 不重用） 的阶段 ② 与 [experiments.md](experiments.md) E5（nonce 唯一性 × 崩溃点重放） 的判据③一起失效。
> I-6.2（明文头字段白名单） 白名单里那一项因此写作「nonce 代号」而不是含糊的「序号」。
>
> ⚠️ **I-6.1（nonce 不重用） 是抗 bug 检查，不是抗攻击检查。** 阶段 ② 靠明文头 magic 枚举已写出的块，
> 而一个能写裸设备的攻击者抹掉 magic 就能让块从扫描里消失。
> 对抗场景下它给不出任何保证——[decisions.md](decisions.md) D9（加密） 已定项 6 记了适用范围。
>
> I-6.2（明文头字段白名单） 与 D18（块里携带什么信息） 已定项 3 的打架已按
> 「两张白名单、按单元类分」收口（2026-09-02，三方论证判决）；扫描器解密前先读
> 共同前缀里的类标签位分出类，再套对应那张表。**头校验和罩仍在明文的部分，MAC 罩密文**（覆盖范围见 I-2.4（头校验和覆盖范围）；加密是系统配置级全有全无），
> 加密元数据的头完整性由两者分层。仍挂在
> [checks-owed.md](checks-owed.md) C68（不变量白名单与已定条款打架） 的那一半是
> nonce 12 字节的归属三处打架与门禁比对工装。
>
> ⚠️ **I-6.2（明文头字段白名单） 的白名单是「允许出现在明文头」的清单，不是「可以拿来判定」的清单。**
> 里面每一项都只是提示：**必须在密文侧有权威副本，读者取密文侧那一份。**
> 从明文侧取期望值等于让攻击者控制判定的输入。依据见 [decisions.md](decisions.md) D9（加密） 已定项 6 的「期望值来源」一节。
>
> ⚠️ I-6.1（nonce 不重用） 与 I-6.5（nonce 水位单调） 是**唯一能把「nonce 重用」这类灾难变成可检出事实的检查**。
> nonce 重用在 ChaCha20 这类流密码下直接泄漏明文，而它不会表现为任何读写错误——
> 没有这两条检查，它永远不会被发现。验证手段见 [experiments.md](experiments.md) E5（nonce 唯一性 × 崩溃点重放）。

```

**出处 `.claude/kb/invariants.md:263-277`（整段抄，未转述）**

```markdown
## 待补

以下类别在对应决策定案前无法写出：

- 反向索引一致性（D1（数据可移动性 / 反向索引） 定案后：正向引用与反向索引必须双向可达且一致）
- 条带 / 多副本布局（D2（RAID 条带策略） 细化后）
- 快照可见性（D6（快照实现模型） 定案后）
- 索引结构自身的结构不变量，如树的有序性、平衡性（D8（核心索引结构） 定案后）
- 密钥层级与换钥（主密钥槽 80 与 KDF 标识 4 的宽度与落点 2026-09-13 由 D22（单元原子性怎么合成） 已定项 9 定死；还剩主密钥槽的内部构成与换钥不重写数据）

---


---

```

**出处 `.claude/kb/invariants.md:278-318`（整段抄，未转述）**

```markdown
## I-9 inode 树结构 —— 类

**依赖 D8（核心索引结构） 已定项 6，形态已定（码 2 内部节点 + 码 3 叶容器、记录定长 140、inode 号单调不复用），checker 未实现。**
与 I-1（块自描述） 类的分工：I-1.6（类标签登记且三处相等） 管单元自己的类标签合不合法，I-1.7（打包容器合法与判定顺序） 管一个容器自己合不合法并且任一步失败即容器级中止；
I-9（inode 树结构） 管跨单元的结构约束（谁该指向什么、号与 key 的关系）与对象级（一条记录）的判定，对象级判红只让那条记录 EIO，不进 I-1.7（打包容器合法与判定顺序） 的中止链。
**共同前置**：checker 要知道一棵树是 inode 树才能对它判这一类，而树表条目里有没有「树的种类」一段无落点（D8（核心索引结构） 已定项 8）；第一版镜像上可由打包记录类型 2 容器头里的出生树反推哪棵树是 inode 树，所以这一类今天不是恒不评估。
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-9.1 | inode 树根恒码 2 | 树表指向的 inode 树根，共同前缀偏移 6 恒为 2（索引节点），哪怕树只有一片叶或是空的（零条目、key 区间 [0, 0]）——D19（块指针的结构与宽度预算） 的指针不带被指对象的类与大小，根的类若会变，读者按 16 KiB 读 32 KiB 是静默误读 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-9.2 | 条目身份与子头相符 | inode 树内部节点的每条条目：类型段 ∈ {0, 2}；类型段 0 ⇒ 子单元偏移 6 = 2 且身份引用的出生树、容器号、出生代三段为 0；类型段 2 ⇒ 子单元偏移 6 = 3，子头的 (出生树, 打包记录类型, 容器号, 容器出生代) 与条目的身份引用四段逐字相等，子指针头部的出生树 == 身份引用的出生树，且子容器的打包记录类型 == 2（出生树那两处 2026-09-12 随 C297（inode 内部条目里出生树两份没有定处理） 用户定案加）；其余判该条目损坏。跨头共享时出生树的读法随 C110（跨头共享与加密后元数据类的块头一致性读法） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-9.3 | 内部节点消息数恒零 | inode 树内部节点的消息计数恒 0（ε = 0，D11（索引节点要不要留消息缓冲区） 已定项 6 的树轴）；计数字段住节点内部布局（D15（格式冻结政策） 第 4 层，未冻结），那一层定了这条才可判 | 未实现，前置：树的种类 + 节点内部布局 |
| I-9.4 | 容器号不超最小 key | 打包记录类型 2 的容器内每条记录的 inode 号 ≥ 容器号；沿叶序容器号严格递增，且左容器的最大 key < 右容器号（这条逼出合并只许左吸收右 / 左从右借） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-9.5 | 删除队列里的号仍在树里 | `deleted_inodes` 里的每个 inode 号在同一棵树的 inode 树里存在且记录的 nlink = 0（记录在回收完成前留在 inode 树里，`deleted_inodes` 只存号） | 未实现（三·3 实现；`deleted_inodes` 的形态已定在 D8（核心索引结构） 已定项 16） |
| I-9.6 | 水位大于两处最大号 | 每个可写头的「inode 号水位」统计量 > 该树 inode 树内最大 key，> 该树全部已发布的（按实例表判）墓碑记录的对象 ID（墓碑记录住墓碑树，D18（块里携带什么信息） 已定项 24；被崩溃恢复抛弃的时间线上写的墓碑不在所选根的墓碑树里）——2026-09-16 随 D8（核心索引结构） 已定项 6 收窄：被抛弃时间线的墓碑不算——崩溃恢复落到较旧的根之后水位随所选根那棵账载入，被抛弃时间线里发过的号会被重发，算上那些墓碑 checker 按扫描读就判红（管理员回退是挂着时的一次向前发布，水位取 max、不重发，D23（journal 的角色与格式） 已定项 14）；checker 遍历侧算出的最大值与记账里的水位是两条独立路径 | 已实现（2026-09-22，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判）。**判的只有「水位 > 树内最大 key」这一半**：墓碑那一半（水位 > 墓碑树里全部记录的对象 ID 最大值）未实现，三·3 实现 |
| I-9.7 | 记录三段恒零 | 打包记录类型 2 的记录偏移 108 / 112 / 120 三段（填充 4 / flags 8 / 预留 20）恒 0；非 0 判该记录 EIO，容器其余记录照读（前向兼容政策，D15（格式冻结政策） 判定表；不是损坏检出，载荷校验和照样过） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-9.8 | 记录 inode 等于 key | 打包记录类型 2 的记录偏移 0 的 inode 号 == 它在 inode 树里的 key；不等判红、该记录 EIO，不许静默修复 | 未实现，前置：树的种类 |
| I-9.9 | locality 三值对照 | 记录的 `locality_id` == 该对象每条 extent key 的首段；extent 树未经重建时输出红 / 绿；系统配置 `map_provenance` 标 rebuilt 时两侧同源，输出「不可判定」，不许输出「通过」；按侧判（2026-09-16）：记录那一侧是从 key 编出来的、或 key 那一侧是从记录编出来的，同样输出「不可判定」；分批重新聚簇的中间态怎么判随 C346（后台重新聚簇没有定义） 定 | 未实现 |
| I-9.10 | 对象出生代三处一致 | 记录偏移 8 的对象出生代 == 该对象每个数据单元头五元组里的对象出生代 == 该对象的墓碑记录里的对象出生代（若有；墓碑记录住墓碑树，D18（块里携带什么信息） 已定项 24；只对 inode 记录还在的对象判，删完之后记录不在，这一个等号无对象）；重建后两侧同源，输出「不可判定」，与 I-9.9（locality 三值对照） 同形 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判；墓碑那一个等号未实现，三·3 实现） |
| I-9.11 | 可写克隆头有水位行 | 每个可写克隆头的树 ID 在记账里都有「inode 号水位」行（树表可枚举、记账可查）；「初值 ≥ origin 那一刻的水位」是建克隆那一刻的操作断言，转模型对拍——它的右操作数在 K 次发布后被 D5（快照 / 空间记账机制） 已定项 2 的点删丢掉，对着镜像读不到 | 未实现，前置：树的种类 |
| I-9.12 | 分隔 key 落在孩子区间之外 | inode 树内部节点的条目按分隔 key 严格递增；第 i 条的分隔 key ≤ 第 i 个孩子的 key 区间下界，且 > 第 i−1 个孩子的 key 区间上界。判的字段是内部节点条目头 8 字节那个分隔 key（D8（核心索引结构） 已定项 6：条目 120 字节 = 分隔 key 8 + 身份引用 26 + 子指针 86，「分隔 key = 该孩子创建时的最小 key，条目按分隔 key 单调递增」）；孩子的 key 区间对码 3 容器由它装的记录现算，容器头不带 key 区间 | 已实现（2026-09-22，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-9.13 | 类型 2 容器不空 | 任一打包记录类型 2 的容器记录数 ≥ 1（空叶在重建时没有 key 范围；唯一的叶被删空时不落盘、根变成零条目） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-9.14 | 树表条目的诞生 txg 跨根不变 | 同一棵树（按树 ID）的树表条目，在**同一条时间线上**的任意两条有效根各自的树表里，诞生 txg 必须相同；只有这棵树第一次出现在树表里时才设这个值（D8（核心索引结构） 已定项 8（树表条目的字段）；C374（释放代与树表诞生 txg 只有验收断言盯着））。「同一条时间线上的有效根」就是回退候选集（D23（journal 的角色与格式） 已定项 14 + D16（发布语义） 已定项 1）：最新那条根本身；以及根环里 (i, T) 满足两条的根——按**最新那条根指着的实例表**，没有实例 i 的行、或有行 (i, Ti, Wi) 且 T ≤ Ti；并且 T ≥ 最新根带的回退下界 F。有行 (i, Ti, Wi) 且 T > Ti 的根在被崩溃恢复切掉的旧线上，**不与现行线上的根比**（C511（回退到无文件那一版之后诞生代怎么接） 2026-09-23 用户定案收窄射程）。候选集不足两条根，或同一条时间线上没有一棵树的条目出现在两个不同的树表单元里时，报「不适用」，不报成立——几条根指着同一个树表单元、拿同一份字节比自己是恒真判定。判别力：① 把树表条目的诞生 txg 改成跟着这次发布的 txg 走，必须红；② 管理员回退前后同一条线上记得不同，必须红（回退是向前发布，前后是同一条线）；③ 只有被切掉的那条线记得不同，不许红。推翻条件：找到一条历史，最新根的实例表把某条根判成有效，而它不是最新根的祖先（不在同一条线上）。 | 已实现（2026-09-18，`crates/singlefs-checker` 的 `walk::judge_release_generation_and_tree_table_birth`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `each_release_generation_and_tree_table_birth_bad_image_reddens_only_its_own_invariant_after_the_overwrite`；层 0 每个崩溃状态都判。2026-09-23 收窄之后同文件两条：判别力 ② 的坏镜像 `birth_txg_that_the_line_after_a_rollback_records_differently_from_the_rollback_target_reddens_only_the_birth_invariant`（挂着时回退到 A 之后，回退那次发布 D 的树表把 extent 树（树 65536）的诞生 txg 记成 D 的 txg 9、A 那一版记 3：A 与 D 在同一条线上，只有 I-9.14（树表条目的诞生 txg 跨根不变） 红），判别力 ③ 的正向用例 `birth_txg_recorded_differently_only_on_the_timeline_cut_off_by_a_recovery_is_not_compared_and_every_invariant_holds`（被切掉的线由崩溃恢复造：C 被抛弃之后只改 C 那片树表里的诞生 txg，每一条都成立）；判定在 `walk::judge_tree_table_birth_txg`） |
| I-9.15 | inode 记录的 blocks 等于 ⌈size ÷ 512⌉ | 打包记录类型 2 的每条记录，偏移 48 的 blocks == ⌈偏移 40 的 size ÷ 512⌉（D8（核心索引结构） 已定项 6：blocks 是逻辑长度的 512 字节块数，不表示分到的空间；C480（inode 记录的 blocks 怎么算全仓没有条款） 用户 2026-09-23 定）；不等判红。读者要不要也判、判了报什么没有条款 | 已实现（2026-09-24，池级 checker `walk::check_pool_image` 走 inode 叶容器时逐条判，与 I-9.7（记录三段恒零） 同一处；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`：blocks 写成 64 / 5 / 7 各只红它；层 0 每个走得到 inode 叶的崩溃状态都判；变异：判定恒真、按向下取整比、写路径写回 64；里程碑「覆盖写、释放、回退与复用」收口表第 46 行） |
| I-9.16 | 树表条目按树 ID 严格升序且合发号次序 | 同一张树表（D8（核心索引结构） 已定项 8 的字段表）里，① 盘上相邻两条的树 ID 严格升序；② 树表里有的种类按发号次序（extent、inode、分配记录、记账、livelist、稀疏旁表、deadlist；中央映射树不进树表）排，相邻两棵的树 ID 严格升序。两道任一道不成立即违例。依据 D8（核心索引结构） 已定项 8「排序契约」与它的 ②「十二棵树的号从水位起连号发，次序照已定项 11 那一组 65536..65547」。判别力落在 ②：① 与 checker 在树表上判的 I-1.1（块头自述逻辑地址） 那组 key 严格递增是同一个谓词，只违反 ① 的镜像两条一起红 | 已实现（2026-09-27，core `crates/singlefs-core/src/recovery.rs` 常量 `TREE_TABLE_ENTRIES_ORDERING_CONTRACT`，两道合成一个入口，`rebuild_version` 与 `walk_to_file` 在读任何一棵树之前拒；池级 checker `crates/singlefs-checker/src/walk.rs` 另写一份、违例写明哪一版根与哪两条，列进 `image.rs` 的 `IMPLEMENTED_INVARIANTS`；坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs`「互换两条的种类」那份只红 I-9.16（树表条目按树 ID 严格升序且合发号次序）；只违反 ① 的钉在 `tests/tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs`；实现员报告 `research/prompts/m2-rev-tree-table-ordering-implementer-report.md`） |
| I-9.17 | 谱系树条目身份与子头相符 | 谱系树（D18（块里携带什么信息） 已定项 24）的每条条目：类型段 ∈ {0, 5}；类型段 0 ⇒ 子单元是码 2（谱系树根满长高之后，D8（核心索引结构） 已定项 6），身份引用的出生树、容器号、出生代三段为 0；类型段 5 ⇒ 子单元偏移 6 = 3，子头的 (出生树, 打包记录类型, 容器号, 容器出生代) 与条目的身份引用四段逐字相等，子指针头部的出生树 == 身份引用的出生树，且子容器的打包记录类型 == 5；其余判该条目损坏。与 I-9.2（条目身份与子头相符） 同形 | 未实现（三·3 实现） |
| I-9.18 | 删除队列里的号有谱系上的墓碑 | `deleted_inodes`（D8（核心索引结构） 已定项 16）里的每个号 N，墓碑树（D18（块里携带什么信息） 已定项 24）里都有一条在谱系上的墓碑记录 (N, N 的 inode 记录偏移 8 的对象出生代, [0, 2⁶⁴ − 1])。「在谱系上」：墓碑记录的死亡 txg 落在挂载根的父链上——从挂载根的 txg 起往回走，有谱系记录 (T_rb, T_old) 的那个 txg（T_rb）的父是 T_old，其余 txg 的父是 txg − 1（谱系记录住谱系树，D18（块里携带什么信息） 已定项 24；txg − 1 那一步核心层没有显式断言，C616（谱系父链靠 txg 连号没有断言））。不按谱系过滤的写法放过改坏（`research/prompts/m3-delete-r3-main-verification.md` 第三节判新不变量写法那一格） | 未实现（三·3 实现） |
| I-9.19 | 谱系上的墓碑不指活对象 | 墓碑树里每条在谱系上的墓碑记录（「在谱系上」照 I-9.18（删除队列里的号有谱系上的墓碑） 的定义）指认的 (号, 对象出生代)，inode 树里有号与对象出生代都相同的记录时，那条记录的 nlink = 0 | 未实现（三·3 实现） |
| I-9.20 | 谱系上的墓碑不重号且记录合法 | 墓碑树里在谱系上的墓碑记录（「在谱系上」照 I-9.18（删除队列里的号有谱系上的墓碑） 的定义）按 (号, 对象出生代) 不重复；每条记录区间起 < 区间止，第一版只许整对象 [0, 2⁶⁴ − 1]；每个打包记录类型 1 容器的记录数 ≤ 815（⌊(32768 − 136) ÷ 40⌋）。不按谱系过滤的写法在「删 → 回退 → 再删」的合法历史上判红（`research/prompts/m3-delete-r3-main-verification.md` 第三节判新不变量写法那一格） | 未实现（三·3 实现） |
| I-9.21 | 产品镜像里非最右墓碑容器装满 | 没有用只供测试的容量旋钮的镜像上，墓碑树里容器号沿叶序严格递增（根长高之后根的条目是类型段 0、容器号恒 0，见 I-9.23（墓碑树条目身份与子头相符），所以不沿根的条目读），除最右那一个之外的每个打包记录类型 1 容器记录数 == 815（开放容器满了才关，D18（块里携带什么信息） 已定项 24） | 未实现（三·3 实现） |
| I-9.22 | 测试镜像里墓碑容器不超旋钮值 | 只供测试的容量旋钮压小了一个墓碑容器装的记录数的镜像上，墓碑树里容器号沿叶序严格递增，每个打包记录类型 1 容器记录数 ≤ 旋钮值；旋钮不落盘，checker 从镜像外拿旋钮值。按产品容量判这类镜像会在合法状态上红（`research/prompts/m3-delete-r3-main-verification.md` 第三节判容量旋钮那一格） | 未实现（三·3 实现） |
| I-9.23 | 墓碑树条目身份与子头相符 | 墓碑树（D18（块里携带什么信息） 已定项 24）每个内部条目的类型段 ∈ {0, 1}：0 ⇒ 子单元是码 2（根长高之后，D8（核心索引结构） 已定项 6），身份引用的出生树、容器号、出生代三段为 0；1 ⇒ 子单元是码 3，子头 (出生树, 打包记录类型, 容器号, 容器出生代) 与身份引用四段逐字相等、子指针头部出生树 == 身份引用出生树、子容器打包记录类型 == 1。与 I-9.2（条目身份与子头相符） 同形（墓碑正推报告 `research/prompts/m3-tomb-r1-sonnet-output.md` 第 241 行 N1；根长高之后类型段 0 那一支是主 agent 按 K-R3-3 补的，被攻过零轮） | 未实现（三·3 实现） |
| I-9.24 | 删除队列至多一条 key | 任何一版里 `deleted_inodes`（D8（核心索引结构） 已定项 16）至多一条 key；每个被判的版本各判一次，不遍历版本之间的历史。依据：会话里每一次用户改动先把现行那一版的队列走空再做自己（D8（核心索引结构） 已定项 16 第 6 条的增补，用户定案），判决 `research/prompts/m3-blanks-r1-main-verification.md` 第二节「B4 + B8 队列上界与接口」一行 | 未实现（三·3 实现） |
| I-9.25 | 树表条目的 previous_snapshot_txg 取值 | 走读到的任一根（候选集里每条，含由记录施加出来的那几版）的树表里：树的种类不是 inode 树（种类码 ≠ 2）的条目，偏移 100 的 `previous_snapshot_txg` 恒 0；inode 树的条目为 0，或 1 ≤ p ≤ 这条根的 `checkpoint_txg`。依据 D5（快照 / 空间记账机制） 已定项 9（`previous_snapshot_txg` 只写在该头 inode 树那条树表条目里，其余条目写 0 且 0 = 不适用）；inode 树条目「≤ 根 txg」那一半的条款是用户补定的（`research/prompts/m3-inv-b-r1-main-verification.md` 第四节用户定案第 2 条）。 | 已实现（2026-10-02，池级 checker `walk_tree_table_entry` 每走到一条树表条目判一次，每个走到的版本（含记录施加出来的）都走它；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，inode 树条目的 previous_snapshot_txg 压过根 txg 那一份由 2026-10-03 补；⚠️ 树表单元被几版共享时只在第一个走到它的那一版判（`visited_units`），共享它的别的候选根漏判，与条款字面「候选集里每条」不符，代码注释自认判得更松，代码改回 kb 字面，实现进下一批（主 agent 2026-10-03 定；`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 2 行，推的，没造镜像）；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 5 行） |
| I-9.26 | inode 改动计数落在出生代与容器诞生代号之间 | 走读到的任一 inode 叶容器（码 3，打包记录类型 2）里每条记录：对象出生代（记录偏移 8）≤ 改动计数（记录偏移 88）≤ 这片容器头的诞生代号（偏移 73）。上界不是等于：容器因别的记录改动而重写时，没改的那条记录的改动计数小于容器诞生代号是合法的；下界要「创建算一次改动」，这一句条款是用户补定的。依据 D8（核心索引结构） 已定项 6 偏移 88 那一行（改动计数 = 最后一次改动所在发布的 `checkpoint_txg`）与 D18（块里携带什么信息） 已定项 9 之后的容器重写规则。 | 已实现（2026-10-02，池级 checker `walk::walk_inode_leaf_container`：每条记录判偏移 8、偏移 88 与容器头里偏移 73 的诞生代号（不是偏移 61 的容器出生代）；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，下界一份（改动计数 = 对象出生代 − 1）由 2026-10-03 补；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 5 行与第四节用户定案第 2 条） |
| I-9.27 | 树表条目的头 ID 取值 | 走读到的任一版（候选集里每条根，含由记录施加出来的那几版）的树表有条目时，逐条判条目偏移 116 的头 ID：种类码 2（inode 树）的条目等于它自己的树 ID（条目偏移 0）；种类 1（extent 树）、9（`deleted_inodes`）、10（墓碑树）的条目等于这张树表里种类 2 条目的树 ID，表里有多于一条种类 2 条目时放宽成「等于其中某一条的树 ID」（收严留给多头那一批），表里没有种类 2 条目时这三种判红；种类 3、4、6、7、8、11、12 的条目等于 0；别的种类码不判。树表 0 条的版本一条也不判。不拿字面 65537 比：它只是树 ID 从 mkfs 水位 65536 起发的那一版上的取值。依据 D8（核心索引结构） 已定项 8「头 ID 8 字节的取值」与 D5（快照 / 空间记账机制） 已定项 9。 | 未实现（C673（三 oracle 合谋测试的补 oracle 改法没做） ⑤） |

**不进 I-9（inode 树结构） 的**：分裂 / 合并的身份传递（左半身份不变、右半号 = 触发分裂的新记录号、出生树 = 执行分裂的树）与合并时机是操作断言，对着一个镜像判不了，走单测 / 模型对拍与崩溃点重放（C116（叶容器的分裂 / 合并纪律没有会失败的检查））；
nlink == dirent 条数的门（dirent 树存在）今天没有输入，写不成可判定形式，随 dirent 树上线时一起立。

```

**出处 `.claude/kb/decisions/16-发布语义.md:1-4`（整段抄，未转述）**

```markdown
## D16 发布语义 —— 已定

D16（发布语义） 管发布这一格：一个新根什么时候发、一次发布按什么顺序落盘、发布计数器是哪一个数、这一批释放什么时候回可分配集合、新实例的第一个根怎么先覆盖两块盘、一次空发布写不写单元、journal 记录格式在 D15（格式冻结政策） 的冻结分层里算什么。不管的：journal 的记录字段、环几何与重放前缀在 D23（journal 的角色与格式）；根环参数、根记录与系统配置的字段表在 D22（单元原子性怎么合成）；快照的记账模型与生死边界表在 D5（快照 / 空间记账机制） 与 D6（快照实现模型）；准入不等式与挂载期承诺量在 D3（空间分配） 与 D28（挂载期承诺量）；一次发布事件内部的内存序在 `litmus/commit-publish.litmus`，那份钉的是写块内容 → `smp_wmb` → 写 generation，对频率一字未提。

```

**出处 `.claude/kb/decisions/16-发布语义.md:27-77`（整段抄，未转述）**

```markdown
#### 已定项 1：defer 窗口取按盘回退下界的收严形态，按状态数读

**定案**：回退候选集保留最近 4 个可退到的不同状态——只数改过用户可见状态的根，推空与抬 F 产生的空发布根不算；同一个状态只数一次（去重的判法在「「非空」从盘上怎么认」那一段）。**根环容量是这句的边界**：两个状态来回交替超过根环一圈时，环里的根全是这两个状态，保留的就只有环里有的那几个，任何机制都给不出环里没有的状态。**窗口的单位是发布，不是秒**：defer 窗口保护的是「环里的根还能不能用」，普查过的消费者没有一个按秒计价。形态取 E139（按盘回退下界的收严形态） 的臂 G2S；回收门槛取一圈窗口之后，装置里没有窗口这一项，这一句不再与装置里跑的一字不差，欠里记着 E139（按盘回退下界的收严形态） 要重跑：

| 项 | 定案 |
|---|---|
| 可再分配 | 已释放 ∧ 释放代 ≤ max(F_生效, 一圈窗口里最旧有效根) |
| 环里最旧有效根 | 盘上全部根槽里自证合法、按实例表判仍然有效的根的最小 checkpoint_txg；写失败的槽按旧内容算，在飞、没持久的发布不算 |
| 一圈窗口里最旧有效根 | N = 环里读得出的最新有效根的 txg（挂载重建分配器时取它、不取 first_txg − 1；进程内每记一条根时取这条根）；窗口 = txg > N − 3S（一圈，3S 是环长）；窗口里自证合法、按实例表判仍然有效的根的最小 checkpoint_txg。挂载时读不出的槽在窗口里只对应一两个 txg：门槛按最小的算，根环表记最大的（零轮，只在攻方模型上量过）；进程内记根时，根环表里这一项成了窗口里最小的那个，先重读那一槽，读得出且自证过就照读出来的根算，仍读不出才按记的最大 txg 算（用户 2026-10-03 定，零轮）。同取这个窗口的六处：分配器的回收门槛（含抬 F 那一遍，根环表里挂载时读不出的那几项同样按上一句并进最旧有效根）、checker 的候选过滤、checker 的 I-3.1（已分配统计对得上） 丁-defer 门槛（max(F_生效, 窗口里最旧有效根)）、影子账的候选过滤、回退候选集、抬 F 的上限（含判「非空」那一遍） |
| 回退下界 F | 根记录带 8 字节（D22（单元原子性怎么合成） 已定项 7，242 → 250），系统配置另记一份 8 字节（D22（单元原子性怎么合成） 已定项 9）。平时不动，窗口就是整个根环；准入不够时抬，正常卸载时抬到现行那一版的 txg |
| 抬 F 的上限 | **准入抬 F**：min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)；非空有效根不足 4 个时取最旧的有效根；「有效根」「非空」都只在一圈窗口里的根上取（用户 2026-10-03 定，零轮：窗口外的旧根，窗口已许可它的树表被复用，读不出不再拒抬 F）。一次处置的目标 = min(这次释放的释放代, 第 4 新的非空根)。**卸载抬 F**：抬到现行那一版的 txg，不另判上限（等价于上限取最新的持久有效根，不按盘取小） |
| 根槽这一次读坏 | 算准入抬 F 的上限、挂着时管理员回退判候选（根环里有没有目标）与算 F_生效 时读根环：挂载那一刻就读不出或自证不过、这个进程之后也没写过的槽，当没有根；挂载那一刻读得出、或这个进程写过且 FUA 返回过的槽，这一次读不出或自证不过就重读一次，仍坏就拒这次抬 F 或这次回退、报它自己的错误成员，不按有根或没根猜（抬 F 那一半用户 2026-09-26 定，推的，被攻过零轮；回退那一半主 agent 2026-09-28 定，被攻过一轮：代码三方第三轮判决 `research/prompts/m2-closeout-code-r3-main-verification.md` 第一节判替没写的条款做了选择、回退扩写拒错或放错那一格没打中（一次抽样））。可写挂载的择根按 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续处置（D23（journal 的角色与格式） 已定项 14：系统配置见证过比所选那一版新的发布就把读阶段重读一次，仍判真拒可写）；只读挂载的择根、「环里最旧有效根」怎么处置这一格没判，见 C563（读根环一槽读坏就当没有根，别的调用方没判） |
| 生效 | F 另记进每块盘的系统配置（「抬 F 那一串」那一行写明先写后抬）；恢复后生效值 F_生效 = max(各幸存盘最新持久有效根所带 F 的最大值, 被抛弃时间线上的根所带 F 的最大值, 系统配置里读得出的 F 的最大值)；系统配置里读得出的取每块幸存盘两槽里全部自证过（校验和过且 fsid 与本池相同）的槽，与取号式子同一取法（D18（块里携带什么信息） 已定项 11） |
| 回退候选集 | 按现行那一版的实例表判仍然有效 ∧ txg ≥ F_生效 ∧ txg 落在一圈窗口里（D23（journal 的角色与格式） 已定项 14 的挂着时回退取同一个集合） |
| 抬 F 那一串 | 准入与卸载共用。动盘之前在分配器的拷贝上把这一串空发布整串预演：任一次取不到落点，就一次都不发、分配器原样换回抬 F 之前那一份（`MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite`）；**预演过了、第一次发布之前，先把新 F 写进每块盘的系统配置、过一道屏障**，之后才发第一条带新 F 的根；**先写那一步任何一块盘失败，这一串的根一条都不发、返回错误；已写进别的盘系统配置的新 F 留着、不回卷，F 的生效值只增不减**（主 agent 2026-09-26 定，推的，被攻过零轮）；空发布推到每块盘上都有一条带新 F 的根为止；真发时落盘途中写错，交回已落盘几次（`MountError::RaiseFloorSequencePublishFailed`），那一格里扣住的槽怎么放没有条款，见 C546（抬 F 被拒时扣住的槽不退回） |
| 正常卸载 | 推一串空发布把 F 抬到现行那一版的 txg（「抬 F 那一串」照准入的走法，上限见「抬 F 的上限」那一行）；这一串写的根在根记录 flags 里带卸载记号（D22（单元原子性怎么合成） 已定项 7）；第一版几何（R = 3、区域归属 0 / 1 / 0）下空发布 2 或 3 次（现行 txg 除以 3 余 1 时 3 次，余 0 或余 2 时 2 次）；不加干净关闭标记，下次挂载照旧走恢复（D23（journal 的角色与格式） 已定项 14） |
| 准入 | 可分配 = min(可再分配 + 活元数据 − 保留池, `df`)；准入不够、或准入放行而落点取不到时，先推空发布抬 F 再判（写行那次发布之前不推：它是新实例的第一次发布，元数据走切换预留，同一次发布里的用户数据重做照走准入、不够返回 ENOSPC）；**可写挂载的准入不够时，写行那次发布照走切换预留，写完行之后推抬 F 的空发布、再判**；一次准入最多 B = 4 + 2 k_tol 次发布（k_tol = 2 ⇒ 8），做满仍不够才报 ENOSPC；挂载处推满仍不够时挂载照样做成可写，之后的发布照发布准入判、不够报 ENOSPC，正常卸载照常（D2（RAID 条带策略） 已定项 13 那一格的例外）（用户 2026-09-26 定，被攻过零轮）；挂载续做的删除类发布被落点那一判以空间拒时同样推抬 F 再试，推满仍放不下挂载照样做成可写（D8（核心索引结构） 已定项 16 第 6 条，用户 2026-10-03 定） |
| `df` | 没用过的 + 已释放的 + 活元数据 − 保留池 − 推空最坏残留 − 滞后量；保留池 = 2 × 5 + (B − 1) × c_max = 10 + 7 c_max，残留 = 5 + (B − 2) × c_max = 5 + 6 c_max；滞后量 = 释放代 > max(第 4 新的非空有效根, 环里最旧有效根) 且不是空发布放掉的已释放块 |
| 环深下限 | 每区槽数 S ≥ k_tol + 2 = 4（D22（单元原子性怎么合成） 已定项 2 的 S 下界随之抬） |

**「非空」从盘上怎么认**：一条有效根算非空 ⟺ 它的树表里用户可见的树（inode 树、extent 树）的根指针，与它前一条有效根（按 txg 排；有效 = 按实例表判仍然有效 ∧ txg ≥ 当前的 F）树表里的不同。比的是树表里这两棵树的条目、不是树表单元自己的落点——每一次发布（连空发布）都重写树表单元（`crates/singlefs-core/src/transaction.rs`），按落点比会把空发布全算成非空。**数「第 N 新的非空有效根」与「4 个不同状态」时去重**：从新到旧数，一条非空根只有在它的 (inode 树根指针, extent 树根指针) 与每一条比它新、已计入的根都不同时才计入——管理员回退发出的新根与它前一条有效根不同、按「「非空」从盘上怎么认」那一判法算非空，而它的状态与更早的某一条相同，不去重就会把同一个状态数两次（第二轮 X7：来回回退之后按内容只剩 2 个不同状态）。反方向只让 F 停住，直到第 4 个状态的根离开根环。

**改新池新建文件的字节：是**：根记录加 F 8 字节；新池新建文件里 F 恒 0（mkfs 与第一次发布时非空有效根不足 4 个，上限取最旧有效根 txg 0）。系统配置加 F 8 字节，新池新建文件里写 0（D22（单元原子性怎么合成） 已定项 9）。

**连带改的**：D22（单元原子性怎么合成） 已定项 7 的根记录加 F（242 → 250）、flags 占一位卸载记号，已定项 9 的系统配置加 F 8 字节，已定项 2 的 S 下界；D23（journal 的角色与格式） 已定项 14 的回退候选集加「txg ≥ F_生效」；D3（空间分配） 已定项 9 第 2 条写明界；I-7.4（近 K 代块未被复用） 的 K 改由 F 表达；I-7.9（回退下界 F 不高于抬 F 的上限） 按卸载记号分两支。

**射程**：定的是 defer 窗口与回退下界怎么算，不定环几何、不定 c_max 的口径（c_max 是声明的最坏空发布开销，按 D28（挂载期承诺量） 已定项 4 按当时结构现算）。已知边角七样：

- **「生效」取 SysPre**：F 只住根记录、根槽不镜像（已定项 7）时，带新 F 的根坏掉 F_生效 就回落，更早的根重新进回退候选集、而它们引用的块已被合法复用——I-7.4（近 K 代块未被复用） 在这段历史上不成立（`research/prompts/m2-step45-code-r2-main-verification.md` 第四节、`research/prompts/m2-step6-checker-r1-main-verification.md` 第四节）；病根是 F 只住根记录，与生效取哪条规则无关（第三轮 K4：带 F 的根全坏那一格，今天的规则、MAX、MAX+HOLD 候选坏 12 / 140 / 12，SysPre 0）。代价：系统配置多 8 字节（动格式）；每次抬 F（准入与卸载）每块盘多一次系统配置写、加一道屏障；checker 与层 0 要跟着读系统配置里的 F——今天的 checker 按根上带的 F 判，带 F 的根全坏时会误红 I-7.4（近 K 代块未被复用） 等五条（第三轮 Y7，推的，没实现），欠 C556（checker 与层 0 不读系统配置里的 F）。「先写后抬」是第三轮攻方提的，被攻过零轮，用户 2026-09-26 知情定。被抛弃时间线上的根带的 F 算进 F_生效（用户 2026-09-28 定「算进」，被攻过一轮，判决 `research/prompts/abandoned-floor-r1-main-verification.md` M2：被抛弃根带着比别处高的 F 时，不算的话 I-7.12（系统配置 F 不低于同盘根上的 F） 红、回退退得到那个 F 之下，而被抛弃时间线抬 F 时已按它回收过槽；实现 `crates/singlefs-core/src/recovery.rs` 的 `effective_rollback_floor_of_the_roots_read`，checker 另算一份 `crates/singlefs-checker/src/walk.rs`）；C331（择根倒挂压过已确认的写） 取甲之后，被抛弃根带着更高 F 的盘面只在构造的镜像上有（那一形要可写挂载时一槽系统配置一时读不出，甲 拒这样的可写挂载）。与 `research/prompts/alloc-basis-r3-main-verification.md` 的 F 口径岔路一起判的那一件仍开着。
- F 回落之后第 0 代根重新进有效根、它的树表单元已被合法复用、此后每一次抬 F 都被拒到根环把它盖掉为止（`research/prompts/m2-emptypool-nonempty-r1-main-verification.md` 第一节 Z1）：前提是 F 回落，取 SysPre 之后在带 F 的根全坏那一格上不再出现（推的，第三轮没造那一形）。一次挂载里连续发布超过根环容量时挂载内不回收，checker 在合法状态上判 I-3.1（已分配统计对得上） 红（E153（账本形态与环上有洞的代价） 在 `crates/` 副本上量到），仍开着，归里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 ② 行。
- **正常卸载的代价**：每次卸载多 2 或 3 次空发布，第一版几何上每次 8 个单元、两盘合计 21 次写调用、279040 字节（第一轮攻方 t10、t5a，副本上量的）；卸载之后 F = 现行那一版的 txg，回退候选集只剩现行那一版。用户选时知情的代价是「停机时崩溃兜底只剩最新一条根」（按条款推的）；第二轮在卸载之后再挂、最新 j 条根坏的 75 行上恢复全读对、可写挂载全挂上，这一条没出现：F 之下的根还在环里，它们的记录能把状态重放回这一串的末尾（第二轮判决 G4）。
- **用户知情接受的代价三条**：删掉的空间要再发生 3 次改变用户可见状态的发布才回可分配集合，在那之前 `df` 不把它算空闲、按 `df` 写它是真 ENOSPC；`df` 少报 = 保留池 + 残留 = 15 + 13 c_max（c_max = 5 / 10 时 80 / 145 块）再加滞后量，而滞后量 = 4 × 最近四个窗口的释放量、**没有上界**（一窗删一个 8 块对象时平均约 34 块、最大 44 / 52，一窗删 32 个时 1044 块），删掉 s 字节 `df` 一个字节都不涨；c_max 按现算取，第一版池规模 9 块 ⇒ 保留池 73 块。
- **最少保留 4 个状态只在一次处置里根槽写失败 ≤ k_tol 次时兑现**：S = 4、c₀ = 5、3 次失败那一格违反 24 次，按字面记一次输、按 k_tol 的射程在外；全枚举下 S = 4、c₀ = 1 也违反 160 次（同为超过容忍的格）。
- **代价是选项本身的，不是这一族的缺陷**：一个被保留的状态钉住的是在它那一刻活着、后来才死的全部块，选最近 4 个就是每次删除都撞上——滞后量与「删了要等 3 次状态」都从这里来。
- **E139（按盘回退下界的收严形态） 答不了的**：计数模型，按发布计不按墙钟；只建根一级的引用、没建 journal 记录重放；实例切换的挂载时预留由 D28（挂载期承诺量） 已定项 3 扣；容量 4000 块、两块盘；k_tol 固定 2；滞后量口径是写登记的人替状态数读法构造的；保留池扫描的最小值落在扫描下沿、真正要多少没量到；环深下限是探针的数。两轮前身 E135（动态回退下界） 与 E138（按盘回退下界与推空的空间要求） 的结论已被 E139（按盘回退下界的收严形态） 取代，不再当依据用。

**依据**：

- E139（按盘回退下界的收严形态）：臂 G2S 在全部故障世界（活化区间内崩溃不丢盘、先跳号再抬 F、推空期间 1..3 次根槽写失败、坏槽、开销升到 c_max）零假候选、零不可恢复，各格按自己的 `df` 假性 ENOSPC 为 0，一次准入的发布数实测最大 7 ≤ 8，跑前保留池下不卡死——定案取的就是这条臂（跑前登记 `e139-preregistration.md`，产物 `e139-tightened-floor-2026-09-12.out`）。
- 四轮三方论证，第四轮判决 `research/prompts/d16-item1-r4-main-verification.md`；第三轮 `research/prompts/d16-item1-r3-main-verification.md`。
- 用户三次定案（候选集可以一直缩、最少保留 4 个、「4」数的是可退到的不同状态、三种「哪 4 个」里选最近 4 个），原话在变更史；「非空」怎么认与「写行那次发布之前不推」同为用户定案，原话在变更史。
- 用户定案 2026-09-25：抬 F 那一串改成整串预演，推翻 2026-09-24「维持逐次发布」（原话在变更史）；实现是实二六（`research/prompts/m2-e26-implementer-report.md`）。
- 生效取 SysPre、卸载抬 F 到现行那一版的 txg（B1）、「4 个不同状态」去重与根环容量边界、候选按现行那一版的实例表判：无实验，三方原型（`research/prompts/m2-rollback-forward-r1-opus-model/`、`m2-rollback-forward-r2-opus-model/`、`m2-rollback-forward-r3-opus-model/`）在冻结副本上量过，没立实验号，原型上的数不是入库装置上的数；三轮判决 `research/prompts/m2-rollback-forward-r1-main-verification.md`、`m2-rollback-forward-r2-main-verification.md`、`m2-rollback-forward-r3-main-verification.md`（第三轮第三节站住的、第四节交用户的）。SysPre 的「先写后抬」被攻过零轮。系统配置里读得出的 F 取每块盘两槽里全部自证过的槽、先写那一步失败时一条根都不发且已写进的 F 不回卷，这两处是主 agent 2026-09-26 定的，推的，被攻过零轮。
- 用户定案 2026-09-25（正常卸载时抬 F、取 B1）与 2026-09-26（C419（F 生效之后靠什么保持不回落） 取 SysPre，推翻 2026-09-25 定的「生效值取各盘 F 的最大值」；「4 个不同状态」加根环容量边界），原话在变更史。
- 「根槽这一次读坏」那一行：抬 F 那一半是用户定案，原话在变更史，构造的反例在 `records/2026-09-24-里程碑二收尾调度.md` 第三节「实四丙交回」那一行；回退那一半是主 agent 2026-09-28 定（同一份调度记录第 309 行），经过在变更史。实现已合入：`crates/singlefs-core/src/recovery.rs` 的 `readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once` 对这个进程知道住着根的槽「读不出或自证不过」都重读一次，仍坏报 `RootRingSlotKnownToHoldARootStillBadAfterOneReread`（带 `first_reading` / `reread`）；算上限那一遍是 `crates/singlefs-core/src/mount.rs` 的 `roots_of_the_ring_for_the_ceiling`（仍坏报 `MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`），抬 F 与卸载重算影子账与回收门槛那一遍仍坏报 `MountError::NewerStateStillUnreadableAfterOneReread`（`StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot`），管理员回退的 `rollback_candidate` 判候选与算 F_生效（`recovery.rs` 的 `effective_rollback_floor_rereading_known_ring_slots_and_the_newest_instance_table_once`）仍坏报 `RollbackError::CandidateJudgementStillUnreadableAfterOneReread`、在任何写之前。用例：`crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs` 的 `raising_the_floor_while_a_root_ring_slot_known_to_hold_a_root_stays_not_self_verified_is_refused_before_any_write`、`a_root_ring_slot_known_to_hold_a_root_not_self_verified_once_is_read_on_the_reread_and_the_floor_is_raised`、`rolling_back_while_the_targets_root_ring_slot_stays_unreadable_is_refused_before_any_write`；`crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs` 的 `rolling_back_while_a_root_slot_this_process_wrote_reads_not_self_verified_is_refused_before_any_write` 与 `crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs` 的 `rolling_back_after_the_floor_carrier_this_process_wrote_on_device_one_reads_not_self_verified_is_refused_before_any_write`（这个进程写过的根被改坏之后回退拒、盘上不变）（`research/prompts/m2-impl-r2-fixes-b-implementer-report.md` 第三节第 3 行；合入后验证一收窄成只管读不出、之后由改法 B 还原成全句，见 `research/prompts/m2-impl-merge-verify-1-implementer-report.md` 第三节第 1 小节）。
- 「准入」那一行的落点被拒也推、挂载处写行之后推：用户定案，原话在变更史；安全设计轮第三轮判决 `research/prompts/m2-safety-r3-main-verification.md`（会话里落点取不到 C283（准入失败时不先推发布就报 ENOSPC） 不管、只崩了再挂就把小盘挂死）。
- 回收门槛取一圈窗口：三方判决 `research/prompts/m3-ring-r2-main-verification.md` 第二节第 4、5、6 行（窗口字面第一次抽样没打中；锚写死成环里读得出的最新有效根；分配器、checker、影子账候选过滤、回退候选集同改）与 `research/prompts/m3-ring-r3-main-verification.md` 第二节第 3、4、5 行（第二次抽样没打中；读坏的中间根那一格门槛按最小 txg 算、根环表记最大，零轮；同改清单加 checker 的 I-3.1（已分配统计对得上） 丁-defer 门槛，合计五处；定案里「与装置里跑的一字不差」那一句改写）。用户定案：取一圈窗口、「抹」不做（原话在变更史）。无实验：窗口形态在三方副本上量过，没立实验号，副本上的数不是入库装置上的数，E139（按盘回退下界的收严形态） 重跑欠着（欠里记着）。
- 抬 F 那一遍并进挂载时读不出的槽、进程内记根先重读、抬 F 的上限只取窗口里的根：代码轮判决 `research/prompts/m3-root-ring-code-r1-main-verification.md` 第二节 K2-e、K2-b、K2-b2 三行（攻方原型量过，主 agent 复跑 115 份输出逐字节相同）与第三节表二之后用户定案那一段（E2、B2）。

**欠**：C282（环里最旧根没有定义）——定义与条款写在已定项 1 里，会红的检查仍欠；C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元）（与 defer 窗口取多大无关，病根在 D23（journal 的角色与格式） 已定项 14 的影子账）；C329（写行那次发布之前推抬 F 的空发布没有检查）；C563（读根环一槽读坏就当没有根，别的调用方没判）；C660（回收门槛改成一圈窗口之后按盘回退下界的装置要重跑）。

```

**出处 `.claude/kb/decisions/16-发布语义.md:78-90`（整段抄，未转述）**

```markdown
#### 已定项 2：checkpoint 的触发判据取时间与脏数据量，两者取先到

**定案**：`触发 ⟺ (now − 上次 checkpoint 的时刻 ≥ T_time) ∨ (脏字节数 ≥ T_dirty)`。两个阈值是**系统配置声明值**，挂载时求值一次（形态与 D21（权威态与派生态的分界）「单元几何挂载时求值一次」同型）。

**射程**：定的是什么时候触发，不定两个阈值取多少（已定项 5），也不定触发之后怎么切分事务（同样在已定项 5）。已知边角一样：`.claude/rules/fs-design.md` 硬要求 5 说的分支变量是那两个阈值、它们在挂载期间恒定，`now` 与脏字节数是被比较的量、会变，不是分支变量。

**依据**：

- 无实验：`∨` 这个形态是两条失败模式互相堵出来的，没有可量的量——只按脏数据量，空闲系统上最后一批写可以无限期不落、崩溃丢窗口无界；只按时间，突发写会把一个 checkpoint 的 journal 占用撑过环几何，而 I-8.1（环几何够大） 要的是「任一事务的最坏 journal 占用」有界。那个 `∨` 式子就是全部，没有魔数以外的自由度，`.claude/rules/fs-design.md` 硬要求 1「判据显式、可计算、写下来」因此满足。可量的量是两个阈值的取值与它们的可行域，在已定项 5。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/16-发布语义.md:91-111`（整段抄，未转述）**

```markdown
#### 已定项 3：journal 记录格式是四层之外的第三个独立冻结组件

**定案**：journal 不进 D15（格式冻结政策） 的四层链。它的记录格式——记录头（含已定待落地的事务号 + 提交标记、反向链、载荷校验和三笔增量）、点名项、载荷校验和的覆盖范围、事务边界——立成四层之外的第三个独立冻结组件；住系统配置的 journal 几何字段（记录尺寸、环大小、在飞记录数上限、tail 槽）是系统配置字段，随第 1 层冻。新记录格式走「新记录类型 + 系统配置 log-incompat 位」：环里有这种格式的记录时置位、环干净时清掉，不认识就拒绝挂载；永久要读得懂的只有判定「这条记录不施加」要用的那几个头字段（magic、类型、自述长度、`jsn`、`checkpoint_txg`、头校验和）。

**冻结前置**（不早于它嵌进来的每一个外部定义）：第 2 层（点名项里的位置条目）；落点粒度那一格（D3（空间分配） 已定项 7 欠着的处置）；D18（块里携带什么信息） 已定项 11 的单元类登记表（点名项里的单元类型标签）；树 ID 所在的树表单元——它今天没有冻结层归属，所以这一条前置今天就挂着。**冻结时刻是提交 `format-frozen/journal-record.toml` 的那个 commit**，不许写成「第一个写出记录的镜像」。

**射程**：只答 journal。树表单元、实例表单元与根记录的冻结层归属仍是 C246（树表单元没有冻结层归属） 那一格。三样已知边角：

- **置位与清位是运行期动作**，不走提交这条时间线，要另配一道运行期的闸（写第一条新格式记录之前先持久置位，缺位即拒绝写），随组件的 spec 一起落。
- **今天还冻不了**，挡着五样：点名项 56 字节只定了总宽，构成只有 [layout/01-first-txn.md](../layout/01-first-txn.md)「六、journal」的预想表，也没给加密字段留位（D23（journal 的角色与格式） 已定项 1 的 D 条「项的大小由记录类型决定」管着这件事）；记录头登记的格式常量是 78，三笔增量落地后是 95；spec 文件不存在；log-incompat 位还没有位号，也没进 D15（格式冻结政策） 那张 feature bit 记账表；冻结前清单第 1 项量宽度、第 7 项量未定分项，两项都看不见 `jsn` 解释口径打架那笔欠账，按今天的清单冻这个组件时两项都不红。
- **「什么时候算环干净、可以清位」要按本工程的重放下界定**（D23（journal 的角色与格式） 已定项 14 的水位），不照搬现役实现。

**依据**：

- 无实验：冻结层归属是格式政策，能核的是各层之间的依赖顺序，没有可量的量；记录头宽度那笔字节账在 C94（登记的格式常量与后来的定案对不上）。
- 三方论证（材料 `_d16-items134-background.md`，产物同目录 `d16-items134-*`：`d16-items134-sonnet-output.md`、`d16-items134-opus-output.md`、`d16-items134-local-output-s1.md` 与 `-s2.md`）：正推腿判「独立组件」是唯一与 D15（格式冻结政策） 已定项 1 自洽的答案，反推腿补上 log-incompat 位之后演进那一格赢，并打中两处前置；本地腿两次抽样的反例把字段挂错了层，主 agent 核后不采纳。判决取独立组件 + log-incompat 位，冻结前置按反推腿收严。
- 现役参照：XFS 有 `XFS_SB_FEAT_INCOMPAT_LOG_ALL`，在主系统配置里置位、日志里没有受保护的条目时清掉（本机树 `fs/xfs/libxfs/xfs_format.h:426-441`、`fs/xfs/xfs_mount.c:1560`、`:1629-1639`），**未在本项目验证**；一处已知差异：XFS 的日志要重做逻辑与物理混合的条目，本工程的记录只承载指针层目标态。
- 用户定案，原话在变更史。

**欠**：C94（登记的格式常量与后来的定案对不上）、C119（冻结组件没有 spec 文件）（journal 记录格式是第三个，那一行的简称还写「两个」）、C246（树表单元没有冻结层归属）、C199（实例代号递增与 jsn 断号即止互相矛盾）（冻它之前要先还）、C225（组件的冻结时刻绕开门禁触发）。

```

**出处 `.claude/kb/decisions/16-发布语义.md:112-131`（整段抄，未转述）**

```markdown
#### 已定项 4：一次发布整体施加或整体不施加，第二份 replay 停在句法层

**定案**：「施加一条记录在指针层上做什么」那个二选一取「一次发布整体施加或整体不施加」。记录核对器那第二份 replay 实现因此停在句法层：前缀判定 + 择根 + 续做，不实现树语义、不重新生成祖先。与 D13（验证路线） 的关系照收窄后的四样：

| # | 定了什么 | 依据 |
|---|---|---|
| ① | **共享**：与实现只共享 D13（验证路线） 已定项 5 那份生成常量；与 checker **默认不共享**，要共享得另行论证；不与 RefFS 共享恢复代码 | D13（验证路线） 冲突 1 的折中管的是 checker 自己遍历、扫描两个方向之间，不授权两个判决器之间共享（两条云端腿一致） |
| ② | **入参拆成三份**：崩溃态镜像（择根与前缀判定看它）、记录流（崩溃态镜像环里的 journal 记录）、实现恢复后的镜像（比对的对象） | 恢复本身会改盘（实例切换写行、重发在飞 checkpoint，D23（journal 的角色与格式） 已定项 14），D13（验证路线） 定案写的「崩溃后镜像」在 harness 先恢复、后核对的顺序下是两份 |
| ③ | **缺一个数**：前缀第一条要的「在飞记录数上限」本仓没有值（I-8.3（重放前缀严格连续）；D23（journal 的角色与格式） 已定项 1 的 C 条要它进系统配置） | 那个数在本仓零命中 |
| ④ | **合谋测试**：记录核对器是第四个判决器；每个判决器都要有一个只有它抓得住、而且对可见状态或结构有害的坏实现。「摘掉根槽前的屏障」不够格——它抓到的缺失只伤记录核对器自己；候选待找：原来的候选回退行 W 截断随管理员回退改成挂着时的向前发布一起删掉（D23（journal 的角色与格式） 已定项 14），第二轮量到截断在今天的代码上本来就走不到。O3（独立规约执行器） 今天不在 [verification-build.md](../verification-build.md) 的落地步骤里 | 反推腿 4-D |

**射程**：定的是第二份 replay 做什么、不做什么，**不定它要多少人工**——零代码时那个数没有可核的答案，只有 research 里三样工作的模型等价物当下界替身（`e42_transaction_records.rs` 240 行、`e77_publish_order.rs` 506 行、`e78_replay_start.rs` 556 行，反推腿 `wc -l`），第四样（把期望态算出来）一行模型都没有。指针层其余部分（允许部分施加时换哪个指针、祖先谁来 COW、落点谁分配）仍归 C284（施加一条记录在指针层上做什么没有定义）；D8（核心索引结构） 已定项 13（同一次发布里同 key 的中间态折叠）靠这一支：施加只落在发布边界上，同一次发布里的中间态才不会被 replay 出来。

**依据**：

- 无实验：第二份实现必须独立实现什么、允许共享什么，是验证路线的分工，没有可量的量；「要多少人工」在零代码时也没有可核的答案，只有行数下界替身。
- 三方论证之后的用户定案收窄问法（问的是「那第二份实现必须独立实现什么、允许共享什么」加「它在合谋测试里站什么位置」），原话在变更史。

**欠**：C284（施加一条记录在指针层上做什么没有定义）。

```

**出处 `.claude/kb/decisions/16-发布语义.md:132-156`（整段抄，未转述）**

```markdown
#### 已定项 5：`T_time` 与 `T_dirty` 取多少，丢失窗口的口径

**定案**：`T_time` 默认值 5 s，取可行区间 [200 ms, 5 s] 的上端——默认丢失窗口是 5 秒，取更小只减少摊销、不多买任何东西，拦住丢失的是那 5 秒本身。**`T_time` 用户可设，不限制**：挂载时传的值覆盖盘上的字段并回写；超出 [200 ms, 5 s] 不拒绝，只给警告（用户 2026-09-20）。`T_dirty` = 2 GiB（初值，写代码时再调）；它不由丢失窗口定——`∨` 规则下时间那一支已经保证了窗口，`T_dirty` 管的是**量**，它就是脏页内存预算本身。**`T_dirty` 是可调值，不是格式常量**：系统配置里那个字段是格式，字段里放什么数不是。**挂载时还要再夹一次**：有效 `T_dirty` = min(系统配置里那个字段, 环长 ÷ F)；环长是 mkfs 参数（默认 768 MiB、约束环 ≤ 设备容量 ÷ 4），所以「环装不装得下 `F ×` 最坏占用」不由一条 mkfs 时的下界保证，改成挂载时把脏字节预算压下去；默认环下有效值 = min(2 GiB, 256 MiB) = **256 MiB**，系统配置里 `T_dirty` 仍写 2 GiB、字段不动。

**丢失窗口的口径**：取号 = 事务被分配进某个 checkpoint 那一刻，checkpoint 的切分只关闭成员资格——触发之后不再分配事务进它，已分配的事务把各自的单元写完，发布阶段等全部单元落盘（任何 COW checkpoint 都要等最慢的那次 I/O，ZFS 的 txg sync 同样等最慢的 vdev，未在本项目验证）。事务切分纪律是一事务一单元。落到取号上，事务切分纪律把一次写请求按单元切成若干事务各自取号，一个事务最多写一个单元的用户数据，**同一次请求切出的若干事务按其单元的 key 升序取号**（提交路径要记「同一请求内取号序与 key 序不一致的次数」、非 0 判红）。单元写的重试预算是墙钟 T_retry（具名可调参数，不是格式常量；用尽即按失败的性质走实例切换或转只读，D23（journal 的角色与格式） 已定项 14——它同时是丢失窗口的加项、挂载内实例切换的判据、以及连续切换计数的时间基准）。**由此崩溃最多丢的是 T_time + N_switch × (T_retry + T_redo)，不是恰好 5 秒**——一次挂载允许 N_switch 次实例切换，每次都要熬完 T_retry 再把在飞 checkpoint 重做一遍（T_redo），而重做期间开放的那个 checkpoint 发布不了、只能一直攒。这里的 `T_time` 是本次挂载生效的那个值，不是 5 秒这个常数；`T_time` 可调之后，2026-08 用户定案的「崩溃最多丢 5 秒的写」降级成**默认值下的承诺**，不是全局承诺。**「上次 checkpoint 的时刻」在重发时取重发那一刻**（取原来那一刻的话，切换期间触发条件一直成立而在飞深度只有两个，触发只能空转）；在飞 checkpoint 深度 = 开放 + 发布中两个；并发 fsync 的返回时刻被同 checkpoint 的其他事务拖住是组提交固有的代价。这是已定项 14 那条「快照发布点钉在这个 checkpoint 全部数据变更之后」的可执行形态。

**射程**：定的是两个阈值取什么数与崩溃最多丢多少，不定环几何本身、也不定重做期间开放的那个 checkpoint 会不会堵住写者（D28（挂载期承诺量） 已定项 3 点名的那一格口径空白，D16（发布语义） 没写）。七样已知边角：

- **`T_time` 的下界只适用于非 fsync 触发的发布**：那步推导以「一窗多次操作共享一次发布」为前提，fsync 密集负载下窗口由 fsync 自身触发。引用这条下界时连这句一起引；上界 5 s（用户定的丢失窗口）不受影响。
- **「向上取整到 200 ms」与「预算取 0.1」都是选的，不是算出来的**：0.2 给 94 ms、0.05 给 410 ms（外推），下界随预算线性挪，改预算就要重算。
- **由 `T_dirty` 推出的两个派生数按已定记录格式算**：D23（journal 的角色与格式） 已定项 12 的 4 KiB 定长记录（头 95 B、每项 56 B、71 项/条）下，同一个 `T_dirty` 满窗是 1847 条记录、占用 **7388 KiB**，I-8.1（环几何够大） 的环下界 F = 2 时 **14776 KiB**、F = 3 时 **22164 KiB**。两个阈值的定案值不变，变的只是这两个派生数的口径。
- **一事务一单元下的记录账与这个派生数打架**：按六个负载点算，每次 fsync 的记录数等于数据单元数（seq 批 1 是 8 条、批 10 是 80 条），journal 恒占用户数据 12.5%，`T_dirty` 满窗 65 536 条记录、256 MiB，是字节表预想的 64 MiB 环的 4 倍。两条口径原样并列，改法要用户定。
- **本机持续写带宽没有实测**（E58（校验和粒度的端到端代价） 那个 2045 MiB/s 是校验和实验里的顺序口径，不是写带宽，不许拿来换算），所以「5 秒能写多少字节」算不出来。
- **`T_dirty` 与 `T_time` 是两种规则**：`T_dirty` 走夹取——有效值 = min(字段, 环长 ÷ F)，不动；`T_time` 走警告——越界不拒绝，只给警告。同一段里两个字段两种规则，不按同一规则读。
- **`T_time` 默认值后期按盘给推荐**（用户 2026-09-20）：要先有一批盘的实测与探测输入，落在 D12（目标介质） 那条线上（第一条布局线是纯 SSD）。记成 `T_time` 默认值的重开条件，不是现在要做的事。

**依据**：

- E70（checkpoint 两个阈值的可行域）：两个阈值的可行域——`T_dirty` 的上界绑环几何但不承重（64 MiB 环就允许 16 GiB 脏数据，实际卡住它的是脏页内存与崩溃丢失窗口），`T_time` 的下界由摊销预算反解，按最差负载 multistream 定而不按主负载 seq 定，取摊销预算 `B = 0.1` 块/操作 ⇒ 反解 547 次操作 ⇒ 197 ms、向上取整记 200 ms，seq 负载一点下界都不需要。
- E44（序号位宽的本机实测与代价）：本机 2785 fsync/秒，是把摊销预算反解成 `T_time` 下界的那个速率。
- E91（已定格式下的环账与准入账）：按已定记录格式重算 `T_dirty` 满窗的 journal 占用与环下界，旧的 32 B/条目口径少算 1.80 倍。
- 用户定案三次（两个阈值、丢失窗口口径的收口、挂载时再夹一次），原话在变更史。

**欠**：C310（事务切分纪律与记录数口径打架）、C319（请求内单元按 key 升序发出没有条款也没有检查）、C425（T_time 小于一次发布耗时时触发空转）。

```

**出处 `.claude/kb/decisions/16-发布语义.md:157-179`（整段抄，未转述）**

```markdown
#### 已定项 6：发布计数器，四个等号写成一条

**定案**：每次发布把 checkpoint_txg 加一——fsync 触发的发布也是发布，没有「小发布不记号」的例外；根槽的轮转键（D22（单元原子性怎么合成） 已定项 2 的「区域 = txg mod R」里那个 txg）、记账的代（D5（快照 / 空间记账机制） 已定项 2）、恢复重放的水位（D23（journal 的角色与格式） 已定项 14）都是这同一个计数。不另设「发布代号」字段——它与 checkpoint_txg 是同一个东西。**四个等号**：fsync = 提前发布、区域 = txg mod R、一个 txg 一个号、记账代 = checkpoint 号，四处读的是同一个数，哪个实现读岔一处就会撞根槽或读错账。

**四条骑手条款**（与定案同日采纳，不是可选项）：

1. **根记录必须带 32 位实例代号**（与 D23（journal 的角色与格式） 已定项 9 的 journal 实例代号是同一个计数），择新在 checkpoint_txg 平局时按实例代号高者赢。机制：丢盘回退 + 盘失而复得会造出两条同 txg 的自证合法根——journal 靠实例代号分辨时间线，根环此前什么都没有。字段落点在 D22（单元原子性怎么合成） 已定项 7。
2. **记账 key 的「代」段宽度 ≥ 48 位**：按本机 2785 发布/秒，32 位 17.8 天回绕。记账 key 的宽度算术自陈是假设，要按此重算。
3. **`T_time` 下界的推导（197 ms）只适用于非 fsync 触发的发布**——那步推导以「一窗多次操作共享一次发布」为前提，fsync 密集负载下窗口由 fsync 自身触发。引用已定项 5 的下界时连这句一起引；上界 5 s（用户定的丢失窗口）不受影响。
4. **记账条目的跨发布合并不再可行**（代进了 key，「后者胜」并不掉不同代的两条）⇒ 每代「统计量数 × 2」次写就是实付值，不是上界。代价，照实记。

**射程**：定的是这个计数怎么加、四处怎么共用一个数，不定根环几何本身（D22（单元原子性怎么合成） 已定项 2）、也不定 journal 实例代号什么时候递增（D23（journal 的角色与格式） 已定项 9）。已知边角一样：这条定案当天的对抗测试实际跑在两条腿上——本地腿那一份后来被收窄后的字词损坏检查判红（`proposed proposed`，实词自复读那一类），按 `.claude/rules/three-way-inference.md`「闸判红之后那一轮作废」，**不许再拿它当「三方一致」引用**；定案本身不因此推翻，两条云端腿各自的判定性产出仍在。

**依据**：

- E48（根环几何的可行点）：轮转键就叫 txg 且逐次发布递增，9 个可行点全部建在这个读法上。
- E50（根环槽数的上下界）：源码逐字「一圈时长（秒）= 槽总数 ÷ fsync 率。每次 fsync 发一次根」——已入库的根环算术只在「每次发布 checkpoint_txg 加一」这条等式下为真，不钉它那批实测追溯性作废。
- E53（丢一整块盘之后根环还挂不挂得上）：丢盘回退 + 盘失而复得的那条历史，就是骑手条款 1（根记录带实例代号）要分辨的东西。
- E44（序号位宽的本机实测与代价）：本机 2785 发布/秒，32 位 17.8 天回绕——骑手条款 2 的「≥ 48 位」是从这个速率来的。
- 对抗测试（用户要求先攻再定）的原始输出 `c85-publication-counter-forward 那一轮的腿报告（产物在 git 里）`、`c85-publication-counter-reverse 那一轮的腿报告（产物在 git 里）`、`c85-publication-counter-local.out`；用户定案，原话在变更史。

**欠**：C85（发布计数器的四个等号没写死）（条款已写成一条，会红的检查仍欠）、C88（根环的时间线判别未实现）（骑手条款 1 的 checker 侧，反推腿那一轮找出来的可达破绽）。

```

**出处 `.claude/kb/decisions/16-发布语义.md:180-196`（整段抄，未转述）**

```markdown
#### 已定项 7：发布的持久顺序

**定案**：一次发布的持久顺序恒为 COW 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）→ 系统配置槽 → 屏障；fsync 等系统配置轮换持久之后才返回。恢复重放在施加任何记录之前，必须逐项验证点名单元的校验和。**系统配置槽在根槽持久之后再更新，每个 checkpoint 一次**，它是发布序列的最后一步、进层 0 枚举，一次发布的段序列因此唯一。**抬 F 那一串（准入与卸载）在它的第一次发布之前多一步**：先把新 F 写进每块盘的系统配置、过一道屏障（已定项 1「抬 F 那一串」）；这一次系统配置写不属于任何一次发布，每次发布内部的顺序照旧。**fsync 返回条件多一句**：新实例在本实例写成的根覆盖两块盘之前，fsync 不返回（做法与次数在已定项 8）；管理员回退是挂着时的一次向前发布（D23（journal 的角色与格式） 已定项 14），它的根持久之后才向管理员确认，与 fsync 同一个返回条件；实现待派，排在实审 A3c 之后（同在 `transaction.rs`），代价另登记一个小实验量。

**射程**：定的是一次发布内部这几步的先后与 fsync 什么时候返回，不定屏障的真机代价、也不定根槽本身的冗余形态（D22（单元原子性怎么合成） 已定项 2）。三样已知边角：

- **「fsync 等系统配置轮换持久之后才返回」的射程**：返回之后这一代的根只由那一个根槽罩着（根槽不镜像），见证它的系统配置已落盘。同一实例里退一代，靠 journal 重放追得上；**每个新实例（每次可写挂载、恢复、切换）的第一个根在下一次发布之前是单点**——那个槽读不出或它所在的盘掉了，恢复退到上一个实例的根，重放按严格前缀停在实例边界（D23（journal 的角色与格式） 已定项 14 的注 1），这次挂载里 fsync 已返回的事务丢，一个故障就够。要不要让新实例先把根写到两块盘上再确认，是已定项 8。
- **屏障口径**：两道 FLUSH + 根槽 FUA + 系统配置轮换之后一道屏障 = 每次发布四个序点，比 D25（目标负载优先级） 推导的两个多两个——多的两个是第二道屏障与「发布返回前加屏障」的价钱，知情接受（后者用户 2026-09-27 定，起因是 C577（系统配置没见证到的最新根，乙罩不到） 那条用例在抬 F 之前更早红：发布路径根槽 FUA 之后轮换系统配置就返回，没有屏障）。
- **第二道屏障买的是记录流完整性**，不是数据完整性：数据完整性的必要屏障恰一道（根槽之前）；不上第二道有 7 类「根在案而记录缺席」的状态，记录核对器与反向链的输入有洞。用户定案「需要」，两道都上。

**依据**：

- E77（发布的持久顺序）：崩溃子集穷举——不上根槽之前那道屏障，1024 个崩溃状态里 504 个违例；拿掉「重放施加前验证点名单元」这一步，b_rs 臂 63 处静默嫁接，所以它是承重步骤。
- 用户定案，原话在变更史；「发布返回前加屏障」（系统配置轮换之后那一道，序点从三个变四个）是用户 2026-09-27 定的，起因、弹窗事实与岔路在 `research/prompts/c554-fix-forks.md` 第 4 行，实现已合入（`crates/singlefs-core/src/transaction.rs` 的 `persist_the_root_then_rotate_the_system_configuration`，实 C577（系统配置没见证到的最新根，乙罩不到） 报告 `research/prompts/m2-impl-c577-barrier-implementer-report.md`），代价仍另登记小实验量。

**欠**：C26（屏障数从没量过）（真机每次 fsync 的屏障数仍没量）。崩溃点重放 harness 的两条自证用例（摘掉根槽前的屏障必须红、关掉重放验证必须红）随 C76（发布时序没有权威落点） 已还清。

```

**出处 `.claude/kb/decisions/16-发布语义.md:197-218`（整段抄，未转述）**

```markdown
#### 已定项 8：暖机取戊，新实例的根先覆盖两块盘

**定案**：新实例在本实例写成（FUA 返回）的根覆盖两块盘之前，不让任何 fsync 返回；做法是连推空发布，第一版几何至多 R = 3 次。**第一版实付 2 次，而且它是格式常量不是运行时谓词**：根环三个区域的设备归属第一版写死 0 / 1 / 0（D2（RAID 条带策略） 已定项 7），于是 txg 1 落区域 1（盘 1）、txg 2 落区域 2（盘 0），两次正好覆盖两块盘 ⇒ 次数由归属唯一确定。登记成 `format-const` WARM_UP_EMPTY_PUBLISHES = 2 与 NEW_POOL_FILE_CREATION_TXG = 3（[layout/01-first-txn.md](../layout/01-first-txn.md) 八），条款在 D22（单元原子性怎么合成） 已定项 16 第 5 句。**后续挂载与挂载中途崩了再挂**：次数现算——写行那次发布的 txg 取 D23（journal 的角色与格式） 已定项 14 注 3 那个下界本身（max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1），之后每次空发布加一，按根环落点看落哪块盘，推到本实例的根覆盖每块盘为止，至多 R 次；取号之前的准入与取号之后推发布用同一份计划（`crates/singlefs-core/src/mount.rs` 的 `warm_up_publish_txgs`）。

**射程**：定的是新实例确认之前要做什么，不定实例切换本身的判据（D23（journal 的角色与格式） 已定项 14）、不定空发布写出多少单元（已定项 9）。四样已知边角：

- **前置 C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元） 的条款那一半已满足**（用户取影子账，条款写进 D23（journal 的角色与格式） 已定项 14 与 I-7.4（近 K 代块未被复用）），暖机已生效；**检查那一半仍欠**：多次挂载的录制流已有（层 0 第二条流），逐句核仍差格 1 的专门断言，C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元） 因此仍在未还表。
- **归属改了那两个格式常量要重算**（加盘或开条带表时）；「至多 R = 3 次」那句仍是几何上界，不是第一版走得到的次数。第一版实付 2 次这一档没有走三方论证，可推翻。
- **代价**：改已定项 7 的 fsync 返回条件（新实例确认前多一个条件）；新池新建文件的 checkpoint_txg 从 1 变 3，多两条空记录与两次根写；D28（挂载期承诺量） 已定项 3 的切换预留加「每次切换至多 R = 3 次空发布」，按 D28（挂载期承诺量） 已定项 4 现算的 c_max 取（池规模 9 块 ⇒ 27 块）；空发布占根环槽位，把旧的非空根更早挤出环。
- **另两臂**：甲（不补）判据 2 一个故障就丢；丁（重放跨实例边界）出局。
- **白推**：写行 txg ≡ 2 (mod 3) 时第一次空发布落在已经覆盖的盘 0 上，白推一次——副本装置上量到每次多写 147968 字节、多 2 道屏障、多 1 次 FUA、多占根环一槽，只重开不写时每次挂载都落在这一档，24 槽的根环 8 次挂载就整圈转完（入库装置上没重做）。这是写行 txg 取下界本身的直接后果，认下；「普通挂载的写行 txg 往后挪到只推 1 次的值」那条改法被攻过零轮，没采纳。

**依据**：

- E150（回退复用被抛弃的根引用的单元）：无暖机时 1 个故障就落到被抛弃的根上，暖机之后要 4 个；影子账与墓碑两种修法都零违例，用户取影子账。
- E148（提交固定点按两棵记录树重算）：空发布在池规模、提交块聚簇时 9 块，切换预留那一半按它算。
- E142（新池新建文件的干跑）：checkpoint_txg 1 → 3 的改动落到新池新建文件的字节上之后，262162 个状态零违例。
- 两轮三方论证 `research/prompts/c199-r2-main-verification.md`；用户定案，原话在变更史。
- 后续挂载那一句与白推：三方第一轮 `research/prompts/m2-presumed-clauses-r1-main-verification.md` K2（推到覆盖为止在后续挂载、回退、挂载中途崩了再挂三种起点上都构造不出不够），用户 2026-09-23 定照今天写、认白推，原话在变更史。

**欠**：C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元）（检查那一半）。

```

**出处 `.claude/kb/decisions/16-发布语义.md:219-233`（整段抄，未转述）**

```markdown
#### 已定项 9：一次空发布也写单元

**定案**：记账树存在时就写——空发布也是发布，按 D5（快照 / 空间记账机制） 已定项 2「每行每发布重写」重写记账行，连带记账树节点、分配记录、映射条目与树表单元。第一次可写挂载的暖机时树表 0 条 ⇒ 零单元；以后的空发布按 D28（挂载期承诺量） 已定项 4 现算的 c_max 块。**树表 0 条的一版上写行的那次发布写两个单元**：实例表与一片分配记录节点，分配记录树根指针住根记录（D22（单元原子性怎么合成） 已定项 7）；被换下的上一版实例表与上一片分配记录节点记成已释放，重开时从这片节点重建这一版的账。根记录那一项只有树表 0 条、写过行的那一版写非零，mkfs 的第 0 代与带文件的一版写全 0，零单元发布照抄上一版。

**射程**：定的是空发布写不写单元、写哪几样，不定 c_max 怎么算（D28（挂载期承诺量） 已定项 4 按当时结构现算）、也不定空发布什么时候推（推抬 F 在已定项 1，暖机在已定项 8）。写行那次发布写的那片分配记录节点，单元头树 ID 写 0（实现今天的取法：那一版的树 ID 水位是 mkfs 的 65536，写分配记录树的 65538 会让 I-7.8（根记录树 ID 水位不低于全池最大树 ID） 当场红），没有条款；它进不进中央映射也没有条款——那一版没有映射树，D19（块指针的结构与宽度预算） 已定项 8 的自举豁免三类里没有它。

**依据**：

- E148（提交固定点按两棵记录树重算）：空发布的固定点按两棵记录树重算是池规模 9 块、新池新建文件规模 4 块，「以后的空发布按现算的 c_max 块」那句的数出自这里。
- E142（新池新建文件的干跑）：第一次可写挂载的暖机那两次空发布逐字节写出来，树表 0 条时零单元。
- 用户定案 2026-09-23（写行那次发布在树表 0 条的一版上写实例表与分配记录节点两个单元、分配记录树根指针住根记录），原话在变更史；实现与用例点名在 [checks-owed.md](../checks-owed.md) 已还清表 C512（树表 0 条的一版上被换下的实例表记在哪没有条款） 那一行。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/16-发布语义.md:234-249`（整段抄，未转述）**

```markdown
#### 已定项 10：发布粒度取攒批，不是每个事务发一次

**定案**：攒够一批脏节点（或到时间）再发布一次根，不是每个事务发一次。

**射程**：定的是频率，不定一次发布事件内部的顺序——那在 `litmus/commit-publish.litmus`，它钉的是写块内容 → `smp_wmb` → 写 generation，对频率一字未提。两样已知边角：

- **净代价分布**：格式侧几乎为零（journal 本来就要有），验证侧一大笔——中间态语义改写（已定项 12）、第二份 replay 实现（已定项 4 与 13），再加 D13（验证路线） 的「Merkle 根链差分」粒度变粗：一代根里塞了 K 个事务的改动，定位精度从「哪个事务」降到「哪个 checkpoint」，而 D13（验证路线） 已写明全量档的真代价是定位成本。
- **发布语义决定「走几次」**：每事务发根 → K 批走 K 次；checkpoint → 同一窗口内的 K 批只走约一次，各批只留一条 journal 记录（小、顺序写）⇒ 分批大删除那一格的元数据写放大从 `K × M × H` 降为 `checkpoint 数 × M × H`。

**依据**：

- 无实验：定案靠两条互相独立的推理，当时没有可量的量。① 它与 D5（快照 / 空间记账机制） 已定的 ZFS birth txg 模型天然相容——ZFS 的 txg 本身就是 checkpoint（`zfs_txg_timeout` 默认 5 秒，或攒够一定脏数据量），一个 txg 批量大量操作、txg 结束才更新一次 uberblock；采纳 checkpoint 之后 birth txg 的号粒度问题自动消失，一个 txg 一个号，deadlist 的比较粒度就是 checkpoint 粒度、不必重算（未在本项目验证）。② **根是丢失写的最终见证者**：检测「盘上这块是不是它应该是的那个版本」需要一个外部见证者，块自身办不到（合法的旧版本与合法的新版本在块内层面同构，D18（块里携带什么信息） 有完整论证），而见证者必须在不同的失败域上——父节点若在同一次事件里也没写成就要祖父，递归到根 ⇒ checkpoint 发布的那个根是这条递归的终点，它的持久化必须比任何单个块更可靠（ZFS 的形态是 4 个 label × 每个 128 KiB 的 uberblock 环、且偶数与奇数 label 分两阶段写；本工程根自身的冗余形态由 D22（单元原子性怎么合成） 已定项 2 定）。频率的可量代价（两个阈值的可行域）在已定项 5。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/16-发布语义.md:250-262`（整段抄，未转述）**

```markdown
#### 已定项 11：没有任何操作可以绕过 journal 直接发根

**定案**：所有结构、所有操作先进同一个 journal、共用同一个 checkpoint 节奏。fsync 的效果是「提前触发一次发布」，**不是另开一条日志格式、另一套恢复逻辑**。判据是：journal 是所有提交的必经之路，还是某些路径的旁路——前者是一套机制、持久化分两级，合规；只有 fsync（或只有元数据、只有小写）走 journal、别的直接发根，就是 `.claude/rules/fs-design.md` 点名的「为 fsync 单开一套日志树」。

**射程**：说的是挂载之后的操作；mkfs 种子写第 0 代根不经 journal（D13（验证路线） 已定项 1 自己列的根槽三种写者之一，mkfs 一条记录都不写），不在这句的射程里。已知边角一样：D23（journal 的角色与格式） 已定项 1 的轴一取甲压在这一条上，重开它要回答的依据里有这一条。

**依据**：

- 无实验：它是 `.claude/rules/fs-design.md`「一个事务层，所有结构共用」在发布这一格的形态，判的是机制的形状（必经之路还是旁路），不是一个能量出来的数；不写死就会退化成「为某个路径开第二套事务机制」。每次持久化的写量是另一个问题，归 D23（journal 的角色与格式） 已定项 1。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/16-发布语义.md:263-275`（整段抄，未转述）**

```markdown
#### 已定项 12：中间状态的合法性语义改写成「根 + journal 任意前缀」

**定案**：D8（核心索引结构）「中间状态必须每一个都是合法的文件系统状态」改写为「**根 + journal 任意前缀，replay 之后合法**」。

**射程**：定的是合法性怎么判，不定前缀本身怎么截——前缀判定的完整口径是 D23（journal 的角色与格式） 已定项 14 的五条，缺一不可。已知边角一样：块复用那一侧的真正约束比 checkpoint 边界更严，按 journal 位置说（已定项 15 与 D22（单元原子性怎么合成） 已定项 4）。

**依据**：

- E42（一事务几条记录）：1601 个前缀里 1400 个被前缀切开（87.4%），一事务一条那一臂非法前缀恒 0——那是「任意前缀 replay 之后合法」的直接形式；跨多条而无边界字段时非法前缀恰等于被切开的前缀数，加边界字段之后 0。这条语义在模型上是可数的，而且数得出它什么时候破。
- 用户定案，原话在变更史。

**欠**：C42（残留记录冒充合法前缀）（上一条时间线留在环里的记录会被下一次恢复当作合法前缀重放）。

```

**出处 `.claude/kb/decisions/16-发布语义.md:276-288`（整段抄，未转述）**

```markdown
#### 已定项 13：这份 journal replay 归记录核对器，不归 O2（独立解析器 + checker）

**定案**：D13（验证路线） 已定 O2（独立解析器 + checker） 是一元谓词、只判单个镜像，所以这份 journal replay 归「记录核对器」。那是第二次实现 replay，而 replay 正是崩溃语义最容易出**共同误解**的地方——D13（验证路线） 的「三 oracle 合谋测试」要防的就是这个。**这是 D16（发布语义） 最贵的一笔。**

**射程**：定的是归谁，做什么与允许共享什么在已定项 4。已知边角一样：D23（journal 的角色与格式） 轴一定甲之后「redo 为空」，最贵那一笔的大头随之消失，只剩前缀判定 + 择根 + 续做。⚠️ **「redo 为空」只对「根已持久」的发布成立**：E42（一事务几条记录） 实测 1601 个前缀里 1400 个被前缀切开（87.4%），被切开的比例与边界字段在不在无关——变的是这 1400 个非法还是合法（无边界字段臂 1400 非法，有边界字段臂 0）⇒ **恢复期重放非空是常态，不是边角。**

**依据**：

- 无实验：哪个判决器实现 replay 是验证路线的分工，没有可量的量；第二份实现的成本量级与它必须独立实现什么在已定项 4。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/16-发布语义.md:289-301`（整段抄，未转述）**

```markdown
#### 已定项 14：快照发布点与 checkpoint 边界对齐，钉在全部数据变更之后

**定案**：快照发布点必须与 checkpoint 边界对齐，且位置钉死在「本 checkpoint 全部数据变更之后」，快照不许出现在 checkpoint 中间。同一 checkpoint 内建两个快照是允许的：它们捕获同一状态、其间 deadlist 为空；但 checker 里的「上一个快照」要用 `(txg, 快照 id)` 打破并列。

**射程**：定的是快照点落在哪，不定快照本身的记账模型（D5（快照 / 空间记账机制） 与 D6（快照实现模型））。已知边角一样：这条规则的可执行形态是已定项 5 那段丢失窗口口径——发布阶段等全部单元落盘，快照点就落在那之后。

**依据**：

- 无实验：理由是机制性的，没有可量的量——`birth` 的分辨率只到 checkpoint，**块无法表达「我生在本 checkpoint 里快照点之前还是之后」**；不钉死它，D5（快照 / 空间记账机制） 那张边界表里 `birth == S.txg` 与 `death == S.txg` 两行会同时失去确定答案，而它们一个是**数据丢失**、一个是**空间泄漏**。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/16-发布语义.md:302-321`（整段抄，未转述）**

```markdown
#### 已定项 15：checkpoint C 中产生的释放，C 发布之前不进可分配集合

**定案**：checkpoint C 中产生的一切释放，在 C 被发布之前不得进入可分配集合。两个理由，第二个是 D16（发布语义） 独有且更严的：

1. **回滚**：崩溃后可能回到 C−1 的根，而生于 C 之前、死于 C 的块在那时仍然活着。
2. **已定项 12 把 D8（核心索引结构） 的中间态语义改写成「根 + journal 任意前缀，replay 后合法」**：若一个块在 journal 位置 p 被释放、位置 q > p 被重新分配并写入新内容，那么重放到 `p < r < q` 的前缀会得到一棵指向该地址的树，**而盘上的字节已经是 q 的内容了**（物理写不受 journal 位置约束）。**这不是理论风险，是必然的错。**

**射程**：定的是「什么时候可以回可分配集合」这条下界，不定窗口取多大（已定项 1）。三样已知边角：

- **第二条理由是按 journal 位置说的** ⇒ 真正的约束是整个 journal 重放窗口，不是 checkpoint 边界；D22（单元原子性怎么合成） 已定项 4 收在那一形，缺了它的失败模式是「可检测（父指针校验和失配），不可恢复」。
- **窗口容量与 ENOSPC 直接相关**（D3（空间分配）：释放空间这个操作本身不许申请空间），而攒批让「建了又删」在同一窗口内闭合的概率大幅上升 ⇒ **defer 窗口落在热路径上**。
- **外部线索**：ZFS 有同型机制且更保守，`include/sys/txg.h` 的 `#define TXG_DEFER_SIZE 2 /* Number of txgs worth of frees we defer adding to in-core spacemaps */`，**未在本项目验证**；⚠️ 那个 2 为什么是 2，没查到一手解释，本工程的 defer 窗口要按自己的重放语义推，**不许照抄**。

**依据**：

- E82（准入的在飞合成）：把窗口内产生的释放记成可用的那条臂，20 格吃进 defer 该扣住的块——这条规则被静默绕过时的现象是可数的，准入侧的形态因此写进了 D28（挂载期承诺量） 已定项 2。
- 用户定案，原话在变更史。

**欠**：无（块复用那一侧的会红检查由 I-7.4（近 K 代块未被复用） 承担，欠账在已定项 1 列的那几笔）。

```

**出处 `.claude/kb/decisions/16-发布语义.md:322-334`（整段抄，未转述）**

```markdown
#### 已定项 16：checkpoint 与消息缓冲买同一样东西，E7（离线索引 harness） 的对照组钉死

**定案**：checkpoint 与 D11（索引节点要不要留消息缓冲区） 的消息缓冲区买的是同一样东西，差别只在**摊销窗口住在哪**：checkpoint 的窗口住**内存**、不进格式；消息缓冲的窗口住**盘**、必须进格式。⇒ **D11（索引节点要不要留消息缓冲区） 的边际收益 = 「跨 checkpoint 的摊销」减去「checkpoint 内摊销」**。**E7（离线索引 harness） 的对照组必须是「有 checkpoint、无消息缓冲」。**

**射程**：定的是那个对照组怎么设，不定 D11（索引节点要不要留消息缓冲区） 取哪一边——那由 D11（索引节点要不要留消息缓冲区） 自己定。已知边角一样：BetrFS 那组数字里这两块从没被拆开过（它同时有 4 MiB 节点的消息缓冲**和** 5 秒 checkpoint），**未在本项目验证**。

**依据**：

- 无实验：这是「对照组怎么设」的规定，不是实验结论；文献里没有一个对照组带 write buffer 前端，消息缓冲相对本工程现方向的边际收益，现有文献一个数字都没给，本项目也还没有可引的量。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/16-发布语义.md:335-357`（整段抄，未转述）**

```markdown
#### 已定项 17：fsync 报错之后，调用方能假设什么

**定案**：

- fsync 报错之后，这次发布的数据**可以出现也可以不出现**——调用方不许把「报错」读成「这次没成立」。
- **报错之后不许假设可以照旧复用同一份内存态重试。**（`close(2)` 的 NOTES 段明写「Retrying the close() after a failure return is the wrong thing to do」；本工程取同一条语义。）
- 要知道到底成没成立，调用方自己去读现行根：`crates/singlefs-core/src/recovery.rs` 的 `choose_system_configuration`、`choose_root` 对着仍打开的设备句柄就判得出，判据是「这次的 checkpoint_txg 是不是现行根」，不用卸载重挂。今天 `transaction.rs` 没有调它们的路径，缺的是一段胶水。
- 这一条不定 fsync 什么时候返回，那是已定项 7。

**射程**：

- **只在「转只读」与「重做又失败」那几格上生效。** `D23`（journal 的角色与格式） 已定项 14 失败表判「切换」的那几格，调用方根本收不到错误（实例切换会把在飞 checkpoint 重发掉），这一条对那几格无话可说。
- 管的是「报错之后调用方能假设什么」，不管失败本身怎么分流（那是 `D23`（journal 的角色与格式） 已定项 14），也不管持久顺序（已定项 7）。

**依据**：

- 无实验：这一条是对外语义的承诺，没有可量的量。撑它的是 `research/prompts/c381-r3-main-verification.md` 第一节 K1 那一格——H1（第六步系统配置槽轮换失败、前五步全成）上，「报错即没成立」这条语义**本身就是 `C381`（根已落盘之后发布失败，分配器仍退回） 的成因之一**：它让调用方以为没成立、拿旧状态接着发。
- `research/prompts/c381-r3-main-verification.md` 第三节（候选甲站住，两样收严的来处：K2 的 POSIX 线索、K4-e 的射程限定）。
- 用户定案 2026-09-21，原话在变更史。
- ⚠️ **依据里要写明**：定案的后两句（禁止性条款、射程限定）**被攻过零轮**（`.claude/rules/three-way-inference.md`：攻方腿自己提的收严算没被攻过），用户 2026-09-21 知情接受。

**欠**：`C381`（根已落盘之后发布失败，分配器仍退回）（改法落地之前这一格还开着）。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:1-11`（整段抄，未转述）**

```markdown
## D19 块指针的结构与宽度预算 —— 已定

D19（块指针的结构与宽度预算） 管块指针这一格：指针分几段、每段有哪些字段、各多宽、按什么次序排；位置条目与中央映射谁是解引用与释放判定的权威；中央映射条目的 key 与 value 怎么编码；映射 key 的哪几段跟着指针走，出生序号怎么发。不管的：单元头的字段表与单元类型码在 D18（块里携带什么信息）；节点大小与树内条目布局在 D8（核心索引结构）；落点坐标系与分配记录的 key 在 D3（空间分配） 已定项 3 / 7，提交内生块的 bump 次序在 D3（空间分配） 已定项 10；校验和覆盖多大、逻辑 extent 与单元的关系在 D4（校验和位置）；加密算法、nonce 怎么产出、AAD 期望值从哪来在 D9（加密）；根记录与系统配置的字段表在 D22（单元原子性怎么合成） 已定项 7 / 9；条带宽度的粒度在 D2（RAID 条带策略） 已定项 1。

<!-- doc-lint:registry name-col=2 -->

| 编号 | 简称 |
|---|---|
| N1 | 读不出先重读一次 |
| N3 | 对不上的留在已分配 |

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:32-45`（整段抄，未转述）**

```markdown
#### 已定项 1：位置条目带设备身份

**定案**：位置条目里带设备身份（dev id），每个物理位置各带一份；不引入「逻辑地址 + chunk 映射表」这层间接。

**射程**：管「带不带」，不管宽度（已定项 4）与次序（已定项 11）。D2（RAID 条带策略） 已定项 1 的「逐次可变」是这一格正推那一侧的前提，而带 dev 在两种粒度下都成立（它是能力更强的那一侧）⇒ 粒度定案只影响位置条目能不能被压缩表达，不影响带不带。宽指针成立与否与 D8（核心索引结构） 的节点大小耦合：同样 40 字节的指针头部，4 KiB 节点付 +98%、16 KiB 节点只付 +14%，D8（核心索引结构） 已定项 2 取 16 KiB 并钉成常量。E20（指针变宽的 CPU 缓存代价） 量的是 40 字节的头部，今天是 50（已定项 7 / 11），多出来的那几字节 E128（指针带出生 (树, txg) 之后的点查代价） 量到记账树点查慢 1.8%–4.5%，前提没翻。带 dev 最大的那笔代价（移除设备要全量扫描重写每一棵带指针的树）由 D1（数据可移动性 / 反向索引） 已定的反向索引降到 O(该设备的引用)，那条路本工程本来就要走。

**依据**：

- E20（指针变宽的 CPU 缓存代价）：指针变宽的代价只在工作集超出 L3 时出现，16 KiB 节点把它吸收掉——宽指针在本工程成立的直接前提，也是 D8（核心索引结构） 已定项 2 的依据侧。
- 现役实现（2026-08-28 现查，未在本项目验证）：带 dev 的 bcachefs 移除设备要全量扫描重写每一棵带指针的树（`bch2_dev_usrdata_drop()`），靠 `BTREE_ID_backpointers` 才降到 O(该设备的引用)，ZFS 把 dev 焊死在 blkptr 里、移除设备只能加一层永不消失的 `vdev_indirect_mapping`（只能 condense，不能删）；不带 dev 的 btrfs 要为映射表的自举在系统配置里挖一块定长不可扩的 `sys_chunk_array[BTRFS_SYSTEM_CHUNK_ARRAY_SIZE]`（2048 字节，`include/uapi/linux/btrfs_tree.h:502,720`），且 balance 照样重写全部引用。
- 三方论证（2026-08-28）：正推腿与反推腿结论相反，分辨它们的事实是 D2（RAID 条带策略） 的可变粒度；本地腿整轮作废，腿况与经过在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:46-59`（整段抄，未转述）**

```markdown
#### 已定项 2：密文校验和取 32 位

**定案**：位置条目里的密文校验和取 32 位，正好是完整 CRC32C 的宽度，不涉及截断；实现不许走「更宽的 CRC 截断到 32 位」这条路。

**射程**：结论是条件式的，引用时条件要一起引——它只在这一组输入下成立：10 TB 池、4 KiB 块、每周 scrub、10 年、损坏率 1e-9 每块读、漏检期望上限 1e-6。重开条件可判定：拿到现场损坏率且它超过 3.37e-9，这一格当场重开。候选算法一个都没被淘汰（偏离都在一个数量级以内）。威胁模型里介质损坏不是攻击者，这一格不判抗攻击性。

**依据**：

- E40（密文校验和位宽）：这组输入下最小位宽 31（与原推导那句「n > 30.3」取整一致），32 位的余量 3.37 倍——五个输入（池容量、损坏率、服役年限、scrub 周期、块大小）每一个各自错这么多倍才要 33 位；截断的 CRC 不是短一点的 CRC——CRC32C 全宽对单比特翻转 0 漏检是结构性质，截断之后变成概率检出。
- 旁证（现查，未在本项目验证）：btrfs 的默认校验和就是 32 位 CRC32C（`BTRFS_CSUM_TYPE_CRC32 = 0`），它那 256 位槽位是为可选的 SHA256 / BLAKE2 留的。
- 用户定案，原话与那天的推导在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:60-73`（整段抄，未转述）**

```markdown
#### 已定项 3：定宽，不加密时留位

**定案**：指针按最坏情形定宽——头部里的 MAC（128 位）与 nonce（96 位）恒定占位，不加密的卷把它们留成空位，不用类型标记省掉；这些位在所有卷上都占着，也不挪去装别的字段。

**射程**：可选的是 AAD 那道运算（加密关着就不算），不是格式（`.claude/rules/fs-design.md`「加密是可选的运算，不是可选的格式」）。按介质开放布局分支那个机制管不到这一格：incompat 位管的是读者认不认识这套布局，这一格管的是在同一套布局里一个指针内部怎么编码；先定宽、以后再加一套变长布局不是扩张，是多一套要永远维护的解析器加 checker 加崩溃点重放。不加密时白占 MAC 16 字节 + nonce 12 字节 = 28 字节 / 逻辑 extent，按 D4（校验和位置） 已定项 7 的口径（逻辑 extent 占盘恒 32768 字节）是 `28 / 32768 = 0.0854%`。变长那一侧省的是那 28 字节、不是省一个字段（算法类型字段 D9（加密） 已定项 12 第 4 项本来就要有），要付的是解析必须从头走：定宽指针可以按偏移直接定位第 k 个，变长不行；「恢复期扫一个部分损坏的节点时能不能跳过坏掉的那一条继续读」是推论，未实测。 加密关着时 MAC 16、nonce 12 恒 0，读者遇到非 0 判该指针所在的结构损坏，同 I-2.4（头校验和覆盖范围） 给单元头 29 字节的读法（实审 A3a 落地在两处读者——多层码 2 树内部条目、journal 新根段指针：`crates/singlefs-core/src/pointer.rs` 的 `PointerHeadFieldOutsideTheFirstVersion::MacOrNonceNotZero`）；extent 偏移那 2 字节第一版恒 0（第一版指针指整个单元），读者遇到非 0 判该指针所在的结构损坏，与加密开不开无关；checker 在走读跟随的每条指针上判，core 每个读盘上指针头部的读者都判，落点在 [checks-owed.md](../checks-owed.md) 已还清表 C581（extent 偏移非 0 没有判据） 那一行。

**依据**：

- 无实验：定宽换变长省下来的是空间，省掉的是「一个指针单独捡起来能不能直接按偏移解出来」，两边不是同一个量纲，没有可量的量。撑它的是 `.claude/rules/fs-design.md` 两条——「不为省空间牺牲自包含」，以及格式分支的判据（看它能不能把「用错了」变成「挂不上」，且判据里不许出现「这样更快 / 更省」）。
- 用户定案，原话与 2026-09-12 的重申在变更史。
- 判决 2026-10-01 采：extent 偏移第一版非 0 判该指针所在的结构损坏（`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 11 行）。

**欠**：C210（三个「第二套」属于哪一类扩展没人判过）。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:74-94`（整段抄，未转述）**

```markdown
#### 已定项 4：位置条目 14 字节

<!-- format-const: LOC_ENTRY = 14 stale=11 * 2|tree_entry=61 -->

**定案**：设备 ID 4 字节、物理偏移 6 字节（16 KiB 槽号）、密文校验和 4 字节 ⇒ 位置条目 14 字节。

| 段 | 宽 | 为什么是这个宽 |
|---|---|---|
| 设备 ID | **4** | 上界指不到任何已定条款（D12（目标介质） 不封顶介质套数、全仓没有 dev id 回收条款 ⇒ 窄宽度定的是池一生里出现过的设备身份总数），按 `.claude/rules/fs-design.md`「不为省空间牺牲自包含」取宽的一档；先例是已定项 3 |
| 物理偏移 | **6**，编码为 **16 KiB 槽号** | 索引节点按 16384 对齐、大小就是 16 KiB（D18（块里携带什么信息） 已定项 9）⇒ 按 32 KiB 单元号编码表达不了半数索引节点的落点，按字节偏移则让非 16 KiB 对齐的起点变得可表示。6 字节槽号寻址 4 EiB / 设备，同宽的字节偏移只有 256 TiB |
| 密文校验和 | **4** | 已定项 2 |

**射程**：物理偏移与 D3（空间分配） 已定项 7 是同一个坐标系（D3（空间分配） 已定项 3 依据第 2 条逐字要求同一个坐标系）⇒ 两处一起定，不许各定各的。树表单元指针与根记录合计这两个下游数随它改，而根记录的合计不在这里登记，唯一权威处是 D22（单元原子性怎么合成） 已定项 7 的字段表。相对旧的 11 字节，每个位置条目多 3 字节、每个指针多 6 字节。

**依据**：

- E97（记账与分配记录的条目编码）：判据 7 按可表示性判落点编码——按 16 KiB 槽号编码时全部合法起点都表达得了，按更粗的单元号有一半写不出来。
- 用户定案，原话与「这个数此前没走过决策流程却已经被消费掉」那次复核在变更史。

**欠**：C238（位置条目从 11 变 14 之后「53」这个派生数没人回扫）。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:95-129`（整段抄，未转述）**

```markdown
#### 已定项 5：位置条目降为提示，中央映射是唯一入口

**定案**：取混合——

| 件 | 定案 |
|---|---|
| 位置条目（已定项 4 的 14 字节） | **降为提示**。宽度与段划分不变，变的是它的地位：读到它可以直接去那个落点试，但它可能过期 |
| 中央映射 | **解引用与释放判定的唯一入口**。搬迁只改这里一条条目，「谁引用我」根本不用知道 |
| 态别 | 两者都不是权威态清单的第四项：映射可从单元自描述扫描重建，承重面仍是 D20（承重面：单元的原子性与自包含） 的单元 |

随定案一起生效的三条硬规则：

1. **释放一律经映射，不经提示**；经映射核到那条映射条目之后，释放之前还要按它位置项里带的单元校验和读盘核一次。**核出对不上时隔离那个槽，发布照成**：逻辑上照样释放（映射条目去掉），物理槽不还回空闲池，记进隔离并计数报出。
2. **单元头或提示必须带块 birth**——今天已满足：D18（块里携带什么信息） 已定项 7 的数据单元类身份段本来就含诞生代号 8 字节。
3. **提示过期的多跳率要有运行时观测点**（走错分支是性能悬崖，而悬崖不报错，`.claude/rules/fs-design.md` 硬要求 4）。

**挂载态怎么读映射**：第一版挂载态打开池时把中央映射整片读进挂载态（`crates/singlefs-core/src/mounted_read.rs` 的 `central_mapping_entries`），之后的解引用查这份内存里的条目、不再发映射读；中央映射长成多层时，挂载态打开池时把整棵映射树读进挂载态，解引用照旧只查内存、不为映射发设备读；写侧每次发布按映射树现在的条目数与这次发布最多新增的条目数做映射树容量准入，不靠别的树容量上的数字巧合。多层之后映射树没有条目数上限，结构上限只有头里层级那 1 字节；在规划那一步、任何落盘之前按这次真要写的条目算出形状，层级写不下就拒（`HeightBeyondTheLevelField`），不另立条目数上界（实十九取，主 agent 定）。「提示过期的一次解引用发 3 次设备读」这个读数就骑在这一条上，这个读数是根兼叶时量的，多层落地时由用例重量（里程碑「覆盖写、释放、回退与复用」收口表第 28 行）。分配记录树挂载时同样整棵读进挂载态；extent 树不整棵读，打开文件时按位置走下去（D8（核心索引结构） 已定项 14，用户 2026-09-24 定）。

**硬规则 1 的读盘核读不出、核出对不上时怎么办**：释放之前按位置项读盘核校验和，读盘本身失败或核出对不上，都先重读一次，两次都读不出或都对不上才按对不上处置（重读对不上那一半主 agent 2026-09-25 定：一次瞬时坏读不许让单元永久隔离；实二五已实现）。两次读结果不同类（先读不出、重读对不上，或反过来）按两次都没对上处置，报重读那一次的样子（实二五取，被攻过零轮）。任一份核出对不上，这个单元在每块盘上的分配记录都留在「已分配」、不改成已释放（另一块盘上那一份对得上也一起留，各盘的账保持对称）——落盘即跨重挂，准入里照已分配算、不另进式子。位置项指向一块不在池里的盘，或两条位置项指同一块盘，当映射条目损坏：在任何写之前拒绝，盘上不变。从盘上重建上一版（`crates/singlefs-core/src/recovery.rs` 的 `rebuild_version`）时读不出的数据单元照抄它的位置项、不读内容，挂载照常，读到那个文件时才报错。

**射程**：同时收口 D21（权威态与派生态的分界） 那条「共享之下多引用者住哪」的清单那一半与 D26（后台整理与放置回收） 已定项 5 的位置权威三选一，别处一律链到已定项 5；它原本还收口 C34（明文映射层的态别三处打架），而 C34（明文映射层的态别三处打架） 2026-09-07 随 D9（加密） 已定项 5 改取「只做 scrub」而无对象，那一份收口落空、另两处不受影响。映射条目的字段表不在这一格里，归已定项 6。三处已知边角没有编号：映射树的 key 布局要兑现 D8（核心索引结构） 已定项 7 那条骑手（去重窗口要被 `T_dirty` 界住）；提示的刷新政策只有「随重写刷新」一种被建模过；缓存命中这一维零测量，E109（位置权威三臂的运行时代价） 明说不答。自举豁免的三类见已定项 8。硬规则 1 的读盘核读不出、核出对不上时怎么办见定案那一段（C394（释放判定不核映射条目位置项里的单元校验和） 的三问）；「先重读一次」只重读一次，不退避、不多次重试。

**依据**：

- E109（位置权威三臂的运行时代价）：三条臂逐维量出来——搬一个单元的代价在指针权威下随引用树数线性、在中央映射与混合下与它无关；发布多付一笔与叶数无关的定额；读只多一成到四成。绝对值要连取样点一起引：它的主取样点上引用树与映射树恰好同高，那是一个退化点。
- E96（混合架构的一致性）：按提示解落点会误放，硬规则 1 由它逼出来；它的 `hdr_key` 臂引的是 D18（块里携带什么信息） 已定项 3 而不是已定项 7 的字段表，那条「要加字段」的格式代价不存在。
- 用户定案 2026-09-23：释放之前读盘核位置项里的单元校验和，核出对不上时隔离那个槽、发布照成（`research/prompts/m2-placement-falsepositive-r1-main-verification.md` 第三节判 C394（释放判定不核映射条目位置项里的单元校验和） 那一格：三条硬规则都不回答要不要核），原话在变更史。
- 用户定案 2026-09-24：挂载态整片读映射照今天的实现写成条款，多层映射由用户同日另一次定案（四棵树的分裂都在里程碑二做，含挂载态读多层映射）改成整棵读，实十九落地（C483（挂载态读映射与树节点回退都没有条款） ①，并行线二落地时定下、没有条款），原话在变更史。
- 用户定案 2026-09-24：C394（释放判定不核映射条目位置项里的单元校验和） 三问——读盘失败先重读一次再按对不上处置、重建时容下读不出的数据单元、隔离那一份的分配记录留在已分配（判决 `research/prompts/m2-newq-r1-main-verification.md` 第三节 N1（读不出先重读一次）–N3（对不上的留在已分配），原话在变更史）；无实验：候选的读数来自三方攻方腿在副本上的原型（`research/prompts/m2-newq-r1-opus-model/`），没有登记成实验。
- 用户定案 2026-09-24：多层映射挂载时整棵读进挂载态、写侧补映射树容量准入（原话在变更史；三方 `research/prompts/m2-treesplit-r1-main-verification.md` T3）。
- 用户定案 2026-09-25：只一份核出对不上时两块盘一起留、位置项指池外的盘照今天拒绝写成条款（实十六接续报告 `research/prompts/m2-writepath-implementer-report.md` 第六节 Q2、Q3），原话在变更史；无实验：两条都不改盘上格式，只定处置。
- 主 agent 定案 2026-09-25：`records/2026-09-24-里程碑二收尾调度.md` 第三节「瞬时坏读让单元永久隔离」那一行。

**欠**：C248（树表单元的自举豁免没写过）、C253（66 048 那笔定额两份 kb 说法相反）、C133（搬迁的转发信息没有载体）、C279（打包之后扫描重建同时看到旧单元与槽）；C417（位置提示的刷新政策只建模过一种）、C418（位置权威三臂的缓存命中维零测量）——依据那两条实验各自自陈答不了的格；C543（重建上一版只带实例表第 0 片）。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:130-159`（整段抄，未转述）**

```markdown
#### 已定项 6：中央映射 key 取按类复用写序键（收严后）

**定案**：key 定宽、逐段排布，定义与宽度逐字同 E134（映射 key 格式代价·槽基线与两条新臂） 的 `class_reuse_tagged` 臂——

| 类 | key（定宽，逐段） | 宽 |
|---|---|---|
| 码 1 数据单元 | 类标签 1 + 出生树 8 + 出生 txg 8 + 写序 10（实例代号 4 + 事务号 6） | 27 |
| 码 2 / 码 3 | 类标签 1 + 出生树 8 + 出生 txg 8 + 实例代号 4 + 出生序号 4 | 25 |

value 照已定项 4 / 已定项 5：位置条目 × w 副本（w = 2 时 28 字节）。补齐与条目总宽归已定项 10。

**它带着的四条收严**（三轮对抗里按「记一次输 / 只许收严 / 写明收严在哪」加上的，定案之后照样要守）：

- 类标签取被命名对象在 D18（块里携带什么信息） 已定项 11 登记表里的码，D27（小数据打包容器） 槽里的对象恒为码 1；打包与整容器重写时，槽里 key 的各段逐字节取打包前那一版单元头里的值。
- 出生树取类身份段里的出生树（码 3 偏移 43），出生 txg 取诞生代号（码 3 偏移 73；码 1、码 2 同取诞生代号），实例代号取写序的前 4 字节。
- D16（发布语义） 已定项 5 末段那句「一个事务最多写一个单元的用户数据」要按罩全部码 1 单元读（预分配、截断的尾单元、打洞两端、reflink 两端的拷贝都算），最好写进那一句。
- 出生序号只做 key 的一段，不拿来给固定点多版择新——拿来择新就落进 D18（块里携带什么信息） 已定项 1 否掉的「为分辨陈旧副本而设的序号」；D18（块里携带什么信息） 已定项 1 照写序那条先例欠一条射程注。

**射程**：管 key 与 value 各是哪几段、各多宽，不管补齐与条目总宽（已定项 10）、不管装哪几类（已定项 8）、不管出生序号怎么发（已定项 9）。它落 D15（格式冻结政策） 第 2 层。码 1 那一半的唯一性靠「码 1 的写序已经是唯一的版本判别子」这条推论，而它建在 D16（发布语义） 已定项 5 末段那条事务切分纪律上（配 D23（journal 的角色与格式） 已定项 7 一条记录一个事务号）；那一段 2026-09-13 随 C123（用户定案被改后无人复核） 的复核收口、切分纪律没改，推论今天成立，剩下的口径打架记在 C310（事务切分纪律与记录数口径打架）。「先」照录：这一格是按今天的代价与性能数取的，新证据推翻它时重开，不是论证出来的唯一解。

**依据**：

- E137（映射 key 形态的性能差距）：三条臂在每次 fsync、后台负载、随机读三类负载上的差距——按类复用写序键在每次 fsync 的多数格与随机读全格领先，在全域随机覆写、删文件、单文件快照删除上输；用户看了这张表取它。⚠️ 模型里物理落点就是写入顺序，这是它在搬迁与快照删除上最有利的情形，老化之后缩多少没量。
- E133（映射 key 各候选的格式代价）：两轮三方论证里出现过的每一种 key 落到格式上各付多少字节，七条候选的指针宽、条目宽与扇出由它给。
- E134（映射 key 格式代价·槽基线与两条新臂）：这一格取的那条臂就是它的 `class_reuse_tagged`，收严加的 1 字节类标签与 D27（小数据打包容器） 槽的两种基线由它按行报出。
- E136（映射 key 岔路的代价行）：交用户定之前四样缺的代价数——码 2 头加 4 字节在 D18（块里携带什么信息） 已定项 14 的三档真实头宽下一格扇出都不掉，映射树层数只在一段池规模区间里差一层。
- 三轮对抗的判决与三轮论证的经过在变更史；用户定案，原话在变更史。

**欠**：C288（映射 key 定案的连带改动没落到别的决策）、C289（码 2 的出生树取哪个字段没写）、C163（固定点各轮写序相同使平局破除落空）、C286（只读后 remount 取不取新实例代号）、C287（切换收养开放 checkpoint 的事务后再崩）、C310（事务切分纪律与记录数口径打架）。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:160-173`（整段抄，未转述）**

```markdown
#### 已定项 7：指针头部带出生 (树, txg)，各 8 字节

**定案**：指针头部在 MAC 16 + nonce 12 + 算法类型 1 + 压缩算法码 1 + 压后长度 2 + extent 偏移 2 之后再带出生树 8 与出生 txg 8，头部合计 **50**；指针照旧定宽（已定项 3）。压缩那 3 字节是压缩算法码 1 字节（登记表住 D9（加密），0 = 不压缩，第一版写 0）加压后长度 2 字节（第一版写 0，够表达 [0, 32768) 的单元内长度）。

**射程**：定的是哪 16 字节住指针头部，不定指针总宽（那是已定项 8 与已定项 11 合出来的 88 / 86）。压缩那 3 字节补的是 D9（加密） 已定项 13 第 2 条那条硬要求的格式载体——它要求压缩与加密的次序在格式上说得清，而在此之前仓里没有任何字段承载「这份密文是压过的、压前多长」。⚠️ 主 agent 推荐的是不留（压缩是可选运算，而「为加密留的位所有卷都留」那条原则只管加密），用户弹窗定案选了留，没有走三方论证，可推翻。从指针拿这两段的消费者有四个：D5（快照 / 空间记账机制） 的 O(1) 判定读出生 txg；deadlist 条目按版本身份记，出生树要从指针取；中央映射 key；AAD 形态要指针有这两个字段的位置，而值要经独立验收（D9（加密） 已定项 16 逐字不许从指针里读回来）。改新池新建文件的字节：是——每一条指针加 3 字节。

**依据**：

- E128（指针带出生 (树, txg) 之后的点查代价）：记账树内部条目加宽之后四轮都更慢，inode 树那一对四轮不稳定、不能引；它量的只是这 16 字节，是这一格代价的下界。
- 一轮三方论证，判决在 `research/prompts/d19-item7-r1-main-verification.md`：反推腿指出已定项 8 当时的末句把「住在指针里还是住在引用条目里」留了口子，两种读法都说得通。
- 用户定案（「16 字节住在指针里」），原话与候选乙、丙的出局经过在变更史。

**欠**：C295（指针里的 key 段算不算 AAD 的查找路径）、C296（journal 重放时凑不齐映射 key）、C297（inode 内部条目里出生树两份没有定处理）、C237（码 1 / 码 2 的指针里没有出生 (树, txg) 的位置）。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:174-197`（整段抄，未转述）**

```markdown
#### 已定项 8：全部指针进映射，引用侧按类携带映射 key 的各段

**定案**：中央映射装码 1、码 2、码 3 三类单元；指向一个单元的指针携带该类映射 key 除类标签外的各段，类标签由查找路径给。

| 被指的类 | 指针携带（已定项 6 的 key 各段） | 指针宽 |
|---|---|---|
| 码 1 | 出生树 8 + 出生 txg 8 + 写序 10 | 数据指针 **88** |
| 码 2 / 码 3 | 出生树 8 + 出生 txg 8 + 实例代号 4 + 出生序号 4 | 节点指针 **86** |

**不进映射的三类（自举豁免）**：映射树自己的节点、树表单元、实例表单元。它们的父指针（映射树的根、根记录里的树表指针与实例表指针）里的位置条目是权威，不是提示，已定项 5 的「唯一入口」与硬规则 1 对这三类不适用；三类都是池级单份、引用者只有一个。这三类的父指针与指向码 2 / 码 3 的节点指针同宽 86：位置条目 14 × 2 是权威，实例代号 4 + 出生序号 4 照填被指单元头里的值、不拿去查映射；解析器按被指单元的类定指针宽，不按对面是不是豁免类区分。豁免三类之外的单元（码 1 数据单元与码 2 树节点——extent 树根、inode 树根、inode 叶容器）位置提示读不出时一律经映射回退；豁免三类（映射树根、树表、根记录那条链）不回退。

**射程**：判的是映射装哪几类、指针带 key 的哪几段，不随指针宽度变（其中出生 (树, txg) 那 16 字节住指针头部由已定项 7 定）。只装码 1 那一支在两处正确性上输：码 2 / 码 3 的块同样进 deadlist / livelist，而 D5（快照 / 空间记账机制） 定 deadlist 条目按版本身份记、经映射释放，不进映射就只能按落点记，整理或掉盘重建把它挪走之后会按旧落点释放；搬一个被只读快照引用的码 2 / 码 3 要改写快照树的祖先，取新 birth 违反引用区间、保留旧 birth 违反 `birth` 的定义句。全部指针进映射在这两处都不输：deadlist / livelist 按映射 key 记，搬迁只改映射一条条目，被搬单元逐字节不变、birth 不变。代价：每次 fsync 与后台每单元的映射块都比只装码 1 多一到两成以上，随机读未命中也涨；这一列还少算两块（派生树的码 2 在映射里的删插、extent 节点自己提示过期走映射的读）。

**依据**：

- E137（映射 key 形态的性能差距）：两种配置（只装码 1 / 全部指针进映射）逐格量出代价差，这一格的代价行整段取自它。
- E134（映射 key 格式代价·槽基线与两条新臂）：两种配置下的指针宽、条目宽与扇出由它给，88 / 86 是它那两行加上已定项 7 的压缩 3 字节。
- E128（指针带出生 (树, txg) 之后的点查代价）：定案那天漏录、次日补上的那一笔点查代价，记在这一格的代价行里。
- 两轮三方论证，判决在 `research/prompts/d19-item8-r1-main-verification.md` 与 `research/prompts/d19-item8-r2-main-verification.md`（第一轮两条云端腿结论相反，用户要求重推，第二轮立场对调）。
- 用户定案，原话与两条补法为什么都要改一批已定条款在变更史。
- 2026-09-23 判决 `m2-lines234-code-r1-main-verification.md` 里判树节点要不要经映射回退那一格——攻方腿实测同一种损伤下数据单元多跳一次读回正确字节，而 extent 树根让 `open_pool_for_read` 整池挂不上。

**欠**：C280（中央映射装哪几类单元没有条款）、C293（全部指针进映射之后 E137 少算的两块）、C292（码 2 身份段零消费者的理由已不成立）、C248（树表单元的自举豁免没写过）。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:198-210`（整段抄，未转述）**

```markdown
#### 已定项 9：出生序号怎么发，按什么次序发

**定案**：出生序号从 0 起；同一棵树内每写出一个码 2 或码 3 单元加 1（两类共用一个计数）；作用域换到下一个 checkpoint 时清零；同一个 checkpoint 里同一个节点被固定点重写第二次换新号。发号次序与提交内生块从开放聚簇段 bump 的次序是同一个次序——按树 ID 升序，树内先叶后根、同层按 key 升序，中央映射树倒数第二，树表单元最末。

**射程**：次序本身的唯一登记位是 D3（空间分配） 已定项 10，这里不存第二份。出生序号是中央映射 key 的一段（已定项 6），key 必须在单元写出去之前就定得下来，所以发号绑在落点的分配次序上：实现里只有一个游标、checker 只核一份次序。它只做 key 的一段，不拿来给固定点多版择新（已定项 6 的第四条收严）。用户弹窗定案，没有走三方论证，可推翻；主 agent 推荐的也是这一档。改新池新建文件的字节：是——新池新建文件写出的七个提交内生块的落点与出生序号都随它定：t2 extent 根 50240（树 11，序号 0）、t3 inode 叶容器 50242–50243（树 12，序号 0）、t4 inode 根 50244（树 12，序号 1）、t5 分配记录树 50245（树 13，序号 0）、t6 记账树 50246（树 14，序号 0）、t7 中央映射树 50247（树 15，序号 0）、t8 树表单元 50248（无树归属，序号 0）。

**依据**：

- 无实验：次序与初值本身没有可量的量，发号与落点共用一条规则这件事的代价在 D3（空间分配） 已定项 10 那一侧；撑这一格的是「key 要在单元写出去之前定得下来」这条推论加已定项 6 收严论证里那两句（同一容器在两个 checkpoint 各拿到出生序号 0、多买到的只是运行期映射不撞）只在重写换号下成立。
- 用户定案，原话在变更史。

**欠**：C291（出生序号的分配规则只在实验源码里）。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:211-223`（整段抄，未转述）**

```markdown
#### 已定项 10：映射 key 一律 27，条目一律 55

**定案**：中央映射的 key **一律 27 字节**——码 1 的 (类标签 1, 出生树 8, 出生 txg 8, 写序 10) 天然就是 27；码 2 / 码 3 的 (类标签 1, 出生树 8, 出生 txg 8, 实例代号 4, 出生序号 4) 是 25 字节，**末尾补零到 27**。中央映射条目因此**一律 55 字节**（key 27 + value 28，value = 位置条目 14 × 2）。映射树节点头的 key 区间同样按 27 存。

**射程**：连带两处随它回到定宽——D5（快照 / 空间记账机制） 已定项 6 的稀疏旁表条目定宽 31（映射 key 27 + 计数 4），D6（快照实现模型） 已定项 2 的 livelist 条目定宽 36（头树 ID 8 + 映射 key 27 + 事件类型 1）。按类变长会把 D8（核心索引结构） 已定项 11 的两条通用规则一起破掉：「定宽条目连续排列」与「声明长度 = 条目数 × 条目宽」——码 2 的条目宽字段只有一个值，两宽就得给它一个哨兵，而哨兵一立，节点内的定宽二分就没了，扫描期解一个映射节点要从头顺序走。代价是每条码 2 / 码 3 条目多 2 字节，新池新建文件多 10 字节。用户弹窗定案，没有走三方论证，可推翻；主 agent 推荐的也是这一档。改新池新建文件的字节：是。

**依据**：

- E145（码 2 自描述头与映射树 key 宽的代价）：它量过的是一宽那条臂；「两宽 55 / 53、按 key 首字节自定界」是那一轮没有量过的第四种形态，拿它当定案等于引一个没有产物的数。
- 用户定案，原话在变更史。

**欠**：C321（码 2 头的条目宽是单值而映射树条目按类两宽）。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:224-251`（整段抄，未转述）**

```markdown
#### 已定项 11：指针的偏移表、位置条目的顺序、映射树的根住哪

**定案**：指针从偏移 0 起逐段排布——

| 段 | 偏移 | 宽 |
|---|---|---|
| MAC | 0 | 16 |
| nonce | 16 | 12 |
| 算法类型 | 28 | 1 |
| 压缩算法码 | 29 | 1 |
| 压后长度 | 30 | 2 |
| extent 距起点偏移 | 32 | 2 |
| 出生树 | 34 | 8 |
| 出生 txg | 42 | 8 |
| 位置条目 × 2，按设备身份升序 | 50 | 14 × 2 |
| key 尾段：码 1 写序 10；码 2 / 码 3 实例代号 4 + 出生序号 4 | 78 | 10 / 8 |

⇒ 指向码 1 的指针 **88**，指向码 2 / 码 3 的指针 **86**。位置条目按设备身份升序排列，立成 I-2.5（位置条目按设备身份升序）。**中央映射树的根住根记录**：根记录带一条 86 字节的中央映射树根指针，不进树表；根记录合计 **457** 字节。树表第 1 版 **7** 条。

**射程**：定的只是次序——段的集合与宽度是已定项 3 / 4 / 7 / 8 定过的。压缩那 3 字节紧跟算法类型，与它同属「这份密文是怎么产出来的」那一组，排在 extent 偏移之前是为了让加密与压缩四个字段连成一段。根记录合计的唯一权威处是 D22（单元原子性怎么合成） 已定项 7 的字段表，树表第 1 版那一条随 D5（快照 / 空间记账机制） 已定项 11 加 deadlist 而来。用户弹窗定案，没有走三方论证，可推翻。改新池新建文件的字节：是——每一条指针的段偏移与宽度，以及根记录的合计。

**依据**：

- 无实验：偏移表本身没有第二种选法，段的集合与宽度都已经定过，这一格只排次序，没有可量的量。位置条目按设备身份升序是为了让两条位置条目有一个确定的次序，好立成不变量。
- 用户定案，原话在变更史。

**欠**：C304（实例表链指针记录装不下 83 宽的指针）。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:252-264`（整段抄，未转述）**

```markdown
#### 已定项 12：码 2 / 码 3 的映射条目 day-1 写出，固定点一个 checkpoint 只写一次

**定案**：维持 day-1 写出（新池新建文件 6 条）。固定点写死：每个码 2 节点与码 3 容器，一个实例在一个 checkpoint 里只写出一次——发布末尾一次性 COW；第一版禁边插边 COW（D3（空间分配） 已定项 5 允许的那条实现第一版不取）。

**射程**：⚠️ 以后放开边插边 COW，D8（核心索引结构） 已定项 13 照旧要守：同一次发布里同一个 key 只按最终态落叶一次。会红的检查按单元计，不按固定点轮数计。这一格定的是写不写与写几次，不定 key 形态（已定项 6）与条目宽（已定项 10）。定案时仓里还没有任何释放与重用实验，另一条路留成「等有释放的实验之后再看」，而那句触发条件没有写进这一格。

**依据**：

- E81（提交的固定点）：「已脏节点本次发布只 COW 一次」这条读法的出处，也是固定点在这条读法下收敛的依据；照它罩码 2 与码 3，同一 checkpoint 里两版同 key 那一输才不再出现。
- 用户定案，原话在变更史。

**欠**：C352（day-1 写映射条目的定案，重开条件没人叫醒）、C163（固定点各轮写序相同使平局破除落空）。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:265-280`（整段抄，未转述）**

```markdown
#### 已定项 13：块指针分两段

**定案**：**块指针分两段：一份「头部」跟着逻辑 extent，N 个「位置条目」跟着物理落点。**
逻辑属性（MAC、nonce、算法类型、extent 距起点偏移）在头部，每个逻辑 extent 一份；
物理属性（落点、以及供无密钥侧检损坏的密文校验和）在位置条目，每个物理位置一份。

**射程**：分段把「指针总宽度」从一个约束降级为一个预算，D4（校验和位置） 那一格因此改写成分两段的形态。它只定哪一类属性住哪一段，不定各段有哪些字段（已定项 4 / 7）与次序（已定项 11）；头部后来又加了出生树与出生 txg（已定项 7），加的仍是逻辑属性。「挪出指针」不等于多一次 I/O：D4（校验和位置） 要求的是父节点存子节点校验和，它约束的是住在哪个节点，不是住在哪个结构体。

**依据**：

- D2（RAID 条带策略） 已定「多副本是同一份密文写 N 遍」⇒ 所有副本必须共用同一个 nonce；逐指针各存一份等于把同一个数抄 N 遍，且某副本那份被写错时同一份密文会得到两个互不相容的解密结果。可判定形式是 I-6.7（副本共用 nonce）。
- 现役实现给了同一个形状（2026-08-28 现查逐字核实，未在本项目验证）：bcachefs 的 `struct bch_extent_ptr` 恰好 64 位（`offset:44, dev:8, gen:8` 加四个标志位），校验和 / nonce / 压缩信息全部挪到同一个 bkey 里的独立变长条目（`bch_extent_crc32/64/128`），一条 crc 条目对其后所有 ptr 条目连续生效；nonce 本身不存，`fs/bcachefs/checksum.h` 的 `extent_nonce()` 由 `bversion`（96 位，本来就在 bkey 头部）加 per-crc 的一个小偏移（`crc64.nonce:10` / `crc128.nonce:13`）现算。
- 无实验：分两段是格式结构的取舍，撑它的是一条已定规则（副本共用 nonce）与现役实现的形状，没有可量的量。

**欠**：无。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:281-293`（整段抄，未转述）**

```markdown
#### 已定项 14：位置条目逐个列出，不用几何公式从单一起点推算

**定案**：N 个位置条目逐个列出，各带设备身份；不从单一起点加一个尺寸推算出其余各列的位置。

**射程**：管位置条目怎么表达，不管它带哪几段（已定项 4）。前置口径是 D2（RAID 条带策略） 已定项 1 把「宽度可变」的粒度定成**逐次可变**——格式侧物理位置逐个列出、各带设备身份、不引 chunk 映射表，正是这条推导要的那个前提。它与 D2（RAID 条带策略） 已定项 1 同型：买到的是「不指错盘」，不是「失败域独立」。

**依据**：

- ZFS 能用「一个 DVA + asize」推出 RAID-Z 全部列位置（`BP_GET_ASIZE` 把 3 个 DVA 的 asize 相加），前提是该 top-level vdev 的子设备数**在池创建时固定**（现查，未在本项目验证）；D2（RAID 条带策略） 已定项 1 否定了这个前提。
- 无实验：能不能推算由条带宽度的粒度直接决定，是一条推论，没有可量的量。

**欠**：无。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:294-306`（整段抄，未转述）**

```markdown
#### 已定项 15：extent 距起点偏移的值域与第一版取值

**定案**：extent 距起点偏移的值域由**最大 extent 尺寸**封顶，不由文件尺寸封顶；本工程的值域是 [0, 32768)，**15 位够**。第一版该字段恒 0——切分是预留特性、第一版关闭；位仍按已定项 3 的「定宽留位」留着，理由是政策不是用量。

**射程**：说的是值域，宽度取 2 字节在已定项 11 的偏移表里。D9（加密） 已定项 12 第 2 项按 32–64 位估这个字段，那是按文件尺寸估的，估高了。本工程值域更窄的依据是 D4（校验和位置） 已定项 7：extent 占盘恒 = 单元 = 32768。

**依据**：

- 现役实测（2026-08-28 现查 `fs/bcachefs/extents_format.h`，未在本项目验证）：`crc32.offset:7` / `crc64.offset:9` / `crc128.offset:13`，配套 `CRC128_SIZE_MAX (1U << 13)` ⇒ 这个字段的值域由最大 extent 尺寸封顶。
- 无实验：值域是两条已定条款相除得出的算术，没有可量的量；切分第一版关不关由 D4（校验和位置） 已定项 7 定。

**欠**：无。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:307-317`（整段抄，未转述）**

```markdown
### 与其他决策的连锁

| 决策 | 连锁点 |
|---|---|
| D4（校验和位置） | 已定项 13 把「指针总宽度」从一个约束降级为一个预算，D4（校验和位置） 那一格因此改写成分两段的形态；逻辑 extent 占盘恒 32768 由 D4（校验和位置） 已定项 7 给，已定项 3 与已定项 15 都踩在它上面 |
| D2（RAID 条带策略） | 已定项 1 与已定项 14 的前置是条带宽度可变的口径，由 D2（RAID 条带策略） 已定项 1 定为逐次可变 |
| D9（加密） | nonce 载体由已定项 13 定了一半（属头部、每逻辑 extent 一份）；算法类型与压缩那几段的登记表住 D9（加密），AAD 的期望值不许从指针里读回来（D9（加密） 已定项 16） |
| D12（目标介质） | D12（目标介质） 已定项 4 的挂载比对表里，设备身份错配要拒绝挂载，依据是已定项 1 的位置条目带 dev。⚠️ 那条判据说的是「用到它的**地址算术**是否携带设备身份」，btrfs 的地址算术里有设备身份、只是住在 chunk 树；判据成立，倒下的是「携带 ⇒ 必须在指针里」这个跳步 |
| D15（格式冻结政策） | 指针字段布局是 D15（格式冻结政策） 已定项 7 的第 2 层；已定项 6 的中央映射条目字段表也落在第 2 层（D15（格式冻结政策） 已定项 1 的依据逐字点名） |
| D8（核心索引结构） | 指针宽进内部条目与树表条目，D8（核心索引结构） 已定项 6 / 8 的几个宽度随已定项 7 / 8 改；宽指针成立与否与 D8（核心索引结构） 已定项 2 的 16 KiB 节点耦合 |

```

**出处 `.claude/kb/decisions/09-加密.md:1-4`（整段抄，未转述）**

```markdown
## D9 加密 —— 已定

D9（加密） 管本工程的加密：整卷 AEAD 的方向与明文豁免范围、格式 day-1 要为加密留的位、MAC / nonce / 算法 / AAD 的形态、三条与选型无关的硬要求、无密钥的一侧能做什么、系统配置与根的认证和回滚防护、加密进不进第一个可运行版本。加密是可选的运算、不是可选的格式（`.claude/rules/fs-design.md`「加密是可选的运算，不是可选的格式」）：加密关着时 MAC 与 AAD 不算，为加密留的位所有卷照留、全 0、不挪去装别的字段。不管的：各字段的宽度与偏移——指针归 D19（块指针的结构与宽度预算） 已定项 11，根记录归 D22（单元原子性怎么合成） 已定项 7，系统配置归 D22（单元原子性怎么合成） 已定项 9，journal 记录头归 D23（journal 的角色与格式） 已定项 4，单元头里的 nonce / MAC / 算法类型预留位归 D18（块里携带什么信息） 已定项 14 / 已定项 16；单元类型标签的值域归 D18（块里携带什么信息） 已定项 11 的登记表。调查见 [prior-art.md](../prior-art.md) 第六节。`crates/` 里今天没有任何加密代码（`grep -rniE "chacha|aes|aead|encrypt|decrypt|poly1305|cipher" crates` 零命中）：写路径把这些位写 0，读路径（`PointerHead::read_from`、`SystemConfiguration::parse_slot`）与池级 checker 跳过它们不判。

```

**出处 `.claude/kb/decisions/09-加密.md:27-39`（整段抄，未转述）**

```markdown
#### 已定项 1：指针里额外存一份密文的非密钥校验和

**定案**：指针里有两个完整性字段：MAC（要密钥，认证）+ 密文校验和（不要密钥，查损坏）。所有读路径都用密文校验和，不是只有 scrub 用。

**射程**：无密钥只做检测，不做判定谁是好副本——攻击者能造出校验和自洽的块骗过无密钥的一侧（已定项 6 命题 1 在无密钥 scrub 上不成立，是同一件事）。代价是指针变大。密文校验和住哪、多宽归 D19（块指针的结构与宽度预算） 已定项 4（位置条目里的 4 字节）；它的可判定形式是 I-6.8（密文校验和匹配），不需要密钥。

**依据**：

- 无实验：要不要这个字段是能力取舍，没有可量的代价——无密钥的一侧只做 scrub（已定项 5），检测损坏、报告坏块全靠这个不要密钥的字段，没有它未解锁时就完全瞎。
- ZFS 同形态：MAC 占校验和空间的一半，另一半在有没有加载密钥时都用来查盘上损坏（`include/sys/spa.h:242-246`，原文在变更史）。

**欠**：无（I-6.8（密文校验和匹配） 的 checker 状态在 [invariants.md](../invariants.md)，加密不进第一版）。

```

**出处 `.claude/kb/decisions/09-加密.md:40-58`（整段抄，未转述）**

```markdown
#### 已定项 2：MAC 取满 128 位，不截断

**定案**：MAC 取满 128 位（16 字节），不截断。

**射程**：MAC 的职责只剩认证——无密钥的损坏检测由已定项 1 的密文校验和独立承担，截断省下的 8 字节买不到任何检测能力。管所有指针里的 MAC 与自证单元自己身上的 MAC（已定项 12 第 1 项）。已定项 13 硬要求 3（截断 MAC 时读重试上限要写死）因此不承重；「MAC 长度取决于读重试上限」这条判据不成立，C3（读重试无硬上限） 已销账，理由是前提口径错了、不是已拦住。引截断的危险性时不许拿 Ferguson 2005 的短 tag 惨案外推到 Poly1305：那个攻击的病根是 GHASH 在 GF(2¹²⁸) 上的线性结构，Poly1305 在素域 2¹³⁰−5 上、掩码是模加不是 XOR，那条 bit-forcing 路径没有对应物（这一条本身是推理，查不到权威来源明写它免疫）。

**依据**：

- 无实验：取值由标准与内核接口的硬约束定，两边没有可量的代价要比。
- 三方论证一致（经过在变更史）。
- SP 800-38D §5.2.1.2 逐字「t may be any one of the following five values: 128, 120, 112, 104, or 96」，80 不在其中；Linux `chachapoly_setauthsize` 要求 authsize 恰为 16 字节，`crypto_gcm_check_authsize` 拒 10 字节（[prior-art.md](../prior-art.md) 6.7）。
- RFC 8439 §4「Tag truncation MUST NOT be done.」：未经分析的保守禁止、不被普遍执行（BoringSSL、OpenSSL 放行到 1 字节），只当中等强度佐证；来龙去脉在变更史。
- 零个现役实现钉死过跨请求的伪造尝试上限（dm-integrity、dm-verity、bcachefs 现查，全内核 `MaxInvalids` 0 命中）；`MaxInvalids` 出自 SP 800-38B，38D 附录 C 那一列计的是全部解密调用。
- 唯一支持 80 位的现役数据点是 bcachefs 的默认值：那是 `bch_extent_crc64` 位域挤出来的余量，靠绕开内核 AEAD 接口手动截断做到，而它自己的元数据仍恒用 128 位。
- 截断 Poly1305 的危险性独立于 RFC 那条禁令：Wegman-Carter 类 MAC 的伪造界是 `fℓ/2^τ`；一次成功伪造在 Poly1305 上同样暴露 r（DJB *Cryptography in NaCl* §9 逐字）；安全截断要两级哈希或额外子密钥，Poly1305 一样都没有。
- OpenZFS 面对同一个冲突保满 128 位 MAC（`include/sys/zio.h:121` `ZIO_DATA_MAC_LEN 16`），宁可付「加密块最多 2 副本」。

**欠**：无。

```

**出处 `.claude/kb/decisions/09-加密.md:59-71`（整段抄，未转述）**

```markdown
#### 已定项 3：运行时按 CPU 特性选，格式里不钉死

**定案**：盘上记算法类型，运行时按有没有 AES-NI 挑默认值。ChaCha20-Poly1305 与 AES-256-GCM，两种算法都必须实现。算法类型字段的登记表：0 未加密、1 ChaCha20-Poly1305、2 AES-256-GCM；新算法只追加不重用码，读者遇到未登记的码显式失败，不按别的算法校验（`.claude/rules/fs-design.md`「每个写的算法类型字段」那一格；D22（单元原子性怎么合成） 已定项 17 把登记位指到这里）。加密路径必须能并行。

**射程**：决定的是默认值怎么选，格式侧没有新增。并行那条：无 AES-NI 时 AES-256-GCM 单核只有 244.74 MiB/s，低于任何现代 NVMe 的顺序带宽（本机 nvme0n1 3.6 T，其顺序带宽未单独实测——这一步按介质类别推）。单核加密不是 flush 的关键路径，关键路径是 flush 的树插入 I/O（E17（write buffer 合并上限） 末尾的比较集更正）。加密路径的并行与 D13（验证路线） 已定项 2（先串行提交）怎么叠加没有评估过。`crates/` 的读路径今天跳过算法类型那个字节（`PointerHead::read_from`、`JournalRecord::parse`、`RootRecord::parse_slot`），「未登记的码显式失败」没有代码在判。

**依据**：

- E6（加密算法选型）：两档 CPU 特性下 AES 与 ChaCha 的快慢反转、八格全扫都在数倍——挑一个写进格式在另一类机器上必然选错；屏蔽 AES-NI 时 AES 单核吞吐低于 NVMe 顺序带宽，所以加密路径必须能并行。
- 登记表：用户定案，原话在变更史。

**欠**：无 C 编号——并行与先串行提交的叠加、读侧「未登记的码显式失败」的判定，两笔都没立账。

```

**出处 `.claude/kb/decisions/09-加密.md:72-87`（整段抄，未转述）**

```markdown
#### 已定项 4：nonce 每 extent 存，不现算

**定案**：指针头部为每个逻辑 extent 存一份 96 位 nonce。不采用 bcachefs 那种由 `bversion` 现算的形态。

**射程**：

- 买到：① 解密不依赖任何**别处**的字段仍然正确 ⇒ 单元自包含（D20（承重面：单元的原子性与自包含））在加密开启时也成立——这一句靠 D18（块里携带什么信息） 已定项 14 才按字面成立：完整 96 位 nonce 另有一份住单元明文头，指针里那份保留；② I-6.1（nonce 不重用） 变成**对着镜像可判定**的性质——扫一遍全部指针头部即可。付出：**12 字节 / 逻辑 extent**；不加密时是留位（D19（块指针的结构与宽度预算） 已定项 3）。
- 它没有解掉 extent 切分那一格：已定项 12 第 2 项仍然必需——**若**启用切分，切分后的 N 个子 extent **共享同一份密文、同一个 nonce、同一个 MAC**，存下来的那份 nonce 是「原始 extent 的」，子 extent 靠偏移找回它。**存 nonce 不等于每个子 extent 各存一个。**第一版没有这一格：切分是预留特性、第一版关闭，`extent_off` 恒 0、每个单元恒一份 nonce 与 MAC（D4（校验和位置） 已定项 7）。

**依据**：

- 无实验：存还是算是自包含政策，没有可量的代价——nonce 是写出那一刻就该被钉死的东西；现算把「这个密文该配哪个 nonce」变成一条推导，推导的每个输入（bversion、per-crc 偏移、extent 切分历史）都是后来可能被改动的量，存下来则一次写入、此后只读。并列的第二条依据是 `.claude/rules/fs-design.md` 的**不为省空间牺牲自包含**。
- 用户定案，原话在变更史。

**欠**：C184（重建路径拿不到完整 nonce）：nonce 取值来源在扫描重建路径上可达的门禁比对。

```

**出处 `.claude/kb/decisions/09-加密.md:88-106`（整段抄，未转述）**

```markdown
#### 已定项 5：只做 scrub，安全优先

**定案**：**加密开启时，无密钥的一侧只读只检测——不搬、不疏散、不回收。**「一个块的物理地址的权威记录点」**留在加密结构里**（指针里的位置条目，D19（块指针的结构与宽度预算） 已定项 4），不设明文映射层、不设明文反向索引。无密钥能做到的是检测损坏、报告坏块；搬不动任何东西，碎片和坏盘疏散仍要密钥。

**射程**：

- **知情接受的代价**：D3（空间分配） 的「常驻、自动、用户无需知道的后台整理」这条硬承诺**在加密开启时不成立**，锁盘期整理与疏散全部停摆，近满卷上的换盘与缩容做不成。
- 另两个候选不取：「索引内部节点留明文」（能搬、能疏散，但内部节点带分隔 key，dirent 树的分隔 key 就是文件名——泄漏面比 ZFS 大得多，ZFS 的间接块是纯指针数组，没有 key），「反向索引 + bucket 代号留明文」（能搬，泄漏物理布局与引用关系）。撤回前取过的第三个候选与它的经过在变更史。
- 无密钥 scrub 要读哪些结构，全仓没有清单（C303（D2 边界 ③ 的依据随 D9 已定项 7 撤回而失效））。

**依据**：

- 无实验：不是取舍改了偏好，是**「能搬」这一侧被证明不可达**，没有可量的代价要比。四条已定条款不能同时为真：搬一个单元就是发出一条新条带（D2（RAID 条带策略） 已定项 6 逐字「`w ≥ 2` 是硬下界（零冗余的条带不许发出）」），新条带每格要一条成员记录（D2（RAID 条带策略） 已定项 12），而成员表逐字「**必须住在单元的载荷里**」，载荷是密文 ⇒ **无密钥侧写不了成员表** ⇒ 搬不成；`w ≥ 2` 是硬下界 ⇒ 对**任何**池成立。明文映射层是为「能搬」付的代价（D3（空间分配） 已定项 16），目的没了，代价就不必付。
- 瓶颈的准确位置不是 MAC 的位置，是**「一个块的物理地址的权威记录点在不在密文里」**：把 MAC 挪到块自己的明文头（bcachefs 对 btree 节点就是这么做的，`extents_format.h` 一行注释直说 `/* Btree pointers don't carry around checksums: */`）只松一格——元数据块可以原样搬（MAC 覆盖块自己的字节、nonce 不由物理位置派生），但父指针里的**地址字段**仍要改，而父节点的载荷是密文。
- 三方：已定项 9 的六轮三方论证把「能搬」那一侧的形态逐个打穿（`research/prompts/d9-item9-r1-*` 到 `d9-item9-r6-*`，`records/2026-09-06-D9项9三轮对抗.md`）；用户定案，原话在变更史。
- bcachefs「无密钥后台整理瘫痪」是推理、不是引文：一手材料里没有散文明说，推论建在五条可核事实上（检索范围与五条事实在变更史）。

**欠**：C303（D2 边界 ③ 的依据随 D9 已定项 7 撤回而失效）：列出无密钥 scrub 要读的全部结构；C23（无密钥 scrub 分不出活槽与陈旧槽）：扫到一个自洽的单元时判不出它活不活；C32（无密钥的一侧写不写得出字节没有闸）：「只读」今天没有任何闸；C33（scrub 检出之后做什么没有条款）：报告落在哪、解锁后谁消费；C134（scrub 水位自己没有落点与保护）；C136（无密钥能验哪几条不变量没人标过）；C166（明文侧取消之后格式预留要不要跟着收）；C167（明文映射层的引用全仓回扫没做）。

```

**出处 `.claude/kb/decisions/09-加密.md:107-175`（整段抄，未转述）**

````markdown
#### 已定项 6：AEAD 的关联数据绑逻辑身份，物理位置一律不进

**定案**：**AAD = `(单元类型标签, 树 ID, 对象 ID, 对象出生代, 锚点偏移)`，定长定序的规范编码。物理落点、设备 ID、以及任何明文侧字段，一律不进。**AAD 是加密开启时才算的那道运算，加密关着不算（`.claude/rules/fs-design.md`「加密是可选的运算，不是可选的格式」）。每个字段的期望值从哪来归已定项 16。

| 字段 | 为什么必须在里面 | 谁提供期望值 |
|---|---|---|
| **单元类型标签**（值域是 D18（块里携带什么信息） 已定项 11 的登记表——数据 extent = 码 1、btree 节点与记账树节点 = 码 2、打包记录单元 = 码 3、根记录 = 码 16；它是 AAD 规范编码的第一个字节） | 不同单元类的字段集合不同。没有域分隔时两类单元的 AAD 可序列化成同一串字节 ⇒ **类型混淆变成一次合法的 MAC 通过** | 读者的查找路径 |
| **树 ID** | D8（核心索引结构） 是多 keyspace，对象号在不同 keyspace 里各自编号、**必然碰撞** | 查找路径 |
| **对象 ID（inode）** | 同一棵树里文件 A 的 offset 0 与文件 B 的 offset 0 除对象号外全同。指针挂错对象 ⇒ 读 A 得 B | 查找路径 |
| **对象出生代（inode generation）** | 按五元组 / AAD 的 day-1 契约保留（D18（块里携带什么信息） 已定项 3 / 已定项 11）。它不是深度防御，是唯一性本身：D8（核心索引结构） 已定项 6 定的是「已发布的 (inode 号, 对象出生代) 不复用」，号在回退之后按设计会被重发，在 AAD / nonce 两层把同号的新旧对象分开的正是这一段；它同时仍是 I-6.1（nonce 不重用） 失效时（崩溃窗口 / nonce 水位回滚）的第二道闸 | 密文侧 |
| **锚点偏移** = 该 extent 首次写出时的对象内起始偏移，读者现算 `key.offset − ptr.extent_off` | 同一文件内两个 extent 互换，对数据库文件 / VM 镜像就是一次真实破坏 | 查找路径 + 密文侧 `extent_off` |

一定不能进 AAD 的，以及各自撞的是哪一条：

| 字段 | 撞哪条 | 机制 |
|---|---|---|
| **物理落点 / 设备 ID** | **A7（含义待查） + D19（块指针的结构与宽度预算）** | D2（RAID 条带策略） 已定「N 个副本是同一份密文写 N 遍、共用一个 nonce」，D19（块指针的结构与宽度预算） 已定「MAC 每逻辑 extent 一份、落点每物理位置一份」⇒ 落点进 AAD ⇒ N 个副本 N 个 AAD ⇒ 要 N 个 MAC ⇒ 当场推翻 D19（块指针的结构与宽度预算） 的分段结构 |
| **extent 距原始起点偏移** | **已定项 12 第 2 项** | **AAD 的字段组成是 day-1 永久契约**：它必须在加密开机之前定死，此后改一次就是一次格式断裂 ⇒ 判据只能按「这个字段将来可不可能逐指针不同」定，不能按「它今天是什么值」定。而**若**启用切分，N 个子 extent 共享同一份密文、nonce、MAC 而该字段逐指针不同 ⇒ N 个 AAD 对 1 个 MAC ⇒ **N−1 个子 extent 全部验不过**，失配成为正常操作的必然结果 ⇒ 排除。⚠️ **不许用「第一版恒 0 所以放进去无害」重新论证**（反推腿点名的陷阱）：恒 0 是第一版的运行时事实，不是格式契约；按那条理由放回 AAD，切分一启用就当场破 |
| **`locality_id`** | **D8（核心索引结构） 已定项 3 逐字**：「它是提示，不是正确性的一部分——可以是错的、可以过期」 | 让声明为「可以是错的」的字段承重；D18（块里携带什么信息） 已定项 3 那条给块头加 `locality_id` 的提议用户定案不收，这一行维持 |
| **任何明文侧字段**（明文头 flags / 长度 / nonce 代号、位置条目里的密文校验和）| 期望值来源判据（已定项 16） | 期望值由被读对象自己提供，攻击者可同时改两边，AEAD 对此零保护 |
| **读者当前所处的快照 ID** | COW 共享 | 一个块从 N 个快照可达，绑读者侧快照 ⇒ 该块只在一个快照里验得过（这条排除本身成立，快照这一维的缺口见射程「AAD 缺快照维」） |
| **引用计数 / 记账量 / deadlist 归属 / 当前 txg** | D5（快照 / 空间记账机制） | 全是集合属性，块不被重写就会变 ⇒ 每变一次要重新加密一次 |
| **父节点物理地址** | D18（块里携带什么信息） 已定项 23 | 时序上不可能：COW 是子先写父后写，子块下笔时那个值还不存在 |

格式必须 day-1 提供的三样（少一样，checker 就判不了 I-6.3（MAC 与密文匹配） 与 I-6.9（密文过而 MAC 败判身份错配））：
① **单元类型标签在权威侧显式**，不许靠「它在哪棵树里」推；
② **反向索引条目的身份口径 == AAD 口径 == 权威树 key 口径，同一份定义**（三份口径就是三份定义）；
③ **btree 节点的逻辑身份要有定义**——AAD 的字段清单只覆盖数据 extent；D18（块里携带什么信息） 已定项 1 定了不做谱系重写序号，索引节点的 AAD 字段集要与码 2 类身份段（树 ID + 层级 + key 区间）对齐，欠在 C107（索引节点的 AAD 身份无落点）。

**射程**：

- fsid 不在 AAD 里，它是密钥派生的输入（已定项 8）：跨卷绑定发生在密钥层，另一个卷的密文在本卷的派生密钥下**根本解不开**，比 AAD 逐单元失配更早、更彻底；**不是字段，就不受 A6（AAD 期望值不来自明文侧） 约束**。
- 不必要但零成本的两项：算法类型、密钥代号。可以放，但必须记成「无成本的冗余」：它们住在密文侧、攻击者改不了，用错必然失配。算法类型够不着最危险的那一格：字段标为「未加密」时读者根本不跑 MAC，而 AAD 是 MAC 之内的东西；那一格的防线是 I-6.4（无未加密块） 加系统配置的加密声明。单元长度不必进 AAD：AEAD 构造已隐式认证它（`crypto/chacha20poly1305.c:184-185` 收尾喂 `tail.assoclen` / `tail.cryptlen`）。
- **AAD 缺快照维**：这五项对快照维是瞎的，而且是静默的——快照 key 错配时读出另一个可写头的数据而 MAC 全绿，而「快照 key 改写」正是链 B 点名的三类实现 bug 之一。排除「读者当前所处的快照 ID」的理由本身成立；倒下的是「排除了读者侧快照 ⇒ 快照这一维整个不进 AAD」这个跳步。出路只剩绑出生快照——把快照维折进现算的 nonce 是已定项 4 的另一个取值，而已定项 4 已定「每 extent 存，不现算」。处置是「定案不改，缺口记入」，字段表欠一次重开（C63（已定的 AAD 对快照维是瞎的））。每头一棵树下的可行形态：出生 (树, txg) 进 AAD、指针带声明、读者先拿**克隆祖先表**验收（声明的树在祖先链上且出生 txg ≤ 对应克隆点）再进 MAC；指针那两个字段的位置由 D19（块指针的结构与宽度预算） 已定项 7 给了（出生树 8 + 出生 txg 8），克隆祖先表要住已认证结构、读者可达（C110（跨头共享与加密后元数据类的块头一致性读法））；祖先验收区间之内的版本互换没测。**引用这五项时必须连这条一起引。**
- AAD 真正买到的是哪一环——不是「MAC 会失配」：

```
逻辑身份 q  ──链 B──  指针（头部：nonce / MAC）  ──链 A──  盘上的密文 C
```

| 链 | 由什么保证 | 主要威胁是谁 |
|---|---|---|
| **链 A：指针 ↔ 密文块** | **nonce**（在密文侧指针头里，I-6.1（nonce 不重用） 保证全盘唯一；tag 是 nonce 的函数）⇒ **不需要 AAD** | 攻击者 |
| **链 B：逻辑身份 ↔ 指针** | **只有 AAD**。nonce 只说「这块配这个指针」，不说「这个指针该挂在哪个 key 下」 | **本工程自己的实现 bug**——btree split/merge、write buffer 的 seq 去重、快照 key 改写 |

「MAC 会失配」只回答了链 A；AAD 的价值几乎全在链 B，而链 B 的主要威胁是自家代码（E17（write buffer 合并上限） 实测漏掉 `seq` 就是这一类）。

- 三条命题的适用范围：

| 命题 | 判决 | 收窄到什么范围 |
|---|---|---|
| 1 骗不过 checker | **对有密钥的 checker 成立** | **对无密钥 scrub 完全不成立**：密文校验和在明文侧，攻击者搬块时连校验和一起改成自洽即可。这是已定项 1 逐字接受过的范围 |
| 2 不能提权 | **在两条被单独关掉之后才成立** | ①「就近修复」：密文校验和过而 MAC 败时若当成损坏去走副本修复，攻击被转成数据销毁——由 I-6.9（密文过而 MAC 败判身份错配） 关掉；② **明文系统配置里的 nonce 水位可回滚**（A2（系统配置是明文））⇒ 写者重用 nonce ⇒ ChaCha20 下两段明文直接 XOR，**AAD 对它零作用**——由已定项 8 子问题 2 关掉 |
| 3 读不出他人旧数据 | **在单卷、非整根回滚、非块级克隆的范围内成立** | 挡住重排的主力是**密文侧的 nonce，不是 AAD**；AAD 的净增只有 I-6.1（nonce 不重用） 失效时的降级保护与抓链 B 的实现 bug。跨卷那一格已移交密钥派生层（已定项 8）|

**两格 AAD 补不上**：**整根回滚**（被回滚到的旧根每个 (nonce, AAD, 密文) 三元组都是真的——任何「这是上一版」的判定都需要 AEAD 之外的见证者）；**块级克隆**（克隆卷与原卷 fsid 相同、密钥相同、nonce 空间从同一点分叉 ⇒ 分叉后两卷可能对不同内容发同一个 nonce，**这是一次 nonce 重用事件，AAD 里放什么都修不好**，要修只能在 mkfs / 克隆时换密钥域）。「防不住整卷回滚」不是本工程独有的短板，但必须像 fscrypt、dm-verity 一样写在明面上。

- 对应的不变量都已写进 [invariants.md](../invariants.md)，checker 状态全是未实现：I-6.3（MAC 与密文匹配） 的关联数据每个字段由 checker 从权威树的 key 现算、不许从指针或明文侧读回；I-6.9（密文过而 MAC 败判身份错配） 不许归类为损坏、不许触发副本修复或投毒；I-6.2（明文头字段白名单） 的白名单是「允许出现在明文头」，不是「可以拿来判定」；I-6.1（nonce 不重用） 是抗 bug 检查，不是抗攻击检查。

**依据**：

- E31（AAD 缺快照维）：五项 AAD 对快照 key 错配静默放行，绑读者侧快照打坏共享读，绑出生快照与折进现算 nonce 两条判据全过——排除读者侧快照成立，缺口与出路由它给出。
- E90（树 ID 进 AAD 与跨头共享）：每头一棵树下读者按查找路径取树维会把合法继承读一并拒掉，出生 (树, txg) 进 AAD 加克隆祖先验收两条判据全过——绑出生快照的可行形态。
- 三方：三条腿一致判三条命题在提案之下成立，收窄条件由两条云端腿给出；正推腿抓到 extent 距原始起点偏移不能进 AAD（材料 `_d9q6-background.md`，腿况与主 agent 自核的四条外部事实在变更史）。
- ext4 的 inode 校验种子是 (fsid, 对象 ID, 对象出生代) 三件套，但三项进入的层次不同（[prior-art.md](../prior-art.md) 6.5）：重合点是「哪三项」，按层次读它支持已定项 8 的形状；折进种子的形态只能验证一个猜测、不能反解身份，不补 D20（承重面：单元的原子性与自包含） 的承重面洞（数据块自描述靠 D18（块里携带什么信息） 已定项 3 的显式字段）。
- fscrypt 明写不保证离线篡改之后的机密性与认证（[prior-art.md](../prior-art.md) 6.1）。

**欠**：C63（已定的 AAD 对快照维是瞎的）：AAD 字段表重开一次（绑出生快照）与 checker 侧的错配判定；C107（索引节点的 AAD 身份无落点）；C110（跨头共享与加密后元数据类的块头一致性读法）：克隆祖先表的落点。

````

**出处 `.claude/kb/decisions/09-加密.md:176-188`（整段抄，未转述）**

```markdown
#### 已定项 7：已撤回·此编号不再使用

**定案**：已撤回，此编号不再使用。原题：已定项 5 那句「无密钥侧的 GC 能搬块或回收块」与 D21（权威态与派生态的分界） 冲突（记账是权威态且加密 ⇒ 无密钥写不了记账），无密钥侧的「回收」到底改哪个结构。已定项 5 改取只做 scrub 之后无密钥侧不搬，让渡区 A 存在的唯一目的是给无密钥侧一块能分配的地方**用来搬**，A **无对象**；「`|A|` 该多大」「签发判据」两问随之无对象。连带作废：C135（无密钥要读的结构不在明文豁免清单里）、C137（搬运的源与目的在盘上分不开）、C150（让渡区的成员身份在盘上没有落点）、C151（无密钥搬运必然要写一张加密的权威表）——四条全是「无密钥侧要搬」这个前提的枝叶。

**射程**：撤回前的论证与定案（让渡区 A∪E、命题 5、那次重开的四条）只留在变更史，是将来重开无密钥搬运时的起点，不是现行设计；撤回的理由若将来失效，被撤回的结论也不自动回来（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「撤回的理由失效，不等于被撤回的结论自动回来」）。

**依据**：

- 无实验：撤回是已定项 5 改取只做 scrub 的连带，没有可量的量。
- 用户定案，原话在变更史；撤回前那一轮三方的材料 `_d9q7-background.md`。

**欠**：无——原来挂在这一项下的五笔 2026-09-20 按无密钥 scrub 这一侧的形态改写，前置改挂已定项 5，改写经过在 [checks-owed.md](../checks-owed.md) 历史版本节 2026-09-20 那一条。

```

**出处 `.claude/kb/decisions/09-加密.md:189-213`（整段抄，未转述）**

```markdown
#### 已定项 8：fsid 进 KDF，系统配置因 nonce 水位而必须认证，整卷回滚宣布不防

**定案**：

| 子问题 | 定案 |
|---|---|
| 1 系统配置被认证吗 | **要认证，但理由不是 fsid。** fsid 移进 KDF 后它与 A6（AAD 期望值不来自明文侧） 的矛盾自动消失；逼出认证需求的是 nonce 水位 |
| 2 nonce 水位住哪 | **不许住在裸明文系统配置里**，必须落在一个被主密钥派生 MAC 覆盖的字段上——无密钥侧不许写它 |
| 3 整卷回滚 | **宣布为已知不防的范围**，与 fscrypt 同口径。防它需要 AEAD 之外的见证者，本工程不引入 |

**射程**：

- 子问题 1 的解法不是给系统配置加认证去救 fsid，是把 fsid 挪出 AAD：另一个卷的密文搬进本卷 ⇒ 派生密钥不同 ⇒ **解不开**；攻击者改明文 fsid ⇒ 全卷解不开，是拒绝服务而非机密性突破，且**立刻可见**。KDF 只在加密开启时存在，第一版没有这条跨卷防线（C74（无加密期的跨卷错向写没防线））。
- 子问题 2：I-6.5（nonce 水位单调） 的水位若住在裸明文系统配置里，把它回滚就是**密钥被绕过**（写者重用 nonce，ChaCha20 下两段明文直接 XOR，AAD 里放什么都修不好）。
- 子问题 3：D22（单元原子性怎么合成） 已写下「任何『这是上一版』的判定都需要外部见证者，这是定义性的，不是工程限制」；这里只把它对系统配置的形态写明：**MAC 给完整性，不给新鲜性**——一个被重放的、真实的旧系统配置每一位都是合法的，验签必过。
- 系统配置的 MAC 具体覆盖哪些字段、算法与密钥怎么定，属格式细节，落点是 D22（单元原子性怎么合成） 已定项 9（系统配置的字段表）：表里已有系统配置自身 MAC 16、nonce 水位 12、KDF 标识 4、加密类型 1、MAC 长度声明 1、主密钥槽 80（内联）；覆盖范围条款里还没写。

**依据**：

- 无实验：三个子问题都是威胁模型与已定条款的推导，没有可量的代价。
- 现役先例：ext4 把 `s_uuid` 折进 `s_csum_seed`（挂载时算一次），逐对象校验只喂 `inum` 与 `gen`——「卷身份进派生层、对象身份进逐单元层」是现役做法（[prior-art.md](../prior-art.md) 6.5）；dm-integrity 的 `recalc_sector` 住明文系统配置，回滚即绕过，上游带密钥时默认禁掉那条路径（`drivers/md/dm-integrity.c:395`，[prior-art.md](../prior-art.md) 6.6）——本工程取同一形状。
- 三方：三条腿给了三个不同的答案，本地腿独有「fsid 该进 KDF 而不是 AAD」；分歧不靠投票，靠查 ext4 把 uuid 放在哪一层解（材料 `_d9q8-background.md`，腿况与四轮作废的本地腿在变更史）。

**欠**：C74（无加密期的跨卷错向写没防线）。

```

**出处 `.claude/kb/decisions/09-加密.md:214-226`（整段抄，未转述）**

```markdown
#### 已定项 9：已撤回·此编号不再使用

**定案**：已撤回，此编号不再使用。原题：明文映射层带哪些字段，以及它们自身的完整性怎么保（不参与 MAC）。已定项 5 改取只做 scrub：物理地址的权威记录点**留在加密结构里**，不设明文映射层、不设明文反向索引 ⇒ **无对象**。连带作废：C34（明文映射层的态别三处打架）、C129（锁盘期的明文映射层没有任何一方认证过）、C138（明文侧泄漏面超出已定项声明的范围）；I-6.10（已停用·明文映射层不存在了） 同样无对象。改新池新建文件的字节：否。

**射程**：撤回前六轮三方论证的记录（两条候选各塌在哪、明文层的键取物理、候选丁与候选戊、让渡区 A 的三处互斥、泄漏面推断链）只留在变更史与 `records/2026-09-06-D9项9三轮对抗.md`，不是现行设计。撤回前用户给过条件定案「如果物理键的性价比和效率高的话就定物理键，反之定逻辑键」，E108（明文层的代价交叉点） 把那个条件量成了数；那一轮的数留档，不再有消费者。E57（明文侧字段集的权威落点） 的「按字段拆」仍然成立，只是物理地址那一行的权威从明文映射层变成加密结构内的位置条目（D19（块指针的结构与宽度预算） 已定项 4）。C34（明文映射层的态别三处打架） 曾被三处当阻塞点名——D19（块指针的结构与宽度预算）、D21（权威态与派生态的分界） 的「引用者清单」那一半、D26（后台整理与放置回收） 已定项 5 的位置权威三选一——它们各自还剩什么挡着要复核。

**依据**：

- 无实验：撤回是已定项 5 改取只做 scrub 的连带，没有可量的量。
- 用户定案，原话在变更史。

**欠**：C177（C34 作废之后被解锁的三处没复核）。

```

**出处 `.claude/kb/decisions/09-加密.md:227-238`（整段抄，未转述）**

```markdown
#### 已定项 10：加密不进第一个可运行版本，只预留格式字段

**定案**：加密不进第一个可运行版本，只预留格式字段。实现等事务层与 checker 跑通之后再接（`.claude/rules/fs-design.md`：从事务开始，不从功能开始）。

**射程**：预留的是已定项 12 的四样，加密关闭时全 0、所有卷照留、不挪作他用（D19（块指针的结构与宽度预算） 已定项 3）。`crates/` 里今天没有加密代码：写路径把这些位写 0（`PointerHead::write_to`、`RootRecord` 与 `JournalRecord` 的写出、`SystemConfiguration::to_slot`、单元头的 29 字节预留）；core 读路径今天判单元头 29 字节全 0（I-2.4（头校验和覆盖范围））、系统配置加密类型恒 0、两处读者（多层码 2 树内部条目、journal 新根段指针）的指针头部 MAC / nonce 恒 0，非 0 一律判损坏（实审 A3a）；这两处读者之外还没换上的指针读者、系统配置其余结构常量、journal 记录自己的算法类型 / nonce / MAC 预留，读路径仍跳过。池级 checker 判三种码单元头 29 字节全 0与格式版本认得、及走读跟随的指针头部 MAC / nonce 全 0，均归 I-2.4（头校验和覆盖范围）；系统配置格式版本、加密类型越出这一版读者收的范围，归新立的 I-7.13（系统配置池级字段在读者收的范围里）。

**依据**：

- 无实验：先后次序是 `.claude/rules/fs-design.md`「从事务开始，不从功能开始」的直接应用，没有可量的量。

**欠**：C166（明文侧取消之后格式预留要不要跟着收）：为已取消的明文映射层 / 明文反向索引预留的位收还是留。

```

**出处 `.claude/kb/decisions/09-加密.md:239-253`（整段抄，未转述）**

```markdown
#### 已定项 11：方向是整卷 AEAD，加密边界与校验和边界重合

**定案**：**整卷 AEAD，加密边界与校验和边界重合：D4（校验和位置） 那个内联进指针的字段，在加密开启时就是 MAC。除系统配置外全部加密，元数据留最小明文头。**这句话没有例外项：journal、记账树、分配记录、条带成员表**一律在密文侧**，不必逐个再问。不做 per-directory / per-subvolume 加密；不为「无 key 也能后台维护」把元数据挪到明文。

**射程**：明文头里许出现哪些字段按单元类分两张白名单（I-6.2（明文头字段白名单）；D18（块里携带什么信息） 已定项 13 把码 1 的身份段与声明长度收进密文）；系统配置不加密但要认证（已定项 8）。加密开启时剩下的明文泄漏面没有一张可对照的表（C173（明文头的泄漏面在明文侧取消后没有辩护））。

**依据**：

- 无实验：加密边界放在哪是威胁模型与格式的取舍，没有可量的代价。
- AEAD 的位置已经由 D4（校验和位置） 腾出来了：块层加密（LUKS）做不了认证，是因为它没地方放 MAC 和 nonce 而不破坏块对齐；fscrypt 不认证，给的理由也是密文膨胀。
- 不做 per-directory / per-subvolume 加密：bcachefs 拒绝它的理由在本工程更重——fsck 需要文件大小 ⇒ 大小得明文；而本工程的 checker 要靠全盘遍历验记账（I-3.1（已分配统计对得上）），有一部分子卷锁着就等于那部分记账验不了。
- 不为无 key 后台维护把元数据挪到明文：ZFS 选了那条路，代价是间接块全明文（池结构、文件大小、文件洞、快照结构都泄漏），认证退化成 SHA512 聚合而不是 MAC 树，并且要在 objset 层维护 portable / local 两个根 MAC——[prior-art.md](../prior-art.md) 6.4 记的那串 send/recv 损坏 bug 就长在这套复杂度上。这条同时压住已定项 5 的第二个候选（索引内部节点留明文）：那是同一条路，在本工程还更糟，因为内部节点的分隔 key 带语义。

**欠**：C173（明文头的泄漏面在明文侧取消后没有辩护）：加密开启时的泄漏清单写成可对照的表。

```

**出处 `.claude/kb/decisions/09-加密.md:254-272`（整段抄，未转述）**

```markdown
#### 已定项 12：格式必须 day-1 预留的四样

**定案**：

| # | 字段 | 不留会怎样 |
|---|---|---|
| 1 | 指针里的 MAC + nonce 空间；**自证单元自己身上的 MAC + nonce + 算法类型** | 事后加等于改指针宽度，全格式重来。**自证单元（根记录、journal 记录头、系统配置槽）没有父指针**，它们那份 MAC / nonce / 算法类型只能住在自己身上：根记录末尾 算法类型 1 + nonce 12 + MAC 16（D22（单元原子性怎么合成） 已定项 7），journal 记录头的 MAC 16 排在 fsid 之后、nonce 12 与算法类型 1 在十个字段里（D23（journal 的角色与格式） 已定项 4），系统配置的 MAC 16 / nonce 水位 12 / 加密类型 1 在字段表里（D22（单元原子性怎么合成） 已定项 9）。⚠️ **这三样都不进 I-6（加密不变量）**：不留位的话，开加密那天只剩两条路——整条走明文豁免（根记录泄漏 `checkpoint_txg`、树表指针、映射根指针、树 ID 水位；journal 记录头泄漏 `checkpoint_txg`、新根段两条指针与点名项位置），或者一次布局变更。 |
| 2 | extent 的「距原始 extent 起点偏移」 | **若**启用切分，被切分的 extent 重写时无法既不重用 nonce 又不重算（[prior-art.md](../prior-art.md) 6.3）⇒ 位要 day-1 留。⚠️ **切分是预留特性、第一版关闭，该字段第一版恒 0**（D4（校验和位置） 已定项 7）——留位的理由是 D19（块指针的结构与宽度预算） 已定项 3 定宽、不加密时也留位这条政策，不是「第一版会用到」。宽度：值域由**最大 extent 尺寸**封顶、不由文件尺寸封顶（extent 恒 = 单元 32768 ⇒ 15 位够），D19（块指针的结构与宽度预算） 已定项 11 的偏移表给它 **2 字节**（偏移 32） |
| 3 | 系统配置的 KDF 标识 + 主密钥槽 + 加密类型 | 换不了 KDF，也没法在既有文件系统上开加密。落点 D22（单元原子性怎么合成） 已定项 9：KDF 标识 4 字节、主密钥槽内联进系统配置槽 80 字节、加密类型 1 字节，加密关闭时全 0；主密钥槽的内部构成随开加密时定 |
| 4 | 每个数据/元数据写的算法类型字段 | 换不了算法；bcachefs 把加密做成一种校验和类型，新数据用新算法、老数据按各自字段解 |

**射程**：这四样在多布局下共用不共用，权威记录在 D12（目标介质） 已定项 2：第 1、2、3 项**必须全布局共用**，第 4 项**可以各布局各异**。不在 D9（加密） 记第二份。加密关闭时这些位所有卷照留、全 0、不挪去装别的字段（`.claude/rules/fs-design.md`「加密是可选的运算，不是可选的格式」）；journal 记录头的算法类型那 1 字节属第 1 项（自证单元自己身上那一份）。

**依据**：

- 无实验：要留哪几样是「事后加不进去」的格式判断，没有可量的代价；第 1 项的射程扩到自证单元是用户定案，原话在变更史。

**欠**：C360（字节表 journal 那行的算法类型指到错的预留项）。

```

**出处 `.claude/kb/decisions/09-加密.md:273-289`（整段抄，未转述）**

```markdown
#### 已定项 13：三条与最终选型无关的硬要求

**定案**：

1. **nonce 不许由物理位置派生**，否则 balance / copygc 搬一个块就要重新加密（要改 MAC、要密钥），多副本还会因此字节不同，D2（RAID 条带策略） 的 parity 直接算不成。**nonce 跟逻辑身份走并存在指针里。**
2. **顺序固定为 压缩 → 加密 → MAC。** 反过来不可能（密文压不动）。由此密文长度泄漏明文可压缩性（Kelsey, FSE 2002），写进已知泄漏清单，不装作没有。格式载体：指针头部在算法类型（偏移 28）之后带 **压缩算法码 1 字节（偏移 29）+ 压后长度 2 字节（偏移 30）**（D19（块指针的结构与宽度预算） 已定项 7 / 已定项 11），压缩算法码的登记表在已定项 14。
3. **若采用截断 MAC，读重试次数上限必须是写死的小常数，并且有检查。** bcachefs 的 80 位 MAC 安全论证整个押在「MAC 对不上就判设备坏、很快停用」上——**那是一条实现行为，不是密码学性质**。（已定项 2 取满 128 位，此条因此不承重。）

**射程**：三条与最终选型无关。「已知泄漏清单」全仓没有这张表（`grep -rn 已知泄漏 .claude/kb` 只命中 D9（加密）），它的落点在 C173（明文头的泄漏面在明文侧取消后没有辩护）。压缩那组字段第一版写 0（已定项 14）。

**依据**：

- 无实验：三条都是从搬运、密码学次序与已定项 2 推出的约束，没有可量的量。
- 压缩那组字段：用户定案选了留（主 agent 推荐的是不留，压缩是可选运算、`.claude/rules/fs-design.md` 要求格式上吝啬分支，而「为加密留的位所有卷都留」那条原则只管加密、不管压缩），原话在变更史；没有走三方论证，可推翻。

**欠**：C173（明文头的泄漏面在明文侧取消后没有辩护）：已知泄漏清单那张表。

```

**出处 `.claude/kb/decisions/09-加密.md:290-307`（整段抄，未转述）**

```markdown
#### 已定项 14：压缩算法码的登记表

**定案**：

| 码 | 算法 | 第一版 |
|---|---|---|
| 0 | 不压缩 | **写 0**，压后长度同时写 0 |

新算法只追加、不重用码；读者遇到不认识的码一律判该指针无法解引用（与算法类型那个字段同一条政策，D22（单元原子性怎么合成） 已定项 17）。**压后长度**取 2 字节，够表达 `[0, 32768)`（D4（校验和位置） 已定项 7 定 extent 占盘恒 = 单元 = 32768）；码 0 时它恒 0，非 0 判该指针损坏。

**射程**：唯一登记位在这里（D19（块指针的结构与宽度预算） 已定项 7 的压缩那 3 字节指到这里）。它管码与压后长度的取值，不管压缩在写路径上什么时候做（顺序归已定项 13 硬要求 2）。`crates/` 写 0（`PointerHead::write_to`），读时跳过这 3 字节（`PointerHead::read_from`）：「不认识的码判无法解引用」「码 0 而压后长度非 0 判损坏」两条今天没有代码在判。

**依据**：

- 无实验：登记表是约定，没有可量的量；用户定案，原话在变更史。

**欠**：无 C 编号——读侧两条判定没有代码，也没立账。

```

**出处 `.claude/kb/decisions/09-加密.md:308-327`（整段抄，未转述）**

```markdown
#### 已定项 15：前提编号（A 系列）的登记表

**定案**：A 系列前提编号在本仓没有定义处，只有引用（全仓 grep 无定义行）；简称按引用处的原话反推，反推不出来的一条显式标为待查，不猜。这张表是它们唯一的登记位。

<!-- doc-lint:registry name-col=2 -->

| 前提 | 简称 | 内容（来源：D9（加密）引用处的原话） |
|---|---|---|
| A2 | 系统配置是明文 | 引用处原话：系统配置是明文 |
| A6 | AAD 期望值不来自明文侧 | AAD 每个字段的期望值一律不许来自明文侧 |
| A7 | 含义待查 | 只知道「物理落点 / 设备 ID 不能进 AAD」撞的是它，原文未找到 |

**射程**：登记的是编号与简称，不是规则：A6（AAD 期望值不来自明文侧） 的现行规则在已定项 16；A2（系统配置是明文） 说的是系统配置不加密，它照样要认证（已定项 8）。

**依据**：

- 无实验：登记表是命名约定，没有可量的量。

**欠**：无（A7（含义待查） 的原文没找到，按「待查」登记）。

```

**出处 `.claude/kb/decisions/09-加密.md:328-345`（整段抄，未转述）**

```markdown
#### 已定项 16：AAD 每个字段的期望值只许来自查找路径或已认证的密文侧

**定案**：

> **AAD 每个字段的期望值只许来自 ① 读者的查找路径，或 ② 已被上一级 MAC 认证的密文侧字段。
> 一律不许来自明文侧，也不许从指针里读回来。**

机制：从明文侧取期望值 ⇒ 攻击者控制 AAD 的输入，AAD 从防线变成自由变量；从指针里读回来 ⇒ checker 验的是「指针自洽」，**逻辑身份 ↔ 指针**那一环完全没查。

**射程**：管已定项 6 的 AAD 每一段；fsid 不是 AAD 字段（已定项 8），不受它约束。它的可判定形式是 I-6.3（MAC 与密文匹配）：关联数据的每个字段由 checker 从权威树的 key 现算，不许从指针或明文侧读回。两处边角没有条款：指针里的 key 段（D19（块指针的结构与宽度预算） 已定项 7 之后指针带出生 (树, txg)）算「查找路径」还是算「从指针里读回来」（C295（指针里的 key 段算不算 AAD 的查找路径））；跨文件 reflink 之后对象 ID 与锚点偏移没有期望值来源（C275（跨文件 reflink 下 AAD 两段没有期望值来源））。

**依据**：

- E90（树 ID 进 AAD 与跨头共享）：树维从指针读回来的那条臂三次换挂全部静默、连伪造的出生 txg 都看不见——期望值来源判据（不许从指针读回来）的可执行证明。
- 三方：两条腿从不同方向各自撞到它，且都判「这条不成立则三条命题全部崩塌」（已定项 6 那一轮，材料 `_d9q6-background.md`）。

**欠**：C295（指针里的 key 段算不算 AAD 的查找路径）；C275（跨文件 reflink 下 AAD 两段没有期望值来源）。

```

**出处 `.claude/kb/decisions/09-加密.md:346-367`（整段抄，未转述）**

```markdown
#### 已定项 17：与其他决策的连锁

**定案**：

| 决策 | 连锁点 |
|---|---|
| D1（数据可移动性 / 反向索引） | 数据可移动是硬需求 → nonce 不许绑物理位置（已定项 13 硬要求 1）。**bucket generation 只管失效不管搬运** |
| D2（RAID 条带策略） | parity 对**密文**算；多副本必须字节相同，即同一份密文写 N 遍，不是加密 N 次 |
| D3（空间分配） | 无密钥能验不能搬：scrub 可做，搬运要读写索引，见已定项 5 |
| D4（校验和位置） | 加密开启时该字段就是 MAC，两者是同一个字段的两种取值 |
| D5（快照 / 空间记账机制） | **birth txg 不能当 nonce**——同一个事务里的块共享它，nonce 会重复 |
| D8（核心索引结构） | extent key 要留 version number 与偏移字段；btree 节点明文头的字段集合要一起定 |
| D10（随机写 / 碎片化的真答案） | 任何绕过 COW 的快路径**不许绕过加密**；原地覆盖写会直接造成 nonce 重用 |

**射程**：只列 D9（加密） 压在别的决策上的约束；那一侧怎么兑现归各自的决策。D8（核心索引结构） 那一行的 version number 今天在 D8（核心索引结构） 的 extent key 里没有（它的已定项 3 是 `(locality_id, inode, offset)`），而已定项 4 定了 nonce 每 extent 存、不从 key 现算，这一行要不要留没有人判过。

**依据**：

- 无实验：每一行都从 D9（加密） 的已定项推出（nonce 跟逻辑身份走、MAC 与校验和同一字段、只做 scrub），没有可量的量。

**欠**：无。

```

**出处 `.claude/kb/decisions/16-发布语义.md:6-26`（整段抄，未转述）**

```markdown

| # | 分项 | 定案 |
|---|---|---|
| 1 | **defer 窗口取多大** | 按盘回退下界的收严形态、按状态数读：候选集保留最近 4 个可退到的不同状态（去重、以根环装得下为界），可再分配 ⟺ 已释放 ∧ 释放代 ≤ max(F_生效, 窗口里最旧有效根)；F 另记进系统配置 **状态：已定。** |
| 2 | **checkpoint 的触发判据** | 时间与脏数据量两者取先到：`触发 ⟺ (now − 上次 checkpoint 的时刻 ≥ T_time) ∨ (脏字节数 ≥ T_dirty)` **状态：已定。** |
| 3 | **journal 在 D15（格式冻结政策） 的冻结分层里记第几层** | 不进四层，立成四层之外的第三个独立冻结组件；新记录格式走记录类型 + 系统配置 log-incompat 位 **状态：已定。** |
| 4 | **记录核对器那第二份 replay 实现** | 一次发布整体施加或整体不施加；第二份实现停在句法层（前缀判定 + 择根 + 续做），在合谋测试里是第四个判决器 **状态：已定。** |
| 5 | **`T_time` 与 `T_dirty` 取多少，丢失窗口的口径** | `T_time` = 5 s、越界只警告；`T_dirty` = 2 GiB、夹取成 min(字段, 环长 ÷ F)；崩溃最多丢 T_time + N_switch × (T_retry + T_redo) **状态：已定。** |
| 6 | **发布计数器** | 每次发布 checkpoint_txg 加一，fsync 触发的也是发布；根槽轮转键、记账的代、重放水位是同一个计数 **状态：已定。** |
| 7 | **发布的持久顺序** | 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）→ 系统配置槽 → 屏障；重放施加前必须逐项验证点名单元 **状态：已定。** |
| 8 | **新实例要不要先让本实例的根落到两块盘上** | 取戊：根覆盖两块盘之前不让 fsync 返回，连推空发布至多 R = 3 次 **状态：已定。** |
| 9 | **一次空发布写不写单元** | 记账树存在时就写：空发布也是发布，按 D5（快照 / 空间记账机制） 已定项 2 重写记账行与连带的四样 **状态：已定。** |
| 10 | **发布粒度** | 攒够一批脏节点（或到时间）再发布一次根，不是每个事务发一次；发布频率只由 D16（发布语义） 定 **状态：已定。** |
| 11 | **能不能绕过 journal 直接发根** | 不能：所有结构、所有操作先进同一个 journal、共用同一个 checkpoint 节奏，fsync 只是提前触发一次发布 **状态：已定。** |
| 12 | **中间状态的合法性语义** | D8（核心索引结构）「中间状态必须每一个都是合法的文件系统状态」改写成「根 + journal 任意前缀，replay 之后合法」 **状态：已定。** |
| 13 | **这份 journal replay 归谁** | 归记录核对器，不归 O2（独立解析器 + checker）；它是第二次实现 replay，是 D16（发布语义） 最贵的一笔 **状态：已定。** |
| 14 | **快照发布点的位置** | 与 checkpoint 边界对齐，钉在这个 checkpoint 全部数据变更之后；同窗两个快照用 `(txg, 快照 id)` 打破并列 **状态：已定。** |
| 15 | **checkpoint C 中产生的释放什么时候进可分配集合** | C 被发布之前不得进入；两条理由，第二条按 journal 位置说，比 checkpoint 边界更严 **状态：已定。** |
| 16 | **checkpoint 与消息缓冲的摊销窗口分界** | 两者买同一样东西，差别只在窗口住内存还是住盘；E7（离线索引 harness） 的对照组钉成「有 checkpoint、无消息缓冲」 **状态：已定。** |
| 17 | **fsync 报错之后调用方能假设什么** | 报错之后这次发布可能已成也可能未成，不许假设可复用旧内存态重试；成没成由调用方自己读现行根判定 **状态：已定。** |

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:13-31`（整段抄，未转述）**

```markdown

| # | 分项 | 定案 |
|---|---|---|
| 1 | **位置条目带不带设备身份** | 带：每个物理位置各带一份 dev id，不引一层 chunk 映射表 **状态：已定。** |
| 2 | **密文校验和的位宽** | 32 位；取完整 CRC32C，不许由更宽的 CRC 截断而来 **状态：已定。** |
| 3 | **定宽还是变长** | 定宽，不加密时留位：MAC 与 nonce 恒定占位，留成空位、不挪作他用 **状态：已定。** |
| 4 | **位置条目各段的宽度** | 设备 ID 4 字节、物理偏移 6 字节（16 KiB 槽号）、密文校验和 4 字节 ⇒ 位置条目 14 字节 **状态：已定。** |
| 5 | **位置的权威住哪** | 位置条目降为提示，中央映射是解引用与释放判定的唯一入口 **状态：已定。** |
| 6 | **中央映射条目的字段表** | key 取按类复用写序键（收严后）：码 1 为 27 字节、码 2 / 码 3 为 25 字节；value 照已定项 4 / 5 **状态：已定。** |
| 7 | **指针头部带哪几段** | 头部 50：MAC 16、nonce 12、算法类型 1、压缩算法码 1、压后长度 2、extent 偏移 2、出生树 8、出生 txg 8 **状态：已定。** |
| 8 | **引用侧携带多少出生身份，映射装哪几类** | 全部指针进映射，指向码 1 的指针 88、指向码 2 / 码 3 的 86；映射树节点、树表单元、实例表单元自举豁免 **状态：已定。** |
| 9 | **出生序号怎么发、按什么次序发** | 树内从 0 起、码 2 与码 3 共用一个计数、换 checkpoint 清零、重写换号；次序同提交内生块的 bump 次序 **状态：已定。** |
| 10 | **映射 key 定宽还是按类变长** | 一律 27 字节，码 2 / 码 3 末尾补零；中央映射条目一律 55 字节 **状态：已定。** |
| 11 | **指针的偏移表、位置条目的顺序、映射树的根住哪** | 段按偏移表排，位置条目按设备身份升序；映射树根住根记录，根记录合计 457 **状态：已定。** |
| 12 | **码 2 / 码 3 的映射条目 day-1 写不写，固定点写几次** | day-1 写出；一个实例在一个 checkpoint 里每个码 2 节点与码 3 容器只写出一次 **状态：已定。** |
| 13 | **块指针分几段** | 分两段：逻辑属性住头部、每逻辑 extent 一份，物理属性住位置条目、每物理位置一份 **状态：已定。** |
| 14 | **位置条目能不能用几何公式从单一起点推算** | 不能，逐个列出 **状态：已定。** |
| 15 | **extent 距起点偏移的值域与第一版取值** | 值域 [0, 32768)、15 位够；第一版恒 0，位照留 **状态：已定。** |

```

**出处 `.claude/kb/decisions/09-加密.md:6-26`（整段抄，未转述）**

```markdown

| # | 分项 | 定案 |
|---|---|---|
| 1 | **指针里要不要额外存一份密文的非密钥校验和** | 要：指针里有 MAC 与密文校验和两个完整性字段，后者不要密钥、所有读路径都用 **状态：已定。** |
| 2 | **MAC 长度：80 位截断 vs 128 位** | 取满 128 位，不截断 **状态：已定。** |
| 3 | **算法：ChaCha20-Poly1305 vs AES-256-GCM** | 两种都实现，盘上记算法类型，运行时按 CPU 特性选默认值，格式里不钉死；加密路径必须能并行 **状态：已定。** |
| 4 | **nonce 载体：现算还是每 extent 存** | 每 extent 存一份 96 位 nonce，不现算 **状态：已定。** |
| 5 | **无密钥维护做到哪一步** | 只做 scrub：加密开启时无密钥的一侧只读只检测，不搬、不疏散、不回收 **状态：已定。** |
| 6 | **AEAD 的关联数据里放什么** | 绑逻辑身份五项，物理位置一律不进；五项对快照维是瞎的且静默，字段表欠一次重开——引这五项必须连这条一起引 **状态：已定。** |
| 7 | **已撤回·此编号不再使用** | 原题「无密钥侧的回收改哪个结构」；已定项 5 取只做 scrub 之后无密钥侧不搬，无对象 **状态：已定。** |
| 8 | **系统配置与根的认证和回滚防护** | fsid 进 KDF、系统配置因 nonce 水位而必须认证、整卷回滚不防 **状态：已定。** |
| 9 | **已撤回·此编号不再使用** | 原题「明文映射层带哪些字段、完整性靠什么」；已定项 5 取只做 scrub 之后不设明文映射层，无对象 **状态：已定。** |
| 10 | **加密进不进第一个可运行版本** | 不进，只预留格式字段 **状态：已定。** |
| 11 | **方向：整卷 AEAD** | 整卷 AEAD，加密边界与校验和边界重合；除系统配置外全部加密，元数据留最小明文头，没有例外项 **状态：已定。** |
| 12 | **格式必须 day-1 预留的四样** | MAC + nonce 空间（含自证单元自己那份）、extent 距原始起点偏移、系统配置的 KDF 标识 + 主密钥槽 + 加密类型、每个写的算法类型字段 **状态：已定。** |
| 13 | **三条与最终选型无关的硬要求** | nonce 不许由物理位置派生；顺序固定为压缩 → 加密 → MAC；若截断 MAC，读重试上限写死并有检查 **状态：已定。** |
| 14 | **压缩算法码的登记表** | 0 = 不压缩，第一版写 0、压后长度同时写 0；新算法只追加不重用码，不认识的码判该指针无法解引用 **状态：已定。** |
| 15 | **前提编号（A 系列）的登记表** | A2（系统配置是明文）、A6（AAD 期望值不来自明文侧）、A7（含义待查）的唯一登记位 **状态：已定。** |
| 16 | **AAD 每个字段的期望值从哪来** | 只许来自读者的查找路径或已被上一级 MAC 认证的密文侧字段；不许来自明文侧，也不许从指针里读回来 **状态：已定。** |
| 17 | **与其他决策的连锁** | nonce 不绑物理位置；parity 对密文算、副本是同一份密文；birth txg 不能当 nonce；绕过 COW 的快路径不许绕过加密 **状态：已定。** |

```
