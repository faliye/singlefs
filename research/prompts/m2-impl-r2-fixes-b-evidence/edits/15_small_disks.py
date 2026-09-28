"""三档小盘（A3b Q1：环 6 MiB、单元区从 1408 起、盘宽 = 起点 + 单元区槽数）上按步数钉的用例，钉值照草稿探针在新几何上现跑的数改。"""
from edit_lib import edit

AF = "crates/singlefs-harness/tests/second_transaction_supplement_two_admission_formula.rs"
edit(AF, "/// 两块盘的可用(d) 都是 −47 槽 < 0：", "/// 两块盘的可用(d) 都是 −39 槽 < 0（A3b 把小盘改成 6 MiB 环、单元区从 1408 起之前是 −47 槽）：")
edit(AF, "                    available: AvailableBytesOnOneDevice(-47 * i128::from(SLOT_BYTES)),",
         "                    available: AvailableBytesOnOneDevice(-39 * i128::from(SLOT_BYTES)),")
edit(AF, "        \"取号之前两块盘都短：可用 −47 槽，挂载这一刻不另要需求\"",
         "        \"取号之前两块盘都短：可用 −39 槽，挂载这一刻不另要需求\"")

FR = "crates/singlefs-harness/tests/a_floor_raise_refused_for_space_counts_as_short_of_space.rs"
edit(FR, "/// 覆盖写 18 次之后第 19 次：用户那次发布取不到落点，会话推抬 F，那一串两次空发布在预演里第 2 次也取不到落点。",
         "/// 覆盖写 19 次之后第 20 次：用户那次发布取不到落点，会话推抬 F，那一串三次空发布在预演里第 1 次就取不到落点\n"
         "/// （A3b 把小盘改成 6 MiB 环、单元区从 1408 起之前是「18 次之后第 19 次、两次空发布里第 2 次」：分配记录树按绝对槽号按位置寻址，\n"
         "/// 单元区挪了位置，每次发布重写的节点跟着变，撞墙的步数与那一串的次数都变；草稿探针在新几何上逐次覆盖写现跑的数）。")
edit(FR, "    let mut pool = pool_after_overwrites_with_the_space_admission_switched_off(18);",
         "    let mut pool = pool_after_overwrites_with_the_space_admission_switched_off(19);")
edit(FR, "        .expect_err(\"第 19 次覆盖写：单元区满了，推抬 F 也推不出空间\");",
         "        .expect_err(\"第 20 次覆盖写：单元区满了，推抬 F 也推不出空间\");")
edit(FR, """            MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite {
                refused_publish_in_the_sequence: 2,
                publishes_in_the_sequence: 2,
                cause: PublishError::PlacementRefused {
                    refusal: PlacementRefusal::NoFreeSlotOnAnyDevice,
                    ..
                },
            }
        ),
        "抬 F 那一串两次空发布在预演里第 2 次取不到落点：{raise_refusal:?}\"""", """            MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite {
                refused_publish_in_the_sequence: 1,
                publishes_in_the_sequence: 3,
                cause: PublishError::PlacementRefused {
                    refusal: PlacementRefusal::NoFreeSlotOnAnyDevice,
                    ..
                },
            }
        ),
        "抬 F 那一串三次空发布在预演里第 1 次就取不到落点：{raise_refusal:?}\"""")
print("15_small_disks part 1 done")

