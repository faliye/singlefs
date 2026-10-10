**出处 `.claude/kb/invariants.md:1-8`（整段抄，未转述）**

```markdown
# 不变量清单

**checker 是 invariants.md 的可执行形式。** invariants.md 每加一条，checker 就要加一个检查。
两者不同步的 commit 一律不收（见 [format-evolution.md](../rules/format-evolution.md)「硬约束」）。

每条不变量必须写成**可判定**的形式——能对着一个镜像回答"成立/不成立"。池级 checker 对一条不变量的判定有四种：成立、不成立、不适用（必须带理由，不许静默算成立）、部分没判（`InvariantVerdict::PartiallyJudged`，`crates/singlefs-checker/src/image.rs`）。部分没判是判到的那一部分都对得上、射程里另有一部分这张镜像上判不了，说明逐条列出没判到的是什么；它不是红，也不是成立：层 0 与崩溃放量把它算成不红，层 0 计数并进「不适用」那一格，GPU 判器只记违例位、它在那一侧与成立、不适用同是 0。今天报它的不止一条，逐条见各行（例：I-5.4（分配记录罩住的槽互不相交）、I-5.5（从 mkfs 水位建树那一版的 t1 落点）、I-9.28（全部可读根里同一树 ID 的诞生 txg 相同））。
写不成可判定形式的，说明还没想清楚，不许进 invariants.md。

```

**出处 `.claude/kb/invariants.md:125-135`（整段抄，未转述）**

```markdown
## I-2 校验和 —— 类
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-2.1 | 校验和与内容匹配 | 任一被引用的块，其校验和与内容匹配 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判；2026-09-17 起「被引用」按 I-3.1（已分配统计对得上） 那个候选集里的根算：F 之下的根引用的单元可以已被回收复用，不在它们上判——不抬 F 就回收复用时 A 的根还在候选集里、这一条红，是 C22（刚释放的块立即重分配） 的必红） |
| I-2.2 | 校验和算法全盘一致 | 校验和算法标识在整个文件系统内一致。**字段 2026-09-14 用户定案有了**（D22（单元原子性怎么合成） 已定项 9）：系统配置自举头段「校验和算法标识 1」，登记表 0 = 无效、1 = CRC-32C，第一版写 1；读到 0 或未登记的码一律拒绝挂载。算法的四个参数写死在 D18（块里携带什么信息） 已定项 17 | 未实现（条款已写、检查仍欠） |
| I-2.3 | 补齐字节为零且参与校验和 | 任一数据单元（码 1）或打包记录单元（码 3，D18（块里携带什么信息） 已定项 11），其声明长度（净荷字节数；补齐起点 = 头 + 声明长度）之后到单元末尾的补齐字节**恒为 0**，且校验和 / MAC 覆盖**整个单元含补齐**（[decisions.md](decisions.md) D4（校验和位置） 已定项 2：不足一个单元的 extent 补齐到 32 KiB；码 3 由头校验和（到头末尾）与载荷校验和（头之后到单元末尾）两段合起来覆盖整个单元）。补齐字节非零、或校验和只覆盖声明长度，一律判红。索引节点（码 2）从 2026-09-13 起同样在射程内（D18（块里携带什么信息） 已定项 18 用户定案）：声明长度 = 条目区字节数，之后到节点末尾的补齐字节恒为 0，载荷 CRC 覆盖 [86 + 2 × key 宽, 16384)——起点是明文头末尾，含 29 字节预留位、条目区与其后的补齐区，口径与码 1 的 105、码 3 的 107 同（预留位 2026-09-14 从 28 加到 29，含预留位的头是 115 + 2 × key 宽）。⚠️ **加密开启时判的是解密后的明文，不是盘上字节**（2026-09-07 用户定案，还清 C189（I-2.3 在加密开启时判明文还是盘上字节没定））：补齐在盘上是 `E(0)` 而不是 0，按盘上字节判必然全红；checker 本来就要解整单元，取值同一步就有。**与 D18（块里携带什么信息） 已定项 7 的载荷 CRC 方向相反是对的**——载荷 CRC 要给无密钥一侧用，所以算盘上字节；这一条只有 checker 用，所以判明文 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-2.4 | 头校验和覆盖范围 | 任一单元的头校验和覆盖从偏移 0 到该类明文头的最后一个字节，端点写死：码 1 到偏移 104、码 2 到偏移 85 + 2 × key 宽、码 3 到偏移 106（D18（块里携带什么信息） 已定项 7 / 已定项 11 / 已定项 18 的字段表）；头校验和字段自身按 0 参与。**29 字节 nonce / MAC / 算法类型预留位虽然算进头（已定项 16）、也是明文，但不在头校验和覆盖内**——MAC 在头校验和之后才算得出来，罩不住它；单元头的这 29 字节、指针头部的 MAC 16 + nonce 12 与算法类型 1、压缩算法码 1、压后长度 2（D19（块指针的结构与宽度预算） 已定项 3；第一版不加密、压缩算法码登记表只有码 0、码 0 时压后长度恒 0，D9（加密） 已定项 10 / 已定项 14）由同一条规则守：加密关着时恒 0、读者遇到非 0 一律判该结构损坏（已定项 17；指针头部那一半 2026-09-27 主 agent 定，池级判定在 A3-checker-2 里做），加密开启后由 MAC 自己罩（2026-09-14 用户定案把预留位从 28 加到 29，多的那 1 字节是算法类型）。根槽偏移 428..457 的算法类型 1 + nonce 12 + MAC 16（D22（单元原子性怎么合成） 已定项 7 的字段表）是同一条规则的第三处：加密关着时恒 0，池级 checker 对根环里每一条读得出、自证过的根判这 29 字节全 0（不按回退候选集滤），core 的根记录读者不读它们、挂载透明也不比它们（判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md` 第二节 Y1「算法、nonce、MAC」那一行：单镜像性质，归池级 checker）。指针头部的 extent 偏移 2 字节第一版恒 0（第一版指针指整个单元），读者遇到非 0 判该指针所在的结构损坏，与 MAC / nonce 同一处置、与加密开不开无关（D19（块指针的结构与宽度预算） 已定项 3；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 11 行）。根记录里四条指针（树表、实例表、中央映射树、分配记录树）头部有一段不是第一版的取值（MAC / nonce 非 0、算法类型非 0、压缩算法码非 0、压后长度非 0、extent 偏移非 0，五样任一）按这条根自证不过处置：core 的根记录读者（`RootRecord::parse_slot`，与 journal 新根段那四条指针同一判法）把这条根算不可择、退回上一条根（靠 journal 重放可以回到同一版，挂载照样成功），checker 红这一条而 core 挂载成功，两边各判各的、不矛盾（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 6 行）。加密开启（系统配置级、全有全无，I-6.4（无未加密块））后覆盖范围 = I-6.2（明文头字段白名单） 该类白名单里的全部明文头字段，密文段由 MAC 罩。改动覆盖区内任一字节必须失配、改动净荷字节不许失配 | 已实现（`crates/singlefs-checker` 的 `check_unit`：三类的端点各自写死，里程碑步 2 / 步 3 验收里改坏头任一字节判红，2026-09-14；池级走读在 `walk::judge_unit_header`（`crates/singlefs-checker/src/walk.rs:1174` 起）另判单元头偏移 4 的格式版本 = 1（walk.rs:1220，不认得就判该结构损坏、不按今天的字段表往下解）与 29 字节预留位全 0（walk.rs:1258）；走读跟随的每条指针头部按三个判法各判一遍（`crates/singlefs-checker/src/image.rs`：`judge_pointer_mac_and_nonce_are_zero`（:1381）判 MAC 16 + nonce 12 全 0，`judge_pointer_encryption_and_compression_fields_take_the_first_version_values`（:1396）判算法类型、压缩算法码、压后长度三段各一格全 0，`judge_pointer_extent_offset_is_zero`（:1456）判 extent 偏移 0），五个入口是码 2 节点指针（`Walk::read_index_node`，walk.rs:1301 起：树表单元、树表条目的树根、中央映射树、inode 树、extent 树、分配记录树的节点都经它，根记录里树表、中央映射树、分配记录树那三条根指针也经它）、墓碑树与谱系树内部条目的子指针（`walk_the_entries_of_a_container_tree_node`，walk.rs:2150 起）、inode 内部条目子指针（`walk_inode_internal_node`，walk.rs:2443 起）、实例表链指针（`walk_instance_table_chain`，walk.rs:2836 起，根记录里的实例表指针是链的第 0 片）、extent 数据指针（`walk_extent_data_pointer`，walk.rs:3009 起），只判不断、指针指向的单元照样走下去，2026-09-27 实审 A3-checker-2；根槽偏移 428..457 那一处也已实现（`image::judge_root_record_algorithm_nonce_and_mac_are_zero`，image.rs:1431，`walk::check_pool_image` 经 `check_pool_image_given_the_tombstone_container_capacity_knob` 在 `valid_roots` 的结果上调它，walk.rs:10413）；坏镜像 `crates/singlefs-harness/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs`）。**extent 偏移那一半与算法类型、压缩算法码、压后长度那几段已实现**（checker 走读跟随的每条指针都判这三格；GPU 判器走读内核 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 的 `w_judge_pointer_head` 与 `w_judge_pointer_extent_offset` 在同样那几个入口判，根槽那一处是同文件的 `judge_root_record_algorithm_nonce_and_mac_are_zero`；core 每个读盘上指针头部的读者都判这五段（MAC / nonce、算法类型、压缩算法码、压后长度、extent 偏移）、判该结构损坏：树表条目的树根指针、inode 内部条目的子指针、extent 叶记录的数据指针、extent 上段叶条目的下段根指针与内联数据指针、extent 树内部条目的子指针、分配记录树内部条目的子指针、实例表链指针各一处，加 journal 新根段、码 2 内部条目、根记录三处（`crates/singlefs-core/src/pointer.rs` 的 `read_judging_the_encryption_compression_and_extent_offset_fields_from`，`PointerHead`、`NodePointer`、`DataPointer` 各一个；`grep -rn 'Pointer::read_from' crates/singlefs-core/src/` 只剩 `pointer.rs` 自己的单测一处）；影子账认单元的两个读者（`allocation_record_tree::node_pointers_as_far_as_readable`、`recovery::instance_table_page_pointers_as_far_as_readable`）经 `NodePointer::read_with_head_fields_judgement_from` 的 `NotJudgedByTheShadowLedger` 不判（影子账要多隔离、不漏隔离）；走读不跟随的指针 I-2.4 不判：根环里走读没走到的旧根，它们的树表单元只被 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 读条目取诞生 txg、不跟随条目里的树根指针，这些指针的头部没人判（测过的镜像上 12 条：三个旧树表单元各四条树根指针，`research/prompts/m3-formal-code-b30-r1-sonnet-output.md`；core 把那样一条根当恢复目标时经 `TreeTableEntry::parse` 判，推的、没造盘面验证）；坏镜像遍历偏移 32、33，core 每类读者各一条用例在 `crates/singlefs-harness/tests/pointer_extent_offset_that_is_not_zero_is_refused_by_every_tree_reader.rs`，加密、压缩那几段 core 每类读者各一条在 `crates/singlefs-harness/tests/pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_every_tree_reader.rs`，根记录四条指针各改一类字段、两种挂载都退到上一条根的在 `crates/singlefs-harness/tests/root_record_pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_both_mounts.rs`；池级 checker 的坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`：extent 偏移落在墓碑树、谱系树的容器子指针上（`an_extent_offset_on_the_container_pointer_of_the_tombstone_tree_root_reddens_only_the_pointer_head_invariant` 与谱系树那一份），算法类型、压缩算法码、压后长度各一份（`an_inline_data_pointer_naming_an_encryption_algorithm_reddens_only_the_header_coverage_invariant`、`a_tree_table_root_pointer_naming_an_unregistered_compression_algorithm_reddens_only_the_header_coverage_invariant`、`an_inline_data_pointer_carrying_a_compressed_length_without_compression_reddens_only_the_header_coverage_invariant`），根槽算法类型、nonce、MAC 各一份（`a_root_slot_whose_algorithm_type_is_one_reddens_only_the_header_checksum_coverage_invariant` 等三条）；对应的账是 C581（extent 偏移非 0 没有判据）；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 11 行、`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 5 行、`research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L4 的容器子指针那一行、`research/prompts/m3-formal-code-b30-r1-main-verification.md` 第二节 M2 的算法类型等三段与旧树表根指针两行、`research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P2 的根记录四条指针那一行） |
| I-2.5 | 位置条目按设备身份升序 | 任一块指针、中央映射条目的 value、journal 点名项里的位置条目数组按设备身份严格升序（D19（块指针的结构与宽度预算） 已定项 11、D23（journal 的角色与格式） 已定项 17，2026-09-13 用户定案）；两条位置条目设备身份相同或逆序，一律判损坏。⚠️ **射程（2026-09-14 补）：整条 86 字节全零的指针表示「注册了但还没有根」，整条豁免升序判定**——它内部那两条位置条目都是 (设备 0, 槽 0, 校验和 0)，按字面正是「设备身份相同」那一档。今天写出全零指针的有四处：树表里 livelist 树、稀疏旁表树与 deadlist 树那三条条目（D6（快照实现模型） 已定项 2、D5（快照 / 空间记账机制） 已定项 6 与 D5（快照 / 空间记账机制） 已定项 11 各自定案的「day-1 注册进树表，根指针为零」），以及 mkfs 与暖机的根记录里那个还没发布过的中央映射树根指针。非全零的指针不享这条豁免。 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |

