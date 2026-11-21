use super::*;

#[test]
fn the_pressed_back_inode_number_watermark_refuses_the_invalid_inode_number_zero() {
    assert_eq!(
        InodeNumberWatermarkPressedBackForTests::new(0),
        None,
        "0 是无效 inode 号，不是一个发过的号"
    );
    assert_eq!(
        InodeNumberWatermarkPressedBackForTests::new(FIRST_INODE_NUMBER)
            .map(InodeNumberWatermarkPressedBackForTests::next_inode_number),
        Some(FIRST_INODE_NUMBER),
        "第一个 inode 号压得到"
    );
}

#[test]
#[should_panic(expected = "要小于它才是压回一个已发过的号")]
fn pressed_back_inode_number_watermark_not_below_the_previous_watermark_is_a_misused_switch() {
    let not_pressed_back = InodeNumberWatermarkSource::PressedBackForTests(
        InodeNumberWatermarkPressedBackForTests::new(5).expect("5 不小于第一个 inode 号"),
    );
    let _ = not_pressed_back.watermark_this_publish_starts_from(5);
}

#[test]
fn publish_without_a_previous_version_starts_the_inode_number_watermark_at_two_on_the_product_path()
{
    assert_eq!(
        InodeNumberWatermarkSource::FromThePreviousVersionsAccounting
            .watermark_of_a_publish_without_a_previous_version(),
        2,
        "新池新建文件：树里只有 inode 1，下一个可用号是 2（字节表六那一行）"
    );
}

#[test]
#[should_panic(expected = "开关只给有上一版的发布，装着它走到这里是开关给错了")]
fn pressed_back_inode_number_watermark_on_a_publish_without_a_previous_version_is_a_misused_switch()
{
    let pressed_back = InodeNumberWatermarkSource::PressedBackForTests(
        InodeNumberWatermarkPressedBackForTests::new(FIRST_INODE_NUMBER)
            .expect("第一个 inode 号压得到"),
    );
    let _ = pressed_back.watermark_of_a_publish_without_a_previous_version();
}
