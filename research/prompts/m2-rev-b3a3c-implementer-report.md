# 实审 B3a-3c 报告（implementation-writer）：四份层 0 文件的流重新布置，position_addressed 的打印溢出

开工 2026-09-27。规格 `/tmp/claude-1000/impl-rev-b3a3c/spec.md`。

## 一、结论

- **单元数变多的来源查清了，对得上定案，所以往下做了。**
  - 第一个文件版本那次发布：a1f4691 的代码是 8 个单元（33 写、单元段 18）；HEAD 的代码（HEAD 的 `crates/` 就是 346f5e6 的）与工作区副本都是 12 个（41 写、单元段 26）。三棵树各跑一次同一个探针，原样见第二节。
  - 多出来的 4 个单元都是 `AllocationTreeNodeBelowTheRoot`：两盘各叶 61、两盘各第 1 层 0 号。B3a-3b 说「盘 0、盘 1 各一片叶」，少数了两个第 1 层节点。
  - 改动在 346f5e6：新文件 `crates/singlefs-core/src/allocation_record_tree.rs`（分配记录树按位置寻址），加上 `transaction.rs` 第 1834 行的新成员。不在 HEAD 与工作区之间的差异里：工作区对这份文件只改了一个测试名（第二节）。
  - 对应的定案有三处：D8 已定项 14（第 372 行起，第 396 行「根罩整个 key 空间、按盘分流」）；调度表 `records/2026-09-24-里程碑二收尾调度.md` 第 108、109、125 行；同一份调度表第 140 行 ⑩「第一个事务从 8 个单元变 12 个」，以及 ⑨「已有层 0 流钉着旧布局……红了派实现员重钉」。
- **5 份文件都改了，只动测试代码。** 5 份的 `prepare` 与钉值都在副本上用探针核过，5 条探针 `test result: ok. 1 passed`（第七节）。
  - 新钉值、改前数与算式在第四节。
  - `formatted_pool`：机械的一份，照 `step_seven` 同形改（18 → 26、33 → 41、两态闭式 262165 → 67108885）。
  - `parallel_line_one`：把内容挑小，单元数也回不到 ≤ 10。原因是固定开销：一个数据单元时 11 个单元，两个起 12 个；B 至少要两个数据单元才有两条记录，所以最少 14 个。
    - 钉值照今天的形状改成 B 14、C 15、段 `[…,26,…,30,…,32,…]`，文件头写明前提今天不成立、交主 agent 定。
    - 全量：今天的形状三态 12230590578、两态 5435818083。另两个候选的数在第五节。
  - `spill_over`：叶容器数不再写死。改成在起点镜像的内存拷贝上试发一次 `publish_new_inodes`，数 `TransactionOutput::rewritten` 里叶容器之外的角色，再迭代到不动点。
    - 今天是 11 个角色、57 片叶容器，回到「68 项、末条 1 项」。段 `[136, 4, 1, 2]`，快用例 25、根槽已落 9 都不变。
  - `tree_split`：7 条流的节点容量重新布置，每条都打中它名字说的那一形。每条流录之前与录之后的树形都用探针打出来核过，连叶里每条 key 的类、出生树、出生 txg 都打了。
    - 单元写段 18 到 26 写。全量七条合计 94634068，最大的是两层连着分裂那条，67108876。快档每条还是 13。
  - `position_addressed`（F2）：计数行改用带检查的闭式，装不进 u64 时打「超过 2^64 − 1，装不进 u64（最长一段 322 写）」，不 panic。比较与钉值没动。
- **变异**：追加 5 行，写在 `mutations-append.tsv`，主表没动。5 行点的都是 layer0 目标，全部留给 59 号。副本里逐条施加、只跑探针，5 条都在预期的断言上停（第八节），这只是旁证。
- **要主 agent 知道的**（第十节）：
  - G1：parallel_line_one 的前提与全量要定。
  - G2：tree_split 全量 9463 万个状态，跑不跑要定。
  - G3：根降高那条流改坏「根只剩一个孩子时降高」那一支也不红。
  - G4：别的会话在我开工之后改了 9 份在改文件，我在它们改动之后的第二份副本上重跑了探针，结果见第七节。
  - G5：74 号红的两条与 B3a-3b 报的相同。
- **什么现象会推翻这些结论**：
  - 提交时跑层 0，任何一份在 `prepare` 上红；
  - 快档状态数不是 37（formatted_pool）、117（parallel_line_one）、25 与根槽已落 9（spill_over）、每条 13（tree_split）、25 / 13 / 10 / 13 / 33（position_addressed）；
  - position_addressed 的快用例在计数行上 panic；
  - 追加的 5 行在 59 号上有一行不红；
  - 重跑第二节那个探针，a1f4691 不是 8 个单元，或 HEAD 不是 12 个（那样的话，归因要重查）。

## 二、单元数为什么变多（先做的那一件）

**做法**：三棵树各放一个探针目标 `tests/probe_b3a3c_first_file_units.rs`（源码 `draft/probe_b3a3c_first_file_units.rs`）。探针只做 `common::build_pool`，再打 `pool.output.rewritten` 与 mkfs 之后的段序列，不枚举。三棵树是：
- `git archive a1f4691`（上一次还钉 16 的那次提交，`first_transaction_step_five_publish.rs` 第 362 行 `(39, "2+2+1+2+2+1+18+2+1+2", 262_165)`）；
- `git archive HEAD`；
- 工作区副本（开工时取）。

每棵树用自己的 target，经 `run-with-memory-cap.sh 8G`、`capped.sh 5`。日志是 `logs/history-probe-{a1f4691,head,copy}.log`，原样：
```text
== a1f4691
PROBE_HISTORY rewritten_len=8 rewritten=[Data(DataUnitIndexInFile(0)), ExtentRoot, InodeLeafContainer(InodeLeafContainerIndexInTree(0)), InodeRoot, AllocationTree, AccountingTree, MappingTree, TreeTable]
PROBE_HISTORY writes=33 sizes=[2, 2, 1, 2, 2, 1, 18, 2, 1, 2]
== head
PROBE_HISTORY rewritten_len=12 rewritten=[Data(DataUnitIndexInFile(0)), ExtentRoot, InodeLeafContainer(InodeLeafContainerIndexInTree(0)), InodeRoot, AllocationTreeNodeBelowTheRoot(AllocationRecordTreeNodePosition { level: 0, device: DeviceIdentity(0), index_in_device: 61 }), AllocationTreeNodeBelowTheRoot(AllocationRecordTreeNodePosition { level: 0, device: DeviceIdentity(1), index_in_device: 61 }), AllocationTreeNodeBelowTheRoot(AllocationRecordTreeNodePosition { level: 1, device: DeviceIdentity(0), index_in_device: 0 }), AllocationTreeNodeBelowTheRoot(AllocationRecordTreeNodePosition { level: 1, device: DeviceIdentity(1), index_in_device: 0 }), AllocationTree, AccountingTree, MappingTree, TreeTable]
PROBE_HISTORY writes=41 sizes=[2, 2, 1, 2, 2, 1, 26, 2, 1, 2]
== copy（工作区）
（rewritten 与 head 逐字相同）
PROBE_HISTORY writes=41 sizes=[2, 2, 1, 2, 2, 1, 26, 2, 1, 2]
```
三条都是 `test result: ok. 1 passed`。

