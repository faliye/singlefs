//! 回退下界 F 的抬升：I-7.9（回退下界 F 不高于抬 F 的上限） 用到的根环、实例表行与从 journal 重建的抬 F 那一刻的根环，
//! 与 I-7.12（系统配置 F 不低于同盘根上的 F）。

use super::*;

/// 一条根的树表里用户可见的那几条根指针的盘上原样（D16（发布语义） 已定项 1「「非空」从盘上怎么认」：比的是树表条目里这几棵树的根指针，
/// 不比树表单元自己的落点——每一次发布（连空发布）都重写树表单元）。比较与去重的元组是 inode 树、extent 树与全部快照树（码 14、码 15）的根指针：
/// 建快照那次发布两棵用户树不动、只多出一对快照条目它「算非空」（比较与去重的元组加快照树根，core 与 checker 一致）；删快照最后一批删掉那一对，同样算非空。
/// core 那一份是 `crates/singlefs-core/src/recovery.rs` 的 `UserVisibleTreeRootPointers`，两边按同一口径各写一份。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct UserVisibleTreeRootPointers {
    pub(super) inode_tree: Option<Vec<u8>>,
    pub(super) extent_tree: Option<Vec<u8>>,
    /// 码 14、码 15 条目的根指针，按树表里的次序（树表按树 ID 升序）：哪一个快照多了、少了，这一串就不同。
    pub(super) snapshot_tree_roots: Vec<Vec<u8>>,
}

impl UserVisibleTreeRootPointers {
    /// 没有前一条有效根时拿它比：两棵树的条目都没有、一个快照都没有（mkfs 种的第 0 版树表就是这样）。
    const ABSENT: Self = Self {
        inode_tree: None,
        extent_tree: None,
        snapshot_tree_roots: Vec::new(),
    };
}

/// 一条根的树表里 inode 树、extent 树与快照树那几条根指针（树表条目偏移 14 起的 86 字节）。只读不判。
/// 树表指针全零、树表读不出或解不开 ⇒ None：这条根算不算非空判不了，不按空或非空猜（条目那一层的读法在
/// [`user_visible_tree_root_pointers_of_the_tree_table_entries`]）。
pub(super) fn user_visible_tree_root_pointers(
    reader: &dyn ImageReader,
    record: &[u8],
    cache: &mut IndexNodeCache,
) -> Option<UserVisibleTreeRootPointers> {
    user_visible_tree_root_pointers_through_the_tree_table_pointer(reader, &record[36..122], cache)
}

/// 同 [`user_visible_tree_root_pointers`]，只是树表指针（86 字节）直接给：journal 记录新根段里的树表指针（I-7.9 重建抬 F 那一刻的根环时，
/// 根槽已被盖掉的那一版按记录认）走这一条。树表单元已被回收复用时整单元校验和对不上指针、读不出 ⇒ None。
fn user_visible_tree_root_pointers_through_the_tree_table_pointer(
    reader: &dyn ImageReader,
    tree_table_pointer_bytes: &[u8],
    cache: &mut IndexNodeCache,
) -> Option<UserVisibleTreeRootPointers> {
    let tree_table_pointer = parse_node_pointer(tree_table_pointer_bytes);
    if tree_table_pointer.all_zero {
        return None;
    }
    let tree_table = read_index_node_without_judging(
        reader,
        &tree_table_pointer,
        PositionOfASecondaryRead::ThePointerItself,
        cache,
    )?;
    user_visible_tree_root_pointers_of_the_tree_table_entries(&tree_table.entries)
}

/// 一张树表的条目里 inode 树、extent 树与快照树（码 14、码 15）那几条根指针。条目窄于字段表、inode 树或 extent 树出现两条 ⇒ None。
/// 条目格式没有条款的那几种树（livelist、稀疏旁表与不认识的码）与分配记录树、记账树、deadlist 树、`deleted_inodes`、墓碑树、谱系树、
/// 退役树不进「非空」的比较，跳过。
pub(super) fn user_visible_tree_root_pointers_of_the_tree_table_entries(
    tree_table_entries: &[Vec<u8>],
) -> Option<UserVisibleTreeRootPointers> {
    let mut pointers = UserVisibleTreeRootPointers::ABSENT;
    for entry in tree_table_entries {
        if entry.len() < tree_table_entry_bytes() {
            return None;
        }
        let pointer_of_this_tree = match TreeKindForReferenceScan::of(read_u16(entry, 10)) {
            TreeKindForReferenceScan::Inode => &mut pointers.inode_tree,
            TreeKindForReferenceScan::Extent => &mut pointers.extent_tree,
            TreeKindForReferenceScan::SnapshotInodeRoot
            | TreeKindForReferenceScan::SnapshotExtentRoot => {
                pointers.snapshot_tree_roots.push(entry[14..100].to_vec());
                continue;
            }
            TreeKindForReferenceScan::Allocation
            | TreeKindForReferenceScan::Accounting
            | TreeKindForReferenceScan::DeletedInodes
            | TreeKindForReferenceScan::ContainerTree
            | TreeKindForReferenceScan::Deadlist
            | TreeKindForReferenceScan::LoggedOperations
            | TreeKindForReferenceScan::WithoutWalkableEntryFormat(_) => continue,
        };
        if pointer_of_this_tree
            .replace(entry[14..100].to_vec())
            .is_some()
        {
            return None;
        }
    }
    Some(pointers)
}

/// D16（发布语义） 已定项 1 的抬 F 上限：min(每块盘上最新的有效根的 txg, 第 4 新的不同状态的非空有效根的 txg)，不足 4 个时取最旧有效根的 txg。
/// `distinct_non_empty_valid_root_txgs` 是去重之后计入的那几条（[`newest_first_distinct_state_txgs`]）。
/// 对计入的集合单调不减：多计入一条，第 4 新的只会更新或不变，而最旧有效根不新于任何一条计入的根。
fn rollback_floor_ceiling_from(
    newest_valid_root_txg_on_every_device: u64,
    distinct_non_empty_valid_root_txgs: &[u64],
    oldest_valid_root_txg: u64,
) -> u64 {
    let mut newest_first = distinct_non_empty_valid_root_txgs.to_vec();
    newest_first.sort_unstable_by(|left, right| right.cmp(left));
    let fourth_newest_non_empty_or_oldest_valid = newest_first
        .get(3)
        .copied()
        .unwrap_or(oldest_valid_root_txg);
    newest_valid_root_txg_on_every_device.min(fourth_newest_non_empty_or_oldest_valid)
}

/// 数「第 4 新的非空有效根」时去重（D16（发布语义） 已定项 1「「非空」从盘上怎么认」末段）：按 (txg, 实例) 从新到旧，
/// 一条非空根只有在它的两棵用户可见树的根指针与每一条比它新、已计入的根都不同时才计入。`always_counted` 是空不空判不了的那几条
/// （树表读不出，状态也就读不出）：算上限的上沿时它们一律按「计入、状态与谁都不同」算，它们不挡更旧的根计入——计入的集合只多不少，
/// 上沿不低于真值；算下沿时不传它们。交回计入的那几条的 txg。
///
/// 循环：按 (txg, 实例) 从新到旧走一遍（轮数 = 两串条数之和），跨轮携带已计入的状态；没有提前出口。
fn newest_first_distinct_state_txgs(
    determinable_non_empty: &[(u64, u32, UserVisibleTreeRootPointers)],
    always_counted: &[(u64, u32)],
) -> Vec<u64> {
    let mut newest_first: Vec<(u64, u32, Option<&UserVisibleTreeRootPointers>)> =
        determinable_non_empty
            .iter()
            .map(|(txg, instance, pointers)| (*txg, *instance, Some(pointers)))
            .chain(
                always_counted
                    .iter()
                    .map(|(txg, instance)| (*txg, *instance, None)),
            )
            .collect();
    newest_first.sort_unstable_by_key(|(txg, instance, _)| std::cmp::Reverse((*txg, *instance)));
    let mut counted_states: Vec<&UserVisibleTreeRootPointers> = Vec::new();
    let mut counted = Vec::new();
    for (txg, _, pointers) in newest_first {
        match pointers {
            None => counted.push(txg),
            Some(state) => {
                if !counted_states.contains(&state) {
                    counted_states.push(state);
                    counted.push(txg);
                }
            }
        }
    }
    counted
}

