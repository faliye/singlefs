//! 这次发布之后的容器树：`ContainerTreeAfterThisPublish`，往开着的容器里追加记录（`append_record_to_the_open_container`、`append_records_to_the_open_container`）与追加进度的规格和引理。

use super::*;

verus! {

/// 一版容器树里一个码 2 节点从哪来。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContainerTreeNodeOrigin {
    RewrittenThisPublish,
    /// 上一版同一个位置上的那个节点，一个字节不动（长高那次的旧根也是这一种：位置不变、只是不再是根）。
    CarriedFromThePreviousVersion,
}

/// 这次发布之后一棵容器树的样子：叶容器（按叶序）、这次重写哪几片（叶序）、这次之后的形状与这次之前那一版每层的节点数。
/// 码 2 节点重写哪几个由它们推：有容器重写时是重写的那几片的全部祖先（COW 叶 + 祖先），没有时一个都不重写。
/// 这次之前那一版只记每层节点数（[`ContainerTreeNodeCounts`]）：发布对它只问哪个位置上有节点、哪个是根，不照抄它的形状。
#[derive(Clone, Debug, PartialEq, Eq)]
#[verifier::allow(autoderive_clone_without_spec)]
pub struct ContainerTreeAfterThisPublish<Record> {
    pub containers: Vec<RecordContainer<Record>>,
    pub rewritten_containers: Vec<usize>,
    pub shape: ContainerTreeShape,
    pub node_counts_before_this_publish: ContainerTreeNodeCounts,
}