```

**出处 `.claude/kb/invariants.md:190-212`（整段抄，未转述）**

```markdown
## I-5 分配器 —— 类
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-5.1 | 物理范围不重叠 | 不存在被两个不同引用同时占用的物理范围（除非是显式的多副本） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） 引用按版判（2026-09-27，`research/prompts/m2-rev-b1-implementer-report.md` 第六节第 1 条、`research/prompts/m2-rev-b2-implementer-report.md` 第八节）：同一版（根环里的一条根，或由记录施加出来的一版）里同一个 (设备, 槽) 被两条指针引用判红，(设备, 槽) 取自位置条目的设备 ID 与物理偏移（D19（块指针的结构与宽度预算） 已定项 4），一条指针自己的两条位置条目同落点算一个引用（那一格归 I-2.5（位置条目按设备身份升序））（`walk::Walk::note_references_of_a_pointer_followed_by_the_walk`）；中央映射条目指的物理范围算进引用集合；别的版本走过的共享子树不重走，按缓存把它下面的落点并进这一版再比、不读盘（`walk::Walk::merge_placements_below_a_unit_walked_by_an_earlier_version`）。坏镜像在 `crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs`。 |
| I-5.2 | 空闲统计对得上 | 空闲空间统计 == 总空间 − 已分配空间 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-5.3 | 报出的空闲都兑现得了 | `df` 报给用户的空闲 ≤ 当下可分配 + 能在有界步数内放回的扣住量（defer 窗口里、回退下界以下的释放）。运行时的等价形态：`df` 报出 s 字节空闲时写 s 字节必须成功；删掉 s 字节之后同样大小的写在有界步数内成功（[decisions.md](decisions.md) D3（空间分配） 已定项 9，用户定案）。界随 D16（发布语义） 已定项 1 的形态定。判别力：把「放回扣住量」那一步摘掉，近满盘上删一个对象再写同样大小必须由绿转红 | 未实现 ⚠️ **口径注（2026-09-13，D28（挂载期承诺量） 已定项 4）**：`df` 要扣的 checkpoint 保留池是每次发布现算的量，发布即重算，近满盘上报出的空闲会随树高变小。 |
| I-5.4 | 分配记录罩住的槽互不相交 | 判的版本是回退候选集里每条根，加由记录施加出来、根槽从没落盘的那几版（已被一次恢复施加过、实例表里有行，树表指针与「树表 0 条那一版的分配记录树根」取自记录的新根段）与下一次挂载先施加的那一版（记录与最新根同实例、txg 比最新根大 1、带提交标记，根环里没有它的根）；这些版本的分配记录树里，同一块盘上任意两条分配记录（不论已分配、已释放、释放代是否已低于回退下界）罩住的槽区间 [槽号, 槽号 + 跨度) 互不相交。复用一个已回收的落点时，新记录罩住的槽上别的已回收记录要随这次分配一起删掉；留着它，下一次可写挂载重建分配器时两条记录罩住同一个槽。同一条下另判一格：树表非空（带树）的那一版，根记录里「树表 0 条那一版的分配记录树根」那 86 字节必须全零（D22（单元原子性怎么合成） 已定项 7 字段表「分配记录树根指针」那一行），非零就是这一版有两棵分配记录树，判违例。这一格成立的那一次，只在这一版自己的分配记录树（树表种类 3 那一条指着的那一棵）走到叶时计进 I-5.4 的评估，没走到叶的版本不计。部分没判（逐版列出，不判红；判出违例照报违例）：树表指针非零而树表读不出的版本、树表指针全零的版本（core 的根记录读者照它读盘 0 槽 0、读不出），都认不出带不带树，根记录里分配记录树根全零那一格与它自己的分配记录树都没判；树表读得出、0 条而分配记录树由根记录直接持有（C512（树表 0 条的一版上被换下的实例表记在哪没有条款））、那棵树读不出的版本；带树一版自己的分配记录树没走到叶的版本。候选集里没有一棵分配记录树走得到叶（第 0 代树表是空的，或树表 / 分配记录树读不出、位置对不上）报不适用。 | 已实现（2026-09-18，`crates/singlefs-checker` 的 `walk::judge_allocation_records_disjoint`，吃一张版本表（`VersionToJudgeTheAllocationRecordsOf`：称呼、树表指针、树表 0 条那一版的分配记录树根），表里是回退候选集里每条根、由记录施加出来的那几版与下一次挂载先施加的那一版（`check_pool_image` 拼表）；GPU 判器扫描内核 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 的 `judge_allocation_records_disjoint_of_a_version`，候选根、由记录施加出来的版本、下一次挂载先施加的那一版各调一次，只有违例位、部分没判不置位；这两类版本上的坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `two_allocation_records_covering_one_slot_in_a_version_applied_only_by_its_journal_record_redden_only_the_disjointness_invariant` 与 `two_allocation_records_covering_one_slot_in_the_version_the_next_mount_applies_first_redden_only_the_disjointness_invariant`，读不出的几格各有用例（`the_tree_table_of_a_version_applied_only_by_its_journal_record_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged`、`the_allocation_record_tree_held_by_the_root_record_of_a_version_without_trees_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged`、`an_all_zero_tree_table_pointer_in_an_older_candidate_root_is_followed_like_the_core_and_every_tree_table_reading_names_it_unjudged` 等）；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `an_allocation_record_whose_span_covers_the_next_record_reddens_only_the_disjointness_invariant`；层 0 每个崩溃状态都判，同日在新代码上全量：第二条流 2,104,413 个状态里评估 1,842,252 个、违例 0，其余 262,161 个报不适用，第一条流 262,165 个状态里评估 4 个、违例 0（`m2-wave2-crash-verifier-report.md`「主 agent 要的两个数」）。同日立，依据 `research/prompts/m2-wave2-code-r1-main-verification.md` 第二节第 1 行：复用时跨度变了留下重叠记录，下一次可写挂载在重建分配器时 panic，而 I-5.1（物理范围不重叠） 只管树里的引用、判不出记录之间的重叠；带树一版分配记录树根全零那一格的挂法见判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J1「I-5.4 挂法」那一行与 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L2「成立要至少一棵树走到叶」那一行，树表读不出与根记录直接持有的树读不出两形见 `research/prompts/m3-formal-code-b30-r1-main-verification.md` 第二节 M2「I-5.4 按版本：坏盘」那一行与 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P2 第一行，射程含两类由记录造出来的版本见同一节 P2「由记录施加出来的版本与下一次挂载先施加的那一版」那一行，树表指针全零的版本见同一节 P2「树表指针全零的根」那一行与第五节第 1 条） |
| I-5.5 | 从 mkfs 水位建树那一版的 t1 落点 | 走读到的任一版 V（候选集里每条根，含由记录施加出来的那几版）若是从 mkfs 水位第一次建树的那一版——① V 的树表非空，且种类 1（extent 树）条目的树 ID 等于 65536；② V 的树表里每条条目的诞生 txg（条目偏移 108）都等于 V 的 `checkpoint_txg`——则 V 引用的数据单元里，每块盘上落点槽号最小的那一个，槽号等于这一版分配记录里 D3（空间分配） 已定项 8 第 1 条那条政策算出的那一对的起点槽号（起点最小、32768 对齐、两槽都空、不在开放段里）。新池新建文件那一次是 U + 4，U 是所选系统配置偏移 417 的单元区起始槽号（默认 768 MiB 环 U = 50176，所以是 50180）；回收之后（例如 mkfs 的实例表 U、U + 1 那两槽被回收）不再是 U + 4。V 没有数据单元时不判；不满足 ①② 的版本不判。这一类含新池新建文件，也含 mkfs 之后重挂、实例 1、2、3 从 mkfs 水位写的第一个文件（用户 2026-10-04 定案，原话在决策变更史 D3（空间分配） 节）。只判 t1，t2–t12 不判（D3（空间分配） 已定项 10 射程没有「偏离即不合法」）。已知限度：这一版盘面上复算不出的几样只能近似——已释放的记录一律当占着（回收门槛复算不出：写死从 U + 4 起扫的坏实现与回收门槛放宽的坏实现两个方向都判不出，C689（建树落点判据抓不到写死从 U + 4 起扫的坏实现））；这一版自己分配的记录当空；开放段拿这一版树表单元所在的段当它（这次挂载更早开过、已经用尽的段认不出；提交内生块回落时会误排：这要求这一版发布时每块盘上都没有全空的 64 槽段，今天的写侧在「只暖机 / 只写行的挂载接发第一个文件」这一族里量到 0 次回落（单元区 240 / 256 / 384 槽，连续挂载 k ≤ 60，一次抽样、不是证明）；分配器上留了回落计数，`crates/singlefs-harness/tests/allocator_fallback_count_on_small_devices_after_repeated_mounts_and_the_first_file.rs` 断言它是 0，哪天走进回落、用例红，回来重看开放段认法）；隔离位、抬 F 扣住位与保护期内的 R 盘上都没有，没有分配记录罩着的槽一律当空。依据 D3（空间分配） 已定项 8 射程与已定项 10 ③ ④，以及判决 `research/prompts/m3-formal-code-b17-r1-main-verification.md` 第二节 K2 与用户 2026-10-04 对判据改法的定案（原话在决策变更史 D3（空间分配） 节）。 | 已实现（判据是 f2（按这一版分配记录现算），代码轮判决 `research/prompts/m3-formal-code-b19-r1-main-verification.md`；池级 checker `crates/singlefs-checker/src/walk.rs` 的 `judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark`：候选集里每条根与由记录施加出来的那几版各自重读树表与 extent 树判，U 取所选系统配置偏移 417；GPU 判器扫描内核 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 的 `judge_first_data_unit_of_a_version`，`IMPLEMENTED_INVARIANTS` 下标 67；判不了的几格报部分没判（`InvariantVerdict::PartiallyJudged`，没有违例时整条报它，逐版列出实例、txg 与为什么，形态同 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同））：这一版的树表读不出（含树表指针全零：core 的根记录读者照它读盘 0 槽 0、读不出）或条目宽不到 200 字节，认不出这一版是不是这一类；认出来的那一版 extent 树读不全（数据单元数不全）、分配记录树读不全（落点算不出）；种类 1 条目不止一条、种类 3（分配记录树）条目不是恰好一条，这两格的树表本身由 I-9.16（树表条目按树 ID 严格升序且合发号次序） ③ ④ 判红、这里只记没判；有数据单元而池里某块盘上一个都没有，那块盘没有对象，跳过、不记（CPU 与 GPU 一致）；GPU 判器只有违例位，这几格不置位；走读到的版本里一版都不是从 mkfs 水位第一次建树的那一版时，整条报专门的不适用理由（`walk::NO_VERSION_BUILDS_TREES_FROM_THE_MKFS_WATERMARK`，`crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs` 里那条用例的 I-5.5 一支只认它），认出来而没有数据单元那一形仍落缺省理由；三格的坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `a_tree_table_that_does_not_read_back_leaves_the_first_data_unit_invariant_partially_judged`、`an_extent_tree_that_does_not_read_back_leaves_the_first_data_unit_invariant_partially_judged`、`an_allocation_record_tree_that_does_not_read_back_leaves_the_first_data_unit_invariant_partially_judged`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（数据单元从 50180 搬到 50182），合法镜像在 `crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs`） |

