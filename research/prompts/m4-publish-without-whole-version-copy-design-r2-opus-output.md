# m4-publish-without-whole-version-copy-design-r2 攻方腿报告

日期：2026-11-18。设计推论轮第二轮，攻方腿（Opus）。前几轮判决：`research/prompts/m4-publish-without-whole-version-copy-design-r1-main-verification.md`（第一轮）、`research/prompts/m4-gpugc-publish-growth-r1-main-verification.md`（2026-11-06）。
攻击面三样：第一轮的修补本身（甲三处难点的解法）/ 产生结论的方法（探针）/ 从未被看过的地方（三条共有那一格、测试与装置里的读者）。不再攻乙的「版本是值」、丙的链长、甲三处难点存不存在。
行号都是主工作区此刻的树（`crates/singlefs-core/src/transaction.rs`、`allocator.rs` 此刻与 HEAD 相同，`git status` 没列它们）。没改主工作区的 `crates/`。没读禁读清单里的文件。

## 复跑命令与文件指纹

模型目录 `research/prompts/m4-publish-without-whole-version-copy-design-r2-opus-model/`：

```
1087455f1968a348297499ac175903689d8d73c2c3051e254e6a4536d9b5cda6  probe/publish-change-probe.diff
ed6c56006b8279bd6ee9c2c6c57a96283d3c247ebadee1051e49a6874ad930c5  tools/rerun.sh
e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b  tools/rerun-from-a-clean-work-directory.sh
5e82b13a512da37919e2c488d22f4b4367db254c4c78d73a62708ffce0adac0e  tools/summarize_publish_change_probe.py
566be7a19b87350393bac231683c295a6ba9330651b859f260b76c2339b65b2c  tests-and-bins-readers-of-units-and-allocation-records.tsv
```

