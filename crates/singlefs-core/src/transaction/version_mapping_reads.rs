//! 从一版里按 inode、按角色、按 key 读中央映射：映射的位置与逐片叶的读法。

use super::*;

/// 一版的 `files_other_than_the_first` 按 inode 号找文件的批量形态（七·6 代码轮第三轮判决第三节第 1 条）：每个 inode 号的答案与
/// [`TransactionOutput::file_other_than_the_first`]（在表上 `iter().find`，取最先那一项）相同，但走 [`TransactionOutput::files_other_than_the_first_by_inode`]
/// 那个索引。释放的三处与快照判定那一圈对换下的每个角色问一次它在上一版里有没有落点，逐个在表上找是角色数乘文件数。
/// 索引在第一次查时才建：一个第一个文件之外的角色都不问的不建。建与查记在调用方此刻所在的那一处（调用方各进各的：
/// [`LookupSite::FilesOfTheReplacedRolesInThePreviousVersion`]、[`LookupSite::FilesOfTheMappedRolesReleasedByThisPublish`]、
/// [`LookupSite::FilesOfTheUserVisibleRolesOfTheSnapshotJudgement`]）：每问一个角色现建一份读者的，建的那几份也记在那一处。
pub(crate) struct FilesOtherThanTheFirstOfAVersionReadByInode<'version> {
    pub(super) version: &'version TransactionOutput,
    index_built_at_the_first_lookup:
        Option<FirstEntryOfEachIdentity<'version, InodeNumber, FileOfAnInodeOtherThanTheFirst>>,
}

impl<'version> FilesOtherThanTheFirstOfAVersionReadByInode<'version> {
    /// 还什么都没建。
    pub(crate) fn of_the_version(version: &'version TransactionOutput) -> Self {
        Self {
            version,
            index_built_at_the_first_lookup: None,
        }
    }

    /// 同 [`TransactionOutput::file_other_than_the_first`]`(inode)`。
    pub(crate) fn file_other_than_the_first(
        &mut self,
        inode: InodeNumber,
    ) -> Option<&'version FileOfAnInodeOtherThanTheFirst> {
        let version = self.version;
        self.index_built_at_the_first_lookup
            .get_or_insert_with(|| version.files_other_than_the_first_by_inode())
            .first_entry_of(&inode)
    }
}

/// 一版里进映射的角色按角色取那条映射条目的批量形态（七·6，用户 2026-10-21 定并进）：每个角色的答案与「在 `mapped_units` 上按角色
/// `iter().find` 取最先那一把 key、再 [`central_mapping_locations_in_the_version`]」逐个相同，但 key 走 `mapped_units` 按角色的索引、
/// 落点走批量读者（每片叶只解一次）。释放判定路径、核位置项、释放前读盘核校验和、快照判定那一圈进 deadlist 的那一支各对一串角色逐个查，
/// 逐个在表上找、逐 key 重解叶是角色数乘表长。两个索引在第一次查时才建：一个角色都不查的不建。
pub struct MappedUnitsOfAVersionReadByRole<'version> {
    pub(super) version: &'version TransactionOutput,
    indexes_built_at_the_first_lookup: Option<IndexesOfAVersionReadByRole<'version>>,
}

/// [`MappedUnitsOfAVersionReadByRole`] 第一次查时建的两样。
pub(super) struct IndexesOfAVersionReadByRole<'version> {
    /// `mapped_units` 按角色的索引：取角色的映射 key。
    pub(super) mapped_units_by_role:
        FirstEntryOfEachIdentity<'version, TransactionUnit, (TransactionUnit, Vec<u8>)>,
    /// 中央映射的批量读者：按 key 取映射条目，每片叶只解一次。
    pub(super) central_mapping: CentralMappingOfAVersionReadLeafByLeaf<'version>,
}

impl<'version> MappedUnitsOfAVersionReadByRole<'version> {
    /// 还什么都没建。
    #[must_use]
    pub fn of_the_version(version: &'version TransactionOutput) -> Self {
        Self {
            version,
            indexes_built_at_the_first_lookup: None,
        }
    }

