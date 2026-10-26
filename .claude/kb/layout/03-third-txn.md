# 删除写出哪些字节（新池新建文件与覆盖写之后新出现的盘上形态）

**中文** · [English](03-third-txn.en.md) · [日本語](03-third-txn.ja.md)

**里程碑「删除与多次 COW」三·6 的登记表**（[03-third-txn.md](../milestone/03-third-txn.md)），2026-10-01 立。新池新建文件的每一段字节在 [layout/01-first-txn.md](01-first-txn.md)，覆盖写、释放、回退与复用新写出的形态在 [layout/02-second-txn.md](02-second-txn.md)；这里只登记删除（单文件 unlink）与管理员回退的谱系记录新写出的形态。同一个字段的宽度与落点以 layout/01-first-txn.md 为准、不另抄。每一行指一条决策分项，指不到分项的写成「格式级空白」。删除已经实装：`crates/singlefs-core/src/transaction.rs` 的 `publish_unlink_intent`（意图）、`publish_unlink_step`（一批）、`walk_the_deletion_queue_until_it_is_empty`（走空），挂载续做在 `crates/singlefs-core/src/mount.rs` 的 `continue_the_deletion_queue_until_it_is_empty`；`crates/singlefs-harness/tests/unlink_single_file_publishes.rs` 钉住意图、一批、最后一批三种发布的重写角色集合。盘上字节由 E172（删除写出的字节的干跑） 照 E142（新池新建文件的干跑） 的形态逐字段核过，核的是退役树加进之前那一版：退役树带来的 tr@0、tr 两格与标签 6 的退役项没核（C775（E172 装置没有退役项））；段序列表里标「推的，待干跑核」的几格照旧是推的。

## 一、形态

