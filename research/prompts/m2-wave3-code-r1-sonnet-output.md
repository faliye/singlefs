# 云端正推（Sonnet）：`m2-wave3-code-r1` Y2、Y5

分工：Y2、Y5。立场：推得出还是推不出——为每格找能推出今天取法的已定分项或不变量条文，整段抄原文，写从原文到代码的那一步；推不出的写清缺哪一句。不判设计，只判「代码是不是条款的字面后果」。

判据（正文第五节）：兑现了条款（抄出整段原文 + 从原文到代码不引用没有条款的取法）／替没写的条款做了选择（两个都说得通的选择，盘上字节或历史上分得开）／和条款说反话（指出具体字节或历史差异）。

方案按 `crates/` 今天的实现谈：本报告所有 `crates/` 行号都是本次现查（`grep -n`）取自主工作区当前文件，kb 行号同样现查取自 `.claude/kb/`、`.claude/rules/` 当前文件，不从背景材料的附录里数。

---

## Y2　中央映射树根的 I-1.3 按根指针的出生树判

### 子问 1：这是 I-1.3 条文的字面后果，还是替条文做了选择？

**结论：兑现了条款。**

**出处**（`.claude/kb/invariants.md:26`，整行抄）：

> I-1.3 | 块头树 ID 一致 | 任一单元，其头记录的所属树 ID 与实际引用它的树一致。中央映射树不进树表，「实际引用它的树」取根记录（或记录新根段）里中央映射树根指针带的出生树，不写死 15：回退到无文件那一版之后再发第一个文件版本，八棵树从水位起重新发号（D8（核心索引结构） 已定项 8 ②），映射树的号随之变（主 agent 2026-09-23 定，C511（回退到无文件那一版之后诞生代怎么接） 第 3 步实现员查出写死 15 的读法在合法的回退历史上误红）。自证单元（含类型 4 实例表单元）与 I-1.1（块头自述逻辑地址）同口径显式豁免。⚠️ 打包记录单元（码 3）与被快照共享的数据单元头里的树 ID 是出生树，跨头共享时与引用树不同，那一情形的判法随 D9（加密） 那次「绑出生快照」的重开（C110（跨头共享与加密后元数据类的块头一致性读法））。这一射程覆盖码 3 全部打包记录类型，含类型 2（inode 树的叶，D8（核心索引结构） 已定项 6）：克隆头共享 origin 的叶时，叶头的出生树是 origin | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判）

从原文到代码的那一步：条文逐字写「「实际引用它的树」取根记录（或记录新根段）里中央映射树根指针带的出生树，不写死 15」。四处代码逐一核过，都直接读那条指针自己的 `birth_tree`（`PointerHead.birth_tree`，即出生树），不再引用 `TREE_IDENTIFIER_CENTRAL_MAPPING = 15`：

| 位置 | 现查行号 | 现查内容 |
|---|---|---|
| checker | `crates/singlefs-checker/src/walk.rs:449-471`（函数 `walk_tree_table_and_central_mapping_root`） | 467 行起：`self.read_index_node(mapping_root_pointer, parse_node_pointer(mapping_root_pointer).birth_tree, KEY_SCHEMA_MAPPING, "中央映射树的根")`，463-466 行的注释逐字写「I-1.3（块头树 ID 一致） 的「实际引用它的树」按那条指针头部的出生树取…号不写死 15」 |
| 只读挂载 | `crates/singlefs-core/src/mounted_read.rs:328` | `let mapping_tree = root.mapping_root.head.birth_tree;`，随后 `read_tree_root(reader, mapping_tree, …)` |
| 恢复（reconstruct 树 ID 集合） | `crates/singlefs-core/src/recovery.rs:808-824`（函数体内 `tree_identifiers` 构造） | 824 行：`central_mapping: root.mapping_root.head.birth_tree,`，808-809 行注释「中央映射树不进树表、取根记录里它那条根指针的出生树」 |
| 实现（挂载重建树根集合） | `crates/singlefs-core/src/recovery.rs:1527-1534`（`TreeRoots` 构造） | 1531-1533 行：`mapping: read_tree_root(reader, root.mapping_root.head.birth_tree, 27, &root.mapping_root, root, expected_filesystem_identifier)?`，1530 行注释同一句话 |