/// 按一条抬 F 的根之前的根算出来的上限。有效根里有树表读不出的（它或它前一条读不出，它空不空判不了）时上限只知道一个区间：
/// 那几条都算空是下沿、都算非空是上沿（[`rollback_floor_ceiling_from`] 对非空集合单调）；树表都读得出时两沿相等。
struct RollbackFloorCeilingBeforeTheRaise {
    lowest_possible: u64,
    highest_possible: u64,
    newest_valid_root_per_device: BTreeMap<u32, u64>,
    non_empty_valid_root_txgs: Vec<u64>,
    valid_root_txgs_whose_emptiness_is_undeterminable: Vec<u64>,
    oldest_valid_root_txg: u64,
    /// 去掉最旧那条有效根之后、空不空判得了的非空有效根去重计入了几条（[`newest_first_distinct_state_txgs`]，不传判不了的）。
    /// 最旧那条的「非空」是跟 [`UserVisibleTreeRootPointers::ABSENT`] 比的，它真正的前一条可以是已经不在这一组里的旧根，不算它。
    distinct_non_empty_states_newer_than_the_oldest_valid_root: usize,
}

/// D16（发布语义） 已定项 1 的抬 F 上限数到第几新的非空有效根（I-7.9 判窗口里的根够不够定住上限时用）。
const NON_EMPTY_VALID_ROOT_RANK_THE_CEILING_TAKES: usize = 4;

/// 算抬 F 上限用的一条有效根：住哪块盘、(txg, 实例)、它树表里用户可见的那几条根指针（None：树表读不出，空不空判不了）。
/// 窗口里的根（[`rollback_floor_ceiling_before_the_raise`]）与重建出来的抬 F 那一刻的根环（[`rollback_floor_against_the_ring_reconstructed_at_the_raise`]）
/// 共用一个上限算法（[`rollback_floor_ceiling_of_the_valid_roots`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct RootBeforeTheRaise {
    pub(super) device: u32,
    pub(super) checkpoint_txg: u64,
    pub(super) instance: u32,
    user_visible_tree_root_pointers: Option<UserVisibleTreeRootPointers>,
}

/// 一组按 (txg, 实例) 升序排好的有效根（`valid_roots_oldest_first`，有效已在调用方滤过）算出的抬 F 上限两沿（[`RollbackFloorCeilingBeforeTheRaise`]）。
/// 非空 = 跟前一条有效根比两条用户可见树的根指针，最旧的那条跟 [`UserVisibleTreeRootPointers::ABSENT`] 比；它或它前一条读不出的空不空判不了。
/// 一条有效根都没有 ⇒ None。循环：按 (txg, 实例) 从旧到新走一遍，跨轮带前一条的指针；没有提前出口。
fn rollback_floor_ceiling_of_the_valid_roots(
    valid_roots_oldest_first: &[RootBeforeTheRaise],
) -> Option<RollbackFloorCeilingBeforeTheRaise> {
    let oldest_valid_root = valid_roots_oldest_first.first()?;
    let oldest_valid_root_txg = oldest_valid_root.checkpoint_txg;
    let mut newest_valid_root_per_device: BTreeMap<u32, u64> = BTreeMap::new();
    for root in valid_roots_oldest_first {
        let newest = newest_valid_root_per_device
            .entry(root.device)
            .or_insert(root.checkpoint_txg);
        *newest = (*newest).max(root.checkpoint_txg);
    }
    let newest_valid_root_txg_on_every_device = newest_valid_root_per_device
        .values()
        .copied()
        .min()
        .expect("上面 first() 过：至少一条有效根，它的盘在这张表里");
    let mut non_empty_valid_roots: Vec<(u64, u32, UserVisibleTreeRootPointers)> = Vec::new();
    let mut valid_roots_whose_emptiness_is_undeterminable: Vec<(u64, u32)> = Vec::new();
    let mut previous_valid_root_pointers = Some(UserVisibleTreeRootPointers::ABSENT);
    for root in valid_roots_oldest_first {
        let pointers = root.user_visible_tree_root_pointers.clone();
        match (&pointers, &previous_valid_root_pointers) {
            (Some(this_root), Some(previous_root)) => {
                if this_root != previous_root {
                    non_empty_valid_roots.push((
                        root.checkpoint_txg,
                        root.instance,
                        this_root.clone(),
                    ));
                }
            }
            (None, _) | (_, None) => {
                valid_roots_whose_emptiness_is_undeterminable
                    .push((root.checkpoint_txg, root.instance));
            }
        }
        previous_valid_root_pointers = pointers;
    }
    let non_empty_valid_roots_newer_than_the_oldest: Vec<(u64, u32, UserVisibleTreeRootPointers)> =
        non_empty_valid_roots
            .iter()
            .filter(|(txg, instance, _)| {
                (*txg, *instance) != (oldest_valid_root_txg, oldest_valid_root.instance)
            })
            .cloned()
            .collect();
    let non_empty_valid_root_txgs: Vec<u64> = non_empty_valid_roots
        .iter()
        .map(|(txg, _, _)| *txg)
        .collect();
    let valid_root_txgs_whose_emptiness_is_undeterminable: Vec<u64> =
        valid_roots_whose_emptiness_is_undeterminable
            .iter()
            .map(|(txg, _)| *txg)
            .collect();
    Some(RollbackFloorCeilingBeforeTheRaise {
        lowest_possible: rollback_floor_ceiling_from(
            newest_valid_root_txg_on_every_device,
            &newest_first_distinct_state_txgs(&non_empty_valid_roots, &[]),
            oldest_valid_root_txg,
        ),
        highest_possible: rollback_floor_ceiling_from(
            newest_valid_root_txg_on_every_device,
            &newest_first_distinct_state_txgs(
                &non_empty_valid_roots,
                &valid_roots_whose_emptiness_is_undeterminable,
            ),
            oldest_valid_root_txg,
        ),
        newest_valid_root_per_device,
        non_empty_valid_root_txgs,
        valid_root_txgs_whose_emptiness_is_undeterminable,
        oldest_valid_root_txg,
        distinct_non_empty_states_newer_than_the_oldest_valid_root:
            newest_first_distinct_state_txgs(&non_empty_valid_roots_newer_than_the_oldest, &[])
                .len(),
    })
}

/// 窗口里的根算出的上限的下沿，是不是就是抬 F 那一刻整个环算出的下沿（C717（回退下界上限的单窗口盲带）丁那一判之前先问的一句）。
/// 窗口外那些抬 F 那一刻在环里的根（之后被盖掉的、洞里留着的）txg 都 ≤ N − 3S，比窗口里每一条都旧，两样同时成立时它们改不了下沿：
/// ① 根环每个区域所在的盘上窗口里都有一条有效根——「每块盘上最新的有效根」只看每块盘上最新那条，更旧的根挤不进来；
/// ② 去掉窗口里最旧那条，空不空判得了的非空有效根去重仍计入不少于 4 条——第 4 新的那条落在它们里面，更旧的根排在后面
/// （最旧那条不算：它真正的前一条可以是窗口外的旧根，跟 ABSENT 比出的「非空」不作数）。
/// 两样有一样不成立，窗口里的上限可以高过抬 F 那一刻的（C717 的盲带），要按 journal 重建那一刻的环再判。
fn window_lowest_ceiling_is_the_lowest_ceiling_of_the_ring_at_the_raise(
    ceiling_of_the_window: &RollbackFloorCeilingBeforeTheRaise,
    geometry: &PoolGeometry,
) -> bool {
    let every_device_of_the_ring_holds_a_valid_root_in_the_window =
        (0..geometry.regions).all(|region| {
            // 下标在范围内不是这里判的：`geometry_of` 已经把 R > 3 的槽拒掉。
            let device = geometry.region_devices[usize::try_from(region).expect(
                "区域号：geometry_of 判过 R ≤ REGION_DEVICE_FIELDS_IN_THE_SYSTEM_CONFIGURATION",
            )];
            ceiling_of_the_window
                .newest_valid_root_per_device
                .contains_key(&device)
        });
    every_device_of_the_ring_holds_a_valid_root_in_the_window
        && ceiling_of_the_window.distinct_non_empty_states_newer_than_the_oldest_valid_root
            >= NON_EMPTY_VALID_ROOT_RANK_THE_CEILING_TAKES
}