    /// 进映射的角色 `identity` 在这一版映射节点里的那条映射条目（按这一版记的 key 查）。
    ///
    /// # Panics
    /// `identity` 不是这一版里进映射的角色（映射树、树表、实例表豁免映射）：调用方按角色分过。
    /// key 走到的那片叶的角色不在这一版的 `units` 里（同 [`central_mapping_locations_in_the_version`]）。
    pub fn locations_of_role(&mut self, identity: TransactionUnit) -> MappingLookup {
        let version = self.version;
        let IndexesOfAVersionReadByRole {
            mapped_units_by_role,
            central_mapping,
        } = self
            .indexes_built_at_the_first_lookup
            .get_or_insert_with(|| IndexesOfAVersionReadByRole {
                mapped_units_by_role: version.mapped_units_by_role(),
                central_mapping: CentralMappingOfAVersionReadLeafByLeaf::of_the_version(version),
            });
        let (_, key) = mapped_units_by_role
            .first_entry_of(&identity)
            .expect("进映射的单元每个一把 key（数据、extent 根、每片 inode 叶容器、inode 根、分配记录树、记账树的每个节点）");
        central_mapping.locations_of_key(key)
    }
}

/// 在一版的中央映射树里按 key 查落点：按这一版的形状从根沿分隔 key 走到那片叶（D8（核心索引结构） 已定项 11：
/// 最后一个分隔 key ≤ key 的那个孩子），叶的字节取 `units` 里它那个角色的单元、在叶里按 key 查（`mapping_locations_for_key`）。
/// 查的是叶的字节，不是内存里记的 key：叶被改过就照实查不到。
#[must_use]
pub fn central_mapping_locations_in_the_version(
    version: &TransactionOutput,
    key: &[u8],
) -> MappingLookup {
    let shape = &version.central_mapping_tree.shape;
    let Some(leaf) = leaf_position_routed_to(
        shape,
        &CodeTwoTreeKey::new(key, CodeTwoKeyFieldWidths::CENTRAL_MAPPING),
    ) else {
        return MappingLookup::NodeMalformedOrNoEntryWithThisKey;
    };
    let leaf_role = MultiLevelCodeTwoTree::CentralMapping.role_of_node(leaf, shape);
    lookup_work::count(|work| work.mapping_leaf_decodes += 1);
    mapping_locations_for_key(&version.unit(leaf_role).bytes, key)
}

/// 一版的中央映射树按 key 查落点的批量形态：每把 key 的答案与 [`central_mapping_locations_in_the_version`] 逐个相同
/// （走到哪片叶、叶解不开、叶里有切不动的条目、同一把 key 有几条取最后一条，几种情形都同），但每片叶只解一次、叶的单元按角色从索引取。
/// 整理读一版时对全部进映射单元的 key 各查一次，逐 key 查是每把 key 线性找叶再整片重解（里程碑四七·6）。
/// 一片叶在第一次有 key 走到它时才解：没有 key 走到的叶不碰，与逐 key 查一样（那片叶的角色不在 `units` 里时同样 panic）。
/// 每解一片记一次解叶（[`lookup_work`]）：测试读的是解了几次，不是缓存里有几片。
pub struct CentralMappingOfAVersionReadLeafByLeaf<'version> {
    pub(super) shape: &'version CodeTwoTreeShape,
    pub(super) units_by_role: FirstEntryOfEachIdentity<'version, TransactionUnit, PublishedUnit>,
    leaves_read: BTreeMap<TransactionUnit, CentralMappingLeafRead>,
}

/// 一片中央映射叶解一次的样子，对走到这片叶的任一把 key 给出的答案与 [`mapping_locations_for_key`] 相同。
pub(super) enum CentralMappingLeafRead {
    /// 叶解不开：走到它的每把 key 都是 [`MappingLookup::NodeMalformedOrNoEntryWithThisKey`]。
    NodeMalformed,
    /// 叶里有一条条目切不动（带的是最先那条切不动的条目的字节数）：走到它的每把 key 都是
    /// [`MappingLookup::EntryNarrowerThanItsFieldTable`]，不论那把 key 的条目排在它前面还是后面（`mapping_locations_for_key` 遇到它就返回）。
    EntryNarrowerThanItsFieldTable { entry_bytes: usize },
    /// 叶里每把 key 的两条位置条目；同一把 key 有几条时取最后一条（`mapping_locations_for_key` 逐条往后覆盖 `found`）。
    LocationsOfEachKey(BTreeMap<Vec<u8>, [LocationEntry; 2]>),
}

impl CentralMappingLeafRead {
    fn of_the_leaf_bytes(mapping_node_bytes: &[u8]) -> Self {
        lookup_work::count(|work| work.mapping_leaf_decodes += 1);
        let Ok(node) = parse_index_node(mapping_node_bytes) else {
            return Self::NodeMalformed;
        };
        let mut locations_of_each_key = BTreeMap::new();
        // 迭代上界是叶里的条目数；跨轮携带的是已解出的 key → 落点，后面同一把 key 的条目覆盖前面的。
        for entry in &node.entries {
            let Some((key, locations)) = parse_mapping_entry(entry) else {
                return Self::EntryNarrowerThanItsFieldTable {
                    entry_bytes: entry.len(),
                };
            };
            locations_of_each_key.insert(key, locations);
        }
        Self::LocationsOfEachKey(locations_of_each_key)
    }

