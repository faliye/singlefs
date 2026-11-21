//! 池的一版（`PoolVersion`）与事务里的单元角色（`TransactionUnit`）、多层码 2 树、文件版本的树标识与发布出的单元。

use super::*;

/// 池里现行的那一版：还没发布过文件版本（树表 0 条，零单元发布或写行那次发布写出的根）、带文件的一版，
/// 或有树而文件集合为空的一版（D8（核心索引结构） 已定项 16 第 5 条，判定见 [`TransactionOutput::is_a_version_with_trees_and_no_file`]）。
/// 有树的两种由 [`PoolVersion::of_a_version_of_trees`] 按同一个判定分，不由调用方挑。
/// 有树的一版按引用计数共享（判决 `research/prompts/m4-gpugc-publish-growth-r1-main-verification.md` 第二节 Q2.2「版本列表与挂载交回值共享同一份」）：
/// 会话与挂载的现行那一版、交回给调用方的版本列表（[`DeletionQueueWalk::steps_published`]、`mounted_session::VersionsPublishedInThisChange`、
/// `mount::MountOutput` 里的那几版）指着同一份 [`TransactionOutput`]，不再各拷一整版。一版发出来之后不再改：要改的地方经 [`Arc::make_mut`]
/// 改一份自己的（还有别人指着时先拷一份，没有时就地改），别人指着的那一份不动。相等照内容比，与共不共享无关。
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(
    clippy::large_enum_variant,
    reason = "一次挂载只有几个这样的值、不进集合，树表 0 条那一版按值搬无所谓；装箱只多一层解引用"
)]
pub enum PoolVersion {
    WithoutFile(VersionWithoutFilePublishOutput),
    WithFile(Arc<TransactionOutput>),
    /// 有树、文件集合为空（D8（核心索引结构） 已定项 16 第 5 条，不借 `WithoutFile`）：接在它后面的只有 `publish_new_inodes`（size 0）、
    /// 写行与空发布（K-U5 X1），会话、挂载按这个成员分流。
    WithTreesAndNoFile(Arc<TransactionOutput>),
}

impl PoolVersion {
    /// 一版有树的发布出来（或从盘上重建出来）之后归哪个成员：[`TransactionOutput::is_a_version_with_trees_and_no_file`] 为真归
    /// `WithTreesAndNoFile`，否则 `WithFile`。
    #[must_use]
    pub fn of_a_version_of_trees(version: TransactionOutput) -> PoolVersion {
        PoolVersion::of_a_shared_version_of_trees(Arc::new(version))
    }
    /// 同 [`Self::of_a_version_of_trees`]，收的是已经共享着的一版（版本列表里那一份）：成员里指着同一份，不拷。
    #[must_use]
    pub fn of_a_shared_version_of_trees(version: Arc<TransactionOutput>) -> PoolVersion {
        if version.is_a_version_with_trees_and_no_file() {
            PoolVersion::WithTreesAndNoFile(version)
        } else {
            PoolVersion::WithFile(version)
        }
    }
    #[must_use]
    pub fn root(&self) -> &RootRecord {
        match self {
            PoolVersion::WithoutFile(version) => &version.root,
            PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => {
                &version.root
            }
        }
    }
    #[must_use]
    pub fn record(&self) -> &JournalRecord {
        match self {
            PoolVersion::WithoutFile(version) => &version.record,
            PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => {
                &version.record
            }
        }
    }
    #[must_use]
    pub fn record_bytes(&self) -> &[u8] {
        match self {
            PoolVersion::WithoutFile(version) => &version.record_bytes,
            PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => {
                &version.record_bytes
            }
        }
    }
    /// 带文件的那一版；还没发布过文件版本、或有树而文件集合为空时 `None`。
    #[must_use]
    pub fn file_version(&self) -> Option<&TransactionOutput> {
        match self {
            PoolVersion::WithoutFile(_) | PoolVersion::WithTreesAndNoFile(_) => None,
            PoolVersion::WithFile(version) => Some(Arc::as_ref(version)),
        }
    }
    /// 同 [`Self::file_version`]，交回自己的一份：没有别人指着就原样交出，还有别人指着（版本列表里那一份）就拷一份。
    #[must_use]
    pub fn into_file_version(self) -> Option<TransactionOutput> {
        match self {
            PoolVersion::WithoutFile(_) | PoolVersion::WithTreesAndNoFile(_) => None,
            PoolVersion::WithFile(version) => Some(Arc::unwrap_or_clone(version)),
        }
    }
    /// 有树的那一版（带文件的、有树而文件集合为空的）；树表 0 条时 `None`。
    #[must_use]
    pub fn version_of_trees(&self) -> Option<&TransactionOutput> {
        match self {
            PoolVersion::WithoutFile(_) => None,
            PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => {
                Some(Arc::as_ref(version))
            }
        }
    }
    /// 同 [`Self::version_of_trees`]，交回共享着的那一份本身（不拷）：放进版本列表、与现行那一版指着同一份时用。
    #[must_use]
    pub fn shared_version_of_trees(&self) -> Option<&Arc<TransactionOutput>> {
        match self {
            PoolVersion::WithoutFile(_) => None,
            PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => {
                Some(version)
            }
        }
    }
    /// 同 [`Self::into_file_version`] 的有树那两种。
    #[must_use]
    pub fn into_version_of_trees(self) -> Option<TransactionOutput> {
        self.into_shared_version_of_trees()
            .map(Arc::unwrap_or_clone)
    }
    /// 同 [`Self::shared_version_of_trees`]，交出共享着的那一份本身（不拷）。
    #[must_use]
    pub fn into_shared_version_of_trees(self) -> Option<Arc<TransactionOutput>> {
        match self {
            PoolVersion::WithoutFile(_) => None,
            PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => {
                Some(version)
            }
        }
    }
    /// 树表 0 条的那一版；有树的一版时 `None`。
    #[must_use]
    pub fn into_version_without_file(self) -> Option<VersionWithoutFilePublishOutput> {
        match self {
            PoolVersion::WithoutFile(version) => Some(version),
            PoolVersion::WithFile(_) | PoolVersion::WithTreesAndNoFile(_) => None,
        }
    }
    /// 这一版的写账：这次发布交给设备、设备报了成功的写（落盘或重发那一遍之后现记）。
    #[must_use]
    pub fn writes(&self) -> &WritesByStructureKind {
        match self {
            PoolVersion::WithoutFile(version) => &version.writes,
            PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => {
                &version.writes
            }
        }
    }
    /// 换上写账、别的一个字段都不动（成员不换）。
    #[must_use]
    pub(super) fn with_writes(mut self, writes: WritesByStructureKind) -> Self {
        match &mut self {
            PoolVersion::WithoutFile(version) => version.writes = writes,
            PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => {
                Arc::make_mut(version).writes = writes;
            }
        }
        self
    }
}

