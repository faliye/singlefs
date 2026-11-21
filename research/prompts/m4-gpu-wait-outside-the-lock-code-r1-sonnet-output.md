# m4-gpu-wait-outside-the-lock-code-r1 正推腿（Sonnet）报告

立场：从设计第三轮判决「三、改法」十四条与 D26 已定项 13 第 5、6 条推出「应该是什么」，与主工作区今天的 `crates/` 逐格比。只读码与只读命令；没写模型，没起 cargo，没跑门禁；读码推出来、没量过的都标「推的，没量过」。

条款出处（行号是此刻在文件里现查的）：
- 设计第三轮判决十四条：`research/prompts/m4-gpu-wait-outside-the-lock-design-r3-main-verification.md:46-59`（第 1 条在 46，第 14 条在 59）。
- D26 已定项 13：`.claude/kb/decisions/26-后台整理与放置回收.md@已定项 13：后台任务的计算放在哪、怎么开、什么节奏`，第 5 条在文件第 256 行、第 6 条在第 257 行（本报告引文用小节写法）。
- 卸载不等：同节第 4 条（第 255 行）的「卸载时不等还在跑的提交」。

本报告里「变异行 N」指 `mutation-rows-appended-by-the-batch.tsv` 的第 N 行（冻结快照，108 行）；「替换行 N」指 `mutation-rows-replaced-by-the-batch.tsv`。读码前我先量了两件事（原样结果见第四节）：108 行追加行里每一行的测试函数名都在 `crates/` 里恰有定义；每一行的「原文」栏在它点名的文件里恰好出现一次（即改回去的锚点今天还在）。

## 一、十四条逐条（代码处 / 钉它的测试与变异行 / 判定）

### 第 1 条：核心层后台入口拆三段，会话上记「有一批在锁外」，取输入第一步就判

