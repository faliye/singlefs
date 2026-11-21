# m4-gpu-wait-outside-the-lock-design-r1 云端正推腿（Sonnet）报告

日期 2026-11-18。立场：正推，给一份能照着写的形态；Q1、Q2 主攻。方案按 `crates/` 今天的实现谈；岔路写成候选加代价，不替用户定。
状态：读码与读 kb 推出。纯算术模型与 Rust 基线探针我都**没跑、没编**（子 agent 不起任务、不起 cargo），下面凡是读数一律「推的」或「待主 agent 跑」。
引文：代码写主工作区路径加行号（本轮没有冻结副本，主工作区的 `crates/singlefs-mount-runtime/src/lib.rs`、`crates/singlefs-core/src/mounted_session.rs`、`crates/singlefs-core/src/background_compute.rs` 此刻在 git 里没有未提交改动，行号是此刻现查的）；kb 写 kb 文件加行号，行号是在 kb 文件里现查的。

## 先读这一节

模型与探针（要主 agent 跑，细节在文末「要主 agent 跑的命令」）：

- 复跑命令（整条交任务服务；先自证再打 96 格网格，输出写进运行目录的 `selftest.txt` 与 `grid.txt`）：
  `task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-design-r1-sonnet-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-design-r1-snapshot /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r1-sonnet-run /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-design-r1-sonnet-model`
- 内存与时间（推的，没量过）：纯标准库 Python，每格存一份前台等待列表（至多十几万个浮点数），峰值内存推在百兆以内；96 格每格十万量级事件，推在几分钟以内。
- 每个文件的 sha256（`sha256sum` 现算）：

| 文件 | sha256 |
|---|---|
| `research/prompts/m4-gpu-wait-outside-the-lock-design-r1-sonnet-model/lock_split_cost_model.py` | `484104fd8295c876140ce1bccc7c99f97e40f1dbababd1c90b2e9925216a9c9b` |
| `research/prompts/m4-gpu-wait-outside-the-lock-design-r1-sonnet-model/tools/rerun.sh` | `568676ec734ac9db70ce8fccb57923ddb5b66f0da06723a7afa7f0cfad9802d2` |
| `research/prompts/m4-gpu-wait-outside-the-lock-design-r1-sonnet-model/tools/rerun-from-a-clean-work-directory.sh` | `e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b` |
| `research/prompts/m4-gpu-wait-outside-the-lock-design-r1-sonnet-model/probes/mount_runtime_non_fsync_foreground_wait_baseline.rs` | `02fedc33414f83ded9d5fdd5dbeba3d0b66a01ec9b31044d86d41e27efed06a8` |

### 结论一览（每条的推翻条件在末尾「判定一览」）

1. **拆成三段，切点在 `advance_compaction_by_one_publish` 的调用两侧**：取段（锁里：判起跑、排空待发、开一批、取输入）、算段（锁外：建索引、选段循环、读盘、三段 GPU，不借会话也不借盘表）、收段（锁里：复核、计划、发布、记账）。今天的单次调用 = 三段首尾相接，拆开前后逐项相同（Q1）。
2. **读盘放哪里是真岔路**：选段循环的数据依赖是 丙（GPU 数每段活槽）→ 选段 → 读 R 的单元 → 乙 → 甲，所以「一批只放一次锁」要求读盘也在锁外，而 `SessionAndDevices` 把盘表和会话放在同一把锁里（`crates/singlefs-mount-runtime/src/lib.rs:396`）；活意图那一支 R 已知，取段时在锁里读完（至多 64 条记录）就行。候选与代价见 Q1。
3. **拆开新造了一条前台等 GPU 的路**：GPU 实现把手的 `Mutex` 在锁外等 GPU 时被后台线程持着，而前台 `switch_background_compute` 在会话锁里经校验去取同一把（`crates/singlefs-core/src/background_compute.rs:1176`），前台切换会在会话锁里等一整批 GPU，所有前台跟着等。不堵上，C998 在切换这条路上原样回来（Q1）。
4. **回锁复核**：「前台发过一次布就整批作废」最简，但在前台因空间被拒、`accept` 每次推抬 F 空发布的那一格上，每次 txg 都变，全空段低于底额的破例批永远发不出（活锁）；只核这一批的 R 的复核对活意图那一支成立（R 在意图期内排他），代价是回锁里多做不超过一段的查找（Q2）。
5. **让路**：登记点是 `accept`、`publish_user_change`、`switch_background_compute`、`with_the_session_and_devices` 四处，加上今天已有的 `publish_pending_now`；停因另起成员，不冒充 fsync；放锁之后先给前台靠「放锁后看计数再取锁」，`std::sync::Mutex` 本身不保证（Q3）。
6. **下限**必须随破例批一起设计：破例批若会被作废，下限就是空话（Q2、Q3）。
7. **崩溃**：拆开不新增盘上写序列，前提是「收段发布的内容等于同一输入在锁内重算的内容」；这一条是可测的等价性，不是新崩溃流（Q4）。
8. **卸载与 Drop** 默认还是等那一批（join，最长约 40 秒，读码算的）；不等要给等 GPU 的那一处加叫停，而 `crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs` 里没有叫停（Q5）。
9. 验收清单 14 条：替身卡能验 11 条，非真卡不可 3 条（Q6）。

## Q1　拆法的形态（主攻）

### 1.1 今天一次后台调用在锁里做的事（现查，全部在同一次取锁里）

运行库一拍里每次调用都是取锁、调一次核心层入口、放锁（`crates/singlefs-mount-runtime/src/lib.rs:613` 取锁，`crates/singlefs-mount-runtime/src/lib.rs:615` 调入口）。入口里按次序：