/// 新池新建文件写出的十二个单元（八个角色，分配记录树那一个角色按位置寻址是五个节点），声明序 = D3（空间分配） 已定项 10 ⑤ 的 bump 次序（按树 ID 升序、树内先叶后根、映射树倒数第二、树表最末），
/// 也是出生序号的发号次序（D19（块指针的结构与宽度预算） 已定项 9）与字节表零的 t1..t12。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TransactionUnit {
    /// 文件的一个数据单元（码 1）。带的是它在文件里的单元序号：一个文件跨多个单元时（里程碑「覆盖写、释放、回退与复用」并行线一）
    /// 每个单元一个角色，落点、映射 key、点名项与按结构种类记的账都按角色走，共用一个角色就分不开两个单元。
    /// 只有一个单元的文件（新池新建文件那一档）就是 [`DataUnitIndexInFile::FIRST`]。
    Data(DataUnitIndexInFile),
    /// 第一个文件那一棵 extent 树下段里的一个节点（D8（核心索引结构） 已定项 14：下段一个文件一棵、按数据单元号按位置寻址），
    /// 带它在下段里的位置（层级、同层序号）；下段的根也是这一族（最高那一层的第 0 个）。只有一个数据单元的文件不建下段（内联），没有这一族。
    ExtentLowerNode(ExtentLowerNodePosition),
    /// extent 树上段里根之下的一个节点（按 inode 号按位置寻址），带它在上段里的位置。上段只有一层时没有这一族（根兼叶是 `ExtentRoot`）。
    ExtentUpperNodeBelowTheRoot(ExtentUpperNodePosition),
    /// extent 树的根：上段的根（树表条目里 extent 树那一条指着它）。
    ExtentRoot,
    /// inode 树的一片叶容器（码 3 打包记录类型 2，一容器 233 条 140 字节记录，D8（核心索引结构） 已定项 6）。
    /// 带的是它在树里的叶序，不是容器号：一棵树可以有好几片，角色按叶序分开
    /// （落点、映射 key、点名项、按结构种类记的账都按角色走，共用一个角色就分不开两片叶）。
    InodeLeafContainer(InodeLeafContainerIndexInTree),
    InodeRoot,
    /// 分配记录树里根之下的一个节点（D8（核心索引结构） 已定项 14：按绝对槽号按位置寻址），带它的位置（层级、盘、同盘同层序号）。
    /// 位置由槽号算出、不随版本挪动：上一版同一个位置上的节点照抄进来时角色不变。
    AllocationTreeNodeBelowTheRoot(AllocationRecordTreeNodePosition),
    /// 分配记录树的根（罩整个 key 空间；带文件的一版里树表条目指着它，树表 0 条的一版里根记录那一项指着它）。
    AllocationTree,
    /// 记账树里根之下的一个节点（D8（核心索引结构） 已定项 11：多层码 2 树）：带它在这一版树里的位置（层级、同层从左数第几个）。
    /// 根兼叶那一档没有这个角色——记账树只有一个节点时它就是 `AccountingTree`。位置是这一版的：上一版的节点照抄进来时
    /// 位置可以变（它左边的叶切开了），角色跟着这一版的位置走（`code_two_tree::CodeTwoTreePlan::origins` 记着它从哪来）。
    AccountingTreeNodeBelowTheRoot(CodeTwoTreeNodePosition),
    /// 记账树的根（只有一个节点时它就是根兼叶）。
    AccountingTree,
    /// deadlist 树里根之下的一个节点（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3：第三棵多层码 2 树），
    /// 同 `AccountingTreeNodeBelowTheRoot`。
    DeadlistTreeNodeBelowTheRoot(CodeTwoTreeNodePosition),
    /// deadlist 树的根（只有一个节点时它就是根兼叶）：树里有条目的一版才有它，没有条目时树表条目的根指针置零、这个角色不在那一版里。
    DeadlistTree,
    /// `deleted_inodes` 树的根（码 2、层级 0，D8（核心索引结构） 已定项 16）：队列非空的一版才有它，队列空时树表条目的根指针置零、
    /// 这个角色不在那一版里。任何一版至多一条 key（第 6 条），根之下不长层。
    DeletedInodesRoot,
    /// 墓碑树的一片叶容器（码 3 打包记录类型 1，D18（块里携带什么信息） 已定项 24）：带它在树里的叶序（容器号从 1 起、叶序从 0 起）。
    TombstoneLeafContainer(TombstoneContainerIndexInTree),
    /// 墓碑树的根（码 2，条目照 inode 树内部节点的 120 字节形态；层级 1 时孩子是容器，长高之后孩子是码 2 节点）：树里有容器的一版才有它。
    TombstoneRoot,
    /// 墓碑树根之下的一个码 2 节点（根满长高之后才有，D8（核心索引结构） 已定项 6）：带它的位置（层级、同层序号），位置不随版本挪动
    /// （长高那次旧根留在原位置，从这一版起是这个角色）。
    TombstoneNodeBelowTheRoot(ContainerTreeNodePosition),
    /// 谱系树的一片叶容器（码 3 打包记录类型 5，D18（块里携带什么信息） 已定项 24 第 5 条）：带它在树里的叶序。
    /// 只有管理员回退那次发布追加记录、重写它；别的发布照抄。
    LineageLeafContainer(LineageContainerIndexInTree),
    /// 谱系树的根（码 2，条目同墓碑树根）：树里有容器的一版才有它。
    LineageRoot,
    /// 谱系树根之下的一个码 2 节点（根满长高之后才有），同 `TombstoneNodeBelowTheRoot`。
    LineageNodeBelowTheRoot(ContainerTreeNodePosition),
    /// logged_ops 树的根（码 2、层级 0，判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1：单树、定宽联合条目 44 字节，
    /// `crate::logged_operations`）：树里有条目的一版才有它，没有条目时树表条目的根指针置零、这个角色不在那一版里。全池活整理意图
    /// 至多一条（e1），一个根装 369 条，根之下不长层。只有整理签发意图、删意图的那次发布重写它（[`publish_compaction_step`]），别的发布照抄。
    LoggedOperationsRoot,
    /// 退役树的一片叶容器（码 3 打包记录类型 3，判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 5 条）：带它在树里的叶序。
    /// 只有身份消失（删空摘掉、被左邻吸收）的那次发布追加记录、重写它；别的发布照抄。
    RetirementLeafContainer(RetirementContainerIndexInTree),
    /// 退役树的根（码 2，条目同墓碑树根）：树里有容器的一版才有它。
    RetirementRoot,
    /// 退役树根之下的一个码 2 节点（根满长高之后才有），同 `TombstoneNodeBelowTheRoot`。
    RetirementNodeBelowTheRoot(ContainerTreeNodePosition),
    /// 中央映射树里根之下的一个节点，同 `AccountingTreeNodeBelowTheRoot`。映射树的节点都豁免映射
    /// （D19（块指针的结构与宽度预算） 已定项 8「映射树自己的节点」）：父条目里的位置条目是权威。
    MappingTreeNodeBelowTheRoot(CodeTwoTreeNodePosition),
    /// 中央映射树的根（只有一个节点时它就是根兼叶），根指针住根记录（D19（块指针的结构与宽度预算） 已定项 11）。
    MappingTree,
    TreeTable,
    /// 实例表单元（码 3 打包记录类型 4）：mkfs 种下第一片，之后每次可写挂载写行时重写（D18（块里携带什么信息） 已定项 11）；
    /// 不在新池新建文件的八个角色里，覆盖写、释放、回退与复用起才进发布路径。它是链上第 0 片——根记录直接持有的那一片；
    /// 多于一片时第 1 片起是 [`TransactionUnit::InstanceTablePageAfterTheFirst`]。
    InstanceTable,
    /// 实例表链上第 1 片起的一片（码 3 打包记录类型 4，身份 (0, 4, 片序号, 0)，D18（块里携带什么信息） 已定项 11）：它的指针不在根记录里，
    /// 在上一片末尾那条链指针记录里。带的是片序号，恒 ≥ 1（第 0 片是 [`TransactionUnit::InstanceTable`]，
    /// 两者由 [`TransactionUnit::of_instance_table_page`] 按片序号分）。表的行多于一片装得下的 369 行时写行那次发布才写出它们，
    /// 在 bump 次序里尾片先（D3（空间分配） 已定项 10 ⑤，用户 2026-09-24 定案）。
    InstanceTablePageAfterTheFirst(InstanceTablePageIndex),
    /// 第一个文件之外一个文件的一个数据单元（码 1；按 inode 号写数据的入口写出来的，[`publish_sequential_write_of_an_inode`]）：
    /// 带它属于哪个 inode 与它在那个文件里的单元序号。第一个文件的数据单元仍是 [`TransactionUnit::Data`]——落点、映射 key、点名项、
    /// 按结构种类记的账都按角色走，两个文件共用一个角色就分不开各自的单元。在 bump 次序里同 [`TransactionUnit::Data`]（数据单元排最前）。
    DataOfAFileOtherThanTheFirst {
        inode: InodeNumber,
        unit: DataUnitIndexInFile,
    },
    /// 第一个文件之外一个文件那一棵 extent 树下段里的一个节点（D8（核心索引结构） 已定项 14：下段一个文件一棵、按数据单元号按位置寻址），
    /// 带它属于哪个 inode 与它在下段里的位置。只有一个数据单元的文件不建下段（内联），没有这一族。在 bump 次序里排在第一个文件的下段之后、
    /// 上段之前（树内先叶后根：上段叶条目要下段根这一版的指针）。
    ExtentLowerNodeOfAFileOtherThanTheFirst {
        inode: InodeNumber,
        position: ExtentLowerNodePosition,
    },
}

