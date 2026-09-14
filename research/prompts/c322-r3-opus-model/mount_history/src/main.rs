//! C322（取号那一步的屏障怎么放没有条款） 第三轮攻方腿（Opus）的模型 M：只用 std，没有 I/O、随机源与并发，确定性。
//! 从第一个事务写完起（两盘槽 0 = (世代号 4, 实例 1)、槽 1 = (5, 1)；根环实例 {0, 1}；单元与记录带 {0, 1}），
//! 连着三次可写挂载（实例切换的取号走同一个过程），每次按臂取号、写超级块，穷举下列分支：
//!   取号两写各自报 Ok / Err；报 Err 的那一份其实落没落、重读看不看得见；重试写同一槽，还是按重读出的槽重算世代号；
//!   屏障（甲″ 一道 / 乙′ 两道）报 Ok，还是报错并丢掉那块盘的易失缓存；报错之后按全或无失败回卷，还是重发屏障后继续；
//!   回卷写的实例代号取哪个；取号之后本实例做到哪一步（什么都没做 / 单元或记录落了 / 根落了且下一次读得出 /
//!   根落了而下一次读不出 / 根与轮换都发了）；任何一道完成了的屏障之前的写，崩溃时各自落或没落。
//! 超级块槽写按扇区原子（481 字节住第一个 512 扇区、补齐恒 0，撕裂 = 整份旧或整份新），与层 0 的「撕裂态并进没持久」同。
//! 一次取号「撞号」= 它发出的号已经被盘上某个单元、记录或根带着。
//! 臂：世代号规则（G2 每盘 + 1 / G1 全池 + 1）× 取号读法（全部自证过的槽 / 每盘择到的那一份）× 屏障（甲″ / 乙′）
//!    × 回卷写的实例代号（新号 − 1 / 这块盘择到的旧号 / 被取号写盖掉的那一槽原来的号 / 不回卷）
//!    × 重试（同一槽 / 重读重算）× 屏障报错之后（全或无失败 / 重发屏障后继续）。

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Slot {
    generation: u32,
    instance: u32,
}

/// 盘上的样子：两盘两槽；带着各实例代号的单元 / 记录、读得出的根、下一次挂载读不出的根，按实例代号的位记。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Pool {
    disks: [[Option<Slot>; 2]; 2],
    carried_by_units_or_records: u64,
    readable_roots: u64,
    unreadable_roots: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SlotWrite {
    disk: usize,
    slot: usize,
    content: Slot,
}

const fn bit(instance: u32) -> u64 {
    1u64 << instance
}

