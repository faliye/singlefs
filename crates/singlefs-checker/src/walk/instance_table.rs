//! 实例表：遍历方向沿链走实例表页、判页里的记录形态与每一行，以及只读不判地读出一条根的实例表行。

use super::*;

/// 实例表里的一行（D18（块里携带什么信息） 已定项 11 的字段表：`kind` 1 + 实例代号 4 + 所选根的 `checkpoint_txg` 8 +
/// 属于该实例的最大已施加事务号 W 8 + flags 1 + 预留 66）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct InstanceTableRow {
    pub(super) instance: u32,
    /// 这一行记的 T：该实例被施加到哪一代为止（C113（扫描重建时多版单元的现行版本判定无输入） P2 的 `T_pub`）。
    pub(super) published_checkpoint_txg: u64,
    /// 这一行记的 W：属于该实例的、被这次重放施加的最大事务号。
    pub(super) applied_transaction_high_water_mark: u64,
}

/// 实例表一片末尾那条链指针记录（D18（块里携带什么信息） 已定项 11：`kind 1 | 有无下一片 1 | 位置指针 86（无下一片时清零占位）| 预留 0`）
/// 在 checker 这边的读法，按字段表另写一份，不用实现的解析。「有无下一片」的「有」按 1 读：条款只写了字段名，
/// 0 是 mkfs 那一片写的「无」（字节表七 m1），1 与拒收别的值是按字段名与同一张表里 flags 字节的口径读的。
#[derive(PartialEq)]
pub(super) enum ChainRecordView {
    LastPage,
    NextPage(PointerView),
    /// 记录宽不是 88、kind 不是 1、有无下一片不是 0 或 1、无下一片而位置指针不全零、有下一片而位置指针全零、
    /// 有下一片而位置指针头部有一段不是第一版的取值（[`pointer_head_fields_take_the_first_version_values`]）。
    Malformed,
}

/// 一条指针头部 extent 偏移及之前那几段——MAC 16 与 nonce 12、算法类型、压缩算法码、压后长度、extent 偏移——都是第一版的取值
/// （I-2.4 在走读跟随的指针上逐段判的那几格，`image::judge_pointer_mac_and_nonce_are_zero` 等三个判法读的同一组字段）。
/// 实例表链指针记录说「有下一片」时只在这几段都过时算合法：有一段不过是 I-3.14 第五形，这一片解不开（ K2-1 甲，
/// `research/prompts/m3-batch10o-code-r1-forks.md`；实现 `InstanceTableChainRecord::parse` 经 `NodePointer::read_with_head_fields_judgement_from`
/// 判同一组，D19（块指针的结构与宽度预算） 已定项 3、D9（加密） 已定项 14、C581）。
pub(super) fn pointer_head_fields_take_the_first_version_values(pointer: &PointerView) -> bool {
    let fields = pointer.encryption_and_compression_fields;
    pointer.is_mac_and_nonce_all_zero
        && fields.algorithm_type == 0
        && fields.compression_algorithm_code == 0
        && fields.compressed_length == 0
        && pointer.is_extent_offset_zero
}

/// 链指针记录里位置指针的偏移：kind 1 字节 + 有无下一片 1 字节。
const CHAIN_RECORD_POINTER_OFFSET: usize = 2;
/// 实例表记录宽（D18（块里携带什么信息） 已定项 11）：链指针记录与行记录同宽。
pub(super) const INSTANCE_TABLE_RECORD_BYTES: usize = 88;

pub(super) fn chain_record_view(record: &[u8]) -> ChainRecordView {
    if record.len() != INSTANCE_TABLE_RECORD_BYTES || record[0] != 1 {
        return ChainRecordView::Malformed;
    }
    let pointer = parse_node_pointer(&record[CHAIN_RECORD_POINTER_OFFSET..]);
    match record[1] {
        0 if pointer.all_zero => ChainRecordView::LastPage,
        1 if !pointer.all_zero && pointer_head_fields_take_the_first_version_values(&pointer) => {
            ChainRecordView::NextPage(pointer)
        }
        _flag_or_pointer_that_does_not_match => ChainRecordView::Malformed,
    }
}

