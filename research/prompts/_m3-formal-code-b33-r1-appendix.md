**出处 `.claude/kb/invariants.md:1-8`（整段抄，未转述）**

```markdown
# 不变量清单

**checker 是 invariants.md 的可执行形式。** invariants.md 每加一条，checker 就要加一个检查。
两者不同步的 commit 一律不收（见 [format-evolution.md](../rules/format-evolution.md)「硬约束」）。

每条不变量必须写成**可判定**的形式——能对着一个镜像回答"成立/不成立"。池级 checker 对一条不变量的判定有四种：成立、不成立、不适用（必须带理由，不许静默算成立）、部分没判（`InvariantVerdict::PartiallyJudged`，`crates/singlefs-checker/src/image.rs`）。部分没判是判到的那一部分都对得上、射程里另有一部分这张镜像上判不了，说明逐条列出没判到的是什么；它不是红，也不是成立：层 0 与崩溃放量把它算成不红，层 0 计数并进「不适用」那一格，GPU 判器只记违例位、它在那一侧与成立、不适用同是 0。今天只有 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同）报它。
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
| I-2.4 | 头校验和覆盖范围 | 任一单元的头校验和覆盖从偏移 0 到该类明文头的最后一个字节，端点写死：码 1 到偏移 104、码 2 到偏移 85 + 2 × key 宽、码 3 到偏移 106（D18（块里携带什么信息） 已定项 7 / 已定项 11 / 已定项 18 的字段表）；头校验和字段自身按 0 参与。**29 字节 nonce / MAC / 算法类型预留位虽然算进头（已定项 16）、也是明文，但不在头校验和覆盖内**——MAC 在头校验和之后才算得出来，罩不住它；单元头的这 29 字节与指针头部的 MAC 16 + nonce 12（D19（块指针的结构与宽度预算） 已定项 3）由同一条规则守：加密关着时恒 0、读者遇到非 0 一律判该结构损坏（已定项 17；指针头部那一半 2026-09-27 主 agent 定，池级判定在 A3-checker-2 里做），加密开启后由 MAC 自己罩（2026-09-14 用户定案把预留位从 28 加到 29，多的那 1 字节是算法类型）。指针头部的 extent 偏移 2 字节第一版恒 0（第一版指针指整个单元），读者遇到非 0 判该指针所在的结构损坏，与 MAC / nonce 同一处置、与加密开不开无关（D19（块指针的结构与宽度预算） 已定项 3；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 11 行）。根记录里的指针偏移非 0 按这条根自证不过处置：core 退回上一条根（靠 journal 重放可以回到同一版，挂载照样成功），checker 红这一条而 core 挂载成功，两边各判各的、不矛盾（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 6 行）。加密开启（系统配置级、全有全无，I-6.4（无未加密块））后覆盖范围 = I-6.2（明文头字段白名单） 该类白名单里的全部明文头字段，密文段由 MAC 罩。改动覆盖区内任一字节必须失配、改动净荷字节不许失配 | 已实现（`crates/singlefs-checker` 的 `check_unit`：三类的端点各自写死，里程碑步 2 / 步 3 验收里改坏头任一字节判红，2026-09-14；池级走读在 `walk::judge_unit_header`（`crates/singlefs-checker/src/walk.rs:674` 起）另判单元头偏移 4 的格式版本 = 1（walk.rs:722，不认得就判该结构损坏、不按今天的字段表往下解）与 29 字节预留位全 0（walk.rs:758）；走读跟随的每条指针在 `judge_pointer_mac_and_nonce_are_zero`（`crates/singlefs-checker/src/image.rs:699`）判头部 MAC 16 + nonce 12 全 0，四个入口分别是码 2 节点指针（walk.rs:815）、inode 内部条目子指针（walk.rs:1485）、实例表链指针（walk.rs:1811）、extent 数据指针（walk.rs:1961），只判不断、指针指向的单元照样走下去，2026-09-27 实审 A3-checker-2；坏镜像 `crates/singlefs-harness/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs`）。**extent 偏移那一半已实现**（checker 的指针判定 `judge_pointer_mac_and_nonce_are_zero` 加这 2 字节，走读跟随的每条指针都判；core 每个读盘上指针头部的读者都判 extent 偏移非 0、判该结构损坏：树表条目的树根指针、inode 内部条目的子指针、extent 叶记录的数据指针、extent 上段叶条目的下段根指针与内联数据指针、extent 树内部条目的子指针、分配记录树内部条目的子指针、实例表链指针各一处，加原有的 journal 新根段、码 2 内部条目、根记录三处（`crates/singlefs-core/src/pointer.rs` 的 `read_judging_the_extent_offset_from`；`grep -rn 'Pointer::read_from' crates/singlefs-core/src/` 只剩 `pointer.rs` 自己的单测一处）；影子账认单元的两个读者（`allocation_record_tree::node_pointers_as_far_as_readable`、`recovery::instance_table_page_pointers_as_far_as_readable`）今天也判，已定改回不判（影子账要多隔离、不漏隔离），改动在下一批；坏镜像遍历偏移 32、33，core 每类读者各一条用例在 `crates/singlefs-harness/tests/pointer_extent_offset_that_is_not_zero_is_refused_by_every_tree_reader.rs`；对应的账是 C581（extent 偏移非 0 没有判据）；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 11 行与 `research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 5 行） |
| I-2.5 | 位置条目按设备身份升序 | 任一块指针、中央映射条目的 value、journal 点名项里的位置条目数组按设备身份严格升序（D19（块指针的结构与宽度预算） 已定项 11、D23（journal 的角色与格式） 已定项 17，2026-09-13 用户定案）；两条位置条目设备身份相同或逆序，一律判损坏。⚠️ **射程（2026-09-14 补）：整条 86 字节全零的指针表示「注册了但还没有根」，整条豁免升序判定**——它内部那两条位置条目都是 (设备 0, 槽 0, 校验和 0)，按字面正是「设备身份相同」那一档。今天写出全零指针的有四处：树表里 livelist 树、稀疏旁表树与 deadlist 树那三条条目（D6（快照实现模型） 已定项 2、D5（快照 / 空间记账机制） 已定项 6 与 D5（快照 / 空间记账机制） 已定项 11 各自定案的「day-1 注册进树表，根指针为零」），以及 mkfs 与暖机的根记录里那个还没发布过的中央映射树根指针。非全零的指针不享这条豁免。 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |

```

**出处 `.claude/kb/invariants.md:171-189`（整段抄，未转述）**

```markdown
## I-4 事务与崩溃一致性 —— 类
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-4.1 | 重放后树可遍历 | 任一崩溃点重放后，所选根（重放之后那一版）指向的树全部可完整遍历。起点含介质损坏的镜像（前缀停在一条点名单元两份没都验过的记录上）报不适用。另三处报不适用：所选根那次发布读得出的记录里带末条标志的多于一条（锚点认哪一条条款没写，实现在任何写之前拒挂，重放不出一版）；所选根的 checkpoint_txg 是 u64 的最大值（接在它后面的发布取不到 txg）；所选根之上一次发布都不施加（重放之后那一版就是所选根，可不可遍历归 I-7.2（最新根可完整遍历））——没有记录压在最新根之上的镜像上这一条因此恒报不适用、不报成立。重放用的读环按 (实例代号, 计数器) 建键，两块盘同一计数器坐着不同实例的记录时两条都留，与实现恢复同键（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 2、8 行）。 | 已实现（2026-10-02，池级 checker `walk::judge_trees_of_the_replayed_version_are_traversable`：R-独，checker 自己照 D23（journal 的角色与格式） 已定项 14 的前缀规则复刻只读重放（`replay_the_next_mount_would_do`），施加出来的那一版（所选根换上末条记录新根段的七项：树表指针、中央映射树根指针、树 ID 水位、F、实例表指针、分配记录树根、卸载记号，只有 fsid 照所选根；D23（journal 的角色与格式） 已定项 15）另起一份走读，走读断了判红；在飞上限按环长 ÷ 4096 ÷ `JOURNAL_SAFETY_FACTOR` 现算。读环键 2026-10-03 改成 (实例代号, 计数器)（`records_the_replay_reads`），反向链那一关改按 (实例代号, 计数器 − 1) 找前一条（`record_back_chain_lets_it_into_the_replay_prefix`），消掉两块盘同一计数器坐着不同实例的记录时与实现分开的形态（推为第一版两盘池上不可达）；`records_as_the_recovery_reads_them` 与 `record_can_enter_a_replay_prefix` 两处同改按 (实例代号, 计数器) 建键，已做（主 agent 2026-10-03 定）。坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（重放六条规则各一份、两块盘同计数器不同实例两份）；层 0 里与 core 的 `effective_root` 逐状态差分今天没有能跑的入口，要先开口子，进下一批清单（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 8、9 行）；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 2 行与第四节用户定案） |
| I-4.2 | 无被引用未提交块 | 任一崩溃点重放后，不存在被引用但未提交的块 | 已实现（2026-09-22，池级 checker `walk::check_pool_image` 的 `judge_birth_identity_of_a_referenced_unit`：走读到的每个被引用单元，头里的诞生代号与写序（D18（块里携带什么信息） 已定项 7 单元头字段表里的字段）按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的已发布谓词判成已发布——**「未提交」取的就是「按那个谓词判不成已发布」**，谓词与 I-1.8（归并后版本全序） 扫描方向共用同一份 `PublishedPredicate`。打包记录类型 4 的实例表单元按 I-1.2（块头写序已发布） 那一行的例外不走谓词，不进这一条。坏镜像是把数据单元头里的诞生代号与引用它的 extent 记录里那条数据指针的诞生代号一起抬到最新根的 txg 之上——指针仍与头一致 ⇒ I-1.2（块头写序已发布） 不跟着红，分得开两条的就是这一份（`crates/singlefs-harness/tests/checker_known_bad_images.rs`）；层 0 每个崩溃状态都跑它。⚠️ **「任一崩溃点重放后」那半句没实现**：池级 checker 判的是手上这一份镜像、不跑恢复，判的是「被引用 ⇒ 已发布」这个态上的性质；重放之后再判一遍归层 0） |
| I-4.3 | 提交原子 | 提交是原子的：不存在"部分生效"的事务 | 未实现（它要判的是 D23（journal 的角色与格式） 已定项 7 那两个事务边界字段——事务号与提交标记——施加之后的效果。**对着一份镜像判不出来，没硬写**：「部分生效」要拿「这次事务该有的全部效果」当 oracle 才分得出来，而一份镜像上任何状态都自洽——同一份字节既可能是「事务整体没生效的旧态」，也可能是「生效了一半」，池级 checker 手上没有第二个输入。判它的是层 0 崩溃点重放加 oracle：`.claude/kb/milestone/02-second-txn.md` 第 486 / 487 行那两条（崩在第二条记录之前 ⇒ 整个事务不施加、文件是旧长度；把提交标记提前到第一条记录 ⇒ 恢复施加了半个事务、I-4.3（提交原子） 红）说的就是这条路。形态同 C116（叶容器的分裂 / 合并纪律没有会失败的检查）：操作断言，不进池级 checker，恒真的检查比没有检查更糟） |
| I-4.4 | 意图只描述目标态 | 任一未完成的意图 key，其字段只描述目标状态，不含任何增量字段（"再删 N 个"一类） | 已实现（2026-10-05，在清单里的第一版恒报不适用：池级 checker 在 `walk::check_pool_image` 里对它调 `not_applicable`，理由逐字是 `crates/singlefs-checker/src/image.rs` 的 `INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION`；不要求坏镜像，`crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `INVARIANTS_WITHOUT_A_BAD_IMAGE` 点名；GPU 判器下标 73，内核不置位）。第一版不适用：意图载体（nlink = 0 的记录、`deleted_inodes` 的 key、墓碑）没有可装增量的字段，checker 报不适用并写理由；logged_ops 有字段表时实现（用户 2026-10-05 定，岔路单 `research/prompts/m3-delete-checker-r1-forks.md`） |
| I-4.5 | 已停用·需两镜像 | **此编号不再使用**——幂等要比对两个镜像，单镜像判不了；判据落在 [experiments.md](experiments.md) E3（意图日志的断电语义） 判据③ | 不适用 |
| I-4.8 | 近 K 代根校验和自洽 | 任一崩溃点重放后，从回退候选集（D23（journal 的角色与格式） 已定项 14：按实例表判仍然有效 ∧ txg ≥ F_生效）里**任一**根出发遍历，所有块的校验和均与其父指针记录的一致（候选集的深度由 D16（发布语义） 已定项 1 的回退下界 F 定：平时整个根环，盘紧时缩到最近 4 个可退到的状态，每块盘上最新的持久有效根都在内）。**判别力**：抓「本事务内释放的块被重新分配并写入」这一类——那会让旧根指向的块被写入新内容，**使旧根成为假的回退候选**，而只验最新那一代根是发现不了的 | 已实现（2026-09-17，池级 checker `walk::check_pool_image`，最新根与候选集里每条更早的根各判一格：从它出发的遍历有单元对不上或读不出就红；坏镜像是 mkfs 树表单元抹成零；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-4.6 | 中间态其余不变量成立 | 存在未完成意图的镜像上，其余全部不变量仍然成立（分批之间的中间态是合法状态）。「其余全部不变量仍然成立」由各条在每个被判的版本上各自判，中间态的版本也在内；I-4.6（中间态其余不变量成立） 自己判的是中间态特有的一格：每条 nlink = 0 的记录，数据单元数与 size 对得上——size 为 0 时 0 或 1 个单元，size > 0 时 ⌈size ÷ 数据单元净荷容量⌉（实现 `crates/singlefs-core/src/write_request_split.rs` 第 61 行 `div_ceil(...).max(1)`）。判哪几版、配对与读不全的报法照 I-9.5（删除队列里的号仍在树里） 的共用判法 | 已实现（2026-10-05，池级 checker `walk::check_pool_image`，函数 `judge_data_unit_counts_of_the_inodes_being_deleted_against_their_size`；净荷容量 checker 自己算一份（`data_unit_payload_capacity_in_bytes`）；每条 nlink = 0 的记录判一次，一条都没有时报不适用并带理由；坏镜像：第一个文件的 size 加一个净荷（`crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs` 的 `raise_the_size_of_the_first_file_by_one_data_unit_payload`）；GPU 判器下标 72，扫描内核判；用户 2026-10-05 弹窗定「只判单元数对 size」，出处 `research/prompts/m3-delete-checker-r1-main-verification.md` 第四节与岔路单 `research/prompts/m3-delete-checker-r1-forks.md`；射程只对 nlink = 0 的记录，每条记录都判的那一形不挂在这一行名下，另记欠账 C711（单元数对 size 只判了 nlink = 0 的记录）） |
| I-4.7 | 删除中 inode 有账 | 任一处于删除中的 inode，要么在删除中队列里有条目，要么其 extent 已全部释放——两者皆非的中间态不存在。「处于删除中的 inode」指两样：inode 树里 nlink = 0 的记录；extent 树里有数据单元、而 inode 树里没有 nlink ≥ 1 的记录的号。判哪几版、配对与读不全的报法照 I-9.5（删除队列里的号仍在树里） 的共用判法 | 已实现（2026-10-05，池级 checker `walk::check_pool_image`，函数 `judge_inodes_being_deleted_against_the_queue_and_their_data_units`；每个处于删除中的号判一次，一个都没有时报不适用并带理由；坏镜像：extent 树里有数据单元、inode 树里没有它的记录、号也不在队列里（`crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs` 的 `drop_the_record_of_the_first_file_and_keep_its_data_units`）；GPU 判器下标 71，扫描内核判；对象取这两样，出处 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节逐格判决表里 I-4.7（删除中 inode 有账） 对象那一格） |
| I-4.9 | 非最新候选根引用的单元不晚于它 | 回退候选集里不是最新的每一条根 R，它引用的单元（按落点认，读落点上单元头里的诞生代号，头校验和过）的诞生代号 ≤ R 的 `checkpoint_txg`。最新那条根归 I-4.2（无被引用未提交块），不在这一条里。依据 D16（发布语义） 已定项 7（单元先于根落盘）。 | 已实现（2026-10-02，池级 checker `walk::judge_unit_birth_against_an_older_candidate_root` 判走读里每个被引用的单元，共享子树里别的版本先走过的单元由 `merge_placements_below_a_unit_walked_by_an_earlier_version` 补判；候选集只剩一条根时报不适用，同 I-3.9（释放代落在停止引用它的那一格区间里）；坏镜像沿 I-4.2（无被引用未提交块） 那一份的造法，映射 key 也要跟着改，在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 1 行） |

> ⚠️ **编号只往上加，不回收、不重编。** 回收一个编号会让别处的旧引用悄悄指向另一条陈述，
> 而检索只端出其中一条，不会告诉你它换了意思（`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 4 条「矛盾比空白更糟」）。
> 退役的编号留一行说明去向，比留一个无解释的空档安全。

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
| I-5.4 | 分配记录罩住的槽互不相交 | 任何一条有效根的分配记录树里，同一块盘上任意两条分配记录（不论已分配、已释放、释放代是否已低于回退下界）罩住的槽区间 [槽号, 槽号 + 跨度) 互不相交。复用一个已回收的落点时，新记录罩住的槽上别的已回收记录要随这次分配一起删掉；留着它，下一次可写挂载重建分配器时两条记录罩住同一个槽 | 已实现（2026-09-18，`crates/singlefs-checker` 的 `walk::judge_allocation_records_disjoint`，按回退候选集里每条根判；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `an_allocation_record_whose_span_covers_the_next_record_reddens_only_the_disjointness_invariant`；层 0 每个崩溃状态都判，同日在新代码上全量：第二条流 2,104,413 个状态里评估 1,842,252 个、违例 0，其余 262,161 个报不适用，第一条流 262,165 个状态里评估 4 个、违例 0（`m2-wave2-crash-verifier-report.md`「主 agent 要的两个数」）。同日立，依据 `research/prompts/m2-wave2-code-r1-main-verification.md` 第二节第 1 行：复用时跨度变了留下重叠记录，下一次可写挂载在重建分配器时 panic，而 I-5.1（物理范围不重叠） 只管树里的引用、判不出记录之间的重叠） |
| I-5.5 | 从 mkfs 水位建树那一版的 t1 落点 | 走读到的任一版 V（候选集里每条根，含由记录施加出来的那几版）若是从 mkfs 水位第一次建树的那一版——① V 的树表非空，且种类 1（extent 树）条目的树 ID 等于 65536；② V 的树表里每条条目的诞生 txg（条目偏移 108）都等于 V 的 `checkpoint_txg`——则 V 引用的数据单元里，每块盘上落点槽号最小的那一个，槽号等于这一版分配记录里 D3（空间分配） 已定项 8 第 1 条那条政策算出的那一对的起点槽号（起点最小、32768 对齐、两槽都空、不在开放段里）。新池新建文件那一次是 U + 4，U 是所选系统配置偏移 417 的单元区起始槽号（默认 768 MiB 环 U = 50176，所以是 50180）；回收之后（例如 mkfs 的实例表 U、U + 1 那两槽被回收）不再是 U + 4。V 没有数据单元时不判；不满足 ①② 的版本不判。这一类含新池新建文件，也含 mkfs 之后重挂、实例 1、2、3 从 mkfs 水位写的第一个文件（用户 2026-10-04 定案，原话在决策变更史 D3（空间分配） 节）。只判 t1，t2–t12 不判（D3（空间分配） 已定项 10 射程没有「偏离即不合法」）。已知限度：这一版盘面上复算不出的几样只能近似——已释放的记录一律当占着（回收门槛复算不出：写死从 U + 4 起扫的坏实现与回收门槛放宽的坏实现两个方向都判不出，C689（建树落点判据抓不到写死从 U + 4 起扫的坏实现））；这一版自己分配的记录当空；开放段拿这一版树表单元所在的段当它（这次挂载更早开过、已经用尽的段认不出；提交内生块回落时会误排，今天的准入下够不着，C687（从 mkfs 水位建树那一版落点判不了的几格））；隔离位、抬 F 扣住位与保护期内的 R 盘上都没有，没有分配记录罩着的槽一律当空。依据 D3（空间分配） 已定项 8 射程与已定项 10 ③ ④，以及判决 `research/prompts/m3-formal-code-b17-r1-main-verification.md` 第二节 K2 与用户 2026-10-04 对判据改法的定案（原话在决策变更史 D3（空间分配） 节）。 | 已实现（判据是 f2（按这一版分配记录现算），代码轮判决 `research/prompts/m3-formal-code-b19-r1-main-verification.md`；池级 checker `crates/singlefs-checker/src/walk.rs` 的 `judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark`：候选集里每条根与由记录施加出来的那几版各自重读树表与 extent 树判，U 取所选系统配置偏移 417；GPU 判器扫描内核 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 的 `judge_first_data_unit_of_a_version`，`IMPLEMENTED_INVARIANTS` 下标 67；这一版的树表读不出或条目宽不到 200 字节、种类 1 条目不止一条、extent 树读不全、有数据单元而池里某块盘上一个都没有、种类 3（分配记录树）条目不是恰好一条、分配记录树读不全，这几格报射程里判不了（没有违例时整条报不适用），该判红还是不适用欠在 C687（从 mkfs 水位建树那一版落点判不了的几格）；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（数据单元从 50180 搬到 50182），合法镜像在 `crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs`） |

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

**出处 `.claude/kb/decisions/16-发布语义.md:27-83`（整段抄，未转述）**