| 形态 | 第一次出现在哪次发布 | 写成什么 | 决策分项 | 里程碑那一步 |
|---|---|---|---|---|
| 意图（不立 logged_ops 条目） | 意图那次发布 | 一次发布里写出三样：被删的 inode 记录写成 nlink = 0、size 不变（「inode 记录的 nlink 第一次为 0」那一行）；`deleted_inodes` 加 key = 被删的 inode 号（「`deleted_inodes` 节点条目」那一行）；往墓碑树开放容器追加一条整对象墓碑（「墓碑记录」「墓碑容器」两行）；不立 logged_ops 条目 | D8（核心索引结构） 已定项 16 第 1 条 | 三·2、三·6 |
| `deleted_inodes` 节点条目 | 意图那次发布（被删的 inode 一条）；树本身在第一个文件版本那次发布零根注册 | key = inode 号 8 字节，无 value，不带对象出生代；队列 = inode 树里 nlink = 0 的号集合（派生态）；最后一批删掉那条 key，队列空时树表条目的根指针置零 | D8（核心索引结构） 已定项 16 | 三·2、三·6 |
| `deleted_inodes` 根（码 2） | 意图那次发布（队列由空变成一个号）；不是最后一批的批照抄；最后一批队列空，不写它，树表里它的根指针置零 | 单元类型标签 2；头 131（含预留位 29）和各字段的落点以 [layout/01-first-txn.md](01-first-txn.md)「inode 树根（码 2）」那一行为准（诞生代号、fsid、写序、出生序号、载荷 CRC 同形）；写的值：树 ID 65544、key 宽 8、条目数 = 队列里的号个数（任何一版至多 1）、条目宽 8、条目区从 131 起（每个号 8 字节小端）；一个根装得下 2031 个号（`deleted_inodes_one_root_node_holds`：(16384 − 131) ÷ 8 取整），多于它拒成类型化错误。状态：层级已定：写 0（D8（核心索引结构） 已定项 16 第 3 条）；key 区间已定：叶层码 2 节点取首末两条条目的 key（D18（块里携带什么信息） 已定项 2）；代码 key 区间写队列首末两个号（一条 key 时 [N, N]），与条款同值；这个根的区间由池级 checker 在树表走读里判（`crates/singlefs-checker/src/walk.rs` 的 `read_index_node` 按 `FirstAndLastEntries` 调 `crates/singlefs-checker/src/lib.rs` 的 `check_index_node_keys`，头里的区间与首末条目的 key 不等判 I-1.1（块头自述逻辑地址） 红） | D8（核心索引结构） 已定项 16 第 1 条（key 与条目）/ 第 6 条（至多一条 key、多于一个根的拒绝） | 三·2、三·6 |
| inode 记录的 nlink 第一次为 0 | 意图那次发布 | [layout/01-first-txn.md](01-first-txn.md)「四、文件元数据（inode）」那一节 inode 记录偏移 24 那一格（mode / uid / gid / nlink）里的 nlink，偏移 36（`research/prompts/m3-delete-r1-main-verification.md` 第三节判 nlink 那一格）：意图那次写成 0、size 不变；`InodeRecord` 的 parse 读它、to_bytes 写结构里的值 | D8（核心索引结构） 已定项 16 | 三·2、三·6 |
| nlink = 0 的 inode 记录一起改的字段 | 意图那次发布（改动计数、三个时间秒）；不是最后一批的每一批（size、blocks、改动计数）；最后一批（删记录） | 偏移与宽度照 [layout/01-first-txn.md](01-first-txn.md)「四、文件元数据（inode）」那一节的 inode 记录。偏移 88 改动计数 = 这次发布的 checkpoint_txg，意图与每一批都改；偏移 64 起 atime / mtime / ctime 秒三格同值，意图写调用方给的值，各批照抄上一版这条记录的值；偏移 40 size，意图不变，每一批 = 剩下的数据单元数 × 32634（一个数据单元的净荷容量）；偏移 48 blocks = ⌈size ÷ 512⌉，随 size 现算；其余偏移意图与各批照抄。最后一批不写这条记录，从 inode 叶容器里删掉它：删的是 inode 树里最后一条记录时，inode 树写零条目根（key 区间 [0, 0]）；那一片删空而树里还有别的叶时，整片不落盘（`remove_record_from_leaf_containers`） | D8（核心索引结构） 已定项 16 第 2 条（size 与删记录）/ 第 5 条（删空）/ 第 7 条 ①b（改动计数与时间秒） | 三·2、三·6 |
| 墓碑记录 | 意图那次发布（被删的对象一条） | 40 字节，字段在「二、墓碑记录」那一节 | D18（块里携带什么信息） 已定项 24 | 三·2、三·6 |
| 谱系记录 | 管理员回退那次发布（每次回退一条） | 24 字节，字段在「三、谱系记录」那一节 | D18（块里携带什么信息） 已定项 24；D23（journal 的角色与格式） 已定项 14 | 三·2、三·6 |
| 墓碑容器 | 意图那次发布；有新记录的那次发布才重写开放容器，不删除的发布照抄；满了才关 | 码 3 打包记录单元，类身份段的打包记录类型 = 1、记录宽 = 40、出生树 = 墓碑树的树 ID；含 29 字节预留位的头 136，一个 32 KiB 容器 ⌊(32768 − 136) ÷ 40⌋ = 815 条；容器号沿叶序（按叶从左到右）严格递增，从 1 起（根长高之后根那一层的条目容器号恒 0，见 I-9.21（产品镜像里非最右墓碑容器装满）），下一个号 = 墓碑树里最大号 + 1，零根时 1，0 不用（与 D18（块里携带什么信息） 已定项 11 里克隆头类型 1 的容器号从 1 起同一条规则） | D18（块里携带什么信息） 已定项 24 / 已定项 11 | 三·2、三·6 |
| 墓碑树根（码 2） | 意图那次发布（墓碑树由零根变成有根，根跟着开放容器重写）；不是最后一批的批与最后一批照抄 | 单元类型标签 2；头 131 与各字段的落点同「`deleted_inodes` 根（码 2）」那一行，以 [layout/01-first-txn.md](01-first-txn.md)「inode 树根（码 2）」那一行为准；写的值：树 ID 65545、key 宽 8（容器号）、条目宽 120 = 分隔 key（容器号）8 + 身份引用 26（出生树 65545、打包记录类型 1、容器号、容器出生代）+ 子指针 86，条目数 = 根的孩子个数，孩子是码 3 墓碑容器；根满时长高。状态：层级已定：根直接管容器时 1、根满长高一次加一（D18（块里携带什么信息） 已定项 24 第 1 条）；key 区间是子树覆盖区间（D18（块里携带什么信息） 已定项 2）：层级 1 贴紧首末两条容器号，层级 ≥ 2 是 [第一个孩子那一枝最左容器号, 最后一个孩子那一枝最右容器号] | D18（块里携带什么信息） 已定项 24 第 1 条 | 三·2、三·6 |
| 谱系容器 | 管理员回退那次发布 | 码 3 打包记录单元，打包记录类型 = 5（不复用登记过的 3）、记录宽 = 24、出生树 = 谱系树的树 ID；一个 32 KiB 容器 1359 条；只增不删；容器号从 1 起，下一个号 = 谱系树里最大号 + 1，零根时 1，0 不用（规则同墓碑容器） | D18（块里携带什么信息） 已定项 24 / 已定项 11 | 三·2、三·6 |
| 退役容器 | 有退役记录的那次发布（叶删空摘掉、合并被吸收）；有新记录的那次发布才重写开放容器，没有的发布照抄；满了才关；第一版不回收 | 码 3 打包记录单元，打包记录类型 = 3、记录宽 = 34、出生树 = 退役树的树 ID；含 29 字节预留位的头 136，一个 32 KiB 容器 ⌊(32768 − 136) ÷ 34⌋ = 959 条；容器号从 1 起，下一个号 = 退役树里最大号 + 1，零根时 1，0 不用（规则同墓碑容器）；树根与墓碑树同形：码 2 根、条目宽 120，孩子是码 3 退役容器，根满时长高；字段在「五、退役记录」那一节 | D18（块里携带什么信息） 已定项 24 / 已定项 11 | 三·6 |
| 树表四条新条目 | 第一个文件版本那次发布 | `deleted_inodes`、墓碑树、谱系树、logged_ops（零根，D8（核心索引结构） 已定项 5）各一条，条目字段照 [layout/01-first-txn.md](01-first-txn.md)「七、发布（根记录与根槽）」那一节的树表条目；根指针零；树表 7 → 11 条（再加退役树一条为 12，见「树表退役树条目」那一行）；四棵的树 ID：`deleted_inodes` 65544、墓碑树 65545、谱系树 65546、logged_ops 65547（原来的八棵占 65536 到 65543），种类码：`deleted_inodes` 9、墓碑树 10、谱系树 11、logged_ops 12；树 ID 水位 mkfs 时 65536、第一个文件版本那次之后 65549（四棵占到 65547，退役树 65548，见「树表退役树条目」那一行）；头 ID（树表条目里 8 字节）：`deleted_inodes`、墓碑树写 inode 树的树 ID 65537，谱系树与 logged_ops 写 0 | D8（核心索引结构） 已定项 9 / 已定项 11 / 已定项 16；D18（块里携带什么信息） 已定项 24；D5（快照 / 空间记账机制） 已定项 9 | 三·2、三·6 |
| 树表退役树条目 | 第一个文件版本那次发布（与上一行同一次） | 退役树一条，零根，条目字段照 [layout/01-first-txn.md](01-first-txn.md)「七、发布（根记录与根槽）」那一节的树表条目；根指针零；树表 11 → 12 条；树 ID 65548，种类码 13；树 ID 水位第一个文件版本那次之后 65549；头 ID（树表条目里 8 字节）写 inode 树的树 ID 65537（同墓碑树） | D8（核心索引结构） 已定项 9 / 已定项 11；D18（块里携带什么信息） 已定项 24 第 6 条 | 三·6 |
| 记账标签 6 的值 | 第一个文件版本那次发布起，每一版都按那一版现算 | 记账树叶里 key (6, 0, 0xFFFF_FFFF, 代) 那一行的 value 8 字节（行的形态在 [layout/01-first-txn.md](01-first-txn.md)「记账 key」那一行），新池新建文件里这个值是式子的值（n_live = n_记录 = 1、两个 H 都是 0 时 196 760，[layout/01-first-txn.md](01-first-txn.md)「记账条目」那一行的已承诺预留就是它）：副本数 × ((2 + H_墓碑 + 1 + 2 + H_退役 + 1) × 16384 + 41 × n_live + 35 × n_记录)，单位字节、按副本之和记（C375（待删占用与已承诺预留按份还是按和记没有条款））；H_墓碑、H_退役 是现行那一版墓碑树、退役树的内部层数（零根与只有根时 0），n_live 是这一版 inode 树里 nlink ≥ 1 的记录数，n_记录 是这一版 inode 树里全部记录数（含 nlink = 0 的），41 = ⌈2 × 16384 ÷ 815⌉、35 = ⌈32768 ÷ 959⌉ 每份副本、每个对象先取整再乘 n_live、n_记录；树表里没有 `deleted_inodes` 条目的版本取 0；行内偏移与值随实现，连带记账树单元的载荷校验和变 | D3（空间分配） 已定项 2；D5（快照 / 空间记账机制） 已定项 4 | 三·2、三·6 |

