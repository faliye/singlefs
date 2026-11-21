# m4-publish-without-whole-version-copy-design-r1 正推腿报告

日期：2026-11-18。腿：三方正推（`three-way-forward`），第一轮。方案按 `crates/` 今天的实现来谈，不改 `crates/`。

标记法：【读】读到的（文件与行，现读）；【量】命令此刻跑出的数；【推】推的，没量过，写明依据与会被什么推翻。
没写模型：下文所有「一次发布的代价」公式都是【推】，只有「今天的基线」取自任务 19831 的分段产物（【量】，经主 agent 转述，本腿读了那份产物文件）。
要主 agent 跑的命令：无（见末尾「要主 agent 跑的命令」）。

## 先校正背景材料里的三个数（影响读者面）

1. 单元图包不读 `TransactionOutput`。`crates/singlefs-unit-graph/Cargo.toml` 的依赖里没有 `singlefs-core`，`crates/singlefs-unit-graph/src` 里 `TransactionOutput` 零命中；
   它的 `.units` 是编译单元（闭包、入口分组），不是版本里的单元。【量】`grep -rnE '\.units\b' crates/singlefs-unit-graph/src` 122 行、`.../tests` 15 行，合 137 行，都在背景材料的 386 里。
2. 池级 checker 也不读 `TransactionOutput`。`crates/singlefs-checker/Cargo.toml:6` 写着「与实现只共享格式常量模块」，依赖只有 `singlefs-format`；
   checker 源里 `TransactionOutput` 只出现在注释（`crates/singlefs-checker/src/walk.rs:6847`、`crates/singlefs-checker/src/rebuild.rs:230`）。它的 `.units`（`walk.rs:18835`、`:18959`）与 `.allocation_records`（4 行）是它自己的类型。【读】
3. 386 行 `.units` 的构成【量】：`grep -rnE '\.units\b' crates/*/src crates/*/tests` 按目录数：单元图 src 122 + tests 15、验证包 src 103 + tests 104、核心 src 36、checker src 2、GPU 测试包 src 4，合 386（与背景一致）。
   其中真正读 `TransactionOutput.units` 的只可能在核心、验证包、GPU 测试包（共 247 行上限），验证包里还混着崩溃事实表的 `facts.units`、`publish.units`（它们是对录下的写的下标，见问题 3），所以 247 是上界不是读者数。
   `.allocation_records` 206 行【量】：GPU 测试包 src 13 + tests 2、checker 4（自己的类型）、核心 src 29、挂载运行库 tests 2、验证包 src 62 + tests 94。
   `.mapping_keys` 27 行【量】：真读 `TransactionOutput.mapping_keys` 的在 src 里 0 处（只有 `crates/singlefs-core/src/recovery.rs:3787` 构造与 `crates/singlefs-core/src/mount.rs:10805` 回退时照抄），
   在 tests 里 19 行 / 9 份文件（`new_pool_file_creation_publish.rs:1053`、`overwrite_in_one_instance.rs:212`、`crash_judge_gpu_reads_long_files_and_lineages_block_by_block.rs:233` 等），其余 6 行是别的结构的同名字段。

## 问题 1：`TransactionOutput` 里随池涨、每次发布整份重建的字段

定义在 `crates/singlefs-core/src/transaction.rs:3375`。变量：U = 一版里的单元数（数据单元加各树节点），R = 分配记录条数，M = 中央映射条目数（≈ U 加留存的），S = 活快照数，I = inode 数，C = 墓碑 / 谱系 / 退役容器数。
基线【量】（任务 19831 的 `section-timing-after-the-first-speedup-task-19831.txt`，P=4096 交错写 4224 次发布，本机负载高、只看比例）：
每次发布平均 R ≈ 43861（`admitted.allocation_records_sorted_copy` items=185268494 ÷ 4224）、U ≈ 4500（`admitted.units_assembly` items=19007912 ÷ 4224）、M ≈ 4468（`admitted.central_mapping` items=18871770 ÷ 4224）。R 约是 U 的 10 倍。

| 字段 | 随谁涨 | 每次发布怎么重建【读】 | 读者与读法 |
|---|---|---|---|
| `units`（`:3387`） | U | `transaction.rs:17032` 起新开 `Vec`，数据单元、extent、inode、分配记录树、记账、deadlist、删除三树、映射树逐族 push / extend 到 `:17260`；照抄的从上一版的按角色索引取（索引 `:16089` 每次发布整建，O(U log U)）；实例表链照抄走 `:17243` 整版扫一遍 | 全版：整理读版本时扫全部算钉住的槽（`mounted_session.rs:4939`）；可写挂载判落后盘与补写闸、整版逐单元读（`mount.rs:6899`、`:7118`）；管理员回退取目标版的用户可见单元（`mount.rs:10742`）、算现行版本全部单元的槽（`mount.rs:11039`，在 `roll_back_by_a_forward_publish` 里）；验证包对拍（`model_comparison.rs:190`、`:215`）。只读这次改的：要写的单元与记录点名的单元按 `rewritten` 取（`transaction.rs:17456` 起）；`container_tree_root_unit_positions` 记下根单元下标（`:17619`，整版扫两次） |
| `mapped_units` | U | `transaction.rs:16642` 起整份重装，`:16648` 与准入之前 resolve 现算的另一整份逐项比 | 全版：整理读映射（`mounted_session.rs:4906` 起，每把 key 查一次）；准入只读长度（`admission.rs:1271`）；回退（`mount.rs:10752`、`:10757`、`:10867`）；【量】`\.mapped_units\b` 74 行：核心 src 12、验证包 src 7 + tests 51、GPU 测试包 src 4 |
| `mapping_keys` | M | `transaction.rs:16666` 从 `mapping_entries_by_key` 的键整份拷出 | src 里 0 个读者（见上）；tests 19 行只读长度、成员、排名 |
| `allocation_records` | R | `transaction.rs:16232` `allocator.records().to_vec()`，`:16233` 整份排序（分配器那份是插入序，`allocator.rs:2294`、`:2743`） | 全版：准入之前固定点三轮拿它当「上一版记录」与分配器拷贝逐叶比（`transaction.rs:14085`、`:14105` 调 `allocation_record_tree.rs:368`）；真发处再比一遍（`:16245`）；整理选段与区域扫描（`mounted_session.rs:5400`、`:5414`、`:5430`、`:5478`）、区域扫描全记录过滤（`region_scan.rs:452`）；GPU 产品包经 `CompactionComputeOfTheBatch` 收切片（`crates/singlefs-core/src/background_compute.rs:623`）；挂载重建分配器（`mount.rs:3259`）；回退照抄（`mount.rs:10806`）；验证包 `history.rs`、`model_comparison.rs:191` |
| `data_pointers`、`files_other_than_the_first[*].data_pointers` | 文件的数据单元数 | 照抄的前缀也整份拷（`transaction.rs:15651` `carried_file_version_units`）；覆盖写 P 个单元的文件，每次发布从上一版取 P 个指针 | 多数读者只读长度（`transaction.rs:4936`）或按下标取；【量】`\.data_pointers\b` 行数：核心 src 57、验证包 src 17 + tests 244（含别的结构的同名字段；核心里 `recovery.rs` 占大半，是挂载读盘重建，一次挂载一回） |
| `allocation_record_tree.nodes`（`allocation_record_tree.rs:474`） | R ÷ 叶容量 | 按计划整份重列（`transaction.rs:14791` 的 `build_allocation_record_tree`）；`pointer_of`（`allocation_record_tree.rs:487`）是线性找，照抄每个节点各找一次 | 全版：装树时（`transaction.rs:14811` 每个照抄节点问一次上一版指针）；验证包 39 行 |
| `extent_tree.upper_nodes / lower_nodes`（`extent_tree.rs:760`） | 文件大小 | 重列；`upper_pointer_of`、`lower_pointer_of`（`extent_tree.rs:776`、`:784`）线性找 | 核心 32 行、验证包 41 行 |
| `central_mapping_tree`（`CodeTwoTreeVersion`，`code_two_tree.rs:722`） | M | `shape` 里每片叶带全部 key（`code_two_tree.rs:185`），key 是两个 `Vec`（`code_two_tree.rs:77`）；`transaction.rs:17915` `plan.shape.clone()` 整份深拷；计划函数 `code_two_tree.rs:637` 收整份 key 集、`:650` 从上一版形状再拷一份 key 集做差 | 全版：装映射树（`transaction.rs:17796` 起）、整理读映射叶（`mounted_session.rs:4906`）；验证包 34 行 |
| `accounting_entries`、`accounting_tree` | 常数行加每个造 inode 的发布多一行台阶 | `transaction.rs:8175` 每次现算；台阶行从上一版全部带上（`transaction.rs:4425`）；`admission.rs:1361` 断言每次发布至多多一条 | 准入读长度与按行查（`admission.rs:271`、`:1345`，运行时决策路径，不遍历）；增长是 O(造 inode 的发布数)，不随数据单元涨 |
| `deadlist_entries`、`deadlist_tree`、`mapping_entries_kept_for_the_snapshots`、`snapshots_being_destroyed`、`tree_table_entries` | S（活快照数）；树表十二条加每快照一条 | `transaction.rs:17651` 等整份 `clone()` | 释放判定、删快照批、整理（`transaction.rs:6686`、`:6855`、`:6866`）；【量】行数：`.mapping_entries_kept_for_the_snapshots` 核心 14、验证包 src 4 + tests 36；`.deadlist_entries` 核心 3、验证包 src 4 + tests 64（checker 的 11 行是它自己的类型）；`.tree_table_entries` 核心 17、验证包 src 16 + tests 57。无快照时是常数 |
| `inode_leaf_containers` | I | `build_inode_tree_units`（`transaction.rs:14912`）每片叶容器每次发布都出一个 `PublishedUnit`；叶容器内容整份移进版本 | 【量】`\.inode_leaf_containers\b` 行数：核心 src 15、验证包 src 11 + tests 79、GPU 测试包 src 2（含同名字段）；下一次发布按它算记录落哪一片（`transaction.rs:4359`、`:4371`） |
| `tombstone / lineage / retirement_containers` 与 `*_tree_index_nodes` | C（只增不删） | `build_deletion_tree_units`（`transaction.rs:15312`）每片容器每次发布都出一个单元（`transaction.rs:17184` 起三族各一个循环） | 【量】行数：核心 src 10 + 9 + 9；验证包 tests 26 + 20 + 31（含同名字段） |

