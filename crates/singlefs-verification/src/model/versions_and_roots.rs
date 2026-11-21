//! 一版的内容与一条根：第一个文件、别的 inode、截断字段、实例表的行、extent 上段的位置与用户可见状态。

use super::*;

/// 第一个文件那条 inode 记录里的截断字段（判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R1、R2、R5，第六节）：
/// 记录偏移 120 起两段 8 字节，T（截断字段的 txg）与 N_w（截断字段的 size）。截断意图那次，这次放单元之前存在的单元数多于 ⌈N ÷ 净荷⌉.max(1)
/// 时写成（这次 txg, N），否则照抄上一版（[`ModelTruncationField::after_a_truncation_intent`]）；各批、最后一批、变长与 N = S、unlink 的意图与各批
/// 照抄；整份写（顺序写、覆盖写）清零；回退随回退目标那一版（R5，`contents` 整个取回退目标的）。模型按条款自己写一份，不引实现的算法
/// （D13（验证路线） 已定项 5）。封闭集合：没写过的两段都是 0（R1「T = 0 ⇒ N_w = 0」），写过的 T 是那次截断意图的 txg、恒不为 0。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelTruncationField {
    /// 没写过，或整份写清过：T = 0、N_w = 0。
    NotWritten,
    /// 一次放了单元的截断意图写的：T 是那次意图的 txg，N_w 是那次截到的 size。
    WrittenByATruncationIntent {
        truncation_txg: ModelCheckpointTxg,
        size_in_bytes: u64,
    },
}

impl ModelTruncationField {
    /// 截断意图那次之后的字段（「放了单元才写、整个替换」）：`data_units_present_before` 是这次放单元之前存在的单元数
    /// （截断意图接在带活文件的一版后面，那一版存在的单元数），多于截后 size 罩的 ⌈N ÷ 净荷⌉.max(1) 个时写成（这次 txg, N），否则照抄 `previous`
    /// ——截到洞里不放单元、截到 0 而只有一个单元的都照抄。
    #[must_use]
    pub fn after_a_truncation_intent(
        previous: ModelTruncationField,
        data_units_present_before: u64,
        new_size_in_bytes: u64,
        intent_txg: ModelCheckpointTxg,
    ) -> ModelTruncationField {
        if data_units_present_before > data_units_of_content(new_size_in_bytes) {
            ModelTruncationField::WrittenByATruncationIntent {
                truncation_txg: intent_txg,
                size_in_bytes: new_size_in_bytes,
            }
        } else {
            previous
        }
    }

    /// 盘上那两段的值（R1：没写过的两段都是 0）。
    #[must_use]
    pub fn recorded_values(self) -> ObservedTruncationField {
        match self {
            ModelTruncationField::NotWritten => ObservedTruncationField {
                truncation_txg: ModelCheckpointTxg(0),
                size_in_bytes: 0,
            },
            ModelTruncationField::WrittenByATruncationIntent {
                truncation_txg,
                size_in_bytes,
            } => ObservedTruncationField {
                truncation_txg,
                size_in_bytes,
            },
        }
    }
}

/// 实现写出的第一个文件那条 inode 记录里截断字段那两段的原样值（R1：偏移 120 是 T、偏移 128 是 N_w，各 8 字节小端），胶水从记录的字节里取，
/// 不判合法性（T = 0 而 N_w ≠ 0 也原样交回，模型判对不上）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObservedTruncationField {
    pub truncation_txg: ModelCheckpointTxg,
    pub size_in_bytes: u64,
}

/// 一版的文件：最近写它的 inode 树、extent 树根指针的那次发布的身份与内容。树表里这两棵树的根指针由写出它们的那次发布唯一定（指针带诞生代号），
/// 所以「两条根的这两棵树的根指针相同」⟺「两条根的 `written_by` 相同」（D16（发布语义） 已定项 1「非空」从盘上怎么认）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelFileVersion {
    /// 最近一次写这一版 inode 树或 extent 树根指针的那次发布：写内容的、截断意图、截断的批（只写 extent 树，判决
    /// `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K2「批不碰 inode 记录」）、放了单元的最后一批、不缩短的截断。
    pub written_by: ModelRootKey,
    /// 最近一次写 inode 1 那条记录（inode 树的根）的那次发布：截断的批与最后一批不写它（K2），这时与 `written_by` 不同；
    /// 建快照抄的码 14 根指针是它写的（[`ModelVersionContents::inode_tree_written_by`]）。
    pub inode_record_written_by: ModelRootKey,
    /// 读回的整份字节：size 就是它的长度，洞读 0（K5「读者读洞答 0、不读盘」）。截断途中是目标那 N 字节（K2「先落目标」）。
    pub content: Rc<[u8]>,
    /// 盘上存在的数据单元数：从单元 0 起连号；不多于 ⌈size ÷ 净荷⌉.max(1)，少于它时尾部是洞（K5）；截断途中 size 之外还挂着
    /// 没放的单元时多于它（K2）。
    pub data_units_present: u64,
    /// 上段叶条目现在的标签（K5）：最近一次重写 extent 上段根时按那一刻的单元数与 size 写的（[`ModelExtentUpperLeafEntryLabel::written_for`]），
    /// 之后不重写上段根的发布照抄。
    pub extent_upper_leaf_entry_label: ModelExtentUpperLeafEntryLabel,
    /// 这一版第一个文件那条 inode 记录里的截断字段（[`ModelTruncationField`]）。
    pub truncation_field: ModelTruncationField,
}

impl ModelFileVersion {
    /// size 罩几个数据单元（⌈size ÷ 净荷⌉.max(1)，[`data_units_of_content`]）。
    #[must_use]
    pub fn data_units_within_the_size(&self) -> u64 {
        data_units_of_content(u64::try_from(self.content.len()).expect("内容长度"))
    }

    /// 存在的单元 `index` 的声明长度：min(size − 起点, 净荷)。截断途中 size 之外还挂着的单元不问（读者不读它们，K2）；带洞的文件存在的单元都是满的
    /// （变长把最后一个存在的单元补零到 min(N − 起点, 净荷)，K5），这个式子同样对。
    pub(super) fn declared_length_of_the_data_unit(&self, index: u64) -> u64 {
        let payload = data_unit_payload_capacity_in_bytes();
        u64::try_from(self.content.len())
            .expect("内容长度")
            .saturating_sub(index * payload)
            .min(payload)
    }
}

