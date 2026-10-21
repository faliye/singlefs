# m3-multi-file-unlink-r1：云端正推腿报告（Sonnet，2026-10-19）

立场：从条款原文与补丁树 `~/.local/share/singlefs/work/m3-multi-file-unlink-r1-tree` 的代码推「应该是什么」，再与被判对象（D8 已定项 16 现有句子、背景材料第二节的观测）逐格比。
没读攻方腿（`research/prompts/m3-multi-file-unlink-r1-opus-output.md`、`…-opus-model/`）。代码行号都是那棵树里的；kb 行号是 kb 文件自己的行号，都在写进来之前 `grep -nF` 过。

## 0　模型与复跑

我写了一个独立小模型（标准库 Python，不读 `crates/`，几何常量当参数）：上段形状规则 A（形状只由有条目 inode 集合 S 定）对规则 B（根层级只升不降，写路径今天的读法），
按「变了的号那一串重写」与「按内容算最小重写集」逐步对拍，附带下段批规则（`last_data_unit < data_units_kept`）的同款对拍与三个必红变异。
模型目录 `research/prompts/m3-multi-file-unlink-r1-sonnet-model/`。

复跑命令（工作目录随意，`--work` 给新目录）：

```
bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-multi-file-unlink-r1-sonnet-model/tools/rerun.sh --repo <仓根> --frozen ~/.local/share/singlefs/work/m3-multi-file-unlink-r1-tree --work <新的草稿目录>
```

我这一次真跑（经内存包装，排了 24 秒）的末两行：

```
name=rerun_roots repo=/home/fy5090/code/singlefs frozen=/home/fy5090/.local/share/singlefs/work/m3-multi-file-unlink-r1-tree model=/home/fy5090/code/singlefs/research/prompts/m3-multi-file-unlink-r1-sonnet-model work=/home/fy5090/.local/share/singlefs/work/m3-multi-file-unlink-r1-sonnet/rerun-work2
name=rerun_compare result=identical lines=31 sha256=7783d076f0cc9fa21d06296d6af1c221550df0d86c5ee7095ec3f29bbea7d454
```

| 文件 | sha256 |
|---|---|
| `upper_shape_model.py` | b210c42d5174449465422a971231c126f318a2c5a872f187ec0062e08d4430b4 |
| `tools/rerun.sh` | d8278b878173c0d9bb7ede75d1a500f8bf8ffeead57678e9bb94083eb3401774 |
| `out/run.out`（31 行，复跑逐行相同） | 7783d076f0cc9fa21d06296d6af1c221550df0d86c5ee7095ec3f29bbea7d454 |

`tools/rerun.sh` 是 `scripts/agents/three-way-rerun-template.sh` 只改 `rerun_steps` 一段；模板自带的 `--selftest` 测的是模板默认步骤，改了步骤之后它会红，没当证据用。
模型不模拟 core 的角色名（根与根之下的节点名）、映射 key、inode 树，只管上段位置集合与「照抄 / 重写」；这些在下面各格里标「推的」。

## 1　背景材料第二节的观测，我现查的结果

