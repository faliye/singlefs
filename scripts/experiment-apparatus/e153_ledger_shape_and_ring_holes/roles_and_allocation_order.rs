//! 一次发布触碰的九个角色、三种发布各自的角色表与分配次序。

// ---------------------------------------------------------------------
// 角色（登记第五节 M0.2 / M0.4 的骨架推导）
// ---------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub(super) enum Role {
    InstanceTable,
    TableUnit,
    CentralMapTree,
    ExtentRoot,
    InodeRoot,
    AllocationTree,
    AcctTree,
    DataUnit,
    InodeLeaf,
}

impl Role {
    pub(super) fn span_in_slots(self) -> u64 {
        match self {
            Role::InstanceTable | Role::DataUnit | Role::InodeLeaf => 2,
            Role::TableUnit | Role::CentralMapTree | Role::ExtentRoot | Role::InodeRoot | Role::AllocationTree | Role::AcctTree => 1,
        }
    }
}

/// E 触碰的四个角色（提交内生块的「记账/索引」那一半）。
pub(super) const EMPTY_PUBLISH_ROLES: [Role; 4] = [Role::TableUnit, Role::CentralMapTree, Role::AllocationTree, Role::AcctTree];
/// O 在 E 之外多触碰的四个「文件」角色。
const OVERWRITE_EXTRA_FILE_ROLES: [Role; 4] = [Role::ExtentRoot, Role::InodeRoot, Role::DataUnit, Role::InodeLeaf];

pub(super) fn overwrite_roles() -> Vec<Role> {
    let mut roles: Vec<Role> = EMPTY_PUBLISH_ROLES.to_vec();
    roles.extend_from_slice(&OVERWRITE_EXTRA_FILE_ROLES);
    roles
}

pub(super) fn write_row_roles() -> Vec<Role> {
    let mut roles: Vec<Role> = EMPTY_PUBLISH_ROLES.to_vec();
    roles.push(Role::InstanceTable);
    roles
}

/// 实例切换里被重发的在飞发布是 O 时，写行发布再加 O 的四个文件角色（登记 M0.4）。
pub(super) fn write_row_with_redo_overwrite_roles() -> Vec<Role> {
    let mut roles: Vec<Role> = write_row_roles();
    roles.extend_from_slice(&OVERWRITE_EXTRA_FILE_ROLES);
    roles
}

/// 一次发布分配次序（`transaction.rs` 810-835）：数据单元、实例表单元、extent 根、inode 叶容器、
/// inode 根、分配记录树、记账树、中央映射树、树表单元。只用于落点政策 P-impl 的段内次序。
const ALLOCATION_ORDER: [Role; 9] =
    [Role::DataUnit, Role::InstanceTable, Role::ExtentRoot, Role::InodeLeaf, Role::InodeRoot, Role::AllocationTree, Role::AcctTree, Role::CentralMapTree, Role::TableUnit];

pub(super) fn ordered_roles(roles: &[Role]) -> Vec<Role> {
    ALLOCATION_ORDER.iter().copied().filter(|role| roles.contains(role)).collect()
}
