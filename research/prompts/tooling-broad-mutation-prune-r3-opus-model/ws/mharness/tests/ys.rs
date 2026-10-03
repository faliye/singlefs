//! Y1–Y8：C1 的场景，每个一条测试。
use mcore::y6_trait::Sz;

#[test]
fn y1_tag_of_zero_has_two_bytes() {
    assert_eq!(mcore::y1::tag(0).len(), 2);
}

#[test]
fn y2_wrap_is_even() {
    assert_eq!(mcore::y2::wrap(2).0 % 2, 0);
}

#[test]
fn y3_join_is_even() {
    assert_eq!(mcore::y3::join(2).0 % 2, 0);
}

#[test]
fn y4_bump_stays_small() {
    assert!(mcore::y4::bump(2) < 6);
}

#[test]
fn y5_pick_is_even() {
    assert_eq!(mcore::y5::pick(mcore::y5::S, 2) % 2, 0);
}

#[test]
fn y6_size_is_even() {
    assert_eq!(mcore::y6_impl::P.size() % 2, 0);
}

#[test]
fn y7_next_of_zero_is_positive() {
    assert!(mcore::y7::next(0) >= 1);
}

#[test]
fn y8_scaled_is_even() {
    assert_eq!(mcore::y8::scaled(2) % 2, 0);
}

#[test]
fn y9_make_of_zero_holds_zero() {
    assert_eq!(mcore::y9::make(0).0, 0);
}