> ⚠️ **I-3.1（已分配统计对得上） 与 I-5.2（空闲统计对得上） 的等式在 zoned 布局上有一个已知缺口（[decisions.md](decisions.md) D12（目标介质） 定了多布局之后才出现）。**
> zoned 设备上一个 zone 里只要还有一块活数据就不能 reset ⇒
> **一个活快照钉住的物理空间是 zone 粒度，而它引用的块数是块粒度**。
> 于是「已分配 == 遍历所有引用得到的和」在那套布局上不成立，
> 除非把「已分配」明确定义为「被引用」，**并另立一个「不可回收」量**。
>
> 连带一条对用户可见的要求：**报给用户的「这个快照占了多少」不许被定义成引用块数之和**——
> 那个数在 zoned 上会**系统性地低于**删掉该快照真正能拿回的空间，
> 而 [pitfalls.md](pitfalls.md) 第 12 条骂的正是这类「`df` 输出对用户没有意义」。
>
> **本缺口尚未有对应条目**，补它需要先定义「不可回收空间」这个量。

```

**出处 `.claude/kb/decisions/16-发布语义.md:1-4`（整段抄，未转述）**

```markdown
## D16 发布语义 —— 已定

D16（发布语义） 管发布这一格：一个新根什么时候发、一次发布按什么顺序落盘、发布计数器是哪一个数、这一批释放什么时候回可分配集合、新实例的第一个根怎么先覆盖两块盘、一次空发布写不写单元、journal 记录格式在 D15（格式冻结政策） 的冻结分层里算什么。不管的：journal 的记录字段、环几何与重放前缀在 D23（journal 的角色与格式）；根环参数、根记录与系统配置的字段表在 D22（单元原子性怎么合成）；快照的记账模型与生死边界表在 D5（快照 / 空间记账机制） 与 D6（快照实现模型）；准入不等式与挂载期承诺量在 D3（空间分配） 与 D28（挂载期承诺量）；一次发布事件内部的内存序在 `litmus/commit-publish.litmus`，那份钉的是写块内容 → `smp_wmb` → 写 generation，对频率一字未提。

```

**出处 `.claude/kb/decisions/16-发布语义.md:27-88`（整段抄，未转述）**

```markdown
#### 已定项 1：defer 窗口取按盘回退下界的收严形态，按状态数读

**定案**：回退候选集保留最近 4 个可退到的不同状态——只数改过用户可见状态的根，推空与抬 F 产生的空发布根不算；同一个状态只数一次（去重的判法在「「非空」从盘上怎么认」那一段）。**根环容量是这句的边界**：两个状态来回交替超过根环一圈时，环里的根全是这两个状态，保留的就只有环里有的那几个，任何机制都给不出环里没有的状态。**窗口的单位是发布，不是秒**：defer 窗口保护的是「环里的根还能不能用」，普查过的消费者没有一个按秒计价。形态取 E139（按盘回退下界的收严形态） 的臂 G2S；回收门槛退回环里最旧有效根之后，装置的候选下界还没带系统配置里的 F，这一句仍不与装置里跑的一字不差，欠里记着 E139（按盘回退下界的收严形态） 要重跑：

