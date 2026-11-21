//! 遍历方向走容器树（墓碑、谱系、退役）与 inode 树：内部节点、叶容器、分隔 key 与层级。

use super::*;

impl<'reader> Walk<'reader> {
    /// 墓碑树或谱系树的码 2 根（或根长高之后的内部节点）往下走：条目 120 字节（分隔 key 8 + 身份引用 26 + 子指针 86），
    /// 类型段 = `record_type` 的走到码 3 容器（判容器头、出生树、容器格式，单元记进这一版走到的落点），类型段 0 的走进下一层码 2 节点
    /// （层级 = 父层级 − 1），别的类型段判 `identity_invariant` 红、不往下走；孩子头里自证过的类标签与类型段说的那一类不符的同样判红、
    /// 不往下走（[`Self::judge_a_container_tree_entry_whose_child_is_of_another_class`]，孩子读多长按它头里的类标签定、不按类型段定）——
    /// 另有一份头里就是类型段那一类、按那一类的长度自证得过的除外：判红之后照类型段那一支走那一份（F-W，
    /// [`some_copy_of_the_child_verifies_as_the_class_the_entry_type_names`]）；
    /// 这两支不往下走的孩子每一份副本照判 I-2.1（[`Self::judge_every_copy_of_a_child_the_walk_does_not_go_into`]）。每条条目的身份按 `identity_invariant`（墓碑树 I-9.23、
    /// 谱系树 I-9.17）单独判定，身份不符不记走读失败。
    /// 用 checker 自己的读法，不调 `singlefs_core`（D13（验证路线） 已定项 5）。
    /// 递归的上界：每进一层类型段 0，层级减一（层级 0 的码 2 节点在这两棵树里没有条款定的形状，记走读失败）。
    ///
    /// 往下记的落点都记在 `node_placement` 这个节点下面（`placements_referenced_below_a_walked_unit`）：别的版本共享这个节点时
    /// 照这张表把它下面的容器并进那一版（I-5.1 与 I-3.5 都要），不记在外面那片树表下面。
    pub(super) fn walk_container_tree_root(
        &mut self,
        node: &crate::IndexNodeView,
        node_placement: (u32, u64),
        tree: u64,
        record_type: u16,
        identity_invariant: &'static str,
        name: &str,
    ) {
        let previous_parent_unit = self.start_walking_below(node_placement);
        self.walk_the_entries_of_a_container_tree_node(
            node,
            tree,
            record_type,
            identity_invariant,
            name,
        );
        self.stop_walking_below(previous_parent_unit);
    }

    /// [`Self::walk_container_tree_root`] 那一个节点的条目逐条走（循环里有 `continue`，所以与换走读父单元那两步分开写）。
    fn walk_the_entries_of_a_container_tree_node(
        &mut self,
        node: &crate::IndexNodeView,
        tree: u64,
        record_type: u16,
        identity_invariant: &'static str,
        name: &str,
    ) {
        for (entry_index, entry) in node.entries.iter().enumerate() {
            let entry_type = read_u16(entry, 16);
            let identity = (
                read_u64(entry, 8),
                entry_type,
                read_u64(entry, 18),
                read_u64(entry, 26),
            );
            let child = parse_node_pointer(&entry[34..120]);
            let what = format!("{name}内部条目的子指针");
            judge_location_order(&mut self.judgements, &child, &what);
            judge_pointer_mac_and_nonce_are_zero(&mut self.judgements, &child, &what);
            judge_pointer_extent_offset_is_zero(&mut self.judgements, &child, &what);
            judge_pointer_encryption_and_compression_fields_take_the_first_version_values(
                &mut self.judgements,
                &child,
                &what,
            );
            let class_the_entry_type_names =
                match ContainerTreeEntryType::of(entry_type, record_type) {
                    ContainerTreeEntryType::InternalNode => Some(MappingKeyUnitClass::IndexNode),
                    ContainerTreeEntryType::Container => {
                        Some(MappingKeyUnitClass::PackedRecordUnit)
                    }
                    ContainerTreeEntryType::Unregistered(_) => None,
                };
            // 孩子落在哪按类型段说的那一类查中央映射（查找路径给类，D19（块指针的结构与宽度预算） 已定项 8）；类型段不认得的没有类、照提示。
            let child_authority = class_the_entry_type_names.map_or(
                PositionAuthority::ThePointerItself,
                PositionAuthority::TheCentralMapping,
            );
            let child = self.located(&child, child_authority);
            if let Some(class_the_entry_type_names) = class_the_entry_type_names {
                if let Some(child_class) = child_unit_class_other_than_the_entry_type_names(
                    self.reader,
                    &child,
                    Some(class_the_entry_type_names),
                ) {
                    // 另一份头里就是类型段那一类、按那一类的长度自证得过（F-W，判决
                    // `research/prompts/m3-batch28-code-r1-main-verification.md` 第二节「K2 H-V 两份都看之后判错」那一行、第三节第 1 条）：
                    // 读者按类型段那一类的长度逐份读、用对得上的那一份，那一份的子树照类型段那一支走、照判（槽数按类型段那一类登记）；
                    // 条目那一条照判红。没有这样一份才不往下走。
                    if some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(
                        self.reader,
                        &child,
                        class_the_entry_type_names,
                    ) {
                        self.judgements.judge(identity_invariant, false, || {
                            format!(
                                "{name}内部条目类型段 {entry_type} 说孩子是 {class_the_entry_type_names:?}，孩子头里的类标签是 {child_class:?}（按那一类的长度读回、整单元校验和对得上）；另一份头里就是类型段那一类、按那一类的长度自证得过，照类型段那一支往下走那一份"
                            )
                        });
                    } else {
                        self.judge_a_container_tree_entry_whose_child_is_of_another_class(
                            &child,
                            entry_type,
                            class_the_entry_type_names,
                            child_class,
                            identity_invariant,
                            name,
                        );
                        continue;
                    }
                }
            }
            match ContainerTreeEntryType::of(entry_type, record_type) {
                ContainerTreeEntryType::InternalNode => {
                    // 「子单元是码 2」与 I-1.6（类标签登记且三处相等） 重合，照 I-9.2（条目身份与子头相符） 的先例留作合取
                    // （第二节 T7）：读不出的孩子这一小句不判，读不出归 I-2.1 与走读失败。
                    // 孩子的类只认按它自己那一类的长度整份自证过的副本（F-ID-b，判决
                    // `research/prompts/m3-batch31-code-r1-main-verification.md` 第二节 K2 N1 盘序翻面 A3 那一行、第四节）：头里是码 3、
                    // 只有前 16 KiB 与位置条目里的校验和对得上的那一份不算，不随它在哪块盘翻面。
                    // 两份都没自证过的当读不出，「子单元是码 2」这一小句不判。
                    let (birth_tree, _, container, container_birth) = identity;
                    let child_class = class_of_the_first_copy_verifying_at_the_length_of_its_class(
                        self.reader,
                        &child,
                    );
                    let entry_holds = birth_tree == 0
                        && container == 0
                        && container_birth == 0
                        && child_class.is_none_or(|class| class == MappingKeyUnitClass::IndexNode);
                    self.judgements.judge(identity_invariant, entry_holds, || {
                        format!(
                            "{name}内部条目类型段 0：孩子自证过的类 {child_class:?}（要码 2；两份都没自证过的不判），身份引用的出生树 / 容器号 / 出生代 {birth_tree} / {container} / {container_birth}（要三段为 0）"
                        )
                    });
                    self.walk_the_internal_node_below_a_container_tree_entry(
                        node,
                        entry_index,
                        &child,
                        tree,
                        record_type,
                        identity_invariant,
                        name,
                    );
                }
                ContainerTreeEntryType::Container => {
                    self.walk_the_container_below_a_container_tree_entry(
                        &child,
                        identity,
                        tree,
                        record_type,
                        identity_invariant,
                        name,
                    );
                }
                ContainerTreeEntryType::Unregistered(unregistered_type) => {
                    // 先登记引用再判红：不登记的话这个孩子占的槽不进引用并集，I-3.1（已分配统计对得上） 连带红（第二节 T7）。
                    // 孩子的形状由这一层定：层级 1 的节点下面是码 3 容器（两槽），层级 ≥ 2 的下面是码 2 节点（一槽）。
                    // 孩子不往下走，每一份副本照判 I-2.1。
                    let span_in_slots = if node.level >= 2 { 1 } else { 2 };
                    self.note_references_of_a_pointer_followed_by_the_walk(
                        &child,
                        span_in_slots,
                        PositionAuthority::ThePointerItself,
                        &what,
                    );
                    self.judge_every_copy_of_a_child_the_walk_does_not_go_into(&child, &what);
                    self.judgements.judge(identity_invariant, false, || {
                        format!("{name}内部条目的类型段 {unregistered_type} 不在 {{0, {record_type}}} 里")
                    });
                }
            }
        }
    }

