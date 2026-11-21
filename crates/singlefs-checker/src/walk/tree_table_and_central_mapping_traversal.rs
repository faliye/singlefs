//! 遍历方向从一条根走下去的上半段：单元头、索引节点、根记录、树表与中央映射根、树表条目与快照条目底下的树。

use super::*;

impl<'reader> Walk<'reader> {
    /// 解一个头已经判过能用的码 2 节点（`index_node_view`）；解不开记走读失败。解不开的理由是「条目宽 0 而条目数非 0」、
    /// 这棵树又登记了条目字段表时，再报 **I-1.10（码 2 条目宽等于字段表宽）**：宽 0 不是任何登记宽，而声明长度 0 过得了
    /// 「= 条目数 × 条目宽」，走读里按宽判 I-1.10 的那几处在解码之后，碰不到这一格。
    pub(super) fn index_node_view_judging_a_zero_entry_width(
        &mut self,
        unit: &[u8],
        entry_field_table_registration: EntryFieldTableRegistration,
        what: &str,
    ) -> Option<crate::IndexNodeView> {
        match index_node_view(unit) {
            Ok(view) => Some(view),
            Err(refusal) => {
                if refusal == Verdict::EntryWidthZeroWithEntries {
                    match entry_field_table_registration {
                        EntryFieldTableRegistration::Registered => {
                            self.judgements.judge("I-1.10", false, || {
                                format!(
                                    "{what}：头里自述的条目宽 0 而条目数不是 0，0 不是这棵树登记的条目字段表宽度"
                                )
                            });
                        }
                        EntryFieldTableRegistration::NotRegistered => {}
                    }
                }
                self.walk_failures.push(format!("{what} 的条目区解不开"));
                None
            }
        }
    }

