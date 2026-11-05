# m4-admission-delta-r1 攻方 opus-s2 抽样报告（2026-11-05）

立场：攻方（假设主 agent 的倾向是错的），攻击面照正文第五节第一轮分工：过度外推与越位 / 内部矛盾与循环依赖 / 哪一条最脆弱。
代码读冻结副本 `/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/`（下文代码引文都写这份副本的绝对路径；`transaction.rs` 与 `mounted_session.rs` 在主工作区已经改过，行号只对冻结副本）。
没读禁读清单里的三样。不下「选哪条」的结论。

## 复跑命令与模型文件

模型目录 `research/prompts/m4-admission-delta-r1-opus-s2-model/`：

| 文件 | sha256 |
|---|---|
| `admission_before_full_planning_reorders_refusals.rs` | `baccfecd85896dab2627c8fab8b589c13f5de208d021d777a541dee09a612eca` |
| `tools/apply_a3_emulation.py` | `72e1b8c2764237a6901d18f0c05ac461e5d4d186e1f7492c05b1b52fc011f37c` |
| `tools/rerun.sh` | `8e26f6b936ab533112f80df37a388c31cff0173dc4d35d68680535ff8594e182` |

- `admission_before_full_planning_reorders_refusals.rs`：harness 档测试目标（不进 checker 档包），四条用例，盘都是内存稀疏盘。`tools/rerun.sh` 把它拷进冻结副本的 `crates/singlefs-harness/tests/`，跑两遍：今天的代码一遍；`tools/apply_a3_emulation.py` 给副本的 `judge_the_publish_before_any_read_or_write` 套上「候选 A3 模拟」（式子那一判挪到走固定点之前，需求取普通分配加 ckpt_cost）之后再一遍。
- 复跑：`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-admission-delta-r1-opus-s2-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo --work /home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1-attack-s2/rerun-1`（详见末尾「要主 agent 跑的命令」）。
- `rerun.sh` 的 `--selftest` 在这份拷贝上不成立：模板自证用的假冻结副本里没有 `Cargo.toml`，换了 `rerun_steps` 之后那一格必红；只用上面那条真跑。
- 这一腿没跑过任何一条命令（照定义不交任务），下文凡是模型的数都写「待主 agent 跑」。

## 各格判定一览

