
// ───────────── b30 正推腿 M3 探针：整盘镜像 CRC-32C 钉值的第二条路 ─────────────
// 在 T_time 写 5（秒）的那一版上：先照原样现算整盘 CRC（应是旧钉值 4_048_568_891），再把每块盘每个系统配置槽偏移 [433, 437) 写成 5000、重封，
// 再算整盘 CRC：应等于新钉值 1_329_175_690。在 T_time 写 5000 的那一版（入库版）上：直接现算，应同样等于新钉值。
#[test]
fn b30_m3_image_crc_by_patching_the_old_slots_or_by_the_new_code() {
    let mut pool = pool_after_the_first_file(JOURNAL_RING_DEFAULT_BYTES, FOUR_GIBIBYTES);
    let as_built = crc32_of_the_images(&pool.devices, pool.device_bytes);
    let first_slot = system_configuration_slot_bytes(&pool.devices, DeviceIdentity(0), 0, &pool.parameters);
    let t_time_as_built = u32::from_le_bytes(first_slot[433..437].try_into().unwrap());
    let parameters = pool.parameters.clone();
    // 改之前每块盘写过的扇区（扇区号 → 512 字节），改之后逐扇区比：哪些扇区变了、写过的扇区集合有没有变
    let snapshot = |devices: &[(DeviceIdentity, SparseBlockDevice)], device_bytes: u64| -> Vec<(u32, u64, Vec<u8>)> {
        let mut out = Vec::new();
        for (identity, device) in devices {
            for sector in device.image.written_sectors_in(DeviceOffsetInBytes(0), device_bytes) {
                out.push((identity.0, sector, device.image.read(DeviceOffsetInBytes(sector * SECTOR_BYTES), usize::try_from(SECTOR_BYTES).expect("512"))));
            }
        }
        out
    };
    let before = snapshot(&pool.devices, pool.device_bytes);
    rewrite_every_system_configuration_slot(&mut pool.devices, &parameters, |slot| {
        slot[433..437].copy_from_slice(&5000u32.to_le_bytes());
    });
    let after = snapshot(&pool.devices, pool.device_bytes);
    let same_sector_set = before.iter().map(|(d, s, _)| (*d, *s)).collect::<Vec<_>>() == after.iter().map(|(d, s, _)| (*d, *s)).collect::<Vec<_>>();
    let changed: Vec<(u32, u64)> = before.iter().zip(after.iter()).filter(|(a, b)| a.2 != b.2).map(|(a, _)| (a.0, a.1)).collect();
    println!("B30SECTORS written_sectors={} same_sector_set_before_and_after={same_sector_set} changed_sectors={changed:?}", before.len());
    let patched = crc32_of_the_images(&pool.devices, pool.device_bytes);
    println!("B30IMAGES t_time_as_built={t_time_as_built} crc_as_built={as_built} crc_after_writing_5000_into_every_slot={patched}");
}