/// 两组容器逐片相同：身份相同、记录逐条相同。
pub open spec fn containers_are_the_same<Record>(
    earlier: Seq<RecordContainer<Record>>,
    later: Seq<RecordContainer<Record>>,
) -> bool {
    &&& earlier.len() == later.len()
    &&& forall|position: int|
        0 <= position < earlier.len() ==> (#[trigger] later[position]).identity == earlier[position].identity
            && later[position].records@ == earlier[position].records@
}

/// 照抄一片容器的记录。
fn copy_of_the_records<Record: Copy>(records: &[Record]) -> (copy: Vec<Record>)
    ensures
        copy@ == records@,
{
    let mut copy: Vec<Record> = Vec::new();
    let mut position: usize = 0;
    while position < records.len()
        invariant
            position <= records.len(),
            copy@ == records@.take(position as int),
        decreases records.len() - position,
    {
        copy.push(records[position]);
        position += 1;
        assert(copy@ =~= records@.take(position as int));
    }
    assert(records@.take(records.len() as int) =~= records@);
    copy
}

/// 逐片照抄容器（记录逐条照抄）。
fn copy_of_the_containers<Record: Copy>(containers: &[RecordContainer<Record>]) -> (copy: Vec<RecordContainer<Record>>)
    ensures
        containers_are_the_same(containers@, copy@),
        records_of_containers(copy@) == records_of_containers(containers@),
{
    let mut copy: Vec<RecordContainer<Record>> = Vec::new();
    let mut position: usize = 0;
    while position < containers.len()
        invariant
            position <= containers.len(),
            containers_are_the_same(containers@.take(position as int), copy@),
            records_of_containers(copy@) == records_of_containers(containers@.take(position as int)),
        decreases containers.len() - position,
    {
        let container = &containers[position];
        let ghost copy_before = copy@;
        copy.push(RecordContainer { identity: container.identity, records: copy_of_the_records(&container.records) });
        position += 1;
        proof {
            assert(containers@.take(position as int).drop_last() =~= containers@.take(position as int - 1));
            assert(copy@.drop_last() =~= copy_before);
        }
    }
    proof {
        assert(containers@.take(containers.len() as int) =~= containers@);
    }
    copy
}

impl<Record: Copy> ContainerTreeAfterThisPublish<Record> {
    /// 这次不追加的发布：容器、形状都照抄上一版，这次之前那一版只记每层节点数。
    ///
    /// # Panics
    /// 形状里的容器数与容器的片数不等（`requires` 排除的那一种）：两样出自同一版。
    #[must_use]
    pub fn carried(containers: Vec<RecordContainer<Record>>, shape: ContainerTreeShape) -> (carried: Self)
        requires
            container_count(shape.levels()) == containers.len(),
        ensures
            carried.containers@ == containers@,
            carried.rewritten_containers@.len() == 0,
            carried.shape.levels() == shape.levels(),
            carried.node_counts_before_this_publish.counts() == node_counts_of(shape.levels()),
    {
        assert!(
            shape.containers() == containers.len(),
            "形状里层级 1 的孩子数之和就是容器数"
        );
        let node_counts_before_this_publish = shape.node_counts();
        Self { containers, rewritten_containers: Vec::new(), shape, node_counts_before_this_publish }
    }
}

impl<Record> ContainerTreeAfterThisPublish<Record> {
    /// 根跟着重写：有容器重写（COW 叶 + 祖先）。容器只增不减，没有「删空一片」那一格。
    #[must_use]
    pub fn is_the_root_rewritten(&self) -> (is_the_root_rewritten: bool)
        ensures
            is_the_root_rewritten == (self.rewritten_containers.len() > 0),
    {
        !self.rewritten_containers.is_empty()
    }

    /// 这次之后的内部层数（根的层级减一，零根与只有根时 0）。
    #[must_use]
    pub fn internal_levels(&self) -> (internal_levels: u64)
        requires
            self.shape.levels().len() <= 255,
        ensures
            internal_levels == if self.shape.levels().len() == 0 { 0 } else { self.shape.levels().len() - 1 },
    {
        self.shape.internal_levels()
    }

    /// 这次重写的容器从第几片起（重写的是最右端连着的那几片，[`rewritten_containers_are_the_rightmost`]）；一片都没重写时 `None`。
    /// 发布路径问「第几片这次重写了没有」按它比：序号不小于它就重写了，不在重写清单上逐个找（`crate::transaction` 列映射单元、装容器单元）。
    ///
    /// # Panics
    /// 重写清单不是最右端连着的那几片（`requires` 排除的那一种）：造它的只有 [`append_records_to_the_open_container`] 与 [`Self::carried`]，
    /// 两处的 `ensures` 都写着它是。逐片核一遍，代价是重写的片数。
    #[must_use]
    pub fn first_rewritten_container(&self) -> (first: Option<usize>)
        requires
            rewritten_containers_are_the_rightmost(self.rewritten_containers@, self.containers@.len() as int),
        ensures
            first is None <==> self.rewritten_containers@.len() == 0,
            first matches Some(first_position) ==> first_position == self.containers@.len() - self.rewritten_containers@.len()
                && forall|position: int|
                0 <= position < self.containers@.len() ==> (#[trigger] self.rewritten_containers@.contains(position as usize)
                    <==> position >= first_position),
    {
        let rightmost_count = self.rightmost_count_of_the_rewritten_containers(self.containers.len());
        if rightmost_count == 0 {
            return None;
        }
        let first_position = self.containers.len() - rightmost_count;
        proof {
            let rewritten = self.rewritten_containers@;
            assert forall|position: int| 0 <= position < self.containers@.len() implies (#[trigger] rewritten.contains(
                position as usize,
            ) <==> position >= first_position) by {
                if position >= first_position {
                    assert(rewritten[position - first_position] == position);
                }
                if rewritten.contains(position as usize) {
                    let found = choose|found: int| 0 <= found < rewritten.len() && rewritten[found] == position as usize;
                    assert(rewritten[found] == first_position + found);
                }
            }
        }
        Some(first_position)
    }

    /// 重写清单有几片，连带逐片核它就是最右端连着的那几片（一共 `total` 片）：代价是重写的片数。
    ///
    /// # Panics
    /// 重写清单不是最右端连着的那几片（`requires` 排除的那一种）：见 [`Self::first_rewritten_container`]。
    fn rightmost_count_of_the_rewritten_containers(&self, total: usize) -> (rightmost_count: usize)
        requires
            rewritten_containers_are_the_rightmost(self.rewritten_containers@, total as int),
        ensures
            rightmost_count == self.rewritten_containers@.len(),
    {
        let rightmost_count = self.rewritten_containers.len();
        assert!(
            rightmost_count <= total,
            "重写的 {rightmost_count} 片是这一版 {total} 片里最右端的那几片"
        );
        let mut listed: usize = 0;
        while listed < rightmost_count
            invariant
                rewritten_containers_are_the_rightmost(self.rewritten_containers@, total as int),
                rightmost_count == self.rewritten_containers@.len(),
                rightmost_count <= total,
                listed <= rightmost_count,
            decreases rightmost_count - listed,
        {
            assert!(
                self.rewritten_containers[listed] == total - rightmost_count + listed,
                "重写清单是最右端连着的那几片：造它的只有 append_records_to_the_open_container 与 carried"
            );
            listed += 1;
        }
        rightmost_count
    }

    /// 这次长了一层：上一版有根、这次之后的根层级比它高。那一次上一版的根不重写、换成根之下的节点留在原位置，不被这次换下。
    #[must_use]
    pub fn grew_a_level_this_publish(&self) -> (grew_a_level: bool)
        requires
            self.shape.levels().len() <= 255,
            self.node_counts_before_this_publish.counts().len() <= 255,
        ensures
            grew_a_level == (self.node_counts_before_this_publish.counts().len() > 0 && self.shape.levels().len()
                > self.node_counts_before_this_publish.counts().len()),
    {
        self.node_counts_before_this_publish.root_level() > 0
            && self.shape.root_level() > self.node_counts_before_this_publish.root_level()
    }

    /// 这次长了一层、而上一版的根一个字节不动地留在原位置当根之下的节点（它下面没有容器重写）：那一次新根在上一版里没有落点，
    /// 上一版的根不被这次换下。一次追加好几条、上一版的开放容器先装满再长层时，旧根下面有容器重写，它跟着重写、照样被换下。
    #[must_use]
    pub fn previous_root_is_carried_down_unchanged(&self) -> (is_carried_down_unchanged: bool)
        requires
            shape_fits_the_position_fields(self.shape.levels()),
            self.node_counts_before_this_publish.counts().len() <= 255,
            rewritten_containers_are_the_rightmost(self.rewritten_containers@, container_count(self.shape.levels())),
        ensures
            is_carried_down_unchanged == (self.node_counts_before_this_publish.counts().len() > 0 && self.shape.levels().len()
                > self.node_counts_before_this_publish.counts().len() && !ancestors_below_level(
                self.shape.levels(),
                self.rewritten_containers@,
                self.shape.levels().len() as int,
            ).contains(position_at(self.node_counts_before_this_publish.counts().len() - 1, 0))),
    {
        if !self.grew_a_level_this_publish() {
            return false;
        }
        match self.node_counts_before_this_publish.root_position() {
            Some(previous_root) => {
                let rewritten_nodes = self.rewritten_nodes();
                !position_is_listed(&rewritten_nodes, previous_root)
            },
            None => false,
        }
    }

    /// 这次重写的码 2 节点，按 bump 次序（层级升序，根最末）：重写的每片容器的全部祖先（COW 叶 + 祖先）。重写的是最右端连着的那几片，
    /// 每层的祖先是一段后缀（[`ContainerTreeShape::ancestors_of_the_rightmost_containers`]），只读那几段。
    ///
    /// # Panics
    /// 重写清单不是最右端连着的那几片（`requires` 排除的那一种）：见 [`Self::first_rewritten_container`]。
    #[must_use]
    pub fn rewritten_nodes(&self) -> (rewritten_nodes: Vec<ContainerTreeNodePosition>)
        requires
            shape_fits_the_position_fields(self.shape.levels()),
            rewritten_containers_are_the_rightmost(self.rewritten_containers@, container_count(self.shape.levels())),
        ensures
            rewritten_nodes@ == ancestors_below_level(
                self.shape.levels(),
                self.rewritten_containers@,
                self.shape.levels().len() as int,
            ),
    {
        let rightmost_count = self.rightmost_count_of_the_rewritten_containers(self.shape.containers());
        proof {
            lemma_rewritten_containers_are_the_rightmost_ones(self.rewritten_containers@, container_count(self.shape.levels()));
        }
        if rightmost_count == 0 {
            proof {
                assert(self.rewritten_containers@ =~= Seq::<usize>::empty());
                lemma_no_containers_have_no_ancestors(self.shape.levels(), self.shape.levels().len() as int);
            }
            return Vec::new();
        }
        self.shape.ancestors_of_the_rightmost_containers(rightmost_count)
    }

    /// 这一版每个码 2 节点从哪来，按 bump 次序。这次重写的是重写的那几片的祖先：每层是一段后缀，按每层那一段的起点判
    /// （[`ContainerTreeShape::ancestor_suffix_starts_of_the_rightmost_containers`]），不在重写的节点清单上逐个找。
    ///
    /// # Panics
    /// 一个照抄的位置在上一版里没有节点（`requires` 排除的那一种）：容器与节点都只在最右端新开，最右那一条路径之外的位置上一版都有。
    /// 重写清单不是最右端连着的那几片：见 [`Self::first_rewritten_container`]。
    #[must_use]
    pub fn node_origins(&self) -> (node_origins: Vec<(ContainerTreeNodePosition, ContainerTreeNodeOrigin)>)
        requires
            shape_fits_the_position_fields(self.shape.levels()),
            rewritten_containers_are_the_rightmost(self.rewritten_containers@, container_count(self.shape.levels())),
            every_node_absent_from_the_previous_version_is_rewritten(
                self.node_counts_before_this_publish.counts(),
                self.shape.levels(),
                self.rewritten_containers@,
            ),
        ensures
            node_origins.len() == bump_order_positions(self.shape.levels(), self.shape.levels().len() as int).len(),
            forall|listed: int|
                0 <= listed < node_origins.len() ==> {
                    let origin = #[trigger] node_origins@[listed];
                    &&& origin.0 == bump_order_positions(self.shape.levels(), self.shape.levels().len() as int)[listed]
                    &&& (origin.1 is RewrittenThisPublish <==> ancestors_below_level(
                        self.shape.levels(),
                        self.rewritten_containers@,
                        self.shape.levels().len() as int,
                    ).contains(origin.0))
                    &&& origin.1 is CarriedFromThePreviousVersion ==> position_is_counted(
                        self.node_counts_before_this_publish.counts(),
                        origin.0,
                    )
                },
    {
        proof {
            use_type_invariant(&self.shape);
            lemma_bump_order_positions_are_nodes(self.shape.levels(), self.shape.levels().len() as int);
            assert forall|level_index: int, node: int|
                #[trigger] node_is_in_the_levels(self.shape.levels(), level_index, node) implies level_index < self.shape.levels().len() by {}
            lemma_ancestors_below_level_contains(
                self.shape.levels(),
                self.rewritten_containers@,
                self.shape.levels().len() as int,
            );
            lemma_rewritten_containers_are_the_rightmost_ones(self.rewritten_containers@, container_count(self.shape.levels()));
        }
        let ghost levels = self.shape.levels();
        let ghost rewritten = ancestors_below_level(levels, self.rewritten_containers@, levels.len() as int);
        let rightmost_count = self.rightmost_count_of_the_rewritten_containers(self.shape.containers());
        // 每层重写的那一段从哪个节点起；一片都没重写时一个节点都不重写。
        let suffix_starts = if rightmost_count == 0 {
            None
        } else {
            Some(self.shape.ancestor_suffix_starts_of_the_rightmost_containers(rightmost_count))
        };
        proof {
            if rightmost_count == 0 {
                assert(self.rewritten_containers@ =~= Seq::<usize>::empty());
            }
        }
        let positions = self.shape.positions_in_bump_order();
        let mut node_origins: Vec<(ContainerTreeNodePosition, ContainerTreeNodeOrigin)> = Vec::new();
        let mut listed: usize = 0;
        while listed < positions.len()
            invariant
                levels == self.shape.levels(),
                container_tree_shape_is_well_formed(levels),
                shape_fits_the_position_fields(levels),
                positions@ == bump_order_positions(levels, levels.len() as int),
                rewritten == ancestors_below_level(levels, self.rewritten_containers@, levels.len() as int),
                self.rewritten_containers@ == rightmost_containers(container_count(levels), rightmost_count as int),
                rightmost_count == self.rewritten_containers@.len(),
                every_node_absent_from_the_previous_version_is_rewritten(
                    self.node_counts_before_this_publish.counts(),
                    levels,
                    self.rewritten_containers@,
                ),
                match suffix_starts {
                    Some(starts) => rightmost_count >= 1 && starts@.len() == levels.len() && forall|level_index: int, node: int|
                        #[trigger] node_is_in_the_levels(levels, level_index, node) ==> (node_is_an_ancestor_of_one_of(
                            levels,
                            level_index,
                            node,
                            self.rewritten_containers@,
                        ) <==> node >= starts@[level_index]),
                    None => rightmost_count == 0,
                },
                forall|any_listed: int|
                    0 <= any_listed < positions.len() ==> {
                        let position = #[trigger] positions@[any_listed];
                        &&& 1 <= position.level <= levels.len()
                        &&& position.index_in_level < levels[position.level - 1].len()
                    },
                forall|level_index: int, node: int|
                    #[trigger] node_is_in_the_levels(levels, level_index, node) && level_index < levels.len() ==> (
                    rewritten.contains(position_at(level_index, node)) <==> node_is_an_ancestor_of_one_of(
                        levels,
                        level_index,
                        node,
                        self.rewritten_containers@,
                    )),
                listed <= positions.len(),
                node_origins.len() == listed,
                forall|done: int|
                    0 <= done < listed ==> {
                        let origin = #[trigger] node_origins@[done];
                        &&& origin.0 == positions@[done]
                        &&& (origin.1 is RewrittenThisPublish <==> rewritten.contains(origin.0))
                        &&& origin.1 is CarriedFromThePreviousVersion ==> position_is_counted(
                            self.node_counts_before_this_publish.counts(),
                            origin.0,
                        )
                    },
            decreases positions.len() - listed,
        {
            let position = positions[listed];
            let level_index = usize::from(position.level) - 1;
            let index_in_level = usize::try_from(position.index_in_level).expect("同层序号装得进 usize");
            proof {
                assert(1 <= position.level <= levels.len());
                assert(node_is_in_the_levels(levels, level_index as int, index_in_level as int));
                assert(position == position_at(level_index as int, index_in_level as int));
            }
            let is_rewritten = match &suffix_starts {
                Some(starts) => index_in_level >= starts[level_index],
                None => false,
            };
            proof {
                if rightmost_count == 0 {
                    assert(!node_is_an_ancestor_of_one_of(levels, level_index as int, index_in_level as int, self.rewritten_containers@));
                }
                assert(is_rewritten == rewritten.contains(position));
            }
            if is_rewritten {
                node_origins.push((position, ContainerTreeNodeOrigin::RewrittenThisPublish));
            } else {
                proof {
                    assert(!node_is_an_ancestor_of_one_of(levels, level_index as int, index_in_level as int, self.rewritten_containers@));
                    assert(node_is_counted(self.node_counts_before_this_publish.counts(), level_index as int, index_in_level as int));
                }
                let level = position.level;
                let index_in_level_of_the_position = position.index_in_level;
                assert!(
                    self.node_counts_before_this_publish.has_a_node_at(position),
                    "照抄的容器树节点（层级 {level}、同层第 {index_in_level_of_the_position} 个）在上一版里有：只有最右那一条路径上的节点是新开的"
                );
                node_origins.push((position, ContainerTreeNodeOrigin::CarriedFromThePreviousVersion));
            }
            listed += 1;
        }
        node_origins
    }
}