/// 第一个文件之外的一个 inode 在一版里的样子（[`ModelRoot::inodes_other_than_the_first`]）：建 inode 那次建出来（size 0、没有数据单元、
/// extent 树上段里没有它的条目），之后经按 inode 号写数据的两个入口从偏移 0 整份写（里程碑「删除与多次 COW」一·1
/// 「先给 core 加多 inode 写数据」：每次整份重写全部数据单元、同一个分配代，长度 0 的内容也写一个声明长度 0 的单元），也经改 size 缩短与变长
/// （判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S3、S5射程「任意活 inode 都能缩短与变长」：
/// 与第一个文件同一套，尾单元重写、尾部的洞、截断途中 size 之外挂着的单元都照 K2、K3、K5）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelInodeOtherThanTheFirst {
    /// 读回的整份字节：size 就是它的长度，洞读 0（K5）。没写过数据、没改过 size 的是空的；截断途中是目标那 N 字节（K2「先落目标」）。
    pub content: Rc<[u8]>,
    /// 它每个存在的数据单元是哪次发布写的，按单元号（第 i 个是单元 i）：整份写那次全是那一次的 txg，改 size 重写尾单元之后那一个是那一次的
    /// （K4 推到任意 inode：换下时各按各的判快照生死），截断的批与 unlink 的批从尾部去掉。没写过数据的是空的。
    pub data_unit_written_at: Rc<Vec<ModelCheckpointTxg>>,
    /// 它的 extent 树下段每个节点的分配代（换下时按它判钉不钉，判决 m4-snapshot-r1 S3、m4-trunc-r1 K4）：整份写那次整段重写，改 size 与各批只重写罩着
    /// 重写的或放掉的单元的那几个；只剩一个单元时没有下段（同第一个文件的 [`ModelRoot::extent_lower_node_written_at`]）。
    pub extent_lower_node_written_at:
        Rc<BTreeMap<ModelExtentLowerNodePosition, ModelCheckpointTxg>>,
    /// 它那条记录里的截断字段（判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R1、R2 推到任意 inode）：建出来与整份写之后没写过，
    /// 截断意图那次按放不放单元写或照抄（[`ModelTruncationField::after_a_truncation_intent`]），别的照抄；逐版逐个比（判决 m4-multi-inode-size-r1
    /// 第二节 S5「截断字段按 inode 比」）。
    pub truncation_field: ModelTruncationField,
    /// 它那条记录偏移 8 的对象出生代：建它那次发布的 txg（D8（核心索引结构） 已定项 6；按 inode 号整份写不改它）。
    /// K-U5 的 β 时序级读法按它判（[`ModelRefusalReason::DataWriteToAnInodeBornAfterTheFirstFileWasDeleted`]）。
    pub object_birth: ModelCheckpointTxg,
    /// 活着还是删除中（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M1：删任一号）。
    pub liveness: ModelInodeLiveness,
    /// 上段里有它那一条标签 0（一个单元都没有）：改 size 到 size > 0 那一次写上、之后缩短、缩到 0、再变长都留着，到 unlink 最后一批才去掉
    /// （粘住的标签 0，判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S2）。建出来没改过 size 的是 `false`。
    pub keeps_a_tag_zero_upper_leaf_entry: bool,
    /// 这一版 logged_ops 里点名它的那一条截断条目的签发 txg（截断意图那次写进、最后一批删掉，判决 m4-trunc-r1 K1、K2 推到任意活 inode）；
    /// 不在截断途中为 `None`。
    pub truncation_issued_txg: Option<ModelCheckpointTxg>,
}

/// 第一个文件之外的一个 inode 活着还是删除中（D8（核心索引结构） 已定项 16 第 1、2 条：意图那次 nlink 写 0、号进 `deleted_inodes`，
/// 不是最后一批的批每批从尾部放 144 个单元，最后一批删掉记录）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelInodeLiveness {
    Live,
    /// unlink 的意图那次之后、最后一批之前：记录 nlink = 0（读不出），还剩 `data_units_remaining` 个数据单元（单元号 0 起连着，
    /// 分配代仍按 [`ModelInodeOtherThanTheFirst::data_unit_written_at`] 记，个数与它相同）。
    BeingDeleted {
        data_units_remaining: u64,
    },
}

impl ModelInodeOtherThanTheFirst {
    /// 盘上存在的数据单元数（按单元号记的分配代有几个）：整份写过的至少一个，没写过的 0 个；带洞的少于 size 罩的、截断途中多于它（K2、K5）；
    /// 删除中的是还剩的。
    ///
    /// # Panics
    /// 删除中而还剩的单元数与按单元号记的个数不同（unlink 的批两样一起减）。
    #[must_use]
    pub fn data_units_present(&self) -> u64 {
        let data_units_present = u64::try_from(self.data_unit_written_at.len()).expect("单元数");
        match self.liveness {
            ModelInodeLiveness::Live => {}
            ModelInodeLiveness::BeingDeleted {
                data_units_remaining,
            } => assert_eq!(
                data_units_remaining, data_units_present,
                "删除中的那个号还剩的单元数与按单元号记的个数相同（unlink 的批两样一起减）"
            ),
        }
        data_units_present
    }

    /// size 罩几个数据单元（⌈size ÷ 净荷⌉.max(1)，[`data_units_of_content`]）。
    #[must_use]
    pub fn data_units_within_the_size(&self) -> u64 {
        data_units_of_content(u64::try_from(self.content.len()).expect("内容长度"))
    }

    /// 存在的单元 `index` 的声明长度：min(size − 起点, 净荷)（同 [`ModelFileVersion`] 那一份：带洞的文件存在的单元都是满的，截断途中 size 之外
    /// 还挂着的不问）。
    pub(super) fn declared_length_of_the_data_unit(&self, index: u64) -> u64 {
        let payload = data_unit_payload_capacity_in_bytes();
        u64::try_from(self.content.len())
            .expect("内容长度")
            .saturating_sub(index * payload)
            .min(payload)
    }

    /// 上段里有它的条目：有数据单元，或粘住的标签 0（[`ModelInodeOtherThanTheFirst::keeps_a_tag_zero_upper_leaf_entry`]）。
    #[must_use]
    pub fn has_an_extent_upper_leaf_entry(&self) -> bool {
        self.data_units_present() > 0 || self.keeps_a_tag_zero_upper_leaf_entry
    }

    /// 活着（nlink 不是 0）：读得出、按 inode 号写与 unlink 点得到它。
    #[must_use]
    pub fn is_live(&self) -> bool {
        match self.liveness {
            ModelInodeLiveness::Live => true,
            ModelInodeLiveness::BeingDeleted { .. } => false,
        }
    }
}

/// extent 树上段一个节点的位置（D8（核心索引结构） 已定项 14：按 inode 号按位置寻址、不分裂，叶罩 143 个号、内部扇出 147；层级 0 是叶，
/// 同层从 inode 0 起数第几个）。模型自己一份，不引实现的类型（D13（验证路线） 已定项 5）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelExtentUpperNodePosition {
    pub level: u64,
    pub index_in_level: u64,
}

/// 上段第 `level` 层一个节点罩几个 inode 号：143 × 147^level（饱和在 u64 的上界）。
fn extent_upper_span_in_inodes(level: u64) -> u64 {
    (0..level).fold(EXTENT_TREE_UPPER_LEAF_INODES, |span, _| {
        span.saturating_mul(EXTENT_TREE_INTERNAL_FANOUT)
    })
}

impl ModelExtentUpperNodePosition {
    /// 这个位置罩不罩 inode 号 `inode`。
    fn covers(self, inode: u64) -> bool {
        inode / extent_upper_span_in_inodes(self.level) == self.index_in_level
    }
}

