# m3-code-r3 攻方（Opus）的错实现表。r2 攻方原样那几条从 m3-code-r2-opus-model/wrongs.py 原样取（不改一个字）；
# 新的两条（RU、RGC）写在下面。每项是一串 (文件, 原文, 替换文)，原文在文件里必须恰好命中一次。
import importlib.util, os
_here = os.path.dirname(os.path.abspath(__file__))
_spec = importlib.util.spec_from_file_location("wrongs_r2", os.path.join(_here, "wrongs_r2.py"))
_r2 = importlib.util.module_from_spec(_spec); _spec.loader.exec_module(_r2)
M = _r2.M
B = _r2.B
WRONGS = {name: _r2.WRONGS[name] for name in [
    "RG_region_number_as_device",
    "RP_no_reread_in_the_three_other_entries",
    "EC_any_self_verified_root_in_the_three_other_entries",
    "AC_only_region_zero_device_in_the_three_other_entries",
    "Z0_zero_length_before_the_request_check",
    "E1_reinterrupted_punch_reported",
]}
FIRST_AND_REREAD = ("    let first_reading = reading_other_than_the_version_root()?;\n"
                    "    let reread = reading_other_than_the_version_root()?;\n")
WRONGS.update({
    # 新（W2）：缺根那一半只在第一次「读不出」（设备报错）时重读；第一次读得出而自证不过（读回的字节坏了一位）、或自证得过而是另一条根，
    # 直接当成两次读到的都是它。五个入口（会话发布、可写挂载、卸载、准入抬 F、回退）同一处。
    "RU_reread_only_after_a_read_error": [
        (M, FIRST_AND_REREAD,
            "    let first_reading = reading_other_than_the_version_root()?;\n"
            "    let reread = if matches!(\n"
            "        first_reading,\n"
            "        RingSlotReadingOtherThanTheVersionRoot::Bad(crate::recovery::BadRootRingSlotReading::Unreadable)\n"
            "    ) {\n"
            "        reading_other_than_the_version_root()?\n"
            "    } else {\n"
            "        first_reading\n"
            "    };\n"),
    ],
    # 新（W2）：正常卸载、准入抬 F、管理员回退那条路（caller_inputs_agreeing_with_the_disk）上把区域号当盘号；会话发布与可写挂载照原样
    # （r2 的 RG 只在这条路上做，与 r2 的 AC、EC 同一种限定）。
    "RGC_region_number_as_device_in_the_three_other_entries": [
        (M, _r2.ROOT_FN, _r2.CI_FLAG + _r2.ROOT_FN),
        (M, "    let device_of_the_ring_slot = parameters_of_the_pool.region_devices\n        [usize::try_from(ring_slot.region).expect(\"区域号是 txg 对区域数取模，小于 3\")];\n",
            "    let device_of_the_ring_slot = if R2_CI.with(std::cell::Cell::get) {\n        DeviceIdentity(u32::try_from(ring_slot.region).expect(\"小\"))\n    } else {\n        parameters_of_the_pool.region_devices\n            [usize::try_from(ring_slot.region).expect(\"区域号是 txg 对区域数取模，小于 3\")]\n    };\n"),
        (M, _r2.CI_CALL[0], _r2.CI_CALL[1]),
    ],
})
# 组合：只为一次在 harness 全包与 core 单测上看两个新错实现放不放得过（改的行不重叠）。
WRONGS["COMBO_RU_RGC"] = WRONGS["RU_reread_only_after_a_read_error"] + WRONGS["RGC_region_number_as_device_in_the_three_other_entries"]
# W3（两块盘以外的几何）：两块盘、归属 0 / 1 / 0 上与对的同值的两种「根槽住哪块盘」算法，只在 W3 的三盘副本上套。
_DEVICE_LINE = "    let device_of_the_ring_slot = parameters_of_the_pool.region_devices\n        [usize::try_from(ring_slot.region).expect(\"区域号是 txg 对区域数取模，小于 3\")];\n"
WRONGS.update({
    # 区域号对 2 取模再查归属表（两块盘时区域 2 与区域 0 同归盘 0，同值）
    "MOD2_region_modulo_two": [
        (M, _DEVICE_LINE,
            "    let device_of_the_ring_slot = parameters_of_the_pool.region_devices\n        [usize::try_from(ring_slot.region % 2).expect(\"小\")];\n"),
    ],
    # 不读盘上的归属，写死第一版的 0 / 1 / 0（两块盘时盘上就是它，同值）
    "FV_first_version_layout_hard_coded": [
        (M, _DEVICE_LINE,
            "    let device_of_the_ring_slot = crate::make_filesystem::FIRST_VERSION_REGION_DEVICES\n        [usize::try_from(ring_slot.region).expect(\"小\")];\n"),
    ],
})
