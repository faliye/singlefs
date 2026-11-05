# m4-admission-delta-r3 正推腿报告（Sonnet，2026-11-05）

腿：三方推论正推腿，第三轮（最终轮）。逐格正推正文 `research/prompts/_m4-admission-delta-r3-body.md` 第四节格一至格五；背景材料 `research/prompts/_m4-admission-delta-r3-background.md`。前两轮判决 `research/prompts/m4-admission-delta-r1-main-verification.md`、`research/prompts/m4-admission-delta-r2-main-verification.md` 当线索读，其中的数一律回到任务日志或产物再核。不下「选哪条」的结论。

约定：
- 代码行号取冻结副本 `/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/`（下称冻结副本）；引文写绝对路径，表里只写文件名加行号的指冻结副本里的同名文件。kb 条款写 kb 文件名加那份文件自己的行号。
- 标注：「读」= 读冻结副本源码或 kb 所得，可 `grep -nF` 复核；「推」= 由读到的事实推的，没跑；「任务 N 读数」= 我直接读 `/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-N.attempt-1.log` 的原样行（任务服务此刻连不上，`task-server/task log` 退 1，日志文件在盘上）；「E199 产物行」= `research/results/e199-transaction-output-copy-2026-11-05.out` 的行，那是 E199 第一段的纯算术计数模型（E199 页写明「第一段全是公式的算术」），不是实现里量的。
- 没写模型，没起脚本，没交任务，没有文件要报 sha256。要主 agent 跑的命令在「要主 agent 跑的命令」一节。
- 符号：W = 这次发布写的数据单元数，U = 这一版数据单元总数，M = `mapped_units` 项数，N_cm = 中央映射树节点数，A = 记账行数（含台阶行），S = 单元区槽数，R = 分配记录总数，N_a = 分配记录树里装着记录的节点数，C_t、C_r、C_l = 墓碑、退役、谱系容器数，F_n = 第一个文件之外有条目的文件数，S_c = 这次挂载开过的聚簇段数，P = 第一对空槽之下被占的槽对数，r = 固定点循环进的次数，h = 树高，Δ = 这一次进出的 key、记录或角色数。
- 「准入之前」按正文第一节读成两道准入（落点判 `transaction.rs:12523`、式子判 `:12621`）都判完之前。B6 把「哪一道算准入」换了位置，下面逐处写明按哪种读法。

## 先说一件会决定后面几格怎么读的事：B6、B7 的字面都没点名「谁来算 ckpt_cost 的根头与记账树」，而两条都绕不开它

