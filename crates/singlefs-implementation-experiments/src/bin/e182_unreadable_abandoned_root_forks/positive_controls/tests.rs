use super::*;
use crate::helpers_shared_by_the_tests::{small_history, specification_of_the_shape};
use std::sync::OnceLock;

#[test]
fn the_detection_control_finds_every_changed_unit() {
    let (trials, detected, _without_a_unit) = detection_control(small_history());
    assert!(trials > 0, "至少试了一次");
    assert_eq!(detected, trials, "每次改了单元的字节「被盖」都判得出");
}

// ------------------------------------------------------------------------
// B4 的阳性对照（登记 5.2 最后一句，修订 12.12）。
// ------------------------------------------------------------------------

/// 已确认档带藏根的小历史（G默认、64 个单元、2 次覆盖写）：类 a 只出在这一族里。
fn confirmed_specification_of_the_shape(shape: OverwriteShape) -> HistorySpecification {
    HistorySpecification {
        family: Family::HiddenRoot(Confirmation::Confirmed),
        ..specification_of_the_shape(shape)
    }
}

fn small_confirmed_history() -> &'static RecordedHistory {
    static HISTORY: OnceLock<RecordedHistory> = OnceLock::new();
    HISTORY.get_or_init(|| {
        record_the_history(&confirmed_specification_of_the_shape(
            OverwriteShape::OneUnit,
        ))
        .expect("已确认档的小历史建得起来")
    })
}

fn small_confirmed_units_kept_history() -> &'static RecordedHistory {
    static HISTORY: OnceLock<RecordedHistory> = OnceLock::new();
    HISTORY.get_or_init(|| {
        record_the_history(&confirmed_specification_of_the_shape(
            OverwriteShape::KeepingTheFirstFileUnits,
        ))
        .expect("补臂的已确认档小历史建得起来")
    })
}

/// 乙入判不出之下对照记作废：
/// 已确认档的实例 2 可写挂载把每块盘较新的那一槽系统配置读成全零，读回全零的槽自证不过、当读错计，没有一块盘两槽都自证过，
/// 判不出，重读一次仍是这样，拒可写（`NewerStateStillUnreadableAfterOneReread`，比法成员 `SomeSlotFailedSelfVerification…`）。
/// 拒了就一个字节都不写，录制流里没有根槽写，类 a 状态一个也挑不到，B4 的「类 a 变类 b」对照造不出，记作废（不是失败，也不是通过）。
/// 旧行为（C971 之前：挑到 2 个类 a 状态、2 次试、2 次由类 a 变类 b、status holds）作废，不再断言。
#[test]
fn the_confirmed_none_covered_control_is_voided_because_the_confirmed_mount_is_refused_by_the_newer_configuration_slots_reading_zero(
) {
    let history = small_confirmed_history();
    assert!(
        history.consistency_problems.is_empty(),
        "{:?}",
        history.consistency_problems
    );
    let refused = history
        .mount
        .refused
        .as_deref()
        .expect("已确认档的可写挂载被拒");
    assert!(
        refused.contains(THE_COMPARISON_THAT_VOIDS_THE_CONFIRMED_NONE_COVERED_CONTROL),
        "{refused}"
    );
    assert!(
        refused.contains("NewerStateStillUnreadableAfterOneReread"),
        "{refused}"
    );
    assert_eq!(
        history.row_publish_root_write, None,
        "拒了可写就没有写行那次发布的根槽写"
    );
    let reading =
        class_confirmed_none_covered_to_class_some_unit_covered_control(history).expect("枚举得完");
    assert_eq!(
        reading,
        ConfirmedNoneCoveredControlReading::default(),
        "读数全零：{reading:?}"
    );
    assert_eq!(
        the_confirmed_none_covered_control_status(
            reading.trials,
            reading.class_confirmed_none_covered_before_the_change,
            reading.changed_to_class_some_unit_covered
        ),
        "not_made"
    );
    assert!(the_confirmed_none_covered_control_is_voided(
        history.mount.refused.as_deref(),
        &reading
    ));
    // 产物里的记法：这条历史的对照行记 voided，汇总里作废数 1、不算检测对照失败。
    let mut total = ConfirmedNoneCoveredControlTotal::default();
    emit_the_confirmed_none_covered_control(history, &mut total);
    assert_eq!(total.histories, 1);
    assert_eq!(total.histories_voided, 1, "{total:?}");
    assert_eq!(total.histories_not_made, 0, "{total:?}");
    assert_eq!(total.status(), "voided");
    assert_eq!(total.failures(), 0, "作废不算失败");
}

