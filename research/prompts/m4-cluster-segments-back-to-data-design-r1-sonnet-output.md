# 设计轮 m4-cluster-segments-back-to-data-design-r1 正推腿报告

日期：2026-11-18。立场：正推（从已定条款与 `crates/` 今天的实现推「应该是什么」，再逐格对）。方案按 `crates/` 今天的实现谈，没改 `crates/`。

标记：**[读]** 现读文件所得（带文件与行）；**[量]** 日志或产物里已有的读数；**[推]** 我从上面两样推出来的，没有跑。没写模型，没跑任何项目代码，「要主 agent 跑的命令」一节写无。

## 结论（一页）

1. 缺陷的机理是两件事叠起来：集合只进不出（`crates/singlefs-core/src/allocator.rs:3075` 是写集合的唯一一处），而且关闭的聚簇段里的空槽谁都用不了——数据被集合挡着，提交内生块开新段只认全空段（`crates/singlefs-core/src/allocator.rs:1410`）。任务 20071 被拒那一刻，1416 个聚簇段各剩一个活槽，各有 31 个空槽对被锁着。
2. 四条候选里：甲（全空即还）还得回的最少，还偏偏把提交内生块要的「全空段」这一样东西送给数据；乙（低于门槛即还）看门槛怎么写，写成「活槽数 ≤ θ 就不在集合里」时数据自己落进去会把段再关上；丙（整理搬空再还）今天有三处走不通；丁（提交内生块复用稀疏段）治的是集合长得太快，不治「数据已经被关在外面」。
3. **推荐戊（新加的一条）**：数据落点的候选序改成「集合外先、集合内后」，集合内的段只有当前开放的那一段永远不给；集合外没有两槽都空的落点时，才在集合内取（有活槽的段先于全空段）。条款上是把「关着」改成「排在后面」。它在被拒那一刻还得回约 49.5 千个槽对（推，下面第 3 节），够那一次写的约 3 千对需求的十几倍；没有压力时行为与今天逐字相同；准入那一判与「抬 F 全放出来也放不下」的上界都从「O(集合大小)」变成「O(1)」，且与分配器真实行为用同一个函数。
4. 丁作为第二步另立一轮：它改的是提交内生块的开段规则（已定项 10 ① 的「全空段」三个字），碰到已定项 5 要的固定点常数上界，没有量过；它让聚簇段个数从「每次发布涨 0.073 段」变成「随活的提交内生块涨」，但不能代替戊（大批删除之后集合里全是空洞、数据照样进不去）。
5. 「整理在本次挂载里也还不回来」坐实，而且比调查员推的更强：整理连这次挂载开过的聚簇段都不选（第 6 节）。

## 0. 读到的事实

### 0.1 集合与它的读写点 [读]

- 集合字段：`crates/singlefs-core/src/allocator.rs:2293`：「cluster_segments: BTreeSet<SlotNumber>,」，字段文档（`crates/singlefs-core/src/allocator.rs:2292`）：「段里的块被回收空了也仍算聚簇段：它还能被 `lowest_empty_segment` 重新开来装提交内生块。」
- 写集合只有一处。命令与原样输出（`crates/singlefs-core/src`、`crates/singlefs-mount-runtime`、`crates/singlefs-verification/src` 三处都搜）：

```
$ grep -rnE 'cluster_segments\.(insert|remove|clear|retain|extend|append|pop_first|pop_last|take|drain)' crates/singlefs-core/src crates/singlefs-mount-runtime crates/singlefs-verification/src
crates/singlefs-core/src/allocator.rs:3075:                self.cluster_segments.insert(segment_start);
crates/singlefs-core/src/compaction_plan.rs:2215:                    cluster_segments.insert(SlotNumber(start_slot));
```

  第二行是 `compaction_plan.rs` 测试模块里的局部变量（同一文件 2184 行 `let mut cluster_segments = BTreeSet::new();`）。重开时集合经 `Self::new` 清空：`crates/singlefs-core/src/allocator.rs:2674`：「let mut allocator = Self::new(devices);」。
