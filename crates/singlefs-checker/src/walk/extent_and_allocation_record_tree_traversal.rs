//! 遍历方向走打包容器、extent 树（上段节点、下段节点、数据指针）与分配记录树。

use super::*;

impl<'reader> Walk<'reader> {
    pub(super) fn judge_packed_container(
        &mut self,
        unit: &[u8],
        expected_type: u16,
        what: &str,
    ) -> Option<Vec<Vec<u8>>> {
        if !self.judge_unit_header(unit, 3, what) {
            return None;
        }
        let record_type = read_u16(unit, 51);
        let count = usize::from(read_u16(unit, 69));
        let width = usize::from(read_u16(unit, 71));
        let declared = usize::from(read_u16(unit, 8));
        let legal = record_type == expected_type
            && registered_record_width(record_type) == Some(width)
            && count * width <= declared
            && declared <= data_unit_bytes() - 136;
        self.judgements.judge("I-1.7", legal, || format!("{what}：类型 {record_type}、记录宽 {width}、记录数 {count}、声明长度 {declared} 不合法"));
        legal.then(|| {
            unit[136..136 + count * width]
                .chunks(width.max(1))
                .map(<[u8]>::to_vec)
                .collect()
        })
    }

    /// extent 树里指向一个数据单元的一条（下段叶的 extent 叶记录，或上段叶条目里内联的那一个）：数据指针 88 字节，
    /// 所在的 key 是 (0, `inode`, `offset`)。读那个数据单元、判头、出生身份、树 ID 与五元组。
    pub(super) fn walk_extent_data_pointer(
        &mut self,
        pointer_bytes: &[u8],
        inode: u64,
        offset: u64,
        tree: u64,
    ) {
        let raw_pointer = parse_data_pointer(pointer_bytes);
        judge_location_order(&mut self.judgements, &raw_pointer, "extent 记录的数据指针");
        judge_pointer_mac_and_nonce_are_zero(
            &mut self.judgements,
            &raw_pointer,
            "extent 记录的数据指针",
        );
        judge_pointer_encryption_and_compression_fields_take_the_first_version_values(
            &mut self.judgements,
            &raw_pointer,
            "extent 记录的数据指针",
        );
        judge_pointer_extent_offset_is_zero(
            &mut self.judgements,
            &raw_pointer,
            "extent 记录的数据指针",
        );
        let authority = PositionAuthority::TheCentralMapping(MappingKeyUnitClass::DataUnit);
        let pointer = self.located(&raw_pointer, authority);
        self.note_references_of_a_pointer_followed_by_the_walk(
            &pointer,
            2,
            authority,
            &format!("inode {inode} 偏移 {offset} 的数据单元"),
        );
        let Some((unit, placement_it_was_read_from)) = self
            .read_a_unit_to_walk_and_the_placement_it_was_read_from(
                &pointer,
                data_unit_bytes(),
                "数据单元",
            )
        else {
            self.walk_failures
                .push("数据单元两份都读不到对得上的".to_string());
            return;
        };
        if !self.visited_units.insert(placement_it_was_read_from) {
            self.note_a_unit_an_earlier_version_walked_referenced_again(
                &unit,
                &pointer,
                placement_it_was_read_from,
                ReferenceTheWalkFollowsToAUnit::DataUnitOfExtentKey {
                    tree,
                    inode,
                    offset,
                },
                "数据单元",
            );
            return;
        }
        if !self.judge_unit_header(&unit, 1, "数据单元") {
            return;
        }
        self.walked_units_whose_header_the_walk_could_use
            .insert(placement_it_was_read_from);
        self.judge_birth_identity_of_a_referenced_unit(
            &unit,
            &pointer,
            placement_it_was_read_from,
            "数据单元",
        );
        self.judge_pointer_birth_tree_against_the_unit_header(&unit, &pointer, "数据单元");
        self.judge_a_data_unit_against_its_extent_key(&unit, tree, inode, offset);
        // I-9.10 只拿活头的数据单元对活头的 inode 记录：快照的树里同一个 inode 号可以是删过、重发之后的另一个对象
        // （[`TreeSharingInTheWalk::SnapshotTreesSharingUnitsWithTheHead`]）。
        match self.tree_sharing_in_the_walk {
            TreeSharingInTheWalk::OwnTreesOfTheVersion => self.data_unit_objects.push((
                inode,
                read_u64(&unit, 59),
                format!("inode {inode} 偏移 {offset} 的数据单元"),
            )),
            TreeSharingInTheWalk::SnapshotTreesSharingUnitsWithTheHead => {}
        }
    }