    /// 墓碑树、谱系树里一条条目的类型段说孩子是一类（类型段 0 ⇒ 码 2，类型段 = 打包记录类型 ⇒ 码 3），孩子头里自证过的类标签是另一类：
    /// I-9.23 / I-9.17 字面「0 ⇒ 子单元是码 2 … 1（谱系 5）⇒ 子单元是码 3」不成立，按 `identity_invariant` 判红，
    /// 「身份不符单独判定，不再记走读失败」：不往下走、不记走读失败（只在没有一份按类型段那一类自证过时走到这里，F-W）。孩子按它自己那一类占的槽先登记引用（同类型段不认得那一支：
    /// 不登记的话它占的槽不进引用并集，I-3.1（已分配统计对得上） 连带红）。每一份副本照判 I-2.1
    /// （[`Self::judge_every_copy_of_a_child_the_walk_does_not_go_into`]：另一份校验和对不上的照红）。
    fn judge_a_container_tree_entry_whose_child_is_of_another_class(
        &mut self,
        child: &PointerView,
        entry_type: u16,
        class_the_entry_type_names: MappingKeyUnitClass,
        child_class: MappingKeyUnitClass,
        identity_invariant: &'static str,
        name: &str,
    ) {
        let span_in_slots = slots_spanned_by_a_unit_of_the_class(child_class)
            .expect("child_unit_class_other_than_the_entry_type_names 只交回有长度的登记码");
        let what = format!("{name}内部条目的子指针");
        self.note_references_of_a_pointer_followed_by_the_walk(
            child,
            span_in_slots,
            PositionAuthority::ThePointerItself,
            &what,
        );
        self.judge_every_copy_of_a_child_the_walk_does_not_go_into(child, &what);
        self.judgements.judge(identity_invariant, false, || {
            format!(
                "{name}内部条目类型段 {entry_type} 说孩子是 {class_the_entry_type_names:?}，孩子头里的类标签是 {child_class:?}（按那一类的长度读回、整单元校验和对得上）"
            )
        });
    }

    /// 走读不往下读的孩子（墓碑树、谱系树、inode 树内部条目里类型段不认得的，与孩子自证成类型段之外另一类的；墓碑树、谱系树层级 1 节点里
    /// 类型段 0 的）：每一份副本照判
    /// I-2.1（`crates/singlefs-checker/invariants/04-校验和.md` I-2.1「任一被引用的块，其校验和与内容匹配」；
    /// 第二节 H-J3a、H-J3b 两行与第三节第 1 条）。一份对不对得上按
    /// [`copy_matches_its_location_checksum_at_the_length_its_header_names`] 判；两份都对不上记走读失败（I-4.8、I-7.2、I-7.4 照旧跟着）。
    /// 只判这两样：不往下走，引用由调用方按它定的形状先登记。
    fn judge_every_copy_of_a_child_the_walk_does_not_go_into(
        &mut self,
        child: &PointerView,
        what: &str,
    ) {
        let mut some_copy_matches = false;
        for location in &child.locations {
            let matches = copy_matches_its_location_checksum_at_the_length_its_header_names(
                self.reader,
                location,
            );
            self.judgements.judge("I-2.1", matches, || {
                format!(
                    "{what}（走读不往下读的孩子）在盘 {} 槽 {} 的那一份按头里那一类的长度（没登记的 16 KiB、32 KiB 都试）与位置条目里的校验和对不上",
                    location.device, location.slot
                )
            });
            some_copy_matches |= matches;
        }
        if !some_copy_matches {
            self.walk_failures.push(format!(
                "{what}（走读不往下读的孩子）两份都与位置条目里的校验和对不上"
            ));
        }
    }