```markdown
#### 已定项 1：defer 窗口取按盘回退下界的收严形态，按状态数读

**定案**：回退候选集保留最近 4 个可退到的不同状态——只数改过用户可见状态的根，推空与抬 F 产生的空发布根不算；同一个状态只数一次（去重的判法在「「非空」从盘上怎么认」那一段）。**根环容量是这句的边界**：两个状态来回交替超过根环一圈时，环里的根全是这两个状态，保留的就只有环里有的那几个，任何机制都给不出环里没有的状态。**窗口的单位是发布，不是秒**：defer 窗口保护的是「环里的根还能不能用」，普查过的消费者没有一个按秒计价。形态取 E139（按盘回退下界的收严形态） 的臂 G2S；回收门槛退回环里最旧有效根之后，装置的候选下界还没带系统配置里的 F，这一句仍不与装置里跑的一字不差，欠里记着 E139（按盘回退下界的收严形态） 要重跑：

| 项 | 定案 |
|---|---|
| 可再分配 | 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根) |
| 环里最旧有效根 | 盘上全部根槽里自证合法、按实例表判仍然有效的根的最小 checkpoint_txg；写失败的槽按旧内容算，在飞、没持久的发布不算。**同取这个值的六处**（用户 2026-10-04 定，被攻过一轮）：分配器的回收门槛（含抬 F 那一遍）、checker 的候选过滤、checker 的 I-3.1（已分配统计对得上） 丁-defer 门槛（max(F_生效, 环里最旧有效根)）、影子账的候选过滤、回退候选集、抬 F 的上限（含判「非空」那一遍）；六处都不再按一圈窗口滤根，窗口之外的有效根（崩在记录之后、根槽之前那一形留在槽里的旧根）也在候选集里、也压着门槛。**挂载时读不出的槽**：N = 环里读得出的最新有效根的 txg（挂载重建分配器时取它、不取 first_txg − 1；进程内每记一条根时取这条根）；读不出的槽在 txg > N − 3S（一圈，3S 是环长）里只对应一两个 txg：门槛按最小的算，根环表记最大的（零轮，只在攻方模型上量过），抬 F 那一遍里根环表挂载时读不出的那几项同样按这一句并进最旧有效根；一圈窗口这个量现在只剩这一处在用，给读不出的槽划出它能住的 txg 的范围。进程内记根时，根环表里这一项成了最小的那个，先重读那一槽：读得出、自证过、按实例表仍然有效的，照读出来的根算；读得出而自证不过的，与仍读不出同一处置，按记的最大 txg 算；读得出、自证过而按实例表被抛弃的，当没有根（用户 2026-10-03 定先重读，2026-10-04 定自证不过那一格取按记的最大 txg 算，零轮）；重读读出有效根的那一槽，算进「这个进程知道住着根的槽」（「根槽这一次读坏」那一行）。 |
| 回退下界 F | 根记录带 8 字节（D22（单元原子性怎么合成） 已定项 7，242 → 250），系统配置另记一份 8 字节（D22（单元原子性怎么合成） 已定项 9）。平时不动，窗口就是整个根环；准入不够时抬，正常卸载时抬到现行那一版的 txg |
| 抬 F 的上限 | **准入抬 F**：min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)；非空有效根不足 4 个时取最旧的有效根；「有效根」「非空」在环里全部的根上取；挂载那一刻读不出或自证不过、之后这个进程没写过的槽，当作可能是一条非空有效根，F 不越过它（用户 2026-10-04 定，零轮：先定只并读不出的槽，代码轮第三轮之后重定为读不出或自证不过两类都并；与「环里最旧有效根」那一行把读不出的槽并进回收门槛同一个思路）。一次处置的目标 = min(这次释放的释放代, 第 4 新的非空根)。**卸载抬 F**：抬到现行那一版的 txg，不另判上限（等价于上限取最新的持久有效根，不按盘取小） |
| 根槽这一次读坏 | 算准入抬 F 的上限、挂着时管理员回退判候选（根环里有没有目标）与算 F_生效 时读根环：挂载那一刻就读不出或自证不过、这个进程之后也没写过的槽，算准入抬 F 的上限时当作可能是一条非空有效根、F 不越过它（「抬 F 的上限」那一行，用户 2026-10-04 先定只并读不出的槽、代码轮第三轮之后重定为读不出或自证不过都并，零轮），判回退候选与算 F_生效 时当没有根；挂载那一刻读得出、这个进程写过且 FUA 返回过、或进程内记根时重读读出了有效根的槽（这三类是「这个进程知道住着根的槽」），这一次读不出或自证不过就重读一次，仍坏就拒这次抬 F 或这次回退、报它自己的错误成员，不按有根或没根猜（抬 F 那一半用户 2026-09-26 定，推的，被攻过零轮；回退那一半主 agent 2026-09-28 定，被攻过一轮：代码三方第三轮判决 `research/prompts/m2-closeout-code-r3-main-verification.md` 第一节判替没写的条款做了选择、回退扩写拒错或放错那一格没打中（一次抽样））。可写挂载的择根按 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续处置（D23（journal 的角色与格式） 已定项 14：系统配置见证过比所选那一版新的发布就把读阶段重读一次，仍判真拒可写）；只读挂载的择根、「环里最旧有效根」怎么处置这一格没判，见 C563（读根环一槽读坏就当没有根，别的调用方没判） |
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

**射程**：定的是 defer 窗口与回退下界怎么算，不定环几何、不定 c_max 的口径（c_max 是声明的最坏空发布开销，按 D28（挂载期承诺量） 已定项 4 按当时结构现算）。已知边角八样：

- **「生效」取 SysPre**：F 只住根记录、根槽不镜像（已定项 7）时，带新 F 的根坏掉 F_生效 就回落，更早的根重新进回退候选集、而它们引用的块已被合法复用——I-7.4（近 K 代块未被复用） 在这段历史上不成立（`research/prompts/m2-step45-code-r2-main-verification.md` 第四节、`research/prompts/m2-step6-checker-r1-main-verification.md` 第四节）；病根是 F 只住根记录，与生效取哪条规则无关（第三轮 K4：带 F 的根全坏那一格，今天的规则、MAX、MAX+HOLD 候选坏 12 / 140 / 12，SysPre 0）。代价：系统配置多 8 字节（动格式）；每次抬 F（准入与卸载）每块盘多一次系统配置写、加一道屏障；checker 与层 0 要跟着读系统配置里的 F——今天的 checker 按根上带的 F 判，带 F 的根全坏时会误红 I-7.4（近 K 代块未被复用） 等五条（第三轮 Y7，推的，没实现），欠 C556（checker 与层 0 不读系统配置里的 F）。「先写后抬」是第三轮攻方提的，被攻过零轮，用户 2026-09-26 知情定。被抛弃时间线上的根带的 F 算进 F_生效（用户 2026-09-28 定「算进」，被攻过一轮，判决 `research/prompts/abandoned-floor-r1-main-verification.md` M2：被抛弃根带着比别处高的 F 时，不算的话 I-7.12（系统配置 F 不低于同盘根上的 F） 红、回退退得到那个 F 之下，而被抛弃时间线抬 F 时已按它回收过槽；实现 `crates/singlefs-core/src/recovery.rs` 的 `effective_rollback_floor_of_the_roots_read`，checker 另算一份 `crates/singlefs-checker/src/walk.rs`）；C331（择根倒挂压过已确认的写） 取甲之后，被抛弃根带着更高 F 的盘面只在构造的镜像上有（那一形要可写挂载时一槽系统配置一时读不出，甲 拒这样的可写挂载）。与 `research/prompts/alloc-basis-r3-main-verification.md` 的 F 口径岔路一起判的那一件仍开着。
- F 回落之后第 0 代根重新进有效根、它的树表单元已被合法复用、此后每一次抬 F 都被拒到根环把它盖掉为止（`research/prompts/m2-emptypool-nonempty-r1-main-verification.md` 第一节 Z1）：前提是 F 回落，取 SysPre 之后在带 F 的根全坏那一格上不再出现（推的，第三轮没造那一形）。一次挂载里连续发布超过根环容量时挂载内不回收，checker 在合法状态上判 I-3.1（已分配统计对得上） 红（E153（账本形态与环上有洞的代价） 在 `crates/` 副本上量到），仍开着，归里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 ② 行。
- **正常卸载的代价**：每次卸载多 2 或 3 次空发布（树表 0 条的一版上不推，见定案那一句），第一版几何上每次 8 个单元、两盘合计 21 次写调用、279040 字节（第一轮攻方 t10、t5a，副本上量的）；卸载之后 F = 现行那一版的 txg，回退候选集只剩现行那一版。用户选时知情的代价是「停机时崩溃兜底只剩最新一条根」（按条款推的）；第二轮在卸载之后再挂、最新 j 条根坏的 75 行上恢复全读对、可写挂载全挂上，这一条没出现：F 之下的根还在环里，它们的记录能把状态重放回这一串的末尾（第二轮判决 G4）。
- **用户知情接受的代价三条**：删掉的空间要再发生 3 次改变用户可见状态的发布才回可分配集合，在那之前 `df` 不把它算空闲、按 `df` 写它是真 ENOSPC；`df` 少报 = 保留池 + 残留 = 15 + 13 c_max（c_max = 5 / 10 时 80 / 145 块）再加滞后量，而滞后量 = 4 × 最近四个窗口的释放量、**没有上界**（一窗删一个 8 块对象时平均约 34 块、最大 44 / 52，一窗删 32 个时 1044 块），删掉 s 字节 `df` 一个字节都不涨；c_max 按现算取，第一版池规模 9 块 ⇒ 保留池 73 块。
- **最少保留 4 个状态只在一次处置里根槽写失败 ≤ k_tol 次时兑现**：S = 4、c₀ = 5、3 次失败那一格违反 24 次，按字面记一次输、按 k_tol 的射程在外；全枚举下 S = 4、c₀ = 1 也违反 160 次（同为超过容忍的格）。
- **代价是选项本身的，不是这一族的缺陷**：一个被保留的状态钉住的是在它那一刻活着、后来才死的全部块，选最近 4 个就是每次删除都撞上——滞后量与「删了要等 3 次状态」都从这里来。
- **回收门槛取环里最旧有效根的代价**：崩在记录之后、根槽之前那一形留在环里的旧根要等被覆盖一圈才离开，每个洞回收门槛多压一圈（正推腿端口上 S = 16 平均 893 块对窗口门槛 583 块）；一块盘持续写不进根时扣住的单元随发布线性涨、没有上界（攻方模型上量，持续写失败 15 / 30 / 60 / 120 次时最多扣住 50 / 80 / 140 / 260）；崩在记录之后、根槽之前连崩 13 次之后挂载（S = 4），已分配 166 块、窗口门槛同一段 26 块（攻方副本上量）。数两腿各量一种口径，不是入库装置上的数；用户 2026-10-04 知情定。
- **E139（按盘回退下界的收严形态） 答不了的**：计数模型，按发布计不按墙钟；只建根一级的引用、没建 journal 记录重放；实例切换的挂载时预留由 D28（挂载期承诺量） 已定项 3 扣；容量 4000 块、两块盘；k_tol 固定 2；滞后量口径是写登记的人替状态数读法构造的；保留池扫描的最小值落在扫描下沿、真正要多少没量到；环深下限是探针的数。两轮前身 E135（动态回退下界） 与 E138（按盘回退下界与推空的空间要求） 的结论已被 E139（按盘回退下界的收严形态） 取代，不再当依据用。

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
- 进程内重读读得出而自证不过按记的最大 txg 算、重读读出有效根的槽算进这个进程知道住着根的槽、抬 F 的上限把挂载那一刻读不出的槽当作可能是非空有效根：代码轮第二轮判决 `research/prompts/m3-root-ring-code-r2-main-verification.md` 第二节第 2、6 行（有效根所在的槽挂载那一刻读错、重读又读坏，实现当没有根，回收门槛越过盘上有效根，攻方模型上 12 组参数 12 组红，改法 A 12 组 0 红；抬 F 越过读不出的有效根那一形，攻方探针上 F = 22、读得出时 F = 21，推的）与第四节 RS1、RS3 两行（改法 A 的代价是从没写过的槽加挂载读错时记账多，安全方向，2 组；上限算进读不出的槽的代价是读故障时 F 抬得少一点，推的）；用户定案见同一份判决第六节，原话在变更史。无实验：数在攻方模型上量，没立实验号。
- 正常卸载那一串回收的槽只放开一截：设计轮判决 `research/prompts/m3-unmount-after-short-continuation-r1-main-verification.md`（续做推满之后卸载在预演里被拒，丙1 站住、零轮）；代码轮第三轮判决 `research/prompts/m3-unlink-code-r3-main-verification.md` 第二节 N1 各行（丙1 在单盘幸存时写掉被择中的旧根引用的单元，两次抽样同向打中，主 agent 复跑坐实；恢复择根不看 F）与岔路单 `research/prompts/m3-unlink-code-r3-forks.md`。用户定案 2026-10-03：「丙 1 进下一批」、单盘那一问「甲 GA（推荐）」，原话在变更史。无实验：数在三方腿的拷贝上量，没立实验号。
- 有一块盘没有有效根时整截扣、读不出的槽当没有根、卸载半途崩溃之后的下一次可写挂载只放开一截：代码轮判决 `research/prompts/m3-batch14-code-r1-main-verification.md` 第二节（N1「根环里读不出的槽、有洞」那一行是代码取的保守读法；「形 R」那一行攻方打中、主 agent 复跑坐实，`research/prompts/m3-batch14-code-r1-main-checks/`）与岔路单 `research/prompts/m3-batch14-code-r1-forks.md`。用户定案 2026-10-04 选「甲 再挂也只放开一截（推荐）」，原话在变更史。无实验：数在三方腿的拷贝上量，没立实验号。
- 再挂时某块盘一条有效根都读不出只放到最旧有效根（该判决写作一圈窗口里最旧有效根，随回收门槛退回环里最旧有效根）、当场放开的式子带上最旧有效根那一项：代码轮判决 `research/prompts/m3-batch16-code-r1-main-verification.md` 第二节（N1「再挂期间某块盘根环整段读坏」那一行攻方打中、主 agent 复跑坐实，`research/prompts/m3-batch16-code-r1-main-checks/`；窗口那一项是正推腿报的条款漏写）与岔路单 `research/prompts/m3-batch16-code-r1-forks.md`。用户定案 2026-10-04 选「甲 只放到窗口里最旧有效根（推荐）」，原话在变更史。无实验：数在三方腿的拷贝上量，没立实验号。
- 树表 0 条那一版上正常卸载一个字节都不写、F 不动：用户定案 2026-10-04（选乙「不推，条款照实现写」，原话在变更史），把实现里主 agent 2026-09-26 那条定案（`crates/singlefs-core/src/mount.rs` 的 `Unmounted::NothingWrittenOnAVersionWithoutFile` 的注释，推的，被攻过零轮）写进条款；代码轮判决 `research/prompts/m3-formal-code-b16-r1-main-verification.md` 第二节 K1 与第五节第 1 行（照字面推空发布那一读法在攻方副本上量过：模型推 2 条、对拍报 PublishCount）。无实验：数在三方腿的副本上量，没立实验号。

**欠**：C282（环里最旧根没有定义）——定义与条款写在已定项 1 里，会红的检查仍欠；C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元）（与 defer 窗口取多大无关，病根在 D23（journal 的角色与格式） 已定项 14 的影子账）；C329（写行那次发布之前推抬 F 的空发布没有检查）；C563（读根环一槽读坏就当没有根，别的调用方没判）；C660（回收门槛改成一圈窗口之后按盘回退下界的装置要重跑）。

```

