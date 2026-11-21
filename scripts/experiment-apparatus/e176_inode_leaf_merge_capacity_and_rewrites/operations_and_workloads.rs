//! 操作与随机数、删除次序、登记的序列与它们的生成器。

#[cfg(test)]
use super::*;

/// 负载序列里的一个操作：建一个 inode（写一条记录），或 unlink 最后一批（删一条记录）。一次发布一个操作。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    Create(u64),
    Unlink(u64),
}

impl Operation {
    /// 进操作流指纹的 9 字节：一个标记字节加小端 8 字节号。
    #[must_use]
    pub fn fingerprint_bytes(self) -> [u8; 9] {
        let (marker, inode) = match self {
            Operation::Create(inode) => (b'C', inode),
            Operation::Unlink(inode) => (b'U', inode),
        };
        let mut bytes = [0u8; 9];
        bytes[0] = marker;
        bytes[1..].copy_from_slice(&inode.to_le_bytes());
        bytes
    }
}

/// splitmix64 伪随机源，种子即初值（跑前登记第六节 W-R 那一行）。
#[derive(Clone, Debug)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    #[must_use]
    pub fn seeded(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_value(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut mixed = self.state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        mixed ^ (mixed >> 31)
    }
}

/// W-S 每轮删的次序：升序删最小的（最老的），降序删最大的（最新的）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeletionOrder {
    SmallestFirst,
    LargestFirst,
}

/// W-A 每批留下哪些：留 1/s 的删 (n − b) mod s ≠ 0 的；留 3/4 的删 (n − b) mod 4 = 3 的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeptInEachBatch {
    OneIn(u64),
    ThreeInFour,
}

/// 三族负载（跑前登记第六节「负载序列」那张表）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Workload {
    /// W-R：每一步取一个 53 位均匀数 u，活集合为空或 u < p 就插入，否则删活集合（按插入次序追加的 Vec）第 next() mod 长度 个、swap_remove 取出。
    /// p 写成百分数，比较按整数做：u × 100 < p_percent × 2^53。
    RandomInsertDelete { insert_probability_percent: u64 },
    /// W-S：一轮插入 A 条，再按次序删活集合里最小（或最大）的 D 个。
    SequentialDelete { inserted_per_round: u64, deleted_per_round: u64, order: DeletionOrder },
    /// W-A：一轮插入 A 条连号 b … b + A − 1，再按号升序删掉这一批里的一部分。
    AlternateDelete { batch_size: u64, kept: KeptInEachBatch },
}

/// 一个登记的取样点：名字、负载、种子（只 W-R 用）、是不是主取样点。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegisteredSequence {
    pub name: &'static str,
    pub family: &'static str,
    pub workload: Workload,
    pub seed: u64,
    pub is_main_sampling_point: bool,
}

/// 第一段的 22 条序列，次序就是产物里的次序。
#[must_use]
pub fn registered_sequences() -> Vec<RegisteredSequence> {
    let mut sequences = Vec::new();
    let random_points: [(u64, &'static [&'static str; 5], bool); 3] = [
        (55, &["wr_p055_seed1", "wr_p055_seed2", "wr_p055_seed3", "wr_p055_seed4", "wr_p055_seed5"], true),
        (75, &["wr_p075_seed1", "wr_p075_seed2", "wr_p075_seed3", "wr_p075_seed4", "wr_p075_seed5"], false),
        (95, &["wr_p095_seed1", "wr_p095_seed2", "wr_p095_seed3", "wr_p095_seed4", "wr_p095_seed5"], false),
    ];
    for (insert_probability_percent, names, is_main_sampling_point) in random_points {
        for (seed_offset, name) in names.iter().enumerate() {
            sequences.push(RegisteredSequence {
                name,
                family: "W-R",
                workload: Workload::RandomInsertDelete { insert_probability_percent },
                seed: u64::try_from(seed_offset).expect("五个种子") + 1,
                is_main_sampling_point,
            });
        }
    }
    let sequential_points = [
        ("ws_a1000_d500_smallest_first", 500, DeletionOrder::SmallestFirst, true),
        ("ws_a1000_d950_smallest_first", 950, DeletionOrder::SmallestFirst, false),
        ("ws_a1000_d500_largest_first", 500, DeletionOrder::LargestFirst, false),
    ];
    for (name, deleted_per_round, order, is_main_sampling_point) in sequential_points {
        sequences.push(RegisteredSequence {
            name,
            family: "W-S",
            workload: Workload::SequentialDelete { inserted_per_round: 1000, deleted_per_round, order },
            seed: 0,
            is_main_sampling_point,
        });
    }
    let alternate_points = [
        ("wa_keep1of2_a1000", 1000, KeptInEachBatch::OneIn(2), true),
        ("wa_keep1of2_a2330", 2330, KeptInEachBatch::OneIn(2), false),
        ("wa_keep1of233_a2330", 2330, KeptInEachBatch::OneIn(233), false),
        ("wa_keep3of4_a1000", 1000, KeptInEachBatch::ThreeInFour, false),
    ];
    for (name, batch_size, kept, is_main_sampling_point) in alternate_points {
        sequences.push(RegisteredSequence {
            name,
            family: "W-A",
            workload: Workload::AlternateDelete { batch_size, kept },
            seed: 0,
            is_main_sampling_point,
        });
    }
    sequences
}

