# m3-close-w5-code-r1 正推腿（Sonnet）报告（2026-10-19）

背景材料 `research/prompts/_m3-close-w5-code-r1-background.md`。立场：逐格核「代码做的是不是条款说的」，每条条款找代码行与测试。被判代码读只读树副本
`/home/fy5090/.local/share/singlefs/work/m3-close-w5-code-r1-tree`（下称「树」）；下面 `crates/…` 的行号都是树里的行号。

## 〇、前置事实与方法

- 树与快照哈希：九份被判文件 `sha256sum -c tree-sha256.txt` 全 OK（原样输出在附录）。树的 `crates/` 与此刻主工作区 `crates/` 只差两份不变量说明
  `03-journal.md`、`README.md`（`diff -rq` 原样输出在附录），所以报告里引的 `crates/…` 行号在树里与仓里是同一行。⚠️ 开头读 `extent_tree.rs`、`unlink.rs`、`transaction.rs` 等九份核心文件是读树里的；后半程为了行号和 cite-check（它按仓根找文件）有几次 `Read` 与 `grep` 的路径写成了主工作区的 `crates/`（`recovery.rs` 3236–3316 一段、`transaction.rs` 12520–12556 一段，加 harness、checker、checker-tier 的 grep），因为两边内容相同（附录 `diff -rq`），读到的东西没有差别，但这不是派发要求的读法，主 agent 核的时候请注意。