/// 算一条抬 F 的根 `raising_root` 那一刻的上限（I-7.9（回退下界 F 不高于抬 F 的上限））：
/// 只用根环里 txg 比它小、落在锚在这张镜像根环里最新根上的一圈窗口里的根（txg > N − 3S，`window_anchored_at_the_newest_root_of_the_image`），
/// 挡的是环上有洞时洞里留下的旧根（理由见 [`judge_rollback_floor_raises_against_their_ceilings`]）。抬 F 的上限本身取环里全部有效根
/// 原来另取的那个锚在它自己 txg 上的窗口（照上限只在一圈窗口里取）随之拿掉：N 是镜像里最新根的 txg、
/// 不小于被判那条根的 txg，那个窗口本来就罩着这个窗口，拿掉不改判定。有效 = 按**它自己**指着的实例表不被抛弃 ∧ txg ≥ `floor_before_the_raise`（抬之前的 F）；
/// 非空 = 按 (txg, 实例) 排，跟前一条有效根比两条用户可见树的根指针，最旧的那条跟 [`UserVisibleTreeRootPointers::ABSENT`] 比。
/// 它自己指着的实例表读不出、它之前一条有效根都没有 ⇒ None（上限无从算起：抬 F 的入口在这两格上拒，盘上出现这样一条抬 F 的根，
/// 要么表后来坏了，要么撑它的根已被环盖掉）。
fn rollback_floor_ceiling_before_the_raise(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
    roots: &[(u64, u64, crate::RootView)],
    raising_root: &crate::RootView,
    floor_before_the_raise: u64,
    window_anchored_at_the_newest_root_of_the_image: OneRevolutionWindow,
    cache: &mut IndexNodeCache,
) -> Option<RollbackFloorCeilingBeforeTheRaise> {
    let instance_table_rows_of_the_raising_root =
        instance_table_rows_of_root_without_judging(reader, &raising_root.record_bytes)?;
    let mut valid_roots_before: Vec<(u32, &crate::RootView)> = roots
        .iter()
        .filter(|(_, _, root)| {
            root.checkpoint_txg < raising_root.checkpoint_txg
                && window_anchored_at_the_newest_root_of_the_image.contains(root.checkpoint_txg)
                && root.checkpoint_txg >= floor_before_the_raise
                && !abandoned_by_instance_table_rows(&instance_table_rows_of_the_raising_root, root)
        })
        .map(|(region, _, root)| {
            // 下标在范围内不是这里判的：`geometry_of` 已经把 R > 3 的槽拒掉，`roots` 的区域号都来自 `root_slot_positions`。
            let device = geometry.region_devices[usize::try_from(*region).expect(
                "区域号：geometry_of 判过 R ≤ REGION_DEVICE_FIELDS_IN_THE_SYSTEM_CONFIGURATION",
            )];
            (device, root)
        })
        .collect();
    valid_roots_before.sort_unstable_by_key(|(_, root)| (root.checkpoint_txg, root.instance));
    let valid_roots_oldest_first: Vec<RootBeforeTheRaise> = valid_roots_before
        .iter()
        .map(|(device, root)| RootBeforeTheRaise {
            device: *device,
            checkpoint_txg: root.checkpoint_txg,
            instance: root.instance,
            user_visible_tree_root_pointers: user_visible_tree_root_pointers(
                reader,
                &root.record_bytes,
                cache,
            ),
        })
        .collect();
    rollback_floor_ceiling_of_the_valid_roots(&valid_roots_oldest_first)
}

/// 重建抬 F 那一刻的根环时，镜像上一个根环槽的样子（C717（回退下界上限的单窗口盲带）：借 journal 判红）。
#[derive(Clone, Debug, PartialEq, Eq)]
enum RingSlotOfTheImageSeenFromTheRaise {
    /// 坐着一条自证过的根，txg 比抬 F 那条根小：抬 F 那一刻它就在这一槽（一槽里先后住的根 txg 只增；抬 F 之后写的根 txg 都比抬 F 那条大，
    /// 新实例接在环里最大的 txg 之后，而抬 F 那条根一直在环里）。
    HoldsARootOlderThanTheRaise(RootBeforeTheRaise),
    /// 坐着一条自证过的根，txg 不小于抬 F 那条根：抬 F 那一刻住在这一槽的那条已被盖掉，要按 journal 认。
    OverwrittenSinceTheRaise,
    /// 读得到、整槽全零：从没写过，抬 F 那一刻也空着。
    AllZero,
    /// 读不到，或不全零而不自证：抬 F 那一刻住的是什么无从认。
    DoesNotReadBack,
}

/// journal 里带提交标记、由恢复的读者收得下的记录认出的一版（实例、它新根段里那张树表的用户可见根指针；树表读不出、单元已被复用时 None）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct VersionNamedByTheJournal {
    pub(super) instance: u32,
    user_visible_tree_root_pointers: Option<UserVisibleTreeRootPointers>,
}

/// 按重建出的抬 F 那一刻的根环判 F（[`rollback_floor_against_the_ring_reconstructed_at_the_raise`]）的五种结局。
/// 被盖那几槽的住户落没落盘判不了时环有几种可能（[`possible_roots_in_the_ring_slot_at_the_raise`]），每一种各判一次：前三种是每一种都判成同一种。
#[derive(Clone, Debug, PartialEq, Eq)]
enum RollbackFloorAgainstTheRingAtTheRaise {
    /// F 不高于重建的环算出的上限下沿。
    Holds,
    /// F 高于重建的环算出的上限上沿：上限、最旧有效根与非空有效根各是哪几条写进说明。
    Violated(String),
    /// 有效根里有树表读不出的，F 落在上限两沿之间。
    CeilingBetweenItsBounds,
    /// 环重建不出（journal 罩不到、槽读不出、没有有效根、可能的环多于上界）：说明写卡在哪一槽、哪一版。
    RingNotReconstructed(String),
    /// 可能的环几种各判一次，判得不一样：说明写几种、各判成什么。
    PossibleRingsDisagree(String),
}

/// 按重建出的抬 F 那一刻的一种可能的根环判 F 的四种结局（[`rollback_floor_against_the_ring_reconstructed_at_the_raise`] 逐种判、再比是不是同一种）。
#[derive(Clone, Debug, PartialEq, Eq)]
enum RollbackFloorAgainstAPossibleRingAtTheRaise {
    Holds,
    Violated(String),
    CeilingBetweenItsBounds,
    WithoutAValidRoot,
}

/// 按 journal 重建抬 F 那一刻的根环时，每一槽可能住户的组合至多判这么多种，多于它整条报部分没判（判决
/// `research/prompts/m3-close-b19-reason-r1-main-verification.md` 第二节 Q2「定」那一行要的上界；取两腿模型同一个数，
/// `research/prompts/m3-close-b19-reason-r1-opus-model/ding_port.py` 的 `MAX_COMBOS`）。GPU 判器扫描内核
/// `crash_judge_checker_scan.wgsl` 的 `C_POSSIBLE_RINGS_AT_THE_RAISE_JUDGED_AT_MOST` 是同一个数。
const POSSIBLE_RINGS_AT_THE_RAISE_JUDGED_AT_MOST: usize = 4096;

/// mkfs 种下的第 0 代根的实例代号（D22（单元原子性怎么合成） 已定项 8；实现一侧是 `crates/singlefs-core/src/make_filesystem.rs` 的
/// `MKFS_INSTANCE_GENERATION`，池级 checker 不引实现，这里另记一份）。按 journal 重建抬 F 那一刻的根环时，已被盖掉的第 0 代根按它排序、认实例表的行。
const GENESIS_ROOT_INSTANCE: u32 = 0;

/// 根环槽（区域 `region`、槽 `slot`）按落点式（I-7.17：区域 txg mod R、槽 (txg div R) mod S）住的正 txg 里比 `raising_txg` 小的最大那个；
/// 一个都没有（落点式住进这一槽的最小正 txg 不比它小）⇒ None。住进这一槽的正 txg 是 region + R × slot 加环长 3S 的倍数。
fn largest_positive_txg_the_ring_slot_names_below(
    region: u64,
    slot: u64,
    raising_txg: u64,
    geometry: &PoolGeometry,
) -> Option<u64> {
    let ring_length = geometry.regions * geometry.slots_per_region;
    let residue = region + geometry.regions * slot;
    let below = raising_txg.checked_sub(1)?;
    let largest = residue + (below.checked_sub(residue)? / ring_length) * ring_length;
    (largest > 0).then_some(largest)
}