**落到哪次提交、哪份文件**：
- `git log --format=%h 346f5e6..HEAD -- crates` 是 0 条，所以 HEAD 的 `crates/` 就是 346f5e6 的。
- `git grep -c AllocationTreeNodeBelowTheRoot a1f4691 -- crates` 是 0 条；346f5e6 里 `transaction.rs` 有 15 处。
- 346f5e6 新增了 `crates/singlefs-core/src/allocation_record_tree.rs`（`git show 346f5e6 --stat` 记为 1028 行）。
- 机制：`AllocationRecordTreeGeometry::of_devices`（第 100 行起）求根层级 R：取最小的 R，使每块盘在 R − 1 层的格数加起来 ≤ 扇出 169。4 GiB × 2 时 R = 2，同文件第 904 行的单测钉着 `root_level() == 2`。
  - `nodes_holding_records`（第 333 行）与 `nodes_whose_contents_changed`（第 363 行）把一片叶的每个祖先都收进来。
  - 所以一次发布改到两盘各叶 61 的记录时，写的是 2 片叶、2 个第 1 层节点、1 个根，共 5 个节点。原来分配记录树只有一个 `AllocationTree`。
- 同一批实现员报告 `research/prompts/m2-keyspace-implementer-report.md` 第 329 行（实二一，原文整行）：
  `- 第一个事务的写清单因此变了：4 GiB × 2 上分配记录树五个节点（两盘各叶 61、两盘各第 1 层节点 0、根），先叶后根，`
- HEAD 与工作区之间：`git diff HEAD -- crates/singlefs-core/src/allocation_record_tree.rs` 只改了一个单测名（`a_changed_record_changes_…` → `changed_record_changes_…`）。探针上工作区与 HEAD 的数也相同。所以变多不在没提交的改动里。

**定案**（原文现查）：
- D8 已定项 14「两棵派生树的结构」与「实现取值」（`.claude/kb/decisions/08-核心索引结构.md` 第 372 行起）。第 396 行：`    - 根罩整个 key 空间、按盘分流；根层取最小的 R ≥ 1，使 Σ_盘 ⌈盘上槽数 ÷ (W × 169^(R−1))⌉ ≤ 169。4 GiB × 2 时根在第 2 层、树高 3。`
- 调度表第 108 行「派生树不分裂」、第 109 行「D8 已定项 14 改成允许异构」、第 125 行「key 空间四项」（K1–K4，用户 2026-09-24）。
- 调度表第 140 行「实二一交回的十四条」（主 agent 2026-09-25）：
  - ① W = 812、根按盘分流，写成实现取值；
  - ⑨「已有层 0 流钉着旧布局：提交时崩溃验证员跑，红了派实现员重钉」；
  - ⑩「第一个事务从 8 个单元变 12 个」。
- 所以这是定案要的变化，不是 bug。这一轮就是 ⑨ 说的「重钉」。

## 三、写过的文件

只写了派发给的 5 份，都在 `crates/singlefs-harness/tests/`，都只动测试代码。改动行数是对开工快照 `/tmp/claude-1000/impl-rev-b3a3c/originals/` 用 `git diff --no-index --numstat` 数的：

| 文件 | +/− |
|---|---|
| `second_transaction_step_three_formatted_pool_layer0.rs` | +15/−8 |
| `second_transaction_parallel_line_one_layer0.rs` | +38/−21 |
| `second_transaction_parallel_line_three_spill_over_layer0.rs` | +94/−8 |
| `second_transaction_supplement_two_tree_split_layer0.rs` | +54/−43 |
| `second_transaction_position_addressed_trees_layer0.rs` | +21/−4 |

- 全份 diff 在 `/tmp/claude-1000/impl-rev-b3a3c/logs/my-changes-vs-originals.diff`，549 行。
- `crates/mutations.tsv` 主表没改。要追加的 5 行在 `/tmp/claude-1000/impl-rev-b3a3c/mutations-append.tsv`（六段、制表符分隔，`awk -F'\t' '{print NF}'` 5 行全是 6），变异名：
  1. `实审 B3a-3c：按位置寻址的树计数行又直接算 1u64 << |段|（第三条流 322 写的单元写段移位溢出，快用例 panic）`
  2. `实审 B3a-3c：L8 叶容器数又按写死的 5 个角色取（63 片叶容器，点名 74 项、末条 7 项）`
  3. `实审 B3a-3c：树分裂 层 0：叶切在末尾（根分裂那条流录之后是 L9 L1 I2，不是 L5 L5 I2）`
  4. `实审 B3a-3c：树分裂 层 0：内部节点装不下也不切（两层连着分裂那条流录之后还是 L3 L3 L4 I3，不长第三层）`
  5. `实审 B3a-3c：树分裂 层 0：删空的叶不从父节点摘掉（摘空那条流在发布里就停在「规划与读回都不交出空节点」）`
- 5 行用脚本核过三样：名字在主表里没有；原文在各自的源文件里恰好命中 1 次；点名的测试函数在点名的测试文件里恰好 1 个。
- 不要 `mutations-replacements.tsv`：主表里点这 5 个目标的行（第 58、59、66、414、415、518、519、575、576、1041、1043、1044、1045、1046 行），锚点都在 `src/`，点名的测试名我一个没改。

## 四、每份文件：改了什么、钉值（改前 → 改后、算式）

行号都是改后文件自己的，`grep -n` 现取。

**`formatted_pool`**（照 `first_transaction_step_seven_layer0.rs` 已改的那几处同形）：