`TREE_IDENTIFIER_CENTRAL_MAPPING` 常量本身仍在 `crates/singlefs-format/src/lib.rs:127` 定义为 15，但只在 `crates/singlefs-core/src/transaction.rs:1220`（`trees.central_mapping` 的写号公式）与该文件 lib.rs:357 处使用，均属发号（Y1 watermark 的范围），不在判定读法里；四处判定读法（checker / 只读挂载 / 恢复 / 挂载重建）已逐一现查，无一处仍用写死的 15。

**推翻条件**：四处判定读法（checker、只读挂载、恢复、挂载重建）中若有任意一处仍以 `TREE_IDENTIFIER_CENTRAL_MAPPING` 或字面 `15` 判定「实际引用它的树」，或条文原文与这四处实际读的字段位置不一致（例如条文说读指针头部但代码读的是别的偏移），就会推翻这一格「兑现了条款」的结论。今天现查的四处逐一核对一致，未出现这种情形。

### 子问 2：checker 与被判的实现现在用同一个来源判这一格，`.claude/rules/fs-design.md`「审计与被审计用同一段代码」那一条有没有被碰到？

**结论：推不出——这条规则按它自己的正文，射程管不到这一格。**

**出处**（`.claude/rules/fs-design.md:17-41`，整段抄，未转述）：

```markdown
## 记账是事务的副产品，不是事后的遍历

**线划在「谁消费这个数」，不划在「用什么机制算出来」。**

| 消费者 | 允不允许遍历 | 为什么 |
|---|---|---|
| **运行时决策路径**（分配、ENOSPC 准入、defer 窗口、生命周期判定） | **不许**，且代价不许随盘容量增长 | 不是慢，是**在被问到的那一刻没有答案**。准入控制要「进门前先算最坏情况」，而「释放空间这个操作本身不需要申请空间」也压在同一个数上 |
| **checker / 审计** | **必须**遍历 | 若运行时也用遍历算，checker 的遍历与运行时就是**同一次计算**，对照关系当场归零 |
| **后台、可续做、非决策路径**（销毁快照、整理、scrub） | **无戒律** | 它们走「写意图 → 分批 → 可续做」，不参与任何即时判定 |

**空间统计与配额落在第一格**：必须在事务提交时增量维护出来。

⚠️ **一个量落不落在第一格，按这一格自己的判据判：「被问到那一刻有没有答案」，不按名词清单判。**
「快照用量」就是这样判出第一格的：没有任何运行时判定读「每树独占 / 每树共享字节」，
那两个统计量因此撤回、第一版不提供每快照用量——普查、依据与将来做 per-snapshot 配额时怎么办，
权威记录在 `.claude/kb/decisions/05-快照-空间记账机制.md` D5（快照 / 空间记账机制） 已定项索引表第 8 行。

⚠️ **被禁止的不是「遍历」，是两件事**：
「在需要答案的那一刻还没有答案」，以及「审计与被审计用同一段代码」。
第二条不依赖任何关于性能的假设——它要求的恰恰是**运行时与 checker 必须用不同的算法**。

⚠️ **别拿第一格的禁令去否第三格的设计。** 本工程已经在第二、三格豁免过三次：
checker 的全盘遍历、无界删除的意图分批、代际增量 scrub。
**一条被三次豁免的规则，它的判据就不在它字面写的那个量上。**

```

从原文到「推不出」这一步：这条规则的标题、判据表与「被禁止的两件事」全部锚定在同一个对象上——**「这个数」**（记账/统计量，例句是空间统计、配额、快照用量），判据是「运行时要不要靠遍历现算出它」。「审计与被审计用同一段代码」这半句紧跟在这张表之后，说的是同一件事的另一半：若运行时也靠遍历算这个统计量，checker 再靠遍历去核它，两边就是同一次计算，对照关系归零。

Y2 这一格里，checker、只读挂载、恢复三处读的不是一个靠遍历现算出来的统计量，是 `NodePointer` 自己带的 `PointerHead.birth_tree`——一个 O(1) 的字段读取（`parse_node_pointer(...).birth_tree` / `root.mapping_root.head.birth_tree`），不涉及任何遍历、任何现算。fs-design.md 这条规则通篇没有出现「块头自描述」「I-1 类」或「两个存量互相核对」这类字眼，它的射程逐字都在「记账」（第一段小标题）与「统计量」（表格与随后两段 ⚠️）上；I-1.3 判的是块自身头部字段与引用它的指针字段是否一致，属于块自描述一致性检查（I-1 类），不是运行时用遍历现算出来再让 checker 复算一遍的统计量。把这条规则套到 Y2，等于把它的射程从「记账统计量该不该遍历算」外推到「checker 读值的来源能不能与写入端相同」——原文没有写这一步，推不出。