/// 要开一片新容器时下一个容器号装不下：最右那片的号（盘上读来的 8 字节）已是 `u64::MAX`，加一越过顶（C841
/// 「写路径报类型化错误」：开新容器那一次发布报这个错、不 panic，读盘不变）。容器号从 1 起逐片加一，健康的树走不到顶，
/// 坏镜像、外来镜像才有。`tree` 是要开新容器的那棵树（墓碑树、谱系树或退役树），`rightmost_container` 是最右那片的号。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NextContainerNumberPastTheTopOfItsRange {
    pub tree: TreeIdentifier,
    pub rightmost_container: u64,
}

/// 第 `position` 片是这次新开的：身份照参数、容器号比左邻大一（第 0 片是 1）、装了 1 到 `records_per_container` 条。
pub open spec fn container_is_opened_by_this_publish<Record>(
    containers: Seq<RecordContainer<Record>>,
    position: int,
    tree: TreeIdentifier,
    record_type: u16,
    checkpoint_txg: CheckpointTxg,
    records_per_container: int,
) -> bool {
    let identity = containers[position].identity;
    &&& identity.birth_tree == tree
    &&& identity.record_type == record_type
    &&& identity.container_birth == checkpoint_txg
    &&& identity.container == if position == 0 {
        1
    } else {
        containers[position - 1].identity.container + 1
    }
    &&& 1 <= containers[position].records@.len() <= records_per_container
}