| 处 | 改前 | 改后 | 算式 |
|---|---|---|---|
| 第 59 行 段序列 | `[2,2,1,2,2,1,18,2,1,2]` | `[2,2,1,2,2,1,26,2,1,2]` | 系统配置两写 + 十二个单元各两盘 24 = 26 |
| 第 65 行 写数 | `33` | `2 + 2 * 5 + (24 + 2 + 1 + 2)` = 41 | 取号 2 + 两次零单元发布各 5 + 文件版本 29（24 单元写、两份记录、根槽、两盘轮换） |
| 第 134、137 行 两态闭式 | `262_165` | `1 + 6 * 3 + 3 + ((1 << 26) - 1)` 与 `67_108_885` | 1 + 六个 2 写段各 3 + 三个 1 写段各 1 + 2^26 − 1 |
| 文件头第 3 行、第 60、66 行的消息、第 126–127 行、第 140 行的说明 | 「18 写」「十六个单元写」「文件版本 21」 | 「26 写」「二十四个单元写」「文件版本 29」 | — |
| 快用例 | 22（两态）/ 37（三态） | 不变 | 26 写那一段不展开，18 还是 26 不影响 |

**`parallel_line_one`**（今天的形状；前提与全量交主 agent，第五节）：

| 处 | 改前 | 改后 | 算式 |
|---|---|---|---|
| 第 147 行 B 的单元数 | `9`（「≤ 10」） | `2 + 2 + 2 + 5 + 3` = 14 | 两个数据单元 + extent 下段根兼叶与上段根 + inode 叶容器与根 + 分配记录树五个节点 + 记账树、中央映射树、树表 |
| 第 161 行 C 的单元数 | `10` | `3 + 2 + 2 + 5 + 3` = 15 | 三个数据单元 + 同样的 12 |
| 第 173 行 段序列 | `[…,18,2,1,20,4,1,22,6,1,2]` | `[2,2,1,2,2,1,26,2,1,30,4,1,32,6,1,2]` | A 24 + 2、B 28 + 2、C 30 + 2 |
| 快用例 | 102（两态）/ 117（三态） | 不变 | 展开的只有 < 10 写的段，逐段与改前相同 |
| 第 352、355 行 全量两态 | `…((1<<18)-1)+((1<<20)-1)…((1<<22)-1)…`、`5_505_123` | `1 + 6 * 3 + 5 + ((1 << 26) - 1) + ((1 << 30) - 1) + 15 + ((1 << 32) - 1) + 63`、`5_435_818_083` | 1 + Σ(2^\|段\| − 1)，十六段 |
| 第 368 行起、第 375 行 全量三态 | `…9·2^16…9·2^18…9·2^20…`、`12_386_418` | `1 + 3·8 + 3·3 + 5 + (9·2^24 − 1) + (9·2^28 − 1) + 15 + (9·2^30 − 1) + 63`、`12_230_590_578` | 三个系统配置 2 写段各 3² − 1、三个记录 2 写段各 3、五个 1 写段、A / B / C 单元段各 3² · 2^(n−2) − 1、4 写段 15、6 写段 63 |
| 第 346 行 `#[ignore]` 的说明与文件头 | 「12386418 … debug 下数小时」 | 「12230590578 …前提今天不成立，要不要跑交主 agent 定」，文件头写明「≤ 10」今天不成立与原因 | — |

**`spill_over`**：
- 删掉 `JOURNAL_NAMED_ENTRIES_PER_RECORD - 5 + 1`，换成三样：
  - 第 50 行常量 `NAMED_ENTRIES_OF_THE_SPILLING_PUBLISH = JOURNAL_NAMED_ENTRIES_PER_RECORD + 1`；
  - 第 60 行 `roles_other_than_leaf_containers_named_by_a_trial_publish`：在起点镜像的内存拷贝上试发一次。分配器与上一版也用拷贝，盘由 `common::crash_state_devices` 给，写进一条另起的流。取 `singlefs_core::transaction::publish_new_inodes` 交回的 `TransactionOutput::rewritten`，除去 `TransactionUnit::InodeLeafContainer`，剩下的个数就是角色数；
  - 第 92 行 `leaf_containers_naming_one_more_entry_than_a_record_holds`：叶容器数 = 项数 − 这么多叶容器时的角色数，迭代到不动点，上界 8 轮。
- 角色数随叶容器数变，所以要迭代。探索探针（`logs/explore-probe_b3a3c_spill_over_explore.log`）的读数：1 片叶容器时 9 个角色；50、56、57、58、63、67 片时都是 11 个。多出来的两个是两盘各一片叶 62：叶容器落点越过叶 61 的末槽 50343。今天从 67 起算，第 2 轮就不动了，得 57。
- 第 165 行 `rewritten.len() == 68`：改前是 `68`，消息「63 片叶容器 + 5 个角色」；改后仍钉 68，消息带现数，另加一条断言：68 = `NAMED_ENTRIES_OF_THE_SPILLING_PUBLISH`。
- `(1, 67, 1)`、段 `[136, 4, 1, 2]`、快用例 25（= 1 + 15 + 1 + 8）、根槽已落 9（= 8 + 1）、边界计数 `1 / 6 / 9 / 9`：这些都没改，探针在今天的流上核过（第七节）。

**`tree_split`**：7 条流的布置与钉值见第六节。另改了三处：
- 全量用例的说明与 `#[ignore]`（第 367 行）：「约 22 万」改成 94634068；
- 文件头「比已有的流多罩了什么」：映射树节点数改成 1–6，补一句分配记录树五个节点；
- `plan()` 上面加了一段注释：第一个文件版本那 10 条映射 key 的次序，探针核过。

**`position_addressed`（F2）**：
- 第 400 行新函数 `closed_form_of_every_segment_for_the_count_line`：逐段 `checked_shl` / `checked_sub` / `checked_add`，装不进 u64 就交回「超过 2^64 − 1，装不进 u64（最长一段 N 写）」。
- 第 450 行计数行改调它。原来调的 `closed_form_state_count` 不再用，import 删掉。
- 比较（`tally.states ==` 三态函数）与钉值没动。

## 五、parallel_line_one：前提「一次发布不超过 10 个单元」与全量（交主 agent 定）

**挑内容大小压不下来**（`logs/explore-probe_b3a3c_parallel_line_one_explore.log`）。A 之后第一次顺序写，按内容大小列：

| 内容 | 字节 | 记录条数 | 单元数 |
|---|---|---|---|
| 1 字节 | 1 | 1 | 12 |
| 一个单元装满 | 32634 | 1 | 12 |
| 两个单元（第二个装一半） | 48951 | 2 | 14 |
| 两个单元装满 | 65268 | 2 | 14 |
| 三个单元（第三个装一半） | 81585 | 3 | 15 |
| 三个单元装满 | 97902 | 3 | 15 |
| 四个单元（第四个装一半） | 114219 | 4 | 16 |