    /// 判一个按位置寻址的节点在树里的位置（I-1.1（块头自述逻辑地址）：索引节点的身份是树 ID + 层级 + key 区间，
    /// 按位置寻址的树的区间是这个节点按位置规定罩的那一段，D18（块里携带什么信息） 已定项 2）与这一层的条目宽（I-1.10）。
    /// 交回这一支还往不往下走。
    fn position_and_entry_width_hold(
        &mut self,
        view: &crate::IndexNodeView,
        level: u8,
        prescribed_range: &(Vec<u8>, Vec<u8>),
        entry_width: usize,
        what: &str,
    ) -> bool {
        if !self.judge_the_level_and_the_key_range_the_position_prescribes(
            view,
            level,
            prescribed_range,
            what,
        ) {
            self.walk_failures.push(format!(
                "{what}：层级不是它的位置规定的那一层，这一支走不下去"
            ));
            return false;
        }
        self.judgements
            .judge("I-1.10", view.entry_width == entry_width, || {
                format!(
                    "{what}：头里自述的条目宽 {} 不等于这一层的条目字段表宽度 {entry_width}",
                    view.entry_width
                )
            });
        view.entry_width == entry_width
    }

    /// 递归的上界：每一层下降层级减一，每个单元只走一次（`visited_units`）。
    pub(super) fn walk_allocation_record_node(
        &mut self,
        pointer_bytes: &[u8],
        cell: AllocationRecordTreeCell,
        level: u8,
        tree: u64,
        name: &str,
        pool_devices: &[u32],
    ) {
        let what = match cell {
            AllocationRecordTreeCell::Root => format!("{name}（树 {tree}）的根"),
            AllocationRecordTreeCell::BelowTheRoot {
                level: cell_level,
                device,
                index,
            } => format!("{name}（树 {tree}）层级 {cell_level} 盘 {device} 第 {index} 格的节点"),
        };
        let (view, node_placement) = match self.read_index_node(
            pointer_bytes,
            tree,
            KEY_SCHEMA_ALLOCATION,
            &what,
            KeyRangeReading::PrescribedByThePositionJudgedByTheCaller,
            EntryFieldTableRegistration::Registered,
            PositionAuthority::TheCentralMapping(MappingKeyUnitClass::IndexNode),
        ) {
            IndexNodeReading::WalkedFirstByThisVersion {
                view,
                placement_it_was_read_from,
            } => (view, placement_it_was_read_from),
            IndexNodeReading::WalkedByAnEarlierVersion {
                header_the_earlier_walk_could_use: Some(view),
                ..
            } => {
                self.judge_the_level_and_the_key_range_the_position_prescribes(
                    &view,
                    level,
                    &allocation_record_tree_cell_range(cell),
                    &what,
                );
                return;
            }
            IndexNodeReading::WalkedByAnEarlierVersion {
                header_the_earlier_walk_could_use: None,
                ..
            }
            | IndexNodeReading::NotWalkable => return,
        };
        let entry_width = if level == 0 {
            allocation_record_bytes()
        } else {
            usize::try_from(ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES).expect("96")
        };
        if !self.position_and_entry_width_hold(
            &view,
            level,
            &allocation_record_tree_cell_range(cell),
            entry_width,
            &what,
        ) {
            return;
        }
        if level == 0 {
            for entry in &view.entries {
                let record = parse_allocation_record(entry);
                self.judgements.judge(
                    "I-1.1",
                    allocation_record_fits_in_the_leaf(
                        cell,
                        record.device,
                        record.slot,
                        record.span_slots,
                    ),
                    || {
                        format!(
                            "{what}：盘 {} 槽 {} 跨 {} 的记录不落在这片叶按位置罩的那一段里，或末槽越过叶的末槽",
                            record.device, record.slot, record.span_slots
                        )
                    },
                );
            }
            return;
        }
        let key_width = KEY_SCHEMA_ALLOCATION.width();
        let mut previous_child: Option<AllocationRecordTreeCell> = None;
        let previous_parent_unit = self.start_walking_below(node_placement);
        for entry in &view.entries {
            let child = allocation_record_tree_child_of_entry_key(
                cell,
                level,
                &entry[..key_width],
                pool_devices,
            )
            .filter(|child| previous_child.is_none_or(|previous| previous < *child));
            self.judgements.judge("I-1.1", child.is_some(), || {
                format!("{what}：一条内部条目的 key 不是这个节点里一个孩子那一段的起点，或孩子不按位置严格递增")
            });
            let Some(child) = child else {
                self.walk_failures.push(format!(
                    "{what}：内部条目指不到一个合法的孩子，这一支走不下去"
                ));
                continue;
            };
            previous_child = Some(child);
            let child_pointer_bytes = &entry[key_width..key_width + node_pointer_bytes()];
            judge_location_order(
                &mut self.judgements,
                &parse_node_pointer(child_pointer_bytes),
                &format!("{what} 的子指针"),
            );
            self.walk_allocation_record_node(
                child_pointer_bytes,
                child,
                level - 1,
                tree,
                name,
                pool_devices,
            );
        }
        self.stop_walking_below(previous_parent_unit);
    }

