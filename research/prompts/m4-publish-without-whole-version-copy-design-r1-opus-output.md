# m4-publish-without-whole-version-copy-design-r1 攻方腿报告

日期：2026-11-18。设计推论轮第一轮，攻方腿；前几轮判决：无。攻击面三样：过度外推与越位 / 内部矛盾与循环依赖 / 哪一条最脆弱。
行号全部是主工作区此刻的树（`crates/singlefs-core/src/mount.rs` 在工作区里有没提交的改动，行号按此刻的文件）。没改 `crates/`。

## 复跑命令与文件指纹

模型目录 `research/prompts/m4-publish-without-whole-version-copy-design-r1-opus-model/`（纯算术 Python，只用标准库、不读仓）：

```
4b1caf9ce35e0dd0ca568424b0062dffbf932f0f2b49dc37ebbd1b9d00b02e04  version_representation_counting_model.py
c5f38711fa6e8a425e668217572fe3d30e58c095e4a10982da8f54d6a0659efd  tools/rerun.sh
e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b  tools/rerun-from-a-clean-work-directory.sh
```

复跑（整条在「要主 agent 跑的命令」一节）：`tools/rerun-from-a-clean-work-directory.sh <仓根> <冻结副本> <运行目录> <模型目录>`。
`--frozen` 填本轮快照目录 `research/prompts/m4-publish-without-whole-version-copy-design-r1-snapshot`（模型不读它，只为 rerun.sh 的参数检查；rerun.sh 拷它进运行目录，两份小文件）。
探针的内存与时间：没跑过（子 agent 不交任务）；推的：模拟里最大的是 4096 次长 4096 的列表拷贝与丙 4096 个键各沿链找（合计约 1.65×10⁷ 步 Python 循环），估几十秒、内存百 MB 以内。
模型里的数全部「推的，待主 agent 跑」：我只做了 `python3 -m py_compile`，没跑过。

## 各格判定一览

| 攻击面 | 候选 | 判定 | 一句话 |
|---|---|---|---|
| 一 过度外推 | 甲 | 打中（不分辨臂的那一半另记） | 整理一批与「从盘上重建一版」都走 `take_the_mapping_positions_over_the_location_hints`，按全版每一处指针走一遍并 `&mut` 改；甲下照现写法要么读满 N、要么把持久树的每个节点都拷一遍；分配记录的源头是分配器自己的 `Vec`，甲只换 `TransactionOutput` 时 `allocation_records_sorted_copy` 那一段不动 |
| 一 过度外推 | 乙 | 打中 | 一次用户改动交回几版（抬 F 的空发布、走空的步、改动自己、另发的一批），验收在调用返回之后逐版读全版分配记录与全版内容；乙的全版只剩最新那一份，前几版要么拷满 N 交出来、要么带撤销记录倒推（模型：三版 12291 条） |
| 一 过度外推 | 丙 | 打中 | 按键随机找沿链走：冷键在 P 次发布之后走 P 步（模型 4096），全部键合计 16517088 步；整理一批读全版要沿链合出 N₀+P 条；只留最新一版时整条链仍被拽着，内存随发布次数涨 |
| 一 过度外推 | 三条共有 | 打中，不分辨臂 | 结算固定点每轮 `allocator.clone()` 与 `nodes_whose_contents_changed(全上一版, 全演练)`、发布前整份克隆分配器，读写的都是分配器的 `records: Vec`，不在 `TransactionOutput` 里；三条候选只换版本表示，这几段照旧与全版条数成正比 |
| 一 过度外推 | 三条 | 没打中 | 取号全或无失败的回卷写在任何单元之前，不碰版本 |
| 一 过度外推 | 背景材料 | 打中（材料越位） | 「崩溃枚举留每一版」：崩溃点重放与崩溃枚举的版本表是 `PublishedVersion { content: Vec<u8> }` 与按写表下标的 `PublishWrites`，不留 `TransactionOutput` |
| 二 内部矛盾 | 背景材料「压着这件事的条款」第四条 | 打中（引文归错） | 「审计与被审计不共用一段代码」在 `.claude/rules/verification.md` 里没有这句；原文在 `.claude/rules/fs-design.md` 第 22 行 |
| 二 内部矛盾 | 池级 checker | 没打中 | checker 只依赖 `singlefs-format`，不读 `TransactionOutput`，三条候选都碰不到它 |
| 二 内部矛盾 | 崩溃点重放 | 没打中 | 读的是录制写流分出的发布表，不读版本 |
| 二 内部矛盾 | 乙 / 模型对拍与验收 | 打中 | 乙下「现行那一版不动」那几条断言比的是差量，写者的可变索引被一次被拒的发布改坏时照判相等（模型：版本相等 1、全版相等 0）；验收的分配代判定从「两份全版自己比」变成「读写者自报的差量」或「读写者的索引」 |
| 二 内部矛盾 | 乙 / 冻结发布 | 打中 | `FrozenPublish` 同时拿着发布之后那一版与发布之后的分配器，冻结期间调用方手里是发布之前的；乙只有一份可变索引，两份全态并存要拷满或另起一套暂存差量 |
| 二 内部矛盾 | 甲 / 两条线并跑 | 打中（循环依赖） | 不动形态那一批在改「发布前克隆」；甲要把分配记录省下来就得把分配器的 `records` 也换成持久结构，那时发布前克隆自然变 O(1)；两条线改同一处、次序不定 |
| 二 内部矛盾 | 一个事务层 | 没打中 | 三条候选都只动内存表示，不开第二条发布路径 |
| 三 最脆弱 | 乙 | 判定：最脆弱 | 卡在「版本是值」这一约定上：`VersionsPublishedInThisChange`、`RaisedFloor::publishes`、`CompactionRun::versions_published`、`FrozenPublish::version`、管理员回退拼出的混合版都要多版全态并存 |
| 三 最脆弱 | 丙 | 次脆弱 | 卡在按位置与按身份找（`ContainerTreeRootUnitPositions` 存的是 `units` 下标、每次发布建 `units_by_role`、整理建映射索引），链长不受控 |
| 三 最脆弱 | 甲 | 最不脆弱，但有两处做不下去的点 | 现写法里「全版 `&mut` 走一遍」的几处（指针换映射位置）与按 `Vec` 下标记的位置；外部包与自写各有代价（`crates/` 现无持久容器） |
| 另 | 「整理一批两次整版深拷贝」 | 核了，成立，且不止两次 | `mounted_session.rs` 5369–5371 拷上一版、5610 / 5727 / 6022 拷发出来的那一版；带搬迁的发布在 `transaction.rs:15822` 再拷一次上一版 |
| 另 | 「`crates/` 里没有持久共享的容器」 | 核了，成立 | `grep -rn 'im::\|rpds\|persistent' crates/*/src` 17 行，全在 `e158_root_choice_repair.rs`、说的是故障「持续」 |