    /// 墓碑树、谱系树里类型段 0 那条条目下面的码 2 节点：层级要是父层级 − 1，条目宽 120 才往下走（[`Self::walk_container_tree_root`]）。
    /// 读回它时（这一遍第一次走到的，与别的根先走过、头用得了的，C896）判父节点头里的 key 区间是子树覆盖区间（I-1.1，D18（块里携带什么信息） 已定项 2）：第一条条目的孩子头里的最小 key
    /// 等于父头里的最小 key，最后一条条目的孩子头里的最大 key 等于父头里的最大 key（孩子自己的区间在读它的那一步判过，层级 1 的贴紧首末两条容器号）；
    /// 父或孩子自述的 key 宽不是容器号的 8 字节时不比（读节点那一步已经判 I-1.1 红）。
    /// 父节点层级 ≤ 1 时孩子不往下走、记走读失败；孩子每一份副本照判 I-2.1
    /// 引用照旧不登记。
    #[allow(
        clippy::too_many_arguments,
        reason = "条目、孩子指针与树的三样都是这一步要的输入，各有名字与类型，不为凑短签名收成结构体"
    )]
    pub(super) fn walk_the_internal_node_below_a_container_tree_entry(
        &mut self,
        node: &crate::IndexNodeView,
        entry_index: usize,
        child: &PointerView,
        tree: u64,
        record_type: u16,
        identity_invariant: &'static str,
        name: &str,
    ) {
        if node.level <= 1 {
            self.judge_every_copy_of_a_child_the_walk_does_not_go_into(
                child,
                &format!("{name}内部条目的子指针"),
            );
            self.walk_failures.push(format!(
                "{name}层级 {} 的节点有类型段 0 的条目：孩子会是层级 0 的码 2 节点，这两棵树里没有条款定的形状",
                node.level
            ));
            return;
        }
        let entry = &node.entries[entry_index];
        // 别的根先走过的孩子不往下走（`child_placement_walked_first_by_this_version` 是 `None`），这一版的父条目定的覆盖区间与层级照判
        // （`.claude/kb/checks-owed.md` C896）。
        let (child_node, child_placement_walked_first_by_this_version) = match self.read_index_node(
            &entry[34..120],
            tree,
            KEY_SCHEMA_CONTAINER_NUMBER,
            &format!("{name}根之下的内部节点"),
            KeyRangeReading::SubtreeCoverageOfAContainerTreeAboveLevelOneJudgedByTheCaller,
            EntryFieldTableRegistration::Registered,
            PositionAuthority::TheCentralMapping(MappingKeyUnitClass::IndexNode),
        ) {
            IndexNodeReading::WalkedFirstByThisVersion {
                view,
                placement_it_was_read_from,
            } => (view, Some(placement_it_was_read_from)),
            IndexNodeReading::WalkedByAnEarlierVersion {
                header_the_earlier_walk_could_use: Some(view),
                ..
            } => (view, None),
            IndexNodeReading::WalkedByAnEarlierVersion {
                header_the_earlier_walk_could_use: None,
                ..
            }
            | IndexNodeReading::NotWalkable => return,
        };
        let container_number_key_width = KEY_SCHEMA_CONTAINER_NUMBER.width();
        if node.key_width == container_number_key_width
            && child_node.key_width == container_number_key_width
        {
            if entry_index == 0 {
                self.judgements.judge(
                    "I-1.1",
                    node.smallest_key == child_node.smallest_key,
                    || {
                        format!(
                            "{name}层级 {} 的节点：头里的最小 key 不是第一个孩子头里的最小 key（子树覆盖区间）",
                            node.level
                        )
                    },
                );
            }
            if entry_index + 1 == node.entries.len() {
                self.judgements.judge(
                    "I-1.1",
                    node.largest_key == child_node.largest_key,
                    || {
                        format!(
                            "{name}层级 {} 的节点：头里的最大 key 不是最后一个孩子头里的最大 key（子树覆盖区间）",
                            node.level
                        )
                    },
                );
            }
        }
        self.judgements
            .judge("I-1.1", child_node.level + 1 == node.level, || {
                format!(
                    "{name}内部节点层级 {} 不是父层级 {} − 1",
                    child_node.level, node.level
                )
            });
        let Some(child_placement) = child_placement_walked_first_by_this_version else {
            return;
        };
        if child_node.level + 1 == node.level
            && child_node.entry_width == inode_internal_entry_bytes()
        {
            self.walk_container_tree_root(
                &child_node,
                child_placement,
                tree,
                record_type,
                identity_invariant,
                name,
            );
        }
    }

    /// 墓碑树、谱系树里类型段 = `record_type` 那条条目下面的码 3 容器：先登记引用、读容器，按 `identity_invariant` 判条目身份
    /// （容器头四段与身份引用逐字相等、子指针头部出生树相等、类标签 3 与打包记录类型重合 I-1.6 / I-1.7 的那几小句留作合取；
    /// 容器两份都读不出时只判子指针头部出生树那一小句）；
    /// 这一遍第一次走到时再判树 ID（I-1.3）、出生身份（I-1.2 / I-4.2）与容器格式（I-1.7）。
    fn walk_the_container_below_a_container_tree_entry(
        &mut self,
        child: &PointerView,
        identity: (u64, u16, u64, u64),
        tree: u64,
        record_type: u16,
        identity_invariant: &'static str,
        name: &str,
    ) {
        let what = format!("{name}内部条目的子指针");
        self.note_references_of_a_pointer_followed_by_the_walk(
            child,
            2,
            PositionAuthority::TheCentralMapping(MappingKeyUnitClass::PackedRecordUnit),
            &what,
        );
        let Some((unit, placement_it_was_read_from)) = self
            .read_a_unit_to_walk_and_the_placement_it_was_read_from(
                child,
                data_unit_bytes(),
                &what,
            )
        else {
            // 容器两份都读不出：「子指针头部出生树 == 身份引用出生树」只比父条目里的两段，照判（判决
            // `research/prompts/m3-batch26-code-r1-main-verification.md` 第二节 K2 H3 H-B 那一行、第三节第 1 条）；比容器头的那几小句不判。
            let identity_birth_tree = identity.0;
            self.judgements.judge(
                identity_invariant,
                child.birth_tree == identity_birth_tree,
                || {
                    format!(
                        "{name}内部条目类型段 {record_type}：容器两份都读不出（比容器头的那几小句不判），子指针头部的出生树 {} 不等于身份引用的出生树 {identity_birth_tree}",
                        child.birth_tree
                    )
                },
            );
            self.walk_failures
                .push(format!("{name}的容器两份都读不到对得上的"));
            return;
        };
        let header_identity_of_the_container = (
            read_u64(&unit, 43),
            read_u16(&unit, 51),
            read_u64(&unit, 53),
            read_u64(&unit, 61),
        );
        let container_entry_holds = unit[6] == 3
            && header_identity_of_the_container == identity
            && header_identity_of_the_container.1 == record_type
            && child.birth_tree == identity.0;
        self.judgements
            .judge(identity_invariant, container_entry_holds, || {
                format!(
                    "{name}内部条目的身份 {identity:?} 与容器头 {header_identity_of_the_container:?}（类 {}、子指针出生树 {}）不符",
                    unit[6], child.birth_tree
                )
            });
        let container_name = format!("{name}的容器");
        if !self.visited_units.insert(placement_it_was_read_from) {
            self.note_a_unit_an_earlier_version_walked_referenced_again(
                &unit,
                child,
                placement_it_was_read_from,
                ReferenceTheWalkFollowsToAUnit::PackedContainerOfTree { tree },
                &container_name,
            );
            return;
        }
        self.judge_the_birth_tree_of_a_packed_container(&unit, tree, &container_name);
        self.judge_birth_identity_of_a_referenced_unit(
            &unit,
            child,
            placement_it_was_read_from,
            &container_name,
        );
        // 容器格式（类标签、记录类型、记录宽与声明长度）按 I-1.7 那一套判；记录本身的不变量归删除态扫描（I-9.18 起那几条）。
        if self
            .judge_packed_container(&unit, record_type, &container_name)
            .is_some()
        {
            self.walked_units_whose_header_the_walk_could_use
                .insert(placement_it_was_read_from);
        }
    }

    /// I-9.1：inode 树根恒码 2（读到这里时它已经按码 2 判过头）；层级 ≥ 1 的根往下走（[`Self::walk_inode_internal_node`]），
    /// 走完按叶序判 I-9.4（容器号不超最小 key） 的第二、三句（[`Self::judge_container_numbers_along_the_leaf_order`]）。
    pub(super) fn walk_inode_root(
        &mut self,
        root: &crate::IndexNodeView,
        tree: u64,
        root_placement: (u32, u64),
        what: &str,
    ) {
        if root.level == 0 {
            return;
        }
        match self.tree_sharing_in_the_walk {
            TreeSharingInTheWalk::OwnTreesOfTheVersion => self.inode_tree_walked = true,
            TreeSharingInTheWalk::SnapshotTreesSharingUnitsWithTheHead => {}
        }
        let mut leaves_in_leaf_order: Vec<InodeLeafContainerInLeafOrder> = Vec::new();
        // 根下面叶到没到全，根自己那一判在 `walk_inode_internal_node` 里已经用过；这里只要叶序。
        self.walk_inode_internal_node(root, tree, root_placement, what, &mut leaves_in_leaf_order);
        self.judge_container_numbers_along_the_leaf_order(&leaves_in_leaf_order);
    }

    /// I-9.4（容器号不超最小 key） 的第二句：沿叶序（这一版的 inode 树从左到右）逐对比，容器号严格递增。原来的第三句（左容器的最大 key < 右容器号）
    /// 随 N1 删掉：左从右借之后右片保留身份，
    /// 它的容器号可以不大于左片的最大 key。第一句（记录 inode ≥ 容器号）与第四句（同一片容器里记录号沿记录序严格递增）在走叶容器时逐条判。
    /// 叶序里有别的版本已经走过的叶（缓存的容器号与 key 区间）；读不出的叶不在叶序里，它两边的叶直接相邻着比
    /// （合法镜像上容器号与 key 沿叶序单调，隔一片比照样成立）。
    fn judge_container_numbers_along_the_leaf_order(
        &mut self,
        leaves_in_leaf_order: &[InodeLeafContainerInLeafOrder],
    ) {
        for adjacent in leaves_in_leaf_order.windows(2) {
            let (left, right) = (adjacent[0], adjacent[1]);
            self.judgements
                .judge("I-9.4", left.container < right.container, || {
                    format!(
                        "沿叶序容器号不严格递增：容器 {} 之后是容器 {}",
                        left.container, right.container
                    )
                });
        }
    }

    /// inode 树的一个内部节点（根，或类型段 0 指着的码 2 节点）：条目逐条判 I-9.2（条目身份与子头相符） 并往下走——类型段 2 走到叶容器
    /// （[`Self::walk_inode_leaf_container`]），类型段 0 走进下一层内部节点（[`Self::walk_inode_internal_child`]），别的类型段判红、不往下走；
    /// 逐条记下「分隔 key 与这个孩子的 key 区间」，走完按 I-9.12（分隔 key 落在孩子区间之外） 判一遍，再按子树覆盖区间判这个节点头里的 key 区间
    /// （I-1.1，[`Self::judge_the_key_range_of_an_inode_internal_node_against_its_leaves`]），交回这一枝的叶是不是全都到得了。
    /// 它下面按叶序的叶容器追加进 `leaves_in_leaf_order`，也记进 `inode_leaf_order_below_a_walked_unit`（别的版本共享它时接上叶序）。
    /// 递归的上界：每进一层类型段 0，层级减一（[`Self::walk_inode_internal_child`] 判层级 = 父层级 − 1 且 > 0 才往下走）。
    pub(super) fn walk_inode_internal_node(
        &mut self,
        node: &crate::IndexNodeView,
        tree: u64,
        node_placement: (u32, u64),
        what: &str,
        leaves_in_leaf_order: &mut Vec<InodeLeafContainerInLeafOrder>,
    ) -> EveryLeafReached {
        let leaves_before_this_node = leaves_in_leaf_order.len();
        // 读不出孩子的那几条不进来，I-9.12 只判读得出的那些。
        let mut separator_and_child_range: Vec<SeparatorAndInodeChildRange> = Vec::new();
        // 下面每条条目的孩子是不是都到得了叶（跨轮携带：有一条没到就一直是 No；判子树覆盖区间要它）。
        let mut every_leaf_reached = EveryLeafReached::Yes;
        let previous_parent_unit = self.start_walking_below(node_placement);
        for entry in &node.entries {
            let record_type = read_u16(entry, 16);
            let entry_type = InodeInternalEntryType::of(record_type);
            let identity = (
                read_u64(entry, 8),
                record_type,
                read_u64(entry, 18),
                read_u64(entry, 26),
            );
            let child = parse_node_pointer(&entry[34..120]);
            judge_location_order(&mut self.judgements, &child, "inode 内部条目的子指针");
            judge_pointer_mac_and_nonce_are_zero(
                &mut self.judgements,
                &child,
                "inode 内部条目的子指针",
            );
            judge_pointer_encryption_and_compression_fields_take_the_first_version_values(
                &mut self.judgements,
                &child,
                "inode 内部条目的子指针",
            );
            judge_pointer_extent_offset_is_zero(
                &mut self.judgements,
                &child,
                "inode 内部条目的子指针",
            );
            // I-9.2 判孩子的类按孩子头里的类标签认，不按类型段定读多长（同 I-9.17 / I-9.23，判决
            // `research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 K2 N4 那一行「inode 树 I-9.2 同形」）：
            // 孩子自证过的类与类型段说的不符（类型段不认得的，孩子读得出）判 I-9.2 红，按它自己那一类占的槽登记引用，不往下走、不记走读失败；
            // 每一份副本照判 I-2.1（校验和与内容匹配）：自证成另一类的之外，那一份校验和对不上的照红。
            // 另有一份按类型段那一类自证过的，判红之后照类型段那一支往下走那一份（F-W，见下）。
            let class_the_entry_type_names = match entry_type {
                InodeInternalEntryType::LeafContainer => {
                    Some(MappingKeyUnitClass::PackedRecordUnit)
                }
                InodeInternalEntryType::InternalNode => Some(MappingKeyUnitClass::IndexNode),
                InodeInternalEntryType::Unregistered(_) => None,
            };
            // 孩子落在哪按类型段说的那一类查中央映射（同墓碑树、谱系树那一处，`Self::walk_the_entries_of_a_container_tree_node`）。
            let child = self.located(
                &child,
                class_the_entry_type_names.map_or(
                    PositionAuthority::ThePointerItself,
                    PositionAuthority::TheCentralMapping,
                ),
            );
            // 认出另一类、另有一份头里就是类型段那一类且按那一类的长度自证得过（F-W，同墓碑树、谱系树那一处，
            // `Self::walk_the_entries_of_a_container_tree_node`）：I-9.2 照判红，再照类型段那一支往下走那一份（槽数按类型段那一类登记）。
            let child_class_the_walk_does_not_go_below =
                match child_unit_class_other_than_the_entry_type_names(
                    self.reader,
                    &child,
                    class_the_entry_type_names,
                ) {
                    None => None,
                    Some(child_class) => {
                        let some_copy_of_the_class_the_entry_type_names_verifies =
                            class_the_entry_type_names.is_some_and(|class| {
                                some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(
                                    self.reader,
                                    &child,
                                    class,
                                )
                            });
                        if some_copy_of_the_class_the_entry_type_names_verifies {
                            self.judgements.judge("I-9.2", false, || {
                                format!(
                                    "inode 内部条目类型段 {record_type} 说孩子是 {class_the_entry_type_names:?}，孩子头里的类标签是 {child_class:?}（按那一类的长度读回、整单元校验和对得上）；另一份头里就是类型段那一类、按那一类的长度自证得过，照类型段那一支往下走那一份"
                                )
                            });
                            None
                        } else {
                            Some(child_class)
                        }
                    }
                };
            if let Some(child_class) = child_class_the_walk_does_not_go_below {
                let span_in_slots = slots_spanned_by_a_unit_of_the_class(child_class).expect(
                    "child_unit_class_other_than_the_entry_type_names 只交回有长度的登记码",
                );
                self.note_references_of_a_pointer_followed_by_the_walk(
                    &child,
                    span_in_slots,
                    PositionAuthority::ThePointerItself,
                    "inode 树的孩子",
                );
                self.judge_every_copy_of_a_child_the_walk_does_not_go_into(
                    &child,
                    "inode 树的孩子",
                );
                self.judgements.judge("I-9.2", false, || {
                    format!(
                        "inode 内部条目类型段 {record_type} 说孩子是 {class_the_entry_type_names:?}，孩子头里的类标签是 {child_class:?}（按那一类的长度读回、整单元校验和对得上）"
                    )
                });
                every_leaf_reached = EveryLeafReached::No;
                continue;
            }
            let registered_entry_type = match entry_type {
                InodeInternalEntryType::LeafContainer => {
                    RegisteredInodeInternalEntryType::LeafContainer
                }
                InodeInternalEntryType::InternalNode => {
                    RegisteredInodeInternalEntryType::InternalNode
                }
                InodeInternalEntryType::Unregistered(unregistered_type) => {
                    // 类型段不认得、上面也没认出孩子自证过的类：不往下走孩子，与墓碑树、谱系树类型段不认得那一支同判
                    // （`Self::walk_the_entries_of_a_container_tree_node`）：I-9.2 红。孩子先按这一层定的形状登记引用
                    // （层级 1 的节点下面是码 3 叶容器两槽，层级 ≥ 2 的下面是码 2 节点一槽），不登记的话它占的槽不进引用并集，I-3.1 连带红。
                    // 孩子不往下走，每一份副本照判 I-2.1、两份都对不上记走读失败。
                    let child_span_in_slots_by_the_level = if node.level >= 2 { 1 } else { 2 };
                    self.note_references_of_a_pointer_followed_by_the_walk(
                        &child,
                        child_span_in_slots_by_the_level,
                        PositionAuthority::ThePointerItself,
                        "inode 树的孩子",
                    );
                    self.judge_every_copy_of_a_child_the_walk_does_not_go_into(
                        &child,
                        "inode 树的孩子",
                    );
                    self.judgements.judge("I-9.2", false, || {
                        format!(
                            "inode 内部条目的类型段 {unregistered_type} 不在 {{0, 2}} 里（孩子没有一份按它头里那一类的长度自证过，不往下走）"
                        )
                    });
                    every_leaf_reached = EveryLeafReached::No;
                    continue;
                }
            };
            let (length, span_in_slots, child_class) = match registered_entry_type {
                RegisteredInodeInternalEntryType::LeafContainer => {
                    (data_unit_bytes(), 2, MappingKeyUnitClass::PackedRecordUnit)
                }
                RegisteredInodeInternalEntryType::InternalNode => {
                    (node_bytes(), 1, MappingKeyUnitClass::IndexNode)
                }
            };
            self.note_references_of_a_pointer_followed_by_the_walk(
                &child,
                span_in_slots,
                PositionAuthority::TheCentralMapping(child_class),
                "inode 树的孩子",
            );
            let Some((unit, placement_it_was_read_from)) = self
                .read_a_unit_to_walk_and_the_placement_it_was_read_from(
                    &child,
                    length,
                    "inode 树的孩子",
                )
            else {
                judge_an_inode_entry_whose_child_does_not_read_back(
                    &mut self.judgements,
                    registered_entry_type,
                    identity,
                    child.birth_tree,
                );
                self.walk_failures
                    .push("inode 树的孩子两份都读不到对得上的".to_string());
                every_leaf_reached = EveryLeafReached::No;
                continue;
            };
            let subtree = match registered_entry_type {
                RegisteredInodeInternalEntryType::LeafContainer => {
                    let header_identity = (
                        read_u64(&unit, 43),
                        read_u16(&unit, 51),
                        read_u64(&unit, 53),
                        read_u64(&unit, 61),
                    );
                    let entry_holds = unit[6] == 3
                        && header_identity == identity
                        && child.birth_tree == identity.0;
                    self.judgements.judge("I-9.2", entry_holds, || {
                        format!(
                            "inode 内部条目的身份 {identity:?} 与孩子的头 {header_identity:?}（类 {}）不符",
                            unit[6]
                        )
                    });
                    self.walk_inode_leaf_container(&unit, &child, tree, placement_it_was_read_from)
                }
                RegisteredInodeInternalEntryType::InternalNode => {
                    let (birth_tree, _, container, container_birth) = identity;
                    let entry_holds =
                        unit[6] == 2 && birth_tree == 0 && container == 0 && container_birth == 0;
                    self.judgements.judge("I-9.2", entry_holds, || {
                        format!(
                            "inode 内部条目类型段 0：孩子的类标签 {}（要 2），身份引用的出生树 / 容器号 / 出生代 {birth_tree} / {container} / {container_birth}（要三段为 0）",
                            unit[6]
                        )
                    });
                    self.walk_inode_internal_child(
                        &unit,
                        &child,
                        node.level,
                        tree,
                        placement_it_was_read_from,
                    )
                }
            };
            // 别的版本先走过的孩子同样判（`.claude/kb/checks-owed.md` C896 的口径：分隔 key 由这一版的父条目定，命中时照判、不往下走），
            // 它的区间取先走那一遍记下的叶序（`inode_subtree_walked_by_an_earlier_version`，不重读盘）。
            let smallest = subtree
                .leaves_in_leaf_order
                .iter()
                .map(|leaf| leaf.smallest_inode)
                .min();
            let largest = subtree
                .leaves_in_leaf_order
                .iter()
                .map(|leaf| leaf.largest_inode)
                .max();
            if let (Some(smallest), Some(largest)) = (smallest, largest) {
                separator_and_child_range.push(SeparatorAndInodeChildRange {
                    separator: read_u64(entry, 0),
                    smallest_key_of_the_child: smallest,
                    largest_key_of_the_child: largest,
                    child_walked_first_by_this_version: subtree.walked_first_by_this_version,
                });
            }
            every_leaf_reached = every_leaf_reached.and(subtree.every_leaf_reached);
            leaves_in_leaf_order.extend(subtree.leaves_in_leaf_order);
        }
        self.stop_walking_below(previous_parent_unit);
        self.inode_leaf_order_below_a_walked_unit.insert(
            node_placement,
            InodeLeafOrderBelowAWalkedUnit {
                leaves_in_leaf_order: leaves_in_leaf_order[leaves_before_this_node..].to_vec(),
                every_leaf_reached,
            },
        );
        self.judge_separators_against_the_inode_children(&separator_and_child_range);
        self.judge_the_key_range_of_an_inode_internal_node_against_its_leaves(
            node,
            what,
            &leaves_in_leaf_order[leaves_before_this_node..],
            every_leaf_reached,
        );
        every_leaf_reached
    }

    /// I-1.1 的 inode 树那一半（D18（块里携带什么信息） 已定项 2「区间取子树覆盖区间」）：一个 inode 树内部节点（根，或类型段 0 指着的码 2 节点）
    /// 头里的 key 区间 = [它下面叶序里最小的记录号, 最大的记录号]（孩子是码 3 容器的内部节点上，max_key 是最后那片容器里最后一条记录的 key，
    /// 不是最后一条条目的分隔 key）。只在这一枝的叶全都到得了时判（[`EveryLeafReached`]）；零条目的节点（删空之后的零条目根，写者写 [0, 0]）
    /// 与自述 key 宽不是 inode 号 8 字节的（读节点那一步已经判 I-1.1 红）不判。叶序里有别的版本先走过的叶（缓存的最小、最大记录号）。
    fn judge_the_key_range_of_an_inode_internal_node_against_its_leaves(
        &mut self,
        node: &crate::IndexNodeView,
        what: &str,
        leaves_below_this_node: &[InodeLeafContainerInLeafOrder],
        every_leaf_reached: EveryLeafReached,
    ) {
        match every_leaf_reached {
            EveryLeafReached::Yes => {}
            EveryLeafReached::No => return,
        }
        if node.entries.is_empty() || node.key_width != KEY_SCHEMA_INODE.width() {
            return;
        }
        let smallest = leaves_below_this_node
            .iter()
            .map(|leaf| leaf.smallest_inode)
            .min();
        let largest = leaves_below_this_node
            .iter()
            .map(|leaf| leaf.largest_inode)
            .max();
        let (Some(smallest), Some(largest)) = (smallest, largest) else {
            return;
        };
        let declared_smallest = read_u64(&node.smallest_key, 0);
        let declared_largest = read_u64(&node.largest_key, 0);
        self.judgements.judge(
            "I-1.1",
            declared_smallest == smallest && declared_largest == largest,
            || {
                format!(
                    "{what}（inode 树层级 {}）：头里的 key 区间 [{declared_smallest}, {declared_largest}] 不是子树覆盖区间 [{smallest}, {largest}]（叶序里最小、最大的记录号）",
                    node.level
                )
            },
        );
    }

    /// I-9.7：一条 inode 记录的
    /// 预留段偏移 108–119 与 136–139 恒 0；偏移 120 起的截断字段 T（8 字节）与 N_w（8 字节）判「T = 0 ⇒ N_w = 0」
    /// （截到 0 时 T ≠ 0、N_w = 0 合法，反方向不判）。各判各的一次，说明写明是哪一段。
    fn judge_reserved_bytes_and_the_truncation_field_of_an_inode_record(
        &mut self,
        inode: u64,
        record: &[u8],
    ) {
        self.judgements.judge(
            "I-9.7",
            record[INODE_RECORD_RESERVED_BYTES_BEFORE_THE_TRUNCATION_FIELD]
                .iter()
                .all(|byte| *byte == 0),
            || format!("inode {inode} 的记录偏移 108–119（填充 4 与 flags 8）不全为 0"),
        );
        self.judgements.judge(
            "I-9.7",
            record[INODE_RECORD_RESERVED_BYTES_AFTER_THE_TRUNCATION_FIELD]
                .iter()
                .all(|byte| *byte == 0),
            || format!("inode {inode} 的记录偏移 136–139（预留 4）不全为 0"),
        );
        let truncation_field_txg = read_u64(record, INODE_RECORD_TRUNCATION_FIELD_TXG_OFFSET);
        let truncation_field_size_in_bytes =
            read_u64(record, INODE_RECORD_TRUNCATION_FIELD_SIZE_OFFSET);
        self.judgements.judge(
            "I-9.7",
            truncation_field_txg != 0 || truncation_field_size_in_bytes == 0,
            || {
                format!(
                    "inode {inode} 的记录截断字段 T = 0 而 N_w = {truncation_field_size_in_bytes}：没写过的截断字段两段都是 0"
                )
            },
        );
    }

    /// 一片 inode 叶容器（码 3 打包记录类型 2）：这一遍第一次走到时判树 ID（I-1.3）、出生身份（I-1.2 / I-4.2）、容器（I-1.7 等）与容器里
    /// 每条记录（I-9.13、I-9.7、I-9.4 第一句与第四句、I-9.15），交回它的容器号与装的最小、最大 inode 号；别的版本已经走过时不重读，
    /// 从 `inode_leaf_order_below_a_walked_unit` 接上它在叶序里的那一格，它下面的落点并进这一版（I-5.1）。
    /// 走过、并进前一版、叶序与诞生代号都按读回的那一份的落点 `placement_it_was_read_from` 认（
    /// [`Self::read_a_unit_to_walk_and_the_placement_it_was_read_from`]）。
    fn walk_inode_leaf_container(
        &mut self,
        unit: &[u8],
        child: &PointerView,
        tree: u64,
        placement_it_was_read_from: (u32, u64),
    ) -> InodeSubtreeInTheWalk {
        if !self.visited_units.insert(placement_it_was_read_from) {
            self.note_a_unit_an_earlier_version_walked_referenced_again(
                unit,
                child,
                placement_it_was_read_from,
                ReferenceTheWalkFollowsToAUnit::PackedContainerOfTree { tree },
                "inode 树叶",
            );
            return self.inode_subtree_walked_by_an_earlier_version(placement_it_was_read_from);
        }
        let mut leaves_in_leaf_order: Vec<InodeLeafContainerInLeafOrder> = Vec::new();
        self.judge_the_birth_tree_of_a_packed_container(unit, tree, "inode 树叶");
        self.judge_birth_identity_of_a_referenced_unit(
            unit,
            child,
            placement_it_was_read_from,
            "inode 树的叶容器",
        );
        if let Some(records) =
            self.judge_packed_container(unit, PACKED_TYPE_INODE, "inode 树的叶容器")
        {
            self.walked_units_whose_header_the_walk_could_use
                .insert(placement_it_was_read_from);
            let container = read_u64(unit, 53);
            self.judgements.judge("I-9.13", !records.is_empty(), || {
                "类型 2 容器一条记录都没有".to_string()
            });
            let mut smallest_inode_in_this_container: Option<u64> = None;
            let mut largest_inode_in_this_container: Option<u64> = None;
            let mut inode_of_the_previous_record: Option<u64> = None;
            for record in records {
                let inode = read_u64(&record, 0);
                // I-9.4 第四句：同一片打包记录类型 2 容器里记录号沿记录序严格递增（判决
                // `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T11 取甲）。与第二、三句合起来就是整棵树的 key 沿叶序严格递增。
                if let Some(previous_inode) = inode_of_the_previous_record {
                    self.judgements.judge("I-9.4", inode > previous_inode, || {
                        format!(
                            "容器 {container} 里记录号沿记录序不严格递增：inode {previous_inode} 之后是 inode {inode}"
                        )
                    });
                }
                inode_of_the_previous_record = Some(inode);
                self.judge_reserved_bytes_and_the_truncation_field_of_an_inode_record(
                    inode, &record,
                );
                self.judgements.judge("I-9.4", inode >= container, || {
                    format!("inode {inode} 小于它所在容器的号 {container}")
                });
                let size_in_bytes = read_u64(&record, INODE_RECORD_SIZE_OFFSET);
                let blocks = read_u64(&record, INODE_RECORD_BLOCKS_OFFSET);
                let logical_length_in_blocks =
                    size_in_bytes.div_ceil(INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES);
                self.judgements
                        .judge("I-9.15", blocks == logical_length_in_blocks, || {
                            format!(
                                "inode {inode} 的记录 blocks {blocks} 不等于 ⌈size {size_in_bytes} ÷ {INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES}⌉ = {logical_length_in_blocks}"
                            )
                        });
                // I-9.26：对象出生代 ≤ 改动计数 ≤ 这片容器头的诞生代号（D8（核心索引结构） 已定项 6 偏移 88；下界「创建算一次改动」）。
                // 上界不是等于：容器因别的记录改动而重写时，没改的那条记录的改动计数小于容器诞生代号是合法的。
                let object_birth = read_u64(&record, INODE_RECORD_OBJECT_BIRTH_OFFSET);
                let change_count = read_u64(&record, INODE_RECORD_CHANGE_COUNT_OFFSET);
                let container_birth_txg = read_u64(unit, PACKED_UNIT_BIRTH_TXG_OFFSET);
                self.judgements.judge(
                    INODE_CHANGE_COUNT_BETWEEN_OBJECT_BIRTH_AND_CONTAINER_BIRTH,
                    object_birth <= change_count && change_count <= container_birth_txg,
                    || {
                        format!(
                            "inode {inode} 的改动计数 {change_count} 不在对象出生代 {object_birth} 与所在容器的诞生代号 {container_birth_txg} 之间"
                        )
                    },
                );
                // 对象出生代（I-9.10）与 inode 树内最大号（I-9.6）只记活头的树：快照里的记录是建快照那一刻的样子，同号可以已是别的对象。
                match self.tree_sharing_in_the_walk {
                    TreeSharingInTheWalk::OwnTreesOfTheVersion => {
                        self.inode_object_birth.insert(inode, object_birth);
                        self.largest_inode_number_in_the_inode_tree = Some(
                            self.largest_inode_number_in_the_inode_tree
                                .map_or(inode, |seen| seen.max(inode)),
                        );
                    }
                    TreeSharingInTheWalk::SnapshotTreesSharingUnitsWithTheHead => {}
                }
                smallest_inode_in_this_container =
                    Some(smallest_inode_in_this_container.map_or(inode, |seen| seen.min(inode)));
                largest_inode_in_this_container =
                    Some(largest_inode_in_this_container.map_or(inode, |seen| seen.max(inode)));
            }
            if let (Some(smallest), Some(largest)) = (
                smallest_inode_in_this_container,
                largest_inode_in_this_container,
            ) {
                leaves_in_leaf_order.push(InodeLeafContainerInLeafOrder {
                    container,
                    smallest_inode: smallest,
                    largest_inode: largest,
                });
            }
        }
        // 一片叶到得了 = 它解得开、至少一条记录、记进了叶序。
        let every_leaf_reached = if leaves_in_leaf_order.is_empty() {
            EveryLeafReached::No
        } else {
            EveryLeafReached::Yes
        };
        self.inode_leaf_order_below_a_walked_unit.insert(
            placement_it_was_read_from,
            InodeLeafOrderBelowAWalkedUnit {
                leaves_in_leaf_order: leaves_in_leaf_order.clone(),
                every_leaf_reached,
            },
        );
        InodeSubtreeInTheWalk {
            leaves_in_leaf_order,
            every_leaf_reached,
            walked_first_by_this_version: true,
        }
    }

    /// 别的版本已经走过的 inode 树单元（叶容器或类型段 0 的内部节点）：不重读，从 `inode_leaf_order_below_a_walked_unit` 接上它下面的叶序与
    /// 那一枝的叶是不是全都到得了。表里没有（先走那一版在记下之前就停了）当一片都没到。
    fn inode_subtree_walked_by_an_earlier_version(
        &self,
        placement_it_was_read_from: (u32, u64),
    ) -> InodeSubtreeInTheWalk {
        let below = self
            .inode_leaf_order_below_a_walked_unit
            .get(&placement_it_was_read_from);
        InodeSubtreeInTheWalk {
            leaves_in_leaf_order: below
                .map(|below| below.leaves_in_leaf_order.clone())
                .unwrap_or_default(),
            every_leaf_reached: below
                .map_or(EveryLeafReached::No, |below| below.every_leaf_reached),
            walked_first_by_this_version: false,
        }
    }

    /// inode 树里类型段 0 指着的码 2 内部节点头里的层级 = 引用它的父节点层级 − 1（I-1.1），交回成立与否。首次走读与别的版本先走过的节点
    /// （`.claude/kb/checks-owed.md` C896：层级由这一版的父节点定）共用这一判。
    fn judge_the_level_of_an_inode_internal_child(
        &mut self,
        node_level: u8,
        parent_level: u8,
        what: &str,
    ) -> bool {
        let expected_level = parent_level.checked_sub(1);
        self.judgements
            .judge("I-1.1", Some(node_level) == expected_level, || {
                format!(
                    "{what}：头里的层级是 {node_level}，引用它的父节点层级是 {parent_level}（要父层级减一）"
                )
            });
        Some(node_level) == expected_level
    }

    /// 类型段 0 指着的码 2 内部节点：这一遍第一次走到时判头（I-1.6 / I-2.4 / I-2.3 / I-1.4）、出生身份（I-1.2 / I-4.2）、树 ID（I-1.3）、
    /// key（I-1.1，与 inode 根同一个读法：这里只判分隔 key 严格递增，头里的区间是子树覆盖区间，走完它下面的叶判）、
    /// 层级 = 父层级 − 1（I-1.1）与条目宽 120（I-1.10），再往下走（[`Self::walk_inode_internal_node`]）；
    /// 别的版本已经走过时不重读，从 `inode_leaf_order_below_a_walked_unit` 接上它下面的叶序，它下面的落点并进这一版（I-5.1）。
    /// 层级 0 的码 2 节点在 inode 树里没有条款定的形状（叶是码 3 容器，I-9.1 那一行只给根留了「空的根」）：记走读失败、不往下走。
    /// 走过、父单元、叶序、并进前一版与诞生代号都按读回的那一份的落点 `placement_it_was_read_from` 认（
    /// 同 [`Self::walk_inode_leaf_container`]）。
    pub(super) fn walk_inode_internal_child(
        &mut self,
        unit: &[u8],
        child: &PointerView,
        parent_level: u8,
        tree: u64,
        placement_it_was_read_from: (u32, u64),
    ) -> InodeSubtreeInTheWalk {
        let not_walked_below = InodeSubtreeInTheWalk {
            leaves_in_leaf_order: Vec::new(),
            every_leaf_reached: EveryLeafReached::No,
            walked_first_by_this_version: true,
        };
        let what = "inode 树的内部节点";
        if !self.visited_units.insert(placement_it_was_read_from) {
            self.note_a_unit_an_earlier_version_walked_referenced_again(
                unit,
                child,
                placement_it_was_read_from,
                ReferenceTheWalkFollowsToAUnit::IndexNodeOfTree { tree },
                what,
            );
            // 这一版的父节点定的层级照判（`.claude/kb/checks-owed.md` C896），不往下走、不记走读失败。
            if let Some(view) =
                self.view_of_a_node_an_earlier_version_walked(unit, placement_it_was_read_from)
            {
                self.judge_the_level_of_an_inode_internal_child(view.level, parent_level, what);
            }
            return self.inode_subtree_walked_by_an_earlier_version(placement_it_was_read_from);
        }
        if !self.judge_unit_header(unit, 2, what) {
            return not_walked_below;
        }
        self.walked_units_whose_header_the_walk_could_use
            .insert(placement_it_was_read_from);
        self.judge_birth_identity_of_a_referenced_unit(
            unit,
            child,
            placement_it_was_read_from,
            what,
        );
        let Some(view) = self.index_node_view_judging_a_zero_entry_width(
            unit,
            EntryFieldTableRegistration::Registered,
            what,
        ) else {
            return not_walked_below;
        };
        self.judge_the_tree_identifier_of_an_index_node(&view, tree, what);
        // 头里的 key 区间是子树覆盖区间（D18（块里携带什么信息） 已定项 2），走完它下面的叶在 `walk_inode_internal_node` 里判；这里只判 key 宽与分隔 key 严格递增。
        let keys = check_separators_of_a_node_whose_subtree_coverage_the_walk_judges(
            &view,
            KEY_SCHEMA_INODE,
        );
        self.judgements.judge("I-1.1", keys.is_ok(), || {
            format!("{what}：key 宽 / 分隔 key 次序不符（{keys:?}）")
        });
        if !self.judge_the_level_of_an_inode_internal_child(view.level, parent_level, what) {
            self.walk_failures
                .push(format!("{what}：层级不是父层级减一，这一支走不下去"));
            return not_walked_below;
        }
        if view.level == 0 {
            self.walk_failures.push(format!(
                "{what}：类型段 0 的条目指着层级 0 的码 2 节点，inode 树里没有这种形状，这一支走不下去"
            ));
            return not_walked_below;
        }
        let entry_width = view.entry_width;
        self.judgements.judge(
            "I-1.10",
            entry_width == inode_internal_entry_bytes(),
            || {
                format!(
                    "{what}：头里自述的条目宽 {entry_width} 不等于 inode 树的条目字段表宽度 {}",
                    inode_internal_entry_bytes()
                )
            },
        );
        if entry_width != inode_internal_entry_bytes() {
            return not_walked_below;
        }
        let mut leaves_in_leaf_order = Vec::new();
        let every_leaf_reached = self.walk_inode_internal_node(
            &view,
            tree,
            placement_it_was_read_from,
            what,
            &mut leaves_in_leaf_order,
        );
        InodeSubtreeInTheWalk {
            leaves_in_leaf_order,
            every_leaf_reached,
            walked_first_by_this_version: true,
        }
    }

    /// I-9.12（分隔 key 落在孩子区间之外）：一个 inode 树内部节点的条目按分隔 key 严格递增；第 i 条的分隔 key ≤ 第 i 个孩子的 key 区间下界，
    /// 且 > 第 i−1 个孩子的 key 区间上界。孩子的 key 区间：叶容器按它装的记录现算，类型段 0 的内部节点按它下面各片叶装的记录现算
    /// （D18（块里携带什么信息） 已定项 2「子树里到得了的最小 key 与最大 key」）。
    /// 判的是读得出的那些孩子，别的版本先走过的也判（分隔 key 由这一版的父条目定，`.claude/kb/checks-owed.md` C896 的口径；它的区间取
    /// 先走那一遍记下的叶序）；一个都读不出来时这条在这份镜像上没有对象（末尾按「走到过 inode 树」报不适用）。
    fn judge_separators_against_the_inode_children(
        &mut self,
        separator_and_child_range: &[SeparatorAndInodeChildRange],
    ) {
        for (position, this_entry) in separator_and_child_range.iter().enumerate() {
            let separator = this_entry.separator;
            let smallest = this_entry.smallest_key_of_the_child;
            let where_the_range_comes_from = this_entry.where_the_range_of_the_child_comes_from();
            self.judgements.judge("I-9.12", separator <= smallest, || {
                format!("第 {position} 条的分隔 key {separator} 大于这个孩子的最小 key {smallest}{where_the_range_comes_from}")
            });
            if position > 0 {
                let previous_entry = &separator_and_child_range[position - 1];
                let previous_separator = previous_entry.separator;
                let previous_largest = previous_entry.largest_key_of_the_child;
                let where_the_previous_range_comes_from =
                    previous_entry.where_the_range_of_the_child_comes_from();
                self.judgements
                    .judge("I-9.12", separator > previous_separator, || {
                        format!("第 {position} 条的分隔 key {separator} 不大于上一条的 {previous_separator}")
                    });
                self.judgements
                    .judge("I-9.12", separator > previous_largest, || {
                        format!(
                            "第 {position} 条的分隔 key {separator} 不大于上一个孩子的最大 key {previous_largest}{where_the_previous_range_comes_from}"
                        )
                    });
            }
        }
    }
}
