//! inode 树的叶容器：一容器 233 条 140 字节记录，满了就在末尾分裂（里程碑「第二个事务」并行线三的写路径）。
//!
//! 压着它的条款是 D8（核心索引结构） 已定项 6 的三条写路径纪律，逐字：
//! - 「inode 号在每一条时间线上单调（回退回到 R_old 的树与它的水位，之后发的号仍大于那棵树里每一个 key）⇒
//!   插入永远落在最右的叶 ⇒ 分裂只发生在最右叶，**分裂点取末尾**」；
//! - 「左半保留全部原有记录与身份，右半从触发分裂的那条新记录起，**右半容器号 = 那条新记录的 inode 号**，
//!   出生代 = 本次发布的 checkpoint_txg，**出生树 = 执行这次分裂的那棵树**（分裂是新建，与「COW 重写身份不变」是两件事）」；
//! - 「第一个容器的号 = 第一条记录的 inode 号」。
//!
//! 这三条今天没有会失败的检查（C116（叶容器的分裂 / 合并纪律没有会失败的检查）），
//! [invariants.md](../../../.claude/kb/invariants.md) 逐字把它们排除在 I-9（inode 树结构） 之外：
//! 「分裂 / 合并的身份传递……与合并时机是操作断言，对着一个镜像判不了，走单测 / 模型对拍与崩溃点重放」。
//! 这个模块就是那个能判的地方：它只算「这次发布之后树里是哪几片容器、各装哪些记录、哪几片要重写」，
//! 不发一个写、不动分配器、不碰落点，所以三条纪律在这里是纯函数的后置条件，单测判得了。
//!
//! **合并不在这个模块里**：这一版没有删除（`deleted_inodes` 的形态无落点，C118（`deleted_inodes` 树的形态无落点）），
//! 记录只增不减，合并（只许左吸收右 / 左从右借）没有对象。

use singlefs_format::{INODE_INTERNAL_ENTRY, INODE_LEAF_RECORDS};

use crate::address::{CheckpointTxg, InodeNumber, TreeIdentifier};
use crate::records::InodeRecord;
use crate::unit::{index_node_entry_capacity, PackedIdentity, PACKED_TYPE_INODE};

/// inode 树里第几片叶容器，按叶序（= key 升序）从 0 数。
///
/// 它是**树里的位置**，不是容器号：容器号是首次插入它的那条记录的 inode 号（D8（核心索引结构） 已定项 6），
/// 位置随左边有没有别的容器而变，两者不是一个量。这一版没有删除与合并 ⇒ 位置一旦定下就不再挪
/// （新容器只在最右端出现），所以它可以当发布路径里的角色序号用。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InodeLeafContainerIndexInTree(pub u32);

impl InodeLeafContainerIndexInTree {
    /// 最左那一片。树只有一片叶时就是它（第一个事务那一档）。
    pub const LEFTMOST: Self = Self(0);

    /// 当下标用：一棵树的容器数不会超过一个码 2 根装得下的条目数（135），`usize` 一定装得下。
    #[must_use]
    pub fn position(self) -> usize {
        usize::try_from(self.0).expect("叶容器序号小于一个码 2 根的条目数")
    }

    /// 第几片，从 0 数。
    #[must_use]
    pub fn of_position(position: usize) -> Self {
        Self(u32::try_from(position).expect("叶容器序号小于一个码 2 根的条目数"))
    }
}

/// 一片 inode 叶容器这一版装的东西：身份四段（出生树、打包记录类型、容器号、容器出生代）与它的记录，记录按 inode 号升序。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InodeLeafContainer {
    pub identity: PackedIdentity,
    pub records: Vec<InodeRecord>,
}

impl InodeLeafContainer {
    /// 容器号（身份里那 8 字节）：首次插入它的那条记录的 inode 号。
    #[must_use]
    pub fn container_number(&self) -> InodeNumber {
        InodeNumber(self.identity.container)
    }

    /// 它装的最小 inode 号；空容器没有（记录数为 0 的类型 2 容器不落盘，D8（核心索引结构） 已定项 6）。
    #[must_use]
    pub fn smallest_inode_number(&self) -> Option<InodeNumber> {
        self.records.first().map(|record| InodeNumber(record.inode))
    }

    /// 它装的最大 inode 号。
    #[must_use]
    pub fn largest_inode_number(&self) -> Option<InodeNumber> {
        self.records.last().map(|record| InodeNumber(record.inode))
    }

