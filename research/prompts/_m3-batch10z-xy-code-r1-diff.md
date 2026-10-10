# 根环实现第 10z 批第二十四、二十五块：代码轮第一轮改动全文（2026-10-09）

两块实现员各交的 `crates.patch` 原样，按合入次序：第二十四块（返工那一份，撤掉了挂载身份）、第二十五块。各块变异表追加行原样附在各块之后；两块都没有整行替换行，合入时没有改动任何一行。

## 第二十四块

````diff
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -224,7 +224,8 @@
     /// 被抛弃根罩着、C 不罩的不在其内。
     pub isolated_slots_on_each_device: Vec<(DeviceIdentity, u64)>,
     /// 被拒的那一次要接在后面的那一版（报这一形那一刻会话的现行那一版）的实例代号与 txg：要出路时会话不在这一版上就不预演
-    /// （[`WaysOutOfTheWideProtectionRefused::SessionMovedPastTheVersionRefusedOnNotSupportedInTheFirstVersion`]）。
+    /// （[`WaysOutOfTheWideProtectionRefused::SessionMovedPastTheVersionRefusedOnNotSupportedInTheFirstVersion`]）。报里不记是哪个会话报的：
+    /// 别的会话的报这两个数恰好相同时不拦（同一成员的文档）。
     instance_of_the_version_refused_on: InstanceGeneration,
     checkpoint_txg_of_the_version_refused_on: CheckpointTxg,
     /// 被拒的那一次之后这次用户改动还剩什么要发（内容拷一份）：要出路时推完环照它再预演一次。
@@ -261,18 +262,32 @@
     pub ways_out: Vec<String>,
 }
 
-/// 要出路时没预演。
+/// 要出路时没预演。成员按调用方要做的决定分：再发一次这次改动拿新的报、换回挂载时那一份盘表。
+/// 两种都在预演之前返回：一个盘都没读、一个写都没发，会话不动。
 #[derive(Debug)]
 pub enum WaysOutOfTheWideProtectionRefused {
-    /// 会话的现行那一版已不是报「被宽护卡住」时被拒的那一次要接在后面的那一版（之后又发过：推过环、别的改动发成、走空走了几步；或这一份报
-    /// 不是这个会话的）：报里的 R 那一槽、要推的次数与挡着的槽数说的是那一刻，照这一刻预演会与它们对不上，按哪一刻算条款没写，第一版不支持。
-    /// 一个盘都没读、一个写都没发，会话不动。调用方要做的是再发一次这次改动、拿新的那一份报再要出路。
+    /// 会话的现行那一版已不是报「被宽护卡住」时被拒的那一次要接在后面的那一版（之后又发过：推过环、别的改动发成、走空走了几步）：
+    /// 报里的 R 那一槽、要推的次数与挡着的槽数说的是那一刻，照这一刻预演会与它们对不上，按哪一刻算条款没写，第一版不支持。
+    /// 调用方要做的是再发一次这次改动、拿新的那一份报再要出路。
+    ///
+    /// 这一道只比实例代号与 txg，报里不记是哪个会话报的。别的会话（同一个池卸载或崩了之后再挂、同一份 mkfs 起的另一个池、读盘造的另一条会话）
+    /// 的报若这两个数恰好与这个会话的现行那一版相同，这里不拦：照这个会话的状态（分配器、盘、现行那一版）预演，出路一的文字仍是报里的数
+    /// （R 的 txg 或 R 那一槽），列出路二时其中的 R 那一槽与要推的次数也是报里的（第 10z 批代码轮攻方「打中一」；用户 2026-10-09 定撤掉会话身份、照实写）。
+    /// 调用方要拿报去问报它的那个会话。
     SessionMovedPastTheVersionRefusedOnNotSupportedInTheFirstVersion {
         instance_refused_on: InstanceGeneration,
         checkpoint_txg_refused_on: CheckpointTxg,
         instance_of_the_current_version: InstanceGeneration,
         checkpoint_txg_of_the_current_version: CheckpointTxg,
     },
+    /// 会话还在被拒的那一版上（实例代号与 txg 与报里记的相同），交进来的盘表的设备身份（按次序）却与这次挂载核过的那一份不同（多一块、少一块、换了次序、
+    /// 同一个身份交两次；与 [`UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount`] 同一判，`mount::ParametersAndDeviceTableOfTheMount::is_the_device_table_of_the_mount`）。
+    /// 预演里推环那一串核盘表不看次序：不先核这一判时，次序对调的盘表预演放得下、列推环，照出路二推完再发这次改动却拒成
+    /// `DeviceTableOtherThanTheOneOfTheMount`（第 10z 批代码轮攻方「打中二」）。调用方要做的是换回挂载时那一份盘表再要出路。
+    DeviceTableOtherThanTheOneOfTheMount {
+        device_table_of_the_mount: Vec<DeviceIdentity>,
+        device_table_handed_in: Vec<DeviceIdentity>,
+    },
 }
 
 impl StuckBehindTheWideProtection {
@@ -1240,12 +1255,16 @@
     ///
     /// 收 `&self`：预演只读交进来的盘、在分配器的拷贝上走，会话的分配器、现行那一版一样都不动——类型上就动不了（`PoolAllocator` 里没有内部可变），
     /// 调用方不必为了要出路把会话交出去。盘表只读借：只进内存的盘只读借底下那块盘，写一个都发不到它上面。
-    /// 交进来的盘表照推环入口那一套核（在预演的推环里，核不过就是推环被拒、出路只列卸载再挂），与照出路二真推时同一个结局。
+    /// 交进来的盘表先照 [`Self::publish_user_change`] 那样与挂载时核过的那一份逐项比（次序也比），再照推环入口那一套核（在预演的推环里，
+    /// 核不过就是推环被拒、出路只列卸载再挂），与照出路二真推、再发这次改动时同一个结局。
     ///
-    /// 第一道看会话还在不在被拒的那一版上（实例代号与 txg 都与报里记的相同）：不在就不预演，一个盘都不读。
+    /// 第一道看会话还在不在被拒的那一版上（实例代号与 txg 都与报里记的相同），再比盘表：哪一道不过都不预演，一个盘都不读。
+    /// 第一道只比这两个数，认不出 `stuck` 是不是这个会话报的：别的会话的报若这两个数恰好与这个会话的现行那一版相同，照样过，
+    /// 照这个会话的状态预演（[`WaysOutOfTheWideProtectionRefused::SessionMovedPastTheVersionRefusedOnNotSupportedInTheFirstVersion`]）。
+    /// 调用方要拿报去问报它的那个会话。
     ///
     /// # Errors
-    /// [`WaysOutOfTheWideProtectionRefused`] 的一种。
+    /// [`WaysOutOfTheWideProtectionRefused`] 的两种。
     pub fn ways_out_of_the_wide_protection<Device: BlockDevice>(
         &self,
         devices: &[(DeviceIdentity, Device)],
@@ -1277,6 +1296,20 @@
                 );
             }
         };
+        if !self
+            .parameters_and_device_table
+            .is_the_device_table_of_the_mount(devices)
+        {
+            return Err(
+                WaysOutOfTheWideProtectionRefused::DeviceTableOtherThanTheOneOfTheMount {
+                    device_table_of_the_mount: self
+                        .parameters_and_device_table
+                        .device_identities_in_table_order()
+                        .to_vec(),
+                    device_table_handed_in: devices.iter().map(|(identity, _)| *identity).collect(),
+                },
+            );
+        }
         let pushing_the_root_ring = rehearse_the_push_then_the_remainder_of_the_change(
             self.parameters_and_device_table.parameters_on_disk(),
             devices,
@@ -1601,10 +1634,23 @@
     }
 
     impl DeviceOfBytes {
-        /// 每个字节按位置与 `seed` 填一个图样：读回哪一块看得出来。
+        /// 物理块 [`PHYSICAL_BLOCK_BYTES`]、[`BLOCKS_OF_THE_DISK`] 块的盘，每个字节按位置与 `seed` 填一个图样：读回哪一块看得出来。
         fn filled_with_a_pattern(seed: u8) -> Self {
+            Self::filled_with_a_pattern_on_blocks_of(
+                PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
+                BLOCKS_OF_THE_DISK,
+                seed,
+            )
+        }
+
+        /// 物理块 `physical_block_size`、`blocks_of_the_disk` 块的盘，图样同 [`Self::filled_with_a_pattern`]。
+        fn filled_with_a_pattern_on_blocks_of(
+            physical_block_size: PhysicalBlockSizeInBytes,
+            blocks_of_the_disk: u64,
+            seed: u8,
+        ) -> Self {
             let device_bytes =
-                usize::try_from(u64::from(PHYSICAL_BLOCK_BYTES) * BLOCKS_OF_THE_DISK)
+                usize::try_from(u64::from(physical_block_size.0) * blocks_of_the_disk)
                     .expect("盘长装得进 usize");
             DeviceOfBytes {
                 bytes: (0..device_bytes)
@@ -1614,7 +1660,7 @@
                             .wrapping_add(seed)
                     })
                     .collect(),
-                physical_block_size: PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
+                physical_block_size,
             }
         }
 
@@ -1926,41 +1972,64 @@
     }
 
     /// 偏移：多半落在物理块边界上、盘以内，少数不对齐、正好在盘尾、越过盘尾、贴着 u64 的顶。