- `probe/publish-change-probe.diff`：探针，只数不改（相对主工作区此刻的四份文件加一份新文件，`patch -p1` 打，打在我留存的原件上 `patch --dry-run` 五份都过）。
- `tools/summarize_publish_change_probe.py`：读装置产物，另算一遍分布、与装置自己打的分布行逐格比，完整性闸，再照跑前写死的翻面读数打 `name=flip_reading` 行。
- `tools/rerun.sh`：照 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps`：对 `--frozen` 那棵树建最小副本、打探针、跑装置、删编译目录、跑汇总。模板自带的 `--selftest` 原样留着，它用假仓跑 `rerun_steps`，换过的这一段在假仓上起不了 `minimal-copy.py`，这份脚本的 `--selftest` 会红（推的，没跑）。
- `tests-and-bins-readers-of-units-and-allocation-records.tsv`：第三·2 节逐行分类的整张表（318 行，加表头 1 行）。
- 复跑：「要主 agent 跑的命令」第 3 条（`rerun-from-a-clean-work-directory.sh <仓根> <冻结副本> <运行目录> <模型目录>`，冻结副本给仓根或开工那一刻的树）。
- 探针的内存与时间（推的，没跑）：同一档主工作区的造池一趟 159.792 秒（背景材料引的任务 19694）；探针每次发布另做一份分配器拷贝、三张表各排一次序逐条比、两块盘各 4 张逐槽位图逐位比，按 6566 次发布（上一趟同档产物 `construction_timing … publishes=6566`）每次 3–5 ms 估，多 20–33 秒；编译另算。常驻内存同上一趟约 1.08 GB 量级（那一趟产物 `resident_bytes=1079296000`）。估出的全量挂钟远低于 40 分钟，不缩取样。
- 这一条腿没有崩溃状态要枚举：探针不录写流、不调 `enumerate_crash_states`。

## 各格判定一览

| 攻击面 | 格 | 判定 | 一句话 |
|---|---|---|---|
| 一 修补本身 | 反查索引（第一轮攻方给的） | 撤回（我第一轮的方案） | 搬迁计划里每个单元已带它在上一版里的角色（`RelocatedUnitHolder::RoleOfThisVersion`），按 key 找角色的索引今天就有（`mapped_units_by_key`），缺的只是「角色 → 指针所在处」这一张写死的表，不随发布维护、不随全版涨 |
| 一 修补本身 | 反查索引与审计独立 | 没打中 | 写者内部用的表，checker 不读；与恢复路径的整版走一遍是同一语义的两条实现，测试构建里两条比一遍即可，不是「审计与被审计共用代码」 |
| 一 修补本身 | 另立的键：存下标的读者 | 打中（一处测试的世界在甲下写不出） | `container_tree_root_units_read_at_their_recorded_positions.rs` 靠「同一角色在 `units` 里两项、假的在前」造世界；按角色当键的表装不下两项，这条测试与它守的 `ContainerTreeRootUnitPositions` 一起没了对象 |
| 一 修补本身 | 另立的键：复刻次序、同一角色多项 | 推不出，交探针 | 位置寻址的角色在照抄时会改名（`CarriedFrom(previous_index)`、`identity_in_the_previous_version`）；改名的项按角色当键算加一删一。探针数「照抄换了角色的项」与「两版共有角色的次序翻转」「同一角色多项」，跑前写死翻面线（T2、T3） |
| 一 修补本身 | 二层分块：哪一种发布退回整份拷贝 | 打中（正推自己的第一版） | 正推第一版「从各树建出来的 `Vec` 经 `from_sorted` 建表」每次发布从头切块，两版之间一块都不共用；甲的保留与相等快路全落空，只剩克隆变 O(1)。散在全池的释放（删垫片）是第二种，交探针量（T1） |
| 一 修补本身 | 二层分块：相等怎么判 | 打中（会假红，不会假绿） | 两处测试拿「从盘上重建的一版」或「另一个池里发出来的一版」与发布出来的一版逐条比分配记录；块界随建法与历史不同，按块派生的相等会判不等 |
| 一 修补本身 | 二层分块：析构递归 | 没打中 | 两层，节点之间不成链；版本之间没有引用 |
| 二 方法 | d 的探针 | 给出，待主 agent 跑 | 核心层发布路径上只数不改；三段分布、两条路互核、完整性闸、五条跑前写死的翻面读数 |
| 三 没看过的 | 共有那一格：发布前克隆是不是变常数 | 打中（推的，探针量大小） | 记录表与空闲图之外，`PoolAllocator` 还有随池涨的集合：聚簇段集、已回收集、根环里被抛弃根的整份落点；只换记录表与空闲图，克隆降到这几样之和，不到常数（T5） |
| 三 没看过的 | 共有那一格：冻结发布之后那一份 | 没打中 | 持久共享下「发布之前」是常数代价的克隆、「发布之后」就是改过的那一份，冻结照今天的 `std::mem::replace` 走，两份都在；2026-11-06 那一轮 Q2.3 ③ 的难处只属于撤销日志那种写法 |
| 三 没看过的 | 共有那一格：六种没有撤销步的改动 | 没打中（换了一个前提） | 持久共享不要撤销步，这六种照常改、改到的块各复制一次；前提是它们改的每一个字段都进了持久结构（逐盘十三个字段，第三·1 节列） |
| 三 没看过的 | 测试与装置里的读者 | 逐行分完 | `.units` 173 行、`.allocation_records` 145 行（单元图包除外）；在甲下要改的：`.units` 17 行（下标 4、改写 11、次序 1、交切片 1），`.allocation_records` 54 行（交切片）；另有相等 4 行在块界不定的实现下会假红 |

## 攻击面一：第一轮的修补本身

### 一·1 反查索引：撤回第一轮我给的那一条

第一轮我写的是「要真到 O(批大小)，得另有一份「映射 key → 哪一处指针」的反向索引，跟着每次发布维护」（`research/prompts/m4-publish-without-whole-version-copy-design-r1-opus-output.md:67`），同一行还写了「这份索引今天没有」；判决把它记成零轮改法：「甲在带搬迁的发布上要另加映射 key 到指针的反查索引（攻方）：零轮。」（`research/prompts/m4-publish-without-whole-version-copy-design-r1-main-verification.md:51`）。两处都撤：

1. 按 key 找角色的索引今天就有，每批整理建一次：「mapped_units_by_key: version.mapped_units_by_key(),」（`crates/singlefs-core/src/transaction.rs:11346`），计划里的每个单元拿它查出自己在上一版里的角色，记进搬迁项：「RelocatedUnitHolder::RoleOfThisVersion(*role),」（`crates/singlefs-core/src/transaction.rs:11390`）。所以发布路径收到的 `RelocationOfThePublish` 已经点名每个被搬单元的角色；留给快照的那一种没有角色，只在 `mapping_entries_kept_for_the_snapshots` 里，按 key 找。
2. 带搬迁的发布今天不用这个角色，而是拷上一版之后整版走一遍、对每一处指针现算 key 去问：「let mut relocated_version = previous_version.clone();」（`crates/singlefs-core/src/transaction.rs:15822`），之后调 `take_the_mapping_positions_over_the_location_hints`，最后按建好的索引改 `units`：「version.units[*position].slot = slot;」（`crates/singlefs-core/src/transaction.rs:4107`）。
3. 角色本身带位置（`TransactionUnit` 的 `Data(DataUnitIndexInFile)`、`AllocationTreeNodeBelowTheRoot(AllocationRecordTreeNodePosition)`、`DataOfAFileOtherThanTheFirst { inode, unit }` 这一类，`crates/singlefs-core/src/transaction.rs:2695` 起），所以「角色 → 这一版里指向它的那几处」是一张按枚举成员写死的表：那一族的节点指针那一格，树根另有树表条目那一格，再加 `units` 里那一项的落点。它不随发布维护、不随全版涨；甲下每处都是一次按键取（O(log U)），一批 K 个是 O(K·log U)。

所以甲在带搬迁的发布上要的不是一份新索引，是把 `take_the_mapping_positions_over_the_location_hints` 的整理那一支改成按搬迁项逐个改；恢复那一支（`crates/singlefs-core/src/recovery.rs:3832`，每把 key 都有位置）照旧整版走，一次挂载一回。
「谁维护、放哪、随不随全版涨」三问的答案：没有要维护的新东西；按 key 找角色那一份若甲也做成持久的（不再每批整建 O(U)），它就是映射条目表按 key 排的那一份，每次发布改 d 条，与映射条目表同一张。

与审计独立冲不冲突：没打中。这张表与按 key 找角色的索引都只在写者里用；`.claude/rules/fs-design.md@记账是事务的副产品，不是事后的遍历` 禁的是「审计与被审计用同一段代码」，管运行时与 checker 的分界，池级 checker 不读版本（第一轮判决第二节第 1 条）。要防的是整理那一支（按搬迁项改）与恢复那一支（整版走）对同一把 key 改的处数不一样：测试构建里对每一批两条都走一遍、逐字段比，这是同一语义的两种实现互核，不是审计。

什么会推翻这一撤回：版本里有一处指针指向被搬单元、却不在「它的角色那一格、树表条目那一格、`units` 那一项」三处里（比如某个节点的字节里另存的指针在内存里也有一份）。核法就是上面那条互核：任何一批两条走法改出的版本不同，撤回作废。我只读了 `take_the_mapping_positions_over_the_location_hints` 的十几处 `&mut` 走法（`crates/singlefs-core/src/transaction.rs:3992`–`:4110`），每一处的 key 都由那一格指针自己算出，没见到第四种。

### 一·2 另立的键与存下标的读者

`units` 的次序今天由装配次序定：各族按 bump 次序依次 `push`（`crates/singlefs-core/src/transaction.rs:17032`–`:17248`），族内的项由各树的装法给。照抄的单元会换角色：inode 叶容器照抄上一版那个叶序上的一份、装进来换成这一版的叶序：「InodeLeafContainerOrigin::CarriedFrom(previous_index) => {」（`crates/singlefs-core/src/transaction.rs:17104`）；容器树根之下的节点照抄时按上一版的角色拷、换成这一版的角色：「node.identity_in_the_previous_version,」（`crates/singlefs-core/src/transaction.rs:17175`）。
按角色当键的表里，换了角色的照抄项是删一条、加一条：一次叶分裂或树长高，其后同一族的每一项都换角色，d 跟着那一族的项数走，不跟这次真写的走。这一格本身不是新攻点（正推第一轮已提），新的是能不能量：探针数「落点与字节那份都与上一版某一项相同、角色却换了」的项数（`units_carried_under_a_new_role`），与这次真写出的角色数（`rewritten_roles`）并列；跑前写死 T2。

次序能不能只看角色就复刻：探针数两样。一是两版都有的角色，按这一版的次序排、看它们在上一版里的下标，往回走了几次（`order_flips_among_common_roles`）——大于 0 就不存在只看角色的静态全序；二是同一种角色（同一个枚举成员）相邻两项按派生次序往回走的次数（`descents_within_a_role_kind`）——大于 0 就是族内次序也不是派生次序，键要另写族内那一段。同一角色多项：数 `repeated_roles`。三样都只在 E202 造池这一种历史上量，跑前写死 T3。

存下标的读者，现读：

- `ContainerTreeRootUnitPositions`（「pub struct ContainerTreeRootUnitPositions {」，`crates/singlefs-core/src/transaction.rs:3489`）三处装（`transaction.rs:17619`、`mount.rs:10794`、`recovery.rs:3770`），两处读（`transaction.rs:4457`、`:4469`，准入读两个 H）。
- `take_the_mapping_positions_over_the_location_hints` 里按角色记 `units` 下标的 `SortedIdentityIndex`（`transaction.rs:4093`–`:4107`）。
- `units_by_role`、`mapped_units_by_role`、`mapped_units_by_key` 三个访问器（`transaction.rs:4539`–`:4557`），每调一次整建一份按位置的索引。
- 测试与装置里另有 4 处 `.position(...)` 取下标再按下标改、11 处 `iter_mut`/下标赋值/`retain` 改版本（第三·2 节表里「下标」「改写」两类）。

甲下换成什么：按角色当键的表自己就是索引，三个访问器与 `SortedIdentityIndex` 都删；`ContainerTreeRootUnitPositions` 不要换成 `get(角色)`——正推第一轮写的是「索引访问器直接走 `get`，不再每次 `FirstEntryOfEachIdentity::of_the_table` 整建」（`research/prompts/m4-publish-without-whole-version-copy-design-r1-sonnet-output.md:114`），用在准入那两处就是 O(log U)。`.claude/rules/fs-design.md:16` 那一格写的是「| **运行时决策路径**（分配、ENOSPC 准入、defer 窗口、生命周期判定） | **不许**，代价不许随盘容量增长 |」；分配器自己在决策路径上已经用 `BTreeSet` 取最小（`segments_with_an_unblocked_slot_pair`，`allocator.rs:485`），O(log) 算不算「随盘容量增长」仓里没有判过，我不判，交主 agent。不碰这一问的写法：装版本时把两个根单元本身（`PublishedUnit`，字节是 `Arc`）各留一份放在版本里，准入直接读，O(1)。

在甲下写不出的那条测试（打中）：`crates/singlefs-verification/tests/container_tree_root_units_read_at_their_recorded_positions.rs` 先断言「assert!(real_root_position > 0, "前提：真根单元不在 units 头一项");」（第 24 行），再把头一项改成同一个角色、字节全 0 的假根：「let decoy = &mut tampered.units[0];」（`crates/singlefs-verification/tests/container_tree_root_units_read_at_their_recorded_positions.rs:26`）。它守的是「准入按记下的下标取、不逐项找」；按角色当键的表里同一角色只有一项，这个世界构造不出来，逐项找与按下标取也没有区别。甲落地时这条测试与它的变异行一起删，守「准入不逐项找」的那一判要换成另一种形态（比如数准入那一路进了几次按键取，`lookup_work` 已有的计数口径）。这一格分辨臂：乙、丙不换 `units` 的表示就不碰它。

### 一·3 自己写的二层分块持久表

正推第一轮的丁：「有序序列切成每块至多 B 条的 `Arc` 块，外面一条 `Arc<Vec<Arc<块>>>` 的脊」（`research/prompts/m4-publish-without-whole-version-copy-design-r1-sonnet-output.md:65`）。

**哪一种发布让它退回整份拷贝**（打中两种，第三种交探针）：

1. 正推自己的第一版。它的分步表写「第一版：从各树建出来的 `Vec` 经 `from_sorted` 建表（仍 O(U)，换到的是之后的克隆、保留、换回为 O(1)）」（`research/prompts/m4-publish-without-whole-version-copy-design-r1-sonnet-output.md:113`）。`from_sorted` 按条数从头切块：上一版与这一版只要在前面差一条，其后每一块的起止都错开一条，两版之间一块都不共用；块界改成按键切也一样，从 `Vec` 现建的块都是新分配的 `Arc`，除非逐块与上一版比内容、相同就换回上一版那一块（那一比本身 O(N)）。后果：留着的 V 版各占一整份（内存一格没买到）；正推「版本相等」一格靠的块指针相同快路一次都不中，比相等退回逐条 O(N)；「换回之后根指针与发布前相同」那一钉只在克隆那一刻成立。买到的只剩「克隆一份是 O(N/B)」。
   反例（一串发布）：任何两次相邻的用户覆写，第二次在分配记录表头部附近多一条记录（两槽单元落在单元区低端的空槽，`try_allocate_user_data` 取最低空槽对）。块界按条数切，这一条之后全部块都换了内容。
   修法（推的，零轮）：块界按键取、不按条数取（正推对分配记录写的就是按分配记录树的叶界切，叶罩 `[k×W,(k+1)×W)`），并且发布路径只改碰到的块、不从头建；也就是正推的「第二版」（它的第 5 步），而那一步它自己写「推不出」。所以丁在第 5 步做完之前，对每次发布的代价是零收益。
2. 位置寻址的角色换名（一·2）：`units` 按角色切块时，叶分裂、树长高让其后同一族的每一项换键，碰到的块数跟着那一族的项数走。量法与翻面线同 T2。
3. 散在全池的释放：删一个散着落的大文件，释放的记录分在全池各处的叶里，碰到的块数趋近全部块数。E202 造池的「删垫片」那一段正是这一形（垫片与活数据交错写，删垫片释放的槽遍布已写的段）。探针按上一版键次序每 64、256 条一块数碰到的块（`*_blocks64`、`*_blocks256`），汇总脚本算「碰到的块 × 块长 + 块指针数」占全表的比；判在交错写那一段（T1、T4），删垫片那一段只报。

**相等怎么判**（打中，假红不假绿）：两处测试拿不同建法的两版逐条比分配记录。

- 「after_the_move.allocation_records, before_the_move.allocation_records,」（`crates/singlefs-verification/tests/tree_nodes_and_the_central_mapping_on_mounted_read.rs:722`）：`after_the_move` 是 `rebuild_version` 从盘上重建的（同一文件第 700 行起），`before_the_move` 是发布出来的。
- `crates/singlefs-verification/tests/sequential_write_one_record_per_data_unit.rs:105`：两个池各自发出来的两版比 `allocation_records`，同一文件第 96 行比 `units`。

块界随建法（`from_sorted` 对增量改）与历史不同，若相等照 `#[derive(PartialEq)]` 按块比，这几处内容相同也判不等、测试红。方向是假红：派生的按块比在内容不同的时候不会判相等。所以这一格不会让错过去，只会让正确的实现过不了测试；修法是手写按内容比（先块指针相同快路，再逐条），正推第一轮已写「不许按树形状或节点哈希比」，这里补的是会撞上它的那几处读者。另有 8 处 `assert_eq!(pool.output, current_before, …)` 比的是同一份克隆前后，块界相同，不受影响。

