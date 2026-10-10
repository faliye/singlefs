//! inode 树的叶容器：一容器 233 条 140 字节记录，满了就在末尾分裂（里程碑「覆盖写、释放、回退与复用」并行线三的写路径）。
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
//! **删除的落法**（[`remove_record_from_leaf_containers`]，unlink 最后一批那次发布）：删掉那一条；之后照 E176（inode 树叶合并三候选的容量与写代价）
//! M3 那一臂的写路径策略合并（D8（核心索引结构） 已定项 6「左吸收右与左从右借两样都做」，合并时机是写路径策略）：
//! - 那一片删空了就整片不落盘（「记录数为 0 的类型 2 容器不落盘」），它的左右邻都在时从左邻起按「先查右邻、再查左邻，两片之和 ≤ 233 就左吸收右」
//!   一直并到没有一对装得下；
//! - 没删空就从它起照同一条规则并；一次都没并、它的记录数 < 117（⌈233 ÷ 2⌉）、它有右邻时，从右邻借最小的 ⌊(右 − 左) ÷ 2⌋ 条
//!   （均分：借完两片条数相差不超过 1），右片保留身份、分隔 key 改成它这一版的最小 key（N1），再从右片起照同一条规则并。
//!
//! 合并保留左半身份；被吸收的右半与删空的那一片身份消失，交回给发布路径写退役记录（[`InodeLeafContainersAfterThisPublish::retired`]）。
//! **只重写内容变了的叶**：身份消失的那几片右边的每一片位置左移，但内容、身份、分隔 key 都没变，这次照抄上一版那一份（落点、字节、指针都不动），
//! 只是在这一版里换到左边的叶序上（[`InodeLeafContainerOrigin::CarriedFrom`]）——删空最左一片那次新写的槽数不随右边片数涨
//! （判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 7 条）。

use singlefs_format::{INODE_INTERNAL_ENTRY, INODE_LEAF_RECORDS};

use crate::address::{CheckpointTxg, InodeNumber, TreeIdentifier};
use crate::records::InodeRecord;
use crate::unit::{index_node_entry_capacity, PackedIdentity, PACKED_TYPE_INODE};

/// inode 树里第几片叶容器，按叶序（= key 升序）从 0 数。
///
/// 它是**树里的位置**，不是容器号：容器号是首次插入它的那条记录的 inode 号（D8（核心索引结构） 已定项 6），
/// 位置随左边有没有别的容器而变，两者不是一个量。新容器只在最右端出现；一片删空或被吸收时它右边的每一片左移，
/// 那一次发布照抄挪了位置的几片、把它们换到这一版的叶序上（[`InodeLeafContainerOrigin::CarriedFrom`] 记着上一版的叶序），
/// 所以位置照样当这一版里的角色序号用，照抄时按上一版的叶序取上一版那一份。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InodeLeafContainerIndexInTree(pub u32);

impl InodeLeafContainerIndexInTree {
    /// 最左那一片。树只有一片叶时就是它（新池新建文件那一档）。
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

/// 一片 inode 叶容器这一版装的东西：身份四段（出生树、打包记录类型、容器号、容器出生代）、它在父节点条目里的分隔 key、
/// 它的记录，记录按 inode 号升序。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InodeLeafContainer {
    pub identity: PackedIdentity,
    /// 父节点里这片容器那一条条目的分隔 key（D8（核心索引结构） 已定项 6「分隔 key = 该孩子创建时的最小 key，左从右借的那次发布改成
    /// 被借的右孩子这一版的最小 key」）：诞生时是容器号，只有被左邻借走最小那几条的那次发布改成它这一版的最小 key，之后照抄。
    pub separator_key_in_the_parent_entry: u64,
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

    /// 这片容器在父节点里的分隔 key：D8（核心索引结构） 已定项 6 逐字「**分隔 key** = 该孩子创建时的最小 key，左从右借的那次发布改成
    /// 被借的右孩子这一版的最小 key」。创建时的最小 key 就是容器号——第一个容器的号 = 第一条记录的 inode 号，分裂出来的右半容器号 =
    /// 触发分裂那条新记录的 inode 号。**别的发布不取今天的最小 key**：容器被 COW 重写、记录增删之后今天的最小 key 可以比上一次定下的大，
    /// 而分隔 key 不许跟着变（跟着变就等于改父节点里的路由）；只有被借的那一次随借走的那几条改（[`remove_record_from_leaf_containers`]）。
    #[must_use]
    pub fn separator_key(&self) -> u64 {
        self.separator_key_in_the_parent_entry
    }

    /// 这片容器还能再装几条：一容器 233 条（D8（核心索引结构） 已定项 6，`INODE_LEAF_RECORDS`）。
    ///
    /// # Panics
    /// 这片容器已经装了多于 233 条。容器里的记录只有两个来处，两处都装不到第 234 条：
    /// 写者往里追加只经 [`write_records_into_leaf_containers`]，它只在这个函数报「还装得下」时才追加；
    /// 盘上读来的（可写挂载重建上一版，`recovery::rebuild_version`）由读者判过记录宽 140
    /// （`recovery::inode_leaf_container_judged_against_its_entry`）、记录区不越过单元末尾（`unit::parse_packed_unit`），
    /// 一片至多 ⌊(32768 − 107 − 29) ÷ 140⌋ = 233 条（C476 普查「缺口」表 `inode_tree.rs:88` 那一行；这条算术由单测
    /// `a_packed_unit_holds_no_more_inode_records_than_the_leaf_record_limit` 盯着）。装到了说明这两道之一被绕过了，
    /// 不许当成「还能装 0 条」往下走。
    #[must_use]
    pub fn free_record_slots(&self) -> u64 {
        INODE_LEAF_RECORDS
            .checked_sub(u64::try_from(self.records.len()).expect("一容器 233 条"))
            .expect(
                "一容器最多 233 条记录（D8 已定项 6）：写者只经 write_records_into_leaf_containers 追加（分裂纪律），盘上读来的由 \
                 recovery::inode_leaf_container_judged_against_its_entry 判过记录宽 140、unit::parse_packed_unit 判过记录区不越过单元末尾",
            )
    }
}

/// 一个码 2 根装得下几条 inode 内部条目：扇出 135（D8（核心索引结构） 已定项 6：分隔 key 8 + 身份引用 26 + 子指针 86 = 120）。
#[must_use]
pub fn leaf_containers_one_root_node_holds() -> usize {
    index_node_entry_capacity(8, usize::try_from(INODE_INTERNAL_ENTRY).expect("120"))
}