**附带事实**（不改变上面的判定，但记下供别的格引用）：`TREE_IDENTIFIER_CENTRAL_MAPPING = 15` 这个独立常量被拿掉之后，中央映射树的 I-1.3 判定确实变成了「块自己的头部字段」与「指向它的指针自己的字段」互相核对，而这两个字段今天由同一次写（`transaction.rs` 构造映射树节点与它的 `NodePointer` 时用的是同一个 `trees.central_mapping` 变量）各自落盘——checker 判定用的「预期值」不再来自任何独立于这次写入的登记表。这是不是应该另立一条欠账，属于 Y1/Y3/Y4 或攻方腿的射程，Y2 判据第 1、2 条都没有要求判「设计好不好」，本报告不越界判定。

**推翻条件**：若 fs-design.md 正文里出现一句把「记账/统计量的遍历」之外的场合（例如块自描述一致性检查、两个存量字段互相核对）也纳入「审计与被审计用同一段代码」射程的话，或者能证明「实际引用它的树」这个量本身也是某条运行时决策路径要靠遍历现算的统计量（今天不是——它是一次 O(1) 字段读取），就会推翻「推不出」这一判定。

---

## Y5　C512 根记录加分配记录树根、I-9.14 按时间线收窄

### 子问 1：新字段只在树表 0 条、写过行的那一版非零，这是 D16（发布语义）「已定项 9」与 D22（单元原子性怎么合成）「已定项 7」的字面后果吗？

**结论：兑现了条款。**

**出处**（`.claude/kb/decisions/16-发布语义.md:207-221`，整段抄，未转述）：

```markdown
#### 已定项 9：一次空发布也写单元

**定案**：记账树存在时就写——空发布也是发布，按 D5（快照 / 空间记账机制） 已定项 2「每行每发布重写」重写记账行，连带记账树节点、分配记录、映射条目与树表单元。第一次可写挂载的暖机时树表 0 条 ⇒ 零单元；以后的空发布按 D28（挂载期承诺量） 已定项 4 现算的 c_max 块。**树表 0 条的一版上写行的那次发布写两个单元**：实例表与一片分配记录节点，分配记录树根指针住根记录（D22（单元原子性怎么合成） 已定项 7）；被换下的上一版实例表与上一片分配记录节点记成已释放，重开时从这片节点重建这一版的账。根记录那一项只有树表 0 条、写过行的那一版写非零，mkfs 的第 0 代与带文件的一版写全 0，零单元发布照抄上一版。

**射程**：定的是空发布写不写单元、写哪几样，不定 c_max 怎么算（D28（挂载期承诺量） 已定项 4 按当时结构现算）、也不定空发布什么时候推（推抬 F 在已定项 1，暖机在已定项 8）。写行那次发布写的那片分配记录节点，单元头树 ID 写 0（实现今天的取法：那一版的树 ID 水位是 mkfs 的 11，写分配记录树的 13 会让 I-7.8（根记录树 ID 水位不低于全池最大树 ID） 当场红），没有条款；它进不进中央映射也没有条款——那一版没有映射树，D19（块指针的结构与宽度预算） 已定项 8 的自举豁免三类里没有它。

**依据**：

- E148（提交固定点按两棵记录树重算）：空发布的固定点按两棵记录树重算是池规模 9 块、第一个事务规模 4 块，「以后的空发布按现算的 c_max 块」那句的数出自这里。
- E142（第一个事务的干跑）：第一次可写挂载的暖机那两次空发布逐字节写出来，树表 0 条时零单元。
- 用户定案 2026-09-23（写行那次发布在树表 0 条的一版上写实例表与分配记录节点两个单元、分配记录树根指针住根记录），原话在变更史；实现与用例点名在 [checks-owed.md](../checks-owed.md) 已还清表 C512（树表 0 条的一版上被换下的实例表记在哪没有条款） 那一行。
- 用户定案，原话在变更史。

**欠**：C363（现算保留池时树高从哪读没有条款）。

```