/// 追加到这一条为止，容器、形状与重写清单对上一版与已追加的记录逐条成立的那几样（[`append_records_to_the_open_container`] 的循环不变量与 `ensures`）：
/// 「key 到条目」——全部记录依次连起来 = 上一版的接上已追加的；最右那片之前的容器原样；最右那片身份不变、原有记录是前缀；
/// 新开的容器身份照参数、容器号逐片加一；形状在最右端长出来、容器数等于片数；重写清单升序、罩住新开的每一片、最后一项是最右那片，
/// 而且就是最右端连着的那几片（[`rewritten_containers_are_the_rightmost`]：发布路径按它判第几片、哪个节点重写了，不在清单上逐个找）；
/// 一条都没追加时什么都没动；不在上一版的节点都罩着重写的某一片（照抄的节点上一版都有）。
pub open spec fn appending_progress<Record>(
    containers_before_this_publish: Seq<RecordContainer<Record>>,
    shape_before_this_publish: Seq<Seq<usize>>,
    appended_records: Seq<Record>,
    checkpoint_txg: CheckpointTxg,
    tree: TreeIdentifier,
    record_type: u16,
    records_per_container: int,
    containers: Seq<RecordContainer<Record>>,
    levels: Seq<Seq<usize>>,
    rewritten_containers: Seq<usize>,
) -> bool {
    let before = containers_before_this_publish;
    let before_count = before.len();
    &&& records_of_containers(containers) == records_of_containers(before) + appended_records
    &&& containers.len() >= before_count
    &&& forall|position: int|
        0 <= position < before_count - 1 ==> (#[trigger] containers[position]).identity == before[position].identity
            && containers[position].records@ == before[position].records@
    &&& before_count >= 1 ==> containers[before_count - 1].identity == before[before_count - 1].identity
    &&& before_count >= 1 ==> containers[before_count - 1].records@.take(before[before_count - 1].records@.len() as int)
        == before[before_count - 1].records@
    &&& before_count >= 1 ==> containers[before_count - 1].records@.len() >= before[before_count - 1].records@.len()
    &&& forall|position: int|
        before_count <= position < containers.len() ==> #[trigger] container_is_opened_by_this_publish(
            containers,
            position,
            tree,
            record_type,
            checkpoint_txg,
            records_per_container,
        )
    &&& container_count(levels) == containers.len()
    &&& levels_extend(shape_before_this_publish, levels)
    &&& forall|earlier: int, later: int|
        0 <= earlier < later < rewritten_containers.len() ==> #[trigger] rewritten_containers[earlier]
            < #[trigger] rewritten_containers[later]
    &&& forall|listed: int| 0 <= listed < rewritten_containers.len() ==> #[trigger] rewritten_containers[listed] < containers.len()
    &&& forall|position: int|
        before_count <= position < containers.len() ==> #[trigger] rewritten_containers.contains(position as usize)
    &&& appended_records.len() == 0 ==> rewritten_containers.len() == 0 && containers_are_the_same(before, containers)
        && levels == shape_before_this_publish
    &&& appended_records.len() > 0 ==> rewritten_containers.len() >= 1 && rewritten_containers.last() == containers.len() - 1
    &&& rewritten_containers_are_the_rightmost(rewritten_containers, containers.len() as int)
    &&& every_node_absent_from_the_previous_version_is_rewritten(node_counts_of(shape_before_this_publish), levels, rewritten_containers)
}

/// [`append_records_to_the_open_container`] 的 `ensures`：交回的样子对上一版、这次给的记录与参数照 [`appending_progress`] 成立，
/// 这次之前那一版记的每层节点数就是上一版形状的。
pub open spec fn appending_keeps_the_key_to_entry_mapping<Record>(
    containers_before_this_publish: Seq<RecordContainer<Record>>,
    shape_before_this_publish: Seq<Seq<usize>>,
    records: Seq<Record>,
    checkpoint_txg: CheckpointTxg,
    tree: TreeIdentifier,
    record_type: u16,
    records_per_container: int,
    after: ContainerTreeAfterThisPublish<Record>,
) -> bool {
    &&& after.node_counts_before_this_publish.counts() == node_counts_of(shape_before_this_publish)
    &&& appending_progress(
        containers_before_this_publish,
        shape_before_this_publish,
        records,
        checkpoint_txg,
        tree,
        record_type,
        records_per_container,
        after.containers@,
        after.shape.levels(),
        after.rewritten_containers@,
    )
}

/// 一条都还没追加时，照抄上一版就对上了 [`appending_progress`]。
proof fn lemma_appending_progress_at_the_start<Record>(
    containers_before_this_publish: Seq<RecordContainer<Record>>,
    shape_before_this_publish: Seq<Seq<usize>>,
    checkpoint_txg: CheckpointTxg,
    tree: TreeIdentifier,
    record_type: u16,
    records_per_container: int,
    containers: Seq<RecordContainer<Record>>,
)
    requires
        container_tree_shape_is_well_formed(shape_before_this_publish),
        container_count(shape_before_this_publish) == containers_before_this_publish.len(),
        containers_are_the_same(containers_before_this_publish, containers),
        records_of_containers(containers) == records_of_containers(containers_before_this_publish),
    ensures
        appending_progress(
            containers_before_this_publish,
            shape_before_this_publish,
            Seq::empty(),
            checkpoint_txg,
            tree,
            record_type,
            records_per_container,
            containers,
            shape_before_this_publish,
            Seq::empty(),
        ),
{
    let before = containers_before_this_publish;
    assert(records_of_containers(before) + Seq::<Record>::empty() =~= records_of_containers(before));
    if before.len() >= 1 {
        assert(containers[before.len() - 1].records@ == before[before.len() - 1].records@);
        assert(containers[before.len() - 1].records@.take(before[before.len() - 1].records@.len() as int)
            =~= before[before.len() - 1].records@);
    }
    assert forall|level_index: int| 0 <= level_index < shape_before_this_publish.len() implies nodes_extend(
        #[trigger] shape_before_this_publish[level_index],
        shape_before_this_publish[level_index],
    ) by {
        assert(shape_before_this_publish[level_index].len() >= 1);
    }
    assert forall|level_index: int, node: int|
        #[trigger] node_is_in_the_levels(shape_before_this_publish, level_index, node) implies node_is_counted(
        node_counts_of(shape_before_this_publish),
        level_index,
        node,
    ) by {}
}