<!-- format-const: TOMBSTONE_RECORD_BYTES = 40 -->
<!-- format-const: TOMBSTONE_CONTAINER_RECORDS = 815 -->
<!-- format-const: LINEAGE_RECORD_BYTES = 24 stale=记录宽 = 16 -->
<!-- format-const: LINEAGE_CONTAINER_RECORDS = 1359 -->
<!-- format-const: DELETED_INODES_ENTRY_BYTES = 8 -->

## 二、墓碑记录

| 偏移 | 字段 | 宽度 | 取值 | 指向 |
|---|---|---|---|---|
| 0 | 对象 ID | 8 | 被删的 inode 号 | D18（块里携带什么信息） 已定项 24 |
| 8 | 对象出生代 | 8 | 那条 inode 记录偏移 8 的对象出生代 | D18（块里携带什么信息） 已定项 24 |
| 16 | 区间起 | 8 | 0（第一版只写整对象） | D18（块里携带什么信息） 已定项 24 |
| 24 | 区间止 | 8 | 2⁶⁴ − 1 | D18（块里携带什么信息） 已定项 24 |
| 32 | 死亡 txg | 8 | 意图那次发布的 checkpoint_txg | D18（块里携带什么信息） 已定项 24 |

## 三、谱系记录

| 偏移 | 字段 | 宽度 | 取值 | 指向 |
|---|---|---|---|---|
| 0 | T_rb | 8 | 回退那次发布的 checkpoint_txg | D18（块里携带什么信息） 已定项 24 |
| 8 | T_old | 8 | 回退目标 R_old 的 txg | D18（块里携带什么信息） 已定项 24 |
| 16 | H_cur | 8 | cur（回退之前的现行那一版）的 inode 号水位，从 cur 那一版读 | D18（块里携带什么信息） 已定项 24 第 5 条 |