ckpt_cost 在一次发布里算两遍：落点判的参数（`transaction.rs:12523` 之后那几行里的 `checkpoint_cost_of_the_version_to_build_on_with_node_capacities`）与式子判的读数（`admission.rs:1744` → `:1711` → `:1211`）。它读四个根头（经 `unit()` 线性找，「.find(|unit| unit.identity == identity)」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:4351`），调用点 `transaction.rs:4404`、`:4422`），又在 `admission.rs:1339`、`:1349`、`:1358` 全量规划记账树。所以任何一条要在规划之前拿到「这次要写多少」的候选，自己的上界算式先要用到这两处；B6 的上界自己就要先做 B7 的「根头按下标取」与「ckpt_cost 记账树按差量」两件（见格一 1.2）。

## 格一　B6 的上界与回落

### 1.1 B6 在冻结副本上具体改哪几步

B6 的字面（背景材料第三节表）：准入之前先由差量算一个这一次分配量的上界，放得下才走快路，放行之后固定点、记录条数、合取 7 与今天一字不改，放不下整条走今天的路。落到冻结副本上：

| 步 | 文件:行 | 改什么 |
|---|---|---|
| 加一道「快路闸」 | `transaction.rs:12489`（`judge_the_publish_before_any_read_or_write` 开头，12501 的取盘身份之后、12510 之前） | 闸里算四组上界（见 1.3），两道判都放得下才置「快」。闸本身不拒绝，只分快慢：放不下就整条走今天的代码 |
| 快路跳过精确的普通半与落点判 | 「let ordinary_allocation_roles = plan」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12510`）到「admit_the_units_landing_on_every_device(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12523`）这一段（含 12513 起的 `units_landing_is_judged`） | 快路上不调这一遍 `ordinary_allocation_part`；`resolve` 里每一轮仍调（「let ordinary_part = self.ordinary_allocation_part(previous, trees, deletion_and_lineage)?;」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:8729`）），所以快路只省固定点之外的那一遍 |
| 式子判的位置 | 「admit_on_every_device(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12621`）那一段（12595–12640） | 两种写法，见 1.2：保留精确判（读法乙）或换成断言「精确需求 ≤ 快路用的上界」（读法甲，要改 D28 两句） |
| 新的上界函数 | `admission.rs` 里 `units_of_a_publish_to_land`（1025）旁边新加；记账树一项的闭式另起一个函数，不动 1325 起的精确函数 | 纯函数，只读 `plan`、`previous` 的存下来的数、分配器计数；见 1.3 每一项的式子 |
| 闸要的两件前置（B7 的两件） | 根头：`transaction.rs:4348–4351`、`:4404`、`:4422`；聚簇段求和：`allocator.rs:574–578` | 根头改按装 `units` 时记下的下标取（照墓碑、退役两个根已有的做法，`transaction.rs:4273`、`:4285`）；聚簇段内没挡的槽对数加增量计数。不改的话闸每次发布仍走 O(U) 的找根头与 O(S_c) 的求和 |
| 另一份固定点不在闸里 | 「for _round in 0..ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT {」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:1907`）是树表 0 条那一版上写行的固定点（`:1876` 起），自带 `:1938` 的分配器克隆 | B6 的闸只装在 `:12489` 的主路径上；写行那一份不在 B6 字面里 |
| 条款 | `.claude/kb/decisions/28-挂载期承诺量.md:35` 与 `:38`；D8 已定项 16 射程（`.claude/kb/decisions/08-核心索引结构.md:516`） | 读法甲改 D28 两句；两种读法都要用户确认「准入之后」免责的口径（1.5） |

读：这四个改点之外，快路放行之后的 `settle_the_allocation_record_tree`（「let settled = settle_the_allocation_record_tree(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12536`））、记录条数（12554）、合取 7（12577）一个字不改，所以固定点每轮的全部代价（格三表里「每轮」那几行）在快路上原样在。

### 1.2 改完以后，两道准入之前还剩哪几步随总数涨

按读法分开写，因为 B6 字面没说式子判降不降成断言。

读法乙（式子判保留精确，位置不动）：式子判的读数与需求都要固定点走完才有（需求 = `settled.resolved.rewritten_roles` 的跨度和，读读数要 ckpt_cost 精确值与 `resolved` 里的墓碑退役层数）。这一读法下快路只省 12510 那一遍普通半。E199 产物行给的 G0（见格五）上，那一遍是 5 遍里的 1 遍。其余格三表里的每一行都还在两道之间。B6 在这一读法下不满足「两道都判完之前不随总数涨」，我推的结论是它什么都没改变，只少了一遍。

读法甲（快路上式子判换成断言，落点判与式子判都由闸按上界判）：「准入」就是闸。闸里还剩（读）：
- 找根头：ckpt_cost 的四次 `unit()`（`transaction.rs:4351`），随 U 涨；闸里只算一遍 ckpt_cost 的上界版，但不改根头下标仍是 4 次线性找。
- 聚簇段求和：`allocator.rs:578` 的「let pairs_inside_the_cluster_segments: u64 = cluster_segments」逐段 `map` 求和，由 `admission.rs:992` 的 `of_every_device_of_the_allocator` 每块盘调一次，随 S_c 涨。
- 数 W：`plan.file_content_transactions()` 随 W 涨（这是 Δ，不是总数）。
- 其余全是 O(盘数 × 根层级) 或 O(h × log N)：K1 一项（`admission.rs:1381`）、中央映射树按层二分（`admission.rs:1437`）、记账树闭式、墓碑退役层数（存下来的下标）。
闸之后的固定点与全部规划不在「准入之前」，但它们是决策（1.5）。

### 1.3 上界每一项的式子，是不是真上界

先写两个方向，因为 B6 里有两种上界，方向相反，不能混：
- 需求一侧（落点判的单元数、式子判的需求）：上界要 ≥ 精确值，放得下就保证精确值也放得下。
- 读数一侧（ckpt_cost 保留池、已承诺预留）：它们在式子里是扣减项，要 ≥ 精确值，扣得更多，可用更小，放得下才保证精确读数也放得下。方向与需求一侧同为「取大」，但对象是「扣」，不是「要」。

码 2 树的共用引理（读，我自己从规划器推的，没穷举）：规划器先删后插（「for deleted in previous_keys.difference(keys_after_this_publish) {」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/code_two_tree.rs:656`）、「for inserted in keys_after_this_publish.difference(&previous_keys) {」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/code_two_tree.rs:659`）），重写的节点 = 删或插的 key 的路径上的节点（`mark_rewritten`，`code_two_tree.rs:374`）加切出来的右半（`carried_from: None`）加新根。设删 D 把、插 I 把，上一版第 ℓ 层有 n_ℓ 个节点：第 ℓ 层重写的老节点 ≤ min(n_ℓ, D + I)；切出的新节点按「老节点第一次切至少要加 1 条、切出的两半再切一次至少要加 ⌈容量 ÷ 2⌉ 条」数，叶层加的是 I、内部层加的是下一层切出的个数；根切开长高一层。这与 `admission.rs:1484` 的 `central_mapping_tree_nodes_an_empty_publish_rewrites_at_most` 逐层同一个式子（「let splits_at_this_level = match level {」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:1512`））。所以：只要 D 与 I 给得住，这个式子对任意上一版形状、任意进出集合都是真上界。D、I 给不给得住，是下表每一行的问题。

| 项 | 上界式子（B6 用） | 真上界？ | 依据与缺口 |
|---|---|---|---|
| 数据单元 | W（精确，`plan.file_content_transactions()` 的项数） | 精确，不是上界 | 落点判的 `data_units` 本来就是精确个数（`admission.rs:1025`） |
| inode 树 | 写类步骤：重写容器 ≤ 写入条数 n_w，另加根 1；删最后一批：≤ 2 片 + 根 | 写类步骤：读，真上界；删最后一批：沿用第一轮转述，我没读内部 | 写一条记录至多重写一片：覆盖在持有它的容器里，或追加进最右有空位的容器，或开一片新的（`inode_tree.rs:1147` 起的循环，规格 `writing_progress`）。`remove_record_from_leaf_containers`（`inode_tree.rs:1692`）的内部我没读 |
| extent 树 | 下段：这个文件的 `lower_segment_nodes_of_a_file_without_holes(w)`；上段：每个被改的 inode 至多（上段根层级 + 2） | 沿用第一轮正推 A3.2 表，我这一轮没复核 | 标「推，没复核」；攻方可直接攻这一行 |
| 墓碑树（意图步） | 容器 ≤ 1 片（开放容器重写，或新开一片），索引节点 ≤ H + 2（H = 存下来的内部层数），另加 `deleted_inodes` 根 1、logged_ops 根 1 | 推，真上界；已有的 ckpt_cost 谱系式子 2 + 1 + H 在最坏边上少 1（下一行） | 最右端新开一片容器：停的那一层及其之下各层新开节点、之上各层整条重写、最右边整条都满时再长一层（`container_tree.rs:2358` 的文档注释）。边上每层都满时新节点 = 根层级个（H + 1）+ 新根 1 = H + 2，老节点一个不重写 |
| 谱系树（回退步）、退役树 | 谱系：容器 2 槽 + 索引节点 ≤ H + 2；退役：≤ 2 片容器（4 槽）+ 索引节点 ≤ H + 2，合 6 + H | 退役：与 `admission.rs:470` 的常量「6」一致（读）；谱系：已有的 `checkpoint_cost_of_one_lineage_container_cow` = 2 + 1 + H 在「开放容器满、最右边每层都满」那一形上少 1 个节点（推，没跑） | 谱系容器容量 1359 条（E199 产物行 1 的 `lineage_capacity=1359`），回退每次追加 1 条，开新容器的那次最坏边是 H + 2 个索引节点（上一行同一推导）。这是已有 ckpt_cost 谱系一项的缺口，不是 B6 引入的；B6 若「沿用 ckpt_cost 现成的两项」（第二轮辩方报告第三节）就带着它 |
| 分配记录树 | K1：1 + Σ_盘 Σ_层级 的位置数（`admission.rs:1381`） | 真上界，无条件 | 重写的节点 ⊆ 这一版的节点 ⊆ 单元区罩到的位置（`admission.rs:1381` 起的文档注释「恒是上界，不读这一版有哪些节点」）。代价：比实际多一大截（两块 4 GiB 盘 K1 = 529，实际改 3–4 片叶加祖先；任务 6265 读数把今天的节点数设成 17，是设的，不是量的） |
| 记账树 | D ≤ 3 + 6 × 盘数（上一代的行），I = 同数；式子取码 2 树引理，n_ℓ 取存下来的分层节点数 | 引理成立（读），D、I 给得住（读：`transaction.rs:7980` 的 `accounting_entries_of_this_publish` 每次发布只重做当代的 3 + 6 × 盘数行，台阶行原样照抄） | 台阶行从不进 D、I；所以台阶行再多，这一项只随 h 与 log N |
| deadlist 树 | 没有活快照：0；有：D ≤ 删快照这一批放的条数，I ≤ 这次换下的用户可见角色数 | 推；没有活快照那一支读得出 | 条目增减在 `transaction.rs:6540` 起的 `deadlist_changes_of_this_publish` 里；没活快照时每个换下的角色的判定都是「放进 defer 队列」，不进 deadlist |
| 中央映射树 | D ≤ 这次换下的、进映射的角色数；I ≤ 这次新写的、进映射的角色数（把上面各树的上界加起来，含 K1 个分配记录树节点与记账树节点）；整理的一批另加搬走的 key 数 × h（`transaction.rs:9416` 起的搬迁路径） | 引理成立；D、I 是上面各行之和 | 每个新写的进映射的单元一把新 key：数据单元一把一个，所以 I 随 W 涨。**ckpt_cost 的中央映射项不能顶这一项**：它按空发布的删插数算。任务 6394 读数：「fixed_point_vs_checkpoint_cost capacities=capped_mapping_4_8 data_units=85 checkpoint_cost_of_the_previous_version=29 fixed_point_slots_written=43 excess_slots=14 mapping_tree_nodes_after=38」，压小容量下写 85 个数据单元，固定点写的槽比上一版的 ckpt_cost 多 14 |
| 树表、实例表页 | 树表 1；实例表页只在写行发布 | 精确 | `admission.rs:1216` 的常量与 `:1306` 的相加；写行发布罩在切换预留里 |
| ckpt_cost（读数一侧） | K1 与中央映射项照今天的闭式（精确，O(盘数 × 层数 + h × log N)）；记账项换成「这一版节点数 + 码 2 引理的新增节点数上界」；谱系、退役两项补上上面少的 1；树表 1 | 记账项 ≥ 精确值（读：精确值是「这一版节点数 − 摘掉的 + 新增的」，去掉摘掉的只会更大）；中央映射项对记账项单调，所以也 ≥ | 比今天的精确值最多多出记账项的新增节点（我推 ≤ 2h，没跑），其余项一字不改；这是「放得下」判据的带宽，不改变放行的方向 |
| 已承诺预留（读数一侧） | `committed_reservation_for_tombstones_and_retirement_records`（`admission.rs:414`）取：上一版层数（意图步或有退役时加 1）、`live_objects` 与 `inode_records` 取上一版的计数加这次写的条数 | 读：该函数对三个量都是非负系数的线性式，单调；上界成立 | 要读墓碑、退役树根头的层数，已经是存下来的下标（`transaction.rs:4273`、`:4285`），不线性找 |
| 记录条数 | 不要上界：留在固定点之后精确判（`transaction.rs:12554`） | 不适用 | 任务 6394 读数：「bounded_journal_count checkpoint_cost=664 named_entries_per_record=64 records_under_the_bound=95 record_limit=85 today_outcome=published floor_raises=0」，用上界数记录会把今天照发的拒掉，所以 B6 把它留在精确判是对的 |

哪几项只能取无条件上界：分配记录树（K1，再没有别的）；deadlist 有活快照那一支（被钉住的数要逐个问判定，上界取换下数）。其余各项都有按差量的式子。

收紧不了的地方也在这张表里：中央映射项的 I 里装着 K1 个分配记录树节点，所以放行带宽 ≥ K1 − 实际固定点节点数，再加中央映射项自己的 min(n_ℓ, D + I) 的松量。任务 6265 读数（条件：设每盘今天改 17 或 25 个节点）：「name=k1_band geometry=2x4GiB slots_per_device=262144 root_level=2 positions_per_device_by_level=[262, 2] k1=529 todays_settled_nodes_assumed=17 band_blocks_at_least=512 band_per_device_mib=8.0 band_share_of_device_percent=0.195」。落进带里的发布整条走今天的路，而被拒的发布正是带里的：拒绝路径本来就是整条今天的代码，B6 对它没有任何改动（1.4）。

### 1.4 回落那一支的拒绝次序与种类与今天逐个相同吗

答：相同，条件是上表「需求一侧」与「读数一侧」都真取大；一条不成立，快路就能放行今天要拒的发布，次序与种类立刻变。逐类：

| 今天的拒（先后次序，冻结副本行） | 种类归类（`refusal_is_short_of_space`，`mounted_session.rs:1871`） | 快路上 | 慢路上 |
|---|---|---|---|
| 普通半的拒：`InodeTreeWriteRefused`、`DeletedInodesQueueRefused`、`NextContainerNumberPastTheTopOfItsRange`（12510 的 `?`） | 都不推抬 F，原样交回 | 改成从固定点第一轮的 `resolve` 里出（`transaction.rs:13063`），因为 `resolve` 开头调的是同一个 `ordinary_allocation_part`（8729），同一个第一个错 | 照旧 |
| 落点判拒 `SpaceAdmissionRefused`（12523） | 推抬 F | 闸已保证不拒 | 照旧 |
| 固定点里的拒：`MultiLevelCodeTwoTreeRefused`、释放判定的错、`PlacementRefused`（推抬 F）、`AllocationRecordTreeRewriteSetDidNotSettle`（12536） | 前一类不推，`PlacementRefused` 推 | 一字不改 | 照旧 |
| `JournalRecordsOfThePublishExceedTheLimit`（12554） | 不推 | 一字不改 | 照旧 |
| 合取 7 `EmptyClusterSegmentsBelowTheFloor`（12577） | 不推 | 一字不改 | 照旧 |
| 式子判拒 `SpaceAdmissionRefused`（12621） | 推抬 F | 读法乙：位置不动，闸保证不拒；读法甲：换成断言（上界错了就 panic） | 照旧 |

两个边：
- 闸遇到「只供测试的关准入开关」（`SpaceAdmission::SkippedByTheTestOnlySwitch`，12513 起那个 `match`）必须直接走慢路，否则开关关了闸还在判。
- 读法甲把式子判挪到固定点之前，而 D28 已定项 1 的接线写的是「发布路径在 `transaction::prepare_the_version_publish` 里判：算定这次发布的形状之后、读盘核与动分配器之前。」（`.claude/kb/decisions/28-挂载期承诺量.md:35`），需求写的是「需求 = 这次发布新写的全部槽（普通分配加这次写出的固定点，不加这次换下的槽）」（`.claude/kb/decisions/28-挂载期承诺量.md:38`）：时序一句与「判需求用上界」两处要改，这是条款变更，不是实现细节（第二轮判决格四已写）。

推翻「次序与种类逐个相同」的现象：找到一段历史，闸放行而今天的落点判、记录条数或式子判会拒；或今天有一种拒不在上表里。第二种我读 `resolve` 的 `?` 点（8718 起）与 `settle` 的 `?` 点只找到上表这几类。

### 1.5 快路放行之后的全量规划在 D8 已定项 16 射程免责原文「装单元、拷字节」之内吗

不在。射程原文：「准入之前的步代价不随盘上容器总数、记录总数涨；准入之后装单元、拷字节不算运行时决策路径。」（`.claude/kb/decisions/08-核心索引结构.md:516`）。免责只点名两样：装单元、拷字节。快路放行之后还在跑的（读）：
- 规划与折叠：`resolve` 每轮重跑普通半、谱系树、deadlist 改动、多层码 2 树规划（`transaction.rs:8718`、`:8976`），这些是算形状，不是装单元，也不是拷字节。
- 分配：固定点在分配器的拷贝上放掉换下的、取这次所有角色的落点（「allocate_placement_for_role(&mut rehearsal, *identity, plan.txg)?;」，`transaction.rs:13117`）。`.claude/rules/fs-design.md` 把分配点名成运行时决策路径：「| **运行时决策路径**（分配、ENOSPC 准入、defer 窗口、生命周期判定） | **不许**，且代价不许随盘容量增长 |」（`.claude/rules/fs-design.md:23`）。预演里的克隆（「let mut rehearsal = allocator.clone();」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:13098`））随单元区涨：任务 6553 读数「allocator_copy_by_unit_area unit_area_slots=16384 … allocator_clone_ns_median=2870」「allocator_copy_by_unit_area unit_area_slots=16777216 unit_area_gib_per_device=256 allocation_records=116 allocator_clone_ns_median=34949849」。
- 仍能拒：快路放行之后固定点还会交回多层码 2 树拒、取不到落点、记录超上限、合取 7（上表第三到第五行）。这些是决策，不是拷字节。
- 属于「拷字节」的只有那几处整份拷墓碑、退役、谱系容器与形状（`transaction.rs:8606`、`:8651`、`:8939`，`container_tree.rs:3575`–3576）；按字面它们免责，规划与分配不免。

所以快路的收益只有在用户确认「准入」就是上界闸、「准入之后」免责的口径比字面宽（包括规划、预演里的分配与仍能拒的几步）时才算数；这与 B4 要改的是同一处条款（第二轮判决格三），B6 没有绕开它。

推翻这一格的现象：找到 D8 已定项 16 或 `fs-design.md` 里另有一句把「放行之后的规划与预演」点名免责；我 `grep -n '准入之后'` 的结果：`08-核心索引结构.md` 只命中 516 一行，`fs-design.md` 零命中。

## 格二　B7 改完还有没有随总数涨的步

### 2.1 B7 在冻结副本上具体改哪几步

B7 = 多层码 2 树与三棵容器树的规划只读进出的 key 与路径、节点身份不按平铺位置 + 固定点每轮从同一个底子重放走分配器差量视图 + 五件。逐件落到文件:行：

| 件 | 文件:行 | 改什么 | 连带要改的 |
|---|---|---|---|
| 码 2 树规划只读进出 key | `code_two_tree.rs:637`（规划器）：入参从「这一版全部 key 的集合」改成「删集、插集」；去掉 650 的全 key 收集、`planning_node_of`（390，整棵建规划树）、`flatten`（583，整棵摊平）、685 的 `replaced_previous_nodes`（过滤上一版全部节点） | 输出 `CodeTwoTreePlan` 现在是全部节点 + 全部来源（`code_two_tree.rs` 的结构体），要改成「重写的节点 + 形状增量」 | 调用方 `transaction.rs:9015`（记账）、deadlist 树、中央映射树（9399 起）、`admission.rs:1358`（ckpt_cost 的记账项）、`code_two_tree.rs:704` 的零根版本 |
| 节点身份不按平铺位置 | `transaction.rs:2542`、`:2547`、`:2580` 的三个变体（`AccountingTreeNodeBelowTheRoot`、`MappingTreeNodeBelowTheRoot`、`DeadlistTreeNodeBelowTheRoot`，身份里装着 `CodeTwoTreeNodePosition`，`role_of_node` 在 `transaction.rs:2621`） | 身份换成不随左边插删而挪的标识 | 这三个变体名在冻结副本 14 份源文件里共命中 98 行（读：`grep -rE` 三个名字按文件 `grep -c` 加总；`transaction.rs` 56 行，余下在 `mount.rs`、`mounted_session.rs`、`admission.rs`、`write_accounting.rs`、`model_comparison.rs`、各测试与实验二进制）。身份是内存里的角色名；journal 的点名条目存类、出生树、出生 txg 与 key 尾（`transaction.rs:16434–16447`），不存角色名；`units` 的查找、释放判定、`mapped_units` 都按它走 |
| 差量规划的消费者 | `transaction.rs:8976` 起 `plan_the_multi_level_trees`（记账键集合 9013–9030、deadlist 键集合、9399 的中央映射键集合）、`mapped_units` 的整张表（9057–9392 各个循环）；`ResolvedPublish.mapped_units`（`transaction.rs:7061`、`:8071`）；`deadlist_changes_of_this_publish`（6540，6659 起的 `entries_after` 全量） | `mapped_units` 与 deadlist 条目都改成差量；全量表挪到装单元那一步 | `publish_admitted` 里 15414–15623 的 `mapped_units_with_locations`（用全量表，在准入之后）；`assert_eq!(mapped_units, resolved.mapped_units)`（15619）要改成对差量核 |
| 三棵容器树按引用读 | 「container.contents.clone())」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:8606`、`:8651`、`:8939`）三处整份拷容器；`container_tree.rs:3575`、`:3576` 的 `copy_of_the_containers` 与形状 `clone`；`transaction.rs:6912` 的 `mapped_units_of_a_container_tree`（6923、6937 两个循环逐容器逐节点） | `append_records_to_the_open_container` 现在返回「全部容器 + 全部形状」，要改成返回「重写与新开的容器 + 形状增量」 | 这个函数带 Verus 规格（`container_tree.rs:3537` 起的 `ensures appending_keeps_the_key_to_entry_mapping(containers_before_this_publish@, …)` 对整个容器序列说话），换返回类型规格要重写；`tombstone_tree.rs:164`、谱系、退役的包装一并 |
| 固定点同一个底子重放走差量视图 | 「let mut rehearsal = allocator.clone();」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:13098`）与写行那一份 `:1938` | 视图叠的状态见第二轮攻方报告 G3 那张表（三张逐槽位图、按段计数表与七个总数、记录表与已回收集合、开放段与聚簇段集合） | `rehearsal.records()` 的三个消费者（`transaction.rs:13107`、`:13120`、`:13121`）要换成「视图里被动过的记录」，否则整份记录切片照旧传进 `allocation_record_tree.rs:337`、`:367` |
| 取空槽与记一条分配按索引找 | `allocator.rs:971`、`:974`（用户数据落点）、`:1152`（`make_room_for_record_on_device` 的相交过滤） | 空闲对按段摘要找，聚簇段集合按起点 `range` 查；记录表按 (盘, 槽) 建索引 | 同一函数里 `:1144` 起的 `iter_mut().find` 与 `records.retain(…)` 也是线性的；`records` 是平 `Vec`（`allocator.rs:1877`），`retain` 会让下标索引失效，要么换有序映射、要么每次删后重建 |
| 释放按索引找 | `allocator.rs:2265`、`:2211` | 按 (盘, 槽) 建索引 | 同上 |
| 根头按下标取 | `transaction.rs:4348–4351`、调用点 `:4404`、`:4422` | 装 `units` 时记下四个根的下标，照墓碑、退役根已有的 `unit_at_the_recorded_position`（`transaction.rs:4328`） | ckpt_cost 两次求值都受益 |
| 墓碑退役谱系容器按引用读 | 已并入上面「三棵容器树」那一行；`:12511` 是落点判前那一遍 `ordinary_allocation_part` 的调用 | 同上 | 同上 |
| ckpt_cost 记账树一项按差量规划 | `admission.rs:1325` 起的 `accounting_tree_nodes_after_the_publish_built_on`：「entry.generation == version_txg)」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:1339`）、「.chain(version_to_build_on.inode_number_watermark_steps())」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:1349`）、「let nodes = match plan_the_tree_after_this_publish(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:1358`） | 只走这一代的 15 行删插（两块盘）；这一代的行与最后一级台阶、节点数随一版另记 | `TransactionOutput.accounting_entries` 之外多存一份「这一代的行」；`inode_number_watermark_steps`（`transaction.rs:4243`，逐行过滤、排序）在 `plan_the_multi_level_trees` 里（记账键集合那一步）的调用也一并去掉 |

### 2.2 改完还剩哪几步随总数涨

分两类写：B7 字面没点名的；B7 字面点名、但消费它的函数不改就仍涨的。每步写随哪个总量涨。

字面没点名的（读）：

| 步 | 文件:行 | 随什么涨 |
|---|---|---|
| 聚簇段求和 | 「let pairs_inside_the_cluster_segments: u64 = cluster_segments」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/allocator.rs:578`），由 `admission.rs:992` 每块盘调一次，落点判的读数 | S_c |
| 开新聚簇段的线性扫 | 「let full_segments = self.allocated.len() / per_segment;」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/allocator.rs:1004`）起的 `.find` 从第 0 段扫到第一个全空段；回落路径 `allocator.rs:1035–1050` 从单元区起点逐槽扫。提交内生块的开放段装不下时每块盘每个角色走一次 | S / 64（扫过的被占段数）、回落时 S。第二轮攻方 G2 点名了 1004，没点名 1041；B7 的「取空槽」若只指 971、974，这两处不在内 |
| extent 树与第一个文件之外的文件 | `transaction.rs:7660`（每个文件造一份计划并拷 `extent_lower_nodes`）、`:4090`（把全部有条目的 inode 收成 `BTreeSet`）、`:7847`（`flat_map` 遍历每个文件的下段节点）；`mapped_units` 里 9087、9126、9208 三个循环（第一个文件的照抄数据单元、其余各文件的每个数据单元、每个下段节点） | F_n、U、extent 节点数。B7 字面里没有 extent 树；第一轮 A2 的定义里有 |
| inode 叶容器的整份拷 | `transaction.rs:4205`（`inode_leaf_container_contents`，4208 逐片 `contents.clone()`）、`inode_tree.rs:1093` 的 `copy_of_the_leaf_containers` | 封顶：至多 135 片 × 每片至多 233 条 = 31455 条（读：`inode_tree.rs` 的 `MoreLeafContainersThanOneRootNodeHolds` 与 233 的规格）；按定长 140 字节算每份 ≈ 4.4 MB（推）；每次发布 (1 + r) 份加写者再拷 r 份。不随总数无限涨，但不是 Δ |
| 版本侧的两张全索引 | 「let mut previous_read_by_role = MappedUnitsOfAVersionReadByRole::of_the_version(previous);」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:4613`，同形 `:5213`、`:5290`）第一次查时建 `mapped_units_by_role()`；「let mapped_units_of_the_previous_version = previous_version.mapped_units_by_role();」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:6581`）无条件建；按 inode 号找文件的 `FilesOtherThanTheFirstOfAVersionReadByInode::of_the_version`（`transaction.rs:4614`、`:5132`、`:6557`） | M log M、F_n；每轮至少一遍。B7 的「释放按索引找」是分配器侧（`allocator.rs:2265`、`:2211`），这两张表是版本侧，要「索引随版本带着」才去得掉 |
| 分配记录树自己的计划与节点集 | 「previous_version.allocation_record_tree.node_set(),」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:13045`）、「let mut nodes_after: BTreeSet<AllocationRecordTreeNode> = previous_nodes.clone();」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:13051`）、`allocation_record_tree.rs:429` 的 `plan_the_tree_after_this_publish`（遍历 `nodes_after` 全部、再遍历 `previous_nodes` 全部）、「let before = records_of_each_leaf(geometry, previous_records);」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/allocation_record_tree.rs:372`）与同形的 `after`、`nodes_holding_records`（`allocation_record_tree.rs:337`）：每轮把全部记录分叶再比 | N_a、R log R；每轮约 5 遍全记录（`nodes_whose_contents_changed` 在 13107 与 13120 各一次，每次 before、after 各一遍，再加 13121 的 `nodes_holding_records`）。B7 写的是「重放走分配器的差量视图」，这几个函数收的是记录切片与节点集，不是分配器视图 |
| 差量的前提没有断言 | `settle_the_allocation_record_tree` 不要求进门时分配器里的记录与上一版的 `allocation_records` 相同（进门只取 `previous_version.allocation_records` 当「上一版」，`transaction.rs:13045` 一带）；只有真发后一句核「准入之前在分配器拷贝上走到的固定点，与真发时取完落点的记录说的是同一棵树」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:15225`） | 差量版只看被动过的叶，要么每个会改分配器记录的入口都登记「自上一版以来被动过的记录」，要么进门前 O(R) 核一遍（又涨）。这是 B7 要新增的增量状态，不是已有的 |
| 实例表链（写行发布） | `transaction.rs:8768`（`instance_table_page_roles_in_bump_order`）、「instance_table_chain_to_release(&rewrite.replaced_chain, allocator)?;」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12945`） | 实例表行数（每次可写挂载一行）；只在写行发布走到，每轮的释放判定里再来一遍 |
| 抬 F 的预演 | `mounted_session.rs:2918–2921`：一份分配器克隆后连发的次数上限写在「至多环长 3S ≤ 48 次」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/mounted_session.rs:2914`）里，每次是一整条 `prepare_the_version_publish`；被拒后会话重判（`refusal_is_short_of_space`，`mounted_session.rs:1871`） | 不是新的随总数涨的步，是乘数：每次被拒的发布，两道准入之前的全部步骤要再来 1 次（真发的重判）加预演至多 48 次。B7 把单次的步变便宜，乘数不变 |

字面点名、但连带的消费者不改就仍涨的：`mapped_units` 整张表与 `ResolvedPublish.mapped_units`（7061、8071、9057–9392、9399）、`deadlist` 的 `entries_after`（6659 起）、`plan_the_files_other_than_the_first`（7660）的结果类型。这三个若不变成差量，码 2 树的差量规划虽然只走路径，调用方仍整表收一遍键集合。

### 2.3 第二轮 G1 现量的「固定点逐轮挤动先放下的槽」在 B7 的「同一个底子重放」下还在不在

答：现象还在，但它不再是一条反例。理由（读 + 推）：
- 今天的固定点每轮就是从新克隆起放掉全部换下的、再按 bump 次序给全部重写的角色重新取落点（`transaction.rs:13098` 起到 `:13117`）。也就是说今天已经是「每轮从同一个底子重放」，G1 量到的「同一个角色在相邻两轮落在不同槽」是今天自己的行为：任务 6553 读数：「settle name=settle_rounds change=sequential_write_of_64_data_units settling_calls=1 allocation_rounds_per_call=[2] release_only_rounds=1 roles_compared=74 roles_shifted_between_rounds=5 new_roles_placed_before_a_role_of_the_earlier_round=2 first_shift=AllocationTreeNodeBelowTheRoot(AllocationRecordTreeNodePosition { level: 0, device: DeviceIdentity(1), index_in_device: 1 }):1580->1581」。
- B7 用视图替掉克隆，视图的语义若等于克隆（释放、取落点、回收三类操作叠在同一个底子上，下一次找看得见刚置的位），每轮的落点逐个与今天相同，所以挤动也逐个与今天相同。G1 打中的是读法乙（接着上一轮只补新进来的），那一读法下新并进重写集的分配记录树节点在 bump 次序里排在已取过槽的记账树、中央映射树之前，后者的槽整串往后挪，结局与今天不同。B7 的定义写的是「每轮从同一个底子重放」，是读法甲，G1 不中。
- 样本的宽度：任务 6553 读数的汇总行「settle name=settle_rounds_summary changes=18 settling_calls=20 calls_with_two_or_more_allocation_rounds=1 most_allocation_rounds_in_one_call=2 roles_compared=74 roles_shifted_between_rounds=5 new_roles_placed_before_a_role_of_the_earlier_round=2」：20 次固定点调用里只有 1 次走了两轮取落点，挤动只在那 1 次里量到。其余 19 次一轮取落点，「roles_shifted=0」是没有可比的第二轮，不是「比过没挤动」。所以 G1 证明的是存在，不是常见。
- B7 下仍随总数涨的代价：每轮重放全部重写角色（W 个数据单元也在内），每个角色一次索引查加一次记录插入；轮数 r 个。随 W·r，W 是这次发布自己的大小，不是总数。

分配器差量视图与本体不一致时谁先看见（读）：
1. 固定点自己看不见：`settle` 里「装着记录的节点等于这一版的节点」与「变了的叶都在重写集里」两个判据，都是拿视图算出的记录比视图算出的记录，自洽，不看本体。
2. 第一个看见的是真发：`publish_admitted` 先真释放（「allocator.release_leaving_the_record_allocated_on(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:14714`）），再真取落点（「let placement = allocate_placement_for_role(allocator,」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:14819`））。视图说取得到而本体取不到，这里交回 `PlacementRefused`，而分配器已经被真释放改过；`transaction.rs:14823` 的注释写「失败时整个分配器由调用方换回去」，我没有逐个核调用方是不是都换（没核）。
3. 叶集合不同时才撞断言：「准入之前在分配器拷贝上走到的固定点，与真发时取完落点的记录说的是同一棵树」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:15225`）比的是叶集合与节点集合；槽号不同而叶集合相同，三处都不叫：盘上写的是真发那一份，固定点那份没被用上。式子判只看跨度，也看不见。
所以不一致的最坏观察者是第 2 条（落点取不到时分配器已动），最隐蔽的是第 3 条（槽号不同而叶集合相同）。

推翻「G1 在 B7 下不中」的现象：造出视图与克隆在某个操作序列上落点不同（例如视图里刚置的位下一次找看不见），或证明 B7 的「同一个底子」读作「第 k 轮以第 k − 1 轮的预演结果为底」。

## 格三　随总数涨的步的清单完不完整

答：不完整。第一轮正推表一、表二与第二轮 G7–G9 之后，我多找到三条（表里标「新」）：回落路径的线性扫、实例表链、抬 F 预演的乘数。其余各行都能对到前两轮的某一条。下表按执行次序，从 `prepare_the_version_publish`（`transaction.rs:12409`）到式子判（`transaction.rs:12621`），每步写随哪个总量涨。「每轮」= 在 `settle_the_allocation_record_tree` 的循环里每进一次再来一遍（`resolve` 开头就是 `ordinary_allocation_part`）。

| # | 步 | 文件:行（冻结副本） | 随什么涨 | 对到哪 |
|---|---|---|---|---|
| 1 | 普通半：拷全部 inode 叶容器；写者再拷一遍 | `transaction.rs:4205`；`inode_tree.rs:1093` | 封顶：≤ 135 片 × 233 条 = 31455 条（读），按 140 字节 ≈ 4.4 MB 每份（推） | 表一 #2、#3；每轮再来 |
| 2 | 第一个文件之外的文件逐个造计划并拷下段节点 | `transaction.rs:7660` | F_n × 下段节点数 | 表一 #6；每轮再来 |
| 3 | extent 上段：把全部有条目的 inode 收成集合 | `transaction.rs:4090` | F_n + 1 | 表一 #7；每轮再来 |
| 4 | 墓碑树、退役树、谱系树：容器整份拷、形状 `clone`；退役树每次发布无条件调追加 | `transaction.rs:8606`、`:8651`、`:8939`；`container_tree.rs:3575`–3576 | C_t、C_r、C_l 与各自节点数 | 表一 #9、#10、表二 #23，第二轮 G8；每轮再来 |
| 5 | 取这一半重写的角色（extent 一半遍历每个文件的下段节点） | `transaction.rs:7847` | F_n × 下段节点数 | 表一 #12 |
| 6 | 落点判的参数：ckpt_cost 的四个根头线性找 | `transaction.rs:4351`（调用点 `:4404`、`:4422`） | U | 表一 #13，第二轮 G7 |
| 7 | 落点判的参数：ckpt_cost 的记账树一项（全量收键、全量规划） | `admission.rs:1339`、`:1349`、`:1358` | A（含台阶行） | 表一 #14，第二轮 G10 |
| 8 | 落点判的读数：聚簇段求和 | `allocator.rs:578`（`admission.rs:992` 调） | S_c | 表一 #17，第二轮 G9 |
| 9 | 每轮：分配记录树的节点集、计划、`intersection` | `transaction.rs:13045`、`:13051`；`allocation_record_tree.rs:429` | N_a | 表二 #20、#21 |
| 10 | 每轮：`resolve` 重跑普通半 | `transaction.rs:8729` | 同 #1–#5 | 表二 #22 |
| 11 | 每轮：deadlist 改动（`mapped_units_by_role` 全索引、`entries_after`、`kept_mapping_after`） | `transaction.rs:6540`、`:6581`、`:6659` 起 | M log M、E_d、E_k | 表二 #24 |
| 12 | 每轮：多层码 2 树规划（全 key 集合、整棵规划树、摊平、过滤全部上一版节点） | `transaction.rs:8976`；`code_two_tree.rs:637`（650、390、583、685） | 记账行数、映射 key 数 | 表二 #25 |
| 13 | 每轮：`mapped_units` 整张表与键集合 | `transaction.rs:9057`–9392、`:9399` | M | 表二 #25、#26 |
| 14 | 每轮：三棵容器树的 `mapped_units_of_a_container_tree` | `transaction.rs:6912`（6923、6937） | 各自容器数与节点数 | 表二 #26 |
| 15 | 每轮：整理搬迁标路径尾部过滤上一版全部节点 | `transaction.rs:9416` 起 | N_cm | 表二 #27 |
| 16 | 每轮：释放判定两张版本侧全索引与按 inode 找文件的索引 | `transaction.rs:12880`→`:4602`、`:5118` | M、F_n | 表二 #28 |
| 17 | 每轮：克隆分配器 | `transaction.rs:13098` | S 与 R | 表二 #29 |
| 18 | 每轮：每次释放在记录表里线性找（逐盘） | `allocator.rs:2265`、`:2211` | R × 释放个数 × 盘数 | 第二轮 G2（3） |
| 19 | 每轮：用户数据落点从单元区起点逐对扫，每个候选逐个比聚簇段 | `allocator.rs:971`、`:974` | P × S_c × 数据角色数 | 第二轮 G2（1） |
| 20 | 每轮：记一条分配在全部记录里过滤相交的 | `allocator.rs:1152` | R × 重写角色数 | 第二轮 G2（2） |
| 21 | 每轮：提交内生块开新段的线性扫 | `allocator.rs:1004` | S / 64（被占段） | 第二轮 G2（1）末句 |
| 22 | 每轮：**新** 提交内生块回落路径从单元区起点逐槽扫，每个候选再对跨度逐槽判 | `allocator.rs:1035`–1050（「let mut candidate = self.unit_area_start.slot().0;」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/allocator.rs:1041`）） | S（没有全空段时才走到） | 第二轮 G2 只点名 1004，没点名 1041 |
| 23 | 每轮：全记录分叶再比（`nodes_whose_contents_changed` 两次、`nodes_holding_records` 一次） | `transaction.rs:13107`、`:13120`、`:13121`；`allocation_record_tree.rs:337`、`:367`、`:372` | R log R，约 5 遍 | 表二 #30 |
| 24 | 每轮：记账行表 | `transaction.rs:7980`、`:8017` | A | 表二 #31 |
| 25 | **新** 写行发布：实例表链页数加旧链逐片释放 | `transaction.rs:8768`、`:12945` | 实例表行数（每次可写挂载一行） | 前两轮都没列 |
| 26 | 式子判的读数：ckpt_cost 第二遍（#6、#7 再来一遍） | `admission.rs:1744`→`:1711`→`:1211` | U、A | 表二 #32 |
| 27 | **新（乘数）** 被拒后会话抬 F 再判；抬 F 的预演在一份分配器克隆上连发至多 48 次整条 `prepare_the_version_publish` | `mounted_session.rs:1871`、`:2914`、`:2918`、`:2921` | 不是新的随总数涨的步，是上面所有步的乘数：每次被拒的发布再来 1 次 + 预演至多 48 次 | 第一轮 A3 乙里提过「被拒路径至多 8 次整发布」，没提预演 |

查过没有随总数涨的（读）：盘身份（12501，盘数）；`deleted_inodes`、logged_ops（常数；沿用第一轮表一 #8、#11 的转述，I-9.24「删除队列至多一条 key」与签发处的断言，我没复核）；中央映射树的 ckpt_cost 一项（`admission.rs:1437` 按层二分，h × log N）；K1 一项（`admission.rs:1381`，盘数 × 层数）；合取 7 的 `empty_segments`（`allocator.rs:907` 返回一个增量计数）；式子读数的 `AdmissionReading::of_allocator`（逐盘读增量计数，`admission.rs:759`）；已承诺预留（读存下来的根头下标）。

推翻「清单只缺这三条」的现象：在 12409 到 12621 之间找到一个随 U、M、R、S、C、F_n 之一涨、而上表与前两轮都没有的函数调用。我是顺着 `judge_the_publish_before_any_read_or_write` 与 `settle_the_allocation_record_tree` 的调用链读的，没有对每个被调函数做调用图的机械展开。

## 格四　前两轮的数能不能核

我把三个任务的日志读了一遍（任务服务此刻连不上，直接读 `/home/fy5090/.local/state/singlefs/task-server/logs/` 下的文件；任务 6265、6394 在本机跑，任务 6553 在第二台跑，日志里写两台工具链相同）。每份读数：取样点、支撑哪句、有没有一句用得比数宽。

| 任务 | 取样点（读） | 判决里用它的那一句 | 够不够 |
|---|---|---|---|
| 6265 `full_root_leaf_one_insert` | 6 个容量（1、2、3、8、64、255）各一行：「name=full_root_leaf_one_insert leaf_entries=1 height_before=1 height_after=2 rewritten=3 literal_bound=1」 | 第一轮判决第二节 A3 格：「根叶装满再插一把重写 3 个节点而式子给 1」 | 够：是按构造的存在性反例，六个容量一致 |
| 6265 `keys_times_height_sweep` | 5 个容量 × 每个 5 段历史 × 每段 400 次发布 = 9975 次：「name=keys_times_height_sweep capacities=5 histories_per_capacity=5 publishes_per_history=400 publishes_judged=9975 literal_bound_exceeded=1139 largest_excess_over_the_literal_bound=28 corrected_bound_exceeded=0」 | 同一格：「9975 次发布里超过 1139 次」 | 存在性够（上一行已经够）；这个频率只是模型自己的历史生成器里的频率，不能读成真实负载里的频率；`corrected_bound_exceeded=0` 只说这 9975 次里修正式子没被超过，撑不起「修正式子是真上界」（第二轮判决格六已这样写，同意） |
| 6265 `one_key_publish_output_size` | 3 个总数（1000、10000、100000，高 2、3、4），一个容量，只发一把 key：「name=one_key_publish_output_size total_keys=100000 height=4 nodes_before=3124 nodes_in_the_returned_shape=3124 origins_returned=3124 rewritten=4 replaced=4 carried_at_a_new_position=0 input_key_set=100001」 | 第一轮 A4 格：「交回的形状随总数涨、重写只有 4」 | 够，范围就是「规划器交回的返回值」；「与上一版怎么表示无关」那半句是读码，不是这个数 |
| 6265 `k1_band` | 4 个几何（2×4 GiB、2×64 GiB、2×1 TiB、2×16 TiB）；`todays_settled_nodes_assumed=17`（前两个）与 25（后两个）是设的，不是量的 | 第一轮 A3 格：「判它会多拒一带」 | 存在性够（K1 > 今天实际是算术）；带宽（`band_per_device_mib=8.0`、`band_share_of_device_percent=0.195`、`0.245`、`0.248`）依赖设的 17 / 25，只能写成条件式。第二轮辩方引它时已写「带宽是假设每盘实际改 17 或 25 个节点，推的」，同意 |
| 6394 `reorder_probe` | 一个几何（单元区 1216 槽，从 4000 槽往下按 32 槽一步扫了 88 个几何才扫到的第一个）：今天「outcome=refused_publish cause=journal_records_exceed_the_limit(86>85) floor_raises=0 images_unchanged=true」，模拟 A3 后「outcome=refused_no_space_after_raising_the_floor last_refusal=space_admission_refused floor_raises=0 images_unchanged=true」 | 第一轮 A3 格「A3 改了拒成哪一种」；第二轮判决第一节的更正 | 存在性够，更正写得对（两边 `floor_raises=0`，推抬 F 写盘在这条历史上没发生）；「A3 模拟」是模型作者自己在副本里把式子判挪前，不是任何人写的 A3 实现 |
| 6394 `bounded_journal_count` | 一个点：「name=bounded_journal_count checkpoint_cost=664 named_entries_per_record=64 records_under_the_bound=95 record_limit=85 today_outcome=published floor_raises=0」 | 第一轮 H2（按 ckpt_cost 数记录条数会拒掉今天照发的） | 够，是存在性；一个点 |
| 6553 `settle_rounds` | 18 次用户改动、20 次固定点调用：「settle name=settle_rounds_summary changes=18 settling_calls=20 calls_with_two_or_more_allocation_rounds=1 most_allocation_rounds_in_one_call=2 roles_compared=74 roles_shifted_between_rounds=5 new_roles_placed_before_a_role_of_the_earlier_round=2」 | 第二轮判决格一：B2 读法乙「与今天不逐个相同」 | 存在性够：那一次调用里确有 5 个角色在两轮间换了槽；但只有 1 次调用走了两轮取落点，其余 19 次「roles_shifted=0」是没有第二轮可比，不是比过没挤动；这 18 次是模型作者挑的一串，不是负载分布。判决括号里「覆盖写与建 inode 那几次都是 roles_shifted_between_rounds=0、一轮」前半句容易读成「没挤动」，要连「一轮」读 |
| 6553 `lowest_free_pair_search` | 4 个点（压在底下的数据单元 0、128、400、1200），单元区 16384 槽：候选数 3、163、435、1267 | 格二：取空槽线性扫 | 候选数随 P 涨够；「segment_comparisons_at_most=2534」是算出来的上界（候选数 × 聚簇段数），不是量到的比较次数；聚簇段只开了 1–2 个，P × S_c 的乘积没被量到 |
| 6553 `allocator_copy_by_unit_area` | 4 个点（16384、131072、1048576、16777216 槽），记录数只在 84–116 之间 | 格二：「克隆分配器随单元区涨（16384 槽 2870 ns，16777216 槽 34949849 ns）」 | 对 S 够；对 R 没取样（克隆随 R 涨是读码）；不是线性（131072→1048576 是 8 倍槽对应 63 倍时间，缓存效应）；`one_unit_overwrite_publish_ns_median` 同条里一并量，第二台一次 |
| 6553 `root_heads_behind_the_data_units` | 4 个点（写 1、16、128、400 个数据单元） | 第二轮 G7：「根头排在全部数据单元之后」 | 位置够；没量时间 |
| 6553 `tombstones_every_publish_copies` | 4 个点（1、8、32、96 条墓碑），墓碑容器数恒为 1（容量 815 来自 E199 产物行 1 的 `tombstone_capacity=815`） | 第二轮 G8：「覆盖写带着全部墓碑」 | 记录数够；容器数没取样超过 1，「拷 (1 + 轮数) 遍」是读码；多容器时随容器数涨是推 |

有没有一句用得比数宽：
- 「9975 次发布里超过 1139 次」当频率读就宽了（只在模型的历史生成器里）。
- 第二轮判决「克隆分配器随单元区涨」没写记录数，数是在约 100 条记录上量的。
- E199 的 G0 取 s = 4（固定点循环进 4 次）；任务 6553 的 20 次固定点调用里，19 次进的次数是 2（一次只释放、一次取落点），大写那一次是 3（读：覆盖写与建 inode 行是 `allocation_rounds_per_call=[1] release_only_rounds=1`，大写行是 `allocation_rounds_per_call=[2] release_only_rounds=1`，unlink 每次两个调用各 `[1]`）。s 取 4 比样本里常见的大；这影响格五的数，不影响哪条候选省哪一遍。
- 任务 6394 的日志里有两行判决没引、对格一有用：「name=fixed_point_vs_checkpoint_cost capacities=capped_mapping_4_8 data_units=85 checkpoint_cost_of_the_previous_version=29 fixed_point_slots_written=43 excess_slots=14 mapping_tree_nodes_after=38」（压小容量下 ckpt_cost 不是这次发布固定点的上界，1.3 已用）；「name=accounting_rows publishes=40 rows_at_the_start=15 rows_at_the_end=55 watermark_steps_at_the_start=1 watermark_steps_at_the_end=41 accounting_tree_nodes=1」（40 次发布台阶行 1→41，记账树仍 1 个节点：台阶行随历史涨量到了，「记账树跨过叶容量」那一形没被取样到）。

推翻「撑不起宽的那几句」的现象：在更多几何或历史上重跑出与现有读数同向的数；那样存在性不变，频率与带宽的句子才可能放宽。

## 格五　代价量级

E199 的几何：主几何 G0 = 10^6 次 unlink 留下的墓碑记录、一次 unlink 意图、退役记录 10^3、谱系记录 16、inode 10^3、数据单元 0、固定点循环进 4 次（`research/prompts/e199-preregistration.md:400` 的 G0 一行；产物行 4 的 `slots_per_device=2501`）。E199 只量两类字节：「照抄字节：登记第三节「甲臂的照抄点」那张表每一处在内存里再复制一份的字节数；新装字节四条臂相同、单列；现算重建字节单列成第三类。」（`.claude/kb/experiments/199-发布照抄上一版的代价.md:44`），并且「准入之前与准入之后两段各报各的，任何一格都不把两段加起来判。」（`.claude/kb/experiments/199-发布照抄上一版的代价.md:45`）。它不数「扫」类的步（格三表里 #6、#8、#17–#23 那些），所以下面每条候选的扫类代价 E199 都没有数。

E199 产物行里的数（标「推」：计数模型的算术，不是实现里量的）：
- 「E7RESULT name=arm_bytes point=main_geometry arm=jia_after_patch copied_before=404072244 copied_after=81200231 recomputed_before=1814496 recomputed_after=925712」（`research/results/e199-transaction-output-copy-2026-11-05.out:5`）：今天的做法（甲，带补丁）在 G0 上准入之前照抄 404072244 字节（≈ 385 MiB）、现算重建 1814496 字节。
- 分项行：「E7RESULT name=jia_after_patch_parts point=main_geometry ordinary_allocation_pass=80346376 settle_round=585019 outside_the_rounds=288」（`research/results/e199-transaction-output-copy-2026-11-05.out:13`，该行后面还有别的分项）。我核了算术：5 × 80346376 + 4 × 585019 + 288 = 404072244，与行 5 的 `copied_before` 逐字节相同（`python3 -I` 算的，输出贴在末节）。即准入之前的照抄 = 普通半 5 遍（落点判前 1 遍 + 固定点 4 轮各 1 遍）+ 每轮 585019 字节 + 288 字节。
- 增长：「E7RESULT name=growth_order quantity=jia_after_patch_copied_before from_tombstone_records=1e5 to_tombstone_records=1e6 lower=41402308 upper=404072244 slope=0.98943」（`research/results/e199-transaction-output-copy-2026-11-05.out:964`）与 1e6 到 1e7 的斜率 0.99892（`:965`）：照抄字节随墓碑记录数近似线性。
- 同一几何上别的臂：「E7RESULT name=arm_bytes point=main_geometry arm=yi2_persistent copied_before=2014312 copied_after=421160 recomputed_before=64160 recomputed_after=132912」（`research/results/e199-transaction-output-copy-2026-11-05.out:7`）；丙（上一版按值交进来）准入之前照抄 0 字节、现算重建同为 64160（`:8`）。

| 候选 | 准入之前每次发布的代价随什么涨 | 在 G0 上大约多少 | 要不要另建计数模型 |
|---|---|---|---|
| B1 今天 | 照抄字节随墓碑、退役、谱系记录与容器数线性涨；扫类随 U、R、S、P × S_c、M、F_n、N_a 涨（格三表）；被拒时再乘 1 + 至多 48 | 照抄 404072244 字节（推；s = 4）；用任务 6553 样本里常见的 s = 2 重算：3 × 80346376 + 2 × 585019 + 288 = 242209454（推）。扫类：G0 的 U = 0、单元区 2501 槽，克隆约 0.4 µs（推：任务 6553 读数 16384 槽 2870 ns 按槽数线性缩，实际不线性），找根头走的 `units` 表几十项，所以 G0 上扫类很小（推），字节类占大头 | 字节类 E199 已有；扫类要另建步数模型（E199 不数） |
| B6（放行路径） | 闸：O(W + h·log N + 盘数 × 层数 + S_c)，另加不改根头下标时的 4U；闸之后：B1 的两道之间那一整段原样在（固定点每轮的全部代价）；放不下的带里走 B1 全部加闸 | 读法甲下「准入」= 闸，闸本身在 G0 上可忽略；闸之后 s = 4 时照抄 404072244 − 80346376 = 323725868（推，占 B1 的 80.1%），s = 2 时 161863078（占 66.8%）；现算重建 1814496 一字不省。读法乙：只省这一遍 | 要：放行率（取决于池满度分布与带宽，带宽见 1.3，量级 0.2%–0.25% 盘容量，是条件式）与带内剩余扫类；还要把 1.3 的上界引理穷举验证 |
| B7 | 重放 r 轮 × (W + 释放数) × log R；Δ·h·log N；加格二 2.2 表里字面没点名的那几步：inode 叶容器整份拷（封顶 4.4 MB 每份）、F_n、S_c、N_a、版本侧两张全索引（M）、回落扫（S） | 照抄字节上限 4 × 585019 + 288 = 2340364（s = 4）、s = 2 时 1170326（推：把普通半 5 遍全去掉、每轮分项一点不去的上限，下限 0）；现算重建约同乙2/丙/丁臂的 64160（推，B7 与这三条臂不是同一个东西：B7 不共享版本间的持久结构）；E199 乙2 臂 2014312 字节、丙臂 0 字节是同量级的参照 | 要：扫类步数模型；视图与克隆等价的差分检验；字面没点名的那几步改不改是 B7 自己的定义问题 |

读这张表要带上的三条：
- B6 在 G0 上省的比例只由 s 定（1/(s + 1)），与 u 无关；剩下的 80% 或 67% 还是随 u 线性涨的。所以 B6 不让「准入之前」不随总数涨，只在读法甲下把这段挪到「准入之后」。
- B7 的数是上限：5 遍普通半能去掉多少取决于容器按引用读做到什么程度；我没有把每轮 585019 字节分到格三的哪几行（E199 行 13 在 `settle_round` 之后还有分项，我没核每个分项的定义）。
- E199 G0 的 s = 4 比任务 6553 样本里常见的 2 大，G0 的单元区只有 2501 槽（任务 6553 量的单元区是 16384 到 16777216 槽）。把 B1 和 B7 比的话，扫类在大盘上会反超字节类，这一点 E199 的 G0 看不到，需要另一个几何。

推翻这一格的现象：E199 的产物行 5 与行 13 的分项相加不等于 `copied_before`（我核过相等）；或 B7 的实现里普通半仍整份拷容器（则上限不成立）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 格一 B6 改点 | 列出 | 闸装在 `transaction.rs:12489`，跳过 12510–12523，式子判（12621）二选一；闸要先做 B7 的根头下标与聚簇段计数两件 |
| 格一 B6 读法乙 | 什么都没省 | 保留精确式子判则固定点之后才有读数，只少一遍普通半 |
| 格一 B6 读法甲 | 改 D28 已定项 1 两句（`28-挂载期承诺量.md:35`、`:38`） | 「准入」成了上界闸，时序一句被改 |
| 格一 上界：码 2 树（记账、deadlist、中央映射） | 真上界（读，没穷举），D、I 给得住 | 与 `admission.rs:1484` 同一个引理；记账 D、I ≤ 3 + 6 × 盘数，台阶行不进；中央映射的 I 随 W 涨，ckpt_cost 的中央映射项不能顶 |
| 格一 上界：分配记录树 | 只能取 K1（无条件） | 真上界但宽：0.2%–0.25% 盘容量的带（任务 6265，条件式） |
| 格一 上界：三棵容器树 | 推；已有谱系一项在最坏边上少 1 个节点 | 新开容器且最右边每层都满时是 H + 2 个索引节点（`container_tree.rs:2358` 的注释），ckpt_cost 谱系式子是 H + 1 |
| 格一 上界：ckpt_cost | 读数一侧，要 ≥ 精确值 | 记账项换闭式即可；它不是这次发布固定点的上界（任务 6394：29 对 43） |
| 格一 上界：extent、inode | inode 读、extent 沿用第一轮 | extent 没复核 |
| 格一 回落次序与种类 | 相同，条件是上界真取大 | 不成立的唯一出口是上界偏小 |
| 格一 免责原文 | 不在「装单元、拷字节」之内 | 规划、预演里的分配、仍能拒的几步都不是；与 B4 同一处条款 |
| 格二 B7 改点 | 列出 | 九行；身份换名触 14 份源文件 98 行；容器按引用读碰 Verus 规格 |
| 格二 还剩的步 | 8 类 | inode 叶封顶、聚簇段求和、回落与开段扫、extent/其余文件、版本侧索引、分配记录树自己的计划、差量前提无断言、实例表链；另有抬 F 乘数 |
| 格二 G1 在 B7 下 | 现象还在，不构成反例 | 今天就是每轮从新克隆重放；G1 打中的是读法乙 |
| 格二 视图与本体不一致 | 固定点自己看不见；最坏观察者是真发取落点（`transaction.rs:14819`，分配器已被真释放改过） | 槽号不同而叶集合相同时三处都不叫 |
| 格三 清单完整性 | 不完整，补三条 | 回落扫 `allocator.rs:1041`、实例表链、抬 F 预演乘数 |
| 格四 数能不能核 | 存在性够；频率、带宽、R 的依赖不够 | 6265 的 9975/1139 是模型生成器的频率；k1_band 带宽依赖设的 17 / 25；6553 的 R 没取样、挤动只在 1/20 次调用里量到 |
| 格五 代价量级 | B1 G0 照抄 404072244（s=4），B6 323725868，B7 ≤ 2340364（推） | 三条都要扫类步数模型；B6 还要放行率 |

## 核过的命令与原样输出

- 身份变体命中数（冻结副本 crates 下）：`grep -rcE 'MappingTreeNodeBelowTheRoot|AccountingTreeNodeBelowTheRoot|DeadlistTreeNodeBelowTheRoot' --include=*.rs . | awk -F: '$2>0{n++; s+=$2} END{print "files="n, "lines="s}'` → `files=14 lines=98`。
- 「准入之后」在 kb 与规则里的命中：`grep -n '准入之后' .claude/kb/decisions/08-核心索引结构.md` 只命中 516 一行；`grep -c '准入之后' .claude/rules/fs-design.md` → `0`。
- E199 行 5 与行 13 的算术：`python3 -I -c "a=80346376;b=585019;o=288; print(5*a+4*b+o)"` → `404072244`；`1814496/64160` → `28.28`；`135*233` → `31455`，`135*233*140` → `4403700`。
- 任务日志：`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-6265.attempt-1.log`（21 行）、`…-6394.attempt-1.log`（213 行）、`…-6553.attempt-1.log`（72 行）；上面表里引的行逐字在其中。

## 要主 agent 跑的命令

没有可整条给出的命令：本报告没写模型、没起脚本。下面两项需要新派同立场的腿先写模型，写好后才有命令：
1. 码 2 树引理的穷举验证（1.3）：容量取最小（叶 2、内部 2，再加 3、4 两档），穷举可达的上一版形状与所有删集、插集，比 `code_two_tree.rs:637` 规划器的重写节点数与 `admission.rs:1484` 式子；红了就是引理不成立。这是该腿自己的模型（与实现不共用代码），或在副本里写一个调规划器的测试目标（后者起 cargo，要主 agent 经服务交）。
2. 步数模型（格五）：数格三表里每一步在 G0 与大盘几何上的执行次数，B1、B6、B7 各一列；没有这张模型，三条候选的扫类代价都只有标「推」的阶。

## 没做什么

- 不替主 agent 采纳或出判决，不下「选哪条」的结论；没往统一任务服务交任务，判决要引的数没复跑（上一节说明了为什么没有整条命令）。
- 没写模型、没量任何用时与内存；本报告里「推」的数是按读到的字段与调用次数推的，G0 的字节数是 E199 计数模型产物行里的算术，不是实现里量的。
- 没复核：extent 树的上界式子（沿用第一轮 A3.2）、`remove_record_from_leaf_containers` 的内部、每个 `allocator` 调用方是不是都在失败时换回分配器（`transaction.rs:14823` 的注释）、E199 行 13 各分项的定义。
- 没对 12409 到 12621 之间的每个被调函数做机械的调用图展开，清单是顺着调用链读出来的。
- 没读禁读清单（第三轮攻方腿的报告与模型）。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r3-forward` 里只放过 `log6394.txt`（`task log` 连不上服务的报错，两行，没有用），已删（目录此刻 16K，只剩认领留下的两个标记文件）；没建仓副本与编译目录。