/// 这一版的一片叶容器从哪来。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InodeLeafContainerOrigin {
    /// 这次重写（内容、或分隔 key 变了，或是新分裂出来的）。
    RewrittenThisPublish,
    /// 上一版那一片一个字节不动，带的是它在上一版里的叶序（左边有片身份消失时比这一版的大）。
    CarriedFrom(InodeLeafContainerIndexInTree),
}

/// 这次身份消失的一片叶容器（删空摘掉，或被左邻吸收）：它在上一版里的叶序与它的身份。发布路径给它的身份写一条退役记录、
/// 按上一版的叶序换下它那一份（D8（核心索引结构） 已定项 6「死亡分三层」第 ② 层）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RetiredInodeLeafContainer {
    pub previous_index: InodeLeafContainerIndexInTree,
    pub identity: PackedIdentity,
}

/// 这次发布之后 inode 树的叶容器，以及这次要重写哪几片、每片从哪来、哪几片身份消失。
///
/// 没被点到的那几片一个字节都不变（分裂时的左半就是这样：它的记录与身份都没动 ⇒ 不进这次的重写清单，
/// 连 COW 都不做），调用方按 `origins` 里上一版的叶序照抄上一版的落点与指针。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InodeLeafContainersAfterThisPublish {
    /// 左起按 key 序的全部叶容器。
    pub containers: Vec<InodeLeafContainer>,
    /// 这次要重写的那几片，按叶序升序（就是 `origins` 里 `RewrittenThisPublish` 的那几个叶序）。
    pub rewritten: Vec<InodeLeafContainerIndexInTree>,
    /// 每片从哪来，与 `containers` 一一对应。
    pub origins: Vec<InodeLeafContainerOrigin>,
    /// 这次身份消失的那几片，按它们在上一版里的叶序升序（只有 [`remove_record_from_leaf_containers`] 会有）：
    /// 删空的是最右那一片时重写清单是空的，根照样要重写（少了一条条目）。
    pub retired: Vec<RetiredInodeLeafContainer>,
}

impl InodeLeafContainersAfterThisPublish {
    /// 这次要不要重写 inode 树根：有叶容器重写（COW 叶 + 全部祖先，D23（journal 的角色与格式） 已定项 1 甲），或有一片身份消失、不再落盘。
    #[must_use]
    pub fn is_the_root_rewritten(&self) -> bool {
        !self.rewritten.is_empty() || !self.retired.is_empty()
    }

    /// 这次身份消失的那几片的身份，按上一版的叶序：这次发布的退役集合（上一版的叶身份集 − 这一版的叶身份集）。
    #[must_use]
    pub fn retired_identities(&self) -> Vec<PackedIdentity> {
        self.retired
            .iter()
            .map(|retired| retired.identity)
            .collect()
    }

    /// 这一版第 `index` 片照抄的话，它在上一版里的叶序；这次重写的交回 `None`。
    ///
    /// # Panics
    /// `index` 不在这一版里：调用方只拿这一版的叶序问。
    #[must_use]
    pub fn previous_index_of_a_carried_container(
        &self,
        index: InodeLeafContainerIndexInTree,
    ) -> Option<InodeLeafContainerIndexInTree> {
        match self.origins[index.position()] {
            InodeLeafContainerOrigin::RewrittenThisPublish => None,
            InodeLeafContainerOrigin::CarriedFrom(previous_index) => Some(previous_index),
        }
    }
}

/// 左从右借的下限：一片叶删完一条之后记录数少于它、这次又没并过、它有右邻，就从右邻借（E176 M3 那一臂的主取样点
/// f = ⌈233 ÷ 2⌉ = 117，`research/e7-index-bench/src/bin/e176_inode_leaf_merge_capacity_and_rewrites.rs` 的 `BORROW_FLOOR_MAIN`）。
#[must_use]
pub fn records_below_which_a_leaf_borrows_from_its_right_neighbour() -> usize {
    usize::try_from(INODE_LEAF_RECORDS.div_ceil(2)).expect("117")
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
    /// （类型段 0 那一支今天盘上一条都没有）。里程碑「覆盖写、释放、回退与复用」并行线三把
    /// 「内部节点条目 120、扇出 135 ⇒ 十万个文件约 430 片叶、树高 2」整句标成**预想**。
    MoreLeafContainersThanOneRootNodeHolds { containers: usize, capacity: usize },
    /// unlink 最后一批要删的那条记录不在树里（[`remove_record_from_leaf_containers`]）：调用方按现行那一版的队列取号，
    /// 号在队列里而记录不在树里说明那一版的 `deleted_inodes` 与 inode 树对不上。
    RecordToRemoveIsNotInTheTree { inode: InodeNumber },
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
/// 树是空的（新池新建文件那一档）时第一条记录建第一片容器，容器号 = 它自己的 inode 号。
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
                            separator_key_in_the_parent_entry: inode.0,
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
    let origins = (0..containers.len())
        .map(|position| {
            let index = InodeLeafContainerIndexInTree::of_position(position);
            if rewritten.binary_search(&index).is_ok() {
                InodeLeafContainerOrigin::RewrittenThisPublish
            } else {
                InodeLeafContainerOrigin::CarriedFrom(index)
            }
        })
        .collect();
    Ok(InodeLeafContainersAfterThisPublish {
        containers,
        rewritten,
        origins,
        retired: Vec::new(),
    })
}

