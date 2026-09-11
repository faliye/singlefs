//! E37：序号高位带「日志实例代号」，是不是一条真出路
//!
//! 设计与依据见 `.claude/kb/experiments/37-日志实例代号.md`。
//!
//! **它接的是 D23（journal 的角色与格式）已定项 8 出路表里唯一没测过的那一条。**
//! 另外三条的状态：反向链（已测，两条判据全过，花记录头字节）、
//! 恢复时物理作废残留槽（已测，全过，但要求原地覆写、zoned 做不到）、
//! 槽位与序号解耦（E36 已测，**不成立**）。
//!
//! ## 这条出路长什么样
//!
//! 每次恢复把一个「日志实例代号」加一，并把它放进序号的高位：
//! `jsn = (epoch << EPOCH_SHIFT) | counter`。
//! 上一条时间线的残留记录带的是**旧 epoch**，于是它的 `jsn` 与新时间线期望的下一个数
//! 不在同一个高位区间里 ⇒ 断号即止当场停下。
//!
//! ## 但它有一个隐藏前置，本实验要把它量出来
//!
//! **epoch 必须持久，而且必须防回滚。** 它住哪儿？
//! - 住 journal 记录里：恢复要先读记录才知道 epoch，而读记录正需要 epoch 来判——循环。
//! - 住超级块：那就与 D9（加密）已定项 8 定案里的 **nonce 水位**同类——
//!   D9（加密）逐字要求水位「不许住在裸明文超级块里……必须落在一个被 MAC 覆盖的字段上」，
//!   理由是回滚它等于绕过密钥。**epoch 被回滚的后果与之同型**：
//!   回滚到旧 epoch ⇒ 残留记录重新变成「同一实例」⇒ 出路失效。
//!
//! ⇒ 本实验四条臂：`baseline`（现规范）、`epoch`（只加字段、恢复规则不动）、
//! `epoch_rolled_back`（代号住在可回滚的地方）、
//! `epoch_aware_recovery`（代号有效**且恢复规则跟着改**）。
//!
//! ## 实测抓到的那一处：这条出路的代价比记的重
//!
//! D23（journal 的角色与格式）已定项 8 的出路表把它记成「要，但只占序号里的位」。
//! **只加字段不够**：代号一加，新记录的 `jsn` 就与旧前缀不连续，
//! 而已定判据是 `jsn == expected`、不等即止 ⇒ **新写的记录全丢**，与 `skip_hole` 同病。
//! ⇒ 它还要**改一条已定的前缀规则**（接受「代号 +1 且计数器接着走」）。
//!
//! ## 判据（两条，必须同时满足）
//!
//! 1. `stale_replayed == 0`；2. `lost_new == 0`。
//!
//! ## 阳性对照
//!
//! `baseline` 那条臂**必须**重放残留，且条数服从 E32（上一条时间线的残留）测出的式子
//! `max(0, 残留 − (新写 − 1))`。对不上说明本实验的模型跑偏了。

use e7_index_bench::Emitter;

/// epoch 占序号的高位从第几位起。取 48 ⇒ 低 48 位给计数器，高 16 位给代号。
const EPOCH_SHIFT: u32 = 48;
/// 计数器位宽推出的每实例最大记录数。
const COUNTER_BITS: u32 = EPOCH_SHIFT;
/// 代号位宽。
const EPOCH_BITS: u32 = 64 - EPOCH_SHIFT;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    /// 现规范：只有一个单调计数器，没有实例代号
    Baseline,
    /// 每次恢复把实例代号加一，放进序号高位
    Epoch,
    /// 代号住在可回滚的地方，崩溃后回到旧值
    EpochRolledBack,
    /// 代号有效，**且恢复规则跟着改**：接受「代号 +1 且计数器接着走」这种续接
    EpochAwareRecovery,
}

const ARMS: [Arm; 4] = [
    Arm::Baseline,
    Arm::Epoch,
    Arm::EpochRolledBack,
    Arm::EpochAwareRecovery,
];