每次发布都已是增量、不随池涨的（本次的账，不进「整份重建」）：`rewritten`（`:3389`）、`released`、`quarantined_after_release_checksum_mismatch`（`:3466`）、`units_relocated_by_this_publish`（`:3470`）、
`earlier_records_of_this_publish`（`:3384`）、`writes`（`:3474`）、`record`、`record_bytes`、`root`、`tree_identifiers`、`inode_record`、`key_order_mismatches`、`highest_transaction_number_in_this_instance`、`inode_record_counts`、`container_tree_root_unit_positions`（后者存下标，随 `units` 重装重算）。
`deleted_inodes` 与 `logged_operations` 是删除队列与至多一条整理 / 截断意图，小。
`Arc<[u8]>` 单元字节已共享（`PublishedUnit.bytes`，`transaction.rs:3358` 起），所以 `units` 里随池涨的是 `Vec` 本身加每项的 `Arc` 引用计数（原子加），不是字节。【读】

### 读者按包

| 包 | 读哪几样、读全版还是只读这次改的 | 依据 |
|---|---|---|
| 核心 `singlefs-core` | 写者：发布（`publish_admitted`）读上一版的 `units`（索引）、`allocation_records`、`mapped_units`、`data_pointers`、各树版本，是全版读；固定点三轮读上一版 `allocation_records` 与分配器拷贝逐叶比（`transaction.rs:14085`、`:14105`）。整理（后台、无戒律）：每步先整版深拷（`mounted_session.rs:5371`），再整版读映射、`units`、`allocation_records`（`mounted_session.rs:4906`、`:4939`、`:5400`）。挂载与回退：`recovery.rs:3778` 与 `mount.rs:10795` 两处构造、`mount.rs:3259` 从版本重建分配器。准入只读长度与按行查 | 上表各行 |
| 池级 checker `singlefs-checker` | 不读：只收盘镜像字节，依赖只有 `singlefs-format` | `crates/singlefs-checker/Cargo.toml:6` |
| 验证包 `singlefs-verification` | 读版本的：`history.rs`（随机历史，逐发布比上一发布的 `allocation_records`，`history.rs:2717`、`:5470`、`:7542`、`:8387`）、`model_comparison.rs`（对拍，读 `units` 与 `allocation_records`，`:190`、`:191`）、`scenario.rs`、`on_device_modes.rs`、`mount_transparency.rs:345`。tests 里 `assert_eq!(pool.output, current_before, …)` 整版相等（8 处，如 `rollback_by_a_forward_publish_verifies_every_copy_of_an_isolated_unit_the_target_references.rs:177`）。**不读版本的**：崩溃点重放与 GPU 判器的输入表（`crash_judge_tables.rs:1745` 起 `judge_publishes_of` 吃的是 `memory_pool::PublishWrites`，里面的 `units` 是对录下的写的下标；`crash.rs:140` 同），这几处的 `.units` 不是版本里的 |  `grep -n TransactionOutput crates/singlefs-verification/src/*.rs` 只命中 `crash_injection.rs`（3 行，一行 `use`、一行注释、一行取 `inode_record` 的小函数 `:3190`）、`history.rs`、`model_comparison.rs`、`model.rs`（注释）、`mount_transparency.rs`、`on_device_modes.rs`、`scenario.rs`、`verification_read.rs`（测试里）。【量】 |
| 产品 GPU 包 `singlefs-background-gpu` | 不直接读 `TransactionOutput`（`src` 里零命中）；经核心定义的 trait 收 `&[AllocationRecord]` 切片：`background_compute.rs:623`；包内 `background_gpu_compute.rs:89`、`:147`、`:728`、`live_slot_kernels.rs:99`、`:342`、`:480`、`:497`、`background_gpu.rs:4072`、`:4131` 共 9 处签名。输入是整理这一步取的整版记录，要上传显存，本来就 O(R) | 【量】`grep -rcE '&\[AllocationRecord\]' crates/singlefs-background-gpu/src` |
| GPU 测试包 `singlefs-background-gpu-tests` | 装置 `e202_gpu_cpu_gc_comparison.rs` 读 `units`（`:1534`、`:1733`、`:1981`）、`allocation_records`（13 行）、签名 12 处 `&[AllocationRecord]`；tests 里 1 处签名 | 【量】同上类命令 |
| 挂载运行库 `singlefs-mount-runtime` | tests 里读 `allocation_records` 2 行、`tree_table_entries` 1 行、`logged_operations` 1 行；src 不读 | 【量】 |
| 单元图 `singlefs-unit-graph` | 不读（见上） | 【量】 |

## 问题 2：候选逐格答、可加候选、一条推荐

