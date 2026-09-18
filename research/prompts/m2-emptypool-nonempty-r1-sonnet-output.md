# m2-emptypool-nonempty-r1 云端正推腿报告

立场：核代码做的是不是条款说的（不替攻方找反例）。分到 Z5、Z7。行号均现查：kb 引用给 kb 文件自己的行号，代码引用给主仓 `crates/` 文件自己的行号（与背景材料附录一致，已用 `grep -n` 核过没有偏移）。

## 各格一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Z5-a：已定项 9 罩不罩得住「写行那次」 | **不一致（部分）** | 已定项 9 字面只说「暖机时」，一个字没提「写行那次」；「写行那次」这个概念真正的条款出处是 D18 已定项 11，不是 D16 已定项 9 |
| Z5-b：⚠️「非空」段与代码三处说法 | **不一致（两处是实现员自己补的）** | 「前一条有效根」一致；「按 (txg, 实例) 排」条款只说「按 txg 排」，instance 是实现员加的 tiebreaker；「最旧那条与没有两棵树比」条款完全没提，是实现员自己填的边界约定 |
| Z5-c：四个拒绝有没有条款 | **一致（四个都没有条款，但记录程度不一）** | 全部 grep 零命中；其中两个milestone 已记决策点，一个只有代码自注「没有条款」但未进决策点清单，一个（`FileVersionWithoutAnyJournalRecord`）既没有条款也没进决策点清单 |
| Z7：milestone 步 3 / 步 5 现状、layout/01-first-txn.md 八 | **一致** | 三处引用的行为、错误名、段序列、闭式状态数都核对上代码与用例；步 5 现状的「非空」概括句比 Z5-b 揭出的实现细节更粗，但没有说错 |

## 一、Z5-a：D16 已定项 9「树表 0 条 ⇒ 零单元」的射程罩不罩得住写行那一次

**已定项 9 原文**（`.claude/kb/decisions/16-发布语义.md:192`，整行抄）：

> 9. **一次空发布写不写单元（2026-09-13，用户定案）：记账树存在时就写：空发布也是发布，按 D5（快照 / 空间记账机制） 已定项 2「每行每发布重写」重写记账行，连带记账树节点、分配记录、映射条目与树表单元；第一次可写挂载的暖机时树表 0 条 ⇒ 零单元；以后的空发布按 D28（挂载期承诺量） 已定项 4 现算的 c_max 块。** 正文见 D16（发布语义）「已定项 9」。 **状态：已定。**

`grep -n "^#### 已定项 9\|^### 已定项 9" .claude/kb/decisions/16-发布语义.md` 零命中——它承诺的「正文见……『已定项 9』」在文件里不存在，第 192 行这一句就是全部。

**这句话的主语是「一次空发布」，字面举的唯一例子是「第一次可写挂载的暖机时」。它没有一个字提到「写行那次发布」。**

而「写行那次发布」是一个独立命名的概念，出处是 D18（块里携带什么信息） 已定项 11（`.claude/kb/decisions/18-块里携带什么信息.md:882`，摘录，整段很长，只抄与本问相关的一句）：

> 每次可写挂载都写行（实例 0 不写；写行那次发布是新实例的第一次发布，新实例的每一个根引用的实例表都已含 [max(所选根的实例, 1), 新实例) 的行；写行之前不推抬 F 的空发布，写行那次的元数据走切换预留，D28（挂载期承诺量） 已定项 3；2026-09-14 用户定案，C329（写行那次发布之前推抬 F 的空发布没有检查） 三轮三方）

**这条才是真正给「写行那次发布」定名字、定语义（新实例的第一次发布）的条款。D16 已定项 9 从没提过这个名字。**

**代码做的是什么**（`crates/singlefs-core/src/mount.rs` `establish_instance`，765-882 行）：`PreviousVersion::WithoutFile` 分支（对应 V2/V3 的「只做过 mkfs 的池」场景）里，`row_publish` 直接调用 `publish_without_units`（`crates/singlefs-core/src/transaction.rs:392`）——与暖机循环里 `publish_empty_after` 走的是**同一层意义上的「零单元发布」**（都不携带任何单元写），但函数本身是两个不同的函数、两处不同的调用点。`establish_instance` 第 824-825 行代码注释自己写：

