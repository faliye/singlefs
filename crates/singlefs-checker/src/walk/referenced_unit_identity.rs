//! 遍历方向上每个被引用单元的身份：头里的诞生代号、写序与出生序号对不对得上引用它的指针，树 ID、层级与 key 区间对不对得上父条目；
//! 连同走读时记下的落点，与先走过的版本再引用到同一单元时要判的那几格。

use super::*;

/// 指针尾段装的那个量：码 1 的指针 88 尾段是写序的事务号（6 字节），码 2 / 码 3 的指针 86 尾段是出生序号（4 字节）
/// （`image::parse_data_pointer` / `image::parse_node_pointer` 各解一份；`KEY_SCHEMA_MAPPING` 那一行的注同一个口径）。
/// 码 3 的写序里还有一个事务号，指针不带它 ⇒ 不比它。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TailOfTheBirthIdentity {
    TransactionNumber(u64),
    BirthSequence(u32),
}

impl TailOfTheBirthIdentity {
    pub(super) fn birth_identity_tail_number(self) -> u64 {
        match self {
            TailOfTheBirthIdentity::TransactionNumber(transaction) => transaction,
            TailOfTheBirthIdentity::BirthSequence(birth_sequence) => u64::from(birth_sequence),
        }
    }

    pub(super) fn name(self) -> &'static str {
        match self {
            TailOfTheBirthIdentity::TransactionNumber(_) => "事务号",
            TailOfTheBirthIdentity::BirthSequence(_) => "出生序号",
        }
    }
}

/// 单元头里与映射 key 的出生树那一段相比的字段（D19（块指针的结构与宽度预算） 已定项 6 的收严句）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BirthTreeInTheHeader {
    /// 码 1 五元组里的树（偏移 43）、码 3 类身份段里的出生树（偏移 43）。
    AtOffsetFortyThree(u64),
    /// 码 2：类身份段里没有叫「出生树」的字段，取哪个还开着（C289（码 2 的出生树取哪个字段没写）），今天不比。
    FieldStillOpenUnderC289,
}

/// 映射 key 的出生树在被指单元头里对的是哪个字段。类标签不在 1 / 2 / 3 里的没有这一格（交 None，那一格按类标签判）。
pub(super) fn birth_tree_in_the_header_as_the_mapping_key_names_it(
    unit: &[u8],
) -> Option<BirthTreeInTheHeader> {
    match MappingKeyUnitClass::of(unit[6]) {
        MappingKeyUnitClass::DataUnit | MappingKeyUnitClass::PackedRecordUnit => {
            Some(BirthTreeInTheHeader::AtOffsetFortyThree(read_u64(unit, 43)))
        }
        MappingKeyUnitClass::IndexNode => Some(BirthTreeInTheHeader::FieldStillOpenUnderC289),
        MappingKeyUnitClass::Unregistered(_) => None,
    }
}

/// 中央映射 key 的类标签（D19（块指针的结构与宽度预算） 已定项 6：取被命名对象在 D18（块里携带什么信息） 已定项 11 登记表里的码）
/// 在走读里的读法：盘上读来的一字节，语义上允许未知值，做成显式成员（`code-discipline.md`「分支：穷举写全，不写通配臂」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MappingKeyUnitClass {
    DataUnit,
    IndexNode,
    PackedRecordUnit,
    /// 码 0 与登记表之外的码：按 I-1.6（类标签登记且三处相等） 判损坏。
    Unregistered(u8),
}

impl MappingKeyUnitClass {
    pub(super) const fn of(tag: u8) -> Self {
        match tag {
            1 => Self::DataUnit,
            2 => Self::IndexNode,
            3 => Self::PackedRecordUnit,
            unregistered => Self::Unregistered(unregistered),
        }
    }

    /// 写进映射 key 首字节的类标签（D18（块里携带什么信息） 已定项 11 的登记表）。
    pub(super) const fn tag(self) -> u8 {
        match self {
            Self::DataUnit => 1,
            Self::IndexNode => 2,
            Self::PackedRecordUnit => 3,
            Self::Unregistered(tag) => tag,
        }
    }
}

/// 走读跟随一条指针时，被指单元落在哪里以谁为准（D19（块指针的结构与宽度预算） 已定项 5「位置条目降为提示，中央映射是唯一入口」、
/// 已定项 8「不进映射的三类（自举豁免）」； 与第七节）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PositionAuthority {
    /// 自举豁免的三类——映射树自己的节点、树表单元、实例表单元：父指针里的位置条目是权威，不拿去查映射。
    ThePointerItself,
    /// 豁免三类之外的码 1 / 码 2 / 码 3：按查找路径给的类（指针不带类标签，D19 已定项 8）拼出映射 key，这一版的中央映射里查得到就以映射
    /// 条目的位置为准，指针里的位置条目只是提示；查不到（映射读不全、这一版没有映射）照提示走，那一形由映射那一遍与 I-1.13 说话。
    TheCentralMapping(MappingKeyUnitClass),
}

/// 一条根自己的中央映射，只读不判：根记录偏移 256 起那 86 字节指着的映射树（D19（块指针的结构与宽度预算） 已定项 11；
/// [`CentralMappingLocationsOfAVersion::read_without_judging`]，读不全时是空表）。次级读法里只取最新根、或逐条根读的那几遍经这里取。
pub(super) fn central_mapping_of_a_root_record_without_judging(
    reader: &dyn ImageReader,
    root_record: &[u8],
    cache: &mut IndexNodeCache,
) -> CentralMappingLocationsOfAVersion {
    CentralMappingLocationsOfAVersion::read_without_judging(
        reader,
        &parse_node_pointer(&root_record[ROOT_RECORD_MAPPING_ROOT_POINTER]),
        cache,
    )
}