| 项 | 定案 |
|---|---|
| 可再分配 | 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根) |
| 环里最旧有效根 | 盘上全部根槽里自证合法、按实例表判仍然有效的根的最小 checkpoint_txg；写失败的槽按旧内容算，在飞、没持久的发布不算。**同取这个值的六处**（用户 2026-10-04 定，被攻过一轮）：分配器的回收门槛（含抬 F 那一遍）、checker 的候选过滤、checker 的 I-3.1（已分配统计对得上） 丁-defer 门槛（max(F_生效, 环里最旧有效根)）、影子账的候选过滤、回退候选集、抬 F 的上限（含判「非空」那一遍）；六处都不再按一圈窗口滤根，窗口之外的有效根（崩在记录之后、根槽之前那一形留在槽里的旧根）也在候选集里、也压着门槛。**挂载时读不出的槽**：N = 环里读得出的最新有效根的 txg（挂载重建分配器时取它、不取 first_txg − 1；进程内每记一条根时取这条根）；读不出的槽在 txg > N − 3S（一圈，3S 是环长）里只对应一两个 txg：门槛按最小的算，根环表记最大的（零轮，只在攻方模型上量过），抬 F 那一遍里根环表挂载时读不出的那几项同样按这一句并进最旧有效根；一圈窗口这个量现在只剩这一处在用，给读不出的槽划出它能住的 txg 的范围。进程内记根时，根环表里这一项成了最小的那个，先重读那一槽：读得出、自证过、按实例表仍然有效的，照读出来的根算；读得出而自证不过的，与仍读不出同一处置，回收门槛照旧钉在它可能住过的最旧 txg（「根槽这一次读坏」那一行）；读得出、自证过而按实例表被抛弃的，当没有根（用户 2026-10-03 定先重读；2026-10-04 先定自证不过那一格取按记的最大 txg 算、同日后来定读坏的槽在回收门槛里钉在可能住过的最旧 txg，2026-10-05 定以钉最旧 txg 为准，零轮）；重读读出有效根的那一槽，算进「这个进程知道住着根的槽」（「根槽这一次读坏」那一行）。 |
| 回退下界 F | 根记录带 8 字节（D22（单元原子性怎么合成） 已定项 7，242 → 250），系统配置另记一份 8 字节（D22（单元原子性怎么合成） 已定项 9）。平时不动，窗口就是整个根环；准入不够时抬，正常卸载时抬到现行那一版的 txg |
| 抬 F 的上限 | **准入抬 F**：min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)；非空有效根不足 4 个时取最旧的有效根；「有效根」「非空」在环里全部的根上取；挂载那一刻读不出或自证不过、之后这个进程没写过的槽，当作可能是一条非空有效根，F 不越过它（用户 2026-10-04 定，零轮：先定只并读不出的槽，代码轮第三轮之后重定为读不出或自证不过两类都并；与「环里最旧有效根」那一行把读不出的槽并进回收门槛同一个思路）。读坏槽并进上限、卡住就在这次挂载里报 ENOSPC、不推空发布；读回全零的槽不并进上限、F 可能越过住在全零槽里的有效根（用户认的代价）。一次处置的目标 = min(这次释放的释放代, 第 4 新的非空根)。**卸载抬 F**：抬到现行那一版的 txg，不另判上限（等价于上限取最新的持久有效根，不按盘取小） |
| 根槽这一次读坏 | 算准入抬 F 的上限、挂着时管理员回退判候选（根环里有没有目标）与算 F_生效 时读根环：挂载那一刻就读不出或自证不过、这个进程之后也没写过的槽，算准入抬 F 的上限时当作可能是一条非空有效根、F 不越过它（「抬 F 的上限」那一行，用户 2026-10-04 先定只并读不出的槽、代码轮第三轮之后重定为读不出或自证不过都并，零轮），判回退候选与算 F_生效 时当没有根；挂载那一刻读得出、这个进程写过且 FUA 返回过、或进程内记根时重读读出了有效根的槽（这三类是「这个进程知道住着根的槽」），这一次读不出或自证不过就重读一次，仍坏就拒这次抬 F 或这次回退、报它自己的错误成员，不按有根或没根猜（抬 F 那一半用户 2026-09-26 定，推的，被攻过零轮；回退那一半主 agent 2026-09-28 定，被攻过一轮：代码三方第三轮判决 `research/prompts/m2-closeout-code-r3-main-verification.md` 第一节判替没写的条款做了选择、回退扩写拒错或放错那一格没打中（一次抽样））。可写挂载的择根按 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续处置（D23（journal 的角色与格式） 已定项 14：系统配置见证过比所选那一版新的发布就把读阶段重读一次，仍判真拒可写）；只读挂载的择根、「环里最旧有效根」怎么处置这一格没判，见 C563（读根环一槽读坏就当没有根，别的调用方没判）；**读坏的槽与读回全零的槽**：挂载那一刻读坏（读不出，或读得出不是全零而自证不过）与读回全零的槽，回收门槛三处（挂载时含 m 取不到那一支、进程内、抬 F 那一遍）钉在它可能住过的最旧 txg，这个进程写掉那一槽、或进程内记根时重读读出有效根或被抛弃的根才解开，重读读回全零照旧钉着；读坏的并进准入抬 F 的上限，读回全零的不并（「抬 F 的上限」那一行）。**可写挂载重建分配器读根环那一遍**：有一块盘一条不被抛弃的自证有效根都没读出（按读出的那一槽所在区域归盘）就整环重读一次，重读之前调钩子；重读之后逐槽取两遍里较好的那一份（任一遍自证过的根、两遍都自证过取重读那一条 ＞ 读坏不是全零 ＞ 全零），根环表、回收门槛与影子账都用并出来的那一份；两遍并起来一条不被抛弃的自证有效根都没有，在取号之前拒可写挂载，报 `MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`（与读阶段择不到根的 `Recovery(NoValidRoot)` 分开报） |
| 生效 | F 另记进每块盘的系统配置（「抬 F 那一串」那一行写明先写后抬）；恢复后生效值 F_生效 = max(各幸存盘最新持久有效根所带 F 的最大值, 被抛弃时间线上的根所带 F 的最大值, 系统配置里读得出的 F 的最大值)；系统配置里读得出的取每块幸存盘两槽里全部自证过（校验和过且 fsid 与本池相同）的槽，与取号式子同一取法（D18（块里携带什么信息） 已定项 11） |
| 回退候选集 | 按现行那一版的实例表判仍然有效 ∧ txg ≥ F_生效（D23（journal 的角色与格式） 已定项 14 的挂着时回退取同一个集合） |
| 抬 F 那一串 | 准入与卸载共用。动盘之前在分配器的拷贝上把这一串空发布整串预演：任一次取不到落点，就一次都不发、分配器原样换回抬 F 之前那一份（`MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite`）；**预演过了、第一次发布之前，先把新 F 写进每块盘的系统配置、过一道屏障**，之后才发第一条带新 F 的根；**先写那一步任何一块盘失败，这一串的根一条都不发、返回错误；已写进别的盘系统配置的新 F 留着、不回卷，F 的生效值只增不减**（主 agent 2026-09-26 定，推的，被攻过零轮）；空发布推到每块盘上都有一条带新 F 的根为止；这一串回收出来的槽在生效（每块盘上都有带新 F 的根）之前扣住、不发出去，生效之后放开（正常卸载那一格例外，见「正常卸载」那一行）；真发时落盘途中写错，交回已落盘几次（`MountError::RaiseFloorSequencePublishFailed`），那一格里扣住的槽怎么放没有条款，见 C546（抬 F 被拒时扣住的槽不退回） |
| 正常卸载 | 推一串空发布把 F 抬到现行那一版的 txg（「抬 F 那一串」照准入的走法，上限见「抬 F 的上限」那一行）；这一串回收出来的槽，只有释放代不高于「每块盘上最新有效根的 txg」最小者的那一截当场可用（预演与真发都算上），其余与准入那一串一样扣到生效（恢复择根不看 F，丢一块盘时择的是剩下那块盘上最新的根，引用它的单元不能被这一串写掉）；有一块盘一条有效根都读不出时这一截为空、整截扣到生效，这一遍读不出、自证不过的槽当没有根；卸载半途崩溃（系统配置已带新 F、带新 F 的根没落满每块盘）之后的下一次可写挂载同样只放开一截：重建分配器时当场可用的只到释放代不高于 max(min(F_生效, 「每块盘上最新有效根的 txg」最小者), 环里最旧有效根) 的槽，某块盘一条有效根都读不出时只到环里最旧有效根（挂载时读不出的槽按它能住的最小 txg 并进这个最小值，与回收门槛同一个读数），其余扣到这次挂载的暖机在每块盘上都落了根（用户 2026-10-04 定，零轮）；这一串写的根在根记录 flags 里带卸载记号（D22（单元原子性怎么合成） 已定项 7）；第一版几何（R = 3、区域归属 0 / 1 / 0）下空发布 2 或 3 次（现行 txg 除以 3 余 1 时 3 次，余 0 或余 2 时 2 次）；不加干净关闭标记，下次挂载照旧走恢复（D23（journal 的角色与格式） 已定项 14）；现行那一版树表 0 条（mkfs 之后只暖机、还没写过第一个文件）时一个字节都不写、F 不动、不推空发布，卸载照常做成（`crates/singlefs-core/src/mount.rs` 的 `Unmounted::NothingWrittenOnAVersionWithoutFile`） |
| 准入 | 可分配 = min(可再分配 + 活元数据 − 保留池, `df`)；准入不够、或准入放行而落点取不到时，先推空发布抬 F 再判（写行那次发布之前不推：它是新实例的第一次发布，元数据走切换预留，同一次发布里的用户数据重做照走准入、不够返回 ENOSPC）；**可写挂载的准入不够时，写行那次发布照走切换预留，写完行之后推抬 F 的空发布、再判**；一次准入最多 B = 4 + 2 k_tol 次发布（k_tol = 2 ⇒ 8），做满仍不够才报 ENOSPC；挂载处推满仍不够时挂载照样做成可写，之后的发布照发布准入判、不够报 ENOSPC，正常卸载照常（D2（RAID 条带策略） 已定项 13 那一格的例外）（用户 2026-09-26 定，被攻过零轮）；挂载续做的删除类发布被落点那一判以空间拒时同样推抬 F 再试，推满仍放不下挂载照样做成可写（D8（核心索引结构） 已定项 16 第 6 条，用户 2026-10-03 定） |
| `df` | 没用过的 + 已释放的 + 活元数据 − 保留池 − 推空最坏残留 − 滞后量；保留池 = 2 × 5 + (B − 1) × c_max = 10 + 7 c_max，残留 = 5 + (B − 2) × c_max = 5 + 6 c_max；滞后量 = 释放代 > max(第 4 新的非空有效根, 环里最旧有效根) 且不是空发布放掉的已释放块 |
| 环深下限 | 每区槽数 S ≥ k_tol + 2 = 4（D22（单元原子性怎么合成） 已定项 2 的 S 下界随之抬） |

**「非空」从盘上怎么认**：一条有效根算非空 ⟺ 它的树表里用户可见的树（inode 树、extent 树）的根指针，与它前一条有效根（按 txg 排；有效 = 按实例表判仍然有效 ∧ txg ≥ 当前的 F）树表里的不同。比的是树表里这两棵树的条目、不是树表单元自己的落点——每一次发布（连空发布）都重写树表单元（`crates/singlefs-core/src/transaction.rs`），按落点比会把空发布全算成非空。**数「第 N 新的非空有效根」与「4 个不同状态」时去重**：从新到旧数，一条非空根只有在它的 (inode 树根指针, extent 树根指针) 与每一条比它新、已计入的根都不同时才计入——管理员回退发出的新根与它前一条有效根不同、按「「非空」从盘上怎么认」那一判法算非空，而它的状态与更早的某一条相同，不去重就会把同一个状态数两次（第二轮 X7：来回回退之后按内容只剩 2 个不同状态）。反方向只让 F 停住，直到第 4 个状态的根离开根环。

**改新池新建文件的字节：是**：根记录加 F 8 字节；新池新建文件里 F 恒 0（mkfs 与第一次发布时非空有效根不足 4 个，上限取最旧有效根 txg 0）。系统配置加 F 8 字节，新池新建文件里写 0（D22（单元原子性怎么合成） 已定项 9）。

**连带改的**：D22（单元原子性怎么合成） 已定项 7 的根记录加 F（242 → 250）、flags 占一位卸载记号，已定项 9 的系统配置加 F 8 字节，已定项 2 的 S 下界；D23（journal 的角色与格式） 已定项 14 的回退候选集加「txg ≥ F_生效」；D3（空间分配） 已定项 9 第 2 条写明界；I-7.4（近 K 代块未被复用） 的 K 改由 F 表达；I-7.9（回退下界 F 不高于抬 F 的上限） 按卸载记号分两支。

**射程**：定的是 defer 窗口与回退下界怎么算，不定环几何、不定 c_max 的口径（c_max 是声明的最坏空发布开销，按 D28（挂载期承诺量） 已定项 4 按当时结构现算）。已知边角十样：

