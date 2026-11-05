# m4-admission-delta-r2 辩方腿报告（Sonnet，2026-11-05）

腿：三方推论辩方腿，一轮一条。复核第一轮判决 `research/prompts/m4-admission-delta-r1-main-verification.md` 第二节 A3、A4 两格够不够得着、是不是同样打中正文第三节的 B2 至 B5，并逐格答正文第四节格一至格六。输入：`research/prompts/_m4-admission-delta-r2-body.md`。没读禁读清单里的两样（`research/prompts/m4-admission-delta-r2-opus-output.md`、`research/prompts/m4-admission-delta-r2-opus-model/`）。

约定：
- 代码行号取冻结副本 `/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/`（下称冻结副本，引文里写绝对路径；表里只写文件名加行号的，指冻结副本里的同名文件）。正文第二节写明主工作区之后的改动没碰准入那一段，我没再 `cmp`。
- 标注：「读」= 读冻结副本源码或 kb 所得，可 `grep -nF` 复核，没运行；「推」= 由读到的事实推的，没跑；「任务 N 的读数」= 我用 `task-server/task log N --all` 只读到的原样行（没交任务）。
- 没写模型（派发没给模型目录），没交任务，没有文件要报 sha256。要主 agent 跑的命令在「要主 agent 跑的命令」一节。
- 新名字 B6：我替 A3 写的变体，不在正文第三节的表里（第三节细写）。它与 B5 的关系在第三节末。
- 符号：Δ = 这一次进出的 key、记录或角色数，h = 树高，N = 某棵树或某张表的总数，R = 分配器 records 总数。

## 一、一眼表

判定三档：站得住 / 够不着 / 同样打中别的臂。同一格可以同时是两档（对 A3 本身站得住、对 A3 一族够不着）。

| 判决条目 | 辩方判定 | 证据（细节在后面各节） |
|---|---|---|
| A3 打中之一：改了「拒成哪一种」（攻方 H7、第二次抽样 H1、任务 6394） | 对「式子判挪到固定点之前、没有回落」的 A3：站得住。对 A3 一族：够不着。不打中 B2、B3、B4；B5 只在它的上界是真上界时才不被打中 | 无回落的上界判决不了排在式子之前的几种拒（要精确角色）；带回落的变体次序与今天逐个相同（第三节 B6 的逐类表） |
| A3 打中之二：字面式子「key 数 × 树高」不是上界（攻方 H3、任务 6265） | 对字面式子：站得住（存在性由确定性用例 `full_root_leaf_one_insert` 坐实，不靠扫描）。对 A3 的定义：够不着（定义写的是「之类的式子」）。「修正式子 0 次超过」撑不起「它是真上界」 | 代码里已有逐层带切分与长高的式子 `admission.rs:1484`；先删后插的规划器读码推出修正式子成立（推） |
| A3 打中之三：分配记录树能拿准的上界只有 K1（攻方 H2、第二次抽样 H3 后半） | 「K1 是无条件上界」：站得住。「A3 因此错拒一带」：只对无回落的 A3。「不判 K1 就回到固定点在准入之前」：对 B6 不成立 | K1 注释 `admission.rs:1193`；任务 6265 的 `k1_band` 四行里的 `circular_trail` 是模型的迭代，不是真历史 |
| A3 打中之四：记录条数按 ckpt_cost 上界数会拒掉今天照发的写（第二次抽样 H2） | 对该变体站得住，B6 不中（条数只当快路条件，不够就回落） | 任务 6394 读数：`records_under_the_bound=95 record_limit=85 today_outcome=published` |
| A3 乙与 D28 已定项 1 两句字面冲突（第二次抽样 H5） | 对「上界即准入」站得住；B5 若读成「上界即准入」同样打中，若保留精确判则不冲突但省不下什么（格四） | `28-挂载期承诺量.md:35`、`:38` |
| A4 只换表示 | 站得住：对「不改规划算法」。不打中 B2、B3 的定义（它们改算法）；但 B2、B3 欠一笔下游消费者的账，第一轮没人看过 | 任务 6265 的 `one_key_publish_output_size` 行；规划器 `code_two_tree.rs:651`、`:653` |
| 「A3、A4 出局」这个结论本身 | A3：出局只对无回落的定义；一族没出局，但 B6 的收益只在放行路径（第三节）。A4：作为单独一条出局，作为 B2、B3、B4 的零件不出局 | 第三、四节 |

## 二、第一关：A3 打中的依据，逐条看够不够得着