| 编号 | 打的是哪条候选、哪一格 | 判定 | 依据的形态 |
|---|---|---|---|
| H1 | A3（A3 格、A5 格）：拒成哪一种与今天不同 | 打中（代码读出的次序；模型待主 agent 跑） | 式子已经不够、而这次顺序写切出的记录多于在飞上限：今天交回「记录超上限」、不推抬 F；A3 先见空间不够、推抬 F（写空发布、抬 F、丢回退候选），之后再报原来那一种或报 ENOSPC |
| H2 | A3 的变体（把记录条数也挪到差量上界上数） | 打中（算术，模型打数；待主 agent 跑） | 1 MiB 环、4 GiB 盘上顺序写 85 个数据单元今天照发；记录条数按 ckpt_cost 罩固定点去数，条数 84 + ⌈(1 + ckpt_cost) ÷ 64⌉ > 85，被拒成记录超上限（非空间、不推抬 F） |
| H3 | A3：上界放行而真实不够 / 上界提前拒 | 一半打中（推的 + 量数待跑） | 代码里现成的固定点上界 ckpt_cost 是「空发布」的上界，不罩写入类发布这次插进中央映射树的 key；今天靠走完固定点之后按真数的那一判兜底（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12621`），A3 把它挪走就没了兜底。另一头：分配记录树能拿得准的上界只有 K1，A3 判式子时多扣 K1 − 真写的分配记录树节点数 |
| H4 | A1、A2、A3、A4 四条都没罩的那一步 | 打中（代码读出；行数增长的模型待主 agent 跑） | ckpt_cost 的记账树那一项每次发布在准入之前全量规划一遍「下一次空发布之后的记账树」：逐行过滤全部记账行、把全部 inode 号水位台阶行收进 BTreeSet、交给全量规划器；台阶行每次建 inode 的发布多一行、一直留着。它在式子可用那一侧（保留池、切换预留的 c_max），A3 挪不走，A2 的定义只管「这一次的规划」，A4 不动它的 BTreeSet |
| H5 | 越位（正文与材料） | 打中（文本核对） | A3 与 D28（挂载期承诺量） 已定项 1「接线」两句逐字相冲（式子「算定这次发布的形状之后」判、「需求 = 这次发布新写的全部槽」），而背景材料没有 D28 的小节清单、正文 A5 格也没点名它 |
| H6 | A2：差量规划与全量规划重写的节点集合是不是逐个相同 | 打中一处要求（推的，没模型） | 中央映射树的 key 集合是「进映射的单元 ∪ 被快照留存的条目」的并；一个单元这次被杀而被快照钉住，key 从前一份挪进后一份、并集不变：全量规划不动它那片叶，按两份清单各自的进出算差量的 A2 会先删后插、重写那条路径 |
| H7 | A2、A4：节点身份按位置 | 线索（推的） | 记账树、中央映射树根之下节点的角色带「这一版的位置」，左边一片叶切开、右边同层每个节点的位置都变；规划器摊平时给全部节点重编号、换下集合逐个扫上一版全部节点 |
| L1 | 今天的环窗口 W（D28（挂载期承诺量） 已定项 1）与 ckpt_cost 的 K1 说法互相打架 | 线索（推的，没模型，不在本腿判的五格里） | W 里分配记录树每层取 min(N_ℓ, 2·(K + 26))，是一个按差量的上界；ckpt_cost 的注释说分配记录树的不动点「收不了口，拿得准的上界只有几何上整棵树」 |

没打中的形状与取样范围见「没打中的形状」一节。各条候选的代价量级见「各条候选的代价量级」一节。

## 攻击面一：哪一条最脆弱——A3 把「拒成哪一种」换掉了（H1、H2）

### H1　式子挪到全量规划之前，非空间的拒绝被空间拒绝抢先，会话推抬 F

今天 `judge_the_publish_before_any_read_or_write` 的次序（冻结副本，逐行现查）：

1. 落点那一判：「admit_the_units_landing_on_every_device(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12523`），固定点那一截拿 ckpt_cost 顶：「checkpoint_cost_of_the_version_to_build_on_with_node_capacities(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12527`）。
2. 全量规划（固定点）：「let settled = settle_the_allocation_record_tree(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12536`）。多层码 2 树的拒绝、释放判定的错、`AllocationRecordTreeRewriteSetDidNotSettle` 都在这里交回。
3. 记录条数：「let journal_records_of_the_publish = records_of_a_publish_naming_the_relocated_copies(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12549`）、「refuse_a_publish_of_more_journal_records_than_the_limit(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12554`）——条数按 `settled.resolved.rewritten_roles` 数，要全量规划之后才有。
4. 合取 7：「admit_the_empty_cluster_segments_floor_on_every_device(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12577`）。
5. 式子：「admit_on_every_device(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12621`），需求是走完固定点的真数。

会话只对两种拒绝推抬 F：「PublishError::SpaceAdmissionRefused(_) | PublishError::PlacementRefused { .. } => true,」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/mounted_session.rs:1873`）；记录超上限原样交回：「| PublishError::JournalRecordsOfThePublishExceedTheLimit { .. }」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/mounted_session.rs:1895`），同一张表里 `MultiLevelCodeTwoTreeRefused`、`AllocationRecordTreeRewriteSetDidNotSettle`、`Release*` 那几种也是原样交回。

A3 的定义是「准入之前只由差量算出这一次分配量的上界……全量规划照今天的写法挪到准入之后」。第 3 步的条数、第 2 步的那几种拒绝都要全量规划才算得出，于是只能排在式子之后（除非连它们也改成上界，那是 H2）。

**最小反例（历史）**：两块内存盘、1 MiB journal 环（在飞上限 256 ÷ 3 = 85 条）、单元区取「第一个文件之后式子那一判每块盘可用少于 172 槽」的那一档（模型从 4000 槽按 32 槽一步往下扫，取头一个满足的）；mkfs → 取号 → 暖机 → 第一个文件 → 经会话顺序写 86 个数据单元。

| | 今天（冻结副本） | A3（副本上的模拟） |
|---|---|---|
| 走到哪一判先拒 | 落点那一判过（原始空槽远多于 172 槽：式子另扣环窗口、切换预留、保留池），全量规划过，记录条数 86 > 85 拒 | 式子在全量规划之前判：需求至少 86 × 2 = 172 槽（这只是数据单元，确切需求更大），可用 < 172，拒成 `SpaceAdmissionRefused` |
| 会话做什么 | 原样交回 `UserChangeRefused::Publish { cause: JournalRecordsOfThePublishExceedTheLimit { 86, 85 } }`，一串抬 F 都不推 | 推抬 F 的空发布（每一次落盘、F 往上抬），再判；推满 8 次仍不够报 `NoSpaceAfterRaisingTheFloor`，够了就走到全量规划、再报记录超上限 |
| 盘上 / F | 逐字节不变，F 不动 | 多了若干条空发布的根，F 抬高，回退候选集里的旧状态少了 |
| 调用方被叫去做什么 | 把改动切小 | 腾空间（ENOSPC），或切小——且已经丢了回退目标 |

这里的分辨与 A3 的上界取多紧无关：可用低于这次写的数据单元本身，任何不小于真需求的上界都拒。

**四句**：
- 分不分辨臂：分辨。A1、A2（规划仍在准入之前、只换算法）、A4（只换表示）次序不变，都交回记录超上限；只有把式子挪到全量规划之前的 A3 换了拒绝种类。
- 被判的系统当时看不看得到判别它的东西：看得到。记录条数今天就在准入之前算好（`journal_records_of_the_publish`），A3 要么把它也挪后（看不到），要么另给一个上界（H2）。
- 满足的是判据字面的哪一个分句：正文第一节「准入判出的结论（放不放行、拒成哪一种）与今天相同或可证更严」里的「拒成哪一种」——A3 在这条历史上拒成了另一种，且多写了盘、抬了 F；「可证更严」盖不住：推抬 F 是有副作用的动作，不是「更严的拒绝」。
- 跑前条款给的每个改法在这几格上还中不中：正文没给改法。我能想到的两种：(a) 把记录条数也改成上界（H2，引出另一种不同）；(b) 在式子之前先跑一次只数记录条数的规划——那就是把全量规划留在准入之前，回到 A1/A2。两种都修不掉这一格。

**做成会红的测试**：模型里 `a_write_past_the_journal_limit_on_a_pool_short_by_the_formula_is_refused_for_its_records_without_raising_the_floor` 钉今天的结局（记录超上限、`floor_raises` 为空、盘逐字节不变、F 不动），今天该绿、套上 A3 模拟该红。推的，待主 agent 跑（rerun.sh 的 `name=reorder_probe` 与 `name=rerun_verdict` 两行）。同形还可以换成别的「全量规划才判得出」的拒绝：从盘上重建、分隔 key 坏了的中央映射树（D8（核心索引结构） 已定项 11「从盘上重建的树分隔 key 坏了」那一条），今天交回 `MultiLevelCodeTwoTreeRefused`、盘上不变，A3 下先推抬 F 再报它——这一形没写模型。

### H2　A3 的变体：记录条数也按上界数，今天照发的写被拒成「切小」

若 A3 为了躲开 H1 把记录条数也挪到准入之前、按上界数，固定点那一截能用的上界是 ckpt_cost（今天落点那一判就这么顶：「每个一槽（码 2 节点与树表单元都是一槽）——这次重写分配记录树哪几个节点要在分配器上走到固定点才知道，拿 ckpt_cost 的最坏数」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:1019`））。一条记录点名 64 项，末条要点名「最后那个数据单元 + 固定点那几个角色 + 普通分配的非数据角色」。

**最小反例**：1 MiB 环、两块 4 GiB 盘、第一个文件之后顺序写 85 个数据单元：今天切成 85 条、等于上限、照发（仓里现成的 `a_sequential_write_cut_into_as_many_journal_records_as_the_in_flight_limit_is_published` 同形）。4 GiB 盘单元区 [1088, 262144) 上分配记录树每盘 322 片叶、层级 1 两格，K1 = 1 + 2 × 324 = 649（推的，按 `allocation_record_tree_positions_over_the_unit_areas` 的式子手算；模型打出真值）；上界口径条数 ≥ 84 + ⌈(1 + 649) ÷ 64⌉ = 95 > 85，拒成记录超上限：非空间，会话不推，调用方被叫去切小一个今天照发的写。
- 量级外推（推的）：默认环 768 MiB 的在飞上限 65536 条；K1 ≈ 每盘单元区槽数 ÷ 812 × 2，⌈K1 ÷ 64⌉ > 65536 要 K1 > 4194304，约每盘 25 TiB 起。到那一档，这一变体下每一次带普通分配的发布都被拒成记录超上限，与剩多少空间无关。
- 四句：分辨（只有按上界数条数的变体中）；看得到（条数今天准入之前就数）；满足「放不放行」那一分句（今天放行、变体拒）；改法——把固定点那一截的上界收紧到按差量（见 H3：今天代码里没有拿得准的差量上界）。
- 测试：模型 `a_write_at_the_journal_limit_is_published_today_and_a_fixed_point_bound_from_the_checkpoint_cost_would_cut_it_past_the_limit` 打 `name=bounded_journal_count`，断言今天照发、上界口径条数 > 85。待主 agent 跑。

### H3　A3 的上界：现成的那个罩不住写入类发布，拿得准的那个又松到 K1

今天准入读的是「一半上界、一半真数」：落点那一判用普通分配的真数 + ckpt_cost（上界），式子用走完固定点的真数。A3 要的是一个只靠差量就成立的固定点上界。代码里现成的两个：

- **ckpt_cost** 是「一次空发布至多写出的固定点单元数」，它的中央映射树那一项只数空发布会改的 key：「一次空发布在中央映射树里改的条目是这两棵树的节点（别的进映射的单元照抄上一版、key 不变）：重写的每个节点删一把旧 key」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:1240`）。写入类发布还往中央映射树里插这次每个新数据单元、extent 与 inode 节点的 key；上一版中央映射树小（条目 + 空发布的插入数装得进一片叶 ⇒ 那一层记 0 次切分）、这次插很多 key 时，真写出的映射树节点多于 ckpt_cost 里那一项。ckpt_cost 另有谱系、退役两项（各 3 块左右）与 K1 减去真写的分配记录树节点数当余量；余量吃完就是「A3 放行、真实不够」：放行之后保留池被这次发布吃掉一截，下一次空发布的固定点不再有保证（D28（挂载期承诺量） 已定项 1「它保证 checkpoint 自己的固定点写得出去」那一项落空）。
  - 推的门槛（产品节点容量，没量）：中央映射树叶 294 条左右、从中间切；第一个文件之后一次插 N 个数据单元，叶约 N ÷ 147 片；窄盘上 K1 ≈ 每盘单元区槽数 ÷ 406、K1 与真写的分配记录树节点数几乎相等时，余量约 6 块，N 约 1000 以上才翻过去，而那要 journal 在飞上限 ≥ N，环要 12 MiB 以上、单元区要容得下 2N 槽加环窗口——是一条大写，不是常见形。
  - 模型（量的待主 agent 跑）：`the_fixed_point_a_write_publishes_against_the_checkpoint_cost_of_the_version_it_builds_on` 在 1 MiB 环、单元区 2000 槽的窄盘上，产品容量与只供测试压小的映射树（叶 4、内部 8）两档，第一个文件之后顺序写 1 / 10 / 20 / 40 / 85 个数据单元，打 `name=fixed_point_vs_checkpoint_cost … excess_slots=…`：`excess_slots` > 0 的那几格就是 ckpt_cost 罩不住这次固定点的反例。我预期压小那一档 40、85 两格为正、产品那一档全不为正（推的）。压小容量是只供测试的旋钮，它说明的是「ckpt_cost 的式子不数写入类发布插的 key」这件事，产品容量下要上面那条大写。
  - 今天为什么没出事：式子那一判在走完固定点之后按真数判，落点那一判的低估由它兜底，放行之后落点再取不到也照「准入放行而落点取不到 ⇒ 先推抬 F」办。A3 把按真数的那一判挪走，兜底就只剩「规划出的实际量不超过上界由断言守住」——断言打中时要么 panic，要么在准入之后补一种拒绝，正文的 A3 定义没说是哪一种。
- **K1**（几何上整棵分配记录树）：「一次发布改几片叶的不动点收不了口，拿得准的上界只有几何上整棵树。代价随单元区线性涨（两块 4 GiB 盘 529 个节点）。」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:1193`）。A3 若照这句取 K1，式子比今天多扣「K1 − 这次真写的分配记录树节点数」：两块 4 GiB 盘约 529 − 个位数 ≈ 520 槽每盘（约 8 MiB）；两块 16 TiB 盘 K1 约 266 万槽每盘（约 40 GiB，推的）。这是「提前 ENOSPC」：推满 8 次抬 F 也腾不出来（扣的是上界与真数之差，不是被钉住的空间），于是 C283（准入失败时不先推发布就报 ENOSPC） 那一条的「先推再报」照做了、仍报 ENOSPC。它与 D16（发布语义） 已定项 1「准入」那一行合不合，取决于读「做满仍不够才报 ENOSPC」时「不够」按真需求还是按上界——那一行没说；D28（挂载期承诺量） 已定项 1 说了，见 H5。
- **按差量的上界有没有**：L1 那一条说今天代码里同时有一个按差量的分配记录树上界（W 里每层 2·(K + 26)）。要么它对、K1 那句注释过严（A3 能用它），要么 K1 那句对、W 少扣（今天的环窗口有洞）。这一腿没量，交主 agent 判是哪一边。

四句（H3 两头合起来）：分辨（只 A3 换口径；A1/A2/A4 式子仍按真数）；看得到（ckpt_cost、K1 都在准入之前算得出）；满足「放不放行」分句（K1 那头今天放行、A3 拒；ckpt_cost 那头 A3 放行、今天拒）；改法——给写入类发布另立一个数插入 key 的映射树上界（推的式子：把 `central_mapping_tree_nodes_an_empty_publish_rewrites_at_most` 的 `inserted_at_most` 换成「空发布那一份 + 这次普通分配的角色数」），它修 ckpt_cost 那一头，修不了 K1 那一头。只在我的推导上成立、被攻过零轮。

**分配记录树的固定点在上界下怎么办**（A3 格点名的那一问）：今天的固定点在分配器拷贝上走（`settle_the_allocation_record_tree`），它本身就是「全量」的一部分；A3 把它挪到准入之后，准入读的只能是 K1 或 L1 那个 W 式上界，两样各有上面那个代价或疑问。没有第三样现成的。

## 攻击面二：内部矛盾与循环依赖——四条候选都没罩的那一步（H4）

正文 A1 格点名了「记账树的行表 `transaction.rs:7978`」，把记账树归在「这一次的多层码 2 树规划」里。还有第二处全量规划记账树、而且排在准入之前的：ckpt_cost 的记账树那一项 `accounting_tree_nodes_after_the_publish_built_on`（admission.rs，与主工作区逐字节相同）。它规划的是「接在这一版后面的那次空发布之后」的记账树：

- 逐行过滤这一版全部记账行：「.filter(|entry| entry.generation == version_txg)」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:1339`）；
- 把全部 inode 号水位台阶行并进 key 集合：「.chain(version_to_build_on.inode_number_watermark_steps())」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:1349`）；
- 交给全量规划器：「let nodes = match plan_the_tree_after_this_publish(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:1358`），规划器再把上一版全部 key 收一遍：「previous.keys_in_order().into_iter().cloned().collect();」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/code_two_tree.rs:651`），并把上一版整棵形状建成规划树：「Some(previous_root) => planning_node_of(previous, previous_root.position),」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/code_two_tree.rs:653`）。

每次发布准入之前它至少算两遍：落点那一判一遍（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12527`），式子的读数一遍：「let checkpoint_cost = checkpoint_cost_of_the_version_to_build_on_with_node_capacities(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:1719`）。

台阶行随什么涨：每次水位变了的发布留一行，「下一次发布照抄这几行（随根 COW、一直留着）」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:4238`）——每次建 inode 的发布多一行，永不回收。所以这一步的代价随「建过 inode 的发布次数」涨，那是记账树的记录总数；D8（核心索引结构） 已定项 16 射程那一句「准入之前的步代价不随盘上容器总数、记录总数涨」（`.claude/kb/decisions/08-核心索引结构.md:516`）管得到它。

同一族更小的一处：`inode_number_watermark` 逐行扫全部记账行取代段最大的那一行：「pub fn inode_number_watermark(&self) -> u64 {」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:4222`），建 inode 的发布入口在准入之前调它：「let first_new_inode_number = previous.inode_number_watermark();」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:9872`）。

四条候选各自：

| 候选 | 这一步还在不在准入之前、还随台阶行数涨不涨 | 理由 |
|---|---|---|
| A1 今天 | 在、涨 | 上面几行 |
| A2 按差量规划 | 在、涨（推的） | A2 的定义是「准入之前的规划只读这一次进出的 key」，这一步规划的是下一次空发布、不是这一次；要罩它得另写一句。差量本身很小（下一次空发布只换池级 3 行与每盘 6 行的代段，台阶行照抄），罩得住，但定义没写 |
| A3 差量上界准入 | 在、涨 | ckpt_cost 在式子的**可用**那一侧（保留池、切换预留里的 c_max），不是「这一次的分配量」，挪不到准入之后；A3 只说把这一次的全量规划挪走 |
| A4 只换表示 | 在、涨 | 规划算法不改，`BTreeSet` 与 `keys_in_order()` 原样；只换上一版的表示省不掉逐行过滤与重收 key |

改法（推的、被攻过零轮）：ckpt_cost 的记账树那一项换成「这一版的节点数 + 至多一次切分沿路径长出的节点数」（接在后面的空发布至多多 1 行，`assert!` 里已写明），代价是树高；或者记账树的节点数随一版带着按差量维护（与 `InodeRecordCounts` 同一个做法）。

**模型**（待主 agent 跑）：`every_inode_creating_publish_leaves_one_more_accounting_row_that_the_checkpoint_cost_walks` 在 4 GiB 两盘上第一个文件之后接连 40 次建 inode，每一次断言记账行恰好多 1，打 `name=accounting_rows …`。它量的是行数随次数涨，不量用时；用时随行数涨是按上面几行代码推的。

四句：不分辨臂（四条一起中——照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」的「打中不分辨臂」办：这是四条候选共用的前提里的洞，另立一笔账先修，不拿它判臂）；看得到（台阶行就在这一版里）；满足「准入之前的步代价不随……记录总数涨」那一分句；改法上面一条，与选哪条候选无关。

## 攻击面三：过度外推与越位——A3 与一条材料里没有的已定条款逐字相冲（H5）

D28（挂载期承诺量） 已定项 1 的「接线」写着判在哪、需求是什么：

- 「发布路径在 `transaction::prepare_the_version_publish` 里判：算定这次发布的形状之后、读盘核与动分配器之前。」（`.claude/kb/decisions/28-挂载期承诺量.md:35`）
- 「需求 = 这次发布新写的全部槽（普通分配加这次写出的固定点，不加这次换下的槽）」（`.claude/kb/decisions/28-挂载期承诺量.md:38`）

A3「准入之前只由差量算出这一次分配量的上界……全量规划照今天的写法挪到准入之后」与这两句都逐字相冲：判在形状算定之前，需求是上界不是「新写的全部槽」。所以 A3 不是「只改实现」，要改 D28（挂载期承诺量） 已定项 1 的条款文字；A2、A4 不碰这两句。

越位在哪：正文第一节把「准入本身的语义」归 D16（发布语义） 已定项 1，A5 格只点名 D16 已定项 1、D8 已定项 11 ①与码 2 树形状的不变量；背景材料的小节清单只有 D8、D16、E199 与欠账表四份，没有 D28（`grep -c 'D28'` 背景材料命中 29 处，全是别的条款里的转引）。D16 已定项 1「准入」那一行的式子是「可分配 = min(可再分配 + 活元数据 − 保留池, `df`)」（`.claude/kb/decisions/16-发布语义.md:44`），它不说准入判在哪一步、需求怎么数；那两件只在 D28 里。各条腿拿这份材料判「A3 碰到哪几条已定条款」，只能判出 D16，判不出 D28。`quote-kb.py --cited` 只抓正文里字面出现的编号，正文没写 D28，所以没拦住（`.claude/rules/three-way-inference.md`「正文没点名的枢纽条款，`--cited` 看不见」那一条说的就是这一形）。

四句：不分辨臂里的取舍，但分辨「A3 要不要改条款」——只 A3 中；看得到（条款文字）；满足的是 A5 格「哪一条要改条款、哪一条只改实现」；改法是给下一轮材料补 D28 的清单与附录，不是改候选。

## 攻击面二续：A2 自己的两处（H6、H7）与一条旁支（L1）

### H6　「这一次进出的 key」要在并集上算，不在各份清单上算

中央映射树规划用的 key 集合是两份清单的并：「let mapping_keys: BTreeSet<CodeTwoTreeKey> = mapped_units」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:9399`）连着「.kept_mapping_entries_after_this_publish」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:9404`）。全量规划只删「上一版有、这一版没有」的 key：「for deleted in previous_keys.difference(keys_after_this_publish) {」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/code_two_tree.rs:656`）。删一把 key 就把叶到根标成重写：「node.mark_rewritten();」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/code_two_tree.rs:539`）。

**最小形状**：建快照 → 覆盖写第一个文件的一个数据单元。被换下的那个单元被快照钉住，它的映射 key（出生身份，不随位置变：`mapping_key_for_node`、`mapping_key_for_data` 都按出生身份写）从「进映射的单元」挪进「留存的映射条目」，并集里那把 key 不变。全量规划不碰它那片叶；A2 若按两份清单各自的进出算差量（离开 `mapped_units` 记一删、进 `kept_mapping_entries_after_this_publish` 记一插），会先删后插同一把 key，那片叶与它的祖先被标成重写（删空时还会摘掉再长一片）。结果：重写的节点集合不同 → 需求不同 → 盘上节点的出生身份不同。正文 A2 格问「差量规划与全量规划重写的节点集合是不是逐个相同」，这一形答「不同，除非差量在并集上算」。
- 四句：分辨（只 A2 中）；看得到（两份清单都在这一版里，判并集要按 key 查另一份清单在不在，是一次查找、不随总数涨）；满足 A2 格「逐个相同」那一问；改法：A2 的定义写明「进出指并集的进出」。
- 能不能做成会红的测试：能（推的）：在今天的代码上建快照、覆盖写，记下中央映射树规划的 `origins`；再拿同一份上一版形状、按「两份清单各自的进出」喂 `plan_the_tree_after_this_publish` 的同一套删插，比两边的 `origins`。这一腿没写，原因是要拼 `deadlist` 那一段的内部状态，副本外没有公开入口（推的，没核全）。

### H7　节点身份是「这一版的位置」：一处切分让右边整层改名

角色带这一版的位置：「根兼叶那一档没有这个角色——记账树只有一个节点时它就是 `AccountingTree`。位置是这一版的：上一版的节点照抄进来时」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:2540`）。规划器摊平时按层给全部节点重编号：「let index_in_level = u32::try_from(nodes_by_level[level_index].len())」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/code_two_tree.rs:611`），换下的集合逐个扫上一版全部节点：「let replaced_previous_nodes = previous」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/code_two_tree.rs:685`）。映射 key 不受它影响（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/records.rs:958` 的 `mapping_key_for_node` 只写出生身份），所以这不改盘上内容；它改的是内存里按角色索引的那几份表（`mapped_units` 是 `Vec<(TransactionUnit, Vec<u8>)>`）。最左一片叶切开，这一层右边每个照抄的节点角色都变——按位置当身份的任何表示，A2 都要 O(这一层宽度) 改名。
- 它算不算准入之前：需求只要重写节点的个数，不要它们的位置；换下集合今天在 `settle` 里算、在式子之前（释放判定要它）。所以 A2 要么把身份换成出生身份（改动面大：落点、记录点名、按结构种类记的账都按角色走），要么证明准入之前只用到个数。推的，没量；与 A4（换上一版的表示）是同一个口子：A4 换表示时同样要决定身份按位置还是按出生。

### L1　今天 W 的分配记录树项与 K1 那句注释互相打架（旁支，不判）

D28（挂载期承诺量） 已定项 1 的环窗口：「W = 24 + (6 + H_退役) + Σ_ℓ min(N_ℓ, 2·(K + 26))」（`.claude/kb/decisions/28-挂载期承诺量.md:31`）——一次删除类发布分配记录树每层至多 2·(K + 26) 个节点，是按差量的上界；而 ckpt_cost 的注释说「一次发布改几片叶的不动点收不了口，拿得准的上界只有几何上整棵树」，并点名三种少扣（开新段、回落最低空槽、换下很久以前落在别处的节点）。第三种在删除批上同样成立：这次重写的叶换下的旧副本落在别的叶管的槽里，释放它要重写那片叶，那片叶的旧副本又落在第三片叶……链长按历史里各片叶上次被重写时游标在哪定，最长到 N_0。两块 4 GiB 盘 N_0 = 524 > 2·(144 + 26) = 340。我没造出这条链的历史（要让几百片叶的现行副本按一个长循环散开），只是两句话放在一起对不上。它不在这一轮五格里，交主 agent 判要不要另立一题。

## 各条候选的代价量级（全是推的，没量）

| 候选 | 准入之前的步代价 | 要动的条款 | 量级备注 |
|---|---|---|---|
| A1 今天 | O(记账行 + 映射条目 + 各树节点)，每次发布 | 无 | 基线；ckpt_cost 的记账树全量规划算两遍（H4） |
| A2 按差量规划 | O((进出 key 数) × 树高) 加 H4 那一步（定义没罩）加 H7 的改名（若身份仍按位置） | 不碰 D28；D8 已定项 11 的形状规则照旧 | 实现面大：规划器要从上一版形状按路径懒取；并集差量（H6） |
| A3 差量上界准入 | O(进出 key 数 × 树高)，H4 那一步仍在 | 改 D28（挂载期承诺量） 已定项 1 接线两句（H5） | 拒绝种类换掉（H1）；提前拒约 K1 − 真写的分配记录树节点数：两块 4 GiB 约每盘 520 槽，两块 16 TiB 约每盘 40 GiB（H3）；若用 ckpt_cost 当上界，大写上放行而真实不够（H3） |
| A4 只换表示 | 不变：`BTreeSet` 与 `keys_in_order()` 都在规划器里（`code_two_tree.rs:651`）、H4 那一步也在 | 无 | 只换表示省下的是照抄字节，省不下准入之前的全量收 key |

## 没打中的形状

| 形状 | 攻的是哪条 | 为什么没打中 | 取样范围 |
|---|---|---|---|
| 差量规划漏判「上一版形状按分隔 key 走不到它自己叶里的 key」 | A2 放行而今天拒 | 全量规划器只对要删的 key 判路由：「return Err(CodeTwoTreeRefusal::PreviousShapeRoutesAKeyAwayFromTheLeafHoldingIt);」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/code_two_tree.rs:564`），插入不判；差量规划删的是同一批 key，判出来的一样 | 读代码，`delete_at_the_root` 与 `insert_below` 两条路 |
| A3 的已承诺预留（标签 6）在全量规划之前算不出 | A3 放行而真实不够 | 式子只判非删除类、非删快照的发布；那几类发布不追加墓碑、不消失身份，标签 6 的四个量（n_live、n_记录、H_墓碑、H_退役）由差量与上一版的根头就定得出（n_live、n_记录 已按差量维护，`InodeRecordCounts`） | 读 `committed_reservation_for_tombstones_and_retirement_records_after_this_publish` 与式子判不判的那一段 |
| 位置挪动让映射 key 跟着变、差量放大 | A2 仍随总数涨 | 映射 key 是出生身份，不含位置（H7） | 读 `mapping_key_for_node`、`mapping_key_for_data` |
| A3 在 H1 那条历史之外换拒绝种类：合取 7（全空聚簇段 < N₀） | A3 拒成哪一种 | 合取 7 只读分配器上增量维护的段数，A3 可以把它留在原位；只有当实现者把它也挪到全量规划之后才中——那是实现选择，不是定义推出来的 | 读代码 |
| 用户动作放开：H1 里被拒之后用户做什么 | — | H1 的结论在被拒那一刻就成立（会话已推抬 F、F 已抬），之后用户切小还是腾空间不改它；所以没扫用户动作。H1 的前缀固定为「第一个文件之后一次大写」，单元区按 32 槽一步扫（4000 → 1100 槽） | 模型扫几何；用户动作没扫 |

以上「没打中」都只抽了这一次样，照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」不算「没打中」。

## 这条腿自己的限度

- 这一腿一条命令都没跑（照定义不交任务）：模型的四条用例、A3 模拟补丁都没编过，可能编不过；编不过时交给同立场的腿承接修，不当「模型不成立」。所有「待主 agent 跑」的数都按推的算。
- A3 模拟只是 A3 的一种写法（需求 = 普通分配的角色槽数 + ckpt_cost，已承诺预留取上一版记下的值）。H1 的分辨不依赖上界取多紧（可用低于数据单元本身）；H3 的两头依赖，取别的上界数会变。
- 崩溃状态：定义第 3b 步要每段历史的崩溃状态一个不落。这一模型没枚举崩溃状态：H1 在今天那一侧被拒于任何写之前（盘上逐字节不变，用例断言），没有崩溃状态可枚举；A3 模拟那一侧推的抬 F 空发布走的是今天已有崩溃枚举的那条路（`crash_enumeration_floor_raise_pushed_by_admission.rs`，checker 档，按定义不拷不引）；H2、H4 是数条数与行数。这一条偏离写在这里交主 agent 判；要补的话是另起一个调枚举函数的原型，这一腿没做。
- 状态数：没调枚举函数，0 个状态（量的：没跑）。
- H6、H7、L1 没有模型，全是读代码推的。
- 引文只核了冻结副本与主工作区此刻的 kb；`transaction.rs`、`mounted_session.rs` 在主工作区已改过，引文写的是冻结副本的绝对路径。

## 要主 agent 跑的命令

1. 模型整份复跑（harness 档，不是重型；两遍编译加四条用例，估 10–30 分钟，推的）：
   - 命令：`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-admission-delta-r1-opus-s2-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo --work /home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1-attack-s2/rerun-1`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`（`rerun.sh` 里两条 `cargo test -p singlefs-harness --test admission_before_full_planning_reorders_refusals` 各带范围声明，在 `--work` 下的副本 `tree/` 里跑，跑完副本删掉，输出留 `today.out`、`patched.out`、`a3_emulation.diff`）。
   - 看哪几行：`TASK_SERVER_RESULT` 的 `exit=0` 算过；`task-server/task log <号> --all` 里 `name=rerun_verdict expected=yes`、`name=rerun_summary today_rc=0 patched_rc=… failed_in_patched=1 reorder_failed_in_patched=1`；`today name=reorder_probe … outcome=refused_publish cause=journal_records_exceed_the_limit(86>85) floor_raises=0 images_unchanged=true` 与 `patched name=reorder_probe … outcome=…`（A3 模拟下的结局，H1 要它不是今天那一行）；`today name=bounded_journal_count …`（H2：`records_under_the_bound` > `record_limit`）；`today name=accounting_rows …`（H4）；`today name=fixed_point_vs_checkpoint_cost …` 十行（H3：看 `excess_slots` 的正负）。退非 0 或 `expected=no` 时整段 `today.out` / `patched.out` 交承接的腿看。
2. 跑完新派同立场的腿，派发提示写「承接：research/prompts/m4-admission-delta-r1-opus-s2-output.md」与任务号，读 `task-server/task log <号> --all` 把「待主 agent 跑」的几格改成量的。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；副本上量出的数不算入库装置上的数；没往统一任务服务交任务，判决要引的数没复跑（命令在上一节）。
- 没给 H6 写会红的测试，没给 L1 造历史，没量 A2 / A4 在大盘上的用时。
- 没下「选哪条」的结论。

## 删了什么

- 这一腿没建仓副本与编译目录（草稿目录里只放过一个拼 `rerun_steps` 用的文本片段 `rerun_steps.block`，交回前删）。