- B 两个数据单元时写的单元（原样缩写）：`[Data(0), Data(1), ExtentLowerNode(…), ExtentRoot, InodeLeafContainer(0), InodeRoot, 分配记录树 L0d0#61, L0d1#61, L1d0#0, L1d1#0, AllocationTree, AccountingTree, MappingTree, TreeTable]`。
- 所以固定开销是：一个数据单元时 11 个，两个起 12 个。
- 两条记录至少要两个数据单元（记录条数等于数据单元数，上表），所以 B 最少 14、C（三条记录）最少 15，都超过 10。
- 就算池换成小盘让分配记录树只有一层（根 + 两盘各一片叶，3 个节点），B 也还有 12 个。这一条是按 `allocation_record_tree.rs` 的几何推的，没量过；这份文件用的 `common::build_pool` 写死 4 GiB × 2，我也没改。

**候选与各自的状态数**（探针在副本上现算，`logs/probe-round1-probe_b3a3c_parallel_line_one.log`；第二份副本上逐字相同）：

| 候选 | 段序列 | 全量三态（层 0 枚举域） | 全量两态 | 甲二快档三态 | 文件里的快用例（< 10 写的段） | 丢掉什么 |
|---|---|---|---|---|---|---|
| **今天的形状**（B 2 单元 / 14 个、C 3 单元 / 15 个；这一轮钉的就是它） | `[2,2,1,2,2,1,26,2,1,30,4,1,32,6,1,2]` | 12230590578 | 5435818083 | 168 | 117 | — |
| 只留 B，不发 C | `[2,2,1,2,2,1,26,2,1,30,4,1,2]` | 2566914099 | 1140850724 | 87 | 53 | 三条记录那一格（C 的 12 个断在记录之间的状态） |
| C 也只用两个数据单元（14 个单元） | `[2,2,1,2,2,1,26,2,1,30,4,1,30,4,1,2]` | 4982833218 | 2214592563 | 120 | 69 | 同上：C 变成两条记录，断在记录之间的状态 3 个而不是 12 个 |

- 「新形状」：挑内容大小没有一种能让单元数回到 ≤ 10，所以没有新形状；最小的内容就是今天的内容。上表后两行是减流的候选，不是挑内容。
- 全量要不要跑、前提改不改（改成「≤ 15」，或者只跑甲二快档加一条按段取样的层 1），我没定。
- 文件里：
  - `#[ignore]` 的说明写「要不要跑交主 agent 定」；
  - 文件头写明前提今天不成立、原因是什么；
  - 快用例（117）照旧能跑，它不受单元段长度影响。

## 六、tree_split：七条流逐条

**第一个文件版本的中央映射树**（探针打的叶里 key，`c` 类标签、`t` 出生树、`g` 出生 txg）：`[c1t11g3, c2t11g3, c2t12g3, c2t13g3 ×5, c2t14g3, c3t12g3]`，共 10 条。依次是数据单元、extent 根、inode 根、分配记录树五个节点、记账树、inode 叶容器。改前的布置按 6 条排，所以打不中。

- 空发布换掉分配记录树五个节点与记账树那 6 条：先删，再插 6 条 g4 的新 key，都落在 inode 根与 inode 叶容器之间。
- 覆盖写换掉全部 10 条。

| 流 | 第一个文件版本 / 被录那一次的容量（叶, 内部） | 录之前 | 录的那一次发生了什么 | 录之后 | 写几个单元（改前钉） | 单元写段 | 快档 | 全量三态（改前按旧 u 算） |
|---|---|---|---|---|---|---|---|---|
| CentralMappingRootSplit | 产品 / (9, 3) | `L10` | 删 6 剩 4，插回 6 条，第 10 条时从中间切（5 + 5），长出新根：树高 1 → 2 | `L5 L5 I2` | 10（6） | 20 | 13 | 1048588（4108） |
| CentralMappingLeafSplit | (9, 3) / (5, 3) | `L5 L5 I2` | 左叶删到 3 条、右叶删到只剩 inode 叶容器 1 条；新 key 全进右叶，到 6 条切一次（3 + 3），最后一条进切出的右边那片；根 3 个孩子装得下，树高不变 | `L3 L3 L4 I3` | 11（6） | 22 | 13 | 4194316（4108） |
| CentralMappingTwoLevelsSplitInARow | (9, 2) / (5, 2) | `L5 L5 I2` | 同上那一次叶分裂，根有 3 个孩子装不下（内部容量 2），根也切（2 + 1），长出第三层：树高 2 → 3 | `L3 L3 L4 I2 I1 I2` | 13（8） | 26 | 13 | 67108876（65548） |
| CentralMappingEmptyLeafDropped | (5, 3) / (8, 3) | `L3 L3 L4 I3`，中间那片只装分配记录树 3 条 | 中间那片删空、摘掉；新 key 全进右叶（inode 叶容器那 1 条 + 6 条 = 7，容量 8 不切）；最左那片一条没变、照抄 | `L3 L7 I2` | 9（8） | 18 | 13 | 262156（65548） |
| CentralMappingRootLowered | (9, 3) / 产品（覆盖写） | `L5 L5 I2` | 10 条全换：两片叶都删空摘掉，新的 10 条一个节点装得下：树高 2 → 1 | `L10` | 12（8） | 24 | 13 | 16777228（65548） |
| AccountingRootSplit | 产品 / (14, 3)（记账树） | 记账 `L15` | 15 行整批换代，第 15 行时切（8 + 7），长出新根 | `L8 L7 I2` | 10（6） | 20 | 13 | 1048588（4108） |
| AccountingLeafSplit | (14, 3) / (8, 3)（记账树） | 记账 `L8 L7 I2` | 15 行插到第 9 行切一次、第 14 行再切一次 | `L5 L5 L5 I3` | 11（7） | 22 | 13 | 4194316（16396） |

- 全量三态 = 1 + (2^(2u) − 1) + 3 + 1 + (3² − 1)。u 是被录那一次写的单元数；3、1、8 分别是记录段、根槽段、轮换段。七条合计 94634068。
- 探针对每条流都核了三样：这个闭式等于 `layer0_state_count_with_torn_in_place_overwrites(…, full_expansion)`；快档（< 4 写的段展开）等于 13；段序列等于 `[2u, 2, 1, 2]`。
- 改前那一列是 B3a-3b 按旧 u 算的钉值（文件里的全量用例按 `units_written` 现算，那个数没写死）。
- B3a-3b 报过两层连着分裂那条今天是 2^42 量级：那是旧布置在今天的树上长歪了（录之前就已经三层、21 个单元）。重新布置之后是 2^26 量级。

