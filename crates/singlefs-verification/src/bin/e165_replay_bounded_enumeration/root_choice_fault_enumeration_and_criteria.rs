//! 带读故障一族一个 (域, M, U) 格里全部历史的枚举（登记 5.3 第 6 至 10 条）与判据、计数（登记 5.5）。

use super::*;

// ── 枚举一个 (域, M, U) 格里的全部历史（登记 5.3 第 6 至 10 条）

/// 走枚举的一方：收工作单元的起点，或判最后一次挂载崩溃点上的每个状态。
pub(super) trait FaultVisitor {
    /// 走到「第 `mount_number` 次挂载开始之前」（只在 `stop_before_mount` 点名的那一次挂载上触发，之后不再往下走）。
    fn reach_the_start_of_a_mount(
        &mut self,
        world: &FaultWorld,
        mount_number: usize,
        user_publishes_left: usize,
    );

    /// 最后一次挂载走完的一个状态：崩溃点已落在 `world` 上；最后一次挂载被拒时是被拒之后的盘（没有崩溃点）。
    fn visit_a_final_state(&mut self, world: &FaultWorld);
}

pub(super) struct FaultEnumeration {
    pub(super) domain: FaultDomain,
    pub(super) mounts: usize,
    pub(super) stop_before_mount: Option<usize>,
}

/// 一次往下走的挂载的几样不变的设定。
#[derive(Clone, Copy, Debug)]
pub(super) struct MountSetting {
    pub(super) mount_number: usize,
    pub(super) user_publishes_after_this_mount: usize,
    /// 这次挂载里一直读不出的落点（TP 故障）：系统配置写算世代号时不看它们。
    pub(super) unreadable_all_mount_long: FaultLocations,
    pub(super) is_the_last_mount: bool,
}

impl FaultEnumeration {
    pub(super) fn in_flight_limit(&self) -> usize {
        self.domain.geometry().ring().local_in_flight_limit()
    }

    /// 第 `mount_number` 次挂载：挂载 2 起开头先取一项故障（13 项），读阶段、闸、取号、写行与暖机的空发布、用户发布 u_m 次，崩溃点在第 u_m 次用户发布里；
    /// u_m = 0 时在写行或任一次暖机发布里。非最后一次挂载 u_m 取 0..=剩余的用户发布数，最后一次用完剩余的。`only_menu_index` 只走故障菜单的那一项。
    pub(super) fn enumerate_a_mount(
        &self,
        world: &FaultWorld,
        mount_number: usize,
        user_publishes_left: usize,
        only_menu_index: Option<usize>,
        visitor: &mut dyn FaultVisitor,
    ) {
        if self.stop_before_mount == Some(mount_number) {
            visitor.reach_the_start_of_a_mount(world, mount_number, user_publishes_left);
            return;
        }
        for entry in mount_fault_menu(world, mount_number) {
            if only_menu_index.is_some_and(|only| only != entry.menu_index) {
                continue;
            }
            self.enumerate_a_mount_under_a_fault(
                world,
                mount_number,
                user_publishes_left,
                entry,
                visitor,
            );
        }
    }

    /// 读阶段：第一遍读视图 = 盘减去故障落点，在它上面做条款的理想恢复；闸为真就用第二遍读视图（盘减去 TP 的落点）重做，仍为真拒；
    /// 往下走之前再判取号那一刻（取甲：两块盘的四个系统配置槽在 TP 视图里有一槽读不出或撕裂就拒；R2：判不出、或某块盘一份自证过的都没有就拒，
    /// 登记 r5 修订 1）。KG 关掉闸与取号那一刻（登记 5.3 第 6 条）。
    fn enumerate_a_mount_under_a_fault(
        &self,
        world: &FaultWorld,
        mount_number: usize,
        user_publishes_left: usize,
        entry: MountFaultMenuEntry,
        visitor: &mut dyn FaultVisitor,
    ) {
        let limit = self.in_flight_limit();
        let first_view = fault_landing_of_a_view(world, entry.first_read_unreadable);
        let mut landed = ideal_landed_version_of_a_view(&first_view, limit)
            .expect("停机条款 TH9：读阶段第一遍的读视图里一条根都没有");
        let mut settled_unreadable = entry.first_read_unreadable;
        let mut outcome = MountOutcome::WentDownOnTheFirstRead;
        let mut refusal: Option<MountRefusalReason> = None;
        if self.domain.gate_is_on() {
            if newer_publish_is_witnessed(world, entry.first_read_unreadable, landed) {
                let second_view = fault_landing_of_a_view(world, entry.every_read_unreadable);
                let second_landed = ideal_landed_version_of_a_view(&second_view, limit)
                    .expect("停机条款 TH9：读阶段第二遍的读视图里一条根都没有");
                if newer_publish_is_witnessed(world, entry.every_read_unreadable, second_landed) {
                    refusal = Some(MountRefusalReason::TheGateAfterTheOneReread);
                } else {
                    settled_unreadable = entry.every_read_unreadable;
                    landed = second_landed;
                    outcome = MountOutcome::WentDownOnTheSecondRead;
                }
            }
            if refusal.is_none() {
                refusal = acquisition_refusal_of(world, entry.every_read_unreadable);
            }
        }
        if let Some(reason) = refusal {
            let mut refused_world = world.clone();
            refused_world.refused_mounts += 1;
            refused_world.refusals_by_reason[reason.index()] += 1;
            refused_world.mount_log.push(MountLogEntry {
                fault: entry,
                outcome: MountOutcome::Refused,
                row_write_txg: 0,
            });
            if mount_number == self.mounts {
                visitor.visit_a_final_state(&refused_world);
            } else {
                self.enumerate_a_mount(
                    &refused_world,
                    mount_number + 1,
                    user_publishes_left,
                    None,
                    visitor,
                );
            }
            return;
        }
        self.go_down_and_write(
            world,
            (mount_number, user_publishes_left),
            (entry, outcome, settled_unreadable, landed),
            visitor,
        );
    }
}