- **「生效」取 SysPre**：F 只住根记录、根槽不镜像（已定项 7）时，带新 F 的根坏掉 F_生效 就回落，更早的根重新进回退候选集、而它们引用的块已被合法复用——I-7.4（近 K 代块未被复用） 在这段历史上不成立（`research/prompts/m2-step45-code-r2-main-verification.md` 第四节、`research/prompts/m2-step6-checker-r1-main-verification.md` 第四节）；病根是 F 只住根记录，与生效取哪条规则无关（第三轮 K4：带 F 的根全坏那一格，今天的规则、MAX、MAX+HOLD 候选坏 12 / 140 / 12，SysPre 0）。代价：系统配置多 8 字节（动格式）；每次抬 F（准入与卸载）每块盘多一次系统配置写、加一道屏障；checker 与层 0 要跟着读系统配置里的 F——今天的 checker 按根上带的 F 判，带 F 的根全坏时会误红 I-7.4（近 K 代块未被复用） 等五条（第三轮 Y7，推的，没实现），欠 C556（checker 与层 0 不读系统配置里的 F）。「先写后抬」是第三轮攻方提的，被攻过零轮，用户 2026-09-26 知情定。被抛弃时间线上的根带的 F 算进 F_生效（用户 2026-09-28 定「算进」，被攻过一轮，判决 `research/prompts/abandoned-floor-r1-main-verification.md` M2：被抛弃根带着比别处高的 F 时，不算的话 I-7.12（系统配置 F 不低于同盘根上的 F） 红、回退退得到那个 F 之下，而被抛弃时间线抬 F 时已按它回收过槽；实现 `crates/singlefs-core/src/recovery.rs` 的 `effective_rollback_floor_of_the_roots_read`，checker 另算一份 `crates/singlefs-checker/src/walk.rs`）；C331（择根倒挂压过已确认的写） 取甲之后，被抛弃根带着更高 F 的盘面只在构造的镜像上有（那一形要可写挂载时一槽系统配置一时读不出，甲 拒这样的可写挂载）。与 `research/prompts/alloc-basis-r3-main-verification.md` 的 F 口径岔路一起判的那一件仍开着。
- F 回落之后第 0 代根重新进有效根、它的树表单元已被合法复用、此后每一次抬 F 都被拒到根环把它盖掉为止（`research/prompts/m2-emptypool-nonempty-r1-main-verification.md` 第一节 Z1）：前提是 F 回落，取 SysPre 之后在带 F 的根全坏那一格上不再出现（推的，第三轮没造那一形）。一次挂载里连续发布超过根环容量时挂载内不回收，checker 在合法状态上判 I-3.1（已分配统计对得上） 红（E153（账本形态与环上有洞的代价） 在 `crates/` 副本上量到），仍开着，归里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 ② 行。
- **正常卸载的代价**：每次卸载多 2 或 3 次空发布（树表 0 条的一版上不推，见定案那一句），第一版几何上每次 8 个单元、两盘合计 21 次写调用、279040 字节（第一轮攻方 t10、t5a，副本上量的）；卸载之后 F = 现行那一版的 txg，回退候选集只剩现行那一版。用户选时知情的代价是「停机时崩溃兜底只剩最新一条根」（按条款推的）；第二轮在卸载之后再挂、最新 j 条根坏的 75 行上恢复全读对、可写挂载全挂上，这一条没出现：F 之下的根还在环里，它们的记录能把状态重放回这一串的末尾（第二轮判决 G4）。
- **用户知情接受的代价三条**：删掉的空间要再发生 3 次改变用户可见状态的发布才回可分配集合，在那之前 `df` 不把它算空闲、按 `df` 写它是真 ENOSPC；`df` 少报 = 保留池 + 残留 = 15 + 13 c_max（c_max = 5 / 10 时 80 / 145 块）再加滞后量，而滞后量 = 4 × 最近四个窗口的释放量、**没有上界**（一窗删一个 8 块对象时平均约 34 块、最大 44 / 52，一窗删 32 个时 1044 块），删掉 s 字节 `df` 一个字节都不涨；c_max 按现算取，第一版池规模 9 块 ⇒ 保留池 73 块。
- **最少保留 4 个状态只在一次处置里根槽写失败 ≤ k_tol 次时兑现**：S = 4、c₀ = 5、3 次失败那一格违反 24 次，按字面记一次输、按 k_tol 的射程在外；全枚举下 S = 4、c₀ = 1 也违反 160 次（同为超过容忍的格）。
- **代价是选项本身的，不是这一族的缺陷**：一个被保留的状态钉住的是在它那一刻活着、后来才死的全部块，选最近 4 个就是每次删除都撞上——滞后量与「删了要等 3 次状态」都从这里来。
- **回收门槛取环里最旧有效根的代价**：崩在记录之后、根槽之前那一形留在环里的旧根要等被覆盖一圈才离开，每个洞回收门槛多压一圈（正推腿端口上 S = 16 平均 893 块对窗口门槛 583 块）；一块盘持续写不进根时扣住的单元随发布线性涨、没有上界（攻方模型上量，持续写失败 15 / 30 / 60 / 120 次时最多扣住 50 / 80 / 140 / 260）；崩在记录之后、根槽之前连崩 13 次之后挂载（S = 4），已分配 166 块、窗口门槛同一段 26 块（攻方副本上量）。数两腿各量一种口径，不是入库装置上的数；用户 2026-10-04 知情定。
- **E139（按盘回退下界的收严形态） 答不了的**：计数模型，按发布计不按墙钟；只建根一级的引用、没建 journal 记录重放；实例切换的挂载时预留由 D28（挂载期承诺量） 已定项 3 扣；容量 4000 块、两块盘；k_tol 固定 2；滞后量口径是写登记的人替状态数读法构造的；保留池扫描的最小值落在扫描下沿、真正要多少没量到；环深下限是探针的数。两轮前身 E135（动态回退下界） 与 E138（按盘回退下界与推空的空间要求） 的结论已被 E139（按盘回退下界的收严形态） 取代，不再当依据用。
- **I-7.9（回退下界 F 不高于抬 F 的上限） 的重算上限不在六处之内**：池级 checker 与 GPU 判器的重算上限仍只取锚在镜像最新根上的一圈窗口里的根（`crates/singlefs-checker/src/walk.rs` 的 `judge_rollback_floor_raises_against_their_ceilings` 文档注释，5726–5736 行，写了理由：后来的镜像上重算，洞里留下的旧根会把上限算低、在合法镜像上误红），六处退回「环里最旧有效根」不含这一处。
- **重建分配器读根环那一遍按槽归盘、逐槽并两遍**：条文照用户定案写（「根槽这一次读坏」那一行），实现在第 10h 批。

**依据**：

- E139（按盘回退下界的收严形态）：臂 G2S 在全部故障世界（活化区间内崩溃不丢盘、先跳号再抬 F、推空期间 1..3 次根槽写失败、坏槽、开销升到 c_max）零假候选、零不可恢复，各格按自己的 `df` 假性 ENOSPC 为 0，一次准入的发布数实测最大 7 ≤ 8，跑前保留池下不卡死——定案取的就是这条臂（跑前登记 `e139-preregistration.md`，产物 `e139-tightened-floor-2026-09-12.out`）。
- 四轮三方论证，第四轮判决 `research/prompts/d16-item1-r4-main-verification.md`；第三轮 `research/prompts/d16-item1-r3-main-verification.md`。
- 用户三次定案（候选集可以一直缩、最少保留 4 个、「4」数的是可退到的不同状态、三种「哪 4 个」里选最近 4 个），原话在变更史；「非空」怎么认与「写行那次发布之前不推」同为用户定案，原话在变更史。
- 用户定案 2026-09-25：抬 F 那一串改成整串预演，推翻 2026-09-24「维持逐次发布」（原话在变更史）；实现是实二六（`research/prompts/m2-e26-implementer-report.md`）。
- 生效取 SysPre、卸载抬 F 到现行那一版的 txg（B1）、「4 个不同状态」去重与根环容量边界、候选按现行那一版的实例表判：无实验，三方原型（`research/prompts/m2-rollback-forward-r1-opus-model/`、`m2-rollback-forward-r2-opus-model/`、`m2-rollback-forward-r3-opus-model/`）在冻结副本上量过，没立实验号，原型上的数不是入库装置上的数；三轮判决 `research/prompts/m2-rollback-forward-r1-main-verification.md`、`m2-rollback-forward-r2-main-verification.md`、`m2-rollback-forward-r3-main-verification.md`（第三轮第三节站住的、第四节交用户的）。SysPre 的「先写后抬」被攻过零轮。系统配置里读得出的 F 取每块盘两槽里全部自证过的槽、先写那一步失败时一条根都不发且已写进的 F 不回卷，这两处是主 agent 2026-09-26 定的，推的，被攻过零轮。
- 用户定案 2026-09-25（正常卸载时抬 F、取 B1）与 2026-09-26（C419（F 生效之后靠什么保持不回落） 取 SysPre，推翻 2026-09-25 定的「生效值取各盘 F 的最大值」；「4 个不同状态」加根环容量边界），原话在变更史。
- 「根槽这一次读坏」那一行：抬 F 那一半是用户定案，原话在变更史，构造的反例在 `records/2026-09-24-里程碑二收尾调度.md` 第三节「实四丙交回」那一行；回退那一半是主 agent 2026-09-28 定（同一份调度记录第 309 行），经过在变更史。实现已合入：`crates/singlefs-core/src/recovery.rs` 的 `readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once` 对这个进程知道住着根的槽「读不出或自证不过」都重读一次，仍坏报 `RootRingSlotKnownToHoldARootStillBadAfterOneReread`（带 `first_reading` / `reread`）；算上限那一遍是 `crates/singlefs-core/src/mount.rs` 的 `roots_of_the_ring_for_the_ceiling`（仍坏报 `MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`），抬 F 与卸载重算影子账与回收门槛那一遍仍坏报 `MountError::NewerStateStillUnreadableAfterOneReread`（`StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot`），管理员回退的 `rollback_candidate` 判候选与算 F_生效（`recovery.rs` 的 `effective_rollback_floor_rereading_known_ring_slots_and_the_newest_instance_table_once`）仍坏报 `RollbackError::CandidateJudgementStillUnreadableAfterOneReread`、在任何写之前。用例：`crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs` 的 `raising_the_floor_while_a_root_ring_slot_known_to_hold_a_root_stays_not_self_verified_is_refused_before_any_write`、`a_root_ring_slot_known_to_hold_a_root_not_self_verified_once_is_read_on_the_reread_and_the_floor_is_raised`、`rolling_back_while_the_targets_root_ring_slot_stays_unreadable_is_refused_before_any_write`；`crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs` 的 `rolling_back_while_a_root_slot_this_process_wrote_reads_not_self_verified_is_refused_before_any_write` 与 `crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs` 的 `rolling_back_after_the_floor_carrier_this_process_wrote_on_device_one_reads_not_self_verified_is_refused_before_any_write`（这个进程写过的根被改坏之后回退拒、盘上不变）（`research/prompts/m2-impl-r2-fixes-b-implementer-report.md` 第三节第 3 行；合入后验证一收窄成只管读不出、之后由改法 B 还原成全句，见 `research/prompts/m2-impl-merge-verify-1-implementer-report.md` 第三节第 1 小节）。
- 「准入」那一行的落点被拒也推、挂载处写行之后推：用户定案，原话在变更史；安全设计轮第三轮判决 `research/prompts/m2-safety-r3-main-verification.md`（会话里落点取不到 C283（准入失败时不先推发布就报 ENOSPC） 不管、只崩了再挂就把小盘挂死）。
- 回收门槛与同取窗口的另外五处退回环里最旧有效根：判决 `research/prompts/m3-reclaim-window-r1-main-verification.md` 第二节 W1、W3 各行与第六节（窗口门槛在目标形态上出局：丢一块盘之后挂载落到幸存盘上更旧的根，读出来是坏数据，计数模型全枚举；只改分配器门槛或只改 checker 都重开 I-3.1（已分配统计对得上） 误红，副本量过；六处一起退，两腿独立量到零假候选、零不可恢复，被攻过一轮）；用户定案 2026-10-04（Q1 RGall、Q2 下一轮就改），原话在变更史。E139（按盘回退下界的收严形态） 第 1 次重跑：旧门槛对照臂 G2SW_ringgate 判的行零假候选、零不可恢复，加窗口的 G2SW 有非零行；其中一部分是装置的候选下界没带系统配置里的 F 造出来的（判决 W2 行），数不当依据，以实验页为准，装置补 F 之后的第 2 次重跑欠着（欠里记着）。无实验：判决里的数在三方腿的副本与模型上量，没立实验号，不是入库装置上的数。此前取一圈窗口的三方判决（`research/prompts/m3-ring-r2-main-verification.md`、`research/prompts/m3-ring-r3-main-verification.md`）与用户 2026-10-02 的定案，经过与原话在变更史。
- 抬 F 的上限并进挂载那一刻读不出或自证不过两类槽：代码轮第三轮判决 `research/prompts/m3-root-ring-code-r3-main-verification.md` 第二节 K2 的 RS3 行（一槽从挂载起读回全零、读得出而自证不过，根环表不记它；准入抬 F 的上限算成 22，越过盘上完好的非空有效根 21，池级 checker 判 I-7.9（回退下界 F 不高于抬 F 的上限） 红；设备报错那一形停在 21、0 红；副本量过，主 agent 复跑对上）与第六节 Q3 行；用户定案 2026-10-04（读不出与自证不过两类都并），原话在变更史。无实验：数在攻方副本上量，没立实验号。
- 抬 F 那一遍并进挂载时读不出的槽、进程内记根先重读、抬 F 的上限只取窗口里的根（这一半随回收门槛退回环里最旧有效根而作废）：代码轮判决 `research/prompts/m3-root-ring-code-r1-main-verification.md` 第二节 K2-e、K2-b、K2-b2 三行（攻方原型量过，主 agent 复跑 115 份输出逐字节相同）与第三节表二之后用户定案那一段（E2、B2）。
- 进程内重读读得出而自证不过与仍读不出同一处置（2026-10-04 RS1 先定按记的最大 txg 算，2026-10-05 用户定以钉在可能住过的最旧 txg 为准：记最大在 RGall 下回收过 mkfs txg 0 那条有效根引用的单元，`records/2026-09-30-里程碑三开工第一轮.md` 第 10 批 core 簇交回那一条）、重读读出有效根的槽算进这个进程知道住着根的槽、抬 F 的上限把挂载那一刻读不出的槽当作可能是非空有效根：代码轮第二轮判决 `research/prompts/m3-root-ring-code-r2-main-verification.md` 第二节第 2、6 行（有效根所在的槽挂载那一刻读错、重读又读坏，实现当没有根，回收门槛越过盘上有效根，攻方模型上 12 组参数 12 组红，改法 A 12 组 0 红；抬 F 越过读不出的有效根那一形，攻方探针上 F = 22、读得出时 F = 21，推的）与第四节 RS1、RS3 两行（改法 A 的代价是从没写过的槽加挂载读错时记账多，安全方向，2 组；上限算进读不出的槽的代价是读故障时 F 抬得少一点，推的）；用户定案见同一份判决第六节，原话在变更史。无实验：数在攻方模型上量，没立实验号。
- 正常卸载那一串回收的槽只放开一截：设计轮判决 `research/prompts/m3-unmount-after-short-continuation-r1-main-verification.md`（续做推满之后卸载在预演里被拒，丙1 站住、零轮）；代码轮第三轮判决 `research/prompts/m3-unlink-code-r3-main-verification.md` 第二节 N1 各行（丙1 在单盘幸存时写掉被择中的旧根引用的单元，两次抽样同向打中，主 agent 复跑坐实；恢复择根不看 F）与岔路单 `research/prompts/m3-unlink-code-r3-forks.md`。用户定案 2026-10-03：「丙 1 进下一批」、单盘那一问「甲 GA（推荐）」，原话在变更史。无实验：数在三方腿的拷贝上量，没立实验号。
- 有一块盘没有有效根时整截扣、读不出的槽当没有根、卸载半途崩溃之后的下一次可写挂载只放开一截：代码轮判决 `research/prompts/m3-batch14-code-r1-main-verification.md` 第二节（N1「根环里读不出的槽、有洞」那一行是代码取的保守读法；「形 R」那一行攻方打中、主 agent 复跑坐实，`research/prompts/m3-batch14-code-r1-main-checks/`）与岔路单 `research/prompts/m3-batch14-code-r1-forks.md`。用户定案 2026-10-04 选「甲 再挂也只放开一截（推荐）」，原话在变更史。无实验：数在三方腿的拷贝上量，没立实验号。
- 再挂时某块盘一条有效根都读不出只放到最旧有效根（该判决写作一圈窗口里最旧有效根，随回收门槛退回环里最旧有效根）、当场放开的式子带上最旧有效根那一项：代码轮判决 `research/prompts/m3-batch16-code-r1-main-verification.md` 第二节（N1「再挂期间某块盘根环整段读坏」那一行攻方打中、主 agent 复跑坐实，`research/prompts/m3-batch16-code-r1-main-checks/`；窗口那一项是正推腿报的条款漏写）与岔路单 `research/prompts/m3-batch16-code-r1-forks.md`。用户定案 2026-10-04 选「甲 只放到窗口里最旧有效根（推荐）」，原话在变更史。无实验：数在三方腿的拷贝上量，没立实验号。
- 树表 0 条那一版上正常卸载一个字节都不写、F 不动：用户定案 2026-10-04（选乙「不推，条款照实现写」，原话在变更史），把实现里主 agent 2026-09-26 那条定案（`crates/singlefs-core/src/mount.rs` 的 `Unmounted::NothingWrittenOnAVersionWithoutFile` 的注释，推的，被攻过零轮）写进条款；代码轮判决 `research/prompts/m3-formal-code-b16-r1-main-verification.md` 第二节 K1 与第五节第 1 行（照字面推空发布那一读法在攻方副本上量过：模型推 2 条、对拍报 PublishCount）。无实验：数在三方腿的副本上量，没立实验号。
- 「根槽这一次读坏」那一行的钉最旧 txg、全零槽、重建时整环重读一次再拒、按槽并两遍：用户定案 2026-10-04（读坏的槽在回收门槛里钉在可能住过的最旧 txg）与 2026-10-05（全零槽在回收门槛里照自证不过钉住、只在准入抬 F 的上限里不并；重建那一遍重读一次再拒；Q3 另起拒因；Q4 采用 rr2；按槽并两遍），原话在变更史；判决 `research/prompts/m3-batch10-code-r1-main-verification.md` 第二、五、六节、`research/prompts/m3-batch10-code-r2-main-verification.md` 第二、五、六节。E173（上限卡在读坏槽的三种处置） 不撑这一格。无实验：数在三方腿的副本与模型上量，没立实验号。
- 「抬 F 的上限」那一行读坏槽并进上限、读回全零不并：E173（上限卡在读坏槽的三种处置） 的分臂表（照今天不让 F 越过读坏槽里的非空根，推空比它只少拒 4 / 15552 个位置（条款读法），而推空发布有取不到落点的位置；不并让 F 越过 180 个位置；全零不并在条款读法下 F 越过 90 / 7794 个位置、全零并多拒 18 个位置，数以实验页为准）；用户定案 2026-10-05（Q1 照今天、Q2 照今天不并），原话在变更史。
- 池级 checker 与 GPU 判器的 I-7.9（回退下界 F 不高于抬 F 的上限） 重算上限仍取一圈窗口：判决 `research/prompts/m3-batch10-code-r1-main-verification.md` 第二节 K3 与第五节（正推冲突并排第 1、2 行）。