同一句在 D22（单元原子性怎么合成）已定项 7 的字段表里又重申了一遍（`.claude/kb/decisions/22-单元原子性怎么合成.md:145`，整行抄）：

> | **分配记录树根指针** | **86** | D16（发布语义） 已定项 9：树表 0 条的一版上写行那次发布写的那一片分配记录节点；只有树表 0 条、写过行的那一版写非零，mkfs 的第 0 代与带文件的一版写全 0；与树表单元指针同型 |

条文给了三种情形各自的取值：mkfs 第 0 代全零、树表 0 条写行那一版非零、带文件的一版全零。三处现查逐一对应：

| 情形 | 现查行号 | 现查内容 |
|---|---|---|
| mkfs 第 0 代 | `crates/singlefs-core/src/make_filesystem.rs:318` | `allocation_record_tree_root: NodePointer::empty_root(),`，紧邻注释「mkfs 的第 0 代不写分配记录树：这一版的账由实例表与树表两条指针直接算得出」 |
| 树表 0 条、写行那一版 | `crates/singlefs-core/src/transaction.rs:901-952`（`publish_instance_table_after_the_release_check`） | 901 行起构造 `allocation_record_tree_root`（指向新取的 `allocation_placement`），952 行把它填进 `RootRecord` |
| 带文件的一版 | `crates/singlefs-core/src/transaction.rs:3372` | `allocation_record_tree_root: NodePointer::empty_root(),`，紧邻注释「带文件的一版的分配记录树住树表条目…根记录这一项恒全零，两处都写就成了同一个量的两份手抄」 |
| 零单元发布（推抬 F） | `crates/singlefs-core/src/transaction.rs:611` | `allocation_record_tree_root: previous_root.allocation_record_tree_root,`，紧邻注释「零单元发布一个字节都不写：分配记录树照抄上一版的那一条指针」 |
| 重开释放上一片 | `crates/singlefs-core/src/transaction.rs:729-737`（`placements_to_release_on_a_version_without_file`） | `if previous_root.allocation_record_tree_root == NodePointer::empty_root() { None } else { Some(…) }`——只有上一版非零（即上一版也是「树表 0 条写行」）才释放，mkfs 第 0 代那一版（全零）不释放 |

四种情形、四处写入点，与条文给的三种取值（含「零单元发布照抄上一版」这一句）逐一对应，字段宽度（86 字节）、偏移（342，见 `.claude/kb/decisions/22-单元原子性怎么合成.md:725` 的偏移累加表）与 `ROOT_RECORD_BYTES = 457`（`crates/singlefs-format/src/lib.rs:136`）也与条文的字段表（`.claude/kb/decisions/22-单元原子性怎么合成.md:707-725`）一致。从原文到代码这一步不需要引用任何条文之外的取法。

**推翻条件**：找到一条可达历史，上述四个写入点之外还存在第五处写 `allocation_record_tree_root` 的路径，或者四处里有任意一处写出的值与条文给的三种取值（mkfs 全零 / 树表 0 条写行非零 / 带文件全零，零单元发布照抄）不符，就会推翻「兑现了条款」。今天现查的五个位置（含释放判定）逐一对应，未发现例外。

### 子问 2：I-9.14 的收窄与 `.claude/kb/invariants.md` I-9.14 那一行今天的条文对得上吗？

**结论：兑现了条款。**

**出处**（`.claude/kb/invariants.md:281`，整行抄）：