**析构会不会递归太深**：没打中。两层结构，脊里是块的 `Arc`，块里是条目；版本与版本之间没有指针，`versions_published` 是 `Vec`，逐个析构。冻结的那一份分配器挂在分配器上（「frozen_publish: Option<Box<FrozenPublish>>,」，`crates/singlefs-core/src/allocator.rs:2326`），冻结时调用方手里那一份的 `frozen_publish` 是空的（`refuse_while_a_publish_is_frozen` 先拒），只嵌一层。试过的形状：丙那种一版指上一版的链（第一轮已列，不重复）、持久 B 树（深 log_F N，F = 32、N = 66244 时 4 层）、冻结嵌冻结（被先拒挡住）。

**行数与验法**：不攻（辩方那一格）。

## 攻击面二：产生结论的方法——d 的探针

第一轮「甲最站得住」靠读码与我自写的小模型，没有一个数出自核心层（第一轮判决第一节「核不动」两条）。下面这一份探针在核心层量，只数不改。

### 二·1 探针怎么数

副本 `/home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r2-opus/copy`（`python3 scripts/hosts/minimal-copy.py` 从主工作区建，`--with-test-inputs`）。改动即模型目录的 `probe/publish-change-probe.diff`：

1. 接点：`publish_version_of_trees_relocating_units_after_a_check`（`crates/singlefs-core/src/transaction.rs:13193`）。带树的发布全经它（调它的五处：`transaction.rs:11110`、`:11180`、`:12917`、`:13131`、`:13269`；「全经它」是读这五处的调用方推的，完整性闸兜），落盘成了之后（`persist_the_publish_or_freeze_it` 交回 `Ok`）拿上一版 `previous` 与这一版比；冻结、被拒的不数。发布之前另拷一份分配器只给探针比（原来那一份照旧交给落盘与冻结），这一份拷贝不进任何状态。树表 0 条那一版写行的发布（`transaction.rs:1930` 那一路）不数，只在建池开头一两次。
2. 三张表都按「两份按键有序的表逐条比」数加、删、值变（新文件 `publish_change_probe.rs` 的 `change_of_sorted_tables`）：
   - `units`：键是角色（`TransactionUnit` 的派生次序，同一角色取最先一项），值是（落点, 字节那份 `Arc` 的地址）。照抄的项 `Arc` 不换，值不变；重写的项值变；换角色的项是删一加一。
   - 中央映射条目：键是映射 key，值是第一份位置项的槽（进映射的角色取它在 `units` 里的落点，加上留给快照的条目）。
   - 分配记录：键是 (设备, 槽号)，值是整条记录。
   每张表另按上一版键次序每 64 条、每 256 条一块，数加、删、值变落到几块（加的按插入点）：这就是二层分块持久表（块界按键）每次要复制的块数。