/// 一条指针指的那个单元的中央映射 key（D19（块指针的结构与宽度预算） 已定项 6 / 已定项 8 / 已定项 10）：类标签由查找路径给（`class`），
/// 出生树、出生 txg、实例代号取指针头里的；尾段 6 字节取指针尾段（码 1 是写序的事务号 6 字节，码 2 / 码 3 是出生序号 4 字节、其后 2 字节补零）。
pub(super) fn central_mapping_key_of_a_pointer(
    pointer: &PointerView,
    class: MappingKeyUnitClass,
) -> CentralMappingKey {
    let mut key = vec![0u8; mapping_key_bytes()];
    key[MAPPING_KEY_UNIT_CLASS_OFFSET] = class.tag();
    key[MAPPING_KEY_BIRTH_TREE_OFFSET..MAPPING_KEY_BIRTH_TXG_OFFSET]
        .copy_from_slice(&pointer.birth_tree.to_le_bytes());
    key[MAPPING_KEY_BIRTH_TXG_OFFSET..MAPPING_KEY_INSTANCE_OFFSET]
        .copy_from_slice(&pointer.birth_txg.to_le_bytes());
    key[MAPPING_KEY_INSTANCE_OFFSET..MAPPING_KEY_TAIL_OFFSET]
        .copy_from_slice(&pointer.instance.to_le_bytes());
    let tail_bytes = mapping_key_bytes() - MAPPING_KEY_TAIL_OFFSET;
    key[MAPPING_KEY_TAIL_OFFSET..].copy_from_slice(&pointer.tail.to_le_bytes()[..tail_bytes]);
    CentralMappingKey(key)
}

/// 指针自己那两条位置条目（提示）里第一份按它带的整单元校验和对得上的那一份，只读不判：提示过期（槽已释放、已被复用）不是违例，
/// 被引用的块在映射位置上，I-2.1 在那里判（D19（块指针的结构与宽度预算） 已定项 5；判决 G6「提示 ≠ 映射、提示那一槽已释放」在合法搬迁之后不许红）。
///
/// 校验和对得上还要头里的出生身份（诞生代号、写序实例、尾段）与查找路径的那一把相同：提示读到一个头与查找路径不符的单元照「读不出」走映射
pub(super) fn copy_at_a_location_hint(
    reader: &dyn ImageReader,
    pointer: &PointerView,
    unit_bytes: usize,
) -> Option<Vec<u8>> {
    pointer.location_hints.iter().find_map(|hint| {
        reader
            .read(hint.device, hint.slot * SLOT_BYTES, unit_bytes)
            .filter(|content| crc32_castagnoli_table(content) == hint.checksum)
            .filter(|content| {
                birth_identity_in_the_header(content).is_some_and(|identity| {
                    identity.write_order.birth_txg() == pointer.birth_txg
                        && identity.write_order.instance() == pointer.instance
                        && identity.tail.birth_identity_tail_number() == pointer.tail
                })
            })
    })
}

/// 一个被引用单元的头里，I-1.2（块头写序已发布） 的判定输入：诞生代号 b、写序 (i, n) 与出生序号。
/// 落点按 D18（块里携带什么信息） 已定项 7 的字段表（码 1：诞生代号 75、写序 91；码 2：诞生代号 52 + 2k、
/// 写序 68 + 2k、出生序号 72 + 2k）与已定项 11 的类身份段表（码 3：诞生代号 73、写序 93、出生序号 103）各解一份。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct BirthIdentityInTheHeader {
    pub(super) write_order: WriteOrderUnderThePublishedPredicate,
    pub(super) tail: TailOfTheBirthIdentity,
}

/// 从一个被引用单元的头里取出 I-1.2 的判定输入。类标签不在 1 / 2 / 3 里的没有这一格（那一格归 I-1.6（类标签登记且三处相等））。
pub(super) fn birth_identity_in_the_header(unit: &[u8]) -> Option<BirthIdentityInTheHeader> {
    match unit[6] {
        1 => Some(BirthIdentityInTheHeader {
            write_order: WriteOrderUnderThePublishedPredicate::DataUnit {
                birth_txg: read_u64(unit, 75),
                instance: read_u32(unit, 91),
                transaction: read_six_byte_unsigned(unit, 95),
            },
            tail: TailOfTheBirthIdentity::TransactionNumber(read_six_byte_unsigned(unit, 95)),
        }),
        2 => {
            let key_width = usize::from(unit[51]);
            Some(BirthIdentityInTheHeader {
                write_order: WriteOrderUnderThePublishedPredicate::IndexOrPackedRecordUnit {
                    birth_txg: read_u64(unit, 52 + 2 * key_width),
                    instance: read_u32(unit, 68 + 2 * key_width),
                },
                tail: TailOfTheBirthIdentity::BirthSequence(read_u32(unit, 72 + 2 * key_width)),
            })
        }
        3 => Some(BirthIdentityInTheHeader {
            write_order: WriteOrderUnderThePublishedPredicate::IndexOrPackedRecordUnit {
                birth_txg: read_u64(unit, 73),
                instance: read_u32(unit, 93),
            },
            tail: TailOfTheBirthIdentity::BirthSequence(read_u32(unit, 103)),
        }),
        _outside_the_three_unit_classes => None,
    }
}
impl<'reader> Walk<'reader> {
    /// 遍历方向上的两格，判的是同一个被引用单元，输入也是同一份（头里的诞生代号 b、写序 (i, n)、出生序号）：
    ///
    /// ① **I-1.2（块头写序已发布）**：头里的这几段与**引用它的那条指针**记的相同。两处不等时「头记录的写序」
    ///    就不是这个单元的真实身份，②那个谓词判的东西是假的。恢复路径今天在遍历方向判的正是这一格
    ///    （`crates/singlefs-core/src/recovery.rs` 的 `RecoveryFailure::InvariantViolated { invariant: "I-1.2" }`
    ///    四处：树根出生序号与指针不符、inode 叶的出生身份与子指针不符、inode 叶写序实例与子指针不符、
    ///    数据单元头与指针的出生身份不符），池级 checker 自己按字段表解一份，不从实现拿偏移。
    ///    **扫描方向那一半（「任一单元」都按谓词判，i > 挂载根实例 ⇒ 判损坏）池级 checker 判不了**，
    ///    理由写在 [`PublishedPredicate::holds_for`] 与 `crates/singlefs-checker/invariants/01-块自描述.md` I-1.2 的状态列里。
    ///
    /// ② **I-4.2（无被引用未提交块）**：这几段按 C113（扫描重建时多版单元的现行版本判定无输入） P2 的
    ///    已发布谓词判成已发布——「未提交」在本工程里就是「按那个谓词判不成已发布」，被引用的块必须已提交。
    ///    I-1.8（归并后版本全序） 那一边的扫描式方向照同一条条款另写了一份（[`PublishedPredicateOfTheScanDirection`]，C410（checker 两个方向共享什么没有检查）），
    ///    两个方向不共用这一段代码。
    ///
    /// **打包记录类型 4 的实例表单元不进这个函数**：I-1.2 那一行的「**例外**」写着它只免谓词那一半，
    /// 它的现行版本由根记录持有的物理指针唯一确定；它的出生身份与指针相等那一格在 `walk_instance_table_chain` 里单判
    /// （[`Self::judge_birth_identity_against_the_pointer`]），不进 I-4.2、也不进 I-4.9 与「判过几个被引用单元」那个数。
    ///
    /// 诞生代号记在 `placement_it_was_read_from` 下面：读回、自证过的那一份的落点，就是这个单元在走读里的身份
    /// （[`Self::read_a_unit_to_walk_and_the_placement_it_was_read_from`]）；I-4.9 那一行「按落点认，读落点上单元头里的诞生代号」。
    pub(super) fn judge_birth_identity_of_a_referenced_unit(
        &mut self,
        unit: &[u8],
        pointer: &PointerView,
        placement_it_was_read_from: (u32, u64),
        what: &str,
    ) {
        let Some(identity) = birth_identity_in_the_header(unit) else {
            return;
        };
        self.referenced_units_judged_against_their_pointer += 1;
        let birth_txg = identity.write_order.birth_txg();
        self.birth_txg_of_walked_units
            .insert(placement_it_was_read_from, birth_txg);
        self.judge_unit_birth_against_an_older_candidate_root(birth_txg, what);
        self.judge_birth_identity_against_the_pointer(identity, pointer, what);
        let mount_root_instance = self.mount_root_instance;
        let mount_root_checkpoint_txg = self.mount_root_checkpoint_txg;
        let published = PublishedPredicate {
            mount_root_instance,
            mount_root_checkpoint_txg,
            instance_table_rows: &self.instance_table_rows,
        };
        let is_published = published.holds_for(identity.write_order);
        // 借的是 `instance_table_rows` 这一个字段，`judge` 要的是 `judgements` 那一个：两处不重叠。
        // 不 clone：崩溃点重放每个崩溃状态都要在每个被引用单元上走一遍这里。
        let rows = &self.instance_table_rows;
        self.judgements.judge("I-4.2", is_published, || {
            format!(
                "{what}：头里的诞生代号 {}、写序实例 {} 按已发布谓词判不成已发布（挂载根实例 {mount_root_instance}、txg {mount_root_checkpoint_txg}，实例表行 {rows:?}）——被引用的块必须是已提交的块",
                identity.write_order.birth_txg(),
                identity.write_order.instance()
            )
        });
    }