-    fn random_offset(state: &mut u64, device_size: u64) -> u64 {
-        let block = u64::from(PHYSICAL_BLOCK_BYTES);
+    fn random_offset(state: &mut u64, device_size: u64, block_bytes: u64) -> u64 {
         match next_random(state) % 8 {
-            0 => next_random(state) % (device_size + 2 * block),
+            0 => next_random(state) % (device_size + 2 * block_bytes),
             1 => u64::MAX - next_random(state) % 1024,
             2 => device_size,
-            _ => (next_random(state) % (device_size / block)) * block,
+            _ => (next_random(state) % (device_size / block_bytes)) * block_bytes,
         }
     }
 
     /// 长度：多半是一到几个物理块，少数是 0、不对齐、跨过盘尾。
-    fn random_length(state: &mut u64, device_size: u64) -> u64 {
-        let block = u64::from(PHYSICAL_BLOCK_BYTES);
+    fn random_length(state: &mut u64, device_size: u64, block_bytes: u64) -> u64 {
         match next_random(state) % 8 {
             0 => 0,
             1 => next_random(state) % (2 * device_size),
-            _ => (1 + next_random(state) % 4) * block,
+            _ => (1 + next_random(state) % 4) * block_bytes,
         }
     }
 
-    #[test]
-    fn the_device_keeping_writes_in_memory_agrees_with_a_device_written_through_on_a_random_request_stream(
-    ) {
-        let disk = DeviceOfBytes::filled_with_a_pattern(23);
+    /// 一串随机请求（读、写、写零，偶尔一道屏障）同时交给盖在底下那块盘上的只进内存的盘与一块把写直接写进去的同样的盘之后看到的。
+    struct RandomRequestStreamAgainstADeviceWrittenThrough {
+        /// 两边对不上的请求，逐条写明第几个、什么请求、哪一样不同（读回的字节、交回值）。
+        disagreements: Vec<String>,
+        refused_requests: u64,
+        kept_writes: u64,
+        /// 一串请求之后整盘读回的字节与写穿的那块盘相同。
+        whole_disk_reads_back_as_written_through: bool,
+        /// 一串请求之后底下那块盘一个字节都没变。
+        disk_below_is_unchanged: bool,
+    }
+
+    /// 物理块 `physical_block_size`、`blocks_of_the_disk` 块、图样种子 `seed` 的盘上跑 [`REQUESTS_OF_THE_RANDOM_STREAM`] 个随机请求。
+    fn random_request_stream_against_a_device_written_through(
+        physical_block_size: PhysicalBlockSizeInBytes,
+        blocks_of_the_disk: u64,
+        seed: u8,
+    ) -> RandomRequestStreamAgainstADeviceWrittenThrough {
+        let disk = DeviceOfBytes::filled_with_a_pattern_on_blocks_of(
+            physical_block_size,
+            blocks_of_the_disk,
+            seed,
+        );
         let bytes_before = disk.bytes.clone();
         let device_size = disk.size_in_bytes();
-        let mut written_through = DeviceOfBytes::filled_with_a_pattern(23);
+        let block_bytes = u64::from(physical_block_size.0);
+        let mut written_through = DeviceOfBytes::filled_with_a_pattern_on_blocks_of(
+            physical_block_size,
+            blocks_of_the_disk,
+            seed,
+        );
         let mut kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
         let mut state = 1_u64;
+        let mut disagreements: Vec<String> = Vec::new();
         let mut refused_requests = 0_u64;
         let mut kept_writes = 0_u64;
-        // 迭代上界是请求个数；跨轮带着两块盘与随机数的状态；出口只有断言。
+        // 迭代上界是请求个数；跨轮带着两块盘、随机数的状态与两个计数；没有提前出口。
         for request_number in 1..=REQUESTS_OF_THE_RANDOM_STREAM {
-            let offset = DeviceOffsetInBytes(random_offset(&mut state, device_size));
-            let length = random_length(&mut state, device_size);
+            let offset = DeviceOffsetInBytes(random_offset(&mut state, device_size, block_bytes));
+            let length = random_length(&mut state, device_size, block_bytes);
             let length_in_memory = usize::try_from(length).expect("长度不过两倍盘长");
             let (request, through, in_memory) = match next_random(&mut state) % 3 {
                 0 => {
@@ -1968,11 +2037,12 @@
                     let mut in_memory_buffer = vec![0xAA_u8; length_in_memory];
                     let through = written_through.read_at(offset, &mut through_buffer);
                     let in_memory = kept.read_at(offset, &mut in_memory_buffer);
-                    assert!(
-                        through_buffer == in_memory_buffer,
-                        "第 {request_number} 个请求（读 {} 起 {length} 字节）：读回的字节相同",
-                        offset.0
-                    );
+                    if through_buffer != in_memory_buffer {
+                        disagreements.push(format!(
+                            "第 {request_number} 个请求（读 {} 起 {length} 字节）：读回的字节不同",
+                            offset.0
+                        ));
+                    }
                     ("读", through, in_memory)
                 }
                 1 => {
@@ -1991,12 +2061,14 @@
                     kept.write_zeroes_at(offset, length),
                 ),
             };
-            assert_eq!(
-                format!("{through:?}"),
-                format!("{in_memory:?}"),
-                "第 {request_number} 个请求（{request} {} 起 {length} 字节）：交回值相同",
-                offset.0
-            );
+            let (through_returned, in_memory_returned) =
+                (format!("{through:?}"), format!("{in_memory:?}"));
+            if through_returned != in_memory_returned {
+                disagreements.push(format!(
+                    "第 {request_number} 个请求（{request} {} 起 {length} 字节）：写穿的盘交回 {through_returned}，只进内存的盘交回 {in_memory_returned}",
+                    offset.0
+                ));
+            }
             match through {
                 Ok(()) => kept_writes += u64::from(request != "读"),
                 Err(_) => refused_requests += 1,
@@ -2005,17 +2077,200 @@
                 kept.barrier().expect("屏障什么都不做");
             }
         }
+        let mut whole_disk =
+            vec![0xAA_u8; usize::try_from(device_size).expect("盘长装得进 usize")];
+        kept.read_at(DeviceOffsetInBytes(0), &mut whole_disk)
+            .expect("整盘一次读：对齐、没越过盘尾");
+        RandomRequestStreamAgainstADeviceWrittenThrough {
+            disagreements,
+            refused_requests,
+            kept_writes,
+            whole_disk_reads_back_as_written_through: whole_disk == written_through.bytes,
+            disk_below_is_unchanged: disk.bytes == bytes_before,
+        }
+    }
+
+    /// 物理块 512 字节的盘上：只进内存的盘与写穿的盘逐个请求交回值、读回的字节相同，整盘读回相同，底下那块盘一字不动。
+    #[test]
+    fn the_device_keeping_writes_in_memory_agrees_with_a_device_written_through_on_a_random_request_stream(
+    ) {
+        let stream = random_request_stream_against_a_device_written_through(
+            PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
+            BLOCKS_OF_THE_DISK,
+            23,
+        );
+        assert!(
+            stream.disagreements.is_empty(),
+            "每个请求两边交回值与读回的字节相同，而 {} 个对不上，头几个：{:?}",
+            stream.disagreements.len(),
+            stream.disagreements.iter().take(5).collect::<Vec<_>>()
+        );
         assert!(
-            refused_requests > 0 && kept_writes > 0,
-            "这一串里既有被拒的请求，也有做成的写与写零（阳性对照）：拒 {refused_requests} 个、写 {kept_writes} 个"
+            stream.refused_requests > 0 && stream.kept_writes > 0,
+            "这一串里既有被拒的请求，也有做成的写与写零（阳性对照）：拒 {} 个、写 {} 个",
+            stream.refused_requests,
+            stream.kept_writes
         );
         assert!(
-            read_blocks(&kept, 0, BLOCKS_OF_THE_DISK) == written_through.bytes,
+            stream.whole_disk_reads_back_as_written_through,
             "整盘读回与写穿的那块盘相同"
         );
         assert!(
-            disk.bytes == bytes_before,
+            stream.disk_below_is_unchanged,
+            "一串写之后底下那块盘一个字节都没变"
+        );
+    }
+
+    /// 物理块 4096 字节的盘（正推 `research/prompts/m3-batch10z-vw-code-r1-sonnet-output.md` K2：入库用例原来只在 512 的盘上，
+    /// 判据里把物理块写死 512 照绿）：同一串随机请求按 4096 的块取偏移与长度，两边逐个请求相同、整盘读回相同、底下那块盘一字不动。
+    #[test]
+    fn the_device_keeping_writes_in_memory_on_4096_byte_physical_blocks_agrees_with_a_device_written_through_on_a_random_request_stream(
+    ) {
+        let stream = random_request_stream_against_a_device_written_through(
+            PhysicalBlockSizeInBytes(FOUR_KIBIBYTE_PHYSICAL_BLOCK_BYTES),
+            BLOCKS_OF_THE_DISK,
+            29,
+        );
+        assert!(
+            stream.disagreements.is_empty(),
+            "每个请求两边交回值与读回的字节相同，而 {} 个对不上，头几个：{:?}",
+            stream.disagreements.len(),
+            stream.disagreements.iter().take(5).collect::<Vec<_>>()
+        );
+        assert!(
+            stream.refused_requests > 0 && stream.kept_writes > 0,
+            "这一串里既有被拒的请求，也有做成的写与写零（阳性对照）：拒 {} 个、写 {} 个",
+            stream.refused_requests,
+            stream.kept_writes
+        );
+        assert!(
+            stream.whole_disk_reads_back_as_written_through,
+            "整盘读回与写穿的那块盘相同"
+        );
+        assert!(
+            stream.disk_below_is_unchanged,
             "一串写之后底下那块盘一个字节都没变"
         );
     }
+
+    /// 4096 字节的物理块。
+    const FOUR_KIBIBYTE_PHYSICAL_BLOCK_BYTES: u32 = 4096;
+
+    /// 物理块 4096 字节的盘上，落在 512 字节边界上、不在 4096 边界上的请求（偏移或长度）照两个后端那一份判据拒成 `Unaligned`、
+    /// 带 4096 这个物理块，一个块都不记；同一块盘上按 4096 对齐的写读得回来。判据里物理块写死 512 时这几个请求都放行。
+    #[test]
+    fn requests_on_512_byte_boundaries_off_the_4096_byte_physical_blocks_are_refused_before_anything_is_kept(
+    ) {
+        let block = PhysicalBlockSizeInBytes(FOUR_KIBIBYTE_PHYSICAL_BLOCK_BYTES);
+        let block_bytes = u64::from(FOUR_KIBIBYTE_PHYSICAL_BLOCK_BYTES);
+        let disk = DeviceOfBytes::filled_with_a_pattern_on_blocks_of(block, BLOCKS_OF_THE_DISK, 31);
+        let mut kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
+        let one_block = vec![0x3C_u8; usize::try_from(block_bytes).expect("一块装得进 usize")];
+        let one_sector = vec![0x3C_u8; 512];
+        let offset_on_a_sector_inside_block_one = DeviceOffsetInBytes(block_bytes + 512);
+        let refusals = [
+            (
+                "偏移在 512 边界上、不在 4096 边界上的写",
+                kept.write_at(
+                    offset_on_a_sector_inside_block_one,
+                    &one_block,
+                    WriteDurability::Plain,
+                ),
+                offset_on_a_sector_inside_block_one,
+                block_bytes,
+            ),
+            (
+                "长度 512 的写",
+                kept.write_at(
+                    DeviceOffsetInBytes(2 * block_bytes),
+                    &one_sector,
+                    WriteDurability::Plain,
+                ),
+                DeviceOffsetInBytes(2 * block_bytes),
+                512,
+            ),
+            (
+                "长度 512 的写零",
+                kept.write_zeroes_at(DeviceOffsetInBytes(3 * block_bytes), 512),
+                DeviceOffsetInBytes(3 * block_bytes),
+                512,
+            ),
+            (
+                "偏移在 512 边界上、不在 4096 边界上的读",
+                kept.read_at(offset_on_a_sector_inside_block_one, &mut vec![0_u8; 512]),
+                offset_on_a_sector_inside_block_one,
+                512,
+            ),
+        ];
+        for (request, outcome, offset, length) in refusals {
+            assert!(
+                matches!(
+                    outcome,
+                    Err(BlockDeviceError::Unaligned {
+                        offset: refused_offset,
+                        length: refused_length,
+                        physical_block_size: FOUR_KIBIBYTE_PHYSICAL_BLOCK_BYTES,
+                    }) if refused_offset == offset && refused_length == length
+                ),
+                "{request}：拒成 Unaligned，带偏移 {}、长度 {length} 与物理块 4096，而交回 {outcome:?}",
+                offset.0
+            );
+        }
+        kept.write_at(
+            DeviceOffsetInBytes(5 * block_bytes),
+            &one_block,
+            WriteDurability::Plain,
+        )
+        .expect("按 4096 对齐的一块写做得成（阳性对照）");
+        let mut expected = disk.bytes.clone();
+        let start_of_block_five = usize::try_from(5 * block_bytes).expect("偏移装得进 usize");
+        expected[start_of_block_five..start_of_block_five + one_block.len()]
+            .copy_from_slice(&one_block);
+        let mut whole_disk = vec![0xAA_u8; disk.bytes.len()];
+        kept.read_at(DeviceOffsetInBytes(0), &mut whole_disk)
+            .expect("整盘一次读：对齐、没越过盘尾");
+        assert!(
+            whole_disk == expected,
+            "被拒的请求一个块都没记：整盘读回只有按 4096 对齐写的那一块是写进去的"
+        );
+    }
+
+    /// 「被宽护卡住」那一份报拷下的这次改动（[`UserChangeKeptByTheRefusal`]），要出路时读回成交给预演的改动，与被拒那一次交进来的改动
+    /// 逐项相同：种类、内容全长、写入时间、inode、个数（正推 `research/prompts/m3-batch10z-vw-code-r1-sonnet-output.md` K3-3：
+    /// 记错这几项的 8 种改坏原来照绿，其中读回时把覆盖写当顺序写那一种只在读回这一步，报的 `{:?}` 看不出来）。
+    #[test]
+    fn each_kind_of_change_kept_by_the_refusal_reads_back_as_the_change_handed_in() {
+        let overwrite_content: Vec<u8> = (0..2999_u32)
+            .map(|position| u8::try_from(position % 241).expect("小于 256"))
+            .collect();
+        let sequential_write_content: Vec<u8> = (0..5003_u32)
+            .map(|position| u8::try_from((position * 7) % 239).expect("小于 256"))
+            .collect();
+        let changes_handed_in = [
+            UserChange::Overwrite(FirstFile {
+                content: &overwrite_content,
+                write_time_seconds: 1_000_003,
+            }),
+            UserChange::SequentialWrite(FirstFile {
+                content: &sequential_write_content,
+                write_time_seconds: 1_000_007,
+            }),
+            UserChange::NewInodes {
+                count: 37,
+                write_time_seconds: 1_000_011,
+            },
+            UserChange::Unlink {
+                inode: InodeNumber(4_099),
+                write_time_seconds: 1_000_013,
+            },
+        ];
+        for change_handed_in in changes_handed_in {
+            let kept = UserChangeKeptByTheRefusal::kept_from(change_handed_in);
+            assert_eq!(
+                format!("{:?}", kept.as_user_change()),
+                format!("{change_handed_in:?}"),
+                "拷下的改动读回成被拒那一次交进来的改动（种类、内容全长、写入时间、inode、个数逐项相同）"
+            );
+        }
+    }
 }
--- a/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs
+++ b/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs
@@ -39,6 +39,13 @@
 //!   照它走完放得下；出路一按 R 那一槽有没有读得出的有效根分两种说法，出路二写推几次、写掉几槽、放出来的隔离槽数（推完前后之差，两槽 C 重叠时不重数）；
 //!   推环入口冻结着时第一道就拒、不核盘表时盘体对调照推（攻方 M1、M2）、两槽 C 推到较晚的那一槽（攻方 M12）；C 记着落点而一槽都没隔离、
 //!   停因是预算用完或抬 F 取不到落点时不报被宽护卡住，记着空串的槽不算（正推 K4-2 活下来的几个）；根槽读坏的被抛弃根那一圈补满整段跨度。
+//! - 要出路先核盘表次序（用户 2026-10-09 弹窗定「现在补第二十四块再攻一轮」，判决 `research/prompts/m3-batch10z-vw-code-r1-main-verification.md`
+//!   第二节 K1-3、第三节；第 10z 批第二十四块实现员定形，被攻过零轮）：次序对调、多一块、少一块的盘表要出路，在预演之前拒成盘表那一种、
+//!   一个盘都不读（攻方「打中二」）。报不带会话身份（用户 2026-10-09 弹窗定「撤掉身份，文档照实写」）：别的会话实例代号与 txg 相同的报
+//!   要出路时不拦（攻方「打中一」），照实写在 `WaysOutOfTheWideProtectionRefused` 的文档里，这里不钉。
+//! - 判别力补的两样（判决第二节 K1-2、K3）：推环做得成、推完之后走空照旧被拒的那一形（正推 K3-4 的 k = 4，注入级）只列卸载再挂，
+//!   预演的拒绝与推完真再发同一个 unlink 的拒绝逐字相同（`walkSkipped` 红）；四种改动被宽护卡住时报里记的这次改动逐项相同
+//!   （正推 K3-3 的 7 种改坏红；读回时换种类那一种与物理块 4096 的盘由 `singlefs-core` 的库单测钉）。
 // 池（384 槽那一档小盘）与 R 那块盘都经 `tests/common_unwitnessed_publish/mod.rs` 起（它转出 `tests/common_admission_pool/mod.rs` 的全部）：
 // 再声明 `common_admission` 就把 `common_admission_pool` 当模块装两遍（clippy `duplicate_mod`）。
 mod common_unwitnessed_publish;
@@ -6205,7 +6212,7 @@
             ),
             "带被拒那一版与现行那一版的实例代号与 txg"
         ),