/// 实例表一条记录的种类：首字节 `kind`（D18（块里携带什么信息） 已定项 11 只登记两种：0 行、1 链指针记录）。
/// 别的值没登记，留着原值好写进违例说明。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum InstanceTableRecordKind {
    Row,
    ChainRecord,
    Unregistered(u8),
}

impl InstanceTableRecordKind {
    pub(super) fn of_record(record: &[u8]) -> Self {
        match record
            .first()
            .copied()
            .expect("实例表记录宽 88：judge_packed_container 判过记录宽等于登记的宽")
        {
            0 => Self::Row,
            1 => Self::ChainRecord,
            unregistered_kind => Self::Unregistered(unregistered_kind),
        }
    }
}

/// 实例表行记录（`kind` 0）里 flags 字节的偏移：kind 1 + 实例代号 4 + 所选根的 checkpoint_txg 8 + W 8（D18（块里携带什么信息） 已定项 11）。
pub(super) const INSTANCE_TABLE_ROW_FLAGS_OFFSET: usize = 21;
/// 行记录 flags 第一版唯一合法的值（D18（块里携带什么信息） 已定项 11：「第一版恒 0、非 0 拒收」，bit0 退役也在内）。
pub(super) const INSTANCE_TABLE_ROW_FLAGS_OF_THE_FIRST_VERSION: u8 = 0;

/// 最新根的实例表沿链在哪一片停下、没走到末片：第 0 片（根记录直接持有的那一片）还是之后的某一片。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InstanceTablePageWhereTheChainStopped {
    FirstPageHeldByTheRootRecord,
    LaterPageReachedThroughTheChain,
}

/// 沿链走实例表时没走到末片的那一处出口（`Walk::walk_instance_table_chain` 里每一个 `break`，末片那一个除外）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InstanceTableChainExitBeforeTheLastPage {
    /// 两份都读不到对得上的。
    ReadsBackFromNeitherCopy,
    /// 读回的那一份落点这一遍已经走过（链指回了走过的片）。
    PlacementAlreadyWalked,
    /// 码 3 容器那几条（头、I-1.7）不成立，记录取不出来。
    ContainerRecordsAreIllegal,
    /// 身份四元组不是 (0, 4, 片序号, 0)（I-1.1）。
    IdentityIsNotThisPageOfTheChain,
    /// I-3.14 不成立（记录形态那三格，或最后一条不是合法的链指针记录）。
    RecordFormsDoNotDecode,
}

