use super::*;

/// 一次读数（MiB）：空闲 `free_mebibytes`，空卡上能用的是一张 16 GiB 卡的 15849，这个进程占着的照 `occupied_by_this_process`。
fn reading_of_a_sixteen_gibibyte_card(
    free_mebibytes: u64,
    occupied_by_this_process: OccupiedByThisProcess,
) -> crate::gpu_memory_ledger::CardMemoryReading {
    crate::gpu_memory_ledger::CardMemoryReading {
        free_mebibytes,
        total_less_reserved_mebibytes: 15_849,
        occupied_by_this_process,
    }
}

/// 照一串假读数走一遍等卡（[`super::card_memory_for_a_judge_waiting_for_this_process`]，额度是服务交的 16384 MiB）：交判定、读到第几份、停了几次。
/// 读的次数超出 `readings` 就 panic（等卡没在该交的时候交）。
fn card_memory_judged_over_the_readings(
    readings: &[crate::gpu_memory_ledger::CardMemoryReading],
    most_readings: u64,
) -> (super::CardMemoryForAJudge, usize, usize) {
    let mut readings_taken = 0_usize;
    let mut pauses = 0_usize;
    let (judged, reading) = super::card_memory_for_a_judge_waiting_for_this_process(
        Some(16_384),
        &mut || {
            let reading = *readings.get(readings_taken).unwrap_or_else(|| {
                panic!(
                    "等卡读了第 {} 份，只给了 {} 份",
                    readings_taken + 1,
                    readings.len()
                )
            });
            readings_taken += 1;
            Ok(reading)
        },
        &mut || pauses += 1,
        most_readings,
    )
    .expect("假读数都读得出");
    assert_eq!(
        Some(&reading),
        readings.get(readings_taken - 1),
        "交回的是最后读的那一份"
    );
    (judged, readings_taken, pauses)
}

/// 场景：服务交的一张 16 GiB 卡上空闲只剩 579 MiB，占着的 10680 MiB 是同一个测试进程里别的 GPU 用例的；之后这个进程还了，空闲回到 13268 MiB。
/// 预期：这张卡不判用不了，等这个进程还；读到空闲回到 13268 MiB 就放得下，前两份各停一次。不要卡（假读数）。
/// 判别力（`crates/mutations.tsv`「GPU 取卡 判卡时空闲不加回这个进程自己占着的」）：空闲不加回这个进程占着的，头一份就判用不了，这条红。
#[test]
fn card_held_by_other_gpu_tests_of_this_process_is_waited_for_until_they_return_its_memory() {
    let (judged, readings_taken, pauses) = card_memory_judged_over_the_readings(
        &[
            reading_of_a_sixteen_gibibyte_card(
                579,
                OccupiedByThisProcess::AlsoOutsideTheBooks { mebibytes: 10_680 },
            ),
            reading_of_a_sixteen_gibibyte_card(
                1_892,
                OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes: 9_842 },
            ),
            reading_of_a_sixteen_gibibyte_card(
                13_268,
                OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes: 2_400 },
            ),
        ],
        600,
    );
    assert_eq!(
        (judged, readings_taken, pauses),
        (super::CardMemoryForAJudge::RoomNow, 3, 2),
        "这个进程自己占着的卡要等它还、还了就起，不判用不了"
    );
}

/// 同一张卡空闲 579 MiB，这个进程在卡上一点都没占（占着的是别的进程）。
/// 预期：当场判用不了，读一次、一次都不停（别的进程占着，等这个进程还等不来）。不要卡（假读数）。
/// 判别力（`crates/mutations.tsv`「GPU 取卡 别的进程占着的卡也当这个进程自己占着去等」）：加回之后还不够也判成等，这条红。
#[test]
fn card_held_by_another_process_is_not_usable_and_not_waited_for() {
    let (judged, readings_taken, pauses) = card_memory_judged_over_the_readings(
        &[reading_of_a_sixteen_gibibyte_card(
            579,
            OccupiedByThisProcess::NotListed,
        )],
        600,
    );
    assert_eq!(
        (judged, readings_taken, pauses),
        (
            super::CardMemoryForAJudge::NoRoomEvenWithWhatThisProcessHolds,
            1,
            0
        ),
        "别的进程占着的卡当场判用不了"
    );
}