**每条流录之前、录之后的 key**（探针原样，`logs/probe-round1-probe_b3a3c_tree_split.log`，`mapping_leaf_keys_*` 那两段）：
```text
CentralMappingRootSplit before=[c1t11g3,c2t11g3,c2t12g3,c2t13g3,c2t13g3,c2t13g3,c2t13g3,c2t13g3,c2t14g3,c3t12g3] after=[c1t11g3,c2t11g3,c2t12g3,c2t13g4,c2t13g4] [c2t13g4,c2t13g4,c2t13g4,c2t14g4,c3t12g3]
CentralMappingLeafSplit before=[c1t11g3,c2t11g3,c2t12g3,c2t13g3,c2t13g3] [c2t13g3,c2t13g3,c2t13g3,c2t14g3,c3t12g3] after=[c1t11g3,c2t11g3,c2t12g3] [c2t13g4,c2t13g4,c2t13g4] [c2t13g4,c2t13g4,c2t14g4,c3t12g3]
CentralMappingEmptyLeafDropped before=[c1t11g3,c2t11g3,c2t12g3] [c2t13g3,c2t13g3,c2t13g3] [c2t13g3,c2t13g3,c2t14g3,c3t12g3] after=[c1t11g3,c2t11g3,c2t12g3] [c2t13g4,c2t13g4,c2t13g4,c2t13g4,c2t13g4,c2t14g4,c3t12g3]
CentralMappingRootLowered before=[c1t11g3,c2t11g3,c2t12g3,c2t13g3,c2t13g3] [c2t13g3,c2t13g3,c2t13g3,c2t14g3,c3t12g3] after=[c1t11g4,c2t11g4,c2t12g4,c2t13g4,c2t13g4,c2t13g4,c2t13g4,c2t13g4,c2t14g4,c3t12g4]
```
（每行的格式是我从探针那一行里摘出来重排的：只留流名与 `mapping_leaf_keys_before` / `_after` 两段，前缀改短成 `before=` / `after=`。两层连着分裂的叶与叶分裂相同，这里不另列；两条记账树流的映射树都是根兼叶，也不另列。）

**容量是怎么挑的**：
- 探索探针 `probe_b3a3c_tree_split_explore` 按格子逐个试：根分裂叶容量 4–10；叶分裂、两层分裂第一个文件版本 4–9 × 被录那一次 2–9；摘空 2–5 × 两种内部容量 × 八种目标叶容量 × 两种记账树容量；降高 4–9。每格打录之前、录之后的形状与单元数，日志 `logs/explore-probe_b3a3c_tree_split_explore.log`。
- 挑法：每一格只发生名字说的那一次结构变化，又不多出别的变化。
  - 根分裂：容量 4、5 会切两次（`L3 L3 L4 I3`），10 不切。
  - 叶分裂：目标容量 2、3 会切多次，7–9 不切。
  - 摘空：第一个文件版本容量 2 会摘掉两片，4、5 恰摘一片。
- 叶分裂、两层分裂、降高三条的「录之前」都取根分裂那一形 `L5 L5 I2`，好对照。

## 七、探针（钉值怎么算的；没有跑层 0）

- **副本**：
  - 第一份 `/tmp/claude-1000/impl-rev-b3a3c/copy`，用 `rsync -a --exclude target --exclude .git` 从主工作区取，用它自己的 target。取之前记了 18 份文件的 sha256，副本里逐份相同（`logs/sha-at-start.txt`）：13 份在改文件与我这 5 份。
  - 开工之后别的会话改了 9 份在改文件：`mount.rs`、`transaction.rs`、`journal.rs`、`unit.rs`、`history.rs`、`model.rs`、`first_transaction_on_device.rs`、format `lib.rs`、checker `lib.rs`。所以又取了第二份 `copy2`，sha 记在 `logs/sha-at-copy2.txt`、逐份相同，那时主树编得过。
- **做法**：
  - 5 份改后的层 0 文件各整份拷成名字不含 layer0 的目标 `tests/probe_b3a3c_*.rs`，摘掉 `#[test]` / `#[ignore]`，末尾接一条探针用例。生成器是 `draft/probe/make_probes.py`（copy2 用 `make_probes_copy2.py`）。
  - 探针只调 `prepare`（文件里的形状断言照跑），再调状态数函数，拿文件里钉的算式比，不枚举。
  - `formatted_pool` 那两条不枚举的用例、`tree_split` 的 `the_accounting_streams_read_the_tree_height_from_the_root_node_header`，探针整条照调。
- **每条都经 `run-with-memory-cap.sh 8G` 与 `capped.sh 5`**，一次都没撞到包装的 250–254。
- 第一份副本上 5 条都是 `test result: ok. 1 passed`（`logs/probe-round1-*.log`）；第二份副本上 5 条也都是 `ok. 1 passed`（`logs/probe-copy2-round2-*.log`），打出的数与第一份逐字相同。