字母：d = 这次改到的键数，N = 那一张表的条数（R、U 或 M），F = 树的扇出，h = ⌈log_F N⌉（F=32、N=43861 时 h=4，推的），B = 分块大小。
这几个候选里的「表」都指同一类东西：按键有序、按键取、要遍历全部、要整版相等的一张表。`units` 的键是角色在今天 `Vec` 里的次序（见问题 3 的 `UnitRank`），`allocation_records` 的键是 `(设备, 槽号)`（`AllocationRecord::sort_key`，`crates/singlefs-core/src/allocator.rs:121`），`mapped_units` 同 `units` 的键。

加的候选：
- **丁　二层分块持久表**：有序序列切成每块至多 B 条的 `Arc` 块，外面一条 `Arc<Vec<Arc<块>>>` 的脊；改一条 = 复制那一块与脊。块界对分配记录可以直接取分配记录树的叶界（`allocation_record_tree.rs:186` 的 `leaf_of_slot`：叶罩 `[k×W,(k+1)×W)`、W ≤ 812），于是「变了的叶」就是「指针不同的块」。它是甲的接口下更小的第一版实现，不是另一种读法。
- 不加「不换表示、只加索引与增量缓存」：那是不动形态的那两批的活，本题不判。

| 格 | 甲 持久共享结构（自写 Arc 节点的有序 B 树，带子树条数） | 乙 只带差量、全版在写者的可变索引 | 丙 差量加指向上一版的引用 | 丁 二层分块持久表 |
|---|---|---|---|---|
| 一次发布的代价 | 【推】O(d·h) 个节点复制，每个至多 F 条；与全版条数只经 h 相关。最坏发布：d 本身就是 O(N)：删 / 截断大文件释放 P 个单元、整文件覆盖、整理一批搬 K 个、建删快照批；位置寻址的树节点分裂会让同层后面每个节点换角色（`TransactionUnit` 里的 `…NodeBelowTheRoot(position)`），`units` 键变，d ≈ 分裂点之后的节点数。这些在任何表示下都要付 Ω(d) | 【推】O(d) 记差量 + 对写者索引 O(d log N) 原地改；最坏同左 | 【推】O(d) 记差量 + O(1) 连链；每 L 版压平一次 O(N)，摊到每次 O(N/L)，不压平则读代价无界。最坏同左 | 【推】O(d·B + N/B)：改到的块各复制 B 条，脊复制 N/B 个指针。N=43861、B=256 时脊 172 个指针；仍随 N 线性，系数约今天整份拷贝的 1/750（推的，按 24 字节一条记录估，没量 `size_of`） |
| 读全版的代价 | 【推】取一条 O(h)、范围 O(h+k)、第 i 条 O(h)（带条数）、条数 O(1)、遍历 O(N)（叶内连续、叶间跳）。今天每次发布整建的按角色索引（`transaction.rs:16089`，O(U log U)）不再需要：表自己就是索引。`units` 的 `.units[i]` 下标读（16 处）走第 i 条 | 【推】写者持有的最新版同甲；**过去的版本读不到**（索引只有一份）。要读就得先拷 O(N)，等于回到今天 | 【推】取一条最坏 O(链长)，遍历 O(N + 链上差量总数)；发布路径每次对约 U 个角色各取一次上一版，乘链长。要有界就要压平，压平版就是甲的丑版 | 【推】取一条 O(log(N/B) + log B)；遍历按块给连续切片，GPU 产品包要的 `&[AllocationRecord]` 可逐块交（`live_slot_kernels.rs:99`）而不必先合成一整份 |
| 读者改动面 | 见问题 3。`.units` 常见写法是 `iter()` 链（签名里写 `&[PublishedUnit]` 的只有 5 行：`model_comparison.rs` 3、`transaction.rs` 1、`mount.rs` 1，【量】`grep -rcE '&\[PublishedUnit\]' crates`）；能留与今天同形的只读视图，多数读者不改（推的，改动处数要看第一步的编译错误数）。`allocation_records`：112 行签名写 `&[AllocationRecord]`（29 份文件，【量】），树不连续，**留不了切片视图**，只能给范围与迭代接口，GPU 上传前 `to_vec()` | 读者面最小（索引还是 `Vec` 或 `BTreeMap`），但「读过去的版本」的读者（`history.rs` 逐发布比上一发布、tests 里 `current_before`）不成立 | 同甲的视图接口，每个读者都经链走一次 | 同甲，只是 `&[T]` 可以按块给，签名里要一条的改成按块迭代 |
| 版本相等与换回 | 【推】`PartialEq` = 根指针 `Arc::ptr_eq`（O(1)，节点不可变所以足以证内容相同）否则先比条数再按序逐条比 O(N)；不许按树形状或节点哈希比（盘上重建出来的树与写者建的树节点界不同）。换回：分配器拷贝是 O(1) 取根（`transaction.rs:13213`、`:1930`），失败换回（`:13230`、`:1662`）之后根指针与发布前同一个，「回到发布前」可以用指针相同钉住，比今天的逐字段值相等更强 | 差量相等不等于全版相等：`assert_eq!(pool.output, current_before, …)`（8 处）比的是两份差量，写者索引被留在半新状态时它照样绿，断言空转。要钉住须再留一份索引拷贝 O(N)。换回要既留发布前又留发布后的索引（冻结重发那一支要发布后的，`transaction.rs:1662`；上一轮判决 `research/prompts/m4-gpugc-publish-growth-r1-main-verification.md` 已判「回滚日志只给得出之前那一份」） | 换回最简：丢掉新头、上一版没被改过；相等 O(N) 合出再比，或头指针相同 O(1) | 同甲，块 `ptr_eq` 比 O(N/B)，不同源的两棵比 O(N) |
| 审计独立 | 池级 checker 不依赖核心（`crates/singlefs-checker/Cargo.toml:6`），崩溃点重放与 GPU 判器的输入表读的是录下的写、不是版本（`crash_judge_tables.rs:1745` 起）：这两处在任何候选下都不变。受影响的只有验证包里读版本字段的对拍与测试：它们读的是写者同一份表代码。独立的一条腿：`recovery::rebuild_version`（`recovery.rs:8047`，读盘字节重建，用 `from_sorted_iter` 建表）与写者发布出来的版本做逻辑相等；另在验证包里放一份与核心不共用代码的 `BTreeMap` 参照对表本身做对拍 | 对拍读的是写者持有的可变索引，也就是被审的东西本身，`.claude/rules/fs-design.md@记账是事务的副产品，不是事后的遍历` 禁的「审计与被审计用同一段代码」在这里最直接；读过去的版本读到的是此刻的索引 | 同甲（合并代码是共用的一段），再加合并逻辑本身要审 | 同甲 |
| 内存（V 个版本留着） | 【推】基数 + V·d·h·F·条目字节；今天每个留着的版本一整份（R×约 24 + U×约 48 + 映射 key 两层 `Vec` 加 `shape` 里再一份……，没量 `size_of`）。`versions_published`（`mounted_session.rs:2040`）、`steps_published`（`mount.rs:8376`）、验证包 `e156_allocation_basis_counts.rs:3028` 都留版本 | 基数 + V·d（只差量）。但过去版本不可读，所以不是同一件事的比较 | 基数 + V·d + (V/L)·N（压平的版）| 基数 + V·(N/B·8 + 改到的块·B·条目字节) |
| 依赖 | 不引外部包：`grep -rn 'im::\|rpds' crates/*/src --include=*.rs | wc -l` 输出 0（背景材料的模式多一个 `persistent`，那个词只命中 `e158_root_choice_repair.rs:2761` 的变量名，不是包）；`singlefs-core` 依赖只有 `libc`、`singlefs-format`、`singlefs-replay-prefix`、`vstd`。自己写约 400 至 700 行（推的，含改与删的路径复制、范围、条数、差异、按序建表）。验证：与 `BTreeMap` 的随机操作对拍（含「克隆之后再改，旧克隆不变」这条持久性检查）、`crates/mutations.tsv` 里加变异行（跳过路径复制原地改、分裂偏一格、`ptr_eq` 快路径不看内容）、问题 5 的影子比对。要不要引 `im` / `rpds`：推不出（没读到禁引的规则，也没读到允许；要问用户） | 无 | 约 200 行（推的） | 约 250 行（推的），验证同甲 |
| 分步 | 能。按字段一样一样换，每步全部测试不改期望（问题 3） | 不能一样一样换：一个字段换成差量，凡读那个字段过去版本的读者同时失效 | 能，但每个字段各定压平策略 | 能，同甲 |
| 发布语义不变的证据 | 问题 5：同一套场景换前换后逐次发布的写日志摘要与版本规范文本摘要逐字相同；表自己与 `BTreeMap` 对拍 | 同左，另加「合并视图与旧 `Vec` 相同」一条，而合并视图在乙下只对最新版存在 | 同左，另加「沿链合出与旧 `Vec` 相同」 | 同甲 |