3. 角色次序三样：同一角色多项、同一种角色内派生次序往回走、两版共有角色的相对次序翻转；照抄换了角色的项数；这次真写出的角色数；搬走的单元数。
4. 分配器（`allocator.rs` 末尾新加的 `change_count_since`，只读）：发布前后四种逐槽位有一种变了的槽数、按 4096 槽一块碰到的块数、六种逐段计数有一种变了的段数；此刻聚簇段集、已回收集、每盘两张段号集、根环里被抛弃根的落点数与分配器记录条数。
5. 装置（副本里的 `e202_gpu_cpu_gc_comparison.rs`）：造池的钟每开一段、收一段，把探针表整份取走，按段打 `name=publish_change_probe_row`（每次发布一行）与 `name=publish_change_probe_phase`（每段每张表一行分布：d 与 N 的最小、中位、最大，d ÷ N 的最小、中位、最大，按 64 与 256 条一块的拷贝比中位与最大）。

汇总脚本 `tools/summarize_publish_change_probe.py` 只读产物里的逐次发布行，另算一遍分布，与装置打的分布行逐格比（`d_median`、`copy64_over_n_median`、`d_over_n_max`），对不上退 1；完整性闸：每段探针数到的、有上一版的发布次数不少于装置那一段自己报的 `publishes=`，少了说明有带树的发布路径没走接点，这一趟作废、退 1。
三段：顺序写 `sequential_half`、交错写 `interleaved_half`、删垫片 `unlinking_the_spacer_files`（「放开垫片的槽」那一段上一趟 `publishes=0`，不判）。

### 二·2 跑前写死：什么读数让「甲值得做」翻面

判在交错写那一段：上一趟同档（2288MiB、P=4096、shared）它占造池挂钟的大头（`phase=interleaved_half seconds=48.125`，总 `construction_seconds=63.438`，第二批提速之后那一趟的产物）。另两段只报。拷贝比的口径：一次发布在一张表上，按 64 条一块要复制「碰到的块数 × 64 + ⌈N ÷ 64⌉ 个块指针」条，除以 N（N 取两版里大的那个）。