探针原样（第二份副本，树分裂那几行删掉了 `mapping_leaf_keys_*` 与 `rewritten=`）：
```text
PROBE formatted_pool writes=41 sizes=[2, 2, 1, 2, 2, 1, 26, 2, 1, 2] closed_form_every_segment=67108885 fast_two_state_expanded=22 fast_three_state=37 full_three_state=150994980
PROBE position_addressed stream=ExtentInlineToLowerSegment segments=[28, 4, 1, 2] fast_three_state=25 closed_form_for_the_count_line=268435475
PROBE position_addressed stream=ExtentLowerSegmentBackToInline segments=[24, 2, 1, 2] fast_three_state=13 closed_form_for_the_count_line=16777223
PROBE position_addressed stream=ExtentLowerSegmentGrowsToTwoLevels segments=[322, 290, 1, 2] fast_three_state=10 closed_form_for_the_count_line=超过 2^64 − 1，装不进 u64（最长一段 322 写）
PROBE position_addressed stream=AllocationRecordTreeTwoLeavesPerDevice segments=[28, 2, 1, 2] fast_three_state=13 closed_form_for_the_count_line=268435463
PROBE position_addressed stream=VersionWithoutFileRowPublishWithItsOwnTree segments=[2, 14, 2, 1, 2, 2, 1, 2] fast_three_state=33 closed_form_for_the_count_line=16401
PROBE spill_over leaf_containers=57 roles=11 sizes=[136, 4, 1, 2] states=25 root_persisted=9 record_writes=[[136, 137], [138, 139]]
PROBE tree_split stream=CentralMappingRootSplit shape_before=L10 shape_after=L5 L5 I2 units=10 sizes=[20, 2, 1, 2] quick=13 full=1048588 closed_form_two_state=1048583
PROBE tree_split stream=CentralMappingLeafSplit shape_before=L5 L5 I2 shape_after=L3 L3 L4 I3 units=11 sizes=[22, 2, 1, 2] quick=13 full=4194316 closed_form_two_state=4194311
PROBE tree_split stream=CentralMappingTwoLevelsSplitInARow shape_before=L5 L5 I2 shape_after=L3 L3 L4 I2 I1 I2 units=13 sizes=[26, 2, 1, 2] quick=13 full=67108876 closed_form_two_state=67108871
PROBE tree_split stream=CentralMappingEmptyLeafDropped shape_before=L3 L3 L4 I3 shape_after=L3 L7 I2 units=9 sizes=[18, 2, 1, 2] quick=13 full=262156 closed_form_two_state=262151
PROBE tree_split stream=CentralMappingRootLowered shape_before=L5 L5 I2 shape_after=L10 units=12 sizes=[24, 2, 1, 2] quick=13 full=16777228 closed_form_two_state=16777223
PROBE tree_split stream=AccountingRootSplit shape_before=L15 shape_after=L8 L7 I2 units=10 sizes=[20, 2, 1, 2] quick=13 full=1048588 closed_form_two_state=1048583
PROBE tree_split stream=AccountingLeafSplit shape_before=L8 L7 I2 shape_after=L5 L5 L5 I3 units=11 sizes=[22, 2, 1, 2] quick=13 full=4194316 closed_form_two_state=4194311
PROBE tree_split full_total=94634068
PROBE_MULTI_RECORD publish=0 record_writes=[[69, 70], [71, 72]]
PROBE_MULTI_RECORD publish=1 record_writes=[[106, 107], [108, 109], [110, 111]]
PROBE_CANDIDATE today_b_two_units_c_three_units sizes=[2, 2, 1, 2, 2, 1, 26, 2, 1, 30, 4, 1, 32, 6, 1, 2] full_two_state=5435818083 full_three_state=12230590578 quick_tier_three_state=168 fast_three_state=117
```
- 探针里比的式子与文件里钉的式子逐字一样，不等就 panic。
- `formatted_pool` 的 22 / 37、`parallel_line_one` 的 102 / 117 / 5435818083 / 12230590578、`spill_over` 的 25 / 9、`tree_split` 每条的 13 与全量闭式，都过了。
- `position_addressed` 第三条流打出了说明、没 panic。另外四条打的数与 B3a-3b 记的 `two_state_full` 相同（268435475、16777223、268435463、16401）。

## 八、变异

追加 5 行（第三节）。都点名字带 layer0 的目标，`prove-red.sh` 会跳过，我也不许跑这类目标，所以**一行都没证红，全部留给 59 号**。

| # | 改坏哪一行 | 点名的用例 | 该红在哪 |
|---|---|---|---|
| 1 | `position_addressed` 第 403 行 `let proper_subsets = 1u64.checked_shl(segment_length)?.checked_sub(1)?;` → `(1u64 << segment_length).checked_sub(1)?` | 快用例 `every_position_addressed_tree_stream_recovers_cleanly_in_every_state_of_its_small_segments` | 第三条流计数行 `attempt to shift left with overflow`（F2 退回来） |
| 2 | `spill_over` 第 148 行 `new_inode_count_filling(leaf_containers)` → `new_inode_count_filling(JOURNAL_NAMED_ENTRIES_PER_RECORD - 5 + 1)` | `every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records_applies_it_whole_or_not_at_all` | `prepare` 里 `rewritten.len() == 68`：74 ≠ 68 |
| 3 | `code_two_tree.rs` `let right_keys = keys.split_off(keys.len().div_ceil(2));` → `keys.len() - 1`（与主表第 552 行同一处） | tree_split 快用例 `every_tree_split_stream_recovers_cleanly_in_every_state_after_its_unit_writes` | `prepare` 的 `shape_after`：根分裂 `L9 L1 I2` ≠ `L5 L5 I2` |
| 4 | `code_two_tree.rs` `if children.len() > capacity.internal_entries {` → `if false && …`（与主表第 555 行同一处） | 同上 | 两层连着分裂 `L3 L3 L4 I3` ≠ `L3 L3 L4 I2 I1 I2` |
| 5 | `code_two_tree.rs` `if children[child_index].node.entry_count() == 0 {` → `if false && …`（`delete_below` 里摘空节点那一句） | 同上 | 摘空那条流在发布里 `expect("规划与读回都不交出空节点")`（`code_two_tree.rs:1149`）panic |

**旁证**（不是证红）：
- 做法：在第一份副本里逐条施加，只跑对应的探针（探针调同一个 `prepare`）。脚本 `draft/mutation-effect-2.sh`，日志 `logs/mutation-{ma,mb,mc,md,me,mf}.log`。
- 还原：每条跑完从 `draft/copy-originals/` 拷回并 `touch`，跑完 `cmp` 三份都与原件相同（`progress.md` 的 `restored-identical` 三行）。
- 原样：
```text
ma: panicked at crates/singlefs-harness/tests/probe_b3a3c_position_addressed.rs:404:30: attempt to shift left with overflow
mb: assertion `left == right` failed: 57 片叶容器 + 叶容器之外的角色（由试发现数），恰好比一条记录装得下的 67 多一项：[…]  left: 74  right: 68
mc: assertion `left == right` failed: CentralMappingRootSplit：被录那一次之后这棵树的样子  left: "L9 L1 I2"  right: "L5 L5 I2"
md: assertion `left == right` failed: CentralMappingTwoLevelsSplitInARow：被录那一次之后这棵树的样子  left: "L3 L3 L4 I3"  right: "L3 L3 L4 I2 I1 I2"
me: panicked at crates/singlefs-core/src/code_two_tree.rs:1149:18: 规划与读回都不交出空节点（前三条流打过 PROBE 行，停在第四条摘空）
mf: test result: ok. 1 passed（见第十节 G3，这一条没进追加表）
```
- 这些行是从日志里摘的，`…` 是我删掉的 `rewritten` 长清单。每条之后的 `test result` 都是 `FAILED. 0 passed; 1 failed`，只有 mf 是 ok。
- 旁证说明的只到一步：变异改到了用例钉的那个量，并且在同一个 `prepare` 上停。层 0 用例是不是真红在这里，要 59 号证。

## 九、交回前的验证（第 4 步那几样，末尾原样）