/// 往还装得下的最右那片里追加一条之后，[`appending_progress`] 照样成立。
proof fn lemma_appending_into_the_open_container_keeps_the_progress<Record>(
    containers_before_this_publish: Seq<RecordContainer<Record>>,
    shape_before_this_publish: Seq<Seq<usize>>,
    appended_records: Seq<Record>,
    record: Record,
    checkpoint_txg: CheckpointTxg,
    tree: TreeIdentifier,
    record_type: u16,
    records_per_container: int,
    containers: Seq<RecordContainer<Record>>,
    levels: Seq<Seq<usize>>,
    rewritten_containers: Seq<usize>,
    containers_after: Seq<RecordContainer<Record>>,
    rewritten_after: Seq<usize>,
)
    requires
        appending_progress(
            containers_before_this_publish,
            shape_before_this_publish,
            appended_records,
            checkpoint_txg,
            tree,
            record_type,
            records_per_container,
            containers,
            levels,
            rewritten_containers,
        ),
        1 <= containers.len() <= usize::MAX,
        containers.last().records@.len() < records_per_container,
        containers_after.len() == containers.len(),
        containers_after.drop_last() == containers.drop_last(),
        containers_after.last().identity == containers.last().identity,
        containers_after.last().records@ == containers.last().records@.push(record),
        rewritten_after == if rewritten_containers.len() >= 1 && rewritten_containers.last() == containers.len() - 1 {
            rewritten_containers
        } else {
            rewritten_containers.push((containers.len() - 1) as usize)
        },
    ensures
        appending_progress(
            containers_before_this_publish,
            shape_before_this_publish,
            appended_records.push(record),
            checkpoint_txg,
            tree,
            record_type,
            records_per_container,
            containers_after,
            levels,
            rewritten_after,
        ),
{
    let before = containers_before_this_publish;
    let before_count = before.len();
    let last = containers.len() - 1;
    lemma_records_of_containers_after_appending_to_the_last(containers, containers_after, record);
    assert(records_of_containers(before) + appended_records.push(record) =~= (records_of_containers(before)
        + appended_records).push(record));
    assert forall|position: int| 0 <= position < last implies #[trigger] containers_after[position] == containers[position] by {
        assert(containers_after.drop_last()[position] == containers_after[position]);
        assert(containers.drop_last()[position] == containers[position]);
        assert(containers_after.drop_last()[position] == containers.drop_last()[position]);
    }
    assert(containers_after[last].identity == containers[last].identity);
    if before_count >= 1 {
        assert(containers_after[before_count - 1].identity == before[before_count - 1].identity);
        assert(containers_after[before_count - 1].records@.len() >= before[before_count - 1].records@.len());
    }
    assert forall|position: int| 0 <= position < before_count - 1 implies (#[trigger] containers_after[position]).identity
        == before[position].identity && containers_after[position].records@ == before[position].records@ by {
        assert(containers_after[position] == containers[position]);
    }
    assert(container_count(levels) == containers_after.len());
    if before_count >= 1 && last == before_count - 1 {
        let earlier_records = before[before_count - 1].records@;
        assert(containers_after[last].records@.take(earlier_records.len() as int) =~= containers[last].records@.take(
            earlier_records.len() as int,
        ));
    }
    assert forall|position: int| before_count <= position < containers_after.len() implies #[trigger] container_is_opened_by_this_publish(
        containers_after,
        position,
        tree,
        record_type,
        checkpoint_txg,
        records_per_container,
    ) by {
        assert(container_is_opened_by_this_publish(
            containers,
            position,
            tree,
            record_type,
            checkpoint_txg,
            records_per_container,
        ));
        if position >= 1 && position - 1 < last {
            assert(containers_after[position - 1] == containers[position - 1]);
        }
    }
    if appended_records.len() == 0 {
        assert(rewritten_containers.len() == 0);
        assert(rewritten_after =~= seq![last as usize]);
        assert(rewritten_after.last() == last as usize);
        assert(rewritten_after.len() == 1);
        lemma_more_rewritten_containers_keep_every_new_node_rewritten(
            node_counts_of(shape_before_this_publish),
            levels,
            rewritten_containers,
            last as usize,
        );
        assert forall|position: int| before_count <= position < containers_after.len() implies #[trigger] rewritten_after.contains(
            position as usize,
        ) by {
            assert(rewritten_after[0] == position as usize);
        }
    } else {
        assert(rewritten_after == rewritten_containers);
        assert(rewritten_after.last() == containers.len() - 1);
    }
    let appended_after = appended_records.push(record);
    assert(records_of_containers(containers_after) == records_of_containers(before) + appended_after);
    assert(containers_after.len() >= before_count);
    assert(before_count >= 1 ==> containers_after[before_count - 1].records@.take(before[before_count - 1].records@.len() as int)
        == before[before_count - 1].records@);
    assert(levels_extend(shape_before_this_publish, levels));
    assert(forall|earlier: int, later: int|
        0 <= earlier < later < rewritten_after.len() ==> #[trigger] rewritten_after[earlier] < #[trigger] rewritten_after[later]);
    assert(forall|listed: int| 0 <= listed < rewritten_after.len() ==> #[trigger] rewritten_after[listed] < containers_after.len());
    assert(appended_after.len() > 0 ==> rewritten_after.len() >= 1 && rewritten_after.last() == containers_after.len() - 1);
    assert(every_node_absent_from_the_previous_version_is_rewritten(node_counts_of(shape_before_this_publish), levels, rewritten_after));
    assert forall|listed: int| 0 <= listed < rewritten_after.len() implies #[trigger] rewritten_after[listed] == containers_after.len()
        - rewritten_after.len() + listed by {
        if appended_records.len() == 0 {
            assert(rewritten_after[0] == last as usize);
        } else {
            assert(rewritten_after[listed] == rewritten_containers[listed]);
        }
    }
    assert(rewritten_containers_are_the_rightmost(rewritten_after, containers_after.len() as int));
}

/// 最右那片装满（或树是空的）、开一片新容器追加一条之后，[`appending_progress`] 照样成立。
proof fn lemma_opening_a_container_keeps_the_progress<Record>(
    containers_before_this_publish: Seq<RecordContainer<Record>>,
    shape_before_this_publish: Seq<Seq<usize>>,
    appended_records: Seq<Record>,
    record: Record,
    checkpoint_txg: CheckpointTxg,
    tree: TreeIdentifier,
    record_type: u16,
    records_per_container: int,
    entries_per_node: int,
    containers: Seq<RecordContainer<Record>>,
    levels: Seq<Seq<usize>>,
    rewritten_containers: Seq<usize>,
    opened: RecordContainer<Record>,
    rewritten_after: Seq<usize>,
)
    requires
        appending_progress(
            containers_before_this_publish,
            shape_before_this_publish,
            appended_records,
            checkpoint_txg,
            tree,
            record_type,
            records_per_container,
            containers,
            levels,
            rewritten_containers,
        ),
        container_tree_shape_is_well_formed(levels),
        container_count(levels) + 1 <= usize::MAX,
        2 <= entries_per_node <= usize::MAX,
        records_per_container >= 1,
        opened.identity == (PackedIdentity {
            birth_tree: tree,
            record_type,
            container: if containers.len() == 0 {
                1
            } else {
                (containers.last().identity.container + 1) as u64
            },
            container_birth: checkpoint_txg,
        }),
        containers.len() >= 1 ==> containers.last().identity.container + 1 <= u64::MAX,
        opened.records@ == seq![record],
        rewritten_after == if rewritten_containers.len() >= 1 && rewritten_containers.last() == containers.len() {
            rewritten_containers
        } else {
            rewritten_containers.push(containers.len() as usize)
        },
    ensures
        appending_progress(
            containers_before_this_publish,
            shape_before_this_publish,
            appended_records.push(record),
            checkpoint_txg,
            tree,
            record_type,
            records_per_container,
            containers.push(opened),
            levels_after_opening_at_the_right_end(levels, entries_per_node),
            rewritten_after,
        ),
{
    let before = containers_before_this_publish;
    let before_count = before.len();
    let containers_after = containers.push(opened);
    let levels_after = levels_after_opening_at_the_right_end(levels, entries_per_node);
    lemma_records_of_containers_after_a_push(containers, opened);
    assert(records_of_containers(before) + appended_records.push(record) =~= (records_of_containers(before)
        + appended_records) + seq![record]);
    assert forall|position: int| 0 <= position < containers.len() implies #[trigger] containers_after[position]
        == containers[position] by {}
    assert forall|position: int| before_count <= position < containers_after.len() implies #[trigger] container_is_opened_by_this_publish(
        containers_after,
        position,
        tree,
        record_type,
        checkpoint_txg,
        records_per_container,
    ) by {
        if position < containers.len() {
            assert(container_is_opened_by_this_publish(
                containers,
                position,
                tree,
                record_type,
                checkpoint_txg,
                records_per_container,
            ));
            if position >= 1 {
                assert(containers_after[position - 1] == containers[position - 1]);
            }
        } else {
            if position >= 1 {
                assert(containers_after[position - 1] == containers.last());
            }
        }
    }
    lemma_opening_keeps_the_shape_well_formed(levels, entries_per_node);
    lemma_opening_extends_the_levels_at_the_right_end(levels, entries_per_node);
    lemma_levels_extend_transitively(shape_before_this_publish, levels, levels_after);
    assert forall|listed: int| 0 <= listed < rewritten_containers.len() implies #[trigger] rewritten_containers[listed]
        < container_count(levels) by {}
    lemma_opening_keeps_every_new_node_rewritten(
        node_counts_of(shape_before_this_publish),
        levels,
        entries_per_node,
        rewritten_containers,
    );
    assert(rewritten_after == rewritten_containers.push(containers.len() as usize));
    assert forall|position: int| before_count <= position < containers_after.len() implies #[trigger] rewritten_after.contains(
        position as usize,
    ) by {
        if position < containers.len() {
            assert(rewritten_containers.contains(position as usize));
            let found = choose|found: int|
                0 <= found < rewritten_containers.len() && rewritten_containers[found] == position as usize;
            assert(rewritten_after[found] == position as usize);
        } else {
            assert(rewritten_after[rewritten_containers.len() as int] == position as usize);
        }
    }
    assert forall|earlier: int, later: int| 0 <= earlier < later < rewritten_after.len() implies #[trigger] rewritten_after[earlier]
        < #[trigger] rewritten_after[later] by {
        if later < rewritten_containers.len() {
            assert(rewritten_after[earlier] == rewritten_containers[earlier]);
            assert(rewritten_after[later] == rewritten_containers[later]);
        } else {
            assert(rewritten_after[earlier] == rewritten_containers[earlier]);
        }
    }
    assert forall|listed: int| 0 <= listed < rewritten_after.len() implies #[trigger] rewritten_after[listed] == containers_after.len()
        - rewritten_after.len() + listed by {
        if listed < rewritten_containers.len() {
            assert(rewritten_after[listed] == rewritten_containers[listed]);
        } else {
            assert(rewritten_after[listed] == containers.len());
        }
    }
    assert(rewritten_containers_are_the_rightmost(rewritten_after, containers_after.len() as int));
    if appended_records.len() == 0 {
        assert(levels == shape_before_this_publish);
    }
}