    /// 这片容器在父节点里的分隔 key：D8（核心索引结构） 已定项 6 逐字「**分隔 key** = 该孩子创建时的最小 key」。
    /// 创建时的最小 key 就是容器号——第一个容器的号 = 第一条记录的 inode 号，分裂出来的右半容器号 = 触发分裂那条新记录的 inode 号，
    /// 两者都是那片容器诞生那一刻装的第一条记录。**不取今天的最小 key**：容器被 COW 重写、记录增删之后
    /// 今天的最小 key 可以比诞生时大，而分隔 key 不许跟着变（跟着变就等于改父节点里的路由）。
    #[must_use]
    pub fn separator_key(&self) -> u64 {
        self.identity.container
    }

    /// 这片容器还能再装几条：一容器 233 条（D8（核心索引结构） 已定项 6，`INODE_LEAF_RECORDS`）。
    ///
    /// # Panics
    /// 这片容器已经装了多于 233 条。装记录的唯一入口是
    /// [`write_records_into_leaf_containers`]，它只在这个函数报「还装得下」时才往里追加 ⇒ 装不到 234 条；
    /// 装到了说明那条纪律被绕过了，不许当成「还能装 0 条」往下走。
    #[must_use]
    pub fn free_record_slots(&self) -> u64 {
        INODE_LEAF_RECORDS
            .checked_sub(u64::try_from(self.records.len()).expect("一容器 233 条"))
            .expect("一容器最多 233 条记录（D8 已定项 6）：装到第 234 条说明分裂纪律被绕过了")
    }
}

/// 一个码 2 根装得下几条 inode 内部条目：扇出 135（D8（核心索引结构） 已定项 6：分隔 key 8 + 身份引用 26 + 子指针 86 = 120）。
#[must_use]
pub fn leaf_containers_one_root_node_holds() -> usize {
    index_node_entry_capacity(8, usize::try_from(INODE_INTERNAL_ENTRY).expect("120"))
}

/// 这次发布之后 inode 树的叶容器，以及这次要重写哪几片。
///
/// 没被点到的那几片一个字节都不变（分裂时的左半就是这样：它的记录与身份都没动 ⇒ 不进这次的重写清单，
/// 连 COW 都不做），调用方照抄上一版的落点与指针。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InodeLeafContainersAfterThisPublish {
    /// 左起按 key 序的全部叶容器。
    pub containers: Vec<InodeLeafContainer>,
    /// 这次要重写的那几片，按叶序升序。
    pub rewritten: Vec<InodeLeafContainerIndexInTree>,
}

/// 往 inode 树里写记录时走得到、而条款没写的那几格。三样都在**任何落盘动作之前**交回，盘上逐字节不变。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InodeTreeWriteRefusal {
    /// 要写的 inode 号既不在树里、又不大于树里每一个 key：这条记录要插进中间某片叶。
    ///
    /// D8（核心索引结构） 已定项 6 的分裂纪律整条建在「inode 号在每一条时间线上单调 ⇒ 插入永远落在最右的叶 ⇒
    /// 分裂只发生在最右叶，分裂点取末尾」上；中间插入要的是非末尾分裂（左右两半各装哪些记录、身份给谁、
    /// 分隔 key 取什么），仓里一个字都没写。
    InsertWouldNotLandInTheRightmostLeaf {
        inode: InodeNumber,
        largest_inode_number_in_the_tree: InodeNumber,
    },
    /// 这次写之后的叶容器数超过一个码 2 根装得下的条目数：inode 树要在根之下再长一层内部节点。
    ///
    /// D8（核心索引结构） 已定项 6 只写了「内部节点同样在右端溢出、末尾分裂，右半是码 2、没有容器身份」，
    /// 没写树长高那一步——新的根从哪来、它的 key 区间与出生序号怎么取、两层内部节点的条目怎么指
    /// （类型段 0 那一支今天盘上一条都没有）。里程碑「第二个事务」并行线三把
    /// 「内部节点条目 120、扇出 135 ⇒ 十万个文件约 430 片叶、树高 2」整句标成**预想**。
    MoreLeafContainersThanOneRootNodeHolds { containers: usize, capacity: usize },
    /// 同一次写里的记录没有按 inode 号严格升序排。
    ///
    /// 升序是调用方的义务，不是这里替它排：一次请求内按 key 升序取号是 D16（发布语义） 已定项 5 末段的纪律
    /// （「同一次请求切出的若干事务按其单元的 key 升序取号」），这里替它排就把那条纪律的违反盖掉了。
    RecordsNotInStrictlyAscendingInodeOrder {
        earlier: InodeNumber,
        later: InodeNumber,
    },
}

