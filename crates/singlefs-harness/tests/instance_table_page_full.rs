//! 里程碑「覆盖写、释放、回退与复用」增补 2 收口，代码三方第二轮判决（`research/prompts/m2-wave2-code-r1-main-verification.md`）第二节第 2 行（Z1-a）：
//! 每次可写挂载都给上一个实例写一行（D18（块里携带什么信息） 已定项 11），一片实例表 370 条记录，链指针记录恒为最后一条、
//! 数据行至多 369。改之前取号之前的准入不算实例表：第 370 次可写挂载取号写完之后在装实例表单元时越界 panic（`bytes.rs` 的
//! `ByteWriter`），每试一次再烧一个实例代号（攻方探针 `research/prompts/m2-wave2-code-r1-opus-model/opus_probe_pristine.rs` 的
//! `pristine_instance_table_rows_overflow_after_acquisition`，日志 `pristine-run.log`）。
//! 第二片进里程碑二（增补 2 收口表第 38 行），两条写法：行一片写满 369 行再开下一片、
//! 多于一片时在 bump 次序里尾片先。所以一片写满之后的那一次挂载不再拒，写第二片（真写者写出的两片怎么排、checker 怎么判见
//! `instance_table_second_page_write.rs`）。管理员回退改成挂着时的一次向前发布之后不写行、
//! 实例表照现行那一版的（D23（journal 的角色与格式） 已定项 14），「回退按 R_old 那一版表算行数」那一格没有对象，删了。
//!
//! 「取号之后崩溃」（取号写完、写行那次发布之前掉电）是今天就走得到的历史：下一次挂载要给中间那些实例各补一行 (i, 0, 0)，
//! 一次挂载就能写很多行。本文件用它把表快速填到边上（`acquire_instance` 连取 k 次号 = 连着 k 次取号之后崩溃）。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/instance_table_page_full.rs"]
mod scenarios_of_instance_table_page_full;

use common::parameters;
use common_images_for_the_checker_tier::PoolCheckerImages;
use scenarios_of_instance_table_page_full::{
    crash_right_after_acquisition, pool_after_the_new_pool_file_creation,
    rows_of_each_page_of_the_row_publish, ROWS_PER_PAGE,
};
use singlefs_core::address::InstanceGeneration;
use singlefs_core::mount::mount_writable;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_instance_table_page_full::writable_mounts_fill_the_instance_table_page_and_the_three_hundred_seventy_first_opens_the_second_page`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn writable_mounts_fill_the_instance_table_page_and_the_three_hundred_seventy_first_opens_the_second_page(
) {
    scenarios_of_instance_table_page_full::writable_mounts_fill_the_instance_table_page_and_the_three_hundred_seventy_first_opens_the_second_page(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 一次挂载要写很多行：连着 368 次取号之后崩溃，下一次挂载取 370、给 [1, 370) 写 369 行，正好写满一片；
/// 连着 369 次，要写 370 行——一片写满 369 行、第二片装 1 行。
#[test]
fn mount_after_crashes_right_after_acquisition_fills_the_page_and_one_more_row_opens_the_second_page(
) {
    let (mut filled_exactly, _) = pool_after_the_new_pool_file_creation();
    crash_right_after_acquisition(&mut filled_exactly, 368);
    let mounted = mount_writable(&parameters(), &mut filled_exactly)
        .unwrap_or_else(|error| panic!("369 行正好写满一片，要放行：{error:?}"));
    assert_eq!(mounted.output.instance, InstanceGeneration(370));
    assert_eq!(mounted.output.rows_written.len(), ROWS_PER_PAGE);
    assert_eq!(
        rows_of_each_page_of_the_row_publish(&filled_exactly, &mounted),
        vec![ROWS_PER_PAGE]
    );

    let (mut one_row_more, _) = pool_after_the_new_pool_file_creation();
    crash_right_after_acquisition(&mut one_row_more, 369);
    let mounted_past_one_page = mount_writable(&parameters(), &mut one_row_more)
        .unwrap_or_else(|error| panic!("370 行写两片：{error:?}"));
    assert_eq!(
        mounted_past_one_page.output.instance,
        InstanceGeneration(371)
    );
    assert_eq!(
        mounted_past_one_page.output.rows_written.len(),
        ROWS_PER_PAGE + 1
    );
    assert_eq!(
        rows_of_each_page_of_the_row_publish(&one_row_more, &mounted_past_one_page),
        vec![ROWS_PER_PAGE, 1]
    );
}
