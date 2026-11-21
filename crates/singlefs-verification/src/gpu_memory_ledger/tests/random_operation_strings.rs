//! 随机操作串：账知道设备丢了之后，借、还、丢失与重起怎么交错，都不借出比卡上剩下的多。

use super::{ledger_on_a_card_under_the_ledger, room_now, room_truly_left};
use crate::gpu_memory_ledger::{
    DeviceStartedOnTheBooks, DeviceWaitBeforeReturningTheLease, GpuMemoryBooking,
};

/// 随机操作串里的一步（[`random_operation_strings_never_lend_more_than_the_card_has_left_once_the_ledger_knows_of_the_loss`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RandomStepOnTheCard {
    /// 借 1 000、4 000、9 000 或 15 000（借不到就不借）。
    Book,
    /// 设备没丢时，把头一笔没建的借据建满。
    BuildTheFirstUnbuiltLease,
    /// 随机丢一笔借据：设备没丢就等成（它与之前没等成的缓冲都交回驱动），丢了就没等成。
    DropALeaseAfterTheWait,
    /// 随机丢一笔借据，没等成（建过的缓冲留在卡上）。
    DropALeaseAfterAFailedWait,
    /// 设备丢了：它的驱动那一份与卡上的缓冲都成了丢了的设备留在卡上的。
    LoseTheDevice,
    /// 卡池借之前那一步（设备丢了就交丢了，没丢的 poll 交还有在飞的）。
    StepBeforeBooking,
    /// 设备丢了就重起（驱动那一份照此刻判，放得下才起）。
    RestartTheLostDevice,
    /// 放掉最早的那一台丢了的设备（它留在卡上的都交回驱动要等最后一台放掉）。
    ReleaseTheEarliestLostDevice,
}

/// 随机操作串每一步从这里按下标取（与 T5 的操作号同序：0 借、1 建、2 等成丢、3 没等成丢、4 丢设备、5 借之前那一步、6 重起、7 放掉）。
const RANDOM_STEPS_ON_THE_CARD: [RandomStepOnTheCard; 8] = [
    RandomStepOnTheCard::Book,
    RandomStepOnTheCard::BuildTheFirstUnbuiltLease,
    RandomStepOnTheCard::DropALeaseAfterTheWait,
    RandomStepOnTheCard::DropALeaseAfterAFailedWait,
    RandomStepOnTheCard::LoseTheDevice,
    RandomStepOnTheCard::StepBeforeBooking,
    RandomStepOnTheCard::RestartTheLostDevice,
    RandomStepOnTheCard::ReleaseTheEarliestLostDevice,
];

/// 随机操作串借的那几种量（MiB）。
const RANDOM_BOOKING_SIZES_MEBIBYTES: [u64; 4] = [1_000, 4_000, 9_000, 15_000];

/// 线性同余的随机数（T5 那一个，同一个种子出同一串）：种子先乘加混一次，之后每取一次乘加一次、取高 31 位。
struct OperationStringGenerator {
    state: u64,
}

impl OperationStringGenerator {
    const SEED_MULTIPLIER: u64 = 2_862_933_555_777_941_757;
    const SEED_INCREMENT: u64 = 3_037_000_493;
    const STEP_MULTIPLIER: u64 = 6_364_136_223_846_793_005;
    const STEP_INCREMENT: u64 = 1_442_695_040_888_963_407;
    const DISCARDED_LOW_BITS: u32 = 33;

    fn from_seed(seed: u64) -> Self {
        Self {
            state: seed
                .wrapping_mul(Self::SEED_MULTIPLIER)
                .wrapping_add(Self::SEED_INCREMENT),
        }
    }

    /// `0..bound` 里的一个（`bound` 不为 0）。
    fn next_below(&mut self, bound: usize) -> usize {
        self.state = self
            .state
            .wrapping_mul(Self::STEP_MULTIPLIER)
            .wrapping_add(Self::STEP_INCREMENT);
        let bound_as_u64 = u64::try_from(bound).expect("下标的上界装得进 u64");
        usize::try_from((self.state >> Self::DISCARDED_LOW_BITS) % bound_as_u64)
            .expect("比 bound 小，装得进 usize")
    }
}

/// 随机操作串里一笔活着的借据建到哪一步。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LeaseBuiltOnTheCard {
    NotBuilt,
    BuiltOnTheDeviceInUse,
    /// 建在后来丢了的那一台上（缓冲已经算进丢了的设备留在卡上的）。
    BuiltOnALostDevice,
}