**出处 `.claude/kb/experiments/156-alloc-basis四条岔路的代价数.md:1-19`（整段抄，未转述）**

```markdown
## E156 alloc-basis 四条岔路的代价数 —— 部分已跑（2026-09-26 第 4 次重跑：前提 1 成立（今天的代码上「记录已持久、根槽没持久」的崩溃之后可写挂载仍造得出洞）、前提 2 不成立（两个 F 口径在抬 F 那一串上合成一个）⇒ 岔路单第 3 行失效、不量；前提 3 只有岔路 2 那一族按旧式子定义、Hh 历史碰不到新准入式子；岔路单第 1 行在 38 条排得下的历史 × 「实」「每」两个回收时点上量完、按登记的够判条件够判：12 个取样点上 Q1c-前 都越过 2 槽（h_缺 = 1 时已 10 槽）、Q1d 都随 h_缺 增长，F1 触发；岔路单第 1 行用户 2026-09-27 定留到里程碑三、要考虑重排算法；入库装置，确定性，17 单测 / 装置的 22 条变异全抓；岔路单第 2、7 行用户已定不量；2026-10-05 第 5 次重跑：core 断言之后 W7 那一格的阳性对照换成 PC-钉门槛、6 个取样点两列全看到；第 4 次读数在新 `crates/` 上规范化之后 294 行不同（F1 仍触发，Q1d 在「实」列 4 个取样点不再随 h_缺 增长，K10 在 34 个观测点不符），归属跑证明不是 31-1 带来的，哪一份承重交主 agent 定；2026-10-05 第 6 次重跑：回收门槛照用户 2026-10-05 定乙读（挂载时读回全零的根环槽钉在它可能住过的最旧 txg）之后，38 格 K10、F24 都是 0，副本里去掉 core 的全零钉住时两项都红在推出来的 34 个观测点上；「实」列与第 5 次逐字段相同，「每」列随之与「实」列相同，Q1d 在「每」列 4 个取样点由随 h_缺 增长变成不单调，F1 仍触发、Q1c-前 判定不变；PC-钉门槛 六格两列与第 5 次逐字节相同；单测 19 条与变异 M45–M47 没跑，归提交时）

<!-- doc-lint:not-numbers K1 K8 S1 S2 S3 S4 S5 S9 V1 V2 V3 V6 V7 V8 V10 V11 V13 F1 F2 F3 F4 F5 F6 F7 F8 F9 F10 F11 F12 F13 F14 F16 Q1 Q2 Q3 Q7 R1 R2 R3 R4 R5 R6 R7 R8 G7 G8 G12 G26 G27 G28 H0 HF K9 U5 U7 U8 U11 U13 F23 F24 M32 M34 A10 A14 F26 F27 F28 F29 F30 K10 K15 L1 L3 M35 M44 S11 S13 S14 U14 U19 V18 W7 W14 P7 Q1r5a Q1r5b Q2d R1a R1b R1c R1d R2a R2b R2c K16 L6 M45 M46 M47 U20 F31 F32 F33 F34 F35 V19 V20 S15 S16 S17 S18 -->
本文里 K1、锚1/锚2/锚3（对应登记「七」第二类锚点表逐行的本地简写，不写成编号形状——kb 别的登记表已经用同一形状登记过别的含义，这里避开撞号）、K8、K9、S1–S5、S9、V1–V8、V10、V11、V13、F1–F14、F16、R1–R8、Q1a–Q1f、Q2a/Q2b/Q2c、Q3a–Q3e、Q7a–Q7f、U5、U7、U8、G7、G27、G12、甲-T1、F-扣、H0、HR、HK、HK-F0、HF、Hh(k)、HX、HY、β0/β1/β2/βK-w/r/f/l2/β_syn/β_F0、Bd(k)、Bk1/Bk2/Bk3
这些编号与代号都是 `e156-preregistration.md`／`e156-r2-prereg.md` 跑前登记自己的锚点 / 判据 / 历史族 / 臂名，登记位在那两份文件里，不是 kb 的 D / E / C / I 编号。

<!-- doc-lint:registry name-col=2 -->

| 编号 | 简称 |
|---|---|
| X8 | 小池耗尽两口径构造 |
| P21 | 释放越早等得越久 |
| I3 | 一次挂载分配记录只增不减 |
| N2 | 释放时改写的分配代字段 |

**备料**：给 alloc-basis 四条岔路（`research/prompts/alloc-basis-forks.md`）交用户之前配代价数——判决收口表第 ② 行因此挡着第 5、60、54、43 行；用户 2026-09-21 定「先跑计数实验，量完一次全交」。第二、七行判决标着「无运行时代价」，第二行量的是「删掉 D28（挂载期承诺量） 已定项 1『− defer 待释放』这一项要同步改的清单能不能被一条命令核」，第七行量的是「G27（给 defer 账加的一条检查）分不分得出今天两条检查判不出的差别」。等 D28（挂载期承诺量） 已定项 1 与 D16（发布语义） 已定项 1 按岔路定案时引用本实验。岔路单第 2、7 行用户已定（岔路单那两行的状态列），第 3 行按第 4 次重跑的前提 2 失效、不量；第 1 行用户 2026-09-27 定留到里程碑三、要考虑重排算法（弹窗原话「可以留到里程碑3 但是要考虑重排算法」），记成欠账 C566（环上有洞时多扣的槽随洞数无界增长）。

**问题**：第 4 次重跑登记 `research/prompts/e156-r4-prereg.md`（回答问题单 `research/prompts/e156-r4-questions.md` 的四节）。实一至实五之后，管理员回退改成挂着时的一次向前发布（挂载时回退与截断删了）、每次发布都做挂载内回收、准入不再扣「defer 待释放」；岔路单第 2、7 行用户已定不量，第 1、3 行先答三个前提：前提 1（今天的代码上根环还会不会出洞）、前提 2（两个 F 口径在今天的抬 F 路径上还分不分得开）、前提 3（哪几个量是按旧准入式子定义的）。前提 1 成立才量第 1 行（登记第五节 5.5 的第二段），前提 2 不成立则第 3 行失效、不设量。装置是**入库装置**：`crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs`，只读驱动真实 mkfs、暖机、新池新建文件、覆盖写、空发布、可写挂载、挂着回退、抬 F，不改 `crates/singlefs-core`、`crates/singlefs-checker` 的生产代码。

```

**出处 `.claude/kb/experiments/156-alloc-basis四条岔路的代价数.md:20-93`（整段抄，未转述）**

````markdown
### 第 6 次重跑（2026-10-05）：回收门槛照定乙读之后 38 格 K10、F24 都是 0，去掉 core 的全零钉住正好红在推出来的 34 点

重跑登记 `research/prompts/e156-r6-prereg.md`（差分形态，以第 5 次登记为底，含末尾「主 agent 认定：第 6 次登记交回」六条），问题单 `research/prompts/e156-r6-questions.md` 三行：第 1 行（装置的回收门槛照定乙读之后，今天的 `crates/` 上 38 格的 K10 与 F24 还有没有不符的点，另在副本里去掉 core 的全零钉住证这一判分得出）、第 2 行（岔路单第 1 行的读数在新门槛下各是什么、与第 5 次产物比哪几格变了，只报不重下判决）、第 3 行（PC-钉门槛 六格两列还都判「不符」、都含 6S − 1 吗）。起因是第 5 次的 K10 34 点不符：core 照用户 2026-10-05 定乙，把挂载那一刻读回全零的根环槽钉在它可能住过的最旧 txg（`crates/singlefs-core/src/allocator.rs` 的 `RootRingOccupant::AllZeroAtMount`），装置与锚点模型照的是旧读法。

**起跑前提与开跑前的核**（登记 S15、S2、S3）：

| 项 | 命令与原样输出 | 结论 |
|---|---|---|
| 第 31 批代码轮判决写完 | `ls research/prompts/m3-formal-code-b31-r1-main-verification.md` 在；派发提示写明判决已写完 | 成立 |
| 装置没改过 | `sha256sum crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs` → `e69e423a6fda2d973f1b7d4b741bbb280e54d70723d953fda666bba00e2bfe20`，`wc -l` → `5819` | 成立 |
| core 里有全零钉住 | `grep -c AllZeroAtMount crates/singlefs-core/src/allocator.rs` → `11`；`grep -c '            &ring_slots_reading_back_all_zero,$' crates/singlefs-core/src/mount.rs` → `1` | 成立 |
| 条款没变（S2） | 登记命令一出口 `a54efaa24231508eb8a0d99ab6b9a4ed761ae337b1af256388f163f79c2a5bbb`、与第 5 次第二节 `diff` 0 行；命令二出口：定乙四段 `a4ac82034664e49c452d9de59ebe7b8f510f01fe15e8dafc2f6ab6764b3805f7`、问题单 `7889590e8aed051fc02b3edd4ae4686ca0e6d4ba1dd85c9ee191d24ee27a1a8c` | 不触发 |
| 锚点脚本（命令三、四、五） | 汇总 `700ec0b0…`（第 4 次与第 6 次脚本同）、`--dump` 第 4 次 `7bb1e088…` 第 6 次 `47a7d193…`（各 1092 行，`diff` 23 行、`bab6a323…`）、`--dump --pins` `8a222fe4…`、`count_pin_points.py` 输出 `9d5cf9ce…`、`lag_bound_r5.py` 在两份 `--dump` 上都是 `b980971a…` | 与登记第十三节相同 |
| 实现快照（S3） | 编产物二进制之前、主产物跑完之后：`git diff -- crates/ \| sha256sum` 都是 `ad4c31366a677a2df4f08c069d77fc7ebc04345921bcd83371b0f32076ba6ae0`，`git status --short crates/ \| sha256sum` 都是 `8995a6f9b7c12c51e377589e09dc69beb5bb6ebe758392ace82f9825cd129a28`（152 行）；PC-全零钉 跑完之后再取一次，`git diff` 只多了 `crates/mutations.tsv` 末尾追加的三行变异 | 不触发 |