/// 镜像上读回全零的根环槽给一条抬 F 的根（txg `raising_txg`）定的上限（C961（全零根槽没并进 checker 的抬 F 上限），D16（发布语义） 已定项 1
/// 「抬 F 的上限」那一行：读坏槽与读回全零的槽都并进上限，挂载那一刻读回全零、之后这个进程没写过的槽当作可能是一条非空有效根，F 不越过它，
/// 能住的最大 txg 的取法同读坏的槽）。交回最低的那个上限连同它那一槽（区域, 槽）；一槽都不定上限 ⇒ None。
/// 每个读回全零的槽取落点式住进它、比抬 F 那条根小的最大正 txg（[`largest_positive_txg_the_ring_slot_names_below`]）：镜像上这一刻全零的槽，
/// 抬 F 那一次挂载时也全零（槽只会被写、不会被抹回全零）；抬 F 那个进程从它第一次发布起每个 txg 都落了根，落点式落进这一槽的 txg 要是在那一段里，
/// 这一槽就不再全零——所以这个 txg 不在那一段里，就是写者那一边在 (N − 3S, 新实例第一次发布的 txg − 1] 里取的那一个
/// （`crates/singlefs-core/src/mount.rs` 的 `txgs_ring_slots_bad_at_mount_can_hold`：一圈窗口里每一槽恰好落一个 txg，那个 txg 一定在窗口里）。
/// 只认比 `oldest_txg_of_the_raising_instance_in_the_ring`（环里抬 F 那个实例最旧的那条根）小的：那个实例第一次发布的 txg 不高于它，
/// 合法写者写出的镜像上这样的槽能住的 txg 都比它小；不比它小的（txg 中间跳过了一段、环里那几槽从没写过的手造流）写者不会并，checker 也不并。
/// 只认不低于抬之前的 F 的（写者只并能住的最大 txg 不低于今天的 F 的槽）。读坏的槽不在这里：checker 看不到挂载那一刻的读数（判决
/// `research/prompts/m3-close-b19-reason-r3-main-verification.md` 第二节 Q3 第 2 问那一行）。
fn rollback_floor_ceiling_held_by_the_ring_slots_reading_back_all_zero(
    geometry: &PoolGeometry,
    root_ring_slot_readings: &BTreeMap<(u64, u64), RootRingSlotReading>,
    raising_txg: u64,
    oldest_txg_of_the_raising_instance_in_the_ring: u64,
    floor_before_the_raise: u64,
) -> Option<(u64, (u64, u64))> {
    root_ring_slot_readings
        .iter()
        .filter_map(|((region, slot), reading)| match reading {
            RootRingSlotReading::AllZero => largest_positive_txg_the_ring_slot_names_below(
                *region,
                *slot,
                raising_txg,
                geometry,
            )
            .filter(|txg_the_slot_can_hold| {
                *txg_the_slot_can_hold < oldest_txg_of_the_raising_instance_in_the_ring
                    && *txg_the_slot_can_hold >= floor_before_the_raise
            })
            .map(|txg_the_slot_can_hold| (txg_the_slot_can_hold, (*region, *slot))),
            RootRingSlotReading::DoesNotReadBack | RootRingSlotReading::SelfVerified { .. } => None,
        })
        .min()
}

/// 抬 F 那一刻一个根环槽里可能住的每一条根（不判有效）；交回的每一项是一种可能，`None` 是那一刻空着，或住着 txg 低于抬之前的 F 的根
/// （那样的根不是有效根，对上限与空着一样）。槽在镜像上坐着比抬 F 那条根旧的根、整槽全零时只有一种可能。
/// 槽在镜像上已被盖掉时从落点式在它比抬 F 那条根小的最大 txg `u` 起往下逐层认
/// 按 journal 认那一层的那一版（[`VersionNamedByTheJournal`]），它与抬 F 那条根同一实例，或它那个实例在抬 F 那条根的
/// 实例表里有行、每一行 (i, T, W) 都有 u < T（实例中间的一版：之后那个实例还发布过，它的根落了盘），认作那一刻住着它、到此为止；
/// u 是某个实例的 T、在被抛弃的时间线上（u ≥ T）、或那个实例没有行，它的根槽可以没落、那一槽仍住着上一圈的根：它算一种可能，
/// 再往下认落点式住进这一槽的上一圈 u − 3S。落点式住进这一槽的正 txg 认完了（只剩 mkfs 种的第 0 代根那一槽或从没写过）：槽 0 而抬之前的 F 是 0 时
/// 只算第 0 代根一种（mkfs 在每个区域的槽 0 都种了它，不另算空着；它的树表是 mkfs 种的 0 条那一版，按 [`UserVisibleTreeRootPointers::ABSENT`] 记，
///  第 3 问「定」），别的槽算空着一种；认到的那一层 txg 低于抬之前的 F（窗口外）：
/// 那一层与再往下的每一层都不是有效根，记一种 `None`、到此为止。journal 里没有那一层带提交标记的记录（转出 journal 环了）或有几个实例的、
/// 槽读不出 ⇒ Err(卡在哪)。层数不超过重放读的那一份环里带提交标记的版本数加二（每多认一层，那一层都要有 journal 里的记录）。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是重建一槽要的一样输入：几何、那一槽、镜像上的样子、journal 认出的版本、抬 F 那条根的两样身份与它的实例表、抬之前的 F"
)]
fn possible_roots_in_the_ring_slot_at_the_raise(
    geometry: &PoolGeometry,
    (region, slot): (u64, u64),
    slot_of_the_image: &RingSlotOfTheImageSeenFromTheRaise,
    versions_named_by_the_journal: &BTreeMap<u64, Vec<VersionNamedByTheJournal>>,
    (raising_txg, raising_instance): (u64, u32),
    instance_table_rows_of_the_raising_root: &[InstanceTableRow],
    floor_before_the_raise: u64,
) -> Result<Vec<Option<RootBeforeTheRaise>>, String> {
    // 下标在范围内不是这里判的：`geometry_of` 已经把 R > 3 的槽拒掉，`region` 来自 0..R。
    let device = geometry.region_devices[usize::try_from(region)
        .expect("区域号：geometry_of 判过 R ≤ REGION_DEVICE_FIELDS_IN_THE_SYSTEM_CONFIGURATION")];
    match slot_of_the_image {
        RingSlotOfTheImageSeenFromTheRaise::HoldsARootOlderThanTheRaise(root) => {
            Ok(vec![Some(root.clone())])
        }
        RingSlotOfTheImageSeenFromTheRaise::AllZero => Ok(vec![None]),
        RingSlotOfTheImageSeenFromTheRaise::DoesNotReadBack => Err(format!(
            "根环槽（区域 {region}、槽 {slot}）读不出：抬 F 那一刻它住的是什么无从认"
        )),
        RingSlotOfTheImageSeenFromTheRaise::OverwrittenSinceTheRaise => {
            let ring_length = geometry.regions * geometry.slots_per_region;
            let mut possible_roots: Vec<Option<RootBeforeTheRaise>> = Vec::new();
            let mut txg_the_slot_may_hold =
                largest_positive_txg_the_ring_slot_names_below(region, slot, raising_txg, geometry);
            loop {
                let Some(overwritten_txg) = txg_the_slot_may_hold else {
                    // 落点式住进这一槽的正 txg 认完了：每个区域的槽 0 住着 mkfs 种的第 0 代根（mkfs 在三个区域的槽 0 都写了它，那一槽不可能空着），
                    // 别的槽从没写过、空着。
                    // 第 0 代根的树表是 mkfs 种的第 0 版（0 条条目，`.claude/kb/layout/01-first-txn.md` 的 m2）：两棵用户可见树都没有、一个快照都没有。
                    // 抬之前的 F 高于 0 时第 0 代根不是有效根，对上限与空着一样。
                    if slot == RING_SLOT_THE_GENESIS_ROOT_IS_SEEDED_IN_EVERY_REGION
                        && floor_before_the_raise == GENESIS_CHECKPOINT_TXG
                    {
                        possible_roots.push(Some(RootBeforeTheRaise {
                            device,
                            checkpoint_txg: GENESIS_CHECKPOINT_TXG,
                            instance: GENESIS_ROOT_INSTANCE,
                            user_visible_tree_root_pointers: Some(
                                UserVisibleTreeRootPointers::ABSENT,
                            ),
                        }));
                    } else {
                        possible_roots.push(None);
                    }
                    return Ok(possible_roots);
                };
                if overwritten_txg < floor_before_the_raise {
                    // 窗口外：这一层与再往下的每一层 txg 都低于抬之前的 F，住哪一条都不是有效根。
                    possible_roots.push(None);
                    return Ok(possible_roots);
                }
                let versions = versions_named_by_the_journal
                    .get(&overwritten_txg)
                    .map_or(&[][..], Vec::as_slice);
                let [version] = versions else {
                    return Err(format!(
                        "根环槽（区域 {region}、槽 {slot}）在抬 F 之后被盖，抬 F 那一刻可能住的 txg {overwritten_txg} 那一版 journal 里有 {} 个实例带提交标记的记录（要恰好一个）",
                        versions.len()
                    ));
                };
                let rows_of_its_instance: Vec<&InstanceTableRow> =
                    instance_table_rows_of_the_raising_root
                        .iter()
                        .filter(|row| row.instance == version.instance)
                        .collect();
                let its_root_landed = version.instance == raising_instance
                    || (!rows_of_its_instance.is_empty()
                        && rows_of_its_instance
                            .iter()
                            .all(|row| overwritten_txg < row.published_checkpoint_txg));
                possible_roots.push(Some(RootBeforeTheRaise {
                    device,
                    checkpoint_txg: overwritten_txg,
                    instance: version.instance,
                    user_visible_tree_root_pointers: version
                        .user_visible_tree_root_pointers
                        .clone(),
                }));
                if its_root_landed {
                    return Ok(possible_roots);
                }
                // 它的根槽可以没落：那一槽仍住着落点式住进它的上一圈那一条（或更早的），往下认。
                txg_the_slot_may_hold = overwritten_txg
                    .checked_sub(ring_length)
                    .filter(|earlier_txg| *earlier_txg > 0);
            }
        }
    }
}