> I-9.14 | 树表条目的诞生 txg 跨根不变 | 同一棵树（按树 ID）的树表条目，在**同一条时间线上**的任意两条有效根各自的树表里，诞生 txg 必须相同；只有这棵树第一次出现在树表里时才设这个值（D8（核心索引结构） 已定项 8（树表条目的字段）；C374（释放代与树表诞生 txg 只有验收断言盯着））。「同一条时间线上的有效根」就是回退候选集（D23（journal 的角色与格式） 已定项 14 + D16（发布语义） 已定项 1）：最新那条根本身；以及根环里 (i, T) 满足两条的根——按**最新那条根指着的实例表**，没有实例 i 的行、或有行 (i, Ti, Wi) 且 T ≤ Ti；并且 T ≥ 最新根带的回退下界 F。有行 (i, Ti, Wi) 且 T > Ti 的根在被回退切掉的旧线上，**不与现行线上的根比**（C511（回退到无文件那一版之后诞生代怎么接） 2026-09-23 用户定案收窄射程）。候选集不足两条根，或同一条时间线上没有一棵树的条目出现在两个不同的树表单元里时，报「不适用」，不报成立——几条根指着同一个树表单元、拿同一份字节比自己是恒真判定。判别力：① 把树表条目的诞生 txg 改成跟着这次发布的 txg 走，必须红；② 跨过回退行、同一条线上记得不同，必须红；③ 只有被切掉的那条线记得不同，不许红。推翻条件：找到一条历史，最新根的实例表把某条根判成有效，而它不是最新根的祖先（不在同一条线上）。 | 已实现（2026-09-18，`crates/singlefs-checker` 的 `walk::judge_release_generation_and_tree_table_birth`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `each_c374_bad_image_reddens_only_its_own_invariant_after_the_overwrite`；层 0 每个崩溃状态都判。2026-09-23 收窄之后同文件两条：判别力 ② 的坏镜像 `birth_txg_that_the_line_after_a_rollback_records_differently_from_the_rollback_target_reddens_only_the_birth_invariant`（回退到 A 之后新线上的树表把树 11 的诞生记成 9、A 记 3，只有 I-9.14（树表条目的诞生 txg 跨根不变） 红），判别力 ③ 的正向用例 `birth_txg_recorded_differently_only_on_the_timeline_cut_off_by_a_rollback_is_not_compared_and_every_invariant_holds`；判定在 `walk::judge_tree_table_birth_txg`）

条文把「同一条时间线上的有效根」定义成**回退候选集**，给出精确谓词：无实例 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti（不在被回退抛弃的时间线上）；并且 T ≥ 最新根带的回退下界 F。从原文到代码这一步现查了两层：

**第一层：`judge_tree_table_birth_txg` 只在候选集内部比较**（`crates/singlefs-checker/src/walk.rs:1387-1435`，函数定义与调用点 1441-1493 的 `judge_release_generation_and_tree_table_birth`）：该函数的入参 `scanned: &[ScannedCandidateRoot]` 只包含调用方传入的 `candidate_indexes` 对应的根（1441-1493 行，`for index in candidate_indexes.iter().copied()` 逐条构造 `scanned`），不遍历根环里全部可读根；1410-1417 行按树分组，`distinct_tree_tables.len() < 2` 时 `continue`（跳过，不报违例），与条文「同一条时间线上没有一棵树的条目出现在两个不同的树表单元里时，报『不适用』」一致；1418 行 `judgements.judge("I-9.14", birth_txg_is_the_same_across_roots, …)` 只在 `scanned`（候选集）内部比较诞生 txg。

**第二层：`candidate_indexes` 本身按条文的谓词现算**（`crates/singlefs-checker/src/walk.rs:2810-2828`）：

```rust
let candidate_indexes: Vec<usize> = roots
    .iter()
    .enumerate()
    .filter(|(index, (_, _, root))| {
        let abandoned = instance_table_rows.iter().any(|row| {
            row.instance == root.instance && root.checkpoint_txg > row.published_checkpoint_txg
        });
        let below_floor = root.checkpoint_txg < newest_rollback_floor;
        let walked = *index == newest_index || (!abandoned && !below_floor);
        ...
        walked
    })
    .map(|(index, _)| index)
    .collect();
```

`abandoned` 判的正是条文「有行 (i, Ti, Wi) 且 T > Ti」（`row.checkpoint_txg > row.published_checkpoint_txg` 对应 T > Ti，即被回退切掉的旧线）；`below_floor` 判的正是条文「T ≥ 最新根带的回退下界 F」的反面；`walked = 最新根本身 ∨ (¬abandoned ∧ ¬below_floor)`，与条文「最新那条根本身；以及…没有实例 i 的行、或有行 (i,Ti,Wi) 且 T ≤ Ti；并且 T ≥ F」逐词对应。`newest_rollback_floor` 现查为 `u64::from_le_bytes(roots[newest_index].2.record_bytes[130..138]...)`（`walk.rs:2803-2807`），偏移 130 正是 D22（单元原子性怎么合成） 已定项 7 字段表给回退下界 F 的偏移（`.claude/kb/decisions/22-单元原子性怎么合成.md:725`：「回退下界 F 130」）。