    /// I-1.2（块头写序已发布） 遍历方向那一格：头里的诞生代号、写序实例与尾段（码 1 事务号、码 2 / 码 3 出生序号）与引用它的指针记的相同。
    /// 被引用单元（[`Self::judge_birth_identity_of_a_referenced_unit`]）与实例表链上每一片（打包记录类型 4 按 I-1.2 那一行的例外只免谓词那一半，
    /// `research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 3 行）都判它。
    pub(super) fn judge_birth_identity_against_the_pointer(
        &mut self,
        identity: BirthIdentityInTheHeader,
        pointer: &PointerView,
        what: &str,
    ) {
        let matches_the_pointer = identity.write_order.birth_txg() == pointer.birth_txg
            && identity.write_order.instance() == pointer.instance
            && identity.tail.birth_identity_tail_number() == pointer.tail;
        self.judgements.judge("I-1.2", matches_the_pointer, || {
            format!(
                "{what}：头里的出生身份（诞生代号 {}、写序实例 {}、{} {}）与引用它的指针记的（诞生代号 {}、实例 {}、尾段 {}）不符",
                identity.write_order.birth_txg(),
                identity.write_order.instance(),
                identity.tail.name(),
                identity.tail.birth_identity_tail_number(),
                pointer.birth_txg,
                pointer.instance,
                pointer.tail
            )
        });
    }

    /// 一遍新的走读：判定接着 `judgements` 往下记，别的状态都是空的；挂载根取 (`mount_root_instance`, `mount_root_checkpoint_txg`)。
    pub(super) fn starting_with(
        reader: &'reader dyn ImageReader,
        judgements: Judgements,
        filesystem_identifier_low: u64,
        mount_root_instance: u32,
        mount_root_checkpoint_txg: u64,
    ) -> Self {
        Walk {
            reader,
            judgements,
            filesystem_identifier_low,
            references: BTreeMap::new(),
            visited_units: BTreeSet::new(),
            walked_units_whose_header_the_walk_could_use: BTreeSet::new(),
            placements_referenced_in_this_version: BTreeMap::new(),
            walk_parent_unit: None,
            placements_referenced_below_a_walked_unit: BTreeMap::new(),
            inode_leaf_order_below_a_walked_unit: BTreeMap::new(),
            walk_failures: Vec::new(),
            tree_identifiers_in_tables: BTreeSet::new(),
            inode_object_birth: BTreeMap::new(),
            inode_tree_walked: false,
            largest_inode_number_in_the_inode_tree: None,
            data_unit_objects: Vec::new(),
            accounting: BTreeMap::new(),
            accounting_seen: false,
            accounting_tree_root_birth_txg: None,
            inode_number_watermark_rows: Vec::new(),
            instance_table_rows: Vec::new(),
            abandoned_roots_filtering:
                AbandonedRootsFilteringOfTheCandidateSet::UnfilteredAsTheMainWalkStoppedBeforeTheLastPageOfTheNewestRootsInstanceTable,
            mount_root_instance,
            mount_root_checkpoint_txg,
            referenced_units_judged_against_their_pointer: 0,
            version_being_walked: VersionBeingWalked::NewestRoot {
                checkpoint_txg: mount_root_checkpoint_txg,
            },
            birth_txg_of_walked_units: BTreeMap::new(),
            units_judged_against_an_older_candidate_root: 0,
            unit_of_each_placement: BTreeMap::new(),
            placements_first_noted_through_the_central_mapping: BTreeSet::new(),
            user_visible_tree_of_each_placement_in_this_version: BTreeMap::new(),
            user_visible_tree_of_each_placement_below_a_walked_tree_table: BTreeMap::new(),
            user_visible_tree_being_walked: None,
            user_visible_tree_roots_of_this_version: Vec::new(),
            kind_of_a_tree_table_entry_without_a_clause_in_this_version: None,
            tree_table_entries_of_a_walked_tree_table: BTreeMap::new(),
            birth_txg_of_walked_instance_table_pages: BTreeMap::new(),
            versions_walked_for_the_appearance_rule: Vec::new(),
            central_mapping_of_this_version: CentralMappingLocationsOfAVersion::default(),
            unit_of_each_central_mapping_key: BTreeMap::new(),
            tree_sharing_in_the_walk: TreeSharingInTheWalk::OwnTreesOfTheVersion,
            snapshot_trees_being_destroyed_in_this_version: BTreeSet::new(),
        }
    }

