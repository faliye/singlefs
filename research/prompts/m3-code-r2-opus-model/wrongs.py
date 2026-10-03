# m3-code-r2 攻方（Opus）的错实现表：每项是一串 (文件, 原文, 替换文)，原文在文件里必须恰好命中一次。
M = "crates/singlefs-core/src/mount.rs"
B = "crates/singlefs-core/src/block_device.rs"
REREAD = "    let reread = reading_other_than_the_version_root()?;\n"
ROOT_FN = "fn root_of_the_version_not_in_its_ring_slot_on_the_device<"
WRONGS = {
    # r1 的 A：缺根那一半只查 region 0 那块盘（对照：H2 盘 1 那条该红）
    "A_only_region_zero_device": [
        (M, "    let device_of_the_ring_slot = parameters_of_the_pool.region_devices\n        [usize::try_from(ring_slot.region).expect(\"区域号是 txg 对区域数取模，小于 3\")];\n",
            "    let device_of_the_ring_slot = parameters_of_the_pool.region_devices[0];\n"),
    ],
    # r1 的 E：根槽里自证得过就算是它（对照：H2 转圈那条该红）
    "E_any_self_verified_root": [
        (M, "            if root.instance == instance && root.checkpoint_txg == checkpoint_txg =>\n",
            "            if true || root.instance == instance && root.checkpoint_txg == checkpoint_txg =>\n"),
    ],
    # 变异表 C585 第一行：去掉根槽那一半（对照）
    "T01_no_root_half": [
        (M, "        root_of_the_version,\n    );\n    root_not_in_its_ring_slot.map(",
            "        root_of_the_version,\n    )\n    .filter(|_| false);\n    root_not_in_its_ring_slot.map("),
    ],
    # 新：把区域号当盘号（区域 2 归盘 0，被当成没有的盘 2）
    "RG_region_number_as_device": [
        (M, "    let device_of_the_ring_slot = parameters_of_the_pool.region_devices\n        [usize::try_from(ring_slot.region).expect(\"区域号是 txg 对区域数取模，小于 3\")];\n",
            "    let device_of_the_ring_slot = DeviceIdentity(u32::try_from(ring_slot.region).expect(\"小\"));\n"),
    ],
    # 新：AnotherRoot 只比实例不比 txg（题面点名）
    "IO_instance_only": [
        (M, "            if root.instance == instance && root.checkpoint_txg == checkpoint_txg =>\n",
            "            if root.instance == instance =>\n"),
    ],
    # 新：会话发布与可写挂载重读，抬 F、准入抬 F、卸载、管理员回退（caller_inputs_agreeing_with_the_disk）不重读
    "RP_no_reread_in_the_three_other_entries": [
        (M, ROOT_FN, "thread_local! { static R2_NO_REREAD: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }\n" + ROOT_FN),
        (M, REREAD, "    let reread = if R2_NO_REREAD.with(std::cell::Cell::get) {\n        first_reading\n    } else {\n        reading_other_than_the_version_root()?\n    };\n"),
        (M, "    let devices_behind =\n        devices_behind_the_current_version(devices, &parameters_of_the_pool, current);\n",
            "    R2_NO_REREAD.with(|flag| flag.set(true));\n    let devices_behind =\n        devices_behind_the_current_version(devices, &parameters_of_the_pool, current);\n    R2_NO_REREAD.with(|flag| flag.set(false));\n"),
    ],
    # 新：挂着之后四个入口重读两次（读三次），可写挂载照旧读两次
    "R3_reread_twice_after_the_mount": [
        (M, ROOT_FN, "thread_local! { static R2_THIRD: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }\n" + ROOT_FN),
        (M, REREAD, REREAD + "    let reread = if R2_THIRD.with(std::cell::Cell::get) {\n        reading_other_than_the_version_root()?\n    } else {\n        reread\n    };\n"),
        (M, "    let mut devices_behind = Vec::new();\n    for device in reader.device_identities() {\n        let Some(newest_self_verified)",
            "    let mut devices_behind = Vec::new();\n    R2_THIRD.with(|flag| flag.set(true));\n    for device in reader.device_identities() {\n        let Some(newest_self_verified)"),
        (M, "    }\n    devices_behind\n}\n", "    }\n    R2_THIRD.with(|flag| flag.set(false));\n    devices_behind\n}\n"),
    ],
    # 新：长度 0 的早退挪到 check_request 之前（越界、没对齐的长度 0 也交 Ok）
    "Z0_zero_length_before_the_request_check": [
        (B, "        self.check_request(offset, length)?;\n        // 长度 0 不调系统调用",
            "        if length == 0 {\n            return Ok(());\n        }\n        self.check_request(offset, length)?;\n        // 长度 0 不调系统调用"),
    ],
    # 新：EINTR 只重打一次，第二次被打断就报错
    "E1_reinterrupted_punch_reported": [
        (B, "                    punch_attempts_interrupted += 1;\n",
            "                    punch_attempts_interrupted += 1;\n                    if punch_attempts_interrupted > 1 {\n                        return Err(BlockDeviceError::InputOutput(punch_error));\n                    }\n"),
    ],
}
CI_FLAG = "thread_local! { static R2_CI: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }\n"
CI_CALL = ("    let devices_behind =\n        devices_behind_the_current_version(devices, &parameters_of_the_pool, current);\n",
           "    R2_CI.with(|flag| flag.set(true));\n    let devices_behind =\n        devices_behind_the_current_version(devices, &parameters_of_the_pool, current);\n    R2_CI.with(|flag| flag.set(false));\n")