/// I-3.8（实例表行唯一且低于挂载根）在最新根的实例表没走到末片时报的不适用理由（C742（实例表没收全时行唯一的不适用理由没分清））：
/// 哪一片停下、在哪个出口停下各点名一句，不判已收到的前几片的行，
/// 也不落到 `Judgements::into_report` 的缺省理由「这个镜像上没有它判的对象」。
const fn instance_table_rows_not_judged_reason(
    page: InstanceTablePageWhereTheChainStopped,
    exit: InstanceTableChainExitBeforeTheLastPage,
) -> &'static str {
    use InstanceTableChainExitBeforeTheLastPage as Exit;
    use InstanceTablePageWhereTheChainStopped as Page;
    match (page, exit) {
        (Page::FirstPageHeldByTheRootRecord, Exit::ReadsBackFromNeitherCopy) => {
            "最新根的实例表第 0 片两份都读不到对得上的、行没收全：没有行可判"
        }
        (Page::FirstPageHeldByTheRootRecord, Exit::PlacementAlreadyWalked) => {
            "最新根的实例表第 0 片读回的落点这一遍已经走过、行没收全：没有行可判"
        }
        (Page::FirstPageHeldByTheRootRecord, Exit::ContainerRecordsAreIllegal) => {
            "最新根的实例表第 0 片容器记录不合法（头或 I-1.7 不成立）、行没收全：没有行可判"
        }
        (Page::FirstPageHeldByTheRootRecord, Exit::IdentityIsNotThisPageOfTheChain) => {
            "最新根的实例表第 0 片身份不对（不是链上第 0 片）、行没收全：没有行可判"
        }
        (Page::FirstPageHeldByTheRootRecord, Exit::RecordFormsDoNotDecode) => {
            "最新根的实例表那一片解不开（I-3.14 不成立）、行没收全：没有行可判"
        }
        (Page::LaterPageReachedThroughTheChain, Exit::ReadsBackFromNeitherCopy) => {
            "最新根的实例表第 0 片之后有一片两份都读不到对得上的、行没收全：不照前几片的行判"
        }
        (Page::LaterPageReachedThroughTheChain, Exit::PlacementAlreadyWalked) => {
            "最新根的实例表链指回了这一遍走过的片、没走到末片、行没收全：不照前几片的行判"
        }
        (Page::LaterPageReachedThroughTheChain, Exit::ContainerRecordsAreIllegal) => {
            "最新根的实例表第 0 片之后有一片容器记录不合法（头或 I-1.7 不成立）、行没收全：不照前几片的行判"
        }
        (Page::LaterPageReachedThroughTheChain, Exit::IdentityIsNotThisPageOfTheChain) => {
            "最新根的实例表第 0 片之后有一片身份不对（不是链上该到的那一片）、行没收全：不照前几片的行判"
        }
        (Page::LaterPageReachedThroughTheChain, Exit::RecordFormsDoNotDecode) => {
            "最新根的实例表第 0 片之后有一片解不开（I-3.14 不成立）、行没收全：不照前几片的行判"
        }
    }
}

/// 一片实例表的记录形态（I-3.14 那几格）合不合法。不合法的那一片解不开： D18（块里携带什么信息） 已定项 11 补
/// 「kind 不是 0 或 1 ⇒ 这一片解不开（表不可读）」，实现的 `InstanceTablePage::parse` 对「最后一条不是合法的链指针记录」「它前面有不是行的记录」
/// 「行的 flags 不是 0」都交 None；主走读照「片解不开」走（[`Walk::walk_instance_table_chain`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InstanceTablePageRecordForms {
    WellFormed,
    IllFormedSoThePageDoesNotDecode,
}