**欠**：C282（环里最旧根没有定义）——定义与条款写在已定项 1 里，会红的检查仍欠；C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元）（与 defer 窗口取多大无关，病根在 D23（journal 的角色与格式） 已定项 14 的影子账）；C329（写行那次发布之前推抬 F 的空发布没有检查）；C563（读根环一槽读坏就当没有根，别的调用方没判）；C660（回收门槛改成一圈窗口之后按盘回退下界的装置要重跑）。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:1-11`（整段抄，未转述）**

```markdown
## D19 块指针的结构与宽度预算 —— 已定

D19（块指针的结构与宽度预算） 管块指针这一格：指针分几段、每段有哪些字段、各多宽、按什么次序排；位置条目与中央映射谁是解引用与释放判定的权威；中央映射条目的 key 与 value 怎么编码；映射 key 的哪几段跟着指针走，出生序号怎么发。不管的：单元头的字段表与单元类型码在 D18（块里携带什么信息）；节点大小与树内条目布局在 D8（核心索引结构）；落点坐标系与分配记录的 key 在 D3（空间分配） 已定项 3 / 7，提交内生块的 bump 次序在 D3（空间分配） 已定项 10；校验和覆盖多大、逻辑 extent 与单元的关系在 D4（校验和位置）；加密算法、nonce 怎么产出、AAD 期望值从哪来在 D9（加密）；根记录与系统配置的字段表在 D22（单元原子性怎么合成） 已定项 7 / 9；条带宽度的粒度在 D2（RAID 条带策略） 已定项 1。

<!-- doc-lint:registry name-col=2 -->

| 编号 | 简称 |
|---|---|
| N1 | 读不出先重读一次 |
| N3 | 对不上的留在已分配 |

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:60-73`（整段抄，未转述）**

```markdown
#### 已定项 3：定宽，不加密时留位

**定案**：指针按最坏情形定宽——头部里的 MAC（128 位）与 nonce（96 位）恒定占位，不加密的卷把它们留成空位，不用类型标记省掉；这些位在所有卷上都占着，也不挪去装别的字段。

**射程**：可选的是 AAD 那道运算（加密关着就不算），不是格式（`.claude/rules/fs-design.md`「加密是可选的运算，不是可选的格式」）。按介质开放布局分支那个机制管不到这一格：incompat 位管的是读者认不认识这套布局，这一格管的是在同一套布局里一个指针内部怎么编码；先定宽、以后再加一套变长布局不是扩张，是多一套要永远维护的解析器加 checker 加崩溃点重放。不加密时白占 MAC 16 字节 + nonce 12 字节 = 28 字节 / 逻辑 extent，按 D4（校验和位置） 已定项 7 的口径（逻辑 extent 占盘恒 32768 字节）是 `28 / 32768 = 0.0854%`。变长那一侧省的是那 28 字节、不是省一个字段（算法类型字段 D9（加密） 已定项 12 第 4 项本来就要有），要付的是解析必须从头走：定宽指针可以按偏移直接定位第 k 个，变长不行；「恢复期扫一个部分损坏的节点时能不能跳过坏掉的那一条继续读」是推论，未实测。 加密关着时 MAC 16、nonce 12 恒 0，读者遇到非 0 判该指针所在的结构损坏，同 I-2.4（头校验和覆盖范围） 给单元头 29 字节的读法（实审 A3a 落地在两处读者——多层码 2 树内部条目、journal 新根段指针：`crates/singlefs-core/src/pointer.rs` 的 `PointerHeadFieldOutsideTheFirstVersion::MacOrNonceNotZero`）；extent 偏移那 2 字节第一版恒 0（第一版指针指整个单元），读者遇到非 0 判该指针所在的结构损坏，与加密开不开无关；checker 在走读跟随的每条指针上判，core 每个读盘上指针头部的读者都判，落点在 [checks-owed.md](../checks-owed.md) 已还清表 C581（extent 偏移非 0 没有判据） 那一行。

**依据**：

- 无实验：定宽换变长省下来的是空间，省掉的是「一个指针单独捡起来能不能直接按偏移解出来」，两边不是同一个量纲，没有可量的量。撑它的是 `.claude/rules/fs-design.md` 两条——「不为省空间牺牲自包含」，以及格式分支的判据（看它能不能把「用错了」变成「挂不上」，且判据里不许出现「这样更快 / 更省」）。
- 用户定案，原话与 2026-09-12 的重申在变更史。
- 判决 2026-10-01 采：extent 偏移第一版非 0 判该指针所在的结构损坏（`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 11 行）。

**欠**：C210（三个「第二套」属于哪一类扩展没人判过）。

```

**出处 `.claude/kb/decisions/09-加密.md:1-4`（整段抄，未转述）**