开跑前 `ps`：没有 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`。有别的会话的 `cargo test`（`second_transaction_supplement_three_*`、`probe_a4c_leaves`）与一个上游 `gate.sh`，我的命令都加了 `nice -n 19`、经 `capped.sh 5`。等锁没单独计时，各步挂钟在 `progress.md`。

- **动到的测试二进制**：5 个都是名字带 layer0 的，按定义一个都不跑。它们的比较与钉值交提交时的层 0 验证（第十一节）。探针（第七节）核的是同一份 `prepare` 与钉值的算式。
- `cargo build --offline --all-targets`（主树，最后一次改文件之后）：退出 0。末行 `    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.22s`，日志里 warning 0 条。
  - 这一趟快，是因为别的会话在我改完之后已经编过。
  - 我核了主树 `target/debug/deps` 里这 5 个层 0 测试二进制的修改时间都晚于各自源文件。
- `rustfmt --edition 2021 --check` 逐个查我这 5 份：都退出 0，输出 0 字节。第一次查时 `position_addressed` 的一处 `use` 换行不合格式，改了再查。
  - `cargo fmt --all -- --check`（主树）：退出 1。`Diff in` 67 处，全在 `crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`，不是我的。
- clippy（check.sh 那套：`-D warnings` 加 7 条）：
  - 只查我这 5 个目标（`-p singlefs-harness --no-deps --all-features` 加 5 个 `--test`）：退出 0，末两行 `    Checking singlefs-harness v0.1.0 (crates/singlefs-harness)` / `    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.97s`。
  - 整个工作区 `--all-targets --all-features --keep-going`：退出 101。`could not compile` 的是 `checker_narrow_invariants_and_abandoned_roots`（test）、`e158_root_choice_repair`（bin 与 bin test）、`e156_allocation_basis_counts`（bin 与 bin test），`-->` 点名的也只有这三份文件。日志里我这 5 个目标的名字出现 0 次。
- 命名检查（不归我，顺手跑的）：`naming-lint.sh .` 退出 1，输出里我这 5 份文件名出现 0 次。
- 登记给我的门禁阶段（主树，经 `capped.sh 5`），退出码与判定行原样：
  - 33 号：退出 0。判定行 `  ✓ 148 个实验二进制都有成形的变异表，1690 条变异的原文各命中源码一次；crates/mutations.tsv 1088 条的原文各命中源码一次；…`，主表我没动。
  - 53 号：退出 0，`  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：D23（journal 的角色与格式） 已定项 19；C506（每区槽数 S 写成编译期常量，条文说它住系统配置）；D22（单元原子性怎么合成） 已定项 1；C323（镜像大小全仓没有条款））`
  - 92 号：退出 0，`  ✓ 布局清单 1 套布局、6 条路径都在，位号都对得上 D15（格式冻结政策） 已定项 4 的登记表；这次改动比 73ba4a4c019b9e3fc9c92f3122bfbbdaee93c321，106 个格式常量里变了 8 个（checker 在同一次改动里跟了 8 个，按滞后表放行 0 个），都不欠 checker 跟进`
  - 94 号：退出 0，`  ✓ checker 与实现只共享常量模块 `singlefs-format`（传递闭包的交集减去它为空：checker 闭包 1 个、实现闭包 2 个，内部依赖图 4 个 crate）；checker 的 4 份源码零处引 `singlefs_core`（别名引进来的 0 个）；共享模块 1 份源码的正文 285 行里没有分支与循环（`#[cfg(test)]` 标着的项 271 行不扫）`
  - 93 号：退出 0，`  ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 56 个 .rs，认出 4 处 feature bit 常量、解出位号 3 处；没判位号的 1 处：SUPPORTED_INCOMPAT_BITS（crates/singlefs-core/src/system_configuration.rs:31，常量声明没写在一行里，解不出值））`
  - 89 号：退出 77，`  ⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（5 条逐字探针、覆盖 4 笔，逐条对上今天的值）`。本次未跑，不是通过。
  - 74 号（阶段里面自己经内存包装，外面只套 `capped.sh 5`）：退出 1，`test result: FAILED. 22 passed; 2 failed; 2 ignored; 0 measured; 0 filtered out; finished in 43.21s`，`  ✗ 随机历史的测试二进制判红（上面是 cargo test 的尾部）`。
    - 红的是 `crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43` 与 `random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation`，与 B3a-3b 报的是同两条。
    - 那是 `second_transaction_supplement_three_random_history` 这个测试二进制。我只改了 5 个层 0 测试文件，它们编不进它，所以不是这一轮带出来的。

## 十、交主 agent 的问题与发现

**G1：parallel_line_one 的前提与全量**（第五节）。
- 里程碑 `.claude/kb/milestone/02-second-txn.md` 第 490 行「层 0 小负载：一次发布的单元数不许超过 10」今天做不到：两条记录的发布最少 14 个单元。
- 我照今天的形状重钉、写明前提不成立，没改 kb，也没定全量跑不跑。
- 候选与状态数：今天的形状 12230590578；只留 B 2566914099；C 也用两单元 4982833218；甲二快档依次 168 / 87 / 120。

**G2：tree_split 全量 94634068 个状态**。`#[ignore]` 照旧写「交 crash-verifier 在 release 下跑」。最大的一条 67108876，与第一条流全量的两态数（67108885）同一量级。跑不跑、怎么分片由主 agent 定。

**G3：根降高那条流判不出「根只剩一个孩子时降高」那一支**。
- 现象：把 `code_two_tree.rs` 的 `delete_at_the_root` 里 `PlanningNode::Internal { children, .. } if children.len() == 1 => {` 改成 `if false && …`（与主表第 554 行同一处），树分裂探针照样 `ok. 1 passed`，降高那条录之后照样是 `L10`（`logs/mutation-mf.log`）。
- 原因（读代码推的）：覆盖写把 10 条 key 全删掉。产品路径是左叶删空后根剩一个孩子、降高。改坏之后根不降高，右叶也删空后走 `children.is_empty()` 臂变回空叶。两条路结局都是一个节点装回 10 条。
- 这条流打中了「树高 2 → 1」，那一支由单测（主表第 554 行）守着，层 0 这条流守不住。所以我没把它加进追加表。
- 要让层 0 也判得出，得让录的那一次删空一片叶、同时留下一条不换的 key。比如先用 `publish_new_inodes` 多建一片 inode 叶容器，覆盖写不改它，它的映射 key 就留着。这要改流的形状，不在这一轮的「重算节点容量」里，没做，交主 agent 定。

