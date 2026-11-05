# m4-admission-delta-r1 攻方腿报告（Opus，2026-11-05）

攻击面：正文第五节第一轮分工——过度外推与越位 / 内部矛盾与循环依赖 / 哪一条最脆弱。代码读冻结副本 `~/.local/share/singlefs/work/m4-admission-delta-r1/repo/`。冻结副本与主工作区的 `transaction.rs`、`mounted_session.rs`、`container_tree.rs` 不同（`cmp` 量的），这三份引文写冻结副本的绝对路径；`admission.rs`、`code_two_tree.rs`、`allocator.rs`、`allocation_record_tree.rs`、`inode_tree.rs` 两边逐字节相同，写仓内相对路径。引 kb 写 kb 文件自己的行号。

## 复跑命令与模型文件

模型目录 `research/prompts/m4-admission-delta-r1-opus-model/`。三份文件的 sha256（`sha256sum` 量的）：

```
a9650483478b734a253b37ae916fa2686087107c1e28f91af2e0bda7cde8af71  planner_bound.rs
3d6c34a260b971bb1bfd10298c30580f32c326db71f1f778adcbd03316a2e65c  k1_band.py
46fc5f69a806cd9dbe4a883178213af64d64f7c68fe07e8fbc1b16861e588c87  tools/rerun.sh
```