### 推荐：甲，接口上用丁做第一版实现

依据（都是读到的）：
1. 版本是值，且被留着、被读：`UserChangePublished.versions_published`（`mounted_session.rs:2040`）、冻结重发的 `FrozenPublish.version`（`transaction.rs:1575` 起的结构）、预演先 `clone()` 再改（`mounted_session.rs:1866`、`transaction.rs:12791`、`mount.rs:5810`）、`previous: Option<&TransactionOutput>`（`transaction.rs:15685` 一带，不可变借用）。乙的「索引只有一份」与这几处不相容。
2. 发布路径每次对约 U 个角色各取一次上一版（`transaction.rs:16089` 的索引、`:17243` 的整版扫），丙的链上取值乘上链长。
3. 换回今天靠「值」：失败就丢新版本、换回旧分配器（`transaction.rs:13230`、`:13235`）；甲保持这一点并让分配器的那份记录拷贝从 O(R) 变 O(1)。
4. 三处构造都得产出同一种表：`transaction.rs:17620`、`recovery.rs:3778`、`mount.rs:10795`；甲的「按序建表」O(N) 只在挂载与回退时付一次。

这条推荐依赖的前提（会推翻它的现象）：
- 若第一步测出「每次发布改到的键数 d 本身就接近 N」（例如交错写让每次发布改到大半个 `allocation_records`），甲对记录这一张表就买不到东西。【推】从任务 19831 的 `settle.nodes_changed_after_allocation` 一类行读不出 d，要第一步加计数（见问题 3 第 0 步）。
- 若 `units` 里同一角色出现多项、或今天的次序不是角色的函数（这两点我没核，见问题 3 第 0 步的断言），键就立不起来，`units` 要换成按位置的持久序列。
- 若自写树在对拍里找不到过不去的变异，则依赖格没有问题；找到 ≥ 1 个活的变异（对拍抓不到），则退回丁。

## 问题 3：推荐的表示写成对实现的改动

### 3.1 新类型与键

新文件 `crates/singlefs-core/src/shared_ordered_table.rs`（`crates/` 里没有同类容器：`grep -rn 'im::\|rpds' crates/*/src` 在包与类型两层都零命中）：
`SharedOrderedTable<Key, Value>`，节点 `Arc`、不可变，写走路径复制（`Arc::make_mut`），节点里带子树条数。接口只有：`len`、`get`、`nth`、`range`、`iter`、`insert`、`remove`、`update`、`from_sorted`（O(N)，挂载与回退用）、`changed_keys_since(&other)`（跳过 `ptr_eq` 的子树，发布路径与「哪几片叶变了」用）、`Clone`（O(1)）、`PartialEq`（先 `ptr_eq`、再按序逐条比；不许按节点形状比，原因见问题 2 版本相等一格）。
三个字段各包一个有名字的新类型，`Debug` 打印成与今天 `Vec` 相同的 `[a, b, …]`：`UnitsOfAVersion`、`MappedUnitsOfAVersion`、`AllocationRecords`。

键：
- `allocation_records`：`(设备, 槽号)`，即 `AllocationRecord::sort_key`（`allocator.rs:121`），也就是今天 `transaction.rs:16233` 排序用的键。
- `units`、`mapped_units`：新加 `UnitRank`，由 `TransactionUnit` 算出来、等于今天 `units` 里的次序。**不能直接用 `TransactionUnit` 的派生 `Ord`**【读】：它的声明序是 bump 次序（`transaction.rs:2692` 的文档串），而 `units` 里的次序不是它：
  `DataOfAFileOtherThanTheFirst` 声明在 `InstanceTablePageAfterTheFirst` 之后（`:2774` 对 `:2770`）、`units` 里却紧跟 `Data`（`:17047` 起）；
  `TombstoneRoot` 声明在 `TombstoneNodeBelowTheRoot` 之前（`:2734` 对 `:2737`）、`units` 里先根之下节点再根（`:17190` 对 `:17193`）。
  这与上一轮判决 `research/prompts/m4-gpugc-publish-growth-r1-main-verification.md` Q2.2 一格「按身份有序会改 `units` 的次序与下标语义」是同一件事：只要 `UnitRank` 复刻今天的次序，次序与下标语义就不变；复刻对不对，第 0 步的断言来判（今天我没核，推不出）。

### 3.2 要改的结构与读者（现读，行号是主工作区此刻的）