impl FaultEnumeration {
    /// 往下走：实例代号 = 之前取过号的挂载数 + 1；写行 txg = 往下走那一遍读得出的根与（闸开时）记录的最大 txg + 1；第一条记录的计数器 = 那一遍读得出的最大计数器 + 1；
    /// 取号那一写每块盘一次系统配置写，tail = 取号那一刻读得出的槽 tail 的最大值（不取崩溃点）；之后写行、暖机、用户发布（登记 5.3 第 6 条）。
    fn go_down_and_write(
        &self,
        world: &FaultWorld,
        (mount_number, user_publishes_left): (usize, usize),
        (entry, outcome, settled_unreadable, landed): (
            MountFaultMenuEntry,
            MountOutcome,
            FaultLocations,
            RootVersion,
        ),
        visitor: &mut dyn FaultVisitor,
    ) {
        let settled_view = fault_landing_of_a_view(world, settled_unreadable);
        let highest_root_txg = settled_view
            .roots_in_ring_order
            .iter()
            .flatten()
            .map(|material| material.version.checkpoint_txg)
            .max()
            .expect("停机条款 TH9：往下走那一遍的读视图里一条根都没有");
        let highest_record_txg = settled_view
            .records
            .iter()
            .map(|record| record.material.version.checkpoint_txg)
            .max()
            .unwrap_or(0);
        let row_write_txg = if self.domain.gate_is_on() {
            highest_root_txg.max(highest_record_txg) + 1
        } else {
            highest_root_txg + 1
        };
        let first_counter = settled_view
            .records
            .iter()
            .map(counter_of_the_landed_record)
            .max()
            .unwrap_or(0)
            + 1;
        let instance = world.next_instance;
        let mut acquired = world.clone();
        let (_readable_slot_count, largest_tail_at_the_acquisition) =
            readable_system_slots_of(&world.system_slots, entry.every_read_unreadable);
        acquired.write_the_system_configuration_of_every_device(
            largest_tail_at_the_acquisition,
            instance,
            entry.every_read_unreadable,
        );
        acquired.next_instance += 1;
        if outcome == MountOutcome::WentDownOnTheSecondRead {
            acquired.reread_mounts += 1;
        }
        acquired.mount_log.push(MountLogEntry {
            fault: entry,
            outcome,
            row_write_txg,
        });
        let overhead_txgs = root_choice_warm_up_plan(row_write_txg);
        let is_the_last_mount = mount_number == self.mounts;
        let user_publish_counts: Vec<usize> = if is_the_last_mount {
            vec![user_publishes_left]
        } else {
            (0..=user_publishes_left).collect()
        };
        for user_publish_count in user_publish_counts {
            let setting = MountSetting {
                mount_number,
                user_publishes_after_this_mount: user_publishes_left - user_publish_count,
                unreadable_all_mount_long: entry.every_read_unreadable,
                is_the_last_mount,
            };
            let mut running = acquired.clone();
            let mut cursor = RootChoiceMountCursor {
                instance,
                next_counter: first_counter,
                next_transaction: 1,
                previous_chain_value: None,
                next_checkpoint_txg: overhead_txgs[0],
            };
            let mut previous_version = landed;
            for overhead_txg in &overhead_txgs {
                assert_eq!(
                    cursor.next_checkpoint_txg, *overhead_txg,
                    "开销发布的 txg 逐个加一"
                );
                let material = Arc::new(root_choice_publish_material(
                    &cursor.plan_of_the_next_publish(
                        self.domain.geometry(),
                        running.publishes_started(),
                        1,
                        0,
                        false,
                    ),
                ));
                running.register_a_started_publish(&material, previous_version);
                if user_publish_count == 0 {
                    self.enumerate_the_crash_points(&running, &material, setting, visitor);
                }
                running.apply_a_publish_in_full(
                    &material,
                    false,
                    setting.unreadable_all_mount_long,
                );
                cursor = cursor.after_the_publish_of(&material);
                previous_version = material.version;
            }
            if user_publish_count > 0 {
                self.enumerate_user_publishes(
                    &running,
                    cursor,
                    (previous_version, user_publish_count),
                    setting,
                    visitor,
                );
            }
        }
    }

