# m4-close-code-r2 攻方第二次抽样（Opus，2026-11-03）

攻击面：正文第四节第二轮分工——第一轮的修补本身 / 产生结论的方法 / 从未被看过的地方。代码读冻结副本 `~/.local/share/singlefs/work/m4-close-code-r2/repo/`，行号以它为准；下文引的 `crates/` 各文件与主工作区此刻逐字节相同（`cmp` 核过，`crates/mutations.tsv` 除外，那份只按变异名引、不带行号）。

## 复跑命令与模型文件

模型目录 `research/prompts/m4-close-code-r2-opus-s2-model/`：

| 文件 | sha256 |
|---|---|
| `tests/inode_record_counts_on_a_writable_mount_of_overlapping_inode_leaves.rs` | `6a1f01c2c709117554cbb136251d4286c6c52f1f50efdd5879cb117056d402a4` |
| `tools/rerun.sh`（照 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps`） | `92f788397af71d81024d503ff420c231d6c7063a40d9903c11c9e0f61c69fe68` |

复跑整条（在新 shell 里起，`--work` 给一个新目录；命令原样在「要主 agent 跑的命令」一节第 2 条）：

```
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-close-code-r2-opus-s2-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-close-code-r2/repo --work /home/fy5090/.local/share/singlefs/work/m4-close-code-r2-attack-s2/rerun-1
```

模型两段：冻结树上跑模型的两条用例（要都绿）；同一棵树把「按叶序折半找」改成「恒取最右那片」之后，合法两片叶那条要红、`inode_record_counts_kept_with_each_version` 整个目标照绿。三样都合预期 `rerun.sh` 退 0，末行 `name=attack_s2_summary all_three_as_expected=yes`。**下文所有读数都还没跑过，一律是「推的，待主 agent 跑」。**

这个模型不调 `enumerate_crash_states`：打中的是可写挂载收下一份坏镜像之后的发布行为，不是崩溃一致性；没有崩溃状态要枚举，第 3b 步的状态数上限与批次估时不适用（全量状态数 0）。盘是 `SparseBlockDevice` 内存稀疏盘，两条用例各建一次池，不建镜像文件。

## 各格判定一览

| 格 | 攻击面 | 判定 | 一句话 |
|---|---|---|---|
| T4 | 从未被看过的地方 / 修补本身 | **打中**（推的，待主 agent 跑） | 两片 inode 叶交叠的坏镜像过得了可写挂载；按叶序折半找漏掉右片的记录，维护的 n_live、n_记录漂走、标签 6 按漂走的值落盘，最后一批在 `expect` 上 panic。改前按现数算，这两样都没有 |
| T4 | 产生结论的方法 | **打中**（推的，待主 agent 跑） | 钉维护值的那份测试一段历史至多 30 步、建 inode 一步一个，inode 树走不出一片叶；按叶序折半找跨片的那一半没有一个分辨样本，「恒取最右那片」的改坏在它上面照绿 |
| T4 | 修补本身（其余几问） | 没打中 | 会改 inode 记录的路径都经同一份 `record_writes`；回退照抄 R_old 的两个数；恢复数一遍在重建里、不在准入热路径；checker 有 I-3.20 (b) 拿现数比标签 6 |
| T1 | 修补本身 / 方法 | 没打中 | 开片原地改与 `node_origins` 都在 Verus 里证过；只记节点数之后要读孩子数的判断找不到。对拍取样里 `node_origins` 的上一版恒取同一份形状，长高那一形不在对拍里（由证明罩着，记一笔、不算打中） |
| T2 | 修补本身 | 没打中 | 三棵容器树只经 `append_records_to_the_open_container` 与 `carried` 造重写清单，重写的恒是最右端连着的几片；unlink 合并叶、截断、整理搬迁、回退都不改容器树的重写清单 |
| T3 | 修补本身 / 从未被看过的地方 | 没打中 | `units` 装好之后只有 `transaction.rs:3925` 改 `slot`，不挪次序；三处记下标都在装好之后；根之下节点表的次序与重建的逐层读一致；断言失败是 panic，守的是写者自己的不变量 |
| T5 | 修补本身 | 没打中 | 补的两条成形条件按号写与协议按下标写逐条等价；core 抬 F 取的是已发布那一版的号；整理只经排空待发之后起（攒批会话不交出可写的内层会话） |

## 一、从未被看过的地方：按叶序折半找的前提，可写挂载不判（T4，打中）

### 最小反例

输入（模型 `a_writable_mount_of_overlapping_inode_leaves_lets_the_kept_counts_drift_and_the_final_batch_panic`）：两块 4 GiB 内存盘上 mkfs → 取号 → 暖机 → 第一个文件 → 建 232 个空 inode（左片 1..=233 装满）→ 再建 3 个（右片 234..=236，水位 237）。只改盘上两处、校验和沿引用链补到根槽（用 `common_deletion_in_progress::change_the_inode_record` 与 `change_the_unit_and_propagate_its_checksum`）：

1. 左片最后一条 233 号的 inode 号改成 237（左片仍严格升序，size 0、上段没有条目）；
2. inode 树根头里的最大 key 236 改成 237（子树覆盖区间跟着叶走）。

这份镜像违反 I-9.12：「且 > 第 i−1 个孩子的 key 区间上界」（`crates/singlefs-checker/invariants/09-inode树结构.md:24`）——右片的分隔 key 234 不大于左片的上界 237；I-9.4 的「沿叶序容器号严格递增」与「同一片打包记录类型 2 容器里记录号沿记录序严格递增」（`crates/singlefs-checker/invariants/09-inode树结构.md:16`）两句仍成立。

走到的地方（期望 / 实际）：

1. 可写挂载的重建对每片叶只判「if record.inode < separator_key || record.inode < identity.container {」（`crates/singlefs-core/src/recovery.rs:6641`），不判两片交叠；函数头说判的是「inode 叶容器的 I-9.2 / I-1.2 / I-9.4」（`crates/singlefs-core/src/recovery.rs:3199`），I-9.12 不在其中。期望：拒挂、报带名字的错。实际（推的）：收下，起始值 `counted_over_the_inode_tree` 数出 236 条、236 条活的（与现数相等）。
2. 按叶序折半找写着前提「叶容器左起按 key 序、各片的记录按号升序、两片之间不交叠」（`crates/singlefs-core/src/transaction.rs:4184`），做法是「let holding = self.inode_leaf_containers.partition_point(|container| {」（`crates/singlefs-core/src/transaction.rs:4189`）：问 234 号，左片最后一条 237 不小于 234，停在左片，左片里折半找不到，交回 `None`。同一份文档说它「与 [`TransactionOutput::inode_record_of`] 交回同一条」（`crates/singlefs-core/src/transaction.rs:4185`）；逐条找的 `inode_record_of` 交回 234 号。模型里直接比这两个，期望相同，实际（推的）一个 `Some`、一个 `None`。
3. unlink 234 号的意图逐条找记录：「.find(|record| record.inode == inode.0 && record.nlink != NLINK_OF_AN_INODE_BEING_DELETED)」（`crates/singlefs-core/src/transaction.rs:11185`），找得到、发得出；写路径逐片找：「match search_records_by_inode(&containers[position].records, inode.0) {」（`crates/singlefs-core/src/inode_tree.rs:855`），在右片原地换掉。维护那一边问上一版那条记录走的是折半找：「previous.and_then(|version| version.inode_record_found_by_its_leaf(inode))」（`crates/singlefs-core/src/transaction.rs:8390`），拿到 `None`，`after_writing` 走「.or_else(|| record_before_this_publish(InodeNumber(record.inode)));」（`crates/singlefs-core/src/admission.rs:338`）之后当成新记录「counts.records += 1;」（`crates/singlefs-core/src/admission.rs:346`）。期望：意图那一版 n_live 235、n_记录 236；实际（推的）：记着 n_live 236、n_记录 237。
4. 标签 6 按记着的写：「live_objects: resolved.inode_record_counts.live_objects,」（`crates/singlefs-core/src/transaction.rs:6979`）。意图那一版落盘的标签 6 比闭式多 1 × 41 + 1 × 35 字节乘副本数（推的，按 `admission.rs` 的两个常数算：41 + 35 = 76 字节每份副本，两份副本 152 字节）；池级 checker 的 I-3.20 (b)「最新根记账里的已承诺预留（标签 6）对 D3（空间分配） 已定项 2 的闭式现算」（`crates/singlefs-checker/src/walk.rs:21908`）在这一版上该判红（推的，模型没跑 checker）。
5. 接着走空：最后一批问上一版那条记录，`.expect("最后一批删的那条记录在上一版的 inode 树里：不在的那一种 remove_record_from_leaf_containers 已经拒了")`（`crates/singlefs-core/src/transaction.rs:8395`）。删记录那一边逐片找、删得掉，折半找那一边交回 `None`，期望：报带名字的错或照删；实际（推的）：在这一行 panic。

改前（diff 里删掉的 `live_objects_of_the_inode_tree` 与 `inode_records_of_the_inode_tree`）每次按现数逐片数，同一份镜像上两个数恒等于现数、没有这个 `expect`：漂走与 panic 都是这一轮的修补带进来的。病根早就在：写路径的 Verus 前提「leaf_containers_are_well_formed(containers_before_this_publish@),」（`crates/singlefs-core/src/inode_tree.rs:1033`）要的是全部叶记录连起来严格升序「&&& inode_records_strictly_ascend(leaf_records(containers))」（`crates/singlefs-core/src/inode_tree.rs:325`），而调它的是没验证的 `transaction.rs`，跨片升序在盘上读回来之后没有人判；改前写路径逐片找，跨片交叠时碰巧照样找得对，这一轮多了一个真依赖跨片升序的读者。

项目的规矩：「能恢复的失败（I/O 错、盘上数据损坏、资源不够）走 Result」（`.claude/singlefs-ai-sop/rules/code-discipline.md:145`，「错误：能恢复的写进类型，破坏不变量的就断言」那一节），C841 那一条同样写着走得到顶的「坏镜像、外来镜像才有」（`crates/singlefs-core/src/container_tree.rs:3022`）、写路径报类型化错误；这里坏镜像先让一版带着错的标签 6 落盘，再 panic。

### 能不能做成会红的测试

能。模型那条用例断言的是今天的行为（绿 = 缺陷在）；修好之后的那一条把第 1 步改成「可写挂载在任何写之前拒、盘上逐字节不变」，照 `corrupt_on_disk_content_is_refused_instead_of_panicking.rs` 的 `writable_mount_refused_before_any_write` 写，今天会红在「坏镜像被可写挂载收下了」。它进那份坏镜像测试文件，或进 `checker_known_bad_images.rs` 那一族（池级 checker 那一边判 I-9.12 红，读者这一边拒）。

### 打中之后的四句

- 分不分辨臂：这一格没有臂之分，判的是一份实现；打中的是这一轮的改法（按叶序折半找）与改前（逐片数）之间的差：改前不中，改后中。
- 被判的系统当时看不看得到判别它的东西：看得到。可写挂载重建时左片最后一条与右片的分隔 key 都在手里（`recovery.rs` 读每片叶时拿着 `separator_key`），判一句「这一片的最小号大于上一片的最大号」就分得出。
- 满足的是判据字面的哪一个分句：正文 T4「维护值与现数不等时谁会先看见」——维护值与现数在意图那一版就不等，先看见的是 panic（最后一批）与池级 checker I-3.20 (b)（推的）；还满足 T4「哪几条会改 inode 记录的路径……没走 `after_writing` / `after_removing`」的反面：路径都走了，是 `after_writing` 拿到的「上一版那条」错了。
- 跑前条款给的每个改法在打中的那几格上还中不中：正文与 r1 判决没给 T4 的备选改法；下面的表是我自己的，「只在我的模型上量过、被攻过零轮」，而且一格都还没量。

| 改法（只在我的模型上量过、被攻过零轮） | 漂走那一格 | panic 那一格 | 跨片交叠的读者前提那一格 |
|---|---|---|---|
| 可写挂载重建按 I-9.12 判两片交叠（连同 I-9.4 片内严格升序），坏了拒、报带名字的错 | 修（推的，待主 agent 跑） | 修（推的，待主 agent 跑） | 修（推的）：写路径的 Verus 前提也跟着有人判 |
| `after_writing` 与最后一批找上一版那条改回逐条找（`inode_record_of`） | 修（推的，待主 agent 跑） | 修（推的，待主 agent 跑） | 不修：写路径的前提照旧没人判，代价回到每写一条一次全表扫 |
| 最后一批那一处 `expect` 换成类型化错误 | 不修 | 修（推的） | 不修：标签 6 照旧带着漂走的值落盘 |

## 二、产生结论的方法

### 2.1 钉 n_live 与 n_记录的那份测试走不出一片叶（T4，打中）

`crates/singlefs-harness/tests/inode_record_counts_kept_with_each_version.rs` 是这一簇唯一拿维护值逐版比现数的测试。它的取样：

- 随机历史每段「const OPERATIONS_PER_HISTORY: usize = 30;」（`crates/singlefs-harness/tests/inode_record_counts_kept_with_each_version.rs:29`），两组比重是「&GenerationWeights::DELETING_FILES_OF_SEVERAL_LOWER_LEAVES,」（`crates/singlefs-harness/tests/inode_record_counts_kept_with_each_version.rs:226`）与「compare_the_counts_over_the_seeds(&GenerationWeights::BROAD);」（`crates/singlefs-harness/tests/inode_record_counts_kept_with_each_version.rs:247`）；两组里建 inode 只有「(HistoryOperationKind::CreateInode, 40),」（`crates/singlefs-harness/src/history.rs:1275`、`crates/singlefs-harness/src/history.rs:1471`）这一种，一步一个：「经挂着的会话发（`UserChange::NewInodes`，count 1）」（`crates/singlefs-harness/src/history.rs:5917`）。一段历史至多 30 + 1 条记录，一片叶装 233 条，**inode 树恒是零片或一片叶**。
- 两条写死的用例各是 3 条记录、一片叶。

一片叶上 `partition_point` 只有「停在第 0 片」与「越过它、`get` 落空」两种结局，跨片那一半（停在中间哪一片）一个样本都没有。这一簇登记的那一行变异（变异名「n_live 与 n_记录：按叶序找上一版那条记录时找到下一片（片里最后一条找不到、当成新记录）」，`<` 改 `<=`）在一片叶上就红得出来，罩不到跨片。

会红的变异：把 `crates/singlefs-core/src/transaction.rs:4189` 起那一段 `partition_point` 换成 `let holding = self.inode_leaf_containers.len().saturating_sub(1);`（恒取最右那片）。零片与一片叶上它与原文逐条相同，所以 `inode_record_counts_kept_with_each_version` 整个目标照绿（推的，待主 agent 跑 `rerun.sh` 第二段）；模型的第二条用例 `unlinking_a_record_of_the_left_leaf_of_two_keeps_the_counts_equal_to_the_counts_over_the_tree`（合法历史：两片叶、unlink 左片的 5 号到底）在它上面红（推的，同上），在冻结树上绿（推的，同上）。

别的测试会不会顺带罩住：按 inode 号写、部分写、别的 inode 改 size 那几组比重一次建至多 160 个，走得到两片叶；它们若每一步都跑池级 checker，I-3.20 (b) 会拿现数比标签 6、漂走就红。哪几条测试真这样跑、跑的步里有没有改左片记录的，我没查（「我印象里」那一档，不当证据）；`rerun.sh` 只钉这一份测试照绿这一件。

### 2.2 Y5 桥补的两条，必红世界只落在它自己的见证上（T5，方法上打中、不涉实现行为）

r1 判决给的活是「核实缺着就补不变式与测试」（`research/prompts/m4-close-code-r1-main-verification.md:47`）。这一轮补进 `root_ring_is_shaped`（`crates/singlefs-core/src/verified_model.rs:212`）的两条，必红世界表登记了两行：「根环模型·成形去掉「内存 F 不高于最后一条根的号」」（`crates/singlefs-core/verus-must-red-worlds.tsv:453`）与「根环模型·成形去掉「持久 F 之上的根装得进根环」」（`crates/singlefs-core/verus-must-red-worlds.tsv:454`），落点都只有 `witness_the_root_ring_shape_keeps_a_root_above_the_floor_and_fits_the_window` 一个函数。

全仓 `grep -rn root_ring_is_shaped crates/`（冻结副本）只命中定义与见证里的三处断言：没有一条引理以它为前提，core 里也没有从 `RootRingModel` 到协议 `RunRingState` 的映射引理，没有一条测试拿 core 运行时的根环、内存 F、持久 F 去判它。去掉任何一条，除见证以外每条证明照过——**这两条不承重**。必红世界证的是「见证分得出补没补」，不是「桥接上了」。r1 判决表里这一格的病是「桥的谓词写了没接」（`research/prompts/m4-close-code-r1-main-verification.md:34`），补完之后谓词多了两条，照旧没接。

能不能做成会红的东西：桥本身要么写一条映射引理（core 的成形 ⇒ 协议的 `run_ring_is_shaped`，按号与按下标的换算在引理里证），去掉两条之一时落点里多出这条引理；要么在 harness 里拿真历史（抬 F、卸载、准入抬 F、整理）每一步之后读根环与两层 F 判这两条（推的：core 抬 F 取已发布那一版的号——「let rollback_floor = current_file_version.root.checkpoint_txg;」（`crates/singlefs-core/src/mount.rs:4380`），第一条我推它成立；第二条靠准入在根环转满之前抬 F，我没核）。

### 2.3 容器小树的对拍取样（T1 / T2，记一笔、不算打中）

`node_origins` 的对拍把上一版取成同一份形状：「node_counts_before_this_publish: shape.node_counts(),」（`crates/singlefs-core/src/container_tree.rs:4087`），长高、新开右半那几形（上一版没有那个位置）不在对拍里；对拍的扇出取「for entries in [2, 3, 135] {」（`crates/singlefs-core/src/container_tree.rs:4029`），产品扇出 135 开 1 到 40 片只有一层。这两处由 Verus 罩着（`node_origins` 的 `requires` / `ensures` 与 `after_a_container_is_opened_at_the_right_end` 的 `ensures` 都在 725 项里证过），对拍不承重；但从没验证的 `transaction.rs` 调进来时 `requires` 不在运行时判，留下的运行时防线只有「self.node_counts_before_this_publish.has_a_node_at(position),」（`crates/singlefs-core/src/container_tree.rs:3009`）那一句断言。我找不到一条走得到它的输入（容器树只在最右端开片，见第三节 T2），不算打中。

## 三、第一轮的修补本身：五格逐格（打中的见第一、二节，这里写没打中的那几问）

### T1　开片原地改、上一版只记每层节点数

- 长高、正好填满一层、连开几片跨层：`after_a_container_is_opened_at_the_right_end` 的 `ensures` 是 `levels_after_opening_at_the_right_end`（与改前逐层照抄那一套同一份 spec），Verus 证过；按值交进来、`Self { .. } = self` 拆开原地改，不留共享的旧形状给调用方。没找到运行时能绕过证明的输入。
- 只记节点数之后读不到孩子数的判断：上一版只被问「某位置有没有节点、是不是根、根在哪、根的层级」四样（`has_a_node_at`、`is_the_root`、`root_position`、`root_level`），`previous_root_is_carried_down_unchanged` 改读这一版的 `rewritten_nodes`。照抄节点的字节与指针取上一版同一位置（`build_container_tree_units` 的 `carried_node_below_the_root_pointer`），不读上一版的孩子数。没找到。

### T2　求祖先从最右端往左

前提「重写的是最右端连着的那几片」在三棵容器树（墓碑、谱系、退役）上由构造保证：「造它的只有 [`append_records_to_the_open_container`] 与 [`Self::carried`]」（`crates/singlefs-core/src/container_tree.rs:2726`），全仓 `ContainerTreeAfterThisPublish {` 的字面构造只在 `container_tree.rs` 的追加函数与它自己的测试里。逐条路径过：追加（墓碑、退役、谱系）走追加；unlink 合并叶、截断、整理搬迁改的是 inode 树、extent 树与映射，不改容器树的重写清单；回退那一版容器树取现行那一版：「tombstone_containers: current.tombstone_containers.clone(),」（`crates/singlefs-core/src/mount.rs:10214`），回退那次发布在它上面往谱系树追加；回退之后第一次发布同样接在现行那一版上。重写清单不是最右段时 `rightmost_count_of_the_rewritten_containers` 逐片断言、代价是重写片数。没打中。

### T3　孩子区间顺着累加、按位置取指针、记两个根单元的下标

- `units` 的次序：全仓改 `units` 的只有映射位置盖提示那一处「version.units[*position].slot = slot;」（`crates/singlefs-core/src/transaction.rs:3925`），改 `slot`、不挪次序；三处记下标「let container_tree_root_unit_positions = ContainerTreeRootUnitPositions::of_the_units(&units);」（`crates/singlefs-core/src/transaction.rs:16584`、`crates/singlefs-core/src/recovery.rs:3394`、`crates/singlefs-core/src/mount.rs:10178`）都在装好 `units` 之后、构造 `TransactionOutput` 之前；全仓 `TransactionOutput {` 的字面构造只有这三处。搬迁、清扫、截断、挂载续做都经 `publish_admitted` 出新的一版，不改旧一版的 `units`。没找到挪次序的地方。
- 按位置取指针：下标按「.map(|level| self.shape.nodes_at_level(level))」（`crates/singlefs-core/src/transaction.rs:4026`）把下面各层节点数加起来；根排在 bump 次序最末、不在表里，不影响根之下各项的下标。重建那一边逐层往下读、再「levels_from_the_root_down.reverse();」（`crates/singlefs-core/src/recovery.rs:5638`）成层级升序、同层左起，与发布那一边按 `node_origins` 的 bump 次序同序。
- 断言失败是 panic：三处断言守的都是写者自己装出来的表（发布、重建、回退装 `units` 与指针表），盘上读来的形状已在 `from_children_of_each_node_at_each_level` 与 `read_container_tree_index_nodes` 判过；照「不变量被破坏是 bug，用断言」办合规。没打中。

### T4　其余几问

- 会改 inode 记录的路径：普通分配那一半里写进的都在 `record_writes` 一张表里，`inode_tree` 与两个数用同一张（「inode_record_counts_before.after_writing(&record_writes, record_before_this_publish)」，`crates/singlefs-core/src/transaction.rs:8405`）；删掉的只有最后一批那一条（`crates/singlefs-core/src/transaction.rs:8393`）。截断两种批、整理搬迁、快照建删不改 inode 记录，`record_writes` 空、两个数照抄。回退照抄 R_old 那一版重建时数出来的两个数（`crates/singlefs-core/src/mount.rs:10234`）。合法历史上我没找到漏走的路径。
- 谁先看见：池级 checker 的 I-3.20 (b) 拿现数比标签 6（`crates/singlefs-checker/src/walk.rs:21908`）；运行时没有自比。坏镜像那一形的先看见见第一节。
- 恢复数一遍在不在热路径：在 `rebuild_version`（`crates/singlefs-core/src/recovery.rs:3396`），每重建一版数一遍；挂载一次、回退时 `highest_inode_number_watermark_in_the_ring`（`crates/singlefs-core/src/mount.rs:10038`）对环里每条根各重建一次也各数一遍。重建本来就逐片读回每片叶，多数一遍不改量级；准入只读随一版带着的两个数。不在准入热路径上。

### T5　整理门与根环成形

- 协议这一门与 core：core 的后台整理在攒批会话里「在这一判说起跑之后、跑之前先排空待发」（`crates/singlefs-core/src/mounted_session.rs:2427`），排空被拒就不跑；攒批会话只交出不可变的内层会话（`crates/singlefs-core/src/mounted_session.rs:5163`），没有绕过排空起整理的口子；排空交错盘表那一形有变异行钉着。发布冻结之后整理停不停是 r1 已判过的前提，这一轮没改它，不重攻。
- 补的两条按号写与按下标写：根按号严格递增时，「内存 F 之上至少一条根」与「最后一条根的号不低于内存 F」等价（注释 `crates/singlefs-core/src/verified_model.rs:205` 写的就是这一步），受保护的根是一段后缀、条数不超过 window 与「每个受保护的下标不小于 len − window」等价（`crates/singlefs-core/src/verified_model.rs:219`）。换算我逐条推过，对。core 的值上成不成立见 2.2：没有一样东西判它。
- 发出协议不许的整理发布的历史：没造出来（试过的形状见末节）。

## 要主 agent 跑的命令

两条都不是重型（singlefs-harness 的测试目标，不碰 checker 档包）。

1. 在草稿目录里的仓副本上只跑模型那个测试目标（副本 = 冻结副本加一份新测试文件，`diff -rq` 核过只多这一份；在那个认领目录的 `.deliverables` 写成了「交接」）：
   ```
   SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 inode_record_counts_on_a_writable_mount_of_overlapping_inode_leaves; 范围=1 个测试目标; 为什么=核攻方第二次抽样第一节坏镜像那条与第二节合法两片叶那条在冻结副本上的结局' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-close-code-r2-attack-s2/repo-copy -- cargo test -p singlefs-harness --test inode_record_counts_on_a_writable_mount_of_overlapping_inode_leaves -- --nocapture
   ```
   在哪起：`--cwd` 是 `/home/fy5090/.local/share/singlefs/work/m4-close-code-r2-attack-s2/repo-copy`。看哪几行：`TASK_SERVER_RESULT` 那一行 `type=` 是成功、`exit=0`；`task-server/task log <号> --all` 里 `test result: ok. 2 passed`，以 `attack-s2:` 起头的观测行里有「可写挂载收下」「234 号逐条找 Some(234)；按叶序折半找 None」「意图那一版 …记着的 InodeRecordCounts { live_objects: 236, records: 237 }，现数 InodeRecordCounts { live_objects: 235, records: 236 }」（括号里的数是我推的，以日志为准）与「走空 panic：最后一批删的那条记录在上一版的 inode 树里…」。坏镜像那条若红在「可写挂载拒了这份坏镜像（攻方的前提不成立）」，第一节的打中撤回、拒的成员原样记下；红在别处（造池、改盘的前提断言）是模型写错，交回给我接着改。
2. 模型整份复跑（照第 9 步；含冻结树一段与改坏一段）：
   ```
   task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-close-code-r2-opus-s2-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-close-code-r2/repo --work /home/fy5090/.local/share/singlefs/work/m4-close-code-r2-attack-s2/rerun-1
   ```
   在新 shell 里起，工作目录随意（判决复跑时把模型目录拷进主 agent 的草稿目录，`--work` 换成那边的新目录）。看哪几行：`task wait` 末行 `exit=0`；`log` 里 `name=attack_s2_frozen_model_test rc=0 expected=0`、`name=attack_s2_mutant_two_leaf_control rc=<非 0> expected=nonzero`、`name=attack_s2_mutant_existing_counts_test rc=0 expected=0`、`name=attack_s2_summary all_three_as_expected=yes`。改坏一段的锚点对不上时 python 那一步断言失败，日志里有「改坏的锚点在 transaction.rs 里恰好一处」。脚本里三条 `cargo test` 各自带了范围声明。

等这两条的结果才能往下做的：第一节打中与 2.1 打中的全部读数、改法表里每一格「推的，待主 agent 跑」。

## 没打中的形状与取样范围

- T2：逐条翻了会重写容器的六条路径（追加、开片、unlink 合并叶、截断、整理搬迁、回退之后第一次发布），找构造重写清单的每一处（全仓 `rewritten_containers` 与 `ContainerTreeAfterThisPublish {`）；没有一条让重写清单离开最右段。
- T3：全仓找改 `units` 的写法（`.units.push/retain/sort/insert/remove/swap/extend/truncate/drain/clear/iter_mut`、整字段赋值、`mem::take/replace/swap`）与 `TransactionOutput {` 的字面构造，只有三处构造与一处改 `slot`；比了重建与发布两边根之下节点表的排法。
- T1：读了开片原地改与 `node_origins` 的运行时断言与证明前提，找从没验证的调用点进来时 `requires` 可能不成立的输入（盘上读回的非右满形状、测试扇出换回产品扇出、回退那一版的容器树）；都只在最右那条路径上加节点。
- T4：逐个过 `DeletionStepOfThePublish` 的九个成员与 `inode_record_writes`；比了回退、整理一批、快照建删三条发布与重建的数法；造了合法的两片叶历史（模型第二条用例）——合法历史上维护值不漂（推的）。坏镜像只试了「左片末条改大、越过右片」这一形；片内乱序、重复号、空叶（重建拒）三形没进模型。
- T5：核了补的两条按号与按下标的换算；找绕过排空起整理的口子（攒批会话只交出不可变的内层会话，后台入口先排空）；没造出 core 发出协议这一门不许的整理发布的历史。冻结之后整理停不停是 r1 判过的前提，没碰。
- 取样范围：这一腿没有随机历史，模型两条用例各一段写死的历史（建 235 个空 inode、unlink 一个），没有分批、没有缩。

## 这条腿自己的限度

- 模型一条都没跑过：腿不交任务。坏镜像那条依赖 `change_the_inode_record` 那一串把新校验和补到根槽、journal 记录新根段之后可写挂载照常择到这条根；我没跑，不知道择根、journal 施加那一步会不会因为别的原因拒或落到更早的根。拒的话第一节撤回。
- 「改前没有漂走与 panic」是按背景材料的改动全文读出来的（删掉的两个按现数数的函数、新加的 `expect`），改前的代码不在冻结副本里，没在改前的树上跑。
- 2.1 里「别的测试会不会顺带罩住」没查，只写了查法。
- 2.2 只说桥没接上；core 的值上那两条成不成立，第二条我没核。
- 改法表三行都是我自己提的，「只在我的模型上量过、被攻过零轮」，而且一格都没量。
- 没读的：r1 各腿报告（不在禁读清单，但第一轮已判成立的结论不重攻，用不上）；kb 决策条款没有引用，打中的两条不依赖哪一条已定项的措辞。

## 草稿目录与副本

- 留着：`/home/fy5090/.local/share/singlefs/work/m4-close-code-r2-attack-s2/repo-copy`（47M，冻结副本加一份测试；`.deliverables` 写成「交接」，主 agent 照第 1 条命令在里面跑，用完删）。
- 草稿目录里另有两份我写的辅助文本（`rerun_steps.txt`、`rerun_steps2.txt`，拼 `rerun.sh` 用的片段），没有编译目录、没有库拷贝。

## 没做什么（固定会有的）

不判正推、辩方那几格；不替主 agent 采纳；副本上量出的数不算入库装置上的数；没往统一任务服务交任务，判决要引的数没复跑（命令在上面「要主 agent 跑的命令」一节）。