/// 把一组 inode 记录写进 inode 树的叶容器，返回这次发布之后的全部容器与这次要重写的那几片。
///
/// 一条记录的落法只有两种：
/// - 它的 inode 号已经在树里 ⇒ **原地换掉那一条**（覆盖写改 size / 改动计数走这一支），记录数不变、容器身份不变，
///   只有那一片进重写清单；
/// - 它的 inode 号大于树里每一个 key ⇒ 追加到最右那片叶；最右那片已经装满 233 条就**在末尾分裂**：
///   左半（原来那片）一条记录都不动、身份不动、**不进重写清单**，右半是一片新容器，容器号 = 这条新记录的 inode 号、
///   出生代 = 这次发布的 checkpoint_txg、出生树 = `inode_tree`。
///
/// 树是空的（第一个事务那一档）时第一条记录建第一片容器，容器号 = 它自己的 inode 号。
///
/// # Errors
/// [`InodeTreeWriteRefusal`] 的三格：中间插入、容器数超过一个根装得下的、给的记录不按 inode 号严格升序。
pub fn write_records_into_leaf_containers(
    containers_before_this_publish: &[InodeLeafContainer],
    records_to_write: &[InodeRecord],
    checkpoint_txg: CheckpointTxg,
    inode_tree: TreeIdentifier,
) -> Result<InodeLeafContainersAfterThisPublish, InodeTreeWriteRefusal> {
    for pair in records_to_write.windows(2) {
        if pair[0].inode >= pair[1].inode {
            return Err(
                InodeTreeWriteRefusal::RecordsNotInStrictlyAscendingInodeOrder {
                    earlier: InodeNumber(pair[0].inode),
                    later: InodeNumber(pair[1].inode),
                },
            );
        }
    }
    let mut containers = containers_before_this_publish.to_vec();
    // 重写清单按叶序升序、同一片只进一次：同一次发布里改同一片叶两次（改一条记录、再往它里面追加一条）只重写一遍。
    let mut rewritten: Vec<InodeLeafContainerIndexInTree> = Vec::new();
    let note_rewritten =
        |index: InodeLeafContainerIndexInTree,
         noted_so_far: &mut Vec<InodeLeafContainerIndexInTree>| {
            if let Err(position) = noted_so_far.binary_search(&index) {
                noted_so_far.insert(position, index);
            }
        };
    for record in records_to_write {
        let inode = InodeNumber(record.inode);
        let existing = position_of_the_container_holding(&containers, inode);
        match existing {
            Some(index) => {
                let container = &mut containers[index.position()];
                let position_in_container = container
                    .records
                    .binary_search_by_key(&inode.0, |existing_record| existing_record.inode)
                    .expect("position_of_the_container_holding 找到的就是这条记录所在的容器");
                container.records[position_in_container] = *record;
                note_rewritten(index, &mut rewritten);
            }
            None => {
                let largest = largest_inode_number_in_the_tree(&containers);
                if let Some(largest_inode_number_in_the_tree) = largest {
                    if inode <= largest_inode_number_in_the_tree {
                        return Err(
                            InodeTreeWriteRefusal::InsertWouldNotLandInTheRightmostLeaf {
                                inode,
                                largest_inode_number_in_the_tree,
                            },
                        );
                    }
                }
                let rightmost = rightmost_container_with_room(&containers);
                match rightmost {
                    Some(index) => {
                        containers[index.position()].records.push(*record);
                        note_rewritten(index, &mut rewritten);
                    }
                    None => {
                        // 末尾分裂：左半（原来那片满的叶）一条记录、一段身份都不动，右半从这条新记录起。
                        containers.push(InodeLeafContainer {
                            identity: PackedIdentity {
                                birth_tree: inode_tree,
                                record_type: PACKED_TYPE_INODE,
                                container: inode.0,
                                container_birth: checkpoint_txg,
                            },
                            records: vec![*record],
                        });
                        note_rewritten(
                            InodeLeafContainerIndexInTree::of_position(containers.len() - 1),
                            &mut rewritten,
                        );
                    }
                }
            }
        }
    }
    let capacity = leaf_containers_one_root_node_holds();
    if containers.len() > capacity {
        return Err(
            InodeTreeWriteRefusal::MoreLeafContainersThanOneRootNodeHolds {
                containers: containers.len(),
                capacity,
            },
        );
    }
    Ok(InodeLeafContainersAfterThisPublish {
        containers,
        rewritten,
    })
}