/// I-7.9 不带卸载记号那一支的丁（C717（回退下界上限的单窗口盲带））：
/// 按镜像上的根环槽（`ring_slots_of_the_image`，键 (区域, 槽)，缺的槽当读不出）与 journal 认出的版本（`versions_named_by_the_journal`，键 txg）
/// 认抬 F 那一刻环里每一槽可能住的每一条根（[`possible_roots_in_the_ring_slot_at_the_raise`]），每一槽取一种可能拼成一种可能的环，
/// 每一种各滤出有效根（txg ≥ 抬之前的 F、按抬 F 那条根的实例表不被抛弃）、照 D16（发布语义） 已定项 1 的准入式子算上限
/// （[`rollback_floor_ceiling_of_the_valid_roots`]）判一次 F；每一种判成同一种才照它报，判得不一样报可能的环判得不一致
/// 洞里留着的有效根 X 在这里与窗口里的根一起算，F 落在 X 与窗口里最旧有效根 W 之间时判红；有一槽认不出、可能的环多于
/// [`POSSIBLE_RINGS_AT_THE_RAISE_JUDGED_AT_MOST`] 种整条报环重建不出，不报红。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是重建要的一样输入：几何、镜像上的根环槽、journal 认出的版本、抬 F 那条根的两样身份与它的实例表、抬之前与之后的 F"
)]
fn rollback_floor_against_the_ring_reconstructed_at_the_raise(
    geometry: &PoolGeometry,
    ring_slots_of_the_image: &BTreeMap<(u64, u64), RingSlotOfTheImageSeenFromTheRaise>,
    versions_named_by_the_journal: &BTreeMap<u64, Vec<VersionNamedByTheJournal>>,
    (raising_txg, raising_instance): (u64, u32),
    instance_table_rows_of_the_raising_root: &[InstanceTableRow],
    floor_before_the_raise: u64,
    raised_floor: u64,
) -> RollbackFloorAgainstTheRingAtTheRaise {
    let mut possible_roots_of_each_slot: Vec<Vec<Option<RootBeforeTheRaise>>> = Vec::new();
    let mut possible_ring_count: usize = 1;
    for region in 0..geometry.regions {
        for slot in 0..geometry.slots_per_region {
            let slot_of_the_image = ring_slots_of_the_image
                .get(&(region, slot))
                .unwrap_or(&RingSlotOfTheImageSeenFromTheRaise::DoesNotReadBack);
            match possible_roots_in_the_ring_slot_at_the_raise(
                geometry,
                (region, slot),
                slot_of_the_image,
                versions_named_by_the_journal,
                (raising_txg, raising_instance),
                instance_table_rows_of_the_raising_root,
                floor_before_the_raise,
            ) {
                Err(stuck_at) => {
                    return RollbackFloorAgainstTheRingAtTheRaise::RingNotReconstructed(stuck_at)
                }
                Ok(possible_roots) => {
                    possible_ring_count = possible_ring_count.saturating_mul(possible_roots.len());
                    possible_roots_of_each_slot.push(possible_roots);
                }
            }
        }
    }
    if possible_ring_count > POSSIBLE_RINGS_AT_THE_RAISE_JUDGED_AT_MOST {
        return RollbackFloorAgainstTheRingAtTheRaise::RingNotReconstructed(format!(
            "被盖那几槽的住户落没落盘判不了，可能的环至少 {possible_ring_count} 种，多于上界 {POSSIBLE_RINGS_AT_THE_RAISE_JUDGED_AT_MOST} 种"
        ));
    }
    // 第 possible_ring_index 种：把它写成混合进制，每一槽一位、那一槽的可能数是这一位的进制，这一位上的数字就是那一槽取第几种可能。
    let verdicts_of_the_possible_rings: Vec<RollbackFloorAgainstAPossibleRingAtTheRaise> = (0
        ..possible_ring_count)
        .map(|possible_ring_index| {
            let mut digits_left = possible_ring_index;
            let mut valid_roots_oldest_first: Vec<RootBeforeTheRaise> = Vec::new();
            for possible_roots in &possible_roots_of_each_slot {
                let possible_root = &possible_roots[digits_left % possible_roots.len()];
                digits_left /= possible_roots.len();
                let Some(root) = possible_root else {
                    continue;
                };
                let is_valid = root.checkpoint_txg >= floor_before_the_raise
                    && !instance_table_rows_of_the_raising_root.iter().any(|row| {
                        row.instance == root.instance
                            && root.checkpoint_txg > row.published_checkpoint_txg
                    });
                if is_valid {
                    valid_roots_oldest_first.push(root.clone());
                }
            }
            valid_roots_oldest_first
                .sort_unstable_by_key(|root| (root.checkpoint_txg, root.instance));
            let Some(ceiling) = rollback_floor_ceiling_of_the_valid_roots(&valid_roots_oldest_first)
            else {
                return RollbackFloorAgainstAPossibleRingAtTheRaise::WithoutAValidRoot;
            };
            if raised_floor > ceiling.highest_possible {
                return RollbackFloorAgainstAPossibleRingAtTheRaise::Violated(format!(
                    "按 journal 重建的抬 F 那一刻的根环算出的上限 {}：每块盘上最新的有效根 {:?}，非空有效根 {:?}，空不空判不了的有效根 {:?}，最旧有效根 txg {}",
                    ceiling.highest_possible,
                    ceiling.newest_valid_root_per_device,
                    ceiling.non_empty_valid_root_txgs,
                    ceiling.valid_root_txgs_whose_emptiness_is_undeterminable,
                    ceiling.oldest_valid_root_txg
                ));
            }
            if raised_floor <= ceiling.lowest_possible {
                RollbackFloorAgainstAPossibleRingAtTheRaise::Holds
            } else {
                RollbackFloorAgainstAPossibleRingAtTheRaise::CeilingBetweenItsBounds
            }
        })
        .collect();
    let verdict_of_the_first_possible_ring = verdicts_of_the_possible_rings
        .first()
        .expect("每一槽至少一种可能，可能的环至少一种")
        .clone();
    let every_possible_ring_judges_alike = verdicts_of_the_possible_rings.iter().all(|verdict| {
        std::mem::discriminant(verdict)
            == std::mem::discriminant(&verdict_of_the_first_possible_ring)
    });
    if !every_possible_ring_judges_alike {
        let count_of = |kind: &RollbackFloorAgainstAPossibleRingAtTheRaise| {
            verdicts_of_the_possible_rings
                .iter()
                .filter(|verdict| std::mem::discriminant(*verdict) == std::mem::discriminant(kind))
                .count()
        };
        return RollbackFloorAgainstTheRingAtTheRaise::PossibleRingsDisagree(format!(
            "被盖那几槽的住户落没落盘判不了，可能的环 {possible_ring_count} 种各判一次：成立 {}、F 高于上限 {}、F 落在上限两沿之间 {}、一条有效根都没有 {}",
            count_of(&RollbackFloorAgainstAPossibleRingAtTheRaise::Holds),
            count_of(&RollbackFloorAgainstAPossibleRingAtTheRaise::Violated(String::new())),
            count_of(&RollbackFloorAgainstAPossibleRingAtTheRaise::CeilingBetweenItsBounds),
            count_of(&RollbackFloorAgainstAPossibleRingAtTheRaise::WithoutAValidRoot)
        ));
    }
    match verdict_of_the_first_possible_ring {
        RollbackFloorAgainstAPossibleRingAtTheRaise::Holds => {
            RollbackFloorAgainstTheRingAtTheRaise::Holds
        }
        RollbackFloorAgainstAPossibleRingAtTheRaise::Violated(ceiling_of_the_ring) => {
            RollbackFloorAgainstTheRingAtTheRaise::Violated(if possible_ring_count == 1 {
                ceiling_of_the_ring
            } else {
                format!("{ceiling_of_the_ring}（可能的环 {possible_ring_count} 种都判红，这是其中头一种）")
            })
        }
        RollbackFloorAgainstAPossibleRingAtTheRaise::CeilingBetweenItsBounds => {
            RollbackFloorAgainstTheRingAtTheRaise::CeilingBetweenItsBounds
        }
        RollbackFloorAgainstAPossibleRingAtTheRaise::WithoutAValidRoot => {
            RollbackFloorAgainstTheRingAtTheRaise::RingNotReconstructed(
                "重建出的抬 F 那一刻的根环里一条有效根都没有：上限无从算起".to_string(),
            )
        }
    }
}

