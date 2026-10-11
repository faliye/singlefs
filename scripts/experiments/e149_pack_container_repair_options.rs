//! E149：打包容器坏了靠什么补——D27（小数据打包容器） 已定项 1 列的三条补法各付什么。
//!
//! 在 E114（小数据打包容器的总账） 的口径下（容器头 107、每槽自描述 47、副本数 2、整理后的 `packbg` 形态），
//! 把「容器强制多副本 / 降低 cap / 放弃打包」各算成空间收益、写字节比、丢失半径与丢失概率幂次。
//! 只报数不判输赢；判据与失败条款在 `research/prompts/e149-preregistration.md`。

use e7_index_bench::Emitter;

/// D4 已定项 5。
const UNIT_BYTES: u64 = 32768;
/// D18 已定项 11：码 3 打包记录容器头。
const PACK_HEADER_BYTES: u64 = 107;
/// E114 口径：每槽的自描述（五元组 33 + 写序 10 + 槽表 4）。
const SLOT_SELF_DESCRIPTION_BYTES: u64 = 47;
/// D2 已定项 9：副本数。
const REPLICA_COUNT: u64 = 2;
/// 多一份副本那一臂的副本数。
const EXTRA_COPY_REPLICA_COUNT: u64 = 3;
const BASIS_POINTS: u64 = 10000;

const OBJECT_SIZES: [u64; 5] = [512, 1024, 4096, 8192, 16384];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Repair {
    None,
    ExtraCopy,
    CapLimited(u64),
    NoPack,
}

const REPAIRS: [Repair; 6] = [Repair::None, Repair::ExtraCopy, Repair::CapLimited(2), Repair::CapLimited(4), Repair::CapLimited(8), Repair::NoPack];

impl Repair {
    fn name(self) -> String {
        match self {
            Repair::None => "none".into(),
            Repair::ExtraCopy => "extra_copy".into(),
            Repair::CapLimited(limit) => format!("cap_{limit}"),
            Repair::NoPack => "no_pack".into(),
        }
    }
    fn replica_count(self) -> u64 {
        match self {
            Repair::None | Repair::CapLimited(_) | Repair::NoPack => REPLICA_COUNT,
            Repair::ExtraCopy => EXTRA_COPY_REPLICA_COUNT,
        }
    }
    /// 一个容器装几个对象（补法之后）。
    fn objects_per_container(self, object_bytes: u64) -> u64 {
        let cap = capacity(object_bytes);
        match self {
            Repair::None | Repair::ExtraCopy => cap,
            Repair::CapLimited(limit) => cap.min(limit),
            Repair::NoPack => 1,
        }
    }
}

/// E114 的 cap：⌊(32768 − 107) / (size + 47)⌋。
fn capacity(object_bytes: u64) -> u64 {
    (UNIT_BYTES - PACK_HEADER_BYTES) / (object_bytes + SLOT_SELF_DESCRIPTION_BYTES)
}

/// 空间收益（万分之）：每份用户数据占的设备字节，打包相对不打包省几倍——每单元装 k 个对象、副本 r 份 ⇒ k × 2 / r。
fn space_benefit_basis_points(objects_per_container: u64, replica_count: u64) -> u64 {
    objects_per_container * REPLICA_COUNT * BASIS_POINTS / replica_count
}

/// 写字节比（万分之）：n → ∞ 时 packbg 每对象写 1 个独立单元 + 1/k 个容器，再乘副本数比；不打包是 1。
fn write_ratio_basis_points(objects_per_container: u64, replica_count: u64) -> u64 {
    if objects_per_container <= 1 {
        return BASIS_POINTS * replica_count / REPLICA_COUNT;
    }
    (BASIS_POINTS + BASIS_POINTS / objects_per_container) * replica_count / REPLICA_COUNT
}

fn emit(emitter: &mut Emitter, line: &str) {
    println!("{}", emitter.emit_raw(line));
}