/// 从 inode 树的叶容器里删掉 `inode` 那一条记录（unlink 最后一批那次发布：D8（核心索引结构） 已定项 16 第 2 条「删 inode 记录」），
/// 再照 E176 M3 那一臂合并（文件头那几条）。
///
/// - 删完没并、没借、没删空 ⇒ 只有那一片进重写清单，身份不变（COW 重写身份不变，D8 已定项 6）；
/// - 那一片删空了 ⇒ 整片不落盘（「记录数为 0 的类型 2 容器不落盘」），身份进 `retired`；左吸收右的那一对，左片进重写清单、右片身份进 `retired`；
///   左从右借的那一对两片都进重写清单、身份都不变，右片的分隔 key 改成它这一版的最小 key；
/// - 身份消失的那几片右边的每一片位置左移，内容没变的照抄（`origins` 记着上一版的叶序），不进重写清单。
///
/// 删的是树里最后一条时交回零片容器：inode 树写零条目根（key 区间 `[0, 0]`，D8 已定项 6），那一版是「有树、文件集合为空」
/// （D8 已定项 16 第 5 条），怎么装它归发布路径。
///
/// # Errors
/// 树里没有这条记录 ⇒ [`InodeTreeWriteRefusal::RecordToRemoveIsNotInTheTree`]（在任何落盘动作之前）。
pub fn remove_record_from_leaf_containers(
    containers_before_this_publish: &[InodeLeafContainer],
    inode: InodeNumber,
) -> Result<InodeLeafContainersAfterThisPublish, InodeTreeWriteRefusal> {
    let Some(index) = position_of_the_container_holding(containers_before_this_publish, inode)
    else {
        return Err(InodeTreeWriteRefusal::RecordToRemoveIsNotInTheTree { inode });
    };
    let mut tree =
        LeafContainersBeingMerged::from_the_previous_version(containers_before_this_publish);
    let holding_position = index.position();
    let holding = &mut tree.containers[holding_position];
    let position_in_container = holding
        .contents
        .records
        .binary_search_by_key(&inode.0, |existing_record| existing_record.inode)
        .expect("position_of_the_container_holding 找到的就是这条记录所在的容器");
    holding.contents.records.remove(position_in_container);
    holding.is_changed_this_publish = true;
    if holding.contents.records.is_empty() {
        tree.retire(holding_position);
        let has_new_neighbours = holding_position > 0 && holding_position < tree.containers.len();
        if has_new_neighbours {
            tree.absorb_until_no_pair_fits(holding_position - 1);
        }
    } else {
        let absorbed_at_least_once = tree.absorb_until_no_pair_fits(holding_position);
        if !absorbed_at_least_once {
            tree.borrow_from_the_right_neighbour_if_below_the_floor(holding_position);
        }
    }
    Ok(tree.after_this_publish())
}

/// 一次删除发布里合并到一半的叶容器：每片带着它在上一版里的叶序与这次有没有改过；身份消失的那几片挪进 `retired`。
struct LeafContainersBeingMerged {
    containers: Vec<LeafContainerBeingMerged>,
    retired: Vec<RetiredInodeLeafContainer>,
}

struct LeafContainerBeingMerged {
    contents: InodeLeafContainer,
    previous_index: InodeLeafContainerIndexInTree,
    is_changed_this_publish: bool,
}

impl LeafContainersBeingMerged {
    fn from_the_previous_version(containers: &[InodeLeafContainer]) -> Self {
        Self {
            containers: containers
                .iter()
                .enumerate()
                .map(|(position, contents)| LeafContainerBeingMerged {
                    contents: contents.clone(),
                    previous_index: InodeLeafContainerIndexInTree::of_position(position),
                    is_changed_this_publish: false,
                })
                .collect(),
            retired: Vec::new(),
        }
    }

    fn records_at(&self, position: usize) -> usize {
        self.containers[position].contents.records.len()
    }

    /// 这一对（`left_position` 与它右邻）合起来装得下一片（≤ 233 条）。
    fn pair_fits_in_one_container(&self, left_position: usize) -> bool {
        u64::try_from(self.records_at(left_position) + self.records_at(left_position + 1))
            .expect("两片之和至多 466 条")
            <= INODE_LEAF_RECORDS
    }

    /// 那一片的身份消失（删空摘掉或被吸收）：从这一版拿掉，身份连同上一版的叶序进 `retired`。
    fn retire(&mut self, position: usize) -> InodeLeafContainer {
        let retired = self.containers.remove(position);
        self.retired.push(RetiredInodeLeafContainer {
            previous_index: retired.previous_index,
            identity: retired.contents.identity,
        });
        retired.contents
    }

    /// 左片吸收它右邻：左片保留身份与分隔 key，右片的记录接在后面，右片身份消失。
    fn absorb_the_right_neighbour(&mut self, left_position: usize) {
        let absorbed = self.retire(left_position + 1);
        let survivor = &mut self.containers[left_position];
        survivor.contents.records.extend(absorbed.records);
        survivor.is_changed_this_publish = true;
    }

    /// 从 `start_position` 那片起先查它与右邻、再查左邻与它，两片之和 ≤ 233 就左吸收右；并完从幸存那片接着查，直到没有一对装得下
    /// （E176 M3 主取样点「先查右边」）。交回这一趟有没有并过。
    ///
    /// 迭代上界是片数：每一轮要么并掉一片、要么退出；跨轮携带的是「当前那片」的位置，它只往左挪（左邻吸收它时换成左邻），
    /// 被摘掉的总在它右边或就是它被左邻吸收，所以位置一直指着同一片幸存的叶。
    fn absorb_until_no_pair_fits(&mut self, start_position: usize) -> bool {
        let mut current_position = start_position;
        let mut absorbed_at_least_once = false;
        loop {
            let right_pair_fits = current_position + 1 < self.containers.len()
                && self.pair_fits_in_one_container(current_position);
            let left_pair_fits =
                current_position > 0 && self.pair_fits_in_one_container(current_position - 1);
            if right_pair_fits {
                self.absorb_the_right_neighbour(current_position);
            } else if left_pair_fits {
                self.absorb_the_right_neighbour(current_position - 1);
                current_position -= 1;
            } else {
                return absorbed_at_least_once;
            }
            absorbed_at_least_once = true;
        }
    }

    /// 左从右借（D8（核心索引结构） 已定项 6；E176 M3）：`borrower_position` 那片记录数小于下限且有右邻时，把右邻最小的
    /// ⌊(右 − 左) ÷ 2⌋ 条挪到它末尾（均分：借完两片条数相差不超过 1），两片都保留身份，右邻的分隔 key 改成它这一版的最小 key（N1）；
    /// 借完从右邻起照 [`Self::absorb_until_no_pair_fits`] 并。它是最右那片时不借（右从左借会破「记录号 ≥ 容器号」）。
    ///
    /// # Panics
    /// 要借的条数不在 [1, 右邻记录数) 里：调用方只在「从这片起一次都没并过」之后调它，那时它与右邻之和 > 233，
    /// 它又少于 117 条 ⇒ 右邻多于 116 条、借的条数 ⌊(右 − 左) ÷ 2⌋ ≥ 117 − 左 ≥ 1，且小于右邻的条数。
    fn borrow_from_the_right_neighbour_if_below_the_floor(&mut self, borrower_position: usize) {
        let lender_position = borrower_position + 1;
        if lender_position >= self.containers.len() {
            return;
        }
        let records_in_the_borrower = self.records_at(borrower_position);
        if records_in_the_borrower >= records_below_which_a_leaf_borrows_from_its_right_neighbour()
        {
            return;
        }
        let records_in_the_lender = self.records_at(lender_position);
        let moved_count = records_in_the_lender
            .checked_sub(records_in_the_borrower)
            .map_or(0, |difference| difference / 2);
        assert!(
            moved_count >= 1 && moved_count < records_in_the_lender,
            "借的条数要在 [1, 右邻记录数) 里：借方 {records_in_the_borrower} 条、右邻 {records_in_the_lender} 条、借 {moved_count} 条\
             （这一对没并成说明两片之和 > 233，借方又少于 117 条）"
        );
        let moved: Vec<InodeRecord> = self.containers[lender_position]
            .contents
            .records
            .drain(..moved_count)
            .collect();
        let lender = &mut self.containers[lender_position];
        lender.contents.separator_key_in_the_parent_entry = lender
            .contents
            .records
            .first()
            .map(|record| record.inode)
            .expect("右邻借出的条数少于它的记录数：借完至少还剩一条");
        lender.is_changed_this_publish = true;
        let borrower = &mut self.containers[borrower_position];
        borrower.contents.records.extend(moved);
        borrower.is_changed_this_publish = true;
        self.absorb_until_no_pair_fits(lender_position);
    }

