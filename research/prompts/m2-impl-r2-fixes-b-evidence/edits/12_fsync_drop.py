from edit_lib import edit

FD = "crates/singlefs-harness/tests/second_transaction_supplement_two_fsync_drop_and_devices_without_the_selected_version.rs"
edit(FD, """use singlefs_core::mount::{
    mount_writable, DeviceCount, DeviceWithoutTheSelectedVersion, MountError, RollbackTarget,
    SelectedVersionLackingOnDevice, STRIPE_WIDTH_LOWER_BOUND,
};""", """use singlefs_core::mount::{
    mount_writable, DeviceCount, DeviceWithoutTheSelectedVersion, MountError, NewerPublishWitness,
    RollbackTarget, SelectedVersionAgainstTheWitness, SelectedVersionLackingOnDevice,
    StillUnreadableAfterOneReread, WitnessedCounterComparison, STRIPE_WIDTH_LOWER_BOUND,
};""")
edit(FD, """fn version_key(instance: u32, txg: u64) -> RollbackTarget {""", """/// C554 乙先拒的那一格（`MountError::NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)`）：
/// 读阶段交出的所选那一版是 `selected_version`、它那次发布末条记录读得出、计数器 `selected_version_last_record_counter`，另一块盘的系统配置
/// 见证过计数器 `witnessed_journal_counter`（更大）；重读一遍读数相同。乙在读阶段之后、取号之前逐盘核之前判，所以这几格走不到逐盘核
/// （调查 `research/prompts/m2-investigate-three-reds-report.md` 第二节）。
fn assert_refused_by_the_witness_of_a_newer_publish(
    error: &MountError,
    what: &str,
    selected_version: RollbackTarget,
    witnessed_journal_counter: u64,
    selected_version_last_record_counter: u64,
) {
    let reading = SelectedVersionAgainstTheWitness {
        selected_version,
        witness: NewerPublishWitness {
            witnessed_journal_counter,
            comparison: WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord {
                selected_version_last_record_counter,
            },
        },
    };
    assert!(
        matches!(
            error,
            MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
                if **still_unreadable
                    == StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
                        first_read: reading,
                        reread: reading,
                    }
        ),
        "{what}：该被 C554 乙拒（见证 {witnessed_journal_counter} > 所选那一版末条 {selected_version_last_record_counter}），实际 {error:?}"
    );
}

fn version_key(instance: u32, txg: u64) -> RollbackTarget {""")
edit(FD, """/// 空盘在盘 1（盘 0 带着 txg 5 那一版）或在盘 0（盘 1 带着的最新一版是 txg 4：txg 5 的根在盘 0 上，它那条记录的点名单元
/// 盘 0 那一份验不过、施加不了）：可写挂载在取号之前拒掉，报新成员、点名那块空盘「没有自证过的系统配置」，
/// 带出所选那一版与它那次发布末条记录的 jsn；两块盘逐字节不变。
#[test]
fn blank_device_refuses_the_writable_mount_by_name_before_any_write() {""", """/// 空盘在盘 1（盘 0 带着 txg 5 那一版）：可写挂载在取号之前拒掉，报新成员、点名那块空盘「没有自证过的系统配置」，
/// 带出所选那一版与它那次发布末条记录的 jsn；两块盘逐字节不变。
/// 空盘在盘 0（盘 1 带着的最新一版是 txg 4：txg 5 的根在盘 0 上，它那条记录的点名单元盘 0 那一份验不过、施加不了）：
/// 盘 1 的系统配置见证过 txg 5 那次发布（jsn 5 > 所选那一版末条 4），C554 乙在逐盘核之前拒，逐盘核这一格不可达（C554 乙之后）；
/// 同样拒在取号之前、两块盘逐字节不变。逐盘核点名空盘那一判由「盘 1 是空盘」那一格与
/// `stale_device_one_refuses_the_writable_mount_naming_every_unit_it_misses_before_any_write` 钉着。
#[test]
fn blank_device_refuses_the_writable_mount_by_name_before_any_write() {""")
edit(FD, """    let cases = [
        (
            "盘 1 是空盘",
            DEVICE_ONE,
            vec![
                (DEVICE_ZERO, sparse_from(&pool.full, DEVICE_ZERO)),
                (DEVICE_ONE, blank_device()),
            ],
            version_key(1, 5),
            journal_position(1, 5),
        ),
        (
            "盘 0 是空盘",
            DEVICE_ZERO,
            vec![
                (DEVICE_ZERO, blank_device()),
                (DEVICE_ONE, sparse_from(&pool.full, DEVICE_ONE)),
            ],
            version_key(1, 4),
            journal_position(1, 4),
        ),
    ];
    for (what, blank, mut devices, expected_version, expected_position) in cases {""", """    let cases = [(
        "盘 1 是空盘",
        DEVICE_ONE,
        vec![
            (DEVICE_ZERO, sparse_from(&pool.full, DEVICE_ZERO)),
            (DEVICE_ONE, blank_device()),
        ],
        version_key(1, 5),
        journal_position(1, 5),
    )];
    let mut devices_with_device_zero_blank = vec![
        (DEVICE_ZERO, blank_device()),
        (DEVICE_ONE, sparse_from(&pool.full, DEVICE_ONE)),
    ];
    let image_before_the_mount_with_device_zero_blank = memory_pool_of(&devices_with_device_zero_blank);
    let refusal_with_device_zero_blank = mount_writable(&pool.parameters, &mut devices_with_device_zero_blank)
        .expect_err("盘 0 是空盘，可写挂载要拒");
    assert_refused_by_the_witness_of_a_newer_publish(
        &refusal_with_device_zero_blank,
        "盘 0 是空盘",
        version_key(1, 4),
        5,
        4,
    );
    assert_eq!(
        memory_pool_of(&devices_with_device_zero_blank),
        image_before_the_mount_with_device_zero_blank,
        "盘 0 是空盘：拒在取号之前，两块盘逐字节不变"
    );
    for (what, blank, mut devices, expected_version, expected_position) in cases {""")
