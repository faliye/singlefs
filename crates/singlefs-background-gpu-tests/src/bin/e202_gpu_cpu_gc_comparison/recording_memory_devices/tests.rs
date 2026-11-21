use super::*;

fn write_of(bytes: &[u8]) -> RecordedDeviceOperation<'_> {
    RecordedDeviceOperation::Write {
        device: DeviceIdentity(0),
        offset_in_bytes: 4096,
        bytes,
        durability: WriteDurability::Plain,
    }
}

#[test]
fn the_recording_digest_tells_apart_two_writes_differing_in_one_byte() {
    let mut one_write = Sha256Digest::new();
    feed_the_operation_into_the_digest(&mut one_write, write_of(&[1, 2, 3, 4]));
    let mut other_write = Sha256Digest::new();
    feed_the_operation_into_the_digest(&mut other_write, write_of(&[1, 2, 3, 5]));
    assert_ne!(one_write.finish_as_hex(), other_write.finish_as_hex());
}

#[test]
fn the_memory_device_reads_back_what_was_written_and_zeroes_after_a_discard() {
    let recording: SharedRecordingDigest = Arc::new(Mutex::new(RecordingDigest::empty()));
    let mut devices = empty_pool_devices(4 * MEMORY_DEVICE_CHUNK_BYTES, &recording);
    let (_identity, device) = &mut devices[0];
    let written: Vec<u8> = (0..1024_u32)
        .map(|index| u8::try_from(index % 256).expect("小于 256"))
        .collect();
    let offset = DeviceOffsetInBytes(MEMORY_DEVICE_CHUNK_BYTES - 512);
    device
        .write_at(offset, &written, WriteDurability::Plain)
        .expect("跨块写");
    let mut read_back = vec![0u8; 1024];
    device.read_at(offset, &mut read_back).expect("跨块读");
    assert_eq!(read_back, written);
    device
        .discard_at(
            DeviceOffsetInBytes(MEMORY_DEVICE_CHUNK_BYTES),
            MEMORY_DEVICE_CHUNK_BYTES,
        )
        .expect("整块 discard");
    device.read_at(offset, &mut read_back).expect("读");
    assert_eq!(&read_back[..512], &written[..512]);
    assert_eq!(&read_back[512..], &[0u8; 512][..]);
    assert_eq!(recording.lock().expect("录制").operations, 2);
}

/// 一个块那么长、没有一个字节是零的内容（下标 i 处是 i mod 251 + 1 再加 `shift`）。
fn chunk_content(shift: u8) -> Vec<u8> {
    (0..MEMORY_DEVICE_CHUNK_BYTES)
        .map(|index| {
            u8::try_from(index % 251 + 1)
                .expect("小于 256")
                .wrapping_add(shift)
                .max(1)
        })
        .collect()
}

/// 共用的存法（[`ChunkStorage::SharedWhenTheBytesAreEqual`]）：两块盘先后往同一偏移写同一份字节（核心层每个单元两块盘各写一遍），
/// 按半块写两次也一样，存下的只有一份，两块盘读回的都是写的字节。
#[test]
fn two_devices_writing_the_same_bytes_at_the_same_offset_store_one_chunk() {
    let recording: SharedRecordingDigest = Arc::new(Mutex::new(RecordingDigest::empty()));
    let mut devices = empty_pool_devices(4 * MEMORY_DEVICE_CHUNK_BYTES, &recording);
    let written = chunk_content(0);
    let half = written.len() / 2;
    for (offset, bytes) in [
        (MEMORY_DEVICE_CHUNK_BYTES, &written[..half]),
        (
            MEMORY_DEVICE_CHUNK_BYTES + u64::try_from(half).expect("半块"),
            &written[half..],
        ),
    ] {
        for (_identity, device) in &mut devices {
            device
                .write_at(DeviceOffsetInBytes(offset), bytes, WriteDurability::Plain)
                .expect("半块写");
        }
    }
    assert_eq!(
        chunk_bytes_of_the_pool(&devices),
        ChunkBytesOfThePool {
            held_by_the_devices: 2 * MEMORY_DEVICE_CHUNK_BYTES,
            stored: MEMORY_DEVICE_CHUNK_BYTES,
        }
    );
    for (_identity, device) in &devices {
        assert_eq!(read_a_chunk(device, 1), written);
    }
}

/// 共用的存法：写成全是零的块不存，读回照旧是零；各存各的存法照存。
#[test]
fn chunk_written_back_to_zeroes_is_not_stored_and_reads_back_zero() {
    let recording: SharedRecordingDigest = Arc::new(Mutex::new(RecordingDigest::empty()));
    let mut shared = empty_pool_devices(4 * MEMORY_DEVICE_CHUNK_BYTES, &recording);
    let mut copied = empty_pool_devices_storing_the_chunks(
        4 * MEMORY_DEVICE_CHUNK_BYTES,
        &recording,
        ChunkStorage::EachDeviceCopiesItsChunks,
    );
    for devices in [&mut shared, &mut copied] {
        let (_identity, device) = &mut devices[0];
        device
            .write_at(DeviceOffsetInBytes(0), &[7u8; 512], WriteDurability::Plain)
            .expect("写");
        device
            .write_at(DeviceOffsetInBytes(0), &[0u8; 512], WriteDurability::Plain)
            .expect("写回零");
        assert!(read_a_chunk(device, 0).iter().all(|byte| *byte == 0));
    }
    assert_eq!(
        chunk_bytes_of_the_pool(&shared),
        ChunkBytesOfThePool::default()
    );
    assert_eq!(
        chunk_bytes_of_the_pool(&copied).stored,
        MEMORY_DEVICE_CHUNK_BYTES
    );
}

/// 共用的存法：拷一个池只拷块的引用（拷出来的与原来的指同一份），拷出来的那个池写到一块才复制那一块，原来的读回照旧。
#[test]
fn copied_pool_shares_the_chunks_until_one_of_its_devices_writes_a_chunk() {
    let recording: SharedRecordingDigest = Arc::new(Mutex::new(RecordingDigest::empty()));
    let mut devices = empty_pool_devices(4 * MEMORY_DEVICE_CHUNK_BYTES, &recording);
    let written = chunk_content(0);
    for (_identity, device) in &mut devices {
        device
            .write_at(DeviceOffsetInBytes(0), &written, WriteDurability::Plain)
            .expect("整块写");
    }
    let (mut copied, _copied_recording) = copied_pool_devices(&devices);
    assert!(Arc::ptr_eq(
        &devices[0].1.chunks[&0],
        &copied[0].1.chunks[&0]
    ));
    let rewritten = chunk_content(1);
    copied[0]
        .1
        .write_at(DeviceOffsetInBytes(0), &rewritten, WriteDurability::Plain)
        .expect("拷出来的池写一块");
    assert_eq!(read_a_chunk(&copied[0].1, 0), rewritten);
    assert_eq!(read_a_chunk(&copied[1].1, 0), written);
    for (_identity, device) in &devices {
        assert_eq!(read_a_chunk(device, 0), written);
    }
    assert_eq!(
        chunk_bytes_of_the_pool(&devices).stored,
        MEMORY_DEVICE_CHUNK_BYTES
    );
    assert_eq!(
        chunk_bytes_of_the_pool(&copied).stored,
        2 * MEMORY_DEVICE_CHUNK_BYTES
    );
}
