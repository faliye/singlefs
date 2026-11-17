# m4-gpugc-one-vulkan-instance-code-r1：第 98 批产品代码改动全文（2026-11-08）

来自补丁目录 `~/.local/share/singlefs/work/m4-gpugc-one-vulkan-instance-writer-r2/patch/crates.patch`，只抄三份产品代码文件的块，原样照录。

```diff
diff --git a/crates/singlefs-background-gpu/src/background_gpu.rs b/crates/singlefs-background-gpu/src/background_gpu.rs
index c43d65cc2ca8d579ad828970ff2426c350486310..a4072166efe6b9a96432503895227003cb1fa024 100644
--- a/crates/singlefs-background-gpu/src/background_gpu.rs
+++ b/crates/singlefs-background-gpu/src/background_gpu.rs
@@ -58,8 +58,8 @@ use crate::adapter_inventory::{
     discrete_gpus_of_the_enumeration_under_the_named,
     identifiers_of_the_discrete_gpus_in_the_enumeration,
     participating_gpus_admitted_by_the_mount_checks, AdapterDeviceKind, BackgroundGpuSelection,
-    DiscreteGpuListRefused, DiscreteGpuOnThisHost, DiscreteGpusPresentNow,
-    EnumeratedAdapterDescription,
+    DiscreteGpuListRefused, DiscreteGpuOnThisHost, DiscreteGpusOfTheEnumeration,
+    DiscreteGpusPresentNow, EnumeratedAdapterDescription, ParticipatingGpu,
 };
 use crate::checksum_kernels::{
     checksum_outcomes_of_the_units_cut_into_pieces, decoded_kernel_outcomes,
@@ -168,8 +168,12 @@ pub enum GpuFailureCause {
 /// 挂载那一刻开不了后台 GPU（第 4 条：拒绝挂载并报错，不降级、不退回 CPU）。成员按调用方要改哪一样分。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub enum BackgroundGpuOpenRefused {
-    /// 挂载参数配错或 GPU 不在：核心层那一套校验拒的（改参数，或装上卡）。
+    /// 挂载参数配错或 GPU 不在：核心层那一套校验拒的（改参数，或装上卡）。核心层拒成「点名的卡不在卡清单里」的不在这里，在
+    /// [`Self::NamedGpuDeviceNotOnTheListOfThisEnumeration`]。
     SettingsRefused(BackgroundComputeSettingsRefused),
+    /// 点名的卡不在这一次枚举交成的卡清单里（核心层拒成 `NamedGpuDeviceNotInTheInventory`）：连同这一次枚举到的每个适配器与没收进清单的那几张，
+    /// 分得清是这一次枚举里没有这张卡、还是有独显而它交不成一张卡（例：没报总线地址）。
+    NamedGpuDeviceNotOnTheListOfThisEnumeration(NamedGpuDeviceNotOnTheListOfThisEnumeration),
     /// 枚举到的独立显卡交不成一份卡清单（条款没写的形态，第一版不支持）。
     DiscreteGpuListNotSupported(DiscreteGpuListRefused),
     /// 枚举适配器这一步就没成（等 wgpu 交回适配器列表超过了上限）：还没有哪一张卡可点名。
@@ -183,6 +187,33 @@ pub enum BackgroundGpuOpenRefused {
     },
 }
 
+/// 点名的卡不在这一次枚举交成的卡清单里：点的是哪张卡、这一次枚举到的每个适配器（按枚举的次序，不论算不算显卡）、
+/// 给了 `background_gpu_devices` 而没点名、交不成一张卡而没收进清单的那几张独显（各带原因，按枚举的次序；同
+/// [`BackgroundGpuOverTheCards::unnamed_discrete_gpus_left_off_the_list`]）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct NamedGpuDeviceNotOnTheListOfThisEnumeration {
+    pub device: GpuDeviceIdentifier,
+    pub adapters_in_the_enumeration: Vec<AdapterSeenInTheEnumeration>,
+    pub unnamed_left_off_the_list: Vec<DiscreteGpuListRefused>,
+}
+
+/// 枚举到的一个适配器在拒绝里报的两样：哪一类设备与 PCI 总线地址（驱动不报时是空串）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct AdapterSeenInTheEnumeration {
+    pub device_kind: AdapterDeviceKind,
+    pub pci_bus_address: String,
+}
+
+impl AdapterSeenInTheEnumeration {
+    #[must_use]
+    pub fn of_the_description(description: &EnumeratedAdapterDescription) -> Self {
+        Self {
+            device_kind: description.device_kind,
+            pci_bus_address: description.pci_bus_address.clone(),
+        }
+    }
+}
+
 /// 一张参与的卡连这一批最小的一片都装不下：哪张卡、
 /// 那一片要切的条目在这一批里的位置、给它的显存上限（不给是 `None`：最小的一片装不进绑定上限）、
 /// 最小的一片合占的字节数（这张卡至少要剩这么多）。这一批一张卡都还没交活。
@@ -362,7 +393,8 @@ pub struct BackgroundGpuOverTheCards<Card: CardOfTheBackgroundGpu> {
     submissions_since_the_last_take: GpuSubmissionsOfTheBatch,
     gpus_leaked_when_dropped: GpusLeakedWhenTheBackgroundGpuIsDropped,
     memory_leaked_in_this_process: GpuMemoryLeakedInThisProcess,
-    /// 此刻本机在的独显（卡清单每次答之前现查）。产品里它带着枚举出卡的那个 wgpu 实例：排在卡后面，卡都丢了才丢它。
+    /// 此刻本机在的独显（卡清单每次答之前现查）。产品里它带着这个进程的 Vulkan 实例（[`VulkanInstanceOfThisProcess`] 的一份克隆）现枚举：
+    /// 排在卡后面，卡都丢了才丢它（实例本身由调用方留着，活过这一次挂载）。
     discrete_gpus_present_now: Box<dyn DiscreteGpusPresentNow + Send>,
 }
 
@@ -707,17 +739,83 @@ fn submissions_of_a_live_slot_plan(plan: &[RecordSliceWithItsChunks]) -> usize {
         .sum()
 }
 
+/// 这个进程的那一个 wgpu 实例（只开 Vulkan 后端、不带显示句柄）：调用方在进程里建一份、跨多次挂载留着，每次开卡都交同一份
+/// （传法同 [`GpuMemoryLeakedInThisProcess`]）。开卡时的枚举、挂着切换时现查本机在的独显（[`AdaptersEnumeratedByTheInstance`] 拿它的一份克隆，
+/// 克隆指的是同一个实例）、本包测试与实验装置枚举本机适配器都用它，不另建。
+/// 与泄漏的那一份并列、不合进去：泄漏的那一份不要卡也建（不要卡的测试与 CPU 臂随处新建），实例只在要开卡、要枚举本机适配器时才建，
+/// 不开 `tests-on-a-real-gpu` 的测试不建（`Cargo.toml` 的特性说明「不碰 GPU」）。
+///
+/// 为什么一个进程只建一个：同一个进程里新建的实例多了，Vulkan 驱动不一定还认得出独显——第二台上同一个进程里新建到第 12 个实例起，
+/// 枚举只剩软件 Vulkan、三张独显一张都不报（前面的实例都已丢掉），开卡就拒点名的卡不在清单里；只建一个实例、反复枚举，每一次三张都在。
+#[derive(Clone, Debug)]
+pub struct VulkanInstanceOfThisProcess {
+    instance: wgpu::Instance,
+}
+
+impl VulkanInstanceOfThisProcess {
+    /// 建这个进程的那一个实例：调用方一个进程只建这一次、留着它，之后每次开卡都交它。
+    #[must_use]
+    pub fn created_for_this_process() -> Self {
+        Self::with_the_backends(wgpu::Backends::VULKAN)
+    }
+
+    /// 一个后端都不开的实例：不装 Vulkan 加载器、不碰 GPU，枚举交回空列表。只给不要卡的单测走开卡那一条路。
+    #[cfg(test)]
+    fn without_any_backend() -> Self {
+        Self::with_the_backends(wgpu::Backends::empty())
+    }
+
+    fn with_the_backends(backends: wgpu::Backends) -> Self {
+        Self {
+            instance: wgpu::Instance::new(wgpu::InstanceDescriptor {
+                backends,
+                ..wgpu::InstanceDescriptor::new_without_display_handle()
+            }),
+        }
+    }
+
+    /// 这个实例此刻看得到的 Vulkan 适配器，按枚举的次序，在 `wait_limit` 之内等。
+    ///
+    /// # Errors
+    /// 等 wgpu 交回适配器列表超过 `wait_limit`。
+    pub fn vulkan_adapters_enumerated_within(
+        &self,
+        wait_limit: Duration,
+    ) -> Result<Vec<wgpu::Adapter>, GpuFailureCause> {
+        wait_for_the_wgpu_future(
+            self.instance.enumerate_adapters(wgpu::Backends::VULKAN),
+            wait_limit,
+            &SteadyClockOfThisProcess::starting_now(),
+        )
+    }
+}
+
+/// 开卡从哪里取这个进程的 Vulkan 实例：产品交 [`VulkanInstanceOfThisProcess`] 本身；不要卡的单测换上记下取了几次的替身，
+/// 钉住每次开卡都取交进来的那一份、不另建。
+pub trait VulkanInstanceHandedToTheOpening {
+    /// 这一次开卡用的实例（交回的克隆指的是同一个实例）。
+    fn vulkan_instance_for_this_opening(&self) -> VulkanInstanceOfThisProcess;
+}
+
+impl VulkanInstanceHandedToTheOpening for VulkanInstanceOfThisProcess {
+    fn vulkan_instance_for_this_opening(&self) -> VulkanInstanceOfThisProcess {
+        self.clone()
+    }
+}
+
 /// 挂载那一刻：枚举适配器、判卡清单与校验、在本机每一张独立显卡上起设备与各段两份核、量一次提交的主机 CPU 时间（同
-/// [`open_background_gpu_with_the_memory_leaked_in_this_process`]，进程里还一张卡都没泄漏过）。
+/// [`open_background_gpu_with_the_memory_leaked_in_this_process`]，进程里还一张卡都没泄漏过；实例是调用方留着的 `vulkan_instance`）。
 ///
 /// # Errors
-/// [`BackgroundGpuOpenRefused`] 的四种。
+/// [`BackgroundGpuOpenRefused`] 的五种。
 pub fn open_background_gpu(
     selection: &BackgroundGpuSelection,
+    vulkan_instance: &VulkanInstanceOfThisProcess,
 ) -> Result<BackgroundGpu, BackgroundGpuOpenRefused> {
     open_background_gpu_with_the_memory_leaked_in_this_process(
         selection,
         &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+        vulkan_instance,
     )
 }
 
@@ -732,33 +830,26 @@ pub fn open_background_gpu(
 /// 显存上限连最小的一片都装不下的卡照开、不量代价、不参与、报出来（[`BackgroundGpuOverTheCards::gpus_left_out_under_their_memory_limit`]），挂载不因此拒绝；
 /// 参与的卡因此一张都不剩时照样交回，三段每一批都报「参与的卡一张都不剩」（[`BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut`]），
 /// 核心层挂载那一刻就读到、当场报警停着。丢掉它时泄漏的卡记进 `memory_leaked_in_this_process`。
+/// 枚举与之后现查本机在的独显都用 `vulkan_instance_handed_over` 交来的那一个实例（调用方在进程里留着的那一份，[`VulkanInstanceOfThisProcess`]），不另建。
 ///
 /// # Errors
-/// [`BackgroundGpuOpenRefused`] 的四种。
+/// [`BackgroundGpuOpenRefused`] 的五种。
 pub fn open_background_gpu_with_the_memory_leaked_in_this_process(
     selection: &BackgroundGpuSelection,
     memory_leaked_in_this_process: &GpuMemoryLeakedInThisProcess,
+    vulkan_instance_handed_over: &impl VulkanInstanceHandedToTheOpening,
 ) -> Result<BackgroundGpu, BackgroundGpuOpenRefused> {
-    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
-        backends: wgpu::Backends::VULKAN,
-        ..wgpu::InstanceDescriptor::new_without_display_handle()
-    });
-    let adapters = wait_for_the_wgpu_future(
-        instance.enumerate_adapters(wgpu::Backends::VULKAN),
-        selection.background_gpu_wait_limit,
-        &SteadyClockOfThisProcess::starting_now(),
-    )
-    .map_err(|cause| BackgroundGpuOpenRefused::AdapterEnumerationFailed { cause })?;
+    let vulkan_instance = vulkan_instance_handed_over.vulkan_instance_for_this_opening();
+    let adapters = vulkan_instance
+        .vulkan_adapters_enumerated_within(selection.background_gpu_wait_limit)
+        .map_err(|cause| BackgroundGpuOpenRefused::AdapterEnumerationFailed { cause })?;
     let descriptions: Vec<EnumeratedAdapterDescription> =
         adapters.iter().map(description_of_the_adapter).collect();
-    let discrete_gpus = discrete_gpus_of_the_enumeration_under_the_named(
-        &descriptions,
-        selection.background_gpu_devices.as_deref(),
-    )
-    .map_err(BackgroundGpuOpenRefused::DiscreteGpuListNotSupported)?;
-    let participating =
-        participating_gpus_admitted_by_the_mount_checks(selection, &discrete_gpus.listed)
-            .map_err(BackgroundGpuOpenRefused::SettingsRefused)?;
+    let (discrete_gpus, participating) =
+        discrete_gpus_and_participating_gpus_admitted_from_the_enumeration(
+            selection,
+            &descriptions,
+        )?;
     let gpus_opened = gpus_opened_at_the_mount(
         discrete_gpus.listed,
         &participating,
@@ -793,26 +884,72 @@ pub fn open_background_gpu_with_the_memory_leaked_in_this_process(
         gpus_opened.left_out_under_their_memory_limit,
         memory_leaked_in_this_process.clone(),
         Box::new(AdaptersEnumeratedByTheInstance {
-            instance,
+            vulkan_instance,
             wait_limit: selection.background_gpu_wait_limit,
         }),
     ))
 }
 
-/// 挂着切换那一刻现查本机在的独显（[`DiscreteGpusPresentNow`]）：拿开卡的那个 wgpu 实例再枚举一次，在开卡时那个等的上限之内等；
-/// 它也是卡的设备都丢了之后才丢的那个实例。
+/// 同一次枚举（`descriptions`）交成卡清单（给了 `background_gpu_devices` 时只判点名的卡）、照它判挂载那一刻的校验、取参与的卡：判与取读同一份清单。
+/// 核心层拒成「点名的卡不在卡清单里」时，拒绝带上这一次枚举到的每个适配器（类型与总线地址）与没点名、没收进清单的那几张
+/// （[`BackgroundGpuOpenRefused::NamedGpuDeviceNotOnTheListOfThisEnumeration`]）；核心层别的拒照原样交回。
+///
+/// # Errors
+/// [`BackgroundGpuOpenRefused`] 的 `DiscreteGpuListNotSupported`、`NamedGpuDeviceNotOnTheListOfThisEnumeration` 与 `SettingsRefused`。
+fn discrete_gpus_and_participating_gpus_admitted_from_the_enumeration(
+    selection: &BackgroundGpuSelection,
+    descriptions: &[EnumeratedAdapterDescription],
+) -> Result<(DiscreteGpusOfTheEnumeration, Vec<ParticipatingGpu>), BackgroundGpuOpenRefused> {
+    let discrete_gpus = discrete_gpus_of_the_enumeration_under_the_named(
+        descriptions,
+        selection.background_gpu_devices.as_deref(),
+    )
+    .map_err(BackgroundGpuOpenRefused::DiscreteGpuListNotSupported)?;
+    match participating_gpus_admitted_by_the_mount_checks(selection, &discrete_gpus.listed) {
+        Ok(participating) => Ok((discrete_gpus, participating)),
+        Err(BackgroundComputeSettingsRefused::NamedGpuDeviceNotInTheInventory { device }) => {
+            Err(BackgroundGpuOpenRefused::NamedGpuDeviceNotOnTheListOfThisEnumeration(
+                NamedGpuDeviceNotOnTheListOfThisEnumeration {
+                    device,
+                    adapters_in_the_enumeration: descriptions
+                        .iter()
+                        .map(AdapterSeenInTheEnumeration::of_the_description)
+                        .collect(),
+                    unnamed_left_off_the_list: discrete_gpus.unnamed_left_off_the_list,
+                },
+            ))
+        }
+        Err(
+            refused @ (BackgroundComputeSettingsRefused::GpuNotPresent
+            | BackgroundComputeSettingsRefused::MemoryLimitCountDifferentFromTheParticipatingDevices {
+                ..
+            }
+            | BackgroundComputeSettingsRefused::MemoryLimitAboveTheTotalMemoryOfTheDevice { .. }
+            | BackgroundComputeSettingsRefused::EmptyGpuDeviceListWhoseMeaningIsUndecided
+            | BackgroundComputeSettingsRefused::GpuDeviceNamedMoreThanOnceWhoseMeaningIsUndecided {
+                ..
+            }
+            | BackgroundComputeSettingsRefused::GpuWaitLimitOfZero
+            | BackgroundComputeSettingsRefused::ParticipatingGpuDeviceNotOpenedAtTheMount(_)
+            | BackgroundComputeSettingsRefused::GcCpuCompensationThresholdAboveTheWholeCapacity {
+                ..
+            }),
+        ) => Err(BackgroundGpuOpenRefused::SettingsRefused(refused)),
+    }
+}
+
+/// 挂着切换那一刻现查本机在的独显（[`DiscreteGpusPresentNow`]）：拿开卡用的那一份进程级 Vulkan 实例再枚举一次，在开卡时那个等的上限之内等。
 struct AdaptersEnumeratedByTheInstance {
-    instance: wgpu::Instance,
+    vulkan_instance: VulkanInstanceOfThisProcess,
     wait_limit: Duration,
 }
 
 impl DiscreteGpusPresentNow for AdaptersEnumeratedByTheInstance {
     fn identifiers_of_the_discrete_gpus_present_now(&self) -> Vec<GpuDeviceIdentifier> {
-        let Ok(adapters) = wait_for_the_wgpu_future(
-            self.instance.enumerate_adapters(wgpu::Backends::VULKAN),
-            self.wait_limit,
-            &SteadyClockOfThisProcess::starting_now(),
-        ) else {
+        let Ok(adapters) = self
+            .vulkan_instance
+            .vulkan_adapters_enumerated_within(self.wait_limit)
+        else {
             // 枚举不成（等 wgpu 超过上限）：一张都不算在，切换照「GPU 不在」拒（DiscreteGpusPresentNow 的约定）。
             return Vec::new();
         };
@@ -1658,7 +1795,7 @@ impl CardOfTheBackgroundGpu for GpuInUse {
 impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
     /// 挂载那一刻开卡之后：开成的卡（与 `discrete_gpus_after_the_opening.opened` 一一对应、同序）、开卡之后的本机独显、没点名而没收进清单的那几张、
     /// 开卡那一组参与的卡与因显存上限不用的卡（`gpus_of_this_host::gpus_opened_at_the_mount` 交回的），进程里泄漏的那一份（丢它时记进去），
-    /// 与卡清单每次答之前现查本机在的独显的那一样（产品带着开卡的 wgpu 实例）。测试拿它把替身卡装进来。
+    /// 与卡清单每次答之前现查本机在的独显的那一样（产品带着开卡用的那一份进程级 Vulkan 实例）。测试拿它把替身卡装进来。
     ///
     /// # Panics
     /// 开成的卡与 `discrete_gpus_after_the_opening.opened` 张数不等。
@@ -4460,4 +4597,164 @@ mod tests {
             "5 个字节的内容按整字补成 8 字节，与切批记账同一种算法"
         );
     }
+
+    /// 开卡取实例的替身：交回一个不开任何后端的实例（枚举交回空列表，不碰 GPU），记下开卡取了它几次。
+    struct VulkanInstanceCountingTheOpenings {
+        instance_without_any_backend: VulkanInstanceOfThisProcess,
+        openings_that_took_it: Cell<usize>,
+    }
+
+    impl VulkanInstanceHandedToTheOpening for VulkanInstanceCountingTheOpenings {
+        fn vulkan_instance_for_this_opening(&self) -> VulkanInstanceOfThisProcess {
+            self.openings_that_took_it
+                .set(self.openings_that_took_it.get() + 1);
+            self.instance_without_any_backend.clone()
+        }
+    }
+
+    /// 同一个进程里开卡几次的轮数：开卡每次另建实例时取交来的那一份的次数是 0，与它不等。
+    const OPENINGS_ON_THE_HANDED_OVER_INSTANCE: usize = 3;
+
+    fn selection_naming_the_card_at(pci_bus_address: &str) -> BackgroundGpuSelection {
+        BackgroundGpuSelection {
+            background_gpu_devices: Some(vec![card_identifier(pci_bus_address)]),
+            background_gpu_memory_limit: None,
+            background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
+        }
+    }
+
+    /// 同一个进程里反复开卡，每一次都从调用方交来的那一份取实例、不另建：替身记下的次数等于开卡的次数。替身的实例不开后端、枚举是空的，
+    /// 点名的卡每一次都拒成「不在这一次枚举的清单里」、枚举到的适配器一个都没有（不要卡，也不建 Vulkan 实例）。
+    #[test]
+    fn every_opening_in_one_process_takes_the_vulkan_instance_handed_over_instead_of_creating_one()
+    {
+        let vulkan_instance = VulkanInstanceCountingTheOpenings {
+            instance_without_any_backend: VulkanInstanceOfThisProcess::without_any_backend(),
+            openings_that_took_it: Cell::new(0),
+        };
+        let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
+        let selection = selection_naming_the_card_at("0000:ff:1f.7");
+        for opening_round in 0..OPENINGS_ON_THE_HANDED_OVER_INSTANCE {
+            let refused = open_background_gpu_with_the_memory_leaked_in_this_process(
+                &selection,
+                &memory_leaked,
+                &vulkan_instance,
+            )
+            .err();
+            assert_eq!(
+                refused,
+                Some(
+                    BackgroundGpuOpenRefused::NamedGpuDeviceNotOnTheListOfThisEnumeration(
+                        NamedGpuDeviceNotOnTheListOfThisEnumeration {
+                            device: card_identifier("0000:ff:1f.7"),
+                            adapters_in_the_enumeration: Vec::new(),
+                            unnamed_left_off_the_list: Vec::new(),
+                        }
+                    )
+                ),
+                "第 {opening_round} 轮（从 0 数）开卡在交来的不开后端的实例上枚举，一个适配器都没有，点名的卡不在清单里"
+            );
+        }
+        assert_eq!(
+            vulkan_instance.openings_that_took_it.get(),
+            OPENINGS_ON_THE_HANDED_OVER_INSTANCE,
+            "开了 {OPENINGS_ON_THE_HANDED_OVER_INSTANCE} 次卡，每一次都取交来的那一份实例（不另建）"
+        );
+    }
+
+    /// 点名一张卡开包时的那一次枚举里一张独显都没有（只有软件 Vulkan）。
+    fn enumeration_with_only_the_software_vulkan() -> Vec<EnumeratedAdapterDescription> {
+        vec![EnumeratedAdapterDescription {
+            name: "llvmpipe software rasterizer".to_string(),
+            device_kind: AdapterDeviceKind::SoftwareRenderingOnTheCpu,
+            pci_bus_address: String::new(),
+            device_local_memory: None,
+        }]
+    }
+
+    /// 点名的卡这一次枚举里根本没有：拒绝里列出枚举到的那一个软件 Vulkan，没有一张独显被留在清单外。
+    #[test]
+    fn naming_a_card_absent_from_the_enumeration_is_refused_listing_the_adapters_seen_and_none_left_off(
+    ) {
+        assert_eq!(
+            discrete_gpus_and_participating_gpus_admitted_from_the_enumeration(
+                &selection_naming_the_card_at("0000:01:00.0"),
+                &enumeration_with_only_the_software_vulkan(),
+            ),
+            Err(
+                BackgroundGpuOpenRefused::NamedGpuDeviceNotOnTheListOfThisEnumeration(
+                    NamedGpuDeviceNotOnTheListOfThisEnumeration {
+                        device: card_identifier("0000:01:00.0"),
+                        adapters_in_the_enumeration: vec![AdapterSeenInTheEnumeration {
+                            device_kind: AdapterDeviceKind::SoftwareRenderingOnTheCpu,
+                            pci_bus_address: String::new(),
+                        }],
+                        unnamed_left_off_the_list: Vec::new(),
+                    }
+                )
+            ),
+            "枚举里没有这张卡：只列出枚举到的软件 Vulkan，清单外一张都没有"
+        );
+    }
+
+    /// 枚举里有一张独显而它没报总线地址：拒绝里列出这张独显（地址是空串）与它被留在清单外的原因，与「枚举里没有」分得开。
+    #[test]
+    fn naming_a_card_while_the_discrete_gpu_reports_no_bus_address_is_refused_listing_that_gpu_left_off(
+    ) {
+        let mut adapters = vec![EnumeratedAdapterDescription {
+            name: "discrete graphics without an address".to_string(),
+            device_kind: AdapterDeviceKind::DiscreteGpu,
+            pci_bus_address: String::new(),
+            device_local_memory: Some(GpuMemoryInBytes(EIGHT_GIBIBYTES)),
+        }];
+        adapters.extend(enumeration_with_only_the_software_vulkan());
+        assert_eq!(
+            discrete_gpus_and_participating_gpus_admitted_from_the_enumeration(
+                &selection_naming_the_card_at("0000:01:00.0"),
+                &adapters,
+            ),
+            Err(
+                BackgroundGpuOpenRefused::NamedGpuDeviceNotOnTheListOfThisEnumeration(
+                    NamedGpuDeviceNotOnTheListOfThisEnumeration {
+                        device: card_identifier("0000:01:00.0"),
+                        adapters_in_the_enumeration: vec![
+                            AdapterSeenInTheEnumeration {
+                                device_kind: AdapterDeviceKind::DiscreteGpu,
+                                pci_bus_address: String::new(),
+                            },
+                            AdapterSeenInTheEnumeration {
+                                device_kind: AdapterDeviceKind::SoftwareRenderingOnTheCpu,
+                                pci_bus_address: String::new(),
+                            },
+                        ],
+                        unnamed_left_off_the_list: vec![
+                            DiscreteGpuListRefused::DiscreteGpuWithoutAPciBusAddressIsNotSupported {
+                                name: "discrete graphics without an address".to_string(),
+                            }
+                        ],
+                    }
+                )
+            ),
+            "有独显而它没报总线地址：列出这张独显（地址空串）与它没收进清单的原因"
+        );
+    }
+
+    /// 核心层别的拒照原样交回：一张独显都没有、也没点名时是「GPU 不在」。
+    #[test]
+    fn no_discrete_gpu_without_named_cards_is_refused_as_gpu_not_present_through_the_settings() {
+        assert_eq!(
+            discrete_gpus_and_participating_gpus_admitted_from_the_enumeration(
+                &BackgroundGpuSelection {
+                    background_gpu_devices: None,
+                    background_gpu_memory_limit: None,
+                    background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
+                },
+                &enumeration_with_only_the_software_vulkan(),
+            ),
+            Err(BackgroundGpuOpenRefused::SettingsRefused(
+                BackgroundComputeSettingsRefused::GpuNotPresent
+            )),
+            "没点名、一张独显都没有：照核心层的「GPU 不在」交回"
+        );
+    }
 }
diff --git a/crates/singlefs-background-gpu/src/background_gpu_compute.rs b/crates/singlefs-background-gpu/src/background_gpu_compute.rs
index 691c917d3db21ba4093412fa3d4d6c7589c1c8a4..7f50f793e0417b4f7d9cbab8d9d865963de338ed 100644
--- a/crates/singlefs-background-gpu/src/background_gpu_compute.rs
+++ b/crates/singlefs-background-gpu/src/background_gpu_compute.rs
@@ -35,7 +35,7 @@ use crate::background_gpu::{
     open_background_gpu_with_the_memory_leaked_in_this_process, BackgroundGpu,
     BackgroundGpuOpenRefused, BackgroundGpuOverTheCards, BatchNotSplitOverTheGpus,
     CardOfTheBackgroundGpu, ChecksumBatchFailed, GpuFailureCause, LiveSlotBatchFailed,
-    RegionRecordBatchFailed, SmallestPieceNotHeldOnTheGpu,
+    RegionRecordBatchFailed, SmallestPieceNotHeldOnTheGpu, VulkanInstanceOfThisProcess,
 };
 use crate::checksum_kernels::{
     reconciled_unit_checksums, ChecksumKernelOutcome, UnitChecksumToVerify,
@@ -192,17 +192,21 @@ pub struct OpenedBackgroundGpuCompute {
 /// 核心层挂载时照这份实现答的卡清单（此刻本机在的独显，连同没开成的那几张）再校验一次，并把校验过的那一组交过来
 /// （`use_the_settings_from_the_next_batch`），之后每次切换生效时另交，参与的卡、显存上限、等的上限都照核心层交来的那一组。
 /// `memory_leaked_in_this_process` 是调用方在进程里留着的那一份：此前几次挂载泄漏在一张卡上的字节先从它的余量里扣掉，这一次卸载泄漏的记进去。
+/// `vulkan_instance` 同样是调用方在进程里留着的那一份（[`VulkanInstanceOfThisProcess`]）：开卡的枚举与之后现查本机在的独显都用它，不另建。
 ///
 /// # Errors
-/// [`BackgroundGpuOpenRefused`] 的四种：设置被拒（GPU 不在、点名的卡不存在、上限个数不对、上限超过显存总量……）、参与的独显交不成清单、
-/// 枚举适配器超过等 GPU 的上限、一张参与的卡用不上。显存上限装不下最小的一片的卡不拒（见 [`OpenedBackgroundGpuCompute`]）。
+/// [`BackgroundGpuOpenRefused`] 的五种：设置被拒（GPU 不在、上限个数不对、上限超过显存总量……）、点名的卡不在这一次枚举的清单里
+/// （带这一次枚举到的适配器与没收进清单的独显）、参与的独显交不成清单、枚举适配器超过等 GPU 的上限、一张参与的卡用不上。
+/// 显存上限装不下最小的一片的卡不拒（见 [`OpenedBackgroundGpuCompute`]）。
 pub fn open_background_gpu_compute(
     selection: &BackgroundGpuSelection,
     memory_leaked_in_this_process: &GpuMemoryLeakedInThisProcess,
+    vulkan_instance: &VulkanInstanceOfThisProcess,
 ) -> Result<OpenedBackgroundGpuCompute, BackgroundGpuOpenRefused> {
     let background_gpu = open_background_gpu_with_the_memory_leaked_in_this_process(
         selection,
         memory_leaked_in_this_process,
+        vulkan_instance,
     )?;
     let gpus_not_opened_at_the_mount = background_gpu.gpus_not_opened_at_the_mount().to_vec();
     let gpus_left_out_under_their_memory_limit = background_gpu
diff --git a/crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs b/crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs
index cf60c2ddbdd32fc0cf10b05820209f959b1a70ae..7aa2bd40594f385489f655bcba00d2622fa700d5 100644
--- a/crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs
+++ b/crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs
@@ -46,7 +46,7 @@ use singlefs_background_gpu::adapter_inventory::{
 };
 use singlefs_background_gpu::background_gpu::{
     host_cpu_time_of_this_thread, monotonic_raw_time_now,
-    open_background_gpu_with_the_memory_leaked_in_this_process, wait_for_the_wgpu_future,
+    open_background_gpu_with_the_memory_leaked_in_this_process, VulkanInstanceOfThisProcess,
 };
 use singlefs_background_gpu::background_gpu_compute::{
     open_background_gpu_compute, BackgroundGpuComputeOnTheParticipatingGpus,
@@ -55,7 +55,6 @@ use singlefs_background_gpu::gpus_of_this_host::{
     GpuMemoryLeakedInThisProcess, GpusLeakedWhenTheBackgroundGpuIsDropped,
 };
 use singlefs_background_gpu::region_record_kernels::cpu_reference_classification;
-use singlefs_background_gpu::waiting_for_the_gpu::SteadyClockOfThisProcess;
 use singlefs_core::address::{
     CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber, SlotNumber,
     TreeIdentifier,
@@ -2747,19 +2746,14 @@ struct DiscreteGpuSeenByThisProcess {
     model: String,
 }
 
-/// 本进程看得到的独立显卡，按 PCI 地址次序（产品没点名时填满的次序）：另起一个只开 Vulkan 的 wgpu 实例枚举一次，
+/// 本进程看得到的独立显卡，按 PCI 地址次序（产品没点名时填满的次序）：在这个进程的那一份 Vulkan 实例（开卡也用它）上枚举一次，
 /// 判法同产品（集显、虚拟显卡、软件 Vulkan、认不出类型的与没有总线地址的不算）。
-fn discrete_gpus_seen_by_this_process() -> Result<Vec<DiscreteGpuSeenByThisProcess>, String> {
-    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
-        backends: wgpu::Backends::VULKAN,
-        ..wgpu::InstanceDescriptor::new_without_display_handle()
-    });
-    let adapters = wait_for_the_wgpu_future(
-        instance.enumerate_adapters(wgpu::Backends::VULKAN),
-        LOCAL_GPU_WAIT_LIMIT_BY_DEFAULT,
-        &SteadyClockOfThisProcess::starting_now(),
-    )
-    .map_err(|cause| format!("枚举适配器没在等的上限之内交回：{cause:?}"))?;
+fn discrete_gpus_seen_by_this_process(
+    vulkan_instance: &VulkanInstanceOfThisProcess,
+) -> Result<Vec<DiscreteGpuSeenByThisProcess>, String> {
+    let adapters = vulkan_instance
+        .vulkan_adapters_enumerated_within(LOCAL_GPU_WAIT_LIMIT_BY_DEFAULT)
+        .map_err(|cause| format!("枚举适配器没在等的上限之内交回：{cause:?}"))?;
     let mut discrete_gpus: Vec<DiscreteGpuSeenByThisProcess> = adapters
         .iter()
         .map(wgpu::Adapter::get_info)
@@ -4232,11 +4226,20 @@ struct GpuOfTheRun {
     gpus_leaked_when_dropped: Option<GpusLeakedWhenTheBackgroundGpuIsDropped>,
 }
 
-/// 照这条臂开卡（挂载与开卡都在第一次 GC 调用之前做完、在窗口外：登记第一节「不含挂载那一刻量代价」）。
+/// GPU 臂开卡用的那一份进程级 Vulkan 实例（这条命令枚举独显时建的）；这条命令没建（不枚举独显）交回 `Err`。
+fn vulkan_instance_for_the_gpu_arm(
+    vulkan_instance: Option<&VulkanInstanceOfThisProcess>,
+) -> Result<&VulkanInstanceOfThisProcess, String> {
+    vulkan_instance
+        .ok_or_else(|| "GPU 臂要开卡，这条命令却没建进程级 Vulkan 实例（没枚举独显）".to_string())
+}
+
+/// 照这条臂开卡（挂载与开卡都在第一次 GC 调用之前做完、在窗口外：登记第一节「不含挂载那一刻量代价」），在 `vulkan_instance` 上开。
 fn open_the_gpu_of_the_arm(
     arm: LockstepArm,
     settings: &BackgroundComputeSettings,
     memory_leaked: &GpuMemoryLeakedInThisProcess,
+    vulkan_instance: Option<&VulkanInstanceOfThisProcess>,
 ) -> Result<GpuOfTheRun, String> {
     if !arm.compute.uses_the_gpu() {
         return Ok(GpuOfTheRun {
@@ -4245,10 +4248,11 @@ fn open_the_gpu_of_the_arm(
             gpus_leaked_when_dropped: None,
         });
     }
+    let vulkan_instance = vulkan_instance_for_the_gpu_arm(vulkan_instance)?;
     let selection = BackgroundGpuSelection::of_the_settings(settings);
     match arm.rejudging {
         RejudgingOfTheArm::NotRejudged => {
-            let opened = open_background_gpu_compute(&selection, memory_leaked)
+            let opened = open_background_gpu_compute(&selection, memory_leaked, vulkan_instance)
                 .map_err(|refused| format!("开后台 GPU：{refused:?}"))?;
             Ok(GpuOfTheRun {
                 gpu_compute: Some(opened.gpu_compute),
@@ -4260,6 +4264,7 @@ fn open_the_gpu_of_the_arm(
             let background_gpu = open_background_gpu_with_the_memory_leaked_in_this_process(
                 &selection,
                 memory_leaked,
+                vulkan_instance,
             )
             .map_err(|refused| format!("开后台 GPU：{refused:?}"))?;
             let gpus_leaked_when_dropped = background_gpu.gpus_leaked_when_dropped();
@@ -4944,6 +4949,8 @@ struct LockstepRunSetting<'run> {
     image: &'run StartingImage,
     discrete_gpus: &'run [DiscreteGpuSeenByThisProcess],
     memory_leaked: &'run GpuMemoryLeakedInThisProcess,
+    /// 开卡用的那一份进程级 Vulkan 实例；只跑 CPU 臂的没有。
+    vulkan_instance: Option<&'run VulkanInstanceOfThisProcess>,
     user_changes: u64,
     batch_units: usize,
 }
@@ -4956,7 +4963,12 @@ fn run_lockstep_once(
 ) -> Result<LockstepRunSummary, String> {
     let host_at_the_start = host_load_reading()?;
     let settings = background_compute_settings_of_the_arm(arm.compute, setting.discrete_gpus);
-    let gpu = open_the_gpu_of_the_arm(arm, &settings, setting.memory_leaked)?;
+    let gpu = open_the_gpu_of_the_arm(
+        arm,
+        &settings,
+        setting.memory_leaked,
+        setting.vulkan_instance,
+    )?;
     let (session, devices, recording) = measuring_mount(
         setting.image,
         settings,
@@ -5554,18 +5566,21 @@ fn run_one_lockstep_repeat(
 
 // ---- 子命令共用的开头：产物、开跑检查、crates 版本（停机条款第 2 条：与产物里已有的比）、卡 ----
 
-/// 子命令开头读到的：产物里已有的结果行、这是第几次起这条命令、合起来的 crates 摘要、本进程看得到的独显。
+/// 子命令开头读到的：产物里已有的结果行、这是第几次起这条命令、合起来的 crates 摘要、本进程看得到的独显，
+/// 与枚举它们的那一份进程级 Vulkan 实例（这条命令里开卡都用它，不另建；不枚举独显的命令不建，是 `None`）。
 struct OpeningOfTheCommand {
     lines_already_in_the_product: Vec<KindAndFieldsOfAResultLine>,
     session_number: u64,
     registered_heavy_threads: usize,
     discrete_gpus: Vec<DiscreteGpuSeenByThisProcess>,
+    vulkan_instance: Option<VulkanInstanceOfThisProcess>,
 }
 
-/// 这条命令要不要枚举本进程看得到的独显（几何核不要卡）。封闭集合。
-#[derive(Clone, Copy, Debug, PartialEq, Eq)]
-enum GpusOfTheCommand {
-    Enumerated,
+/// 这条命令要不要枚举本进程看得到的独显（几何核不要卡）：要的带着这个进程的那一份 Vulkan 实例（`main` 照要卡的子命令建，一个进程一份，
+/// 枚举与这条命令里开卡都用它）。封闭集合。
+#[derive(Clone, Copy, Debug)]
+enum GpusOfTheCommand<'process> {
+    EnumeratedOn(&'process VulkanInstanceOfThisProcess),
     NotNeeded,
 }
 
@@ -5573,7 +5588,7 @@ fn opening_of_the_command(
     product: &mut ProductWriter,
     product_path: &Path,
     command_fields: &str,
-    gpus: GpusOfTheCommand,
+    gpus: GpusOfTheCommand<'_>,
 ) -> Result<OpeningOfTheCommand, i32> {
     let lines_already_in_the_product =
         result_lines_already_in_the_product(product_path).map_err(|reason| {
@@ -5606,22 +5621,25 @@ fn opening_of_the_command(
         );
         return Err(EXIT_STOPPED);
     }
-    let discrete_gpus = if gpus == GpusOfTheCommand::Enumerated {
-        discrete_gpus_seen_by_this_process().map_err(|reason| {
-            product.emit(
-                "gpus",
-                &format!("enumerated=false reason={}", field_value(&reason)),
-            );
-            EXIT_RUN_CONDITION_NOT_MET
-        })?
-    } else {
-        Vec::new()
+    let (vulkan_instance, discrete_gpus) = match gpus {
+        GpusOfTheCommand::EnumeratedOn(vulkan_instance) => {
+            let discrete_gpus =
+                discrete_gpus_seen_by_this_process(vulkan_instance).map_err(|reason| {
+                    product.emit(
+                        "gpus",
+                        &format!("enumerated=false reason={}", field_value(&reason)),
+                    );
+                    EXIT_RUN_CONDITION_NOT_MET
+                })?;
+            (Some(vulkan_instance.clone()), discrete_gpus)
+        }
+        GpusOfTheCommand::NotNeeded => (None, Vec::new()),
     };
     product.emit(
         "gpus",
         &format!(
             "enumerated={} count={} pci_bus_addresses={} models={}",
-            gpus == GpusOfTheCommand::Enumerated,
+            vulkan_instance.is_some(),
             discrete_gpus.len(),
             field_value(
                 &discrete_gpus
@@ -5644,6 +5662,7 @@ fn opening_of_the_command(
         session_number,
         registered_heavy_threads,
         discrete_gpus,
+        vulkan_instance,
     })
 }
 
@@ -5793,7 +5812,10 @@ fn repeats_already_done(
     (done, numbers, next_repeat)
 }
 
-fn lockstep_command(command: &LockstepCommand) -> i32 {
+fn lockstep_command(
+    command: &LockstepCommand,
+    vulkan_instance: &VulkanInstanceOfThisProcess,
+) -> i32 {
     let mut product = match ProductWriter::appending_to(&command.product) {
         Ok(product) => product,
         Err(reason) => {
@@ -5813,7 +5835,7 @@ fn lockstep_command(command: &LockstepCommand) -> i32 {
         &mut product,
         &command.product,
         &command_fields,
-        GpusOfTheCommand::Enumerated,
+        GpusOfTheCommand::EnumeratedOn(vulkan_instance),
     ) {
         Ok(opening) => opening,
         Err(exit_code) => {
@@ -5867,6 +5889,7 @@ fn lockstep_command(command: &LockstepCommand) -> i32 {
         image: &image,
         discrete_gpus: &opening.discrete_gpus,
         memory_leaked: &memory_leaked,
+        vulkan_instance: opening.vulkan_instance.as_ref(),
         user_changes: command.user_changes,
         batch_units: command.batch_units,
     };
@@ -6561,6 +6584,8 @@ struct ConcurrentRunSetting<'run> {
     image: &'run StartingImage,
     discrete_gpus: &'run [DiscreteGpuSeenByThisProcess],
     memory_leaked: &'run GpuMemoryLeakedInThisProcess,
+    /// 开卡用的那一份进程级 Vulkan 实例；只跑 CPU 臂的没有。
+    vulkan_instance: Option<&'run VulkanInstanceOfThisProcess>,
     user_changes: u64,
     foreground_interval: Duration,
 }
@@ -6574,7 +6599,12 @@ fn run_concurrent_once(
     let host_at_the_start = host_load_reading()?;
     let settings =
         background_compute_settings_of_the_arm(arm.compute_of_the_arm(), setting.discrete_gpus);
-    let gpu = open_the_gpu_of_the_arm(arm.as_a_lockstep_arm(), &settings, setting.memory_leaked)?;
+    let gpu = open_the_gpu_of_the_arm(
+        arm.as_a_lockstep_arm(),
+        &settings,
+        setting.memory_leaked,
+        setting.vulkan_instance,
+    )?;
     let (session, devices, _recording) = measuring_mount(
         setting.image,
         settings,
@@ -7037,7 +7067,10 @@ fn command_fields_of_a_concurrent_command(command: &ConcurrentCommand) -> String
     )
 }
 
-fn concurrent_command(command: &ConcurrentCommand) -> i32 {
+fn concurrent_command(
+    command: &ConcurrentCommand,
+    vulkan_instance: &VulkanInstanceOfThisProcess,
+) -> i32 {
     let mut product = match ProductWriter::appending_to(&command.product) {
         Ok(product) => product,
         Err(reason) => {
@@ -7050,7 +7083,7 @@ fn concurrent_command(command: &ConcurrentCommand) -> i32 {
         &mut product,
         &command.product,
         &command_fields,
-        GpusOfTheCommand::Enumerated,
+        GpusOfTheCommand::EnumeratedOn(vulkan_instance),
     ) {
         Ok(opening) => opening,
         Err(exit_code) => {
@@ -7104,6 +7137,7 @@ fn concurrent_command(command: &ConcurrentCommand) -> i32 {
         image: &image,
         discrete_gpus: &opening.discrete_gpus,
         memory_leaked: &memory_leaked,
+        vulkan_instance: opening.vulkan_instance.as_ref(),
         user_changes: CONCURRENT_USER_CHANGES,
         foreground_interval: command.foreground_interval,
     };
@@ -7224,12 +7258,19 @@ fn sensitivity_concurrent_command(product: &Path) -> ConcurrentCommand {
 }
 
 /// 两点串着跑、写进同一份产物：交替跑那一点不是 0 就不跑并发那一点、交回它的退出码。
-fn sensitivity_command(product: &Path, user_changes: u64) -> i32 {
-    let lockstep_exit = lockstep_command(&sensitivity_lockstep_command(product, user_changes));
+fn sensitivity_command(
+    product: &Path,
+    user_changes: u64,
+    vulkan_instance: &VulkanInstanceOfThisProcess,
+) -> i32 {
+    let lockstep_exit = lockstep_command(
+        &sensitivity_lockstep_command(product, user_changes),
+        vulkan_instance,
+    );
     if lockstep_exit != EXIT_SUCCESS {
         return lockstep_exit;
     }
-    concurrent_command(&sensitivity_concurrent_command(product))
+    concurrent_command(&sensitivity_concurrent_command(product), vulkan_instance)
 }
 
 // ---- 子命令 geometry-check（登记第五节「分段」第一段：只造池、不计时、不要卡；第一节未定第 6 项：量每档造池耗时与每次跑的批数）----
@@ -7250,6 +7291,7 @@ fn geometry_check_of_a_tier(tier: UnitAreaTier) -> GeometryCheckOfATier {
                 image: &image,
                 discrete_gpus: &[],
                 memory_leaked: &memory_leaked,
+                vulkan_instance: None,
                 user_changes,
                 batch_units: BATCH_UNITS_AT_THE_MAIN_SAMPLING_POINT,
             };
@@ -7796,12 +7838,14 @@ fn stage_three_share_of_one_greedy_call(
     arm: ComputeOfTheArm,
     discrete_gpus: &[DiscreteGpuSeenByThisProcess],
     memory_leaked: &GpuMemoryLeakedInThisProcess,
+    vulkan_instance: Option<&VulkanInstanceOfThisProcess>,
 ) -> Result<StageThreeShareOfOneGreedyCall, String> {
     let settings = background_compute_settings_of_the_arm(arm, discrete_gpus);
     let (gpu_compute, timing, gpus_leaked_when_dropped) = if arm.uses_the_gpu() {
         let background_gpu = open_background_gpu_with_the_memory_leaked_in_this_process(
             &BackgroundGpuSelection::of_the_settings(&settings),
             memory_leaked,
+            vulkan_instance_for_the_gpu_arm(vulkan_instance)?,
         )
         .map_err(|refused| format!("开后台 GPU：{refused:?}"))?;
         let gpus_leaked_when_the_gpu_is_dropped = background_gpu.gpus_leaked_when_dropped();
@@ -7888,6 +7932,7 @@ fn stage_three_counts_on_the_arm(
     unit_area: UnitAreaOfOneDevice,
     discrete_gpus: &[DiscreteGpuSeenByThisProcess],
     memory_leaked: &GpuMemoryLeakedInThisProcess,
+    vulkan_instance: Option<&VulkanInstanceOfThisProcess>,
 ) -> Result<StageThreeCountsOfTheArm, String> {
     match arm {
         ComputeOfTheArm::Cpu => stage_three_counts_of_the_counter(
@@ -7900,6 +7945,7 @@ fn stage_three_counts_on_the_arm(
             let background_gpu = open_background_gpu_with_the_memory_leaked_in_this_process(
                 &BackgroundGpuSelection::of_the_settings(&settings),
                 memory_leaked,
+                vulkan_instance_for_the_gpu_arm(vulkan_instance)?,
             )
             .map_err(|refused| format!("开后台 GPU：{refused:?}"))?;
             let gpus_leaked_when_the_gpu_is_dropped = background_gpu.gpus_leaked_when_dropped();
@@ -7939,7 +7985,11 @@ fn allocation_records_and_unit_area_of_the_image(
     ))
 }
 
-fn stage_three_cost_command(tier: UnitAreaTier, product_path: &Path) -> i32 {
+fn stage_three_cost_command(
+    tier: UnitAreaTier,
+    product_path: &Path,
+    vulkan_instance: &VulkanInstanceOfThisProcess,
+) -> i32 {
     let mut product = match ProductWriter::appending_to(product_path) {
         Ok(product) => product,
         Err(reason) => {
@@ -7954,7 +8004,7 @@ fn stage_three_cost_command(tier: UnitAreaTier, product_path: &Path) -> i32 {
             "command=stage-three-cost tier={} repeats={STAGE_THREE_COST_REPEATS}",
             tier.name()
         ),
-        GpusOfTheCommand::Enumerated,
+        GpusOfTheCommand::EnumeratedOn(vulkan_instance),
     ) {
         Ok(opening) => opening,
         Err(exit_code) => {
@@ -8004,6 +8054,7 @@ fn stage_three_cost_command(tier: UnitAreaTier, product_path: &Path) -> i32 {
             unit_area,
             &opening.discrete_gpus,
             &memory_leaked,
+            opening.vulkan_instance.as_ref(),
         )
         .and_then(|counts| {
             stage_three_share_of_one_greedy_call(
@@ -8011,6 +8062,7 @@ fn stage_three_cost_command(tier: UnitAreaTier, product_path: &Path) -> i32 {
                 arm,
                 &opening.discrete_gpus,
                 &memory_leaked,
+                opening.vulkan_instance.as_ref(),
             )
             .map(|share| (counts, share))
         });
@@ -8394,23 +8446,36 @@ fn command_of_the_arguments(arguments: &[String]) -> Result<CommandOfTheApparatu
     Ok(command)
 }
 
+/// 一个进程只起一条子命令；要卡的子命令在这里建这个进程的那一份 Vulkan 实例交下去（敏感度那一条的交替跑与并发跑共用它），不要卡的不建。
 fn main() {
     let arguments: Vec<String> = std::env::args().skip(1).collect();
     let exit_code = match command_of_the_arguments(&arguments) {
         Ok(CommandOfTheApparatus::GeometryCheck { product }) => geometry_check_command(&product),
-        Ok(CommandOfTheApparatus::Lockstep(command)) => lockstep_command(&command),
-        Ok(CommandOfTheApparatus::Concurrent(command)) => concurrent_command(&command),
+        Ok(CommandOfTheApparatus::Lockstep(command)) => lockstep_command(
+            &command,
+            &VulkanInstanceOfThisProcess::created_for_this_process(),
+        ),
+        Ok(CommandOfTheApparatus::Concurrent(command)) => concurrent_command(
+            &command,
+            &VulkanInstanceOfThisProcess::created_for_this_process(),
+        ),
         Ok(CommandOfTheApparatus::Sensitivity {
             product,
             user_changes,
-        }) => sensitivity_command(&product, user_changes),
+        }) => sensitivity_command(
+            &product,
+            user_changes,
+            &VulkanInstanceOfThisProcess::created_for_this_process(),
+        ),
         Ok(CommandOfTheApparatus::JudgeTrend {
             product,
             lockstep_products,
         }) => judge_trend_command(&product, &lockstep_products),
-        Ok(CommandOfTheApparatus::StageThreeCost { tier, product }) => {
-            stage_three_cost_command(tier, &product)
-        }
+        Ok(CommandOfTheApparatus::StageThreeCost { tier, product }) => stage_three_cost_command(
+            tier,
+            &product,
+            &VulkanInstanceOfThisProcess::created_for_this_process(),
+        ),
         Ok(CommandOfTheApparatus::ConstructionTiming(command)) => {
             construction_timing_command(&command)
         }
@@ -9864,6 +9929,7 @@ mod tests {
             image: &image,
             discrete_gpus: &[],
             memory_leaked: &memory_leaked,
+            vulkan_instance: None,
             user_changes: LOCKSTEP_USER_CHANGES_LADDER[0],
             batch_units: BATCH_UNITS_AT_THE_MAIN_SAMPLING_POINT,
         };
@@ -10964,6 +11030,7 @@ mod tests {
             image: &image,
             discrete_gpus: &[],
             memory_leaked: &memory_leaked,
+            vulkan_instance: None,
             user_changes: 8,
             foreground_interval: Duration::from_millis(5),
         };
@@ -11219,9 +11286,14 @@ mod tests {
             ));
         }
         let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
-        let share =
-            stage_three_share_of_one_greedy_call(&image, ComputeOfTheArm::Cpu, &[], &memory_leaked)
-                .expect("CPU 臂挂载得成");
+        let share = stage_three_share_of_one_greedy_call(
+            &image,
+            ComputeOfTheArm::Cpu,
+            &[],
+            &memory_leaked,
+            None,
+        )
+        .expect("CPU 臂挂载得成");
         assert!(share.gc_call_wall > Duration::ZERO, "{share:?}");
         assert_eq!(share.stage_three, None);
     }
@@ -11232,7 +11304,9 @@ mod tests {
     #[test]
     fn the_gpu_arm_on_a_real_card_counts_the_same_live_slots_per_segment_as_the_apparatus_on_the_small_pool(
     ) {
-        let discrete_gpus = discrete_gpus_seen_by_this_process().expect("枚举得到适配器");
+        let vulkan_instance = VulkanInstanceOfThisProcess::created_for_this_process();
+        let discrete_gpus =
+            discrete_gpus_seen_by_this_process(&vulkan_instance).expect("枚举得到适配器");
         assert!(
             !discrete_gpus.is_empty(),
             "带 tests-on-a-real-gpu 跑，本进程要看得到至少一张独显"
@@ -11249,6 +11323,7 @@ mod tests {
             unit_area,
             &discrete_gpus,
             &memory_leaked,
+            Some(&vulkan_instance),
         )
         .expect("GPU 点名一张卡臂数得成");
         assert_eq!(on_the_card.walls.len(), 5);
@@ -11261,6 +11336,7 @@ mod tests {
             ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress,
             &discrete_gpus,
             &memory_leaked,
+            Some(&vulkan_instance),
         )
         .expect("GPU 点名一张卡臂挂载得成");
         let stage_three = share.stage_three.expect("GPU 臂外面套了丙段计时");
```
