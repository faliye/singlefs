//! P6：「只供测试的开关」今天的做法在发布构建、非测试目标里够不够得着。
//! 这个二进制是普通的 bin 目标（不是测试、不经 dev 依赖、不开任何特性），用 `cargo build --release` 编：
//! 编得过就说明下面两个入口在产品构建里对任何依赖 singlefs-core 的调用方开着。
//! - 会话持有者把空间准入关掉：`MountedSession` 的 `allocator` 是 pub 字段，`PoolAllocator::set_space_admission` 是 pub，
//!   `SpaceAdmission::SkippedByTheTestOnlySwitch` 是 pub 成员（admission.rs 的文档：「只供测试的开关……产品路径恒 `JudgedByTheFormula`」）。
//! - 挂载时就关：`mount::mount_writable_with_space_admission` 是 pub、没有 cfg 门。
//! 不起池、不发布：要证的只是「编得过、够得着」，效果（关掉之后被式子拒的那次覆盖写做成）已有入库用例
//! `crates/singlefs-harness/tests/admission_formula_on_the_mounted_state.rs` 的
//! `an_overwrite_whose_new_slots_exceed_the_available_bytes_is_refused_by_the_space_admission_before_any_write`。
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mounted_session::MountedSession;

fn turn_the_space_admission_off_on_a_live_session(session: &mut MountedSession) {
    session
        .allocator
        .set_space_admission(SpaceAdmission::SkippedByTheTestOnlySwitch);
}

fn main() {
    let entry: fn(&mut MountedSession) = turn_the_space_admission_off_on_a_live_session;
    let mount_entry_is_public = {
        // 只取函数项的名字，不调用：泛型参数由调用方的块设备定，这里不起池。
        let name = std::any::type_name_of_val(&singlefs_core::mount::mount_writable_with_space_admission::<singlefs_core::block_device::FileBackedBlockDevice>);
        name.contains("mount_writable_with_space_admission")
    };
    println!(
        "name=test_only_switch_in_a_release_binary cfg_test={} debug_assertions={} session_entry_compiles={} mount_entry_is_public={} switch_branch_name={}",
        cfg!(test),
        cfg!(debug_assertions),
        !std::ptr::eq(entry as *const (), std::ptr::null()),
        mount_entry_is_public,
        SpaceAdmission::SkippedByTheTestOnlySwitch.branch_name()
    );
}