    /// 第 `user_publishes_in_the_mount`（≥ 1）次用户发布起，每次的 k 与提交标记摆法各一种；崩溃点在这一挂载的最后一次用户发布里。
    pub(super) fn enumerate_user_publishes(
        &self,
        running: &FaultWorld,
        cursor: RootChoiceMountCursor,
        (previous_version, user_publishes_in_the_mount): (RootVersion, usize),
        setting: MountSetting,
        visitor: &mut dyn FaultVisitor,
    ) {
        for record_count in self.domain.user_publish_record_counts() {
            for commit_marker_bits in 0..(1u32 << (record_count - 1)) {
                let material = Arc::new(root_choice_publish_material(
                    &cursor.plan_of_the_next_publish(
                        self.domain.geometry(),
                        running.publishes_started(),
                        *record_count,
                        commit_marker_bits,
                        true,
                    ),
                ));
                let mut branch = running.clone();
                branch.register_a_started_publish(&material, previous_version);
                if user_publishes_in_the_mount == 1 {
                    self.enumerate_the_crash_points(&branch, &material, setting, visitor);
                } else {
                    branch.apply_a_publish_in_full(
                        &material,
                        true,
                        setting.unreadable_all_mount_long,
                    );
                    self.enumerate_user_publishes(
                        &branch,
                        cursor.after_the_publish_of(&material),
                        (material.version, user_publishes_in_the_mount - 1),
                        setting,
                        visitor,
                    );
                }
            }
        }
    }

    /// 崩在 `material` 这次发布的每个崩溃点上：最后一次挂载交给 `visitor` 判；之前的挂载接着枚举下一次挂载。
    pub(super) fn enumerate_the_crash_points(
        &self,
        running: &FaultWorld,
        material: &Arc<RootChoicePublishMaterial>,
        setting: MountSetting,
        visitor: &mut dyn FaultVisitor,
    ) {
        for point in fault_crash_points_of(material.records.len(), setting.is_the_last_mount) {
            let crashed =
                running.with_crash_applied(material, point, setting.unreadable_all_mount_long);
            if setting.is_the_last_mount {
                visitor.visit_a_final_state(&crashed);
            } else {
                self.enumerate_a_mount(
                    &crashed,
                    setting.mount_number + 1,
                    setting.user_publishes_after_this_mount,
                    None,
                    visitor,
                );
            }
        }
    }
}

pub(super) const FAULT_CRITERION_COUNT: usize = 5;
pub(super) const KJ1_INDEX: usize = 1;
pub(super) const DESCENDS_FROM_THE_LAST_CONFIRMED_PUBLISH_INDEX: usize = 2;
pub(super) const KJ2_INDEX: usize = 3;
pub(super) const KJ3_INDEX: usize = 4;
/// 本地择根序的个数：登记 9.1 的五种加平局取后者赢（v1822），再加 KF1 用的「取键最小的根」。
pub(super) const FAULT_LOCAL_RULE_COUNT: usize = 7;
pub(super) const FAULT_OLDEST_RULE_INDEX: usize = 6;
/// 每个 (域, M, U) 格里按「被判时故障 × 历史标签」分的格数。
pub(super) const FAULT_CELL_COUNT: usize = 10;

/// 一个 (被判时故障, 历史标签) 格里数的量：状态数、有已确认的态数、上限截掉的态数，与每条判据的判数、违例数（上限截掉的态里的违例另记）。
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct FaultCellTally {
    pub(super) states: u64,
    pub(super) confirmed_states: u64,
    pub(super) cut_states: u64,
    pub(super) judged: [u64; FAULT_CRITERION_COUNT],
    pub(super) violations: [u64; FAULT_CRITERION_COUNT],
    pub(super) cut_violations: [u64; FAULT_CRITERION_COUNT],
    /// 条款的理想恢复在这一格里 KJ1、KJ1T 各违例几次（TQ3 把实现的数与它并列）。
    pub(super) ideal_violations: [u64; 2],
}

impl FaultCellTally {
    pub(super) fn merge(&mut self, later: &Self) {
        self.states += later.states;
        self.confirmed_states += later.confirmed_states;
        self.cut_states += later.cut_states;
        for criterion in 0..FAULT_CRITERION_COUNT {
            self.judged[criterion] += later.judged[criterion];
            self.violations[criterion] += later.violations[criterion];
            self.cut_violations[criterion] += later.cut_violations[criterion];
        }
        for criterion in 0..2 {
            self.ideal_violations[criterion] += later.ideal_violations[criterion];
        }
    }
}

/// 格的下标：被判时故障的次序 × 2 + 历史标签（0 罩着、1 不罩）。
pub(super) fn fault_cell_index_of(judged_fault: JudgedFault, history_is_covered: bool) -> usize {
    judged_fault.index() * 2 + usize::from(!history_is_covered)
}

/// M = 2 的格按挂载 2 开头那项故障报的三样。
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct MountTwoFaultCount {
    pub(super) histories: u64,
    pub(super) refused: u64,
    pub(super) reread: u64,
}