    /// I-4.9（非最新候选根引用的单元不晚于它）：正在走的是回退候选集里不是最新的一条根时，它引用的一个单元的诞生代号不大于这条根的
    /// `checkpoint_txg`（D16（发布语义） 已定项 7：单元先于根落盘）。最新那条根归 I-4.2（无被引用未提交块），由记录施加出来的那一版不在射程里。
    fn judge_unit_birth_against_an_older_candidate_root(&mut self, birth_txg: u64, what: &str) {
        if let VersionBeingWalked::OlderCandidateRoot { checkpoint_txg } = self.version_being_walked
        {
            self.units_judged_against_an_older_candidate_root += 1;
            self.judgements.judge(
                UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT,
                birth_txg <= checkpoint_txg,
                || {
                    format!(
                        "{what}：诞生代号 {birth_txg} 大于引用它的候选根的 txg {checkpoint_txg}"
                    )
                },
            );
        }
    }

    /// I-1.12（指针出生树等于被指单元头）：走读跟随的一条指针偏移 34 的出生树等于被指单元头里的出生树——码 1、码 3 取头偏移 43；
    /// 码 2 取哪个字段随 C289（码 2 的出生树取哪个字段没写） 定，不判。inode 内部条目指着叶容器那一类由 I-9.2（条目身份与子头相符） 罩住，
    /// 调用点不在那里。
    pub(super) fn judge_pointer_birth_tree_against_the_unit_header(
        &mut self,
        unit: &[u8],
        pointer: &PointerView,
        what: &str,
    ) {
        match birth_tree_in_the_header_as_the_mapping_key_names_it(unit) {
            Some(BirthTreeInTheHeader::AtOffsetFortyThree(birth_tree_in_the_header)) => {
                self.judgements.judge(
                    POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER,
                    pointer.birth_tree == birth_tree_in_the_header,
                    || {
                        format!(
                            "{what}：指针记的出生树 {} 与单元头里的出生树 {birth_tree_in_the_header} 不符",
                            pointer.birth_tree
                        )
                    },
                );
            }
            Some(BirthTreeInTheHeader::FieldStillOpenUnderC289) | None => {}
        }
    }

    /// I-1.3（块头树 ID 一致） 的码 2 节点那一格：头里的树 ID 是引用它的树。读节点那两处（[`Self::read_index_node`]、
    /// [`Self::walk_inode_internal_child`]）与别的版本先走过的节点被再引用时（[`Self::judge_the_cells_the_reference_decides_on_a_unit_walked_earlier`]）共用。
    pub(super) fn judge_the_tree_identifier_of_an_index_node(
        &mut self,
        view: &crate::IndexNodeView,
        expected_tree: u64,
        what: &str,
    ) {
        self.judgements
            .judge("I-1.3", view.tree_identifier == expected_tree, || {
                format!(
                    "{what}：头里的树 ID {} 不是引用它的树 {expected_tree}",
                    view.tree_identifier
                )
            });
    }

    /// I-1.3（块头树 ID 一致） 的码 3 容器那一格：偏移 43 的出生树是引用它的那棵树（I-1.3 那一行：码 3 头里的树 ID 是出生树）。
    /// 走 inode 叶容器、墓碑 / 谱系 / 退役树容器的两处与别的版本先走过的容器被再引用时共用。
    pub(super) fn judge_the_birth_tree_of_a_packed_container(
        &mut self,
        unit: &[u8],
        expected_tree: u64,
        what: &str,
    ) {
        self.judgements
            .judge("I-1.3", read_u64(unit, 43) == expected_tree, || {
                format!("{what}的出生树 {} 不是 {expected_tree}", read_u64(unit, 43))
            });
    }

    /// 数据单元按 extent key (`tree`, `inode`, `offset`) 定的两格：I-1.3（块头树 ID 一致，数据单元头里的树 ID 是出生树）与
    /// I-1.1（块头自述逻辑地址，五元组里的树、对象、锚点就是这条 key）。走 extent 记录那一处与别的版本先走过的数据单元被再引用时共用。
    pub(super) fn judge_a_data_unit_against_its_extent_key(
        &mut self,
        unit: &[u8],
        tree: u64,
        inode: u64,
        offset: u64,
    ) {
        self.judgements
            .judge("I-1.3", read_u64(unit, 43) == tree, || {
                format!(
                    "数据单元头里的树 ID {} 不是 extent 树 {tree}",
                    read_u64(unit, 43)
                )
            });
        let five_tuple_holds = read_u64(unit, 51) == inode
            && read_u64(unit, 67) == offset
            && read_u64(unit, 43) == tree;
        self.judgements.judge("I-1.1", five_tuple_holds, || {
            format!("数据单元五元组（树 {}、对象 {}、锚点 {}）与 extent key（{tree}, {inode}, {offset}）不符", read_u64(unit, 43), read_u64(unit, 51), read_u64(unit, 67))
        });
    }

    /// I-1.1（块头自述逻辑地址） 的实例表那一格：链上第 `page_index` 片的身份四元组 (出生树, 类型, 容器号, 容器出生代) = (0, 4, 片序号, 0)。
    /// 交回成不成立（走链那一处不成立就停）。走链那一处与别的版本先走过的一片被再引用时共用。
    pub(super) fn judge_the_identity_of_an_instance_table_page(
        &mut self,
        unit: &[u8],
        page_index: u64,
        what: &str,
    ) -> bool {
        let identity = (
            read_u64(unit, 43),
            read_u16(unit, 51),
            read_u64(unit, 53),
            read_u64(unit, 61),
        );
        let expected_identity = (0, PACKED_TYPE_INSTANCE_TABLE, page_index, 0);
        self.judgements
            .judge("I-1.1", identity == expected_identity, || {
                format!(
                    "{what}：身份四元组 (出生树, 类型, 容器号, 容器出生代) = {identity:?}，链上第 {page_index} 片该是 {expected_identity:?}"
                )
            });
        identity == expected_identity
    }

