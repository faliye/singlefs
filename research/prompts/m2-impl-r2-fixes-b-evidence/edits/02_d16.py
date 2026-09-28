from edit_lib import edit, replace_everywhere

RC = "crates/singlefs-core/src/recovery.rs"
MT = "crates/singlefs-core/src/mount.rs"
UA = "crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs"
for path in (RC, MT, UA):
    replace_everywhere(path, "RootRingSlotKnownToHoldARootStillUnreadableAfterOneReread", "RootRingSlotKnownToHoldARootStillBadAfterOneReread")

edit(RC, """/// 读根环时，这个进程知道住着一条根的一个槽（挂载那一刻读得出、或这个进程写过且 FUA 返回过，
/// `allocator::RootRingOccupancy::ring_slots_known_to_hold_a_root`）这一次读不出（[`BadRootRingSlotReading::Unreadable`]），
/// 重读一次仍读不出（[`readable_roots_rereading_ring_slots_known_to_hold_a_root_once`]；D16（发布语义） 已定项 1「根槽这一次读坏」那一行的
/// 两类分法，用户 2026-09-26 定）：那一槽里的根在不在判不了，不按有根或没根猜。挂着时抬 F 与管理员回退拒这一次，经
/// `mount::StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot` 交出。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RootRingSlotKnownToHoldARootStillBadAfterOneReread {
    pub ring_slot: RootRingSlot,
}""", """/// 读根环时，这个进程知道住着一条根的一个槽（挂载那一刻读得出、或这个进程写过且 FUA 返回过，
/// `allocator::RootRingOccupancy::ring_slots_known_to_hold_a_root`）这一次读坏（`first_reading`：读不出或自证不过），
/// 重读一次仍坏（`reread`；[`readable_roots_rereading_ring_slots_known_to_hold_a_root_once`]；D16（发布语义） 已定项 1「根槽这一次读坏」那一行
/// 全句，用户 2026-09-26 定）：那一槽里的根在不在判不了，不按有根或没根猜。挂着时抬 F 与管理员回退拒这一次，经
/// `mount::StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot` 交出。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RootRingSlotKnownToHoldARootStillBadAfterOneReread {
    pub ring_slot: RootRingSlot,
    pub first_reading: BadRootRingSlotReading,
    pub reread: BadRootRingSlotReading,
}""")
edit(RC, """/// 读根环，读不出的槽按这个进程知不知道那里住着一条根分两类（D16（发布语义） 已定项 1「根槽这一次读坏」那一行的分法，用户 2026-09-26 定；
/// 与 `mount::rollback_floor_ceiling` 读根环同一个分法）：每个槽读一次；读不出（[`BadRootRingSlotReading::Unreadable`]：设备报错、越界）的槽
/// 不在 `ring_slots_known_to_hold_a_root` 里（挂载那一刻就读不出或自证不过、这个进程之后也没写过）的当没有根、不重读；在里面的
/// （挂载那一刻读得出、或这个进程写过且 FUA 返回过，`allocator::RootRingOccupancy::ring_slots_known_to_hold_a_root`）立即重读一次
/// （C554 乙的形态，R = 1），重读读得出就照它算，仍读不出就报错，不按有根或没根猜。读得出、自证不过的槽（第一遍或重读那一遍）照旧当没有根，
/// 不分类、不重读：合入后验证一的规格只要「读不出」这一支（D16 那一行另写的「或自证不过」没接进这两处读，带 F 的根被改坏、F 由系统配置撑着的
/// 那一形——`second_transaction_step_five_reuse.rs` 的 `floor_carried_by_only_one_device_root_and_the_system_configuration_…`——
/// 管理员回退照旧判到「低于 F_生效」）。交回读得出的每一条根连同它的槽，按根环槽的次序。""", """/// 读根环，读坏的槽按这个进程知不知道那里住着一条根分两类（D16（发布语义） 已定项 1「根槽这一次读坏」那一行全句，用户 2026-09-26 定；
/// 与 `mount::rollback_floor_ceiling` 读根环同一个分法）：每个槽读一次；读坏（读不出 [`BadRootRingSlotReading::Unreadable`]：设备报错、越界；
/// 或自证不过 [`BadRootRingSlotReading::NotSelfVerified`]）的槽不在 `ring_slots_known_to_hold_a_root` 里（挂载那一刻就读不出或自证不过、
/// 这个进程之后也没写过）的当没有根、不重读；在里面的（挂载那一刻读得出、或这个进程写过且 FUA 返回过，
/// `allocator::RootRingOccupancy::ring_slots_known_to_hold_a_root`）立即重读一次（C554 乙的形态，R = 1），重读自证得过就照它算，
/// 仍坏（读不出或自证不过）就报错，不按有根或没根猜。交回读得出的每一条根连同它的槽，按根环槽的次序。""")
edit(RC, """/// 知道住着根的槽重读仍读不出 ⇒ [`RootRingSlotKnownToHoldARootStillBadAfterOneReread`]（按 [`every_root_ring_slot`] 的次序第一个）。""", """/// 知道住着根的槽重读仍坏 ⇒ [`RootRingSlotKnownToHoldARootStillBadAfterOneReread`]（按 [`every_root_ring_slot`] 的次序第一个）。""")
edit(RC, """    // 迭代次数的上界是根环槽数 R × S；跨轮只带已认出的根。提前出口只有「知道住着根的槽重读仍读不出」一个。
    for ring_slot in every_root_ring_slot(immutable_sizes) {
        match read_the_slot(ring_slot) {
            RootRingSlotReading::SelfVerified(root) => {
                roots.push((ring_slot, root));
                continue;
            }
            RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified) => continue,
            RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable) => {}
        }
        if !ring_slots_known_to_hold_a_root.contains(&ring_slot) {
            continue;
        }
        match read_the_slot(ring_slot) {
            RootRingSlotReading::SelfVerified(root) => roots.push((ring_slot, root)),
            RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified) => {}
            RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable) => {
                return Err(RootRingSlotKnownToHoldARootStillBadAfterOneReread {
                    ring_slot,
                });
            }
        }
    }""", """    // 迭代次数的上界是根环槽数 R × S；跨轮只带已认出的根。提前出口只有「知道住着根的槽重读仍坏」一个。
    for ring_slot in every_root_ring_slot(immutable_sizes) {
        let first_reading = match read_the_slot(ring_slot) {
            RootRingSlotReading::SelfVerified(root) => {
                roots.push((ring_slot, root));
                continue;
            }
            RootRingSlotReading::Bad(first_reading) => first_reading,
        };
        if !ring_slots_known_to_hold_a_root.contains(&ring_slot) {
            continue;
        }
        match read_the_slot(ring_slot) {
            RootRingSlotReading::SelfVerified(root) => roots.push((ring_slot, root)),
            RootRingSlotReading::Bad(reread) => {
                return Err(RootRingSlotKnownToHoldARootStillBadAfterOneReread {
                    ring_slot,
                    first_reading,
                    reread,
                });
            }
        }
    }""")
