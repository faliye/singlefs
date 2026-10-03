# m3-unlink-code-r1 云端正推腿（Sonnet）报告

日期：2026-10-03。立场：照背景材料第四节分工表「云端正推」那一行，每一格从条款原文推到 `crates/` 里的行，再推到钉它的测试与变异行；不判别、不出判决。
读法：代码行号全部取自冻结副本 `/tmp/claude-1000/m3-unlink-code-r1/tree/`，kb 行号取自 kb 文件自己（用 `grep -n` 现取，不用背景材料里的数）；引文由 `research/scripts/cite-check.py` 复核（末行贴在最后）。
禁读清单照守：没读攻方腿的报告、模型目录、草稿目录与派发提示。

我自己加的复核（不是主 agent 要求的格，写在这里是因为它们回答了 K1、K3 的问题）：在草稿副本 `/tmp/claude-1000/m3-unlink-code-r1-sonnet/work/`（冻结副本逐文件拷贝）里写了一个只加不改的 harness 档探针测试
`research/prompts/m3-unlink-code-r1-sonnet-model/sonnet_probe_multi_leaf_unlink.rs`，
把 145 / 287 / 288 / 289 / 300 / 431 个数据单元的文件经真发布路径（`publish_unlink_intent` + 产品批大小 144 的 `publish_unlink_step`）删到底，逐版判：池级 checker 无红、从盘上重建的版本与发布的版本相同、冷走读答「没有文件」。
编译与跑法：`CARGO_TARGET_DIR=<草稿目录>/target nice -n 19 bash research/scripts/capped.sh 10 cargo test --offline -p singlefs-harness --test sonnet_probe_multi_leaf_unlink`，经 `research/scripts/run-with-memory-cap.sh 8G` 跑（会话里另有一条在跑，设了并行理由）；没碰 checker 档。

## K1　分批与幂等（L1）

### K1.a　只改尾叶那一支与 K-R3-1 逐字

条款（两处，整句抄）：
- 「删除批只重写被截掉那一片下段叶与它的祖先（K-R3-1，extent 写法的新分支，要过 .claude/rules/fs-design.md 五条硬要求）」（`.claude/kb/decisions/08-核心索引结构.md:459`）
- 「只改尾叶：删除批只重写下段里被截掉那一片叶与它的祖先（下段按位置寻址），W 上界不含文件大小项」（`research/prompts/m3-delete-r3-forks.md:9`）

从条款推出的应有样子：留下的单元数记 k。罩的单元全在 `[0, k)` 里的下段节点一个字节不变、照抄；新下段里罩着第 k 个单元的那一枝（每层最右那个）重写；整个落在被截掉那一段里的节点不再出现、换下；k = 1 不建下段（内联）；k 恰在叶边界上时留下的最后一片叶照抄并成为新下段根；上段只重写这个文件那片叶到根的一串，别的照抄。