先把今天的次序钉死（读，冻结副本 `/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/`）：
- 落点判「admit_the_units_landing_on_every_device(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12523`），在它之前还有一个会拒的步：普通分配那一半「let ordinary_allocation_roles = plan」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12510`）后面带 `?`。
- 固定点「let settled = settle_the_allocation_record_tree(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12536`），多层码 2 树拒、释放判定错、取不到落点（`PlacementRefused`）、64 轮不收口，都在这里交回。
- 记录条数「refuse_a_publish_of_more_journal_records_than_the_limit(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12554`）；合取 7「admit_the_empty_cluster_segments_floor_on_every_device(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12577`）；式子判「admit_on_every_device(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12621`）。
- 会话只对两种拒推抬 F：「PublishError::SpaceAdmissionRefused(_) | PublishError::PlacementRefused { .. } => true,」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/mounted_session.rs:1873`）；记录超上限与合取 7 原样交回：「| PublishError::JournalRecordsOfThePublishExceedTheLimit { .. }」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/mounted_session.rs:1895`）、「PublishError::EmptyClusterSegmentsBelowTheFloor(_)」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/mounted_session.rs:1876`）。

### 2.1 次序（攻方 H7、第二次抽样 H1、任务 6394）

依据撑住什么：把式子判挪到固定点之前、挪走之后不再回头做精确判的 A3，必然换掉「拒成哪一种」。结构上的理由比那一条历史更强：记录条数、固定点的几种拒要精确角色才判得出，上界决不了它们谁先谁后，所以「上界判在前、没有回落」的定义不可能保住次序。第一轮判决把它写成「现量坐实」，我读任务 6394 的原样行，坐实的比判决写的少一截：

- 今天（任务 6394 的读数）：「outcome=refused_publish cause=journal_records_exceed_the_limit(86>85) floor_raises=0 images_unchanged=true」。
- 套上 A3 模拟之后（同一任务）：「outcome=refused_no_space_after_raising_the_floor last_refusal=space_admission_refused floor_raises=0 images_unchanged=true floor_before=CheckpointTxg(0) floor_after=CheckpointTxg(0)」，以及「patched test a_write_past_the_journal_limit_on_a_pool_short_by_the_formula_is_refused_for_its_records_without_raising_the_floor ... FAILED」。
- 量到的只是拒绝的种类变了（调用方从「把改动切小」变成「腾空间」）。判决与攻方写的「会话推抬 F、F 抬了、回退候选少了、盘上多空发布」这一截，这条历史上没发生：`floor_raises=0`、盘不变、F 前后同为 0（池刚建好，defer 里没有可回收的槽，推不出抬 F）。那一截只在 defer 里有可回收槽的池上成立，是推的，没量。
- 所以这条依据够得着「A3 违反正文第一节『拒成哪一种与今天相同』」，够不着「A3 写盘」。推翻本条的现象：找到一个无回落的 A3 写法，在所有包含「精确角色才判得出的拒」的历史上与今天逐类相同——我不信存在（上界 ≥ 精确量，拒的先后决不了）。

够不着的地方：这条只对「没有回落」成立。一个「上界放得下才走快路、放不下整条回今天的路」的变体，次序与今天逐个相同（第三节逐类表）。

打中 B2 至 B5 吗：B2、B3 规划仍在两道判之前，次序没动，不中。B4 两道之间照今天，不中。B5 定义写明「上界放不下才走今天的全量规划与两道判」「拒绝的次序与种类保持今天的样子」，次序靠回落保住，不中；条件是它的上界是真上界，否则快路上放行了真实不够的写（见 2.2）。

### 2.2 字面式子不是上界（攻方 H3、任务 6265）

- 存在性是确定性的：任务 6265 的读数「name=full_root_leaf_one_insert leaf_entries=255 height_before=1 height_after=2 rewritten=3 literal_bound=1」（leaf_entries 取 1、2、3、8、64、255 各一行，都是 rewritten=3 literal_bound=1）。这一条不靠扫描，站得住。
- 扫描行「name=keys_times_height_sweep capacities=5 histories_per_capacity=5 publishes_per_history=400 publishes_judged=9975 literal_bound_exceeded=1139 largest_excess_over_the_literal_bound=28 corrected_bound_exceeded=0」：5 种容量（叶、内部都取 2 到 16 的小数）× 5 条历史是线性同余造的随机历史加一条升序，不是全体可达形状。它撑「字面式子常被超过」（1139 次是这个取样分布下的频数，不是发生率），撑不起「修正式子是真上界」：0 次只是取样没碰到，攻方自己的模型注释也写「推的式子，只打出违反的次数，不断言」（`research/prompts/m4-admission-delta-r1-opus-model/planner_bound.rs` 第 5 行）。
- A3 的定义（`research/prompts/_m4-admission-delta-r1-body.md:38`）把式子写成「这一次进出的 key 数 × 树高」之类，「之类」没把式子钉死。代码里已有一族逐层带切分与长高的式子：「fn central_mapping_tree_nodes_an_empty_publish_rewrites_at_most(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:1484`），按层取 min(这一层节点数, 删加插) 再加切的次数与长高，用户 2026-09-27 定按最坏计。H3 本身也写了「这一格不致命」。
- 修正式子成立的读码推导（推，没跑）：规划器先删后插，「for deleted in previous_keys.difference(keys_after_this_publish) {」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/code_two_tree.rs:656`）、「for inserted in keys_after_this_publish.difference(&previous_keys) {」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/code_two_tree.rs:659`）。删只重写路径（≤ 删之前的高 h_前 个）、摘空与降高不新写节点；插重写路径（≤ 插时的高）加每层至多切出一个右半、根切开再多一个新根，所以一把插至多 2·h_后 个。先删后插保证插的阶段树高只增不减，h_后 就是插阶段的最高。攻方的式子「插的 key 数 × 2 × 之后的高 + 删的 key 数 × 之前的高」于是成立。推翻它的现象：任何一段历史上规划器的 `rewritten_positions().len()` 超过这个式子；最便宜的找法是容量取 2 的穷举（所有可达的上一版形状 × 所有进出集合），不是随机取样。
- 打中 B2 至 B5 吗：B2、B3、B4 不用上界，不中。B5 的上界若取字面式子则同样中，但 B5 在快路上做全量规划并断言，上界不真时断言响，不会静默放行真实不够的写（这是 B5 比无断言 A3 多出的一层）。

### 2.3 分配记录树只有 K1（攻方 H2、第二次抽样 H3 后半）

- 「拿得准的上界只有整棵树」是 kb 自己写的：「拿得准的上界只有整棵树」（`.claude/kb/decisions/28-挂载期承诺量.md:117`），代码注释同句「拿得准的上界只有几何上整棵树」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:1193`）。这一句站得住。
- 攻方 H2 的「循环」靠模型 `k1_band.py` 的迭代，任务 6265 的读数「circular_trail=[12, 48, 156, 480, 529, 529, 529, ...」是一个「每个新写的节点再引来 h 倍的记录变化」的玩具递推，假设每个重写的节点把自己的新旧副本都散到新叶里。真分配器的提交内生块从开放段 bump：「提交内生块：从开放段 bump；容器按 32768 对齐档，节点取游标处最低空槽」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/allocator.rs:2433`），这一次发布新写的角色连续落在开放段里，新副本的记录挤在很少几片叶里（读），旧副本才可能散。所以递推给出的 529 是封顶值，不是历史；循环在无假设的意义上真，不意味着真历史上会走到封顶。
- 后果因臂而异：无回落的 A3 里 K1 变成错拒一带（任务 6265 的 `k1_band` 四行：两块 1 TiB 盘「band_per_device_mib=2595.8 band_share_of_device_percent=0.248」，这个宽度是假设每盘实际只改 25 个节点，推的）；B6 或 B5 里同一带只是「快路用不上、回今天的路」，没有错拒。代价落在放行路径之外，见第三节。
- 打中 B2 至 B5 吗：B2、B3 的固定点算精确值，不需要上界，不中（它们的难处是格一）。B4 不中。B5 中在「带内快路不可用」，不是中在正确性。

### 2.4 其余两条依据

- 按 ckpt_cost 数记录条数（第二次抽样 H2）：任务 6394 的读数「name=bounded_journal_count checkpoint_cost=664 named_entries_per_record=64 records_under_the_bound=95 record_limit=85 today_outcome=published floor_raises=0」。对「把记录条数硬挪到准入之前并按上界拒」的变体站得住；B6 只把条数当快路条件，不够就回落到今天的路，今天说 85 条照发，B6 也照发。
- A3 乙与 D28 两句冲突：「发布路径在 `transaction::prepare_the_version_publish` 里判：算定这次发布的形状之后、读盘核与动分配器之前。」里的「算定这次发布的形状之后、读盘核与动分配器之前」（`.claude/kb/decisions/28-挂载期承诺量.md:35`）与「需求 = 这次发布新写的全部槽」（`.claude/kb/decisions/28-挂载期承诺量.md:38`）。对「上界即准入」站得住；对保留精确判的写法不冲突（格四把这件事摊开）。
- 第一轮把 ckpt_cost 当上界的批评（第二次抽样 H3 前半）：欠账表自己写着「ckpt_cost 不是这一次的上界」（`.claude/kb/checks-owed.md:348`），今天的落点判已经带着这个洞（C573，条款没定，写前三种拒都在写之前）。这是 A3 与今天共有的，不能记在 A3 账上；它只说明 B6 的快路不能用 ckpt_cost 顶固定点，要用写入类发布自己的上界。

### 2.5 第一关小结：打中 B2 至 B5 的矩阵

| 依据 | 无回落 A3 | B2 | B3 | B4 | B5 |
|---|---|---|---|---|---|
| 次序变了（H7、H1、6394） | 中 | 不中 | 不中 | 不中 | 不中（靠回落，前提是上界真） |
| 字面式子不是上界（H3、6265） | 中 | 不用上界 | 不用上界 | 不用上界 | 取字面式子才中；有断言兜底 |
| 只有 K1（H2） | 错拒一带 | 不用上界 | 不用上界 | 不用上界 | 带内快路不可用，不错拒 |
| 记录条数按上界（s2 H2） | 中（该变体） | 不中 | 不中 | 不中 | 不中（回落） |
| D28 两句冲突（s2 H5） | 中（A3 乙） | 不中 | 不中 | 改的是 D8 射程那一句，不碰这两句 | 读成「上界即准入」才中 |
| 记账树台阶行（共用的洞） | 中 | 中 | 中 | 中 | 中（快路上 ckpt_cost 的读数仍扫台阶行） |
| 分配器每轮整份克隆、全记录分叶（H4） | 挪到准入之后 | 定义里罩住 | 罩住 | 不罩 | 快路上挪到准入之后；慢路不罩 |

## 三、替 A3 辩护：B6「上界快路加回落」

### 3.1 定义

在 `judge_the_publish_before_any_read_or_write` 最前（`ordinary_allocation_part` 之前）算四个上界，纯函数、不改任何状态：
1. U_普通：落点判要的两个数（数据单元数、提交内生块的槽数），按盘。
2. U_固定点：分配记录树取 K1（无条件上界）；中央映射树取 `admission.rs:1484` 那一族逐层式子，删插数换成这次换下与新写的角色数；记账树取闭式（格五）；谱系、退役沿用 ckpt_cost 里现成的两项；树表 1。
3. U_记录：这次切成几条 journal 记录的上界。
4. U_预留：已承诺预留的上界；写入类发布不重写墓碑树与退役树，读上一版根头与按差量维护的 `InodeRecordCounts`（推；第二次抽样「没打中的形状」同判）。

快路条件，全成立才走：发布不是删除类、不是删快照；落点判在（U_普通，ckpt_cost）上过；U_记录不超上限；式子判在（U_普通 + U_固定点，U_预留）上过。

走快路：不算精确的普通半、不做 12523 与 12621 两道的精确读数；12536 起的固定点、记录条数、合取 7 一字不改地照旧跑；式子判的位置换成断言「精确需求 ≤ 快路用的上界」。不走快路：今天的整条路，一字不改。

### 3.2 次序与种类与今天逐个相同（逐类）

判的单调性：两道判都是「可用小于需求才拒」。式子判「(available.0 < i128::from(demand.0)).then_some(DeviceShortOfDemand {」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:904`），落点判「if room_on_the_device.slot_pairs_for_data_units < units.data_units {」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:1080`）。可用不依赖需求，所以需求换成不小于它的上界，过了就意味着精确值也过。

| 今天的拒 | 在今天的位置 | B6 快路上 | B6 慢路上 |
|---|---|---|---|
| 普通半的拒（`InodeTreeWriteRefused` 等，`?`） | 12510，排在一切判之前 | 放行之后随固定点交回；今天排在落点判之前，但快路上落点判必过，两者之间没有可见的先后差别 | 照旧 |
| 落点判拒（推抬 F） | 12523 | 快路条件已保证不拒 | 照旧 |
| 固定点里的拒：多层码 2 树拒、释放判定错、`PlacementRefused`（推抬 F）、64 轮不收口 | 12536 | 放行之后一字不改；今天它们本来排在式子判之前、没被任何空间拒抢先 | 照旧 |
| 记录超上限（不推抬 F） | 12554 | 快路条件保证不拒 | 照旧 |
| 合取 7（不推抬 F） | 12577 | 只读分配器计数，位置不动 | 照旧 |
| 式子判拒（推抬 F） | 12621 | 快路条件保证不拒，位置换断言 | 照旧 |

全部成立的前提只有一个：上界真。除此之外每一类的先后与种类，在快路上要么「保证不发生」要么「位置不动」，在慢路上就是今天的代码。快路本身不写盘、不动分配器，所以没有副作用要核对。推翻它的现象：找到一段历史，快路条件成立而精确的落点判、记录条数或式子判在今天的代码上会拒；或找到一种今天的拒不在上表里。

### 3.3 上界是真上界吗（逐项，诚实标）

| 项 | 状态 | 依据 |
|---|---|---|
| 分配记录树 K1 | 读：无条件 | `admission.rs:1193` 的注释与 `28-挂载期承诺量.md:117` |
| 码 2 树（记账、deadlist、中央映射）的重写节点数 | 推：2.2 的读码推导 | 先删后插、路径加切分加新根；尚无穷举验证 |
| 中央映射树的删插数 | 推：删 ≤ 这次换下的角色数，插 ≤ 这次新写的角色数（含 K1 个分配记录树节点） | `admission.rs:1260` 起 ckpt_cost 的同一种传递 |
| 普通半（inode 树、extent 树、墓碑、谱系、退役、logged_ops） | 推：第一轮正推 A3.2 表给了每棵树的式子（`research/prompts/m4-admission-delta-r1-sonnet-output.md:159`），我没逐棵复核 | 同上 |
| 已承诺预留 | 推 | 3.1 第 4 项 |
| 记账树闭式 | 推 | 格五 |

没有一项是量过的「真上界」。这是 B6 当前最弱的地方：第一轮对 A3 的攻击没有谁去证伪这些式子，我也没有。便宜的证伪法：对每棵树，容量取最小（叶 2、内部 2）穷举所有可达的上一版形状与所有进出集合，比规划器的重写数；这一条交新派的腿做（「要主 agent 跑的命令」一节）。

### 3.4 B6 没做到的（不替它回避）

1. 只救放行路径。拒绝路径（包括会话推抬 F 之后的重判，一次准入至多 8 次发布，`.claude/kb/decisions/16-发布语义.md` 与 C283 的口径）与「上界放不下而精确放得下」那一带，整条走今天的路，准入之前仍随总数涨。要让拒绝路径也不涨，必须把「谁先拒」的那几个条件也按差量判出来，那就是 B2、B3 的活，到那一步 A3 的上界就没有存在的理由。
2. 带的宽度：至少是 K1 减去这次真改的分配记录树节点数。任务 6265 的读数「name=k1_band geometry=2x4GiB ... k1=529 ... band_per_device_mib=8.0 band_share_of_device_percent=0.195」与「geometry=2x1TiB ... k1=166159 ... band_per_device_mib=2595.8」（带宽是假设每盘实际改 17 或 25 个节点，推的）；中央映射树那一项的上界还会再加一份，B6 的带比这宽。今天的保留池里已有一份 K1，带相当于再扣一份，约盘的 0.25% 到 0.5%（推）。
3. 快路之后的全量规划与固定点仍在，总开销没降，只是排到「准入之后」。D8 射程这句免责的原文是「准入之后装单元、拷字节不算运行时决策路径」（`.claude/kb/decisions/08-核心索引结构.md:516`），字面只豁免装单元与拷字节，没有豁免规划与固定点；快路放行之后还要在那一段里做三种拒的决定（树拒、取不到落点、记录超上限）。B6 因此和 B4 一样要用户确认免责的口径比字面宽。B2、B3 不需要这个确认。
4. 式子判降成断言就改了 D28 已定项 1 的两句（`:35`、`:38`）；保留精确判不改条款，但那样准入之前的全量规划一步没省（格四）。
5. 记账树台阶行那个共用的洞（格五）不修，快路上式子判的读数里 ckpt_cost 照样扫台阶行：「.filter(|entry| entry.generation == version_txg)」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:1339`）与「.chain(version_to_build_on.inode_number_watermark_steps())」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:1349`）。

### 3.5 与 B5 的关系

B5 的定义写「准入先判一个按差量算的上界，上界放不下才走今天的全量规划与两道判；上界放得下时照样做全量规划，用断言核全量结果不超上界。拒绝的次序与种类保持今天的样子」。B6 就是把「照样做全量规划」排在上界判之后（放行之后），并把快路条件写全。B5 若读成「全量规划排在上界判之前」，它与今天一样，什么都不省；读成「排在之后」，它就是 B6。这一歧义是格四的核心。

结论（对第一关）：A3 一族有一个次序与今天逐个相同的写法（B6），它的上界项除 K1 外全是推的；第一轮打中 A3 的依据，次序一条对无回落的 A3 站得住，其余几条或者只对字面式子、或者只对错拒一带，够不着 B6。B6 的收益限于放行路径，且依赖用户确认免责口径比字面宽。

## 四、第二关：A4 只换表示

依据：规划器收全量 key、深拷整棵上一版、交回整棵新形状，与上一版怎么表示无关（攻方 H6、第一轮正推 A4 表、任务 6265）。

- 读：规划器入口把上一版全部 key 收成集合「previous.keys_in_order().into_iter().cloned().collect();」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/code_two_tree.rs:651`），把上一版整棵建成规划树「Some(previous_root) => planning_node_of(previous, previous_root.position),」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/code_two_tree.rs:653`），换下的集合再扫上一版全部节点「let replaced_previous_nodes = previous」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/code_two_tree.rs:685`）。
- 任务 6265 的读数：「name=one_key_publish_output_size total_keys=100000 height=4 nodes_before=3124 nodes_in_the_returned_shape=3124 origins_returned=3124 rewritten=4 replaced=4 carried_at_a_new_position=0 input_key_set=100001」。一把 key 的发布，输入集合 100001 把、交回 3124 个节点，真重写 4 个。
- 判定：站得住，对 A4 的定义「不改规划算法」。换乙、丙、丁哪种表示，入口还是要全量 key，出口还是整棵形状。第一轮正推另报的丙与固定点每轮复用上一版不相容（`research/prompts/m4-admission-delta-r1-sonnet-output.md:219`），只对丙。
- 够不着的地方：A4 作为零件。第一轮正推列的「换表示确实能去掉的」那几行（`research/prompts/m4-admission-delta-r1-sonnet-output.md:217`）是拷类，B4 要把落点判前的拷类去掉，借用与按引用就是 A4 乙那一路；A4 的出局只说它不能单独成立，不说它不能当零件。
- 打中 B2、B3 吗：6265 量的是今天那个规划器的入口与出口，B2、B3 的定义恰好要换掉它，所以不中。但这里有一笔第一轮没人看过的账，不是打中，是没看过：B2 的新规划器出口是什么，下游吃不吃得下。读上一版这一侧不需要换表示：形状是按位置排好序的 Vec，按位置取节点是二分「.binary_search_by_key(&position, |node| node.position)」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/code_two_tree.rs:223`），逐层数节点已经是二分「let first = nodes.partition_point(|node| node.position.level < level);」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:1447`），今天就有 O(h·log N) 的先例。难在出口：`CodeTwoTreePlan` 的形状、来源、换下集合三样都是整版序列，第一轮正推表二 #25 至 #28 列的下游（`resolve` 里的映射 key、容器树的 `node_origins`、释放判定的两张全索引）都吃它（转述，我没逐项复核）。B2、B3 要先说清「只交差量」之后这几处吃什么，否则 A4 打中的那条根（产物是整版序列）会在它们的下游再出现一次。
- 推翻本条的现象：给出一个不改规划器算法、只换表示、准入之前一把 key 的发布不再收全量 key 的实现。我不信存在。

## 五、格一至格六

### 格一　B2 的固定点做得成差量吗

判定：做得成的前提三条，今天的代码一条都没有担保；L1 两处说法都对，各答不同的问题。

固定点今天怎么走（读）：每一轮拿「重写集」和「这一版的节点」算分配记录树的计划，跑一遍 `resolve`，在分配器的克隆上放掉换下的、取这次所有角色的落点，按走出来的记录判哪些叶变了；「for _round in 0..ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT {」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:13053`）、「let mut rehearsal = allocator.clone();」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:13098`）、「let changed_by_the_release =」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:13106`）、「let holding_records = nodes_holding_records(&geometry, rehearsal.records());」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:13121`）、「rewritten.extend(changed);」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:13130`）。上限「pub const ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT: usize = 64;」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:13141`）。

收敛到同一个固定点的论证（推）：每一轮的状态只有两个集合（重写集、这一版的节点），下一轮由「变了的叶」「装着记录的节点」两个函数决定；这两个函数的值只取决于记录的差。若把「全部记录分叶再比」换成「只比被动过的记录所在的叶」，对每一轮得到同一对集合，轮序列相同，结局与轮数都相同。前提：

1. 进入固定点时分配器里的记录与上一版的 `allocation_records` 是同一个集合。今天的代码对比的是 `previous_records`（「let changed_by_the_release =」那一段取的就是它），不要求两者相等；进入时没有一处断言它们相等，只有真发之后那一句核「准入之前在分配器拷贝上走到的固定点，与真发时取完落点的记录说的是同一棵树」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:15225`）。差量版只看被动过的叶，两边若有一处内存与上一版不一致，今天会把那片叶算进重写集，差量版漏掉。
2. 叶是否还装着记录要有出处。记录在重放里除了翻已释放还会被删：「fn make_room_for_record_on_device(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/allocator.rs:1144`）里对相交的已回收记录 `retain` 删掉。叶空了节点就不在了，所以差量版要么随版本带每叶记录计数，要么靠上一版记录按 (设备, 槽) 排好序的事实二分取叶的切片（真发时「allocation_records.sort_by_key(AllocationRecord::sort_key);」，`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:15205`）；新一版保存的 `allocation_records` 是不是这份排好序的我没追到出口，要核）。
3. 每一轮里 B2 的定义没罩到的行。每一轮都整份跑 `resolve`，里面的普通半、deadlist 条目索引、三棵容器树的 `node_origins`、释放判定的两张全索引、记账行表，第一轮正推表二 #22 至 #31 都列了（转述），B2 的定义写的是「多层码 2 树与三棵容器树的规划」，没有写这些。固定点按差量，`resolve` 不按差量，整一轮还是 O(N)。
4. 两份固定点：除了 `settle_the_allocation_record_tree`，树表 0 条那一版上写行也有一份同形的「for _round in 0..ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT {」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:1907`），要同步改。

轮数上限 64 照旧够不够：轮序列不变所以轮数不变，「够不够」是分配器行为的事，B2 不改变它。代码注释写「产品路径上三四轮就定」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:13138`），没有人造过逼近 64 的历史。我的推：轮数约等于释放链的长度加二，因为每一轮并进来的是上一轮重写的节点的旧副本所在的叶。

L1（W 的每层 2·(K + 26) 与 K1 那句「收不了口」到底哪一处对）：都对，答的是两个问题。
- K1 那句：「一次发布改几片叶的不动点收不了口」，无条件，不依赖历史形状；它点名的第三种情形是换下很久以前落在别处的节点（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:1193` 那一段注释）。站得住。
- W 的 2·(K + 26)：D28 写成「W 是一次删除类发布每盘新占槽数的上界」（`.claude/kb/decisions/28-挂载期承诺量.md:31`），是删除批的经验式。出处我从 git 里取回了（`31128273b^:research/prompts/m3-delete-r3-sonnet-output.md`，取回的副本 `/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r2-defense/m3-delete-r3-sonnet.md`，sha256 53770c875e7cd27f9d30eebbe9ff9180655d18c3f08496378354ef7737c23900）：转述其第 93 行，字面最坏 `2×min(K+26,262)+5` 在 K = 144 上是 345，造了一单元一叶的散落世界量到 307 至 312，贴近到 81% 至 84%；第 95 行，层数 ≥ 3 的盘「没造出对应的散落世界，推的」。这个式子要成立，隐含一条没写出来的假设：分配记录树自己的节点旧副本落在已进重写集的叶里（落点局部性）。K1 注释的第三种情形否认的正是这条局部性。
- 所以「对不上」是 W 少写了一条假设，不是 K1 过严。没人造出破坏局部性的历史（第二次抽样自己也写了没造出），所以 W 没被证伪，也没被证明。W 不够的后果落在环窗口（删除批的新写兜底），不落在写入类发布的准入上。
- 对 B2 的意义：B2 的固定点算精确值，不依赖这两个上界中的任何一个；它的风险只在上面三条前提与 64 轮。
- 推翻：造出一段历史使删除批新占的分配记录树节点数超过 2·(K + 26)（推翻 W）；或证明局部性是分配器的不变量（推翻我对 W 的评语）。

### 格二　B3 与 B2 差在哪

判定：B3 不是 B2 之外多出来的一档，是 B2 最后一句话缺的那一半。

- B2 写「不克隆整份分配器」，没写怎么在不克隆的前提下知道这次释放与取落点之后记录变成什么样。固定点要的是「这次所有角色会落在哪里、换下的旧副本在哪」，只有跑分配才知道。B3 写的是怎么跑：走差量视图。所以 B2 单独成立不了，B3 补的是机制；备选机制还有两种（在活分配器上记撤销日志再回滚、按盘分块的写时复制），渐近相同。
- 视图要维护什么（读：重放只调用释放与取落点）：释放：每盘把那条记录翻成已释放加释放代（「.find(|record| record.device == device.device && record.slot == placement.slot)」，`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/allocator.rs:2265`）；取走：新记录，或把已回收的同槽记录改写、把相交的已回收记录删掉（`make_room_for_record_on_device`，`allocator.rs:1144`），标已分配位，段内计数（`used_per_segment`、未释放计数、没挡的槽对数）、开放段、bump 游标、这次挂载开过的聚簇段集合。隔离位与抬 F 扣住位重放里只读：重放不调 `release_leaving_the_record_allocated_on`（那是真发读盘核出坏副本才用的），所以「隔离」「预留」两样在视图里不用写，只读底座。
- 与分配器本体不一致时谁先看见：今天只有真发之后的那句断言（「准入之前在分配器拷贝上走到的固定点，与真发时取完落点的记录说的是同一棵树」，`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:15225`），它在真发取完落点之后、发任何写之前，比的是叶的集合，不比落点；视图选了和真分配器不同的槽但叶集合相同，它看不见（也无害，节点集合才进固定点）。要在更早处看见，办法是让真发也走同一个视图类型再提交，只留一份实现；两份实现并存，则「谁先看见」是这句断言，看见的方式是 panic。
- B3 比 B2 多省的是什么（推，按字段类型算）：每一轮克隆里的位图与段计数。每盘每槽三个 `Vec<bool>`（「allocated: Vec<bool>,」`allocator.rs:313`、「isolated: Vec<bool>,」`allocator.rs:336`、「held_until_floor_takes_effect: Vec<bool>,」`allocator.rs:344`）共 3 字节，加五个按段的 `Vec<u64>`（「used_per_segment: Vec<u64>,」`allocator.rs:315` 一族）每 64 槽 40 字节，约 3.625 字节每槽；一块 1 TiB 盘 2^26 槽，约 232 MiB，两块盘每轮约 464 MiB，三四轮约 1.4 至 1.9 GiB。外加全部记录的整份拷贝（「records: Vec<AllocationRecord>,」`allocator.rs:1877`；两块 1 TiB 盘半满、每落点一条每盘，约 3.4e7 条，按每条 32 字节约 1 GiB，条宽是估的）与每轮约 5 趟全记录分叶。
- 视图补不上的：分配器自己的查找是线性的。用户数据取落点从单元区起点一路扫「pub fn lowest_user_data_slot_outside_the_region(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/allocator.rs:965`），循环「while candidate + 1 < end {」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/allocator.rs:973`），每个候选里再扫聚簇段集合「cluster_segments.iter().any(|segment| {」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/allocator.rs:974`）；发布路径对数据单元调它：「PlacementRule::UserData => allocator.try_allocate_user_data(txg),」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:2384`）；释放与让位按记录逐条找（`allocator.rs:2265`、`allocator.rs:1144`）。视图再薄，底座这几条查找不加索引就还是 O(已占前缀)、O(R) 每次。B2、B3 的定义都没写分配器要加索引。第一轮没人列这一步（第一轮正推表二没有它）；E181 的 F11 登记过「量扫描挂钟随单元区的增长」（`.claude/kb/experiments/181-落得下那一判与挂载下界.md:7`，问的是落点判要不要算各盘最低落点再比），那一页第一段「还没跑，原件未留存」（`.claude/kb/experiments/181-落得下那一判与挂载下界.md:15`），没有数。
- 推翻：证明重放里这几条查找在产品路径上被游标命中而不是线性扫（例如有一条我没读到的缓存）；或量出它们在大盘上的用时可忽略。

### 格三　B4 改射程是不是越位

判定：是条款变更，不是实现选择；用户那句话够不着两道之间。

- 原文。D8 已定项 16 射程：「准入之后装单元、拷字节不算运行时决策路径」（`.claude/kb/decisions/08-核心索引结构.md:516`）。变更史：「条款读法「管，只管准入之前」」里的话 `管，只管准入之前`（`.claude/kb/decisions-history.md:3514`）。线划在哪：「线划在「谁消费这个数」」里的 `谁消费这个数`（`.claude/rules/fs-design.md:19`），第一格是「**运行时决策路径**（分配、ENOSPC 准入、defer 窗口、生命周期判定）」（`.claude/rules/fs-design.md:23`）。
- 越位的三条理由：
  1. 免责原文只豁免「装单元、拷字节」。两道之间是规划、扫描、克隆、分组，产物是式子判的需求，由 ENOSPC 准入那一判消费，按 fs-design 自己的判据（谁消费）它们在决策路径上。B4 把它们划到免责里，免责的范围就从「装单元、拷字节」变成「落点判之后的一切」。
  2. 代码里两道都叫准入，拒的是同一种错：落点判「准入先判这次的单元落不落得下」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12506`）并且 `.map_err(PublishError::SpaceAdmissionRefused)?`；式子判同样是 `SpaceAdmissionRefused`。「准入」在 B4 里被读成其中一道，另一道排在「准入之后」。
  3. 主 agent 在第一轮判决已经写了读法：「照字面要连两道准入一起算，这是读法，条款不改」（`research/prompts/m4-admission-delta-r1-main-verification.md:47`）。B4 改的是这个读法。
- 用户 2026-10-31 那句话够不够：不够。变更史里只留了「管，只管准入之前」和排法「五处加准入现在改，根源先量」（`.claude/kb/decisions-history.md:3514`），那一次的问题单与选项我读不到；当时的判决（`research/prompts/m4-close-code-r1-main-verification.md:32`）那一行记的是容器小树的标记表，没有提准入分两道（推：从那一行的字面看，不是看到了弹窗原文）。那句话回答的是「要不要管」，没有回答「两道之间归哪边」。
- 两道之间大盘上的代价量级（推，没量；正文第二节也写主 agent 没量过）：每轮克隆约 464 MiB（两块 1 TiB 盘）加 5 趟全记录分叶加一次整份 `resolve`，三四轮；上限 64 轮。
- B4 与 B6 同类：都要用户确认免责口径比字面宽。B2、B3 不需要。
- 推翻：找到那次弹窗的原问题与选项，若选项里写的「准入」明说是落点判，则本条的第二条理由被削弱（第一条仍在）。

### 格四　B5 的断言与拒绝次序

判定：B5 的写法有两种读法，哪一种都不能同时「省下准入之前的代价」和「不改 D28 两句」。

- 读法一：全量规划排在上界判与精确判之前，与今天一样；上界判只是个前置的快速放行，精确判仍在。准入之前的代价照旧随总数涨，什么都没省，断言只是多一层自检。
- 读法二（B6）：上界判就是准入，全量规划排在它之后，精确判降成断言。准入之前的代价在快路上只随 Δ，省下的是放行路径；式子判不再「算定形状之后」判，需求也不是「新写的全部槽」，改的是 D28 已定项 1 的两句：「算定这次发布的形状之后、读盘核与动分配器之前」（`.claude/kb/decisions/28-挂载期承诺量.md:35`）、「需求 = 这次发布新写的全部槽」（`.claude/kb/decisions/28-挂载期承诺量.md:38`）。
- 上界放不下时：整条走今天的路，拒绝的种类与先后与今天逐个相同（第三节 3.2 的表，慢路列「照旧」）。上界放得下时：快路条件保证落点判、记录条数、式子判三样不拒，剩下的拒（树拒、取不到落点、释放判定错、不收口、合取 7）照今天的先后在放行之后出现，种类相同。
- 断言打中（全量超过上界）时：这是「上界 ≥ 精确量」这条不变量被破坏。`code-discipline.md` 说「不变量被破坏是 bug，用断言」，「不带着坏状态往下跑、更不往盘上写」（`.claude/singlefs-ai-sop/rules/code-discipline.md:150`），按字面是 panic。降级成返回 `SpaceAdmissionRefused` 有两处不合：一是同一份规则把「资源不够」划给 `Result`（`.claude/singlefs-ai-sop/rules/code-discipline.md:145`），降级在这个意义上也说得通，但它把一个 bug 伪装成正常的空间不足；二是会话对 `SpaceAdmissionRefused` 会推抬 F 的空发布（「PublishError::SpaceAdmissionRefused(_) | PublishError::PlacementRefused { .. } => true,」，`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/mounted_session.rs:1873`），也就是因为上界写错了而往盘上写，正对着「更不往盘上写」那半句。仓里的先例：同类的「两份算法必须一致」今天就是 assert（`transaction.rs:15225`）。两条路都不是我选的；要降级的话，用一个不推抬 F 的新错误成员，而不是复用 `SpaceAdmissionRefused`。
- 推翻：找到一个快路条件成立而精确判会拒的历史（推翻「种类相同」）；或用户明说精确判不可降成断言（读法二不成立，只剩读法一，什么都不省）。

### 格五　记账树台阶行

判定：有一种做法，不改 D8 已定项 1。

- D8 已定项 1 管的是记账条目的 value 语义与「每条根只装一代」，它自己已经把台阶行除外：「除 inode 号水位的台阶行随根保留外」（`.claude/kb/decisions/08-核心索引结构.md:32`）。台阶行随根保留是 D5 已定项 2 射程的另一句：「台阶行随根 COW、一直留着」（`.claude/kb/decisions/05-快照-空间记账机制.md:55`），而且「不随每次发布重写」（`.claude/kb/decisions/05-快照-空间记账机制.md:139`）。行多了，每次发布并不重写它们。
- 随行数涨的是 ckpt_cost 里记账树那一项在每次发布（两道判各一次）现算：「.filter(|entry| entry.generation == version_txg)」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:1339`）扫全部记账行，「.chain(version_to_build_on.inode_number_watermark_steps())」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:1349`）把全部台阶行收进集合再跑全量规划器。这一项是 D28 已定项 4 的「记账树不进 Σ，单列，按每发布的节点数」（`.claude/kb/decisions/28-挂载期承诺量.md:118`），是内存里的读数，不是 D8 已定项 1 的东西。
- 两种做法（推）：(一) 精确：记账树的节点数随版本带着，下一次发布的变化只是「这一代 15 行换代加至多 1 行台阶行」，对记账树做一次差量规划（就是 B2 的规划器用在这棵树上），台阶行当照抄的 key 不碰；值与今天逐个相同，不改 D28 已定项 4。(二) 上界：取「这一版节点数 + 16 × (h + 1)」（16 把变化 key，每把至多多 h 个节点加一个新根；推，按 2.2 的推导），多留约几十块，D28 已定项 4 那一项从「节点数」变成「节点数的上界」，要改字。另有同族的小项：`inode_number_watermark` 逐行扫记账行（「pub fn inode_number_watermark(&self) -> u64 {」，`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:4222`），在准入之前被调（「let first_new_inode_number = previous.inode_number_watermark();」，`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:9872`），随版本带一个水位值即可，不碰条款。
- 要改条款的情形：想让台阶行条数自己有上界（例如合并旧台阶行）。那要改的是 D5 已定项 2 射程那一句「台阶行随根 COW、一直留着」和 D8 已定项 1 括号里的「除 inode 号水位的台阶行随根保留外」，还牵到管理员回退「带 cur 的台阶行」的语义，不是计算上的事。
- 推翻：量出做法（一）的差量规划在台阶行跨叶门槛（第 463 级台阶起 +2，admission.rs 注释）那一次与现算不同。

### 格六　第一轮的方法与判定够不够

判定：撑得住「A3、A4 各自被打中的存在性」，撑不住「A3 一族出局」，也撑不住「修正式子是真上界」。

- `keys_times_height_sweep`：只扫 5 种小容量、每种 5 段历史（4 段随机、1 段升序，每段 400 次发布），判了 9975 次（任务 6265 的读数）。作用：给「字面式子被超过」一个频数，但频数只属于这个取样分布。「corrected_bound_exceeded=0」不是修正式子的证明（2.2）。字面式子不是上界的存在性不靠它，靠 `full_root_leaf_one_insert`。更好的方法是容量取 2 的穷举，不是加大随机历史。
- A3 模拟只在一处挪了式子判（任务 6394）：它证明的是「挪了就变」，而且只量到了拒绝种类变（2.1 里 `floor_raises=0`）。结构理由（上界决不了排在式子之前的拒谁先谁后）才是撑「无回落 A3 必变」的东西，模拟只是它的一个实例。模拟用的上界是 ckpt_cost 顶固定点（`research/prompts/m4-admission-delta-r1-opus-s2-model/tools/apply_a3_emulation.py` 里「+ checkpoint_cost_of_the_version_to_build_on_with_node_capacities(」），这个上界本身不真（C573），换成任何更大的真上界，结论只会更强，所以这不削弱存在性。
- 有没有一种 A3 变体次序同今天、上界又是真上界：有，B6（第三节）。上界项除 K1 外都是推的，要靠穷举模型补量。
- 推翻：对 B6 的快路条件找到一个历史，使快路放行而今天的代码会拒（落点判、记录条数、式子判三样之一）。

## 六、每条候选一行代价量级（全是推，没量）

符号见开头。两块 1 TiB 盘、半满的取值：克隆每轮约 464 MiB，记录约 3.4e7 条（条宽 32 字节是估的）。

| 候选 | 准入之前的代价 |
|---|---|
| B1 今天 | O(总 key + 总节点 + R) 乘（1 + 三四轮）；每轮克隆 + 约 5 趟全记录分叶 + 整份 `resolve`；分配器自己的线性查找再加一层 |
| B2 差量规划连固定点 | 树与容器树 O(Δ·h·log N)；固定点若不配机制仍整份克隆；`resolve` 里定义没写到的行（普通半、释放判定索引、deadlist 条目、记账行表）仍 O(N)，每轮重来 |
| B3 差量规划连分配器差量预演 | B2 的克隆与全记录分叶没了；分配器自己的取落点扫前缀、释放按记录逐条找若不加索引，仍是 O(已占前缀)、O(R) 每次 |
| B4 只修落点判前置并重述射程 | 落点判之前 O(Δ·h)（记账树取闭式之后）；两道之间那一段与 B1 同，不变 |
| B5 读法一（全量规划在前） | 与 B1 同，外加一次 O(Δ·h) 的上界计算 |
| B5 读法二、B6 | 放行路径：准入之前 O(Δ·h + 盘数 × 根层级)，全量规划排到之后，总量与 B1 相同；拒绝路径与带内：与 B1 同 |

## 七、同类先例（找到的、找不到的）

- 逐层带删插数的差量上界：`admission.rs:1484` 的 `central_mapping_tree_nodes_an_empty_publish_rewrites_at_most`（用户 2026-09-27 定按最坏计）。B6 的中央映射树项是它的推广，不是新发明。
- 随版本带着、提交时增量维护的量：`InodeRecordCounts`，原文「准入（运行时决策路径）读它，所以随一版带着、在提交时增量维护」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:284`）。格五的「水位随版本带」和记账树节点数随版本带，同形。
- 上界对真值的实测对照：E181 的 F9 登记了「上界 P1 与今天的 P2 在中央映射树两层、三层的写上逐次比真写的固定点」，第一段「还没跑，原件未留存」（`.claude/kb/experiments/181-落得下那一判与挂载下界.md:15`）；欠账 C573 记着 2026-10-12 判决量到的 1 至 16 槽（`.claude/kb/checks-owed.md:348`）。W 的 2·(K + 26) 是另一个实测对照（格一，取回的辩方报告第 93 行）。
- 上界判在前、放不下回落到精确路径的同形先例：找不到。仓里「回落」一词出现的是分配器的落点回落，与此无关。B6 因此没有可贴的先例，它的正确性只能靠 3.2 的逐类论证加 3.3 补的穷举。

## 八、要主 agent 跑的命令

我没写模型，没有现成的命令能把本报告的「推」变成「量」。下面是两条后续，都不是我能直接给整条命令的，列出来供主 agent 决定要不要新派腿：

1. 穷举验证 B6 的码 2 树上界（3.3、2.2）：需要先有人写一个 singlefs-core 的测试目标，把容量取最小（叶 2、内部 2，再加 3 与 4 两档），穷举所有可达的上一版形状与所有进出集合，调 `plan_the_tree_after_this_publish`，比 `rewritten_positions().len()` 与「插的 key 数 × 2 × 之后的高 + 删的 key 数 × 之前的高」；预期 0 次违反（推），出现违反就是 B6 的码 2 树项要改。写法可从 `research/prompts/m4-admission-delta-r1-opus-model/planner_bound.rs` 拷（它的 `rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只换 `rerun_steps`）。命令形态由那条腿写；不是重型（singlefs-core 的测试目标）。
2. 量分配器取落点扫描随单元区的用时（格二最后一条）：E181 页写的装置段 `f11`，整条（范围声明的写法照 `.claude/hooks/run-scope-guard.sh` 文件头，主 agent 按实际调；范围含 singlefs-checker-tier，按重型处理）：
   - 命令：`SINGLEFS_RUN_SCOPE='种类=experiment; 跑什么=e181 装置的 f11 段; 范围=1 段; 为什么=量分配器取落点扫描的挂钟随单元区增长，答 m4-admission-delta-r2 格二' task-server/task submit -- cargo run --release -p singlefs-checker-tier --bin e181_landing_check_fixed_point_and_mount_lower_bound -- f11`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`。
   - 跑完看哪几行：`TASK_SERVER_RESULT` 的 `type=` 与 `exit=`；`task-server/task log <号> --all` 里 F11 的 `median_nanoseconds` 与 `maximum_nanoseconds`（E181 页「复跑」一节下面那一段说计时随机器忙闲变）。
3. 跑完新派同立场的腿，写「承接：research/prompts/m4-admission-delta-r2-sonnet-output.md」与任务号，读 `task-server/task log <号>` 把标「推」的几格改成量的。

## 九、没做什么

- 不替主 agent 撤销或维持判决，只交复核结果；没往统一任务服务交任务，判决要引的数没复跑（命令在上一节）；任务 6394、6265 的数是我只读 `task-server/task log` 取的原样行。
- 没写模型，没量任何用时和内存，第六节全是按字段类型与调用次数推的。
- 没读禁读清单；没逐项复核第一轮正推表二 #22 至 #31 与表一各行（转述处已标）。
- 没有给任何一条候选下「选哪条」的结论；B6 是替 A3 找的变体，不是推荐。
- 没验证 `allocator.records` 与上一版 `allocation_records` 在进入固定点时是否恒相等（格一前提一）；没追新一版保存的 `allocation_records` 是不是排好序的。
- 没读 E181 那次弹窗的原问题与选项（读不到），格三的「够不着」是从变更史留下的字面推的。
- 草稿目录里只有一份从 git 取回的 `m3-delete-r3-sonnet.md`（68 KB 文本，没有仓副本与编译目录）。

## 十、补：第一关各条没写推翻现象的，在这里补

- 2.3（K1 是无条件上界、循环只是封顶值）：推翻它的现象是造出一段真历史，使一次发布重写的分配记录树节点数随释放链逼近 K1（而不是几十个）；第二次抽样自己写了没造出。造出了，B6 的带按 K1 算就不是过宽，而是贴身。
- 2.4（ckpt_cost 不是写入类发布的上界，是 A3 与今天共有的洞）：推翻它的现象是 C573 记的 1 至 16 槽在现在的代码上不再复现（那条是 2026-10-12 的量，之后落点判或 ckpt_cost 改过没有，我没查）。
- 2.5 矩阵：推翻某一格的现象就是那一格对应的依据在该臂上也复现。最便宜的是 B5 的「次序不中」：写一条「快路条件成立而今天会拒」的历史。
- 3.4 的 1（B6 只救放行路径）：推翻它的现象是给出一个让拒绝路径也只随 Δ 的做法而不需要 B2、B3 的差量判（我不信有：拒谁先谁后要精确角色）。
- 3.4 的 3（免责口径比字面宽）：推翻它的现象是用户明说「准入之后」的免责包括规划与固定点。