    pub(super) fn locations_of_key(&self, key: &[u8]) -> MappingLookup {
        match self {
            Self::NodeMalformed => MappingLookup::NodeMalformedOrNoEntryWithThisKey,
            Self::EntryNarrowerThanItsFieldTable { entry_bytes } => {
                MappingLookup::EntryNarrowerThanItsFieldTable {
                    entry_bytes: *entry_bytes,
                }
            }
            Self::LocationsOfEachKey(locations_of_each_key) => {
                locations_of_each_key.get(key).map_or(
                    MappingLookup::NodeMalformedOrNoEntryWithThisKey,
                    |locations| MappingLookup::Found(*locations),
                )
            }
        }
    }
}

impl<'version> CentralMappingOfAVersionReadLeafByLeaf<'version> {
    /// 建 `version` 的单元索引；叶一片都还没解。
    #[must_use]
    pub fn of_the_version(version: &'version TransactionOutput) -> Self {
        Self {
            shape: &version.central_mapping_tree.shape,
            units_by_role: version.units_by_role(),
            leaves_read: BTreeMap::new(),
        }
    }

    /// 同 [`TransactionOutput::unit`]：这一版里角色是 `identity` 的那个单元，按角色从这份读者建好的单元索引取（整理发批取计划里单元的字节，
    /// 七·6 代码轮第三轮判决第三节第 1 条），不另建索引、不在 `units` 上线性找。
    ///
    /// # Panics
    /// 同 [`TransactionOutput::unit`]。
    pub(crate) fn unit_of_role(&self, identity: TransactionUnit) -> &'version PublishedUnit {
        self.units_by_role.unit(identity)
    }

    /// 同 [`central_mapping_locations_in_the_version`]`(version, key)`。
    ///
    /// # Panics
    /// `key` 走到的那片叶的角色不在这一版的 `units` 里（同 [`central_mapping_locations_in_the_version`]）。
    pub fn locations_of_key(&mut self, key: &[u8]) -> MappingLookup {
        let Some(leaf) = leaf_position_routed_to(
            self.shape,
            &CodeTwoTreeKey::new(key, CodeTwoKeyFieldWidths::CENTRAL_MAPPING),
        ) else {
            return MappingLookup::NodeMalformedOrNoEntryWithThisKey;
        };
        let leaf_role = MultiLevelCodeTwoTree::CentralMapping.role_of_node(leaf, self.shape);
        let units_by_role = &self.units_by_role;
        self.leaves_read
            .entry(leaf_role)
            .or_insert_with(|| {
                CentralMappingLeafRead::of_the_leaf_bytes(&units_by_role.unit(leaf_role).bytes)
            })
            .locations_of_key(key)
    }
}

/// 在一个映射节点里按 key 查落点的结果。三个成员按**调用方要做的决定**分：交出落点、报「不在映射」、报「条目切不动」。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MappingLookup {
    /// 这把 key 的两条位置条目。
    Found([LocationEntry; 2]),
    /// 节点解不开，或者解得开、里面没有这把 key。两样让调用方做同一个决定（报「不在映射」、一个落点都不释放），
    /// 所以不分成两个成员（`code-discipline.md`「错误」：成员按调用方要做的决定分，不按底层原因逐个展开）。
    NodeMalformedOrNoEntryWithThisKey,
    /// 节点自述的条目宽窄于映射条目的字段表（55）：盘上读来的宽度，不是不变量（panic 面普查 R2）。
    EntryNarrowerThanItsFieldTable { entry_bytes: usize },
}

/// 在一个映射节点里按 key 查落点。
#[must_use]
pub fn mapping_locations_for_key(mapping_node_bytes: &[u8], key: &[u8]) -> MappingLookup {
    let Ok(node) = parse_index_node(mapping_node_bytes) else {
        return MappingLookup::NodeMalformedOrNoEntryWithThisKey;
    };
    let mut found = None;
    for entry in &node.entries {
        let Some((candidate, locations)) = parse_mapping_entry(entry) else {
            return MappingLookup::EntryNarrowerThanItsFieldTable {
                entry_bytes: entry.len(),
            };
        };
        if candidate == key {
            found = Some(locations);
        }
    }
    match found {
        Some(locations) => MappingLookup::Found(locations),
        None => MappingLookup::NodeMalformedOrNoEntryWithThisKey,
    }
}
