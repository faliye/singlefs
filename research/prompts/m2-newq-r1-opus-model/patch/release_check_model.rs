
/// m2-newq-r1 云端攻方腿在副本上加的最小「释放前读盘核校验和 + 隔离」（主工作区没有；另有实现员在写正式版）。
/// 只为在同一个二进制里切换 N1 / N2 / N3 的候选臂，默认关（`enabled = false` 时发布路径与今天逐字节相同）。
pub mod release_check_model {
    use std::cell::RefCell;

    use crate::address::{DeviceIdentity, SlotNumber};
    use crate::allocator::Placement;
    use crate::block_device::BlockDevice;
    use crate::checksum::crc32_castagnoli;
    use crate::pointer::LocationEntry;
    use singlefs_format::SLOT_BYTES;

    /// N1：读盘本身失败（设备报错）时怎么办。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum OnReadFailure {
        /// N1-A：当成对不上处置（隔离、发布照成）。
        AsMismatch,
        /// N1-B：发布失败交回，一个状态都不动。
        FailPublish,
        /// N1-C：走 D23 已定项 14 的失败表（副本里只交回一个点名的错，探针写与只读复核由驱动模拟）。
        FailureTable,
    }

    /// 两份镜像怎么合成一个判定。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum MirrorRule {
        /// 任一份读得出且校验和对得上即过（与 `recovery::read_unit_via_locations` 同一条）。
        AnyMirror,
        /// 每一份都要读得出且对得上。
        EveryMirror,
    }

    /// 核出对不上时隔离哪几块盘上的那个槽。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum IsolationScope {
        EveryDevice,
        MismatchingDeviceOnly,
    }

    /// 核出对不上时「逻辑上照样释放」按哪一种读。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum OnMismatch {
        /// 分配记录照样改写成已释放 + 释放代，另置隔离位（硬规则 1 的字面）。
        ReleaseRecordAndIsolate,
        /// 分配记录不改写（留在已分配），映射条目随新一版自然去掉；不置隔离位。
        KeepRecordAllocated,
    }

    #[derive(Clone, Copy, Debug)]
    pub struct Arms {
        pub enabled: bool,
        pub on_read_failure: OnReadFailure,
        pub mirror_rule: MirrorRule,
        pub scope: IsolationScope,
        pub on_mismatch: OnMismatch,
        /// N3「重挂时现算」：挂载时把「分配记录已释放、却被所选根那一版引用着」的槽隔离。
        pub recompute_at_mount: bool,
    }

    impl Arms {
        pub const OFF: Arms = Arms {
            enabled: false,
            on_read_failure: OnReadFailure::AsMismatch,
            mirror_rule: MirrorRule::AnyMirror,
            scope: IsolationScope::EveryDevice,
            on_mismatch: OnMismatch::ReleaseRecordAndIsolate,
            recompute_at_mount: false,
        };
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum MirrorVerdict {
        Match,
        Mismatch,
        ReadFailed,
    }

    #[derive(Clone, Debug)]
    pub struct CheckEvent {
        pub unit: String,
        pub slot: SlotNumber,
        pub verdicts: Vec<(DeviceIdentity, MirrorVerdict)>,
        pub outcome: &'static str,
    }

    thread_local! {
        pub static ARMS: RefCell<Arms> = const { RefCell::new(Arms::OFF) };
        pub static LOG: RefCell<Vec<CheckEvent>> = const { RefCell::new(Vec::new()) };
        /// 释放核验这一路读了几次盘、读了多少字节（代价那一列）。
        pub static READS: RefCell<(u64, u64)> = const { RefCell::new((0, 0)) };
        /// 这一路置过隔离位的槽数（逐盘累计，与影子账分开记）。
        pub static ISOLATED_BY_CHECK: RefCell<Vec<(DeviceIdentity, u64)>> = const { RefCell::new(Vec::new()) };
        /// 重挂现算那一臂隔离的槽数。
        pub static ISOLATED_AT_MOUNT: RefCell<u64> = const { RefCell::new(0) };
    }

    pub fn arms() -> Arms {
        ARMS.with(|arms| *arms.borrow())
    }
    pub fn set_arms(arms: Arms) {
        ARMS.with(|slot| *slot.borrow_mut() = arms);
    }
    pub fn reset_counters() {
        LOG.with(|log| log.borrow_mut().clear());
        READS.with(|reads| *reads.borrow_mut() = (0, 0));
        ISOLATED_BY_CHECK.with(|isolated| isolated.borrow_mut().clear());
        ISOLATED_AT_MOUNT.with(|isolated| *isolated.borrow_mut() = 0);
    }

    /// 按一个单元的位置条目逐份读、逐份比校验和。`stop_at_first_match` 为真时读到第一份对得上的就停（AnyMirror 的最少读数）。
    pub fn verdicts<Device: BlockDevice>(
        devices: &[(DeviceIdentity, Device)],
        locations: &[LocationEntry; 2],
        span_slots: u64,
        stop_at_first_match: bool,
    ) -> Vec<(DeviceIdentity, MirrorVerdict)> {
        let length = usize::try_from(span_slots * SLOT_BYTES).expect("单元字节数");
        let mut out = Vec::new();
        for location in locations {
            let Some((_, device)) = devices.iter().find(|(id, _)| *id == location.device) else {
                out.push((location.device, MirrorVerdict::ReadFailed));
                continue;
            };
            let mut buffer = vec![0u8; length];
            READS.with(|reads| {
                let mut reads = reads.borrow_mut();
                reads.0 += 1;
                reads.1 += u64::try_from(length).expect("字节数");
            });
            let verdict = match device.read_at(location.slot.to_device_offset(), &mut buffer) {
                Err(_) => MirrorVerdict::ReadFailed,
                Ok(()) if crc32_castagnoli(&buffer) == location.unit_checksum => MirrorVerdict::Match,
                Ok(()) => MirrorVerdict::Mismatch,
            };
            out.push((location.device, verdict));
            if stop_at_first_match && verdict == MirrorVerdict::Match {
                break;
            }
        }
        out
    }

    pub fn note_isolated(device: DeviceIdentity, span: u64) {
        ISOLATED_BY_CHECK.with(|isolated| {
            let mut isolated = isolated.borrow_mut();
            if let Some(entry) = isolated.iter_mut().find(|(id, _)| *id == device) {
                entry.1 += span;
            } else {
                isolated.push((device, span));
            }
        });
    }

    /// 核出来要做的事：隔离哪些（槽、跨度、哪几块盘），哪些不改写分配记录。
    #[derive(Clone, Debug, Default)]
    pub struct CheckPlan {
        pub isolate: Vec<(Placement, Vec<DeviceIdentity>)>,
        pub keep_allocated: Vec<Placement>,
    }
}