```
// 上一个实例是 0、要写的行为空：第一次可写挂载的形态，零单元（D16（发布语义） 已定项 9「树表 0 条 ⇒ 零单元」；
```

**这一句引用本身就是把已定项 9 的射程往外拉伸了一步**：已定项 9 字面只谈「暖机时」，这里把它套到「写行那次」头上。拉伸站得住脚的理由是：在 V2 这个具体场景里，`refuse_instance_rows_on_version_without_file`（`mount.rs:744-762`）已经把要写的行区间算成 `[max(0,1), 1) = [1,1)` 空区间（因为实例 0 按 D18 已定项 11「实例 0 不写」不占行、而这次取到的号又恰好是 1），于是「写行」这个阶段名下**没有任何行可写**，于是这一次发布也就没有任何理由重写实例表单元——这时它和「暖机」是同一种物理形态（零单元发布）。但**这个推理链条（0 行 ⇒ 不重写实例表单元 ⇒ 算作已定项 9 的『零单元』）在条款里一处都没有写出来**，是实现员自己接上的，条款字面没有覆盖到这一步。

**结论**：不一致（部分）。已定项 9 的字面射程不包含「写行那次」；覆盖这个场景的条款主体是 D18 已定项 11（给「写行那次发布」定名、定「实例 0 不写」），而「0 行时是否重写/要不要走已定项 9 的零单元路径」这句话，两条条款都没写，是代码自己补的。这本身落在「反向接受条款」的「打中的格落在没有条款的地方」一类，但它是**代码行为本身合理、只是条款字面没跟上**的那种缺口，不是代码做错了。

**什么现象会推翻这个判定**：若能在 D16「已定项 9」或 D18 已定项 11 的正文里再找到一句明确写「写行那次若行为空，视同已定项 9 的零单元发布」，则这一格改判「一致」。我已用 `grep -n "写行"` `.claude/kb/decisions/16-发布语义.md` 通读全文，没有这样一句；`.claude/kb/decisions/18-块里携带什么信息.md:882` 那一大段也没有反过来引已定项 9。

## 二、Z5-b：⚠️「非空」段与代码的三处说法

**条款原文**（`.claude/kb/decisions/16-发布语义.md:383`，整行抄）：

> ⚠️ **「非空」从盘上怎么认（2026-09-17 用户定案）**：一条有效根算非空 ⟺ 它的树表里用户可见的树（inode 树、extent 树）的根指针，与它前一条有效根（按 txg 排；有效 = 按实例表判仍然有效 ∧ txg ≥ 当前的 F）树表里的不同。用户选的是「比较树表与上一条根」；比的是树表里这两棵树的条目、不是树表单元自己的落点——每一次发布（连空发布）都重写树表单元，按落点比会把空发布全算成非空（主 agent 核代码时收窄，`crates/singlefs-core/src/transaction.rs` 里空发布也重写树表）。

逐句对代码（`crates/singlefs-core/src/mount.rs` `rollback_floor_ceiling`，449-519 行；内部循环在 496-514 行）：

