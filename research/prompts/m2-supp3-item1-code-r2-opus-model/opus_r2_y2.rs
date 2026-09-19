//! 攻方腿（Opus）m2-supp3-item1-code-r2 · Y2：只放进打了 y2-patch.py 的仓副本。跑快档与专门取样点（与入库用例同种子、同步数、同比重），
//! 报两档各自的收尾计数与新发现签名，再报挂载里（写行、暖机）与挂载外复用改写已回收记录的次数、挂载里写分配记录的次数。

use singlefs_core::allocator::{OPUS_MOUNT_RECORDS, OPUS_MOUNT_REWRITES, OPUS_OTHER_REWRITES, opus_mode};
use singlefs_harness::history::{run_history_campaign, FindingShrinking, GenerationWeights};
use std::sync::atomic::Ordering;

#[test]
#[ignore = "攻方探针，副本上手动跑"]
fn opus_r2_y2_mount_only_generation() {
    for (label, first, seeds, weights) in [
        ("fast", 0u64, 96u64, GenerationWeights::BROAD),
        ("reuse", 0, 48, GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR),
    ] {
        let before = (
            OPUS_MOUNT_REWRITES.load(Ordering::Relaxed),
            OPUS_OTHER_REWRITES.load(Ordering::Relaxed),
            OPUS_MOUNT_RECORDS.load(Ordering::Relaxed),
        );
        let report = run_history_campaign(first, seeds, 30, &weights, 16, FindingShrinking::ReportSeedsOnly);
        let rendered = report.render();
        println!("Y2 mode={} tier={label} new_findings={} mount_rewrites={} other_rewrites={} mount_records={}",
            opus_mode(),
            report.new_findings.len(),
            OPUS_MOUNT_REWRITES.load(Ordering::Relaxed) - before.0,
            OPUS_OTHER_REWRITES.load(Ordering::Relaxed) - before.1,
            OPUS_MOUNT_RECORDS.load(Ordering::Relaxed) - before.2);
        for line in rendered.lines().take(40) {
            println!("  {line}");
        }
    }
}