impl Arm {
    fn name(self) -> &'static str {
        match self {
            // 没有 `_ =>` —— 新增一条臂不补这里就编译不过
            Arm::Baseline => "baseline",
            Arm::Epoch => "epoch",
            Arm::EpochRolledBack => "epoch_rolled_back",
            Arm::EpochAwareRecovery => "epoch_aware_recovery",
        }
    }
    /// 恢复时接不接受「代号 +1 且计数器接着走」。
    /// ⚠️ **这是对 D23（journal 的角色与格式）已定前缀规则的修改**，不只是加个字段。
    fn recovery_accepts_epoch_bump(self) -> bool {
        matches!(self, Arm::EpochAwareRecovery)
    }
    /// 恢复之后新时间线用哪个实例代号。
    fn epoch_after_recovery(self, old: u64) -> u64 {
        match self {
            // 没有 `_ =>`
            Arm::Baseline => old,          // 没有代号这个概念
            Arm::Epoch => old + 1,              // 递增
            Arm::EpochRolledBack => old,        // 该递增却没能持久，回到旧值
            Arm::EpochAwareRecovery => old + 1, // 递增，且恢复认这个跳变
        }
    }
}

fn compose(epoch: u64, counter: u64) -> u64 {
    (epoch << EPOCH_SHIFT) | counter
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct JournalRecord {
    jsn: u64,
    timeline: u32,
    is_checksum_valid: bool,
}

const TIMELINE_PREVIOUS_LAP: u32 = 0;
const TIMELINE_BEFORE_RECOVERY: u32 = 1;
const TIMELINE_AFTER_RECOVERY: u32 = 2;

struct Ring {
    slot: Vec<Option<JournalRecord>>,
}

impl Ring {
    fn new(slot_count: usize) -> Self {
        Ring { slot: vec![None; slot_count] }
    }
    /// 槽位由**低位计数器**决定，不由整个 `jsn` 决定——
    /// 否则换个 epoch 会把整个环的落点平移，那是另一个设计，不是本条出路。
    fn slot_index_of(&self, jsn: u64) -> usize {
        ((jsn & ((1u64 << COUNTER_BITS) - 1)) as usize) % self.slot.len()
    }
    fn put(&mut self, record: JournalRecord) {
        let slot_index = self.slot_index_of(record.jsn);
        self.slot[slot_index] = Some(record);
    }
    fn get(&self, jsn: u64) -> Option<JournalRecord> {
        self.slot[self.slot_index_of(jsn)]
    }
}

/// 恢复：全环扫描 + 逐条验证 + 断号即止。
///
/// `accept_epoch_bump = false` 时是 D23（journal 的角色与格式）已定的那套，一个字没改。
/// 为 true 时额外接受「代号恰好 +1 且计数器接着走」——**那是对已定规则的修改**。
fn recover(ring: &Ring, tail_jsn: u64, accept_epoch_bump: bool) -> Vec<JournalRecord> {
    let mut accepted_prefix = Vec::new();
    let mut expected = tail_jsn;
    while accepted_prefix.len() < ring.slot.len() {
        let record = match ring.get(expected) {
            Some(record) => record,
            None => break,
        };
        if !record.is_checksum_valid {
            break;
        }
        if record.jsn != expected {
            // 已定规则到此为止；带 epoch 感知时再看一眼「代号 +1、计数器不变」
            let bumped = compose(
                (expected >> EPOCH_SHIFT) + 1,
                expected & ((1u64 << COUNTER_BITS) - 1),
            );
            if !(accept_epoch_bump && record.jsn == bumped) {
                break;
            }
            expected = bumped;
        }
        accepted_prefix.push(record);
        expected += 1;
    }
    accepted_prefix
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct RecoveryOutcome {
    first_prefix: u64,
    stale_replayed: u64,
    lost_new: u64,
}

const SLOTS: usize = 64;
const SETTLED: u64 = 20;
const EPOCH0: u64 = 3;

fn run(arm: Arm, stale: u64, new_after: u64) -> RecoveryOutcome {
    let mut ring = Ring::new(SLOTS);

    // 稳态布景：上一圈铺满环
    for previous_lap_counter in 1..=SLOTS as u64 {
        ring.put(JournalRecord { jsn: compose(EPOCH0, previous_lap_counter), timeline: TIMELINE_PREVIOUS_LAP, is_checksum_valid: true });
    }

    let tail_counter = SLOTS as u64 + 1;
    let tail_jsn = compose(EPOCH0, tail_counter);

    // 时间线 A：安定段
    for settled_index in 0..SETTLED {
        ring.put(JournalRecord {
            jsn: compose(EPOCH0, tail_counter + settled_index),
            timeline: TIMELINE_BEFORE_RECOVERY,
            is_checksum_valid: true,
        });
    }
    let hole_counter = tail_counter + SETTLED;
    // 空洞那条没写成；后面几条落盘了
    for stale_index in 0..stale {
        ring.put(JournalRecord {
            jsn: compose(EPOCH0, hole_counter + 1 + stale_index),
            timeline: TIMELINE_BEFORE_RECOVERY,
            is_checksum_valid: true,
        });
    }

    // 第一次恢复
    let first_recovery_prefix = recover(&ring, tail_jsn, arm.recovery_accepts_epoch_bump());
    let mut outcome = RecoveryOutcome { first_prefix: first_recovery_prefix.len() as u64, ..Default::default() };

    // 恢复之后：实例代号怎么走，是本实验的分歧点
    let new_epoch = arm.epoch_after_recovery(EPOCH0);
    // 计数器一律接前缀末 + 1（本实验不测续写策略那一维，E32 已测过）
    let resume_counter = hole_counter;

    for new_record_index in 0..new_after {
        ring.put(JournalRecord {
            jsn: compose(new_epoch, resume_counter + new_record_index),
            timeline: TIMELINE_AFTER_RECOVERY,
            is_checksum_valid: true,
        });
    }

    // 第二次恢复：从同一个 tail 起
    // ⚠️ **tail 也要跟着实例代号走**——否则新时间线的记录连第一条都接不上。
    // 这正是这条出路的隐藏成本：tail 与 epoch 必须一起持久、一起回滚或一起不回滚。
    let tail2 = compose(EPOCH0, tail_counter);
    let second_recovery_prefix = recover(&ring, tail2, arm.recovery_accepts_epoch_bump());
    outcome.stale_replayed = second_recovery_prefix
        .iter()
        .filter(|record| record.timeline == TIMELINE_BEFORE_RECOVERY && (record.jsn & ((1u64 << COUNTER_BITS) - 1)) >= hole_counter)
        .count() as u64;
    let replayed_new = second_recovery_prefix.iter().filter(|record| record.timeline == TIMELINE_AFTER_RECOVERY).count() as u64;
    outcome.lost_new = new_after - replayed_new;
    outcome
}

fn main() {
    let mut emitter = Emitter::new();
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config slots={SLOTS} settled={SETTLED} epoch_bits={EPOCH_BITS} \
             counter_bits={COUNTER_BITS} note=日志实例代号"
        ))
    );
    for arm in ARMS {
        for (stale, new_after) in [(3u64, 1u64), (3, 3), (7, 1)] {
            let outcome = run(arm, stale, new_after);
            let pass = outcome.stale_replayed == 0 && outcome.lost_new == 0;
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=cell arm={} stale={} new_after={} first_prefix={} \
                     stale_replayed={} lost_new={} pass={}",
                    arm.name(),
                    stale,
                    new_after,
                    outcome.first_prefix,
                    outcome.stale_replayed,
                    outcome.lost_new,
                    pass
                ))
            );
        }
    }
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **阳性对照：`baseline` 必须复现 E32（上一条时间线的残留）测出的那条式子。**
    /// 对不上说明本实验的模型跑偏了。
    #[test]
    fn the_baseline_reproduces_what_e33_measured() {
        for (stale, new_after) in [(3u64, 1u64), (3, 3), (7, 1)] {
            let want = stale.saturating_sub(new_after - 1);
            let outcome = run(Arm::Baseline, stale, new_after);
            assert_eq!(
                outcome.stale_replayed, want,
                "baseline 该复现 E32 的式子：残留 {stale}、新写 {new_after} ⇒ 重放 {want}"
            );
        }
    }

    /// **只加字段、不改恢复规则 ⇒ 残留是挡住了，新写的记录全丢。**
    ///
    /// 机制：代号一加，新记录的 `jsn` 就与旧前缀**不连续**，
    /// 而 D23（journal 的角色与格式）已定的判据是 `jsn == expected`、不等即止。
    /// ⇒ **这条出路的代价不是「只占序号里的位」，它还要改一条已定规则。**
    #[test]
    fn an_epoch_without_an_epoch_aware_recovery_loses_every_new_record() {
        for (stale, new_after) in [(3u64, 1u64), (3, 3), (7, 1)] {
            let outcome = run(Arm::Epoch, stale, new_after);
            assert_eq!(outcome.stale_replayed, 0, "残留确实挡住了（残留 {stale}）");
            assert_eq!(
                outcome.lost_new, new_after,
                "而新写的 {new_after} 条全丢——与 skip_hole 同病"
            );
        }
    }

    /// **恢复规则跟着改之后，两条判据才同时满足。**
    #[test]
    fn only_an_epoch_aware_recovery_satisfies_both_criteria() {
        for (stale, new_after) in [(3u64, 1u64), (3, 3), (7, 1)] {
            let outcome = run(Arm::EpochAwareRecovery, stale, new_after);
            assert_eq!(outcome.stale_replayed, 0, "残留 {stale}、新写 {new_after}");
            assert_eq!(outcome.lost_new, 0, "残留 {stale}、新写 {new_after}");
        }
    }

    /// **代号被回滚一格，出路当场失效，退回 `baseline` 的行为。**
    /// ⇒ 这条出路的正确性**全押在「代号不许回滚」上**，
    /// 与 D9（加密）已定项 8 对 nonce 水位的要求同型。
    #[test]
    fn rolled_back_epoch_degrades_all_the_way_to_the_baseline() {
        for (stale, new_after) in [(3u64, 1u64), (3, 3), (7, 1)] {
            let rolled = run(Arm::EpochRolledBack, stale, new_after);
            let base = run(Arm::Baseline, stale, new_after);
            assert_eq!(
                rolled, base,
                "代号被回滚时该与 baseline 逐格相同（残留 {stale}、新写 {new_after}）"
            );
            assert!(rolled.stale_replayed > 0, "而 baseline 是会重放的");
        }
    }

    /// **判别力：第一次恢复必须真的停在空洞处。**
    #[test]
    fn the_first_recovery_stops_at_the_hole_in_every_arm() {
        for arm in ARMS {
            let outcome = run(arm, 3, 1);
            assert_eq!(outcome.first_prefix, SETTLED, "{} 没停在空洞处", arm.name());
        }
    }

    /// **绝对值：位宽的划分由构造给出，不从被测代码反解。**
    /// 16 位代号 ⇒ 一个卷一生最多 65535 次恢复；48 位计数器 ⇒ 每实例 2.8e14 条记录。
    #[test]
    fn the_bit_budget_is_pinned() {
        assert_eq!(EPOCH_BITS, 16);
        assert_eq!(COUNTER_BITS, 48);
        assert_eq!(compose(1, 0), 1u64 << 48);
        assert_eq!(compose(0, 5), 5);
        // 代号与计数器互不串位
        assert_eq!(compose(7, 12) >> EPOCH_SHIFT, 7);
        assert_eq!(compose(7, 12) & ((1u64 << COUNTER_BITS) - 1), 12);
    }

    /// **槽位必须由低位计数器决定，不能由整个序号决定。**
    /// 否则换一个代号会把整个环的落点平移，那是另一个设计。
    #[test]
    fn the_slot_is_chosen_by_the_counter_not_by_the_whole_sequence_number() {
        let ring = Ring::new(SLOTS);
        for counter in [1u64, 5, 63, 64, 65] {
            assert_eq!(
                ring.slot_index_of(compose(3, counter)),
                ring.slot_index_of(compose(9, counter)),
                "同一个计数器在不同代号下该落在同一个槽"
            );
        }

        // ⚠️ **上面那几格在 2 的幂环上是白给的**：`SLOTS = 64` 整除 `2^48`，
        // 高位取模之后天然消失，掩不掩码结果都一样
        // （变异 M3 实测：把掩码摘掉，只有这几格时一个测试都不红）。
        // ⇒ 用一个**非 2 的幂**的环把「掩码这一步真的在起作用」钉住。
        let odd = Ring::new(60);
        let masked_slot_under_epoch_3 = odd.slot_index_of(compose(3, 7));
        let masked_slot_under_epoch_9 = odd.slot_index_of(compose(9, 7));
        assert_eq!(masked_slot_under_epoch_3, masked_slot_under_epoch_9, "掩码之后，同一计数器在不同代号下仍落同一槽");
        let unmasked_slot_under_epoch_3 = (compose(3, 7) as usize) % 60;
        let unmasked_slot_under_epoch_9 = (compose(9, 7) as usize) % 60;
        assert_ne!(
            unmasked_slot_under_epoch_3, unmasked_slot_under_epoch_9,
            "不掩码的话它们会落到不同的槽——这正是掩码在挡的事"
        );
    }
}