/// 镜像上每个根环槽从抬 F 那条根（txg `raising_txg`）看过去的样子（[`RingSlotOfTheImageSeenFromTheRaise`]）。坐着比它旧的根时顺带读那条根的树表。
fn ring_slots_of_the_image_seen_from_the_raise(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
    roots: &[(u64, u64, crate::RootView)],
    root_ring_slot_readings: &BTreeMap<(u64, u64), RootRingSlotReading>,
    raising_txg: u64,
    cache: &mut IndexNodeCache,
) -> BTreeMap<(u64, u64), RingSlotOfTheImageSeenFromTheRaise> {
    root_ring_slot_readings
        .iter()
        .map(|((region, slot), reading)| {
            let seen = match reading {
                RootRingSlotReading::AllZero => RingSlotOfTheImageSeenFromTheRaise::AllZero,
                RootRingSlotReading::DoesNotReadBack => {
                    RingSlotOfTheImageSeenFromTheRaise::DoesNotReadBack
                }
                RootRingSlotReading::SelfVerified { .. } => {
                    match roots.iter().find(|(root_region, root_slot, _)| {
                        (*root_region, *root_slot) == (*region, *slot)
                    }) {
                        // 自证过却不在择根读的那一组里：照读不出办，不猜。
                        None => RingSlotOfTheImageSeenFromTheRaise::DoesNotReadBack,
                        Some((_, _, root)) if root.checkpoint_txg >= raising_txg => {
                            RingSlotOfTheImageSeenFromTheRaise::OverwrittenSinceTheRaise
                        }
                        Some((_, _, root)) => {
                            // 下标在范围内不是这里判的：`geometry_of` 已经把 R > 3 的槽拒掉，读数的区域号都来自 `root_slot_positions`。
                            let device = geometry.region_devices[usize::try_from(*region).expect(
                                "区域号：geometry_of 判过 R ≤ REGION_DEVICE_FIELDS_IN_THE_SYSTEM_CONFIGURATION",
                            )];
                            RingSlotOfTheImageSeenFromTheRaise::HoldsARootOlderThanTheRaise(
                                RootBeforeTheRaise {
                                    device,
                                    checkpoint_txg: root.checkpoint_txg,
                                    instance: root.instance,
                                    user_visible_tree_root_pointers: user_visible_tree_root_pointers(
                                        reader,
                                        &root.record_bytes,
                                        cache,
                                    ),
                                },
                            )
                        }
                    }
                }
            };
            ((*region, *slot), seen)
        })
        .collect()
}

/// 重放读的那一份环（`records_the_replay_reads`，[`records_as_the_recovery_reads_them`]）里 checkpoint_txg 落在 `txgs` 里、带提交标记的记录，
/// 按 txg 认出几版（每个实例一版；新根段取带末条标志的那一条的，读不到末条时取先读到的那一条，同 [`versions_applied_only_by_records`]），
/// 读它新根段里那张树表的用户可见根指针。
fn versions_named_by_the_journal_at_the_txgs(
    reader: &dyn ImageReader,
    records_the_replay_reads: &BTreeMap<(u32, u64), &ScannedJournalRecord>,
    txgs: &BTreeSet<u64>,
    cache: &mut IndexNodeCache,
) -> BTreeMap<u64, Vec<VersionNamedByTheJournal>> {
    let mut segments: BTreeMap<(u64, u32), SegmentTakenForTheVersion> = BTreeMap::new();
    for record in records_the_replay_reads.values() {
        if record.commit_marker != CommitMarker::Present || !txgs.contains(&record.checkpoint_txg) {
            continue;
        }
        let carries_the_last_record_flag =
            record.record_flags_byte & RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH != 0;
        let takes_this_record = match segments.get(&(record.checkpoint_txg, record.instance)) {
            None => true,
            Some(SegmentTakenForTheVersion::FromTheLastRecordOfThePublish(_)) => false,
            Some(SegmentTakenForTheVersion::FromTheFirstRecordReadBecauseNoLastRecordWasSeen(
                _,
            )) => carries_the_last_record_flag,
        };
        if takes_this_record {
            let new_root_segment = record.new_root_segment.clone();
            segments.insert(
                (record.checkpoint_txg, record.instance),
                if carries_the_last_record_flag {
                    SegmentTakenForTheVersion::FromTheLastRecordOfThePublish(new_root_segment)
                } else {
                    SegmentTakenForTheVersion::FromTheFirstRecordReadBecauseNoLastRecordWasSeen(
                        new_root_segment,
                    )
                },
            );
        }
    }
    let mut versions: BTreeMap<u64, Vec<VersionNamedByTheJournal>> = BTreeMap::new();
    for ((checkpoint_txg, instance), taken) in segments {
        let new_root_segment = match taken {
            SegmentTakenForTheVersion::FromTheLastRecordOfThePublish(new_root_segment)
            | SegmentTakenForTheVersion::FromTheFirstRecordReadBecauseNoLastRecordWasSeen(
                new_root_segment,
            ) => new_root_segment,
        };
        versions
            .entry(checkpoint_txg)
            .or_default()
            .push(VersionNamedByTheJournal {
                instance,
                user_visible_tree_root_pointers:
                    user_visible_tree_root_pointers_through_the_tree_table_pointer(
                        reader,
                        &new_root_segment[NEW_ROOT_SEGMENT_TREE_TABLE_POINTER],
                        cache,
                    ),
            });
    }
    versions
}

/// 丁的外壳（[`rollback_floor_against_the_ring_reconstructed_at_the_raise`] 是判法）：读抬 F 那条根自己的实例表、镜像上每个根环槽的样子，
/// 只为在镜像上已被盖掉的那几槽要认的 txg 读 journal 认出的版本与它们的树表，再交给判法。抬 F 那条根自己的实例表读不出 ⇒ 环重建不出。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是一样输入：读镜像的、几何、根环、根环槽读数、重放读的那一份环、抬 F 那条根、抬之前的 F、节点缓存"
)]
fn rollback_floor_against_the_journal_reconstruction_of_the_ring_at_the_raise(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
    roots: &[(u64, u64, crate::RootView)],
    root_ring_slot_readings: &BTreeMap<(u64, u64), RootRingSlotReading>,
    records_the_replay_reads: &BTreeMap<(u32, u64), &ScannedJournalRecord>,
    raising_root: &crate::RootView,
    floor_before_the_raise: u64,
    cache: &mut IndexNodeCache,
) -> RollbackFloorAgainstTheRingAtTheRaise {
    let Some(instance_table_rows_of_the_raising_root) =
        instance_table_rows_of_root_without_judging(reader, &raising_root.record_bytes)
    else {
        return RollbackFloorAgainstTheRingAtTheRaise::RingNotReconstructed(
            "抬 F 那条根自己指着的实例表读不出".to_string(),
        );
    };
    let ring_slots_of_the_image = ring_slots_of_the_image_seen_from_the_raise(
        reader,
        geometry,
        roots,
        root_ring_slot_readings,
        raising_root.checkpoint_txg,
        cache,
    );
    // 被盖那几槽往下逐层认（[`possible_roots_in_the_ring_slot_at_the_raise`]）认到的每一层都要在 journal 里有带提交标记的记录：
    // 只读这几槽落点式住得进、比抬 F 那条根小、不低于抬之前的 F、journal 里有带提交标记的记录的那几版（不多读别的版本的树表）。
    let overwritten_slots: BTreeSet<(u64, u64)> = ring_slots_of_the_image
        .iter()
        .filter(|(_, seen)| **seen == RingSlotOfTheImageSeenFromTheRaise::OverwrittenSinceTheRaise)
        .map(|(region_and_slot, _)| *region_and_slot)
        .collect();
    let txgs_held_by_the_overwritten_slots: BTreeSet<u64> = records_the_replay_reads
        .values()
        .filter(|record| {
            record.commit_marker == CommitMarker::Present
                && record.checkpoint_txg > 0
                && record.checkpoint_txg < raising_root.checkpoint_txg
                && record.checkpoint_txg >= floor_before_the_raise
                && overwritten_slots.contains(&root_ring_slot_the_txg_names(
                    record.checkpoint_txg,
                    geometry,
                ))
        })
        .map(|record| record.checkpoint_txg)
        .collect();
    let versions_named_by_the_journal = versions_named_by_the_journal_at_the_txgs(
        reader,
        records_the_replay_reads,
        &txgs_held_by_the_overwritten_slots,
        cache,
    );
    rollback_floor_against_the_ring_reconstructed_at_the_raise(
        geometry,
        &ring_slots_of_the_image,
        &versions_named_by_the_journal,
        (raising_root.checkpoint_txg, raising_root.instance),
        &instance_table_rows_of_the_raising_root,
        floor_before_the_raise,
        raising_root.rollback_floor,
    )
}

