//! 崩溃注入的计数：每一截、每条路径各跑过几次、判过几个，二次崩溃摆了几个、判了几个。

use super::*;
use std::fmt::Write as _;

/// 跑过的崩溃状态的计数：绝对数，证明各条路径真的跑到了（`test-discipline.md`「阴性结果要能和「代码没跑到」分开」）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CrashInjectionTally {
    pub histories: u64,
    /// 这段历史自己怎么收尾（崩溃状态之外）。
    pub histories_completed: u64,
    pub histories_ended_known_red: BTreeMap<usize, u64>,
    pub histories_ended_new_finding: u64,
    /// 一个崩溃状态都摆不出的历史段数（流里一个候选段都没有）。
    pub histories_without_any_crash_point: u64,
    pub crash_points: u64,
    /// 按段内被扣下的写的种类分（四种落点都扣下过才算罩全）；一个崩溃状态扣下几种就各记一次。
    pub crash_points_withholding_write_kind: BTreeMap<&'static str, u64>,
    /// 按崩溃状态落在哪一类操作里分（固定脚本只有一条路，这里证明挂载、回退、抬 F、冷启动的写上都摆过）。
    pub crash_points_by_operation_kind: BTreeMap<HistoryOperationKind, u64>,
    /// 崩溃状态落在起点那一段里（这个数证明真的摆到了）。
    pub crash_points_inside_the_starting_point: u64,
    /// 段内持久的不是前缀：「后发的写先持久」那一类状态摆出来了几个。
    pub crash_points_withholding_a_write_before_a_persisted_one: u64,
    /// 截断之后盘上已持久的最新根槽，比这段历史最后提交的那一版旧：真的把状态截回了过去。
    pub crash_points_that_land_before_the_last_committed_version: u64,
    /// 一次发布已经写过东西而它的根槽还没持久。
    pub crash_points_inside_an_unfinished_publish: u64,
    pub recoveries_reading_a_file: u64,
    pub recoveries_without_a_file: u64,
    pub recoveries_failed: u64,
    /// 恢复真的施加过 journal 记录前缀的崩溃状态数。
    pub recoveries_that_applied_journal_records: u64,
    pub checker_runs: u64,
    /// 第一截各状态上逐条不变量判绿（判到而没违例）的次数，从给定状态表的计数向量并进来。
    pub invariant_holds: BTreeMap<&'static str, u64>,
    /// 第一截各状态上逐条不变量没判到的次数：不适用或部分没判（GPU 判器的判到位不分这两样，两个判器同一个数法），不红，也不算判绿。
    pub invariant_not_judged: BTreeMap<&'static str, u64>,
    /// 记录核对器跑过几次、两类各判红几次（与崩溃点重放那一路同一份判据，`crash::check_records_against`）。
    pub record_checks: u64,
    pub record_root_without_record: u64,
    pub record_claimed_state_missing_unit: u64,
    /// 问过模型「恢复到的这一版允不允许」的次数（每个崩溃状态一次）。
    pub model_judgements: u64,
    /// 模型认得择到的那条根、并真的逐字节比过内容的次数。
    pub model_contents_compared: u64,
    /// 模型认得择到的那条根、那一版树表 0 条、实现也报没有文件的次数。
    pub model_versions_without_a_file_matched: u64,
    /// 攒到的「模型提交过的每一版」目录里，最大的一段有几版。
    pub most_committed_versions_in_one_history: usize,
    /// 在崩溃后镜像上起的可写挂载（每个崩溃状态一次）：做成的、在单元区墙上被拒的、被别的理由拒的（后者判红）。
    pub writable_mounts_after_the_crash: u64,
    pub writable_mounts_after_the_crash_succeeded: u64,
    pub writable_mounts_after_the_crash_refused_at_the_unit_area_wall: u64,
    pub writable_mounts_after_the_crash_refused: u64,
    /// 挂载做成之后的那一次发布（第一个文件还活着的一版上覆盖写、树表 0 条的一版上发第一个文件、第一个文件已删完或正在删的一版上建一个 inode）：
    /// 做成的、单元区墙、别的理由（判红）。
    pub publishes_after_the_writable_mount: u64,
    pub publishes_after_the_writable_mount_refused_at_the_unit_area_wall: u64,
    pub publishes_after_the_writable_mount_refused: u64,
    /// 挂载与那次发布之后的池上跑的池级 checker 与记录核对器。
    pub checker_runs_after_the_writable_mount: u64,
    pub record_checks_after_the_writable_mount: u64,
    /// 崩溃后镜像上那次可写挂载末尾的清扫发布删了行的次数，与那几次删行之前一共抹了几个头（清扫那两段摆没摆得到二次崩溃按它们看）。
    pub writable_mounts_after_the_crash_deleting_instance_table_rows: u64,
    pub heads_erased_by_the_instance_table_sweep_after_the_crash: u64,
    /// 挂载途中的二次崩溃：摆了几个、按崩在挂载的哪一段分。
    pub second_crash_points: u64,
    pub second_crash_points_by_phase: BTreeMap<WritableMountPhase, u64>,
    /// 二次崩溃状态上：恢复的三种结局、问模型的次数、池级 checker 与记录核对器跑的次数。
    pub second_crash_recoveries_reading_a_file: u64,
    pub second_crash_recoveries_without_a_file: u64,
    pub second_crash_recoveries_failed: u64,
    pub second_crash_model_judgements: u64,
    pub second_crash_checker_runs: u64,
    pub second_crash_record_checks: u64,
    /// 二次崩溃状态上判 I-8.4 的次数（每个二次崩溃状态一次，在它上面再起一次可写挂载）：判了的按崩在挂载的哪一段分、
    /// 读出的内容与没崩过的那一次相同的、不同的（判红），没判的按为什么没判分。
    pub remounts_after_a_second_crash: u64,
    pub remount_idempotence_judged_by_phase: BTreeMap<WritableMountPhase, u64>,
    pub remounts_after_a_second_crash_reading_the_same_content: u64,
    pub remounts_after_a_second_crash_reading_different_content: u64,
    pub remount_idempotence_not_judged: BTreeMap<RemountIdempotenceNotJudged, u64>,
    /// 二次崩溃之后再挂被拒、拒因是判据 N-配置 的（第三截判 C670（无读故障的崩溃态上可写挂载见证判据为真没有判官） 那一条）：判过的、
    /// 崩溃态没注入读故障而判据在二次崩溃镜像上复算仍为真、拒得不合法的（判红）。
    pub remount_refusals_for_a_newer_publish_witnessed_by_system_configuration: u64,
    pub remount_refusals_with_the_witness_criterion_true_without_read_faults: u64,
    /// 二次崩溃之后再挂被拒、拒因是「读阶段判完之后系统配置又读不出」的（第三截判改法 G4 那一条，与 C670 同形）：判过的、
    /// 崩溃态没注入读故障而那一拒的判据在二次崩溃镜像上复算为真、拒得不合法的（判红）。
    pub remount_refusals_for_a_system_configuration_unreadable_after_the_read_stage: u64,
    pub remount_refusals_with_the_system_configuration_unreadable_criterion_true_without_read_faults:
        u64,
    /// 二次崩溃之后再挂被拒、拒因是「重建分配器读根环整环重读一次仍一条不被抛弃的自证有效根都没有」的（第三截判 Q3 那一条，与 G4 同形）：
    /// 判过的、崩溃态没注入读故障而那一拒的判据在二次崩溃镜像上复算为真、拒得不合法的（判红）。
    pub remount_refusals_for_no_valid_root_in_the_root_ring_of_the_rebuilt_allocator: u64,
    pub remount_refusals_with_the_no_valid_root_in_the_root_ring_criterion_true_without_read_faults:
        u64,
    /// 二次崩溃之后再挂被拒、拒因是「记录有缺口时拒可写」的（D23（journal 的角色与格式） 已定项 14，F1 乙）：判过的次数。这一拒没有 C670 同形的那一条
    /// （判官不列「没注入读故障时判据复算为真」那一类条件），只差 U6 那几条的照别的拒因不报红。
    pub remount_refusals_for_a_publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify:
        u64,
    /// 二次崩溃之后再挂被拒、拒因是 C 读一次没施加的发布的账读不出、读得出的记录不齐、映射也现算不成的（`C_MAPPING` 收窄）：判过的次数。
    /// 与 F1 乙同样没有 C670 同形的那一条，只差 U6 那几条的照别的拒因不报红。
    pub remount_refusals_for_an_account_of_an_unapplied_publish_with_incomplete_readable_records:
        u64,
    /// 每一截（崩溃镜像本身、挂载与一次发布之后、二次崩溃）判到已知红与新发现的状态数之和。
    pub crash_states_ending_known_red: BTreeMap<usize, u64>,
    pub crash_states_ending_new_finding: u64,
    /// 挂载透明：第一次崩溃镜像上按动作判了几次、二次崩溃镜像上按崩在挂载的哪一段判了几次；整个动作没判的按理由分；
    /// 判红的按字段分（名字里带「量过」或「推的」，提交时的门禁一眼分得开）；某一样没判的按理由分。
    pub mount_transparency_judged: BTreeMap<MountTransparencyComparison, u64>,
    pub mount_transparency_judged_on_second_crashes_by_phase: BTreeMap<WritableMountPhase, u64>,
    pub mount_transparency_not_judged: BTreeMap<&'static str, u64>,
    pub mount_transparency_violations_by_field: BTreeMap<&'static str, u64>,
    pub mount_transparency_fields_not_judged: BTreeMap<&'static str, u64>,
    /// 挂载透明判的时候 R2 那一版是清扫发布那一版、把清扫删的行放回去再比的次数（C704（挂载透明不认清扫发布）的处置真走到了）。
    pub mount_transparency_judged_with_the_rows_deleted_by_the_sweep_put_back: u64,
    /// 崩溃后镜像上的可写挂载被拒、理由在 D23（journal 的角色与格式） 已定项 14 那张单子上的：拒得合法的、不合法的（判红）。
    pub writable_mount_refusals_on_the_legal_refusal_list_legal: u64,
    pub writable_mount_refusals_on_the_legal_refusal_list_illegal: u64,
    /// 崩溃窗口末尾之前历史里有一步在注入的读故障下发过东西的崩溃状态：第二截判拒可写时不判 C670（无读故障的崩溃态上可写挂载见证判据为真没有判官）
    /// 与改法 G4 那两条。
    pub crash_points_after_a_step_reading_under_injected_faults: u64,
    /// 第一截与二次崩溃那一截上 RefFS 与 O1 判官两票合起来（任一红即红），按哪一边红分开计；
    /// RefFS 弃权的按理由另计（两截并在一起）。
    pub reference_filesystem_and_model_verdicts_on_crash_images:
        BTreeMap<JointVerdictOfTheReferenceFileSystemAndTheModel, u64>,
    pub reference_filesystem_and_model_verdicts_on_second_crashes:
        BTreeMap<JointVerdictOfTheReferenceFileSystemAndTheModel, u64>,
    pub reference_filesystem_abstentions: BTreeMap<&'static str, u64>,
    /// 给定状态表里判器交了占位结论的张数（[`GivenStatesConclusionsSource::PlaceholderOfATableLeftToAnotherHost`]：两台分片、这张表归别的机器判、
    /// 库里又没有）：不为 0 时这一批的计数偏少、新发现可能漏，报告里另打一行（[`CrashInjectionReport::render`]），入口照「本次未跑」交回。
    pub given_states_tables_left_to_other_hosts: u64,
    /// 给定状态表里判器只录了没判的张数（[`GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly`]：这一趟只跑录入段，
    /// `SINGLEFS_CRASH_AMPLIFICATION_STAGE=record`）：交的是占位结论，这一批的计数与新发现不作数，入口照「本次未跑」交回、等判段那一趟。
    pub given_states_tables_recorded_only: u64,
}

