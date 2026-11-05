# m4-close-code-r2 攻方腿（Opus，第一次抽样，2026-11-03）

代码读冻结副本 `~/.local/share/singlefs/work/m4-close-code-r2/repo/`，下文 `crates/…:行` 都是它的行号。被攻过零轮的改法都标「只在我的模型上量过、被攻过零轮」或「推的」。这一腿没有往统一任务服务交任务；探针在副本上没编、没跑（命令在「要主 agent 跑的命令」一节），下面凡是探针的结局都是「推的，待主 agent 跑」。

## 各格判定一览

| 格 | 判定 | 打中的形状 / 没打中的理由 | 量的还是推的 |
|---|---|---|---|
| T1 开片原地改、上一版只记节点数 | 没打中 | 开片的 `ensures` 仍是整份函数式规格，规格本身这一批没改；「上一版不在的节点」改按节点数判，与改前按位置判同义 | 读码 |
| T2 求祖先从最右端往左 | 没打中 | 三棵容器树只经 `append_records_to_the_open_container` 与 `carried` 造，两处都保证最右连段；运行期另有逐片断言 | 读码 |
| T3 孩子区间累加、按下标取根单元 | 没打中 | `units` 装好之后只改槽号不改次序；三处装 `units` 都在装好之后记下标；断言 panic 合编码纪律「破坏不变量的就断言」 | 读码 |
| T4 准入不遍历 inode 树 | **打中**（新破绽，坏镜像才有） | 两片 inode 叶交叠时，新加的按叶序折半找与写者、删者逐片找交回不同：维护的 n_live / n_记录 漂离现数，最后一批 `expect` 落空 panic；恢复收下这种镜像 | 读码推的；探针待主 agent 跑 |
| T5 G7 整理门 | 没打中 | core 只在排空待发之后起整理，待发至多一个文件、一次发完 | 读码 |
| T5 Y5 根环成形两条 | **打中**（方法：桥没接上；语义差推的） | 补的两条只有同一批写的见证在用，必红世界两行只能证见证会红；协议的 F 随根环挤出抬、core 的 F 照抄，「持久 F 之上的根装得进根环」在 core 上要么不成立、要么空真，看 `roots` 怎么取，模型没写 | 推的 |
| 方法：对拍取样点 | 弱点，不算打中 | `node_origins` 那组对拍上一版与这一版取同一份形状，「上一版没有这个位置」那条断言在对拍里一次没走到；Verus 证明罩着，所以不算破绽 | 读码 |
| 从未看过的：准入之前列全部容器与节点 | 归属空着（不开格） | `mapped_units_of_a_container_tree` 在 `resolve` 里每轮走全部容器与全部节点，E199 跑前登记把「列全部容器、列全部节点那几遍」记成射程外、不量 | 读码 |

## 复跑命令与模型文件

模型目录 `research/prompts/m4-close-code-r2-opus-model/`：

| 文件 | sha256 |
|---|---|
| `probe/inode_record_lookup_over_overlapping_leaves_probe.rs` | `deae465286a880f967da93465140dea204a8849d534ef96248ef63bdeb88dc87` |
| `tools/rerun.sh` | `0c95b21ca0a522134330c721ea6a37de5f8e33d6797a0dfd2b64f5527b0d186a` |

