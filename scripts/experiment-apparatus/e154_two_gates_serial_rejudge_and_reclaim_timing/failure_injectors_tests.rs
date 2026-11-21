use super::*;

/// `FailureInjector`（H5-k-位 R8「切换的写行发布的根也算在连续失败里」的窗口）：位=首（`window_start=1`）
/// 连续 k 次真返回、之后恒假。
#[test]
fn failure_injector_first_position_injects_from_first_call() {
    let mut injector = FailureInjector::new(1, 2);
    assert_eq!([injector.next(0), injector.next(0), injector.next(0), injector.next(0)], [true, true, false, false]);
    assert_eq!(injector.injected, 2);
}

/// 位=覆（`window_start` > 1）：窗口之前恒假，进窗口之后连续 k 次真，窗口用完恒假。
#[test]
fn failure_injector_cover_position_injects_from_window_start() {
    let mut injector = FailureInjector::new(3, 2);
    assert_eq!(
        [injector.next(0), injector.next(0), injector.next(0), injector.next(0), injector.next(0)],
        [false, false, true, true, false]
    );
    assert_eq!(injector.injected, 2);
}

/// `cover_push_index` 与「四」I1 锚点同一个 txg mod 3 公式：起点状态最后一条根 txg=2（暖机第二条），
/// 2 mod 3 = 2 ⇒ 2 次；推一次空发布到 txg=3（3 mod 3 = 0）⇒ 仍是 2 次；再推一次到 txg=4
/// （4 mod 3 = 1）⇒ 3 次。
#[test]
fn cover_push_index_matches_ring_rotation_anchor_formula() {
    let geometry = baseline_geometry();
    let mut pool = Pool::seed(geometry);
    assert_eq!(cover_push_index(&pool), 2, "起点状态最后一条根 txg=2，2 mod 3 = 2 ⇒ 2 次");
    push_one_empty_publish(&mut pool, ReclaimTiming::AfterEffectiveFloor, &mut |_last_txg| false);
    assert_eq!(cover_push_index(&pool), 2, "txg=3，3 mod 3 = 0 ⇒ 2 次");
    push_one_empty_publish(&mut pool, ReclaimTiming::AfterEffectiveFloor, &mut |_last_txg| false);
    assert_eq!(cover_push_index(&pool), 3, "txg=4，4 mod 3 = 1 ⇒ 3 次");
}

// ===== H8（第四段）：`LazyFailureInjector` =====

/// 位=首：`window_start` 恒为 1，与 `next` 的参数无关——传任意 `last_txg` 都不改变这一点，
/// 与 `FailureInjector::new(1, k)` 逐次读数相同。
#[test]
fn lazy_failure_injector_first_position_ignores_last_txg_argument() {
    let mut injector = LazyFailureInjector::new(FailurePosition::First, 2);
    assert_eq!(
        [injector.next(999), injector.next(0), injector.next(12345), injector.next(7)],
        [true, true, false, false]
    );
    assert_eq!(injector.injected, 2);
}

/// 位=覆：`window_start` 在**第一次**被问到时，用那一次的 `last_txg` 现算并锁定——之后的
/// 调用即使传不同的 `last_txg`，也不会重算。第一次传 2（2 mod 3 = 2）⇒ `window_start = 2`，
/// 与 `cover_push_index_matches_ring_rotation_anchor_formula` 的起点状态锚点同一个值。
#[test]
fn lazy_failure_injector_cover_position_locks_window_start_on_first_call() {
    let mut injector = LazyFailureInjector::new(FailurePosition::Cover, 2);
    let first = injector.next(2);
    let rest = [injector.next(999), injector.next(999), injector.next(999)];
    assert!(!first, "call_index=1 < window_start=2");
    assert_eq!(rest, [true, true, false], "window_start 锁定之后不再看后续调用传的 999");
    assert_eq!(injector.injected, 2);
}

/// 同一条锚点公式的另一个取样点：第一次传 4（4 mod 3 = 1）⇒ `window_start = 3`——
/// 证明 `window_start` 真的是从第一次调用的参数现算出来的，不是写死的 2。
#[test]
fn lazy_failure_injector_cover_position_with_different_first_txg_matches_ring_rotation_anchor() {
    let mut injector = LazyFailureInjector::new(FailurePosition::Cover, 1);
    assert_eq!([injector.next(4), injector.next(4), injector.next(4)], [false, false, true]);
    assert_eq!(injector.injected, 1);
}