# admission_formula 第二条：240 槽、5 次覆盖写的盘面上挂载之后第一次直接覆盖写就放行了（新几何下固定点落点变了）。
# 草稿探针在三档、覆盖写 1–12 次、挂载之后接连直接覆盖写 1–4 次上扫：「可用落在 [普通分配 6, 需求) 之间」只有 256 槽、覆盖写 4 次、
# 挂载之后第 4 次直接覆盖写那一格（可用 11、需求 14，与改之前那一格的两个数相同）。
edit(AF, """/// 发布路径那一处（C363 (b) 判决第四节第 2 条）：两块单元区 240 槽的小盘，第一个文件之后可写挂载、覆盖写 5 次的盘面上，进程重开、
/// 可写挂载（取号之前就够），在这次挂载里直接调发布路径再覆盖写一次：需求 = 这次发布新写的全部槽（D28（挂载期承诺量） 已定项 1 接线，""", """/// 发布路径那一处（C363 (b) 判决第四节第 2 条）：两块单元区 256 槽的小盘，第一个文件之后可写挂载、覆盖写 4 次的盘面上，进程重开、
/// 可写挂载（取号之前就够），在这次挂载里直接调发布路径接连覆盖写三次（都放行），第四次：需求 = 这次发布新写的全部槽（D28（挂载期承诺量） 已定项 1 接线，""")
edit(AF, """/// 覆盖写 9 次那一份盘面（按树高、按每块盘一条叶路径计时这一格用它）在按最坏情况计之下挂载那一刻就要推抬 F、推完可用 44 槽，
/// 分不出「普通分配装得下、固定点装不下」，换成 5 次。""", """/// 覆盖写 9 次那一份盘面（按树高、按每块盘一条叶路径计时这一格用它）在按最坏情况计之下挂载那一刻就要推抬 F、推完可用 44 槽，
/// 分不出「普通分配装得下、固定点装不下」，换成 5 次；A3b 把小盘改成 6 MiB 环、单元区从 1408 起之后（分配记录树按绝对槽号按位置寻址，
/// 每次发布重写的节点跟着变），240 槽 5 次那一格第一次直接覆盖写就放行了。草稿探针在三档、覆盖写 1–12 次、挂载之后接连直接覆盖写
/// 1–4 次上扫：可用落在 [普通分配 6, 需求) 之间的只有 256 槽、覆盖写 4 次、第 4 次直接覆盖写那一格，取它。""")
edit(AF, """    let device_width = HistoryDeviceWidth::UnitAreaOf240Slots;
    let image = small_pool_image_after_overwrites(device_width, 5);
    let content = content_of(DIRECT_OVERWRITE_CONTENT_BYTES, 1);
    let publish_parameters = device_width.parameters();
""", """    let device_width = HistoryDeviceWidth::UnitAreaOf256Slots;
    let image = small_pool_image_after_overwrites(device_width, 4);
    let content = content_of(DIRECT_OVERWRITE_CONTENT_BYTES, 4);
    let publish_parameters = device_width.parameters();
""")
edit(AF, """    let current = current
        .into_file_version()
        .expect("可写挂载之后现行那一版带文件");
    let image_after_the_mount = image_of_the_devices(&devices, image.device_size_in_bytes);""", """    let mut current = current
        .into_file_version()
        .expect("可写挂载之后现行那一版带文件");
    for seed in 1..=3 {
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        current = publish_overwrite(
            &mut writer,
            &mut allocator,
            &current,
            FirstFile {
                content: &content_of(DIRECT_OVERWRITE_CONTENT_BYTES, seed),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS
                    + 600
                    + u64::try_from(seed).expect("小"),
            },
            mount_output.instance,
        )
        .unwrap_or_else(|refusal| panic!("挂载之后第 {seed} 次直接覆盖写放行：{refusal:?}"));
    }
    let image_after_the_mount = image_of_the_devices(&devices, image.device_size_in_bytes);""")
edit(AF, """            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 600,
            },
            mount_output.instance,
        )
    };
    let Err(PublishError::SpaceAdmissionRefused(refusal)) = refused else {""", """            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 604,
            },
            mount_output.instance,
        )
    };
    let Err(PublishError::SpaceAdmissionRefused(refusal)) = refused else {""")
edit(AF, """            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 600,
            },
            mount_output.instance,
        )
        .expect("同一次挂载里关掉准入，同一次覆盖写做成：拒的是式子，不是落点")""", """            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 604,
            },
            mount_output.instance,
        )
        .expect("同一次挂载里关掉准入，同一次覆盖写做成：拒的是式子，不是落点")""")

