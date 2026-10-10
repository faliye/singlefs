#[test]
fn zz_probe_what_each_batch25_image_reddens() {
    let set_u16_local = |bytes: &mut Vec<u8>, at: usize, v: u16| bytes[at..at + 2].copy_from_slice(&v.to_le_bytes());
    // P1 N5-a
    let (img, _) = image_with_an_abandoned_root_naming_another_object_birth_and_the_newest_instance_table_changed(
        "probe-a",
        |bytes| {
            let rc = usize::from(u16::from_le_bytes([bytes[PACKED_UNIT_RECORD_COUNT_OFFSET], bytes[PACKED_UNIT_RECORD_COUNT_OFFSET + 1]]));
            let start = INSTANCE_TABLE_FIRST_ROW_START + (rc - 1) * INSTANCE_TABLE_RECORD_BYTES;
            let chain = bytes[start..start + INSTANCE_TABLE_RECORD_BYTES].to_vec();
            let mut k2 = vec![0u8; INSTANCE_TABLE_RECORD_BYTES];
            k2[0] = INSTANCE_TABLE_RECORD_KIND_UNREGISTERED;
            bytes[start..start + INSTANCE_TABLE_RECORD_BYTES].copy_from_slice(&k2);
            bytes[start + INSTANCE_TABLE_RECORD_BYTES..start + 2 * INSTANCE_TABLE_RECORD_BYTES].copy_from_slice(&chain);
            set_u16_local(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET, (rc + 1) as u16);
            set_u16_local(bytes, PACKED_UNIT_DECLARED_LENGTH_OFFSET, ((rc + 1) * INSTANCE_TABLE_RECORD_BYTES) as u16);
        },
        true,
    );
    let v = check_pool_image(&img);
    eprintln!("PROBE N5-a violated={:?} i929={:?}", violated_invariants(&v), verdict(&img, ONE_OBJECT_BIRTH_PER_INODE_NUMBER));
    // P2 N5-c
    let (img, _) = image_with_an_abandoned_root_naming_another_object_birth_and_the_newest_instance_table_changed(
        "probe-c",
        |bytes| {
            bytes[PACKED_UNIT_PAYLOAD_CRC_OFFSET] ^= 1;
            singlefs_core::unit::seal_header_checksum(bytes, PACKED_UNIT_HEADER_END);
        },
        false,
    );
    let v = check_pool_image(&img);
    eprintln!("PROBE N5-c violated={:?} i929={:?}", violated_invariants(&v), verdict(&img, ONE_OBJECT_BIRTH_PER_INODE_NUMBER));
    // P3 N5-b (flag 2)
    for flag in [2u8, 1u8] {
        let (img, _) = image_with_an_abandoned_root_naming_another_object_birth_and_the_newest_instance_table_changed(
            "probe-b",
            |bytes| {
                let rc = usize::from(u16::from_le_bytes([bytes[PACKED_UNIT_RECORD_COUNT_OFFSET], bytes[PACKED_UNIT_RECORD_COUNT_OFFSET + 1]]));
                let start = INSTANCE_TABLE_FIRST_ROW_START + (rc - 1) * INSTANCE_TABLE_RECORD_BYTES;
                bytes[start + CHAIN_RECORD_HAS_NEXT_PAGE_OFFSET] = flag;
            },
            true,
        );
        let v = check_pool_image(&img);
        eprintln!("PROBE N5-b flag={flag} violated={:?} i929={:?}", violated_invariants(&v), verdict(&img, ONE_OBJECT_BIRTH_PER_INODE_NUMBER));
    }
    // P4 clean
    let img = image_after_a_recovery_abandoned_the_third_root_whose_inode_record_names_another_object_birth("probe-clean");
    let v = check_pool_image(&img);
    eprintln!("PROBE clean violated={:?} i929={:?}", violated_invariants(&v), verdict(&img, ONE_OBJECT_BIRTH_PER_INODE_NUMBER));
}
