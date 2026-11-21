//! 挂载推的抬 F（挂载写的那几串与二次崩溃镜像上落了几步），与树表里只由挂载续做改根的那几种树的根指针。

use super::*;

/// 挂载交回值里挂载透明要的那一样：挂载推没推抬 F，推了的话最后一串抬到的 F。挂载准入在写行与暖机之后推的（`MountOutput::space_admission`）
/// 与挂载续做里为空间推的（`MountOutput::floor_raises_pushed_by_the_continuation`）都算：「续做算挂载的一部分」。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloorRaisedByTheMount {
    NotRaised,
    RaisedTo(CheckpointTxg),
}

impl FloorRaisedByTheMount {
    /// 最后一串是续做里推的（推了的话）：续做接在准入推的那几串之后（`MountOutput::deletion_queue_continuation` 的契约）。
    ///
    /// # Panics
    /// 推过的一串里一次空发布都没有：抬 F 那一串推空发布直到每块盘上都有带新 F 的根，至少一次（`mount::RaisedFloor::publishes`）。
    #[must_use]
    pub fn of_the_mount_output(output: &MountOutput) -> Self {
        match output
            .space_admission
            .floor_raises()
            .iter()
            .chain(output.floor_raises_pushed_by_the_continuation.iter())
            .last()
        {
            None => FloorRaisedByTheMount::NotRaised,
            Some(last_raise) => FloorRaisedByTheMount::RaisedTo(
                last_raise
                    .publishes
                    .last()
                    .expect("抬 F 那一串至少推一次带新 F 的空发布（RaisedFloor::publishes 的契约）")
                    .root
                    .rollback_floor,
            ),
        }
    }
}

/// 二次崩溃镜像上，挂载推的抬 F 那几串可能已经落了一部分没有：挂载没推，或抬 F 的写一个都没持久（崩在取号、写行、暖机里，
/// 暖机那一次发布连同它末尾的系统配置轮换带的都是挂载之前的 F），一步都还没落——F 照 r5 判决 U5 前半句判不变。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloorRaisesAtTheSecondCrash {
    NoneLandedYet,
    /// 挂载推了抬 F，二次崩溃镜像上抬 F 的写有持久的（从第一串把新 F 写进系统配置的那个写起）：F 生效值落在 R1 的与抬到的之间哪一处，条款没写。
    SomeMayHaveLanded,
    /// 挂载推了抬 F，抬 F 从挂载写表的哪个写起取不出（[`FloorRaisesInTheMountWrites::NotLocatedInTheMountWrites`]）：分不开暖机与抬 F，
    /// 这一次挂载的二次崩溃整段 F 不判。
    BoundaryNotLocatedInTheMountWrites,
}

/// 挂载那一段写流（屏障不进表，与二次崩溃切段的是同一张写表）里，挂载推的抬 F 从第几个写起。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloorRaisesInTheMountWrites {
    NotPushed,
    /// 第一串抬 F 的第一个写的下标：那一串先把新 F 写进每块盘的系统配置、过一道屏障，之后才发带新 F 的空发布
    /// （`singlefs_core::transaction::write_the_raised_floor_into_every_system_configuration`）；这个下标之前的写都是取号、写行与暖机的。
    FirstWriteAt(usize),
    /// 挂载交回推了抬 F，那几串的写在这张写表里对不上位置（写数比表还长、界上那个写不是系统配置槽写、或某一次抬 F 发布的根槽写不在界之后）：
    /// 交来的写表与挂载交回的账对不上，界取不出。
    NotLocatedInTheMountWrites,
}

/// 挂载里一次发布交给设备的写数与它写的根身份 (实例代号, checkpoint_txg)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PublishInTheMountWrites {
    pub write_calls: u64,
    pub root: (InstanceGeneration, CheckpointTxg),
}

impl PublishInTheMountWrites {
    #[must_use]
    pub fn of_the_publish(publish: &TransactionOutput) -> Self {
        PublishInTheMountWrites {
            write_calls: publish.writes.total().write_calls,
            root: (publish.root.instance, publish.root.checkpoint_txg),
        }
    }
}