/// 上段的规范形（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M2）：根层级是罩得住有条目最大号的最低一层
/// （[`extent_upper_segment_root_level`]），节点是每个有条目的号从它那片叶到根那一串的并；集合空时只有层级 0 第 0 格那个零条目根兼叶。
/// 模型按条款自己算一份，不调实现的计划（D13（验证路线） 已定项 5）。
#[must_use]
pub fn extent_upper_segment_positions(
    inodes_with_an_entry: &BTreeSet<u64>,
) -> BTreeSet<ModelExtentUpperNodePosition> {
    let root_level = extent_upper_segment_root_level(inodes_with_an_entry);
    if inodes_with_an_entry.is_empty() {
        return BTreeSet::from([ModelExtentUpperNodePosition {
            level: 0,
            index_in_level: 0,
        }]);
    }
    inodes_with_an_entry
        .iter()
        .flat_map(|inode| {
            (0..=root_level).map(move |level| ModelExtentUpperNodePosition {
                level,
                index_in_level: inode / extent_upper_span_in_inodes(level),
            })
        })
        .collect()
}

/// 一次发布之后上段每个节点的分配代（规范形计划「一次发布里进出 S 或条目变了的号那一串上留在新形状里的节点与新出现的节点重写，
/// 别的照抄」）：新形状里的一个位置，上一版没有它、或它罩着 `inodes_whose_entry_changed` 里的一个号，分配代取这次的 `checkpoint_txg`；
/// 别的照抄上一版的分配代。长高那次旧根就这样照抄成根之下的节点，降层那次上一版里降后层级的第 0 个节点照抄成根。
#[must_use]
pub fn extent_upper_node_written_at_after(
    previous: &BTreeMap<ModelExtentUpperNodePosition, ModelCheckpointTxg>,
    inodes_with_an_entry_after: &BTreeSet<u64>,
    inodes_whose_entry_changed: &BTreeSet<u64>,
    checkpoint_txg: ModelCheckpointTxg,
) -> BTreeMap<ModelExtentUpperNodePosition, ModelCheckpointTxg> {
    extent_upper_segment_positions(inodes_with_an_entry_after)
        .into_iter()
        .map(|position| {
            let covers_a_changed_inode = inodes_whose_entry_changed
                .iter()
                .any(|inode| position.covers(*inode));
            let written_at = match previous.get(&position) {
                Some(previous_written_at) if !covers_a_changed_inode => *previous_written_at,
                Some(_) | None => checkpoint_txg,
            };
            (position, written_at)
        })
        .collect()
}

/// extent 树上段的根在第几层（D8（核心索引结构） 已定项 14：叶罩 143 个 inode 号、内部扇出 147，按位置寻址、不分裂；层级是罩得住有条目的
/// 最大 inode 号的最低一层，只有叶兼根是 0 层，一条条目都没有的也是 0 层）：按 inode 号写长高、删除降层那两形看它变没变。
/// `inodes_with_an_entry`：这一版有数据单元的文件的 inode 号（第一个文件有单元时连 inode 1）。
#[must_use]
pub fn extent_upper_segment_root_level(inodes_with_an_entry: &BTreeSet<u64>) -> u64 {
    let Some(largest_inode_with_an_entry) = inodes_with_an_entry.last().copied() else {
        return 0;
    };
    let mut level = 0;
    let mut span_in_inodes = EXTENT_TREE_UPPER_LEAF_INODES;
    // 迭代上界：每一层的罩宽乘 147，罩宽超过最大的号时停（u64 里至多十来层）；跨轮带的是这一层的罩宽与层级。
    while span_in_inodes <= largest_inode_with_an_entry {
        level += 1;
        span_in_inodes = span_in_inodes.saturating_mul(EXTENT_TREE_INTERNAL_FANOUT);
    }
    level
}

/// 一版里有什么（D8（核心索引结构） 已定项 16 第 4、5 条：树表 0 条的、带文件的、有树而文件集合为空的；第一个文件之外的 inode 另记在
/// [`ModelRoot::inodes_other_than_the_first`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelVersionContents {
    /// 树表 0 条（第 0 代根、暖机根、零单元发布）：没有树、没有文件。
    WithoutTrees,
    /// inode 1 是活记录：读回它的内容。
    LiveFile(ModelFileVersion),
    /// unlink 的意图那次之后、最后一批之前：inode 1 的记录 nlink = 0、号在 `deleted_inodes` 里，读回「没有文件」，它的单元还在。
    /// `trees_written_by`：写出这一版 inode 树、extent 树根指针的那次发布（意图那次或上一个不是最后一批的删除批；回退到这一版的根照抄它）。
    /// `data_units_remaining`：这一版里这个文件还剩几个数据单元（意图那次 = 文件的单元数，每个不是最后一批的删除批少 144 个）：
    /// 下一步是不是最后一批由它定（D8（核心索引结构） 已定项 16 第 2 条）。
    /// `truncation_field`：删除中那条记录里的截断字段，unlink 的意图与各批照抄 unlink 之前那一版的（判决
    /// `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R2）。
    FileBeingDeleted {
        trees_written_by: ModelRootKey,
        data_units_remaining: u64,
        truncation_field: ModelTruncationField,
    },
    /// 最后一批之后：有树而文件集合为空（`deleted_inodes` 空、inode 树与 extent 树零条目根），读回「没有文件」。
    TreesAndNoFile { trees_written_by: ModelRootKey },
    /// 第一个文件已删完而 inode 树里还有别的记录（删完之后建过 inode，或删第一个文件时树里还有别的 inode，判决
    /// `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M3）：没有 inode 1，读回「没有文件」；别的文件记在
    /// [`ModelRoot::inodes_other_than_the_first`]（可以带数据、可以删除中）。实现把它归带文件的一版（inode 树有条目，D8（核心索引结构） 已定项 16
    /// 第 5 条的判定不成立）。
    /// `trees_written_by`：最近一次写这一版 inode 树或 extent 树根指针的那次发布（回退到这一版的根照抄它）。
    /// `inode_tree_written_by`：最近一次写 inode 树的那次发布：别的 inode 截断的批与放了单元的最后一批只写 extent 树（K2「批不碰 inode 记录」，判决
    /// m4-multi-inode-size-r1 第二节 S3），这时与 `trees_written_by` 不同；建快照抄的码 14 根指针是它写的。
    /// `inode_records_in_the_tree`：这一版 inode 树里有几条记录（就是第一个文件之外的 inode 数）：下一个建的落在哪一片叶容器由它定
    /// （[`inode_leaf_container_position_of_a_new_record`]）。
    InodesWithoutTheFirstFile {
        trees_written_by: ModelRootKey,
        inode_tree_written_by: ModelRootKey,
        inode_records_in_the_tree: u64,
    },
    /// 截断的意图那次之后、最后一批之前（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K2「先落目标」）：inode 1 的记录 size 已是
    /// 目标 N，读回 N 字节（`file.content`）；logged_ops 里有一条截断条目（签发 txg 是 `truncation_issued_txg`，目标 size 是 `file.content` 的长度）；
    /// size 之外还挂着 `file.data_units_present` − ⌈N ÷ 净荷⌉.max(1) 个没放的单元，下一步是不是最后一批由它定。
    FileBeingTruncated {
        file: ModelFileVersion,
        truncation_issued_txg: ModelCheckpointTxg,
    },
}

