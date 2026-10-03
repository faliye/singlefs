
// ─── m3-code-r3 攻方（Opus）原型：W3 两块盘以外的几何。接在 entries 用例文件原样之后，名字都带 r3opus ───

/// 三块盘在今天的 mkfs 上：区域归属取几种，各交回什么（panic 也接住、打出来）；两块盘时归属不是 0 / 1 / 0 交回什么。
#[test]
fn r3opus_w3_make_filesystem_on_three_devices() {
    let layouts: [(&str, usize, [DeviceIdentity; 3]); 4] = [
        ("three-012", 3, [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(2)]),
        ("three-010", 3, [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)]),
        ("three-021", 3, [DeviceIdentity(0), DeviceIdentity(2), DeviceIdentity(1)]),
        ("two-011", 2, [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(1)]),
    ];
    for (name, count, layout) in layouts {
        let mut parameters = DEVICE_WIDTH.parameters();
        parameters.region_devices = layout;
        let mut devices: SparseDevices = (0..count)
            .map(|index| {
                (
                    DeviceIdentity(u32::try_from(index).expect("小")),
                    SparseBlockDevice::new(DEVICE_WIDTH.device_bytes(), PhysicalBlockSizeInBytes(512)),
                )
            })
            .collect();
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            singlefs_core::make_filesystem::make_filesystem(&parameters, &mut devices).map(|_| ())
        }));
        let shown = match outcome {
            Ok(Ok(())) => "Ok".to_string(),
            Ok(Err(error)) => format!("Err({error:?})"),
            Err(panic) => format!(
                "panic({})",
                panic
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| panic.downcast_ref::<&str>().map(|text| (*text).to_string()))
                    .unwrap_or_default()
                    .replace('\n', " | ")
            ),
        };
        println!("W3MKFS layout={name} devices={count} outcome={}", r3opus_w3_short(shown));
    }
}

fn r3opus_w3_short(text: String) -> String {
    text.chars().take(300).collect()
}

/// 挂着的两块盘的池，盘表里多交一块盘（身份 2：一块空盘，或盘 1 盘体的拷贝），经每个入口交进去一次。
#[test]
fn r3opus_w3_a_third_device_handed_to_every_entry() {
    for third in ["blank", "copy-of-device-one"] {
        for entry in EVERY_ENTRY {
            let (mut pool, _, now) = pool_with_a_snapshot_after_the_first_file();
            let mut devices = devices_from([
                (DeviceIdentity(0), &now, DeviceIdentity(0)),
                (DeviceIdentity(1), &now, DeviceIdentity(1)),
            ]);
            let third_device = if third == "blank" {
                SparseBlockDevice::new(now.device_size_in_bytes, PhysicalBlockSizeInBytes(512))
            } else {
                copy_of_device(&now, DeviceIdentity(1))
            };
            devices.push((DeviceIdentity(2), third_device));
            let refusal = go_through(entry, &mut pool, &mut devices);
            println!("W3THIRD third={third} entry={entry:?} outcome={}", r3opus_w3_short(describe(&refusal)));
        }
    }
}

/// 两块盘（归属 0 / 1 / 0，今天唯一造得出的）与假想的三块盘（归属 0 / 1 / 2）上，P 的 txg 落哪个区域、哪个槽、哪块盘；
/// 几种「算错根槽住哪块盘」的错实现各算出哪块盘，与对的不同就是那个 txg 对它敏感（只换 P 的根住的那块盘时，查错了盘就放过）。
/// 槽在 P 之前写过没有（根环转过一圈）只看 txg：S × 3 = 24 槽，txg ≥ 24 的那一槽上一圈写过（与盘数无关）。
#[test]
fn r3opus_w3_which_device_holds_the_root_and_which_wrong_mappings_each_txg_tells_apart() {
    let slots_per_region = DEVICE_WIDTH.parameters().geometry.root_ring_slots_per_region;
    let layouts: [(&str, [DeviceIdentity; 3]); 2] = [
        ("two-010", [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)]),
        ("three-012", [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(2)]),
    ];
    for (name, layout) in layouts {
        for txg in r3opus_env_list("R3_W3_TXGS", "6,7,8,24,25,26,27,28,29,30,31,32") {
            let ring_slot = target_for_publish(CheckpointTxg(txg), slots_per_region);
            let region = usize::try_from(ring_slot.region).expect("小");
            let correct = layout[region];
            let wrongs: [(&str, DeviceIdentity); 4] = [
                ("RG", DeviceIdentity(u32::try_from(ring_slot.region).expect("小"))),
                ("AC", layout[0]),
                ("MOD2", layout[region % 2]),
                ("FIRSTVERSION", singlefs_core::make_filesystem::FIRST_VERSION_REGION_DEVICES[region]),
            ];
            let told_apart: Vec<&str> = wrongs
                .iter()
                .filter(|(_, device)| *device != correct)
                .map(|(wrong, _)| *wrong)
                .collect();
            println!(
                "W3MAP layout={name} txg={txg} region={} slot={} root_dev={} lap={} wrongs_told_apart={told_apart:?}",
                ring_slot.region,
                ring_slot.slot,
                correct.0,
                txg >= 24
            );
        }
    }
}
