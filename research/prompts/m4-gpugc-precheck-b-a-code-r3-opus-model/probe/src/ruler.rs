//! 第三轮加的两样读数（只读会话里那一份分配器与拒绝的 Debug 文字，不动池）：
//!   同一把尺：每块盘上乙读的「两槽都不罩在未释放记录下的段外槽对数」（`slot_pairs_without_an_unreleased_record_outside_the_cluster_segments`）
//!   不小于落得下那一判读的「段外两槽都没挡的槽对数」（`unblocked_slot_pairs_outside_the_cluster_segments`），两样都排这次挂载的聚簇段；
//!   被拒的那一次是哪一种：落得下与式子那两判报的 `SpaceAdmissionRefused`、取落点被拒的 `PlacementRefused:<落点拒绝的成员>`、别的取成员名。

use crate::pool::{leading_text, Pool};
use crate::tally::Tally;

/// 每块盘比一次乙的上界读数与落得下那一判的读数：上界小于落得下就记 `ruler bound-below-landing` 并留一行样例。
pub fn check_the_ruler(pool: &Pool, tally: &mut Tally, context: &str, when: &str) {
    let allocator = &pool.session().allocator;
    let cluster_segments = allocator.cluster_segments();
    for device_map in &allocator.devices {
        let bound = device_map
            .slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(cluster_segments)
            .0;
        let landing = device_map
            .unblocked_slot_pairs_outside_the_cluster_segments(cluster_segments)
            .0;
        tally.add("ruler checked", 1);
        if bound > landing {
            tally.add("ruler bound-above-landing", 1);
        }
        if bound < landing {
            tally.add("ruler bound-below-landing", 1);
            tally.example(format!(
                "RULER_BOUND_BELOW_LANDING {context} when={when} device={:?} bound={bound} landing={landing}",
                device_map.device
            ));
        }
    }
}

/// 被拒的那一次是哪一种（按 Debug 文字认，不引事务层的类型）。
pub fn last_refusal_kind(last_refusal_text: &str) -> String {
    let leading = leading_text(last_refusal_text);
    if leading == "PlacementRefused" {
        let placement = last_refusal_text
            .split_once("refusal: ")
            .map(|(_, rest)| leading_text(rest))
            .unwrap_or_default();
        format!("PlacementRefused:{placement}")
    } else {
        leading
    }
}

/// 被拒的那一次归哪一类：`space-admission`（落得下与式子那两判）、`placement`（取落点）、`other`。
pub fn last_refusal_class(kind: &str) -> &'static str {
    if kind == "SpaceAdmissionRefused" {
        "space-admission"
    } else if kind.starts_with("PlacementRefused") {
        "placement"
    } else {
        "other"
    }
}