impl ModelVersionContents {
    /// 树表不是 0 条。
    #[must_use]
    pub fn has_trees(&self) -> bool {
        match self {
            ModelVersionContents::WithoutTrees => false,
            ModelVersionContents::LiveFile(_)
            | ModelVersionContents::FileBeingDeleted { .. }
            | ModelVersionContents::TreesAndNoFile { .. }
            | ModelVersionContents::InodesWithoutTheFirstFile { .. }
            | ModelVersionContents::FileBeingTruncated { .. } => true,
        }
    }

    /// 读得出的那个文件（inode 1 是活记录时；截断途中读回目标那 N 字节，K2）。
    #[must_use]
    pub fn live_file(&self) -> Option<&ModelFileVersion> {
        match self {
            ModelVersionContents::LiveFile(file)
            | ModelVersionContents::FileBeingTruncated { file, .. } => Some(file),
            ModelVersionContents::WithoutTrees
            | ModelVersionContents::FileBeingDeleted { .. }
            | ModelVersionContents::TreesAndNoFile { .. }
            | ModelVersionContents::InodesWithoutTheFirstFile { .. } => None,
        }
    }

    /// 这一版盘上存在的数据单元数：带文件的与截断途中的取文件记的（截断途中连 size 之外还挂着的），删除中的取还剩的，别的没有数据单元。
    #[must_use]
    pub fn data_units_present(&self) -> u64 {
        match self {
            ModelVersionContents::LiveFile(file)
            | ModelVersionContents::FileBeingTruncated { file, .. } => file.data_units_present,
            ModelVersionContents::FileBeingDeleted {
                data_units_remaining,
                ..
            } => *data_units_remaining,
            ModelVersionContents::WithoutTrees
            | ModelVersionContents::TreesAndNoFile { .. }
            | ModelVersionContents::InodesWithoutTheFirstFile { .. } => 0,
        }
    }

    /// 这一版的用户可见树（inode 树、extent 树）的根指针是谁写的：树表 0 条时 None。树表里这两棵树的根指针由写出它们的那次发布唯一定
    /// （指针带诞生代号），「两条根的这两棵树的根指针相同」⟺「是同一次发布写的」（D16（发布语义） 已定项 1「非空」从盘上怎么认）。
    fn user_visible_trees_written_by(&self) -> Option<ModelRootKey> {
        match self {
            ModelVersionContents::WithoutTrees => None,
            ModelVersionContents::LiveFile(file)
            | ModelVersionContents::FileBeingTruncated { file, .. } => Some(file.written_by),
            ModelVersionContents::FileBeingDeleted {
                trees_written_by, ..
            }
            | ModelVersionContents::TreesAndNoFile { trees_written_by }
            | ModelVersionContents::InodesWithoutTheFirstFile {
                trees_written_by, ..
            } => Some(*trees_written_by),
        }
    }

    /// 这一版 inode 树的根是谁写的（建快照抄进码 14 的那条根指针）：树表 0 条时 None。截断的批与最后一批只写 extent 树（K2），
    /// 这时与 [`ModelVersionContents::user_visible_trees_written_by`] 不同；删除的各步与建 inode 都写 inode 树，两者相同。
    pub(super) fn inode_tree_written_by(&self) -> Option<ModelRootKey> {
        match self {
            ModelVersionContents::LiveFile(file)
            | ModelVersionContents::FileBeingTruncated { file, .. } => {
                Some(file.inode_record_written_by)
            }
            ModelVersionContents::InodesWithoutTheFirstFile {
                inode_tree_written_by,
                ..
            } => Some(*inode_tree_written_by),
            ModelVersionContents::WithoutTrees
            | ModelVersionContents::FileBeingDeleted { .. }
            | ModelVersionContents::TreesAndNoFile { .. } => self.user_visible_trees_written_by(),
        }
    }

    /// 这一版第一个文件那条 inode 记录里的截断字段：带活文件的、截断途中的与删除中的那几版有那条记录；树表 0 条的、删完的与删完之后建过 inode 的
    /// 没有那条记录，None。
    #[must_use]
    pub fn first_file_record_truncation_field(&self) -> Option<ModelTruncationField> {
        match self {
            ModelVersionContents::LiveFile(file)
            | ModelVersionContents::FileBeingTruncated { file, .. } => Some(file.truncation_field),
            ModelVersionContents::FileBeingDeleted {
                truncation_field, ..
            } => Some(*truncation_field),
            ModelVersionContents::WithoutTrees
            | ModelVersionContents::TreesAndNoFile { .. }
            | ModelVersionContents::InodesWithoutTheFirstFile { .. } => None,
        }
    }

    /// 截断途中这一版 logged_ops 里那条截断条目的签发 txg（K1：key = 种类 2 + 签发 txg）；不在截断途中为 None。
    #[must_use]
    pub fn logged_truncation_issued_txg(&self) -> Option<ModelCheckpointTxg> {
        match self {
            ModelVersionContents::FileBeingTruncated {
                truncation_issued_txg,
                ..
            } => Some(*truncation_issued_txg),
            ModelVersionContents::WithoutTrees
            | ModelVersionContents::LiveFile(_)
            | ModelVersionContents::FileBeingDeleted { .. }
            | ModelVersionContents::TreesAndNoFile { .. }
            | ModelVersionContents::InodesWithoutTheFirstFile { .. } => None,
        }
    }

    /// 删完第一个文件之后建的 inode 在这一版 inode 树里有几条：有树而文件集合为空的一版 0 条；
    /// 别的几种不是第一个文件已删完的一版，None（建 inode 只接在这两种后面）。
    pub(super) fn inode_records_created_after_the_first_file_was_deleted(&self) -> Option<u64> {
        match self {
            ModelVersionContents::TreesAndNoFile { .. } => Some(0),
            ModelVersionContents::InodesWithoutTheFirstFile {
                inode_records_in_the_tree,
                ..
            } => Some(*inode_records_in_the_tree),
            ModelVersionContents::WithoutTrees
            | ModelVersionContents::LiveFile(_)
            | ModelVersionContents::FileBeingDeleted { .. }
            | ModelVersionContents::FileBeingTruncated { .. } => None,
        }
    }

    /// 有树、文件集合不空的那一版（带活文件的、删除中的、截断途中的、删完之后建过 inode 的）：可写会话里覆盖写、回退、抬 F 的执行器只在实现报「带文件」时调，
    /// 实现把删除中的那一版（`deleted_inodes` 不空）与只有 size 0 的 inode 的那一版（inode 树有条目）也归带文件。
    pub(super) fn is_a_version_with_a_file_or_a_file_being_deleted(&self) -> bool {
        match self {
            ModelVersionContents::LiveFile(_)
            | ModelVersionContents::FileBeingDeleted { .. }
            | ModelVersionContents::InodesWithoutTheFirstFile { .. }
            | ModelVersionContents::FileBeingTruncated { .. } => true,
            ModelVersionContents::WithoutTrees | ModelVersionContents::TreesAndNoFile { .. } => {
                false
            }
        }
    }
}