| 条款说法 | 条款字面 | 代码怎么做 | 判定 |
|---|---|---|---|
| 「前一条有效根」 | 「与它前一条有效根……树表里的不同」 | `mount.rs:496-505`：`previous_candidates.iter().filter(|candidate| (candidate.checkpoint_txg, candidate.instance) < (root.checkpoint_txg, root.instance)).max_by_key(...)` 取的正是「排在它之前、离它最近的那一条有效根」 | **一致**：概念对上了，取的就是「前一条」 |
| 「按 (txg, 实例) 排」 | 条款只写「**按 txg 排**」，一个字没提「实例」 | `mount.rs:499-505` 排序键是元组 `(checkpoint_txg, instance)`，`instance` 是第二关键字；这处 tiebreaker 连代码自己的文档注释都写明了（`mount.rs:439`：「非空 = 树表里 inode 树、extent 树的根指针与前一条有效根（按 (txg, 实例) 排……）的不同」），只是这句注释没有反过来指出它比 kb 里的 ⚠️ 段多了一维 | **不一致，是实现员自己补的**。条款字面的排序键只有 txg 一维 |
| 「最旧那条与没有两棵树比」 | ⚠️ 段完全没提这一边界情形——它只定义了「有前一条时怎么比」，没说「没有前一条时怎么办」 | `mount.rs:506-508`：`match previous_valid_root { Some(previous) => pointers_of(previous)?, None => UserVisibleTreeRootPointers::ABSENT }`——没有前一条时，拿它去与「两棵树都不存在」这个哨兵值比。这一步同样在代码自己的文档注释里写清楚了：`mount.rs:427-428`（`user_visible_trees_changed` 函数头）「最旧的有效根没有前一条，拿 `UserVisibleTreeRootPointers::ABSENT` 比」 | **条款没写，代码自己填的边界约定并且自己文档化了**（不是「不一致」，是「条款空白，代码补了、也在代码里写清楚了」） |

**关于「按 (txg, 实例) 排」这一句是不是真的会改变结果**：我读了另一处旁证条款——C143（inode 号水位在回退后会退回去重发） 定案 CJ2（`.claude/kb/decisions/23-journal的角色与格式.md:1240`，D23（journal 的角色与格式） 已定项 14 分项 3 之内，整段引在背景材料附录）指向「checkpoint_txg 在全池范围内严格递增，新实例的第一次发布的 checkpoint_txg 必须超过环里已见过的全部 txg」，也就是说**两条根记录不应该共享同一个 checkpoint_txg**——若这条全局不变量成立，`instance` 这个 tiebreaker 在实践中永远不会被用到（比较结果与只按 txg 排完全一样）。**但这只是我推出来的旁证，⚠️ 段本身没有引用它、也没有说明这层理由**；条款字面就是只提 txg。所以我把这一格判「不一致，是实现员自己补的」而不是「一致，因为反正没差」——因为 Z5 问的是「条款字面」对不对得上，不是「代码是否更安全」。

**什么现象会推翻这个判定**：若能证明存在一段可达历史里两条有效根拥有相同的 checkpoint_txg 但不同的 instance（此时 (txg, instance) 排序与纯 txg 排序会给出不同的「前一条」，非空判定的结果也会不同），那么「实例是实现员自己补的、条款没写」这句话仍然成立，但会额外证明这处补漏是**必要**的、不只是防御性冗余；反之若能在 `16-发布语义.md` 里找到一句写着「排序时相同 txg 按实例排」，这一格改判「一致」。

**关于四个 MountError 是否有条款**，见下一节。

## 三、Z5-c：四个拒绝各自有没有条款

**方法**：对每个错误变体名，在 `.claude/kb/decisions/` 全目录下 grep 命中次数（不截断）；再检查 milestone 决策点清单里有没有提到它。

```
$ grep -rn "FileVersionWithoutAnyJournalRecord" .claude/kb/decisions/ | wc -l
0
$ grep -rn "InstanceRowsOnVersionWithoutFileUnsupported" .claude/kb/decisions/ | wc -l
0
$ grep -rn "RollbackToVersionWithoutFileUnsupported" .claude/kb/decisions/ | wc -l
0
$ grep -rn "VersionWithoutFileNotWrittenByMakeFilesystem" .claude/kb/decisions/ | wc -l
0
```

四个变体在 `.claude/kb/decisions/` 全目录（决策正文，也就是「条款」的权威住处）里**一次都没被点名**。逐个再看它们各自的处境：