/// I-3.14（实例表每片记录形态合法）在一片上判三件，各一格：每条记录的 `kind` 只有 0 或 1；`kind` 1（链指针记录）不出现在最后一条之前；
/// `kind` 0 行的 flags 字节为 0。最后一条是不是合法的链指针记录是同一条不变量的第四格，在 `walk_instance_table_chain` 里判。
/// 交回三件是不是都成立：有一件不成立，这一片解不开（[`InstanceTablePageRecordForms`]）。
fn judge_instance_table_page_record_forms(
    judgements: &mut Judgements,
    records: &[Vec<u8>],
    what: &str,
) -> InstanceTablePageRecordForms {
    use crate::image::INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED;
    let kinds: Vec<InstanceTableRecordKind> = records
        .iter()
        .map(|record| InstanceTableRecordKind::of_record(record))
        .collect();
    let unregistered_kinds: Vec<(usize, u8)> = kinds
        .iter()
        .enumerate()
        .filter_map(|(position, kind)| match kind {
            InstanceTableRecordKind::Unregistered(raw_kind) => Some((position, *raw_kind)),
            InstanceTableRecordKind::Row | InstanceTableRecordKind::ChainRecord => None,
        })
        .collect();
    judgements.judge(
        INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
        unregistered_kinds.is_empty(),
        || {
            format!(
                "{what}：（第几条, kind）{unregistered_kinds:?} 的 kind 不是 0（行）或 1（链指针记录）：这一片解不开"
            )
        },
    );
    let last_position = records.len().saturating_sub(1);
    let chain_records_before_the_last: Vec<usize> = kinds
        .iter()
        .enumerate()
        .filter_map(|(position, kind)| match kind {
            InstanceTableRecordKind::ChainRecord if position < last_position => Some(position),
            InstanceTableRecordKind::ChainRecord
            | InstanceTableRecordKind::Row
            | InstanceTableRecordKind::Unregistered(_) => None,
        })
        .collect();
    judgements.judge(
        INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
        chain_records_before_the_last.is_empty(),
        || {
            format!(
                "{what}：第 {chain_records_before_the_last:?} 条是链指针记录，不在这一片（{} 条）的末尾",
                records.len()
            )
        },
    );
    let rows_with_flags_off_the_first_version: Vec<(usize, u8)> = records
        .iter()
        .zip(&kinds)
        .enumerate()
        .filter_map(|(position, (record, kind))| match kind {
            InstanceTableRecordKind::Row => {
                let flags = record[INSTANCE_TABLE_ROW_FLAGS_OFFSET];
                (flags != INSTANCE_TABLE_ROW_FLAGS_OF_THE_FIRST_VERSION)
                    .then_some((position, flags))
            }
            InstanceTableRecordKind::ChainRecord | InstanceTableRecordKind::Unregistered(_) => None,
        })
        .collect();
    judgements.judge(
        INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
        rows_with_flags_off_the_first_version.is_empty(),
        || {
            format!(
                "{what}：（第几条, flags）{rows_with_flags_off_the_first_version:?} 的行 flags 不是 0"
            )
        },
    );
    if unregistered_kinds.is_empty()
        && chain_records_before_the_last.is_empty()
        && rows_with_flags_off_the_first_version.is_empty()
    {
        InstanceTablePageRecordForms::WellFormed
    } else {
        InstanceTablePageRecordForms::IllFormedSoThePageDoesNotDecode
    }
}
impl<'reader> Walk<'reader> {
    /// 沿链走一张实例表（D18（块里携带什么信息） 已定项 11：根记录直接持有第 0 片，第 k 片末尾的链指针记录指着第 k + 1 片）。
    /// 每一片判 I-2.5（位置条目升序）、I-2.1（校验和）、码 3 容器那几条（`judge_packed_container`）、I-1.1（身份四元组 =
    /// (0, 4, 片序号, 0)：链指针指到了别的单元、链成了环，都在这一格红）与 I-3.14（实例表每片记录形态合法：最后一条是合法的链指针记录，
    /// 加 `judge_instance_table_page_record_forms` 那三件）；
    /// 挂载根那一张（`mount_root_instance_when_newest` 给了实例代号）走到末片才把各片的记录交给 `judge_instance_table_rows` 判行那一半，
    /// 没走到末片时 I-3.8 整条报不适用、理由点名停在哪一片哪个出口（[`instance_table_rows_not_judged_reason`]，C742）；
    /// 走到末片（链指针记录说「没有下一片」）时记下候选集滤得全（`abandoned_roots_filtering`）。
    /// 迭代上界：第 k 片要求容器号是 k，走过的单元不再走（`visited_units`，按读回的那一份的落点认），每一轮是盘上一个不同的单元。
    /// 提前出口，链都不再往下走：读不出、身份不对（被引用的单元坏了或换了人，记进走读失败；头用不了由 `judge_unit_header` 记）；
    /// 容器记录不合法、I-3.14 不成立（这一片自己写错了，只红 I-1.7 / I-3.14 那一格；这一片解不开，行不收）；走到已经走过的单元（前一条根走过这条链）。
    /// I-3.14 不成立的五形（kind 不是 0 或 1、片中间有链指针记录、行的 flags 不是 0、最后一条不是合法的链指针记录、它说有下一片而位置指针头部
    /// 有一段不是第一版的取值）一律当这一片解不开，与实现 `InstanceTablePage::parse` 同一个读法（ D18（块里携带什么信息）
    /// 已定项 11 补的那一句；第五形 K2-1 甲）。第五形那条指针不跟随，走读不往下走，它头部那几段 I-2.4 不判（走读不跟随的指针 I-2.4 不判）。
    pub(super) fn walk_instance_table_chain(
        &mut self,
        first_page_pointer: &PointerView,
        mount_root_instance_when_newest: Option<u32>,
    ) {
        use crate::image::INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED;
        let mut pointer = *first_page_pointer;
        let mut page_index: u64 = 0;
        let mut pages: Vec<Vec<Vec<u8>>> = Vec::new();
        // 第 0 片由根记录里的指针指着（外面那一层的走读父单元照旧）；第 k + 1 片由第 k 片末尾的链指针记录指着，记在第 k 片下面。
        let parent_unit_when_the_chain_starts = self.walk_parent_unit;
        // 没走到末片时停在哪一片、哪个出口（C742：最新根那一张任一片停下，I-3.8 整条报不适用并点名）；走到末片时是 None。
        let mut chain_stopped_before_the_last_page: Option<(
            InstanceTablePageWhereTheChainStopped,
            InstanceTableChainExitBeforeTheLastPage,
        )> = None;
        loop {
            let page_where_the_chain_would_stop = if page_index == 0 {
                InstanceTablePageWhereTheChainStopped::FirstPageHeldByTheRootRecord
            } else {
                InstanceTablePageWhereTheChainStopped::LaterPageReachedThroughTheChain
            };
            let (what, pointer_what) = if page_index == 0 {
                ("实例表单元".to_string(), "实例表指针".to_string())
            } else {
                (
                    format!("实例表第 {page_index} 片"),
                    format!("实例表第 {} 片的链指针", page_index - 1),
                )
            };
            judge_location_order(&mut self.judgements, &pointer, &pointer_what);
            judge_pointer_mac_and_nonce_are_zero(&mut self.judgements, &pointer, &pointer_what);
            judge_pointer_encryption_and_compression_fields_take_the_first_version_values(
                &mut self.judgements,
                &pointer,
                &pointer_what,
            );
            judge_pointer_extent_offset_is_zero(&mut self.judgements, &pointer, &pointer_what);
            self.note_references_of_a_pointer_followed_by_the_walk(
                &pointer,
                2,
                PositionAuthority::ThePointerItself,
                &what,
            );
            let Some((unit, placement_it_was_read_from)) = self
                .read_a_unit_to_walk_and_the_placement_it_was_read_from(
                    &pointer,
                    data_unit_bytes(),
                    &what,
                )
            else {
                self.walk_failures
                    .push(format!("{what}两份都读不到对得上的"));
                chain_stopped_before_the_last_page = Some((
                    page_where_the_chain_would_stop,
                    InstanceTableChainExitBeforeTheLastPage::ReadsBackFromNeitherCopy,
                ));
                break;
            };
            // 「走过」按读回、自证过的那一份的落点认：
            // 第 k 片指针的盘 0 位置条目指回走过的片、盘 1 那条完好时，按第 0 条位置条目认会把读到手的这一片当成走过、丢掉它的行。
            // 别处的键（父单元、诞生代号那张表、并进前一版的落点表）同一个落点（与走读别的单元同一个读法）。
            if !self.visited_units.insert(placement_it_was_read_from) {
                self.note_a_unit_an_earlier_version_walked_referenced_again(
                    &unit,
                    &pointer,
                    placement_it_was_read_from,
                    ReferenceTheWalkFollowsToAUnit::InstanceTablePage { page_index },
                    &what,
                );
                chain_stopped_before_the_last_page = Some((
                    page_where_the_chain_would_stop,
                    InstanceTableChainExitBeforeTheLastPage::PlacementAlreadyWalked,
                ));
                break;
            }
            self.walk_parent_unit = Some(placement_it_was_read_from);
            let Some(records) =
                self.judge_packed_container(&unit, PACKED_TYPE_INSTANCE_TABLE, &what)
            else {
                chain_stopped_before_the_last_page = Some((
                    page_where_the_chain_would_stop,
                    InstanceTableChainExitBeforeTheLastPage::ContainerRecordsAreIllegal,
                ));
                break;
            };
            self.walked_units_whose_header_the_walk_could_use
                .insert(placement_it_was_read_from);
            self.judge_pointer_birth_tree_against_the_unit_header(&unit, &pointer, &what);
            if let Some(birth_identity) = birth_identity_in_the_header(&unit) {
                self.birth_txg_of_walked_instance_table_pages.insert(
                    placement_it_was_read_from,
                    birth_identity.write_order.birth_txg(),
                );
                self.judge_birth_identity_against_the_pointer(birth_identity, &pointer, &what);
            }
            if !self.judge_the_identity_of_an_instance_table_page(&unit, page_index, &what) {
                self.walk_failures
                    .push(format!("{what}：指到的单元不是链上第 {page_index} 片"));
                chain_stopped_before_the_last_page = Some((
                    page_where_the_chain_would_stop,
                    InstanceTableChainExitBeforeTheLastPage::IdentityIsNotThisPageOfTheChain,
                ));
                break;
            }
            let record_forms =
                judge_instance_table_page_record_forms(&mut self.judgements, &records, &what);
            let chain = records
                .last()
                .map_or(ChainRecordView::Malformed, |last| chain_record_view(last));
            self.judgements.judge(
                INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
                chain != ChainRecordView::Malformed,
                || {
                    format!(
                        "{what}：最后一条不是合法的链指针记录（kind 1、有无下一片 0 或 1、无下一片时位置指针全零、有下一片时不全零且头部 MAC / nonce、算法类型、压缩算法码、压后长度、extent 偏移都是第一版的取值）"
                    )
                },
            );
            // I-3.14 不成立的一片解不开（D18（块里携带什么信息） 已定项 11；实现 `InstanceTablePage::parse` 同样交 None），
            // 照上面「容器记录不合法」那一处出口走：这一片的行不收、链不往下走、没走到末片（候选集滤不全，I-9.29 的门关）。
            // 不记走读失败——走读失败驱动 I-7.4 / I-4.8「引用的单元已被复用或抹头」，这里没有哪个被引用的单元坏了，是这一片自己的记录写错了。
            match record_forms {
                InstanceTablePageRecordForms::WellFormed => {}
                InstanceTablePageRecordForms::IllFormedSoThePageDoesNotDecode => {
                    chain_stopped_before_the_last_page = Some((
                        page_where_the_chain_would_stop,
                        InstanceTableChainExitBeforeTheLastPage::RecordFormsDoNotDecode,
                    ));
                    break;
                }
            }
            match chain {
                ChainRecordView::LastPage => {
                    pages.push(records);
                    if mount_root_instance_when_newest.is_some() {
                        self.abandoned_roots_filtering =
                            AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable;
                    }
                    break;
                }
                ChainRecordView::NextPage(next_page) => {
                    pages.push(records);
                    pointer = next_page;
                    page_index += 1;
                }
                // 链指针记录自相矛盾：上面 I-3.14 那一格已经红了，这一片同样解不开（行不收），链不往下走。
                ChainRecordView::Malformed => {
                    chain_stopped_before_the_last_page = Some((
                        page_where_the_chain_would_stop,
                        InstanceTableChainExitBeforeTheLastPage::RecordFormsDoNotDecode,
                    ));
                    break;
                }
            }
        }
        self.walk_parent_unit = parent_unit_when_the_chain_starts;
        if let Some(mount_root_instance) = mount_root_instance_when_newest {
            // C742：没走到末片就是行没收全，漏掉的那几片里重复的实例代号看不到，不照已收到的前几片判。
            match chain_stopped_before_the_last_page {
                None => self.judge_instance_table_rows(&pages, mount_root_instance),
                Some((page, exit)) => self
                    .judgements
                    .not_applicable("I-3.8", instance_table_rows_not_judged_reason(page, exit)),
            }
        }
    }

