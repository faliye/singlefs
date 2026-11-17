# m4-gpugc-background-runtime-design-r1 云端正推腿（Sonnet）报告

立场：云端正推。按 `crates/` 今天的实现从条款与代码推「应该是什么」，再与背景材料逐格比；一样写一样，不一样把两边整行并排抄出。引代码写主工作区路径与行号（读的时候逐处 `grep -nF` 过）。数分三类：**量的**（我自己读文件、`grep` 得的）、**推的**（读码推出来、没量过）、**拍的**（模型参数，不是量的）。我没起 cargo、没往服务交任务。

模型目录 `research/prompts/m4-gpugc-background-runtime-design-r1-sonnet-model/`（纯算术，只读 `--frozen` 树里几个常量对锚点）：

- 复跑：见末尾「要主 agent 跑的命令」第 1 项。我这边只做了 `python3 -m py_compile` 与 `bash -n`，模型本身没跑；报告里引模型的数都是我手算后写进断言的，**主 agent 跑出来才算数**（跑前一律当「推的」）。
- 文件 sha256 见末尾「文件清单」。

## 先读这一节：背景材料没点到、但「接入并运行」一定撞的五处

背景材料的题面把「挂载就起的周期线程」「三条水位真正实现」当成缺的两样。读完 `crates/` 之后，缺的不止这两样。下面五处任何一处不先定，周期线程起来也是「空转」或「卡死」，验收第一条（挂载之后不调任何后台入口，GC 自己跑、腾出空段）过不了。