edit(FD, """/// 可写挂载在取号之前拒掉，报 `WritableMountRefusedByDevicesWithoutTheSelectedVersion`，点名盘 1「没有自证过的系统配置」——
/// 两个系统配置槽都读不出，这块盘不「可见」（D18（块里携带什么信息） 已定项 11「可写挂载的顺序」）。一个写、一道屏障都没落下。
/// 所选那一版是 (1, 3)、不是 (1, 4)：txg 4 的根只落在区域 1（盘 1，读不出），txg 4 那条记录点名的单元在盘 1 上那一份验不了、
/// 那次发布不施加（同「盘 0 是空盘」那一格所选的是 (1, 4) 而不是 (1, 5) 的道理）。""", """/// 所选那一版是 (1, 3)、不是 (1, 4)：txg 4 的根只落在区域 1（盘 1，读不出），txg 4 那条记录点名的单元在盘 1 上那一份验不了、
/// 那次发布不施加（同「盘 0 是空盘」那一格所选的是 (1, 4) 而不是 (1, 5) 的道理）。盘 0 的系统配置见证过 txg 4 那次发布
/// （jsn 4 > 所选那一版末条 3），可写挂载在取号之前被 C554 乙拒——逐盘核（点名盘 1「没有自证过的系统配置」）在乙之后不可达，
/// 那一判由 `blank_device_refuses_the_writable_mount_by_name_before_any_write` 的「盘 1 是空盘」那一格钉着。一个写、一道屏障都没落下。""")
edit(FD, """    let refusal = refusal_by_devices_without_the_selected_version(&error, "盘 1 每次读都报错");
    assert_eq!(
        refusal.selected_version,
        version_key(1, 3),
        "所选那一版是 txg 3 那一版：txg 4 的根只在盘 1 上，它那条记录施加不了"
    );
    assert_eq!(
        refusal.selected_version_journal_position,
        journal_position(1, 3),
        "所选那一版那次发布的末条记录"
    );
    assert_eq!(
        refusal.devices,
        vec![DeviceWithoutTheSelectedVersion {
            device: DEVICE_ONE,
            lacking: SelectedVersionLackingOnDevice::NoSelfVerifiedSystemConfiguration,
        }],
        "点名盘 1：两个系统配置槽都读不出，没有自证过的一份"
    );""", """    assert_refused_by_the_witness_of_a_newer_publish(
        &error,
        "盘 1 每次读都报错",
        version_key(1, 3),
        4,
        3,
    );""")
edit(FD, """fn mount_writable_while_every_read_of_device_one_fails_is_refused_naming_device_one() {""", """fn mount_writable_while_every_read_of_device_one_fails_is_refused_by_the_witness_before_any_write() {""")
print("12_fsync_drop done")