| 位置 | 改什么 |
|---|---|
| `transaction.rs:3387`、`:3396`、`:3398`、`:3459` | 四个字段：`units` → `UnitsOfAVersion`，`mapped_units` → `MappedUnitsOfAVersion`，`allocation_records` → `AllocationRecords`，`mapping_keys` 删字段、加方法 `mapping_keys()`（取 `mapped_units` 的 key 并上留存条目，按 `CodeTwoTreeKey` 序，与 `:16666` 的 `BTreeMap` 键同序） |
| `transaction.rs:17620`–`:17739`（构造）、`:16232`–`:16233`、`:16642`–`:16669`、`:17032`–`:17260` | 发布出口。第一版：从各树建出来的 `Vec` 经 `from_sorted` 建表（仍 O(U)，换到的是之后的克隆、保留、换回为 O(1)）；第二版：见 3.3 第 5 步，建树的几个函数只交这次改的 |
| `transaction.rs:4096`–`:4107`（`take_the_mapping_positions_over_the_location_hints`）、`:4517`–`:4560`（`unit`、`units_by_role`、`mapped_units_by_role`、`mapped_units_by_key`） | 按槽改位置走 `update`；索引访问器直接走 `get`，不再每次 `FirstEntryOfEachIdentity::of_the_table` 整建 |
| `allocator.rs:2294`（`records`）、`:1548`（`make_room_for_record_on_device`，`:1586` 的 `retain`）、`:2744`–`:2780`、`:2790`、`:2800`（`release`、`release_leaving_the_record_allocated_on`）、`:2670`（`rebuild_from_records`）、`:1600`（`AllocationRecordTreeOfTheVersionWithoutFile.records`） | 分配器的记录改成同一种表；今天这几处是对未排序 `Vec` 的线性 `find` 与 `retain`，换成键查；`allocator.clone()`（`transaction.rs:13213`、`:1930`）里记录那一份变 O(1)，两张位图（每盘一张）照旧拷 |
| `allocation_record_tree.rs:312`（`records_of_each_leaf`）、`:349` 一带（`nodes_holding_records`）、`:368`（`nodes_whose_contents_changed`） | 加按叶取的范围接口；加 `changed_leaves_between(previous, after)`，用 `changed_keys_since` 得变了的键，按 `leaf_of_slot` 归叶，再带祖先 |
| `recovery.rs:3778`（构造）、`:4737`（`mapped_units_of_the_version`）、`:4887`（`units_of_the_version`）、`mount.rs:3259`（重建分配器）、`mount.rs:10740`–`:10867`（回退：`:10742`、`:10752`、`:10805`、`:10806`、`:10867`） | 按序建表；回退里 `current.… .clone()` 与 `target_version.… .clone()` 成 O(1) |
| `mount.rs:1335`、`:6899`、`:7118`、`:7403`、`:11039`；`mounted_session.rs:4906`–`:4960`、`:5059`、`:5400`–`:5667` | 读法不变（迭代全版）；`mount.rs:1335` 的 `Cow::Borrowed(&output.units)` 要改成迭代器或视图；`mounted_session.rs:5371` 的整版 `clone()` 变 O(1) |
| `region_scan.rs:384`、`:410`、`:452`（`records_starting_in_the_region`）、`:521`、`:1489`；`compaction_plan.rs:97`、`:128`、`:1322`、`:1349`、`:1397`、`:1436`、`:1933`、`:2831`；`background_compute.rs:623`（trait 方法）与两个替身（`:2415`、`:2877`） | 签名 `&[AllocationRecord]` 改成 `&AllocationRecords`；区域扫描里「整表过滤一遍」改成按 `(设备, 槽号)` 范围取：O(R) → O(h + 属于这个整理区域的条数)，上界一条也要看「起在区外、伸进区里」的前一条（`region_scan.rs:464` 起的越界判） |
| GPU 产品包：`background_gpu_compute.rs:89`、`:147`、`:728`；`live_slot_kernels.rs:99`、`:342`、`:480`、`:497`；`background_gpu.rs:4072`、`:4131` | 签名同上；要上传显存的地方 `to_vec()`（本来就是 O(R) 的上传，不新增阶） |
| 验证包：`model_comparison.rs`（3 条 `&[PublishedUnit]`）、`history.rs`（19 条 `&[AllocationRecord]`）、`on_device_modes.rs:197`、tests（见问题 2 读者改动面） | 机械改写：`.mapping_keys` 改 `.mapping_keys()`（9 份文件 19 行）；`.units`、`.allocation_records` 的迭代写法不动，传切片的改成传视图；少数改版本字段的测试（`.units.push` 等，≤ 7 行，含非版本类型）改走 `from_sorted` 重建 |

「能不能留与今天同形的只读视图」：
- `units`、`mapped_units`：能。`iter()`、`len()`、`is_empty()`、`Index<usize>`、`IntoIterator for &`、`get(角色)`、`.clone()` 同名同义；`for unit in &version.units`、`.units.iter().find(…)`、`.units.len()` 不用改。不同的只有 5 行 `&[PublishedUnit]` 签名与改字段的测试。
- `allocation_records`：不能全留。树不连续，没有 `&[T]`。留 `iter()`、`len()`、`is_empty()`、`get(设备, 槽号)`、`range(设备, 槽区间)`、`to_vec()`；112 行 `&[AllocationRecord]` 签名里，产品这一侧（核心 + GPU 产品包）约 40 行要改，验证包与 tests 约 70 行。
- `mapping_keys`：字段改方法，tests 19 行机械改。
改动处数的最终数以第 2 步的编译错误数为准（上一轮甲档同类改动量到 14 行编译错误，`research/prompts/m4-gpugc-publish-growth-r1-main-verification.md` Q2.1 一格；本次字段更多，没有同口径的量）。

### 3.3 分步，每步的验收（每步之后全部测试不改期望）

每步都用同一套对照（问题 5）：步 0 先录「金标」，之后每一步重跑，写日志摘要与版本规范文本摘要两样必须与金标逐字相同；`cargo test` 点名的目标与门禁照共用约束只点这一步改到的。

0. **只加不改的取证**（验证包新增，不碰 `crates/singlefs-core`）：
   a. 取证脚手架：固定几组场景（小 P 的 E202 起步池写入、随机历史 `history.rs` 的几个种子、建 / 删快照、整理一批、回退、删文件、截断），逐次发布输出两个摘要：本次发布写进设备的 `(偏移, 长度, 字节)` 按序的 SHA-256；`format!("{:#?}", 版本)` 的 SHA-256。
   b. 在三个构造点后加仅测试构建里生效的断言：`units`、`mapped_units` 在候选 `UnitRank` 下严格递增、`allocation_records` 在 `(设备, 槽号)` 下严格递增无重复；同一角色在 `units` 里至多一项（今天 `FirstEntryOfEachIdentity` 的「取最先那一项」说明它没排除重复，`transaction.rs:4527` 的文档串）。
   c. 计数：每次发布 `units`、`mapped_units`、`allocation_records` 各有多少键相对上一版变了（d），与对应总数（N）并列，进验证包的产物，用来校第 2 格的前提。
   验收：同一树上跑两遍摘要逐字相同；人为把一条记录的 `generation` 加 1，摘要变（证红）；三条断言在全部现有测试里绿，红了先判是 `UnitRank` 写错还是今天确有重复 / 乱序，红了就停在这里交主 agent。
1. **`mapping_keys` 改方法。** 改动：`transaction.rs:3396`、`:16666`、`:17644`，`recovery.rs:3787` 与 `:4653`–`:4695`、`mount.rs:10805`，tests 9 份文件 19 行。验收：测试期望不改（只有取值写法变）；规范文本摘要里 `mapping_keys` 用方法的值写进，与金标相同。得：去掉一份每次发布整拷的 key 列表。
2. **`AllocationRecords` 包一层，里面仍是 `Vec`。** 2a：先 `Deref<Target=[AllocationRecord]>`，读者一行不改，只改构造；2b：去掉 `Deref`，由编译错误带路逐个把读者换成视图接口，热点读者（区域扫描、`nodes_whose_contents_changed`）换范围接口。验收：每个小步都是金标逐字相同、测试期望不改；2b 之后记编译错误数写进报告，以便回头校「改动面」一格。
3. **换内部：`AllocationRecords` 与分配器的 `records` 换成 `SharedOrderedTable`。** 改动见 3.2 表里 `allocator.rs`、`allocation_record_tree.rs`、`transaction.rs:16232`–`:16233` 三行。验收：金标逐字相同；开「影子模式」（仅测试构建）：每次对表的操作同时在一份 `Vec` 上做，每次发布之后断言两者逐条相同，全部现有验证包测试里零差；表自己的对拍（与 `BTreeMap`、含克隆后再改旧克隆不变）；`crates/mutations.tsv` 的新增变异行逐行证红。得：任务 19831 分段产物里 `items` 恰为记录条数的三行（`admitted.allocation_records_sorted_copy`、`publish.allocator_clone_before`、`admitted.allocation_record_tree`，items 都在 185.2M 上下，【量】）里随 R 的那一部分不再是 O(R)（这三行各行里记录以外的部分，如位图拷贝、节点重装，不变）；固定点里按叶比较那几行（`settle.leaves_before_this_publish` 的 items=370451084 是记录条数的两倍，对到哪段源码我没核，推的）同理；是否真得到，用步 0 的计数与分段计时在这一步前后比（这些只在主 agent 经服务跑得到，见「要主 agent 跑的命令」）。
   **一处要主 agent 判**：`transaction.rs:16245` 的整表断言（真发处比一遍变了的叶），上一轮判决 Q2.3 ② 一格写「`assert!` 照原失败路径留着，不挪」；它是 O(R)，留着则这一步买不到 O(d)。可行的并存做法（推的）：发布路径改走 `changed_leaves_between`，整表断言留在测试构建里。这改的是那一条判决的前提，推不出能不能改，交主 agent 判。
