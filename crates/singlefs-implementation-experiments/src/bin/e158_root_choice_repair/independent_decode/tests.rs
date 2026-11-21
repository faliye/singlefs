use super::*;

/// bootstrap 之后装置自记的时间线与 checker 独立解出的可读根集合逐条一致（S5 的最小单测版本）。
#[test]
fn bootstrap_timeline_matches_independent_decode() {
    let node = bootstrap(&GEOMETRY_PRIMARY, 1).expect("bootstrap 应当成功");
    let geometry_view = independent_geometry(&node.pool).expect("几何应当能独立解出来");
    let readable = readable_roots_independent(&node.pool, &geometry_view);
    let timeline_set: BTreeSet<TimelineRoot> = node.timeline.iter().copied().collect();
    assert_eq!(
        readable, timeline_set,
        "不注入故障时，装置自记的时间线应当与根环里全部自证根的集合相同"
    );
}