| 背景材料说的 | 现查 | 判 |
|---|---|---|
| 叶罩 143 个 inode 号、扇出 147 | `crates/singlefs-format/src/lib.rs:186`：「pub const EXTENT_TREE_UPPER_LEAF_INODES: u64 = 143;」、`crates/singlefs-format/src/lib.rs:176`：「pub const EXTENT_TREE_INTERNAL_FANOUT: u64 = 147;」 | 一致 |
| 写别的文件的上段计划在 transaction.rs 第 6426 行 | `crates/singlefs-core/src/transaction.rs:6426`：「pub fn plan_the_extent_tree_after_writing_a_file_other_than_the_first(」 | 一致 |
| unlink 批在 unlink.rs 第 157 行，最后一批在第 234 行，断言上段只有根兼叶 | `crates/singlefs-core/src/unlink.rs:157`：「pub fn plan_the_extent_tree_after_a_deletion_batch(」、`crates/singlefs-core/src/unlink.rs:234`：「pub fn plan_the_extent_tree_after_the_final_batch_of_the_first_file(」、`crates/singlefs-core/src/unlink.rs:240`：「"第一版只有第一个文件在 extent 树里有条目：上段只有根兼叶一个节点"」 | 一致 |
| 下一步在 transaction.rs 第 9279 行，别的 inode 剩余单元恒取 0 | `crates/singlefs-core/src/transaction.rs:9279`：「fn next_unlink_step_after(」、`crates/singlefs-core/src/transaction.rs:9296`：「// 第一版只有第一个文件写数据：别的 inode 一个数据单元都没有，意图之后直接是最后一批。」 | 一致 |
| 拒绝在 transaction.rs 第 9712 至 9740 行 | `crates/singlefs-core/src/transaction.rs:9713`：「fn refuse_an_unlink_the_deletion_steps_do_not_cover(」，函数到 9738 收（差一行，不影响） | 一致 |
| 恢复要求第一个文件之外的文件至少一个单元 | `crates/singlefs-core/src/recovery.rs:5228`：「if data_pointers_of_the_file.is_empty()」 | 一致 |
| `is_a_version_with_trees_and_no_file` 的判定 | `crates/singlefs-core/src/transaction.rs:3468`：「let inode_tree_root_has_no_entry = self.inode_leaf_containers.is_empty();」、`crates/singlefs-core/src/transaction.rs:3470`：「self.data_pointers.is_empty() && self.files_other_than_the_first.is_empty();」 | 一致 |
| 池级 checker 对墓碑压着的对象报不适用 | `crates/singlefs-checker/src/rebuild.rs:769`：「if tombstoned_objects.contains(&(inode, object_birth)) {」、`crates/singlefs-checker/src/rebuild.rs:771`：「return Err(ScanRebuildNotDecided::DeletionInProgressNotWrittenForTheScanRebuild);」 | 一致 |
| 「上段的根…层级是罩得住有条目的最大 inode 号的最低一层」 | 只在代码文档里：`crates/singlefs-core/src/extent_tree.rs:5`：「层级是罩得住有条目的最大 inode 号的最低一层。」。kb 里没有：`grep -rnF "最低一层" .claude/kb --include=*.md`（去掉 `.ja.md` `.en.md`）0 行 | 一致（背景没说它在 kb；要补一句：这条规则 kb 条款没有） |

## 2　条款与代码并排（不一样的地方，整行抄两边）

| 编号 | 一边 | 另一边 | 判定 |
|---|---|---|---|
| X1 上段根层级 | `crates/singlefs-core/src/extent_tree.rs:5`：「层级是罩得住有条目的最大 inode 号的最低一层。」 | `crates/singlefs-core/src/transaction.rs:6445`：「upper_root_level_for(largest_inode_with_an_entry).max(previous_upper_root_level);」，第一个文件的写路径同款：`crates/singlefs-core/src/transaction.rs:6203`：「upper_root_level_for(FIRST_INODE_NUMBER).max(previous_upper_root_level.unwrap_or(0));」 | 冲突，今天不显：有条目集合只增不减时「最低一层」只升不降，`.max(上一版)` 不起作用；一有删除就起作用（见 U2） |
| X2 删完之后 | `.claude/kb/decisions/08-核心索引结构.md:485`：「删完之后只能做 `publish_new_inodes`（size 0）、写行与空发布（K-U5 X1）」（第 5 条该句，整句） | 补丁树按 inode 号写的入口只拒两样：`crates/singlefs-core/src/transaction.rs:8338`：「.ok_or(PublishError::WriteToAnInodeThatIsNotInTheTree { inode: inode.0 })?;」、`crates/singlefs-core/src/transaction.rs:8341`：「return Err(PublishError::WriteToAnInodeBeingDeleted { inode: inode.0 });」；第一个文件删完之后给新建的 inode 写带数据的版本不拒 | 冲突（实现员报告 Q1 已报；用户 2026-10-19 定「拦」，还没实现） |
| X3 射程句 | `.claude/kb/decisions/08-核心索引结构.md:489`：「**射程**：第一版只做单文件 unlink、按 inode 号、无目录、不带 truncate（用户 2026-09-28 定）；删完之后不再建带数据的文件（K-U5，记欠账）」 | 会话入口已有截断：`crates/singlefs-core/src/mounted_session.rs:128`：「Truncate {」；unlink 各步的前提句（`crates/singlefs-core/src/transaction.rs:9296`：「第一版只有第一个文件写数据」）也已被按 inode 号写数据推翻 | 冲突（「不带 truncate」半句早被里程碑四推翻；本轮改这一句时要一并改，不是本轮的格） |
| X4 删除类步骤的 inode | `.claude/kb/decisions/08-核心索引结构.md:481`：「`deleted_inodes` 加 key = N（8 字节、无 value、不带对象出生代），往墓碑树开放容器追加一条墓碑」（第 1 条，N 是任一 inode） | `crates/singlefs-core/src/transaction.rs:4986`：「DeletionStepOfThePublish::FinalBatch { inode } if inode.0 == FIRST_INODE_NUMBER => 0,」、`crates/singlefs-core/src/transaction.rs:4983`：「| DeletionStepOfThePublish::TruncationFinalBatch { data_units_kept } => {」；`Batch` 一支（`crates/singlefs-core/src/transaction.rs:4941`：「Batch {」）没有 inode 字段，`data_units_after_this_publish` 把它的 `data_units_kept` 当第一个文件的 | 冲突（条款对任一 inode 写，代码的批只认第一个文件；`refuse_…` 正好挡在前面（`crates/singlefs-core/src/transaction.rs:9209`：「refuse_an_unlink_the_deletion_steps_do_not_cover(previous, inode)?;」），只删拒绝不改这一支会让对 inode N 的批截第一个文件，见 U1） |

## 3　U1　删一个第一个文件之外、带数据单元的文件

### 3.1　D8 已定项 16 现有的句子能不能原样套到 inode N（N 不是 1）

| 步 | 条款现句（kb 行） | 套到 N 上 | 判定 |
|---|---|---|---|
| 意图 | `.claude/kb/decisions/08-核心索引结构.md:481`：「`deleted_inodes` 加 key = N（8 字节、无 value、不带对象出生代），往墓碑树开放容器追加一条墓碑」 | 字面对任一 N。代码本来就不分：记录从 inode 树按号取，墓碑用记录自己的对象出生代：`crates/singlefs-core/src/transaction.rs:9225`：「tombstone: TombstoneRecord::of_the_whole_object(inode, record.object_birth, txg),」；建 inode 时对象出生代 = 创建那次发布的 txg：`crates/singlefs-core/src/transaction.rs:8196`：「object_birth: txg,」。数据单元与 extent 树照抄。唯一挡路的是 `crates/singlefs-core/src/transaction.rs:9209`：「refuse_an_unlink_the_deletion_steps_do_not_cover(previous, inode)?;」 | 一致（条款对任一 N 成立，代码只被那一句拒绝挡着） |
| 分批 | `.claude/kb/decisions/08-核心索引结构.md:482`：「从尾部按数据单元个数分批，每批释放尾部 K 个单元、size 改成」（第 2 条该句，后接「剩下单元数 × 净荷容量」） | 字面对任一文件。没写「剩下单元数」从哪读：第一个文件是 `data_pointers`，别的文件在 `files_other_than_the_first`（`crates/singlefs-core/src/transaction.rs:9297`：「let data_units_remaining = if inode.0 == FIRST_INODE_NUMBER {」，else 支恒 0）。也没写批要不要连上段 N 那一串一起重写 | 规则没说（剩余单元数的来源、上段那一串） |
| 最后一批 | `.claude/kb/decisions/08-核心索引结构.md:482`：「最后一批释放剩下全部单元、删 inode 记录、删 `deleted_inodes` 那条 key，同一次发布；只有这一条路，不设一次做完的快路。」 | 对 N 字面成立。没写 extent 上段 N 那一条怎么办 | 规则没说（上段那一格） |
| 同一事务 | `.claude/kb/decisions/08-核心索引结构.md:120`：「删除意图必须与最后一批工作在同一个事务里提交」（前半句是「不可能出现意图没了但目标状态没达成」） | 对 N 成立：最后一批一次发布里删记录、删队列 key、去上段条目 | 一致 |

### 3.2　最后一批对上段那一格：去掉条目，还是写标签 0

| 看哪 | 去掉条目（上段叶里不再有 N 那一条；叶空了就换下，U2） | 写标签 0（N 那一条留在叶里，载荷全零） |
|---|---|---|
| 写者 | 同第一个文件今天的最后一批：没有单元就不写那一条，`crates/singlefs-core/src/transaction.rs:11228`：「(None, None) => None,」。零行新语义 | 写者要第一次写标签 0。kb 现句 `.claude/kb/decisions/08-核心索引结构.md:431`：「标签 0 全零，只有读者认、写者不写」；代码文档同：`crates/singlefs-core/src/extent_tree.rs:13`：「从偏移 0 顺序写、长度 0 的内容也写一个声明长度 0 的数据单元；变长不写新单元，最后一个单元之后到 size 是洞，K5）；标签 0 由读者与」。要改这两句 |
| 读者 `open_file` | 记录已删，`NoSuchInode`，不走到上段 | 同左（记录已删，不看条目） |
| 恢复 | 无改动：删除中间态里 N 至少留一个单元，最后一批之后上段没有 N | `crates/singlefs-core/src/recovery.rs:5228`：「if data_pointers_of_the_file.is_empty()」对第一个文件之外的文件判 `UnitMalformed`，要放开；放开后 `FileOfAnInodeOtherThanTheFirst` 要能装 0 个数据指针，与 `crates/singlefs-core/src/transaction.rs:3094`：「每个文件的数据指针与它那一棵下段的节点。只写过建 inode（size 0）的 inode 没有数据单元，不在这里。」冲突 |
| 池级 checker | I-4.6 / I-4.7 / I-4.10 / I-9.5 都按「记录」或「extent 树里的单元」循环，没有 N 的记录也没有单元，不报 | I-4.6 按 `records` 循环（`crates/singlefs-checker/src/walk.rs:11541`：「for (inode, record) in records {」），N 记录没了，那条标签 0 孤儿条目没有任何不变量盯它；要新立一条「标签 0 条目必须配一条活记录」 |
| 扫描重建（I-1.13） | 只看单元头与 inode 记录，不看上段，无影响 | 同左，无影响 |
| 上段形状 | 叶里最后一条被去掉时叶是空的，读者 `crates/singlefs-core/src/extent_tree.rs:816`：「if header.entries.is_empty() && !(is_the_root_of_the_tree && expected_level == 0) {」与 checker `crates/singlefs-checker/src/lib.rs:701`：「if view.entries.is_empty() {」都拒空的非根节点，所以那片叶必须换下（U2） | 叶不空，叶与祖先都留着；但「有条目的 inode 集合」要把已删的号也算进去，同一个活文件集合有多种上段形状（取决于历史），U2 的「一种形状」落空；而且没有任何清理者，已删的号的条目永远留着（推的：没找到会摘它的代码路径） |

结论（推的 + 上面逐格现查）：最后一批去掉条目。写标签 0 要同时改 kb 一句、恢复一处、`TransactionOutput` 的约定、新立一条不变量，
还留下孤儿与多形状。没有一格上标签 0 更便宜（写节点数两边相同：N 那一片叶到根那一串都重写；去掉条目时空叶不重写，反而少一个）。
**什么现象会推翻它**：读者或恢复需要「这个号曾经存在过」在上段留痕（例如某条不变量要用已删号的条目判墓碑）；或标签 0 因别的原因变成写者要写的（改 `.claude/kb/decisions/08-核心索引结构.md` 第 431 行那一句）。

### 3.3　要改的 `crates/` 函数清单（行号是补丁树里的；行数是推的，没编译过）

| # | 位置 | 改法一句 | 估行 |
|---|---|---|---|
| 1 | `crates/singlefs-core/src/transaction.rs` 第 9297 行（`next_unlink_step_after`） | 剩余单元数按号取：1 取 `data_pointers.len()`，别的取 `file_other_than_the_first(inode)` 的单元数，没有取 0 | +8 / −5 |
| 2 | `crates/singlefs-core/src/transaction.rs` 第 4941、4979、4986 行（`DeletionStepOfThePublish::{Batch, FinalBatch}`、`data_units_after_this_publish`） | `Batch` 带 inode；`data_units_after_this_publish` 只回答第一个文件，对别的号的 `Batch` 不得把 `data_units_kept` 当第一个文件的（唯一的使用处在 `crates/singlefs-core/src/transaction.rs:6968`：「.data_units_after_this_publish(data_units_before);」）；别的文件还剩几个另一个函数答 | +25 |
| 3 | `crates/singlefs-core/src/transaction.rs` 第 6989、6998 行（extent 计划的选择）与第 7021 行（写别的文件的覆盖） | `Batch` / `FinalBatch` 按号分流：1 走第一个文件的计划，别的走新的「别的文件的删除计划」；最后一批对 1 也走 U2 的统一上段计划（不再断言上段只有根兼叶） | +40 |
| 4 | `crates/singlefs-core/src/transaction.rs` 第 6340 行（`plan_the_files_other_than_the_first_after_this_publish`）与第 6270 行（`FileOtherThanTheFirstPlan.lower_nodes: Vec<ExtentLowerNodePosition>`） | 一个文件这次之后有四种来历：整份重写、照抄、被截（留前 k 个）、被摘；下段节点要逐个带「照抄 / 重写」（今天是整个文件一个来历） | +60 / −15 |
| 5 | `crates/singlefs-core/src/unlink.rs` 第 157 行 | 把「下段按留下的单元数重算、整片留下的照抄」抽成与 inode 无关的函数，第一个文件与别的文件共用。**变异表按字面锚着这几处**：`crates/singlefs-core/src/unlink.rs:169`：「last_data_unit < data_units_kept」、`crates/singlefs-core/src/unlink.rs:211`：「!carried_lower.contains(position))」，抽函数时字面原样保留或同批改 `crates/mutations.tsv` 的行（变异表里点名 unlink.rs 的 11 行，`awk` 数的） | +30 / −20 |
| 6 | `crates/singlefs-core/src/transaction.rs` 第 11108、11186、7586、12358、13234 行（装下段、上段叶条目、映射 key、数据指针、数据单元） | 被截的文件只装留下的单元与指针；被摘的文件不进这几张表；被截文件的数据指针取上一版前 k 个（今天照抄的一支整份 `.clone()`，`crates/singlefs-core/src/transaction.rs:12366`：「.expect("照抄的文件在上一版里")」之前） | +45 |
| 7 | `crates/singlefs-core/src/transaction.rs` 第 6776 行（`previous_roles_without_a_successor`）、第 5916 行（`previous_roles_replaced_by_this_publish`） | 被截 / 被摘文件尾部的 `DataOfAFileOtherThanTheFirst` 与不再出现的下段节点进「没有同名后继的上一版角色」，次序按 bump 次序（第一个文件的数据、别的文件的数据、extent 下段、上段） | +25 |
| 8 | `crates/singlefs-core/src/transaction.rs` 第 9713 行（`refuse_an_unlink_…`）、`crates/singlefs-core/src/deleted_inodes.rs` 第 35 行（枚举成员）、`crates/singlefs-harness/tests/writes_by_inode_number_of_several_files.rs` 第 482 行起的拒绝用例 | 删函数、删成员、拒绝用例改成成功用例；`PublishError` 本身不动（加成员会让 checker 档 e181 的穷举 `match` 编不过，实现员 Q4） | −45 / +60（测试） |
| 9 | `crates/singlefs-core/src/recovery.rs`、`mount.rs`、`mounted_session.rs` | 无改动（推的）：删除中间态每一版 N 至少留一个单元、标签 1 或 2，过 `crates/singlefs-core/src/recovery.rs` 第 5228 行那一判；续做取队首号，`crates/singlefs-core/src/transaction.rs:9562`：「let Some(inode) = version_rehearsed.deleted_inodes.first().copied() else {」，与号无关 | 0 |

核心层合计约 +235 / −85（推的，逐行相加；行 8 的 −45 与 +60 测试不在内），不含 U2 的统一上段函数（那一格另计）和测试。

### 3.4　条款原句形态（建议改的是 D8 已定项 16 第 2 条，在现句上插两处、改一处，不重抄整条）

1. 在「从尾部按数据单元个数分批，」之后插：「对任一 inode N 同一条：N 现行那一版剩下的数据单元数 r 取 N 在 extent 树里的单元数（第一个文件取它自己那一棵，别的文件取上段 N 那一条所指的，没有那一条的取 0，那样的号只有最后一批）；r 多于 K 释放尾部 K 个，不多于 K 是最后一批；」
2. 在「删除批只重写被截掉那一片下段叶与它的祖先」之后插：「，再加上段 N 那一片叶到根那一串（上段形状见第 8 条）」
3. 「最后一批释放剩下全部单元、删 inode 记录、删 `deleted_inodes` 那条 key」之后插：「、把 N 的条目从 extent 树上段去掉（不写标签 0，写者不写它，D8 已定项 14）」，其后「同一次发布」不动。

**什么现象会推翻 U1 的结论**：(a) 若 `Batch` 的其它消费处按 `record.inode` 分了流（我 `grep -n "DeletionStepOfThePublish::Batch" crates/singlefs-core/src/*.rs` 只看到 transaction.rs 第 6968 行一处用 kept 数、第 6989 行起一处用 `FIRST_INODE_NUMBER`，其余都是 `{ .. }`），则第 2 行的 +25 可以少；(b) 补丁树上做一个 inode 2 带 300 个单元、inode 1 带 3 个单元的用例，只删拒绝跑 unlink(2)：第一批之后第一个文件被截（预期会红，我没编译跑，标「推的」）。

## 4　U2　上段形状随删除收缩

### 4.1　今天四处各算各的形状

| 处 | 它怎么定上段形状 | 行 |
|---|---|---|
| 写第一个文件 | 根层级 = max(罩得住 inode 1 的层级, 上一版根层级)；节点 = 第一个文件那一串 ∪ 上一版全部上段节点（只增不减） | `crates/singlefs-core/src/transaction.rs:6203`：「upper_root_level_for(FIRST_INODE_NUMBER).max(previous_upper_root_level.unwrap_or(0));」、`crates/singlefs-core/src/transaction.rs:6218`：「.union(&previous_upper)」 |
| 写别的文件 | 根层级 = max(罩得住有条目最大号的层级, 上一版根层级)；节点 = 有条目各号的叶到根那一串的并；上一版有而这一版不在任何一串上的换下 | `crates/singlefs-core/src/transaction.rs:6445`：「upper_root_level_for(largest_inode_with_an_entry).max(previous_upper_root_level);」 |
| 删除批 | 层级不变，上段全照抄、只重写这个文件那一串 | `crates/singlefs-core/src/unlink.rs:189`：「let upper_root_level = previous.upper_root().level;」 |
| 删第一个文件的最后一批 | 断言上段只有一个根兼叶，重写成零条目根 | `crates/singlefs-core/src/unlink.rs:240`：「"第一版只有第一个文件在 extent 树里有条目：上段只有根兼叶一个节点"」 |

截断另有一处同款（`crates/singlefs-core/src/truncate.rs:275`：「let upper_root_level = previous.upper_root().level;」），截断每次至少留一个单元（`crates/singlefs-core/src/truncate.rs:241`：「(1..=data_units_before).contains(&data_units_after),」），不改有条目集合，所以不用动。

### 4.2　什么形状是合法的（读者与 checker 已经定死的部分）

- 非根的节点不许零条目：读者 `crates/singlefs-core/src/extent_tree.rs:816`：「if header.entries.is_empty() && !(is_the_root_of_the_tree && expected_level == 0) {」；
  池级 checker 经 I-1.1 同判：`crates/singlefs-checker/src/lib.rs:701`：「if view.entries.is_empty() {」。装叶的断言比这两道松一档：`crates/singlefs-core/src/extent_tree.rs:607`：「(!entries.is_empty() || position.index == 0)」放过层级 0 的第 0 片叶（不问它是不是根），不是第三道拒。
  所以「删空一片非根上段叶后留成零条目叶」不是候选，读者与 checker 两道都拒；零条目只许层级 0 的根兼叶。
- 根的层级取不取最低一层，读者与 checker 都不判：读者从根节点头里读层级，`walk_extent_upper_node` 只核层级与位置相符（`crates/singlefs-checker/src/walk.rs` 第 4449 行起 `walk_extent_upper_node` 那一段没有「层级最低」一判，我读完了整段）；
  kb 里没有这句话：`grep -rnF "最低一层" .claude/kb --include=*.md`（去掉 `.ja.md` `.en.md`）0 行。「最低一层」只在代码文档里：`crates/singlefs-core/src/extent_tree.rs:5`：「层级是罩得住有条目的最大 inode 号的最低一层。」
  所以 A（降层）与 B（只升不降）都读得出、判得过；规则是空的，不是被谁否掉的。

### 4.3　候选规则与模型量到的数（量的：`research/prompts/m3-multi-file-unlink-r1-sonnet-model/out/run.out`；两条规则跑同一串操作，S 与规则无关）

规则 A：形状只由有条目 inode 集合 S 定。节点 = S 里每个号从叶到根那一串的并；根层级 = 罩得住 max(S) 的最低一层；S 空时层级 0 第 0 格一个零条目根兼叶。
规则 B：写路径今天的读法——根层级 = max(罩得住 max(S) 的最低一层, 上一版根层级)，S 空时落回零条目根。

| 几何（40 个种子 × 120 步） | 规则 | 删时重写节点 | 写时重写节点 | 删时换下节点 | 降层次数 | 同一个 S 出现两种形状的 S 数 / 出现过的 S 数 |
|---|---|---|---|---|---|---|
| 小几何 U=3 F=2（层级常变） | A | 4287 | 10871 | 7435 | 1037 | 0 / 1877 |
| 小几何 U=3 F=2 | B | 5854 | 12394 | 8689 | 724 | 206 / 1877 |
| 产品几何 U=143 F=147 | A | 3734 | 7880 | 5897 | 257 | 0 / 4441 |
| 产品几何 U=143 F=147 | B | 3914 | 7937 | 6003 | 183 | 25 / 4441 |

行号：`out/run.out` 第 1、3、5、7 行（各一行 name=survey）。边界上反复写删（S={1}，号 143 写了删 100 轮，逼层级 0 与 1 来回）：
`research/prompts/m3-multi-file-unlink-r1-sonnet-model/out/run.out:9`：「name=boundary_churn rule=A rounds=100 rewritten_by_the_writes=200 rewritten_by_the_deletes=0 final_root_level=0」、
`research/prompts/m3-multi-file-unlink-r1-sonnet-model/out/run.out:10`：「name=boundary_churn rule=B rounds=100 rewritten_by_the_writes=200 rewritten_by_the_deletes=100 final_root_level=1」。

读法：四格里 A 重写的节点数都不多于 B（删时少 5%–27%，写时少 1%–12%）；B 的状态层级更高、路径更长，边界上反复写删时每次删 B 要重写根，A 把层级 0 那片叶直接照抄成根。
B 下同一个 S 有两种形状：表里最后一列（小几何 206 个 S、产品几何 25 个 S 出现两种）；一个具体例子，S={1} 在 B 下先写过大号再删掉就是三层脊线，`research/prompts/m3-multi-file-unlink-r1-sonnet-model/out/run.out:18`：「second_order_after_first_delete=(((0, 0), (1, 0), (2, 0)), 2)」，A 下同一个 S 是一个节点，`research/prompts/m3-multi-file-unlink-r1-sonnet-model/out/run.out:19`：「second_order_after_first_delete=(((0, 0),), 0)」。
`name=survey_problems … kinds=none` 四行（`:2`、`:4`、`:6`、`:8`）：两条规则的「按路径定照抄 / 重写」与「按内容定最小重写集」在每一步都相同，没有照抄了内容变了的节点，也没有多重写。
校验路径会红：三个变异（留空叶、只重写叶不重写祖先、降层时留着旧根）各自判红，`out/run.out` 第 28、29、30 行，对照第 31 行（不弄坏时不红）。
这个模型不读 `crates/`，是另写的；它与 `crates/` 的对应是我读 transaction.rs 第 6426 行起那一段对出来的（A 就是那一段去掉 `.max(上一版)` 之后），没有编译比过。

### 4.4　问：写与删两边形状规则不同，判据 ①b 与「逻辑内容」会不会出事

不会出事，两条线上都不会：
- ①b（`.claude/kb/decisions/08-核心索引结构.md:487`：「①b 同一份崩溃镜像各自重挂载续做同一批，两份盘逐字节相同」）比的是同一份镜像续做同一批，两边跑的是同一个确定性的计划函数，输入是同一个「上一版」，A、B 都逐字节相同；
- 「逻辑内容」（同一行：「「逻辑内容」含 inode 每条（号、对象出生代、size、nlink）、extent 单元序号集与内容校验和、`deleted_inodes` key 集、墓碑记录多重集（含死亡 txg）」）不含上段节点的形状与指针；③比的是同一串操作崩溃重开对不崩溃，同一个计划函数，形状同。
B 真正出问题的地方不在这两条：它是历史相关的（同一个 S 两种形状），池级 checker 与理想模型只看得到 S，判不了、预测不了形状，「重写角色集合钉绝对值」（同一行末句）要把「上一版根层级」当输入。A 能被一条纯函数不变量判（见 4.6），B 不能。

### 4.5　三种终态的形状（模型逐步量出，S = {1, 500}，产品几何，号 500 落在上段叶 3、层级 1）

| 删序 | 删完这一个之后 S | 上段节点位置（层级, 序号） | 根层级 | 这一批重写的上段节点 | 这一批换下的上一版上段节点 | 模型行 |
|---|---|---|---|---|---|---|
| 删 500 | {1} | (0,0) | 0 | 无（上一版的 (0,0) 叶内容没变，照抄成根） | (0,3)、(1,0) | `research/prompts/m3-multi-file-unlink-r1-sonnet-model/out/run.out:15`：「delete 500 -> S=[1] positions=[(0, 0)] level=0 rewritten=[] replaced=[(0, 3), (1, 0)]」 |
| 删 1 | {500} | (0,3)、(1,0) | 1 | (1,0)（孩子少了 (0,0)） | (0,0)、(1,0) | `research/prompts/m3-multi-file-unlink-r1-sonnet-model/out/run.out:12`：「delete 1 -> S=[500] positions=[(0, 3), (1, 0)] level=1 rewritten=[(1, 0)] replaced=[(0, 0), (1, 0)]」 |
| 删 500 之后再删 1（全删空） | ∅ | (0,0) 零条目根兼叶 | 0 | (0,0) | 上一版的 (0,0) | `research/prompts/m3-multi-file-unlink-r1-sonnet-model/out/run.out:16`：「delete 1 -> S=[] positions=[(0, 0)] level=0 rewritten=[(0, 0)] replaced=[(0, 0)]」 |
| 删 1 之后再删 500（全删空） | ∅ | (0,0) 零条目根兼叶 | 0 | (0,0) | (0,3)、(1,0) | `research/prompts/m3-multi-file-unlink-r1-sonnet-model/out/run.out:13`：「delete 500 -> S=[] positions=[(0, 0)] level=0 rewritten=[(0, 0)] replaced=[(0, 3), (1, 0)]」 |

只剩别的一个（S={500}）：叶 (0,0) 因为没有条目被摘掉，不留空叶；(1,0) 是根、只有一个孩子 (0,3)。零条目叶只在 S 空时出现（读者拒别处的空节点，4.2）。
降层时旧根下面的节点怎么处置：新根（上一版里降后层级的第 0 个节点）的整棵子树照抄；被删的号那一串（它的叶与叶上面直到旧根的祖先）因为里面再没有别的号，随旧根与脊线上方的节点一起换下（第一行：(0,3) 与 (1,0) 换下，(0,0) 成了根）；别的兄弟分支按 S 本来就没有节点。根层级降不降：降到罩得住 max(S) 的最低一层。

两种删序的终态位置集合相同，节点的出生号不同（`research/prompts/m3-multi-file-unlink-r1-sonnet-model/out/run.out:17`：「name=order_pair rule=A final_positions_equal=True final_birth_ids_differ=True」）。
所以「次序不同的两段历史终态是否逐字节相同」：上段位置集合相同，字节不同（出生号、死亡 txg、改动计数都不同），也不需要相同：①b 与 ③ 比的是同一串操作的两次走法，不比两串不同的操作（U3 再答）。

### 4.6　推荐的条款一句 + 要改的地方

条款一句（加在 `.claude/kb/decisions/08-核心索引结构.md` 第 433 行那一条「上段叶罩 143 个 inode」之后；D8 已定项 16 加第 8 条指过去）：

> extent 树上段的形状只由「有条目的 inode 集合 S」定：S = 现行那一版里至少有一个数据单元的 inode 号（第一个文件与别的文件同等对待；标签 0 写者不写，所以不进 S）；节点 = S 里每个号从它那片叶到根那一串的并，根层级 = 罩得住 max(S) 的最低一层，S 为空时是层级 0 第 0 格一个零条目的根兼叶；非根节点不许零条目。写与删共用这一条：一次发布里某个号进了 S、出了 S 或条目内容变了，它那一串上留在新形状里的节点与上一版没有的节点重写，别的照抄，上一版有而新形状里没有的节点换下；层级长高那次旧根照抄成根之下的节点、新根重写，层级降低那次，上一版里降后层级的第 0 个节点照抄成根、旧根与脊线上方摘掉的节点换下。

| # | 位置 | 改法一句 | 估行（推的） |
|---|---|---|---|
| 1 | `crates/singlefs-core/src/extent_tree.rs`（新函数，放在 `ExtentTreeVersion` 旁） | 纯函数：入参上一版上段位置与根层级、这次之后的 S、条目内容变了的号；交回新位置与来历（照抄 / 重写）与换下的上一版位置。上面模型的 `Planner.step` 就是它的样子 | +70，测试 +100 |
| 2 | `crates/singlefs-core/src/transaction.rs` 第 6171 行（写第一个文件）、第 6426 行（写别的文件） | 两处都改调 #1；去掉两处各自的 `.max(上一版)` 与「并上一版全部节点」 | −95 / +25 |
| 3 | `crates/singlefs-core/src/unlink.rs` 第 234 行（第一个文件最后一批）、第 157 行起的批里上段那一半 | 最后一批对任一号调 #1（删这个号出 S）；批的上段那一串仍重写这个号的串，层级不变（S 不变） | −42 / +15 |
| 4 | `crates/singlefs-checker/src/walk.rs`（新不变量，放在 `walk_extent_upper_node` 走完上段之后） + `crates/singlefs-checker/invariants/09-inode树结构.md` 加一行 | 走读时收两样：各叶里的 inode 号集合 S、各节点位置；核位置集合等于 S 定的形状、根层级最低。必红世界：多出一个脊线节点、根层级高一层、少一个祖先 | +90，不变量行 +1，必红镜像 +60 |
| 5 | GPU 判器 `crates/singlefs-checker-tier/src/crash_judge_checker*.wgsl` | 不做，进覆盖表、内核不置位、记欠账（先例：`crates/singlefs-checker/invariants/01-块自描述.md:18`：「GPU 判器的覆盖表收了这一条（下标 92）、内核不置位」）；要不要这一批就补是交用户的岔路 3 | 0（欠账） |
| 6 | `crates/mutations.tsv` | 新增变异：留空叶、降层时留旧根、重写集合漏祖先、`.max(上一版)` 放回去（4 条）；现有变异表没有行锚着这几处字面（`grep -cF -- ".max(previous_upper_root_level)" crates/mutations.tsv` 等七串都是 0 行，我查了 `.max(previous_upper_root_level)`、`path_of_the_written_file`、`upper_root_level_for`、`.union(&previous_upper)` 与三个计划函数名），搬动不会让锚点失效 | +4 行 |

**什么现象会推翻 A**：(a) 一条真实操作序列使 A 的重写节点数多于 B（模型 40 个种子没有，但模型的操作分布是我选的；换分布再量）；(b) 降层时「照抄的降后层级第 0 个节点换成根的角色」在 core 里走不通——我只读到了对称方向（长高时旧根照抄、换成根之下的角色）：`crates/singlefs-core/src/transaction.rs:7567`：「// 照抄的取上一版那个角色的 key（上段的根层级这次没变时角色名照旧；变了的只有这次重写的那一串）。」与 `crates/singlefs-core/src/transaction.rs:7605`：「previous_upper_root_level.unwrap_or(upper_root_level),」，没有编译试降层方向。若走不通，改成降层时把新根也重写（每次降层多一个节点），其余不变。

## 5　U3　第一个文件之外还有文件时删第一个文件

### 5.1　最后一批改成什么

今天的做法（`crates/singlefs-core/src/transaction.rs` 第 6997 至 7003 行的那一支）：`crates/singlefs-core/src/transaction.rs:6998`：「if carried_data_units == 0 && data_units_before > 0 =>」，
用「第一个文件有数据单元、这一步之后一个不剩」去猜「删的是第一个文件」，调 `plan_the_extent_tree_after_the_final_batch_of_the_first_file`，那个函数断言上段只有根兼叶（`crates/singlefs-core/src/unlink.rs:240`：「"第一版只有第一个文件在 extent 树里有条目：上段只有根兼叶一个节点"」）。别的文件落在第 0 片叶之外（上段不止一个节点）时这个断言会 panic；都落在第 0 片叶里时断言放过，上段叶按内存里别的文件的条目重装（`crates/singlefs-core/src/transaction.rs:11235`：「upper_leaf_entries_of_the_files_other_than_the_first」），碰巧对。两种都被 `crates/singlefs-core/src/transaction.rs:9209`：「refuse_an_unlink_the_deletion_steps_do_not_cover(previous, inode)?;」挡在意图那次之前。

应该是什么：最后一批对任一号 N 同一句（U1 3.4 第 3 处）。删 inode 1 时：
- inode 树：删 1 的记录（`crates/singlefs-core/src/inode_tree.rs:354`：「pub fn remove_record_from_leaf_containers(」按号删，本来就不分第一个文件）；
- 数据：第一个文件的单元全部换下（`crates/singlefs-core/src/transaction.rs:4986`：「DeletionStepOfThePublish::FinalBatch { inode } if inode.0 == FIRST_INODE_NUMBER => 0,」）；
- extent 树：第一个文件的下段整段换下；上段按 U2 的统一规则（S 去掉 1）；别的文件的下段与数据单元一个字节不动、照抄；
- `deleted_inodes` 根删、墓碑在意图那次已写。
`TransactionOutput.inode_record` 变 `None`（`crates/singlefs-core/src/transaction.rs:3127`：「（D8（核心索引结构） 已定项 16 第 5 条：删完还剩别的 inode 的那一版仍是带文件的一版，只是没有第一个文件那条记录、没有数据单元、」那段文档已经这么写），
装版本时那条断言成立，因为这一步 `data_pointers` 为空：`crates/singlefs-core/src/transaction.rs:12421`：「inode_record.is_some() || data_pointers.is_empty(),」。
改法：触发条件改成按 `FinalBatch { inode }` 里的号分（transaction.rs 第 6997 行那一支），不再用 `data_units_before > 0` 猜。
还有一处同款的坑，对任一 N ≠ 1 的最后一批都中：第一个文件有单元时 `carried_data_units`（= `data_units_before`，`data_units_after_this_publish` 对别的号的 `FinalBatch` 回原数，transaction.rs 第 4987 至 4991 行）大于 0，第一个文件已删时 `data_units_before` 是 0，两种今天的条件都不成立，落进默认的整棵照抄那一支（`crates/singlefs-core/src/transaction.rs:7014`：「) => plan_the_extent_tree_after_this_publish(」）。
结果是 N 的下段节点与上段条目留在新一版里，而 N 的单元没人点名换下（`previous_roles_without_a_successor` 只认第一个文件的 `Data` 角色，`crates/singlefs-core/src/transaction.rs:6776`：「(self.carried_data_units..previous.data_pointers.len())」）——孤儿上段条目加泄漏的单元。只删拒绝、不改这几处，就是静默留下它们（推的：读代码，没编译；I-4.7 第二类对象「extent 树里有数据单元、而 inode 树里没有 nlink ≥ 1 记录的号」会红）。

### 5.2　删完之后这一版的成员

答：`PoolVersion::WithFile` / `PreviousVersion::WithFile` / `RebuiltVersion::WithFile`，**不要第四个成员，也不改第三个成员的判定**。依据：
- 第三个成员的判定逐字：`.claude/kb/decisions/08-核心索引结构.md:485`：「判定 = 树表有条目 ∧ inode 树零条目根（key 区间 `[0, 0]`，照已定项 6）∧ extent 树零条目根（key 区间按位置规定）∧ `deleted_inodes` 根指针置零；不看墓碑树、谱系树与退役树。」；
  代码 `crates/singlefs-core/src/transaction.rs:3468`：「let inode_tree_root_has_no_entry = self.inode_leaf_containers.is_empty();」。别的 inode 还有一条记录（含 size 0 的），第一合取就不成立，归带文件的一版（`crates/singlefs-core/src/transaction.rs:2297`：「if version.is_a_version_with_trees_and_no_file() {」否则 `WithFile`）。
- 这不是新情形：今天删完第一个文件再建 inode（size 0）就是这样，理想模型已经这么写：`crates/singlefs-harness/src/model.rs:982`：「/// 实现把它归带文件的一版（inode 树有条目，D8（核心索引结构） 已定项 16 第 5 条的判定不成立）。」
- 带 `None` 的 `inode_record` 的各处消费都是 `Option`（`grep -n "inode_record" crates/singlefs-core/src/*.rs` 里 `expect` / `unwrap` 只出现在 `recovery.rs` 对已取出的记录上，不在 `Option` 字段上；我看了全部命中）；第一个文件的写入口拒成 `WriteToAnInodeThatIsNotInTheTree`（`crates/singlefs-core/src/transaction.rs:9749`：「let Some(record) = previous.inode_record else {」）、截断拒成 `InodeIsNotALiveRecord`（`crates/singlefs-core/src/transaction.rs:9369`：「TruncationRefusal::InodeIsNotALiveRecord { inode: inode.0 },」）。

条款要补的一句（D8 已定项 16 第 5 条末尾）：「inode 树里还有任一条记录（含 size 0 的）或 extent 树里还有条目的一版是带文件的一版，不管 inode 1 在不在；inode 1 的记录不在时对 inode 1 的覆盖写、顺序写、截断拒成已有的那几种拒绝；第三个成员只指文件集合真的空。」

### 5.3　「删完第一个文件」指哪一版；K-U5 的「拦」管不管这一版上原有的别的文件

「删完第一个文件」= 第一个让 inode 1 的记录不在 inode 树里的那一版 = 删 inode 1 的最后一批所发的那一版。版本内容上可判：`previous.inode_record_of(InodeNumber(1)).is_none()`（第一个文件版本总是建 inode 1：`.claude/kb/decisions/08-核心索引结构.md:201`：「第一版的唯一文件取 inode 1（0 无效），nlink = 1，不进孤儿回收。」；所以「有树、树里没有 1 的记录」⇒ 它被删过，这一步是推的，推翻它的现象：有不经第一个文件版本建出来的池）。
「拦」写在哪：补丁树 `plan_of_a_whole_write_of_a_file_other_than_the_first`，在 `crates/singlefs-core/src/transaction.rs:8341`：「return Err(PublishError::WriteToAnInodeBeingDeleted { inode: inode.0 });」之后加一判；今天的两个入口都经过它，且没有这一判（见 2 节 X2）。

背景材料第一节转述的用户 2026-10-19 弹窗定案是：删完第一个文件、再建 inode 时，新入口拒写数据；其中加引号的用户原话只有「拦，K-U5 照旧欠着」。没说这一版上**原有的**别的文件。三种读法的拒绝集合：

| 目标 inode（第一个文件已删） | α 版本级：inode 1 不在就拒一切按 inode 号写数据 | γ 目标级：目标现在没有数据单元就拒 | β 时序级：目标的对象出生代晚于 inode 1 的死亡 txg 就拒 |
|---|---|---|---|
| 原有的、带数据的别的文件，整份改写 | 拒 | 允许 | 允许 |
| 原有的、size 0 的（删之前建的），第一次写数据 | 拒 | 拒 | 允许 |
| 删完之后建的（size 0），第一次写数据 | 拒 | 拒 | 拒 |
| 判得出的输入 | 版本里有没有 inode 1 的记录 | 版本里 extent 树有没有这个号的条目 | 墓碑树里 inode 1 的那条记录的死亡 txg 与目标记录偏移 8 的对象出生代 |
| 实现估行（推的） | +10（含错误成员），理想模型 +15 | +12，模型 +25 | +45（读墓碑树），模型 +60 |
| 靠什么前提 | 无 | 无 | 墓碑第一版不回收（`.claude/kb/decisions/18-块里携带什么信息.md:579`：「**回收**：第一版不回收墓碑」），回收一落地它就失效 |

三种读法在「删完之后建的 inode」这一行都拒——用户已经定下的那一格。分歧只在前两行（原有的别的文件）。这是岔路 1，不带倾向，写进末节。

### 5.4　删完第一个文件之后，还活着的别的文件能不能删、能不能写

- 删：能。U1 的步骤不依赖第一个文件在不在（意图与各批按号取记录；队首号来自 `deleted_inodes`）。
- 写（按 inode 号覆盖写 / 顺序写）：由岔路 1 定。
- 建 inode（size 0）：能，条款原文（`.claude/kb/decisions/08-核心索引结构.md:485`：「删完之后只能做 `publish_new_inodes`（size 0）、写行与空发布（K-U5 X1）」）就放行。
- 写 inode 1：拒（`WriteToAnInodeThatIsNotInTheTree`，已有）。

### 5.5　删序不同的两段历史，终态是否逐字节相同

不相同，也不需要相同。S = {1, 500} 删完：先删 1 后删 500 与先删 500 后删 1，终态上段位置集合相同（模型 `research/prompts/m3-multi-file-unlink-r1-sonnet-model/out/run.out:17`：「final_positions_equal=True final_birth_ids_differ=True」），节点出生号不同；
`.claude/kb/decisions/08-核心索引结构.md` 第 487 行的「逻辑内容」含「墓碑记录多重集（含死亡 txg）」：两个墓碑的死亡 txg 对调，两段历史的逻辑内容本来就不同，而判据 ①b、③ 比的都是同一串操作的两种走法。要补的一句话（第 7 条末尾）：「不同操作序列的终态不要求逻辑内容相同；要求的是每个终态上上段形状只由 S 定（第 8 条）、每个崩溃态 checker 全绿」。
**什么现象会推翻 5.2–5.5**：(a) 会话或挂载里有一处把 `WithFile` 当成「inode 1 一定在」而不走 `Option`（我 grep 到的消费处都走 `Option`，但没编译、没跑会话用例）；(b) 用户要的「拦」不是版本内容能判的（例如要按「建 inode 的次序」判），那就要新记一个持久量。

### 5.6　U3 的 `crates/` 清单（与 U1、U2 重叠的不重复计）

| # | 位置 | 改法一句 | 估行（推的） |
|---|---|---|---|
| 1 | `crates/singlefs-core/src/transaction.rs` 第 6997 行那一支 | 条件按 `FinalBatch { inode }` 的号分；1 与别的号都调 U2 的统一上段计划 | 含在 U1 #3 |
| 2 | `crates/singlefs-core/src/transaction.rs` 第 8341 行之后 | 「拦」一判（岔路 1 选定后写）；`PublishError` 新增一个成员 | +12，视选定 |
| 3 | `crates/singlefs-checker-tier/src/bin/e181_landing_check_fixed_point_and_mount_lower_bound.rs` 第 821 行附近的穷举 `match`（该行 `PublishError::WriteToAnInodeThatIsNotInTheTree { .. }` 与相邻成员并列） | 新成员加一行（checker 档文件） | +1 |
| 4 | `crates/singlefs-core/src/mounted_session.rs` | 无改动：K-U5 的拒绝经 `UserChangeRefused::Publish` 原样上交，不推抬 F（不是空间不够） | 0 |

## 6　U4　续做、队列、墓碑、快照与 checker

### 6.1　逐项：多文件时还成立吗，要改哪一句

| 项 | 现句 / 判法 | 多文件 | 要改哪一句 |
|---|---|---|---|
| `deleted_inodes` 至多一条 key | `.claude/kb/decisions/08-核心索引结构.md:486`：「任何一版里 `deleted_inodes` 至多一条 key；多于一个根装得下那一支拒成类型化错误、不是空间不够。」；同行：「会话里每一次用户改动（含 unlink、写、建 inode）先把现行那一版的 `deleted_inodes` 队列走空，再做它自己」；checker `crates/singlefs-checker/invariants/09-inode树结构.md:36`：「| I-9.24 | 删除队列至多一条 key |」；代码 `crates/singlefs-core/src/transaction.rs:9199`：「previous.deleted_inodes.is_empty(),」（意图入口的断言） | 成立。两条理由都与「删的是哪个号」无关：意图只在队列走空之后发，一次 unlink 走完全部批才返回；多文件只改变队首是几号 | 一字不改 |
| 挂载续做 | 同一行：「挂载续做照旧；unlink 一次调用 = 走空 + 意图 + 全部批（与续做共用同一个走空）」；预演 `crates/singlefs-core/src/transaction.rs:9562`：「let Some(inode) = version_rehearsed.deleted_inodes.first().copied() else {」 | 成立。续做与预演都取队首号交给 `next_unlink_step_after`，号无关；它认得 N ≠ 1 之后（U1 #1、#2）续做自动认得 | 条款一字不改；崩溃注入的 `WritableMountPhase::Continuation` 照旧 |
| 墓碑每个对象一条 | `.claude/kb/decisions/18-块里携带什么信息.md:577`：「整对象写 [0, 2⁶⁴ − 1]」、同行「死亡 txg = 意图那次发布的 checkpoint_txg」 | 成立。N 的对象出生代取 N 自己那条记录偏移 8（`crates/singlefs-core/src/transaction.rs:9225`：「tombstone: TombstoneRecord::of_the_whole_object(inode, record.object_birth, txg),」），建 inode 时 = 创建发布的 txg（`crates/singlefs-core/src/transaction.rs:8196`：「object_birth: txg,」），按 inode 号整份改写不改它（`crates/singlefs-core/src/transaction.rs:8274`：「对象出生代照它那条记录、不改」）；已发布的 (号, 出生代) 不复用（D8 已定项 6），N 的墓碑与别的号的互不相碰 | 一字不改。D18 已定项 24 第 4 条「第一版不回收墓碑」（`.claude/kb/decisions/18-块里携带什么信息.md:579`）是岔路 1 的 β 读法的前提，别的读法不依赖 |
| 快照下删别的文件 | `.claude/kb/decisions/05-快照-空间记账机制.md:422`：「elif birth > p: 立即释放 → 进 defer 队列（不是直接进空闲表，见 D16（发布语义） 已定项 15）」；`.claude/kb/decisions/05-快照-空间记账机制.md:438`：「待删除结构（意图 key、删除中队列）**算作活引用**。」 | 成立，且代码路径已通：生死判定对非第一个文件的角色照问，`crates/singlefs-core/src/transaction.rs:5514`：「TransactionUnit::DataOfAFileOtherThanTheFirst { .. }」与下一行的下段节点角色都在 `asks_the_snapshot_lifetime_judgement` 的 `true` 一支里；上一版里有没有落点也认（`crates/singlefs-core/src/transaction.rs:3965`：「TransactionUnit::DataOfAFileOtherThanTheFirst { inode, unit } => previous」）。批在意图之后、建快照之前或之后，钉不钉住由批那一刻的 `birth > p` 判，不由「文件是几号」定 | 一字不改；D5 已定项 20 的「rm 返回之后建的快照会钉住那批尚未释放的 extent」对 N 同样成立，不用另写 |
| I-4.7 删除中 inode 有账 | `crates/singlefs-checker/invariants/06-事务与崩溃一致性.md:13`：「| I-4.7 | 删除中 inode 有账 |」；判法 `crates/singlefs-checker/src/walk.rs:11463`：「fn judge_inodes_being_deleted_against_the_queue_and_their_data_units(」按 `records`、`data_units_of_each_inode` 两张按号的表循环，没有第一个文件的特判 | 不用改。第二类对象（extent 树里有单元、inode 树里没有 nlink ≥ 1 的记录的号）正好抓「只删拒绝、不改 5.1 那个条件」留下的孤儿上段条目 | 一字不改 |
| I-9.5 / I-3.4 / I-4.10 / I-4.6 / I-9.24 / I-9.18 | 同上，都按号循环；I-4.6 现句 `crates/singlefs-checker/invariants/06-事务与崩溃一致性.md:12`：「标签 0 与没有上段叶条目的号一个单元都没有」，最后一批去掉条目（3.2）与它相容 | 不用改 | 一字不改 |
| 扫描重建 I-1.13 | `crates/singlefs-checker/src/rebuild.rs:769`：「if tombstoned_objects.contains(&(inode, object_birth)) {」按 (号, 出生代) 判，记录还在报「删除中」不适用、记录没了数据单元判死 | 对 N 同款；最后一批之后 N 的单元已换下，墓碑压着、记录没了，判死，不多不少 | 一字不改 |
| 池级 checker 新增 | —— | 一条：上段形状 = 由有条目 inode 集合定（U2 4.6 #4）。这是唯一要新写的判法 | 新增一行 I-9.x，状态「未实现」随实现改 |
| GPU 判器 | 删除态扫描内核按号通用：`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:163`：「if (d_data_unit_inode_count >= D_MAX_DATA_UNIT_INODES) {」，容量 `crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:1015`：「const D_MAX_DATA_UNIT_INODES: u32 = 1024u;」 | 删除态扫描那几条不用改（容量 1024 个号，多文件流的号数没数过，标未核）。**恢复内核不通用**：`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3171`：「if (upper_count != 1u) {」、`:3174` 要求唯一那条是 inode 1。多文件的镜像上它判 `UnitMalformed`，CPU 恢复判过 | 要改：这是多文件写（实现员的补丁）就带来的缺口，不是删除新增的；删除流进崩溃放量之前必须解决或明确记欠账（岔路 3） |

### 6.2　幂等五条判据 ①–④ 在多文件历史上要加的取样点

判据现句（`.claude/kb/decisions/08-核心索引结构.md` 第 487 行，①–④ 与「逻辑内容」口径）一字不改，只是它的取样点从「第一个文件」扩到下面这些（每个都得在一条崩溃流或一个 harness 用例里出现一次）：

| 判据 | 新取样点 |
|---|---|
| ① 计划相等 | 对 N ≠ 1 的 `next_unlink_step`（纯函数，与号无关，`unlink::tests` 里已有的第一个文件用例照搬一份，批大小压到 1、2、144、145 各一） |
| ①b 同一份崩溃镜像各自续做逐字节相同 | (i) 崩在 N 的意图之后、第一批之后、最后一批之前；(ii) 崩在删第一个文件的最后一批之前（上段 S 去掉 1，此时可能降层）；(iii) 删最后一个带数据的号的最后一批之前（S 变空，零条目根） |
| ② 删完再续做一个写都不发 | N 的最后一批之后、第一个文件的最后一批之后（别的文件还在）、全删空之后 |
| ③ 每个写点截断、重开续做到底，终态逻辑内容与不崩溃的终态相同、每个崩溃态 checker 全绿 | 上面三处的每个写点；加上「上段降层那一次发布」的每个写点（kb 已经点名的第三种新写序：`.claude/kb/decisions/08-核心索引结构.md:445`：「三种新写序（长高多层时位置 0 那一串节点同一次发布写、矮下去多层时同一串一起释放、内联与下段互换）要进崩溃点重放」）。现有崩溃流没有一条走到上段降层，且 `crates/singlefs-checker-tier/tests/crash_enumeration_position_addressed_trees.rs:260`：「"{stream:?}：上段只有 inode 1 所在那片根兼叶"」断言上段只有一个根兼叶——多文件流不能复用这条断言 |
| ④ 标签 4、6 | 同 ③ 的每个被判状态；多文件时已分配里要含 N 没释放的单元（I-3.4 的现句已覆盖，不改） |
| 逻辑内容口径 | 「extent 单元序号集与内容校验和」「读者的答案」从「第一个文件」读成「每个号」；不含的那一串（实例号、根与单元头里的 txg、改动计数、落点……）不变，再加「上段节点的出生号与指针」（本来就在「不含」的精神里，现句没点名，补上） |

**什么现象会推翻 U4「一字不改」**：(a) 在补丁树上对 N ≠ 1 跑一次 unlink 全程、再跑池级 checker，I-9.5 / I-4.7 / I-4.6 任一条红（我没编译跑，这是读代码推的）；(b) 5.1 说的「默认整棵照抄」那一支被修了之后，仍有一条按号循环的判法在第一个文件已删的版本上把 `records.get(&1)` 当必有——我在 `walk.rs` 里 grep 了 `FIRST_INODE` / `first_file`，只有注释，没有特判；(c) GPU 删除态扫描的 1024 个号的容量被多文件流撞满（`UNDECIDABLE_TOO_MANY_ENTRIES`）。

## 7　U5　每处要改的地方与估的行数

表里「行」一律是推的（读代码估的，没编译、没改过）；「量的」只有第 4 节模型那几个数。核心层与 U1–U3 三张表是同一批改动，这里按层汇总，不重复列行号。

| 层 | 改法一句 | 估行（推的） |
|---|---|---|
| 核心层 `crates/singlefs-core/src/extent_tree.rs` | 新增上段统一计划的纯函数（4.6 #1），含单测 | +70，测试 +100 |
| 核心层 `crates/singlefs-core/src/unlink.rs` | 批的下段那一半抽成与号无关的函数（保留变异表锚着的两处字面）；删「第一个文件的最后一批」专用函数；加非第一个文件的批 / 最后一批单测 | +30 / −42，测试 +120 |
| 核心层 `crates/singlefs-core/src/transaction.rs` | U1 #1–#8、U2 #2、U3 #1–#2 共那些处：步骤带号、计划四种来历、下段节点逐个带来历、被截 / 被摘文件的数据与角色、两处写路径改调统一函数、去拒绝、加「拦」 | 非测试 +260 / −180 |
| 核心层 `crates/singlefs-core/src/deleted_inodes.rs` | 删枚举成员 `UnlinkWhileAFileOtherThanTheFirstHoldsDataUnitsIsNotDecidedByTheClauses` 与文档里那一段 | −12 |
| 核心层 `recovery.rs`、`mount.rs`（续做）、`mounted_session.rs` | 无改动：删除中间态的每一版过 `crates/singlefs-core/src/recovery.rs:5228`：「if data_pointers_of_the_file.is_empty()」那一判（批留至少一个单元，最后一批之后没有这个号的条目）；续做取队首号 | 0 |
| harness 测试 | `crates/singlefs-harness/tests/writes_by_inode_number_of_several_files.rs`（523 行）里拒绝那几个用例改成成功用例；新增一个文件放 N ≠ 1 的意图 / 批 / 最后一批、删第一个文件而别的带数据、全删空、两种删序、快照下删 N、K-U5 的拦（按岔路 1 选定的读法）的用例；`crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs`（1258 行）加按号的坏镜像构造 | +450（测试） |
| 池级 checker `crates/singlefs-checker/src/walk.rs` + `crates/singlefs-checker/invariants/09-inode树结构.md` | 新增「上段形状由有条目 inode 集合定」一条（4.6 #4）；别的不变量一句不改（6.1）；必红镜像：多出脊线节点、根层级高一层、最后一批留着已删号的条目（会同时红 I-4.7） | +90，不变量 +1 行，必红镜像 +120 |
| GPU 判器（若有新判） | 删除态扫描内核按号通用，不用改（容量 1024 个号，未核）；新不变量进覆盖表、内核不置位；恢复内核 `crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3171`：「if (upper_count != 1u) {」起那一段要改成按条目收（岔路 3） | 覆盖表 +1；恢复内核若本批补 +60 / −15 |
| 理想模型 `crates/singlefs-harness/src/model.rs` | 今天只罩 inode 1（实现员报告第 2 项没做）。删除部分：`ModelVersionContents::FileBeingDeleted`（`crates/singlefs-harness/src/model.rs:974`：「FileBeingDeleted {」）带号与「每个号剩几个单元」，`answer_unlink_intent`（`crates/singlefs-harness/src/model.rs:4270`：「pub fn answer_unlink_intent(&self) -> Result<ModelAnswer, ModelDisagreement> {」）与 `answer_deletion_step`（`crates/singlefs-harness/src/model.rs:4311`：「pub fn answer_deletion_step(&self) -> Result<ModelAnswer, ModelDisagreement> {」）按号；上段重写角色数用规则 A 的纯函数（`research/prompts/m3-multi-file-unlink-r1-sonnet-model/upper_shape_model.py` 的 `Planner.step` 就是它的样子） | +300（含形状 +60），依赖「模型罩多 inode 写」那一项先做 |
| RefFS `crates/singlefs-reference-filesystem` 与翻译 | 无改动：翻译到 unlink 写了盘那一步就停，`crates/singlefs-harness/src/reference_filesystem_translation.rs:473`：「HistoryOperation::Unlink => match &seen.ending {」，多文件 unlink 一样停 | 0 |
| 历史生成 `crates/singlefs-harness/src/history.rs` | `HistoryOperation::Unlink`（`crates/singlefs-harness/src/history.rs:707`：「Unlink,」）带号，`apply_unlink`（`crates/singlefs-harness/src/history.rs:7006`：「HistoryOperation::Unlink => apply_unlink(pool, write_time_seconds),」）选号，权重表按「有几个活号」取；快照下删、回退到意图那一版要能抽到 N ≠ 1 | +80 |
| 崩溃流（checker 档，写不跑） | 新流 1：mkfs → 第一个文件（3 单元）→ 建 143 个 inode → 写 inode 144（上段层级 0→1）→ 压小批大小 unlink 144（降层）→ unlink 1（S 变空）；新流 2：同前缀，先 unlink 1（别的文件还在，S 去掉 1）再 unlink 144。参照 `crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs`（1375 行）的公共前缀写法 | 每条 +600（推的），总 +1200 |
| 变异与必红世界 `crates/mutations.tsv` | 新增约 11 行：批用第一个文件的 kept 数截别的号（U1 的坑）、最后一批对 N 不去条目、最后一批写标签 0、降层留旧根、留空叶、放回 `.max(上一版)`、被截文件数据指针不截、被摘文件下段节点不换下、续做取 inode 1 不取队首、「拦」判反、删第一个文件的条件仍用 `data_units_before > 0` | +11 行 |

合计（推的）：核心非测试净 +130 左右（70 −12 +80 −12，逐行相加），测试 +670，checker +210，模型与历史 +380，崩溃流 +1200。最大的两块是崩溃流与模型，都不在 `crates/singlefs-core`。
数不出来的：新流的崩溃点数与单节点重放秒数（要先有流才能数；`crates/singlefs-checker-tier/tests/` 里现有的 unlink 流各自的 `exhaustive=` 行我没读，也没跑 `--list-items`）。
**什么现象会推翻这些行数**：实现员打开 `transaction.rs` 发现 `build_extent_tree` 的「来历」不好逐节点拆（我读的是 transaction.rs 第 11108 行起的下段循环与第 7586 行起的映射 key 循环，两处都按文件整体的来历分支）；或者把被截文件当「整份重写」做（重写 144 个单元之外的下段，违反 K-R3-1 只重写尾叶，不可取）。

## 8　交用户的岔路（候选、翻面观测、够判条件；不带倾向）

| # | 候选 | 翻面观测 | 够判条件 |
|---|---|---|---|
| 1　K-U5「拦」在第一个文件已删的版本上管到哪（5.3） | α 版本级：版本里没有 inode 1 的记录，就拒一切按 inode 号写数据。γ 目标级：同上前提，且目标 inode 现在没有数据单元才拒。β 时序级：目标的对象出生代晚于 inode 1 的死亡 txg 才拒 | 没有可量的量，这是产品语义。能量的只有实现代价（α +10、γ +12、β +45 行，推的）和 β 的前提：墓碑第一版不回收（`.claude/kb/decisions/18-块里携带什么信息.md:579`：「**回收**：第一版不回收墓碑」），墓碑开始回收时 β 失效，换 α 或 γ | 用户答一句「删完第一个文件之后，原有的别的文件还要不要能改写；删之前建的 size 0 的 inode 要不要能第一次写数据」。三个候选的拒绝集合（5.3 表）与各自的判法输入已摆全；三者在「删完之后建的 inode」那一格都拒，这一格用户 2026-10-19 已定 |
| 2　新的上段形状规则（4.6）落地到哪一层 | (i) 只进计划函数、单测、理想模型。(ii) 再加池级 checker 一条新不变量。(iii) 再加 core 读者拒非规范形状 | 若有一条合法写者路径（回退到旧根、整理搬迁、截断）会发出非规范形状，(iii) 就不成立：模型覆盖了写、删两种改 S 的操作（`out/run.out` 第 1–8 行，`violations=0`），回退与搬迁没覆盖；读代码看，截断不改 S（`crates/singlefs-core/src/truncate.rs:241`：「(1..=data_units_before).contains(&data_units_after),」）、搬迁只改落点 | 把回退、整理搬迁两种发布也过一遍模型（各加一个操作，模型 +40 行），结果 `violations=0` 即够判 (iii) 能不能上；要是出红，(ii)、(iii) 都要先收窄规则 |
| 3　GPU 恢复内核对多文件镜像 | (i) 本批补：`crates/singlefs-checker-tier/src/crash_judge_common.wgsl` 第 3171 行起按上段条目收，加对拍。(ii) 记欠账：多文件删除流只由 CPU 判器判，GPU 对拍这几条不比（先例：`crates/singlefs-checker/invariants/01-块自描述.md:18`：「GPU 判器的覆盖表收了这一条（下标 92）、内核不置位」） | 一份含两条以上上段叶条目的镜像上，GPU 恢复判与 CPU 恢复判不一致的格数。读代码推是每份都不一致（GPU 判 `UnitMalformed`，`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3174`：「if (!eq64(extent_key_inode(sw(R_UPPER), sw(R_UPPER + 1u)), u64_of(FIRST_INODE_NUMBER))) {」），没跑 | 在一份由 `publish_sequential_write_of_an_inode` 写出的两文件镜像上各跑一次 GPU 与 CPU 恢复判，数不一致的格数（checker 档，要主 agent 跑）。数出来是 0 则这一条岔路消失 |
| 4　多文件 unlink 崩溃流挂在哪（7 节） | (i) 并进「里程碑删除与多次 COW」一·1 的第二条流（多个 inode 交错覆盖写，`.claude/kb/milestone/03-third-txn.md` 第 53 行）后面加 unlink 段。(ii) 另开一条，挂三·4。(iii) 两处各一条 | 流的崩溃点数与单点重放秒数，我没有数（要先有草稿流） | 写出草稿流、数出崩溃点，与现有 `crash_enumeration_unlink_after_the_multi_record_publish_stream.rs`、`crash_enumeration_unlink_after_the_fixed_script_stream.rs` 两条的点数比；差一个数量级以上时选最小的那条路 |

不交用户、我推荐，标「零轮」（没被攻过；`.claude/rules/three-way-inference.md` 多轮一节：腿提的收严、判决里推的组合都算被攻过零轮）的几处：最后一批去掉条目（3.2）、规则 A（4.3）、成员取 `WithFile`（5.2）、批的批大小 K 对任一号同取 144（`.claude/kb/decisions/08-核心索引结构.md` 第 482 行的「自由选取」不变）、降层时新根照抄（4.6 末，对称方向只读到、没编译，写进「没做什么」）。

## 9　推荐的条款形态（对 `.claude/kb/decisions/08-核心索引结构.md` 已定项 16 与 14；不重抄整条，写位置与增补）

1. 第 1 条：不动。
2. 第 2 条：按 3.4 的三处增补（任一 inode 同一条；剩余单元数的来源；最后一批把 N 的条目从 extent 树上段去掉、不写标签 0）。
3. 第 5 条：保留判定与 `WithTreesAndNoFile` 的定义不动，末尾加 5.2 那一句（带文件的一版不管 inode 1 在不在）。K-U5 那一句（`.claude/kb/decisions/08-核心索引结构.md:485`：「删完之后只能做 `publish_new_inodes`（size 0）、写行与空发布（K-U5 X1）」）按用户答岔路 1 的读法改写，写成可判的版本内容句，例如 γ 读法：「第一个文件已删（inode 树里没有 inode 1 的记录）的版本上，按 inode 号写数据的入口拒给还没有数据单元的 inode 写数据（K-U5，C618）」。
   我的推荐是 γ：它最贴用户原话与射程句里的「建带数据的文件」（给没有数据的 inode 首次写数据才算建，改写已有数据的文件不算建），判法只看版本内容、不依赖墓碑，不额外挡原有的带数据文件；α 多挡了一类，β 依赖墓碑不回收。这是推荐，不是替用户定。
4. 第 6 条：不动（`deleted_inodes` 至多一条 key，号无关）。
5. 第 7 条：判据 ①–④ 不动；末尾加 U4 6.2 末行那一句（取样点扩到每个号）与 U3 5.5 那一句（不同操作序列的终态不要求逻辑内容相同，要求上段形状只由 S 定）。
6. 新增第 8 条：4.6 那一句形状规则（S、节点集合、根层级、零条目根、长高 / 降层各一句）；D8 已定项 14 的 extent 一条（`.claude/kb/decisions/08-核心索引结构.md` 第 433 行那条之后）放同一句，两处指同一份文字。
7. 射程句（`.claude/kb/decisions/08-核心索引结构.md` 第 489 行）：「第一版只做单文件 unlink、按 inode 号、无目录、不带 truncate」改成「按 inode 号删任一 inode、无目录」，「不带 truncate」那半句删（里程碑四已有截断，2 节 X3）；K-U5 半句随第 3 点。
8. D18 已定项 24、D5 已定项 19 与 20：不动。
9. 欠账：C618 那一行写明「拦」的读法与它在哪一版成立；新增欠账：GPU 判器一侧（按岔路 3 选定：恢复内核缺口，或新不变量内核不置位）、降层方向角色名走通的编译验证。
10. 池级 checker 不变量清单：`crates/singlefs-checker/invariants/09-inode树结构.md` 加 I-9.x「上段形状由有条目 inode 集合定」，状态「未实现」，随实现改；别的行一句不改（6.1）。

## 10　判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| U1 意图 | 一致 | 条款对任一 N 成立，代码只被 `refuse_an_unlink_…` 挡着；推翻：意图入口还有别处按第一个文件假设 |
| U1 分批 | 规则没说 + 冲突（X4） | 剩余单元数的来源与上段那一串没写；代码的 `Batch` 没有号，只删拒绝会让别的号的批截第一个文件；推翻：别处按 `record.inode` 分了流（grep 只见 6968、6989 两处） |
| U1 最后一批 | 规则没说 | 去掉条目，不写标签 0（kb:431 写者不写、读者与 checker 拒空非根节点）；推翻：读者要靠已删号的条目留痕 |
| U2 形状规则 | 规则没说 + 冲突（X1） | 规则 A（形状只由 S 定）；kb 没有这句，代码文档有、写路径 `.max(上一版)` 与之相抵，删除一进来才显；推翻：真实操作序列上 A 比 B 多写节点，或降层方向 core 走不通 |
| U2 ①b / 逻辑内容 | 一致 | 两种规则下 ①b 与逻辑内容都不出事，B 的代价是历史相关、checker 与模型预测不了 |
| U2 三种终态 | 量的 | 只剩 1：根兼叶照抄；只剩别的一个：脊线 + 一个孩子；全空：零条目根兼叶（`out/run.out` 第 11–16 行） |
| U3 最后一批 | 规则没说 | 对 1 也走统一上段计划，条件按 `FinalBatch` 的号分；推翻：默认整棵照抄那一支另有人依赖 |
| U3 成员 | 一致 | 取 `WithFile`，第三个成员的定义不动；模型已这样归（`crates/singlefs-harness/src/model.rs` 第 982 行）；推翻：有会话处把 `WithFile` 当 inode 1 一定在 |
| U3 K-U5 | 冲突（X2） + 岔路 1 | 补丁树不拦；用户已定拦，读法 α / γ / β 交用户 |
| U3 两种删序 | 量的 | 位置集合相同、字节与逻辑内容本来就不同，不要求相同 |
| U4 | 一致（GPU 恢复内核除外） | 队列、续做、墓碑、快照、checker 各条按号通用，一字不改；要加取样点；GPU 恢复内核对多文件镜像不通用（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl` 第 3171 行），岔路 3；推翻：补丁树上跑 N ≠ 1 全程后池级 checker 红 |
| U5 | 估算（推的） | 核心非测试净 +130，测试 +670，checker +210，模型与历史 +380，崩溃流 +1200；推翻：下段循环的「来历」拆不到节点 |

## 11　没做什么

- 没编译、没跑 cargo、没跑任何重型测试；所有 `crates/` 行数是读代码估的，都标了「推的」。降层方向「照抄的节点换成根的角色」只读到了对称方向（长高）走得通，没有编译试。
- 「一字不改」「不用改」这几处是单次读代码的否定结论（`.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」：没打中的至少再抽一次），没有第二次抽样；归攻方腿与主 agent 现查。
- 模型只管上段位置集合与照抄 / 重写；没模型：角色名与映射 key、inode 树、回退、整理搬迁、快照、下段的内联与建下段互换。下段批规则那一组（`out/run.out` 第 20–27 行）是对现有规则的对拍，741 个小几何点与 7 个产品几何点上路径规则等于内容规则。
- 没读攻方腿的报告与模型；没替主 agent 采纳或判决；没给岔路的倾向（第 9 节第 3 点的推荐单列、已声明）。
- 没量：新崩溃流的崩溃点数与重放秒数；GPU 删除态扫描 1024 个号的容量够不够多文件流；GPU 与 CPU 恢复判在多文件镜像上的实际不一致格数。
- 背景材料附录没读（kb 原文直接读 kb 文件）；kb 引文都是 `grep -nF` 过的整句，过长的几处（例如 `.claude/kb/decisions/08-核心索引结构.md` 第 485、487 行）只引了与结论有关的整句，没有整行。
- 日志里排队等内存包装：复跑经 `scripts/hosts/run-with-memory-cap.sh 2G` 排了 24 秒，第一次 4G 的排了 252 秒（本机别的会话占着 39.3 GiB）；没有性能测量在跑的进程挡我。