/// 挂载推的一串抬 F 的写账：先把新 F 写进每块盘系统配置的那几个写（`RaisedFloor::system_configuration_writes_before_the_first_publish`），
/// 再是各次带新 F 的空发布（按先后，至少一次：`mount::RaisedFloor::publishes` 的契约，构造时核）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FloorRaiseInTheMountWrites {
    system_configuration_write_calls: u64,
    publishes: Vec<PublishInTheMountWrites>,
}

impl FloorRaiseInTheMountWrites {
    /// # Panics
    /// `publishes` 是空的：抬 F 那一串推空发布直到每块盘上都有带新 F 的根，至少一次（`mount::RaisedFloor::publishes` 的契约）。
    #[must_use]
    pub fn new(
        system_configuration_write_calls: u64,
        publishes: Vec<PublishInTheMountWrites>,
    ) -> Self {
        assert!(
            !publishes.is_empty(),
            "抬 F 那一串至少推一次带新 F 的空发布（RaisedFloor::publishes 的契约）"
        );
        FloorRaiseInTheMountWrites {
            system_configuration_write_calls,
            publishes,
        }
    }

    #[must_use]
    pub fn of_the_raise(raise: &RaisedFloor) -> Self {
        FloorRaiseInTheMountWrites::new(
            raise
                .system_configuration_writes_before_the_first_publish
                .total()
                .write_calls,
            raise
                .publishes
                .iter()
                .map(PublishInTheMountWrites::of_the_publish)
                .collect(),
        )
    }

    fn write_calls(&self) -> u64 {
        self.system_configuration_write_calls
            + self
                .publishes
                .iter()
                .map(|publish| publish.write_calls)
                .sum::<u64>()
    }

    fn checkpoint_txg_of_the_first_publish(&self) -> CheckpointTxg {
        self.publishes
            .first()
            .expect("构造时核过：抬 F 那一串至少一次空发布")
            .root
            .1
    }
}

/// 一次可写挂载交回的、取抬 F 那条界要用的三样：挂载准入在写行与暖机之后推的那几串（`MountOutput::space_admission`）、
/// 续做发成的每一步（`MountOutput::deletion_queue_continuation`）与续做里为空间推的那几串（`MountOutput::floor_raises_pushed_by_the_continuation`，
/// 与续做的各步按 txg 交错）。挂载写表的末尾依次是：准入推的那几串、续做的各步与续做推的那几串。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FloorRaisesOfTheMount {
    pub pushed_by_the_admission: Vec<FloorRaiseInTheMountWrites>,
    pub continuation_steps: Vec<PublishInTheMountWrites>,
    pub pushed_by_the_continuation: Vec<FloorRaiseInTheMountWrites>,
}

impl FloorRaisesOfTheMount {
    #[must_use]
    pub fn of_the_mount_output(output: &MountOutput) -> Self {
        FloorRaisesOfTheMount {
            pushed_by_the_admission: output
                .space_admission
                .floor_raises()
                .iter()
                .map(FloorRaiseInTheMountWrites::of_the_raise)
                .collect(),
            continuation_steps: output
                .deletion_queue_continuation
                .iter()
                .map(std::sync::Arc::as_ref)
                .map(PublishInTheMountWrites::of_the_publish)
                .collect(),
            pushed_by_the_continuation: output
                .floor_raises_pushed_by_the_continuation
                .iter()
                .map(FloorRaiseInTheMountWrites::of_the_raise)
                .collect(),
        }
    }

