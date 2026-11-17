# m4-gpugc-mount-runtime-code-r1：产品代码改动全文（2026-11-09）

两份补丁里产品代码的块，原样照录。

## 补丁 m4-gpugc-mount-runtime-merged/patch/crates.patch

````diff
diff --git a/crates/singlefs-background-gpu/Cargo.toml b/crates/singlefs-background-gpu/Cargo.toml
index ba182e6d072dd6321bf88d4947fb67d7803e8e64..3daa6611fbbf287d74c6cc05776891ca5d545746 100644
--- a/crates/singlefs-background-gpu/Cargo.toml
+++ b/crates/singlefs-background-gpu/Cargo.toml
@@ -7,6 +7,8 @@ description = "产品的后台 GPU 路径（D26（后台整理与放置回收）
 
 [dependencies]
 singlefs-core = { path = "../singlefs-core" }
+# 开卡器实现运行库的特质（src/gpu_opener_of_this_process.rs）。依赖方向只许本包 → 运行库：运行库只依赖核心层，不开 GPU 的调用方不拖上 wgpu。
+singlefs-mount-runtime = { path = "../singlefs-mount-runtime" }
 # 版本与特性照验证包那一份写死（crates/singlefs-verification/Cargo.toml 的 wgpu 一行）：两个包在同一个工作区里解析成同一份 wgpu。
 wgpu = { version = "=30.0.1", default-features = false, features = ["vulkan", "wgsl", "std"] }
 # 显存总量：wgpu 的适配器信息不报显存，经 wgpu 的 hal 层拿到 Vulkan 实例，读物理设备的内存堆（src/background_gpu.rs 的 largest_device_local_memory_heap）。
@@ -42,3 +44,7 @@ required-features = ["tests-on-a-real-gpu"]
 [[test]]
 name = "compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu"
 required-features = ["tests-on-a-real-gpu"]
+
+[[test]]
+name = "mount_runtime_on_a_real_gpu_writes_the_same_block_sequence_as_on_the_cpu"
+required-features = ["tests-on-a-real-gpu"]
diff --git a/crates/singlefs-background-gpu/src/background_gpu.rs b/crates/singlefs-background-gpu/src/background_gpu.rs
index 79cfa5d10b0d2f7fe1f5b3402b90f2a7567da39b..a86d22cee73a67b0daff6b335a051f214d8c7e6a 100644
--- a/crates/singlefs-background-gpu/src/background_gpu.rs
+++ b/crates/singlefs-background-gpu/src/background_gpu.rs
@@ -751,7 +751,8 @@ fn submissions_of_a_live_slot_plan(plan: &[RecordSliceWithItsChunks]) -> usize {
 /// 枚举只剩软件 Vulkan、三张独显一张都不报（前面的实例都已丢掉），开卡就拒点名的卡不在清单里；只建一个实例、反复枚举，每一次三张都在。
 ///
 /// 这个实例枚举到的驱动清单定在建它的那一刻：之后修好、装上或藏起驱动，同一个实例枚举到的不变，新建的实例才看得到（只换驱动清单的环境变量
-/// `VK_DRIVER_FILES`、`VK_ICD_FILENAMES` 量的，真拔卡、真装驱动没量）。跨多次挂载的长进程遇到修好驱动这种情形怎么办，归运行库的设计，这里没定。
+/// `VK_DRIVER_FILES`、`VK_ICD_FILENAMES` 量的，真拔卡、真装驱动没量）。跨多次挂载的长进程遇到修好驱动这种情形，开卡器（[`crate::gpu_opener_of_this_process`]）
+/// 显式重建，新建的次数有上限。
 #[derive(Clone, Debug)]
 pub struct VulkanInstanceOfThisProcess {
     instance: wgpu::Instance,
diff --git a/crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs b/crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs
new file mode 100644
index 0000000000000000000000000000000000000000..ef8f463216f6a033d57b6fe8e5648704c2d1a490
--- /dev/null
+++ b/crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs
@@ -0,0 +1,238 @@
+//! 开卡器（D26（后台整理与放置回收） 已定项 13 第 4 条「挂载时由调用方把它的实现递进去」）：运行库的特质
+//! [`singlefs_mount_runtime::OpensTheBackgroundGpu`] 在产品 GPU 包这一侧的实现。进程级的三样由它持有：Vulkan 实例（第一次要开卡才建，之后反复用）、
+//! 泄漏账（这个进程里之前几次挂载泄漏在一张卡上的字节，[`GpuMemoryLeakedInThisProcess`]）、建过几个实例的计数。
+//!
+//! 开卡（枚举适配器在内：wgpu 在造 future 那一步同步做完枚举，等待上限管不到它）放工作线程，调用线程至多等 `opening_deadline`。
+//! 到点挂载拒绝，工作线程留在驱动里不杀（没有安全的杀法）；它回来之前闸住新开卡，不再多起一条枚举线程
+//! （[`BackgroundGpuNotOpened::AnEarlierOpeningIsStillHung`]；同一个驱动加载器里再起一条会不会也挂没量过）。它回来之后开出来的东西没人要，随丢弃记进泄漏账，闸放开。
+//! 实例对象在超时之后的状态读不到：造不出真挂住的驱动，这一格核不了。
+//!
+//! 实例枚举到的驱动清单定在建它的那一刻：修好驱动要新建。[`GpuOpenerOfThisProcess::forget_the_instance_so_the_next_opening_creates_one`] 显式重建；
+//! 同一进程里新建到一定数量之后 Vulkan 驱动不再认得出独显（前面的实例丢掉也不还额度），所以新建的次数有上限（[`VULKAN_INSTANCES_OF_THIS_PROCESS_AT_MOST`]），到了就拒，出路是重启进程。
+//!
+//! 只点一张卡也被没点名的卡拖：开卡按卡串行、没参与的卡也开，最坏是卡数乘一张卡的上限（默认期限 [`OPENING_DEADLINE_BY_DEFAULT`]）。第一版接受。
+
+use std::sync::atomic::{AtomicBool, Ordering};
+use std::sync::mpsc::{channel, RecvTimeoutError};
+use std::sync::{Arc, Mutex, PoisonError};
+use std::time::Duration;
+
+use singlefs_core::background_compute::BackgroundComputeSettings;
+use singlefs_mount_runtime::{BackgroundGpuNotOpened, OpenedBackgroundGpu, OpensTheBackgroundGpu};
+
+use crate::adapter_inventory::BackgroundGpuSelection;
+use crate::background_gpu::VulkanInstanceOfThisProcess;
+use crate::background_gpu_compute::open_background_gpu_compute;
+use crate::gpus_of_this_host::GpuMemoryLeakedInThisProcess;
+use crate::waiting_for_the_gpu::WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT;
+
+/// 一个进程至多新建几个 Vulkan 实例（数是拍的：比第二台那一套驱动上量到的约 12 个低得多，那个数只对那一套驱动成立）。
+pub const VULKAN_INSTANCES_OF_THIS_PROCESS_AT_MOST: u32 = 4;
+
+/// 开卡期限的默认：三张卡逐张开各至多一张卡的上限（第二台三张卡，推的，没量过真挂住的驱动）。挂载参数那一层可换（[`GpuOpenerOfThisProcess::of_this_process`]）。
+pub const OPENING_DEADLINE_BY_DEFAULT: Duration =
+    Duration::from_secs(3 * WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT.as_secs());
+
+/// 工作线程上这一次开卡要做的事：拿到这个进程的实例、设置与泄漏账，交回开成的把手或开卡器报的原样。
+type OpeningOnTheWorkerThread<Instance> = Arc<
+    dyn Fn(
+            &Instance,
+            &BackgroundGpuSelection,
+            &GpuMemoryLeakedInThisProcess,
+        ) -> Result<OpenedBackgroundGpu, String>
+        + Send
+        + Sync,
+>;
+
+/// 建这个进程的一个实例。
+type InstanceMaker<Instance> = Arc<dyn Fn() -> Instance + Send + Sync>;
+
+/// 进程级实例与它的计数。
+struct InstanceOfThisProcess<Instance> {
+    instance: Option<Instance>,
+    instances_created: u32,
+}
+
+/// 见模块说明。泛型 `Instance` 只为让不要卡的单测换上不碰 Vulkan 的替身；产品是 [`VulkanInstanceOfThisProcess`]（[`Self::of_this_process`]）。
+pub struct GpuOpenerOfThisProcess<Instance> {
+    opening_deadline: Duration,
+    /// 同一时刻只有一次开卡：两次挂载同时开卡时后到的那一次在这里等。
+    one_opening_at_a_time: Mutex<()>,
+    /// 上一次开卡的工作线程还没回来（调用线程已经到点不等了）。
+    an_opening_is_hung: Arc<AtomicBool>,
+    instance_of_this_process: Arc<Mutex<InstanceOfThisProcess<Instance>>>,
+    memory_leaked_in_this_process: GpuMemoryLeakedInThisProcess,
+    make_an_instance: InstanceMaker<Instance>,
+    open_on_the_worker_thread: OpeningOnTheWorkerThread<Instance>,
+}
+
+impl GpuOpenerOfThisProcess<VulkanInstanceOfThisProcess> {
+    /// 产品的开卡器：一个进程建一份、跨多次挂载留着。实例第一次要开卡才建：选 `cpu` 的挂载不碰 Vulkan。
+    #[must_use]
+    pub fn of_this_process(opening_deadline: Duration) -> Self {
+        Self::with_the_instance_maker_and_the_opening(
+            opening_deadline,
+            Arc::new(VulkanInstanceOfThisProcess::created_for_this_process),
+            Arc::new(the_background_gpu_opened_by_the_product_path),
+        )
+    }
+
+    /// 同 [`Self::of_this_process`]，实例用调用方已经建好的那一份（真卡测试枚举服务交来的卡时建过：一个进程只建一个）。
+    #[must_use]
+    pub fn over_the_vulkan_instance(
+        opening_deadline: Duration,
+        vulkan_instance: VulkanInstanceOfThisProcess,
+    ) -> Self {
+        Self::with_the_instance_maker_and_the_opening(
+            opening_deadline,
+            Arc::new(move || vulkan_instance.clone()),
+            Arc::new(the_background_gpu_opened_by_the_product_path),
+        )
+    }
+}
+
+/// 工作线程上的产品开卡（[`open_background_gpu_compute`]），结局换成运行库收的样子。
+fn the_background_gpu_opened_by_the_product_path(
+    vulkan_instance: &VulkanInstanceOfThisProcess,
+    selection: &BackgroundGpuSelection,
+    memory_leaked_in_this_process: &GpuMemoryLeakedInThisProcess,
+) -> Result<OpenedBackgroundGpu, String> {
+    open_background_gpu_compute(selection, memory_leaked_in_this_process, vulkan_instance)
+        .map(|opened| OpenedBackgroundGpu {
+            report_of_the_opening: format!(
+                "gpus_not_opened={:?} gpus_left_out={:?} unnamed_gpus_left_off={:?}",
+                opened.gpus_not_opened_at_the_mount,
+                opened.gpus_left_out_under_their_memory_limit,
+                opened.unnamed_discrete_gpus_left_off_the_list
+            ),
+            gpu_compute: opened.gpu_compute,
+        })
+        .map_err(|refused| format!("{refused:?}"))
+}
+
+impl<Instance: Clone + Send + 'static> GpuOpenerOfThisProcess<Instance> {
+    /// 实例怎么建、工作线程上怎么开卡由调用方给：产品走 [`GpuOpenerOfThisProcess::of_this_process`]，不要卡的单测换上替身钉期限、闸与计数。
+    #[must_use]
+    pub fn with_the_instance_maker_and_the_opening(
+        opening_deadline: Duration,
+        make_an_instance: InstanceMaker<Instance>,
+        open_on_the_worker_thread: OpeningOnTheWorkerThread<Instance>,
+    ) -> Self {
+        GpuOpenerOfThisProcess {
+            opening_deadline,
+            one_opening_at_a_time: Mutex::new(()),
+            an_opening_is_hung: Arc::new(AtomicBool::new(false)),
+            instance_of_this_process: Arc::new(Mutex::new(InstanceOfThisProcess {
+                instance: None,
+                instances_created: 0,
+            })),
+            memory_leaked_in_this_process: GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+            make_an_instance,
+            open_on_the_worker_thread,
+        }
+    }
+
+    /// 这个进程建过几个实例。
+    #[must_use]
+    pub fn instances_created(&self) -> u32 {
+        self.instance_of_this_process
+            .lock()
+            .unwrap_or_else(PoisonError::into_inner)
+            .instances_created
+    }
+
+    /// 上一次开卡的工作线程还在驱动里没回来。
+    #[must_use]
+    pub fn is_an_opening_hung(&self) -> bool {
+        self.an_opening_is_hung.load(Ordering::SeqCst)
+    }
+
+    /// 修好驱动之后用：丢掉这个进程留着的实例，下一次开卡新建一个（已开着的把手持有自己的设备，不动）。只改内存，新建的计数不还。
+    pub fn forget_the_instance_so_the_next_opening_creates_one(&self) {
+        self.instance_of_this_process
+            .lock()
+            .unwrap_or_else(PoisonError::into_inner)
+            .instance = None;
+    }
+
+    /// 这一次开卡用的实例：有就用留着的，没有就新建、计数加一；计数到上限就拒。在工作线程上调：建实例装 Vulkan 加载器，驱动也可能挂在这里。
+    fn the_instance_or_a_new_one_under_the_limit(
+        instance_of_this_process: &Mutex<InstanceOfThisProcess<Instance>>,
+        make_an_instance: &(dyn Fn() -> Instance + Send + Sync),
+    ) -> Result<Instance, BackgroundGpuNotOpened> {
+        let mut held = instance_of_this_process
+            .lock()
+            .unwrap_or_else(PoisonError::into_inner);
+        if let Some(instance) = &held.instance {
+            return Ok(instance.clone());
+        }
+        if held.instances_created >= VULKAN_INSTANCES_OF_THIS_PROCESS_AT_MOST {
+            return Err(BackgroundGpuNotOpened::InstanceLimitOfThisProcessReached {
+                instances_created: held.instances_created,
+            });
+        }
+        let instance = make_an_instance();
+        held.instances_created += 1;
+        held.instance = Some(instance.clone());
+        Ok(instance)
+    }
+}
+
+impl<Instance: Clone + Send + 'static> OpensTheBackgroundGpu for GpuOpenerOfThisProcess<Instance> {
+    fn open_the_background_gpu(
+        &self,
+        settings: &BackgroundComputeSettings,
+    ) -> Result<OpenedBackgroundGpu, BackgroundGpuNotOpened> {
+        let _only_this_opening = self
+            .one_opening_at_a_time
+            .lock()
+            .unwrap_or_else(PoisonError::into_inner);
+        if self.an_opening_is_hung.load(Ordering::SeqCst) {
+            return Err(BackgroundGpuNotOpened::AnEarlierOpeningIsStillHung);
+        }
+        self.an_opening_is_hung.store(true, Ordering::SeqCst);
+        let (outcome_sender, outcome_receiver) = channel();
+        let an_opening_is_hung = Arc::clone(&self.an_opening_is_hung);
+        let instance_of_this_process = Arc::clone(&self.instance_of_this_process);
+        let make_an_instance = Arc::clone(&self.make_an_instance);
+        let open_on_the_worker_thread = Arc::clone(&self.open_on_the_worker_thread);
+        let memory_leaked_in_this_process = self.memory_leaked_in_this_process.clone();
+        let selection = BackgroundGpuSelection::of_the_settings(settings);
+        let started = std::thread::Builder::new()
+            .name("singlefs-gpu-opener".to_string())
+            .spawn(move || {
+                let outcome = Self::the_instance_or_a_new_one_under_the_limit(
+                    &instance_of_this_process,
+                    &*make_an_instance,
+                )
+                .and_then(|instance| {
+                    open_on_the_worker_thread(&instance, &selection, &memory_leaked_in_this_process)
+                        .map_err(|description| BackgroundGpuNotOpened::RefusedByTheOpening {
+                            description,
+                        })
+                });
+                // 闸先放开再送：调用线程收到结局时下一次开卡不会被这一次闸住。
+                an_opening_is_hung.store(false, Ordering::SeqCst);
+                // 送不出去只有一种：调用线程到点不等了、这次的结局没人要，开成的把手随丢弃记进泄漏账。
+                let _nobody_waits_for_this_opening_any_more = outcome_sender.send(outcome);
+            });
+        if let Err(error) = started {
+            self.an_opening_is_hung.store(false, Ordering::SeqCst);
+            return Err(BackgroundGpuNotOpened::RefusedByTheOpening {
+                description: format!("起不了开卡的工作线程：{error}"),
+            });
+        }
+        match outcome_receiver.recv_timeout(self.opening_deadline) {
+            Ok(outcome) => outcome,
+            Err(RecvTimeoutError::Timeout) => Err(BackgroundGpuNotOpened::TimedOut {
+                deadline: self.opening_deadline,
+            }),
+            Err(RecvTimeoutError::Disconnected) => {
+                self.an_opening_is_hung.store(false, Ordering::SeqCst);
+                Err(BackgroundGpuNotOpened::RefusedByTheOpening {
+                    description: "开卡的工作线程 panic 了".to_string(),
+                })
+            }
+        }
+    }
+}
diff --git a/crates/singlefs-background-gpu/src/lib.rs b/crates/singlefs-background-gpu/src/lib.rs
index 97420824298045aad621db1d2b1621faee08b07b..7888d28dc6779dd6e669290b48180e66893ed29d 100644
--- a/crates/singlefs-background-gpu/src/lib.rs
+++ b/crates/singlefs-background-gpu/src/lib.rs
@@ -25,6 +25,7 @@ pub mod adapter_inventory;
 pub mod background_gpu;
 pub mod background_gpu_compute;
 pub mod checksum_kernels;
+pub mod gpu_opener_of_this_process;
 pub mod gpus_of_this_host;
 pub mod live_slot_kernels;
 pub mod measured_host_cpu_time_per_submission;
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index 0192391c4630c40dd1c94d33d1125a4593aa28a0..908013e6ccd2a8fd36d340f8d5fc47b22ff5c967 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -8,8 +8,7 @@
 //! 落点按设备取（D3（空间分配） 已定项 8 第 1 条「在每一块被选中的设备上各自取」）：每块盘按自己的空闲图各答一个，各盘一致才分配；
 //! 有的盘答不出、或各盘答得不同，在动任何状态之前拒绝（`PlacementRefusal`）——`Placement` 两盘同槽，各盘不同槽第一版不支持。
 //! 记账（D5（快照 / 空间记账机制） 已定项 7 / 已定项 8）：已分配 = 落点之和、容量 = 单元区、runs = 空闲槽的连续段数（逐设备）、
-//! 逐段的已用槽数——这几样在分配那一刻增量维护，运行时不扫盘（`.claude/rules/fs-design.md` 第一格）。全空聚簇段数逐设备不是增量维护的：
-//! `DeviceFreeMap::empty_segments` 每次被读都把内存里逐段的已用槽数数一遍（不读盘，代价随段数长）。
+//! 逐段的已用槽数、全空聚簇段数（逐设备，`DeviceFreeMap::empty_segments`）——这几样在分配那一刻增量维护，运行时不扫盘（`.claude/rules/fs-design.md` 第一格）。
 
 use singlefs_format::{
     ALLOCATION_RECORD_BYTES, CLUSTER_SEGMENT_SLOTS, JOURNAL_RING_DEFAULT_BYTES, ROOT_RING_REGIONS,
diff --git a/crates/singlefs-core/src/compaction_plan.rs b/crates/singlefs-core/src/compaction_plan.rs
index b054f0879ea3b57fddc4dfa18610582692f0d64c..e11f39ed2cb4ce823f6d73181f66cf9c69777a95 100644
--- a/crates/singlefs-core/src/compaction_plan.rs
+++ b/crates/singlefs-core/src/compaction_plan.rs
@@ -16,10 +16,12 @@ use singlefs_replay_prefix::compaction_plan_proofs::{
     CompactionKernelSegment, CompactionKernelSegmentScan,
 };
 
-use crate::address::{DeviceIdentity, SlotNumber};
-use crate::allocator::{AllocationRecord, Placement, UnitAreaStart};
+use crate::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
+use crate::admission::UNRECLAIMABLE_ON_A_DEVICE_OUTSIDE_ZONED;
+use crate::allocator::{AllocationRecord, DeviceFreeMap, Placement, PoolAllocator, UnitAreaStart};
 use crate::background_compute::{BackgroundTaskAfterTheComputeError, ClusterSegmentEntryKey};
 use crate::logged_operations::{CompactionIntent, CompactionRegion};
+use crate::mounted_session::{BackgroundCompactionTrigger, CompactionRunReach};
 use crate::region_scan::{AllocationInRegion, RegionScan, RegionScanRefusal, ScannedAllocation};
 
 /// 选段判据。封闭集合：加一种判据就加一个成员。
@@ -425,12 +427,18 @@ pub fn plan_batch_skipping_the_units_this_publish_rewrites_anyway(
     }
 }
 
-/// 一批至多搬几个单元从哪来。封闭集合。产品路径的批大小条款没写（判决 G2 的 K 归 E10（老化：结构抗老化 vs 事后整理））：
-/// 产品路径照 [`CompactionBatchSize::NotDecidedByTheClauses`]，整理一次运行在签任何意图、发任何一个写之前停下，停机原因
-/// [`CompactionRunStop::BatchSizeNotDecidedByTheClauses`]；只供测试的开关给出一个数才走得进去。
+/// 批大小的产品默认：一批至多搬 16 个单元（D26（后台整理与放置回收） 已定项 1；数是拍的，下界由一份 `B_gc` 做得完一条意图算出：
+/// 默认几何下一条 64 单元的意图要 1 + ⌈64/16⌉ = 5 次发布）。挂载参数，不存盘（[`CompactionBatchSize::OfTheMount`]）。
+pub const COMPACTION_BATCH_UNITS_BY_DEFAULT: CompactionBatchUnits = match NonZeroUsize::new(16) {
+    Some(units) => CompactionBatchUnits(units),
+    None => panic!("16 不是 0"),
+};
+
+/// 一批至多搬几个单元从哪来。封闭集合。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum CompactionBatchSize {
-    NotDecidedByTheClauses,
+    /// 这次挂载给的（挂载参数，默认 [`COMPACTION_BATCH_UNITS_BY_DEFAULT`]）。
+    OfTheMount(CompactionBatchUnits),
     /// 只供测试。
     ForcedForTests(CompactionBatchUnits),
 }