代码（冻结副本）：
- 判「整片留下」：「let is_entirely_kept = |position: &ExtentLowerNodePosition| {」（`crates/singlefs-core/src/unlink.rs:176`），条件是 「last_data_unit < data_units_kept」（`crates/singlefs-core/src/unlink.rs:178`）；
- 新下段只列 `lower_segment_nodes_of_a_file_without_holes(data_units_kept)` 的节点：「lower_segment_nodes_of_a_file_without_holes(data_units_kept)」（`crates/singlefs-core/src/unlink.rs:181`）；
- 上段这个文件的路径重写、别的照抄：「if path_of_the_file.contains(position) {」（`crates/singlefs-core/src/unlink.rs:204`）；
- 换下的是上一版里没被照抄的下段节点加上段那一串：「.filter(|(position, _)| !carried_lower.contains(position))」（`crates/singlefs-core/src/unlink.rs:220`）；
- 发布路径只在 Batch 这一步调它：「(DeletionStepOfThePublish::Batch { .. }, Some(previous_version)) => {」（`crates/singlefs-core/src/transaction.rs:4942`）。

判定：**一致**。

核对（三条互不共用的路）：
1. 读代码对条款：上面五处与应有样子逐项相符。截在叶边界（k = 144、288）：新下段只有叶 `(0, 0)` 或 `(0, 0)(0, 1)(1, 0)`，罩的单元全 < k 的叶判「整片留下」；k = 1：「if data_units <= 1 {」（`crates/singlefs-core/src/extent_tree.rs:210`） 交回空下段，内联。
2. 入库的纯函数单测：「fn a_deletion_batch_rewrites_only_the_cut_lower_leaf_and_its_ancestors() {」（`crates/singlefs-core/src/unlink.rs:389`）（300 → 200）、「fn a_deletion_batch_cut_on_a_leaf_boundary_carries_the_kept_leaf_as_the_new_lower_root() {」（`crates/singlefs-core/src/unlink.rs:427`）（300 → 144）、「fn a_deletion_batch_down_to_one_data_unit_drops_the_whole_lower_segment() {」（`crates/singlefs-core/src/unlink.rs:452`）（3 → 1）；变异行 「三·2 unlink：删除批把整片留下的下段叶也重写」（`crates/mutations.tsv:2056`）、「三·2 unlink：截在叶边界上时留下的叶也重写」（`crates/mutations.tsv:2057`）、「三·2 unlink：删除批不换下被截掉的下段节点」（`crates/mutations.tsv:2058`）。
3. 我的探针（真发布路径，产品批大小 144，原样输出见 `research/prompts/m3-unlink-code-r1-sonnet-model/probe-output.txt`）：
   - 289 → 145：重写角色 `t2下@0.1,t2下@1.0,t2,…`，即被截掉那片叶 `(0, 1)`、下段根 `(1, 0)`、上段根；下段节点 `(0,0)(0,1)(1,0)` 里 `(0,0)` 照抄；
   - 300 → 156 → 12：第一批重写 `(0,1)`、`(1,0)`、`t2`，第二批重写 `(0,0)` 与 `t2`，下段缩成单叶；
   - 288 → 144（叶边界）：这一批 extent 树的重写角色里只有 `t2`（上段根），`(0,0)` 照抄成为新下段根；145 → 1：extent 树的重写角色只有 `t2`，下段为空（内联）（`t2` 是 extent 树，`t3` inode 叶，`t4` inode 根，`t5` 分配记录树，`t6` 记账树，`t7` 映射树，`t8` 树表，`td` deleted_inodes 根，`tt` 墓碑树——对照入库用例里的角色表）；
   - 这六个文件大小（145 / 287 / 288 / 289 / 300 / 431）在每一版上：池级 checker 无红、`rebuild_version` 重建出来的记录、队列、数据指针、extent 树、映射 key 与发布的那一版逐字段相等、冷走读答「没有文件」。

推翻它的现象：任一版里 `rebuild_version` 与发布的版本在 extent 树或映射 key 上不等，或 checker 红，或照抄的下段节点罩着被截掉的单元。

**附带的覆盖缺口（用例是不是真判）**：入库的 harness 用例里删除全都是 3 个单元的文件、批大小 1（「assert_eq!(grown.data_pointers.len(), 3, "第一个文件三个数据单元");」（`crates/singlefs-harness/tests/unlink_single_file_publishes.rs:57`）），多于一片下段叶的文件只在纯函数单测里判过计划，没有一条入库用例让真发布路径装出被照抄的下段叶 / 内部节点的字节。
我在草稿副本里把 `is_entirely_kept` 的 `<` 改成 `<=`（k = 143 / 287 时，罩着第 k 个、也就是被截掉的第一个单元的那片叶会被当成整片留下）：
- `cargo test -p singlefs-core --lib -- unlink::` 7 条全绿（`test result: ok. 7 passed; 0 failed`，原样输出在 `probe-output.txt`）；入库 harness 删除用例的 k 只取 1、2，`143 <= 2` 为假，变异前后行为逐字相同（推的，没逐条重跑）；
- 我的探针 287、431 两条红：`重建：Walk(InvariantViolated { invariant: "E142 走读同款", detail: "这个文件的 extent 记录条数与 inode size 按净荷容量除出来的单元数不符" })`。
所以 `<` 与 `<=` 在入库的变异表里没有区分（变异行只改成 `data_units_kept < last_data_unit`），这一格的字节级判别只有我的探针有。

K-R3-1 要求的 `.claude/rules/fs-design.md` 五条硬要求（「每条分支单独验，包括崩溃点重放。」（`.claude/rules/fs-design.md:149`））里第 3 条「崩溃点重放」：extent 新分支的逐写点崩溃枚举属于三·4（「覆盖：固定脚本到 E 之后加 unlink 段」（`.claude/kb/milestone/03-third-txn.md:86`）），这一批没有；harness 里只有一个写点的用例（见 K1.c ③）。判定：**规则没说**（本批是否要含这一条，作业点表三·2 的验收栏没点名崩溃枚举，点名的是「崩在每两批之间的中间态 checker 全绿」）。

### K1.b　最后一批三件事同一次发布

条款：「最后一批释放剩下全部单元、删 inode 记录、删 deleted_inodes 那条 key，同一次发布；只有这一条路，不设一次做完的快路」（`.claude/kb/decisions/08-核心索引结构.md:459`）。

代码：一个入口 `DeletionStepOfThePublish::FinalBatch`，三件事在同一个 `publish_deletion_step` 里算：
- 删 inode 记录：「remove_record_from_leaf_containers(&containers_before, inode)?」（`crates/singlefs-core/src/transaction.rs:4925`）；
- 删队列里的号：「DeletionStepOfThePublish::FinalBatch { inode } => keys.retain(|key| *key != inode),」（`crates/singlefs-core/src/transaction.rs:3949`）；
- 释放剩下的单元（第一个文件）：「DeletionStepOfThePublish::FinalBatch { inode } if inode.0 == FIRST_INODE_NUMBER => 0,」（`crates/singlefs-core/src/transaction.rs:3929`） 加 「(self.carried_data_units..previous.data_pointers.len())」（`crates/singlefs-core/src/transaction.rs:4839`）；
- 「不设快路」：「if data_units_remaining <= data_units_per_batch {」（`crates/singlefs-core/src/unlink.rs:120`）——剩下不多于一批也走最后一批这一个形状。
用例：「fn the_final_batch_removes_the_record_the_queue_key_and_every_remaining_unit_in_the_same_publish() {」（`crates/singlefs-harness/tests/unlink_single_file_publishes.rs:244`）（txg 只加 1、三件事都在）；变异行 「三·2 unlink 最后一批：不删 deleted_inodes 那条 key（删 key 不与最后一批同一次发布）」（`crates/mutations.tsv:2068`）。
判定：**一致**。推翻它的现象：最后一批之后的版本里 inode 记录、队列 key、剩余数据单元三者任何一个还在，或它们分处两个 txg。

### K1.c　幂等判据五条，各自的用例是不是真判那一条

条款：「①b 同一份崩溃镜像各自重挂载续做同一批，两份盘逐字节相同；② 删完再续做一个写都不发；③ 每个写点截断、重开续做到底，终态逻辑内容与不崩溃的终态相同、每个崩溃态 checker 全绿；④ 每个被判状态的记账标签 4、6 等于从这一版镜像现算的值。」（`.claude/kb/decisions/08-核心索引结构.md:464`）
「逻辑内容」的定义：「含 inode 每条（号、对象出生代、size、nlink）、extent 单元序号集与内容校验和、deleted_inodes key 集、墓碑记录多重集（含死亡 txg）、谱系记录多重集、标签 12 / 4 / 6、读者的答案」（`.claude/kb/decisions/08-核心索引结构.md:464`）；另加 「另加 harness 层每种发布的重写角色集合钉绝对值（零轮）」（`.claude/kb/decisions/08-核心索引结构.md:464`）。

实现一侧五条都**一致**：下一步只由「现行那一版还剩几个单元」与批大小定（「下一步只由「现行那一版这个文件还剩几个数据单元」与批大小定」（`crates/singlefs-core/src/unlink.rs:6`）），续做与不崩溃一路做下来因此同一个计划；删完队列空时 `walk_the_deletion_queue_until_it_is_empty` 的第一轮就返回（「let Some(inode) = current.deleted_inodes.first().copied() else {」（`crates/singlefs-core/src/transaction.rs:5906`）），一个写都不发。

| 条 | 钉它的入库用例 | 用例判别力 | 缺口 / 依据 |
|---|---|---|---|
| ① 计划相等 | 纯函数 「fn the_plan_continued_from_any_intermediate_version_equals_the_rest_of_the_uninterrupted_plan()」（`crates/singlefs-core/src/unlink.rs:365`）；harness 「fn the_publishes_continued_by_a_remount_between_two_batches_equal_the_rest_of_the_uninterrupted_run(」（`crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs:205`）；变异行 「三·2 第 7 条 ①：挂载续做不按装上分配器的批大小（续做的计划与一路做下来的不等）」（`crates/mutations.tsv:2091`） | 部分真判 | 用例比的 `LogicalContent` 里数据单元只比个数（「data_units: usize,」（`crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs:156`）），条款要的「extent 单元序号集与内容校验和」「谱系记录多重集」「读者的答案」不在比较里（读者的答案只在 ③ 那一条的终态上判）。数据单元内容只由第一个文件定、续做不重写它们，所以漏掉的三样今天不会让用例假绿，但改坏「截哪几个单元」而个数不变的变异它判不出（推的，没跑这个变异） |
| ①b 两份盘逐字节相同 | 「fn two_copies_of_the_image_between_two_batches_continue_to_byte_identical_disks() {」（`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:399`），比的是两份续做之后的整块内存盘 「first_copy == second_copy,」（`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:425`），并核续做真的写了盘 「assert!(first_copy != image_between_two_batches, "续做真的写了盘");」（`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:428`）；变异行 「三·2 第 4 条：可写挂载不续做（两份镜像各自续做那一条）」（`crates/mutations.tsv:2080`） | 真判（判的是不确定性） | 两份由同一段确定性代码从同一份镜像跑出，它抓的是时钟、哈希序、线程这类不确定来源，抓不到「两份都错得一样」；条款要的就是前者 |
| ② 删完再续做不发写 | 「fn continuing_after_the_queue_is_empty_publishes_nothing() {」（`crates/singlefs-harness/tests/unlink_single_file_publishes.rs:396`）；变异行 「三·2 unlink 走空：队列空时也记一次发布」（`crates/mutations.tsv:2070`） | 真判 | 判的是 API 层（`publish_unlink_steps_until_the_queue_is_empty` 交回 0 且现行版本不变）；可写挂载层的早退 「if current_version_of_trees.deleted_inodes.is_empty() {」（`crates/singlefs-core/src/mount.rs:4337`） 只是省一次克隆，去掉它走空的第一轮照样返回、一个写都不发（推的）；变异行 「三·2 第 4 条：可写挂载不续做」（`crates/mutations.tsv:2079`） 钉的是反方向（队列不空时不续做） |
| ③ 每个写点截断 | 「fn a_crash_halfway_through_the_writes_of_the_first_batch_continues_on_remount_to_the_uninterrupted_end_state(」（`crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs:262`） | 一个写点（第一批的写取半） | 用例自己的注释写明 「逐写点的全量枚举是 checker 档的活」（`crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs:260`）；崩溃枚举流属三·4，这一批没有。我的补充见下 |
| ④ 标签 4、6 现算 | 「fn tags_four_and_six_of_every_deletion_version_equal_the_values_recomputed_from_its_image() {」（`crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs:314`）：n_live 数重建出来的 inode 树，H 从盘上墓碑树根单元的码 2 头读 「u64::from(parse_index_node(&root.bytes).expect("墓碑树根解得开").level) - 1」（`crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs:342`），并钉 98 386 / 98 304 | 真判（只判发布出来的五版） | 校验路径与实现不共用代码（用例里自己再算一遍式子）。崩溃态、挂载续做出来的版本不在这条用例里；回退那一版的标签 6 另有 「fn rolling_back_to_the_version_before_the_intent_recomputes_tag_six_from_the_rollback_targets_inode_tree(」（`crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs:438`） |
| 重写角色集合钉绝对值 | 意图 「意图那次只重写 inode 叶与根、分配记录树、记账树、deleted_inodes 根、墓碑容器与根、映射树、树表：数据单元与 extent 树照抄」（`crates/singlefs-harness/tests/unlink_single_file_publishes.rs:138`）、一批 「一批只重写被截掉那一片下段叶与它的祖先、inode 叶与根（size 改了）与固定点单元」（`crates/singlefs-harness/tests/unlink_single_file_publishes.rs:214`）、最后一批 「最后一批重写 extent 零条目根、inode 零条目根与固定点单元；deleted_inodes 删空取根指针置零、不写它的节点」（`crates/singlefs-harness/tests/unlink_single_file_publishes.rs:260`） 各一份整张角色表 | 真判（3 个单元的文件） | 多片下段叶的文件的角色表没有钉绝对值（见 K1.a 的缺口；我的探针打印了 145 / 288 / 289 / 300 个单元的每一版角色表，没有断言） |

判定：实现 **一致**；用例对 ① 部分、③ 只有一个写点，其余真判。

我的补充（③ 的多写点）：探针 `every_prefix_cut_of_the_first_batch_over_289_units` / `…288…`：289 个单元的文件意图之后的第一批（重写被截掉那片叶、下段根、上段根，共 45 个写）与 288 个单元的第一批（叶边界，41 个写）的录制流，**每一个前缀**（289：1324..=1369 共 46 个，288：1318..=1359 共 42 个）作崩溃镜像：池级 checker 无红；按产品批大小可写挂载续做到队列空；终态（队列空、inode 记录无、数据单元 0、墓碑一条）相同；续做之后 checker 无红。88 个点全绿（release 与 debug 各跑一遍，结果相同，原样输出在 `probe-output.txt` 第 6、7、10 段）。这只切录制流的前缀，不排列没落盘的写，所以不替代崩溃枚举。
推翻 ③ 的现象：任何一个前缀上 checker 红、续做 `Err`、或终态不同。

### K1.d　墓碑容器号与谱系容器号的取号

条款：「容器号从 1 起，下一个号 = 树里最大号 + 1，零根时 1（与已定项 11 里克隆头的类型 1 容器号从 1 起按树内单调分配同一条规则）」（`.claude/kb/decisions/18-块里携带什么信息.md:566`）；谱系 「谱系树根满同样长高，容器号从 1 起，下一个号 = 树里最大号 + 1，零根时 1」（`.claude/kb/decisions/18-块里携带什么信息.md:570`）。
代码（两棵树共用一个函数）：「.map_or(1, |rightmost| rightmost.identity.container + 1);」（`crates/singlefs-core/src/container_tree.rs:453`），出生代取这次发布的 txg 「container_birth: checkpoint_txg,」（`crates/singlefs-core/src/container_tree.rs:459`），出生树取调用方给的树 「birth_tree: tree,」（`crates/singlefs-core/src/container_tree.rs:456`）。
用例：「fn the_first_tombstone_opens_container_number_one_born_in_this_publish() {」（`crates/singlefs-core/src/tombstone_tree.rs:188`）、「fn a_full_open_container_is_closed_and_the_next_one_takes_the_largest_number_plus_one() {」（`crates/singlefs-core/src/tombstone_tree.rs:243`）、「fn the_first_lineage_record_opens_container_number_one_and_the_second_is_appended_to_it() {」（`crates/singlefs-core/src/lineage_tree.rs:132`）；变异行 「三·2 墓碑树：零根时第一个容器号取 0（不是 1）」（`crates/mutations.tsv:2048`）、「三·2 墓碑树：下一个容器号取最大号 + 2」（`crates/mutations.tsv:2050`）、「三·2 第 5 条：容器树零根时新容器号从 0 起」（`crates/mutations.tsv:2102`）。
判定：**一致**。推翻它的现象：树里最大号 + 1 与实现取的号不同（例如回退之后墓碑树随现行那一版，取号读的不是现行那一版的容器）；墓碑树 / 谱系树随现行那一版的取树见 K4.a。

## K2　续做、走空与推抬 F（L3、L4）

### K2.a　挂载续做与会话走空是不是同一个函数

条款：「unlink 一次调用 = 走空 + 意图 + 全部批（与续做共用同一个走空）」（`.claude/kb/decisions/08-核心索引结构.md:463`）。
代码：走空只有一个定义 「pub fn walk_the_deletion_queue_until_it_is_empty<Device: BlockDevice>(」（`crates/singlefs-core/src/transaction.rs:5896`）；会话那一处经 「fn publish_the_queue_until_it_is_empty<Device: BlockDevice>(」（`crates/singlefs-core/src/mounted_session.rs:205`） 调它（「walk_the_deletion_queue_until_it_is_empty(」（`crates/singlefs-core/src/mounted_session.rs:215`）），挂载续做直接调它（「let walk = walk_the_deletion_queue_until_it_is_empty(」（`crates/singlefs-core/src/mount.rs:4347`））。
判定：**一致**（共用的是走空这一个函数）。要说明的一点：被拒时「推抬 F 再试」的外层循环不是共用的，各写了一份——会话的 「fn publish_the_queue_until_it_is_empty_raising_the_floor_for_space<Device: BlockDevice>(」（`crates/singlefs-core/src/mounted_session.rs:536`） 与挂载的 「fn continue_the_deletion_queue_until_it_is_empty<Device: BlockDevice>(」（`crates/singlefs-core/src/mount.rs:4320`），两份里「这次准入已推几次」的计数、清零条件、预算判断是逐行平行的两段代码（见 K2.b），只在错误的包法上分开（会话包成 `UserChangeRefused`，挂载包成 `MountError`）。两份将来各改一处会分叉，今天没有用例同时钉住两份（见 K2.b 的变异）。
推翻它的现象：续做与会话对同一份盘面、同一个空间状况给出不同的发布序列。

### K2.b　「每一次被拒的删除类发布各算一次准入」在续做那一趟里怎么数

用户定案（原话，调度记录同日条目）：「Q-V 预算「甲 每次被拒各算一次（推荐）」」（`records/2026-10-01-删除与并发事务.md:338`）、「挂载续做被空间拒「甲 同样推抬 F 再试（推荐）」」（`records/2026-10-01-删除与并发事务.md:338`）；它们要连着的既有条款：「一次准入最多 B = 4 + 2 k_tol 次发布（k_tol = 2 ⇒ 8），做满仍不够才报 ENOSPC」（`.claude/kb/decisions/16-发布语义.md:43`）。
从这三句推出的应有数法：一次准入 = 一次被拒的发布 + 为它推的几串空发布，合起来不超过 8；同一次发布被拒后推一串、再被拒，推的数累加；这一次发布发成、下一次发布又被拒，是另一次准入，从 0 数。
代码：预算 「pub const PUBLISHES_PER_ADMISSION_AT_MOST: usize = 8;」（`crates/singlefs-core/src/mount.rs:2207`）；判 「let publishes_of_this_admission = publishes_pushed + publishes_of_the_next_raise + 1;」（`crates/singlefs-core/src/mount.rs:2395`） 与 「if publishes_of_this_admission > PUBLISHES_PER_ADMISSION_AT_MOST {」（`crates/singlefs-core/src/mount.rs:2396`）；累加 「publishes_pushed_in_this_admission += raised.publishes.len();」（`crates/singlefs-core/src/mount.rs:4392`）；清零（挂载续做）「if !steps_published.is_empty() {」（`crates/singlefs-core/src/mount.rs:4360`） 后一行置 0，会话那一份在 「if !steps_published.is_empty() {」（`crates/singlefs-core/src/mounted_session.rs:560`）。
手算三条轨迹（没跑）：(1) 走空发成 S1、S2，S3 被拒：这一轮 `steps_published` 非空，计数清 0，再推一串（比如 2 次）→ 计数 2；(2) 下一轮 S3 仍被拒（`steps_published` 空）：计数不清，2 + 下一串 + 1 ≤ 8 才再推；(3) 下一轮 S3 发成、S4 被拒：`steps_published` 非空，清 0，S4 是新的一次准入。三条都与应有数法相符。
判定：**一致**。「有一步落了盘就从 0 数」这句实现员的话就是这段代码，它等价于「每个被拒的发布各算一次」，因为一轮走空里被拒的只可能是它发成的最后一步之后的那一次发布。

用例是不是真判（我的补充，会红的路子）：把会话与挂载续做两处的清零 `publishes_pushed_in_this_admission = 0;` 改成 `+= 0;`（永不清零，变异文件 `research/prompts/m3-unlink-code-r1-sonnet-model/mutant-no-reset-of-the-admission-count.diff`），跑 `admission_raises_the_floor_before_refusing` 里 5 条新用例（`an_unlink_*` 三条、`a_mount_whose_deletion_continuation_*` 两条）：**5 条全绿**（`test result: ok. 5 passed; 0 failed`，`probe-output.txt` 第 8 段）。变异表里这两处也没有对应行（`crates/mutations.tsv` 2127–2129 三行是「不推」「当作走完」「丢掉意图那一版」）。
原因：这 5 条用例每条至多发生一次「落了盘之后又被拒」，没有「同一趟里两次被拒各需要几次空发布」的盘面，清零与否算出的数相同。所以用户这一句「每次被拒各算一次」没有被任何入库用例钉住；也没有人造出过能区分的盘面——我也没造出来（没有量过这样的盘面存不存在）。
推翻上面「没钉住」的现象：造出一个盘面，同一趟走空里两次被拒、各要 ≥ 4 次空发布才放行，不清零的变异在它上面红。

### K2.c　推满仍不够时盘上留下的几版是不是条款允许的中间态

条款：「中间状态必须每一个都是合法的文件系统状态」（`.claude/kb/decisions/08-核心索引结构.md:124`）；挂载那一侧既有的收尾条款：「挂载处推满仍不够时挂载照样做成可写，之后的发布照发布准入判、不够报 ENOSPC，正常卸载照常（D2（RAID 条带策略） 已定项 13 那一格的例外）」（`.claude/kb/decisions/16-发布语义.md:43`）。
代码：
- 会话推满仍不够：「NoSpaceAfterRaisingTheFloor(Box<NoSpaceAfterRaisingTheFloor>),」（`crates/singlefs-core/src/mounted_session.rs:133`），`current` 停在最后一次落盘的那一版（「`current` 停在最后一次落盘的那一版上」（`crates/singlefs-core/src/mounted_session.rs:534`））；
- 挂载推满仍不够：「return Err(walk_failed(refusal));」（`crates/singlefs-core/src/mount.rs:4400`）（`Stopped` 那一臂，整个挂载交回 `MountError::Publish`），已发成的写行、暖机、续做步留在盘上。
用例：会话 「现行那一版是意图那一版：队列留一条、F 没动」（`crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs:263`）；挂载 「挂载写下的 13 次发布（写行、暖机与续做发成的几步）留在盘上；为那一批推的抬 F 一串都没推成，F 没动」（`crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs:458`），并核只读挂载答「没有文件」。

判定分两半：
1. 中间态合法：**一致**。卡住时盘上是「意图 + 若干批 + 为空间推的空发布」，这些发布每一个都是 K1.c 判过的删除类版本（checker 无红、重建相同）；入库的三条会话用例都不跑 checker（前两条在文档里说明了理由：标成已分配的槽只在分配器的位图上、不写记录），挂载那条（`stays_short`）只走只读挂载；所以「卡住那一刻 checker 无红」靠逐版组合推出，没有直接判（推的）。
2. 挂载卡住时整个挂载报错、不做成可写：**规则没说**，并且与既有那一行方向相反。两边整行：
   - 既有条款（挂载处准入的收尾）：「挂载处推满仍不够时挂载照样做成可写，之后的发布照发布准入判、不够报 ENOSPC，正常卸载照常」（上面第一条引文，`.claude/kb/decisions/16-发布语义.md`）；
   - 实现（挂载续做的收尾，用户 2026-10-03 只定了「同样推抬 F 再试」，没有说推满之后怎么收）：「是空间不够、推满仍不够（一次准入的发布数做满、F 已在上限、抬 F 那一串自己也取不到落点或被空间准入拒）：照旧交回最后那一次被拒」（`crates/singlefs-core/src/mount.rs:4317`）。
   「照旧」指续做在 L4 之前遇到空间不够就交回错。背景材料 L4 那一行的「改动」栏写了「挂载交回 `MountError::Publish`」，「压着的条款」栏列的用户原话只有「同样推抬 F 再试」（也就是上面两句）；调度记录同日条目里我没找到说推满之后挂载怎么收的话。
   这一半的后果（推的，没量过）：卡住的挂载返回 `Err`，没有 `Mounted`、没有分配器，调用方手里没有会话可以卸载（卸载抬 F 到现行那一版、不判上限，是既有条款留给「挂载照样做成」那一支的出路）；每次重试再取一个实例号、再发写行与暖机，然后在同一处停。会不会永久卡住，取决于每次重试能不能多发成一步续做（多发成的步是新的非空根，会把「第 4 新的非空根」往前推，F 的上限随之抬高）；F 的上限的定义见 「准入抬 F：min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)」（`.claude/kb/decisions/16-发布语义.md:37`）。这一条是给攻方腿的线索，我没造出这个盘面、没量。
推翻第 1 半的现象：卡住那一刻的镜像上池级 checker 红。推翻第 2 半（「规则没说」）的现象：用户在别处另有一句说推满之后挂载交回错。

### K2.d　`deleted_inodes` 至多一条 key 的拒绝是不是类型化、不归空间不够

条款：「任何一版里 deleted_inodes 至多一条 key；多于一个根装得下那一支拒成类型化错误、不是空间不够」（`.claude/kb/decisions/08-核心索引结构.md:463`）。
代码：多于一个根装得下的那一支是类型化错误 「MoreKeysThanOneRootNodeHolds { keys: usize, capacity: usize },」（`crates/singlefs-core/src/deleted_inodes.rs:31`），在发布路径上包成 「.map_err(PublishError::DeletedInodesQueueRefused)?;」（`crates/singlefs-core/src/transaction.rs:4978`），判「是不是空间不够」时落在 `false` 那一臂（「| PublishError::DeletedInodesQueueRefused(_)」（`crates/singlefs-core/src/mounted_session.rs:198`）），所以会话与挂载续做都不为它推抬 F。「至多一条」这一半是前置条件而不是错误：「发意图之前队列已走空：任何一版的 deleted_inodes 至多一条 key」（`crates/singlefs-core/src/transaction.rs:5770`） 是 `assert!`（调用方违约就 panic），会话在发意图之前先走空，所以走不到。
判定：**一致**。两处细节：(1) 类型化错误那一支在发布路径上走不到（「只在构造函数这一层测——发布路径上至多一条，走不到它」（`crates/singlefs-core/src/deleted_inodes.rs:7`）），只有构造函数一级的单测 「fn a_queue_longer_than_one_root_holds_is_refused_as_a_typed_error() {」（`crates/singlefs-core/src/deleted_inodes.rs:94`） 与变异行 「三·2 deleted_inodes：多于一个根装得下的号多放一个才拒」（`crates/mutations.tsv:2041`）；(2) 「它不归空间不够」那个分类（`refusal_is_short_of_space` 里它在 `false` 臂）没有任何用例或变异行钉着：全仓除 `mounted_session.rs` 的分类臂之外，`DeletedInodesQueueRefused` 只出现在 `transaction.rs`（定义与包装）和 harness 的成员名映射里，变异表里没有含它的行（grep 与原样输出见本报告末尾「附：命令与原样输出」）；把它挪到 `true` 臂，今天没有一条用例会红，因为发布路径造不出这个错误。
推翻它的现象：发布路径上真出现 `DeletedInodesQueueRefused` 而会话为它推了抬 F。

### K2.e　崩溃注入 Continuation 那一段认得出续做的写

代码：「挂载续做：暖机与推的抬 F 之后、交回之前把所选那一版的 `deleted_inodes` 走空的那几步删除发布（所选那一版有删除中的文件时才有）。」（`crates/singlefs-checker-tier/src/crash_injection.rs:277`）、分段规则 「从它起是续做（没崩过的那一次挂载被拒、交不出这个数时 None：写行之后都算暖机）」（`crates/singlefs-checker-tier/src/crash_injection.rs:2194`），续做之前发了几次由 「let publishes_before_the_deletion_queue_continuation = mounted.as_ref().ok().map(|mounted| {」（`crates/singlefs-checker-tier/src/crash_injection.rs:1605`） 数（写行 1 + 暖机 + 准入推的抬 F；续做自己为空间推的抬 F 在续做那一段里）。
用例与变异行：「fn the_writes_after_the_publishes_before_the_continuation_belong_to_the_continuation_phase_and_get_a_second_crash_point(」（`crates/singlefs-checker-tier/src/crash_injection.rs:3442`）；「三·2 Q-J：可写挂载的段表里没有续做（续做那一段摆不到二次崩溃）」（`crates/mutations.tsv:2123`）、「三·2 Q-J：续做从「续做之前发了几次」那个根槽写之后再晚一次才算起（续做第一步的写归暖机）」（`crates/mutations.tsv:2124`）。
判定：**一致**。注意：两条战役用例都在 `Continuation` 上 `continue` 跳过（「这两段短历史里没有 unlink，那一段的分段与摆点由」（`crates/singlefs-checker-tier/tests/crash_injection_writable_mount_after_the_crash.rs:70`）），所以这一段在整条崩溃注入战役里是否真被抽到、抽到之后二次崩溃是否过判器，要等 checker 档整轮跑才知道（我没跑，也不该跑）。已知 ① 的那一格（续做里推的抬 F 不在 `FloorRaisesInTheMountWrites` 里）背景材料已列，不重复。
推翻它的现象：续做的写被归进暖机段，或二次崩溃一个都没摆到续做段。

## K3　准入（L6）

### K3.a　环窗口一式，逐项对条款

条款：「环窗口 = (根环槽数 3 × S + 1) × W，S 是系统配置每区槽数」（`.claude/kb/decisions/28-挂载期承诺量.md:29`）；「W = 24 + Σ_ℓ min(N_ℓ, 2·(K + 26))，K 是删除批大小，N_ℓ 是分配记录树第 ℓ 层两块盘合计罩到的位置数」（`.claude/kb/decisions/28-挂载期承诺量.md:29`）；「删除队列非空时，可写挂载前那一判把环窗口算作可动用」（`.claude/kb/decisions/28-挂载期承诺量.md:29`）。
代码逐项：
- 根环槽数 3 × S + 1：「let roots_in_the_ring_plus_one = ROOT_RING_REGIONS * slots_per_region.count() + 1;」（`crates/singlefs-core/src/admission.rs:394`），`ROOT_RING_REGIONS` 是 「pub const ROOT_RING_REGIONS: u64 = 3;」（`crates/singlefs-format/src/lib.rs:260`），S 由调用方按系统配置每区槽数交进来（发布路径 `pool.parameters.geometry.root_ring_slots_per_region`，「pool.parameters.geometry.root_ring_slots_per_region,」（`crates/singlefs-core/src/transaction.rs:6302`） 在 「admission_reading_before_a_publish_on_a_ring_of(」（`crates/singlefs-core/src/transaction.rs:6295`） 的调用里）；
- W 的三个常数 24 / 2 / 26：「pub const DELETION_PUBLISH_SLOTS_OUTSIDE_THE_ALLOCATION_RECORD_TREE_AT_MOST: u64 = 24;」（`crates/singlefs-core/src/admission.rs:298`）、「pub const DELETION_PUBLISH_ALLOCATION_RECORD_TREE_LEVEL_BOUND_FACTOR: u64 = 2;」（`crates/singlefs-core/src/admission.rs:301`）、「pub const DELETION_PUBLISH_ALLOCATION_RECORD_TREE_LEVEL_BOUND_UNITS_BEYOND_THE_BATCH: u64 = 26;」（`crates/singlefs-core/src/admission.rs:302`）；每层 min：「.map(|positions| (*positions).min(bound_of_one_level))」（`crates/singlefs-core/src/admission.rs:320`）；
- K 取分配器上装的删除开关（产品 144）：「批大小 K 取分配器上装的删除开关」（`crates/singlefs-core/src/admission.rs:357`）；
- N_ℓ 按每块盘单元区在第 ℓ 层罩到的位置数、各盘相加、ℓ 从 0 到根层级 − 1：「let positions_at_this_level = last_slot_of_the_unit_area / span_at_this_level」（`crates/singlefs-core/src/admission.rs:1059`） 所在的循环（`for level in 0..root_level`）；条款写「两块盘合计」，代码对池里全部盘求和（两块时相同，多于两块盘时是推广，条款没写这一格）；
- 「队列非空时可写挂载前那一判算作可动用」：「Some(version) if !version.deleted_inodes.is_empty() => {」（`crates/singlefs-core/src/admission.rs:350`）；写行之后推抬 F 再判是同一份读数（「RingWindowOfTheReading::of_a_writable_mount(version_to_build_on, slots_per_region),」（`crates/singlefs-core/src/admission.rs:1431`））；
- 「每块盘各扣这一份」「只住内存、不进记账行」：窗口只出现在 `AdmissionReading` 的逐盘项里，没有记账行（「只住内存、不进记账行，每次发布按几何现算。」（`crates/singlefs-core/src/admission.rs:381`））。
手算核数（独立于代码）：两块 4 GiB 盘，单元区 [50176, 262144)，分配记录树叶一片 812 槽：层 0 每盘 ⌊262143 ÷ 812⌋ − ⌊50176 ÷ 812⌋ + 1 = 322 − 61 + 1 = 262，两盘 524；层 1 每盘 2，两盘 4；根层级 2。W = 24 + min(524, 340) + min(4, 340) = 368；mkfs 的 S = 8 ⇒ 环窗口 (3 × 8 + 1) × 368 = 25 × 368 = 9 200 槽 = 150 732 800 字节。我的探针在这个几何上打出的「环窗口每盘」是 `BytesOnOneDevice(150732800)`（`probe-output.txt` 第 3 段），与手算一致；入库单测 「fn ring_window_on_two_four_gibibyte_devices_is_twenty_five_times_three_hundred_sixty_eight_slots(」（`crates/singlefs-core/src/admission.rs:1682`） 钉的也是 368 / 9 200。
变异行：「三·2 第 6 条：W 每层不取 min(N_ℓ, 2·(K + 26))」（`crates/mutations.tsv:2059`）、「三·2 第 6 条：环窗口少乘那一格 + 1（(3 × S) × W）」（`crates/mutations.tsv:2060`）、「三·2 第 6 条：环窗口的批大小不取分配器上装的删除开关（恒按产品 144）」（`crates/mutations.tsv:2085`）、「三·2 第 6 条：可写挂载那一判在删除队列空时把环窗口算作可动用（队列判反）」（`crates/mutations.tsv:2086`）、「三·2 第 6 条：可用(d) 不扣环窗口（式子末项漏扣）」（`crates/mutations.tsv:2094`）。
判定：**一致**。推翻它的现象：换一个几何（盘数、盘宽、S）手算的 W 与 `ring_window_on_one_device` 不等。

### K3.b　墓碑预付一式，逐项对条款

条款：「标签 6 = 副本数 × ((2 + H + 1) × 16384 + 41 × n_live)，单位字节、按副本之和记」（`.claude/kb/decisions/03-空间分配.md:52`）；「H 是现行那一版墓碑树的内部层数（零根与只有根时 0），从墓碑树根节点码 2 头现读」（`.claude/kb/decisions/03-空间分配.md:52`）；「41 = ⌈2 × 16384 ÷ 815⌉，每份副本、每个对象先取整再乘 n_live」（`.claude/kb/decisions/03-空间分配.md:52`）；「n_live = 这一版 inode 树里 nlink ≥ 1 的记录数」（`.claude/kb/decisions/03-空间分配.md:52`）。
代码逐项：
- 41 先取整再乘：「DATA_UNIT_BYTES.div_ceil(TOMBSTONE_CONTAINER_RECORDS);」（`crates/singlefs-core/src/admission.rs:237`）（⌈32768 ÷ 815⌉ = 41）、乘 n_live 在后：「TOMBSTONE_PREPAY_BYTES_PER_LIVE_OBJECT_ON_ONE_REPLICA」（`crates/singlefs-core/src/admission.rs:259`） 所在的 `checked_mul(live_objects)`；
- (2 + H + 1) 个槽：「let resident_slots = TOMBSTONE_PREPAY_OPEN_CONTAINER_SLOTS_ON_ONE_REPLICA」（`crates/singlefs-core/src/admission.rs:276`） 加 H 加 1 个根槽（开放容器 2 槽 = 32768 ÷ 16384）；
- 按副本之和记：「.checked_mul(replicas.replicas())」（`crates/singlefs-core/src/admission.rs:289`）；
- n_live 数 nlink ≥ 1：「.filter(|record| record.nlink != NLINK_OF_AN_INODE_BEING_DELETED)」（`crates/singlefs-core/src/admission.rs:229`）；
- 树表里没有 deleted_inodes 条目的版本取 0：「TombstonePrepayBasis::VersionWithoutADeletedInodesTreeTableEntry => {」（`crates/singlefs-core/src/admission.rs:269`） 那一臂；
- 「写入类发布按发布之后的标签判」：「committed_reservation_for_tombstones_after_this_publish(」（`crates/singlefs-core/src/transaction.rs:4106`） 的调用在 「committed_reservation_for_tombstones_after_this_publish(」（`crates/singlefs-core/src/transaction.rs:6298`） 处传给式子那一判，记账行与式子那一判读同一个函数。
数：n_live = 1、H = 0：2 × (3 × 16384 + 41) = 98 386；n_live = 0：98 304；H = 1、n_live = 1：2 × (4 × 16384 + 41) = 131 154。三个数与入库用例（「fn tombstone_prepay_on_a_root_only_tombstone_tree_is_three_slots_plus_forty_one_bytes_per_live_object(」（`crates/singlefs-core/src/admission.rs:1735`）、「fn tombstone_prepay_after_the_tombstone_tree_grows_one_level_counts_one_more_slot() {」（`crates/singlefs-core/src/admission.rs:1753`））及 K1.c ④ 里的 98 386 / 98 304 逐个相等。
判定：数值与各项取法 **一致**。一处读法与条款字面不同（弱）：条款写 H「从墓碑树根节点码 2 头现读」，同一个读法的先例 D28 已定项 4 写「不用内存里另存一份」（「不用内存里另存一份」（`.claude/kb/decisions/28-挂载期承诺量.md:111`））。实现取 H 的地方读的是内存里的形状：「u64::from(self.root_level()).saturating_sub(1)」（`crates/singlefs-core/src/container_tree.rs:191`），标签 6 的调用点 「tombstone_tree_internal_levels: resolved.tombstone_tree.internal_levels(),」（`crates/singlefs-core/src/transaction.rs:4112`），谱系那一项 ckpt_cost 同样读形状（「self.lineage_tree_index_nodes.shape.internal_levels()」（`crates/singlefs-core/src/transaction.rs:2799`），文档自述「形状与根节点头里的层级出自同一次发布或同一次从盘上重建」）。算标签 6 的这一刻是「这次发布之后」那一版，它的根节点还没装出来，没有头可读，只能取形状；形状在挂载时由盘上每个节点头的层级排出，发布时由同一个函数推进，所以两者只会在构造出来的坏形状上不等。校验路径（K1.c ④）从重建出来的墓碑树根单元头里现读层级，不经形状，与记账行逐版相等。
推翻「一致」的现象：任一版里形状的内部层数与那一版墓碑树根节点头的层级 − 1 不等，而标签 6 取了前者。

### K3.c　删除类发布绕开式子、落点那一判照判；落点要的槽数会不会比真写的多出很多

条款：「删除类发布（意图、各批、挂载里的续做批）与管理员回退也不判式子，删除类发布的落点那一判照判」（`.claude/kb/decisions/28-挂载期承诺量.md:36`）；D3 同句 「删除类发布（意图、各批、挂载里的续做批）不判式子那一判，落点那一判照判；写入类发布按发布之后的标签判；环窗口从第一个文件版本那次发布起计」（`.claude/kb/decisions/03-空间分配.md:52`）。
代码：「// 删除类发布（意图、各批、最后一批）不判式子那一判、落点那一判照判（D28（挂载期承诺量） 已定项 1：它们的空间罩在环窗口里）。」（`crates/singlefs-core/src/transaction.rs:6280`） 之下 `demand_is_judged` 对 Intent / Batch / FinalBatch 三臂恒 `false`；落点那一判（`admit_the_units_landing_on_every_device`）按「这次有普通分配」判，不分删除类与否：「publish_has_an_ordinary_allocation(&ordinary_allocation_roles)」（`crates/singlefs-core/src/transaction.rs:6232`）。删除树的节点被归成普通分配：「// 删除那两棵树的节点只在删除类发布里重写：落点那一判照判（普通分配），式子那一判删除类发布整次不判」（`crates/singlefs-core/src/admission.rs:1235`）。
要的槽数的算法（条款没写）：「固定点按 `checkpoint_cost` 计，」（`crates/singlefs-core/src/admission.rs:757`）——普通分配那一半的角色按各自的槽数加上 ckpt_cost（整棵分配记录树的最坏数 + 映射 + 记账 + 谱系一次 COW + 树表），与写入类发布同一口径，对删除类发布也加上谱系那一项（「checkpoint_cost_of_one_lineage_container_cow(lineage_tree_internal_levels).0」（`crates/singlefs-core/src/admission.rs:993`））。
我量的（探针，300 个单元的文件，两块 4 GiB 盘，产品批大小，`probe-output.txt` 第 3 段）：

| 发布 | 这次真写（槽） | 其中普通分配 | 落点那一判要（槽） | 此刻每盘没挡的槽 | 环窗口每盘（槽） |
|---|---|---|---|---|---|
| 意图 txg 5 | 16 | 7 | 549 | 211 269 | 9 200 |
| 第一批 txg 6 | 17 | 6 | 548 | 211 269 | 9 200 |
| 第二批 txg 7 | 17 | 5 | 547 | 211 269 | 9 200 |
| 最后一批 txg 8 | 12 | 2 | 544 | 211 269 | 9 200 |

（「此刻没挡的槽」是删完之后分配器的读数，不是各版当时的；ckpt_cost 542 按各版的上一版现算。）
判定：**一致**（条款只说「照判」）；要的槽数的口径 **规则没说**。量到的事实：落点那一判要的是真写的约 33–45 倍（544–549 对 12–17）。大头是 ckpt_cost 542，其中分配记录树按整棵最坏数计（注释自己算的 「1 + 2 × 264 = 529」（`crates/singlefs-core/src/admission.rs:1021`））；它比环窗口（9 200 槽 / 盘）小一个数量级，环窗口先于它被扣，所以在这个几何上它不会成为误拒的原因；单元区窄的盘上两者的比值会变（窄盘用例里窗口是 700–750 槽），那里的行为由 K2.c 引的几条用例钉。
推翻「在 4 GiB 盘上不是瓶颈」的现象：一个几何上，式子放行（可用 ≥ 0 扣环窗口之后）而落点那一判以「没挡的槽 < 要的槽」拒一次删除类发布。

### K3.d　标签 4 恒 0 与 I-3.4 的读法

条款：「恒 0 的保留位：删除中 inode 没释放的单元本来就在「已分配」里」（`.claude/kb/decisions/05-快照-空间记账机制.md:119`）；「记账标签 4（待删除但意图未完成占用）恒 0，可用空间统计因此已扣除待删占用」（`.claude/kb/invariants.md:148`）。
代码：「pub const PENDING_DELETE_OF_THE_FIRST_VERSION: BytesSummedOverAllReplicas =」（`crates/singlefs-core/src/admission.rs:196`） 恒 0，记账行与准入读数读同一个常量（「待删占用与准入读数读的是同一个常量（`admission`）：一处定义，两处各抄一个 0 会分叉。」（`crates/singlefs-core/src/transaction.rs:7824`））；删除批释放的尾部单元进 defer（「截掉的第三个数据单元经映射释放」（`crates/singlefs-harness/tests/unlink_single_file_publishes.rs:238`）），而「已分配」的读法是 「分配器的占着槽数——仍分配的加上已释放、还在 defer 窗口里的」（`crates/singlefs-core/src/admission.rs:474`），所以可用空间里已扣。用例：K1.c ④ 把标签 4 钉成 0。
判定：**一致**。I-3.4 本身：不变量表标「未实现」（同一行），池级 checker 今天不判它（`crates/singlefs-checker/src/walk.rs` 里没有 `I-3.4`），归三·3；这一批不欠。推翻它的现象：某一版记账行里标签 4 非 0，或删除中的 inode 的单元不在「已分配」里。

## K4　回退、删空与读者（L2、L5）

### K4.a　回退挑树的种类表

条款：「新根的用户可见树（inode 树、extent 树）与 deleted_inodes 的根指针取 R_old 那一版的；墓碑树、谱系树随现行那一版，不取 R_old，回退那次发布写一条谱系记录 (T_rb, T_old)」（`.claude/kb/decisions/23-journal的角色与格式.md:378`）；D8 同句 「deleted_inodes 也取 R_old 那一版的（否则 R_old 之后删掉、随 R_old 的 inode 树回来的对象会被后台回收再收一次」（`.claude/kb/decisions/08-核心索引结构.md:173`）。
应有的种类表：extent、inode、`deleted_inodes` 取 R_old；墓碑树、谱系树、logged_ops、分配记录树、记账树、树表、实例表随现行或现算。
代码：树表条目逐条挑 「|| entry.kind == TREE_KIND_DELETED_INODES」（`crates/singlefs-core/src/mount.rs:5426`）（前两个是 `TREE_KIND_EXTENT`、`TREE_KIND_INODE`），别的种类取 `entry.root`（现行）；队列与单元：「deleted_inodes: target_version.deleted_inodes.clone(),」（`crates/singlefs-core/src/mount.rs:5473`）、「tombstone_containers: current.tombstone_containers.clone(),」（`crates/singlefs-core/src/mount.rs:5474`）、「lineage_containers: current.lineage_containers.clone(),」（`crates/singlefs-core/src/mount.rs:5477`）；单元归属 「TransactionUnit::DeletedInodesRoot => true,」（`crates/singlefs-core/src/mount.rs:5096`） 把 `deleted_inodes` 根单元归「取目标的」，所以复活、释放、逐盘验、映射 key 都按这一组走（「复活、释放、逐盘验与映射 key 都按这一组走：`deleted_inodes` 根指针取 R_old 的而它的单元不复活，新根就指着一个记成已释放的槽。」（`crates/singlefs-core/src/mount.rs:5091`）），不会让新根指着一个已释放的槽。
用例：「fn rolling_back_to_the_intent_takes_its_deleted_inodes_root_and_the_next_mount_continues_it_to_empty(」（`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:432`）；变异行 「三·2 回退：deleted_inodes 随现行那一版」（`crates/mutations.tsv:2082`）、「三·2 回退：树表 deleted_inodes 那一条根指针随现行那一版」（`crates/mutations.tsv:2083`）。
判定：**一致**。推翻它的现象：回退之后的新根里墓碑树或谱系树的根指针不等于现行那一版的（回退那次追加谱系记录除外），或 `deleted_inodes` 的根指针不等于 R_old 那一版的。

### K4.b　谱系记录与标签 6 现算

条款：「记录 (T_rb 8, T_old 8) = 16 字节（T_rb 是回退那次发布的 txg，T_old 是 R_old 的 txg，读作「T_rb 的父是 T_old」）」（`.claude/kb/decisions/18-块里携带什么信息.md:570`）；「记账标签 4、6 按新根这一版现算」（`.claude/kb/decisions/23-journal的角色与格式.md:387`）。
代码：「rollback_target_txg: target_root.checkpoint_txg,」（`crates/singlefs-core/src/mount.rs:5716`）（`rollback_txg` 是这次发布的 txg）；追加走 「match publish_version_appending_a_lineage_record(」（`crates/singlefs-core/src/mount.rs:5721`）；标签 6 按新根这一版的 inode 树现数 「live_objects: live_objects_of_the_inode_tree(&resolved.inode_tree.containers),」（`crates/singlefs-core/src/transaction.rs:4113`），新根的 inode 树取的是 R_old 的（K4.a），所以「R_old 的 inode 树记录数减队列 key 数」那句等价于「nlink ≥ 1 的记录数」（「两种写法是同一个数；这里按 nlink 数，盘上读来的队列与记录对不上时也不会减成负数。」（`crates/singlefs-core/src/admission.rs:223`））。
用例：「(98_304, 98_386),」（`crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs:466`）（删完时 98 304，回退到意图之前那一版后 98 386）；「fn a_rollback_appends_one_lineage_record_that_reads_back_field_by_field() {」（`crates/singlefs-harness/tests/rollback_lineage_record_and_its_checkpoint_cost_term.rs:91`）；变异行 「三·2 第 5 条：谱系记录的 T_old 写成回退那次的 txg」（`crates/mutations.tsv:2097`）、「三·2 第 5 条：回退新根的标签 6 不按 R_old 的 inode 树现算（n_live 恒取 0）」（`crates/mutations.tsv:2093`）。
判定：**一致**。推翻它的现象：回退发布的版本里谱系记录的 (T_rb, T_old) 不是 (新根 txg, R_old 的 txg)，或标签 6 不等于按新根 inode 树现算的值。

### K4.c　第三个成员的判定只看那四样、不看墓碑树与谱系树

条款：「判定 = 树表有条目 ∧ inode 树零条目根（key 区间 `[0, 0]`，照已定项 6）∧ extent 树零条目根（key 区间按位置规定）∧ `deleted_inodes` 根指针置零；不看墓碑树与谱系树」（`.claude/kb/decisions/08-核心索引结构.md:462`）。
代码：「let inode_tree_root_has_no_entry = self.inode_leaf_containers.is_empty();」（`crates/singlefs-core/src/transaction.rs:2807`）、「let extent_tree_root_has_no_entry = self.data_pointers.is_empty();」（`crates/singlefs-core/src/transaction.rs:2808`）、「let deleted_inodes_root_pointer_is_zero = self」（`crates/singlefs-core/src/transaction.rs:2809`） 三个都为真；发布出来的与重建出来的两处同一个判定（「判定与 [`crate::transaction::PoolVersion::of_a_version_of_trees`] 同一个」（`crates/singlefs-core/src/recovery.rs:2422`））。树表有条目：`TransactionOutput` 恒有（文档自述）。
一处等价替换要说明：条款写「extent 树零条目根」，实现用「没有数据指针」。第一版只有 inode 1 在 extent 树里有叶条目、写不进 `NoDataUnit` 条目（「一个单元都不剩（unlink 最后一批）就不写这一条，」（`crates/singlefs-core/src/transaction.rs:6846`）），所以两者等价；若将来别的 inode 有数据，这一句要改（K-U5 已记欠账 C618）。
用例：「fn only_the_version_after_the_final_batch_rebuilds_as_with_trees_and_no_file() {」（`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:165`）（五版里只有最后一批之后是第三个成员）；变异行 「三·2 删空：第三个成员的判定把 deleted_inodes 根指针读反」（`crates/mutations.tsv:2076`）；删完再建 inode 之后归带文件的一版（「建了 inode 之后 inode 树有条目，归带文件的一版」（`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:341`））与条款一致（inode 树不再是零条目根）。
判定：**一致**。推翻它的现象：一版的墓碑树或谱系树有无条目改变了它归哪个成员。

### K4.d　从意图提交起只读挂载、冷走读、模型三处都答「没有文件」

条款：「从意图提交起只读挂载、冷走读、模型读回答「没有文件」」（`.claude/kb/decisions/08-核心索引结构.md:461`）。
三处代码：
- 只读挂载：「record.inode == inode.0 && record.nlink != NLINK_OF_AN_INODE_BEING_DELETED」（`crates/singlefs-core/src/mounted_read.rs:635`）；
- 冷走读：「if inode_record.nlink == NLINK_OF_AN_INODE_BEING_DELETED {」（`crates/singlefs-core/src/recovery.rs:4902`） 返回 `Ok(None)`，在数据单元个数与 inode size 的核对之前，所以中间态（size 已改小、单元数未必对得上）不会被当损坏；
- 模型 / 胶水：「Some(record) if record.nlink == NLINK_OF_AN_INODE_BEING_DELETED => {」（`crates/singlefs-harness/src/model_comparison.rs:173`） 与模型自己的 `ModelVersionContents::live_file()`（只有 `LiveFile` 有读得出的文件）。
用例：「fn from_the_intent_on_the_read_only_mount_and_the_cold_walk_answer_no_file() {」（`crates/singlefs-harness/tests/unlink_single_file_publishes.rs:318`）（只读挂载与冷走读），模型：「fn unlinking_the_first_file_then_reading_back_and_mounting_runs_to_the_end_with_the_model_agreeing()」（`crates/singlefs-harness/tests/unlink_in_histories_judged_by_the_model.rs:25`）；变异行 「三·2 unlink 读者：nlink = 0 的记录照样打开」（`crates/mutations.tsv:2069`）、「三·2 第 8 条：胶水把删除中的那一版（nlink = 0）的数据单元当读得出的文件」（`crates/mutations.tsv:2121`）。
我的补充：冷走读那一支在变异表里**没有对应行**：`awk -F'\t' '$2 ~ /singlefs-core\/src\/recovery.rs/ && $3 ~ /nlink|NLINK_OF_AN_INODE_BEING_DELETED/ {print NR}' crates/mutations.tsv` 零行输出（recovery.rs 上三·2 的变异行只有 1610、2064、2075、2098、2099、2106、2113）。我在草稿副本里把它改成 `if false && …` 重跑上面那条用例：**红**（`from_the_intent_on_the_read_only_mount_and_the_cold_walk_answer_no_file ... FAILED`，`probe-output.txt` 第 9 段），所以用例判得住，缺的是一行变异表。
判定：**一致**。推翻它的现象：任何一处从意图那一版起仍读出文件内容或报损坏。

### K4.e　写入口拒 nlink = 0 的每一条路

条款：「现造记录的入口拒「nlink = 0」与「没有这条记录」」（`.claude/kb/decisions/08-核心索引结构.md:461`）；「写入口拒 nlink = 0 的 inode」（`.claude/kb/decisions/08-核心索引结构.md:461`）。
入口逐个（发布路径里会写 inode 记录的）：
- 覆盖写、顺序写：先过 「let first_file_record = live_record_of_the_first_file(previous)?;」（`crates/singlefs-core/src/transaction.rs:5594`） 与 「let first_file_record = live_record_of_the_first_file(previous)?;」（`crates/singlefs-core/src/transaction.rs:5648`），nlink = 0 报 「return Err(PublishError::WriteToAnInodeBeingDeleted {」（`crates/singlefs-core/src/transaction.rs:5977`），没有记录报 `WriteToAnInodeThatIsNotInTheTree`；
- 第一个文件版本：要求树表 0 条，否则 「FirstFileVersionOnAVersionThatAlreadyHasAFile { tree_table_entries },」（`crates/singlefs-core/src/transaction.rs:5510`）（删完之后的版本树表不是 0 条）；
- unlink 意图：只认 nlink ≠ 0 的记录（「.find(|record| record.inode == inode.0 && record.nlink != NLINK_OF_AN_INODE_BEING_DELETED)」（`crates/singlefs-core/src/transaction.rs:5776`））；unlink 一步：只认 nlink = 0 且在队列里的号；
- 建 inode：号取水位，永远是新号（「let first_new_inode_number = previous.inode_number_watermark();」（`crates/singlefs-core/src/transaction.rs:5709`）），不会碰删除中的号；
- 会话：删空的一版上只放建 inode，别的改动 「return Err(UserChangeRefused::NoFileVersionToChange);」（`crates/singlefs-core/src/mounted_session.rs:390`）。
用例：「fn overwrite_of_an_inode_being_deleted_is_refused_before_any_write() {」（`crates/singlefs-harness/tests/write_entries_refuse_an_inode_being_deleted.rs:32`）、「fn sequential_write_to_an_inode_being_deleted_is_refused_before_any_write() {」（`crates/singlefs-harness/tests/write_entries_refuse_an_inode_being_deleted.rs:68`）（两条在「现造一版把 nlink 改成 0」上判，拒在任何写之前、盘面逐项不变）；变异行 「三·2 写入口：删完的第一个文件报成删除中」（`crates/mutations.tsv:2072`）、「三·2 第 3 条：顺序写不先判写的是不是删除中的 inode」（`crates/mutations.tsv:1615`）。
判定：**一致**。一处过期的注释：这份用例文件头还写着 「意图那次发布今天还写不出来（三·2 停在墓碑容器号那一格）」（`crates/singlefs-harness/tests/write_entries_refuse_an_inode_being_deleted.rs:5`），意图发布现在已经有了；无功能影响。推翻它的现象：某个入口在上一版第一个文件 nlink = 0 时发成了带 nlink = 1 的记录。

## K5　新树与 checker（L7）

### K5.a　根长高与内部条目 120 字节、类型段

条款：「码 2 根装满时长一层——新根 1 个码 2、分出的右半 1 个码 2（右半没有容器身份）」（`.claude/kb/decisions/08-核心索引结构.md:155`）；「内部节点条目 = 分隔 key 8 + 身份引用 26 + 子指针 86 = 120 字节」（`.claude/kb/decisions/08-核心索引结构.md:164`）；「墓碑树 {0, 1}（I-9.23（墓碑树条目身份与子头相符））、谱系树 {0, 5}」（`.claude/kb/decisions/08-核心索引结构.md:164`）；墓碑 / 谱系树自己的 「根满时长高，照 D8（核心索引结构） 已定项 6，实现要做这一步（用户定案 K-R3-3）」（`.claude/kb/decisions/18-块里携带什么信息.md:566`）。
代码（写侧）：
- 长高的形状规则：「nodes.push(1);」（`crates/singlefs-core/src/container_tree.rs:308`） 之后 「levels.push(vec![2]);」（`crates/singlefs-core/src/container_tree.rs:311`）——最右那一层装满就在最右端新开一个只有一个孩子的节点，最高一层（根）也满了再长一层、新根两个孩子（旧根、刚开的右半）；左边的节点一个都不动；
- 条目 120 字节：「writer.assert_position(INODE_INTERNAL_ENTRY, "inode 内部条目");」（`crates/singlefs-core/src/records.rs:227`），布局是 key 8、出生树 8、类型 2、容器号 8、容器出生代 8、子指针 86；
- 孩子是码 2 节点的条目身份引用全 0、类型段 0：「pub const IDENTITY_OF_A_CODE_TWO_CHILD: PackedIdentity = PackedIdentity {」（`crates/singlefs-core/src/container_tree.rs:43`）；孩子是容器的取容器自己的身份：「container.contents.identity,」（`crates/singlefs-core/src/transaction.rs:7330`）；
- 分隔 key：层级 1 取容器号、更高层取那一枝最左容器的号（「let smallest_container_number = entries.first().expect("形状里每个节点至少一个孩子").0;」（`crates/singlefs-core/src/transaction.rs:7350`））；节点头的 key 区间取首末两条条目的分隔 key；
- 旧根长高时留在原位、不被换下：`ContainerTreeNodeOrigin::CarriedFromThePreviousVersion` 与 `identity_in_the_previous_version`（「长高那次旧根上一版是根」（`crates/singlefs-core/src/transaction.rs:7210`））。
代码（读侧，自己核）：「what_index_node: "墓碑树的码 2 节点不是非空、层级不合父层级、或类型段 0 的条目身份引用不全 0、分隔 key 不是孩子的首 key",」（`crates/singlefs-core/src/recovery.rs:2764`），谱系树同款，读出每层每个节点的孩子数排成形状（`ContainerTreeShape::from_children_of_each_node_at_each_level`，不变量不满足交回 `None`）。
用例：「fn the_third_tombstone_container_grows_the_tree_and_keeps_the_old_root_in_place() {」（`crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs:164`）、「fn the_fifth_tombstone_container_grows_the_tree_to_three_levels_and_rebuild_reads_it() {」（`crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs:225`）、「fn a_tree_grown_under_the_capped_fanout_takes_the_next_container_under_the_product_fanout() {」（`crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs:255`）、「fn the_third_rollback_grows_the_lineage_tree_and_the_checkpoint_cost_counts_the_internal_level() {」（`crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs:279`）（长高、从盘上重建读回、池级 checker 全绿）；核心层单测 「fn the_product_fanout_grows_the_root_at_the_hundred_and_thirty_sixth_container() {」（`crates/singlefs-core/src/container_tree.rs:537`）；变异行 「三·2 第 7 条：容器树的码 2 节点装满了还往里挂（产品扇出下第 136 片仍挂在层级 1 的根下）」（`crates/mutations.tsv:2109`） 等（2104–2114 一带共十一行）。
判定：**一致**。附一格「规则没说」：D8 已定项 6 写 inode 树与墓碑树共用长高这一条，但那一句自己说 inode 树「根长高之前今天的代码在 `crates/singlefs-core/src/inode_tree.rs` 的 `MoreLeafContainersThanOneRootNodeHolds` 拒」（「根长高之前今天的代码在」（`.claude/kb/decisions/08-核心索引结构.md:155`）），这一批没改 inode 树的拒绝（`inode_tree.rs` 里该成员原样在，diff 只加了删除一支），条款没有说这一批要不要做。推翻「一致」的现象：长高那次旧根被释放，或孩子是码 2 节点的条目类型段非 0，或形状读回与写出的不等。

### K5.b　只供测试的旋钮会不会漏进落盘

条款：「关闭路径加只供测试的容量旋钮，旋钮不落盘」（`.claude/kb/decisions/18-块里携带什么信息.md:568`）。
旋钮：批大小（`DeletionSwitches.batch_size`，装在分配器上）、墓碑容器条数、谱系容器条数、码 2 节点条数（`ContainerTreeCapacities`，装在写入口上）。自述 「都只住内存、不落盘」（`crates/singlefs-core/src/unlink.rs:23`）。
核对：
1. 没有任何旋钮值被写进单元或根：旋钮只在 `append_record_to_the_open_container` 判「开放容器满没满」「节点满没满」与 `next_unlink_step` 定批大小时被读；字节里只有它们的结果（一片容器装了几条记录、一个节点有几个孩子），这两个数在盘上本来就是自述字段，合法范围内的任何值都过读者。
2. 压小旋钮写出来的树，换回产品旋钮接着写：形状从盘上读回，不靠旋钮（「不从容器数与扇出现算：只供测试的扇出旋钮不落盘」（`crates/singlefs-core/src/container_tree.rs:9`）；用例 「fn a_tree_grown_under_the_capped_fanout_takes_the_next_container_under_the_product_fanout() {」（`crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs:255`），变异行 「三·2 第 7 条：容器树的码 2 节点装满了还往里挂（扇出 2 写出的树换回产品扇出之前就没长高）」（`crates/mutations.tsv:2114`））。
3. 批大小旋钮进了环窗口的 K（K 取分配器上的值）：那是内存里的准入量，不落盘；变异行 「三·2 第 6 条：环窗口的批大小不取分配器上装的删除开关（恒按产品 144）」（`crates/mutations.tsv:2085`） 钉着这一处跟着旋钮走。
4. 入口：带旋钮的挂载入口是 `pub`，不是 `cfg(test)`：「pub fn mount_writable_with_test_only_switches_and_deletion_switches<Device: BlockDevice>(」（`crates/singlefs-core/src/mount.rs:4821`），与 `SpaceAdmission::SkippedByTheTestOnlySwitch` 同一个约定（名字带 test_only，产品入口 `mount_writable` 不接旋钮）。
判定：**一致**。推翻它的现象：压小旋钮写出的盘，换一个不带旋钮的读者读不了或判红；或不带旋钮的产品入口构造出了非产品的旋钮值。

### K5.c　池级 checker 遍历四棵新树用的是不是自己的算法（D13 已定项 5）

条款：「checker 与实现之间只共享一样东西：一份由 kb 的字段表生成的常量模块」（`.claude/kb/decisions/13-验证路线.md:138`），其余「格式解析各写一份、遍历与记账代码交集为空」。
代码：checker 的依赖只有格式常量包：「singlefs-format = { path = "../singlefs-format" }」（`crates/singlefs-checker/Cargo.toml:9`）；新树的遍历是 checker 自己写的 「用 checker 自己的读法，不调 `singlefs_core`（D13（验证路线） 已定项 5）。」（`crates/singlefs-checker/src/walk.rs:1618`），条目偏移、类型段、容器头身份都用 checker 自己的 `read_u16` / `read_u64` 取（`walk_container_tree_root`），记录宽 40 / 16 / 8 是 checker 里写的字面量（「PACKED_TYPE_TOMBSTONE => Some(40),」（`crates/singlefs-checker/src/walk.rs:139`）），不引 `singlefs_format` 的同名常量。
判定：算法独立 **一致**。一格「规则没说」：新加的五个宽（`TOMBSTONE_RECORD_BYTES` 40、`TOMBSTONE_CONTAINER_RECORDS` 815、`LINEAGE_RECORD_BYTES` 16、`LINEAGE_CONTAINER_RECORDS` 2039、`DELETED_INODES_ENTRY_BYTES` 8）在 `singlefs-format` 里没有 `format-const:` 标记（`grep -n 'format-const: \(TOMB\|LINEAGE\|DELETED\)' crates/singlefs-format/src/lib.rs` 零行；kb 里也没有这五个名字的标记，`grep -rn 'format-const: \(TOMBSTONE\|LINEAGE\|DELETED\)' .claude/kb` 零行）。D13 已定项 5 说值「由门禁 code-source-discipline 的 format-constants 格按 kb 里的 format-const 标记绑住，只绑标了的那些」（「只绑标了的那些」（`.claude/kb/decisions/13-验证路线.md:144`）），所以这五个数与 kb 的对应关系今天只由 crate 内的一条单测 「fn deletion_widths_equal_the_field_sums_and_the_container_capacity_they_stand_for() {」（`crates/singlefs-format/src/lib.rs:444`） 与变异行 「三·2 格式常量：墓碑容器条数写成 816」（`crates/mutations.tsv:2065`） 钉着，门禁的 kb 绑定不罩它们。
推翻「算法独立」的现象：checker 包里出现对 `singlefs_core` 的引用。

### K5.d　单元有没有漏算进 I-3.1、I-3.11 那几项统计

代码：引用集合 `walk.references` 有两个来源，I-3.1 / I-3.11 读的是它们的并（「let slots_referenced_by_every_walked_version = slots_referenced_per_device(&walk.references);」（`crates/singlefs-checker/src/walk.rs:7246`））：(1) 树里的指针——容器树的根、根之下的码 2 节点、每片容器经 `read_index_node` / `note_references_of_a_pointer_followed_by_the_walk` 记进去（「self.note_references_of_a_pointer_followed_by_the_walk(&pointer, 1, what);」（`crates/singlefs-checker/src/walk.rs:956`） 在 `read_index_node` 里，「self.note_references_of_a_pointer_followed_by_the_walk(&child, 2, &what);」（`crates/singlefs-checker/src/walk.rs:1678`） 在容器那一支，`walk_container_tree_root` 对类型段 0 的孩子递归到所有层级）；(2) 中央映射条目——每个进映射的单元，容器和节点都在内（「self.note_reference(location, span_in_slots, "映射条目指的单元");」（`crates/singlefs-checker/src/walk.rs:1180`））。`deleted_inodes` 根与墓碑 / 谱系根走同一个 `read_index_node`。
用例：「fn the_pool_checker_finds_no_violation_after_the_intent_and_after_every_batch() {」（`crates/singlefs-harness/tests/unlink_single_file_publishes.rs:300`）；长到层级 2、3 的墓碑树与长高的谱系树上池级 checker 全绿（K5.a 引的四条）；变异行 「三·2 池级 checker：墓碑记录宽登记成 41」（`crates/mutations.tsv:2074`） 只钉记录宽。
我的补充（会红的路子）：(1) 只删树里指针那一路对容器的记录（变异 D）：`unlink_single_file_publishes` 9 条与 `container_trees_grow…` 4 条**全绿**（`probe-output.txt` 第 13 段）——两个来源互为冗余，I-3.1 不会因一个来源漏记而红，这一路只对 I-5.1 的「同一版第二次引用同一个落点」那一格有独占的作用（推的，没单独造变异去验）；(2) 两个来源都删（变异 E）：`container_trees_grow…` 4 条**红**（第 14 段）。所以容器单元确实算进了 I-3.1，且是被两条互相独立的路径算进去的。
一处弱化（checker 自己声明的）：另一条路径 `collect_tree_references`（给 I-3.9 / I-9.14 扫其它候选根的引用集合用）对层级 > 1 的容器树不往下数，如实记成数不全（「根长高之后（层级 > 1）的引用集合这一遍还不往下数，如实记成数不全」（`crates/singlefs-checker/src/walk.rs:3594`））；后果是 I-3.9 在那些根后面的已释放记录上报「部分没判」（「有带已释放标志的记录落在比还引用它的最后一条根晚、又走不完的候选根后面」（`crates/singlefs-checker/src/walk.rs:3696`）），不会假红也不会假绿。产品扇出 135，要 136 次 unlink（或 136 次回退）才走到这一步。
判定：I-3.1 / I-3.11 **一致**（容器单元由两条独立路径计入）；I-3.9 在长高之后的树上弱化，**规则没说**（三·3 之前不欠）。

## K6　模型与随机历史（L8）

### K6.a　模型删除那一臂与实现在哪种历史上会分叉

模型罩的范围（写在代码里的）：只罩 inode 1 一个文件，文件只有一个数据单元：「模型的文件只有一个数据单元，意图之后的下一步就是它」（`crates/singlefs-harness/src/model.rs:125`）；随机历史里写进去的内容最长一个净荷（更长的是被拒的取值，「装不下：载荷容量 + 1。」（`crates/singlefs-harness/src/history.rs:306`）），空内容也切出一个事务（`write_request_split` 里「长度为 0 的请求也切出一个事务」），所以每个第一个文件恰有 1 个数据单元，产品批大小 144 下 unlink = 意图 + 一个最后一批，与模型一致。墓碑树恒只有容器 1（「模型今天只罩墓碑树一片容器」（`crates/singlefs-harness/src/model_comparison.rs:116`））；谱系树的单元不对应模型角色（「模型的回退不记谱系容器与谱系树根的落点与分配代」（`crates/singlefs-harness/src/model_comparison.rs:124`））。
按这些范围推，**可达的分叉**只有一类（推的）：写行之后的发布失败时挂载入口不交回已发成的几版，执行器按「拒之前一个字节都不写」判（「挂载半路报错（写行之后的发布失败）：第 1 件不比分配代，入口没交回写出去的那几次发布；模型按「拒之前一个字节都不写」判。」（`crates/singlefs-harness/src/history.rs:3894`）），模型那一判是 「if *wrote_anything && *publishes_completed == 0 {」（`crates/singlefs-harness/src/model.rs:2094`）。L4 之后挂载续做卡住（K2.c 第 2 半）也交回 `MountError::Publish`，它的理由被映射成发布的理由（「MountError::Publish(failed) | MountError::RaiseFloorSequencePublishFailed(failed) => {」（`crates/singlefs-harness/src/model_comparison.rs:765`）），但那一次挂载已经写了行、暖机与若干续做步，「一个字节都没写」不成立。要走到：历史里有一次 unlink 在会话里走空时推满仍不够（`NoSpaceAfterRaisingTheFloor`，意图已落盘，模型逐版比过），或回退到意图那一版，接着抽到一次可写挂载，而那次续做又推满仍不够。执行器在 unlink 之后的估计里把可写挂载抽到的份额给得不小（10–20 / 100）。
我扫了新种子（探针 `sonnet_probe_seed_scan.rs`，种子基 = 测试周期种子基 + 100 000，**每步 checker 关着**，所以只看模型与执行器的判定与 panic）：
- 第一遍 60 段 / 配置（`probe-output.txt` 第 11 段）与第二遍 400 段 / 配置（第 12 段，种子区间含第一遍）：单元区墙 256 槽（准入判着、关着）、384 槽（判着）各 150 步，复用取样点 4 GiB 30 步，快档比重 4 GiB 60 步；第二遍含 unlink 的段数 396 / 396 / 396 / 238 / 334（共 2 000 段里 1 760 段含 unlink），结局不是 `Completed` 的 **0 段**。
判定：**规则没说**——「挂载续做卡住」时模型该怎么答，条款与模型都没有（它取决于 K2.c 第 2 半的收尾怎么定）。在新种子上没有量到分叉（2 000 段，每步 checker 关着）；我没造出能走到那一类的历史（它要窄盘 + 准入判着 + unlink 被拒 + 之后一次挂载；我没有数这 2 000 段里有没有 unlink 被拒之后再挂载的）。
推翻「没量到分叉」的现象：某个种子上 `HistoryEnding::NewFinding` 带 `WroteBeforeRefusing` / `PublishCount`，且那段历史里有 unlink。

### K6.b　执行器按「有树」挑版本之后，还有没有一种版本被比较链跳过

已经按「有树」挑的地方（不跳）：挂载写行与暖机的分配记录比较（「写行与暖机按「有树」挑、不按「带文件」挑：有树而文件集合为空的一版」（`crates/singlefs-harness/src/history.rs:3727`），变异行 「三·2 后续 1：执行器比挂载的分配记录时写行与暖机照旧按「带文件」挑（删完的一版上那几次被跳过）」（`crates/mutations.tsv:2125`））、`observed_root_of_pool_version*`（「PoolVersion::WithFile(output) | PoolVersion::WithTreesAndNoFile(output) => {」（`crates/singlefs-harness/src/model_comparison.rs:446`））、会话推的抬 F 与续做步按 txg 交错比（`floor_raises_and_versions_in_txg_order`）。
**仍按「带文件」挑、第三个成员（有树而文件集合为空）被跳过的**：用户那一类的回退与抬 F——「let PoolVersion::WithFile(current) = current else {」（`crates/singlefs-harness/src/history.rs:4179`）（`apply_rollback_while_mounted`）与 「let PoolVersion::WithFile(current) = current else {」（`crates/singlefs-harness/src/history.rs:4312`）（`apply_raise_rollback_floor`）在会话停在 `WithTreesAndNoFile` 时返回 `not_applicable(CurrentVersionWithoutFile)`，这个前提的名字叫「现行版本树表 0 条」（「MissingPrecondition::CurrentVersionWithoutFile => "现行版本树表 0 条",」（`crates/singlefs-harness/src/history.rs:1350`）），对第三个成员说的不是实话。模型那一侧也配套不答：回退的答案要先过 「.is_a_version_with_a_file_or_a_file_being_deleted()」（`crates/singlefs-harness/src/model.rs:1694`），`TreesAndNoFile` 在那里为 false，答 `NotModeled`。
影响（推的）：删完第一个文件之后、建 inode 之前，那一版上的回退 / 抬 F 不被对拍；一旦建过 inode，现行那一版归 `WithFile`（`inode_record` 为 `None`），回退与抬 F 照比，回退目标取删前的版本（文件复活那一支）是可达且被比的。比重表给「删完之后」的状态 15 + 15（快档，回退与抬 F）、25 + 5（复用）份，建 inode 占 40：前几步有一部分抽中后记成 `not_applicable`。所以被跳过的只是一个很窄的窗口。
判定：这一格的「跳过」**一致**于模型与执行器各自写下的范围（两边配套），但前提的名字写错；没有条款要求从第三个成员回退必须被对拍。推翻「窗口很窄」的现象：快档统计里 `RollBackWhileMounted` 的 `not_applicable` 因 `CurrentVersionWithoutFile` 的次数在含 unlink 的历史里占大头。

### K6.c　建 inode 只接在第一个文件删完之后，比重表换表会不会让别的组的旧钉值悄悄失效

事实（探针 `sonnet_probe_weight_tables.rs`，只生成不执行，旧表 = 新表去掉 unlink、把份数还给可写挂载、去掉「删完之后」那张表；种子区间与步数照 `random_histories.rs` 的常量；`probe-weight-tables-output.txt`）：

| 取样点 | 段数 × 步数 | 含 unlink 的段 | 与旧表生成的历史不同的段 | 第一处不同的步 |
|---|---|---|---|---|
| 快档 BROAD | 96 × 30 | 55 | 55 | 1 到 28，中位约 13 |
| 大档默认 BROAD | 8 × 60 | 7 | 7 | 3 到 37 |
| 偏向抬 F 之后复用 | 48 × 30 | 26 | 26 | 0 到 29 |
| 单元区墙（TOWARD_THE_UNIT_AREA_WALL） | 32 × 150 | 32 | 32 | 0 到 109 |
| 种子 4000000045（复用比重 60 步） | 1 | 0 | 0 | 没有 unlink |
| 种子 4000000001（复用比重 60 步） | 1 | 1 | 1 | 第 54 步，晚于钉住的第 30 步 |

读法：换表的结构性后果与代码里的自述相符（unlink 紧挨在可写挂载之后切 3 份、和仍为 100，先前的各格边界不动）；但是哪个种子落在那 3 份里，之后的几步按「删完之后」的估计抽，所以改过的取样点里 57%（快档）、54%（复用）、100%（单元区墙）的历史换了样子。没有改比重表的组（回退、原分配记录墙）历史逐项不变。
两条被 Q-T 改钉的种子用例（4000000045 第 27 步、4000000001 第 30 步）的历史**没有**被换表改变（第一处不同要么没有、要么在第 54 步），它们的钉值是被「环窗口补足把小盘放大」改的，不是被换表改的。
这些取样点背着的「这条变异必须红」：我把变异表里「必须红」的测试依赖这几个取样点的 20 行（快档 11 行、复用 1 行、回退 1 行、单元区墙 7 行；`reproduction_line` 一类只核复现行、不依赖历史内容的不算）逐行套到草稿副本上、跑它点名的测试（`mutation-rows-result.txt`，release）：
- **红（该红的红了）**：17 行（含新行 2136）；
- **没红**：行 147（`allocator.rs` 把「每块盘上都没有」报成「小盘写满」，必须红的是改名后的 `unit_area_wall_sampling_…_runs_to_the_end_without_reaching_the_placement_wall`；那条用例现在钉 `(32, 0, 0, 1141)`，历史一次都没走到落点墙，变异的报错路径没人走，用例绿；原始输出 `mutation-row-logs/row147.log`，汇总 `mutation-rows-result.txt`）；行 319（`mount.rs`，回退行树 ID 水位，偏向回退的取样点）；
- **没跑完**：行 1327（它跑整档 `--include-ignored`，我跑了 11 分钟停了）。
行 147 与背景材料「已知」的 478、481 同一类（Q-T 改钉后取样点不敏感），但背景材料没列 147；行 319 属回退取样点，该组比重没换表，是否本批之前就不红我没有量（没有改动前的树可比），不归到这一批。
判定：别的组「旧钉值」里数值型的（Q-T 的四条）都已按今天的值重钉、没有悄悄失效；**悄悄失效的是变异行 147 的判别力**（用例绿、变异没人抓）。推翻它的现象：行 147 套上去之后改名后的那条用例红。

### K6.d　Q-T 改钉后的 4 条用例名与断言说的是不是同一件事

| 用例 | 名字说 | 断言钉 | 是否同一件事 |
|---|---|---|---|
| 「fn unit_area_wall_sampling_on_small_devices_with_the_space_admission_skipped_runs_to_the_end_without_reaching_the_placement_wall(」（`crates/singlefs-harness/tests/random_histories.rs:477`） | 32 段跑到底、没到落点墙 | 没有新发现；「(32, 0, 0, 1141),」（`crates/singlefs-harness/tests/random_histories.rs:517`） | 一致 |
| 「fn eighteen_empty_overwrites_on_240_slot_devices_with_the_admission_skipped_leave_room_for_a_writable_mount_with_one_warm_up(」（`crates/singlefs-harness/tests/random_histories.rs:727`） | 18 次空覆盖写后可写挂载有位置、一次暖机 | 挂载 `Applied`、实例 2、`publishes: 2`、直接挂载取号 2 暖机 1 | 一致 |
| 「fn seed_4000000045_on_240_slot_devices_with_the_admission_skipped_raises_the_floor_to_19_reclaiming_135_placements(」（`crates/singlefs-harness/tests/random_histories.rs:1614`） | F 抬到 19、回收 135 个落点 | `RaisedFloor { new_floor: 19, publishes: 3, reclaimed_placements: 135 }` | 一致 |
| 「fn seed_4000000001_on_240_slot_devices_with_the_admission_skipped_raises_the_floor_to_7_in_two_publishes(」（`crates/singlefs-harness/tests/random_histories.rs:1632`） | F 抬到 7、两次空发布 | `RaisedFloor { new_floor: 7, publishes: 2, reclaimed_placements: 0 }` | 一致 |

四条名字与断言逐项对得上。两处要说明：(1) 第一条的「判别力」自述 「判别力：这一段生成的历史或落点规则变了（比重里 unlink 那一格改成 0），这几个数跟着变。」（`crates/singlefs-harness/tests/random_histories.rs:474`） 对「生成的历史变了」成立（变异行 2136 红），对「落点规则变了」不成立——行 147 就是落点规则被改而这几个数不动（K6.c）；(2) 四条改名之后，它们原先作为判别力的几行变异（147、478、481）不再红，落点墙那一格只剩 「落点墙那一格只靠写死的那 5 条用例罩着」（`crates/singlefs-harness/tests/random_histories.rs:472`），用户 2026-10-03 已按「乙 照现值改钉并改名」定，这里只记结果。
判定：**一致**。推翻它的现象：某条用例的名字与它的断言说的不是同一件事（我逐条核过，没有）。

## 判定一览

判定三种：一致 / 冲突 / 规则没说。「用例」一列是我另判的入库用例是不是真判那一格（真判 / 部分 / 无）。

| 格 | 判定 | 用例 | 一句话 |
|---|---|---|---|
| K1.a 只改尾叶（K-R3-1） | 一致 | 部分 | 代码与条款逐项相符；纯函数单测罩着计划，真发布路径上多片下段叶的字节级没有入库用例，`<` 改 `<=` 的变异存活（探针红） |
| K1.a 五条硬要求第 3 条（崩溃重放） | 规则没说 | 无 | 逐写点枚举属三·4；作业点三·2 的验收栏没点名 |
| K1.b 最后一批三件事同一次发布 | 一致 | 真判 | 删记录、删 key、释放单元在同一个 `FinalBatch` 里算、同一个 txg |
| K1.c ① 计划相等 | 一致 | 部分 | 比较的逻辑内容里数据单元只比个数 |
| K1.c ①b 两份盘逐字节相同 | 一致 | 真判 | 判的是不确定性 |
| K1.c ② 删完续做不发写 | 一致 | 真判 | API 层判；挂载层只钉反方向 |
| K1.c ③ 每个写点截断 | 一致 | 部分 | 入库用例一个写点；我的探针补了 88 个前缀点，全绿 |
| K1.c ④ 标签 4、6 现算 | 一致 | 真判 | 用例自己重算，不共用代码；只判发布出来的五版 |
| K1.c 重写角色集合钉绝对值 | 一致 | 部分 | 3 单元文件三张表；多片叶的没有 |
| K1.d 墓碑 / 谱系容器号 | 一致 | 真判 | 最大号 + 1，零根 1，两棵树共用一个函数 |
| K2.a 续做与走空同一个函数 | 一致 | 部分 | 走空函数共用；「被拒推抬 F」外层循环各写一份 |
| K2.b 每次被拒各算一次准入 | 一致 | 无 | 清零条件符合；去掉清零的变异 5 条新用例全绿，变异表无行 |
| K2.c 推满仍不够的中间态 | 一致 | 部分 | 每一版都是合法的删除类版本；卡住那一刻 checker 无红靠组合推出 |
| K2.c 挂载卡住整个挂载报错 | 规则没说 | 真判 | 与 D16 准入行「挂载照样做成可写」方向相反，用户没定这半句；后果没量 |
| K2.d 至多一条 key 的拒绝 | 一致 | 部分 | 类型化错误只在构造函数一级测；「不归空间不够」的分类无用例无变异行 |
| K2.e Continuation 段 | 一致 | 部分 | 分段规则有单测与两行变异；战役用例里该段被 `continue` 跳过，真抽到要等 checker 档 |
| K3.a 环窗口式 | 一致 | 真判 | 手算 368 / 9 200 槽与代码、探针、单测三处相等 |
| K3.b 墓碑预付式 | 一致 | 真判 | 98 386 / 98 304 / 131 154 逐个相等；H 取形状而非根头（数值等价，读法字面弱） |
| K3.c 绕开式子、落点照判 | 一致 | 部分 | 条款只说照判；要的槽数口径规则没说，量到约为真写的 33–45 倍，在 4 GiB 盘上不是瓶颈 |
| K3.d 标签 4 恒 0 / I-3.4 | 一致 | 真判 | I-3.4 checker 未实现，归三·3 |
| K4.a 回退挑树 | 一致 | 真判 | extent、inode、`deleted_inodes` 取 R_old，别的随现行 |
| K4.b 谱系记录与标签 6 | 一致 | 真判 | (T_rb, T_old) 与 98 304 → 98 386 |
| K4.c 第三个成员的判定 | 一致 | 真判 | 四样；extent 零条目根用「没有数据指针」等价替换 |
| K4.d 三处答「没有文件」 | 一致 | 真判 | 冷走读那一支用例判得住（我的变异红）但变异表无对应行 |
| K4.e 写入口拒 nlink = 0 | 一致 | 真判 | 覆盖写、顺序写、第一个文件版本、意图、建 inode、会话各一条路都核了 |
| K5.a 根长高与 120 字节、类型段 | 一致 | 真判 | inode 树的长高这一批没做，条款没说本批含不含 |
| K5.b 旋钮不落盘 | 一致 | 真判 | 压小旋钮写出的树换回产品旋钮接着写有用例与变异行 |
| K5.c checker 算法独立 | 一致 | 部分 | 算法独立；新加五个宽没有 `format-const` 标记，门禁的 kb 绑定罩不到 |
| K5.d I-3.1 / I-3.11 统计 | 一致 | 部分 | 容器单元由树指针与映射条目两条独立路径计入（删一路不红、两路都删红）；I-3.9 对长高后的树弱化（自己声明） |
| K6.a 模型与实现的分叉 | 规则没说 | 无 | 范围内没量到分叉（新种子 5 配置 × 400 段）；挂载续做卡住时模型怎么答没有条款 |
| K6.b 被比较链跳过的版本 | 一致 | 部分 | 第三个成员上的用户回退 / 抬 F 被跳过，前提的名字写成「树表 0 条」 |
| K6.c 换表让旧钉值失效 | 一致 | 部分 | 数值型钉值已重钉；变异行 147 判别力悄悄失效 |
| K6.d Q-T 四条名字与断言 | 一致 | 真判 | 四条逐项对得上 |

冲突：没有。规则没说 6 格（K1.a-崩溃重放、K2.c-挂载卡住、K3.c-槽数口径内含、K5.a 内含的 inode 树长高、K5.c 内含的常量绑定、K6.a），其余 一致。

## 没做什么

- 没出判决、不替主 agent 采纳；「推的」「没量」「复核不了」都在各格里写明。
- checker 档一律没跑：`singlefs-checker-tier` 的测试、层 0、整表变异、QEMU、形式化；`crates/mutations.tsv` 整表没跑（只逐行套了 20 行，见 K6.c，和 K1.a / K2.b / K4.d 各一条自己的变异）。
- 没读攻方腿的任何东西。没碰主工作区里的任何文件；所有编译与变异在 `/tmp/claude-1000/m3-unlink-code-r1-sonnet/work/`（冻结副本的拷贝）里做，每条变异跑完还原，收尾时 `diff -rq` 比过只多我加的探针文件。
- 没验的：K2.c 第 2 半的后果（卡住的挂载能不能靠多次重试走出来）；K3.c 在窄盘上的槽数；K6.a 的「挂载续做卡住」历史没造出来；K6.c 里行 319 是否本批之前就不红；行 1327 没跑完。
- 没改任何 `crates/` 文件、没改 kb。
- 编译目录与草稿副本（`/tmp/claude-1000/m3-unlink-code-r1-sonnet/target`、`/tmp/claude-1000/m3-unlink-code-r1-sonnet/target-release`、`/tmp/claude-1000/m3-unlink-code-r1-sonnet/work`）交回前已删；复现要的东西都在模型目录里（探针 `.rs`、变异 diff，命令见附二），再从冻结副本拷一份即可。

## 附：命令与原样输出

### 附一　K2.d、K4.d、K5.c、K2.e 用到的 grep / awk（在冻结副本根上跑，输出原样）

```
$ cd /tmp/claude-1000/m3-unlink-code-r1/tree && grep -rn "DeletedInodesQueueRefused" --include=*.rs crates | grep -v "^crates/singlefs-core/src/deleted_inodes.rs"
crates/singlefs-core/src/mounted_session.rs:198:        | PublishError::DeletedInodesQueueRefused(_)
crates/singlefs-core/src/transaction.rs:3724:    DeletedInodesQueueRefused(DeletedInodesQueueRefusal),
crates/singlefs-core/src/transaction.rs:4978:            .map_err(PublishError::DeletedInodesQueueRefused)?;
crates/singlefs-core/src/transaction.rs:7422:/// 后一样在 `ordinary_allocation_part` 里已经拒成 `DeletedInodesQueueRefused`。
crates/singlefs-harness/src/history.rs:2492:        PublishError::DeletedInodesQueueRefused(_) => "DeletedInodesQueueRefused",
crates/singlefs-harness/src/model_comparison.rs:681:        | PublishError::DeletedInodesQueueRefused(_)
$ grep -c "DeletedInodesQueueRefused" crates/mutations.tsv
0
$ awk -F"\t" '$2 ~ /singlefs-core\/src\/recovery.rs/ && $3 ~ /nlink|NLINK_OF_AN_INODE_BEING_DELETED/ {print NR}' crates/mutations.tsv | wc -l
0
$ grep -n "format-const: \(TOMB\|LINEAGE\|DELETED\)" crates/singlefs-format/src/lib.rs; echo rc=$?
rc=1
$ cd /home/fy5090/code/singlefs && grep -rn "format-const: \(TOMBSTONE\|LINEAGE\|DELETED\)" .claude/kb; echo rc=$?
rc=1
$ grep -n "I-3.4" crates/singlefs-checker/src/walk.rs | wc -l
0
$ grep -rc "unlink\|Unlink" crates/singlefs-checker-tier/tests | grep -v ":0$"
crates/singlefs-checker-tier/tests/crash_injection_writable_mount_after_the_crash.rs:1
```

### 附二　模型目录 `research/prompts/m3-unlink-code-r1-sonnet-model/`（sha256 现算）

```
e6788bd392a825b9e3ee90c03ffbcb1dd7b7b36415eaebd3cd55ca49b597cbd4  mutant-checker-container-reference.diff
3b62d70eb35ce76f3a2f753b488dae9b385ef231fdb00d53d0ae0880f327276a  mutant-cold-walk-ignores-nlink-zero.diff
57ac239c5c7fd8a973f33fd613b9fe4f89b83081b676ed223c1182c30a7d2f15  mutant-is-entirely-kept.diff
89540973e19ef1f4bd1b86aa283fff425740980e05861d5524d2e308834cd677  mutant-no-reset-of-the-admission-count.diff
9f5c49eee4a279b68283184ced0da29ef6de6c0beb2dc0a417540a6c177f56be  mutation-rows-result.txt
e677b450f1b62cea4c39447b4e28e0e2034a253f149eeb327ce63b7ec11e3e20  probe-output.txt
bde9e8c5d39fbed4bbae430d8e943de368e06088ed31549d3df2bba550667ebd  probe-weight-tables-output.txt
d417e0be23c5699f1879526c1b3c36dc2d163432a3312c3fb62822af08edf3c0  run_rows.py
ba345d37dba283a83e2bc8929d6c3de2bdd0967953f8861bf79e17256f023457  sonnet_probe_multi_leaf_unlink.rs
b997d83a651ff869fd7567e4b2528c754063a4adab714ad2b216ab4ead6179b5  sonnet_probe_seed_scan.rs
1feefc528c31e7ebbe51996a187e779dd844873019038f2bbf93d9f2acc82da4  sonnet_probe_weight_tables.rs
```
另有目录 `mutation-row-logs/`（20 行变异的原始输出，`row<行号>.log`）。
- `sonnet_probe_multi_leaf_unlink.rs`：K1、K3 的探针（多片下段叶 unlink、每个写点前缀截断、落点要的槽数 vs 真写的槽数）；
- `sonnet_probe_weight_tables.rs`、`probe-weight-tables-output.txt`：K6.c 的换表差分；
- `sonnet_probe_seed_scan.rs`：K6.a 的新种子扫描；
- `run_rows.py`、`mutation-rows-result.txt`、`mutation-row-logs/`：K6.c 的 20 行变异表逐行套用；
- `mutant-*.diff`：四个我加的变异（`is_entirely_kept` 的 `<` 改 `<=`；会话与挂载续做的清零改成 `+= 0`；冷走读里 nlink = 0 的那一句改成 `if false && …`；checker 容器单元的两个引用来源）；
- `probe-output.txt`：以上全部命令的原样输出，按段编号（第 1 到 14 段）。
复现（探针都在草稿副本里，把探针 `.rs` 放进副本的 `crates/singlefs-harness/tests/` 即可）：`CARGO_TARGET_DIR=<草稿>/target-release nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 10 cargo test --release --offline -p singlefs-harness --test <探针名> -- --nocapture`；会话里另有一条在跑时设 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON`，上限超过峰值表 2 倍时设 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON`。

### 附三　可入变异表的三行（给主 agent，零轮，我没有写进 `crates/mutations.tsv`）

1. `crates/singlefs-core/src/unlink.rs` 的 `last_data_unit < data_units_kept` → `last_data_unit <= data_units_kept`；要它红需要一条入库的多片下段叶用例（k = 143 或 287 的删除批，可由我的探针 `product_batches_over_287_units` 改成入库用例）；
2. 会话与挂载续做里 `publishes_pushed_in_this_admission = 0;` → `+= 0;`；要它红需要造出一趟走空里两次被拒、各要 ≥ 4 次空发布的盘面，我没造出；
3. `crates/singlefs-core/src/recovery.rs` 冷走读里 `if inode_record.nlink == NLINK_OF_AN_INODE_BEING_DELETED {` → `if false && …`；入库用例 `from_the_intent_on_the_read_only_mount_and_the_cold_walk_answer_no_file` 已经能红，只缺一行表。
另：变异行 147 现在判别不了（K6.c），需要在落点墙那一格改指那 5 条写死的用例。

## cite-check 末行

`python3 research/scripts/cite-check.py research/prompts/m3-unlink-code-r1-sonnet-output.md` 的末行（默认根 = 仓根；另用 `--root /tmp/claude-1000/m3-unlink-code-r1/tree --root <仓根> --background research/prompts/_m3-unlink-code-r1-background.md` 跑过一遍，同为 259 / 259）：

```
  ✓ 核了 259 处引文，对上 259 处，没判 0 处（逐处列在上面）
```