/// 一个 (域, M, U) 格的全部计数。违例类的量在第一次加一时记一个见证。
#[derive(Clone, Debug)]
pub(super) struct FaultTally {
    pub(super) cells: [FaultCellTally; FAULT_CELL_COUNT],
    /// 每个格每条判据的第一个见证。
    pub(super) witnesses: Vec<Option<String>>,
    /// 承诺内（历史罩着 ∧ 被判时罩着 ∧ 不是上限截掉）的态数与 KJ1、KJ1T 的违例数，装置直接数（登记 5.5 末段）。
    pub(super) promised_states: u64,
    pub(super) promised_violations: [u64; 2],
    pub(super) promised_witnesses: [Option<String>; 2],
    pub(super) histories: u64,
    pub(super) histories_with_an_uncovered_fault: u64,
    pub(super) judged_states: u64,
    pub(super) confirmed_states: u64,
    pub(super) refused_mounts: u64,
    /// 被拒的挂载按拒因分（登记 r5 修订 1 修1.五第 4 条；只在 `--gate r2` 时打进产物）。
    pub(super) refusals_by_reason: [u64; MOUNT_REFUSAL_REASON_COUNT],
    pub(super) reread_mounts: u64,
    pub(super) states_with_a_torn_system_slot: u64,
    pub(super) states_with_same_txg_different_instance_roots: u64,
    pub(super) states_with_higher_instance_lower_txg_roots: u64,
    pub(super) states_without_a_readable_root: u64,
    pub(super) above_newest_root_maximum: usize,
    pub(super) confirmed_publishes_maximum: usize,
    pub(super) largest_counter: u64,
    pub(super) ideal_loss_by_limit: u64,
    pub(super) ideal_loss_covered_other: u64,
    pub(super) ideal_loss_uncovered: u64,
    pub(super) mount_two: [MountTwoFaultCount; MOUNT_FAULT_MENU_LENGTH],
    pub(super) panicked_states: u64,
    pub(super) chose_nothing_with_a_readable_root: u64,
    pub(super) chose_nothing_without_a_readable_root: u64,
    pub(super) replay_failed_states: u64,
    pub(super) replay_applied_states: u64,
    pub(super) slot_readings_checked: u64,
    pub(super) slot_readings_differing_from_the_truth: u64,
    pub(super) slot_reading_witness: Option<String>,
    /// KJ0 的对照：在指定的一态上 panic，别的态不算。
    pub(super) kj0_control_violations: u64,
    /// 本地择根序加理想重放读到的 KJ1、KJ1T、KJ2 违例数（登记第七节「本地择根序」那张表；下标 6 是 KF1 的「取键最小的根」）。
    pub(super) local_rules: [[u64; 3]; FAULT_LOCAL_RULE_COUNT],
    /// 本地择根序的 KJ1T 违例数按被判时故障分（C331 的判别力自证：今天的序在被判时无故障的态上的合计）。
    pub(super) local_kj1t_by_judged_fault: [[u64; 5]; FAULT_LOCAL_RULE_COUNT],
    /// KF2 的 KJ1、KJ1T、KJ2、KJ3 读数；KF4 的 KJ1、KJ1T 读数（登记 5.6）。
    pub(super) kf2: [u64; 4],
    pub(super) kf4: [u64; 2],
    pub(super) kf4_states_without_a_root_off_the_confirmed_chain: u64,
    /// KJ1 与 KJ1T 对同一态判得不同的态数（TQ15）。
    pub(super) kj1_and_kj1t_differing_states: u64,
    pub(super) kj1_and_kj1t_differing_witness: Option<String>,
}

impl FaultTally {
    pub(super) fn new() -> Self {
        Self {
            cells: [FaultCellTally::default(); FAULT_CELL_COUNT],
            witnesses: vec![None; FAULT_CELL_COUNT * FAULT_CRITERION_COUNT],
            promised_states: 0,
            promised_violations: [0; 2],
            promised_witnesses: [None, None],
            histories: 0,
            histories_with_an_uncovered_fault: 0,
            judged_states: 0,
            confirmed_states: 0,
            refused_mounts: 0,
            refusals_by_reason: [0; MOUNT_REFUSAL_REASON_COUNT],
            reread_mounts: 0,
            states_with_a_torn_system_slot: 0,
            states_with_same_txg_different_instance_roots: 0,
            states_with_higher_instance_lower_txg_roots: 0,
            states_without_a_readable_root: 0,
            above_newest_root_maximum: 0,
            confirmed_publishes_maximum: 0,
            largest_counter: 0,
            ideal_loss_by_limit: 0,
            ideal_loss_covered_other: 0,
            ideal_loss_uncovered: 0,
            mount_two: [MountTwoFaultCount::default(); MOUNT_FAULT_MENU_LENGTH],
            panicked_states: 0,
            chose_nothing_with_a_readable_root: 0,
            chose_nothing_without_a_readable_root: 0,
            replay_failed_states: 0,
            replay_applied_states: 0,
            slot_readings_checked: 0,
            slot_readings_differing_from_the_truth: 0,
            slot_reading_witness: None,
            kj0_control_violations: 0,
            local_rules: [[0; 3]; FAULT_LOCAL_RULE_COUNT],
            local_kj1t_by_judged_fault: [[0; 5]; FAULT_LOCAL_RULE_COUNT],
            kf2: [0; 4],
            kf4: [0; 2],
            kf4_states_without_a_root_off_the_confirmed_chain: 0,
            kj1_and_kj1t_differing_states: 0,
            kj1_and_kj1t_differing_witness: None,
        }
    }
}