/// 多层码 2 树里哪一棵：记账树、中央映射树（D8（核心索引结构） 已定项 11 的两棵照分裂做的树）或 deadlist 树
/// （判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3：第三棵多层码 2 树，种类 8，key 43、value 空）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MultiLevelCodeTwoTree {
    Accounting,
    CentralMapping,
    Deadlist,
}

impl MultiLevelCodeTwoTree {
    /// 这棵树在这一版里某个位置上的节点的角色：根是 `AccountingTree` / `MappingTree`，根之下各按位置。
    #[must_use]
    pub fn role_of_node(
        self,
        position: CodeTwoTreeNodePosition,
        shape: &CodeTwoTreeShape,
    ) -> TransactionUnit {
        match (self, shape.is_root(position)) {
            (MultiLevelCodeTwoTree::Accounting, true) => TransactionUnit::AccountingTree,
            (MultiLevelCodeTwoTree::Accounting, false) => {
                TransactionUnit::AccountingTreeNodeBelowTheRoot(position)
            }
            (MultiLevelCodeTwoTree::CentralMapping, true) => TransactionUnit::MappingTree,
            (MultiLevelCodeTwoTree::CentralMapping, false) => {
                TransactionUnit::MappingTreeNodeBelowTheRoot(position)
            }
            (MultiLevelCodeTwoTree::Deadlist, true) => TransactionUnit::DeadlistTree,
            (MultiLevelCodeTwoTree::Deadlist, false) => {
                TransactionUnit::DeadlistTreeNodeBelowTheRoot(position)
            }
        }
    }