fn highest_instance(mask: u64) -> u32 {
    if mask == 0 {
        0
    } else {
        63 - mask.leading_zeros()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GenerationRule {
    PerDiskPlusOne,
    PoolWidePlusOne,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Reading {
    EverySelfVerifiedSlot,
    ChosenSlotPerDisk,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BarrierPlacement {
    JiaDoublePrime,
    YiPrime,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RollbackCode {
    NewCodeMinusOne,
    DiskChosenBefore,
    OverwrittenSlotBefore,
    NoRollback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RetryTarget {
    SameSlot,
    RereadSlots,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BarrierErrorPolicy {
    FailAcquisition,
    RetryFlushThenProceed,
}

#[derive(Clone, Copy, Debug)]
struct Arm {
    rule: GenerationRule,
    reading: Reading,
    barrier: BarrierPlacement,
    rollback: RollbackCode,
    retry: RetryTarget,
    policy: BarrierErrorPolicy,
}

/// 一条历史里的一步，只在打印第一条撞号历史时渲染。
#[derive(Clone, Copy, Debug)]
enum Event {
    Acquire { code: u32, collides: bool },
    Issued { what: &'static str, write: SlotWrite },
    Fault(&'static str),
    Reached(&'static str),
    CrashLandedMask(u32),
}

/// 一次挂载的一个结局：崩溃（或挂载结束）时盘上的样子、这次挂载注入的故障数、这次取号撞没撞号、这次挂载的步骤。
#[derive(Clone, Debug)]
struct Outcome {
    end: Pool,
    faults: u32,
    collides: bool,
    events: Vec<Event>,
}

impl Pool {
    fn after_first_transaction() -> Self {
        let disk = [
            Some(Slot { generation: 4, instance: 1 }),
            Some(Slot { generation: 5, instance: 1 }),
        ];
        Pool {
            disks: [disk, disk],
            carried_by_units_or_records: bit(0) | bit(1),
            readable_roots: bit(0) | bit(1),
            unreadable_roots: 0,
        }
    }
    fn write(&mut self, write: &SlotWrite) {
        self.disks[write.disk][write.slot] = Some(write.content);
    }
    fn valid_slots(&self, disk: usize) -> impl Iterator<Item = Slot> + '_ {
        self.disks[disk].iter().flatten().copied()
    }
    fn disk_highest_generation(&self, disk: usize) -> u32 {
        self.valid_slots(disk).map(|slot| slot.generation).max().unwrap_or(0)
    }
    fn pool_highest_generation(&self) -> u32 {
        (0..2).map(|disk| self.disk_highest_generation(disk)).max().unwrap_or(0)
    }
    /// 择槽（D22（单元原子性怎么合成） 已定项 16）：校验和过且世代号最大的那一槽。
    fn disk_chosen(&self, disk: usize) -> Option<Slot> {
        self.valid_slots(disk).max_by_key(|slot| slot.generation)
    }
    fn disk_highest_instance(&self, disk: usize) -> u32 {
        self.valid_slots(disk).map(|slot| slot.instance).max().unwrap_or(0)
    }
    fn every_slot_highest_instance(&self) -> u32 {
        (0..2).map(|disk| self.disk_highest_instance(disk)).max().unwrap_or(0)
    }
    fn chosen_highest_instance(&self) -> u32 {
        (0..2)
            .filter_map(|disk| self.disk_chosen(disk))
            .map(|slot| slot.instance)
            .max()
            .unwrap_or(0)
    }
    fn every_carrier(&self) -> u64 {
        self.carried_by_units_or_records | self.readable_roots | self.unreadable_roots
    }
    /// 取号：max(超级块按读法, 根环里读得出的根) + 1（D18（块里携带什么信息） 第 879 行）。
    fn next_code(&self, reading: Reading) -> u32 {
        let from_superblocks = match reading {
            Reading::EverySelfVerifiedSlot => self.every_slot_highest_instance(),
            Reading::ChosenSlotPerDisk => self.chosen_highest_instance(),
        };
        from_superblocks.max(highest_instance(self.readable_roots)) + 1
    }
}

/// 一次超级块槽写：世代号按规则、从实现相信的盘上状态算，槽 = 世代号 mod 2（D22（单元原子性怎么合成） 已定项 16）。
fn slot_write(belief: &Pool, disk: usize, rule: GenerationRule, instance: u32) -> SlotWrite {
    let generation = match rule {
        GenerationRule::PerDiskPlusOne => belief.disk_highest_generation(disk) + 1,
        GenerationRule::PoolWidePlusOne => belief.pool_highest_generation() + 1,
    };
    SlotWrite {
        disk,
        slot: usize::try_from(generation % 2).expect("0 或 1"),
        content: Slot { generation, instance },
    }
}

/// 一道完成了的屏障之前发出的写，崩溃时各自落或没落；按发出次序施加，同一槽后发的盖先发的。
fn crash_states(physical: &Pool, pending: &[SlotWrite]) -> Vec<(Pool, u32)> {
    (0..(1u32 << pending.len()))
        .map(|mask| {
            let mut pool = *physical;
            for (position, write) in pending.iter().enumerate() {
                if mask & (1 << position) != 0 {
                    pool.write(write);
                }
            }
            (pool, mask)
        })
        .collect()
}

/// 回卷写：只回卷返回过 Ok 的那几份（D18（块里携带什么信息） 第 879 行「先把已经写出的那几份回卷成旧代号」），
/// 世代号按实现相信已落的状态算；「旧代号」三种读法见 RollbackCode。
fn rollback_writes(
    start: &Pool,
    belief: &Pool,
    returned_ok: &[Option<SlotWrite>; 2],
    code: u32,
    arm: &Arm,
) -> Vec<SlotWrite> {
    let mut writes = Vec::new();
    for disk in 0..2 {
        let Some(acquisition_write) = returned_ok[disk] else {
            continue;
        };
        let old_instance = match arm.rollback {
            RollbackCode::NewCodeMinusOne => code - 1,
            RollbackCode::DiskChosenBefore => {
                start.disk_chosen(disk).map_or(code - 1, |slot| slot.instance)
            }
            RollbackCode::OverwrittenSlotBefore => start.disks[disk][acquisition_write.slot]
                .map_or(code - 1, |slot| slot.instance),
            RollbackCode::NoRollback => return Vec::new(),
        };
        writes.push(slot_write(belief, disk, arm.rule, old_instance));
    }
    writes
}

fn finish(end: Pool, faults: u32, collides: bool, events: &[Event], extra: &[Event]) -> Outcome {
    let mut all = events.to_vec();
    all.extend_from_slice(extra);
    Outcome {
        end,
        faults,
        collides,
        events: all,
    }
}

/// 那道（些）屏障完成之后本实例做到哪一步。`physical` 是屏障之后盘上真有的，`belief` 是实现相信的
/// （屏障报错并丢了缓存、又重发屏障继续时两者不同）。单元与记录都只带实例代号，在这个模型里是同一件事。
fn progress(
    physical: &Pool,
    belief: &Pool,
    code: u32,
    arm: &Arm,
    faults: u32,
    collides: bool,
    events: &[Event],
) -> Vec<Outcome> {
    let mut outcomes = vec![finish(*physical, faults, collides, events, &[Event::Reached("nothing_after_barrier")])];
    let mut with_units = *physical;
    with_units.carried_by_units_or_records |= bit(code);
    outcomes.push(finish(with_units, faults, collides, events, &[Event::Reached("units_or_record_no_root")]));
    let mut with_root = with_units;
    with_root.readable_roots |= bit(code);
    outcomes.push(finish(with_root, faults, collides, events, &[Event::Reached("root_readable_next_mount")]));
    let mut with_unreadable_root = with_units;
    with_unreadable_root.unreadable_roots |= bit(code);
    outcomes.push(finish(
        with_unreadable_root,
        faults + 1,
        collides,
        events,
        &[Event::Reached("root_unreadable_next_mount")],
    ));
    let rotation: Vec<SlotWrite> = (0..2).map(|disk| slot_write(belief, disk, arm.rule, code)).collect();
    for (end, mask) in crash_states(&with_root, &rotation) {
        let mut extra = vec![Event::Reached("root_then_rotation")];
        extra.extend(rotation.iter().map(|write| Event::Issued { what: "rotation", write: *write }));
        extra.push(Event::CrashLandedMask(mask));
        outcomes.push(finish(end, faults, collides, events, &extra));
    }
    outcomes
}

/// 甲″：两盘的取号写一起发，之后一道屏障（P 第 1 句）。
fn mount_jia_double_prime(start: &Pool, code: u32, collides: bool, arm: &Arm) -> Vec<Outcome> {
    let acquisition = [slot_write(start, 0, arm.rule, code), slot_write(start, 1, arm.rule, code)];
    let issued = vec![
        Event::Acquire { code, collides },
        Event::Issued { what: "acquisition", write: acquisition[0] },
        Event::Issued { what: "acquisition", write: acquisition[1] },
    ];
    let mut outcomes = Vec::new();
    let mut believed = *start;
    believed.write(&acquisition[0]);
    believed.write(&acquisition[1]);
    for (end, mask) in crash_states(start, &acquisition) {
        let extra = [Event::Reached("both_ok_crash_before_barrier"), Event::CrashLandedMask(mask)];
        outcomes.push(finish(end, 0, collides, &issued, &extra));
    }
    outcomes.extend(progress(&believed, &believed, code, arm, 0, collides, &issued));
    let barrier_errors = [
        ([true, false], "barrier_error_disk0_cache_lost"),
        ([false, true], "barrier_error_disk1_cache_lost"),
        ([true, true], "barrier_error_both_cache_lost"),
    ];
    for (lost, label) in barrier_errors {
        let mut physical = *start;
        for disk in 0..2 {
            if !lost[disk] {
                physical.write(&acquisition[disk]);
            }
        }
        let faults = u32::from(lost[0]) + u32::from(lost[1]);
        let mut events = issued.clone();
        events.push(Event::Fault(label));
        match arm.policy {
            BarrierErrorPolicy::FailAcquisition => {
                let returned_ok = [Some(acquisition[0]), Some(acquisition[1])];
                let rollbacks = rollback_writes(start, &believed, &returned_ok, code, arm);
                events.extend(rollbacks.iter().map(|write| Event::Issued { what: "rollback", write: *write }));
                for (end, mask) in crash_states(&physical, &rollbacks) {
                    outcomes.push(finish(end, faults, collides, &events, &[Event::CrashLandedMask(mask)]));
                }
            }
            BarrierErrorPolicy::RetryFlushThenProceed => {
                events.push(Event::Reached("flush_retried_ok_proceed"));
                outcomes.extend(progress(&physical, &believed, code, arm, faults, collides, &events));
            }
        }
    }
    for failed in 0..2 {
        let succeeded = 1 - failed;
        for failed_write_visible_on_reread in [false, true] {
            if failed_write_visible_on_reread && arm.retry == RetryTarget::SameSlot {
                continue;
            }
            let mut events = issued.clone();
            events.push(Event::Fault(if failed == 0 { "write_error_disk0" } else { "write_error_disk1" }));
            let mut pending = vec![acquisition[0], acquisition[1]];
            if failed_write_visible_on_reread {
                let mut reread = *start;
                reread.write(&acquisition[failed]);
                let retry = slot_write(&reread, failed, arm.rule, code);
                events.push(Event::Issued { what: "retry_after_reread", write: retry });
                pending.push(retry);
            }
            let mut believed_after_failure = *start;
            believed_after_failure.write(&acquisition[succeeded]);
            let mut returned_ok = [None, None];
            returned_ok[succeeded] = Some(acquisition[succeeded]);
            let rollbacks = rollback_writes(start, &believed_after_failure, &returned_ok, code, arm);
            events.extend(rollbacks.iter().map(|write| Event::Issued { what: "rollback", write: *write }));
            pending.extend(rollbacks);
            for (end, mask) in crash_states(start, &pending) {
                outcomes.push(finish(end, 1, collides, &events, &[Event::CrashLandedMask(mask)]));
            }
        }
    }
    let mut events = issued.clone();
    events.push(Event::Fault("write_error_both"));
    for (end, mask) in crash_states(start, &acquisition) {
        outcomes.push(finish(end, 2, collides, &events, &[Event::CrashLandedMask(mask)]));
    }
    outcomes
}

/// 乙′：盘 0 取号写 → 屏障 → 盘 1 取号写 → 屏障（第二轮排除的那一条，这里当对照臂）。
fn mount_yi_prime(start: &Pool, code: u32, collides: bool, arm: &Arm) -> Vec<Outcome> {
    let mut outcomes = Vec::new();
    let first = slot_write(start, 0, arm.rule, code);
    let issued = vec![Event::Acquire { code, collides }, Event::Issued { what: "acquisition", write: first }];
    let with_retry = |base: &Pool, write: SlotWrite, disk: usize, events: &mut Vec<Event>| -> Vec<SlotWrite> {
        let mut pending = vec![write];
        if arm.retry == RetryTarget::RereadSlots {
            let mut reread = *base;
            reread.write(&write);
            let retry = slot_write(&reread, disk, arm.rule, code);
            events.push(Event::Issued { what: "retry_after_reread", write: retry });
            pending.push(retry);
        }
        pending
    };
    let mut events = issued.clone();
    events.push(Event::Fault("write_error_disk0"));
    let pending = with_retry(start, first, 0, &mut events);
    for (end, mask) in crash_states(start, &pending) {
        outcomes.push(finish(end, 1, collides, &events, &[Event::CrashLandedMask(mask)]));
    }
    for (end, mask) in crash_states(start, &[first]) {
        let extra = [Event::Reached("disk0_ok_crash_before_first_barrier"), Event::CrashLandedMask(mask)];
        outcomes.push(finish(end, 0, collides, &issued, &extra));
    }
    let mut durable_first = *start;
    durable_first.write(&first);
    let mut events = issued.clone();
    events.push(Event::Fault("first_barrier_error_disk0_cache_lost"));
    match arm.policy {
        BarrierErrorPolicy::FailAcquisition => {
            let rollbacks = rollback_writes(start, &durable_first, &[Some(first), None], code, arm);
            events.extend(rollbacks.iter().map(|write| Event::Issued { what: "rollback", write: *write }));
            for (end, mask) in crash_states(start, &rollbacks) {
                outcomes.push(finish(end, 1, collides, &events, &[Event::CrashLandedMask(mask)]));
            }
        }
        BarrierErrorPolicy::RetryFlushThenProceed => {
            let second = slot_write(&durable_first, 1, arm.rule, code);
            let mut physical = *start;
            physical.write(&second);
            let mut believed = durable_first;
            believed.write(&second);
            events.push(Event::Issued { what: "acquisition", write: second });
            events.push(Event::Reached("flush_retried_ok_proceed"));
            outcomes.extend(progress(&physical, &believed, code, arm, 1, collides, &events));
        }
    }
    let second = slot_write(&durable_first, 1, arm.rule, code);
    let mut issued_second = issued.clone();
    issued_second.push(Event::Issued { what: "acquisition", write: second });
    let mut events = issued_second.clone();
    events.push(Event::Fault("write_error_disk1"));
    let mut pending = with_retry(&durable_first, second, 1, &mut events);
    let rollbacks = rollback_writes(start, &durable_first, &[Some(first), None], code, arm);
    events.extend(rollbacks.iter().map(|write| Event::Issued { what: "rollback", write: *write }));
    pending.extend(rollbacks);
    for (end, mask) in crash_states(&durable_first, &pending) {
        outcomes.push(finish(end, 1, collides, &events, &[Event::CrashLandedMask(mask)]));
    }
    for (end, mask) in crash_states(&durable_first, &[second]) {
        let extra = [Event::Reached("disk1_ok_crash_before_second_barrier"), Event::CrashLandedMask(mask)];
        outcomes.push(finish(end, 0, collides, &issued_second, &extra));
    }
    let mut durable_both = durable_first;
    durable_both.write(&second);
    outcomes.extend(progress(&durable_both, &durable_both, code, arm, 0, collides, &issued_second));
    let mut events = issued_second.clone();
    events.push(Event::Fault("second_barrier_error_disk1_cache_lost"));
    match arm.policy {
        BarrierErrorPolicy::FailAcquisition => {
            let returned_ok = [Some(first), Some(second)];
            let rollbacks = rollback_writes(start, &durable_both, &returned_ok, code, arm);
            events.extend(rollbacks.iter().map(|write| Event::Issued { what: "rollback", write: *write }));
            for (end, mask) in crash_states(&durable_first, &rollbacks) {
                outcomes.push(finish(end, 1, collides, &events, &[Event::CrashLandedMask(mask)]));
            }
        }
        BarrierErrorPolicy::RetryFlushThenProceed => {
            events.push(Event::Reached("flush_retried_ok_proceed"));
            outcomes.extend(progress(&durable_first, &durable_both, code, arm, 1, collides, &events));
        }
    }
    outcomes
}

/// 第一个事务之后连着几次挂载；第四次取号只算号、不写，用来判第三次挂载留下的盘撞不撞。
const MOUNTS: usize = 3;

fn mount_step(start: &Pool, arm: &Arm) -> Vec<Outcome> {
    let code = start.next_code(arm.reading);
    let collides = start.every_carrier() & bit(code) != 0;
    match arm.barrier {
        BarrierPlacement::JiaDoublePrime => mount_jia_double_prime(start, code, collides, arm),
        BarrierPlacement::YiPrime => mount_yi_prime(start, code, collides, arm),
    }
}

fn second_sentence_red(pool: &Pool, values: &[u32]) -> bool {
    let larger = values.iter().copied().max().unwrap_or(0);
    values.iter().any(|value| *value != larger) && pool.every_carrier() & bit(larger) != 0
}

/// 每次挂载结束时的盘（崩溃态）上 ① ② 各判什么；态分三类：按这条臂的读法下一次取号会撞号（危险）、
/// 不危险但某个被带着的最大号在某块盘上已经没有（单见证）、都不是（良性）。② 三种读法依次是每盘最大、每盘择到的、每个槽。
#[derive(Default)]
struct StateTally {
    states: u64,
    dangerous: u64,
    single_witness: u64,
    benign: u64,
    first_red_on_benign: u64,
    first_green_on_dangerous: u64,
    second_red_on_benign: [u64; 3],
    second_red_on_single_witness: [u64; 3],
    second_green_on_dangerous: [u64; 3],
}

impl StateTally {
    fn observe(&mut self, pool: &Pool, reading: Reading) {
        let carried_highest = highest_instance(pool.every_carrier());
        let dangerous = pool.every_carrier() & bit(pool.next_code(reading)) != 0;
        let single_witness =
            !dangerous && (0..2).any(|disk| pool.disk_highest_instance(disk) < carried_highest);
        let first_red = carried_highest > pool.every_slot_highest_instance();
        let by_disk_highest: Vec<u32> = (0..2).map(|disk| pool.disk_highest_instance(disk)).collect();
        let by_disk_chosen: Vec<u32> =
            (0..2).filter_map(|disk| pool.disk_chosen(disk)).map(|slot| slot.instance).collect();
        let by_every_slot: Vec<u32> = (0..2)
            .flat_map(|disk| pool.valid_slots(disk).map(|slot| slot.instance).collect::<Vec<u32>>())
            .collect();
        let second_red = [
            second_sentence_red(pool, &by_disk_highest),
            second_sentence_red(pool, &by_disk_chosen),
            second_sentence_red(pool, &by_every_slot),
        ];
        self.states += 1;
        if dangerous {
            self.dangerous += 1;
            self.first_green_on_dangerous += u64::from(!first_red);
            for (reading_index, is_red) in second_red.iter().enumerate() {
                self.second_green_on_dangerous[reading_index] += u64::from(!*is_red);
            }
        } else if single_witness {
            self.single_witness += 1;
            for (reading_index, is_red) in second_red.iter().enumerate() {
                self.second_red_on_single_witness[reading_index] += u64::from(*is_red);
            }
        } else {
            self.benign += 1;
            self.first_red_on_benign += u64::from(first_red);
            for (reading_index, is_red) in second_red.iter().enumerate() {
                self.second_red_on_benign[reading_index] += u64::from(*is_red);
            }
        }
    }
}

#[derive(Default)]
struct ArmResult {
    histories: u64,
    colliding_histories: u64,
    fewest_faults_collision: Option<(u32, Vec<usize>)>,
    tally: StateTally,
}

/// 深度优先走完每一条历史；`path` 记每次挂载取的是第几个结局，打印时按它重放。
fn explore(pool: &Pool, arm: &Arm, depth: usize, faults: u32, collided: bool, path: &mut Vec<usize>, result: &mut ArmResult) {
    if depth == MOUNTS {
        let leaf_collides = pool.every_carrier() & bit(pool.next_code(arm.reading)) != 0;
        result.histories += 1;
        if collided || leaf_collides {
            result.colliding_histories += 1;
            if result.fewest_faults_collision.as_ref().is_none_or(|(fewest, _)| faults < *fewest) {
                result.fewest_faults_collision = Some((faults, path.clone()));
            }
        }
        return;
    }
    for (index, outcome) in mount_step(pool, arm).into_iter().enumerate() {
        result.tally.observe(&outcome.end, arm.reading);
        path.push(index);
        explore(&outcome.end, arm, depth + 1, faults + outcome.faults, collided || outcome.collides, path, result);
        path.pop();
    }
}

fn render_pool(pool: &Pool) -> String {
    let slot_text = |slot: &Option<Slot>| slot.map_or("invalid".to_string(), |slot| format!("g{}i{}", slot.generation, slot.instance));
    let instances = |mask: u64| (0..64).filter(|instance| mask & bit(*instance) != 0).collect::<Vec<u32>>();
    format!(
        "[d0({} {}) d1({} {}) units_or_records={:?} readable_roots={:?} unreadable_roots={:?}]",
        slot_text(&pool.disks[0][0]),
        slot_text(&pool.disks[0][1]),
        slot_text(&pool.disks[1][0]),
        slot_text(&pool.disks[1][1]),
        instances(pool.carried_by_units_or_records),
        instances(pool.readable_roots),
        instances(pool.unreadable_roots)
    )
}

fn render_event(event: &Event) -> String {
    match event {
        Event::Acquire { code, collides } => format!("acquire(code={code},collides={collides})"),
        Event::Issued { what, write } => format!(
            "{what}(d{}s{}:g{}i{})",
            write.disk, write.slot, write.content.generation, write.content.instance
        ),
        Event::Fault(label) => format!("FAULT:{label}"),
        Event::Reached(label) => format!("reached:{label}"),
        Event::CrashLandedMask(mask) => format!("crash_landed_mask={mask:b}"),
    }
}

/// 按 `path` 重放一条历史：每次挂载一行，末尾是第四次取号。
fn render_history(arm: &Arm, path: &[usize]) -> Vec<String> {
    let mut pool = Pool::after_first_transaction();
    let mut lines = Vec::new();
    for (mount_offset, index) in path.iter().enumerate() {
        let outcome = mount_step(&pool, arm).swap_remove(*index);
        let steps: Vec<String> = outcome.events.iter().map(render_event).collect();
        lines.push(format!(
            "  mount={} faults={} {} => {}",
            mount_offset + 2,
            outcome.faults,
            steps.join(" "),
            render_pool(&outcome.end)
        ));
        pool = outcome.end;
    }
    let next = pool.next_code(arm.reading);
    lines.push(format!(
        "  mount={} acquire(code={next},collides={})",
        path.len() + 2,
        pool.every_carrier() & bit(next) != 0
    ));
    lines
}

fn arm_with(base: Arm, rule: GenerationRule, reading: Reading, barrier: BarrierPlacement) -> Arm {
    Arm {
        rule,
        reading,
        barrier,
        rollback: base.rollback,
        retry: base.retry,
        policy: base.policy,
    }
}

/// P = G2 + 全部自证过的槽 + 甲″；另三条臂各把 P 的一句换成第二轮排除的那一条（G1、择到的那一份、乙′）。
fn arms() -> Vec<(&'static str, Arm)> {
    let mut arms = Vec::new();
    let rollbacks = [
        RollbackCode::NewCodeMinusOne,
        RollbackCode::DiskChosenBefore,
        RollbackCode::OverwrittenSlotBefore,
        RollbackCode::NoRollback,
    ];
    for rollback in rollbacks {
        for retry in [RetryTarget::SameSlot, RetryTarget::RereadSlots] {
            for policy in [BarrierErrorPolicy::FailAcquisition, BarrierErrorPolicy::RetryFlushThenProceed] {
                let candidate = Arm {
                    rule: GenerationRule::PerDiskPlusOne,
                    reading: Reading::EverySelfVerifiedSlot,
                    barrier: BarrierPlacement::JiaDoublePrime,
                    rollback,
                    retry,
                    policy,
                };
                arms.push(("P", candidate));
                let with_g1 = arm_with(candidate, GenerationRule::PoolWidePlusOne, candidate.reading, candidate.barrier);
                arms.push(("P_with_G1", with_g1));
                let with_chosen = arm_with(candidate, candidate.rule, Reading::ChosenSlotPerDisk, candidate.barrier);
                arms.push(("P_with_chosen_reading", with_chosen));
                let with_yi = arm_with(candidate, candidate.rule, candidate.reading, BarrierPlacement::YiPrime);
                arms.push(("P_with_yi_prime", with_yi));
            }
        }
    }
    arms
}

/// T3 与附带：D18（块里携带什么信息） 第 879 行的写行规则与已发布谓词在几条四次挂载的历史上各给什么。
/// 根按 (checkpoint_txg, 实例代号) 取最新可读的（D22（单元原子性怎么合成） 已定项 7）；新实例第一次发布的 txg = 所选根 + 1
/// （这些历史里没有记录可重放）；回退的新根 txg = 根环最大 + 1（D23（journal 的角色与格式） 已定项 14）；
/// 写行给 [所选根的实例, 新实例) 每个实例 (i, 所选根的 txg, 0)；谓词取码 2 / 码 3 那一支 b ≤ T_pub（码 1 多一支 n ≤ W，W = 0 不改结论）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RootModel {
    instance: u32,
    txg: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RowModel {
    instance: u32,
    published_txg: u64,
}

fn select_root(roots: &[RootModel], unreadable: &[RootModel]) -> RootModel {
    roots
        .iter()
        .filter(|root| !unreadable.contains(root))
        .copied()
        .max_by_key(|root| (root.txg, root.instance))
        .expect("至少一条根读得出")
}

fn rows_for(selected: RootModel, new_instance: u32) -> Vec<RowModel> {
    (selected.instance..new_instance)
        .map(|instance| RowModel { instance, published_txg: selected.txg })
        .collect()
}

/// 已发布谓词（D18（块里携带什么信息） 第 879 行，码 2 / 码 3 那一支）；行按实例代号后写覆盖。
fn predicate(rows: &[RowModel], mounted: RootModel, unit_instance: u32, unit_birth_txg: u64) -> &'static str {
    if unit_instance > mounted.instance {
        return "corrupt(i>i_now)";
    }
    if unit_instance == mounted.instance {
        return if unit_birth_txg <= mounted.txg { "published" } else { "unpublished" };
    }
    match rows.iter().rev().find(|row| row.instance == unit_instance) {
        None => "published(no_row)",
        Some(row) if unit_birth_txg <= row.published_txg => "published",
        Some(_) => "unpublished",
    }
}

/// 附带：第二次挂载（普通恢复，或管理员回退到 (1, 1)）第一次发布的根 (2, 4) 落了、暖机第二个根没落就崩；
/// 第三次挂载那条根读得出或读不出（一次暂时读不出，下一次又读得出）；第三次挂载第一次发布的单元落了、根没落就崩；
/// 第四次挂载全部根读得出，写行、第一次发布之后按谓词判第三次挂载留下的单元。
fn transient_root_history(rollback_at_second_mount: bool, second_root_unreadable_at_third_mount: bool) -> String {
    let mut roots = vec![
        RootModel { instance: 0, txg: 0 },
        RootModel { instance: 1, txg: 1 },
        RootModel { instance: 1, txg: 2 },
        RootModel { instance: 1, txg: 3 },
    ];
    let second_selected = select_root(&roots, &[]);
    let ring_highest_txg = roots.iter().map(|root| root.txg).max().unwrap_or(0);
    let second_root = RootModel { instance: 2, txg: ring_highest_txg + 1 };
    let table_of_second_root = if rollback_at_second_mount {
        vec![RowModel { instance: 1, published_txg: 1 }]
    } else {
        rows_for(second_selected, 2)
    };
    roots.push(second_root);
    let unreadable_at_third: Vec<RootModel> =
        if second_root_unreadable_at_third_mount { vec![second_root] } else { Vec::new() };
    let third_selected = select_root(&roots, &unreadable_at_third);
    let readable_highest_instance = roots
        .iter()
        .filter(|root| !unreadable_at_third.contains(root))
        .map(|root| root.instance)
        .max()
        .unwrap_or(0);
    let third_code = 2u32.max(readable_highest_instance) + 1;
    let third_rows = rows_for(third_selected, third_code);
    let third_birth = third_selected.txg + 1;
    let fourth_selected = select_root(&roots, &[]);
    let fourth_code = third_code.max(roots.iter().map(|root| root.instance).max().unwrap_or(0)) + 1;
    let mut fourth_table = if fourth_selected == second_root { table_of_second_root.clone() } else { Vec::new() };
    fourth_table.extend(rows_for(fourth_selected, fourth_code));
    let fourth_mounted = RootModel { instance: fourth_code, txg: fourth_selected.txg + 1 };
    let verdict = predicate(&fourth_table, fourth_mounted, third_code, third_birth);
    format!(
        "T3_HISTORY rollback_at_mount2={rollback_at_second_mount} root_(2,{})_unreadable_at_mount3={second_root_unreadable_at_third_mount} | mount2 selected=({},{}) acquired=2 first_root=({},{}) table_of_that_root={:?} crash_before_second_warm_up_root | mount3 selected=({},{}) acquired={third_code} rows={:?} first_publish_txg={third_birth} units(instance {third_code}, birth {third_birth}) landed crash_before_root | mount4 selected=({},{}) acquired={fourth_code} table_after_first_publish={:?} mounted_root=({},{}) | predicate(mount3 units)={verdict}",
        second_root.txg,
        second_selected.instance,
        second_selected.txg,
        second_root.instance,
        second_root.txg,
        table_of_second_root,
        third_selected.instance,
        third_selected.txg,
        third_rows,
        fourth_selected.instance,
        fourth_selected.txg,
        fourth_table,
        fourth_mounted.instance,
        fourth_mounted.txg
    )
}

/// T3：管理员回退到 (1, 1) 的那次挂载取号 2 落了，回退那次发布的单元（写序实例 2、诞生 4）与记录（实例 2、计数器 4）落了，
/// 回退根 (2, 4) 没落；下一次挂载是普通恢复。对照：同一个盘上没发起回退。
fn rollback_crashed_before_root() -> Vec<String> {
    let roots = [
        RootModel { instance: 0, txg: 0 },
        RootModel { instance: 1, txg: 1 },
        RootModel { instance: 1, txg: 2 },
        RootModel { instance: 1, txg: 3 },
    ];
    let mut lines = Vec::new();
    for attempted in [true, false] {
        let superblock_highest = if attempted { 2u32 } else { 1 };
        let selected = select_root(&roots, &[]);
        let code = superblock_highest.max(roots.iter().map(|root| root.instance).max().unwrap_or(0)) + 1;
        let rows = rows_for(selected, code);
        let mounted = RootModel { instance: code, txg: selected.txg + 1 };
        // 重放：所选根覆盖到计数器 3（实例 1 最后一条）；计数器 4 是回退那次的记录，实例 2 ≠ 1 ⇒ 停（D23 已定项 14 的注 1）。
        let applied_records = 0;
        let orphan_verdict = if attempted { predicate(&rows, mounted, 2, 4) } else { "absent" };
        let left = if attempted {
            "[superblock(instance 2)@disk0 superblock(instance 2)@disk1 units(instance 2,birth 4) journal_record(instance 2,counter 4)]"
        } else {
            "[]"
        };
        lines.push(format!(
            "T3_ROLLBACK_CRASHED_BEFORE_ROOT rollback_attempted={attempted} next_mount selected=({},{}) applied_records={applied_records} acquired={code} rows={rows:?} rollback_orphans(instance 2,birth 4)={orphan_verdict} bytes_left_by_the_attempt={left}",
            selected.instance, selected.txg
        ));
    }
    lines
}

fn main() {
    println!(
        "CONFIG mounts_after_first_transaction={MOUNTS} acquisitions_checked={} start={}",
        MOUNTS + 1,
        render_pool(&Pool::after_first_transaction())
    );
    let mut examples = Vec::new();
    for (name, arm) in arms() {
        let mut result = ArmResult::default();
        explore(&Pool::after_first_transaction(), &arm, 0, 0, false, &mut Vec::new(), &mut result);
        let tally = &result.tally;
        let fewest = result
            .fewest_faults_collision
            .as_ref()
            .map_or("none".to_string(), |(faults, _)| faults.to_string());
        println!(
            "ARM name={name} rule={:?} reading={:?} barrier={:?} rollback={:?} retry={:?} barrier_error={:?} histories={} colliding_histories={} fewest_faults_to_collide={fewest} states={} dangerous={} single_witness={} benign={} first_sentence(red_on_benign/green_on_dangerous)={}/{} second_by_disk_highest(red_on_benign/red_on_single_witness/green_on_dangerous)={}/{}/{} second_by_disk_chosen={}/{}/{} second_by_every_slot={}/{}/{}",
            arm.rule, arm.reading, arm.barrier, arm.rollback, arm.retry, arm.policy,
            result.histories, result.colliding_histories, tally.states, tally.dangerous, tally.single_witness, tally.benign,
            tally.first_red_on_benign, tally.first_green_on_dangerous,
            tally.second_red_on_benign[0], tally.second_red_on_single_witness[0], tally.second_green_on_dangerous[0],
            tally.second_red_on_benign[1], tally.second_red_on_single_witness[1], tally.second_green_on_dangerous[1],
            tally.second_red_on_benign[2], tally.second_red_on_single_witness[2], tally.second_green_on_dangerous[2]
        );
        if name == "P" && arm.retry == RetryTarget::SameSlot {
            if let Some((faults, path)) = &result.fewest_faults_collision {
                examples.push(format!(
                    "EXAMPLE arm=P rollback={:?} retry={:?} barrier_error={:?} fewest_faults={faults}",
                    arm.rollback, arm.retry, arm.policy
                ));
                examples.extend(render_history(&arm, path));
            }
        }
    }
    for line in examples {
        println!("{line}");
    }
    for rollback_at_second_mount in [false, true] {
        for unreadable in [false, true] {
            println!("{}", transient_root_history(rollback_at_second_mount, unreadable));
        }
    }
    for line in rollback_crashed_before_root() {
        println!("{line}");
    }
}
