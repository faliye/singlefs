//! C742（实例表没收全时行唯一的不适用理由没分清）：最新根的实例表第 0 片之后有一片
//! 解不开时，池级 checker 不照已收到的前几片的行判 I-3.8（实例表行唯一且低于挂载根），整条报不适用、理由点名停在第 0 片之后的那一片。
//! 两片的表由真写者写出（`instance_table_second_page_write.rs` 那一份搭法：连着 369 次取号之后崩溃，再可写挂载一次，370 行写成两片）。
mod common;
mod common_images_for_the_given_states_engine;
mod common_pointer_head_corruption;
mod common_shared_by_harness_scenarios;
#[path = "common_images_for_the_given_states_engine/instance_table_second_page_write.rs"]
mod scenarios_of_instance_table_second_page_write;

use common::memory_pool_of_sparse_devices;
use common_pointer_head_corruption::newest_root;
use scenarios_of_instance_table_second_page_write::pool_whose_row_publish_opened_the_second_page;
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::recovery::{instance_table_chain_of_root, PoolReader};
use singlefs_verification::memory_pool::MemoryPool;

/// 实例表一片的字节数（码 3 打包记录单元，两槽）。
const INSTANCE_TABLE_PAGE_BYTES: usize = 32768;
/// 码 3 头里载荷 CRC 那 4 字节的头一个字节（D18（块里携带什么信息） 已定项 11 的头字段表）。
const PACKED_UNIT_PAYLOAD_CRC_OFFSET: usize = 89;

fn verdict(pool: &MemoryPool, invariant: &str) -> InvariantVerdict {
    check_pool_image(pool)
        .into_iter()
        .find(|(identifier, _)| *identifier == invariant)
        .expect("清单里有")
        .1
}

/// 两片的表上，第 1 片两份都把载荷 CRC 翻一位、不补引用链（第 0 片链指针记录里那条指针的整单元校验和对不上）：第 1 片两份都读不到对得上的。
/// 第 0 片的 369 行照样收得到，可第 1 片的那一行看不到（它与前几片的行重不重号判不出），
/// I-3.8 报不适用、理由点名停在第 0 片之后的那一片读不出，不照第 0 片的行判成立。
/// 判别力（`crates/mutations.tsv` 里点名这条的行）：没走到末片照已收到的行判时 I-3.8 判成立，这条红。
#[test]
fn page_after_the_first_reading_back_from_neither_copy_leaves_the_row_invariant_not_applicable_instead_of_judging_the_rows_before_it(
) {
    let (devices, _mounted) = pool_whose_row_publish_opened_the_second_page();
    let mut image = memory_pool_of_sparse_devices(&devices);
    assert_eq!(
        verdict(&image, "I-3.8"),
        InvariantVerdict::Holds,
        "两片都读得出时 I-3.8 在 370 行上真被评估过且成立"
    );
    let newest = newest_root(&image);
    let chain = instance_table_chain_of_root(&image, &newest).expect("改之前两片都读得出");
    assert_eq!(chain.page_pointers.len(), 2, "最新那一版的表是两片");
    for location in chain.page_pointers[1].locations {
        let offset = location.slot.to_device_offset();
        let mut bytes =
            PoolReader::read(&image, location.device, offset, INSTANCE_TABLE_PAGE_BYTES)
                .expect("第 1 片那一份读得到");
        bytes[PACKED_UNIT_PAYLOAD_CRC_OFFSET] ^= 1;
        image
            .devices
            .get_mut(&location.device)
            .expect("盘")
            .write(offset, &bytes);
    }
    assert_eq!(
        verdict(&image, "I-3.8"),
        InvariantVerdict::NotApplicable(
            "最新根的实例表第 0 片之后有一片两份都读不到对得上的、行没收全：不照前几片的行判"
        ),
        "第 1 片两份都读不出：I-3.8 报不适用并点名停在第 0 片之后的那一片，不照第 0 片的 369 行判成立"
    );
}