/// 往容器树的开放容器（最右那一片）追加一条记录：[`append_records_to_the_open_container`] 只给一条的那一格。
///
/// # Errors
/// 同 [`append_records_to_the_open_container`]。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：上一版的容器与形状、这条记录、txg、树、记录类型、一片装几条、一个节点装几条"
)]
pub fn append_record_to_the_open_container<Record: Copy>(
    containers_before_this_publish: &[RecordContainer<Record>],
    shape_before_this_publish: &ContainerTreeShape,
    record: Record,
    checkpoint_txg: CheckpointTxg,
    tree: TreeIdentifier,
    record_type: u16,
    records_per_container: usize,
    node_entries: ContainerTreeNodeEntries,
) -> (after: Result<ContainerTreeAfterThisPublish<Record>, NextContainerNumberPastTheTopOfItsRange>)
    requires
        container_count(shape_before_this_publish.levels()) == containers_before_this_publish.len(),
        records_per_container >= 1,
    ensures
        match after {
            Ok(after) => appending_keeps_the_key_to_entry_mapping(
                containers_before_this_publish@,
                shape_before_this_publish.levels(),
                seq![record],
                checkpoint_txg,
                tree,
                record_type,
                records_per_container as int,
                after,
            ),
            Err(error) => error.tree == tree && error.rightmost_container == u64::MAX,
        },
{
    let records = vec![record];
    proof {
        assert(records@ =~= seq![record]);
    }
    append_records_to_the_open_container(
        containers_before_this_publish,
        shape_before_this_publish,
        records.as_slice(),
        checkpoint_txg,
        tree,
        record_type,
        records_per_container,
        node_entries,
    )
}

/// 往容器树的开放容器（最右那一片）按给的次序逐条追加记录：最右那片还装得下（记录数小于 `records_per_container`）就追加进去；装满了
/// （或树是空的）就开一片新容器：容器号 = 树里最大号 + 1（零根时 1），出生代 = 这次发布的 checkpoint_txg，出生树 = `tree`，
/// 打包记录类型 = `record_type`，形状照 [`ContainerTreeShape::after_a_container_is_opened_at_the_right_end`] 往上挂（根满长高）。
/// 这次追加过的每一片进重写清单（叶序升序、各一次）；别的容器一个字节都不动。`records` 为空时整棵照抄（[`ContainerTreeAfterThisPublish::carried`]）。
///
/// # Errors
/// 要开新容器而最右那片的号已是 `u64::MAX` ⇒ [`NextContainerNumberPastTheTopOfItsRange`]（C841）：在开那一片之前交回，
/// 只算、不写，调用方照样一个字节都没写。最右那片还装得下时照常追加，不看它的号。
///
/// # Panics
/// `shape_before_this_publish` 里的容器数与 `containers_before_this_publish` 不等：两样出自同一版。`records_per_container` 为 0：
/// 三棵树的容量开关的构造函数都判过至少 1。两样都是 `requires` 排除的。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：上一版的容器与形状、这几条记录、txg、树、记录类型、一片装几条、一个节点装几条"
)]
pub fn append_records_to_the_open_container<Record: Copy>(
    containers_before_this_publish: &[RecordContainer<Record>],
    shape_before_this_publish: &ContainerTreeShape,
    records: &[Record],
    checkpoint_txg: CheckpointTxg,
    tree: TreeIdentifier,
    record_type: u16,
    records_per_container: usize,
    node_entries: ContainerTreeNodeEntries,
) -> (after: Result<ContainerTreeAfterThisPublish<Record>, NextContainerNumberPastTheTopOfItsRange>)
    requires
        container_count(shape_before_this_publish.levels()) == containers_before_this_publish.len(),
        records_per_container >= 1,
    ensures
        match after {
            Ok(after) => appending_keeps_the_key_to_entry_mapping(
                containers_before_this_publish@,
                shape_before_this_publish.levels(),
                records@,
                checkpoint_txg,
                tree,
                record_type,
                records_per_container as int,
                after,
            ),
            Err(error) => error.tree == tree && error.rightmost_container == u64::MAX,
        },
{
    assert!(
        shape_before_this_publish.containers() == containers_before_this_publish.len(),
        "上一版容器树的形状与它的容器出自同一版"
    );
    assert!(
        records_per_container >= 1,
        "一片容器至少装一条：容量开关的构造函数判过"
    );
    let ghost before = containers_before_this_publish@;
    let ghost levels_before = shape_before_this_publish.levels();
    let mut containers = copy_of_the_containers(containers_before_this_publish);
    let mut shape = shape_before_this_publish.clone();
    let mut rewritten_containers: Vec<usize> = Vec::new();
    let entries_per_node = node_entries.entries_per_node();
    proof {
        use_type_invariant(shape_before_this_publish);
        lemma_appending_progress_at_the_start(
            before,
            levels_before,
            checkpoint_txg,
            tree,
            record_type,
            records_per_container as int,
            containers@,
        );
        assert(records@.take(0) =~= Seq::<Record>::empty());
        assert(rewritten_containers@ =~= Seq::<usize>::empty());
    }
    // 迭代次数 = 这次追加的条数；跨轮携带的是容器与形状（形状里的容器数恒等于容器数）与升序不重复的重写清单。
    let mut record_position: usize = 0;
    while record_position < records.len()
        invariant
            record_position <= records.len(),
            before == containers_before_this_publish@,
            levels_before == shape_before_this_publish.levels(),
            2 <= entries_per_node <= 135,
            records_per_container >= 1,
            appending_progress(
                before,
                levels_before,
                records@.take(record_position as int),
                checkpoint_txg,
                tree,
                record_type,
                records_per_container as int,
                containers@,
                shape.levels(),
                rewritten_containers@,
            ),
        decreases records.len() - record_position,
    {
        proof {
            use_type_invariant(&shape);
        }
        let record = records[record_position];
        let ghost containers_at_the_start = containers@;
        let ghost levels_at_the_start = shape.levels();
        let ghost rewritten_at_the_start = rewritten_containers@;
        let open_container_has_room = !containers.is_empty()
            && containers[containers.len() - 1].records.len() < records_per_container;
        if open_container_has_room {
            let mut open = containers.pop().expect("上一句判过最右那片在");
            open.records.push(record);
            containers.push(open);
        } else {
            // 容器号沿叶序严格递增、从 1 起（0 不用），下一个号 = 树里最大号 + 1：最右那片的号就是最大号。
            // 最右那片的号是盘上读来的 8 字节：加一越过顶的交回类型化错误（C841），不 panic。
            let next_container_number = if containers.is_empty() {
                1
            } else {
                let rightmost_container = containers[containers.len() - 1].identity.container;
                rightmost_container.checked_add(1).ok_or(
                    NextContainerNumberPastTheTopOfItsRange { tree, rightmost_container },
                )?
            };
            proof {
                assert(container_count(levels_at_the_start) == containers_at_the_start.len());
            }
            containers.push(
                RecordContainer {
                    identity: PackedIdentity {
                        birth_tree: tree,
                        record_type,
                        container: next_container_number,
                        container_birth: checkpoint_txg,
                    },
                    records: vec_of_one(record),
                },
            );
            proof {
                assert(containers@.len() == containers_at_the_start.len() + 1);
                assert(containers.len() <= usize::MAX);
            }
            shape = shape.after_a_container_is_opened_at_the_right_end(entries_per_node);
        }
        let rewritten_position = containers.len() - 1;
        if rewritten_containers.is_empty() || rewritten_containers[rewritten_containers.len() - 1] != rewritten_position {
            rewritten_containers.push(rewritten_position);
        }
        proof {
            if open_container_has_room {
                assert(containers@.drop_last() =~= containers_at_the_start.drop_last());
                assert(containers_at_the_start.len() == containers.len());
                lemma_appending_into_the_open_container_keeps_the_progress(
                    before,
                    levels_before,
                    records@.take(record_position as int),
                    record,
                    checkpoint_txg,
                    tree,
                    record_type,
                    records_per_container as int,
                    containers_at_the_start,
                    levels_at_the_start,
                    rewritten_at_the_start,
                    containers@,
                    rewritten_containers@,
                );
            } else {
                assert(containers@ == containers_at_the_start.push(containers@.last()));
                lemma_opening_a_container_keeps_the_progress(
                    before,
                    levels_before,
                    records@.take(record_position as int),
                    record,
                    checkpoint_txg,
                    tree,
                    record_type,
                    records_per_container as int,
                    entries_per_node as int,
                    containers_at_the_start,
                    levels_at_the_start,
                    rewritten_at_the_start,
                    containers@.last(),
                    rewritten_containers@,
                );
            }
            assert(records@.take(record_position as int).push(record) =~= records@.take(record_position as int + 1));
        }
        assert!(
            shape.containers() == containers.len(),
            "形状里层级 1 的孩子数之和就是容器数"
        );
        record_position += 1;
    }
    proof {
        assert(records@.take(records.len() as int) =~= records@);
    }
    Ok(
        ContainerTreeAfterThisPublish {
            containers,
            rewritten_containers,
            shape,
            node_counts_before_this_publish: shape_before_this_publish.node_counts(),
        },
    )
}

