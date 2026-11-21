use super::*;
use crate::helpers_shared_by_the_tests::specification_of_the_shape;

/// 实例 1 结束时恢复读回的文件内容就是最后一次覆盖写的整份内容：补臂是 s 个数据单元，第一段的读法是一个数据单元（登记修订 12.4）。
#[test]
fn the_file_after_the_replacing_writes_has_all_its_units_in_the_units_kept_arm_and_one_unit_in_the_registered_reading(
) {
    for (shape, units) in [
        (OverwriteShape::OneUnit, 1usize),
        (OverwriteShape::KeepingTheFirstFileUnits, 64usize),
    ] {
        let specification = specification_of_the_shape(shape);
        let first = run_the_first_instance(&specification).expect("实例 1 跑得完");
        let mut pool = empty_pool(specification.geometry.device_bytes());
        pool.apply(&first.operations);
        let RecoveryOutcome::FileRead { content, .. } =
            recover(&pool, JournalPolicy::Consult).outcome
        else {
            panic!("{shape:?}：实例 1 结束的镜像上恢复应当读回文件");
        };
        let expected_length = units * DATA_UNIT_PAYLOAD_BYTES_LOCAL;
        assert_eq!(content.len(), expected_length, "{shape:?}：文件的字节数");
        assert_eq!(
            content,
            deterministic_content(101, expected_length),
            "{shape:?}：内容是最后一次覆盖写（标签 100 + 1）的整份内容"
        );
    }
}

/// 补臂的被藏根引用的单元比第一段的读法多至少 63 个（64 个数据单元对 1 个）；绝对值另钉：至少 64 个。
#[test]
fn the_hidden_root_of_the_units_kept_arm_references_at_least_sixty_three_more_units_than_the_registered_reading(
) {
    let referenced_units = |shape: OverwriteShape| -> usize {
        let first =
            run_the_first_instance(&specification_of_the_shape(shape)).expect("实例 1 跑得完");
        first
            .roots
            .get(&first.last_root)
            .expect("被藏根在全知日志里")
            .units
            .len()
    };
    let one_unit = referenced_units(OverwriteShape::OneUnit);
    let kept = referenced_units(OverwriteShape::KeepingTheFirstFileUnits);
    assert!(kept >= 64, "补臂的被藏根至少引用 64 个落点，实际 {kept}");
    assert!(
        kept >= one_unit + 63,
        "补臂 {kept} 个落点，第一段的读法 {one_unit} 个"
    );
}