fn merge_the_first_witness(earlier: &mut Option<String>, later: &Option<String>) {
    if earlier.is_none() {
        earlier.clone_from(later);
    }
}

impl FaultTally {
    /// 违例加一，并在这个格这条判据的第一条违例上记一个见证（枚举次序里最先的那个，合并时按工作单元的次序保留最先的）。
    pub(super) fn add_cell_violation(
        &mut self,
        cell_index: usize,
        criterion_index: usize,
        describe_witness: &dyn Fn() -> String,
    ) {
        self.cells[cell_index].violations[criterion_index] += 1;
        let witness = &mut self.witnesses[cell_index * FAULT_CRITERION_COUNT + criterion_index];
        if witness.is_none() {
            *witness = Some(describe_witness());
        }
    }

    pub(super) fn witness_of_a_cell(&self, cell_index: usize, criterion_index: usize) -> &str {
        self.witnesses[cell_index * FAULT_CRITERION_COUNT + criterion_index]
            .as_deref()
            .unwrap_or("none")
    }

    /// 一条判据在全部格上的违例合计。
    pub(super) fn violations_of_a_criterion(&self, criterion_index: usize) -> u64 {
        self.cells
            .iter()
            .map(|cell| cell.violations[criterion_index])
            .sum()
    }

    pub(super) fn judged_of_a_criterion(&self, criterion_index: usize) -> u64 {
        self.cells
            .iter()
            .map(|cell| cell.judged[criterion_index])
            .sum()
    }

    pub(super) fn merge(&mut self, later: &Self) {
        for (cell, later_cell) in self.cells.iter_mut().zip(&later.cells) {
            cell.merge(later_cell);
        }
        for (witness, later_witness) in self.witnesses.iter_mut().zip(&later.witnesses) {
            merge_the_first_witness(witness, later_witness);
        }
        self.promised_states += later.promised_states;
        for criterion in 0..2 {
            self.promised_violations[criterion] += later.promised_violations[criterion];
            merge_the_first_witness(
                &mut self.promised_witnesses[criterion],
                &later.promised_witnesses[criterion],
            );
        }
        self.histories += later.histories;
        self.histories_with_an_uncovered_fault += later.histories_with_an_uncovered_fault;
        self.judged_states += later.judged_states;
        self.confirmed_states += later.confirmed_states;
        self.refused_mounts += later.refused_mounts;
        for (count, later_count) in self
            .refusals_by_reason
            .iter_mut()
            .zip(&later.refusals_by_reason)
        {
            *count += later_count;
        }
        self.reread_mounts += later.reread_mounts;
        self.states_with_a_torn_system_slot += later.states_with_a_torn_system_slot;
        self.states_with_same_txg_different_instance_roots +=
            later.states_with_same_txg_different_instance_roots;
        self.states_with_higher_instance_lower_txg_roots +=
            later.states_with_higher_instance_lower_txg_roots;
        self.states_without_a_readable_root += later.states_without_a_readable_root;
        self.above_newest_root_maximum = self
            .above_newest_root_maximum
            .max(later.above_newest_root_maximum);
        self.confirmed_publishes_maximum = self
            .confirmed_publishes_maximum
            .max(later.confirmed_publishes_maximum);
        self.largest_counter = self.largest_counter.max(later.largest_counter);
        self.ideal_loss_by_limit += later.ideal_loss_by_limit;
        self.ideal_loss_covered_other += later.ideal_loss_covered_other;
        self.ideal_loss_uncovered += later.ideal_loss_uncovered;
        for (count, later_count) in self.mount_two.iter_mut().zip(&later.mount_two) {
            count.histories += later_count.histories;
            count.refused += later_count.refused;
            count.reread += later_count.reread;
        }
        self.panicked_states += later.panicked_states;
        self.chose_nothing_with_a_readable_root += later.chose_nothing_with_a_readable_root;
        self.chose_nothing_without_a_readable_root += later.chose_nothing_without_a_readable_root;
        self.replay_failed_states += later.replay_failed_states;
        self.replay_applied_states += later.replay_applied_states;
        self.slot_readings_checked += later.slot_readings_checked;
        self.slot_readings_differing_from_the_truth += later.slot_readings_differing_from_the_truth;
        merge_the_first_witness(&mut self.slot_reading_witness, &later.slot_reading_witness);
        self.kj0_control_violations += later.kj0_control_violations;
        for rule in 0..FAULT_LOCAL_RULE_COUNT {
            for criterion in 0..3 {
                self.local_rules[rule][criterion] += later.local_rules[rule][criterion];
            }
            for fault in 0..5 {
                self.local_kj1t_by_judged_fault[rule][fault] +=
                    later.local_kj1t_by_judged_fault[rule][fault];
            }
        }
        for criterion in 0..4 {
            self.kf2[criterion] += later.kf2[criterion];
        }
        for criterion in 0..2 {
            self.kf4[criterion] += later.kf4[criterion];
        }
        self.kf4_states_without_a_root_off_the_confirmed_chain +=
            later.kf4_states_without_a_root_off_the_confirmed_chain;
        self.kj1_and_kj1t_differing_states += later.kj1_and_kj1t_differing_states;
        merge_the_first_witness(
            &mut self.kj1_and_kj1t_differing_witness,
            &later.kj1_and_kj1t_differing_witness,
        );
    }
}