| 读数 | 翻面线 | 翻成什么 |
|---|---|---|
| T1 分配记录，交错写段 `copy64_over_n` 中位 | ≥ 0.25 | 「甲在分配记录表上值得做」→「不值得」：块界按键的持久表每次仍要复制四分之一以上，换不到一个量级 |
| T2 `units`，交错写段 `units_carried_under_a_new_role` 中位 | > `rewritten_roles` 中位 | 「按角色当键」→「角色不是合适的键」：照抄换名的项比真写的多，d 由换名定 |
| T3 全部段 `repeated_roles` 最大或 `order_flips_among_common_roles` 最大 | > 0 | 「另立的键能复刻今天的次序」→「不能」：同一角色多项，或不存在只看角色的静态全序；`descents_within_a_role_kind` 最大 > 0 另记「族内次序不是派生次序」 |
| T4 中央映射条目，交错写段 `copy64_over_n` 中位 | ≥ 0.25 | 同 T1，映射条目那一张 |
| T5 全部段（聚簇段集 + 已回收集 + 根环里被抛弃根的落点）÷ 分配器记录条数，最大；每盘空闲图里那两张段号集算在空闲图里，不进分子，另行报 | ≥ 0.05 | 「记录表与空闲图换持久之后发布前克隆近似常数」→「不是」：剩下的集合仍与池同阶、超过今天记录表那份克隆的二十分之一 |

结果反过来我接受的写法（跑前写下）：T1、T4 都没翻，我收回「甲对每次发布买不到东西」这一类怀疑，第三轮攻方的靶子换成删垫片那一段与正式跑；T2、T3 都没翻，一·2 里「按角色当键会被换名拖累」降为只在 E202 之外的历史上待证；T5 没翻，第三·1 节的打中降为「推的、E202 上不显」。
建这份探针时我倾向的是：交错写段 d 很小（每次发布改到几十条，比 N 小三个量级以上），T1、T4 不翻；删垫片那一段拷贝比接近 1；T5 翻（聚簇段集随发布次数涨）。这几条倾向不进判定，判定只看上表。

d 本身与全版成正比的发布形状（正推列的删或截断大文件、整文件覆盖、整理搬大批）在 E202 里占不占：造池那一部分由这一趟的删垫片段回答（它就是「删一个散着落的大文件」）；正式跑（lockstep、concurrent 里的用户改动与整理批）这一趟不量，整理一批至多 16 个单元（`COMPACTION_BATCH_UNITS_BY_DEFAULT`）是 d 小的那一头，用户改动的形状要读装置的 `--user-changes` 那一支，我没读，推不出。

取样只在一个点：2288MiB、P=4096、shared。「不许先有结论再建模型」第四、六问没答满：d 随 P 与盘大小怎么变，这一趟回答不了；要扫就在 `--live-data-units` 给 `1024,2048,4096` 三个 P（同一个进程串着造；装置一侧每个 P 一个钟、探针行带 `live_data_units=`，分得开；汇总脚本只按段分、没按 P 分，要扫先改脚本，这一版没做）。

## 攻击面三：从未被看过的地方

### 三·1 三条共有的那一格：分配器换成持久共享之后

发布前那份整份克隆：「let allocator_before_this_publish = allocator.clone();」（`crates/singlefs-core/src/transaction.rs:13213`），注释说它是「两盘各一张 211968 位的位图」的拷贝（同一函数第 13212 行）。`PoolAllocator`（`crates/singlefs-core/src/allocator.rs:2284` 起）里随池涨的字段，现读：

| 字段 | 位置 | 涨随什么 | 在不在「记录表与空闲图」里 |
|---|---|---|---|
| `records: Vec<AllocationRecord>` | `allocator.rs:2294` | 分配记录条数 R（上一趟同档末了 66244） | 记录表 |
| 每盘 `DeviceFreeMap` 的四张逐槽 `Vec<bool>`（`allocated`、`isolated`、`held_until_floor_takes_effect`、`is_under_an_unreleased_record`） | `allocator.rs:439`、`:462`、`:470`、`:488` | 单元区槽数 | 空闲图 |
| 每盘六张逐段计数（`used_per_segment`、`unreleased_per_segment`、`isolated_per_segment`、`held_per_segment`、`unblocked_slot_pairs_per_segment`、`slot_pairs_without_an_unreleased_record_per_segment`）与逐叶一张 | `allocator.rs:441`、`:445`、`:463`、`:471`、`:479`、`:492`、`:498` | 段数、叶数 | 空闲图 |
| 每盘两张段号集（「segments_with_an_unblocked_slot_pair: BTreeSet<usize>,」、「segments_reclaimed_not_yet_discarded: BTreeSet<usize>,」） | `allocator.rs:485`、`:501` | 段数 | 空闲图（不是数组，持久化要另一种容器） |
| 「cluster_segments: BTreeSet<SlotNumber>,」 | `allocator.rs:2293` | 这次挂载开过的聚簇段数，开过就一直算数 | 不在 |
| 「reclaimed: BTreeSet<(DeviceIdentity, SlotNumber)>,」 | `allocator.rs:2296` | 已回收、还没复用的落点数 | 不在 |
| `root_ring` 里被抛弃根的账：「referenced_placements: Vec<PlacementOnDevice>,」 | `allocator.rs:1708` | 一条被抛弃根引用的全部落点（一整版） | 不在 |
| `compaction` 里发布路径记下的隔离槽 | `crates/singlefs-core/src/compaction_plan.rs:804` | 隔离次数，小 | 不在 |

所以「记录表与空闲图换成持久共享之后克隆自然变常数」不成立（打中，读码推的）：剩下聚簇段集、已回收集、被抛弃根的落点三样，克隆降到它们之和，不到常数。三样各自涨多快是数据问题：探针打 `cluster_segments`、`reclaimed_placements`、`abandoned_root_placements` 每次发布的值，T5 判。被抛弃根那一样 E202 造池里应是 0（不崩不挂回退，推的），它的代价出现在「挂载时有被抛弃根」的那一类历史上：一整版落点随每次发布前克隆拷一次，直到那条根离开根环；修法是装好之后不再变，挂载时包一层 `Arc`（推的，零轮）。
这几样要么也做成持久结构，要么接受克隆随它们涨；两样都比「只换记录表与空闲图」多出改动，第一轮判决「三条共有」那一笔的射程要把它们写进去。