impl CrashInjectionTally {
    /// 把紧跟在后面的那一段的计数并进来：逐项相加，按字段拆开写全——新加一个字段而这里没并，编译不过。
    pub fn absorb(&mut self, following: &CrashInjectionTally) {
        let CrashInjectionTally {
            histories,
            histories_completed,
            histories_ended_known_red,
            histories_ended_new_finding,
            histories_without_any_crash_point,
            crash_points,
            crash_points_withholding_write_kind,
            crash_points_by_operation_kind,
            crash_points_inside_the_starting_point,
            crash_points_withholding_a_write_before_a_persisted_one,
            crash_points_that_land_before_the_last_committed_version,
            crash_points_inside_an_unfinished_publish,
            recoveries_reading_a_file,
            recoveries_without_a_file,
            recoveries_failed,
            recoveries_that_applied_journal_records,
            checker_runs,
            invariant_holds,
            invariant_not_judged,
            record_checks,
            record_root_without_record,
            record_claimed_state_missing_unit,
            model_judgements,
            model_contents_compared,
            model_versions_without_a_file_matched,
            most_committed_versions_in_one_history,
            writable_mounts_after_the_crash,
            writable_mounts_after_the_crash_succeeded,
            writable_mounts_after_the_crash_refused_at_the_unit_area_wall,
            writable_mounts_after_the_crash_refused,
            publishes_after_the_writable_mount,
            publishes_after_the_writable_mount_refused_at_the_unit_area_wall,
            publishes_after_the_writable_mount_refused,
            checker_runs_after_the_writable_mount,
            record_checks_after_the_writable_mount,
            writable_mounts_after_the_crash_deleting_instance_table_rows,
            heads_erased_by_the_instance_table_sweep_after_the_crash,
            second_crash_points,
            second_crash_points_by_phase,
            second_crash_recoveries_reading_a_file,
            second_crash_recoveries_without_a_file,
            second_crash_recoveries_failed,
            second_crash_model_judgements,
            second_crash_checker_runs,
            second_crash_record_checks,
            remounts_after_a_second_crash,
            remount_idempotence_judged_by_phase,
            remounts_after_a_second_crash_reading_the_same_content,
            remounts_after_a_second_crash_reading_different_content,
            remount_idempotence_not_judged,
            remount_refusals_for_a_newer_publish_witnessed_by_system_configuration,
            remount_refusals_with_the_witness_criterion_true_without_read_faults,
            remount_refusals_for_a_system_configuration_unreadable_after_the_read_stage,
            remount_refusals_with_the_system_configuration_unreadable_criterion_true_without_read_faults,
            remount_refusals_for_no_valid_root_in_the_root_ring_of_the_rebuilt_allocator,
            remount_refusals_with_the_no_valid_root_in_the_root_ring_criterion_true_without_read_faults,
            remount_refusals_for_a_publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify,
            remount_refusals_for_an_account_of_an_unapplied_publish_with_incomplete_readable_records,
            crash_states_ending_known_red,
            crash_states_ending_new_finding,
            mount_transparency_judged,
            mount_transparency_judged_on_second_crashes_by_phase,
            mount_transparency_not_judged,
            mount_transparency_violations_by_field,
            mount_transparency_fields_not_judged,
            mount_transparency_judged_with_the_rows_deleted_by_the_sweep_put_back,
            writable_mount_refusals_on_the_legal_refusal_list_legal,
            writable_mount_refusals_on_the_legal_refusal_list_illegal,
            crash_points_after_a_step_reading_under_injected_faults,
            reference_filesystem_and_model_verdicts_on_crash_images,
            reference_filesystem_and_model_verdicts_on_second_crashes,
            reference_filesystem_abstentions,
            given_states_tables_left_to_other_hosts,
            given_states_tables_recorded_only,
        } = following;
        self.histories += histories;
        self.histories_completed += histories_completed;
        for (form, count) in histories_ended_known_red {
            *self.histories_ended_known_red.entry(*form).or_insert(0) += count;
        }
        self.histories_ended_new_finding += histories_ended_new_finding;
        self.histories_without_any_crash_point += histories_without_any_crash_point;
        self.crash_points += crash_points;
        for (kind, count) in crash_points_withholding_write_kind {
            *self
                .crash_points_withholding_write_kind
                .entry(kind)
                .or_insert(0) += count;
        }
        for (kind, count) in crash_points_by_operation_kind {
            *self
                .crash_points_by_operation_kind
                .entry(*kind)
                .or_insert(0) += count;
        }
        self.crash_points_inside_the_starting_point += crash_points_inside_the_starting_point;
        self.crash_points_withholding_a_write_before_a_persisted_one +=
            crash_points_withholding_a_write_before_a_persisted_one;
        self.crash_points_that_land_before_the_last_committed_version +=
            crash_points_that_land_before_the_last_committed_version;
        self.crash_points_inside_an_unfinished_publish += crash_points_inside_an_unfinished_publish;
        self.recoveries_reading_a_file += recoveries_reading_a_file;
        self.recoveries_without_a_file += recoveries_without_a_file;
        self.recoveries_failed += recoveries_failed;
        self.recoveries_that_applied_journal_records += recoveries_that_applied_journal_records;
        self.checker_runs += checker_runs;
        for (invariant, count) in invariant_holds {
            *self.invariant_holds.entry(invariant).or_insert(0) += count;
        }
        for (invariant, count) in invariant_not_judged {
            *self.invariant_not_judged.entry(invariant).or_insert(0) += count;
        }
        self.record_checks += record_checks;
        self.record_root_without_record += record_root_without_record;
        self.record_claimed_state_missing_unit += record_claimed_state_missing_unit;
        self.model_judgements += model_judgements;
        self.model_contents_compared += model_contents_compared;
        self.model_versions_without_a_file_matched += model_versions_without_a_file_matched;
        self.most_committed_versions_in_one_history = self
            .most_committed_versions_in_one_history
            .max(*most_committed_versions_in_one_history);
        self.writable_mounts_after_the_crash += writable_mounts_after_the_crash;
        self.writable_mounts_after_the_crash_succeeded += writable_mounts_after_the_crash_succeeded;
        self.writable_mounts_after_the_crash_refused_at_the_unit_area_wall +=
            writable_mounts_after_the_crash_refused_at_the_unit_area_wall;
        self.writable_mounts_after_the_crash_refused += writable_mounts_after_the_crash_refused;
        self.publishes_after_the_writable_mount += publishes_after_the_writable_mount;
        self.publishes_after_the_writable_mount_refused_at_the_unit_area_wall +=
            publishes_after_the_writable_mount_refused_at_the_unit_area_wall;
        self.publishes_after_the_writable_mount_refused +=
            publishes_after_the_writable_mount_refused;
        self.checker_runs_after_the_writable_mount += checker_runs_after_the_writable_mount;
        self.record_checks_after_the_writable_mount += record_checks_after_the_writable_mount;
        self.writable_mounts_after_the_crash_deleting_instance_table_rows +=
            writable_mounts_after_the_crash_deleting_instance_table_rows;
        self.heads_erased_by_the_instance_table_sweep_after_the_crash +=
            heads_erased_by_the_instance_table_sweep_after_the_crash;
        self.second_crash_points += second_crash_points;
        for (phase, count) in second_crash_points_by_phase {
            *self.second_crash_points_by_phase.entry(*phase).or_insert(0) += count;
        }
        self.second_crash_recoveries_reading_a_file += second_crash_recoveries_reading_a_file;
        self.second_crash_recoveries_without_a_file += second_crash_recoveries_without_a_file;
        self.second_crash_recoveries_failed += second_crash_recoveries_failed;
        self.second_crash_model_judgements += second_crash_model_judgements;
        self.second_crash_checker_runs += second_crash_checker_runs;
        self.second_crash_record_checks += second_crash_record_checks;
        self.remounts_after_a_second_crash += remounts_after_a_second_crash;
        for (phase, count) in remount_idempotence_judged_by_phase {
            *self
                .remount_idempotence_judged_by_phase
                .entry(*phase)
                .or_insert(0) += count;
        }
        self.remounts_after_a_second_crash_reading_the_same_content +=
            remounts_after_a_second_crash_reading_the_same_content;
        self.remounts_after_a_second_crash_reading_different_content +=
            remounts_after_a_second_crash_reading_different_content;
        for (reason, count) in remount_idempotence_not_judged {
            *self
                .remount_idempotence_not_judged
                .entry(*reason)
                .or_insert(0) += count;
        }
        self.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration +=
            remount_refusals_for_a_newer_publish_witnessed_by_system_configuration;
        self.remount_refusals_with_the_witness_criterion_true_without_read_faults +=
            remount_refusals_with_the_witness_criterion_true_without_read_faults;
        self.remount_refusals_for_a_system_configuration_unreadable_after_the_read_stage +=
            remount_refusals_for_a_system_configuration_unreadable_after_the_read_stage;
        self.remount_refusals_with_the_system_configuration_unreadable_criterion_true_without_read_faults +=
            remount_refusals_with_the_system_configuration_unreadable_criterion_true_without_read_faults;
        self.remount_refusals_for_no_valid_root_in_the_root_ring_of_the_rebuilt_allocator +=
            remount_refusals_for_no_valid_root_in_the_root_ring_of_the_rebuilt_allocator;
        self.remount_refusals_with_the_no_valid_root_in_the_root_ring_criterion_true_without_read_faults +=
            remount_refusals_with_the_no_valid_root_in_the_root_ring_criterion_true_without_read_faults;
        self.remount_refusals_for_a_publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify +=
            remount_refusals_for_a_publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify;
        self.remount_refusals_for_an_account_of_an_unapplied_publish_with_incomplete_readable_records +=
            remount_refusals_for_an_account_of_an_unapplied_publish_with_incomplete_readable_records;
        for (form, count) in crash_states_ending_known_red {
            *self.crash_states_ending_known_red.entry(*form).or_insert(0) += count;
        }
        self.crash_states_ending_new_finding += crash_states_ending_new_finding;
        for (comparison, count) in mount_transparency_judged {
            *self
                .mount_transparency_judged
                .entry(*comparison)
                .or_insert(0) += count;
        }
        for (phase, count) in mount_transparency_judged_on_second_crashes_by_phase {
            *self
                .mount_transparency_judged_on_second_crashes_by_phase
                .entry(*phase)
                .or_insert(0) += count;
        }
        for (reason, count) in mount_transparency_not_judged {
            *self
                .mount_transparency_not_judged
                .entry(reason)
                .or_insert(0) += count;
        }
        for (field, count) in mount_transparency_violations_by_field {
            *self
                .mount_transparency_violations_by_field
                .entry(field)
                .or_insert(0) += count;
        }
        for (reason, count) in mount_transparency_fields_not_judged {
            *self
                .mount_transparency_fields_not_judged
                .entry(reason)
                .or_insert(0) += count;
        }
        self.mount_transparency_judged_with_the_rows_deleted_by_the_sweep_put_back +=
            mount_transparency_judged_with_the_rows_deleted_by_the_sweep_put_back;
        self.writable_mount_refusals_on_the_legal_refusal_list_legal +=
            writable_mount_refusals_on_the_legal_refusal_list_legal;
        self.writable_mount_refusals_on_the_legal_refusal_list_illegal +=
            writable_mount_refusals_on_the_legal_refusal_list_illegal;
        self.crash_points_after_a_step_reading_under_injected_faults +=
            crash_points_after_a_step_reading_under_injected_faults;
        for (verdict, count) in reference_filesystem_and_model_verdicts_on_crash_images {
            *self
                .reference_filesystem_and_model_verdicts_on_crash_images
                .entry(*verdict)
                .or_insert(0) += count;
        }
        for (verdict, count) in reference_filesystem_and_model_verdicts_on_second_crashes {
            *self
                .reference_filesystem_and_model_verdicts_on_second_crashes
                .entry(*verdict)
                .or_insert(0) += count;
        }
        for (reason, count) in reference_filesystem_abstentions {
            *self
                .reference_filesystem_abstentions
                .entry(reason)
                .or_insert(0) += count;
        }
        self.given_states_tables_left_to_other_hosts += given_states_tables_left_to_other_hosts;
        self.given_states_tables_recorded_only += given_states_tables_recorded_only;
    }