/// 把工作单元分给几条线程跑，再按工作单元的次序合并（同 [`root_choice_tally_of_work_units`]，计数的种类不同）。
pub(super) fn fault_tally_of_work_units<Unit: Sync>(
    units: &[Unit],
    thread_count: usize,
    work: &(dyn Fn(usize, &Unit) -> FaultTally + Sync),
) -> FaultTally {
    let next_unit_index = AtomicUsize::new(0);
    let finished_units: Mutex<Vec<(usize, FaultTally)>> = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _worker in 0..thread_count.clamp(1, units.len().max(1)) {
            scope.spawn(|| loop {
                let unit_index = next_unit_index.fetch_add(1, Ordering::Relaxed);
                let Some(unit) = units.get(unit_index) else {
                    break;
                };
                let tally = work(unit_index, unit);
                finished_units
                    .lock()
                    .expect("别的线程没有在拿着这把锁的时候 panic")
                    .push((unit_index, tally));
            });
        }
    });
    let mut finished_units = finished_units
        .into_inner()
        .expect("线程都已经收回，没有谁在拿着这把锁的时候 panic");
    finished_units.sort_by_key(|(unit_index, _tally)| *unit_index);
    assert_eq!(finished_units.len(), units.len(), "每个工作单元恰好跑一遍");
    let mut total = FaultTally::new();
    for (_unit_index, tally) in &finished_units {
        total.merge(tally);
    }
    total
}

impl FaultLocalRule {
    /// 读根环次序里往后走，`candidate` 换掉已择的 `chosen` 吗。
    pub(super) fn replaces(self, candidate: RootVersion, chosen: RootVersion) -> bool {
        match self {
            Self::Today => {
                candidate.checkpoint_txg > chosen.checkpoint_txg
                    || (candidate.checkpoint_txg == chosen.checkpoint_txg
                        && candidate.instance > chosen.instance)
            }
            Self::IgnoresTheInstance => candidate.checkpoint_txg > chosen.checkpoint_txg,
            Self::FirstSelfVerifiedWins => false,
            Self::TieTakesTheLowerInstance => {
                candidate.checkpoint_txg > chosen.checkpoint_txg
                    || (candidate.checkpoint_txg == chosen.checkpoint_txg
                        && candidate.instance < chosen.instance)
            }
            Self::InstanceFirstThenTxg => {
                candidate.instance > chosen.instance
                    || (candidate.instance == chosen.instance
                        && candidate.checkpoint_txg > chosen.checkpoint_txg)
            }
            Self::TieTakesTheLater => {
                candidate.checkpoint_txg > chosen.checkpoint_txg
                    || (candidate.checkpoint_txg == chosen.checkpoint_txg
                        && candidate.instance >= chosen.instance)
            }
            Self::OldestWins => candidate < chosen,
        }
    }

    pub(super) fn chooses_among<'world>(
        self,
        roots_in_ring_order: &[Option<&'world RootChoicePublishMaterial>],
    ) -> Option<&'world RootChoicePublishMaterial> {
        let mut chosen: Option<&RootChoicePublishMaterial> = None;
        for candidate in roots_in_ring_order.iter().copied().flatten() {
            if chosen.is_none_or(|already_chosen| {
                self.replaces(candidate.version, already_chosen.version)
            }) {
                chosen = Some(candidate);
            }
        }
        chosen
    }
}

/// 一个被判态判之前先算好的真相（判据一侧只用这些，不调 core 的判定函数）。
pub(super) struct FaultStateFacts {
    pub(super) last_confirmed: Option<RootVersion>,
    pub(super) newest_readable: Option<RootVersion>,
    /// 条款的理想恢复（读视图里版本最大的根起）落到的六个字段。
    pub(super) ideal_fields: Option<VersionFields>,
}

/// 一条落点按各条判据读出来的结果；`Some(true)` 是违例，`Some(false)` 是判过没违例，`None` 是这一态不判这一条。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct FaultCriteriaReading {
    pub(super) kj1: Option<bool>,
    pub(super) kj1t: Option<bool>,
    pub(super) kj2: Option<bool>,
    pub(super) kj3: Option<bool>,
}

