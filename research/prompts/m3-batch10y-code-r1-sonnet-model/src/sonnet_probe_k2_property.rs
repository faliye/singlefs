//! 正推腿探针（草稿副本里的，不进仓）：随机读数 × 随机账 × 随机额度，查 `card_room_for_this_process` 给的此刻能借的有没有超过现读的空闲。
use singlefs_harness::gpu_memory_ledger::{
    card_ledger_capacity_mebibytes, card_room_for_this_process, BookedByThisProcessOnACard, CardMemoryReading,
    DriverShareOnTheCard, OccupiedByThisProcess,
};

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    fn below(&mut self, n: u64) -> u64 { self.next() % n }
}

#[test]
fn room_now_never_exceeds_the_free_memory_read() {
    const DRIVER: u64 = 2560;
    let mut rng = Lcg(20261007);
    let (mut samples, mut over_free, mut max_over) = (0u64, 0u64, 0u64);
    for _ in 0..3_000_000 {
        let total_less_reserved = 4096 + rng.below(40_000);
        let free = rng.below(total_less_reserved + 1);
        let occupied = match rng.below(3) {
            0 => OccupiedByThisProcess::NotListed,
            1 => OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes: rng.below(total_less_reserved + 1) },
            _ => OccupiedByThisProcess::AlsoOutsideTheBooks { mebibytes: rng.below(total_less_reserved + 1) },
        };
        let booked = BookedByThisProcessOnACard {
            leases_mebibytes: rng.below(30_000),
            driver_share: match rng.below(3) {
                0 => DriverShareOnTheCard::NotRegistered,
                1 => DriverShareOnTheCard::RegisteredDeviceNotStarted,
                _ => DriverShareOnTheCard::DeviceStarted,
            },
            returned_possibly_still_on_the_card_mebibytes: rng.below(30_000),
        };
        let env_quota = if rng.below(2) == 0 { None } else { Some(rng.below(40_000)) };
        let configured = if rng.below(2) == 0 { None } else { Some(rng.below(40_000)) };
        let capacity = |may_hold: u64| {
            let budget = match configured {
                Some(q) => q.min(may_hold),
                None => may_hold / 4 * 3,
            };
            card_ledger_capacity_mebibytes(env_quota, configured, budget, may_hold, DRIVER)
        };
        let reading = CardMemoryReading { free_mebibytes: free, total_less_reserved_mebibytes: total_less_reserved, occupied_by_this_process: occupied };
        let room = card_room_for_this_process(reading, booked, DRIVER, &capacity);
        samples += 1;
        if room.room_now_mebibytes > free {
            over_free += 1;
            max_over = max_over.max(room.room_now_mebibytes - free);
        }
    }
    println!("SAMPLES {samples} room_now>free: {over_free} max_over={max_over}");
}
