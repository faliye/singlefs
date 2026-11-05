//! 辩方腿 m3-close-b19-code-r2-sonnet 的探针（U6，T5 C690）：S = 4 的小环池上反复可写挂载加卸载，产品路径（开关关着、阈值 365）。
//! 看：① 清扫第一次触发时挂载末尾现行那一版的行数（修补后的文档说 366 行起）；② 清扫做成之后行数；
//! ③ 整个过程里行数有没有越过一片（369 行）、链有没有长出第二片；④ 清扫的结局分布。
//! 每次挂载写一行（实例号逐次加一）。

#![allow(dead_code, reason = "几份 common 模块各只用到一部分")]

mod common;
mod common_root_ring_holes;

use common_root_ring_holes::{
    parameters_of_the_ring_hole_pools, pool_overwritten_through, devices_of,
};
use singlefs_core::instance_table_sweep::{
    InstanceTableSweepAtTheEndOfTheMount, InstanceTableSweepOutcome,
};
use singlefs_core::mount::{mount_writable, unmount, Mounted, ShadowLedger};
use singlefs_core::recovery::instance_table_chain_of_root;
use singlefs_core::admission::INSTANCE_ROWS_PER_INSTANCE_TABLE_PAGE;

const MOUNTS: usize = 420;

#[test]
fn u6_the_default_threshold_starts_the_sweep_at_366_rows_and_the_chain_stays_in_one_page() {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut devices = devices_of(&pool_overwritten_through(4));
    let mut first_swept: Option<(usize, u64)> = None;
    let mut max_rows_at_the_decision = 0u64;
    let mut max_rows_after = 0u64;
    let mut not_swept = 0usize;
    let mut swept_rows_deleted = 0usize;
    let mut swept_no_row_deleted = 0usize;
    let mut swept_other = 0usize;
    let mut rows_over_one_page = 0usize;
    let mut last_not_swept_rows = 0u64;
    for mount_number in 1..=MOUNTS {
        let mounted = mount_writable(&parameters, &mut devices).expect("可写挂载");
        let Mounted { output, mut allocator, mut current } = mounted;
        let rows_after = u64::try_from(
            instance_table_chain_of_root(&devices, current.root())
                .expect("现行那一版的实例表读得出")
                .records
                .rows
                .len(),
        )
        .expect("行数装得进 u64");
        max_rows_after = max_rows_after.max(rows_after);
        if rows_after > INSTANCE_ROWS_PER_INSTANCE_TABLE_PAGE {
            rows_over_one_page += 1;
        }
        match &output.instance_table_sweep {
            InstanceTableSweepAtTheEndOfTheMount::NotSweptAtOrBelowTheThreshold { rows, threshold } => {
                not_swept += 1;
                last_not_swept_rows = *rows;
                max_rows_at_the_decision = max_rows_at_the_decision.max(*rows);
                assert!(*rows <= threshold.0, "不扫的那一次行数不越过阈值");
            }
            InstanceTableSweepAtTheEndOfTheMount::HandedToTheBackgroundSweepWhileMounted => {
                panic!("产品路径开关关着")
            }
            InstanceTableSweepAtTheEndOfTheMount::Swept(outcome) => {
                if first_swept.is_none() {
                    first_swept = Some((mount_number, last_not_swept_rows + 1));
                    println!(
                        "PROBE name=u6_first_sweep mount_number={mount_number} last_not_swept_rows={last_not_swept_rows} rows_after_the_mount={rows_after}"
                    );
                }
                match outcome {
                    InstanceTableSweepOutcome::RowsDeleted { rows_deleted, .. } => {
                        swept_rows_deleted += 1;
                        if swept_rows_deleted <= 3 {
                            println!("PROBE name=u6_rows_deleted mount_number={mount_number} deleted={} rows_after_the_mount={rows_after}", rows_deleted.len());
                        }
                    }
                    InstanceTableSweepOutcome::NoRowDeleted(why) => {
                        swept_no_row_deleted += 1;
                        if swept_no_row_deleted <= 3 {
                            println!("PROBE name=u6_no_row_deleted mount_number={mount_number} why={:?} rows_after_the_mount={rows_after}", format!("{why:?}").chars().take(120).collect::<String>());
                        }
                    }
                    _ => swept_other += 1,
                }
            }
        }
        unmount(&parameters, &mut devices, &mut allocator, &mut current, ShadowLedger::On).expect("卸载");
    }
    println!(
        "PROBE name=u6_summary mounts={MOUNTS} not_swept={not_swept} rows_deleted={swept_rows_deleted} no_row_deleted={swept_no_row_deleted} other={swept_other} max_rows_at_a_not_swept_decision={max_rows_at_the_decision} max_rows_after_a_mount={max_rows_after} mounts_with_rows_over_one_page={rows_over_one_page}"
    );
    let (_, rows_at_the_first_sweep) = first_swept.expect("420 次挂载之内清扫触发过");
    assert_eq!(rows_at_the_first_sweep, 366, "修补后的文档：366 行起扫");
}
