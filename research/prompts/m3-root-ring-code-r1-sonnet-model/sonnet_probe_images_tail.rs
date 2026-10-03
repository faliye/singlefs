#[test]
fn probe_dump_default_ring_images_for_python_crc() {
    let pool = pool_after_the_first_file(JOURNAL_RING_DEFAULT_BYTES, FOUR_GIBIBYTES);
    let mut out = Vec::new();
    let mut count = 0u64;
    for (identity, device) in &pool.devices {
        for sector in device.image.written_sectors_in(DeviceOffsetInBytes(0), pool.device_bytes) {
            out.extend_from_slice(&identity.0.to_le_bytes());
            out.extend_from_slice(&sector.to_le_bytes());
            out.extend_from_slice(&device.image.read(DeviceOffsetInBytes(sector * SECTOR_BYTES), 512));
            count += 1;
        }
    }
    std::fs::create_dir_all("/tmp/claude-1000/m3-root-ring-code-r1-sonnet/dump").unwrap();
    std::fs::write("/tmp/claude-1000/m3-root-ring-code-r1-sonnet/dump/default-ring-fed.bin", &out).unwrap();
    println!("PROBE images: sectors={count} fed_bytes={} test-crc={}", out.len(), crc32_of_the_images(&pool.devices, pool.device_bytes));
}