- **条款来源有两层，且不同步**：kb 现存文本（`.claude/kb/decisions/08-核心索引结构.md` 已定项 14、16）是 M1–M7 之前的版本，
  `grep -c '规范形' .claude/kb/decisions/08-核心索引结构.md` 得 0；M1–M3 与 β 读法的条款在判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md`
  第三、五节。本报告把「判决」当 W2、W3、W4 的条款，kb 现存文本当它没改的部分，两层对不上的地方在各格「对不上的」里写。
- **没有运行任何测试、没有编译**（定义：不编 Rust、重型不跑）。「测试会不会真红」这一问只能静态答：变异表 95 行（`crates/mutations.tsv:4810`–`4904`）的原文锚点在树里
  各恰好命中一次（`tools/anchor_check.py`，输出见附录）、点名的 74 个测试函数名都在树里找得到。真红与否「复核不了」，要主 agent 在副本上跑变异。
- 背景材料「六处实现员自己定」：1（β 取最大、只在 inode 1 记录不在时判）、2（回退水位）、6（别的文件截断变长拒）在九份核心文件里；
  4（一批 64）在 GPU 恢复内核 `crates/singlefs-checker-tier/src/crash_judge_common.wgsl:259`，不在九份文件里；5（少一个祖先改成少一片叶）对应 GPU 一侧的用例
  `crates/singlefs-checker-tier/tests/crash_judge_gpu_judges_files_written_by_inode_number_like_the_cpu_judge.rs:797`（CPU 单测红的是少一个祖先：`crates/singlefs-checker/src/walk.rs:18423`；
  真镜像里少了祖先，叶从根走不到，造不出来，所以 GPU 那边改成少一片叶，推的）；3（上段节点槽数按上界）我在树里没找到带这个名字的代码或注释，按「没找到对应代码」写，不猜。

## W1　写入口与条款

### 对上的

1. **一个事务状态机、没开第二条发布路径。**
   - 条款 `.claude/rules/fs-design.md@一个事务层，所有结构共用`。
   - 代码：`crates/singlefs-core/src/transaction.rs:8482` `publish_sequential_write_of_an_inode` 对 inode 1 直接转给 `publish_sequential_write`（`:8491`–`:8501`），
     别的号拼计划后调「publish_version(pool, allocator, plan, Some(previous))」（`crates/singlefs-core/src/transaction.rs:8510`）；
     `publish_overwrite_of_an_inode`（`:8431`）走「publish_version_holding_one_data_unit(pool, allocator, plan, Some(previous))」（`crates/singlefs-core/src/transaction.rs:8459`）再进 `publish_version`。
   - 封闭枚举：`DeletionStepOfThePublish`（`crates/singlefs-core/src/transaction.rs:4976`）没有为这两个入口加成员；产品代码 diff 里没有任何提交步骤枚举（`CommitStep` 一类）的改动：
     `grep -n 'CommitStep\|BlockDeviceAction' research/prompts/m3-close-w5-code-r1-snapshot/product-code.diff` 零命中。`UserChange` 加两个成员（`crates/singlefs-core/src/mounted_session.rs:136`、`:143`），
     分发在 `:1760`、`:1773`。
   - 测试：`crates/singlefs-harness/tests/writes_by_inode_number_of_several_files.rs:316`（`writing_one_inode_leaves_the_data_units_and_extent_lower_nodes_of_the_other_files_untouched_on_disk`）、
     `:271`（两个 inode 交替覆盖、跨两次可写挂载读回）；随机历史组 `crates/singlefs-harness/src/history.rs:1554`（`WRITES_BY_INODE_NUMBER`）配模型对拍
     `crates/singlefs-harness/tests/writes_by_inode_number_in_random_histories_judged_by_the_model.rs:105`、`:458`。

2. **上段、下段的写与换下的节点进 defer（同第一个文件）。**
   - 条款 D5 已定项 19：`.claude/kb/decisions/05-快照-空间记账机制.md:422`「elif birth > p: 立即释放 → 进 defer 队列」。
   - 代码：别的文件的单元与下段进「上一版被换下」清单——`previous_roles_replaced_by_this_publish`（`crates/singlefs-core/src/transaction.rs:5935`）里
     `files_other_than_the_first.replaced_previous_data_roles`、`replaced_previous_lower_node_roles`、`previous_roles_without_a_successor` 三段（`:5977`–`:5999`）；
     写一个文件整份换下（`:6504`–`:6522`），删除批只重写被截那片叶与祖先（`:6523`–`:6560`），最后一批整份没有同名后继（`:6561`–`:6568`）。
     生死判定入口 `asks_the_snapshot_lifetime_judgement`（`crates/singlefs-core/src/transaction.rs:5564`）加了两个新角色。
   - 测试：同上 `:316`；删除各批的释放在 `crates/singlefs-harness/tests/unlink_of_files_other_than_the_first.rs:266`。

3. **准入扣的量（同第一个文件）。**
   - 条款 D16 已定项 1「准入」那一行（`.claude/kb/decisions/16-发布语义.md:42`）与 D28 已定项 1：普通分配判空间。
   - 代码：两个新角色归 `OrdinaryAllocation`（`crates/singlefs-core/src/admission.rs:1484` 起，`space_budget_of_role` 在 `:1482`）；数据单元计数改按 `is_a_data_unit()`
     （`crates/singlefs-core/src/admission.rs:943`、`:947`），所以别的文件的数据单元算「成对的槽」，不当提交内生块算；写账归类
     `crates/singlefs-core/src/write_accounting.rs:69`（数据）、`:73`（下段节点算 `ExtentTreeNode`，按树数不按文件数）。
   - 测试：没有专点；`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:272` 与 `crates/singlefs-harness/tests/admission_checkpoint_cost_per_device_paths.rs:443` 只是穷举匹配里多了一个分支。见 W5。

4. **会话入口。**
   - 条款 D8 已定项 16 第 6 条（`.claude/kb/decisions/08-核心索引结构.md:486`）：「会话里每一次用户改动（含 unlink、写、建 inode）先把现行那一版的 `deleted_inodes` 队列走空，再做它自己」。
   - 代码：走空在所有 `UserChange` 之前（「if !current_version_of_trees.queue_of_committed_intents_is_empty() {」，`crates/singlefs-core/src/mounted_session.rs:2033`），
     两个新成员也在 `match change` 的「删完之后只许建 inode」那一支里按已有拒绝处理（`crates/singlefs-core/src/mounted_session.rs:2053`–`:2071`）；
     两个新拒绝不算空间不够（`crates/singlefs-core/src/mounted_session.rs:1657`、`crates/singlefs-core/src/mount.rs:5513`）。
   - 测试：`crates/singlefs-harness/tests/writes_by_inode_number_of_several_files.rs:485`（删别的带数据文件再删第一个文件）；
     `:557`（删完第一个文件之后给之后建的 inode 写数据，拒）。

5. **D16 已定项 5 的切分纪律对别的号同样成立。** 条款「事务切分纪律是一事务一单元」（`.claude/kb/decisions/16-发布语义.md:155`）；
   别的号的内容也经 `file_content_transactions`（`crates/singlefs-core/src/transaction.rs:6959`）里的 `split_sequential_write_into_one_unit_transactions`（`:6961`）切成一单元一事务，
   取号序与 key 序不一致的次数并进同一个计数（`crates/singlefs-core/src/transaction.rs:12517` 一带）。测试：145 个单元的 inode 150 写进去再删
   （`crates/singlefs-harness/tests/unlink_batches_of_an_inode_other_than_the_first_follow_its_own_data_units.rs:283`）。

### 对不上的

- **没有代码与条款字面冲突的一格。** 下面是「条款没写 / 条款写了而测试变异没罩」：
- **规则没说**：D8 已定项 14 extent 那一条（`.claude/kb/decisions/08-核心索引结构.md:419`「上段按 inode 号的位置寻址；下段一个文件一棵」）没写「按 inode 号写一个文件，它的下段整份重写、上一版的每个数据单元与下段节点整份换下」；
  代码这么做（`crates/singlefs-core/src/transaction.rs:6509`–`:6521`），与第一个文件整份重写同形。条款对写没有「只重写变了的叶」这一条，只对删除批写了「只重写被截掉那一片下段叶」（D8-16-2，`.claude/kb/decisions/08-核心索引结构.md:482`）。
  推翻条件：条款或用户补一句「写别的文件也只重写变了的下段叶」，这一格才算冲突。
- **条款写了而没有测试没有变异**：D8 已定项 16 第 4 条「写入口拒 nlink = 0 的 inode」（`.claude/kb/decisions/08-核心索引结构.md:484`）。
  别的号这一支是「return Err(PublishError::WriteToAnInodeBeingDeleted { inode: inode.0 });」（`crates/singlefs-core/src/transaction.rs:8547`），紧跟的「previous.queue_of_committed_intents_is_empty(),」断言（`:8550`）也没有。
  测试只拒第一个文件（`crates/singlefs-harness/tests/write_entries_refuse_an_inode_being_deleted.rs:32`、`:68` 调的是 `publish_overwrite` / `publish_sequential_write`），
  变异表没有一行动 `:8545`–`:8548`（附录原样输出：`WriteToAnInodeBeingDeleted` 在测试里只出现在那一份文件，在变异表里只有 1884 与 4812 两行，前者是第一个文件的，后者把「号不在树里」换成它）。
  推翻条件：拿到一条点名别的号、nlink = 0、直调 `publish_sequential_write_of_an_inode` 的用例，或一行动这一支的变异且点名的测试真红。
- **条款写了而测试没罩**：D8-16-6 的空间拒（会话走空被空间拒推抬 F 再试，`.claude/kb/decisions/08-核心索引结构.md:486`）对别的号的删除批：
  四份多文件用例里 `SpaceAdmissionRefused|NoSpaceAfterRaisingTheFloor` 计数都是 0（附录）。代码里这一路不分号（`is_a_deletion_class_publish`，`crates/singlefs-core/src/transaction.rs:5071`，按步分，不按号），
  所以是「没测」不是「错」。

## W2　上段规范形

条款（判决 M2，`research/prompts/m3-multi-file-unlink-r1-main-verification.md:38`）：「形状只由有条目 inode 集合 S 定：节点 = S 里每个号从它那片叶到根那一串的并；根层级 = 罩得住 max(S) 的最低一层；S 空时是层级 0 第 0 格一个零条目根兼叶」。
同一规则的不变量版是 I-9.40（`crates/singlefs-checker/invariants/09-inode树结构.md:52`）。

### 对上的

1. **写与删都经同一个纯函数 `plan_the_extent_upper_segment`。** 函数本体 `crates/singlefs-core/src/extent_tree.rs:212`；唯一的调用点是
   `extent_tree_plan_with_the_unified_upper_segment`（`crates/singlefs-core/src/transaction.rs:6313`），它的四个调用方：写第一个文件 `plan_the_extent_tree_after_this_publish`（`:6248`，调用在 `:6299`）、
   写别的文件 `plan_the_extent_tree_after_writing_a_file_other_than_the_first`（`:6616`，调用在 `:6623`）、删除批 `crates/singlefs-core/src/unlink.rs:252`（调用在 `:270`）、最后一批 `crates/singlefs-core/src/unlink.rs:285`（调用在 `:308`）。
2. **形状只由 S 定。** 「let root_level = upper_root_level_for(largest_inode_with_an_entry);」（`crates/singlefs-core/src/extent_tree.rs:189`）、
   「.flat_map(|inode| upper_path_of_inode(*inode, root_level))」（`:192`，各号那一串取并），空集分支返回层级 0、位置 (0,0)（`:183`–`:188`）。
   重写判据「if covers_a_changed_inode(position) || !previous_upper_nodes.contains(position) {」（`crates/singlefs-core/src/extent_tree.rs:231`），换下判据
   「!positions_after_this_publish.contains(position) || covers_a_changed_inode(position)」（`:242`）。长高时旧根不罩变了的号就照抄成根之下节点，降层时降后层级的第 0 个节点照抄成根，
   角色名按上一版与这一版各自的根层级取（`crates/singlefs-core/src/transaction.rs:11389`–`:11390` 一带）。
3. **同一个 S 两条历史同一个形状**：`crates/singlefs-harness/tests/unlink_of_files_other_than_the_first.rs:543`（直接写成 {1,2} 与先写 150 再删 150）。
   独立判器各写各的：池级 checker 自己的规范形 `crates/singlefs-checker/src/walk.rs:10993`、判一版的 `:11033`；理想模型自己的计划
   `crates/singlefs-harness/src/model.rs:11328`（`the_model_upper_plan_grows_lowers_and_empties_the_segment_as_the_forward_model_does`）。
4. **层级升降、叶变空、三种终态各有一条用例**（绝对值，不拿实现的输出当期望）：
   - 长高：`crates/singlefs-harness/tests/writes_by_inode_number_of_several_files.rs:396`；
   - 降层（只剩第一个文件）：`crates/singlefs-harness/tests/unlink_of_files_other_than_the_first.rs:426`，断言 `vec![upper(0, 0)]` 且第 0 片叶指针不变；
   - 只剩别的一个文件（第 0 片叶摘掉、根留在层级 1）：`crates/singlefs-harness/tests/unlink_of_files_other_than_the_first.rs:460`，断言 `vec![upper(0, 3), upper(1, 0)]`、500 那片叶指针不变、这一版归 `PoolVersion::WithFile`（M3）；
   - 全删空、两种删序同一个零条目根：`crates/singlefs-harness/tests/unlink_of_files_other_than_the_first.rs:502`（断言 `vec![upper(0, 0)]`）；
   - 三终态的纯函数版：`crates/singlefs-core/src/extent_tree.rs:1567`。
   - 变异行：`crates/mutations.tsv:4841`、`:4842`、`:4843`、`:4850`、`:4851`、`:4856`、`:4857`、`:4904`（根层级取错、只重写叶不重写祖先、最后一批不把删的号拿出 S、不把它当条目变了的号）。
5. **一批 64 与 64 以上的上段条目**：CPU 恢复没有这个批界，`for (inode, extents) in files {`（`crates/singlefs-core/src/recovery.rs:5218`）逐条收；批只在 GPU 恢复内核，
   `const MAX_UPPER_ENTRIES: u32 = 64u;`（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:259`），那一段表在 `:260` 起（注释写「这一段装得下 256 条」）。
   跨批用例 `crates/singlefs-checker-tier/tests/crash_judge_gpu_judges_files_written_by_inode_number_like_the_cpu_judge.rs:703`，变异 `crates/mutations.tsv:4876`、`:4877`。
   核心里准入按角色清单逐个数槽（`crates/singlefs-core/src/admission.rs:938` 一带，角色清单来自计划），没有「按 64 估」的地方，所以一批 64 不会让准入低估。
6. **大号、稀疏号**：层级跨度是饱和乘法（`crates/singlefs-core/src/extent_tree.rs:84`），`upper_root_level_for` 只对 `u64::MAX` panic（`:103`）；条款声明号上界 2^63
   （`.claude/kb/decisions/08-核心索引结构.md:173`「声明生命周期创建总数上界 2^63」），所以不会走到。稀疏形态由 150（层级 1，两片叶）与 500（第 4 片叶）两个号覆盖。

### 对不上的

- **kb 现存文本没有规范形。** 判决 M2 的落点是「写进 D8 已定项 14 extent 那一条与 D8 已定项 16 新增第 8 条」（`research/prompts/m3-multi-file-unlink-r1-main-verification.md:38`）。
  kb 里 D8 已定项 16 只有第 1–7 条（`.claude/kb/decisions/08-核心索引结构.md:481`–`:487`，`grep -o` 原样输出在附录），`grep -c '规范形' .claude/kb/decisions/08-核心索引结构.md` 为 0。
  已定项 14 extent 那一条仍是：
  「标签 0 全零，只有读者认、写者不写」（`.claude/kb/decisions/08-核心索引结构.md:431`）与「上段按 inode 号的位置寻址；下段一个文件一棵」（`.claude/kb/decisions/08-核心索引结构.md:419`），
  没有「形状只由 S 定」那一句。判决「回看决策」表（`research/prompts/m3-multi-file-unlink-r1-main-verification.md:59`）写「2026-10-19 改了」，kb 文件里没有。
  这是文字不同步，不是代码错；代码按判决写，kb-scribe 那一步还没落。推翻条件：`git diff` 显示 kb 那两处已在别处落了。
- **I-9.40 的 GPU 一句过期。** 不变量说明写「GPU 判器下标 102 这一批不进内核、对拍不比」（`crates/singlefs-checker/invariants/09-inode树结构.md:52`），而代码里表是空的：
  「pub const POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS: &[usize] = &[];」（`crates/singlefs-checker-tier/src/crash_judge_conclusions.rs:438`），
  扫描内核照判置位：「judge(INVARIANT_I_9_40, holds);」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3858`），变异 `crates/mutations.tsv:4878`–`:4884` 也按「内核判」写。
  判决 M2 要的正是后者（两路逐位相同）；说明里那一句该改。
- **S 的口径：核心与 checker 不同（只在标签 0 上）。** 核心的 S 只数有数据单元的号（`crates/singlefs-core/src/transaction.rs:3494`，第一个文件「(!self.data_pointers.is_empty()).then_some(FIRST_INODE_NUMBER)」），
  checker 的 S 数「任一标签」：`ExtentUpperLeafEntryView::NoDataUnit { inode }` 也进集合（`crates/singlefs-checker/src/walk.rs:10963`），I-9.40 同写「任一标签都算」。
  恢复收下标签 0 的第一个文件（`crates/singlefs-core/src/recovery.rs:5262`、注释「今天的写路径写不出这一形」在 `:5264`）。
  所以如果有一份盘上带标签 0 条目的镜像（只能是别处造的），核心按它不在 S 里重算形状，checker 按它在 S 里判，两边会对同一版给出不同的「规范」。今天写路径写不出，所以只是边角；
  推翻条件：任何一条核心写路径能写出标签 0（现在没有：写者只在有单元时写条目，`crates/singlefs-core/src/transaction.rs:11396`–`:11405`）。
- **截断不经统一函数。** `plan_the_extent_tree_after_a_size_change`（`crates/singlefs-core/src/truncate.rs:228`）自己算上段：照抄上一版的节点、只重写这个文件那一串（`:275` 起）。
  判决 M2 只点名「写第一个文件、写别的文件、删除」三处共用，没点截断；因为断言「截断留至少一个单元、变长不加单元」（`crates/singlefs-core/src/truncate.rs:242`）S 不变，形状自然不变，所以一致。
  规则没说：若以后截断能把 S 里的号拿掉（截到 0 个单元），这里要接进统一函数。

## W3　删除各步

条款：D8 已定项 16 第 1、2、5、6、7 条（`.claude/kb/decisions/08-核心索引结构.md:481`、`:482`、`:485`、`:486`、`:487`）加判决 M1、M3（`research/prompts/m3-multi-file-unlink-r1-main-verification.md:37`、`:39`）。

### 对上的

1. **第 1 条意图对任一号。** 「tombstone: TombstoneRecord::of_the_whole_object(inode, record.object_birth, txg),」（`crates/singlefs-core/src/transaction.rs:9430`），
   记录写成 nlink = 0、size 不变（`:9420`–`:9429`），`deleted_inodes` 加号（`:5086` 起 `deleted_inodes_after_this_publish`），不立 logged_ops（`LoggedOperationsStepOfThePublish::CarryTheLoggedOperations`，`:9435`）。
   死亡 txg 取意图那次发布的 txg 与 D18 已定项 24 第 2 条一致：`.claude/kb/decisions/18-块里携带什么信息.md:577`「死亡 txg = 意图那次发布的 checkpoint_txg」。
2. **第 2 条批大小。** 「OneExtentLowerLeaf => EXTENT_TREE_LOWER_LEAF_DATA_UNITS,」（`crates/singlefs-core/src/unlink.rs:103`，产品 144）；
   「if data_units_remaining <= data_units_per_batch {」（`crates/singlefs-core/src/unlink.rs:139`）剩下的不多于一批就走最后一批；剩余单元数取被删号自己的：
   「let data_units_remaining = if inode.0 == FIRST_INODE_NUMBER {」（`crates/singlefs-core/src/transaction.rs:9504`），别的号取 `files_other_than_the_first` 那一项、没有的取 0（`:9507`–`:9510`）。
   测试把期望写成绝对值：`crates/singlefs-harness/tests/unlink_batches_of_an_inode_other_than_the_first_follow_its_own_data_units.rs:232`（批 1、三个单元 → `[2,1,0]`）、
   `:249`（批 2、七个 → `[5,3,1,0]`）、`:266`（144 个、产品批 → `[0]`）、`:283`（145 个 → `[1,0]`）；变异 `crates/mutations.tsv:4845`、`:4868`–`:4871`。
3. **第 2 条 size。** 批的记录 size = 剩下单元数 × 净荷：「size_after_the_batch_in_bytes: data_units_kept」（`crates/singlefs-core/src/unlink.rs:147`，乘 32634 在 `:148`），
   写进记录「size: size_after_the_batch_in_bytes,」（`crates/singlefs-core/src/transaction.rs:9521`）；写入时间照抄（`:9525`）、改动计数取本次 txg（`:9500` 一带）、nlink 仍 0。
   没有一条用例直接断言别的号批后记录的 size 数；间接判有两道：理想模型对拍要求「各单元接起来恰好是 size」
   （`crates/singlefs-harness/src/model_comparison.rs:272`），挂载恢复核单元数对 size（`crates/singlefs-core/src/recovery.rs:3327`，函数 `:5964`）。
4. **第 2 条最后一批：删记录、删 `deleted_inodes` 那条 key、释放剩下全部单元，同一次发布。** 记录删除在 `ordinary_allocation_part` 的 FinalBatch 支
   「remove_record_from_leaf_containers(&containers_before, inode)?」（`crates/singlefs-core/src/transaction.rs:7088`），队列 key 删除
   「DeletionStepOfThePublish::FinalBatch { inode } => keys.retain(|key| *key != inode),」（`crates/singlefs-core/src/transaction.rs:5097`），别的号的数据单元与下段整份换下
   「ChangeOfAFileOtherThanTheFirst::FinalDeletionBatch { inode } => {」（`crates/singlefs-core/src/transaction.rs:6561`）；三者在同一个 `publish_deletion_step`（`crates/singlefs-core/src/transaction.rs:9872`）里。
   测试：`crates/singlefs-harness/tests/unlink_of_files_other_than_the_first.rs:266`（三单元的 inode 2：两个版本——意图与最后一批；单元、下段、记录、队列、上段条目一起没）；挂载续做 `:609`。
   变异 `crates/mutations.tsv:4847`、`:4848`、`:4858`。
5. **事务号。** 「let transaction_of_this_deletion = previous.highest_transaction_number_in_this_instance + 1;」（`crates/singlefs-core/src/transaction.rs:9897`），意图与每一批各占一个，与号无关。
6. **第 5 条：删第一个文件而别的文件还在，这一版取 `WithFile`，不加成员。** `is_a_version_with_trees_and_no_file`（`crates/singlefs-core/src/transaction.rs:3468`）要求 inode 树零叶容器、没有任何单元（含别的文件）、`deleted_inodes` 根置零；
   `PoolVersion::of_a_version_of_trees` 按它分（`crates/singlefs-core/src/transaction.rs:2298`），恢复用同一个函数（`crates/singlefs-core/src/recovery.rs:2929`）。
   测试 `crates/singlefs-harness/tests/unlink_of_files_other_than_the_first.rs:460`（断言 `PoolVersion::WithFile(_)` 且 `inode_record.is_none()`）。
7. **两种删序。** 判决 M3：「不要求相同；上段位置集合在规范形下相同」（`research/prompts/m3-multi-file-unlink-r1-main-verification.md:29`，同一格前半句写不同删序终态逐字节与逻辑内容都不同）。
   测试只比上段位置集合：`crates/singlefs-harness/tests/unlink_of_files_other_than_the_first.rs:502`；没有任何用例断言两种删序的盘逐字节相同，和判决一致。
8. **第 6 条会话与续做共用同一个走空**：`walk_the_deletion_queue_until_it_is_empty` 队首号逐步 `publish_unlink_step`（`crates/singlefs-core/src/transaction.rs:9823`、`:9840`），
   `next_unlink_step_after`（`:9484`）只读现行那一版，不分号，所以 ①「计划相等」对任一号成立。崩溃流 `crates/singlefs-checker-tier/tests/crash_enumeration_unlink_of_two_files_holding_data_streams.rs:234`
   把两种删序、每个崩溃状态上可写挂载续做、同一份崩溃镜像两次续做逐字节相同（①b）都罩了（重型，我没跑）。

### 对不上的

- **没有代码与条款冲突的一格。**
- **kb 现存的第 5、7 条和射程没跟上判决（文字不同步，同 W2 第一条）。** 并排：
  - 判决 M3：「第 7 条末尾补：不同操作序列的终态不要求逻辑内容相同，要求上段形状只由 S 定、每个崩溃态 checker 全绿」（`research/prompts/m3-multi-file-unlink-r1-main-verification.md:39`）；
    kb 第 7 条（`.claude/kb/decisions/08-核心索引结构.md:487`）没有这一句。
  - 判决 M3：「D8 已定项 16 第 5 条末尾补一句（inode 树还有任一记录或 extent 树还有条目的一版是带文件的一版，不管 inode 1 在不在；第三个成员只指文件集合真的空）」（同一行）；
    kb 第 5 条（`.claude/kb/decisions/08-核心索引结构.md:485`）停在「删完之后只能做 `publish_new_inodes`（size 0）、写行与空发布（K-U5 X1）」，没有补句。
  - 判决：「第 2 条对任一号（M1），第 5、7 条补句与新增第 8 条（M2、M3），射程句改」（`research/prompts/m3-multi-file-unlink-r1-main-verification.md:60`，改成「按 inode 号删任一 inode、无目录」，K-U5 那句随 β 改写）；kb 射程现在是：
    「第一版只做单文件 unlink、按 inode 号、无目录、不带 truncate（用户 2026-09-28 定）；删完之后不再建带数据的文件（K-U5，记欠账）」（`.claude/kb/decisions/08-核心索引结构.md:489`）。
    代码做的是后者之外的事：删别的带数据文件（M1）、删完之后对删之前就有的 inode 照写（β）。代码按判决，kb 没改。
- **规则没说**：第 2 条没写「别的号的批」重写哪几个上段节点；代码取 N 那一串（`crates/singlefs-core/src/unlink.rs:270`–`:275`，`inodes_whose_entry_changed = {inode}`），
  判决 M2 的「条目变了的号那一串重写」补上了这一句，kb 没有。

## W4　恢复、续做、墓碑与 β

条款：D8 已定项 16 第 1、4、6 条（`.claude/kb/decisions/08-核心索引结构.md:481`、`:484`、`:486`）、D18 已定项 24 第 2、4 条（`.claude/kb/decisions/18-块里携带什么信息.md:577`、`:579`）、
D8 已定项 6 的水位句（`.claude/kb/decisions/08-核心索引结构.md:171`）、K-U5 β（判决第五节，`research/prompts/m3-multi-file-unlink-r1-main-verification.md:53`）。

### 对上的

1. **恢复收多文件，删除中间态不会被「至少一个单元、单元连号、尾部没有洞」误拒。**
   - 判据在 `crates/singlefs-core/src/recovery.rs:5228`–`:5235`（「if data_pointers_of_the_file.is_empty()」起，三条或）。
   - 逐态推：意图那次记录 nlink = 0、size 与单元都没动；批之后别的号留下前 K 个单元（≥ 1，`unlink.rs:139` 保证 `Batch` 只在剩余多于一批时出现），单元号从 0 连号，size = 留下数 × 净荷
     （`crates/singlefs-core/src/unlink.rs:147`），上段条目恒是标签 1 或 2（「ExtentUpperLeafTarget::LowerSegmentRoot(*lower_root_pointer)」，`crates/singlefs-core/src/transaction.rs:11368`；
     内联是 `:11370`），没有标签 3、4，所以尾部不会有洞；最后一批是一次发布里记录与条目一起没，没有「有单元没记录」的中间态。恢复里「extent 树里有它的数据单元、inode 树里没有它的记录」才拒
     （`crates/singlefs-core/src/recovery.rs:3320` 起的 `ok_or(RecoveryFailure::UnitMalformed {…})`），那一形今天写不出。
   - 测试：`crates/singlefs-harness/tests/unlink_of_files_other_than_the_first.rs:609`（只发 inode 2 的意图就关镜像，重开可写挂载把删除走完）；GPU 恢复内核对半删中的别的文件逐位对拍
     `crates/singlefs-checker-tier/tests/crash_judge_gpu_judges_files_written_by_inode_number_like_the_cpu_judge.rs:679`；每个崩溃状态上可写挂载一次的崩溃流
     `crates/singlefs-checker-tier/tests/crash_enumeration_unlink_of_two_files_holding_data_streams.rs:234`（重型，没跑）。
2. **挂载续做对任一号走得完。** 可写挂载在写行与暖机之后调同一个走空：「let walk = walk_the_deletion_queue_until_it_is_empty(」（`crates/singlefs-core/src/mount.rs:6801`），
   取号之前预演整个续做用同一份 `next_unlink_step_after`（`mount.rs:5445` 调 `rehearse_every_step_of_the_deletion_queue_walk`，实现在 `crates/singlefs-core/src/transaction.rs:9762`）。队列至多一条 key
   （断言 `crates/singlefs-core/src/transaction.rs:9405`），所以每次只有一个号在走。删除类发布按步分准入，不按号分（`transaction.rs:5071`）。
3. **墓碑每个对象一条、出生代写对象自己的。** 「tombstone: TombstoneRecord::of_the_whole_object(inode, record.object_birth, txg),」（`crates/singlefs-core/src/transaction.rs:9430`），
   条款：「死亡 txg = 意图那次发布的 checkpoint_txg」（`.claude/kb/decisions/18-块里携带什么信息.md:577`）；回收：「第一版不回收墓碑」（`.claude/kb/decisions/18-块里携带什么信息.md:579`）。
   没有一条用例直接数别的号的墓碑多重集（`unlink_of_files_other_than_the_first.rs` 里 `tombstone` 只在第一个文件的死亡 txg 上出现，`:587`–`:596`）；
   间接判是 I-9.18：「都有一条在谱系上的墓碑记录 (N, N 的 inode 记录偏移 8 的对象出生代, [0, 2⁶⁴ − 1])」（`crates/singlefs-checker/invariants/09-inode树结构.md:30`），
   多文件用例的 `assert_no_violation`（`crates/singlefs-harness/tests/unlink_of_files_other_than_the_first.rs:174`）每一版都跑池级 checker。
4. **β：只拒对象出生代晚于 inode 1 墓碑死亡 txg 的写。**
   - 代码两段：函数 `death_txg_of_the_deleted_first_file`（`crates/singlefs-core/src/transaction.rs:3511`，「if self.inode_record.is_some() {」在 `:3512` 先放过 inode 1 记录还在的，
     再取 inode 1 的墓碑里最大的死亡 txg，`:3515`–`:3520`）；写入口比较「if record.object_birth > death_txg_of_the_first_file {」（`crates/singlefs-core/src/transaction.rs:8535`），
     在「nlink = 0」那一拒（`:8545`）与队列断言（`:8549`）之前。
   - 删之前就有的照写：`crates/singlefs-harness/tests/unlink_of_files_other_than_the_first.rs:579`；删完之后才建的拒、盘上逐字节不变：
     `crates/singlefs-harness/tests/writes_by_inode_number_of_several_files.rs:557`；模型对拍 `crates/singlefs-harness/tests/writes_by_inode_number_in_random_histories_judged_by_the_model.rs:157`，
     模型自己的比较在 `crates/singlefs-harness/src/model.rs:5665`（「.is_some_and(|death_txg| written_inode.object_birth > death_txg);」）。变异 `crates/mutations.tsv:4814`、`:4815`、`:4852`、`:4862`、`:4864`。
5. **回退之后水位不退回去（树 ID、inode 号）。** 条款：「水位取 max(现行那一版内存里的, 环里读得出、按实例表没被崩溃恢复抛弃的根的)」（`.claude/kb/decisions/08-核心索引结构.md:171`）。
   代码：`inode_number_watermark_of_the_rollback`（`crates/singlefs-core/src/mount.rs:8683`，现行与环里没被抛弃的取大）、`tree_identifier_watermark_of_the_rollback`（`:8703`，现行与环里全部根取大）。
   背景材料第二节写「回退之后的水位取现行那一版的」，字面比条款窄：代码取的是 max（现行，环里没被抛弃的），与条款同。这一批对回退只多了一处：
   「files_other_than_the_first: target_version.files_other_than_the_first.clone(),」（`crates/singlefs-core/src/mount.rs:8787`，别的文件的单元与下段取回退目标那一版的，与第一个文件同）。
   水位的单元测试 `mount.rs:10383`、`:10398`、`:10413`（inode 号）与 `:10481`（树 ID）在这一批之前就在，没动。

### 对不上的

- **β 两处的覆盖（条款有、测试变异没罩）。** 「取死亡 txg 最大」是 `.max()`（`crates/singlefs-core/src/transaction.rs:3520`），「只在 inode 1 的记录不在时判」是 `:3512` 那个 `is_some()` 提前返回。
  变异表里没有一行的原文或替换文含 `death_txg_of_the_deleted_first_file`、`tombstone.death_txg` 之于核心的（命令与原样输出在附录）；单条用例只有一次删除一条墓碑（`unlink_of_files_other_than_the_first.rs:579` 断言 `Some(death_txg_of_the_first_file)`），
  没有「删第一个文件 → 回退到删之前 → 再删」两条墓碑的用例。随机历史组（`crates/singlefs-harness/src/history.rs:1554`）同时抽 unlink 与回退（`:1576`、`:1589`），理论上走得到，
  但变异表不动 `.max()`，没法说这条路会红。推翻条件：一行把 `.max()` 换成 `.min()` 或 `.next()` 的变异，点名的测试在副本上真红。
- **I-1.13 说明里 GPU 一句过期。** 说明写「GPU 判器的覆盖表收了这一条（下标 92）、内核不置位」与「GPU 扫描重建内核是欠账」（`crates/singlefs-checker/invariants/01-块自描述.md:18`），
  代码里表是空的（`crates/singlefs-checker-tier/src/crash_judge_conclusions.rs:438`），扫描重建有自己的内核常量
  （`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:345` 的 `KERNEL_INVARIANT_INDEX_OF_THE_SCAN_REBUILD_JUDGE`），变异 `crates/mutations.tsv:4885`–`:4888` 也按「内核判」写。
  同 W2 的 I-9.40：说明落后于代码，不是代码错。
- **`assert_no_violation` 不要求其余不变量「判过」。** 它只拒 `Violated`，`NotApplicable`、`PartiallyJudged` 一律放过（`crates/singlefs-harness/tests/unlink_of_files_other_than_the_first.rs:174`–`:187`），
  只对 I-9.40 另断言 `Holds`（`:190`–`:198`）。所以这批多文件用例对 I-4.7、I-9.5、I-9.18、I-1.13 的「绿」可能是「没判」。M5 那一条另有专门用例
  （`crates/singlefs-harness/tests/scan_rebuild_agrees_with_the_walked_index.rs:321`、`:335`），I-4.7、I-9.18 没有。推翻条件：在这些用例的某一版上打出这几条的判定，全是 `Holds`。
- **规则没说**：β 在管理员回退之后怎么读（墓碑随现行那一版、不取回退目标的，D18 已定项 24 射程只写「第一版单头、无快照、无清扫」）。代码与模型都取现行那一版的墓碑
  （`crates/singlefs-harness/src/model.rs:3754` 一带注释「回退那次的 `previous` 就是现行那一版」），与 D8 已定项 6 里 `deleted_inodes` 取 R_old、墓碑树随现行的分法同向；条款没有 β 这一问的字。

## W5　测试与变异

核的是「测试测条款还是测实现」「每处改法有没有变异行」「变异点名的测试会不会真红」。第三问我没跑，只做了两样静态核（见〇）：95 行变异锚点各恰好命中一次，74 个点名测试都在树里。真红「复核不了」。

### 对上的

1. **期望值多半写成条款推出的绝对值，不是拿实现的输出当期望。**
   - 删除各步的单元数序列写死：`[2,1,0]`、`[5,3,1,0]`、`[0]`、`[1,0]`（`crates/singlefs-harness/tests/unlink_batches_of_an_inode_other_than_the_first_follow_its_own_data_units.rs:232`、`:249`、`:266`、`:283`），
     再与发布路径逐步比；四个终态的上段位置写死（W2 第 4 点）；期望出自正推腿的独立模型（用例注释写「`run.out` 第 12、13、15–17 行」，`crates/singlefs-harness/tests/unlink_of_files_other_than_the_first.rs:423`、`:457`、`:498`）。
   - 理想模型是另写的一份：规范形计划 `crates/singlefs-harness/src/model.rs:11328`、β 比较 `model.rs:5665`；池级 checker 的规范形 `crates/singlefs-checker/src/walk.rs:10993` 与实现各写各的。
   - 变异表里 M4 一族直接动模型（`crates/mutations.tsv:4859`–`:4867`、`:4872`），说明对拍的另一侧真被当成会红的一边守着。
2. **变异覆盖面（按改法）。** 核心里的新逻辑有行的：按号写不重写第一个文件（`:4811`）、上段取并（`:4813`）、β 比较与错误成员（`:4814`、`:4815`、`:4852`）、
   根层级与重写判据（`:4841`、`:4842`、`:4851`）、最后一批拿号出 S 与标「条目变了」（`:4843`、`:4850`、`:4856`、`:4857`、`:4904`）、批的剩余单元数取自己的（`:4845`、`:4846`、`:4868`–`:4871`）、
   摘单元与下段与上段条目（`:4847`–`:4849`）、挂载续做（`:4858`）、重建漏别的文件（`:4810`）。检查与 GPU 一侧各有行（`:4853`–`:4855`、`:4873`–`:4888`）。
3. **B 树叶容器的合并 / 借**在核心里有纯函数单测覆盖中间位置（`crates/singlefs-core/src/inode_tree.rs:1221`），500 号池的两条删序用例走到「三片叶容器」的头与尾
   （`crates/singlefs-harness/tests/unlink_of_files_other_than_the_first.rs:245` 的 `pool_with_the_first_file_and_inode_five_hundred` 建 499 个空 inode）。中间一片的端到端删除没有用例。

### 对不上的（条款有，测试与变异都没罩到，或只罩一半）

下面每条都给「推翻条件」，命令与原样输出在附录。

- **A　别的号的「写入口拒 nlink = 0」。** 条款 `.claude/kb/decisions/08-核心索引结构.md:484`；代码 `crates/singlefs-core/src/transaction.rs:8545`–`:8553`；无测试、无变异（W1）。
  建议的一行变异：`:8545` 的 `let is_being_deleted = record.nlink == NLINK_OF_AN_INODE_BEING_DELETED;` 换成 `let is_being_deleted = false;`。
- **B　β 的 `.max()` 与 `is_some()` 提前返回。** `crates/singlefs-core/src/transaction.rs:3520`、`:3512`；无变异、无两条墓碑的用例（W4）。建议：`.max()` → `.min()`；`if self.inode_record.is_some() {` → `if false {`。
- **C　快照 × 别的文件。** 条款 D5 已定项 19（`.claude/kb/decisions/05-快照-空间记账机制.md:422`）：「elif birth > p: 立即释放 → 进 defer 队列」；
  判决把它记成「不影响」（`research/prompts/m3-multi-file-unlink-r1-main-verification.md:64`：「`DataOfAFileOtherThanTheFirst` 已在问生死那一支」）。
  代码里就是 `asks_the_snapshot_lifetime_judgement` 多了两个成员（`crates/singlefs-core/src/transaction.rs:5564`）。四份多文件用例里 `CreateSnapshot|DestroySnapshot|CompactWhileMounted` 计数全 0（附录），
  按号写的随机历史组「快照、截断、整理不抽」（`crates/singlefs-harness/src/history.rs:1552`），变异表没有行点名这个分支。所以「快照下覆盖写 / 删除别的文件，被钉住的单元进 deadlist 而不是立即释放」没有任何一条会红的东西。
  推翻条件：任一用例在有活快照时覆盖写或删除一个第一个文件之外的文件，并断言 deadlist 条目与立即释放的分界。
- **D　整理（搬迁）× 别的文件。** `take_the_mapping_positions_over_the_location_hints`（`crates/singlefs-core/src/transaction.rs:3258`）里新加的一段把别的文件的数据指针与下段指针换成映射位置；
  同一个组不抽整理（`history.rs:1552`），多文件用例不调整理（附录计数），变异表无行。条款：C417 第一版政策「随重写刷新」（该函数的文档注释），无单独的 D8 条文。
- **E　空间拒推抬 F × 别的号的删除批 / 写。** D8 已定项 16 第 6 条（`.claude/kb/decisions/08-核心索引结构.md:486`）；多文件用例里 `SpaceAdmissionRefused|NoSpaceAfterRaisingTheFloor` 计数全 0（附录）。
- **F　「新加的穷举匹配分支」没有变异行：** `crates/singlefs-core/src/write_accounting.rs:69`、`:73`，`crates/singlefs-core/src/admission.rs:943`、`:947`（`is_a_data_unit`）、`:1484`，
  `crates/singlefs-core/src/mount.rs:8282`、`:8324`、`:8787`（回退取目标那一版的别的文件），`crates/singlefs-core/src/mounted_session.rs:1760`、`:1773`（分发）、`:2065`。变异表按文件数这些新分支的行是 0，
  `mounted_session.rs` 的 1 行（`crates/mutations.tsv:2499`）只是原文锚点恰好包着 `match change`，改的是 unlink 的时间。
  其中 `is_a_data_unit` 值得点出：把 `:943` 换回 `matches!(role, TransactionUnit::Data(_))`，别的文件的数据单元就按提交内生块算槽而不是按成对的槽对算（`crates/singlefs-core/src/admission.rs:985` 的「槽对」判），
  准入的数低估 / 多估谁红没有用例。
- **G　上段「换下」的那一半。** `plan_the_extent_upper_segment` 的换下判据（「!positions_after_this_publish.contains(position) || covers_a_changed_inode(position)」，`crates/singlefs-core/src/extent_tree.rs:242`）
  没有变异行（`:4842` 只改重写判据 `:231`）。其中任一半去掉，被换下的上段节点漏进释放清单或多释放，只有释放判定 + 分配记录一致性（I-3.1 之类）才抓。
- **H　`assert_no_violation` 的「绿」可能是「没判」**（W4 最后一条）。
- **I　随机历史对按号写的取样：** 快照、截断、整理不抽（`crates/singlefs-harness/src/history.rs:1552`），删除要求 inode 树只有一片叶容器
  （`history.rs:1549`），写入的内容长度分类受 `WriteOfAnInodeChoice` 限制；这些是取样点的边，不是错，但 C、D 两格的覆盖全靠这一条限制之外的确定性用例，而那样的用例没有。

## 要主 agent 跑的命令（可选，复核不了的那几样）

我没有跑任何测试与变异。W5 的三样建议变异（原文 → 替换文，均在 `crates/singlefs-core/src/` 的树副本上加，不进主工作区）要主 agent 在副本里加行、跑 `--item` 看点名的测试红不红；
命令形态照 `.claude/rules/verification.md`「两档：轻型与重型」，轻型那条由主 agent 起：

- A：`transaction.rs` 的 `let is_being_deleted = record.nlink == NLINK_OF_AN_INODE_BEING_DELETED;` → `let is_being_deleted = false;`，找一个直调 `publish_sequential_write_of_an_inode` 且点名 nlink = 0 的别的号的用例，没有就是缺口。
- B：`transaction.rs` 的 `.max()`（`death_txg_of_the_deleted_first_file` 末行）→ `.min()`；`if self.inode_record.is_some() {` → `if false {`。
- G：`extent_tree.rs` 的 `!positions_after_this_publish.contains(position) || covers_a_changed_inode(position)` 去掉后半 / 前半各一行。
- 静态核复跑：`python3 -I /home/fy5090/.local/share/singlefs/work/m3-close-w5-code-r1-sonnet/tools/anchor_check.py /home/fy5090/.local/share/singlefs/work/m3-close-w5-code-r1-tree 4810 4904`，期望末行 `rows_checked 95 bad 0`；
  `/home/fy5090/.local/share/singlefs/work/m3-close-w5-code-r1-sonnet/tools/anchor_check.py` 的 sha256 是 `1450aab32dd2b642616a3003409e604d597965289ac4216abf237bad0161a519`。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| W1 写入口与条款 | 一致（代码对条款）；覆盖缺口 A、E、F | 两个新入口走同一个 `publish_version`，没有第二条发布路径；换下进 defer、准入、会话入口与第一个文件同一套；别的号的 nlink = 0 拒绝、空间拒推抬 F、新增分支没有测试或变异 |
| W2 上段规范形 | 一致（代码对判决 M2）；冲突（文字）：kb 现存 D8 已定项 14/16 没有规范形、I-9.40 说明的 GPU 一句与代码相反；边角：标签 0 的 S 口径核心与 checker 不同 | 写、删都只经 `plan_the_extent_upper_segment`；终态三形、升降、叶变空各有绝对值用例；一批 64 只在 GPU 读侧，核心准入按角色数槽 |
| W3 删除各步 | 一致；文字不同步（kb 第 5、7 条与射程） | 批大小、size、最后一批同一次发布、版本成员 `WithFile`、两种删序只比形状，都与判决一致；kb 没补句 |
| W4 恢复、续做、墓碑与 β | 一致；覆盖缺口 B、H；文字过期（I-1.13 说明的 GPU 一句） | 删除中间态不会被「至少一个单元、连号、无洞」误拒；续做对任一号共用同一个走空；墓碑出生代写对象自己的；β 与回退水位同条款，但 β 的 `.max()` 与提前返回没有变异也没有两条墓碑的用例 |
| W5 测试与变异 | 期望多为绝对值、变异锚点 95/95 命中、点名测试 74/74 存在；缺口 A–I | 快照、整理、空间拒、别的号 nlink = 0 拒、β 两处、上段换下那一半没有会红的东西；真红与否复核不了 |

## 没做什么

- 没跑测试、没编译、没跑变异、没跑任何门禁阶段（定义与派发：重型不跑，不编 Rust）。W5 所有「会不会红」只做静态判。
- 没读攻方腿（`research/prompts/m3-close-w5-code-r1-opus-output.md` 与它的草稿目录）的任何东西。
- 读树还是读仓：见〇的 ⚠️，后半程有几处读的是主工作区 `crates/`，内容与树相同（附录 `diff -rq`，只差两份与本轮无关的不变量说明）。`.claude/kb/`、`crates/singlefs-checker/invariants/`、`research/prompts/` 下的条款文件是现读仓里的。
- 没判「哪一格会错」（攻方的立场）；没替主 agent 采纳或出判决；没有把「建议的变异」加进变异表。
- 没核：GPU 内核（`crash_judge_*.wgsl`）的逻辑、背景材料里「六处实现员自己定」的第 3、5 两处（树里没找到对应名字的代码）；崩溃流与随机历史的真跑结果（重型，没跑）。
- 没核旧数据：判决里的 `run.out` 行号（正推腿的旧模型输出），只信用例注释里引的那几行是写死的期望，没有重跑那份模型。

## 附录　原样输出（这一次真跑出来的）

```
$ grep -rno "WriteToAnInodeBeingDeleted" crates/singlefs-harness/tests crates/singlefs-checker-tier/tests --include=*.rs
crates/singlefs-harness/tests/write_entries_refuse_an_inode_being_deleted.rs:2:WriteToAnInodeBeingDeleted
crates/singlefs-harness/tests/write_entries_refuse_an_inode_being_deleted.rs:49:WriteToAnInodeBeingDeleted
crates/singlefs-harness/tests/write_entries_refuse_an_inode_being_deleted.rs:94:WriteToAnInodeBeingDeleted
$ grep -no "WriteToAnInodeBeingDeleted\|is_being_deleted" crates/mutations.tsv
1884:WriteToAnInodeBeingDeleted
4812:WriteToAnInodeBeingDeleted
$ 四份多文件 harness 用例里 SpaceAdmissionRefused|NoSpaceAfterRaisingTheFloor 的行数（grep -c）
writes_by_inode_number_of_several_files: 0
unlink_of_files_other_than_the_first: 0
unlink_batches_of_an_inode_other_than_the_first_follow_its_own_data_units: 0
writes_by_inode_number_in_random_histories_judged_by_the_model: 0
$ 同四份里 CreateSnapshot|DestroySnapshot|CompactWhileMounted|create_snapshot|publish_relocation_batch 的行数（grep -c）
writes_by_inode_number_of_several_files: 0
unlink_of_files_other_than_the_first: 0
unlink_batches_of_an_inode_other_than_the_first_follow_its_own_data_units: 0
writes_by_inode_number_in_random_histories_judged_by_the_model: 0
$ 变异表里原文含新角色 / 新错误成员 / is_a_data_unit 的行数（awk，按文件）
write_accounting.rs: 0
admission.rs: 0
mount.rs: 0
mounted_session.rs: 1
deleted_inodes.rs: 0
$ grep -c 'death_txg_of_the_deleted_first_file' crates/mutations.tsv；awk 原文含 tombstone.death_txg 且文件是 singlefs-core 的行数
0 / 0
$ grep -c '规范形' .claude/kb/decisions/08-核心索引结构.md
0
$ grep -n -o '^[0-9]\. \*\*[^*]*\*\*' .claude/kb/decisions/08-核心索引结构.md（只取 D8 已定项 16 那一段，行 481–487）
481:1. **意图**
482:2. **分批**
483:3. **建树**
484:4. **读者与写入口**
485:5. **删空**
486:6. **续做**
487:7. **幂等的判据五条**
$ python3 -I tools/anchor_check.py 〔树〕 4810 4904
rows_checked 95 bad 0
$ diff -rq crates 〔树〕/crates
Files crates/singlefs-checker/invariants/03-journal.md and 〔树〕/crates/singlefs-checker/invariants/03-journal.md differ
Files crates/singlefs-checker/invariants/README.md and 〔树〕/crates/singlefs-checker/invariants/README.md differ
$ （在树根）sha256sum -c tree-sha256.txt
admission.rs: OK
deleted_inodes.rs: OK
mounted_session.rs: OK
mount.rs: OK
recovery.rs: OK
transaction.rs: OK
write_accounting.rs: OK
extent_tree.rs: OK
unlink.rs: OK
$ grep -n 'CommitStep\|BlockDeviceAction' research/prompts/m3-close-w5-code-r1-snapshot/product-code.diff | wc -l
0
```