`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只改了 `rerun_steps`：拷冻结副本进 `--work`、把探针接到 `crates/singlefs-core/src/mount.rs` 末尾、跑 `cargo test -p singlefs-core --lib inode_record_lookup_by_leaf_order_over_overlapping_leaves_probe -- --show-output`，末行打 `name=rerun_attack_m4_close_code_r2 cargo_exit=… control_ok=… overlap_failed=… probe_lines=…`。对照组一条过、交叠组三条都红才退 0（打中照原样复现）；编不过、对照组红、交叠组有绿的都退 1。复跑命令（仓根起；`--work` 给新目录）：`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-close-code-r2-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-close-code-r2/repo --work /home/fy5090/.local/share/singlefs/work/m4-close-code-r2-attack-rerun`；看哪几行在「要主 agent 跑的命令」一节。

## T4：准入不遍历 inode 树——打中

### 机制

- 新加的找法：「叶容器左起按 key 序、各片的记录按号升序、两片之间不交叠」（`crates/singlefs-core/src/transaction.rs:4184`）是它写明的前提，「与 [`TransactionOutput::inode_record_of`] 交回同一条」（`crates/singlefs-core/src/transaction.rs:4185`）是它写明的契约。做法是「let holding = self.inode_leaf_containers.partition_point(|container| {」（`crates/singlefs-core/src/transaction.rs:4189`）按每片最后一条找一片，只在那一片里折半。
- 发布路径用它改两个数：「previous.and_then(|version| version.inode_record_found_by_its_leaf(inode))」（`crates/singlefs-core/src/transaction.rs:8390`），写入类走「inode_record_counts_before.after_writing(&record_writes, record_before_this_publish)」（`crates/singlefs-core/src/transaction.rs:8405`），找不到就当新号：「counts.records += 1;」（`crates/singlefs-core/src/admission.rs:346`）。最后一批走「.expect("最后一批删的那条记录在上一版的 inode 树里：不在的那一种 remove_record_from_leaf_containers 已经拒了"),」（`crates/singlefs-core/src/transaction.rs:8395`）。
- 同一次发布里树本身是写者、删者算的，它们逐片找：「let existing = position_of_the_container_holding(&containers, inode);」（`crates/singlefs-core/src/inode_tree.rs:1145`）、「let Some(index) = position_of_the_container_holding(containers_before_this_publish, inode)」（`crates/singlefs-core/src/inode_tree.rs:1721`），逐片里各折半一次：「match search_records_by_inode(&containers[position].records, inode.0) {」（`crates/singlefs-core/src/inode_tree.rs:855`）。写入之前判「这个号在不在树里」的入口也是逐条找的 `inode_record_of`：「let Some(record) = previous.inode_record_of(inode) else {」（`crates/singlefs-core/src/transaction.rs:11677`，部分写那一支）。
- 写者与删者要的「叶间严格升序」只写在 Verus 的 `requires` 里：「leaf_containers_are_well_formed(containers_before_this_publish@),」（`crates/singlefs-core/src/inode_tree.rs:1033`），产品代码从普通 Rust 调它时不判。恢复读 inode 叶只判下界：「if record.inode < separator_key || record.inode < identity.container {」（`crates/singlefs-core/src/recovery.rs:6641`），外加根的 key 区间按全体记录的最小、最大号判，左片最大号小于右片分隔 key 这一条没判。池级 checker 判它：「.judge("I-9.12", separator > previous_largest, || {」（`crates/singlefs-checker/src/walk.rs:4689`）。可写挂载不跑池级 checker（`crates/singlefs-core/src/mount.rs` 里没有对 `singlefs_checker` 的引用，grep 0 处）。

### 最小反例（推的，探针待主 agent 跑）

镜像：inode 树两片叶。左片分隔 key 1、容器号 1、记录 inode 1 与 inode 9；右片分隔 key 5、容器号 5、记录 inode 5。每条记录都不小于自己那片的分隔 key 与容器号，根的 key 区间 [1, 9] 与全体记录的最小、最大号相同；水位取 10。恢复照上面那两判收下（推的：只读了 inode 叶这一段的判法，没造镜像去挂）。

| 步 | 走到 | 期望（改前那一版也是这样） | 实际（冻结副本） |
|---|---|---|---|
| 挂上之后部分写 inode 5（或整份写、改 size） | `inode_record_of` 逐条找到 inode 5，放行；写者逐片在右片找到它、原地换掉 | 树里仍 3 条、3 个活的；改前在 `resolve` 里对这次之后的叶逐片现数（改动全文删掉的那一行是 `live_objects: live_objects_of_the_inode_tree(&resolved.inode_tree.containers),`），交叠叶上照样是 3 / 3 | 按叶序找：左片最后一条 9 不小于 5、右片最后一条 5 不小于 5，`partition_point` 交 0，在左片 [1, 9] 里折半找不到 5，交 `None`；`after_writing` 当成新号，n_记录 与 n_live 各记成 4。记账标签 6 按维护值写，比闭式多 副本数 × (41 + 35) 字节，池级 checker 的 I-3.20（b）在新根上判红（推的） |
| unlink inode 5：意图那一次 | 同上，记录写成 nlink = 0 | n_live 减 1 | 当成新号：n_记录 加 1、n_live 不减 |
| 最后一批 | 删者逐片在右片找到 inode 5，删掉 | 两个数各减 | `record_before_this_publish(inode)` 交 `None`，第 8395 行 `expect` panic；改前这一支不找上一版记录，不 panic |

同一片里记录乱序（例：一片装 5、3）两边用的是同一个片内折半，交回相同，不在这一格里；只有片与片交叠时两种找法分开。

### 打中之后四句

1. 分不分辨：分辨改前改后。改前每次发布对这次之后的叶现数，交叠镜像上两个数仍等于现数、最后一批不找上一版记录；改后漂、panic。「恢复该不该收交叠镜像」是改前就有的缺口，改前改后一样，不分辨。
2. 被判的代码那一刻看不看得到：看得到。两片叶都在内存里，写者逐片就找到了；新找法是自己只看了一片。
3. 满足的是哪一句：「与 [`TransactionOutput::inode_record_of`] 交回同一条」（`crates/singlefs-core/src/transaction.rs:4185`）那句在恢复收下的状态上不成立；第 4184 行写的前提「两片之间不交叠」在读盘那一侧没有人保证。另碰到 C841 那条用户定案的口径（坏镜像在写路径上报类型化错误、不 panic，`container_tree.rs` 里 `NextContainerNumberPastTheTopOfItsRange` 的文档注释原话「用户 2026-10-15 定」）：最后一批这一处是新加的 panic。
4. 改法在打中的格上中不中：见下表。

### 改法与它们各修哪一格（都只在我的读码上推过、被攻过零轮）

格：① 两种找法交回相同；② 维护的两个数等于现数；③ 最后一批不 panic；④ 写者、删者的 `requires` 在运行期有来源。

| 改法 | ① | ② | ③ | ④ | 代价 |
|---|---|---|---|---|---|
| 甲：恢复读 inode 叶时照池级 checker 的 I-9.12 判叶间严格升序（右片分隔 key 大于左片最大号）、片内严格升序，不合拒挂（类型化错误） | 推的，中 | 推的，中 | 推的，中 | 推的，中 | 挂载时每片一次比较，读盘本来就逐片读；准入之前不加步 |
| 乙：`inode_record_found_by_its_leaf` 改成与写者同一种逐片找 | 推的，中 | 推的，中 | 推的，中 | 不中（写者照旧靠 `requires`） | inode 树至多 135 片叶（`write_records_into_leaf_containers` 的 `ensures` 写着 `after.containers.len() <= 135`），每写一条至多 135 次片内折半，不随记录总数涨；推的，没量 |
| 丙：最后一批的 `expect` 换成类型化错误 | 不中 | 不中 | 推的，中 | 不中 | 无 |

甲同时补上 r1 判决 E 簇同一形的缺口（读者收下、写者假设），我倾向甲；乙可以与甲并用当保险。

### 能不能做成会红的测试

能。探针（模型目录 `probe/` 那一份，接在 `mount.rs` 末尾的测试模块）不造镜像，拿第一个文件之后那一版、把 inode 叶换成上面那两片，直接比：

- `overlapping_leaves_agree_on_the_lookup`：按叶序找与 `inode_record_of` 对 inode 5 交回相同（推的：红，前者 `None`、后者 `Some`）；
- `overlapping_leaves_keep_the_maintained_counts_equal_to_the_recount`：写者改一条 inode 5 之后，`after_writing` 的维护值等于对写完的叶逐片现数（推的：红，维护值 4 / 4、现数 3 / 3）；
- `overlapping_leaves_find_the_record_the_final_batch_removes`：删者删得掉 inode 5，按叶序也找得到它（推的：红，删者 `Ok`、找法 `None`）；
- 对照组 `leaves_in_strict_key_order_agree_on_every_lookup_count_and_removal`：两片 [1, 3] 与 [5, 9]（偏移写法 [0, 2]、[4, 8]），四个号逐个比三样（推的：绿）。

整条链（造一份交叠的镜像、可写挂载收下、部分写之后池级 checker 的 I-3.20 红、unlink 到最后一批 panic）也做得成会红的 harness 用例，要造带校验和的盘，这一腿没写。改成甲之后，「恢复拒交叠镜像」那一条就是它的必红测试；变异行可以钉恢复里新加的那一判（去掉就该红）。

### 什么现象会推翻这一格

- 恢复里另有一处判了叶间升序（我只读了 `inode_tree_read_and_judged` 与 `inode_leaf_container_judged_against_its_entry` 两处，没读 `rebuild_version` 之后的每一判）：主 agent 跑探针之外再看一眼，或造镜像挂一次；挂不上就是推翻了「恢复收下」，①②③ 退成只在内存里造得出的形状。
- 探针的交叠三条有绿的：说明我对 `partition_point` 交回值的手推错了。

## T5：整理门与根环成形

### G7 整理门——没打中

协议这一门：「RunRingStep::CompactionCheckpoint => state.roots.len() >= 1」（`crates/singlefs-replay-prefix/src/gc_ring_interval_proofs.rs:250`）再要现行逻辑内容等于最后一个根。core 一侧找能发出「现行领先最后一个根时的整理发布」的历史，试了这几样：

- 攒批会话起整理之前先排空：「let pending_changes_published_before_the_run = match self.publish_the_pending_changes(」（`crates/singlefs-core/src/mounted_session.rs:5597`），排空被拒就不跑。排空是一次发完：「let pending = std::mem::replace(&mut self.pending, PendingDataChanges::empty());」（`crates/singlefs-core/src/mounted_session.rs:5231`），待发至多一个文件（`accept` 的文档注释第 3 条），没有「排了一半」。
- 不攒批的 `MountedSession::run_compaction_in_the_background` 也是 `pub`：它那一侧每次用户改动当场发布，没有待发，现行就是最后一个根。
- 冻结着没重发的发布：整理的发布与别的发布一样先过 `refuse_while_a_publish_is_frozen`。
- 管理员回退：core 的回退是一次向前发布（写根），在协议上是 Rollback 紧接 UserCheckpoint，中间那一刻 core 不交出来。
- 挂载续做整理意图：挂载时现行就是最后一个根。

都没造出 core 发了协议这一门不许的整理发布。core 的整理发布接在 `previous` 后面只搬单元，就算待发没排空，它落的根也是上一个根的逻辑内容；这时协议没有对应的一步（门拦着），但用户看得见的那一层不变——这是读码推的，没证。

### Y5 根环成形两条——打中（方法），语义差推的

1. **桥没接上。** 补的两条是「&&& ring.in_memory_floor.0 <= ring.roots.last().checkpoint_txg.0」（`crates/singlefs-core/src/verified_model.rs:218`）与「&&& forall|index: int| #[trigger] root_is_protected(ring, index) ==> ring.roots.len() - ring.window <= index」（`crates/singlefs-core/src/verified_model.rs:219`）。这份文件自己写着「这份文件是规格底座：只放 spec 与钉住它们取值的见证（proof fn），不接协议核，也不被产品代码调用；」（`crates/singlefs-core/src/verified_model.rs:5`）。`root_ring_is_shaped(` 在 core 与 replay-prefix 的 `src/` 里除定义与那份见证之外 0 处命中（grep 过）。必红世界两行（`crates/singlefs-core/verus-must-red-worlds.tsv:453`、`crates/singlefs-core/verus-must-red-worlds.tsv:454`）落点都是同一批新写的 `witness_the_root_ring_shape_keeps_a_root_above_the_floor_and_fits_the_window`：去掉哪一条，红的只是那份为钉这两条写的见证，红不出「core 的根环满足它」。r1 判决 D 簇写的是「在 core 一侧核，核实缺着就补不变式与测试」；这一批补了规格与见证，core 一侧的运行期判与测试没有，core 的值从来没被装进 `RootRingModel` 比过。
2. **两边的 F 不是同一个量（推的）。** 协议里用户发布落根时 F 跟着挤出抬：「| UserCheckpoint | 无 | 现行落成新根；根数超过容量时最旧的候选被挤出（内存 F 加一），持久 F 跟上新的内存 F；新 F 之上的旧根原本就受保护 |」（`crates/singlefs-replay-prefix/src/README.md:66`）。core 的 F 照抄上一版：「// F 照抄建在上面的那一版（D16（发布语义） 已定项 1：F 只升不降）。」（`crates/singlefs-core/src/transaction.rs:9729`），生效的 F 取各根带的 F 与系统配置里的 F 的最大（`crates/singlefs-core/src/recovery.rs:2462` 起），里面没有「根环挤出旧根就抬」那一项；成形条件又钉着「&&& ring.persisted_floor == ring.roots.last().rollback_floor」（`crates/singlefs-core/src/verified_model.rs:216`）。于是：
   - `roots` 若取「F 之上的全部根」（被根环盖掉的也算，core 的回收门槛 `generation <= floor` 照样护着它们罩的槽）：不抬 F 连发 window + 1 次之后受保护的根多于 window，第 219 行那一条在 core 的值上不成立；
   - `roots` 若取「此刻根环里读得出的根」：根数本来就不超过根环容量，第 219 行空真，换算没有说出协议那一条的意思。
   模型对 `roots` 只写了「// - roots 旧到新，最后一条是崩溃后重现的那一条；」（`crates/singlefs-core/src/verified_model.rs:180`），两种读法它都允许。这一格打中的是「桥的两条在 core 的值上成立吗」这个问题在代码里没有答案，不是 core 行为错：core 多护着一些槽是保守的一边。第 218 行那一条在 core 上我推是成立的（core 没有脱离根的内存 F，空间被拒那一次抬 F 是带新 F 的发布），同样没有判法罩着。

四句：分辨改前改后（改前两条不在，改后在、但没人读它们）；系统看得到（core 一侧什么都没读）；满足的是 r1 判决 D 簇「核实缺着就补不变式与测试」里「核」的那半句没做；改法——给 `RootRingModel` 写一个从 core 的根环读数（根环里各槽的根、生效 F、根环容量）装出来的抽象函数，配一条 harness 用例在发布序列上逐次判 `root_ring_is_shaped`（推的，被攻过零轮），先定 `roots` 取哪一种读法。

## T1：开片原地改与上一版只记每层节点数——没打中

- 形状逐个相同：`after_a_container_is_opened_at_the_right_end` 的 `ensures` 是「opened.levels() == levels_after_opening_at_the_right_end(self.levels(), entries_per_node as int),」（`crates/singlefs-core/src/container_tree.rs:2374`），规格 `levels_after_opening_at_the_right_end` 在改动全文里只出现在上下文行、没有改。改前那一份照抄再改的实现证的是同一个 `ensures`，所以长高、正好填满一层、连开几片跨层这几处，改前改后交回的形状相同，由证明保证，不靠取样。对拍取样的弱点记在「方法」一节。
- 只记节点数之后读不到的东西：上一版形状在这次发布里只被问四样（位置上有没有节点、是不是根、根在哪、根的层级），`ContainerTreeNodeCounts` 四样都答得出。「照抄的节点一个字节不动」靠的是 `appending_progress` 里的 `levels_extend`（最右节点之前的孩子数逐个相同）与「上一版不在的节点都罩着重写的某一片」：唯一改了孩子数的已有节点是停的那一层最右那个，它是新开那片的祖先，跟着重写。规格那一句只把 `node_is_in_the_levels(previous_levels, …)` 换成 `node_is_counted(node_counts_of(…), …)`，两者都是「层下标在范围内、同层序号小于那一层节点数」，同义。
- 试过的形状：扇出 2 时一次追加让旧根先装满、再连长两层（上一版一层一个节点、追加 3 片）；长高那次旧根下面又有容器重写；`carried` 一条不追加。三处 `node_origins` 的标法与 `previous_root_is_carried_down_unchanged` 都对得上手推。
- 没再看的：一字节层级装满 255 层那一格（`requires` 排除，健康树走不到）。

## T2：求祖先从最右端往左——没打中

- 前提「重写的是最右连段」在 `crates/` 今天的每条发布路径上成立：`ContainerTreeAfterThisPublish` 的字段虽是 `pub`，`crates/singlefs-core/src/` 里构造它的只有 `append_records_to_the_open_container` 与 `carried`（grep `rewritten_containers:` 与结构体字面量，测试之外 0 处别的）；墓碑树、谱系树、退役树三棵都只经这两处。逐条对：追加、开片照「只往最右那片里装、装满了在最右端开新的」；unlink 合并叶改的是 inode 树，往退役树里只追加；截断、整理搬迁、建删快照对三棵容器树走 `carried`；管理员回退那次三棵树随现行那一版（`mount.rs` 的 `version_to_publish_the_rollback_after` 里 `tombstone_containers: current.tombstone_containers.clone(),` 等），只往谱系树追加一条；回退之后第一次发布接着现行那一版追加。
- 前提万一不成立时：运行期有逐片断言「self.rewritten_containers[listed] == total - rightmost_count + listed,」（`crates/singlefs-core/src/container_tree.rs:2786`），走到就 panic，不会静默漏掉或多算。两处 `position >= first`（`transaction.rs` 列映射单元那一处与装容器单元那一处「let first_rewritten_container = after.first_rewritten_container();」（`crates/singlefs-core/src/transaction.rs:14110`））都先调 `first_rewritten_container`，同一道断言罩着。

## T3：孩子区间累加与按位置取指针——没打中

- `units` 的次序：装 `units` 的三处都是装好再记下标——「let container_tree_root_unit_positions = ContainerTreeRootUnitPositions::of_the_units(&units);」（`crates/singlefs-core/src/transaction.rs:16584`、`crates/singlefs-core/src/recovery.rs:3394`、`crates/singlefs-core/src/mount.rs:10178`）。之后改 `units` 的只有整理那一批与恢复之后盖映射位置，改的是槽号：「version.units[*position].slot = slot;」（`crates/singlefs-core/src/transaction.rs:3925`）。搬迁那一批改的是上一版的一份拷贝，下标照搬；清扫、截断、挂载续做都经发布重装 `units`。在 core 与 harness 的 `src/` 里 grep `.units.` 之后接 `push`、`retain`、`sort`、`insert`、`remove`、`iter_mut` 与 `.units =` 整份赋值：命中的只有 `region_scan.rs` 里另一个结构的 `units` 表（不是 `TransactionOutput` 的）与 checker 档实验装置、unit-graph 里的同名字段，`TransactionOutput::units` 上 0 处。
- 长高那次旧根在 `units` 里换成根之下的角色（`container_tree_node_below_the_root_unit` 按这一版的角色装），一版里不会有两个 `TombstoneRoot`，`of_the_units` 取「最先那一项」不会取到旧根。
- 根之下节点的指针表：发布按 `node_origins` 的 bump 次序装；恢复从根往下按层读、`reverse` 之后逐层逐个编号，也是 bump 次序。`pointer_below_the_root` 的下标式在这两种来路上都对。
- panic 还是返错：记下的下标对不上用 `assert_eq!` panic，合 `.claude/singlefs-ai-sop/rules/code-discipline.md@错误：能恢复的写进类型，破坏不变量的就断言` 那一节「不变量被破坏是 bug，用断言」；下标只由这个进程装 `units` 时记，不来自盘上字节，不属于坏镜像那一类。

## 方法：对拍取样点、钉代价的测试、必红世界行

- **对拍取样点**：`node_origins` 那组对拍写着「别的都是照抄（上一版取同一份形状，每个位置上一版都有）。」（`crates/singlefs-core/src/container_tree.rs:4068`），装的是「node_counts_before_this_publish: shape.node_counts(),」（`crates/singlefs-core/src/container_tree.rs:4087`）。上一版与这一版节点数相同，`node_origins` 里「照抄的位置上一版有」那条运行期断言在 3386 组里一次也没处在「上一版没有」的一边；上一版节点数真比这一版少的取样只有 `the_version_before_this_publish_is_kept_as_the_node_count_of_each_level` 一个形状。形状本身全由被测函数开出来：「shapes.push(shape_after_opening(containers, entries));」（`crates/singlefs-core/src/container_tree.rs:4031`），只有换扇出那 31 份不是左边都满的样子。这一格有 Verus 罩着，对拍是补充，所以记弱点、不记打中；要补就从盘上读法会交出的不规整形状（左边节点不满）取样，并让上一版取「开片之前」的那一份。
- **钉代价的测试**：`opening_a_container_changes_the_levels_in_place_without_copying_them` 比的是每层 `Vec` 的地址不变，钉的是「不逐层照抄」；`ancestors_of_the_rightmost_containers_read_only_the_nodes_on_their_paths` 拿最左节点写 `usize::MAX` 的形状证明求祖先读不到它们。两条都只钉 T1、T2 那两个函数自己；准入之前别的步（下一节）没有一条钉。
- **必红世界行**：T5 Y5 那两行只罩见证（上一节）。G7 那几行我没逐行核，原因写在「这条腿自己的限度」。

## 从未被看过的地方：准入之前还在走全部容器与节点的步（不开格，交主 agent 定归属）

用户定案的读法在 D8（核心索引结构） 已定项 16 的射程段末句：「准入之前的步代价不随盘上容器总数、记录总数涨；准入之后装单元、拷字节不算运行时决策路径。」（`.claude/kb/decisions/08-核心索引结构.md:516`）。这一批把成员判断换成后缀起点之后，`mapped_units_of_a_container_tree` 照旧「for position in 0..tree.containers.len() {」（`crates/singlefs-core/src/transaction.rs:6923`）走全部容器、「for (position, origin) in tree.node_origins() {」（`crates/singlefs-core/src/transaction.rs:6937`）走全部节点（`node_origins` 里「let positions = self.shape.positions_in_bump_order();」（`crates/singlefs-core/src/container_tree.rs:2926`））。它在 `resolve` 调的 `plan_the_multi_level_trees` 里（三棵树各一次），`resolve` 在准入之前的固定点里跑 s 轮。墓碑树第一版不回收，容器数随删过的对象数涨。正文第二节说三处照抄归 E199、不开格；E199（发布照抄上一版的代价）第二份跑前登记第三节把「`node_origins` / `children_of` 与列全部容器、列全部节点那几遍」记成「射程外，不量」。照抄映射 key 那一半在 E199 的表里，「走一遍」那一半两边都不管。我不为它开格，写在这里交主 agent 定它归谁；量它的代价是纯计数，r1 B 簇那条钉代价测试的形状（大容器总数、少量点名）照搬就能红。

## 要主 agent 跑的命令

两条都不是重型（`singlefs-core` 的 lib 测试，不碰 checker 档包）。判决要引的数照规矩由主 agent 在自己的副本里复跑第 2 条。

1. 探针，在攻方草稿副本里跑（副本已接好探针，`.deliverables` 写成「交接」）：
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-core 的 lib 测试里攻方探针那一组 inode_record_lookup_by_leaf_order_over_overlapping_leaves_probe; 范围=1 个测试目标; 为什么=m4-close-code-r2 攻方腿 T4 交叠叶的探针：对照组应过、三条交叠用例应红' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-close-code-r2-attack/repo -- cargo test -p singlefs-core --lib inode_record_lookup_by_leaf_order_over_overlapping_leaves_probe -- --show-output`
   - 在哪起：`--cwd` 即副本根 `/home/fy5090/.local/share/singlefs/work/m4-close-code-r2-attack/repo`。
   - 看哪几行：`TASK_SERVER_RESULT` 的 `exit=` 应是 101（有测试红；0 说明交叠三条全绿，打中被推翻；编不过是别的码）。`task-server/task log <号> --all` 里：`leaves_in_strict_key_order_agree_on_every_lookup_count_and_removal ... ok` 一行、`overlapping_leaves_… ... FAILED` 三行、`test result:` 那一行「1 passed; 3 failed」；`probe=` 起头的行贴进承接腿的报告（`probe=after_writing … maintained=… recounted=…` 那一行给出漂了多少）。
2. 模型复跑（新 shell 里起）：
   - 命令：`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-close-code-r2-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-close-code-r2/repo --work /home/fy5090/.local/share/singlefs/work/m4-close-code-r2-attack-rerun`
   - 在哪起：仓根；`--work` 给一个还不在的目录（主 agent 复跑时换成自己草稿目录下的新目录、`--model` 指自己拷的模型目录）。
   - 看哪几行：`TASK_SERVER_RESULT` 的 `exit=0`；`log` 末行 `name=rerun_attack_m4_close_code_r2 cargo_exit=101 control_ok=1 overlap_failed=3 probe_lines=…`。`exit=1` 时看同一行哪一项不对。脚本里那条 `cargo test` 自带范围声明（驱动脚本逐行判那一条）；外面这一条要不要另带声明，照 `run-scope-guard.sh` 拒绝说明办。

跑完新派同立场的腿承接，读 `task-server/task log <号> --all`，把「推的，待主 agent 跑」改成量到的数。

## 没打中的形状

| 格 | 试过的形状 | 取样范围 |
|---|---|---|
| T1 | 旧根装满后一次追加连长两层（扇出 2，上一版 2 片、追加 3 片）；长高时旧根下面有容器重写；不追加（`carried`）；规格前后逐句比 | 手推 3 个形状；规格比对覆盖全部输入（证明） |
| T2 | 六条路径逐条找三棵容器树的构造处：追加、开片、unlink 合并叶、截断、整理搬迁、管理员回退与回退之后第一次发布 | `crates/singlefs-core/src/` 全部构造处（grep） |
| T3 | 搬迁的上一版拷贝、恢复之后盖映射位置、回退那一版拼 `units`、长高时旧根的角色、恢复读出的节点次序 | `TransactionOutput::units` 的全部写处（grep） |
| T4 其余 | nlink 归零、截断三步、改 size、建删快照、回退（照抄目标那一版的两个数，目标由恢复数一遍）、整理（不写记录）都走 `after_writing` / `after_removing` 或不动记录；同一次写两次同号（写者先拒非严格升序，`after_writing` 那一支走不到）；恢复数一遍只在挂载与回退时、不在每次发布上；池级 checker 的 I-3.20（b）拿盘上现数比标签 6 | 读码 |
| T5 G7 | 攒批、不攒批、冻结、回退、挂载续做五种起整理的处境 | 读码 |

上面「没打中」都只抽了一次，照 `.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算` 不支撑「没打中」的结论。

## 这条腿自己的限度

- 探针没编、没跑：交叠三条红、对照组绿都是手推的，`partition_point` 的交回值是照它的文档读的。探针的编译错（字段名、可见性）我只按源码核了一遍。
- 「恢复收下交叠镜像」只读了 inode 叶读取那两个函数与根区间那一判，没造镜像去挂；推翻条件写在 T4 节末。
- T5 只看了 `gc_ring_interval_proofs.rs` 改动的那几处与 `verified_model.rs`，没起 Verus，没逐行核 G7 那几条必红世界行与投影引理的新证明（Verus 的轻型结果按正文第二节，主 agent 跑的）。
- 准入之前列全部容器与节点那一节只看了代码，没量；E199 登记读的是第二份跑前登记的第三节与第十一节那两行。
- 正推腿的报告在禁读清单里，没读；与它重不重由主 agent 判。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；副本上量出的数（要是承接腿量到）不算入库装置上的数。
- 没往统一任务服务交任务；探针与模型复跑的命令在「要主 agent 跑的命令」一节，判决要引的数没复跑。
- 没写整条链的 harness 用例（造交叠镜像、挂载、写、checker 判），也没写改法甲的补丁。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-close-code-r2-attack/` 里留着：`repo/`（47M，仓副本，`.deliverables` 写成「交接」：主 agent 在里面跑第 1 条命令）、`mount.rs.before-probe`（接探针之前的原样，备份）；没有编译目录。

交回前 cite-check（`python3 scripts/agents/cite-check.py research/prompts/m4-close-code-r2-opus-output.md`，仓根起）末行：「✓ 核了 37 处引文、5 处裸行号，对上 37 处，没判 5 处（其中裸行号没有引文可比 5 处；逐处列在上面）」；`--root` 换成冻结副本再跑一次，末行相同。
