# 攻方腿报告：这次挂载开过的聚簇段怎么在挂载期间还给用户数据（m4-cluster-segments-back-to-data-design-r1）

日期：2026-11-18。设计推论轮第一轮，攻方腿（Opus）。攻击面：过度外推与越位、内部矛盾与循环依赖、哪一条最脆弱。没改 `crates/`，没跑任何项目代码；下文「量的」只指任务 20071 日志里已有的行，模型的数全部「推的，待主 agent 跑」。

## 复跑命令与模型文件

模型目录 `research/prompts/m4-cluster-segments-back-to-data-design-r1-opus-model/`（纯 python 计数模型，只写标准输出与运行目录；不涉及崩溃状态，没调崩溃枚举函数）。复跑命令见文末「要主 agent 跑的命令」一节（外层 `tools/rerun-from-a-clean-work-directory.sh` 先删运行目录再调 `tools/rerun.sh`；`rerun.sh` 先跑一小段估全量挂钟，超过 40 分钟退 3 不跑全量）。

| 文件 | sha256 |
|---|---|
| `model/cluster_segments_model.py` | `469868cec8a12b946af385123aa626f5265b136c2a0974702b220ca346b4933d` |
| `tools/rerun.sh` | `3e153dc114c818f5d70ca27e3d580c92c27158bb2b0dfd285f29b89febf6520d` |
| `tools/rerun-from-a-clean-work-directory.sh` | `e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b` |

`out/` 是空的：模型一次没跑过，绝对值断言（`out/cluster-segments-model.lines` 逐行比）要等主 agent 第一次跑出之后由承接的腿钉；在那之前 `rerun.sh` 只打「还没有」。`rerun.sh` 与模板 `scripts/agents/three-way-rerun-template.sh` 只差 `rerun_steps` 那一段（`diff <(sed -n '1,79p;85,$p' 模板) <(sed -n '1,79p;105,$p' rerun.sh)` 退 0）。

## 各格判定一览

| 攻击面 | 格 | 判定 | 一句话 |
|---|---|---|---|
| 过度外推与越位 | 甲在任务 20071 那段历史上 | 打中 | 被拒那一刻只放出 8 段（≤ 256 对，量的直方图 `0:8`），而且这 8 段今天正是提交内生块唯一的全空段；还给数据之后合取 7 先拒，推的只多撑约 76 次发布 |
| 过度外推与越位 | 乙在长寿块多的历史上 | 打中 | 门槛 t 是负载的函数：每段钉住的活槽数超过 t 的历史（模型 `long_lived_heavy`，推的每段约 9 个）乙一段都不还 |
| 过度外推与越位 | 丙在长寿块是中央映射节点的历史上 | 打中 | 中央映射树节点、树表、实例表在整理那里是钉住的，带钉住槽的段没资格当 R（`compaction_plan_proofs.rs:64`）；这种段丙永远搬不空 |
| 过度外推与越位 | 丁在「先建一阵元数据、再删掉」的历史上 | 打中 | 丁不让集合出段：删掉之后这次挂载开过的段全空了也对数据关着；被拒那一刻（20071）丁一对都不放 |
| 过度外推与越位 | 聚簇段个数随发布次数怎么涨 | 推不出（模型已写，待跑） | 今日量的 0.0734 段/次发布（20071 日志）；各候选的涨法在模型里，没跑 |
| 内部矛盾与循环依赖 | 规则护着什么有没有漏 | 打中 | 背景材料列的四样之外，今天的单调集合还暗中兜着三样：提交内生块的全空段储备（合取 7 靠它）、抬 F 上界「只多不少」、整理的 R 不会是开放段 |
| 内部矛盾与循环依赖 | 甲、乙与「抬 F 全放出来也放不下」的上界 | 打中 | 集合会缩之后 `allocator.rs:887` 那句「只会比真放开之后成的对数多，不会少」不再成立，判不下而推一串之后落得下（假性 ENOSPC 那一类） |
| 内部矛盾与循环依赖 | 丙与 D26 已定项 1、2 | 打中 | 放宽资格之后 R 可以是开放段，搬迁写诞生代，目标态的单调性破；签发前判在数据段写满时恒拒；产出谓词与「还给数据」目标相反 |
| 内部矛盾与循环依赖 | 丁与 D3 已定项 10 ① | 打中 | 「段起点 = 单元区内最低的、64 槽对齐的全空段」与「复用有空槽的旧段」字面不能同真 |
| 内部矛盾与循环依赖 | 乙、戊与整理签发前判 | 打中 | 「只装提交内生块的段不算非全空」：还给数据的稀疏段是数据目的地却不算空洞，签发前判少数 |
| 内部矛盾与循环依赖 | 准入「判得下、分配器落不下」 | 没打中 | 数据单元先取落点、准入判在固定点之前；集合在一次发布里只会因开段而进，进的是数据取完之后 |
| 内部矛盾与循环依赖 | 崩溃在「还给数据」前后、重挂之后落点 | 没打中（甲乙丁戊）/ 打中（丙，形状） | 甲乙丁戊只动内存，盘上一个字节不因「还」而变，重挂走 `Self::new` 与今天相同；丙引入 R 是这次挂载开过的段的整理意图，新的崩溃点组合 |
| 最脆弱 | 四条里哪一条 | 丙 | 要同时改一条 Verus 证过的资格 spec、补一条「R 不是开放段」、改签发前判与产出谓词，还绕不过钉住的映射节点 |
| 核背景材料没核的一句 | 「整理在本次挂载里也还不回来」 | 成立（读码，推的；没量） | 比调查员说的多三道：核里资格排除本次挂载开过的段、签发前判在 20071 那一刻空洞数为 0、映射节点钉住 |