/// 实例表的一行 (i, T, W)（D18（块里携带什么信息） 已定项 11）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelInstanceRow {
    pub instance: ModelInstanceGeneration,
    pub selected_root_txg: ModelCheckpointTxg,
    pub applied_transaction_high_water: u64,
}

/// 一条已提交的根与它指着的那一版。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelRoot {
    pub key: ModelRootKey,
    pub journal_counter: ModelJournalCounter,
    pub rollback_floor: ModelCheckpointTxg,
    /// 这一版里有什么：树表 0 条、带文件、删除中、有树而文件集合为空。
    pub contents: ModelVersionContents,
    pub instance_table_rows: Rc<Vec<ModelInstanceRow>>,
    /// 这一版每个角色的单元是哪次发布写的：它的分配记录的分配代就是这个 txg（D3（空间分配） 已定项 3 / 7：value = 分配代；
    /// COW：单元只在分配它的那次发布里写）。mkfs 写的实例表与第 0 版树表记 txg 0。数据单元不在这里，按单元号记在 `data_unit_written_at`。
    pub role_written_at: BTreeMap<ModelUnitRole, ModelCheckpointTxg>,
    /// 这一版每个存在的数据单元是哪次发布写的，按单元号（第 i 个是单元 i，长度 = [`ModelVersionContents::data_units_present`]）：
    /// 分配代与映射 key 里的出生 txg 都是它（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K4：截断只重写尾单元之后
    /// 一个文件的数据单元不再同一个出生 txg，换下时各按各的判快照生死）。
    pub data_unit_written_at: Rc<Vec<ModelCheckpointTxg>>,
    /// 每盘已占槽数的上界：沿来路每次发布写出的单元的槽数之和，释放与回收一概不扣（分配记录树每次重写的节点数取按位置寻址现算的上界）。
    pub occupied_slots_upper_bound_per_device: u64,
    /// 这一版中央映射树节点数的上界（[`central_mapping_tree_nodes_upper_bound`] 沿来路带下来，每盘一个节点占一个槽）；
    /// 树表 0 条的一版没有这棵树，记 0。
    pub central_mapping_tree_nodes_upper_bound: u64,
    /// 这一版记账里 inode 号水位那一统计量的台阶行有几行（D5（快照 / 空间记账机制） 已定项 2 射程「inode 号水位的台阶行」）：
    /// 水位变了的每次发布留一级，之后每次写记账行的发布照抄（管理员回退那次照抄现行那一版的，不取回退目标的）。
    /// 模型不记水位的值，按发布的种类认水位变没变（[`ModelPublishKind::leaves_an_inode_number_watermark_step`]）。
    /// 没写过记账树的一版记 0。
    pub inode_number_watermark_steps: u64,
    /// 这一版记账树的行数：写它的那次发布写的池级三行、每块盘六行与照抄的台阶行（[`IdealModel::accounting_rows_of_a_publish_on`]）；
    /// 没写过记账树的一版记 0。
    pub accounting_rows: u64,
    /// 这一版记账树节点数的上界（[`accounting_tree_nodes_upper_bound`] 沿来路带下来，每盘一个节点占一个槽）：台阶行多到一片叶装不下时
    /// 记账树分裂、长出根之下的节点（D8（核心索引结构） 已定项 11）。根之下的节点不对应角色，模型不记 key、不记是哪几个（同中央映射树）；
    /// 写记账树的发布按它带占槽、按它数进映射条目。没写过记账树的一版记 0。
    pub accounting_tree_nodes_upper_bound: u64,
    /// 写出这条根的那次发布的记录里最大的事务号：写文件内容的发布（第一个文件、覆盖写、顺序写）一事务一单元（D16（发布语义） 已定项 5）
    /// ⇒ 内容切成几个数据单元就承载几个事务、接着实例内计数往下取，这里记最后那一个号；删除类发布与建 inode 各一个事务；
    /// 空发布、写行、暖机、抬 F、回退的记录事务号 0（D23（journal 的角色与格式） 已定项 19 ①），这里记 0。
    /// 恢复施加这次发布的记录时，W 按它取（D23（journal 的角色与格式） 已定项 14 第 4 条；空发布的 0 不进 max）。
    pub highest_transaction_number_of_its_publish: u64,
    /// 这个实例到这条根为止发过的最大事务号：事务号按实例计数、从 1 起（D23（journal 的角色与格式） 已定项 7），
    /// 换了实例（写行那次发布）从 0 重新数。
    pub highest_transaction_number_in_its_instance: u64,
    /// 这一版树表里的只读快照，按（诞生 txg, 树号）升序——就是建的先后（判决 `research/prompts/m4-snapshot-r1-main-verification.md`
    /// 第三节 S1、S2：每次建快照是一次独立发布，txg 各加一）。树表 0 条的一版一个都没有。
    pub snapshots: Rc<Vec<ModelSnapshot>>,
    /// 活头 inode 树那条条目的 `previous_snapshot_txg`（D5（快照 / 空间记账机制） 已定项 9：最近一个快照的 S.txg，还没打过快照为 0；
    /// 删最后一个快照的意图那次换成被删那一个的前驱，判决 S4）。树表 0 条的一版没有那条条目，记 0、不比。
    pub previous_snapshot_txg_of_the_live_head: ModelCheckpointTxg,
    /// 这一版的 deadlist 条目（判决 S3），次序不论：换下的用户可见单元出生 txg 不晚于活头前驱的进这里、不释放。
    pub deadlist: Rc<Vec<ModelDeadlistEntry>>,
    /// 这一版留在中央映射树里、被活快照钉住的映射条目条数（判决 S3 的 α）：每钉住一个单元留一条，删快照的批放掉一条删一条。
    /// 与 deadlist 条目数各按各的来路算（条款说两者一一对应，模型两样都比）。
    pub pinned_mapping_entries: u64,
    /// 这一版 deadlist 树节点数的上界（[`deadlist_tree_nodes_upper_bound`] 沿来路带下来；零根时 0）。
    pub deadlist_tree_nodes_upper_bound: u64,
    /// 这一版第一个文件 extent 树下段每个节点的分配代（换下时按它判钉不钉，判决 S3）：写内容那次整段重写，不是最后一批的删除批只重写
    /// 罩着被截掉那一段的节点（K-R3-1）、整个落在留下那一段里的照抄，最后一批整段换下。模型罩的版本里只有第一个文件有下段。
    pub extent_lower_node_written_at:
        Rc<BTreeMap<ModelExtentLowerNodePosition, ModelCheckpointTxg>>,
    /// 这一版 logged_ops 里那一条活整理意图（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1：全池至多一条）；没有为 `None`。
    /// 签发那次写进、之后每次发布照抄（管理员回退随现行那一版，丁 3 交回的读法第 9 条）、删意图那次没了；崩溃恢复与可写挂载落到哪一版就是那一版的。
    pub logged_compaction_intent: Option<ModelCompactionIntent>,
    /// 这一版引用着的用户可见树的根单元（[`ModelRoot::user_visible_root_units`]）里被整理搬过的，最后一次搬它的那次发布的 txg：树表里指着它的
    /// 根指针从那次起写的是映射位置，盘上字节与搬之前不同（「照字节比，搬根算非空」）。只记这一版还引用着的单元。
    pub user_visible_root_relocations: Rc<BTreeMap<ModelUserVisibleRootUnit, ModelCheckpointTxg>>,
    /// 这一版 inode 树里第一个文件之外的每个活着的 inode，按 inode 号（建 inode 那次建、按 inode 号写数据那次整份写，
    /// [`ModelInodeOtherThanTheFirst`]）：随用户可见树走——别的发布照抄，管理员回退取回退目标那一版的。树表 0 条的一版是空的。
    pub inodes_other_than_the_first: Rc<BTreeMap<u64, ModelInodeOtherThanTheFirst>>,
    /// 这一版的 inode 号水位：下一个建的 inode 取的号（D5（快照 / 空间记账机制） 已定项 8 池级三行之一）。没写过记账树的一版 0；第一个文件那次
    /// 发掉 inode 1、水位 2；建 inode 每建一个加一；管理员回退那次取现行那一版的（[`ModelPublishKind::leaves_an_inode_number_watermark_step`]
    /// 那一段：环里没被崩溃恢复抛弃的根的最大值，模型罩的历史里就是现行那一版的），不取回退目标的；别的发布照抄。
    pub inode_number_watermark: u64,
    /// 这一版父链上 inode 1 那条整对象墓碑的死亡 txg（D18（块里携带什么信息） 已定项 24：死亡 txg = 意图那次发布的 txg）：unlink 第一个文件的
    /// 意图那次写成这次的 txg，别的发布照抄；管理员回退那次取回退目标那一版的——回退那次的父是回退目标（谱系记录读作「T_rb 的父是 T_old」），
    /// 现行那一版在回退目标之后删第一个文件留下的墓碑虽然随现行那一版留在墓碑树里，却在被回退抛下的那一段上，不管这一版（代码轮判决
    /// `research/prompts/m3-close-w5-code-r1-main-verification.md` 第三节第 1 条：β 只认这一版父链上的墓碑）。模型不记墓碑树与谱系树的条目，
    /// 照「一条父链上第一个文件至多删一次」直接带着这一条走。父链上没删过第一个文件为 `None`。
    /// K-U5 的 β 时序级读法拿它与目标的对象出生代比。
    pub death_txg_of_the_first_file: Option<ModelCheckpointTxg>,
}