AR = "crates/singlefs-harness/tests/admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments.rs"
edit(AR, """/// 两块单元区 384 槽的小盘，第一个文件之后崩了再挂；这次挂载里直接推 80 次空发布（固定点在这次挂载开的聚簇段里 bump，根环转过之后回收，
/// 段里空出来的槽照旧只给提交内生块，D3（空间分配） 已定项 8 第 2 条），再直接把文件顺序写到 6 个数据单元，都做成。
/// 接着顺序写到 7 个单元：每块盘段外成对的空槽 0 对、段内 79 对；按字节算的式子（`admission_reading_before_a_publish`）每块盘可用 49 槽，
/// 放得下这次需求的上界（6 个单元那次新写的槽 + 多一个数据单元 2 槽 + 一次空发布的 ckpt_cost）。发布路径在走固定点之前判「这次的单元落得下」：
/// 7 个数据单元要 7 对、段外 0 对，交回 `SpaceAdmissionRefused`，每块盘报段外 0 槽 < 数据单元 14 槽；两块盘逐字节不变、分配器不动。""", """/// 两块单元区 384 槽的小盘，第一个文件之后崩了再挂；这次挂载里直接推 200 次空发布（固定点在这次挂载开的聚簇段里 bump，根环转过之后回收，
/// 段里空出来的槽照旧只给提交内生块，D3（空间分配） 已定项 8 第 2 条），再直接把文件顺序写到 5 个数据单元，都做成。
/// 接着顺序写到 6 个单元：每块盘段外成对的空槽 0 对、段内 85 对；按字节算的式子（`admission_reading_before_a_publish`）每块盘可用 60 槽，
/// 放得下这次需求的上界（5 个单元那次新写的槽 + 多一个数据单元 2 槽 + 一次空发布的 ckpt_cost，31 槽）。发布路径在走固定点之前判「这次的单元落得下」：
/// 6 个数据单元要 6 对、段外 0 对，交回 `SpaceAdmissionRefused`，每块盘报段外 0 槽 < 数据单元 12 槽；两块盘逐字节不变、分配器不动。
/// A3b 把小盘改成 6 MiB 环、单元区从 1408 起之前是「80 次空发布、写到 6 个、段外 0 对段内 79 对、可用 49 槽、7 个单元 14 槽」：分配记录树按
/// 绝对槽号按位置寻址，单元区挪了位置，每次发布重写的节点跟着变。草稿探针在空发布 60–200 次上扫：段外恰好 0 对、式子又放得下上界的是 200 次这一格
/// （140 次那一格段外也是 0 对，式子可用 30 槽 < 上界 35 槽）。""")
edit(AR, """    const EMPTY_PUBLISHES_IN_THIS_MOUNT: usize = 80;
    const DATA_UNITS_THAT_STILL_LAND: usize = 6;""", """    const EMPTY_PUBLISHES_IN_THIS_MOUNT: usize = 200;
    const DATA_UNITS_THAT_STILL_LAND: usize = 5;""")
edit(AR, """        vec![(DeviceIdentity(0), 0, 79), (DeviceIdentity(1), 0, 79)],
        "每块盘成对的空槽：这次挂载开过的聚簇段外 0 对、段内 79 对\"""", """        vec![(DeviceIdentity(0), 0, 85), (DeviceIdentity(1), 0, 85)],
        "每块盘成对的空槽：这次挂载开过的聚簇段外 0 对、段内 85 对\"""")
edit(AR, """            BytesOnOneDevice::of_slots(slots_of_data_units(7)),
        ),
        "每块盘段外 0 对（0 槽）< 7 个数据单元（14 槽）\"""", """            BytesOnOneDevice::of_slots(slots_of_data_units(6)),
        ),
        "每块盘段外 0 对（0 槽）< 6 个数据单元（12 槽）\"""")