**算不算甲的一部分、放在分步的哪一步**：分配器的记录表不是可选的——版本里的分配记录今天是每次发布从分配器整份拷出来再排序（第一轮判决第一节现读的 `transaction.rs:16232`），甲要让版本里那一张不随 R 拷，就得让版本直接共享分配器那一份，所以分配器的记录表与版本的分配记录是甲的同一步（正推第 3 步已这么排）。空闲图与上表那几样不在版本里，与版本表示无关，是另一步；它的收益只落在发布前克隆与结算演练的克隆（`transaction.rs:14076`）上，排在甲之后、单独量。

**落盘失败冻结「发布之后那一份」**（没打中）：冻结那一支今天是「let allocator_after_the_publish =」「std::mem::replace(allocator, allocator_before_this_publish);」（`crates/singlefs-core/src/transaction.rs:1662-1663`），把改过的那一份挪进 `FrozenPublish`、把发布前的克隆换回去。持久共享下「发布前的克隆」是常数代价（全部字段都持久的前提下），「发布之后」就是被改过的那一份本身，两份共享没改的块，挪动不拷贝；这一支一行不用改。2026-11-06 那一轮 Q2.3 ③ 的难处——撤销日志只给得出发布之前那一份——只属于撤销日志那种写法。

**空闲图上六种没有撤销步的改动**（没打中，换了一个前提）：实现员报的六种（复活、两种隔离、清隔离位、放开扣住、摘 discard 段，`~/.local/share/singlefs/work/e202-pool-construction-speedup-writer-1/result-3.md` 第 278 行，副本里的行号）在持久共享下不需要撤销步：发布前那一份是独立的值，发布里怎么改都不动它。成问题的前提换成了另一句：它们改到的每一个字段都得在持久结构里。按上表，复活与隔离改逐槽位与逐段计数（数组，块状持久化可做），摘 discard 段改 `segments_reclaimed_not_yet_discarded`（集合），放开扣住改 `held_until_floor_takes_effect`；漏一个字段不做持久，那一个就退回「改了不能撤」——且没有任何断言会报（今天的撤销账那一版在演练里碰到就断言，持久那一版漏了是静默的，推的）。修法：给 `PoolAllocator` 的克隆写一条只在测试构建里的检查，克隆之后在拷贝上改、断言原件逐字段不变（零轮）。
另一句与两条线的次序有关（第一轮我报过「两条线改同一处」，这里只补新的一点）：造池提速第二批在副本里加了演练的撤销账（`rehearse_and_undo`）；分配器做成持久之后，演练可以回到「克隆一份、改、丢」，那套撤销账整个多余。先合撤销账、后做持久，那一套要整删。

### 三·2 测试与装置里的读者，逐行分类（不抽样）

范围：`grep -rnP '\.units\b' crates/*/tests crates/*/src/bin --include=*.rs` 去掉 `crates/singlefs-unit-graph` 那 20 行（单元图包自己的同名字段）得 173 行；`grep -rnP '\.allocation_records\b' crates/*/tests crates/*/src/bin --include=*.rs` 得 145 行。每一行按它前两行、后三行读接收者的类型与用法，分进一类；一行只记主用法。整张表在模型目录的 `tests-and-bins-readers-of-units-and-allocation-records.tsv`（表头 1 行 + 318 行）。

分类口径：「非版本」= 接收者不是 `TransactionOutput`（录制流的 `PublishWrites` 写表下标、崩溃事实表、装置自己的结构、`.units()` 方法、同名局部变量、格式串与注释）；「写行版」= 树表 0 条那一版的 `VersionWithoutFilePublishOutput`（另一个类型）；其余是 `TransactionOutput` 的字段，分「下标」（取下标再按下标读）、「改写」（`iter_mut`、按下标赋值、`retain`）、「次序」（结论依赖项的先后）、「相等」（整份 `assert_eq!`）、「遍历」（`iter`/`for`/`find`/`filter`/`any`/`all`/`contains`/`{:?}`）、「个数」（只读 `len`、`is_empty`）、「交切片」（`&x.allocation_records` 交给收 `&[T]` 的函数、`.as_slice()`、`.to_vec()`、`.clone()` 成 `Vec`）。

| 字段 | 非版本 | 写行版 | 遍历 | 个数 | 相等 | 下标 | 改写 | 次序 | 交切片 | 合计 |
|---|---|---|---|---|---|---|---|---|---|---|
| `.units` | 80 | 8 | 65 | 2 | 1 | 4 | 11 | 1 | 1 | 173 |
| `.allocation_records` | 20 | 4 | 49 | 15 | 3 | 0 | 0 | 0 | 54 | 145 |

（数出自 `cut -f1,2 <表> | sort | uniq -c`，命令在草稿目录跑过，输出见上表各格。）

甲下要改的（假定持久表给 `iter()`、`IntoIterator for &`、`len()`、`is_empty()`、按键取、按内容的 `PartialEq` 与列表样的 `Debug`）：

