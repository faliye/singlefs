
/// 攻方腿副本专用的探针（抬 F 先算代码轮第三轮，只拼进冻结副本的拷贝，不进仓）：先算每读一块被拒绝点名的盘，按线程记下那块盘的读数、
/// 上界、照代码判没判出放不下；判的结局按开关照代码交、一律交「放得下」（关掉先算：看推了会怎样）、或一律交「放不下」（探针自证用）。
/// 先算每进一次（每一次被拒）记下那一次拒绝的成员名。判法本身不动。
pub mod floor_raise_precheck_r3_probe {
    use super::{
        device_short_even_with_everything_a_floor_raise_could_release,
        most_a_floor_raise_could_make_available, DeviceShortEvenWithEverythingAFloorRaiseCouldRelease,
        SlotsAFloorRaiseCouldRelease,
    };
    use crate::admission::{DeviceShortOfDemand, JudgementThatFoundTheDeviceShort};
    use crate::transaction::PublishError;
    use std::cell::{Cell, RefCell};

    /// 先算判出的结局怎么交。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Precheck {
        /// 照冻结副本的代码。
        AsCoded,
        /// 一律判放得下：先算关掉，被拒就推（看推了会怎样）。
        NeverFires,
        /// 一律判放不下：头一次被拒就不推（探针自证用：误拒那一数要能红）。
        AlwaysFires,
    }

    thread_local! {
        static PRECHECK: Cell<Precheck> = const { Cell::new(Precheck::AsCoded) };
        static SEEN: RefCell<Vec<PrecheckSeen>> = const { RefCell::new(Vec::new()) };
        static REFUSALS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    }

    /// 先算读的一块盘：这是这一次调用里第几次被拒（从 1 数）、哪一判、拒绝里的可用与需求（字节）、三样槽数、照代码的上界
    /// （提交内生块那一判没有上界）、照代码判没判出放不下。
    #[derive(Clone, Debug)]
    pub struct PrecheckSeen {
        pub refusal_index: usize,
        pub device: u32,
        pub judgement: &'static str,
        pub available_bytes: i128,
        pub demand_bytes: u64,
        pub deferred: u64,
        pub isolated: u64,
        pub free_but_blocked: u64,
        pub bound: Option<i128>,
        pub fired_as_coded: bool,
    }

    pub fn set_precheck(mode: Precheck) {
        PRECHECK.with(|cell| cell.set(mode));
    }

    pub fn take_seen() -> Vec<PrecheckSeen> {
        SEEN.with(|seen| std::mem::take(&mut *seen.borrow_mut()))
    }

    /// 这一次调用里每一次被拒的成员名，按先后。
    pub fn take_refusals() -> Vec<String> {
        REFUSALS.with(|refusals| std::mem::take(&mut *refusals.borrow_mut()))
    }

    fn leading_name(text: &str) -> String {
        text.split(|character: char| {
            character == '(' || character == '{' || character.is_whitespace()
        })
        .next()
        .unwrap_or("")
        .to_string()
    }

    /// 先算每进一次记一次（空间拒绝之外的那几种也记）。
    pub(super) fn note_refusal(refusal: &PublishError) {
        let name = leading_name(&format!("{refusal:?}"));
        REFUSALS.with(|refusals| refusals.borrow_mut().push(name));
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

    /// 代替 `device_short_even_with_everything_a_floor_raise_could_release` 的那一处调用：照代码判一次、记下，再按开关交结局。
    pub(super) fn judged(
        short: &DeviceShortOfDemand,
        slots: SlotsAFloorRaiseCouldRelease,
    ) -> Option<DeviceShortEvenWithEverythingAFloorRaiseCouldRelease> {
        let as_coded = device_short_even_with_everything_a_floor_raise_could_release(short, slots);
        let bound = most_a_floor_raise_could_make_available(short, slots);
        let seen = PrecheckSeen {
            refusal_index: REFUSALS.with(|refusals| refusals.borrow().len()),
            device: short.device.0,
            judgement: judgement_name(short.judged_short_by),
            available_bytes: short.available.0,
            demand_bytes: short.demand.0,
            deferred: slots.deferred,
            isolated: slots.isolated,
            free_but_blocked: slots.free_but_blocked,
            bound: bound.map(|upper| upper.0),
            fired_as_coded: as_coded.is_some(),
        };
        SEEN.with(|all| all.borrow_mut().push(seen));
        match PRECHECK.with(Cell::get) {
            Precheck::AsCoded => as_coded,
            Precheck::NeverFires => None,
            Precheck::AlwaysFires => Some(as_coded.unwrap_or(
                DeviceShortEvenWithEverythingAFloorRaiseCouldRelease {
                    device: short.device,
                    judged_short_by: short.judged_short_by,
                    most_a_floor_raise_could_make_available: bound.unwrap_or(short.available),
                    demand: short.demand,
                },
            )),
        }
    }
}