    /// 叶条目的字段表宽：记账条目 34、映射条目 55、deadlist 条目 43（value 空，条目就是 key）。
    #[must_use]
    pub fn leaf_entry_width_in_bytes(self) -> usize {
        match self {
            MultiLevelCodeTwoTree::Accounting => {
                usize::try_from(ACCOUNTING_ENTRY_BYTES).expect("34")
            }
            MultiLevelCodeTwoTree::CentralMapping => {
                usize::try_from(MAPPING_ENTRY_BYTES).expect("55")
            }
            MultiLevelCodeTwoTree::Deadlist => DEADLIST_ENTRY_KEY_BYTES,
        }
    }

    #[must_use]
    pub const fn key_field_widths(self) -> CodeTwoKeyFieldWidths {
        match self {
            MultiLevelCodeTwoTree::Accounting => CodeTwoKeyFieldWidths::ACCOUNTING,
            MultiLevelCodeTwoTree::CentralMapping => CodeTwoKeyFieldWidths::CENTRAL_MAPPING,
            MultiLevelCodeTwoTree::Deadlist => CodeTwoKeyFieldWidths::DEADLIST,
        }
    }

    /// 从盘上读这棵树时对它的期望（`code_two_tree::read_code_two_tree`）：这一版里它的树 ID 由调用方给（从水位发的号）。
    #[must_use]
    pub fn read_expectation(self, tree_identifier: TreeIdentifier) -> CodeTwoTreeReadExpectation {
        CodeTwoTreeReadExpectation {
            tree: tree_identifier,
            field_widths: self.key_field_widths(),
            internal_entry_name: match self {
                MultiLevelCodeTwoTree::Accounting => "记账树内部条目",
                MultiLevelCodeTwoTree::CentralMapping => "中央映射树内部条目",
                MultiLevelCodeTwoTree::Deadlist => "deadlist 树内部条目",
            },
        }
    }

    /// 格式算出来的节点容量：记账树叶 477、内部 150，中央映射树叶 294、内部 143，deadlist 树叶 376、内部 125。
    #[must_use]
    pub fn node_capacity_of_the_node_format(self) -> CodeTwoTreeNodeCapacity {
        CodeTwoTreeNodeCapacity::of_the_node_format(
            self.key_field_widths().key_width_in_bytes(),
            self.leaf_entry_width_in_bytes(),
        )
    }
}

/// 记账树与中央映射树的节点容量从哪来：产品路径按节点格式算；只供测试的开关把两棵树各自压小
/// （`.claude/rules/fs-design.md` 五条硬要求第 2 条：每条分支必须能被测试强制进入——格式算出来的容量下，记账树要 80 块盘才分裂，
/// 中央映射树今天走得到的条目数装不满一个节点，分裂、收缩、降高这几条写序不压小就进不去）。只住写入口的内存：重开之后按产品路径起步。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodeTwoTreeNodeCapacities {
    FromTheNodeFormat,
    CappedForTests {
        accounting: CodeTwoTreeNodeCapacity,
        central_mapping: CodeTwoTreeNodeCapacity,
    },
}

impl CodeTwoTreeNodeCapacities {
    /// 运行时报出走的是哪一条（`.claude/rules/fs-design.md` 五条硬要求第 4 条：分支必须可观测）。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            CodeTwoTreeNodeCapacities::FromTheNodeFormat => {
                "code_two_tree_node_capacities_from_the_node_format"
            }
            CodeTwoTreeNodeCapacities::CappedForTests { .. } => {
                "code_two_tree_node_capacities_capped_for_tests"
            }
        }
    }

    /// 这棵树这会儿一个节点最多装几条。
    #[must_use]
    pub fn of_tree(self, tree: MultiLevelCodeTwoTree) -> CodeTwoTreeNodeCapacity {
        match (self, tree) {
            (CodeTwoTreeNodeCapacities::FromTheNodeFormat, _) => {
                tree.node_capacity_of_the_node_format()
            }
            (
                CodeTwoTreeNodeCapacities::CappedForTests { accounting, .. },
                MultiLevelCodeTwoTree::Accounting,
            ) => accounting,
            (
                CodeTwoTreeNodeCapacities::CappedForTests {
                    central_mapping, ..
                },
                MultiLevelCodeTwoTree::CentralMapping,
            ) => central_mapping,
            // deadlist 树不随这个开关压小：开关只罩记账树与中央映射树（它的两个字段），deadlist 树的分裂、收缩走的是同一段
            // `code_two_tree` 规划（那两棵树压小之后测过的那几条写序）。
            (CodeTwoTreeNodeCapacities::CappedForTests { .. }, MultiLevelCodeTwoTree::Deadlist) => {
                MultiLevelCodeTwoTree::Deadlist.node_capacity_of_the_node_format()
            }
        }
    }
}