    /// 码 3 容器：I-1.7（记录数 × 记录宽 ≤ 声明长度 ≤ 32768 − 136、记录宽 = 登记的宽）。返回记录，用不了返回 None。
    /// I-3.8（实例表行唯一且低于挂载根）：整张表（链上各片的行接起来）里 `kind` 0 行按实例代号唯一、每行的实例代号 < 挂载根（最新根）的实例。
    /// 记录形态（`kind` 只有 0 或 1、链指针记录恰好在每一片末尾、行的 flags 为 0）归 I-3.14，在 `walk_instance_table_chain` 里每一片判；
    /// 这里只收 `kind` 0 的行，别的记录跳过。「行只在回收条件成立后删」对着一个镜像判不了。
    fn judge_instance_table_rows(&mut self, pages: &[Vec<Vec<u8>>], mount_root_instance: u32) {
        let mut instances: Vec<u32> = Vec::new();
        let mut unique = true;
        let mut below_mount_root = true;
        for records in pages {
            for row in records {
                match InstanceTableRecordKind::of_record(row) {
                    InstanceTableRecordKind::Row => {
                        let instance = u32::from_le_bytes(row[1..5].try_into().expect("4 字节"));
                        if instances.contains(&instance) {
                            unique = false;
                        }
                        instances.push(instance);
                        self.instance_table_rows.push(parse_instance_table_row(row));
                        if instance >= mount_root_instance {
                            below_mount_root = false;
                        }
                    }
                    InstanceTableRecordKind::ChainRecord
                    | InstanceTableRecordKind::Unregistered(_) => {}
                }
            }
        }
        self.judgements.judge(
            "I-3.8",
            unique && below_mount_root,
            || {
                format!(
                    "实例表（{} 片）：行的实例代号 {instances:?}（唯一 = {unique}，都低于挂载根实例 {mount_root_instance} = {below_mount_root}）",
                    pages.len()
                )
            },
        );
    }
}