    fn after_this_publish(mut self) -> InodeLeafContainersAfterThisPublish {
        self.retired.sort_by_key(|retired| retired.previous_index);
        let mut rewritten = Vec::new();
        let mut origins = Vec::with_capacity(self.containers.len());
        let mut containers = Vec::with_capacity(self.containers.len());
        for (position, container) in self.containers.into_iter().enumerate() {
            if container.is_changed_this_publish {
                rewritten.push(InodeLeafContainerIndexInTree::of_position(position));
                origins.push(InodeLeafContainerOrigin::RewrittenThisPublish);
            } else {
                origins.push(InodeLeafContainerOrigin::CarriedFrom(
                    container.previous_index,
                ));
            }
            containers.push(container.contents);
        }
        InodeLeafContainersAfterThisPublish {
            containers,
            rewritten,
            origins,
            retired: self.retired,
        }
    }
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
    /// 新池新建文件把树建起来那次发布的 checkpoint_txg（字节表四：容器 1、出生代 3）。
    const TREE_BIRTH_TXG: CheckpointTxg = CheckpointTxg(3);

    fn record(inode: u64, size: u64, change_count: u64) -> InodeRecord {
        InodeRecord {
            inode,
            object_birth: TREE_BIRTH_TXG,
            nlink: crate::records::NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
            size,
            change_count,
            write_time_seconds: 1_788_000_000,
        }
    }

    /// 新池新建文件那一档：树是空的，第一条记录建第一片容器，容器号 = 它自己的 inode 号、出生代 = 这次发布的 txg。
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