- 「遍历」「个数」不改（推的；`.units` 里有 1 行用 `Vec::contains`，`crates/singlefs-verification/tests/fsync_drop_and_devices_without_the_selected_version.rs:436`，要表也给 `contains`）。
- 「下标」「改写」「次序」「交切片」全改：`.units` 17 行，全在测试与两个装置里——`crates/singlefs-verification/tests/release_checksum_quarantine.rs:912`、`:916`；`crates/singlefs-verification/tests/overwrite_in_one_instance.rs:502`、`:506`、`:622`、`:626`、`:657`、`:661`（这三对是「找到映射树根、换掉它的字节」，改成按角色取可写的一项）；`unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired.rs:2558`；`crates/singlefs-verification/tests/unlink_mount_continuation_and_the_emptied_version.rs:882`；`crates/singlefs-verification/tests/common_images_for_the_given_states_engine/publish_lookup_by_identity.rs:601`、`:935`、`:1022`（`:601` 拿 `&version.units` 当切片核按角色索引「每个角色列一次、取最先那一项」，甲下这个索引本身删掉，这一判跟着删）；`container_tree_root_units_read_at_their_recorded_positions.rs:20`、`:26`（一·2，删）；`crates/singlefs-implementation-experiments/src/bin/e180_admission_reading_versus_writable_bytes/pool_and_publish_log.rs:143`、`:733`（装置把数据单元的字节按下标摘下、再按下标装回，下标要换成角色）。
- `.allocation_records` 的 54 行「交切片」：37 行交给核心层的公开函数（`region_is_empty_by_the_records` 22、`empty_cluster_segments_by_the_records` 5、`live_slots_per_cluster_segment_by_the_records` 4、`nodes_whose_contents_changed` 2、`region_scans_of_the_segments_without_an_unreleased_record`、`choose_region`、`scan_region`、`scan_region_over_the_identity_index` 各 1），这几个函数的签名在产品那一侧本来就要改（正推第一轮 3.2 表），测试跟着签名机械改；13 行交给测试与装置自己的小函数（`record_of` 4、`a_later_record_covers` 3、`live_slots_per_device` 2、另 4 个各 1）；4 行把整份拷成 `Vec`（`.clone()` 2、`.as_slice()` 1、`.to_vec()` 1），甲下照拷、代价不变。按文件：E202 装置 11 行最多，其次 `compaction_intent_check_fallbacks_reached_by_transient_read_errors.rs` 4 行。
- 「相等」4 行不改写法，但在块界不定的实现下会假红（一·3）：`sequential_write_one_record_per_data_unit.rs:96`、`:105`，`tree_nodes_and_the_central_mapping_on_mounted_read.rs:722`，`crates/singlefs-verification/tests/new_pool_file_creation_publish.rs:1155`（拿手写的期望 `Vec` 比）。
- 「次序」：`.allocation_records` 0 行。我没找到哪一行的结论依赖分配记录的先后（`record_of`、`a_later_record_covers`、`region_is_empty_by_the_records` 都是逐条找；`nodes_whose_contents_changed` 进去先按叶分组、组内排序，`crates/singlefs-core/src/allocation_record_tree.rs:331`）。`.units` 1 行（上面那条测试的前提）。

这一格分辨臂：乙与丙也要动 `.units`、`.allocation_records` 的这些读者，但改法不同（乙、丙下读到的不再是全版，第一轮判过），这里只量甲。

## 打中之后的四句

| 打中 | 分不分辨臂 | 系统当时看不看得到判别它的东西 | 满足判据字面的哪一分句 | 改法在打中的格上还中不中 |
|---|---|---|---|---|
| 一·2 那条测试的世界在甲下写不出 | 分辨：只有把 `units` 换成按角色当键的表（甲）才碰它 | 看得到：世界是测试自己造的 | 判据格「次序与下标」：「存下标的读者换成什么；换完哪些测试的期望会动」 | 「删测试、改用 `lookup_work` 计数守准入不逐项找」起作用（推的）；「准入改 `get(角色)`」碰不到这一格，另带 O(log U) 那一问 |
| 一·3 正推第一版每次 `from_sorted` 现建 | 分辨：只打丁（甲的第一版实现） | 看得到：块指针是否相同就是判别子 | 判据格「容器」的「自己写的行数与验法」与正文问的「哪一种发布让它退回整份拷贝」 | 「块界按键、只改碰到的块」起作用，但它就是正推的第 5 步（它自己写推不出）；「块界按键、仍从 `Vec` 现建」不起作用 |
| 一·3 按块派生的相等会假红 | 分辨：只打块界不定的实现 | 看得到：两边内容相同，测试红 | 「版本相等与换回」 | 手写按内容比起作用（零轮） |
| 三·1 克隆不到常数 | 不分辨：三条候选都不碰分配器的这几样，照 `.claude/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种` 记成「打中不分辨臂」，并进第一轮另立的那一笔 | 看得到：字段就在 `PoolAllocator` 上 | 背景材料「问题」第 3 条第一问「是不是自然变成常数代价」 | 「三样也做持久或挂载时包 `Arc`」起作用（推的）；只换记录表与空闲图不起作用 |

T1–T5 五条是跑前写死的翻面线，结局待主 agent 跑，不在这张表里。

## 我提的改法（只在读码上推过、被攻过零轮；没有一条在模型上量过）

| 改法 | 修哪一格 | 标 |
|---|---|---|
| 整理那一支按搬迁项逐个改「角色 → 指针所在处」，不加反查索引；测试构建里与整版走一遍互核 | 一·1 | 推的 |
| 准入那两个根单元在版本里各留一份 `PublishedUnit`，不留下标、不走 `get` | 一·2 | 推的 |
| `container_tree_root_units_read_at_their_recorded_positions.rs` 与它的变异行随甲删，「准入不逐项找」改用计数守 | 一·2 | 推的 |
| 分块表块界按键、发布只改碰到的块；第一版不做 `from_sorted` 现建 | 一·3 | 推的；收益多少待主 agent 跑 T1、T4 |
| 持久表手写按内容的 `PartialEq` | 一·3 | 推的 |
| 聚簇段集、已回收集做持久，被抛弃根的落点挂载时包 `Arc` | 三·1 | 推的；值不值待主 agent 跑 T5 |
| 分配器克隆后在拷贝上改、断言原件逐字段不变（测试构建） | 三·1 | 推的 |

## 要主 agent 跑的命令

1. 探针那一趟（副本里编、跑装置）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r2-opus/copy -- cargo run --release -p singlefs-background-gpu-tests --bin e202_gpu_cpu_gc_comparison -- construction-timing --unit-area 2288MiB --live-data-units 4096 --chunks shared --product /home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r2-opus/publish-change-probe-2288MiB-P4096.out`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r2-opus/copy`（副本，`.deliverables` 写成「交接」）。
   - 看哪几行：末行 `TASK_SERVER_RESULT` 的 `exit=0`；产物里有 `name=publish_change_probe_row` 与 `name=publish_change_probe_phase` 行、有 `name=construction_timing` 一行。这份探针我没编过（子 agent 不交任务），`exit=101` 多半是编译错，`task-server/task log <号> --all` 里 `error[` 那几行指到 `publish_change_probe.rs`、`allocator.rs` 末尾、`transaction.rs:13213` 一带或装置的 `probe_lines_of_the_phase`，新派攻方腿承接修。