**G4：别的会话在我开工之后改了 9 份在改文件**（第七节列了名）。我在它们改动之后的第二份副本上重跑了 5 条探针，打出的数与第一份逐字相同。之后它们再改，钉值可能还会动，提交时的层 0 那一趟为准。

**G5：74 号红的两条**（第九节）与 B3a-3b、B3a-2 报的同两条，在随机历史那个二进制里，不是这一轮带出来的。

**G6：F2 的改法在测试文件里另写了一份带检查的闭式**。`crash::closed_form_state_count`（`crash.rs` 第 620 行起）照旧直接算 `1u64 << |段|`，别的用例碰到长段照样会 panic。它在 B3c-2 在改的 `crash.rs` 里，我不碰。要不要把它改成带检查、交回 `Option`，交主 agent 排。

**G7：spill_over 的 `prepare` 多了两次试发**（67 片、57 片叶容器各一次，都在内存拷贝上）。第一份副本上整条探针 9.79 s（debug），第二份上 5.32 s。只是记一下。

**F5 同形：没有停在「条款没写」的分支上**。这一轮只改测试里的布置、比较与钉值，没加错误成员、`todo!`、`assert!`。

## 十一、受影响的层 0 流与崩溃枚举用例

checker（`crates/singlefs-checker/src/`）没动，按定义这一节可以不写。列出来是给集成时排快档用：
- 5 份的快用例（提交时的层 0 那一趟）：
  - `formatted_pool` 的三条（两条不枚举，一条快档 37）；
  - `parallel_line_one` 快档 117，断在记录之间的 `[3, 12]`；
  - `spill_over` 25，边界计数 `1 / 6 / 9 / 9`；
  - `tree_split` 七条各 13，另有一条树高用例；
  - `position_addressed` 五条 25 / 13 / 10 / 13 / 33。
  - 这 5 份在 `prepare` 上过得去，是探针在副本上核的。枚举本身（oracle、checker、记录核对器在每个状态上清不清）没核：tree_split 的 7 条流形状全换了，它们的单元写段子集第一次按新布置跑。
- 带 `#[ignore]` 的全量：
  - `parallel_line_one`：12230590578，G1；
  - `tree_split`：94634068，G2；
  - `position_addressed`：四条流，数没变，由 B3a-3b 记。
  - 按 `research/prompts/m2-layer0-scale-r1-opus-output.md` 第 205 行，这几条没有哪道门禁跑。
- 没有新加层 0 流，也没有新的崩溃枚举用例（没新写 `enumerate_layer0` 一族的测试函数），不涉及 `crash-case:` 登记。

## 十二、`git diff --stat -- crates litmus`

主树原样。工作区里同时有别的会话与此前各批没提交的改动，分不出谁的，我写过的以第三节为准。全份在 `logs/git-diff-stat.txt`，这里摘含 layer0 的几行与末行：
```text
 .../tests/first_transaction_step_seven_layer0.rs   |  261 +-
 .../second_transaction_parallel_line_one_layer0.rs |  103 +-
 ...action_parallel_line_three_spill_over_layer0.rs |  138 +-
 ..._transaction_position_addressed_trees_layer0.rs |   49 +-
 ...action_step_three_acquisition_barrier_layer0.rs |   55 +-
 ...transaction_step_three_formatted_pool_layer0.rs |   55 +-
 .../tests/second_transaction_step_zero_layer0.rs   |  663 +-
 ...ction_supplement_two_rollback_witness_layer0.rs |  457 --
 ...transaction_supplement_two_tree_split_layer0.rs |  137 +-
 94 files changed, 30874 insertions(+), 11385 deletions(-)
```
这 5 份对 HEAD 的行数里含 B3a-3b 没提交的那一批，我的只有第三节那几行。

## 十三、草稿目录删了什么、留了什么

- **删了**（都是我这一轮建的，里面各有自己的 target）：
  - 仓副本 `/tmp/claude-1000/impl-rev-b3a3c/copy`（2.7G）；
  - `/tmp/claude-1000/impl-rev-b3a3c/copy2`（2.1G）；
  - 两份 `git archive` 展开的树 `/tmp/claude-1000/impl-rev-b3a3c/history/a1f4691`（780M）与 `history/head`（1.2G），连同 `history/` 目录。
  - 删之前把副本里生成的探针源码拷进了 `draft/probe/generated/`。
- **留着**，都在 `/tmp/claude-1000/impl-rev-b3a3c/` 下，共 1.2M，没有仓副本与编译目录：
  - 报告 `report.md`，规格 `spec.md`，进度 `progress.md`；
  - 变异行 `mutations-append.tsv`，主 agent 用 `apply-writer-patch.py` 或手工写回；
  - 开工快照 `originals/`（5 份）；
  - `draft/`：
    - 探针生成器 `probe/make_probes.py`、`make_probes_copy2.py`，生成出的 9 份在 `probe/generated/`；
    - 三份探索探针，第一个文件版本探针 `probe_b3a3c_first_file_units.rs`；
    - 驱动脚本 `run-history.sh`、`run-explore.sh`、`run-probes.sh`、`run-probes-copy2.sh`、`run-checks.sh`、`run-gates.sh`、`mutation-effect.sh` / `mutation-effect-2.sh`（跑的是后者）、`apply_mutation.py`；
    - 变异原文与替换文 `mutations/`，副本原件 `copy-originals/`。
  - 日志 `logs/`：历史探针、探索、两趟探针、变异旁证、build、clippy、fmt、rustfmt、命名检查、门禁各阶段、sha、git diff。
- 这些都不入库。探针只是用来算钉值、核归类的一次性工具，依据已经写进这份报告。要复算，照第七节在新副本里重建。

## 十四、没做什么

- 没跑任何名字带 layer0 的测试二进制（规格与定义都不许），只编了。5 份改后的比较与钉值都没在枚举上核过，交提交时的层 0 验证与 59 号。
- 变异 5 行一行都没证红（都点 layer0 目标），全部留给 59 号；第八节的探针旁证不算证红。
- 没改 `crates/mutations.tsv` 主表，没改 kb（里程碑第 490 行的「≤ 10」、D8 已定项 14 都没动）。
- parallel_line_one 的前提改不改、全量跑不跑，tree_split 全量跑不跑，根降高那条流要不要改形状：都没定，交主 agent（第十节 G1–G3）。
- 没碰 `crash.rs`（F2 的根在它的 `closed_form_state_count`，B3c-2 在改），也没碰别的会话在改的文件与 `tests/common`、`tests/common_tree_split`。
- 没跑三方对抗；层 0、QEMU、herd7 与 crates 变异整表归 `crash-verifier`；没提交。