impl ModelRoot {
    /// 这一版 inode 树里有几条记录：第一个文件那条（活着、删除中、截断途中都在）加第一个文件之外的每个 inode。建 inode 的新记录落在哪一片叶容器
    /// 由它定（[`inode_leaf_container_position_of_a_new_record`]）。
    pub(super) fn inode_records_in_the_tree(&self) -> u64 {
        let first_file_record = match self.contents {
            ModelVersionContents::LiveFile(_)
            | ModelVersionContents::FileBeingDeleted { .. }
            | ModelVersionContents::FileBeingTruncated { .. } => 1,
            ModelVersionContents::WithoutTrees
            | ModelVersionContents::TreesAndNoFile { .. }
            | ModelVersionContents::InodesWithoutTheFirstFile { .. } => 0,
        };
        first_file_record + u64::try_from(self.inodes_other_than_the_first.len()).expect("条数")
    }

    /// 这一版 extent 树上段里有条目的 inode 号（[`inodes_with_an_extent_upper_leaf_entry`]）。
    #[must_use]
    pub fn inodes_with_an_extent_upper_leaf_entry(&self) -> BTreeSet<u64> {
        inodes_with_an_extent_upper_leaf_entry(&self.contents, &self.inodes_other_than_the_first)
    }

    /// 这一版 logged_ops 里那条截断条目的签发 txg（K1：全池至多一条）：截断途中的第一个文件，或截断条目点名的第一个文件之外的 inode
    /// （判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S3）；不在截断途中为 `None`。
    #[must_use]
    pub fn logged_truncation_issued_txg(&self) -> Option<ModelCheckpointTxg> {
        self.contents.logged_truncation_issued_txg().or_else(|| {
            self.inodes_other_than_the_first
                .values()
                .find_map(|inode| inode.truncation_issued_txg)
        })
    }

    /// 这一版第一个文件之外有没有文件带数据单元：快照、截断、整理那几臂在这一形上照答（判决
    /// `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S5），计数按它分出「别的文件带数据时」那几格。
    #[must_use]
    pub fn has_a_file_other_than_the_first_holding_data_units(&self) -> bool {
        self.inodes_other_than_the_first
            .values()
            .any(|inode| inode.data_units_present() > 0)
    }

    /// 这一版删除中的那个号（`deleted_inodes` 至多一条 key，D8（核心索引结构） 已定项 16 第 6 条）与它还剩几个数据单元；没有删除中的为 `None`。
    pub(super) fn inode_being_deleted(&self) -> Option<(ModelDeletedInode, u64)> {
        if let ModelVersionContents::FileBeingDeleted {
            data_units_remaining,
            ..
        } = self.contents
        {
            return Some((ModelDeletedInode::FirstFile, data_units_remaining));
        }
        self.inodes_other_than_the_first
            .iter()
            .find_map(|(number, inode)| match inode.liveness {
                ModelInodeLiveness::BeingDeleted {
                    data_units_remaining,
                } => Some((
                    ModelDeletedInode::OtherThanTheFirst(*number),
                    data_units_remaining,
                )),
                ModelInodeLiveness::Live => None,
            })
    }

    /// 这一版 extent 树上段每个节点的分配代：根（罩得住有条目最大号的最低一层的第 0 个）取 [`ModelUnitRole::ExtentRoot`] 的，根之下的取
    /// [`ModelUnitRole::ExtentUpperNodeBelowTheRoot`] 那几个的。树表 0 条的一版没有上段，空的。
    pub(super) fn extent_upper_node_written_at(
        &self,
    ) -> BTreeMap<ModelExtentUpperNodePosition, ModelCheckpointTxg> {
        Self::extent_upper_node_written_at_of(
            &self.role_written_at,
            &self.inodes_with_an_extent_upper_leaf_entry(),
        )
    }

    /// 按角色记的分配代（`role_written_at`）与这一版有条目的号（`inodes_with_an_entry`）拼出 extent 树上段每个节点的分配代
    /// （[`ModelRoot::extent_upper_node_written_at`]）：发布写新根时那一版的 [`ModelRoot`] 还没拼好，用这一份。
    pub(super) fn extent_upper_node_written_at_of(
        role_written_at: &BTreeMap<ModelUnitRole, ModelCheckpointTxg>,
        inodes_with_an_entry: &BTreeSet<u64>,
    ) -> BTreeMap<ModelExtentUpperNodePosition, ModelCheckpointTxg> {
        let root_level = extent_upper_segment_root_level(inodes_with_an_entry);
        role_written_at
            .iter()
            .filter_map(|(role, written_at)| match role {
                ModelUnitRole::ExtentRoot => Some((
                    ModelExtentUpperNodePosition {
                        level: root_level,
                        index_in_level: 0,
                    },
                    *written_at,
                )),
                ModelUnitRole::ExtentUpperNodeBelowTheRoot(position) => {
                    Some((*position, *written_at))
                }
                ModelUnitRole::Data
                | ModelUnitRole::InodeLeaf
                | ModelUnitRole::InodeLeafRightOfTheLeftmost(_)
                | ModelUnitRole::InodeRoot
                | ModelUnitRole::AllocationTree
                | ModelUnitRole::AccountingTree
                | ModelUnitRole::MappingTree
                | ModelUnitRole::TreeTable
                | ModelUnitRole::InstanceTable
                | ModelUnitRole::DeletedInodesRoot
                | ModelUnitRole::LoggedOperationsRoot
                | ModelUnitRole::TombstoneLeaf
                | ModelUnitRole::TombstoneRoot
                | ModelUnitRole::RetirementLeaf
                | ModelUnitRole::RetirementRoot => None,
            })
            .collect()
    }
}