/// 这个进程一直占着同一张卡（读了 3 份都是空闲 579 MiB、这个进程占着 10680 MiB），最多读 3 份。
/// 预期：读满 3 份、停 2 次就交「等这个进程还」那一种（调用方照它判用不了），不一直等。不要卡（假读数，读第 4 份就 panic）。
/// 判别力（`crates/mutations.tsv`「GPU 取卡 等这个进程还显存不设上限」）：读满了还接着等，读第 4 份 panic，这条红。
#[test]
fn waiting_for_this_process_to_return_card_memory_stops_after_the_most_readings() {
    let held = reading_of_a_sixteen_gibibyte_card(
        579,
        OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes: 10_680 },
    );
    let (judged, readings_taken, pauses) =
        card_memory_judged_over_the_readings(&[held, held, held], 3);
    assert_eq!(
        (judged, readings_taken, pauses),
        (
            super::CardMemoryForAJudge::RoomOnceThisProcessReturnsWhatItHolds,
            3,
            2
        ),
        "读满最多的份数就交回，不再等"
    );
}

/// 门槛是驱动那一份加最小一份判表（`crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER` 加 1 MiB），不另写一个数：
/// 空闲 2254 MiB 的卡（比 2048 大、比驱动那一份小，一个状态都放不下）判此刻放不下；额度正好是门槛的放得下、少 1 MiB 的放不下。不要卡。
/// 判别力（`crates/mutations.tsv`「GPU 取卡 门槛不照驱动那一份加最小一份判表算」）：门槛写回 2048，这条红。
#[test]
fn the_minimum_memory_budget_is_the_driver_share_plus_the_smallest_judge_tables() {
    assert_eq!(
        super::MINIMUM_MEMORY_BUDGET_MEBIBYTES,
        crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER + 1,
        "门槛是驱动那一份加最小一份判表（一个字的表上卡按 MiB 向上取整是 1 MiB）"
    );
    let judged_with_free = |free_mebibytes: u64| {
        super::card_memory_for_a_judge(
            Some(16_384),
            reading_of_a_sixteen_gibibyte_card(free_mebibytes, OccupiedByThisProcess::NotListed),
        )
    };
    assert_eq!(
        judged_with_free(2_254),
        super::CardMemoryForAJudge::NoRoomEvenWithWhatThisProcessHolds,
        "空闲 2254 MiB 放不下驱动那一份与最小一份表"
    );
    assert_eq!(
        judged_with_free(super::MINIMUM_MEMORY_BUDGET_MEBIBYTES),
        super::CardMemoryForAJudge::RoomNow,
        "额度正好是门槛的放得下"
    );
    assert_eq!(
        judged_with_free(super::MINIMUM_MEMORY_BUDGET_MEBIBYTES - 1),
        super::CardMemoryForAJudge::NoRoomEvenWithWhatThisProcessHolds,
        "额度比门槛少 1 MiB 的放不下"
    );
}

/// 不走卡池起的卡空着时的额度照空卡上能用的算：额度 16384 MiB、此刻空闲 2254 MiB、空卡上能用 15849 MiB 的卡空着时给 15849 MiB；
/// 没读过空卡上能用的（卡池里的卡）与此刻的额度相同。不要卡。
/// 判别力（`crates/mutations.tsv`「GPU 取卡 空着时的额度照此刻的空闲算」）：空着时也照此刻的空闲算，这条红。
#[test]
fn the_memory_budget_of_a_card_when_it_is_empty_follows_the_usable_memory_of_the_empty_card() {
    assert_eq!(
        super::memory_budget_when_the_card_is_empty_mebibytes_of(Some(16_384), 2_254, Some(15_849)),
        15_849,
        "空着时照空卡上能用的算"
    );
    assert_eq!(
        super::memory_budget_when_the_card_is_empty_mebibytes_of(Some(16_384), 2_254, None),
        2_254,
        "没读过空卡上能用的，与此刻的额度相同"
    );
}
