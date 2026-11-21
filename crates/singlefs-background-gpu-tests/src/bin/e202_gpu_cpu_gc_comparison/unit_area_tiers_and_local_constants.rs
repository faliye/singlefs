//! 盘容量的三档与装置的本地常量：几何算式、发布预算，开跑与核心层公开的同一个量回比（停机条款第 1 条）。

use super::*;

/// 盘容量的一档：单元区多大（登记第一节，定占用读法，活数据占单元区 50%；三档的取值照登记第十二节修订第 4 条，都是 mkfs 认的）。
/// 封闭集合，按单元区从小到大。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum UnitAreaTier {
    TwoThousandTwoHundredEightyEightMebibytes,
    FourGibibytes,
    EightGibibytes,
}

impl UnitAreaTier {
    pub(super) const EVERY_TIER: [UnitAreaTier; 3] = [
        UnitAreaTier::TwoThousandTwoHundredEightyEightMebibytes,
        UnitAreaTier::FourGibibytes,
        UnitAreaTier::EightGibibytes,
    ];

    /// 命令行与结果行里的写法（登记第十四节起法里的 `--unit-area`）。
    pub(super) const fn name(self) -> &'static str {
        match self {
            UnitAreaTier::TwoThousandTwoHundredEightyEightMebibytes => "2288MiB",
            UnitAreaTier::FourGibibytes => "4GiB",
            UnitAreaTier::EightGibibytes => "8GiB",
        }
    }

    pub(super) fn of_the_name(name: &str) -> Option<UnitAreaTier> {
        UnitAreaTier::EVERY_TIER
            .into_iter()
            .find(|tier| tier.name() == name)
    }

    const fn unit_area_bytes(self) -> u64 {
        match self {
            UnitAreaTier::TwoThousandTwoHundredEightyEightMebibytes => 2288 << 20,
            UnitAreaTier::FourGibibytes => 4 << 30,
            UnitAreaTier::EightGibibytes => 8 << 30,
        }
    }

    pub(super) const fn device_bytes(self) -> u64 {
        LOCAL_BYTES_BEFORE_THE_UNIT_AREA + self.unit_area_bytes()
    }

    /// 活的码 1 数据单元 P：单元区能装的数据单元的一半（定占用读法 50%）。
    pub(super) const fn live_data_units(self) -> u64 {
        self.unit_area_bytes() / LOCAL_DATA_UNIT_BYTES / 2
    }

    /// 每块盘单元区的整段数（登记第七节乙类第 1 行）。
    pub(super) const fn unit_area_segments(self) -> u64 {
        local_unit_area_slots_of_device(self.device_bytes()) / LOCAL_CLUSTER_SEGMENT_SLOTS
    }

    /// 造碎片之后的碎片段数 P ÷ 32（登记第七节乙类第 3 行）：一段 64 槽装 32 个数据单元，造数据挂载写的 P 个单元里一半留下。
    pub(super) const fn fragment_segments(self) -> u64 {
        self.live_data_units() / data_units_of_a_cluster_segment()
    }
}

/// 一个聚簇段装几个码 1 数据单元（64 槽 × 16 KiB ÷ 32 KiB = 32）。
pub(super) const fn data_units_of_a_cluster_segment() -> u64 {
    LOCAL_CLUSTER_SEGMENT_SLOTS * LOCAL_SLOT_BYTES / LOCAL_DATA_UNIT_BYTES
}

pub(super) const fn local_unit_area_slots_of_device(device_bytes: u64) -> u64 {
    device_bytes / LOCAL_SLOT_BYTES - LOCAL_UNIT_AREA_START_SLOT
}

/// mkfs 认的每块盘至少是 journal 环的这么多倍：环超过设备容量的四分之一就拒（`crates/singlefs-core/src/make_filesystem.rs` 的
/// `JournalRingTooLargeForDevice`，D23（journal 的角色与格式） 已定项 19）。同包测试 `the_make_filesystem_refuses_a_device_one_sector_short_of_four_rings`
/// 拿真 mkfs 回比这个数。
const LOCAL_DEVICE_BYTES_PER_JOURNAL_RING_BYTES_AT_LEAST: u64 = 4;

/// 默认环长下 mkfs 认的最小一块盘（3072 MiB）。
pub(super) const fn smallest_device_bytes_the_make_filesystem_accepts() -> u64 {
    LOCAL_DEVICE_BYTES_PER_JOURNAL_RING_BYTES_AT_LEAST * LOCAL_JOURNAL_RING_DEFAULT_BYTES
}

/// 这一档登记的每块盘字节 mkfs 认不认（登记第一节「每块盘字节 = 784 MiB + 单元区」在默认环长下，单元区不到 2288 MiB 的档 mkfs 都拒）。
/// 不认的那一档不造池、照第十一节作废第 6 条交主 agent（登记的算式与 mkfs 对不上）。
pub(super) const fn the_make_filesystem_accepts_the_device_bytes(device_bytes: u64) -> bool {
    device_bytes >= smallest_device_bytes_the_make_filesystem_accepts()
}

/// 本地式子算的整理发布预算 `max(1, 3S − 4 − p)`。
fn local_publish_budget(publishes_already_passed: u64) -> u64 {
    (3 * LOCAL_ROOT_RING_SLOTS_PER_REGION)
        .saturating_sub(4)
        .saturating_sub(publishes_already_passed)
        .max(1)
}