/// 第一个文件的 inode 号（D8（核心索引结构） 已定项 16：第一版的文件是 inode 1）。模型自己写一份，不引实现的常量（D13（验证路线） 已定项 5）。
pub(super) const FIRST_INODE_NUMBER: u64 = 1;

/// 一版 extent 树上段里有条目的 inode 号：第一个文件有数据单元时（活着、截断途中、删除中还剩单元）连 inode 1，加第一个文件之外有数据单元的
/// 每个文件（建出来没写过数据的没有条目）。
pub(super) fn inodes_with_an_extent_upper_leaf_entry(
    contents: &ModelVersionContents,
    inodes_other_than_the_first: &BTreeMap<u64, ModelInodeOtherThanTheFirst>,
) -> BTreeSet<u64> {
    let first_file = (contents.data_units_present() > 0).then_some(FIRST_INODE_NUMBER);
    first_file
        .into_iter()
        .chain(
            inodes_other_than_the_first
                .iter()
                .filter(|(_, inode)| inode.has_an_extent_upper_leaf_entry())
                .map(|(number, _)| *number),
        )
        .collect()
}

/// 一版的中央映射树里第一个文件之外的文件那几条条目（D19（块指针的结构与宽度预算） 已定项 8：每个数据单元、extent 树每个节点各一条）：
/// 每个有数据单元的文件，单元数加它下段的节点数（[`extent_lower_segment_nodes`]）。
pub(super) fn central_mapping_entries_of_the_files_other_than_the_first(
    inodes_other_than_the_first: &BTreeMap<u64, ModelInodeOtherThanTheFirst>,
) -> u64 {
    inodes_other_than_the_first
        .values()
        .map(|inode| {
            let data_units = inode.data_units_present();
            data_units + extent_lower_segment_nodes(data_units)
        })
        .sum()
}

/// 一个整理区域 R（判决 G1：一个聚簇段，起始槽 64 对齐、槽数 64；两盘同槽时罩住每块盘上的同号槽，「R 按记录全空」按 `device` 那块盘的记录判）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelCompactionRegion {
    pub device: ModelDeviceIdentity,
    pub start_slot: u64,
    pub slot_count: u64,
}

/// logged_ops 里的一条整理意图，模型记、比的那几样（判决 G1 的字段表）。`segs0` 记签发那次发布之前按记录口径的全空聚簇段数，停机谓词
/// 「全局差值」相对它比（判决 `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H3「取签发之前」）；
/// 模型不记分配记录、数不出它，这里不记、不比。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelCompactionIntent {
    /// key 里的签发 txg：签发那次发布的 txg。
    pub issued_txg: ModelCheckpointTxg,
    pub region: ModelCompactionRegion,
    /// `c0`：签发那一刻的分配代，就是签发那次发布接在后面的那一版的 txg。
    pub allocation_generation_at_issue: ModelCheckpointTxg,
}

/// 用户可见的两种树（D16（发布语义） 已定项 1「非空」比的条目：活头与快照的 inode 树根、extent 树根）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModelUserVisibleTree {
    InodeTree,
    ExtentTree,
}

/// 用户可见树的一个根单元：哪一种树的根、出生 txg（写它的那次发布）。一次发布至多写一种树的一个根，这两样定一个单元；活头与快照条目抄的是
/// 同一个单元时是同一个值（搬它，两条根指针一起变）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelUserVisibleRootUnit {
    pub tree: ModelUserVisibleTree,
    pub birth_txg: ModelCheckpointTxg,
}

impl ModelRoot {
    /// 这一版引用着的用户可见树的根单元，按「非空」比的次序：活头的 inode 树根、extent 树根（树表 0 条的一版没有），再是每个快照的两个
    /// （按建的先后）。活头的取这一版那个角色的分配代（= 出生 txg，D26（后台整理与放置回收） 已定项 5 第 2 条：搬迁不改它）。
    #[must_use]
    pub fn user_visible_root_units(&self) -> Vec<ModelUserVisibleRootUnit> {
        let live_head = [
            (ModelUserVisibleTree::InodeTree, ModelUnitRole::InodeRoot),
            (ModelUserVisibleTree::ExtentTree, ModelUnitRole::ExtentRoot),
        ]
        .into_iter()
        .filter_map(|(tree, role)| {
            self.role_written_at
                .get(&role)
                .map(|birth_txg| ModelUserVisibleRootUnit {
                    tree,
                    birth_txg: *birth_txg,
                })
        });
        let of_the_snapshots = self.snapshots.iter().flat_map(|snapshot| {
            [
                ModelUserVisibleRootUnit {
                    tree: ModelUserVisibleTree::InodeTree,
                    birth_txg: snapshot.inode_tree_written_by.checkpoint_txg,
                },
                ModelUserVisibleRootUnit {
                    tree: ModelUserVisibleTree::ExtentTree,
                    birth_txg: snapshot.extent_root_written_at,
                },
            ]
        });
        live_head.chain(of_the_snapshots).collect()
    }

    /// 这条根的用户可见树（inode 树、extent 树）的根指针是谁写的：树表 0 条时 None。
    fn user_visible_trees_written_by(&self) -> Option<ModelRootKey> {
        self.contents.user_visible_trees_written_by()
    }

    /// 「非空」与「4 个不同状态」比的那一样（`mount::user_visible_trees_changed`，判决 `research/prompts/m4-snapshot-r1-main-verification.md`
    /// 之后用户可见的树连同快照那几对条目的根指针）：inode 树、extent 树是谁写的，加树表里每个快照两条根指针抄的是谁，按表里的次序。
    /// 只比根指针，不比「销毁中」记号与前驱：删快照的意图那次与不是最后一批的批不改它，建快照与最后一批改它。
    /// 整理搬过的根单元（[`ModelRoot::user_visible_root_relocations`]）按最后一次搬它的 txg 进来：搬一次根指针就换一次映射位置，字节变了就是新状态
    /// （「照字节比，搬根算非空」）。
    pub(super) fn user_visible_state(&self) -> ModelUserVisibleState {
        ModelUserVisibleState {
            trees_written_by: self.user_visible_trees_written_by(),
            snapshot_roots: self
                .snapshots
                .iter()
                .map(|snapshot| {
                    (
                        snapshot.inode_tree_written_by,
                        snapshot.extent_root_written_at,
                    )
                })
                .collect(),
            root_relocations: self
                .user_visible_root_units()
                .iter()
                .map(|unit| self.user_visible_root_relocations.get(unit).copied())
                .collect(),
        }
    }
}