/// 只装一条的一组记录（新开容器的第一条）。
fn vec_of_one<Record: Copy>(record: Record) -> (records: Vec<Record>)
    ensures
        records@ == seq![record],
{
    let records = vec![record];
    assert(records@ =~= seq![record]);
    records
}

} // verus!

#[cfg(test)]
mod tests {
    use super::*;
    use crate::container_tree::tests::{
        ancestors_by_marking_every_container, container_tree_node_entries_capped_at, position,
        shape_after_opening, shapes_for_the_ancestor_differential,
    };

    /// 长高那次重写的是新开的容器、右半与新根（最右那一条路径），旧根照抄、位置不变；从盘上读回来的孩子数换回原样的形状。
    #[test]
    fn the_growth_publish_rewrites_the_rightmost_path_and_carries_the_old_root() {
        let before = shape_after_opening(2, 2);
        let containers: Vec<RecordContainer<u64>> = (1..=2)
            .map(|number| RecordContainer {
                identity: PackedIdentity {
                    birth_tree: TreeIdentifier(65545),
                    record_type: 1,
                    container: number,
                    container_birth: CheckpointTxg(5),
                },
                records: vec![number],
            })
            .collect();
        let after = append_record_to_the_open_container(
            &containers,
            &before,
            3_u64,
            CheckpointTxg(9),
            TreeIdentifier(65545),
            1,
            1,
            container_tree_node_entries_capped_at(2),
        )
        .expect("最右那片的号 2，加一装得下");
        assert!(after.grew_a_level_this_publish());
        assert_eq!(after.containers[2].identity.container, 3);
        assert_eq!(
            after.node_origins(),
            vec![
                (
                    position(1, 0),
                    ContainerTreeNodeOrigin::CarriedFromThePreviousVersion
                ),
                (
                    position(1, 1),
                    ContainerTreeNodeOrigin::RewrittenThisPublish
                ),
                (
                    position(2, 0),
                    ContainerTreeNodeOrigin::RewrittenThisPublish
                ),
            ]
        );
        assert_eq!(
            ContainerTreeShape::from_children_of_each_node_at_each_level(vec![vec![2, 1], vec![2]]),
            Some(after.shape.clone())
        );
        assert_eq!(
            ContainerTreeShape::from_children_of_each_node_at_each_level(vec![vec![2, 1], vec![1]]),
            None,
            "根的孩子数之和要等于下一层的节点数"
        );
    }