    /// 从第一串抬 F 的第一个写起、到挂载写表末尾一共几个写，连同这一截里每一次抬 F 发布的根身份；一串都没推时 None。
    /// 第一串是准入推的（推了的话）：它之后是准入推的别的串、续做的每一步与续做推的每一串。准入一串都没推、续做里推了的：
    /// 第一串是续做推的第一串，它之后是续做推的每一串与 txg 大于它第一次空发布的那几步续做（之前的那几步在界之前）。
    fn writes_from_the_first_floor_raise_on(
        &self,
    ) -> Option<(u64, Vec<(InstanceGeneration, CheckpointTxg)>)> {
        let writes_of = |raises: &[FloorRaiseInTheMountWrites]| {
            raises
                .iter()
                .map(FloorRaiseInTheMountWrites::write_calls)
                .sum::<u64>()
        };
        let roots_of = |raises: &[FloorRaiseInTheMountWrites]| {
            raises
                .iter()
                .flat_map(|raise| raise.publishes.iter().map(|publish| publish.root))
                .collect::<Vec<_>>()
        };
        let continuation_steps_after = |checkpoint_txg: Option<CheckpointTxg>| {
            self.continuation_steps
                .iter()
                .filter(|step| checkpoint_txg.is_none_or(|boundary| step.root.1 > boundary))
                .map(|step| step.write_calls)
                .sum::<u64>()
        };
        if !self.pushed_by_the_admission.is_empty() {
            let mut roots = roots_of(&self.pushed_by_the_admission);
            roots.extend(roots_of(&self.pushed_by_the_continuation));
            return Some((
                writes_of(&self.pushed_by_the_admission)
                    + continuation_steps_after(None)
                    + writes_of(&self.pushed_by_the_continuation),
                roots,
            ));
        }
        let first_pushed_by_the_continuation = self.pushed_by_the_continuation.first()?;
        Some((
            writes_of(&self.pushed_by_the_continuation)
                + continuation_steps_after(Some(
                    first_pushed_by_the_continuation.checkpoint_txg_of_the_first_publish(),
                )),
            roots_of(&self.pushed_by_the_continuation),
        ))
    }
}

impl FloorRaisesInTheMountWrites {
    /// 按位置取界（`research/prompts/m3-formal-code-r3-main-verification.md` 第三节第 3 条）：挂载准入推的与续做里推的
    /// 抬 F 那几串都认（[`FloorRaisesOfTheMount`]），界 = 写表长度减去从第一串抬 F 起到写表末尾的写数（每一串是先把新 F 写进每块盘系统配置的
    /// 那几个写再加各次空发布自己的写，续做的各步按 txg 落在第一串之后的那几步也在这一截里）。崩溃注入与 harness 用例取界都经这一份。
    #[must_use]
    pub fn of_the_mount_output(output: &MountOutput, mount_writes: &[RetainedWrite]) -> Self {
        FloorRaisesInTheMountWrites::of_the_floor_raises_of_the_mount(
            &FloorRaisesOfTheMount::of_the_mount_output(output),
            mount_writes,
        )
    }

    /// 不按写进系统配置的 F 认：系统配置里写的是抬到的 F 与盘上已生效的 F 的大者（`singlefs_core::transaction` 写系统配置槽那一处），
    /// 盘上已生效的 F 更高时不等于根里的 F，挂载里更早的系统配置写也可能恰带同一个 F。
    /// 取出的界再核两样：界上那个写是系统配置槽写（每一串先把新 F 写进系统配置），每一次抬 F 发布的根槽写（按实例与 txg 认）都在界之后；
    /// 对不上的交 [`FloorRaisesInTheMountWrites::NotLocatedInTheMountWrites`]，不 panic。
    #[must_use]
    pub fn of_the_floor_raises_of_the_mount(
        floor_raises: &FloorRaisesOfTheMount,
        mount_writes: &[RetainedWrite],
    ) -> Self {
        let Some((writes_from_the_first_floor_raise_on, floor_raise_roots)) =
            floor_raises.writes_from_the_first_floor_raise_on()
        else {
            return FloorRaisesInTheMountWrites::NotPushed;
        };
        let Some(first_write) = usize::try_from(writes_from_the_first_floor_raise_on)
            .ok()
            .and_then(|writes| mount_writes.len().checked_sub(writes))
        else {
            return FloorRaisesInTheMountWrites::NotLocatedInTheMountWrites;
        };
        let the_boundary_is_a_system_configuration_slot_write = mount_writes
            .get(first_write)
            .is_some_and(|write| write.kind == StepKind::SystemConfigurationSlot);
        let roots_written_from_the_boundary: Vec<(InstanceGeneration, CheckpointTxg)> =
            mount_writes[first_write..]
                .iter()
                .filter(|write| write.kind == StepKind::RootRecordFua)
                .filter_map(|write| write.bytes().map(root_identity_written_by))
                .collect();
        let every_floor_raise_root_is_written_from_the_boundary = floor_raise_roots
            .iter()
            .all(|root| roots_written_from_the_boundary.contains(root));
        if the_boundary_is_a_system_configuration_slot_write
            && every_floor_raise_root_is_written_from_the_boundary
        {
            FloorRaisesInTheMountWrites::FirstWriteAt(first_write)
        } else {
            FloorRaisesInTheMountWrites::NotLocatedInTheMountWrites
        }
    }