| 步 | 位置 | 读盘 | 等 GPU | 说明 |
|---|---|---|---|---|
| 判起跑 | `crates/singlefs-core/src/mounted_session.rs:3386` `whether_the_background_compaction_starts` | 可能（读挂载预算前移那一读） | 否 | 不起跑就不排空待发 |
| 排空待发 | 「let pending_changes_published_before_the_run = match self.publish_the_pending_changes(」（`crates/singlefs-core/src/mounted_session.rs:7664`） | 写 | 否 | 整理是屏障，待发先发 |
| 回退门 | 「stop_of_the_rollback_gate(parameters, &*devices, allocator, current)」（`crates/singlefs-core/src/mounted_session.rs:4778`） | 读根环与树表 | 否 | 每批之前一次 |
| 让路问答 | 「match caller.answer_before_the_next_batch() {」（`crates/singlefs-core/src/mounted_session.rs:4732`） | 否 | 否 | 读两个原子量，不取锁 |
| 开一批 | 「let Ok(batch_under_way) = background_compute.begin_a_batch(」（`crates/singlefs-core/src/mounted_session.rs:5123`） | 否 | 否 | 交设置、问实现，要取 GPU 把手的锁（`crates/singlefs-core/src/background_compute.rs:1098`） |
| 取输入 | 「compaction.take_the_quarantines_noted_by_the_publishes(allocator.compaction_mut());」（`crates/singlefs-core/src/mounted_session.rs:5367`）起到 5396 | 否 | 否 | 深拷一整版（`crates/singlefs-core/src/mounted_session.rs:5371`），走一遍映射树建条目表与按身份索引（`crates/singlefs-core/src/mounted_session.rs:5392`、`crates/singlefs-core/src/mounted_session.rs:5396`） |
| 借 GPU 实现 | 「let mut implementation = gpu_compute.lock();」（`crates/singlefs-core/src/mounted_session.rs:5160`） | — | 是，持到 5179 | 三段共用这一次借出 |
| 选段循环与扫描 | 「let scanned = scan_of_the_batch(region, &*devices, compute)」（`crates/singlefs-core/src/mounted_session.rs:5532`）、活意图支同一句（`crates/singlefs-core/src/mounted_session.rs:5634`） | 每段至多 64 条记录，每条 16 或 32 KiB | 乙、甲各一次提交，丙每批一次 | 读盘在 GPU 乙段之前（`crates/singlefs-core/src/region_scan.rs:549` 的 `for record in &records_in_region {` 先读完，`crates/singlefs-core/src/region_scan.rs:593` 才交 GPU） |
| 签发 / 一批发布 | 「allocator.compaction_mut().issue(issued);」（`crates/singlefs-core/src/mounted_session.rs:5598`）、「publish_the_planned_compaction_step_falling_back_on_the_region_check(」（`crates/singlefs-core/src/mounted_session.rs:5682`） | 写 | 否 | 一次发布 |

读单元的字节数：「跨度 1 读 16 KiB、跨度 2 读 32 KiB」（`crates/singlefs-core/src/background_compute.rs:189`），一个整段 64 槽，所以一次扫描读出至多 1 MiB（推的：槽 16 KiB，由这一句反推，没另量）。
选段循环的上界是单元区整段数（`crates/singlefs-core/src/mounted_session.rs:5494`：「每答一次「先扫」下一次就多一段有结论」），典型只有一两圈（推的，没量过：圈数随盘龄与 `choose_region_over_the_live_slots`（`crates/singlefs-core/src/compaction_plan.rs:189`）的评分而变）。

### 1.2 三段与接口草案

名字取英文描述性名，不带序号。三段首尾相接 = 今天的 `run_compaction_in_the_background_for_at_most_one_publish`（`crates/singlefs-core/src/mounted_session.rs:7559`）；核心层仍「不起线程、不持锁」（`crates/singlefs-core/src/mounted_session.rs:7555`：「挂着的会话这一层不起线程、不持锁，放锁是调用方的事」），三段之间放锁仍是调用方（运行库）的事。

| 段 | 在哪 | 入参 | 交回 | 谁持什么 |
|---|---|---|---|---|
| 取段 `begin_a_background_call` | 锁里 | `&mut` 会话、`&mut` 盘表、节奏、问答方、跨度（同今天入口） | `BackgroundCallBegun`：`FinishedInTheLock(调用结局)`（不起跑、让路停因、删除续做一步、只删意图、停着……凡不需要锁外算的路径，今天怎么走完这里怎么走完，不经算段）或 `BatchToComputeOutsideTheLock(Box<票>)` | 会话锁由调用方持；票是**自有值**（`Send + 'static`），不借会话也不借盘表；GPU 把手只带一份克隆（`crates/singlefs-core/src/background_compute.rs:719` 的把手 `Clone`，里面是 `Arc<Mutex<…>>`，`crates/singlefs-core/src/background_compute.rs:720`） |
| 算段 `compute_outside_the_lock` | 锁外，在维护线程上 | 票（按值）、读单元字节的读者（见 1.4）、问答方（只用来「放弃得早一点」，见 Q3） | `BatchComputedOutsideTheLock`：选中的段与扫描结果、这一轮不确定的记录、新记下的被拒段、`GpuWorkOfTheBatch`、`GpuSubmissionsOfTheBatch`、或 `BackgroundComputeError` | 算段自己借 GPU 实现一次（同今天 「一批里三段都经这一次借出算」，`crates/singlefs-core/src/background_compute.rs:739`），借出的守卫活在算段这个函数里；不碰会话 |
| 收段 `finish_a_background_call` | 锁里 | `&mut` 会话、`&mut` 盘表、算段交回的值、节奏、问答方 | 同今天入口的 `BackgroundCompactionAfterThePendingChanges` | 会话锁；复核（Q2）、持久状态的并回、孔洞判、签发或计划与发布、`end_the_batch` 记账、出错的停下与删意图都在这里 |

切点：`run_compaction_on_with_the_region_check_of_the_intent_deletion`（`crates/singlefs-core/src/mounted_session.rs:4452`）循环里调 `advance_compaction_by_one_publish` 的那一句（「} = advance_compaction_by_one_publish(」，`crates/singlefs-core/src/mounted_session.rs:4793`）之前归取段，之后归收段。至多一次发布的跨度下这个循环至多转一轮（`crates/singlefs-core/src/mounted_session.rs:4708` 的 `if span_of_the_call == SpanOfTheBackgroundCall::AtMostOnePublish`），所以只需要把循环外的局部量（预算、`reach`、`versions_published`、`background_compute_of_each_batch`、`was_frozen_when_the_run_started`、`finish` 闭包用到的开关快照）收成一个 `CompactionRunInProgress` 放进票里；整份函数约四百行，这是改动里最大的一块（读码估的，没量工作量）。
`advance_compaction_by_one_publish`（`crates/singlefs-core/src/mounted_session.rs:5110`）与 `advance_compaction_by_one_publish_computing_on`（`crates/singlefs-core/src/mounted_session.rs:5355`）本身再切：前者的进门（`begin_a_batch`、借实现之前）归取段，借实现与 `take_the_submissions_of_the_batch`（`crates/singlefs-core/src/background_compute.rs:665`）归算段，`end_the_batch`（`crates/singlefs-core/src/background_compute.rs:2103`）归收段；后者的上半（到选出 R 为止）归算段，下半（孔洞判、签发、计划、发布）归收段。

会话上多一个标记，防误用：`CompactionOfThisMount` 记「有一批在锁外」和它的戳（Q2）；这时再调取段，核心层拒（新拒绝成员，名字取 `BatchStillOutsideTheLock`）；收段收到戳对不上的值同样拒。运行库只有一条维护线程，不会撞；核心层的显式入口与可写挂载续做不经三段（它们没有锁外），遇到标记拒，不静默等。

### 1.3 取段在锁里拷什么、读什么

| 输入 | 今天 | 取段怎么交 | 锁外期间谁能改它 |
|---|---|---|---|
| 现行那一版 | 深拷一整版：`.clone();`（`crates/singlefs-core/src/mounted_session.rs:5371`） | 拷 `Arc<TransactionOutput>` 的指针：`pub fn shared_version_of_trees(&self) -> Option<&Arc<TransactionOutput>> {`（`crates/singlefs-core/src/transaction.rs:2637`）；「一版发出来之后不再改」（`crates/singlefs-core/src/transaction.rs:2551`） | 不改；前台发布换新的一版，旧的在票里的指针上活着。这一步本身还省掉锁里一次整版深拷 |
| 单元区 | `allocator.devices.first()` 取 | 值拷（`Copy`） | 几何挂载时定 |
| 开过的聚簇段 | `allocator.cluster_segments()` 借 | 克隆 `BTreeSet` | 前台发布开新段会改 |
| 隔离集 | `let quarantined_slots = compaction.quarantined_slots().clone();`（`crates/singlefs-core/src/mounted_session.rs:5393`） | 克隆；取段先并一遍待并入的隔离落点（`crates/singlefs-core/src/compaction_plan.rs:887`） | 前台发布会记下新的：`pub fn note_quarantined_placement(&mut self, slot: SlotNumber, span: u64) {`（`crates/singlefs-core/src/compaction_plan.rs:882`） |
| 被拒段表 | `pub segments_refused_by_the_region_scan: SegmentsRefusedByTheRegionScan,`（`crates/singlefs-core/src/compaction_plan.rs:918`） | 克隆，算段只写副本，新记的拒绝随结果交回 | 只有后台写 |
| 活意图与 R | `live_on_disk`（`crates/singlefs-core/src/mounted_session.rs:5422`） | 值拷 | 只有后台改（Q2 逐项核） |
| 后台计算设置 | `begin_a_batch` 读一次（`crates/singlefs-core/src/background_compute.rs:2055`） | `BackgroundBatchUnderWay`（`pub(crate) struct BackgroundBatchUnderWay {`，`crates/singlefs-core/src/background_compute.rs:1406`）按值放进票 | 前台切换只记成待生效（`crates/singlefs-core/src/background_compute.rs:1962` 起那段），生效只在下一次 `begin_a_batch` |
| 单元字节 | 在锁里一条条读 | 活意图支：取段里读完 R 的全部记录（至多约 1 MiB）放进票；签发支：见 1.4 | 取段里读的字节对着的就是戳那一版 |

### 1.4 选段循环每一圈读盘放哪里、一圈放几次锁（岔路，候选加代价）

数据依赖决定了岔路：没有活意图那一支的次序是 丙（GPU 数每段活槽，`crates/singlefs-core/src/mounted_session.rs:5488`）→ 选段（`crates/singlefs-core/src/compaction_plan.rs:189`，可能答「先扫」）→ 读 R 的单元 → 乙 → 甲 → 再选。读哪一段要等丙与选段算完才知道，所以「取段里一次读完」不成立（要读尽的是单元区所有可能被选的段，按 1.1 的数是每段至多 1 MiB，整池读不得）。

| 候选 | 做法 | 锁里读多少 | 要改什么 | 代价与风险 |
|---|---|---|---|---|
| 每圈回锁读 | 算段遇到要读的段就停下，交回 `NeedsTheBytesOfTheRegion`，运行库回锁读这一段（至多 1 MiB）再放锁继续；一批进锁 `1 + 圈数 + 1` 次 | 每圈至多 1 MiB | 算段改成可恢复的状态机（或把循环搬到运行库）；每次回锁都核戳 | 每圈都多一次「前台可能已发布」的窗，戳一变整批作废（Q2）；读盘仍占锁，盘慢（机械盘一段 64 次随机读，推的，没量）时锁里读的时长不比等一次 GPU 小；不碰盘所有权 |
| 读盘在锁外 | 盘表与会话分锁：读者持读锁读单元，发布持写锁；算段整批锁外 | 0 | `SessionAndDevices`（`crates/singlefs-mount-runtime/src/lib.rs:387`）拆成两把；`with_the_session_and_devices` 的 30 处测试用法（`grep -rn "locked\.session\|locked\.devices" crates/singlefs-mount-runtime/tests \| wc -l` 量的）要跟着改；与里程碑六二·1「先改设备所有权」同一处（`.claude/kb/milestone/06-sixth-txn.md:78`）；`std::sync::RwLock` 不保证写者不饿死 | 一批只放一次锁，最干净；前台发布写盘要等当前读者的一次 `read_at`；锁外读的是旧版本的单元，要靠 Q2 的戳挡住「单元已被释放又复用」的读（defer 窗口压着复用：「已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」（`.claude/kb/decisions/16-发布语义.md:35`）；锁外期间发布得比窗口还多时读到的就可能是复用后的新内容，推的） |
| 只拆活意图支 | 活意图那一支（R 已知）取段里读完、锁外只算乙甲；签发那一支整段仍在锁里 | 活意图支至多 1 MiB | 最小：只在 `advance_compaction_by_one_publish_computing_on` 的 `let Some((_issued_txg, intent)) = live_on_disk else {`（`crates/singlefs-core/src/mounted_session.rs:5422`）之后那一半拆 | 一条意图一趟至多 `1 + ⌈64 ÷ 16⌉ = 5` 次发布（`crates/singlefs-core/src/compaction_plan.rs:1557` 的注释）：五次里有一次是签发，签发那一次的丙、乙、甲仍在锁里等 GPU，所以不满足「等 GPU 拆到锁外」，只满足一部分；写成一个可用的第一步 |

### 1.5 GPU 实现那把锁（`BackgroundGpuComputeHandle` 的 `Mutex`）

- **谁持**：算段持。借出一次、三段与 `take_the_submissions_of_the_batch` 一并算完再放，与今天同形（`crates/singlefs-core/src/background_compute.rs:739`：「一批里三段都经这一次借出算」）。取段里 `begin_a_batch` 交设置与问状态也取这把锁（`crates/singlefs-core/src/background_compute.rs:1098` 与 `crates/singlefs-core/src/background_compute.rs:1116` 两处），那时算段没在跑（同一条维护线程串行），不互等。
- **新造的堵点**：前台 `switch_background_compute`（`crates/singlefs-mount-runtime/src/lib.rs:1046`）经 `request_switch_with_the_gc_cpu_compensation_threshold_from_the_next_batch`（`crates/singlefs-core/src/background_compute.rs:1942`）做与挂载同一套校验，校验里 `let implementation = gpu_compute.lock();`（`crates/singlefs-core/src/background_compute.rs:1176`）在会话锁里取把手锁。算段正持着它等 GPU 时（最长约 40 秒，见 Q5），这一个前台持着会话锁在把手锁上等，所有别的前台跟着等会话锁：C998 在切换这条路上原样回来，而且比今天更糟（今天前台至多等会话锁，不会同时持着会话锁去等别人）。
- 候选（也是岔路）：

| 候选 | 做法 | 代价 |
|---|---|---|
| 缓存卡清单 | 把手在递给核心层时带一份 `devices_with_total_memory` 的快照，校验读快照，不取实现的锁 | 卡清单对开着的与没开的卡都照列（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:337`：「没开的照列，切换可以点它」），不随开卡变；前提是一次挂载内卡不增减（没量过）；把手结构要加字段 |
| `try_lock` 拒绝 | 校验取不到把手锁就拒绝切换，报新错误 | 与 D26 已定项 13 第 4 条「就拒绝挂载（或拒绝切换）并报错」的拒绝范围不同（`.claude/kb/decisions/26-后台整理与放置回收.md:255`），要改条款；前台调用方得会重试 |
| 校验推到下一批开头 | 切换只记请求，取段里校验 | 同步拒绝变成异步拒绝，同条款「切换那一刻」读不通；调用方得不到当场的错 |

- **卸载与丢弃碰它**：`unmount` 先 join 线程（`crates/singlefs-mount-runtime/src/lib.rs:1123`）再取回会话，算段早已放了把手锁，不互等。
- **欠账 C943**（共用的后台 GPU 把手在交设置与读失败之间能被插队，`.claude/kb/checks-owed.md:558`）：取段里「交设置、问状态、决定走哪条路」是三次各自取放的短取锁，到算段再取一次，窗口的个数与今天相同（今天是「一批开头交完设置放锁、到整批取锁之间」）；大小只取决于取段结束到算段开借之间的间隔。算段**第一件事**就借实现、不在两段之间插让路等待，窗口不大于今天。产品路径一个把手只给一个会话（同一行），不触发。

### 1.6 CPU 路径拆不拆成同样的段（岔路）

| 候选 | 做法 | 代价 |
|---|---|---|
| 同样三段 | `OnTheCpu` 走同一套：算段里跑 CPU 写法的扫描与选段 | 一份代码；CPU 路径的读盘、建索引、扫描都出锁，前台等锁同样变短；GPU 与 CPU 两份实现在测试里对拍（「GPU 路径与 CPU 路径必须是两份实现」（`.claude/kb/decisions/24-后台重活能不能卸给GPU.md:38`）），取段与收段共用一套，两份实现只在算段里的核不同 |
| 只拆 GPU | CPU 路径照旧整批在锁里 | 改动小；但 CPU 路径锁里仍有读盘、整版映射走查与索引（`crates/singlefs-core/src/mounted_session.rs:5392`），这些今天就在锁里，前台等锁不会因为拆 GPU 变短；两条路径的并发形态不同，Q4 要各验各的 |

「前台等锁都让路」这一条用户已定，CPU 路径整批在锁里的那一段正是前台要让的；这是倾向同样三段的理由（推的），但取舍归用户。

## Q2　回到锁里之后复核什么才够（主攻）

### 2.1 原则与戳

算段交回的东西只是「对着戳那一版算出来的一份读数」。收段先在锁里用**此刻**的状态判这份读数还能不能用，能用才进入今天那一段代码（孔洞判、签发或计划、发布、记账）；那一段从 `let units_to_move = u64::try_from(scan.movable_units().count()).expect("一段 64 槽");`（`crates/singlefs-core/src/mounted_session.rs:5569`）起，逐字不动，只是 `version` 取此刻的现行版，不取票里的。戳取这几样（取段时在锁里记下，随票走）：现行版的 `checkpoint_txg` 与 `Arc` 指针、活意图（有或无与它的 R）、此刻开关快照（`allocator.compaction().switches`）、待并入隔离落点的个数、冻结与否、`in_effect` 设置的相等性。

### 2.2 判据表：逐项列锁外期间可能变的状态

「谁能改」栏里的前台发布指 `publish_pending_now`（`crates/singlefs-mount-runtime/src/lib.rs:981`）、`publish_user_change`（`crates/singlefs-mount-runtime/src/lib.rs:1020`）、`accept` 触发的发布与它为空间推的抬 F 空发布（`crates/singlefs-mount-runtime/src/lib.rs:950`：「全程持着会话那把锁」）；维护线程自己的 `publish_when_due_once`（`crates/singlefs-mount-runtime/src/lib.rs:559`）与算段串行，不并发。

| 状态 | 锁外谁能改 | 变了怎么判 | 对不上怎么办 |
|---|---|---|---|
| 现行版 txg 与 `Arc` | 前台任何一次发布；空发布也要（「空发布也是发布，按 D5（快照 / 空间记账机制） 已定项 2」要重写记账行、记账树节点、分配记录、映射条目与树表单元，`.claude/kb/decisions/16-发布语义.md:257`），所以空发布也让记录与条目表变 | 戳的 txg 等于此刻 txg 且 `Arc::ptr_eq` | 相等：原样用（与今天逐字相同）。不等：见 2.3，按「这一批是签发支还是活意图支」分流 |
| 分配器：空闲图 | 只在发布里变 | 孔洞数在收段里现算，不取自算段（`let holes = …` 在 `if !holes_admit_the_issue(holes, units_to_move) {` 之前，`crates/singlefs-core/src/mounted_session.rs:5581`） | 不需复核，本来就在锁里算 |
| 分配器：开过的聚簇段 | 前台发布开新段 | 选中的 R 的起点在不在此刻的开过的段里（选段资格第一条，`crates/singlefs-core/src/compaction_plan.rs:84` 起那段说明） | 在：这一段结果作废，签发支重来；活意图支不判（R 已签，不会被前台开成「开过的段」，R 排他，`.claude/kb/decisions/26-后台整理与放置回收.md:30`） |
| 分配器：活意图与 R 排他 | 只有后台 | 此刻 `live_compaction_intent_of_the_entries(&current.logged_operations)` 等于票里的，且分配器侧那一条仍与它一致（取段那一句断言 `assert_eq!` 在 `crates/singlefs-core/src/mounted_session.rs:5373` 起，收段重做一遍） | 不等：内部错误（有了第二个写者），收段断言，不静默作废 |
| 隔离集 | 前台发布里释放前核出对不上会记下新落点（`crates/singlefs-core/src/compaction_plan.rs:882`） | 收段先并入新记的（`crates/singlefs-core/src/compaction_plan.rs:887`），再看新增的槽与 R 罩住的槽有没有交集 | 活意图支：交集非空走今天已有的处置——R 里出现隔离槽就发一次只删意图的发布丢意图（`if region_holds_a_quarantined_slot(region, &quarantined_slots) {` 那一支，`crates/singlefs-core/src/mounted_session.rs:5631`），不是新规则；签发支：选中的段含新隔离槽就作废这一段 |
| 被拒段表 | 只有后台 | 算段只写副本；并回规则：只把「那一段的 R 局部复核过了」的新拒绝并进会话 | 复核不过的拒绝丢掉：它是对旧版的判，现版上可能已不成立，并进会话就成了本次挂载永久的排除（`crates/singlefs-core/src/compaction_plan.rs:60` 的 `note_refusal`） |
| 活意图（盘上那一条） | 整理意图只有后台改；前台改的是截断意图那一种（kb 登记两种种类） | 同「分配器：活意图」那一行 | 同上 |
| 后台计算设置 | 前台切换只能记成待生效（`crates/singlefs-core/src/background_compute.rs:1962` 附近）并解除停着；生效只在下一次 `begin_a_batch` | `end_the_batch` 里 `assert_eq!(self.in_effect, under_way.settings_read_at_the_start_of_the_batch, …)`（`crates/singlefs-core/src/background_compute.rs:2110`）原样保留在收段 | 断言不等是内部错误；锁外期间的切换请求按今天一批之内收到切换的处置：算段出错时 `if self.requested_from_the_next_batch.is_some() {`（`crates/singlefs-core/src/background_compute.rs:1741`）当场解除停下，沿用，不新增规则 |
| 冻结的发布 | 前台发布写盘失败，分配器记冻结，**txg 不变** | 收段先问 `allocator.frozen_publish().is_some()` | 冻结：不发，作废这一批，新停因（Q3 的命名风格），不记成整理失败。不先判的话，发布路径自己会拒（`fn refuse_while_a_publish_is_frozen(allocator: &PoolAllocator)`，`crates/singlefs-core/src/transaction.rs:1631`），而今天对被拒的处置是 `note_the_kind_of_a_publish_frozen_by_this_call(` 加 `CompactionRunOutcome::Failed(CompactionRunFailure::Batch(refused))`（`crates/singlefs-core/src/mounted_session.rs:4873`），一次无辜的前台冻结会被记成整理自己失败（读码推的，没造用例） |
| 待发（`accept` 收下、没发的） | 前台 `accept` | 收段看 `pending_changes_accepted()` | 候选：(一) 不管，批照发，待发不并进批，今天的「整理是屏障——待发里不并进整理的发布」（`crates/singlefs-core/src/mounted_session.rs:7528`）只保「不并进」，丢的是「待发先于整理批落盘」的次序；(二) 先排空再复核：排空是一次发布，txg 变，回到第一行。两者的代价量不出来，归用户取舍 |
| 预算与「上一条完成的意图」 | 前台用户改动发成会重置预算、清掉它（`.claude/kb/decisions/26-后台整理与放置回收.md:257`：「每次用户改动发成就清掉它」） | 收段按此刻值记账（`compaction.publishes_since_the_last_user_change += 1;`，`crates/singlefs-core/src/mounted_session.rs:4810`） | 前台只会让预算变宽，不需复核 |
| 开关快照 | 只供测试的开关（`with_the_session_and_devices` 闭包里能改任何东西，`crates/singlefs-mount-runtime/src/lib.rs:1075`） | 戳里的开关等于此刻的 | 不等：作废；产品路径没有人改 |

### 2.3 戳对不上（txg 变了）怎么办：整批丢掉，还是只认没变的那一部分（岔路，候选加代价）

先分两支，因为两支依赖的输入不同：

- **活意图支**（`let Some((_issued_txg, intent)) = live_on_disk else {` 之后，`crates/singlefs-core/src/mounted_session.rs:5422`）：算段只扫一个 R（`crates/singlefs-core/src/mounted_session.rs:5634`），R 在意图期内是分配目的地之外（`.claude/kb/decisions/26-后台整理与放置回收.md:30`：「`R` 在意图期内不可作分配目的地」），前台不会往 R 里分配，只会释放 R 里的单元、改它们的映射、隔离其中的槽。
- **签发支**（`else` 那一边，选段循环）：扫的是候选段，签发前判孔洞与「上一条完成的意图有没有产出」，这些读整版（`empty_cluster_segments_by_the_records`，`slots_holding_user_data` 要整版映射走查，`crates/singlefs-core/src/mounted_session.rs:4934` 起）。签发之前候选段不是 R，前台可以往里分配。

| 候选 | 做法 | 对哪一支 | 代价 |
|---|---|---|---|
| 整批作废 | 戳不等就不发，丢票，计一次作废，下一次调用从取段重来 | 两支都能用 | 最简，和今天的次序逐字相同；活锁暴露最大（2.5） |
| 只核 R | 活意图支：算段交回的结果只对 R；收段在此刻现行版上核 R 的输入没变就在**此刻现行版**上继续计划与发布：① 起点在 R 里的分配记录整体相等（`let records_in_region = match records_starting_in_the_region(region, allocation_records) {`，`crates/singlefs-core/src/region_scan.rs:534` 是扫描取记录的那一步），② 钉住与隔离的槽落在 R 里的部分相等，③ 扫描判成可搬的每个单元，它的映射 key 在此刻版里的位置项还是原来的，④ 判成死或不一致的记录，它头里的 key 在此刻版里有没有映射还是原来的（`let header_key_is_mapped = mapping_index.holds_mapping_key(&header.mapping_key);`，`crates/singlefs-core/src/region_scan.rs:740`），⑤ 开关快照相等 | 只活意图支 | 收段多做至多 64 条记录的区间比较与至多 64 次按 key 的映射查（`crates/singlefs-core/src/transaction.rs:11370` 的 `relocation_checked_against_the_version` 发布时本来就对每个计划单元查一遍，查的是同一类东西）；**最脆弱的一个前提**：前台不能凭空造出一条指向 R 里已有记录的新映射条目（回退复活、快照 deadlist 复活这类走不走前台入口我没核）。核不动就加上「此刻版映射条目数与戳相等」这一条保守判，代价是前台一写映射树就退回整批作废 |
| 签发支在锁里补算 | txg 变了，收段在锁里整版重走 `version_as_the_compaction_reads_it`（`crates/singlefs-core/src/mounted_session.rs:4934`）只为孔洞判与产出判，选中的 R 沿用 | 只签发支 | 把今天每批在锁里的整版映射走查留在了锁里，只出锁了 GPU 与读盘；前台写多时每个签发都付这笔 |
| 乐观：靠发布路径兜底 | 不核，直接在此刻版上发，让发布时的逐条计划核查（`RelocationBatchRefused::PlannedSourceIsNotTheMappingPosition {`，`crates/singlefs-core/src/transaction.rs:11457`）拒掉过期的单元 | — | 今天对被拒的处置是丢意图、暂停（`pause_by_dropping_the_live_intent_or_hand_up`，`crates/singlefs-core/src/mounted_session.rs:5705`），一个无辜的前台覆盖写会让整条意图被丢；还产生新写序列（Q4）。不取 |

只核 R 与整批作废的结果在被核的那几样没变时**逐项相同**，这一点可测：测试开关强制收段在锁内另算一遍 R 的扫描与计划，断言与锁外算的相等（Q6 第 9 条）；这个开关也满足 `.claude/rules/fs-design.md` 五条硬要求第 2 条「每条分支必须能被测试强制进入」（`.claude/rules/fs-design.md:78`）。

### 2.4 丢掉的那一批怎么记

| 账 | 怎么记 | 依据 |
|---|---|---|
| 发布预算 | 不动：没发布。今天只在发成时加一（`compaction.publishes_since_the_last_user_change += 1;`，`crates/singlefs-core/src/mounted_session.rs:4810`） | 读码 |
| 「用户改动之后至多一批」 | 不算一批：`CompactionRunStop::OneBatchAfterTheUserChangeDone` 的判据读已发成的批数（`crates/singlefs-core/src/mounted_session.rs:4705`） | 读码 |
| 这一趟没完 | 作废的停因归「趟仍在途」那一组，和被节奏截住同待遇：加进 `let is_held_back_by_the_pacing = matches!(`（`crates/singlefs-core/src/mounted_session.rs:3629`）那一组，运行库的 `keeps_going` 也认它（`crates/singlefs-mount-runtime/src/lib.rs:671`） | 读码 |
| 不进备忘 | `publishes_nothing_until_the_next_publish`（`crates/singlefs-core/src/compaction_plan.rs:672`）里这个新停因取 `false`：版本变了，下一次要重判 | 该函数注释「GPU 两份核不一致、被节奏或空间截住、出错的那几种下一次可能翻面，不进备忘」 |
| D26 已定项 13 第 8 条的连续不确定计数 | 住在 GPU 实现里（`crates/singlefs-core/src/background_compute.rs:588` 起那段说明），算段交回时已记；今天的规矩是「一批放弃（GPU 出错、等超过上限）时，那一批里已经交回的段照算一轮，放弃的那一段计数不动」（`.claude/kb/decisions/26-后台整理与放置回收.md:259`）。作废的一批沿用这句：已交回的段照算一轮。后果：前台一直发布时同一落点会在两批里各记一轮，较快到 N = 2 交 CPU 判；CPU 的判定是权威，所以更保守不更危险，但稀释了「隔轮」的本意。候选：给 `BackgroundGpuCompute` 加一个「丢掉这一批记的轮数」方法（改 GPU 包，没量过要不要） | kb 原句 |
| GPU 记账 | `end_the_batch(under_way, Some(gpu_work), None)` 照调（`crates/singlefs-core/src/background_compute.rs:2103`）；`BackgroundComputeOfOneBatch`（`crates/singlefs-core/src/background_compute.rs:1396`）加一个字段记「作废了、为什么」，满足五条硬要求第 4 条「分支必须可观测」 | 读码 |

### 2.5 活锁：有没有、下限怎么给

**结构性的一格：破例批永远发不出。** 全空段低于底额、前台已写不进时，这一趟第一次发布破例，排在让路之前（`.claude/kb/decisions/26-后台整理与放置回收.md:256`：「破例排在第 6 条的让路之前，前台有 fsync 在等也照发（前台已写不进，让路换不来什么）」）。前台写不进正是 `accept` 被空间拒、当场推抬 F 空发布再判的情形：一次准入「至多 B − 1 次」空发布（`crates/singlefs-mount-runtime/src/lib.rs:950`），每次空发布 txg 加一（`.claude/kb/decisions/16-发布语义.md:38`：「推一次不抬 F 的空发布（带现行 F、txg 加一」）。前台在 ENOSPC 上重试，txg 就一直在变。整批作废在这一格上**每一批都被作废**，GC 腾不出空段，前台永远写不进——这是反向饿死的一种（前台不断，后台一批都发不出），而且正落在用户 2026-11-09 定「破例每趟至少发 1 次」想救的那一格（`.claude/kb/decisions/26-后台整理与放置回收.md:269` 用户定案那一行）。所以**下限必须随破例批一起设计**。

**一般的一格：前台发布到达率 λ 与锁外那一段 W。** 整批作废时一批被作废的比例是 `1 − e^{−λW}`（假设前台发布泊松到达、任一次发布都作废；推的算术，W 没量过）：

| W（毫秒）＼λ（每秒） | 1 | 10 | 100 | 300 |
|---|---|---|---|---|
| 5 | 0.005 | 0.049 | 0.393 | 0.777 |
| 20 | 0.020 | 0.181 | 0.865 | 0.998 |
| 100 | 0.095 | 0.632 | 1.000 | 1.000 |
| 1000 | 0.632 | 1.000 | 1.000 | 1.000 |

（这张表是 `awk 'BEGIN{ … 1-exp(-L[i]*W[j]/1000) … }'` 现算的；模型 `lock_split_cost_model.py` 的自证里「split-discard-on-any-publish 作废比例」那一格用仿真对拍同一个式子，待主 agent 跑。）D24 已定项 2 射程写一次真卡提交约 5.3 ms（`.claude/kb/decisions/24-后台重活能不能卸给GPU.md:40`），一批有丙加每个扫描段的乙、甲，W 落在表里 5 到 20 毫秒两行附近（推的，真值要基线量）；每秒一百次前台发布时，作废比例在 0.39（W 取 5 毫秒）到 0.87（W 取 20 毫秒）之间。

**只核 R 时有界**：活意图支每次作废都要求 R 里某条记录被前台释放、隔离或映射被改；一条记录从「未释放」到「已释放」只一次，R 至多 64 槽，所以一条意图上作废次数有界（至多 R 里的记录数，推的，依赖 2.3 那个最脆弱的前提）。签发支没有这个界。

**下限候选**（岔路）：

| 候选 | 做法 | 代价 |
|---|---|---|
| 连着作废 k 次后整批放回锁里 | 运行库或核心层记「连续作废数」，到 k 下一批走今天的路径（取段到收段一口气在锁里） | 保证每 k+1 次尝试至少发一批；前台每 k+1 批会遇到一次今天那种锁内等 GPU（最长一次 GPU 等待）。k 取 2 时仿真格 `split-discard-on-any-publish-floor2` 给数（待跑） |
| 作废后降级复核 | 连着作废后下一批的复核降成「只核 R」 | 只对活意图支有用；签发支仍无下限 |
| 破例批永远走锁内 | 只对破例批整批放回锁里，别的批用整批作废 | 破例批本来就是前台已写不进、等锁的前台没有别的事可做；代价只落在最该推进的那一格，最小；但「前台已写不进」与「前台在等锁」要在取段里同判一次（`foreground_cannot_write_under_the_switches`，`crates/singlefs-core/src/mounted_session.rs:3649` 一带已有同一判） |

下限取什么观测（Q3 接着答）：连续作废数是核心层可见的（会话上记），不需要钟；「距上一次后台发布的时长」要运行库的钟。

## Q3　任何前台等锁都让路

### 3.1 计数登记在哪几处入口

今天只有 `publish_pending_now` 登记：`let _waiting = self.foreground_fsync_is_waiting();`（`crates/singlefs-mount-runtime/src/lib.rs:987`），守卫在 `pub struct ForegroundFsyncWaiting<'runtime> {`（`crates/singlefs-mount-runtime/src/lib.rs:441`）里，丢弃时减一。守卫变量先于锁守卫声明，所以先放锁、后减计数：后台看到计数回到 0 时，前台已经交出了锁。新增的登记点沿用这个声明次序。

| 入口 | 位置 | 今天 | 新 |
|---|---|---|---|
| `accept` | `crates/singlefs-mount-runtime/src/lib.rs:956` | 不登记；被空间拒时持锁推一串空发布（`crates/singlefs-mount-runtime/src/lib.rs:950`） | 登记「锁等待」 |
| `publish_pending_now`（fsync） | `crates/singlefs-mount-runtime/src/lib.rs:981` | 登记 fsync 等待 | 两个计数都登记（fsync 是锁等待的子集），后台仍分得出是不是 fsync |
| `publish_user_change` | `crates/singlefs-mount-runtime/src/lib.rs:1020` | 不登记 | 登记「锁等待」 |
| `switch_background_compute` | `crates/singlefs-mount-runtime/src/lib.rs:1046` | 不登记 | 登记「锁等待」 |
| `with_the_session_and_devices`（读路径 `read_at` 只能走它） | `crates/singlefs-mount-runtime/src/lib.rs:1075` | 不登记 | 登记「锁等待」 |
| 维护线程自己的取锁（`publish_when_due_once`、`resend_the_frozen_publish_if_any`、`run_one_maintenance_pass`） | `crates/singlefs-mount-runtime/src/lib.rs:559`、`crates/singlefs-mount-runtime/src/lib.rs:578`、`crates/singlefs-mount-runtime/src/lib.rs:600` | 不登记 | 不登记（它们就是被让的一方） |
| `unmount` / `Drop` | `crates/singlefs-mount-runtime/src/lib.rs:1122`、`crates/singlefs-mount-runtime/src/lib.rs:1200` | 先叫停线程 | 不登记（Q5） |

锁是 `std::sync::Mutex`：`locked: Mutex<SessionAndDevices<Device>>,`（`crates/singlefs-mount-runtime/src/lib.rs:396`），读与读也互斥。读路径高频时（多个读线程排队）「锁等待」计数几乎一直大于 0，后台因此一批都不起：**即使没有写，也会反向饿死**，所以下限（3.5）不是可选项。

登记时机是一处岔路：

| 候选 | 做法 | 代价 |
|---|---|---|
| 取锁之前无条件登记 | 同 fsync 今天的做法 | 每次读两次原子 `fetch_add`/`fetch_sub`（推的，每次几十纳秒，没量）；锁空着时也登记，后台会「让」一个根本没在等的前台 |
| `try_lock` 失败才登记 | `lock_the_session` 先 `try_lock`，拿到就不登记；`WouldBlock` 再登记后 `lock()` | 只在真有人等时计数，读路径无冲突时零开销；登记晚了一个 `try_lock` 的时间（微秒级，推的）；锁中毒的判法要照 `lock_the_session` 现有的一份（`crates/singlefs-mount-runtime/src/lib.rs:491`）保持 |

### 3.2 停因另起成员，不冒充 fsync

| 位置 | 加什么 | 语义 |
|---|---|---|
| `AnswerOfTheCallerBeforeTheNextBatch`（`crates/singlefs-core/src/compaction_plan.rs:1541`） | `ForegroundCallIsWaitingForTheLock` | 前台有入口在等会话锁，等的不是 fsync；两个计数都大于 0 时问答方答 `ForegroundFsyncIsWaiting`（更具体的那个） |
| `CompactionRunStop`（`crates/singlefs-core/src/compaction_plan.rs:623` 是 fsync 那个） | `YieldedToAForegroundLockWaiter`，`name()` 取 `compaction_yielded_to_a_foreground_lock_waiter` | 同 fsync 那个：不发下一批；趟仍在途 |
| 备忘 `publishes_nothing_until_the_next_publish`（`crates/singlefs-core/src/compaction_plan.rs:672`） | 新成员归 `false` 那一组 | 让路不是「版本不变就判同一个结论」 |
| 趟状态（`crates/singlefs-core/src/mounted_session.rs:3629` 的 `is_held_back_by_the_pacing`，`crates/singlefs-core/src/mounted_session.rs:3642` 的 `is_yielding_while_the_foreground_cannot_write`） | 新成员进这两处 | 前者让趟不丢；后者是低于底额时让路那一趟不留着（让破例下一趟还能发） |
| 运行库 `keeps_going`（`crates/singlefs-mount-runtime/src/lib.rs:671`） | 新成员与 `YieldedToAForegroundFsync` 同组 | 让路之后这一拍停，不接着空转 |
| `RuntimeObservations`（`crates/singlefs-mount-runtime/src/lib.rs:223`） | `yields_to_a_foreground_fsync`、`yields_to_a_foreground_lock_waiter`、`lock_waiters_registered`、`consecutive_yields`（下限用，一批发成清零） | 五条硬要求第 4 条「分支必须可观测」（`.claude/rules/fs-design.md:80`） |

一批现在有三处可以「看一眼前台」：

| 位置 | 谁判 | 取舍 |
|---|---|---|
| 取段之前（今天 `match caller.answer_before_the_next_batch() {` 那处，`crates/singlefs-core/src/mounted_session.rs:4732`） | 核心层问调用方，照旧 | 破例要在核心层判（`let is_this_publish_the_exception = budget_rule`，`crates/singlefs-core/src/mounted_session.rs:4721`），所以这一处必须留在核心层；若改成运行库在两次调用之间判，运行库不知道本批是不是破例批，破例「排在让路之前」就丢了 |
| 算段到收段之间 | 运行库 | 候选：(一) 让——收段排在前台后面，前台发布过戳就变，整批作废多；(二) 不让——收段只持锁一个发布的时长，前台多等的至多这一次发布，已算完的批不浪费。**不让**的理由是已经算完的批丢掉比多等一次发布更贵（推的）；严格读用户那句「任何前台等锁都让」则取让，归用户 |
| 算段进行中 | 问答方只用来「早点放弃」 | 前台发布过戳就已变，算下去只会被作废：算段在每段 GPU 之间看一眼「戳还对不对」，不对就早退（Q5 的超时重交也靠它省掉等待）。这是省时间，不是让路 |

### 3.3 放锁之后怎样保证前台先拿到锁

`std::sync::Mutex` 不保证公平：放锁的线程立刻再取可以赢过刚被唤醒的等待者。今天一拍里每次调用之后：记观测、送事件、`self.publish_when_due_once()?;`（`crates/singlefs-mount-runtime/src/lib.rs:685`，它又取锁），再下一次调用；两次取锁之间没有等前台的一步。候选：

| 候选 | 做法 | 代价 |
|---|---|---|
| 计数门（有界等） | 后台每次要取会话锁之前看「锁等待」计数；大于 0 就 `park_timeout`（条件变量，前台守卫丢弃时通知），至多等 `T_gate`；到时不管，继续取 | 前台在登记之后、后台判门之前到的，保证先拿到；登记与判门之间的微小窗里后台仍可能先取（取段/收段都很短，前台多等一次短段）；`T_gate` 同时就是时间型下限（3.5）；`T_gate` 的值没有依据，待量 |
| 后台只 `try_lock` | 后台取不到就不排队，下一轮再试；前台 `lock()` | 后台永不排在前台后面抢，但放锁瞬间后台 `try_lock` 仍可赢；要配计数门才完整 |
| 票号锁 | 运行库自己做一个 FIFO（`Mutex<(next, serving)>` 加 `Condvar`），所有入口过它 | 严格先到先得，不用拍 `T_gate`；每个入口多一次唤醒（`notify_all` 的惊群），`lock_the_session`（`crates/singlefs-mount-runtime/src/lib.rs:491`）要重写 |
| 换公平锁的库 | 如 `parking_lot` 的 `lock_fair` | 运行库今天只依赖核心层（`crates/singlefs-mount-runtime/Cargo.toml`），加依赖归用户 |

### 3.4 全空段低于底额的破例照旧不让

破例批的判在 `AnswerOfTheCallerBeforeTheNextBatch::ForegroundFsyncIsWaiting if is_this_publish_the_exception => {}`（`crates/singlefs-core/src/mounted_session.rs:4733`），新成员同样加一条 `if is_this_publish_the_exception => {}` 的臂。运行库那一层的计数门不知道破例，所以门必须有界（`T_gate`），或核心层给一个「这一批是破例」的读口让门看。破例批还要照 2.5「破例批永远走锁内」那一条整批放回锁里，不然会被前台的空发布作废。

### 3.5 反向饿死：要不要「至少推一批」的下限，取什么观测

要。上面 3.1 已经说了：读路径本身就能让计数一直大于 0。候选：

| 候选 | 观测 | 代价 |
|---|---|---|
| 次数下限 | 连续让路数 `consecutive_yields`（运行库计，批发成清零）达 N 后下一批不让 | 不要钟；N 的值没有依据；前台极快时 N 次让路也只是几毫秒，下限形同虚设 |
| 时间下限 | 距上一次后台发布的时长（运行库钟，`CallerOfTheBackgroundCompaction::clock_reading`，`crates/singlefs-core/src/compaction_plan.rs:1551`）达 `starvation_limit` 后下一批不让 | 与前台速度无关；要多一个挂载参数；值没有依据（和写带宽上限的默认值一样，等 C412 那类量） |
| 计数门有界 | `T_gate`（3.3）就是下限 | 一个参数兼两用；每次被让都等满 `T_gate`，前台每次多等的不多于它 |

下限批要不要走锁内、会不会被戳作废，见 2.5「破例批永远走锁内」那一条：下限批若会被作废，下限就是空话。

### 3.6 D26 已定项 13 第 6 条的改句草案

今天的原句（`.claude/kb/decisions/26-后台整理与放置回收.md:257`）：「前台有 fsync 在等时不发下一批，这一判别子要 GC 挪出调用线程之后才看得到；它只约束整理批（每一批之前问调用方一次），整理开头先走的那一遍删除不是整理批，不受它约束。」

改句草案（〔〕里是岔路没定的值，不替用户定）：

> 前台有任何入口在等会话锁（fsync、收下改动、单发改动、读、切换后台计算）时不发下一批，这一判别子要 GC 挪出调用线程之后才看得到；它只约束整理批（每一批之前问调用方一次；一批在会话锁外算完、回到锁里发布之前〔不再让路 / 也让路〕），整理开头先走的那一遍删除不是整理批，不受它约束。前台入口不断时整理仍至少〔每连续让路 N 次 / 每隔 starvation_limit / 每等满 T_gate〕推一批，这一批不让路；全空聚簇段数低于底额、前台已写不进时的破例批同样不让路，也不被前台的发布作废。

连带要改的句子：第 5 条「前台有 fsync 在等也照发」（`.claude/kb/decisions/26-后台整理与放置回收.md:256`）改成「前台有入口在等也照发」；D24 已定项 2 射程那一句「第一版一批之内周期线程持前台与后台共用的那把锁等 GPU（GPU 在锁内先接受）」（`.claude/kb/decisions/24-后台重活能不能卸给GPU.md:40`）改成「等 GPU 在会话锁外」；`crates/singlefs-mount-runtime/src/lib.rs:5` 的文件头注释与 `crates/singlefs-mount-runtime/src/lib.rs:1109` 的「一批之内 GPU 还在等时要等到那一批出来」按 Q5 的取舍改；欠账 C998 的检查按 Q6 第 4 条扩成读与 `accept`。

## Q4　崩溃语义与一个事务层变没变

### 4.1 盘上写序列：多出来的有没有

- **算段不写盘**（只读盘、算）。写只剩两处，都在锁里：取段里排空待发的那一次发布（今天同一处：`let pending_changes_published_before_the_run = match self.publish_the_pending_changes(`，`crates/singlefs-core/src/mounted_session.rs:7664`）与删除续做一步、只删意图等「在锁里走完」的路径；收段里的一次发布，走的还是 `publish_the_planned_compaction_step_falling_back_on_the_region_check(`（`crates/singlefs-core/src/mounted_session.rs:5682`）与 `publish_the_compaction_step_once`，即同一个事务状态机、同一个封闭的提交步骤枚举。没有哪一步变成第二条发布路径（`.claude/rules/fs-design.md:54`：「绕开状态机的第二条发布路径不允许」）。
- **发布次序的集合不变**：今天每次调用至多一次发布，两次调用之间放锁，前台发布可以插在后台两次发布之间（`crates/singlefs-mount-runtime/src/lib.rs:613` 取放锁一圈）。拆开之后前台发布多出来的位置是「取段里排空待发那一次」与「收段里那一次发布」之间；这两次发布都还是原子的、次序里任意交错的发布，盘上字节序列的**形状**都是「后台发布、前台发布若干、后台发布」，今天已有。
- **新东西不在写序列，在内容**：收段发布的内容是「对着此刻现行版、用锁外算的读数」生成的。只要复核过的那几样没变，它与同一输入在锁内重算逐项相同（Q2），写序列就与今天同一输入下的完全一致；这一条是可测的等价性（Q6 第 9 条），不是新崩溃流。
- **意图签发与 R 排他在锁外那一段是什么态**：活意图支——意图已在盘上、R 已在分配器侧排他（签发那次发布之前装上，`allocator.compaction_mut().issue(issued);`，`crates/singlefs-core/src/mounted_session.rs:5598`），锁外期间原样；这个「意图活着、没有人在推」的态今天就有（两次调用之间，以及卸载时「活意图不用做完：留在盘上，下次挂载续做」，`crates/singlefs-mount-runtime/src/lib.rs:1118`；用例 `fn unmounting_with_a_live_intent_leaves_it_for_the_next_mount_to_continue() {`，`crates/singlefs-mount-runtime/tests/mount_runtime_compacts_by_itself_and_unmounts_clean.rs:544`），只是持续时间变长。签发支——还没有意图、没有 R 排他，候选段只是候选，前台可以往里分配，这正是 2.3 签发支要核「记录在 R 里整体相等」的原因。

### 4.2 崩溃点重放要不要新流

| 候选 | 新写序列 | 要不要新流 | 依据 |
|---|---|---|---|
| 整批作废 | 无（作废不写） | 不要 | 作废后下一次调用从取段重来，盘上与「没发这一批」相同 |
| 只核 R（相等才用） | 无（与锁内重算逐项相同的前提下） | 不要，但要加 Q6 第 9 条的等价性用例；前提被打穿（核不动那个最脆弱的前提）时才要 | 现有流已经罩着「前台写插在两批之间、下一批对 R 重扫」：`crash-case:compaction-with-a-write-while-the-intent-is-live-stream`（`.claude/gate.d/stage-inputs.tsv:160`，「整理搬迁流第一批之后插一次覆盖写第一个文件，下一批对 R 重扫」） |
| 乐观（过期的计划直接发，靠发布路径拒） | 有：前台覆盖写了 R 里的单元，后台批被 `PlannedSourceIsNotTheMappingPosition` 拒，随后丢意图、暂停（`crates/singlefs-core/src/mounted_session.rs:5705`） | 要：一条「覆盖写 R 里的单元之后批被拒、丢意图」的新流；与 `crash-case:compaction-paused-short-of-space-dropping-the-intent-stream`（`.claude/gate.d/stage-inputs.tsv:161`）同形，但触发原因不同 | 所以 2.3 不取这个候选 |

新流（若要）比已有流多罩的状态：只有一类——「R 里的某个单元在意图期间被前台释放，而后台带着旧读数来发」那几个根槽落盘中间态；锁外那一段本身没有盘上状态（崩在锁外 = 崩在两次后台发布之间，盘上与「后台刚发完上一次」相同）。进程内的新同步点（计数门、票的取放）不在崩溃点重放的域里，归 `.claude/rules/fs-design.md:103` 一节说的并发交错工具；仓里 `litmus/` 现有 8 份都是发布屏障的次序（`ls litmus` 量的），没有运行库计数门的，要判它得另立一份（没量过值不值）。

### 4.3 有没有哪一步成了第二条发布路径

收段里的发布仍然只经 `publish_the_compaction_step_once` 一族；算段交回的只是读数。有两处要防的口子：一是「戳对不上时直接发」的乐观候选（已排除）；二是 `with_the_session_and_devices`（`crates/singlefs-mount-runtime/src/lib.rs:1075`）给的闭包可以改会话里任何东西，产品路径不调，用例摆开关才调，戳里放开关快照就是为它（Q2 判据表最后一行）。

## Q5　卸载、丢弃、切换、出错与 panic

| 事件 | 在哪处置 | 条款哪一句管它 | 候选与代价 |
|---|---|---|---|
| 卸载 `unmount`（`crates/singlefs-mount-runtime/src/lib.rs:1122`） | 先在锁外 join 维护线程（`fn stop_and_join_the_maintenance_thread(&mut self) {`，`crates/singlefs-mount-runtime/src/lib.rs:1110`），之后在锁里（拿回会话）排空待发、核心层卸载 | D26 已定项 13 第 4 条「卸载时不等还在跑的提交，有这种提交的那张卡整张泄漏、不析构」（`.claude/kb/decisions/26-后台整理与放置回收.md:255`） | **等那一批**（默认，同今天）：叫停（`stopping`）算段看不见，`grep -n "stopping\|cancel\|叫停" crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs` 零命中（我自己现跑的），join 等到算段做完，一段最长约 4 个等待上限（上限 10 秒，`pub const BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT`，`crates/singlefs-core/src/background_compute.rs:905`；重交 3 次，`pub const RESUBMISSIONS_AFTER_A_TIMEOUT: u32 = 3;`，`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:120`），约 40 秒；三段各在第 4 次才成时更长（推的，没量）。**叫停**：在等的那个循环里（`pub fn wait_for_completion_before_the_deadline<Output>(`，`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:99`）每轮看一个停标记，要把一次等分成短等；代价是改 GPU 包、每轮多一次唤醒；叫停后那次提交还在卡上，按第 4 条整张泄漏，卸载真的「不等还在跑的提交」。两者收段都先看 `stopping`，在停就不发（沿用 `CallerIsStoppingItsThread`，`crates/singlefs-core/src/compaction_plan.rs:1544`：「照样不发下一批」），活意图留在盘上 |
| 丢弃 `Drop`（`crates/singlefs-mount-runtime/src/lib.rs:1200`） | 同样先 join，不写盘 | 同上，加文件头「丢弃（Drop）只停线程，不写盘，等价于崩溃」 | 与卸载同，只是析构函数里阻塞最长约 40 秒（今天同，注释已写，`crates/singlefs-mount-runtime/src/lib.rs:1109`）；没法报错 |
| 切换到 `cpu` 或换一组卡 | 请求在锁里记成待生效；生效在下一次 `begin_a_batch`（锁里取段） | 第 1 条「从下一批开始生效；一批之内不换」（`.claude/kb/decisions/26-后台整理与放置回收.md:252`）；`.claude/rules/fs-design.md:81` 分支变量不许在操作中途改变 | 锁外那一批照旧算完，切换要等那一批（最长同上）才生效；「关掉全部开着的卡」发生在下一次取段，那时没有算段在持把手，不撞。切换的校验要不要等把手锁见 Q1.5 |
| GPU 交回出错 | 算段带着错出来，收段在锁里：`stop_the_task_after_a_compute_error` 加 `drop_the_live_intent_on_a_stop`（`let task_after_the_error = background_compute.stop_the_task_after_a_compute_error(`，`crates/singlefs-core/src/mounted_session.rs:5188`）；报警在同一次调用末尾的 `let alarm = self.session.alarm_of_the_compaction_at_the_end_of_a_wake();`（`crates/singlefs-core/src/mounted_session.rs:7728`）出 | 第 4 条「挂着的时候 GPU 出错……正在算的那一批放弃，意图也删掉」 | 停下、报警、按占用在 CPU 上补跑都在取段/收段（锁里）走，算段只出错、不处置；「判卡用不了那一刻读一次 PCI 状态」在 GPU 包的判断处，现在在锁外（读 sysfs 不再占会话锁，今天占着）。补跑的批走哪条（CPU 同样三段，还是锁内整批）见 Q1.6 |
| 等超时重交 3 次 | 全在 GPU 包里，算段内，会话锁外 | 第 4 条「在同一张卡上重交……至多 3 次，都超时才报警、判那张卡用不了」 | 前台不受影响；但这 40 秒里前台任何发布都会让戳变，算完也作废。省法：算段在每段 GPU 调用之间看戳（3.2 第三行），但 GPU 包内部的重交循环看不到戳，省不到段内 |
| 等的那一处 panic（C1030，`.claude/kb/checks-owed.md:632`） | 常见的「图形库等提交时报设备丢失而 panic」由外壳在等的那一处接住，照卡当场报出错办（同第 4 条末句「等卡回话的那一等里图形库 panic 的……外壳在等的那一处接住」）；没接住的 panic 在算段里展开到维护线程的 `catch_unwind`（`let body = catch_unwind(AssertUnwindSafe(|| {`，`crates/singlefs-mount-runtime/src/lib.rs:778`）记坏、送事件 | 文件头「线程或前台在锁里 panic 等价于崩溃」（`crates/singlefs-mount-runtime/src/lib.rs:14`） | 会话锁**没有**中毒、会话内容一致（算段不碰会话）；之后每个入口照 `lock_the_session` 先看坏标记报 `RuntimeFailed`（`if let Some(failure) = self.failure_so_far() {`，`crates/singlefs-mount-runtime/src/lib.rs:492`），卸载报 `UnmountRefused::Failed`（`if failure.is_some() || is_poisoned {`，`crates/singlefs-mount-runtime/src/lib.rs:1144`），可观测结果与今天相同，原因变了。**新口子**：GPU 把手的 `Mutex` 在算段 panic 时中毒，而 `lock()` 对中毒是 `.expect`（`.expect("GPU 实现上一次借出时 panic 过`，`crates/singlefs-core/src/background_compute.rs:746`）：此后取段里 `begin_a_batch` 的交设置、前台切换里的校验（`crates/singlefs-core/src/background_compute.rs:1176`）再取它会 panic，后者发生在持着会话锁的前台线程上，毒了会话锁。窗口是 panic 展开到 `note_the_failure` 记坏之间（毫秒级，推的）。候选：`lock()` 对中毒改成交回错误（改核心层，欠账 C1030 那类用例要补） |
| 前台在锁里 panic、会话锁中毒时锁外那一批怎么收 | 算段照常做完，收段 `lock_the_session()` 报 `RuntimeFailed`，线程体 `Ok(Err(_)) => {}` 静默退出（`crates/singlefs-mount-runtime/src/lib.rs:782`），票随栈丢掉 | 同上 | 算段没有在卡上留下未完成的提交时无事；卸载 join 等到算段做完（同第一行的最长），之后 `Failed` 交回盘 |
| 欠账 C943（共用的把手在交设置与读失败之间能被插队） | 取段里的几次短取放，与算段开借之间的间隔 | `.claude/kb/checks-owed.md:558` | **不变大**：窗口个数与今天相同；只要算段一开始就借实现、两段之间不插让路等待，窗口时长不大于今天（Q1.5）。产品路径一个把手一个会话，不触发 |

## Q6　怎么验才算数（验收清单）

「替身卡」指测试里自己实现 `BackgroundGpuCompute` 的假卡（`crates/singlefs-mount-runtime/tests/mount_runtime_compacts_by_itself_and_unmounts_clean.rs` 里已有 `GpuComputeThatOnlyAnswersTheInventory` 一类，探针里另给了一份照 CPU 写法算、每调用先睡一段的 `GpuDoubleThatSleepsLikeACard`），可以在调用里睡、panic、阻塞到测试放行、在回调里替前台动手；它验得了的是**锁的结构与回锁复核**，验不了的是真驱动、真 PCIe、真设备丢失。

| 序 | 要验的 | 怎么断言 | 替身卡 / 真卡 |
|---|---|---|---|
| 1 | **今天的基线**：非 fsync 的前台在维护线程一拍里等多久、拿到锁之前后台发了几次整理发布 | 探针 `probes/mount_runtime_non_fsync_foreground_wait_baseline.rs` 的两条（CPU 路径与睡 N 毫秒的替身卡路径），输出行 `name=baseline_foreground_wait ...`；先量出来再定第 2 条的上界。主 agent 此前量过一条形状相近但不同的：整趟入口 `run_compaction_in_the_background` 在一把锁里前台等 79152 微秒、期间发了 20 次整理发布（`research/prompts/m4-gpugc-background-runtime-design-r1-main-verification.md:20`），它走的是整趟入口、不经运行库放锁，不能当本条基线 | 替身卡 |
| 2 | 拆开之后同一用例 | 前台等锁时长上界取基线的若干分之一（数待第 1 条）；拿到锁之前后台发布数 ≤ 1（计数门生效）；弄坏开关（取消计数门）下必须红 | 替身卡 |
| 3 | 卡挂住的替身上前台各入口等多久 | 替身 `classify_region_records` 阻塞到测试放行；放行前依次发 `accept`、`publish_user_change`、`with_the_session_and_devices`（读）、`switch_background_compute`、fsync，各量取锁时长，断言都不含替身的阻塞时长；其中切换在 Q1.5 的堵点没做时必须红（它会在把手锁上等满） | 替身卡 |
| 4 | **C998 的检查扩到读与 `accept`**：「真卡上后台一批等 GPU 时前台发 fsync，断言前台取到锁之前没有等一次 GPU 提交」（`.claude/kb/checks-owed.md:602`）的结构版 | 替身在每次 GPU 调用里置「在飞」标记；前台取到锁那一刻读标记，断言为假；入口取 fsync、读、`accept` 各一条。拆之前的代码上这条必须红（探针证它判得出差别） | 替身卡（结构）；真卡版见第 13 条 |
| 5 | **回锁复核每一种对不上各一条**（替身在算段回调里替前台动手，再放行） | 前台发布过一次、txg 变；冻结（让盘写失败一次）；隔离集新增一槽落在 R 里；隔离集新增在 R 外；开过的聚簇段新增（签发支）；活意图变了（构造第二写者，要收段断言）；开关变；待发非空（按 2.2 候选）。每条断言收段交回什么停因、发没发布、盘字节有没有写、预算有没有动 | 替身卡 |
| 6 | 作废的账 | 预算不动、`OneBatchAfterTheUserChangeDone` 不触发、停因不进备忘（`publishes_nothing_until_the_next_publish`）、GPU 记账字段有值、第 8 条计数按「已交回的段照算一轮」记 | 替身卡 |
| 7 | **活锁与下限**：前台 ENOSPC 重试 `accept`，破例批仍然发出 | 照 `fn below_the_floor(`（`crates/singlefs-mount-runtime/tests/mount_runtime_compacts_by_itself_and_unmounts_clean.rs:942`）一带的造法压到「前台写不进」，前台线程不停重试 `accept`（每次空发布让 txg 变）；断言有界步数内破例批发成。无下限的弄坏开关下必须红（整批作废永远发不出） | 替身卡 |
| 8 | 反向饿死：读线程不断 | 几个线程不停 `with_the_session_and_devices`；断言后台至少每「下限」推一批，`consecutive_yields` 或时间观测不超过下限 | 替身卡 |
| 9 | **等价性**：只核 R 通过时，收段发布与锁内重算逐项相同 | 测试开关强制收段在锁内对 R 另扫一遍、另计划一遍，断言与算段交回的扫描与计划相等；前台在锁外随机发布（覆盖写、unlink、建 inode），断言通过就相等、不等就作废；加弄坏开关（复核恒过）下必须红。同一组输入下块序列与锁内整批逐字节相同 | 替身卡 |
| 10 | 卸载、Drop、切换、panic 在卡挂住的替身上 | 卸载：替身阻塞时调 `unmount`，量等多久（候选「等那一批」约等于阻塞时长；候选「叫停」应小于一次短等），之后盘重挂、活意图续做；`Drop` 同；切换到 `cpu`：当批照旧算完、下一批在 CPU；算段里 panic：会话锁没毒、入口报 `RuntimeFailed`、卸载交回盘；把手中毒后前台切换不 panic（若取 `lock()` 改法） | 替身卡 |
| 11 | GPU 与 CPU 两条路径在同一组输入上逐项相同 | 验证包 `compaction_batches_on_the_gpu_path_through_the_background_gpu_compute.rs` 与 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 一类照旧绿；拍子推的确定性路径上三段首尾相接，块序列必须与拆之前逐字节相同 | 替身卡（验证包里 CPU 撑着的替身） |
| 12 | 真卡上一批三段每段等多久、整批 W 的分布 | 即欠账 C926「量一批三段在各张卡上的提交等待时长分布」（`.claude/kb/checks-owed.md:548`）；用来给 2.5 的 W、3.3 的 `T_gate`、下限的值定数。仿真网格只给「W 取多少时作废比例多大」的形状，真值要这条量 | 非真卡不可 |
| 13 | 真卡上后台一批等 GPU 时前台读、`accept`、fsync 取到锁之前没有等一次 GPU 提交；同一组输入块序列与 CPU 逐字节相同 | 把 `fn the_maintenance_thread_compacts_on_the_real_gpus_while_the_foreground_fsyncs() {`（`crates/singlefs-background-gpu-tests/tests/mount_runtime_thread_on_a_real_gpu_compacts_while_the_foreground_fsyncs.rs:82`，文件头今天写「GPU 在锁内等一批」）扩到读与 `accept`，量取锁时长；`mount_runtime_on_a_real_gpu_writes_the_same_block_sequence_as_on_the_cpu.rs` 照旧绿 | 非真卡不可 |
| 14 | 真卡掉线时的 panic 路径（C1030） | 在 wgpu 层注入设备丢失或拔卡（`.claude/kb/checks-owed.md:632` 的「怎么拦」）；断言结局是任务停下、报警带 `UncapturedErrorOrDeviceLost`，不是 panic；加拆开之后会话锁不中毒这一条 | 非真卡不可（替身造不出驱动那条路，C1030 原话） |

替身卡 11 条、非真卡不可 3 条。第 1 条要在今天的代码上先量，第 2、3 条的数才有对照。

## 判定一览

| 格 | 判定 | 一句话 | 什么现象会推翻它 |
|---|---|---|---|
| Q1 三段 | 可写 | 切点在 `advance_compaction_by_one_publish` 两侧，三段首尾相接 = 今天的单次调用；改动大头是把 `run_compaction_on_with_the_region_check_of_the_intent_deletion` 的局部量收成 `CompactionRunInProgress` | 照 1.2 落代码后，拍子推的确定性路径上块序列与拆之前不逐字节相同；或循环外局部量多到票装不下 |
| Q1 读盘 | 岔路，三候选 | 每圈回锁读 / 盘表读写分锁 / 只拆活意图支 | 基线量出锁里读盘一圈远小于一次 GPU 等待（那就「每圈回锁读」够用）；或机械盘上读盘时长压过 GPU 等待（那就非分锁不可） |
| Q1 把手锁 | 会错的一格 | 前台切换在会话锁里取把手锁，会等满一批 GPU；要堵 | 第 3 条的卡挂住替身上，切换取锁时长不含阻塞时长（说明校验没取把手锁，则我读错了 `crates/singlefs-core/src/background_compute.rs:1176`） |
| Q1 CPU 路径 | 岔路 | 同三段 / 只拆 GPU | 基线 CPU 路径锁内时长远小于前台能感知的量 |
| Q2 复核表 | 可写 | 十二项逐项判；活意图支只核 R | 构造出一条前台入口能凭空造出指向 R 里已有记录的映射条目（最脆弱的前提）；或收段里「只核 R」的检查在锁里的时长不小于今天一批 |
| Q2 活锁 | 有，且在破例批上必现 | 整批作废 + 前台空间拒重试 = 破例批永远发不出；下限必须随破例批设计 | 第 7 条的用例在整批作废、无下限时仍然发得出破例批（说明前台空发布没让戳变，我读错了 D16 已定项 9） |
| Q3 登记与停因 | 可写 | 五处登记、新停因 `YieldedToAForegroundLockWaiter`、门有界 | 读路径登记的两次原子操作在基线里占去不可忽略的时长；或 `std::sync::Mutex` 在这台机器上放锁时实际是公平的（那么计数门多余） |
| Q3 下限 | 岔路 | 次数 / 时间 / 有界门，值没依据 | 第 12 条量出的 W 与前台速度使任何一个下限值都要么形同虚设要么吃掉让路 |
| Q4 | 不新增写序列、不要新流 | 前提：收段发布内容等于锁内重算 | 第 9 条的等价性用例红；或选了乐观候选 |
| Q5 | 六行各有处置 | 卸载默认等那一批；panic 默认记坏（等价崩溃）；新口子是把手中毒时 `lock()` 的 `expect` | 第 10 条里算段 panic 之后会话锁被毒（说明有取段外的东西持着会话锁跨过算段） |
| Q6 | 14 条 | 替身 11、真卡 3 | 第 4 条在拆之前的代码上不红（检查判不出差别） |

## 对 `crates/` 的改动清单（实现员照这张表落；每行来自上面哪一节）

| 文件 | 改什么 | 来自 |
|---|---|---|
| `crates/singlefs-core/src/mounted_session.rs` | 取段、收段两个入口与 `BatchToComputeOutsideTheLock`、`BatchComputedOutsideTheLock`、`BackgroundCallBegun`；`run_compaction_on_with_the_region_check_of_the_intent_deletion` 的局部量收成 `CompactionRunInProgress`；`advance_compaction_by_one_publish*` 按 1.2 再切；版本取 `Arc` | Q1 |
| `crates/singlefs-core/src/mounted_session.rs` | 收段复核（判据表）与新拒绝成员 `BatchStillOutsideTheLock` | Q2 |
| `crates/singlefs-core/src/compaction_plan.rs` | 新 `AnswerOfTheCallerBeforeTheNextBatch` 成员、新 `CompactionRunStop` 成员（让路、作废）、备忘与 `name()`；会话记「连续作废数」 | Q2、Q3 |
| `crates/singlefs-core/src/background_compute.rs` | 把手带卡清单快照（或 `try_lock` 拒绝）；`lock()` 对中毒改交回错误；`BackgroundComputeOfOneBatch` 加作废字段 | Q1.5、Q2.4、Q5 |
| `crates/singlefs-mount-runtime/src/lib.rs` | 「锁等待」计数与守卫、五处登记、计数门、算段在维护线程上跑、`RuntimeObservations` 新字段、文件头与 `stop_and_join_the_maintenance_thread` 的注释 | Q3、Q5 |
| `crates/singlefs-mount-runtime/src/lib.rs`（岔路定了「读盘在锁外」才改） | `SessionAndDevices` 拆盘表读写锁 | Q1.4 |
| `crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs`（岔路定了「叫停」才改） | 等的循环里看停标记 | Q5 |
| kb | D26 已定项 13 第 5、6 条、D24 已定项 2 射程按 3.6；C998 的检查改述 | Q3 |

## 要主 agent 跑的命令

每项三样：命令原样、在哪个目录起、跑完看哪几行。我自己一条都没往任务服务交，也没起 cargo。

**一、模型自证加网格**（纯标准库 Python，不碰 `crates/`、不写仓、不用卡）

- 命令：`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-design-r1-sonnet-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-design-r1-snapshot /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r1-sonnet-run /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-design-r1-sonnet-model`
- 在哪个目录起：任意（命令里都是绝对路径）；`--frozen` 位给的是开工快照目录（本轮是设计轮，没有冻结的树，那个目录里只有 `kb-sha256.txt`）。运行目录名以 `-run` 收尾、在工作根下，包装脚本先删它再跑。
- 看哪几行：任务末行 `TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号>` 里自证那一行 `自检查了 9 格，红 0 格`、没有 `✗ 自检：`；`name=rerun_done grid_lines=96`；运行目录里的 `grid.txt` 有 96 行 `name=lock_split_cell ...`。自证红了先别信网格（我没跑过，闭式与仿真对不上说明仿真有 bug 或我写错了闭式）。
- 弄坏开关证红（各一条，各自只跑自证）：`task-server/task submit -- env LOCK_SPLIT_MODEL_BREAK=never-discard python3 -I /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-design-r1-sonnet-model/lock_split_cost_model.py --selftest` 与把开关换成 `fg-never-waits` 的同样一条；前者看 `exit=1` 且至少「split-discard-on-any-publish 作废比例」「下限 2」两格是 `✗ 自检：`，后者看 `exit=1` 且「today 前台平均等」「split-discard-on-any-publish 前台平均等」两格红。
- 内存与时间：推的，没量过，见文首。

**二、今天的代码上的基线探针**（Q6 第 1 条，要先建仓副本；探针没编过）

- 步骤零（主 agent 直接跑，不经服务）：`python3 scripts/hosts/minimal-copy.py /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r1-sonnet/repo-copy --with-test-inputs`，再 `cp` 探针 `research/prompts/m4-gpu-wait-outside-the-lock-design-r1-sonnet-model/probes/mount_runtime_non_fsync_foreground_wait_baseline.rs` 到副本的 `crates/singlefs-mount-runtime/tests/`。
- 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-mount-runtime 的测试目标 mount_runtime_non_fsync_foreground_wait_baseline 里的 2 条测试; 范围=2 个测试; 为什么=在今天的代码上量非 fsync 前台等锁时长与拿到锁之前后台发布数的基线（设计轮 Q6 第 1 条）' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r1-sonnet/repo-copy -- env PROBE_REPEATS=5 PROBE_GPU_SLEEP_MILLISECONDS=5 cargo test -p singlefs-mount-runtime --test mount_runtime_non_fsync_foreground_wait_baseline -- foreground_wait_against_a_running_maintenance_thread_on_the_cpu_route foreground_wait_against_a_running_maintenance_thread_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1`
- 看哪几行：末行 `TASK_SERVER_RESULT` 的 `exit=0` 且日志里 `2 passed`；两行 `name=baseline_foreground_wait route=cpu ...` 与 `name=baseline_foreground_wait route=gpu_double sleep_ms_per_gpu_call=5 ...`，读 `wait_micros_mean`、`wait_micros_p99`、`wait_micros_max`、`samples_with_a_publish_in_between`、`max_publishes_in_between`。编不过的（我没编过）写进承接，新派同类 agent 改探针，不改 `crates/`。睡眠数 `PROBE_GPU_SLEEP_MILLISECONDS` 取 0、5、50 各一趟，看前台等锁时长是不是随它线性涨（基线「GPU 在锁内等」的直接证据）。
- 副本用完要删：`du -sh` 记大小，`task-server/task clean` 不碰副本，副本在草稿目录里由主 agent 自己 `rm`。

**三、没命令可写的**：Q6 第 12、13、14 条要真卡与服务交卡，我写不出命令，**复核不了**，原因：卡只归 GPU 批处理验证服务（`.claude/rules/verification.md`「卡只归 GPU 服务，任务不拿卡」），第 14 条还要先有能在 wgpu 层注入设备丢失的装置（C1030 原话）。

## 没做什么

- 不判别的腿的格、不替主 agent 采纳或出判决；岔路（读盘三候选、把手锁三候选、CPU 路径、待发处置、卸载取舍、下限形态与值、登记时机、门的形态）都只写了候选与代价，没替用户定。
- 没往统一任务服务交任务；判决要引的数没复跑：模型没跑、探针没编，命令都在上一节。
- 没读禁读清单里的文件（本轮攻方腿的报告、模型目录、草稿目录）。
- 没量：锁外那一段 W 的真值、选段循环典型圈数、锁里读盘一圈的时长（机械盘上尤其）、`std::sync::Mutex` 在这台机器上放锁后实际谁先拿到、一段内三次超时重交仍成功时的最长等待；这些在正文里都标了「推的」或「没量过」。
- 没核：回退复活与快照 deadlist 复活是不是只经前台入口之外的路径（只核 R 那条候选最脆弱的前提）；GPU 外壳「等的那一处接住 panic」的代码本身（只引了 D26 已定项 13 第 4 条与欠账 C1030 的描述，没读 `crates/singlefs-background-gpu/src/background_gpu.rs` 里的实现）；同一块盘上前台写与锁外读并发时 `RecordedMemoryDevice` 之外的真盘实现行不行。
- 引文：`crates/` 下我引的文件此刻在 git 里没有未提交改动（`git status --short` 对 `crates/singlefs-mount-runtime`、`crates/singlefs-background-gpu`、`crates/singlefs-background-gpu-tests` 与我引的那几份核心层文件量得 0 行；`crates/singlefs-core` 下另有 `make_filesystem.rs`、`mount.rs`、`recovery.rs`、`system_configuration.rs` 四个文件有未提交改动，我没引它们）；kb 里只有 `.claude/kb/checks-owed.md` 有未提交改动，它的行号取工作区此刻的样子，别的 kb 文件与 `.claude/gate.d/stage-inputs.tsv` 都是干净的；`.claude/gate.d/stage-inputs.tsv` 的行号同样是此刻现查的。
- 偏差一处：报告里有一处行号（`mounted_session.rs` 的 3646 改 3649）我用 `sed -i` 就地改了，写法不合「只许点名的写入工具」那一条；内容只是一个行号，改前改后的引文都用 `cite-check.py` 判过。
- 没建仓副本、没起编译目录，没有要删的东西；写过的只有模型目录与草稿目录里的 `rename_policies.py`、几份 `fix*.json`（改报告用的替换规格）与那份重命名中间稿。
