# m4-gpugc-code-r3：第 59、60 批补丁里产品代码的块（按打进主工作区的次序原样抄录）

## 第 59 批：`~/.local/share/singlefs/work/m4-gpugc-code-r2-fix-writer/patch/crates.patch`

```diff
diff --git a/crates/singlefs-background-gpu/src/background_gpu.rs b/crates/singlefs-background-gpu/src/background_gpu.rs
index dcf1dda..ce5d8d9 100644
--- a/crates/singlefs-background-gpu/src/background_gpu.rs
+++ b/crates/singlefs-background-gpu/src/background_gpu.rs
@@ -9,7 +9,8 @@
 //! 挂着的时候每一批按量出的代价分给参与的卡、按显存上限切段、两份核各算一次、逐条比对。三段都接上了：扫描里的校验和核对（[`BackgroundGpuOverTheCards::verify_unit_checksums`]）、
 //! 区域扫描里的单元头解析与映射反查（[`BackgroundGpuOverTheCards::classify_region_records`]）与选段打分的每段活槽数（[`BackgroundGpuOverTheCards::count_live_slots_per_cluster_segment`]）。
 //! 本机的卡清单（核心层 `GpuInventory`）答的是此刻本机在的独显与各自的显存总量（按总线地址排，挂载那一刻没开成的照列；每次答之前现枚举一次，
-//! 挂载之后掉了的卡不列，切换点它照第 4 条「GPU 不在」拒），连同其中没开成的那几张与原因：挂载与切换照它校验设置，切换可以点开包时没参与而开成了的卡；
+//! 挂载之后掉了的卡不列，切换点它照第 4 条「GPU 不在」拒），连同挂载那一刻没开成的那几张与原因（挂载那一刻定下，答它不现枚举：一次切换只现枚举一次）：
+//! 挂载与切换照它校验设置，切换可以点开包时没参与而开成了的卡；
 //! 点名不在本机的卡就是点名的卡不存在，参与的卡里有没开成的照第 4 条拒。
 //!
 //! 外壳分两层：一张开着的卡要做的几样（[`CardOfTheBackgroundGpu`]：交活之前先等前一次超时的那次提交、重交之前等、交一段两份核）由 [`GpuInUse`]
@@ -532,6 +533,12 @@ struct KernelDispatch<'buffers> {
     outcome_buffer: &'buffers wgpu::Buffer,
 }
 
+/// 一段在同一个只接显存不足的错误范围里建成的缓冲（[`GpuInUse::buffers_created_within_the_free_memory`]）：各段自己的那几样与两份核各自的回读暂存。
+struct BuffersOfAChunk<Buffers> {
+    buffers: Buffers,
+    readbacks: [wgpu::Buffer; 2],
+}
+
 /// 挂载那一刻：枚举适配器、判卡清单与校验、在本机每一张独立显卡上起设备与各段两份核、量一次提交的主机 CPU 时间（同
 /// [`open_background_gpu_with_the_memory_leaked_in_this_process`]，进程里还一张卡都没泄漏过）。
 ///
@@ -923,23 +930,38 @@ impl GpuInUse {
             })
     }
 
-    /// 在只接显存不足的错误范围里建这一段的缓冲（`create_the_buffers`），这一段的账（`bytes_held_by_this_chunk`）调用方已经记下：
-    /// 建得成交回缓冲；建不成（显存不足，第 3 条「卡上此刻空闲的不到这个数」）就放掉这一段的账、交回
-    /// [`GpuFailureCause::OutOfMemoryCreatingTheBuffers`]，一次都不交。等 wgpu 交回那个错误范围也在等的上限之内。
+    /// 两份核各自的回读暂存（`outcome_bytes` 字节，映射来读、接结果缓冲的拷贝），按派发的次序。
+    fn readbacks_of_both_kernels(&self, outcome_bytes: u64) -> [wgpu::Buffer; 2] {
+        std::array::from_fn(|_kernel| {
+            self.device.create_buffer(&wgpu::BufferDescriptor {
+                label: None,
+                size: outcome_bytes,
+                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
+                mapped_at_creation: false,
+            })
+        })
+    }
+
+    /// 在只接显存不足的错误范围里建这一段的缓冲（`create_the_buffers`）与两份核各自的回读暂存（`outcome_bytes` 字节；它们算在这一段的占用里，
+    /// 判决 m4-gpugc-code-r2 第三节第 5 条：建在范围之外时它们显存不足报成没接住的错、这一批出错停下，不走「这一批转给别的卡」），
+    /// 这一段的账（`bytes_held_by_this_chunk`）调用方已经记下：建得成交回缓冲与回读暂存；建不成（显存不足，第 3 条「卡上此刻空闲的不到这个数」）
+    /// 就放掉这一段的账、交回 [`GpuFailureCause::OutOfMemoryCreatingTheBuffers`]，一次都不交。等 wgpu 交回那个错误范围也在等的上限之内。
     fn buffers_created_within_the_free_memory<Buffers>(
         &mut self,
         bytes_held_by_this_chunk: u64,
+        outcome_bytes: u64,
         create_the_buffers: impl FnOnce(&Self) -> Buffers,
-    ) -> Result<Buffers, GpuFailureCause> {
+    ) -> Result<BuffersOfAChunk<Buffers>, GpuFailureCause> {
         let out_of_memory_scope = self.device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
         let buffers = create_the_buffers(self);
+        let readbacks = self.readbacks_of_both_kernels(outcome_bytes);
         let captured = wait_for_the_wgpu_future(
             out_of_memory_scope.pop(),
             self.budget.wait_limit(),
             &SteadyClockOfThisProcess::starting_now(),
         );
         let failure = match captured {
-            Ok(None) => return Ok(buffers),
+            Ok(None) => return Ok(BuffersOfAChunk { buffers, readbacks }),
             Ok(Some(out_of_memory)) => GpuFailureCause::OutOfMemoryCreatingTheBuffers {
                 bytes_held_by_the_chunk: bytes_held_by_this_chunk,
                 reason: format!("{out_of_memory}"),
@@ -947,18 +969,21 @@ impl GpuInUse {
             Err(cause) => cause,
         };
         drop(buffers);
+        drop(readbacks);
         self.budget
             .memory_account_mut()
             .release(bytes_held_by_this_chunk);
         Err(failure)
     }
 
-    /// 两份核在同一次提交里各算一遍、各自的结果缓冲拷进各自的回读暂存，等做完、等两份映射好，交回两份读回的字节（按派发的次序），
+    /// 两份核在同一次提交里各算一遍、各自的结果缓冲拷进各自的回读暂存（`readbacks`，按派发的次序，与别的缓冲在同一个错误范围里建好的），
+    /// 等做完、等两份映射好，交回两份读回的字节（按派发的次序），
     /// 连同这次提交等完之后在卡上还占不占显存（没等到它做完的，调用方照它把这一段的账留着）。
     /// 一次提交里的几处等共用一个期限，合起来不超过此刻设置给这张卡的等的上限。
     fn dispatch_both_kernels_and_read_back(
         &self,
         kernel_dispatches: [KernelDispatch<'_>; 2],
+        readbacks: &[wgpu::Buffer; 2],
         workgroups: u32,
         outcome_bytes: u64,
     ) -> (
@@ -966,14 +991,6 @@ impl GpuInUse {
         SubmissionOfTheChunk<wgpu::SubmissionIndex>,
     ) {
         let device = &self.device;
-        let readbacks: [wgpu::Buffer; 2] = std::array::from_fn(|_kernel| {
-            device.create_buffer(&wgpu::BufferDescriptor {
-                label: None,
-                size: outcome_bytes,
-                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
-                mapped_at_creation: false,
-            })
-        });
         let bind_groups: Vec<wgpu::BindGroup> = kernel_dispatches
             .iter()
             .map(|kernel_dispatch| {
@@ -1005,7 +1022,7 @@ impl GpuInUse {
                 pass.dispatch_workgroups(workgroups, 1, 1);
             }
         }
-        for (kernel_dispatch, readback) in kernel_dispatches.iter().zip(&readbacks) {
+        for (kernel_dispatch, readback) in kernel_dispatches.iter().zip(readbacks) {
             encoder.copy_buffer_to_buffer(
                 kernel_dispatch.outcome_buffer,
                 0,
@@ -1016,7 +1033,7 @@ impl GpuInUse {
         }
         let submission = self.queue.submit(std::iter::once(encoder.finish()));
         let (mapping_sender, mapping_receiver) = std::sync::mpsc::channel();
-        for readback in &readbacks {
+        for readback in readbacks {
             let sender_for_this_readback = mapping_sender.clone();
             readback
                 .slice(..)
@@ -1036,7 +1053,7 @@ impl GpuInUse {
             SubmissionOfTheChunk::after_waiting_for_its_completion(submission, &polled);
         let read_back = self.both_readbacks_once_the_submission_completed(
             polled,
-            &readbacks,
+            readbacks,
             &mapping_receiver,
             &deadline,
             &clock,
@@ -1175,8 +1192,14 @@ impl CardOfTheBackgroundGpu for GpuInUse {
         self.budget
             .memory_account_mut()
             .hold(bytes_held_by_this_chunk);
-        let [parameter_buffer, input_buffer, descriptor_buffer, table_driven_outcomes, bitwise_outcomes] =
-            self.buffers_created_within_the_free_memory(bytes_held_by_this_chunk, |gpu| {
+        let BuffersOfAChunk {
+            buffers:
+                [parameter_buffer, input_buffer, descriptor_buffer, table_driven_outcomes, bitwise_outcomes],
+            readbacks,
+        } = self.buffers_created_within_the_free_memory(
+            bytes_held_by_this_chunk,
+            outcome_bytes,
+            |gpu| {
                 [
                     gpu.buffer_with_contents(&parameter_bytes, wgpu::BufferUsages::UNIFORM),
                     gpu.buffer_with_contents(&packed.input_bytes, wgpu::BufferUsages::STORAGE),
@@ -1184,7 +1207,8 @@ impl CardOfTheBackgroundGpu for GpuInUse {
                     gpu.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE),
                     gpu.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE),
                 ]
-            })?;
+            },
+        )?;
         let (read_back, submission_of_the_chunk) = self.dispatch_both_kernels_and_read_back(
             [
                 KernelDispatch {
@@ -1208,6 +1232,7 @@ impl CardOfTheBackgroundGpu for GpuInUse {
                     outcome_buffer: &bitwise_outcomes,
                 },
             ],
+            &readbacks,
             entry_count_in_words.div_ceil(INVOCATIONS_PER_WORKGROUP),
             outcome_bytes,
         );
@@ -1287,8 +1312,14 @@ impl CardOfTheBackgroundGpu for GpuInUse {
         self.budget
             .memory_account_mut()
             .hold(bytes_held_by_this_chunk);
-        let [parameter_buffer, input_buffer, descriptor_buffer, entry_buffer, location_item_buffer, key_sorted_position_buffer, walking_outcomes, searching_outcomes] =
-            self.buffers_created_within_the_free_memory(bytes_held_by_this_chunk, |gpu| {
+        let BuffersOfAChunk {
+            buffers:
+                [parameter_buffer, input_buffer, descriptor_buffer, entry_buffer, location_item_buffer, key_sorted_position_buffer, walking_outcomes, searching_outcomes],
+            readbacks,
+        } = self.buffers_created_within_the_free_memory(
+            bytes_held_by_this_chunk,
+            outcome_bytes,
+            |gpu| {
                 [
                     gpu.buffer_with_contents(&parameter_bytes, wgpu::BufferUsages::UNIFORM),
                     gpu.buffer_with_contents(&packed.input_bytes, wgpu::BufferUsages::STORAGE),
@@ -1302,7 +1333,8 @@ impl CardOfTheBackgroundGpu for GpuInUse {
                     gpu.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE),
                     gpu.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE),
                 ]
-            })?;
+            },
+        )?;
         let (read_back, submission_of_the_chunk) = self.dispatch_both_kernels_and_read_back(
             [
                 KernelDispatch {
@@ -1330,6 +1362,7 @@ impl CardOfTheBackgroundGpu for GpuInUse {
                     outcome_buffer: &searching_outcomes,
                 },
             ],
+            &readbacks,
             record_count.div_ceil(REGION_RECORD_INVOCATIONS_PER_WORKGROUP),
             outcome_bytes,
         );
@@ -1404,8 +1437,14 @@ impl CardOfTheBackgroundGpu for GpuInUse {
         self.budget
             .memory_account_mut()
             .hold(bytes_held_by_this_chunk);
-        let [parameter_buffer, segment_bound_buffer, record_buffer, sorted_position_buffer, walking_outcomes, searching_outcomes] =
-            self.buffers_created_within_the_free_memory(bytes_held_by_this_chunk, |gpu| {
+        let BuffersOfAChunk {
+            buffers:
+                [parameter_buffer, segment_bound_buffer, record_buffer, sorted_position_buffer, walking_outcomes, searching_outcomes],
+            readbacks,
+        } = self.buffers_created_within_the_free_memory(
+            bytes_held_by_this_chunk,
+            outcome_bytes,
+            |gpu| {
                 [
                     gpu.buffer_with_contents(&parameter_bytes, wgpu::BufferUsages::UNIFORM),
                     gpu.buffer_with_contents(&segment_bound_bytes, wgpu::BufferUsages::STORAGE),
@@ -1414,7 +1453,8 @@ impl CardOfTheBackgroundGpu for GpuInUse {
                     gpu.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE),
                     gpu.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE),
                 ]
-            })?;
+            },
+        )?;
         let (read_back, submission_of_the_chunk) = self.dispatch_both_kernels_and_read_back(
             [
                 KernelDispatch {
@@ -1439,6 +1479,7 @@ impl CardOfTheBackgroundGpu for GpuInUse {
                     outcome_buffer: &searching_outcomes,
                 },
             ],
+            &readbacks,
             segment_count.div_ceil(LIVE_SLOT_INVOCATIONS_PER_WORKGROUP),
             outcome_bytes,
         );
@@ -2161,6 +2202,7 @@ impl<Card: CardOfTheBackgroundGpu> Drop for BackgroundGpuOverTheCards<Card> {
             |card| GpuLeakedWhenDropped {
                 device: card.discrete_gpu().identifier.clone(),
                 bytes_held_on_the_device: card.budget().memory_account().bytes_held(),
+                bytes_leaked_by_earlier_mounts: card.budget().bytes_leaked_by_earlier_mounts(),
             },
         );
         self.memory_leaked_in_this_process
@@ -2171,7 +2213,9 @@ impl<Card: CardOfTheBackgroundGpu> Drop for BackgroundGpuOverTheCards<Card> {
 
 /// 卡清单（核心层挂载与切换照它校验设置，D26（后台整理与放置回收） 已定项 13 第 2、4 条）：此刻本机在的独显与各自的显存总量，按总线地址排
 /// （挂载那一刻没开成的照列；没给 `background_gpu_devices` 时上限按这个次序一张卡一个），连同没开成的那几张与原因（[`DiscreteGpusAfterTheOpening`]）。
-/// 每次答之前现查本机在的独显（[`DiscreteGpusPresentNow`]）：挂载之后掉了的卡不列，切换点它照「点名的卡不存在」拒、不点名时它不参与、一张都不剩照「GPU 不在」拒。
+/// 答卡清单之前现查本机在的独显（[`DiscreteGpusPresentNow`]）：挂载之后掉了的卡不列，切换点它照「点名的卡不存在」拒、不点名时它不参与、一张都不剩照「GPU 不在」拒。
+/// 没开成的那几张是挂载那一刻定下的，答它不现查（判决 m4-gpugc-code-r2 第三节第 1 条：一次切换只现枚举一次，判与取参与的卡读同一份；
+/// 两样各现查一次时，第二次枚举答空就把点名一张没开成的卡的切换收下了），掉了的没开成的卡照列，它不在卡清单里、不参与。
 /// 切换可以点开包时没参与而开成了的卡；参与的卡里有没开成的照第 4 条拒。
 impl<Card: CardOfTheBackgroundGpu> GpuInventory for BackgroundGpuOverTheCards<Card> {
     fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
@@ -2186,11 +2230,6 @@ impl<Card: CardOfTheBackgroundGpu> GpuInventory for BackgroundGpuOverTheCards<Ca
 
     fn devices_not_opened_at_the_mount(&self) -> Vec<GpuDeviceNotOpenedAtTheMount> {
         self.discrete_gpus_after_the_opening
-            .still_present(
-                &self
-                    .discrete_gpus_present_now
-                    .identifiers_of_the_discrete_gpus_present_now(),
-            )
             .devices_not_opened_at_the_mount()
     }
 }
@@ -3139,6 +3178,7 @@ mod tests {
             Some(vec![GpuLeakedWhenDropped {
                 device: card_identifier("0000:01:00.0"),
                 bytes_held_on_the_device: 600,
+                bytes_leaked_by_earlier_mounts: 0,
             }]),
             "只泄漏还在跑的那一张"
         );
@@ -3192,4 +3232,210 @@ mod tests {
             "卡掉了、点名它：点名的卡不存在"
         );
     }
+
+    /// 切换那一刻现查的「此刻本机在的独显」：头一次枚举交回 `present_while_nothing_has_enumerated_yet`，之后的枚举一张都没有
+    /// （等 wgpu 超过上限当一张都不在，或两次之间掉了卡）；枚举了几次记在 `enumerations`。
+    struct DiscreteGpusPresentOnlyUntilTheEnumerationIsRepeated {
+        enumerations: Arc<std::sync::atomic::AtomicUsize>,
+        present_while_nothing_has_enumerated_yet: Vec<GpuDeviceIdentifier>,
+    }
+
+    impl DiscreteGpusPresentNow for DiscreteGpusPresentOnlyUntilTheEnumerationIsRepeated {
+        fn identifiers_of_the_discrete_gpus_present_now(&self) -> Vec<GpuDeviceIdentifier> {
+            if self
+                .enumerations
+                .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
+                == 0
+            {
+                self.present_while_nothing_has_enumerated_yet.clone()
+            } else {
+                Vec::new()
+            }
+        }
+    }
+
+    /// 判决 m4-gpugc-code-r2 第三节第 1 条：本机两张独显，0000:41:00.0 挂载那一刻没开成；切换点名它，现查本机在的独显头一次答两张都在、之后答一张都没有——
+    /// 一次切换只现枚举一次（判与取参与的卡读同一份），照第 4 条拒成「参与的卡里有挂载那一刻没开成的」，不收下、不交给实现（交了就在外壳里 panic）。
+    #[test]
+    fn a_switch_naming_a_card_not_opened_at_the_mount_is_refused_on_a_single_enumeration_though_a_repeated_one_would_come_back_empty(
+    ) {
+        let opened_card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
+        let opened = opened_card.discrete_gpu.clone();
+        let not_opened = DiscreteGpuNotOpenedAtTheMount {
+            discrete_gpu: DiscreteGpuOnThisHost {
+                position_in_the_enumeration: 1,
+                identifier: card_identifier("0000:41:00.0"),
+                pci_bus_address: "0000:41:00.0".to_string(),
+                name: "cpu card that did not open at the mount".to_string(),
+                total_memory: GpuMemoryInBytes(EIGHT_GIBIBYTES),
+            },
+            cause: GpuFailureCause::DeviceOrPipelineDidNotStart {
+                reason: "替身：挂载那一刻没开成".to_string(),
+            },
+        };
+        let enumerations = Arc::new(std::sync::atomic::AtomicUsize::new(0));
+        let background_gpu = BackgroundGpuOverTheCards::of_the_cards_opened_at_the_mount(
+            vec![opened_card],
+            DiscreteGpusAfterTheOpening {
+                opened: vec![opened],
+                not_opened: vec![not_opened],
+            },
+            Vec::new(),
+            ParticipatingGpusOnThisHost::of_the_positions(vec![0]),
+            Vec::new(),
+            GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+            Box::new(DiscreteGpusPresentOnlyUntilTheEnumerationIsRepeated {
+                enumerations: Arc::clone(&enumerations),
+                present_while_nothing_has_enumerated_yet: vec![
+                    card_identifier("0000:01:00.0"),
+                    card_identifier("0000:41:00.0"),
+                ],
+            }),
+        );
+        let admitted = admit_gpu_settings_against_the_inventory(
+            &BackgroundComputeSettings {
+                background_compute: BackgroundComputeBackend::Gpu,
+                background_gpu_devices: Some(vec![card_identifier("0000:41:00.0")]),
+                background_gpu_memory_limit: None,
+                background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
+            },
+            &background_gpu,
+        );
+        assert!(
+            matches!(
+                &admitted,
+                Err(BackgroundComputeSettingsRefused::ParticipatingGpuDeviceNotOpenedAtTheMount(
+                    not_opened_at_the_mount
+                )) if not_opened_at_the_mount.device == card_identifier("0000:41:00.0")
+            ),
+            "点名挂载那一刻没开成的卡：拒（{admitted:?}）"
+        );
+        assert_eq!(
+            enumerations.load(std::sync::atomic::Ordering::SeqCst),
+            1,
+            "一次切换只现枚举一次"
+        );
+    }
+
+    /// 判决第三节第 2 条在外壳上：校验和那一段一条 1000 字节的单元，头一次交等超过上限，重交之前等前一次那一等又超时 2 回、第 3 回等到再交、做完——
+    /// 这一段照常交回、与 CPU 参照相同，这张卡上交了 2 次，这一段记下重交 1 次（只数真交出去的）。
+    #[test]
+    fn a_checksum_batch_whose_wait_before_the_resubmission_times_out_twice_reports_one_resubmission(
+    ) {
+        let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
+        card.submissions_timing_out = VecDeque::from([true]);
+        card.earlier_submission_waits_timing_out = VecDeque::from([true, true]);
+        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
+            vec![card],
+            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+        );
+        let unit_bytes = unit_bytes_of(1, 1000);
+        let units = units_to_verify(&unit_bytes);
+        assert_eq!(
+            background_gpu.both_kernel_outcomes_of_the_batch(&units),
+            Ok(cpu_reference_outcomes_of(&units)),
+            "超时 3 次之后做完：照常交回"
+        );
+        let submissions = background_gpu
+            .take_the_submissions_of_the_batch()
+            .unit_checksum_verification;
+        assert_eq!(
+            (
+                background_gpu.cards_on_this_host[0].submissions_made,
+                submissions.resubmissions_after_timeouts
+            ),
+            (2, 1),
+            "这张卡上交了 2 次，重交只数真交出去的那 1 次（{submissions:?}）"
+        );
+    }
+
+    /// 判决第三节第 4 条在外壳上：同一进程里两次挂载同时开着、都在同一张卡上各有一次等超过上限还在跑的提交（先开的 600 字节、后开的 300 字节，
+    /// 开卡那一刻都没算上对方的），两次都丢掉——进程里这张卡记 900（按次累加），新一次挂载这张卡的账从 900 起。
+    #[test]
+    fn two_mounts_alive_at_once_leaking_on_one_gpu_add_up_through_the_drops_of_the_background_gpu()
+    {
+        let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
+        let card = card_identifier("0000:01:00.0");
+        let (mount_leaking_six_hundred_bytes, _present_of_the_mount_leaking_six_hundred_bytes) =
+            background_gpu_over_cards_on_the_cpu(
+                vec![CardComputedOnTheCpuForTests::of_the_address(
+                    "0000:01:00.0",
+                    None,
+                    memory_leaked.bytes_leaked_on(&card),
+                )
+                .with_a_submission_running_past_its_wait(600)],
+                &memory_leaked,
+            );
+        let (mount_leaking_three_hundred_bytes, _present_of_the_mount_leaking_three_hundred_bytes) =
+            background_gpu_over_cards_on_the_cpu(
+                vec![CardComputedOnTheCpuForTests::of_the_address(
+                    "0000:01:00.0",
+                    None,
+                    memory_leaked.bytes_leaked_on(&card),
+                )
+                .with_a_submission_running_past_its_wait(300)],
+                &memory_leaked,
+            );
+        drop(mount_leaking_six_hundred_bytes);
+        drop(mount_leaking_three_hundred_bytes);
+        assert_eq!(
+            memory_leaked.bytes_leaked_on(&card),
+            900,
+            "两次各泄漏 600、300 字节：记 900，不让后丢的盖掉先丢的"
+        );
+        assert_eq!(
+            CardComputedOnTheCpuForTests::of_the_address(
+                "0000:01:00.0",
+                Some(1 << 20),
+                memory_leaked.bytes_leaked_on(&card),
+            )
+            .budget
+            .memory_account()
+            .bytes_held(),
+            900,
+            "新一次挂载这张卡的账从 900 起"
+        );
+    }
+
+    /// 同一条：一次挂载在一张卡上泄漏 600 字节、丢掉；同一张卡新一次挂载开卡那一刻账从 600 起，再有一次等超过上限还在跑的提交（300 字节），
+    /// 丢掉时那一刻占着 900——进程里只加这一次自己的 300，记 900，不把开卡时已算上的 600 再加一遍。
+    #[test]
+    fn a_later_mount_leaking_again_on_the_same_gpu_adds_only_its_own_bytes_in_the_process() {
+        let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
+        let card = card_identifier("0000:01:00.0");
+        let (earlier_mount, _earlier_present) = background_gpu_over_cards_on_the_cpu(
+            vec![
+                CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0)
+                    .with_a_submission_running_past_its_wait(600),
+            ],
+            &memory_leaked,
+        );
+        drop(earlier_mount);
+        assert_eq!(memory_leaked.bytes_leaked_on(&card), 600);
+        let (later_mount, _later_present) = background_gpu_over_cards_on_the_cpu(
+            vec![CardComputedOnTheCpuForTests::of_the_address(
+                "0000:01:00.0",
+                None,
+                memory_leaked.bytes_leaked_on(&card),
+            )
+            .with_a_submission_running_past_its_wait(300)],
+            &memory_leaked,
+        );
+        let gpus_leaked_by_the_later_mount = later_mount.gpus_leaked_when_dropped();
+        drop(later_mount);
+        assert_eq!(
+            gpus_leaked_by_the_later_mount.gpus_leaked(),
+            Some(vec![GpuLeakedWhenDropped {
+                device: card.clone(),
+                bytes_held_on_the_device: 900,
+                bytes_leaked_by_earlier_mounts: 600,
+            }]),
+            "后一次挂载丢掉那一刻占着 900，其中 600 是开卡时已算上的"
+        );
+        assert_eq!(
+            memory_leaked.bytes_leaked_on(&card),
+            900,
+            "进程里只加这一次自己的 300"
+        );
+    }
 }
diff --git a/crates/singlefs-background-gpu/src/gpus_of_this_host.rs b/crates/singlefs-background-gpu/src/gpus_of_this_host.rs
index f79e4f8..8117f2e 100644
--- a/crates/singlefs-background-gpu/src/gpus_of_this_host.rs
+++ b/crates/singlefs-background-gpu/src/gpus_of_this_host.rs
@@ -51,6 +51,8 @@ pub struct GpuBudgetUnderTheSettings<Submission> {
     largest_entry_count_per_dispatch: u64,
     memory_account: GpuMemoryAccount<Submission>,
     wait_limit: Duration,
+    /// 开卡那一刻这个进程里此前几次挂载泄漏在这张卡上的字节（账从它起）：丢的时候扣掉它，算这一次挂载自己泄漏的（[`GpuLeakedWhenDropped`]）。
+    bytes_leaked_by_earlier_mounts: u64,
 }
 
 impl<Submission> GpuBudgetUnderTheSettings<Submission> {
@@ -89,6 +91,7 @@ impl<Submission> GpuBudgetUnderTheSettings<Submission> {
                 bytes_leaked_by_earlier_mounts,
             ),
             wait_limit,
+            bytes_leaked_by_earlier_mounts,
         }
     }
 
@@ -125,6 +128,12 @@ impl<Submission> GpuBudgetUnderTheSettings<Submission> {
         self.wait_limit
     }
 
+    /// 开卡那一刻已经算上的、这个进程里此前几次挂载泄漏在这张卡上的字节。
+    #[must_use]
+    pub fn bytes_leaked_by_earlier_mounts(&self) -> u64 {
+        self.bytes_leaked_by_earlier_mounts
+    }
+
     #[must_use]
     pub fn memory_account(&self) -> &GpuMemoryAccount<Submission> {
         &self.memory_account
@@ -609,11 +618,27 @@ pub fn plans_of_a_batch_leaving_out_the_gpus_short_of_the_smallest_piece<Plan>(
     }
 }
 
-/// 一张卡丢后台 GPU 时泄漏了（还有一次等超过上限、没在 GPU 上做完的提交，设备不析构）：哪张卡、它上面那一刻本包占着的字节（泄漏的那一片）。
+/// 一张卡丢后台 GPU 时泄漏了（还有一次等超过上限、没在 GPU 上做完的提交，设备不析构）：哪张卡、它上面那一刻本包占着的字节（泄漏的那一片），
+/// 其中开卡那一刻已经算上的、这个进程里此前几次挂载泄漏在它上面的字节（[`GpuBudgetUnderTheSettings::bytes_leaked_by_earlier_mounts`]，
+/// 恒不超过占着的：账从它起、这一次挂载里不放）。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct GpuLeakedWhenDropped {
     pub device: GpuDeviceIdentifier,
     pub bytes_held_on_the_device: u64,
+    pub bytes_leaked_by_earlier_mounts: u64,
+}
+
+impl GpuLeakedWhenDropped {
+    /// 这一次挂载自己泄漏的字节：那一刻占着的扣掉开卡时已算上的此前泄漏的那一片。
+    ///
+    /// # Panics
+    /// 此前泄漏的比那一刻占着的多：账从此前泄漏的那一片起，这一次挂载里一直记着、不放（`GpuMemoryAccount::with_the_bytes_leaked_by_earlier_mounts`）。
+    #[must_use]
+    pub fn bytes_leaked_by_this_mount(&self) -> u64 {
+        self.bytes_held_on_the_device
+            .checked_sub(self.bytes_leaked_by_earlier_mounts)
+            .expect("账从此前泄漏的那一片起、这一次挂载里不放：占着的不比它少")
+    }
 }
 
 /// 一次挂载的后台 GPU 丢掉（卸载、丢掉核心层收的把手）那一刻泄漏了哪几张卡：调用方开卡时拿到一份（`background_gpu_compute::OpenedBackgroundGpuCompute`），
@@ -651,7 +676,8 @@ impl GpusLeakedWhenTheBackgroundGpuIsDropped {
 
 /// 这个进程里每张卡泄漏着的字节（几次挂载的后台 GPU 丢掉时泄漏的设备不析构，那一片显存这个进程活着就一直占着）：调用方在进程里留一份、
 /// 每次开卡都交同一份（`background_gpu_compute::open_background_gpu_compute`），新一次挂载这张卡的可用余量先扣掉它（第 3 条）。
-/// 同一张卡再泄漏一次时记那一刻它上面占着的全部字节（已经含此前泄漏的那一片）。几份把手共用同一份。
+/// 同一张卡上每丢一次泄漏一次就按次累加这一次挂载自己泄漏的字节（[`GpuLeakedWhenDropped::bytes_leaked_by_this_mount`]），不覆盖：
+/// 两次挂载同时开着时各自开卡那一刻都没算上对方的那一片，后丢的那一次也不含先丢的（判决 m4-gpugc-code-r2 第三节第 4 条）。几份把手共用同一份。
 #[derive(Clone, Debug, Default)]
 pub struct GpuMemoryLeakedInThisProcess {
     bytes_leaked_on_each_device: Arc<Mutex<Vec<(GpuDeviceIdentifier, u64)>>>,
@@ -677,7 +703,7 @@ impl GpuMemoryLeakedInThisProcess {
             .map_or(0, |(_device, bytes)| *bytes)
     }
 
-    /// 记下一次丢后台 GPU 时泄漏的卡：每张记它那一刻占着的字节（已含此前泄漏的那一片）。
+    /// 记下一次丢后台 GPU 时泄漏的卡：每张加上这一次挂载自己泄漏的字节（那一刻占着的扣掉开卡时已算上的此前泄漏的那一片）。
     pub fn note_the_gpus_leaked_when_dropped(&self, gpus_leaked: &[GpuLeakedWhenDropped]) {
         let mut bytes_leaked_on_each_device = self
             .bytes_leaked_on_each_device
@@ -688,9 +714,9 @@ impl GpuMemoryLeakedInThisProcess {
                 .iter_mut()
                 .find(|(leaked_device, _bytes)| *leaked_device == leaked.device)
             {
-                Some((_device, bytes)) => *bytes = leaked.bytes_held_on_the_device,
+                Some((_device, bytes)) => *bytes += leaked.bytes_leaked_by_this_mount(),
                 None => bytes_leaked_on_each_device
-                    .push((leaked.device.clone(), leaked.bytes_held_on_the_device)),
+                    .push((leaked.device.clone(), leaked.bytes_leaked_by_this_mount())),
             }
         }
     }
@@ -1526,6 +1552,7 @@ mod tests {
             |gpu| GpuLeakedWhenDropped {
                 device: card(gpu.name),
                 bytes_held_on_the_device: 600,
+                bytes_leaked_by_earlier_mounts: 0,
             },
         );
         assert_eq!(
@@ -1533,6 +1560,7 @@ mod tests {
             vec![GpuLeakedWhenDropped {
                 device: card("还在跑的卡"),
                 bytes_held_on_the_device: 600,
+                bytes_leaked_by_earlier_mounts: 0,
             }],
             "泄漏还在跑的那一张"
         );
@@ -1643,13 +1671,14 @@ mod tests {
     }
 
     /// 判决第三节第 6 条：一次丢后台 GPU 时泄漏了一张卡 600 字节，进程里记下；同一张卡新一次挂载、上限 1000 字节，账从 600 起、
-    /// 可用余量只剩 400：五条各 100 字节切成两条一段（296 字节）的三段，与只剩 400 字节时相同。同一张卡再泄漏一次，记那一刻的全部字节。
+    /// 可用余量只剩 400：五条各 100 字节切成两条一段（296 字节）的三段，与只剩 400 字节时相同。同一张卡这一次挂载再泄漏（那一刻占着 900，其中 600 是开卡时已算上的），加上这一次自己的 300，记 900、不重算已含的那一片。
     #[test]
     fn memory_leaked_by_an_earlier_mount_is_taken_off_the_room_of_the_next_mount_on_that_gpu() {
         let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
         memory_leaked.note_the_gpus_leaked_when_dropped(&[GpuLeakedWhenDropped {
             device: card("0000:01:00.0"),
             bytes_held_on_the_device: 600,
+            bytes_leaked_by_earlier_mounts: 0,
         }]);
         assert_eq!(
             (
@@ -1685,10 +1714,30 @@ mod tests {
         memory_leaked.note_the_gpus_leaked_when_dropped(&[GpuLeakedWhenDropped {
             device: card("0000:01:00.0"),
             bytes_held_on_the_device: 900,
+            bytes_leaked_by_earlier_mounts: 600,
         }]);
         assert_eq!(memory_leaked.bytes_leaked_on(&card("0000:01:00.0")), 900);
     }
 
+    /// 判决 m4-gpugc-code-r2 第三节第 4 条：同一进程里两次挂载同时开着、都在同一张卡上泄漏（开卡那一刻都没算上对方的），先丢的那一次 300 字节、
+    /// 后丢的那一次 600 字节——进程里这张卡按次累加记 900，不让后丢的那一次盖掉先丢的。
+    #[test]
+    fn two_mounts_alive_at_once_leaking_on_one_gpu_add_up_in_the_process() {
+        let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
+        for bytes_leaked_by_the_mount in [300, 600] {
+            memory_leaked.note_the_gpus_leaked_when_dropped(&[GpuLeakedWhenDropped {
+                device: card("0000:01:00.0"),
+                bytes_held_on_the_device: bytes_leaked_by_the_mount,
+                bytes_leaked_by_earlier_mounts: 0,
+            }]);
+        }
+        assert_eq!(
+            memory_leaked.bytes_leaked_on(&card("0000:01:00.0")),
+            900,
+            "两次各 300、600 字节：记 900"
+        );
+    }
+
     /// 同一条在开卡那一刻：一张参与的卡上限 1000 字节、进程里记着它泄漏了 600 字节——余量 400 装不下最小的一片（972 字节）也不拒挂载，
     /// 这张卡照开、不量代价、不参与，报出它的上限、最小的一片与已占的 600 字节；不泄漏时同一个上限照常参与。
     #[test]
@@ -1707,6 +1756,7 @@ mod tests {
         memory_leaked.note_the_gpus_leaked_when_dropped(&[GpuLeakedWhenDropped {
             device: card("0000:01:00.0"),
             bytes_held_on_the_device: 600,
+            bytes_leaked_by_earlier_mounts: 0,
         }]);
         let opening = gpus_opened_at_the_mount(
             discrete_gpus,
diff --git a/crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs b/crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs
index aa178d6..86d13de 100644
--- a/crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs
+++ b/crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs
@@ -134,7 +134,7 @@ fn waited_past_the_limit(cause: &GpuFailureCause) -> bool {
     }
 }
 
-/// 一批里一张卡上到此刻等超过上限了几次、重交了几次（第 4 条「放弃的那一批在同一张卡上重交，至多 3 次，都超时才报警」按批、按卡数：
+/// 一批里一张卡上到此刻等超过上限了几次、重交了几次（重交只数真交出去的：重交之前那一等又等满了上限、没交的那一回只算超时）（第 4 条「放弃的那一批在同一张卡上重交，至多 3 次，都超时才报警」按批、按卡数：
 /// 一批切成几片、几段时各片的超时合起来算，交活之前等前一次超时的那次提交那一等也算）。一批开头每张卡各一份，从 0 起。
 /// 外壳的「一批」是三段各自交给 GPU 的那一次（`BackgroundGpu` 的三段方法各调一次是一批）。
 #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
@@ -207,8 +207,14 @@ pub fn outcome_resubmitted_after_timeouts<Gpu, Outcome>(
             }
             Err(cause) => return Err(cause),
         }
-        timeouts_of_the_batch.resubmissions += 1;
-        attempt = wait_for_the_timed_out_submission(gpu).and_then(|()| submit_once(gpu));
+        // 「重交了几次」只数真交出去的：重交之前那一等又等满了上限的那一回没交（判决 m4-gpugc-code-r2 第三节第 2 条）。
+        attempt = match wait_for_the_timed_out_submission(gpu) {
+            Ok(()) => {
+                timeouts_of_the_batch.resubmissions += 1;
+                submit_once(gpu)
+            }
+            Err(cause) => Err(cause),
+        };
     }
 }
 
@@ -714,6 +720,77 @@ mod tests {
         );
     }
 
+    /// 在一张脚本卡上交一次（套重交），交回结局与这一批这张卡的超时次数、重交次数。
+    fn outcome_and_timeouts_of_one_submission(
+        gpu: &mut GpuWithScriptedSubmissions,
+    ) -> (Result<u32, GpuFailureCause>, TimeoutsOfTheBatchOnOneGpu) {
+        let mut timeouts_of_the_batch = TimeoutsOfTheBatchOnOneGpu::none_yet();
+        let outcome = outcome_resubmitted_after_timeouts(
+            gpu,
+            &mut timeouts_of_the_batch,
+            GpuWithScriptedSubmissions::wait_for_the_timed_out_submission,
+            GpuWithScriptedSubmissions::submit_once,
+        );
+        (outcome, timeouts_of_the_batch)
+    }
+
+    /// 判决 m4-gpugc-code-r2 第三节第 2 条：头一次交超时，重交之前等前一次那一等又超时 2 回、第 3 回等到再交、做完——
+    /// 超时 3 次，真交出去的重交 1 次，「重交了几次」记 1。
+    #[test]
+    fn two_waits_timing_out_before_the_resubmission_count_one_resubmission_actually_made() {
+        let mut gpu = GpuWithScriptedSubmissions::scripted(
+            vec![Err(this_submission_timed_out()), Ok(5)],
+            vec![
+                Err(earlier_submission_still_running()),
+                Err(earlier_submission_still_running()),
+                Ok(()),
+            ],
+        );
+        let (outcome, timeouts_of_the_batch) = outcome_and_timeouts_of_one_submission(&mut gpu);
+        assert_eq!(outcome, Ok(5), "第 3 回等到、重交做完：照常交回");
+        assert_eq!(
+            (
+                gpu.submissions_made,
+                timeouts_of_the_batch.timeouts(),
+                timeouts_of_the_batch.resubmissions()
+            ),
+            (2, 3, 1),
+            "交了 2 次，超时 3 次，真交出去的重交 1 次"
+        );
+    }
+
+    /// 同一条：头一次交超时，之后那一等连着超时 3 回、一次都没重交——第 4 次超时照旧报这张卡用不了，「重交了几次」记 0。
+    #[test]
+    fn three_waits_timing_out_before_any_resubmission_count_no_resubmission_and_report_the_gpu_unusable(
+    ) {
+        let mut gpu = GpuWithScriptedSubmissions::scripted(
+            vec![Err(this_submission_timed_out())],
+            vec![
+                Err(earlier_submission_still_running()),
+                Err(earlier_submission_still_running()),
+                Err(earlier_submission_still_running()),
+            ],
+        );
+        let (outcome, timeouts_of_the_batch) = outcome_and_timeouts_of_one_submission(&mut gpu);
+        assert_eq!(
+            outcome,
+            Err(GpuFailureCause::UnusableAfterResubmissions {
+                resubmissions: 3,
+                last_cause: Box::new(earlier_submission_still_running()),
+            }),
+            "第 4 次超时：这张卡用不了"
+        );
+        assert_eq!(
+            (
+                gpu.submissions_made,
+                timeouts_of_the_batch.timeouts(),
+                timeouts_of_the_batch.resubmissions()
+            ),
+            (1, 4, 0),
+            "只交了头一次，超时 4 次，一次都没真重交"
+        );
+    }
+
     /// 不是等超过上限的出错不重交：当场照原样交回。
     #[test]
     fn a_failure_other_than_a_timeout_is_not_resubmitted() {
diff --git a/crates/singlefs-core/src/background_compute.rs b/crates/singlefs-core/src/background_compute.rs
index 1620917..ccebcdc 100644
--- a/crates/singlefs-core/src/background_compute.rs
+++ b/crates/singlefs-core/src/background_compute.rs
@@ -35,27 +35,19 @@ pub enum BackgroundComputeBackend {
     Gpu,
 }
 
+crate::enum_listing_every_member! {
 /// 计算可以放在别处的后台任务种类（第 1 条「凡有 GPU 路径的后台任务」）。封闭集合：里程碑七的 GPU 打包搬迁
 /// （`.claude/kb/milestone/07-seventh-txn.md`「三　GPU 打包搬迁」）加一个成员，挂载参数、校验与切换都不改。
+/// 每一个成员的表（`BackgroundTask::ALL`，按声明的次序）由 `enum_listing_every_member!` 从这一张成员表生成：加成员不会漏进表
+/// （判决 m4-gpugc-code-r2 第三节第 7 条；`code-discipline.md`「重复要生成，不许手抄」）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum BackgroundTask {
     /// 整理（GC）的搬迁：`mounted_session` 里整理每推一次发布算这一种任务的一批。
     CompactionRelocation,
 }
+}
 
 impl BackgroundTask {
-    /// 每一个成员各一次，按 [`Self::position_in_every_member`] 的次序。加成员时 `position_in_every_member` 与 [`Self::has_a_gpu_path`]
-    /// 的 match 先编不过；补那两臂时把它补进这里，下面编译期那一判核表里第 i 个成员的位置就是 i。
-    pub const EVERY_MEMBER: [BackgroundTask; 1] = [BackgroundTask::CompactionRelocation];
-
-    /// 它在 [`Self::EVERY_MEMBER`] 里的位置。
-    #[must_use]
-    pub const fn position_in_every_member(self) -> usize {
-        match self {
-            BackgroundTask::CompactionRelocation => 0,
-        }
-    }
-
     /// 有没有 GPU 路径（第 1 条「凡有 GPU 路径的后台任务」）：有的那几种，挂载那一刻或切换生效时 GPU 一张卡都用不上就当场停着。
     #[must_use]
     pub const fn has_a_gpu_path(self) -> bool {
@@ -65,15 +57,6 @@ impl BackgroundTask {
     }
 }
 
-// 编译期核 `BackgroundTask::EVERY_MEMBER` 与 `position_in_every_member` 一一对应：表里第 i 个成员的位置就是 i。
-const _: () = {
-    let mut position = 0;
-    while position < BackgroundTask::EVERY_MEMBER.len() {
-        assert!(BackgroundTask::EVERY_MEMBER[position].position_in_every_member() == position);
-        position += 1;
-    }
-};
-
 /// 一张显卡的标识（`background_gpu_devices` 列表里的一项）：不透明的字节，只比相等。卡用什么认、怎么排是里程碑四设计轮（七·2）
 /// 还挂着的岔路，这里不定格式：卡清单（[`GpuInventory`]）交出什么，调用方就点什么。
 #[derive(Clone, Debug, PartialEq, Eq)]
@@ -101,6 +84,9 @@ pub trait GpuInventory {
     /// 给了 `background_gpu_devices` 而没点名的卡开不了，挂载跳过它、照挂）。参与的卡里有它就拒（第 4 条，挂载与切换同一判：
     /// [`BackgroundComputeSettingsRefused::ParticipatingGpuDeviceNotOpenedAtTheMount`]）。
     /// 不开卡的清单（测试里的假清单、开卡之前那一刻的枚举）没有这一样，交回空。
+    /// 挂载那一刻定下、之后不变：答它不现查本机此刻在的卡（一次校验里只有 [`Self::devices_with_total_memory`] 现查一次，判与取参与的卡读的是同一份，
+    /// 判决 m4-gpugc-code-r2 第三节第 1 条），所以可以列着此刻已不在 [`Self::devices_with_total_memory`] 里的卡（挂载之后掉了的）；
+    /// 校验只拿参与的卡去比，掉了的卡不参与。
     fn devices_not_opened_at_the_mount(&self) -> Vec<GpuDeviceNotOpenedAtTheMount> {
         Vec::new()
     }
@@ -1205,9 +1191,11 @@ impl BackgroundComputeError {
 
 /// 后台任务因一批出错停着时，每次叫起都交回的报警（D26（后台整理与放置回收） 已定项 13 第 4 条；用户 2026-10-30 定「GPU 后台任务照旧保留超时重交
 /// 三次失败后要报警  之后每次都要报警，不管后台有没有gpu可以用。一定要告诉fs用户 提醒。」，同日弹窗定「每次」是每次后台整理被叫起、通道是返回值里必须处理的报警）：
-/// 哪个后台任务、停下的那一个错与它点名的卡、停了多久、停着期间在 CPU 上补跑了几批。停了多久不读挂钟，数叫起的次数：从停下那一刻起每次叫起结束时停着就加一
-/// （叫起时停下的那一次算第 1 次；挂载那一刻就停着的，头一次叫起算第 1 次）。这一次叫起在 CPU 上补跑了一批还是照旧停着、此刻 GPU 用不用得上都照报，
-/// 直到一次通过校验的切换解除停下。
+/// 哪个后台任务、停下的那一个错与它点名的卡、停了多久、停着期间在 CPU 上补跑了几批。停了多久不读挂钟，数叫起的次数：停下之后每次叫起结束时停着就加一。
+/// 一批出错停下的那一趟叫起照报、不数这一趟（报 0），下一次叫起才是第 1 次；在叫起里停下与在可写挂载续做里停下（那里不是叫起）数法相同，
+/// 都是停下之后头一次叫起报 1（用户 2026-11-01 定「改代码」：出错停下的那一趟不算，判决 m4-gpugc-code-r2 第三节第 9 条）。
+/// 挂载那一刻就停着的、挂着切换生效时因一张卡都用不上停着的，头一次叫起（切换生效的那一次）算第 1 次。
+/// 这一次叫起在 CPU 上补跑了一批还是照旧停着、此刻 GPU 用不用得上都照报，直到一次通过校验的切换解除停下。
 #[must_use = "后台任务因 GPU 出错停着：每次叫起都要把这份报警告诉文件系统的用户（用户 2026-10-30 定）"]
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct AlarmOfABackgroundTaskStoppedAfterAComputeError {
@@ -1215,7 +1203,7 @@ pub struct AlarmOfABackgroundTaskStoppedAfterAComputeError {
     /// 停下的那一个错点名的卡（[`BackgroundComputeError::gpu_devices_named`]），按错里的次序；不归哪张卡的是空。
     pub gpu_devices_named_by_the_error: Vec<GpuDeviceIdentifier>,
     pub error_that_stopped_the_task: BackgroundComputeError,
-    /// 停下以来叫起了几次（含这一次）。
+    /// 停下以来叫起了几次（含这一次；一批出错停下的那一趟不算，那一趟报 0）。
     pub wakes_while_stopped: u64,
     /// 停下以来在 CPU 上补跑的批数（[`BackgroundTaskStoppedAfterAComputeError::batches_compensated_on_the_cpu`]，含这一次叫起补跑的）。
     pub batches_compensated_on_the_cpu: u64,
@@ -1226,14 +1214,24 @@ pub struct AlarmOfABackgroundTaskStoppedAfterAComputeError {
 struct WakesOfAStoppedTask {
     task: BackgroundTask,
     wakes_while_stopped: u64,
+    /// 它是在此刻还没结束的那一次叫起里一批出错停下的：那一次叫起结束时不数（报 0），之后的叫起照数。
+    is_stopped_in_the_wake_under_way: bool,
+}
+
+/// 此刻有没有一次叫起在做（[`BackgroundComputeOfThisMount::begin_a_wake`] 开、[`BackgroundComputeOfThisMount::alarm_at_the_end_of_a_wake`] 关）：
+/// 一批出错停下时据它分「停在一次叫起里」与「停在可写挂载续做里」（续做不是叫起）。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum WakeOfTheBackgroundTasks {
+    NoneUnderWay,
+    UnderWay,
 }
 
-/// 有 GPU 路径的后台任务各一条停着（[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]，补跑的批数 0），按 [`BackgroundTask::EVERY_MEMBER`]
+/// 有 GPU 路径的后台任务各一条停着（[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]，补跑的批数 0），按 [`BackgroundTask::ALL`]
 /// 的次序、由 [`BackgroundTask::has_a_gpu_path`] 现挑：加成员时不用改这里。
 fn tasks_with_a_gpu_path_stopped_by(
     failure: &BackgroundGpuComputeFailed,
 ) -> Vec<BackgroundTaskStoppedAfterAComputeError> {
-    BackgroundTask::EVERY_MEMBER
+    BackgroundTask::ALL
         .into_iter()
         .filter(|task| task.has_a_gpu_path())
         .map(|task| BackgroundTaskStoppedAfterAComputeError {
@@ -1311,9 +1309,12 @@ pub struct BackgroundComputeOfThisMount {
     in_effect: AdmittedBackgroundComputeSettings,
     requested_from_the_next_batch: Option<AdmittedBackgroundComputeSettings>,
     stopped_after_a_compute_error: Vec<BackgroundTaskStoppedAfterAComputeError>,
-    /// 停着的后台任务停下以来叫起了几次：头一次叫起时记下，停着的那几条换掉或清掉时一起清掉（[`Self::replace_the_tasks_stopped`]）。
+    /// 停着的后台任务停下以来叫起了几次：头一次叫起时记下（一次叫起里一批出错停下的，停下那一刻就记下 0），停着的那几条换掉或清掉时一起清掉
+    /// （[`Self::replace_the_tasks_stopped`]）。
     /// 一种后台任务只经换掉、清掉不再停着，所以这里没有不停着的任务的那一条。
     wakes_of_the_tasks_stopped: Vec<WakesOfAStoppedTask>,
+    /// 此刻有没有一次叫起在做：挂载那一刻与可写挂载续做时没有。
+    wake_under_way: WakeOfTheBackgroundTasks,
     /// 只供测试的钩子在这一批里注入的错，这一批读完它就清掉（[`Self::take_the_compute_error_of_the_batch_under_way`]）。
     compute_error_of_the_batch_under_way: Option<BackgroundComputeError>,
     /// 补跑那一判照什么判：产品恒是照读数判，只供测试的开关能改。
@@ -1337,6 +1338,7 @@ impl BackgroundComputeOfThisMount {
             requested_from_the_next_batch: None,
             stopped_after_a_compute_error,
             wakes_of_the_tasks_stopped: Vec::new(),
+            wake_under_way: WakeOfTheBackgroundTasks::NoneUnderWay,
             compute_error_of_the_batch_under_way: None,
             gc_cpu_compensation_judgement: GcCpuCompensationJudgement::ByTheReading,
         }
@@ -1379,7 +1381,8 @@ impl BackgroundComputeOfThisMount {
     }
 
     /// 一批出错之后（第 4 条）：出错之前就挂着待生效的切换，就当场解除——那一次切换算停着时收到的请求，`task` 不记成停着，
-    /// 下一批开头照常换上它；没有就把 `task` 记成停着，等下一次通过校验的切换请求。不读写盘。
+    /// 下一批开头照常换上它；没有就把 `task` 记成停着，等下一次通过校验的切换请求。停在一次叫起里的（[`Self::begin_a_wake`] 开过、还没结束），
+    /// 那一次叫起结束时报警不数这一趟（[`Self::alarm_at_the_end_of_a_wake`] 报 0）；停在可写挂载续做里的不是叫起，停下之后头一次叫起照数 1。不读写盘。
     pub(crate) fn stop_the_task_after_a_compute_error(
         &mut self,
         task: BackgroundTask,
@@ -1389,16 +1392,38 @@ impl BackgroundComputeOfThisMount {
             return BackgroundTaskAfterTheComputeError::StopLiftedByTheSwitchRequestedBeforeTheError;
         }
         if !self.is_stopped_after_a_compute_error(task) {
+            assert!(
+                self.wakes_of_the_tasks_stopped
+                    .iter()
+                    .all(|wakes| wakes.task != task),
+                "停下以来叫起的次数只记停着的任务：没停着的这一种不该有一条"
+            );
             self.stopped_after_a_compute_error
                 .push(BackgroundTaskStoppedAfterAComputeError {
                     task,
                     error,
                     batches_compensated_on_the_cpu: 0,
                 });
+            match self.wake_under_way {
+                WakeOfTheBackgroundTasks::UnderWay => {
+                    self.wakes_of_the_tasks_stopped.push(WakesOfAStoppedTask {
+                        task,
+                        wakes_while_stopped: 0,
+                        is_stopped_in_the_wake_under_way: true,
+                    });
+                }
+                WakeOfTheBackgroundTasks::NoneUnderWay => {}
+            }
         }
         BackgroundTaskAfterTheComputeError::LeftStopped
     }
 
+    /// 一次叫起开头（整理的显式入口每一趟、后台入口每一次判起跑，入口在任何读写之前就拒的那几判之后）：记下此刻有一次叫起在做，
+    /// 这一次叫起里一批出错停下的，到它结束时不数（[`Self::stop_the_task_after_a_compute_error`]）。不读写盘。
+    pub(crate) fn begin_a_wake(&mut self) {
+        self.wake_under_way = WakeOfTheBackgroundTasks::UnderWay;
+    }
+
     /// 换掉停着的那几条（`tasks_stopped` 可以是空：解除全部停下），停下以来叫起的次数一起清掉：换上的每一条都从这一刻起重新数。
     fn replace_the_tasks_stopped(
         &mut self,
@@ -1408,14 +1433,15 @@ impl BackgroundComputeOfThisMount {
         self.wakes_of_the_tasks_stopped.clear();
     }
 
-    /// 一次叫起结束时（整理的显式入口每一趟、后台入口每一次判起跑，不论起没起跑、补没补跑；入口在任何读写之前就拒的那一次不算）：`task` 此刻停着，
-    /// 就把它停下以来叫起的次数加一、交回这一次的报警（[`AlarmOfABackgroundTaskStoppedAfterAComputeError`]，用户 2026-10-30 定）；没停着交回 `None`。
-    /// 不读写盘。
+    /// 一次叫起结束时（整理的显式入口每一趟、后台入口每一次判起跑，不论起没起跑、补没补跑；入口在任何读写之前就拒的那一次不算）：这一次叫起到此结束；
+    /// `task` 此刻停着，就把它停下以来叫起的次数加一（这一次叫起里一批出错停下的不加，报 0）、交回这一次的报警
+    /// （[`AlarmOfABackgroundTaskStoppedAfterAComputeError`]，用户 2026-10-30 定）；没停着交回 `None`。不读写盘。
     #[must_use = "停着的后台任务每次叫起都要报警：交回的报警要交到调用方手里"]
     pub(crate) fn alarm_at_the_end_of_a_wake(
         &mut self,
         task: BackgroundTask,
     ) -> Option<AlarmOfABackgroundTaskStoppedAfterAComputeError> {
+        self.wake_under_way = WakeOfTheBackgroundTasks::NoneUnderWay;
         let stopped = self
             .stopped_after_a_compute_error
             .iter()
@@ -1426,6 +1452,10 @@ impl BackgroundComputeOfThisMount {
             .iter_mut()
             .find(|wakes| wakes.task == task)
         {
+            Some(wakes) if wakes.is_stopped_in_the_wake_under_way => {
+                wakes.is_stopped_in_the_wake_under_way = false;
+                wakes.wakes_while_stopped
+            }
             Some(wakes) => {
                 wakes.wakes_while_stopped += 1;
                 wakes.wakes_while_stopped
@@ -1434,6 +1464,7 @@ impl BackgroundComputeOfThisMount {
                 self.wakes_of_the_tasks_stopped.push(WakesOfAStoppedTask {
                     task,
                     wakes_while_stopped: 1,
+                    is_stopped_in_the_wake_under_way: false,
                 });
                 1
             }
@@ -1705,7 +1736,7 @@ mod tests {
             }],
         };
         let stopped = tasks_with_a_gpu_path_stopped_by(&failure);
-        for task in BackgroundTask::EVERY_MEMBER {
+        for task in BackgroundTask::ALL {
             let stopped_for_the_task: Vec<&BackgroundTaskStoppedAfterAComputeError> = stopped
                 .iter()
                 .filter(|stopped_task| stopped_task.task == task)
@@ -1726,7 +1757,7 @@ mod tests {
         }
         assert_eq!(
             stopped.len(),
-            BackgroundTask::EVERY_MEMBER
+            BackgroundTask::ALL
                 .into_iter()
                 .filter(|task| task.has_a_gpu_path())
                 .count(),
@@ -1734,6 +1765,93 @@ mod tests {
         );
     }
 
+    /// 停下之后连着叫起三次，交回的报警里停下以来的叫起次数。
+    fn wakes_while_stopped_of_three_wakes(
+        background_compute: &mut BackgroundComputeOfThisMount,
+    ) -> Vec<Option<u64>> {
+        (0..3)
+            .map(|_wake| {
+                background_compute.begin_a_wake();
+                background_compute
+                    .alarm_at_the_end_of_a_wake(BackgroundTask::CompactionRelocation)
+                    .map(|alarm| alarm.wakes_while_stopped)
+            })
+            .collect()
+    }
+
+    /// 判决 m4-gpugc-code-r2 第三节第 9 条（用户 2026-11-01 定「改代码」）：一次叫起里一批出错停下，那一趟照报、叫起次数报 0；
+    /// 之后连着叫起三次报 1、2、3。
+    // harness-test-granularity:one-scenario 同一份状态上按次序叫起几次，数的是停下以来的叫起次数，拆开就数不成
+    #[test]
+    fn a_stop_in_a_wake_is_reported_as_zero_wakes_and_the_next_three_wakes_count_one_two_three() {
+        let mut background_compute =
+            BackgroundComputeOfThisMount::of_a_mount(AdmittedBackgroundComputeSettings::PRODUCT);
+        background_compute.begin_a_wake();
+        assert_eq!(
+            background_compute.stop_the_task_after_a_compute_error(
+                BackgroundTask::CompactionRelocation,
+                BackgroundComputeError::InjectedByTheTestOnlyHook,
+            ),
+            BackgroundTaskAfterTheComputeError::LeftStopped
+        );
+        assert_eq!(
+            background_compute
+                .alarm_at_the_end_of_a_wake(BackgroundTask::CompactionRelocation)
+                .map(|alarm| alarm.wakes_while_stopped),
+            Some(0),
+            "出错停下的那一趟照报、不数这一趟"
+        );
+        assert_eq!(
+            wakes_while_stopped_of_three_wakes(&mut background_compute),
+            vec![Some(1), Some(2), Some(3)],
+            "下一次叫起才是第 1 次"
+        );
+    }
+
+    /// 同一条：停在可写挂载续做里（不是叫起，没开过叫起），停下之后连着叫起三次报 1、2、3，与停在叫起里的数法相同。
+    #[test]
+    fn a_stop_in_the_writable_mount_continuation_counts_the_next_three_wakes_one_two_three() {
+        let mut background_compute =
+            BackgroundComputeOfThisMount::of_a_mount(AdmittedBackgroundComputeSettings::PRODUCT);
+        assert_eq!(
+            background_compute.stop_the_task_after_a_compute_error(
+                BackgroundTask::CompactionRelocation,
+                BackgroundComputeError::InjectedByTheTestOnlyHook,
+            ),
+            BackgroundTaskAfterTheComputeError::LeftStopped
+        );
+        assert_eq!(
+            wakes_while_stopped_of_three_wakes(&mut background_compute),
+            vec![Some(1), Some(2), Some(3)],
+            "续做里停下之后头一次叫起就是第 1 次"
+        );
+    }
+
+    /// 同一条：一次叫起结束（交过报警）之后、下一次叫起开头之前停下的不在叫起里（叫起结束时就关了），停下之后头一次叫起报 1，不报 0。
+    #[test]
+    fn a_stop_after_a_wake_has_ended_is_not_counted_as_stopped_in_that_wake() {
+        let mut background_compute =
+            BackgroundComputeOfThisMount::of_a_mount(AdmittedBackgroundComputeSettings::PRODUCT);
+        background_compute.begin_a_wake();
+        assert_eq!(
+            background_compute.alarm_at_the_end_of_a_wake(BackgroundTask::CompactionRelocation),
+            None,
+            "没停着：不报警"
+        );
+        assert_eq!(
+            background_compute.stop_the_task_after_a_compute_error(
+                BackgroundTask::CompactionRelocation,
+                BackgroundComputeError::InjectedByTheTestOnlyHook,
+            ),
+            BackgroundTaskAfterTheComputeError::LeftStopped
+        );
+        assert_eq!(
+            wakes_while_stopped_of_three_wakes(&mut background_compute),
+            vec![Some(1), Some(2), Some(3)],
+            "叫起结束之后停下的：下一次叫起就是第 1 次"
+        );
+    }
+
     /// 同一段一批里算了两次（选段时逐段先扫）：各数相加，卡照先后并起来、同一张卡不重列。
     #[test]
     fn submissions_of_a_segment_computed_twice_in_a_batch_add_up_and_list_each_device_once() {
diff --git a/crates/singlefs-core/src/mounted_session.rs b/crates/singlefs-core/src/mounted_session.rs
index 5eeae8c..be2cea0 100644
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -2342,6 +2342,8 @@ impl MountedSession {
         }
         let (parameters, background_compute) = parameters_and_device_table
             .parameters_on_disk_and_the_background_compute_of_this_mount_mut();
+        // 过了在任何读写之前的那两判，这一趟就是一次叫起：这一趟里一批出错停下的，这一趟的报警不数它（用户 2026-11-01 定）。
+        background_compute.begin_a_wake();
         let mut run = run_compaction_on_with_the_region_check_of_the_intent_deletion(
             parameters,
             devices.as_mut_slice(),
@@ -2362,7 +2364,7 @@ impl MountedSession {
             parameters,
             current.root(),
         );
-        // 用户 2026-10-30 定：整理停着期间显式入口每一趟都报警（这一趟里出错停下的也报）。
+        // 用户 2026-10-30 定：整理停着期间显式入口每一趟都报警（这一趟里出错停下的也报，叫起次数报 0）。
         run.alarm_of_the_compaction_stopped_after_a_compute_error =
             background_compute.alarm_at_the_end_of_a_wake(BackgroundTask::CompactionRelocation);
         Ok(run)
@@ -2451,6 +2453,10 @@ impl MountedSession {
             PoolVersion::WithFile(_) | PoolVersion::WithTreesAndNoFile(_) => {}
             PoolVersion::WithoutFile(_) => return Err(CompactionRunRefused::NoVersionOfTrees),
         }
+        // 过了在任何读写之前的那两判，这一次判起跑就是一次叫起（结束时取报警）：之后起跑的那一趟里一批出错停下的，这一次的报警不数它（用户 2026-11-01 定）。
+        parameters_and_device_table
+            .background_compute_of_this_mount_mut()
+            .begin_a_wake();
         // 挂着切到一张卡都用不上的那一组：在判停没停之前换上它（两次调用之间不在一批之内），与挂载那一刻一样当场停着，不取走「该跑了」。
         parameters_and_device_table
             .background_compute_of_this_mount_mut()
@@ -5104,8 +5110,14 @@ pub struct BackgroundCompactionAfterThePendingChanges {
 pub enum BackgroundCompactionAfterThePendingChangesRefused {
     /// 后台入口自己在任何读写之前拒（同 [`MountedSession::run_compaction_in_the_background`]）。
     CompactionRefused(CompactionRunRefused),
-    /// 判过起跑、跑之前排空待发没发成：这一趟整理没跑，用户改动之后记下的「该跑了」放回去。
-    PublishOfThePendingChangesRefused(Box<UserChangeRefused>),
+    /// 判过起跑、跑之前排空待发没发成（`refusal` 原样）：这一趟整理没跑，用户改动之后记下的「该跑了」放回去。判过起跑的这一次是一次叫起
+    /// （不是入口在任何读写之前就拒的那一种），整理停着时照样数一次叫起、带着这一次的报警（判决 m4-gpugc-code-r2 第三节第 3 条；
+    /// 同 [`BackgroundCompactionCall::alarm_of_the_compaction_stopped_after_a_compute_error`]），没停着是 `None`。
+    PublishOfThePendingChangesRefused {
+        refusal: Box<UserChangeRefused>,
+        alarm_of_the_compaction_stopped_after_a_compute_error:
+            Option<AlarmOfABackgroundTaskStoppedAfterAComputeError>,
+    },
 }
 
 /// [`MountedSessionWithPendingChanges::into_session_after_publishing_the_pending_changes`] 交不出会话：排空待发没发成，攒批的会话原样交回。
@@ -5595,10 +5607,15 @@ impl MountedSessionWithPendingChanges {
                     }
                     BackgroundCompactionTrigger::TimerFallback => {}
                 }
+                // 用户 2026-10-30 定每次叫起都报：判过起跑之后排空被拒的这一次也是一次叫起，照数、照报。
+                let alarm_of_the_wake_refused_at_the_drain =
+                    self.session.alarm_of_the_compaction_at_the_end_of_a_wake();
                 return Err(
-                    BackgroundCompactionAfterThePendingChangesRefused::PublishOfThePendingChangesRefused(
+                    BackgroundCompactionAfterThePendingChangesRefused::PublishOfThePendingChangesRefused {
                         refusal,
-                    ),
+                        alarm_of_the_compaction_stopped_after_a_compute_error:
+                            alarm_of_the_wake_refused_at_the_drain,
+                    },
                 );
             }
         };
```