    /// 读一个码 2 节点并判头、树 ID（I-1.3）与 key 区间（I-1.1）。`key_range_reading` 说内部节点的 key 区间怎么判：
    /// 首末两条条目的 key（树表、层级 0 的根等今天的写法），或子树覆盖区间（多层码 2 树、inode 树、容器树的内部节点，孩子读回来之后由走读那一方判）。
    /// 「走过」、诞生代号与并进前一版都按读回那一份的落点认（同 [`Self::read_a_unit_to_walk_and_the_placement_it_was_read_from`]），
    /// 「先走过」作为一种结局交回调用方（[`IndexNodeReading`]）。
    /// `authority`：树表单元与映射树自己的节点（自举豁免）按指针自己的位置读，别的码 2 节点按这一版中央映射认的位置（[`Self::located`]）；
    /// I-2.5 与指针头部那几格判的是指针自己的字节。
    #[allow(
        clippy::too_many_arguments,
        reason = "指针字节、树、key 形态、称呼、key 区间读法、条目字段表登记与位置以谁为准，各有名字与类型，不为凑短签名收成结构体"
    )]
    pub(super) fn read_index_node(
        &mut self,
        pointer_bytes: &[u8],
        expected_tree: u64,
        schema: crate::KeySchema,
        what: &str,
        key_range_reading: KeyRangeReading,
        entry_field_table_registration: EntryFieldTableRegistration,
        authority: PositionAuthority,
    ) -> IndexNodeReading {
        let pointer = parse_node_pointer(pointer_bytes);
        if pointer.all_zero {
            return IndexNodeReading::NotWalkable;
        }
        judge_location_order(&mut self.judgements, &pointer, what);
        judge_pointer_mac_and_nonce_are_zero(&mut self.judgements, &pointer, what);
        judge_pointer_encryption_and_compression_fields_take_the_first_version_values(
            &mut self.judgements,
            &pointer,
            what,
        );
        judge_pointer_extent_offset_is_zero(&mut self.judgements, &pointer, what);
        let pointer = self.located(&pointer, authority);
        self.note_references_of_a_pointer_followed_by_the_walk(&pointer, 1, authority, what);
        let Some((unit, placement_it_was_read_from)) = self
            .read_a_unit_to_walk_and_the_placement_it_was_read_from(&pointer, node_bytes(), what)
        else {
            self.walk_failures
                .push(format!("{what} 两份都读不到对得上的"));
            return IndexNodeReading::NotWalkable;
        };
        if !self.visited_units.insert(placement_it_was_read_from) {
            self.note_a_unit_an_earlier_version_walked_referenced_again(
                &unit,
                &pointer,
                placement_it_was_read_from,
                ReferenceTheWalkFollowsToAUnit::IndexNodeOfTree {
                    tree: expected_tree,
                },
                what,
            );
            return IndexNodeReading::WalkedByAnEarlierVersion {
                placement_it_was_read_from,
                header_the_earlier_walk_could_use: self
                    .view_of_a_node_an_earlier_version_walked(&unit, placement_it_was_read_from),
            };
        }
        if !self.judge_unit_header(&unit, 2, what) {
            return IndexNodeReading::NotWalkable;
        }
        self.walked_units_whose_header_the_walk_could_use
            .insert(placement_it_was_read_from);
        self.judge_birth_identity_of_a_referenced_unit(
            &unit,
            &pointer,
            placement_it_was_read_from,
            what,
        );
        let Some(view) = self.index_node_view_judging_a_zero_entry_width(
            &unit,
            entry_field_table_registration,
            what,
        ) else {
            return IndexNodeReading::NotWalkable;
        };
        self.judge_the_tree_identifier_of_an_index_node(&view, expected_tree, what);
        let keys = match (key_range_reading, view.level) {
            (KeyRangeReading::FirstAndLastEntries, _)
            | (KeyRangeReading::SubtreeCoverageOfInternalNodesJudgedByTheCaller, 0)
            | (KeyRangeReading::SubtreeCoverageOfTheInodeTreeJudgedByTheCaller, 0)
            | (
                KeyRangeReading::SubtreeCoverageOfAContainerTreeAboveLevelOneJudgedByTheCaller,
                0 | 1,
            ) => check_index_node_keys(&view, schema),
            (KeyRangeReading::SubtreeCoverageOfTheInodeTreeJudgedByTheCaller, _)
            | (KeyRangeReading::SubtreeCoverageOfAContainerTreeAboveLevelOneJudgedByTheCaller, _) => {
                check_separators_of_a_node_whose_subtree_coverage_the_walk_judges(&view, schema)
            }
            (KeyRangeReading::PrescribedByThePositionOfTheExtentUpperRootJudgedByTheCaller, 0)
                if view.entries.is_empty() =>
            {
                if schema.width() == view.key_width {
                    Ok(())
                } else {
                    Err(Verdict::KeyWidthMismatch)
                }
            }
            (KeyRangeReading::SubtreeCoverageOfInternalNodesJudgedByTheCaller, _)
            | (KeyRangeReading::PrescribedByThePositionJudgedByTheCaller, _)
            | (KeyRangeReading::PrescribedByThePositionOfTheExtentUpperRootJudgedByTheCaller, _) => {
                check_internal_node_separators(&view, schema)
            }
        };
        self.judgements.judge("I-1.1", keys.is_ok(), || {
            format!("{what}：key 宽 / key 区间与条目不符（{keys:?}）")
        });
        IndexNodeReading::WalkedFirstByThisVersion {
            view,
            placement_it_was_read_from,
        }
    }

    pub(super) fn walk_root(&mut self, record: &[u8], is_newest: bool) {
        self.begin_walking_a_version();
        let checkpoint_txg = read_u64(record, ROOT_RECORD_CHECKPOINT_TXG_OFFSET);
        self.version_being_walked = if is_newest {
            VersionBeingWalked::NewestRoot { checkpoint_txg }
        } else {
            VersionBeingWalked::OlderCandidateRoot { checkpoint_txg }
        };
        // 实例表：根记录直接持有第 0 片（自证单元，I-1.3 豁免），后面各片由前一片的链指针记录指着。
        let first_page_pointer = parse_node_pointer(&record[170..256]);
        let mount_root_instance = u32::from_le_bytes(record[24..28].try_into().expect("4 字节"));
        self.walk_instance_table_chain(
            &first_page_pointer,
            is_newest.then_some(mount_root_instance),
        );
        // 树表 0 条那一版的分配记录树的根住根记录（C512（树表 0 条的一版上被换下的实例表记在哪没有条款））：
        // 那一版没有树表条目可放，也还没登记过任何树 ⇒ 那棵树的树 ID 是 0（由根记录独占持有，同树表单元与实例表单元）。
        // 按位置寻址、可以多层（D8（核心索引结构） 已定项 14）：整棵走下去。带文件的一版这一项恒全零，全零指针不走。
        if !parse_node_pointer(&record[342..428]).all_zero {
            self.walk_allocation_record_tree(&record[342..428], 0, "树表 0 条那一版的分配记录树");
        }
        // 中央映射树的根住根记录（D19（块指针的结构与宽度预算） 已定项 11）。
        let version_name = format!(
            "根（实例 {}、txg {}）",
            read_u32(record, ROOT_RECORD_INSTANCE_OFFSET),
            read_u64(record, ROOT_RECORD_CHECKPOINT_TXG_OFFSET)
        );
        self.walk_tree_table_and_central_mapping_root(
            &record[36..122],
            &record[256..342],
            &version_name,
            TreeTablePointerHolder::RootRecord,
        );
    }

    /// 一版的树表（树 ID 0）与它下面每一条树表条目，再加中央映射树的根：根环里的根从根记录里取这两条指针，
    /// 由记录施加出来的那一版从记录的新根段里取（两处都是 86 字节的码 2 指针）。树表读不出时映射根也不走（走读已经断了）；
    /// 树表别的版本先走过时条目不重走，映射根照样往下走（共享的就并进这一版）。
    /// `version_name` 是这一版的称呼（哪条根、或由哪条记录施加出来），I-9.16 判红时写进第一处违例。
    /// 树表指针整条全零时两类都照 core 的根记录读者跟过去（[`Walk::follow_an_all_zero_tree_table_pointer_like_the_root_record_reader_of_the_core`]），
    /// 映射根不走；`tree_table_pointer_holder` 只用来在说明里写明这条指针是从哪里读来的。
    pub(super) fn walk_tree_table_and_central_mapping_root(
        &mut self,
        tree_table_pointer: &[u8],
        mapping_root_pointer: &[u8],
        version_name: &str,
        tree_table_pointer_holder: TreeTablePointerHolder,
    ) {
        // 树表单元：不属于任何树（树 ID 0）。
        let tree_table_pointer_view = parse_node_pointer(tree_table_pointer);
        if tree_table_pointer_view.all_zero {
            self.follow_an_all_zero_tree_table_pointer_like_the_root_record_reader_of_the_core(
                &tree_table_pointer_view,
                version_name,
                tree_table_pointer_holder,
            );
            return;
        }
        // 这一版的中央映射先只读读出来：下面走树表条目时，豁免三类之外的
        // 每条指针按它认位置。映射树本身的判定在这一版树表条目走完之后（下面 `walk_code_two_subtree` 那一遍）。
        self.central_mapping_of_this_version =
            CentralMappingLocationsOfAVersion::read_without_judging(
                self.reader,
                &parse_node_pointer(mapping_root_pointer),
                &mut IndexNodeCache::new(),
            );
        let tree_table = self.read_index_node(
            tree_table_pointer,
            0,
            KEY_SCHEMA_TREE_TABLE,
            "树表单元",
            KeyRangeReading::FirstAndLastEntries,
            EntryFieldTableRegistration::NotRegistered,
            PositionAuthority::ThePointerItself,
        );
        match tree_table {
            IndexNodeReading::WalkedFirstByThisVersion {
                view: tree_table,
                placement_it_was_read_from: tree_table_placement,
            } => self.walk_the_entries_of_a_tree_table_walked_first_by_this_version(
                &tree_table,
                tree_table_placement,
                version_name,
            ),
            // 别的版本先走过这片树表（零单元发布、卸载与抬 F 的空发布照抄上一版的树表与映射根）：它下面的落点已经并进这一版
            // （`read_index_node` 里的 `merge_placements_below_a_unit_walked_by_an_earlier_version`），条目照先走那一版记下的判；
            // 映射根照样往下并（这一版也引用它，I-3.5 比相邻两版的引用集合要它，`research/prompts/m3-birth-floor-r3-main-verification.md`）。
            // 「先走过」按读回那一份的落点认，由 `read_index_node` 交回（第 0 条那一份对不上、第 1 条那一份被先走那一版读过时也认得出）。
            IndexNodeReading::WalkedByAnEarlierVersion {
                placement_it_was_read_from: tree_table_placement,
                ..
            } => {
                self.take_the_user_visible_tree_labels_below_a_tree_table_walked_earlier(
                    tree_table_placement,
                );
                self.judge_the_tree_table_entries_an_earlier_version_walked(tree_table_placement);
            }
            // 树表读不出、头用不了：走读已经断了，映射根也不走。
            IndexNodeReading::NotWalkable => return,
        }
        // 中央映射树不进树表，引用它的是那条映射根指针（根记录里的，或由记录施加出来那一版的新根段里的）：
        // I-1.3（块头树 ID 一致） 的「实际引用它的树」按那条指针头部的出生树取（D19（块指针的结构与宽度预算） 已定项 7），
        // 与树表条目里的树 ID 对树表指着的根是同一个读法。号不写死映射树的号（mkfs 那条流上是 65540）：回退到树表 0 条的一版之后再发第一个文件版本，
        // 十二棵树从水位重新发号（C511（回退到无文件那一版之后诞生代怎么接））。
        // 映射树可以是多层（D8（核心索引结构） 已定项 11）：整棵走下去，每个节点按父条目核层级、区间与分隔 key（I-1.1）。
        // 叶的条目宽**只守走读要几个字节，不判「映射条目该有多宽」**：后者 D19（块指针的结构与宽度预算） 已定项 10 已定一律 55，
        // 但 I-1.10（码 2 条目宽等于字段表宽） 的射程里还没有中央映射树（`field_table_width_of_tree_kind` 不认种类 5），
        // 接进去欠在 C307（映射树两种 key 宽怎么装进一棵定宽 key 的树）。
        let mapping_tree = MultiLevelTreeInTheWalk {
            tree: parse_node_pointer(mapping_root_pointer).birth_tree,
            schema: KEY_SCHEMA_MAPPING,
            leaf_entry_width: EntryWidthInTheWalk::GuardedForTheWalkOnly {
                bytes_the_walk_needs: mapping_entry_bytes(),
            },
            internal_entry_width: EntryWidthInTheWalk::GuardedForTheWalkOnly {
                bytes_the_walk_needs: internal_entry_bytes(KEY_SCHEMA_MAPPING),
            },
            name: "中央映射树",
            position_authority: PositionAuthority::ThePointerItself,
        };
        let mut leaf_entries = Vec::new();
        self.walk_code_two_subtree(mapping_root_pointer, None, &mapping_tree, &mut leaf_entries);
        self.walk_central_mapping_entries(&leaf_entries);
    }

    /// 一版的树表指针整条全零：根环里自证过的根（
    /// 第五节第 1 条「甲 checker 判它损坏」），以及由记录造出来的两类版本——由记录施加出来的那几版、重放施加出来的那一版
    /// core 的根记录读者不把全零当特例，照它那两条位置条目（都是盘 0 槽 0）去读、读不出（可写挂载拒成
    /// `Recovery(UnitUnreadable { slot: SlotNumber(0) })`；施加记录之后拼出的根交给同一个读者）。checker 跟过去：两条位置条目照 I-2.1 判，
    /// 这一版的走读记一次失败（最新根红 I-7.2、I-4.8、I-7.4，别的候选根与由记录施加出来的那几版红 I-4.8、I-7.4，重放施加出来的那一版红 I-4.1），
    /// 不再读成「这一版没有树表」。
    /// 不判 I-2.5：那一行把整条全零的指针整条豁免于升序判定。
    fn follow_an_all_zero_tree_table_pointer_like_the_root_record_reader_of_the_core(
        &mut self,
        tree_table_pointer: &PointerView,
        version_name: &str,
        tree_table_pointer_holder: TreeTablePointerHolder,
    ) {
        let holder = match tree_table_pointer_holder {
            TreeTablePointerHolder::RootRecord => "根记录",
            TreeTablePointerHolder::NewRootSegmentOfTheRecord => "记录新根段",
        };
        let what = format!(
            "{version_name}的树表（{holder}里树表指针全零，照 core 的根记录读者读它那两条位置条目）"
        );
        let unit = read_referenced_unit(
            self.reader,
            &mut self.judgements,
            &tree_table_pointer.locations,
            node_bytes(),
            &what,
        );
        self.walk_failures.push(match unit {
            None => format!("{what} 两份都读不到对得上的"),
            Some(_) => format!("{what} 读回来的那一份不在单元区里，不是一张树表"),
        });
    }

    /// 这一版第一个走到的树表：判条目次序（I-9.16）与每一条的 previous_snapshot_txg（I-9.25），把条目里 I-9.25 与 I-3.5 读的几样记下
    /// （别的版本共享这片树表时照它判），再逐条往下走；走进用户可见树时给记下的落点打上那棵树的标签（I-3.5）。
    fn walk_the_entries_of_a_tree_table_walked_first_by_this_version(
        &mut self,
        tree_table: &crate::IndexNodeView,
        tree_table_placement: (u32, u64),
        version_name: &str,
    ) {
        judge_tree_table_entries_ordering(&mut self.judgements, &tree_table.entries, version_name);
        // I-9.27 只看同一张表里条目之间的关系、与哪一版无关：共享这片树表的别的版本不重判（与 I-9.16 同）。
        judge_tree_table_entry_head_identifiers(
            &mut self.judgements,
            &tree_table.entries,
            version_name,
        );
        let entries_of_the_walk: Vec<TreeTableEntryOfTheWalk> = tree_table
            .entries
            .iter()
            .filter_map(|entry| tree_table_entry_of_the_walk(entry))
            .collect();
        self.tree_table_entries_of_a_walked_tree_table
            .insert(tree_table_placement, entries_of_the_walk.clone());
        self.judge_previous_snapshot_txg_of_the_tree_table_entries(&entries_of_the_walk);
        self.note_the_first_tree_table_entry_without_a_clause(&entries_of_the_walk);
        self.user_visible_tree_roots_of_this_version = entries_of_the_walk
            .iter()
            .filter_map(|entry| entry.user_visible_root)
            .collect();
        self.snapshot_trees_being_destroyed_in_this_version =
            snapshot_trees_of_snapshots_being_destroyed(&tree_table.entries);
        let previous_parent_unit = self.start_walking_below(tree_table_placement);
        for entry in &tree_table.entries {
            self.user_visible_tree_being_walked = tree_table_entry_of_the_walk(entry)
                .and_then(|entry_of_the_walk| entry_of_the_walk.user_visible_root)
                .map(|root| root.tree);
            self.walk_tree_table_entry(entry);
            self.user_visible_tree_being_walked = None;
        }
        self.stop_walking_below(previous_parent_unit);
        // 标签只在上面逐条走条目时记（走读在一棵用户可见树里才记），这一版之前没记过：此刻这一张就是这片树表下面的全部标签。
        let labels_below_the_tree_table = self
            .user_visible_tree_of_each_placement_in_this_version
            .clone();
        self.user_visible_tree_of_each_placement_below_a_walked_tree_table
            .insert(tree_table_placement, labels_below_the_tree_table);
    }

    /// 别的版本先走过这片树表：它下面的落点已经并进这一版（并的那一刻走读不在任何一棵树里，没打标签），标签照第一个走到它的那一版记下的接上。
    fn take_the_user_visible_tree_labels_below_a_tree_table_walked_earlier(
        &mut self,
        tree_table_placement: (u32, u64),
    ) {
        let Some(labels) = self
            .user_visible_tree_of_each_placement_below_a_walked_tree_table
            .get(&tree_table_placement)
        else {
            return;
        };
        for (placement, tree) in labels {
            self.user_visible_tree_of_each_placement_in_this_version
                .entry(*placement)
                .or_insert(*tree);
        }
    }

    /// 中央映射树叶里的条目逐条走（D19（块指针的结构与宽度预算） 已定项 6 / 10：key 27 = 类标签 1 + 出生树 8 + 出生 txg 8 +
    /// 实例代号 4 + 尾段 6，value = 位置条目 14 × 2）。入参的条目宽由走叶的那一步判过 ≥ 55（`EntryWidthInTheWalk::GuardedForTheWalkOnly`），
    /// 这里按字段表的固定偏移切。每一条：
    /// ① 两条位置条目按升序判（I-2.5）；
    /// ② key 的类标签不在登记表里（码 0、4…255）判 **映射 key 与单元头相符**（`MAPPING_KEY_MATCHES_THE_UNIT_HEADER`新立、
    ///    编号待主 agent 写回 kb 时定），那个单元不读——不知道它多长，按哪个长度读都是猜；
    /// ③ 指的物理范围（跨度按 key 的类：码 2 一槽、码 1 / 码 3 两槽）记进引用集合，参与 I-3.1（已分配统计对得上） 与 I-5.1（物理范围不重叠）：
    ///    映射是解引用与释放判定的唯一入口（D19（块指针的结构与宽度预算） 已定项 5），它指的那一段占着空间。与树里指向同一单元的那条引用
    ///    同 (设备, 起点槽, 跨度)，记成一个；不进「同一版第二次引用」那一判（映射条目与树里的指针指同一个单元是条款定的两处引用）；
    /// ④ 两份各读一遍（I-2.1），再判映射 key 与单元头相符（实审 B1 暂记在 I-1.6 / I-1.2 下的两处搬到这一条，实审 B2）：
    ///    读出来的单元头里的类标签要等于 key 的类标签；头里的出生身份（诞生代号、写序实例、尾段）要等于 key 里的
    ///    （与 `judge_birth_identity_of_a_referenced_unit` 拿指针比同一个读法）；码 2 / 码 3 的尾段是出生序号 4 字节，其后 2 字节补零恒 0；
    ///    码 1 / 码 3 的 key 里的出生树要等于单元头偏移 43 的出生树（码 1 五元组的树、码 3 类身份段的出生树）。
    ///    码 2 的出生树不比：取哪个字段还开着（C289（码 2 的出生树取哪个字段没写））。
    fn walk_central_mapping_entries(&mut self, entries: &[Vec<u8>]) {
        for entry in entries {
            let locations_pointer: Vec<u8> =
                [vec![0u8; 50], entry[27..55].to_vec(), vec![0u8; 8]].concat();
            let view = parse_node_pointer(&locations_pointer);
            judge_location_order(&mut self.judgements, &view, "映射条目");
            let key_class_tag = entry[MAPPING_KEY_UNIT_CLASS_OFFSET];
            let unit_bytes = match MappingKeyUnitClass::of(key_class_tag) {
                MappingKeyUnitClass::DataUnit | MappingKeyUnitClass::PackedRecordUnit => {
                    data_unit_bytes()
                }
                MappingKeyUnitClass::IndexNode => node_bytes(),
                MappingKeyUnitClass::Unregistered(unregistered_tag) => {
                    self.judgements.judge(MAPPING_KEY_MATCHES_THE_UNIT_HEADER, false, || {
                        format!(
                            "映射条目：key 的类标签 {unregistered_tag} 不在登记表里（1 / 2 / 3），它指的单元不读"
                        )
                    });
                    continue;
                }
            };
            let span_in_slots = u64::try_from(unit_bytes).expect("单元字节数") / SLOT_BYTES;
            for location in &view.locations {
                self.note_reference(location, span_in_slots, "映射条目指的单元");
            }
            let Some(unit) = read_referenced_unit(
                self.reader,
                &mut self.judgements,
                &view.locations,
                unit_bytes,
                "映射条目指的单元",
            ) else {
                self.walk_failures
                    .push("映射条目指的单元两份都读不到对得上的".to_string());
                continue;
            };
            let header_class_tag = unit[6];
            self.judgements.judge(
                MAPPING_KEY_MATCHES_THE_UNIT_HEADER,
                header_class_tag == key_class_tag,
                || {
                    format!(
                        "映射条目：key 的类标签是 {key_class_tag}，它指的单元头里的类标签是 {header_class_tag}"
                    )
                },
            );
            if header_class_tag != key_class_tag {
                continue;
            }
            let identity = birth_identity_in_the_header(&unit)
                .expect("头里的类标签等于 key 的类标签、上面已判过在登记表里（1 / 2 / 3）");
            let key_birth_txg = read_u64(entry, MAPPING_KEY_BIRTH_TXG_OFFSET);
            let key_instance = read_u32(entry, MAPPING_KEY_INSTANCE_OFFSET);
            let key_tail = match identity.tail {
                TailOfTheBirthIdentity::TransactionNumber(_) => {
                    read_six_byte_unsigned(entry, MAPPING_KEY_TAIL_OFFSET)
                }
                TailOfTheBirthIdentity::BirthSequence(_) => {
                    u64::from(read_u32(entry, MAPPING_KEY_TAIL_OFFSET))
                }
            };
            let matches_the_key = identity.write_order.birth_txg() == key_birth_txg
                && identity.write_order.instance() == key_instance
                && identity.tail.birth_identity_tail_number() == key_tail;
            self.judgements
                .judge(MAPPING_KEY_MATCHES_THE_UNIT_HEADER, matches_the_key, || {
                    format!(
                        "映射条目：key 里的出生身份（诞生代号 {key_birth_txg}、实例 {key_instance}、尾段 {key_tail}）与它指的单元头里的（诞生代号 {}、写序实例 {}、{} {}）不符",
                        identity.write_order.birth_txg(),
                        identity.write_order.instance(),
                        identity.tail.name(),
                        identity.tail.birth_identity_tail_number()
                    )
                });
            match identity.tail {
                TailOfTheBirthIdentity::TransactionNumber(_) => {}
                TailOfTheBirthIdentity::BirthSequence(_) => {
                    let padding = &entry[MAPPING_KEY_TAIL_PADDING];
                    self.judgements.judge(
                        MAPPING_KEY_MATCHES_THE_UNIT_HEADER,
                        padding.iter().all(|byte| *byte == 0),
                        || {
                            format!(
                                "映射条目：码 {key_class_tag} 的 key 尾段在出生序号之后的补零 2 字节是 {padding:?}，不是 0"
                            )
                        },
                    );
                }
            }
            let key_birth_tree = read_u64(entry, MAPPING_KEY_BIRTH_TREE_OFFSET);
            match birth_tree_in_the_header_as_the_mapping_key_names_it(&unit)
                .expect("头里的类标签等于 key 的类标签、上面已判过在登记表里（1 / 2 / 3）")
            {
                BirthTreeInTheHeader::AtOffsetFortyThree(header_birth_tree) => {
                    self.judgements.judge(
                        MAPPING_KEY_MATCHES_THE_UNIT_HEADER,
                        header_birth_tree == key_birth_tree,
                        || {
                            format!(
                                "映射条目：码 {key_class_tag} 的 key 里的出生树 {key_birth_tree} 与它指的单元头里的出生树 {header_birth_tree} 不符"
                            )
                        },
                    );
                }
                BirthTreeInTheHeader::FieldStillOpenUnderC289 => {}
            }
        }
    }

    /// 多层码 2 树（记账树、中央映射树，D8（核心索引结构） 已定项 11）的一个节点连同它下面：节点照 `read_index_node` 判头、
    /// 出生身份（I-1.2 / I-4.2）、树 ID（I-1.3）与自己的 key（I-1.1：叶的区间贴紧首末条目、内部节点的分隔 key 严格递增），
    /// 再按引用它的父条目判它在树里的身份——**I-1.1（块头自述逻辑地址）**：索引节点的身份是树 ID + 层级 + key 区间
    /// （D18（块里携带什么信息） 已定项 2 的子树覆盖区间），全从块头自带的那几样读，不另存旁路表：
    /// ① 孩子头里的层级 = 父层级 − 1（层级 0 是叶）；
    /// ② 分隔 key_i ≤ 第 i 个孩子头里的最小 key，且 > 第 i − 1 个孩子头里的最大 key（它管的区间就是父条目给的那一段）；
    /// ③ 内部节点头里的区间 = [第一个孩子头里的最小 key, 最后一个孩子头里的最大 key]。
    /// 条目宽：叶按 `leaf_entry_width`、内部节点按 `internal_entry_width`（记账树、deadlist 树、logged_ops 树判 I-1.10，中央映射树只守走读要几个字节）。
    /// 叶里的条目按走到的次序（= key 升序）追加进 `leaf_entries`。
    ///
    /// 迭代与递归的上界：每一层下降层级减一（①不成立的孩子不往下走），每个单元只走一次（`visited_units`）。
    fn walk_code_two_subtree(
        &mut self,
        pointer_bytes: &[u8],
        expected_level: Option<u8>,
        tree: &MultiLevelTreeInTheWalk<'_>,
        leaf_entries: &mut Vec<Vec<u8>>,
    ) -> SubtreeInTheWalk {
        let pointer = parse_node_pointer(pointer_bytes);
        let what = match expected_level {
            None => format!("{}（树 {}）的根", tree.name, tree.tree),
            Some(level) => format!("{}（树 {}）层级 {level} 的节点", tree.name, tree.tree),
        };
        if pointer.all_zero {
            if expected_level.is_some() {
                self.walk_failures
                    .push(format!("{what}：父条目里的子指针全零"));
            }
            return SubtreeInTheWalk::NotWalkable;
        }
        let (view, node_placement) = match self.read_index_node(
            pointer_bytes,
            tree.tree,
            tree.schema,
            &what,
            KeyRangeReading::SubtreeCoverageOfInternalNodesJudgedByTheCaller,
            tree.leaf_entry_width.entry_field_table_registration(),
            tree.position_authority,
        ) {
            IndexNodeReading::WalkedFirstByThisVersion {
                view,
                placement_it_was_read_from,
            } => (view, placement_it_was_read_from),
            IndexNodeReading::WalkedByAnEarlierVersion {
                header_the_earlier_walk_could_use,
                ..
            } => {
                return self.subtree_an_earlier_version_walked_judged_against_the_parent_entry(
                    header_the_earlier_walk_could_use,
                    expected_level,
                    tree.schema,
                    &what,
                );
            }
            IndexNodeReading::NotWalkable => return SubtreeInTheWalk::NotWalkable,
        };
        // 头里自述的 key 宽是盘上读来的一字节：与这棵树的 key 形态不等时 `read_index_node` 已经按 I-1.1 判红，这里不往下走——
        // 下面按它切子指针（`entry[key_width..key_width + 86]`）、把头里的 key 交回父节点按形态逐字段读，宽了越过条目、窄了越过 key，
        // 都是当场越界（代码审阅 6c 第 6 条）。
        if view.key_width != tree.schema.width() {
            self.walk_failures.push(format!(
                "{what}：头里自述的 key 宽 {} 不是这棵树的 key 宽 {}，这一支走不下去",
                view.key_width,
                tree.schema.width()
            ));
            return SubtreeInTheWalk::NotWalkable;
        }
        if !self.judge_the_level_the_parent_entry_names(view.level, expected_level, &what) {
            self.walk_failures
                .push(format!("{what}：层级不是父层级减一，这一支走不下去"));
            return SubtreeInTheWalk::NotWalkable;
        }
        let entry_width_reading = if view.level == 0 {
            tree.leaf_entry_width
        } else {
            tree.internal_entry_width
        };
        if !self.entry_width_holds(&view, entry_width_reading, &what) {
            return SubtreeInTheWalk::NotWalkable;
        }
        if view.level == 0 {
            leaf_entries.extend(view.entries.iter().cloned());
            return SubtreeInTheWalk::Walked {
                smallest_key: view.smallest_key,
                largest_key: view.largest_key,
                is_complete: true,
            };
        }
        let key_width = view.key_width;
        let fields = |key: &[u8]| tree.schema.fields(key);
        let mut is_complete = true;
        let mut children: Vec<(Vec<u8>, SubtreeInTheWalk)> = Vec::with_capacity(view.entries.len());
        let previous_parent_unit = self.start_walking_below(node_placement);
        for entry in &view.entries {
            let separator_key = entry[..key_width].to_vec();
            let child_pointer_bytes = &entry[key_width..key_width + node_pointer_bytes()];
            judge_location_order(
                &mut self.judgements,
                &parse_node_pointer(child_pointer_bytes),
                &format!("{what} 的子指针"),
            );
            let child = self.walk_code_two_subtree(
                child_pointer_bytes,
                Some(view.level - 1),
                tree,
                leaf_entries,
            );
            match &child {
                SubtreeInTheWalk::Walked {
                    is_complete: child_is_complete,
                    ..
                } => is_complete &= *child_is_complete,
                SubtreeInTheWalk::AlreadyWalkedFromAnotherRoot { .. } => {}
                SubtreeInTheWalk::NotWalkable => is_complete = false,
            }
            children.push((separator_key, child));
        }
        self.stop_walking_below(previous_parent_unit);
        // ② 两条不等式：判读回来、层级对得上的孩子——这一遍第一次走到的，与别的根先走过的（C896：那时按别的版本的父条目判过，
        // 证明不了这一版的分隔 key 也对）。
        for (position, (separator_key, child)) in children.iter().enumerate() {
            let Some((smallest_key, _)) = child.key_range_in_the_header() else {
                continue;
            };
            self.judgements.judge(
                "I-1.1",
                fields(separator_key) <= fields(smallest_key),
                || format!("{what}：第 {position} 条的分隔 key 大于这个孩子头里的最小 key"),
            );
            if position == 0 {
                continue;
            }
            if let Some((_, largest_key)) = children[position - 1].1.key_range_in_the_header() {
                self.judgements
                    .judge("I-1.1", fields(separator_key) > fields(largest_key), || {
                        format!("{what}：第 {position} 条的分隔 key 不大于左邻孩子头里的最大 key")
                    });
            }
        }
        // ③ 头里的区间是子树覆盖区间（首末两个孩子是别的根先走过的也判，同 ②）。
        if let (Some((smallest_key, _)), Some((_, largest_key))) = (
            children
                .first()
                .and_then(|(_, child)| child.key_range_in_the_header()),
            children
                .last()
                .and_then(|(_, child)| child.key_range_in_the_header()),
        ) {
            let covers = smallest_key == view.smallest_key && largest_key == view.largest_key;
            self.judgements.judge("I-1.1", covers, || {
                format!("{what}：头里的 key 区间不是子树覆盖区间（第一个孩子的最小 key 到最后一个孩子的最大 key）")
            });
        }
        SubtreeInTheWalk::Walked {
            smallest_key: view.smallest_key,
            largest_key: view.largest_key,
            is_complete,
        }
    }

    /// 多层码 2 树里别的根先走过的节点（[`Self::walk_code_two_subtree`] 读到 `WalkedByAnEarlierVersion`）：不往下走，按这一版的父条目照判
    /// 层级 = 父层级 − 1（I-1.1），把头里的 key 区间交回父节点判两条不等式与覆盖区间（`.claude/kb/checks-owed.md` C896：审计
    /// `records/2026-10-24-singlefs-checker-审计复核.md` C01 一节「其他 schema、层级等约束存在相似早退边界」）。
    /// 头用不了（`header_the_earlier_walk_could_use` 是 `None`）、自述的 key 宽不是这棵树的（首次走读那一支同样不往下判）、层级对不上时
    /// 不交区间；命中不记走读失败（这一版的落点已经并进来，同 C01 的命中只判）。
    fn subtree_an_earlier_version_walked_judged_against_the_parent_entry(
        &mut self,
        header_the_earlier_walk_could_use: Option<crate::IndexNodeView>,
        expected_level: Option<u8>,
        schema: crate::KeySchema,
        what: &str,
    ) -> SubtreeInTheWalk {
        let Some(view) = header_the_earlier_walk_could_use else {
            return SubtreeInTheWalk::AlreadyWalkedFromAnotherRoot {
                key_range_in_its_header: None,
            };
        };
        if view.key_width != schema.width()
            || !self.judge_the_level_the_parent_entry_names(view.level, expected_level, what)
        {
            return SubtreeInTheWalk::AlreadyWalkedFromAnotherRoot {
                key_range_in_its_header: None,
            };
        }
        SubtreeInTheWalk::AlreadyWalkedFromAnotherRoot {
            key_range_in_its_header: Some((view.smallest_key, view.largest_key)),
        }
    }

    /// 一个多层码 2 树节点的条目宽：按 I-1.10 判（记账树、deadlist 树、logged_ops 树），或只守走读要几个字节（中央映射树）；不成立就不往下切。
    fn entry_width_holds(
        &mut self,
        view: &crate::IndexNodeView,
        reading: EntryWidthInTheWalk,
        what: &str,
    ) -> bool {
        match reading {
            EntryWidthInTheWalk::JudgedAsInvariantOneTen { field_table_bytes } => {
                let entry_width = view.entry_width;
                self.judgements
                    .judge("I-1.10", entry_width == field_table_bytes, || {
                        format!(
                            "{what}：头里自述的条目宽 {entry_width} 不等于这一层的条目字段表宽度 {field_table_bytes}"
                        )
                    });
                entry_width == field_table_bytes
            }
            EntryWidthInTheWalk::GuardedForTheWalkOnly {
                bytes_the_walk_needs,
            } => {
                if view.entry_width < bytes_the_walk_needs {
                    self.walk_failures.push(format!(
                        "{what}自述的条目宽 {} 小于走读要的 {bytes_the_walk_needs} 字节，这一支走不下去",
                        view.entry_width
                    ));
                    return false;
                }
                true
            }
        }
    }

    /// 树表的一条条目：条目宽是树表单元头里的一个**盘上字段**（`index_node_view` 只判了它 ≥ key 宽 8），而下面三步——
    /// 读树 ID（偏移 0）、读种类（偏移 10）、切根指针（`entry[14..100]`）——都按字段表的固定偏移切，窄一个字节就当场越界。
    /// 切之前先判一次，窄的如实记一条「走读断了」（I-4.8（近 K 代根校验和自洽） / I-7.4（近 K 代块未被复用） 由它说话），
    /// **不判违例**：I-1.10（码 2 条目宽等于字段表宽） 判的是「树表指着的那棵树的根节点」的条目宽、住在这两处切片之后，
    /// 罩不到树表自己（C504（树表条目宽在走读里无守卫，今天没坏法打得到），
    /// I-1.10（码 2 条目宽等于字段表宽） 的射程一个字不扩）。守卫与 `references_under_root` /
    /// `allocation_record_count_under_root` 那两处同款。
    fn walk_tree_table_entry(&mut self, entry: &[u8]) {
        if entry.len() < tree_table_entry_bytes() {
            self.walk_failures.push(format!(
                "树表单元自述的条目宽 {} 小于走读要的 {} 字节，树表条目这一支走不下去",
                entry.len(),
                tree_table_entry_bytes()
            ));
            return;
        }
        let tree = read_u64(entry, 0);
        let kind = read_u16(entry, 10);
        // I-7.18、I-7.19 认建账的发布：第一条记账树条目的根指针出生 txg（GPU 走读内核 w_walk_tree_table_entry 同一处记）。
        if kind == TREE_KIND_ACCOUNTING && self.accounting_tree_root_birth_txg.is_none() {
            let accounting_tree_root = parse_node_pointer(&entry[14..100]);
            if !accounting_tree_root.all_zero {
                self.accounting_tree_root_birth_txg = Some(accounting_tree_root.birth_txg);
            }
        }
        self.tree_identifiers_in_tables.insert(tree);
        let what = format!("树 {tree}（种类 {kind}）的根");
        if kind == TREE_KIND_SNAPSHOT_INODE_ROOT || kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT {
            self.walk_the_tree_of_a_snapshot_entry(entry, tree, kind, &what);
            return;
        }
        // 没有 key 形态的条目走读走不下去：这一版的引用没收全由 `note_the_first_tree_table_entry_without_a_clause` 记下，
        // 走到树表这一层时就记了（共享这片树表的版本不走到这里）。
        let Some(schema) = key_schema_for_tree_kind(kind) else {
            return;
        };
        if kind == TREE_KIND_INODE {
            let pointer = self.located(
                &parse_node_pointer(&entry[14..100]),
                PositionAuthority::TheCentralMapping(MappingKeyUnitClass::IndexNode),
            );
            if !pointer.all_zero {
                if let Some(bytes) = self.reader.read(
                    pointer.locations[0].device,
                    pointer.locations[0].slot * SLOT_BYTES,
                    node_bytes(),
                ) {
                    self.judgements.judge("I-9.1", bytes[6] == 2, || {
                        format!("树表指向的 inode 树根类标签是 {}，不是 2", bytes[6])
                    });
                }
            }
        }
        // 记账树可以是多层（D8（核心索引结构） 已定项 11）：整棵走下去，每个节点按父条目判层级、区间与分隔 key（I-1.1），
        // 条目宽按层判 I-1.10（叶 34、内部节点 108 = key 22 + 子指针 86）。记账行只取走全了的那一遍：有一个节点读不出、
        // 头用不了或条目宽不对，行就数不全，I-3.1 那几条按「这一版没有记账可比」报不适用，不拿半张账去比。
        if kind == TREE_KIND_ACCOUNTING {
            let accounting_tree = MultiLevelTreeInTheWalk {
                tree,
                schema,
                leaf_entry_width: EntryWidthInTheWalk::JudgedAsInvariantOneTen {
                    field_table_bytes: field_table_width_of_tree_kind(TREE_KIND_ACCOUNTING)
                        .expect("记账树登记了条目字段表"),
                },
                internal_entry_width: EntryWidthInTheWalk::JudgedAsInvariantOneTen {
                    field_table_bytes: internal_entry_bytes(schema),
                },
                name: "记账树",
                position_authority: PositionAuthority::TheCentralMapping(
                    MappingKeyUnitClass::IndexNode,
                ),
            };
            let mut rows = Vec::new();
            if let SubtreeInTheWalk::Walked {
                is_complete: true, ..
            } = self.walk_code_two_subtree(&entry[14..100], None, &accounting_tree, &mut rows)
            {
                self.accounting_seen = true;
                // I-3.13：记账代 = checkpoint 号（D16（发布语义） 已定项 6），每条根的记账树只装这条根那一代的行；只判走全了的树。
                // inode 号水位那一统计量的台阶行例外（D5（快照 / 空间记账机制） 已定项 2 射程「I-3.13 对台阶行同样不适用」）：
                // 那一统计量、不带设备维、代段早于这一版 txg 的行就是台阶行，跳过；它代段等于这一版 txg 的那一行与别的统计量照判。
                let accounting_version_txg = self.version_being_walked.checkpoint_txg();
                for row in &rows {
                    let row_generation = read_u64(row, ACCOUNTING_KEY_GENERATION_OFFSET);
                    let is_an_inode_number_watermark_step = read_u16(row, 0)
                        == STATISTIC_INODE_WATERMARK
                        && read_u32(row, ACCOUNTING_ROW_DEVICE_OFFSET)
                            == STATISTIC_NO_DEVICE_DIMENSION
                        && row_generation < accounting_version_txg;
                    if is_an_inode_number_watermark_step {
                        continue;
                    }
                    self.judgements.judge(
                        ACCOUNTING_ROWS_CARRY_THE_GENERATION_OF_THEIR_VERSION,
                        row_generation == accounting_version_txg,
                        || {
                            format!(
                                "记账树（树 {tree}）里统计量 {}、盘 {} 那一行的代段 {row_generation} 不是这一版的 txg {accounting_version_txg}",
                                read_u16(row, 0),
                                read_u32(row, 10)
                            )
                        },
                    );
                }
                for row in &rows {
                    self.accounting
                        .insert((read_u16(row, 0), read_u32(row, 10)), read_u64(row, 22));
                    if read_u16(row, 0) == STATISTIC_INODE_WATERMARK
                        && read_u32(row, ACCOUNTING_ROW_DEVICE_OFFSET)
                            == STATISTIC_NO_DEVICE_DIMENSION
                    {
                        self.inode_number_watermark_rows.push((
                            read_u64(row, ACCOUNTING_KEY_GENERATION_OFFSET),
                            read_u64(row, ACCOUNTING_ROW_VALUE_OFFSET),
                        ));
                    }
                }
            }
            return;
        }
        // deadlist 树是第三棵多层码 2 树：整棵走下去，每个节点按父条目判
        // 层级、区间与分隔 key（I-1.1），条目宽按 I-1.10 判（叶 43、内部节点 43 + 86，等于才往下走，同实现拒宽于字段表的条目；判决
        // `research/prompts/m3-close-w13-code-r1-main-verification.md` 第二节 K5 那一行），走到的节点都记进引用（I-3.1）；
        // 条目本身（被钉的单元）不在这里判，在 [`judge_snapshots_and_deadlists_of_the_versions`] 里每一版各判（I-3.5 ③ ④、I-3.6）。
        // 根指针全零（day-1 注册、一条条目都没有）时一步都不走。
        if kind == TREE_KIND_DEADLIST {
            let deadlist_tree = MultiLevelTreeInTheWalk {
                tree,
                schema,
                leaf_entry_width: EntryWidthInTheWalk::JudgedAsInvariantOneTen {
                    field_table_bytes: field_table_width_of_tree_kind(TREE_KIND_DEADLIST)
                        .expect("deadlist 树登记了条目字段表"),
                },
                internal_entry_width: EntryWidthInTheWalk::JudgedAsInvariantOneTen {
                    field_table_bytes: deadlist_internal_entry_field_table_bytes(),
                },
                name: "deadlist 树",
                position_authority: PositionAuthority::TheCentralMapping(
                    MappingKeyUnitClass::IndexNode,
                ),
            };
            let mut deadlist_entries_walked = Vec::new();
            self.walk_code_two_subtree(
                &entry[14..100],
                None,
                &deadlist_tree,
                &mut deadlist_entries_walked,
            );
            return;
        }
        // logged_ops 树：整棵按多层码 2 树走下去，
        // 每个节点按父条目判层级、区间与分隔 key（I-1.1），条目宽按 I-1.10 判（叶 44、内部节点 10 + 86），走到的节点都记进引用（I-3.1）。
        // 条目本身（意图）不在这里判，在 [`judge_compaction_intents_of_the_versions`] 里每一版各判（I-4.4、I-5.6）。根指针全零时一步都不走。
        if kind == TREE_KIND_LOGGED_OPERATIONS {
            let logged_operations_tree = MultiLevelTreeInTheWalk {
                tree,
                schema,
                leaf_entry_width: EntryWidthInTheWalk::JudgedAsInvariantOneTen {
                    field_table_bytes: logged_operations_entry_bytes(),
                },
                internal_entry_width: EntryWidthInTheWalk::JudgedAsInvariantOneTen {
                    field_table_bytes: internal_entry_bytes(schema),
                },
                name: "logged_ops 树",
                position_authority: PositionAuthority::TheCentralMapping(
                    MappingKeyUnitClass::IndexNode,
                ),
            };
            let mut logged_operations_walked = Vec::new();
            self.walk_code_two_subtree(
                &entry[14..100],
                None,
                &logged_operations_tree,
                &mut logged_operations_walked,
            );
            return;
        }
        // 分配记录树与 extent 树按 key 空间定形状（D8（核心索引结构） 已定项 14）：整棵按位置走下去，每个节点核它罩的就是它的位置规定的那一段。
        if kind == TREE_KIND_ALLOCATION {
            if !parse_node_pointer(&entry[14..100]).all_zero {
                self.walk_allocation_record_tree(&entry[14..100], tree, "分配记录树");
            }
            return;
        }
        if kind == TREE_KIND_EXTENT {
            if !parse_node_pointer(&entry[14..100]).all_zero {
                self.walk_extent_upper_node(&entry[14..100], None, tree);
            }
            return;
        }
        let IndexNodeReading::WalkedFirstByThisVersion {
            view: node,
            placement_it_was_read_from: node_placement,
        } = self.read_index_node(
            &entry[14..100],
            tree,
            schema,
            &what,
            key_range_reading_of_a_tree_walked_from_its_tree_table_entry(kind),
            match field_table_width_of_tree_kind(kind) {
                Some(_) => EntryFieldTableRegistration::Registered,
                None => EntryFieldTableRegistration::NotRegistered,
            },
            PositionAuthority::TheCentralMapping(MappingKeyUnitClass::IndexNode),
        )
        else {
            return;
        };
        // I-1.10：头里自述的条目宽等于这棵树登记的条目字段表宽度，**先于**按字段表解条目判
        // （与 I-1.7（打包容器合法与判定顺序） 判码 3 记录宽同一纪律）。不等就不往下解：下面每一支都按
        // 24 / 34 / 112 / 120 这些固定偏移切条目，窄一个字节就当场越界（普查 R13）。
        // 中央映射树（种类 5）走不到这里判：`field_table_width_of_tree_kind` 里没有它；条目宽已定 55（D19（块指针的结构与宽度预算） 已定项 10），
        // 把种类 5 接进这一判欠在 C307。
        if let Some(entry_field_table_bytes) = field_table_width_of_tree_kind(kind) {
            let entry_width = node.entry_width;
            self.judgements
                .judge("I-1.10", entry_width == entry_field_table_bytes, || {
                    format!(
                        "{what}：头里自述的条目宽 {entry_width} 不等于这棵树的条目字段表宽度 {entry_field_table_bytes}"
                    )
                });
            if entry_width != entry_field_table_bytes {
                return;
            }
        }
        if kind == TREE_KIND_INODE {
            self.walk_inode_root(&node, tree, node_placement, &what);
        }
        // 墓碑树、谱系树：码 2 根之下是码 3 容器（类型段墓碑 {0, 1}、谱系 {0, 5}，D18（块里携带什么信息） 已定项 24）。
        // 这里走到每片容器、判每条内部条目的身份（墓碑树 I-9.23、谱系树 I-9.17）、容器的头与容器格式（单元算进可达与已有统计）；
        // 记录本身的那几条（I-9.18–I-9.22、I-9.24、I-9.29 与 I-9.10 墓碑那一个等号）在删除态扫描里每一版各判。
        if kind == TREE_KIND_TOMBSTONE {
            self.walk_container_tree_root(
                &node,
                node_placement,
                tree,
                PACKED_TYPE_TOMBSTONE,
                TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
                "墓碑树",
            );
        }
        if kind == TREE_KIND_LINEAGE {
            self.walk_container_tree_root(
                &node,
                node_placement,
                tree,
                PACKED_TYPE_LINEAGE,
                LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
                "谱系树",
            );
        }
        // 退役树：形态照墓碑树，类型段 {0, 3}，
        // 每条内部条目按 I-9.31 判身份；记录那几条（I-9.32–I-9.35）在墓碑与谱系那一遍里每一版各判。
        if kind == TREE_KIND_RETIREMENT {
            self.walk_container_tree_root(
                &node,
                node_placement,
                tree,
                PACKED_TYPE_RETIREMENT,
                RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
                "退役树",
            );
        }
    }

    /// 一条活快照的条目：码 14 照 inode 树往下走、码 15 照 extent 树往下走
    /// 判快照独有的那些单元的结构——头、出生身份、树 ID、key、条目宽与 inode 树、extent 树
    /// 那几条（I-1.x、I-2.x、I-9.2、I-9.4 等），与活头、与别的快照共享的单元已经走过、不重报（[`TreeSharingInTheWalk::SnapshotTreesSharingUnitsWithTheHead`]）。
    /// 节点头里的树 ID 是建快照那一刻活头那棵树的号，不是这条条目自己的树号：I-1.3 的「实际引用它的树」取根指针头部的出生树
    /// （同一判决 S1「读快照指着的根按根指针头部的出生树比、不按条目自己的树号」）。
    /// 带销毁中记号的码 14 与头 ID 配它的码 15 不走：从写意图那一刻起它不再钉住块，删快照的批可以已经放掉、复用了它的单元（D5（快照 / 空间记账机制）
    /// 已定项 20 的例外句）。根指针全零的不走。
    fn walk_the_tree_of_a_snapshot_entry(
        &mut self,
        entry: &[u8],
        tree: u64,
        kind: u16,
        what: &str,
    ) {
        if self
            .snapshot_trees_being_destroyed_in_this_version
            .contains(&tree)
        {
            return;
        }
        let root_pointer = parse_node_pointer(&entry[14..100]);
        if root_pointer.all_zero {
            return;
        }
        let tree_the_nodes_belong_to = root_pointer.birth_tree;
        let tree_sharing_before = std::mem::replace(
            &mut self.tree_sharing_in_the_walk,
            TreeSharingInTheWalk::SnapshotTreesSharingUnitsWithTheHead,
        );
        let user_visible_tree_before = self.user_visible_tree_being_walked.take();
        if kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT {
            self.walk_extent_upper_node(&entry[14..100], None, tree_the_nodes_belong_to);
        } else if let IndexNodeReading::WalkedFirstByThisVersion {
            view: node,
            placement_it_was_read_from: node_placement,
        } = self.read_index_node(
            &entry[14..100],
            tree_the_nodes_belong_to,
            KEY_SCHEMA_INODE,
            what,
            KeyRangeReading::SubtreeCoverageOfTheInodeTreeJudgedByTheCaller,
            EntryFieldTableRegistration::Registered,
            PositionAuthority::TheCentralMapping(MappingKeyUnitClass::IndexNode),
        ) {
            let inode_tree_entry_bytes =
                field_table_width_of_tree_kind(TREE_KIND_INODE).expect("inode 树登记了条目字段表");
            let entry_width = node.entry_width;
            self.judgements
                .judge("I-1.10", entry_width == inode_tree_entry_bytes, || {
                    format!(
                        "{what}：头里自述的条目宽 {entry_width} 不等于 inode 树的条目字段表宽度 {inode_tree_entry_bytes}"
                    )
                });
            if entry_width == inode_tree_entry_bytes {
                self.walk_inode_root(&node, tree_the_nodes_belong_to, node_placement, what);
            }
        }
        self.user_visible_tree_being_walked = user_visible_tree_before;
        self.tree_sharing_in_the_walk = tree_sharing_before;
    }

    /// I-9.25：`previous_snapshot_txg` 只写在 inode 树那条条目与码 14（快照 inode 根）条目里，其余条目恒 0（0 = 不适用）；
    /// inode 树那条不大于这一版的 txg（D5（快照 / 空间记账机制） 已定项 9）。
    /// 树表里有码 14 条目时按前驱链判（码 14 的前驱 = 排在它前面那条的
    /// 诞生 txg，活头 inode 条目的前驱 = 最后一条的；D5 已定项 9「= 捕获那一刻活头的值」）：码 14 按（诞生 txg, 树 ID）排成链，
    /// 第一条的 `previous_snapshot_txg` 是 0、之后每条等于前一条的诞生 txg，inode 树那条等于最后一条的诞生 txg。
    /// 一条条目一判。走到的每一版（候选集里每条，含由记录施加出来的）拿自己的 txg 判自己树表里的每一条：树表被几版共享时每一版各判一次
    /// （[`Self::judge_the_tree_table_entries_an_earlier_version_walked`]），不只在第一个走到它的那一版判。
    fn judge_previous_snapshot_txg_of_the_tree_table_entries(
        &mut self,
        entries: &[TreeTableEntryOfTheWalk],
    ) {
        let version_txg = self.version_being_walked.checkpoint_txg();
        // 码 14 的前驱链：（诞生 txg, 树 ID）升序，同诞生 txg 按树 ID 升序。
        // 带「销毁中」记号的那一条在链上被跳过（写意图那次发布把它的前驱 P
        // 传给链上它的下一侧；D5（快照 / 空间记账机制） 已定项 12）：每条码 14 与活头的前驱 = 排在它前面、不带记号的最近那一条的诞生 txg，
        // 没有为 0；带记号的那一条自己的前驱不跟着改（它的前驱本来就是排在它前面那一条的，第一版删快照串行，链上至多一条带记号）。
        let mut snapshot_chain: Vec<(u64, u64, bool)> = entries
            .iter()
            .filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
            .map(|entry| {
                (
                    entry.birth_txg,
                    entry.tree,
                    entry.is_a_snapshot_being_destroyed,
                )
            })
            .collect();
        snapshot_chain.sort_unstable();
        let birth_txg_of_the_nearest_live_snapshot_among = |links: &[(u64, u64, bool)]| {
            links
                .iter()
                .rev()
                .find(|(_, _, is_a_snapshot_being_destroyed)| !is_a_snapshot_being_destroyed)
                .map(|(birth_txg, _, _)| *birth_txg)
        };
        let birth_txg_of_the_last_snapshot =
            birth_txg_of_the_nearest_live_snapshot_among(&snapshot_chain);
        for entry in entries {
            let TreeTableEntryOfTheWalk {
                tree,
                kind,
                previous_snapshot_txg,
                birth_txg,
                ..
            } = *entry;
            let (previous_snapshot_txg_holds, expectation) = match kind {
                TREE_KIND_INODE => match birth_txg_of_the_last_snapshot {
                    None => (
                        previous_snapshot_txg <= version_txg,
                        format!("inode 树的不大于这一版的 txg {version_txg}"),
                    ),
                    Some(last_snapshot_txg) => (
                        previous_snapshot_txg <= version_txg
                            && previous_snapshot_txg == last_snapshot_txg,
                        format!(
                            "树表里有快照时 inode 树的等于最后一个不带销毁中记号的快照（码 14）的诞生 txg {last_snapshot_txg}、且不大于这一版的 txg {version_txg}"
                        ),
                    ),
                },
                TREE_KIND_SNAPSHOT_INODE_ROOT => {
                    let position_in_the_chain = snapshot_chain
                        .iter()
                        .position(|(link_birth_txg, link_tree, _)| {
                            (*link_birth_txg, *link_tree) == (birth_txg, tree)
                        })
                        .expect("前驱链就是这张表里的码 14 条目排出来的：这一条在链上");
                    let predecessor_birth_txg = birth_txg_of_the_nearest_live_snapshot_among(
                        &snapshot_chain[..position_in_the_chain],
                    )
                    .unwrap_or(0);
                    (
                        previous_snapshot_txg == predecessor_birth_txg,
                        format!(
                            "码 14 的等于前驱链上排在它前面、不带销毁中记号的最近那一条的诞生 txg（没有是 0），这一条该是 {predecessor_birth_txg}"
                        ),
                    )
                }
                _ => (
                    previous_snapshot_txg == 0,
                    "不是 inode 树、也不是码 14 的条目恒 0".to_string(),
                ),
            };
            self.judgements.judge(
                PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES,
                previous_snapshot_txg_holds,
                || {
                    format!(
                        "树 {tree}（种类 {kind}）的树表条目 previous_snapshot_txg {previous_snapshot_txg}：{expectation}"
                    )
                },
            );
        }
    }

    /// 这一版树表里第一条「走到没有条款的形状」的条目的种类记下（I-3.5）：先走这片树表的那一版与共享它的版本都经这里，
    /// 共享的版本不重走条目，引用同样没收全。不进 `walk_failures`：I-7.2、I-4.8、I-7.4 判的是走读断没断，这一条由 I-9.16 ④ 判红。
    fn note_the_first_tree_table_entry_without_a_clause(
        &mut self,
        entries: &[TreeTableEntryOfTheWalk],
    ) {
        self.kind_of_a_tree_table_entry_without_a_clause_in_this_version =
            entries.iter().find_map(|entry| {
                tree_table_entry_is_a_shape_without_a_clause(
                    entry.kind,
                    entry.root_pointer_is_all_zero,
                )
                .then_some(entry.kind)
            });
    }

    /// 这一版的树表单元别的版本先走过（`visited_units`）：不重读盘，照先走那一版记下的条目判这一版的 I-9.25，
    /// 这一版树表里几棵用户可见树的根也照它取（I-3.5）。先走那一版头用不了、没记下条目的，这里没有对象。
    fn judge_the_tree_table_entries_an_earlier_version_walked(
        &mut self,
        tree_table_placement: (u32, u64),
    ) {
        let Some(entries) = self
            .tree_table_entries_of_a_walked_tree_table
            .get(&tree_table_placement)
            .cloned()
        else {
            return;
        };
        self.judge_previous_snapshot_txg_of_the_tree_table_entries(&entries);
        self.note_the_first_tree_table_entry_without_a_clause(&entries);
        self.user_visible_tree_roots_of_this_version = entries
            .iter()
            .filter_map(|entry| entry.user_visible_root)
            .collect();
    }
}

#[cfg(test)]
mod tests;