- `planner_bound.rs`：singlefs-core 的测试目标，三个用例，直接调冻结副本里 `singlefs_core::code_two_tree::plan_the_tree_after_this_publish`（pub）。
- `k1_band.py`：纯算术，照 `admission.rs` 与 `allocation_record_tree.rs` 的式子算各几何的 K1 与 A3 多拒那一带的下界。
- `tools/rerun.sh`：从 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps`：拷冻结副本的 singlefs-format、singlefs-core、singlefs-replay-prefix 三个包（冻结副本里没有 prune-server，整个工作区解析不了），工作区清单换成只列这三个包，把 `planner_bound.rs` 放进 `crates/singlefs-core/tests/m4_admission_delta_attack_planner.rs`，`cargo test --release -p singlefs-core --test m4_admission_delta_attack_planner`，再跑 `k1_band.py`，打出 `name=` 行，最后删掉拷出的树。改过 `rerun_steps` 之后模板自带的 `--selftest` 那几格不再适用（它造的假仓上起不了 cargo）。

复跑命令照「要主 agent 跑的命令」一节，一条都没交。报告里凡是出自这两份模型的数，都标「推的，待主 agent 跑」；只有 K1 那几个数我另用 `python3 -c` 照同一式子算过一遍（下文标「算的」）。

## 各格判定一览

| 编号 | 攻击面 | 打的是哪条候选 / 哪一格 | 结论 | 分不分辨候选 | 依据的种类 |
|---|---|---|---|---|---|
| H1 | 内部矛盾 | 正文第二节与 A1 格、候选 A1 的定义 | 打中：「准入」在代码里是三判、两个时点；A1 格的走查范围截在第一判，正文第二节点名的规划又都在第一判之后、第三判之前；候选 A1「准入判读规划的结果」只对式子那一判成立，第一判今天已是上界判 | 不分辨（共用前提） | 代码现查 |
| H2 | 循环依赖 / 最脆弱 | 候选 A3，A3 格 | 打中：分配记录树的固定点对「进出 key 数 × 树高」是循环的，能拿得准的上界只有 K1；A3 判 K1 就比今天多拒一带，宽度至少 K1 − 实际重写数（实际重写数按每盘 4 片叶连到根估：两块 4 GiB 盘约 512 块 ≈ 8 MiB／盘，两块 1 TiB 盘约 166 千块 ≈ 2.5 GiB／盘）；不判 K1、把固定点留在准入之前，又回到 H4 那一步随总数涨 | 分辨 A3 | 代码现查 + 算术（推的） |
| H3 | 过度外推 | 候选 A3 的字面式子 | 打中：根叶装满再插一把，重写 3 个节点，「1 把 × 树高 1」只给 1；式子不是上界，A3 照它判会放行而真实不够 | 分辨 A3 | 规划器代码现查；模型待跑 |
| H4 | 某一步仍随总数涨 | 候选 A2（按正文定义），连带 A4 | 打中：分配记录树的固定点每一轮整份拷分配器（逐槽一位的位图加全部分配记录），再把全部记录按叶分组比两遍；A2 的定义只管树的规划，这一步原样留在式子那一判之前 | 分辨 A2 与 A3（A3 不中，A1、A4 同中） | 代码现查 |
| H5 | 某一步仍随总数涨 / 越位 | 四条候选都中（A2 要另加一件） | 打中：ckpt_cost 里记账树那一项在第一判的参数里现算，扫全部记账行、按全部行规划下一次发布的记账树；inode 号水位的台阶行每次建 inode 的发布留一行、永不删，所以随历史无界涨 | 不分辨 | 代码现查 |
| H6 | 某一步仍随总数涨 | 候选 A4 | 打中：规划器的输入是全量 key 集合、开头整棵深拷上一版形状、交回整棵新形状，与上一版怎么表示无关；只换表示，一次只改一把 key 的发布规划器仍是 Ω(节点数) | 分辨 A4 | 代码现查；模型待跑 |
| H7 | 结论与今天不同（拒成哪一种） | 候选 A3 | 打中：今天三种「推抬 F 也没用」的拒（journal 记录超上限、合取 7、多层码 2 树拒或固定点没收口）排在式子那一判之前、盘上逐字节不变地交回；A3 把式子挪到规划之前，同一次发布先被式子拒，会话推抬 F 的空发布（F 抬了、回退候选少了），再报原来那种拒或报空间不够 | 分辨 A3 | 代码现查（推的历史） |
| H8 | 越位（口径） | 四条候选都中 | 打中一半：第一判的读数逐个走这次挂载开过的每个聚簇段，挂载里只增不减；按正文的口径（只随这一次碰到的东西涨）它不合，按代码注释引的 fs-design 口径它被注释认作合规；正文与 D8（核心索引结构） 已定项 16 射程用的是不同的总量 | 不分辨 | 代码现查 |
| H9 | 内部矛盾（命名） | 正文第三、四节 | 打中（轻）：A1–A4 同时是候选名与格名，「A1」「A3」在正文里各指两样东西 | 不分辨 | 正文现查 |

没打中的角度在「没打中的形状」一节。各条候选的代价量级在「代价量级」一节，都是推的。

## 攻击面一：内部矛盾与循环依赖

### H1 「准入」是三判、两个时点，正文与候选 A1 的定义各取了一个

代码里 `judge_the_publish_before_any_read_or_write` 依次做五件事（行号都在冻结副本）：

1. 普通分配那一半先算出来：「let ordinary_allocation_roles = plan」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12510`）。
2. 「这次的单元落得下」那一判：「admit_the_units_landing_on_every_device(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12523`），参数里现算 ckpt_cost：「checkpoint_cost_of_the_version_to_build_on_with_node_capacities(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12527`）。
3. 走固定点：「let settled = settle_the_allocation_record_tree(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12536`），每一轮「let resolved = plan.resolve(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:13063`），`resolve` 里才调「} = self.plan_the_multi_level_trees(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:8828`）。
4. journal 记录条数「refuse_a_publish_of_more_journal_records_than_the_limit(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12554`）、合取 7「admit_the_empty_cluster_segments_floor_on_every_device(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12577`）。
5. 式子那一判：「admit_on_every_device(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12621`）。

矛盾在三处：

- 正文 A1 格把走查范围写成「从 `prepare_the_version_publish` 到 `admit_the_units_landing_on_every_device` 之间走一遍」（`research/prompts/_m4-admission-delta-r1-body.md:46`），截在第 2 步；正文第二节又把多层码 2 树的规划当成准入之前的一步：「多层码 2 树（记账树、deadlist 树、中央映射树）在 `transaction.rs:8976` `plan_the_multi_level_trees` 里规划」（`research/prompts/_m4-admission-delta-r1-body.md:22`）。按代码，那一步在第 3 步里，排在第 2 步之后。
- 候选 A1 的定义是「准入之前做全量规划（全部 key、全部节点），准入判读规划的结果」（`research/prompts/_m4-admission-delta-r1-body.md:36`）。这只对第 5 步成立。第 2 步读的是普通分配那一半加 ckpt_cost，固定点那一份已经是上界：「（分配记录树那一项是 K1 整棵树）顶它」（`crates/singlefs-core/src/admission.rs:1020`）。也就是说今天的第一判本来就是 A3 那种形状，只是判在没扣保留池的槽数上。
- D28（挂载期承诺量） 已定项 1 的接线把式子那一判钉在形状定下之后：「算定这次发布的形状之后、读盘核与动分配器之前」（`.claude/kb/decisions/28-挂载期承诺量.md:35`）。D8（核心索引结构） 已定项 16 射程那一句「准入之前的步代价不随盘上容器总数、记录总数涨」（`.claude/kb/decisions/08-核心索引结构.md:516`）没说准入是哪一判。

后果：「准入」取第一判还是取式子那一判，A1 格的清单就差出整个第 3、4 步。取第一判，准入之前只剩普通分配那一半与 ckpt_cost（H5），A3 今天已经有一半；取式子那一判，第 3 步的固定点与全部规划都在里面（H4）。两种读法对应的改法不同，这一点不定下来，候选之间比的不是同一个范围。

能不能做成会红的东西：能。写一个计数，按发布路径的调用次序记下第 2、3、5 步的先后（今天的次序就够，不用实现候选）：第 3 步在第 2 步之后、第 5 步之前，与正文 A1 格的范围对不上。这是文档对代码的事实核对，不是测试，主 agent 读这几行就核得到。

### H9 候选名与格名撞名

第三节的候选是 A1–A4，第四节的格也叫 A1–A5（「A1 今天」与「A1　准入之前到底有哪几步随总数涨」，「A3 准入判挪到差量上界」与「A3　差量上界准入」）。各腿报告与判决里一写「A3」就分不清是候选还是格。本报告里候选写「候选 A3」，格写「A3 格」。建议下一轮把一族改名（例如候选改成按内容的短名），照共用约束「变体起新名字」那一条办。

### H2 候选 A3 对分配记录树是循环的：能拿得准的上界只有 K1

固定点怎么走（第 3 步）：每一轮在分配器的拷贝上释放、再给这次重写的每个角色取落点，按走出来的记录算哪几片叶变了，变了的并进重写集再走一轮——「let mut rehearsal = allocator.clone();」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:13098`）起的那一段。分配记录树重写的节点本身也要取落点，取落点又改这棵树的记录。

A3 的定义要「每棵树按「这一次进出的 key 数 × 树高」之类的式子」（`research/prompts/_m4-admission-delta-r1-body.md:38`）。对这棵树，进出的 key 就是这次变了的分配记录，里面含固定点自己写的节点。记 k = 普通分配与释放改的记录数、x = 固定点自己写的节点数、h = 树高，式子成 x ≤ (k + x) × h，h ≥ 1 时没有有限解（`k1_band.py` 的 `circular_trail` 把这一迭代打出来，封顶在 K1；推的，待主 agent 跑）。kb 已经写了这件事：「一次发布改几片叶的不动点收不了口」（`.claude/kb/decisions/28-挂载期承诺量.md:117`），同一行「拿得准的上界只有整棵树」。所以候选 A3 在这棵树上只能判 K1。

判 K1 的代价：式子那一判的可用已经扣了保留池（ckpt_cost，里面含 K1，`.claude/kb/decisions/28-挂载期承诺量.md:28` 第八项），需求再加一份 K1，等于扣两次。今天第 5 步放行的条件是「可用 ≥ 普通分配 + 实际固定点」，A3 是「可用 ≥ 普通分配 + K1 + 其余几棵树的上界」，差出的那一带宽度至少是 K1 − 今天实际重写的分配记录树节点数。

| 几何（两块盘） | K1（算的，`python3 -c` 照 `admission.rs` 的式子） | 每盘多扣（推的） | 占一块盘 |
|---|---|---|---|
| 4 GiB | 529（与 `.claude/kb/decisions/28-挂载期承诺量.md:117` 写的 529 相同） | 约 8 MiB | 约 0.20% |
| 64 GiB | 10273 | 约 160 MiB | 约 0.24% |
| 1 TiB | 166159 | 约 2.5 GiB | 约 0.25% |
| 16 TiB | 2660309 | 约 41 GiB | 约 0.25% |

大盘的单元区起点没查，四行都按 4 GiB 那一份的 50176 槽算；「今天实际重写数」按 D28 那一行引的「一块盘改 3–4 片叶」取每盘 4 片叶连到根估（4 GiB 上 17 个，1 TiB 上 25 个），都是推的。这一带随盘容量线性涨。

最小历史（推的，没跑）：两块 4 GiB 盘，只做顺序写新数据（不覆盖，defer 里没有已释放的槽），写到某块盘式子那一判的可用落进 [普通分配 + 实际固定点, 普通分配 + K1) 那一带，再写一个单元。今天第 5 步放行。候选 A3 拒（`SpaceAdmissionRefused`），会话推抬 F 的空发布，defer 里没东西可回收，推满 8 次仍拒，交回「抬 F 之后仍没空间」（`UserChangeRefused::NoSpaceAfterRaisingTheFloor`），已发成的空发布留在盘上。用户能定的只有每次写多大，带宽 8 MiB 远大于一次单单元写的需求，所以按单元写下去必然落进这一带，不靠挑剩余量。

两头都不行：A3 若不判 K1、把固定点留在准入之前算实际值，固定点要先跑 `resolve`（它就在固定点的每一轮里），准入之前就又有了 H4 那一步与全部规划，A3 定义里「全量规划照今天的写法挪到准入之后」做不到。

能不能做成会红的测试：能，做成钉今天行为的 harness 用例——窄池上把可用压进上面那一带，断言这一次写放行、盘上多出这次的单元。今天绿；候选 A3 照定义实现之后红。用例没写（我的写范围不含 `crates/`），交主 agent 定要不要派人写。

## 攻击面二：过度外推与越位

### H3 候选 A3 的字面式子不是上界：根叶装满再插一把

码 2 规划器插一把 key 时把路径上的节点标成重写，装不下就从中间切、交回新写的右半，根切开就长出新根（`crates/singlefs-core/src/code_two_tree.rs` 的 `insert_below`、`insert_at_the_root`）。最小反例：树只有一片装满的根叶（高 1），这次发布插一把 key。左半重写、右半新写、新根新写，共 3 个节点；「这一次进出的 key 数 × 树高」= 1 × 1 = 1。可用正好等于 1 个节点加普通分配时，A3 放行，固定点真要 3 个，真实不够（A3 格问的「放行而真实不够」那一种）。路径上每层都满时，一把 key 至多重写 2h + 1 个（h 是插之前的高）。

模型：`planner_bound.rs` 的 `one_key_into_a_full_root_leaf_rewrites_three_nodes_where_keys_times_height_gives_one` 在叶容量 1、2、3、8、64、255 上各插一次，断言重写 3 个；`keys_times_height_is_exceeded_over_sampled_histories` 在 5 种容量 × 5 条历史 × 400 次发布上数字面式子被超过几次、超出多少，同时数攻方推的式子「插的 key 数 × 2 × 之后的高 + 删的 key 数 × 之前的高」被超过几次（不断言，只打出）。数都是推的，待主 agent 跑。

这一格不致命：码 2 树有一个按差量算、不随总数涨的上界，今天就在代码里——`central_mapping_tree_nodes_an_empty_publish_rewrites_at_most`（`crates/singlefs-core/src/admission.rs` 里，逐层按「min(这一层的节点数, 删的 + 插的)」加切的次数加长高）。所以 H3 只说明「× 树高」那一类写法要换成带切分与长高的式子；致命的是 H2（分配记录树没有这样的式子）。

能不能做成会红的测试：能，就是模型里那一个用例的反面——给候选 A3 的上界函数写一条性质测试「上界 ≥ 规划器实际重写数」，喂根叶装满插一把那一形，字面式子红。

### H5 ckpt_cost 里的记账树那一项随历史无界涨，四条候选都没碰它

第 2 步的参数现算 ckpt_cost（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12527`），第 5 步的读数又算一次：「let checkpoint_cost = checkpoint_cost_of_the_version_to_build_on_with_node_capacities(」（`crates/singlefs-core/src/admission.rs:1719`）。其中记账树那一项「let accounting_tree_nodes = accounting_tree_nodes_after_the_publish_built_on(」（`crates/singlefs-core/src/admission.rs:1234`）：

- 扫这一版全部记账行挑代段等于这一版 txg 的（`crates/singlefs-core/src/admission.rs:1337` 起的 `.accounting_entries` 那一串），再接上全部台阶行「.chain(version_to_build_on.inode_number_watermark_steps())」（`crates/singlefs-core/src/admission.rs:1349`）收成 `BTreeSet`；
- 按这些行跑一遍规划器，取整棵树的节点数「Ok(plan) => plan.shape.nodes().len(),」（`crates/singlefs-core/src/admission.rs:1363`）。规划器本身是 Ω(整棵树)（H6）。

台阶行有多少：「水位变了的每次发布留一行」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:4235`），并且「下一次发布照抄这几行（随根 COW、一直留着）」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:4238`）。新号从水位往上发（`transaction.rs` 里 `inode_record_writes` 的注释：新建的号从记账水位起发、都大于树里每一个 key），每次建 inode 的发布水位都变。

最小历史（推的）：反复「建一个 inode、写一个单元、unlink 它、走空」N 轮。活 inode 至多一个，inode 树、extent 树都不长；台阶行 = N 行，记账树节点数随 N 线性涨。第 N + 1 次发布的第 2 步里，ckpt_cost 那一次求值要扫 N 行、规划 N 行的树——这一步在任何一判之前。

四条候选的处境：

- 候选 A2 的定义（`research/prompts/_m4-admission-delta-r1-body.md:37`）与 A2 格只说这一次发布的多层码 2 树与三棵容器树的规划；ckpt_cost 规划的是「接在这一版后面的下一次发布」的记账树，台阶行是从 `Vec` 里扫出来的。照字面不在 A2 的范围里；要另加一件（台阶行带一个最后一级的索引、下一次发布的节点数按差量推）。
- 候选 A3 只把这一次的需求换成上界，保留池不动，这一步原样。
- 候选 A4 换上一版的表示，台阶行可以建索引，但规划器照旧（H6）。

另一笔、不分辨候选的账：ckpt_cost 的记账树项取的是整棵树的节点数，不是一次空发布真重写的节点数；台阶行照抄的那几片叶空发布并不重写（规划器只重写进出 key 的路径）。保留池与暖机那一半的 c_max 因此随台阶行涨，可用随历史缩。这是今天的问题、不是候选的问题，判据自己会写错那一条里的「打中不分辨臂」，另立一笔账，不拿它判候选。D28（挂载期承诺量） 已定项 4 写的是「记账树每发布的节点数」，读成「每发布重写的」还是「每发布之后有的」，条款没说清（推的，没查变更史）。

### H8 第一判的读数逐个走这次挂载开过的聚簇段；正文与 kb 用的不是同一个总量

第一判的槽对数「let pairs_inside_the_cluster_segments: u64 = cluster_segments」（`crates/singlefs-core/src/allocator.rs:578`）对这次挂载开过的每个聚簇段查一次；那个集合「这次挂载开过的聚簇段的起点，开过就一直算数」（`crates/singlefs-core/src/allocator.rs:1872`），挂载期间只增不减，上限是单元区的段数（每段 64 槽），即随盘容量涨。代码注释认为这合规：「读它们 O(这次挂载开过的聚簇段数)，不扫单元区」（`crates/singlefs-core/src/admission.rs:977`），引的是 fs-design 那一格，`.claude/rules/fs-design.md@记账是事务的副产品，不是事后的遍历`：「不许，且代价不许随盘容量增长」。

三处口径对不上：fs-design 那一格说盘容量；D8（核心索引结构） 已定项 16 射程改读成「准入之前的步代价不随盘上容器总数、记录总数涨」（`.claude/kb/decisions/08-核心索引结构.md:516`）；正文第一节要「只随这一次碰到的容器、节点、记录数（加树高）涨」。按正文的口径，一次长挂载里的这一步不合；按 D8 的口径，它不随容器或记录总数涨，合；按 fs-design 原文，它随盘容量涨，不合。四条候选都不动这一步，这一格不分辨候选，病根是口径没定，另立一笔账。

最小历史（推的）：一次可写挂载里连发 M 次小写，每次的提交内生块把开放段写满、开新段；第 M 次发布的第一判走 ≈ M × (每次的固定点块数 ÷ 64) 个段。

另一处过度外推（轻）：正文第二节把 `inode_leaf_container_contents` 拷全部 inode 叶列成随总数涨的一处（`research/prompts/_m4-admission-delta-r1-body.md:25`）。第一版 inode 树根至多装 135 片叶容器，多了拒成类型化错误：「InodeTreeWriteRefusal::MoreLeafContainersThanOneRootNodeHolds { containers: containers.len(), capacity },」（`crates/singlefs-core/src/inode_tree.rs:1258`），每片至多 233 条，所以这一拷有常数上限（约 3.1 万条记录）。真正无界的是同一函数里整份照抄墓碑容器与退役容器：「let tombstone_containers_before: Vec<TombstoneContainer> = previous」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:8601`）、「let retirement_containers_before: Vec<RetirementContainer> = previous」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:8646`）。墓碑第一版不回收（「这一读法依赖墓碑第一版不回收」，`.claude/kb/decisions/08-核心索引结构.md:516`），每次 unlink 加一条；退役记录每消失一片叶容器加一条。这两份照抄在第一判之前，分别随删过的 inode 总数与消失过的叶容器总数涨；正文第二节没点名它们。

## 攻击面三：哪一条最脆弱

### H4 候选 A2 照定义做完，固定点那一步仍每一轮整份拷分配器

固定点每一轮：「let mut rehearsal = allocator.clone();」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:13098`）。分配器里有逐槽的位图「allocated: Vec<bool>,」（`crates/singlefs-core/src/allocator.rs:313`，每盘单元区每槽一个）与全部分配记录「records: Vec<AllocationRecord>,」（`crates/singlefs-core/src/allocator.rs:1877`）。拷完之后两次「nodes_whose_contents_changed(&geometry, previous_records, rehearsal.records());」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:13107`，同一句在 13120 行又一次），里面把前后两份全部记录按叶分组：「let before = records_of_each_leaf(geometry, previous_records);」（`crates/singlefs-core/src/allocation_record_tree.rs:372`）；再「let holding_records = nodes_holding_records(&geometry, rehearsal.records());」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:13121`）；开头还收一份上一版分配记录树的全部节点「previous_version.allocation_record_tree.node_set(),」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:13045`）。轮数上限「pub const ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT: usize = 64;」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:13141`），注释说产品路径三四轮。

候选 A2 的定义是「准入之前的规划只读这一次进出的 key 与它们碰到的路径（节点身份、分裂与合并），上一版的形状按引用读、不收全量 key」（`research/prompts/_m4-admission-delta-r1-body.md:37`），A2 格点名的是多层码 2 树与三棵容器树。固定点里这几样不是树的规划，是分配器上的预演与整份比对；照字面做完 A2，它们原样留在式子那一判之前。

量级（推的，按类型宽度算，没跑）：两块 1 TiB 盘，每盘单元区约 2^26 槽，`Vec<bool>` 一槽一字节，一次拷贝光位图就约 128 MiB，再加按段的几份计数与全部分配记录；三四轮就是一次单单元写在式子那一判之前拷约 0.4–0.5 GiB、按叶分组全部记录六到八遍。随盘容量与分配记录总数涨，两种口径（H8）下都不合。

最小历史：任何一次带普通分配的发布都走这一步，与历史无关；要显出随总数涨，只需在两个大小不同的池上各写一个单元，比这一次发布里拷了多少字节、分组了多少条记录。能做成计数测试：在副本里给 `PoolAllocator::clone` 与 `records_of_each_leaf` 各加一个只供测试的计数（照 `lookup-work-counters` 那个特性的做法），两个池上比；今天两次的比随池大小线性涨。用例没写。

对四条候选：A2（照定义）中；A4 不换分配器的表示，中；A1 中；A3 把固定点挪到准入之后，不中，但换来 H2。所以要让 A2 成立，得另加一件——固定点在分配器上记一份撤销日志与这次碰过的槽、按碰过的槽算变了的叶，不整份拷、不整份比。这是一条新候选，我叫它「A6 差量规划连分配器差量预演」：A2 加上分配器差量预演、ckpt_cost 的记账树项按差量推（H5）、墓碑与退役容器按引用追加（H8 末段）。A6 是推的，没实现、没量、被攻过零轮。

### H6 候选 A4 单独做不到：规划器的输入、开头一步与产出都是整棵树

`plan_the_tree_after_this_publish` 的签名就收全量 key 集合「keys_after_this_publish: &BTreeSet<CodeTwoTreeKey>,」（`crates/singlefs-core/src/code_two_tree.rs:639`），开头把上一版全部 key 收一遍「previous.keys_in_order().into_iter().cloned().collect();」（`crates/singlefs-core/src/code_two_tree.rs:651`），再把上一版形状整棵深拷成规划树「Some(previous_root) => planning_node_of(previous, previous_root.position),」（`crates/singlefs-core/src/code_two_tree.rs:653`；深拷是递归的「node: planning_node_of(shape, child.child),」，`crates/singlefs-core/src/code_two_tree.rs:406`，叶里的 key 逐片「keys: keys.clone(),」，`crates/singlefs-core/src/code_two_tree.rs:397`），最后整棵摊平「flatten(root, &mut nodes_by_level);」（`crates/singlefs-core/src/code_two_tree.rs:663`），换下的节点在上一版全部节点上筛「let replaced_previous_nodes = previous」（`crates/singlefs-core/src/code_two_tree.rs:685`）。这五处与上一版用乙（共享）、丙（按值交进来）、丁（只记差量）哪一种表示都无关：只要「不改规划算法」，一次只改一把 key 的发布规划器也是 Ω(整棵树)。丁更差一层：规划器要的是一份完整的形状，只记差量的上一版得先合出整棵来。

另一处：码 2 节点的角色就是它的位置「TransactionUnit::MappingTreeNodeBelowTheRoot(position)」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:2633`）。左边一切开，同层右边每个照抄的节点位置都挪一格、角色跟着换，`plan_the_multi_level_trees` 里按角色给每个节点发映射 key 那一段要走全部节点（它先建上一版全部进映射单元的索引「let mapped_units_of_the_previous_version =」，`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:9048`）。这一段在式子那一判之前。

模型：`a_one_key_publish_gets_back_a_shape_holding_every_node_of_the_tree` 在 1 千、1 万、10 万把 key（叶与内部容量都 64）的树上插一把落在最左那片叶的 key，打出交回的形状节点数、重写数、照抄而换了位置的节点数；推的预期是形状节点数等于整棵树、重写数 ≤ 2 × 高、换了位置的照抄节点随 key 数涨。数待主 agent 跑。

能不能做成会红的测试：能，就是这个用例加一句断言「交回的东西大小不随总 key 数涨」——今天红，候选 A4 照定义做完照样红，所以它判得出 A4 单独不够。

### H7 候选 A3 改了「拒成哪一种」：推抬 F 也没用的那几种拒被挤到式子那一判之后

今天的第 4 步排在第 5 步之前，会话对这几种拒不推抬 F：只有「PublishError::SpaceAdmissionRefused(_) | PublishError::PlacementRefused { .. } => true,」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/mounted_session.rs:1873`）；「PublishError::EmptyClusterSegmentsBelowTheFloor(_)」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/mounted_session.rs:1876`）、「| PublishError::JournalRecordsOfThePublishExceedTheLimit { .. }」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/mounted_session.rs:1895`）以及 `MultiLevelCodeTwoTreeRefused`、`AllocationRecordTreeRewriteSetDidNotSettle` 都在交回假的那一臂，原样交回、盘上逐字节不变。

候选 A3 把式子那一判挪到全量规划之前。journal 记录条数要等规划定下才知道（第 4 步读的是走完固定点的角色，`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12549` 那一句「let journal_records_of_the_publish = records_of_a_publish_naming_the_relocated_copies(」），于是它只能排到 A3 的式子之后。

最小历史（推的，没跑）：窄池写到式子那一判今天就不够，再发一次数据单元数超过 journal 记录上限的大写。今天交回 `JournalRecordsOfThePublishExceedTheLimit`，一个写都没发，调用方知道要把改动切小。候选 A3 先被式子拒，会话推抬 F 的空发布（D16（发布语义） 已定项 1「准入不够时抬」），F 抬了、回退候选集少了、空发布落了盘；推完之后要么式子过了、规划完再报 journal 超限，要么推满仍不够报「抬 F 之后仍没空间」。前一种拒的种类同、盘上多了几次空发布与一个更高的 F；后一种拒的种类就变了。正文要的是「准入判出的结论（放不放行、拒成哪一种）与今天相同或可证更严」（`research/prompts/_m4-admission-delta-r1-body.md:7`）。

A3 能不能把 journal 那一判也提前、按上界判：能，代价是另一种提前拒——记录条数 = 数据单元数 − 1 + ⌈其余角色 ÷ 64⌉，按上界多算的固定点（H2 的 K1 那一份）多出约 ⌈K1 ÷ 64⌉ 条：两块 4 GiB 盘约 9 条，两块 1 TiB 盘约 2597 条（推的）。今天离上限这么近的大写放行，A3 报 journal 超限。合取 7 那一种可以挪到前面（判不判只看有没有普通分配，第 1 步就知道），不算打中。

能不能做成会红的测试：能，钉今天的行为——式子与 journal 两样都不够时交回 `JournalRecordsOfThePublishExceedTheLimit`、盘上逐字节不变、F 不动。今天绿，候选 A3 照定义实现之后红。用例没写。

### 排序（只排脆弱，不选）

1. 候选 A3 最脆弱：H2 两头都不行（判 K1 就随盘容量多拒一带、与 D28（挂载期承诺量） 已定项 1「需求 = 这次发布新写的全部槽（普通分配加这次写出的固定点，不加这次换下的槽）」（`.claude/kb/decisions/28-挂载期承诺量.md:38`）和接线那一句（`.claude/kb/decisions/28-挂载期承诺量.md:35`）都冲突；不判 K1 就做不到它自己的定义），H3 字面式子错，H7 改了拒的种类。
2. 候选 A4：H6 按构造就做不到「准入之前不随总数涨」，它只能省照抄的字节（E199（发布照抄上一版的代价） 量的那一件），不改渐近。
3. 候选 A2：准入判出的结论与今天逐个相同这一点我没打穿（「没打中的形状」第 1 条），但它的定义漏了 H4 与 H5 两步，照字面做完准入之前仍随总数涨；补齐就是 A6。

推翻这个排序的现象：有人给出分配记录树固定点一个按差量算、证得出的有限上界（不是 K1），H2 就不成立，A3 退到第二；或者 A2 的差量规划在某种分裂、合并、降高、整理搬迁的组合上重写集合与全量规划不同，A2 就也打中了。

## 打中之后的四句

正文没有跑前条款与「打中 ⇒ 在几种改法之间选」的写法，第四句按「每条候选当一个改法，在打中的那几格上还中不中」答。

| 编号 | 分不分辨候选 | 被判的系统当时看不看得到判别它的东西 | 满足的是判据字面的哪一个分句 | 各候选在打中的那几格上还中不中 |
|---|---|---|---|---|
| H1 | 不分辨：四条候选共用「准入」这个词 | 看得到：三判的位置在代码里写死 | 正文 A1 格「从 … 到 `admit_the_units_landing_on_every_device` 之间走一遍」与第二节「在 … `plan_the_multi_level_trees` 里规划」 | 四条都受影响：范围定下之前比不出谁改得够 |
| H2 | 分辨 A3 | 看不到：A3 在准入那一刻没有固定点的实际值，只有几何；这正是它判 K1 的原因 | 第一节「准入判出的结论（放不放行、拒成哪一种）与今天相同或可证更严」里的「相同」（多拒不是相同；「可证更严」能不能收下交用户定） | A1、A2、A4 不中（固定点在准入之前算实际值）；A3 中；A6 不中 |
| H3 | 分辨 A3 的字面式子 | 看得到：插之前的形状在手里，满不满可以读 | A3 格「有没有一条历史让上界放行而真实分配不够」 | 换成带切分与长高的式子就不中；只影响 A3 |
| H4 | 分辨：A1、A2（照字面）、A4 中，A3 不中 | 看得到：分配器与记录都在手里 | 第一节「准入之前每一步的代价只随这一次碰到的容器、节点、记录数（加树高）涨」 | A6 不中（推的） |
| H5 | 不分辨：A1–A4 都中 | 看得到 | 同 H4 那一句；另一笔（保留池随台阶行涨）是今天的问题，另立账 | A6 不中（推的，要另加台阶行索引） |
| H6 | 分辨 A4 | 看得到 | 正文 A4 格「不改规划算法、只换表示，准入之前能不能不随总数涨」 | 只有 A4 中；A2、A6 换了算法，不中 |
| H7 | 分辨 A3 | 看不到：journal 记录条数在 A3 的准入那一刻只有上界 | 第一节同一句里的「拒成哪一种」 | A1、A2、A4、A6 不中；A3 中，改成上界判 journal 换成另一种提前拒 |
| H8 | 不分辨 | 看得到 | 第一节那一句与 D8（核心索引结构） 已定项 16 射程那一句用的总量不同 | 四条都中；是口径的账 |
| H9 | 不分辨 | — | — | 只是命名 |

## 改法各修哪一格

都是攻方提的，只在推理上、没实现没跑，被攻过零轮；表里「推的」的格都待主 agent 派人实现、跑过才算数。

| 改法 | H2 | H3 | H4 | H5 | H6 | H7 | H8 |
|---|---|---|---|---|---|---|---|
| 候选 A3 的上界换成带切分与长高的式子（照 `central_mapping_tree_nodes_an_empty_publish_rewrites_at_most`） | 推的：不修（分配记录树仍只能 K1） | 推的：修 | 推的：不碰 | 推的：不碰 | — | 推的：不修 | 推的：不碰 |
| A6 差量规划连分配器差量预演（撤销日志加碰过的槽） | 推的：绕开（不走上界） | 推的：绕开 | 推的：修 | 推的：要另加台阶行索引才修 | 推的：修（换了规划算法） | 推的：不中（判的次序不动） | 推的：不碰 |
| 台阶行带一份「最后一级」的索引，ckpt_cost 的记账树项按差量推 | — | — | — | 推的：修求值的代价；保留池随台阶行涨那一笔不碰 | — | — | — |
| 第一判的槽对数改成按段增量维护（去掉逐段求和） | — | — | — | — | — | — | 推的：修 |

## 代价量级（全是推的）

| 候选 | 准入之前每次发布的代价 | 别的代价 |
|---|---|---|
| A1 今天 | 规划 Ω(中央映射条目 + 记账行 + 墓碑与退役容器) × 固定点轮数 3–4；固定点每轮整份拷分配器 Ω(单元区槽数 + 分配记录数) | 无 |
| A2（照正文定义） | 树的规划降到 O(这次进出的 key × 树高)；固定点每轮的分配器整拷与整份比对、ckpt_cost 的记账树项、墓碑与退役容器照抄不变 | 改规划器、改 `node_origins` 等 Verus 规格（正推腿的格） |
| A3 | 准入之前 O(盘数 × 根层级) 加普通分配那一半；ckpt_cost 的记账树项若不另改仍 Ω(台阶行) | 多拒一带：每盘至少 K1 − 实际重写数，约 0.20–0.25% 盘容量（两块 4 GiB 约 8 MiB／盘，两块 1 TiB 约 2.5 GiB／盘）；拒的种类与次序变（H7）；D28（挂载期承诺量） 已定项 1 需求那一行与接线那一行要改条款 |
| A4 | 渐近不变；省照抄字节的常数（E199（发布照抄上一版的代价） 量的那一件） | 改表示，Verus 规格里交回整份 `Vec` 的几处要跟着改 |
| A6（攻方提，零轮） | O(这次进出的 key × 树高 × 固定点轮数) 加 O(这次碰过的槽) | 分配器加撤销日志与碰过的槽；台阶行索引；规格改得最多 |

## 要主 agent 跑的命令

一条，轻型（singlefs-core 不是 checker 档包；不调崩溃枚举，不涉崩溃状态）。我一条都没交。

1. 模型整份复跑（第 9 步）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1-attack -- bash /home/fy5090/code/singlefs/research/prompts/m4-admission-delta-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo --work /home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1-attack/rerun-1`
   - 在哪起：仓根起 submit，`--cwd` 是本腿草稿目录。脚本里 `cargo test` 那一行自带 `SINGLEFS_RUN_SCOPE='种类=target; …; 范围=1 个测试目标; …'`。
   - 看哪几行：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=0`；日志里 `name=planner_test exit=0`、`name=k1_band exit=0`；`test result: ok. 3 passed`；`name=full_root_leaf_one_insert` 六行每行 `rewritten=3 literal_bound=1`；`name=keys_times_height_sweep` 一行的 `literal_bound_exceeded` 大于 0、`corrected_bound_exceeded` 是几（我推的是 0，不是 0 就说明攻方推的那个式子也错，照打出的 `name=corrected_bound_violation` 行改）；`name=one_key_publish_output_size` 三行的 `nodes_in_the_returned_shape` 与 `nodes_before` 相同、随 `total_keys` 涨，`rewritten` ≤ 2 × `height`；`name=k1_band` 四行的 `k1` 依次是 529、10273、166159、2660309（与我用 `python3 -c` 算的对得上才算）。
   - 跑完新派同立场的腿（带「承接：research/prompts/m4-admission-delta-r1-opus-output.md」与任务号），把 H3、H6 两格的「推的，待主 agent 跑」换成读到的数。

没写进命令、交主 agent 定要不要派人写的三条钉今天行为的 harness 用例（都是推的形状，见 H2、H4、H7 各节末）：窄池上可用落进 K1 那一带时单单元写放行；两个大小不同的池上固定点拷贝字节与分组记录数之比；式子与 journal 两样都不够时交回 journal 超限、盘上不变、F 不动。

## 没打中的形状

1. 候选 A2 的差量规划与全量规划重写集合是不是逐个相同。码 2 规划器先按 key 升序删「上一版有、这一版没有」的，再按 key 升序插「这一版有、上一版没有」的（`crates/singlefs-core/src/code_two_tree.rs` 里 `plan_the_tree_after_this_publish` 那两段 `difference` 循环），重写集合只由这两个集合定；差量规划拿到的两个集合与它相同，结果就相同。按这一条走过：叶切开、根切开长高、删空摘节点、根只剩一个孩子降高（孩子照抄不重写）、整理搬迁按 key 重写路径（`plan_rewriting_the_paths_to_the_moved_entries` 只需知道搬走的 key 这次删没删）、deadlist 删到零根。没找到不同的一形。只读了代码，没写模型；单次抽样，按规则不算「没打中」。另记一个实现陷阱（不是候选的问题）：记账树里这一版 txg 那一行若是台阶行要照抄、不是就删，差量实现若把上一版代段的行一律当删，形状就和今天不同——可以做成一条会红的单测。三棵容器树（extent、墓碑、谱系与退役）的差量规划没走查。
2. 候选 A3 与 C283（准入失败时不先推发布就报 ENOSPC） 合不合：A3 照样先推抬 F 再报，C283 本身不中；打中的是 D28（挂载期承诺量） 已定项 1 的需求与接线两行（H2）。
3. 删除类发布：式子那一判今天就不判（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12603` 那一句 `let demand_is_judged`），第一判读普通分配加 ckpt_cost，A3 不改它们的结论。
4. 合取 7 在 A3 下的次序：判不判只看有没有普通分配，第 1 步就知道，挪得到式子之前，不算打中（H7 末段）。
5. 候选 A4 改不改准入判出的结论：换表示不改值，结论与今天相同；它打中的只是代价（H6）。
6. inode 记录计数（`InodeRecordCounts`）：已经随一版带着、按差量维护，准入读它不遍历（正文第二节最后第二条），没打中。
7. 由用户决定的动作：H2、H7 的历史里用户只定每次写多大与写不写大写，我按「写多大随便取」推的；H4、H5、H6、H8 与用户动作无关（每次带普通分配的发布都走那一步）。我没有装置，没把用户动作写死在任何一个模型里。

## 这条腿自己的限度

- 一个命令都没跑：两份模型都没编、没跑，H3、H6 的数全是推的；只有 K1 那四个数用 `python3 -c` 照式子算过。
- 最小历史都是按代码推的，没有一条在 harness 上真走过；窄池能不能把可用正好压进 H2 那一带、journal 上限在产品几何上是多少，都没量。
- 大盘的单元区起点按 4 GiB 那一份取，K1 在大盘上可能偏几十个节点。
- 没读 `container_tree.rs` 的 `node_origins` 与 extent 树规划的内部，A2 的「重写集合相同」只在码 2 规划器上走查过。
- 没查 D28（挂载期承诺量） 变更史里「记账树每发布的节点数」当时怎么定的（H5 另一笔）。
- 「没打中」的几条都是单次抽样。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；不下「选哪条」的结论。
- 没往统一任务服务交任务；判决要引的数没复跑（命令在「要主 agent 跑的命令」一节）。
- 副本上量出的数不算入库装置上的数——这一腿连副本也没建，草稿目录里只有认领文件，没有要删的仓副本与编译目录。
- 没写 harness 用例（写范围不含 `crates/`），三条建议用例交主 agent 定。