impl TransactionUnit {
    pub const IN_BUMP_ORDER: [TransactionUnit; 8] = [
        TransactionUnit::Data(DataUnitIndexInFile::FIRST),
        TransactionUnit::ExtentRoot,
        TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST),
        TransactionUnit::InodeRoot,
        TransactionUnit::AllocationTree,
        TransactionUnit::AccountingTree,
        TransactionUnit::MappingTree,
        TransactionUnit::TreeTable,
    ];

    /// 落点跨几个槽：用户数据与 32768 对齐的提交内生块 2 槽，16 KiB 节点 1 槽。
    #[must_use]
    pub fn span_slots(self) -> u64 {
        match self.placement() {
            PlacementRule::UserData => 2,
            PlacementRule::CommitGenerated(UnitFootprint::TwoSlotsAligned) => 2,
            PlacementRule::CommitGenerated(UnitFootprint::OneSlot) => 1,
        }
    }

    /// 字节表七的步号。字节表七只登记了一片叶容器那一档的 `t3`（新池新建文件的 inode 树只有一片叶）；
    /// 第二片起写成 `t3+叶序`，那是这一版给多容器起的名字，只出现在报错与用例消息里，不进字节表、不进段序列登记表。
    /// 数据单元同一个写法：第一个单元是字节表的 `t1`，第二个起写成 `t1+单元序号`。
    #[must_use]
    pub fn tag(self) -> String {
        match self {
            TransactionUnit::Data(DataUnitIndexInFile::FIRST) => "t1".to_string(),
            TransactionUnit::Data(index) => format!("t1+{}", index.0),
            // 字节表七只登记了 extent 树根兼叶那一档的 t2；下段与上段根之下的节点写成「t2下@层级.序号」「t2上@层级.序号」，
            // 只出现在报错与用例消息里。
            TransactionUnit::ExtentLowerNode(position) => {
                format!("t2下@{}.{}", position.level, position.index)
            }
            TransactionUnit::ExtentUpperNodeBelowTheRoot(position) => {
                format!("t2上@{}.{}", position.level, position.index)
            }
            // 第一个文件之外的文件：字节表七没有登记，写成「t1#inode 号+单元序号」「t2下#inode 号@层级.序号」，只出现在报错与用例消息里。
            TransactionUnit::DataOfAFileOtherThanTheFirst { inode, unit } => {
                format!("t1#{}+{}", inode.0, unit.0)
            }
            TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { inode, position } => {
                format!("t2下#{}@{}.{}", inode.0, position.level, position.index)
            }
            TransactionUnit::ExtentRoot => "t2".to_string(),
            TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST) => {
                "t3".to_string()
            }
            TransactionUnit::InodeLeafContainer(index) => format!("t3+{}", index.0),
            TransactionUnit::InodeRoot => "t4".to_string(),
            // 分配记录树根之下的节点写成「t5@层级.盘.同盘同层序号」，只出现在报错与用例消息里。
            TransactionUnit::AllocationTreeNodeBelowTheRoot(position) => format!(
                "t5@{}.{}.{}",
                position.level, position.device.0, position.index_in_device
            ),
            TransactionUnit::AllocationTree => "t5".to_string(),
            // 字节表七只登记了根兼叶那一档的 t6 / t7；根之下的节点写成「t6@层级.同层序号」，只出现在报错与用例消息里。
            TransactionUnit::AccountingTreeNodeBelowTheRoot(position) => {
                format!("t6@{}.{}", position.level, position.index_in_level)
            }
            TransactionUnit::AccountingTree => "t6".to_string(),
            // deadlist 树两种角色字节表七没有登记：写成「tdl@层级.同层序号」「tdl」，只出现在报错与用例消息里。
            TransactionUnit::DeadlistTreeNodeBelowTheRoot(position) => {
                format!("tdl@{}.{}", position.level, position.index_in_level)
            }
            TransactionUnit::DeadlistTree => "tdl".to_string(),
            // 删除新写出的三种角色字节表七没有登记（layout/03-third-txn.md 只登记形态、不编步号）：写成「td」「tt@叶序」「tt」，
            // 只出现在报错与用例消息里。
            TransactionUnit::DeletedInodesRoot => "td".to_string(),
            TransactionUnit::TombstoneLeafContainer(index) => format!("tt@{}", index.0),
            TransactionUnit::TombstoneRoot => "tt".to_string(),
            TransactionUnit::TombstoneNodeBelowTheRoot(position) => {
                format!("tt^{}.{}", position.level, position.index_in_level)
            }
            // 谱系树两种角色同样没有登记：写成「tl@叶序」「tl」。
            TransactionUnit::LineageLeafContainer(index) => format!("tl@{}", index.0),
            TransactionUnit::LineageRoot => "tl".to_string(),
            TransactionUnit::LineageNodeBelowTheRoot(position) => {
                format!("tl^{}.{}", position.level, position.index_in_level)
            }
            // logged_ops 根同样没有登记：写成「tlo」。
            TransactionUnit::LoggedOperationsRoot => "tlo".to_string(),
            // 退役树三种角色同样没有登记：写成「tr@叶序」「tr」「tr^层级.同层序号」。
            TransactionUnit::RetirementLeafContainer(index) => format!("tr@{}", index.0),
            TransactionUnit::RetirementRoot => "tr".to_string(),
            TransactionUnit::RetirementNodeBelowTheRoot(position) => {
                format!("tr^{}.{}", position.level, position.index_in_level)
            }
            TransactionUnit::MappingTreeNodeBelowTheRoot(position) => {
                format!("t7@{}.{}", position.level, position.index_in_level)
            }
            TransactionUnit::MappingTree => "t7".to_string(),
            TransactionUnit::TreeTable => "t8".to_string(),
            TransactionUnit::InstanceTable => "ti".to_string(),
            TransactionUnit::InstanceTablePageAfterTheFirst(page) => format!("ti+{}", page.0),
        }
    }

    /// 实例表链上第 `page` 片的角色：第 0 片是根记录持有的 [`TransactionUnit::InstanceTable`]，第 1 片起是
    /// [`TransactionUnit::InstanceTablePageAfterTheFirst`]。
    #[must_use]
    pub fn of_instance_table_page(page: InstanceTablePageIndex) -> Self {
        if page == InstanceTablePageIndex::FIRST {
            TransactionUnit::InstanceTable
        } else {
            TransactionUnit::InstanceTablePageAfterTheFirst(page)
        }
    }

    /// 这个角色是不是实例表链上的一片（第 0 片或第 1 片起）。
    #[must_use]
    pub const fn is_a_page_of_the_instance_table(self) -> bool {
        match self {
            TransactionUnit::InstanceTable | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
                true
            }
            TransactionUnit::DataOfAFileOtherThanTheFirst { .. }
            | TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. }
            | TransactionUnit::Data(_)
            | TransactionUnit::ExtentLowerNode(_)
            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
            | TransactionUnit::ExtentRoot
            | TransactionUnit::InodeLeafContainer(_)
            | TransactionUnit::InodeRoot
            | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
            | TransactionUnit::AllocationTree
            | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
            | TransactionUnit::AccountingTree
            | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
            | TransactionUnit::DeadlistTree
            | TransactionUnit::DeletedInodesRoot
            | TransactionUnit::LoggedOperationsRoot
            | TransactionUnit::TombstoneLeafContainer(_)
            | TransactionUnit::TombstoneRoot
            | TransactionUnit::TombstoneNodeBelowTheRoot(_)
            | TransactionUnit::LineageLeafContainer(_)
            | TransactionUnit::LineageRoot
            | TransactionUnit::LineageNodeBelowTheRoot(_)
            | TransactionUnit::RetirementLeafContainer(_)
            | TransactionUnit::RetirementRoot
            | TransactionUnit::RetirementNodeBelowTheRoot(_)
            | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
            | TransactionUnit::MappingTree
            | TransactionUnit::TreeTable => false,
        }
    }

    /// 这个角色是不是一个数据单元（码 1）：第一个文件的（[`TransactionUnit::Data`]），或第一个文件之外一个文件的
    /// （[`TransactionUnit::DataOfAFileOtherThanTheFirst`]）。按事务切记录、准入数段外槽对、换下清单分段读的都是这一判。
    #[must_use]
    pub const fn is_a_data_unit(self) -> bool {
        match self {
            TransactionUnit::Data(_) | TransactionUnit::DataOfAFileOtherThanTheFirst { .. } => true,
            TransactionUnit::ExtentLowerNode(_)
            | TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. }
            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
            | TransactionUnit::ExtentRoot
            | TransactionUnit::InodeLeafContainer(_)
            | TransactionUnit::InodeRoot
            | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
            | TransactionUnit::AllocationTree
            | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
            | TransactionUnit::AccountingTree
            | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
            | TransactionUnit::DeadlistTree
            | TransactionUnit::DeletedInodesRoot
            | TransactionUnit::LoggedOperationsRoot
            | TransactionUnit::TombstoneLeafContainer(_)
            | TransactionUnit::TombstoneRoot
            | TransactionUnit::TombstoneNodeBelowTheRoot(_)
            | TransactionUnit::LineageLeafContainer(_)
            | TransactionUnit::LineageRoot
            | TransactionUnit::LineageNodeBelowTheRoot(_)
            | TransactionUnit::RetirementLeafContainer(_)
            | TransactionUnit::RetirementRoot
            | TransactionUnit::RetirementNodeBelowTheRoot(_)
            | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
            | TransactionUnit::MappingTree
            | TransactionUnit::TreeTable
            | TransactionUnit::InstanceTable
            | TransactionUnit::InstanceTablePageAfterTheFirst(_) => false,
        }
    }

    #[must_use]
    pub const fn unit_class(self) -> u8 {
        match self {
            TransactionUnit::DataOfAFileOtherThanTheFirst { .. } => UNIT_CLASS_DATA,
            TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. } => {
                UNIT_CLASS_INDEX_NODE
            }
            TransactionUnit::Data(_) => UNIT_CLASS_DATA,
            TransactionUnit::InodeLeafContainer(_)
            | TransactionUnit::TombstoneLeafContainer(_)
            | TransactionUnit::LineageLeafContainer(_)
            | TransactionUnit::RetirementLeafContainer(_)
            | TransactionUnit::InstanceTable
            | TransactionUnit::InstanceTablePageAfterTheFirst(_) => UNIT_CLASS_PACKED,
            TransactionUnit::ExtentLowerNode(_)
            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
            | TransactionUnit::ExtentRoot
            | TransactionUnit::InodeRoot
            | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
            | TransactionUnit::AllocationTree
            | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
            | TransactionUnit::AccountingTree
            | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
            | TransactionUnit::DeadlistTree
            | TransactionUnit::DeletedInodesRoot
            | TransactionUnit::LoggedOperationsRoot
            | TransactionUnit::TombstoneRoot
            | TransactionUnit::TombstoneNodeBelowTheRoot(_)
            | TransactionUnit::LineageRoot
            | TransactionUnit::LineageNodeBelowTheRoot(_)
            | TransactionUnit::RetirementRoot
            | TransactionUnit::RetirementNodeBelowTheRoot(_)
            | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
            | TransactionUnit::MappingTree
            | TransactionUnit::TreeTable => UNIT_CLASS_INDEX_NODE,
        }
    }

    /// 点名项里的归属树：这一版那几棵树各自的号（`trees`，第一个文件版本那次从水位发出来、之后照抄）；
    /// 树表单元与实例表单元不属于任何一棵树（写 0）。
    #[must_use]
    pub const fn tree(self, trees: &FileVersionTreeIdentifiers) -> TreeIdentifier {
        match self {
            TransactionUnit::DataOfAFileOtherThanTheFirst { .. }
            | TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. } => trees.extent,
            TransactionUnit::Data(_)
            | TransactionUnit::ExtentLowerNode(_)
            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
            | TransactionUnit::ExtentRoot => trees.extent,
            TransactionUnit::InodeLeafContainer(_) | TransactionUnit::InodeRoot => trees.inode,
            TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
            | TransactionUnit::AllocationTree => trees.allocation_records,
            TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
            | TransactionUnit::AccountingTree => trees.accounting,
            TransactionUnit::DeadlistTreeNodeBelowTheRoot(_) | TransactionUnit::DeadlistTree => {
                trees.deadlist
            }
            TransactionUnit::DeletedInodesRoot => trees.deleted_inodes,
            TransactionUnit::TombstoneLeafContainer(_)
            | TransactionUnit::TombstoneRoot
            | TransactionUnit::TombstoneNodeBelowTheRoot(_) => trees.tombstone,
            TransactionUnit::LineageLeafContainer(_)
            | TransactionUnit::LineageRoot
            | TransactionUnit::LineageNodeBelowTheRoot(_) => trees.lineage,
            TransactionUnit::LoggedOperationsRoot => trees.logged_operations,
            TransactionUnit::RetirementLeafContainer(_)
            | TransactionUnit::RetirementRoot
            | TransactionUnit::RetirementNodeBelowTheRoot(_) => trees.retirement,
            TransactionUnit::MappingTreeNodeBelowTheRoot(_) | TransactionUnit::MappingTree => {
                trees.central_mapping
            }
            TransactionUnit::TreeTable
            | TransactionUnit::InstanceTable
            | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
                TreeIdentifier(TREE_IDENTIFIER_NONE)
            }
        }
    }

    /// 落点怎么取：用户数据按政策函数，其余是提交内生块（码 3 容器按数据单元那一档，D3（空间分配） 已定项 10 ⑤）。
    #[must_use]
    pub const fn placement(self) -> PlacementRule {
        match self {
            TransactionUnit::DataOfAFileOtherThanTheFirst { .. } => PlacementRule::UserData,
            TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. } => {
                PlacementRule::CommitGenerated(UnitFootprint::OneSlot)
            }
            TransactionUnit::Data(_) => PlacementRule::UserData,
            TransactionUnit::InodeLeafContainer(_)
            | TransactionUnit::TombstoneLeafContainer(_)
            | TransactionUnit::LineageLeafContainer(_)
            | TransactionUnit::RetirementLeafContainer(_)
            | TransactionUnit::InstanceTable
            | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
                PlacementRule::CommitGenerated(UnitFootprint::TwoSlotsAligned)
            }
            TransactionUnit::ExtentLowerNode(_)
            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
            | TransactionUnit::ExtentRoot
            | TransactionUnit::InodeRoot
            | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
            | TransactionUnit::AllocationTree
            | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
            | TransactionUnit::AccountingTree
            | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
            | TransactionUnit::DeadlistTree
            | TransactionUnit::DeletedInodesRoot
            | TransactionUnit::LoggedOperationsRoot
            | TransactionUnit::TombstoneRoot
            | TransactionUnit::TombstoneNodeBelowTheRoot(_)
            | TransactionUnit::LineageRoot
            | TransactionUnit::LineageNodeBelowTheRoot(_)
            | TransactionUnit::RetirementRoot
            | TransactionUnit::RetirementNodeBelowTheRoot(_)
            | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
            | TransactionUnit::MappingTree
            | TransactionUnit::TreeTable => PlacementRule::CommitGenerated(UnitFootprint::OneSlot),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlacementRule {
    UserData,
    CommitGenerated(UnitFootprint),
}

