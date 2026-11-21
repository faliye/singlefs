//! 单元级检查重放（G3 的 Q3f、Q3g）、分区计时与键的算法的单测。

use super::*;

/// PC3 的缩小版，读计数钟（不读机器时间，判定与负载无关）：D1-small 每个状态在看 journal 那一区忙等一次，
/// 六个计时区里只有看 journal 那一区记到 53 × 2 ms，别的区都是 0，整个状态的计时同样记到 53 × 2 ms。
#[test]
fn busy_wait_in_the_consult_zone_lands_in_the_consult_zone() {
    let (_stream, busy, small) = evaluate_the_first_stream_small_domain(
        1,
        TimingControl {
            busy_zone: Some(TimingZone::RecoveryConsult),
            clock: ZoneClock::Counted,
        },
    );
    let mut sums = TimingSums::default();
    for segment_sums in busy.timing_by_segment.values() {
        sums.absorb(segment_sums);
    }
    let busy_wait_of_the_domain = u128::try_from(small.len()).expect("状态数装得进 u128")
        * TIMING_CONTROL_BUSY_WAIT.as_nanos();
    assert_eq!(
        sums.zone_nanoseconds,
        [busy_wait_of_the_domain, 0, 0, 0, 0, 0],
        "六个区依次是 {:?}：忙等只记到看 journal 那一区",
        TIMING_ZONES.map(TimingZone::label)
    );
    assert_eq!(
        sums.state_nanoseconds, busy_wait_of_the_domain,
        "整个状态的计时包住看 journal 那一区"
    );
}