**装置怎么改的**（登记第五节 5.3）：`crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs` 5819 行 → 6205 行，19 条单测（`grep -c '#\[test\]'`，第 5 次 18）。挂载点在交给 `mount_writable` 之前读切段镜像的 3S 个槽，分自证过、读回全零、读坏三类（`read_root_ring_slot_content`）；挂载交回之后按新实例第一个 txg 给每个全零槽取「落到它、小于第一个 txg 的最小 txg」，整张换进 `HhRunState` 的钉住表，打一行 `name=all_zero_pins_at_mount`。`read_hh_state` 在算门槛之前对钉住表里每一槽读这一刻那一槽的根：根的 txg ≥ 第一个 txg 算这个进程写掉了、< 第一个 txg 算重读读出（另计 `released_by_reread_only`），都不是的照旧钉着、并入门槛；`reclaim_threshold_read_from_the_ring_image` 的函数体与 `let every_publish_floor = ring_threshold;` 一个字没动。锚点模型照登记命令四的补丁改同一处。每格在 `name=q1_cell_checks` 之后打 `name=all_zero_pin_cell`，第二段 `name=failure_clauses` 之后打 `name=all_zero_pin_summary`；已有的行一行都没改格式。新单测 U20 与变异 M45–M47（`crates/mutations.tsv` 第 2786–2788 行）。只对这一份文件跑了 `rustfmt --edition 2021`，`rustfmt --check` 无输出（第 4999 行起的 `use` 块随之改好）。`compile-then-swap.py` 在仓副本里 `cargo build` 与 `cargo test --no-run` 都编过才换进主工作区（换上之后 sha256 `3129ec95e2f129ef0a49f1be15a998aae5c4abb508615a8663cdece91d1ef279`）。

**产物**：主产物 `research/results/e156-alloc-basis-counts-2026-10-05-r2.out`（4063 行，sha256 `1c8812b19456165b10b6cd01f3e00fb2ddd1e3673f700cedac151f5da778176b`，完成标记 `E7RESULT name=done emitted=4063`），用仓根 `target/release/` 的二进制（sha256 `1227561466a0008a1b1a7004bafe73d1d71cff7b1af1a51eaab19c40416f2a48`）跑，同一个二进制再跑一遍逐字节相同；`replay.sh` 的驱动用 debug 二进制，debug 一遍约 13 分钟，执行员前台跑不完，只跑了前 225 秒，那 1803 行与 release 产物的前 1803 行逐字节相同，整份 debug 复跑交 `replay.sh` 的 exact 比对。判别力对照 `research/results/e156-alloc-basis-counts-2026-10-05-all-zero-pin-removed.out`（4063 行，sha256 `eaf6b8b63d3b1d899377559a5f09fb32f6774cd14031f41567898c62873679ff`），副本里对 core 的改动 `research/results/e156-alloc-basis-counts-2026-10-05-all-zero-pin-removed.patch`（`mount.rs` 一行，挂载时一个全零槽都不装进根环表，与调查报告 x1 补丁的块体逐行相同、只差行号），同一份装置源码在副本里另编 release。

**第 1 行：照定乙读门槛之后 38 格 K10、F24 都是 0**（登记 R1a、R1b）。主产物 `:4050`、`:4048`、`:4049`：

```
E7RESULT name=verdict stage=second cells_run=38 cells_not_fitting=4 truncated_cells=0 v1_over_withheld_controls_void=0 v2_a10_every_mismatch_cells=0 v2_a12_every_nonzero_points=0 v2_a14_mismatch_points=0 v2_a15_every_below_points=0 v2_pc_pinned_floor_seen=true v2_pc_c2_seen=true v6_a11_mismatch_points=0 v6_devices_unequal_points=0 v6_pc_hole_not_seen_cells=0 v16_judge_cases_wrong=0 v17_a13_every_nonzero_points=0 s1_failures=0 s1h_root_ring_missing_points=0 s11_a10_real_mismatch_cells=0 s11_a12_real_nonzero_points=0 s11_a15_real_below_points=0 s11_k10_violation_points=0 s12_g_adm_nonzero=0 k8_mismatches=0 referenced_but_free_points=0 abandoned_or_unknown_root_points=0 anchor_rows_missing=0
E7RESULT name=all_zero_pin_summary cells=38 cells_with_pins=16 pins_installed=64 threshold_lowered_points=34 released_by_reread_only=0 bad_slots_at_mount=0 bad_slots_still_pinned=0
```

逐格：`grep ' name=q1_cell_checks ' research/results/e156-alloc-basis-counts-2026-10-05-r2.out | grep -o 'k10_violation_points=[0-9]*' | sort | uniq -c` → `38 k10_violation_points=0`；同一命令换 `real_every_differ_points` → `38 real_every_differ_points=0`；`failure_clauses` 行（`:4048`）的 `f24_real_every_differ_points=0`。38 格是 S ∈ {4, 8, 16} × ρ ∈ {1, 1/4} 的 k = 0 六格、洞位置为前与为后各 16 格。按登记第八节 8.2 的写法：K10、F24 在这 38 个取样点上一致为 0，只在这 38 个取样点上。

**判别力（登记 R1c，PC-全零钉）**：同一份装置配去掉全零钉住的 core，判别力对照 `:4050` 的 `s11_k10_violation_points=34`、`:4048` 的 `f24_real_every_differ_points=34`（整行在「判决行点名」）。不为 0 的观测点集合用 `research/results/e156-alloc-basis-counts-2026-10-05-r6-point-sets.py`（只读两份产物与锚点脚本的两份输出）与登记 L6 × 两个 ρ 比，输出 `research/results/e156-alloc-basis-counts-2026-10-05-r6-point-sets.txt` 第 1、3 行：

```
L6_points_x_rho=34
pc: k10_points=34 f24_points=34 k10_eq_L6=True f24_eq_L6=True k10_minus_L6=[] L6_minus_k10=[] f24_minus_L6=[] L6_minus_f24=[]
```

即 K10、F24 都红在前洞格 txg h + 3S − 1 那一个观测点上（S = 4 的 16、21，S = 8 的 28、33、39、45，S = 16 的 52、57、63、69，按 k 取前 k 个），后洞格与 k = 0 的格一点不红；作废 V19 与停机 S16 都没触发。两个输入文件是登记命令四 `--dump --pins` 与命令五的原样输出：`research/results/e156-alloc-basis-counts-2026-10-05-r6-anchor-pins.txt`（`8a222fe4…`）、`research/results/e156-alloc-basis-counts-2026-10-05-r6-anchor-lowered.txt`（`9d5cf9ce…`）。

**钉住簿记（登记 R1d）**：同一份输出第 4–7 行：

```
R1d_1: hh_mount_rows=68 checked=68 mismatches=[]
R1d_2: cells=38 mismatches=[]
R1d_3: cells_with_reread_release=[]
R1d_4: cells_with_bad_slots=[]
```

68 次挂载（含第一段那两段历史）每次钉的 txg 集合与锚点脚本那一格挂载做完那一行的 `pinned` 相同，后洞格与 k = 0 一槽不钉；每格 `threshold_lowered_points` 等于 L6 那一格的点数；`released_by_reread_only` 全是 0、`bad_slots_at_mount` 与 `bad_slots_still_pinned` 全是 0，停机 S17、S18 不触发。锚点逐行相同（登记第七节 7.2 的最后一条断言）：主产物 `name=anchor_dump` 1092 行去掉前缀之后与命令四 `--dump` 的 `DUMP` 行 `cmp` 无输出（作废 V20 不触发）。

**第 2 行：岔路单第 1 行的读数与第 5 次比**（登记 R2a–R2c，只报、不重下岔路单第 1 行的判决）。比较脚本 `research/results/e156-alloc-basis-counts-2026-10-05-r6-vs-r5-compare.py`，输出 `research/results/e156-alloc-basis-counts-2026-10-05-r6-vs-r5-compare.txt`（146 行）第 1 行：

```
pairs=102 unequal_pairs=42 only_old=0 only_new=0 differing_fields_real=0 differing_fields_every=122 differing_fields_unsplit=3 judgement_flips=8 other_judgement_changes=12
```

| 读数 | 第 5 次 | 第 6 次 |
|---|---|---|
| F1（`failure_clauses` 的 `f1_front_above_two_groups`） | 12 组全在（触发） | 12 组全在（触发），不翻面，F33 不触发 |
| Q1c-前 判定（实、每两列 12 组的 `judgement_two_slots`、`judgement_one_slot`） | 全 `above`，越过 2 槽与 1 槽的最小 h_缺 都是 1，第一次越过在 txg 7 | 同左，不翻面 |
| Q1c-前「每」列的 Δ 峰值（`by_h_que`，S = 4、ρ = 1，h_缺 = 1） | `peak10` | `peak20`，与「实」列相同（「每」列 6 组的 `by_h_que` 都变了） |
| Q1d（`q1d` 的 `growth_by_h_que`）「实」列 | s=4 ρ=1、s=4 ρ=1/4、s=8 ρ=1/4、s=16 ρ=1/4 `not_monotonic`，其余 `grows_with_holes` | 同左 |
| Q1d「每」列 | 12 组全 `grows_with_holes` | 与「实」列相同：上面那四组翻成 `not_monotonic`（F34 触发，照报） |

翻面的 8 行是比较输出第 2–9 行（`q1d` 与 `verdict stage=second_group` 的 `q1d_growth` 各 4 组，都在「每」列）。判定字段变了而不属于 Q1c-前、Q1d 的 12 处在第 10–21 行：`q1c quantity=q1c_not_on_holes` 的「每」列 6 组两个判定都由 `at_most` 变 `above`。R2c：归「实」列的不同处 0 个（`differing_fields_real=0`），「实」列与第 5 次逐字段相同，F35 不触发。第 6 次产物里每组的 `q1c`、`q1d`、`q1c_front_threshold_moved` 行去掉 `timing=` 之后实、每两行逐字相同（`sed 's/ timing=[a-z_]*//' | sort | uniq -c` 每组都是 2）。

**第 3 行：PC-钉门槛**（登记 R3）。主产物 `name=pc_pinned_floor` 六行与汇总行（`:3908`–`:3914`）与第 5 次产物同名七行 `cmp` 逐字节相同：六格实、每两列 `judge_*_mismatch=1`、`contains_bound_*=true`，汇总 `E7RESULT name=pc_pinned_floor_summary cells=6 seen_real_cells=6 seen_every_cells=6`，`v2_pc_pinned_floor_seen=true`。

**判决行点名**（`grep -n 'name=verdict' <产物>`）：主产物 14 条（`:1217`、`:4050`–`:4062`），没有取值为 `false` 或 `not_run` 的字段，违例、不匹配、失败类的整数计数都是 0；`premise_two_answer=not_holds`、`fork_three=void_not_measured` 与第 5 次相同（前提 2 不成立、岔路单第 3 行失效）。判别力对照 14 条（行号同），不为 0 的两项：`s11_k10_violation_points=34`（登记 R1c 要的那一项）与 `s11_a15_real_below_points=16`（`:4050`；16 个前洞格各 1 点，「实」列 Q1a 低于锚点模型照定乙读算的树表下界，推的：core 不钉时在 txg h + 3S − 1 提前回收了一个树表单元；登记没写这一项的预期，对照不是被测状态）。完整行：

```
E7RESULT name=verdict stage=second cells_run=38 cells_not_fitting=4 truncated_cells=0 v1_over_withheld_controls_void=0 v2_a10_every_mismatch_cells=0 v2_a12_every_nonzero_points=0 v2_a14_mismatch_points=0 v2_a15_every_below_points=0 v2_pc_pinned_floor_seen=true v2_pc_c2_seen=true v6_a11_mismatch_points=0 v6_devices_unequal_points=0 v6_pc_hole_not_seen_cells=0 v16_judge_cases_wrong=0 v17_a13_every_nonzero_points=0 s1_failures=0 s1h_root_ring_missing_points=0 s11_a10_real_mismatch_cells=0 s11_a12_real_nonzero_points=0 s11_a15_real_below_points=16 s11_k10_violation_points=34 s12_g_adm_nonzero=0 k8_mismatches=0 referenced_but_free_points=0 abandoned_or_unknown_root_points=0 anchor_rows_missing=0
```

