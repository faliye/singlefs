use super::*;

pub(super) fn today() -> FourthRunArm {
    fourth_run_arm_named("today").expect("今天那一臂在臂表里")
}

pub(super) fn row_forty_three() -> SegmentTwoHistory {
    SegmentTwoHistory {
        family: SegmentTwoFamily::RandomRed,
        label: ROW_FORTY_THREE_LABEL.to_string(),
        history: closeout_row_forty_three_history(),
    }
}

pub(super) fn shortest() -> SegmentTwoHistory {
    SegmentTwoHistory {
        family: SegmentTwoFamily::RandomRed,
        label: SHORTEST_REPRODUCTION_LABEL.to_string(),
        history: shortest_reproduction_history(),
    }
}