/// 补臂的已确认档小历史同样作废：挑不到类 a 状态，「试过的都判出」会是 0 == 0 空过，这里钉住作废。
#[test]
fn the_confirmed_none_covered_control_on_the_units_kept_arm_is_voided_the_same_way() {
    let history = small_confirmed_units_kept_history();
    assert!(
        history.consistency_problems.is_empty(),
        "{:?}",
        history.consistency_problems
    );
    let refused = history
        .mount
        .refused
        .as_deref()
        .expect("补臂的已确认档可写挂载被拒");
    assert!(
        refused.contains(THE_COMPARISON_THAT_VOIDS_THE_CONFIRMED_NONE_COVERED_CONTROL),
        "{refused}"
    );
    assert_eq!(history.row_publish_root_write, None);
    let reading =
        class_confirmed_none_covered_to_class_some_unit_covered_control(history).expect("枚举得完");
    assert_eq!(
        reading,
        ConfirmedNoneCoveredControlReading::default(),
        "读数全零：{reading:?}"
    );
    assert!(the_confirmed_none_covered_control_is_voided(
        history.mount.refused.as_deref(),
        &reading
    ));
    // 产物里的记法：这条历史的对照行记 voided，汇总里作废数 1、不算检测对照失败。
    let mut total = ConfirmedNoneCoveredControlTotal::default();
    emit_the_confirmed_none_covered_control(history, &mut total);
    assert_eq!(total.histories, 1);
    assert_eq!(total.histories_voided, 1, "{total:?}");
    assert_eq!(total.histories_not_made, 0, "{total:?}");
    assert_eq!(total.status(), "voided");
    assert_eq!(total.failures(), 0, "作废不算失败");
}

/// 作废的认法与汇总：拒因不是那个比法成员、或读数不全零、或没被拒，都不算作废；全部作废时汇总记 voided、不算失败，
/// 作废混着没造出的仍算失败（没造出不是作废）。
#[test]
fn control_is_voided_only_when_the_refusal_names_the_comparison_and_nothing_was_chosen() {
    let zero = ConfirmedNoneCoveredControlReading::default();
    let named = format!(
        "NewerStateStillUnreadableAfterOneReread({})",
        THE_COMPARISON_THAT_VOIDS_THE_CONFIRMED_NONE_COVERED_CONTROL
    );
    assert!(the_confirmed_none_covered_control_is_voided(
        Some(&named),
        &zero
    ));
    assert!(!the_confirmed_none_covered_control_is_voided(None, &zero));
    assert!(!the_confirmed_none_covered_control_is_voided(
        Some("NewerStateStillUnreadableAfterOneReread(InstanceTable…)"),
        &zero
    ));
    let chosen = ConfirmedNoneCoveredControlReading {
        class_confirmed_none_covered_states_chosen: 1,
        ..zero
    };
    assert!(!the_confirmed_none_covered_control_is_voided(
        Some(&named),
        &chosen
    ));
    let all_voided = ConfirmedNoneCoveredControlTotal {
        histories: 3,
        histories_voided: 3,
        ..ConfirmedNoneCoveredControlTotal::default()
    };
    assert_eq!(all_voided.status(), "voided");
    assert_eq!(all_voided.failures(), 0, "作废不算失败");
    let voided_and_not_made = ConfirmedNoneCoveredControlTotal {
        histories: 3,
        histories_voided: 2,
        histories_not_made: 1,
        ..ConfirmedNoneCoveredControlTotal::default()
    };
    assert_eq!(voided_and_not_made.status(), "not_made");
    assert_eq!(voided_and_not_made.failures(), 1, "混着没造出的不当作废");
}

/// 判定本身：没试成不当通过；换之前不是类 a 或换之后不是类 b，都是 violated。
#[test]
fn the_confirmed_none_covered_control_status_holds_only_when_every_trial_went_from_class_confirmed_none_covered_to_class_some_unit_covered(
) {
    assert_eq!(
        the_confirmed_none_covered_control_status(0, 0, 0),
        "not_made"
    );
    assert_eq!(the_confirmed_none_covered_control_status(2, 2, 2), "holds");
    assert_eq!(
        the_confirmed_none_covered_control_status(2, 2, 1),
        "violated"
    );
    assert_eq!(
        the_confirmed_none_covered_control_status(2, 2, 0),
        "violated"
    );
    assert_eq!(
        the_confirmed_none_covered_control_status(2, 1, 2),
        "violated"
    );
    let one_made_and_held = ConfirmedNoneCoveredControlTotal {
        histories: 3,
        histories_not_made: 2,
        histories_violated: 0,
        histories_voided: 0,
        trials: 2,
        changed_to_class_some_unit_covered: 2,
    };
    assert_eq!(one_made_and_held.status(), "holds");
    assert_eq!(one_made_and_held.failures(), 0);
    let none_made = ConfirmedNoneCoveredControlTotal {
        histories: 3,
        histories_not_made: 3,
        ..ConfirmedNoneCoveredControlTotal::default()
    };
    assert_eq!(none_made.status(), "not_made");
    assert_eq!(none_made.failures(), 1, "一次都没试成不当通过");
    let one_violated = ConfirmedNoneCoveredControlTotal {
        histories_violated: 1,
        trials: 4,
        changed_to_class_some_unit_covered: 3,
        ..one_made_and_held
    };
    assert_eq!(one_violated.status(), "violated");
    assert_eq!(one_violated.failures(), 1);
}

#[test]
fn both_arms_avoid_the_hidden_root_in_a_state_where_the_new_instance_reached_the_kept_disk() {
    let reading = arm_witness_control(small_history());
    assert!(reading.applicable, "挑得到一个状态：{reading:?}");
    assert!(reading.holds, "{reading:?}");
}