    /// 一个二次崩溃状态上抬 F 那几串落没落（`persisted_in_the_mount` 与挂载那一段写表逐条对应）：
    /// 从抬 F 的第一个写起有一个持久的，就算可能落了一部分。
    ///
    /// # Panics
    /// `persisted_in_the_mount` 比抬 F 第一个写的下标还短：持久集合与写表对不上。
    #[must_use]
    pub fn at_a_second_crash(self, persisted_in_the_mount: &[bool]) -> FloorRaisesAtTheSecondCrash {
        match self {
            FloorRaisesInTheMountWrites::NotPushed => FloorRaisesAtTheSecondCrash::NoneLandedYet,
            FloorRaisesInTheMountWrites::NotLocatedInTheMountWrites => {
                FloorRaisesAtTheSecondCrash::BoundaryNotLocatedInTheMountWrites
            }
            FloorRaisesInTheMountWrites::FirstWriteAt(first_write) => {
                let persisted_from_the_floor_raises = persisted_in_the_mount
                    .get(first_write..)
                    .expect("持久集合与挂载那一段写表逐条对应，抬 F 的第一个写在表里");
                if persisted_from_the_floor_raises.contains(&true) {
                    FloorRaisesAtTheSecondCrash::SomeMayHaveLanded
                } else {
                    FloorRaisesAtTheSecondCrash::NoneLandedYet
                }
            }
        }
    }
}

/// 树表里只由挂载续做改根的那六种树（判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md` 第二节 Y1：墓碑、谱系、
/// deleted_inodes 三根并进「树表里 inode 树、extent 树的根指针」那一行，同一条续做规则）：extent、inode、deleted_inodes、墓碑树、谱系树，
/// 与退役树（续做的最后一批身份消失时追加，挂载本身不改它；判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 1 条，
/// 正推 K1.2 那一行）。
pub const TREE_KINDS_WHOSE_ROOTS_ONLY_THE_CONTINUATION_REWRITES: [u16; 6] = [
    TREE_KIND_EXTENT,
    TREE_KIND_INODE,
    TREE_KIND_DELETED_INODES,
    TREE_KIND_TOMBSTONE,
    TREE_KIND_LINEAGE,
    TREE_KIND_RETIREMENT,
];

/// 树表里根恒全零、挂载前后必须相同的那四种树（同上判决 Y1「树表种类 6、7、8、12 的根」那一行）：livelist、稀疏旁表、deadlist、logged_ops。
/// 种类 3、4（分配记录树、记账树）的根每一版都重写，不比，归「分配记录」那一行（池级 checker 判 R2 的镜像）。
pub const TREE_KINDS_WHOSE_ROOTS_THE_MOUNT_KEEPS: [u16; 4] = [
    TREE_KIND_LIVELIST,
    TREE_KIND_SPARSE_SIDE_TABLE,
    TREE_KIND_DEADLIST,
    TREE_KIND_LOGGED_OPERATIONS,
];

/// 一版树表里按种类取的条目根指针（条目偏移 14 起 86 字节的盘上原样）；同一种有几条时取盘上先出现的那条（同一种两条由池级 checker 的
/// I-9.16 ③ 判，这里不另判）。只装调用方点名的那几种。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeTableEntryRootPointersByKind {
    pub root_pointer_of_each_kind: BTreeMap<u16, Vec<u8>>,
}