## 第 60 批：`~/.local/share/singlefs/work/m4-gpugc-batch60-writer/patch/crates.patch`

```diff
diff --git a/crates/singlefs-background-gpu/src/background_gpu.rs b/crates/singlefs-background-gpu/src/background_gpu.rs
index ce5d8d9..c64fd63 100644
--- a/crates/singlefs-background-gpu/src/background_gpu.rs
+++ b/crates/singlefs-background-gpu/src/background_gpu.rs
@@ -533,10 +533,98 @@ struct KernelDispatch<'buffers> {
     outcome_buffer: &'buffers wgpu::Buffer,
 }
 
-/// 一段在同一个只接显存不足的错误范围里建成的缓冲（[`GpuInUse::buffers_created_within_the_free_memory`]）：各段自己的那几样与两份核各自的回读暂存。
-struct BuffersOfAChunk<Buffers> {
+/// 一段在同一个只接显存不足的错误范围里建成的缓冲（[`buffers_and_both_readbacks_created_within_one_out_of_memory_scope`]）：
+/// 各段自己的那几样与两份核各自的回读暂存（按派发的次序）。
+struct BuffersOfAChunk<Buffers, Readback> {
     buffers: Buffers,
-    readbacks: [wgpu::Buffer; 2],
+    readbacks: [Readback; 2],
+}
+
+/// 建一段缓冲那一刻要设备做的三样：开一个只接显存不足的错误范围、建一份回读暂存、关上那个范围并在等的上限之内取它接住的显存不足。
+/// 产品是 `wgpu::Device`；单测换上在内存里记账的替身（`tests::DeviceOfAChunkInMemoryForTests`）：它照 wgpu 错误范围的规矩
+/// （范围开着时建不成的错归最里层那个范围，没有范围开着时报成没接住的错）判得出回读暂存是不是建在范围里（判决 m4-gpugc-code-r2 第三节第 5 条、
+/// 四之一「造不出就换方法继续造」：真卡上造不出只让回读暂存建不成的条件）。
+trait DeviceCreatingTheBuffersOfAChunk {
+    type Readback;
+    type OutOfMemoryScope;
+
+    fn open_the_out_of_memory_scope(&self) -> Self::OutOfMemoryScope;
+
+    /// 一份核的回读暂存：`outcome_bytes` 字节，映射来读、接结果缓冲的拷贝。
+    fn readback_of_one_kernel(&self, outcome_bytes: u64) -> Self::Readback;
+
+    /// 关上 `scope`，交回它接住的显存不足（一句原因；没接住的是 `None`）。
+    ///
+    /// # Errors
+    /// 等设备交回那个范围超过 `wait_limit`。
+    fn out_of_memory_captured_by(
+        &self,
+        scope: Self::OutOfMemoryScope,
+        wait_limit: Duration,
+    ) -> Result<Option<String>, GpuFailureCause>;
+}
+
+impl DeviceCreatingTheBuffersOfAChunk for wgpu::Device {
+    type Readback = wgpu::Buffer;
+    type OutOfMemoryScope = wgpu::ErrorScopeGuard;
+
+    fn open_the_out_of_memory_scope(&self) -> wgpu::ErrorScopeGuard {
+        self.push_error_scope(wgpu::ErrorFilter::OutOfMemory)
+    }
+
+    fn readback_of_one_kernel(&self, outcome_bytes: u64) -> wgpu::Buffer {
+        self.create_buffer(&wgpu::BufferDescriptor {
+            label: None,
+            size: outcome_bytes,
+            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
+            mapped_at_creation: false,
+        })
+    }
+
+    fn out_of_memory_captured_by(
+        &self,
+        scope: wgpu::ErrorScopeGuard,
+        wait_limit: Duration,
+    ) -> Result<Option<String>, GpuFailureCause> {
+        let captured = wait_for_the_wgpu_future(
+            scope.pop(),
+            wait_limit,
+            &SteadyClockOfThisProcess::starting_now(),
+        )?;
+        Ok(captured.map(|out_of_memory| format!("{out_of_memory}")))
+    }
+}
+
+/// 在同一个只接显存不足的错误范围里建这一段的缓冲（`create_the_buffers`）与两份核各自的回读暂存（`outcome_bytes` 字节）：回读暂存算在这一段的占用里
+/// （`bytes_held_by_this_chunk`），建在范围之外时它显存不足报成没接住的错、这一批出错停下，不走「这一批转给别的卡」（判决 m4-gpugc-code-r2 第三节第 5 条）。
+/// 建得成交回缓冲与回读暂存；范围接住了显存不足就丢掉建出来的、交回 [`GpuFailureCause::OutOfMemoryCreatingTheBuffers`]；
+/// 等设备交回那个范围超过 `wait_limit` 照它的错交回。不动这一段的账（调用方放）。
+///
+/// # Errors
+/// 上面那两种。
+fn buffers_and_both_readbacks_created_within_one_out_of_memory_scope<Device, Buffers>(
+    device: &Device,
+    wait_limit: Duration,
+    bytes_held_by_this_chunk: u64,
+    outcome_bytes: u64,
+    create_the_buffers: impl FnOnce() -> Buffers,
+) -> Result<BuffersOfAChunk<Buffers, Device::Readback>, GpuFailureCause>
+where
+    Device: DeviceCreatingTheBuffersOfAChunk,
+{
+    let out_of_memory_scope = device.open_the_out_of_memory_scope();
+    let buffers = create_the_buffers();
+    let readbacks: [Device::Readback; 2] =
+        std::array::from_fn(|_kernel| device.readback_of_one_kernel(outcome_bytes));
+    let captured = device.out_of_memory_captured_by(out_of_memory_scope, wait_limit);
+    match captured {
+        Ok(None) => Ok(BuffersOfAChunk { buffers, readbacks }),
+        Ok(Some(reason)) => Err(GpuFailureCause::OutOfMemoryCreatingTheBuffers {
+            bytes_held_by_the_chunk: bytes_held_by_this_chunk,
+            reason,
+        }),
+        Err(cause) => Err(cause),
+    }
 }
 
 /// 挂载那一刻：枚举适配器、判卡清单与校验、在本机每一张独立显卡上起设备与各段两份核、量一次提交的主机 CPU 时间（同
@@ -930,50 +1018,30 @@ impl GpuInUse {
             })
     }
 
-    /// 两份核各自的回读暂存（`outcome_bytes` 字节，映射来读、接结果缓冲的拷贝），按派发的次序。
-    fn readbacks_of_both_kernels(&self, outcome_bytes: u64) -> [wgpu::Buffer; 2] {
-        std::array::from_fn(|_kernel| {
-            self.device.create_buffer(&wgpu::BufferDescriptor {
-                label: None,
-                size: outcome_bytes,
-                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
-                mapped_at_creation: false,
-            })
-        })
-    }
-
-    /// 在只接显存不足的错误范围里建这一段的缓冲（`create_the_buffers`）与两份核各自的回读暂存（`outcome_bytes` 字节；它们算在这一段的占用里，
-    /// 判决 m4-gpugc-code-r2 第三节第 5 条：建在范围之外时它们显存不足报成没接住的错、这一批出错停下，不走「这一批转给别的卡」），
-    /// 这一段的账（`bytes_held_by_this_chunk`）调用方已经记下：建得成交回缓冲与回读暂存；建不成（显存不足，第 3 条「卡上此刻空闲的不到这个数」）
-    /// 就放掉这一段的账、交回 [`GpuFailureCause::OutOfMemoryCreatingTheBuffers`]，一次都不交。等 wgpu 交回那个错误范围也在等的上限之内。
+    /// 在只接显存不足的错误范围里建这一段的缓冲（`create_the_buffers`）与两份核各自的回读暂存
+    /// （[`buffers_and_both_readbacks_created_within_one_out_of_memory_scope`]），这一段的账（`bytes_held_by_this_chunk`）调用方已经记下：
+    /// 建得成交回缓冲与回读暂存；建不成（显存不足，第 3 条「卡上此刻空闲的不到这个数」，或等 wgpu 交回那个错误范围超过上限）
+    /// 就放掉这一段的账、交回那个错，一次都不交。
     fn buffers_created_within_the_free_memory<Buffers>(
         &mut self,
         bytes_held_by_this_chunk: u64,
         outcome_bytes: u64,
         create_the_buffers: impl FnOnce(&Self) -> Buffers,
-    ) -> Result<BuffersOfAChunk<Buffers>, GpuFailureCause> {
-        let out_of_memory_scope = self.device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
-        let buffers = create_the_buffers(self);
-        let readbacks = self.readbacks_of_both_kernels(outcome_bytes);
-        let captured = wait_for_the_wgpu_future(
-            out_of_memory_scope.pop(),
-            self.budget.wait_limit(),
-            &SteadyClockOfThisProcess::starting_now(),
+    ) -> Result<BuffersOfAChunk<Buffers, wgpu::Buffer>, GpuFailureCause> {
+        let gpu_in_use: &Self = self;
+        let created = buffers_and_both_readbacks_created_within_one_out_of_memory_scope(
+            &gpu_in_use.device,
+            gpu_in_use.budget.wait_limit(),
+            bytes_held_by_this_chunk,
+            outcome_bytes,
+            || create_the_buffers(gpu_in_use),
         );
-        let failure = match captured {
-            Ok(None) => return Ok(BuffersOfAChunk { buffers, readbacks }),
-            Ok(Some(out_of_memory)) => GpuFailureCause::OutOfMemoryCreatingTheBuffers {
-                bytes_held_by_the_chunk: bytes_held_by_this_chunk,
-                reason: format!("{out_of_memory}"),
-            },
-            Err(cause) => cause,
-        };
-        drop(buffers);
-        drop(readbacks);
-        self.budget
-            .memory_account_mut()
-            .release(bytes_held_by_this_chunk);
-        Err(failure)
+        if created.is_err() {
+            self.budget
+                .memory_account_mut()
+                .release(bytes_held_by_this_chunk);
+        }
+        created
     }
 
     /// 两份核在同一次提交里各算一遍、各自的结果缓冲拷进各自的回读暂存（`readbacks`，按派发的次序，与别的缓冲在同一个错误范围里建好的），
@@ -1833,7 +1901,8 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
         let mut outcomes_from_the_first_entry: Vec<(usize, Vec<Outcome>)> = Vec::new();
         let mut devices_given_work: Vec<GpuDeviceIdentifier> = Vec::new();
         let mut recuts_after_running_out_of_memory: u64 = 0;
-        let mut entries_to_hand_out: Vec<Range<usize>> = vec![0..entry_count];
+        // 一张待分的条目段的栈，起头只有整批那一段（不是 0 到 entry_count 这几个数）：装不下最小一片的卡交回的那几条再压回来。
+        let mut entries_to_hand_out: Vec<Range<usize>> = std::iter::once(0..entry_count).collect();
         let mut failure = None;
         'handing_out: while let Some(entries) = entries_to_hand_out.pop() {
             let plans = match self.plans_of_a_batch(
@@ -3438,4 +3507,229 @@ mod tests {
             "进程里只加这一次自己的 300"
         );
     }
+
+    /// 下面三条用例里一张卡分到几个单元（几张卡代价相同、平分）。
+    const UNITS_PER_CARD_IN_THE_SPLIT: usize = 4;
+
+    /// 七·3 验收「一张、两张、三张卡各跑一遍，数据量造到每张参与的卡都分到活、各自交过」（用户 2026-11-01 定）在替身卡上：`card_count` 张代价相同、
+    /// 上限各 1 GiB 的替身卡全都参与，一批 `UNITS_PER_CARD_IN_THE_SPLIT × card_count` 个 100 字节的单元——按代价平分，每张卡各分到相连的一段、各交一次：
+    /// 结果与 CPU 参照逐条相同；这一段记下交成 `card_count` 次、单元数那么多片，分到活的按本机次序正好是这几张卡。
+    fn every_card_of_a_batch_split_over_cards_on_the_cpu_is_given_work_and_submits(
+        card_count: usize,
+    ) {
+        const ONE_GIBIBYTE: u64 = 1 << 30;
+        let addresses: Vec<String> = (0..card_count)
+            .map(|card| format!("0000:{:02x}:00.0", 0x01 + 0x40 * card))
+            .collect();
+        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
+            addresses
+                .iter()
+                .map(|address| {
+                    CardComputedOnTheCpuForTests::of_the_address(address, Some(ONE_GIBIBYTE), 0)
+                })
+                .collect(),
+            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+        );
+        let unit_bytes = unit_bytes_of(UNITS_PER_CARD_IN_THE_SPLIT * card_count, 100);
+        let units = units_to_verify(&unit_bytes);
+        assert_eq!(
+            background_gpu.both_kernel_outcomes_of_the_batch(&units),
+            Ok(cpu_reference_outcomes_of(&units)),
+            "分到几张卡上算，结果照样与 CPU 参照逐条相同"
+        );
+        assert_eq!(
+            background_gpu
+                .take_the_submissions_of_the_batch()
+                .unit_checksum_verification,
+            GpuSubmissionsOfOneSegment {
+                submissions_completed: u64::try_from(card_count).expect("卡数"),
+                pieces_submitted: u64::try_from(units.len()).expect("单元数"),
+                resubmissions_after_timeouts: 0,
+                recuts_after_running_out_of_memory: 0,
+                devices_given_work: addresses
+                    .iter()
+                    .map(|address| card_identifier(address.as_str()))
+                    .collect(),
+            },
+            "每张参与的卡都分到活、各交一次"
+        );
+    }
+
+    #[test]
+    fn a_batch_over_one_card_on_the_cpu_gives_it_work_and_it_submits() {
+        every_card_of_a_batch_split_over_cards_on_the_cpu_is_given_work_and_submits(1);
+    }
+
+    #[test]
+    fn a_batch_over_two_cards_on_the_cpu_gives_each_card_work_and_each_submits() {
+        every_card_of_a_batch_split_over_cards_on_the_cpu_is_given_work_and_submits(2);
+    }
+
+    #[test]
+    fn a_batch_over_three_cards_on_the_cpu_gives_each_card_work_and_each_submits() {
+        every_card_of_a_batch_split_over_cards_on_the_cpu_is_given_work_and_submits(3);
+    }
+
+    /// 在内存里建的一份缓冲：建成了是它的字节数，建不成（显存不足）是 `None`。
+    #[derive(Debug, PartialEq, Eq)]
+    struct BufferInMemoryForTests {
+        bytes_if_created: Option<u64>,
+    }
+
+    /// 建一段缓冲的设备替身：卡上此刻空闲 `free_bytes`，每建一份照它扣，不够就建不成、记一条显存不足。记法照 wgpu 的错误范围：
+    /// 有范围开着时那条错归最里层那一个范围（每个范围只留它接住的第一条），一个范围都没开着时记成没接住的错（真卡上那条错走设备的没接住错误回调、
+    /// 这一批报 `UncapturedErrorOrDeviceLost`）。
+    struct DeviceOfAChunkInMemoryForTests {
+        free_bytes: Cell<u64>,
+        out_of_memory_captured_by_each_open_scope: std::cell::RefCell<Vec<Option<String>>>,
+        uncaptured_out_of_memory: std::cell::RefCell<Vec<String>>,
+    }
+
+    /// 替身开的一个范围：开时范围栈里的位置。
+    struct OutOfMemoryScopeInMemoryForTests {
+        position_in_the_open_scopes: usize,
+    }
+
+    impl DeviceOfAChunkInMemoryForTests {
+        fn with_free_bytes(free_bytes: u64) -> Self {
+            Self {
+                free_bytes: Cell::new(free_bytes),
+                out_of_memory_captured_by_each_open_scope: std::cell::RefCell::new(Vec::new()),
+                uncaptured_out_of_memory: std::cell::RefCell::new(Vec::new()),
+            }
+        }
+
+        fn buffer_of(&self, bytes: u64) -> BufferInMemoryForTests {
+            let free_bytes = self.free_bytes.get();
+            if bytes <= free_bytes {
+                self.free_bytes.set(free_bytes - bytes);
+                return BufferInMemoryForTests {
+                    bytes_if_created: Some(bytes),
+                };
+            }
+            let reason = format!("建 {bytes} 字节的缓冲：卡上只剩 {free_bytes} 字节");
+            let mut open_scopes = self.out_of_memory_captured_by_each_open_scope.borrow_mut();
+            match open_scopes.last_mut() {
+                Some(innermost_scope) => {
+                    if innermost_scope.is_none() {
+                        *innermost_scope = Some(reason);
+                    }
+                }
+                None => self.uncaptured_out_of_memory.borrow_mut().push(reason),
+            }
+            BufferInMemoryForTests {
+                bytes_if_created: None,
+            }
+        }
+    }
+
+    impl DeviceCreatingTheBuffersOfAChunk for DeviceOfAChunkInMemoryForTests {
+        type Readback = BufferInMemoryForTests;
+        type OutOfMemoryScope = OutOfMemoryScopeInMemoryForTests;
+
+        fn open_the_out_of_memory_scope(&self) -> OutOfMemoryScopeInMemoryForTests {
+            let mut open_scopes = self.out_of_memory_captured_by_each_open_scope.borrow_mut();
+            open_scopes.push(None);
+            OutOfMemoryScopeInMemoryForTests {
+                position_in_the_open_scopes: open_scopes.len() - 1,
+            }
+        }
+
+        fn readback_of_one_kernel(&self, outcome_bytes: u64) -> BufferInMemoryForTests {
+            self.buffer_of(outcome_bytes)
+        }
+
+        fn out_of_memory_captured_by(
+            &self,
+            scope: OutOfMemoryScopeInMemoryForTests,
+            _wait_limit: Duration,
+        ) -> Result<Option<String>, GpuFailureCause> {
+            let mut open_scopes = self.out_of_memory_captured_by_each_open_scope.borrow_mut();
+            assert_eq!(
+                scope.position_in_the_open_scopes + 1,
+                open_scopes.len(),
+                "关的是最里层那个范围"
+            );
+            Ok(open_scopes.pop().expect("上面判过：关的那个范围还开着"))
+        }
+    }
+
+    /// 这一段自己那几样缓冲合起来的字节数与一份核的结果字节数（两份回读暂存各这么大）：只用来造「那几样建得成、回读暂存建不成」。
+    const BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK: u64 = 1000;
+    const OUTCOME_BYTES_OF_THE_CHUNK: u64 = 64;
+
+    /// 判决 m4-gpugc-code-r2 第三节第 5 条、四之一（用户 2026-11-01 定「造不出就换方法继续造」）：卡上空闲的装得下这一段自己那几样缓冲，
+    /// 装不下两份回读暂存——回读暂存建在同一个只接显存不足的范围里，那一条显存不足被范围接住：交回 `OutOfMemoryCreatingTheBuffers`
+    /// （外壳照它放掉这一段的账、走「这一批转给别的卡」），一条没接住的错都没有。回读暂存建在范围之外时，那一条显存不足没被接住、这一段照「建成了」交回。
+    #[test]
+    fn readbacks_short_of_free_memory_are_caught_by_the_out_of_memory_scope_of_the_chunk() {
+        let device = DeviceOfAChunkInMemoryForTests::with_free_bytes(
+            BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK + OUTCOME_BYTES_OF_THE_CHUNK - 1,
+        );
+        let created = buffers_and_both_readbacks_created_within_one_out_of_memory_scope(
+            &device,
+            Duration::from_secs(1),
+            BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK + 2 * OUTCOME_BYTES_OF_THE_CHUNK,
+            OUTCOME_BYTES_OF_THE_CHUNK,
+            || device.buffer_of(BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK),
+        );
+        assert!(
+            matches!(
+                created,
+                Err(GpuFailureCause::OutOfMemoryCreatingTheBuffers {
+                    bytes_held_by_the_chunk,
+                    ..
+                }) if bytes_held_by_the_chunk
+                    == BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK + 2 * OUTCOME_BYTES_OF_THE_CHUNK
+            ),
+            "回读暂存建不成的那一条显存不足被这一段的范围接住、报成建缓冲显存不足（带这一段的账）：{:?}",
+            created.as_ref().map(|_buffers| "建成了")
+        );
+        assert_eq!(
+            *device.uncaptured_out_of_memory.borrow(),
+            Vec::<String>::new(),
+            "没有一条显存不足落到范围之外"
+        );
+    }
+
+    /// 同一条的阳性对照：空闲的装得下这一段自己那几样与两份回读暂存——交回建成的那几样与两份各 64 字节的回读暂存，没有错、范围都关上了。
+    #[test]
+    fn a_chunk_and_both_readbacks_within_the_free_memory_are_created_in_one_out_of_memory_scope() {
+        let device = DeviceOfAChunkInMemoryForTests::with_free_bytes(
+            BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK + 2 * OUTCOME_BYTES_OF_THE_CHUNK,
+        );
+        let created = buffers_and_both_readbacks_created_within_one_out_of_memory_scope(
+            &device,
+            Duration::from_secs(1),
+            BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK + 2 * OUTCOME_BYTES_OF_THE_CHUNK,
+            OUTCOME_BYTES_OF_THE_CHUNK,
+            || device.buffer_of(BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK),
+        )
+        .expect("空闲的正好装得下：建得成");
+        assert_eq!(
+            (created.buffers, created.readbacks),
+            (
+                BufferInMemoryForTests {
+                    bytes_if_created: Some(BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK)
+                },
+                [
+                    BufferInMemoryForTests {
+                        bytes_if_created: Some(OUTCOME_BYTES_OF_THE_CHUNK)
+                    },
+                    BufferInMemoryForTests {
+                        bytes_if_created: Some(OUTCOME_BYTES_OF_THE_CHUNK)
+                    },
+                ]
+            )
+        );
+        assert_eq!(device.free_bytes.get(), 0, "三样都照字节数扣了");
+        assert!(
+            device.uncaptured_out_of_memory.borrow().is_empty()
+                && device
+                    .out_of_memory_captured_by_each_open_scope
+                    .borrow()
+                    .is_empty(),
+            "没有错、范围关上了"
+        );
+    }
 }
diff --git a/crates/singlefs-core/src/compaction_plan.rs b/crates/singlefs-core/src/compaction_plan.rs
index 175e49a..571bbe6 100644
--- a/crates/singlefs-core/src/compaction_plan.rs
+++ b/crates/singlefs-core/src/compaction_plan.rs
@@ -1923,14 +1923,14 @@ mod tests {
         ];
         let opened = BTreeSet::new();
         let refused = SegmentsRefusedByTheRegionScan::default();
-        let check = |scans: &[RegionScan], pins: &BTreeSet<SlotNumber>, expected| {
+        let check = |scans_in_this_order: &[RegionScan], pins: &BTreeSet<SlotNumber>, expected| {
             let actual = choose_region(
                 &records,
                 area,
                 &opened,
                 None,
                 pins,
-                scans,
+                scans_in_this_order,
                 &refused,
                 RegionChoiceCriterion::Greedy,
             );
@@ -1943,7 +1943,7 @@ mod tests {
                     &opened,
                     None,
                     pins,
-                    scans,
+                    scans_in_this_order,
                     &refused,
                     RegionChoiceCriterion::Greedy
                 )
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 763c267..c1fac04 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -9007,6 +9007,46 @@ pub fn mount_writable_with_background_compute_settings_and_the_gc_cpu_compensati
     .map_err(WritableMountWithBackgroundComputeRefused::Mount)
 }
 
+/// 同 [`mount_writable_with_background_compute_settings_and_the_gc_cpu_compensation_threshold`]，整理的只供测试的开关
+/// （`compaction_plan::CompactionSwitches`，判决 G6）也由调用方给（[`mount_writable_with_compaction_switches`] 那一种做法）：装在交回的分配器上，
+/// 挂载续做盘上那一条未完成的整理意图照它与这次挂载给的后台计算设置推，这次挂载之后的会话照它。可写挂载续做里一批在 GPU 上出错停下那一种
+/// 由它强制进入（`.claude/rules/fs-design.md` 五条硬要求第 2 条；判决 `research/prompts/m4-gpugc-code-r2-main-verification.md` 四之一）：
+/// 产品路径的入口恒给 `CompactionSwitches::PRODUCT`，批大小条款没写，续做在签任何东西、交任何一段给 GPU 之前停下。
+/// 与别的只供测试的开关一样在发布构建里够得着（C921（只供测试的开关在发布构建里够得着））。
+///
+/// # Errors
+/// 同 [`mount_writable_with_background_compute_settings_and_the_gc_cpu_compensation_threshold`]。
+pub fn mount_writable_with_background_compute_settings_the_gc_cpu_compensation_threshold_and_compaction_switches<
+    Device: BlockDevice,
+>(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut Vec<(DeviceIdentity, Device)>,
+    background_compute_settings: BackgroundComputeSettings,
+    gpu_compute: Option<BackgroundGpuComputeHandle>,
+    gc_cpu_compensation_threshold: BackgroundGcCpuCompensationThreshold,
+    compaction_switches: CompactionSwitches,
+) -> Result<Mounted, WritableMountWithBackgroundComputeRefused> {
+    let background_compute =
+        admit_background_compute_settings_with_the_gpu_compute_and_the_gc_cpu_compensation_threshold(
+            background_compute_settings,
+            gpu_compute,
+            gc_cpu_compensation_threshold,
+        )
+        .map_err(WritableMountWithBackgroundComputeRefused::BackgroundComputeSettings)?;
+    mount_writable_with_every_test_only_switch(
+        parameters,
+        devices,
+        SpaceAdmission::JudgedByTheFormula,
+        ShadowLedger::On,
+        DeletionSwitches::PRODUCT,
+        compaction_switches,
+        BeforeTheOneReread::RereadImmediately,
+        InstanceTableSweepConfiguration::PRODUCT,
+        background_compute,
+    )
+    .map_err(WritableMountWithBackgroundComputeRefused::Mount)
+}
+
 /// 可写挂载择系统配置（S3，C707）：先按调用方给的本池 fsid 筛（[`choose_system_configuration_of_the_pool`]：自述别的 fsid 的槽当自证不过）；
 /// 筛完一块盘可择的都没有（`NoValidSystemConfiguration`：盘上没有一槽是调用方说的这个池的）才不筛再择一次，择到的 fsid 与调用方的不同，
 /// 由后面的参数核报成 [`MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration`]（调用方交错了 fsid 那一形照旧报）。
```

