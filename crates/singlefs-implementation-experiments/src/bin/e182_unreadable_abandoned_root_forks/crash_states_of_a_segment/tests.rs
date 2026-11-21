use super::*;

#[test]
fn segment_of_exactly_sixteen_writes_is_expanded_in_full_and_seventeen_takes_the_in_place_subsets()
{
    assert_eq!(expansion_of_segment(16), Expansion::FullSubsets);
    assert_eq!(
        expansion_of_segment(17),
        Expansion::QuickInPlaceSubsetsTimesUnitWritesNoneOrAll
    );
    assert_eq!(expansion_of_segment(0), Expansion::FullSubsets);
}
