use super::*;

#[test]
fn chunk_digests_of_a_memory_device_and_its_image_file_agree_and_one_byte_changes_them() {
    let mut device = SparseBlockDevice::new(
        3 * IMAGE_CHUNK_BYTES,
        PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
    );
    device
        .write_at(
            DeviceOffsetInBytes(IMAGE_CHUNK_BYTES + 512),
            &[3_u8; 1024],
            WriteDurability::Plain,
        )
        .expect("写");
    // 守卫认领镜像：用例做完、断言失败、panic 都删掉它
    let path = singlefs_verification::temporary_files::TemporaryPathRemovedOnDrop::claiming(
        std::env::temp_dir().join(format!("e185-digest-test-{}.img", std::process::id())),
    );
    let _ = std::fs::remove_file(&path);
    create_a_zero_filled_image(&path, 3 * IMAGE_CHUNK_BYTES).expect("建全零镜像");
    write_the_memory_device_into(&path, &device).expect("写进镜像");
    let file_digest = digest_of_the_image_file(&path).expect("算摘要");
    assert_eq!(
        file_digest,
        digest_of_the_memory_device(&device),
        "内存盘与镜像文件按同一个定义算"
    );
    let file = OpenOptions::new().write(true).open(&path).expect("打开");
    file.write_all_at(&[1_u8], 2 * IMAGE_CHUNK_BYTES + 7)
        .expect("改一个字节");
    drop(file);
    assert_ne!(
        digest_of_the_image_file(&path).expect("算摘要"),
        file_digest,
        "改一个字节摘要就不同"
    );
}

#[test]
fn dropping_the_page_cache_of_a_file_keeps_its_bytes_and_a_missing_file_is_an_error() {
    // 守卫认领临时文件：断言失败、panic 时也删掉它（用例自己先删一次，要看文件不在时报什么）
    let path = singlefs_verification::temporary_files::TemporaryPathRemovedOnDrop::claiming(
        std::env::temp_dir().join(format!("e185-page-cache-drop-{}", std::process::id())),
    );
    std::fs::write(&path, vec![9_u8; 8192]).expect("写得进临时文件");
    drop_the_page_cache_of_the_file(&path).expect("扔页缓存成功");
    assert_eq!(std::fs::read(&path).expect("读得回"), vec![9_u8; 8192]);
    std::fs::remove_file(&path).expect("删掉这条单测建的文件");
    assert!(
        drop_the_page_cache_of_the_file(&path).is_err(),
        "文件不在时报错"
    );
}
