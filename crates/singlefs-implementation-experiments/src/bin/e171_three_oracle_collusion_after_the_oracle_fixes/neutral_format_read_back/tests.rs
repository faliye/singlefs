//! 中性格式（E170（O3规约执行器第一版） 登记 5.3）的文件名、读回与读回后重建镜像的摘要的单测。

use super::*;

fn bytes_write(device: u32, offset: u64, bytes: Vec<u8>) -> RetainedWrite {
    RetainedWrite {
        device: DeviceIdentity(device),
        kind: StepKind::SystemConfigurationSlot,
        is_force_unit_access: false,
        offset: DeviceOffsetInBytes(offset),
        contents: WrittenContents::Bytes(bytes),
    }
}

fn neutral(
    device: u32,
    kind: RecordedOperationKind,
    offset: u64,
    length: u64,
    payload_start: Option<usize>,
) -> NeutralOperation {
    NeutralOperation {
        device,
        kind,
        offset,
        length,
        payload_start,
    }
}

/// 小池上同一个状态两条路算摘要：`CrashImage` 直接读（带撕裂镜像的枚举写表）与中性格式按落地向量重建，两份逐字节相同；
/// 落地向量按写序号是「2 2 0 1」。
#[test]
fn rebuilding_from_the_neutral_format_matches_the_crash_image_digest_on_a_small_pool() {
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], 1 << 20);
    base.apply_writes(&[
        bytes_write(0, 0, vec![0x11; 512]),
        bytes_write(1, 8192, vec![0x22; 1024]),
    ]);
    let written_at_4096 = vec![0x33; 1024];
    let zeroes_over_the_base_sector = RetainedWrite {
        device: DeviceIdentity(0),
        kind: StepKind::ZeroFill,
        is_force_unit_access: false,
        offset: DeviceOffsetInBytes(0),
        contents: WrittenContents::Zeros { length: 512 },
    };
    let not_persisted = vec![0x44; 512];
    let overwrite_new = vec![0x55; 1024];
    let torn = [vec![0x55; 512], vec![0x22; 512]].concat();
    let writes = vec![
        bytes_write(0, 4096, written_at_4096.clone()),
        zeroes_over_the_base_sector,
        bytes_write(1, 0, not_persisted.clone()),
        bytes_write(1, 8192, overwrite_new.clone()),
        bytes_write(1, 8192, torn.clone()),
    ];
    let image = CrashImage {
        base: &base,
        writes: &writes,
        persisted: vec![true, true, false, false, true],
    };
    let landing = landing_vector_of(&image.persisted, 4, &BTreeMap::from([(3usize, 4usize)]));
    assert_eq!(landing, "2201");
    let payload = [written_at_4096, not_persisted, overwrite_new].concat();
    let stream = NeutralStream {
        base_sectors: BTreeMap::from([
            ((0u32, 0u64), vec![0x11; 512]),
            ((1, 16), vec![0x22; 512]),
            ((1, 17), vec![0x22; 512]),
        ]),
        operations: vec![
            neutral(0, RecordedOperationKind::Write, 4096, 1024, Some(0)),
            neutral(0, RecordedOperationKind::Barrier, 0, 0, None),
            neutral(0, RecordedOperationKind::WriteZeroes, 0, 512, None),
            neutral(1, RecordedOperationKind::Write, 0, 512, Some(1024)),
            neutral(
                1,
                RecordedOperationKind::WriteForceUnitAccess,
                8192,
                1024,
                Some(1536),
            ),
        ],
        payload,
        torn_bytes_by_write: BTreeMap::from([(3usize, (0usize, 1024usize))]),
        torn_payload: torn,
    };
    let from_the_image = crash_image_digest(&image);
    assert_eq!(rebuilt_image_digest(&stream, &landing), from_the_image);
    assert_ne!(
        rebuilt_image_digest(&stream, "2202"),
        from_the_image,
        "撕裂那一态记成持久，重建出来的是新字节，摘要必须不同"
    );
}