4. **`units`、`mapped_units` 换成 `UnitsOfAVersion`、`MappedUnitsOfAVersion`（内部先 `Vec`，再换表）。** 同第 2、3 步的两小步做法。验收同。
5. **建树的几个函数只交这次改的。** 表示换了之后发布仍是 O(N)，因为出口是从整棵树的整份产物重建的：`build_extent_tree`（`transaction.rs:14431`）、`build_allocation_record_tree`（`:14791`，`records_of_each_leaf` 整表分组）、`build_inode_tree_units`（`:14912`）、`build_deletion_tree_units`（`:15312`）、`build_multi_level_tree`（`:17796`，对每个节点照抄或重写、`shape: plan.shape.clone()` 在 `:17915`）、计划函数 `plan_the_tree_after_this_publish`（`code_two_tree.rs:637`，收整份 key 集，`:650` 再拷一份上一版 key 集做差）。这几个函数要改成「输入上一版的表加这次的变更，输出这次重写的单元与被换角色的清单」。这一步多大、每个函数几处，我没法按今天的代码估出（它们同时是不动形态那两批的改动对象，两边要排次序），**推不出**；缺的是不动形态那两批做完之后的函数形状。每个函数一小步，验收仍是金标逐字相同。
6. **`data_pointers`（含各文件的）换按位置的持久序列、各树版本的 `shape` / `pointers` 换成共享节点。** 同上，放在第 5 步之后，用第 0 步的计数先看它们的 d 与 N 值不值得。

### 3.4 这一条与条款的关系

- `.claude/rules/fs-design.md@一个事务层，所有结构共用`：不开第二条发布路径。本方案只换版本里表的表示，`publish_admitted`、`persist_the_publish_or_freeze_it`、`resend_the_frozen_publish` 一条路，发布语义与持久顺序一字不动。
- `.claude/rules/fs-design.md@记账是事务的副产品，不是事后的遍历`：准入读长度与按行查（`admission.rs:271`、`:1271`、`:1345`）仍是 O(1) 与 O(h)。整理（后台、无戒律）仍整版读，不在这条里。
- `.claude/singlefs-ai-sop/rules/code-discipline.md@类型：让非法状态写不出来`：三个新类型把「按序、键唯一」放进类型，不再靠 `Vec` 的约定；`UnitRank` 一个函数定次序，不再有「声明序」与「`units` 次序」两个口径。

## 问题 4：三条路在推荐的表示（甲）下各怎么走

### 4.1 发布失败换回

今天的走法【读】：发布前整个分配器拷一份（`transaction.rs:13213`；树表 0 条那一版的写行路径同，`:1930`）；上一版是 `previous: Option<&TransactionOutput>` 的不可变借用，新版本是局部值。三个失败点：
1. 准备阶段被拒：`*allocator = allocator_before_this_publish`（`transaction.rs:13230`），没有新版本；
2. 落盘前检查被拒（`check_before_persisting`，`:13234`）：同样换回（`:13235`）；
3. 落盘中途块设备报错：`persist_the_publish_or_freeze_it`（`:1649`）把分配器换回发布前（`:1662`），冻结的是**发布后的**分配器与新版本（`FrozenPublish`，`:1575` 起），重发（`resend_the_frozen_publish`，`:1684`）时整个换成发布后的那份。

甲下：控制流一行不改。区别在于代价与证据：
- 分配器里 `records` 是表，`allocator.clone()` 对它 O(1)；之后分配器上的改动走路径复制，发布前与发布后两个值共享没动的节点。换回 = 换回发布前那一个值，根指针与发布前相同，`Arc::ptr_eq` O(1) 可钉；冻结时发布前后各一根，额外内存 O(d·h·F)。
- 新增一条 O(1) 的一致性检查（今天只能 O(R)）：冻结的 `version.allocation_records` 的根必须与 `allocator_after_the_publish` 的记录表同一个根（因为发布出口取的是 `allocator` 表的克隆）。这条放进 `resend_the_frozen_publish` 的断言里。
- 位图（`devices`，每盘一张，`transaction.rs:13212` 的注释说约 450 KiB）不在这一题里，仍整份拷；`publish.allocator_clone_before` 里记录以外的部分仍在。
- 预演拷贝（`transaction.rs:2038`、`:14076`、`mounted_session.rs:1865` 的 `rehearsed_allocator`、`:1866` 的 `rehearsed_version`、`transaction.rs:12791`、`mount.rs:5810`）都是「克隆、改、丢」，克隆变 O(1)，改变 O(d·h)，丢掉时节点引用计数归还。
- 乙下换回要同时留发布前与发布后的索引（上一轮判决 Q2.3③ 的结论），丙下与甲同。

什么会推翻「甲下换回不变」：若有任何地方在两个值共享节点之后绕过 `Arc::make_mut` 原地改节点（自写树的实现错误），旧值会被改，`ptr_eq` 与值相等都会给假绿或假红；由问题 5 的「克隆后再改、旧克隆不变」对拍与 8 处 `assert_eq!(pool.output, current_before, …)` 兜（【量】`grep -rnE 'assert_eq!\(pool\.output, current_before' crates | wc -l` 输出 8）。

### 4.2 取号全或无失败的回卷

【读】`acquire_instance`（`transaction.rs:1112`）只收 `PoolWriter`，读与写的都是系统配置槽；失败时「已写出的盘每块回卷一次成旧代号」（`write_the_witnessed_acquisition`，`:1324`，说明在 `:770` 起的 `AcquisitionFailed` 文档串）。
我在 `transaction.rs:1112` 到 `:1500` 之间按 `TransactionOutput`、`PoolVersion`、`.units`、`allocation_records`、`mapping_keys` 搜过一遍：只有第 1474 行的文档串与第 1493 行（树表 0 条那一版的产出结构 `VersionWithoutFilePublishOutput` 的字段）命中，取号函数体里没有。
所以：这条路不碰版本里的表，任何候选下都不变。它之后的影响只有一处：取号失败 → 这次挂载报错或只读 → 下一次挂载由 `rebuild_version`（`recovery.rs:8047`）读盘重建版本，经 `from_sorted` 建表（O(N)，一次挂载一回，今天同阶）；`mount.rs:3259` 重建分配器时可以直接共享版本的记录表（O(1)），今天是 `clone()` 整份。
「回卷之后与取号之前相同」只在盘上有（系统配置槽的字节），在内存里没有版本可比：钉法不变，仍是现有的设备字节级用例。

### 4.3 整理一批发两次

我读到的「两次」有两种读法，都按代码列出，没法确定背景里指哪一种（推不出，缺的是调查员结果文件里那句话的上下文，我没复核它）：
- 读法一：一次整理循环先发签意图那一次（`mounted_session.rs:5610`），再发搬一批那一次（`:5727`），每次发布都在入口整版深拷一份（`:5371`），出口又拷一份交 `current` 与返回值（`:5610`、`:5727`、`:6022` 的 `published.clone()`）。
- 读法二：搬一批的发布里，`publish_admitted` 先把上一版整版拷一份、把搬走的单元的位置改到目的地（`transaction.rs:15822` 的 `previous_version.clone()`，再 `take_the_mapping_positions_over_the_location_hints`，`:3992`）。
- 还有一条重试环：计划里的单元本来就被这次发布重写时被拒（`CompactionStepRefused::PlannedUnitsRewrittenAnywayByThisPublish`，`mounted_session.rs:5694`），把这些 key 跳过再发一次。