    /// extent 树上段一个节点连同它下面（D8（核心索引结构） 已定项 14：上段按 inode 号按位置寻址）：`position` 是 (层级, 序号)，
    /// 根那一个（`None`）的层级取它自己头里的、序号 0。每个节点核层级、位置规定的那一段、条目宽（叶 113、内部 110）；叶条目核标签
    /// （0 到 4，标签 3、4 是「之后到 size 是洞」）、inode 号落在叶罩的那一段里，标签 1、3 走进那个文件的下段，
    /// 标签 2、4 走那个内联的数据单元（key (0, inode, 0)）。洞与单元数对不对得上 size 归删除态扫描（I-4.6）。
    pub(super) fn walk_extent_upper_node(
        &mut self,
        pointer_bytes: &[u8],
        position: Option<(u8, u64)>,
        tree: u64,
    ) {
        let what = match position {
            None => format!("extent 树（树 {tree}）的根"),
            Some((level, index)) => {
                format!("extent 树（树 {tree}）上段层级 {level} 第 {index} 格的节点")
            }
        };
        let (view, node_placement) = match self.read_index_node(
            pointer_bytes,
            tree,
            KEY_SCHEMA_EXTENT,
            &what,
            match position {
                None => {
                    KeyRangeReading::PrescribedByThePositionOfTheExtentUpperRootJudgedByTheCaller
                }
                Some(_) => KeyRangeReading::PrescribedByThePositionJudgedByTheCaller,
            },
            EntryFieldTableRegistration::Registered,
            PositionAuthority::TheCentralMapping(MappingKeyUnitClass::IndexNode),
        ) {
            IndexNodeReading::WalkedFirstByThisVersion {
                view,
                placement_it_was_read_from,
            } => (view, placement_it_was_read_from),
            IndexNodeReading::WalkedByAnEarlierVersion {
                header_the_earlier_walk_could_use: Some(view),
                ..
            } => {
                let (level, index) = position.unwrap_or((view.level, 0));
                self.judge_the_level_and_the_key_range_the_position_prescribes(
                    &view,
                    level,
                    &extent_upper_cell_range(level, index),
                    &what,
                );
                return;
            }
            IndexNodeReading::WalkedByAnEarlierVersion {
                header_the_earlier_walk_could_use: None,
                ..
            }
            | IndexNodeReading::NotWalkable => return,
        };
        let (level, index) = position.unwrap_or((view.level, 0));
        let entry_width = if level == 0 {
            usize::try_from(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES).expect("113")
        } else {
            usize::try_from(EXTENT_TREE_INTERNAL_ENTRY_BYTES).expect("110")
        };
        if !self.position_and_entry_width_hold(
            &view,
            level,
            &extent_upper_cell_range(level, index),
            entry_width,
            &what,
        ) {
            return;
        }
        let span = extent_upper_span_in_inodes(level);
        let first_inode = index.saturating_mul(span);
        let last_inode = first_inode.saturating_add(span - 1);
        let previous_parent_unit = self.start_walking_below(node_placement);
        if level == 0 {
            for entry in &view.entries {
                let entry_view = extent_upper_leaf_entry_view(entry);
                let inode_of_the_entry = match &entry_view {
                    ExtentUpperLeafEntryView::NoDataUnit { inode }
                    | ExtentUpperLeafEntryView::LowerSegmentRoot { inode, .. }
                    | ExtentUpperLeafEntryView::InlineDataUnit { inode, .. } => Some(*inode),
                    ExtentUpperLeafEntryView::Malformed => None,
                };
                self.judgements.judge(
                    "I-1.1",
                    inode_of_the_entry
                        .is_some_and(|inode| (first_inode..=last_inode).contains(&inode)),
                    || {
                        format!("{what}：一条上段叶条目解不开（标签不是 0 到 4 或该是零的字节不是零），或它的 inode 号不在这片叶罩的那一段里")
                    },
                );
                match entry_view {
                    ExtentUpperLeafEntryView::LowerSegmentRoot { inode, pointer, .. } => {
                        judge_location_order(
                            &mut self.judgements,
                            &parse_node_pointer(&pointer),
                            &format!("{what} 里 inode {inode} 的下段根指针"),
                        );
                        self.walk_extent_lower_node(&pointer, None, inode, tree);
                    }
                    ExtentUpperLeafEntryView::InlineDataUnit { inode, pointer, .. } => {
                        self.walk_extent_data_pointer(&pointer, inode, 0, tree);
                    }
                    ExtentUpperLeafEntryView::NoDataUnit { .. }
                    | ExtentUpperLeafEntryView::Malformed => {}
                }
            }
            self.stop_walking_below(previous_parent_unit);
            return;
        }
        let key_width = KEY_SCHEMA_EXTENT.width();
        let mut previous_child: Option<(u8, u64)> = None;
        for entry in &view.entries {
            let child = extent_upper_child_of_entry_key(level, index, &entry[..key_width])
                .filter(|child| previous_child.is_none_or(|previous| previous < *child));
            self.judgements.judge("I-1.1", child.is_some(), || {
                format!("{what}：一条内部条目的 key 不是这个节点里一个孩子那一段的起点，或孩子不按位置严格递增")
            });
            let Some(child) = child else {
                self.walk_failures.push(format!(
                    "{what}：内部条目指不到一个合法的孩子，这一支走不下去"
                ));
                continue;
            };
            previous_child = Some(child);
            let child_pointer_bytes = &entry[key_width..key_width + node_pointer_bytes()];
            judge_location_order(
                &mut self.judgements,
                &parse_node_pointer(child_pointer_bytes),
                &format!("{what} 的子指针"),
            );
            self.walk_extent_upper_node(child_pointer_bytes, Some(child), tree);
        }
        self.stop_walking_below(previous_parent_unit);
    }