    /// 一个码 3 单元按 inode 记录宽装得下的条数不多于一容器的上限：`free_record_slots` 那道断言对盘上读来的容器从盘上走不到，
    /// 靠的就是这条算术（读者判记录宽 = `INODE_RECORD_BYTES`、记录区不越过单元末尾）。改了记录宽、单元头宽或预留位而不重算
    /// `INODE_LEAF_RECORDS`，盘上一片就可能装得下第 234 条（C476 普查「缺口」表 `inode_tree.rs:88` 那一行）。
    #[test]
    fn a_packed_unit_holds_no_more_inode_records_than_the_leaf_record_limit() {
        use singlefs_format::{
            DATA_UNIT_BYTES, INODE_RECORD_BYTES, NONCE_MAC_ALGORITHM_RESERVED_BYTES,
            PACKED_UNIT_HEADER_BYTES,
        };
        let records_a_packed_unit_holds =
            (DATA_UNIT_BYTES - PACKED_UNIT_HEADER_BYTES - NONCE_MAC_ALGORITHM_RESERVED_BYTES)
                / INODE_RECORD_BYTES;
        assert!(
            records_a_packed_unit_holds <= INODE_LEAF_RECORDS,
            "一个码 3 单元装得下 {records_a_packed_unit_holds} 条 inode 记录，多于一容器的上限 {INODE_LEAF_RECORDS}"
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

    /// 三片叶容器：第一片 1..=233、第二片 234..=466、第三片只有 467（分裂纪律写出来的形状）。
    fn three_containers() -> Vec<InodeLeafContainer> {
        let after = write_records_into_leaf_containers(
            &[],
            &(1..=467u64)
                .map(|inode| record(inode, 0, 3))
                .collect::<Vec<_>>(),
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect("467 条");
        assert_eq!(after.containers.len(), 3, "233 + 233 + 1");
        after.containers
    }

    /// `full_containers` 片装满 233 条的叶容器（inode 1 起连号；容器号 = 每片第一条的号、分隔 key 同容器号）。
    fn full_containers(full_containers: u64) -> Vec<InodeLeafContainer> {
        let after = write_records_into_leaf_containers(
            &[],
            &(1..=233 * full_containers)
                .map(|inode| record(inode, 0, 3))
                .collect::<Vec<_>>(),
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect("满的几片");
        assert_eq!(
            after.containers.len(),
            usize::try_from(full_containers).expect("片数")
        );
        after.containers
    }

    /// 把第 `position` 片的记录留成按号升序的前 `kept` 条（直接在内容上改：这里判的是删一条之后的落法，不是怎么删到这个样子）。
    fn keep_the_first_records(containers: &mut [InodeLeafContainer], position: usize, kept: usize) {
        containers[position].records.truncate(kept);
    }

    fn container_numbers(containers: &[InodeLeafContainer]) -> Vec<u64> {
        containers
            .iter()
            .map(|container| container.identity.container)
            .collect()
    }

    fn record_counts(containers: &[InodeLeafContainer]) -> Vec<usize> {
        containers
            .iter()
            .map(|container| container.records.len())
            .collect()
    }

    /// 删一条、那一片还有记录、与左右邻都并不了、不少于 117 条：只重写那一片，身份不变，别的片照抄，没有身份消失。
    #[test]
    fn removing_a_record_that_triggers_no_merge_rewrites_only_that_container() {
        let before = full_containers(3);
        let after = remove_record_from_leaf_containers(&before, InodeNumber(300)).expect("删 300");
        assert_eq!(record_counts(&after.containers), vec![233, 232, 233]);
        assert_eq!(
            after.containers[1].identity, before[1].identity,
            "COW 重写身份不变"
        );
        assert!(
            after.containers[1]
                .records
                .iter()
                .all(|kept| kept.inode != 300),
            "那一条不在了"
        );
        assert_eq!(after.containers[0], before[0]);
        assert_eq!(after.containers[2], before[2]);
        assert_eq!(
            after.rewritten,
            vec![InodeLeafContainerIndexInTree::of_position(1)]
        );
        assert_eq!(
            after.origins,
            vec![
                InodeLeafContainerOrigin::CarriedFrom(InodeLeafContainerIndexInTree::of_position(
                    0
                )),
                InodeLeafContainerOrigin::RewrittenThisPublish,
                InodeLeafContainerOrigin::CarriedFrom(InodeLeafContainerIndexInTree::of_position(
                    2
                )),
            ]
        );
        assert!(after.retired.is_empty());
        assert!(after.is_the_root_rewritten());
    }

    /// 删空最右那一片：整片不落盘、身份消失，重写清单是空的，根照样重写（少了一条条目）。它左边没有右邻可并。
    #[test]
    fn removing_the_only_record_of_the_rightmost_container_drops_it_and_still_rewrites_the_root() {
        let before = three_containers();
        let after = remove_record_from_leaf_containers(&before, InodeNumber(467)).expect("删 467");
        assert_eq!(after.containers, before[..2].to_vec(), "前两片原样");
        assert!(after.rewritten.is_empty(), "没有一片要重写");
        assert_eq!(
            after.retired,
            vec![RetiredInodeLeafContainer {
                previous_index: InodeLeafContainerIndexInTree::of_position(2),
                identity: before[2].identity,
            }]
        );
        assert!(after.is_the_root_rewritten(), "根少一条条目，要重写");
    }

    /// 删空最左一片（M1 写出来的树才有这种形状：左边一片只剩一条）：整片不落盘、身份消失；它没有左邻，不并；
    /// 右边每一片左移一格、内容没变，照抄、不进重写清单（`origins` 记着上一版的叶序）——20 片与 40 片两种几何重写清单都是空的，
    /// 不随右边片数涨（判决 m3-retire-r1 第三节第 7 条）。
    #[test]
    fn dropping_the_emptied_leftmost_container_carries_every_container_to_its_right_whatever_their_number(
    ) {
        for containers_in_the_tree in [20_u64, 40] {
            let mut before = full_containers(containers_in_the_tree);
            keep_the_first_records(&mut before, 0, 1);
            let after = remove_record_from_leaf_containers(&before, InodeNumber(1)).expect("删 1");
            assert_eq!(
                after.containers,
                before[1..].to_vec(),
                "{containers_in_the_tree} 片：右边各片内容、身份、分隔 key 原样"
            );
            assert!(
                after.rewritten.is_empty(),
                "{containers_in_the_tree} 片：挪了位置的一片都不重写"
            );
            assert_eq!(
                after.origins,
                (1..before.len())
                    .map(|previous_position| InodeLeafContainerOrigin::CarriedFrom(
                        InodeLeafContainerIndexInTree::of_position(previous_position)
                    ))
                    .collect::<Vec<_>>(),
                "{containers_in_the_tree} 片：这一版第 i 片照抄上一版第 i + 1 片"
            );
            assert_eq!(after.retired_identities(), vec![before[0].identity]);
        }
    }

    /// 删空中间一片（M1 写出来的形状）：身份消失；左邻与新右邻之和 > 233 不并；新右邻左移一格照抄，左边那片不动。
    #[test]
    fn dropping_an_emptied_middle_container_carries_the_ones_to_its_right_without_rewriting_them() {
        let mut before = three_containers();
        keep_the_first_records(&mut before, 1, 1);
        let after = remove_record_from_leaf_containers(&before, InodeNumber(234)).expect("删 234");
        assert_eq!(after.containers, vec![before[0].clone(), before[2].clone()]);
        assert!(
            after.rewritten.is_empty(),
            "左邻 233 + 新右邻 1 = 234 > 233，不并"
        );
        assert_eq!(
            after.origins[1],
            InodeLeafContainerOrigin::CarriedFrom(InodeLeafContainerIndexInTree::of_position(2))
        );
        assert_eq!(after.retired_identities(), vec![before[1].identity]);
    }

    /// 左吸收右（M3）：第一片删完一条剩 99 条、第二片 100 条，之和 199 ≤ 233 ⇒ 第一片吸收第二片：保留左半身份与分隔 key，记录接在后面；
    /// 第二片身份消失；第三片左移一格照抄。前后叶序：容器号 [1, 234, 467] → [1, 467]。
    #[test]
    fn the_left_container_absorbs_its_right_neighbour_when_the_two_fit_in_one_and_keeps_its_identity(
    ) {
        let mut before = full_containers(3);
        keep_the_first_records(&mut before, 0, 100);
        keep_the_first_records(&mut before, 1, 100);
        let after = remove_record_from_leaf_containers(&before, InodeNumber(5)).expect("删 5");
        assert_eq!(container_numbers(&after.containers), vec![1, 467]);
        assert_eq!(record_counts(&after.containers), vec![199, 233]);
        assert_eq!(
            after.containers[0].identity, before[0].identity,
            "保留左半身份"
        );
        assert_eq!(after.containers[0].separator_key(), 1, "左半分隔 key 不变");
        assert_eq!(
            after.containers[0]
                .records
                .last()
                .map(|record| record.inode),
            Some(333),
            "右半的记录接在后面"
        );
        assert_eq!(after.containers[1], before[2], "第三片原样");
        assert_eq!(
            after.rewritten,
            vec![InodeLeafContainerIndexInTree::LEFTMOST]
        );
        assert_eq!(
            after.origins[1],
            InodeLeafContainerOrigin::CarriedFrom(InodeLeafContainerIndexInTree::of_position(2))
        );
        assert_eq!(
            after.retired,
            vec![RetiredInodeLeafContainer {
                previous_index: InodeLeafContainerIndexInTree::of_position(1),
                identity: before[1].identity,
            }],
            "被吸收的右半身份消失"
        );
    }

    /// 左从右借（M3 均分借）：第一片删完一条剩 99 条、第二片 233 条，之和 > 233 不并；99 < 117 ⇒ 从第二片借最小的 ⌊(233 − 99) ÷ 2⌋ = 67 条，
    /// 借完两片 166 / 166（相差不超过 1）；两片都保留身份，第二片的分隔 key 改成它这一版的最小 key 301（N1）；没有身份消失，第三片照抄。
    #[test]
    fn a_container_below_the_floor_borrows_half_the_difference_from_its_right_neighbour() {
        let mut before = full_containers(3);
        keep_the_first_records(&mut before, 0, 100);
        let after = remove_record_from_leaf_containers(&before, InodeNumber(5)).expect("删 5");
        assert_eq!(container_numbers(&after.containers), vec![1, 234, 467]);
        assert_eq!(record_counts(&after.containers), vec![166, 166, 233]);
        assert_eq!(after.containers[0].identity, before[0].identity);
        assert_eq!(
            after.containers[1].identity, before[1].identity,
            "右片保留身份"
        );
        assert_eq!(
            after
                .containers
                .iter()
                .map(InodeLeafContainer::separator_key)
                .collect::<Vec<_>>(),
            vec![1, 301, 467],
            "右片的分隔 key = 它这一版的最小 key（N1），别的不变"
        );
        assert_eq!(
            after.containers[0]
                .records
                .last()
                .map(|record| record.inode),
            Some(300),
            "借来的是右片最小的 67 条：234..=300"
        );
        assert_eq!(after.containers[2], before[2]);
        assert_eq!(
            after.rewritten,
            vec![
                InodeLeafContainerIndexInTree::of_position(0),
                InodeLeafContainerIndexInTree::of_position(1)
            ]
        );
        assert!(after.retired.is_empty(), "借不退役");
    }

    /// 借了之后再删到 116、右片又借、……：同一对一路删 233 条，最后一条让两片之和恰好 233，左吸收右。
    /// 均分借每次都让两片条数相差不超过 1；中间每一版相邻两片之和都 ≥ 234（M3 那一臂的丙类断言）。
    #[test]
    fn deleting_from_the_left_of_a_pair_borrows_until_the_pair_fits_in_one_and_then_absorbs() {
        let mut containers = full_containers(3);
        let mut borrows = 0;
        let mut deletions = 0;
        let mut absorbed = false;
        // 迭代上界：这一对一共 466 条，每一轮删一条；吸收那一轮退出。
        while !absorbed {
            let inode = containers[0].records[1].inode;
            let after = remove_record_from_leaf_containers(&containers, InodeNumber(inode))
                .expect("删左片第二条");
            deletions += 1;
            absorbed = !after.retired.is_empty();
            if after.rewritten.len() == 2 {
                borrows += 1;
                let counts = record_counts(&after.containers);
                assert!(counts[0].abs_diff(counts[1]) <= 1, "均分借：{counts:?}");
            }
            if !absorbed {
                for pair in after.containers.windows(2) {
                    assert!(
                        pair[0].records.len() + pair[1].records.len() >= 234,
                        "相邻两片之和 ≥ 234"
                    );
                }
            }
            containers = after.containers;
        }
        assert_eq!(deletions, 233, "两片之和 466 删到 233 才装得下一片");
        assert_eq!(
            borrows, 7,
            "116 → 174 / 145 / 131 / 123 / 120 / 118 / 117 七次借"
        );
        assert_eq!(container_numbers(&containers), vec![1, 467]);
        assert_eq!(record_counts(&containers), vec![233, 233]);
    }

    /// 三片各 117 条、删中间那片的一条（判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F1，攻方第二次抽样 K4-4）：
    /// 中间那片剩 116 条，与左邻、右邻之和都恰好 233，两对都装得下。M3「先查右邻」⇒ 中间那片吸收右邻，退役的是右邻（容器号 467），
    /// 左邻原样照抄；先查左邻的写法让左邻吸收中间那片、退役的是中间那片（容器号 234）。两种写法的镜像池级 checker 都不红，方向只有这里钉。
    #[test]
    fn removing_from_the_middle_of_three_half_full_containers_absorbs_the_right_neighbour_first() {
        let mut before = full_containers(3);
        for position in 0..3 {
            keep_the_first_records(&mut before, position, 117);
        }
        let after = remove_record_from_leaf_containers(&before, InodeNumber(300)).expect("删 300");
        assert_eq!(
            container_numbers(&after.containers),
            vec![1, 234],
            "中间那片吸收右邻：留下左邻与中间那片"
        );
        assert_eq!(record_counts(&after.containers), vec![117, 233]);
        assert_eq!(after.containers[0], before[0], "左邻一个字节都不动");
        assert_eq!(
            after.containers[1].identity, before[1].identity,
            "吸收方保留中间那片的身份"
        );
        assert_eq!(
            after.retired,
            vec![RetiredInodeLeafContainer {
                previous_index: InodeLeafContainerIndexInTree::of_position(2),
                identity: before[2].identity,
            }],
            "先查右邻：退役的是右邻，不是中间那片"
        );
        assert_eq!(
            after.rewritten,
            vec![InodeLeafContainerIndexInTree::of_position(1)]
        );
        assert_eq!(
            after.origins[0],
            InodeLeafContainerOrigin::CarriedFrom(InodeLeafContainerIndexInTree::LEFTMOST)
        );
    }

    /// 删的是树里唯一一条（unlink 唯一的文件）：一片都不剩、没有要重写的片，那片身份消失，根照样重写成零条目根。
    #[test]
    fn removing_the_last_record_of_the_tree_leaves_no_container_and_rewrites_the_root() {
        let before = write_records_into_leaf_containers(
            &[],
            &[record(1, 4096, 3)],
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect("一条")
        .containers;
        let after = remove_record_from_leaf_containers(&before, InodeNumber(1)).expect("删 1");
        assert!(after.containers.is_empty(), "一片都不剩");
        assert!(after.rewritten.is_empty());
        assert_eq!(after.retired_identities(), vec![before[0].identity]);
        assert!(after.is_the_root_rewritten(), "根重写成零条目根");
    }

    /// 要删的号不在树里：在任何落盘动作之前拒。
    #[test]
    fn removing_a_number_that_is_not_in_the_tree_is_refused() {
        let before = three_containers();
        assert_eq!(
            remove_record_from_leaf_containers(&before, InodeNumber(468)),
            Err(InodeTreeWriteRefusal::RecordToRemoveIsNotInTheTree {
                inode: InodeNumber(468)
            })
        );
    }

    /// 等价性留档要比的两种改写（`.claude/rules/mutation-sampling.md`「没红的三类，判据不同」第二行：在所有输入上与原式同值的变异，
    /// 把等价性写成测试留档）。判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md` 第三节 G4：两条腿都推等价，
    /// 前提是「相邻两片之和 ≥ 234」——E176 M2、M3 的断言；追加只在最右那片满了才新开一片，删一条之后能并的一对之和恰好 233、
    /// 并完那片满 233 条，删空的那一片删之前只有 1 条、它的两个邻居都满着。
    #[derive(Clone, Copy, Debug)]
    enum MergeRuleRewriting {
        /// 正推腿 m3、攻方 `k4-no-chain`：从某片起的每一趟并掉一对就停，不从幸存那片接着查（借完那一趟也是）。
        EveryPassStopsAfterAbsorbingOnePair,
        /// 正推腿 m4：删空的那一片摘掉之后不从它的左邻起并。
        NothingIsAbsorbedAfterDroppingTheEmptiedContainer,
    }

    /// 改写之下从 `start_position` 那片起的一趟并：交回并没并过。
    fn absorb_pass_under_the_rewriting(
        tree: &mut LeafContainersBeingMerged,
        start_position: usize,
        rewriting: MergeRuleRewriting,
    ) -> bool {
        match rewriting {
            MergeRuleRewriting::EveryPassStopsAfterAbsorbingOnePair => {
                if start_position + 1 < tree.containers.len()
                    && tree.pair_fits_in_one_container(start_position)
                {
                    tree.absorb_the_right_neighbour(start_position);
                    true
                } else if start_position > 0 && tree.pair_fits_in_one_container(start_position - 1)
                {
                    tree.absorb_the_right_neighbour(start_position - 1);
                    true
                } else {
                    false
                }
            }
            MergeRuleRewriting::NothingIsAbsorbedAfterDroppingTheEmptiedContainer => {
                tree.absorb_until_no_pair_fits(start_position)
            }
        }
    }

    /// 改写之下的左从右借：借的条数、挪哪几条、右片的分隔 key 与实现相同，借完那一趟并照改写走。
    fn borrow_under_the_rewriting(
        tree: &mut LeafContainersBeingMerged,
        borrower_position: usize,
        rewriting: MergeRuleRewriting,
    ) {
        match rewriting {
            MergeRuleRewriting::EveryPassStopsAfterAbsorbingOnePair => {
                let lender_position = borrower_position + 1;
                if lender_position >= tree.containers.len()
                    || tree.records_at(borrower_position)
                        >= records_below_which_a_leaf_borrows_from_its_right_neighbour()
                {
                    return;
                }
                let moved_count =
                    (tree.records_at(lender_position) - tree.records_at(borrower_position)) / 2;
                let moved_records: Vec<InodeRecord> = tree.containers[lender_position]
                    .contents
                    .records
                    .drain(..moved_count)
                    .collect();
                let smallest_kept_by_the_lender =
                    tree.containers[lender_position].contents.records[0].inode;
                tree.containers[lender_position]
                    .contents
                    .separator_key_in_the_parent_entry = smallest_kept_by_the_lender;
                tree.containers[lender_position].is_changed_this_publish = true;
                tree.containers[borrower_position]
                    .contents
                    .records
                    .extend(moved_records);
                tree.containers[borrower_position].is_changed_this_publish = true;
                absorb_pass_under_the_rewriting(tree, lender_position, rewriting);
            }
            MergeRuleRewriting::NothingIsAbsorbedAfterDroppingTheEmptiedContainer => {
                tree.borrow_from_the_right_neighbour_if_below_the_floor(borrower_position);
            }
        }
    }

    /// 改写之下删一条：删法、删空摘片、先并后借的次序与 [`remove_record_from_leaf_containers`] 相同，只有改写点到的那一处换掉。
    fn remove_record_under_the_rewriting(
        containers_before_this_publish: &[InodeLeafContainer],
        inode: InodeNumber,
        rewriting: MergeRuleRewriting,
    ) -> InodeLeafContainersAfterThisPublish {
        let holding_position =
            position_of_the_container_holding(containers_before_this_publish, inode)
                .expect("族里每一次删的号都取自那棵树")
                .position();
        let mut tree =
            LeafContainersBeingMerged::from_the_previous_version(containers_before_this_publish);
        tree.containers[holding_position]
            .contents
            .records
            .retain(|kept| kept.inode != inode.0);
        tree.containers[holding_position].is_changed_this_publish = true;
        if tree.records_at(holding_position) == 0 {
            tree.retire(holding_position);
            let has_new_neighbours_on_both_sides =
                holding_position > 0 && holding_position < tree.containers.len();
            match rewriting {
                MergeRuleRewriting::EveryPassStopsAfterAbsorbingOnePair => {
                    if has_new_neighbours_on_both_sides {
                        absorb_pass_under_the_rewriting(&mut tree, holding_position - 1, rewriting);
                    }
                }
                MergeRuleRewriting::NothingIsAbsorbedAfterDroppingTheEmptiedContainer => {}
            }
        } else if !absorb_pass_under_the_rewriting(&mut tree, holding_position, rewriting) {
            borrow_under_the_rewriting(&mut tree, holding_position, rewriting);
        }
        tree.after_this_publish()
    }

    /// 等价性那一族每片记录数的取值：1（删空）、2（借后吸收里的第三片）、116 / 117 / 118（删一条可并、借、借 1 条）、233（满片）。
    const RECORD_COUNTS_OF_THE_EQUIVALENCE_FAMILY: [usize; 6] = [1, 2, 116, 117, 118, 233];
    /// 那一族的树至多这么多片。
    const CONTAINERS_IN_THE_EQUIVALENCE_FAMILY_AT_MOST: usize = 4;

    /// 1 到 4 片、每片记录数取自 [`RECORD_COUNTS_OF_THE_EQUIVALENCE_FAMILY`]、相邻两片之和都 ≥ 234 的每一组记录数。
    fn record_counts_where_every_adjacent_pair_overflows_one_container() -> Vec<Vec<usize>> {
        let capacity = usize::try_from(INODE_LEAF_RECORDS).expect("233");
        let mut of_the_current_length: Vec<Vec<usize>> = vec![Vec::new()];
        let mut family = Vec::new();
        // 迭代上界：CONTAINERS_IN_THE_EQUIVALENCE_FAMILY_AT_MOST 轮，每轮多一片。
        for _ in 0..CONTAINERS_IN_THE_EQUIVALENCE_FAMILY_AT_MOST {
            of_the_current_length = of_the_current_length
                .iter()
                .flat_map(|prefix| {
                    RECORD_COUNTS_OF_THE_EQUIVALENCE_FAMILY
                        .iter()
                        .filter(|count| prefix.last().is_none_or(|last| last + **count > capacity))
                        .map(|count| {
                            let mut longer = prefix.clone();
                            longer.push(*count);
                            longer
                        })
                })
                .collect();
            family.extend(of_the_current_length.iter().cloned());
        }
        family
    }

    /// 记录数照 `record_counts` 的一棵树：inode 号从 1 起连号，每片容器号与分隔 key = 它的第一条（分裂纪律写出来的形状）。
    fn tree_of_record_counts(record_counts: &[usize]) -> Vec<InodeLeafContainer> {
        let mut first_inode_of_the_next_container = 1_u64;
        record_counts
            .iter()
            .map(|count| {
                let first_inode = first_inode_of_the_next_container;
                first_inode_of_the_next_container += u64::try_from(*count).expect("至多 233");
                InodeLeafContainer {
                    identity: PackedIdentity {
                        birth_tree: INODE_TREE,
                        record_type: PACKED_TYPE_INODE,
                        container: first_inode,
                        container_birth: TREE_BIRTH_TXG,
                    },
                    separator_key_in_the_parent_entry: first_inode,
                    records: (first_inode..first_inode_of_the_next_container)
                        .map(|inode| record(inode, 0, 3))
                        .collect(),
                }
            })
            .collect()
    }

    /// 每片删第一条与最后一条（只有一条的那片删一次）：删哪一片决定落法，删片里哪一条只改挪走哪几个号。
    fn inodes_removed_from_each_container(containers: &[InodeLeafContainer]) -> Vec<InodeNumber> {
        let mut removed = Vec::new();
        for container in containers {
            let smallest = container.smallest_inode_number().expect("族里每片至少一条");
            let largest = container.largest_inode_number().expect("族里每片至少一条");
            removed.push(smallest);
            if largest != smallest {
                removed.push(largest);
            }
        }
        removed
    }

    /// 这次删除里有片的分隔 key 变了（只有借会改：右片改成它这一版的最小 key）。
    fn some_separator_key_moved(
        before: &[InodeLeafContainer],
        after: &InodeLeafContainersAfterThisPublish,
    ) -> bool {
        after.containers.iter().any(|kept| {
            before.iter().any(|earlier| {
                earlier.identity == kept.identity
                    && earlier.separator_key_in_the_parent_entry
                        != kept.separator_key_in_the_parent_entry
            })
        })
    }

    /// 正推腿 m3、攻方 `k4-no-chain`（连锁只并一轮）的等价性留档：相邻两片之和都 ≥ 234 的每一棵树上删每一片的头尾一条，
    /// 实现与「每一趟并掉一对就停」逐片相同（容器、重写清单、来处、退役，整份比）——能并的一对之和恰好 233、并完那片满着，
    /// 再与哪个邻居相加都 > 233；借之后借方与借出片之和不变、仍 > 233，借出片吸收右邻之后同理。
    /// 族里删一条并、借后吸收、删空摘片后从左邻起那一趟各至少走到一次；删完之后相邻两片之和仍 ≥ 234（前提对删除封闭）。
    #[test]
    fn stopping_every_absorb_pass_after_one_pair_changes_nothing_when_every_adjacent_pair_overflows_one_container(
    ) {
        let rewriting = MergeRuleRewriting::EveryPassStopsAfterAbsorbingOnePair;
        let mut removals_that_absorbed_without_borrowing = 0;
        let mut removals_that_borrowed_and_then_absorbed = 0;
        let mut removals_that_dropped_a_container_between_two_neighbours = 0;
        for record_counts in record_counts_where_every_adjacent_pair_overflows_one_container() {
            let before = tree_of_record_counts(&record_counts);
            for inode in inodes_removed_from_each_container(&before) {
                let holding_position = position_of_the_container_holding(&before, inode)
                    .expect("取自这棵树")
                    .position();
                let implementation = remove_record_from_leaf_containers(&before, inode)
                    .expect("族里每一次删的号都在树里");
                assert_eq!(
                    implementation,
                    remove_record_under_the_rewriting(&before, inode, rewriting),
                    "各片 {record_counts:?} 删 {inode:?}：实现与「每一趟并掉一对就停」不同"
                );
                for pair in implementation.containers.windows(2) {
                    assert!(
                        pair[0].records.len() + pair[1].records.len() >= 234,
                        "各片 {record_counts:?} 删 {inode:?}：删完之后相邻两片之和 < 234（前提不再成立）"
                    );
                }
                let emptied = record_counts[holding_position] == 1;
                let borrowed = some_separator_key_moved(&before, &implementation);
                if emptied && holding_position > 0 && holding_position + 1 < record_counts.len() {
                    removals_that_dropped_a_container_between_two_neighbours += 1;
                }
                if !emptied && !implementation.retired.is_empty() {
                    if borrowed {
                        removals_that_borrowed_and_then_absorbed += 1;
                    } else {
                        removals_that_absorbed_without_borrowing += 1;
                    }
                }
            }
        }
        assert!(
            removals_that_absorbed_without_borrowing >= 1,
            "族里有删一条就并的"
        );
        assert!(
            removals_that_borrowed_and_then_absorbed >= 1,
            "族里有借完借出的那一片接着并的"
        );
        assert!(
            removals_that_dropped_a_container_between_two_neighbours >= 1,
            "族里有删空摘片之后两边还有邻居、从左邻起看一趟的"
        );
    }

    /// 正推腿 m4（删空摘片后不再并）的等价性留档：相邻两片之和都 ≥ 234 的每一棵树上删每一片的头尾一条，实现与「删空的那一片摘掉之后
    /// 不从它的左邻起并」逐片相同（整份比）——删空的那一片删之前只有 1 条，它的两个邻居都满 233 条，摘掉之后那一对之和 466，
    /// 左邻再与它的左邻之和也 > 233。族里删空摘片之后两边还有邻居的至少走到一次。
    #[test]
    fn not_absorbing_after_dropping_an_emptied_container_changes_nothing_when_every_adjacent_pair_overflows_one_container(
    ) {
        let rewriting = MergeRuleRewriting::NothingIsAbsorbedAfterDroppingTheEmptiedContainer;
        let mut removals_that_dropped_a_container_between_two_neighbours = 0;
        for record_counts in record_counts_where_every_adjacent_pair_overflows_one_container() {
            let before = tree_of_record_counts(&record_counts);
            for inode in inodes_removed_from_each_container(&before) {
                let holding_position = position_of_the_container_holding(&before, inode)
                    .expect("取自这棵树")
                    .position();
                assert_eq!(
                    remove_record_from_leaf_containers(&before, inode)
                        .expect("族里每一次删的号都在树里"),
                    remove_record_under_the_rewriting(&before, inode, rewriting),
                    "各片 {record_counts:?} 删 {inode:?}：实现与「删空摘片之后不再并」不同"
                );
                if record_counts[holding_position] == 1
                    && holding_position > 0
                    && holding_position + 1 < record_counts.len()
                {
                    removals_that_dropped_a_container_between_two_neighbours += 1;
                }
            }
        }
        assert!(
            removals_that_dropped_a_container_between_two_neighbours >= 1,
            "族里有删空摘片之后两边还有邻居的"
        );
    }
}
