# m3-batch22-code-r1 云端正推（Sonnet）报告：K1 条款对实现（N1–N12）、K3 十二份各一格

<!-- doc-lint:not-numbers N1 N2 N3 N4 N5 N6 N7 N8 N9 N10 N11 N12 K1 K2 K3 T2 T3 T4 T5 T6 T7 T8 T10 T11 T14 M1 M2 M3 M4 M5 M6 M7 M8 C9 -->

立场：正推（从条款、判决格、用户定案推到冻结副本的行，再推到钉它的用例与变异行）。被引的 kb 行号取 `.claude/kb/invariants.md` 今天的行；代码行号取冻结副本 `/tmp/claude-1000/m3-batch22-code-r1/tree/` 里的全路径行号，下文省去前缀只写 `crates/…`。

## 零、核对与过程里的两件事

1. 冻结副本十二份对 `research/prompts/m3-batch22-code-r1-snapshot/crates-sha256.txt`：在冻结副本根下 `sha256sum -c`，十二行全 OK（`12 OK`）。主工作区今天有三份已与清单不同（`crates/mutations.tsv` 等，别的线之后又改过），不影响按冻结副本判。
2. 线程上限：派发之后收到两次改动（改成 6、又改回 10）。第一次来在我读完 `walk.rs` 之后、起第一条 cargo 之前，之后的命令按 6；第二次来在重跑目录里第一次调用（`baseline`、`0` 两项）之后，之后的命令按 10（`run-one.sh` 的 `THREADS` 默认 10）。结果与线程数无关（`SUMMARY_MATCHES` 两边相同）。
3. 内存包装：这个会话里主 agent 已有一条日常在跑，包装退 252 拒我；我设了 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON`（派出去的腿各跑各的）。编译上限 6G 超过峰值表里实测峰值的 2 倍，包装退 251，我设了 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON`（派发提示点名 6G 不写 auto）。两个理由都写在 `research/prompts/m3-batch22-code-r1-sonnet-model/run-one.sh` 里。整机负载 35 上下，别的 cargo 在跑，我的每条命令都加 `nice -n 19`。

## 一、K1：逐条（N1–N12）

### N1　`IMPLEMENTED_INVARIANTS` 74 → 83、GPU 覆盖表登记九位