edit(RC, """/// 知道住着根的槽重读仍读不出 ⇒ [`EffectiveFloorReadingStillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot`]；""", """/// 知道住着根的槽重读仍坏 ⇒ [`EffectiveFloorReadingStillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot`]；""")
edit(MT, """    /// 判候选、算 F_生效 读根环：`ring_slot` 是这个进程知道住着一条根的槽（挂载那一刻读得出、或这个进程写过且 FUA 返回过），这一次读不出，
    /// 重读一次仍读不出——那一槽里的根在不在判不了，不按有根或没根猜（D16（发布语义） 已定项 1「根槽这一次读坏」那一行的分法，
    /// 用户 2026-09-26 定）。挂载那一刻就读坏、之后没写过的槽不走这一条，当没有根。
    /// 实审 A3b 报告 Q4：与乙那一族同级，改之前是 `RecoveryFailure::RootRingSlotStillUnreadableAfterOneReread`（那时不分槽知不知道住着根）。
    RootRingSlotKnownToHoldARoot { ring_slot: RootRingSlot },
}""", """    /// 判候选、算 F_生效 读根环：`ring_slot` 是这个进程知道住着一条根的槽（挂载那一刻读得出、或这个进程写过且 FUA 返回过），这一次读坏
    /// （`first_reading`：读不出或自证不过），重读一次仍坏（`reread`）——那一槽里的根在不在判不了，不按有根或没根猜（D16（发布语义） 已定项 1
    /// 「根槽这一次读坏」那一行全句，用户 2026-09-26 定）。挂载那一刻就读坏、之后没写过的槽不走这一条，当没有根。
    /// 实审 A3b 报告 Q4：与乙那一族同级，改之前是 `RecoveryFailure::RootRingSlotStillUnreadableAfterOneReread`（那时不分槽知不知道住着根）。
    RootRingSlotKnownToHoldARoot {
        ring_slot: RootRingSlot,
        first_reading: BadRootRingSlotReading,
        reread: BadRootRingSlotReading,
    },
}""")
edit(MT, """    fn from(still_unreadable: RootRingSlotKnownToHoldARootStillBadAfterOneReread) -> Self {
        Self::RootRingSlotKnownToHoldARoot {
            ring_slot: still_unreadable.ring_slot,
        }""", """    fn from(still_bad: RootRingSlotKnownToHoldARootStillBadAfterOneReread) -> Self {
        Self::RootRingSlotKnownToHoldARoot {
            ring_slot: still_bad.ring_slot,
            first_reading: still_bad.first_reading,
            reread: still_bad.reread,
        }""")
print("02_d16 done")