-        Ok(found) => panic!("会话已不在被拒的那一版上：不预演，而交回 {found:?}"),
+        other => panic!("会话已不在被拒的那一版上：不预演，而交回 {other:?}"),
     }
     assert_eq!(
         (
@@ -6234,20 +6241,10 @@
     let mut pool = crashed.mounted_and_stuck_once_by_a_write_grown_by(
         DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY,
     );
-    for device_map in &mut pool.session.as_mut().expect("挂着的会话").allocator.devices {
-        let unit_area_start = device_map.unit_area_start().slot().0;
-        // 迭代上界是单元区槽数；跨轮只带没挡的槽数；出口：只剩下要留的那几个。
-        for slot in unit_area_start..unit_area_start + device_map.unit_area_slots() {
-            if device_map.unblocked_slots()
-                <= UNBLOCKED_SLOTS_LEFT_FOR_THE_WALK_AFTER_THE_UNLINK_INTENT_TO_BE_STUCK
-            {
-                break;
-            }
-            if !device_map.is_blocked_for_commit_generated(SlotNumber(slot)) {
-                device_map.mark_allocated(SlotNumber(slot), 1);
-            }
-        }
-    }
+    leave_only_unblocked_slots_on_each_device(
+        &mut pool.session.as_mut().expect("挂着的会话").allocator,
+        UNBLOCKED_SLOTS_LEFT_FOR_THE_WALK_AFTER_THE_UNLINK_INTENT_TO_BE_STUCK,
+    );
     pool.write_time_seconds += 1;
     let refusal = pool
         .publish(
@@ -6365,3 +6362,382 @@
         "覆盖写之前的走空",
     );
 }
+
+/// 每块盘的单元区里没挡的槽标成已分配、只留 `unblocked_slots_left` 个（注入级，位图上占着、不写记录；照
+/// `admission_raises_the_floor_before_refusing.rs` 的 `pool_ready_to_unlink_with_few_unblocked_slots`）。
+fn leave_only_unblocked_slots_on_each_device(
+    allocator: &mut PoolAllocator,
+    unblocked_slots_left: u64,
+) {
+    for device_map in &mut allocator.devices {
+        let unit_area_start = device_map.unit_area_start().slot().0;
+        // 迭代上界是单元区槽数；跨轮只带没挡的槽数；出口：只剩下要留的那几个。
+        for slot in unit_area_start..unit_area_start + device_map.unit_area_slots() {
+            if device_map.unblocked_slots() <= unblocked_slots_left {
+                break;
+            }
+            if !device_map.is_blocked_for_commit_generated(SlotNumber(slot)) {
+                device_map.mark_allocated(SlotNumber(slot), 1);
+            }
+        }
+    }
+}
+
+// ---------- 第 10z 批第二十四块：要出路先核盘表次序（判决 `research/prompts/m3-batch10z-vw-code-r1-main-verification.md` 第二节 K1-3、第三节） ----------
+
+/// 盘表怎么换（要出路时交进去的那一份）。
+#[derive(Clone, Copy, Debug)]
+enum DeviceTableChange {
+    /// 两块盘次序对调（身份与盘不拆开）：攻方 `research/prompts/m3-batch10z-vw-code-r1-opus-model/atk_vw_part3.rs` 的 `table-reversed`。
+    Reversed,
+    /// 多交一块本池之外的空盘。
+    OneDeviceMore,
+    /// 少交最后一块盘。
+    OneDeviceFewer,
+}
+
+/// 被宽护卡住（建 inode、前缀 40 次那一形）之后，在这一刻盘面与会话的一份拷贝上照 `table_change` 换盘表要出路：在任何读之前拒成
+/// 盘表那一种、带两份盘表，一块只进内存的盘都没建、一个盘都没读、一个改盘调用都没有，盘逐字节不变；换回挂载时那一份盘表照常预演
+/// （同一个会话原样照旧）。不核盘表次序时，次序对调的盘表在预演里推环那一串照样过（核盘表不看次序）、列推环，照出路二推完再发却被拒成盘表那一种
+/// （攻方「打中二」）；多一块、少一块在预演里推环被拒，但已经读过盘。
+fn assert_the_ways_out_with_the_device_table_changed_are_refused_without_reading_a_device(
+    table_change: DeviceTableChange,
+) {
+    let PoolStuckBehindTheWideProtection { pool, stuck, .. } =
+        pool_stuck_behind_the_wide_protection(
+            FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation {
+                overwrites_before: 40,
+            },
+        );
+    let (mut counting_pool, calls_of_the_pool) =
+        pool_failing_one_changing_call_copied_from(&pool, None);
+    let device_table_of_the_mount: Vec<DeviceIdentity> = counting_pool
+        .devices
+        .iter()
+        .map(|(identity, _)| *identity)
+        .collect();
+    let mut device_left_out = None;
+    match table_change {
+        DeviceTableChange::Reversed => counting_pool.devices.reverse(),
+        DeviceTableChange::OneDeviceMore => counting_pool.devices.push((
+            DeviceIdentity(u32::try_from(device_table_of_the_mount.len()).expect("几块盘")),
+            DeviceFailingOneChangingCall {
+                inner: SparseBlockDevice::new(
+                    counting_pool.device_bytes,
+                    PhysicalBlockSizeInBytes(512),
+                ),
+                calls_of_the_pool: Rc::clone(&calls_of_the_pool),
+            },
+        )),
+        DeviceTableChange::OneDeviceFewer => device_left_out = counting_pool.devices.pop(),
+    }
+    let device_table_handed_in: Vec<DeviceIdentity> = counting_pool
+        .devices
+        .iter()
+        .map(|(identity, _)| *identity)
+        .collect();
+    assert_ne!(
+        device_table_handed_in, device_table_of_the_mount,
+        "{table_change:?}：交进去的盘表与挂载时的不同（阳性对照）"
+    );
+    let image_before = counting_pool.image();
+    match counting_pool
+        .session()
+        .ways_out_of_the_wide_protection(&counting_pool.devices, &stuck)
+    {
+        Err(WaysOutOfTheWideProtectionRefused::DeviceTableOtherThanTheOneOfTheMount {
+            device_table_of_the_mount: refused_table_of_the_mount,
+            device_table_handed_in: refused_table_handed_in,
+        }) => assert_eq!(
+            (refused_table_of_the_mount, refused_table_handed_in),
+            (device_table_of_the_mount.clone(), device_table_handed_in),
+            "{table_change:?}：带挂载时那一份盘表与交进去的那一份"
+        ),
+        other => panic!(
+            "{table_change:?}：盘表与挂载时的不同：在预演之前拒成盘表那一种，而交回 {other:?}"
+        ),
+    }
+    assert_eq!(
+        (
+            calls_of_the_pool.probes_of_the_physical_block_size.get(),
+            calls_of_the_pool.reads.get(),
+            calls_of_the_pool.calls.get()
+        ),
+        (0, 0, 0),
+        "{table_change:?}：不预演：一块只进内存的盘都没建、一个盘都没读、一个改盘调用都没有"
+    );
+    assert!(
+        counting_pool.image() == image_before,
+        "{table_change:?}：盘逐字节不变"
+    );
+    match table_change {
+        DeviceTableChange::Reversed => counting_pool.devices.reverse(),
+        DeviceTableChange::OneDeviceMore => {
+            counting_pool.devices.pop();
+        }
+        DeviceTableChange::OneDeviceFewer => counting_pool
+            .devices
+            .push(device_left_out.expect("少交的那一块留着")),
+    }
+    ways_out_of(
+        &counting_pool,
+        &stuck,
+        &format!("{table_change:?} 之后换回挂载时那一份盘表"),
+    );
+}
+
+#[test]
+fn asking_for_the_ways_out_with_the_two_devices_in_reversed_order_is_refused_without_reading_a_device(
+) {
+    assert_the_ways_out_with_the_device_table_changed_are_refused_without_reading_a_device(
+        DeviceTableChange::Reversed,
+    );
+}
+
+#[test]
+fn asking_for_the_ways_out_with_one_device_more_than_the_mount_is_refused_without_reading_a_device()
+{
+    assert_the_ways_out_with_the_device_table_changed_are_refused_without_reading_a_device(
+        DeviceTableChange::OneDeviceMore,
+    );
+}
+
+#[test]
+fn asking_for_the_ways_out_with_one_device_fewer_than_the_mount_is_refused_without_reading_a_device(
+) {
+    assert_the_ways_out_with_the_device_table_changed_are_refused_without_reading_a_device(
+        DeviceTableChange::OneDeviceFewer,
+    );
+}
+
+// ---------- 第 10z 批第二十四块：walkSkipped（预演里只剩走空那一支恒放得下）要红（判决第二节 K3） ----------
+
+/// 正推 `research/prompts/m3-batch10z-vw-code-r1-sonnet-model/src/vw/part10_k3.rs` 的 `vw_k3_walk_after_the_push_sweep` 那一形
+/// （k = 4）：C 那一圈整个换成从现行那一版起第几次发布写的那一槽。
+const PUBLISHES_UNTIL_THE_RING_SLOT_HOLDING_THE_INJECTED_ISOLATION: u64 = 4;
+/// 那一槽每块盘记着几个隔离着的空闲槽。
+const ISOLATED_FREE_SLOTS_THE_INJECTED_RING_SLOT_HOLDS_ON_EACH_DEVICE: u64 = 4;
+
+/// 攻方 K1 那一形挂上之后长 13 个单元被拒一次（会话推一串抬 F、把 F 抬到上限）；C 那一圈整个换成
+/// [`PUBLISHES_UNTIL_THE_RING_SLOT_HOLDING_THE_INJECTED_ISOLATION`] 次发布之后写的那一槽、每块盘记着几个隔离着的空闲槽
+/// （注入级；原来那一圈置的隔离位照旧留着，推环放不出它们），再把每块盘没挡的槽只留
+/// [`UNBLOCKED_SLOTS_LEFT_FOR_THE_WALK_AFTER_THE_UNLINK_INTENT_TO_BE_STUCK`] 个。unlink 第一个文件：意图发成、之后的走空被宽护卡住。
+/// 交回挂着的池、这一份报与那一个 unlink。
+fn pool_whose_unlink_is_stuck_in_the_walk_after_the_intent_behind_a_short_push() -> (
+    PoolUnderTest<SparseBlockDevice>,
+    StuckBehindTheWideProtection,
+    UserChange<'static>,
+) {
+    let crashed = pool_crashed_after_the_first_record_of_a_long_sequential_write();
+    let mut pool = crashed.mounted_and_stuck_once_by_a_write_grown_by(
+        DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY,
+    );
+    let ring_slot_holding_the_injected_isolation = target_for_publish(
+        CheckpointTxg(
+            pool.session().current.root().checkpoint_txg.0
+                + PUBLISHES_UNTIL_THE_RING_SLOT_HOLDING_THE_INJECTED_ISOLATION,
+        ),
+        pool.parameters.geometry.root_ring_slots_per_region,
+    );
+    {
+        let allocator = &mut pool.session.as_mut().expect("挂着的会话").allocator;
+        let placements = isolate_free_slots(
+            allocator,
+            ISOLATED_FREE_SLOTS_THE_INJECTED_RING_SLOT_HOLDS_ON_EACH_DEVICE,
+        );
+        allocator.note_placements_named_by_unapplied_journal_records(BTreeMap::from([(
+            ring_slot_holding_the_injected_isolation,
+            placements,
+        )]));
+        leave_only_unblocked_slots_on_each_device(
+            allocator,
+            UNBLOCKED_SLOTS_LEFT_FOR_THE_WALK_AFTER_THE_UNLINK_INTENT_TO_BE_STUCK,
+        );
+    }
+    pool.write_time_seconds += 1;
+    let unlink = UserChange::Unlink {
+        inode: InodeNumber(FIRST_INODE_NUMBER),
+        write_time_seconds: pool.write_time_seconds,
+    };
+    let refusal = pool
+        .publish(unlink, None)
+        .expect_err("没挡的槽只剩那么几个：unlink 被拒");
+    let WriteStuckBehindTheWideProtection {
+        versions_published,
+        stuck,
+        ..
+    } = stuck_behind_the_wide_protection_reported_by(refusal, "短推环那一形的 unlink");
+    assert_eq!(
+        versions_published, 1,
+        "意图那一次发成、被拒的是它之后走空的一步（阳性对照：卡在走空里）"
+    );
+    assert_eq!(
+        stuck.ring_slot_of_r, ring_slot_holding_the_injected_isolation,
+        "推环推到注入的那一槽（阳性对照：C 那一圈只剩这一槽）"
+    );
+    (pool, stuck, unlink)
+}
+
+/// unlink 意图之后的走空被宽护卡住、推环做得成、推完之后走空照旧被拒（攻方 `research/prompts/m3-batch10z-vw-code-r1-opus-output.md`
+/// 第六节、正推 K3-4 都造出了这一形）：报里记着的这次改动还剩什么只是走空；要出路时预演推环做成、推完之后走空照旧被拒，只列卸载再挂；
+/// 预演交回的拒绝与照出路二真推完、再发同一个 unlink 交回的拒绝 `Debug` 逐字相同。预演里只剩走空那一支当成恒放得下
+/// （变异 `walkSkipped`）时这一形列推环、预演说推完放得下。
+#[test]
+fn an_unlink_whose_walk_after_the_intent_stays_refused_after_a_push_that_goes_through_lists_only_unmounting(
+) {
+    let (pool, stuck, unlink) =
+        pool_whose_unlink_is_stuck_in_the_walk_after_the_intent_behind_a_short_push();
+    let remainder = remainder_of_the_change_recorded_in(&stuck);
+    assert!(
+        remainder.starts_with("remainder_of_the_change: WalkTheQueue }"),
+        "意图已发成：报里记着的这次改动还剩什么只是走空，而记的是 {remainder}"
+    );
+    let WaysOutOfTheWideProtection {
+        pushing_the_root_ring,
+        ways_out,
+    } = ways_out_of(&pool, &stuck, "短推环那一形");
+    assert_eq!(
+        ways_out.len(),
+        1,
+        "推完之后走空照旧被拒：只列卸载再挂：{ways_out:?}"
+    );
+    assert_the_rehearsed_refusal_is_the_refusal_after_pushing_the_root_ring(
+        &pool,
+        &pushing_the_root_ring,
+        unlink,
+        "短推环那一形",
+    );
+}
+
+// ---------- 第 10z 批第二十四块：报里记的这次改动还剩什么逐项钉（判决第二节 K1-2，正推 K3-3 的 8 种改坏） ----------
+
+/// 攻方 K1 那一形挂上之后长 13 个单元被拒一次（会话推一串抬 F、把 F 抬到上限），每块盘没挡的槽只留 `unblocked_slots_left` 个
+/// （`None` 不动），再发 `change`：被宽护卡住在这次改动本身（之前队列是空的、一版都没发成），交回报里记着的这次改动还剩什么。
+fn remainder_recorded_when_the_change_itself_is_stuck(
+    change: UserChange<'_>,
+    unblocked_slots_left: Option<u64>,
+    why: &str,
+) -> String {
+    let crashed = pool_crashed_after_the_first_record_of_a_long_sequential_write();
+    let mut pool = crashed.mounted_and_stuck_once_by_a_write_grown_by(
+        DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY,
+    );
+    if let Some(left) = unblocked_slots_left {
+        leave_only_unblocked_slots_on_each_device(
+            &mut pool.session.as_mut().expect("挂着的会话").allocator,
+            left,
+        );
+    }
+    let refusal = pool.publish(change, None).expect_err(why);
+    let WriteStuckBehindTheWideProtection {
+        versions_published,
+        stuck,
+        ..
+    } = stuck_behind_the_wide_protection_reported_by(refusal, why);
+    assert_eq!(
+        versions_published, 0,
+        "{why}：被拒的是这次改动本身，一版都没发成（阳性对照）"
+    );
+    remainder_of_the_change_recorded_in(&stuck)
+}
+
+/// 写入时间：与池里别的写都不同，记成 0 的改坏看得出来。
+const WRITE_TIME_SECONDS_OF_THE_CHANGE_STUCK_BEHIND_THE_WIDE_PROTECTION: u64 = 9_000_017;
+
+/// 顺序写（攻方 K1 那一形 g = 13，不注入）：报里记着走空再发这次顺序写，内容全长、写入时间逐字相同。
+#[test]
+fn a_sequential_write_stuck_behind_the_wide_protection_keeps_its_whole_content_and_write_time() {
+    let crashed = pool_crashed_after_the_first_record_of_a_long_sequential_write();
+    let content = content_of(
+        crashed
+            .bytes_of_the_file_grown_by(DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY),
+        WRITE_TIME_SECONDS_OF_THE_CHANGE_STUCK_BEHIND_THE_WIDE_PROTECTION,
+    );
+    let change = UserChange::SequentialWrite(FirstFile {
+        content: &content,
+        write_time_seconds: WRITE_TIME_SECONDS_OF_THE_CHANGE_STUCK_BEHIND_THE_WIDE_PROTECTION,
+    });
+    assert_eq!(
+        remainder_recorded_when_the_change_itself_is_stuck(change, None, "顺序写"),
+        format!(
+            "remainder_of_the_change: WalkTheQueueThenPublishTheChange(SequentialWrite {{ content: {content:?}, write_time_seconds: {} }}) }}",
+            WRITE_TIME_SECONDS_OF_THE_CHANGE_STUCK_BEHIND_THE_WIDE_PROTECTION
+        ),
+        "报里记着的是走空再发这次顺序写：种类、内容全长、写入时间逐字相同"
+    );
+}
+
+/// 覆盖写、建 inode、unlink 被宽护卡住在这次改动本身：每块盘没挡的槽只留这么几个（注入级）。
+const UNBLOCKED_SLOTS_LEFT_FOR_THE_CHANGE_ITSELF_TO_BE_STUCK: u64 = 10;
+
+/// 覆盖写：报里记着走空再发这次覆盖写，内容全长、写入时间逐字相同。
+#[test]
+fn an_overwrite_stuck_behind_the_wide_protection_keeps_its_whole_content_and_write_time() {
+    let content = content_of(
+        2999,
+        WRITE_TIME_SECONDS_OF_THE_CHANGE_STUCK_BEHIND_THE_WIDE_PROTECTION,
+    );
+    let change = UserChange::Overwrite(FirstFile {
+        content: &content,
+        write_time_seconds: WRITE_TIME_SECONDS_OF_THE_CHANGE_STUCK_BEHIND_THE_WIDE_PROTECTION,
+    });
+    assert_eq!(
+        remainder_recorded_when_the_change_itself_is_stuck(
+            change,
+            Some(UNBLOCKED_SLOTS_LEFT_FOR_THE_CHANGE_ITSELF_TO_BE_STUCK),
+            "覆盖写"
+        ),
+        format!(
+            "remainder_of_the_change: WalkTheQueueThenPublishTheChange(Overwrite {{ content: {content:?}, write_time_seconds: {} }}) }}",
+            WRITE_TIME_SECONDS_OF_THE_CHANGE_STUCK_BEHIND_THE_WIDE_PROTECTION
+        ),
+        "报里记着的是走空再发这次覆盖写：种类、内容全长、写入时间逐字相同"
+    );
+}
+
+/// 建 inode 的个数：不是 1，记错个数看得出来。
+const INODES_CREATED_BY_THE_CHANGE_STUCK_BEHIND_THE_WIDE_PROTECTION: u64 = 3;
+
+/// 建 inode：报里记着走空再建这几个 inode，个数、写入时间逐字相同。
+#[test]
+fn an_inode_creation_stuck_behind_the_wide_protection_keeps_its_count_and_write_time() {
+    let change = UserChange::NewInodes {
+        count: INODES_CREATED_BY_THE_CHANGE_STUCK_BEHIND_THE_WIDE_PROTECTION,
+        write_time_seconds: WRITE_TIME_SECONDS_OF_THE_CHANGE_STUCK_BEHIND_THE_WIDE_PROTECTION,
+    };
+    assert_eq!(
+        remainder_recorded_when_the_change_itself_is_stuck(
+            change,
+            Some(UNBLOCKED_SLOTS_LEFT_FOR_THE_CHANGE_ITSELF_TO_BE_STUCK),
+            "建 inode"
+        ),
+        format!(
+            "remainder_of_the_change: WalkTheQueueThenPublishTheChange(NewInodes {{ count: {}, write_time_seconds: {} }}) }}",
+            INODES_CREATED_BY_THE_CHANGE_STUCK_BEHIND_THE_WIDE_PROTECTION,
+            WRITE_TIME_SECONDS_OF_THE_CHANGE_STUCK_BEHIND_THE_WIDE_PROTECTION
+        ),
+        "报里记着的是走空再建这几个 inode：个数、写入时间逐字相同"
+    );
+}
+
+/// unlink 卡在意图那一次：报里记着走空再 unlink 这个 inode，inode、写入时间逐字相同。
+#[test]
+fn an_unlink_stuck_behind_the_wide_protection_at_the_intent_keeps_its_inode_and_write_time() {
+    let change = UserChange::Unlink {
+        inode: InodeNumber(FIRST_INODE_NUMBER),
+        write_time_seconds: WRITE_TIME_SECONDS_OF_THE_CHANGE_STUCK_BEHIND_THE_WIDE_PROTECTION,
+    };
+    assert_eq!(
+        remainder_recorded_when_the_change_itself_is_stuck(
+            change,
+            Some(UNBLOCKED_SLOTS_LEFT_FOR_THE_CHANGE_ITSELF_TO_BE_STUCK),
+            "unlink 意图那一次"
+        ),
+        format!(
+            "remainder_of_the_change: WalkTheQueueThenPublishTheChange(Unlink {{ inode: {:?}, write_time_seconds: {} }}) }}",
+            InodeNumber(FIRST_INODE_NUMBER),
+            WRITE_TIME_SECONDS_OF_THE_CHANGE_STUCK_BEHIND_THE_WIDE_PROTECTION
+        ),
+        "报里记着的是走空再 unlink 这个 inode：inode、写入时间逐字相同"
+    );
+}
````

第二十四块 `crates/mutations.tsv` 追加行：

````tsv
第10z批第二十四块 要出路不核盘表次序（次序对调的盘表预演照过、列推环，攻方打中二）	crates/singlefs-core/src/mounted_session.rs	        if !self\n            .parameters_and_device_table\n            .is_the_device_table_of_the_mount(devices)\n        {	        if false\n            && !self\n                .parameters_and_device_table\n                .is_the_device_table_of_the_mount(devices)\n        {	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- asking_for_the_ways_out_with_the_two_devices_in_reversed_order_is_refused_without_reading_a_device	asking_for_the_ways_out_with_the_two_devices_in_reversed_order_is_refused_without_reading_a_device
第10z批第二十四块 要出路不核盘表次序（多交一块盘：预演里读过盘才拒）	crates/singlefs-core/src/mounted_session.rs	        if !self\n            .parameters_and_device_table\n            .is_the_device_table_of_the_mount(devices)\n        {	        if false\n            && !self\n                .parameters_and_device_table\n                .is_the_device_table_of_the_mount(devices)\n        {	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- asking_for_the_ways_out_with_one_device_more_than_the_mount_is_refused_without_reading_a_device	asking_for_the_ways_out_with_one_device_more_than_the_mount_is_refused_without_reading_a_device
第10z批第二十四块 要出路不核盘表次序（少交一块盘：预演里读过盘才拒）	crates/singlefs-core/src/mounted_session.rs	        if !self\n            .parameters_and_device_table\n            .is_the_device_table_of_the_mount(devices)\n        {	        if false\n            && !self\n                .parameters_and_device_table\n                .is_the_device_table_of_the_mount(devices)\n        {	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- asking_for_the_ways_out_with_one_device_fewer_than_the_mount_is_refused_without_reading_a_device	asking_for_the_ways_out_with_one_device_fewer_than_the_mount_is_refused_without_reading_a_device
第10z批第二十四块 正推 walkSkipped 预演里只剩走空那一支恒放得下（推完之后走空照旧被拒那一形列推环）	crates/singlefs-core/src/mounted_session.rs	        RemainderOfTheUserChange::WalkTheQueue => {\n            let mut version_to_walk	        RemainderOfTheUserChange::WalkTheQueue if true => Ok(()),\n        RemainderOfTheUserChange::WalkTheQueue => {\n            let mut version_to_walk	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- an_unlink_whose_walk_after_the_intent_stays_refused_after_a_push_that_goes_through_lists_only_unmounting	an_unlink_whose_walk_after_the_intent_stays_refused_after_a_push_that_goes_through_lists_only_unmounting
第10z批第二十四块 正推 newInodesCountPlus 报里记的建 inode 个数记大	crates/singlefs-core/src/mounted_session.rs	            UserChange::NewInodes {\n                count,\n                write_time_seconds,\n            } => UserChangeKeptByTheRefusal::NewInodes {\n                count,\n	            UserChange::NewInodes {\n                count,\n                write_time_seconds,\n            } => UserChangeKeptByTheRefusal::NewInodes {\n                count: count + 1_000_000,\n	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- an_inode_creation_stuck_behind_the_wide_protection_keeps_its_count_and_write_time	an_inode_creation_stuck_behind_the_wide_protection_keeps_its_count_and_write_time
第10z批第二十四块 正推 inodesWriteTimeZero 报里记的建 inode 写入时间记成 0	crates/singlefs-core/src/mounted_session.rs	            } => UserChangeKeptByTheRefusal::NewInodes {\n                count,\n                write_time_seconds,\n	            } => UserChangeKeptByTheRefusal::NewInodes {\n                count,\n                write_time_seconds: 0,\n	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- an_inode_creation_stuck_behind_the_wide_protection_keeps_its_count_and_write_time	an_inode_creation_stuck_behind_the_wide_protection_keeps_its_count_and_write_time
第10z批第二十四块 正推 unlinkInodeZero 报里记的 unlink 的 inode 记成 0	crates/singlefs-core/src/mounted_session.rs	            } => UserChangeKeptByTheRefusal::Unlink {\n                inode,\n	            } => UserChangeKeptByTheRefusal::Unlink {\n                inode: InodeNumber(0),\n	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- an_unlink_stuck_behind_the_wide_protection_at_the_intent_keeps_its_inode_and_write_time	an_unlink_stuck_behind_the_wide_protection_at_the_intent_keeps_its_inode_and_write_time
第10z批第二十四块 正推 unlinkWriteTimeZero 报里记的 unlink 写入时间记成 0	crates/singlefs-core/src/mounted_session.rs	            } => UserChangeKeptByTheRefusal::Unlink {\n                inode,\n                write_time_seconds,\n	            } => UserChangeKeptByTheRefusal::Unlink {\n                inode,\n                write_time_seconds: 0,\n	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- an_unlink_stuck_behind_the_wide_protection_at_the_intent_keeps_its_inode_and_write_time	an_unlink_stuck_behind_the_wide_protection_at_the_intent_keeps_its_inode_and_write_time
第10z批第二十四块 正推 overwriteContentHalf 报里记的覆盖写内容只记一半	crates/singlefs-core/src/mounted_session.rs	            UserChange::Overwrite(file) => UserChangeKeptByTheRefusal::Overwrite {\n                content: file.content.to_vec(),	            UserChange::Overwrite(file) => UserChangeKeptByTheRefusal::Overwrite {\n                content: file.content[..file.content.len() / 2].to_vec(),	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- an_overwrite_stuck_behind_the_wide_protection_keeps_its_whole_content_and_write_time	an_overwrite_stuck_behind_the_wide_protection_keeps_its_whole_content_and_write_time
第10z批第二十四块 正推 overwriteWriteTimeZero 报里记的覆盖写写入时间记成 0	crates/singlefs-core/src/mounted_session.rs	            UserChange::Overwrite(file) => UserChangeKeptByTheRefusal::Overwrite {\n                content: file.content.to_vec(),\n                write_time_seconds: file.write_time_seconds,	            UserChange::Overwrite(file) => UserChangeKeptByTheRefusal::Overwrite {\n                content: file.content.to_vec(),\n                write_time_seconds: 0,	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- an_overwrite_stuck_behind_the_wide_protection_keeps_its_whole_content_and_write_time	an_overwrite_stuck_behind_the_wide_protection_keeps_its_whole_content_and_write_time
第10z批第二十四块 正推 seqWriteTimeZero 报里记的顺序写写入时间记成 0	crates/singlefs-core/src/mounted_session.rs	            UserChange::SequentialWrite(file) => UserChangeKeptByTheRefusal::SequentialWrite {\n                content: file.content.to_vec(),\n                write_time_seconds: file.write_time_seconds,	            UserChange::SequentialWrite(file) => UserChangeKeptByTheRefusal::SequentialWrite {\n                content: file.content.to_vec(),\n                write_time_seconds: 0,	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- a_sequential_write_stuck_behind_the_wide_protection_keeps_its_whole_content_and_write_time	a_sequential_write_stuck_behind_the_wide_protection_keeps_its_whole_content_and_write_time
第10z批第二十四块 正推 newInodesCountPlus 报里记的建 inode 个数记大（读回那一条）	crates/singlefs-core/src/mounted_session.rs	            UserChange::NewInodes {\n                count,\n                write_time_seconds,\n            } => UserChangeKeptByTheRefusal::NewInodes {\n                count,\n	            UserChange::NewInodes {\n                count,\n                write_time_seconds,\n            } => UserChangeKeptByTheRefusal::NewInodes {\n                count: count + 1_000_000,\n	-p singlefs-core --lib -- mounted_session::tests::each_kind_of_change_kept_by_the_refusal_reads_back_as_the_change_handed_in	each_kind_of_change_kept_by_the_refusal_reads_back_as_the_change_handed_in
第10z批第二十四块 正推 inodesWriteTimeZero 报里记的建 inode 写入时间记成 0（读回那一条）	crates/singlefs-core/src/mounted_session.rs	            } => UserChangeKeptByTheRefusal::NewInodes {\n                count,\n                write_time_seconds,\n	            } => UserChangeKeptByTheRefusal::NewInodes {\n                count,\n                write_time_seconds: 0,\n	-p singlefs-core --lib -- mounted_session::tests::each_kind_of_change_kept_by_the_refusal_reads_back_as_the_change_handed_in	each_kind_of_change_kept_by_the_refusal_reads_back_as_the_change_handed_in
第10z批第二十四块 正推 unlinkInodeZero 报里记的 unlink 的 inode 记成 0（读回那一条）	crates/singlefs-core/src/mounted_session.rs	            } => UserChangeKeptByTheRefusal::Unlink {\n                inode,\n	            } => UserChangeKeptByTheRefusal::Unlink {\n                inode: InodeNumber(0),\n	-p singlefs-core --lib -- mounted_session::tests::each_kind_of_change_kept_by_the_refusal_reads_back_as_the_change_handed_in	each_kind_of_change_kept_by_the_refusal_reads_back_as_the_change_handed_in
第10z批第二十四块 正推 unlinkWriteTimeZero 报里记的 unlink 写入时间记成 0（读回那一条）	crates/singlefs-core/src/mounted_session.rs	            } => UserChangeKeptByTheRefusal::Unlink {\n                inode,\n                write_time_seconds,\n	            } => UserChangeKeptByTheRefusal::Unlink {\n                inode,\n                write_time_seconds: 0,\n	-p singlefs-core --lib -- mounted_session::tests::each_kind_of_change_kept_by_the_refusal_reads_back_as_the_change_handed_in	each_kind_of_change_kept_by_the_refusal_reads_back_as_the_change_handed_in
第10z批第二十四块 正推 overwriteContentHalf 报里记的覆盖写内容只记一半（读回那一条）	crates/singlefs-core/src/mounted_session.rs	            UserChange::Overwrite(file) => UserChangeKeptByTheRefusal::Overwrite {\n                content: file.content.to_vec(),	            UserChange::Overwrite(file) => UserChangeKeptByTheRefusal::Overwrite {\n                content: file.content[..file.content.len() / 2].to_vec(),	-p singlefs-core --lib -- mounted_session::tests::each_kind_of_change_kept_by_the_refusal_reads_back_as_the_change_handed_in	each_kind_of_change_kept_by_the_refusal_reads_back_as_the_change_handed_in
第10z批第二十四块 正推 overwriteWriteTimeZero 报里记的覆盖写写入时间记成 0（读回那一条）	crates/singlefs-core/src/mounted_session.rs	            UserChange::Overwrite(file) => UserChangeKeptByTheRefusal::Overwrite {\n                content: file.content.to_vec(),\n                write_time_seconds: file.write_time_seconds,	            UserChange::Overwrite(file) => UserChangeKeptByTheRefusal::Overwrite {\n                content: file.content.to_vec(),\n                write_time_seconds: 0,	-p singlefs-core --lib -- mounted_session::tests::each_kind_of_change_kept_by_the_refusal_reads_back_as_the_change_handed_in	each_kind_of_change_kept_by_the_refusal_reads_back_as_the_change_handed_in
第10z批第二十四块 正推 seqWriteTimeZero 报里记的顺序写写入时间记成 0（读回那一条）	crates/singlefs-core/src/mounted_session.rs	            UserChange::SequentialWrite(file) => UserChangeKeptByTheRefusal::SequentialWrite {\n                content: file.content.to_vec(),\n                write_time_seconds: file.write_time_seconds,	            UserChange::SequentialWrite(file) => UserChangeKeptByTheRefusal::SequentialWrite {\n                content: file.content.to_vec(),\n                write_time_seconds: 0,	-p singlefs-core --lib -- mounted_session::tests::each_kind_of_change_kept_by_the_refusal_reads_back_as_the_change_handed_in	each_kind_of_change_kept_by_the_refusal_reads_back_as_the_change_handed_in
第10z批第二十四块 正推 asUserChangeSwapOverwrite 报里拷下的覆盖写读回成顺序写（读回那一条）	crates/singlefs-core/src/mounted_session.rs	            UserChangeKeptByTheRefusal::Overwrite {\n                content,\n                write_time_seconds,\n            } => UserChange::Overwrite(FirstFile {	            UserChangeKeptByTheRefusal::Overwrite {\n                content,\n                write_time_seconds,\n            } => UserChange::SequentialWrite(FirstFile {	-p singlefs-core --lib -- mounted_session::tests::each_kind_of_change_kept_by_the_refusal_reads_back_as_the_change_handed_in	each_kind_of_change_kept_by_the_refusal_reads_back_as_the_change_handed_in
第10z批第二十四块 正推 checkRequestBlock512 只进内存的盘判据里物理块写死 512（4096 盘上落在 512 边界的请求）	crates/singlefs-core/src/mounted_session.rs	        check_aligned_and_in_range(\n            offset,\n            length,\n            self.physical_block_size,	        check_aligned_and_in_range(\n            offset,\n            length,\n            PhysicalBlockSizeInBytes(512),	-p singlefs-core --lib -- mounted_session::tests::requests_on_512_byte_boundaries_off_the_4096_byte_physical_blocks_are_refused_before_anything_is_kept	requests_on_512_byte_boundaries_off_the_4096_byte_physical_blocks_are_refused_before_anything_is_kept
第10z批第二十四块 正推 checkRequestBlock512 只进内存的盘判据里物理块写死 512（4096 盘差分对拍那一条）	crates/singlefs-core/src/mounted_session.rs	        check_aligned_and_in_range(\n            offset,\n            length,\n            self.physical_block_size,	        check_aligned_and_in_range(\n            offset,\n            length,\n            PhysicalBlockSizeInBytes(512),	-p singlefs-core --lib -- mounted_session::tests::the_device_keeping_writes_in_memory_on_4096_byte_physical_blocks_agrees_with_a_device_written_through_on_a_random_request_stream	the_device_keeping_writes_in_memory_on_4096_byte_physical_blocks_agrees_with_a_device_written_through_on_a_random_request_stream
````

## 第二十五块

````diff
--- a/crates/singlefs-checker-tier/src/crash_amplification.rs
+++ b/crates/singlefs-checker-tier/src/crash_amplification.rs
@@ -4655,31 +4655,55 @@
     }
 
     /// `source`（这份源文件，或用例在它上面改过的一份）去掉测试模块之后的文本：从测试模块的头两行起，到与它配对的顶格右括号那一行止，
-    /// 连同这两处一起去掉。头两行只认代码里的：注释、字符串里写的一份不算（照原文找第一处时，块注释里写一份就把切口挪到注释那里，
-    /// 注释与真测试模块之间的代码被当成测试切掉，第 10z 批第二十三块）。
+    /// 连同这两处一起去掉。头两行只认这一种，与卡池扫源码那一份（`gpu_unit_checks.rs` 的 `code_before_the_tests_module`）同一口径、
+    /// 按行逐字：
+    /// - 相邻两整行逐字是 [`TESTS_MODULE_OPENING`] 那两行（`#[cfg(test)]`，与 `mod tests`、一个空格、一个左花括号；行首起、行尾止，不带别的空白与注释，换行是 `\n`），在代码里（注释与字面量之外）；
+    /// - 在花括号的第 0 层，也在圆括号、方括号的第 0 层；
+    /// - 前一个词（空白、注释、字面量都跳过）是分号、右花括号、右方括号或文件开头。
+    ///
+    /// 照原文找第一处时，块注释里写一份就把切口挪到注释那里，注释与真测试模块之间的代码被当成测试切掉（第 10z 批第二十三块）；
+    /// 拿一个吃掉 `#[cfg(test)]` 的宏把测试模块包进一次宏调用的圆括号里，那个「测试模块」在非测试构建里照编、里面藏的层照样可调用，
+    /// 头两行却仍在花括号第 0 层（第 10z 批第二十五块，正推 K4-1 S14），宏调用的括号里头两行前一个词是 `(`，前面先放一条以分号收尾的
+    /// 条目也还在圆括号里。这一份原先每行去掉行尾空白再比，第二十五块对齐成逐字：这两份文件是 rustfmt 出的，头两行就是这一种写法，
+    /// 别的写法（`mod tests` 后面不空格就接左花括号、写在一行、`#[ cfg ( test ) ]`、多空格、缩进、行尾带空白或注释、CRLF 换行）一律判红，是故意拒。
     ///
     /// # Panics
-    /// 代码里（花括号第 0 层、注释与字面量之外）测试模块的头两行不是恰好一处，或它之后没有收尾的顶格右括号行。
+    /// 照上面那样认的头两行不是恰好一处，或它之后没有收尾的顶格右括号行。
     fn text_outside_the_tests_module(source: &str) -> String {
         let lines: Vec<&str> = source.split('\n').collect();
         let read_lines = lines_read_past_comments_and_literals(&lines);
         let (attribute_line, module_line) = TESTS_MODULE_OPENING
             .split_once('\n')
             .expect("测试模块的头两行是两行");
+        let code_before_the_line = |line_index: usize| -> String {
+            read_lines[..line_index]
+                .iter()
+                .map(|read_line| read_line.without_comments_and_literals.as_str())
+                .collect::<Vec<&str>>()
+                .join("\n")
+        };
         let openings: Vec<usize> = read_lines
             .windows(2)
             .enumerate()
-            .filter(|(_line_index, pair)| {
-                pair[0].brace_depth_at_the_start == 0
-                    && pair[0].without_comments_and_literals.trim_end() == attribute_line
-                    && pair[1].without_comments_and_literals.trim_end() == module_line
+            .filter(|(line_index, pair)| {
+                pair[0].without_comments_and_literals == attribute_line
+                    && pair[1].without_comments_and_literals == module_line
+                    && pair[0].brace_depth_at_the_start == 0
+                    && parenthesis_and_bracket_depth_at_the_end_of(&code_before_the_line(
+                        *line_index,
+                    )) == 0
+                    && {
+                        let code_before_the_attribute_line = code_before_the_line(*line_index);
+                        let last_code = code_before_the_attribute_line.trim_end();
+                        last_code.is_empty() || last_code.ends_with([';', '\u{7d}', ']'])
+                    }
             })
             .map(|(line_index, _pair)| line_index)
             .collect();
         assert_eq!(
             openings.len(),
             1,
-            "代码里测试模块的头两行恰好一处（注释、字符串里的不算），读到的行号（从 0 数）：{openings:?}"
+            "代码里测试模块的头两行恰好一处（两整行逐字、括号第 0 层、前一个词是条目收尾；注释、字符串、宏调用括号里的不算），读到的行号（从 0 数）：{openings:?}"
         );
         let opening = openings[0];
         let closing = opening
@@ -4700,6 +4724,22 @@
             .join("\n")
     }
 
+    /// `code`（注释与字面量已换成空格）读到结尾时还没收尾的圆括号与方括号有几层。
+    ///
+    /// # Panics
+    /// 右圆括号、右方括号多于左的。
+    fn parenthesis_and_bracket_depth_at_the_end_of(code: &str) -> usize {
+        code.chars().fold(0_usize, |depth, character| {
+            if matches!(character, '(' | '[') {
+                depth + 1
+            } else if matches!(character, ')' | ']') {
+                depth.checked_sub(1).expect("右圆括号、右方括号不多于左的")
+            } else {
+                depth
+            }
+        })
+    }
+
     /// 这份源文件测试模块之外的代码行（注释行不算）。
     fn code_lines_outside_the_tests() -> Vec<String> {
         text_outside_the_tests_module(include_str!("crash_amplification.rs"))
@@ -4976,14 +5016,42 @@
             .collect()
     }
 
-    /// 叫停那种闭包的形（不问参数、交 `bool`），去掉空白；`FnMut`、`FnOnce` 与函数指针 `fn() -> bool` 也算。
-    const STOP_REQUEST_SHAPES_WITHOUT_SPACES: [&str; 4] = [
-        "Fn()->bool",
-        "FnMut()->bool",
-        "FnOnce()->bool",
-        "fn()->bool",
+    /// 叫停那种闭包的形（不问参数）到返回类型之前那一截，去掉空白；`FnMut`、`FnOnce` 与函数指针 `fn() -> …` 也算。返回类型交 `bool`
+    /// 的才算叫停（[`end_of_a_bool_return_type`]）。
+    const STOP_REQUEST_SHAPES_UP_TO_THE_RETURN_TYPE_WITHOUT_SPACES: [&str; 4] =
+        ["Fn()->", "FnMut()->", "FnOnce()->", "fn()->"];
+
+    /// 叫停那种闭包交的 `bool` 的写法，去掉空白：按路径写到 `core`、`std` 里那个原始类型的也算（第 10z 批第二十五块，正推 K4-1 N01）。
+    const WAYS_TO_WRITE_BOOL_WITHOUT_SPACES: [&str; 5] = [
+        "bool",
+        "::core::primitive::bool",
+        "core::primitive::bool",
+        "::std::primitive::bool",
+        "std::primitive::bool",
     ];
 
+    /// `text`（叫停那种闭包的形 `->` 之后那一截）以交 `bool` 的返回类型起头时，那个 `bool` 收尾之后的位置；不是就交 `None`。
+    /// 空白随意，外面套几层括号的（`(bool)`，第 10z 批第二十五块，正推 K4-1 N02）、按路径写的（[`WAYS_TO_WRITE_BOOL_WITHOUT_SPACES`]）都算，
+    /// 后面紧跟标识符的字（`boolean`）不算。
+    fn end_of_a_bool_return_type(text: &str) -> Option<usize> {
+        let is_identifier_character =
+            |character: char| character.is_alphanumeric() || character == '_';
+        let after_the_parentheses = text
+            .trim_start_matches(|character: char| character == '(' || character.is_whitespace());
+        let parentheses_length = text.len() - after_the_parentheses.len();
+        WAYS_TO_WRITE_BOOL_WITHOUT_SPACES
+            .iter()
+            .find_map(|way_to_write_bool| {
+                let bool_length =
+                    end_of_a_prefix_ignoring_whitespace(after_the_parentheses, way_to_write_bool)?;
+                (!after_the_parentheses[bool_length..]
+                    .chars()
+                    .next()
+                    .is_some_and(is_identifier_character))
+                .then_some(parentheses_length + bool_length)
+            })
+    }
+
     /// `code`（注释与字面量已换成空格）里写着叫停那种闭包的形、却不在 `places` 哪一处钉住的类型里的地方，一处一句（往前最近的 `fn`
     /// 后面那个名字连同那一行）：叫停写成 `&(dyn Fn() -> bool + Sync)` 之外的类型（多加 `Send`、`'static`，`&mut`、`&'static`，
     /// `Box<dyn …>`，`dyn Sync + Fn() -> bool`，`&dyn Fn() -> bool`，`impl`、泛型、这份源文件里的类型别名）收叫停的层，按钉住的类型找不到，
@@ -4995,7 +5063,7 @@
         let is_identifier_character =
             |character: char| character.is_alphanumeric() || character == '_';
         let mut found = Vec::new();
-        for shape in STOP_REQUEST_SHAPES_WITHOUT_SPACES {
+        for shape in STOP_REQUEST_SHAPES_UP_TO_THE_RETURN_TYPE_WITHOUT_SPACES {
             let first_word = &shape[..shape.find('(').expect("形里有左括号")];
             for position in whole_word_positions(code, first_word) {
                 let Some(shape_length) =
@@ -5003,11 +5071,7 @@
                 else {
                     continue;
                 };
-                if code[position + shape_length..]
-                    .chars()
-                    .next()
-                    .is_some_and(is_identifier_character)
-                {
+                if end_of_a_bool_return_type(&code[position + shape_length..]).is_none() {
                     continue;
                 }
                 if places.iter().any(|place| {
@@ -5354,6 +5418,11 @@
     /// 叫停写成钉住之外的类型（多加 `Send`、`'static`，`&mut`、`&'static`，类型里夹注释，`Box<dyn …>`，`dyn Sync + Fn() -> bool`，
     /// `&dyn Fn() -> bool`，`impl`、泛型、这份源文件里的类型别名，函数指针）的新层照 `Fn() -> bool` 这个形找到、判红
     /// （`the_stop_request_scan_finds_new_layers_whose_stop_type_is_written_another_way`，正推 K4-2 C02–C12）。
+    /// 第 10z 批第二十五块（第 10z 批第二十二、二十三块代码轮判决 K3、K4）：测试模块的头两行只认按行逐字、括号第 0 层、前一个词是条目收尾的那一处，
+    /// 拿一个吃掉 `#[cfg(test)]` 的宏把测试模块包进宏调用的圆括号里（正推 K4-1 S14）、包着的假测试模块里藏一层（S19）都红，
+    /// 恰好一处的上界与第 0 层两道各有用例钉（正推 K3-5 W8、W9、X6、X7，`the_tests_module_is_cut_only_at_one_opening_of_two_whole_lines_at_the_top_level_after_an_item`）；
+    /// 叫停闭包的返回类型写成 `::core::primitive::bool`、`std::primitive::bool`、套括号的 `(bool)` 的新层照形找到、红
+    /// （正推 K4-1 N01、N02，`the_stop_request_scan_finds_new_layers_whose_stop_returns_bool_by_a_path_or_in_parentheses`）。
     ///
     /// 拦不住的写法，一种一条：
     /// - 同一行原样交下去而下一层的函数体里不用（下一层由它自己那一条管；第 ① 段在卡上判那一层里 `stop_request` 往下怎么用由
@@ -5363,8 +5432,14 @@
     /// - 被调的那一层自己改了（短的几层整句比钉的是调谁、怎么交，不是被调的函数里做什么）；harness 档的 `StopRequestRememberedInThisRun::new` 改了；
     /// - 叫停的类型写成别的源文件、别的 crate 里定义的别名或 trait（`&crate::某处::叫停类型`），或换成不带 `Fn() -> bool` 字样的类型
     ///   （`&StopRequestRememberedInThisRun`、自己的 trait）的新层：只扫这一份源文件、只认这个形；
+    /// - 叫停闭包的返回类型写成别名（`type 别名 = bool;` 之后 `Fn() -> 别名`，正推 K4-1 N03）或类型宏（`Fn() -> 某宏!()`，N15）的新层：
+    ///   只认 `bool`、按路径写到 `core`、`std` 里的那个原始类型与外面套的括号，别名与宏展开出来的是什么，扫这一份源文件看不出；
     /// - 在函数上面加 `#[cfg]`，让照上一条另写类型的同名函数编进来（同样写成钉住的类型或这份源文件里看得见的形的，读到的名单多一层、红）；
-    /// - 长的那一层里宏展开出来的调用与绑定。
+    /// - 长的那一层里宏展开出来的调用与绑定；
+    /// - 过程宏（属性宏）挂在测试模块上、或把它包住，把测试模块编进非测试构建：切口只认头两行的写法与所在的层，宏做了什么扫源码看不见。
+    ///
+    /// 合法写法判红、是故意拒的：测试模块头两行换写法（`mod tests` 后面不空格就接左花括号、写在一行、`#[ cfg ( test ) ]`、多空格、缩进、行尾带空白或注释）
+    /// 与 CRLF 换行（[`text_outside_the_tests_module`]，两份文件是 rustfmt 出的，头两行就是一种写法）。
     #[test]
     fn every_layer_handed_the_stop_request_hands_it_on_or_asks_it() {
         assert_eq!(
@@ -5750,6 +5825,194 @@
             );
         }
     }
+
+    /// 跑 `check`，它 panic 时交 panic 的消息，不 panic 交 `None`：扫源码的用例看判法在改过的源码上红没红、红在哪一句。
+    fn panic_message_of(check: impl FnOnce() + std::panic::UnwindSafe) -> Option<String> {
+        std::panic::catch_unwind(check).err().map(|payload| {
+            payload
+                .downcast_ref::<String>()
+                .cloned()
+                .or_else(|| {
+                    payload
+                        .downcast_ref::<&str>()
+                        .map(|message| (*message).to_string())
+                })
+                .unwrap_or_default()
+        })
+    }
+
+    /// 第 10z 批第二十五块（第 10z 批第二十二、二十三块代码轮判决 K3、K4，正推 K3-5 `X6`、`X7`，K4-1 `S14`、`S19`）：真测试模块头两行之前
+    /// 写一份头两行，认不认照 [`text_outside_the_tests_module`] 那几条。宏调用圆括号里的（前一个词是左括号；里面藏一层收叫停的层，`S19`，
+    /// 那一层留在测试模块之外、照样找得到；前面先放一条以分号收尾的条目）、花括号里的（前面先放一条以分号收尾的条目）、前一个词不是条目收尾的，
+    /// 都不认，切口照旧在真测试模块；代码里两处都认的（恰好一处的上界）、真测试模块包进宏调用的圆括号（`S14`）、头两行换写法与 CRLF 换行
+    /// （按行逐字的口径，故意拒），都红在「恰好一处」那一句。扫这份源码改过的一份，不要卡。
+    #[test]
+    fn the_tests_module_is_cut_only_at_one_opening_of_two_whole_lines_at_the_top_level_after_an_item(
+    ) {
+        let source = include_str!("crash_amplification.rs");
+        let outside_the_tests = text_outside_the_tests_module(source);
+        let hidden_layer = concat!(
+            "    pub fn a_layer_hidden_in_a_wrapped_copy_of_the_tests_module(stop: &(dyn Fn() -> bool + Sync)) -> bool ",
+            "\u{7b} stop() \u{7d}\n"
+        );
+        let wrapped_copy_hiding_a_layer =
+            format!("keep_hidden!(\n{TESTS_MODULE_OPENING}\n{hidden_layer}") + "\u{7d}\n);\n";
+        for (form, copy_of_the_opening) in [
+            (
+                "宏调用圆括号里的一份，里面藏一层收叫停的层（S19）",
+                wrapped_copy_hiding_a_layer.as_str(),
+            ),
+            (
+                "宏调用圆括号里、前面先放一条以分号收尾的条目的一份",
+                "keep_hidden!(\nconst BEFORE_THE_COPY: u8 = 0;\n#[cfg(test)]\nmod tests \u{7b}\n\u{7d}\n);\n",
+            ),
+            (
+                "花括号里、前面先放一条以分号收尾的条目的一份",
+                "mod a_module_holding_a_copy \u{7b}\nconst BEFORE_THE_COPY: u8 = 0;\n#[cfg(test)]\nmod tests \u{7b}\n\u{7d}\n\u{7d}\n",
+            ),
+            (
+                "前一个词不是条目收尾的一份",
+                "static NOT_ENDED_BY_A_SEMICOLON: u8 = 0\n#[cfg(test)]\nmod tests \u{7b}\n\u{7d}\n",
+            ),
+        ] {
+            let with_a_copy = source.replacen(
+                TESTS_MODULE_OPENING,
+                &format!("{copy_of_the_opening}{TESTS_MODULE_OPENING}"),
+                1,
+            );
+            assert_eq!(
+                text_outside_the_tests_module(&with_a_copy).len(),
+                outside_the_tests.len() + copy_of_the_opening.len(),
+                "{form}不认，切口照旧在真测试模块"
+            );
+        }
+        let with_a_wrapped_copy_hiding_a_layer = source.replacen(
+            TESTS_MODULE_OPENING,
+            &format!("{wrapped_copy_hiding_a_layer}{TESTS_MODULE_OPENING}"),
+            1,
+        );
+        let found =
+            ways_a_layer_handed_the_stop_request_could_drop_it(&with_a_wrapped_copy_hiding_a_layer);
+        assert!(
+            found
+                .iter()
+                .any(|finding| finding
+                    .contains("a_layer_hidden_in_a_wrapped_copy_of_the_tests_module")),
+            "宏包着的假测试模块里藏的那一层找得到、红（S19）：{found:#?}"
+        );
+        let mut refused_sources: Vec<(&str, String)> = vec![
+            (
+                "代码里两处都认的头两行",
+                source.replacen(
+                    TESTS_MODULE_OPENING,
+                    &format!(
+                        "{TESTS_MODULE_OPENING}{TOP_LEVEL_CLOSING_BRACE_LINE}{TESTS_MODULE_OPENING}"
+                    ),
+                    1,
+                ),
+            ),
+            (
+                "真测试模块包进宏调用的圆括号（S14）",
+                source.replacen(
+                    TESTS_MODULE_OPENING,
+                    &format!("keep_hidden!(\n{TESTS_MODULE_OPENING}"),
+                    1,
+                ),
+            ),
+            (
+                "真测试模块包进宏调用的圆括号、前面先放一条以分号收尾的条目",
+                source.replacen(
+                    TESTS_MODULE_OPENING,
+                    &(String::from("keep_hidden!(\nconst BEFORE_THE_TESTS: u8 = 0;\n")
+                        + TESTS_MODULE_OPENING),
+                    1,
+                ),
+            ),
+            (
+                "真测试模块包进宏调用的花括号、前面先放一条以分号收尾的条目",
+                source.replacen(
+                    TESTS_MODULE_OPENING,
+                    &(String::from("keep_hidden! \u{7b}\nconst BEFORE_THE_TESTS: u8 = 0;\n")
+                        + TESTS_MODULE_OPENING),
+                    1,
+                ),
+            ),
+            ("CRLF 换行", source.replace('\n', "\r\n")),
+        ];
+        for (form, other_writing) in [
+            ("mod 行少一个空格", "#[cfg(test)]\nmod tests\u{7b}"),
+            ("写在一行", "#[cfg(test)] mod tests \u{7b}"),
+            ("属性里多空格", "#[ cfg ( test ) ]\nmod tests \u{7b}"),
+            ("mod 行多空格", "#[cfg(test)]\nmod  tests  \u{7b}"),
+            ("头一行缩进", "    #[cfg(test)]\nmod tests \u{7b}"),
+            ("头一行行尾带空白", "#[cfg(test)]  \nmod tests \u{7b}"),
+            ("头一行行尾带注释", "#[cfg(test)] // 单测\nmod tests \u{7b}"),
+            ("第二行行尾带注释", "#[cfg(test)]\nmod tests \u{7b} // 单测"),
+        ] {
+            refused_sources.push((
+                form,
+                source.replacen(TESTS_MODULE_OPENING, other_writing, 1),
+            ));
+        }
+        for (form, refused_source) in &refused_sources {
+            let message = panic_message_of(|| {
+                drop(text_outside_the_tests_module(refused_source));
+            });
+            assert!(
+                message
+                    .as_deref()
+                    .is_some_and(|message| message.contains("测试模块的头两行恰好一处")),
+                "{form}：切口红在「恰好一处」那一句，读到 {message:?}"
+            );
+        }
+    }
+
+    /// 第 10z 批第二十五块（第 10z 批第二十二、二十三块代码轮判决 K4，正推 K4-1 N01、N02：叫停闭包的返回类型写成 `::core::primitive::bool`、
+    /// `(bool)` 的新层，叫停原样可用，按形找不到、判绿）：返回类型按路径写到 `core`、`std` 里那个原始类型的、外面套括号的，
+    /// 照 [`end_of_a_bool_return_type`] 认成 `bool`，新层找得到、红。扫这份源码改过的一份，不要卡。
+    #[test]
+    fn the_stop_request_scan_finds_new_layers_whose_stop_returns_bool_by_a_path_or_in_parentheses()
+    {
+        let source = include_str!("crash_amplification.rs");
+        for (marker, new_layer) in [
+            (
+                "an_extra_layer_whose_stop_returns_bool_by_the_core_path",
+                "fn an_extra_layer_whose_stop_returns_bool_by_the_core_path(stop: &(dyn Fn() -> ::core::primitive::bool + Sync)) -> bool \u{7b} false \u{7d}",
+            ),
+            (
+                "an_extra_layer_whose_stop_returns_bool_by_a_relative_core_path",
+                "fn an_extra_layer_whose_stop_returns_bool_by_a_relative_core_path(stop: &(dyn Fn() -> core::primitive::bool + Sync)) -> bool \u{7b} false \u{7d}",
+            ),
+            (
+                "an_extra_layer_whose_stop_returns_bool_by_the_std_path",
+                "fn an_extra_layer_whose_stop_returns_bool_by_the_std_path(stop: &(dyn Fn() -> std :: primitive :: bool + Sync)) -> bool \u{7b} false \u{7d}",
+            ),
+            (
+                "an_extra_layer_whose_stop_returns_bool_in_parentheses",
+                "fn an_extra_layer_whose_stop_returns_bool_in_parentheses(stop: &(dyn Fn() -> (bool) + Sync)) -> bool \u{7b} false \u{7d}",
+            ),
+            (
+                "an_extra_layer_whose_stop_returns_bool_by_a_path_in_two_parentheses",
+                "fn an_extra_layer_whose_stop_returns_bool_by_a_path_in_two_parentheses(stop: &(dyn Fn() -> ( ( ::std::primitive::bool ) ) + Sync)) -> bool \u{7b} false \u{7d}",
+            ),
+            (
+                "an_extra_layer_whose_stop_is_a_function_pointer_returning_bool_in_parentheses",
+                "fn an_extra_layer_whose_stop_is_a_function_pointer_returning_bool_in_parentheses(stop: fn() -> (bool)) -> bool \u{7b} false \u{7d}",
+            ),
+            (
+                "an_extra_layer_whose_stop_is_fn_mut_returning_bool_by_a_path",
+                "fn an_extra_layer_whose_stop_is_fn_mut_returning_bool_by_a_path(stop: &mut dyn FnMut() -> core::primitive::bool) -> bool \u{7b} false \u{7d}",
+            ),
+        ] {
+            let source_with_the_new_layer = format!("{source}\n{new_layer}\n");
+            let found =
+                ways_a_layer_handed_the_stop_request_could_drop_it(&source_with_the_new_layer);
+            assert!(
+                found.iter().any(|finding| finding.contains(marker)),
+                "叫停闭包的返回类型换了写法的新层 {marker} 找得到、红：{found:#?}"
+            );
+        }
+    }
 
     /// 第 10z 批第二十三块（第 10z 批第二十、二十一块代码轮判决 K4，正推 K4-1 A01：长的那一层钉住那一行上面挂 `#[cfg(any())]`，模块里另放一个
     /// 叫 `stop_request` 的静态量，编译时那一行去掉、下面用的是静态量，整行比、层数、`let` 次数照绿）：钉住那一行前面紧挨着的（跳过注释）
--- a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
+++ b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
@@ -2216,6 +2216,11 @@
 
     /// 第 10z 批第三块（第 10z 批代码轮正推 K3 第 1 条）：不走卡池的两个取卡入口起设备都照不归卡池起（起之前记下总线号），卡池起设备只有一处、
     /// 照归卡池起。扫这份源码（测试那一段之外），不要卡。
+    ///
+    /// 拦不住的写法，一种一条（第 10z 批第二十五块写明）：
+    /// - 两个入口各取第一处：前面放一份挂 `#[cfg(any())]` 的干净副本、后面写真生效的另一份（照不归卡池起的那一句换成先 `let` 一个
+    ///   `DeviceOwnerOnTheCard` 再交进去），读到的是编译时去掉的那一份；与正推 K4-1 G14、G16 同一类，第二十五块只把借据丢掉时那一步、
+    ///   四个借卡入口与卡池模块改成按词恰好一处、不挂 `#[cfg]`（[`index_of_the_one_pinned_item_without_a_conditional_attribute`]），这两个入口没改。
     #[test]
     fn the_entries_outside_the_card_pool_start_their_devices_as_outside_the_card_pool() {
         let source = include_str!("gpu_unit_checks.rs");
@@ -2336,21 +2341,45 @@
     /// 借据丢掉时那一步整个交给 harness 档的 `wait_for_the_device_and_take_in_the_outcome`（等、接住 panic、记结局、展开中不再抛都在那边测），
     /// 等的是 [`poll_type_before_returning_a_lease`]、结局照 [`device_wait_before_returning_a_lease`] 判、记进这一笔借据；卡池的四个入口借之前
     /// 都先 poll 没等成的那几张卡（K1-2 的出路）。扫这份源码（测试那一段之外），不要卡。
+    /// 第 10z 批第二十五块（第 10z 批第二十二、二十三块代码轮判决 K4，正推 K4-1 G14、G16）：借据丢掉时那一步（按词 `Drop for GpuCardLease`）
+    /// 与四个入口（按词 `fn 名字`）各恰好一处、前面不挂 `#[cfg]`、`#[cfg_attr]`，挂 `#[cfg(any())]` 的干净副本在前、坏版本在后判红
+    /// （`the_card_pool_scans_refuse_a_second_copy_or_a_conditional_attribute_on_the_lease_return_the_booking_entries_and_the_pool_module`）。
+    ///
+    /// 拦不住的写法，一种一条：
+    /// - 坏版本的借据丢掉时那一步把类型写成路径或别名（`impl Drop for self::GpuCardLease`、`type 别名 = GpuCardLease;`），干净的那一份
+    ///   不靠挂在它上面的属性去掉（包进一个展开成空的宏调用、放在函数体里）：按词只数得到干净的那一份；
+    /// - 宏（过程宏、拼名字的宏）展开出来的同名方法或 `impl Drop`，干净的那一份照上一条去掉。
     #[test]
     fn the_card_lease_hands_its_return_to_the_ledger_and_every_booking_entry_polls_first() {
-        let source = include_str!("gpu_unit_checks.rs");
+        assert_the_lease_return_and_every_booking_entry_are_pinned(include_str!(
+            "gpu_unit_checks.rs"
+        ));
+    }
+
+    /// `source`（这份源文件，或用例在它上面改过的一份）测试那一段之外，借据丢掉时那一步与卡池的四个借卡入口照钉住的那样写
+    /// （判法在 `the_card_lease_hands_its_return_to_the_ledger_and_every_booking_entry_polls_first`）。
+    ///
+    /// # Panics
+    /// 有一样没照钉住的那样写。
+    fn assert_the_lease_return_and_every_booking_entry_are_pinned(source: &str) {
         let code = code_before_the_tests_module(source);
-        let body_until = |signature: &str, closing_line: &str| -> String {
-            let start = code
-                .find(signature)
-                .unwrap_or_else(|| panic!("源码里有 {signature}"));
-            let after = &code[start + signature.len()..];
+        let tokens_with_their_offsets = rust_tokens_with_their_offsets(code);
+        let tokens: Vec<&str> = tokens_with_their_offsets
+            .iter()
+            .map(|(_offset, token)| *token)
+            .collect();
+        let body_until = |signature: &[&str], closing_line: &str| -> String {
+            let signature_index =
+                index_of_the_one_pinned_item_without_a_conditional_attribute(&tokens, signature);
+            let (last_offset, last_token) =
+                tokens_with_their_offsets[signature_index + signature.len() - 1];
+            let after = &code[last_offset + last_token.len()..];
             after[..after
                 .find(closing_line)
-                .unwrap_or_else(|| panic!("{signature} 收得了尾"))]
+                .unwrap_or_else(|| panic!("{signature:?} 收得了尾"))]
                 .to_string()
         };
-        let drop_body = body_until("impl Drop for GpuCardLease \u{7b}", "\n        \u{7d}\n");
+        let drop_body = body_until(&["Drop", "for", "GpuCardLease"], "\n        \u{7d}\n");
         for expected in [
             "wait_for_the_device_and_take_in_the_outcome(",
             "&mut self.booking,",
@@ -2365,14 +2394,14 @@
             );
         }
         for entry in [
-            "fn lease_card(",
-            "pub fn lease_the_first_card_with_room_now(",
-            "pub fn book_the_scratch_of_the_cards_with_room_now(",
-            "pub fn card_rooms_now_and_when_empty_of(",
+            "lease_card",
+            "lease_the_first_card_with_room_now",
+            "book_the_scratch_of_the_cards_with_room_now",
+            "card_rooms_now_and_when_empty_of",
         ] {
             // 第 10z 批第十一块（第 10z 批第七八九块代码轮判决 K5，正推六·二：那一句注释掉、包进 `if false` 照绿）：注释行不算，
             // 那一句要整行、缩进十六格（方法体顶层的语句），包进别的块里的缩进更深、同一行里另有别的不是整行
-            let body = body_until(entry, "\n            \u{7d}\n");
+            let body = body_until(&["fn", entry], "\n            \u{7d}\n");
             let code_lines: Vec<&str> = body
                 .lines()
                 .filter(|line| !line.trim_start().starts_with("//"))
@@ -2610,50 +2639,86 @@
         tokens
     }
 
-    /// `source`（这份源文件，或用例在它上面改过的一份）测试模块头两行之前的那一段。头两行只认代码里的（花括号第 0 层，注释与字面量之外）：
-    /// 照原文找第一处时，注释或字符串里写一份头两行就把切口挪到那里，它与真测试模块之间的代码哪一条扫源码的用例都看不见
-    /// （第 10z 批第二十三块，攻方 K4-2；崩溃放量那一份是 `crash_amplification.rs` 的 `text_outside_the_tests_module`）。
+    /// `source`（这份源文件，或用例在它上面改过的一份）测试模块头两行之前的那一段。头两行只认这一种，与崩溃放量那一份
+    /// （`crash_amplification.rs` 的 `text_outside_the_tests_module`）同一口径、按行逐字：
+    /// - 相邻两整行逐字是 [`TESTS_MODULE_OPENING`] 那两行（`#[cfg(test)]`，与 `mod tests`、一个空格、一个左花括号；行首起、行尾止，不带别的空白与注释，换行是 `\n`），在代码里（注释与字面量之外）；
+    /// - 在花括号的第 0 层，也在圆括号、方括号的第 0 层；
+    /// - 前一个词（空白、注释、字面量都跳过）是分号、右花括号、右方括号或文件开头。
+    ///
+    /// 照原文找第一处时，注释或字符串里写一份头两行就把切口挪到那里，它与真测试模块之间的代码哪一条扫源码的用例都看不见（第 10z 批第二十三块，
+    /// 攻方 K4-2）；拿一个吃掉 `#[cfg(test)]` 的宏把测试模块包进一次宏调用的圆括号里，那个「测试模块」在非测试构建里照编，头两行却仍在
+    /// 花括号第 0 层（第 10z 批第二十五块，正推 K4-1 S14），宏调用的括号里头两行前一个词是 `(`，前面先放一条以分号收尾的条目也还在
+    /// 圆括号里。卡池这一份原先按词认头两行，第二十五块对齐成按行逐字：这两份文件是 rustfmt 出的，头两行就是这一种写法，
+    /// 别的写法（`mod tests` 后面不空格就接左花括号、写在一行、`#[ cfg ( test ) ]`、多空格、缩进、行尾带空白或注释、CRLF 换行）一律判红，是故意拒。
     ///
     /// # Panics
-    /// 代码里测试模块的头两行不是恰好一处。
+    /// 照上面那样认的头两行不是恰好一处。
     fn code_before_the_tests_module(source: &str) -> &str {
-        let opening_tokens = rust_tokens_without_comments_and_literals(TESTS_MODULE_OPENING);
         let tokens = rust_tokens_with_their_offsets(source);
-        let mut depth = 0_usize;
+        let mut brace_depth = 0_usize;
+        let mut parenthesis_and_bracket_depth = 0_usize;
         let mut openings = Vec::new();
         for (index, (offset, token)) in tokens.iter().enumerate() {
-            if depth == 0
-                && tokens[index..]
-                    .iter()
-                    .map(|(_offset, later_token)| *later_token)
-                    .take(opening_tokens.len())
-                    .eq(opening_tokens.iter().copied())
+            let token_before_the_attribute = index
+                .checked_sub(1)
+                .map(|index_before| tokens[index_before].1);
+            if *token == "#"
+                && is_the_tests_module_opening_as_two_whole_lines_at(source, *offset)
+                && brace_depth == 0
+                && parenthesis_and_bracket_depth == 0
+                && token_before_the_attribute
+                    .is_none_or(|token_before| matches!(token_before, ";" | "\u{7d}" | "]"))
             {
                 openings.push(*offset);
             }
             if *token == "\u{7b}" {
-                depth += 1;
+                brace_depth += 1;
             } else if *token == "\u{7d}" {
-                depth = depth.checked_sub(1).expect("右花括号不多于左花括号");
+                brace_depth = brace_depth.checked_sub(1).expect("右花括号不多于左花括号");
+            } else if matches!(*token, "(" | "[") {
+                parenthesis_and_bracket_depth += 1;
+            } else if matches!(*token, ")" | "]") {
+                parenthesis_and_bracket_depth = parenthesis_and_bracket_depth
+                    .checked_sub(1)
+                    .expect("右圆括号、右方括号不多于左的");
             }
         }
         assert_eq!(
             openings.len(),
             1,
-            "代码里测试模块的头两行恰好一处（注释、字符串里的不算），读到的字节位置：{openings:?}"
+            "代码里测试模块的头两行恰好一处（两整行逐字、括号第 0 层、前一个词是条目收尾；注释、字符串、宏调用括号里的不算），读到的字节位置：{openings:?}"
         );
         &source[..openings[0]]
     }
 
+    /// `source` 从 `offset` 起是不是测试模块头两行那两整行：`offset` 在行首（文件开头或紧跟 `\n`），往后逐字是 [`TESTS_MODULE_OPENING`]，
+    /// 再往后是 `\n` 或文件结尾。
+    fn is_the_tests_module_opening_as_two_whole_lines_at(source: &str, offset: usize) -> bool {
+        let starts_a_line = offset == 0 || source[..offset].ends_with('\n');
+        starts_a_line
+            && source[offset..]
+                .strip_prefix(TESTS_MODULE_OPENING)
+                .is_some_and(|after_the_opening| {
+                    after_the_opening.is_empty() || after_the_opening.starts_with('\n')
+                })
+    }
+
     /// 这份源码（测试那一段之外）里卡池模块（`mod card_pool`）的词，各带它在模块里第几层花括号里：从模块的左花括号之后
     /// 到与它配对的右花括号之前，模块的条目在第 0 层（左花括号算它外面那一层，右花括号算它收回到的那一层）。
+    /// 卡池模块按词恰好一处、不挂 `#[cfg]`（[`index_of_the_one_pinned_item_without_a_conditional_attribute`]）：取第一处时，前面放一份挂 `#[cfg(any())]`、
+    /// 只有那四条 `use` 的干净卡池模块，扫的就是它，真生效的那一份里另起 `mod wgpu` 也看不见（第 10z 批第二十五块，与正推 K4-1 G14 同形）。
     fn card_pool_module_tokens_with_their_brace_depth(code: &str) -> Vec<(usize, &str)> {
         let tokens = rust_tokens_without_comments_and_literals(code);
-        let opening = tokens
-            .windows(3)
-            .position(|window| window == ["mod", "card_pool", "\u{7b}"])
-            .expect("这份源码里有卡池模块")
-            + 3;
+        let module_keyword = index_of_the_one_pinned_item_without_a_conditional_attribute(
+            &tokens,
+            &["mod", "card_pool"],
+        );
+        assert_eq!(
+            tokens.get(module_keyword + 2),
+            Some(&"\u{7b}"),
+            "卡池模块写在这一段里（mod card_pool 后面紧跟左花括号，不是 mod card_pool; 另起文件）"
+        );
+        let opening = module_keyword + 3;
         let mut depth = 0_usize;
         let mut module_tokens = Vec::new();
         for token in &tokens[opening..] {
@@ -2844,6 +2909,48 @@
         names
     }
 
+    /// `tokens`（一段代码的词）里钉住的那一个条目，交 `signature`（按词：`["fn", 名字]`、`["mod", 名字]`、`["Drop", "for", 类型]`）
+    /// 在 `tokens` 里那一处的下标。按词恰好一处，那个条目（往前到最近的 `impl`、`fn`、`mod`，再往前跳过 `const`、`async`、`unsafe`、
+    /// `extern`、`safe`）前面不挂 `#[cfg]`、`#[cfg_attr]`：取第一处时，前面先放一份挂 `#[cfg(any())]` 的干净副本、后面写真生效的坏版本，
+    /// 读到的是编译时去掉的那一份（第 10z 批第二十五块，正推 K4-1 G14、G16）；只有一份而挂着 `#[cfg]` 的，编译时整个去掉。
+    /// 与第二十三块 `use` 上挂 `#[cfg]` 判红是同一个判法（[`names_of_the_attributes_right_before`]）。
+    ///
+    /// # Panics
+    /// 按词不是恰好一处，或那一处的条目挂着 `#[cfg]`、`#[cfg_attr]`。
+    fn index_of_the_one_pinned_item_without_a_conditional_attribute(
+        tokens: &[&str],
+        signature: &[&str],
+    ) -> usize {
+        let places: Vec<usize> = tokens
+            .windows(signature.len())
+            .enumerate()
+            .filter(|(_index, window)| *window == signature)
+            .map(|(index, _window)| index)
+            .collect();
+        assert_eq!(
+            places.len(),
+            1,
+            "钉住的 {signature:?} 按词恰好一处（挂 #[cfg(any())] 的干净副本在前、真生效的在后，取第一处就读到副本），读到的词下标：{places:?}"
+        );
+        let item_keyword = tokens[..=places[0]]
+            .iter()
+            .rposition(|token| matches!(*token, "impl" | "fn" | "mod"))
+            .expect("钉住的条目由 impl、fn 或 mod 起头");
+        let qualifiers = tokens[..item_keyword]
+            .iter()
+            .rev()
+            .take_while(|token| matches!(**token, "const" | "async" | "unsafe" | "extern" | "safe"))
+            .count();
+        let attributes = names_of_the_attributes_right_before(tokens, item_keyword - qualifiers);
+        assert!(
+            !attributes
+                .iter()
+                .any(|attribute| matches!(*attribute, "cfg" | "cfg_attr")),
+            "钉住的 {signature:?} 前面挂着 {attributes:?}：#[cfg] 让它编译时去掉"
+        );
+        places[0]
+    }
+
     /// `code`（这份源码测试那一段之外，或用例造的一段）的卡池模块里，能把钉住那两段按路径首段解析的 `std`、`wgpu` 与按名字调的
     /// 那几个函数换掉的写法，一处一句；一处没有交空。判法与名单在 `the_card_pool_module_shadows_neither_std_nor_wgpu_nor_the_functions_its_pinned_bodies_call`。
     fn ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(
@@ -3010,13 +3117,21 @@
     /// - 条目这一层调宏或定义宏（`thread_local!` 也算）：宏名按文本作用域解析，卡池模块外面先定义的同名 `macro_rules!` 会顶替标准库那一个，
     ///   展开出来的条目也能叫 `std`、`wgpu`；要用就放进函数体里；
     /// - 另起模块（`#[cfg(test)] mod …` 也算）：叫 `std`、`wgpu` 的直接遮住外部 crate，别的名字的经 `#[macro_use]` 把里面的宏漏到这一层，
-    ///   `mod 名字;` 与 `#[path]` 的源码不在这一段里；单测写在这份文件的测试模块里。
+    ///   `mod 名字;` 与 `#[path]` 的源码不在这一段里；单测写在这份文件的测试模块里。函数体里的 `mod` 也拒（第 10z 批第二十五块写明，
+    ///   正推 K4-2 N33）：它只罩那个函数体、遮不住钉住那两段，是合法写法，判红是故意的，判法不分模块写在哪一层；
+    /// - `extern crate`，挂 `#[cfg(any())]` 的也拒（第 10z 批第二十五块写明，正推 K4-2）：编译时去掉的那一条遮不住什么，是合法写法，
+    ///   判红是故意的，判法不看 `extern crate` 上挂的属性。
+    ///
+    /// 测试模块头两行换写法（`mod tests` 后面不空格就接左花括号、写在一行、`#[ cfg ( test ) ]`、多空格、缩进、行尾带空白或注释）与 CRLF 换行，切口当场判红，
+    /// 也是故意拒（[`code_before_the_tests_module`]）。
     ///
     /// 拦不住的写法（扫这一个模块的源码看不见），一种一条：
     /// - 过程宏（派生宏、属性宏）在卡池模块里展开出来的条目；
     /// - crate 根上改 `std` 指什么（`#![no_std]` 加 `extern crate … as std`）；
     /// - `Cargo.toml` 里把 `wgpu` 换成另一个包（`package = …`）；
-    /// - 被引的那一处自己换了：`super::super` 那一层的同名函数改了，或改成从别处 `pub use` 进来。
+    /// - 被引的那一处自己换了：`super::super` 那一层的同名函数改了，或改成从别处 `pub use` 进来；
+    /// - 过程宏（属性宏）挂在测试模块上、或把它包住，把测试模块编进非测试构建：切口（[`code_before_the_tests_module`]）只认头两行的写法与
+    ///   所在的层，宏做了什么扫源码看不见。
     #[test]
     fn the_card_pool_module_shadows_neither_std_nor_wgpu_nor_the_functions_its_pinned_bodies_call()
     {
@@ -3174,6 +3289,8 @@
     /// `K5_cfg_test_mod_inside`）：故意拒的三类（为什么写在
     /// `the_card_pool_module_shadows_neither_std_nor_wgpu_nor_the_functions_its_pinned_bodies_call` 的名单里），
     /// 写的东西本身不叫 `std`、`wgpu` 也拒。不要卡。
+    /// 第 10z 批第二十五块（第 10z 批第二十二、二十三块代码轮判决 K4，正推 K4-2：合法写法判红的写明故意拒）：函数体里的 `mod`、
+    /// 挂 `#[cfg(any())]` 的 `extern crate` 两样也在这里钉成拒。
     #[test]
     fn the_card_pool_scan_refuses_globs_item_level_macros_and_inner_modules_even_when_they_name_nothing_pinned(
     ) {
@@ -3187,6 +3304,14 @@
                 "另起模块",
                 "    #[cfg(test)]\n    mod unit_checks_of_the_pool \u{7b} \u{7d}",
             ),
+            (
+                "函数体里另起模块",
+                "    fn a_helper_with_an_inner_module() -> u8 \u{7b} mod inner \u{7b}\u{7d} 0 \u{7d}",
+            ),
+            (
+                "挂 cfg 的 extern crate",
+                "    #[cfg(any())]\n    extern crate alloc;",
+            ),
         ] {
             let module = card_pool_module_with_the_pinned_uses_and(items);
             assert_ne!(
@@ -3244,6 +3369,254 @@
         }
     }
 
+    /// 跑 `check`，它 panic 时交 panic 的消息，不 panic 交 `None`：扫源码的用例看判法在改过的源码上红没红、红在哪一句。
+    fn panic_message_of(check: impl FnOnce() + std::panic::UnwindSafe) -> Option<String> {
+        std::panic::catch_unwind(check).err().map(|payload| {
+            payload
+                .downcast_ref::<String>()
+                .cloned()
+                .or_else(|| {
+                    payload
+                        .downcast_ref::<&str>()
+                        .map(|message| (*message).to_string())
+                })
+                .unwrap_or_default()
+        })
+    }
+
+    /// 第 10z 批第二十五块（第 10z 批第二十二、二十三块代码轮判决 K3、K4，正推 K3-5 `W8`、`W9`，K4-1 `S14`、`S19`）：真测试模块头两行之前
+    /// 写一份头两行，认不认照 [`code_before_the_tests_module`] 那几条。宏调用圆括号里的（前一个词是左括号，`S19`；前面先放一条以分号收尾的条目）、
+    /// 花括号里的（前面先放一条以分号收尾的条目）、前一个词不是条目收尾的，都不认，切口照旧在真测试模块；代码里两处都认的（恰好一处的上界）、
+    /// 真测试模块包进宏调用的圆括号（`S14`）、头两行换写法与 CRLF 换行（按行逐字的口径，故意拒），都红在「恰好一处」那一句。不要卡。
+    #[test]
+    fn the_card_pool_scans_cut_only_at_one_opening_of_two_whole_lines_at_the_top_level_after_an_item(
+    ) {
+        let source = include_str!("gpu_unit_checks.rs");
+        let code = code_before_the_tests_module(source);
+        for (form, copy_of_the_opening) in [
+            (
+                "宏调用圆括号里的一份（S19）",
+                "keep_hidden!(\n#[cfg(test)]\nmod tests \u{7b}\n    fn hidden_in_the_wrapped_copy() \u{7b}\u{7d}\n\u{7d}\n);\n",
+            ),
+            (
+                "宏调用圆括号里、前面先放一条以分号收尾的条目的一份",
+                "keep_hidden!(\nconst BEFORE_THE_COPY: u8 = 0;\n#[cfg(test)]\nmod tests \u{7b}\n\u{7d}\n);\n",
+            ),
+            (
+                "花括号里、前面先放一条以分号收尾的条目的一份",
+                "mod a_module_holding_a_copy \u{7b}\nconst BEFORE_THE_COPY: u8 = 0;\n#[cfg(test)]\nmod tests \u{7b}\n\u{7d}\n\u{7d}\n",
+            ),
+            (
+                "前一个词不是条目收尾的一份",
+                "static NOT_ENDED_BY_A_SEMICOLON: u8 = 0\n#[cfg(test)]\nmod tests \u{7b}\n\u{7d}\n",
+            ),
+        ] {
+            let with_a_copy = source.replacen(
+                TESTS_MODULE_OPENING,
+                &format!("{copy_of_the_opening}{TESTS_MODULE_OPENING}"),
+                1,
+            );
+            assert_eq!(
+                code_before_the_tests_module(&with_a_copy).len(),
+                code.len() + copy_of_the_opening.len(),
+                "{form}不认，切口照旧在真测试模块"
+            );
+        }
+        let mut refused_sources: Vec<(&str, String)> = vec![
+            (
+                "代码里两处都认的头两行",
+                source.replacen(
+                    TESTS_MODULE_OPENING,
+                    &format!(
+                        "{TESTS_MODULE_OPENING}{TOP_LEVEL_CLOSING_BRACE_LINE}{TESTS_MODULE_OPENING}"
+                    ),
+                    1,
+                ),
+            ),
+            (
+                "真测试模块包进宏调用的圆括号（S14）",
+                source.replacen(
+                    TESTS_MODULE_OPENING,
+                    &format!("keep_hidden!(\n{TESTS_MODULE_OPENING}"),
+                    1,
+                ),
+            ),
+            (
+                "真测试模块包进宏调用的圆括号、前面先放一条以分号收尾的条目",
+                source.replacen(
+                    TESTS_MODULE_OPENING,
+                    &(String::from("keep_hidden!(\nconst BEFORE_THE_TESTS: u8 = 0;\n")
+                        + TESTS_MODULE_OPENING),
+                    1,
+                ),
+            ),
+            (
+                "真测试模块包进宏调用的花括号、前面先放一条以分号收尾的条目",
+                source.replacen(
+                    TESTS_MODULE_OPENING,
+                    &(String::from("keep_hidden! \u{7b}\nconst BEFORE_THE_TESTS: u8 = 0;\n")
+                        + TESTS_MODULE_OPENING),
+                    1,
+                ),
+            ),
+            ("CRLF 换行", source.replace('\n', "\r\n")),
+        ];
+        for (form, other_writing) in [
+            ("mod 行少一个空格", "#[cfg(test)]\nmod tests\u{7b}"),
+            ("写在一行", "#[cfg(test)] mod tests \u{7b}"),
+            ("属性里多空格", "#[ cfg ( test ) ]\nmod tests \u{7b}"),
+            ("mod 行多空格", "#[cfg(test)]\nmod  tests  \u{7b}"),
+            ("头一行缩进", "    #[cfg(test)]\nmod tests \u{7b}"),
+            ("头一行行尾带空白", "#[cfg(test)]  \nmod tests \u{7b}"),
+            ("头一行行尾带注释", "#[cfg(test)] // 单测\nmod tests \u{7b}"),
+            ("第二行行尾带注释", "#[cfg(test)]\nmod tests \u{7b} // 单测"),
+        ] {
+            refused_sources.push((
+                form,
+                source.replacen(TESTS_MODULE_OPENING, other_writing, 1),
+            ));
+        }
+        for (form, refused_source) in &refused_sources {
+            let message = panic_message_of(|| {
+                code_before_the_tests_module(refused_source);
+            });
+            assert!(
+                message
+                    .as_deref()
+                    .is_some_and(|message| message.contains("测试模块的头两行恰好一处")),
+                "{form}：切口红在「恰好一处」那一句，读到 {message:?}"
+            );
+        }
+    }
+
+    /// 第 10z 批第二十五块（第 10z 批第二十二、二十三块代码轮判决 K4，正推 K4-1 G14、G16：借据丢掉时那一步、四个借卡入口取第一处，前面先放一份挂
+    /// `#[cfg(any())]` 的干净副本、后面写真生效的坏版本，14 条卡池用例全绿）：借据丢掉时那一步、借卡入口、卡池模块各按词恰好一处、不挂 `#[cfg]`
+    /// （[`index_of_the_one_pinned_item_without_a_conditional_attribute`]）。干净副本在前、坏版本在后（G14、G16；干净副本不挂 `#[cfg]`、包进函数体里一次展开成空的
+    /// 宏调用的也算）与干净的卡池模块副本在前，红在「恰好一处」；
+    /// 唯一的一份借据丢掉时那一步挂 `#[cfg(any())]`，红在「前面挂着」；只有坏版本（G15、G17），照旧红在钉住的那几句。扫这份源码改过的一份，不要卡。
+    #[test]
+    fn the_card_pool_scans_refuse_a_second_copy_or_a_conditional_attribute_on_the_lease_return_the_booking_entries_and_the_pool_module(
+    ) {
+        let source = include_str!("gpu_unit_checks.rs");
+        let piece_of_the_source = |opening: &str, closing: &str| -> (usize, usize) {
+            let start = source
+                .find(opening)
+                .unwrap_or_else(|| panic!("源码里有 {opening}"));
+            let end = start
+                + source[start..]
+                    .find(closing)
+                    .unwrap_or_else(|| panic!("{opening} 收得了尾"))
+                + closing.len();
+            (start, end)
+        };
+        let replaced = |start: usize, end: usize, replacement: &str| {
+            format!("{}{replacement}{}", &source[..start], &source[end..])
+        };
+        let (drop_start, drop_end) = piece_of_the_source(
+            "        impl Drop for GpuCardLease \u{7b}\n",
+            "\n        \u{7d}\n",
+        );
+        let pinned_drop = &source[drop_start..drop_end];
+        let drop_doing_nothing = "        impl Drop for GpuCardLease \u{7b}\n            fn drop(&mut self) \u{7b}\u{7d}\n        \u{7d}\n";
+        let (entry_start, entry_end) = piece_of_the_source(
+            "            pub fn lease_the_first_card_with_room_now(",
+            "\n            \u{7d}\n",
+        );
+        let pinned_entry = &source[entry_start..entry_end];
+        let entry_without_the_poll = pinned_entry.replacen(
+            "                self.take_in_the_devices_found_idle();\n",
+            "",
+            1,
+        );
+        assert_ne!(
+            entry_without_the_poll, pinned_entry,
+            "造的坏借卡入口少了借之前那一步"
+        );
+        for (form, changed_source, expected_red) in [
+            (
+                "干净的借据丢掉时那一步挂 cfg 在前、坏版本在后（G14）",
+                replaced(
+                    drop_start,
+                    drop_end,
+                    &format!("        #[cfg(any())]\n{pinned_drop}{drop_doing_nothing}"),
+                ),
+                "按词恰好一处",
+            ),
+            (
+                "干净的借据丢掉时那一步包进函数体里一次展开成空的宏调用在前、坏版本在后",
+                replaced(
+                    drop_start,
+                    drop_end,
+                    &(String::from(
+                        "        fn a_place_for_the_clean_copy() \u{7b}\n            expands_to_nothing! \u{7b}\n",
+                    ) + pinned_drop
+                        + "            \u{7d}\n        \u{7d}\n"
+                        + drop_doing_nothing),
+                ),
+                "按词恰好一处",
+            ),
+            (
+                "只有坏的借据丢掉时那一步（G15）",
+                replaced(drop_start, drop_end, drop_doing_nothing),
+                "借据丢掉时那一步要有一处",
+            ),
+            (
+                "唯一的一份借据丢掉时那一步挂 cfg",
+                replaced(
+                    drop_start,
+                    drop_end,
+                    &format!("        #[cfg(any())]\n{pinned_drop}"),
+                ),
+                "前面挂着",
+            ),
+            (
+                "干净的借卡入口挂 cfg 在前、坏版本在后（G16）",
+                replaced(
+                    entry_start,
+                    entry_end,
+                    &format!("            #[cfg(any())]\n{pinned_entry}{entry_without_the_poll}"),
+                ),
+                "按词恰好一处",
+            ),
+            (
+                "只有坏的借卡入口（G17）",
+                replaced(entry_start, entry_end, &entry_without_the_poll),
+                "借之前先 poll 没等成的那几张卡",
+            ),
+        ] {
+            let message = panic_message_of(|| {
+                assert_the_lease_return_and_every_booking_entry_are_pinned(&changed_source);
+            });
+            assert!(
+                message
+                    .as_deref()
+                    .is_some_and(|message| message.contains(expected_red)),
+                "{form}：红在「{expected_red}」那一句，读到 {message:?}"
+            );
+        }
+        let pool_opening = "    mod card_pool \u{7b}\n";
+        let with_a_clean_pool_copy_first = source.replacen(
+            pool_opening,
+            &format!(
+                "    #[cfg(any())]\n{}{pool_opening}",
+                card_pool_module_with_the_pinned_uses_and("")
+            ),
+            1,
+        );
+        let message = panic_message_of(|| {
+            let found = ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(
+                code_before_the_tests_module(&with_a_clean_pool_copy_first),
+            );
+            drop(found);
+        });
+        assert!(
+            message
+                .as_deref()
+                .is_some_and(|message| message.contains("按词恰好一处")),
+            "干净的卡池模块挂 cfg 在前：红在「恰好一处」那一句，读到 {message:?}"
+        );
+    }
+
     /// 第 10z 批第二十三块（第 10z 批第二十、二十一块代码轮判决 K4，正推 K4-1 B12、B13：钉住那两段调的 `device_found_idle_by_a_poll` 改由一条挂
     /// `#[cfg(any())]` 的 `use` 引进、模块里另起一个同名函数，编译时那条 `use` 去掉、按名字调的是另起的那一个，扫源码照绿）：挂 `#[cfg]`、
     /// 展开成 `cfg` 的 `#[cfg_attr]`、`pub(crate)` 之前的 `#[cfg]`，都红。不要卡。
````

第二十五块 `crates/mutations.tsv` 追加行：

````tsv
第10z批第二十五块 GPU 取卡 K3 卡池扫源码测试模块头两行不核恰好一处的上界（代码里两处头两行取第一处，正推 K3-5 W8）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	            openings.len(),	            openings.len().min(1),	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_scans_cut_only_at_one_opening_of_two_whole_lines_at_the_top_level_after_an_item	the_card_pool_scans_cut_only_at_one_opening_of_two_whole_lines_at_the_top_level_after_an_item
第10z批第二十五块 GPU 取卡 K3 卡池扫源码测试模块头两行不核花括号第 0 层（花括号里前面以分号收尾的一份也认，正推 K3-5 W9）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	                && brace_depth == 0	                && (brace_depth == 0 || true)	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_scans_cut_only_at_one_opening_of_two_whole_lines_at_the_top_level_after_an_item	the_card_pool_scans_cut_only_at_one_opening_of_two_whole_lines_at_the_top_level_after_an_item
第10z批第二十五块 GPU 取卡 K4 卡池扫源码测试模块头两行不核圆括号方括号第 0 层（宏调用圆括号里前面以分号收尾的一份也认，正推 K4-1 S14 同形）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	                && parenthesis_and_bracket_depth == 0	                && (parenthesis_and_bracket_depth == 0 || true)	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_scans_cut_only_at_one_opening_of_two_whole_lines_at_the_top_level_after_an_item	the_card_pool_scans_cut_only_at_one_opening_of_two_whole_lines_at_the_top_level_after_an_item
第10z批第二十五块 GPU 取卡 K4 卡池扫源码测试模块头两行不看前一个词（前一个词不是条目收尾的一份也认）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	.is_none_or(|token_before| matches!(token_before, ";"	.is_none_or(|token_before| true || matches!(token_before, ";"	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_scans_cut_only_at_one_opening_of_two_whole_lines_at_the_top_level_after_an_item	the_card_pool_scans_cut_only_at_one_opening_of_two_whole_lines_at_the_top_level_after_an_item
第10z批第二十五块 GPU 取卡 K4 卡池扫源码测试模块头两行改回按词认（mod tests{、写在一行、行尾带空白、CRLF 又认作头两行，与崩溃放量按行逐字的口径不一致，正推 K4-2）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	                && is_the_tests_module_opening_as_two_whole_lines_at(source, *offset)	                && tokens[index..].iter().map(|(_offset, later_token)| *later_token).take(10).eq(rust_tokens_without_comments_and_literals(TESTS_MODULE_OPENING))	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_scans_cut_only_at_one_opening_of_two_whole_lines_at_the_top_level_after_an_item	the_card_pool_scans_cut_only_at_one_opening_of_two_whole_lines_at_the_top_level_after_an_item
第10z批第二十五块 GPU 取卡 K4 卡池扫源码钉住的条目按词不核恰好一处（挂 cfg 的干净副本在前、坏版本在后取第一处，正推 K4-1 G14、G16）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	            places.len(),	            places.len().min(1),	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_scans_refuse_a_second_copy_or_a_conditional_attribute_on_the_lease_return_the_booking_entries_and_the_pool_module	the_card_pool_scans_refuse_a_second_copy_or_a_conditional_attribute_on_the_lease_return_the_booking_entries_and_the_pool_module
第10z批第二十五块 GPU 取卡 K4 卡池扫源码钉住的条目不看挂的 cfg（唯一的一份借据丢掉时那一步挂 #[cfg(any())] 编译时去掉漏过）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	                .any(|attribute| matches!(*attribute, "cfg" | "cfg_attr")),	                .any(|attribute| false && matches!(*attribute, "cfg" | "cfg_attr")),	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_scans_refuse_a_second_copy_or_a_conditional_attribute_on_the_lease_return_the_booking_entries_and_the_pool_module	the_card_pool_scans_refuse_a_second_copy_or_a_conditional_attribute_on_the_lease_return_the_booking_entries_and_the_pool_module
第10z批第二十五块 GPU 取卡 K4 卡池扫源码借据丢掉时那一步与借卡入口改回取第一处（挂 cfg 的干净副本在前漏过，正推 K4-1 G14、G16）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	                index_of_the_one_pinned_item_without_a_conditional_attribute(&tokens, signature);	                tokens.windows(signature.len()).position(|window| window == signature).expect("变异：取第一处");	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_scans_refuse_a_second_copy_or_a_conditional_attribute_on_the_lease_return_the_booking_entries_and_the_pool_module	the_card_pool_scans_refuse_a_second_copy_or_a_conditional_attribute_on_the_lease_return_the_booking_entries_and_the_pool_module
第10z批第二十五块 GPU 取卡 K4 卡池扫源码卡池模块改回取第一处（挂 cfg 的干净卡池模块副本在前、真模块里另起 mod wgpu 漏过）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	        let module_keyword = index_of_the_one_pinned_item_without_a_conditional_attribute(\n            &tokens,\n            &["mod", "card_pool"],\n        );	        let module_keyword = tokens.windows(2).position(|window| window == ["mod", "card_pool"]).expect("变异：取第一处");	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_scans_refuse_a_second_copy_or_a_conditional_attribute_on_the_lease_return_the_booking_entries_and_the_pool_module	the_card_pool_scans_refuse_a_second_copy_or_a_conditional_attribute_on_the_lease_return_the_booking_entries_and_the_pool_module
第10z批第二十五块 GPU 取卡 K4 卡池扫源码只拒模块这一层的 mod（故意拒的函数体里的 mod 放过，正推 K4-2 N33）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	        if tokens.contains(&"mod") {	        if module_tokens.iter().any(|(depth, token)| *depth == 0 && *token == "mod") {	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_scan_refuses_globs_item_level_macros_and_inner_modules_even_when_they_name_nothing_pinned	the_card_pool_scan_refuses_globs_item_level_macros_and_inner_modules_even_when_they_name_nothing_pinned
第10z批第二十五块 崩溃放量 K3 测试模块头两行不核恰好一处的上界（代码里两处头两行取第一处，正推 K3-5 X6）	crates/singlefs-checker-tier/src/crash_amplification.rs	            openings.len(),	            openings.len().min(1),	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::the_tests_module_is_cut_only_at_one_opening_of_two_whole_lines_at_the_top_level_after_an_item	the_tests_module_is_cut_only_at_one_opening_of_two_whole_lines_at_the_top_level_after_an_item
第10z批第二十五块 崩溃放量 K3 测试模块头两行不核花括号第 0 层（花括号里前面以分号收尾的一份也认，正推 K3-5 X7）	crates/singlefs-checker-tier/src/crash_amplification.rs	                    && pair[0].brace_depth_at_the_start == 0	                    && (pair[0].brace_depth_at_the_start == 0 || true)	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::the_tests_module_is_cut_only_at_one_opening_of_two_whole_lines_at_the_top_level_after_an_item	the_tests_module_is_cut_only_at_one_opening_of_two_whole_lines_at_the_top_level_after_an_item
第10z批第二十五块 崩溃放量 K4 测试模块头两行不核圆括号方括号第 0 层（宏调用圆括号里前面以分号收尾的一份也认，正推 K4-1 S14 同形）	crates/singlefs-checker-tier/src/crash_amplification.rs	                    && parenthesis_and_bracket_depth_at_the_end_of(&code_before_the_line(	                    && 0 * parenthesis_and_bracket_depth_at_the_end_of(&code_before_the_line(	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::the_tests_module_is_cut_only_at_one_opening_of_two_whole_lines_at_the_top_level_after_an_item	the_tests_module_is_cut_only_at_one_opening_of_two_whole_lines_at_the_top_level_after_an_item
第10z批第二十五块 崩溃放量 K4 测试模块头两行不看前一个词（前一个词不是条目收尾的一份也认）	crates/singlefs-checker-tier/src/crash_amplification.rs	last_code.is_empty() || last_code.ends_with(	true || last_code.ends_with(	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::the_tests_module_is_cut_only_at_one_opening_of_two_whole_lines_at_the_top_level_after_an_item	the_tests_module_is_cut_only_at_one_opening_of_two_whole_lines_at_the_top_level_after_an_item
第10z批第二十五块 崩溃放量 K4 测试模块头一行改回去掉行尾空白再比（行尾带空白或注释的头一行又认，与卡池按行逐字的口径不一致，正推 K4-2）	crates/singlefs-checker-tier/src/crash_amplification.rs	                pair[0].without_comments_and_literals == attribute_line	                pair[0].without_comments_and_literals.trim_end() == attribute_line	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::the_tests_module_is_cut_only_at_one_opening_of_two_whole_lines_at_the_top_level_after_an_item	the_tests_module_is_cut_only_at_one_opening_of_two_whole_lines_at_the_top_level_after_an_item
第10z批第二十五块 崩溃放量 K4 叫停返回类型只认 bool 这一个写法（::core::primitive::bool 的新层漏过，正推 K4-1 N01）	crates/singlefs-checker-tier/src/crash_amplification.rs	        WAYS_TO_WRITE_BOOL_WITHOUT_SPACES\n            .iter()\n            .find_map(	        WAYS_TO_WRITE_BOOL_WITHOUT_SPACES\n            .iter()\n            .take(1)\n            .find_map(	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::the_stop_request_scan_finds_new_layers_whose_stop_returns_bool_by_a_path_or_in_parentheses	the_stop_request_scan_finds_new_layers_whose_stop_returns_bool_by_a_path_or_in_parentheses
第10z批第二十五块 崩溃放量 K4 叫停返回类型不跳外面套的括号（(bool) 的新层漏过，正推 K4-1 N02）	crates/singlefs-checker-tier/src/crash_amplification.rs	.trim_start_matches(|character: char| character == '(' || character.is_whitespace());	.trim_start_matches(char::is_whitespace);	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::the_stop_request_scan_finds_new_layers_whose_stop_returns_bool_by_a_path_or_in_parentheses	the_stop_request_scan_finds_new_layers_whose_stop_returns_bool_by_a_path_or_in_parentheses
````