| 错误变体 | 代码自己的 `///` 文档怎么说（`mount.rs:34-79`，MountError 枚举定义，整段抄见背景材料附录一） | 有没有被记进 milestone 决策点清单 |
|---|---|---|
| `InstanceRowsOnVersionWithoutFileUnsupported` | 「没有文件版本的一版上它的落点记在哪没有条款，第一版不支持」——**代码自己承认没有条款** | **有**：`.claude/kb/milestone/02-second-txn.md:135`，「两格要重写实例表而这一版没有分配记录树与记账树，落点与换下的 mkfs 实例表谁护着没有条款，是设计判断、没做」 |
| `RollbackToVersionWithoutFileUnsupported` | 「没有文件版本的一版上它的落点记在哪没有条款，第一版不支持」——**代码自己承认没有条款** | **有**，同上一行，同一句话覆盖了这两个变体 |
| `VersionWithoutFileNotWrittenByMakeFilesystem` | 「这样一版的分配记录在哪没有条款，这个实现自己写不出这样的根……拒绝挂载」——**代码自己承认没有条款** | **没有**：`grep -n "VersionWithoutFileNotWrittenByMakeFilesystem" .claude/kb/milestone/02-second-txn.md` 零命中，`grep -rn "VersionWithoutFileNotWrittenByMakeFilesystem" .claude/kb/` 零命中——不在 milestone 决策点清单里，也不在 `checks-owed.md` 里 |
| `FileVersionWithoutAnyJournalRecord` | 「从盘上重建上一版要一条记录顶着，拿不出来（树表 0 条的根用不到记录，刚 mkfs 的池不走这里）」——**代码自己也没说这是条款空白**，只是陈述现象 | **没有**：同上，milestone 与全 kb 里都是零命中；它只在 `milestone/02-second-txn.md:135` 现状描述里被提了一句「所选根有文件而环里一条记录都没有返回 \`FileVersionWithoutAnyJournalRecord\`」，那是转述行为，不是把它列进决策点 |

**结论**：四个都没有条款（判定一致，因为这正是背景材料「实现今天的样子」一节自己描述的状态，代码也没有谎称有条款）。但四个的「记录程度」并不一样：前两个已经进了 milestone 的决策点清单（用户看得到「这里还欠一个设计判断」）；后两个没有——`VersionWithoutFileNotWrittenByMakeFilesystem` 代码里明写「没有条款」却漏记决策点，`FileVersionWithoutAnyJournalRecord` 连代码自己都没有点破它没有条款、也没有被记成决策点。

`FileVersionWithoutAnyJournalRecord` 补一句机理核实：它不只在 `mount_rollback` 里被直接 `.ok_or(...)` 出（`mount.rs:1027` 附近，行号见背景材料附录，`mount_rollback` 一节），也可能经 `mount_writable` → `rebuild_previous_version` → `rebuild_version` 的 `NoRecordStandingForFileVersion` 被 `map_rebuild_failure`（`mount.rs:139-146`）转换出来——确认路径：

```
$ grep -n "fn rebuild_previous_version\|FileVersionWithoutAnyJournalRecord\|NoRecordStandingForFileVersion" crates/singlefs-core/src/mount.rs crates/singlefs-core/src/recovery.rs
crates/singlefs-core/src/recovery.rs:476:    NoRecordStandingForFileVersion,
crates/singlefs-core/src/recovery.rs:490:/// 树表不是 0 条而 `record_standing_for_root` 是 `None` ⇒ `NoRecordStandingForFileVersion`；单元读不到或解不开 ⇒ 走读同款的错。
crates/singlefs-core/src/recovery.rs:509:        record_standing_for_root.ok_or(RebuildVersionFailure::NoRecordStandingForFileVersion)?;
crates/singlefs-core/src/mount.rs:36:    FileVersionWithoutAnyJournalRecord,
crates/singlefs-core/src/mount.rs:142:        RebuildVersionFailure::NoRecordStandingForFileVersion => {
crates/singlefs-core/src/mount.rs:143:            MountError::FileVersionWithoutAnyJournalRecord
crates/singlefs-core/src/mount.rs:167:fn rebuild_previous_version<Device: BlockDevice>(
crates/singlefs-core/src/mount.rs:1027:        .ok_or(MountError::FileVersionWithoutAnyJournalRecord)?;
```

**什么现象会推翻这个判定**：若在 `.claude/kb/` 任意文件里找到这四个错误变体名任一处非零命中（我的 grep 没截断、覆盖了 `.claude/kb/decisions/` 全目录与 `milestone/02-second-txn.md`），这一格改判。

## 四、Z7：kb 回写与代码对不对得上

### 4.1 `.claude/kb/milestone/02-second-txn.md:135`（步 3 现状）

原句（整段很长，只抄与本轮改法直接相关的一截，位置见小节标题的行号）：

> ……树表 0 条时返回没有文件的那一版——2026-09-17 用户定案允许只做过 mkfs 的池可写挂载：取号 1、不写行、零单元写行与暖机各一次（txg 1、2）、之后在同一个进程里发第一个文件版本（txg 3），用例 `crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs`，层 0 第三条流 `second_transaction_step_three_formatted_pool_layer0.rs`（闭式 262165，段序列与第一个事务相同）接进门禁 54 号；树表 0 条而要写实例表行（第一个事务暖机之后、文件版本之前崩溃再重开）在取号之前返回 `InstanceRowsOnVersionWithoutFileUnsupported`，回退到树表 0 条的根在任何写之前返回 `RollbackToVersionWithoutFileUnsupported`——两格要重写实例表而这一版没有分配记录树与记账树，落点与换下的 mkfs 实例表谁护着没有条款，是设计判断、没做；所选根有文件而环里一条记录都没有返回 `FileVersionWithoutAnyJournalRecord`……

逐句核：

- **「不写行、零单元写行与暖机各一次（txg 1、2）」**：初读像自相矛盾（先说「不写行」又说「写行」），核代码后是一致的——「不写行」指**没有任何实例表行被写出**（`establish_instance` 第 785 行起的写行循环 `for row_instance in first_row_instance..instance.0`，V2 场景下区间 `[1, 1)` 为空，`rows_written` 是空 `Vec`，`mount.rs:819-823` 处 `assert!(rows_written.is_empty(), ...)` 断言过这一点）；「零单元写行……一次」指**写行这个阶段作为一次发布事件仍然发生**（调用 `publish_without_units`，txg 1），二者不矛盾，只是同一句话里连用了「写行」两种含义（阶段名 vs. 行内容）。**一致，但这句话读起来容易被误解成互相矛盾，值得在下次改写时拆成两句**（这是写法建议，不是事实错误，不计入判定）。
- **段序列 `2+2+1+2+2+1+18+2+1+2`、闭式 262165**：直接读用例源码核过，`second_transaction_step_three_formatted_pool_layer0.rs:54-58` 断言 `segments.iter().map(Vec::len)... == vec![2,2,1,2,2,1,18,2,1,2]`，第 123 行 `assert_eq!(closed_form_state_count(&prepared.segments), 262_165)`——**一致**。
- **`InstanceRowsOnVersionWithoutFileUnsupported` / `RollbackToVersionWithoutFileUnsupported` 触发时机与「没有条款、是设计判断、没做」**：与代码 `mount.rs:56-65` 文档注释、`refuse_instance_rows_on_version_without_file`（`mount.rs:744-762`）、`mount_rollback` 里 `tree_table_has_no_entries` 检查一致（见 Z5-c 一节）——**一致**。
- **`FileVersionWithoutAnyJournalRecord`**：文本只说「所选根有文件而环里一条记录都没有返回」，没有交代它其实有两条触发路径（`mount_writable` 经 `map_rebuild_failure` 间接触发、`mount_rollback` 直接 `.ok_or` 触发，见 Z5-c）——**没说错，但比代码实际的两条路径更粗略**。这不算「不一致」，因为milestone这句话出现在**步 3**（可写挂载）的现状段落里，只需要交代 `mount_writable` 这条路径就够了；它没有跨到步 4（回退）去重复交代 `mount_rollback` 的那条路径，分工上说得通。

### 4.2 `.claude/kb/milestone/02-second-txn.md:189`（步 5 现状，「非空」概括句）

原句片段（整行很长，只抄与「非空」判法直接相关的一段）：

> `rollback_floor_ceiling` = min(每块盘上最新的有效根, 第 4 新的非空有效根)（有效 = 自证 ∧ 按最新根实例表有效 ∧ txg ≥ 今天的 F；非空 = 树表里 inode 树与 extent 树的根指针与前一条有效根的不同，2026-09-17 用户定案，`recovery::user_visible_tree_root_pointers`；一条有效根的树表读不出时抬 F 返回 `RollbackFloorCeilingNeedsUnreadableValidRootTreeTable`、不回收不发布，用户对读不出的态度是走修复、不许猜）

核对：函数名 `rollback_floor_ceiling`、错误名 `RollbackFloorCeilingNeedsUnreadableValidRootTreeTable`、函数名 `user_visible_tree_root_pointers` 都与代码逐字对上（`mount.rs:449`、`mount.rs:73-78` 错误定义、`recovery.rs:702`）。这句概括没有提到 Z5-b 揭出的「按 (txg, 实例) 排」与「最旧那条与没有比」两处实现细节——**一致，但比 Z5-b 的粒度粗**：milestone 现状本身就是概括性质的记录，没有义务复述到排序键这一层，判它「一致」不是因为它没漏东西，而是因为它没写错东西，也没有声称自己覆盖了排序细节。

### 4.3 `.claude/kb/layout/01-first-txn.md:398`（八，「只做过 mkfs 的池的可写挂载」一行）

原文整行已在第一节以上文字里核对过关键点（段序列、闭式状态数、D16 已定项 9 引用）。**一致**，但这一行引的条款同样只写「D16（发布语义） 已定项 9（树表 0 条时空发布写零个单元）」——继承了 Z5-a 揭出的同一个缺口：真正给「写行那次」定名字的是 D18 已定项 11，这一行没有引它。

## 五、副本上跑的十条变异（第 62–71 行，先读第 4 条要求）

副本：`/tmp/claude-1000/m2-emptypool-nonempty-r1-sonnet/`（`rsync -a --exclude target --exclude .git`），独立 `CARGO_TARGET_DIR=/tmp/claude-1000/m2-emptypool-nonempty-r1-sonnet-target`，全程 `nice -n 19`，没跑 release 全量枚举（第三条流的 `#[ignore]` 全量测试没碰）。每条：先核锚点（原文在文件里恰好命中 1 次）→ 改坏 → 跑点名的 `cargo test` 参数 → 记录返回码与 FAILED 行 → 还原（脚本 `finally` 块里用改坏前读进内存的原文写回，不依赖 git）→ 用 `diff -q` 与主仓逐文件核对确认还原干净 → 再跑一遍全部十条点名用例的并集，确认在还原后的代码上全部转绿。

**锚点核实**（十条全部恰好命中 1 次，没有腐化）：

```
line 62 步 5：「非空」退回按环里记录的事务号认（txg 14 的记 anchor_count=1
line 63 步 5：「非空」的前一条取任意可读根（不排除被抛弃的根与 F anchor_count=1
line 64 步 5：「非空」只比 inode 树的根指针、不比 exte anchor_count=1
line 65 步 3：只做过 mkfs 的池重开后分配器不认 mkfs 写 anchor_count=1
line 66 步 3：只做过 mkfs 的池上零单元暖机的反向链写 0 anchor_count=1
line 67 步 3：零单元发布在记录与根之间少一道屏障 anchor_count=1
line 68 步 3：只做过 mkfs 的池重开后把 mkfs 实例表按  anchor_count=1
line 69 步 3：树表 0 条的一版上要写行时，拒绝挪到取号之后（超级 anchor_count=1
line 70 步 4：回退到树表 0 条的根不在写之前拒绝 anchor_count=1
line 71 步 5：算抬 F 上限时有效根的树表读不出，按「树表里没有这 anchor_count=1
```

**改坏 → 点名用例的原样结果**（每条取 `test result:` 那一行与命中 FAILED 的那一行，来自当次真跑）：

| 行 | 变异名（截断） | 点名用例 | 改坏后 | 
|---|---|---|---|
| 62 | 非空退回按事务号认 | `torn_journal_record_does_not_turn_its_root_into_an_empty_root_for_the_floor_ceiling` | `test torn_journal_record_does_not_turn_its_root_into_an_empty_root_for_the_floor_ceiling ... FAILED` / `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out` |
| 63 | 前一条取任意可读根 | `the_rollback_publish_is_compared_with_the_previous_valid_root_and_not_with_the_abandoned_root_before_it` | `... FAILED` / `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out` |
| 64 | 只比 inode 树 | `root_is_non_empty_when_either_user_visible_tree_pointer_differs_from_the_previous_valid_root` | `test mount::non_empty_root_tests::root_is_non_empty_when_either_user_visible_tree_pointer_differs_from_the_previous_valid_root ... FAILED` / `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 38 filtered out` |
| 65 | 分配器不认 mkfs 两个单元 | `writable_mount_of_a_formatted_pool_takes_instance_one_publishes_two_zero_unit_roots_and_the_first_file_version_reads_back_cold` | `... FAILED` / `test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out` |
| 66 | 零单元暖机反向链写 0 | 同上 | `... FAILED` / `test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out` |
| 67 | 零单元发布少一道屏障 | `the_formatted_pool_mount_and_first_file_stream_keeps_the_first_transaction_segment_sequence` | `... FAILED` / `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out` |
| 68 | mkfs 实例表按 1 槽记 | `every_crash_state_outside_the_unit_segment_of_the_formatted_pool_mount_recovers_to_the_version_its_root_claims` | `... FAILED` / `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out` |
| 69 | 拒绝挪到取号之后 | `writable_mount_after_a_crash_between_warm_up_and_the_first_file_is_refused_before_acquiring_an_instance` | `test writable_mount_after_a_crash_between_warm_up_and_the_first_file_is_refused_before_acquiring_an_instance ... FAILED`（补跑单独核实，见下），panic 消息：`assertion \`left == right\` failed: 两盘超级块槽逐字节不变：没有取号` |
| 70 | 回退到树表 0 条的根不拒 | `rolling_back_to_a_warm_up_root_without_a_file_version_is_refused_before_any_write` | `... FAILED` / `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 6 filtered out` |
| 71 | 树表读不出按空猜 | `raising_the_floor_is_refused_when_a_valid_root_tree_table_is_unreadable` | `... FAILED` / `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out` |

第 69 行第一遍脚本跑批时输出被截到 3000 字符尾部、没能在自动抓取里看到那一行 `FAILED`（只看到汇总行），**没有直接放过**：单独重跑确认（命令与 panic 消息见上表），确认它同样正确变红。

**还原 → 绿**：还原后 `diff -q` 逐文件核对 `mount.rs` / `transaction.rs` / `recovery.rs` 与主仓当前工作区**逐字节相同**；随后一次性跑十条用例覆盖到的全部测试二进制：

```
$ diff -q /home/fy5090/code/singlefs/crates/singlefs-core/src/mount.rs /tmp/claude-1000/m2-emptypool-nonempty-r1-sonnet/crates/singlefs-core/src/mount.rs
（无输出，文件相同）
$ diff -q .../transaction.rs .../transaction.rs
（无输出）
$ diff -q .../recovery.rs .../recovery.rs
（无输出）
```

```
$ CARGO_TARGET_DIR=/tmp/claude-1000/m2-emptypool-nonempty-r1-sonnet-target cargo test -p singlefs-harness \
  --test second_transaction_step_five_reuse --test second_transaction_step_three_formatted_pool \
  --test second_transaction_step_three_formatted_pool_layer0 --test second_transaction_step_four_rollback \
  -p singlefs-core --lib
...
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.94s   （five_reuse）
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.10s    （four_rollback）
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.38s    （three_formatted_pool）
test result: ok. 2 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.46s    （three_formatted_pool_layer0，全量枚举那条按定义 ignored 不碰）
```

单独确认 `root_is_non_empty_when_either_user_visible_tree_pointer_differs_from_the_previous_valid_root`（在 `-p singlefs-core --lib` 的输出里，未单独截出）：

```
$ CARGO_TARGET_DIR=/tmp/claude-1000/m2-emptypool-nonempty-r1-sonnet-target cargo test -p singlefs-core --lib -- root_is_non_empty_when_either
test mount::non_empty_root_tests::root_is_non_empty_when_either_user_visible_tree_pointer_differs_from_the_previous_valid_root ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 38 filtered out; finished in 0.00s
```

**十条全部「改坏 → 点名用例红、还原 → 绿」，符合表里承诺的行为；这十条本身不归我判（Z6 是本地攻方的格），这里只按「先读」要求提供原始结果，不下 Z6 的判定。**

## 六、这条腿自己的限度

- **不替攻方找反例**：我没有去构造能让 Z5-b 里「实现员自己补的两处」（`(txg, 实例)` 排序、「最旧比 ABSENT」）在某段可达历史上产生错误结果的具体场景——那是 Opus 攻方腿（Z1）的工作，我只核了条款字面对不对得上、以及代码有没有自己把这两处写进注释（有）。
- **Z1–Z4、Z6 不归我判**：我按「先读」要求跑了第 62–71 行十条变异并贴了原样结果，但没有对「变异是不是真钉住了那条改法」「改坏之后会不会在别的改法上也红」下判定——那是本地攻方腿的格（Z6）。
- **D16 已定项 6、D23 已定项 14 全文没有展开核对**：Z5-b 里我只引了 C143 CJ2 那一句作旁证，没有把 D16 已定项 6（发布计数器）与 D23 已定项 14（重放的下界由所选根给出）整条读一遍去确认「checkpoint_txg 全局唯一」这条推论在所有分支（普通挂载、切换、回退）下都成立——这条只是我用来判断「(txg, 实例) 这个 tiebreaker 是不是纯防御性冗余」的旁证，不影响 Z5-b 的主判定（条款字面确实只写了「按 txg 排」）。
- **没有跑门禁 54 / 59 号**：这两个阶段不在我的 `stage-owners.tsv` 名下（按 agent-common 的说法，逐个 `awk` 查询本该做，但本轮任务书没有单列这一步；十条变异我自己在副本上手动跑等价的改坏-测试-还原，覆盖的是与 59 号相同的判据，但不等于跑过 59 号本身）。
- **第一版草稿里多处代码行号引用有误，写作过程中自查发现并现场订正**（用 `research/scripts/replace-once.py` 定点改，每处都重新 `grep -n` 核实）：`mount.rs:250-256`→`496-508`/`499-505`、`mount.rs:788-795`→`819-823`、`establish_instance 第 850 行`→`824-825 行`、`mount.rs:458 行起`→`785 行起`、`mount.rs:96-118`→`34-79`、`mount.rs:96-105`→`56-65`、`mount.rs:113-118`→`73-78`、C143 CJ2 的出处从误引 `16-发布语义.md` 订正为 `.claude/kb/decisions/23-journal的角色与格式.md:1240`。错误的共同根源是**把背景材料附录里代码块相对于摘录本身的行号，误当成了 kb / 主仓文件的绝对行号**；订正后的每一处都用 `grep -n` 或 `sed -n` 现查过、不是凭记忆改的。这条本身正是 `three-way-inference.md`「引产物就整行抄」在代码引用上的同类坑，写在这里供核查员复核时留意——如果还有漏网的，判据是：任何 `文件.rs:数字` 都应该能用 `sed -n '数字,数字p' 文件.rs` 现查对上引文。

## 没做什么

- 不判 Z1（非空判法的漏判与误判，含要不要造反例的历史）、Z2（拒绝时机）、Z3（空池挂载之后第一个文件版本的字节一致性）、Z4（层 0 第三条流的枚举正确性）——这四格是 Opus 攻方腿的。
- 不判 Z6（变异与用例的钉住程度），只提供原始复跑结果。
- 不出判决、不采纳或推翻任何一条改法——判决是主 agent 的事。
- 没有跑 `check.sh`、命名纪律、门禁 54 / 59 号全量——背景材料称主 agent 已经跑过并全绿，我没有重复验证这几项。