/// 落点 `landed_version` 按登记 5.5 的四条判据读：KJ1 版本序不早于最后一次已确认的发布；KJ1T 落在它本身或后代上（父链，登记 5.3 第 7 条）；
/// KJ2 不早于读视图里任一读得出的根；KJ3 六个字段等于条款的理想恢复。
pub(super) fn fault_criteria_of_a_landing(
    world: &FaultWorld,
    facts: &FaultStateFacts,
    landed_version: RootVersion,
    landed_fields: Option<VersionFields>,
) -> FaultCriteriaReading {
    FaultCriteriaReading {
        kj1: facts.last_confirmed.map(|last| landed_version < last),
        kj1t: facts
            .last_confirmed
            .map(|last| !world.is_the_version_or_a_descendant_of(landed_version, last)),
        kj2: facts.newest_readable.map(|newest| landed_version < newest),
        kj3: match (facts.ideal_fields, landed_fields) {
            (Some(ideal), Some(fields)) => Some(fields != ideal),
            (None, _) | (_, None) => None,
        },
    }
}

/// 一个被判态的结局在每条判据上的读数：真实现的结局或阳性对照的假结局都走这一段。
pub(super) fn count_a_criteria_reading_into_a_cell(
    reading: FaultCriteriaReading,
    is_cut_by_the_limit: bool,
    cell: &mut FaultCellTally,
) -> [bool; FAULT_CRITERION_COUNT] {
    let mut violated = [false; FAULT_CRITERION_COUNT];
    for (criterion_index, verdict) in [
        (KJ1_INDEX, reading.kj1),
        (DESCENDS_FROM_THE_LAST_CONFIRMED_PUBLISH_INDEX, reading.kj1t),
        (KJ2_INDEX, reading.kj2),
        (KJ3_INDEX, reading.kj3),
    ] {
        if let Some(is_violation) = verdict {
            cell.judged[criterion_index] += 1;
            if is_violation {
                cell.violations[criterion_index] += 1;
                if is_cut_by_the_limit {
                    cell.cut_violations[criterion_index] += 1;
                }
                violated[criterion_index] = true;
            }
        }
    }
    violated
}

/// 一个被判态上三步的结局（只在 `run_the_three_steps` 时算；单测里为省时可以关掉）。
pub(super) fn outcome_of_the_three_steps_on_a_faulted_disk(
    disk: &FaultedRootChoiceDisk<'_, '_>,
    system_configuration: &SystemConfiguration,
) -> RootChoiceOutcome {
    IS_INSIDE_THE_REPLAY_CALL.with(|is_inside| is_inside.set(true));
    let outcome = outcome_of_the_three_steps_with_the_panic_hook_quiet(disk, system_configuration);
    IS_INSIDE_THE_REPLAY_CALL.with(|is_inside| is_inside.set(false));
    outcome
}

pub(super) fn outcome_of_the_three_steps_with_the_panic_hook_quiet(
    disk: &FaultedRootChoiceDisk<'_, '_>,
    system_configuration: &SystemConfiguration,
) -> RootChoiceOutcome {
    let chosen_root =
        match panic::catch_unwind(AssertUnwindSafe(|| choose_root(disk, system_configuration))) {
            Err(_panic_payload) => {
                return RootChoiceOutcome::Panicked {
                    step: "choose_root",
                }
            }
            Ok(None) => return RootChoiceOutcome::ChoseNothing,
            Ok(Some(root)) => root,
        };
    let records = match panic::catch_unwind(AssertUnwindSafe(|| {
        scan_journal(disk, system_configuration)
    })) {
        Err(_panic_payload) => {
            return RootChoiceOutcome::Panicked {
                step: "scan_journal",
            }
        }
        Ok(records) => records,
    };
    let replay = panic::catch_unwind(AssertUnwindSafe(|| {
        replay_journal(
            disk,
            &chosen_root,
            system_configuration.immutable.sizes.journal_ring_bytes,
            &records,
            NamedUnitVerification::Verify,
        )
    }));
    match replay {
        Err(_panic_payload) => RootChoiceOutcome::Panicked {
            step: "replay_journal",
        },
        Ok(Err(_failure)) => RootChoiceOutcome::Landed {
            chosen: RootVersion::of_root(&chosen_root),
            landed_version: RootVersion::of_root(&chosen_root),
            landed_fields: VersionFields::of_root(&chosen_root),
            replay_failed: true,
        },
        Ok(Ok((_report, returned_root))) => RootChoiceOutcome::Landed {
            chosen: RootVersion::of_root(&chosen_root),
            landed_version: RootVersion::of_root(&returned_root),
            landed_fields: VersionFields::of_root(&returned_root),
            replay_failed: false,
        },
    }
}

/// 读槽对拍（停机 TH4 的输入，登记 5.5 末段）：被判时故障的落点必须读成 `Bad(Unreadable)`；别的槽读成自证过的，它的 (txg, 实例代号) 等于真相；
/// 全 0 的槽读成自证不过、而且真相里这个槽是空的。
fn fault_slot_reading_agrees_with_the_truth(
    reading: RootRingSlotReading,
    truth: Option<&RootChoicePublishMaterial>,
    is_unreadable_in_this_view: bool,
) -> bool {
    if is_unreadable_in_this_view {
        return matches!(
            reading,
            RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable)
        );
    }
    root_slot_reading_agrees_with_the_truth(reading, truth)
}