/// 实例表一行（`kind` 0 的记录）按字段表解出来：主走读判行那一半（`judge_instance_table_rows`）与 I-7.9 读一条根自己的实例表共用。
fn parse_instance_table_row(row: &[u8]) -> InstanceTableRow {
    InstanceTableRow {
        instance: u32::from_le_bytes(row[1..5].try_into().expect("4 字节")),
        published_checkpoint_txg: u64::from_le_bytes(row[5..13].try_into().expect("8 字节")),
        applied_transaction_high_water_mark: u64::from_le_bytes(
            row[13..21].try_into().expect("8 字节"),
        ),
    }
}

/// 一条根自己指着的那张实例表的全部行，沿链读（D18（块里携带什么信息） 已定项 11：根记录直接持有第 0 片，第 k 片末尾的链指针记录指着第 k + 1 片）。
/// 只读不判：哪一片读不出（两份的整单元 CRC 都对不上位置条目）、解不开（头判不过、类不是码 3、记录类型不是 4、记录宽不是 88、
/// 记录数 × 记录宽超过声明长度或声明长度超过容量、身份不是 (0, 4, 片序号, 0)、链指针记录之前有不是行的记录、行的 flags 不是 0、
/// 最后一条不是合法的链指针记录，含它说有下一片而位置指针头部有一段不是第一版的取值）⇒ None，判定归主走读那一遍（I-2.1、I-1.7、I-1.1、I-3.14）。
/// I-3.14 的五形与主走读、与实现同一个读法：解不开。
/// 迭代上界：第 k 轮要求读到的单元身份里容器号是 k，两轮读到同一个单元就要它的容器号同时等于两个数 ⇒ 每一轮读的是盘上不同的单元，
/// 轮数不超过盘上装得下的 32 KiB 单元数。跨轮带的是下一片的指针、片序号与已经接起来的行；提前出口只有读不出、解不开。
pub(super) fn instance_table_rows_of_root_without_judging(
    reader: &dyn ImageReader,
    record: &[u8],
) -> Option<Vec<InstanceTableRow>> {
    let mut pointer = parse_node_pointer(&record[170..256]);
    let mut page_index: u64 = 0;
    let mut rows: Vec<InstanceTableRow> = Vec::new();
    loop {
        let unit = read_unit_without_judging(reader, &pointer, data_unit_bytes())?;
        if !matches!(check_unit(&unit), Ok(3)) {
            return None;
        }
        let identity = (
            read_u64(&unit, 43),
            read_u16(&unit, 51),
            read_u64(&unit, 53),
            read_u64(&unit, 61),
        );
        let record_count = usize::from(read_u16(&unit, 69));
        let record_width = usize::from(read_u16(&unit, 71));
        let declared_length = usize::from(read_u16(&unit, 8));
        if identity != (0, PACKED_TYPE_INSTANCE_TABLE, page_index, 0)
            || record_width != INSTANCE_TABLE_RECORD_BYTES
            || record_count * record_width > declared_length
            || declared_length > data_unit_bytes() - 136
        {
            return None;
        }
        let records: Vec<&[u8]> = unit[136..136 + record_count * record_width]
            .chunks(record_width)
            .collect();
        let (chain_record, rows_of_this_page) = records.split_last()?;
        for row in rows_of_this_page {
            if row[0] != 0
                || row[INSTANCE_TABLE_ROW_FLAGS_OFFSET]
                    != INSTANCE_TABLE_ROW_FLAGS_OF_THE_FIRST_VERSION
            {
                return None;
            }
            rows.push(parse_instance_table_row(row));
        }
        match chain_record_view(chain_record) {
            ChainRecordView::LastPage => return Some(rows),
            ChainRecordView::NextPage(next_page) => {
                pointer = next_page;
                page_index += 1;
            }
            ChainRecordView::Malformed => return None,
        }
    }
}