| 名 | 事实 | 后果 | 谁来定 |
|---|---|---|---|
| 批大小 K 没有产品值 | 「`let batch_units = match switches.batch_size {`」（`crates/singlefs-core/src/mounted_session.rs:3753`），产品值「batch_size: CompactionBatchSize::NotDecidedByTheClauses,」（`crates/singlefs-core/src/compaction_plan.rs:570`），这一臂「CompactionRunOutcome::Stopped(CompactionRunStop::BatchSizeNotDecidedByTheClauses),」（`crates/singlefs-core/src/mounted_session.rs:3760`）在签任何意图之前返回 | 水位判据即使真越线，周期线程每次醒来都在签意图之前停下，一个写都没发；挂载续做盘上留下的活意图同理停下 | 用户（K 归 E10）；下界由预算算出，见 Q3 与模型 A |
| 起跑判据是常量函数 | 「pub const fn compaction_starts_by_the_watermarks(forcing: CompactionRunForcing) -> bool {」（`crates/singlefs-core/src/compaction_plan.rs:1405`），两处调用：「if compaction_starts_by_the_watermarks(allocator.compaction().switches.run_forcing) {」（`crates/singlefs-core/src/mounted_session.rs:2627`）与「if !compaction_starts_by_the_watermarks(allocator.compaction().switches.run_forcing)」（`crates/singlefs-core/src/mounted_session.rs:2937`） | 它连读数都不收；要改签名，两处调用与所有只供测试的开关调用点都要动 | 实现员（Q2 给签名） |
| 闲置预算是一份 | 预算「(3 * root_ring_slots_per_region)」减 4（`crates/singlefs-core/src/compaction_plan.rs:1383`），mkfs 默认 S 是「默认 S = min(8, ⌊P × chunk ÷ 固定结构槽距⌋)，小于 5 拒绝 mkfs」（`crates/singlefs-format/src/lib.rs:305`），所以 mkfs 做出的池 S 在 5 到 8，预算在 11 到 20 次发布（量的：公式与区间读自 `lib.rs`，模型锚点对过）；归零只在成功的用户改动里：「compaction.publishes_since_the_last_user_change = 0;」（`crates/singlefs-core/src/mounted_session.rs:2621`），被拒的改动在「let mut published = published?;」（`crates/singlefs-core/src/mounted_session.rs:2596`）就返回 | 挂载之后没有用户改动，周期线程总共只能发 20 次整理发布，之后每次醒来都报 `PublishBudgetOfTheLastUserChangeSpent`；全空段不够时前台写入类发布又被合取 7 拒，**拒绝的改动不归零预算**⇒ 可达死局（Q3 H3，推的，没量过） | 用户（岔路 F-3） |
| 一趟没产出的醒来是 O(容量) | 起跑判过之后，哪怕马上停在「没有能签的段」，也先做「let read_of_the_version = version_as_the_compaction_reads_it(&version);」（`crates/singlefs-core/src/mounted_session.rs:4439`）与逐段 `ScanBeforeChoosing` 读盘（`crates/singlefs-core/src/mounted_session.rs:4578`）；单元区第一块盘取「.expect("池里至少一块盘：两盘同槽，选段与判类按第一块盘的记录做（判决 G1 攻方 K-n）");」（`crates/singlefs-core/src/mounted_session.rs:4432`） | 水位持续越线而选不出段时，每 30 秒在锁里做一遍 O(分配记录数 + 映射条目数) 的活（推的，没量过）；需要「没产出的醒来」备忘（Q3） | 实现员 |
| 核心层只有 `&mut` 同步入口，且没有产品调用方 | 「调用方全在 E202 的装置与验证包测试里」（背景材料第二节，我 `grep -rln run_compaction_in_the_background crates` 复核：测试与 `crates/singlefs-verification/src/history.rs`、E202 装置，产品代码零命中，量的）；崩溃枚举的整理流走的是显式入口「let run = session.run_compaction(devices)」（`crates/singlefs-verification/tests/common_compaction_relocation_on_the_background_compute/mod.rs:248`），不是后台入口 | 背景材料说「今天的崩溃枚举与模型对拍都是单线程调后台入口」，我现查对不上：崩溃枚举流调显式入口，后台入口只在确定性用例里被调（Q5） | 主 agent 判 |

## Q1　运行库的形态（主攻）

### 1.1 新包放哪、叫什么、依赖方向

**应该是**：新包 `crates/singlefs-mount-runtime`（运行库），加进根 `Cargo.toml` 的 `members`（现有成员表见 `Cargo.toml:6-16`）。依赖只指向核心层 `singlefs-core`，**不依赖 `singlefs-background-gpu`**，也不带 `[features]`。

理由（逐条可核）：

- 产品 GPU 包的 `wgpu` 是无条件依赖：「wgpu = { version = "=30.0.1", default-features = false, features = ["vulkan", "wgsl", "std"] }」（`crates/singlefs-background-gpu/Cargo.toml:11`）；`tests-on-a-real-gpu` 特性只挡测试（背景材料第二节，我读 `Cargo.toml` 核过）。运行库若直接依赖它，不开 GPU 的调用方也拖上 wgpu。
- 把 GPU 做成运行库自己的可选特性行不通，原因是验证包要依赖运行库做交错与崩溃验证：「不设 [features]：验证包的构建脚本取直接依赖的特性要依赖带 links 与构建脚本」（`crates/singlefs-device-library/Cargo.toml:8`），带特性而不带 `links` 的直接依赖让验证包构建当场停下。这一条我只读了注释，没编过（推的）。
- 验证包与产品 GPU 包不共用代码，是门禁的要求（`crates/singlefs-background-gpu/src/lib.rs` 文件头）；运行库不依赖产品 GPU 包，验证包就可以依赖运行库而不破这一条。

**GPU 开卡的口子反转**：运行库里定义一个特质，调用方（产品入口，今天没有）把实现递进来。`BackgroundGpuComputeHandle` 是核心层的类型（「pub struct BackgroundGpuComputeHandle {」，`crates/singlefs-core/src/background_compute.rs:433`），挂载入口收的也是它（「pub fn mount_writable_with_background_compute_settings_and_the_gc_cpu_compensation_threshold<」，`crates/singlefs-core/src/mount.rs:9587`），所以运行库不需要认识产品 GPU 包的任何类型。粘合代码（把 `open_background_gpu_compute` 包成特质实现，持有进程级实例与泄漏账）放产品入口 crate 或一个只做粘合的小包；不放运行库。条款「挂载时按参数开卡、建实例、挂载、起周期线程」（`.claude/kb/decisions/26-后台整理与放置回收.md:249`）仍由运行库的 `mount` 发起——它调开卡器、再挂载、再起线程——只是开卡器的实现住粘合层，所以这不是改条款。

### 1.2 谁持有会话与盘表

**应该是**：`MountedSessionWithPendingChanges` 加 `Vec<(DeviceIdentity, Device)>`，一起放进同一把锁。

- 前台入口 `accept`、`publish_pending_now`、`publish_user_change` 与周期入口 `publish_when_due`、后台入口 `run_compaction_in_the_background` 都在攒批的会话上（`crates/singlefs-core/src/mounted_session.rs:5718`、`:5959`、`:5977`、`:5941`、`:6021`）；只持 `MountedSession`（`crates/singlefs-core/src/mounted_session.rs:110`）就没有 `accept` 与 `publish_when_due`。
- 卸载要会话本身：「pub fn into_session_after_publishing_the_pending_changes<Device: BlockDevice>(」（`crates/singlefs-core/src/mounted_session.rs:6102`）先排空待发再交出。
- 盘表归运行库：每个核心入口都收 `&mut Vec<(DeviceIdentity, Device)>` 并先核盘表与挂载核过的是否一致。运行库**拥有**盘（按值收进来），不借，否则线程拿不到 `'static`。
- 线程间移动的前提是 `Send`。我读了 `PoolAllocator`、`CompactionOfThisMount`、`BackgroundComputeOfThisMount` 的字段，没发现 `Rc`、`RefCell`、无 `Send` 的 `dyn`；GPU 把手是「shared: Arc<Mutex<dyn BackgroundGpuCompute + Send>>,」（`crates/singlefs-core/src/background_compute.rs:434`）；文件盘 `FileBackedBlockDevice` 的字段是 `File` 加整数（`crates/singlefs-core/src/block_device.rs:599`）。**推的，没编过**：草稿目录里有一份编译断言 `assert_the_session_moves_between_threads.rs`，命令见「要主 agent 跑的命令」第 2 项。区域扫描的计数器是 `thread_local!`（`crates/singlefs-core/src/region_scan.rs:60`）：测试若在主线程读后台线程里的计数会读到 0，运行库的测试要把读计数也放进同一条线程里做。

### 1.3 接口草案（类型与签名；是草案，不是实现）

```rust
// crates/singlefs-mount-runtime/src/lib.rs —— 依赖只有 singlefs-core
pub struct RuntimeParameters {
    pub background_compute: BackgroundComputeSettings,            // core background_compute.rs:623
    pub gc_cpu_compensation_threshold: BackgroundGcCpuCompensationThreshold,
    pub compaction_watermarks: CompactionWatermarks,              // 新，core compaction_plan.rs（Q2）
    pub compaction_period: Duration,                              // 默认 30 s；大于 0
    pub publish_poll_period: Duration,                            // D16 的 P；大于 0
    pub write_bandwidth_limit: CompactionWriteBandwidthLimit,     // core compaction_plan.rs:1326
    pub gpu_open_deadline: Duration,                              // 整个开卡（含枚举）的期限，Q4
}
pub trait OpensTheBackgroundGpu: Send + Sync {                    // 产品入口把实现递进来
    fn open(&self, settings: &BackgroundComputeSettings, give_up_after: Duration)
        -> Result<OpenedBackgroundGpu, GpuOpenRefusal>;
}
pub struct OpenedBackgroundGpu { pub handle: BackgroundGpuComputeHandle, pub report: GpuOpenReport }
pub trait ClockOfTheRuntime: Send + Sync {                        // 钟与睡眠都能换成假的
    fn reading(&self) -> CallerMonotonicClockReading;
    fn sleep_until(&self, wake_at: CallerMonotonicClockReading, interrupted: &dyn Fn() -> bool);
}
pub trait ReceiverOfTheAlarms: Send + Sync {                      // BackgroundCompactionCall 是 #[must_use]
    fn call_returned(&self, call: &BackgroundCompactionCall);     // 后台入口每次的结局都送这里，含报警
}
pub struct MountedRuntime<Device> { /* Arc<Shared<Device>> + JoinHandle */ }
impl<Device: BlockDevice + Send + 'static> MountedRuntime<Device> {
    pub fn mount(parameters: &MakeFilesystemParameters, devices: Vec<(DeviceIdentity, Device)>,
                 runtime: RuntimeParameters, gpu: Option<&dyn OpensTheBackgroundGpu>,
                 clock: Arc<dyn ClockOfTheRuntime>, receiver: Arc<dyn ReceiverOfTheAlarms>)
        -> Result<Self, RuntimeMountRefused<Device>>;             // 拒绝时盘原样交回
    pub fn accept(&self, change: UserChange<'_>) -> Result<ChangeAccepted, RuntimeRefused<AcceptRefused>>;
    pub fn publish_pending_now(&self)                             // fsync 走这里，进门先登记「有 fsync 在等」
        -> Result<Option<PublishReportedByTheSession>, RuntimeRefused<PublishOfThePendingChangesRefused>>;
    pub fn publish_user_change(&self, change: UserChange<'_>)
        -> Result<Vec<PublishReportedByTheSession>, RuntimeRefused<UserChangeAfterThePendingChangesRefused>>;
    pub fn switch_background_compute(&self, settings: BackgroundComputeSettings,
        gpu: Option<BackgroundGpuComputeHandle>, threshold: Option<BackgroundGcCpuCompensationThreshold>)
        -> Result<(), RuntimeRefused<BackgroundComputeSettingsRefused>>;
    pub fn observations(&self) -> RuntimeObservations;            // 最近一次醒来的结局、累计、报警次数
    pub fn run_one_maintenance_pass(&self) -> MaintenancePass;    // 线程体的一次循环；测试不起线程直接调
    pub fn unmount(self) -> Result<UnmountReport<Device>, UnmountRefused<Device>>;
}
pub enum RuntimeRefused<E> { Refused(E), Failed(RuntimeFailure) } // Failed：后台线程或前台在锁里 panic 过，只能重挂
```

**与核心层 `&mut` 入口怎么接**（左边是核心层今天的入口，右边是运行库的包法；行号都是 `grep` 过的）：

| 核心层入口 | 运行库怎么调 | 要不要动核心层 |
|---|---|---|
| 挂载 `mount_writable_with_background_compute_settings_and_the_gc_cpu_compensation_threshold`（`crates/singlefs-core/src/mount.rs:9587`） | `mount` 里在开卡之后调；盘用 `&mut` 借给它，成功后随会话一起进锁 | 要：挂载内部写死「CompactionSwitches::PRODUCT,」（`crates/singlefs-core/src/mount.rs:9615`）交给内部的「mount_writable_with_every_test_only_switch」（`crates/singlefs-core/src/mount.rs:9695`），水位与批大小要从挂载参数进，见 Q2 |
| `MountedSession::of_the_mount`（`crates/singlefs-core/src/mounted_session.rs:2443`）、`MountedSessionWithPendingChanges::of_the_session`（`:5591`） | `mount` 末尾 | 不动 |
| `accept`、`publish_user_change`、`publish_pending_now`（`:5718`、`:5977`、`:5959`） | 取锁后原样调；`now` 由运行库的钟读 | 不动 |
| `publish_when_due`（`:5941`） | 维护线程每个 `publish_poll_period` 调一次 | 不动 |
| `run_compaction_in_the_background`（攒批版，`:6021`） | 维护线程每个 `compaction_period` 调一次，`caller` 由运行库实现（下节） | 要：起跑判据换成读数函数（Q2）、没产出的醒来备忘（Q3） |
| 挂着切换 `switch_background_compute_from_the_next_batch`（`:2726`）与带阈值那个（`:2741`） | `switch_background_compute` | **要**：攒批的会话只有「pub fn session(&self) -> &MountedSession {」（`crates/singlefs-core/src/mounted_session.rs:5619`），没有 `&mut` 的口子，`grep -n 'fn session_mut' …/mounted_session.rs` 零命中（量的）；要在攒批的会话上加两个同名转发方法 |
| 卸载 `into_session_after_publishing_the_pending_changes`（`:6102`） | `unmount` 停线程之后调 | 不动 |

### 1.4 一把锁够不够：一批整理持锁多久，前台会被挡多久，「前台有 fsync 在等」谁来答

**正确性上一把锁够**：核心层每个入口都是 `&mut self` 的同步调用，一把锁把前台与后台串成「入口调用的某种次序」，与核心层今天被测的单线程次序同一类（Q5 逐格比）。

**「前台有 fsync 在等时不发下一批」在一把锁下答得出，但谓词必须放在锁外面**。条款原文：「前台有 fsync 在等时不发下一批，这一判别子要 GC 挪出调用线程之后才看得到；它只约束整理批（每一批之前问调用方一次），整理开头先走的那一遍删除不是整理批，不受它约束。」（`.claude/kb/decisions/26-后台整理与放置回收.md:250`）。核心层问的位置是「match caller.foreground_fsync() {」（`crates/singlefs-core/src/mounted_session.rs:3841`），问的时候后台线程正拿着会话锁。所以：

- 共享状态里放一个 `AtomicU32 fsync_waiters` 和一个 `AtomicBool stopping`，都不在会话锁里；
- 前台 `publish_pending_now` 进门先 `fetch_add(1)`，**取锁之前**；返回（含错误、panic 展开）时 `fetch_sub(1)`（用守卫对象，丢弃时减）；
- `CallerOfTheBackgroundCompaction`（「pub trait CallerOfTheBackgroundCompaction {」，`crates/singlefs-core/src/compaction_plan.rs:1371`）由运行库实现：`foreground_fsync()` 在 `fsync_waiters > 0 || stopping` 时答 `Waiting`，否则 `NoneWaiting`；`clock_reading()` 答运行库的钟。卸载时让它答 `Waiting`，整理在下一批之前就返回，线程很快能 join。

**前台被挡多久**（持锁的几段，哪一段有上界）：

| 持锁的一段 | 上界 | 依据 |
|---|---|---|
| 起跑判（`whether_the_background_compaction_starts`） | O(1)，不读盘 | 「一个盘都不读、一个写都不发」（`crates/singlefs-core/src/mounted_session.rs:2867`） |
| 判过起跑之后先排空待发 | 一次发布 | 「let pending_changes_published_before_the_run = match self.publish_the_pending_changes(」（`crates/singlefs-core/src/mounted_session.rs:6053`） |
| 先走空的删除续做 | 无节奏约束，步数随排队的删除意图 | 条款明说不受 fsync 约束（上面的 `:250`），代码在「publish_the_queue_until_it_is_empty(」（`crates/singlefs-core/src/mounted_session.rs:3779`）之前没有 `foreground_fsync` 一问 |
| 整理批 | 一个已发批 + 其前的读盘与计算；GPU 等最长 10 s、同张卡超时重交至多 3 次 | 「pub const BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT: Duration = Duration::from_secs(10);」（`crates/singlefs-core/src/background_compute.rs:619`）；重交条款在 D26 已定项 13 第 4 条（`.claude/kb/decisions/26-后台整理与放置回收.md:248`） |
| 一趟之后的 discard | 随整段可再分配的段数 | 「run.discards = discard_the_segments_reusable_as_a_whole(」（`crates/singlefs-core/src/mounted_session.rs:3018`） |

各段的**墙钟都没量过**：欠账 C412 自陈「前台抢 I/O 与产品默认路径的挂钟仍没有数」（`.claude/kb/checks-owed.md:286`），E185 量的是一批搬迁的拆分，不是持锁时长。所以「一批整理持锁多久」我只能给结构，不能给数（推的，没量过）；量法见末尾「要主 agent 跑的命令」第 3 项的说明。fsync 到来的最坏等待是 max(正在跑的那一段) 而不是一批：这是一把锁的固有代价，下面是岔路 F-1。

### 1.5 停线程、卸载、丢弃、panic、持锁太久、卸载时有活意图

| 事件 | 应该怎么办 | 依据 / 怎么验 |
|---|---|---|
| 后台线程怎么停 | `unmount` 置 `stopping` 并 `Condvar` 通知；线程在两次醒来之间检查它；醒来的一趟里靠 `foreground_fsync()` 答 `Waiting`（上节）让整理在下一批之前返回；之后 `join` | 停的判别子复用核心层已有的让路点，不新增核心层入口 |
| 卸载次序 | 置 `stopping` → `join` → 取锁 → `into_session_after_publishing_the_pending_changes`（`crates/singlefs-core/src/mounted_session.rs:6102`）→ 丢会话 → 丢 GPU 把手 → 把盘交回 | 排空被拒时核心层把攒批的会话原样交回（「pub session_with_pending_changes: MountedSessionWithPendingChanges,」，`crates/singlefs-core/src/mounted_session.rs:5583`），`unmount` 照样把运行库原样还给调用方，不丢盘、不丢待发 |
| 卸载时 GPU 上有没交完的提交 | 不等：那张卡整张泄漏、不析构，字节记进进程里的泄漏账 | 「卸载时不等还在跑的提交，有这种提交的那张卡整张泄漏、不析构。」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`） |
| 卸载时有活整理意图 | 不用做完。意图留在盘上，下次挂载续做；`R` 排他只在内存里，卸载即消失 | 目标态单调：「前台在 `R` 内新分配的落点，其分配代必然 > `c0`」（`.claude/kb/decisions/26-后台整理与放置回收.md:54`） |
| 维护线程 panic | 线程体外套 `catch_unwind`：先把 `failed` 置位并存下 panic 信息（放在会话锁**之外**的单独小锁里），再让会话锁照标准库规矩中毒。前台每个入口先读 `failed`；读到或遇到中毒的锁都回 `RuntimeRefused::Failed`。**不**用 `into_inner()` 接着用内存里的会话 | 会话可能停在半个更新里（分配器与 `current` 不一致），不能继续写；panic 与「在任意位置崩溃」同类，只能丢掉内存、从盘重挂，重挂走的是已被崩溃点重放测过的恢复路径。对比：`singlefs-background-gpu` 里用 `unwrap_or_else(PoisonError::into_inner)` 的只是泄漏账这种计数（「.unwrap_or_else(std::sync::PoisonError::into_inner) = Some(gpus_leaked);」，`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:728`），不是会话 |
| 前台持锁太久 | 维护线程阻塞等锁，不超时放弃；取锁前后各查一次 `stopping` | 前台入口的耗时由前台的负载决定，后台线程没有可丢的东西 |
| 前台在锁外 panic | 无影响（没拿锁）；`fsync_waiters` 的守卫在展开时 `Drop` 减一 | 单测：panic 的前台线程不留「有 fsync 在等」的残值 |
| 丢弃（没调 `unmount` 就 `Drop`） | 只停线程并 `join`，**不**排空、不写盘；运行库标 `#[must_use]` | `Drop` 里不能返回错误，也不该在析构里做写盘；等价于崩溃，丢失窗口按 D16 的 `T_time + P + d + 排队时长`（`.claude/kb/decisions/16-发布语义.md:167`）。是否改成「`Drop` 尽力排空」交用户，见 F-2 |

### Q1 岔路（候选与代价，不替用户定）

- **F-1　持锁上界**。候选甲：接受一把锁，fsync 最坏等待 = 当时持锁的那一段（上表，没量过）。候选乙：核心层给后台入口加「至多一次发布就返回」的成员（`CompactionRunReach` 现有 `AtMostOneBatchAfterAUserChange` 等成员，`crates/singlefs-core/src/mounted_session.rs:2918`），运行库在一趟里多次调、每次之间放锁。乙的代价：每次调用都重走起跑判与「version_as_the_compaction_reads_it」（`crates/singlefs-core/src/mounted_session.rs:4439`）那 O(映射条目数) 的读版本，批间开销按发布次数乘；删除续做与 discard 两段乙也切不开。丙：把前台读写与后台分到两把锁——核心层 `&mut self` 的会话模型不允许，要等里程碑六的并行提交（`.claude/kb/milestone/06-sixth-txn.md`），不在这一轮。**翻面观测**：量到「持锁最长一段」超过用户能接受的 fsync 延迟时由甲翻乙。
- **F-2　`Drop` 要不要尽力排空**。候选甲：只停线程。候选乙：尽力排空，错误只写进 `ReceiverOfTheAlarms`。乙的代价：析构里写盘可能阻塞调用方线程，且失败无处可报；甲的代价：忘调 `unmount` 就丢最多一个 `T_time + P` 的写，与崩溃同。

## Q2　三条水位（主攻）

### 2.1 判据写成什么函数、读哪份记账、代价随不随盘容量涨

三个量的读数都已经是分配器在分配那一刻增量维护的计数，读一次 O(1)，与记账树里写出去的是同一份（记账行就是读它们：「PerDeviceAccountingRow::FragmentationRuns => device_map.free_runs(),」`crates/singlefs-core/src/transaction.rs:15645`、「PerDeviceAccountingRow::EmptyClusterSegments => device_map.empty_segments(),」`crates/singlefs-core/src/transaction.rs:15646`）：

| 量 | 读哪里（`DeviceFreeMap`） | 备注 |
|---|---|---|
| 全空聚簇段数 E(d) | 「pub fn empty_segments(&self) -> u64 {」（`crates/singlefs-core/src/allocator.rs:907`），注释「增量维护，读一次 O(1)。」（`crates/singlefs-core/src/allocator.rs:905`） | 整段数 N_seg(d) 用「pub fn unit_area_slots(&self) -> u64 {」（`crates/singlefs-core/src/allocator.rs:614`）除 64 |
| 空闲槽连续段数 runs(d) | 「pub fn free_runs(&self) -> u64 {」（`crates/singlefs-core/src/allocator.rs:899`） | 碎片度 F 的分子 |
| 已分配槽数 L(d) | 「pub fn allocated_slots(&self) -> u64 {」（`crates/singlefs-core/src/allocator.rs:622`） | 条款里 `F = runs / L` 的 L 没有定义（`.claude/kb/decisions/26-后台整理与放置回收.md:28` 只写 `F = runs / L`）；见 2.4 |
| 不可回收字节 U(d) | 常量 0：「pub const UNRECLAIMABLE_ON_A_DEVICE_OUTSIDE_ZONED: BytesOnOneDevice = BytesOnOneDevice::ZERO;」（`crates/singlefs-core/src/admission.rs:250`），记账行写的也是它（「crate::admission::UNRECLAIMABLE_ON_A_DEVICE_OUTSIDE_ZONED.0」，`crates/singlefs-core/src/transaction.rs:15641`） | 第一版不跑 zoned，恒 0；见 2.4 |

**代价**：判一次是 O(盘数)，与盘容量无关，满足 `.claude/rules/fs-design.md` 第一格「运行时决策路径…代价不许随盘容量增长」（`.claude/rules/fs-design.md:23`；起跑判据虽在后台，但用户改动发成之后的那一查在前台线程上，`crates/singlefs-core/src/mounted_session.rs:2627`，所以按第一格的尺子写）。

**函数形态**（替换 `crates/singlefs-core/src/compaction_plan.rs:1405` 的常量函数；读数结构仿已有先例「pub fn of_the_fullest_device_of_the_allocator(allocator: &PoolAllocator) -> Self {」，`crates/singlefs-core/src/background_compute.rs:684`）：

```rust
pub struct WatermarkReadingOfADevice {
    pub device: DeviceIdentity,
    pub empty_cluster_segments: u64,         // E(d)
    pub cluster_segments_in_the_unit_area: u64, // N_seg(d) = unit_area_slots / 64
    pub free_runs: u64,                      // runs(d)
    pub allocated_slots: u64,                // L(d)
    pub unreclaimable_bytes: u64,            // U(d)，今天恒 0
}
pub enum WatermarkLine<Value> { Off, At(Value) }              // 「关」要能说出来
pub struct CompactionWatermarks {
    pub empty_cluster_segments_below: WatermarkLine<EmptyClusterSegmentsLine>, // E(d) < 线 ⇒ 越线
    pub fragmentation_above: WatermarkLine<FragmentationLine>,                 // F(d) > 线 ⇒ 越线（定点数，不用浮点）
    pub unreclaimable_bytes_at_least: WatermarkLine<NonZeroU64>,               // U(d) ≥ 线 ⇒ 越线
}
pub enum WatermarkQuantity { EmptyClusterSegments, Fragmentation, UnreclaimableBytes }
pub enum WatermarkVerdict {
    Off,                                                          // 三条都关
    ForcedByTheTestOnlySwitch,                                    // CompactionRunForcing::ForcedToStartForTests
    NoLineCrossed,
    Crossed { device: DeviceIdentity, quantity: WatermarkQuantity, reading: u64, line: u64 }, // 可观测（硬要求 4）
}
pub fn compaction_starts_by_the_watermarks(
    forcing: CompactionRunForcing, watermarks: &CompactionWatermarks,
    readings: &[WatermarkReadingOfADevice]) -> WatermarkVerdict;
```

判法：先看 `forcing`（`ForcedToStopForTests` ⇒ `NoLineCrossed`；`ForcedToStartForTests` ⇒ `ForcedByTheTestOnlySwitch`），再按盘次序、每盘按 E、F、U 的次序找**第一个**越线的，返回它。F 的越线判法用整数：`runs × 比例分母 > 线 × L`（`u128`），`L == 0` 或 `runs ≤ 1` 时不越线（空盘与只有一整块空闲的盘不起跑）。线的值存在哪里是 2.3 的事；两处调用（`crates/singlefs-core/src/mounted_session.rs:2627`、`:2937`）改成传会话上的水位与读数。

### 2.2 带设备维时「任一越线」怎么算

**条款**：「三水位任一越线起跑；停机取 `R` 本地谓词「删意图那一刻 `R` 按记录全空」，签发前判空洞够不够，无产出就不签下一条；`R` 在意图期内不可作分配目的地」（`.claude/kb/decisions/26-后台整理与放置回收.md:13`），三个量「物理量、带设备维」（`.claude/kb/decisions/26-后台整理与放置回收.md:28`）。**代码**：选段只读第一块盘：「.expect("池里至少一块盘：两盘同槽，选段与判类按第一块盘的记录做（判决 G1 攻方 K-n）");」（`crates/singlefs-core/src/mounted_session.rs:4432`），选段文档同说「第一版两盘同槽：活槽数按 `unit_area.device` 那块盘的记录数」（`crates/singlefs-core/src/compaction_plan.rs:87`）。

两边并排：条款说任一盘越线就起跑；选段却永远在第一块盘上选。**两盘同容量且同槽时各盘读数相同，「任一」与「第一块」等价**；盘容量不同（单元区槽数按盘容量算，`allocator.rs` 的 `unit_area_slots_of_device`）时，E 与 N_seg 在两盘上不同，会出现「非第一块盘越线、第一块盘不越线」，起跑后在第一块盘上选不出段，空转一趟。候选：

- 甲：按条款字面，每盘求值、取「或」，空转由 Q3 的「没产出的醒来备忘」兜；
- 乙：只用第一块盘的读数，与选段一致，条款里的「任一」留到选段按盘做的时候再兑现；
- 丙：取每个量上各盘最差的一个（E 取最小、F 取最大），一次判定。

三者代价都是 O(盘数)。我倾向甲（贴条款、有兜底），但甲的兜底（备忘）本身还没被攻过，写「零轮」。

### 2.3 系统配置那三个字段与三个量怎么对；哪一边是挂载参数、存不存盘

**事实**（量的）：

- 盘上 24 字节今天只写不读：运行配置是个无字段的单元结构「pub struct SystemRuntimeConfiguration;」（`crates/singlefs-core/src/system_configuration.rs:349`），三条水位由一个返回常量的方法给出「[COMPACTION_WATERMARK_BUILT_IN_DEFAULT; COMPACTION_WATERMARK_COUNT]」（`crates/singlefs-core/src/system_configuration.rs:370`），写系统配置时循环写出「for watermark in self.runtime.compaction_watermarks() {」（`crates/singlefs-core/src/system_configuration.rs:681`）。`grep -n compaction_watermarks crates/singlefs-core/src/system_configuration.rs` 只有定义与这一处使用，没有读回的路径。
- 字节表：「| 可调值 | 整理低水位 / 高水位 / 停止线 | 8 × 3 | 0 = 内置默认 |」（`.claude/kb/layout/01-first-txn.md:153`）；挂载参数表说这三个「存：同 `T_time`」，而 `T_time` 的「挂载时给的值覆盖盘上字段并回写」那条通道不存在（欠账 C422，挂载参数表自陈「挂载入口不存在」）；表下一段自陈「哪个字段对哪个量没有条款。」（`.claude/kb/mount-parameters.md:35`）。
- 条款：「取保守默认、做成挂载可调，水位真正实现、给默认值；数值未定、不在这里，数值设计轮之后由用户定。」（`.claude/kb/decisions/26-后台整理与放置回收.md:32`）。

**「阈值数值不进格式」与盘上字段矛盾吗**：不矛盾，是读法问题。D16 已定项 5 对 `T_dirty` 写得清楚：「系统配置里那个字段是格式，字段里放什么数不是」（`.claude/kb/decisions/16-发布语义.md:165`）；「数值不进格式」说的是数值不是格式常量，字段这个位置可以在格式里。真正对不上的是另外四处：① 三个名字（低水位、高水位、停止线）像一个量的滞回带，而条款是三个量各一条起跑线、停机不靠水位（`.claude/kb/mount-parameters.md:35`）；② `0 = 内置默认` 没有「关」的编码，而 `WatermarkLine::Off` 要有；③ `u64` 装不下碎片度这种比值，要先定标度；④ 「存盘 + 挂载可覆盖并回写」要的通道不存在。

**候选**（格式第一个外部用户之前是软的，`.claude/rules/format-evolution.md`；改字节要同步 checker 与 `invariants/`）：

| 候选 | 做法 | 改的字节 | 代价 |
|---|---|---|---|
| 甲：水位全是挂载参数，不存盘 | 数值只在 `RuntimeParameters.compaction_watermarks`；盘上 24 字节保留为全 0 的预留位 | 否（逐字节不变） | 与 30 秒周期、写带宽、后台计算同属「不存」那一组，一致；代价是重挂必须再给一遍，忘给就用内置默认。要改 kb 的字节表与挂载参数表措辞（三个字段改叫预留） |
| 乙：水位全是挂载参数，删掉那 24 字节 | 同甲，另把字段从字节表去掉 | 是（偏移后移、checker 与 mkfs 字节测试全动） | 格式更干净；动的文件最多，只为去掉 24 个零字节 |
| 丙：存盘，三个字段改名各对一个量 | 字段改叫「空段数线、碎片度线、不可回收字节线」，0 保留为「用内置默认」，另需一个「关」的编码；挂载可覆盖并回写 | 语义变、mkfs 写的默认仍可为 0 | 需要先补挂载选项通道（C422）；盘上值与运行库参数谁赢要另定；一份盘换一台机器行为就变 |

我不替用户选；若要「最小改动且字节不动」是甲。

### 2.4 每个量的候选默认值与取法（拍的 / 量的），越线太早、太晚各怎样

**E 全空聚簇段数**（越线条件 E(d) < 线）。硬边界（量的，读码）：

- **下界**：前台写入类发布在 E(d) < N₀ 时被合取 7 拒。N₀ 取 1（「pub const EMPTY_CLUSTER_SEGMENTS_FLOOR_OF_THE_FIRST_VERSION: u64 = 1;」，`crates/singlefs-core/src/compaction_plan.rs:488`），判在「if empty_cluster_segments_conjunct_is_judged {」（`crates/singlefs-core/src/transaction.rs:12896`），拒绝成员 `EmptyClusterSegmentsBelowTheFloor`；核心层注释已写这是给调用方的「先整理」信号：「调用方分得出「先整理」与「真没空间」」（`crates/singlefs-core/src/mounted_session.rs:1998-2001`）。所以线 ≥ N₀ + 1 = 2，否则前台先被拒、GC 才起跑。
- **上界**：线(N_seg) ≤ N_seg，否则空盘就越线。模型 B 里 `E1c16`（固定 16 段）在 6 段的小盘上空盘就越线，是唯一一个。
- **要赶得上写入**：一个整段 1 MiB（16384 字节 × 64 槽，量的，模型锚点对过）。线至少要 N₀ + 写入速率 r × (周期 T + 一趟整理时长 R)（模型 C；r 与 R 是拍的输入，周期 T = 30 s）：r = 100 MiB/s、R = 10 s 要 4001 段（约 4 GiB），r = 1000 MiB/s、R = 60 s 要 90001 段。固定几十段的线对持续写入毫无余量，线必须随盘容量或写入速率定。

| 候选 | 线 | 取法 | 太早 | 太晚 |
|---|---|---|---|---|
| E1 固定段数 c（4 或 16） | c | 拍 | c 大于小盘整段数时空盘就起跑 | 持续写入下赶不上，前台先吃合取 7 的拒 |
| E2 盘的整段数的 p%（2、5、10），下限 2 | max(2, ⌈p·N_seg⌉) | 拍 | 装得很满但装得很紧的盘（整段要么满要么空）E 本来就小，天天越线、天天白醒（靠 Q3 备忘兜） | p 小时大盘上仍是几千段，赶不上 1 GiB/s 级写入 |
| E3 E2 与固定上限取小 | max(2, min(c, ⌈p·N_seg⌉)) | 拍 | 同 E2 | c 小时大盘上形同 E1 |
| E4 按写入速率现算 | N₀ + ⌈r̂·(T+R̂)⌉，r̂ 与 R̂ 取近期实测 | 量（要先有写入速率与整理时长的采样，目前没有） | 采样噪声 | 采样滞后；而且引入运行时变量，违反「分支变量不许在操作中途改变」（`.claude/rules/fs-design.md` 五条硬要求第 5 条），要挂载时定死 |

模型 B 打出各候选在 6 段、64 GiB、1 TiB、16 TiB 上的线（主 agent 跑模型才有）；我手算 1 TiB（1 048 576 段）：2% 是 20 972，5% 是 52 429，10% 是 104 858。

**F 碎片度**（越线条件 F(d) > 线）。这是三个量里最弱的一环，三处先说清：

- 条款只写 `F = runs / L`，没定义 L；E93 里的 L 是按 key 序遍历对象时的口径，runs 后来换成了「单元区空闲槽的连续段数」，条款自己写了「两个口径量的不是同一个量、不能直接搬」（`.claude/kb/decisions/26-后台整理与放置回收.md:178`），欠账 C326。E93 的数不能用来取线。
- 空闲槽的连续段数在新池上是 1（尾部一整块空闲）：`free_runs` 起步值是「u64::from(unit_area_slots > 0)」那一类（`crates/singlefs-core/src/allocator.rs:473`），用得很少的盘上 L 很小而 runs ≥ 1，`runs / L` 大，会在近乎空盘上越线。必须带绝对下限。
- 整理能不能把 runs 压下来，E93 的结论是「预算最高那条整理臂的 `runs` 轨迹单调上行」（`.claude/kb/decisions/26-后台整理与放置回收.md:38`，旧口径）：按旧口径整理压不下 runs，新口径没量过。F 越线而整理不产出的情形会很常见。

| 候选 | 判法 | 取法 | 太早 | 太晚 |
|---|---|---|---|---|
| F-a 条款字面 | runs·分母 > 线·L，加 `runs ≥ 绝对下限`；线取 1/16、1/8、1/4 | 拍，L 取已分配槽数 | 近乎空盘越线（靠下限挡）；小文件多的盘 runs 天然高，天天越线 | 线大时碎到顶才起跑，F 不再是早期信号 |
| F-b 每个非空段的空洞数 | runs > 线·(N_seg − E)，线取 0.5、1、2；同样加绝对下限 | 拍 | 同上 | 同上 |
| F-c 本轮不用 | `WatermarkLine::Off` | — | — | F 这一支永不起跑，E 一支独自撑着起跑；与「三条水位真正实现」的字面差一条，要用户点头 |

**U 不可回收字节**（越线条件 U(d) ≥ 线）。读数恒 0（上表），线只要 > 0 就永不越线。候选：甲，照条款接上、线默认关，等 zoned 或条带钉住让 U 非零再有意义（D2 条款把钉住的格计入它，`.claude/kb/decisions/02-RAID条带策略.md:181`；代码里没有写非零 U 的地方，上面 `:15641` 与 `:250`）；乙，把第三个量换成「defer 队列字节」（记账第 5 项，`deferred_slots`，已有 O(1) 读数），但它是已释放等窗口过期的空间，整理加速不了它，起跑了也白跑，不推荐；丙，去掉第三个量。乙、丙要改 D26 已定项 1 的正文，用户定。

**默认值候选集合（交用户，我不挑）**：E 取 E2 的 p ∈ {2%, 5%, 10%} 或 E3；F 取 F-c（关）或 F-a/F-b 加下限；U 取甲（关）。「量的」只有两端边界（下界 N₀ + 1，上界 ≤ N_seg）与赶得上的公式（模型 C），具体 p 与 F 线都是拍的。要把 E 的 p 变成量的，缺两个输入：本机持续写入速率（D16 已写明 E58 的 2045 MiB/s 是校验和实验的顺序口径，不许拿来换算，`.claude/kb/decisions/16-发布语义.md:175`）与一趟整理的挂钟 R（欠账 C412，E185 量了一批搬迁的拆分，没量前台抢 I/O 与产品默认路径）；测量命令写在末尾第 3 项。

### 2.5 要不要回滞；空盘不起跑、满盘不进死循环

- **停机不靠水位**（`.claude/kb/mount-parameters.md:35`），所以不需要「停止线」；起跑线附近反复的代价被三样东西封顶：一趟至多 B_gc 次发布（`crates/singlefs-core/src/compaction_plan.rs:1379`）、周期 30 秒才查一次、签下一条意图要全局差值大于 0（`the_completed_intent_yielded`，`crates/singlefs-core/src/compaction_plan.rs:1269`）。线在 E = 线 附近来回，最坏每次醒来签一条意图、产出一个段，不会连环。**回滞换成「备忘」**（Q3）：越线但上一趟没产出，且版本没变，就不再白做。
- **空盘**：E = N_seg ≥ 线（线 ≤ N_seg 是构造条件），F 在 L = 0 或 runs ≤ 1 时不越线，U ≡ 0。应有的会红的用例：新 mkfs 的几种容量的池，判决恒 `NoLineCrossed`；把线造成大于 N_seg 的值，用例必须红。
- **满盘**（所有整段都满）：E = 0 < 线，越线；起跑后选不出段或签发前空洞不够，趟以 `NoEligibleSegment` 或 `HolesOutsideTheRegionFewerThanItsUnitsToMove` 停下，一个写都没发。**不进死循环**取决于下一次醒来是否又花 O(容量)：没有备忘就花（Q3 H1），有备忘就 O(1)。

## Q3　起跑之后会不会空转或不收敛

### 3.1 周期每次醒来今天会做什么、花多少

按 `crates/singlefs-core/src/mounted_session.rs` 的代码顺序（后台入口 `:2842`，起跑判 `:2872`，起跑后的一趟 `:3690`）：

| 醒来的结局 | 花费 | 下次醒来 |
|---|---|---|
| 没起跑：定时兜底关着 / 间隔没到 / 水位没越线 / 预算扣完（「BackgroundCompactionNotStarted::PublishBudgetOfTheLastUserChangeSpent,」`crates/singlefs-core/src/mounted_session.rs:2956`） | O(盘数)，不读盘 | 原样再来一遍，便宜 |
| 起跑了但批大小没定 | 在一切读写之前返回（`:3753`），连先走空的删除续做都没走 | 每次都是这一句；K 不定就永远这样 |
| 起跑了，停在 `NoEligibleSegment` / 签发前空洞不够 / `NoYieldFromTheLastIntent` / `NothingMovableButTheRegionIsNotEmpty`，一个批都没发 | **先读版本再判**：读版本「let read_of_the_version = version_as_the_compaction_reads_it(&version);」（`:4439`，O(映射条目数)）、「empty_cluster_segments_by_the_records(&version.allocation_records, unit_area);」（`:4477`，O(分配记录数)）、选段时对每个候选段读盘扫描（`:4578`） | **原样再做一遍**：停机原因只记进观测 `last_stop`，没有备忘；定时兜底那一趟被节奏截住时也不放回「该跑了」（`:3012` 只对用户改动之后那一趟放回） |
| 停在前台 fsync / 写带宽 | 一趟之前问一次 | 用户改动之后那一趟「该跑了」放回，定时兜底那一趟下个周期再判 |
| 整理因 GPU 出错停着 | 每次叫起都带一份报警（核心层契约，`#[must_use]`） | 30 秒一份，一天 2880 份；运行库的接收方要自己合并展示，不能当每条都要弹窗（Q4） |

所以「不需要清理就不一直跑」在 E 一支上成立（线下不越线 ⇒ 一个读都不做）；F 一支在加了绝对下限与备忘之后同样成立，U 一支读数恒 0、线非零就恒不越线，平凡成立；**在越线却产不出段时不成立**：每个周期白做一遍 O(容量) 的活，而且是在会话锁里（推的，没量过 `version_as_the_compaction_reads_it` 在大池上要多久；条目数随池大小线性涨是读码得的）。

### 3.2 「没产出的醒来」备忘（零轮形态）

**加在核心层**：`CompactionOfThisMount`（`crates/singlefs-core/src/compaction_plan.rs:833`）加一个字段 `futile_at: Option<CheckpointTxg>`。

- 写：一趟结束时没发任何版本（`versions_published` 空）、且停机原因属于 {`NoEligibleSegment`, `HolesOutsideTheRegionFewerThanItsUnitsToMove`, `NoYieldFromTheLastIntent`} 时，记下现行根的 txg；
- 读：`whether_the_background_compaction_starts`（`crates/singlefs-core/src/mounted_session.rs:2872`）在查水位**之前**比 txg，相等就返回新成员 `BackgroundCompactionNotStarted::FutileSinceTheLastPublish`，不读版本；
- 清：任何一次发布 txg 就变，备忘自然失效。输入（分配记录、映射、快照钉住的槽）都随版本，内存里的几样（常驻隔离集、被区域扫描拒过的段表、开着的聚簇段）也只在发布里变。**不进备忘集**的：`IssueHeldBackByRecordsUncertainThisRound`、`OnlyRecordsUncertainThisRoundLeftToMove`（GPU 两份核不一致，下一轮会变，条款要求留到下一轮重算，`.claude/kb/decisions/26-后台整理与放置回收.md:252`）与所有被节奏、空间、出错截住的停机。
- 这是我提的收严，**没被攻过（零轮）**。判别力自证：备忘打开时，同一 txg 上连续两次醒来，第二次的读盘次数为 0 且结局与「不开备忘全量重做」逐项相同（差分对拍）；关掉备忘，第二次读盘次数 > 0，必须红。

### 3.3 预算与周期共用时的收敛：模型 A、可达死局 H3

预算原文：「一次用户改动给出一份 `B_gc`…扣完是 0，等下一次用户改动（会话发成一次用户改动就把已经过去的发布数归零）才再有」（`crates/singlefs-core/src/compaction_plan.rs:1390-1391`），公式 B = max(3S − 4, 1)，S 是每区根槽数：格式区间 4 到 16 ⇒ B ∈ [8, 44]；mkfs 做出的池 S 在 5 到 8 ⇒ B ∈ {11, 14, 17, 20}（模型 A，主 agent 跑过才算量的）。预算存在的理由：「O11：GC 发布不改变用户可见状态，连发会把回退候选集挤成 1 个状」（`research/prompts/m4-gc-r1-main-verification.md:39`）。

**一条意图在一份预算里做不做得完**：签发一次，之后每批一次，最后一批同一次发布删意图。一个整段至多 64 个单元，所以一条意图要 1 + ⌈64/K⌉ 次发布。模型 A 手算：S = 8（mkfs 默认的上限）要 K ≥ 4，S = 5（mkfs 能做出的最小）要 K ≥ 7，S = 4（格式下界）要 K ≥ 10，S = 16 要 K ≥ 2；K = 1 时任何 S 都做不完一条 64 单元的意图。所以 mkfs 做得出的池要 K ≥ 7，格式合法的所有 S 要 K ≥ 10。**这给 K 一个下界**，而 K 现在没有产品值（先读开头那一节「批大小 K 没有产品值」那一行）。K 取不到这个下界，一条意图要跨好几份预算，而预算只有用户改动才补。

**H3　预算耗尽 + 前台被合取 7 挡住 = 死局（可达，推的，没量过）**：

1. 前置：池里全空段只剩 N₀ − 1 = 0 个（前台写入类发布被合取 7 拒，`crates/singlefs-core/src/transaction.rs:12896`），上一次**成功**的用户改动之后整理已经把那一份 B_gc 花光（例如 K 偏小，一条意图没做完）。
2. 前台：写入类改动一律被拒，「let mut published = published?;」（`crates/singlefs-core/src/mounted_session.rs:2596`）在归零那一句（`:2621`）之前就返回 ⇒ 预算不补。删除类改动（unlink、删快照）免合取 7（`crates/singlefs-core/src/transaction.rs:12762`），成功就会归零——这是用户能自救的出路，但前台要自己想到去删东西。
3. 后台：每次醒来都是 `PublishBudgetOfTheLastUserChangeSpent`，整理不再推；意图留在盘上，`R` 排他还在。
4. 结局：只有删东西或重挂能出来。相邻的一条：`last_completed_intent` 记着「上一条完成而无产出」时，「if !the_completed_intent_yielded(empty_now, completed.empty_cluster_segments_at_issue) {」（`crates/singlefs-core/src/mounted_session.rs:4478`）挡住下一条签发，要靠用户改动清（`:2626`），同一类死局；无产出的一种来源是签发开新段抵消了腾出的那一段（「后续只腾空一段仅抵消开段，不算净产出」，`.claude/kb/decisions/26-后台整理与放置回收.md:78`）。

现有测试只钉了闲置池在定时兜底下合计不超过那一份预算（用例 `an_idle_pool_under_the_timer_fallback_publishes_no_more_than_the_budget_of_its_only_user_change_until_the_next_change`，`crates/singlefs-verification/tests/compaction_off_the_calling_thread_and_admission.rs:419`），**没有**钉「预算花光而前台被挡」这一格。要造它：小池（384 槽那种）把全空段耗到 0，K 设成 1，调后台入口直到预算扣完，再 `accept` 一次写入类改动看是否被拒、再调后台入口看是否仍是 `PublishBudgetOfTheLastUserChangeSpent`。

**岔路 F-3（交用户，候选与代价，不替用户定）**：

- 甲：接受。与用户 2026-11-08 的第 4 条「预算照旧共用」字面一致；代价是死局靠用户删东西或重挂，运行库要把「整理停着而前台一直被合取 7 拒」报给调用方（观测里给 `starved: bool`）。
- 乙：合取 7 拒绝的写入类改动也让预算归零（核心层在拒绝的那一支补一句）。代价：预算存在是为了不挤掉回退候选，这一支里用户可见状态没变，GC 却又得一份预算，候选集被挤的风险原样回来，但这时前台本来也写不进，没有新状态要保护。
- 丙：E(d) < N₀ 时后台入口改用显式入口的预算规则（`CompactionRunBudget::SinceTheLastUserChange` 每趟至少 1 次发布，`crates/singlefs-core/src/mounted_session.rs:492`），其余时候照共用。代价：多一支分支，要按硬要求 2、4 配强制进入的开关与观测；每 30 秒至少一次发布，一条 17 次发布的意图要 8 分钟。
- 丁：每隔 N 个周期补一份新预算。代价：破「闲置时合计不过一份」，与用户的第 4 条相反，需用户明说放宽。

**翻面观测**：量到「K 取定后，一条意图在默认 S 下一份预算内做得完」就让 H3 只剩 K 取小或意图堆积两种来路，甲可以接受；量不到（K 偏小）则非乙或丙不可。

### 3.4 其余可达历史

| 编号 | 历史 | 结局 | 罩没罩 |
|---|---|---|---|
| H1 | 满盘（整段全满），E = 0 越线 | 每醒来一趟 O(容量) 白做（3.1），无备忘 | 没有（推的） |
| H2 | 快照钉住候选段 / 区域扫描整次拒过的段太多 | 同 H1，趟以 `NoEligibleSegment` 停（`crates/singlefs-core/src/mounted_session.rs:4601`） | 选段资格有用例（`compaction_plan_chooses_regions_and_batches_by_the_clauses.rs`），醒来备忘无 |
| H4 | 持续有 fsync 在等：每次一批前问一次都答 `Waiting` | 整理一批都发不出，E 往下掉到合取 7，写被拒、fsync 的等者消失、GC 才得空：自纠，但中间前台吃一轮拒 | 单次让路有用例（`compaction_off_the_calling_thread_and_admission.rs:1190`），持续的没有 |
| H5 | 水位在线附近反复 | 每次醒来至多签一条意图；封顶见 2.5；不用滞回 | 随 2.5 的用例 |
| H6 | GPU 出错停着，占用不超过补跑阈值 | 每次醒来 `NotStarted` 带一份报警，不写盘（`crates/singlefs-core/src/mounted_session.rs:2908`） | 有（`compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs`） |
| H7 | 趟开头发现版本没有文件 | 后台入口 `Err(NoVersionOfTrees)`（`crates/singlefs-core/src/mounted_session.rs:2896`）；运行库每个周期收到一个 `Err`，要当「还没到时候」不当故障 | 无 |

## Q4　GPU 开卡的时序与判决留下的两条

### 4.1 挂载那一刻在运行库里的次序，以及每一步失败落什么结局

次序按动作写（读参数 → 开卡 → 挂载 → 接会话 → 起线程）。**开卡放在挂载之前**，理由：条款与挂载入口都要求设置错了「就在读任何一块盘之前拒绝挂载」、盘上逐字节不变（「超过 100% 就在读任何一块盘之前拒绝挂载」，`crates/singlefs-core/src/mount.rs:9583`），而「选 `gpu` 而 GPU 不在」是拒绝挂载不是降级（`.claude/kb/mount-parameters.md` 表中 `background_compute` 一行）；先挂载后开卡，拒绝时盘已被挂载写过。

| 动作 | 做什么 | 失败的结局 | 盘与资源 |
|---|---|---|---|
| 读参数并校验 | 周期、读盘周期 P 大于 0，水位线合法（线 ≤ N_seg 要等读到盘才知道，放到挂载之后判），写带宽取值 | `RuntimeMountRefused::Parameters`（不读盘） | 盘原样交回 |
| 开卡（仅 `background_compute = gpu`） | 在**工作线程**里调 `OpensTheBackgroundGpu::open`，主线程 `recv_timeout(gpu_open_deadline)`；选 `cpu` 时不碰特质、不建 Vulkan 实例 | 设置被拒 / 点名的卡不在 / 参与的卡交不成清单 / 枚举超过等待上限 / 一张参与的卡用不上，共五种（「[`BackgroundGpuOpenRefused`] 的五种」，`crates/singlefs-background-gpu/src/background_gpu_compute.rs:198`）⇒ `RuntimeMountRefused::Gpu(原因)`；超过 `gpu_open_deadline` ⇒ `RuntimeMountRefused::GpuOpenTimedOut`（4.2） | 盘还没读；已开成的卡随把手丢掉 |
| 挂载 | `mount_writable_with_background_compute_settings_and_the_gc_cpu_compensation_threshold`，盘 `&mut` 借给它；核心层再按这份实现答的卡清单校验一次（「核心层挂载时照这份实现答的卡清单（此刻本机在的独显，连同没开成的那几张）再校验一次」，`crates/singlefs-background-gpu/src/background_gpu_compute.rs:192`）；挂载里已含「续做盘上那条未完成的整理意图」，在线程起来之前跑完，所以**续做与后台线程起跑没有交错** | `RuntimeMountRefused::Mount(MountError)` 或 `::Parameters`（核心层校验拒）；GPU 把手丢掉（没有在飞提交，不泄漏） | 盘交回；挂载拒绝时盘上状态按核心层今天的口径 |
| 接会话 | `MountedSession::of_the_mount`（`crates/singlefs-core/src/mounted_session.rs:2443`）→ `MountedSessionWithPendingChanges::of_the_session`（`:5591`） | 无 | — |
| 起维护线程 | `std::thread::Builder::spawn`；水位线对 N_seg 的合法性此刻判 | 起不来 ⇒ 按卸载收尾（排空待发、丢会话、泄漏账记进去）后 `RuntimeMountRefused::ThreadNotStarted`，盘交回 | 此时盘已被挂载写过，所以要走卸载收尾，不能只丢 |

### 4.2 打中 1：枚举放线程加期限；超时之后那个线程怎么办、算不算把实例弄坏

**事实**（量的）：等 wgpu 的 future 的上限不含造 future 那一步：「期限从进这个函数起算：实参求值时做的活（造 future 那一步同步做完的枚举）不在 `wait_limit` 之内。」（`crates/singlefs-background-gpu/src/background_gpu.rs:2615`）；判决任务 10414 的读数「FUTURE_BUILT_BEFORE_THE_DEADLINE outcome=Ok(7) elapsed_millis=300」（`research/prompts/m4-gpugc-one-vulkan-instance-code-r1-main-verification.md:16`）。真挂住的枚举，等待上限管不到，调用 `open` 的线程跟着挂住。

**形态**：整个 `open` 放进工作线程，调用线程只做有期限的等。三种结局：

- 按时返回 ⇒ 照 4.1。
- 超时 ⇒ 挂载拒绝（`GpuOpenTimedOut`），调用线程不再等。工作线程**不杀**（没有安全的杀法）；它持有实例的一份克隆（`VulkanInstanceOfThisProcess` 是「#[derive(Clone, Debug)]」，`crates/singlefs-background-gpu/src/background_gpu.rs:755`，克隆指的是同一个 wgpu 实例）。
- 工作线程之后醒来 ⇒ 结果送进一个没人收的通道，丢弃；若是开成的把手，丢弃时泄漏账照常记。

**超时之后再来一次怎么办**：粘合层的开卡器在内部记「上一次开卡还没回来」，这期间任何新的开卡直接回 `EarlierOpenStillHung`，**不**再起第二条枚举线程。理由是同一个驱动加载器里再起一条枚举会不会也挂、会不会和第一条争状态，没量过（推的）；不挡的话，周期性重挂的监督者每次超时都多留一条挂住的线程。

**算不算把实例弄坏**：不知道。能说的只有：实例在超时之后不再被新开卡用到（上面的闸），也没被丢掉；挂住的是驱动那一侧，实例对象本身的状态没读到。复核要真的挂住的驱动，**复核不了**（原因：本机与第二台的驱动都能正常枚举，造不出真挂住；用假的开卡器可以验运行库这一侧的行为，验不了驱动那一侧）。

`gpu_open_deadline` 的默认是拍的：一张卡的开卡上限是 5 分钟（「pub const WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT: Duration = Duration::from_secs(5 * 60);」，`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:271`），几张卡逐张开最坏是它的倍数。候选：甲取一张卡的 5 分钟加枚举余量，代价是多卡逐张慢开会被截；乙取卡数 × 5 分钟，代价是挂住时挂载等很久；丙把期限做成挂载参数。用户定。

### 4.3 打中 6：一进程一份实例与「修好驱动重挂」怎么同时成立

**事实**（量的）：实例枚举到的驱动清单定在建它的那一刻（「这个实例枚举到的驱动清单定在建它的那一刻：之后修好、装上或藏起驱动，同一个实例枚举到的不变，新建的实例才看得到」，`crates/singlefs-background-gpu/src/background_gpu.rs:753`）；任务 10428 的读数 `same_instance_after_the_drivers_came_back=0 new_instance_after_they_came_back=3`（`research/prompts/m4-gpugc-one-vulkan-instance-code-r1-main-verification.md:17`，只换环境变量，没真拔卡）；同一进程里新建到大约第 12 个实例起，第二台上枚举只剩软件 Vulkan，而且「前面的实例都已丢掉」仍如此（`crates/singlefs-background-gpu/src/background_gpu.rs:751`）⇒ **丢掉旧实例不归还额度，额度按进程生命期累计**。判决打中 7：这个 12 只对那一套驱动成立。

**让两件事同时成立的做法**：实例由粘合层的开卡器持有（`Arc`，进程一份，惰性建，第一次要开卡才建），带一个只增不减的「本进程建过几个实例」计数。重建的策略三选一（用户定，我给代价）：

| 候选 | 做法 | 代价 |
|---|---|---|
| 甲：不重建 | 修好驱动要重启进程 | 长驻守护进程要多一步重启；零额度风险 |
| 乙：只在用户显式要求时重建，计数到上限就拒 | `reset_the_gpu_instance()`；上限取比已知额度低得多的数（候选 4） | 多一个入口；上限 4 是拍的，12 只量过一套驱动 |
| 丙：开卡因「GPU 不在」「枚举失败」被拒时自动重建一次再试，同一个计数封顶 | 监督者反复重挂 + 驱动真没了时，每次尝试烧一个额度，上限到了就不再建、直接拒 | 自动，但最坏几次尝试就把额度用光，之后即使驱动修好也只能重启进程 |

两条共同的约束：重建只影响**之后的**开卡，已开着的把手持有自己的设备，不动；开卡器内部串行化（同一时刻只有一条枚举），避免两次挂载同时建实例。第二台约 12 个的界不会被「一次挂载一个实例」撞上，因为实例按进程建一份、重挂复用；会撞上的只有乙、丙的重建次数，所以它们必须带上限。

### 4.4 `GpuMemoryLeakedInThisProcess` 由谁持有

粘合层的开卡器，和实例放在同一个对象里，进程一份，每次开卡都交同一份：类型文档就是这么要求的（「调用方在进程里留一份、每次开卡都交同一份」，`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:732`），它本身是 `Arc<Mutex<…>>` 的克隆共享（`:737`）。运行库不碰它；运行库只在卸载时丢把手，泄漏账由把手丢弃时自己记。

### 4.5 `gpu` 用不上时的报警、停下、CPU 补跑，报警送到哪

后台入口每次返回的 `BackgroundCompactionAfterThePendingChanges`（`crates/singlefs-core/src/mounted_session.rs:5560`）里有排空待发的那一次发布、有 `call`（带报警）。条款要求：「通道是返回值里必须处理（`#[must_use]`）的报警，不是日志，调用方不处理编译就警告」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`）。运行库就是这个调用方：

- 每次返回的 `call` 与排空那一次发布，一律交 `ReceiverOfTheAlarms`（1.3 里的特质，建议改叫 `ReceiverOfTheRuntimeEvents`，多一个 `publish_reported`）；不丢，丢了编译警告。
- 排空那一次发布是**前台调用方不知道的发布点**：前台 `accept` 返回「收下」之后，后台线程在下一次醒来把它发掉。D16 的 RefFS 翻译按会话报告的发布点判合法态，所以这些发布点必须送出去（运行库的事件流就是交错的记录，验证包按它对拍）。
- 运行库自带的默认接收方：保留最近一份报警、累计次数、`wakes_while_stopped` 与 `batches_compensated_on_the_cpu`，经 `observations()` 给调用方；停着期间 30 秒一份，默认接收方只合并计数，不逐条弹。**报警真正怎么告诉文件系统的用户**不在运行库：首个真正的入口是 FUSE 守护进程（「FUSE 守护进程把内核发来的请求翻成已有的事务」，`.claude/kb/milestone/12-twelfth-txn.md:9`），它实现接收方；这一轮只给到特质与默认实现。

## Q5　前台与后台交错之后崩溃语义变没变

### 5.1 一把锁串出来的次序集合，有多少已被罩到

一把锁下，每个核心入口是原子的一次调用，运行库能产生的历史就是「入口调用序列」，字母表是：前台的 `accept`、`publish_pending_now`、`publish_user_change`、`switch_background_compute`；维护线程的 `publish_when_due`、`run_compaction_in_the_background`。崩溃语义（磁盘上每个写前缀之后是什么状态）是每次发布自己的事，与入口怎么排没有关系；变的只是「哪些入口序列会出现」。逐格看已有测试罩了哪些（都是我 `grep` 读文件得的）：

| 层 | 现有的 | 字母表里有没有后台入口 |
|---|---|---|
| 攒批会话的随机历史 | 「pub enum BatchingSessionCall {」（`crates/singlefs-verification/src/history.rs:10055`）只有五个成员：`accept` 覆盖写、`accept` 顺序写、`accept` 建 inode、`PublishWhenDue`、`PublishPendingNow` | **没有** |
| 会话级随机历史里的整理 | `HistoryOperation::CompactWhileMounted`（`crates/singlefs-verification/src/history.rs:579`）经**显式入口**推，`CompactionRunEnding` 有「推到停」与「至多推 N 批后崩溃再挂载」两种（`:974-984`） | 显式入口，不是后台入口，也没有 `accept` 的待发 |
| 崩溃枚举的整理流 | 调的是 `session.run_compaction(devices)`（`crates/singlefs-verification/tests/common_compaction_relocation_on_the_background_compute/mod.rs:248`） | 显式入口 |
| 后台入口 | 确定性用例：用户改动之后的一批（`crates/singlefs-verification/tests/compaction_off_the_calling_thread_and_admission.rs:419` 等）、「后台入口先排空待发再跑」一条（`crates/singlefs-verification/tests/batching_across_requests_and_barriers.rs:382`）、`accept` 与 `publish_when_due` 不在调用线程上跑整理（`:469`） | 有，但每条是一个固定序列，不带崩溃枚举 |

**背景材料说「今天的崩溃枚举与模型对拍都是单线程调后台入口」，对不上**：崩溃枚举与模型对拍走的是显式入口，后台入口只在确定性用例里被调（`grep -rln run_compaction_in_the_background crates/singlefs-verification` 命中 `history.rs` 与六个测试文件，没有一个崩溃枚举流，量的）。这不影响「崩溃语义没变」这个结论，但影响「已有测试罩到什么」。

**新的交错**（相对已有测试）：

- 排空后接整理：后台入口里先排空待发、再整理（排空那一句在 `crates/singlefs-core/src/mounted_session.rs:6053`）：一次用户改动的发布后面立刻接整理的发布，两者之间的崩溃点；显式入口没有这个前置发布。
- 读数起算点：整理发布也会把「上次发布的读数」往前推（「self.note_a_publish(now);」，`crates/singlefs-core/src/mounted_session.rs:6087`），影响之后 `T_time` 的起算点；待发已被排空所以不会晚发，方向安全，但是新的状态依赖。
- 让路后再接着推：整理被节奏截住（fsync、带宽、预算）停在意图中途，随后前台发布若干次、再醒来接着推：显式入口的随机历史有「推 N 批后崩溃」，没有「推 N 批后让路、前台再发、再接着推」。
- 线程层面：锁带来的**线程层面**的新东西没有崩溃语义，只有活性问题（死锁、饿死、`stopping` 不被响应），这是线程测试的事，不是崩溃点重放的事。

### 5.2 `publish_when_due` 的周期 P 与整理周期要不要同一个线程

**一个维护线程、两个到期时刻**。理由：两件事都要同一把会话锁，分两条线程不会让任何一件更早做成，只多一条线程要停、要 `join`、要处理 panic；单线程里 P 的到期优先于整理的到期（先查发布，再决定是否整理）。D16 的丢失窗口公式里 P 与排队时长归调用方：「轮询周期 P 与排队时长在里程碑四归调用方负责，条款只点名它们」（`.claude/kb/decisions/16-发布语义.md:167`），运行库就是这个调用方，所以**排队时长里要算上维护线程里一次整理调用中不可打断的最长一段**（3.1 的表，没量过）：整理调用的最长一段直接加进丢失窗口的上界。

P 的默认值是拍的，候选：250 ms（相对 `T_time` = 5 s 加 5%）、1 s（加 20%）、100 ms；轮询的代价是每次一次取锁加 O(1) 判（待发空时「一个盘都不读」，`crates/singlefs-core/src/mounted_session.rs:5937`）。另一种形态：线程用 `reading_of_the_last_publish()`（`:5643`）与阈值（`:5625`）算「待发非空时的下一个到期」，睡在条件变量上，`accept` 收下第一份待发时唤醒它，P 退化成调度抖动。代价是前台多一次通知、线程多一份「到期时刻」的状态；判别力不比轮询高，我不推荐在这一轮做。

**核心层的定时兜底间隔与运行库的醒来周期是两道闸，要对齐**。核心层自己还会判「离上一次查够不够一个间隔」：「Some(last_check) => now.0.saturating_sub(last_check.0) >= interval,」（`crates/singlefs-core/src/compaction_plan.rs:1055`）。运行库若按定速调度（每 30 秒整点醒来）并把兜底间隔也设成 30 秒，抖动让某次醒来比上一次早到几毫秒，核心层就答「间隔没到」，实际周期隔一次才放行。模型 D 手算：醒来时刻 `k × 30 s` 加奇数次 2 ms 抖动，间隔等于周期时放行的是第 0、1、3、5、7、9 次，间隔取周期的一半时十次全放行；定延迟调度（睡 30 s 再醒）没有这个问题。**做法**：运行库把传给核心层的兜底间隔取周期的一半，周期本身由运行库的睡眠定，核心层那一道闸只剩兜底作用。

### 5.3 挂载续做意图与后台线程起跑的先后

续做在挂载里跑完（`MountOutput::compaction_continuation`，`crates/singlefs-core/src/mount.rs:1135`），线程在挂载返回之后才起（4.1），所以**没有交错**，不用加锁序。注意续做的结果依赖批大小：产品值没定，续做在签任何东西之前停下（「产品路径的入口恒给 `CompactionSwitches::PRODUCT`，批大小条款没写，续做在签任何东西、交任何一段给 GPU 之前停下」，`crates/singlefs-core/src/mount.rs:9627`）；K 定了之后，崩溃留下的活意图才能在重挂时被续做完。

### 5.4 要不要加新的崩溃点重放流，多罩什么

按 `.claude/agents/implementation-writer.md` 做什么第 7 步（新流要写明比已有流多罩了哪些崩溃状态；基线镜像、写表、段序列逐项相同的不新开枚举，只加一条小用例钉「相同」）：

- **不新开崩溃枚举流**（推的）：一次后台醒来写出的是「排空待发的那次发布」后面接「整理的若干次发布」，每次发布的写表与段序列和已有流里的同类发布一样；崩溃状态是每个写前缀。**加一条小用例钉住「相同」**：同一个起点池上，后台入口的一次醒来录下的写序列，与「`publish_pending_now` 之后接显式入口 `run_compaction`」录下的逐字节相同（放在 `compaction_off_the_calling_thread_and_admission.rs` 一族）。这条用例若红了，才说明后台入口写出了显式入口没有的东西，那时才需要新流。
- **加到随机历史的字母表里**（模型对拍）：`BatchingSessionCall` 加一个成员 `RunCompactionInTheBackground { clock_advance_in_milliseconds, a_foreground_fsync_is_waiting }`，让 5.1 的「排空后接整理」「读数起算点」「让路后再接着推」三种交错在按会话报告的发布点与模型、RefFS 对拍里出现。
- **线程层面另写**（不是崩溃点重放）：不起线程的 `run_one_maintenance_pass()` 加假钟、假的 `CallerOfTheBackgroundCompaction`，覆盖全部分支；起线程的只留五条与时间无关的：停线程 `join` 得上、panic 后前台回 `Failed`、`fsync_waiters` 的守卫在 panic 展开时归位、开卡超时不阻塞挂载、卸载时有待发会被排空。靠 `Barrier` 与通道定次序，不靠 `sleep`。

## Q6　「接入并运行」怎么验才算数

### 6.1 验收清单（每条落到文件与行；新文件写「新」）

| 验收 | 落点 | 钟与负载怎么换成假的 | 必红世界（弄坏什么，哪条该红） |
|---|---|---|---|
| 挂载之后不调任何后台入口，水位越线的池子上 GC 自己跑、腾出空段、卸载干净、冷启动检查器绿 | 新 `crates/singlefs-verification/tests/mount_runtime_background_compaction_runs_by_itself_on_a_pool_over_the_watermark.rs`；池的搭法借 `crates/singlefs-verification/tests/compaction_off_the_calling_thread_and_admission.rs:365` 的「idle_session_after_a_sequential_write_in_an_earlier_mount」；运行库新包 `crates/singlefs-mount-runtime/` | `ClockOfTheRuntime` 的假实现：测试自己推进读数、直接调 `run_one_maintenance_pass()`；`ReceiverOfTheRuntimeEvents` 记下每次醒来的结局；起线程的一条另列 | 水位线置 `Off`：E 不涨；批大小仍是产品的「没定」：结局 `BatchSizeNotDecidedByTheClauses`；备忘关掉：同 txg 第二次醒来读盘次数 > 0 |
| 水位判据的全部分支 | 新 `compaction_starts_by_the_watermarks` 的纯函数用例，放在 `crates/singlefs-verification/tests/` 新文件；读数造成结构直接传 | 不需要钟；读数手造 | 比较方向写反（`≤` 与 `<`）；少看一块盘；F 的 `L == 0` 判掉；线大于 N_seg 的空盘越线；U 线非零而读数 0 却越线 |
| 起跑后「前台有 fsync 在等」不发下一批，且是真线程里答出来的 | 新用例：`fsync_waiters` 由测试拨，维护线程里的整理一批都发不出；已有的单线程用例钉过让路本身（`crates/singlefs-verification/tests/compaction_off_the_calling_thread_and_admission.rs:1190`） | 登记原子计数，不 `sleep` | `foreground_fsync()` 恒答 `NoneWaiting`：红 |
| 停线程、卸载干净 | 新用例：整理一趟进行中置 `stopping`，`join` 在有限步内回来；`unmount` 之后盘上冷启动检查器绿、待发已发 | `Barrier` 定次序 | `stopping` 不进 `foreground_fsync()` 的答案：`join` 超出看门狗，红 |
| panic 之后前台拿 `Failed`，不接着用中毒的会话 | 新线程用例 | 假的整理入口里 `panic!`（只供测试的一支） | 换成 `into_inner()` 接着用：红 |
| 开卡超时不挡住挂载 | 新用例，假开卡器卡在 `Barrier` 上 | 假开卡器 | 去掉 `recv_timeout`：红（用看门狗兜住） |
| 实例计数上限 | 粘合层的单测 | 假的「建实例」闭包只数次数 | 上限不判：超过上限仍建，红 |
| 核心层：备忘、预算 F-3 选定的那一支、水位函数 | `crates/singlefs-core/src/compaction_plan.rs` 与 `mounted_session.rs` 的改动处；每个新分支配只供测试的开关，见硬要求 2、4（`.claude/rules/fs-design.md:122`、`:127`） | 同上 | 各一条变异行写进 `crates/mutations.tsv` |
| CPU 模式与真卡 GPU 模式写出的块序列逐字节相同 | **已有**：`crates/singlefs-background-gpu/tests/compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu.rs` 文件头写明「两趟发的每一版、录制流、盘上字节、停在哪、搬了几批、整理观测与 discard 逐项相同」；七·4 的要求「两种模式写出的块序列逐字节相同」（`.claude/kb/milestone/04-fourth-txn.md:152`） | 池在内存盘上；卡由服务交，没交就每条打 `NOT_RUN_NO_GPU_ADAPTER` 成功返回（文件头同段） | 必红世界在七·4 那一行：「GPU 计划改掉一个源落点，块序列比对红」 |
| 同一条，但**经运行库**（假钟逐醒来一趟） | 新，放粘合层的 `tests/`（特性 `tests-on-a-real-gpu`，仿上一条） | 假钟、`run_one_maintenance_pass()`；两个池一个 `cpu` 一个 `gpu` | 在运行库的醒来循环里让 GPU 模式少走一批：块序列比对红 |
| 两台都跑得起 | 全部新用例用内存盘或稀疏盘，不写主机名；真卡的用例只开服务交来的卡（`crates/singlefs-background-gpu/tests/real_gpu_tests_open_only_the_gpus_the_task_server_handed_over.rs`），三张卡那一格在第二台上跑（七·3 验收的末句，`.claude/kb/milestone/04-fourth-txn.md:151`） | — | — |

### 6.2 七·4、七·5 现有的测试与 E202 要不要改接运行库

- **七·4 的崩溃枚举与对拍不改接**。验证包不依赖产品 GPU 包（`crates/singlefs-verification/Cargo.toml` 的 `[dependencies]` 里没有它，量的），走的是测试实现；崩溃枚举要的是确定性的单线程序列，接上真线程反而不可重放。
- **七·4 里「经运行库」的那一格新加**（上表倒数第二行），原有那条不动。
- **E202（七·5 的对照组）今天不改接**。登记状态是「跑前登记已写、装置未跑」（`.claude/kb/experiments/202-GPU与CPU两种模式GC对照组.md:1`）：口径在跑前定死，装置现在改接运行库等于改口径，要连登记一起改；而它要比的是 GC 占的主机 CPU 时间，直接驱动核心层的后台入口已经能比。等运行库落地之后，可以在装置里加一臂「经运行库」量线程与锁多出来的开销，作为另一条登记，不替换现有臂。

### 6.3 这一轮「我没做到」的验收

- 持锁时长、水位线的合适值、一趟整理的挂钟：都没量过，量法在下一节第 3 项。
- 真挂住的驱动枚举（4.2）复核不了：造不出。
- 备忘（3.2）与 F-3 乙丙丁是我提的收严，没被攻过。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Q1 包与依赖 | 与背景材料「新包依赖核心层与产品 GPU 包」不同 | 运行库只依赖核心层、不带特性；GPU 通过特质由粘合层递进，验证包才能依赖运行库 |
| Q1 持有者 | 一致 | 攒批的会话加盘表同在一把锁里；攒批的会话缺挂着切换的转发，核心层要加 |
| Q1 一把锁 | 部分一致 | 正确性够；「前台有 fsync 在等」谓词必须放锁外的原子计数；持锁最长一段没量过，岔路 F-1 |
| Q1 停/panic/丢弃 | 规则没说 | 给出策略；panic 等价崩溃只能重挂，`Drop` 只停线程（F-2） |
| Q2 判据函数 | 冲突（条款/代码） | 起跑判据是常量函数不收读数；三个量的读数都是 O(1) 增量计数，U 恒 0，F 的 L 没定义 |
| Q2 字段 | 冲突（条款/条款） | 三个字段名对不上三个量，也没有「关」的编码；「数值不进格式」与字段不矛盾；三个候选，甲字节不动 |
| Q2 默认值 | 规则没说 | 只有两端边界与「赶得上」公式是推得出的；p 与 F 线都是拍的 |
| Q3 空转 | 冲突（背景材料/代码） | 「不需要清理就不跑」在 E 一支成立；越线而产不出段时每醒来一趟 O(容量) 白做，要备忘 |
| Q3 收敛 | 冲突（题面/代码） | 批大小没定、闲置预算只有一份、合取 7 挡住前台时预算不补：三处不先定，GC 自己跑不起来 |
| Q4 开卡次序 | 一致 | 读参数 → 开卡（线程加期限）→ 挂载 → 接会话 → 起线程；每步失败结局有表 |
| Q4 打中 1、6 | 规则没说 | 枚举线程加期限、超时后闸住新开卡；实例额度按进程累计，重建带上限；真挂住复核不了 |
| Q5 交错 | 冲突（背景材料/代码） | 崩溃枚举走显式入口，不走后台入口；一把锁的次序集合是显式入口历史的子集加三种新交错 |
| Q5 新流 | 规则没说 | 不新开枚举，加一条「后台一趟 ≡ 排空加显式入口」的钉相同用例，另扩随机历史字母表 |
| Q6 验收 | 一致 | 清单见 6.1；七·4 的核心层对拍已有，经运行库那一格新加；E202 不改接 |

## 对 `crates/` 的改动清单（实现员照这张表落；每行来自上面哪一节）

| 文件 | 改什么 | 来源 |
|---|---|---|
| 根 `Cargo.toml:6-16` | `members` 加 `crates/singlefs-mount-runtime`；若粘合层单开包，再加一个 | 1.1 |
| 新 `crates/singlefs-mount-runtime/` | 1.3 的类型与线程体；`run_one_maintenance_pass()`；依赖只有 `singlefs-core`，不设 `[features]` | 1.1、1.3、1.4、1.5 |
| `crates/singlefs-core/src/compaction_plan.rs:1405` | 起跑判据换成吃水位与读数的函数，返回 `WatermarkVerdict` | 2.1 |
| `crates/singlefs-core/src/compaction_plan.rs:430-436`、`:570` | `CompactionBatchSize` 加一个产品用的成员，`PRODUCT` 用它；K 的下界见 3.3 | 开头一节「批大小 K 没有产品值」、3.3 |
| `crates/singlefs-core/src/compaction_plan.rs:833` | `CompactionOfThisMount` 加 `futile_at` | 3.2 |
| `crates/singlefs-core/src/mounted_session.rs:2627`、`:2937` | 两处调用换成新函数，读数取 `allocator.devices` | 2.1 |
| `crates/singlefs-core/src/mounted_session.rs:2950`、`:2596` | 按 F-3 选定的那一支改预算 | 3.3 |
| `crates/singlefs-core/src/mounted_session.rs:452` | `BackgroundCompactionNotStarted` 加 `FutileSinceTheLastPublish` | 3.2 |
| `crates/singlefs-core/src/mounted_session.rs:6021` 附近 | 攒批的会话加两个挂着切换的转发方法 | 1.3 表 |
| `crates/singlefs-core/src/mount.rs:9615`、`:9695` | 挂载入口把水位与批大小从参数传进，不再写死 `CompactionSwitches::PRODUCT` | 1.3 表 |
| `crates/singlefs-core/src/system_configuration.rs:349-371`、`:678-682`；`.claude/kb/layout/01-first-txn.md:153`；`.claude/kb/mount-parameters.md` | 按 2.3 选定的那一支改（甲只改措辞，字节不动；乙、丙动字节并同步 checker 与 `invariants/`） | 2.3 |
| `crates/singlefs-core/src/allocator.rs:11-12` | 文件头写「全空聚簇段数逐设备不是增量维护的」，与同文件 `:905` 的「增量维护，读一次 O(1)。」和字段 `:320-322` 相反，是过期的注释，改成现状 | 2.1（量的） |
| 粘合层（产品入口 crate 或单开小包） | 开卡器：进程级实例、泄漏账、实例计数与上限、超时闸 | 4.2、4.3、4.4 |
| `crates/singlefs-verification/tests/` | 6.1 的新用例；`BatchingSessionCall` 加 `RunCompactionInTheBackground`（`history.rs:10055`）；`crates/mutations.tsv` 每条一行变异 | 5.4、6.1 |

## 要主 agent 跑的命令

1. **模型复跑**（纯算术，约几秒）。仓根起；`--frozen` 这一轮没有冻结副本，给仓根本身即可（模型只读两个文件里的常量对锚点）；`--work` 给一个不存在的新目录：
   `task-server/task submit -- bash research/prompts/m4-gpugc-background-runtime-design-r1-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work /home/fy5090/.local/share/singlefs/work/m4-gpugc-background-runtime-design-r1-sonnet/rerun-1`
   判结果：`TASK_SERVER_RESULT` 行 `exit=0`；`task-server/task log <号>` 里每行以 `ok` 起头、没有 `FAIL`，末行 `name=model_summary failures=0`。**我手算的数（B_gc 表、最小 K、赶得上的线、定速调度放行次数）都写进了断言，这一项没跑之前，报告里引模型的数一律当「推的」。**状态：待主 agent 跑。
2. **`Send` 前提**（推的，没编过）。先在草稿目录建仓副本：`python3 scripts/hosts/minimal-copy.py /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpugc-background-runtime-design-r1-sonnet/repo-copy --with-test-inputs`；再 `mkdir -p …/repo-copy/crates/singlefs-core/tests && cp …/assert_the_session_moves_between_threads.rs …/repo-copy/crates/singlefs-core/tests/`；再在副本里交：
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-core 的测试目标 assert_the_session_moves_between_threads; 范围=1 个测试目标; 为什么=核实运行库把攒批的会话与文件盘整个交给另一条线程的前提（Send）' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpugc-background-runtime-design-r1-sonnet/repo-copy -- cargo test -p singlefs-core --test assert_the_session_moves_between_threads`
   判结果：`test result: ok. 1 passed`；编译报错会点名哪个字段不是 `Send`，那就是前提不成立，1.2 与 1.3 要改。状态：待主 agent 跑。用完 `task-server/task clean <号>`，副本 `du -sh` 后删。
3. **没有现成命令、要先写装置才量得了的三个数**（我没写，也没假装量过）：
   - 一趟整理里持锁的每一段的墙钟（1.4 的表）与前台 fsync 的最坏等待：口径是在运行库的假钟之外用真钟量，负载用 E185 的池配方（`.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:286` 起的复跑一节，起法 `scripts/experiment-apparatus/e185-r2-segment-run.sh`）；
   - 越线却产不出段时一次醒来的成本随池大小怎么涨（3.1、H1）：同一配方，造一个全满的池，量「起跑后停在 `NoEligibleSegment`」那一趟的读盘次数与 CPU 时间，几档容量各一遍；
   - 水位线要的两个输入：本机持续写入速率与一趟整理的挂钟 R（2.4 末段，欠账 C412）。
   这三条的装置与登记归实现员或实验设计员，不是这一轮。
4. 交回前我直接跑过 `python3 scripts/agents/cite-check.py research/prompts/m4-gpugc-background-runtime-design-r1-sonnet-output.md --background research/prompts/_m4-gpugc-background-runtime-design-r1-background.md`，末行：「✓ 核了 63 处引文、90 处裸行号，对上 63 处，没判 90 处（其中裸行号没有引文可比 86 处；逐处列在上面）」（量的；裸行号没有引文可比，只核过行号在文件范围内，我读文件时逐处看过）。

## 文件清单与 sha256

```
0d43b9e47ba590710424fa8d877e9c59abf64e9f2941afe0d00bfb3936a2303d  research/prompts/m4-gpugc-background-runtime-design-r1-sonnet-model/watermark_and_budget_model.py
e6da44ec97ea2dd5943f163b4a270aa9e2f2d65ca8074614ab54a8468a23a440  research/prompts/m4-gpugc-background-runtime-design-r1-sonnet-model/tools/rerun.sh
06424250be679337418573ba59d358de573c9d840c6244d258bd4494c911dd18  assert_the_session_moves_between_threads.rs（在 /home/fy5090/.local/share/singlefs/work/m4-gpugc-background-runtime-design-r1-sonnet/ 下）
```

`tools/rerun.sh` 是 `scripts/agents/three-way-rerun-template.sh` 的拷贝，只改了 `rerun_steps` 一段；只做过 `bash -n` 与 `python3 -m py_compile`，没跑过。

## 没做什么

- 不判别的腿的格：没有替主 agent 采纳或出判决；攻方腿主攻的 Q3、Q5 我也答了（正推要答全），但「没打中」「构造不出反例」一类说法一概不写。
- 没往统一任务服务交任务，没起 cargo、没跑模型；判决要引的数没复跑（命令在上一节）。
- 水位的默认值一个也没替用户定：只给了两端边界、赶得上的公式与候选集合；K 的产品值同样没定，只给了下界。
- F-1、F-2、F-3、盘上字段三候选、`gpu_open_deadline`、实例重建策略、F 与 U 是否关闭：都是交用户的岔路，没写倾向，除了 2.2 里「我倾向甲」那一句（已标零轮）。
- 没动 `crates/` 与 kb 里的任何文件；allocator 文件头那条过期注释只登记在改动清单。