@@ -438,12 +446,13 @@ pub enum CompactionBatchSize {
 /// 整理什么时候起跑。封闭集合。产品路径照水位（[`CompactionRunForcing::FollowTheWatermarks`]）；另两种只供测试。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum CompactionRunForcing {
-    /// 定时兜底那一次与用户改动发成之后那一次按水位判起不起跑（D26（后台整理与放置回收） 已定项 13 第 5 条）。第一版三条水位恒 0、
-    /// 0 = 关，所以这一支在产品路径上不起跑；显式入口（`mounted_session::MountedSession::run_compaction`）
-    /// 不看水位、照常跑。
+    /// 定时兜底那一次与用户改动发成之后那一次按水位判起不起跑（D26（后台整理与放置回收） 已定项 13 第 5 条，[`CompactionWatermarks`]）；
+    /// 后台入口连同不发布的停做的备忘、挂载那一份预算从根环读、破例与回退门都只在这一支判。显式入口
+    /// （`mounted_session::MountedSession::run_compaction`）不看水位、不过回退门，照常跑。
     FollowTheWatermarks,
     /// 只供测试：起跑判据不看水位、恒为真——定时兜底查到的是「该跑」，用户改动发成之后记下的也是「该跑了」，由后台入口
-    /// （`mounted_session::MountedSession::run_compaction_in_the_background`）去跑。
+    /// （`mounted_session::MountedSession::run_compaction_in_the_background`）去跑。备忘、挂载那一份预算从根环读、破例与回退门这一支都不判：
+    /// 用它的用例钉的是一趟怎么推，发布数由用例自己摆。
     ForcedToStartForTests,
     /// 只供测试：两个入口都不起跑，停机原因 [`CompactionRunStop::ForcedToStopForTests`]。
     ForcedToStopForTests,
@@ -567,7 +576,7 @@ pub struct CompactionSwitches {
 
 impl CompactionSwitches {
     pub const PRODUCT: CompactionSwitches = CompactionSwitches {
-        batch_size: CompactionBatchSize::NotDecidedByTheClauses,
+        batch_size: CompactionBatchSize::OfTheMount(COMPACTION_BATCH_UNITS_BY_DEFAULT),
         forced_region: None,
         run_forcing: CompactionRunForcing::FollowTheWatermarks,
         region_exclusion: CompactionRegionExclusion::ExcludedWhileTheIntentIsLive,
@@ -583,8 +592,6 @@ impl CompactionSwitches {
 /// 一次整理运行停在哪。封闭集合。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum CompactionRunStop {
-    /// 产品路径的批大小条款没写（[`CompactionBatchSize::NotDecidedByTheClauses`]）：一个意图都没签、一个写都没发。
-    BatchSizeNotDecidedByTheClauses,
     /// 只供测试的开关叫停（[`CompactionRunForcing::ForcedToStopForTests`]）。
     ForcedToStopForTests,
     /// 一段都没资格（[`RegionChoice::NoEligibleSegment`]），或强制选的那一段不合格。
@@ -632,16 +639,54 @@ pub enum CompactionRunStop {
     /// GPU 路径上选中要签的那一段里有这一轮两份核判得不一样的记录（D26 已定项 13 第 8 条）：R 里要搬几个数不准，签发前那一判判不了，
     /// 这一批不签，下一批重扫照判。
     IssueHeldBackByRecordsUncertainThisRound { records_uncertain_this_round: u64 },
+    /// 后台入口：这次整理发布要写的根槽与它之后 [`ROOT_RING_SLOTS_LEFT_BY_A_COMPACTION_PUBLISH`] 槽里的根一走，根环里不同用户状态就少于 4 个
+    /// （D16（发布语义） 已定项 1 的回退候选集；D26 已定项 13 第 5 条的加门）：不发，一个写都没发。全空段低于底额、前台写不进时每趟第一次发布破例不判。
+    CompactionPublishWouldLeaveFewerThanFourRollbackStates {
+        states_now: u64,
+        states_after_the_publish_and_the_margin: u64,
+    },
+    /// 后台入口：回退门读根环或树表没读通（读不清有几个状态就不发）：一个写都没发，下一次叫起重判。
+    RollbackStatesOfTheRingUnreadable,
+    /// 后台入口放锁的那一种（`mounted_session::MountedSessionWithPendingChanges::run_compaction_in_the_background_for_at_most_one_publish`）：
+    /// 这一趟这一次调用发成了一次，调用方放锁之后接着调。不是停机，这一趟记在会话上没完。
+    OnePublishOfTheTripDone,
 }
 
 impl CompactionRunStop {
+    /// 这一种停在现行那一版不变、开关不变时重判还是同一个结论，又不发布：记进备忘，定时兜底下一次不重判
+    /// （[`CompactionOfThisMount::note_the_stop_of_a_run`]）。选段、签发前判、无产出与回退门读的（分配记录、映射、快照钉住的槽、根环）都随发布变，
+    /// 内存里的隔离集与不合格段表也只在发布里变。GPU 两份核不一致、被节奏或空间截住、出错的那几种下一次可能翻面，不进备忘。
+    #[must_use]
+    pub const fn publishes_nothing_until_the_next_publish(self) -> bool {
+        match self {
+            CompactionRunStop::NoEligibleSegment
+            | CompactionRunStop::HolesOutsideTheRegionFewerThanItsUnitsToMove { .. }
+            | CompactionRunStop::NoYieldFromTheLastIntent { .. }
+            | CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates {
+                ..
+            } => true,
+            CompactionRunStop::ForcedToStopForTests
+            | CompactionRunStop::PublishBudgetSpent { .. }
+            | CompactionRunStop::PausedShortOfSpace
+            | CompactionRunStop::IssueRefusedShortOfSpace
+            | CompactionRunStop::NoLiveIntentLeftToContinue
+            | CompactionRunStop::NothingMovableButTheRegionIsNotEmpty
+            | CompactionRunStop::OneBatchAfterTheUserChangeDone
+            | CompactionRunStop::YieldedToAForegroundFsync
+            | CompactionRunStop::WriteBandwidthLimitReached { .. }
+            | CompactionRunStop::BatchAbandonedAfterABackgroundComputeError { .. }
+            | CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError
+            | CompactionRunStop::OnlyRecordsUncertainThisRoundLeftToMove { .. }
+            | CompactionRunStop::IssueHeldBackByRecordsUncertainThisRound { .. }
+            | CompactionRunStop::RollbackStatesOfTheRingUnreadable
+            | CompactionRunStop::OnePublishOfTheTripDone => false,
+        }
+    }
+
     /// 运行时报出停在哪一条（`.claude/rules/fs-design.md` 五条硬要求第 4 条）。
     #[must_use]
     pub const fn name(self) -> &'static str {
         match self {
-            CompactionRunStop::BatchSizeNotDecidedByTheClauses => {
-                "compaction_stopped_batch_size_not_decided_by_the_clauses"
-            }
             CompactionRunStop::ForcedToStopForTests => "compaction_stopped_by_the_test_only_switch",
             CompactionRunStop::NoEligibleSegment => "compaction_stopped_no_eligible_segment",
             CompactionRunStop::HolesOutsideTheRegionFewerThanItsUnitsToMove { .. } => {
@@ -684,6 +729,13 @@ impl CompactionRunStop {
             CompactionRunStop::IssueHeldBackByRecordsUncertainThisRound { .. } => {
                 "compaction_issue_held_back_by_records_uncertain_on_the_gpu"
             }
+            CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates {
+                ..
+            } => "compaction_stopped_the_publish_would_leave_fewer_than_four_rollback_states",
+            CompactionRunStop::RollbackStatesOfTheRingUnreadable => {
+                "compaction_stopped_the_rollback_states_of_the_ring_are_unreadable"
+            }
+            CompactionRunStop::OnePublishOfTheTripDone => "compaction_one_publish_of_the_trip_done",
         }
     }
 }
@@ -721,6 +773,10 @@ pub struct CompactionObservations {
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct CompactionSwitchesAndTheExcludedRegion {
     pub switches: CompactionSwitches,
+    /// 三条水位线（D26（后台整理与放置回收） 已定项 1，挂载参数，不存盘）。
+    pub watermarks: CompactionWatermarks,
+    /// 只供测试：不看读数，判起跑判据时当这个量越了线（[`CompactionWatermarkCrossingForcing`]）。
+    pub watermark_crossing_forcing: CompactionWatermarkCrossingForcing,
     live_intent: Option<LiveCompactionIntent>,
     has_the_forced_release_checksum_mismatch_been_used: bool,
     quarantines_noted_by_the_publishes: Vec<(SlotNumber, u64)>,
@@ -732,6 +788,8 @@ impl CompactionSwitchesAndTheExcludedRegion {
     pub fn of_a_new_mount() -> Self {
         Self {
             switches: CompactionSwitches::PRODUCT,
+            watermarks: CompactionWatermarks::BY_DEFAULT,
+            watermark_crossing_forcing: CompactionWatermarkCrossingForcing::NotForced,
             live_intent: None,
             has_the_forced_release_checksum_mismatch_been_used: false,
             quarantines_noted_by_the_publishes: Vec::new(),
@@ -836,6 +894,9 @@ pub struct CompactionOfThisMount {
     /// 上一次用户改动之后整理发了几次布：会话发成一次用户改动就归零。
     pub publishes_since_the_last_user_change: u64,
     pub observations: CompactionObservations,
+    /// 最近一次判起跑判据的结论：哪块盘哪个量越了线、读数与线各是多少（`.claude/rules/fs-design.md` 五条硬要求第 4 条）。还没判过是 `None`。
+    /// 判了没起跑（[`WatermarkVerdict::NoLineCrossed`]）也记：它答的就是这一次为什么没起跑，所以不进只记起跑之后那几样的 [`CompactionObservations`]。
+    pub last_watermark_verdict: Option<WatermarkVerdict>,
     /// 用户改动发成之后起跑判据查出来的结果，后台入口取走（[`CompactionStartJudgement`]）。
     pub start_judgement: CompactionStartJudgement,
     /// 这次挂载里上一条完成的意图（「上一条无产出就不签」读它的 `segs0`）：跨趟留着，不随一趟结束丢掉，
@@ -865,6 +926,31 @@ pub struct CompactionOfThisMount {
     /// 只供测试的开关：为另发的那一批推的抬 F 当作自己报错（[`FloorRaiseForTheRelocationKeptForTheSnapshotsFails`]）。新挂载是产品值。
     floor_raise_for_the_relocation_kept_for_the_snapshots_fails:
         FloorRaiseForTheRelocationKeptForTheSnapshotsFails,
+    /// 上一次不发布的停（[`CompactionRunStop::publishes_nothing_until_the_next_publish`]）记在哪一版、哪一组开关上：这两样都没变，定时兜底不重判。
+    stop_that_publishes_nothing_until_the_next_publish:
+        Option<StopThatPublishesNothingUntilTheNextPublish>,
+    /// 这次挂载那一份预算还没从根环读：挂载不给新预算（D26 已定项 13 第 5 条的加门），头一次判后台起跑时
+    /// 把已经过去的发布数取成根环里最新那个用户状态之后的根数；这之前发成用户改动就不用读了。
+    publishes_of_the_mount_are_still_to_be_read_from_the_ring: bool,
+    /// 放锁的后台入口上一次调用之后还没完的那一趟（[`BackgroundTripUnderWay`]）。
+    background_trip_under_way: Option<BackgroundTripUnderWay>,
+}
+
+/// 上一次不发布的停记下的那几样（[`CompactionOfThisMount::note_the_stop_of_a_run`]）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+struct StopThatPublishesNothingUntilTheNextPublish {
+    checkpoint_txg: CheckpointTxg,
+    switches: CompactionSwitches,
+    stop: CompactionRunStop,
+}
+
+/// 放锁的后台入口跨几次调用推的那一趟：一趟只判一次起跑，每次调用至多一次发布。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub(crate) struct BackgroundTripUnderWay {
+    pub(crate) trigger: BackgroundCompactionTrigger,
+    pub(crate) reach: CompactionRunReach,
+    /// 这一趟前几次调用已发的批数（用户改动之后那一趟至多一批，跨调用数）。
+    pub(crate) batches_published: u64,
 }
 
 impl CompactionOfThisMount {
@@ -876,6 +962,7 @@ impl CompactionOfThisMount {
             quarantined_slots: BTreeSet::new(),
             publishes_since_the_last_user_change: 0,
             observations: CompactionObservations::default(),
+            last_watermark_verdict: None,
             start_judgement: CompactionStartJudgement::NotDue,
             last_completed_intent: None,
             timer_fallback_last_check: None,
@@ -889,9 +976,70 @@ impl CompactionOfThisMount {
                 RelocationKeptForTheSnapshotsRefusedNotForSpace::NotForced,
             floor_raise_for_the_relocation_kept_for_the_snapshots_fails:
                 FloorRaiseForTheRelocationKeptForTheSnapshotsFails::NotForced,
+            stop_that_publishes_nothing_until_the_next_publish: None,
+            publishes_of_the_mount_are_still_to_be_read_from_the_ring: true,
+            background_trip_under_way: None,
         }
     }
 
+    /// 一趟停下时记下不发布的停：`current_txg` 是现行那一版，`switches` 是这一趟用的开关。别的结局清掉备忘（下一次照常判）。
+    pub(crate) fn note_the_stop_of_a_run(
+        &mut self,
+        stop: CompactionRunStop,
+        current_txg: CheckpointTxg,
+        switches: CompactionSwitches,
+    ) {
+        self.stop_that_publishes_nothing_until_the_next_publish = stop
+            .publishes_nothing_until_the_next_publish()
+            .then_some(StopThatPublishesNothingUntilTheNextPublish {
+                checkpoint_txg: current_txg,
+                switches,
+                stop,
+            });
+    }
+
+    /// 备忘记着的停这一刻还算不算数：现行那一版与开关都没变才算，交回那一次的停。
+    #[must_use]
+    pub(crate) fn stop_still_holding_at(
+        &self,
+        current_txg: CheckpointTxg,
+        switches: CompactionSwitches,
+    ) -> Option<CompactionRunStop> {
+        self.stop_that_publishes_nothing_until_the_next_publish
+            .filter(|noted| noted.checkpoint_txg == current_txg && noted.switches == switches)
+            .map(|noted| noted.stop)
+    }
+
+    /// 这次挂载那一份预算还要不要从根环读，并记下读过了（读没读通都只读这一次）。
+    pub(crate) fn take_the_reading_of_the_publishes_of_the_mount_from_the_ring(&mut self) -> bool {
+        std::mem::replace(
+            &mut self.publishes_of_the_mount_are_still_to_be_read_from_the_ring,
+            false,
+        )
+    }
+
+    /// 发成一次用户改动：已经过去的发布数从 0 数，根环里最新的用户状态就是这一次，挂载那一份不用再读。
+    pub(crate) fn note_a_user_change_published(&mut self) {
+        self.publishes_since_the_last_user_change = 0;
+        self.publishes_of_the_mount_are_still_to_be_read_from_the_ring = false;
+    }
+
+    /// 取走还没完的那一趟（放锁的后台入口接着推它时）。
+    pub(crate) fn take_the_background_trip_under_way(&mut self) -> Option<BackgroundTripUnderWay> {
+        self.background_trip_under_way.take()
+    }
+
+    /// 记下这一趟这次调用之后还没完。
+    pub(crate) fn keep_the_background_trip_under_way(&mut self, trip: BackgroundTripUnderWay) {
+        self.background_trip_under_way = Some(trip);
+    }
+
+    /// 放锁的后台入口有没有一趟没完（运行库按它决定多久之后再醒）。
+    #[must_use]
+    pub fn has_a_background_trip_under_way(&self) -> bool {
+        self.background_trip_under_way.is_some()
+    }
+
     /// 可写挂载读盘建上一版时只坏一份的次数（[`Self::count_unit_reads_with_one_copy_failing_while_mounting`] 记的）。
     #[must_use]
     pub fn unit_reads_with_one_copy_failing_while_mounting(&self) -> u64 {
@@ -1308,8 +1456,8 @@ pub enum CompactionTimerFallback {
     EveryMilliseconds(NonZeroU64),
 }
 
-/// 定时兜底间隔的默认：关（兜底间隔默认 0（关），
-/// 与水位恒 0 同一个读法）。默认值待 C412（整理的挂钟代价没量过） 与 E10（老化：结构抗老化 vs 事后整理）。
+/// 定时兜底间隔的默认：关。核心层不起线程，调用方不给间隔就没有定时兜底；挂载就起的周期线程那一层给它的间隔是周期的一半
+/// （`singlefs_mount_runtime`，周期默认 30 秒）。
 pub const COMPACTION_TIMER_FALLBACK_BY_DEFAULT: CompactionTimerFallback =
     CompactionTimerFallback::Off;
 
@@ -1373,6 +1521,11 @@ pub trait CallerOfTheBackgroundCompaction {
     fn foreground_fsync(&self) -> ForegroundFsync;
 }
 
+/// 回退门给根环留的余量（D26（后台整理与放置回收） 已定项 13 第 5 条的加门）：一次整理发布之后，再发这么多次发布，根环里仍有 4 个不同用户状态。
+/// 下一次可写挂载自己的发布不经门：写行加暖机至多 3 次，续做一条意图在产品批大小 16 下至多 `1 + ⌈64 ÷ 16⌉ = 5` 次，合起来是 8。
+/// 连崩几次用完这几槽之后，挂载自己的发布照样挤掉用户状态。
+pub const ROOT_RING_SLOTS_LEFT_BY_A_COMPACTION_PUBLISH: u64 = 8;
+
 /// 一次整理运行的发布预算：
 /// `3S − 4 − 已经过去的发布数`，至少 1。`S` 是每区根槽数。
 #[must_use]
@@ -1399,18 +1552,314 @@ pub fn publishes_left_of_the_budget_given_by_the_last_user_change(
         .saturating_sub(publishes_since_the_last_user_change)
 }
 
-/// 起跑判据（水位越线再跑；D26（后台整理与放置回收） 已定项 13 第 5 条：定时兜底每隔一段时间查一次，每次用户改动发成之后也查一次，
-/// 两处查的都是它）。第一版三条水位恒 0（关），产品路径恒不起跑；只供测试的开关强制。
+/// 整理的两样挂载参数（都不存盘）：批大小与三条水位线（[`crate::mount::mount_writable_with_background_compute_settings_the_gc_cpu_compensation_threshold_and_compaction_parameters`]）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct CompactionMountParameters {
+    pub batch_units: CompactionBatchUnits,
+    pub watermarks: CompactionWatermarks,
+}
+
+impl CompactionMountParameters {
+    /// 两样都取产品默认。
+    pub const BY_DEFAULT: CompactionMountParameters = CompactionMountParameters {
+        batch_units: COMPACTION_BATCH_UNITS_BY_DEFAULT,
+        watermarks: CompactionWatermarks::BY_DEFAULT,
+    };
+}
+
+/// 全空聚簇段线的产品默认：整段数的 5%（D26（后台整理与放置回收） 已定项 1，数是拍的）。
+pub const EMPTY_CLUSTER_SEGMENTS_LINE_PERCENT_BY_DEFAULT: u64 = 5;
+/// 全空聚簇段线的下限：2 段（前台写入类发布在全空段低于 `N₀` = 1 时被合取 7 拒，线至少 `N₀ + 1`，不然前台先被拒、整理才起跑）。
+pub const EMPTY_CLUSTER_SEGMENTS_LINE_LOWEST_IN_SEGMENTS_BY_DEFAULT: u64 = 2;
+
+/// 全空聚簇段这条水位（越线：全空聚簇段数 < 线）。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum EmptyClusterSegmentsWatermark {
+    Off,
+    /// 线 = max(⌈整段数 × 百分比 ÷ 100⌉, 下限)，再不超过整段数（只有一两个整段的小盘刚 mkfs 时不越线）。
+    PercentOfTheClusterSegments {
+        percent: u64,
+        lowest_line_in_segments: u64,
+    },
+}
+
+impl EmptyClusterSegmentsWatermark {
+    /// 这块盘的线（段数）；关着是 `None`。
+    #[must_use]
+    pub fn line_in_segments(self, cluster_segments_in_the_unit_area: u64) -> Option<u64> {
+        match self {
+            EmptyClusterSegmentsWatermark::Off => None,
+            EmptyClusterSegmentsWatermark::PercentOfTheClusterSegments {
+                percent,
+                lowest_line_in_segments,
+            } => {
+                let percent_of_the_segments = (u128::from(cluster_segments_in_the_unit_area)
+                    * u128::from(percent))
+                .div_ceil(100);
+                let lowest = u128::from(lowest_line_in_segments);
+                let line = percent_of_the_segments
+                    .max(lowest)
+                    .min(u128::from(cluster_segments_in_the_unit_area));
+                Some(u64::try_from(line).expect("线不超过整段数"))
+            }
+        }
+    }
+}
+
+/// 碎片度这条水位（越线：`空闲段数 ÷ 已分配槽数 > 线`，且空闲段数不少于绝对下限；D26 已定项 1 的 `F = runs / L`，`L` 取已分配槽数，
+/// 条款没定义它，归 C326（`runs` 口径换了之后的 L 没定义））。默认关。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum FragmentationWatermark {
+    Off,
+    /// 线是 `free_runs_above ÷ per_allocated_slots`；`at_least_free_runs` 是这一支开了必带的绝对下限：
+    /// 用得很少的盘上已分配槽数很小而空闲段数至少 1，只看比值近乎空盘就越线。
+    FreeRunsPerAllocatedSlotAbove {
+        free_runs_above: NonZeroU64,
+        per_allocated_slots: NonZeroU64,
+        at_least_free_runs: NonZeroU64,
+    },
+}
+
+/// 不可回收字节这条水位（越线：不可回收字节 ≥ 线）。读数第一版恒 0（[`UNRECLAIMABLE_ON_A_DEVICE_OUTSIDE_ZONED`]），开了也不越线；默认关。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum UnreclaimableBytesWatermark {
+    Off,
+    AtLeast(NonZeroU64),
+}
+
+/// 三条水位线（D26 已定项 1：三水位任一越线起跑，带设备维）。挂载参数，不存盘：盘上系统配置那 24 字节是全 0 的预留。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct CompactionWatermarks {
+    pub empty_cluster_segments: EmptyClusterSegmentsWatermark,
+    pub fragmentation: FragmentationWatermark,
+    pub unreclaimable_bytes: UnreclaimableBytesWatermark,
+}
+
+impl CompactionWatermarks {
+    /// 产品默认：全空聚簇段线 5%、下限 2；碎片度与不可回收字节关。
+    pub const BY_DEFAULT: CompactionWatermarks = CompactionWatermarks {
+        empty_cluster_segments: EmptyClusterSegmentsWatermark::PercentOfTheClusterSegments {
+            percent: EMPTY_CLUSTER_SEGMENTS_LINE_PERCENT_BY_DEFAULT,
+            lowest_line_in_segments: EMPTY_CLUSTER_SEGMENTS_LINE_LOWEST_IN_SEGMENTS_BY_DEFAULT,
+        },
+        fragmentation: FragmentationWatermark::Off,
+        unreclaimable_bytes: UnreclaimableBytesWatermark::Off,
+    };
+}
+
+/// 一块盘此刻的三个量（分配器增量维护的计数，读一次 O(1)，与记账树写出去的是同一份）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct WatermarkReadingOfADevice {
+    pub device: DeviceIdentity,
+    pub empty_cluster_segments: u64,
+    pub cluster_segments_in_the_unit_area: u64,
+    pub free_runs: u64,
+    pub allocated_slots: u64,
+    pub unreclaimable_bytes: u64,
+}
+
+impl WatermarkReadingOfADevice {
+    /// 从分配器上这块盘的空闲图读。全空聚簇段数是记录口径：一段里只剩 defer 槽也算全空
+    /// （`allocator::DeviceFreeMap::empty_segments`，开段看的是另一个量，`allocator::DeviceFreeMap::lowest_empty_segment`）。
+    #[must_use]
+    pub fn of_the_device_map(device_map: &DeviceFreeMap) -> Self {
+        WatermarkReadingOfADevice {
+            device: device_map.device,
+            empty_cluster_segments: device_map.empty_segments(),
+            cluster_segments_in_the_unit_area: device_map.unit_area_slots() / CLUSTER_SEGMENT_SLOTS,
+            free_runs: device_map.free_runs(),
+            allocated_slots: device_map.allocated_slots(),
+            unreclaimable_bytes: UNRECLAIMABLE_ON_A_DEVICE_OUTSIDE_ZONED.0,
+        }
+    }
+}
+
+/// 三个量。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum WatermarkQuantity {
+    EmptyClusterSegments,
+    Fragmentation,
+    UnreclaimableBytes,
+}
+
+/// 只供测试（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：不看读数，判起跑判据时当这个量越了线。产品路径恒 [`Self::NotForced`]。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum CompactionWatermarkCrossingForcing {
+    NotForced,
+    ForcedForTests(WatermarkQuantity),
+}
+
+/// 起跑判据的结论，连同走了哪一支（`.claude/rules/fs-design.md` 五条硬要求第 4 条）。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum WatermarkVerdict {
+    /// 只供测试的开关强制起跑（[`CompactionRunForcing::ForcedToStartForTests`]）。
+    ForcedToStartByTheTestOnlySwitch,
+    /// 只供测试的开关强制不起跑（[`CompactionRunForcing::ForcedToStopForTests`]）。
+    ForcedToStopByTheTestOnlySwitch,
+    /// 只供测试的开关当这个量越了线（[`CompactionWatermarkCrossingForcing::ForcedForTests`]）。
+    CrossingForcedByTheTestOnlySwitch(WatermarkQuantity),
+    /// 每块盘的三个量都没越线。
+    NoLineCrossed,
+    /// 按盘的次序、每块盘按全空聚簇段、碎片度、不可回收字节的次序头一个越线的。碎片度的 `reading` 是空闲段数、`line` 是把线换算到这块盘的已分配槽数上的空闲段数。
+    Crossed {
+        device: DeviceIdentity,
+        quantity: WatermarkQuantity,
+        reading: u64,
+        line: u64,
+    },
+}
+
+impl WatermarkVerdict {
+    /// 这一结论起不起跑。
+    #[must_use]
+    pub const fn starts(self) -> bool {
+        match self {
+            WatermarkVerdict::ForcedToStartByTheTestOnlySwitch
+            | WatermarkVerdict::CrossingForcedByTheTestOnlySwitch(_)
+            | WatermarkVerdict::Crossed { .. } => true,
+            WatermarkVerdict::ForcedToStopByTheTestOnlySwitch | WatermarkVerdict::NoLineCrossed => {
+                false
+            }
+        }
+    }
+}
+
+/// 一块盘头一个越线的量（全空聚簇段、碎片度、不可回收字节的次序），连同读数与线。
+fn watermark_crossed_on_the_device(
+    watermarks: &CompactionWatermarks,
+    reading: &WatermarkReadingOfADevice,
+) -> Option<(WatermarkQuantity, u64, u64)> {
+    if let Some(line) = watermarks
+        .empty_cluster_segments
+        .line_in_segments(reading.cluster_segments_in_the_unit_area)
+    {
+        if reading.empty_cluster_segments < line {
+            return Some((
+                WatermarkQuantity::EmptyClusterSegments,
+                reading.empty_cluster_segments,
+                line,
+            ));
+        }
+    }
+    match watermarks.fragmentation {
+        FragmentationWatermark::Off => {}
+        FragmentationWatermark::FreeRunsPerAllocatedSlotAbove {
+            free_runs_above,
+            per_allocated_slots,
+            at_least_free_runs,
+        } => {
+            let line_on_this_device = u128::from(free_runs_above.get())
+                * u128::from(reading.allocated_slots)
+                / u128::from(per_allocated_slots.get());
+            let is_above_the_line = u128::from(reading.free_runs)
+                * u128::from(per_allocated_slots.get())
+                > u128::from(free_runs_above.get()) * u128::from(reading.allocated_slots);
+            if reading.allocated_slots > 0
+                && is_above_the_line
+                && reading.free_runs >= at_least_free_runs.get()
+            {
+                return Some((
+                    WatermarkQuantity::Fragmentation,
+                    reading.free_runs,
+                    u64::try_from(line_on_this_device).expect("低于空闲段数读数，装得进 u64"),
+                ));
+            }
+        }
+    }
+    match watermarks.unreclaimable_bytes {
+        UnreclaimableBytesWatermark::Off => None,
+        UnreclaimableBytesWatermark::AtLeast(line) => (reading.unreclaimable_bytes >= line.get())
+            .then_some((
+                WatermarkQuantity::UnreclaimableBytes,
+                reading.unreclaimable_bytes,
+                line.get(),
+            )),
+    }
+}
+
+/// 起跑判据（水位越线再跑；D26（后台整理与放置回收） 已定项 1、已定项 13 第 5 条：定时兜底每隔一段时间查一次，每次用户改动发成之后也查一次，
+/// 两处查的都是它）：任一块盘的任一个量越线就起跑。只读交进来的读数，代价随盘数、不随盘容量（`.claude/rules/fs-design.md`「记账是事务的副产品」第一格）。
+/// 水位只在起跑时查，停机不靠水位（D26 已定项 1 第 2 条），所以没有回滞。
 #[must_use]
-pub const fn compaction_starts_by_the_watermarks(forcing: CompactionRunForcing) -> bool {
+pub fn compaction_starts_by_the_watermarks(
+    forcing: CompactionRunForcing,
+    watermarks: &CompactionWatermarks,
+    readings: &[WatermarkReadingOfADevice],
+) -> WatermarkVerdict {
     match forcing {
-        CompactionRunForcing::FollowTheWatermarks | CompactionRunForcing::ForcedToStopForTests => {
-            false
+        CompactionRunForcing::ForcedToStartForTests => {
+            return WatermarkVerdict::ForcedToStartByTheTestOnlySwitch
+        }
+        CompactionRunForcing::ForcedToStopForTests => {
+            return WatermarkVerdict::ForcedToStopByTheTestOnlySwitch
+        }
+        CompactionRunForcing::FollowTheWatermarks => {}
+    }
+    for reading in readings {
+        if let Some((quantity, value, line)) = watermark_crossed_on_the_device(watermarks, reading)
+        {
+            return WatermarkVerdict::Crossed {
+                device: reading.device,
+                quantity,
+                reading: value,
+                line,
+            };
+        }
+    }
+    WatermarkVerdict::NoLineCrossed
+}
+
+/// 这个分配器此刻的起跑判据结论：开关、线与每块盘的读数都从它取（[`compaction_starts_by_the_watermarks`]）。
+#[must_use]
+pub fn compaction_watermark_verdict_of_the_allocator(
+    allocator: &PoolAllocator,
+) -> WatermarkVerdict {
+    let compaction = allocator.compaction();
+    match (
+        compaction.switches.run_forcing,
+        compaction.watermark_crossing_forcing,
+    ) {
+        (
+            CompactionRunForcing::FollowTheWatermarks,
+            CompactionWatermarkCrossingForcing::ForcedForTests(quantity),
+        ) => WatermarkVerdict::CrossingForcedByTheTestOnlySwitch(quantity),
+        (
+            CompactionRunForcing::FollowTheWatermarks
+            | CompactionRunForcing::ForcedToStartForTests
+            | CompactionRunForcing::ForcedToStopForTests,
+            CompactionWatermarkCrossingForcing::NotForced
+            | CompactionWatermarkCrossingForcing::ForcedForTests(_),
+        ) => {
+            let readings: Vec<WatermarkReadingOfADevice> = allocator
+                .devices
+                .iter()
+                .map(WatermarkReadingOfADevice::of_the_device_map)
+                .collect();
+            compaction_starts_by_the_watermarks(
+                compaction.switches.run_forcing,
+                &compaction.watermarks,
+                &readings,
+            )
         }
-        CompactionRunForcing::ForcedToStartForTests => true,
     }
 }
 
+/// 全空段低于底额、前台写入类发布已写不进（合取 7，`N₀` 是开关给的那一档）：任一块盘低于就算。后台入口的破例（每趟至少发 1 次）判它，代价随盘数。
+#[must_use]
+pub fn foreground_writes_are_below_the_empty_cluster_segments_floor(
+    allocator: &PoolAllocator,
+) -> bool {
+    let floor = allocator
+        .compaction()
+        .switches
+        .empty_cluster_segments_floor
+        .segments();
+    allocator
+        .devices
+        .iter()
+        .any(|device_map| device_map.empty_segments() < floor)
+}
+
 /// 与 Verus 证过的选段核、计划核逐项对拍（`.claude/kb/milestone/04-fourth-txn.md` 二·4；
 /// 第三节 G7 的 P1、P2、P3 证在 `singlefs_replay_prefix::compaction_plan_proofs` 的两个核上）：小输入上每一种形状都喂给这里的
 /// [`choose_region`]、[`plan_batch`] 与核，答案逐项相同。核那一侧的入参由用例按形状直接写出，不经这里的代码算，
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 1146261c97b6b9d8eb5b4e94dee08e2bccc75fc2..9662c9b3feaf19c867c7f4d50cc7bc75ff76f969 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -31,7 +31,10 @@ use crate::background_compute::{
 };
 use crate::block_device::BlockDevice;
 use crate::checksum::crc32_castagnoli;
-use crate::compaction_plan::{CompactionOfThisMount, CompactionSwitches};
+use crate::compaction_plan::{
+    CompactionBatchSize, CompactionMountParameters, CompactionOfThisMount, CompactionSwitches,
+    ROOT_RING_SLOTS_LEFT_BY_A_COMPACTION_PUBLISH,
+};
 use crate::instance_table_sweep::{
     sweep_the_instance_table_at_the_end_of_the_mount, InstanceTableSweepAtTheEndOfTheMount,
     InstanceTableSweepConfiguration,
@@ -4370,6 +4373,105 @@ pub fn newest_first_distinct_states(mut non_empty: Vec<NonEmptyValidRoot>) -> Ve
     distinct
 }
 
+/// 根环此刻的回退状态（后台整理的回退门与挂载那一份预算读它；D16（发布语义） 已定项 1「4 个不同状态」，数法同 [`newest_first_distinct_states`]，
+/// 整理搬了用户可见树的根单元的那一批照字节比算非空）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub(crate) struct RollbackStatesOfTheRing {
+    /// 根环里此刻不同用户状态的个数。
+    pub(crate) states_now: u64,
+    /// 下一次发布写的那一槽与它之后 [`ROOT_RING_SLOTS_LEFT_BY_A_COMPACTION_PUBLISH`] 槽里的根都走了之后还剩几个（走的那一条是某个状态在环里最后一条时才少一个）。
+    pub(crate) states_after_the_next_publish_and_the_margin: u64,
+    /// 比最新那个用户状态的根新的有效根条数（环里一个状态都没有时是有效根的条数）：挂载那一份预算当它是已经过去的发布数。
+    pub(crate) publishes_after_the_newest_state: u64,
+}
+
+/// 一串有效根（不分先后）从新到旧的不同状态：按 (txg, 实例代号) 逐条跟前一条比认非空（同 `admission_ceilings_not_passing_ring_slots_bad_since_the_mount`
+/// 那一判，最旧的一条跟 [`UserVisibleTreeRootPointers::ABSENT`] 比），再照 [`newest_first_distinct_states`] 去重。纯函数，读根与读树表在调用方。
+fn distinct_states_of_the_valid_roots<'root>(
+    valid_roots: impl Iterator<Item = &'root (RootRecord, UserVisibleTreeRootPointers)>,
+) -> Vec<CheckpointTxg> {
+    let mut oldest_first: Vec<&(RootRecord, UserVisibleTreeRootPointers)> = valid_roots.collect();
+    oldest_first.sort_by_key(|(root, _)| (root.checkpoint_txg, root.instance));
+    let absent = UserVisibleTreeRootPointers::ABSENT;
+    let mut previous_pointers = &absent;
+    let mut non_empty: Vec<NonEmptyValidRoot> = Vec::new();
+    for (root, pointers) in oldest_first {
+        if user_visible_trees_changed(pointers, previous_pointers) {
+            non_empty.push(NonEmptyValidRoot {
+                checkpoint_txg: root.checkpoint_txg,
+                instance: root.instance,
+                user_visible_tree_root_pointers: pointers.clone(),
+            });
+        }
+        previous_pointers = pointers;
+    }
+    newest_first_distinct_states(non_empty)
+}
+
+/// 读根环，交回 [`RollbackStatesOfTheRing`]。有效根同算抬 F 的上限：不低于现行那一版的 F、不被现行那一版的实例表抛弃。读不出的槽当没有根
+/// （少数一个状态：回退门宁可不发）。读的是整个根环与每条有效根的树表，代价随根环槽数（3S），不随盘容量。
+///
+/// # Errors
+/// 现行那一版的实例表读不通，或一条有效根的树表读不通。
+pub(crate) fn rollback_states_of_the_ring(
+    reader: &dyn PoolReader,
+    parameters: &MakeFilesystemParameters,
+    current_root: &RootRecord,
+) -> Result<RollbackStatesOfTheRing, RecoveryFailure> {
+    let table = instance_table_chain_of_root(reader, current_root)?.records;
+    let ring_slots_of_the_next_publish_and_the_margin: Vec<RootRingSlot> = (1..=1
+        + ROOT_RING_SLOTS_LEFT_BY_A_COMPACTION_PUBLISH)
+        .map(|publishes_ahead| {
+            target_for_publish(
+                CheckpointTxg(current_root.checkpoint_txg.0 + publishes_ahead),
+                parameters.geometry.root_ring_slots_per_region,
+            )
+        })
+        .collect();
+    let mut valid_roots: Vec<(RootRecord, UserVisibleTreeRootPointers)> = Vec::new();
+    let mut valid_roots_past_the_next_publish_and_the_margin: Vec<(
+        RootRecord,
+        UserVisibleTreeRootPointers,
+    )> = Vec::new();
+    for ring_slot in every_root_ring_slot(&parameters.geometry) {
+        let reading = read_root_ring_slot(
+            reader,
+            &parameters.region_devices,
+            &parameters.geometry,
+            &parameters.filesystem_identifier,
+            ring_slot,
+        );
+        let RootRingSlotReading::SelfVerified(root) = reading else {
+            continue;
+        };
+        if root.checkpoint_txg < current_root.rollback_floor || abandoned_by_table(&root, &table) {
+            continue;
+        }
+        let pointers = user_visible_tree_root_pointers(reader, &root)?;
+        if !ring_slots_of_the_next_publish_and_the_margin.contains(&ring_slot) {
+            valid_roots_past_the_next_publish_and_the_margin.push((root, pointers.clone()));
+        }
+        valid_roots.push((root, pointers));
+    }
+    let states_now = distinct_states_of_the_valid_roots(valid_roots.iter());
+    let states_after_the_next_publish_and_the_margin =
+        distinct_states_of_the_valid_roots(valid_roots_past_the_next_publish_and_the_margin.iter());
+    let newest_state = states_now.first().copied();
+    let publishes_after_the_newest_state = valid_roots
+        .iter()
+        .filter(|(root, _)| newest_state.is_none_or(|newest| root.checkpoint_txg > newest))
+        .count();
+    Ok(RollbackStatesOfTheRing {
+        states_now: u64::try_from(states_now.len()).expect("状态数不超过根环槽数"),
+        states_after_the_next_publish_and_the_margin: u64::try_from(
+            states_after_the_next_publish_and_the_margin.len(),
+        )
+        .expect("状态数不超过根环槽数"),
+        publishes_after_the_newest_state: u64::try_from(publishes_after_the_newest_state)
+            .expect("根数不超过根环槽数"),
+    })
+}
+
 /// 上限 = min(每块盘上最新的有效根, 第 4 新的不同状态的非空有效根)；去重之后不足 4 个时取最旧的有效根；一条有效根都没有时 `None`。
 /// `distinct_states_newest_first` 是 [`newest_first_distinct_states`] 交回的那一串，`oldest_valid_root` 是有效根里最小的 txg。
 fn ceiling_from_newest_and_non_empty_roots(
@@ -8169,7 +8271,8 @@ fn writes_of_the_publishes_before_the_continuation(
 
 /// 整理意图的续做（判决 G1；D8（核心索引结构） 已定项 5 第 2 条「续做是每次恢复都跑的常规动作」；判决 G8 次序：删除续做 → 抬 F → 整理）：
 /// 现行那一版 logged_ops 里有一条未完成的意图、删除队列走空了，就照分配器上的整理开关推它，推到它删了、停机原因或预算用完，不签新的。
-/// 整理自己不让挂载失败（提示类意图不挡挂载，D8 已定项 5），走到哪儿记进交回值；产品路径批大小条款没写，签任何东西、发任何写之前停下。
+/// 整理自己不让挂载失败（提示类意图不挡挂载，D8 已定项 5），走到哪儿记进交回值。批大小取分配器上开关给的那一个（产品默认 16，挂载参数）。
+/// 回退门不判这一趟：续做只推盘上那一条、不签新的，发布数有上界（1 + ⌈64 ÷ 批大小⌉）。
 /// 删除续做没走空时不推（整理排在它后面），一个写都不发。续做的每一批照这次挂载给的后台计算设置记账（`background_compute`）。
 fn compaction_intent_continued_at_the_mount<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
@@ -9490,7 +9593,7 @@ pub fn mount_writable_with_test_only_switches_and_deletion_switches<Device: Bloc
 
 /// 同 [`mount_writable_with_test_only_switches`]，整理的只供测试的开关（`compaction_plan::CompactionSwitches`，判决 G6）也由调用方给：
 /// 装在交回的分配器上，挂载续做盘上那一条未完成的整理意图（[`MountOutput::compaction_continuation`]）与这次挂载之后的会话照它。
-/// 产品路径走 [`mount_writable`]，恒 `CompactionSwitches::PRODUCT`（批大小条款没写，续做在签任何东西之前停下）。
+/// 产品路径走 [`mount_writable`]，恒 `CompactionSwitches::PRODUCT`（批大小取默认）。
 ///
 /// # Errors
 /// 同 [`mount_writable_with_test_only_switches`]：续做自己不报错（提示类意图不挡挂载，D8（核心索引结构） 已定项 5），走到哪儿记进交回值。
@@ -9513,6 +9616,31 @@ pub fn mount_writable_with_compaction_switches<Device: BlockDevice>(
     )
 }
 
+/// 同 [`mount_writable_with_space_admission`]，整理的只供测试的开关也由调用方给（随机历史的执行器要它：挂载里不续做整理、下一趟整理接着推盘上那一条，
+/// 逐版比那一串里没有续做整理的发布）。
+///
+/// # Errors
+/// 同 [`mount_writable_with_space_admission`]。
+pub fn mount_writable_with_space_admission_and_compaction_switches<Device: BlockDevice>(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut Vec<(DeviceIdentity, Device)>,
+    space_admission: SpaceAdmission,
+    compaction_switches: CompactionSwitches,
+) -> Result<Mounted, MountError> {
+    mount_writable_with_every_test_only_switch(
+        parameters,
+        devices,
+        space_admission,
+        ShadowLedger::On,
+        FirstTxgOfTheNewInstance::PastTheHighestRootAndRecordInTheRing,
+        DeletionSwitches::PRODUCT,
+        compaction_switches,
+        BeforeTheOneReread::RereadImmediately,
+        InstanceTableSweepConfiguration::PRODUCT,
+        AdmittedBackgroundComputeSettings::PRODUCT,
+    )
+}
+
 /// 同 [`mount_writable_with_test_only_switches`]，交回之前那一判实例表清扫的开关与阈值也由调用方给
 /// （D18（块里携带什么信息） 已定项 11「回收」一条：开关与阈值是配置项，阈值默认 365 行，用户 2026-10-29 定，C690；
 /// [`crate::instance_table_sweep::InstanceTableSweepConfiguration`]）。产品路径走 [`mount_writable`]，取
@@ -9620,11 +9748,59 @@ pub fn mount_writable_with_background_compute_settings_and_the_gc_cpu_compensati
     .map_err(WritableMountWithBackgroundComputeRefused::Mount)
 }
 
+/// 同 [`mount_writable_with_background_compute_settings_and_the_gc_cpu_compensation_threshold`]，另给整理的两样挂载参数：批大小与三条水位线
+/// （[`CompactionMountParameters`]；D26（后台整理与放置回收） 已定项 1，不存盘）。挂载续做盘上那一条未完成的整理意图照这个批大小推。
+/// 挂载就起周期线程的运行库（`singlefs_mount_runtime`）走这一个入口。
+///
+/// # Errors
+/// 同 [`mount_writable_with_background_compute_settings_and_the_gc_cpu_compensation_threshold`]。
+pub fn mount_writable_with_background_compute_settings_the_gc_cpu_compensation_threshold_and_compaction_parameters<
+    Device: BlockDevice,
+>(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut Vec<(DeviceIdentity, Device)>,
+    background_compute_settings: BackgroundComputeSettings,
+    gpu_compute: Option<BackgroundGpuComputeHandle>,
+    gc_cpu_compensation_threshold: BackgroundGcCpuCompensationThreshold,
+    compaction_parameters: CompactionMountParameters,
+) -> Result<Mounted, WritableMountWithBackgroundComputeRefused> {
+    let CompactionSwitches {
+        batch_size: _,
+        forced_region,
+        run_forcing,
+        region_exclusion,
+        destination_policy,
+        release_checksum_mismatch_after_issue,
+        empty_cluster_segments_floor,
+        relocation_publish_order,
+    } = CompactionSwitches::PRODUCT;
+    let mut mounted =
+        mount_writable_with_background_compute_settings_the_gc_cpu_compensation_threshold_and_compaction_switches(
+            parameters,
+            devices,
+            background_compute_settings,
+            gpu_compute,
+            gc_cpu_compensation_threshold,
+            CompactionSwitches {
+                batch_size: CompactionBatchSize::OfTheMount(compaction_parameters.batch_units),
+                forced_region,
+                run_forcing,
+                region_exclusion,
+                destination_policy,
+                release_checksum_mismatch_after_issue,
+                empty_cluster_segments_floor,
+                relocation_publish_order,
+            },
+        )?;
+    mounted.allocator.compaction_mut().watermarks = compaction_parameters.watermarks;
+    Ok(mounted)
+}
+
 /// 同 [`mount_writable_with_background_compute_settings_and_the_gc_cpu_compensation_threshold`]，整理的只供测试的开关
 /// （`compaction_plan::CompactionSwitches`，判决 G6）也由调用方给（[`mount_writable_with_compaction_switches`] 那一种做法）：装在交回的分配器上，
 /// 挂载续做盘上那一条未完成的整理意图照它与这次挂载给的后台计算设置推，这次挂载之后的会话照它。可写挂载续做里一批在 GPU 上出错停下那一种
 /// 由它强制进入（`.claude/rules/fs-design.md` 五条硬要求第 2 条；判决 `research/prompts/m4-gpugc-code-r2-main-verification.md` 四之一）：
-/// 产品路径的入口恒给 `CompactionSwitches::PRODUCT`，批大小条款没写，续做在签任何东西、交任何一段给 GPU 之前停下。
+/// 产品路径的入口只有批大小可调（[`mount_writable_with_background_compute_settings_the_gc_cpu_compensation_threshold_and_compaction_parameters`]）。
 /// 与别的只供测试的开关一样在发布构建里够得着（C921（只供测试的开关在发布构建里够得着））。
 ///
 /// # Errors
diff --git a/crates/singlefs-core/src/mounted_session.rs b/crates/singlefs-core/src/mounted_session.rs
index 2f527099a4ce727b933d206abb80f0be82405c38..0d38b2cb98ea7a9632687d6c22437cfee1d2c8cf 100644
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -45,17 +45,18 @@ use crate::block_device::{
 };
 use crate::compaction_plan::{
     choose_region, choose_region_over_the_live_slots, cluster_segment_entry_keys_of_the_unit_area,
-    compaction_starts_by_the_watermarks, empty_cluster_segments_by_the_records,
-    holes_admit_the_issue, holes_outside_the_region_in_segments_holding_user_data,
+    compaction_watermark_verdict_of_the_allocator, empty_cluster_segments_by_the_records,
+    foreground_writes_are_below_the_empty_cluster_segments_floor, holes_admit_the_issue,
+    holes_outside_the_region_in_segments_holding_user_data,
     plan_batch_skipping_the_units_this_publish_rewrites_anyway, publish_budget_of_a_compaction_run,
     publishes_left_of_the_budget_given_by_the_last_user_change, region_holds_a_quarantined_slot,
     region_is_empty_by_the_records, region_scans_of_the_segments_without_an_unreleased_record,
-    the_completed_intent_yielded, CallerMonotonicClockReading, CallerOfTheBackgroundCompaction,
-    CompactionBatchSize, CompactionBatchUnits, CompactionOfThisMount, CompactionPacing,
-    CompactionRunForcing, CompactionRunStop, CompactionStartJudgement,
-    CompactionWriteBandwidthLimit, ForegroundFsync, LiveSlotsOfAClusterSegment, PlannedBatch,
-    PlannedRelocation, RegionChoice, RegionChoiceCriterion, TimerFallbackCheck,
-    UnitAreaOfOneDevice,
+    the_completed_intent_yielded, BackgroundTripUnderWay, CallerMonotonicClockReading,
+    CallerOfTheBackgroundCompaction, CompactionBatchSize, CompactionBatchUnits,
+    CompactionOfThisMount, CompactionPacing, CompactionRunForcing, CompactionRunStop,
+    CompactionStartJudgement, CompactionSwitches, CompactionWriteBandwidthLimit, ForegroundFsync,
+    LiveSlotsOfAClusterSegment, PlannedBatch, PlannedRelocation, RegionChoice,
+    RegionChoiceCriterion, TimerFallbackCheck, UnitAreaOfOneDevice,
 };
 use crate::instance_table::InstanceRow;
 use crate::instance_table_sweep::unit_head_read_by_sweep;
@@ -69,17 +70,17 @@ use crate::mount::{
     empty_publish_plan_after_file_version,
     own_device_numbers_on_disk_differing_from_the_identities_handed_in,
     parameters_of_the_pool_agreeing_with_the_caller_inputs,
-    push_one_floor_raise_within_the_admission_budget, DeviceBehindTheCurrentVersion,
-    FloorRaisePushedWithinTheAdmissionBudget, FloorRaiseStop, MountError, MountOutput, Mounted,
-    OwnDeviceNumberOnDiskDifferingFromTheIdentityHandedIn, ParametersAndDeviceTableOfTheMount,
-    RaisedFloor, ShadowLedger,
+    push_one_floor_raise_within_the_admission_budget, rollback_states_of_the_ring,
+    DeviceBehindTheCurrentVersion, FloorRaisePushedWithinTheAdmissionBudget, FloorRaiseStop,
+    MountError, MountOutput, Mounted, OwnDeviceNumberOnDiskDifferingFromTheIdentityHandedIn,
+    ParametersAndDeviceTableOfTheMount, RaisedFloor, ShadowLedger,
 };
 use crate::mounted_read::{
     open_pool_for_read, FileReadFailure, FileReadOutput, OpenFileFailure, OpenPoolForReadFailure,
 };
 use crate::recovery::{
     instance_table_chain_of_root, PoolReader, PoolReaderCountingUnitReadsWithOneCopyFailing,
-    ReadPassOverThePool,
+    PoolReaderOfOneReadPass, ReadPassOverThePool,
 };
 use crate::region_scan::{
     scan_region_on_the_gpu, scan_region_over_the_identity_index, AllocationInRegion,
@@ -100,14 +101,14 @@ use crate::transaction::{
     publish_the_relocations_and_the_repair_of_the_exempt_units, publish_unlink_intent,
     publish_version, relocations_of_old_data_units_kept_for_the_snapshots,
     relocations_of_the_units_read_with_one_copy_failing_while_mounting,
-    repair_of_the_exempt_units_at_these_locations, units_at_these_locations,
-    walk_the_deletion_queue_until_it_is_empty, CentralMappingOfAVersionReadLeafByLeaf,
-    CompactionIntentStep, CompactionStepRefused, DataChangesPublishedTogether,
-    DeletionAndLineageOfThePublish, DeletionQueueWalk, FirstFile, MappingLookup,
-    NewInodesOfOneChange, PartialWriteBytes, PlannedUnitsRewrittenAnyway, PoolVersion, PoolWriter,
-    PublishError, ReadFailedOnTheDisk, ReadFailingSelfVerification, RegionCheckOfTheIntentDeletion,
-    ReleaseChecksumCheck, RelocationBatchRefused, RepairOfTheExemptUnits, TransactionOutput,
-    TransactionUnit, WholeWriteOfOneInode,
+    repair_of_the_exempt_units_at_these_locations, resend_the_frozen_publish,
+    units_at_these_locations, walk_the_deletion_queue_until_it_is_empty,
+    CentralMappingOfAVersionReadLeafByLeaf, CompactionIntentStep, CompactionStepRefused,
+    DataChangesPublishedTogether, DeletionAndLineageOfThePublish, DeletionQueueWalk, FirstFile,
+    MappingLookup, NewInodesOfOneChange, PartialWriteBytes, PlannedUnitsRewrittenAnyway,
+    PoolVersion, PoolWriter, PublishError, ReadFailedOnTheDisk, ReadFailingSelfVerification,
+    RegionCheckOfTheIntentDeletion, ReleaseChecksumCheck, RelocationBatchRefused,
+    RepairOfTheExemptUnits, TransactionOutput, TransactionUnit, WholeWriteOfOneInode,
 };
 use crate::unit::{data_unit_payload_capacity, unit_filesystem_identifier, UNIT_CLASS_DATA};
 use crate::unlink::DeletionSwitches;
@@ -476,6 +477,9 @@ pub enum BackgroundCompactionNotStarted {
     /// 上一次用户改动给的那一份 `B_gc` 已经扣完（岔路单第 18 行甲：定时兜底与用户改动共用这一份）：等下一次用户改动才再有。
     /// 「该跑了」也取走、不留：下一次用户改动发成之后起跑判据再记。
     PublishBudgetOfTheLastUserChangeSpent,
+    /// 水位越线，而上一趟不发布地停在这一条（[`CompactionRunStop::publishes_nothing_until_the_next_publish`]）之后现行那一版没变、开关没变：不重判，
+    /// 一个盘都不读。任何一次发布都让 txg 变、这条备忘失效。
+    UnchangedSinceAStopThatPublishedNothing { stop: CompactionRunStop },
 }
 
 /// 后台入口这一趟为什么起跑。封闭集合。
@@ -492,11 +496,12 @@ pub enum BackgroundCompactionTrigger {
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 enum BackgroundCompactionStart {
     NotStarted(BackgroundCompactionNotStarted),
-    /// 起跑：为什么起跑、这一趟推到哪、预算按哪一条算。
+    /// 起跑：为什么起跑、这一趟推到哪、预算按哪一条算；放锁的后台入口接着推没完的一趟时，这一趟前几次调用已发了几批。
     Starts {
         trigger: BackgroundCompactionTrigger,
         reach: CompactionRunReach,
         budget: CompactionRunBudget,
+        batches_published_earlier_in_the_trip: u64,
     },
 }
 
@@ -509,6 +514,19 @@ enum CompactionRunBudget {
     /// 后台入口的一趟（用户改动之后那一趟与定时兜底那几趟）：上一次用户改动给的那一份 `B_gc` 还剩的，没有至少 1，扣完是 0
     /// （[`publishes_left_of_the_budget_given_by_the_last_user_change`]；岔路单 `research/prompts/m4-gpugc-design-r1-forks.md` 第 18 行甲）。
     LeftOfTheBudgetGivenByTheLastUserChange,
+    /// 同上，只差全空段低于底额、前台写不进时破例（D26（后台整理与放置回收） 已定项 13 第 5 条的加门）：这一趟至少发 1 次，第一次发布不判回退门。
+    /// 前台等整理腾段，整理不能再等前台的用户改动给预算。
+    AtLeastOnePublishWhileTheForegroundCannotWrite,
+}
+
+/// 后台入口一次调用推多少。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum SpanOfTheBackgroundCall {
+    /// 一次调用把一趟推到停（[`MountedSession::run_compaction_in_the_background`]）。
+    WholeTrip,
+    /// 至多一次发布就返回（[`MountedSessionWithPendingChanges::run_compaction_in_the_background_for_at_most_one_publish`]）：
+    /// 调用方放锁之后接着调，这一趟记在会话上，不重判起跑。先走空的删除续做与一趟之后的 discard 切不开。
+    AtMostOnePublish,
 }
 
 /// 一趟整理每一批之前看不看调用方（D26（后台整理与放置回收） 已定项 13 第 6 条）。封闭集合：显式入口与可写挂载续做不看，后台入口看。
@@ -518,6 +536,8 @@ enum PacingOfTheRun<'caller> {
     PacedByTheCaller {
         write_bandwidth_limit: CompactionWriteBandwidthLimit,
         caller: &'caller dyn CallerOfTheBackgroundCompaction,
+        span: SpanOfTheBackgroundCall,
+        batches_published_earlier_in_the_trip: u64,
     },
 }
 
@@ -2852,13 +2872,15 @@ impl MountedSession {
         // 调用线程（用户 2026-10-26 定「挪出调用线程」；D24（后台重活能不能卸给 GPU） 已定项 2 第 2 条「不许进前台路径」）：这里只把「该跑了」
         // 记在会话上，一批都不发，由后台入口 [`Self::run_compaction_in_the_background`] 在调用方自己的线程上去跑。这一次调用拿着 `&mut self`
         // 从头到尾，为这次改动推的抬 F 与之后的再判之间插不进整理批（第三轮判决第二节 G8 的第三样保证）。
-        compaction.publishes_since_the_last_user_change = 0;
+        compaction.note_a_user_change_published();
         // 改动发成：清掉会话上跨趟留着的上一条完成的意图（用户 2026-10-26 定「每次用户改动发成就清掉」，
         // `records/2026-10-19-里程碑四GPU垃圾回收开工.md`「调查员交回」那一行）。D26（后台整理与放置回收） 已定项 13 第 6 条
         // 「签下一条意图那一判跨趟留着」只留到下一次用户改动发成为止：池子一直被写满时全空段数只降不升，留到回滚或重挂会让整理一直停着；
         // 下一趟从空起照常签新意图，预算照旧是这一次改动给的那一份 `B_gc`（岔路单第 18 行甲）。被拒、没发成的改动在上面 `?` 交回，这一条照旧留着。
         compaction.last_completed_intent = None;
-        if compaction_starts_by_the_watermarks(allocator.compaction().switches.run_forcing) {
+        let watermark_verdict = compaction_watermark_verdict_of_the_allocator(allocator);
+        compaction.last_watermark_verdict = Some(watermark_verdict);
+        if watermark_verdict.starts() {
             compaction.start_judgement = CompactionStartJudgement::DueAfterAUserChange;
         }
         published.discards_after_the_change = discard_the_segments_reusable_as_a_whole(
@@ -2875,7 +2897,6 @@ impl MountedSession {
     /// R 里有隔离槽就发一次删意图的发布丢掉它（攻方 K-c）、计划一批、发这一批，最后一批同一次发布删意图（D8（核心索引结构） 已定项 5）；
     /// 全局差值为假就不签下一条。这一趟至多发 `B_gc` 次整理发布（[`publish_budget_of_a_compaction_run`]，签发、搬一批、删意图各算一次）；
     /// 一次整理发布被空间拒就暂停（撤 R 排他、意图留在盘上）、不推抬 F。发布都返回之后发 discard（判决 G5）。
-    /// 批大小条款没写，产品路径在签任何意图之前停下（`CompactionRunStop::BatchSizeNotDecidedByTheClauses`），只供测试的开关给出批大小才推得动。
     ///
     /// # Errors
     /// 盘表与挂载核过的不同、现行那一版树表 0 条 ⇒ [`CompactionRunRefused`]，在任何读写之前。
@@ -3064,7 +3085,11 @@ impl MountedSession {
     /// 4. 用户改动之后那一趟与定时兜底那一趟都从上一次用户改动给的那一份 `B_gc` 里扣（岔路单 `research/prompts/m4-gpugc-design-r1-forks.md` 第 18 行甲，
     ///    用户 2026-10-26 定「与用户改动共用 `B_gc`」；[`publishes_left_of_the_budget_given_by_the_last_user_change`]）：那一份扣完就不起跑
     ///    （[`BackgroundCompactionNotStarted::PublishBudgetOfTheLastUserChangeSpent`]），等下一次用户改动才再有。闲置时整理发布因此合计不过
-    ///    一份 `B_gc`，不随定时兜底查了几次涨。
+    ///    一份 `B_gc`，不随定时兜底查了几次涨。全空段低于底额、前台写不进时破例：那一份扣完也起跑、这一趟至少发 1 次
+    ///    （[`CompactionRunBudget::AtLeastOnePublishWhileTheForegroundCannotWrite`]）；
+    /// 5. 开关照水位判（[`CompactionRunForcing::FollowTheWatermarks`]）时另有三样：头一次判起跑把挂载那一份预算从根环读（挂载不给新预算）；上一趟不发布地停下之后
+    ///    现行那一版与开关都没变就不重判（[`BackgroundCompactionNotStarted::UnchangedSinceAStopThatPublishedNothing`]；全空段低于底额时停在回退门上的那一次不算）；每一次整理发布之前判回退门
+    ///    （[`CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates`]）。
     ///
     /// 起跑之后每一批之前问调用方：前台有 fsync 在等就不发下一批（[`CompactionRunStop::YieldedToAForegroundFsync`]），这一个写带宽窗口里
     /// 整理写的字节到了 `pacing.write_bandwidth_limit` 也不发（[`CompactionRunStop::WriteBandwidthLimitReached`]）；用户改动之后起跑的那一趟
@@ -3088,8 +3113,16 @@ impl MountedSession {
                 trigger,
                 reach,
                 budget,
+                batches_published_earlier_in_the_trip,
             } => self.run_the_background_compaction_that_starts(
-                devices, pacing, caller, trigger, reach, budget,
+                devices,
+                pacing,
+                caller,
+                trigger,
+                reach,
+                budget,
+                SpanOfTheBackgroundCall::WholeTrip,
+                batches_published_earlier_in_the_trip,
             ),
         };
         // 用户 2026-10-30 定：整理停着期间后台入口每一次判起跑都报警，起没起跑、补没补跑都报。
@@ -3097,6 +3130,35 @@ impl MountedSession {
         Ok(call.with_the_alarm_at_the_end_of_the_wake(alarm))
     }
 
+    /// 把分配器上冻结着的那一次发布原样重发（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」，[`resend_the_frozen_publish`]）：一批整理或一次用户改动写盘报错之后
+    /// 发布路径冻结，前台的写都被拒，靠这一个入口解。重发成了：`current` 换成那一版、分配器换成那次发布成立之后的那一份，活意图的 R 排他按那一版的 logged_ops 重装
+    /// （冻结的那一批可能正是删意图的那一批）。没有冻结着的发布交回 `false`，什么都不写。整理的发布计数不补那一次（差一次，只让这一份预算多发一次）。
+    ///
+    /// # Errors
+    /// 重发又有一步报错（`PublishError::BlockDevice`）：照旧冻结着。
+    pub fn resend_the_frozen_publish<Device: BlockDevice>(
+        &mut self,
+        devices: &mut Vec<(DeviceIdentity, Device)>,
+    ) -> Result<bool, PublishError> {
+        let parameters = self.parameters_and_device_table.parameters_on_disk();
+        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
+        let Some(version) = resend_the_frozen_publish(&mut writer, &mut self.allocator)? else {
+            return Ok(false);
+        };
+        self.current = version;
+        let live_intent_of_the_version =
+            self.current
+                .version_of_trees()
+                .and_then(|version_of_trees| {
+                    live_compaction_intent_of_the_entries(&version_of_trees.logged_operations)
+                });
+        self.allocator.compaction_mut().withdraw_the_live_intent();
+        if let Some((_, intent)) = live_intent_of_the_version {
+            self.allocator.compaction_mut().issue(intent);
+        }
+        Ok(true)
+    }
+
     /// [`Self::run_compaction_in_the_background`] 开头那几判：盘表、有没有树、整理停没停、有没有「该跑了」、定时兜底、上一次用户改动给的那一份
     /// `B_gc` 还剩不剩。取走「该跑了」与定时兜底记下这一次查的读数都在这里做，一个盘都不读、一个写都不发。攒批的会话
     /// （[`MountedSessionWithPendingChanges::run_compaction_in_the_background`]）在这一判说起跑之后、跑之前先排空待发。
@@ -3144,6 +3206,51 @@ impl MountedSession {
                 BackgroundCompactionNotStarted::BackgroundTaskStoppedAfterAComputeError,
             ));
         }
+        let is_judged_as_the_product_does = matches!(
+            allocator.compaction().switches.run_forcing,
+            CompactionRunForcing::FollowTheWatermarks
+        );
+        // 挂载不给新预算（D26（后台整理与放置回收） 已定项 13 第 5 条的加门）：这次挂载头一次判后台起跑时，已经过去的发布数取根环里最新那个用户状态之后的根数
+        // （写行、暖机、上一次挂载的整理都在里面）。读不通当那一份已扣完。
+        if is_judged_as_the_product_does
+            && compaction.take_the_reading_of_the_publishes_of_the_mount_from_the_ring()
+        {
+            let publishes_read_from_the_ring = rollback_states_of_the_ring(
+                &PoolReaderOfOneReadPass::reading_for_the_first_time(devices),
+                parameters,
+                current.root(),
+            )
+            .map_or(
+                publish_budget_of_a_compaction_run(
+                    parameters.geometry.root_ring_slots_per_region.count(),
+                    0,
+                ),
+                |states| states.publishes_after_the_newest_state,
+            );
+            compaction.publishes_since_the_last_user_change = compaction
+                .publishes_since_the_last_user_change
+                .max(publishes_read_from_the_ring);
+        }
+        let foreground_cannot_write = is_judged_as_the_product_does
+            && foreground_writes_are_below_the_empty_cluster_segments_floor(allocator);
+        // 放锁的后台入口上一次调用之后还没完的那一趟接着推：一趟只判一次起跑。整理停着的那一判在上面：停着时这一趟留着，解除之后接着推。
+        if let Some(trip) = compaction.take_the_background_trip_under_way() {
+            if publishes_left_of_the_budget_given_by_the_last_user_change(
+                parameters.geometry.root_ring_slots_per_region.count(),
+                compaction.publishes_since_the_last_user_change,
+            ) == 0
+            {
+                return Ok(BackgroundCompactionStart::NotStarted(
+                    BackgroundCompactionNotStarted::PublishBudgetOfTheLastUserChangeSpent,
+                ));
+            }
+            return Ok(BackgroundCompactionStart::Starts {
+                trigger: trip.trigger,
+                reach: trip.reach,
+                budget: CompactionRunBudget::LeftOfTheBudgetGivenByTheLastUserChange,
+                batches_published_earlier_in_the_trip: trip.batches_published,
+            });
+        }
         let (trigger, reach) = match compaction.start_judgement {
             CompactionStartJudgement::DueAfterAUserChange => {
                 compaction.start_judgement = CompactionStartJudgement::NotDue;
@@ -3168,23 +3275,47 @@ impl MountedSession {
                     }
                     TimerFallbackCheck::Due => {}
                 }
-                if !compaction_starts_by_the_watermarks(allocator.compaction().switches.run_forcing)
-                {
+                let watermark_verdict = compaction_watermark_verdict_of_the_allocator(allocator);
+                compaction.last_watermark_verdict = Some(watermark_verdict);
+                if !watermark_verdict.starts() {
                     return Ok(BackgroundCompactionStart::NotStarted(
                         BackgroundCompactionNotStarted::NotDueByTheWatermarksAtTheTimerFallback,
                     ));
                 }
+                if is_judged_as_the_product_does {
+                    // 全空段低于底额、前台写不进时每趟第一次发布不判门：门关着的备忘挡不住这一趟。
+                    if let Some(stop) = compaction
+                        .stop_still_holding_at(
+                            current.root().checkpoint_txg,
+                            allocator.compaction().switches,
+                        )
+                        .filter(|stop| {
+                            !(foreground_cannot_write
+                                && matches!(
+                                    stop,
+                                    CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates { .. }
+                                ))
+                        })
+                    {
+                        return Ok(BackgroundCompactionStart::NotStarted(
+                            BackgroundCompactionNotStarted::UnchangedSinceAStopThatPublishedNothing {
+                                stop,
+                            },
+                        ));
+                    }
+                }
                 (
                     BackgroundCompactionTrigger::TimerFallback,
                     CompactionRunReach::UntilAStopOrTheBudget,
                 )
             }
         };
-        // 两种起跑共用上一次用户改动给的那一份 `B_gc`：扣完就不起跑，等下一次用户改动。
+        // 两种起跑共用上一次用户改动给的那一份 `B_gc`：扣完就不起跑，等下一次用户改动。全空段低于底额、前台写不进时破例。
         if publishes_left_of_the_budget_given_by_the_last_user_change(
             parameters.geometry.root_ring_slots_per_region.count(),
             compaction.publishes_since_the_last_user_change,
         ) == 0
+            && !foreground_cannot_write
         {
             return Ok(BackgroundCompactionStart::NotStarted(
                 BackgroundCompactionNotStarted::PublishBudgetOfTheLastUserChangeSpent,
@@ -3193,11 +3324,21 @@ impl MountedSession {
         Ok(BackgroundCompactionStart::Starts {
             trigger,
             reach,
-            budget: CompactionRunBudget::LeftOfTheBudgetGivenByTheLastUserChange,
+            budget: if foreground_cannot_write {
+                CompactionRunBudget::AtLeastOnePublishWhileTheForegroundCannotWrite
+            } else {
+                CompactionRunBudget::LeftOfTheBudgetGivenByTheLastUserChange
+            },
+            batches_published_earlier_in_the_trip: 0,
         })
     }
 
     /// [`Self::run_compaction_in_the_background`] 判过起跑之后那一段：推一趟（每一批之前问调用方）、节奏截住时把「该跑了」放回去、发 discard。
+    /// `span` 是至多一次发布时，这一趟没完（发了一次、或被节奏截住）就记在会话上，下一次调用接着推。
+    #[allow(
+        clippy::too_many_arguments,
+        reason = "参数各是一样东西：盘表、节奏、调用方、为什么起跑、推到哪、预算按哪一条算、这次调用推多少、这一趟前几次调用发的批数"
+    )]
     fn run_the_background_compaction_that_starts<Device: BlockDevice>(
         &mut self,
         devices: &mut Vec<(DeviceIdentity, Device)>,
@@ -3206,6 +3347,8 @@ impl MountedSession {
         trigger: BackgroundCompactionTrigger,
         reach: CompactionRunReach,
         budget: CompactionRunBudget,
+        span: SpanOfTheBackgroundCall,
+        batches_published_earlier_in_the_trip: u64,
     ) -> BackgroundCompactionCall {
         let MountedSession {
             allocator,
@@ -3229,6 +3372,8 @@ impl MountedSession {
             PacingOfTheRun::PacedByTheCaller {
                 write_bandwidth_limit: pacing.write_bandwidth_limit,
                 caller,
+                span,
+                batches_published_earlier_in_the_trip,
             },
             RegionCheckOfTheIntentDeletion::JudgedByTheRecords,
             background_compute,
@@ -3242,12 +3387,32 @@ impl MountedSession {
                     | CompactionRunStop::WriteBandwidthLimitReached { .. }
             )
         );
-        match trigger {
-            BackgroundCompactionTrigger::DueAfterAUserChange if is_held_back_by_the_pacing => {
-                compaction.start_judgement = CompactionStartJudgement::DueAfterAUserChange;
+        let is_done_with_this_call_only = matches!(
+            run.outcome,
+            CompactionRunOutcome::Stopped(CompactionRunStop::OnePublishOfTheTripDone)
+        );
+        match span {
+            SpanOfTheBackgroundCall::AtMostOnePublish
+                if is_held_back_by_the_pacing || is_done_with_this_call_only =>
+            {
+                compaction.keep_the_background_trip_under_way(BackgroundTripUnderWay {
+                    trigger,
+                    reach,
+                    batches_published: batches_published_earlier_in_the_trip
+                        + run.batches_published,
+                });
+            }
+            SpanOfTheBackgroundCall::AtMostOnePublish | SpanOfTheBackgroundCall::WholeTrip => {
+                match trigger {
+                    BackgroundCompactionTrigger::DueAfterAUserChange
+                        if is_held_back_by_the_pacing =>
+                    {
+                        compaction.start_judgement = CompactionStartJudgement::DueAfterAUserChange;
+                    }
+                    BackgroundCompactionTrigger::DueAfterAUserChange
+                    | BackgroundCompactionTrigger::TimerFallback => {}
+                }
             }
-            BackgroundCompactionTrigger::DueAfterAUserChange
-            | BackgroundCompactionTrigger::TimerFallback => {}
         }
         run.discards = discard_the_segments_reusable_as_a_whole(
             devices.as_mut_slice(),
@@ -3921,6 +4086,37 @@ pub(crate) fn run_compaction_on<Device: BlockDevice>(
     )
 }
 
+/// 回退门（D16（发布语义） 已定项 1「回退候选集保留最近 4 个可退到的不同状态」，D26（后台整理与放置回收） 已定项 13 第 5 条的加门）：
+/// 下一次发布写的那一槽与它之后留的余量（[`crate::compaction_plan::ROOT_RING_SLOTS_LEFT_BY_A_COMPACTION_PUBLISH`]）里的根一走，根环里不同用户状态就少于 4 个而且比此刻少，这次整理发布就不发。
+/// 此刻本来就不到 4 个、走的那几条又都不是某个状态的最后一条的，不是这次发布挤掉的，照发。读不通不发（[`CompactionRunStop::RollbackStatesOfTheRingUnreadable`]）。
+/// 只读，读根环与每条有效根的树表。
+fn stop_of_the_rollback_gate<Device: BlockDevice>(
+    parameters: &MakeFilesystemParameters,
+    devices: &[(DeviceIdentity, Device)],
+    current: &PoolVersion,
+) -> Option<CompactionRunStop> {
+    match rollback_states_of_the_ring(
+        &PoolReaderOfOneReadPass::reading_for_the_first_time(devices),
+        parameters,
+        current.root(),
+    ) {
+        Err(_unreadable) => Some(CompactionRunStop::RollbackStatesOfTheRingUnreadable),
+        Ok(states) => (states.states_after_the_next_publish_and_the_margin
+            < ROLLBACK_STATES_KEPT_BY_THE_COMPACTION
+            && states.states_after_the_next_publish_and_the_margin < states.states_now)
+            .then_some(
+                CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates {
+                    states_now: states.states_now,
+                    states_after_the_publish_and_the_margin: states
+                        .states_after_the_next_publish_and_the_margin,
+                },
+            ),
+    }
+}
+
+/// 回退门保住的不同用户状态个数（D16（发布语义） 已定项 1）。
+const ROLLBACK_STATES_KEPT_BY_THE_COMPACTION: u64 = 4;
+
 /// 同 [`run_compaction_on`]，另带三样：这一趟的发布预算按 `budget_rule` 算；每一批之前照 `pacing` 问调用方（后台入口：前台有 fsync 在等、
 /// 写带宽到了上限就不发下一批，D26（后台整理与放置回收） 已定项 13 第 6 条）；一批腾空了 R、照同一批删意图再发时删意图那一支的核照 `region_check` 判
 /// （[`MountedSession::run_compaction_with_the_region_check_of_the_intent_deletion_after_a_batch_emptying_the_region`]），
@@ -3948,13 +4144,20 @@ fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockD
     compaction.observations.runs_started += 1;
     let mut versions_published: Vec<TransactionOutput> = Vec::new();
     let mut background_compute_of_each_batch: Vec<BackgroundComputeOfOneBatch> = Vec::new();
+    let switches_of_the_run: CompactionSwitches = allocator.compaction().switches;
     let finish = |compaction_of_the_run: &mut CompactionOfThisMount,
+                  current_txg_of_the_run: CheckpointTxg,
                   versions_published_by_the_run: Vec<TransactionOutput>,
                   background_compute_of_each_batch_of_the_run: Vec<BackgroundComputeOfOneBatch>,
                   batches_published: u64,
                   outcome: CompactionRunOutcome| {
         if let CompactionRunOutcome::Stopped(stop) = &outcome {
             compaction_of_the_run.observations.last_stop = Some(*stop);
+            compaction_of_the_run.note_the_stop_of_a_run(
+                *stop,
+                current_txg_of_the_run,
+                switches_of_the_run,
+            );
         }
         CompactionRun {
             versions_published: versions_published_by_the_run,
@@ -3970,6 +4173,7 @@ fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockD
         CompactionRunForcing::ForcedToStopForTests => {
             return finish(
                 compaction,
+                current.root().checkpoint_txg,
                 versions_published,
                 background_compute_of_each_batch,
                 0,
@@ -3986,6 +4190,7 @@ fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockD
     if compaction_stays_stopped_pausing_the_live_intent(allocator, background_compute) {
         return finish(
             compaction,
+            current.root().checkpoint_txg,
             versions_published,
             background_compute_of_each_batch,
             0,
@@ -3995,16 +4200,8 @@ fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockD
         );
     }
     let batch_units = match switches.batch_size {
-        CompactionBatchSize::NotDecidedByTheClauses => {
-            return finish(
-                compaction,
-                versions_published,
-                background_compute_of_each_batch,
-                0,
-                CompactionRunOutcome::Stopped(CompactionRunStop::BatchSizeNotDecidedByTheClauses),
-            );
-        }
-        CompactionBatchSize::ForcedForTests(batch_units) => batch_units,
+        CompactionBatchSize::OfTheMount(batch_units)
+        | CompactionBatchSize::ForcedForTests(batch_units) => batch_units,
     };
     // 次序（判决 G8）：删除续做在整理批之前。
     let queue_is_empty = match &*current {
@@ -4033,6 +4230,7 @@ fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockD
         if let Err(cause) = walk.stopped_by {
             return finish(
                 compaction,
+                current.root().checkpoint_txg,
                 versions_published,
                 background_compute_of_each_batch,
                 0,
@@ -4051,7 +4249,28 @@ fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockD
                 compaction.publishes_since_the_last_user_change,
             )
         }
+        CompactionRunBudget::AtLeastOnePublishWhileTheForegroundCannotWrite => {
+            publishes_left_of_the_budget_given_by_the_last_user_change(
+                parameters.geometry.root_ring_slots_per_region.count(),
+                compaction.publishes_since_the_last_user_change,
+            )
+            .max(1)
+        }
     };
+    let (span_of_the_call, batches_published_earlier_in_the_trip) = match pacing {
+        PacingOfTheRun::NotPaced => (SpanOfTheBackgroundCall::WholeTrip, 0),
+        PacingOfTheRun::PacedByTheCaller {
+            span,
+            batches_published_earlier_in_the_trip,
+            ..
+        } => (span, batches_published_earlier_in_the_trip),
+    };
+    // 回退门只在后台入口、开关照水位判时判（显式入口没有产品调用方，可写挂载续做只推盘上那一条，发布数有上界）。
+    let is_the_rollback_gate_judged = matches!(pacing, PacingOfTheRun::PacedByTheCaller { .. })
+        && matches!(
+            switches.run_forcing,
+            CompactionRunForcing::FollowTheWatermarks
+        );
     let mut publishes_of_the_run: u64 = 0;
     let mut batches_published: u64 = 0;
     // 循环：每一轮发成一次整理发布或停。迭代上界是预算（每发成一次计一次，到 budget 停）；跨轮携带的是现行那一版、已发的次数与批数。
@@ -4060,32 +4279,50 @@ fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockD
         if publishes_of_the_run >= budget {
             return finish(
                 compaction,
+                current.root().checkpoint_txg,
                 versions_published,
                 background_compute_of_each_batch,
                 batches_published,
                 CompactionRunOutcome::Stopped(CompactionRunStop::PublishBudgetSpent { budget }),
             );
         }
-        if reach == CompactionRunReach::AtMostOneBatchAfterAUserChange && batches_published == 1 {
+        if reach == CompactionRunReach::AtMostOneBatchAfterAUserChange
+            && batches_published + batches_published_earlier_in_the_trip >= 1
+        {
             return finish(
                 compaction,
+                current.root().checkpoint_txg,
                 versions_published,
                 background_compute_of_each_batch,
                 batches_published,
                 CompactionRunOutcome::Stopped(CompactionRunStop::OneBatchAfterTheUserChangeDone),
             );
         }
+        if span_of_the_call == SpanOfTheBackgroundCall::AtMostOnePublish
+            && publishes_of_the_run == 1
+        {
+            return finish(
+                compaction,
+                current.root().checkpoint_txg,
+                versions_published,
+                background_compute_of_each_batch,
+                batches_published,
+                CompactionRunOutcome::Stopped(CompactionRunStop::OnePublishOfTheTripDone),
+            );
+        }
         // D26（后台整理与放置回收） 已定项 13 第 6 条：后台入口每一批之前问调用方，前台有 fsync 在等、写带宽到了上限就不发下一批。
         let write_bandwidth_limit = match pacing {
             PacingOfTheRun::NotPaced => CompactionWriteBandwidthLimit::Unlimited,
             PacingOfTheRun::PacedByTheCaller {
                 write_bandwidth_limit,
                 caller,
+                ..
             } => {
                 match caller.foreground_fsync() {
                     ForegroundFsync::Waiting => {
                         return finish(
                             compaction,
+                            current.root().checkpoint_txg,
                             versions_published,
                             background_compute_of_each_batch,
                             batches_published,
@@ -4102,6 +4339,7 @@ fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockD
                 ) {
                     return finish(
                         compaction,
+                        current.root().checkpoint_txg,
                         versions_published,
                         background_compute_of_each_batch,
                         batches_published,
@@ -4111,6 +4349,22 @@ fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockD
                 write_bandwidth_limit
             }
         };
+        // 全空段低于底额、前台写不进时这一趟的第一次发布破例不判（前台等它腾段）；别的整理发布判过才发。
+        let is_this_publish_the_exception = budget_rule
+            == CompactionRunBudget::AtLeastOnePublishWhileTheForegroundCannotWrite
+            && publishes_of_the_run == 0;
+        if is_the_rollback_gate_judged && !is_this_publish_the_exception {
+            if let Some(stop) = stop_of_the_rollback_gate(parameters, &*devices, current) {
+                return finish(
+                    compaction,
+                    current.root().checkpoint_txg,
+                    versions_published,
+                    background_compute_of_each_batch,
+                    batches_published,
+                    CompactionRunOutcome::Stopped(stop),
+                );
+            }
+        }
         let CompactionPublishAdvanced {
             step,
             background_compute_of_the_batch,
@@ -4163,6 +4417,7 @@ fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockD
                 if kind == CompactionPublishKind::IntentDroppedWithNothingMovable {
                     return finish(
                         compaction,
+                        current.root().checkpoint_txg,
                         versions_published,
                         background_compute_of_each_batch,
                         batches_published,
@@ -4175,6 +4430,7 @@ fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockD
             Ok(CompactionStep::Stopped(stop)) => {
                 return finish(
                     compaction,
+                    current.root().checkpoint_txg,
                     versions_published,
                     background_compute_of_each_batch,
                     batches_published,
@@ -4184,6 +4440,7 @@ fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockD
             Err(refused) => {
                 return finish(
                     compaction,
+                    current.root().checkpoint_txg,
                     versions_published,
                     background_compute_of_each_batch,
                     batches_published,
@@ -6271,10 +6528,100 @@ impl MountedSessionWithPendingChanges {
     ) -> Result<
         BackgroundCompactionAfterThePendingChanges,
         BackgroundCompactionAfterThePendingChangesRefused,
+    > {
+        self.run_compaction_in_the_background_over_a_span(
+            devices,
+            pacing,
+            caller,
+            SpanOfTheBackgroundCall::WholeTrip,
+        )
+    }
+
+    /// 同 [`Self::run_compaction_in_the_background`]，只差至多一次发布就返回（D26（后台整理与放置回收） 已定项 13 第 5 条：前台与后台经一把锁，每次发布之间放锁）：
+    /// 这一趟没完时停因是 [`CompactionRunStop::OnePublishOfTheTripDone`]（或节奏截住），记在会话上，下一次调用不重判起跑、接着推。
+    /// 两次调用之间前台收下的待发在下一次调用先排空。挂着的会话这一层不起线程、不持锁，放锁是调用方的事（运行库 `singlefs_mount_runtime`）。
+    ///
+    /// # Errors
+    /// 同 [`Self::run_compaction_in_the_background`]。
+    pub fn run_compaction_in_the_background_for_at_most_one_publish<Device: BlockDevice>(
+        &mut self,
+        devices: &mut Vec<(DeviceIdentity, Device)>,
+        pacing: CompactionPacing,
+        caller: &dyn CallerOfTheBackgroundCompaction,
+    ) -> Result<
+        BackgroundCompactionAfterThePendingChanges,
+        BackgroundCompactionAfterThePendingChangesRefused,
+    > {
+        self.run_compaction_in_the_background_over_a_span(
+            devices,
+            pacing,
+            caller,
+            SpanOfTheBackgroundCall::AtMostOnePublish,
+        )
+    }
+
+    /// 挂着切换后台计算的设置（[`MountedSession::switch_background_compute_from_the_next_batch`]）：只记成待生效、不读写盘，所以不排空待发。
+    ///
+    /// # Errors
+    /// 同 [`MountedSession::switch_background_compute_from_the_next_batch`]。
+    pub fn switch_background_compute_from_the_next_batch(
+        &mut self,
+        background_compute_settings: BackgroundComputeSettings,
+        gpu_compute: Option<BackgroundGpuComputeHandle>,
+    ) -> Result<(), BackgroundComputeSettingsRefused> {
+        self.session
+            .switch_background_compute_from_the_next_batch(background_compute_settings, gpu_compute)
+    }
+
+    /// 同 [`Self::switch_background_compute_from_the_next_batch`]，连同 GC 停着时在 CPU 上补跑的阈值一起换
+    /// （[`MountedSession::switch_background_compute_with_the_gc_cpu_compensation_threshold_from_the_next_batch`]）。
+    ///
+    /// # Errors
+    /// 同 [`MountedSession::switch_background_compute_with_the_gc_cpu_compensation_threshold_from_the_next_batch`]。
+    pub fn switch_background_compute_with_the_gc_cpu_compensation_threshold_from_the_next_batch(
+        &mut self,
+        background_compute_settings: BackgroundComputeSettings,
+        gpu_compute: Option<BackgroundGpuComputeHandle>,
+        gc_cpu_compensation_threshold: BackgroundGcCpuCompensationThreshold,
+    ) -> Result<(), BackgroundComputeSettingsRefused> {
+        self.session
+            .switch_background_compute_with_the_gc_cpu_compensation_threshold_from_the_next_batch(
+                background_compute_settings,
+                gpu_compute,
+                gc_cpu_compensation_threshold,
+            )
+    }
+
+    /// 重发冻结着的那一次发布（[`MountedSession::resend_the_frozen_publish`]）。没有冻结着的发布交回 `false`，什么都不写。
+    ///
+    /// # Errors
+    /// 同 [`MountedSession::resend_the_frozen_publish`]。
+    pub fn resend_the_frozen_publish<Device: BlockDevice>(
+        &mut self,
+        devices: &mut Vec<(DeviceIdentity, Device)>,
+    ) -> Result<bool, PublishError> {
+        self.session.resend_the_frozen_publish(devices)
+    }
+
+    /// 只供测试（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：底下那个会话（可改），用例摆开关与预算。产品路径一处都不调它：待发与会话私有的不变量靠调用方不改待发。
+    pub fn session_for_tests_only(&mut self) -> &mut MountedSession {
+        &mut self.session
+    }
+
+    /// 两种跨度共用的那一段。
+    fn run_compaction_in_the_background_over_a_span<Device: BlockDevice>(
+        &mut self,
+        devices: &mut Vec<(DeviceIdentity, Device)>,
+        pacing: CompactionPacing,
+        caller: &dyn CallerOfTheBackgroundCompaction,
+        span: SpanOfTheBackgroundCall,
+    ) -> Result<
+        BackgroundCompactionAfterThePendingChanges,
+        BackgroundCompactionAfterThePendingChangesRefused,
     > {
         let now = caller.clock_reading();
         self.note_the_reading_seen(now);
-        let (trigger, reach, budget) = match self
+        let (trigger, reach, budget, batches_published_earlier_in_the_trip) = match self
             .session
             .whether_the_background_compaction_starts(devices, pacing, caller)
             .map_err(BackgroundCompactionAfterThePendingChangesRefused::CompactionRefused)?
@@ -6293,7 +6640,13 @@ impl MountedSessionWithPendingChanges {
                 trigger,
                 reach,
                 budget,
-            } => (trigger, reach, budget),
+                batches_published_earlier_in_the_trip,
+            } => (
+                trigger,
+                reach,
+                budget,
+                batches_published_earlier_in_the_trip,
+            ),
         };
         let pending_changes_published_before_the_run = match self.publish_the_pending_changes(
             devices,
@@ -6302,12 +6655,23 @@ impl MountedSessionWithPendingChanges {
         ) {
             Ok(drained) => drained,
             Err(refusal) => {
-                match trigger {
-                    BackgroundCompactionTrigger::DueAfterAUserChange => {
-                        self.session.compaction.start_judgement =
-                            CompactionStartJudgement::DueAfterAUserChange;
+                match span {
+                    SpanOfTheBackgroundCall::AtMostOnePublish => {
+                        self.session.compaction.keep_the_background_trip_under_way(
+                            BackgroundTripUnderWay {
+                                trigger,
+                                reach,
+                                batches_published: batches_published_earlier_in_the_trip,
+                            },
+                        );
                     }
-                    BackgroundCompactionTrigger::TimerFallback => {}
+                    SpanOfTheBackgroundCall::WholeTrip => match trigger {
+                        BackgroundCompactionTrigger::DueAfterAUserChange => {
+                            self.session.compaction.start_judgement =
+                                CompactionStartJudgement::DueAfterAUserChange;
+                        }
+                        BackgroundCompactionTrigger::TimerFallback => {}
+                    },
                 }
                 // 用户 2026-10-30 定每次叫起都报：判过起跑之后排空被拒的这一次也是一次叫起，照数、照报。
                 let alarm_of_the_wake_refused_at_the_drain =
@@ -6322,7 +6686,14 @@ impl MountedSessionWithPendingChanges {
             }
         };
         let call = self.session.run_the_background_compaction_that_starts(
-            devices, pacing, caller, trigger, reach, budget,
+            devices,
+            pacing,
+            caller,
+            trigger,
+            reach,
+            budget,
+            span,
+            batches_published_earlier_in_the_trip,
         );
         let has_published = match &call {
             BackgroundCompactionCall::Ran { run, .. } => !run.versions_published.is_empty(),
diff --git a/crates/singlefs-core/src/system_configuration.rs b/crates/singlefs-core/src/system_configuration.rs
index 61953e8c13639a3b7f9709e09aaf68cb4a68d678..7159d79d112e7572467b4a673fa9debc7c1ca24b 100644
--- a/crates/singlefs-core/src/system_configuration.rs
+++ b/crates/singlefs-core/src/system_configuration.rs
@@ -98,10 +98,11 @@ const GROUP_SIZE: u8 = 4;
 /// [200 ms, 5 s] 下端在「秒」里写不出），默认 5 s 写 5000。
 const TIME_THRESHOLD_MILLISECONDS: u32 = 5000;
 const DIRTY_THRESHOLD_BYTES: u64 = 2 << 30;
-/// 整理低水位 / 高水位 / 停止线三条（D26（后台整理与放置回收） 已定项 1）。
-const COMPACTION_WATERMARK_COUNT: usize = 3;
-/// 三条水位第一版恒 0，0 的含义是「用内置默认」（D22（单元原子性怎么合成） 已定项 15 ④）。
-const COMPACTION_WATERMARK_BUILT_IN_DEFAULT: u64 = 0;
+/// 字段表里那 3 个 8 字节的预留位（盘上恒全 0）：整理的三条水位是挂载参数、不存盘（D26（后台整理与放置回收） 已定项 1，
+/// `compaction_plan::CompactionWatermarks`）。
+const RESERVED_RUNTIME_WORD_COUNT: usize = 3;
+/// 预留位盘上的值。
+const RESERVED_RUNTIME_WORD_VALUE: u64 = 0;
 
 /// 在飞记录数上限 = 环槽数 ÷ F，语义是一次恢复重放的前缀最多这么多条（D23（journal 的角色与格式） 已定项 18）。
 /// 环槽数 = 环长 ÷ [`JOURNAL_RECORD_BYTES`]，F = [`JOURNAL_SAFETY_FACTOR`]（I-8.1（环几何够大）），两次都取整数部分。
@@ -339,11 +340,11 @@ impl SystemMutableConfiguration {
 
 /// 系统运行配置：36 字节，改它无代价、每次挂载可重设并回写（D22（单元原子性怎么合成） 已定项 26 第三档）。
 ///
-/// 字段表里归这一档的 3 行：可调值段的「T_time 4」「T_dirty 8」「整理低 / 高 / 停三水位 8 × 3」。
+/// 字段表里归这一档的 3 行：可调值段的「T_time 4」「T_dirty 8」「整理水位预留 8 × 3」。
 /// **没有字段**：挂载选项通道不存在（C422（挂载选项通道不存在）），这三行今天没有可设的入口，只有内置默认值。
 /// T_time 与 T_dirty 的发布触发已经接上：挂载时 [`PublishTriggerThresholds::of_the_mount`] 从这里取两个内置默认值求值一次
 /// （脏量那一支按环长夹），会话里 `mounted_session::publish_trigger_cause` 拿它们比时间与待发的 journal 字节；
-/// 整理三水位恒 0 = 内置默认（D22（单元原子性怎么合成） 已定项 15 ④）。所以这一档拿内置默认值写盘，不带状态。
+/// 整理的三条水位不在这一档：挂载参数、不存盘，这 24 字节是全 0 的预留位。所以这一档拿内置默认值与全 0 写盘，不带状态。
 /// 真有了挂载选项通道，值落在这里，下面三个方法退回去当默认值。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct SystemRuntimeConfiguration;
@@ -364,10 +365,10 @@ impl SystemRuntimeConfiguration {
         DIRTY_THRESHOLD_BYTES
     }
 
-    /// 整理低水位 / 高水位 / 停止线，按字段表的次序（D26（后台整理与放置回收） 已定项 1）。
+    /// 字段表里那 3 个预留位，按字段表的次序。
     #[must_use]
-    pub fn compaction_watermarks(self) -> [u64; COMPACTION_WATERMARK_COUNT] {
-        [COMPACTION_WATERMARK_BUILT_IN_DEFAULT; COMPACTION_WATERMARK_COUNT]
+    pub fn reserved_runtime_words(self) -> [u64; RESERVED_RUNTIME_WORD_COUNT] {
+        [RESERVED_RUNTIME_WORD_VALUE; RESERVED_RUNTIME_WORD_COUNT]
     }
 }
 
@@ -678,8 +679,8 @@ impl SystemConfiguration {
         slot.write(SlotFieldMutability::RuntimeConfiguration, |writer| {
             writer.put_u32(self.runtime.time_threshold_milliseconds());
             writer.put_u64(self.runtime.dirty_threshold_bytes());
-            for watermark in self.runtime.compaction_watermarks() {
-                writer.put_u64(watermark);
+            for reserved_word in self.runtime.reserved_runtime_words() {
+                writer.put_u64(reserved_word);
             }
         });
         slot.write(SlotFieldMutability::RuntimeQuantity, |writer| {
@@ -1148,7 +1149,7 @@ mod tests {
         assert_eq!(
             SystemRuntimeConfiguration::FIELD_TABLE_BYTES,
             36,
-            "系统运行配置：T_time 4 + T_dirty 8 + 整理三水位 24"
+            "系统运行配置：T_time 4 + T_dirty 8 + 预留 24"
         );
         assert_eq!(
             SystemRuntimeQuantities::FIELD_TABLE_BYTES,
diff --git a/crates/singlefs-mount-runtime/Cargo.toml b/crates/singlefs-mount-runtime/Cargo.toml
new file mode 100644
index 0000000000000000000000000000000000000000..392090040c3090fe6b2d186d336df80c9ffb58ad
--- /dev/null
+++ b/crates/singlefs-mount-runtime/Cargo.toml
@@ -0,0 +1,14 @@
+[package]
+name = "singlefs-mount-runtime"
+version = "0.1.0"
+edition = "2021"
+publish = false
+description = "挂载就起的后台运行库（D26（后台整理与放置回收） 已定项 13 第 5 条）：挂载入口起周期线程，前台接口与后台批经一把锁，每次发布之间放锁"
+
+# 只依赖核心层：GPU 经特质递进（OpensTheBackgroundGpu，实现在产品 GPU 包），不开 GPU 的调用方不拖上 wgpu。
+# 不设 [features]：验证包的构建脚本取直接依赖的特性要依赖带 links 与构建脚本，带特性而不带 links 的依赖让它当场停下。
+[dependencies]
+singlefs-core = { path = "../singlefs-core" }
+
+[dev-dependencies]
+singlefs-format = { path = "../singlefs-format" }
diff --git a/crates/singlefs-mount-runtime/src/lib.rs b/crates/singlefs-mount-runtime/src/lib.rs
new file mode 100644
index 0000000000000000000000000000000000000000..e8d85e50d9dab3003332c089392c84d7fa6e5dc4
--- /dev/null
+++ b/crates/singlefs-mount-runtime/src/lib.rs
@@ -0,0 +1,1029 @@
+//! 挂载就起的后台运行库（D26（后台整理与放置回收） 已定项 13 第 5、6 条）：核心层不起线程，这一层在挂载入口起一条周期线程，
+//! 调用方挂载之后不调任何后台入口，水位越线的池上 GC 自己跑。
+//!
+//! 一把锁：攒批的会话连同盘表都在一把锁里（[`MountedRuntime`]），前台入口与后台批在它上面排队。每次发布之间放锁
+//! （`MountedSessionWithPendingChanges::run_compaction_in_the_background_for_at_most_one_publish`）；一批之内 GPU 仍在锁内等（D24（后台重活能不能卸给 GPU） 已定项 2 的射程，里程碑六拆）。
+//! 「前台有 fsync 在等」是锁外的原子计数：核心层问它的那一刻后台线程正拿着锁，答案不能在锁里。
+//!
+//! 线程体之外有同步的 [`MountedRuntime::run_one_maintenance_pass`]：确定性测试不起线程，一拍一拍推，录制盘（不能跨线程）也走得通。
+//! 一个维护线程、两个到期时刻：`publish_poll_period` 到期调 `publish_when_due`（D16（发布语义） 丢失窗口里的 P），`compaction_period` 到期推后台整理。
+//!
+//! 卸载等线程、排空待发、走核心层的卸载；丢弃（Drop）只停线程，不写盘，等价于崩溃（下一次挂载走恢复）。线程或前台在锁里 panic 等价于崩溃：
+//! 会话可能停在半个更新里，不接着用，之后每个入口报 [`RuntimeFailed`]，只能丢掉内存重挂。
+#![deny(unsafe_code)]
+
+use std::any::Any;
+use std::fmt;
+use std::panic::{catch_unwind, AssertUnwindSafe};
+use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
+use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
+use std::thread::JoinHandle;
+use std::time::{Duration, Instant};
+
+use singlefs_core::address::DeviceIdentity;
+use singlefs_core::background_compute::{
+    AlarmOfABackgroundTaskStoppedAfterAComputeError, BackgroundComputeBackend,
+    BackgroundComputeSettings, BackgroundComputeSettingsRefused,
+    BackgroundGcCpuCompensationThreshold, BackgroundGpuComputeHandle,
+    BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT,
+};
+use singlefs_core::block_device::BlockDevice;
+use singlefs_core::compaction_plan::{
+    CallerMonotonicClockReading, CallerOfTheBackgroundCompaction, CompactionMountParameters,
+    CompactionPacing, CompactionRunStop, CompactionTimerFallback, CompactionWriteBandwidthLimit,
+    ForegroundFsync, COMPACTION_WRITE_BANDWIDTH_LIMIT_BY_DEFAULT,
+};
+use singlefs_core::make_filesystem::MakeFilesystemParameters;
+use singlefs_core::mount::{
+    mount_writable_with_background_compute_settings_the_gc_cpu_compensation_threshold_and_compaction_parameters,
+    unmount, MountError, MountOutput, Unmounted, WritableMountWithBackgroundComputeRefused,
+};
+use singlefs_core::mounted_session::{
+    AcceptRefused, BackgroundCompactionAfterThePendingChangesRefused, BackgroundCompactionCall,
+    BackgroundCompactionNotStarted, ChangeAccepted, CompactionRunOutcome, MountedSession,
+    MountedSessionWithPendingChanges, PublishOfThePendingChangesRefused,
+    PublishReportedByTheSession, UserChange, UserChangeAfterThePendingChangesRefused,
+    UserChangeRefused,
+};
+use singlefs_core::transaction::PublishError;
+
+/// 后台整理的周期默认：30 秒（D26（后台整理与放置回收） 已定项 13 第 5 条，挂载参数）。
+pub const COMPACTION_PERIOD_BY_DEFAULT: Duration = Duration::from_secs(30);
+/// `publish_when_due` 的周期默认：1 秒（数是拍的，相对 `T_time` 5 秒加 20% 的丢失窗口；挂载参数）。
+pub const PUBLISH_POLL_PERIOD_BY_DEFAULT: Duration = Duration::from_secs(1);
+/// 一拍里至多调几次放锁的后台入口：一趟至多 `3S − 4` 次发布（S 至多 16），另加几趟的收尾；用户改动不停时不让一拍一直不返回（卸载与 `publish_when_due` 要等它）。
+pub const COMPACTION_CALLS_PER_MAINTENANCE_PASS_AT_MOST: u32 = 64;
+
+/// 运行库的挂载参数（都不存盘）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct RuntimeParameters {
+    pub background_compute: BackgroundComputeSettings,
+    pub gc_cpu_compensation_threshold: BackgroundGcCpuCompensationThreshold,
+    /// 批大小与三条水位线（默认 16、全空聚簇段线 5% 下限 2、另两条关）。
+    pub compaction: CompactionMountParameters,
+    /// 后台整理的周期，大于 0。
+    pub compaction_period: Duration,
+    /// `publish_when_due` 的周期（D16（发布语义） 已定项 2、5 的 P），大于 0。
+    pub publish_poll_period: Duration,
+    pub write_bandwidth_limit: CompactionWriteBandwidthLimit,
+}
+
+impl RuntimeParameters {
+    /// 全取默认。
+    #[must_use]
+    pub fn by_default() -> Self {
+        RuntimeParameters {
+            background_compute: BackgroundComputeSettings::PRODUCT,
+            gc_cpu_compensation_threshold: BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT,
+            compaction: CompactionMountParameters::BY_DEFAULT,
+            compaction_period: COMPACTION_PERIOD_BY_DEFAULT,
+            publish_poll_period: PUBLISH_POLL_PERIOD_BY_DEFAULT,
+            write_bandwidth_limit: COMPACTION_WRITE_BANDWIDTH_LIMIT_BY_DEFAULT,
+        }
+    }
+
+    /// 交给核心层后台入口的节奏：定时兜底的间隔取周期的一半。周期由运行库的睡眠定，核心层那一道闸只兜底：
+    /// 间隔等于周期时，某次醒来比上一次早到几毫秒，核心层就答「间隔没到」，实际周期隔一次才放行。
+    fn compaction_pacing(&self) -> CompactionPacing {
+        let half_period_in_milliseconds =
+            u64::try_from(self.compaction_period.as_millis() / 2).unwrap_or(u64::MAX);
+        CompactionPacing {
+            timer_fallback: match std::num::NonZeroU64::new(half_period_in_milliseconds) {
+                Some(interval) => CompactionTimerFallback::EveryMilliseconds(interval),
+                None => CompactionTimerFallback::EveryMilliseconds(std::num::NonZeroU64::MIN),
+            },
+            write_bandwidth_limit: self.write_bandwidth_limit,
+        }
+    }
+}
+
+/// 运行库的钟：核心层不读钟，定时兜底、写带宽与发布触发都读它。两个实现：产品的单调钟（[`MonotonicClockOfThisProcess`]）与测试摆的假钟。
+pub trait ClockOfTheRuntime: Send + Sync {
+    fn reading(&self) -> CallerMonotonicClockReading;
+}
+
+/// 从建它那一刻算的单调钟。
+#[derive(Clone, Copy, Debug)]
+pub struct MonotonicClockOfThisProcess {
+    started_at: Instant,
+}
+
+impl MonotonicClockOfThisProcess {
+    #[must_use]
+    pub fn starting_now() -> Self {
+        MonotonicClockOfThisProcess {
+            started_at: Instant::now(),
+        }
+    }
+}
+
+impl ClockOfTheRuntime for MonotonicClockOfThisProcess {
+    fn reading(&self) -> CallerMonotonicClockReading {
+        CallerMonotonicClockReading(self.started_at.elapsed())
+    }
+}
+
+/// 开卡没开成。成员按调用方要做的决定分。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum BackgroundGpuNotOpened {
+    /// 这一次开卡过了期限还没回来：工作线程留在驱动里不杀（没有安全的杀法），挂载拒绝，调用线程不再等。
+    TimedOut { deadline: Duration },
+    /// 上一次开卡过了期限、到现在还没回来：闸住新开卡，不再多起一条枚举线程（同一个驱动加载器里再起一条会不会也挂没量过）。
+    AnEarlierOpeningIsStillHung,
+    /// 这个进程建实例的次数到了上限（新建到一定数量 Vulkan 驱动不再认得出独显，额度按进程生命期累计）：要重启进程。
+    InstanceLimitOfThisProcessReached { instances_created: u32 },
+    /// 开卡器自己拒的（设置被拒、点名的卡不在、参与的卡用不上这一类）：`description` 是开卡器报的原样。
+    RefusedByTheOpening { description: String },
+}
+
+/// 开成的后台 GPU：交给核心层挂载的把手，与开卡时跳过了哪几张卡的报告（`.claude/rules/fs-design.md` 五条硬要求第 4 条）。
+#[derive(Debug)]
+pub struct OpenedBackgroundGpu {
+    pub gpu_compute: BackgroundGpuComputeHandle,
+    pub report_of_the_opening: String,
+}
+
+/// 开卡器（D26（后台整理与放置回收） 已定项 13 第 4 条「挂载时由调用方把它的实现递进去」）：运行库不依赖产品 GPU 包，实现在那边
+/// （`singlefs_background_gpu::gpu_opener_of_this_process`）。开卡在挂载之前：选 `gpu` 而开不成就在读任何一块盘之前拒绝挂载。
+/// 期限、闸、进程级实例与泄漏账都在实现里；这里只收到一个结局，所以这一次调用要在实现自己的期限之内返回。
+pub trait OpensTheBackgroundGpu {
+    /// # Errors
+    /// [`BackgroundGpuNotOpened`] 的几种。
+    fn open_the_background_gpu(
+        &self,
+        settings: &BackgroundComputeSettings,
+    ) -> Result<OpenedBackgroundGpu, BackgroundGpuNotOpened>;
+}
+
+/// 维护线程上每一件事的结局（`BackgroundCompactionCall` 是 `#[must_use]`：报警通道就是这里）。排空待发那几次发布是前台调用方不知道的发布点，一并送出。
+#[derive(Debug)]
+pub enum MaintenanceEvent<'event> {
+    /// `publish_when_due` 或整理之前排空发成的一次发布。
+    PendingChangesPublished(&'event PublishReportedByTheSession),
+    /// `publish_when_due` 那一次没发成。
+    PublishWhenDueRefused(&'event PublishOfThePendingChangesRefused),
+    /// 后台入口这一次调用的结局，整理停着时带着报警（`BackgroundCompactionCall::alarm_of_the_compaction_stopped_after_a_compute_error`）。
+    CompactionCallReturned(&'event BackgroundCompactionCall),
+    /// 后台入口这一次调用没跑（盘表不对、现行那一版没有树、排空被拒）。
+    CompactionCallRefused(&'event BackgroundCompactionAfterThePendingChangesRefused),
+    /// 一批写盘报错把发布路径冻结了，这一次重发成了：前台的写从此放行。
+    FrozenPublishResent,
+    /// 冻结着的那一次这一拍重发又报错：照旧冻结，前台的写照旧被拒（`AcceptRefused::PublishFrozenAfterAWriteFailureIsNotResentYet`），整理这一拍不跑。
+    FrozenPublishResendFailed(&'event PublishError),
+    /// 维护线程 panic 了：等价于崩溃，运行库从此报 [`RuntimeFailed`]。
+    MaintenanceThreadPanicked { description: &'event str },
+}
+
+/// 维护线程的事件送到哪。至少两个实现：不记的（[`IgnoringTheMaintenanceEvents`]）与调用方自己的（首个真正的入口是 FUSE 守护进程，它把报警告诉文件系统的用户）。
+pub trait ReceiverOfTheMaintenanceEvents: Send + Sync {
+    fn maintenance_event(&self, event: &MaintenanceEvent<'_>);
+}
+
+/// 不记事件的接收方：只看 [`MountedRuntime::observations`] 的调用方用它。
+#[derive(Clone, Copy, Debug, Default)]
+pub struct IgnoringTheMaintenanceEvents;
+
+impl ReceiverOfTheMaintenanceEvents for IgnoringTheMaintenanceEvents {
+    fn maintenance_event(&self, _event: &MaintenanceEvent<'_>) {}
+}
+
+/// 运行库累计的观测（`.claude/rules/fs-design.md` 五条硬要求第 4 条）。只住内存。
+#[derive(Clone, Debug, Default, PartialEq, Eq)]
+pub struct RuntimeObservations {
+    pub maintenance_passes: u64,
+    pub compaction_calls: u64,
+    /// 后台入口发成的整理发布次数。
+    pub compaction_publishes: u64,
+    /// 维护线程发成的排空待发次数。
+    pub pending_publishes: u64,
+    pub alarms_received: u64,
+    pub last_alarm: Option<AlarmOfABackgroundTaskStoppedAfterAComputeError>,
+    pub last_not_started: Option<BackgroundCompactionNotStarted>,
+    pub last_stop: Option<CompactionRunStop>,
+    pub frozen_publishes_resent: u64,
+    pub frozen_publish_resends_failed: u64,
+}
+
+/// 同步的一拍做成了什么（[`MountedRuntime::run_one_maintenance_pass`]）。
+#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
+pub struct MaintenancePass {
+    pub compaction_calls: u32,
+    pub compaction_publishes: u64,
+    /// 这一趟被节奏截住、或到了一拍的调用次数上限而没完：下一拍不等一个周期。
+    pub is_a_trip_under_way: bool,
+}
+
+/// 线程或前台在锁里 panic 过：会话可能停在半个更新里，只能丢掉内存、从盘重挂。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct RuntimeFailed {
+    pub description: String,
+}
+
+/// 前台入口没做成。
+#[derive(Debug)]
+pub enum RuntimeRefused<Refusal> {
+    /// 核心层拒的，原样。
+    Refused(Refusal),
+    /// 运行库已经坏了。
+    Failed(RuntimeFailed),
+}
+
+/// 挂载没做成的原因。
+#[derive(Debug)]
+pub enum RuntimeMountRefusal {
+    /// 周期是 0，在读任何一块盘之前拒。
+    PeriodOfZero,
+    /// 选 `gpu` 而开卡没成，在读任何一块盘之前拒。
+    BackgroundGpuNotOpened(BackgroundGpuNotOpened),
+    /// 核心层拒的，原样。
+    Mount(WritableMountWithBackgroundComputeRefused),
+    /// 维护线程起不来。挂载已经写过盘，会话丢掉，等价于崩溃：下一次挂载走恢复。
+    MaintenanceThreadNotStarted { description: String },
+}
+
+/// 挂载没做成：盘原样交回。
+pub struct RuntimeMountRefused<Device> {
+    pub devices: Vec<(DeviceIdentity, Device)>,
+    pub reason: RuntimeMountRefusal,
+}
+
+impl<Device> fmt::Debug for RuntimeMountRefused<Device> {
+    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
+        formatter
+            .debug_struct("RuntimeMountRefused")
+            .field("reason", &self.reason)
+            .finish_non_exhaustive()
+    }
+}
+
+/// 挂载做成了：挂载写出的东西（挂载续做那一趟的结局在里面）与运行库。
+#[must_use = "丢掉运行库只停线程、不排空待发，等价于崩溃：要 unmount"]
+pub struct RuntimeMounted<Device> {
+    pub output: MountOutput,
+    pub runtime: MountedRuntime<Device>,
+}
+
+impl<Device> fmt::Debug for RuntimeMounted<Device> {
+    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
+        formatter
+            .debug_struct("RuntimeMounted")
+            .field("runtime", &self.runtime)
+            .finish_non_exhaustive()
+    }
+}
+
+/// 卸载做成了：盘交回。
+pub struct UnmountedPool<Device> {
+    pub devices: Vec<(DeviceIdentity, Device)>,
+    /// 卸载之前排空待发那一次（待发空时 `None`）。
+    pub pending_changes_published: Option<PublishReportedByTheSession>,
+    pub unmounted: Unmounted,
+}
+
+impl<Device> fmt::Debug for UnmountedPool<Device> {
+    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
+        formatter
+            .debug_struct("UnmountedPool")
+            .field("pending_changes_published", &self.pending_changes_published)
+            .finish_non_exhaustive()
+    }
+}
+
+/// 卸载没做成。
+pub enum UnmountRefused<Device> {
+    /// 运行库已经坏了：内存不接着用，盘交回，等价于崩溃。
+    Failed {
+        devices: Vec<(DeviceIdentity, Device)>,
+        failure: RuntimeFailed,
+    },
+    /// 排空待发没发成：运行库原样交回（维护线程已经停了），待发没冻结时还在里面。可以重发冻结的那一次或腾出空间之后再卸载；丢掉它就是崩溃。
+    PendingChangesNotPublished {
+        runtime: MountedRuntime<Device>,
+        refusal: Box<UserChangeRefused>,
+    },
+    /// 核心层的卸载没做成：盘交回，内存不接着用，等价于崩溃。
+    UnmountFailed {
+        devices: Vec<(DeviceIdentity, Device)>,
+        refusal: MountError,
+    },
+}
+
+impl<Device> fmt::Debug for UnmountRefused<Device> {
+    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
+        match self {
+            UnmountRefused::Failed { failure, .. } => formatter
+                .debug_struct("Failed")
+                .field("failure", failure)
+                .finish_non_exhaustive(),
+            UnmountRefused::PendingChangesNotPublished { refusal, .. } => formatter
+                .debug_struct("PendingChangesNotPublished")
+                .field("refusal", refusal)
+                .finish_non_exhaustive(),
+            UnmountRefused::UnmountFailed { refusal, .. } => formatter
+                .debug_struct("UnmountFailed")
+                .field("refusal", refusal)
+                .finish_non_exhaustive(),
+        }
+    }
+}
+
+/// 一把锁里的东西：攒批的会话与盘表。
+pub struct SessionAndDevices<Device> {
+    pub session: MountedSessionWithPendingChanges,
+    pub devices: Vec<(DeviceIdentity, Device)>,
+}
+
+/// 运行库各处共用的那一份。`fsync_waiters`、`stopping` 不在会话锁里：核心层问它们的那一刻后台线程正拿着锁。
+struct Shared<Device> {
+    parameters: MakeFilesystemParameters,
+    runtime_parameters: RuntimeParameters,
+    locked: Mutex<SessionAndDevices<Device>>,
+    fsync_waiters: AtomicU32,
+    stopping: AtomicBool,
+    sleep_lock: Mutex<()>,
+    wake_the_maintenance_thread: Condvar,
+    failed: Mutex<Option<RuntimeFailed>>,
+    observations: Mutex<RuntimeObservations>,
+    clock: Arc<dyn ClockOfTheRuntime>,
+    receiver: Arc<dyn ReceiverOfTheMaintenanceEvents>,
+}
+
+/// 后台入口每一批之前问的两样（核心层不起线程，钟与前台 fsync 都在这一层）。停线程时也答「在等」：整理在下一批之前就返回，线程很快 join 得上。
+struct CallerOfTheRuntime<'shared> {
+    clock: &'shared dyn ClockOfTheRuntime,
+    fsync_waiters: &'shared AtomicU32,
+    stopping: &'shared AtomicBool,
+}
+
+impl CallerOfTheBackgroundCompaction for CallerOfTheRuntime<'_> {
+    fn clock_reading(&self) -> CallerMonotonicClockReading {
+        self.clock.reading()
+    }
+
+    fn foreground_fsync(&self) -> ForegroundFsync {
+        if self.fsync_waiters.load(Ordering::SeqCst) > 0 || self.stopping.load(Ordering::SeqCst) {
+            ForegroundFsync::Waiting
+        } else {
+            ForegroundFsync::NoneWaiting
+        }
+    }
+}
+
+/// 前台有一个 fsync 在等：取锁之前登记，丢弃时减（错误返回与 panic 展开都走到）。
+#[must_use = "丢掉就不算在等了"]
+pub struct ForegroundFsyncWaiting<'runtime> {
+    fsync_waiters: &'runtime AtomicU32,
+}
+
+impl Drop for ForegroundFsyncWaiting<'_> {
+    fn drop(&mut self) {
+        self.fsync_waiters.fetch_sub(1, Ordering::SeqCst);
+    }
+}
+
+impl<Device> Shared<Device> {
+    /// 叫维护线程停：拿着睡眠锁通知，线程在睡之前查 `stopping`，通知丢不了。
+    fn ask_the_maintenance_thread_to_stop(&self) {
+        self.stopping.store(true, Ordering::SeqCst);
+        let _sleep_guard = self
+            .sleep_lock
+            .lock()
+            .unwrap_or_else(PoisonError::into_inner);
+        self.wake_the_maintenance_thread.notify_all();
+    }
+}
+
+fn description_of_a_panic(payload: &(dyn Any + Send)) -> String {
+    match (
+        payload.downcast_ref::<&str>(),
+        payload.downcast_ref::<String>(),
+    ) {
+        (Some(text), _) => (*text).to_string(),
+        (None, Some(text)) => text.clone(),
+        (None, None) => "panic 带的不是字符串".to_string(),
+    }
+}
+
+impl<Device: BlockDevice> Shared<Device> {
+    /// 记下坏了（只记第一次）。观测与坏的记号在各自的小锁里，那几把锁里不跑会 panic 的代码，中毒照用。
+    fn note_the_failure(&self, description: String) {
+        let mut failed = self.failed.lock().unwrap_or_else(PoisonError::into_inner);
+        if failed.is_none() {
+            *failed = Some(RuntimeFailed { description });
+        }
+    }
+
+    fn failure_so_far(&self) -> Option<RuntimeFailed> {
+        self.failed
+            .lock()
+            .unwrap_or_else(PoisonError::into_inner)
+            .clone()
+    }
+
+    /// 取会话锁。坏过或锁中毒都报 [`RuntimeFailed`]：不用 `into_inner` 接着用半个更新里的会话。
+    fn lock_the_session(&self) -> Result<MutexGuard<'_, SessionAndDevices<Device>>, RuntimeFailed> {
+        if let Some(failure) = self.failure_so_far() {
+            return Err(failure);
+        }
+        self.locked.lock().map_err(|_poisoned| {
+            let failure = RuntimeFailed {
+                description: "持着会话锁的线程 panic 过".to_string(),
+            };
+            self.note_the_failure(failure.description.clone());
+            failure
+        })
+    }
+
+    fn update_the_observations(&self, update: impl FnOnce(&mut RuntimeObservations)) {
+        let mut observations = self
+            .observations
+            .lock()
+            .unwrap_or_else(PoisonError::into_inner);
+        update(&mut observations);
+    }
+
+    fn deliver(&self, event: &MaintenanceEvent<'_>) {
+        self.receiver.maintenance_event(event);
+    }
+
+    fn caller(&self) -> CallerOfTheRuntime<'_> {
+        CallerOfTheRuntime {
+            clock: &*self.clock,
+            fsync_waiters: &self.fsync_waiters,
+            stopping: &self.stopping,
+        }
+    }
+
+    /// `publish_when_due` 一次（持锁一次发布），结局送接收方。
+    fn publish_when_due_once(&self) -> Result<(), RuntimeFailed> {
+        let now = self.clock.reading();
+        let outcome = {
+            let mut locked = self.lock_the_session()?;
+            let SessionAndDevices { session, devices } = &mut *locked;
+            session.publish_when_due(devices, now)
+        };
+        match &outcome {
+            Ok(Some(published)) => {
+                self.update_the_observations(|observations| observations.pending_publishes += 1);
+                self.deliver(&MaintenanceEvent::PendingChangesPublished(published));
+            }
+            Ok(None) => {}
+            Err(refusal) => self.deliver(&MaintenanceEvent::PublishWhenDueRefused(refusal)),
+        }
+        Ok(())
+    }
+
+    /// 上一批写盘报错冻结了发布就重发一次。交回这一拍还冻不冻着：冻着时整理不跑（每次都是签发一次、被拒一次）。
+    fn resend_the_frozen_publish_if_any(&self) -> Result<bool, RuntimeFailed> {
+        let mut locked = self.lock_the_session()?;
+        let SessionAndDevices { session, devices } = &mut *locked;
+        if session.session().allocator.frozen_publish().is_none() {
+            return Ok(false);
+        }
+        let outcome = session.resend_the_frozen_publish(devices);
+        drop(locked);
+        match outcome {
+            Ok(_resent) => {
+                self.update_the_observations(|observations| {
+                    observations.frozen_publishes_resent += 1;
+                });
+                self.deliver(&MaintenanceEvent::FrozenPublishResent);
+                Ok(false)
+            }
+            Err(refusal) => {
+                self.update_the_observations(|observations| {
+                    observations.frozen_publish_resends_failed += 1;
+                });
+                self.deliver(&MaintenanceEvent::FrozenPublishResendFailed(&refusal));
+                Ok(true)
+            }
+        }
+    }
+
+    /// 一拍：`publish_when_due`、重发冻结的那一次，再一次次调放锁的后台入口，每次之间放锁。没起跑、被节奏截住、出错或到了调用次数上限就停。
+    /// 用户改动之后那一趟（至多一批）停了接着调：定时兜底那一支在同一拍里轮得到，用户改动不断时整理不会只剩每拍一批。
+    fn run_one_maintenance_pass(&self) -> Result<MaintenancePass, RuntimeFailed> {
+        self.update_the_observations(|observations| observations.maintenance_passes += 1);
+        self.publish_when_due_once()?;
+        let mut pass = MaintenancePass::default();
+        if self.resend_the_frozen_publish_if_any()? {
+            return Ok(pass);
+        }
+        let pacing = self.runtime_parameters.compaction_pacing();
+        let caller = self.caller();
+        while pass.compaction_calls < COMPACTION_CALLS_PER_MAINTENANCE_PASS_AT_MOST
+            && !self.stopping.load(Ordering::SeqCst)
+        {
+            let (outcome, is_a_trip_under_way) = {
+                let mut locked = self.lock_the_session()?;
+                let SessionAndDevices { session, devices } = &mut *locked;
+                let outcome = session.run_compaction_in_the_background_for_at_most_one_publish(
+                    devices, pacing, &caller,
+                );
+                let is_a_trip_under_way = session
+                    .session()
+                    .compaction
+                    .has_a_background_trip_under_way();
+                (outcome, is_a_trip_under_way)
+            };
+            pass.compaction_calls += 1;
+            pass.is_a_trip_under_way = is_a_trip_under_way;
+            self.update_the_observations(|observations| observations.compaction_calls += 1);
+            let completed = match &outcome {
+                Ok(completed) => completed,
+                Err(refusal) => {
+                    self.deliver(&MaintenanceEvent::CompactionCallRefused(refusal));
+                    break;
+                }
+            };
+            if let Some(published) = &completed.pending_changes_published_before_the_run {
+                self.update_the_observations(|observations| observations.pending_publishes += 1);
+                self.deliver(&MaintenanceEvent::PendingChangesPublished(published));
+            }
+            self.deliver(&MaintenanceEvent::CompactionCallReturned(&completed.call));
+            if let Some(alarm) = completed
+                .call
+                .alarm_of_the_compaction_stopped_after_a_compute_error()
+            {
+                self.update_the_observations(|observations| {
+                    observations.alarms_received += 1;
+                    observations.last_alarm = Some(alarm.clone());
+                });
+            }
+            let keeps_going = match &completed.call {
+                BackgroundCompactionCall::NotStarted { reason, .. } => {
+                    self.update_the_observations(|observations| {
+                        observations.last_not_started = Some(*reason);
+                    });
+                    false
+                }
+                BackgroundCompactionCall::Ran { run, .. } => {
+                    let publishes =
+                        u64::try_from(run.versions_published.len()).expect("一次调用的发布数");
+                    pass.compaction_publishes += publishes;
+                    self.update_the_observations(|observations| {
+                        observations.compaction_publishes += publishes;
+                    });
+                    match &run.outcome {
+                        CompactionRunOutcome::Stopped(stop) => {
+                            self.update_the_observations(|observations| {
+                                observations.last_stop = Some(*stop);
+                            });
+                            !matches!(
+                                stop,
+                                CompactionRunStop::YieldedToAForegroundFsync
+                                    | CompactionRunStop::WriteBandwidthLimitReached { .. }
+                            )
+                        }
+                        CompactionRunOutcome::Failed(_) => false,
+                    }
+                }
+            };
+            if !keeps_going {
+                break;
+            }
+            self.publish_when_due_once()?;
+        }
+        Ok(pass)
+    }
+
+    /// 线程体：两个到期时刻。`stopping` 在睡之前查、卸载在拿着睡眠锁时通知，通知丢不了。
+    fn maintenance_thread_body(&self) -> Result<(), RuntimeFailed> {
+        let compaction_period = self.runtime_parameters.compaction_period;
+        let publish_poll_period = self.runtime_parameters.publish_poll_period;
+        let mut next_compaction_at = self.clock.reading().0;
+        while !self.stopping.load(Ordering::SeqCst) {
+            let now = self.clock.reading().0;
+            if now >= next_compaction_at {
+                let pass = self.run_one_maintenance_pass()?;
+                next_compaction_at = now
+                    + if pass.is_a_trip_under_way {
+                        publish_poll_period
+                    } else {
+                        compaction_period
+                    };
+            } else {
+                self.publish_when_due_once()?;
+            }
+            let until_the_next_compaction =
+                next_compaction_at.saturating_sub(self.clock.reading().0);
+            let sleep_for = publish_poll_period.min(until_the_next_compaction);
+            let sleep_guard = self
+                .sleep_lock
+                .lock()
+                .unwrap_or_else(PoisonError::into_inner);
+            if !self.stopping.load(Ordering::SeqCst) {
+                let _woken_or_timed_out = self
+                    .wake_the_maintenance_thread
+                    .wait_timeout(sleep_guard, sleep_for)
+                    .unwrap_or_else(PoisonError::into_inner);
+            }
+        }
+        Ok(())
+    }
+}
+
+/// 一次可写挂载之后的运行库。`Device` 要 `Send` 才起得了线程；录制盘走不起线程的那一种（[`Self::mount_without_the_maintenance_thread`]）。
+#[must_use = "丢掉运行库只停线程、不排空待发，等价于崩溃：要 unmount"]
+pub struct MountedRuntime<Device> {
+    shared: Arc<Shared<Device>>,
+    maintenance_thread: Option<JoinHandle<()>>,
+}
+
+impl<Device> fmt::Debug for MountedRuntime<Device> {
+    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
+        formatter
+            .debug_struct("MountedRuntime")
+            .field(
+                "has_a_maintenance_thread",
+                &self.maintenance_thread.is_some(),
+            )
+            .finish_non_exhaustive()
+    }
+}
+
+impl<Device: BlockDevice + Send + 'static> MountedRuntime<Device> {
+    /// 挂载：校验周期 → 开卡（只在选 `gpu` 时碰开卡器，读任何一块盘之前）→ 可写挂载（挂载续做在这里跑完，线程还没起，没有交错）→ 接会话 → 起维护线程。
+    /// 调用方之后不调任何后台入口。
+    ///
+    /// # Errors
+    /// [`RuntimeMountRefused`]：盘原样交回。
+    pub fn mount(
+        parameters: &MakeFilesystemParameters,
+        devices: Vec<(DeviceIdentity, Device)>,
+        runtime_parameters: RuntimeParameters,
+        gpu_opener: Option<&dyn OpensTheBackgroundGpu>,
+        clock: Arc<dyn ClockOfTheRuntime>,
+        receiver: Arc<dyn ReceiverOfTheMaintenanceEvents>,
+    ) -> Result<RuntimeMounted<Device>, RuntimeMountRefused<Device>> {
+        let RuntimeMounted {
+            output,
+            mut runtime,
+        } = MountedRuntime::mount_without_the_maintenance_thread(
+            parameters,
+            devices,
+            runtime_parameters,
+            gpu_opener,
+            clock,
+            receiver,
+        )?;
+        let shared_of_the_thread = Arc::clone(&runtime.shared);
+        let started = std::thread::Builder::new()
+            .name("singlefs-maintenance".to_string())
+            .spawn(move || {
+                let body = catch_unwind(AssertUnwindSafe(|| {
+                    shared_of_the_thread.maintenance_thread_body()
+                }));
+                match body {
+                    Ok(Ok(())) | Ok(Err(_)) => {}
+                    Err(payload) => {
+                        let description = description_of_a_panic(&*payload);
+                        shared_of_the_thread.note_the_failure(description.clone());
+                        shared_of_the_thread.deliver(
+                            &MaintenanceEvent::MaintenanceThreadPanicked {
+                                description: &description,
+                            },
+                        );
+                    }
+                }
+            });
+        match started {
+            Ok(thread) => {
+                runtime.maintenance_thread = Some(thread);
+                Ok(RuntimeMounted { output, runtime })
+            }
+            Err(error) => {
+                let shared = Arc::clone(&runtime.shared);
+                drop(runtime);
+                let devices_handed_back = Arc::try_unwrap(shared)
+                    .unwrap_or_else(|_still_shared| {
+                        panic!("维护线程没起来：运行库是这一份的唯一持有者")
+                    })
+                    .locked
+                    .into_inner()
+                    .unwrap_or_else(PoisonError::into_inner)
+                    .devices;
+                Err(RuntimeMountRefused {
+                    devices: devices_handed_back,
+                    reason: RuntimeMountRefusal::MaintenanceThreadNotStarted {
+                        description: error.to_string(),
+                    },
+                })
+            }
+        }
+    }
+}
+
+impl<Device: BlockDevice> MountedRuntime<Device> {
+    /// 同 [`MountedRuntime::mount`]，不起线程：调用方一拍一拍调 [`Self::run_one_maintenance_pass`]（确定性测试走它，录制盘过不了线程）。
+    ///
+    /// # Errors
+    /// [`RuntimeMountRefused`]：盘原样交回。
+    pub fn mount_without_the_maintenance_thread(
+        parameters: &MakeFilesystemParameters,
+        mut devices: Vec<(DeviceIdentity, Device)>,
+        runtime_parameters: RuntimeParameters,
+        gpu_opener: Option<&dyn OpensTheBackgroundGpu>,
+        clock: Arc<dyn ClockOfTheRuntime>,
+        receiver: Arc<dyn ReceiverOfTheMaintenanceEvents>,
+    ) -> Result<RuntimeMounted<Device>, RuntimeMountRefused<Device>> {
+        if runtime_parameters.compaction_period.is_zero()
+            || runtime_parameters.publish_poll_period.is_zero()
+        {
+            return Err(RuntimeMountRefused {
+                devices,
+                reason: RuntimeMountRefusal::PeriodOfZero,
+            });
+        }
+        let gpu_compute = match (
+            &runtime_parameters.background_compute.background_compute,
+            gpu_opener,
+        ) {
+            (BackgroundComputeBackend::Gpu, Some(opener)) => {
+                match opener.open_the_background_gpu(&runtime_parameters.background_compute) {
+                    Ok(opened) => Some(opened.gpu_compute),
+                    Err(not_opened) => {
+                        return Err(RuntimeMountRefused {
+                            devices,
+                            reason: RuntimeMountRefusal::BackgroundGpuNotOpened(not_opened),
+                        })
+                    }
+                }
+            }
+            (BackgroundComputeBackend::Gpu, None) | (BackgroundComputeBackend::Cpu, _) => None,
+        };
+        let mounted = match mount_writable_with_background_compute_settings_the_gc_cpu_compensation_threshold_and_compaction_parameters(
+            parameters,
+            &mut devices,
+            runtime_parameters.background_compute.clone(),
+            gpu_compute,
+            runtime_parameters.gc_cpu_compensation_threshold,
+            runtime_parameters.compaction,
+        ) {
+            Ok(mounted) => mounted,
+            Err(refused) => {
+                return Err(RuntimeMountRefused {
+                    devices,
+                    reason: RuntimeMountRefusal::Mount(refused),
+                })
+            }
+        };
+        let (output, session) = MountedSession::of_the_mount(mounted);
+        let runtime = MountedRuntime::of_the_session(
+            parameters.clone(),
+            runtime_parameters,
+            SessionAndDevices {
+                session: MountedSessionWithPendingChanges::of_the_session(session),
+                devices,
+            },
+            clock,
+            receiver,
+        );
+        Ok(RuntimeMounted { output, runtime })
+    }
+
+    fn of_the_session(
+        parameters: MakeFilesystemParameters,
+        runtime_parameters: RuntimeParameters,
+        locked: SessionAndDevices<Device>,
+        clock: Arc<dyn ClockOfTheRuntime>,
+        receiver: Arc<dyn ReceiverOfTheMaintenanceEvents>,
+    ) -> Self {
+        MountedRuntime {
+            shared: Arc::new(Shared {
+                parameters,
+                runtime_parameters,
+                locked: Mutex::new(locked),
+                fsync_waiters: AtomicU32::new(0),
+                stopping: AtomicBool::new(false),
+                sleep_lock: Mutex::new(()),
+                wake_the_maintenance_thread: Condvar::new(),
+                failed: Mutex::new(None),
+                observations: Mutex::new(RuntimeObservations::default()),
+                clock,
+                receiver,
+            }),
+            maintenance_thread: None,
+        }
+    }
+
+    /// 前台收下一次用户改动（`MountedSessionWithPendingChanges::accept`）。
+    ///
+    /// # Errors
+    /// 核心层拒的原样；运行库坏了报 `Failed`。
+    pub fn accept(
+        &self,
+        change: UserChange<'_>,
+    ) -> Result<ChangeAccepted, RuntimeRefused<AcceptRefused>> {
+        let now = self.shared.clock.reading();
+        let mut locked = self
+            .shared
+            .lock_the_session()
+            .map_err(RuntimeRefused::Failed)?;
+        let SessionAndDevices { session, devices } = &mut *locked;
+        session
+            .accept(devices, change, now)
+            .map_err(RuntimeRefused::Refused)
+    }
+
+    /// fsync 走这里（`MountedSessionWithPendingChanges::publish_pending_now`）：进门先登记「有 fsync 在等」（取锁之前），整理在下一批之前让路，每次发布之间都放锁。
+    ///
+    /// # Errors
+    /// 同 [`Self::accept`]。
+    pub fn publish_pending_now(
+        &self,
+    ) -> Result<
+        Option<PublishReportedByTheSession>,
+        RuntimeRefused<PublishOfThePendingChangesRefused>,
+    > {
+        let _waiting = self.a_foreground_fsync_is_waiting();
+        let now = self.shared.clock.reading();
+        let mut locked = self
+            .shared
+            .lock_the_session()
+            .map_err(RuntimeRefused::Failed)?;
+        let SessionAndDevices { session, devices } = &mut *locked;
+        session
+            .publish_pending_now(devices, now)
+            .map_err(RuntimeRefused::Refused)
+    }
+
+    /// 单发一次用户改动（`MountedSessionWithPendingChanges::publish_user_change`）。
+    ///
+    /// # Errors
+    /// 同 [`Self::accept`]。
+    pub fn publish_user_change(
+        &self,
+        change: UserChange<'_>,
+    ) -> Result<
+        Vec<PublishReportedByTheSession>,
+        RuntimeRefused<UserChangeAfterThePendingChangesRefused>,
+    > {
+        let now = self.shared.clock.reading();
+        let mut locked = self
+            .shared
+            .lock_the_session()
+            .map_err(RuntimeRefused::Failed)?;
+        let SessionAndDevices { session, devices } = &mut *locked;
+        session
+            .publish_user_change(devices, change, now)
+            .map_err(RuntimeRefused::Refused)
+    }
+
+    /// 挂着切换后台计算的设置，从下一批起生效（不排空待发）。开卡归调用方：开卡器不在这一条路上，`gpu_compute` 由调用方先开好递进来。
+    ///
+    /// # Errors
+    /// 同 [`Self::accept`]。
+    pub fn switch_background_compute(
+        &self,
+        settings: BackgroundComputeSettings,
+        gpu_compute: Option<BackgroundGpuComputeHandle>,
+        gc_cpu_compensation_threshold: Option<BackgroundGcCpuCompensationThreshold>,
+    ) -> Result<(), RuntimeRefused<BackgroundComputeSettingsRefused>> {
+        let mut locked = self
+            .shared
+            .lock_the_session()
+            .map_err(RuntimeRefused::Failed)?;
+        let switched = match gc_cpu_compensation_threshold {
+            Some(threshold) => locked
+                .session
+                .switch_background_compute_with_the_gc_cpu_compensation_threshold_from_the_next_batch(
+                    settings,
+                    gpu_compute,
+                    threshold,
+                ),
+            None => locked
+                .session
+                .switch_background_compute_from_the_next_batch(settings, gpu_compute),
+        };
+        switched.map_err(RuntimeRefused::Refused)
+    }
+
+    /// 在锁里读或改会话与盘（读路径 `read_at` 要盘表，走这里；用例摆开关也走这里）。闭包里 panic 等于前台在锁里 panic：运行库从此报 `Failed`。
+    ///
+    /// # Errors
+    /// 运行库坏了。
+    pub fn with_the_session_and_devices<Answer>(
+        &self,
+        work: impl FnOnce(&mut SessionAndDevices<Device>) -> Answer,
+    ) -> Result<Answer, RuntimeFailed> {
+        let mut locked = self.shared.lock_the_session()?;
+        Ok(work(&mut *locked))
+    }
+
+    /// 同步的一拍：线程体循环里那一次（`publish_when_due`、重发冻结的那一次、放锁的后台入口推到没得推）。不起线程的确定性测试直接调。
+    ///
+    /// # Errors
+    /// 运行库坏了。
+    pub fn run_one_maintenance_pass(&self) -> Result<MaintenancePass, RuntimeFailed> {
+        self.shared.run_one_maintenance_pass()
+    }
+
+    /// 累计的观测。
+    #[must_use]
+    pub fn observations(&self) -> RuntimeObservations {
+        self.shared
+            .observations
+            .lock()
+            .unwrap_or_else(PoisonError::into_inner)
+            .clone()
+    }
+
+    /// 登记一个前台 fsync 在等，丢弃交回的那一份就减（[`Self::publish_pending_now`] 自己走这里；用例拨它钉整理让路）。
+    pub fn a_foreground_fsync_is_waiting(&self) -> ForegroundFsyncWaiting<'_> {
+        self.shared.fsync_waiters.fetch_add(1, Ordering::SeqCst);
+        ForegroundFsyncWaiting {
+            fsync_waiters: &self.shared.fsync_waiters,
+        }
+    }
+
+    /// 停线程并等它结束（线程在一拍之间查 `stopping`，整理在下一批之前让路）。一批之内 GPU 还在等时要等到那一批出来。
+    fn stop_and_join_the_maintenance_thread(&mut self) {
+        if let Some(thread) = self.maintenance_thread.take() {
+            self.shared.ask_the_maintenance_thread_to_stop();
+            let _panicked_is_noted_by_the_thread_itself = thread.join();
+        }
+    }
+
+    /// 卸载：停线程 → 排空待发 → 走核心层的卸载 → 丢会话（GPU 把手随它丢，有没交完的提交的那张卡整张泄漏、记进开卡器那份账）→ 盘交回。
+    /// 活意图不用做完：留在盘上，下次挂载续做。
+    ///
+    /// # Errors
+    /// [`UnmountRefused`] 的三种。
+    pub fn unmount(mut self) -> Result<UnmountedPool<Device>, Box<UnmountRefused<Device>>> {
+        self.stop_and_join_the_maintenance_thread();
+        let failure = self.shared.failure_so_far();
+        let shared = Arc::clone(&self.shared);
+        drop(self);
+        let Shared {
+            parameters,
+            runtime_parameters,
+            locked,
+            clock,
+            receiver,
+            ..
+        } = Arc::try_unwrap(shared).unwrap_or_else(|_still_shared| {
+            panic!("线程 join 过了、前台入口都借着 self：运行库是这一份的唯一持有者")
+        });
+        let locked_or_poisoned = locked.into_inner();
+        let is_poisoned = locked_or_poisoned.is_err();
+        let SessionAndDevices {
+            session,
+            mut devices,
+        } = locked_or_poisoned.unwrap_or_else(PoisonError::into_inner);
+        if failure.is_some() || is_poisoned {
+            return Err(Box::new(UnmountRefused::Failed {
+                devices,
+                failure: failure.unwrap_or(RuntimeFailed {
+                    description: "持着会话锁的线程 panic 过".to_string(),
+                }),
+            }));
+        }
+        let now = clock.reading();
+        let (pending_changes_published, mut session) =
+            match session.into_session_after_publishing_the_pending_changes(&mut devices, now) {
+                Ok(drained_and_session) => drained_and_session,
+                Err(refused) => {
+                    let refused = *refused;
+                    return Err(Box::new(UnmountRefused::PendingChangesNotPublished {
+                        runtime: MountedRuntime::of_the_session(
+                            parameters,
+                            runtime_parameters,
+                            SessionAndDevices {
+                                session: refused.session_with_pending_changes,
+                                devices,
+                            },
+                            clock,
+                            receiver,
+                        ),
+                        refusal: refused.refusal,
+                    }));
+                }
+            };
+        let shadow_ledger = session.shadow_ledger;
+        match unmount(
+            &parameters,
+            &mut devices,
+            &mut session.allocator,
+            &mut session.current,
+            shadow_ledger,
+        ) {
+            Ok(unmounted) => Ok(UnmountedPool {
+                devices,
+                pending_changes_published,
+                unmounted,
+            }),
+            Err(refusal) => Err(Box::new(UnmountRefused::UnmountFailed { devices, refusal })),
+        }
+    }
+}
+
+impl<Device> Drop for MountedRuntime<Device> {
+    /// 只停线程，不排空、不写盘（等价于崩溃）：析构里写盘失败无处可报。
+    fn drop(&mut self) {
+        if let Some(thread) = self.maintenance_thread.take() {
+            self.shared.ask_the_maintenance_thread_to_stop();
+            let _panicked_is_noted_by_the_thread_itself = thread.join();
+        }
+    }
+}
diff --git a/crates/singlefs-mount-runtime/tests/common_pool_on_memory_devices/mod.rs b/crates/singlefs-mount-runtime/tests/common_pool_on_memory_devices/mod.rs
new file mode 100644
index 0000000000000000000000000000000000000000..25cc82e00d2f0c8846db17920be9f64bd1beccef
--- /dev/null
+++ b/crates/singlefs-mount-runtime/tests/common_pool_on_memory_devices/mod.rs
@@ -0,0 +1,452 @@
+//! 运行库用例共用的搭建：两块内存里的稀疏盘上 mkfs → 取号 → 暖机 → 新池新建文件（同验证包 `build_pool` 那一串），另一次挂载里顺序写第一个文件，
+//! 会话丢掉（「进程退出」）。运行库只依赖核心层，所以盘与搭建在这里另写一份。盘是 `Send` 的（录制流在 `Arc<Mutex>` 里）：维护线程带得走。
+//! 盘按 512 字节扇区存（没写过的读回 0），改盘的每个动作按先后记；另带用例摆的两样：数读、下几个写报错。
+#![allow(dead_code, reason = "几份测试文件各自 mod 进来，只用到其中一部分")]
+
+use std::collections::BTreeMap;
+use std::sync::atomic::{AtomicU64, Ordering};
+use std::sync::{Arc, Mutex};
+
+use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber};
+use singlefs_core::allocator::{DeviceFreeMap, PoolAllocator, UnitAreaStart};
+use singlefs_core::block_device::{
+    BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
+};
+use singlefs_core::make_filesystem::{
+    make_filesystem, root_ring_occupancy_after_make_filesystem, MakeFilesystemParameters,
+};
+use singlefs_core::mount::mount_writable;
+use singlefs_core::mounted_session::{MountedSession, UserChange};
+use singlefs_core::root_ring::RootRingSlotsPerRegion;
+use singlefs_core::system_configuration::{
+    SystemImmutableSizes, JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION,
+};
+use singlefs_core::transaction::{
+    acquire_instance, publish_first_file, warm_up, FirstFile, PartialWriteBytes, PoolWriter,
+    FIRST_INODE_NUMBER,
+};
+use singlefs_core::unit::data_unit_payload_capacity;
+
+/// 每块盘的字节数（同验证包 `build_pool` 的镜像）。
+pub const DEVICE_BYTES: u64 = 4 << 30;
+/// 内存盘的扇区，也是物理块大小。
+const SECTOR_BYTES: u64 = 512;
+pub const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"singlefs-runtime";
+pub const FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
+/// 新池新建文件那一份内容的字节数（同验证包 `build_pool` 的 3000 字节）。
+const FIRST_FILE_BYTES: usize = 3000;
+/// 另一次挂载里顺序写第一个文件的数据单元数（同验证包 `compaction_batches_on_the_gpu_path_through_the_background_gpu_compute.rs`）。
+pub const DATA_UNITS_WRITTEN_IN_AN_EARLIER_MOUNT: usize = 40;
+/// 带洞的池：部分写这么多个数据单元（隔一个写一个），旧落点放掉之后成洞。
+const DATA_UNITS_REWRITTEN_TO_LEAVE_HOLES: usize = 20;
+/// 带洞的池：之后再建这么多次 inode，根环转过一圈（默认几何 3 × 8 = 24 槽），放掉的落点过了 defer 窗口。
+const USER_CHANGES_TURNING_THE_ROOT_RING: usize = 30;
+
+/// 用例摆的故障，一个池两块盘共用一份。
+#[derive(Clone, Debug, Default)]
+pub struct DeviceFaults {
+    /// 两块盘读过几次。
+    pub reads: Arc<AtomicU64>,
+    /// 接下来还有几个写报错（写之前报，盘上不变）。
+    pub writes_left_to_fail: Arc<AtomicU64>,
+}
+
+/// 改盘的一个动作，按先后记。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum RecordedDeviceOperation {
+    Write {
+        device: DeviceIdentity,
+        offset_in_bytes: u64,
+        bytes: Vec<u8>,
+        durability: WriteDurability,
+    },
+    WriteZeroes {
+        device: DeviceIdentity,
+        offset_in_bytes: u64,
+        length_in_bytes: u64,
+    },
+    Discard {
+        device: DeviceIdentity,
+        offset_in_bytes: u64,
+        length_in_bytes: u64,
+    },
+    Barrier {
+        device: DeviceIdentity,
+    },
+}
+
+/// 一个池两块盘共用的录制流。
+pub type RecordedStream = Arc<Mutex<Vec<RecordedDeviceOperation>>>;
+
+/// 内存里的一块稀疏盘：写过的扇区（扇区号 → 512 字节），连同它记进的录制流。
+pub struct RecordedMemoryDevice {
+    identity: DeviceIdentity,
+    sectors: BTreeMap<u64, Vec<u8>>,
+    stream: RecordedStream,
+    faults: DeviceFaults,
+}
+
+impl RecordedMemoryDevice {
+    /// 这一个写该不该报错（用例摆的）。
+    fn fault_of_this_write(
+        &self,
+        offset: DeviceOffsetInBytes,
+        length: u64,
+    ) -> Result<(), BlockDeviceError> {
+        let fails = self
+            .faults
+            .writes_left_to_fail
+            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |left| {
+                left.checked_sub(1)
+            })
+            .is_ok();
+        if fails {
+            return Err(BlockDeviceError::OutOfRange {
+                offset,
+                length,
+                device_size: 0,
+            });
+        }
+        Ok(())
+    }
+
+    fn note(&self, operation: RecordedDeviceOperation) {
+        self.stream
+            .lock()
+            .expect("录制流只在这一条线程上记，没在别处 panic 过")
+            .push(operation);
+    }
+
+    fn check_request(
+        &self,
+        offset: DeviceOffsetInBytes,
+        length_in_bytes: u64,
+    ) -> Result<(), BlockDeviceError> {
+        if !offset.0.is_multiple_of(SECTOR_BYTES) || !length_in_bytes.is_multiple_of(SECTOR_BYTES) {
+            return Err(BlockDeviceError::Unaligned {
+                offset,
+                length: length_in_bytes,
+                physical_block_size: u32::try_from(SECTOR_BYTES).expect("512"),
+            });
+        }
+        if offset
+            .0
+            .checked_add(length_in_bytes)
+            .is_none_or(|end| end > DEVICE_BYTES)
+        {
+            return Err(BlockDeviceError::OutOfRange {
+                offset,
+                length: length_in_bytes,
+                device_size: DEVICE_BYTES,
+            });
+        }
+        Ok(())
+    }
+
+    fn zero_fill(&mut self, offset: DeviceOffsetInBytes, length_in_bytes: u64) {
+        let first_sector = offset.0 / SECTOR_BYTES;
+        let past_the_last_sector = first_sector + length_in_bytes / SECTOR_BYTES;
+        let written_in_the_range: Vec<u64> = self
+            .sectors
+            .range(first_sector..past_the_last_sector)
+            .map(|(sector, _bytes)| *sector)
+            .collect();
+        for sector in written_in_the_range {
+            self.sectors.remove(&sector);
+        }
+    }
+}
+
+impl BlockDevice for RecordedMemoryDevice {
+    fn read_at(
+        &self,
+        offset: DeviceOffsetInBytes,
+        buffer: &mut [u8],
+    ) -> Result<(), BlockDeviceError> {
+        let length_in_bytes = u64::try_from(buffer.len()).expect("读长装得进 u64");
+        self.check_request(offset, length_in_bytes)?;
+        self.faults.reads.fetch_add(1, Ordering::SeqCst);
+        let sector_bytes = usize::try_from(SECTOR_BYTES).expect("512");
+        buffer.fill(0);
+        let first_sector = offset.0 / SECTOR_BYTES;
+        for (sector, bytes) in self
+            .sectors
+            .range(first_sector..first_sector + length_in_bytes / SECTOR_BYTES)
+        {
+            let start = usize::try_from(sector - first_sector).expect("扇区下标") * sector_bytes;
+            buffer[start..start + sector_bytes].copy_from_slice(bytes);
+        }
+        Ok(())
+    }
+
+    fn write_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        bytes: &[u8],
+        durability: WriteDurability,
+    ) -> Result<(), BlockDeviceError> {
+        let length_in_bytes = u64::try_from(bytes.len()).expect("写长装得进 u64");
+        self.check_request(offset, length_in_bytes)?;
+        self.fault_of_this_write(offset, length_in_bytes)?;
+        let sector_bytes = usize::try_from(SECTOR_BYTES).expect("512");
+        let first_sector = offset.0 / SECTOR_BYTES;
+        for (sector_in_the_write, sector_contents) in bytes.chunks(sector_bytes).enumerate() {
+            self.sectors.insert(
+                first_sector + u64::try_from(sector_in_the_write).expect("扇区下标"),
+                sector_contents.to_vec(),
+            );
+        }
+        self.note(RecordedDeviceOperation::Write {
+            device: self.identity,
+            offset_in_bytes: offset.0,
+            bytes: bytes.to_vec(),
+            durability,
+        });
+        Ok(())
+    }
+
+    fn write_zeroes_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        length: u64,
+    ) -> Result<(), BlockDeviceError> {
+        self.check_request(offset, length)?;
+        self.zero_fill(offset, length);
+        self.note(RecordedDeviceOperation::WriteZeroes {
+            device: self.identity,
+            offset_in_bytes: offset.0,
+            length_in_bytes: length,
+        });
+        Ok(())
+    }
+
+    /// 内存盘认 discard，把它当整段清零应用（崩溃点重放「持久 → 0」那一态），交回「交给了底层」。
+    fn discard_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        length: u64,
+    ) -> Result<DiscardOutcome, BlockDeviceError> {
+        self.check_request(offset, length)?;
+        self.zero_fill(offset, length);
+        self.note(RecordedDeviceOperation::Discard {
+            device: self.identity,
+            offset_in_bytes: offset.0,
+            length_in_bytes: length,
+        });
+        Ok(DiscardOutcome::Applied)
+    }
+
+    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
+        self.note(RecordedDeviceOperation::Barrier {
+            device: self.identity,
+        });
+        Ok(())
+    }
+
+    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
+        PhysicalBlockSizeInBytes(u32::try_from(SECTOR_BYTES).expect("512"))
+    }
+
+    fn size_in_bytes(&self) -> u64 {
+        DEVICE_BYTES
+    }
+}
+
+/// 一个内存里的池：两块盘与它们共用的录制流。
+pub struct PoolOnMemoryDevices {
+    pub devices: Vec<(DeviceIdentity, RecordedMemoryDevice)>,
+    pub stream: RecordedStream,
+    pub faults: DeviceFaults,
+}
+
+impl PoolOnMemoryDevices {
+    /// 同一份字节的另一个池，录制流是空的：「同一历史」的另一个副本。
+    pub fn copied_with_an_empty_stream(&self) -> PoolOnMemoryDevices {
+        let stream: RecordedStream = Arc::new(Mutex::new(Vec::new()));
+        let faults = DeviceFaults::default();
+        let devices = self
+            .devices
+            .iter()
+            .map(|(identity, device)| {
+                (
+                    *identity,
+                    RecordedMemoryDevice {
+                        identity: *identity,
+                        sectors: device.sectors.clone(),
+                        stream: Arc::clone(&stream),
+                        faults: faults.clone(),
+                    },
+                )
+            })
+            .collect();
+        PoolOnMemoryDevices {
+            devices,
+            stream,
+            faults,
+        }
+    }
+
+    /// 两块盘此刻写过的扇区：两个池逐字节比。
+    pub fn written_sectors_of_every_device(&self) -> Vec<(DeviceIdentity, BTreeMap<u64, Vec<u8>>)> {
+        self.devices
+            .iter()
+            .map(|(identity, device)| (*identity, device.sectors.clone()))
+            .collect()
+    }
+
+    /// 录制流此刻记下的每个动作，按先后。
+    pub fn recorded_operations(&self) -> Vec<RecordedDeviceOperation> {
+        self.stream
+            .lock()
+            .expect("录制流只在这一条线程上记，没在别处 panic 过")
+            .clone()
+    }
+}
+
+/// 默认环长：从第一版的环起点到默认环长下单元区起点（环末尾的下一个槽）之间那么多字节。照核心层的两个起点算，不另抄一份格式常量。
+fn default_journal_ring_bytes() -> u64 {
+    UnitAreaStart::of_the_default_journal_ring()
+        .slot()
+        .to_device_offset()
+        .0
+        - JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION
+            .to_device_offset()
+            .0
+}
+
+/// mkfs 参数（同验证包 `common::parameters`，fsid 换成本包的）。
+pub fn parameters() -> MakeFilesystemParameters {
+    MakeFilesystemParameters {
+        filesystem_identifier: FILESYSTEM_IDENTIFIER,
+        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
+        geometry: SystemImmutableSizes {
+            physical_block_size: u32::try_from(SECTOR_BYTES).expect("512"),
+            minimum_input_output_bytes: u32::try_from(SECTOR_BYTES).expect("512"),
+            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(
+                u32::try_from(SECTOR_BYTES).expect("512"),
+            ),
+            journal_ring_bytes: default_journal_ring_bytes(),
+            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
+        },
+    }
+}
+
+fn contents_of_the_first_file() -> Vec<u8> {
+    (0..FIRST_FILE_BYTES)
+        .map(|index| u8::try_from(index % 251).expect("小于 256"))
+        .collect()
+}
+
+/// 新池新建文件之后，另一次挂载里顺序写第一个文件 [`DATA_UNITS_WRITTEN_IN_AN_EARLIER_MOUNT`] 个数据单元、会话丢掉。交回那一刻的池
+/// （录制流带着这一串的全部动作；拷池时换空流）。
+pub fn pool_after_a_sequential_write_in_an_earlier_mount() -> PoolOnMemoryDevices {
+    let parameters = parameters();
+    let stream: RecordedStream = Arc::new(Mutex::new(Vec::new()));
+    let faults = DeviceFaults::default();
+    let mut devices: Vec<(DeviceIdentity, RecordedMemoryDevice)> =
+        [DeviceIdentity(0), DeviceIdentity(1)]
+            .into_iter()
+            .map(|identity| {
+                (
+                    identity,
+                    RecordedMemoryDevice {
+                        identity,
+                        sectors: BTreeMap::new(),
+                        stream: Arc::clone(&stream),
+                        faults: faults.clone(),
+                    },
+                )
+            })
+            .collect();
+    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
+    let mut allocator = PoolAllocator::new(vec![
+        DeviceFreeMap::new(DeviceIdentity(0), DEVICE_BYTES),
+        DeviceFreeMap::new(DeviceIdentity(1), DEVICE_BYTES),
+    ]);
+    allocator.mark_format_time_units(
+        genesis.instance_table_placement(),
+        genesis.tree_table_genesis_placement(),
+    );
+    // mkfs 同一个进程里接着写的会话：装上 mkfs 刚写下的根环（同验证包 `build_pool`）。
+    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
+        &parameters,
+        &genesis,
+    ));
+    {
+        let mut writer = PoolWriter::new(&parameters, &mut devices);
+        let instance = acquire_instance(&mut writer).expect("取号");
+        let warmed_up = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
+        publish_first_file(
+            &mut writer,
+            &mut allocator,
+            warmed_up.roots.last().expect("暖机两代根"),
+            FirstFile {
+                content: &contents_of_the_first_file(),
+                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
+            },
+            instance,
+            &warmed_up.last_record_bytes,
+        )
+        .expect("新池新建文件");
+    }
+    let mounted = mount_writable(&parameters, &mut devices).expect("可写挂载");
+    let (_, mut session) = MountedSession::of_the_mount(mounted);
+    let content: Vec<u8> = (0..DATA_UNITS_WRITTEN_IN_AN_EARLIER_MOUNT
+        * data_unit_payload_capacity())
+        .map(|index| u8::try_from(index % 241).expect("小于 256"))
+        .collect();
+    session
+        .publish_user_change(
+            &mut devices,
+            UserChange::SequentialWriteOfAnInode {
+                inode: InodeNumber(FIRST_INODE_NUMBER),
+                content: &content,
+                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
+            },
+        )
+        .expect("顺序写第一个文件");
+    drop(session);
+    PoolOnMemoryDevices {
+        devices,
+        stream,
+        faults,
+    }
+}
+
+/// 带洞的池：同 [`pool_after_a_sequential_write_in_an_earlier_mount`]，接着隔一个数据单元部分写一个（旧落点放掉）、再建几次 inode 让根环转过一圈（放掉的落点过了 defer 窗口），
+/// 会话丢掉。数据段里有了洞：产品路径选段之后「R 外的空洞够不够」那一判过得了（只顺序写的池上段都是满的，判不过）。
+pub fn pool_with_holes_in_the_data_segments() -> PoolOnMemoryDevices {
+    let mut pool = pool_after_a_sequential_write_in_an_earlier_mount();
+    let parameters = parameters();
+    let mounted = mount_writable(&parameters, &mut pool.devices).expect("可写挂载");
+    let (_, mut session) = MountedSession::of_the_mount(mounted);
+    let one_byte = [0x5A_u8];
+    for rewritten in 0..DATA_UNITS_REWRITTEN_TO_LEAVE_HOLES {
+        let data_unit_index = u64::try_from(2 * rewritten).expect("单元下标");
+        let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷");
+        session
+            .publish_user_change(
+                &mut pool.devices,
+                UserChange::WriteAtOffset {
+                    inode: InodeNumber(FIRST_INODE_NUMBER),
+                    offset: FileOffsetInBytes(data_unit_index * payload),
+                    bytes: PartialWriteBytes::new(&one_byte).expect("一个字节不是空的"),
+                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
+                },
+            )
+            .expect("部分写一个数据单元");
+    }
+    for _turn in 0..USER_CHANGES_TURNING_THE_ROOT_RING {
+        session
+            .publish_user_change(
+                &mut pool.devices,
+                UserChange::NewInodes {
+                    count: 1,
+                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 3,
+                },
+            )
+            .expect("建一个 inode");
+    }
+    drop(session);
+    pool
+}
diff --git a/crates/singlefs-mount-runtime/tests/mount_runtime_compacts_by_itself_and_unmounts_clean.rs b/crates/singlefs-mount-runtime/tests/mount_runtime_compacts_by_itself_and_unmounts_clean.rs
new file mode 100644
index 0000000000000000000000000000000000000000..0c282e8fd75a444af9aafa72052f0d2d957d7655
--- /dev/null
+++ b/crates/singlefs-mount-runtime/tests/mount_runtime_compacts_by_itself_and_unmounts_clean.rs
@@ -0,0 +1,875 @@
+//! 运行库（D26（后台整理与放置回收） 已定项 13 第 5 条）的验收：挂载之后调用方不调任何后台入口，水位越线的池上 GC 自己跑、腾出空段，卸载干净。
+//! 确定性的格走 `run_one_maintenance_pass` 与测试摆的钟（真 sleep 不进这条路）；起线程的只留几条与时间无关的：线程自己跑得起来、停得下、panic 之后前台拿到错误。
+//! 水位越线靠挂载参数：全空聚簇段线取整段数的 100%，有一段不全空就越线；默认值下的判法在验证包 `compaction_watermarks_judge_every_branch.rs`。
+mod common_pool_on_memory_devices;
+
+use std::num::NonZeroU64;
+use std::sync::atomic::Ordering;
+use std::sync::{Arc, Mutex};
+use std::time::{Duration, Instant};
+
+use common_pool_on_memory_devices::{
+    parameters, pool_after_a_sequential_write_in_an_earlier_mount,
+    pool_with_holes_in_the_data_segments, DeviceFaults, PoolOnMemoryDevices, RecordedMemoryDevice,
+    FIXED_WRITE_TIME_SECONDS,
+};
+use singlefs_core::address::{DeviceIdentity, InodeNumber, SlotNumber};
+use singlefs_core::background_compute::{BackgroundComputeBackend, BackgroundComputeSettings};
+use singlefs_core::compaction_plan::{
+    CallerMonotonicClockReading, CallerOfTheBackgroundCompaction, CompactionBatchSize,
+    CompactionBatchUnits, CompactionMountParameters, CompactionPacing, CompactionRunStop,
+    CompactionStartJudgement, CompactionSwitches, CompactionTimerFallback, CompactionWatermarks,
+    CompactionWriteBandwidthLimit, EmptyClusterSegmentsWatermark, ForegroundFsync,
+    FragmentationWatermark, UnreclaimableBytesWatermark, COMPACTION_BATCH_UNITS_BY_DEFAULT,
+};
+use singlefs_core::logged_operations::{live_compaction_intent_of_the_entries, CompactionRegion};
+use singlefs_core::mounted_session::{
+    AcceptRefused, BackgroundCompactionCall, BackgroundCompactionNotStarted,
+    BackgroundCompactionTrigger, CompactionRunOutcome, ReasonOfThePublish, UserChange,
+};
+use singlefs_core::transaction::{FirstFile, TransactionOutput, FIRST_INODE_NUMBER};
+use singlefs_mount_runtime::{
+    BackgroundGpuNotOpened, ClockOfTheRuntime, IgnoringTheMaintenanceEvents, MaintenanceEvent,
+    MonotonicClockOfThisProcess, MountedRuntime, OpenedBackgroundGpu, OpensTheBackgroundGpu,
+    ReceiverOfTheMaintenanceEvents, RuntimeFailed, RuntimeMountRefusal, RuntimeMounted,
+    RuntimeParameters, RuntimeRefused,
+};
+
+/// 一拍之间钟走多久：默认周期。
+const ONE_PERIOD: Duration = Duration::from_secs(30);
+/// 确定性用例至多推几拍。
+const MAINTENANCE_PASSES_AT_MOST: usize = 40;
+/// 起线程的用例等线程做成一件事的上限：只防挂住，健康时远用不了。
+const THREAD_DEADLINE: Duration = Duration::from_secs(300);
+/// 第一块盘。
+const FIRST_DEVICE: usize = 0;
+
+/// 测试摆的钟。
+#[derive(Debug, Default)]
+struct ClockMovedByTheTest {
+    reading: Mutex<Duration>,
+}
+
+impl ClockMovedByTheTest {
+    fn advance_by(&self, elapsed: Duration) {
+        *self.reading.lock().expect("钟的锁里不 panic") += elapsed;
+    }
+}
+
+impl ClockOfTheRuntime for ClockMovedByTheTest {
+    fn reading(&self) -> CallerMonotonicClockReading {
+        CallerMonotonicClockReading(*self.reading.lock().expect("钟的锁里不 panic"))
+    }
+}
+
+/// 记下后台入口每次调用的结局。
+#[derive(Debug, Default)]
+struct RecordingTheCompactionCalls {
+    /// 每次调用：起跑的话为什么、发了几版；没起跑是 `None`。
+    calls: Mutex<Vec<Option<(BackgroundCompactionTrigger, usize)>>>,
+    reasons_of_the_pending_publishes: Mutex<Vec<ReasonOfThePublish>>,
+}
+
+impl ReceiverOfTheMaintenanceEvents for RecordingTheCompactionCalls {
+    fn maintenance_event(&self, event: &MaintenanceEvent<'_>) {
+        match event {
+            MaintenanceEvent::CompactionCallReturned(call) => {
+                self.calls
+                    .lock()
+                    .expect("这把锁里不 panic")
+                    .push(match call {
+                        BackgroundCompactionCall::NotStarted { .. } => None,
+                        BackgroundCompactionCall::Ran { trigger, run } => {
+                            Some((*trigger, run.versions_published.len()))
+                        }
+                    });
+            }
+            MaintenanceEvent::PendingChangesPublished(published) => {
+                self.reasons_of_the_pending_publishes
+                    .lock()
+                    .expect("这把锁里不 panic")
+                    .push(published.reason);
+            }
+            MaintenanceEvent::PublishWhenDueRefused(_)
+            | MaintenanceEvent::CompactionCallRefused(_)
+            | MaintenanceEvent::FrozenPublishResent
+            | MaintenanceEvent::FrozenPublishResendFailed(_)
+            | MaintenanceEvent::MaintenanceThreadPanicked { .. } => {}
+        }
+    }
+}
+
+/// 全空聚簇段线取整段数的 100%（有一段不全空就越线），另两条关；批大小取默认。
+fn parameters_over_the_watermark() -> RuntimeParameters {
+    let mut runtime_parameters = RuntimeParameters::by_default();
+    runtime_parameters.compaction = CompactionMountParameters {
+        batch_units: COMPACTION_BATCH_UNITS_BY_DEFAULT,
+        watermarks: CompactionWatermarks {
+            empty_cluster_segments: EmptyClusterSegmentsWatermark::PercentOfTheClusterSegments {
+                percent: 100,
+                lowest_line_in_segments: 2,
+            },
+            fragmentation: FragmentationWatermark::Off,
+            unreclaimable_bytes: UnreclaimableBytesWatermark::Off,
+        },
+    };
+    runtime_parameters
+}
+
+fn mounted_without_a_thread(
+    pool: PoolOnMemoryDevices,
+    runtime_parameters: RuntimeParameters,
+    clock: Arc<ClockMovedByTheTest>,
+    receiver: Arc<dyn ReceiverOfTheMaintenanceEvents>,
+) -> RuntimeMounted<RecordedMemoryDevice> {
+    MountedRuntime::mount_without_the_maintenance_thread(
+        &parameters(),
+        pool.devices,
+        runtime_parameters,
+        None,
+        clock,
+        receiver,
+    )
+    .expect("全合法的参数：挂载照常做成")
+}
+
+fn empty_segments_by_the_records(runtime: &MountedRuntime<RecordedMemoryDevice>) -> u64 {
+    runtime
+        .with_the_session_and_devices(|locked| {
+            locked.session.session().allocator.devices[FIRST_DEVICE].empty_segments()
+        })
+        .expect("运行库没坏")
+}
+
+fn current_version_of(
+    locked_session: &singlefs_core::mounted_session::MountedSession,
+) -> &TransactionOutput {
+    locked_session
+        .current
+        .version_of_trees()
+        .expect("有树的一版")
+}
+
+/// 验收第一条：产品开关（批大小默认、不强制选段）、水位真越线，调用方不调任何后台入口，几拍之后 GC 发布、整理完一条意图、记录口径的全空段涨了；
+/// 那一段刚搬空时还开不成（放掉的落点在 defer 窗口里），根环转过一圈之后开得成。
+#[test]
+fn maintenance_passes_on_the_product_switches_empty_a_segment_that_can_be_opened_after_the_ring_turns(
+) {
+    let pool = pool_with_holes_in_the_data_segments();
+    let clock = Arc::new(ClockMovedByTheTest::default());
+    let RuntimeMounted { runtime, .. } = mounted_without_a_thread(
+        pool,
+        parameters_over_the_watermark(),
+        Arc::clone(&clock),
+        Arc::new(IgnoringTheMaintenanceEvents),
+    );
+    let empty_before = empty_segments_by_the_records(&runtime);
+    for _pass in 0..MAINTENANCE_PASSES_AT_MOST {
+        clock.advance_by(ONE_PERIOD);
+        runtime.run_one_maintenance_pass().expect("运行库没坏");
+    }
+    let observations = runtime.observations();
+    assert!(
+        observations.compaction_publishes > 0,
+        "GC 自己发布了：{observations:?}"
+    );
+    let (intent_completed, intents_completed, empty_after_by_the_records, lowest_openable) =
+        runtime
+            .with_the_session_and_devices(|locked| {
+                let session = locked.session.session();
+                (
+                    session.compaction.last_completed_intent,
+                    session.compaction.observations.intents_completed,
+                    session.allocator.devices[FIRST_DEVICE].empty_segments(),
+                    session.allocator.devices[FIRST_DEVICE].lowest_empty_segment(),
+                )
+            })
+            .expect("运行库没坏");
+    assert!(intents_completed >= 1, "至少做完一条意图：{observations:?}");
+    assert!(
+        empty_after_by_the_records > empty_before,
+        "记录口径的全空段涨了：{empty_before} → {empty_after_by_the_records}"
+    );
+    let region: CompactionRegion = intent_completed.expect("记着上一条完成的意图").region;
+    assert!(
+        lowest_openable.is_none_or(|slot| slot.0 > region.start_slot().0),
+        "刚搬空的 R 还在 defer 窗口里，开段开不到它：{lowest_openable:?}，R 从 {:?} 起",
+        region.start_slot()
+    );
+    let mut opened_after_the_ring_turned = false;
+    for _publish in 0..32 {
+        runtime
+            .publish_user_change(UserChange::NewInodes {
+                count: 1,
+                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 9,
+            })
+            .expect("建一个 inode");
+        let lowest = runtime
+            .with_the_session_and_devices(|locked| {
+                locked.session.session().allocator.devices[FIRST_DEVICE].lowest_empty_segment()
+            })
+            .expect("运行库没坏");
+        if lowest.is_some_and(|slot| slot.0 <= region.start_slot().0) {
+            opened_after_the_ring_turned = true;
+            break;
+        }
+    }
+    assert!(
+        opened_after_the_ring_turned,
+        "根环转过一圈之后，R 过了 defer 窗口，开段开得到它"
+    );
+    runtime.unmount().expect("卸载干净");
+}
+
+/// 水位没越线（默认线，大盘上用得很少）：一拍接一拍，一次整理都不发。
+#[test]
+fn a_pool_under_the_default_watermarks_publishes_no_compaction() {
+    let pool = pool_with_holes_in_the_data_segments();
+    let clock = Arc::new(ClockMovedByTheTest::default());
+    let RuntimeMounted { runtime, .. } = mounted_without_a_thread(
+        pool,
+        RuntimeParameters::by_default(),
+        Arc::clone(&clock),
+        Arc::new(IgnoringTheMaintenanceEvents),
+    );
+    for _pass in 0..4 {
+        clock.advance_by(ONE_PERIOD);
+        runtime.run_one_maintenance_pass().expect("运行库没坏");
+    }
+    let observations = runtime.observations();
+    assert_eq!(observations.compaction_publishes, 0, "{observations:?}");
+    assert_eq!(
+        observations.last_not_started,
+        Some(BackgroundCompactionNotStarted::NotDueByTheWatermarksAtTheTimerFallback),
+        "每次都查了水位、没越线"
+    );
+}
+
+/// 每次发布之间放锁：放锁的后台入口每次调用至多发一版。
+#[test]
+fn every_call_of_a_maintenance_pass_publishes_at_most_one_version() {
+    let pool = pool_with_holes_in_the_data_segments();
+    let clock = Arc::new(ClockMovedByTheTest::default());
+    let receiver = Arc::new(RecordingTheCompactionCalls::default());
+    let RuntimeMounted { runtime, .. } = mounted_without_a_thread(
+        pool,
+        parameters_over_the_watermark(),
+        Arc::clone(&clock),
+        Arc::clone(&receiver) as Arc<dyn ReceiverOfTheMaintenanceEvents>,
+    );
+    clock.advance_by(ONE_PERIOD);
+    let pass = runtime.run_one_maintenance_pass().expect("运行库没坏");
+    let calls = receiver.calls.lock().expect("这把锁里不 panic").clone();
+    assert!(
+        pass.compaction_publishes >= 2,
+        "这一趟不止一次发布：{pass:?}"
+    );
+    assert!(
+        calls
+            .iter()
+            .flatten()
+            .all(|(_trigger, versions_published)| *versions_published <= 1),
+        "每次调用至多一版，调用之间放锁：{calls:?}"
+    );
+}
+
+/// 用户改动不断时定时兜底那一支轮得到：每拍之前一次用户改动，用户改动之后那一趟至多一批，同一拍接着调轮到定时兜底。
+#[test]
+fn with_a_user_change_before_every_pass_the_timer_fallback_still_runs() {
+    let pool = pool_with_holes_in_the_data_segments();
+    let clock = Arc::new(ClockMovedByTheTest::default());
+    let receiver = Arc::new(RecordingTheCompactionCalls::default());
+    let RuntimeMounted { runtime, .. } = mounted_without_a_thread(
+        pool,
+        parameters_over_the_watermark(),
+        Arc::clone(&clock),
+        Arc::clone(&receiver) as Arc<dyn ReceiverOfTheMaintenanceEvents>,
+    );
+    for pass_number in 0..4_u64 {
+        clock.advance_by(ONE_PERIOD);
+        runtime
+            .publish_user_change(UserChange::NewInodes {
+                count: 1,
+                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 10 + pass_number,
+            })
+            .expect("建一个 inode");
+        runtime.run_one_maintenance_pass().expect("运行库没坏");
+    }
+    let calls = receiver.calls.lock().expect("这把锁里不 panic").clone();
+    assert!(
+        calls.iter().flatten().any(
+            |(trigger, _versions)| *trigger == BackgroundCompactionTrigger::DueAfterAUserChange
+        ),
+        "用户改动之后那一趟跑了：{calls:?}"
+    );
+    assert!(
+        calls
+            .iter()
+            .flatten()
+            .any(|(trigger, _versions)| *trigger == BackgroundCompactionTrigger::TimerFallback),
+        "定时兜底那一支也轮到了：{calls:?}"
+    );
+}
+
+/// 待发在这一拍被整理排空：同一次调用先排空、再整理，排空的理由是整理之前那一种。
+#[test]
+fn pending_changes_are_published_before_the_compaction_of_the_same_call() {
+    let pool = pool_with_holes_in_the_data_segments();
+    let clock = Arc::new(ClockMovedByTheTest::default());
+    let receiver = Arc::new(RecordingTheCompactionCalls::default());
+    let RuntimeMounted { runtime, .. } = mounted_without_a_thread(
+        pool,
+        parameters_over_the_watermark(),
+        Arc::clone(&clock),
+        Arc::clone(&receiver) as Arc<dyn ReceiverOfTheMaintenanceEvents>,
+    );
+    let content = b"a change kept pending until the compaction starts".to_vec();
+    runtime
+        .accept(UserChange::Overwrite(FirstFile {
+            content: &content,
+            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 4,
+        }))
+        .expect("收进待发");
+    runtime.run_one_maintenance_pass().expect("运行库没坏");
+    assert_eq!(
+        receiver
+            .reasons_of_the_pending_publishes
+            .lock()
+            .expect("这把锁里不 panic")
+            .first()
+            .copied(),
+        Some(ReasonOfThePublish::PendingChangesPublishedBeforeTheBackgroundCompaction),
+        "整理起跑之前排空"
+    );
+    let pending_left = runtime
+        .with_the_session_and_devices(|locked| locked.session.pending_changes_accepted())
+        .expect("运行库没坏");
+    assert_eq!(pending_left, 0);
+}
+
+/// 前台有 fsync 在等：一批都不发（整理让路，这一趟留着，让路结束之后接着推）。
+#[test]
+fn a_waiting_fsync_holds_back_the_next_compaction_publish() {
+    let pool = pool_with_holes_in_the_data_segments();
+    let clock = Arc::new(ClockMovedByTheTest::default());
+    let RuntimeMounted { runtime, .. } = mounted_without_a_thread(
+        pool,
+        parameters_over_the_watermark(),
+        Arc::clone(&clock),
+        Arc::new(IgnoringTheMaintenanceEvents),
+    );
+    clock.advance_by(ONE_PERIOD);
+    {
+        let _waiting = runtime.a_foreground_fsync_is_waiting();
+        let held_back = runtime.run_one_maintenance_pass().expect("运行库没坏");
+        assert_eq!(
+            held_back.compaction_publishes, 0,
+            "一批都不发：{held_back:?}"
+        );
+        assert!(held_back.is_a_trip_under_way, "这一趟留着：{held_back:?}");
+        assert_eq!(
+            runtime.observations().last_stop,
+            Some(CompactionRunStop::YieldedToAForegroundFsync)
+        );
+    }
+    let after_the_fsync = runtime.run_one_maintenance_pass().expect("运行库没坏");
+    assert!(
+        after_the_fsync.compaction_publishes > 0,
+        "fsync 走了，不等一个周期接着推：{after_the_fsync:?}"
+    );
+}
+
+/// 一批写盘报错：发布路径冻结，前台的写被拒、看得到；下一拍重发，前台接着写。
+#[test]
+fn a_frozen_publish_is_visible_to_the_foreground_and_resent_by_the_next_pass() {
+    let pool = pool_with_holes_in_the_data_segments();
+    let faults: DeviceFaults = pool.faults.clone();
+    let clock = Arc::new(ClockMovedByTheTest::default());
+    let RuntimeMounted { runtime, .. } = mounted_without_a_thread(
+        pool,
+        parameters_over_the_watermark(),
+        Arc::clone(&clock),
+        Arc::new(IgnoringTheMaintenanceEvents),
+    );
+    faults.writes_left_to_fail.store(1, Ordering::SeqCst);
+    clock.advance_by(ONE_PERIOD);
+    runtime.run_one_maintenance_pass().expect("运行库没坏");
+    let is_frozen = runtime
+        .with_the_session_and_devices(|locked| {
+            locked
+                .session
+                .session()
+                .allocator
+                .frozen_publish()
+                .is_some()
+        })
+        .expect("运行库没坏");
+    assert!(is_frozen, "写盘报错把发布路径冻结了");
+    let content = b"a foreground write while the publish is frozen".to_vec();
+    let change = UserChange::Overwrite(FirstFile {
+        content: &content,
+        write_time_seconds: FIXED_WRITE_TIME_SECONDS + 5,
+    });
+    assert!(
+        matches!(
+            runtime.accept(change),
+            Err(RuntimeRefused::Refused(
+                AcceptRefused::PublishFrozenAfterAWriteFailureIsNotResentYet { .. }
+            ))
+        ),
+        "前台看得到冻结"
+    );
+    clock.advance_by(ONE_PERIOD);
+    runtime.run_one_maintenance_pass().expect("运行库没坏");
+    assert_eq!(
+        runtime.observations().frozen_publishes_resent,
+        1,
+        "下一拍重发了"
+    );
+    runtime.accept(change).expect("重发之后前台接着写");
+}
+
+/// 上一趟不发布地停下之后现行那一版没变：下一拍不重判、一个盘都不读。
+#[test]
+fn a_stop_that_published_nothing_is_not_judged_again_until_the_next_publish() {
+    let pool = pool_after_a_sequential_write_in_an_earlier_mount();
+    let faults = pool.faults.clone();
+    let clock = Arc::new(ClockMovedByTheTest::default());
+    let RuntimeMounted { runtime, .. } = mounted_without_a_thread(
+        pool,
+        parameters_over_the_watermark(),
+        Arc::clone(&clock),
+        Arc::new(IgnoringTheMaintenanceEvents),
+    );
+    runtime
+        .with_the_session_and_devices(|locked| {
+            let session = locked.session.session_for_tests_only();
+            let mut switches: CompactionSwitches = session.allocator.compaction().switches;
+            switches.forced_region = Some(
+                CompactionRegion::of_one_cluster_segment(DeviceIdentity(0), SlotNumber(1 << 30))
+                    .expect("64 的倍数"),
+            );
+            session.allocator.set_compaction_switches(switches);
+        })
+        .expect("运行库没坏");
+    clock.advance_by(ONE_PERIOD);
+    runtime.run_one_maintenance_pass().expect("运行库没坏");
+    assert_eq!(
+        runtime.observations().last_stop,
+        Some(CompactionRunStop::NoEligibleSegment),
+        "强制选的那一段不在盘上：停、不发布"
+    );
+    let reads_before = faults.reads.load(Ordering::SeqCst);
+    clock.advance_by(ONE_PERIOD);
+    runtime.run_one_maintenance_pass().expect("运行库没坏");
+    assert_eq!(
+        faults.reads.load(Ordering::SeqCst),
+        reads_before,
+        "现行那一版没变：一个盘都不读"
+    );
+    assert_eq!(
+        runtime.observations().last_not_started,
+        Some(
+            BackgroundCompactionNotStarted::UnchangedSinceAStopThatPublishedNothing {
+                stop: CompactionRunStop::NoEligibleSegment
+            }
+        ),
+        "报得出为什么没重判"
+    );
+    runtime
+        .publish_user_change(UserChange::NewInodes {
+            count: 1,
+            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 12,
+        })
+        .expect("建一个 inode");
+    runtime
+        .with_the_session_and_devices(|locked| {
+            locked
+                .session
+                .session_for_tests_only()
+                .compaction
+                .start_judgement = CompactionStartJudgement::NotDue;
+        })
+        .expect("运行库没坏");
+    let reads_before_the_timer_judges_again = faults.reads.load(Ordering::SeqCst);
+    clock.advance_by(ONE_PERIOD);
+    runtime.run_one_maintenance_pass().expect("运行库没坏");
+    assert!(
+        faults.reads.load(Ordering::SeqCst) > reads_before_the_timer_judges_again,
+        "现行那一版变了：备忘失效，定时兜底那一支照常重判"
+    );
+}
+
+/// 后台入口问的两样：钟摆在 0、前台不等。
+struct CallerNotWaiting;
+
+impl CallerOfTheBackgroundCompaction for CallerNotWaiting {
+    fn clock_reading(&self) -> CallerMonotonicClockReading {
+        CallerMonotonicClockReading(Duration::ZERO)
+    }
+
+    fn foreground_fsync(&self) -> ForegroundFsync {
+        ForegroundFsync::NoneWaiting
+    }
+}
+
+/// 卸载带活意图：不用做完，意图留在盘上；再挂续做推到意图删了。
+#[test]
+fn unmounting_with_a_live_intent_leaves_it_for_the_next_mount_to_continue() {
+    let pool = pool_after_a_sequential_write_in_an_earlier_mount();
+    let clock = Arc::new(ClockMovedByTheTest::default());
+    let RuntimeMounted { runtime, .. } = mounted_without_a_thread(
+        pool,
+        parameters_over_the_watermark(),
+        Arc::clone(&clock),
+        Arc::new(IgnoringTheMaintenanceEvents),
+    );
+    let pacing = CompactionPacing {
+        timer_fallback: CompactionTimerFallback::EveryMilliseconds(NonZeroU64::MIN),
+        write_bandwidth_limit: CompactionWriteBandwidthLimit::Unlimited,
+    };
+    runtime
+        .with_the_session_and_devices(|locked| {
+            let session = locked.session.session_for_tests_only();
+            let fullest_segment = {
+                let mut data_units_by_segment: std::collections::BTreeMap<u64, u64> =
+                    std::collections::BTreeMap::new();
+                for record in current_version_of(session).allocation_records.iter().filter(|record| {
+                    record.device == DeviceIdentity(0) && !record.is_released && record.span_slots == 2
+                }) {
+                    *data_units_by_segment
+                        .entry(record.slot.0 / 64 * 64)
+                        .or_default() += 1;
+                }
+                data_units_by_segment
+                    .into_iter()
+                    .max_by_key(|(_segment, data_units)| *data_units)
+                    .expect("写了数据单元")
+                    .0
+            };
+            let mut switches: CompactionSwitches = session.allocator.compaction().switches;
+            switches.forced_region = Some(
+                CompactionRegion::of_one_cluster_segment(
+                    DeviceIdentity(0),
+                    SlotNumber(fullest_segment),
+                )
+                .expect("按 64 取整的段起点"),
+            );
+            switches.batch_size =
+                CompactionBatchSize::ForcedForTests(CompactionBatchUnits::new(1).expect("至少 1"));
+            session.allocator.set_compaction_switches(switches);
+            for _publish in 0..2 {
+                let after_the_call = locked
+                    .session
+                    .run_compaction_in_the_background_for_at_most_one_publish(
+                        &mut locked.devices,
+                        pacing,
+                        &CallerNotWaiting,
+                    )
+                    .expect("盘表是挂载的那一份");
+                assert!(
+                    matches!(
+                        &after_the_call.call,
+                        BackgroundCompactionCall::Ran { run, .. }
+                            if matches!(run.outcome, CompactionRunOutcome::Stopped(CompactionRunStop::OnePublishOfTheTripDone))
+                    ),
+                    "至多一次发布就返回：{:?}",
+                    after_the_call.call
+                );
+            }
+            assert!(
+                locked.session.session().allocator.compaction().live_intent().is_some(),
+                "签发与一批之后意图活着"
+            );
+        })
+        .expect("运行库没坏");
+    let unmounted = runtime.unmount().expect("带活意图照常卸载");
+    let RuntimeMounted {
+        output,
+        runtime: runtime_of_the_next_mount,
+    } = MountedRuntime::mount_without_the_maintenance_thread(
+        &parameters(),
+        unmounted.devices,
+        RuntimeParameters::by_default(),
+        None,
+        Arc::clone(&clock) as Arc<dyn ClockOfTheRuntime>,
+        Arc::new(IgnoringTheMaintenanceEvents),
+    )
+    .expect("再挂");
+    let continuation = output
+        .compaction_continuation
+        .run
+        .as_ref()
+        .expect("盘上有活意图：挂载续做它");
+    assert!(
+        matches!(
+            continuation.outcome,
+            CompactionRunOutcome::Stopped(CompactionRunStop::NoLiveIntentLeftToContinue)
+        ),
+        "默认批大小下续做推到意图删了：{:?}",
+        continuation.outcome
+    );
+    let live_after_the_mount = runtime_of_the_next_mount
+        .with_the_session_and_devices(|locked| {
+            live_compaction_intent_of_the_entries(
+                &current_version_of(locked.session.session()).logged_operations,
+            )
+        })
+        .expect("运行库没坏");
+    assert_eq!(live_after_the_mount, None, "挂完没有活意图");
+    runtime_of_the_next_mount.unmount().expect("卸载干净");
+}
+
+/// 开卡器拒：在读任何一块盘之前拒绝挂载，盘原样交回。
+struct OpenerRefusing;
+
+impl OpensTheBackgroundGpu for OpenerRefusing {
+    fn open_the_background_gpu(
+        &self,
+        _settings: &BackgroundComputeSettings,
+    ) -> Result<OpenedBackgroundGpu, BackgroundGpuNotOpened> {
+        Err(BackgroundGpuNotOpened::AnEarlierOpeningIsStillHung)
+    }
+}
+
+#[test]
+fn a_refusal_of_the_gpu_opener_hands_the_devices_back_before_any_read() {
+    let pool = pool_after_a_sequential_write_in_an_earlier_mount();
+    let faults = pool.faults.clone();
+    let stream = Arc::clone(&pool.stream);
+    let operations_before = stream.lock().expect("这条线程上").len();
+    let mut runtime_parameters = RuntimeParameters::by_default();
+    runtime_parameters.background_compute.background_compute = BackgroundComputeBackend::Gpu;
+    let reads_before = faults.reads.load(Ordering::SeqCst);
+    let refused = MountedRuntime::mount_without_the_maintenance_thread(
+        &parameters(),
+        pool.devices,
+        runtime_parameters,
+        Some(&OpenerRefusing),
+        Arc::new(ClockMovedByTheTest::default()),
+        Arc::new(IgnoringTheMaintenanceEvents),
+    )
+    .expect_err("开卡器拒：挂载拒");
+    assert!(
+        matches!(
+            refused.reason,
+            RuntimeMountRefusal::BackgroundGpuNotOpened(
+                BackgroundGpuNotOpened::AnEarlierOpeningIsStillHung
+            )
+        ),
+        "{refused:?}"
+    );
+    assert_eq!(refused.devices.len(), 2, "盘原样交回");
+    assert_eq!(
+        faults.reads.load(Ordering::SeqCst),
+        reads_before,
+        "一个盘都没读"
+    );
+    assert_eq!(
+        stream.lock().expect("这条线程上").len(),
+        operations_before,
+        "一个写都没发"
+    );
+}
+
+/// 周期是 0：在读任何一块盘之前拒。
+#[test]
+fn a_period_of_zero_is_refused_before_any_read() {
+    let pool = pool_after_a_sequential_write_in_an_earlier_mount();
+    let faults = pool.faults.clone();
+    let reads_before = faults.reads.load(Ordering::SeqCst);
+    let mut runtime_parameters = RuntimeParameters::by_default();
+    runtime_parameters.compaction_period = Duration::ZERO;
+    let refused = MountedRuntime::mount_without_the_maintenance_thread(
+        &parameters(),
+        pool.devices,
+        runtime_parameters,
+        None,
+        Arc::new(ClockMovedByTheTest::default()),
+        Arc::new(IgnoringTheMaintenanceEvents),
+    )
+    .expect_err("周期是 0：挂载拒");
+    assert!(
+        matches!(refused.reason, RuntimeMountRefusal::PeriodOfZero),
+        "{refused:?}"
+    );
+    assert_eq!(
+        faults.reads.load(Ordering::SeqCst),
+        reads_before,
+        "一个盘都没读"
+    );
+}
+
+fn parameters_with_a_short_period() -> RuntimeParameters {
+    let mut runtime_parameters = parameters_over_the_watermark();
+    runtime_parameters.compaction_period = Duration::from_millis(20);
+    runtime_parameters.publish_poll_period = Duration::from_millis(5);
+    runtime_parameters
+}
+
+/// 真线程真钟：调用方不调任何后台入口，线程自己把全空段腾出来；待发在卸载时排空；卸载之后盘重挂得上。
+#[test]
+fn the_maintenance_thread_compacts_on_its_own_and_unmount_stops_it_and_publishes_the_pending_changes(
+) {
+    let pool = pool_with_holes_in_the_data_segments();
+    let RuntimeMounted { runtime, .. } = MountedRuntime::mount(
+        &parameters(),
+        pool.devices,
+        parameters_with_a_short_period(),
+        None,
+        Arc::new(MonotonicClockOfThisProcess::starting_now()),
+        Arc::new(IgnoringTheMaintenanceEvents),
+    )
+    .expect("全合法的参数：挂载照常做成");
+    let given_up_at = Instant::now() + THREAD_DEADLINE;
+    while runtime.observations().compaction_publishes == 0 {
+        assert!(Instant::now() < given_up_at, "线程起了、调了后台入口");
+        std::thread::yield_now();
+    }
+    let content = b"pending at the unmount".to_vec();
+    runtime
+        .accept(UserChange::Overwrite(FirstFile {
+            content: &content,
+            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 6,
+        }))
+        .expect("收进待发");
+    let unmounted = runtime.unmount().expect("卸载干净");
+    let RuntimeMounted {
+        runtime: runtime_of_the_next_mount,
+        ..
+    } = MountedRuntime::mount_without_the_maintenance_thread(
+        &parameters(),
+        unmounted.devices,
+        RuntimeParameters::by_default(),
+        None,
+        Arc::new(ClockMovedByTheTest::default()),
+        Arc::new(IgnoringTheMaintenanceEvents),
+    )
+    .expect("卸载之后重挂得上");
+    let size_of_the_first_file = runtime_of_the_next_mount
+        .with_the_session_and_devices(|locked| {
+            current_version_of(locked.session.session())
+                .inode_record_of(InodeNumber(FIRST_INODE_NUMBER))
+                .expect("第一个文件还在")
+                .size
+        })
+        .expect("运行库没坏");
+    assert_eq!(
+        size_of_the_first_file,
+        u64::try_from(content.len()).expect("内容长度"),
+        "待发在卸载之前排空了（或线程先发了）：重挂读到的是收下的那一份"
+    );
+}
+
+/// 收到后台入口的头一次结局就 panic：维护线程上的 panic。
+#[derive(Debug, Default)]
+struct PanickingOnTheFirstCompactionCall;
+
+impl ReceiverOfTheMaintenanceEvents for PanickingOnTheFirstCompactionCall {
+    fn maintenance_event(&self, event: &MaintenanceEvent<'_>) {
+        assert!(
+            !matches!(event, MaintenanceEvent::CompactionCallReturned(_)),
+            "用例摆的：维护线程上 panic"
+        );
+    }
+}
+
+/// 线程 panic 等价于崩溃：前台入口拿到错误，不接着用；卸载交回盘。
+#[test]
+fn a_panic_on_the_maintenance_thread_makes_the_foreground_report_the_failure() {
+    let pool = pool_with_holes_in_the_data_segments();
+    let RuntimeMounted { runtime, .. } = MountedRuntime::mount(
+        &parameters(),
+        pool.devices,
+        parameters_with_a_short_period(),
+        None,
+        Arc::new(MonotonicClockOfThisProcess::starting_now()),
+        Arc::new(PanickingOnTheFirstCompactionCall),
+    )
+    .expect("全合法的参数：挂载照常做成");
+    let given_up_at = Instant::now() + THREAD_DEADLINE;
+    while runtime.with_the_session_and_devices(|_locked| ()).is_ok() {
+        assert!(
+            Instant::now() < given_up_at,
+            "线程 panic 之后前台拿得到错误"
+        );
+        std::thread::yield_now();
+    }
+    let refused = runtime.unmount().expect_err("坏了不走卸载");
+    assert!(
+        matches!(
+            *refused,
+            singlefs_mount_runtime::UnmountRefused::Failed { ref devices, .. } if devices.len() == 2
+        ),
+        "盘交回：{refused:?}"
+    );
+}
+
+/// 丢弃只停线程：没走卸载，待发不排空、不写盘。
+#[test]
+fn dropping_the_runtime_stops_the_thread_without_publishing_the_pending_changes() {
+    let pool = pool_after_a_sequential_write_in_an_earlier_mount();
+    let stream = Arc::clone(&pool.stream);
+    let mut runtime_parameters = RuntimeParameters::by_default();
+    runtime_parameters.compaction_period = Duration::from_secs(3600);
+    runtime_parameters.publish_poll_period = Duration::from_secs(3600);
+    let RuntimeMounted { runtime, .. } = MountedRuntime::mount(
+        &parameters(),
+        pool.devices,
+        runtime_parameters,
+        None,
+        Arc::new(MonotonicClockOfThisProcess::starting_now()),
+        Arc::new(IgnoringTheMaintenanceEvents),
+    )
+    .expect("全合法的参数：挂载照常做成");
+    let content = b"never published".to_vec();
+    runtime
+        .accept(UserChange::Overwrite(FirstFile {
+            content: &content,
+            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 7,
+        }))
+        .expect("收进待发");
+    let operations_before = stream.lock().expect("线程没 panic").len();
+    drop(runtime);
+    assert_eq!(
+        stream.lock().expect("线程没 panic").len(),
+        operations_before,
+        "丢弃不写盘（线程睡着，周期一小时）"
+    );
+}
+
+/// 前台在锁里 panic：之后每个入口报 `RuntimeFailed`，维护的一拍也报，不接着用半个更新里的会话；卸载交回盘。
+#[test]
+fn a_panic_under_the_session_lock_fails_every_entry_and_unmount_hands_the_devices_back() {
+    let pool = pool_after_a_sequential_write_in_an_earlier_mount();
+    let RuntimeMounted { runtime, .. } = mounted_without_a_thread(
+        pool,
+        RuntimeParameters::by_default(),
+        Arc::new(ClockMovedByTheTest::default()),
+        Arc::new(IgnoringTheMaintenanceEvents),
+    );
+    let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
+        runtime.with_the_session_and_devices(|_locked| panic!("用例摆的：前台在锁里 panic"))
+    }));
+    assert!(panicked.is_err(), "闭包 panic 了");
+    assert!(
+        matches!(
+            runtime.accept(UserChange::NewInodes {
+                count: 1,
+                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 8,
+            }),
+            Err(RuntimeRefused::Failed(_))
+        ),
+        "前台入口报坏了"
+    );
+    let failure: Result<_, RuntimeFailed> = runtime.run_one_maintenance_pass();
+    assert!(failure.is_err(), "维护的一拍也报坏了");
+    let refused = runtime.unmount().expect_err("坏了不走卸载");
+    assert!(
+        matches!(
+            *refused,
+            singlefs_mount_runtime::UnmountRefused::Failed { ref devices, .. } if devices.len() == 2
+        ),
+        "盘交回：{refused:?}"
+    );
+}
diff --git a/Cargo.toml b/Cargo.toml
index cbc48a9385607dd41063bc49e8a18d4d428f8381..12a82c9be340b26134d74016623c3eec0d82553e 100644
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -13,6 +13,7 @@ members = [
     "crates/singlefs-unit-graph",
     "crates/singlefs-background-gpu",
     "crates/singlefs-device-library",
+    "crates/singlefs-mount-runtime",
 ]
 exclude = ["scripts/experiment-apparatus", "prune-server"]
 
````