/// 带文件的一版那十三棵树各自的树 ID，按 D8（核心索引结构） 已定项 11 的次序：extent、inode、分配记录、记账、中央映射、
/// livelist、稀疏旁表、deadlist，再接第一个文件版本那次零根注册的 `deleted_inodes`、墓碑树、谱系树、logged_ops
/// （D18（块里携带什么信息） 已定项 24 第 6 条列的次序）与退役树（D8 已定项 9 码 13，判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 1 条）。
///
/// 第一个文件版本那次发布从它要建在上面的那一版的树 ID 水位起，按这个次序连号发这十三个号
/// （[`FileVersionTreeIdentifiers::issued_from_watermark`]）；之后每一版照抄，再不发号。
/// mkfs 那条流上那一版的水位是 mkfs 种下的 65536，发出来就是已定项 11 登记的 65536..65548；回退到树表 0 条的一版之后再发，
/// 那一版的水位带着回退之前根环里的 max（D8（核心索引结构） 已定项 8 ②），发出来的号高于此前发过的每一个——号永不重发
/// （里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 ④ 行；C511（回退到无文件那一版之后诞生代怎么接））。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileVersionTreeIdentifiers {
    pub extent: TreeIdentifier,
    pub inode: TreeIdentifier,
    pub allocation_records: TreeIdentifier,
    pub accounting: TreeIdentifier,
    pub central_mapping: TreeIdentifier,
    pub livelist: TreeIdentifier,
    pub sparse_side_table: TreeIdentifier,
    pub deadlist: TreeIdentifier,
    pub deleted_inodes: TreeIdentifier,
    pub tombstone: TreeIdentifier,
    pub lineage: TreeIdentifier,
    pub logged_operations: TreeIdentifier,
    pub retirement: TreeIdentifier,
}