- 读集合的地方（按谁在用）：
  - 数据落点跳过集合里的每一段：`crates/singlefs-core/src/allocator.rs:1329`：「let is_open_to_user_data = !cluster_segments.contains(&segment_start)」。
  - 准入「落得下」读数：`crates/singlefs-core/src/admission.rs:996`：「.unblocked_slot_pairs_outside_the_cluster_segments(」；判在 `crates/singlefs-core/src/admission.rs:1077`：「if room_on_the_device.slot_pairs_for_data_units < SlotPairCount(units.data_units) {」。
  - 抬 F 的上界：`crates/singlefs-core/src/mounted_session.rs:6069`：「.slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(」；用在 `crates/singlefs-core/src/mounted_session.rs:6090`：「JudgementThatFoundTheDeviceShort::SlotPairsOutsideTheClusterSegmentsForDataUnits => {」。
  - 整理选段排除它：`crates/singlefs-core/src/compaction_plan.rs:240`：「is_opened_in_this_mount: cluster_segments.contains(&start_slot),」（详见第 6 节）。
- 提交内生块这一侧 [读]：开放段有空槽就 bump（`crates/singlefs-core/src/allocator.rs:3010`：「if let Some(open) = self.open_segment {」）；bump 游标只往前，遇到被挡的槽整个跨度往后挪（`crates/singlefs-core/src/allocator.rs:3249`：「.any(|device| device.is_blocked_for_commit_generated(SlotNumber(slot)))」）；开新段取最低的全空段，不看集合（`crates/singlefs-core/src/allocator.rs:1410`：「self.used_per_segment[*segment] == 0」）；没有全空段就回落并把开放段置空，该段仍在集合里（`crates/singlefs-core/src/allocator.rs:3084`：「self.open_segment = None;」）。
- 数据和提交内生块判「一个槽能不能用」是同一套三种位（已分配、影子账隔离、抬 F 扣住）：`crates/singlefs-core/src/allocator.rs:1154`：「pub fn is_blocked_for_commit_generated(」；集合成不成员不进这三种位，所以集合不决定「什么槽此刻可复用」（决定它的是回收门槛，不是集合）。

### 0.2 任务 20071 被拒那一刻 [量]

日志 `~/.local/state/singlefs/task-server/logs/task-b0f6b921-20071.attempt-3.log` 原样三行（`sed -n '119p;120p;121p'`）：
```
E202 探针 label=refusal publishes=21987 segments=2288 cluster_segments=1614 clusters_in_area=1614 core_pairs_outside=0 core_pairs_outside_once_released=0 core_empty_segments=8 core_unblocked_slots=100916 commit_generated_fell_back=0 scan_outside_segments=674 scan_outside_full=674 scan_outside_empty=0 scan_outside_free_pairs=0 scan_cluster_free_slots=101389 scan_cluster_free_pairs=49770 scan_cluster_live_histogram=0:8,1:1416,16-31:1,2:140,3-7:46,8-15:3
E202 探针 label=after_remount publishes=21987 segments=2288 cluster_segments=1 clusters_in_area=1 core_pairs_outside=49503 core_pairs_outside_once_released=49804 core_empty_segments=32 core_unblocked_slots=100852 commit_generated_fell_back=40 scan_outside_segments=2287 scan_outside_full=674 scan_outside_empty=32 scan_outside_free_pairs=49804 scan_cluster_free_slots=3 scan_cluster_free_pairs=0 scan_cluster_live_histogram=32-63:1
E202 探针 remount_retry=published versions=1
```

读法 [量 + 算术，算术我用 bash 算的]：

- 2288 个段里 1614 个是这次挂载开过的聚簇段，674 个在集合外且全满（探针定义：`scan_outside_full` 是活槽数恰好 64 的段，见 `probe.patch` 里 `if *live_slots == LOCAL_CLUSTER_SEGMENT_SLOTS`）；集合外空槽对 0。
- 直方图 `0:8,1:1416,2:140,3-7:46,8-15:3,16-31:1`，合计 1614。活槽 1 的段，一个活槽只落在一个槽对里，所以每个这样的段恰好 31 个空槽对：1416 × 31 = 43896；活槽 0 的 8 个段 8 × 32 = 256；两类合 44152 对，占 `scan_cluster_free_pairs=49770` 的 89%。剩下 5618 对在活槽 2 到 31 的 190 个段里（我另算了上下界 5402 至约 5900，5618 落在里面）。
- 被拒的写要一个槽对：panic 行里 `demand: BytesOnOneDevice(32768)`（一个数据单元两槽 = 32768 字节）；`most_a_floor_raise_could_make_available: AvailableBytesOnOneDevice(0)`。
- 重挂之后同一次写发得出去：`remount_retry=published versions=1`；重挂后 `core_pairs_outside=49503`（集合只剩 1 个段时核心层的段外没挡槽对）。
- 聚簇段个数随发布次数 [量，探针行逐行取]：顺序写那半段 513 次发布 43 个、8225 次发布 618 个，斜率 (618−43)/(8225−513) = 0.0746 段每次发布；交错写那半段 9281 次发布 694 个、21953 次发布 1610 个，斜率 (1610−694)/(21953−9281) = 0.0723；全程 1614/21987 = 0.0734。折成槽：64 × 0.0734 ≈ 4.7 个提交内生块槽每次发布。线性、没有放缓。
- 还要写多少 [推]：装置的活数据单元 16384 个，穿插的垫片单元至多 `live_data_units / 2 + 32` 个（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:2178`：「live_data_units / 2 + data_units_of_a_cluster_segment()」），一个单元占一个槽对：数据总量至多 16384 + 8192 + 32 = 24608 对；已落进集合外 674 段 × 32 = 21568 对；差 3040 对。另一种估法：交错写计划 8192 步（日志 `planned_at_most=8192`），最后一条进度行 `done=6585`，剩 1607 步 × 每步两个单元 = 3214 对。两种估法都在约 3000 至 3200 对。
- 整理不在这次造池里跑 [读]：`construct_a_starting_image_on_the_clock` 一带（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/main.rs` 第 2381 至 2670 行）里搜 `ompaction|run_compaction|background`，只命中 2631 行 `session.compaction.quarantined_slots(),`，没有调整理入口。

### 0.3 条款现读（整行抄，标 kb 文件自己的行号）

- `.claude/kb/decisions/03-空间分配.md:167`：「第一版用户数据块不进聚簇段。落点」（已定项 8 第 1 条，七条合取里的「不在任何开放的聚簇段里」同行）。
- `.claude/kb/decisions/03-空间分配.md:168`：「聚簇段在挂载期间对用户数据关着」……「上一次挂载开过、装着提交内生块的段对用户数据开放，混放允许」（已定项 8 第 2 条）。
- `.claude/kb/decisions/03-空间分配.md:215`：「段起点 = 单元区内最低的、64 槽对齐的全空段」（已定项 10 ①）；同行 ②「用户数据的候选集加一条合取」。
- `.claude/kb/decisions/03-空间分配.md:147`：「候选落点不在任何开放的聚簇段里（只管用户数据）」（已定项 7 合取表第 6 行，唯一登记位）；`.claude/kb/decisions/03-空间分配.md:148`：「全空聚簇段数 ≥ N₀」（第 7 行）。
- `.claude/kb/decisions/03-空间分配.md:199`：「不许有假性 ENOSPC」，同行「只要 `df` 报出的空闲 ≥ s，写 s 字节就必须成功」（已定项 9 第 1 条，用户定案）。

## 问题 1：聚簇段对用户数据关着，护着的每一样

下表「稀疏段」指：这次挂载开过、已不是开放段、段里只剩一两个活的提交内生块；「全空段」指记录口径里一条未释放记录都没有（`crates/singlefs-core/src/allocator.rs:1255`：「pub fn empty_segments(&self) -> u64 {」）。每一格后面标 [读] 或 [推]。

| # | 护着的东西 | 出处 | 稀疏段还成不成立 | 全空段还成不成立 |
|---|---|---|---|---|
| 1 | 提交内生块在 key 空间上连续，固定点轮数有常数上界 | `.claude/kb/decisions/03-空间分配.md:100`：「必须从 key 空间上连续的聚簇段（bump 段）分配」；依据 E148、E81（`.claude/kb/decisions/03-空间分配.md:106`、`:107`） | **不成立**。连续靠的是 bump 游标在开放段里只往前走（`crates/singlefs-core/src/allocator.rs:3239` 起的 `bump_slot_in_open_segment`）；已关闭的稀疏段不再被 bump，数据落进它的空洞改不了任何一次 bump 的落点 [读]。仅有的触点是回落：回落在每块盘都空的槽里取（`crates/singlefs-core/src/allocator.rs:3101` 起），数据占了空洞，回落少一个候选 [读]，只在没有全空段时才走 | 成立（开放段不变；全空段就是下一个 bump 段，见 #2） |
| 2 | 数据不进 bump 段，否则提交内生块落不进聚簇段 | `.claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:10`：「92%–94% 落不进聚簇段」；`.claude/kb/decisions/03-空间分配.md:179`：「分得开候选的量是提交内生块进不进聚簇段」 | **不成立，且没量过**。E151 量的臂是「用户数据走 bump 段」，即数据和提交内生块共用同一个被 bump 的段；已关闭的稀疏段里放数据，装置里没有这一臂 [推：证据只覆盖开放段] | 成立。开新段只认全空段，数据占了一个全空段，这一格就是 E151 那一臂的缩小版 [推] |
| 3 | 全空段的供给：开新段、底额 N₀、整理起跑水位与停机 | `.claude/kb/decisions/03-空间分配.md:215`：「段起点 = 单元区内最低的、64 槽对齐的全空段」；`.claude/kb/decisions/03-空间分配.md:148`：「全空聚簇段数 ≥ N₀」；`.claude/kb/decisions/26-后台整理与放置回收.md:28`：「全空聚簇段数（物理量、带设备维）」；`.claude/kb/decisions/26-后台整理与放置回收.md:105`：「第一版就留常驻底额」 | 不涉及（它不是全空段，不进这个数）。把它还给数据不改全空段数 [读：数据只改槽位，`empty_segments` 按记录增量维护] | **成立，而且最要紧**。还给数据、数据落进去，全空段数当场少一（`crates/singlefs-core/src/compaction_plan.rs:498`：「pub const EMPTY_CLUSTER_SEGMENTS_FLOOR_OF_THE_FIRST_VERSION: u64 = 1;」，数到 0 前台写就被合取 7 拒） |
| 4 | 段「将来能整段空出来」 | `.claude/warnings/2026-09-23.md:13`：「保住聚簇的目的（段将来能整段空出来）」；`.claude/kb/decisions/03-空间分配.md:175`：「第 2 条混放的代价：装着用户数据的旧段要等段里的用户数据也释放才能整段空出来」 | **看那一两个活块的寿命，分配器分不出来**。寿命短的（马上换下）：成立，还给数据就失去一个将来的空段；长寿的（20071 里的 1416 个是被钉住的树叶，每 16 次发布一个，推）：这个理由不成立，段永远空不出来。分配器只有分配代（记录里的），没有按块的寿命，见 `crates/singlefs-core/src/compaction_plan.rs:127` 一带全是从记录现数活槽 [推] | 不适用（已经空了；它的价值在 #3） |
| 5 | 整理与聚簇的分工（R 不选这次挂载开过的段） | `crates/singlefs-core/src/compaction_plan.rs:84`：「资格五条全成立的段才算：不在这次挂载开过的聚簇段里」；`research/prompts/m4-gc-r1-main-verification.md:33`：「排开装着豁免单元的段与开放段」 | 判决 G2 只点名「豁免单元的段与开放段」，把整个集合都排开是实现的写法（用的是同一个 `cluster_segments`）[读]；攻方 K-b 的模型源码不在工作区（`research/prompts/m4-gc-r1-opus-model` 与 `…-sonnet-model` 下只有 `out`、`tools`），整个集合排开是不是必要没有读到 | 全空段在整理这一侧不是候选（扫过没有可搬的单元，`crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs:139` 一带的 `is_a_candidate`），与集合无关 |
| 6 | discard 按整段发 | `crates/singlefs-core/src/allocator.rs:1226`：「pub fn take_the_segments_to_discard(&mut self) -> Vec<SlotNumber> {」；`crates/singlefs-core/src/allocator.rs:1214`：「pub fn segment_is_reusable_as_a_whole(」 | 不依赖集合：整段可再分配才发，与集合成不成员无关 [读] | 同左；混放只减少整段空出来的机会 |
| 7 | 落点确定性：第一版落点唯一确定 | `.claude/kb/decisions/03-空间分配.md:170`：「第 1 条既然把第一版的落点唯一确定」 | 成立当且仅当新规则只读分配器的内存态（集合、开放段、位图）；这些都是历史的函数 [推]。规则读「后台整理此刻进度」则不成立（见丙） | 同左 |
| 8 | 崩溃一致性 | 集合只在内存：`crates/singlefs-core/src/allocator.rs:2289` 一带字段文档；`allocator.rs:2674` 重开经 `Self::new` 清空；三种位同一套（见 0.1） | 不依赖集合。落点只挑三种位都没置的槽；回收门槛（F 与环里最旧有效根）不看集合，所以数据进哪里都不会盖到还被某条根引用的槽 [读 + 推] | 同左 |
| 9 | 准入的读数与它要对上的落点规则 | `crates/singlefs-core/src/admission.rs:996`；`.claude/kb/decisions/03-空间分配.md:191`：「C1017（准入只数槽不数对）」 | 规则变，读数同变，否则放行了落不下、或拒了落得下 | 同左 |

### 小结：这条规则护着什么、没护着什么

- 全空段这一样（#2、#3）是规则真正护着的东西，量过的是 #2 的开放段那一半，#3 是条款本身的要求。稀疏段（#1、#2、#6、#8）上规则护着的东西不成立；只有 #4 取决于那个活块的寿命，而分配器今天分不出来。
- 规则有一样东西没护、反而伤着：`.claude/kb/decisions/03-空间分配.md:199` 的「不许有假性 ENOSPC」。关闭的稀疏段里的空槽，数据用不了，提交内生块的开新段也用不了（不是全空段），`df` 却把它们算空闲 [推：`df` 口径没定，C370；按槽数记账的统计里它们是空闲]。20071 是这条冲突的现实例子：被拒时 `scan_cluster_free_slots=101389`。
- 用户 2026-09-23 的定案（`.claude/kb/decisions/03-空间分配.md:184`：「用户定案 2026-09-23：一个聚簇段什么时候不再算」）当时接受的风险是跨挂载的混放退化（`.claude/warnings/2026-09-23.md:14` 的已知风险），没有写一次挂载之内段外槽对只降不升。

## 问题 2：候选与判据格八格

### 2.1 候选的定义（每条写死「谁在什么时候做什么」，免得各读各的）

- **甲 全空即还**：事件点是回收（`crates/singlefs-core/src/allocator.rs:1181`：「pub fn mark_reclaimed(&mut self, slot: SlotNumber, span: u64) {」，经 `reclaim_released_up_to`，`allocator.rs:3280`）：一个不是开放段的聚簇段回收之后记录口径全空，就从集合里摘掉。
- **乙 低于门槛即还**：同一个事件点，不是开放段的聚簇段活槽数 ≤ θ 就摘掉。两种读法要分开答：乙粘性（只在事件点判一次，摘掉之后数据落进去不再把它放回）与乙持续（「活槽数 ≤ θ 的聚簇段对数据开放」写成每次落点时现判的谓词）。
- **丙 整理搬空再还**：整理选中集合里的稀疏段做 R，搬空、删意图时把它摘出集合（需要放开第 6 节列的几处）。
- **丁 提交内生块复用稀疏段**：开放段装不下时，先在集合里找未挡槽数 ≥ T 的最低段重开（bump 照旧跳过被挡的槽），找不到再开全空段，再找不到回落。T 是新旋钮。
- **戊（新加）段外没落点才进集合**：用户数据的候选是一个全序：先集合外（起点最小，同今天），集合外没有两槽都空的落点时再取集合内、不是开放段的段（有活槽的段先于记录口径全空的段，各按起点最小）；当前开放的那一段永远不给数据。集合不再是「关着」，是「排在后面」。
- 组合：**戊 + 丁**（先戊后丁）；**戊 + 丙**（长期，丙补完之后整理才能收拾集合里的稀疏段）。甲、乙都不进组合，理由见下。

### 2.2 八格（上）：还得回多少、崩溃一致性、不变量与 checker、整理

| 候选 | 还得回多少（20071 被拒那一刻；换成长寿块钉段的历史） | 崩溃一致性 | 不变量与 checker | 整理 |
|---|---|---|---|---|
| 甲 | ≤ 256 对（8 个全空段 × 32，核心层 `core_empty_segments=8`，上界）；需求约 3040 至 3214 对，**不够**。这 8 个正是提交内生块下次开段的全部供给，数据一占，合取 7 的 N₀=1 先把前台写拒掉 [推]。钉段历史：0（没有全空段）。聚簇段个数照涨 | 盘上字节不改；落点取值变；重挂规则不变 | 池级 checker 不看集合；I-3.16 口径是记录口径，不动 | 停机谓词（记录口径全空段数）不依赖集合，成立；但整理搬空一段、数据同时占掉一段，差值 0，`the_completed_intent_yielded`（`crates/singlefs-core/src/compaction_plan.rs:1446`：「pub const fn the_completed_intent_yielded(」）判无产出就停 [推] |
| 乙粘性 θ=1 | ≤ 44152 对（含 8 个全空段；只数有活槽的是 43896），够 14 倍。θ=2 再加 ≤ 4340。钉 θ+1 个的历史：**0**，被拒照样发生。聚簇段个数照涨 | 同甲 | 同甲 | 摘出的稀疏段成了整理的首选候选（greedy 取活槽最少），但签发前空洞判只数装着用户数据的段（`crates/singlefs-core/src/compaction_plan.rs:1348`：「pub fn holes_outside_the_region_in_segments_holding_user_data<IsUnblocked>(」），被拒那一刻 0，签不出去 [推] |
| 乙持续 θ=1 | 数据在段里占 2 槽，活槽数 1 → 3 > θ，段被立刻放回集合：每段至多 1 对，≤ 1424 对，**不够**（< 3040）[推] | 同甲 | 同甲 | 同乙粘性 |
| 丙 | 被拒那一刻 **0**：整理是后台的，被拒路径不等它（`crates/singlefs-core/src/mounted_session.rs:6228`：「fn judge_before_pushing_a_floor_raise_for_a_refusal(」直接交回）；造池装置也不跑整理（0.2）。补齐三处之后：1416 个单元各在一个段里，每段至少两次发布（签发一次、带一批的最后一批一次，`.claude/kb/decisions/26-后台整理与放置回收.md:30`：「带一批的最后一批照旧排除 R」），约 2800 次发布起 [推]。钉段历史：钉住的、隔离的、不可搬的块搬不动，该段回 0 | 整理批的崩溃流已有；新增的只是「选中集合内的段」 | 同甲；需要在选段核的 spec 里改一条资格（`crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs:62`：「&&& !segment.is_opened_in_this_mount」） | 要改整理三处（第 6 节）。搬空后摘出才算数 |
| 丁 | 追溯不回任何一个。若 20071 从头按丁走：集合约 25 至 100 个段（推，公式见第 3 节），段外约 49 千对 [推]。大批删除之后集合里全是空洞的历史：**回不了**，空洞只给提交内生块 | 盘上字节不改；落点取值变（提交内生块的落点变）；重挂规则不变 | I-5.5 的开放段近似（树表单元所在的段）管的是 mkfs 水位那一类，在新挂载里集合为空，行为与今天相同 [推] | 全空段供给增多，整理起跑水位（全空段 < 5%）越线更少；停机谓词不变 |
| 戊 | 约 49.5 千对 [推：被拒时集合里的没挡槽对；参照量是重挂后 `core_pairs_outside=49503`，同一份盘面、集合清空；再减开放段的 ≤ 32 对]，够 16 倍。钉 h 个活块（h 到 63）的历史：每段 (64−h)/2 左右全可用，没有门槛 | 盘上字节不改；**只在「段外没落点」的历史上落点变**，其余历史落点逐字相同 [推]；重挂规则不变 | 同甲；新立的是分配器内部不变量（见第 4 节），不进池级 checker | 停机谓词不变；数据可落的集合内段多了，整理搬数据单元的目的地也跟着多（走 `try_allocate_user_data`，`crates/singlefs-core/src/transaction.rs:13806` 一带）；签发前空洞判仍只数装着用户数据的段，与戊的可达集不一致，后续换成同一个函数（不在这一轮） |

### 2.3 八格（下）：准入与上界、落点确定性、代价、规则护着什么

| 候选 | 准入与上界 | 落点确定性 | 代价（准入路径上新增的计算） | 规则护着什么（对第 1 节的行号） |
|---|---|---|---|---|
| 甲 | 集合变小、读数跟着；上界（`mounted_session.rs:6069`）同口径改；钉住靠「读数与落点读同一个集合」 | 成立：摘出只发生在回收事件点，是历史的函数 [推] | 事件 O(1)；读数仍 O(集合大小)，集合小了 | #3 全空段被送给数据，**直接破**；#1、#2 稀疏段不涉及 |
| 乙粘性 | 同甲 | 成立（事件点判一次） | 同甲 | #3 送出全空段时同甲；稀疏段 #4 取决于块的寿命 |
| 乙持续 | 同甲 | **不成立**：谓词在一次发布中途越过 θ，违反 `.claude/rules/fs-design.md:71`：「只在操作期间不会变的输入上分支。」 | 每次落点现数活槽，O(段数) | 同乙粘性 |
| 丙 | 同甲；另要把「摘出」限定在发布边界、分配器锁内 | **不成立**（前台落点依赖后台整理的进度，两次跑时序不同则不同）；限定在发布边界后降为「给定整条发布序列，落点是确定的」 [推] | 整理在后台，准入路径不变；整理选段多了集合内的段 | 稀疏段 #4 成立（搬走活块才还），全空段 #3 同甲 |
| 丁 | 不变（集合小了而已） | 成立（开段取最低的 ≥ T 段，BTreeSet 全序） | 开段时要找「未挡槽数 ≥ T 的最低聚簇段」：要么扫集合 O(集合大小)，要么新维护一份有序索引 O(log n)；`lowest_empty_segment` 今天就是线性扫（`crates/singlefs-core/src/allocator.rs:1408` 起的 `(0..full_segments).find`） | #1 变：连续性只剩「同一个 64 槽段内」，段内被钉的槽夹在中间，已定项 5 的固定点上界没有量过 [推]；#3 全空段留着，改善 |
| 戊 | 读数 = 未挡槽对总数 − 开放段的未挡槽对，O(1)；上界 = 记录口径空槽对总数 − 开放段的，O(1)；两处和落点用同一个函数，差分测试钉（第 4 节） | 成立：候选序是（类，起点）全序，类在操作开始时按集合与开放段定，中途不变；一次发布里数据先于提交内生块取落点（`crates/singlefs-core/src/admission.rs:1051`：「数据单元在这次的角色里排最前、先取落点」） | 读数从 O(集合大小) 降到 O(1)（今天 `crates/singlefs-core/src/allocator.rs:579` 的 `pairs_inside_the_cluster_segments` 每次遍历集合）；落点：集合外有落点时与今天同，段外耗尽时两趟走 O(集合大小)，之后可换成按类分开的有序索引 | #1、#2 稀疏段不涉及；#3 靠「有活槽的段先于全空段」与合取 7 守住；#2 的开放段永远不给；#4 在压力下放弃（没有别的出路时） |

### 2.4 为什么推荐戊，不推荐另四条（正推）

- 甲：还的是最该留的东西（#3），还得回的最少（≤ 256 对 < 3040），聚簇段个数照涨。四格里最弱。
- 乙：θ 是新的魔数（`.claude/rules/fs-design.md` 五条硬要求第 1 条要求切换判据显式可计算）；θ+1 个钉的历史回 0，被拒原样发生；持续读法违反分支约束 1；粘性读法在 20071 上够，但每次没压力的时候也在混放，白白放弃将来的空段（#4）。戊在同一段历史上回得一样多，且没压力时不混放。
- 丙：被拒路径不等整理，所以「被拒那一刻」答 0；补齐要改选段核的已证 spec 与签发前判，改动面最大；前台落点依赖后台时序。它是长期的补充（戊 + 丙），不是修这个缺陷的那一刀。
- 丁：治的是集合长得快，不治「数据已被关在外面」；大批删除之后集合里全是空洞，数据照样进不去，照样违反 `.claude/kb/decisions/03-空间分配.md:199`。它动已定项 10 ① 的「全空段」，要先量固定点（E148、E81 那一族）。
- 戊的弱处：它不改聚簇段的增长律（20071 的 0.073 段每次发布照旧，直到全空段耗尽后走回落，回落的代价债是 C1018，`.claude/kb/decisions/03-空间分配.md:191`）。所以丁留作第二步。

## 问题 3：任务 20071 被拒那一刻各条还得回多少、够不够；聚簇段个数随发布次数怎么涨

### 3.1 先把「能回多少」的两个端点算死 [量 + 算术]

- 集合里按记录口径的空槽 `scan_cluster_free_slots=101389`；核心层没挡的槽 `core_unblocked_slots=100916`，这一行集合外没有空槽对、集合外空槽也没有（674 段全满），所以集合里没挡的槽就是 100916（单元区末尾凑不满一段的零头至多 63 槽，下界最多再少约 32 对，不改量级）。差 101389 − 100916 = 473 个槽记录口径空、却被挡着（defer、隔离、抬 F 扣住）。一个被挡的槽至多弄坏一个槽对，所以集合里「现在就没挡」的槽对数在 49770 − 473 = 49297 与 49770 之间（`scan_cluster_free_pairs=49770` 是记录口径，含被挡的）。
- 集合里的活槽 = 1614 × 64 − 101389 = 1907 个（量的两个数相减）。
- 需求 3040 至 3214 对（0.2 节两种估法）。

### 3.2 各条在被拒那一刻

| 候选 | 回得多少（对） | 够不够 | 聚簇段个数随发布次数 |
|---|---|---|---|
| 甲 | ≤ 256 | 不够（≤ 8% 的需求），且把全空段送给数据 | 不变：线性 0.073 段每次发布 |
| 乙粘性 θ=1 | ≤ 44152 | 够（14 倍） | 不变：线性 |
| 乙持续 θ=1 | ≤ 1424 | 不够 | 不变：线性 |
| 丙 | 被拒那一刻 0；补齐之后后台慢慢回 | 前台这一次写不等它，不够 | 搬空之后集合缩小，但增长率不变，要靠整理持续追 |
| 丁 | 追溯 0；从头走则段外约 49 千对 | 够 | **次线性**：见 3.3 |
| **戊** | **49297 至 49770 减开放段的 ≤ 32，即 ≥ 49265** | **够，约 15 倍以上** | 不变：线性 0.073 段每次发布，直到全空段耗尽（被拒时还剩 8 个）；耗尽之后提交内生块走回落、集合不再长 |
| 戊 + 丁 | 同戊 | 够 | 次线性（3.3），全空段不被提交内生块耗尽 |

「不变：线性」的依据是 0.2 节的量：两半段斜率 0.0746、0.0723、全程 0.0734，没有放缓 [量]；戊不碰开段规则，增长律不变 [推]。

### 3.3 丁让个数怎么涨 [推，两个前提写明]

前提一：钉住的块（长寿的提交内生块）不动，每个只占一个槽。前提二：丁的开段能找到「未挡槽数 ≥ T」的集合内段，且钉住的块集中进已有的段。

- 下界：集合里的活槽 L = 1907 个（3.1），每段最多 64 槽，至少 ⌈1907 / 64⌉ = 30 个段，才装得下今天已有的活块。
- 估计：个数 ≈ (L + 窗口内还没回收的槽数) / (64 − T) + 1。窗口内的槽数 ≈ 4.7 × W，W 是一个块从换下到可回收之间隔的发布数，等于根环里根的数目（根环每区槽数 S 的默认值我没有读到，见「没做什么」），所以窗口项是待量的参数；L 随发布次数线性涨：1907/21987 = 0.087 槽每次发布（量的活槽数除以量的发布次数，两个量都来自 0.2 节的探针行；推：把它外推成斜率），折成段 0.087/64 ≈ 0.0014 段每次发布，比今天的 0.0734 小约 50 倍。
- 换历史：钉 h 个活块的段，丁把 h 摊进已有的段，个数 ≈ 钉的总数 / (64 − T)，与 h 无关；今天是「每个钉占一个段」。

### 3.4 换一段历史（长寿块钉段）各条还成不成立

| 每段钉 h 个活块 | 甲 | 乙粘性 θ=1 | 乙粘性 θ=32 | 戊 | 丁 |
|---|---|---|---|---|---|
| h = 1 | 0 | 31 对每段 | 31 对每段 | 31 对每段 | 钉被集中，集合 ≈ 钉数 / 63 |
| h = 2 | 0 | **0** | 约 30 对每段 | 约 30 对每段 | 同上 |
| h = 33 | 0 | **0** | **0**（33 > 32，段里还有 31 个空槽、约 15 对被锁着） | 约 15 对每段 | 同上 |
| 全部是长寿钉、无空槽 | 0 | 0 | 0 | 0 | 0（空间确实没有了） |

读法：乙的任何 θ 都有一个历史让它回 0 而盘上还有空槽（违反 `.claude/kb/decisions/03-空间分配.md:199`）；戊没有门槛，回多少等于盘上真有多少。每格是槽对按「每个钉占一个槽对」算 [推]。

## 第 4 节 写成对实现的改动（背景材料第 5、6 条）

背景材料六条与本报告的对应：1 → 问题 1；2 → 问题 2；3 → 问题 3；4（对已有保证的影响）→ 2.2、2.3 两张表与第 5 节；5（准入与上界）与 6（写成改动）→ 本节；派发第 5 条（整理现读）→ 第 6 节。下面写的是**推荐戊**的改动；丁放在 4.6。改动都是对 `crates/` 今天实现的改，没有补丁、没编译，行号是今天读到的。

### 4.1 `crates/singlefs-core/src/allocator.rs`：改哪一步

1. **候选序，改 `DeviceFreeMap::lowest_user_data_slot_outside_the_region`**（`crates/singlefs-core/src/allocator.rs:1320` 起；今天只有一趟，跳过集合里每一段，`allocator.rs:1329`）。加一个参数 `open_segment: Option<SlotNumber>`，分两趟：
   - 趟一：同今天，原样保留；有答案就返回。
   - 趟二（只在趟一得 `None` 时走）：按段号升序走 `segments_with_an_unblocked_slot_pair`（`allocator.rs:485` 声明，`set_blocking_bit` 增量维护，`allocator.rs:748`），取起点在集合里、不等于 `open_segment`、不在活整理意图的 R 里、且 `unreleased_per_segment` 非 0（有活槽，`allocator.rs:445`）的第一段，交回那一段的 `lowest_unblocked_slot_pair_in_the_segment`；没有再取 `unreleased_per_segment` 为 0 的第一段（记录口径全空，同 `empty_segments`）。
   - 不动 `lowest_empty_segment`（`allocator.rs:1402`）、bump（`allocator.rs:3233`）、回落（`allocator.rs:3084`）、集合唯一的写点（`allocator.rs:3075`）。
2. **`PoolAllocator::try_allocate_user_data`**（`crates/singlefs-core/src/allocator.rs:2925`，今天在 2929 行取 `&self.cluster_segments` 传下去）：多传 `self.open_segment`；各盘一致才给号的 `agreement_across_devices` 不动；新加一个只增的计数「数据落点进了聚簇段的次数」（`.claude/rules/fs-design.md` 五条硬要求第 4 条分支可观测，D26 已定项 6 的观测点里加一格）；新加一个只供测试的开关（硬要求第 2 条）：产品值「先段外后段内」，测试值「聚簇段先于段外」，用来在小几何上强制进入新分支。
3. **读数与上界的两个函数改成「可达」**：`unblocked_slot_pairs_outside_the_cluster_segments`（`allocator.rs:870`）与 `slot_pairs_without_an_unreleased_record_outside_the_cluster_segments`（`allocator.rs:893`）今天都是「总数减集合内各段之和」，每次遍历集合（`allocator.rs:579` 的 `pairs_inside_the_cluster_segments`）。改成「总数减开放段那一段」（`unblocked_slot_pairs_per_segment[开放段]`，`allocator.rs:633` 起建立的那张表），O(1)；顺手把活整理意图的 R 那一段也减掉（今天读数不减 R，落点却排除 R，是今天就有的松，见 4.3）。两个函数改名（名字里没有「集合外」了），`pairs_inside_the_cluster_segments` 若再没有调用方就删。
4. 可选的第二步（不进第一刀）：把「是不是聚簇段」做成 `DeviceFreeMap` 里每段一个标志，趟二按类分开维护有序集，让趟二也 O(log n)；今天趟二最坏扫集合一遍，只在压力下才走。

### 4.2 `crates/singlefs-core/src/admission.rs`：改哪一判

- `PlacementRoomOnOneDevice::of_every_device_of_the_allocator`（`crates/singlefs-core/src/admission.rs:989` 起，`admission.rs:996` 调今天的集合外读数）：改调 4.1 第 3 点的可达读数，传 `allocator.open_segment()`（访问器在 `allocator.rs:3748`）与活整理意图的 R。字段 `slot_pairs_for_data_units`（`admission.rs:978` 起的文档）改写成「数据单元落得下的槽对：两槽都没挡、起点偶数、不在开放段里」。
- 判 ①（`crates/singlefs-core/src/admission.rs:1077`：「if room_on_the_device.slot_pairs_for_data_units < SlotPairCount(units.data_units) {」）的式子不动，输入变了。
- 拒因成员 `SlotPairsOutsideTheClusterSegmentsForDataUnits`（`crates/singlefs-core/src/admission.rs:845`，同名在 `admission.rs:1086`、`admission.rs:2537`、`mounted_session.rs:6090`）改名成说「数据单元可达的槽对」的名字（名字里不再说集合外）；用到它的测试文件现读命中 4 个：`accept_raises_the_floor_for_space.rs`、`unequal_devices.rs`、`admission_refusals_name_the_judgement_that_found_each_device_short.rs`、`admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments.rs`（都在 `crates/singlefs-verification/tests/` 下）。
- 判 ② 不动：`crates/singlefs-core/src/admission.rs:1091` 起的减法只用没挡的槽总数，可达对集合仍是没挡槽对的子集，那句 `expect` 仍真。
- 调用点：发布路径 `crates/singlefs-core/src/transaction.rs:13423`（「&PlacementRoomOnOneDevice::of_every_device_of_the_allocator(allocator),」）不用改，走同一个构造函数。

### 4.3 `crates/singlefs-core/src/mounted_session.rs`：「抬 F 全放出来也放不下」的上界

- `SlotsAFloorRaiseCouldRelease::of_the_device`（`crates/singlefs-core/src/mounted_session.rs:6064` 起）：参数 `cluster_segments` 换成 `open_segment`（与 R）；字段 `slot_pairs_once_everything_is_released`（`mounted_session.rs:6060`）的文档（`:6057` 起）改成「开放段之外、两槽都不罩在未释放记录下的偶数起点槽对」。
- `devices_short_even_with_everything_a_floor_raise_could_release`（`mounted_session.rs:6118` 起，`mounted_session.rs:6138` 传 `allocator.cluster_segments()`）：改传 `allocator.open_segment()`。
- `most_a_floor_raise_could_make_available`（`crates/singlefs-core/src/mounted_session.rs:6079`）的换算不动（槽对 × 一个数据单元的槽数）。
- 「宁大勿小」仍成立：上界按「记录口径的槽对总数减开放段」算，落点保证不进开放段，所以上界不小于抬 F 放开之后真能分到的对数；推串时提交内生块可能新开一个聚簇段把一个全空段变成开放段，使上界偏大，与 `allocator.rs:887` 一带的文档「只会比真放开之后成的对数多，不会少」同一个方向 [读]。
- 今天就有的松（不是这一改引入的）：准入读数不扣活整理意图的 R（`admission.rs:996` 一带只传集合），落点却排除 R（`allocator.rs:1330`）；改成「可达」函数时把 R 一并扣掉，读数与落点才逐对相等。

### 4.4 两处与分配器真实行为怎么钉成一致

1. **同一个函数**：读数、上界、落点两趟都经 `DeviceFreeMap` 里同一组可达函数，不在准入与会话里各写一份「集合外」的算法（今天准入 `admission.rs:996` 与上界 `mounted_session.rs:6069` 各调一个函数，函数之间靠注释对齐）。
2. **差分测试**：今天已有三组把增量计数与按记录逐段扫出来的数对拍的测试（`crates/singlefs-core/src/allocator.rs:3915` 的 `unblocked_slots_and_pairs_outside_the_cluster_segments_by_scan` 与它的调用，`:4052` 一带，`:4188` 一带）。照同样形状加一组：随机历史（开段、回落、释放、回收、隔离、抬 F 扣住、活 R 各占一部分）的每一步，断言「可达读数 == 在分配器拷贝上连续调 `try_allocate_user_data` 直到被拒的成功次数」，与「三样全放开之后的上界 ≥ 同法数出的次数」。连续调用到被拒的形状今天就有一份（`crates/singlefs-core/src/mounted_session.rs:8546` 的 `allocate_every_slot_pair`），只覆盖一个聚簇段。
3. **端到端用例**：`crates/singlefs-verification/tests/admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments.rs` 今天钉的是「集合外没有落点就在任何写之前拒」；改成「集合外与集合内都没有落点才拒」，并新加「集合外耗尽之后数据进集合内、有活槽的段先于全空段、开放段永远不进」三格。
4. **复现**：20071 的原负载（E202 装置，2288MiB 档，16384 个活数据单元）在带改动的副本上原样重跑，判「不再报 `SlotPairsOutsideTheClusterSegmentsForDataUnits`」；按 `.claude/rules/three-way-inference.md` 的零轮改法 A/B 要求，这一改法对「被打中的那一支」要有改前改后两臂（改前 = 调查员那次的探针版，已有读数；改后 = 带戊的同一探针版）。这一臂我没有做，写进「没做什么」。

### 4.5 条款改句（推荐戊；改句是我推的措辞，定不定归用户）

每处先抄原句（逐字，kb 文件自己的行号），再写改句。

**D3（空间分配） 已定项 8 第 2 条**，`.claude/kb/decisions/03-空间分配.md:168`，被改的是开头这一整段（后面从「提交内生块的段耗尽时」起的回落政策一字不动）：

```
2. **聚簇段在挂载期间对用户数据关着**：这次挂载开过的聚簇段（当前开放的那一段，以及回落之前开过的段）都不给用户数据（已定项 10 ② 的「任何开放的聚簇段」指的就是这些段）；重开之后（bump 指针与「这次挂载开过哪些段」都只住内存，挂载后新开一段，已定项 10 ①），上一次挂载开过、装着提交内生块的段对用户数据开放，混放允许。
```

改句：

```
2. **聚簇段在挂载期间对用户数据排在后面**：这次挂载开过的聚簇段（当前开放的那一段，以及回落之前开过的段）排在用户数据候选序的后面：用户数据先在这些段之外取落点（第 1 条）；段外没有两槽都空的落点时，才在这些段里取——当前开放的那一段永远不给；有未释放记录的段先于记录口径全空的段（全空段留给提交内生块开新段与合取 7 的 N₀）；同一类里起点槽号最小的先。重开之后（bump 指针与「这次挂载开过哪些段」都只住内存，挂载后新开一段，已定项 10 ①），上一次挂载开过、装着提交内生块的段与别的段一样对用户数据开放，混放允许。
```

**已定项 8 第 1 条**，`.claude/kb/decisions/03-空间分配.md:167`：被改的是七条合取里的一句——原句「起点槽号最小、起点 32768 对齐、跨度内两槽都空、不在任何开放的聚簇段里、整跨度都过分配准入合取（七条，唯一登记位是已定项 7）的落点」，改成「……两槽都空、不在当前开放的聚簇段里（这次挂载开过的别的聚簇段按第 2 条排在后面）、整跨度都过……」。

**已定项 7 合取表第 6 行**（「唯一登记位」，`.claude/kb/decisions/03-空间分配.md:147`），原行：

```
| 6 | 候选落点不在任何开放的聚簇段里（只管用户数据）| 已定项 10 ②；提交内生块正是从开放段 bump，不受这一条约束 |
```

改成：「候选落点不在当前开放的那一段里；这次挂载开过的别的聚簇段只在段外没有落点时才进候选（只管用户数据）」，出处列「已定项 8 第 2 条、已定项 10 ②」。这一行是合取的唯一登记位，不改它就是两处各说各的。

**已定项 10 ②**，`.claude/kb/decisions/03-空间分配.md:215`，原句：

```
② 用户数据的候选集加一条合取「候选落点不在任何开放的聚簇段里」。
```

改成：「② 用户数据的候选序：集合外先于集合内；开放的那一段不进候选集；集合内其余的段按已定项 8 第 2 条排在后面。」已定项 10 ① 在戊下不动（丁做的时候才改，见 4.6）。

**已定项 8 射程里的混放句**，`.claude/kb/decisions/03-空间分配.md:175`，原句：

```
⚠️ 第 2 条混放的代价：装着用户数据的旧段要等段里的用户数据也释放才能整段空出来，提交内生块的聚簇随挂载次数退化；现象是全空聚簇段数随挂载次数单调下降、回落次数上升。
```

后面补一句：「同一次挂载之内的混放只在段外没有落点时发生，次数是 D26 已定项 6 里新加的观测量『数据落点进了聚簇段的次数』。」**依据**一节加一条：「无实验：被拒那一刻的量来自任务 20071 的探针行；改法在探针上的前后两臂还没有（零轮）」。**欠**一节：C146 第 ① 条不变，新记一笔：「条款已把一次挂载之内的混放放进来，混放对提交内生块聚簇的代价没有量」。

**D26 已定项 1 第 2 条与已定项 3 不改**：停机谓词按记录口径数全空聚簇段，不看集合（`crates/singlefs-core/src/compaction_plan.rs:1435`：「pub fn region_is_empty_by_the_records(」）。

### 4.6 丁（第二步，另立一轮；先写清楚要改哪里，不推荐这一刀就做）

- `crates/singlefs-core/src/allocator.rs:3015` 起「开放段没有或装不下」那一步：每块盘的去处（`commit_generated_answer_without_open_segment_outside_the_region`，`allocator.rs:1470`）加一个成员（`CommitGeneratedDeviceAnswer` 在 `allocator.rs:190`）：「重开集合内未挡槽数 ≥ T 的最低聚簇段」，排在「开全空段」之前；各盘仍要答同一个段起点才分配（`agreement_across_devices`）。bump 本身不用改：`bump_slot_in_open_segment`（`allocator.rs:3233`）今天就逐槽跳过被挡的槽。
- 找「未挡槽数 ≥ T 的最低聚簇段」要有序索引（`set_blocking_bit` 里维护），否则是线性扫集合；`lowest_empty_segment` 今天就是线性扫（`allocator.rs:1408`），这是已有的债，不要再添一份。
- 条款：已定项 10 ① 的「段起点 = 单元区内最低的、64 槽对齐的全空段」与已定项 8 第 2 条的「最低的全空段开新段，没有全空段就答回落」两处改句；T 是新的魔数，要写成显式可计算的判据并留测试开关（`.claude/rules/fs-design.md` 五条硬要求第 1、2 条）。
- 先要量的：固定点轮数与每次发布重写的分配记录树叶数，在「提交内生块被段内钉住的槽隔开」的段里与连续 bump 的段里对比（E148、E81 那一族，`.claude/kb/decisions/03-空间分配.md:106`）；没量之前这一刀对已定项 5 的上界没有证据 [推]。

### 4.7 不变量

- **池级 checker（读盘的那一套）不改**：集合只在内存。`crates/singlefs-checker/invariants/` 里写「聚簇」的现读命中两个文件各一处（`grep -c '聚簇' *.md` 非零的只有 `05-空间记账.md` 与 `09-inode树结构.md`）：I-3.16 的口径是记录口径，与集合无关；I-9.9 说的是重新聚簇，是另一回事。I-5.5 在 `07-分配器.md` 里写「开放段」并近似为树表单元所在的段，该行已知限度里的「提交内生块回落时会误排」那一句管的是回落；戊在段外耗尽的极小几何上可能让 t1 落进集合内的段，这一格我没有量，写成「推：只有单元区只有几个段的几何才会走到；要在崩溃点重放的小几何上走一遍确认」，并在 I-5.5 的已知限度里补一句。
- **新立的是分配器内部不变量**（运行时与测试里断言，不进池级 checker，因为 checker 看不到集合），名字写它说什么：
  - 数据落点不在开放段；
  - 数据落点在集合内，则这一刻集合外没有两槽都没挡的槽对；
  - 数据落点在集合内且段是记录口径全空，则这一刻集合内有活槽的段里没有没挡的槽对；
  - 准入的可达读数等于「在分配器拷贝上连续取数据落点直到被拒」的成功次数（4.4 的差分测试）。
- **崩溃点重放**：不新增盘上写的种类；要补的是让新分支被走到的用例——小几何上段外耗尽之后继续写的历史（硬要求第 3 条「每条分支单独验，包括崩溃点重放」）；今天登记的崩溃用例里有多少走到「段外没落点」的拒绝，我只读到命中 `SlotPairsOutsideTheClusterSegmentsForDataUnits` 的测试文件是 4 个，没有逐个看它们走不走崩溃枚举，待主 agent 查。

## 第 5 节 对已有保证的影响（背景材料第 4 条；推荐戊）

八格里对应的几格已在 2.2、2.3 逐候选写了，这里只写戊的收口与两个容易漏的点。

- **崩溃点重放的状态空间**：盘上字节不改，写的种类不增。落点取值只在「段外没落点」的历史上变，所以这些历史在今天是「拒绝于某一步」、戊之后是「继续往下发布」：可达状态只增不减 [推]；其余历史逐字相同。小几何（崩溃放量用的单元区只有几个段）段外很快耗尽，会走到新分支，所以必须有强制进入的测试开关与计数（4.1 第 2 点）。重挂之后集合清空，趟一等于全单元区，与今天重挂后的落点规则相同 [读：`allocator.rs:2674`、`allocator.rs:2357`]。
- **池级 checker**：不改（4.7）。
- **整理的停机谓词**：按记录口径数全空聚簇段，不看集合，原样成立（`compaction_plan.rs:1435`、`compaction_plan.rs:1446`）。唯一的耦合是「无产出就停」：压力下数据落进集合内的全空段（排在最后一类）会拉低全空段数；戊把全空段排在最后，只有有活槽的段都没有空洞之后才动它，这时前台写的合取 7 先拒（`N₀=1`，`compaction_plan.rs:498`），这是 D26 已定项 3 的底额按设计在拦，不是新的拦法 [推]。
- **落点确定性**：成立（2.3）。两个容易漏的点：（一）候选序的类（集合外、集合内有活槽、集合内全空）要在操作开始时按集合与开放段定，一次发布里数据先于提交内生块，所以提交内生块在同一次发布中途开新段不会改数据已经定下的类；（二）盘不等大时各盘的类可以不同（大盘还有段外落点、小盘没有）：各盘答案不同，今天的拒绝换成「各盘落点不同」那一支（`PlacementRefusal::UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported`，`crates/singlefs-core/src/allocator.rs:2956`），仍是在动任何状态之前拒绝，与 `.claude/kb/decisions/03-空间分配.md:173` 的「第一版盘不等大」限制同一类，不新增坏情形。
- **代价**：准入路径从 O(集合大小) 变 O(1)（2.3）；这是戊顺带的好处，不是理由。

## 第 6 节 整理现读：搬空一个聚簇段之后那一段还算不算聚簇段（派发第 5 条）

结论：**调查员「整理在本次挂载里也还不回来」坐实，而且比她推的更强——今天整理连这次挂载开过的聚簇段都不选，「搬空之后算不算」在现行码里没有发生的场合；放开选段之后，搬空的段仍留在集合里。**逐处：

1. 选段资格排开集合内的段：`crates/singlefs-core/src/compaction_plan.rs:84`：「资格五条全成立的段才算：不在这次挂载开过的聚簇段里」；输入转换 `crates/singlefs-core/src/compaction_plan.rs:240`：「is_opened_in_this_mount: cluster_segments.contains(&start_slot),」。选段核的 spec（已证）里同一条：`crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs:62`：「&&& !segment.is_opened_in_this_mount」，执行体 `crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs:139`：「let is_a_candidate = !segment.is_opened_in_this_mount」。 [读]
2. 实际调用传的就是分配器的集合：`crates/singlefs-core/src/mounted_session.rs:5500` 与 `crates/singlefs-core/src/mounted_session.rs:5510`（`allocator.cluster_segments(),` 两处，CPU 与 GPU 两条）。 [读]
3. 测试开关指定区域的那条路也排开：`crates/singlefs-core/src/mounted_session.rs:5452`：「.contains(&forced_region.start_slot())」。 [读]
4. 集合没有出口：写集合的只有 `crates/singlefs-core/src/allocator.rs:3075`（0.1 的 grep 原样输出）。所以即便一段被搬空，它仍在集合里、仍不给数据；它还能被 `lowest_empty_segment` 重新开来装提交内生块（`allocator.rs:2292` 的字段文档、`allocator.rs:1410` 的开段条件），重开时 `insert` 是幂等的。 [读]
5. 反过来，整理搬空的是集合外的段（旧挂载开过的段、数据段）；这样的段只有被 `lowest_empty_segment` 选中开成开放段那一刻才进集合（`allocator.rs:3075`）。 [读]
6. 就算放开第 1 至 3 处，签发前那一判也过不去：`crates/singlefs-core/src/mounted_session.rs:5581`：「if !holes_admit_the_issue(holes, units_to_move) {」，`holes` 数的是「装着用户数据的段」里没挡的槽（`crates/singlefs-core/src/compaction_plan.rs:1348`：「pub fn holes_outside_the_region_in_segments_holding_user_data<IsUnblocked>(」）。被拒那一刻集合外 674 段全是活槽 64（`scan_outside_full=674` 的探针定义，见 0.2），所以 `holes` 是 0，而稀疏段里要搬的单元 ≥ 1，停在 `HolesOutsideTheRegionFewerThanItsUnitsToMove`（`mounted_session.rs:5583`）。 [读 + 量；「`holes` 是 0」是推：探针没印单槽空洞数，但 674 段都满 64 活槽]
7. 要搬的单元类是可搬的：索引节点在可搬类里（`crates/singlefs-core/src/region_scan.rs:227`：「IndexNode,」），搬迁目的地对节点走提交内生块的开段 bump（`crates/singlefs-core/src/transaction.rs:13810`：「allocator.try_allocate_commit_generated_preferring_the_rewrite_set(」）。 [读]
8. 造池装置不起整理（0.2 节，`e202_gpu_cpu_gc_comparison.rs` 第 2381 至 2670 行无调用）。 [读]

所以丙要成立要补四样：选段资格里把集合内非开放段放进候选（改已证的核 spec 与输入转换）、测试开关那条路同改、签发前空洞判换成与单元类对应的目的地可达数（节点的目的地是开放段与回落，不是用户数据的空洞）、搬空之后把段摘出集合（今天没有这一步）。再加：被拒路径不等整理（`judge_before_pushing_a_floor_raise_for_a_refusal`，`mounted_session.rs:6228`），所以丙只能是长期的补充，不能让被拒那一刻的写成功。

## 什么现象会推翻各条结论

| 结论 | 推翻它的现象（这一轮做得到的观测） |
|---|---|
| 集合只进不出 | 在 `crates/` 里找到 `allocator.rs:3075` 之外的写集合处（0.1 的 grep 命中第二处，或出现一个返回 `&mut BTreeSet` 的访问器；现读 `cluster_segments()` 只有 `allocator.rs:3754` 一个、返回 `&`） |
| 戊在被拒那一刻回 ≥ 49265 对 | 带戊的探针版重放 20071 的负载，被拒那一刻的可达读数小于 3040；或重放仍被拒 `SlotPairsOutsideTheClusterSegmentsForDataUnits`（改名之后是对应的成员）。「49297 至 49770」这个下界依赖 `scan_cluster_free_slots`、`core_unblocked_slots` 两个量的相减，探针行里另一个时点的同名读数对不上它就作废 |
| 戊在没压力时与今天逐字相同 | 4.4 第 2 点的差分测试里，旧分配器与戊的分配器喂同一段随机历史，在「段外有落点」的某一步上落点不同 |
| 甲 ≤ 256 对、乙持续 ≤ 1424 对 | 被拒那一刻直方图 `0` 档不是 8（已量为 8），或乙持续的谓词只数提交内生块的活槽、不数数据（分配器没有按类的槽计数，`allocator.rs:445` 的 `unreleased_per_segment` 不分类，要加才行） |
| 丙今天走不通（第 6 节） | 放开 `compaction_plan.rs:240` 的排除之后，现有用例在「集合外没有空洞」的盘上仍签得出 R；或 `holes` 在被拒那一刻不是 0 |
| 丁让个数次线性（3.3） | 钉住的块不集中：模型里 T 取得大、或 W 大到窗口项 4.7 × W / (64 − T) 压过 L / (64 − T)；或钉住的块并非每个只占一槽 |
| E151 只覆盖开放段 | 装置里找到一臂「数据落进已关闭的稀疏段」且量到提交内生块回落率升高；我只读了 `.claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:10` 与几处 grep，没有读全页 |
| 稀疏段里的活块多数是长寿的（#4） | 在探针行之外逐段量活块的分配代：被拒那一刻稀疏段里的活块多数分配代很新（几次发布之内要换下），则 #4 对这些段成立 |
| 整理不选集合内的段 | 现有用例里整理开着、R 落在 `allocator.cluster_segments()` 里 |

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 问题 1 规则护着什么 | 全空段上两样成立，稀疏段上一样取决于寿命 | 全空段是提交内生块开新段的供给与合取 7 的底额（成立）；稀疏段上连续性与 E151 的量不成立，「将来空出来」取决于活块寿命，分配器分不出 |
| 问题 2 候选与推荐 | 推荐戊，丁留第二步 | 甲还最少且送走全空段；乙的门槛在钉 θ+1 个的历史回 0、持续读法违反分支约束；丙今天四处走不通、被拒路径不等它；丁不治数据被关在外面 |
| 问题 3 还得回多少 | 戊够，甲、乙持续、丙不够 | 戊 ≥ 49265 对对需求 3040 至 3214；甲 ≤ 256；乙粘性 ≤ 44152、乙持续 ≤ 1424；丙 0；个数：戊不改增长律（线性 0.073），丁次线性 |
| 问题 4 改实现与条款 | 已写成改哪一步与改句 | `allocator.rs` 的趟二与两个可达读数、`admission.rs` 的读数构造与拒因改名、`mounted_session.rs` 的上界入参、已定项 8 第 1、2 条与已定项 7 第 6 行与已定项 10 ②；不变量只新立分配器内部的四条 |
| 问题 5 整理现读 | 坐实，且更强 | 整理连集合内的段都不选（`compaction_plan.rs:240`），集合没有出口，签发前空洞判在被拒那一刻也是 0 |
| 格：规则护着什么 | 戊守住全空段 | 有活槽的段先于全空段、开放段永不给；稀疏段本来不被护 |
| 格：还得回多少 | 戊 ≥ 49265 对 | 钉 h 个的历史上每段约 (64−h)/2 对，无门槛 |
| 格：崩溃一致性 | 不改盘上字节 | 只在段外没落点的历史上落点变，可达状态只增；要补强制进入的开关与小几何用例 |
| 格：不变量与 checker | 池级 checker 不改 | 新立四条分配器内部不变量；I-5.5 已知限度补一句（小几何，推） |
| 格：整理 | 停机谓词成立 | 数据的集合内落点只在压力下发生；签发前空洞判与戊的可达集不一致，后续另改 |
| 格：准入与上界 | 两处与落点同一个可达函数 | O(1) 读数；差分测试钉住；顺手补今天不扣 R 的松 |
| 格：落点确定性 | 成立 | 候选序（类，起点）全序，类在操作开始时定 |
| 格：代价 | 准入路径 O(1) | 从 O(集合大小) 降下来；趟二最坏扫集合，只在压力下走 |

## 要主 agent 跑的命令

无。本腿没写模型、没写补丁；做过的只有只读的 `grep`、`sed`、`ls`、bash 算术与 `python3 scripts/kb/replace-once.py`（改自己的报告）。戊的差分测试与 20071 负载的前后两臂都要等补丁，写补丁归实现员，不在这一轮的范围里。

## 没做什么

- 没跑任何项目代码、没交任务，报告里所有「推」都没有跑出来；「量」只指日志与产物里已有的读数，我只做了加减乘除。
- 没写模型。第 3.3 节的丁的个数、第 3.4 节的各 h 的槽对数都是手算，没有计数模型支撑；丁的窗口项（根环每区槽数 S 的默认值）没读到，W 是未知参数。
- 没改 `crates/`、kb 与任何规则，没提交；改句是建议措辞，不是写回。
- 没读 E151、E93 的全页，只读了 E151 第 10 行与两处 grep、D26 第 38 行；E151 只覆盖开放段这一条因此是推。攻方 K-b 的模型源码不在工作区（`research/prompts/m4-gc-r1-opus-model`、`…-sonnet-model` 下只有 `out`、`tools`），整个集合排开对整理是不是必要没有读到。
- 没查 `df` 把不把集合内的空洞算空闲（C370）；「违反已定项 9 第 1 条」按槽数记账的统计口径推。
- 没逐个看登记的崩溃用例里有几个走到「段外没落点」，只数到命中拒因成员的测试文件是 4 个。
- 没量钉住的块到底是什么（调查员推的是写完的文件最后一片 extent 叶，`LIVE_DATA_UNITS_PER_FILE` 是 16，`e202_gpu_cpu_gc_comparison.rs:2158`，与 1/16 ≈ 0.0625 相近，但没有逐块核）。
- 没替主 agent采纳、没出判决；没复跑判决要引的数（0.2、3.1 的数都是日志行的算术，主 agent 用同一份日志复算即可）。

## 引文核对

`python3 scripts/agents/cite-check.py` 对本报告的末行（退出码 0，✗ 零处）：

```
  ✓ 核了 57 处引文、92 处裸行号，对上 57 处，没判 92 处（其中裸行号没有引文可比 87 处；逐处列在上面）
```