## 攻击面一：过度外推与越位

背景材料的目标句是「一次发布的代价只与这次改到的东西有关」。逐条路看它对哪条候选不成立。

### 一·1 三条共有、不分辨臂：结算与发布前克隆读写的是分配器的 `Vec`，不是版本

- 分配器自己装着全部分配记录：「records: Vec<AllocationRecord>,」（`crates/singlefs-core/src/allocator.rs:2294`），对外交切片「pub fn records(&self) -> &[AllocationRecord] {」（`crates/singlefs-core/src/allocator.rs:2743`）。
- 版本里的那一份是每次发布从它整份拷出来再排序：「let mut allocation_records: Vec<AllocationRecord> = allocator.records().to_vec();」（`crates/singlefs-core/src/transaction.rs:16232`）。这就是分段里的 `admitted.allocation_records_sorted_copy`。
- 结算固定点每一轮整份克隆分配器演练释放：「let mut rehearsal = allocator.clone();」（`crates/singlefs-core/src/transaction.rs:14076`），再拿全版对全版找变了的叶：「nodes_whose_contents_changed(&geometry, previous_records, rehearsal.records());」（`crates/singlefs-core/src/transaction.rs:14085`）。
- 发布前整份克隆、失败换回：「let allocator_before_this_publish = allocator.clone();」（`crates/singlefs-core/src/transaction.rs:13213`）。

反例（一串发布）：任何一次用户覆写。甲、乙、丙三条只改 `TransactionOutput` 的表示，上面四处一个都不碰，仍是每次发布 O(N)（结算再乘轮数）。
所以「换表示之后一次发布的代价只与改到的有关」在只改版本表示的前提下对三条都不成立。这一格打中的是三条共用的前提（射程只划到 `TransactionOutput`），照 `.claude/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」记成「打中不分辨臂」，不拿它判臂。
它对甲、乙的含义不同（推的）：乙的「写者持有的可变索引」就是分配器这份 `records`，乙天然把两者合一；甲要省掉这份拷贝，得把分配器的 `records` 也换成同一种持久结构，射程越出 `TransactionOutput`、进 `allocator.rs`（`.records()` 交切片的调用 164 处，`grep -rnP '\.records\(\)' crates/*/src crates/*/tests | wc -l` 的数，不全是分配器的）。

### 一·2 甲：整理一批与恢复重建走「全版每一处指针 `&mut` 走一遍」

带搬迁的发布先整版拷上一版：「let mut relocated_version = previous_version.clone();」（`crates/singlefs-core/src/transaction.rs:15822`），再调 `take_the_mapping_positions_over_the_location_hints`，它对一版里每一处指针都走一遍、可写地改：「for data_pointer in &mut version.data_pointers {」（`crates/singlefs-core/src/transaction.rs:4006`），后面同样 `&mut` 走 extent 树上下段、每片 inode 叶容器、分配记录树、记账树、deadlist 树、三棵容器树与树表，最后按建好的索引改 `units`：「version.units[*position].slot = slot;」（`crates/singlefs-core/src/transaction.rs:4107`）。
恢复重建一版也调它（`crates/singlefs-core/src/recovery.rs:3832`）。

反例（一批整理）：一批搬 16 个冷键。甲下照现写法：
- 拷上一版变 O(1)，但「全版 `&mut` 走一遍」在持久树上等于把这一版每个节点都拷一份（模型 `relocation_walk_jia_mutating_every_node nodes_copied` 断言 273，等于整棵树的节点数）；
- 改成先只读走一遍、只对搬了的键拷路：读满 N₀ = 4096 条，拷 4 个节点（`relocation_walk_jia_targeted`）；
- 要真到 O(批大小)，得另有一份「映射 key → 哪一处指针」的反向索引，跟着每次发布维护（模型 `relocation_with_a_key_to_pointer_index` 读 16）。这份索引今天没有，`take_the_mapping_positions_over_the_location_hints` 的文档说指针按类型现算 key、没有反查。
乙、丙在同一处同样要读满（乙读索引 4096，丙沿链合出 8192），所以「读满」这一半不分辨臂；「`&mut` 走全版把持久树整棵拷一遍」这一半只打甲（照现写法直接换容器时），分辨臂。
整理归 `.claude/rules/fs-design.md` 的第三格：「| **后台、可续做、非决策路径**（销毁快照、整理、scrub） | 无戒律 |」（`.claude/rules/fs-design.md:18`），规则不禁它读满；打中的是背景材料「一次发布的代价只与这次改到的东西有关」对整理那一种发布不成立，E202 测的正是整理。

### 一·3 乙：一次调用交回几版，读者在调用返回之后逐版读全版

- 产品 API 一次交回几版：「pub versions_published: Vec<TransactionOutput>,」（`crates/singlefs-core/src/mounted_session.rs:377`，`CompactionRun`），「pub publishes: Vec<TransactionOutput>,」（`crates/singlefs-core/src/mount.rs:4371`，`RaisedFloor`），另有 `VersionsPublishedInThisChange`、`DeletionQueueWalk::steps_published`。
- 验收在用户改动返回之后逐版走：「let outputs: Vec<&singlefs_core::transaction::TransactionOutput> = match published {」（`crates/singlefs-verification/src/history.rs:6042`），每一版拿全版分配记录与前一版的全版比分配代，「records_of_the_previous_publish = &output.allocation_records;」（`crates/singlefs-verification/src/history.rs:6097`）；比法是两份全表自己找变了的：「let records = records_changed_with_a_generation_outside_the_publish_txgs(」（`crates/singlefs-verification/src/history.rs:5074`）。
- 模型对拍对每一版读全版内容：「pub fn observed_root_of_file_version(output: &TransactionOutput) -> ObservedRoot {」（`crates/singlefs-verification/src/model_comparison.rs:181`），里面按全部 `units` 与全部分配记录配角色：「allocation_records_of_every_role(」（`crates/singlefs-verification/src/model_comparison.rs:189`）。

反例（一次用户改动）：先走空一步、再推一次抬 F、再发改动自己，一次调用交回三版。乙下调用返回时写者的索引只对得上最后一版；前两版的全版要么在发布当时拷满交出来（每版 N，代价退回与全版成正比），要么差量里带旧值、事后从索引拷一份倒推（模型 `change_of_three_versions_yi` 12291 条，且要求差量多带一份撤销记录）。
甲下三版各留一个根，读全版仍是 3N，但两版之间按共用节点比只要 33 对节点（模型 `change_of_three_versions_jia`）；丙下三次沿链合出合计 24582 条。

### 一·4 丙：按槽、按 key 随机找沿链走

丙的读法是「读全版时沿链合出」；按身份找一个没改过的单元要往回走到它最后被改的那一版。
- 发布里按身份找的地方今天是每次建一次全表索引：「FirstEntryOfEachIdentity::of_the_table(&self.units, |unit| unit.identity)」（`crates/singlefs-core/src/transaction.rs:4540`），整理每读一版建一次映射索引：「CentralMappingIdentityIndex::of_the_mapping_entries(&read_of_the_version.mapping_entries);」（`crates/singlefs-core/src/mounted_session.rs:5396`）。
- 反例（P 次只改热键的发布之后找冷键）：模型 `bing_lookup_cold_key hops` 断言 4096（= P），全部 4096 个键各找一次合计 16517088 步；建一次全表索引要先合出全版（`bing_materialize_latest` 8192 条、4097 版）。
- 只留最新一版时，丙的最新一版仍拽着整条链：模型 `retain_latest_only_bing` 8192 条，随发布次数线性涨；甲只留 273 个节点（`retain_latest_only_jia`）。要止住得定期压平（压平那一次是 O(N)），这是丙自己要加的一条路，背景材料没写。

### 一·5 别的几种发布与读路径

- 快照留着的映射条目：每次发布按 resolve 交回的那一份整份重拼，「let kept_mapping_entries_after_this_publish: Vec<(Vec<u8>, [LocationEntry; 2])> = resolved」（`crates/singlefs-core/src/transaction.rs:15808`）。条数是被快照钉住的单元数（不是全版条数），但它在 resolve 那一侧就是整份，三条候选都只接收它、不改 resolve，记「不分辨臂」，不算打中某条。
- 发布失败换回：版本这一侧今天本来就不用换回（新版本另建，失败时上一版没被动过）；要换回的是分配器（一·1）。甲：留旧根，0 拷贝；丙：新差量不挂上链，0 拷贝；乙：索引就是分配器的 `records`，要么照旧整份克隆（4096），要么加撤销记录（1 条）。模型 `failure_swap_back_*` 五行是式子行，不是模拟出的。
- 取号全或无失败的回卷：「/// 取号全或无失败：已写出的那几块盘回卷成旧代号（D18（块里携带什么信息） 已定项 11），回卷写发生在任何单元之前。」（`crates/singlefs-core/src/transaction.rs:770`）。回卷只写系统配置、在任何单元与版本之前，三条候选都碰不到，没打中。
- 「崩溃枚举留每一版」（背景材料判据格「内存」一行的说法，转述）：崩溃点重放的版本表是「pub content: Vec<u8>,」（`crates/singlefs-verification/src/memory_pool.rs:889`，`PublishedVersion` 只留文件内容），分出的发布是写表下标「pub units: Vec<usize>,」（`crates/singlefs-verification/src/memory_pool.rs:1038`），`crash.rs` 与 `crash_judge_tables.rs` 读的 `publish.units` 都是它（「for unit in &publish.units {」，`crates/singlefs-verification/src/crash.rs:140`、`crates/singlefs-verification/src/crash_judge_tables.rs:1754`）。崩溃枚举不留 `TransactionOutput`，「留每一版时内存怎么涨」对崩溃枚举不是这几条候选的代价；真留着多版 `TransactionOutput` 的是模型对拍与验收拿到的版本列表（一·3）。这一条打的是背景材料的越位，不分辨臂。
- 背景材料没列的整版拷贝（核出来的，供正推腿与判决对表）：`Arc` 共享着时的隐式深拷贝——「versions_published.extend(walk.steps_published.into_iter().map(Arc::unwrap_or_clone));」（`crates/singlefs-core/src/mounted_session.rs:4644`）、被拒推抬 F 时「Arc::make_mut(current),」（`crates/singlefs-core/src/mounted_session.rs:6332`）、换写账时「Arc::make_mut(version).writes = writes;」（`crates/singlefs-core/src/transaction.rs:2685`）；这三处在版本列表还指着同一份时各整版拷一次。甲、丙下这种拷贝变成拷几个根（推的），乙下拷的只是差量、但拷出来的东西不再是全版。

## 攻击面二：内部矛盾与循环依赖

### 二·1 背景材料第五条的引文归错了文件

背景材料「压着这件事的条款」第四条把「审计与被审计不共用一段代码」挂在 `.claude/rules/verification.md`「checker 即规范」与「功能正确性靠模型对拍」两节下（转述）。`grep -c '审计与被审计' .claude/rules/verification.md` 输出 0。原文在另一份：「审计与被审计用同一段代码。**运行时与 checker 必须用不同的算法。**」（`.claude/rules/fs-design.md:22`）。
`verification.md` 那两节自己说的是「在内存里维护一个只管语义、不管性能也不管崩溃的理想文件系统」（`.claude/rules/verification.md:86`）。判决引这一条时按 `fs-design.md` 的原句判，不按背景材料的括注判。

### 二·2 审计独立：逐个读者看

| 读者 | 读不读 `TransactionOutput` | 三条候选下 |
|---|---|---|
| 池级 checker | 不读：依赖只有「singlefs-format = { path = "../singlefs-format" }」（`crates/singlefs-checker/Cargo.toml:9`） | 碰不到，没打中 |
| 崩溃点重放、记录核对器、GPU 判器的发布表 | 不读：读录制写流分出的 `PublishWrites`（一·5） | 碰不到，没打中 |
| 模型对拍（`model_comparison.rs`） | 读：每一版的全部 `units`、全部分配记录、inode 叶容器 | 甲：照读全版，没打中；乙、丙：见下 |
| 验收的分配代判定（`history.rs`） | 读：相邻两版各自的全版分配记录，自己找变了的 | 甲：照读，没打中；乙、丙：见下 |

乙、丙下模型对拍与验收拿全版只有两条路：读写者的索引（乙）/沿链合出（丙），或者读每一版自报的差量。
- 读差量：「哪几条变了」由被审计的发布代码交出来，验收只核这几条。今天验收拿两份全表自己比（一·3 引的 `history.rs:5074`），换成读差量就是让被审计那一段代码替审计算出审计的输入，正是 `fs-design.md:22` 禁的那一件；一次发布改了一条记录而差量没列它时，读差量的验收看不见。打中乙、丙（丙的差量就是唯一真相，「改了没列」写不出来，所以这一形对丙较弱：丙下漏列等于没改，盘上那一份由同一份差量装出，靠池级 checker 读盘兜；乙下索引与差量是两份，能分叉）。
- 读写者的索引（乙）：被审计的数据本来就是读者要观测的东西，不算共用代码；但索引可变，读到的是此刻、不是「那一版」（一·3），而「现行那一版不动」这类断言比的版本值只剩差量：「assert_eq!(pool.output, current_before, "{breakage:?}");」（`crates/singlefs-verification/tests/rollback_by_a_forward_publish.rs:429`）。一次被拒的发布把索引改坏而没留差量时，这一句照判相等（模型 `yi_refused_publish_bug version_values_equal=1`、`full_contents_equal=0`）。今天这几条测试另判了分配器的记录，乙下装着单元、映射的那部分索引没有对应的另一判，得逐条补；`assert_eq!(…output…, …before…)` 这一类在 `crates/singlefs-verification/tests` 里按我的正则数到 15 行（命令在「没打中的形状」末尾），没逐条核每一行比的是不是版本。
这一格的「判别子」今天看得到（版本是不可变的值，一次被拒的发布改不了它），乙下看不到（版本值里不含全态），对应 `evidence-discipline.md`「判别子观测不到」一形，落在乙。

### 二·3 乙与冻结发布：两份全态要并存

「allocator_after_the_publish: PoolAllocator,」（`crates/singlefs-core/src/transaction.rs:1583`）与同一结构里的 `version: PoolVersion,`（`crates/singlefs-core/src/transaction.rs:1580`）：落盘途中失败的那次发布冻结着，调用方手里是发布之前的分配器与现行那一版，重发成功时整个换成冻结着的那一份。乙只有一份写者持有的可变索引，冻结期间「之前」「之后」两份全态都得在：要么冻结时整份拷一份（O(N)，只在失败路径），要么另起一套「暂存差量、重发时再落进索引」的机制。后一种是又一条内存里的提交路径，与 `FrozenPublish` 换分配器那一套并存（推的：不违反 `fs-design.md`「一个事务层，所有结构共用」，那一条管的是盘上的发布路径，原文「绕开状态机的第二条发布路径不允许」，`.claude/rules/fs-design.md:54`；但多一处要验的状态机）。

### 二·4 候选之间、候选与另一条线之间的依赖

- 丙要靠甲或乙才站得住（推的，由一·4 的数推出）：丙的按身份找沿链走、链长不受控；要把找一次压到对数，丙得在每一版上另挂一份按身份的索引，这份索引要么是持久共享的（就是甲），要么是写者持有的可变的（就是乙）。丙自己单独做不到「读全版的代价」那一格，三条候选不是互斥的三选一，而是丙 = 甲或乙 + 一条差量链。
- 乙的冻结发布要一套暂存差量（二·3），那一套就是丙的「差量挂在上一版后面」，乙反过来要借丙的一半。
- 甲与不动形态那一批的发布前克隆：背景材料说不动形态的第二批在改发布前克隆（转述「实现今天的样子」第四条）。甲要省掉分配记录的整份拷贝就得把分配器的 `records` 换成持久结构（一·1），那时发布前克隆整份分配器自己变成拷几个根；第二批若先用别的办法（撤销记录、只拷位图）改了发布前克隆，甲落地时那一段要再改一遍或撤掉。两条线改同一处、谁先谁后背景材料没定；判决要定次序或把分配器的 `records` 划进这一题。
- 甲与相等：`TransactionOutput` 今天是 `#[derive(Clone, Debug, PartialEq, Eq)]`（`crates/singlefs-core/src/transaction.rs:3374`），逐字段比 `Vec`。自己写的持久树若形状随插入历史变（分裂点不同），照抄 `derive(PartialEq)` 就成了比形状：从盘上重建的一版与发布出来的一版内容相同、形状不同时判不等。读者里有这一形：「assert_eq!(rebuilt.mapping_keys, output.mapping_keys);」（`crates/singlefs-verification/tests/common_images_for_the_given_states_engine/accounting_and_mapping_tree_split.rs:188`）。这是推的风险（我的模型是定形的树，量不到）；约束是甲的相等必须按内容比，没有共用节点的两版比满全部节点（模型 `jia_equal_without_shared_nodes` 273 对，与今天 O(N) 同阶）。

### 二·5 没打中的

- 「一个事务层，所有结构共用」：三条都只动内存表示，不碰盘上的发布路径与提交步骤，没打中（乙的暂存差量见二·3，算推的边角）。
- 「类型：让非法状态写不出来」：乙下「版本」这个类型装不下全版，`TransactionOutput` 那十几个「这一版的全部…」字段的文档说法在乙下全不成立；这是要改类型名与文档，不是写得出非法状态，记推不出。

## 攻击面三：哪一条最脆弱

### 三·1 读者逐个看（挑承重的，不抽样代替全称）

`.units` 386 行里：137 行在 `crates/singlefs-unit-graph`（那里的 `units` 是另一个东西，`grep -rnP '\.units\b' crates/singlefs-unit-graph | wc -l` 的数），16 行是 `.units()` 方法调用（`grep -rnP '\.units\(\)' crates/*/src crates/*/tests | wc -l`）。下表是我逐行看过的 `src` 读者（core、checker 全部，verification 里背景材料点名的几份），测试里的 `.units` 没逐行分类。

| 读者（文件:行） | 读什么 | 甲 | 乙 | 丙 |
|---|---|---|---|---|
| `crates/singlefs-core/src/transaction.rs:17239` | 照抄上一版全部 `units` 进新一版（`units_assembly`） | 变成共用节点，消掉 | 不再照抄，消掉 | 不再照抄，消掉 |
| `crates/singlefs-core/src/transaction.rs:4540` | 每次发布按全部 `units` 建身份索引 | 结构本身按身份，消掉（若 `units` 存成按身份的持久映射） | 索引就是写者的，消掉 | 先合出全版，O(N+P) |
| `crates/singlefs-core/src/transaction.rs:3490`、`:4519` | `ContainerTreeRootUnitPositions` 记的是 `units` 的下标，「之后不再挪动」 | 按身份的映射没有下标，改成按身份取；按位置的持久向量能留下标但要另一份身份索引 | 差量里没有没重写的根单元，改成查索引 | 同乙，查一次沿链走 |
| `crates/singlefs-core/src/transaction.rs:4006`–`:4107` | `take_the_mapping_positions_over_the_location_hints` 全版 `&mut` 走 | 照现写法整棵拷一遍（一·2），要加反向索引 | 走索引 N | 合出全版 N+P |
| `crates/singlefs-core/src/mount.rs:1335` | 上一版的全部 `units` 交给装多层码 2 树照抄节点 | 交根，按身份取 | 交索引 | 沿链取 |
| `crates/singlefs-core/src/mount.rs:10805`–`:10806` | 管理员回退把现行那一版的映射 key、分配记录整份拷进拼出来的那一版 | 拷根 | 要现行（索引）与目标（`mount.rs:10973` 从盘上重建的全版）两份全态并存 | 目标那一版没有上一版可挂，是一份全版 |
| `crates/singlefs-core/src/mounted_session.rs:4939` | 整理读一版：全部 `units` 找钉住的槽 | 读满 N（整理无戒律） | 读满 | 合出全版 |
| `crates/singlefs-core/src/mounted_session.rs:5369`–`:5371`、`:5610`、`:5727`、`:6022` | 整理拷上一版、拷发出来的一版 | 拷根 | 拷差量，但拷出来的不是全版 | 拷差量与引用 |
| `crates/singlefs-verification/src/model_comparison.rs:181`–`:189` | 每一版全部 `units` 与全部分配记录 | 照读 | 只有最新一版读得到（一·3） | 合出 |
| `crates/singlefs-verification/src/history.rs:6042`–`:6097` | 一次改动交回的每一版的全版分配记录 | 照读，可按共用节点比 | 同上 | 合出 |
| `crates/singlefs-checker/src/walk.rs:18835`、`:18959` | `snapshot.units`，checker 自己读盘建的结构 | 不碰 | 不碰 | 不碰 |
| `crates/singlefs-verification/src/crash.rs:140`、`crash_judge_tables.rs:1754`、`memory_pool.rs:912` | `PublishWrites::units`（写表下标） | 不碰 | 不碰 | 不碰 |
| `crates/singlefs-verification/src/crash_verify_gpu.rs:495`、`:496`、`:532`、`:777` | GPU 核对器自己的事实表（`facts.units`、`self.units`） | 不碰 | 不碰 | 不碰 |

`.mapping_keys` 27 行：`TransactionOutput::mapping_keys` 在产品代码里只有两处，一处装（「mapping_keys: mapping_entries.mapping_keys,」，`crates/singlefs-core/src/recovery.rs:3787`），一处管理员回退照抄（「mapping_keys: current.mapping_keys.clone(),」，`crates/singlefs-core/src/mount.rs:10805`）；`src` 里另外 6 行是别的类型的同名字段（`units_read_with_one_copy_failing.mapping_keys` 这一类），测试 19 行（`grep -rnP '\.mapping_keys\b' crates/*/tests | wc -l`）多是 `.len()` 与相等。它在产品的决策路径上没有读者（推的：只按这次 grep 的 `\.mapping_keys\b` 与读到的上下文；结构体字面量的简写初始化不在这个正则里）。

`.allocation_records` 206 行：core 与 checker 的 `src` 里 33 行逐行看过（`grep -rnP '\.allocation_records\b' crates/singlefs-core/src crates/singlefs-checker/src`）。承重的是结算（`transaction.rs:14023` 读上一版全版、`:16241` 同）、整理选段与签发（`mounted_session.rs` 第 5059 行与 5400–5667 之间，共 10 行，全读满，整理无戒律）、整理删意图那一判（`transaction.rs:11244`、`:11287` 读发出来那一版的全版判 R 空不空）、抬 F（`mount.rs:5521`）与回退（`mount.rs:10806`）；checker 的 4 行是它自己的 `references.allocation_records`。测试与装置里的没逐行分类。

### 三·2 排序与卡在哪

1. 乙最脆弱，卡在「版本是值」这一约定。今天一版发出来就不再改、可以被几处同时拿着（`PoolVersion` 的文档：「一版发出来之后不再改」，`crates/singlefs-core/src/transaction.rs:2551`），读者靠这一点在调用返回之后逐版读（一·3）、冻结发布拿着之后那一版（二·3）、管理员回退把现行那一版与从盘上重建的目标那一版拼起来（`crates/singlefs-core/src/mount.rs:11126` 「let previous = version_to_publish_the_rollback_after(current, &target_version);」）。乙下这几处都要「多份全态并存」，只能拷满或倒推；恢复重建出来的又一定是一份全版，于是乙得同时维持「全版」与「差量 + 索引」两种版本表示。实现中途最先做不下去的读者推的是 `history.rs` 那一圈逐版判（模型对拍与分配代），它们在一次调用之后才拿到结果，中间没有钩子。
2. 丙次之，卡在按身份、按位置找。`ContainerTreeRootUnitPositions` 存的是 `units` 下标（准入读两个 H 走它，在运行时决策路径上），每次发布按身份建索引、整理按身份建映射索引；丙下这几处都要先合出全版，链长随发布次数涨（一·4）。丙要站住就得挂索引，退化成「甲或乙 + 链」（二·4）。
3. 甲最不脆弱，但有三处做不下去的点：现写法里全版 `&mut` 走一遍的指针换映射位置（一·2，要先改成反查）；`units` 的次序与下标是约定（「按 bump 次序」、下标记进 `ContainerTreeRootUnitPositions`），按身份的持久映射丢次序、按位置的持久向量丢按身份找，要两份结构；容器从哪来——核心包的依赖只有 `libc`、`singlefs-format`、`singlefs-replay-prefix`、`vstd`（`crates/singlefs-core/Cargo.toml` 第 14、15、17、21 行），引外部包要过依赖审，自己写要自带对拍与变异。分配记录要省，射程得进分配器（一·1、二·4）。

## 两句主 agent 标了没核的

1. 「整理一批发两次、每次整版深拷贝」（背景材料「实现今天的样子」第四条，转述）：核了。一批整理只发一次（`advance_compaction_by_one_publish_computing_on` 文档「一次调用至多算 R 里能搬的单元数加三次账、发一次」，被拒时重算计划再发，发成的只一次）；整版深拷贝不止两次：
   - 拷上一版：`crates/singlefs-core/src/mounted_session.rs:5369`–`:5371`（`current.version_of_trees().expect(…).clone()`，`version_of_trees` 交的是 `&TransactionOutput`，`.clone()` 是整版深拷贝）；
   - 拷发出来的一版：「*current = PoolVersion::of_a_version_of_trees(published.clone());」（`crates/singlefs-core/src/mounted_session.rs:5610`，签发那一支；`:5727`，搬迁那一支；`:6022`，停下丢意图那一支），每次发布走其中一支；
   - 带搬迁的发布在发布路径里再拷一次上一版：`crates/singlefs-core/src/transaction.rs:15822`。
   合计一批带搬迁的发布三次整版深拷贝（推的计数：两次在会话那一层、一次在发布路径里；没跑计数器）。kb 那一行写的函数名是「整理每批两次整版克隆（`advance_compaction_by_one_publish`）」（`.claude/kb/milestone/04-fourth-txn.md:180`），拷贝其实在它调的 `advance_compaction_by_one_publish_computing_on`（`crates/singlefs-core/src/mounted_session.rs:5355`）里。
2. 「`crates/` 里没有持久共享的容器」：核了，成立。`grep -rn 'im::\|rpds\|persistent' crates/*/src > <草稿>/persistent-grep.txt 2>&1; echo rc=$?` 输出 `rc=0`，17 行全在 `crates/singlefs-implementation-experiments/src/bin/e158_root_choice_repair/main.rs`，说的是故障「持续 / 瞬时」，不是容器；核心包依赖见三·2 第 3 条。范围只到 `crates/*/src` 与 `crates/singlefs-core/Cargo.toml`。

## 打中之后的四句

| 打中 | 分不分辨臂 | 系统当时看不看得到判别它的东西 | 满足判据字面的哪一分句 | 跑前条款的改法在打中的格上还中不中 |
|---|---|---|---|---|
| 一·1 分配器的 `Vec` | 不分辨（三条一起） | 看得到（分配器在发布路径里） | 「一次发布的代价」格的「写出最坏的那一种发布」：任何一次用户覆写 | 背景材料没有跑前改法；我提的「射程进分配器」只对甲、乙起作用（乙天然合一），对丙不起作用 |
| 一·2 指针换映射位置 | 「读满」不分辨；「整棵拷」只打甲 | 看得到 | 同上，最坏的是整理带搬迁的一批 | 加反向索引对三条都起作用 |
| 一·3 乙多版读 | 分辨（只打乙） | 看得到（版本列表在调用方手里） | 「读全版的代价」「版本相等与换回」两格 | 「差量带撤销记录」能读到，但代价回到每版 N |
| 一·4 丙找键 | 分辨（只打丙） | 看得到 | 「读全版的代价」「内存」两格 | 定期压平对内存起作用，压平那一次 O(N) |
| 二·1 引文归错 | 不涉臂 | — | 背景材料的条款引文 | 改引文，判决按 `fs-design.md:22` 判 |
| 二·2 乙的相等判不出 | 分辨（只打乙；丙较弱） | 今天看得到、乙下看不到 | 「审计独立」「版本相等与换回」两格 | 测试另补「索引不动」一判起作用，得逐条补 |
| 二·3 冻结发布 | 分辨（只打乙） | 看得到 | 「版本相等与换回」 | 冻结时整份拷一份起作用，代价只落在失败路径 |
| 二·4 两条线次序 | 只涉甲 | — | 「分步」格 | 把分配器 `records` 划进本题或定次序 |

照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」，以上全部是第一轮的一次打中，挂起待第二、三轮。

## 我提的改法（只在我的模型上量过、被攻过零轮；模型本身也还没跑）

| 改法 | 修哪一格 | 甲 | 乙 | 丙 | 标 |
|---|---|---|---|---|---|
| 射程划进分配器：分配器的 `records` 与版本里的分配记录用同一份结构 | 一·1 | 起作用（持久结构，发布前克隆随之 O(1)） | 天然就是 | 不起作用（丙没有索引） | 推的，待主 agent 跑 |
| 「映射 key → 指针所在处」反向索引，随发布维护；`take_the_mapping_positions_over_the_location_hints` 改成只改搬了的键 | 一·2 | 起作用（16 次查 + 4 个节点） | 起作用 | 起作用但查索引仍要合出 | 推的，待主 agent 跑（模型只有式子行 `relocation_with_a_key_to_pointer_index`） |
| 结算找变了的叶改成按这次释放与取落点的槽集合算，不再全版对全版 | 一·1 结算那一段 | 起作用 | 起作用 | 起作用 | 推的 |
| `TransactionOutput::mapping_keys` 不再存，要时从中央映射树或 `mapped_units` 现算 | 读者改动面 | 省一份 | 省一份 | 省一份 | 推的；只核到产品代码没有决策读者（三·1） |
| 验收的分配代判定保留「自己拷全表自己比」，不改成读差量 | 二·2 | 本来就是 | 要在每次发布处有钩子拷一份 | 合出两版再比 | 推的 |
| 测试里「现行那一版不动」那一类另补「写者索引不动」一判 | 二·2 | 不需要 | 需要，逐条补 | 不需要 | 推的 |

## 模型说明

`version_representation_counting_model.py` 每行打 `name=<格> <量>=<值> expected=<断言>`，对不上退 1。分两类：
- 模拟出的（真建了结构、真走了一遍）：`publish_*`、`retain_*`、`bing_lookup_*`、`bing_materialize_latest`、`relocation_walk_jia_mutating_every_node`、`relocation_walk_jia_targeted`、`relocation_walk_yi`、`relocation_walk_bing`、`change_of_three_versions_jia`、`change_of_three_versions_bing`、`yi_refused_publish_bug`、`jia_equal_without_shared_nodes`。
- 式子行（值是写下的算式，模型只把它打出来，不算量）：`relocation_walk_whole`、`relocation_walk_jia_read_then_targeted_copy`、`relocation_with_a_key_to_pointer_index`、`change_of_three_versions_whole`、`change_of_three_versions_yi`、`failure_swap_back_*`。
取样只在一个点：N₀ = 4096、扇出 16、深 3、P = 4096、热键 64、一批 16、一次改动 3 版。丙的冷键步数是 P 的线性函数、甲的拷贝是扇出与深度的函数，这两个旋钮没扫（「不许先有结论再建模型」第四、六问没答满）。建它之前我已倾向「乙最脆弱」，对面那条（乙不脆弱）最好的样子是「验收在每次发布处装钩子、写者在每版交差量加撤销记录」，模型里的 `change_of_three_versions_yi` 就是按这条算的，量出来能读、代价每版 N。量出相反的数（例如丙的冷键步数不随 P 涨）我会撤一·4。
模型数的是条目与节点个数，不是挂钟，也不读 `crates/`；它说明的是表示本身的代价形状，不证明 `crates/` 里哪一段的实测占比。

## 要主 agent 跑的命令

1. 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r1-opus -- bash /home/fy5090/code/singlefs/research/prompts/m4-publish-without-whole-version-copy-design-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs/research/prompts/m4-publish-without-whole-version-copy-design-r1-snapshot /home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r1-opus/model-run /home/fy5090/code/singlefs/research/prompts/m4-publish-without-whole-version-copy-design-r1-opus-model`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r1-opus`（草稿目录）。
   - 看哪几行：末行 `TASK_SERVER_RESULT` 的 `exit=0` 算过；`task-server/task log <号> --all` 里 `name=rerun_model_exit exit=0` 与 `✓ 全部断言对上` 两行都在算过；有 `✗` 行就是那一格的断言没对上（判决引数之前逐行对本报告的数），输出另留在运行目录的 `version-representation-counting-model.out`。

## 没打中的形状

- 池级 checker 读写者状态：试了 `crates/singlefs-checker/Cargo.toml` 的依赖与 `src` 里 `TransactionOutput` 的出现（只在 `rebuild.rs` 第 230 行、`walk.rs` 第 6847 行两条注释里提到名字），没打中。
- 崩溃点重放读版本：试了 `crash.rs`、`crash_judge_tables.rs`、`memory_pool.rs`、`crash_verify_gpu.rs` 里的 `.units` 与 `TransactionOutput`（`grep -n 'TransactionOutput'` 在这几份里零命中），没打中。
- 取号全或无失败的回卷碰版本：读了 `transaction.rs` 第 770 行起的 `roll_back_acquisition` 与第 836 行起的 `AcquisitionRollback`，回卷在任何单元写之前，没打中。
- 「一个事务层」：三条都不碰盘上发布路径，没打中。
- 甲在用户覆写（不带搬迁）这一种发布上退回 O(N)：照模型 `publish_jia` 每次 3 个节点，`TransactionOutput` 这一侧没找到退回的写法；退回的是一·1 那几段（不分辨臂）。
- 「`assert_eq!(…output…, …before…)` 15 行」的命令：`grep -rnE 'assert_eq!\((&?\*?[a-z_.]*(version|current|output)[a-z_]*), *&?\*?[a-z_.]*(version|current|output|before)' crates/singlefs-verification/tests`，输出 15 行，其中至少 1 行（`fsync_drop_and_devices_without_the_selected_version.rs:551`）比的是版本键、不是版本，这个数只是上界。

## 这条腿自己的限度

- 模型没跑：报告里模型的数全是推的，等「要主 agent 跑的命令」第 1 条。
- `.units` 的 386 行里测试与装置那部分没逐行分类；`.allocation_records` 只逐行看了 core 与 checker 的 `src`；`.mapping_keys` 产品侧的结论只按 `\.mapping_keys\b` 正则加读上下文。
- 一·1 说「三条只换版本表示时不动分配器」是按背景材料「这一题只判剩下那一类：发布的产出 `TransactionOutput` 里装着整版的东西」的射程读的；正推腿若把分配器划进甲，一·1 对甲撤回一半。
- 甲按形状比相等的风险（二·4）是推的，我的模型是定形树。
- 丙的 `Arc` 链很长时默认的递归析构会不会撑爆栈：推的风险（链长到上万版时），没量；照 `Rust` 的 `Drop` 一般行为推，未在本项目验证。
- 没读禁读清单里的文件；没读正推腿的报告。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；没往统一任务服务交任务，判决要引的数没复跑（命令在上一节）；没改 `crates/`；没建仓副本与编译目录。