甲下：
- 入口与出口的整版拷贝都变 O(1)（`mounted_session.rs:5371`、`:5610`、`:5727`、`:6022`、`:2362`、`transaction.rs:15822`）。
- `:15822` 之后的位置改写变 `update` K 条（K = 这批搬的单元数），O(K·h)，对 `units` 的 `slot` 与留存映射条目；表里每个被改的叶复制一份，其余与上一版共享。
- 重试环：被拒发生在落盘之前（`:13230`、`:13235` 的换回），`&version` 是不可变借用、值没动，原样再发。
- 整理读整版（`version_as_the_compaction_reads_it`，`mounted_session.rs:4934`、`central_mapping_as_the_compaction_reads_it`，`:4906`）仍是 O(M) 与 O(U)，这是后台、可续做的非决策路径，`.claude/rules/fs-design.md@记账是事务的副产品，不是事后的遍历` 那张表第三格写「无戒律」；表示换不换它都在。`CentralMappingIdentityIndex` 每步整建（`mounted_session.rs:5396`）同理不在这一题里。
- 乙下：入口取到的版本在发布之后被写者索引改了，返回给调用方的 `Published { version }`（`mounted_session.rs:5612`）与 `current` 读到的是同一份索引的不同时刻，`versions_published` 里留着的各步版本互相读成最新（不成立）。

## 问题 5：拿什么证明换表示前后每次发布写出的字节、落点、版本内容逐项相同

要证的三样：①每次发布交给设备的字节按序相同；②每个单元的落点（`slot`）相同；③版本里被换了表示的几个字段的内容相同。四条互不共用代码的路：

| 路 | 做什么 | 独立在哪 | 抓得到什么，抓不到什么 |
|---|---|---|---|
| A 写日志摘要 | 步 0 的取证脚手架在换表示之前的树上录一份：固定场景集（小 P 的 E202 起步池交错与顺序两半加垫文件删除、`history.rs` 的随机历史若干种子、建删快照、整理一批、回退、删文件、截断、落盘失败冻结加重发、取号失败）里每次发布写进设备的 `(偏移, 长度, 字节)` 按序的 SHA-256。每一步之后同一脚手架重跑，逐次发布摘要逐字相同 | 数据来自录写的设备（`memory_pool.rs` 的 `PublishWrites` 一路），不经版本里的表 | 抓字节与落点（①②）；**抓不到**旧版本被别名改写（那不进新版本也不进写日志） |
| B 版本规范文本摘要 | 同上场景，每次发布的 `format!("{:#?}", 版本)` 的 SHA-256，加 `mapping_keys()` 的输出；换表示之后三个新类型的 `Debug` 必须打成与 `Vec` 相同的 `[a, b, …]` | 跨树比较（换前的树与换后的树）；两边各用自己的迭代，不是同一份表代码 | 抓③；表示换了而迭代顺序变了会红。抓不到两边都错得一样（例如两边都漏了同一条）：由 C 补 |
| C 盘上重建对照 | 每次发布之后（仅测试构建）用 `rebuild_version`（`recovery.rs:8047`）从设备字节重建一版，与写者发布出来的版本在三个字段上逐条逻辑相等（顺序比、不看树形状） | 读的是盘上字节、走 `recovery.rs` 的解析，不经写者的表；表的建法也不同（`from_sorted` 对增量改） | 抓写者表与盘上不一致；已有的先例是 `accounting_and_mapping_tree_split.rs:185`–`:189` 的同类比较，只覆盖记账树与映射树 |
| D 表自己 | 与 `BTreeMap` 的随机操作对拍（插入、删除、克隆、克隆之后再改旧克隆不变、`changed_keys_since` 与暴力差集相同）；步 3 与步 4 的影子模式：每次表操作同时在一份 `Vec` 上做，每次发布之后逐条相同；`crates/mutations.tsv` 新增变异行：跳过路径复制、分裂偏一格、`ptr_eq` 快路径不看内容、范围上界差一 | `BTreeMap` 参照放在验证包、与核心不共用代码 | 抓表本身与别名改写；`current_before` 那 8 处整版相等断言兜「换回之后没被改」 |

三条补充：
1. 证红：每条路各取一个已知故障塞进去看它红：A 与 B 把一条分配记录的 `generation` 加 1；C 把重建版本里一个 `PublishedUnit.slot` 改 1；D 跳过路径复制。没抓到的那条路作废，之前「一致」的记录同作废（`.claude/rules/evidence-discipline.md@校验路径本身也要证明它会红`）。
2. 覆盖不是自带的：场景集有没有走到每一种形状（树分裂在每一层、整理搬共享块、快照下覆盖写、回退、落盘失败冻结），由步 0 的计数产物逐项报出每种形状在场景集里发生了几次；某一种 0 次就写「没覆盖」，不写「一致」。
3. 验收口径：每步报「N 次发布、0 次摘要不同」，N 是脚手架数出来的；「查了 0 项」、没报数的一律当没判。

什么会推翻「A、B、C、D 加起来足以证内容相同」：存在一类发布场景集没走到、而在那一类里表示换了会改出不同内容（比如 `units` 里同一角色多项的版本，或 `UnitRank` 与某个构造点的次序不一致）；步 0 的断言正是为了在动手之前把这两类先炸出来。

## 判定一览

标记：【读】读到、【量】命令跑出、【推】推的。判据格 × 候选（丁是本腿加的候选）：

| 格 | 甲 持久共享结构 | 乙 差量加写者索引 | 丙 差量加上一版引用 | 丁 二层分块持久表 |
|---|---|---|---|---|
| 一次发布的代价 | O(d·h)，最坏 d≈N 的几类发布任何表示都 Ω(d)【推】 | O(d)【推】 | O(d) + 压平摊销 O(N/L)【推】 | O(d·B + N/B)，仍随 N 线性、系数约 1/750【推】 |
| 读全版的代价 | 取 O(h)、遍历 O(N)【推】 | 只有最新版；过去版推不出（缺：索引只有一份） | 取最坏 O(链长)【推】 | 取 O(log)、按块给切片【推】 |
| 读者改动面 | `units` 视图基本不改（`&[PublishedUnit]` 签名 5 行【量】）；`allocation_records` 留不了切片，`&[AllocationRecord]` 签名 112 行 / 29 份文件【量】，产品侧约 40、验证包与 tests 约 70【推】；`mapping_keys` src 0 读者、tests 19 行【量】 | 面小，但读过去版本的读者（`history.rs`、`current_before`）失效 | 同甲 | 同甲，切片可按块给 |
| 版本相等与换回 | `ptr_eq` O(1) 否则 O(N) 值相等；换回 = 换回发布前的根，指针相同可钉【推】 | 差量相等使 8 处 `assert_eq!(pool.output, current_before, …)` 空转；换回要双份索引 | 换回最简；相等 O(N) | 同甲 |
| 审计独立 | checker 不依赖核心、崩溃点重放与 GPU 判器输入表不读版本，三者在任何候选下不变【读】；对拍读同一份表代码，靠 C（盘上重建对照）与 D（`BTreeMap` 对拍）补 | 对拍读被审的索引本身 | 同甲 | 同甲 |
| 内存 | 基数 + V·d·h·F【推】 | 基数 + V·d，但过去版不可读 | 基数 + V·d + (V/L)·N【推】 | 基数 + V·(N/B·8 + 改到的块)【推】 |
| 依赖 | 自写约 400–700 行【推】；不引外部包（`crates/*/src` 里 `im::`、`rpds` 零命中【量】）；要不要引：推不出（缺用户口径） | 无 | 约 200 行【推】 | 约 250 行【推】 |
| 分步 | 能（步 0 至 6） | 不能 | 能 | 能 |
| 发布语义不变的证据 | 问题 5 的 A、B、C、D 四路 | 另加合并视图一路 | 另加沿链合出一路 | 同甲 |