/// I-7.9（回退下界 F 不高于抬 F 的上限）：只判**抬 F 的那一条根**，拿它之前的根算上限（
/// 里程碑「覆盖写、释放、回退与复用」收口表第 26 行；定义是实二报告 `impl-m2-checker2` 第八节那一种）。
///
/// 抬 F 的根：同一实例里 txg 比它小的最新那条根（它的前一条）带的 F 比它带的低。回退那次与新实例的第一条根没有同实例的前一条，
/// 不算抬——它带的是恢复算出来的 F_生效，而按它自己的实例表，当初撑起那个 F 的非空根已在被抛弃的时间线上，拿它算上限会在合法的
/// 「抬 F 之后回退到候选集里的根」上判红（固定用例 `rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version`）。
/// 一次抬 F 推几次空发布、每次都带新 F 时，只有第一条是抬 F 的根（后几条的前一条已带新 F）。
///
/// 上限只用根环里 txg 比它小、落在锚在镜像最新根上的一圈窗口里的根算（[`rollback_floor_ceiling_before_the_raise`]），
/// 不在后来每张镜像上重算：抬之后再回退，后来的根会把当初撑着 F 的非空根判成无效，而 F 不回落。
/// ⚠️ 后来的镜像上当初撑着上限的根可以已被盖掉：环按 txg 从旧到新被盖时，盖掉的是一段最旧的前缀，重算只会更宽；环上有洞时不是这样——
/// 洞那一槽留着上一圈的根，比它新的根反倒先被盖掉，按 [`rollback_floor_ceiling_from`] 的单调性重算出来的上限变低、在合法镜像上判红
/// （inject-6 种子 1 那一形）。窗口只锚在被判那条根上挡不住它：洞里那条旧根 t − 3S（t 是洞的 txg、t 晚于抬 F 那条根）
/// 落在那个窗口里。按这张镜像的最新根 N 取一圈窗口就挡住了：洞里留下的根都 ≤ N − 3S；而窗口里 txg 比抬 F 那条根小的根在抬 F 那一刻
/// 就在环里、之后不会被盖（盖它要一条 txg ≥ 它 + 3S > N 的根），所以留下的是抬 F 那一刻那批根去掉最旧的一段前缀，重算只会更宽。
/// 抬 F 那一刻的上限取环里全部有效根（RGall）时同一个道理成立：那一刻环里的根（窗口外洞里的也算）去掉 ≤ N − 3S 的那一段，仍是去掉最旧的一段前缀。
/// 带卸载记号那一支同一个机理：洞里那条旧根顶上来、比它新的前一条已被盖掉，「txg 比它小的最新那条根」变旧、在合法镜像上判红
/// （随机历史种子 31045、1037 那两段，9-6 报告第六节 Q1）；两支都取同一组根（
/// 同日 RGall 之后锚在被判那条根上的那个窗口被锚在镜像最新根上的罩住，只留后者）。
///
/// 按这条根 flags 里的卸载记号分两支（D22（单元原子性怎么合成） 已定项 7，(b)+(c) 合用）：
/// 带记号的（正常卸载那一串）只判 F 不高于它之前的根（锚在镜像最新根上的一圈窗口里）里 txg 最大那条的 txg，一条都没有不判；不带的（准入抬 F）照下面的准入上限判。
///
/// 不带记号那一支三种结局：F ≤ 上限的下沿 ⇒ 成立；F > 上限的上沿 ⇒ 违例；落在两沿之间、或上限无从算起 ⇒ 这条根不判。
/// 窗口里判成立只在窗口定得住抬 F 那一刻的上限时作数（[`window_lowest_ceiling_is_the_lowest_ceiling_of_the_ring_at_the_raise`]）；定不住时
/// 窗口外的旧根（洞里留着的、抬 F 之后被盖掉的）可以把那一刻的上限压低，F 越过它们窗口看不见（C717（回退下界上限的单窗口盲带））：
/// 这时按 journal 重建抬 F 那一刻的根环再判（丁；[`rollback_floor_against_the_journal_reconstruction_of_the_ring_at_the_raise`]），
/// 重建出的环判成立、违例照报（被盖那几槽的住户落没落盘判不了时可能的环逐种判，判得一致才报）；重建不出（journal 罩不到、槽读不出、
/// 可能的环多于上界）、可能的环判得不一致或 F 落在两沿之间报部分没判，不报成立也不报红。
/// 带卸载记号那一支没有这条盲带：窗口里 txg 比它小的根是抬 F 那一刻环里的根的子集，取最大只会更低、判得更严。
/// 不带记号那一支先看镜像上读回全零的根环槽（C961（全零根槽没并进 checker 的抬 F 上限），D16（发布语义） 已定项 1「抬 F 的上限」那一行：
/// 读回全零的槽并进上限，[`rollback_floor_ceiling_held_by_the_ring_slots_reading_back_all_zero`]）：F 越过其中最低的那个就判红，不越过时照上面判。
/// 一条根都没判到时整条报不适用并带理由，不报成立。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是一样输入：读镜像的、几何、根环、一圈窗口、根环槽读数与重放读的那一份环（丁重建抬 F 那一刻的根环读它们）、节点缓存、判定"
)]
pub(super) fn judge_rollback_floor_raises_against_their_ceilings(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
    roots: &[(u64, u64, crate::RootView)],
    window_anchored_at_the_newest_root_of_the_image: OneRevolutionWindow,
    root_ring_slot_readings: &BTreeMap<(u64, u64), RootRingSlotReading>,
    records_the_replay_reads: &BTreeMap<(u32, u64), &ScannedJournalRecord>,
    cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    let mut raising_roots_judged = 0u64;
    let mut raising_roots_not_judged = 0u64;
    for (_, _, raising_root) in roots {
        let Some(previous_root_of_the_same_instance) = roots
            .iter()
            .map(|(_, _, root)| root)
            .filter(|root| {
                root.instance == raising_root.instance
                    && root.checkpoint_txg < raising_root.checkpoint_txg
            })
            .max_by_key(|root| root.checkpoint_txg)
        else {
            continue;
        };
        let floor_before_the_raise = previous_root_of_the_same_instance.rollback_floor;
        if raising_root.rollback_floor <= floor_before_the_raise {
            continue;
        }
        let (raising_instance, raising_txg) = (raising_root.instance, raising_root.checkpoint_txg);
        let raised_floor = raising_root.rollback_floor;
        match raising_root.unmount_marker {
            // 带卸载记号那一支：正常卸载把 F 抬到现行那一版的 txg、不另判上限（D16（发布语义） 已定项 1「抬 F 的上限」），
            // 判 F 不高于它之前的根里 txg 最大的那条的 txg。「之前的根」与准入那一支取同一组：txg 比它小、落在锚在这张镜像最新根上的
            // 一圈窗口里；
            // 一条都没有（镜像比它新出一圈以上、它前面那几条已被盖掉）这条根不判。记号是不是只由卸载入口打，盘上没有别的东西核它，归崩溃点重放（C557）。
            crate::UnmountMarkerView::WrittenByTheUnmountSequence => {
                let Some(newest_root_txg_before_the_raise) = roots
                    .iter()
                    .map(|(_, _, root)| root.checkpoint_txg)
                    .filter(|txg| {
                        *txg < raising_txg
                            && window_anchored_at_the_newest_root_of_the_image.contains(*txg)
                    })
                    .max()
                else {
                    raising_roots_not_judged += 1;
                    continue;
                };
                raising_roots_judged += 1;
                judgements.judge(
                    "I-7.9",
                    raised_floor <= newest_root_txg_before_the_raise,
                    || {
                        format!(
                            "实例 {raising_instance} txg {raising_txg} 那条带卸载记号的根把回退下界 F 从 {floor_before_the_raise} 抬到 {raised_floor}，高于根环里 txg 比它小的最新那条根的 txg {newest_root_txg_before_the_raise}"
                        )
                    },
                );
                continue;
            }
            crate::UnmountMarkerView::NotWrittenByTheUnmountSequence => {}
        }
        // 读回全零的根环槽并进上限（C961（全零根槽没并进 checker 的抬 F 上限），D16（发布语义） 已定项 1「抬 F 的上限」那一行）：
        // F 越过它就判红，不再往下看窗口与按 journal 重建的环（那两样算出的上限只会被它再压低）；不越过它时它改不了下面任何一种结局。
        let oldest_txg_of_the_raising_instance_in_the_ring = roots
            .iter()
            .map(|(_, _, root)| root)
            .filter(|root| root.instance == raising_instance)
            .map(|root| root.checkpoint_txg)
            .min()
            .unwrap_or(raising_txg);
        if let Some((ceiling_held_by_the_all_zero_slot, (all_zero_region, all_zero_slot))) =
            rollback_floor_ceiling_held_by_the_ring_slots_reading_back_all_zero(
                geometry,
                root_ring_slot_readings,
                raising_txg,
                oldest_txg_of_the_raising_instance_in_the_ring,
                floor_before_the_raise,
            )
        {
            if raised_floor > ceiling_held_by_the_all_zero_slot {
                raising_roots_judged += 1;
                judgements.judge("I-7.9", false, || {
                    format!(
                        "实例 {raising_instance} txg {raising_txg} 那条根把回退下界 F 从 {floor_before_the_raise} 抬到 {raised_floor}，\
                         高于读回全零的根环槽（区域 {all_zero_region}、槽 {all_zero_slot}）能住的最大 txg {ceiling_held_by_the_all_zero_slot}：\
                         读回全零的槽当作可能是一条非空有效根，F 不越过它"
                    )
                });
                continue;
            }
        }
        let Some(ceiling) = rollback_floor_ceiling_before_the_raise(
            reader,
            geometry,
            roots,
            raising_root,
            floor_before_the_raise,
            window_anchored_at_the_newest_root_of_the_image,
            cache,
        ) else {
            raising_roots_not_judged += 1;
            continue;
        };
        if raised_floor > ceiling.lowest_possible && raised_floor <= ceiling.highest_possible {
            raising_roots_not_judged += 1;
            continue;
        }
        if raised_floor <= ceiling.lowest_possible
            && !window_lowest_ceiling_is_the_lowest_ceiling_of_the_ring_at_the_raise(
                &ceiling, geometry,
            )
        {
            // 窗口里判成立而窗口定不住上限（C717（回退下界上限的单窗口盲带））：丁，按 journal 重建抬 F 那一刻的根环再判。
            match rollback_floor_against_the_journal_reconstruction_of_the_ring_at_the_raise(
                reader,
                geometry,
                roots,
                root_ring_slot_readings,
                records_the_replay_reads,
                raising_root,
                floor_before_the_raise,
                cache,
            ) {
                RollbackFloorAgainstTheRingAtTheRaise::Holds => {
                    raising_roots_judged += 1;
                    judgements.judge("I-7.9", true, String::new);
                }
                RollbackFloorAgainstTheRingAtTheRaise::Violated(
                    ceiling_of_the_ring_at_the_raise,
                ) => {
                    raising_roots_judged += 1;
                    judgements.judge("I-7.9", false, || {
                        format!(
                            "实例 {raising_instance} txg {raising_txg} 那条根把回退下界 F 从 {floor_before_the_raise} 抬到 {raised_floor}，高于{ceiling_of_the_ring_at_the_raise}（锚在镜像最新根上的一圈窗口里只算得出 {}）",
                            ceiling.lowest_possible
                        )
                    });
                }
                RollbackFloorAgainstTheRingAtTheRaise::CeilingBetweenItsBounds => {
                    raising_roots_not_judged += 1;
                    judgements.part_of_the_range_left_unjudged(
                        "I-7.9",
                        format!(
                            "实例 {raising_instance} txg {raising_txg} 那条抬 F 的根：窗口里的根定不住上限，按 journal 重建的抬 F 那一刻的根环里有树表读不出的有效根，F {raised_floor} 落在上限两沿之间"
                        ),
                    );
                }
                RollbackFloorAgainstTheRingAtTheRaise::RingNotReconstructed(stuck_at) => {
                    raising_roots_not_judged += 1;
                    judgements.part_of_the_range_left_unjudged(
                        "I-7.9",
                        format!(
                            "实例 {raising_instance} txg {raising_txg} 那条抬 F 的根：窗口里的根定不住上限，journal 罩不到抬 F 那一刻的根环（{stuck_at}）"
                        ),
                    );
                }
                RollbackFloorAgainstTheRingAtTheRaise::PossibleRingsDisagree(possible_rings) => {
                    raising_roots_not_judged += 1;
                    judgements.part_of_the_range_left_unjudged(
                        "I-7.9",
                        format!(
                            "实例 {raising_instance} txg {raising_txg} 那条抬 F 的根：窗口里的根定不住上限，按 journal 重建的抬 F 那一刻的根环判得不一致（{possible_rings}）"
                        ),
                    );
                }
            }
            continue;
        }
        raising_roots_judged += 1;
        judgements.judge("I-7.9", raised_floor <= ceiling.lowest_possible, || {
            format!(
                "实例 {raising_instance} txg {raising_txg} 那条根把回退下界 F 从 {floor_before_the_raise} 抬到 {raised_floor}，高于它之前的根算出的抬 F 上限 {}：每块盘上最新的有效根 {:?}，非空有效根 {:?}，空不空判不了的有效根 {:?}，最旧有效根 txg {}",
                ceiling.highest_possible,
                ceiling.newest_valid_root_per_device,
                ceiling.non_empty_valid_root_txgs,
                ceiling.valid_root_txgs_whose_emptiness_is_undeterminable,
                ceiling.oldest_valid_root_txg
            )
        });
    }
    if raising_roots_judged == 0 {
        judgements.not_applicable(
            "I-7.9",
            if raising_roots_not_judged == 0 {
                "根环里没有抬 F 的根：每条根带的 F 都不高于同一实例里它前一条根带的（回退与新实例的第一条根不算抬）"
            } else {
                "根环里抬 F 的根都判不了：它自己指着的实例表读不出、它之前一条有效根都没有，或有效根的树表读不出、上限落在它带的 F 两边，或带卸载记号、锚在镜像最新根上的一圈窗口里没有它之前的根"
            },
        );
    }
}

