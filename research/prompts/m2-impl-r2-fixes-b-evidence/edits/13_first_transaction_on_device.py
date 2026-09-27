from edit_lib import edit

FT = "crates/singlefs-checker-tier/src/bin/first_transaction_on_device.rs"
edit(FT, """        let operations = stream.retained_operations();
        let new_operations = &operations[operations_before_the_fourth_version..];
        let counts_after_the_raise = device_call_counts(&raised.devices, &raised.plan);""", """        let operations = stream.retained_operations();
        let new_operations = &operations[operations_before_the_fourth_version..];
        let counts_after_the_raise = device_call_counts(&raised.devices, &raised.plan);
        // C577 之后发布 D 以一道池屏障收尾，抬 F 那一串开的新写入口开头又一道，两道之间没有写：录制器
        // （`SharedStream::push` 的末尾一串屏障去重）把后一道并掉，设备照收两次 FLUSH，所以每块盘设备一层比录制流投出的多 1。
        // 录制器并掉相邻屏障、设备收两次，归第五步 55 号六档重录时定（合入后验证一报告第八节；主 agent 定不改装置、不改判据，照今天的数改钉）。
        // 盘 0 的 12 / 13 是合入后验证一那一次跑出来的；盘 1 同一个机制推的，没跑（singlefs-checker-tier 是 checker 档，归提交时的 crash-verifier）。
        let flushes_the_recorder_merged_into_the_barrier_that_closed_publish_d: u64 = 1;""")
edit(FT, """            assert_eq!(
                u64::try_from(projected_flushes).expect("件数"),
                counted.barrier_calls + counted.force_unit_access_writes,
                "发布 D 与抬 F 两段盘 {}：录制流投出的 FLUSH 与设备一层转发的屏障 + FUA 写",
                identity.0
            );""", """            assert_eq!(
                u64::try_from(projected_flushes).expect("件数")
                    + flushes_the_recorder_merged_into_the_barrier_that_closed_publish_d,
                counted.barrier_calls + counted.force_unit_access_writes,
                "发布 D 与抬 F 两段盘 {}：录制流投出的 FLUSH 加录制器并掉的那一道，与设备一层转发的屏障 + FUA 写",
                identity.0
            );""")
edit(FT, """        let windows = [
            (
                "second_transaction",
                &operations[operations_after_the_first_transaction
                    ..operations_after_the_second_transaction],
            ),
            (
                "reopen_and_writable_mount_and_third_transaction",
                &operations[operations_after_the_second_transaction..],
            ),
        ];""", """        // 每一段里录制器并掉、设备照收的屏障数（每块盘）：C577 之后发布 B 以一道池屏障收尾，重开之后取号那个新写入口开头又一道，
        // 两道之间没有写，录制器（`SharedStream::push` 的末尾一串屏障去重）把后一道并进上一段，设备在这一段照收一次 FLUSH。
        // 录制器并掉相邻屏障、设备收两次，归第五步 55 号六档重录时定（合入后验证一报告第八节；主 agent 定不改装置、不改判据，照今天的数改钉）。
        // 发布 B 那一段是 0（合入后验证一那一次两块盘都过了）；重开那一段盘 0 的 16 / 17 是那一次跑出来的，盘 1 同一个机制推的，没跑
        // （singlefs-checker-tier 是 checker 档，归提交时的 crash-verifier）。
        let windows = [
            (
                "second_transaction",
                &operations[operations_after_the_first_transaction
                    ..operations_after_the_second_transaction],
                0_u64,
            ),
            (
                "reopen_and_writable_mount_and_third_transaction",
                &operations[operations_after_the_second_transaction..],
                1,
            ),
        ];""")
edit(FT, """            for ((window, slice), counted) in windows
                .iter()
                .zip([second_transaction_counted, after_reopen_counted])
            {""", """            for ((window, slice, flushes_the_recorder_merged), counted) in windows
                .iter()
                .zip([second_transaction_counted, after_reopen_counted])
            {""")
edit(FT, """                assert_eq!(
                    u64::try_from(projected_flushes).expect("件数"),
                    counted.barrier_calls + counted.force_unit_access_writes,
                    "{window} 盘 {}：录制流投出的 FLUSH 与设备一层转发的屏障 + FUA 写",
                    identity.0
                );""", """                assert_eq!(
                    u64::try_from(projected_flushes).expect("件数") + flushes_the_recorder_merged,
                    counted.barrier_calls + counted.force_unit_access_writes,
                    "{window} 盘 {}：录制流投出的 FLUSH 加录制器并掉的那几道，与设备一层转发的屏障 + FUA 写",
                    identity.0
                );""")
print("13_first_transaction_on_device done")
edit(FT, """    /// 录制器并掉的屏障若在设备上是两次，这里先红，虚机档的逐项比对不会冤判。""", """    /// 录制器并掉的屏障若在设备上是两次，这里先红，虚机档的逐项比对不会冤判。C577 之后重开那一段开头就有这样一道（段里写明的
    /// `flushes_the_recorder_merged`，照今天的数钉），怎么收归第五步 55 号六档重录时定。""")
print("13 doc done")