## 四、段序列

删除类发布（意图、不是最后一批的批、最后一批、挂载续做里的批）和管理员回退都是普通发布的形状：[单元 × 2 盘] 屏障 [journal 记录 × 2 盘] 屏障 [根 FUA] [系统配置槽 × 2 盘] 屏障（[layout/01-first-txn.md](01-first-txn.md) 段序列登记表「普通发布（新池新建文件）」那一行）。单元数一栏里 harness 用例钉住重写角色集合的，写明用例；没钉住的标推的。段串（两块盘）全是推的，三·6 干跑核。角色名照 `TransactionUnit::tag`：t2 extent 树根、t2下@层级.序号 extent 下段叶、t3 inode 叶容器（最左一片）、t4 inode 树根、t5 分配记录树根、t5@层级.盘.序号 分配记录树根之下的节点、t6 记账树根、t7 映射树、t8 树表、td `deleted_inodes` 根、tt@叶序 墓碑容器、tt 墓碑树根、tr@叶序 退役容器、tr 退役树根。

| 发布 | 单元数（每盘） | 段串（两块盘） | 出处 |
|---|---|---|---|
| 意图 | 13：t3 t4 t5@0.0.61 t5@0.1.61 t5@1.0.0 t5@1.1.0 t5 t6 td tt@0 tt t7 t8（harness 钉住）；副本原型量到的 10，加 `deleted_inodes` 根、墓碑容器、墓碑树根各 1 | `26+2+1+2`（推的，待干跑核） | `crates/singlefs-harness/tests/unlink_single_file_publishes.rs` 的 `intent_rewrites_the_inode_leaf_and_root_the_deleted_inodes_root_and_the_tombstone_container_and_root`；`research/prompts/m3-delete-r3-main-verification.md` 第三节判 F1 状态数那一格 |
| 不是最后一批：3 → 2（批大小 1） | 12：t2下@0.0 t2 t3 t4 t5@0.0.61 t5@0.1.61 t5@1.0.0 t5@1.1.0 t5 t6 t7 t8（harness 钉住）；`deleted_inodes` 根与墓碑树照抄，不写 td、tt | `24+2+1+2`（推的，待干跑核） | `crates/singlefs-harness/tests/unlink_single_file_publishes.rs` 的 `a_batch_of_one_unit_rewrites_the_cut_lower_leaf_and_the_extent_root_and_releases_the_tail_unit` |
| 不是最后一批：2 → 1（批大小 1） | 11（推的，待干跑核）：同 3 → 2 那一批去掉 t2下@0.0，截到一个单元不建下段、只重写 extent 根 | `22+2+1+2`（推的，待干跑核） | `crates/singlefs-core/src/unlink.rs` 的 `a_deletion_batch_down_to_one_data_unit_drops_the_whole_lower_segment`（钉 extent 树的规划，没钉整次发布的角色集合） |
| 每一批的新写上界 | 随文件几何：只重写被截掉那一片 extent 下段叶与它的祖先；每批新写上界 W = 24 + (6 + H_退役) + Σ_ℓ min(N_ℓ, 2·(K + 26)) 槽（H_退役 是退役树的内部层数；6 + H_退役 是一次发布新写的退役单元上界） | — | D8（核心索引结构） 已定项 16；D28（挂载期承诺量） 已定项 1 |
| 最后一批（1 → 0，批大小 1） | 12：t2 t4 t5@0.0.61 t5@0.1.61 t5@1.0.0 t5@1.1.0 t5 t6 tr@0 tr t7 t8（harness 钉住）。比 2 → 1 那一批（11）多 1（推的，待干跑核）：少 inode 叶容器 t3，被删的那条记录是那片叶里唯一一条，叶删空整片不落盘；多退役容器 tr@0 与退役树根 tr，那片叶的身份消失，退役树开第一片容器、写根；叶里还有别的记录时那一片照写；删空的不是最右一片时只重写内容变了的叶与祖先、右边各片不挪位重写（判决 `research/prompts/m3-retire-r1-main-verification.md` 第六节 R3，第 50 批），这两支是代码文档注释的读法，harness 没钉。比 3 → 2 那一批少 t2下@0.0 与 t3（extent 树只剩上段零条目根），多 tr@0 与 tr，个数同为 12。`deleted_inodes` 删空取根指针置零，不写它的节点；不是最后一批的批同样不写 td（队列没变就照抄），所以 td 不是最后一批比一批少的那一个 | `24+2+1+2`（推的，待干跑核） | `crates/singlefs-harness/tests/unlink_single_file_publishes.rs` 的 `the_final_batch_removes_the_record_the_queue_key_and_every_remaining_unit_in_the_same_publish`；`crates/singlefs-core/src/inode_tree.rs` 的 `remove_record_from_leaf_containers`；`research/prompts/m3-delete-r2-main-verification.md` 第三节判空形态那一格 |
| 挂载续做 | 每一批与会话里同几何的那一批不同形：续做 2 → 1 与最后一批各比会话批多写分配记录树第 62 片叶一对（t5@0.0.62、t5@0.1.62），有条款依据（D8（核心索引结构） 已定项 14 按位置寻址），差别跟落点有没有跨进第 62 片叶走（E172（删除写出的字节的干跑） 第 2 次，2026-10-05）；harness 没钉续做批的角色集合 | 同几何的会话批的段串，落点跨进第 62 片叶时再多一对叶（E172（删除写出的字节的干跑） 实测，不再标推的） | `crates/singlefs-core/src/mount.rs` 的 `continue_the_deletion_queue_until_it_is_empty`（写行与暖机之后、交回之前走空）；`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs` 的 `the_mount_hands_in_every_continuation_step_after_its_warm_up_in_publish_order`（钉续做三步接在暖机之后、按 txg 连着：留 2、留 1、最后一批） |
| 管理员回退 | 见 [layout/01-first-txn.md](01-first-txn.md) 段序列登记表「实例切换 / 管理员回退」那一行（第 412 行），这里不另抄 | 同那一行 | `research/prompts/m3-delete-r3-main-verification.md` 第三节判其余数那一格 |