impl TreeTableEntryRootPointersByKind {
    /// 从树表条目的原字节取 `kinds` 那几种的根指针。条目宽不到种类那 2 字节或根指针那 86 字节的条目跳过（宽度由树表字段表那一段比）。
    #[must_use]
    pub fn of_the_entries(entries: &[Vec<u8>], kinds: &[u16]) -> Self {
        let mut root_pointer_of_each_kind = BTreeMap::new();
        for entry in entries {
            let (Some(kind_bytes), Some(root_pointer)) = (
                entry.get(TREE_TABLE_ENTRY_KIND.clone()),
                entry.get(TREE_TABLE_ENTRY_ROOT_POINTER.clone()),
            ) else {
                continue;
            };
            let kind = u16::from_le_bytes(kind_bytes.try_into().expect("切的是种类那 2 字节"));
            if kinds.contains(&kind) {
                root_pointer_of_each_kind
                    .entry(kind)
                    .or_insert_with(|| root_pointer.to_vec());
            }
        }
        TreeTableEntryRootPointersByKind {
            root_pointer_of_each_kind,
        }
    }
}

/// 挂载续做（D8（核心索引结构） 已定项 16 第 6 条）每一步那一版树表里只由续做改根的那六种树
/// （[`TREE_KINDS_WHOSE_ROOTS_ONLY_THE_CONTINUATION_REWRITES`]）的根指针（「续做算挂载的一部分」）：
/// 挂载写的版本里，续做之前的那几版（写行、暖机、准入推的抬 F）照抄 R1 那一版的；续做的每一步改了它们，之后挂载写的版本
/// （续做推的抬 F 的空发布）照抄最近一步的。挂载没续做时一步都没有，挂载写的每一版都比 R1 那一版。
/// 名字里的「用户可见」是只装 inode 树、extent 树两种时起的：崩溃注入（`crash_injection.rs`）照这个名字调它，改名随那一份。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserVisibleTreeRootPointersOfTheContinuation {
    /// (那一步的实例代号与 checkpoint_txg, 五种树的根指针)，按 txg 升序（构造时排）。
    steps: Vec<(
        (InstanceGeneration, CheckpointTxg),
        TreeTableEntryRootPointersByKind,
    )>,
}

impl UserVisibleTreeRootPointersOfTheContinuation {
    /// 挂载没续做（`MountOutput::deletion_queue_continuation` 空）。
    pub const NOT_CONTINUED: Self = Self { steps: Vec::new() };

    /// 根指针取续做每一步交回的树表条目（`TransactionOutput::tree_table_entries`）编成盘上字节之后那 86 字节，与只读恢复读盘上树表条目
    /// 的取法（[`TreeTableEntryRootPointersByKind::of_the_entries`]）同一段字节。
    #[must_use]
    pub fn of_the_mount_output(output: &MountOutput) -> Self {
        let mut steps: Vec<(
            (InstanceGeneration, CheckpointTxg),
            TreeTableEntryRootPointersByKind,
        )> = output
            .deletion_queue_continuation
            .iter()
            .map(|step| {
                (
                    (step.root.instance, step.root.checkpoint_txg),
                    root_pointers_the_continuation_rewrites_in_a_published_version(step),
                )
            })
            .collect();
        steps.sort_by_key(|(version, _)| (version.1, version.0));
        UserVisibleTreeRootPointersOfTheContinuation { steps }
    }

    /// 挂载写的 `version` 那一版该带的五种树的根指针：同一个实例里 txg 不大于它的最近一步续做那一版的；它在续做之前（或挂载没续做）时 None，
    /// 调用方比 R1 那一版。
    pub(super) fn of_the_last_step_at_or_before(
        &self,
        version: (InstanceGeneration, CheckpointTxg),
    ) -> Option<&TreeTableEntryRootPointersByKind> {
        self.steps
            .iter()
            .filter(|(step, _)| step.0 == version.0 && step.1 <= version.1)
            .map(|(_, pointers)| pointers)
            .next_back()
    }
}

/// 一次发布交回的树表条目里只由续做改根的那五种树的根指针盘上字节；树表里没有那种树的条目时那一种不在表里。
fn root_pointers_the_continuation_rewrites_in_a_published_version(
    version: &TransactionOutput,
) -> TreeTableEntryRootPointersByKind {
    let entries: Vec<Vec<u8>> = version
        .tree_table_entries
        .iter()
        .map(crate::verification_read::encoded_tree_table_entry)
        .collect();
    TreeTableEntryRootPointersByKind::of_the_entries(
        &entries,
        &TREE_KINDS_WHOSE_ROOTS_ONLY_THE_CONTINUATION_REWRITES,
    )
}