/// 哪一片容器装着这个 inode 号；一条都没装着是 `None`。
fn position_of_the_container_holding(
    containers: &[InodeLeafContainer],
    inode: InodeNumber,
) -> Option<InodeLeafContainerIndexInTree> {
    containers
        .iter()
        .position(|container| {
            container
                .records
                .binary_search_by_key(&inode.0, |record| record.inode)
                .is_ok()
        })
        .map(InodeLeafContainerIndexInTree::of_position)
}

/// 树里最大的 inode 号；树是空的是 `None`。容器按 key 序、容器内记录按 inode 号升序 ⇒ 最后一片的最后一条。
fn largest_inode_number_in_the_tree(containers: &[InodeLeafContainer]) -> Option<InodeNumber> {
    containers
        .last()
        .and_then(InodeLeafContainer::largest_inode_number)
}

/// 最右那片叶还装得下就给它的序号；树是空的、或最右那片已经满 233 条，都是 `None`（两种情况都要新建一片容器）。
fn rightmost_container_with_room(
    containers: &[InodeLeafContainer],
) -> Option<InodeLeafContainerIndexInTree> {
    let last = containers.last()?;
    if last.free_record_slots() == 0 {
        return None;
    }
    Some(InodeLeafContainerIndexInTree::of_position(
        containers.len() - 1,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use singlefs_format::TREE_IDENTIFIER_INODE;

    const INODE_TREE: TreeIdentifier = TreeIdentifier(TREE_IDENTIFIER_INODE);
    /// 第一个事务把树建起来那次发布的 checkpoint_txg（字节表四：容器 1、出生代 3）。
    const TREE_BIRTH_TXG: CheckpointTxg = CheckpointTxg(3);

    fn record(inode: u64, size: u64, change_count: u64) -> InodeRecord {
        InodeRecord {
            inode,
            object_birth: TREE_BIRTH_TXG,
            size,
            change_count,
            write_time_seconds: 1_788_000_000,
        }
    }

    /// 第一个事务那一档：树是空的，第一条记录建第一片容器，容器号 = 它自己的 inode 号、出生代 = 这次发布的 txg。
    #[test]
    fn the_first_record_creates_the_first_container_numbered_after_itself() {
        let after = write_records_into_leaf_containers(
            &[],
            &[record(1, 3000, 3)],
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect("建第一片容器");
        assert_eq!(after.containers.len(), 1);
        assert_eq!(
            after.containers[0].identity,
            PackedIdentity {
                birth_tree: INODE_TREE,
                record_type: PACKED_TYPE_INODE,
                container: 1,
                container_birth: TREE_BIRTH_TXG,
            },
            "容器号 = 第一条记录的 inode 号（D8 已定项 6）"
        );
        assert_eq!(after.containers[0].records, vec![record(1, 3000, 3)]);
        assert_eq!(
            after.rewritten,
            vec![InodeLeafContainerIndexInTree::LEFTMOST]
        );
    }

    /// 覆盖写那一档：号已经在树里 ⇒ 原地换掉那一条，记录数不变、身份不变，只有那一片进重写清单。
    #[test]
    fn writing_a_number_already_in_the_tree_replaces_that_record_and_keeps_the_container_identity()
    {
        let before = write_records_into_leaf_containers(
            &[],
            &[record(1, 3000, 3)],
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect("建第一片容器")
        .containers;
        let after = write_records_into_leaf_containers(
            &before,
            &[record(1, 4096, 4)],
            CheckpointTxg(4),
            INODE_TREE,
        )
        .expect("覆盖写");
        assert_eq!(after.containers.len(), 1);
        assert_eq!(
            after.containers[0].identity, before[0].identity,
            "COW 重写身份不变（D8 已定项 6）：出生代仍是建树那次的 3，不是这次发布的 4"
        );
        assert_eq!(after.containers[0].records, vec![record(1, 4096, 4)]);
        assert_eq!(
            after.rewritten,
            vec![InodeLeafContainerIndexInTree::LEFTMOST]
        );
    }

    /// 一容器 233 条：第 233 条还在第一片里，树只有一片叶。
    #[test]
    fn two_hundred_thirty_three_records_still_fit_one_container() {
        let records: Vec<InodeRecord> = (1..=233u64).map(|inode| record(inode, 0, 3)).collect();
        let after = write_records_into_leaf_containers(&[], &records, TREE_BIRTH_TXG, INODE_TREE)
            .expect("233 条");
        assert_eq!(INODE_LEAF_RECORDS, 233);
        assert_eq!(after.containers.len(), 1, "233 条装得下一片");
        assert_eq!(after.containers[0].records.len(), 233);
        assert_eq!(after.containers[0].free_record_slots(), 0);
    }

    /// 第 234 条触发末尾分裂：左半保留全部原有记录与身份、**一个字节都不重写**；
    /// 右半是新容器，容器号 = 触发分裂那条新记录的 inode 号、出生代 = 本次发布的 checkpoint_txg、出生树 = 执行分裂的那棵树。
    /// 这是 C116（叶容器的分裂 / 合并纪律没有会失败的检查） 那三条纪律的会红的量。
    #[test]
    fn the_two_hundred_thirty_fourth_record_splits_at_the_end_and_the_left_half_keeps_everything() {
        let before = write_records_into_leaf_containers(
            &[],
            &(1..=233u64)
                .map(|inode| record(inode, 0, 3))
                .collect::<Vec<_>>(),
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect("233 条")
        .containers;
        let split_txg = CheckpointTxg(9);
        let after = write_records_into_leaf_containers(
            &before,
            &[record(234, 0, 9)],
            split_txg,
            INODE_TREE,
        )
        .expect("第 234 条");

        assert_eq!(after.containers.len(), 2, "分裂出第二片容器");
        assert_eq!(
            after.containers[0], before[0],
            "左半保留全部原有记录与身份：记录、容器号、出生代、出生树逐项不变"
        );
        assert_eq!(
            after.containers[0].records.len(),
            233,
            "分裂点取末尾：左半一条记录都不搬走"
        );
        assert_eq!(
            after.containers[1].identity,
            PackedIdentity {
                birth_tree: INODE_TREE,
                record_type: PACKED_TYPE_INODE,
                container: 234,
                container_birth: split_txg,
            },
            "右半容器号 = 触发分裂那条新记录的 inode 号，出生代 = 本次发布的 checkpoint_txg"
        );
        assert_eq!(after.containers[1].records, vec![record(234, 0, 9)]);
        assert_eq!(
            after.rewritten,
            vec![InodeLeafContainerIndexInTree::of_position(1)],
            "只重写右半：左半的字节没变，连 COW 都不做"
        );
    }

    /// 分裂之后两片容器的分隔 key 与 key 区间：I-9.4（容器号不超最小 key） 与 I-9.12（分隔 key 落在孩子区间之外） 在树这一侧的形态。
    #[test]
    fn after_the_split_the_separator_keys_are_the_container_numbers_and_the_ranges_do_not_overlap()
    {
        let after = write_records_into_leaf_containers(
            &[],
            &(1..=234u64)
                .map(|inode| record(inode, 0, 3))
                .collect::<Vec<_>>(),
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect("234 条");
        let separators: Vec<u64> = after
            .containers
            .iter()
            .map(InodeLeafContainer::separator_key)
            .collect();
        assert_eq!(separators, vec![1, 234]);
        assert_eq!(
            after.containers[0].smallest_inode_number(),
            Some(InodeNumber(1))
        );
        assert_eq!(
            after.containers[0].largest_inode_number(),
            Some(InodeNumber(233))
        );
        assert_eq!(
            after.containers[1].smallest_inode_number(),
            Some(InodeNumber(234))
        );
        assert!(
            after.containers[0].largest_inode_number()
                < Some(after.containers[1].container_number()),
            "左容器的最大 key < 右容器号（I-9.4 那一句逼出的合并方向）"
        );
        for (container, separator) in after.containers.iter().zip(&separators) {
            assert!(
                container.smallest_inode_number() >= Some(InodeNumber(*separator)),
                "容器号不超它装的最小 key"
            );
        }
    }

    /// 一次写多条：新号接着最右叶追加，跨过 233 时中间那一片满着不动。
    #[test]
    fn appending_across_the_two_hundred_thirty_three_threshold_fills_left_to_right() {
        let after = write_records_into_leaf_containers(
            &[],
            &(1..=500u64)
                .map(|inode| record(inode, 0, 3))
                .collect::<Vec<_>>(),
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect("500 条");
        assert_eq!(after.containers.len(), 3, "500 = 233 + 233 + 34");
        assert_eq!(
            after
                .containers
                .iter()
                .map(|container| container.records.len())
                .collect::<Vec<_>>(),
            vec![233, 233, 34]
        );
        assert_eq!(
            after
                .containers
                .iter()
                .map(|container| container.identity.container)
                .collect::<Vec<_>>(),
            vec![1, 234, 467],
            "每片容器号 = 它诞生时装的第一条记录的 inode 号"
        );
        assert_eq!(
            after.rewritten,
            vec![
                InodeLeafContainerIndexInTree::of_position(0),
                InodeLeafContainerIndexInTree::of_position(1),
                InodeLeafContainerIndexInTree::of_position(2),
            ],
            "这一次三片都动过：第一片从空装到满，后两片是新建的"
        );
    }

    /// 中间插入（号不在树里、又不比树里每一个 key 大）在任何人动盘之前被拒：非末尾分裂的纪律没写。
    #[test]
    fn inserting_a_number_that_would_not_land_in_the_rightmost_leaf_is_refused() {
        let before = write_records_into_leaf_containers(
            &[],
            &[record(1, 0, 3), record(5, 0, 3)],
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect("两条")
        .containers;
        let refusal = write_records_into_leaf_containers(
            &before,
            &[record(3, 0, 4)],
            CheckpointTxg(4),
            INODE_TREE,
        )
        .expect_err("中间插入今天一律被拒");
        assert_eq!(
            refusal,
            InodeTreeWriteRefusal::InsertWouldNotLandInTheRightmostLeaf {
                inode: InodeNumber(3),
                largest_inode_number_in_the_tree: InodeNumber(5),
            }
        );
    }

    /// 容器数超过一个码 2 根装得下的 135 片：树要长高，怎么长没条款 ⇒ 拒。
    #[test]
    fn more_containers_than_one_root_node_holds_is_refused() {
        assert_eq!(leaf_containers_one_root_node_holds(), 135, "扇出 135");
        let records: Vec<InodeRecord> = (1..=(233 * 135 + 1))
            .map(|inode| record(inode, 0, 3))
            .collect();
        let refusal = write_records_into_leaf_containers(&[], &records, TREE_BIRTH_TXG, INODE_TREE)
            .expect_err("第 136 片容器今天装不下");
        assert_eq!(
            refusal,
            InodeTreeWriteRefusal::MoreLeafContainersThanOneRootNodeHolds {
                containers: 136,
                capacity: 135,
            }
        );
    }

    /// 同一次写里的记录不按 inode 号严格升序 ⇒ 拒（升序取号是 D16 已定项 5 末段的纪律，这里不替调用方排）。
    #[test]
    fn records_not_in_ascending_inode_order_are_refused() {
        let refusal = write_records_into_leaf_containers(
            &[],
            &[record(3, 0, 3), record(2, 0, 3)],
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect_err("降序被拒");
        assert_eq!(
            refusal,
            InodeTreeWriteRefusal::RecordsNotInStrictlyAscendingInodeOrder {
                earlier: InodeNumber(3),
                later: InodeNumber(2),
            }
        );
        assert_eq!(
            write_records_into_leaf_containers(
                &[],
                &[record(3, 0, 3), record(3, 1, 3)],
                TREE_BIRTH_TXG,
                INODE_TREE,
            )
            .expect_err("同号两条也被拒"),
            InodeTreeWriteRefusal::RecordsNotInStrictlyAscendingInodeOrder {
                earlier: InodeNumber(3),
                later: InodeNumber(3),
            }
        );
    }

    /// 一次写里既改旧记录又追加新记录：改的那一片与新建的那一片都进重写清单，各进一次。
    #[test]
    fn one_write_that_both_replaces_and_appends_lists_each_touched_container_once() {
        let before = write_records_into_leaf_containers(
            &[],
            &(1..=233u64)
                .map(|inode| record(inode, 0, 3))
                .collect::<Vec<_>>(),
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect("233 条")
        .containers;
        let after = write_records_into_leaf_containers(
            &before,
            &[record(1, 4096, 9), record(234, 0, 9), record(235, 0, 9)],
            CheckpointTxg(9),
            INODE_TREE,
        )
        .expect("改一条、追加两条");
        assert_eq!(
            after.rewritten,
            vec![
                InodeLeafContainerIndexInTree::of_position(0),
                InodeLeafContainerIndexInTree::of_position(1),
            ]
        );
        assert_eq!(after.containers[1].records.len(), 2, "新容器装两条");
        assert_eq!(
            after.containers[1].identity.container, 234,
            "容器号是触发分裂那条（234），不是这次写的最后一条（235）"
        );
        assert_eq!(after.containers[0].records[0], record(1, 4096, 9));
    }
}