**它答不了的（第 6 次）**：① 内联单测 19 条（含新 U20）与 `crates/mutations.tsv` 里点着这个装置的 26 行变异（含 M45–M47）都没跑（checker 档，归提交时），只核了 26 行原文在新源码里各命中 1 次、M45–M47 三条同时施加在副本里 `cargo test --no-run` 编得过；② 这一族只造第一圈里的前洞，那一槽在 [0, 第一个 txg) 里只住得下一个 txg，「钉在最旧 txg」与「钉在能住的最大 txg」同值，分不出 core 钉的是哪一个；③ 「重读读出根才解开」那条路在这一族上一次没出现（`released_by_reread_only` 全 0），装置只计数、不证它；④ 读坏的槽、准入抬 F 的上限这一族不造；⑤ PC-全零钉 证 core 少钉全零槽时装置看得见，不证装置旁路自己读没读对定乙（那一半由锚点逐行相同与 U20 盯）；⑥ 主产物用 release 二进制跑，debug 那一份只比了前 1803 行；⑦ 「实」列与第 5 次相同，第 5 次之后 `crates/` 别的改动碰没碰这一族之外的量没查。

````

**出处 `.claude/kb/experiments/156-alloc-basis四条岔路的代价数.md:94-178`（整段抄，未转述）**

````markdown
### 第 5 次重跑（2026-10-05）：W7 那一格的对照换成 PC-钉门槛，第 4 次读数在新 `crates/` 上 294 行不同

重跑登记 `research/prompts/e156-r5-prereg.md`（差分形态，其余各节同第 4 次登记 `research/prompts/e156-r4-prereg.md`），问题单 `research/prompts/e156-r5-questions.md` 两行：第 1 行（core 在分配器没装根环表而记了一条根时断言之后，装置里 W7 那一格的阳性对照 PC-分配器 造不出来，换成什么）、第 2 行（第 4 次已够判的读数在新 `crates/` 上还相不相同）。起因是用户 2026-10-05 定的 C551（`build_pool` 的分配器没有根环表）：`crates/singlefs-core/src/allocator.rs` 的 `record_root_written_by_this_process` 在没装根环表时 `expect`，消息以「分配器没装根环表却记了一条根」开头。

**起跑前提与开跑前的核**（登记第 5 次的停机 S13、S2、S3）：

| 项 | 命令与原样输出 | 结论 |
|---|---|---|
| 31-1 的断言在主工作区 | `grep -c '分配器没装根环表却记了一条根' crates/singlefs-core/src/allocator.rs` → `2`（`allocator.rs:1920` 断言消息、`:2999` `should_panic`） | 成立 |
| 第 31 批合入验证绿的目录 | 派发提示写明 `/tmp/claude-1000/merge-and-verify/2026-10-05-2822486/` | 成立（目录里的日志执行员没重读） |
| 装置只多 31-3 的改动 | `git show :crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs \| sha256sum` → `f1930d4e…`（登记写时的样子）；`git diff -- 同一文件` 两块，与 `/tmp/claude-1000/m3-formal-31-3/patch/crates.patch` 前两块逐字相同：一块是 `mod tests` 的 `use` 里加 `root_ring_occupancy_after_make_filesystem`（函数体之外），一块是 `new_pool_file_creation_state` 函数体里装根环表 | 登记写的是「只在函数体里」；多出来的那一行 `use` 是同一块补丁带的，执行员认为不改读数、照样开工，写进交回由主 agent 认 |
| 条款没变（S2） | 登记第十三节命令一原样重取五段，`sha256sum` → `a54efaa24231508eb8a0d99ab6b9a4ed761ae337b1af256388f163f79c2a5bbb`，与登记第二节同一个 | 不触发 |
| 锚点脚本（命令三、命令四） | `anchors_e156_r4.py` 的输出 `700ec0b0…`、`--dump` `7bb1e088…`（1092 行）、`lag_bound_r5.py` 的输出 `b980971a…`，与登记第十三节同一个 | 相同 |
| 实现快照（S3） | 产物前、第 1 次产物后、第 2 次复跑后各取 `git diff -- crates/` 与 `git status --short crates/` 的 `sha256sum`：`31db3aba132749d5fd643e4ef8e013af15896679f953b73dd4bb8265caccf58d`、`8c727d9e15719e32467975a09ff64b3b445ff1971be7ebf51ef49e7b326a34a0`，三次相同 | 不触发 |

**装置怎么改的**（登记 5.5 的 E1–E4）：`crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs` 5783 行 → 5819 行。删 `HhAllocatorSource::WithoutRootRingPositiveControl`、`allocator_without_the_root_ring` 与 `main` 里 PC-分配器 那一块；`HhAllocatorSource` 与成员 `ProductPathWithRootRing` 名字不改（`name=q1_cell` 行打 `allocator={:?}`）；加 PC-钉门槛（格表之后、PC-多扣 之前，对格表里 `hole_count == 1` 且位置为后的 6 格用同一个 `judge_hole_history_outcome`，期望取同一 S 的 `anchor_model::tree_table_reclaim_lags`，6S − 1 现算为 `2 * E156_ROOT_RING_REGIONS * slots_per_region - 1`）；`name=verdict stage=second` 里 `v2_pc_allocator_seen` 换名 `v2_pc_pinned_floor_seen`；新单测 U19。`grep -n 'PoolAllocator::new(' crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs` → `5041:        let mut allocator = PoolAllocator::new(`（`new_pool_file_creation_state` 里，第 5071 行 `install_root_ring_occupancy` 紧跟着装根环表）。单测数 `grep -c '#\[test\]' 同一文件` → `18`（第 4 次 17）。`compile-then-swap.py` 在草稿副本里 `cargo build` 与 `cargo test --no-run` 都编过才换进主工作区。

**第 1 行：新对照 PC-钉门槛**。产物 `research/results/e156-alloc-basis-counts-2026-10-05.out`（3955 行，sha256 `a3d29fe0e31a570aca19db52a6fda9f7600017f43dbcfa856996d9126085931c`；同一个二进制再跑一遍与它逐字节相同，`cmp` 无输出）：

```
E7RESULT name=pc_pinned_floor s=4 rho=1 k=1 position=back expected=[12] lags_real=[12,13,14,15,16,17,18,19,20,21,22,23,24] lags_every=[12,13,14,15,16,17,18,19,20,21,22,23,24] judge_real_mismatch=1 judge_every_mismatch=1 bound=23 contains_bound_real=true contains_bound_every=true seen_real=true seen_every=true
E7RESULT name=pc_pinned_floor s=4 rho=1/4 k=1 position=back expected=[12] lags_real=[12,13,14,15,16,17,18,19,20,21,22,23,24] lags_every=[12,13,14,15,16,17,18,19,20,21,22,23,24] judge_real_mismatch=1 judge_every_mismatch=1 bound=23 contains_bound_real=true contains_bound_every=true seen_real=true seen_every=true
E7RESULT name=pc_pinned_floor s=8 rho=1 k=1 position=back expected=[24] lags_real=[24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48] lags_every=[24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48] judge_real_mismatch=1 judge_every_mismatch=1 bound=47 contains_bound_real=true contains_bound_every=true seen_real=true seen_every=true
E7RESULT name=pc_pinned_floor s=8 rho=1/4 k=1 position=back expected=[24] lags_real=[24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48] lags_every=[24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48] judge_real_mismatch=1 judge_every_mismatch=1 bound=47 contains_bound_real=true contains_bound_every=true seen_real=true seen_every=true
E7RESULT name=pc_pinned_floor s=16 rho=1 k=1 position=back expected=[48] lags_real=[48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95,96] lags_every=[48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95,96] judge_real_mismatch=1 judge_every_mismatch=1 bound=95 contains_bound_real=true contains_bound_every=true seen_real=true seen_every=true
E7RESULT name=pc_pinned_floor s=16 rho=1/4 k=1 position=back expected=[48] lags_real=[48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95,96] lags_every=[48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95,96] judge_real_mismatch=1 judge_every_mismatch=1 bound=95 contains_bound_real=true contains_bound_every=true seen_real=true seen_every=true
E7RESULT name=pc_pinned_floor_summary cells=6 seen_real_cells=6 seen_every_cells=6
```

六格两列逐个报：Q1r5a（判定判「不符」）实、每两列都是 1；Q1r5b（滞后集合含 6S − 1）实、每两列都是 true；`v2_pc_pinned_floor_seen=true`（`research/results/e156-alloc-basis-counts-2026-10-05.out:3942`）。登记的够判条件满足（作废 V2 没触发、F28、F29 没触发）：新对照在 6 个取样点上证出了「A10 分得出真实分配器把回收推迟一圈」。它只证推迟，不证全程不回收（那一形由变异 M35 改写后的行盯，提交时跑），也不证 W7 那一格的核认得出没装根环表的分配器（那一件由 core 的单测接管）。滞后集合是 3S 到 6S 的整段（S = 4 时 12 到 24），不是只有 6S − 1 一个值；6S − 1 与 6S 是登记命令四算出的释放代 6、5 的滞后（`lag_bound_r5.py` 的输出 `release_first_free_lag=[(5, 28, 24), (6, 28, 23)]`），整段里别的值产物没按释放代拆开，没核（命令 `grep -c '^E7RESULT name=pc_pinned_floor ' research/results/e156-alloc-basis-counts-2026-10-05.out` → `6`，汇总行 `grep -c '^E7RESULT name=pc_pinned_floor_summary '` → `1`）。

**第 2 行：第 4 次读数在新 `crates/` 上逐字段重比**（登记 Q2a、Q2b）。规范化：第 4 次产物 `research/results/e156-alloc-basis-counts-2026-09-26-r2.out`（4007 行）去掉带 `family=PC-allocator` 项的行、第一项是 `name=pc_allocator` 的行、`name=verdict stage=second` 行里的 `v2_pc_allocator_seen=` 一项；新产物去掉 `name=pc_pinned_floor`、`name=pc_pinned_floor_summary` 行与同一 verdict 行里的 `v2_pc_pinned_floor_seen=` 一项；「第一项」按行首 `E7RESULT ` 之后的第一个空白分隔项算。比对脚本与全部输出都在仓里：

```
python3 research/results/e156-alloc-basis-counts-2026-10-05-q2-normalise-compare.py research/results/e156-alloc-basis-counts-2026-09-26-r2.out research/results/e156-alloc-basis-counts-2026-10-05.out > research/results/e156-alloc-basis-counts-2026-10-05-q2-normalise-compare.txt
head -3 research/results/e156-alloc-basis-counts-2026-10-05-q2-normalise-compare.txt
old_dropped_rows={'family=PC-allocator': 58, 'name=pc_allocator': 1} old_dropped_items=1
new_dropped_rows={'name=pc_pinned_floor': 6, 'name=pc_pinned_floor_summary': 1} new_dropped_items=1
lines_old=3948 lines_new=3948 equal=3654 unequal=294
```

Q2a 判「不同」（登记写死的判据：`unequal = 0` 才算相同）。登记 L3 那一句核过：新产物去掉 7 行 1 项、第 4 次产物去掉 59 行 1 项，去掉的每一行不是 `pc_allocator` 就带 `family=PC-allocator`，V18 没触发。294 行里有 1 行是 `name=done emitted=`（4007 → 3955，恰好是 4007 − 59 + 7；登记的规范化没有把这个计数项摘掉，按字面算进了 294）。

Q2b 按登记的两类分（判决行名单取登记第六节）：

| 类 | 行数 | 键数 | 行名（行数） |
|---|---|---|---|
| 判决行 | 60 | 160 | `q1c`（24）、`q1_cell_checks`（16）、`q1d`（6）、`q1c_front_threshold_moved`（6）、`verdict`（5）、`u13a_rollback_to_oldest_candidate`（1）、`geometry_sensitivity`（1）、`failure_clauses`（1） |
| 计数行 | 234 | 522 | `q1_observation`（82）、`hole_recovery`（69）、`q1_trajectory`（48）、`q1_cell`（18）、`q1_trajectory_extra`（16）、`done`（1） |