正文 Y5 这句话「I-9.14 被回退切掉的旧线不与现行线比」与条文「有行 (i, Ti, Wi) 且 T > Ti 的根在被回退切掉的旧线上，不与现行线上的根比」逐字相同（`.claude/kb/invariants.md:281`），mount.rs 的现状代码注释（`crates/singlefs-core/src/mount.rs:1419`）同样逐字重复这一句。

**推翻条件**：找到一条可达历史，`candidate_indexes` 的过滤逻辑（`abandoned`／`below_floor`／`walked` 三个布尔量）与条文给的谓词在某个具体 (i, T) 组合上算出不同的真值——例如某条根 T = Ti（等于，不是严格大于）时被判成 `abandoned`，或者 T 恰好等于 F 时被判成 `below_floor`——就会推翻「兑现了条款」；今天现查的比较符（`>` 对 T > Ti，`<` 对 T < F，即 T ≥ F 时不算 below_floor）与条文的 `≤`／`≥` 边界逐个对应，未发现边界错位。

---

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Y2 子问 1（I-1.3 按出生树判） | 兑现了条款 | `.claude/kb/invariants.md:26` 逐字写「取…出生树，不写死 15」，checker（`walk.rs:449-471`）、只读挂载（`mounted_read.rs:328`）、恢复（`recovery.rs:808-824`、`1527-1534`）四处现查都直接读 `PointerHead.birth_tree`，无一处仍用常量 15 |
| Y2 子问 2（fs-design.md「审计与被审计用同一段代码」有没有被碰到） | 推不出 | `.claude/rules/fs-design.md:17-41` 通篇管的是「记账/统计量该不该靠遍历现算」，Y2 这里读的是 O(1) 字段而非遍历现算的统计量，条文没有把这类块自描述一致性检查纳入射程 |
| Y5 子问 1（根记录新字段只在树表 0 条写行那一版非零） | 兑现了条款 | `.claude/kb/decisions/16-发布语义.md:207-221` 与 `.claude/kb/decisions/22-单元原子性怎么合成.md:145` 给出的三种取值，与 `make_filesystem.rs:318`、`transaction.rs:901-952`、`transaction.rs:3372`、`transaction.rs:611`、`transaction.rs:729-737` 五处写入/释放判定逐一对应 |
| Y5 子问 2（I-9.14 收窄与条文对得上） | 兑现了条款 | `.claude/kb/invariants.md:281` 把「同一条时间线上的有效根」定义成回退候选集的精确谓词，`walk.rs:2810-2828` 的 `candidate_indexes` 过滤逻辑与 `walk.rs:1387-1493` 的比较范围逐词对应该谓词 |

## 没做什么

- 不判 Y1、Y3、Y4、Y6：按分工分别归云端攻方（Opus）与本地攻方，本报告未读它们这一轮的提示与产出（禁读清单）。
- 不判设计好坏：Y2 子问 2 附带指出「`TREE_IDENTIFIER_CENTRAL_MAPPING` 常量拿掉之后 I-1.3 对中央映射树变成同源字段互核」这一事实，但不把它算成一次判定，也不建议是否该立欠账——这属于攻方腿或主 agent 的射程。
- 未编译、未跑测试、未跑门禁：本报告的结论全部靠 `grep -n` 现查源码与 kb 文件坐实，没有跑 `cargo test`、`cargo build` 或 `crates/mutations.tsv` 里任何一条变异；Y5 子问 1 提到的 `root_record_of_a_file_version_leaves_the_allocation_record_tree_pointer_zero`（`transaction.rs:3371` 注释里点名的用例）今天在 `crates/` 里搜不到同名测试函数——这是不是意味着该断言缺一条用例，属于 Y6（本地攻方，逐文件测试覆盖表）的射程，本报告只如实记下这一现查结果，不做判定。
- 未跑 `research/scripts/quote-kb.py` 抽取：本报告所有引文手工 `grep -nF` 核对过命中且行号一致（见正文各段「出处」），未使用脚本自动抽取，因为要判的是 `crates/` 当前源码而非仅仅 kb 附录里的既有抽取。
- 未复核背景材料附录里 Y1、Y3、Y4、Y6 相关的补丁内容是否完整或准确：只读了与 Y2、Y5 直接相关的补丁片段（`_m2-wave3-code-r1-diff.md` 第五、七、八节）。

文件：`research/prompts/m2-wave3-code-r1-sonnet-output.md`