改法与零轮：本报告提的每一个改法只在我的推理与没跑过的模型上，被攻过零轮；改法表在「改法各修哪一格」一节，格子全标「推的」。

## 过度外推与越位（各候选在哪一段历史上还不回空间）

### 已知事实（量的，任务 20071 日志）

- 被拒那一刻的探针行在日志第 119 行：「label=refusal publishes=21987 segments=2288 cluster_segments=1614」，同一行「scan_cluster_free_pairs=49770 scan_cluster_live_histogram=0:8,1:1416,16-31:1,2:140,3-7:46,8-15:3」，且「core_empty_segments=8」（日志 `~/.local/state/singlefs/task-server/logs/task-b0f6b921-20071.attempt-3.log`，仓外，cite-check 认不出路径）。
- 聚簇段数随发布的涨法：用 awk 从日志里所有 `sequential_half`、`interleaved_half`、`after_alignment`、`refusal` 行取 `(cluster_segments, publishes)` 的首尾两点，打出「rate=0.0734 over 21986 publishes」（量的，首点 publishes=1、cluster_segments=1，末点 publishes=21987、cluster_segments=1614）。逐段看，顺序写那一半与交错写那一半都在每 1056 次发布约 76 段上下，近似线性、没有拐头。
- 全空段数（记录口径）在 publishes=20897 是 119、21953 是 12（同一份日志第 115、118 行）；全空聚簇段水位线按 `compaction_plan.rs` 的默认 5% 算是 ⌈2288 × 5 ÷ 100⌉ = 115 段（算术），所以产品路径上的后台整理要到约第 20897–21953 次发布之间才起跑。

### 甲「全空即还」：在 20071 那段历史上还不回空间（打中）

- 被拒那一刻：甲能还的是这次挂载开过、已经全空的段。直方图里活槽为 0 的是 8 段（量的），最多 8 × 32 = 256 对（推的上界：那 8 段要是还有等回收的槽，按背景材料「没有被扣住的槽」的口径还更少）。那一次写要 1 对，够；可这 256 对只够约 256 次单单元发布。
- 更紧的是合取 7：这 8 段同时就是全池仅有的 8 个全空段（`core_empty_segments=8`，`scan_outside_empty=0`，同一行）。今天它们留在集合里，数据进不去，等于给提交内生块留的储备；甲把它们还给数据，数据取最低空对、提交内生块开段取最低全空段，两边一起吃这 8 段。按量的两个速率（提交内生块 0.0734 段/次、数据 1/32 段/次），8 段约 8 ÷ 0.105 ≈ 76 次发布吃完（推的），之后写之前全空段数为 0 < N₀ = 1，前台写入改由合取 7 拒（「`全空聚簇段数 ≥ N₀`」那一行，`.claude/kb/decisions/03-空间分配.md:148`；N₀ 第一版是 1，「pub const EMPTY_CLUSTER_SEGMENTS_FLOOR_OF_THE_FIRST_VERSION: u64 = 1;」（`crates/singlefs-core/src/compaction_plan.rs:498`））。拒因从「段外槽对」换成「全空聚簇段低于底额」，盘上仍压着约 4.9 万对。
- 换一段历史也一样：甲只还「活槽恰好为 0」的段。今天的开段规则本来就会把回收空了的旧聚簇段重新开来装提交内生块（「段里的块被回收空了也仍算聚簇段：它还能被 `lowest_empty_segment` 重新开来装提交内生块」，`crates/singlefs-core/src/allocator.rs:2292`），所以今天被永久关掉的恰好是「还剩至少一个活槽」的那些段——甲碰不到它们。只要每段写满之后留下一个以上长寿块（20071 里 1416 + 140 + 46 + 3 + 1 = 1606 段是这样，量的直方图），甲在渐近上与今日同速吃段。模型 `pool_construction_like` 把这件事量成 `return_when_empty` 与 `today` 两行 `refused_at_user_publish` 的差（推的，待跑）。

### 乙「低于门槛即还」：在长寿块多的历史上还不回空间（打中）

- 20071 被拒那一刻：门槛 1 能放 8 × 32 + 1416 × 31 = 44152 对，门槛 2 再加 140 × 30 = 4200 对，到 48352 对（推的上界：假设每个活槽各占一对、没有等回收的槽；量的总数 `scan_cluster_free_pairs=49770` 罩住它们）。够。
- 反例历史：每段写满时留下的长寿块数超过门槛 t。模型 `long_lived_heavy` 三棵顺序树每 2、3、5 次发布各换一片叶，每次发布约 1.03 个长寿块，每段约 9 个活槽（推的）；乙门槛 1、2 在这段历史上一段都不还，与今日同速。t 取多大都有这样的历史（长寿块率乘每段装几次发布 > t），所以乙的门槛不是一个能定死的常数，而是负载的函数；t 取到 63 就退化成「只关当前开放段」（模型里的戊，`close_only_the_open_segment`）。
- 这一格把乙推到戊：能还的空间恰好是规则护着的「段将来能整段空出来」那一样（下面「内部矛盾与循环依赖」一节），乙与戊之间没有不付这个代价的中间点。