/// W-S、W-A 一轮里走到哪一步。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RoundPhase {
    Creating { remaining: u64 },
    Deleting { remaining: u64 },
    DeletingFromBatch { batch_start: u64, next_offset: u64 },
}

/// 序列生成器：自己维护活 inode 集合，不看任何一条臂的状态；号从 1 起单调递增。
#[derive(Clone, Debug)]
pub struct WorkloadGenerator {
    pub(super) workload: Workload,
    random: SplitMix64,
    next_inode: u64,
    live_in_creation_order: Vec<u64>,
    live_ascending: std::collections::VecDeque<u64>,
    phase: RoundPhase,
}

impl WorkloadGenerator {
    #[must_use]
    pub fn new(workload: Workload, seed: u64) -> Self {
        let phase = match workload {
            Workload::RandomInsertDelete { .. } => RoundPhase::Creating { remaining: 0 },
            Workload::SequentialDelete { inserted_per_round, .. } => RoundPhase::Creating { remaining: inserted_per_round },
            Workload::AlternateDelete { batch_size, .. } => RoundPhase::Creating { remaining: batch_size },
        };
        Self {
            workload,
            random: SplitMix64::seeded(seed),
            next_inode: 1,
            live_in_creation_order: Vec::new(),
            live_ascending: std::collections::VecDeque::new(),
            phase,
        }
    }

    fn create(&mut self) -> Operation {
        let inode = self.next_inode;
        self.next_inode += 1;
        Operation::Create(inode)
    }

    pub fn next_operation(&mut self) -> Operation {
        match self.workload {
            Workload::RandomInsertDelete { insert_probability_percent } => {
                let uniform_53_bits = self.random.next_value() >> 11;
                let inserts = self.live_in_creation_order.is_empty()
                    || u128::from(uniform_53_bits) * 100 < u128::from(insert_probability_percent) << 53;
                if inserts {
                    let operation = self.create();
                    let Operation::Create(inode) = operation else { unreachable!("create 只交回 Create") };
                    self.live_in_creation_order.push(inode);
                    operation
                } else {
                    let live_count = u64::try_from(self.live_in_creation_order.len()).expect("活数装得进 u64");
                    let pick = usize::try_from(self.random.next_value() % live_count).expect("下标小于活数");
                    Operation::Unlink(self.live_in_creation_order.swap_remove(pick))
                }
            }
            Workload::SequentialDelete { inserted_per_round, deleted_per_round, order } => loop {
                match self.phase {
                    RoundPhase::Creating { remaining: 0 } => self.phase = RoundPhase::Deleting { remaining: deleted_per_round },
                    RoundPhase::Deleting { remaining: 0 } => self.phase = RoundPhase::Creating { remaining: inserted_per_round },
                    RoundPhase::Creating { remaining } => {
                        self.phase = RoundPhase::Creating { remaining: remaining - 1 };
                        let operation = self.create();
                        let Operation::Create(inode) = operation else { unreachable!("create 只交回 Create") };
                        self.live_ascending.push_back(inode);
                        return operation;
                    }
                    RoundPhase::Deleting { remaining } => {
                        self.phase = RoundPhase::Deleting { remaining: remaining - 1 };
                        let inode = match order {
                            DeletionOrder::SmallestFirst => self.live_ascending.pop_front(),
                            DeletionOrder::LargestFirst => self.live_ascending.pop_back(),
                        };
                        return Operation::Unlink(inode.expect("每轮先插 A 条再删 D < A 条，活集合不空"));
                    }
                    RoundPhase::DeletingFromBatch { .. } => unreachable!("W-S 不走按批删"),
                }
            },
            Workload::AlternateDelete { batch_size, kept } => loop {
                match self.phase {
                    RoundPhase::Creating { remaining: 0 } => {
                        self.phase = RoundPhase::DeletingFromBatch { batch_start: self.next_inode - batch_size, next_offset: 0 };
                    }
                    RoundPhase::Creating { remaining } => {
                        self.phase = RoundPhase::Creating { remaining: remaining - 1 };
                        return self.create();
                    }
                    RoundPhase::DeletingFromBatch { batch_start, next_offset } => {
                        let deleted_offset = (next_offset..batch_size).find(|offset| match kept {
                            KeptInEachBatch::OneIn(kept_one_in) => offset % kept_one_in != 0,
                            KeptInEachBatch::ThreeInFour => offset % 4 == 3,
                        });
                        match deleted_offset {
                            Some(offset) => {
                                self.phase = RoundPhase::DeletingFromBatch { batch_start, next_offset: offset + 1 };
                                return Operation::Unlink(batch_start + offset);
                            }
                            None => self.phase = RoundPhase::Creating { remaining: batch_size },
                        }
                    }
                    RoundPhase::Deleting { .. } => unreachable!("W-A 不走按次序删"),
                }
            },
        }
    }
}

#[cfg(test)]
#[path = "operations_and_workloads_tests.rs"]
mod tests;
