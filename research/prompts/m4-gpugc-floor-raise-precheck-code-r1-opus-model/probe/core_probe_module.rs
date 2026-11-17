
/// 攻方腿副本专用的探针（抬 F 先算代码轮，只拼进冻结副本的拷贝，不进仓）：按线程记会话推抬 F 之前那一判判了什么、
/// 每一串推成前后的读数。旁路开关打开时先算判出放不下也照推（等于先算之前的推法）；上界不算 defer 的开关只给探针自证用。
pub mod floor_raise_precheck_probe {
    use super::*;
    use crate::admission::{
        admission_reading_before_a_publish, checkpoint_cost_of_the_version_to_build_on,
        JudgementThatFoundTheDeviceShort,
    };
    use crate::allocator::PoolAllocator;
    use crate::transaction::{PublishError, TransactionOutput};
    use std::cell::{Cell, RefCell};

    thread_local! {
        static BYPASS: Cell<bool> = const { Cell::new(false) };
        static BOUND_WITHOUT_DEFERRED: Cell<bool> = const { Cell::new(false) };
        static PRECHECKS: RefCell<Vec<PrecheckObserved>> = const { RefCell::new(Vec::new()) };
        static RAISES: RefCell<Vec<RaiseObserved>> = const { RefCell::new(Vec::new()) };
    }

    /// 先算那一判里一块被拒绝点名的盘：哪一判、拒绝里的可用与需求、先算算出的上界（提交内生块那一判是 `None`）。
    #[derive(Clone, Debug)]
    pub struct ShortDeviceObserved {
        pub device: u32,
        pub judgement: &'static str,
        pub available: i128,
        pub demand: u64,
        pub bound: Option<i128>,
    }

    /// 一次先算：被拒的是哪一种、判没判出放不下、拒绝点名的每一块盘。
    #[derive(Clone, Debug)]
    pub struct PrecheckObserved {
        pub refusal_kind: &'static str,
        pub fired: bool,
        pub short_devices: Vec<ShortDeviceObserved>,
    }

    /// 一块盘这一刻的读数：式子的可用（分配器上没装根环时没有）、这一版的 ckpt_cost、三样放得开的槽、段外没挡的槽对与没挡的槽。
    #[derive(Clone, Copy, Debug)]
    pub struct DeviceReading {
        pub device: u32,
        pub formula_available: Option<i128>,
        pub checkpoint_cost: u64,
        pub deferred: u64,
        pub isolated: u64,
        pub free_but_blocked: u64,
        pub pairs_outside: u64,
        pub unblocked: u64,
    }

    /// 推成的一串：推之前与推之后每块盘的读数、这一串几次空发布。
    #[derive(Clone, Debug)]
    pub struct RaiseObserved {
        pub before: Vec<DeviceReading>,
        pub after: Vec<DeviceReading>,
        pub publishes: usize,
    }

    pub fn set_bypass(on: bool) {
        BYPASS.with(|cell| cell.set(on));
    }
    pub fn bypass() -> bool {
        BYPASS.with(Cell::get)
    }
    pub fn set_bound_without_deferred(on: bool) {
        BOUND_WITHOUT_DEFERRED.with(|cell| cell.set(on));
    }
    pub fn bound_without_deferred() -> bool {
        BOUND_WITHOUT_DEFERRED.with(Cell::get)
    }
    pub fn take_prechecks() -> Vec<PrecheckObserved> {
        PRECHECKS.with(RefCell::take)
    }
    pub fn take_raises() -> Vec<RaiseObserved> {
        RAISES.with(RefCell::take)
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

    pub(super) fn note_precheck(refusal: &PublishError, allocator: &PoolAllocator, fired: bool) {
        let (refusal_kind, short_devices) = match refusal {
            PublishError::SpaceAdmissionRefused(refused) => (
                "space-admission",
                refused
                    .short_devices
                    .iter()
                    .map(|short| {
                        let device_map = allocator
                            .devices
                            .iter()
                            .find(|device_map| device_map.device == short.device)
                            .expect("探针：拒绝里的盘在分配器里");
                        ShortDeviceObserved {
                            device: short.device.0,
                            judgement: judgement_name(short.judged_short_by),
                            available: short.available.0,
                            demand: short.demand.0,
                            bound: most_a_floor_raise_could_make_available(
                                short,
                                SlotsAFloorRaiseCouldRelease::of_the_device(device_map),
                            )
                            .map(|bound| bound.0),
                        }
                    })
                    .collect(),
            ),
            PublishError::PlacementRefused { .. } => ("placement", Vec::new()),
            _ => ("other", Vec::new()),
        };
        PRECHECKS.with(|cell| {
            cell.borrow_mut().push(PrecheckObserved {
                refusal_kind,
                fired,
                short_devices,
            });
        });
    }

    pub(super) fn observe(allocator: &PoolAllocator, current: &TransactionOutput) -> Vec<DeviceReading> {
        let formula = allocator.root_ring_occupancy().map(|_| {
            admission_reading_before_a_publish(allocator, Some(current)).available_on_each_device()
        });
        let checkpoint_cost = checkpoint_cost_of_the_version_to_build_on(Some(current), allocator).0;
        allocator
            .devices
            .iter()
            .map(|device_map| {
                let slots = SlotsAFloorRaiseCouldRelease::of_the_device(device_map);
                DeviceReading {
                    device: device_map.device.0,
                    formula_available: formula.as_ref().and_then(|readings| {
                        readings
                            .iter()
                            .find(|(device, _)| *device == device_map.device)
                            .map(|(_, available)| available.0)
                    }),
                    checkpoint_cost,
                    deferred: slots.deferred,
                    isolated: slots.isolated,
                    free_but_blocked: slots.free_but_blocked,
                    pairs_outside: device_map
                        .unblocked_slot_pairs_outside_the_cluster_segments(allocator.cluster_segments()),
                    unblocked: device_map.unblocked_slots(),
                }
            })
            .collect()
    }

    pub(super) fn note_raise(
        before: Vec<DeviceReading>,
        allocator: &PoolAllocator,
        current: &TransactionOutput,
        publishes: usize,
    ) {
        let after = observe(allocator, current);
        RAISES.with(|cell| {
            cell.borrow_mut().push(RaiseObserved {
                before,
                after,
                publishes,
            });
        });
    }
}
