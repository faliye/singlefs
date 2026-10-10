

// ───── b30 攻方腿 M3 探针（接在 unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs 的副本后面）：
// 默认环下 mkfs 到第一个文件那两块盘，四个系统配置槽的 [433, 437) 从 5000 换回 5、整槽校验和重封，整盘 CRC 要等于换钉之前的
// 4_048_568_891（30-3 之前一直入库、逐次判绿的那个值）：两块盘与换钉之前只差这四槽的 T_time 与校验和 ─────
#[test]
fn b30_opus_reverting_the_time_threshold_unit_gives_back_the_old_image_crc() {
    let mut pool = pool_after_the_first_file(JOURNAL_RING_DEFAULT_BYTES, FOUR_GIBIBYTES);
    let today = crc32_of_the_images(&pool.devices, pool.device_bytes);
    rewrite_every_system_configuration_slot(&mut pool.devices, &pool.parameters, |slot| {
        assert_eq!(&slot[433..437], &5000u32.to_le_bytes(), "今天每槽写 5000");
        slot[433..437].copy_from_slice(&5u32.to_le_bytes());
    });
    let reverted = crc32_of_the_images(&pool.devices, pool.device_bytes);
    println!("B30CRC today={today} reverted={reverted} old_pin=4048568891 new_pin={DEFAULT_RING_NEW_POOL_FILE_CREATION_IMAGES_CRC32}");
    assert_eq!(today, DEFAULT_RING_NEW_POOL_FILE_CREATION_IMAGES_CRC32);
    assert_eq!(reverted, 4_048_568_891, "换回秒之后等于旧钉值");
}
