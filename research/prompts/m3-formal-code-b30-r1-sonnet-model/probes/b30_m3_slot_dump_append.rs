
// ───────────── b30 正推腿 M3 探针：把两份样本槽的原字节打成十六进制，供草稿里的 Python 独立算 CRC-32C 与摘要 ─────────────
#[test]
fn b30_m3_dump_the_two_sample_slots() {
    for (name, slot) in [
        ("at_mkfs", system_configuration_at_mkfs().to_slot()),
        ("after_the_new_pool_file_creation", system_configuration_after_the_new_pool_file_creation().to_slot()),
    ] {
        let hex: String = slot.iter().map(|b| format!("{b:02x}")).collect();
        println!("B30SLOT {name} sha256={} t_time_bytes_433_437={:?} hex={hex}", sha256_hexadecimal(&slot), &slot[433..437]);
    }
}