/// 从水位起连号发十三个树 ID 时，水位装不下：盘上读来的 8 字节水位离 `u64::MAX` 不到十三个号。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees {
    pub tree_identifier_watermark: u64,
}

impl FileVersionTreeIdentifiers {
    /// 十三棵树在发号次序里各自离水位几个号：mkfs 水位 65536 时发出来的就是 D8（核心索引结构） 已定项 11 登记的常量
    /// （其后四棵是 D18（块里携带什么信息） 已定项 24 第 6 条那四个，最后是退役树），偏移从那几个常量现算，不另抄一份次序。
    const OFFSETS_FROM_THE_WATERMARK_IN_ISSUE_ORDER: [u64; 13] = [
        TREE_IDENTIFIER_EXTENT - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
        TREE_IDENTIFIER_INODE - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
        TREE_IDENTIFIER_ALLOCATION_RECORDS - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
        TREE_IDENTIFIER_ACCOUNTING - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
        TREE_IDENTIFIER_CENTRAL_MAPPING - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
        TREE_IDENTIFIER_LIVELIST - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
        TREE_IDENTIFIER_SPARSE_SIDE_TABLE - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
        TREE_IDENTIFIER_DEADLIST - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
        TREE_IDENTIFIER_DELETED_INODES - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
        TREE_IDENTIFIER_TOMBSTONE - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
        TREE_IDENTIFIER_LINEAGE - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
        TREE_IDENTIFIER_LOGGED_OPERATIONS - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
        TREE_IDENTIFIER_RETIREMENT - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
    ];