/// 按一张实例表，这条根在不在被抛弃的时间线上：有它那个实例的行 (i, Ti, Wi) 且它的 txg > Ti
/// （D23（journal 的角色与格式） 已定项 14 回退段的候选集规则）。
pub(super) fn abandoned_by_instance_table_rows(
    instance_table_rows: &[InstanceTableRow],
    root: &crate::RootView,
) -> bool {
    instance_table_rows.iter().any(|row| {
        row.instance == root.instance && root.checkpoint_txg > row.published_checkpoint_txg
    })
}

/// 一条根记录指着的那张实例表，沿链读出来的全部行（读法与 I-7.9 读一条根自己的实例表相同，只读不判）。
/// 给记录核对器（`singlefs_verification::crash::check_records_against`）按「恢复落到的那一版的实例表」判一次发布在不在被抛弃的时间线上用
/// （D13（验证路线） 已定项 7 的记录核对器；判据是 D23（journal 的角色与格式） 已定项 14 回退段的候选集规则）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstanceTableOfRootRecord {
    rows: Vec<InstanceTableRow>,
}

impl InstanceTableOfRootRecord {
    /// `root_record` 是一条根记录的字节（至少到实例表指针的末尾，偏移 256）。
    /// 链上哪一片读不出、解不开 ⇒ None（读法见 `instance_table_rows_of_root_without_judging`）。
    #[must_use]
    pub fn read(reader: &dyn ImageReader, root_record: &[u8]) -> Option<Self> {
        instance_table_rows_of_root_without_judging(reader, root_record).map(|rows| Self { rows })
    }

    /// (实例代号, checkpoint_txg) 那一版在这张表下被抛弃：有它那个实例的行 (i, Ti, Wi) 且它的 txg > Ti。
    #[must_use]
    pub fn abandons(&self, instance: u32, checkpoint_txg: u64) -> bool {
        self.rows
            .iter()
            .any(|row| row.instance == instance && checkpoint_txg > row.published_checkpoint_txg)
    }
}
