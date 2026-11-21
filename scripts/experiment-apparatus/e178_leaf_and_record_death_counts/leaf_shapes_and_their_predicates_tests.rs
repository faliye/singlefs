use super::*;
use crate::{shape_named, stop_check_known_bad_images_match_the_implementation};

#[test]
fn the_shape_catalogue_holds_every_registered_family() {
    let catalogue = catalogue_of_shapes();
    let count = |family: &str| catalogue.iter().filter(|named| named.family == family).count();
    assert_eq!((count("K"), count("S"), count("BN1"), count("BN2"), count("DV")), (6, 22, 9, 4, 3));
}

/// 停机对拍 S6：K-0 全绿，K-1 至 K-5 只红今天的 I-9.4；K-2（右号 8）N1 的 I-9.4 绿、I-9.12 绿（登记第四节第 4 条推的那一格）。
#[test]
fn known_bad_shapes_redden_only_the_container_number_invariant() {
    let (holds, detail) = stop_check_known_bad_images_match_the_implementation();
    assert!(holds, "{detail}");
    let second = judge_the_shape(&shape_named("K-2"));
    assert!(!second.today_container_numbers_hold && second.raised_separator_container_numbers_hold && second.separators_hold);
}

/// N1 的谓词：BN1 分隔 key 没抬（s = 20，左最大 20）红，抬对（s = 30）绿（第九节 M12 盯这一句）。
#[test]
fn an_unraised_separator_reddens_the_raised_separator_reading() {
    assert!(!judge_the_shape(&shape_named("BN1-s20")).raised_separator_container_numbers_hold);
    assert!(judge_the_shape(&shape_named("BN1-s30")).raised_separator_container_numbers_hold);
    assert!(!judge_the_shape(&shape_named("BN1-s30")).today_container_numbers_hold);
}

#[test]
fn routing_takes_the_last_separator_not_above_the_key() {
    assert!(judge_the_shape(&shape_named("K-0")).routing_holds);
    assert!(!judge_the_shape(&shape_named("DV-right-absorbs-left")).routing_holds);
    assert!(!judge_the_shape(&shape_named("BN1-s31")).routing_holds);
}

#[test]
fn the_sparse_layout_scales_keys_and_takes_midpoints_between_keys() {
    let sparse = sparse_layout_of(&shape_named("K-0"));
    assert_eq!(sparse.separators, vec![100, 1500]);
    assert_eq!(sparse.leaves[1].container_number, 1500);
    assert_eq!(sparse.leaves[0].record_keys, vec![100, 1000]);
    assert_eq!(scaled_by_one_hundred(0, &[1, 10]), 0);
    assert_eq!(scaled_by_one_hundred(31, &[1, 10, 20, 30]), 3100);
}