- 代码：锁里取输入 `crates/singlefs-core/src/mounted_session.rs:9021` 的入口，第一句判是 `crates/singlefs-core/src/mounted_session.rs:9028`：「if self.session.compaction.has_a_batch_out_of_the_lock() {」，在取 GPU 锁（`begin_a_batch` 在 `crates/singlefs-core/src/mounted_session.rs:5680` 之后）与换设置之前就拒；锁外算 `BackgroundCompactionBatchTicket::compute_out_of_the_lock`（`crates/singlefs-core/src/mounted_session.rs:8142`）；回锁发布 `crates/singlefs-core/src/mounted_session.rs:9154`。GPU 与 CPU 同走：`crates/singlefs-core/src/mounted_session.rs:5945` 的 `gpu_compute_of_the_route()` 为 `None` 是 CPU 写法、否则 GPU，两条都过同一对入口。
- 测试：变异行 4（有一批在锁外时取输入拒在换设置之后）、行 5（显式入口照常推一趟）、行 6（攒批会话锁里推完的后台入口照常起跑）；首尾相接与锁里推完同序列由行 3 钉（`three_stages_chained_without_a_window_write_the_same_blocks_as_the_entry_for_at_most_one_p…`，函数名在变异表里被截断，全名要查表）。
- 判：成立。

### 第 2 条：取输入的内容（问让路、排空待发、`begin_a_batch`、现行那一版拷 `Arc` 指针）

- 代码：起跑判 `crates/singlefs-core/src/mounted_session.rs:9033`，排空待发 `crates/singlefs-core/src/mounted_session.rs:9066`：「let pending_changes_published_before_the_run = match self.publish_the_pending_changes(」，再进 `take_the_background_batch_that_starts`（`crates/singlefs-core/src/mounted_session.rs:9087`）；问让路在这之后，住在每批之前的判 `judge_before_the_next_compaction_batch`（`crates/singlefs-core/src/mounted_session.rs:5022`：「let foreground_waiting = match caller.answer_before_the_next_batch() {」）；`begin_a_batch` 在 `crates/singlefs-core/src/mounted_session.rs:5680`，`Arc` 指针拷贝在 `crates/singlefs-core/src/mounted_session.rs:5705`。
- 对不上的一句（并排）：条款与判决列的次序是「问让路、排空待发」——`research/prompts/m4-gpu-wait-outside-the-lock-design-r3-main-verification.md:47`：「取输入：问让路、排空待发、`begin_a_batch`、现行那一版拷 `Arc` 指针。」；代码次序是先排空（9066）后问让路（5022，在 9087 里面）。读码推：前台在等时仍多一次排空，排空发的是前台已收下的待发，早发不违反已定条；这一点没有测试钉次序（变异表里没有「让路与排空对调」的行）。
- 判：内容全有；次序与条款字面不同（推的，没量过影响）。

### 第 3 条：锁外算完先放 GPU 实现那把锁再回会话锁（量过）

- 代码：每一段 GPU 调用在自己的作用域里借锁、调完即放：`crates/singlefs-core/src/mounted_session.rs:5821`（扫描那段，守卫活到那个分支结束）与 `crates/singlefs-core/src/mounted_session.rs:5855`（打分那段，借用链在一个表达式里结束）；交回之前取提交记账那一次借 `crates/singlefs-core/src/mounted_session.rs:5976` 也在 `if let` 里放掉；回会话锁读盘的闭包 `read_in_the_session_lock` 只在 GPU 锁没持着时被调（`crates/singlefs-core/src/mounted_session.rs:8140` 的文档与实现一致）。
- 测试：变异行 7（锁外算完不放 GPU 锁就交回），测试名 `the_gpu_implementation_is_not_lent_while_the_batch_reads_in_the_session_lock_nor_once_the_…`。
- 判：成立。

### 第 4 条：回锁里出错先于一切

- 代码：`crates/singlefs-core/src/mounted_session.rs:6333`：「let task_after_the_error = background_compute.stop_the_task_after_a_compute_error(」，在 `judge_the_computed_compaction_batch_at_the_relock` 里是第一判，先于冻结判（`crates/singlefs-core/src/mounted_session.rs:6395`）与排空（9224）；回锁入口里在它之前只有「认票」与「核盘表」两判，都在任何读写之前（`crates/singlefs-core/src/mounted_session.rs:9160-9187`）。
- 测试：变异行 8（回锁先看版本变没变、后看出错），测试 `a_compute_error_with_a_foreground_publish_in_the_window_stops_the_compaction_as_an_error_…`。
- 判：成立。

### 第 5 条：回锁里不作废——排空待发、重取隔离集；有活意图那一支比整条记录与集合，变了的单元剔掉，其余在现行那一版上发，剔完不发

- 代码：
  - 排空待发：`crates/singlefs-core/src/mounted_session.rs:9224` 在回锁入口里，先于发布；排空被拒走 `crates/singlefs-core/src/mounted_session.rs:9233` 那一支。
  - 重取隔离集：`crates/singlefs-core/src/mounted_session.rs:6476`：「compaction.take_the_quarantines_noted_by_the_publishes(allocator.compaction_mut());」，在发布之前把锁外那一段期间发布路径隔离的槽并进常驻隔离集。
  - 整条记录比：`crates/singlefs-core/src/mounted_session.rs:6705`：「let slots_whose_record_changed: BTreeSet<SlotNumber> = records_of_the_region_at_the_input」，两边是 `BTreeMap<SlotNumber, AllocationRecord>`，值整条比；`AllocationRecord` 带 `quarantine_mark` 且派生 `PartialEq`（`crates/singlefs-core/src/allocator.rs:81`），所以隔离记号变了也算变。键并集比，所以「多出来的」「释放了的」都进变了的集合。
  - 剔与不发：按计划单元的源槽剔（`crates/singlefs-core/src/mounted_session.rs:6742-6745`），`crates/singlefs-core/src/mounted_session.rs:6749`：「if units_planned_before_pruning > 0 && planned.is_empty() {」时停在 `BatchPrunedToNothingAtTheRelock`，不发；其余在 `current_version`（现行那一版）上发。
- 测试：变异行 9（不剔）、10（比 R 里记录一个都不当变了）、11（只比盘槽跨度代、不比隔离记号）、12（不重取隔离集）、13（有活意图那一支在票里那一版上发）、14（把票里那一版当现行那一版）、77、78、79（崩溃流里不剔、全当变了、不记观测）、90（池级 checker 的细节，钉隔离记号用例用）。隔离记号那一格的专门用例是 `a_unit_quarantined_in_the_window_with_only_its_quarantine_mark_changed_is_pruned_at_the_relock…`（行 11、12、90 共点它）；判决里「没量的：隔离记号那一格」现在有了单测钉，但没有探针量过（推的，没量过：这条用例今天的树里红绿，待主 agent 跑，见第五节命令）。
- 读码疑点（没有证据说它错，只是条款没写、实现这样做了）：有活意图那一支，回锁**不再**对 R 调 `region_holds_a_quarantined_slot`；隔离的拦截全靠「记录整条比」。条款写的就是比整条记录，所以不算冲突；但这意味着「R 里出现隔离槽就丢意图」那条规矩（`compaction_plan::region_holds_a_quarantined_slot`）在回锁这一刻只由记录比对间接保证。推的，没量过。
- 判：成立。

### 第 6 条：签发那一支——对选中的段在现行那一版上重判资格与签发前那一判，各量取现行那一版

- 代码：`issue_the_chosen_region_at_the_relock`（`crates/singlefs-core/src/mounted_session.rs:6550`）。版本变了才重判（txg 不同，`crates/singlefs-core/src/mounted_session.rs:6569`）；资格判 `crates/singlefs-core/src/mounted_session.rs:6590`：「if !is_the_region_still_eligible {」，判的是这次挂载开过的段、钉住与隔离的槽、扫描拒过的段、R 里还有未释放记录，没资格停在 `ChosenSegmentNoLongerEligibleAtTheRelock`；签发前那一判的「R 里要搬的数」`crates/singlefs-core/src/mounted_session.rs:6600`：「let units_to_move = if records_of_the_region_at_the_input == records_of_the_region_now {」——记录没变取锁外扫出的能搬数，变了取现行那一版 R 里未释放的记录数（文档自己写明是上界，只多拒不少拒，`crates/singlefs-core/src/mounted_session.rs:6543`）；空洞数在现行那一版记录上算（`crates/singlefs-core/src/mounted_session.rs:6614`）；意图的 c0 取现行那一版的 txg（`crates/singlefs-core/src/mounted_session.rs:6631`：「allocation_generation_at_issue: current_version.root.checkpoint_txg,」），segs0 也在现行那一版记录上算（`crates/singlefs-core/src/mounted_session.rs:6632`）。
- 测试：变异行 15（c0 取票里那一版的 txg，测试 `an_issue_at_the_relock_takes_the_quantities_of_the_intent_from_the_current_version…`）；没了资格这一拍接着叫由行 54 钉，运行库用例 `after_the_chosen_segment_is_no_longer_eligible_at_the_relock_the_pass_calls_again` 走的是真路径（覆盖写第一个文件把那段记录放光，`crates/singlefs-mount-runtime/tests/mount_runtime_computes_the_batch_out_of_the_session_lock.rs:1360`）。
- 对不上的一句（并排）：判决 `research/prompts/m4-gpu-wait-outside-the-lock-design-r3-main-verification.md:51`：「签发那一支：回锁里对选中的段在现行那一版上重判资格与签发前那一判，各个量照现行那一版取（**量过**，…，两臂）。」——「各个量」里 c0 有变异行钉；资格重判本身（6590 那个 `if`）、空洞数用现行那一版记录（6614）、segs0 用现行那一版（6632）**没有变异行**：把它们改回票里那一版，今天的 108 行追加行里没有一行点它们。资格重判那一支有运行库用例走真路径，改成 `if false` 会让那条用例的「前提」断言红（推的，没量过）；空洞数与 segs0 两处没有找到专门的测试（核过 `crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock.rs` 的用例名，只有 c0 那一条点「量取自现行那一版」）。
- 判：成立；变异行覆盖不全（c0 有，资格重判、空洞数、segs0 没有）。

### 第 7 条：回锁那一次发布重判回退门（量过）

- 代码：`crates/singlefs-core/src/mounted_session.rs:6459`：「RollbackGateAtTheRelock::JudgedWhenTheVersionMoved if has_the_version_moved => {」——只在取输入时那一趟本来判回退门（`crates/singlefs-core/src/mounted_session.rs:3851` 的 `rollback_gate_at_the_relock_of(&run)` 决定）且回锁时现行那一版的 txg 与票里不同才重判，门关就返回 `Stopped`、不发。回锁排空发生在这之前（`crates/singlefs-core/src/mounted_session.rs:9224`），所以排空发出的新一版也落在「版本变了」里。版本没变时取输入那一刻刚判过同一份状态，不重判与重判等价（推的，没量过）。
- 测试：变异行 16（回锁那一次发布跳过回退门），测试 `a_relock_after_the_rollback_gate_closed_in_the_window_publishes_nothing`。
- 判：成立。

### 第 8 条：回锁时有冻结的发布，这一批丢掉，单独的停因

- 代码：`crates/singlefs-core/src/mounted_session.rs:6395`：「if txg_of_the_frozen_publish_now.is_some()」，条件的后半 `crates/singlefs-core/src/mounted_session.rs:6396`：「&& txg_of_the_frozen_publish_now != txg_of_the_frozen_publish_at_the_input」；停因 `BatchDroppedWhileAPublishIsFrozenAtTheRelock`，观测 `batches_dropped_while_a_publish_is_frozen_at_the_relock`（`crates/singlefs-core/src/mounted_session.rs:6400`）。
- 对不上的一句（并排）：判决 `research/prompts/m4-gpu-wait-outside-the-lock-design-r3-main-verification.md:53`：「回锁时有冻结的发布：这一批丢掉，单独的停因。」；代码只丢「回锁时冻结着、且与取输入时冻结的不是同一次（txg 不同或取输入时没冻结）」的那些；取输入时就冻结着、回锁时还是同一次冻结的，**照发**。从运行库进来取不到这种输入（一拍开头冻结着就先重发、冻着就不叫整理，`crates/singlefs-mount-runtime/src/lib.rs:789`），所以只在核心层直接调入口时出现（推的，没量过）。
- 测试：变异行 17 把整个 `if` 条件改成 `if false && …`（只钉「有这个分支」，不钉 `!= at_the_input` 那一半）；运行库行 29、30（丢批之后这一拍接着叫 / 计数）。
- 判：成立（取输入之后新冻结的）；字面比条款窄一档，`!= at_the_input` 那一半没有变异行钉。

### 第 9 条：让路——任何前台入口等锁都让、`try_lock` 失败才登记、问答与停因各加一个成员、破例批不让、N 次或 T 毫秒下限、一批发成清零、新停因进下一拍名单

- 代码：
  - 计数在锁外：`crates/singlefs-mount-runtime/src/lib.rs:461`：「foreground_entries_waiting_for_the_session_lock: AtomicU32,」（不在 `Mutex` 里）。
  - `try_lock` 失败才登记：`crates/singlefs-mount-runtime/src/lib.rs:645`：「match self.locked.try_lock() {」，`WouldBlock` 才登记再等；收下改动、单发改动、切换后台计算、读四个入口都走它（`crates/singlefs-mount-runtime/src/lib.rs:1449`、`1517`、`1543`、`1576`）。登记先声明、锁守卫后声明，丢弃时先放锁后减计数。
  - 问答成员：`crates/singlefs-mount-runtime/src/lib.rs:507` 答 `ForegroundEntryOtherThanAnFsyncIsWaitingForTheSessionLock`；停因成员 `YieldedToAForegroundEntryWaitingForTheSessionLock`（运行库那一拍不再叫：`crates/singlefs-mount-runtime/src/lib.rs:1082`）。
  - 破例批不让：`crates/singlefs-core/src/mounted_session.rs:5040`：「if let Some(waiting) = foreground_waiting.filter(|_| !is_this_publish_the_exception) {」。
  - 下限：`crates/singlefs-core/src/compaction_plan.rs:1149`：「if is_the_count_reached || is_the_time_reached {」，次数与时长任一到了就不让；运行库两样都设（`crates/singlefs-mount-runtime/src/lib.rs:126`），默认 N=8（`crates/singlefs-mount-runtime/src/lib.rs:82`）、T=5 秒（`crates/singlefs-mount-runtime/src/lib.rs:86`），文档自称「推的，待量」。
  - 清零：`crates/singlefs-core/src/mounted_session.rs:5108`：「compaction.clear_the_yields_in_a_row();」。
  - 新停因的下一拍名单：条款写的函数名 `keeps_going` 今天的 `crates/` 里没有（`grep -rn keeps_going crates` 没有命中）；同一件事是 `the_pass_calls_again_after`（`crates/singlefs-mount-runtime/src/lib.rs:1079`），穷举 `match`、没有 `_ =>`，新让路停因（`YieldedToAForegroundEntryWaitingForTheSessionLock`）、冻结丢批、有一批在锁外三个在 `false` 那一组，回锁剔光与选中的段没了资格两个在 `true` 那一组（`crates/singlefs-mount-runtime/src/lib.rs:1085-1088`）。
- 测试：核心层行 18、19、20（N 多让一次）、21（T 到了还让）、22（一批发成不清）；运行库行 33–44（含行 44「前台入口取得到锁也登记在等」钉 `try_lock` 先试；行 38、39 钉 fsync 与别的入口都在等时答 fsync）；破例批不让由替换行 26 钉。
- 对不上的两句（并排）：
  1. 判决 `research/prompts/m4-gpu-wait-outside-the-lock-design-r3-main-verification.md:54`：「让路：任何前台入口等会话锁都让，计数在锁外、`try_lock` 失败才登记，问答与停因各加一个成员，破例批不让；」——前台 fsync 这个入口不是这样：`crates/singlefs-mount-runtime/src/lib.rs:1474`：「let _waiting = self.foreground_fsync_is_waiting();」在取锁之前无条件登记，锁空着也登记。这是已有的 fsync 登记（`crates/singlefs-mount-runtime/src/lib.rs:484` 的注释说「进门就登记」），条款第 6 条（D26）把 fsync 与其余入口并列写，没有写 fsync 也要 try_lock 先试；变异表里没有点 fsync 登记方式的行。影响：fsync 登记期间（持锁在做自己那次发布时）后台若恰好在每批之前问，会为一个正持着锁的 fsync 让一次路；让路次数照下限封顶，推的，没量过。
  2. 判决同一行后半：「连着让路 N 次后下一批不让，N 一批发成才清零。」；代码 `crates/singlefs-core/src/mounted_session.rs:5108` 是在「任何一次整理发布发成」（`CompactionStep::Published`，含只签意图、只删意图、照抄意图的发布，不限搬了单元的那种）时清零，而同一个分支里「算一批」只对搬了单元的发布记（`crates/singlefs-core/src/mounted_session.rs:5112`）。条款说的「一批」若按「搬了单元的那一次」读，清零条件比它宽；若按「一次整理发布」读，一致。条款没定义，这一句读法不唯一；变异行 22 钉「一批发成就清」这个方向，没有行钉「只签意图的发布不清」。
- 判：成立；两处读法与条款字面有出入（fsync 登记方式；清零的「一批」含不含非搬单元的发布）。

### 第 10 条：维护线程自己的 `publish_when_due` 与重发冻结在锁外那一段期间照跑

- 代码：`crates/singlefs-mount-runtime/src/lib.rs:1151`：「self.publish_while_a_batch_is_computed_out_of_the_lock()?;」，维护线程把锁外那一段交给一条工作线程算（`crates/singlefs-mount-runtime/src/lib.rs:1114`），自己每过一个 `publish_poll_period` 回会话锁里重发冻结、不冻着就 `publish_when_due`（`crates/singlefs-mount-runtime/src/lib.rs:1056`：「if !self.resend_the_frozen_publish_if_any()? {」）；在停时两件都不做（`crates/singlefs-mount-runtime/src/lib.rs:1053`）。起不了工作线程时在维护线程上算，那期间两件不跑（文档写明）。
- 测试：变异行 24（锁外等着时不 publish_when_due）、25（只 publish_when_due、不重发冻结）、23（锁外算的那一段持着会话锁）；运行库用例的「不切片」注释在 `crates/singlefs-mount-runtime/tests/mount_runtime_computes_the_batch_out_of_the_session_lock.rs:668`、`717`。
- 判：成立。

### 第 11 条：切到 `gpu` 的校验不取 GPU 实现那把锁

- 代码：`crates/singlefs-core/src/background_compute.rs:1312`：「devices_with_total_memory: gpu_compute.inventory_without_waiting(),」；`crates/singlefs-core/src/background_compute.rs:857`：「match self.shared.try_lock() {」——空着就问实现并更新快照，被借着或中毒就读快照。注意字面：条款是「不取」，实现是「不等」（`try_lock` 成功时会短暂取一下）；判决 r3 第 11 条的本意是「不在会话锁里等 GPU 锁」，两者在今天的实现里是一回事（`try_lock` 不阻塞），读码推，没量过。
- 测试：变异行 1（校验改回 `.lock()`），测试 `a_switch_to_the_gpu_is_admitted_without_waiting_for_the_gpu_implementation_lent_to_another_thread`；替换行 9 钉「校验照实现答的卡清单」。
- 判：成立。

### 第 12 条：GPU 把手锁中毒交回错误

- 代码：`crates/singlefs-core/src/background_compute.rs:851`：「.map_err(|_poisoned| GpuImplementationPanickedWhileLent)」；两处借锁点都把它转成 `BackgroundComputeError`（`crates/singlefs-core/src/mounted_session.rs:5823`、`5857`）；交回前取记账那一处（`crates/singlefs-core/src/mounted_session.rs:5976`）对中毒是静默跳过（`if let Ok`），此时提议里已经带着前面交回的错，不丢错。
- 测试：变异行 2（中毒照旧 panic），测试 `a_gpu_implementation_poisoned_while_lent_is_reported_as_an_error_instead_of_panicking`。
- 判：成立。

### 第 13 条：卸载与 Drop 不等锁外那一批（GPU 包里等的那一处读停标记，那次提交留在卡上，整张泄漏）

- 代码：
  - 次序：`crates/singlefs-mount-runtime/src/lib.rs:549`：「self.stopping.store(true, Ordering::SeqCst);」先于 `crates/singlefs-mount-runtime/src/lib.rs:550` 的告诉 GPU 实现在停；标记只是把手里的原子，不取实现那把锁（`crates/singlefs-core/src/background_compute.rs:829` 的文档与实现）。卸载与 Drop 同走（`crates/singlefs-mount-runtime/src/lib.rs:1721` 的 `Drop`）。
  - 放弃那一等：`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:185`：「return Err(GpuFailureCause::GaveUpWaitingForTheCallerIsStopping);」，一次长等拆成至多 100 毫秒的短等（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:60`），每轮没等到就读一次；放弃不重交、不数进超时次数（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:199` 的 `waited_past_the_limit` 排掉它）；提交记账与交活前、重交前、冲掉待写那几处等都从同一个取期限的函数拿（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:129`）。
  - 运行库在停时不拿放弃的错回锁：`crates/singlefs-mount-runtime/src/lib.rs:827`：「if self.stopping.load(Ordering::SeqCst) {」，丢掉这一批、`break`；核心层专门有一个记调用方丢了这一批、并把会话上那条在锁外的记号清掉的入口（`crates/singlefs-core/src/mounted_session.rs:9351`）。
- 测试：GPU 包变异行 59–76（等的循环读标记、短等间隔、放弃不重交、放弃不放账、放弃不判卡用不了、标记转给两份核与卡的账等）；运行库行 26–28、45–53（停时告诉实现、不回锁、卸载排空被拒交回运行库时撤标记等）。
- 没覆盖到的等（读码，推的，没量过）：`crates/singlefs-background-gpu/src/background_gpu.rs:895`：「let captured = wait_for_the_wgpu_future(」建缓冲时收「显存不足」错误域的那一等，用的是不带停标记的 `GpuWaitDeadline::starting_now`（`crates/singlefs-background-gpu/src/background_gpu.rs:4330` 的 `wait_for_the_wgpu_future`），不读停标记；它等的是 wgpu 的 future，平常当场返回，只有驱动挂住时才会让卸载多等到 `wait_limit`。条款的「等的那一处」指等提交结果的那一等，这一处不是；记作残留，没有变异行也没有用例。
- 判：成立（等提交结果那一等）；上面那一处残留。

### 第 14 条：观测——让路、剔掉了几个单元、没资格换段、冻结丢批各有计数或停因成员

- 代码：让路——核心层计数（`crates/singlefs-core/src/compaction_plan.rs:1157`、`1160`）与运行库计数（`crates/singlefs-mount-runtime/src/lib.rs:1017`、`1020`）、停因成员两个；剔掉几个单元——`crates/singlefs-core/src/compaction_plan.rs:833` 与 `crates/singlefs-mount-runtime/src/lib.rs:296`（运行库累加，行 56–58 钉）；冻结丢批——`crates/singlefs-core/src/compaction_plan.rs:837`、`crates/singlefs-mount-runtime/src/lib.rs:298`、停因成员；没资格换段——停因成员 `ChosenSegmentNoLongerEligibleAtTheRelock`，计数 `crates/singlefs-core/src/compaction_plan.rs:835` 的 `issues_given_up_for_a_region_no_longer_eligible_at_the_relock`。
- 量的（`grep -rn issues_given_up_for_a_region_no_longer_eligible_at_the_relock crates`）：这个计数整个 `crates/` 里只有定义（`crates/singlefs-core/src/compaction_plan.rs:835`）与加一（`crates/singlefs-core/src/mounted_session.rs:6593`）两处，没有任何读它的代码或测试，运行库观测里也没有它的镜像。条款要求「有计数或停因成员」，停因成员有，所以条款满足；这个计数本身没有读者、没被任何测试钉。
- 判：成立（停因成员齐、三个计数有读者有测试）；一个计数没读者。

## 二、正文「三、检查单」九格

### 格 1：走不到的理由二次查实（两处）——成立（只读码，推的，没量过）

- `crates/singlefs-core/src/compaction_plan.rs:1190`：「assert!(」，断言「至多一批在锁外」。只有一处调 `hand_a_batch_out_of_the_lock`（`crates/singlefs-core/src/mounted_session.rs:3854`，`grep -rn hand_a_batch_out_of_the_lock crates` 只有定义与这一处调用）；那一处的外层 `take_the_background_batch_that_starts` 只有 `crates/singlefs-core/src/mounted_session.rs:9087` 一处调用；那个入口的第一句判就是 `crates/singlefs-core/src/mounted_session.rs:9028` 的 `has_a_batch_out_of_the_lock()` 为真即返回 `Err`；9028 到 3854 之间都在同一个 `&mut self` 的调用链里，没有别的出口交出过票。理由对。
- `crates/singlefs-core/src/mounted_session.rs:9311`：「Ok(_run_going_on_to_another_batch) => unreachable!(」。票交出之前，`take_the_next_compaction_batch_or_finish_the_run` 的循环顶上每圈先过 `judge_before_the_next_compaction_batch`（`crates/singlefs-core/src/mounted_session.rs:5216` 那一圈），它在 `publishes_of_the_run == 1` 且一次调用至多一次发布时停（`crates/singlefs-core/src/mounted_session.rs:4999`），所以交出票时这一趟发布数是 0；回锁发成一次之后数是 1，再走同一个判时，预算判（`crates/singlefs-core/src/mounted_session.rs:4982`）、「至多一批」判（4990）、「至多一次发布」判（4999）至少最后一道必停；没发成的各支（`Stopped`、被拒、删意图而停）在 `note_the_compaction_batch_advanced` 里就交回 `Err`（`crates/singlefs-core/src/mounted_session.rs:5151`、`5156`、`5142`）。理由对。
- 没有新错误成员「条款没定」的说法：`GaveUpWaitingForTheCallerIsStopping` 确是条款定的那一支，`crates/singlefs-background-gpu/src/background_gpu_compute.rs:23`：「卸载、丢弃时等一批的结果的那一处放弃那一等，这一批报成 `GaveUpWaitingForTheCallerIsStopping`」，条款原句在 D26 已定项 13 第 4 条「卸载时不等还在跑的提交」。成立。

### 格 2：类型与名字——成立，三点记录

- 新类型：`BatchOutOfTheLock(u64)`（`crates/singlefs-core/src/compaction_plan.rs:1026`）是单字段包装；`CompactionYieldFloor` 两个字段是 `Option<NonZeroU64>` 与 `Option<Duration>`（`crates/singlefs-core/src/compaction_plan.rs:1062`），次数与时长不共用裸整数；运行库参数里 N 是 `NonZeroU64`、T 是 `Duration`（`crates/singlefs-mount-runtime/src/lib.rs:101-104`）。同一个量不同语义共用裸整数：没找到（计数字段 `yields_in_a_row`、`units_pruned_at_the_relock` 都是 `u64` 但名字说清是各自的计数，不是一个量被两用）。
- 缩写 / `_ =>`：补丁新增行里没有 `_ =>`（`grep -nE '^\+[^/]*[^a-z_]_ =>'` 在补丁上 0 命中）；新增的 `.expect(` 46 处里没有不带说明的（跑过：把不含引号的 `.expect(` 行挑出来只有一处，是跨行 `.expect(` 后一行起头带文字的 `crates/singlefs-core/src/mounted_session.rs:6453`，说明在下一行，写明「取输入那一刻现行那一版有树」依赖哪条不变量）。
- 序数 / 位置词：新测试里的辅助名带 `first_file`、`first_device`（`overwrite_the_first_file`、`force_the_segment_holding_only_data_units_of_the_first_file`、`data_unit_at_the_slot_on_the_first_device`、`mounted_with_the_segment_of_the_first_file_forced` 等，补丁里 12 行带 `first_/second_/third_` 的函数或变量声明）。`.claude/naming-ordinal-tokens` 的词表只列 `layer0` 一族、`first_stream` 一族，不含 `first_file`，所以这一格的门禁大概率不红；按共用约束「名字不用序数或位置代称」的字面读，`first_file` 是位置代称（推的，没量过），命名检查交主 agent 跑（第五节）。
- `JudgeOfTheEnumeration` 在 6 份崩溃流测试里各定义一份（`grep -ln 'enum JudgeOfTheEnumeration' crates/singlefs-verification/tests/*.rs` 6 个文件），内容相同；另两份（GPU 路径流、批间切换流）用别的写法。是复制而不是共用，因为每份是独立测试目标；不违反这一格，记作重复。

### 格 3：不兼容旧代码与旧格式——盘上格式一侧成立；「旧入口」一侧不成立（事实列出，是否违规主 agent 判）

- 盘上格式、版本号、迁移：补丁新增行里没有 `legacy`、`旧版`、`迁移`、`compat` 字样（grep 0 命中），不动格式。
- 三段之外是否还留着锁内一口气算完的入口：留着。`crates/singlefs-core/src/mounted_session.rs:3359` 与 `crates/singlefs-core/src/mounted_session.rs:8775` 的 `run_compaction_in_the_background`、`crates/singlefs-core/src/mounted_session.rs:8800` 的 `run_compaction_in_the_background_for_at_most_one_publish` 都还是 `pub`，让路下限写死 `NO_FLOOR`（`crates/singlefs-core/src/mounted_session.rs:3692`）。运行库已不调它们；今天的调用方是验证包的历史驱动 `crates/singlefs-verification/src/history.rs:10663`、`e202` 实验二进制，与测试里「锁里现算」的对照（变异行 81、87 的用例名带 `in_lock_reference`）。它们不是为旧盘、旧版本留的，是同一实现的锁内版，用作对照与实验；字面上等于检查单举的那个例子。判不了它算不算「为旧路径留的分支」：推不出。

### 格 4：不新建判定库——成立

- 补丁新增行里没有 `rocksdb`、`sqlite`、`.tsv` 写入、新的目录存放（`grep` 0 命中，仅注释里点名 `crates/mutations.tsv` 的字样）。CPU 两段版照先例经 `crates/singlefs-verification/src/crash_library.rs:420`（`multi_host_configuration_on_the_cpu_keeping_the_prune_server_of`）写一份只含 `ENABLE_GPU=0` 与（有则抄）prune-server 地址两行的临时配置；判定经「全仓那一份库」或 `DisposableLibrary`（一次性、对临时 prune-server 的库，共用约束把它归入「对临时服务起的自证库不算」）。新崩溃流文件里的配置目录是 `ExclusiveTemporaryDirectory`，只放那份配置文件，不放判定。

### 格 5：所有任务多机可用——成立（读码）

- 补丁新增行里没有 `/home/`、主机名、`127.0.0.1`、`/tmp/` 字面（grep 0 命中，上面那一条同一次）；环境读取经 `multi_host_configuration_of_this_process()` 与 `CARDS_BY_PRIORITY_KEY`（环境变量名，不是主机值）。CPU 两段版的配置在测试自己的临时目录里现生成，地址从调用方那份多机配置抄（`crates/singlefs-verification/src/crash_library.rs:424`：「if let Some(address) = read_configuration_value(caller_configuration_file, NETWORK_ADDRESS_KEY)?」）。两台是否都跑得绿：推不出（没起任何测试），第五节有命令。

### 格 6：测试优先多线程——大体成立，一处不全

- 崩溃枚举用例都走 `enumerate_crash_states_with_attached_observers`（`crates/singlefs-verification/src/crash_state_enumeration.rs:479`），新崩溃流文件两处调用，测试文件里不自设线程数。线程数是否取登记值是在 `src/` 里面的事，我没追进去：推不出（同一族旧文件也没有自设线程）。
- 运行库两份新测试：`mount_runtime_computes_the_batch_out_of_the_session_lock.rs` 12 条、有「不切片」理由的 9 条，缺的是 `crates/singlefs-mount-runtime/tests/mount_runtime_computes_the_batch_out_of_the_session_lock.rs:867`、`917`、`1093` 三条；`mount_runtime_yields_to_foreground_entries_waiting_for_the_session_lock.rs` 8 条、有的 4 条，缺的是 `crates/singlefs-mount-runtime/tests/mount_runtime_yields_to_foreground_entries_waiting_for_the_session_lock.rs:138`、`163`、`181`、`347` 四条（量的：对每个 `#[test]` 看前一行是否带「不切片」）。这 7 条是一个池一条维护线程、没有可切区间的同一类，但没写理由；检查单要求「注释写明不切片的理由」。

### 格 7：带 `--item` 的命令与项——文件层成立，能否被列出待主 agent 跑

- 量的：`crash-point-replay-items.tsv` 里含 `on-the-cpu-judge` 或 `out-of-the-lock` 的 `crash-case:` 项 15 个，每一个都在 `stage-inputs.tsv` 里有同名行，每行 `test=` 点名的测试函数在对应测试文件里有 `fn` 定义（脚本核：缺 0 个、test 缺 0 个）；九条整理流各有一个 `-on-the-cpu-judge` 项（暂停流有两个），另有 4 个 `out-of-the-lock-around-*`（覆盖写、删文件各一个流项与一个 CPU 判器项）。`--list-items` 真能不能列出：待主 agent 跑（命令在第五节）。

### 格 8：新崩溃流多罩了什么——成立（读码），没量过计数

- 新流 `crates/singlefs-verification/tests/crash_enumeration_compaction_batch_computed_before_a_foreground_change_and_pruned_at_the_relock_stream.rs` 的用例：覆盖写与删文件各一条全量、各一条 CPU 判器那一条（`full_enumeration_with_an_overwrite_between_the_input_and_the_relock_is_exhaustive_and_clean`、`…_judged_on_the_cpu_judge_is_exhaustive_and_clean` 与删文件一对）；前缀与登记流逐块相同由 `the_in_lock_reference_with_an_overwrite_writes_the_blocks_of_the_registered_flow_with_a_write_while_the_intent_is_live` 钉（变异行 81、89 点它）；回锁剔成真子集由 `the_batch_after_an_overwrite_is_pruned_to_a_strict_subset_of_the_recomputation_in_the_lock` 一族钉（行 79、82、83、86、87、88）。这些函数名都在源文件里（脚本核过）。「多罩了锁外期间前台发布一次再剔再发」这个说法，读码与用例名一致；「多罩了几个崩溃状态」的计数没量（缩域不判全域，主 agent 说全域留给提交时）：推不出多出多少个状态。
- 九条整理流的 CPU 版与 GPU 版录同一条流：每份文件里都是同一个 `enumerate_with_both_observers`，经一个只换判器的参数（6 份用 `JudgeOfTheEnumeration`）；块键相同、多罩的是 CPU 判器那一侧：与文件里的文档一致，没量过「块全复用」，主 agent 的观测说登记的 26 条缩到深度 5 判绿、块全复用。

### 格 9：放的位置——成立

- 新测试都在被测包自己的 `tests/`（运行库两份在 `crates/singlefs-mount-runtime/tests/`，验证包的在 `crates/singlefs-verification/tests/`）；`grep -nE '#\[path|include!' ` 在补丁新增行里 0 命中。

## 三、什么现象会推翻上面的结论

- 第 1、3、4、10、11、12 条「成立」：推翻它们的是，把对应变异行改回去后点名的测试不红（行 4/5/6、7、8、24/25、1、2），或今天的树上那条测试自己就红（第五节命令 2、3）。
- 第 5 条「成立」：在回锁前后只改隔离记号（不动盘、槽、跨度、代）、锁外那一批计划里含这个单元时，回锁没把它剔掉而发出去了；或造出「记录集合相等、整条记录相等、但单元内容在锁外那一段期间变了」的前台路径（我没造，读码没找到）。变异行 11、12、90 点的那条用例今天不红绿，也推翻「有单测钉」这一半。
- 第 6 条「成立；覆盖不全」：第五节命令 4 把候选行施加后，若 5 条候选里有的「抓到」，那一处覆盖不全的说法就不成立（该处被别的测试钉着）；若全「没抓到」，覆盖缺口坐实，需要补变异行。
- 第 8 条「字面窄一档」：造出核心层 API 取输入时就冻结着、回锁还是同一次冻结、而发出去的批写出今天写不出的盘序列；或找到运行库路径能在冻结着时取到输入（我读到 `crates/singlefs-mount-runtime/src/lib.rs:789` 起的一拍开头先重发冻结、冻着就返回）。
- 第 9 条的两处出入：对 fsync 路径：造出前台 fsync 在锁空着时也登记导致后台无谓让路超过下限封顶（次数 N 或时长 T）的历史，即推翻「只是多让几次」；对「清零的一批」：造出只签意图或只删意图的发布清了计数而让前台被连着越过的历史。两处我都没造，推的。
- 第 13 条「成立（等提交结果那一等）」：在卸载期间锁外那一段卡在 `crates/singlefs-background-gpu/src/background_gpu.rs:895` 那类不读停标记的 wgpu future 等上，卸载多等到 `wait_limit` 的实测，会把「残留」升成缺陷。
- 格 3 的「不成立」：主 agent 若判「锁内一口气算完的入口留作对照与实验，不算为旧路径留的分支」，这一格改成立。
- 格 6 的缺注释 7 条：补上理由即消。

## 四、我自己量的事实（原样）

- 追加行 108 行，每行第 6 栏的测试函数名在 `crates/` 里有 `fn <名>` 定义：「108 rows; test fn not found: []」。
- 追加行 108 行，每行第 3 栏的原文在它第 2 栏点名的文件里恰好出现一次：「108 rows; original text not exactly once: []」（所以改回去的锚点今天都还在，行没失效）。
- 新登记的 `crash-case:` 项（名字含 `on-the-cpu-judge` 或 `out-of-the-lock`）15 个：`items-tsv crash-case total 121 new matched 15`、`missing in stage-inputs: []`、`test fn missing: []`。
- `grep -rn issues_given_up_for_a_region_no_longer_eligible_at_the_relock crates` 只有 `crates/singlefs-core/src/compaction_plan.rs:835` 与 `crates/singlefs-core/src/mounted_session.rs:6593` 两处。
- `grep -rn keeps_going crates --include=*.rs` 没有命中。
- 补丁新增行里 `#[path` 与 `include!` 命中 0。

## 五、要主 agent 跑的命令

我没起 cargo、没交任务。五条，每条一项。

1. 列出这一批追加的变异行与新登记项（不起 cargo）。
   - 命令：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 crates-mutation-replay.sh 的 --list-items; 范围=只列变异表可点名的行、不起 cargo; 为什么=核这一批追加的 108 行与新登记的崩溃枚举用例都能被列出' task-server/task submit -- bash .claude/gate.d/crates-mutation-replay.sh --list-items`
   - 在哪起：仓根。
   - 看哪几行：`TASK_SERVER_RESULT` 行 `type=` 与 `exit=0`；日志末尾「键对不上（要现判）的条数」与「锚点失效」两个数是 0，列出的行里能找到上文「放锁的整理」「运行库放锁」「后台 GPU 包 卸载不等」「锁外算那条崩溃流」开头的变异名。
2. 点名跑钉第 5、6、7、8、9 条的五条验证包用例（不整包）。
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 compaction_batch_computed_out_of_the_session_lock 里的 5 条测试; 范围=5 个测试; 为什么=核钉第 5、6、7、8、9 条的用例今天在主工作区的红绿' task-server/task submit -- cargo test -p singlefs-verification --test compaction_batch_computed_out_of_the_session_lock -- a_unit_quarantined_in_the_window_with_only_its_quarantine_mark_changed_is_pruned_at_the_relock_on_the_cpu_judge an_issue_at_the_relock_takes_the_quantities_of_the_intent_from_the_current_version a_relock_after_the_rollback_gate_closed_in_the_window_publishes_nothing a_batch_relocked_while_a_publish_frozen_in_the_window_is_dropped_without_failing_the_compaction a_batch_published_clears_the_yields_in_a_row_and_the_next_batch_yields_again --exact`
   - 在哪起：仓根。若范围声明闸判这个目标会走崩溃枚举，照它的拒绝说明在 `task-server/task submit` 后面补 `env <缩域变量>=<n>` 或加「全域理由：…」重交，不换写法。
   - 看哪几行：`TASK_SERVER_RESULT` 的 `exit=0`，日志里 `test result: ok. 5 passed`。
3. 点名跑钉第 9 条 try_lock 先试与 fsync 优先的两条运行库用例。
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-mount-runtime 的测试目标 mount_runtime_yields_to_foreground_entries_waiting_for_the_session_lock 里的 2 条测试; 范围=2 个测试; 为什么=核 try_lock 先试与 fsync 优先两条用例今天的红绿' task-server/task submit -- cargo test -p singlefs-mount-runtime --test mount_runtime_yields_to_foreground_entries_waiting_for_the_session_lock -- entries_taking_a_free_session_lock_are_not_registered_as_waiting with_an_fsync_and_another_entry_both_waiting_the_trip_yields_to_the_fsync --exact`
   - 在哪起：仓根。
   - 看哪几行：`exit=0`，日志里 `test result: ok. 2 passed`。
4. 把 5 条候选变异行在副本里证红，量第 6、8 条的覆盖缺口（候选行在 `/home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r1-sonnet/candidate-mutation-rows.tsv`，sha256 `7a788f8e0cb267aad40fbccc5a6be02c22905640a3445a7623a4e6b5dde298e8`，每行原文在 `crates/singlefs-core/src/mounted_session.rs` 里恰好出现一次，是我核过的）。
   - 做法（主 agent 自己建副本，我没建）：`python3 scripts/hosts/minimal-copy.py <仓根> <副本> --with-test-inputs`，把候选文件的 5 行追加进 `<副本>/crates/mutations.tsv` 末尾，再交下面这条。
   - 命令：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=变异证红 5 条候选行; 范围=5 行; 为什么=量第 6、8 条的变异行覆盖缺口' task-server/task submit --cwd <仓根> -- bash scripts/mutation/prove-red.sh --copy <副本> singlefs-core '候选：回锁签发那一支不重判资格（版本变了也照签）' '候选：回锁签发前判的空洞数取票里那一版的记录' '候选：回锁签发的 segs0 取票里那一版的记录' '候选：回锁签发前判的要搬数记录变了也取锁外扫出的能搬数' '候选：取输入时就冻结着的那一次回锁也丢这一批'`
   - 看哪几行：逐行「抓到 / 没抓到」；「没抓到」的行就是覆盖缺口；若被范围声明闸拒（这几行点到的验证包目标会走崩溃枚举），照拒绝说明加缩域变量重交。
5. 命名检查（新测试里的 `first_file` 一族读法上是位置代称）。
   - 命令一：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-source-discipline.sh 的 vague-names、test-file-names、ordinal-names、data-names 四格; 范围=4 格; 为什么=核这一批新测试里 first_file 一族名字' task-server/task submit -- bash .claude/gate.d/code-source-discipline.sh --check vague-names,test-file-names,ordinal-names,data-names .`
   - 命令二：`task-server/task submit -- bash .claude/singlefs-ai-sop/scripts/naming-lint.sh .`
   - 在哪起：仓根。
   - 看哪几行：`TASK_SERVER_RESULT` 的 `exit=0` 是过、`exit=1` 是判红；命中在这一批新测试文件里的 `first_file` 一族要新派同类 agent 改名，命中在别的文件里的不修。

## 六、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 第 1 条 三段拆法 | 一致 | 入口、锁外、回锁三段齐，第一判在取 GPU 锁与换设置之前，GPU 与 CPU 同走 |
| 第 2 条 取输入内容 | 一致（次序冲突） | 内容齐；条款列「问让路、排空待发」，代码先排空后问让路 |
| 第 3 条 先放 GPU 锁 | 一致 | 每段调用借完即放，行 7 钉 |
| 第 4 条 出错先于一切 | 一致 | 回锁第一判，先于冻结与排空，行 8 钉 |
| 第 5 条 回锁不作废 | 一致 | 整条记录比含隔离记号，剔光不发，行 9–14、77–79、90 钉；隔离记号那格没有探针量 |
| 第 6 条 签发那一支 | 一致（钉得不全） | 资格重判、空洞数、segs0 取现行那一版，只有 c0 有变异行 |
| 第 7 条 回退门重判 | 一致 | 版本变了才重判，行 16 钉 |
| 第 8 条 冻结丢批 | 一致（字面窄一档） | 只丢取输入之后新冻结的；`!= at_the_input` 那半没有变异行钉 |
| 第 9 条 让路 | 一致（两处冲突） | fsync 无条件登记、不是 try_lock 先试；清零发生在任何整理发布发成，条款「一批」读法不唯一 |
| 第 10 条 publish_when_due 照跑 | 一致 | 行 23–25 钉 |
| 第 11 条 校验不取 GPU 锁 | 一致 | 实现是 try_lock 不等，条款字面是不取，今天等价 |
| 第 12 条 中毒交回错误 | 一致 | 行 2 钉 |
| 第 13 条 卸载与 Drop 不等 | 一致（有残留） | 等提交结果那一等读标记；建缓冲收显存不足错误域那一等不读，推的 |
| 第 14 条 观测 | 一致（一个计数无读者） | 没资格换段的计数无人读、无测试，停因成员有 |
| 格 1 走不到 | 成立 | 两处理由读码对，推的，没量过 |
| 格 2 类型与名字 | 成立 | `first_file` 一族读法上是位置代称；`JudgeOfTheEnumeration` 复制 6 份 |
| 格 3 不兼容旧代码 | 盘上格式成立；旧入口不成立 | 锁内一口气算完的入口仍是 pub，有非运行库调用方；算不算违规主 agent 判 |
| 格 4 不新建判定库 | 成立 | 没有新存放处，配置临时目录只放配置文件 |
| 格 5 多机可用 | 成立（读码） | 没写死主机值；两台是否绿没跑 |
| 格 6 多线程 | 大体成立 | 崩溃用例走枚举一族；运行库测试 7 条缺不切片理由注释 |
| 格 7 带 --item 的命令与项 | 文件层成立 | 15 个新项两张表对得上、测试函数都在；能否列出待主 agent 跑 |
| 格 8 新崩溃流多罩 | 成立（读码） | 覆盖写与删文件各一对，前缀逐块相同有用例；状态数没量 |
| 格 9 放的位置 | 成立 | 都在各自包的 `tests/`，没有 `#[path]` 与 `include!` |

## 七、没做什么

- 不判别的腿的格：没替攻方腿判「攻方立场」的格（让路饿死、停标记漏读、交错写出今天写不出的历史、测试测的是实现还是条款）；上面「推翻条件」里写的只是正推这一侧的反例形状，没造。
- 不替主 agent 采纳或出判决；没往统一任务服务交任何任务、没起 cargo、没跑门禁、没跑模型；判决要引的数没复跑，命令在第五节。
- 没写模型，所以没有 `rerun.sh`、没有每个文件的 sha256（候选变异行文件的 sha256 在第五节第 4 条）。
- 隔离记号那一格、回退复活与快照（判决自己写明没量的几格）我没量，也没造反例。
- 没追 `enumerate_crash_states_with_attached_observers` 里线程数取不取登记值。
- 草稿目录里前一条同立场的腿留下的零碎（`rows.py`、`rows.txt`），按派发要求没读没用，也没删。
- 没建仓副本、没编译目录，没有要主 agent 删的东西；草稿目录里我新写的只有 `candidate-mutation-rows.tsv`（几百字节）。