### 丙「整理搬空稀疏段再还」：在长寿块是中央映射节点的历史上还不回空间（打中）

- 整理把没有映射条目的那几类单元当钉住：「TransactionUnit::MappingTreeNodeBelowTheRoot(_)」（`crates/singlefs-core/src/mounted_session.rs:4941`）与中央映射树根、树表、实例表一起进 `pinned_slots`（「if is_exempt_from_the_mapping {」，`crates/singlefs-core/src/mounted_session.rs:4972`）；选段核的资格里有「&&& !segment.holds_a_pinned_slot」（`crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs:64`）。所以一段里唯一的活槽要是中央映射树的节点，丙放宽多少「本次挂载」的资格都选不中它。
- 中央映射树按单元的映射 key 追加，顺序写时叶写满就不再碰——正是长寿块的来源之一（推的；20071 的直方图分不出活槽是哪一类单元）。模型 `mapping_nodes_pin_the_segments` 让中央映射叶每 7 次发布换一片、搬不动（推的，待跑），丙在这段历史上应与甲同（只有全空段回来）。
- 20071 那一刻照今天的代码丙一对都放不出（见「核背景材料没核的一句」）。

### 丁「提交内生块复用稀疏段」：在「先建一阵元数据、再删掉」的历史上还不回空间（打中）

- 丁只改「开段开哪一段」，不改集合只进不出（「self.cluster_segments.insert(segment_start);」是唯一的写，`crates/singlefs-core/src/allocator.rs:3075`）。反例：一次挂载里先建大量小文件（每次发布留几个长寿元数据块，模型 `metadata_burst_then_delete` 3000 次发布、每次 3 个），开出几百段；再一次删掉它们，这几百段全空、却仍在集合里；之后每次发布写 16 个数据单元，数据一对都进不去这几百段。丁会把其中几段拿来复用装提交内生块，可提交内生块用不了那么多，其余对数据一直关着，到重挂为止。
- 20071 被拒那一刻：丁一对都不放（它只改以后开哪一段）。它若从挂载一开始就生效，20071 这段历史大概率不被拒（推的：复用让集合只随长寿块数涨），模型 `pool_construction_like` 的 `commit_generated_reuses_sparse` 一行量它（待跑）。
- 攻方提的组合「丁加甲」（模型 `reuse_sparse_and_return_when_empty`）在这一格上推的修得好，被攻过零轮，待跑。

### 聚簇段个数随发布次数怎么涨（推不出：模型已写、没跑）

模型每 4096 次用户发布打一行 `name=cluster_trajectory`（`cluster_set`、`opened_in_this_mount`、`pairs_open_to_data`、`empty_segments`、`fallbacks`），结尾一行 `name=cluster_result`。今日臂的标定点是 20071 的 0.0734 段/次（量的）；模型的提交内生块每次发布约 6–7 槽（推的，比 20071 推算的 4.6 槽多），所以今日臂会比 20071 早拒——比的是同一模型里各臂相对今日的差，不比绝对发布数。没跑之前不写各臂的斜率。

## 内部矛盾与循环依赖

### 规则护着什么：条款写了的，与今天的单调集合暗中兜着、条款没写的

条款本身：「这次挂载开过的聚簇段（当前开放的那一段，以及回落之前开过的段）都不给用户数据」（`.claude/kb/decisions/03-空间分配.md:168`）。它的依据只有一条用户定案：「用户定案 2026-09-23：一个聚簇段什么时候不再算——照当天的实现写成条款」（`.claude/kb/decisions/03-空间分配.md:184`），没有实验。那次弹窗用户先选的是只关开放段：「弹窗里先选的选项字面是「只有当前开放的那一段关着」」（`.claude/kb/decisions-history.md:1150`），重问之后改成照代码写；主 agent 当时的推荐是「主 agent 推荐「段里还有已分配槽就仍算」，保住聚簇的目的（段将来能整段空出来）」（`.claude/warnings/2026-09-23.md:13`）——这句就是甲的反面（有活槽的才关），用户没选。

逐样列，各自在「段里只剩一两个活的提交内生块」「段已全空」两种情形下成不成立（推的，读条款与代码）：

| # | 护着什么 | 出处 | 稀疏段（关着的、剩 1–2 活槽） | 全空段（关着的） |
|---|---|---|---|---|
| 1 | 开放段只装提交内生块，固定点有常数上界 | 「全散布时没有上界——常数上界只在聚簇政策下存在」（`.claude/kb/decisions/03-空间分配.md:106`）；「用户数据进 bump 段时它们绝大多数落不进」（`.claude/kb/decisions/03-空间分配.md:179`） | 不需要：已关的段不再被 bump（今天只有开放段被 bump）；只在丁要复用它时需要 | 不需要（同左） |
| 2 | 段将来能整段空出来，全空段供给不退化 | `.claude/warnings/2026-09-23.md:13`；「第 2 条混放的代价：装着用户数据的旧段要等段里的用户数据也释放才能整段空出来」（`.claude/kb/decisions/03-空间分配.md:175`） | 成立，而且是乙、戊要拿走的那一样：还给数据就混放 | 不成立：段已经空了，不护什么 |
| 3 | 崩溃一致性 | 条款射程写明是分配器政策、盘上没有字段（`.claude/kb/decisions/03-空间分配.md:175`「它是**分配器政策**」那一句） | 无关：集合只在内存 | 无关 |
| 4 | 整理与 discard | D26 已定项 1 的全空段水位与产出谓词按记录口径数，与集合无关 | 无关 | 无关 |
| 5 | 分配次序的确定性 | I-5.5 只管从 mkfs 水位建树那一版，checker 把开放段近似成「开放段拿这一版树表单元所在的段当它」（`crates/singlefs-checker/invariants/07-分配器.md:10`），已释放记录一律当占着 | 不受影响（推的：还给数据的段里的空槽都是已释放记录，checker 照样当占着；被数据复用之后成了这一版自己的记录，checker 当空） | 同左 |
| 6（漏）| 提交内生块的全空段储备 | 无条款；今天集合里回收空了的段数据进不去，只有开段能用它们。20071 被拒那一刻 8 个全空段全在集合里（量的） | — | 成立，而且是唯一兜底：合取 7 的 N₀ 靠它兑现；甲拿走它 |
| 7（漏）| 抬 F 先算槽对那一判的上界只多不少 | 「也不问推串时聚簇段会不会多开：只会比真放开之后成的对数多，不会少」（`crates/singlefs-core/src/allocator.rs:887`）：这句的真假押在集合推串期间只进不出 | 成立的前提：集合不缩 | 同左 |
| 8（漏）| 整理的 R 不会是开放段，搬迁目的地不落回 R | 选段核资格「&&& !segment.is_opened_in_this_mount」（`crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs:62`）；开放段 bump 不看 R（`crates/singlefs-core/src/allocator.rs:3233` 起的 `bump_slot_in_open_segment`，挡位只问「.any(|device| device.is_blocked_for_commit_generated(SlotNumber(slot)))」，`crates/singlefs-core/src/allocator.rs:3249`） | 成立的前提：开放段在集合里、集合里的段不许当 R | 同左 |

第 6、7、8 样背景材料「规则护着什么」那一格没列，欠账表里提到聚簇段的 23 行（`grep -c '聚簇段' .claude/kb/checks-owed.md` 打出 23）我逐行看过标题，也没有一条点名它们；最近的是 C571（「`df` 报的可用空间要不要把这些槽算进去没有条款」，`.claude/kb/checks-owed.md:341`），它管的是今天这个现象本身算不算假性 ENOSPC（D3 已定项 9 第 1 条「只要 `df` 报出的空闲 ≥ s，写 s 字节就必须成功」，`.claude/kb/decisions/03-空间分配.md:199`）。

### 上界会缩：甲、乙让集合会缩，「抬 F 全放出来也放不下」的上界不再是上界（判不下而推一串之后落得下）

- 上界读的是推串之前的集合：槽对那一判交「slots.slot_pairs_once_everything_is_released.0 * slots_of_a_data_unit,」（`crates/singlefs-core/src/mounted_session.rs:6094`），那个数由 `slot_pairs_without_an_unreleased_record_outside_the_cluster_segments` 按当时的 `allocator.cluster_segments()` 减去集合里的段（`crates/singlefs-core/src/mounted_session.rs:6138`）。
- 反例历史（推的，读码）：照 20071 的样子走到被拒那一刻，改用甲。上一个开放段 X 刚换段，里面只有最近几版的提交内生块（没有长寿块）。此刻 X 在集合里，上界不数它，交 0，会话照「全放出来也放不下」一串都不推就报没有空间——与 20071 失败行同一个读数「most_a_floor_raise_could_make_available: AvailableBytesOnOneDevice(0)」（背景材料转述的拒因）。可只要推几次空发布：每次空发布重写树根、树表与记账，把 X 里最近几版换下来、抬 F 回收，X 全空、按甲出集合，32 对回来。乙同理（X 的活槽降到 t 以下即出）。
- 四句：分辨臂——分：甲、乙中，戊（集合只有开放段）、丁、今日不中。系统当时看不看得到——看不到：推串之前不知道这一串会换下哪些版本、哪些段会降到门槛以下（判据要区分的「推完之后 X 还在不在集合里」在判的那一刻不可观测）。满足判据字面哪一分句——「准入与上界」格「两处怎么改，怎么与分配器的真实行为钉成一致」。改法在打中那几格上还中不中——见「改法各修哪一格」表：上界改成「集合里除开放段之外按记录口径全放之后算」只会更大、不会更小（推的），这一格修得掉，代价是 20071 这类真放不下的情形也要先推满一串才拒（C1003 那一类的「被宽护卡住」挂钟）。

### 储备被拿走：甲拿走提交内生块的全空段储备，合取 7 先拒

见「甲「全空即还」」那一节讲合取 7 的那一条。四句：分辨臂——分：甲、丙（丙用甲的出集合规则）中；乙、戊在 20071 那一刻先还的是稀疏段，数据按最低空对先吃那些更低的空洞（推的：全空段在数据前沿之上），储备留得更久，但写满时同样归零；今日、丁不中。系统看不看得到——看得到（`empty_segments` 是增量维护的）。满足哪一分句——「规则护着什么」格（「提交内生块的全空段储备」那一样）与「还得回多少」格「够不够那一次写」之后的那一半「换一段历史还成不成立」。改法——数据取落点时把全空段数压在 N₀ 以上（全空段只在全空段数 > N₀ 时给数据），推的，被攻过零轮。

### 丙与 D26 咬合：丙与 D26 已定项 1、2 的三处咬合

1. **放宽资格之后 R 可以是开放段，目标态的单调性破（活锁）。** 今天 R 不会是开放段，只因为开放段在集合里、核的资格排除集合里的段（`crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs:62`；`crates/singlefs-core/src/compaction_plan.rs:240` 把它接成「is_opened_in_this_mount: cluster_segments.contains(&start_slot),」）。提交内生块的搬迁目的地走开放段 bump：「码 1 走用户数据落点函数（D3（空间分配） 已定项 8 第 1 条），节点与打包容器走开放段 bump」（`crates/singlefs-core/src/compaction_plan.rs:473`），而 bump 先于任何 R 判（「if let Some(slot) = self.bump_slot_in_open_segment(open, footprint) {」，`crates/singlefs-core/src/allocator.rs:3011`），bump 本身只看三种挡位（`crates/singlefs-core/src/allocator.rs:3249`）。丙要把「本次挂载开过的段」放进候选，就得另写「不是开放段」；漏写时反例：刚开的开放段只有两三个活槽，按 greedy「活槽最少的段先」它是第一候选 → R = 开放段 → 搬走的节点 bump 回 R。搬迁的目的地记录写诞生代：「目的地那条分配记录的分配代写单元的诞生代（不写搬迁那次发布的 txg）」（`.claude/kb/decisions/26-后台整理与放置回收.md:141`），所以落回 R 的单元分配代 ≤ c0，目标态「R 内不存在分配代 ≤ c0 且未释放的落点」永远达不到；D26 已定项 2 的单调性论证只罩前台：「前台在 `R` 内新分配的落点，其分配代必然 > `c0`」（`.claude/kb/decisions/26-后台整理与放置回收.md:58`），罩不住搬迁落回 R。今天这条路被「R 不在集合里」挡着，条款里没写。
2. **签发前判在数据段写满时恒拒。** 「签发前判 `R` 外非全空段（只装提交内生块的段不算非全空）的空洞数 ≥ `R` 里要搬的单元数」（`.claude/kb/decisions/26-后台整理与放置回收.md:29`），实现「「非全空」按记录口径；只装提交内生块的段不算非全空」（`crates/singlefs-core/src/compaction_plan.rs:1345`），拒在「if !holes_admit_the_issue(holes, units_to_move) {」（`crates/singlefs-core/src/mounted_session.rs:5581`）。20071 被拒那一刻段外 674 段全满（`scan_outside_full=674`、`scan_outside_free_pairs=0`，量的），空洞数为 0，任何 R（哪怕只搬 1 个节点、目的地其实是开放段）都签不出去。这个判法数的是用户数据目的地，对搬提交内生块的 R 是错的量。
3. **产出谓词与「还给数据」相反。** 一条意图完成之后签不签下一条看全空段数涨没涨（`.claude/kb/decisions/26-后台整理与放置回收.md:29`；「if !the_completed_intent_yielded(empty_now, completed.empty_cluster_segments_at_issue) {」，`crates/singlefs-core/src/mounted_session.rs:5431`）。丙的目的是把搬空的 R 还给数据，数据按最低空对取，R 在前沿之下，下一次签发之前前台一写就把 R 吃掉，全空段数回到 segs0 或更低，整理停在「无产出」。D26 已定项 2 射程预言过同形：「两个聚簇段之间来回搬的历史里每一条意图都正常完成而全空段供给恒为零」（`.claude/kb/decisions/26-后台整理与放置回收.md:88`）。模型 `compaction_empties_sparse_then_return` 记两个计数：`issue_check_would_refuse`（照今天的签发前判会被拒的意图数）与 `yield_predicate_false`（照今天的产出谓词会停的次数），待跑。

四句（合并三处）：分辨臂——只中丙。系统看不看得到——看得到（开放段、空洞数、全空段数都是此刻的读数）。满足哪一分句——「整理」格「丙要整理做的事今天的整理做不做得到」与「停机谓词、整理的目的地选择受不受影响」。改法——「R 是开放段」那一处补一条资格「不是开放段」就修掉；签发前判与产出谓词两处要改 D26 已定项 1 第 2 条的两句，改法在表里，都推的。

### 丁与定义句：丁与 D3 已定项 10 ① 字面不能同真

「段起点 = 单元区内最低的、64 槽对齐的全空段」（`.claude/kb/decisions/03-空间分配.md:215`），代码 `lowest_empty_segment_outside_the_region` 判「self.used_per_segment[*segment] == 0」（`crates/singlefs-core/src/allocator.rs:1410`）。丁要开的是「这次挂载开过、有空槽的段」，按定义不是全空段。丁要改这一句与「可开聚簇段」那个登记量的定义（同一行后半「两个量分开登记，不许互相顶替」）。另：乙与丁抢同一批稀疏段——乙要把它们给数据、丁要把它们给提交内生块；两条一起上必须写谁先，混放之后丁再 bump 进去就违反「开放段只装提交内生块」那一样。四句：分辨臂——只中丁。看得到——是（字面）。分句——「规则护着什么」格与问题里「写成对实现的改动」那一条。改法——改 ① 的定义句（推的）。

### 乙戊与签发前判：乙、戊与整理签发前判的口径

还给数据而数据还没进去的稀疏段「只装提交内生块」，按 `crates/singlefs-core/src/compaction_plan.rs:1345` 不算空洞；可它是码 1 搬迁目的地（用户数据落点函数会落进去）。签发前判少数，整理在有目的地时也签不出去（推的）。这一形重挂之后今天就有（上一次挂载开过、只装提交内生块的段对数据开放、却不算空洞），乙、戊把它搬进了同一次挂载。四句：分辨臂——中乙、戊，不中甲（甲还的段全空，不在这一句的射程）。看得到——是。分句——「整理」格。改法——空洞按「数据落点函数此刻认不认」数，推的。

### 没打中：准入「判得下、分配器落不下」

找的形状：准入那一判数过之后、数据单元取落点之前集合变大（数据被挤出）。今天的发布先排数据单元（`admission.rs` 文档「数据单元在这次的角色里排最前、先取落点」那一句所在的 `admit_the_units_landing_on_every_device` 注释），集合只在提交内生块开段时进（`crates/singlefs-core/src/allocator.rs:3075`），开段发生在数据取完之后；甲、乙、戊让集合在释放或换段时出，出只会让数据多一些地方。固定点每轮从拷贝重来，次序不变。所以按读码，这一方向在四条候选上都造不出（推的；多块盘各自的隔离位不同时两块盘的段外槽对可以不同，但那是今天就有的逐盘判，与集合无关）。

### 崩溃在「还给数据」前后、重挂之后的落点

- 甲、乙、丁、戊：「还」只改内存里的集合，盘上一个字节不变；崩溃在「还」前、「还」后，盘上状态相同（「还」之后写进去的数据单元是另一次发布，照常受发布的崩溃一致性管）。重挂走 `rebuild_from_records`，里面「let mut allocator = Self::new(devices);」（`crates/singlefs-core/src/allocator.rs:2674`），集合从空开始，重挂之后的落点规则与今天逐字相同（推的，读码）。崩溃点重放的状态空间：状态的形状不变，但同一段历史写出的字节会变（落点变了），已登记用例里一次挂载内发布次数够多、走到段被还的那几条，期望值要重钉——哪几条要靠主 agent 跑一遍才知道，我没枚举。
- 丙：多了「R 是这次挂载开过的稀疏段」的整理意图。意图在盘上（logged_ops），崩溃之后续做；「一批整理要改写 K+1 个已发布根，这种提交点**组合**没被崩溃点采样枚举过」（`.claude/kb/decisions/26-后台整理与放置回收.md:88`）在丙下多了一类 R。重挂之后 R 不再是「这次挂载开过的段」（集合空了），续做照今天的路走（推的）。

## 哪一条最脆弱

**丙最脆弱**，卡在五处，除崩溃那一处外都是实现中途要改已证过或已定的东西（推的，读码）：

1. 选段资格是 Verus 证过的 spec：「&&& !segment.is_opened_in_this_mount」（`crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs:62`）。改它要改证明，还要翻掉变异表里专门杀「选段不排开这次挂载开过的聚簇段」的那一行（`crates/mutations.tsv:3926`）与它的用例 `a_segment_opened_in_this_mount_is_not_chosen`。
2. 改完必须另补「R 不是开放段」，否则「R 是开放段」那一处的活锁；这一条今天没有任何条款或用例看着（它被选段资格顺带挡住）。
3. 签发前判与产出谓词（D26 已定项 1 第 2 条，`.claude/kb/decisions/26-后台整理与放置回收.md:29`）是用户定案、带判决的已定项，丙要改它两句，等于重开 D26 已定项 1。
4. 钉住的中央映射节点（`crates/singlefs-core/src/mounted_session.rs:4941`）丙绕不过：这一类长寿块丙永远搬不走，丙在这一类历史上退化成甲。
5. 崩溃一致性上丙是四条里唯一新增盘上状态组合的（整理意图的 R 换了一类）；甲、乙、丁、戊不改盘上字节。

次脆弱是**乙**：门槛没有不随负载变的取值（「过度外推与越位」一节），取大就是戊；它还要改两处读集合的读数（上界、签发前判）。**甲**实现最简单（一处 `retain`），但它在 20071 上几乎还不回东西、还拿走合取 7 的储备，属于「做得完、没用」。**丁**要改 D3 已定项 10 ① 的定义句，实现上是开段那一支换一个选段函数，难点在与乙互斥。

## 核背景材料主 agent 标了没核的一句：「整理在本次挂载里也还不回来」

结论：成立（读码，推的；没造盘面、没量）。调查员的理由是集合只进不出（`crates/singlefs-core/src/allocator.rs:3075` 是唯一的写）；那一条之外还有三道独立的挡，任一道都够：

1. 选段不选这次挂载开过的段：「资格五条全成立的段才算：不在这次挂载开过的聚簇段里」（`crates/singlefs-core/src/compaction_plan.rs:84`），核里同一条（`crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs:62`），会话交的是「allocator.cluster_segments(),」（`crates/singlefs-core/src/mounted_session.rs:5500`）。所以那 1606 个还有活槽的聚簇段整理根本不看。
2. 能选的只剩段外的段，20071 那一刻它们 674 段全满（量的）；greedy 选到哪一段，签发前判的空洞数都是 0（数据段全满，只装提交内生块的段不算），停在 `HolesOutsideTheRegionFewerThanItsUnitsToMove`（`crates/singlefs-core/src/mounted_session.rs:5581` 那一判）。往前推：交错写那一半里段外数据段只有前沿那一段有洞，greedy 选中的恰是前沿段（活槽最少的数据段），R 外的空洞仍是 0，同样签不出（推的）。
3. 就算签得出、搬空了一段聚簇段，它还在集合里（调查员那一条）。

另：产品路径上后台整理要到全空段数低于 115 段（约第 20897–21953 次发布之间，量的轨迹）才起跑；E202 装置那一次挂载根本不起跑（调查员读码，`~/.local/share/singlefs/work/e202-pool-construction-no-space-investigation/result.md`「第 2 问」一节「造池不跑整理」那一条，仓外）。推翻这条结论的观测：在 20071 那样的造法里打开后台整理，拒绝那一刻之前出现 `compaction_intents` > 0 而且之后段外槽对涨回来。

## 改法各修哪一格（全部推的，被攻过零轮）

| 改法 | 上界会缩 | 储备被拿走 | R 是开放段 | 签发前判 | 产出谓词 | 丁与定义句 | 乙戊与签发前判 | 20071 还不还得回 |
|---|---|---|---|---|---|---|---|---|
| 上界改成「集合里除开放段之外按记录口径全放之后算」 | 修，推的 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不改变 |
| 数据只在全空段数 > N₀ 时进全空段 | 不碰 | 修，推的 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 甲下仍不回；乙戊下不影响 |
| 选段资格加「不是开放段」 | 不碰 | 不碰 | 修，推的 | 不碰 | 不碰 | 不碰 | 不碰 | 不改变 |
| 签发前判对只搬提交内生块的 R 改数「开放段与可开段的空槽」 | 不碰 | 不碰 | 不碰 | 修，推的 | 不碰 | 不碰 | 不碰 | 丙才可能还 |
| 产出谓词改数「段外数据可落的对数涨没涨」 | 不碰 | 不碰 | 不碰 | 不碰 | 修，推的 | 不碰 | 不碰 | 丙才可能还 |
| D3 已定项 10 ① 改「段起点 = 本次挂载开过、空槽 ≥ k 的段，没有再取最低全空段」 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 修，推的 | 不碰 | 只防以后，不还已关的 |
| 空洞按「数据落点函数此刻认不认」数 | 不碰 | 不碰 | 不碰 | 修一半，推的 | 不碰 | 不碰 | 修，推的 | 不改变 |
| 丁加甲（模型 `reuse_sparse_and_return_when_empty`） | 中（甲那一半让集合会缩） | 中 | 不碰 | 不碰 | 不碰 | 仍要改 ① | 不碰 | 推的，待主 agent 跑 |

没有一格是「量过」：没有任何改法在副本上实现、跑过。

## 模型怎么建的（取样与估时）

- 一块盘代表池，单元区 2288 段（与 20071 同几何的段数），每段 64 槽；状态按槽记空、活、等回收三种，按段增量维护活槽数、挡住的槽数、空对数，跑完逐槽重数一遍对账（`check_the_model`，对不上抛 `ModelInconsistency`）。
- 提交内生块的寿命：几棵「顺序树」每 period 次发布换一片叶，旧叶最后一版留下当长寿块；每次发布另有 2 个总被重写的块（树根、树表这类，当钉住）；分配记录叶一片罩 12 段（推的：20 字节一条、16 KiB 约 800 条），这次分配或释放碰到哪片就重写哪片，固定点到不再碰新叶。释放之后隔 reclaim_lag 次发布回收（取 4 与 32 两档）。
- 准入：前台写入写之前全空段数（记录口径）< 1 判合取 7；段外空对不够判槽对；提交内生块连回落都落不下判提交内生块。第一次拒就停这一趟。
- 历史四段 × 臂八条（今日、甲、乙门槛一、乙门槛二、戊、丙、丁、丁加甲）× 回收滞后两档 = 64 趟，另把 `pool_construction_like`、甲、滞后 32 再跑两次比结果行（`name=cluster_determinism`，模型自己的确定性，不是被测系统的）。
- 估时：`--estimate` 先跑 2000 次发布量每次发布的秒数，按「各趟上限发布数 × 臂权重（丙 ×3）」估上界；`rerun.sh` 超过 40 分钟退 3 不跑全量。我没跑，不知道估出多少（推的：每次发布几十微秒量级，全量约十分钟、内存百兆以内）。
- 没缩：四段历史都跑到第一次被拒或上限（`pool_construction_like`、`long_lived_heavy`、`mapping_nodes_pin_the_segments` 上限 80000 次，`metadata_burst_then_delete` 9000 次）。

## 没打中的形状

- 准入「判得下、分配器落不下」：试的是「准入之后、数据取落点之前集合变大」与「固定点轮之间集合变化」两种，四条候选上读码都造不出（见上）。
- 甲、乙、丁、戊在崩溃前后改盘上字节：读码都只改内存集合，没造出。
- 落点确定性：四条候选的集合进出都只读分配器此刻的状态（活槽数、挡位、开放段），同一历史两次跑相同（推的）；丙让前台能不能写进依赖后台整理跑没跑到，这一层今天就有（R 排他、整理腾段都已经影响落点），没算新打中。
- I-5.5：试的是「还给数据的段里的空对比 checker 认的最低空对更低」，因为 checker 把已释放记录当占着、复用后的记录当空，两边认的落点一致（推的，读 `crates/singlefs-checker/src/walk.rs` 里 `first_data_unit_slot_the_allocation_records_leave_on_a_device`），没造出误红。
- 准入路径代价：甲、乙按段增量维护活槽数，出集合 O(1)；读集合仍是 O(集合段数)（「读它们 O(这次挂载开过的聚簇段数)」，`crates/singlefs-core/src/admission.rs:974`），集合变小只会更便宜。丁的「挑空槽最多的段」要么扫集合、要么另维护一个堆，扫的写法随集合段数涨（推的），算代价不算打中。

## 这条腿自己的限度

- 模型一次没跑：所有模型读数都是「推的，待主 agent 跑」；模型自己的错（标定、寿命过程、分配记录叶的范围）可能让臂的排序反过来。今日臂的标定点（20071 的 0.0734 段/次）与模型的提交内生块槽数对不上（推的模型约 6–7 槽/次，20071 推算约 4.6 槽/次），比的是同一模型里各臂之间的差。
- 20071 的直方图分不出活槽是哪一类单元，「长寿块里有多少是中央映射节点」我没量，丙在 20071 上能搬走多少只给了形状、没给数。
- 「上界会缩」那一格的反例是读码造的，没在副本上走过一遍抬 F 串。
- 欠账表我只按「聚簇段」一词搜了 23 行；别的词（「开放段」「bump 段」）没搜。
- 背景材料里的 kb 附录我没用：kb 引文都是现读 kb 文件、现取行号。

## 要主 agent 跑的命令

1. `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=设计轮 m4-cluster-segments-back-to-data-design-r1 攻方腿的聚簇段计数模型（纯 python，四段历史乘八条臂乘两档回收滞后共 64 趟，另两趟确定性复跑）; 范围=一个模型脚本 66 趟; 为什么=量四条候选各在哪段历史上还不回空间、聚簇段个数随发布次数怎么涨，判决要引的模型数要由主 agent 复跑' task-server/task submit --cwd /home/fy5090/code/singlefs -- bash /home/fy5090/code/singlefs/research/prompts/m4-cluster-segments-back-to-data-design-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs/research/prompts/m4-cluster-segments-back-to-data-design-r1-snapshot /home/fy5090/.local/share/singlefs/work/m4-cluster-segments-back-to-data-design-r1-opus/model-run /home/fy5090/code/singlefs/research/prompts/m4-cluster-segments-back-to-data-design-r1-opus-model`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`（新 shell）。
   - 看哪几行：末行 `TASK_SERVER_RESULT` 的 `exit=0` 算跑完（`exit=3` 是估时超 40 分钟没跑全量，`exit=1` 是模型自检或确定性没过）；`task-server/task log <号> --all` 里 `name=estimate` 一行的 `estimated_minutes_upper_bound`、64 行 `name=cluster_result`（`grep -c '^name=cluster_result'` 应当是 64）、`name=cluster_determinism … two_runs_identical=yes`、没有 `ModelInconsistency`。全量输出在 `/home/fy5090/.local/share/singlefs/work/m4-cluster-segments-back-to-data-design-r1-opus/model-run/cluster-segments-model.out`，去掉 `timing` 与 `name=estimate` 行的在同目录 `cluster-segments-model.lines`（承接的腿把它收进模型目录 `out/` 钉绝对值）。
   - 判读（跑前写下，推的）：`pool_construction_like` 上甲的 `refused_at_user_publish` 比今日多不到 5%、拒因多半是 `empty_cluster_segments_below_the_floor`；乙、戊、丁、丁加甲远多于今日。`long_lived_heavy` 上乙两档与今日相差不到 5%。`mapping_nodes_pin_the_segments` 上丙与甲相差不到 5%。`metadata_burst_then_delete` 上丁与今日的 `data_units` 相差不到 5%，甲、丁加甲多出几百段的对。量出相反的数（比如甲在 `pool_construction_like` 上多撑一倍以上），「过度外推与越位」一节里对应那一格改判「没打中」。

## 没做什么

- 没跑任何命令（模型、编译、测试、门禁都没跑）；判决要引的模型数要主 agent 复跑上面那一条。
- 没改 `crates/`，没建仓副本、没建编译目录；草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-cluster-segments-back-to-data-design-r1-opus` 里除认领文件外没有东西（模型运行目录由那一条命令建）。
- 不判正推腿那几格（问题里「写成对实现的改动」那一条只在改法表里给了攻方视角的推的改法）；不替主 agent 采纳。
- 没读禁读清单里的文件。
- 没逐条核调查员报告里与本题无关的部分（6251 与 6652 步数差、探针补丁）。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/m4-cluster-segments-back-to-data-design-r1-opus-output.md` 末行：「✓ 核了 26 处引文、30 处裸行号，对上 26 处，没判 30 处（其中裸行号没有引文可比 30 处；逐处列在上面）」。仓外的两份（任务 20071 日志、调查员报告）路径认不出，没进这个数；它们的行号是 `grep -n` 现取的。