```markdown
## D9 加密 —— 已定

D9（加密） 管本工程的加密：整卷 AEAD 的方向与明文豁免范围、格式 day-1 要为加密留的位、MAC / nonce / 算法 / AAD 的形态、三条与选型无关的硬要求、无密钥的一侧能做什么、系统配置与根的认证和回滚防护、加密进不进第一个可运行版本。加密是可选的运算、不是可选的格式（`.claude/rules/fs-design.md`「加密是可选的运算，不是可选的格式」）：加密关着时 MAC 与 AAD 不算，为加密留的位所有卷照留、全 0、不挪去装别的字段。不管的：各字段的宽度与偏移——指针归 D19（块指针的结构与宽度预算） 已定项 11，根记录归 D22（单元原子性怎么合成） 已定项 7，系统配置归 D22（单元原子性怎么合成） 已定项 9，journal 记录头归 D23（journal 的角色与格式） 已定项 4，单元头里的 nonce / MAC / 算法类型预留位归 D18（块里携带什么信息） 已定项 14 / 已定项 16；单元类型标签的值域归 D18（块里携带什么信息） 已定项 11 的登记表。调查见 [prior-art.md](../prior-art.md) 第六节。`crates/` 里今天没有任何加密代码（`grep -rniE "chacha|aes|aead|encrypt|decrypt|poly1305|cipher" crates` 零命中）：写路径把这些位写 0，读路径（`PointerHead::read_from`、`SystemConfiguration::parse_slot`）与池级 checker 跳过它们不判。

```

**出处 `.claude/kb/decisions/16-发布语义.md:6-26`（整段抄，未转述）**

```markdown

| # | 分项 | 定案 |
|---|---|---|
| 1 | **defer 窗口取多大** | 按盘回退下界的收严形态、按状态数读：候选集保留最近 4 个可退到的不同状态（去重、以根环装得下为界），可再分配 ⟺ 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)；F 另记进系统配置 **状态：已定。** |
| 2 | **checkpoint 的触发判据** | 时间与脏数据量两者取先到：`触发 ⟺ (now − 上次 checkpoint 的时刻 ≥ T_time) ∨ (脏字节数 ≥ T_dirty)` **状态：已定。** |
| 3 | **journal 在 D15（格式冻结政策） 的冻结分层里记第几层** | 不进四层，立成四层之外的第三个独立冻结组件；新记录格式走记录类型 + 系统配置 log-incompat 位 **状态：已定。** |
| 4 | **记录核对器那第二份 replay 实现** | 一次发布整体施加或整体不施加；第二份实现停在句法层（前缀判定 + 择根 + 续做），在合谋测试里是第四个判决器 **状态：已定。** |
| 5 | **`T_time` 与 `T_dirty` 取多少，丢失窗口的口径** | `T_time` = 5 s、越界只警告；`T_dirty` = 2 GiB、夹取成 min(字段, 环长 ÷ F)；崩溃最多丢 T_time + N_switch × (T_retry + T_redo) **状态：已定。** |
| 6 | **发布计数器** | 每次发布 checkpoint_txg 加一，fsync 触发的也是发布；根槽轮转键、记账的代、重放水位是同一个计数 **状态：已定。** |
| 7 | **发布的持久顺序** | 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）→ 系统配置槽 → 屏障；重放施加前必须逐项验证点名单元 **状态：已定。** |
| 8 | **新实例要不要先让本实例的根落到两块盘上** | 取戊：根覆盖两块盘之前不让 fsync 返回，连推空发布至多 R = 3 次 **状态：已定。** |
| 9 | **一次空发布写不写单元** | 记账树存在时就写：空发布也是发布，按 D5（快照 / 空间记账机制） 已定项 2 重写记账行与连带的四样 **状态：已定。** |
| 10 | **发布粒度** | 攒够一批脏节点（或到时间）再发布一次根，不是每个事务发一次；发布频率只由 D16（发布语义） 定 **状态：已定。** |
| 11 | **能不能绕过 journal 直接发根** | 不能：所有结构、所有操作先进同一个 journal、共用同一个 checkpoint 节奏，fsync 只是提前触发一次发布 **状态：已定。** |
| 12 | **中间状态的合法性语义** | D8（核心索引结构）「中间状态必须每一个都是合法的文件系统状态」改写成「根 + journal 任意前缀，replay 之后合法」 **状态：已定。** |
| 13 | **这份 journal replay 归谁** | 归记录核对器，不归 O2（独立解析器 + checker）；它是第二次实现 replay，是 D16（发布语义） 最贵的一笔 **状态：已定。** |
| 14 | **快照发布点的位置** | 与 checkpoint 边界对齐，钉在这个 checkpoint 全部数据变更之后；同窗两个快照用 `(txg, 快照 id)` 打破并列 **状态：已定。** |
| 15 | **checkpoint C 中产生的释放什么时候进可分配集合** | C 被发布之前不得进入；两条理由，第二条按 journal 位置说，比 checkpoint 边界更严 **状态：已定。** |
| 16 | **checkpoint 与消息缓冲的摊销窗口分界** | 两者买同一样东西，差别只在窗口住内存还是住盘；E7（离线索引 harness） 的对照组钉成「有 checkpoint、无消息缓冲」 **状态：已定。** |
| 17 | **fsync 报错之后调用方能假设什么** | 报错之后这次发布可能已成也可能未成，不许假设可复用旧内存态重试；成没成由调用方自己读现行根判定 **状态：已定。** |

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:13-31`（整段抄，未转述）**

```markdown

| # | 分项 | 定案 |
|---|---|---|
| 1 | **位置条目带不带设备身份** | 带：每个物理位置各带一份 dev id，不引一层 chunk 映射表 **状态：已定。** |
| 2 | **密文校验和的位宽** | 32 位；取完整 CRC32C，不许由更宽的 CRC 截断而来 **状态：已定。** |
| 3 | **定宽还是变长** | 定宽，不加密时留位：MAC 与 nonce 恒定占位，留成空位、不挪作他用 **状态：已定。** |
| 4 | **位置条目各段的宽度** | 设备 ID 4 字节、物理偏移 6 字节（16 KiB 槽号）、密文校验和 4 字节 ⇒ 位置条目 14 字节 **状态：已定。** |
| 5 | **位置的权威住哪** | 位置条目降为提示，中央映射是解引用与释放判定的唯一入口 **状态：已定。** |
| 6 | **中央映射条目的字段表** | key 取按类复用写序键（收严后）：码 1 为 27 字节、码 2 / 码 3 为 25 字节；value 照已定项 4 / 5 **状态：已定。** |
| 7 | **指针头部带哪几段** | 头部 50：MAC 16、nonce 12、算法类型 1、压缩算法码 1、压后长度 2、extent 偏移 2、出生树 8、出生 txg 8 **状态：已定。** |
| 8 | **引用侧携带多少出生身份，映射装哪几类** | 全部指针进映射，指向码 1 的指针 88、指向码 2 / 码 3 的 86；映射树节点、树表单元、实例表单元自举豁免 **状态：已定。** |
| 9 | **出生序号怎么发、按什么次序发** | 树内从 0 起、码 2 与码 3 共用一个计数、换 checkpoint 清零、重写换号；次序同提交内生块的 bump 次序 **状态：已定。** |
| 10 | **映射 key 定宽还是按类变长** | 一律 27 字节，码 2 / 码 3 末尾补零；中央映射条目一律 55 字节 **状态：已定。** |
| 11 | **指针的偏移表、位置条目的顺序、映射树的根住哪** | 段按偏移表排，位置条目按设备身份升序；映射树根住根记录，根记录合计 457 **状态：已定。** |
| 12 | **码 2 / 码 3 的映射条目 day-1 写不写，固定点写几次** | day-1 写出；一个实例在一个 checkpoint 里每个码 2 节点与码 3 容器只写出一次 **状态：已定。** |
| 13 | **块指针分几段** | 分两段：逻辑属性住头部、每逻辑 extent 一份，物理属性住位置条目、每物理位置一份 **状态：已定。** |
| 14 | **位置条目能不能用几何公式从单一起点推算** | 不能，逐个列出 **状态：已定。** |
| 15 | **extent 距起点偏移的值域与第一版取值** | 值域 [0, 32768)、15 位够；第一版恒 0，位照留 **状态：已定。** |

```

**出处 `.claude/kb/checks-owed.md:1-16`（整段抄，未转述）**

```markdown
# 欠的检查清单

**checks-owed.md 放「已经知道要拦什么、但现在还拦不了」的检查。**

与 [invariants.md](invariants.md) 的分界：不变量是**对着一个镜像**可判定的性质，
checker 是它的可执行形式；checks-owed.md 放的是**代码路径的要求**——
它们对着镜像判不出来，只能由门禁在跑的时候拦。

**写进来的门槛**：说得出「拦什么」「怎么拦会红」「现在缺什么前置」。
三样缺一样就是还没想清楚，先别写
（`.claude/singlefs-ai-sop/rules/sop-first.md`：写不出下一步就别加那条检查）。

<!-- doc-lint:registry name-col=2 -->

| # | 简称 | 要拦什么 | 怎么拦（会红的形态） | 前置 | 出处 |
|---|---|---|---|---|---|
```

**出处 `.claude/kb/checks-owed.md:584-584`（整段抄，未转述）**

```markdown
| C735 | GPU 判器里由记录施加的版本没有对拍流与变异行 | GPU 判器 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 里 I-5.4（分配记录罩住的槽互不相交） 按版本判的那一圈，第 34 批 34-1 拆出 `judge_allocation_records_disjoint_of_a_version`，候选根、`R_DATA` 里由记录施加出来的那几版、下一次挂载先施加的那一版各调一次。`R_DATA` 里由记录施加出来的那几版那一次调用，没有一条对拍流造得出「由记录施加出来的版本里两条记录重叠」，也没写变异行，去掉它 GPU 判器照绿（变异下会存活）。下一次挂载先施加的那一版有一条对拍流（`the_gpu_judge_agrees_with_the_cpu_when_two_allocation_records_cover_one_slot_in_the_version_the_next_mount_applies_first`）与一行变异（`crates/mutations.tsv` 里名字带「第34批 34-1 GPU」那一行），两样都在 checker 档、没跑、没证红 | 造一条录制流：一个版本只由记录施加出来（记录已落、根槽没落，或挂载之后只由记录施加），它的分配记录叶里两条记录罩住同一个槽，CPU 判器红，GPU 判器也要红；对拍加这条流，再加一行变异去掉 `R_DATA` 那一次调用，对拍必须由绿转红 | 这一形的录制流今天造不出（34-1 实现员没找到造法）；GPU 对拍与变异在 checker 档（要一张 NVIDIA 独显），归提交时 | 2026-10-06 `records/2026-10-01-里程碑三形式化接入.md` 第 34 批 34-1 交回那一条（「GPU 里由记录施加那一圈没有对拍流与变异行（记欠账）」）；34-1 实现员报告「变异」一节末句与「没做什么」一节最后一条 |
```

**出处 `.claude/kb/checks-owed.md:732-732`（整段抄，未转述）**