    /// 一个单元别的版本先走过（`visited_units` 命中），这一版又跟着一条引用到它：不往下展开，它下面的落点照
    /// [`Self::merge_placements_below_a_unit_walked_by_an_earlier_version`] 并进这一版；这一条引用自己定的那几格照判
    /// （[`Self::judge_the_cells_the_reference_decides_on_a_unit_walked_earlier`]，`records/2026-10-24-singlefs-checker-审计复核.md` 的 C01：
    /// 节点先被一条合法的引用走过，证明不了后来另一条引用也合法）。六处命中都走这里。
    pub(super) fn note_a_unit_an_earlier_version_walked_referenced_again(
        &mut self,
        unit: &[u8],
        pointer: &PointerView,
        placement_it_was_read_from: (u32, u64),
        reference: ReferenceTheWalkFollowsToAUnit,
        what: &str,
    ) {
        self.judge_the_cells_the_reference_decides_on_a_unit_walked_earlier(
            unit,
            pointer,
            placement_it_was_read_from,
            reference,
            what,
        );
        self.merge_placements_below_a_unit_walked_by_an_earlier_version(placement_it_was_read_from);
    }

    /// 别的版本先走过的单元被这一条引用再走到时，照判按这一条引用定的那几格，判的是这一次读回的那一份字节（与先走那一遍同一个落点）：
    /// - I-1.6（类标签登记且三处相等） 类那一半：头里的类标签是这条引用要的那一类；不是就不往下判（字段表按类不同，别的段解出来不是那几样）；
    /// - 先走那一遍头用得了时（`walked_units_whose_header_the_walk_could_use`）：I-1.2（块头写序已发布） 遍历方向那一格，头里的出生身份与这条指针相同；
    ///   I-1.3（块头树 ID 一致）：码 2 节点的树 ID、码 3 容器与数据单元的出生树是这条引用要的树（实例表豁免）；I-1.1（块头自述逻辑地址）：
    ///   数据单元五元组是这条 extent key、实例表的身份四元组是这一片的片号；I-1.12（指针出生树等于被指单元头）：只在先走那一遍判它的
    ///   两处（实例表、数据单元）判。
    ///
    /// 只按单元自己定的那几格不重判：头校验和、格式版本与预留位（I-2.4 / I-2.3 / I-1.4、I-1.6 的 flags 与类身份段首字节）、
    /// I-4.2 的已发布谓词、容器格式与记录（I-1.7 等）、`birth_txg_of_walked_units`。I-4.9 在并进前一版那一步按这一版照判。
    /// 「判过几个被引用单元」（`referenced_units_judged_against_their_pointer`）只在先走那一遍加：它数的是单元，不是引用。
    /// 按父条目定的层级、key 区间与子树覆盖那几格（I-1.1 的那一半，`.claude/kb/checks-owed.md` C896）不在这里：它们要父节点的视角，
    /// 由读码 2 节点的那几处调用方拿 [`Self::view_of_a_node_an_earlier_version_walked`] 交回的视图照判。
    fn judge_the_cells_the_reference_decides_on_a_unit_walked_earlier(
        &mut self,
        unit: &[u8],
        pointer: &PointerView,
        placement_it_was_read_from: (u32, u64),
        reference: ReferenceTheWalkFollowsToAUnit,
        what: &str,
    ) {
        let expected_class = reference.unit_class();
        let class = unit[6];
        self.judgements.judge("I-1.6", class == expected_class, || {
            format!(
                "{what}：别的版本先走过的盘 {} 槽 {} 的单元，头里的类标签是 {class}，这一条引用要的是 {expected_class}",
                placement_it_was_read_from.0, placement_it_was_read_from.1
            )
        });
        if class != expected_class
            || !self
                .walked_units_whose_header_the_walk_could_use
                .contains(&placement_it_was_read_from)
        {
            return;
        }
        if let Some(identity) = birth_identity_in_the_header(unit) {
            self.judge_birth_identity_against_the_pointer(identity, pointer, what);
        }
        match reference {
            ReferenceTheWalkFollowsToAUnit::IndexNodeOfTree { tree } => {
                if let Ok(view) = index_node_view(unit) {
                    self.judge_the_tree_identifier_of_an_index_node(&view, tree, what);
                }
            }
            ReferenceTheWalkFollowsToAUnit::PackedContainerOfTree { tree } => {
                self.judge_the_birth_tree_of_a_packed_container(unit, tree, what);
            }
            ReferenceTheWalkFollowsToAUnit::InstanceTablePage { page_index } => {
                self.judge_pointer_birth_tree_against_the_unit_header(unit, pointer, what);
                self.judge_the_identity_of_an_instance_table_page(unit, page_index, what);
            }
            ReferenceTheWalkFollowsToAUnit::DataUnitOfExtentKey {
                tree,
                inode,
                offset,
            } => {
                self.judge_pointer_birth_tree_against_the_unit_header(unit, pointer, what);
                self.judge_a_data_unit_against_its_extent_key(unit, tree, inode, offset);
            }
        }
    }

    /// 别的版本先走过的码 2 节点（`visited_units` 命中）这一次读回的那一份解开的样子：头里的类标签是码 2、先走那一遍头用得了
    /// （`walked_units_whose_header_the_walk_could_use`）、条目区解得开才有。调用方拿它照判这一版的父条目定的那几格——层级 = 父层级 − 1、
    /// key 落在父条目给的那一段、子树覆盖区间（I-1.1，`.claude/kb/checks-owed.md` C896）——不往下走：节点先被一条合法的父条目走过，
    /// 证明不了这一版的父条目也把它放在了对的地方。头用不了、类标签不是码 2 的那几种，先走那一遍或这一条引用的 I-1.6 已经红了，这里不交视图。
    pub(super) fn view_of_a_node_an_earlier_version_walked(
        &self,
        unit: &[u8],
        placement_it_was_read_from: (u32, u64),
    ) -> Option<crate::IndexNodeView> {
        let index_node_class =
            ReferenceTheWalkFollowsToAUnit::IndexNodeOfTree { tree: 0 }.unit_class();
        if unit[6] != index_node_class
            || !self
                .walked_units_whose_header_the_walk_could_use
                .contains(&placement_it_was_read_from)
        {
            return None;
        }
        index_node_view(unit).ok()
    }

