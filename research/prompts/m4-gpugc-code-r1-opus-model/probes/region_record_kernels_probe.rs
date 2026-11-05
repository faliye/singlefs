
    /// 攻方探针（m4-gpugc-code-r1，T3）：参与的卡的上限正好是开卡与换设置判的「最小的一片」，挂载时它参与；一批里一个落点上有两条条目，
    /// 乙段最小的一片随之变大，这一批在这张卡上切不出，`core_failure_of_the_smallest_piece_not_held` 报给核心层、整理停下。
    #[test]
    fn attack_probe_two_entries_on_one_location_make_the_smallest_piece_larger_than_the_admitted_minimum() {
        use crate::gpus_of_this_host::{
            bytes_of_the_smallest_piece_every_participating_gpu_holds,
            gpu_left_out_under_its_memory_limit,
        };
        use singlefs_core::background_compute::{GpuDeviceIdentifier, GpuMemoryInBytes};
        let admitted_minimum = bytes_of_the_smallest_piece_every_participating_gpu_holds();
        let device = GpuDeviceIdentifier::of_opaque_bytes(b"0000:01:00.0".to_vec());
        let left_out_at_the_minimum = gpu_left_out_under_its_memory_limit(
            &device,
            Some(GpuMemoryInBytes(admitted_minimum)),
            admitted_minimum,
        );
        let (long_unit_bytes, long_unit_key) = data_unit_with_a_long_payload_and_key(4);
        let (_other_bytes, other_key) = data_unit_and_key(5);
        let records = vec![RegionRecordToClassify {
            entry_key: 0_usize,
            record: unreleased_record(slot_in_region(0), 2),
            unit_bytes: Some(long_unit_bytes.as_slice()),
        }];
        let room = ChunkBoundsOfAGpu {
            memory_limit: Some(GpuMemoryInBytes(admitted_minimum)),
            largest_storage_binding_in_bytes: 1 << 30,
            largest_entry_count_per_dispatch: 1 << 20,
        };
        let one_entry = vec![entry(long_unit_key.clone(), locations_at(slot_in_region(0)))];
        let one_entry_tables =
            MappingTablesForTheKernels::of_the_mapping_entries(&one_entry).expect("27 字节 key");
        let plan_with_one_entry =
            region_record_plan_on_a_gpu(&records, &one_entry, &one_entry_tables, room);
        let two_entries = vec![
            entry(long_unit_key, locations_at(slot_in_region(0))),
            entry(other_key, locations_at(slot_in_region(0))),
        ];
        let two_entries_tables =
            MappingTablesForTheKernels::of_the_mapping_entries(&two_entries).expect("27 字节 key");
        let plan_with_two_entries =
            region_record_plan_on_a_gpu(&records, &two_entries, &two_entries_tables, room);
        println!(
            "ATTACK name=batch_dependent_smallest_piece admitted_minimum={admitted_minimum} left_out_at_the_minimum={left_out_at_the_minimum:?} one_entry_plan_ok={} two_entries_plan={:?}",
            plan_with_one_entry.is_ok(),
            plan_with_two_entries.as_ref().err()
        );
        assert!(left_out_at_the_minimum.is_none(), "上限正好是最小的一片：这张卡参与");
        assert!(plan_with_one_entry.is_ok(), "一个落点一条条目：切得出");
        assert!(plan_with_two_entries.is_err(), "一个落点两条条目：这张卡上切不出");
    }
}