    /// 一个文件 extent 树下段的一个节点连同它下面（按数据单元号按位置寻址）：根那一个（`None`）的层级取它自己头里的、序号 0。
    /// 每个节点核层级、位置规定的那一段、条目宽（叶 112、内部 110）；叶里每条 extent 叶记录的 inode 段是这个文件、offset 段是
    /// 单元号 × 净荷容量、落在叶罩的那一段里，再走它指的数据单元。
    pub(super) fn walk_extent_lower_node(
        &mut self,
        pointer_bytes: &[u8],
        position: Option<(u8, u64)>,
        inode: u64,
        tree: u64,
    ) {
        let what = match position {
            None => format!("extent 树（树 {tree}）inode {inode} 下段的根"),
            Some((level, index)) => {
                format!("extent 树（树 {tree}）inode {inode} 下段层级 {level} 第 {index} 格的节点")
            }
        };
        let (view, node_placement) = match self.read_index_node(
            pointer_bytes,
            tree,
            KEY_SCHEMA_EXTENT,
            &what,
            KeyRangeReading::PrescribedByThePositionJudgedByTheCaller,
            EntryFieldTableRegistration::Registered,
            PositionAuthority::TheCentralMapping(MappingKeyUnitClass::IndexNode),
        ) {
            IndexNodeReading::WalkedFirstByThisVersion {
                view,
                placement_it_was_read_from,
            } => (view, placement_it_was_read_from),
            IndexNodeReading::WalkedByAnEarlierVersion {
                header_the_earlier_walk_could_use: Some(view),
                ..
            } => {
                let (level, index) = position.unwrap_or((view.level, 0));
                self.judge_the_level_and_the_key_range_the_position_prescribes(
                    &view,
                    level,
                    &extent_lower_cell_range(inode, level, index),
                    &what,
                );
                return;
            }
            IndexNodeReading::WalkedByAnEarlierVersion {
                header_the_earlier_walk_could_use: None,
                ..
            }
            | IndexNodeReading::NotWalkable => return,
        };
        let (level, index) = position.unwrap_or((view.level, 0));
        let entry_width = if level == 0 {
            extent_leaf_record_bytes()
        } else {
            usize::try_from(EXTENT_TREE_INTERNAL_ENTRY_BYTES).expect("110")
        };
        if !self.position_and_entry_width_hold(
            &view,
            level,
            &extent_lower_cell_range(inode, level, index),
            entry_width,
            &what,
        ) {
            return;
        }
        let previous_parent_unit = self.start_walking_below(node_placement);
        if level == 0 {
            let span = extent_lower_span_in_data_units(0);
            let first_unit = index.saturating_mul(span);
            let last_unit = first_unit.saturating_add(span - 1);
            let payload = data_unit_payload_capacity_in_bytes();
            for record in &view.entries {
                let (record_inode, offset) = (read_u64(record, 8), read_u64(record, 16));
                self.judgements.judge(
                    "I-1.1",
                    read_u64(record, 0) == 0
                        && record_inode == inode
                        && offset.is_multiple_of(payload)
                        && (first_unit..=last_unit).contains(&(offset / payload)),
                    || {
                        format!("{what}：extent 叶记录的 key（inode {record_inode}、偏移 {offset}）不是 (0, 这个文件, 单元号 × 净荷容量)，或不落在这片叶罩的那一段里")
                    },
                );
                self.walk_extent_data_pointer(&record[24..112], record_inode, offset, tree);
            }
            self.stop_walking_below(previous_parent_unit);
            return;
        }
        let key_width = KEY_SCHEMA_EXTENT.width();
        let mut previous_child: Option<(u8, u64)> = None;
        for entry in &view.entries {
            let child = extent_lower_child_of_entry_key(inode, level, index, &entry[..key_width])
                .filter(|child| previous_child.is_none_or(|previous| previous < *child));
            self.judgements.judge("I-1.1", child.is_some(), || {
                format!("{what}：一条内部条目的 key 不是这个文件里一个孩子那一段的起点，或孩子不按位置严格递增")
            });
            let Some(child) = child else {
                self.walk_failures.push(format!(
                    "{what}：内部条目指不到一个合法的孩子，这一支走不下去"
                ));
                continue;
            };
            previous_child = Some(child);
            let child_pointer_bytes = &entry[key_width..key_width + node_pointer_bytes()];
            judge_location_order(
                &mut self.judgements,
                &parse_node_pointer(child_pointer_bytes),
                &format!("{what} 的子指针"),
            );
            self.walk_extent_lower_node(child_pointer_bytes, Some(child), inode, tree);
        }
        self.stop_walking_below(previous_parent_unit);
    }
}