    /// 从 `tree_identifier_watermark`（下一个可用号）起按 D8（核心索引结构） 已定项 11 的次序连号发十三个号，
    /// 连同发完之后的下一个可用号（最大那个号 + 1）。
    ///
    /// # Errors
    /// 水位加十三个号越过 `u64::MAX`（水位是盘上读来的 8 字节）⇒ `TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees`。
    pub fn issued_from_watermark(
        tree_identifier_watermark: u64,
    ) -> Result<(Self, u64), TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees> {
        let next_available_after_this_issue = tree_identifier_watermark
            .checked_add(
                TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
            )
            .ok_or(TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees {
                tree_identifier_watermark,
            })?;
        let issued =
            Self::OFFSETS_FROM_THE_WATERMARK_IN_ISSUE_ORDER.map(|offset_from_the_watermark| {
                TreeIdentifier(tree_identifier_watermark + offset_from_the_watermark)
            });
        let [extent, inode, allocation_records, accounting, central_mapping, livelist, sparse_side_table, deadlist, deleted_inodes, tombstone, lineage, logged_operations, retirement] =
            issued;
        let trees = FileVersionTreeIdentifiers {
            extent,
            inode,
            allocation_records,
            accounting,
            central_mapping,
            livelist,
            sparse_side_table,
            deadlist,
            deleted_inodes,
            tombstone,
            lineage,
            logged_operations,
            retirement,
        };
        assert_eq!(
            trees.highest().0 + 1,
            next_available_after_this_issue,
            "十三个号连号发、最大的是退役树：发完之后的下一个可用号就是 mkfs 那条流上第一次发布之后的水位 65549 平移过来"
        );
        Ok((trees, next_available_after_this_issue))
    }

    /// 十三个号按发号次序。
    #[must_use]
    pub const fn in_issue_order(&self) -> [TreeIdentifier; 13] {
        [
            self.extent,
            self.inode,
            self.allocation_records,
            self.accounting,
            self.central_mapping,
            self.livelist,
            self.sparse_side_table,
            self.deadlist,
            self.deleted_inodes,
            self.tombstone,
            self.lineage,
            self.logged_operations,
            self.retirement,
        ]
    }

    /// 这十三个号里最大的那一个。
    #[must_use]
    pub fn highest(&self) -> TreeIdentifier {
        self.in_issue_order()
            .into_iter()
            .max()
            .expect("十三个号，数组非空")
    }
}

/// 一个写出去的单元：落点、身份、字节。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishedUnit {
    pub slot: SlotNumber,
    pub identity: TransactionUnit,
    /// 字节不可变、按引用计数共享：没重写的角色从上一版照抄（`carried_unit`）、版本整份拷贝时只多一个引用，不再复制这份字节
    /// （判决 `research/prompts/m4-gpugc-publish-growth-r1-main-verification.md` 第二节 Q2.1 甲档）。相等照字节比，与共不共享无关。
    pub bytes: Arc<[u8]>,
}

/// 一次发布写出的一条 journal 记录与它的 4096 字节。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WrittenJournalRecord {
    pub record: JournalRecord,
    pub bytes: Vec<u8>,
}

#[cfg(test)]
mod tests;