问题级判定：

| 问题 | 判定 | 一句话 |
|---|---|---|
| 1 字段与读者 | 答了 | 随池涨的字段列全（15 个，表在问题 1）；按包：核心、验证包、GPU 测试包、挂载运行库 tests 读；checker、单元图、产品 GPU 包不直接读（产品 GPU 包经核心 trait 收记录切片） |
| 2 候选与推荐 | 答了；推荐甲（接口上用丁起步） | 版本被留着被读、换回靠值，乙与这两件事不相容；丙的读代价乘在发布路径的约 U 次取值上 |
| 3 改动与分步 | 答了前半；步 5 推不出 | 表示换了只让克隆、保留、换回变 O(1)；发布本身要 O(d) 还得改建树的几个函数，它们的改动量缺「不动形态两批做完后的函数形状」 |
| 4 三条路 | 答了；整理那条的「发两次」读法推不出 | 换回与整理走法不变、代价降；取号回卷不碰版本表，任何候选下都不变 |
| 5 证明 | 答了方案，没跑 | 四路互不共用代码；覆盖由步 0 的计数产物逐形状报 |

## 与背景材料及上一轮判决不一致、或背景材料没说到的地方（主 agent 判）

1. 读者面：单元图包与池级 checker 不读 `TransactionOutput`（问题 1 开头三条，【量】【读】）；背景「读者按包分」里的这两项读者数是 0。
2. `mapping_keys` 在 `crates/*/src` 里没有读者（只有构造与回退照抄），可以直接改成方法，是最便宜的第一步。
3. `TransactionOutput.units` 的次序不是 `TransactionUnit` 派生 `Ord`（`transaction.rs:2774`、`:2770`、`:2734`、`:2737`、`:17047`、`:17190`、`:17193`）；键要另立 `UnitRank`，并先验证它复刻今天的次序（步 0 断言）。
4. 与上一轮判决的一处冲突：`research/prompts/m4-gpugc-publish-growth-r1-main-verification.md` Q2.3 ② 一格要求 `assert!` 照原失败路径留着，而 `transaction.rs:16245` 的整表断言是 O(R)；留着则记录这一张表买不到 O(d)。交主 agent 判是否放宽（我推的并存做法：发布路径走增量差异、整表断言留测试构建）。
5. 表示换了不够：`build_multi_level_tree`（`transaction.rs:17796`）每次遍历整棵树的每个节点、`CodeTwoTreeVersion.shape` 每片叶带全部 key 且每个 key 是两个 `Vec`（`code_two_tree.rs:77`、`:185`），`plan_the_tree_after_this_publish`（`code_two_tree.rs:637`）收整份 key 集。这几处与不动形态那两批重叠，排次序要两边一起定。

## 每条结论什么现象会推翻它

- 「甲优于乙、丙」：若验证包里读过去版本的用法（`history.rs` 逐发布比上一发布、`current_before`）能全改成「每发布先自己拷一份」而不丢检查力，乙的「读过去版本」一条不再挡；若发布路径对上一版的随机取值能降到每次 O(d) 次，丙的链长乘数不再大。
- 「甲的收益在 `allocation_records`」：若步 0 的计数显示每次发布改到的记录键数 d 接近 R（交错写让大半张表每次都变），收益消失。
- 「`units` 能用键」：若步 0 的断言在任何一处红（次序对不上或同一角色多项），改用按位置的持久序列。
- 「取号回卷不碰版本」：若在 `acquire_instance` 之后到第一次发布之前有任何路径读写 `TransactionOutput` 的这几张表而我没读到（我只读了 `transaction.rs:1112`–`:1500`，没读它的调用方 `mount.rs` 的全部分支），这一条要重判。
- 「A、B、C、D 足以证相同」：见问题 5 末段。

## 没做什么

- 没建模型、没写任何脚本、没起任何任务：所有「一次发布的代价」「内存」公式都是【推】。只有基线（R≈43861、U≈4500、M≈4468 每次发布；`admitted.*`、`publish.allocator_clone_before` 各行秒数）取自 `~/.local/share/singlefs/work/e202-pool-construction-speedup-writer-1/section-timing-after-the-first-speedup-task-19831.txt` 第 857、882、884、886、893、900 行（本腿读了该文件，是主 agent 给的任务 19831 的产物；本机负载高，只看比例）。
- 没量 `size_of`（`AllocationRecord`、`PublishedUnit` 的字节数是按字段估的）、没量每次发布的变化键数 d、没在副本里编过任何东西。
- 没通读 `transaction.rs`（18532 行）：读了 `TransactionOutput` 与发布出口（`:3375`、`:15681`–`:17739` 的装配与构造）、固定点开头、冻结与换回、取号入口；`build_extent_tree`、`build_inode_tree_units`、`build_deletion_tree_units`、`resolve` 的内部、`unlink`、`snapshot` 的各分支只读了调用处，所以它们在步 5、6 里的改动量推不出。
- 没核：`units` 有无同一角色多项、`mapped_units` 的次序是否等于同一个 `UnitRank`、`recovery.rs:4887` 与 `mount.rs:10740` 两处构造点的次序与发布出口一致（都归步 0 的断言）。
- 没复核调查员的 `result.md`（背景里「整理一批发两次」那一句），「发两次」只按代码列了两种读法。
- 没替主 agent 采纳或出判决；没读攻方腿本轮的提示与产出（在禁读清单里）；没往统一任务服务交任务，判决要引的数没复跑。
- 没读 `.claude/kb/` 下条款原文以外的 kb：引用的 kb / 规则条款写成「文件@小节标题」，原文以仓里为准。
- 草稿目录 `~/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r1-sonnet` 里只有认领时建的锁文件，没有仓副本与编译目录（`du -sh` 12K）。

## 要主 agent 跑的命令

无。（没写模型、没写要起任务的脚本。步 0 之后的取证脚手架与各步验收要等主 agent 采纳后由实现员写，命令到那时按共用约束写成 `task-server/task submit` 的整条。）

## 核对用的只读命令与此刻输出（本腿跑的，原样）

```
$ grep -rnE "\.units\b" crates/*/src crates/*/tests | wc -l
386
$ grep -rlE "\.units\b" crates/*/src crates/*/tests | wc -l
89
$ grep -rn TransactionOutput crates/singlefs-unit-graph/src crates/singlefs-unit-graph/tests | wc -l
0
$ grep -n "^singlefs" crates/singlefs-unit-graph/Cargo.toml crates/singlefs-checker/Cargo.toml
crates/singlefs-checker/Cargo.toml:9:singlefs-format = { path = "../singlefs-format" }
$ grep -rn "im::\|rpds" crates/*/src --include=*.rs | wc -l
0
$ grep -rn "TransactionOutput" crates/singlefs-background-gpu/src | wc -l
0
$ grep -rnE "\.mapping_keys\b" crates --include=*.rs | wc -l
27
$ grep -rnE "&\[AllocationRecord\]|&Vec<AllocationRecord>" crates --include=*.rs | wc -l
112
$ grep -rnE "assert_eq!\(pool\.output, current_before" crates | wc -l
8
```