- 一致：数组类型 `crates/singlefs-checker/src/image.rs:94` 写作 「pub const IMPLEMENTED_INVARIANTS: [&str; 83] = [」（`crates/singlefs-checker/src/image.rs:94`）；下标 74–82 九项在 `crates/singlefs-checker/src/image.rs:169` 到 `crates/singlefs-checker/src/image.rs:177`，次序是 I-9.17、I-9.18、I-9.19、I-9.20、I-9.21、I-9.22、I-9.23、I-9.24、I-9.29（第一项 「LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,」（`crates/singlefs-checker/src/image.rs:169`），末项 「ONE_OBJECT_BIRTH_PER_INODE_NUMBER,」（`crates/singlefs-checker/src/image.rs:177`）），与判决 T13 那一格「从下标 74 起往末尾加」同序。
- 一致：GPU 覆盖表末尾追加 「74, 75, 76, 77, 78, 79, 80, 81, 82,」（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:79`），同文件的文档注释写明内核这一批不置这九位（第 71–74 行），与用户 2026-10-03「GPU 判器全覆盖、这一批先登记」同口径；扫描内核没改，GPU 判红位与 CPU 对拍在这九位上会对不上，是题面写明第 23 批补的，不算打中。
- 不一致（kb 侧，不是代码缺陷）：`.claude/kb/invariants.md` 今天 「池级 checker（`walk::check_pool_image`）判 74 条」（`.claude/kb/invariants.md:12`）与 83 不符；九行状态列仍写 「未实现（三·3 实现）」（`.claude/kb/invariants.md:304`，同写法在 305–311、316 行）；文件头 「今天只有 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同）报它」（`.claude/kb/invariants.md:6`）也过期（I-9.18 等新条与删除线旧条今天都走 `part_of_the_range_not_judged`）。判决 T13 写明「同批同步 `invariants.md` 文件头的条数与 `image.rs` 那段文档」，`image.rs` 那段文档已改（`crates/singlefs-checker/src/image.rs:88`–`92` 区），kb 头与状态列这一份快照里没改。
- 推翻条件：冻结副本 `image.rs` 数组下标 74–82 不是这九项或次序不同；或覆盖表缺一位。

### N2　I-1.7 类型 1、类型 5 两种记录宽的坏镜像

- 一致：`crates/singlefs-harness/tests/checker_known_bad_images.rs:9240` 起的 `a_lineage_container_whose_record_width_is_not_sixteen_reddens_only_the_packed_container_invariant` 与它前一条墓碑宽 39 的用例，断言都写 `["I-1.7"]`（红集合恰好等于目标），不是「包含」。条款 「类型 1（墓碑）是 40、类型 5（谱系）是 16」（`.claude/kb/invariants.md:32`）两种宽各一份。
- 变异行（变异行号 = 改动原样里 c 的 20 行 + 1 行替换 + t 的 3 行，按那份文档的次序数成 1–24）：第 5、6 行（`PACKED_TYPE_LINEAGE => Some(16),` 改 15、`PACKED_TYPE_TOMBSTONE => Some(40),` 改 39），锚点在冻结副本 `walk.rs` 各命中 1 次（我用脚本数过，见第三节），改坏的就是名字说的那一处。
- 推翻条件：把登记宽改成别的值而这两条用例仍绿。

### N3　I-9.10 墓碑那一个等号：按号认、每版各比自己的

- 一致：`crates/singlefs-checker/src/walk.rs:9690` 「judgements.judge("I-9.10", record.object_birth == tombstone.object_birth, || {」（`crates/singlefs-checker/src/walk.rs:9690`）；循环（9683–9689）取这一版墓碑树里每条墓碑、不看谱系、按 `records.get(&tombstone.object)` 在同一版 inode 树里找号，找不到就跳过（删完之后记录不在，无对象）。字面 「该对象的墓碑记录按 inode 号认，走读到的每一版各比这一版自己的」（`.claude/kb/invariants.md:297`）对上。
- 用例：`crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs:1143` 的谱系外墓碑出生代减一，断言 `vec!["I-9.10"]` 恰好；变异行第 15 行（`>=`）。
- 规则没说：墓碑树读不全、头 ID 配不上单棵 inode 树时这一个等号静默不判（`walk.rs:9260` 区文档写明与数据单元那一半同口径、归 C701），条款文件头说「不适用必须带理由，不许静默算成立」——这里不是算成立，是不评估，且已有欠账号；不当打中。
- 推翻条件：把等号改成 `>=` 而用例仍绿（第 15 行证它红）。

### N4　I-9.17、I-9.23：身份不符单独判、不再记走读失败；类型段不认得先登记引用再判红；重合小句留合取

- 一致（读码）：类型段 = 本树记录类型的一支 `crates/singlefs-checker/src/walk.rs:2260` 「let container_entry_holds = unit[6] == 3」（`crates/singlefs-checker/src/walk.rs:2260`）起合取 「子头四段与身份引用相等、子头记录类型等于树的记录类型、子指针出生树等于身份引用出生树」（2261–2263 行），三小句与 `invariants.md` I-9.17 那一行逐句对得上；类型段 0 一支 「&& child_class.is_none_or(|class| class == 2);」（`crates/singlefs-checker/src/walk.rs:2133`），孩子读不出时这一小句不判（留给 I-2.1 与走读失败，同 I-9.2 先例）；类型段不认得一支先 `note_references_of_a_pointer_followed_by_the_walk` 再 `judge(identity_invariant, false, …)`，孩子占槽数 「let span_in_slots = if node.level >= 2 { 1 } else { 2 };」（`crates/singlefs-checker/src/walk.rs:2162`）。身份判定放在 `visited_units.insert` 之前，所以每一版走到都判一次，不被别的版本已走过的容器吞掉。
- 用例（红集合恰好）：`checker_known_bad_images.rs` 里墓碑条目身份、墓碑类型段 1→7（9163 行起）、谱系条目身份三份，断言分别是 `[I-9.23]`、`[I-9.23]`、`[I-9.17]`，注释写明改之前连带 I-4.8、I-7.2、I-7.4（题面 K2 的 N4 那一问由攻方腿攻，这里只核断言写法）。
- 不一致（覆盖）：第 1、2 行变异（`let container_entry_holds = unit[6] == 3` 前加 `true ||`）在冻结副本里是**同一处**文本、同一替换，只是必须红的用例分别是墓碑与谱系那一份；两行名字各说一棵树，改的却是两棵树共用的一行。不算错，写明。
- 不一致（覆盖）：类型段 0 那一支（根长高之后指向码 2 节点，字面「身份引用的出生树、容器号、出生代三段为 0」）与谱系侧类型段不认得那一支没有坏镜像、没有变异行；探针 M1（把类型段 0 的 `entry_holds` 恒真）四个相关测试二进制全绿，见第二节。
- 推翻条件：身份不符的镜像上仍红 I-4.8 / I-7.2 / I-7.4 / I-3.1（题面 K2 那一问，攻方腿的格）；或类型段 0 一支被改坏而有用例红（M1 说没有）。

### N5　I-9.18–I-9.20：每版从自己的 txg 与树表里的谱系树起算；版本清单加 `checkpoint_txg`；不按已发布滤

- 一致（谱系链）：`crates/singlefs-checker/src/walk.rs:9105` 「let mut txg_walked_to = version_txg;」（`crates/singlefs-checker/src/walk.rs:9105`）从这一版自己的 txg 起；记录按 T_rb 从大到小过，T_rb 大于此刻走到的 txg 的跳过 「if record.rollback_txg > txg_walked_to {」（`crates/singlefs-checker/src/walk.rs:9108`），否则 [T_rb, 此刻] 整段在链上并跳到 T_old，末了 「segments_on_the_chain.push((0, txg_walked_to));」（`crates/singlefs-checker/src/walk.rs:9114`）。字面 「有谱系记录 (T_rb, T_old) 的那个 txg（T_rb）的父是 T_old，其余 txg 的父是 txg − 1」（`.claude/kb/invariants.md:305`）：逐段推过，T_rb 自己在链上、其余 txg 连号，对得上。谱系记录 T_old ≥ T_rb 或同一个 T_rb 两条交回 `LineageRecordsInconsistent`（判不了，不判红）：条款没写这两种病态记录，归「规则没说」。
- 一致（版本清单）：两处构造都带 txg，根取 「checkpoint_txg: root.checkpoint_txg,」（`crates/singlefs-checker/src/walk.rs:10505`），记录施加出来的取 「checkpoint_txg: version.checkpoint_txg,」（`crates/singlefs-checker/src/walk.rs:10515`）；墓碑与谱系那一遍按（树表首位置设备、槽、校验和、txg）去重 「version.checkpoint_txg,」（`crates/singlefs-checker/src/walk.rs:9283`）——树表相同而 txg 不同的两版各扫一遍。
- 一致（不按已发布滤）：`walk.rs` 的 9076–9872 区间里「已发布」只出现在 9260 行的文档注释（`awk` 数过，只此一处），墓碑没有已发布谓词。
- 一致（I-9.18 三要素）：队列里的号 N 配墓碑要求 「tombstone.object_birth == record.object_birth」（`crates/singlefs-checker/src/walk.rs:9662`）、「&& tombstone.covers_the_whole_object()」（`crates/singlefs-checker/src/walk.rs:9663`）、「&& lineage_chain.holds_txg(tombstone.death_txg)」（`crates/singlefs-checker/src/walk.rs:9664`）。N 没有记录交回 `QueuedInodeWithoutARecord` 报判不了（9647 行起），与 I-9.18 那一行状态列「对『队列里的号没有记录』那一形报判不了而不是红」一致；队列空的一版直接返回（9609 行），整轮都空时报不适用并带理由。
- 一致（I-9.19、I-9.20）：I-9.19 在谱系上的墓碑配同号同出生代记录 「record.nlink == 0,」（`crates/singlefs-checker/src/walk.rs:9720`）；I-9.20 不重号只判在谱系上的（9771 起），区间与死亡 txg 判全部墓碑 「tombstone.object_birth <= tombstone.death_txg && tombstone.death_txg <= version_txg;」（`crates/singlefs-checker/src/walk.rs:9759`），每片 ≤ 815 留作合取（9745 行）。三句与判决 T14 与用户「补」一致。
- 用例：两段合法历史 `…pool_checker_judges_the_deletion_state_of_every_walked_version.rs:1219`（删 → 回退到意图之前 → 再回退到删完）、`…:1260`（删 → 回退 → 再删）断言零违例，并且 I-9.18、I-9.19、I-9.20、I-9.29 真被评估且成立（`verdict_of` … `Holds`）。违反字面的坏镜像：谱系外墓碑（1178 行起，只红 I-9.18）、活对象（只红 I-9.19）、区间不整（只红 I-9.20）、死亡 txg 0（1089 行起，只红 I-9.20）、重号（只红 I-9.20），断言都是 `vec![…]` 整集合。
- 变异行：第 7–14 行（父链减一起算、在谱系上恒真两行、不看谱系、nlink ≤ 1、区间止、死亡 txg 不比出生代、重号放过），每行必须红的用例见 `mutations.tsv` 对应行。
- 不一致（覆盖）：条款字面里有四句没有用例钉，我在冻结副本的拷贝上各改坏一处跑四个相关测试二进制，全绿：M3（I-9.20 不比「死亡 txg ≤ 这一版的 txg」）、M6（I-9.18 不比对象出生代相等）、M7（I-9.18 不比区间整对象）、M8（去重键去掉 txg）。M3 对应「用户定『补』」那一句的上界一半（变异第 13 行只钉了下界「对象出生代 ≤ 死亡 txg」）。原样输出与复跑在第二节。
- 推翻条件：谱系链走法与上面逐段推的不同（例如 T_old 之后漏掉别的记录）；或「删 → 回退 → 再删」类合法历史上任一条红（攻方腿的格）。

### N6　I-9.21、I-9.22：带旋钮值的新入口，旧入口当没装旋钮；坏镜像用「旋钮写出的合法镜像 + 与写时不同的旋钮信号」

- 一致：入口两个（`crates/singlefs-checker/src/walk.rs:9877`、9885 行起）：旧入口传 `None`；`None` 判 I-9.21、阈值 「None => 2,」（`crates/singlefs-checker/src/walk.rs:9810`）两片以下没有对象，`Some` 判 I-9.22、阈值 「Some(_) => 1,」（`crates/singlefs-checker/src/walk.rs:9811`）。字面 「判回退候选集里的每一版，墓碑树零根或只一片容器（没有非最右的容器）报默认不适用」（`.claude/kb/invariants.md:308`）与 「判回退候选集里的每一版，墓碑树零根报默认不适用（只一片容器照判 ≤ 旋钮值）」（`.claude/kb/invariants.md:309`）两支都对上，与判决 T6 同日改准的那一格（I-9.22 只在零根报不适用）一致。
- 用例：`crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs:393`（旋钮压到 1 写出的镜像交旧入口，只红 I-9.21，带旋钮值 1 全绿且 I-9.21 报不适用）、`…:451`（压到 2 写出的镜像报旋钮值 1，只红 I-9.22，报 2 全绿）；旧的「池级 checker 全绿」四处断言改为带旋钮值入口（`…:51` 常量 1）。变异第 18、19 行。
- 不一致（覆盖）：容器号沿叶序严格递增那一句（9818 行 「judgements.judge(invariant, left.container < right.container, || {」（`crates/singlefs-checker/src/walk.rs:9818`））没有用例钉：探针 M5 恒真，四个测试二进制全绿。
- 不一致（文字）：正文 N6 行的落点列了 `crates/singlefs-checker/src/lib.rs`，改动原样里没有 `lib.rs` 的 hunk（用例从 `singlefs_checker::walk` 引入新入口）。不影响结论。
- 推翻条件：旋钮压小的合法镜像带旋钮值入口判红，或产品镜像旧入口判红（攻方腿 K2 的 N6 那一问）。

### N7　I-9.24：每条 `deleted_inodes` 条目都判、读队列提到配对之前、干净的一版报成立

- 一致：`crates/singlefs-checker/src/walk.rs:8594` 「let queue_read_back = deleted_inodes_queue_without_judging(」（`crates/singlefs-checker/src/walk.rs:8594`）在配对 inode 树（8620 行）之前；对每条 `deleted_inodes` 条目判 「queue.len() <= 1,」（`crates/singlefs-checker/src/walk.rs:8603`），读不出报判不了；树表读不出整版判不了的支把 I-9.24 并进列表（8491 行）；整轮没有条目时报不适用（8514 行起）。字面 「每条 `deleted_inodes` 条目都判，不管树表条目的头 ID 配不配得上，干净的一版报成立」（`.claude/kb/invariants.md:311`）对上。「干净的一版报成立」：空队列一版 `judge(…, 0 <= 1)` 评估过，报成立；用例 995 行起的干净镜像断言 `Holds`。
- 用例：`…pool_checker_judges_the_deletion_state_of_every_walked_version.rs:995`（两个删除中的号，手改三处使别的条都不红），断言 `vec![I-9.24]`；变异第 16 行（`<= 2`）。
- 疑点（读码，未跑）：队列读成 `BTreeSet<u64>`（`crates/singlefs-checker/src/walk.rs:8261` 「) -> Result<BTreeSet<u64>, DeletionScanGap> {」），同一个号写了两条的 `deleted_inodes` 根在这里折成一条、`queue.len() <= 1` 成立。字面 「至多一条 key」 对同号两条算一条 key 还是两条条目，条款没写；别的条（节点内 key 严格升序一类）是否另判这一形我没查到。题面 K2 的 N7 一问归攻方腿，这里只记下读码所见。
- 推翻条件：队列读不出或头 ID 配不上时 I-9.24 报了红而不是判不了；或干净的一版报成了不适用。

### N8　I-9.4 第四句

- 一致：`crates/singlefs-checker/src/walk.rs:2503` 「self.judgements.judge("I-9.4", inode > previous_inode, || {」（`crates/singlefs-checker/src/walk.rs:2503`）在走叶容器时逐条判，前一条号是这一片容器里的（每片容器从 `None` 起），严格大于。字面 「同一片打包记录类型 2 容器里记录号沿记录序严格递增」（`.claude/kb/invariants.md:291`）。
- 用例：`checker_known_bad_images.rs:9261`（叶 [1, 2] 改成 [1 nlink 0, 1 nlink 1]、队列空），断言 `["I-9.4"]` 并且违例说明含 「inode 1 之后是 inode 1」；变异第 4 行（`>=`）。
- 不一致（覆盖）：判决 T11 那一行写「S4a 与两种互换只红它自己」，用例只有 S4a（同号两条）一形；倒序（`inode != previous_inode` 放过倒序）探针 M4 四个测试二进制全绿，说明没有倒序的坏镜像钉第四句的「递增」那一半。
- 推翻条件：倒序叶镜像上不红 I-9.4（探针说明用例钉不到，不等于实现错）。

### N9　I-9.29：候选集各版合起来同号只许一个出生代；水位文件用例一、用例二按条款改断言

- 一致：跨版本表 `object_births` 在 `judge_tombstones_and_the_lineage_of_the_versions` 里建一张、各版共用（见 9271 行 `ObjectBirthsOfEachInodeNumber`），每一版的 inode 记录出生代进表（9426–9441），每一版谱系上墓碑的出生代也进表（9526–9541，用 `lineage_chain.holds_txg` 滤），全部扫完每个 (inode 树, 号) 判一次 「judgements.judge(ONE_OBJECT_BIRTH_PER_INODE_NUMBER, births.len() == 1, || {」（`crates/singlefs-checker/src/walk.rs:9861`）。字面 「回退候选集里各版合起来（含由记录施加出来的那几版）同一个 inode 号只许对应一个对象出生代：每一版的 inode 记录的出生代与那一版谱系上墓碑的出生代都算」（`.claude/kb/invariants.md:316`）：两样出生代、各版合起来、含施加出来的版（版本清单含它们，10509 行起）都对上。「最新根的实例表读不出…记判不了」那一句代码没有（题面写明第 23 批补，不当打中）。
- 用例：`crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs:123` 与 `…:313` 把断言从 `vec!["I-9.6"]` 改成 「vec!["I-9.6", ONE_OBJECT_BIRTH_PER_INODE_NUMBER],」（`crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs:123`）；新增 347 行起「压回的正是发过的最大号」那一形，断言 「vec![ONE_OBJECT_BIRTH_PER_INODE_NUMBER],」（`crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs:376`）整集合恰好，并核违例说明点名旧出生代。用例一、用例二的 I-9.29 红法是条款字面的结果（回退候选集里更早的版还留着旧对象），不是新判例。
- 变异行第 20 行（`births.len() == 1` 改 `!births.is_empty()`）：我在冻结副本拷贝上复跑（探针 C9），水位文件三条用例全红（`FAILED_TESTS` 三行，见第二节）。
- 不一致（覆盖）：谱系上墓碑那一侧（`crates/singlefs-checker/src/walk.rs:9534` 「.entry(tombstone.object_birth)」（`crates/singlefs-checker/src/walk.rs:9534`））没有用例钉：探针 M2 把这一块整个跳过，四个相关测试二进制全绿。三条红的用例红的都是 inode 记录那一侧；一个「号被重发、旧对象只有墓碑留着」的镜像没有。
- 推翻条件：一段合法历史（含崩溃恢复落到旧根之后重发）上 I-9.29 报成立以外的东西——这一问交攻方腿；我只核代码读法：产品路径回退时水位取 `ring.max(current.inode_number_watermark())`（`crates/singlefs-core/src/mount.rs:6562`），回退本身不会重发被抛弃分支上的号。

### N10　`DELETION_QUEUE_SCAN_INVARIANTS` 注释改准；`writable_mount_of_a_formatted_pool.rs` 不适用表与条数

- 一致：注释 `crates/singlefs-checker/src/walk.rs:8206`–`8209` 现写 extent 树那一侧三种（指着这棵 inode 树的不是恰好一条、读不全、头 ID 配不上）只记 I-4.7、I-4.6；代码里对应两处 `part_of_the_range_not_judged` 只列这两条（8685–8690、8734–8739 行），I-9.5 照判。与 m3-batch21-code-r1 判决第二节 K1 那一行的小出入对上。
- 一致：`crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs:584` 「const NOT_APPLICABLE_WITHOUT_FILE: [&str; 45]」（`crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs:584`）、`…:638` 「const NOT_APPLICABLE_RIGHT_AFTER_MKFS: [&str; 49]」（`crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs:638`）、`…:703` 「const NOT_APPLICABLE_WITH_ONE_FILE_VERSION: [&str; 16]」（`crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs:703`）；83 − 45 = 38 与文档里「其余 38 条」一致，前两张表各追加九条（含 I-9.24、I-9.29），有一个文件版本之后追加七条（I-9.17–I-9.23），与 `walk.rs` 里「走读到的每一版墓碑树都是零根」等不适用理由、以及 I-9.24 / I-9.29 在有文件之后被评估这一设计一致。主 agent 说合入验证里这个二进制全绿，我自己的复跑没有含它（见「没做什么」）。
- 推翻条件：九条里任一条的不适用或成立落在表的另一侧。

### N11　m3-batch21-code-r1 判决第三节三条

判决第三节原文：「1. N7：G7 纯算用例加一行变异（批数封顶 32 那一刀）。」（`research/prompts/m3-batch21-code-r1-main-verification.md:38`）、「2. N5：G5 比「盘 1 上最新的根之后每一版换下的槽」；`slots_released_by_the_current_version` 文档改准；W2 那一刀进变异表。」（`research/prompts/m3-batch21-code-r1-main-verification.md:39`）、「3. 正推腿三处小出入：`DELETION_QUEUE_SCAN_INVARIANTS` 注释改准；`history.rs` 两处过期行号；`DEVICES_OF_THE_TEST_POOL` 断言等于池盘数。」（`research/prompts/m3-batch21-code-r1-main-verification.md:40`）。

- 第 1 条达到：`crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs:325` 起 「const DATA_UNITS_REMAINING_JUDGED_WITHOUT_A_POOL: [u64; 7] =」（`crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs:325`）七点到 `1 << 40`（326 行），用例 335 行起直接比 「next_unlink_step(data_units_remaining, UnlinkBatchSize::OneExtentLowerLeaf),」（`crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs:341`）；变异行第 22 行（批数封顶 32 那一刀，锚点 `data_units_remaining <= data_units_per_batch` 附近）在冻结副本里命中 1 次。取点：批数封顶在 N、N < 2⁴⁰ ÷ 144 的实现在 2⁴⁰ 那一点分叉，用例文档写明。
- 第 2 条达到：用例改成按「盘 1 上最新的根之后每一版」收槽 「.filter(|(released_by, _)| *released_by > newest_root_on_device_one)」（`crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:733`），并断言恰有最后两次覆盖写那两版、各自换下过槽 「(CheckpointTxg(current_txg_before_the_unmount.0 - 1), false),」（`crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:741`）；`slots_released_by_the_current_version` 的文档改写成「盘 1 单独恢复择到的根可以就是那一版，也可以早它一个或两个 txg」（611 行上方）；W2 那一刀是变异行第 23 行。
- 第 3 条达到两处、一处没达到：`DELETION_QUEUE_SCAN_INVARIANTS` 注释（N10）与盘数断言 「(DEVICES_OF_THE_TEST_POOL, DEVICES_OF_THE_TEST_POOL),」（`crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs:60`）达到，变异行第 24 行钉它；**`history.rs` 两处过期行号没改**：冻结副本里还写着 「`crates/singlefs-core/src/transaction.rs` 第 506 行」（`crates/singlefs-harness/src/history.rs:1695`）与 「已定项 9 在 `.claude/kb/decisions/16-发布语义.md` 第 192 行。」（`crates/singlefs-harness/src/history.rs:1696`），而 `crates/singlefs-core/src/transaction.rs:506` 是 「let spacing = u64::from(self.parameters.geometry.fixed_structure_slot_spacing);」（`crates/singlefs-core/src/transaction.rs:506`），`.claude/kb/decisions/16-发布语义.md:192` 是「fsync 等系统配置轮换持久之后才返回」那一条。`history.rs` 不在这一轮被判的十二份里，实现员两份报告里也没出现 `history`（`grep -n 'history\|过期行号'` 零命中）。正文 N11 行只点了前两项加「盘数断言」，没点 `history.rs`，所以这里记「判决第三节第 3 条的一个小项未做」，由主 agent 定算不算。
- 推翻条件：第 1 条：批数封顶 32 的实现在 `next_unlink_step` 上仍通过用例；第 2 条：W2 在这份用例上仍绿（攻方 m3-batch21 量过红）；第 3 条那一项：`history.rs` 那两处行号在冻结副本里指对了。

### N12　变异表追加 23 行、替换 1 行

- 一致：改动原样里 c 追加 20 行、c 替换 1 行、t 追加 3 行（脚本数出 24 条可解析的行），每条「原文」在冻结副本对应文件里恰好命中 1 次（锚点没腐化，见第三节命令与输出）。逐行读「原文 / 替换文」与名字：除上面 N4 说的两对共用一处文本的（行 1、2 与 9、10）外，每行改的就是名字说的那一处；行 3 的名字写 I-9.23 而锚点是两棵树共用的 `span_in_slots`，只有墓碑那份用例钉它。
- 我只复跑了对照一行（第 20 行，探针 C9），红在水位文件三条用例上，与「必须红的测试名」一栏写的 `pressing_the_watermark_back_…`（第 376 行断言所在的那一条）相符且更多；另外 23 行的「prove-red 全红」是实现员自报，我没复跑，当线索。
- 推翻条件：任一行的原文在冻结副本里命中 0 或 ≥ 2 次；或替换后必须红的那条用例仍绿（主 agent 提交时的变异整表复跑归他）。

## 二、探针：条款字面里没有用例钉的句子（在冻结副本的拷贝上改坏一处，跑四个相关的测试二进制）

做法：把冻结副本拷到草稿目录，一次只改 `crates/singlefs-checker/src/walk.rs` 一处，编译并跑 `checker_known_bad_images`、`pool_checker_judges_the_deletion_state_of_every_walked_version`、`container_trees_grow_a_level_when_the_root_is_full`、`inode_number_watermark_pressed_back_reissues_a_number`（就是变异表第 1–21 行点名的四个二进制）；只跑 harness 档，不跑 GPU、不跑 checker 档。绿 = 这四个二进制里没有用例钉那一句；别的二进制我没跑，所以写「这四个里」。探针清单与改法在 `research/prompts/m3-batch22-code-r1-sonnet-model/mutations.py`，一行一探针（名字、被改条款、原文、替换文）。

对照（证明这个路子会红）：C9 = 变异表第 20 行的原文与替换文（`births.len() == 1` 改 `!births.is_empty()`），同一条流水跑出红。

跑法与原样输出（复跑 `rerun.sh`，在空目录从头跑；`SINGLEFS_REPO=/home/fy5090/code/singlefs`，编译 6G、跑 4G 都显式写在 `run-one.sh` 里）：

```
$ nice -n 19 bash research/prompts/m3-batch22-code-r1-sonnet-model/rerun.sh /tmp/claude-1000/m3-batch22-code-r1-sonnet/rerun-dir 6 7 8 summary
… step 8 exit=0 / PROBE=8 RUN_EXIT=101 / SUMMARY_MATCHES lines=74
$ grep -n 'PROBE=\|test result: FAILED\|FAILED$' research/prompts/m3-batch22-code-r1-sonnet-model/expected-summary.txt
3:PROBE=baseline RUN_EXIT=0
10:PROBE=0 RUN_EXIT=0
17:PROBE=1 RUN_EXIT=0
24:PROBE=2 RUN_EXIT=0
31:PROBE=3 RUN_EXIT=0
38:PROBE=4 RUN_EXIT=0
45:PROBE=5 RUN_EXIT=0
52:PROBE=6 RUN_EXIT=0
59:PROBE=7 RUN_EXIT=0
66:PROBE=8 RUN_EXIT=101
69:test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out
72:test creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_key_reddens_the_watermark_invariant ... FAILED
73:test creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_tombstone_reddens_the_tombstone_half_of_the_watermark_invariant ... FAILED
74:test pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_only_the_one_birth_per_number_invariant ... FAILED
```

每个二进制的汇总行（基线，`expected-summary.txt` 第 4–7 行）：101 passed、6 passed、3 passed、39 passed，全 ok。`expected-summary.txt` 是我先在一个目录里逐项跑出来写的，`rerun-dir` 是空目录从头再跑一遍写出的 `summary.txt`，两边逐字节相同（`SUMMARY_MATCHES lines=74`，包装退出码都是 0 或 cargo 的 101，没有 250–254）。`expected-summary.txt` 的 sha256 是 854a7d40cc511bdf9f4e5e7ffeddc5e2392967aae14b3b5876288432e177fdfd。

| 探针 | 改坏的一处 | 对应条款字面 | 四个二进制里 |
|---|---|---|---|
| M1 | 类型段 0 的条目身份判定恒真（`walk.rs:2133` 那一句后加 `\|\| true`） | I-9.17 / I-9.23 「类型段 0 ⇒ 子单元是码 2」 | 全绿：没钉 |
| M2 | I-9.29 不收谱系上墓碑的出生代（`walk.rs:9526` 起那一块 `continue`） | I-9.29 「与那一版谱系上墓碑的出生代都算」 | 全绿：没钉 |
| M3 | I-9.20 去掉「死亡 txg ≤ 这一版的 txg」（`crates/singlefs-checker/src/walk.rs` 第 9759 行） | I-9.20 「对象出生代 ≤ 死亡 txg ≤ 这一版的 txg」 | 全绿：没钉上界 |
| M4 | I-9.4 第四句改成 `inode != previous_inode`（`walk.rs:2503`） | 「记录号沿记录序严格递增」 | 全绿：倒序没钉 |
| M5 | I-9.21 / I-9.22 容器号沿叶序严格递增那一句恒真（`walk.rs:9818`） | 「容器号沿叶序严格递增」 | 全绿：没钉 |
| M6 | I-9.18 配墓碑时不比出生代（`walk.rs:9662`） | 「(N, N 的 inode 记录偏移 8 的对象出生代, [0, 2⁶⁴ − 1])」里的出生代 | 全绿：没钉 |
| M7 | I-9.18 配墓碑时不比整对象区间（`walk.rs:9663`） | 同上里的区间 | 全绿：没钉 |
| M8 | 墓碑与谱系那一遍的去重键去掉 txg（`walk.rs:9283`） | 「每一版从这一版自己的 txg 起」 | 全绿：没钉（树表相同、txg 不同的版只扫第一个，合法历史上两版同判，说明不了会不会漏判坏镜像） |
| C9 | 对照：I-9.29 `births.len() == 1` 放成 `!births.is_empty()` | I-9.29 | **红 3 条**（水位文件三条用例） |

这八个全绿按判据只算「钉不到」：要成为「漏红」需要这一句的违反在条款里真是违反——M1、M2、M3、M4、M5、M6、M7 都是字面里写了的句子，改坏之后实现违反字面而四个二进制照绿，符合跑前写死的「漏红」定义；M8 是设计选择（去重键）而非条款字面，只记「没钉」，不记漏红。每一条都只抽了一次样（这四个二进制，一次改法），照 `.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算` 只记挂起，不当终局。

推翻这一节：另一个测试二进制（四个之外）里有用例钉住上面某一探针（我没跑别的二进制）；或探针改法本身漏改了（C9 说明路子能红，但每个探针的改法我只读过一遍）。

## 三、命令核

```
$ python3 -B research/prompts/m3-batch22-code-r1-sonnet-model/anchors-check.py research/prompts/_m3-batch22-code-r1-diff.md /tmp/claude-1000/m3-batch22-code-r1/tree | tail -1
ROWS=24 NOT_EXACTLY_ONE=0
$ cd /tmp/claude-1000/m3-batch22-code-r1/tree && sha256sum -c /home/fy5090/code/singlefs/research/prompts/m3-batch22-code-r1-snapshot/crates-sha256.txt | awk '{print $2}' | sort | uniq -c
     12 OK
$ grep -c '^第22批 C' /tmp/claude-1000/m3-batch22-code-r1/tree/crates/mutations.tsv ; grep -c '^删除批第 22 批' /tmp/claude-1000/m3-batch22-code-r1/tree/crates/mutations.tsv
20
3
```

另：冻结副本 `crates/mutations.tsv` 里 23 条新行的锚点各命中 1 次（同一个脚本口径，直接读 `mutations.tsv`，`anchor-bad 0`）。

## 四、K3：十二份各一格

| 文件 | 判定 | 依据（上文节号） |
|---|---|---|
| `crates/singlefs-checker/src/image.rs` | 一致 | N1：数组 83、下标 74–82 次序与 T13 同；kb 头与状态列没跟是 kb 侧 |
| `crates/singlefs-checker/src/walk.rs` | 一致，有缺口 | N3–N9：每条判定与字面对上；八处探针没用例钉；N7 疑点（`BTreeSet` 折同号两条） |
| `crates/singlefs-checker-tier/src/crash_judge_gpu.rs` | 一致 | N1：覆盖表登记九位、内核不置位，题面写明第 23 批补 |
| `crates/singlefs-harness/tests/checker_known_bad_images.rs` | 一致，有缺口 | N2、N4、N8：断言恰好；类型段 0 一支、谱系侧类型段不认得、倒序没有坏镜像 |
| `crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs` | 一致 | N5、N7、N9 的搭建与改法辅助；I-9.5 的 nlink 1 那一份按条款同红 I-9.19，登记在 I-9.5 名下，由 955 行的用例断言两条整集合 |
| `crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs` | 一致 | N6：旋钮两入口各一份坏镜像、恰好；容器号递增句没钉（M5） |
| `crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs` | 一致，有缺口 | N9：三条用例按条款改断言；墓碑那一侧没钉（M2） |
| `crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs` | 一致，有缺口 | N3、N5、N7：坏镜像恰好、两段合法历史；I-9.20 上界、I-9.18 出生代与区间两句没钉 |
| `crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs` | 一致 | N11 第 2 条达到 |
| `crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs` | 一致 | N11 第 1 条与盘数断言达到 |
| `crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs` | 一致 | N10：45 / 49 / 16 三张表与 83 − 45 = 38 |
| `crates/mutations.tsv` | 一致 | N12：23 追加 + 1 替换，锚点各命中 1 次；两对共用一处文本 |

## 五、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 N1 | 一致（代码侧）；kb 头与状态列没跟 | 83 条、下标 74–82 次序同 T13、GPU 表登记九位内核不置位；`invariants.md` 第 12 行仍写 74 条、九行状态列仍写未实现 |
| K1 N2 | 一致 | 两种记录宽各一份坏镜像，断言 `["I-1.7"]` 恰好 |
| K1 N3 | 一致 | I-9.10 墓碑等号按号认、每版比自己的；读不全静默不判是已有欠账口径 |
| K1 N4 | 一致，有覆盖缺口 | 身份单独判、先登记引用再判红、合取留着；类型段 0 一支与谱系侧类型段不认得没有坏镜像（M1 绿） |
| K1 N5 | 一致，有覆盖缺口 | 谱系链逐段推对得上；两段合法历史零违例；I-9.20 上界、I-9.18 出生代与整对象两句、去重键 txg 四处没钉（M3、M6、M7、M8 绿） |
| K1 N6 | 一致，有覆盖缺口 | 两入口、两支阈值与条款对上；容器号递增句没钉（M5 绿） |
| K1 N7 | 一致；一处疑点 | 每条条目都判、读队列在配对前；队列读成 `BTreeSet` 折同号两条（读码，未跑） |
| K1 N8 | 一致，有覆盖缺口 | 第四句 `>`；用例只有 S4a，倒序没钉（M4 绿） |
| K1 N9 | 一致，有覆盖缺口 | 各版合起来、两样出生代都算；C9 对照红 3 条；墓碑那一侧没钉（M2 绿） |
| K1 N10 | 一致 | 注释改准；45 / 49 / 16 三张表与 83 − 45 = 38 |
| K1 N11 | 部分达到 | 判决第三节第 1、2 条达到；第 3 条三小项里 `history.rs` 两处过期行号未改（`history.rs` 不在被判十二份里） |
| K1 N12 | 一致 | 24 条锚点各命中 1 次；两对变异共用一处文本；只复跑对照一行 |
| K3 | 十二份各一格，见第四节 | 十二份判到；其中八份「一致」、四份「一致，有缺口」（容器树那份的缺口 M5 记在 walk.rs 一行） |

## 六、没做什么

- 不判 K2：没有造反例、没有攻 N5/N9/N4/N7 的合法历史（回退、崩溃恢复重发、屏障段界）；第二节探针是「钉不钉得住字面句子」，不是「合法状态会不会误红」。
- GPU 一律没跑；`checker-tier` 不碰；`crash_enumeration_new_pool_file_creation_stream.rs` 的分组计数用例（题面写明第 23 批）没碰。
- 没复跑 23 行新变异（实现员 prove-red 自报当线索，只复跑了对照一行 C9）；变异整表复跑归主 agent 提交时。
- 没跑 `writable_mount_of_a_formatted_pool.rs`、`the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs`、`unlink_writes_per_transaction_do_not_grow_with_the_file.rs` 三个二进制（主 agent 说合入验证里全绿；N10、N11 按读码判，没有重跑）。
- 不替主 agent 采纳或出判决；「N7 的 `BTreeSet` 折同号」「`history.rs` 小项」「kb 头与状态列」三件是否进下一批由主 agent 定。
- 这一轮的条款文本以 `invariants.md` 今天的写法为准；I-9.29「最新根实例表读不出时记判不了」那一句代码没有，按题面不当打中。
- 草稿与编译目录：探针用的拷贝 `/tmp/claude-1000/m3-batch22-code-r1-sonnet/work/`、编译目录 `/tmp/claude-1000/m3-batch22-code-r1-sonnet/target/`、空目录重跑的 `/tmp/claude-1000/m3-batch22-code-r1-sonnet/rerun-dir/` 在交回前删掉（`rerun.sh` 可在空目录从头重建，14 秒编完、每项约 2 分钟）；保留的只有 `/tmp/claude-1000/m3-batch22-code-r1-sonnet/out/` 里的日志与 `/tmp/claude-1000/m3-batch22-code-r1-sonnet/build0.log`、`base-run.log`。
- 模型目录：`research/prompts/m3-batch22-code-r1-sonnet-model/`（`mutations.py` 探针清单、`apply.py`、`run-one.sh`、`rerun.sh`、`anchors-check.py`、`expected-summary.txt`）。`rerun.sh` 在空目录里从头跑通：`SUMMARY_MATCHES lines=74`。