    /// 一次追加三条、一片装两条、扇出压到 2：上一版两片（第二片装着一条、还开着）。第一条装满第二片，第二、三条开第三片（这时根满长一层）。
    /// 这次重写的是第二、三片与它们的全部祖先：旧根 (1, 0) 下面的第二片被重写，旧根跟着重写、照样被这次换下。
    #[test]
    fn appending_several_records_rewrites_every_touched_container_and_all_their_ancestors() {
        let before_shape = shape_after_opening(2, 2);
        let identity = |number: u64| PackedIdentity {
            birth_tree: TreeIdentifier(65548),
            record_type: 3,
            container: number,
            container_birth: CheckpointTxg(5),
        };
        let before = vec![
            RecordContainer {
                identity: identity(1),
                records: vec![10_u64, 11],
            },
            RecordContainer {
                identity: identity(2),
                records: vec![12_u64],
            },
        ];
        let after = append_records_to_the_open_container(
            &before,
            &before_shape,
            &[13_u64, 14, 15],
            CheckpointTxg(9),
            TreeIdentifier(65548),
            3,
            2,
            container_tree_node_entries_capped_at(2),
        )
        .expect("最右那片的号 2，加一装得下");
        assert_eq!(after.containers.len(), 3);
        assert_eq!(after.containers[0], before[0], "装满了的第一片原样");
        assert_eq!(after.containers[1].records, vec![12, 13], "第二片装满");
        assert_eq!(
            after.containers[2].identity,
            PackedIdentity {
                birth_tree: TreeIdentifier(65548),
                record_type: 3,
                container: 3,
                container_birth: CheckpointTxg(9),
            },
            "第三片：容器号 = 最大号 + 1，出生代 = 这次的 txg"
        );
        assert_eq!(after.containers[2].records, vec![14, 15]);
        assert_eq!(after.rewritten_containers, vec![1, 2]);
        assert!(after.grew_a_level_this_publish());
        assert!(
            !after.previous_root_is_carried_down_unchanged(),
            "旧根下面的第二片重写了：旧根跟着重写"
        );
        assert_eq!(
            after.rewritten_nodes(),
            vec![position(1, 0), position(1, 1), position(2, 0)]
        );
        assert_eq!(after.first_rewritten_container(), Some(1));
        assert_eq!(
            after.shape.ancestors_of_the_rightmost_containers(1),
            vec![position(1, 1), position(2, 0)],
            "只看最右那一片时祖先就是最右那一条路径：右半 (1, 1) 与根 (2, 0)，旧根 (1, 0) 不在其内"
        );
        let carried = append_records_to_the_open_container(
            &after.containers,
            &after.shape,
            &[],
            CheckpointTxg(10),
            TreeIdentifier(65548),
            3,
            2,
            container_tree_node_entries_capped_at(2),
        )
        .expect("一条都不追加，不开新容器");
        assert!(carried.rewritten_containers.is_empty() && carried.rewritten_nodes().is_empty());
        assert_eq!(carried.first_rewritten_container(), None);
        assert_eq!(
            carried.containers, after.containers,
            "一条都不追加时整棵照抄"
        );
    }

    /// 同一批形状上：最右端 1 到 n 片重写时，`node_origins` 按每层那一段起点判出的「这次重写」恰好是按全部容器数开标记表找出的祖先，
    /// 别的都是照抄（上一版取同一份形状，每个位置上一版都有）。一片都不重写时一个节点都不重写。合计 3235 + 151 组。
    #[test]
    fn node_origins_mark_exactly_the_ancestors_of_the_rightmost_rewritten_containers() {
        let mut compared: usize = 0;
        for shape in shapes_for_the_ancestor_differential() {
            let container_total = shape.containers();
            for rightmost_count in 0..=container_total {
                let rewritten_containers: Vec<usize> =
                    (container_total - rightmost_count..container_total).collect();
                let expected_rewritten =
                    ancestors_by_marking_every_container(&shape, &rewritten_containers);
                let after: ContainerTreeAfterThisPublish<u64> = ContainerTreeAfterThisPublish {
                    containers: (0..container_total)
                        .map(|_| RecordContainer {
                            identity: IDENTITY_OF_A_CODE_TWO_CHILD,
                            records: Vec::new(),
                        })
                        .collect(),
                    rewritten_containers,
                    node_counts_before_this_publish: shape.node_counts(),
                    shape: shape.clone(),
                };
                let origins = after.node_origins();
                assert_eq!(
                    origins.iter().map(|(node, _)| *node).collect::<Vec<_>>(),
                    shape.positions_in_bump_order(),
                    "形状 {shape:?}：按 bump 次序列全部节点"
                );
                for (node, origin) in origins {
                    let expected_origin = if expected_rewritten.contains(&node) {
                        ContainerTreeNodeOrigin::RewrittenThisPublish
                    } else {
                        ContainerTreeNodeOrigin::CarriedFromThePreviousVersion
                    };
                    assert_eq!(
                        origin, expected_origin,
                        "形状 {shape:?} 上最右端 {rightmost_count} 片重写时节点 {node:?}"
                    );
                }
                compared += 1;
            }
        }
        assert_eq!(
            compared, 3_386,
            "对拍了 151 份形状上最右端的每一组与一片都不重写的那一组"
        );
    }

    /// 这次之前那一版只记每层节点数：一次追加长了一层，记着的是上一版的两层（层级 1 两个节点、根一个），据它判哪个位置上一版有节点、哪个是上一版的根。
    #[test]
    fn the_version_before_this_publish_is_kept_as_the_node_count_of_each_level() {
        let before_shape = shape_after_opening(3, 2);
        let containers: Vec<RecordContainer<u64>> = (1..=3)
            .map(|number| RecordContainer {
                identity: PackedIdentity {
                    birth_tree: TreeIdentifier(65545),
                    record_type: 1,
                    container: number,
                    container_birth: CheckpointTxg(5),
                },
                records: vec![number],
            })
            .collect();
        let after = append_records_to_the_open_container(
            &containers,
            &before_shape,
            &[4_u64, 5],
            CheckpointTxg(9),
            TreeIdentifier(65545),
            1,
            1,
            container_tree_node_entries_capped_at(2),
        )
        .expect("最右那片的号 3，加一装得下");
        assert_eq!(
            after.node_counts_before_this_publish,
            ContainerTreeNodeCounts {
                nodes_at_each_level: vec![2, 1]
            }
        );
        assert_eq!(
            after.node_counts_before_this_publish,
            before_shape.node_counts()
        );
        assert!(after
            .node_counts_before_this_publish
            .has_a_node_at(position(1, 1)));
        assert!(!after
            .node_counts_before_this_publish
            .has_a_node_at(position(1, 2)));
        assert!(!after
            .node_counts_before_this_publish
            .has_a_node_at(position(3, 0)));
        assert!(after
            .node_counts_before_this_publish
            .is_the_root(position(2, 0)));
        assert_eq!(
            after.node_counts_before_this_publish.root_position(),
            Some(position(2, 0))
        );
        assert_eq!(after.rewritten_containers, vec![3, 4]);
        assert_eq!(after.first_rewritten_container(), Some(3));
        assert!(after.grew_a_level_this_publish());
    }

    /// C841：最右那片的号是 `u64::MAX`（坏镜像上读来的）。它还装得下时照常追加进它；装满了、要开新容器时交回
    /// `NextContainerNumberPastTheTopOfItsRange`、点名树与最右那片的号，不 panic。
    #[test]
    fn opening_a_container_after_one_numbered_at_the_top_of_its_range_returns_an_error() {
        let rightmost_at_the_top = RecordContainer {
            identity: PackedIdentity {
                birth_tree: TreeIdentifier(65545),
                record_type: 1,
                container: u64::MAX,
                container_birth: CheckpointTxg(5),
            },
            records: vec![10_u64],
        };
        let before = vec![rightmost_at_the_top];
        let shape_before = shape_after_opening(1, 2);
        let appended_into_the_open_container = append_records_to_the_open_container(
            &before,
            &shape_before,
            &[11_u64],
            CheckpointTxg(9),
            TreeIdentifier(65545),
            1,
            2,
            container_tree_node_entries_capped_at(2),
        )
        .expect("最右那片还装得下：不开新容器，不看它的号");
        assert_eq!(
            appended_into_the_open_container.containers[0].records,
            vec![10, 11]
        );
        assert_eq!(
            append_records_to_the_open_container(
                &before,
                &shape_before,
                &[11_u64],
                CheckpointTxg(9),
                TreeIdentifier(65545),
                1,
                1,
                container_tree_node_entries_capped_at(2),
            ),
            Err(NextContainerNumberPastTheTopOfItsRange {
                tree: TreeIdentifier(65545),
                rightmost_container: u64::MAX,
            }),
            "最右那片装满、要开新容器：号加一越过顶，交回类型化错误"
        );
    }
}