    /// 父条目要的层级（`expected_level`，根是 `None`、不判）与节点头里的层级相同：I-1.1 那一格。首次走读与别的版本先走过的节点
    /// （C896）共用这一判，交回成立与否；不成立时首次走读另记走读失败、不往下走，命中时只判。
    pub(super) fn judge_the_level_the_parent_entry_names(
        &mut self,
        node_level: u8,
        expected_level: Option<u8>,
        what: &str,
    ) -> bool {
        let Some(level) = expected_level else {
            return true;
        };
        self.judgements.judge("I-1.1", node_level == level, || {
            format!("{what}：头里的层级是 {node_level}，引用它的父节点要层级 {level}（父层级减一）")
        });
        node_level == level
    }

    /// 按位置寻址的节点（分配记录树、extent 树上段与下段，D8（核心索引结构） 已定项 14）头里的层级是它的位置要的那一层、头里的 key 区间是
    /// 它的位置规定罩的那一段（I-1.1），交回层级成立与否（层级不对时区间不比）。首次走读（[`Self::position_and_entry_width_hold`]）与别的版本
    /// 先走过的节点共用这一判：位置由这一版的父条目定，别的版本按它自己的父条目走过它，证明不了这一版的父条目把它放在了它罩的那一段（C896）。
    pub(super) fn judge_the_level_and_the_key_range_the_position_prescribes(
        &mut self,
        view: &crate::IndexNodeView,
        level: u8,
        prescribed_range: &(Vec<u8>, Vec<u8>),
        what: &str,
    ) -> bool {
        self.judgements.judge("I-1.1", view.level == level, || {
            format!(
                "{what}：头里的层级是 {}，它的位置要层级 {level}",
                view.level
            )
        });
        if view.level != level {
            return false;
        }
        self.judgements.judge(
            "I-1.1",
            view.smallest_key == prescribed_range.0 && view.largest_key == prescribed_range.1,
            || format!("{what}：头里的 key 区间不是它的位置规定罩的那一段"),
        );
        true
    }

    pub(super) fn note_reference(&mut self, location: &PointerLocation, span: u64, what: &str) {
        self.references
            .entry((location.device, location.slot, span))
            .or_insert_with(|| what.to_string());
    }

    /// 开始走一版（根环里的一条根，或由记录施加出来的一版）：这一版引用过哪些落点从空的记起。
    pub(super) fn begin_walking_a_version(&mut self) {
        self.placements_referenced_in_this_version.clear();
        self.walk_parent_unit = None;
        self.user_visible_tree_being_walked = None;
        self.user_visible_tree_of_each_placement_in_this_version
            .clear();
        self.user_visible_tree_roots_of_this_version.clear();
        self.kind_of_a_tree_table_entry_without_a_clause_in_this_version = None;
        self.central_mapping_of_this_version = CentralMappingLocationsOfAVersion::default();
        self.tree_sharing_in_the_walk = TreeSharingInTheWalk::OwnTreesOfTheVersion;
        self.snapshot_trees_being_destroyed_in_this_version.clear();
    }

    /// 一版走完：把它引用过的落点、树表里几棵用户可见树的根、走读断没断、它的实例表指针出生 txg 与谱系记录记下，留给 I-3.5 在全部版本走完之后
    /// 折成单元、逐对比（[`VersionPlacementsForTheAppearanceRule`]）。`walk_failures_before` 是走这一版之前走读失败记了几条；
    /// `instance_table_pointer_birth_txg` 取这一版走的那条实例表指针（[`instance_table_pointer_birth_txg_of`]）；
    /// `lineage_records` 取这一版走的那条树表指针下面的谱系树（[`lineage_records_of_a_version_for_the_appearance_rule`]）。
    pub(super) fn note_the_version_just_walked_for_the_appearance_rule(
        &mut self,
        instance: u32,
        checkpoint_txg: u64,
        walk_failures_before: usize,
        instance_table_pointer_birth_txg: Option<u64>,
        lineage_records: Result<Vec<LineageRecordOfTheScan>, DeletionScanGap>,
    ) {
        self.versions_walked_for_the_appearance_rule
            .push(VersionPlacementsForTheAppearanceRule {
                instance,
                checkpoint_txg,
                placements: self
                    .placements_referenced_in_this_version
                    .keys()
                    .copied()
                    .collect(),
                user_visible_tree_of_each_placement: std::mem::take(
                    &mut self.user_visible_tree_of_each_placement_in_this_version,
                ),
                user_visible_tree_roots: self.user_visible_tree_roots_of_this_version.clone(),
                references_are_complete: self.walk_failures.len() == walk_failures_before,
                kind_of_a_tree_table_entry_without_a_clause: self
                    .kind_of_a_tree_table_entry_without_a_clause_in_this_version,
                instance_table_pointer_birth_txg,
                lineage_records,
            });
    }

    /// 全部版本走完之后，每一版引用过的落点各带三样（[`PlacementOfTheAppearanceRule`]）：I-3.5 逐对比的输入。比的时候按落点认
    /// （第一次记下时按中央映射 key 折的那几个按折成的单元认），豁免三类的落点不按折叠表折：折叠表在后走的版本读回第 1 条那一份时还会改，按它比会把后一版新出现的落点
    /// 折进前一版引用过的单元里、漏判（C748（最终折叠表漏判引用区间的落点），例子 txg 0、1 两版指 (0, 1)、txg 2 那一版指 (1, 2)）；
    /// 折叠表只拿来取诞生代号，取的是全部版本走完之后的那一张（[`VersionPlacementsForTheAppearanceRule`] 说为什么不在每版走完时取）。
    pub(super) fn versions_folded_into_units_by_the_final_fold_table(
        &self,
    ) -> Vec<VersionReferencesForTheAppearanceRule> {
        self.versions_walked_for_the_appearance_rule
            .iter()
            .map(|version| VersionReferencesForTheAppearanceRule {
                instance: version.instance,
                checkpoint_txg: version.checkpoint_txg,
                placements: version
                    .placements
                    .iter()
                    .map(|placement| {
                        let unit = *self.unit_of_each_placement.get(placement).expect(
                            "这一版引用过的每个落点都经 `note_references_of_a_pointer_followed_by_the_walk` 记过（这一版或先走的那一版），折叠表只改不删",
                        );
                        let identity_compared_across_versions = if self
                            .placements_first_noted_through_the_central_mapping
                            .contains(placement)
                        {
                            unit
                        } else {
                            *placement
                        };
                        (
                            *placement,
                            PlacementOfTheAppearanceRule {
                                identity_compared_across_versions,
                                unit_whose_birth_txg_it_takes: unit,
                                user_visible_tree: version
                                    .user_visible_tree_of_each_placement
                                    .get(placement)
                                    .copied(),
                            },
                        )
                    })
                    .collect(),
                user_visible_tree_roots: version.user_visible_tree_roots.clone(),
                references_are_complete: version.references_are_complete,
                kind_of_a_tree_table_entry_without_a_clause: version
                    .kind_of_a_tree_table_entry_without_a_clause,
                instance_table_pointer_birth_txg: version.instance_table_pointer_birth_txg,
                lineage_records: version.lineage_records.clone(),
            })
            .collect()
    }