整条流：unlink 一次调用 = 走空 + 意图 + 全部批（D8（核心索引结构） 已定项 16 第 6 条）；队列本来就空时走空一个写都不发，挂载续做的走空接在写行与暖机之后。发布与发布之间，上一次发布末尾的 2 个系统配置槽写自成一段（[layout/01-first-txn.md](01-first-txn.md) 段序列登记表后面「发布与下一次发布之间有发布末尾那道屏障」那一段），所以整条流的段串是各次发布的段串首尾相接：3 单元文件、批大小 1 的一次 unlink 是 `26+2+1+2`、`24+2+1+2`、`22+2+1+2`、`24+2+1+2` 四串相接（推的，待干跑核；崩溃状态数没算）。

## 五、退役记录

| 偏移 | 字段 | 宽度 | 取值 | 指向 |
|---|---|---|---|---|
| 0 | 被退役身份·出生树 | 8 | 被退役的叶容器身份的出生树（单头时是 inode 树的树 ID） | D18（块里携带什么信息） 已定项 24 |
| 8 | 被退役身份·打包记录类型 | 2 | 被退役的叶容器身份的打包记录类型（inode 叶容器是 2） | D18（块里携带什么信息） 已定项 24 |
| 10 | 被退役身份·容器号 | 8 | 被退役的叶容器身份的容器号 | D18（块里携带什么信息） 已定项 24 |
| 18 | 被退役身份·容器出生代 | 8 | 被退役的叶容器身份的容器出生代 | D18（块里携带什么信息） 已定项 24 |
| 26 | 死亡 txg | 8 | 身份消失那次发布的 checkpoint_txg（叶删空摘掉、合并被吸收那一次）；不取 unlink 意图那次，不取最后一版的诞生代号 | D18（块里携带什么信息） 已定项 24 |

小端，共 34 字节，无填充、无 flags、无预留；前四段与 `PackedIdentity` 同序同宽。

<!-- format-const: RETIREMENT_RECORD_BYTES = 34 -->
<!-- format-const: RETIREMENT_CONTAINER_RECORDS = 959 -->