fn main() {
    let mut emitter = Emitter::new();
    emit(&mut emitter, &format!("name=config unit_bytes={UNIT_BYTES} pack_header={PACK_HEADER_BYTES} slot_self_description={SLOT_SELF_DESCRIPTION_BYTES} replicas={REPLICA_COUNT}"));
    let mut any_gain_at_16k = false;
    for object_bytes in OBJECT_SIZES {
        emit(&mut emitter, &format!("name=capacity object_bytes={object_bytes} cap={}", capacity(object_bytes)));
        for repair in REPAIRS {
            let per_container = repair.objects_per_container(object_bytes);
            let replicas = repair.replica_count();
            let benefit = space_benefit_basis_points(per_container, replicas);
            if object_bytes == 16384 && benefit > BASIS_POINTS {
                any_gain_at_16k = true;
            }
            emit(&mut emitter, &format!(
                "name=repair object_bytes={object_bytes} arm={} objects_per_container={per_container} replicas={replicas} space_benefit_bp={benefit} write_ratio_bp={} loss_radius={per_container} copies_that_must_fail={replicas}",
                repair.name(), write_ratio_basis_points(per_container, replicas)
            ));
        }
    }
    emit(&mut emitter, &format!(
        "name=sensitive cap_8_changes_512={} cap_8_changes_4096={}",
        Repair::CapLimited(8).objects_per_container(512) != Repair::None.objects_per_container(512),
        Repair::CapLimited(8).objects_per_container(4096) != Repair::None.objects_per_container(4096)
    ));
    emit(&mut emitter, &format!("name=verdict caps=58/30/7/3/1 any_gain_at_16k={any_gain_at_16k}"));
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 阳性对照（条款 a）：cap 五档逐字等于 E114 的 58 / 30 / 7 / 3 / 1。
    #[test]
    fn capacities_are_the_e114_values() {
        assert_eq!(OBJECT_SIZES.map(capacity), [58, 30, 7, 3, 1]);
    }

    /// 取样点不敏感的补格（mutation-sampling.md 第三类）：五档对象大小上容器头 107 → 103、槽自描述 47 → 43 都算出同一组 cap；
    /// 6486 字节的对象跨过整数边界——107 / 47 时 ⌊32661 / 6533⌋ = 4，头 103 时 ⌊32665 / 6533⌋ = 5，槽 43 时 ⌊32661 / 6529⌋ = 5。
    #[test]
    fn object_size_6486_is_sensitive_to_header_and_slot_widths() {
        assert_eq!(capacity(6486), 4);
        assert_eq!((UNIT_BYTES - PACK_HEADER_BYTES) / (6486 + SLOT_SELF_DESCRIPTION_BYTES), 4);
    }

    /// 绝对值断言：多一份副本把收益乘 2/3、写字节乘 3/2；16 KiB 档收益 < 1。
    #[test]
    fn extra_copy_scales_benefit_and_writes() {
        assert_eq!(space_benefit_basis_points(58, 2), 580000);
        assert_eq!(space_benefit_basis_points(58, 3), 386666);
        assert_eq!(write_ratio_basis_points(58, 2), 10172);
        assert_eq!(write_ratio_basis_points(58, 3), 15258);
        assert_eq!(space_benefit_basis_points(1, 3), 6666, "16 KiB 档多一份副本比不打包还费");
    }

    /// 绝对值断言：cap 顶 8 在 512 B 档把 58 压到 8，在 4 KiB 档 7 不动；放弃打包恒 1。
    #[test]
    fn cap_limit_bites_only_where_cap_exceeds_it() {
        assert_eq!(Repair::CapLimited(8).objects_per_container(512), 8);
        assert_eq!(Repair::CapLimited(8).objects_per_container(1024), 8);
        assert_eq!(Repair::CapLimited(8).objects_per_container(4096), 7);
        assert_eq!(Repair::CapLimited(2).objects_per_container(8192), 2);
        assert_eq!(Repair::NoPack.objects_per_container(512), 1);
        assert_eq!(write_ratio_basis_points(1, 2), 10000, "不打包写字节比恒 1");
    }

    /// 条款 b：16 KiB 档没有一臂收益 > 1。
    #[test]
    fn no_arm_gains_at_sixteen_kibibytes() {
        for repair in REPAIRS {
            let per_container = repair.objects_per_container(16384);
            assert!(space_benefit_basis_points(per_container, repair.replica_count()) <= BASIS_POINTS, "{:?}", repair);
        }
    }

    /// 阳性对照跑遍每一臂：半径 = 每容器对象数，副本数只在多副本那一臂是 3。
    #[test]
    fn positive_control_runs_on_every_arm() {
        for repair in REPAIRS {
            for object_bytes in OBJECT_SIZES {
                assert!(repair.objects_per_container(object_bytes) >= 1, "{:?}", repair);
            }
            assert_eq!(repair.replica_count() == 3, repair == Repair::ExtraCopy);
        }
    }
}