/// I-7.12（系统配置 F 不低于同盘根上的 F）：每块盘分开判——这块盘两槽里自证过（整槽校验和过且 fsid 与本池相同）的系统配置槽中
/// 回退下界 F 的最大值，不低于这块盘根槽里每一条自证过的根带的 F（按 SysPre 的写序推的）。
/// 根住哪块盘按它所在区域的设备身份认（`PoolGeometry::region_devices`）。每块有自证过的系统配置槽的盘判一格（这块盘上一条根都没有时
/// 那一格照样成立：没有要比的根）；两槽都自证不过的盘报「不适用」、不判。一块盘都判不了时整条报不适用并带理由。
pub(super) fn judge_system_configuration_floor_against_the_roots_on_each_device(
    verified_system_configurations: &[(u32, Vec<crate::SystemConfigurationView>)],
    roots: &[(u64, u64, crate::RootView)],
    geometry: &PoolGeometry,
    judgements: &mut Judgements,
) {
    let mut devices_judged = 0u64;
    for (device, slots) in verified_system_configurations {
        let Some(highest_system_configuration_floor) =
            slots.iter().map(|view| view.rollback_floor).max()
        else {
            continue;
        };
        devices_judged += 1;
        let highest_floor_root_on_this_device = roots
            .iter()
            .filter(|(region, _, _)| {
                // 下标在范围内不是这里判的：`geometry_of` 已经把 R > 3 的槽拒掉，`roots` 的区域号都来自 `root_slot_positions`。
                geometry.region_devices[usize::try_from(*region).expect(
                    "区域号：geometry_of 判过 R ≤ REGION_DEVICE_FIELDS_IN_THE_SYSTEM_CONFIGURATION",
                )] == *device
            })
            .map(|(_, _, root)| root)
            .max_by_key(|root| root.rollback_floor);
        let holds = highest_floor_root_on_this_device
            .is_none_or(|root| root.rollback_floor <= highest_system_configuration_floor);
        judgements.judge("I-7.12", holds, || {
            let root = highest_floor_root_on_this_device.expect("判红只在这块盘上有根时");
            format!(
                "盘 {device}：两槽里自证过的系统配置带的 F 最大 {highest_system_configuration_floor}，低于这块盘上实例 {} txg {} 那条根带的 F {}",
                root.instance, root.checkpoint_txg, root.rollback_floor
            )
        });
    }
    if devices_judged == 0 {
        judgements.not_applicable(
            "I-7.12",
            "池里没有一块盘有自证过、fsid 与本池相同的系统配置槽：没有系统配置里的 F 可比",
        );
    }
}

#[cfg(test)]
mod tests;