WRONGS.update({
    # 新（Y3）：抬 F、准入抬 F、卸载、管理员回退那条路（caller_inputs_agreeing_with_the_disk）上缺根那一半只查 region 0 那块盘；会话发布与可写挂载照原样
    "AC_only_region_zero_device_in_the_three_other_entries": [
        (M, ROOT_FN, CI_FLAG + ROOT_FN),
        (M, "    let device_of_the_ring_slot = parameters_of_the_pool.region_devices\n        [usize::try_from(ring_slot.region).expect(\"区域号是 txg 对区域数取模，小于 3\")];\n",
            "    let device_of_the_ring_slot = if R2_CI.with(std::cell::Cell::get) {\n        parameters_of_the_pool.region_devices[0]\n    } else {\n        parameters_of_the_pool.region_devices\n            [usize::try_from(ring_slot.region).expect(\"区域号是 txg 对区域数取模，小于 3\")]\n    };\n"),
        (M, CI_CALL[0], CI_CALL[1]),
    ],
    # 新（Y3）：同一条路上根槽里自证得过就算是它；会话发布与可写挂载照原样
    "EC_any_self_verified_root_in_the_three_other_entries": [
        (M, ROOT_FN, CI_FLAG + ROOT_FN),
        (M, "            if root.instance == instance && root.checkpoint_txg == checkpoint_txg =>\n",
            "            if R2_CI.with(std::cell::Cell::get) || root.instance == instance && root.checkpoint_txg == checkpoint_txg =>\n"),
        (M, CI_CALL[0], CI_CALL[1]),
    ],
})
# 组合（只为在 harness 全包与 core 单测上一次看几个错实现放不放得过；彼此改的行不重叠）
WRONGS["COMBO_RG_RP_Z0_E1"] = (WRONGS["RG_region_number_as_device"] + WRONGS["RP_no_reread_in_the_three_other_entries"]
                               + WRONGS["Z0_zero_length_before_the_request_check"] + WRONGS["E1_reinterrupted_punch_reported"])
WRONGS["COMBO_EC_R3"] = WRONGS["EC_any_self_verified_root_in_the_three_other_entries"] + WRONGS["R3_reread_twice_after_the_mount"]
