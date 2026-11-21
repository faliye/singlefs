//! 落地向量与镜像摘要（登记 5.3）的单测。

use super::*;

/// 空消息的 sha256（`python3 -c "import hashlib; print(hashlib.sha256(b'').hexdigest())"` 现算）。
const EMPTY_MESSAGE_SHA256: &str =
    "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
/// 盘 1、扇区 2、512 个 0xAB 那一条的摘要（`struct.pack('<IQ', 1, 2) + b'\xab' * 512` 喂 hashlib 现算）。
const ONE_SECTOR_OF_0XAB_SHA256: &str =
    "c71cae1b3fc678e4cabfa21d575c728edd44a4b372b3389ce763856273abecbf";

#[test]
fn torn_landing_folds_to_one_and_a_persisted_write_to_two() {
    // 写表：0 持久、1 没持久、2 原写没持久而撕裂镜像（枚举写表下标 3）持久。
    let persisted = vec![true, false, false, true];
    let torn_image_index_by_write = BTreeMap::from([(2usize, 3usize)]);
    assert_eq!(
        landing_vector_of(&persisted, 3, &torn_image_index_by_write),
        "201",
        "撕裂那一态折回 1，不是 2"
    );
    let persisted_with_the_torn_image_absent = vec![true, false, false, false];
    assert_eq!(
        landing_vector_of(
            &persisted_with_the_torn_image_absent,
            3,
            &torn_image_index_by_write
        ),
        "200"
    );
}

#[test]
fn an_all_zero_sector_does_not_enter_the_image_digest() {
    let mut message = Vec::new();
    append_sector_to_digest_message(&mut message, 0, 7, &[0u8; 512]);
    assert!(message.is_empty(), "全零扇区不进摘要");
    assert_eq!(sha256_text(&sha256_digest(&message)), EMPTY_MESSAGE_SHA256);
    append_sector_to_digest_message(&mut message, 1, 2, &[0xAB; 512]);
    assert_eq!(message.len(), 4 + 8 + 512);
    assert_eq!(
        sha256_text(&sha256_digest(&message)),
        ONE_SECTOR_OF_0XAB_SHA256
    );
}

#[test]
fn merged_sector_ranges_join_overlapping_and_touching_ranges_only() {
    assert_eq!(
        merged_sector_ranges(vec![(10, 12), (0, 2), (2, 3), (11, 15), (20, 21)]),
        vec![(0, 3), (10, 15), (20, 21)]
    );
}