2. 汇总与判翻面线（等第 1 条退 0 再交）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r2-opus -- python3 /home/fy5090/code/singlefs/research/prompts/m4-publish-without-whole-version-copy-design-r2-opus-model/tools/summarize_publish_change_probe.py /home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r2-opus/publish-change-probe-2288MiB-P4096.out`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r2-opus`（草稿目录）。
   - 看哪几行：`exit=0` 且有「✓ 两条路的分布逐格对上、完整性闸过」算这一趟有效；`exit=1` 看 `✗` 行（分布对不上或完整性闸不过，这一趟作废）。有效时读三段各三张表的 `name=distribution` 行、`name=roles`、`name=allocator` 行与五条 `name=flip_reading … flipped=yes|no`，照第二·2 节的表判。
3. 从干净运行目录整份复跑（判决要引探针的数时用这一条，主 agent 复跑）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r2-opus -- bash /home/fy5090/code/singlefs/research/prompts/m4-publish-without-whole-version-copy-design-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r2-opus-run /home/fy5090/code/singlefs/research/prompts/m4-publish-without-whole-version-copy-design-r2-opus-model`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r2-opus`（草稿目录）。第二个参数是冻结副本：给仓根时复跑的是交任务那一刻的主工作区；判决要对开工那一刻，换成 `snapshot-tree.py` 倒推出的树。
   - 看哪几行：末行 `TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号> --all` 里 `name=rerun_probe_exit exit=0`、`name=apparatus_seconds seconds=<秒>`、「✓ 两条路的分布逐格对上、完整性闸过」与五条 `name=flip_reading`。`exit` 非 0 时运行目录里 `patch.log`（探针打不上：主工作区的四份文件在我建副本之后被改过）、`apparatus.log`（编译或装置）、`summary.out` 分别看。运行目录用完由主 agent 删。

## 没打中的形状

- 反查索引与审计独立：试了「这张表会不会被 checker 或验证包当输入读」——池级 checker 不读版本（第一轮判决），验证包读的是版本的字段，不读写者内部的按 key 索引；没打中。
- 反查索引随全版涨：按 key 找角色那一份是映射条目表按 key 排的同一张，没有另一份；没打中。
- 二层分块析构递归：试了持久 B 树深度（4 层）、冻结嵌冻结（被 `refuse_while_a_publish_is_frozen` 先拒）、版本列表（`Vec`，逐个析构）；没打中。
- 冻结「发布之后那一份」在持久共享下：`std::mem::replace` 挪动、不拷贝，两份共享没改的块；没打中。
- 六种没有撤销步的改动在持久共享下：不要撤销步；没打中（换成「每个被改字段都得进持久结构」这一前提，第三·1 节）。
- 测试与装置里依赖分配记录先后的读者：145 行逐行看，0 行；`.units` 里 1 行（那条测试的前提）。
- 「同一角色多项」在代码里有没有合法来源：读了 `units` 的装配（`transaction.rs:17032`–`:17248`），每族按树的节点各 `push` 一次，没找到同一角色进两次的分支；但 `FirstEntryOfEachIdentity` 的「取最先那一项」说明写的人没排除它，所以交探针在 E202 上数（T3），代码上推不出。
- `ContainerTreeRootUnitPositions` 之外存 `units` 下标、跨调用留着的读者：在 `crates/*/src` 里按 `FirstEntryOfEachIdentity::of_the_table`、`SortedIdentityIndex`、`.units[` 搜过，命中的索引都是现建现用、不存进版本；存进 `TransactionOutput` 的下标只有它一个（只核到这几种写法）。

## 撤回了什么

- 第一轮我给的「…的反向索引，跟着每次发布维护…」（`research/prompts/m4-publish-without-whole-version-copy-design-r1-opus-output.md:67`）：撤。搬迁计划已带角色，按 key 找角色的索引今天已有（每批整建一次），缺的只是一张写死的「角色 → 指针所在处」表（一·1）。同一行里「这份索引今天没有」也撤：`mapped_units_by_key`（`transaction.rs:4554`）就是它的 key → 角色那一半。
- 第一轮我的模型里那一条式子行 `relocation_with_a_key_to_pointer_index`（读 16）随之没有依据：它算的是这份不需要的索引。
- 第一轮判决第三节记的零轮改法「甲在带搬迁的发布上要另加映射 key 到指针的反查索引（攻方）」（`research/prompts/m4-publish-without-whole-version-copy-design-r1-main-verification.md:51`）：建议判决改成「整理那一支按搬迁项改、与整版走一遍互核」。

## 这条腿自己的限度

- 探针没编、没跑：报告里所有关于 d 的数都还没有；T1–T5 只有翻面线、没有读数。我只做了 `cargo fmt`、`patch --dry-run`、`python3 -m py_compile`。
- 探针只在一个点上取样（2288MiB、P=4096、shared，造池；不含正式跑）。块数口径是「按上一版键次序每 64 / 256 条一块」，近似块界按键的持久表；持久 B 树的叶界随分裂移动，碰到的叶数与这个数同阶但不相等。`units` 的键用的是 `TransactionUnit` 的派生次序，不是将来写的 `UnitRank`；这影响「碰到几块」，不影响加删改的条数与换名、翻转、重复三样。
- 读者分类是按上下文读一行的主用法分的，一行只记一类；「遍历」里少数行另有 `{:?}` 打印，没单列。分类我自己做了一遍，没有第二条独立的路核。
- 一·2 里「O(log) 在准入路径上算不算随盘容量增长」我不判。
- 第三·1 节的字段表是读 `PoolAllocator` 与 `DeviceFreeMap` 的定义列的，各字段在 E202 上多大全靠探针；被抛弃根那一样在 E202 造池里推的是 0。
- 正推第一轮的「丁」我只攻了正文点名的三问；它的行数与验法归辩方。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；副本上量出的数（等第 1、2 条）不算入库装置上的数；没往统一任务服务交任务，判决要引的数没复跑（命令在「要主 agent 跑的命令」一节）。
- 没改主工作区的 `crates/`；没读禁读清单里的文件。
- 没删 /home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r2-opus/copy：主 agent 要在上面跑命令（第 1 条），已写进草稿目录 `.deliverables`，种类「交接」。用完由主 agent 删。
- 草稿目录里另有我留的原件备份 `originals/`（四份被改文件改前的样子）、`probe.diff`、读者分类的中间文件与日志，没有编译目录。