edit(AR, """/// 落点那一道照样兜着。两块单元区 384 槽的小盘，崩了再挂、直接推 80 次空发布之后，经会话把文件顺序写到 2、3……6 个数据单元，都一次做成；
/// 写到 7 个单元：走固定点时在分配器的拷贝上取不到数据单元 0 的落点，交回 `PlacementRefused { Data(0), NoFreeSlotOnAnyDevice }`，""", """/// 落点那一道照样兜着。两块单元区 384 槽的小盘，崩了再挂、直接推 200 次空发布之后，经会话把文件顺序写到 2、3……5 个数据单元，都一次做成；
/// 写到 6 个单元：走固定点时在分配器的拷贝上取不到数据单元 0 的落点，交回 `PlacementRefused { Data(0), NoFreeSlotOnAnyDevice }`，""")
edit(AR, """/// 判别力：会话只在准入拒时推（落点被拒原样交回）时，写到 7 个单元报 `UserChangeRefused::Publish { PlacementRefused }`。""", """/// 判别力：会话只在准入拒时推（落点被拒原样交回）时，写到 6 个单元报 `UserChangeRefused::Publish { PlacementRefused }`。
/// A3b 把小盘改成 6 MiB 环、单元区从 1408 起之前是「80 次空发布、写到 6 个、第 7 个推」，与第一格同一次改（草稿探针在新几何上现跑的数）。""")
edit(AR, """    for _ in 0..80 {
        publish_an_empty_version_directly_on_the_session(&mut pool);
    }
    for data_units in 2..=6 {""", """    for _ in 0..200 {
        publish_an_empty_version_directly_on_the_session(&mut pool);
    }
    for data_units in 2..=5 {""")
edit(AR, """        .sequential_write(7 * data_unit_payload_capacity())
        .unwrap_or_else(|refusal| {
            panic!("经会话顺序写到 7 个数据单元，推过抬 F 之后做成：{refusal:?}")
        });""", """        .sequential_write(6 * data_unit_payload_capacity())
        .unwrap_or_else(|refusal| {
            panic!("经会话顺序写到 6 个数据单元，推过抬 F 之后做成：{refusal:?}")
        });""")

# a_floor_raise 第二条：17 次之后那一串三次空发布第 3 次才取不到；18 次之后两次空发布第 1 次就取不到——取 18，留住「第 1 次就取不到」。
edit(FR, """/// 覆盖写 17 次之后崩了再挂（空间准入判着）：取号之前不够，写行与暖机之后推抬 F，那一串三次空发布在预演里第 1 次就取不到落点。""", """/// 覆盖写 18 次之后崩了再挂（空间准入判着）：取号之前不够，写行与暖机之后推抬 F，那一串两次空发布在预演里第 1 次就取不到落点。
/// （A3b 把小盘改成 6 MiB 环、单元区从 1408 起之前是「17 次、三次空发布里第 1 次」；新几何下 17 次那一串是三次里第 3 次才取不到，
/// 草稿探针在 16–18 次上现跑，取 18 次，留住「第 1 次就取不到」那一形。）""")
edit(FR, """    let mut pool = pool_after_overwrites_with_the_space_admission_switched_off(17);""", """    let mut pool = pool_after_overwrites_with_the_space_admission_switched_off(18);""")
edit(FR, """            MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite {
                refused_publish_in_the_sequence: 1,
                publishes_in_the_sequence: 3,
                cause: PublishError::PlacementRefused {
                    refusal: PlacementRefusal::NoFreeSlotOnAnyDevice,
                    ..
                },
            }
        ),
        "抬 F 那一串三次空发布在预演里第 1 次就取不到落点：{raise_refusal:?}"
    );
}""", """            MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite {
                refused_publish_in_the_sequence: 1,
                publishes_in_the_sequence: 2,
                cause: PublishError::PlacementRefused {
                    refusal: PlacementRefusal::NoFreeSlotOnAnyDevice,
                    ..
                },
            }
        ),
        "抬 F 那一串两次空发布在预演里第 1 次就取不到落点：{raise_refusal:?}"
    );
}""")
print("15_small_disks done")