/// （随机操作串拦「告诉账丢了只销那几笔、不照重起记」与 Z6；
/// K3-3 改了之后借之前那一步看见丢了就记）：一张假卡、一台设备，4 000 个种子各 14 步随机的借、建、丢借据（等成或没等成）、设备丢了、
/// 借之前那一步、重起、放掉旧设备。设备没丢、或丢了而账已经知道（丢了之后走过借之前那一步，或重起那一步判过）的每一步之后：
/// 能借的不超过卡上真能再给的（建过没还的缓冲、丢了的设备留在卡上的都不在空闲里，活着而没建的借据还要建）。
#[test]
fn random_operation_strings_never_lend_more_than_the_card_has_left_once_the_ledger_knows_of_the_loss(
) {
    const SEEDS: u64 = 4_000;
    const STEPS_IN_A_STRING: usize = 14;
    const DRIVER_HOLDS_ONCE_STARTED: u64 = 600;
    let mut states_checked = 0usize;
    let mut first_overgiving_strings: Vec<String> = Vec::new();
    let mut overgiving_states = 0usize;
    for seed in 0..SEEDS {
        let mut generator = OperationStringGenerator::from_seed(seed);
        let (card, ledger) = ledger_on_a_card_under_the_ledger();
        let mut first = ledger.book_on_the_card(0, 1).expect("空卡借得到");
        let first_device = ledger.record_the_device_started_on_the_card(0);
        {
            let mut card = card.lock().expect("假卡不 panic");
            card.device_started = true;
            card.driver_occupied_mebibytes = DRIVER_HOLDS_ONCE_STARTED;
        }
        first.take_in_the_wait_before_returning(
            &first_device,
            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
        );
        drop(first);
        let mut device_in_use: std::rc::Rc<DeviceStartedOnTheBooks<'_>> =
            std::rc::Rc::new(first_device);
        let mut lost_devices_kept: Vec<std::rc::Rc<DeviceStartedOnTheBooks<'_>>> = Vec::new();
        let mut alive: Vec<(GpuMemoryBooking<'_>, LeaseBuiltOnTheCard, u64)> = Vec::new();
        let mut is_the_device_lost = false;
        let mut does_the_ledger_know_of_the_loss = false;
        let mut built_by_leases_dropped_after_a_failed_wait_mebibytes: u64 = 0;
        let mut steps_taken = String::new();
        for _step_in_the_string in 0..STEPS_IN_A_STRING {
            let step =
                RANDOM_STEPS_ON_THE_CARD[generator.next_below(RANDOM_STEPS_ON_THE_CARD.len())];
            match step {
                RandomStepOnTheCard::Book => {
                    let mebibytes = RANDOM_BOOKING_SIZES_MEBIBYTES
                        [generator.next_below(RANDOM_BOOKING_SIZES_MEBIBYTES.len())];
                    if let Ok(booking) = ledger.book_on_the_card(0, mebibytes) {
                        alive.push((booking, LeaseBuiltOnTheCard::NotBuilt, mebibytes));
                        steps_taken.push_str(&format!("借{mebibytes} "));
                    }
                }
                RandomStepOnTheCard::BuildTheFirstUnbuiltLease => {
                    if !is_the_device_lost {
                        if let Some(unbuilt) =
                            alive.iter_mut().find(|(_booking, built, _mebibytes)| {
                                *built == LeaseBuiltOnTheCard::NotBuilt
                            })
                        {
                            card.lock().expect("假卡不 panic").built_mebibytes += unbuilt.2;
                            unbuilt.1 = LeaseBuiltOnTheCard::BuiltOnTheDeviceInUse;
                            steps_taken.push_str("建 ");
                        }
                    }
                }
                RandomStepOnTheCard::DropALeaseAfterTheWait
                | RandomStepOnTheCard::DropALeaseAfterAFailedWait => {
                    if !alive.is_empty() {
                        let (mut booking, built, mebibytes) =
                            alive.remove(generator.next_below(alive.len()));
                        let is_the_wait_finished = step
                            == RandomStepOnTheCard::DropALeaseAfterTheWait
                            && !is_the_device_lost;
                        booking.take_in_the_wait_before_returning(
                            &device_in_use,
                            if is_the_wait_finished {
                                DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished
                            } else {
                                DeviceWaitBeforeReturningTheLease::TheWaitFailed
                            },
                        );
                        let built_on_the_device_in_use =
                            if built == LeaseBuiltOnTheCard::BuiltOnTheDeviceInUse {
                                mebibytes
                            } else {
                                0
                            };
                        if is_the_wait_finished {
                            card.lock().expect("假卡不 panic").built_mebibytes -=
                                built_by_leases_dropped_after_a_failed_wait_mebibytes
                                    + built_on_the_device_in_use;
                            built_by_leases_dropped_after_a_failed_wait_mebibytes = 0;
                        } else {
                            built_by_leases_dropped_after_a_failed_wait_mebibytes +=
                                built_on_the_device_in_use;
                        }
                        drop(booking);
                        steps_taken.push_str(if is_the_wait_finished {
                            "等成丢 "
                        } else {
                            "没等成丢 "
                        });
                    }
                }
                RandomStepOnTheCard::LoseTheDevice => {
                    if !is_the_device_lost {
                        is_the_device_lost = true;
                        does_the_ledger_know_of_the_loss = false;
                        let mut card = card.lock().expect("假卡不 panic");
                        card.left_by_a_lost_device_mebibytes +=
                            card.driver_occupied_mebibytes + card.built_mebibytes;
                        card.built_mebibytes = 0;
                        card.driver_occupied_mebibytes = 0;
                        card.device_started = false;
                        for (_booking, built, _mebibytes) in &mut alive {
                            if *built == LeaseBuiltOnTheCard::BuiltOnTheDeviceInUse {
                                *built = LeaseBuiltOnTheCard::BuiltOnALostDevice;
                            }
                        }
                        built_by_leases_dropped_after_a_failed_wait_mebibytes = 0;
                        steps_taken.push_str("丢设备 ");
                    }
                }
                RandomStepOnTheCard::StepBeforeBooking => {
                    ledger.take_in_the_devices_kept_before_booking(
                        |_card_position| Some(std::rc::Rc::clone(&device_in_use)),
                        |_kept| is_the_device_lost,
                        |_kept| false,
                        |kept| &**kept,
                    );
                    if is_the_device_lost {
                        does_the_ledger_know_of_the_loss = true;
                    }
                    steps_taken.push_str("借之前 ");
                }
                RandomStepOnTheCard::RestartTheLostDevice => {
                    if is_the_device_lost {
                        does_the_ledger_know_of_the_loss = true;
                        if ledger
                            .prepare_the_driver_share_before_starting_the_device(0)
                            .is_ok()
                        {
                            let replacement = ledger.record_the_device_started_on_the_card(0);
                            lost_devices_kept.push(std::mem::replace(
                                &mut device_in_use,
                                std::rc::Rc::new(replacement),
                            ));
                            is_the_device_lost = false;
                            let mut card = card.lock().expect("假卡不 panic");
                            card.device_started = true;
                            card.driver_occupied_mebibytes = DRIVER_HOLDS_ONCE_STARTED;
                            steps_taken.push_str("重起 ");
                        } else {
                            steps_taken.push_str("重起被拒 ");
                        }
                    }
                }
                RandomStepOnTheCard::ReleaseTheEarliestLostDevice => {
                    if !lost_devices_kept.is_empty() {
                        drop(lost_devices_kept.remove(0));
                        if lost_devices_kept.is_empty() {
                            card.lock()
                                .expect("假卡不 panic")
                                .left_by_a_lost_device_mebibytes = 0;
                        }
                        steps_taken.push_str("放掉 ");
                    }
                }
            }
            let unbuilt_mebibytes: u64 = alive
                .iter()
                .filter(|(_booking, built, _mebibytes)| *built == LeaseBuiltOnTheCard::NotBuilt)
                .map(|(_booking, _built, mebibytes)| *mebibytes)
                .sum();
            if !is_the_device_lost || does_the_ledger_know_of_the_loss {
                states_checked += 1;
                let room = room_now(&ledger);
                let room_truly_left_on_the_card = room_truly_left(&card, unbuilt_mebibytes);
                if room > room_truly_left_on_the_card {
                    overgiving_states += 1;
                    if first_overgiving_strings.len() < 5 {
                        first_overgiving_strings.push(format!(
                                "种子 {seed}：{steps_taken}能借 {room}，卡上真能再给 {room_truly_left_on_the_card}"
                            ));
                    }
                }
            }
        }
        drop(alive);
        drop(lost_devices_kept);
        drop(device_in_use);
    }
    assert!(states_checked > 0, "随机串里至少有一步落在账知道的状态上");
    assert_eq!(
            overgiving_states, 0,
            "核了 {states_checked} 步，能借的都不超过卡上真能再给的；头几条多给的：{first_overgiving_strings:#?}"
        );
}