不同的行落在哪几格：`name=q1_cell` 的 Hh 行里洞位置为前的 16 格全部不同（k = 1、2、4 各 6、6、4 格），位置为后的只有 k = 2、k = 4 各 1 格，k = 0 的 6 格一格不同；`hole_recovery` 69 行只有一个键不同（`steps_after_root_slot_write` 在 `system_configuration_slot,system_configuration_slot` 之后多了 `,barrier,barrier`）；其余键见 `research/results/e156-alloc-basis-counts-2026-10-05-q2-normalise-compare.txt`（979 行，每个不同的行逐键列旧值、新值）。判决层面的几处：

| 字段 | 第 4 次 | 新产物 |
|---|---|---|
| `failure_clauses` 的 `f24_real_every_differ_points` | 0 | 34 |
| `verdict stage=second` 的 `s11_k10_violation_points` | 0 | 34 |
| `geometry_sensitivity quantity=q1d_growth` 的 `result` | `consistent_on_12_sampling_points` | `unstable`（`research/results/e156-alloc-basis-counts-2026-10-05.out:3940`，12 个取样点里「实」列的 s=4 ρ=1、s=4 ρ=1/4、s=8 ρ=1/4、s=16 ρ=1/4 四点 `not_monotonic`，其余 8 点 `grows_with_holes`） |
| `verdict stage=second_group` 的 `q1d_growth` | 12 组全是 `grows_with_holes` | 4 组 `not_monotonic`（同上四点），8 组不变 |
| `q1c_front_threshold_moved`（S = 4、ρ = 1、实 那一组，新产物 `research/results/e156-alloc-basis-counts-2026-10-05.out:3858`、第 4 次 `research/results/e156-alloc-basis-counts-2026-09-26-r2.out:3910`；「实」列 6 组都不同） | `threshold_between=10` `threshold_at_largest=18` | `threshold_between=18` `threshold_at_largest=20` |
| `u13a_rollback_to_oldest_candidate` 的 `item1_slots` | 374 | 377 |

没变的判决：`f1_front_above_two_groups` 仍是 12 组全在（`research/results/e156-alloc-basis-counts-2026-10-05.out:3941`），即 F1 在新 `crates/` 上照样触发；`name=verdict stage=first` 行（`:1214`）、`v2_a10_every_mismatch_cells`、`v2_a12_every_nonzero_points`、`v2_a14_mismatch_points`、`v2_a15_every_below_points`、`s1h_root_ring_missing_points` 都与第 4 次相同（`:3942`）。`name=verdict stage=second` 整行（`:3942`）：

```
E7RESULT name=verdict stage=second cells_run=38 cells_not_fitting=4 truncated_cells=0 v1_over_withheld_controls_void=0 v2_a10_every_mismatch_cells=0 v2_a12_every_nonzero_points=0 v2_a14_mismatch_points=0 v2_a15_every_below_points=0 v2_pc_pinned_floor_seen=true v2_pc_c2_seen=true v6_a11_mismatch_points=0 v6_devices_unequal_points=0 v6_pc_hole_not_seen_cells=0 v16_judge_cases_wrong=0 v17_a13_every_nonzero_points=0 s1_failures=0 s1h_root_ring_missing_points=0 s11_a10_real_mismatch_cells=0 s11_a12_real_nonzero_points=0 s11_a15_real_below_points=0 s11_k10_violation_points=34 s12_g_adm_nonzero=0 k8_mismatches=0 referenced_but_free_points=0 abandoned_or_unknown_root_points=0 anchor_rows_missing=0
```

**Q2c 归属**：在仓副本里只把 31-1 对 `allocator.rs` 的断言块反着打回去（`record_root_written_by_this_process` 回到 `let Some(occupancy) = self.root_ring.as_mut() else { return; };`），用同一份装置、副本自己的编译目录跑一次，产物 `research/results/e156-alloc-basis-counts-2026-10-05-attribution-without-31-1.out`（3955 行），`cmp` 与 `research/results/e156-alloc-basis-counts-2026-10-05.out` 逐字节相同（两份 sha256 都是 `a3d29fe0…`）。不同不是 31-1 带来的，来自第 4 次产物之后别的批改过的 `crates/`（登记 W14）；登记第四节 P7 的预估（Q2a 相同）错了，F30 的触发条件（Q2c 判出不同来自 31-1）没满足。哪一批带来的没查（登记没立这个量）。

按登记，Q2a「不同」触发 S14 与 F26，K10 在「实」上 34 点不符触发 S11，三条都是交主 agent 的停机，不当结果、不改判据。读码推的一个线索（没跑过、没验）：`crates/singlefs-core/src/allocator.rs` 现在把挂载时读回全零的槽记成 `RootRingOccupant::AllZeroAtMount` 并把回收门槛钉在它可能住过的最旧 txg（`allocator.rs:946` 起的注释，用户 2026-10-05 定乙），而装置 `read_hh_state` 里没有 `AllZero` 的处理（`grep -n 'AllZero' crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs` 无输出）；洞位置为前的格，崩溃那一次的根槽按推算是环第一圈里还没写过的槽（推的，没读产物核），与不同的格大体重合。这一条要对着 `crates/` 与装置两边查，不是这一份产物的结论。

**判决行点名**（`grep -n 'name=verdict' research/results/e156-alloc-basis-counts-2026-10-05.out`）：14 条，行号 `1214`、`3942`–`3954`。没有取值为 `false` 或 `not_run` 的字段；整数计数里不为 0 的，名字表示违例的只有一个：`s11_k10_violation_points=34`（`:3942`，K10 在「实」列 34 个观测点不符，34 点全在洞位置为前的 16 格里；第 4 次是 0，不在登记预期内，触发停机 S11、S14）。`cells_run=38`、`cells_not_fitting=4` 是格数，不是违例。判决行之外同一类字段：`name=failure_clauses`（`:3941`）的 `f24_real_every_differ_points=34`（第 4 次 0）与 `f2_no_observable_difference=false`（取值 false 表示有可观测的差，F2 没触发，与第 4 次相同）。

**第 5 次重跑与页内第 4 次各节的关系**：「这一段做了什么」到「它答不了的」各节写的是 2026-09-26 第 4 次重跑，引的读数出自 `research/results/e156-alloc-basis-counts-2026-09-26-r2.out`，没有随新产物改写。新产物在 Q2a 的规范化之后有 294 行不同，F1 仍触发而 Q1d 随 h_缺 增长的那一句在「实」列 4 个取样点上不再成立（Q2b 判决层面那一组字段）；哪一份承重由主 agent 定（登记 S14），定之前那几节里 `f24_real_every_differ_points=0`、`s11_k10_violation_points=0`、「Q1d 都随 h_缺 增长」这类句子不能原样再引。

**它答不了的（第 5 次）**：① 内联单测与变异没跑（执行员不跑 checker 档）：新单测 U19 与整份装置的 18 条单测、M35 改写与 M44 的实际红绿都没有读数，只验了变异两行的原文在源码里各命中 1 次、替换文在副本里编得过；② PC-钉门槛 证推迟一圈，不证全程不回收；③ W7 那一格认得出没装根环表的分配器这一件，装置里不再有对照，靠 core 的单测 `recording_a_root_on_an_allocator_without_a_root_ring_occupancy_panics_instead_of_never_reclaiming` 与变异「C551：分配器没装根环表记根时静默返回（不断言）」；④ 294 行不同来自哪一批 `crates/` 的改动没查；⑤ `s11_k10_violation_points=34` 是装置读错了还是实现的门槛与条款不同，没两边查完（只有 Q2c 之后那一条读码的线索）。

**要主 agent 在提交时跑的命令**（执行员不跑，重型）：

```
SINGLEFS_HEAVY_TESTS=commit bash scripts/run-with-memory-cap.sh 8G bash scripts/capped.sh 10 cargo test -p singlefs-checker-tier --bin e156_allocation_basis_counts
```

看 `test result:` 行是不是 `18 passed`（Q2d），U19 的名字是 `hh_back_hole_at_four_slots_per_region_delays_the_tree_table_reclaim_to_six_ring_slots_minus_one_and_the_a10_judgement_sees_it`；变异：`crates/mutations.tsv` 第 803 行（旧 M35，替换文 `allocator_without_the_root_ring` 在装置里已不存在，编不过）要换成改写后的 M35、末尾追加 M44（两行的原样内容在 `research/results/e156-alloc-basis-counts-2026-10-05-mutations-replacements.tsv` 与 `research/results/e156-alloc-basis-counts-2026-10-05-mutations-append.tsv`），由门禁 checker-tier-crates-mutation-replay 跑，三个数分开报。

````

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:1-15`（整段抄，未转述）**

```markdown
## E171 三oracle合谋测试 —— 已跑（s4 整表 117 行、F4 扩表 164 行与编不过 6 行的补位都已跑，读数到齐；结论三轮判决已写回）

<!-- doc-lint:not-numbers O1 O2 O3 Q1 Q2 Q3 Q4 Y1 Y3 P1 P2 P3 P4 P5 P8 P9 N1 N2 N3 N4 N5 S1 S2 S3 S4 S5 S6 S7 S8 Z1 Z2 Z3 Z4 F1 F2 F3 F4 F5 F6 F7 X1 X2 X3 X4 X5 X6 X7 X8 X9 X10 X11 X12 X13 X14 X15 X52 X53 X54 X55 X56 X57 X58 X59 X60 X61 X62 X63 X64 X65 X66 X67 X68 X69 X70 X71 X72 X73 A1 A2 A3 A4 A5 A6 A7 A8 A9 A10 A11 V1 V2 W1 K4 -->

跑前登记 `research/prompts/e171-preregistration.md`（含第十二节修订 1–5；Q4 的另一种读法由主 agent 在装置写之前删去，只留整表 ≤ 60 秒那一种）；问题单 `research/prompts/three-oracle-collusion-questions.md` 的 Q1（一张坏实现表在同一批历史与崩溃状态上，有没有一行三条 oracle 全放行）、Q2（手造坏镜像喂 O2、O3，有没有一张两条全放行）、Q3（拿已知被抓的坏实现逐条关掉一条 oracle，剩下的还红不红）、Q4（这道测试一次跑多久、怎么进门禁）。被测条款 D13（验证路线） 已定项 6：三个 oracle 的独立性要靠合谋测试验证，不许自称。

**现状**：坏实现表 117 行用修过的逐行驱动在同一棵冻住的树上跑完一遍（s4，两条线道、每条 2 线程），连同 Q3-核三趟与比对器；s3 那一次整表因驱动把上一行的变异带进下一行而整表作废、留档（「s3 整表作废」一节）。O3 用 E170（O3规约执行器第一版） 第 1 次重跑冻结的执行器，开头与收尾两次冻结核对都是 0 处不同。s4 的分母 D = 19 < 30，登记 F4 触发，扩表 164 行（序号 118–281）在同一份冻住的树上跑完，比对器把两半并成 281 行：分母 D = 37 ≥ 30。两趟里编不过的 6 行（第 33、59、127、163、206、243 行）当时的驱动没按登记 5.2 补位；驱动改成编不过之后补位，补位那一趟在同一份冻住的树上跑完（六行换上的都是未触达的行），比对器把 117 + 164 + 补位并成 `…-s4-f4-substitutes-comparator.out`：分母仍 D = 37，Q1、Q3 名单与补位之前逐字同（「编不过的 6 行补位（已跑）」一节）。读数到齐之后开了三轮结论判决（`research/prompts/e171-conclusion-r1-main-verification.md`、`…-r2-main-verification.md`、`…-r3-main-verification.md`，第三轮是最终轮），判这批读数撑不撑 D13（验证路线） 已定项 6：真盲区候选收成第 15、23、42、242、269、270、271 行，别的全放行行与未触达行里交回值不同的行归另几类（「三轮判决之后的三分」一节）；三分与读数一样只属于 s4 那一棵冻住的树。