    /// 往下走 `unit` 里的指针之前调：之后跟着指针记下的落点都算在 `unit` 下面（`placements_referenced_below_a_walked_unit`）。
    /// 交回之前那一个，走完用 [`Self::stop_walking_below`] 换回去；两次调用之间的循环不许提前返回。
    pub(super) fn start_walking_below(&mut self, unit: (u32, u64)) -> Option<(u32, u64)> {
        self.walk_parent_unit.replace(unit)
    }

    pub(super) fn stop_walking_below(&mut self, previous_parent_unit: Option<(u32, u64)>) {
        self.walk_parent_unit = previous_parent_unit;
    }

    /// I-5.1（物理范围不重叠） 的「同一版里第二次引用同一个落点」那一格：这一版引用过它就判红，没有就记下是谁第一次引用它。
    fn judge_a_placement_referenced_in_this_version(&mut self, placement: (u32, u64), what: &str) {
        match self.placements_referenced_in_this_version.get(&placement) {
            Some(first_reference_in_this_version) => {
                self.judgements.judge("I-5.1", false, || {
                    format!(
                        "盘 {} 槽 {}：被同一版引用了两次——先是 {first_reference_in_this_version}，又是 {what}",
                        placement.0, placement.1
                    )
                });
            }
            None => {
                self.judgements.judge("I-5.1", true, String::new);
                self.placements_referenced_in_this_version
                    .insert(placement, what.to_string());
            }
        }
    }

    /// 走读此刻在这一版的一棵用户可见树里往下走，就把 `placement` 记成落在那棵树里（I-3.5 (b) 的标签，只记这一版）；不在任何一棵里
    /// （快照那两棵树、别的种类的树、树表与实例表本身）不记。同一版里先记下的标签不换。
    fn note_the_user_visible_tree_of_a_placement_in_this_version(&mut self, placement: (u32, u64)) {
        if let Some(tree) = self.user_visible_tree_being_walked {
            self.user_visible_tree_of_each_placement_in_this_version
                .entry(placement)
                .or_insert(tree);
        }
    }

    /// 这一版又引用了一个落点：走这一版自己的树时照 [`Self::judge_a_placement_referenced_in_this_version`] 判；走快照那两棵树时
    /// （[`TreeSharingInTheWalk::SnapshotTreesSharingUnitsWithTheHead`]）不判，这一版还没记过才记下（与活头、与别的快照共享的单元不是第二次引用）。
    fn note_a_placement_referenced_in_this_version(&mut self, placement: (u32, u64), what: &str) {
        match self.tree_sharing_in_the_walk {
            TreeSharingInTheWalk::OwnTreesOfTheVersion => {
                self.judge_a_placement_referenced_in_this_version(placement, what);
            }
            TreeSharingInTheWalk::SnapshotTreesSharingUnitsWithTheHead => {
                self.placements_referenced_in_this_version
                    .entry(placement)
                    .or_insert_with(|| what.to_string());
            }
        }
    }

    /// 按位置条目读一个走读要往下走的单元（每条位置条目照判 I-2.1，同 [`read_referenced_unit`]），另交回「走过」（`visited_units`）要认的落点：
    /// 读回、自证过的那一份的落点，不是第 0 条位置条目的（同实例表链那一处 `placement_it_was_read_from` 的写法）。第 0 条位置条目指着这一遍先走过的
    /// 别的单元、第 1 条那一份完好时，按第 0 条认会把读回来的这一份当成走过、它的头与格式（I-1.3、出生身份、I-1.7）一个字节都不判。
    /// 交回的落点就是这个单元在走读里的身份：走过、诞生代号那张表、并进前一版、父单元（它下面记的落点）、inode 叶序与 I-3.5 的折叠表
    /// 一律按它认。
    /// 折叠表在这里改：读回的是第 1 条那一份、那个落点此刻还折在第 0 条落点那个单元上时，改记成它自己。调用方先记过引用
    /// （[`Self::note_references_of_a_pointer_followed_by_the_walk`]）再调它。
    ///
    /// 指针的提示与映射位置不同时（[`PointerView::hints_differ_from_the_locations_the_walk_takes`]，判决
    /// `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3）：先按提示读、只读不判，一份按指针里的整单元校验和对得上就用它，
    /// 认的落点是映射条目第 0 条位置（这个单元在走读里的身份落在映射位置上，提示那一槽已释放、已复用都不判）；两份都对不上再按映射位置读，
    /// 每条照判 I-2.1。
    pub(super) fn read_a_unit_to_walk_and_the_placement_it_was_read_from(
        &mut self,
        pointer: &PointerView,
        unit_bytes: usize,
        what: &str,
    ) -> Option<(Vec<u8>, (u32, u64))> {
        if pointer.hints_differ_from_the_locations_the_walk_takes() {
            if let Some(unit) = copy_at_a_location_hint(self.reader, pointer, unit_bytes) {
                return Some((
                    unit,
                    (pointer.locations[0].device, pointer.locations[0].slot),
                ));
            }
        }
        let (unit, location_it_was_read_from) =
            crate::image::read_referenced_unit_with_the_location_it_was_read_from(
                self.reader,
                &mut self.judgements,
                &pointer.locations,
                unit_bytes,
                what,
            )?;
        let placement_it_was_read_from = (
            location_it_was_read_from.device,
            location_it_was_read_from.slot,
        );
        let first_placement = (pointer.locations[0].device, pointer.locations[0].slot);
        if placement_it_was_read_from != first_placement
            && self.unit_of_each_placement.get(&placement_it_was_read_from)
                == Some(&first_placement)
        {
            self.unit_of_each_placement
                .insert(placement_it_was_read_from, placement_it_was_read_from);
        }
        Some((unit, placement_it_was_read_from))
    }