/// [`ModelRoot::user_visible_state`] 交回的那一样：两条根相等 ⟺ 它们在盘上的用户可见根指针（连同快照条目的根指针）逐字节相同。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ModelUserVisibleState {
    pub(super) trees_written_by: Option<ModelRootKey>,
    pub(super) snapshot_roots: Vec<(ModelRootKey, ModelCheckpointTxg)>,
    /// [`ModelRoot::user_visible_root_units`] 逐个：最后一次搬它的 txg，没搬过为 `None`。
    pub(super) root_relocations: Vec<Option<ModelCheckpointTxg>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 快照删到一半（意图之后）的那一版与删完的那一版比「非空」（`mount::user_visible_trees_changed` 连同快照条目的根指针）：建快照那次换了状态，
    /// 意图那次只改记号与前驱、状态不变，最后一批删掉那一对条目又换了状态。
    #[test]
    fn snapshot_creation_and_the_final_destruction_batch_change_the_user_visible_state_and_the_intent_does_not(
    ) {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        let before = model
            .current_version_in_the_session()
            .expect("会话开着")
            .user_visible_state();
        let creation = model.answer_create_snapshot().expect("建快照");
        succeed(&mut model, &creation);
        let created = model
            .current_version_in_the_session()
            .expect("会话开着")
            .user_visible_state();
        assert_ne!(created, before, "多了一对快照根：非空");
        let intent = model.answer_destroy_snapshot(0).expect("意图");
        succeed(&mut model, &intent);
        let intended = model
            .current_version_in_the_session()
            .expect("会话开着")
            .user_visible_state();
        assert_eq!(intended, created, "意图那次不算非空");
        let final_batch = model.answer_next_step_of_the_walk().expect("最后一批");
        succeed(&mut model, &final_batch);
        let destroyed = model
            .current_version_in_the_session()
            .expect("会话开着")
            .user_visible_state();
        assert_ne!(destroyed, intended, "删掉那一对：非空");
        assert_eq!(destroyed, before, "删完回到建快照之前的状态");
    }

    /// 快照个数相同、活头相同、快照抄的根不同的两版，用户可见状态（「非空」与「4 个不同状态」比的那一样）要分得出（判决
    /// `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F7，攻方 T11 那条用例）：建快照（抄活头 txg 9 的根）、
    /// 覆盖写成活头 txg 11——这一版记为删之前；删掉那个快照（意图与最后一批）、再建一个（抄活头 txg 11 的根）——这一版记为重建之后。
    /// 两版的活头都是 txg 11 写的、都只有一个快照、都没搬过根，只差快照抄的根。
    #[test]
    fn two_versions_with_one_snapshot_each_differ_in_the_user_visible_state_when_the_snapshots_copy_different_roots(
    ) {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        let creation = model.answer_create_snapshot().expect("建快照");
        succeed(&mut model, &creation);
        let overwrite = model.answer_publish_overwrite(&[42]).expect("覆盖写");
        succeed(&mut model, &overwrite);
        let before_the_destruction = model
            .current_version_in_the_session()
            .expect("会话开着")
            .user_visible_state();
        let intent = model.answer_destroy_snapshot(0).expect("意图");
        succeed(&mut model, &intent);
        let final_batch = model.answer_next_step_of_the_walk().expect("最后一批");
        succeed(&mut model, &final_batch);
        let recreation = model.answer_create_snapshot().expect("再建快照");
        succeed(&mut model, &recreation);
        let after_the_recreation = model
            .current_version_in_the_session()
            .expect("会话开着")
            .user_visible_state();
        assert_eq!(
            (
                &before_the_destruction.trees_written_by,
                &before_the_destruction.root_relocations,
            ),
            (
                &after_the_recreation.trees_written_by,
                &after_the_recreation.root_relocations,
            ),
            "两版的活头相同、快照个数相同（根单元逐个都没搬过）"
        );
        assert_ne!(
            before_the_destruction, after_the_recreation,
            "快照抄的根不同：用户可见状态不同"
        );
    }

    fn upper_position(level: u64, index_in_level: u64) -> ModelExtentUpperNodePosition {
        ModelExtentUpperNodePosition {
            level,
            index_in_level,
        }
    }

    /// 模型自己那一份上段规范形计划（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M2）走正推模型的四种走法
    /// （`research/prompts/m3-multi-file-unlink-r1-sonnet-model/out/run.out` 第 12、13、15、16 行，S = {1, 500}，产品几何）：
    /// 写 500 长高（旧根 (0, 0) 照抄成根之下、(0, 3) 与新根 (1, 0) 这次写）；删 500 降层（上一版的 (0, 0) 照抄成根，分配代不变）；
    /// 删 1（(0, 3) 照抄、根 (1, 0) 少了孩子重写）；删空（零条目根兼叶 (0, 0) 重写）。
    #[test]
    fn the_model_upper_plan_grows_lowers_and_empties_the_segment_as_the_forward_model_does() {
        let written_at_three = ModelCheckpointTxg(3);
        let written_at_five = ModelCheckpointTxg(5);
        let this_publish = ModelCheckpointTxg(9);
        let only_the_first_file = BTreeMap::from([(upper_position(0, 0), written_at_three)]);
        assert_eq!(
            extent_upper_node_written_at_after(
                &only_the_first_file,
                &BTreeSet::from([1, 500]),
                &BTreeSet::from([500]),
                this_publish,
            ),
            BTreeMap::from([
                (upper_position(0, 0), written_at_three),
                (upper_position(0, 3), this_publish),
                (upper_position(1, 0), this_publish),
            ]),
            "写 500 长高：旧根照抄成根之下的节点，500 那一串新写"
        );
        let both_files = BTreeMap::from([
            (upper_position(0, 0), written_at_three),
            (upper_position(0, 3), written_at_five),
            (upper_position(1, 0), written_at_five),
        ]);
        assert_eq!(
            extent_upper_node_written_at_after(
                &both_files,
                &BTreeSet::from([1]),
                &BTreeSet::from([500]),
                this_publish,
            ),
            BTreeMap::from([(upper_position(0, 0), written_at_three)]),
            "删 500 降层：上一版的第 0 片叶照抄成根"
        );
        assert_eq!(
            extent_upper_node_written_at_after(
                &both_files,
                &BTreeSet::from([500]),
                &BTreeSet::from([1]),
                this_publish,
            ),
            BTreeMap::from([
                (upper_position(0, 3), written_at_five),
                (upper_position(1, 0), this_publish),
            ]),
            "删 1：500 那片叶照抄，根少了孩子重写，第 0 片叶摘掉"
        );
        assert_eq!(
            extent_upper_node_written_at_after(
                &only_the_first_file,
                &BTreeSet::new(),
                &BTreeSet::from([1]),
                this_publish,
            ),
            BTreeMap::from([(upper_position(0, 0), this_publish)]),
            "删空：零条目根兼叶重写"
        );
    }
}