/// 逐槽调 `read_root_ring_slot`，与真相对拍；不一致数一笔、记第一个见证，收尾走停机条款 TH4。
pub(super) fn check_the_fault_slot_readings_against_the_truth(
    disk: &FaultedRootChoiceDisk<'_, '_>,
    system_configuration: &SystemConfiguration,
    world: &FaultWorld,
    tally: &mut FaultTally,
    describe_state: &dyn Fn() -> String,
) {
    tally.slot_readings_checked += 1;
    let immutable = &system_configuration.immutable;
    let slots_per_region = world.slots_per_region();
    for ring_slot in every_root_ring_slot(&immutable.sizes) {
        let reading = read_root_ring_slot(
            disk,
            &immutable.region_devices,
            &immutable.sizes,
            &immutable.filesystem_identifier,
            ring_slot,
        );
        let slot_index =
            root_choice_slot_index_of(ring_slot.region, ring_slot.slot, slots_per_region);
        let truth = world.root_slots[slot_index].as_deref();
        let is_unreadable = disk
            .locations
            .root_slot_is_unreadable(slot_index, slots_per_region);
        if !fault_slot_reading_agrees_with_the_truth(reading, truth, is_unreadable) {
            tally.slot_readings_differing_from_the_truth += 1;
            if tally.slot_reading_witness.is_none() {
                tally.slot_reading_witness = Some(format!(
                    "region{}_slot{}_read_as={reading:?}_truth={}_unreadable_in_the_view={is_unreadable};{}",
                    ring_slot.region,
                    ring_slot.slot,
                    truth.map_or_else(|| "empty".to_string(), |material| material.version.describe()),
                    describe_state()
                ));
            }
        }
    }
}

fn describe_a_system_slot(content: SystemSlotContent) -> String {
    match content {
        SystemSlotContent::Torn => "torn".to_string(),
        SystemSlotContent::Verified {
            generation,
            tail,
            instance,
        } => format!("g{generation}t{tail}i{instance}"),
    }
}

/// 一段历史的写法（见证用）：每次挂载开头的故障与走了哪一遍、写行 txg，开始过的每次发布，系统配置槽，最后一次的崩溃点。
pub(super) fn describe_a_fault_history(world: &FaultWorld) -> String {
    let mounts: Vec<String> = world
        .mount_log
        .iter()
        .enumerate()
        .map(|(position, entry)| {
            format!(
                "mount{}:fault={}:{:?}:row_write_txg={}",
                position + 1,
                entry.fault.label(),
                entry.outcome,
                entry.row_write_txg
            )
        })
        .collect();
    let publishes: Vec<String> = world
        .started_publishes
        .iter()
        .skip(1)
        .map(|material| {
            format!(
                "txg{}i{}k{}{}",
                material.version.checkpoint_txg,
                material.version.instance,
                material.records.len(),
                if material.is_user_publish { "u" } else { "o" }
            )
        })
        .collect();
    let system_slots: Vec<String> = world
        .system_slots
        .iter()
        .map(|slots| {
            slots
                .iter()
                .map(|content| describe_a_system_slot(*content))
                .collect::<Vec<String>>()
                .join("/")
        })
        .collect();
    let slots_per_region = usize::try_from(world.slots_per_region()).expect("每区槽数 4 或 8");
    let roots: Vec<String> = world
        .root_slots
        .iter()
        .enumerate()
        .filter_map(|(root_slot_index, root)| {
            root.as_ref().map(|material| {
                format!(
                    "region{}slot{}={}",
                    root_slot_index / slots_per_region,
                    root_slot_index % slots_per_region,
                    material.version.describe()
                )
            })
        })
        .collect();
    let confirmed: Vec<String> = world
        .confirmed_user_publishes
        .iter()
        .map(|version| version.describe())
        .collect();
    let mut chain_of_the_last_confirmed = Vec::new();
    let mut current = world.confirmed_user_publishes.iter().copied().max();
    while let Some(version) = current {
        chain_of_the_last_confirmed.push(version.describe());
        current = world.parent_of(version);
        if chain_of_the_last_confirmed.len() > world.parents.len() {
            break;
        }
    }
    format!(
        "{};publishes={};roots_in_ring_order={};confirmed={};parent_chain_of_the_last_confirmed={};system_slots_d0|d1={};{}",
        mounts.join("+"),
        publishes.join(","),
        roots.join("+"),
        confirmed.join(","),
        chain_of_the_last_confirmed.join("<-"),
        system_slots.join("|"),
        describe_a_fault_crash(world.final_crash)
    )
}

fn describe_a_fault_crash(crash: Option<FaultCrashLabel>) -> String {
    let Some(label) = crash else {
        return "no_crash_the_last_mount_was_refused".to_string();
    };
    let point_text = match label.point {
        FaultCrashPoint::RecordCopiesLanded { copy_mask } => {
            format!("record_copies_mask_{copy_mask:b}_root_not_landed")
        }
        FaultCrashPoint::RootLanded { system_writes } => {
            format!("root_landed_system_writes_{system_writes:?}")
        }
    };
    format!(
        "crash_in_publish_txg{}_instance{}_{}records_{point_text}",
        label.version.checkpoint_txg, label.version.instance, label.record_count
    )
}

#[cfg(test)]
mod root_choice_fault_tests;
