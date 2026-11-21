use super::*;

// ============================================================================
// 四、真值账本（只从历史取，不读镜像；登记第一节「真值」「祖先链」「存在过的版本」）。
//    每个发布过的版本记父版本、发布路径、写者交回的实例表指针出生 txg、根槽落没落、有没有被某次挂载施加过、账本推的实例表行集。
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct VersionKey {
    pub(super) instance: u32,
    pub(super) txg: u64,
}

impl VersionKey {
    pub(super) fn of_root(root: &RootRecord) -> VersionKey {
        VersionKey {
            instance: root.instance.0,
            txg: root.checkpoint_txg.0,
        }
    }

    pub(super) fn text(self) -> String {
        format!("{}@{}", self.instance, self.txg)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum PublishKind {
    MakeFilesystem,
    RowPublish,
    WarmUp,
    Sweep,
    OtherMountPublish,
    FirstFile,
    Overwrite,
    SequentialWrite,
    NewInode,
    EmptyPublish,
    Rollback,
}

impl PublishKind {
    pub(super) fn name(self) -> &'static str {
        match self {
            PublishKind::MakeFilesystem => "mkfs",
            PublishKind::RowPublish => "row_publish",
            PublishKind::WarmUp => "warm_up",
            PublishKind::Sweep => "sweep",
            PublishKind::OtherMountPublish => "other_mount_publish",
            PublishKind::FirstFile => "first_file",
            PublishKind::Overwrite => "overwrite",
            PublishKind::SequentialWrite => "sequential_write",
            PublishKind::NewInode => "new_inode",
            PublishKind::EmptyPublish => "empty_publish",
            PublishKind::Rollback => "rollback",
        }
    }
}

/// 实例表的一行：(实例代号, 所选根的 txg, 最大已施加事务号)。
pub(super) type RowTriple = (u32, u64, u64);

#[derive(Clone, Debug)]
pub(super) struct LedgerVersion {
    /// 父版本（登记第一节「祖先链」）：普通发布、暖机、清扫、回退发布是发它之前那个会话的现行那一版；写行那次发布是那次挂载恢复之后的有效根。
    pub(super) parent: Option<VersionKey>,
    pub(super) kind: PublishKind,
    /// 写者交回的根记录（崩在根槽写之前的那一版取它录制流里那次根槽写的内容）里实例表指针的出生 txg。
    pub(super) pointer_birth_txg: u64,
    pub(super) root_landed: bool,
    pub(super) applied_by_a_mount: bool,
    /// 发布的先后（账本里的第几次发布），P1、P2 按它取「第一版」「相邻两版」。
    pub(super) publish_order: usize,
    /// 账本按每次挂载写的行与清扫删的行推出的这一版实例表行集（停机条款 S3 比它）。
    pub(super) rows: Arc<BTreeSet<RowTriple>>,
}

impl LedgerVersion {
    /// 存在过（登记第一节）：根槽写落进了镜像，或之后某一次可写挂载的恢复把它从 journal 记录里施加了出来。
    pub(super) fn existed(&self) -> bool {
        self.root_landed || self.applied_by_a_mount
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct Ledger {
    pub(super) versions: BTreeMap<VersionKey, LedgerVersion>,
    /// 根环每个 (区域, 槽) 此刻住着哪一版（按根槽写落下的次序，后写的盖前写的；停机条款 S1 比它）。
    pub(super) ring_occupants: BTreeMap<(u64, u64), VersionKey>,
    /// 镜像上被读坏的根环槽（S1 里这几槽不该再被读出）。
    pub(super) corrupted_ring_slots: BTreeSet<(u64, u64)>,
    /// S2：落下的根槽写里 [212, 220) 字节与写者交回的出生 txg 比过几条、对不上的逐条、没有交回值可比的几条。
    pub(super) root_slot_writes_compared: u64,
    pub(super) root_slot_write_mismatches: Vec<String>,
    pub(super) root_slot_writes_without_a_returned_root: u64,
    /// 落点公式（本地公式、`target_for_publish`、录制流里那次写的位置）三方对不上的逐条（S4 的「落点公式」那一格）。
    pub(super) ring_formula_mismatches: Vec<String>,
    /// 账本自己认不出的东西（根槽写解不出根、有效根不在账本里……），非空时这段历史不可信。
    pub(super) consistency_problems: Vec<String>,
}

impl Ledger {
    pub(super) fn rows_of(&self, key: Option<VersionKey>) -> Arc<BTreeSet<RowTriple>> {
        key.and_then(|key| self.versions.get(&key))
            .map(|version| Arc::clone(&version.rows))
            .unwrap_or_default()
    }

    /// R 的祖先链（不含 R 自己），沿父版本一路往前；链上出现环（不该有）就停。
    fn ancestors_of(&self, key: VersionKey) -> Vec<VersionKey> {
        let mut chain = Vec::new();
        let mut seen = BTreeSet::new();
        let mut cursor = self.versions.get(&key).and_then(|version| version.parent);
        while let Some(ancestor) = cursor {
            if !seen.insert(ancestor) {
                break;
            }
            chain.push(ancestor);
            cursor = self
                .versions
                .get(&ancestor)
                .and_then(|version| version.parent);
        }
        chain
    }
}

/// 一个洞对的真值（登记第一节「真值」）：直接接上 ⟺ R 的父版本是 R⁻ 那一组里的某一版；否则看 R⁻ 那一组有没有一版在 R 的祖先链上。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Truth {
    DirectlyBridged,
    ExistedVersionsInBetween,
    PreviousNotOnTheChain,
    /// R 不在账本里（S1、S5 本该先拦下）：这一对不进 E1 / E2，单列。
    VersionMissingFromTheLedger,
}

impl Truth {
    pub(super) fn name(self) -> &'static str {
        match self {
            Truth::DirectlyBridged => "directly_bridged",
            Truth::ExistedVersionsInBetween => "existed_versions_in_between",
            Truth::PreviousNotOnTheChain => "previous_not_on_the_chain",
            Truth::VersionMissingFromTheLedger => "version_missing_from_the_ledger",
        }
    }
}

/// 洞对 (R⁻ 那一组, R) 的真值：只读账本里的父版本链，链上只算存在过的版本（纯函数，单测喂已知答案）。
pub(super) fn truth_of_the_hole_pair(
    ledger: &Ledger,
    previous_group: &[VersionKey],
    version: VersionKey,
) -> Truth {
    if !ledger.versions.contains_key(&version) {
        return Truth::VersionMissingFromTheLedger;
    }
    let ancestors: Vec<VersionKey> = ledger
        .ancestors_of(version)
        .into_iter()
        .filter(|ancestor| {
            ledger
                .versions
                .get(ancestor)
                .is_some_and(LedgerVersion::existed)
        })
        .collect();
    // 往前第一个存在过的祖先（合法历史里父版本都存在过，它就是父版本）。
    if ancestors
        .first()
        .is_some_and(|parent| previous_group.contains(parent))
    {
        return Truth::DirectlyBridged;
    }
    if previous_group
        .iter()
        .any(|previous| ancestors.contains(previous))
    {
        Truth::ExistedVersionsInBetween
    } else {
        Truth::PreviousNotOnTheChain
    }
}

#[cfg(test)]
mod tests;
