# m4-gpugc-code-r2：第 54、55 批补丁里产品代码的块（按打进主工作区的次序原样抄录）

## 第 54 批：`~/.local/share/singlefs/work/m4-gpugc-code-r1-fix-writer/patch/crates.patch`

```diff
diff --git a/crates/singlefs-background-gpu/src/adapter_inventory.rs b/crates/singlefs-background-gpu/src/adapter_inventory.rs
index 8803bf5..d4611f5 100644
--- a/crates/singlefs-background-gpu/src/adapter_inventory.rs
+++ b/crates/singlefs-background-gpu/src/adapter_inventory.rs
@@ -90,42 +90,106 @@ pub fn gpu_device_identifier_of_the_pci_bus_address(pci_bus_address: &str) -> Gp
     GpuDeviceIdentifier::of_opaque_bytes(pci_bus_address.as_bytes().to_vec())
 }
 
-/// 第 2 条「本机全部独立显卡」：只留独立显卡，按 PCI 总线地址的字典序排。
+/// 第 2 条「本机全部独立显卡」：只留独立显卡，按 PCI 总线地址的字典序排；每一张都要交得成（[`discrete_gpus_of_the_enumeration_under_the_named`]
+/// 不给点名的那一种）。
 ///
 /// # Errors
 /// [`DiscreteGpuListRefused`] 的三种，都只看独立显卡；不算显卡的适配器缺什么都不管。
 pub fn discrete_gpus_of_the_enumeration(
     adapters: &[EnumeratedAdapterDescription],
 ) -> Result<Vec<DiscreteGpuOnThisHost>, DiscreteGpuListRefused> {
+    discrete_gpus_of_the_enumeration_under_the_named(adapters, None).map(|listed| listed.listed)
+}
+
+/// 一次枚举交成的卡清单：收进来的独显（按总线地址排），与给了 `background_gpu_devices` 时没点名、交不成一张卡而没收进来的那几张（各带原因，按枚举的次序）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct DiscreteGpusOfTheEnumeration {
+    pub listed: Vec<DiscreteGpuOnThisHost>,
+    pub unnamed_left_off_the_list: Vec<DiscreteGpuListRefused>,
+}
+
+/// 一张独显在这一组点名下算不算参与的：参与的交不成就拒，不参与的交不成就不收、报出来。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum ParticipationUnderTheNamedDevices {
+    /// 没给 `background_gpu_devices`（本机全部独显都参与），或给了而点了它。
+    Participates,
+    /// 给了 `background_gpu_devices` 而没点它（没有总线地址的卡点不到它）。
+    NotNamed,
+}
+
+/// 同 [`discrete_gpus_of_the_enumeration`]，只差一处（第 2 条「点名了就只用点名的卡」）：给了 `background_gpu_devices`（`named_devices`）时只判点名的卡，
+/// 没点名的独显缺总线地址、缺设备本地堆、与别的独显报了同一个总线地址，都不拒、不收进清单，原因报在
+/// [`DiscreteGpusOfTheEnumeration::unnamed_left_off_the_list`] 里（同一个地址报了两次的，那个地址上的几张都不收，报一次）。
+///
+/// # Errors
+/// 参与的卡（没给点名时是每一张独显，给了时是点名的那几张）有 [`DiscreteGpuListRefused`] 的三种之一。
+pub fn discrete_gpus_of_the_enumeration_under_the_named(
+    adapters: &[EnumeratedAdapterDescription],
+    named_devices: Option<&[GpuDeviceIdentifier]>,
+) -> Result<DiscreteGpusOfTheEnumeration, DiscreteGpuListRefused> {
     let mut discrete_gpus: Vec<DiscreteGpuOnThisHost> = Vec::new();
+    let mut unnamed_left_off_the_list = Vec::new();
+    // 没点名而报了两次的总线地址：这个地址上后来的几张也不收。
+    let mut unnamed_addresses_reported_twice: Vec<GpuDeviceIdentifier> = Vec::new();
     for (position_in_the_enumeration, adapter) in adapters.iter().enumerate() {
         if !adapter.device_kind.counts_as_a_background_gpu() {
             continue;
         }
+        let identifier = gpu_device_identifier_of_the_pci_bus_address(&adapter.pci_bus_address);
+        let participation = match named_devices {
+            None => ParticipationUnderTheNamedDevices::Participates,
+            Some(named_devices)
+                if !adapter.pci_bus_address.is_empty() && named_devices.contains(&identifier) =>
+            {
+                ParticipationUnderTheNamedDevices::Participates
+            }
+            Some(_named_devices) => ParticipationUnderTheNamedDevices::NotNamed,
+        };
+        let refused_or_left_off = |refused: DiscreteGpuListRefused,
+                                   left_off: &mut Vec<DiscreteGpuListRefused>|
+         -> Result<(), DiscreteGpuListRefused> {
+            match participation {
+                ParticipationUnderTheNamedDevices::Participates => Err(refused),
+                ParticipationUnderTheNamedDevices::NotNamed => {
+                    left_off.push(refused);
+                    Ok(())
+                }
+            }
+        };
         if adapter.pci_bus_address.is_empty() {
-            return Err(
+            refused_or_left_off(
                 DiscreteGpuListRefused::DiscreteGpuWithoutAPciBusAddressIsNotSupported {
                     name: adapter.name.clone(),
                 },
-            );
+                &mut unnamed_left_off_the_list,
+            )?;
+            continue;
         }
         let Some(total_memory) = adapter.device_local_memory else {
-            return Err(
+            refused_or_left_off(
                 DiscreteGpuListRefused::DiscreteGpuWithoutADeviceLocalMemoryHeapIsNotSupported {
                     name: adapter.name.clone(),
                 },
-            );
+                &mut unnamed_left_off_the_list,
+            )?;
+            continue;
         };
-        let identifier = gpu_device_identifier_of_the_pci_bus_address(&adapter.pci_bus_address);
-        if discrete_gpus
+        if unnamed_addresses_reported_twice.contains(&identifier) {
+            continue;
+        }
+        if let Some(already_listed) = discrete_gpus
             .iter()
-            .any(|already_listed| already_listed.identifier == identifier)
+            .position(|already_listed| already_listed.identifier == identifier)
         {
-            return Err(
+            refused_or_left_off(
                 DiscreteGpuListRefused::PciBusAddressReportedByTwoDiscreteGpusIsNotSupported {
                     pci_bus_address: adapter.pci_bus_address.clone(),
                 },
-            );
+                &mut unnamed_left_off_the_list,
+            )?;
+            discrete_gpus.remove(already_listed);
+            unnamed_addresses_reported_twice.push(identifier);
+            continue;
         }
         discrete_gpus.push(DiscreteGpuOnThisHost {
             position_in_the_enumeration,
@@ -136,7 +200,31 @@ pub fn discrete_gpus_of_the_enumeration(
         });
     }
     discrete_gpus.sort_by(|left, right| left.pci_bus_address.cmp(&right.pci_bus_address));
-    Ok(discrete_gpus)
+    Ok(DiscreteGpusOfTheEnumeration {
+        listed: discrete_gpus,
+        unnamed_left_off_the_list,
+    })
+}
+
+/// 此刻一次枚举里在的独显（有总线地址的）各自的标识，按枚举的次序：挂着切换那一刻现查卡清单（第 4 条「切换那一刻……GPU 不在……就拒绝切换」）读它，
+/// 挂载之后掉了的卡不在里面。
+#[must_use]
+pub fn identifiers_of_the_discrete_gpus_in_the_enumeration(
+    adapters: &[EnumeratedAdapterDescription],
+) -> Vec<GpuDeviceIdentifier> {
+    adapters
+        .iter()
+        .filter(|adapter| {
+            adapter.device_kind.counts_as_a_background_gpu() && !adapter.pci_bus_address.is_empty()
+        })
+        .map(|adapter| gpu_device_identifier_of_the_pci_bus_address(&adapter.pci_bus_address))
+        .collect()
+}
+
+/// 此刻本机在的独显各自的标识（挂着切换那一刻现查）：产品经 wgpu 现枚举一次（外壳 `background_gpu` 里那一份），测试换成假的。
+/// 枚举不成（等 wgpu 超过上限）时交回空：一张都不在，切换照「GPU 不在」拒。
+pub trait DiscreteGpusPresentNow {
+    fn identifiers_of_the_discrete_gpus_present_now(&self) -> Vec<GpuDeviceIdentifier>;
 }
 
 /// 同一次枚举交成核心层的卡清单（`GpuInventory`）：挂载那一刻的校验读它，之后取参与的卡读的也是它。
@@ -606,4 +694,131 @@ mod tests {
             "不给上限就不设限"
         );
     }
+
+    /// 第 2 条「点名了就只用点名的卡」：只点名一张好卡，同一次枚举里没点名的独显一张没报总线地址、一张读不到设备本地堆、两张报了同一个总线地址——
+    /// 都不拒，只收点名的那张，没收的那几张各带原因报出来（攻方探针 T1-2 那一种，改前整张清单交不成、拒挂）。
+    #[test]
+    fn unnamed_discrete_gpus_that_cannot_be_listed_are_left_off_without_refusing_the_named_card() {
+        let mut without_a_heap = adapter(
+            "unnamed card without a heap",
+            AdapterDeviceKind::DiscreteGpu,
+            "0000:21:00.0",
+            8,
+        );
+        without_a_heap.device_local_memory = None;
+        let adapters = vec![
+            adapter(
+                "named card",
+                AdapterDeviceKind::DiscreteGpu,
+                "0000:01:00.0",
+                8,
+            ),
+            adapter(
+                "unnamed card without an address",
+                AdapterDeviceKind::DiscreteGpu,
+                "",
+                8,
+            ),
+            without_a_heap,
+            adapter(
+                "unnamed card seen by one driver",
+                AdapterDeviceKind::DiscreteGpu,
+                "0000:41:00.0",
+                16,
+            ),
+            adapter(
+                "unnamed card seen by another driver",
+                AdapterDeviceKind::DiscreteGpu,
+                "0000:41:00.0",
+                16,
+            ),
+        ];
+        let named = [gpu_device_identifier_of_the_pci_bus_address("0000:01:00.0")];
+        let listed = discrete_gpus_of_the_enumeration_under_the_named(&adapters, Some(&named))
+            .expect("点名的那张交得成：没点名的交不成也不拒");
+        assert_eq!(
+            listed,
+            DiscreteGpusOfTheEnumeration {
+                listed: vec![DiscreteGpuOnThisHost {
+                    position_in_the_enumeration: 0,
+                    identifier: gpu_device_identifier_of_the_pci_bus_address("0000:01:00.0"),
+                    pci_bus_address: "0000:01:00.0".to_string(),
+                    name: "named card".to_string(),
+                    total_memory: GpuMemoryInBytes(8 * GIBIBYTE),
+                }],
+                unnamed_left_off_the_list: vec![
+                    DiscreteGpuListRefused::DiscreteGpuWithoutAPciBusAddressIsNotSupported {
+                        name: "unnamed card without an address".to_string(),
+                    },
+                    DiscreteGpuListRefused::DiscreteGpuWithoutADeviceLocalMemoryHeapIsNotSupported {
+                        name: "unnamed card without a heap".to_string(),
+                    },
+                    DiscreteGpuListRefused::PciBusAddressReportedByTwoDiscreteGpusIsNotSupported {
+                        pci_bus_address: "0000:41:00.0".to_string(),
+                    },
+                ],
+            },
+            "只收点名的那张；没点名的三种都不拒、不收，各报一次"
+        );
+        assert_eq!(
+            discrete_gpus_of_the_enumeration_under_the_named(&adapters, None),
+            Err(
+                DiscreteGpuListRefused::DiscreteGpuWithoutAPciBusAddressIsNotSupported {
+                    name: "unnamed card without an address".to_string(),
+                }
+            ),
+            "没给点名时每一张都参与：同一次枚举照旧拒"
+        );
+    }
+
+    /// 点名的那张自己缺设备本地堆：照旧拒（判的是参与的卡）。
+    #[test]
+    fn a_named_discrete_gpu_without_a_device_local_memory_heap_is_still_refused() {
+        let mut named_without_a_heap = adapter(
+            "named card",
+            AdapterDeviceKind::DiscreteGpu,
+            "0000:01:00.0",
+            8,
+        );
+        named_without_a_heap.device_local_memory = None;
+        let named = [gpu_device_identifier_of_the_pci_bus_address("0000:01:00.0")];
+        assert_eq!(
+            discrete_gpus_of_the_enumeration_under_the_named(&[named_without_a_heap], Some(&named)),
+            Err(
+                DiscreteGpuListRefused::DiscreteGpuWithoutADeviceLocalMemoryHeapIsNotSupported {
+                    name: "named card".to_string()
+                }
+            )
+        );
+    }
+
+    /// 第 4 条「切换那一刻……GPU 不在……就拒绝切换」：现查的那一次枚举里在的独显才算在；挂载之后掉了的卡、集显、没有总线地址的都不在。
+    #[test]
+    fn only_discrete_gpus_with_a_bus_address_in_the_enumeration_count_as_present() {
+        let adapters = vec![
+            adapter(
+                "integrated graphics",
+                AdapterDeviceKind::IntegratedGpu,
+                "0000:00:02.0",
+                2,
+            ),
+            adapter(
+                "card still here",
+                AdapterDeviceKind::DiscreteGpu,
+                "0000:41:00.0",
+                16,
+            ),
+            adapter(
+                "card without an address",
+                AdapterDeviceKind::DiscreteGpu,
+                "",
+                8,
+            ),
+        ];
+        assert_eq!(
+            identifiers_of_the_discrete_gpus_in_the_enumeration(&adapters),
+            vec![gpu_device_identifier_of_the_pci_bus_address("0000:41:00.0")],
+            "挂载时在的「0000:01:00.0」这一次没枚举到：不算在"
+        );
+    }
 }
diff --git a/crates/singlefs-background-gpu/src/background_gpu.rs b/crates/singlefs-background-gpu/src/background_gpu.rs
index 4081042..21850b8 100644
--- a/crates/singlefs-background-gpu/src/background_gpu.rs
+++ b/crates/singlefs-background-gpu/src/background_gpu.rs
@@ -1,28 +1,43 @@
 //! 起 GPU 的外壳（D26（后台整理与放置回收） 已定项 13 第 1–4、8 条）：挂载那一刻枚举 wgpu 适配器、判过校验、在本机每一张独立显卡上起设备与各段两份核的管线、
 //! 量一次提交的主机 CPU 时间；一张卡没开成时，它是参与的卡就拒绝挂载，给了 `background_gpu_devices` 而没点它就跳过、报出来
-//! （[`BackgroundGpu::gpus_not_opened_at_the_mount`]；判法在 `crate::gpus_of_this_host::gpus_opened_at_the_mount`，`records/2026-10-19-里程碑四GPU垃圾回收开工.md`
-//! 第 45 批交回那一行的定法）。参与的卡由设置定（[`BackgroundGpu::participating_gpus`] 报出来），挂着切换时核心层交来新的一组
-//! （[`BackgroundGpu::use_the_settings_from_the_next_batch`]，两批之间），从下一批起只在它点的卡上、照它给的显存上限与等的上限跑，不重开设备
+//! （[`BackgroundGpuOverTheCards::gpus_not_opened_at_the_mount`]；判法在 `crate::gpus_of_this_host::gpus_opened_at_the_mount`，`records/2026-10-19-里程碑四GPU垃圾回收开工.md`
+//! 第 45 批交回那一行的定法）。给了 `background_gpu_devices` 时只判点名的卡：没点名的独显缺总线地址、缺设备本地堆、与别的独显报了同一个地址都不拒，
+//! 不收进卡清单、报出来（[`BackgroundGpuOverTheCards::unnamed_discrete_gpus_left_off_the_list`]，判法 `crate::adapter_inventory::discrete_gpus_of_the_enumeration_under_the_named`）。
+//! 参与的卡由设置定（[`BackgroundGpuOverTheCards::participating_gpus`] 报出来），挂着切换时核心层交来新的一组
+//! （[`BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch`]，两批之间），从下一批起只在它点的卡上、照它给的显存上限与等的上限跑，不重开设备
 //! （`records/2026-10-19-里程碑四GPU垃圾回收开工.md` 第 43 批交回那一行的定法；分活、切段、等的纯逻辑在 `crate::gpus_of_this_host`）。
-//! 挂着的时候每一批按量出的代价分给参与的卡、按显存上限切段、两份核各算一次、逐条比对。三段都接上了：扫描里的校验和核对（[`BackgroundGpu::verify_unit_checksums`]）、区域扫描里的单元头解析与映射反查
-//! （[`BackgroundGpu::classify_region_records`]）与选段打分的每段活槽数（[`BackgroundGpu::count_live_slots_per_cluster_segment`]）。
-//! 本机的卡清单（核心层 `GpuInventory`）答的是本机全部独显与各自的显存总量（按总线地址排，挂载那一刻没开成的照列），连同其中没开成的那几张与原因：
-//! 挂载与切换照它校验设置，切换可以点开包时没参与而开成了的卡；点名不在本机的卡就是点名的卡不存在，参与的卡里有没开成的照第 4 条拒。
+//! 挂着的时候每一批按量出的代价分给参与的卡、按显存上限切段、两份核各算一次、逐条比对。三段都接上了：扫描里的校验和核对（[`BackgroundGpuOverTheCards::verify_unit_checksums`]）、
+//! 区域扫描里的单元头解析与映射反查（[`BackgroundGpuOverTheCards::classify_region_records`]）与选段打分的每段活槽数（[`BackgroundGpuOverTheCards::count_live_slots_per_cluster_segment`]）。
+//! 本机的卡清单（核心层 `GpuInventory`）答的是此刻本机在的独显与各自的显存总量（按总线地址排，挂载那一刻没开成的照列；每次答之前现枚举一次，
+//! 挂载之后掉了的卡不列，切换点它照第 4 条「GPU 不在」拒），连同其中没开成的那几张与原因：挂载与切换照它校验设置，切换可以点开包时没参与而开成了的卡；
+//! 点名不在本机的卡就是点名的卡不存在，参与的卡里有没开成的照第 4 条拒。
+//!
+//! 外壳分两层：一张开着的卡要做的几样（[`CardOfTheBackgroundGpu`]：交活之前先等前一次超时的那次提交、重交之前等、交一段两份核）由 [`GpuInUse`]
+//! （wgpu 设备与六条管线）实现；几张卡上怎么分活、切段、重交、超时按批按卡计数、建缓冲显存不足时重切、丢的时候泄漏谁，在 [`BackgroundGpuOverTheCards`]
+//! 里对任何一种卡都一样，测试换上在 CPU 上算的替身卡就钉得住，不起 GPU。产品那一份是 [`BackgroundGpu`]。
 //!
 //! GPU 出错（设备丢了、wgpu 报了没接住的错、等不成、回读映射不成、核没写到某一条）时这一批交回出错、不换 CPU（第 4 条）；一次提交等超过上限的，
-//! 在同一张卡上重交至多 3 次，都等超过上限才交回「这张卡用不了」（`crate::waiting_for_the_gpu::outcome_resubmitted_after_timeouts`，用户 2026-10-28 定）。
+//! 在同一张卡上重交，这一批在这张卡上（一批切成几片、几段时合起来算，交活之前等前一次超时的那次提交那一等也算）第 4 次等超过上限才交回「这张卡用不了」
+//! （`crate::waiting_for_the_gpu::TimeoutsOfTheBatchOnOneGpu`，用户 2026-10-28 定；重交的单位照旧只是超时的那一片，用户 2026-10-30 定）。
 //! 出错时不动连续不确定的计数（这一批放弃了，不算一轮）。只有一张卡时两份核都在这张卡上跑；几张卡时每张卡在自己分到的那几条上跑两份核。
 //! 每一处阻塞等 GPU 都在此刻设置给的上限之内等（开包时是 `BackgroundGpuSelection::background_gpu_wait_limit`，等法在 `crate::waiting_for_the_gpu`）：
 //! 等 wgpu 的 future 用 std 写的最小等法（[`wait_for_the_wgpu_future`]），等提交用 wgpu 自己带限时的 `poll`，不依赖 `pollster`
 //! （门禁 checker-implementation-disjoint 格第 ⑧ 条）。
-//! 每一段切批之前先扣掉这张卡上本包别的段此刻占着的显存（`ChunkBoundsOfAGpu::with_the_room_left_after`），几段合在一张卡上占的不超过它的上限（第 3 条）。
+//! 每一段切批之前先扣掉这张卡上此刻占着的显存（`ChunkBoundsOfAGpu::with_the_room_left_after`），几段合在一张卡上占的不超过它的上限（第 3 条）。
 //! 一段的提交等超过上限之后，它的缓冲在卡上可能还占着：这一段的账留到那次提交真在 GPU 上做完才放，下一批在这张卡上切段交活之前先在上限之内
-//! 等它做完，等不到就交回出错、这张卡上一段都不交（`crate::memory_limit_and_chunks` 文件头）。
-//! 装不下就切小：单元切成几片（校验和那一段）、映射按落点切成几片（区域扫描那一段）、记录表按记录切成几片（选段打分那一段），逐片交，
-//! 连最小的一片都装不下的卡不用（`crate::gpus_of_this_host` 文件头；用户 2026-10-28 定）。挂载那一刻量代价那一段也按开卡时的上限切片，代价取各片之和。
-//! 丢后台 GPU 时不等还在 GPU 上跑的提交：那张卡的设备与队列泄漏，别的卡照常丢（[`BackgroundGpu`] 的 `Drop`）。
+//! 等它做完（`crate::memory_limit_and_chunks` 文件头）。
+//! 装不下就切小：单元切成几片（校验和那一段）、映射按落点切成几片（区域扫描那一段）、记录表按记录切成几片（选段打分那一段），逐片交；
+//! 一张卡这一批连最小的一片都切不出（显存上限扣掉已占的）就这一批不分给它、活分给别的卡，参与的卡全都切不出才报
+//! （`crate::gpus_of_this_host::plans_of_a_batch_leaving_out_the_gpus_short_of_the_smallest_piece`；用户 2026-10-28 定）。
+//! 卡上建缓冲时显存不足（空闲的比上限少，第 3 条「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」）：这张卡这一批的地方压到建不成的那一段的一半、
+//! 重切、从头再交它分到的那几条，不报出错；一路压到连最小的一片都建不成才报（条款没写，第一版不支持）。
+//! 挂载那一刻量代价那一段也按开卡时的上限切片，代价取各片之和。
+//! 丢后台 GPU 时不等还在 GPU 上跑的提交：那张卡的设备与队列泄漏，别的卡照常丢；泄漏了哪几张、各占着多少报给调用方
+//! （[`BackgroundGpuOverTheCards::gpus_leaked_when_dropped`]），并记进进程里的那一份（`crate::gpus_of_this_host::GpuMemoryLeakedInThisProcess`），
+//! 新一次挂载这张卡的可用余量先扣掉它。
 
 use std::future::Future;
+use std::ops::Range;
 use std::sync::mpsc::RecvTimeoutError;
 use std::sync::{Arc, Mutex};
 use std::task::{Context, Poll, Wake, Waker};
@@ -31,7 +46,8 @@ use std::time::Duration;
 use singlefs_core::allocator::AllocationRecord;
 use singlefs_core::background_compute::{
     AdmittedGpuSettings, BackgroundComputeSettingsRefused, ClusterSegmentEntryKey,
-    GpuDeviceIdentifier, GpuDeviceNotOpenedAtTheMount, GpuInventory, GpuMemoryInBytes,
+    GpuComputedSegment, GpuDeviceIdentifier, GpuDeviceNotOpenedAtTheMount, GpuInventory,
+    GpuMemoryInBytes, GpuSubmissionsOfOneSegment, GpuSubmissionsOfTheBatch,
 };
 use singlefs_core::compaction_plan::UnitAreaOfOneDevice;
 use singlefs_core::logged_operations::CompactionRegion;
@@ -39,8 +55,10 @@ use singlefs_core::region_scan::{AllocationInRegion, CentralMappingEntry};
 use wgpu::util::DeviceExt;
 
 use crate::adapter_inventory::{
-    discrete_gpus_of_the_enumeration, participating_gpus_admitted_by_the_mount_checks,
-    AdapterDeviceKind, BackgroundGpuSelection, DiscreteGpuListRefused, DiscreteGpuOnThisHost,
+    discrete_gpus_of_the_enumeration_under_the_named,
+    identifiers_of_the_discrete_gpus_in_the_enumeration,
+    participating_gpus_admitted_by_the_mount_checks, AdapterDeviceKind, BackgroundGpuSelection,
+    DiscreteGpuListRefused, DiscreteGpuOnThisHost, DiscreteGpusPresentNow,
     EnumeratedAdapterDescription,
 };
 use crate::checksum_kernels::{
@@ -55,8 +73,11 @@ use crate::gpus_of_this_host::{
     bytes_of_the_smallest_piece_every_participating_gpu_holds, gpu_left_out_under_its_memory_limit,
     gpus_dropped_leaking_those_still_running, gpus_opened_at_the_mount,
     measured_cost_summed_over_the_pieces, memory_limit_of_the_device_under_the_settings,
+    plans_of_a_batch_leaving_out_the_gpus_short_of_the_smallest_piece, BatchNotPlannedOverTheGpus,
     CostAtTheOpening, CostProbeChunk, DiscreteGpuNotOpenedAtTheMount, DiscreteGpusAfterTheOpening,
-    GpuBudgetUnderTheSettings, GpuLeftOutUnderItsMemoryLimit, ParticipatingGpusOnThisHost,
+    GpuBudgetUnderTheSettings, GpuLeakedWhenDropped, GpuLeftOutUnderItsMemoryLimit,
+    GpuMemoryLeakedInThisProcess, GpusLeakedWhenTheBackgroundGpuIsDropped,
+    ParticipatingGpusOnThisHost, PlanOfAGpuForTheBatch, RoomOfAGpuForTheBatch,
 };
 use crate::live_slot_kernels::{
     decoded_live_slot_outcomes, live_slot_chunk_footprint,
@@ -80,15 +101,16 @@ use crate::region_record_kernels::{
     region_record_outcome_buffer_before_the_kernel_writes, region_record_parameter_bytes,
     region_record_plan_on_a_gpu, BothKernelOutcomesOfARegionRecordChunk,
     CrcRegistersOfTheTwoKernels, MappingKeyLongerThanTheKernelsHold, MappingTablesForTheKernels,
-    RegionRecordPieceForTheKernels, RegionRecordPlanOnAGpu, RegionRecordToClassify,
+    RegionRecordPieceForTheKernels, RegionRecordToClassify,
     REGION_RECORD_INVOCATIONS_PER_WORKGROUP, REGION_RECORD_OUTCOME_BYTES,
     SEARCHING_KERNEL_ENTRY_POINT, SEARCHING_KERNEL_SOURCE, WALKING_KERNEL_ENTRY_POINT,
     WALKING_KERNEL_SOURCE,
 };
 use crate::two_kernel_reconciliation::{ConsecutiveUncertaintyCounts, ReconciledBatch};
 use crate::waiting_for_the_gpu::{
-    outcome_resubmitted_after_timeouts, wait_for_completion_before_the_deadline, CompletionCheck,
-    GpuWaitDeadline, MonotonicClock, SteadyClockOfThisProcess,
+    outcome_resubmitted_after_timeouts, outcome_waited_for_again_after_timeouts,
+    wait_for_completion_before_the_deadline, CompletionCheck, GpuWaitDeadline, MonotonicClock,
+    SteadyClockOfThisProcess, TimeoutsOfTheBatchOnOneGpu,
 };
 use crate::work_split_by_measured_cost::HostCpuNanosecondsPerSubmission;
 
@@ -96,7 +118,7 @@ use crate::work_split_by_measured_cost::HostCpuNanosecondsPerSubmission;
 const LARGEST_STORAGE_BINDING_THE_KERNELS_ADDRESS_IN_BYTES: u64 = 1 << 31;
 /// 挂载那一刻量代价交的那一段：一条 4096 字节全 0 的单元（大小取一个常见的单元块；它只用来量一次提交的主机 CPU，结果不用），走校验和核对那一段的缓冲。
 /// 量的时候照这张卡开卡时的界切成几片逐段交（[`CostProbeChunk::pieces_under_the_bounds`]，与挂着时校验和那一段同一个切法），代价取各片之和
-/// （[`measured_cost_summed_over_the_pieces`]）；真建的缓冲与 `CHECKSUM_CHUNK_FOOTPRINT` 一致（[`GpuInUse::both_kernel_outcomes_of_a_chunk`] 核过）。
+/// （[`measured_cost_summed_over_the_pieces`]）；真建的缓冲与 `CHECKSUM_CHUNK_FOOTPRINT` 一致（[`CardOfTheBackgroundGpu::both_kernel_outcomes_of_a_checksum_chunk`] 核过）。
 pub const COST_PROBE_CHUNK: CostProbeChunk = CostProbeChunk {
     input_bytes: 4096,
     footprint: CHECKSUM_CHUNK_FOOTPRINT,
@@ -112,7 +134,7 @@ const OUTCOME_BUFFER_USAGE: wgpu::BufferUsages =
 pub enum GpuFailureCause {
     /// 起不了设备或编不了管线（驱动不对、显存不够）。
     DeviceOrPipelineDidNotStart { reason: String },
-    /// wgpu 报了没接住的错（校验错、显存不够）或设备丢了。
+    /// wgpu 报了没接住的错（校验错）或设备丢了。
     UncapturedErrorOrDeviceLost { reason: String },
     /// 等这一次提交没等成（不是超时）。
     WaitForTheSubmissionFailed { reason: String },
@@ -125,7 +147,16 @@ pub enum GpuFailureCause {
     ReadbackMappingFailed { reason: String },
     /// 读回的判定字认不得：核没写到那一条或写坏了。
     KernelLeftAnUnrecognizedVerdictWord(UnrecognizedVerdictWord),
-    /// 一次提交等超过上限之后在同一张卡上重交了 `resubmissions` 次，次次等超过上限（重交之前等前一次那一次没等到也算一次）：这张卡用不了，
+    /// 在卡上建一段的缓冲时显存不足（wgpu 在显存不足的错误范围里报的），`bytes_held_by_the_chunk` 是那一段合占的字节数；这一段没交，它的账已经放掉。
+    /// 挂着时外壳按更小的片重切重交（[`BackgroundGpuOverTheCards`]），不报出来。
+    OutOfMemoryCreatingTheBuffers {
+        bytes_held_by_the_chunk: u64,
+        reason: String,
+    },
+    /// 建缓冲显存不足、一路切小到连这一批最小的一片（`smallest_piece_bytes` 字节）都建不成：空闲的连最小的一片都不到时怎么办条款没写，第一版不支持。
+    FreeMemoryShortOfTheSmallestPiece { smallest_piece_bytes: u64 },
+    /// 一批里一张卡等超过上限、在同一张卡上重交了 `resubmissions` 次之后，这一批在这张卡上第 1 + `resubmissions` 次等超过上限
+    /// （一批切成几片时各片的超时合起来算，交活之前、重交之前等前一次那一次没等到也算一次）：这张卡用不了，
     /// 交核心层当报警（用户 2026-10-28 定「都超时才报警」）。`last_cause` 是最后那一次的原因（等超过上限的那两种之一）。
     UnusableAfterResubmissions {
         resubmissions: u32,
@@ -228,24 +259,101 @@ pub struct ParticipatingGpuReport {
     pub measured_cost: HostCpuNanosecondsPerSubmission,
 }
 
-/// 这一次挂载的后台 GPU：开包时开成的本机独显（按总线地址排），没开成而跳过的那几张，与此刻参与的那几张（设置定的，按参与的次序：
-/// 没点名是总线地址的次序，点了名是点名的次序；显存上限连最小的一片都装不下的不在里面）。生命周期一次挂载，只住内存。
-/// 丢它时不等还在 GPU 上跑的提交（`Drop`）。
-pub struct BackgroundGpu {
+/// 外壳里一张开着的卡要做的几样（产品是 [`GpuInUse`]，测试换上在 CPU 上算的替身卡）：报它是哪张卡、此刻设置给它的界与它上面的账、量一次提交的代价，
+/// 交活之前与重交之前先等它上面前一次等超过上限的那次提交，交一段两份核（三段各一种）。每一样的约定同 [`GpuInUse`] 上对应的那一个。
+/// 一段交成了就照那一段的账放掉；等超过上限的那一段的账留着、记成还在跑的那次提交（`GpuMemoryAccount::settle_the_chunk_after_its_submission`）；
+/// 建缓冲显存不足的那一段不交、账放掉、交回 [`GpuFailureCause::OutOfMemoryCreatingTheBuffers`]。
+pub trait CardOfTheBackgroundGpu {
+    /// 一次提交的把手（等超过上限之后还在卡上跑的那一次记在账上）。
+    type Submission;
+
+    fn discrete_gpu(&self) -> &DiscreteGpuOnThisHost;
+
+    fn budget(&self) -> &GpuBudgetUnderTheSettings<Self::Submission>;
+
+    fn budget_mut(&mut self) -> &mut GpuBudgetUnderTheSettings<Self::Submission>;
+
+    /// 量过的一次提交的主机 CPU 时间；开卡时因显存上限不用、还没量过的是 `None`。
+    fn measured_cost(&self) -> Option<HostCpuNanosecondsPerSubmission>;
+
+    /// 照此刻的界量一次提交的主机 CPU 时间并记下（之后 [`Self::measured_cost`] 交回它）。
+    ///
+    /// # Errors
+    /// 交出错照原样交回，不记。
+    fn measure_the_cost_of_one_submission(&mut self) -> Result<(), GpuFailureCause>;
+
+    /// 交活之前：前一次等超过上限、还在跑的那次提交在此刻设置给的等的上限之内等一回、做完就放掉它的账，交回此刻剩下的界。
+    ///
+    /// # Errors
+    /// 等不到它做完（等满上限是 [`GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit`]），账照旧记着。
+    fn room_once_the_submission_running_past_its_wait_completed(
+        &mut self,
+    ) -> Result<ChunkBoundsOfAGpu, GpuFailureCause>;
+
+    /// 重交之前：同 [`Self::room_once_the_submission_running_past_its_wait_completed`]，不交回界。
+    ///
+    /// # Errors
+    /// 同上。
+    fn wait_for_the_timed_out_submission(&mut self) -> Result<(), GpuFailureCause>;
+
+    /// 校验和核对的一段（切批切出来的几片，各当一条单元）交两份核，交回每片两份的结果（查表那一份在前），按段里的次序。
+    ///
+    /// # Errors
+    /// GPU 出错、等超过上限、建缓冲显存不足。
+    fn both_kernel_outcomes_of_a_checksum_chunk(
+        &mut self,
+        units: &[UnitChecksumToVerify<'_, ()>],
+    ) -> Result<Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)>, GpuFailureCause>;
+
+    /// 区域扫描的一段（几片）交两份核（`mapping_tables` 是这一段交的那一片映射，头一片之后那几片的段是 `None`），约定同
+    /// `region_record_kernels::region_record_classifications_of_the_plan` 交给它的那一个。
+    ///
+    /// # Errors
+    /// 同 [`Self::both_kernel_outcomes_of_a_checksum_chunk`]。
+    fn both_kernel_outcomes_of_a_region_record_chunk(
+        &mut self,
+        pieces: &[RegionRecordPieceForTheKernels<'_>],
+        mapping_tables: Option<&MappingTablesForTheKernels>,
+    ) -> Result<BothKernelOutcomesOfARegionRecordChunk, GpuFailureCause>;
+
+    /// 选段打分的一段（几个段，连同这一片记录的两张表）交两份核，交回每段两份的活槽数（逐条走遍记录那一份在前），按段里的次序。
+    ///
+    /// # Errors
+    /// 同 [`Self::both_kernel_outcomes_of_a_checksum_chunk`]。
+    fn both_kernel_live_slots_of_a_chunk(
+        &mut self,
+        segments: &[ClusterSegmentEntryKey],
+        record_tables: &AllocationRecordTablesForTheKernels,
+        unit_area: UnitAreaOfOneDevice,
+    ) -> Result<Vec<(u64, u64)>, GpuFailureCause>;
+}
+
+/// 这一次挂载的后台 GPU 在几张开着的卡上（`Card` 是一张开着的卡，产品是 [`GpuInUse`]）：开包时开成的本机独显（按总线地址排），没开成而跳过的那几张，
+/// 给了点名时没点名而交不成一张卡、没收进清单的那几张，与此刻参与的那几张（设置定的，按参与的次序：没点名是总线地址的次序，点了名是点名的次序；
+/// 显存上限连最小的一片都装不下的不在里面）。生命周期一次挂载，只住内存。丢它时不等还在 GPU 上跑的提交（`Drop`）。
+pub struct BackgroundGpuOverTheCards<Card: CardOfTheBackgroundGpu> {
     /// 开成的那几张，与 `discrete_gpus_after_the_opening.opened` 一一对应、同序。
-    gpus_on_this_host: Vec<GpuInUse>,
+    cards_on_this_host: Vec<Card>,
     discrete_gpus_after_the_opening: DiscreteGpusAfterTheOpening,
+    unnamed_discrete_gpus_left_off_the_list: Vec<DiscreteGpuListRefused>,
     participating: ParticipatingGpusOnThisHost,
     /// 此刻这组设置点了而因显存上限装不下最小的一片不用的卡，按参与的次序。
     left_out_under_their_memory_limit: Vec<GpuLeftOutUnderItsMemoryLimit>,
     /// 最近一次换设置时量一张新参与的卡的代价出错（这张卡没参与）：之后每一批都报成它的 GPU 出错，直到下一次换设置。
     cost_not_measured_at_the_last_switch: Option<(GpuDeviceIdentifier, GpuFailureCause)>,
-    /// 设备由这个实例枚举出来，随后台 GPU 一起活着。
-    #[allow(dead_code, reason = "只为让实例与设备同寿，不读它")]
-    instance_the_devices_came_from: wgpu::Instance,
+    /// 上一次交回以来三段各自在卡上是怎么交的（[`Self::take_the_submissions_of_the_batch`]）。
+    submissions_since_the_last_take: GpuSubmissionsOfTheBatch,
+    gpus_leaked_when_dropped: GpusLeakedWhenTheBackgroundGpuIsDropped,
+    memory_leaked_in_this_process: GpuMemoryLeakedInThisProcess,
+    /// 此刻本机在的独显（卡清单每次答之前现查）。产品里它带着枚举出卡的那个 wgpu 实例：排在卡后面，卡都丢了才丢它。
+    discrete_gpus_present_now: Box<dyn DiscreteGpusPresentNow + Send>,
 }
 
-struct GpuInUse {
+/// 产品那一份：开着的卡是 wgpu 设备与六条管线。
+pub type BackgroundGpu = BackgroundGpuOverTheCards<GpuInUse>;
+
+/// 一张开着的 wgpu 卡（产品的 [`CardOfTheBackgroundGpu`]）。
+pub struct GpuInUse {
     discrete_gpu: DiscreteGpuOnThisHost,
     device: wgpu::Device,
     queue: wgpu::Queue,
@@ -263,6 +371,110 @@ struct GpuInUse {
     measured_cost: Option<HostCpuNanosecondsPerSubmission>,
 }
 
+/// 一批里一张卡上交成了几次、合起来几片，与这一批这张卡的超时次数（按批、按卡数：`TimeoutsOfTheBatchOnOneGpu`）。
+pub struct SubmissionsOfACardInTheBatch {
+    pub timeouts: TimeoutsOfTheBatchOnOneGpu,
+    pub submissions_completed: u64,
+    pub pieces_submitted: u64,
+}
+
+impl SubmissionsOfACardInTheBatch {
+    /// 一批开头：一次都还没交。
+    #[must_use]
+    pub fn none_yet() -> Self {
+        Self {
+            timeouts: TimeoutsOfTheBatchOnOneGpu::none_yet(),
+            submissions_completed: 0,
+            pieces_submitted: 0,
+        }
+    }
+
+    /// 在这张卡上交一段（`pieces` 片）：等超过上限在同一张卡上重交，超时记进这一批这张卡的那一份
+    /// （`waiting_for_the_gpu::outcome_resubmitted_after_timeouts`）；交成了记一次、记片数。
+    ///
+    /// # Errors
+    /// 同 `outcome_resubmitted_after_timeouts`。
+    pub fn outcome_of_a_chunk<Card: CardOfTheBackgroundGpu, Outcome>(
+        &mut self,
+        card: &mut Card,
+        pieces: usize,
+        submit_once: impl FnMut(&mut Card) -> Result<Outcome, GpuFailureCause>,
+    ) -> Result<Outcome, GpuFailureCause> {
+        let outcome = outcome_resubmitted_after_timeouts(
+            card,
+            &mut self.timeouts,
+            Card::wait_for_the_timed_out_submission,
+            submit_once,
+        )?;
+        self.submissions_completed += 1;
+        self.pieces_submitted += u64::try_from(pieces).expect("一段的片数装得进 u64");
+        Ok(outcome)
+    }
+}
+
+/// 一批没算成（三段各自的出错照它报：`checksum_batch_failed_of` 这三个）。封闭集合。
+#[derive(Clone, Debug, PartialEq, Eq)]
+enum OutcomesOfTheBatchNotComputed {
+    GpuFailed {
+        device: GpuDeviceIdentifier,
+        cause: GpuFailureCause,
+    },
+    EveryParticipatingGpuLeftOut(Vec<GpuLeftOutUnderItsMemoryLimit>),
+    GpuCannotHoldTheSmallestPiece(SmallestPieceNotHeldOnTheGpu),
+}
+
+fn checksum_batch_failed_of(not_computed: OutcomesOfTheBatchNotComputed) -> ChecksumBatchFailed {
+    {
+        match not_computed {
+            OutcomesOfTheBatchNotComputed::GpuFailed { device, cause } => {
+                ChecksumBatchFailed::GpuFailed { device, cause }
+            }
+            OutcomesOfTheBatchNotComputed::EveryParticipatingGpuLeftOut(gpus_left_out) => {
+                ChecksumBatchFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit(gpus_left_out)
+            }
+            OutcomesOfTheBatchNotComputed::GpuCannotHoldTheSmallestPiece(smallest_piece) => {
+                ChecksumBatchFailed::GpuCannotHoldTheSmallestPiece(smallest_piece)
+            }
+        }
+    }
+}
+
+fn region_record_batch_failed_of(
+    not_computed: OutcomesOfTheBatchNotComputed,
+) -> RegionRecordBatchFailed {
+    {
+        match not_computed {
+            OutcomesOfTheBatchNotComputed::GpuFailed { device, cause } => {
+                RegionRecordBatchFailed::GpuFailed { device, cause }
+            }
+            OutcomesOfTheBatchNotComputed::EveryParticipatingGpuLeftOut(gpus_left_out) => {
+                RegionRecordBatchFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit(
+                    gpus_left_out,
+                )
+            }
+            OutcomesOfTheBatchNotComputed::GpuCannotHoldTheSmallestPiece(smallest_piece) => {
+                RegionRecordBatchFailed::GpuCannotHoldTheSmallestPiece(smallest_piece)
+            }
+        }
+    }
+}
+
+fn live_slot_batch_failed_of(not_computed: OutcomesOfTheBatchNotComputed) -> LiveSlotBatchFailed {
+    {
+        match not_computed {
+            OutcomesOfTheBatchNotComputed::GpuFailed { device, cause } => {
+                LiveSlotBatchFailed::GpuFailed { device, cause }
+            }
+            OutcomesOfTheBatchNotComputed::EveryParticipatingGpuLeftOut(gpus_left_out) => {
+                LiveSlotBatchFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit(gpus_left_out)
+            }
+            OutcomesOfTheBatchNotComputed::GpuCannotHoldTheSmallestPiece(smallest_piece) => {
+                LiveSlotBatchFailed::GpuCannotHoldTheSmallestPiece(smallest_piece)
+            }
+        }
+    }
+}
+
 /// 一份核这一次的派发：管线、按绑定号交给它的缓冲（它的结果缓冲也在里面）、它写的那个结果缓冲。
 struct KernelDispatch<'buffers> {
     pipeline: &'buffers wgpu::ComputePipeline,
@@ -270,18 +482,35 @@ struct KernelDispatch<'buffers> {
     outcome_buffer: &'buffers wgpu::Buffer,
 }
 
+/// 挂载那一刻：枚举适配器、判卡清单与校验、在本机每一张独立显卡上起设备与各段两份核、量一次提交的主机 CPU 时间（同
+/// [`open_background_gpu_with_the_memory_leaked_in_this_process`]，进程里还一张卡都没泄漏过）。
+///
+/// # Errors
+/// [`BackgroundGpuOpenRefused`] 的四种。
+pub fn open_background_gpu(
+    selection: &BackgroundGpuSelection,
+) -> Result<BackgroundGpu, BackgroundGpuOpenRefused> {
+    open_background_gpu_with_the_memory_leaked_in_this_process(
+        selection,
+        &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+    )
+}
+
 /// 挂载那一刻：枚举适配器、判卡清单与校验、在本机每一张独立显卡上起设备与各段两份核、量一次提交的主机 CPU 时间；参与的卡照 `selection`
-/// （之后切换时核心层另交，[`BackgroundGpu::use_the_settings_from_the_next_batch`]）。每一处等 wgpu 都在 `selection.background_gpu_wait_limit` 之内。
+/// （之后切换时核心层另交，[`BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch`]）。每一处等 wgpu 都在 `selection.background_gpu_wait_limit` 之内。
+/// 给了 `background_gpu_devices` 时只判点名的卡（没点名的独显交不成一张卡就不收、报出来）。
 /// 没参与的卡也开（之后的切换可能点它，切换不重开设备）；一张卡没开成，参与的就拒，给了列表而没点名的跳过、报出来
-/// （[`gpus_opened_at_the_mount`]、[`BackgroundGpu::gpus_not_opened_at_the_mount`]）。参与的卡照开卡那一组给它的显存上限把量代价那一段
-/// （[`COST_PROBE_CHUNK`]）切成几片逐片量、代价取各片之和；显存上限连最小的一片都装不下的卡照开、不量代价、不参与、报出来
-/// （[`BackgroundGpu::gpus_left_out_under_their_memory_limit`]），挂载不因此拒绝；参与的卡因此一张都不剩时照样交回，三段每一批都报
-/// 「参与的卡一张都不剩」（[`BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut`]），核心层挂载那一刻就读到、当场报警停着。
+/// （[`gpus_opened_at_the_mount`]、[`BackgroundGpuOverTheCards::gpus_not_opened_at_the_mount`]）。参与的卡照开卡那一组给它的显存上限
+/// （先扣掉这个进程里此前几次挂载泄漏在它上面的字节，`memory_leaked_in_this_process`）把量代价那一段（[`COST_PROBE_CHUNK`]）切成几片逐片量、代价取各片之和；
+/// 显存上限连最小的一片都装不下的卡照开、不量代价、不参与、报出来（[`BackgroundGpuOverTheCards::gpus_left_out_under_their_memory_limit`]），挂载不因此拒绝；
+/// 参与的卡因此一张都不剩时照样交回，三段每一批都报「参与的卡一张都不剩」（[`BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut`]），
+/// 核心层挂载那一刻就读到、当场报警停着。丢掉它时泄漏的卡记进 `memory_leaked_in_this_process`。
 ///
 /// # Errors
 /// [`BackgroundGpuOpenRefused`] 的四种。
-pub fn open_background_gpu(
+pub fn open_background_gpu_with_the_memory_leaked_in_this_process(
     selection: &BackgroundGpuSelection,
+    memory_leaked_in_this_process: &GpuMemoryLeakedInThisProcess,
 ) -> Result<BackgroundGpu, BackgroundGpuOpenRefused> {
     let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
         backends: wgpu::Backends::VULKAN,
@@ -295,14 +524,19 @@ pub fn open_background_gpu(
     .map_err(|cause| BackgroundGpuOpenRefused::AdapterEnumerationFailed { cause })?;
     let descriptions: Vec<EnumeratedAdapterDescription> =
         adapters.iter().map(description_of_the_adapter).collect();
-    let discrete_gpus = discrete_gpus_of_the_enumeration(&descriptions)
-        .map_err(BackgroundGpuOpenRefused::DiscreteGpuListNotSupported)?;
-    let participating = participating_gpus_admitted_by_the_mount_checks(selection, &discrete_gpus)
-        .map_err(BackgroundGpuOpenRefused::SettingsRefused)?;
+    let discrete_gpus = discrete_gpus_of_the_enumeration_under_the_named(
+        &descriptions,
+        selection.background_gpu_devices.as_deref(),
+    )
+    .map_err(BackgroundGpuOpenRefused::DiscreteGpuListNotSupported)?;
+    let participating =
+        participating_gpus_admitted_by_the_mount_checks(selection, &discrete_gpus.listed)
+            .map_err(BackgroundGpuOpenRefused::SettingsRefused)?;
     let gpus_opened = gpus_opened_at_the_mount(
-        discrete_gpus,
+        discrete_gpus.listed,
         &participating,
         bytes_of_the_smallest_piece_every_participating_gpu_holds(),
+        memory_leaked_in_this_process,
         |discrete_gpu, memory_limit_at_the_opening, cost_at_the_opening| {
             started_gpu_with_its_cost_measured(
                 &adapters[discrete_gpu.position_in_the_enumeration],
@@ -310,17 +544,57 @@ pub fn open_background_gpu(
                 memory_limit_at_the_opening,
                 selection.background_gpu_wait_limit,
                 cost_at_the_opening,
+                memory_leaked_in_this_process.bytes_leaked_on(&discrete_gpu.identifier),
             )
         },
     )?;
-    Ok(BackgroundGpu {
-        gpus_on_this_host: gpus_opened.opened,
-        discrete_gpus_after_the_opening: gpus_opened.discrete_gpus,
-        participating: gpus_opened.participating,
-        left_out_under_their_memory_limit: gpus_opened.left_out_under_their_memory_limit,
-        cost_not_measured_at_the_last_switch: None,
-        instance_the_devices_came_from: instance,
-    })
+    Ok(BackgroundGpuOverTheCards::of_the_cards_opened_at_the_mount(
+        gpus_opened.opened,
+        gpus_opened.discrete_gpus,
+        discrete_gpus.unnamed_left_off_the_list,
+        gpus_opened.participating,
+        gpus_opened.left_out_under_their_memory_limit,
+        memory_leaked_in_this_process.clone(),
+        Box::new(AdaptersEnumeratedByTheInstance {
+            instance,
+            wait_limit: selection.background_gpu_wait_limit,
+        }),
+    ))
+}
+
+/// 挂着切换那一刻现查本机在的独显（[`DiscreteGpusPresentNow`]）：拿开卡的那个 wgpu 实例再枚举一次，在开卡时那个等的上限之内等；
+/// 它也是卡的设备都丢了之后才丢的那个实例。
+struct AdaptersEnumeratedByTheInstance {
+    instance: wgpu::Instance,
+    wait_limit: Duration,
+}
+
+impl DiscreteGpusPresentNow for AdaptersEnumeratedByTheInstance {
+    fn identifiers_of_the_discrete_gpus_present_now(&self) -> Vec<GpuDeviceIdentifier> {
+        let Ok(adapters) = wait_for_the_wgpu_future(
+            self.instance.enumerate_adapters(wgpu::Backends::VULKAN),
+            self.wait_limit,
+            &SteadyClockOfThisProcess::starting_now(),
+        ) else {
+            // 枚举不成（等 wgpu 超过上限）：一张都不算在，切换照「GPU 不在」拒（DiscreteGpusPresentNow 的约定）。
+            return Vec::new();
+        };
+        let descriptions: Vec<EnumeratedAdapterDescription> = adapters
+            .iter()
+            .map(|adapter| {
+                let adapter_information = adapter.get_info();
+                EnumeratedAdapterDescription {
+                    name: adapter_information.name,
+                    device_kind: AdapterDeviceKind::of_the_wgpu_device_type(
+                        adapter_information.device_type,
+                    ),
+                    pci_bus_address: adapter_information.device_pci_bus_id,
+                    device_local_memory: None,
+                }
+            })
+            .collect();
+        identifiers_of_the_discrete_gpus_in_the_enumeration(&descriptions)
+    }
 }
 
 fn description_of_the_adapter(adapter: &wgpu::Adapter) -> EnumeratedAdapterDescription {
@@ -372,6 +646,7 @@ fn started_gpu_with_its_cost_measured(
     memory_limit: Option<GpuMemoryInBytes>,
     wait_limit: Duration,
     cost_at_the_opening: CostAtTheOpening,
+    bytes_leaked_by_earlier_mounts: u64,
 ) -> Result<GpuInUse, GpuFailureCause> {
     let adapter_limits = adapter.limits();
     let (device, queue) = wait_for_the_wgpu_future(
@@ -439,7 +714,7 @@ fn started_gpu_with_its_cost_measured(
                     .min(REGION_RECORD_INVOCATIONS_PER_WORKGROUP)
                     .min(LIVE_SLOT_INVOCATIONS_PER_WORKGROUP),
             );
-    let budget = GpuBudgetUnderTheSettings::with_nothing_held(
+    let budget = GpuBudgetUnderTheSettings::with_the_bytes_leaked_by_earlier_mounts(
         adapter_limits
             .max_storage_buffer_binding_size
             .min(adapter_limits.max_buffer_size)
@@ -447,6 +722,7 @@ fn started_gpu_with_its_cost_measured(
         largest_entry_count_per_dispatch,
         memory_limit,
         wait_limit,
+        bytes_leaked_by_earlier_mounts,
     );
     let mut gpu_in_use = GpuInUse {
         discrete_gpu,
@@ -462,12 +738,12 @@ fn started_gpu_with_its_cost_measured(
         fault_reported_by_wgpu,
         measured_cost: None,
     };
-    gpu_in_use.measured_cost = match cost_at_the_opening {
+    match cost_at_the_opening {
         CostAtTheOpening::MeasuredUnderTheMemoryLimit => {
-            Some(gpu_in_use.measured_cost_of_one_submission()?)
+            gpu_in_use.measure_the_cost_of_one_submission()?;
         }
-        CostAtTheOpening::NotMeasuredForTheGpuIsLeftOut => None,
-    };
+        CostAtTheOpening::NotMeasuredForTheGpuIsLeftOut => {}
+    }
     Ok(gpu_in_use)
 }
 
@@ -545,51 +821,10 @@ fn completion_check_of_the_submission(
 }
 
 impl GpuInUse {
-    /// 在这张卡上交这一批之前调：前一次等超过上限、还在跑的那次提交先在此刻设置给的等的上限之内等它做完、放掉它的账（第 3 条；等不到交回出错，
-    /// 不越过上限去交），交回此刻剩下的界（`GpuBudgetUnderTheSettings`）。
-    fn room_once_the_submission_running_past_its_wait_completed(
-        &mut self,
-    ) -> Result<ChunkBoundsOfAGpu, GpuFailureCause> {
-        let clock = SteadyClockOfThisProcess::starting_now();
-        let device = &self.device;
-        self.budget
-            .room_once_the_submission_running_past_its_wait_completed(
-                &clock,
-                |submission, time_left| {
-                    completion_check_of_the_submission(device, submission, time_left)
-                },
-            )
-    }
-
-    /// 这张卡连最小的一片都装不下时报的那一条：`range_start` 是这张卡分到的那几条在批里的起点。
-    fn smallest_piece_not_held(
-        &self,
-        range_start: usize,
-        smallest_piece: SmallestPieceLargerThanTheRoom,
-    ) -> SmallestPieceNotHeldOnTheGpu {
-        SmallestPieceNotHeldOnTheGpu {
-            device: self.discrete_gpu.identifier.clone(),
-            entry_position: range_start + smallest_piece.entry_position,
-            memory_limit: self.budget.memory_account().memory_limit(),
-            smallest_piece_bytes: smallest_piece.smallest_piece_bytes,
-        }
-    }
-
-    /// 重交之前调：前一次等超过上限、还在跑的那次提交在此刻设置给的等的上限之内等它做完、放掉它的账。
-    fn wait_for_the_timed_out_submission(&mut self) -> Result<(), GpuFailureCause> {
-        let clock = SteadyClockOfThisProcess::starting_now();
-        let device = &self.device;
-        self.budget.wait_for_the_submission_running_past_its_wait(
-            &clock,
-            |submission, time_left| {
-                completion_check_of_the_submission(device, submission, time_left)
-            },
-        )
-    }
-
     /// 量一次提交的主机 CPU 时间（第 2 条；挂载那一刻，或换设置让开卡时没量的卡参与时）：这条线程的 CPU 钟（不是挂钟）。
-    /// 交的那一段是 [`COST_PROBE_CHUNK`]，照这张卡此刻的界切成几片、几段（装得下就一段一片），每一次是各段各交一遍：建缓冲、编码两份核、交、等、
-    /// 回读、解码，与挂着时交一段走同一条路（[`Self::both_kernel_outcomes_of_a_chunk`]）；先空跑一遍，再量 [`COST_MEASUREMENT_SUBMISSIONS`] 遍，
+    /// 交的那一段是 [`COST_PROBE_CHUNK`]，照这张卡此刻剩下的界（上限扣掉已占的，含这个进程里此前泄漏在它上面的）切成几片、几段
+    /// （装得下就一段一片），每一次是各段各交一遍：建缓冲、编码两份核、交、等、回读、解码，与挂着时交一段走同一条路
+    /// （[`CardOfTheBackgroundGpu::both_kernel_outcomes_of_a_checksum_chunk`]）；先空跑一遍，再量 [`COST_MEASUREMENT_SUBMISSIONS`] 遍，
     /// 每遍取各段之和，取中位数（[`measured_cost_summed_over_the_pieces`]）。
     ///
     /// # Errors
@@ -608,7 +843,7 @@ impl GpuInUse {
             expected_crc32c: 0,
         }];
         let chunks_of_pieces = COST_PROBE_CHUNK
-            .pieces_under_the_bounds(self.budget.chunk_bounds())
+            .pieces_under_the_bounds(self.budget.room_left_now())
             .map_err(
                 |smallest_piece| GpuFailureCause::DeviceOrPipelineDidNotStart {
                     reason: format!(
@@ -623,7 +858,7 @@ impl GpuInUse {
             |pieces| {
                 let piece_units = units_of_the_pieces(&probe, pieces);
                 let cpu_time_before = host_cpu_time_of_this_thread();
-                self.both_kernel_outcomes_of_a_chunk(&piece_units)?;
+                self.both_kernel_outcomes_of_a_checksum_chunk(&piece_units)?;
                 Ok(host_cpu_time_of_this_thread().saturating_sub(cpu_time_before))
             },
         )
@@ -638,6 +873,36 @@ impl GpuInUse {
             })
     }
 
+    /// 在只接显存不足的错误范围里建这一段的缓冲（`create_the_buffers`），这一段的账（`bytes_held_by_this_chunk`）调用方已经记下：
+    /// 建得成交回缓冲；建不成（显存不足，第 3 条「卡上此刻空闲的不到这个数」）就放掉这一段的账、交回
+    /// [`GpuFailureCause::OutOfMemoryCreatingTheBuffers`]，一次都不交。等 wgpu 交回那个错误范围也在等的上限之内。
+    fn buffers_created_within_the_free_memory<Buffers>(
+        &mut self,
+        bytes_held_by_this_chunk: u64,
+        create_the_buffers: impl FnOnce(&Self) -> Buffers,
+    ) -> Result<Buffers, GpuFailureCause> {
+        let out_of_memory_scope = self.device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
+        let buffers = create_the_buffers(self);
+        let captured = wait_for_the_wgpu_future(
+            out_of_memory_scope.pop(),
+            self.budget.wait_limit(),
+            &SteadyClockOfThisProcess::starting_now(),
+        );
+        let failure = match captured {
+            Ok(None) => return Ok(buffers),
+            Ok(Some(out_of_memory)) => GpuFailureCause::OutOfMemoryCreatingTheBuffers {
+                bytes_held_by_the_chunk: bytes_held_by_this_chunk,
+                reason: format!("{out_of_memory}"),
+            },
+            Err(cause) => cause,
+        };
+        drop(buffers);
+        self.budget
+            .memory_account_mut()
+            .release(bytes_held_by_this_chunk);
+        Err(failure)
+    }
+
     /// 两份核在同一次提交里各算一遍、各自的结果缓冲拷进各自的回读暂存，等做完、等两份映射好，交回两份读回的字节（按派发的次序），
     /// 连同这次提交等完之后在卡上还占不占显存（没等到它做完的，调用方照它把这一段的账留着）。
     /// 一次提交里的几处等共用一个期限，合起来不超过此刻设置给这张卡的等的上限。
@@ -766,12 +1031,66 @@ impl GpuInUse {
             bytes_read_back_from(second_readback)?,
         ])
     }
+}
+
+impl CardOfTheBackgroundGpu for GpuInUse {
+    type Submission = wgpu::SubmissionIndex;
+
+    fn discrete_gpu(&self) -> &DiscreteGpuOnThisHost {
+        &self.discrete_gpu
+    }
+
+    fn budget(&self) -> &GpuBudgetUnderTheSettings<wgpu::SubmissionIndex> {
+        &self.budget
+    }
+
+    fn budget_mut(&mut self) -> &mut GpuBudgetUnderTheSettings<wgpu::SubmissionIndex> {
+        &mut self.budget
+    }
+
+    fn measured_cost(&self) -> Option<HostCpuNanosecondsPerSubmission> {
+        self.measured_cost
+    }
+
+    fn measure_the_cost_of_one_submission(&mut self) -> Result<(), GpuFailureCause> {
+        self.measured_cost = Some(self.measured_cost_of_one_submission()?);
+        Ok(())
+    }
 
-    /// 校验和核对的一段（切批切出来的、装得下这张卡的那几条）交给这张卡：两份核在同一次提交里各算一遍，交回每条两份的结果，按段里的次序。
-    /// 本包这一段占的缓冲先记进这张卡的账（超上限断言），那次提交做完放掉；没等到它做完的留着（`GpuMemoryAccount::settle_the_chunk_after_its_submission`）。
-    fn both_kernel_outcomes_of_a_chunk<EntryKey>(
+    /// 在这张卡上交这一批之前调：前一次等超过上限、还在跑的那次提交先在此刻设置给的等的上限之内等它做完、放掉它的账（第 3 条；等不到交回出错，
+    /// 不越过上限去交），交回此刻剩下的界（`GpuBudgetUnderTheSettings`）。
+    fn room_once_the_submission_running_past_its_wait_completed(
         &mut self,
-        units: &[UnitChecksumToVerify<'_, EntryKey>],
+    ) -> Result<ChunkBoundsOfAGpu, GpuFailureCause> {
+        let clock = SteadyClockOfThisProcess::starting_now();
+        let device = &self.device;
+        self.budget
+            .room_once_the_submission_running_past_its_wait_completed(
+                &clock,
+                |submission, time_left| {
+                    completion_check_of_the_submission(device, submission, time_left)
+                },
+            )
+    }
+
+    /// 重交之前调：前一次等超过上限、还在跑的那次提交在此刻设置给的等的上限之内等它做完、放掉它的账。
+    fn wait_for_the_timed_out_submission(&mut self) -> Result<(), GpuFailureCause> {
+        let clock = SteadyClockOfThisProcess::starting_now();
+        let device = &self.device;
+        self.budget.wait_for_the_submission_running_past_its_wait(
+            &clock,
+            |submission, time_left| {
+                completion_check_of_the_submission(device, submission, time_left)
+            },
+        )
+    }
+
+    /// 校验和核对的一段（切批切出来的、装得下这张卡的那几片）交给这张卡：两份核在同一次提交里各算一遍，交回每片两份的结果，按段里的次序。
+    /// 本包这一段占的缓冲先记进这张卡的账（超上限断言），那次提交做完放掉；没等到它做完的留着（`GpuMemoryAccount::settle_the_chunk_after_its_submission`）；
+    /// 建缓冲显存不足的放掉、不交（[`GpuInUse::buffers_created_within_the_free_memory`]）。
+    fn both_kernel_outcomes_of_a_checksum_chunk(
+        &mut self,
+        units: &[UnitChecksumToVerify<'_, ()>],
     ) -> Result<Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)>, GpuFailureCause> {
         assert!(!units.is_empty(), "切批不交空段");
         let packed = packed_checksum_chunk(units);
@@ -806,43 +1125,42 @@ impl GpuInUse {
         self.budget
             .memory_account_mut()
             .hold(bytes_held_by_this_chunk);
-        let (read_back, submission_of_the_chunk) = {
-            let parameter_buffer =
-                self.buffer_with_contents(&parameter_bytes, wgpu::BufferUsages::UNIFORM);
-            let input_buffer =
-                self.buffer_with_contents(&packed.input_bytes, wgpu::BufferUsages::STORAGE);
-            let descriptor_buffer =
-                self.buffer_with_contents(&descriptor_bytes, wgpu::BufferUsages::STORAGE);
-            let table_driven_outcomes =
-                self.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE);
-            let bitwise_outcomes = self.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE);
-            self.dispatch_both_kernels_and_read_back(
+        let [parameter_buffer, input_buffer, descriptor_buffer, table_driven_outcomes, bitwise_outcomes] =
+            self.buffers_created_within_the_free_memory(bytes_held_by_this_chunk, |gpu| {
                 [
-                    KernelDispatch {
-                        pipeline: &self.table_driven_pipeline,
-                        bound_buffers: vec![
-                            (0, &parameter_buffer),
-                            (1, &input_buffer),
-                            (2, &descriptor_buffer),
-                            (3, &table_driven_outcomes),
-                        ],
-                        outcome_buffer: &table_driven_outcomes,
-                    },
-                    KernelDispatch {
-                        pipeline: &self.bitwise_pipeline,
-                        bound_buffers: vec![
-                            (0, &parameter_buffer),
-                            (1, &input_buffer),
-                            (2, &descriptor_buffer),
-                            (3, &bitwise_outcomes),
-                        ],
-                        outcome_buffer: &bitwise_outcomes,
-                    },
-                ],
-                entry_count_in_words.div_ceil(INVOCATIONS_PER_WORKGROUP),
-                outcome_bytes,
-            )
-        };
+                    gpu.buffer_with_contents(&parameter_bytes, wgpu::BufferUsages::UNIFORM),
+                    gpu.buffer_with_contents(&packed.input_bytes, wgpu::BufferUsages::STORAGE),
+                    gpu.buffer_with_contents(&descriptor_bytes, wgpu::BufferUsages::STORAGE),
+                    gpu.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE),
+                    gpu.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE),
+                ]
+            })?;
+        let (read_back, submission_of_the_chunk) = self.dispatch_both_kernels_and_read_back(
+            [
+                KernelDispatch {
+                    pipeline: &self.table_driven_pipeline,
+                    bound_buffers: vec![
+                        (0, &parameter_buffer),
+                        (1, &input_buffer),
+                        (2, &descriptor_buffer),
+                        (3, &table_driven_outcomes),
+                    ],
+                    outcome_buffer: &table_driven_outcomes,
+                },
+                KernelDispatch {
+                    pipeline: &self.bitwise_pipeline,
+                    bound_buffers: vec![
+                        (0, &parameter_buffer),
+                        (1, &input_buffer),
+                        (2, &descriptor_buffer),
+                        (3, &bitwise_outcomes),
+                    ],
+                    outcome_buffer: &bitwise_outcomes,
+                },
+            ],
+            entry_count_in_words.div_ceil(INVOCATIONS_PER_WORKGROUP),
+            outcome_bytes,
+        );
         self.budget
             .memory_account_mut()
             .settle_the_chunk_after_its_submission(
@@ -859,8 +1177,7 @@ impl GpuInUse {
 
     /// 区域扫描的一段（几片）交给这张卡：两份核在同一次提交里各算一遍。`mapping_tables` 是这一段交的那一片映射：从单元头起的那几片的段交它、
     /// 交回每条两份的分类；单元头一片之后那几片的段是 `None`，交一份空映射、交回每片两份算完的寄存器（`region_record_kernels` 文件头「一个单元切成几片」）。
-    /// 按段里的次序。本包这一段占的缓冲先记进这张卡的账（超上限断言），那次提交做完放掉；没等到它做完的留着
-    /// （`GpuMemoryAccount::settle_the_chunk_after_its_submission`）。
+    /// 按段里的次序。账同 [`CardOfTheBackgroundGpu::both_kernel_outcomes_of_a_checksum_chunk`]。
     fn both_kernel_outcomes_of_a_region_record_chunk(
         &mut self,
         pieces: &[RegionRecordPieceForTheKernels<'_>],
@@ -920,51 +1237,52 @@ impl GpuInUse {
         self.budget
             .memory_account_mut()
             .hold(bytes_held_by_this_chunk);
-        let (read_back, submission_of_the_chunk) = {
-            let parameter_buffer =
-                self.buffer_with_contents(&parameter_bytes, wgpu::BufferUsages::UNIFORM);
-            let input_buffer =
-                self.buffer_with_contents(&packed.input_bytes, wgpu::BufferUsages::STORAGE);
-            let descriptor_buffer =
-                self.buffer_with_contents(&descriptor_bytes, wgpu::BufferUsages::STORAGE);
-            let entry_buffer = self.buffer_with_contents(&entry_bytes, wgpu::BufferUsages::STORAGE);
-            let location_item_buffer =
-                self.buffer_with_contents(&location_item_bytes, wgpu::BufferUsages::STORAGE);
-            let key_sorted_position_buffer =
-                self.buffer_with_contents(&key_sorted_position_bytes, wgpu::BufferUsages::STORAGE);
-            let walking_outcomes = self.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE);
-            let searching_outcomes = self.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE);
-            self.dispatch_both_kernels_and_read_back(
+        let [parameter_buffer, input_buffer, descriptor_buffer, entry_buffer, location_item_buffer, key_sorted_position_buffer, walking_outcomes, searching_outcomes] =
+            self.buffers_created_within_the_free_memory(bytes_held_by_this_chunk, |gpu| {
                 [
-                    KernelDispatch {
-                        pipeline: &self.walking_pipeline,
-                        bound_buffers: vec![
-                            (0, &parameter_buffer),
-                            (1, &input_buffer),
-                            (2, &descriptor_buffer),
-                            (3, &entry_buffer),
-                            (6, &walking_outcomes),
-                        ],
-                        outcome_buffer: &walking_outcomes,
-                    },
-                    KernelDispatch {
-                        pipeline: &self.searching_pipeline,
-                        bound_buffers: vec![
-                            (0, &parameter_buffer),
-                            (1, &input_buffer),
-                            (2, &descriptor_buffer),
-                            (3, &entry_buffer),
-                            (4, &location_item_buffer),
-                            (5, &key_sorted_position_buffer),
-                            (6, &searching_outcomes),
-                        ],
-                        outcome_buffer: &searching_outcomes,
-                    },
-                ],
-                record_count.div_ceil(REGION_RECORD_INVOCATIONS_PER_WORKGROUP),
-                outcome_bytes,
-            )
-        };
+                    gpu.buffer_with_contents(&parameter_bytes, wgpu::BufferUsages::UNIFORM),
+                    gpu.buffer_with_contents(&packed.input_bytes, wgpu::BufferUsages::STORAGE),
+                    gpu.buffer_with_contents(&descriptor_bytes, wgpu::BufferUsages::STORAGE),
+                    gpu.buffer_with_contents(&entry_bytes, wgpu::BufferUsages::STORAGE),
+                    gpu.buffer_with_contents(&location_item_bytes, wgpu::BufferUsages::STORAGE),
+                    gpu.buffer_with_contents(
+                        &key_sorted_position_bytes,
+                        wgpu::BufferUsages::STORAGE,
+                    ),
+                    gpu.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE),
+                    gpu.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE),
+                ]
+            })?;
+        let (read_back, submission_of_the_chunk) = self.dispatch_both_kernels_and_read_back(
+            [
+                KernelDispatch {
+                    pipeline: &self.walking_pipeline,
+                    bound_buffers: vec![
+                        (0, &parameter_buffer),
+                        (1, &input_buffer),
+                        (2, &descriptor_buffer),
+                        (3, &entry_buffer),
+                        (6, &walking_outcomes),
+                    ],
+                    outcome_buffer: &walking_outcomes,
+                },
+                KernelDispatch {
+                    pipeline: &self.searching_pipeline,
+                    bound_buffers: vec![
+                        (0, &parameter_buffer),
+                        (1, &input_buffer),
+                        (2, &descriptor_buffer),
+                        (3, &entry_buffer),
+                        (4, &location_item_buffer),
+                        (5, &key_sorted_position_buffer),
+                        (6, &searching_outcomes),
+                    ],
+                    outcome_buffer: &searching_outcomes,
+                },
+            ],
+            record_count.div_ceil(REGION_RECORD_INVOCATIONS_PER_WORKGROUP),
+            outcome_bytes,
+        );
         self.budget
             .memory_account_mut()
             .settle_the_chunk_after_its_submission(
@@ -1002,8 +1320,7 @@ impl GpuInUse {
     }
 
     /// 选段打分的一段（切批切出来的、装得下这张卡的那几个段）交给这张卡：两份核在同一次提交里各数一遍，交回每段两份的活槽数，按段里的次序。
-    /// 整份记录表随每一段交一份。本包这一段占的缓冲先记进这张卡的账（超上限断言），那次提交做完放掉；没等到它做完的留着
-    /// （`GpuMemoryAccount::settle_the_chunk_after_its_submission`）。
+    /// 这一片的记录表随每一段交一份。账同 [`CardOfTheBackgroundGpu::both_kernel_outcomes_of_a_checksum_chunk`]。
     fn both_kernel_live_slots_of_a_chunk(
         &mut self,
         segments: &[ClusterSegmentEntryKey],
@@ -1037,45 +1354,44 @@ impl GpuInUse {
         self.budget
             .memory_account_mut()
             .hold(bytes_held_by_this_chunk);
-        let (read_back, submission_of_the_chunk) = {
-            let parameter_buffer =
-                self.buffer_with_contents(&parameter_bytes, wgpu::BufferUsages::UNIFORM);
-            let segment_bound_buffer =
-                self.buffer_with_contents(&segment_bound_bytes, wgpu::BufferUsages::STORAGE);
-            let record_buffer =
-                self.buffer_with_contents(&record_bytes, wgpu::BufferUsages::STORAGE);
-            let sorted_position_buffer =
-                self.buffer_with_contents(&sorted_position_bytes, wgpu::BufferUsages::STORAGE);
-            let walking_outcomes = self.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE);
-            let searching_outcomes = self.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE);
-            self.dispatch_both_kernels_and_read_back(
+        let [parameter_buffer, segment_bound_buffer, record_buffer, sorted_position_buffer, walking_outcomes, searching_outcomes] =
+            self.buffers_created_within_the_free_memory(bytes_held_by_this_chunk, |gpu| {
                 [
-                    KernelDispatch {
-                        pipeline: &self.live_slot_walking_pipeline,
-                        bound_buffers: vec![
-                            (0, &parameter_buffer),
-                            (1, &segment_bound_buffer),
-                            (2, &record_buffer),
-                            (4, &walking_outcomes),
-                        ],
-                        outcome_buffer: &walking_outcomes,
-                    },
-                    KernelDispatch {
-                        pipeline: &self.live_slot_searching_pipeline,
-                        bound_buffers: vec![
-                            (0, &parameter_buffer),
-                            (1, &segment_bound_buffer),
-                            (2, &record_buffer),
-                            (3, &sorted_position_buffer),
-                            (4, &searching_outcomes),
-                        ],
-                        outcome_buffer: &searching_outcomes,
-                    },
-                ],
-                segment_count.div_ceil(LIVE_SLOT_INVOCATIONS_PER_WORKGROUP),
-                outcome_bytes,
-            )
-        };
+                    gpu.buffer_with_contents(&parameter_bytes, wgpu::BufferUsages::UNIFORM),
+                    gpu.buffer_with_contents(&segment_bound_bytes, wgpu::BufferUsages::STORAGE),
+                    gpu.buffer_with_contents(&record_bytes, wgpu::BufferUsages::STORAGE),
+                    gpu.buffer_with_contents(&sorted_position_bytes, wgpu::BufferUsages::STORAGE),
+                    gpu.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE),
+                    gpu.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE),
+                ]
+            })?;
+        let (read_back, submission_of_the_chunk) = self.dispatch_both_kernels_and_read_back(
+            [
+                KernelDispatch {
+                    pipeline: &self.live_slot_walking_pipeline,
+                    bound_buffers: vec![
+                        (0, &parameter_buffer),
+                        (1, &segment_bound_buffer),
+                        (2, &record_buffer),
+                        (4, &walking_outcomes),
+                    ],
+                    outcome_buffer: &walking_outcomes,
+                },
+                KernelDispatch {
+                    pipeline: &self.live_slot_searching_pipeline,
+                    bound_buffers: vec![
+                        (0, &parameter_buffer),
+                        (1, &segment_bound_buffer),
+                        (2, &record_buffer),
+                        (3, &sorted_position_buffer),
+                        (4, &searching_outcomes),
+                    ],
+                    outcome_buffer: &searching_outcomes,
+                },
+            ],
+            segment_count.div_ceil(LIVE_SLOT_INVOCATIONS_PER_WORKGROUP),
+            outcome_bytes,
+        );
         self.budget
             .memory_account_mut()
             .settle_the_chunk_after_its_submission(
@@ -1091,13 +1407,54 @@ impl GpuInUse {
     }
 }
 
-impl BackgroundGpu {
+impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
+    /// 挂载那一刻开卡之后：开成的卡（与 `discrete_gpus_after_the_opening.opened` 一一对应、同序）、开卡之后的本机独显、没点名而没收进清单的那几张、
+    /// 开卡那一组参与的卡与因显存上限不用的卡（`gpus_of_this_host::gpus_opened_at_the_mount` 交回的），进程里泄漏的那一份（丢它时记进去），
+    /// 与卡清单每次答之前现查本机在的独显的那一样（产品带着开卡的 wgpu 实例）。测试拿它把替身卡装进来。
+    ///
+    /// # Panics
+    /// 开成的卡与 `discrete_gpus_after_the_opening.opened` 张数不等。
+    #[must_use]
+    pub fn of_the_cards_opened_at_the_mount(
+        cards_on_this_host: Vec<Card>,
+        discrete_gpus_after_the_opening: DiscreteGpusAfterTheOpening,
+        unnamed_discrete_gpus_left_off_the_list: Vec<DiscreteGpuListRefused>,
+        participating: ParticipatingGpusOnThisHost,
+        left_out_under_their_memory_limit: Vec<GpuLeftOutUnderItsMemoryLimit>,
+        memory_leaked_in_this_process: GpuMemoryLeakedInThisProcess,
+        discrete_gpus_present_now: Box<dyn DiscreteGpusPresentNow + Send>,
+    ) -> Self {
+        assert_eq!(
+            cards_on_this_host.len(),
+            discrete_gpus_after_the_opening.opened.len(),
+            "开成的卡与开卡之后的独显清单一一对应"
+        );
+        Self {
+            cards_on_this_host,
+            discrete_gpus_after_the_opening,
+            unnamed_discrete_gpus_left_off_the_list,
+            participating,
+            left_out_under_their_memory_limit,
+            cost_not_measured_at_the_last_switch: None,
+            submissions_since_the_last_take: GpuSubmissionsOfTheBatch::default(),
+            gpus_leaked_when_dropped: GpusLeakedWhenTheBackgroundGpuIsDropped::not_yet_dropped(),
+            memory_leaked_in_this_process,
+            discrete_gpus_present_now,
+        }
+    }
+
     /// 挂载那一刻没开成而跳过的卡（给了 `background_gpu_devices` 而没点名的），按总线地址排，各带原因；没跳过一张是空。
     #[must_use]
     pub fn gpus_not_opened_at_the_mount(&self) -> &[DiscreteGpuNotOpenedAtTheMount] {
         &self.discrete_gpus_after_the_opening.not_opened
     }
 
+    /// 给了 `background_gpu_devices` 时没点名、交不成一张卡（缺总线地址、缺设备本地堆、地址重复）而没收进卡清单的独显，各带原因；没有是空。
+    #[must_use]
+    pub fn unnamed_discrete_gpus_left_off_the_list(&self) -> &[DiscreteGpuListRefused] {
+        &self.unnamed_discrete_gpus_left_off_the_list
+    }
+
     /// 此刻这组设置点了而因显存上限连最小的一片都装不下、不用的卡，按参与的次序，各带上限与至少要多少；没有是空。
     /// 与 [`Self::participating_gpus`] 一起看：参与的卡一张都不剩时三段每一批都交回「参与的卡一张都不剩」（[`Self::batch_not_split_over_the_gpus`]）。
     #[must_use]
@@ -1105,6 +1462,17 @@ impl BackgroundGpu {
         &self.left_out_under_their_memory_limit
     }
 
+    /// 丢掉它那一刻泄漏了哪几张卡（还没丢是 `None`）：调用方拿着这一份，丢了之后读（[`GpusLeakedWhenTheBackgroundGpuIsDropped`]）。
+    #[must_use]
+    pub fn gpus_leaked_when_dropped(&self) -> GpusLeakedWhenTheBackgroundGpuIsDropped {
+        self.gpus_leaked_when_dropped.clone()
+    }
+
+    /// 上一次交回以来三段各自在卡上是怎么交的（交成了几次、几片、重交了几次、显存不足重切了几次、分到活的是哪几张卡），交回之后清零。
+    pub fn take_the_submissions_of_the_batch(&mut self) -> GpuSubmissionsOfTheBatch {
+        std::mem::take(&mut self.submissions_since_the_last_take)
+    }
+
     /// 此刻参与的卡，按参与的次序，带此刻设置给的显存上限与量出的一次提交的主机 CPU 时间。
     ///
     /// # Panics
@@ -1115,13 +1483,13 @@ impl BackgroundGpu {
             .positions_on_this_host()
             .iter()
             .map(|position| {
-                let gpu_in_use = &self.gpus_on_this_host[*position];
+                let card = &self.cards_on_this_host[*position];
                 ParticipatingGpuReport {
-                    identifier: gpu_in_use.discrete_gpu.identifier.clone(),
-                    name: gpu_in_use.discrete_gpu.name.clone(),
-                    memory_limit: gpu_in_use.budget.memory_account().memory_limit(),
-                    measured_cost: gpu_in_use
-                        .measured_cost
+                    identifier: card.discrete_gpu().identifier.clone(),
+                    name: card.discrete_gpu().name.clone(),
+                    memory_limit: card.budget().memory_account().memory_limit(),
+                    measured_cost: card
+                        .measured_cost()
                         .expect("参与的卡都量过代价：开卡或换设置让它参与的那一刻量的"),
                 }
             })
@@ -1130,26 +1498,25 @@ impl BackgroundGpu {
 
     /// 从下一批起照核心层交来的这一组跑（D26 已定项 13 第 1–3 条；核心层只在两批之间交）：参与的卡换成它点的那几张、按它的次序，
     /// 每张卡的显存上限与等的上限换成它给的（不参与的卡不设显存上限，它分不到活）。不重开设备，各卡的账照旧记着。
-    /// 显存上限连最小的一片都装不下的卡不用、报出来（[`Self::gpus_left_out_under_their_memory_limit`]）；开卡时因显存上限没量代价、这一组让它参与的卡
-    /// 照新上限量一次，量出错的这张卡不参与，之后每一批报成它的 GPU 出错，直到下一次换设置。
+    /// 显存上限（扣掉卡上已占的）连最小的一片都装不下的卡不用、报出来（[`Self::gpus_left_out_under_their_memory_limit`]）；开卡时因显存上限没量代价、
+    /// 这一组让它参与的卡照新上限量一次，量出错的这张卡不参与，之后每一批报成它的 GPU 出错，直到下一次换设置。
     ///
     /// # Panics
-    /// 它点的卡有一张不在开成的独显里：核心层照这份实现答的卡清单（本机全部独显，连同没开成的那几张）校验过，参与的卡里有没开成的就拒了。
+    /// 它点的卡有一张不在开成的独显里：核心层照这份实现答的卡清单（此刻本机在的独显，连同没开成的那几张）校验过，参与的卡里有没开成的就拒了。
     pub fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings) {
         let identifiers_on_this_host: Vec<GpuDeviceIdentifier> = self
-            .gpus_on_this_host
+            .cards_on_this_host
             .iter()
-            .map(|gpu_in_use| gpu_in_use.discrete_gpu.identifier.clone())
+            .map(|card| card.discrete_gpu().identifier.clone())
             .collect();
         let participating_by_the_settings =
             ParticipatingGpusOnThisHost::of_the_settings(settings, &identifiers_on_this_host);
-        for gpu_in_use in &mut self.gpus_on_this_host {
+        for card in &mut self.cards_on_this_host {
             let memory_limit = memory_limit_of_the_device_under_the_settings(
                 settings,
-                &gpu_in_use.discrete_gpu.identifier,
+                &card.discrete_gpu().identifier,
             );
-            gpu_in_use
-                .budget
+            card.budget_mut()
                 .use_the_settings_from_the_next_batch(memory_limit, settings.wait_limit());
         }
         let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds();
@@ -1157,24 +1524,22 @@ impl BackgroundGpu {
         let mut cost_not_measured = None;
         let mut positions_left_out = Vec::new();
         for position in participating_by_the_settings.positions_on_this_host() {
-            let gpu_in_use = &mut self.gpus_on_this_host[*position];
+            let card = &mut self.cards_on_this_host[*position];
             if let Some(left_out) = gpu_left_out_under_its_memory_limit(
-                &gpu_in_use.discrete_gpu.identifier,
-                gpu_in_use.budget.memory_account().memory_limit(),
+                &card.discrete_gpu().identifier,
+                card.budget().memory_account().memory_limit(),
                 smallest_piece_bytes,
+                card.budget().memory_account().bytes_held(),
             ) {
                 left_out_under_their_memory_limit.push(left_out);
                 positions_left_out.push(*position);
                 continue;
             }
-            if gpu_in_use.measured_cost.is_none() {
-                match gpu_in_use.measured_cost_of_one_submission() {
-                    Ok(measured_cost) => gpu_in_use.measured_cost = Some(measured_cost),
-                    Err(cause) => {
-                        positions_left_out.push(*position);
-                        cost_not_measured
-                            .get_or_insert((gpu_in_use.discrete_gpu.identifier.clone(), cause));
-                    }
+            if card.measured_cost().is_none() {
+                if let Err(cause) = card.measure_the_cost_of_one_submission() {
+                    positions_left_out.push(*position);
+                    cost_not_measured
+                        .get_or_insert((card.discrete_gpu().identifier.clone(), cause));
                 }
             }
         }
@@ -1209,119 +1574,254 @@ impl BackgroundGpu {
         ))
     }
 
-    /// 一批的条目按量出的代价分给参与的卡：（它在本机里的位置，分到的那一段相连的条目），按参与的次序。没有条目交回空。
-    fn entry_ranges_of_a_batch(
-        &self,
+    /// 一批（`entry_count` 条）分给参与的卡、各卡切好（`cut` 切一张卡分到的那几条，照这张卡交活之前那一刻的地方）：交活之前先等这张卡上前一次超时的那次提交
+    /// （等满上限算这一批这张卡超时一次、再等，`waiting_for_the_gpu::outcome_waited_for_again_after_timeouts`），切不出最小的一片（显存上限扣掉已占的）的卡
+    /// 这一批不分给它（`gpus_of_this_host::plans_of_a_batch_leaving_out_the_gpus_short_of_the_smallest_piece`）。没有条目交回空。
+    fn plans_of_a_batch<Plan>(
+        &mut self,
         entry_count: usize,
-    ) -> Result<Vec<(usize, std::ops::Range<usize>)>, BatchNotSplitOverTheGpus> {
-        let not_split = self.batch_not_split_over_the_gpus();
-        match not_split {
-            Some(
-                cost_not_measured @ BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch { .. },
-            ) => {
-                return Err(cost_not_measured);
+        submissions_of_each_card: &mut [SubmissionsOfACardInTheBatch],
+        cut: impl Fn(Range<usize>, ChunkBoundsOfAGpu) -> Result<Plan, SmallestPieceLargerThanTheRoom>,
+    ) -> Result<Vec<PlanOfAGpuForTheBatch<Plan>>, OutcomesOfTheBatchNotComputed> {
+        match self.batch_not_split_over_the_gpus() {
+            Some(BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch { device, cause }) => {
+                return Err(OutcomesOfTheBatchNotComputed::GpuFailed { device, cause });
+            }
+            Some(BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(gpus_left_out)) => {
+                if entry_count == 0 {
+                    return Ok(Vec::new());
+                }
+                return Err(OutcomesOfTheBatchNotComputed::EveryParticipatingGpuLeftOut(
+                    gpus_left_out,
+                ));
             }
-            Some(BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(_)) | None => {}
+            None => {}
         }
         if entry_count == 0 {
             return Ok(Vec::new());
         }
-        if let Some(every_participating_gpu_left_out) = not_split {
-            return Err(every_participating_gpu_left_out);
-        }
-        Ok(self.participating.entry_ranges_by_the_measured_cost_of(
+        let measured_costs: Vec<Option<HostCpuNanosecondsPerSubmission>> = self
+            .cards_on_this_host
+            .iter()
+            .map(CardOfTheBackgroundGpu::measured_cost)
+            .collect();
+        let cards_on_this_host = &mut self.cards_on_this_host;
+        plans_of_a_batch_leaving_out_the_gpus_short_of_the_smallest_piece(
+            &self.participating,
             |position| {
-                self.gpus_on_this_host[position]
-                    .measured_cost
+                measured_costs[position]
                     .expect("参与的卡都量过代价：开卡或换设置让它参与的那一刻量的")
             },
             entry_count,
-        ))
+            |position| {
+                let card = &mut cards_on_this_host[position];
+                let device = card.discrete_gpu().identifier.clone();
+                let room = outcome_waited_for_again_after_timeouts(
+                    &mut *card,
+                    &mut submissions_of_each_card[position].timeouts,
+                    Card::room_once_the_submission_running_past_its_wait_completed,
+                )
+                .map_err(|cause| BatchNotPlannedOverTheGpus::GpuFailed {
+                    device: device.clone(),
+                    cause,
+                })?;
+                let memory_account = card.budget().memory_account();
+                Ok(RoomOfAGpuForTheBatch {
+                    device,
+                    room,
+                    memory_limit: memory_account.memory_limit(),
+                    bytes_already_held_on_the_device: memory_account.bytes_held(),
+                })
+            },
+            cut,
+        )
+        .map_err(|not_planned| match not_planned {
+            BatchNotPlannedOverTheGpus::GpuFailed { device, cause } => {
+                OutcomesOfTheBatchNotComputed::GpuFailed { device, cause }
+            }
+            BatchNotPlannedOverTheGpus::EveryParticipatingGpuLeftOut(gpus_left_out) => {
+                OutcomesOfTheBatchNotComputed::EveryParticipatingGpuLeftOut(gpus_left_out)
+            }
+            BatchNotPlannedOverTheGpus::GpuCannotHoldTheSmallestPiece(smallest_piece) => {
+                OutcomesOfTheBatchNotComputed::GpuCannotHoldTheSmallestPiece(smallest_piece)
+            }
+        })
     }
 
-    /// 一批单元交两份核各算一遍，交回每条两份的结果（按交进来的次序），不比对、不动计数：对拍与 [`Self::verify_unit_checksums`] 都用它。
-    /// 按量出的代价分给各卡（每张卡一段相连的条目），每张卡上再按此刻剩下的显存切段，单独成一段都装不下的单元切成几片、逐片交、
-    /// 各片的值两份各按自己的合法合成整单元的（`checksum_kernels::checksum_outcomes_of_the_units_cut_into_pieces`）。分到活的卡上有前一次等超过上限、
-    /// 还在跑的提交，切段之前先等它做完（等不到交回 [`ChecksumBatchFailed::GpuFailed`]）；先把各卡的切法都算完，有一张卡连最小的一片都装不下或等不到
-    /// 就一张卡都不交。一段等超过上限在同一张卡上重交至多 3 次（`waiting_for_the_gpu::outcome_resubmitted_after_timeouts`）。
+    /// 一段（`segment`）的一批照三段共用的那一套交给参与的卡：分活、切（[`Self::plans_of_a_batch`]），再逐张卡照切法交
+    /// （`run_the_plan_on_a_card`：这一段自己的交法，交它这张卡、分到的那几条、切法与这一批这张卡的那一份，每一段套重交）。
+    /// 建缓冲显存不足（[`GpuFailureCause::OutOfMemoryCreatingTheBuffers`]）就把这张卡这一批的地方压到建不成的那一段的一半、重切、从头再交它分到的那几条
+    /// （第 3 条「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」）；压到连最小的一片都切不出就交回
+    /// [`GpuFailureCause::FreeMemoryShortOfTheSmallestPiece`]（条款没写，第一版不支持）。各卡交回的接起来，按条目的次序。
+    /// 交成了几次、几片、重交了几次、重切了几次、哪几张卡分到了活，记进这一段的那一份（出错也记，[`Self::take_the_submissions_of_the_batch`]）。
     ///
-    /// # Errors
-    /// [`ChecksumBatchFailed`] 的两种。
-    pub fn both_kernel_outcomes_of_the_batch<EntryKey>(
+    /// 重切那一圈：每一轮要么交回、要么这张卡的显存那一项严格变小（建不成的那一段是按此刻的界切的，不比界大，压到它的一半），压到切不出就交回，所以有界。
+    fn outcomes_of_a_batch_over_the_cards<Plan, Outcome>(
         &mut self,
-        units: &[UnitChecksumToVerify<'_, EntryKey>],
-    ) -> Result<Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)>, ChecksumBatchFailed> {
-        let ranges_per_gpu = self
-            .entry_ranges_of_a_batch(units.len())
-            .map_err(|not_split| match not_split {
-                BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch { device, cause } => {
-                    ChecksumBatchFailed::GpuFailed { device, cause }
+        segment: GpuComputedSegment,
+        entry_count: usize,
+        cut: impl Fn(Range<usize>, ChunkBoundsOfAGpu) -> Result<Plan, SmallestPieceLargerThanTheRoom>,
+        mut run_the_plan_on_a_card: impl FnMut(
+            &mut Card,
+            Range<usize>,
+            &Plan,
+            &mut SubmissionsOfACardInTheBatch,
+        ) -> Result<Vec<Outcome>, GpuFailureCause>,
+    ) -> Result<Vec<Outcome>, OutcomesOfTheBatchNotComputed> {
+        let mut submissions_of_each_card: Vec<SubmissionsOfACardInTheBatch> = self
+            .cards_on_this_host
+            .iter()
+            .map(|_card| SubmissionsOfACardInTheBatch::none_yet())
+            .collect();
+        let plans = match self.plans_of_a_batch(entry_count, &mut submissions_of_each_card, &cut) {
+            Ok(plans) => plans,
+            Err(not_planned) => {
+                self.note_the_submissions_of_the_segment(
+                    segment,
+                    &submissions_of_each_card,
+                    Vec::new(),
+                    0,
+                );
+                return Err(not_planned);
+            }
+        };
+        let mut outcomes = Vec::with_capacity(entry_count);
+        let mut devices_given_work = Vec::with_capacity(plans.len());
+        let mut recuts_after_running_out_of_memory: u64 = 0;
+        let mut failure = None;
+        for planned in plans {
+            let card = &mut self.cards_on_this_host[planned.position_on_this_host];
+            let device = card.discrete_gpu().identifier.clone();
+            devices_given_work.push(device.clone());
+            let submissions_of_the_card =
+                &mut submissions_of_each_card[planned.position_on_this_host];
+            let mut room = planned.room;
+            let mut plan = planned.plan;
+            let outcomes_of_the_card = loop {
+                match run_the_plan_on_a_card(
+                    &mut *card,
+                    planned.entries.clone(),
+                    &plan,
+                    &mut *submissions_of_the_card,
+                ) {
+                    Err(GpuFailureCause::OutOfMemoryCreatingTheBuffers {
+                        bytes_held_by_the_chunk,
+                        reason: _,
+                    }) => {
+                        recuts_after_running_out_of_memory += 1;
+                        room = room.with_the_memory_limit_at_most(bytes_held_by_the_chunk / 2);
+                        match cut(planned.entries.clone(), room) {
+                            Ok(recut) => plan = recut,
+                            Err(smallest_piece) => {
+                                break Err(GpuFailureCause::FreeMemoryShortOfTheSmallestPiece {
+                                    smallest_piece_bytes: smallest_piece.smallest_piece_bytes,
+                                })
+                            }
+                        }
+                    }
+                    computed => break computed,
                 }
-                BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(left_out) => {
-                    ChecksumBatchFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit(left_out)
+            };
+            match outcomes_of_the_card {
+                Ok(of_the_card) => outcomes.extend(of_the_card),
+                Err(cause) => {
+                    failure = Some(OutcomesOfTheBatchNotComputed::GpuFailed { device, cause });
+                    break;
                 }
-            })?;
-        let mut pieces_per_gpu = Vec::with_capacity(ranges_per_gpu.len());
-        for (position_on_this_host, range_of_the_gpu) in &ranges_per_gpu {
-            let gpu_in_use = &mut self.gpus_on_this_host[*position_on_this_host];
-            if range_of_the_gpu.is_empty() {
-                pieces_per_gpu.push(Vec::new());
-                continue;
             }
-            let device = gpu_in_use.discrete_gpu.identifier.clone();
-            let room = gpu_in_use
-                .room_once_the_submission_running_past_its_wait_completed()
-                .map_err(|cause| ChecksumBatchFailed::GpuFailed {
-                    device: device.clone(),
-                    cause,
-                })?;
-            let unit_lengths: Vec<u64> = units[range_of_the_gpu.clone()]
-                .iter()
-                .map(|unit| u64::try_from(unit.unit_bytes.len()).expect("装得进 u64"))
-                .collect();
-            let pieces =
-                pieces_cut_to_fit_on_the_gpu(&unit_lengths, CHECKSUM_CHUNK_FOOTPRINT, room)
-                    .map_err(|smallest_piece| {
-                        ChecksumBatchFailed::GpuCannotHoldTheSmallestPiece(
-                            gpu_in_use
-                                .smallest_piece_not_held(range_of_the_gpu.start, smallest_piece),
-                        )
-                    })?;
-            pieces_per_gpu.push(pieces);
         }
-        let mut outcomes = Vec::with_capacity(units.len());
-        for ((position_on_this_host, range_of_the_gpu), pieces) in
-            ranges_per_gpu.iter().zip(&pieces_per_gpu)
-        {
-            let gpu_in_use = &mut self.gpus_on_this_host[*position_on_this_host];
-            let device = gpu_in_use.discrete_gpu.identifier.clone();
-            let outcomes_of_the_gpu = checksum_outcomes_of_the_units_cut_into_pieces(
-                &units[range_of_the_gpu.clone()],
-                pieces,
-                |piece_units| {
-                    outcome_resubmitted_after_timeouts(
-                        &mut *gpu_in_use,
-                        GpuInUse::wait_for_the_timed_out_submission,
-                        |gpu| gpu.both_kernel_outcomes_of_a_chunk(piece_units),
-                    )
-                },
-            )
-            .map_err(|cause| ChecksumBatchFailed::GpuFailed { device, cause })?;
-            outcomes.extend(outcomes_of_the_gpu);
+        self.note_the_submissions_of_the_segment(
+            segment,
+            &submissions_of_each_card,
+            devices_given_work,
+            recuts_after_running_out_of_memory,
+        );
+        if let Some(failure) = failure {
+            return Err(failure);
         }
         assert_eq!(
             outcomes.len(),
-            units.len(),
+            entry_count,
             "每条都有两份结果，按卡的次序接回"
         );
         Ok(outcomes)
     }
 
+    /// 把一段这一批在各卡上交的记进这一段的那一份（同一段一批里算了几次的加起来）。
+    fn note_the_submissions_of_the_segment(
+        &mut self,
+        segment: GpuComputedSegment,
+        submissions_of_each_card: &[SubmissionsOfACardInTheBatch],
+        devices_given_work: Vec<GpuDeviceIdentifier>,
+        recuts_after_running_out_of_memory: u64,
+    ) {
+        let submissions_of_the_segment = GpuSubmissionsOfOneSegment {
+            submissions_completed: submissions_of_each_card
+                .iter()
+                .map(|of_the_card| of_the_card.submissions_completed)
+                .sum(),
+            pieces_submitted: submissions_of_each_card
+                .iter()
+                .map(|of_the_card| of_the_card.pieces_submitted)
+                .sum(),
+            resubmissions_after_timeouts: submissions_of_each_card
+                .iter()
+                .map(|of_the_card| u64::from(of_the_card.timeouts.resubmissions()))
+                .sum(),
+            recuts_after_running_out_of_memory,
+            devices_given_work,
+        };
+        self.submissions_since_the_last_take
+            .of_the_segment_mut(segment)
+            .add(&submissions_of_the_segment);
+    }
+
+    /// 一批单元交两份核各算一遍，交回每条两份的结果（按交进来的次序），不比对、不动计数：对拍与 [`Self::verify_unit_checksums`] 都用它。
+    /// 按量出的代价分给各卡（每张卡一段相连的条目），每张卡上再按此刻剩下的显存切段，单独成一段都装不下的单元切成几片、逐片交、
+    /// 各片的值两份各按自己的合法合成整单元的（`checksum_kernels::checksum_outcomes_of_the_units_cut_into_pieces`）；分活、切段、超时、重切照
+    /// [`Self::outcomes_of_a_batch_over_the_cards`]。
+    ///
+    /// # Errors
+    /// [`ChecksumBatchFailed`] 的三种。
+    pub fn both_kernel_outcomes_of_the_batch<EntryKey>(
+        &mut self,
+        units: &[UnitChecksumToVerify<'_, EntryKey>],
+    ) -> Result<Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)>, ChecksumBatchFailed> {
+        let unit_lengths: Vec<u64> = units
+            .iter()
+            .map(|unit| u64::try_from(unit.unit_bytes.len()).expect("装得进 u64"))
+            .collect();
+        self.outcomes_of_a_batch_over_the_cards(
+            GpuComputedSegment::UnitChecksumVerification,
+            units.len(),
+            |entries, room| {
+                pieces_cut_to_fit_on_the_gpu(&unit_lengths[entries], CHECKSUM_CHUNK_FOOTPRINT, room)
+            },
+            |card, entries, chunks_of_pieces, submissions_of_the_card| {
+                checksum_outcomes_of_the_units_cut_into_pieces(
+                    &units[entries],
+                    chunks_of_pieces,
+                    |piece_units| {
+                        submissions_of_the_card.outcome_of_a_chunk(
+                            &mut *card,
+                            piece_units.len(),
+                            |card_handed_the_chunk| {
+                                card_handed_the_chunk
+                                    .both_kernel_outcomes_of_a_checksum_chunk(piece_units)
+                            },
+                        )
+                    },
+                )
+            },
+        )
+        .map_err(checksum_batch_failed_of)
+    }
+
     /// 扫描里的校验和核对一批（第 8 条）：两份核各算一遍、逐条比对；连续不确定到 N 轮的条目交核心层的 CRC-32C 现判。
-    /// 交回逐条的判定与确认、不确定、交 CPU 判三个计数。GPU 出错（含重交 3 次都等超过上限）时交回出错，`uncertainty` 不动。
+    /// 交回逐条的判定与确认、不确定、交 CPU 判三个计数。GPU 出错（含这一批这张卡第 4 次等超过上限）时交回出错，`uncertainty` 不动。
     ///
     /// # Errors
-    /// [`ChecksumBatchFailed`] 的两种。
+    /// [`ChecksumBatchFailed`] 的三种。
     pub fn verify_unit_checksums<EntryKey: Eq + std::hash::Hash + Clone>(
         &mut self,
         units: &[UnitChecksumToVerify<'_, EntryKey>],
@@ -1336,11 +1836,10 @@ impl BackgroundGpu {
     /// `mapping_entries` 是这一版的中央映射条目（与核心层区域扫描建按身份索引的是同一份）。
     /// 按量出的代价分给各卡，每张卡上再按此刻剩下的显存切段（整份映射每段都占一份；连同一条记录装不下时映射按落点切成几片逐片交，
     /// 最大那条记录连同一个落点上的条目还装不下时单元切成头一片与之后的几片，`region_record_kernels::region_record_plan_on_a_gpu`；
-    /// 用户 2026-10-28 定「装不下应该自适应的分片，缩小逐次进行」）：分到活的卡上有前一次等超过上限、还在跑的提交，切段之前先等它做完
-    /// （等不到交回 [`RegionRecordBatchFailed::GpuFailed`]）；先把各卡的切法都算完，放不下或等不到就一张卡都不交。一段等超过上限在同一张卡上重交至多 3 次。
+    /// 用户 2026-10-28 定「装不下应该自适应的分片，缩小逐次进行」）；分活、切段、超时、重切照 [`Self::outcomes_of_a_batch_over_the_cards`]。
     ///
     /// # Errors
-    /// [`RegionRecordBatchFailed`] 的三种。
+    /// [`RegionRecordBatchFailed`] 的四种。
     ///
     /// # Panics
     /// 有一条记录不是区域扫描会去读的那一种（`region_record_kernels::record_is_one_the_region_scan_reads`）：调用方先按 R 筛过。
@@ -1358,88 +1857,45 @@ impl BackgroundGpu {
         );
         let mapping_tables = MappingTablesForTheKernels::of_the_mapping_entries(mapping_entries)
             .map_err(RegionRecordBatchFailed::MappingKeyLongerThanTheKernelsHold)?;
-        let ranges_per_gpu = self
-            .entry_ranges_of_a_batch(records.len())
-            .map_err(|not_split| match not_split {
-                BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch { device, cause } => {
-                    RegionRecordBatchFailed::GpuFailed { device, cause }
-                }
-                BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(left_out) => {
-                    RegionRecordBatchFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit(
-                        left_out,
-                    )
-                }
-            })?;
-        let mut plans_per_gpu = Vec::with_capacity(ranges_per_gpu.len());
-        for (position_on_this_host, range_of_the_gpu) in &ranges_per_gpu {
-            let gpu_in_use = &mut self.gpus_on_this_host[*position_on_this_host];
-            if range_of_the_gpu.is_empty() {
-                plans_per_gpu.push(RegionRecordPlanOnAGpu {
-                    bytes_from_the_start_of_each_record: Vec::new(),
-                    chunks_of_the_pieces_after_the_heads: Vec::new(),
-                    mapping_slices: Vec::new(),
-                });
-                continue;
-            }
-            let device = gpu_in_use.discrete_gpu.identifier.clone();
-            let room = gpu_in_use
-                .room_once_the_submission_running_past_its_wait_completed()
-                .map_err(|cause| RegionRecordBatchFailed::GpuFailed {
-                    device: device.clone(),
-                    cause,
-                })?;
-            let plan = region_record_plan_on_a_gpu(
-                &records[range_of_the_gpu.clone()],
-                mapping_entries,
-                &mapping_tables,
-                room,
-            )
-            .map_err(|smallest_piece| {
-                RegionRecordBatchFailed::GpuCannotHoldTheSmallestPiece(
-                    gpu_in_use.smallest_piece_not_held(range_of_the_gpu.start, smallest_piece),
-                )
-            })?;
-            plans_per_gpu.push(plan);
-        }
-        let mut classifications = Vec::with_capacity(records.len());
-        for ((position_on_this_host, range_of_the_gpu), plan) in
-            ranges_per_gpu.iter().zip(&plans_per_gpu)
-        {
-            let gpu_in_use = &mut self.gpus_on_this_host[*position_on_this_host];
-            let device = gpu_in_use.discrete_gpu.identifier.clone();
-            let classifications_of_the_gpu = region_record_classifications_of_the_plan(
-                &records[range_of_the_gpu.clone()],
-                plan,
-                |chunk_pieces, mapping_slice| {
-                    outcome_resubmitted_after_timeouts(
-                        &mut *gpu_in_use,
-                        GpuInUse::wait_for_the_timed_out_submission,
-                        |gpu| {
-                            gpu.both_kernel_outcomes_of_a_region_record_chunk(
-                                chunk_pieces,
-                                mapping_slice.map(|slice| &slice.tables),
-                            )
-                        },
-                    )
-                },
-            )
-            .map_err(|cause| RegionRecordBatchFailed::GpuFailed { device, cause })?;
-            classifications.extend(classifications_of_the_gpu);
-        }
-        assert_eq!(
-            classifications.len(),
+        self.outcomes_of_a_batch_over_the_cards(
+            GpuComputedSegment::RegionRecordClassification,
             records.len(),
-            "每条都有两份分类，按卡的次序接回"
-        );
-        Ok(classifications)
+            |entries, room| {
+                region_record_plan_on_a_gpu(
+                    &records[entries],
+                    mapping_entries,
+                    &mapping_tables,
+                    room,
+                )
+            },
+            |card, entries, plan, submissions_of_the_card| {
+                region_record_classifications_of_the_plan(
+                    &records[entries],
+                    plan,
+                    |chunk_pieces, mapping_slice| {
+                        submissions_of_the_card.outcome_of_a_chunk(
+                            &mut *card,
+                            chunk_pieces.len(),
+                            |card_handed_the_chunk| {
+                                card_handed_the_chunk.both_kernel_outcomes_of_a_region_record_chunk(
+                                    chunk_pieces,
+                                    mapping_slice.map(|slice| &slice.tables),
+                                )
+                            },
+                        )
+                    },
+                )
+            },
+        )
+        .map_err(region_record_batch_failed_of)
     }
 
     /// 区域扫描里的单元头解析与映射反查一批（第 1、8 条）：两份核各分一次类、逐条比对；连续不确定到 N 轮的记录交核心层的区域扫描现判
     /// （`region_record_kernels::cpu_reference_classification`，按身份的索引只在头一次要它时建）。交回逐条的判定与三个计数。
-    /// GPU 出错（含重交 3 次都等超过上限）或这一批交不成时交回出错，`uncertainty` 不动。
+    /// GPU 出错或这一批交不成时交回出错，`uncertainty` 不动。
     ///
     /// # Errors
-    /// [`RegionRecordBatchFailed`] 的三种。
+    /// [`RegionRecordBatchFailed`] 的四种。
     ///
     /// # Panics
     /// 同 [`Self::both_kernel_classifications_of_the_region_records`]。
@@ -1467,11 +1923,10 @@ impl BackgroundGpu {
     /// 选段打分一批交两份核各数一遍，交回每段两份的活槽数（按 `segments` 的次序，逐条走遍记录那一份在前），不比对、不动计数：对拍与
     /// [`Self::count_live_slots_per_cluster_segment`] 都用它。`segments` 是 `unit_area` 那块盘上的整段（核心层按单元区给的条目键）。
     /// 按量出的代价分给各卡，每张卡上再按此刻剩下的显存切段（整份记录表每段都占一份；连同一个段装不下时记录表按记录切成几片、各片数出的逐段相加，
-    /// `live_slot_kernels::live_slot_plan_on_a_gpu`）：分到活的卡上有前一次等超过上限、还在跑的提交，切段之前先等它做完
-    /// （等不到交回 [`LiveSlotBatchFailed::GpuFailed`]）；先把各卡的切法都算完，放不下或等不到就一张卡都不交。一段等超过上限在同一张卡上重交至多 3 次。
+    /// `live_slot_kernels::live_slot_plan_on_a_gpu`）；分活、切段、超时、重切照 [`Self::outcomes_of_a_batch_over_the_cards`]。
     ///
     /// # Errors
-    /// [`LiveSlotBatchFailed`] 的两种。
+    /// [`LiveSlotBatchFailed`] 的三种。
     ///
     /// # Panics
     /// 有一个段不在 `unit_area` 那块盘上。
@@ -1487,81 +1942,40 @@ impl BackgroundGpu {
                 .all(|segment| segment.device == unit_area.device),
             "交进来的段都在单元区那块盘上"
         );
-        let ranges_per_gpu = self
-            .entry_ranges_of_a_batch(segments.len())
-            .map_err(|not_split| match not_split {
-                BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch { device, cause } => {
-                    LiveSlotBatchFailed::GpuFailed { device, cause }
-                }
-                BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(left_out) => {
-                    LiveSlotBatchFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit(left_out)
-                }
-            })?;
-        let mut plans_per_gpu = Vec::with_capacity(ranges_per_gpu.len());
-        for (position_on_this_host, range_of_the_gpu) in &ranges_per_gpu {
-            let gpu_in_use = &mut self.gpus_on_this_host[*position_on_this_host];
-            if range_of_the_gpu.is_empty() {
-                plans_per_gpu.push(Vec::new());
-                continue;
-            }
-            let device = gpu_in_use.discrete_gpu.identifier.clone();
-            let room = gpu_in_use
-                .room_once_the_submission_running_past_its_wait_completed()
-                .map_err(|cause| LiveSlotBatchFailed::GpuFailed {
-                    device: device.clone(),
-                    cause,
-                })?;
-            let plan =
-                live_slot_plan_on_a_gpu(allocation_records.len(), range_of_the_gpu.len(), room)
-                    .map_err(|smallest_piece| {
-                        LiveSlotBatchFailed::GpuCannotHoldTheSmallestPiece(
-                            gpu_in_use
-                                .smallest_piece_not_held(range_of_the_gpu.start, smallest_piece),
-                        )
-                    })?;
-            plans_per_gpu.push(plan);
-        }
-        let mut live_slots = Vec::with_capacity(segments.len());
-        for ((position_on_this_host, range_of_the_gpu), plan) in
-            ranges_per_gpu.iter().zip(&plans_per_gpu)
-        {
-            let gpu_in_use = &mut self.gpus_on_this_host[*position_on_this_host];
-            let device = gpu_in_use.discrete_gpu.identifier.clone();
-            let live_slots_of_the_gpu = live_slots_summed_over_the_record_slices(
-                allocation_records,
-                &segments[range_of_the_gpu.clone()],
-                plan,
-                |chunk_segments, _records_of_the_slice, record_tables| {
-                    outcome_resubmitted_after_timeouts(
-                        &mut *gpu_in_use,
-                        GpuInUse::wait_for_the_timed_out_submission,
-                        |gpu| {
-                            gpu.both_kernel_live_slots_of_a_chunk(
-                                chunk_segments,
-                                record_tables,
-                                unit_area,
-                            )
-                        },
-                    )
-                },
-            )
-            .map_err(|cause| LiveSlotBatchFailed::GpuFailed { device, cause })?;
-            live_slots.extend(live_slots_of_the_gpu);
-        }
-        assert_eq!(
-            live_slots.len(),
+        self.outcomes_of_a_batch_over_the_cards(
+            GpuComputedSegment::LiveSlotsPerClusterSegment,
             segments.len(),
-            "每段都有两份活槽数，按卡的次序接回"
-        );
-        Ok(live_slots)
+            |entries, room| live_slot_plan_on_a_gpu(allocation_records.len(), entries.len(), room),
+            |card, entries, plan, submissions_of_the_card| {
+                live_slots_summed_over_the_record_slices(
+                    allocation_records,
+                    &segments[entries],
+                    plan,
+                    |chunk_segments, _records_of_the_slice, record_tables| {
+                        submissions_of_the_card.outcome_of_a_chunk(
+                            &mut *card,
+                            chunk_segments.len(),
+                            |card_handed_the_chunk| {
+                                card_handed_the_chunk.both_kernel_live_slots_of_a_chunk(
+                                    chunk_segments,
+                                    record_tables,
+                                    unit_area,
+                                )
+                            },
+                        )
+                    },
+                )
+            },
+        )
+        .map_err(live_slot_batch_failed_of)
     }
 
     /// 选段打分一批（第 1、8 条）：两份核各数一遍每段的活槽数、逐段比对；连续不确定到 N 轮的段交核心层的
     /// `live_slots_per_cluster_segment_by_the_records` 现判（`live_slot_kernels::cpu_reference_live_slots`）。交回逐段的判定与三个计数。
-    /// GPU 出错（含重交 3 次都等超过上限）或这一批交不成时交回出错，`uncertainty` 不动。
+    /// GPU 出错或这一批交不成时交回出错，`uncertainty` 不动。
     ///
     /// # Errors
-    /// [`LiveSlotBatchFailed`] 的两种。
+    /// [`LiveSlotBatchFailed`] 的三种。
     ///
     /// # Panics
     /// `segments` 不是 `unit_area` 的整段按起点升序一个不漏（`live_slot_kernels::reconciled_live_slots_of_the_segments`）。
@@ -1590,30 +2004,50 @@ impl BackgroundGpu {
 /// 丢后台 GPU（卸载、丢掉核心层收的把手、测试里展开）：不等还在 GPU 上跑的提交（用户 2026-10-28 定「卸载时同样不等、泄漏设备」）——
 /// 还记着一次等超过上限、没做完的提交的卡整张泄漏（设备、队列、管线都不析构：wgpu-core 30.0.1 的 `Queue` 析构调 `wait_for_idle`，
 /// 即 Vulkan 的 `vkQueueWaitIdle`，不带上限），别的卡照常丢（`gpus_of_this_host::gpus_dropped_leaking_those_still_running`）。
-impl Drop for BackgroundGpu {
+/// 泄漏了哪几张、各占着多少字节报给调用方（[`BackgroundGpuOverTheCards::gpus_leaked_when_dropped`]），并记进进程里的那一份，新一次挂载先扣掉。
+impl<Card: CardOfTheBackgroundGpu> Drop for BackgroundGpuOverTheCards<Card> {
     fn drop(&mut self) {
-        let gpus_on_this_host = std::mem::take(&mut self.gpus_on_this_host);
-        let _gpus_leaked =
-            gpus_dropped_leaking_those_still_running(gpus_on_this_host, |gpu_in_use| {
-                gpu_in_use
-                    .budget
+        let cards_on_this_host = std::mem::take(&mut self.cards_on_this_host);
+        let gpus_leaked = gpus_dropped_leaking_those_still_running(
+            cards_on_this_host,
+            |card| {
+                card.budget()
                     .memory_account()
                     .has_a_submission_running_past_its_wait()
-            });
+            },
+            |card| GpuLeakedWhenDropped {
+                device: card.discrete_gpu().identifier.clone(),
+                bytes_held_on_the_device: card.budget().memory_account().bytes_held(),
+            },
+        );
+        self.memory_leaked_in_this_process
+            .note_the_gpus_leaked_when_dropped(&gpus_leaked);
+        self.gpus_leaked_when_dropped.note_the_drop(gpus_leaked);
     }
 }
 
-/// 卡清单（核心层挂载与切换照它校验设置，D26（后台整理与放置回收） 已定项 13 第 2、4 条）：本机全部独显与各自的显存总量，按总线地址排
+/// 卡清单（核心层挂载与切换照它校验设置，D26（后台整理与放置回收） 已定项 13 第 2、4 条）：此刻本机在的独显与各自的显存总量，按总线地址排
 /// （挂载那一刻没开成的照列；没给 `background_gpu_devices` 时上限按这个次序一张卡一个），连同没开成的那几张与原因（[`DiscreteGpusAfterTheOpening`]）。
-/// 切换可以点开包时没参与而开成了的卡；点名不在本机的卡照「点名的卡不存在」拒，参与的卡里有没开成的照第 4 条拒。
-impl GpuInventory for BackgroundGpu {
+/// 每次答之前现查本机在的独显（[`DiscreteGpusPresentNow`]）：挂载之后掉了的卡不列，切换点它照「点名的卡不存在」拒、不点名时它不参与、一张都不剩照「GPU 不在」拒。
+/// 切换可以点开包时没参与而开成了的卡；参与的卡里有没开成的照第 4 条拒。
+impl<Card: CardOfTheBackgroundGpu> GpuInventory for BackgroundGpuOverTheCards<Card> {
     fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
         self.discrete_gpus_after_the_opening
+            .still_present(
+                &self
+                    .discrete_gpus_present_now
+                    .identifiers_of_the_discrete_gpus_present_now(),
+            )
             .devices_with_total_memory()
     }
 
     fn devices_not_opened_at_the_mount(&self) -> Vec<GpuDeviceNotOpenedAtTheMount> {
         self.discrete_gpus_after_the_opening
+            .still_present(
+                &self
+                    .discrete_gpus_present_now
+                    .identifiers_of_the_discrete_gpus_present_now(),
+            )
             .devices_not_opened_at_the_mount()
     }
 }
@@ -1783,4 +2217,694 @@ mod tests {
         }
         std::hint::black_box(accumulator);
     }
+
+    use std::collections::VecDeque;
+
+    use singlefs_core::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
+    use singlefs_core::allocator::{AllocationRecordQuarantineMark, UnitAreaStart};
+    use singlefs_core::background_compute::admit_gpu_settings_against_the_inventory;
+    use singlefs_core::background_compute::{
+        BackgroundComputeBackend, BackgroundComputeSettings, BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
+    };
+
+    use crate::checksum_kernels::cpu_reference_outcome;
+    use crate::live_slot_kernels::live_slot_chunk_footprint;
+    use crate::memory_limit_and_chunks::SubmissionOfTheChunk;
+    use crate::two_kernel_reconciliation::EntryDecision;
+
+    const WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU: Duration = Duration::from_secs(10);
+    const EIGHT_GIBIBYTES: u64 = 8 << 30;
+
+    /// 替身卡上的钟不走：替身等前一次提交照脚本当场交回，不真等。
+    struct ClockOfTheCardsOnTheCpu;
+
+    impl MonotonicClock for ClockOfTheCardsOnTheCpu {
+        fn time_since_the_origin(&self) -> Duration {
+            Duration::ZERO
+        }
+    }
+
+    /// 在 CPU 上算的替身卡：两份核都照 CPU 参照算（校验和那一段 `cpu_reference_outcome`；区域扫描与选段打分那两段只交形状对的结果，用例只看交没交成、
+    /// 交了几次），账与产品那张卡同一份（`GpuBudgetUnderTheSettings`）。按脚本造三样：交活之前、重交之前等前一次超时的那次提交等满上限没等到
+    /// （`earlier_submission_waits_timing_out`，按次序，`true` 是没等到；卡上记着一次还在跑的提交时才等、才取脚本），每一次提交等超过上限
+    /// （`submissions_timing_out`，按次序，`true` 是超时，超时的那一段的账照产品记成还在跑的那次提交），卡上空闲的显存（`free_bytes`：一段合占的比它多就建不成缓冲）。
+    /// 脚本用完之后都等到、都做完。
+    struct CardComputedOnTheCpuForTests {
+        discrete_gpu: DiscreteGpuOnThisHost,
+        budget: GpuBudgetUnderTheSettings<u32>,
+        measured_cost: Option<HostCpuNanosecondsPerSubmission>,
+        earlier_submission_waits_timing_out: VecDeque<bool>,
+        submissions_timing_out: VecDeque<bool>,
+        free_bytes: Option<u64>,
+        submissions_made: u32,
+    }
+
+    impl CardComputedOnTheCpuForTests {
+        /// 总线地址 `pci_bus_address` 的一张 8 GiB 替身卡，量过代价，给它的显存上限 `memory_limit_in_bytes`，这个进程里此前泄漏在它上面 `bytes_leaked` 字节。
+        fn of_the_address(
+            pci_bus_address: &str,
+            memory_limit_in_bytes: Option<u64>,
+            bytes_leaked: u64,
+        ) -> Self {
+            Self {
+                discrete_gpu: DiscreteGpuOnThisHost {
+                    position_in_the_enumeration: 0,
+                    identifier: card_identifier(pci_bus_address),
+                    pci_bus_address: pci_bus_address.to_string(),
+                    name: format!("cpu card at {pci_bus_address}"),
+                    total_memory: GpuMemoryInBytes(EIGHT_GIBIBYTES),
+                },
+                budget: GpuBudgetUnderTheSettings::with_the_bytes_leaked_by_earlier_mounts(
+                    1 << 30,
+                    1 << 20,
+                    memory_limit_in_bytes.map(GpuMemoryInBytes),
+                    WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU,
+                    bytes_leaked,
+                ),
+                measured_cost: Some(HostCpuNanosecondsPerSubmission(1_000)),
+                earlier_submission_waits_timing_out: VecDeque::new(),
+                submissions_timing_out: VecDeque::new(),
+                free_bytes: None,
+                submissions_made: 0,
+            }
+        }
+
+        /// 卡上记着一次等超过上限、还在跑的提交（占 `bytes` 字节），交活之前要先等它。
+        fn with_a_submission_running_past_its_wait(mut self, bytes: u64) -> Self {
+            self.budget.memory_account_mut().hold(bytes);
+            self.budget
+                .memory_account_mut()
+                .settle_the_chunk_after_its_submission(
+                    bytes,
+                    SubmissionOfTheChunk::MayStillBeRunningOnTheGpu(0),
+                );
+            self
+        }
+
+        /// 交一段（合占 `bytes_held_by_the_chunk` 字节）：照产品记账；空闲的不够就放掉、交回显存不足；照脚本超时就把这一段记成还在跑的那次提交、交回超时；
+        /// 做完了放账、交回 `outcome`。
+        fn chunk_submitted_on_the_cpu<Outcome>(
+            &mut self,
+            bytes_held_by_the_chunk: u64,
+            outcome: impl FnOnce() -> Outcome,
+        ) -> Result<Outcome, GpuFailureCause> {
+            self.budget
+                .memory_account_mut()
+                .hold(bytes_held_by_the_chunk);
+            if self
+                .free_bytes
+                .is_some_and(|free_bytes| bytes_held_by_the_chunk > free_bytes)
+            {
+                self.budget
+                    .memory_account_mut()
+                    .release(bytes_held_by_the_chunk);
+                return Err(GpuFailureCause::OutOfMemoryCreatingTheBuffers {
+                    bytes_held_by_the_chunk,
+                    reason: "替身卡上空闲的不够".to_string(),
+                });
+            }
+            self.submissions_made += 1;
+            if self.submissions_timing_out.pop_front().unwrap_or(false) {
+                self.budget
+                    .memory_account_mut()
+                    .settle_the_chunk_after_its_submission(
+                        bytes_held_by_the_chunk,
+                        SubmissionOfTheChunk::MayStillBeRunningOnTheGpu(self.submissions_made),
+                    );
+                return Err(GpuFailureCause::WaitForTheGpuExceededTheLimit {
+                    wait_limit: WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU,
+                });
+            }
+            self.budget
+                .memory_account_mut()
+                .settle_the_chunk_after_its_submission(
+                    bytes_held_by_the_chunk,
+                    SubmissionOfTheChunk::CompletedOnTheGpu,
+                );
+            Ok(outcome())
+        }
+
+        /// 卡上记着一次还在跑的提交、脚本这一回说没等到：交回等满上限。
+        fn earlier_submission_still_running_this_time(&mut self) -> bool {
+            self.budget
+                .memory_account()
+                .has_a_submission_running_past_its_wait()
+                && self
+                    .earlier_submission_waits_timing_out
+                    .pop_front()
+                    .unwrap_or(false)
+        }
+    }
+
+    impl CardOfTheBackgroundGpu for CardComputedOnTheCpuForTests {
+        type Submission = u32;
+
+        fn discrete_gpu(&self) -> &DiscreteGpuOnThisHost {
+            &self.discrete_gpu
+        }
+
+        fn budget(&self) -> &GpuBudgetUnderTheSettings<u32> {
+            &self.budget
+        }
+
+        fn budget_mut(&mut self) -> &mut GpuBudgetUnderTheSettings<u32> {
+            &mut self.budget
+        }
+
+        fn measured_cost(&self) -> Option<HostCpuNanosecondsPerSubmission> {
+            self.measured_cost
+        }
+
+        fn measure_the_cost_of_one_submission(&mut self) -> Result<(), GpuFailureCause> {
+            self.measured_cost = Some(HostCpuNanosecondsPerSubmission(1_000));
+            Ok(())
+        }
+
+        fn room_once_the_submission_running_past_its_wait_completed(
+            &mut self,
+        ) -> Result<ChunkBoundsOfAGpu, GpuFailureCause> {
+            if self.earlier_submission_still_running_this_time() {
+                return Err(
+                    GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit {
+                        wait_limit: WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU,
+                    },
+                );
+            }
+            self.budget
+                .room_once_the_submission_running_past_its_wait_completed(
+                    &ClockOfTheCardsOnTheCpu,
+                    |_submission, _time_left| CompletionCheck::Completed(()),
+                )
+        }
+
+        fn wait_for_the_timed_out_submission(&mut self) -> Result<(), GpuFailureCause> {
+            if self.earlier_submission_still_running_this_time() {
+                return Err(
+                    GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit {
+                        wait_limit: WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU,
+                    },
+                );
+            }
+            self.budget.wait_for_the_submission_running_past_its_wait(
+                &ClockOfTheCardsOnTheCpu,
+                |_submission, _time_left| CompletionCheck::Completed(()),
+            )
+        }
+
+        fn both_kernel_outcomes_of_a_checksum_chunk(
+            &mut self,
+            units: &[UnitChecksumToVerify<'_, ()>],
+        ) -> Result<Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)>, GpuFailureCause> {
+            let input_bytes: usize = units.iter().map(|unit| unit.unit_bytes.len()).sum();
+            let bytes_held_by_the_chunk = CHECKSUM_CHUNK_FOOTPRINT.bytes_held_by_a_chunk(
+                u64::try_from(input_bytes).expect("装得进 u64"),
+                u64::try_from(units.len()).expect("装得进 u64"),
+            );
+            self.chunk_submitted_on_the_cpu(bytes_held_by_the_chunk, || {
+                units
+                    .iter()
+                    .map(|unit| {
+                        let outcome = cpu_reference_outcome(unit.unit_bytes, unit.expected_crc32c);
+                        (outcome, outcome)
+                    })
+                    .collect()
+            })
+        }
+
+        fn both_kernel_outcomes_of_a_region_record_chunk(
+            &mut self,
+            pieces: &[RegionRecordPieceForTheKernels<'_>],
+            mapping_tables: Option<&MappingTablesForTheKernels>,
+        ) -> Result<BothKernelOutcomesOfARegionRecordChunk, GpuFailureCause> {
+            let empty_mapping_tables = MappingTablesForTheKernels::of_the_mapping_entries(&[])
+                .expect("空映射没有长过核装得下的 key");
+            let input_bytes: usize = pieces
+                .iter()
+                .map(|piece| piece.bytes().map_or(0, <[u8]>::len))
+                .sum();
+            let bytes_held_by_the_chunk =
+                region_record_chunk_footprint(mapping_tables.unwrap_or(&empty_mapping_tables))
+                    .bytes_held_by_a_chunk(
+                        u64::try_from(input_bytes).expect("装得进 u64"),
+                        u64::try_from(pieces.len()).expect("装得进 u64"),
+                    );
+            self.chunk_submitted_on_the_cpu(bytes_held_by_the_chunk, || match mapping_tables {
+                None => BothKernelOutcomesOfARegionRecordChunk::RegistersAfterThePieces(vec![
+                    CrcRegistersOfTheTwoKernels::BEFORE_ANY_BYTE;
+                    pieces.len()
+                ]),
+                Some(_tables) => BothKernelOutcomesOfARegionRecordChunk::Classifications(vec![
+                    (AllocationInRegion::Pinned, AllocationInRegion::Pinned);
+                    pieces.len()
+                ]),
+            })
+        }
+
+        fn both_kernel_live_slots_of_a_chunk(
+            &mut self,
+            segments: &[ClusterSegmentEntryKey],
+            record_tables: &AllocationRecordTablesForTheKernels,
+            _unit_area: UnitAreaOfOneDevice,
+        ) -> Result<Vec<(u64, u64)>, GpuFailureCause> {
+            let segment_count = u64::try_from(segments.len()).expect("装得进 u64");
+            let bytes_held_by_the_chunk = live_slot_chunk_footprint(record_tables)
+                .bytes_held_by_a_chunk(segment_count * SEGMENT_BOUND_BYTES, segment_count);
+            self.chunk_submitted_on_the_cpu(bytes_held_by_the_chunk, || {
+                vec![(0, 0); segments.len()]
+            })
+        }
+    }
+
+    fn card_identifier(pci_bus_address: &str) -> GpuDeviceIdentifier {
+        GpuDeviceIdentifier::of_opaque_bytes(pci_bus_address.as_bytes().to_vec())
+    }
+
+    /// 替身的「此刻本机在的独显」：用例改它模拟挂载之后掉了一张卡。
+    struct DiscreteGpusPresentForTests(Arc<Mutex<Vec<GpuDeviceIdentifier>>>);
+
+    impl DiscreteGpusPresentNow for DiscreteGpusPresentForTests {
+        fn identifiers_of_the_discrete_gpus_present_now(&self) -> Vec<GpuDeviceIdentifier> {
+            self.0.lock().expect("用例没在别处 panic 过").clone()
+        }
+    }
+
+    /// 外壳装上这几张替身卡（按交进来的次序就是本机的次序，全都参与），进程里泄漏的那一份是 `memory_leaked`；交回外壳与「此刻本机在的独显」那一份（起初全在）。
+    fn background_gpu_over_cards_on_the_cpu(
+        cards: Vec<CardComputedOnTheCpuForTests>,
+        memory_leaked: &GpuMemoryLeakedInThisProcess,
+    ) -> (
+        BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
+        Arc<Mutex<Vec<GpuDeviceIdentifier>>>,
+    ) {
+        let discrete_gpus: Vec<DiscreteGpuOnThisHost> =
+            cards.iter().map(|card| card.discrete_gpu.clone()).collect();
+        let present = Arc::new(Mutex::new(
+            discrete_gpus
+                .iter()
+                .map(|discrete_gpu| discrete_gpu.identifier.clone())
+                .collect(),
+        ));
+        let participating =
+            ParticipatingGpusOnThisHost::of_the_positions((0..cards.len()).collect());
+        (
+            BackgroundGpuOverTheCards::of_the_cards_opened_at_the_mount(
+                cards,
+                DiscreteGpusAfterTheOpening {
+                    opened: discrete_gpus,
+                    not_opened: Vec::new(),
+                },
+                Vec::new(),
+                participating,
+                Vec::new(),
+                memory_leaked.clone(),
+                Box::new(DiscreteGpusPresentForTests(Arc::clone(&present))),
+            ),
+            present,
+        )
+    }
+
+    /// `count` 个单元，第 i 个是 `bytes_per_unit` 个字节的 i，期望值照 CPU 参照算对。
+    fn unit_bytes_of(count: usize, bytes_per_unit: usize) -> Vec<Vec<u8>> {
+        (0..count)
+            .map(|unit| vec![u8::try_from(unit % 251).expect("小于 256"); bytes_per_unit])
+            .collect()
+    }
+
+    fn units_to_verify(unit_bytes: &[Vec<u8>]) -> Vec<UnitChecksumToVerify<'_, usize>> {
+        unit_bytes
+            .iter()
+            .enumerate()
+            .map(|(entry_key, bytes)| UnitChecksumToVerify {
+                entry_key,
+                unit_bytes: bytes,
+                expected_crc32c: singlefs_core::checksum::crc32_castagnoli(bytes),
+            })
+            .collect()
+    }
+
+    fn cpu_reference_outcomes_of(
+        units: &[UnitChecksumToVerify<'_, usize>],
+    ) -> Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)> {
+        units
+            .iter()
+            .map(|unit| {
+                let outcome = cpu_reference_outcome(unit.unit_bytes, unit.expected_crc32c);
+                (outcome, outcome)
+            })
+            .collect()
+    }
+
+    /// 判决 m4-gpugc-code-r1 第三节第 1 条：校验和那一段一批切成 4 段（上限 1100 字节只装得下一个 1000 字节的单元），每段头一次交都等超过上限、
+    /// 重交一次就做完——这一批在这张卡上第 4 次超时报这张卡用不了。这一段记下交成 3 次、3 片、重交 3 次、分到活的是这张卡。
+    #[test]
+    fn a_checksum_batch_cut_into_four_chunks_each_timing_out_once_reports_the_card_unusable() {
+        let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", Some(1100), 0);
+        card.submissions_timing_out = VecDeque::from([true, false, true, false, true, false, true]);
+        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
+            vec![card],
+            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+        );
+        let unit_bytes = unit_bytes_of(4, 1000);
+        assert_eq!(
+            background_gpu.both_kernel_outcomes_of_the_batch(&units_to_verify(&unit_bytes)),
+            Err(ChecksumBatchFailed::GpuFailed {
+                device: card_identifier("0000:01:00.0"),
+                cause: GpuFailureCause::UnusableAfterResubmissions {
+                    resubmissions: 3,
+                    last_cause: Box::new(GpuFailureCause::WaitForTheGpuExceededTheLimit {
+                        wait_limit: WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU,
+                    }),
+                },
+            }),
+            "四段各超时一次：这一批第 4 次超时报警"
+        );
+        assert_eq!(
+            background_gpu
+                .take_the_submissions_of_the_batch()
+                .unit_checksum_verification,
+            GpuSubmissionsOfOneSegment {
+                submissions_completed: 3,
+                pieces_submitted: 3,
+                resubmissions_after_timeouts: 3,
+                recuts_after_running_out_of_memory: 0,
+                devices_given_work: vec![card_identifier("0000:01:00.0")],
+            }
+        );
+    }
+
+    /// 同一条：一批 1 片超时 3 次、第 4 次交做完——照常交回，结果与 CPU 参照逐条相同；这一段记下交成 1 次、1 片、重交 3 次。
+    #[test]
+    fn a_checksum_batch_of_one_piece_timing_out_three_times_then_completing_is_returned() {
+        let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
+        card.submissions_timing_out = VecDeque::from([true, true, true]);
+        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
+            vec![card],
+            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+        );
+        let unit_bytes = unit_bytes_of(1, 1000);
+        let units = units_to_verify(&unit_bytes);
+        assert_eq!(
+            background_gpu.both_kernel_outcomes_of_the_batch(&units),
+            Ok(cpu_reference_outcomes_of(&units))
+        );
+        assert_eq!(
+            background_gpu
+                .take_the_submissions_of_the_batch()
+                .unit_checksum_verification,
+            GpuSubmissionsOfOneSegment {
+                submissions_completed: 1,
+                pieces_submitted: 1,
+                resubmissions_after_timeouts: 3,
+                recuts_after_running_out_of_memory: 0,
+                devices_given_work: vec![card_identifier("0000:01:00.0")],
+            }
+        );
+    }
+
+    /// 两条区域扫描会去读的记录（R 是第一块盘上槽 57600 起那一段）与空映射。
+    fn two_region_records_with_an_empty_mapping() -> (CompactionRegion, Vec<AllocationRecord>) {
+        let region =
+            CompactionRegion::of_one_cluster_segment(DeviceIdentity(0), SlotNumber(64 * 900))
+                .expect("64 的倍数");
+        let records = (0..2_u64)
+            .map(|offset| AllocationRecord {
+                device: DeviceIdentity(0),
+                slot: SlotNumber(region.start_slot().0 + offset),
+                span_slots: 1,
+                generation: CheckpointTxg(7),
+                is_released: false,
+                quarantine_mark: AllocationRecordQuarantineMark::NotQuarantined,
+            })
+            .collect();
+        (region, records)
+    }
+
+    /// 同一条在区域扫描那一段的接线：卡上记着前一次超时、还在跑的提交，交活之前等了两回没等到、第三回等到，再交一段超时一次、重交做完——
+    /// 这一批在这张卡上超时 3 次，照常交回；这一段记下交成 1 次、2 片、重交 1 次。交活之前那一等不算进超时、或这一段不套重交，都交回出错。
+    #[test]
+    fn a_region_record_batch_waiting_twice_before_handing_over_and_timing_out_once_is_returned() {
+        let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0)
+            .with_a_submission_running_past_its_wait(100);
+        card.earlier_submission_waits_timing_out = VecDeque::from([true, true]);
+        card.submissions_timing_out = VecDeque::from([true]);
+        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
+            vec![card],
+            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+        );
+        let (region, records) = two_region_records_with_an_empty_mapping();
+        let unit_bytes = [7_u8; 64];
+        let to_classify: Vec<RegionRecordToClassify<'_, usize>> = records
+            .iter()
+            .enumerate()
+            .map(|(entry_key, record)| RegionRecordToClassify {
+                entry_key,
+                record: *record,
+                unit_bytes: Some(&unit_bytes),
+            })
+            .collect();
+        assert_eq!(
+            background_gpu
+                .both_kernel_classifications_of_the_region_records(region, &to_classify, &[])
+                .map(|classifications| classifications.len()),
+            Ok(2),
+            "交活之前两回加这一段一回，第 3 次超时之内：照常交回"
+        );
+        assert_eq!(
+            background_gpu
+                .take_the_submissions_of_the_batch()
+                .region_record_classification,
+            GpuSubmissionsOfOneSegment {
+                submissions_completed: 1,
+                pieces_submitted: 2,
+                resubmissions_after_timeouts: 1,
+                recuts_after_running_out_of_memory: 0,
+                devices_given_work: vec![card_identifier("0000:01:00.0")],
+            }
+        );
+    }
+
+    /// 同一条在选段打分那一段的接线：同上一条的时序，两个段。
+    #[test]
+    fn a_live_slot_batch_waiting_twice_before_handing_over_and_timing_out_once_is_returned() {
+        let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0)
+            .with_a_submission_running_past_its_wait(100);
+        card.earlier_submission_waits_timing_out = VecDeque::from([true, true]);
+        card.submissions_timing_out = VecDeque::from([true]);
+        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
+            vec![card],
+            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+        );
+        let unit_area = UnitAreaOfOneDevice {
+            device: DeviceIdentity(0),
+            start: UnitAreaStart::of_the_default_journal_ring(),
+            slots: 128,
+        };
+        let segments: Vec<ClusterSegmentEntryKey> = (0..2_u64)
+            .map(|segment| ClusterSegmentEntryKey {
+                device: DeviceIdentity(0),
+                start_slot: SlotNumber(unit_area.start.slot().0 + 64 * segment),
+            })
+            .collect();
+        assert_eq!(
+            background_gpu.both_kernel_live_slots_of_the_cluster_segments(
+                &[],
+                unit_area,
+                &segments
+            ),
+            Ok(vec![(0, 0), (0, 0)]),
+            "交活之前两回加这一段一回，第 3 次超时之内：照常交回"
+        );
+        assert_eq!(
+            background_gpu
+                .take_the_submissions_of_the_batch()
+                .live_slots_per_cluster_segment,
+            GpuSubmissionsOfOneSegment {
+                submissions_completed: 1,
+                pieces_submitted: 2,
+                resubmissions_after_timeouts: 1,
+                recuts_after_running_out_of_memory: 0,
+                devices_given_work: vec![card_identifier("0000:01:00.0")],
+            }
+        );
+    }
+
+    /// 判决第三节第 4 条在外壳上：两张卡，「0000:01:00.0」上限 1000 字节而这个进程里此前泄漏了 950 字节，余量 50 字节装不下校验和那一段最小的一片（64 字节）——
+    /// 这一批不分给它，两条全交另一张卡，结果与 CPU 参照逐条相同；这一段记下分到活的只有「0000:41:00.0」。
+    #[test]
+    fn a_checksum_batch_goes_wholly_to_the_other_card_when_one_card_is_short_of_the_smallest_piece()
+    {
+        let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
+        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
+            vec![
+                CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", Some(1000), 950),
+                CardComputedOnTheCpuForTests::of_the_address("0000:41:00.0", None, 0),
+            ],
+            &memory_leaked,
+        );
+        let unit_bytes = unit_bytes_of(2, 100);
+        let units = units_to_verify(&unit_bytes);
+        assert_eq!(
+            background_gpu.both_kernel_outcomes_of_the_batch(&units),
+            Ok(cpu_reference_outcomes_of(&units))
+        );
+        assert_eq!(
+            background_gpu
+                .take_the_submissions_of_the_batch()
+                .unit_checksum_verification,
+            GpuSubmissionsOfOneSegment {
+                submissions_completed: 1,
+                pieces_submitted: 2,
+                resubmissions_after_timeouts: 0,
+                recuts_after_running_out_of_memory: 0,
+                devices_given_work: vec![card_identifier("0000:41:00.0")],
+            }
+        );
+    }
+
+    /// 判决第三节第 5 条（第 3 条「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」）：上限 1 MiB、卡上实际只空闲 256 KiB，八个 64 KiB 的单元
+    /// 照上限一段装得下（512 KiB 多）而建缓冲显存不足——压到那一段的一半重切成三、三、二个一段，照常交回，判定与 CPU 参照逐条相同、全都确认；
+    /// 这一段记下重切 1 次、交成 3 次、8 片。
+    #[test]
+    fn a_checksum_batch_on_a_card_with_less_free_memory_than_its_limit_is_recut_and_agrees_with_the_cpu(
+    ) {
+        let mut card =
+            CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", Some(1 << 20), 0);
+        card.free_bytes = Some(256 << 10);
+        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
+            vec![card],
+            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+        );
+        let unit_bytes = unit_bytes_of(8, 64 << 10);
+        let units = units_to_verify(&unit_bytes);
+        let reconciled = background_gpu
+            .verify_unit_checksums(
+                &units,
+                &mut ConsecutiveUncertaintyCounts::with_no_entry_uncertain(),
+            )
+            .expect("显存不足按更小的片重切重交：不报出错");
+        assert_eq!(
+            reconciled.decisions,
+            cpu_reference_outcomes_of(&units)
+                .into_iter()
+                .map(|(outcome, _same)| EntryDecision::ConfirmedByBothKernels(outcome))
+                .collect::<Vec<_>>(),
+            "判定与 CPU 参照逐条相同"
+        );
+        assert_eq!(
+            background_gpu
+                .take_the_submissions_of_the_batch()
+                .unit_checksum_verification,
+            GpuSubmissionsOfOneSegment {
+                submissions_completed: 3,
+                pieces_submitted: 8,
+                resubmissions_after_timeouts: 0,
+                recuts_after_running_out_of_memory: 1,
+                devices_given_work: vec![card_identifier("0000:01:00.0")],
+            }
+        );
+    }
+
+    /// 同一条：卡上只空闲 10 字节，一路压小到连最小的一片（64 字节）都建不成——交回空闲的连最小的一片都不到（条款没写，第一版不支持），重切了 5 次。
+    #[test]
+    fn a_card_whose_free_memory_is_short_of_the_smallest_piece_reports_it() {
+        let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
+        card.free_bytes = Some(10);
+        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
+            vec![card],
+            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+        );
+        let unit_bytes = unit_bytes_of(1, 1000);
+        assert_eq!(
+            background_gpu.both_kernel_outcomes_of_the_batch(&units_to_verify(&unit_bytes)),
+            Err(ChecksumBatchFailed::GpuFailed {
+                device: card_identifier("0000:01:00.0"),
+                cause: GpuFailureCause::FreeMemoryShortOfTheSmallestPiece {
+                    smallest_piece_bytes: 64
+                },
+            })
+        );
+        assert_eq!(
+            background_gpu
+                .take_the_submissions_of_the_batch()
+                .unit_checksum_verification
+                .recuts_after_running_out_of_memory,
+            5,
+            "1060、528、264、132、64 字节的一段各建不成一次"
+        );
+    }
+
+    /// 判决第三节第 8 条：丢外壳时，记着一次还在跑的提交（占 600 字节）的那张卡泄漏、另一张照常丢；泄漏了哪一张、占着多少交到调用方拿着的那一份，
+    /// 并记进进程里的那一份（新一次挂载这张卡先扣掉）。
+    #[test]
+    fn dropping_the_background_gpu_reports_the_card_leaked_and_notes_it_in_the_process() {
+        let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
+        let (background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
+            vec![
+                CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0)
+                    .with_a_submission_running_past_its_wait(600),
+                CardComputedOnTheCpuForTests::of_the_address("0000:41:00.0", None, 0),
+            ],
+            &memory_leaked,
+        );
+        let gpus_leaked = background_gpu.gpus_leaked_when_dropped();
+        assert_eq!(gpus_leaked.gpus_leaked(), None, "还没丢");
+        drop(background_gpu);
+        assert_eq!(
+            gpus_leaked.gpus_leaked(),
+            Some(vec![GpuLeakedWhenDropped {
+                device: card_identifier("0000:01:00.0"),
+                bytes_held_on_the_device: 600,
+            }]),
+            "只泄漏还在跑的那一张"
+        );
+        assert_eq!(
+            (
+                memory_leaked.bytes_leaked_on(&card_identifier("0000:01:00.0")),
+                memory_leaked.bytes_leaked_on(&card_identifier("0000:41:00.0"))
+            ),
+            (600, 0)
+        );
+    }
+
+    /// 判决第三节第 2 条：卡清单每次答之前现查本机在的独显——挂载之后那张卡掉了，不点名的切换照「GPU 不在」拒、点名它照「点名的卡不存在」拒。
+    #[test]
+    fn a_switch_after_the_card_dropped_off_the_host_is_refused_as_gpu_not_present() {
+        let (background_gpu, present) = background_gpu_over_cards_on_the_cpu(
+            vec![CardComputedOnTheCpuForTests::of_the_address(
+                "0000:01:00.0",
+                None,
+                0,
+            )],
+            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+        );
+        let settings_naming = |named: Option<Vec<GpuDeviceIdentifier>>| BackgroundComputeSettings {
+            background_compute: BackgroundComputeBackend::Gpu,
+            background_gpu_devices: named,
+            background_gpu_memory_limit: None,
+            background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
+        };
+        assert!(
+            admit_gpu_settings_against_the_inventory(&settings_naming(None), &background_gpu)
+                .is_ok(),
+            "卡还在：照收"
+        );
+        present.lock().expect("用例没在别处 panic 过").clear();
+        assert_eq!(
+            admit_gpu_settings_against_the_inventory(&settings_naming(None), &background_gpu),
+            Err(BackgroundComputeSettingsRefused::GpuNotPresent),
+            "卡掉了、不点名：GPU 不在"
+        );
+        assert_eq!(
+            admit_gpu_settings_against_the_inventory(
+                &settings_naming(Some(vec![card_identifier("0000:01:00.0")])),
+                &background_gpu
+            ),
+            Err(
+                BackgroundComputeSettingsRefused::NamedGpuDeviceNotInTheInventory {
+                    device: card_identifier("0000:01:00.0")
+                }
+            ),
+            "卡掉了、点名它：点名的卡不存在"
+        );
+    }
 }
diff --git a/crates/singlefs-background-gpu/src/background_gpu_compute.rs b/crates/singlefs-background-gpu/src/background_gpu_compute.rs
index 6b70095..9d44e4b 100644
--- a/crates/singlefs-background-gpu/src/background_gpu_compute.rs
+++ b/crates/singlefs-background-gpu/src/background_gpu_compute.rs
@@ -22,7 +22,7 @@ use singlefs_core::background_compute::{
     AdmittedGpuSettings, BackgroundGpuCompute, BackgroundGpuComputeFailed,
     BackgroundGpuComputeHandle, ClusterSegmentEntryKey, GpuDeviceIdentifier,
     GpuDeviceNotOpenedAtTheMount, GpuEntryDecision, GpuFailureOnTheDevice, GpuInventory,
-    GpuMemoryInBytes, InputNotSupportedOnTheGpu, RegionRecordEntryKey,
+    GpuMemoryInBytes, GpuSubmissionsOfTheBatch, InputNotSupportedOnTheGpu, RegionRecordEntryKey,
     RegionRecordToClassifyOnTheGpu, SubmissionWaitedForPastTheLimit, UnitChecksumOutcome,
     UnitChecksumToVerifyOnTheGpu,
 };
@@ -30,16 +30,20 @@ use singlefs_core::compaction_plan::UnitAreaOfOneDevice;
 use singlefs_core::logged_operations::CompactionRegion;
 use singlefs_core::region_scan::{AllocationInRegion, CentralMappingEntry};
 
-use crate::adapter_inventory::BackgroundGpuSelection;
+use crate::adapter_inventory::{BackgroundGpuSelection, DiscreteGpuListRefused};
 use crate::background_gpu::{
-    open_background_gpu, BackgroundGpu, BackgroundGpuOpenRefused, BatchNotSplitOverTheGpus,
-    ChecksumBatchFailed, GpuFailureCause, LiveSlotBatchFailed, RegionRecordBatchFailed,
-    SmallestPieceNotHeldOnTheGpu,
+    open_background_gpu_with_the_memory_leaked_in_this_process, BackgroundGpu,
+    BackgroundGpuOpenRefused, BackgroundGpuOverTheCards, BatchNotSplitOverTheGpus,
+    CardOfTheBackgroundGpu, ChecksumBatchFailed, GpuFailureCause, LiveSlotBatchFailed,
+    RegionRecordBatchFailed, SmallestPieceNotHeldOnTheGpu,
 };
 use crate::checksum_kernels::{
     reconciled_unit_checksums, ChecksumKernelOutcome, UnitChecksumToVerify,
 };
-use crate::gpus_of_this_host::{DiscreteGpuNotOpenedAtTheMount, GpuLeftOutUnderItsMemoryLimit};
+use crate::gpus_of_this_host::{
+    DiscreteGpuNotOpenedAtTheMount, GpuLeftOutUnderItsMemoryLimit, GpuMemoryLeakedInThisProcess,
+    GpusLeakedWhenTheBackgroundGpuIsDropped,
+};
 use crate::live_slot_kernels::reconciled_live_slots_of_the_segments;
 use crate::region_record_kernels::{
     reconciled_region_record_classifications, RegionRecordToClassify,
@@ -89,9 +93,18 @@ pub trait BothKernelsOfTheThreeSegments: GpuInventory {
     fn batch_not_split_over_the_gpus_under_the_settings_in_use(
         &self,
     ) -> Option<BatchNotSplitOverTheGpus>;
+
+    /// 上一次交回以来三段各自在卡上是怎么交的（交了几片、重交了几次、显存不足重切了几次、用了哪几张卡），交回之后清零
+    /// （`singlefs_core::background_compute::BackgroundGpuCompute::take_the_submissions_of_the_batch` 转来的，一批三段算完之后）。
+    /// 不开卡的替身没有这一样，交回什么都没交的那一份。
+    fn take_the_submissions_of_the_batch(&mut self) -> GpuSubmissionsOfTheBatch {
+        GpuSubmissionsOfTheBatch::default()
+    }
 }
 
-impl BothKernelsOfTheThreeSegments for BackgroundGpu {
+impl<Card: CardOfTheBackgroundGpu> BothKernelsOfTheThreeSegments
+    for BackgroundGpuOverTheCards<Card>
+{
     fn unit_checksums_by_both_kernels(
         &mut self,
         units: &[UnitChecksumToVerify<'_, RegionRecordEntryKey>],
@@ -118,7 +131,7 @@ impl BothKernelsOfTheThreeSegments for BackgroundGpu {
     }
 
     fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings) {
-        BackgroundGpu::use_the_settings_from_the_next_batch(self, settings);
+        BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch(self, settings);
     }
 
     fn batch_not_split_over_the_gpus_under_the_settings_in_use(
@@ -126,6 +139,10 @@ impl BothKernelsOfTheThreeSegments for BackgroundGpu {
     ) -> Option<BatchNotSplitOverTheGpus> {
         self.batch_not_split_over_the_gpus()
     }
+
+    fn take_the_submissions_of_the_batch(&mut self) -> GpuSubmissionsOfTheBatch {
+        BackgroundGpuOverTheCards::take_the_submissions_of_the_batch(self)
+    }
 }
 
 /// 核心层要的那一份实现：两份核那一层，加三段各一份连续不确定的计数（第 8 条，N = 2，只在内存里）。
@@ -157,36 +174,51 @@ impl<Kernels> BackgroundGpuComputeThroughBothKernels<Kernels> {
 /// 没跳过一张是空），与开卡那一组点了而因显存上限连最小的一片都装不下、不用的卡（按参与的次序，各带上限与至少要多少；用户 2026-10-28 定
 /// 「连一片都装不下就不用这张卡」）。参与的卡因此一张都不剩时挂载不拒：核心层挂载那一刻把校验过的那一组交给实现、读
 /// [`BackgroundGpuCompute::failure_under_the_settings_handed_in`]，读到 `EveryParticipatingGpuLeftOutUnderItsMemoryLimit` 就当场报警停着。
+/// 给了 `background_gpu_devices` 时没点名而交不成一张卡、没收进卡清单的独显各带原因（`unnamed_discrete_gpus_left_off_the_list`）；
+/// 卸载（丢掉核心层收的把手）之后泄漏了哪几张卡，读 `gpus_leaked_when_dropped`。
 #[derive(Debug)]
 pub struct OpenedBackgroundGpuCompute {
     pub gpu_compute: BackgroundGpuComputeHandle,
     pub gpus_not_opened_at_the_mount: Vec<DiscreteGpuNotOpenedAtTheMount>,
     pub gpus_left_out_under_their_memory_limit: Vec<GpuLeftOutUnderItsMemoryLimit>,
+    pub unnamed_discrete_gpus_left_off_the_list: Vec<DiscreteGpuListRefused>,
+    pub gpus_leaked_when_dropped: GpusLeakedWhenTheBackgroundGpuIsDropped,
 }
 
 /// 挂载那一刻开后台 GPU、交回核心层挂载与切换收的把手（D26（后台整理与放置回收） 已定项 13「挂载时由调用方把它的实现递进去」）：
 /// 开本机全部独显、逐张量代价，`selection` 定开卡时参与的卡（`BackgroundGpuSelection::of_the_settings` 照挂载那一组取；本包开卡时照它校验一次，
 /// 同核心层 `admit_background_compute_settings`）。给了列表而没点名的卡没开成就跳过、在交回的结局里报出来，参与的卡没开成就拒。
-/// 核心层挂载时照这份实现答的卡清单（本机全部独显，连同没开成的那几张）再校验一次，并把校验过的那一组交过来
+/// 核心层挂载时照这份实现答的卡清单（此刻本机在的独显，连同没开成的那几张）再校验一次，并把校验过的那一组交过来
 /// （`use_the_settings_from_the_next_batch`），之后每次切换生效时另交，参与的卡、显存上限、等的上限都照核心层交来的那一组。
+/// `memory_leaked_in_this_process` 是调用方在进程里留着的那一份：此前几次挂载泄漏在一张卡上的字节先从它的余量里扣掉，这一次卸载泄漏的记进去。
 ///
 /// # Errors
-/// [`BackgroundGpuOpenRefused`] 的四种：设置被拒（GPU 不在、点名的卡不存在、上限个数不对、上限超过显存总量……）、独显交不成清单、
+/// [`BackgroundGpuOpenRefused`] 的四种：设置被拒（GPU 不在、点名的卡不存在、上限个数不对、上限超过显存总量……）、参与的独显交不成清单、
 /// 枚举适配器超过等 GPU 的上限、一张参与的卡用不上。显存上限装不下最小的一片的卡不拒（见 [`OpenedBackgroundGpuCompute`]）。
 pub fn open_background_gpu_compute(
     selection: &BackgroundGpuSelection,
+    memory_leaked_in_this_process: &GpuMemoryLeakedInThisProcess,
 ) -> Result<OpenedBackgroundGpuCompute, BackgroundGpuOpenRefused> {
-    let background_gpu = open_background_gpu(selection)?;
+    let background_gpu = open_background_gpu_with_the_memory_leaked_in_this_process(
+        selection,
+        memory_leaked_in_this_process,
+    )?;
     let gpus_not_opened_at_the_mount = background_gpu.gpus_not_opened_at_the_mount().to_vec();
     let gpus_left_out_under_their_memory_limit = background_gpu
         .gpus_left_out_under_their_memory_limit()
         .to_vec();
+    let unnamed_discrete_gpus_left_off_the_list = background_gpu
+        .unnamed_discrete_gpus_left_off_the_list()
+        .to_vec();
+    let gpus_leaked_when_dropped = background_gpu.gpus_leaked_when_dropped();
     Ok(OpenedBackgroundGpuCompute {
         gpu_compute: BackgroundGpuComputeHandle::of_an_implementation(
             BackgroundGpuComputeOnTheParticipatingGpus::with_no_entry_uncertain(background_gpu),
         ),
         gpus_not_opened_at_the_mount,
         gpus_left_out_under_their_memory_limit,
+        unnamed_discrete_gpus_left_off_the_list,
+        gpus_leaked_when_dropped,
     })
 }
 
@@ -262,6 +294,20 @@ pub fn core_failure_of_the_gpu(
                 verdict_word: unrecognized.verdict_word,
             })
         }
+        GpuFailureCause::OutOfMemoryCreatingTheBuffers {
+            bytes_held_by_the_chunk,
+            reason,
+        } => failed_on_the_device(GpuFailureOnTheDevice::OutOfMemoryCreatingTheBuffers {
+            bytes_held_by_the_chunk: *bytes_held_by_the_chunk,
+            reason: reason.clone(),
+        }),
+        GpuFailureCause::FreeMemoryShortOfTheSmallestPiece {
+            smallest_piece_bytes,
+        } => failed_on_the_device(
+            GpuFailureOnTheDevice::FreeMemoryShortOfTheSmallestPieceWhoseHandlingIsUndecided {
+                smallest_piece_bytes: *smallest_piece_bytes,
+            },
+        ),
         GpuFailureCause::UnusableAfterResubmissions {
             resubmissions,
             last_cause,
@@ -293,6 +339,8 @@ pub fn core_failure_of_the_gpu(
             | GpuFailureCause::WaitForTheSubmissionFailed { .. }
             | GpuFailureCause::ReadbackMappingFailed { .. }
             | GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord(..)
+            | GpuFailureCause::OutOfMemoryCreatingTheBuffers { .. }
+            | GpuFailureCause::FreeMemoryShortOfTheSmallestPiece { .. }
             | GpuFailureCause::UnusableAfterResubmissions { .. }) => {
                 core_failure_of_the_gpu(device, last_cause_other_than_a_wait_past_the_limit)
             }
@@ -515,6 +563,11 @@ impl<Kernels: BothKernelsOfTheThreeSegments> BackgroundGpuCompute
             .batch_not_split_over_the_gpus_under_the_settings_in_use()
             .map(|not_split| core_failure_of_the_batch_not_split_over_the_gpus(&not_split))
     }
+
+    /// 转给两份核那一层（[`BothKernelsOfTheThreeSegments::take_the_submissions_of_the_batch`]）。
+    fn take_the_submissions_of_the_batch(&mut self) -> GpuSubmissionsOfTheBatch {
+        self.kernels.take_the_submissions_of_the_batch()
+    }
 }
 
 #[cfg(test)]
@@ -601,6 +654,26 @@ mod tests {
                     verdict_word: 7,
                 }),
             ),
+            (
+                GpuFailureCause::OutOfMemoryCreatingTheBuffers {
+                    bytes_held_by_the_chunk: 4096,
+                    reason: "假的：显存不足".to_string(),
+                },
+                failed_on_the_card(GpuFailureOnTheDevice::OutOfMemoryCreatingTheBuffers {
+                    bytes_held_by_the_chunk: 4096,
+                    reason: "假的：显存不足".to_string(),
+                }),
+            ),
+            (
+                GpuFailureCause::FreeMemoryShortOfTheSmallestPiece {
+                    smallest_piece_bytes: 64,
+                },
+                failed_on_the_card(
+                    GpuFailureOnTheDevice::FreeMemoryShortOfTheSmallestPieceWhoseHandlingIsUndecided {
+                        smallest_piece_bytes: 64,
+                    },
+                ),
+            ),
         ]
     }
 
@@ -760,11 +833,13 @@ mod tests {
                 device: card(),
                 memory_limit: GpuMemoryInBytes(50),
                 smallest_piece_bytes: 976,
+                bytes_already_held_on_the_device: 0,
             },
             GpuLeftOutUnderItsMemoryLimit {
                 device: GpuDeviceIdentifier::of_opaque_bytes(b"0000:02:00.0".to_vec()),
                 memory_limit: GpuMemoryInBytes(900),
                 smallest_piece_bytes: 976,
+                bytes_already_held_on_the_device: 600,
             },
         ];
         let expected =
diff --git a/crates/singlefs-background-gpu/src/gpus_of_this_host.rs b/crates/singlefs-background-gpu/src/gpus_of_this_host.rs
index 94a318f..f79e4f8 100644
--- a/crates/singlefs-background-gpu/src/gpus_of_this_host.rs
+++ b/crates/singlefs-background-gpu/src/gpus_of_this_host.rs
@@ -19,6 +19,7 @@
 //! 丢后台 GPU 时不等还在 GPU 上跑的提交（同一定案「卸载时同样不等、泄漏设备」）：[`gpus_dropped_leaking_those_still_running`]。
 
 use std::ops::Range;
+use std::sync::{Arc, Mutex};
 use std::time::Duration;
 
 use singlefs_core::background_compute::{
@@ -27,7 +28,9 @@ use singlefs_core::background_compute::{
 };
 
 use crate::adapter_inventory::{DiscreteGpuOnThisHost, ParticipatingGpu};
-use crate::background_gpu::{BackgroundGpuOpenRefused, GpuFailureCause};
+use crate::background_gpu::{
+    BackgroundGpuOpenRefused, GpuFailureCause, SmallestPieceNotHeldOnTheGpu,
+};
 use crate::background_gpu_compute::core_failure_of_the_gpu;
 use crate::checksum_kernels::bytes_of_the_smallest_checksum_piece;
 use crate::live_slot_kernels::bytes_of_the_smallest_live_slot_piece;
@@ -58,15 +61,44 @@ impl<Submission> GpuBudgetUnderTheSettings<Submission> {
         largest_entry_count_per_dispatch: u64,
         memory_limit: Option<GpuMemoryInBytes>,
         wait_limit: Duration,
+    ) -> Self {
+        Self::with_the_bytes_leaked_by_earlier_mounts(
+            largest_storage_binding_in_bytes,
+            largest_entry_count_per_dispatch,
+            memory_limit,
+            wait_limit,
+            0,
+        )
+    }
+
+    /// 开卡那一刻，这个进程里此前几次挂载在这张卡上泄漏了 `bytes_leaked_by_earlier_mounts` 字节（[`GpuMemoryLeakedInThisProcess`]）：
+    /// 账从它起、一直记着，这张卡的可用余量先扣掉它（第 3 条：泄漏的那一片仍占着这张卡）。
+    #[must_use]
+    pub fn with_the_bytes_leaked_by_earlier_mounts(
+        largest_storage_binding_in_bytes: u64,
+        largest_entry_count_per_dispatch: u64,
+        memory_limit: Option<GpuMemoryInBytes>,
+        wait_limit: Duration,
+        bytes_leaked_by_earlier_mounts: u64,
     ) -> Self {
         Self {
             largest_storage_binding_in_bytes,
             largest_entry_count_per_dispatch,
-            memory_account: GpuMemoryAccount::with_nothing_held(memory_limit),
+            memory_account: GpuMemoryAccount::with_the_bytes_leaked_by_earlier_mounts(
+                memory_limit,
+                bytes_leaked_by_earlier_mounts,
+            ),
             wait_limit,
         }
     }
 
+    /// 此刻还能切一段的界：显存上限扣掉此刻还占着的（泄漏的、等超过上限还在跑的那一次提交的）。
+    #[must_use]
+    pub fn room_left_now(&self) -> ChunkBoundsOfAGpu {
+        self.chunk_bounds()
+            .with_the_room_left_after(self.memory_account.bytes_held())
+    }
+
     /// 换上一组新设置给这张卡的显存上限与等的上限（两批之间调）。账照旧记着。
     pub fn use_the_settings_from_the_next_batch(
         &mut self,
@@ -284,6 +316,30 @@ pub struct DiscreteGpuNotOpenedAtTheMount {
     pub cause: GpuFailureCause,
 }
 
+impl DiscreteGpusAfterTheOpening {
+    /// 只留 `identifiers_present_now`（挂着切换那一刻现查的那一次枚举里在的独显）里在的：挂载之后掉了的卡不再算本机的卡，切换点它照「点名的卡不存在」拒，
+    /// 不点名时它不再参与（第 4 条「切换那一刻……GPU 不在……就拒绝切换」）。
+    #[must_use]
+    pub fn still_present(&self, identifiers_present_now: &[GpuDeviceIdentifier]) -> Self {
+        Self {
+            opened: self
+                .opened
+                .iter()
+                .filter(|opened| identifiers_present_now.contains(&opened.identifier))
+                .cloned()
+                .collect(),
+            not_opened: self
+                .not_opened
+                .iter()
+                .filter(|not_opened| {
+                    identifiers_present_now.contains(&not_opened.discrete_gpu.identifier)
+                })
+                .cloned()
+                .collect(),
+        }
+    }
+}
+
 /// 挂载那一刻开卡之后的本机独显：开成的那几张与没开成而跳过的那几张，各按总线地址排。
 /// 卡清单（核心层 `GpuInventory`）照它答：本机全部独显连同没开成的（两样合起来按总线地址排，没给 `background_gpu_devices` 时上限按这个次序
 /// 一张一个，第 3 条），与其中没开成的那几张、原因照挂着时这张卡出错的报法（`core_failure_of_the_gpu`）。
@@ -391,7 +447,7 @@ pub fn measured_cost_summed_over_the_pieces<Chunk, Failure>(
 /// 参与的卡显存上限至少要装得下的那一片：三段各自与输入无关、定得下来的最小的一片取最大的那个——校验和那一段（量代价那一段同它）一个字节的单元，
 /// 选段打分那一段一个段连同一条记录的记录表，区域扫描那一段切开的单元头一片连同只放一条条目的映射（单元能切开之后它不再随最大那条记录变，
 /// 用户 2026-10-28 定「装不下应该自适应的分片，缩小逐次进行」，`crate::region_record_kernels::bytes_of_the_smallest_region_record_piece`）。
-/// 区域扫描一个落点上不止一条条目的那一片随这一批的映射变，挂着时逐批判。
+/// 区域扫描一个落点上不止一条条目的那一片随这一批的映射变，挂着时逐批判（[`plans_of_a_batch_leaving_out_the_gpus_short_of_the_smallest_piece`]）。
 #[must_use]
 pub fn bytes_of_the_smallest_piece_every_participating_gpu_holds() -> u64 {
     bytes_of_the_smallest_checksum_piece()
@@ -402,22 +458,244 @@ pub fn bytes_of_the_smallest_piece_every_participating_gpu_holds() -> u64 {
 /// 一张卡因显存上限装不下最小的一片而不用：核心层定的那一份（参与的卡一张都不剩时原样报给核心层），本包照它报。
 pub use singlefs_core::background_compute::GpuLeftOutUnderItsMemoryLimit;
 
-/// 给这张卡的显存上限装不下 `smallest_piece_bytes` 时交回它不用的那一条；不给上限、装得下都交回 `None`。
+/// 给这张卡的显存上限扣掉卡上已经占着的 `bytes_already_held_on_the_device`（这个进程里此前几次挂载泄漏的、等超过上限还在跑的那一次提交的）
+/// 装不下 `smallest_piece_bytes` 时交回它不用的那一条；不给上限、装得下都交回 `None`。
 #[must_use]
 pub fn gpu_left_out_under_its_memory_limit(
     device: &GpuDeviceIdentifier,
     memory_limit: Option<GpuMemoryInBytes>,
     smallest_piece_bytes: u64,
+    bytes_already_held_on_the_device: u64,
 ) -> Option<GpuLeftOutUnderItsMemoryLimit> {
     memory_limit
-        .filter(|memory_limit| memory_limit.0 < smallest_piece_bytes)
+        .filter(|memory_limit| {
+            memory_limit
+                .0
+                .saturating_sub(bytes_already_held_on_the_device)
+                < smallest_piece_bytes
+        })
         .map(|memory_limit| GpuLeftOutUnderItsMemoryLimit {
             device: device.clone(),
             memory_limit,
             smallest_piece_bytes,
+            bytes_already_held_on_the_device,
         })
 }
 
+/// 一张参与的卡这一批交活之前那一刻的地方（交活之前先等前一次超时的那次提交做完之后）：剩下的界（给的上限扣掉卡上已占的），
+/// 连同报出来要的那几样——哪张卡、给它的上限、已经占着的字节。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct RoomOfAGpuForTheBatch {
+    pub device: GpuDeviceIdentifier,
+    pub room: ChunkBoundsOfAGpu,
+    pub memory_limit: Option<GpuMemoryInBytes>,
+    pub bytes_already_held_on_the_device: u64,
+}
+
+/// 一张参与的卡这一批分到的活与切法：它在本机里的位置、分到的那一段相连的条目、切的时候照的界、切法。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct PlanOfAGpuForTheBatch<Plan> {
+    pub position_on_this_host: usize,
+    pub entries: Range<usize>,
+    pub room: ChunkBoundsOfAGpu,
+    pub plan: Plan,
+}
+
+/// 一批切不成、一张卡都还没交活（[`plans_of_a_batch_leaving_out_the_gpus_short_of_the_smallest_piece`]）。封闭集合。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum BatchNotPlannedOverTheGpus {
+    /// 一张卡交活之前先等前一次超时的那次提交没等成（含这一批在这张卡上第 4 次超时、报这张卡用不了）。
+    GpuFailed {
+        device: GpuDeviceIdentifier,
+        cause: GpuFailureCause,
+    },
+    /// 参与的卡这一批全都因显存上限（扣掉已占的）连这一批最小的一片都切不出：按不用的先后列出每一张。
+    EveryParticipatingGpuLeftOut(Vec<GpuLeftOutUnderItsMemoryLimit>),
+    /// 一张卡切不出，而装不下的不是显存上限（是单个缓冲的绑定上限）：第一版不支持。
+    GpuCannotHoldTheSmallestPiece(SmallestPieceNotHeldOnTheGpu),
+}
+
+/// 一批（`entry_count` 条）按量出的代价分给参与的卡、各卡照交活之前那一刻的地方（`room_of_the_position`，先等前一次超时的那次提交）切好（`cut`）；
+/// 一张卡这一批连最小的一片都切不出、而装不下的是显存上限（扣掉已占的），这一批就不分给它（第 3 条「参与的卡的上限连最小的一片都装不下，这张卡就不用」
+/// 按这一批的最小的一片判：乙段一个落点上不止一条条目的那一片随这一批的映射变），活重新按代价分给剩下的卡、再切；参与的卡全都这样才交回
+/// [`BatchNotPlannedOverTheGpus::EveryParticipatingGpuLeftOut`]。各卡都切好才交回，一张卡都还没交活。没分到活的卡不等、不切、不在交回的里面。
+///
+/// 循环每一轮要么交回、要么多一张这一批不用的卡（它在这一轮还参与），所以至多比参与的卡数多一轮。
+///
+/// # Errors
+/// [`BatchNotPlannedOverTheGpus`] 的三种；`room_of_the_position` 交回的出错照原样交回。
+///
+/// # Panics
+/// `participating` 一张卡都没有（调用方先看 `ParticipatingGpusOnThisHost::is_empty`）。
+pub fn plans_of_a_batch_leaving_out_the_gpus_short_of_the_smallest_piece<Plan>(
+    participating: &ParticipatingGpusOnThisHost,
+    measured_cost_of_the_position: impl Fn(usize) -> HostCpuNanosecondsPerSubmission,
+    entry_count: usize,
+    mut room_of_the_position: impl FnMut(
+        usize,
+    )
+        -> Result<RoomOfAGpuForTheBatch, BatchNotPlannedOverTheGpus>,
+    mut cut: impl FnMut(Range<usize>, ChunkBoundsOfAGpu) -> Result<Plan, SmallestPieceLargerThanTheRoom>,
+) -> Result<Vec<PlanOfAGpuForTheBatch<Plan>>, BatchNotPlannedOverTheGpus> {
+    assert!(!participating.is_empty(), "调用方先看过：至少一张参与的卡");
+    let mut positions_left_out: Vec<usize> = Vec::new();
+    let mut gpus_left_out: Vec<GpuLeftOutUnderItsMemoryLimit> = Vec::new();
+    loop {
+        let participating_this_round = participating
+            .clone()
+            .leaving_out(|position| positions_left_out.contains(&position));
+        if participating_this_round.is_empty() {
+            return Err(BatchNotPlannedOverTheGpus::EveryParticipatingGpuLeftOut(
+                gpus_left_out,
+            ));
+        }
+        let mut plans = Vec::new();
+        let mut left_out_this_round = None;
+        for (position_on_this_host, entries) in participating_this_round
+            .entry_ranges_by_the_measured_cost_of(&measured_cost_of_the_position, entry_count)
+        {
+            if entries.is_empty() {
+                continue;
+            }
+            let room_of_the_gpu = room_of_the_position(position_on_this_host)?;
+            match cut(entries.clone(), room_of_the_gpu.room) {
+                Ok(plan) => plans.push(PlanOfAGpuForTheBatch {
+                    position_on_this_host,
+                    entries,
+                    room: room_of_the_gpu.room,
+                    plan,
+                }),
+                Err(smallest_piece) => {
+                    let short_under_the_memory_limit = match (
+                        room_of_the_gpu.memory_limit,
+                        room_of_the_gpu.room.memory_limit,
+                    ) {
+                        (Some(memory_limit), Some(room_left))
+                            if room_left.0 < smallest_piece.smallest_piece_bytes =>
+                        {
+                            Some(memory_limit)
+                        }
+                        (Some(_), Some(_) | None) | (None, _) => None,
+                    };
+                    let Some(memory_limit) = short_under_the_memory_limit else {
+                        return Err(BatchNotPlannedOverTheGpus::GpuCannotHoldTheSmallestPiece(
+                            SmallestPieceNotHeldOnTheGpu {
+                                device: room_of_the_gpu.device,
+                                entry_position: entries.start + smallest_piece.entry_position,
+                                memory_limit: room_of_the_gpu.memory_limit,
+                                smallest_piece_bytes: smallest_piece.smallest_piece_bytes,
+                            },
+                        ));
+                    };
+                    left_out_this_round = Some((
+                        position_on_this_host,
+                        GpuLeftOutUnderItsMemoryLimit {
+                            device: room_of_the_gpu.device,
+                            memory_limit,
+                            smallest_piece_bytes: smallest_piece.smallest_piece_bytes,
+                            bytes_already_held_on_the_device: room_of_the_gpu
+                                .bytes_already_held_on_the_device,
+                        },
+                    ));
+                    break;
+                }
+            }
+        }
+        let Some((position_left_out, gpu_left_out)) = left_out_this_round else {
+            return Ok(plans);
+        };
+        positions_left_out.push(position_left_out);
+        gpus_left_out.push(gpu_left_out);
+    }
+}
+
+/// 一张卡丢后台 GPU 时泄漏了（还有一次等超过上限、没在 GPU 上做完的提交，设备不析构）：哪张卡、它上面那一刻本包占着的字节（泄漏的那一片）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct GpuLeakedWhenDropped {
+    pub device: GpuDeviceIdentifier,
+    pub bytes_held_on_the_device: u64,
+}
+
+/// 一次挂载的后台 GPU 丢掉（卸载、丢掉核心层收的把手）那一刻泄漏了哪几张卡：调用方开卡时拿到一份（`background_gpu_compute::OpenedBackgroundGpuCompute`），
+/// 丢掉之后读（`.claude/rules/fs-design.md` 五条硬要求第 4 条：泄漏没泄漏、泄漏了几张看得出来）。几份把手共用同一份。
+#[derive(Clone, Debug, Default)]
+pub struct GpusLeakedWhenTheBackgroundGpuIsDropped {
+    leaked: Arc<Mutex<Option<Vec<GpuLeakedWhenDropped>>>>,
+}
+
+impl GpusLeakedWhenTheBackgroundGpuIsDropped {
+    /// 还没丢。
+    #[must_use]
+    pub fn not_yet_dropped() -> Self {
+        Self {
+            leaked: Arc::new(Mutex::new(None)),
+        }
+    }
+
+    /// 丢掉那一刻泄漏的卡，按本机的次序（总线地址）；还没丢是 `None`，丢了而一张都没泄漏是空。
+    #[must_use]
+    pub fn gpus_leaked(&self) -> Option<Vec<GpuLeakedWhenDropped>> {
+        self.leaked
+            .lock()
+            .unwrap_or_else(std::sync::PoisonError::into_inner)
+            .clone()
+    }
+
+    pub(crate) fn note_the_drop(&self, gpus_leaked: Vec<GpuLeakedWhenDropped>) {
+        *self
+            .leaked
+            .lock()
+            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(gpus_leaked);
+    }
+}
+
+/// 这个进程里每张卡泄漏着的字节（几次挂载的后台 GPU 丢掉时泄漏的设备不析构，那一片显存这个进程活着就一直占着）：调用方在进程里留一份、
+/// 每次开卡都交同一份（`background_gpu_compute::open_background_gpu_compute`），新一次挂载这张卡的可用余量先扣掉它（第 3 条）。
+/// 同一张卡再泄漏一次时记那一刻它上面占着的全部字节（已经含此前泄漏的那一片）。几份把手共用同一份。
+#[derive(Clone, Debug, Default)]
+pub struct GpuMemoryLeakedInThisProcess {
+    bytes_leaked_on_each_device: Arc<Mutex<Vec<(GpuDeviceIdentifier, u64)>>>,
+}
+
+impl GpuMemoryLeakedInThisProcess {
+    /// 进程里还一张卡都没泄漏过。
+    #[must_use]
+    pub fn with_nothing_leaked() -> Self {
+        Self {
+            bytes_leaked_on_each_device: Arc::new(Mutex::new(Vec::new())),
+        }
+    }
+
+    /// 这张卡上泄漏着的字节；没泄漏过是 0。
+    #[must_use]
+    pub fn bytes_leaked_on(&self, device: &GpuDeviceIdentifier) -> u64 {
+        self.bytes_leaked_on_each_device
+            .lock()
+            .unwrap_or_else(std::sync::PoisonError::into_inner)
+            .iter()
+            .find(|(leaked_device, _bytes)| leaked_device == device)
+            .map_or(0, |(_device, bytes)| *bytes)
+    }
+
+    /// 记下一次丢后台 GPU 时泄漏的卡：每张记它那一刻占着的字节（已含此前泄漏的那一片）。
+    pub fn note_the_gpus_leaked_when_dropped(&self, gpus_leaked: &[GpuLeakedWhenDropped]) {
+        let mut bytes_leaked_on_each_device = self
+            .bytes_leaked_on_each_device
+            .lock()
+            .unwrap_or_else(std::sync::PoisonError::into_inner);
+        for leaked in gpus_leaked {
+            match bytes_leaked_on_each_device
+                .iter_mut()
+                .find(|(leaked_device, _bytes)| *leaked_device == leaked.device)
+            {
+                Some((_device, bytes)) => *bytes = leaked.bytes_held_on_the_device,
+                None => bytes_leaked_on_each_device
+                    .push((leaked.device.clone(), leaked.bytes_held_on_the_device)),
+            }
+        }
+    }
+}
+
 /// 开一张卡时量不量它的代价：参与而显存上限装不下最小的一片的卡不量（不越过上限去量，第 3 条），只起设备与管线，之后换设置给它装得下的上限时再量。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum CostAtTheOpening {
@@ -443,6 +721,7 @@ pub struct GpusOpenedAtTheMount<Opened> {
 /// 没给时本机全部独显都参与）就拒绝挂载、报出它与原因，后面的卡不再开；不参与（给了列表而没点它）就跳过，不进参与的卡，记进
 /// [`DiscreteGpusAfterTheOpening::not_opened`] 报出来，挂载照常。
 /// `participating` 是开卡那一组照同一份 `discrete_gpus` 校验过、取出来的参与的卡（`adapter_inventory::participating_gpus_admitted_by_the_mount_checks`）。
+/// 这个进程里此前几次挂载泄漏在一张卡上的字节（`memory_leaked_in_this_process`）先从它的上限里扣掉再判装不装得下（第 3 条；外壳开这张卡时账也从它起）。
 ///
 /// # Errors
 /// [`BackgroundGpuOpenRefused::GpuUnusable`]：一张参与的卡没开成。
@@ -453,6 +732,7 @@ pub fn gpus_opened_at_the_mount<Opened>(
     discrete_gpus: Vec<DiscreteGpuOnThisHost>,
     participating: &[ParticipatingGpu],
     smallest_piece_bytes: u64,
+    memory_leaked_in_this_process: &GpuMemoryLeakedInThisProcess,
     mut open_the_gpu: impl FnMut(
         &DiscreteGpuOnThisHost,
         Option<GpuMemoryInBytes>,
@@ -474,6 +754,7 @@ pub fn gpus_opened_at_the_mount<Opened>(
             &discrete_gpu.identifier,
             memory_limit_at_the_opening,
             smallest_piece_bytes,
+            memory_leaked_in_this_process.bytes_leaked_on(&discrete_gpu.identifier),
         );
         let cost_at_the_opening = match &left_out {
             Some(_left_out) => CostAtTheOpening::NotMeasuredForTheGpuIsLeftOut,
@@ -545,16 +826,18 @@ pub fn gpus_opened_at_the_mount<Opened>(
 
 /// 丢后台 GPU 时（卸载、丢掉核心层收的把手）逐张丢开着的卡：还有一次等超过上限、没在 GPU 上做完的提交的那几张不丢、泄漏
 /// （它的设备与队列不析构：wgpu-core 30.0.1 的 `Queue` 析构时调 `wait_for_idle`，即 Vulkan 的 `vkQueueWaitIdle`，不带上限），
-/// 别的照常丢（用户 2026-10-28 定「卸载时同样不等、泄漏设备」）。交回泄漏了几张。不等任何一次提交：调用即返回。
+/// 别的照常丢（用户 2026-10-28 定「卸载时同样不等、泄漏设备」）。交回泄漏的那几张（按交进来的次序，各带它上面那一刻占着的字节，
+/// `leaked_when_dropped` 报）。不等任何一次提交：调用即返回。
 pub fn gpus_dropped_leaking_those_still_running<Gpu>(
     gpus: Vec<Gpu>,
     has_a_submission_still_running: impl Fn(&Gpu) -> bool,
-) -> usize {
-    let mut leaked = 0;
+    leaked_when_dropped: impl Fn(&Gpu) -> GpuLeakedWhenDropped,
+) -> Vec<GpuLeakedWhenDropped> {
+    let mut leaked = Vec::new();
     for gpu in gpus {
         if has_a_submission_still_running(&gpu) {
+            leaked.push(leaked_when_dropped(&gpu));
             std::mem::forget(gpu);
-            leaked += 1;
         } else {
             drop(gpu);
         }
@@ -890,6 +1173,7 @@ mod tests {
             discrete_gpus,
             &participating,
             bytes_of_the_smallest_piece_every_participating_gpu_holds(),
+            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
             |discrete_gpu, memory_limit, _cost_at_the_opening| {
                 if discrete_gpu.pci_bus_address == card_that_does_not_open {
                     Err(reason_the_card_did_not_open(card_that_does_not_open))
@@ -1036,6 +1320,7 @@ mod tests {
             discrete_gpus,
             &participating,
             bytes_of_the_smallest_piece_every_participating_gpu_holds(),
+            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
             |discrete_gpu, memory_limit, cost_at_the_opening| {
                 if cost_at_the_opening == CostAtTheOpening::MeasuredUnderTheMemoryLimit {
                     let mut account = GpuMemoryAccount::<u32>::with_nothing_held(memory_limit);
@@ -1146,6 +1431,7 @@ mod tests {
                 device: card("0000:01:00.0"),
                 memory_limit: GpuMemoryInBytes(smallest_piece_bytes - 1),
                 smallest_piece_bytes,
+                bytes_already_held_on_the_device: 0,
             }],
             "报出不用的那张卡、它的上限与至少要多少"
         );
@@ -1234,12 +1520,224 @@ mod tests {
                 dropped: &dropped,
             },
         ];
-        let leaked = gpus_dropped_leaking_those_still_running(gpus, |gpu| gpu.waits_for_its_queue);
-        assert_eq!(leaked, 1, "泄漏还在跑的那一张");
+        let leaked = gpus_dropped_leaking_those_still_running(
+            gpus,
+            |gpu| gpu.waits_for_its_queue,
+            |gpu| GpuLeakedWhenDropped {
+                device: card(gpu.name),
+                bytes_held_on_the_device: 600,
+            },
+        );
+        assert_eq!(
+            leaked,
+            vec![GpuLeakedWhenDropped {
+                device: card("还在跑的卡"),
+                bytes_held_on_the_device: 600,
+            }],
+            "泄漏还在跑的那一张"
+        );
         assert_eq!(
             *dropped.borrow(),
             vec!["跑完了的卡", "没交过活的卡"],
             "别的照常丢"
         );
     }
+
+    /// 测试自己定的占用：每段另占 16 字节、每条另占 40 字节、按条排的最大缓冲每条 16 字节；最小的一片 4 + 40 + 16 = 60 字节。
+    const FOOTPRINT_OF_THE_PLANNING_CASES: ChunkFootprint = ChunkFootprint {
+        bytes_per_chunk_besides_the_entries: 16,
+        bytes_per_entry_besides_its_input: 40,
+        bytes_per_entry_in_the_largest_entry_buffer: 16,
+    };
+
+    /// 切法是 `pieces_cut_to_fit_on_the_gpu` 那一种（几段，每段几片）的一批的结局。
+    type PlansOfPiecesOverTheGpus =
+        Result<Vec<PlanOfAGpuForTheBatch<Vec<Vec<PieceOfAnEntry>>>>, BatchNotPlannedOverTheGpus>;
+
+    /// 两张参与的卡（本机位置 0、1，代价相同）这一批交活之前那一刻各自给的上限（不设别的界），四条各 100 字节的输入照
+    /// `pieces_cut_to_fit_on_the_gpu` 切。交回切法与每张卡被问了几次地方。
+    fn plans_over_two_gpus_with_the_memory_limits(
+        memory_limits_in_bytes: [u64; 2],
+    ) -> (PlansOfPiecesOverTheGpus, [u32; 2]) {
+        let lengths = [100_u64; 4];
+        let mut rooms_asked = [0_u32; 2];
+        let names = ["0000:01:00.0", "0000:41:00.0"];
+        let plans = plans_of_a_batch_leaving_out_the_gpus_short_of_the_smallest_piece(
+            &ParticipatingGpusOnThisHost::of_the_positions(vec![0, 1]),
+            |_position| HostCpuNanosecondsPerSubmission(1_000),
+            lengths.len(),
+            |position| {
+                rooms_asked[position] += 1;
+                Ok(RoomOfAGpuForTheBatch {
+                    device: card(names[position]),
+                    room: ChunkBoundsOfAGpu {
+                        memory_limit: Some(GpuMemoryInBytes(memory_limits_in_bytes[position])),
+                        largest_storage_binding_in_bytes: 1 << 30,
+                        largest_entry_count_per_dispatch: 1 << 20,
+                    },
+                    memory_limit: Some(GpuMemoryInBytes(memory_limits_in_bytes[position])),
+                    bytes_already_held_on_the_device: 0,
+                })
+            },
+            |entries, room| {
+                pieces_cut_to_fit_on_the_gpu(
+                    &lengths[entries],
+                    FOOTPRINT_OF_THE_PLANNING_CASES,
+                    room,
+                )
+            },
+        );
+        (plans, rooms_asked)
+    }
+
+    /// 判决 m4-gpugc-code-r1 第三节第 4 条：两张参与的卡，一张这一批连最小的一片（60 字节）都切不出（上限 50）——这一批不分给它，四条全分给另一张、
+    /// 照它的上限切，照常交回，不停整理（攻方探针 T3-1 那一种，改前这一批交回切不出、整理停下）。
+    #[test]
+    fn a_gpu_short_of_the_smallest_piece_of_the_batch_is_left_out_and_the_other_gpu_takes_the_whole_batch(
+    ) {
+        let (plans, rooms_asked) = plans_over_two_gpus_with_the_memory_limits([50, 1000]);
+        let plans = plans.expect("另一张卡切得出：照常交回");
+        assert_eq!(
+            plans
+                .iter()
+                .map(|plan| (
+                    plan.position_on_this_host,
+                    plan.entries.clone(),
+                    plan.plan.len()
+                ))
+                .collect::<Vec<_>>(),
+            vec![(1, 0..4, 1)],
+            "四条全分给本机位置 1 那张卡，一段装下（4 × 140 + 16 = 576 字节不超 1000）"
+        );
+        assert_eq!(
+            rooms_asked,
+            [1, 1],
+            "头一轮问到切不出的那张就重分；另一张只在重分之后问一次"
+        );
+    }
+
+    /// 同一条：参与的卡这一批全都切不出最小的一片，才报参与的卡一张都不剩，按不用的先后带每张卡、上限、最小的一片与已占的字节。
+    #[test]
+    fn every_participating_gpu_short_of_the_smallest_piece_of_the_batch_is_reported_as_every_gpu_left_out(
+    ) {
+        let (plans, _rooms_asked) = plans_over_two_gpus_with_the_memory_limits([50, 59]);
+        assert_eq!(
+            plans,
+            Err(BatchNotPlannedOverTheGpus::EveryParticipatingGpuLeftOut(
+                vec![
+                    GpuLeftOutUnderItsMemoryLimit {
+                        device: card("0000:01:00.0"),
+                        memory_limit: GpuMemoryInBytes(50),
+                        smallest_piece_bytes: 60,
+                        bytes_already_held_on_the_device: 0,
+                    },
+                    GpuLeftOutUnderItsMemoryLimit {
+                        device: card("0000:41:00.0"),
+                        memory_limit: GpuMemoryInBytes(59),
+                        smallest_piece_bytes: 60,
+                        bytes_already_held_on_the_device: 0,
+                    },
+                ]
+            ))
+        );
+    }
+
+    /// 判决第三节第 6 条：一次丢后台 GPU 时泄漏了一张卡 600 字节，进程里记下；同一张卡新一次挂载、上限 1000 字节，账从 600 起、
+    /// 可用余量只剩 400：五条各 100 字节切成两条一段（296 字节）的三段，与只剩 400 字节时相同。同一张卡再泄漏一次，记那一刻的全部字节。
+    #[test]
+    fn memory_leaked_by_an_earlier_mount_is_taken_off_the_room_of_the_next_mount_on_that_gpu() {
+        let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
+        memory_leaked.note_the_gpus_leaked_when_dropped(&[GpuLeakedWhenDropped {
+            device: card("0000:01:00.0"),
+            bytes_held_on_the_device: 600,
+        }]);
+        assert_eq!(
+            (
+                memory_leaked.bytes_leaked_on(&card("0000:01:00.0")),
+                memory_leaked.bytes_leaked_on(&card("0000:41:00.0"))
+            ),
+            (600, 0),
+            "只记泄漏的那一张"
+        );
+        let mut budget: GpuBudgetUnderTheSettings<u32> =
+            GpuBudgetUnderTheSettings::with_the_bytes_leaked_by_earlier_mounts(
+                1 << 30,
+                1 << 20,
+                Some(GpuMemoryInBytes(1000)),
+                Duration::from_secs(10),
+                memory_leaked.bytes_leaked_on(&card("0000:01:00.0")),
+            );
+        assert_eq!(
+            budget.room_left_now().memory_limit,
+            Some(GpuMemoryInBytes(400)),
+            "上限 1000 先扣掉泄漏的 600"
+        );
+        assert_eq!(
+            budget.chunks_cut_once_the_submission_running_past_its_wait_completed(
+                &ClockThatNeverMoves,
+                |_submission, _time_left| panic!("没有还在跑的提交，不该等"),
+                &[100, 100, 100, 100, 100],
+                FOOTPRINT_OF_THE_PLANNING_CASES,
+            ),
+            Ok(vec![0..2, 2..4, 4..5]),
+            "只用剩下的 400 字节：两条一段"
+        );
+        memory_leaked.note_the_gpus_leaked_when_dropped(&[GpuLeakedWhenDropped {
+            device: card("0000:01:00.0"),
+            bytes_held_on_the_device: 900,
+        }]);
+        assert_eq!(memory_leaked.bytes_leaked_on(&card("0000:01:00.0")), 900);
+    }
+
+    /// 同一条在开卡那一刻：一张参与的卡上限 1000 字节、进程里记着它泄漏了 600 字节——余量 400 装不下最小的一片（972 字节）也不拒挂载，
+    /// 这张卡照开、不量代价、不参与，报出它的上限、最小的一片与已占的 600 字节；不泄漏时同一个上限照常参与。
+    #[test]
+    fn a_gpu_whose_room_after_the_memory_leaked_in_this_process_is_short_of_the_smallest_piece_is_left_out_at_the_mount(
+    ) {
+        let selection = BackgroundGpuSelection {
+            background_gpu_devices: Some(vec![card("0000:01:00.0"), card("0000:41:00.0")]),
+            background_gpu_memory_limit: Some(vec![GpuMemoryInBytes(1000), GpuMemoryInBytes(1000)]),
+            background_gpu_wait_limit: Duration::from_secs(10),
+        };
+        let discrete_gpus = three_discrete_gpus_of_the_enumeration();
+        let participating =
+            participating_gpus_admitted_by_the_mount_checks(&selection, &discrete_gpus)
+                .expect("两张卡、各 1000 字节：校验合法");
+        let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
+        memory_leaked.note_the_gpus_leaked_when_dropped(&[GpuLeakedWhenDropped {
+            device: card("0000:01:00.0"),
+            bytes_held_on_the_device: 600,
+        }]);
+        let opening = gpus_opened_at_the_mount(
+            discrete_gpus,
+            &participating,
+            bytes_of_the_smallest_piece_every_participating_gpu_holds(),
+            &memory_leaked,
+            |discrete_gpu, _memory_limit, cost_at_the_opening| {
+                Ok((discrete_gpu.pci_bus_address.clone(), cost_at_the_opening))
+            },
+        )
+        .expect("余量装不下最小的一片：不用它，不拒挂载");
+        assert_eq!(
+            opening.left_out_under_their_memory_limit,
+            vec![GpuLeftOutUnderItsMemoryLimit {
+                device: card("0000:01:00.0"),
+                memory_limit: GpuMemoryInBytes(1000),
+                smallest_piece_bytes: 972,
+                bytes_already_held_on_the_device: 600,
+            }]
+        );
+        assert_eq!(
+            opening.participating.positions_on_this_host(),
+            &[1],
+            "只剩没泄漏的那一张参与"
+        );
+        assert_eq!(
+            opening.opened[0],
+            (
+                "0000:01:00.0".to_string(),
+                CostAtTheOpening::NotMeasuredForTheGpuIsLeftOut
+            )
+        );
+    }
 }
diff --git a/crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs b/crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs
index 1f058b5..2078423 100644
--- a/crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs
+++ b/crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs
@@ -76,6 +76,21 @@ impl ChunkBoundsOfAGpu {
             largest_entry_count_per_dispatch: self.largest_entry_count_per_dispatch,
         }
     }
+
+    /// 卡上此刻空闲的显存比剩下的地方少（建缓冲显存不足，第 3 条「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」）时，显存那一项压到至多
+    /// `bytes_at_most`：给了上限的取两者小的，不给上限的取它。绑定上限与派发上限不变。
+    #[must_use]
+    pub fn with_the_memory_limit_at_most(self, bytes_at_most: u64) -> Self {
+        Self {
+            memory_limit: Some(GpuMemoryInBytes(
+                self.memory_limit.map_or(bytes_at_most, |memory_limit| {
+                    memory_limit.0.min(bytes_at_most)
+                }),
+            )),
+            largest_storage_binding_in_bytes: self.largest_storage_binding_in_bytes,
+            largest_entry_count_per_dispatch: self.largest_entry_count_per_dispatch,
+        }
+    }
 }
 
 /// 一条单独就装不进一段（[`chunks_that_fit_on_the_gpu`] 不切单条；要切单条的用 [`pieces_cut_to_fit_on_the_gpu`]）。
@@ -285,9 +300,19 @@ pub struct GpuMemoryAccount<Submission> {
 impl<Submission> GpuMemoryAccount<Submission> {
     #[must_use]
     pub fn with_nothing_held(memory_limit: Option<GpuMemoryInBytes>) -> Self {
+        Self::with_the_bytes_leaked_by_earlier_mounts(memory_limit, 0)
+    }
+
+    /// 这个进程里此前几次挂载在这张卡上泄漏了 `bytes_leaked_by_earlier_mounts` 字节（丢后台 GPU 时还在跑的提交，设备不析构，显存不还）：
+    /// 这一次的账从它起，这一次挂载里一直记着、不放（第 3 条「任何时刻都不许比它多」：泄漏的那一片仍占着这张卡）。
+    #[must_use]
+    pub fn with_the_bytes_leaked_by_earlier_mounts(
+        memory_limit: Option<GpuMemoryInBytes>,
+        bytes_leaked_by_earlier_mounts: u64,
+    ) -> Self {
         Self {
             memory_limit,
-            bytes_held: 0,
+            bytes_held: bytes_leaked_by_earlier_mounts,
             submission_running_past_its_wait: None,
         }
     }
@@ -469,6 +494,8 @@ fn failure_of_waiting_for_the_earlier_submission(cause: GpuFailureCause) -> GpuF
         | GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit { .. }
         | GpuFailureCause::ReadbackMappingFailed { .. }
         | GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord(..)
+        | GpuFailureCause::OutOfMemoryCreatingTheBuffers { .. }
+        | GpuFailureCause::FreeMemoryShortOfTheSmallestPiece { .. }
         | GpuFailureCause::UnusableAfterResubmissions { .. }) => reported_by_the_wait,
     }
 }
diff --git a/crates/singlefs-background-gpu/src/region_record_kernels.rs b/crates/singlefs-background-gpu/src/region_record_kernels.rs
index c27423c..1d67073 100644
--- a/crates/singlefs-background-gpu/src/region_record_kernels.rs
+++ b/crates/singlefs-background-gpu/src/region_record_kernels.rs
@@ -660,7 +660,8 @@ pub fn region_record_plan_on_a_gpu<EntryKey>(
 }
 
 /// 区域扫描那一段与输入无关、定得下来的最小的一片：切开的单元头一片连同只放一条条目、一个落点的那一片映射，与头一片之后一个字节的一片连同空映射，
-/// 取大的。一个落点上不止一条条目的那一片随这一批的映射变，挂着时逐批判（[`region_record_plan_on_a_gpu`]）。
+/// 取大的。一个落点上不止一条条目的那一片随这一批的映射变，挂着时逐批判（[`region_record_plan_on_a_gpu`]）：一张卡这一批切不出就这一批不分给它、
+/// 活分给别的卡（`gpus_of_this_host::plans_of_a_batch_leaving_out_the_gpus_short_of_the_smallest_piece`）。
 #[must_use]
 pub fn bytes_of_the_smallest_region_record_piece() -> u64 {
     let head_piece_with_one_entry =
diff --git a/crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs b/crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs
index a754da0..9e587c9 100644
--- a/crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs
+++ b/crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs
@@ -3,10 +3,12 @@
 //! 等一次提交做完、等回读缓冲映射好）都在同一个上限之内等，上限由调用方开包时给（`BackgroundGpuSelection::background_gpu_wait_limit`）。
 //! 超过上限就交回 [`GpuFailureCause::WaitForTheGpuExceededTheLimit`]。
 //!
-//! 等超过上限的那一次提交在同一张卡上重交，至多 [`RESUBMISSIONS_AFTER_A_TIMEOUT`] 次（[`outcome_resubmitted_after_timeouts`]；用户 2026-10-28 定
+//! 等超过上限的那一次提交在同一张卡上重交（[`outcome_resubmitted_after_timeouts`]；用户 2026-10-28 定
 //! 「重试是把那一批重新交一次」：放弃的那一批在同一张卡上重交，最多 3 次，都超时才报警，原话在 `records/2026-10-19-里程碑四GPU垃圾回收开工.md`
-//! 末尾「用户定案（弹窗两次）」那一行）：重交之前先等前一次没做完的那次（照显存的账，等不到算这一次重交失败）；任何一次做完了照常交回；
-//! 都等超过上限就交回 [`GpuFailureCause::UnusableAfterResubmissions`]（这张卡用不了，带最后一次的原因），这一批放弃、不换 CPU，
+//! 末尾「用户定案（弹窗两次）」那一行；用户 2026-10-30 定重交的单位照旧只是超时的那一片）。超时按批、按卡数（[`TimeoutsOfTheBatchOnOneGpu`]）：
+//! 一批切成几片时各片的超时合起来算，交活之前等前一次超时的那次提交那一等（[`outcome_waited_for_again_after_timeouts`]）也算；
+//! 重交之前先等前一次没做完的那次（照显存的账，等不到算又超时了一次）；任何一次做完了照常交回；
+//! 这一批在这张卡上第 1 + [`RESUBMISSIONS_AFTER_A_TIMEOUT`] 次超时就交回 [`GpuFailureCause::UnusableAfterResubmissions`]（这张卡用不了，带最后一次的原因），这一批放弃、不换 CPU，
 //! 连续不确定的计数不动（`two_kernel_reconciliation::ConsecutiveUncertaintyCounts::reconcile_unless_the_batch_failed`）。别的出错不重交。
 //!
 //! 等法与钟、完成信号拆开：钟是 [`MonotonicClock`]（产品用这个进程的单调钟 [`SteadyClockOfThisProcess`]），完成信号是调用方给的
@@ -113,10 +115,11 @@ pub fn wait_for_completion_before_the_deadline<Output>(
     }
 }
 
-/// 一次提交等超过上限之后在同一张卡上至多重交几次（用户 2026-10-28 定「最多 3 次，都超时才报警」）。
+/// 一批里一张卡等超过上限之后在同一张卡上至多重交几次（用户 2026-10-28 定「最多 3 次，都超时才报警」）：这一批在这张卡上第
+/// 1 + [`RESUBMISSIONS_AFTER_A_TIMEOUT`] 次等超过上限就报这张卡用不了。
 pub const RESUBMISSIONS_AFTER_A_TIMEOUT: u32 = 3;
 
-/// 是不是等超过上限的那两种：这一次提交自己等超过上限，或重交之前等前一次那一次又等满了上限。只有这两种重交。
+/// 是不是等超过上限的那两种：这一次提交自己等超过上限，或交活之前、重交之前等前一次那一次又等满了上限。只有这两种重交、算进一批的超时次数。
 fn waited_past_the_limit(cause: &GpuFailureCause) -> bool {
     match cause {
         GpuFailureCause::WaitForTheGpuExceededTheLimit { .. }
@@ -126,41 +129,111 @@ fn waited_past_the_limit(cause: &GpuFailureCause) -> bool {
         | GpuFailureCause::WaitForTheSubmissionFailed { .. }
         | GpuFailureCause::ReadbackMappingFailed { .. }
         | GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord(..)
+        | GpuFailureCause::OutOfMemoryCreatingTheBuffers { .. }
+        | GpuFailureCause::FreeMemoryShortOfTheSmallestPiece { .. }
         | GpuFailureCause::UnusableAfterResubmissions { .. } => false,
     }
 }
 
-/// 在一张卡（`gpu`）上交一次提交（`submit_once`），等超过上限就在同一张卡上重交，至多 [`RESUBMISSIONS_AFTER_A_TIMEOUT`] 次；
-/// 每次重交之前先等前一次等超过上限、还没做完的那次（`wait_for_the_timed_out_submission`，照显存的账在等的上限之内等），等不到算这一次重交失败、
-/// 不交。任何一次做完了交回它的结果。不是等超过上限的出错当场交回，不重交。
+/// 一批里一张卡上到此刻等超过上限了几次、重交了几次（第 4 条「放弃的那一批在同一张卡上重交，至多 3 次，都超时才报警」按批、按卡数：
+/// 一批切成几片、几段时各片的超时合起来算，交活之前等前一次超时的那次提交那一等也算）。一批开头每张卡各一份，从 0 起。
+/// 外壳的「一批」是三段各自交给 GPU 的那一次（`BackgroundGpu` 的三段方法各调一次是一批）。
+#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
+pub struct TimeoutsOfTheBatchOnOneGpu {
+    timeouts: u32,
+    resubmissions: u32,
+}
+
+impl TimeoutsOfTheBatchOnOneGpu {
+    /// 一批开头：一次都还没超时。
+    #[must_use]
+    pub fn none_yet() -> Self {
+        Self {
+            timeouts: 0,
+            resubmissions: 0,
+        }
+    }
+
+    #[must_use]
+    pub fn timeouts(&self) -> u32 {
+        self.timeouts
+    }
+
+    #[must_use]
+    pub fn resubmissions(&self) -> u32 {
+        self.resubmissions
+    }
+
+    /// 记下一次等超过上限的（`cause` 是等超过上限的那两种之一）：这一批在这张卡上第 1 + [`RESUBMISSIONS_AFTER_A_TIMEOUT`] 次就交回这张卡用不了。
+    ///
+    /// # Errors
+    /// 超时次数越过 [`RESUBMISSIONS_AFTER_A_TIMEOUT`] ⇒ [`GpuFailureCause::UnusableAfterResubmissions`]，带这一次的原因。
+    fn note_a_wait_past_the_limit(
+        &mut self,
+        cause: GpuFailureCause,
+    ) -> Result<(), GpuFailureCause> {
+        self.timeouts += 1;
+        if self.timeouts > RESUBMISSIONS_AFTER_A_TIMEOUT {
+            return Err(GpuFailureCause::UnusableAfterResubmissions {
+                resubmissions: RESUBMISSIONS_AFTER_A_TIMEOUT,
+                last_cause: Box::new(cause),
+            });
+        }
+        Ok(())
+    }
+}
+
+/// 在一张卡（`gpu`）上交一次提交（`submit_once`），等超过上限就在同一张卡上重交；超时记进这一批这张卡的那一份（`timeouts_of_the_batch`），
+/// 这一批在这张卡上超时的次数越过 [`RESUBMISSIONS_AFTER_A_TIMEOUT`] 就不再重交。每次重交之前先等前一次等超过上限、还没做完的那次
+/// （`wait_for_the_timed_out_submission`，照显存的账在等的上限之内等），等不到算这一次重交又超时了一次、不交。任何一次做完了交回它的结果。
+/// 不是等超过上限的出错当场交回，不重交。
 ///
-/// 循环至多 [`RESUBMISSIONS_AFTER_A_TIMEOUT`] 轮，每轮的出口是做完、别的出错；跨轮只带最后一次等超过上限的原因。
+/// 循环每一轮要么交回、要么这一批这张卡的超时次数加一；次数越过 [`RESUBMISSIONS_AFTER_A_TIMEOUT`] 就交回，所以至多 1 + [`RESUBMISSIONS_AFTER_A_TIMEOUT`] 轮。
 ///
 /// # Errors
-/// 交了 1 + [`RESUBMISSIONS_AFTER_A_TIMEOUT`] 次都等超过上限 ⇒ [`GpuFailureCause::UnusableAfterResubmissions`]，带最后一次的原因；
+/// 这一批在这张卡上第 1 + [`RESUBMISSIONS_AFTER_A_TIMEOUT`] 次等超过上限 ⇒ [`GpuFailureCause::UnusableAfterResubmissions`]，带最后一次的原因；
 /// 别的出错照原样交回。
 pub fn outcome_resubmitted_after_timeouts<Gpu, Outcome>(
     gpu: &mut Gpu,
+    timeouts_of_the_batch: &mut TimeoutsOfTheBatchOnOneGpu,
     mut wait_for_the_timed_out_submission: impl FnMut(&mut Gpu) -> Result<(), GpuFailureCause>,
     mut submit_once: impl FnMut(&mut Gpu) -> Result<Outcome, GpuFailureCause>,
 ) -> Result<Outcome, GpuFailureCause> {
-    let mut last_cause_past_the_limit = match submit_once(gpu) {
-        Ok(outcome) => return Ok(outcome),
-        Err(cause) if waited_past_the_limit(&cause) => cause,
-        Err(cause) => return Err(cause),
-    };
-    for _resubmission in 0..RESUBMISSIONS_AFTER_A_TIMEOUT {
-        let resubmitted = wait_for_the_timed_out_submission(gpu).and_then(|()| submit_once(gpu));
-        match resubmitted {
+    let mut attempt = submit_once(gpu);
+    loop {
+        match attempt {
             Ok(outcome) => return Ok(outcome),
-            Err(cause) if waited_past_the_limit(&cause) => last_cause_past_the_limit = cause,
+            Err(cause) if waited_past_the_limit(&cause) => {
+                timeouts_of_the_batch.note_a_wait_past_the_limit(cause)?;
+            }
+            Err(cause) => return Err(cause),
+        }
+        timeouts_of_the_batch.resubmissions += 1;
+        attempt = wait_for_the_timed_out_submission(gpu).and_then(|()| submit_once(gpu));
+    }
+}
+
+/// 交活之前在一张卡上先等前一次等超过上限、还没做完的那次提交（`wait_once`，在等的上限之内等一回、交回此刻剩下的地方）：
+/// 等满上限没做完就记进这一批这张卡的超时次数、再等一回，次数越过 [`RESUBMISSIONS_AFTER_A_TIMEOUT`] 就不再等。
+///
+/// 循环每一轮要么交回、要么这一批这张卡的超时次数加一，所以至多 1 + [`RESUBMISSIONS_AFTER_A_TIMEOUT`] 轮。
+///
+/// # Errors
+/// 同 [`outcome_resubmitted_after_timeouts`]。
+pub fn outcome_waited_for_again_after_timeouts<Gpu, Outcome>(
+    gpu: &mut Gpu,
+    timeouts_of_the_batch: &mut TimeoutsOfTheBatchOnOneGpu,
+    mut wait_once: impl FnMut(&mut Gpu) -> Result<Outcome, GpuFailureCause>,
+) -> Result<Outcome, GpuFailureCause> {
+    loop {
+        match wait_once(gpu) {
+            Ok(outcome) => return Ok(outcome),
+            Err(cause) if waited_past_the_limit(&cause) => {
+                timeouts_of_the_batch.note_a_wait_past_the_limit(cause)?;
+            }
             Err(cause) => return Err(cause),
         }
     }
-    Err(GpuFailureCause::UnusableAfterResubmissions {
-        resubmissions: RESUBMISSIONS_AFTER_A_TIMEOUT,
-        last_cause: Box::new(last_cause_past_the_limit),
-    })
 }
 
 #[cfg(test)]
@@ -168,6 +241,10 @@ mod tests {
     use super::*;
     use std::cell::Cell;
 
+    use crate::checksum_kernels::{
+        checksum_outcomes_of_the_units_cut_into_pieces, ChecksumKernelOutcome, UnitChecksumToVerify,
+    };
+    use crate::memory_limit_and_chunks::PieceOfAnEntry;
     use crate::two_kernel_reconciliation::{
         ConsecutiveUncertaintyCounts, TwoKernelOutcomesOfAnEntry,
     };
@@ -373,12 +450,172 @@ mod tests {
         fn outcome_with_resubmissions(&mut self) -> Result<u32, GpuFailureCause> {
             outcome_resubmitted_after_timeouts(
                 self,
+                &mut TimeoutsOfTheBatchOnOneGpu::none_yet(),
                 GpuWithScriptedSubmissions::wait_for_the_timed_out_submission,
                 GpuWithScriptedSubmissions::submit_once,
             )
         }
     }
 
+    /// 一批 `pieces` 片（每片单独一段、各一条 8 字节的单元）照外壳那样逐片交：同一张卡、同一批共用一份超时次数，每片各自套重交。
+    /// 交回这一批的结局与这一批这张卡的超时次数。
+    fn batch_of_pieces_through_the_resubmissions(
+        gpu: &mut GpuWithScriptedSubmissions,
+        pieces: usize,
+    ) -> (
+        Result<Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)>, GpuFailureCause>,
+        TimeoutsOfTheBatchOnOneGpu,
+    ) {
+        let unit_bytes: Vec<Vec<u8>> = (0..pieces)
+            .map(|piece| vec![u8::try_from(piece).expect("小数"); 8])
+            .collect();
+        let units: Vec<UnitChecksumToVerify<'_, usize>> = unit_bytes
+            .iter()
+            .enumerate()
+            .map(|(entry_key, bytes)| UnitChecksumToVerify {
+                entry_key,
+                unit_bytes: bytes,
+                expected_crc32c: 0,
+            })
+            .collect();
+        let chunks_of_pieces: Vec<Vec<PieceOfAnEntry>> = (0..pieces)
+            .map(|entry_position| {
+                vec![PieceOfAnEntry {
+                    entry_position,
+                    input_bytes: 0..8,
+                }]
+            })
+            .collect();
+        let mut timeouts_of_the_batch = TimeoutsOfTheBatchOnOneGpu::none_yet();
+        let outcome = ChecksumKernelOutcome {
+            computed_crc32c: 0,
+            matches_the_expected_crc32c: true,
+        };
+        let batch = checksum_outcomes_of_the_units_cut_into_pieces(
+            &units,
+            &chunks_of_pieces,
+            |piece_units| {
+                let piece_count = piece_units.len();
+                outcome_resubmitted_after_timeouts(
+                    &mut *gpu,
+                    &mut timeouts_of_the_batch,
+                    GpuWithScriptedSubmissions::wait_for_the_timed_out_submission,
+                    |scripted| {
+                        scripted
+                            .submit_once()
+                            .map(|_outcome| vec![(outcome, outcome); piece_count])
+                    },
+                )
+            },
+        );
+        (batch, timeouts_of_the_batch)
+    }
+
+    /// 第 4 条「放弃的那一批在同一张卡上重交，至多 3 次，都超时才报警」按批数：一批切成 4 片、每片头一次交都等超过上限、重交一次就做完——
+    /// 这一批在这张卡上第 4 次超时就报这张卡用不了，第 4 片不再重交（攻方探针 T1-1 那一种，改前按片各重交 3 次、照常交回）。
+    #[test]
+    fn a_batch_of_four_pieces_each_timing_out_once_reports_the_gpu_unusable_at_the_fourth_timeout()
+    {
+        let mut gpu = GpuWithScriptedSubmissions::scripted(
+            vec![
+                Err(this_submission_timed_out()),
+                Ok(0),
+                Err(this_submission_timed_out()),
+                Ok(0),
+                Err(this_submission_timed_out()),
+                Ok(0),
+                Err(this_submission_timed_out()),
+            ],
+            vec![Ok(()), Ok(()), Ok(())],
+        );
+        let (batch, timeouts_of_the_batch) = batch_of_pieces_through_the_resubmissions(&mut gpu, 4);
+        assert_eq!(
+            batch,
+            Err(GpuFailureCause::UnusableAfterResubmissions {
+                resubmissions: 3,
+                last_cause: Box::new(this_submission_timed_out()),
+            }),
+            "四片各超时一次：这一批第 4 次超时报警"
+        );
+        assert_eq!(
+            (
+                gpu.submissions_made,
+                gpu.waits_made,
+                timeouts_of_the_batch.timeouts(),
+                timeouts_of_the_batch.resubmissions()
+            ),
+            (7, 3, 4, 3),
+            "交了 7 次、重交 3 次，第 4 次超时之后不再重交"
+        );
+    }
+
+    /// 同一定案：一批 1 片，超时 3 次、第 4 次交做完——这一批照常交回，不报警。
+    #[test]
+    fn a_batch_of_one_piece_timing_out_three_times_then_completing_is_not_reported() {
+        let mut gpu = GpuWithScriptedSubmissions::scripted(
+            vec![
+                Err(this_submission_timed_out()),
+                Err(this_submission_timed_out()),
+                Err(this_submission_timed_out()),
+                Ok(0),
+            ],
+            vec![Ok(()), Ok(()), Ok(())],
+        );
+        let (batch, timeouts_of_the_batch) = batch_of_pieces_through_the_resubmissions(&mut gpu, 1);
+        assert!(batch.is_ok(), "超时 3 次之后做完：照常交回（{batch:?}）");
+        assert_eq!(
+            (
+                gpu.submissions_made,
+                timeouts_of_the_batch.timeouts(),
+                timeouts_of_the_batch.resubmissions()
+            ),
+            (4, 3, 3)
+        );
+    }
+
+    /// 交活之前先等前一次超时的那次提交那一等也算进这一批这张卡的超时次数：等了两回没等到、第三回等到，再交一片超时两次——第 4 次超时报警。
+    #[test]
+    fn waits_for_the_earlier_submission_before_handing_over_count_toward_the_timeouts_of_the_batch()
+    {
+        let mut gpu = GpuWithScriptedSubmissions::scripted(
+            vec![
+                Err(this_submission_timed_out()),
+                Err(this_submission_timed_out()),
+            ],
+            vec![
+                Err(earlier_submission_still_running()),
+                Err(earlier_submission_still_running()),
+                Ok(()),
+                Ok(()),
+            ],
+        );
+        let mut timeouts_of_the_batch = TimeoutsOfTheBatchOnOneGpu::none_yet();
+        assert_eq!(
+            outcome_waited_for_again_after_timeouts(
+                &mut gpu,
+                &mut timeouts_of_the_batch,
+                GpuWithScriptedSubmissions::wait_for_the_timed_out_submission,
+            ),
+            Ok(()),
+            "交活之前等了三回，第三回等到"
+        );
+        assert_eq!(timeouts_of_the_batch.timeouts(), 2);
+        assert_eq!(
+            outcome_resubmitted_after_timeouts(
+                &mut gpu,
+                &mut timeouts_of_the_batch,
+                GpuWithScriptedSubmissions::wait_for_the_timed_out_submission,
+                GpuWithScriptedSubmissions::submit_once,
+            ),
+            Err(GpuFailureCause::UnusableAfterResubmissions {
+                resubmissions: 3,
+                last_cause: Box::new(this_submission_timed_out()),
+            }),
+            "交活之前那两回加这一片的两回：第 4 次超时报警"
+        );
+        assert_eq!((gpu.submissions_made, gpu.waits_made), (2, 4));
+    }
+
     const WAIT_LIMIT_OF_THE_RESUBMISSIONS: Duration = Duration::from_secs(10);
 
     fn this_submission_timed_out() -> GpuFailureCause {
diff --git a/crates/singlefs-core/src/background_compute.rs b/crates/singlefs-core/src/background_compute.rs
index 44fea41..bee6349 100644
--- a/crates/singlefs-core/src/background_compute.rs
+++ b/crates/singlefs-core/src/background_compute.rs
@@ -43,6 +43,37 @@ pub enum BackgroundTask {
     CompactionRelocation,
 }
 
+impl BackgroundTask {
+    /// 每一个成员各一次，按 [`Self::position_in_every_member`] 的次序。加成员时 `position_in_every_member` 与 [`Self::has_a_gpu_path`]
+    /// 的 match 先编不过；补那两臂时把它补进这里，下面编译期那一判核表里第 i 个成员的位置就是 i。
+    pub const EVERY_MEMBER: [BackgroundTask; 1] = [BackgroundTask::CompactionRelocation];
+
+    /// 它在 [`Self::EVERY_MEMBER`] 里的位置。
+    #[must_use]
+    pub const fn position_in_every_member(self) -> usize {
+        match self {
+            BackgroundTask::CompactionRelocation => 0,
+        }
+    }
+
+    /// 有没有 GPU 路径（第 1 条「凡有 GPU 路径的后台任务」）：有的那几种，挂载那一刻或切换生效时 GPU 一张卡都用不上就当场停着。
+    #[must_use]
+    pub const fn has_a_gpu_path(self) -> bool {
+        match self {
+            BackgroundTask::CompactionRelocation => true,
+        }
+    }
+}
+
+// 编译期核 `BackgroundTask::EVERY_MEMBER` 与 `position_in_every_member` 一一对应：表里第 i 个成员的位置就是 i。
+const _: () = {
+    let mut position = 0;
+    while position < BackgroundTask::EVERY_MEMBER.len() {
+        assert!(BackgroundTask::EVERY_MEMBER[position].position_in_every_member() == position);
+        position += 1;
+    }
+};
+
 /// 一张显卡的标识（`background_gpu_devices` 列表里的一项）：不透明的字节，只比相等。卡用什么认、怎么排是里程碑四设计轮（七·2）
 /// 还挂着的岔路，这里不定格式：卡清单（[`GpuInventory`]）交出什么，调用方就点什么。
 #[derive(Clone, Debug, PartialEq, Eq)]
@@ -193,20 +224,24 @@ pub enum BackgroundGpuComputeFailed {
         wait_limit: Duration,
         last_submission_waited_for: SubmissionWaitedForPastTheLimit,
     },
-    /// 这一组设置点的卡的显存上限连最小的一片都装不下，参与的卡一张都不剩（用户 2026-10-28 定「连一片都装不下就不用这张卡」，
-    /// 「不能因为装不下就拒绝」）：调大显存上限、点别的卡，或切回 `cpu`。按参与的次序列出每张卡、给它的上限与至少要多少，至少一张。
+    /// 这一组设置点的卡的显存上限（扣掉卡上已占的）连最小的一片都装不下，参与的卡一张都不剩（用户 2026-10-28 定「连一片都装不下就不用这张卡」，
+    /// 「不能因为装不下就拒绝」）：开卡、换设置时按与输入无关的最小的一片判，每一批切段时按这一批的最小的一片判（一张卡这一批切不出就这一批不分给它、
+    /// 活分给别的卡，全都切不出才是这一种）。调大显存上限、点别的卡，或切回 `cpu`。按参与的次序列出每张卡、给它的上限、至少要多少与已占的，至少一张。
     EveryParticipatingGpuLeftOutUnderItsMemoryLimit {
         gpus_left_out: Vec<GpuLeftOutUnderItsMemoryLimit>,
     },
 }
 
-/// 一张卡因显存上限装不下最小的一片而不用（用户 2026-10-28 定「连一片都装不下就不用这张卡」）：哪张卡、给它的上限、至少要多少字节
-/// （[`BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit`] 带着；产品 GPU 包开卡与换设置时报的也是它）。
+/// 一张卡因显存上限装不下最小的一片而不用（用户 2026-10-28 定「连一片都装不下就不用这张卡」）：哪张卡、给它的上限、最小的一片要多少字节，
+/// 与判的那一刻卡上已经占着的字节（这个进程里此前几次挂载泄漏在这张卡上的，或等超过上限之后还在跑的那一次提交的；不用的判据是
+/// 「给的上限扣掉已占的不到最小的一片」）（[`BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit`] 带着；
+/// 产品 GPU 包开卡、换设置与每一批切段时报的也是它）。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct GpuLeftOutUnderItsMemoryLimit {
     pub device: GpuDeviceIdentifier,
     pub memory_limit: GpuMemoryInBytes,
     pub smallest_piece_bytes: u64,
+    pub bytes_already_held_on_the_device: u64,
 }
 
 /// 一张卡出错的原因（[`BackgroundGpuComputeFailed::GpuFailed`] 带着）。`reason` 是 GPU 那一侧报的原话。封闭集合。
@@ -225,6 +260,15 @@ pub enum GpuFailureOnTheDevice {
         entry_position: usize,
         verdict_word: u32,
     },
+    /// 在卡上建一段的缓冲时显存不足（`bytes_held_by_the_chunk` 是那一段合占的字节数）。挂着时一批里遇到它按更小的片重切重交（第 3 条
+    /// 「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」），不报出来；报出来的是挂载那一刻量代价时遇到的。
+    OutOfMemoryCreatingTheBuffers {
+        bytes_held_by_the_chunk: u64,
+        reason: String,
+    },
+    /// 挂着时一批里卡上空闲的显存一路切小到连最小的一片（`smallest_piece_bytes` 字节）都建不成：空闲的连最小的一片都不到时怎么办条款没写，
+    /// 第一版不支持，这一批放弃、那个后台任务停下（第 4 条）。
+    FreeMemoryShortOfTheSmallestPieceWhoseHandlingIsUndecided { smallest_piece_bytes: u64 },
 }
 
 /// 等超过上限的是哪一次提交（[`BackgroundGpuComputeFailed::WaitForTheGpuExceededTheLimit`] 带着）。封闭集合。
@@ -240,18 +284,15 @@ pub enum SubmissionWaitedForPastTheLimit {
 /// 下面这几种切不小的怎么办没写。封闭集合。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub enum InputNotSupportedOnTheGpu {
-    /// 一条（甲段的单元、乙段的记录、丙段的段）单独成一段就超过分到它的那张卡此刻剩下的显存或绑定上限。`entry_position` 是它在这一批里的位置，
-    /// `input_bytes` 是它自己的输入字节数。
+    /// 分到活的那张卡这一批连最小的一片都装不下，而装不下的不是显存上限（给的上限扣掉卡上已占的连最小的一片都装不下的卡，这一批不分给它，
+    /// 第 3 条；参与的卡全都这样才报 [`BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit`]）：是单个缓冲的绑定上限。
+    /// `entry_position` 是要切的那一条在这一批里的位置；`input_bytes` 是那最小的一片（一个字节的输入、或乙段一个落点上的全部条目那一片，补到整字）
+    /// 连同一段的固定占用合占的字节数，即这张卡至少要剩这么多（产品 GPU 包按这个数填）。
     EntryLargerThanAChunkOnTheDevice {
         device: GpuDeviceIdentifier,
         entry_position: usize,
         input_bytes: u64,
     },
-    /// 每一段都要交一份的表（乙段的映射、丙段的分配记录表）最大的那个缓冲装不进分到活的那张卡的绑定上限。
-    TablesLargerThanABindingOnTheDevice {
-        device: GpuDeviceIdentifier,
-        largest_buffer_bytes: u64,
-    },
     /// 映射里有一条 key 比核装得下的长：一张卡都还没分活就判出，不归哪张卡。`entry_position` 是它在映射条目里的位置。
     MappingKeyLongerThanTheKernelsHold {
         entry_position: usize,
@@ -313,6 +354,12 @@ pub trait BackgroundGpuCompute: GpuInventory {
     fn failure_under_the_settings_handed_in(&self) -> Option<BackgroundGpuComputeFailed> {
         None
     }
+
+    /// 上一次交回以来三段各自在卡上是怎么交的（交了几片、重交了几次、用了哪几张卡），交回之后清零。核心层每一批三段算完（含出错放弃）之后读一次，
+    /// 记进这一批的记账（[`GpuWorkOfTheBatch::submissions_on_the_gpus`]）。不开卡的实现（测试里的假实现）没有这一样，交回什么都没交的那一份。
+    fn take_the_submissions_of_the_batch(&mut self) -> GpuSubmissionsOfTheBatch {
+        GpuSubmissionsOfTheBatch::default()
+    }
 }
 
 /// 一张参与的卡：它的标识、显存总量（卡清单答的），与给它的显存上限（`None` 是不设上限，第 3 条）。
@@ -417,8 +464,61 @@ impl GpuEntryCounts {
     }
 }
 
-/// 一批在 GPU 上算了什么：三段各自的条目数，与甲段判成整单元 CRC-32C 对不上的单元数。
-#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
+/// 一段在卡上是怎么交的（`.claude/rules/fs-design.md` 五条硬要求第 4 条：运行时看得出切没切片、重交了几次、用了哪几张卡）：
+/// 交成的提交几次、这几次合起来交了几片（整条一片、切开的一条几片），等超过上限之后在同一张卡上重交了几次，建缓冲显存不足之后按更小的片重切了几次，
+/// 分到活的是哪几张卡（按分活的次序，同一张卡只列一次）。一批里同一段算了几次（选段时逐段先扫）的，各次加起来。
+#[derive(Clone, Debug, Default, PartialEq, Eq)]
+pub struct GpuSubmissionsOfOneSegment {
+    pub submissions_completed: u64,
+    pub pieces_submitted: u64,
+    pub resubmissions_after_timeouts: u64,
+    pub recuts_after_running_out_of_memory: u64,
+    pub devices_given_work: Vec<GpuDeviceIdentifier>,
+}
+
+impl GpuSubmissionsOfOneSegment {
+    /// 把同一段后来算的那一次加进来：各数相加，卡照先后并起来、不重列。
+    pub fn add(&mut self, later: &GpuSubmissionsOfOneSegment) {
+        self.submissions_completed += later.submissions_completed;
+        self.pieces_submitted += later.pieces_submitted;
+        self.resubmissions_after_timeouts += later.resubmissions_after_timeouts;
+        self.recuts_after_running_out_of_memory += later.recuts_after_running_out_of_memory;
+        for device in &later.devices_given_work {
+            if !self.devices_given_work.contains(device) {
+                self.devices_given_work.push(device.clone());
+            }
+        }
+    }
+}
+
+/// 一批三段各自在卡上是怎么交的（[`GpuSubmissionsOfOneSegment`]），由实现在一批算完之后交回（[`BackgroundGpuCompute::take_the_submissions_of_the_batch`]）。
+#[derive(Clone, Debug, Default, PartialEq, Eq)]
+pub struct GpuSubmissionsOfTheBatch {
+    pub unit_checksum_verification: GpuSubmissionsOfOneSegment,
+    pub region_record_classification: GpuSubmissionsOfOneSegment,
+    pub live_slots_per_cluster_segment: GpuSubmissionsOfOneSegment,
+}
+
+impl GpuSubmissionsOfTheBatch {
+    /// `segment` 那一段的那一份。
+    pub fn of_the_segment_mut(
+        &mut self,
+        segment: GpuComputedSegment,
+    ) -> &mut GpuSubmissionsOfOneSegment {
+        match segment {
+            GpuComputedSegment::UnitChecksumVerification => &mut self.unit_checksum_verification,
+            GpuComputedSegment::RegionRecordClassification => {
+                &mut self.region_record_classification
+            }
+            GpuComputedSegment::LiveSlotsPerClusterSegment => {
+                &mut self.live_slots_per_cluster_segment
+            }
+        }
+    }
+}
+
+/// 一批在 GPU 上算了什么：三段各自的条目数，甲段判成整单元 CRC-32C 对不上的单元数，与三段各自在卡上是怎么交的。
+#[derive(Clone, Debug, Default, PartialEq, Eq)]
 pub struct GpuWorkOfTheBatch {
     pub unit_checksum_verification: GpuEntryCounts,
     pub region_record_classification: GpuEntryCounts,
@@ -426,6 +526,8 @@ pub struct GpuWorkOfTheBatch {
     /// 甲段确认或交 CPU 判出整单元 CRC-32C 与位置项对不上的可搬单元数。它们照第 8 条交 CPU：发布路径照常逐条核计划、释放之前读盘核
     /// （那一核读出对不上的那一份隔离，D19（块指针的结构与宽度预算） 已定项 5），区域扫描不因它改分类，结局与 CPU 路径相同；这里只报。
     pub units_whose_checksum_mismatched_on_the_gpu: u64,
+    /// 三段各自交了几片、重交了几次、用了哪几张卡（实现在这一批算完之后交回的，出错放弃的那一批是出错之前与出错那一段交的）。
+    pub submissions_on_the_gpus: GpuSubmissionsOfTheBatch,
 }
 
 impl GpuWorkOfTheBatch {
@@ -940,7 +1042,11 @@ fn participating_gpu_devices(
 }
 
 /// 一批实际在哪算、算了什么。封闭集合。
-#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+#[derive(Clone, Debug, PartialEq, Eq)]
+#[allow(
+    clippy::large_enum_variant,
+    reason = "一批一份的记账，不在热路径上：GPU 那一臂带着三段各自交了几片、用了哪几张卡，装箱只多一次分配，换不来什么"
+)]
 pub enum BackgroundComputePathTaken {
     Cpu(ReasonTheBatchRanOnTheCpu),
     /// 三段经递进来的 GPU 实现算（第 1 条），各段的条目数在里面（第 8 条）。
@@ -1031,28 +1137,33 @@ pub enum BackgroundComputeError {
     GpuUnusableUnderTheSettingsHandedIn { failure: BackgroundGpuComputeFailed },
 }
 
-/// 有 GPU 路径的后台任务（第 1 条）：挂载那一刻或切换生效的那一批开头 GPU 用不上时当场停着的就是这几种。里程碑七的 GPU 打包搬迁接上 GPU 路径时加进来。
-const BACKGROUND_TASKS_WITH_A_GPU_PATH: [BackgroundTask; 1] =
-    [BackgroundTask::CompactionRelocation];
+/// 有 GPU 路径的后台任务各一条停着（[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]，补跑的批数 0），按 [`BackgroundTask::EVERY_MEMBER`]
+/// 的次序、由 [`BackgroundTask::has_a_gpu_path`] 现挑：加成员时不用改这里。
+fn tasks_with_a_gpu_path_stopped_by(
+    failure: &BackgroundGpuComputeFailed,
+) -> Vec<BackgroundTaskStoppedAfterAComputeError> {
+    BackgroundTask::EVERY_MEMBER
+        .into_iter()
+        .filter(|task| task.has_a_gpu_path())
+        .map(|task| BackgroundTaskStoppedAfterAComputeError {
+            task,
+            error: BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn {
+                failure: failure.clone(),
+            },
+            batches_compensated_on_the_cpu: 0,
+        })
+        .collect()
+}
 
 /// 把 `settings_handed_in` 交给它带着的实现之后读一次（[`BackgroundGpuCompute::failure_under_the_settings_handed_in`]）：实现照这一组一张卡都用不上，
-/// 就交回有 GPU 路径的每一种后台任务各一条停着（[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]，补跑的批数 0）；用得上、或选 `cpu`
-/// 交回空。挂载那一刻与切换生效的那一批开头共用这一份，两处停着报的是同一个成员。
+/// 就交回有 GPU 路径的每一种后台任务各一条停着（[`tasks_with_a_gpu_path_stopped_by`]）；用得上、或选 `cpu`
+/// 交回空。挂载那一刻与切换生效时共用这一份，两处停着报的是同一个成员。
 fn tasks_stopped_when_no_gpu_is_usable_under_the_settings_handed_in(
     settings_handed_in: &AdmittedBackgroundComputeSettings,
 ) -> Vec<BackgroundTaskStoppedAfterAComputeError> {
     match settings_handed_in.failure_of_the_implementation_under_the_gpu_settings() {
         None => Vec::new(),
-        Some(failure) => BACKGROUND_TASKS_WITH_A_GPU_PATH
-            .iter()
-            .map(|task| BackgroundTaskStoppedAfterAComputeError {
-                task: *task,
-                error: BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn {
-                    failure: failure.clone(),
-                },
-                batches_compensated_on_the_cpu: 0,
-            })
-            .collect(),
+        Some(failure) => tasks_with_a_gpu_path_stopped_by(&failure),
     }
 }
 
@@ -1308,7 +1419,23 @@ impl BackgroundComputeOfThisMount {
         Ok(())
     }
 
-    /// 一批开头（每批读一次设置的那一处）：先把待生效的换上，换上的是 `gpu` 的就在开算之前把它交给带着的实现（从这一批起照它跑，一批之内不再交），
+    /// 两批之间换上挂着请求、待生效的那一组（没有就什么都不做）：换上的是 `gpu` 的就把它交给带着的实现（从下一批起照它跑），实现照它一张卡都用不上
+    /// 就照挂载那一刻一样，有 GPU 路径的后台任务当场记成出错停着（[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]；切换照收，
+    /// 用户 2026-10-28 定「不能因为装不下就拒绝」）；用得上就没有停着的（请求切换那一刻已解除全部停下）。
+    /// 一批开头（[`Self::begin_a_batch`]）调；整理的两个入口在判「整理停没停」之前也调（显式入口一趟开头、后台入口判起不起跑之前），
+    /// 切到一张卡都用不上的那一组之后，下一次进来在任何一个写之前就判出停着：不走删除续做、不取走「该跑了」，与挂载那一刻同。
+    /// 只在两批之间调：一批之内的切换只记成待生效。不读写盘。
+    pub(crate) fn put_the_switch_requested_from_the_next_batch_in_effect(&mut self) {
+        if let Some(requested) = self.requested_from_the_next_batch.take() {
+            self.in_effect = requested;
+            self.in_effect.hand_the_gpu_settings_to_the_implementation();
+            self.stopped_after_a_compute_error =
+                tasks_stopped_when_no_gpu_is_usable_under_the_settings_handed_in(&self.in_effect);
+        }
+    }
+
+    /// 一批开头（每批读一次设置的那一处）：先把待生效的换上（[`Self::put_the_switch_requested_from_the_next_batch_in_effect`]），
+    /// 换上的是 `gpu` 的就在开算之前把它交给带着的实现（从这一批起照它跑，一批之内不再交），
     /// 实现照它一张卡都用不上就照挂载那一刻一样，有 GPU 路径的后台任务当场记成出错停着（[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]；
     /// 切换照收，用户 2026-10-28 定「不能因为装不下就拒绝」）；再判 `task` 这一批跑不跑（[`Self::whether_the_task_runs_the_next_batch`]，`fullest_device`
     /// 是调用方这一刻从分配器取的池子最满的那一块盘）——停着而不补跑就不开这一批；最后定这一批走哪条路：补跑的走 CPU、停下以来补跑的批数加一，
@@ -1330,12 +1457,7 @@ impl BackgroundComputeOfThisMount {
             self.compute_error_of_the_batch_under_way.is_none(),
             "只供测试注入的错只在一批的钩子里注入、那一批就读走：一批开头不该还挂着上一批的"
         );
-        if let Some(requested) = self.requested_from_the_next_batch.take() {
-            self.in_effect = requested;
-            self.in_effect.hand_the_gpu_settings_to_the_implementation();
-            self.stopped_after_a_compute_error =
-                tasks_stopped_when_no_gpu_is_usable_under_the_settings_handed_in(&self.in_effect);
-        }
+        self.put_the_switch_requested_from_the_next_batch_in_effect();
         let compensation = match self.whether_the_task_runs_the_next_batch(task, fullest_device) {
             WhetherTheTaskRunsTheNextBatch::StaysStopped(_) => {
                 return Err(BatchNotBegunWhileTheTaskStaysStopped);
@@ -1421,3 +1543,80 @@ impl InTheMiddleOfEachBackgroundBatch<'_> {
         }
     }
 }
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    /// 第 1 条「凡有 GPU 路径的后台任务」：GPU 一张卡都用不上时停着的，正好是每一个有 GPU 路径的成员各一条，报同一个错、补跑的批数 0。
+    /// 逐个成员穷举：加成员时这里的 match 编不过，逼着写它停不停。
+    #[test]
+    fn every_background_task_with_a_gpu_path_stops_when_no_gpu_is_usable_and_no_other_does() {
+        let failure = BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit {
+            gpus_left_out: vec![GpuLeftOutUnderItsMemoryLimit {
+                device: GpuDeviceIdentifier::of_opaque_bytes(b"0000:01:00.0".to_vec()),
+                memory_limit: GpuMemoryInBytes(64),
+                smallest_piece_bytes: 972,
+                bytes_already_held_on_the_device: 0,
+            }],
+        };
+        let stopped = tasks_with_a_gpu_path_stopped_by(&failure);
+        for task in BackgroundTask::EVERY_MEMBER {
+            let stopped_for_the_task: Vec<&BackgroundTaskStoppedAfterAComputeError> = stopped
+                .iter()
+                .filter(|stopped_task| stopped_task.task == task)
+                .collect();
+            match task {
+                BackgroundTask::CompactionRelocation => assert_eq!(
+                    stopped_for_the_task,
+                    vec![&BackgroundTaskStoppedAfterAComputeError {
+                        task: BackgroundTask::CompactionRelocation,
+                        error: BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn {
+                            failure: failure.clone(),
+                        },
+                        batches_compensated_on_the_cpu: 0,
+                    }],
+                    "整理有 GPU 路径：停着一条"
+                ),
+            }
+        }
+        assert_eq!(
+            stopped.len(),
+            BackgroundTask::EVERY_MEMBER
+                .into_iter()
+                .filter(|task| task.has_a_gpu_path())
+                .count(),
+            "没有 GPU 路径的一条都不停"
+        );
+    }
+
+    /// 同一段一批里算了两次（选段时逐段先扫）：各数相加，卡照先后并起来、同一张卡不重列。
+    #[test]
+    fn submissions_of_a_segment_computed_twice_in_a_batch_add_up_and_list_each_device_once() {
+        let card = |address: &[u8]| GpuDeviceIdentifier::of_opaque_bytes(address.to_vec());
+        let mut submissions = GpuSubmissionsOfOneSegment {
+            submissions_completed: 2,
+            pieces_submitted: 5,
+            resubmissions_after_timeouts: 1,
+            recuts_after_running_out_of_memory: 0,
+            devices_given_work: vec![card(b"0000:41:00.0")],
+        };
+        submissions.add(&GpuSubmissionsOfOneSegment {
+            submissions_completed: 3,
+            pieces_submitted: 4,
+            resubmissions_after_timeouts: 0,
+            recuts_after_running_out_of_memory: 2,
+            devices_given_work: vec![card(b"0000:01:00.0"), card(b"0000:41:00.0")],
+        });
+        assert_eq!(
+            submissions,
+            GpuSubmissionsOfOneSegment {
+                submissions_completed: 5,
+                pieces_submitted: 9,
+                resubmissions_after_timeouts: 1,
+                recuts_after_running_out_of_memory: 2,
+                devices_given_work: vec![card(b"0000:41:00.0"), card(b"0000:01:00.0")],
+            }
+        );
+    }
+}
diff --git a/crates/singlefs-core/src/mounted_session.rs b/crates/singlefs-core/src/mounted_session.rs
index e56a864..1cd5d57 100644
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -2383,6 +2383,10 @@ impl MountedSession {
             PoolVersion::WithFile(_) | PoolVersion::WithTreesAndNoFile(_) => {}
             PoolVersion::WithoutFile(_) => return Err(CompactionRunRefused::NoVersionOfTrees),
         }
+        // 挂着切到一张卡都用不上的那一组：在判停没停之前换上它（两次调用之间不在一批之内），与挂载那一刻一样当场停着，不取走「该跑了」。
+        parameters_and_device_table
+            .background_compute_of_this_mount_mut()
+            .put_the_switch_requested_from_the_next_batch_in_effect();
         let parameters = parameters_and_device_table.parameters_on_disk();
         let background_compute = parameters_and_device_table.background_compute_of_this_mount();
         if compaction_stays_stopped_pausing_the_live_intent(allocator, background_compute) {
@@ -3203,7 +3207,9 @@ fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockD
         }
     }
     // D26（后台整理与放置回收） 已定项 13 第 4 条：整理因一批出错停着，等一次通过校验的切换请求解除；停着而池子每块盘的已分配占它的容量都不超过补跑阈值时
-    // 一个写都不发（删除续做也不走），超过就照常推、每一批在 CPU 上补跑（用户 2026-10-28 定案）。
+    // 一个写都不发（删除续做也不走），超过就照常推、每一批在 CPU 上补跑（用户 2026-10-28 定案）。挂着切到一张卡都用不上的那一组时，
+    // 先在这里换上它（一趟开头不在一批之内），与挂载那一刻一样当场判出停着。
+    background_compute.put_the_switch_requested_from_the_next_batch_in_effect();
     if compaction_stays_stopped_pausing_the_live_intent(allocator, background_compute) {
         return finish(
             compaction,
@@ -3674,7 +3680,7 @@ fn advance_compaction_by_one_publish<Device: BlockDevice>(
             ),
             Some(gpu_compute) => {
                 let mut implementation = gpu_compute.lock();
-                advance_compaction_by_one_publish_computing_on(
+                let advanced_on_the_gpu = advance_compaction_by_one_publish_computing_on(
                     parameters,
                     devices,
                     allocator,
@@ -3688,7 +3694,11 @@ fn advance_compaction_by_one_publish<Device: BlockDevice>(
                         gpu_compute: &mut *implementation,
                         gpu_work: &mut gpu_work,
                     },
-                )
+                );
+                // 三段在卡上是怎么交的（几片、重交几次、哪几张卡）随这一批的记账报出来，出错放弃的那一批也报。
+                gpu_work.submissions_on_the_gpus =
+                    implementation.take_the_submissions_of_the_batch();
+                advanced_on_the_gpu
             }
         },
     };
```

## 第 55 批：`~/.local/share/singlefs/work/m4-gpugc-alarm-every-wake-writer/patch/crates.patch`

```diff
diff --git a/crates/singlefs-background-gpu/src/background_gpu.rs b/crates/singlefs-background-gpu/src/background_gpu.rs
index 21850b8..dcf1dda 100644
--- a/crates/singlefs-background-gpu/src/background_gpu.rs
+++ b/crates/singlefs-background-gpu/src/background_gpu.rs
@@ -47,7 +47,8 @@ use singlefs_core::allocator::AllocationRecord;
 use singlefs_core::background_compute::{
     AdmittedGpuSettings, BackgroundComputeSettingsRefused, ClusterSegmentEntryKey,
     GpuComputedSegment, GpuDeviceIdentifier, GpuDeviceNotOpenedAtTheMount, GpuInventory,
-    GpuMemoryInBytes, GpuSubmissionsOfOneSegment, GpuSubmissionsOfTheBatch,
+    GpuMemoryInBytes, GpuShortOfFreeMemoryForTheSmallestPiece, GpuSubmissionsOfOneSegment,
+    GpuSubmissionsOfTheBatch,
 };
 use singlefs_core::compaction_plan::UnitAreaOfOneDevice;
 use singlefs_core::logged_operations::CompactionRegion;
@@ -148,13 +149,11 @@ pub enum GpuFailureCause {
     /// 读回的判定字认不得：核没写到那一条或写坏了。
     KernelLeftAnUnrecognizedVerdictWord(UnrecognizedVerdictWord),
     /// 在卡上建一段的缓冲时显存不足（wgpu 在显存不足的错误范围里报的），`bytes_held_by_the_chunk` 是那一段合占的字节数；这一段没交，它的账已经放掉。
-    /// 挂着时外壳按更小的片重切重交（[`BackgroundGpuOverTheCards`]），不报出来。
+    /// 挂着时外壳按更小的片重切重交，切到最小的一片都建不成就这一批不用这张卡、活转给别的卡（[`BackgroundGpuOverTheCards`]），不报出来。
     OutOfMemoryCreatingTheBuffers {
         bytes_held_by_the_chunk: u64,
         reason: String,
     },
-    /// 建缓冲显存不足、一路切小到连这一批最小的一片（`smallest_piece_bytes` 字节）都建不成：空闲的连最小的一片都不到时怎么办条款没写，第一版不支持。
-    FreeMemoryShortOfTheSmallestPiece { smallest_piece_bytes: u64 },
     /// 一批里一张卡等超过上限、在同一张卡上重交了 `resubmissions` 次之后，这一批在这张卡上第 1 + `resubmissions` 次等超过上限
     /// （一批切成几片时各片的超时合起来算，交活之前、重交之前等前一次那一次没等到也算一次）：这张卡用不了，
     /// 交核心层当报警（用户 2026-10-28 定「都超时才报警」）。`last_cause` 是最后那一次的原因（等超过上限的那两种之一）。
@@ -206,6 +205,12 @@ pub enum ChecksumBatchFailed {
     GpuCannotHoldTheSmallestPiece(SmallestPieceNotHeldOnTheGpu),
     /// 这组设置点的卡的显存上限连最小的一片都装不下，参与的卡一张都不剩：按参与的次序列出每张卡、上限与至少要多少。一张卡都没交活。
     EveryParticipatingGpuLeftOutUnderItsMemoryLimit(Vec<GpuLeftOutUnderItsMemoryLimit>),
+    /// 这一批参与的卡一张都不剩，其中至少一张是卡上空闲的显存连最小的一片都建不成（用户 2026-10-28 定「连一片都装不下就不用这张卡」：
+    /// 这一批不用它，它还没交成的活转给这一批别的参与的卡重交，全都不剩才是这一种）：因空闲显存不用的按先后列出，连同这一段因显存上限不用的。
+    EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
+        gpus_short_of_free_memory: Vec<GpuShortOfFreeMemoryForTheSmallestPiece>,
+        gpus_left_out_under_their_memory_limit: Vec<GpuLeftOutUnderItsMemoryLimit>,
+    },
 }
 
 /// 一批区域扫描的单元头解析与映射反查交不成。后三种在一张卡都还没交活之前判出。
@@ -222,6 +227,11 @@ pub enum RegionRecordBatchFailed {
     GpuCannotHoldTheSmallestPiece(SmallestPieceNotHeldOnTheGpu),
     /// 同 [`ChecksumBatchFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit`]。
     EveryParticipatingGpuLeftOutUnderItsMemoryLimit(Vec<GpuLeftOutUnderItsMemoryLimit>),
+    /// 同 [`ChecksumBatchFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece`]。
+    EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
+        gpus_short_of_free_memory: Vec<GpuShortOfFreeMemoryForTheSmallestPiece>,
+        gpus_left_out_under_their_memory_limit: Vec<GpuLeftOutUnderItsMemoryLimit>,
+    },
 }
 
 /// 一批选段打分交不成。后两种在一张卡都还没交活之前判出。
@@ -236,6 +246,11 @@ pub enum LiveSlotBatchFailed {
     GpuCannotHoldTheSmallestPiece(SmallestPieceNotHeldOnTheGpu),
     /// 同 [`ChecksumBatchFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit`]。
     EveryParticipatingGpuLeftOutUnderItsMemoryLimit(Vec<GpuLeftOutUnderItsMemoryLimit>),
+    /// 同 [`ChecksumBatchFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece`]。
+    EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
+        gpus_short_of_free_memory: Vec<GpuShortOfFreeMemoryForTheSmallestPiece>,
+        gpus_left_out_under_their_memory_limit: Vec<GpuLeftOutUnderItsMemoryLimit>,
+    },
 }
 
 /// 此刻这组设置下一批分不到卡上（[`BackgroundGpu::batch_not_split_over_the_gpus`]）：换设置时量一张卡的代价出错了（这张卡没参与，报成它的 GPU 出错），
@@ -412,6 +427,14 @@ impl SubmissionsOfACardInTheBatch {
     }
 }
 
+/// 一段的一次调用里一张卡交它分到的那几条的结局（[`BackgroundGpuOverTheCards`] 的三段共用的那一套）。封闭集合。
+enum OutcomesOfACardInTheBatch<Outcome> {
+    Computed(Vec<Outcome>),
+    /// 卡上空闲的显存一路切小到连最小的一片都建不成：这一批不用这张卡，它分到的那几条重新分给别的卡。
+    ShortOfFreeMemory(GpuShortOfFreeMemoryForTheSmallestPiece),
+    Failed(GpuFailureCause),
+}
+
 /// 一批没算成（三段各自的出错照它报：`checksum_batch_failed_of` 这三个）。封闭集合。
 #[derive(Clone, Debug, PartialEq, Eq)]
 enum OutcomesOfTheBatchNotComputed {
@@ -420,6 +443,10 @@ enum OutcomesOfTheBatchNotComputed {
         cause: GpuFailureCause,
     },
     EveryParticipatingGpuLeftOut(Vec<GpuLeftOutUnderItsMemoryLimit>),
+    EveryParticipatingGpuShortOfFreeMemory {
+        gpus_short_of_free_memory: Vec<GpuShortOfFreeMemoryForTheSmallestPiece>,
+        gpus_left_out_under_their_memory_limit: Vec<GpuLeftOutUnderItsMemoryLimit>,
+    },
     GpuCannotHoldTheSmallestPiece(SmallestPieceNotHeldOnTheGpu),
 }
 
@@ -432,6 +459,13 @@ fn checksum_batch_failed_of(not_computed: OutcomesOfTheBatchNotComputed) -> Chec
             OutcomesOfTheBatchNotComputed::EveryParticipatingGpuLeftOut(gpus_left_out) => {
                 ChecksumBatchFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit(gpus_left_out)
             }
+            OutcomesOfTheBatchNotComputed::EveryParticipatingGpuShortOfFreeMemory {
+                gpus_short_of_free_memory,
+                gpus_left_out_under_their_memory_limit,
+            } => ChecksumBatchFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
+                gpus_short_of_free_memory,
+                gpus_left_out_under_their_memory_limit,
+            },
             OutcomesOfTheBatchNotComputed::GpuCannotHoldTheSmallestPiece(smallest_piece) => {
                 ChecksumBatchFailed::GpuCannotHoldTheSmallestPiece(smallest_piece)
             }
@@ -452,6 +486,15 @@ fn region_record_batch_failed_of(
                     gpus_left_out,
                 )
             }
+            OutcomesOfTheBatchNotComputed::EveryParticipatingGpuShortOfFreeMemory {
+                gpus_short_of_free_memory,
+                gpus_left_out_under_their_memory_limit,
+            } => {
+                RegionRecordBatchFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
+                    gpus_short_of_free_memory,
+                    gpus_left_out_under_their_memory_limit,
+                }
+            }
             OutcomesOfTheBatchNotComputed::GpuCannotHoldTheSmallestPiece(smallest_piece) => {
                 RegionRecordBatchFailed::GpuCannotHoldTheSmallestPiece(smallest_piece)
             }
@@ -468,6 +511,13 @@ fn live_slot_batch_failed_of(not_computed: OutcomesOfTheBatchNotComputed) -> Liv
             OutcomesOfTheBatchNotComputed::EveryParticipatingGpuLeftOut(gpus_left_out) => {
                 LiveSlotBatchFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit(gpus_left_out)
             }
+            OutcomesOfTheBatchNotComputed::EveryParticipatingGpuShortOfFreeMemory {
+                gpus_short_of_free_memory,
+                gpus_left_out_under_their_memory_limit,
+            } => LiveSlotBatchFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
+                gpus_short_of_free_memory,
+                gpus_left_out_under_their_memory_limit,
+            },
             OutcomesOfTheBatchNotComputed::GpuCannotHoldTheSmallestPiece(smallest_piece) => {
                 LiveSlotBatchFailed::GpuCannotHoldTheSmallestPiece(smallest_piece)
             }
@@ -1574,12 +1624,22 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
         ))
     }
 
-    /// 一批（`entry_count` 条）分给参与的卡、各卡切好（`cut` 切一张卡分到的那几条，照这张卡交活之前那一刻的地方）：交活之前先等这张卡上前一次超时的那次提交
+    /// 一批里 `entries` 那几条分给这一批还用的参与的卡、各卡切好（`cut` 切一张卡分到的那几条，照这张卡交活之前那一刻的地方）：
+    /// 这一批因空闲显存不用的卡（`gpus_short_of_free_memory_in_this_batch`，它在本机里的位置连同报的那一条，按不用的先后）不分；交活之前先等这张卡上前一次超时的那次提交
     /// （等满上限算这一批这张卡超时一次、再等，`waiting_for_the_gpu::outcome_waited_for_again_after_timeouts`），切不出最小的一片（显存上限扣掉已占的）的卡
-    /// 这一批不分给它（`gpus_of_this_host::plans_of_a_batch_leaving_out_the_gpus_short_of_the_smallest_piece`）。没有条目交回空。
+    /// 这一批不分给它（`gpus_of_this_host::plans_of_a_batch_leaving_out_the_gpus_short_of_the_smallest_piece`）。交回的各卡分到的条目在 `entries` 里、
+    /// 按卡的次序相连。没有条目交回空。
+    ///
+    /// # Errors
+    /// 换设置时量代价出错、参与的卡因显存上限一张都不剩、这一批还用的卡都因空闲显存不用了（[`OutcomesOfTheBatchNotComputed::EveryParticipatingGpuShortOfFreeMemory`]，
+    /// 连同这一次因显存上限不用的）、交活之前那一等出错、一张卡切不出而装不下的是绑定上限。
     fn plans_of_a_batch<Plan>(
         &mut self,
-        entry_count: usize,
+        entries: Range<usize>,
+        gpus_short_of_free_memory_in_this_batch: &[(
+            usize,
+            GpuShortOfFreeMemoryForTheSmallestPiece,
+        )],
         submissions_of_each_card: &mut [SubmissionsOfACardInTheBatch],
         cut: impl Fn(Range<usize>, ChunkBoundsOfAGpu) -> Result<Plan, SmallestPieceLargerThanTheRoom>,
     ) -> Result<Vec<PlanOfAGpuForTheBatch<Plan>>, OutcomesOfTheBatchNotComputed> {
@@ -1588,7 +1648,7 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
                 return Err(OutcomesOfTheBatchNotComputed::GpuFailed { device, cause });
             }
             Some(BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(gpus_left_out)) => {
-                if entry_count == 0 {
+                if entries.is_empty() {
                     return Ok(Vec::new());
                 }
                 return Err(OutcomesOfTheBatchNotComputed::EveryParticipatingGpuLeftOut(
@@ -1597,22 +1657,46 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
             }
             None => {}
         }
-        if entry_count == 0 {
+        if entries.is_empty() {
             return Ok(Vec::new());
         }
+        let gpus_short_of_free_memory: Vec<GpuShortOfFreeMemoryForTheSmallestPiece> =
+            gpus_short_of_free_memory_in_this_batch
+                .iter()
+                .map(|(_position, short)| short.clone())
+                .collect();
+        let positions_short_of_free_memory: Vec<usize> = gpus_short_of_free_memory_in_this_batch
+            .iter()
+            .map(|(position, _short)| *position)
+            .collect();
+        let participating_in_this_batch = self
+            .participating
+            .clone()
+            .leaving_out(|position| positions_short_of_free_memory.contains(&position));
+        if participating_in_this_batch.is_empty() {
+            return Err(
+                OutcomesOfTheBatchNotComputed::EveryParticipatingGpuShortOfFreeMemory {
+                    gpus_short_of_free_memory,
+                    gpus_left_out_under_their_memory_limit: Vec::new(),
+                },
+            );
+        }
         let measured_costs: Vec<Option<HostCpuNanosecondsPerSubmission>> = self
             .cards_on_this_host
             .iter()
             .map(CardOfTheBackgroundGpu::measured_cost)
             .collect();
         let cards_on_this_host = &mut self.cards_on_this_host;
+        let first_entry = entries.start;
+        let shifted_by_the_first_entry =
+            |relative: Range<usize>| (relative.start + first_entry)..(relative.end + first_entry);
         plans_of_a_batch_leaving_out_the_gpus_short_of_the_smallest_piece(
-            &self.participating,
+            &participating_in_this_batch,
             |position| {
                 measured_costs[position]
                     .expect("参与的卡都量过代价：开卡或换设置让它参与的那一刻量的")
             },
-            entry_count,
+            entries.len(),
             |position| {
                 let card = &mut cards_on_this_host[position];
                 let device = card.discrete_gpu().identifier.clone();
@@ -1633,29 +1717,57 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
                     bytes_already_held_on_the_device: memory_account.bytes_held(),
                 })
             },
-            cut,
+            |relative, room| cut(shifted_by_the_first_entry(relative), room),
         )
+        .map(|plans| {
+            plans
+                .into_iter()
+                .map(|plan| PlanOfAGpuForTheBatch {
+                    position_on_this_host: plan.position_on_this_host,
+                    entries: shifted_by_the_first_entry(plan.entries),
+                    room: plan.room,
+                    plan: plan.plan,
+                })
+                .collect()
+        })
         .map_err(|not_planned| match not_planned {
             BatchNotPlannedOverTheGpus::GpuFailed { device, cause } => {
                 OutcomesOfTheBatchNotComputed::GpuFailed { device, cause }
             }
-            BatchNotPlannedOverTheGpus::EveryParticipatingGpuLeftOut(gpus_left_out) => {
+            BatchNotPlannedOverTheGpus::EveryParticipatingGpuLeftOut(gpus_left_out)
+                if gpus_short_of_free_memory.is_empty() =>
+            {
                 OutcomesOfTheBatchNotComputed::EveryParticipatingGpuLeftOut(gpus_left_out)
             }
+            BatchNotPlannedOverTheGpus::EveryParticipatingGpuLeftOut(gpus_left_out) => {
+                OutcomesOfTheBatchNotComputed::EveryParticipatingGpuShortOfFreeMemory {
+                    gpus_short_of_free_memory,
+                    gpus_left_out_under_their_memory_limit: gpus_left_out,
+                }
+            }
             BatchNotPlannedOverTheGpus::GpuCannotHoldTheSmallestPiece(smallest_piece) => {
-                OutcomesOfTheBatchNotComputed::GpuCannotHoldTheSmallestPiece(smallest_piece)
+                OutcomesOfTheBatchNotComputed::GpuCannotHoldTheSmallestPiece(
+                    SmallestPieceNotHeldOnTheGpu {
+                        device: smallest_piece.device,
+                        entry_position: smallest_piece.entry_position + first_entry,
+                        memory_limit: smallest_piece.memory_limit,
+                        smallest_piece_bytes: smallest_piece.smallest_piece_bytes,
+                    },
+                )
             }
         })
     }
 
-    /// 一段（`segment`）的一批照三段共用的那一套交给参与的卡：分活、切（[`Self::plans_of_a_batch`]），再逐张卡照切法交
+    /// 一段（`segment`）的一次调用照三段共用的那一套交给参与的卡：分活、切（[`Self::plans_of_a_batch`]），再逐张卡照切法交
     /// （`run_the_plan_on_a_card`：这一段自己的交法，交它这张卡、分到的那几条、切法与这一批这张卡的那一份，每一段套重交）。
-    /// 建缓冲显存不足（[`GpuFailureCause::OutOfMemoryCreatingTheBuffers`]）就把这张卡这一批的地方压到建不成的那一段的一半、重切、从头再交它分到的那几条
-    /// （第 3 条「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」）；压到连最小的一片都切不出就交回
-    /// [`GpuFailureCause::FreeMemoryShortOfTheSmallestPiece`]（条款没写，第一版不支持）。各卡交回的接起来，按条目的次序。
+    /// 建缓冲显存不足（[`GpuFailureCause::OutOfMemoryCreatingTheBuffers`]）就把这张卡这一次的地方压到建不成的那一段的一半、重切、从头再交它分到的那几条
+    /// （第 3 条「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」）；压到连最小的一片都切不出，这一批就不用这张卡（用户 2026-10-28 定
+    /// 「连一片都装不下就不用这张卡」；下一批照设置重新起），它分到的那几条（一条都没交回）重新分给这一批还用的参与的卡、照它们的地方切、交；
+    /// 都不用了交回 [`OutcomesOfTheBatchNotComputed::EveryParticipatingGpuShortOfFreeMemory`]。各卡交回的按条目的次序接起来。
     /// 交成了几次、几片、重交了几次、重切了几次、哪几张卡分到了活，记进这一段的那一份（出错也记，[`Self::take_the_submissions_of_the_batch`]）。
     ///
-    /// 重切那一圈：每一轮要么交回、要么这张卡的显存那一项严格变小（建不成的那一段是按此刻的界切的，不比界大，压到它的一半），压到切不出就交回，所以有界。
+    /// 重切那一圈：每一轮要么交回、要么这张卡的显存那一项严格变小（建不成的那一段是按此刻的界切的，不比界大，压到它的一半），压到切不出就停，所以有界。
+    /// 重分那一圈：每次把一段条目放回去，就多一张这一批不用的卡（不再分给它），参与的卡有限，所以至多比参与的卡数多一轮。
     fn outcomes_of_a_batch_over_the_cards<Plan, Outcome>(
         &mut self,
         segment: GpuComputedSegment,
@@ -1673,60 +1785,81 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
             .iter()
             .map(|_card| SubmissionsOfACardInTheBatch::none_yet())
             .collect();
-        let plans = match self.plans_of_a_batch(entry_count, &mut submissions_of_each_card, &cut) {
-            Ok(plans) => plans,
-            Err(not_planned) => {
-                self.note_the_submissions_of_the_segment(
-                    segment,
-                    &submissions_of_each_card,
-                    Vec::new(),
-                    0,
-                );
-                return Err(not_planned);
-            }
-        };
-        let mut outcomes = Vec::with_capacity(entry_count);
-        let mut devices_given_work = Vec::with_capacity(plans.len());
+        let mut gpus_short_of_free_memory_in_this_batch: Vec<(
+            usize,
+            GpuShortOfFreeMemoryForTheSmallestPiece,
+        )> = Vec::new();
+        let mut outcomes_from_the_first_entry: Vec<(usize, Vec<Outcome>)> = Vec::new();
+        let mut devices_given_work: Vec<GpuDeviceIdentifier> = Vec::new();
         let mut recuts_after_running_out_of_memory: u64 = 0;
+        let mut entries_to_hand_out: Vec<Range<usize>> = vec![0..entry_count];
         let mut failure = None;
-        for planned in plans {
-            let card = &mut self.cards_on_this_host[planned.position_on_this_host];
-            let device = card.discrete_gpu().identifier.clone();
-            devices_given_work.push(device.clone());
-            let submissions_of_the_card =
-                &mut submissions_of_each_card[planned.position_on_this_host];
-            let mut room = planned.room;
-            let mut plan = planned.plan;
-            let outcomes_of_the_card = loop {
-                match run_the_plan_on_a_card(
-                    &mut *card,
-                    planned.entries.clone(),
-                    &plan,
-                    &mut *submissions_of_the_card,
-                ) {
-                    Err(GpuFailureCause::OutOfMemoryCreatingTheBuffers {
-                        bytes_held_by_the_chunk,
-                        reason: _,
-                    }) => {
-                        recuts_after_running_out_of_memory += 1;
-                        room = room.with_the_memory_limit_at_most(bytes_held_by_the_chunk / 2);
-                        match cut(planned.entries.clone(), room) {
-                            Ok(recut) => plan = recut,
-                            Err(smallest_piece) => {
-                                break Err(GpuFailureCause::FreeMemoryShortOfTheSmallestPiece {
-                                    smallest_piece_bytes: smallest_piece.smallest_piece_bytes,
-                                })
+        'handing_out: while let Some(entries) = entries_to_hand_out.pop() {
+            let plans = match self.plans_of_a_batch(
+                entries,
+                &gpus_short_of_free_memory_in_this_batch,
+                &mut submissions_of_each_card,
+                &cut,
+            ) {
+                Ok(plans) => plans,
+                Err(not_planned) => {
+                    failure = Some(not_planned);
+                    break 'handing_out;
+                }
+            };
+            for planned in plans {
+                let card = &mut self.cards_on_this_host[planned.position_on_this_host];
+                let device = card.discrete_gpu().identifier.clone();
+                devices_given_work.push(device.clone());
+                let submissions_of_the_card =
+                    &mut submissions_of_each_card[planned.position_on_this_host];
+                let mut room = planned.room;
+                let mut plan = planned.plan;
+                let outcomes_of_the_card = loop {
+                    match run_the_plan_on_a_card(
+                        &mut *card,
+                        planned.entries.clone(),
+                        &plan,
+                        &mut *submissions_of_the_card,
+                    ) {
+                        Err(GpuFailureCause::OutOfMemoryCreatingTheBuffers {
+                            bytes_held_by_the_chunk,
+                            reason: _,
+                        }) => {
+                            recuts_after_running_out_of_memory += 1;
+                            room = room.with_the_memory_limit_at_most(bytes_held_by_the_chunk / 2);
+                            match cut(planned.entries.clone(), room) {
+                                Ok(recut) => plan = recut,
+                                Err(smallest_piece) => {
+                                    break OutcomesOfACardInTheBatch::ShortOfFreeMemory(
+                                        GpuShortOfFreeMemoryForTheSmallestPiece {
+                                            device: device.clone(),
+                                            bytes_of_the_last_chunk_not_created:
+                                                bytes_held_by_the_chunk,
+                                            smallest_piece_bytes: smallest_piece
+                                                .smallest_piece_bytes,
+                                        },
+                                    )
+                                }
                             }
                         }
+                        Ok(of_the_card) => break OutcomesOfACardInTheBatch::Computed(of_the_card),
+                        Err(cause) => break OutcomesOfACardInTheBatch::Failed(cause),
+                    }
+                };
+                match outcomes_of_the_card {
+                    OutcomesOfACardInTheBatch::Computed(of_the_card) => {
+                        outcomes_from_the_first_entry.push((planned.entries.start, of_the_card));
+                    }
+                    OutcomesOfACardInTheBatch::ShortOfFreeMemory(short) => {
+                        gpus_short_of_free_memory_in_this_batch
+                            .push((planned.position_on_this_host, short));
+                        entries_to_hand_out.push(planned.entries);
+                    }
+                    OutcomesOfACardInTheBatch::Failed(cause) => {
+                        failure = Some(OutcomesOfTheBatchNotComputed::GpuFailed { device, cause });
+                        break 'handing_out;
                     }
-                    computed => break computed,
-                }
-            };
-            match outcomes_of_the_card {
-                Ok(of_the_card) => outcomes.extend(of_the_card),
-                Err(cause) => {
-                    failure = Some(OutcomesOfTheBatchNotComputed::GpuFailed { device, cause });
-                    break;
                 }
             }
         }
@@ -1739,10 +1872,20 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
         if let Some(failure) = failure {
             return Err(failure);
         }
+        outcomes_from_the_first_entry.sort_by_key(|(first_entry, _outcomes)| *first_entry);
+        let mut outcomes = Vec::with_capacity(entry_count);
+        for (first_entry, of_the_card) in outcomes_from_the_first_entry {
+            assert_eq!(
+                first_entry,
+                outcomes.len(),
+                "各卡交回的条目按次序相连、不重不漏：重分的那几条接在原来的位置上"
+            );
+            outcomes.extend(of_the_card);
+        }
         assert_eq!(
             outcomes.len(),
             entry_count,
-            "每条都有两份结果，按卡的次序接回"
+            "每条都有两份结果，按条目的次序接回"
         );
         Ok(outcomes)
     }
@@ -2804,9 +2947,23 @@ mod tests {
         );
     }
 
-    /// 同一条：卡上只空闲 10 字节，一路压小到连最小的一片（64 字节）都建不成——交回空闲的连最小的一片都不到（条款没写，第一版不支持），重切了 5 次。
+    /// 只空闲 10 字节的那张替身卡上一个 1000 字节的单元一路压小（1060、528、264、132、64 字节的一段各建不成一次）到连最小的一片（64 字节）都建不成时，
+    /// 它报的那一条：最后建不成的一段 64 字节、最小的一片 64 字节。
+    fn short_of_free_memory_after_cutting_down_a_unit_of_one_thousand_bytes(
+        pci_bus_address: &str,
+    ) -> GpuShortOfFreeMemoryForTheSmallestPiece {
+        GpuShortOfFreeMemoryForTheSmallestPiece {
+            device: card_identifier(pci_bus_address),
+            bytes_of_the_last_chunk_not_created: 64,
+            smallest_piece_bytes: 64,
+        }
+    }
+
+    /// 判决第三节第 5 条连同用户 2026-10-28「连一片都装不下就不用这张卡」：唯一一张卡上只空闲 10 字节，一路压小到连最小的一片（64 字节）都建不成——这一批不用这张卡，参与的卡一张都不剩，
+    /// 报空闲显存那一种（它是因空闲显存不用的那一张，没有因显存上限不用的），重切了 5 次。
     #[test]
-    fn a_card_whose_free_memory_is_short_of_the_smallest_piece_reports_it() {
+    fn a_single_card_short_of_free_memory_for_the_smallest_piece_leaves_every_participating_gpu_short(
+    ) {
         let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
         card.free_bytes = Some(10);
         let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
@@ -2816,12 +2973,16 @@ mod tests {
         let unit_bytes = unit_bytes_of(1, 1000);
         assert_eq!(
             background_gpu.both_kernel_outcomes_of_the_batch(&units_to_verify(&unit_bytes)),
-            Err(ChecksumBatchFailed::GpuFailed {
-                device: card_identifier("0000:01:00.0"),
-                cause: GpuFailureCause::FreeMemoryShortOfTheSmallestPiece {
-                    smallest_piece_bytes: 64
-                },
-            })
+            Err(
+                ChecksumBatchFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
+                    gpus_short_of_free_memory: vec![
+                        short_of_free_memory_after_cutting_down_a_unit_of_one_thousand_bytes(
+                            "0000:01:00.0"
+                        )
+                    ],
+                    gpus_left_out_under_their_memory_limit: Vec::new(),
+                }
+            )
         );
         assert_eq!(
             background_gpu
@@ -2833,6 +2994,130 @@ mod tests {
         );
     }
 
+    /// 用户 2026-10-28「连一片都装不下就不用这张卡」用于空闲显存：两张卡、两个 1000 字节的单元按代价一张一个，「0000:01:00.0」只空闲 10 字节、
+    /// 压到最小的一片都建不成——这一批不用它，它分到的那一条转给「0000:41:00.0」重交，照常交回、与 CPU 参照逐条相同；这一段记下交成 2 次、2 片、
+    /// 重切 5 次、分到活的是两张卡（先「0000:01:00.0」）。
+    #[test]
+    fn a_checksum_batch_goes_to_the_other_card_when_one_card_is_short_of_free_memory_for_the_smallest_piece(
+    ) {
+        let mut card_short_of_free_memory =
+            CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
+        card_short_of_free_memory.free_bytes = Some(10);
+        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
+            vec![
+                card_short_of_free_memory,
+                CardComputedOnTheCpuForTests::of_the_address("0000:41:00.0", None, 0),
+            ],
+            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+        );
+        let unit_bytes = unit_bytes_of(2, 1000);
+        let units = units_to_verify(&unit_bytes);
+        assert_eq!(
+            background_gpu.both_kernel_outcomes_of_the_batch(&units),
+            Ok(cpu_reference_outcomes_of(&units)),
+            "一张卡空闲的装不下最小的一片：活转给另一张，照常交回"
+        );
+        assert_eq!(
+            background_gpu
+                .take_the_submissions_of_the_batch()
+                .unit_checksum_verification,
+            GpuSubmissionsOfOneSegment {
+                submissions_completed: 2,
+                pieces_submitted: 2,
+                resubmissions_after_timeouts: 0,
+                recuts_after_running_out_of_memory: 5,
+                devices_given_work: vec![
+                    card_identifier("0000:01:00.0"),
+                    card_identifier("0000:41:00.0")
+                ],
+            }
+        );
+    }
+
+    /// 同上，两张卡都只空闲 10 字节：这一批两张都不用，报空闲显存那一种，按不用的先后列出两张；重切了 10 次（一张 5 次）。
+    #[test]
+    fn a_checksum_batch_whose_cards_are_all_short_of_free_memory_reports_every_participating_gpu_short(
+    ) {
+        let mut cards = vec![
+            CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0),
+            CardComputedOnTheCpuForTests::of_the_address("0000:41:00.0", None, 0),
+        ];
+        for card in &mut cards {
+            card.free_bytes = Some(10);
+        }
+        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
+            cards,
+            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+        );
+        let unit_bytes = unit_bytes_of(2, 1000);
+        assert_eq!(
+            background_gpu.both_kernel_outcomes_of_the_batch(&units_to_verify(&unit_bytes)),
+            Err(
+                ChecksumBatchFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
+                    gpus_short_of_free_memory: vec![
+                        short_of_free_memory_after_cutting_down_a_unit_of_one_thousand_bytes(
+                            "0000:01:00.0"
+                        ),
+                        short_of_free_memory_after_cutting_down_a_unit_of_one_thousand_bytes(
+                            "0000:41:00.0"
+                        ),
+                    ],
+                    gpus_left_out_under_their_memory_limit: Vec::new(),
+                }
+            )
+        );
+        assert_eq!(
+            background_gpu
+                .take_the_submissions_of_the_batch()
+                .unit_checksum_verification
+                .recuts_after_running_out_of_memory,
+            10
+        );
+    }
+
+    /// 「下一批照设置重新起」：两张卡、「0000:01:00.0」只空闲 10 字节、两个 1000 字节的单元，这一批不用它；下一批（外壳的一批是一段的一次调用）
+    /// 它又照设置分到活（又压小 5 次、又不用它），两条都照常交回。
+    #[test]
+    fn a_card_short_of_free_memory_is_given_work_again_in_the_next_batch() {
+        let mut card_short_of_free_memory =
+            CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
+        card_short_of_free_memory.free_bytes = Some(10);
+        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
+            vec![
+                card_short_of_free_memory,
+                CardComputedOnTheCpuForTests::of_the_address("0000:41:00.0", None, 0),
+            ],
+            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+        );
+        let unit_bytes = unit_bytes_of(2, 1000);
+        let units = units_to_verify(&unit_bytes);
+        assert_eq!(
+            background_gpu.both_kernel_outcomes_of_the_batch(&units),
+            Ok(cpu_reference_outcomes_of(&units))
+        );
+        drop(background_gpu.take_the_submissions_of_the_batch());
+        assert_eq!(
+            background_gpu.both_kernel_outcomes_of_the_batch(&units),
+            Ok(cpu_reference_outcomes_of(&units))
+        );
+        assert_eq!(
+            background_gpu
+                .take_the_submissions_of_the_batch()
+                .unit_checksum_verification,
+            GpuSubmissionsOfOneSegment {
+                submissions_completed: 2,
+                pieces_submitted: 2,
+                resubmissions_after_timeouts: 0,
+                recuts_after_running_out_of_memory: 5,
+                devices_given_work: vec![
+                    card_identifier("0000:01:00.0"),
+                    card_identifier("0000:41:00.0")
+                ],
+            },
+            "下一批那张卡又分到活、又压小 5 次"
+        );
+    }
+
     /// 判决第三节第 8 条：丢外壳时，记着一次还在跑的提交（占 600 字节）的那张卡泄漏、另一张照常丢；泄漏了哪一张、占着多少交到调用方拿着的那一份，
     /// 并记进进程里的那一份（新一次挂载这张卡先扣掉）。
     #[test]
diff --git a/crates/singlefs-background-gpu/src/background_gpu_compute.rs b/crates/singlefs-background-gpu/src/background_gpu_compute.rs
index 9d44e4b..6dcc380 100644
--- a/crates/singlefs-background-gpu/src/background_gpu_compute.rs
+++ b/crates/singlefs-background-gpu/src/background_gpu_compute.rs
@@ -22,9 +22,9 @@ use singlefs_core::background_compute::{
     AdmittedGpuSettings, BackgroundGpuCompute, BackgroundGpuComputeFailed,
     BackgroundGpuComputeHandle, ClusterSegmentEntryKey, GpuDeviceIdentifier,
     GpuDeviceNotOpenedAtTheMount, GpuEntryDecision, GpuFailureOnTheDevice, GpuInventory,
-    GpuMemoryInBytes, GpuSubmissionsOfTheBatch, InputNotSupportedOnTheGpu, RegionRecordEntryKey,
-    RegionRecordToClassifyOnTheGpu, SubmissionWaitedForPastTheLimit, UnitChecksumOutcome,
-    UnitChecksumToVerifyOnTheGpu,
+    GpuMemoryInBytes, GpuShortOfFreeMemoryForTheSmallestPiece, GpuSubmissionsOfTheBatch,
+    InputNotSupportedOnTheGpu, RegionRecordEntryKey, RegionRecordToClassifyOnTheGpu,
+    SubmissionWaitedForPastTheLimit, UnitChecksumOutcome, UnitChecksumToVerifyOnTheGpu,
 };
 use singlefs_core::compaction_plan::UnitAreaOfOneDevice;
 use singlefs_core::logged_operations::CompactionRegion;
@@ -301,13 +301,6 @@ pub fn core_failure_of_the_gpu(
             bytes_held_by_the_chunk: *bytes_held_by_the_chunk,
             reason: reason.clone(),
         }),
-        GpuFailureCause::FreeMemoryShortOfTheSmallestPiece {
-            smallest_piece_bytes,
-        } => failed_on_the_device(
-            GpuFailureOnTheDevice::FreeMemoryShortOfTheSmallestPieceWhoseHandlingIsUndecided {
-                smallest_piece_bytes: *smallest_piece_bytes,
-            },
-        ),
         GpuFailureCause::UnusableAfterResubmissions {
             resubmissions,
             last_cause,
@@ -340,7 +333,6 @@ pub fn core_failure_of_the_gpu(
             | GpuFailureCause::ReadbackMappingFailed { .. }
             | GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord(..)
             | GpuFailureCause::OutOfMemoryCreatingTheBuffers { .. }
-            | GpuFailureCause::FreeMemoryShortOfTheSmallestPiece { .. }
             | GpuFailureCause::UnusableAfterResubmissions { .. }) => {
                 core_failure_of_the_gpu(device, last_cause_other_than_a_wait_past_the_limit)
             }
@@ -358,6 +350,19 @@ pub fn core_failure_of_every_participating_gpu_left_out(
     }
 }
 
+/// 这一批参与的卡一张都不剩、其中至少一张是因空闲显存连最小的一片都建不成，报给核心层：`EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece`，
+/// 因空闲显存不用的按先后、连同这一段因显存上限不用的，原样带过去。
+#[must_use]
+pub fn core_failure_of_every_participating_gpu_short_of_free_memory(
+    gpus_short_of_free_memory: &[GpuShortOfFreeMemoryForTheSmallestPiece],
+    gpus_left_out_under_their_memory_limit: &[GpuLeftOutUnderItsMemoryLimit],
+) -> BackgroundGpuComputeFailed {
+    BackgroundGpuComputeFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
+        gpus_short_of_free_memory: gpus_short_of_free_memory.to_vec(),
+        gpus_left_out_under_their_memory_limit: gpus_left_out_under_their_memory_limit.to_vec(),
+    }
+}
+
 /// 此刻这组设置下一批分不到卡上，报给核心层：换设置时量代价出错照 [`core_failure_of_the_gpu`]，参与的卡一张都不剩照
 /// [`core_failure_of_every_participating_gpu_left_out`]。三段交回的与挂载那一刻核心层读到的都照它报。
 #[must_use]
@@ -402,6 +407,13 @@ pub fn core_failure_of_the_checksum_batch(
         ChecksumBatchFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit(gpus_left_out) => {
             core_failure_of_every_participating_gpu_left_out(gpus_left_out)
         }
+        ChecksumBatchFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
+            gpus_short_of_free_memory,
+            gpus_left_out_under_their_memory_limit,
+        } => core_failure_of_every_participating_gpu_short_of_free_memory(
+            gpus_short_of_free_memory,
+            gpus_left_out_under_their_memory_limit,
+        ),
     }
 }
 
@@ -429,6 +441,13 @@ pub fn core_failure_of_the_region_record_batch(
         RegionRecordBatchFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit(gpus_left_out) => {
             core_failure_of_every_participating_gpu_left_out(gpus_left_out)
         }
+        RegionRecordBatchFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
+            gpus_short_of_free_memory,
+            gpus_left_out_under_their_memory_limit,
+        } => core_failure_of_every_participating_gpu_short_of_free_memory(
+            gpus_short_of_free_memory,
+            gpus_left_out_under_their_memory_limit,
+        ),
     }
 }
 
@@ -445,6 +464,13 @@ pub fn core_failure_of_the_live_slot_batch(
         LiveSlotBatchFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit(gpus_left_out) => {
             core_failure_of_every_participating_gpu_left_out(gpus_left_out)
         }
+        LiveSlotBatchFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
+            gpus_short_of_free_memory,
+            gpus_left_out_under_their_memory_limit,
+        } => core_failure_of_every_participating_gpu_short_of_free_memory(
+            gpus_short_of_free_memory,
+            gpus_left_out_under_their_memory_limit,
+        ),
     }
 }
 
@@ -664,16 +690,6 @@ mod tests {
                     reason: "假的：显存不足".to_string(),
                 }),
             ),
-            (
-                GpuFailureCause::FreeMemoryShortOfTheSmallestPiece {
-                    smallest_piece_bytes: 64,
-                },
-                failed_on_the_card(
-                    GpuFailureOnTheDevice::FreeMemoryShortOfTheSmallestPieceWhoseHandlingIsUndecided {
-                        smallest_piece_bytes: 64,
-                    },
-                ),
-            ),
         ]
     }
 
@@ -895,6 +911,71 @@ mod tests {
         );
     }
 
+    /// 用户 2026-10-28「连一片都装不下就不用这张卡」用于空闲显存：三段报的「这一批参与的卡一张都不剩、其中有因空闲显存不用的」报给核心层是
+    /// `EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece`，因空闲显存不用的（按先后）与因显存上限不用的原样带过去；核心层读得出点名的卡
+    /// （因空闲显存不用的在前）。
+    #[test]
+    fn every_participating_gpu_short_of_free_memory_is_reported_with_each_card_short_and_those_left_out_under_their_limit(
+    ) {
+        let gpus_short_of_free_memory = vec![GpuShortOfFreeMemoryForTheSmallestPiece {
+            device: card(),
+            bytes_of_the_last_chunk_not_created: 64,
+            smallest_piece_bytes: 64,
+        }];
+        let gpus_left_out_under_their_memory_limit = vec![GpuLeftOutUnderItsMemoryLimit {
+            device: GpuDeviceIdentifier::of_opaque_bytes(b"0000:02:00.0".to_vec()),
+            memory_limit: GpuMemoryInBytes(50),
+            smallest_piece_bytes: 64,
+            bytes_already_held_on_the_device: 0,
+        }];
+        let expected =
+            BackgroundGpuComputeFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
+                gpus_short_of_free_memory: gpus_short_of_free_memory.clone(),
+                gpus_left_out_under_their_memory_limit: gpus_left_out_under_their_memory_limit
+                    .clone(),
+            };
+        assert_eq!(
+            core_failure_of_the_checksum_batch(
+                &ChecksumBatchFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
+                    gpus_short_of_free_memory: gpus_short_of_free_memory.clone(),
+                    gpus_left_out_under_their_memory_limit: gpus_left_out_under_their_memory_limit
+                        .clone(),
+                }
+            ),
+            expected,
+            "甲段"
+        );
+        assert_eq!(
+            core_failure_of_the_region_record_batch(
+                &RegionRecordBatchFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
+                    gpus_short_of_free_memory: gpus_short_of_free_memory.clone(),
+                    gpus_left_out_under_their_memory_limit: gpus_left_out_under_their_memory_limit
+                        .clone(),
+                }
+            ),
+            expected,
+            "乙段"
+        );
+        assert_eq!(
+            core_failure_of_the_live_slot_batch(
+                &LiveSlotBatchFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
+                    gpus_short_of_free_memory,
+                    gpus_left_out_under_their_memory_limit,
+                }
+            ),
+            expected,
+            "丙段"
+        );
+        assert_eq!(
+            expected.gpu_devices_named(),
+            vec![
+                card(),
+                GpuDeviceIdentifier::of_opaque_bytes(b"0000:02:00.0".to_vec())
+            ],
+            "因空闲显存不用的在前、因显存上限不用的在后"
+        );
+    }
+
     #[test]
     fn each_entry_decision_maps_to_the_core_decision_of_the_same_kind() {
         assert_eq!(
diff --git a/crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs b/crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs
index 2078423..667fb38 100644
--- a/crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs
+++ b/crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs
@@ -495,7 +495,6 @@ fn failure_of_waiting_for_the_earlier_submission(cause: GpuFailureCause) -> GpuF
         | GpuFailureCause::ReadbackMappingFailed { .. }
         | GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord(..)
         | GpuFailureCause::OutOfMemoryCreatingTheBuffers { .. }
-        | GpuFailureCause::FreeMemoryShortOfTheSmallestPiece { .. }
         | GpuFailureCause::UnusableAfterResubmissions { .. }) => reported_by_the_wait,
     }
 }
diff --git a/crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs b/crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs
index 9e587c9..aa178d6 100644
--- a/crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs
+++ b/crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs
@@ -130,7 +130,6 @@ fn waited_past_the_limit(cause: &GpuFailureCause) -> bool {
         | GpuFailureCause::ReadbackMappingFailed { .. }
         | GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord(..)
         | GpuFailureCause::OutOfMemoryCreatingTheBuffers { .. }
-        | GpuFailureCause::FreeMemoryShortOfTheSmallestPiece { .. }
         | GpuFailureCause::UnusableAfterResubmissions { .. } => false,
     }
 }
@@ -435,14 +434,25 @@ mod tests {
             }
         }
 
+        /// 照脚本交回这一次提交的结局；脚本用完之后都做完（交 [`SUBMISSION_OUTCOME_AFTER_THE_SCRIPT`]）：被测的重交多交了几次时，
+        /// 用例照样走得完、红在断言上，不在脚本的下标上越界。
         fn submit_once(&mut self) -> Result<u32, GpuFailureCause> {
-            let outcome = self.submission_outcomes[self.submissions_made].clone();
+            let outcome = self
+                .submission_outcomes
+                .get(self.submissions_made)
+                .cloned()
+                .unwrap_or(Ok(SUBMISSION_OUTCOME_AFTER_THE_SCRIPT));
             self.submissions_made += 1;
             outcome
         }
 
+        /// 照脚本交回这一回等前一次提交的结局；脚本用完之后都等到。
         fn wait_for_the_timed_out_submission(&mut self) -> Result<(), GpuFailureCause> {
-            let outcome = self.wait_outcomes[self.waits_made].clone();
+            let outcome = self
+                .wait_outcomes
+                .get(self.waits_made)
+                .cloned()
+                .unwrap_or(Ok(()));
             self.waits_made += 1;
             outcome
         }
@@ -618,6 +628,9 @@ mod tests {
 
     const WAIT_LIMIT_OF_THE_RESUBMISSIONS: Duration = Duration::from_secs(10);
 
+    /// 脚本用完之后每次提交都做完、交回的值。
+    const SUBMISSION_OUTCOME_AFTER_THE_SCRIPT: u32 = 0;
+
     fn this_submission_timed_out() -> GpuFailureCause {
         GpuFailureCause::WaitForTheGpuExceededTheLimit {
             wait_limit: WAIT_LIMIT_OF_THE_RESUBMISSIONS,
diff --git a/crates/singlefs-core/src/background_compute.rs b/crates/singlefs-core/src/background_compute.rs
index bee6349..1620917 100644
--- a/crates/singlefs-core/src/background_compute.rs
+++ b/crates/singlefs-core/src/background_compute.rs
@@ -230,6 +230,60 @@ pub enum BackgroundGpuComputeFailed {
     EveryParticipatingGpuLeftOutUnderItsMemoryLimit {
         gpus_left_out: Vec<GpuLeftOutUnderItsMemoryLimit>,
     },
+    /// 这一批参与的卡一张都不剩，其中至少一张是因为卡上此刻空闲的显存连这一批最小的一片都建不成（用户 2026-10-28 定「连一片都装不下就不用这张卡」：
+    /// 给的上限之内建缓冲显存不足就一路切小重交，切到最小的一片都建不成，这一批就不用这张卡、它还没交成的活转给这一批别的参与的卡重交；
+    /// 下一批照设置重新起）。腾出卡上的显存、点别的卡，或切回 `cpu`。`gpus_short_of_free_memory` 按不用的先后列出因空闲显存不用的卡，至少一张；
+    /// `gpus_left_out_under_their_memory_limit` 是这一段切的时候因显存上限（扣掉已占的）不用的卡，可以是空。
+    EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
+        gpus_short_of_free_memory: Vec<GpuShortOfFreeMemoryForTheSmallestPiece>,
+        gpus_left_out_under_their_memory_limit: Vec<GpuLeftOutUnderItsMemoryLimit>,
+    },
+}
+
+impl BackgroundGpuComputeFailed {
+    /// 这个错点名的卡，按错里的次序：出在一张卡上的是那一张；参与的卡一张都不剩的是每一张被晾下的（因空闲显存不用的在前）；
+    /// 不归哪张卡的（映射 key 比核装得下的长）是空。
+    #[must_use]
+    pub fn gpu_devices_named(&self) -> Vec<GpuDeviceIdentifier> {
+        match self {
+            BackgroundGpuComputeFailed::GpuFailed { device, .. }
+            | BackgroundGpuComputeFailed::WaitForTheGpuExceededTheLimit { device, .. }
+            | BackgroundGpuComputeFailed::GpuUnusableAfterResubmissions { device, .. }
+            | BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(
+                InputNotSupportedOnTheGpu::EntryLargerThanAChunkOnTheDevice { device, .. },
+            ) => vec![device.clone()],
+            BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(
+                InputNotSupportedOnTheGpu::MappingKeyLongerThanTheKernelsHold { .. },
+            ) => Vec::new(),
+            BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit {
+                gpus_left_out,
+            } => gpus_left_out
+                .iter()
+                .map(|left_out| left_out.device.clone())
+                .collect(),
+            BackgroundGpuComputeFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
+                gpus_short_of_free_memory,
+                gpus_left_out_under_their_memory_limit,
+            } => gpus_short_of_free_memory
+                .iter()
+                .map(|short| short.device.clone())
+                .chain(
+                    gpus_left_out_under_their_memory_limit
+                        .iter()
+                        .map(|left_out| left_out.device.clone()),
+                )
+                .collect(),
+        }
+    }
+}
+
+/// 一张卡这一批因空闲显存连最小的一片都建不成而不用（[`BackgroundGpuComputeFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece`] 带着）：
+/// 哪张卡、它最后那一段建不成时那一段合占的字节数（再压到它的一半就切不出最小的一片了）、这一段最小的一片要多少字节。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct GpuShortOfFreeMemoryForTheSmallestPiece {
+    pub device: GpuDeviceIdentifier,
+    pub bytes_of_the_last_chunk_not_created: u64,
+    pub smallest_piece_bytes: u64,
 }
 
 /// 一张卡因显存上限装不下最小的一片而不用（用户 2026-10-28 定「连一片都装不下就不用这张卡」）：哪张卡、给它的上限、最小的一片要多少字节，
@@ -261,14 +315,12 @@ pub enum GpuFailureOnTheDevice {
         verdict_word: u32,
     },
     /// 在卡上建一段的缓冲时显存不足（`bytes_held_by_the_chunk` 是那一段合占的字节数）。挂着时一批里遇到它按更小的片重切重交（第 3 条
-    /// 「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」），不报出来；报出来的是挂载那一刻量代价时遇到的。
+    /// 「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」），切到最小的一片都建不成就这一批不用这张卡
+    /// （[`BackgroundGpuComputeFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece`]），不报出来；报出来的是挂载那一刻量代价时遇到的。
     OutOfMemoryCreatingTheBuffers {
         bytes_held_by_the_chunk: u64,
         reason: String,
     },
-    /// 挂着时一批里卡上空闲的显存一路切小到连最小的一片（`smallest_piece_bytes` 字节）都建不成：空闲的连最小的一片都不到时怎么办条款没写，
-    /// 第一版不支持，这一批放弃、那个后台任务停下（第 4 条）。
-    FreeMemoryShortOfTheSmallestPieceWhoseHandlingIsUndecided { smallest_piece_bytes: u64 },
 }
 
 /// 等超过上限的是哪一次提交（[`BackgroundGpuComputeFailed::WaitForTheGpuExceededTheLimit`] 带着）。封闭集合。
@@ -1137,6 +1189,45 @@ pub enum BackgroundComputeError {
     GpuUnusableUnderTheSettingsHandedIn { failure: BackgroundGpuComputeFailed },
 }
 
+impl BackgroundComputeError {
+    /// 这个错点名的卡（[`BackgroundGpuComputeFailed::gpu_devices_named`]）；只供测试的钩子注入的不归哪张卡，是空。
+    #[must_use]
+    pub fn gpu_devices_named(&self) -> Vec<GpuDeviceIdentifier> {
+        match self {
+            BackgroundComputeError::InjectedByTheTestOnlyHook => Vec::new(),
+            BackgroundComputeError::GpuSegmentFailed { failure, .. }
+            | BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn { failure } => {
+                failure.gpu_devices_named()
+            }
+        }
+    }
+}
+
+/// 后台任务因一批出错停着时，每次叫起都交回的报警（D26（后台整理与放置回收） 已定项 13 第 4 条；用户 2026-10-30 定「GPU 后台任务照旧保留超时重交
+/// 三次失败后要报警  之后每次都要报警，不管后台有没有gpu可以用。一定要告诉fs用户 提醒。」，同日弹窗定「每次」是每次后台整理被叫起、通道是返回值里必须处理的报警）：
+/// 哪个后台任务、停下的那一个错与它点名的卡、停了多久、停着期间在 CPU 上补跑了几批。停了多久不读挂钟，数叫起的次数：从停下那一刻起每次叫起结束时停着就加一
+/// （叫起时停下的那一次算第 1 次；挂载那一刻就停着的，头一次叫起算第 1 次）。这一次叫起在 CPU 上补跑了一批还是照旧停着、此刻 GPU 用不用得上都照报，
+/// 直到一次通过校验的切换解除停下。
+#[must_use = "后台任务因 GPU 出错停着：每次叫起都要把这份报警告诉文件系统的用户（用户 2026-10-30 定）"]
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct AlarmOfABackgroundTaskStoppedAfterAComputeError {
+    pub task: BackgroundTask,
+    /// 停下的那一个错点名的卡（[`BackgroundComputeError::gpu_devices_named`]），按错里的次序；不归哪张卡的是空。
+    pub gpu_devices_named_by_the_error: Vec<GpuDeviceIdentifier>,
+    pub error_that_stopped_the_task: BackgroundComputeError,
+    /// 停下以来叫起了几次（含这一次）。
+    pub wakes_while_stopped: u64,
+    /// 停下以来在 CPU 上补跑的批数（[`BackgroundTaskStoppedAfterAComputeError::batches_compensated_on_the_cpu`]，含这一次叫起补跑的）。
+    pub batches_compensated_on_the_cpu: u64,
+}
+
+/// 一种停着的后台任务停下以来叫起了几次（[`BackgroundComputeOfThisMount::alarm_at_the_end_of_a_wake`] 数的）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+struct WakesOfAStoppedTask {
+    task: BackgroundTask,
+    wakes_while_stopped: u64,
+}
+
 /// 有 GPU 路径的后台任务各一条停着（[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]，补跑的批数 0），按 [`BackgroundTask::EVERY_MEMBER`]
 /// 的次序、由 [`BackgroundTask::has_a_gpu_path`] 现挑：加成员时不用改这里。
 fn tasks_with_a_gpu_path_stopped_by(
@@ -1220,6 +1311,9 @@ pub struct BackgroundComputeOfThisMount {
     in_effect: AdmittedBackgroundComputeSettings,
     requested_from_the_next_batch: Option<AdmittedBackgroundComputeSettings>,
     stopped_after_a_compute_error: Vec<BackgroundTaskStoppedAfterAComputeError>,
+    /// 停着的后台任务停下以来叫起了几次：头一次叫起时记下，停着的那几条换掉或清掉时一起清掉（[`Self::replace_the_tasks_stopped`]）。
+    /// 一种后台任务只经换掉、清掉不再停着，所以这里没有不停着的任务的那一条。
+    wakes_of_the_tasks_stopped: Vec<WakesOfAStoppedTask>,
     /// 只供测试的钩子在这一批里注入的错，这一批读完它就清掉（[`Self::take_the_compute_error_of_the_batch_under_way`]）。
     compute_error_of_the_batch_under_way: Option<BackgroundComputeError>,
     /// 补跑那一判照什么判：产品恒是照读数判，只供测试的开关能改。
@@ -1242,6 +1336,7 @@ impl BackgroundComputeOfThisMount {
             in_effect: admitted_at_the_mount,
             requested_from_the_next_batch: None,
             stopped_after_a_compute_error,
+            wakes_of_the_tasks_stopped: Vec::new(),
             compute_error_of_the_batch_under_way: None,
             gc_cpu_compensation_judgement: GcCpuCompensationJudgement::ByTheReading,
         }
@@ -1304,6 +1399,54 @@ impl BackgroundComputeOfThisMount {
         BackgroundTaskAfterTheComputeError::LeftStopped
     }
 
+    /// 换掉停着的那几条（`tasks_stopped` 可以是空：解除全部停下），停下以来叫起的次数一起清掉：换上的每一条都从这一刻起重新数。
+    fn replace_the_tasks_stopped(
+        &mut self,
+        tasks_stopped: Vec<BackgroundTaskStoppedAfterAComputeError>,
+    ) {
+        self.stopped_after_a_compute_error = tasks_stopped;
+        self.wakes_of_the_tasks_stopped.clear();
+    }
+
+    /// 一次叫起结束时（整理的显式入口每一趟、后台入口每一次判起跑，不论起没起跑、补没补跑；入口在任何读写之前就拒的那一次不算）：`task` 此刻停着，
+    /// 就把它停下以来叫起的次数加一、交回这一次的报警（[`AlarmOfABackgroundTaskStoppedAfterAComputeError`]，用户 2026-10-30 定）；没停着交回 `None`。
+    /// 不读写盘。
+    #[must_use = "停着的后台任务每次叫起都要报警：交回的报警要交到调用方手里"]
+    pub(crate) fn alarm_at_the_end_of_a_wake(
+        &mut self,
+        task: BackgroundTask,
+    ) -> Option<AlarmOfABackgroundTaskStoppedAfterAComputeError> {
+        let stopped = self
+            .stopped_after_a_compute_error
+            .iter()
+            .find(|stopped| stopped.task == task)?
+            .clone();
+        let wakes_while_stopped = match self
+            .wakes_of_the_tasks_stopped
+            .iter_mut()
+            .find(|wakes| wakes.task == task)
+        {
+            Some(wakes) => {
+                wakes.wakes_while_stopped += 1;
+                wakes.wakes_while_stopped
+            }
+            None => {
+                self.wakes_of_the_tasks_stopped.push(WakesOfAStoppedTask {
+                    task,
+                    wakes_while_stopped: 1,
+                });
+                1
+            }
+        };
+        Some(AlarmOfABackgroundTaskStoppedAfterAComputeError {
+            task,
+            gpu_devices_named_by_the_error: stopped.error.gpu_devices_named(),
+            error_that_stopped_the_task: stopped.error,
+            wakes_while_stopped,
+            batches_compensated_on_the_cpu: stopped.batches_compensated_on_the_cpu,
+        })
+    }
+
     /// 补跑那一判此刻的读数：`fullest_device` 是调用方从分配器取的池子此刻最满的那一块盘
     /// （[`AllocatedBytesAndCapacityOfADevice::of_the_fullest_device_of_the_allocator`]），阈值取下一批读的那一组（有待生效的取它，没有取生效的）。
     #[must_use]
@@ -1415,7 +1558,7 @@ impl BackgroundComputeOfThisMount {
                 gc_cpu_compensation_threshold,
             )?;
         self.requested_from_the_next_batch = Some(admitted);
-        self.stopped_after_a_compute_error.clear();
+        self.replace_the_tasks_stopped(Vec::new());
         Ok(())
     }
 
@@ -1429,8 +1572,9 @@ impl BackgroundComputeOfThisMount {
         if let Some(requested) = self.requested_from_the_next_batch.take() {
             self.in_effect = requested;
             self.in_effect.hand_the_gpu_settings_to_the_implementation();
-            self.stopped_after_a_compute_error =
+            let tasks_stopped =
                 tasks_stopped_when_no_gpu_is_usable_under_the_settings_handed_in(&self.in_effect);
+            self.replace_the_tasks_stopped(tasks_stopped);
         }
     }
 
diff --git a/crates/singlefs-core/src/mounted_session.rs b/crates/singlefs-core/src/mounted_session.rs
index 1cd5d57..5eeae8c 100644
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -29,12 +29,12 @@ use crate::address::{
 };
 use crate::allocator::{AllocationRecord, Placement, PlacementOnDevice, PoolAllocator};
 use crate::background_compute::{
-    decisions_of_one_segment, AllocatedBytesAndCapacityOfADevice, BackgroundComputeError,
-    BackgroundComputeOfOneBatch, BackgroundComputeOfThisMount, BackgroundComputeSettings,
-    BackgroundComputeSettingsRefused, BackgroundGcCpuCompensationThreshold, BackgroundGpuCompute,
-    BackgroundGpuComputeHandle, BackgroundTask, GcCpuCompensationReading, GpuComputedSegment,
-    GpuEntryDecision, GpuWorkOfTheBatch, InTheMiddleOfEachBackgroundBatch,
-    WhetherTheTaskRunsTheNextBatch,
+    decisions_of_one_segment, AlarmOfABackgroundTaskStoppedAfterAComputeError,
+    AllocatedBytesAndCapacityOfADevice, BackgroundComputeError, BackgroundComputeOfOneBatch,
+    BackgroundComputeOfThisMount, BackgroundComputeSettings, BackgroundComputeSettingsRefused,
+    BackgroundGcCpuCompensationThreshold, BackgroundGpuCompute, BackgroundGpuComputeHandle,
+    BackgroundTask, GcCpuCompensationReading, GpuComputedSegment, GpuEntryDecision,
+    GpuWorkOfTheBatch, InTheMiddleOfEachBackgroundBatch, WhetherTheTaskRunsTheNextBatch,
 };
 use crate::block_device::{
     check_aligned_and_in_range, BlockDevice, BlockDeviceError, DiscardOutcome,
@@ -217,6 +217,8 @@ pub struct DiscardsAfterThePublishes {
 }
 
 /// 一次整理运行（显式入口 [`MountedSession::run_compaction`]，或后台入口 [`MountedSession::run_compaction_in_the_background`] 起跑的那一趟）做成了什么。
+/// 带着整理停着时这一次叫起的报警（[`Self::alarm_of_the_compaction_stopped_after_a_compute_error`]）：调用方要处理，整份丢掉编译就警告。
+#[must_use = "整理因 GPU 出错停着时这一趟带着报警，要告诉文件系统的用户（用户 2026-10-30 定）"]
 #[derive(Debug)]
 pub struct CompactionRun {
     /// 这一趟发成的每一版，按先后：先走空的删除续做、再是整理的各批。最后一个就是会话的 `current`。
@@ -232,6 +234,12 @@ pub struct CompactionRun {
     /// `.claude/rules/fs-design.md` 五条硬要求第 4 条）。删除续做那几步不是整理的批，不在这里；整理停着、占用不超过补跑阈值而没开成的那一批也不在
     /// （一个都没算，用户 2026-10-28 定案）。
     pub background_compute_of_each_batch: Vec<BackgroundComputeOfOneBatch>,
+    /// 这一趟结束时整理因一批出错停着（D26（后台整理与放置回收） 已定项 13 第 4 条；用户 2026-10-30 定「之后每次都要报警」）：这一次叫起的报警，
+    /// 不论这一趟补跑了几批还是照旧停着、此刻 GPU 用不用得上；这一趟里出错停下的也报（叫起次数 1）。没停着是 `None`。
+    /// 显式入口与后台入口起跑的那一趟填它（[`MountedSession::run_compaction`]、[`MountedSession::run_compaction_in_the_background`]）；
+    /// 可写挂载续做那一趟不是叫起，恒是 `None`。
+    pub alarm_of_the_compaction_stopped_after_a_compute_error:
+        Option<AlarmOfABackgroundTaskStoppedAfterAComputeError>,
 }
 
 /// 一次整理运行停在哪：照判据停（[`CompactionRunStop`]），或一次发布没发成。
@@ -262,11 +270,18 @@ pub enum CompactionRunRefused {
     NoVersionOfTrees,
 }
 
-/// 后台入口一次调用做成了什么（[`MountedSession::run_compaction_in_the_background`]）。
+/// 后台入口一次调用做成了什么（[`MountedSession::run_compaction_in_the_background`]），连同整理停着时这一次叫起的报警
+/// （[`Self::alarm_of_the_compaction_stopped_after_a_compute_error`]）：调用方要处理，整份丢掉编译就警告。
+#[must_use = "整理因 GPU 出错停着时后台入口每一次都带着报警，要告诉文件系统的用户（用户 2026-10-30 定）"]
 #[derive(Debug)]
 pub enum BackgroundCompactionCall {
-    /// 没起跑，一个写都没发。
-    NotStarted(BackgroundCompactionNotStarted),
+    /// 没起跑，一个写都没发。`alarm_of_the_compaction_stopped_after_a_compute_error` 同 [`CompactionRun::alarm_of_the_compaction_stopped_after_a_compute_error`]：
+    /// 整理停着就有（停着而占用不超过补跑阈值没起跑的、超过阈值而这一次没到起跑的都有）。
+    NotStarted {
+        reason: BackgroundCompactionNotStarted,
+        alarm_of_the_compaction_stopped_after_a_compute_error:
+            Option<AlarmOfABackgroundTaskStoppedAfterAComputeError>,
+    },
     /// 起跑了一趟：为什么起跑、那一趟停在哪、发了哪几版。装箱：不装箱这一个成员比另一个大两百多字节（clippy `large_enum_variant`）。
     Ran {
         trigger: BackgroundCompactionTrigger,
@@ -274,6 +289,43 @@ pub enum BackgroundCompactionCall {
     },
 }
 
+impl BackgroundCompactionCall {
+    /// 整理停着时这一次叫起的报警：没起跑的在成员里，起跑了的在那一趟里（[`CompactionRun::alarm_of_the_compaction_stopped_after_a_compute_error`]）。
+    #[must_use]
+    pub fn alarm_of_the_compaction_stopped_after_a_compute_error(
+        &self,
+    ) -> Option<&AlarmOfABackgroundTaskStoppedAfterAComputeError> {
+        match self {
+            BackgroundCompactionCall::NotStarted {
+                alarm_of_the_compaction_stopped_after_a_compute_error,
+                ..
+            } => alarm_of_the_compaction_stopped_after_a_compute_error.as_ref(),
+            BackgroundCompactionCall::Ran { run, .. } => run
+                .alarm_of_the_compaction_stopped_after_a_compute_error
+                .as_ref(),
+        }
+    }
+
+    /// 这一次叫起结束时的报警（`alarm`）装进去：没起跑的装进成员，起跑了的装进那一趟。
+    fn with_the_alarm_at_the_end_of_the_wake(
+        self,
+        alarm: Option<AlarmOfABackgroundTaskStoppedAfterAComputeError>,
+    ) -> Self {
+        match self {
+            BackgroundCompactionCall::NotStarted { reason, .. } => {
+                BackgroundCompactionCall::NotStarted {
+                    reason,
+                    alarm_of_the_compaction_stopped_after_a_compute_error: alarm,
+                }
+            }
+            BackgroundCompactionCall::Ran { trigger, mut run } => {
+                run.alarm_of_the_compaction_stopped_after_a_compute_error = alarm;
+                BackgroundCompactionCall::Ran { trigger, run }
+            }
+        }
+    }
+}
+
 /// 后台入口这一次为什么没起跑。封闭集合。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum BackgroundCompactionNotStarted {
@@ -2310,9 +2362,21 @@ impl MountedSession {
             parameters,
             current.root(),
         );
+        // 用户 2026-10-30 定：整理停着期间显式入口每一趟都报警（这一趟里出错停下的也报）。
+        run.alarm_of_the_compaction_stopped_after_a_compute_error =
+            background_compute.alarm_at_the_end_of_a_wake(BackgroundTask::CompactionRelocation);
         Ok(run)
     }
 
+    /// 整理停着时这一次叫起的报警（[`BackgroundComputeOfThisMount::alarm_at_the_end_of_a_wake`]），后台入口每一次判过起跑、跑完之后取。
+    fn alarm_of_the_compaction_at_the_end_of_a_wake(
+        &mut self,
+    ) -> Option<AlarmOfABackgroundTaskStoppedAfterAComputeError> {
+        self.parameters_and_device_table
+            .background_compute_of_this_mount_mut()
+            .alarm_at_the_end_of_a_wake(BackgroundTask::CompactionRelocation)
+    }
+
     /// 整理的后台入口（D26（后台整理与放置回收） 已定项 13 第 5、6 条；用户 2026-10-26 定 GC「挪出调用线程，补齐三样保证」）：调用方在它自己的线程上
     /// 一次次调，核心层不起线程、不读钟（钟与前台的 fsync 由 `caller` 答）。每次调用先看起不起跑，依次：
     /// 1. 整理因一批出错停着（第 4 条）而池子每块盘的已分配占它的容量都不超过补跑阈值 ⇒ 不起跑，「该跑了」留着，活意图的 R 排他撤掉（同出错停下那一刻）；
@@ -2337,18 +2401,22 @@ impl MountedSession {
         pacing: CompactionPacing,
         caller: &dyn CallerOfTheBackgroundCompaction,
     ) -> Result<BackgroundCompactionCall, CompactionRunRefused> {
-        match self.whether_the_background_compaction_starts(devices, pacing, caller)? {
-            BackgroundCompactionStart::NotStarted(not_started) => {
-                Ok(BackgroundCompactionCall::NotStarted(not_started))
-            }
+        let call = match self.whether_the_background_compaction_starts(devices, pacing, caller)? {
+            BackgroundCompactionStart::NotStarted(reason) => BackgroundCompactionCall::NotStarted {
+                reason,
+                alarm_of_the_compaction_stopped_after_a_compute_error: None,
+            },
             BackgroundCompactionStart::Starts {
                 trigger,
                 reach,
                 budget,
-            } => Ok(self.run_the_background_compaction_that_starts(
+            } => self.run_the_background_compaction_that_starts(
                 devices, pacing, caller, trigger, reach, budget,
-            )),
-        }
+            ),
+        };
+        // 用户 2026-10-30 定：整理停着期间后台入口每一次判起跑都报警，起没起跑、补没补跑都报。
+        let alarm = self.alarm_of_the_compaction_at_the_end_of_a_wake();
+        Ok(call.with_the_alarm_at_the_end_of_the_wake(alarm))
     }
 
     /// [`Self::run_compaction_in_the_background`] 开头那几判：盘表、有没有树、整理停没停、有没有「该跑了」、定时兜底、上一次用户改动给的那一份
@@ -3190,6 +3258,7 @@ fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockD
             outcome,
             discards: DiscardsAfterThePublishes::default(),
             background_compute_of_each_batch: background_compute_of_each_batch_of_the_run,
+            alarm_of_the_compaction_stopped_after_a_compute_error: None,
         }
     };
     let switches = allocator.compaction().switches;
@@ -5021,7 +5090,9 @@ pub struct UserChangeAfterThePendingChangesRefused {
     pub refusal: Box<UserChangeRefused>,
 }
 
-/// [`MountedSessionWithPendingChanges::run_compaction_in_the_background`] 做成了什么：判过起跑之后先排空的那一次（待发空、没起跑时 `None`）与后台入口这一次。
+/// [`MountedSessionWithPendingChanges::run_compaction_in_the_background`] 做成了什么：判过起跑之后先排空的那一次（待发空、没起跑时 `None`）与后台入口这一次
+/// （整理停着时带着这一次叫起的报警，[`BackgroundCompactionCall::alarm_of_the_compaction_stopped_after_a_compute_error`]）。
+#[must_use = "整理因 GPU 出错停着时后台入口每一次都带着报警，要告诉文件系统的用户（用户 2026-10-30 定）"]
 #[derive(Debug)]
 pub struct BackgroundCompactionAfterThePendingChanges {
     pub pending_changes_published_before_the_run: Option<PublishReportedByTheSession>,
@@ -5494,11 +5565,15 @@ impl MountedSessionWithPendingChanges {
             .whether_the_background_compaction_starts(devices, pacing, caller)
             .map_err(BackgroundCompactionAfterThePendingChangesRefused::CompactionRefused)?
         {
-            BackgroundCompactionStart::NotStarted(not_started) => {
+            BackgroundCompactionStart::NotStarted(reason) => {
+                let alarm = self.session.alarm_of_the_compaction_at_the_end_of_a_wake();
                 return Ok(BackgroundCompactionAfterThePendingChanges {
                     pending_changes_published_before_the_run: None,
-                    call: BackgroundCompactionCall::NotStarted(not_started),
-                })
+                    call: BackgroundCompactionCall::NotStarted {
+                        reason,
+                        alarm_of_the_compaction_stopped_after_a_compute_error: alarm,
+                    },
+                });
             }
             BackgroundCompactionStart::Starts {
                 trigger,
@@ -5532,11 +5607,13 @@ impl MountedSessionWithPendingChanges {
         );
         let has_published = match &call {
             BackgroundCompactionCall::Ran { run, .. } => !run.versions_published.is_empty(),
-            BackgroundCompactionCall::NotStarted(_) => false,
+            BackgroundCompactionCall::NotStarted { .. } => false,
         };
         if has_published {
             self.note_a_publish(now);
         }
+        let alarm = self.session.alarm_of_the_compaction_at_the_end_of_a_wake();
+        let call = call.with_the_alarm_at_the_end_of_the_wake(alarm);
         Ok(BackgroundCompactionAfterThePendingChanges {
             pending_changes_published_before_the_run,
             call,
```