| 行 | 判定 | 依据（产物） | 照字面之外要一起读的 |
|---|---|---|---|
| Q1 | 有：甲、乙各 12 行三条全放行（s4 的 7 行加扩表的第 242、269、270、271、277 行），丙 13 行（另加第 233 行）；分母 D = 37（补位之前与之后相同） | `e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out` 的 `name=q1_alpha`、`name=q1_beta`、`name=q1_gamma`（`verdict=yes_a_row_is_quiet_on_every_oracle`）、`name=q1_denominator`（`counted_D=37 … enough_to_judge_none_side=true`）；补位之前的 `…-s4-f4-comparator.out` 里 Q1 三种读法的名单行逐字同 | 照登记第六节 Q1-甲「≥ 1 判『有』」，与分母无关；D ≥ 30 只说「没有」那一边的门槛够了，这一批里「有」已成立。s4 那 7 行的读数在 F4 产物里没变（第 1–117 行的结果行逐字节相同）；扩表新出的 5 行与第 233 行（丙）、批 2 的第 265 行的读数在「F4 新出的全放行行」一节。三轮判决之后的三分在「三轮判决之后的三分」一节：真盲区候选 7 行（第 15、23、42、242、269、270、271 行），另几类 5 行（第 27、112、277 行靠记录重建的恢复，第 34 行分配政策，第 92 行条款在字节表与里程碑验收），没有一行还记「对这一批等价」；第 233 行在丙里全放行（甲乙里 O1-崩溃判官抓到），第 265 行批 2 是取样点不敏感。O3 在 37 行上一行都没红（`name=q1_column_red_rows … O3甲=0 O3乙=0`）。补位之后分母仍 37，「有」没翻 |
| Q2 | 有（照登记字面）：甲、乙各 4 张两条全放行，全是 P8 | 同一份产物的 `name=q2_all_quiet`（`table_rows=107 all_quiet_rows=4 verdict=yes_a_bad_image_passes_both`）、`name=f3_p_rows_judged_in_by_o3`、`name=q2_negative_bases`；O2 放行而 O3 判不在的数在 `name=q2_family` 的 P 族与盲测 B 族两行（`o3_only_strict=12`、`o3_only_strict=17`，整行抄在「Q2 里 O2 放行而 O3 判不在的坏镜像」一节） | 4 张都是 E170（O3规约执行器第一版） 设计员推的「不在」而 O3 判「在」的 P8 行，F3 触发；主 agent 照 F3 判成期望推错、这四张不是坏镜像（判决记在 D13（验证路线） 已定项 6 的射程）。Q2 的数与第二段 `…-s2-comparator.out` 同出一批输入，s4 比对器重算一遍；F4 产物里 Q2、F3 与对照的结果行与 s4 那一份逐字节相同。几何两点（`q2_geometry_unstable_kinds=2`）：翻面的是流一 P2、P5 两种补丁的 O2 一列（底态 F 红、Mk 不红），O3 两个底态都判「不在」，所以这两种补丁的「两条全放行」两点都不成立、不翻；全放行的 P8 两个底态都是 `same_on_both_points`。另有 O2 放行而 O3 判不在的坏镜像 29 张（P 族 12 张加盲测 B 族 17 张，12 + 17）：写「O2 放行而 O3 判不在」，不写「O3 独有」，Q2 只比 O2 与 O3 一对，O1 与 RefFS 没在坏镜像上跑；这 29 张是结构自洽的伪造根、伪造续接镜像，补丁与期望同出 E170（O3规约执行器第一版） 的改法代码，细节在「它答不了的」 |
| Q3 | 有一行只靠一条：甲、乙各 23 行（只靠 O1 的 32、45、108、115、172、233、237、249、250、281；只靠 O2 的 8、65、86、87、90、99、102、135、234、238、245、258、265） | 补位之后的 `…-s4-f4-substitutes-comparator.out` 的 `name=q3_alpha`、`name=q3_beta`（补位之前的 `…-s4-f4-comparator.out` 同名行逐字同）；Q3-核 9 行 `name=q3_core_row` 全是 `differences_outside_the_closed_column=0`（Q3-核取第一刀的行，扩表没有第一刀） | Q3-核 9 趟 0 处不同，读法 9 成立、F5 不触发。第 102、233、265 行批 1 与批 2 红的列不同，都是取样点不敏感（批 2 没取到红的那几段或那一类态），记「不稳定」。s4 那 11 行另外两条为什么看不到在「Q3 只靠一条的 11 行」一节，扩表新出的 12 行在「F4 新出的只靠一条的 12 行」一节 |
| Q4 | 只能进全量：117 行整表两条线道 5 569.6 秒，扩表 164 行另一趟 7 607.9 秒，门槛 60 秒 | `e171-three-oracle-collusion-2026-10-03-s4-f4-rows-timing.out` 的两行 `name=table source=table-driver.out`（`rows=117 table_wall_ns=5569579167726`、`rows=164 table_wall_ns=7607936234181`） | 281 行分两趟跑，两趟挂钟相加约 13 177.5 秒（两个数相加，不是一趟量的）；各行之和 11 114.4 + 15 191.9 秒是单线道的估计（没单独跑单线道）。编不过的 6 行补位另一趟 1 277.9 秒（`…-s4-f4-substitutes-rows-timing.out` 的 `name=table source=substitution-driver.out … rows=6 table_wall_ns=1277937171946`），三趟相加约 14 455.5 秒，也是三个数相加。Q3-核每趟三行约 114–130 秒 |

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

**出处 `.claude/kb/checks-owed.md:425-425`（整段抄，未转述）**

```markdown
| C551 | `build_pool` 的分配器没有根环表 | 连续覆盖写那条用例转过根环之后 I-3.1（已分配统计对得上） 红，用例因此只判 I-1.1（块头自述逻辑地址） 与 I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号）。绕法：转过根环之前先可写挂载一次，转环时用的是挂载重建的分配器——`crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs` 的 `build_a_full_ring_whose_oldest_root_is_the_second_version`（C558（回退目标是环里最旧的根时回退那次发布会写坏它） 那条用例的池）这样搭，之后重发与崩溃重开的镜像上判全部不变量。 | `build_pool` 装上根环表之后，同一条用例判全部不变量。 | 实现缺口。 | 2026-09-25 实二一报告 `research/prompts/m2-keyspace-implementer-report.md` 第 13 条；绕法见 2026-09-26 实三交回（`records/2026-09-24-里程碑二收尾调度.md` 第三节「实三交回」那一行） |
```

**出处 `.claude/kb/invariants.md:292-292`（整段抄，未转述）**

```markdown
| I-9.5 | 删除队列里的号仍在树里 | 两个方向都要成立。① `deleted_inodes` 里的每个 inode 号在同一棵树的 inode 树里存在且记录的 nlink = 0（记录在回收完成前留在 inode 树里，`deleted_inodes` 只存号）；② inode 树里 nlink = 0 的每条记录，号都在同一棵树的 `deleted_inodes` 里（队列是派生态，队列 = inode 树里 nlink = 0 的号集合，D8（核心索引结构） 已定项 16 第 3 条）。**共用判法**（I-4.6（中间态其余不变量成立）、I-4.7（删除中 inode 有账）、I-3.4（可用空间扣待删占用） 都照这一段，不另写）：判走读到的任一版（候选集里每条根，含由记录施加出来的那几版），不只判最新根；`deleted_inodes` 与 inode 树经树表条目偏移 116 的头 ID 配对（头 ID 的取值见 I-9.27（树表条目的头 ID 取值）），配不上的版记判不了；树表、`deleted_inodes` 或 inode 树读不全的版报 `part_of_the_range_not_judged`（射程里判不了，不判红），这是今天除 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 外各条用的报法，是否改报第四种判定「部分没判」随 C701（别的判法静默跳过读不出的部分） 那一问由用户答，答了这几条随它一起改。树表指针全零、树表 0 条的版没有这几棵树，不判；树表单元相同的几版内容相同，只扫一次；没有删除的版本上 I-9.5（删除队列里的号仍在树里） 报成立（两个方向比的是队列与整棵 inode 树，干净的一版也真比过），I-4.7（删除中 inode 有账） 与 I-4.6（中间态其余不变量成立） 没有对象时报不适用并带理由 | 已实现（2026-10-05，池级 checker `walk::check_pool_image`，删除态扫描是 `judge_deletion_state_of_the_versions`；坏镜像登记在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `.chain`，搭建与改法在 `crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs`，逐形的用例在 `crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs`；GPU 判器下标 69，扫描内核判）。两个方向各有坏镜像；C708（树里 nlink = 0 的 inode 不在删除队列里，池级 checker 判不出） 还差「队列里的号在树里 nlink ≠ 0」那一形的坏镜像；`deleted_inodes` 的形态已定在 D8（核心索引结构） 已定项 16；判法与读法的出处 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节 |
```

**出处 `.claude/kb/invariants.md:315-315`（整段抄，未转述）**

```markdown
| I-9.28 | 全部可读根里同一树 ID 的诞生 txg 相同 | 根环里全部读得出且自证合法的根（不按实例表判有效与否，含被崩溃恢复抛弃的旧线上的根），任两条根各自的树表里同一树 ID（条目偏移 0）的条目，诞生 txg 必须相同；只有这棵树第一次出现在树表里时才设这个值（D8（核心索引结构） 已定项 8（树表条目的字段））。判定四种：有一处诞生 txg 对不上 → 不成立；各处都对得上，但有一条读得出、自证过的根树表指针非零而树表读不出、或条目比字段表短 → 部分没判，列出这些根（盘号、实例、txg、为什么没判）；没有一棵树出现在两个不同的树表单元里（几条根共用一个树表单元时拿同一份字节比自己，是恒真判定）→ 不适用；其余 → 成立。树表指针全零的根没有树表，不算没判到。与 I-9.14（树表条目的诞生 txg 跨根不变） 的分工：I-9.14（树表条目的诞生 txg 跨根不变） 只比同一条时间线上的根，这一条不分时间线、比全部可读根。依据：树 ID 被重发（C342（树 ID 水位在根读不出时退回去重发））时，新线上的树与老线上同一个 ID 的树诞生 txg 不同，I-9.14（树表条目的诞生 txg 跨根不变） 的同线收窄看不见这一形。推翻条件：任一合法历史（池级 checker 与层 0 其余判定都不红）上红。 | 已实现（2026-10-04，欠账来源 C342（树 ID 水位在根读不出时退回去重发）；池级 checker `walk::check_pool_image` 的 `judge_tree_identifier_birth_txg_across_every_readable_root`，四种结局；GPU 判器的扫描内核挨着 I-9.14（树表条目的诞生 txg 跨根不变） 判、只置违例位；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，合法历史、部分没判、不适用、对不上照报违例的用例在 `crates/singlefs-harness/tests/pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root.rs`；`crates/mutations.tsv` 里以「第9批」「第9批改钉」打头、点名 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同）或第四种判定的各行，与以「第 9 批 9-9 GPU 判器」打头、点名 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同）的 3 行）。用户 2026-10-04 定新立，同时删 I-9.14（树表条目的诞生 txg 跨根不变） 判别力第 ③ 条与它的正向用例，判决 `research/prompts/m3-sweep-and-tree-id-r1-main-verification.md` 第六节 S2-1 行。被攻过零轮：攻方三段合法历史上 0 红（一次抽样），被切掉旧线的镜像上红。部分没判：用户 2026-10-04 定「树表读不出算部分没判」、加第四种判定；四种结局的细则主 agent 定，零轮，`records/2026-09-30-里程碑三开工第一轮.md` 第 9 批「用户 2026-10-04 弹窗定（9-6 交回的两问）」那一条 |
```