```markdown
| C581 | extent 偏移非 0 没有判据 | 条款写进 D19（块指针的结构与宽度预算） 已定项 3：指针头部 extent 偏移非 0 判该指针所在的结构损坏，扩 I-2.4（头校验和覆盖范围） 的指针一半；checker 在走读跟随的每条指针上判（`crates/singlefs-checker/src/image.rs` 的 `judge_pointer_mac_and_nonce_are_zero`），core 每个读盘上指针头部的读者都判（`crates/singlefs-core/src/pointer.rs` 的 `PointerHeadAsRead::judged`，三种指针头的 `read_judging_the_encryption_compression_and_extent_offset_fields_from` 都经它；用例在 `crates/singlefs-harness/tests/pointer_extent_offset_that_is_not_zero_is_refused_by_every_tree_reader.rs` 与 `crates/singlefs-harness/tests/pointer_extent_offset_that_is_not_zero_is_refused_by_both_mounts.rs`），根记录读者 `crates/singlefs-core/src/root_record.rs` 的 `parse_slot` 四条指针与 journal 新根段同一判法（加密、压缩与 extent 偏移那几段，任一非第一版的取值按这条根自证不过处置；第 34 批 34-2 实现，判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P2「F2 与 core」一格与第三节第 1 条第二项；用例 `crates/singlefs-harness/tests/root_record_pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_both_mounts.rs` 四条，只判 extent 偏移那一支 `read_judging_the_extent_offset_from` 随之删掉）；变异行在 `crates/mutations.tsv`（名字带「C581」）；`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 5、6 行；实例表链指针那一行变异与影子账读者那一行的证红还没跑，留给提交时的门禁 checker-tier-crates-mutation-replay | 2026-10-03 |
```

**出处 `.claude/kb/invariants.md:292-292`（整段抄，未转述）**

```markdown
| I-9.5 | 删除队列里的号仍在树里 | 两个方向都要成立。① `deleted_inodes` 里的每个 inode 号在同一棵树的 inode 树里存在且记录的 nlink = 0（记录在回收完成前留在 inode 树里，`deleted_inodes` 只存号）；② inode 树里 nlink = 0 的每条记录，号都在同一棵树的 `deleted_inodes` 里（队列是派生态，队列 = inode 树里 nlink = 0 的号集合，D8（核心索引结构） 已定项 16 第 3 条）。**共用判法**（I-4.6（中间态其余不变量成立）、I-4.7（删除中 inode 有账）、I-3.4（可用空间扣待删占用） 都照这一段，不另写）：判走读到的任一版（候选集里每条根，含由记录施加出来的那几版），不只判最新根；`deleted_inodes` 与 inode 树经树表条目偏移 116 的头 ID 配对（头 ID 的取值见 I-9.27（树表条目的头 ID 取值）），配不上的版记判不了；树表、`deleted_inodes` 或 inode 树读不全的版报 `part_of_the_range_not_judged`（射程里判不了，不判红），这是今天除 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 外各条用的报法，是否改报第四种判定「部分没判」随 C701（别的判法静默跳过读不出的部分） 那一问由用户答，答了这几条随它一起改。树表 0 条的版没有这几棵树，不判；自证过的根树表指针全零照 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 那一行判损坏，这一条报射程里判不了；树表单元相同的几版内容相同，只扫一次；没有删除的版本上 I-9.5（删除队列里的号仍在树里） 报成立（两个方向比的是队列与整棵 inode 树，干净的一版也真比过），I-4.7（删除中 inode 有账） 与 I-4.6（中间态其余不变量成立） 没有对象时报不适用并带理由 | 已实现（2026-10-05，池级 checker `walk::check_pool_image`，删除态扫描是 `judge_deletion_state_of_the_versions`；坏镜像登记在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `.chain`，搭建与改法在 `crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs`，逐形的用例在 `crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs`；GPU 判器下标 69，扫描内核判）。两个方向各有坏镜像，「队列里的号在树里 nlink ≠ 0」那一形也有（C708（树里 nlink = 0 的 inode 不在删除队列里，池级 checker 判不出） 已还清）；`deleted_inodes` 的形态已定在 D8（核心索引结构） 已定项 16；判法与读法的出处 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节 |
```

**出处 `.claude/kb/invariants.md:303-303`（整段抄，未转述）**

```markdown
| I-9.16 | 树表条目按树 ID 严格升序且合发号次序 | 同一张树表（D8（核心索引结构） 已定项 8 的字段表）里，① 盘上相邻两条的树 ID 严格升序；② 树表里有的种类按发号次序（extent、inode、分配记录、记账、livelist、稀疏旁表、deadlist、`deleted_inodes`、墓碑树、谱系树、logged_ops；中央映射树不进树表）排，相邻两棵的树 ID 严格升序；③ 每一种种类码（登记过的与没登记的都算）至多一条；④ 树表非空时进树表的十一种（② 那一串）各恰好一条，读者重建一版时十二棵树的号逐棵取，缺哪一种都重建不出这一版（报「树表里没有那棵树」）。四道任一道不成立即违例；0 条的树表四道都成立，1 条的前三道成立。依据 D8（核心索引结构） 已定项 8「排序契约」与它的 ②「十二棵树的号从水位起连号发，次序照已定项 11 那一组 65536..65547」。判别力落在 ②：① 与 checker 在树表上判的 I-1.1（块头自述逻辑地址） 那组 key 严格递增是同一个谓词，只违反 ① 的镜像两条一起红；③ ④ 各有只违反它的坏镜像（一种出现两次、缺一种） | 已实现（2026-09-27，core `crates/singlefs-core/src/recovery.rs` 常量 `TREE_TABLE_ENTRIES_ORDERING_CONTRACT`，① ② 合成一个入口，`rebuild_version` 与 `walk_to_file` 在读任何一棵树之前拒；③ 是 `tree_table_entries_each_kind_at_most_once`，④ 由 `rebuild_version` 逐种取号时报「树表里没有那棵树」、`walk_to_file` 报「树表里缺一棵有根的树」；池级 checker `crates/singlefs-checker/src/walk.rs` 另写一份：`judge_tree_table_entries_ordering` 判 ① ②，`judge_tree_table_entry_kind_counts` 判 ③ ④（④ 的十一种是 `TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES`），违例写明哪一版根与哪两条或哪一种，列进 `image.rs` 的 `IMPLEMENTED_INVARIANTS`；GPU 判器走读内核 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 的 `w_judge_tree_table_entries_ordering` 同判四道，② ④ 的循环十一种；坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs`「互换两条的种类」那份只红 I-9.16（树表条目按树 ID 严格升序且合发号次序）；只违反 ① 的钉在 `tests/tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs`；③ ④ 的坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，用例名以 `a_tree_table_whose_` 与 `a_tree_table_without_its_` 打头（livelist 或 logged_ops 那一条改成第二条 extent、deadlist 那一条改成第二条分配记录、分配记录条目种类改成没登记的码、整条去掉分配记录 / deadlist / logged_ops 那一条），各只红 I-9.16（树表条目按树 ID 严格升序且合发号次序），去掉 logged_ops 那一条的一份走读也红；实现员报告 `research/prompts/m2-rev-tree-table-ordering-implementer-report.md`） |
```

**出处 `.claude/kb/invariants.md:315-315`（整段抄，未转述）**

```markdown
| I-9.28 | 全部可读根里同一树 ID 的诞生 txg 相同 | 根环里全部读得出且自证合法的根（不按实例表判有效与否，含被崩溃恢复抛弃的旧线上的根），任两条根各自的树表里同一树 ID（条目偏移 0）的条目，诞生 txg 必须相同；只有这棵树第一次出现在树表里时才设这个值（D8（核心索引结构） 已定项 8（树表条目的字段））。判定四种：有一处诞生 txg 对不上 → 不成立；各处都对得上，但有一条读得出、自证过的根树表读不出（含树表指针全零）、或条目比字段表短 → 部分没判，列出这些根（盘号、实例、txg、为什么没判）；没有一棵树出现在两个不同的树表单元里（几条根共用一个树表单元时拿同一份字节比自己，是恒真判定）→ 不适用；其余 → 成立。树表指针全零的根不是「没有树表」：core 的根记录读者不把全零当特例，照它那两条位置条目读盘 0 槽 0、读不出（可写挂载拒成 `Recovery(UnitUnreadable { slot: SlotNumber(0) })`），池级 checker 对根环里自证过的这样的根照 core 跟过去、判这条根损坏（红 I-2.1（校验和与内容匹配），最新根再红 I-4.8（近 K 代根校验和自洽）、I-7.2（最新根可完整遍历）、I-7.4（近 K 代块未被复用），别的候选根再红 I-4.8（近 K 代根校验和自洽）、I-7.4（近 K 代块未被复用）），这一条把它与树表读不出同一处置、报部分没判并点名「树表指针全零」（用户 2026-10-05 弹窗定，选项原文「甲 checker 判它损坏」，`records/2026-10-01-里程碑三形式化接入.md` 第 33 批「第 32–33 批代码轮第一轮判决」那一条，判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第五节第 1 条）。只判根环里自证过的根：由记录施加出来那一版里的树表指针全零，主走读仍当这一版没有树表、不记走读失败（用户定的只是自证过的根，要不要同判没定）；不判 I-2.5（位置条目按设备身份升序）：那一行把整条全零的指针整条豁免，豁免要不要收窄没定。GPU 判器的走读对根记录里全零的树表指针没有跟过去，这种镜像上 CPU 红这几条而 GPU 不红；GPU 对拍的流里没有造这种根的。与 I-9.14（树表条目的诞生 txg 跨根不变） 的分工：I-9.14（树表条目的诞生 txg 跨根不变） 只比同一条时间线上的根，这一条不分时间线、比全部可读根。依据：树 ID 被重发（C342（树 ID 水位在根读不出时退回去重发））时，新线上的树与老线上同一个 ID 的树诞生 txg 不同，I-9.14（树表条目的诞生 txg 跨根不变） 的同线收窄看不见这一形。推翻条件：任一合法历史（池级 checker 与层 0 其余判定都不红）上红。 | 已实现（2026-10-04，欠账来源 C342（树 ID 水位在根读不出时退回去重发）；池级 checker `walk::check_pool_image` 的 `judge_tree_identifier_birth_txg_across_every_readable_root`，四种结局；GPU 判器的扫描内核挨着 I-9.14（树表条目的诞生 txg 跨根不变） 判、只置违例位；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，合法历史、部分没判、不适用、对不上照报违例的用例在 `crates/singlefs-harness/tests/pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root.rs`，树表指针全零的根报部分没判并点名的用例在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `an_all_zero_tree_table_pointer_in_an_older_candidate_root_is_followed_like_the_core_and_every_tree_table_reading_names_it_unjudged`；`crates/mutations.tsv` 里以「第9批」「第9批改钉」打头、点名 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同）或第四种判定的各行，与以「第 9 批 9-9 GPU 判器」打头、点名 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同）的 3 行）。用户 2026-10-04 定新立，同时删 I-9.14（树表条目的诞生 txg 跨根不变） 判别力第 ③ 条与它的正向用例，判决 `research/prompts/m3-sweep-and-tree-id-r1-main-verification.md` 第六节 S2-1 行。被攻过零轮：攻方三段合法历史上 0 红（一次抽样），被切掉旧线的镜像上红。部分没判：用户 2026-10-04 定「树表读不出算部分没判」、加第四种判定；四种结局的细则主 agent 定，零轮，`records/2026-09-30-里程碑三开工第一轮.md` 第 9 批「用户 2026-10-04 弹窗定（9-6 交回的两问）」那一条 |
```