    /// 一个单元别的版本已经走过（`visited_units` 命中），这一版又引用它：不重走，把它下面的落点（孩子、孩子的孩子……，
    /// 按 `placements_referenced_below_a_walked_unit` 那张表）逐个并进这一版的集合再比交叉——老一版在共享子树之外又指了子树里面的
    /// 某个单元，就在这里红（实审 B1 留下的缺口）。一个单元的孩子只展开一次（表里有环时也走得完：每个单元至多进一次 `expanded`）。
    ///
    /// 走快照那两棵树时（[`TreeSharingInTheWalk::SnapshotTreesSharingUnitsWithTheHead`]）照样并，只是不判第二次引用：快照与活头、与别的快照
    /// 共享的单元下面的落点这一版已经记过（[`Self::note_a_placement_referenced_in_this_version`]）。
    ///
    /// 并进来的落点按这一版此刻在哪棵用户可见树里打 I-3.5 的标签（[`Self::note_the_user_visible_tree_of_a_placement_in_this_version`]）：
    /// 共享的单元在这一版里落在哪棵树，它下面的每个落点就落在哪棵树，与先走那一版从哪棵树（或快照那两棵树）走到它无关。
    fn merge_placements_below_a_unit_walked_by_an_earlier_version(
        &mut self,
        unit_walked_earlier: (u32, u64),
    ) {
        let mut expanded: BTreeSet<(u32, u64)> = BTreeSet::new();
        let mut pending = vec![unit_walked_earlier];
        while let Some(unit) = pending.pop() {
            if !expanded.insert(unit) {
                continue;
            }
            if let Some(birth_txg) = self.birth_txg_of_walked_units.get(&unit).copied() {
                self.judge_unit_birth_against_an_older_candidate_root(
                    birth_txg,
                    &format!("别的版本先走过的盘 {} 槽 {} 的单元", unit.0, unit.1),
                );
            }
            let Some(below) = self.placements_referenced_below_a_walked_unit.get(&unit) else {
                continue;
            };
            for (placement, what) in below.clone() {
                self.note_the_user_visible_tree_of_a_placement_in_this_version(placement);
                self.note_a_placement_referenced_in_this_version(placement, &what);
                pending.push(placement);
            }
        }
    }

    /// 走读跟着一条指针往下（树里的指针、根记录里的指针、实例表的链指针；中央映射条目不走这里）：每条位置条目记进引用集合，
    /// 并判 **I-5.1（物理范围不重叠）** 的「同一版里第二次引用同一个落点」那一格——两个 inode 的 extent 指着同一个数据单元、
    /// 两条子指针指着同一个节点，都在这里红。一条指针自己的两条位置条目落在同一个 (设备, 槽) 上是一个引用、不算两次
    /// （两条位置条目设备身份相同那一格归 I-2.5（位置条目按设备身份升序））。
    ///
    /// 记的是走读认的位置（`pointer` 已按 [`Self::located`] 认过：豁免三类之外是映射位置），不是提示（判决
    /// `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3「引用集合记映射位置」）。`authority` 给了类的，落点按中央映射 key 折成单元
    /// （`unit_of_each_central_mapping_key`）。走快照那两棵树时（[`TreeSharingInTheWalk::SnapshotTreesSharingUnitsWithTheHead`]）落点照记进引用并集，
    /// 不判同一版第二次引用；落点照样挂到父单元下面，这一版已经从活头记过的也挂。
    pub(super) fn note_references_of_a_pointer_followed_by_the_walk(
        &mut self,
        pointer: &PointerView,
        span: u64,
        authority: PositionAuthority,
        what: &str,
    ) {
        let first_placement = (pointer.locations[0].device, pointer.locations[0].slot);
        let unit = match authority {
            PositionAuthority::ThePointerItself => first_placement,
            PositionAuthority::TheCentralMapping(class) => *self
                .unit_of_each_central_mapping_key
                .entry(central_mapping_key_of_a_pointer(pointer, class))
                .or_insert(first_placement),
        };
        for (index, location) in pointer.locations.iter().enumerate() {
            self.note_reference(location, span, what);
            let placement = (location.device, location.slot);
            if let std::collections::btree_map::Entry::Vacant(first_note) =
                self.unit_of_each_placement.entry(placement)
            {
                first_note.insert(unit);
                match authority {
                    PositionAuthority::ThePointerItself => {}
                    PositionAuthority::TheCentralMapping(_) => {
                        self.placements_first_noted_through_the_central_mapping
                            .insert(placement);
                    }
                }
            }
            self.note_the_user_visible_tree_of_a_placement_in_this_version(placement);
            let is_an_earlier_location_entry_of_this_pointer = pointer.locations[..index]
                .iter()
                .any(|earlier| (earlier.device, earlier.slot) == placement);
            if is_an_earlier_location_entry_of_this_pointer {
                continue;
            }
            self.note_a_placement_referenced_in_this_version(placement, what);
            // 父单元下面挂它的每一个孩子，走快照那两棵树时这一版已经从活头记过的也挂：父单元在这一版只从快照那两棵树走得到，
            // 在先建快照的那几版里却是活头自己的单元，那几版不重走它、只按这张表并进它下面的落点，少挂一个，那几版的引用集合就少一个落点、
            // 走读又不记失败，I-3.5 把它当成后一版新出现的
            // 并进来的版本若也走快照那两棵树，那一支照 `note_a_placement_referenced_in_this_version` 不判第二次引用；
            // 是活头自己的树时，活头里同一个落点不会再有第二个父单元。
            if let Some(parent_unit) = self.walk_parent_unit {
                self.placements_referenced_below_a_walked_unit
                    .entry(parent_unit)
                    .or_default()
                    .push((placement, what.to_string()));
            }
        }
    }
}

#[cfg(test)]
mod tests;