/// 一条意图要发几次：签发一次，加搬完它那 16 个单元的批数（登记第四节「前提事件」：批大小 4 时 4 批加签发一次 = 5 次）。
pub(super) fn publishes_of_one_intent(batch_units: usize) -> usize {
    1 + UNITS_MOVED_BY_ONE_INTENT.div_ceil(batch_units)
}

/// 本地常量与核心层（或产品 GPU 包测试共用模块）公开的同一个量对不上的一项。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ConstantMismatch {
    pub(super) name: String,
    pub(super) local: u64,
    pub(super) of_the_crates: u64,
}

/// 第五节常量表逐项回比（停机条款第 1 条）。`singlefs-format` 不是本包的直接依赖，格式常量经核心层公开的等价量取：
/// 槽字节取槽 1 的设备偏移，数据单元字节取 `LINEAGE_CONTAINER_BLOCKS_OF_ONE_COPY_ON_WRITE`（= 数据单元字节 ÷ 槽字节）乘槽字节，
/// 聚簇段槽数取核心层每段活槽数那一步给 640 槽单元区分出的段数，环长取单元区起点与环起点两个偏移之差。
pub(super) fn local_constants_disagreeing_with_the_core() -> Vec<ConstantMismatch> {
    let mut mismatches = Vec::new();
    let mut compare = |name: String, local: u64, of_the_crates: u64| {
        if local != of_the_crates {
            mismatches.push(ConstantMismatch {
                name,
                local,
                of_the_crates,
            });
        }
    };
    let slot_bytes_of_the_core = SlotNumber(1).to_device_offset().0;
    compare(
        "slot_bytes".to_string(),
        LOCAL_SLOT_BYTES,
        slot_bytes_of_the_core,
    );
    compare(
        "data_unit_bytes".to_string(),
        LOCAL_DATA_UNIT_BYTES,
        LINEAGE_CONTAINER_BLOCKS_OF_ONE_COPY_ON_WRITE * slot_bytes_of_the_core,
    );
    let probe_unit_area_slots = 10 * LOCAL_CLUSTER_SEGMENT_SLOTS;
    let probe_segments = live_slots_per_cluster_segment_by_the_records(
        &[],
        UnitAreaOfOneDevice {
            device: DeviceIdentity(0),
            start: UnitAreaStart::of_the_default_journal_ring(),
            slots: probe_unit_area_slots,
        },
    )
    .len();
    compare(
        "cluster_segment_slots".to_string(),
        LOCAL_CLUSTER_SEGMENT_SLOTS,
        probe_unit_area_slots / u64::try_from(probe_segments.max(1)).expect("段数装得进 u64"),
    );
    compare(
        "root_ring_slots_per_region".to_string(),
        LOCAL_ROOT_RING_SLOTS_PER_REGION,
        RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM.count(),
    );
    compare(
        "journal_ring_start_slot".to_string(),
        LOCAL_JOURNAL_RING_START_SLOT,
        JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION.0,
    );
    let unit_area_start_of_the_core = UnitAreaStart::of_the_default_journal_ring().slot();
    compare(
        "journal_ring_default_bytes".to_string(),
        LOCAL_JOURNAL_RING_DEFAULT_BYTES,
        unit_area_start_of_the_core.to_device_offset().0
            - JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION
                .to_device_offset()
                .0,
    );
    compare(
        "unit_area_start_slot".to_string(),
        LOCAL_UNIT_AREA_START_SLOT,
        unit_area_start_of_the_core.0,
    );
    for (publishes_already_passed, registered_budget) in REGISTERED_PUBLISH_BUDGETS {
        let name = format!("publish_budget_after_{publishes_already_passed}_publishes");
        compare(
            name.clone(),
            local_publish_budget(publishes_already_passed),
            publish_budget_of_a_compaction_run(
                RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM.count(),
                publishes_already_passed,
            ),
        );
        compare(
            format!("{name}_as_registered"),
            registered_budget,
            local_publish_budget(publishes_already_passed),
        );
    }
    compare(
        "publish_budget_given_by_a_user_change".to_string(),
        LOCAL_PUBLISH_BUDGET_GIVEN_BY_A_USER_CHANGE,
        local_publish_budget(0),
    );
    for (device_mebibytes, registered_slots) in REGISTERED_UNIT_AREA_SLOTS_BY_DEVICE_MEBIBYTES {
        let device_bytes = device_mebibytes << 20;
        let name = format!("unit_area_slots_of_a_{device_mebibytes}_mebibyte_device");
        compare(
            name.clone(),
            local_unit_area_slots_of_device(device_bytes),
            unit_area_slots_of_device(device_bytes),
        );
        compare(
            format!("{name}_as_registered"),
            registered_slots,
            local_unit_area_slots_of_device(device_bytes),
        );
    }
    compare(
        "gpu_wait_limit_by_default_milliseconds".to_string(),
        u64::try_from(LOCAL_GPU_WAIT_LIMIT_BY_DEFAULT.as_millis()).expect("10 秒"),
        u64::try_from(BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT.as_millis()).expect("10 秒"),
    );
    compare(
        "memory_limit_of_each_participating_gpu".to_string(),
        LOCAL_MEMORY_LIMIT_OF_EACH_PARTICIPATING_GPU,
        common_discrete_gpus_of_this_host::MEMORY_LIMIT_OF_EACH_PARTICIPATING_GPU.0,
    );
    mismatches
}

#[cfg(test)]
mod tests;