    /// 记一票：RefFS 与 O1 判官两票合起来的那一格，RefFS 弃权时连理由。
    pub(super) fn note_the_reference_filesystem_vote(
        &mut self,
        stage: StageTheReferenceFileSystemJudges,
        vote: &ReferenceFileSystemVote,
        model_disagreement: Option<&ModelDisagreement>,
    ) {
        let joint = JointVerdictOfTheReferenceFileSystemAndTheModel::of(vote, model_disagreement);
        let verdicts = match stage {
            StageTheReferenceFileSystemJudges::CrashImage => {
                &mut self.reference_filesystem_and_model_verdicts_on_crash_images
            }
            StageTheReferenceFileSystemJudges::SecondCrashInsideTheWritableMount => {
                &mut self.reference_filesystem_and_model_verdicts_on_second_crashes
            }
        };
        *verdicts.entry(joint).or_insert(0) += 1;
        if let ReferenceFileSystemVote::Abstained(reason) = vote {
            *self
                .reference_filesystem_abstentions
                .entry(reason.name())
                .or_insert(0) += 1;
        }
    }

    /// 给人看的一整块：绝对数——摆了几个崩溃状态、恢复成了几次、checker 与记录核对器跑了几次、模型判了几次、新发现几条。
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = String::new();
        let _ = writeln!(
            text,
            "历史 {} 段：跑完 {}、以已知红收尾 {:?}、新发现 {}；一个崩溃状态都摆不出的 {} 段；模型目录最多 {} 版",
            self.histories,
            self.histories_completed,
            self.histories_ended_known_red,
            self.histories_ended_new_finding,
            self.histories_without_any_crash_point,
            self.most_committed_versions_in_one_history
        );
        let _ = writeln!(
            text,
            "崩溃状态 {} 个：截在发布中间（根还没落盘）的 {} 个、截回到最后一版之前的 {} 个、段内有洞（后发的写先持久）的 {} 个、落在起点那一段里的 {} 个",
            self.crash_points,
            self.crash_points_inside_an_unfinished_publish,
            self.crash_points_that_land_before_the_last_committed_version,
            self.crash_points_withholding_a_write_before_a_persisted_one,
            self.crash_points_inside_the_starting_point
        );
        for (kind, count) in &self.crash_points_withholding_write_kind {
            let _ = writeln!(text, "  段内扣下过 {kind}：{count} 个");
        }
        for kind in HistoryOperationKind::ALL {
            let _ = writeln!(
                text,
                "  落在 {kind:?} 里：{} 个",
                self.crash_points_by_operation_kind
                    .get(&kind)
                    .copied()
                    .unwrap_or(0)
            );
        }
        let _ = writeln!(
            text,
            "恢复：读回文件 {} 次、没有文件 {} 次、失败 {} 次；施加过 journal 记录前缀 {} 次",
            self.recoveries_reading_a_file,
            self.recoveries_without_a_file,
            self.recoveries_failed,
            self.recoveries_that_applied_journal_records
        );
        let _ = writeln!(
            text,
            "崩溃状态上 checker 跑了 {} 次；记录核对器跑了 {} 次：根在而记录一条都不在 {} 次、恢复自称新态而单元缺席 {} 次",
            self.checker_runs,
            self.record_checks,
            self.record_root_without_record,
            self.record_claimed_state_missing_unit
        );
        let _ = writeln!(
            text,
            "问模型 {} 次：比过内容 {} 次、树表 0 条对上 {} 次",
            self.model_judgements,
            self.model_contents_compared,
            self.model_versions_without_a_file_matched
        );
        let _ = writeln!(
            text,
            "崩溃后镜像上的可写挂载 {} 次：做成 {}、单元区墙上被拒 {}、别的理由被拒 {}；之后的一次发布做成 {}、单元区墙上被拒 {}、别的理由被拒 {}；之后的池上 checker 跑了 {} 次、记录核对器跑了 {} 次",
            self.writable_mounts_after_the_crash,
            self.writable_mounts_after_the_crash_succeeded,
            self.writable_mounts_after_the_crash_refused_at_the_unit_area_wall,
            self.writable_mounts_after_the_crash_refused,
            self.publishes_after_the_writable_mount,
            self.publishes_after_the_writable_mount_refused_at_the_unit_area_wall,
            self.publishes_after_the_writable_mount_refused,
            self.checker_runs_after_the_writable_mount,
            self.record_checks_after_the_writable_mount
        );
        let _ = writeln!(
            text,
            "崩溃后那次可写挂载末尾的清扫发布删了行 {} 次，删行之前一共抹了 {} 个头",
            self.writable_mounts_after_the_crash_deleting_instance_table_rows,
            self.heads_erased_by_the_instance_table_sweep_after_the_crash
        );
        let _ = writeln!(
            text,
            "挂载途中的二次崩溃 {} 个：恢复读回文件 {} 次、没有文件 {} 次、失败 {} 次；问模型 {} 次、checker {} 次、记录核对器 {} 次",
            self.second_crash_points,
            self.second_crash_recoveries_reading_a_file,
            self.second_crash_recoveries_without_a_file,
            self.second_crash_recoveries_failed,
            self.second_crash_model_judgements,
            self.second_crash_checker_runs,
            self.second_crash_record_checks
        );
        for phase in WritableMountPhase::ALL {
            let _ = writeln!(
                text,
                "  二次崩溃崩在挂载的 {}：{} 个",
                phase.name(),
                self.second_crash_points_by_phase
                    .get(&phase)
                    .copied()
                    .unwrap_or(0)
            );
        }
        let _ = writeln!(
            text,
            "二次崩溃之后再挂 {} 次（I-8.4 的 R-换）：读出的内容与没崩过的那一次相同 {}、不同 {}；判了的按段 {:?}；没判的 {:?}",
            self.remounts_after_a_second_crash,
            self.remounts_after_a_second_crash_reading_the_same_content,
            self.remounts_after_a_second_crash_reading_different_content,
            self.remount_idempotence_judged_by_phase
                .iter()
                .map(|(phase, count)| (phase.name(), *count))
                .collect::<Vec<_>>(),
            self.remount_idempotence_not_judged
        );
        let _ = writeln!(
            text,
            "二次崩溃之后再挂被拒、拒因是判据 N-配置 的（第三截判 C670 那一条）：{} 次，崩溃态没注入读故障而判据复算仍为真、拒得不合法 {}",
            self.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration,
            self.remount_refusals_with_the_witness_criterion_true_without_read_faults
        );
        let _ = writeln!(
            text,
            "二次崩溃之后再挂被拒、拒因是读阶段判完之后系统配置又读不出的（第三截判 G4 那一条）：{} 次，崩溃态没注入读故障而判据复算为真、拒得不合法 {}",
            self.remount_refusals_for_a_system_configuration_unreadable_after_the_read_stage,
            self.remount_refusals_with_the_system_configuration_unreadable_criterion_true_without_read_faults
        );
        let _ = writeln!(
            text,
            "二次崩溃之后再挂被拒、拒因是重建分配器读根环整环重读一次仍一条有效根都没有的（第三截判 Q3 那一条）：{} 次，崩溃态没注入读故障而判据复算为真、拒得不合法 {}",
            self.remount_refusals_for_no_valid_root_in_the_root_ring_of_the_rebuilt_allocator,
            self.remount_refusals_with_the_no_valid_root_in_the_root_ring_criterion_true_without_read_faults
        );
        let _ = writeln!(
            text,
            "二次崩溃之后再挂被拒、拒因是记录有缺口时拒可写的（F1 乙）：{} 次",
            self.remount_refusals_for_a_publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify
        );
        let _ = writeln!(
            text,
            "二次崩溃之后再挂被拒、拒因是没施加的发布的账读不出而读得出的记录不齐的（C_MAPPING）：{} 次",
            self.remount_refusals_for_an_account_of_an_unapplied_publish_with_incomplete_readable_records
        );
        let _ = writeln!(
            text,
            "挂载透明：第一次崩溃镜像上判了 {:?}，二次崩溃镜像上按段判了 {:?}；没判的 {:?}；判红的字段 {:?}；某一样没判的 {:?}；把清扫删的行放回去再比的 {} 次",
            self.mount_transparency_judged
                .iter()
                .map(|(comparison, count)| (comparison.name(), *count))
                .collect::<Vec<_>>(),
            self.mount_transparency_judged_on_second_crashes_by_phase
                .iter()
                .map(|(phase, count)| (phase.name(), *count))
                .collect::<Vec<_>>(),
            self.mount_transparency_not_judged,
            self.mount_transparency_violations_by_field,
            self.mount_transparency_fields_not_judged,
            self.mount_transparency_judged_with_the_rows_deleted_by_the_sweep_put_back
        );
        let _ = writeln!(
            text,
            "可写挂载被拒、理由在 D23 已定项 14 那张单子上的：拒得合法 {}、不合法 {}；崩溃窗口之前有一步带读故障、不判 C670 那一条的崩溃状态 {}",
            self.writable_mount_refusals_on_the_legal_refusal_list_legal,
            self.writable_mount_refusals_on_the_legal_refusal_list_illegal,
            self.crash_points_after_a_step_reading_under_injected_faults
        );
        let _ = writeln!(
            text,
            "RefFS 与 O1 判官两票（任一红即红）：第一截 {:?}；二次崩溃 {:?}；RefFS 弃权的理由 {:?}",
            self.reference_filesystem_and_model_verdicts_on_crash_images
                .iter()
                .map(|(verdict, count)| (verdict.name(), *count))
                .collect::<Vec<_>>(),
            self.reference_filesystem_and_model_verdicts_on_second_crashes
                .iter()
                .map(|(verdict, count)| (verdict.name(), *count))
                .collect::<Vec<_>>(),
            self.reference_filesystem_abstentions
        );
        let _ = writeln!(
            text,
            "崩溃状态：以已知红收尾 {:?}、新发现 {}",
            self.crash_states_ending_known_red, self.crash_states_ending_new_finding
        );
        for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
            let _ = writeln!(
                text,
                "  {invariant}：判绿 {} 次、没判到（不适用或部分没判）{} 次",
                self.invariant_holds.get(invariant).copied().unwrap_or(0),
                self.invariant_not_judged
                    .get(invariant)
                    .copied()
                    .unwrap_or(0)
            );
        }
        text
    }
}
