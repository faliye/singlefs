
/// 攻方腿副本专用的探针（抬 F 先算代码轮第二轮，只拼进冻结副本的拷贝，不进仓）：先算每读一块被拒绝点名的盘，按线程记下那块盘的读数、
/// 上界算不算聚簇段里那一截各是多少、判没判出放不下。聚簇段里那一截按开关照原样读、读成 0（第一轮修补之前的上界）、或读成一个大数（只给探针自证用）。
pub mod floor_raise_precheck_r2_probe {
    use super::{
        device_short_even_with_everything_a_floor_raise_could_release,
        most_a_floor_raise_could_make_available, DeviceShortEvenWithEverythingAFloorRaiseCouldRelease,
        SlotsAFloorRaiseCouldRelease,
    };
    use crate::admission::{DeviceShortOfDemand, JudgementThatFoundTheDeviceShort};
    use crate::transaction::PublishError;
    use std::cell::{Cell, RefCell};

    /// 先算判的时候聚簇段里那一截怎么读。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum InsideTerm {
        /// 照冻结副本的代码。
        AsCoded,
        /// 读成 0：第一轮修补之前的上界。
        Zero,
        /// 读成 2^20 个槽对：槽对那一判永远判不出放不下（探针自证用）。
        Huge,
    }

    thread_local! {
        static INSIDE_TERM: Cell<InsideTerm> = const { Cell::new(InsideTerm::AsCoded) };
        static SEEN: RefCell<Vec<PrecheckSeen>> = const { RefCell::new(Vec::new()) };
        static REFUSALS: Cell<u64> = const { Cell::new(0) };
    }

    /// 先算读的一块盘：这是这个线程第几次进先算（`refusal_index`）、哪一判、拒绝里的可用与需求（字节）、四样槽数、
    /// 上界算不算聚簇段里那一截各是多少（提交内生块那一判没有上界）、按开关判没判出放不下。
    #[derive(Clone, Debug)]
    pub struct PrecheckSeen {
        pub refusal_index: u64,
        pub device: u32,
        pub judgement: &'static str,
        pub available_bytes: i128,
        pub demand_bytes: u64,
        pub deferred: u64,
        pub isolated: u64,
        pub free_but_blocked: u64,
        pub inside_pairs: u64,
        pub bound_with_inside: Option<i128>,
        pub bound_without_inside: Option<i128>,
        pub fired: bool,
    }

    pub fn set_inside_term(term: InsideTerm) {
        INSIDE_TERM.with(|cell| cell.set(term));
    }

    pub fn take_seen() -> Vec<PrecheckSeen> {
        SEEN.with(|seen| std::mem::take(&mut *seen.borrow_mut()))
    }

    /// 先算每进一次（每一次被拒）记一次，空间拒绝之外的那几种也算。
    pub(super) fn note_refusal(_refusal: &PublishError) {
        REFUSALS.with(|count| count.set(count.get() + 1));
    }

    fn judgement_name(judgement: JudgementThatFoundTheDeviceShort) -> &'static str {
        match judgement {
            JudgementThatFoundTheDeviceShort::FormulaAvailableAgainstDemand => "formula",
            JudgementThatFoundTheDeviceShort::SlotPairsOutsideTheClusterSegmentsForDataUnits => {
                "slot-pairs"
            }
            JudgementThatFoundTheDeviceShort::SlotsLeftForCommitGeneratedUnits => "slots-left",
        }
    }

    /// 代替 `device_short_even_with_everything_a_floor_raise_could_release` 的那一处调用：按开关改聚簇段里那一截再判，判法本身不动。
    pub(super) fn judged(
        short: &DeviceShortOfDemand,
        slots: SlotsAFloorRaiseCouldRelease,
    ) -> Option<DeviceShortEvenWithEverythingAFloorRaiseCouldRelease> {
        let inside_as_coded = slots.unblocked_slot_pairs_inside_the_cluster_segments;
        let without_inside = SlotsAFloorRaiseCouldRelease {
            unblocked_slot_pairs_inside_the_cluster_segments: 0,
            ..slots
        };
        let judged_with = match INSIDE_TERM.with(Cell::get) {
            InsideTerm::AsCoded => slots,
            InsideTerm::Zero => without_inside,
            InsideTerm::Huge => SlotsAFloorRaiseCouldRelease {
                unblocked_slot_pairs_inside_the_cluster_segments: 1 << 20,
                ..slots
            },
        };
        let verdict = device_short_even_with_everything_a_floor_raise_could_release(short, judged_with);
        let seen = PrecheckSeen {
            refusal_index: REFUSALS.with(Cell::get),
            device: short.device.0,
            judgement: judgement_name(short.judged_short_by),
            available_bytes: short.available.0,
            demand_bytes: short.demand.0,
            deferred: slots.deferred,
            isolated: slots.isolated,
            free_but_blocked: slots.free_but_blocked,
            inside_pairs: inside_as_coded,
            bound_with_inside: most_a_floor_raise_could_make_available(short, slots).map(|bound| bound.0),
            bound_without_inside: most_a_floor_raise_could_make_available(short, without_inside)
                .map(|bound| bound.0),
            fired: verdict.is_some(),
        };
        SEEN.with(|all| all.borrow_mut().push(seen));
        verdict
    }
}
