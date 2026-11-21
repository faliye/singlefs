//! 带读故障一族一个格的计数（枚举、判、按工作单元分给几条线程）与格的钉值。

use super::*;

// ── 一个格的计数：枚举、判、按工作单元分给几条线程

/// 工作单元：挂载 M 开始之前的盘，再加故障菜单里的一项。前面几次挂载的全部走法在崩溃之后得到的盘，每个单元接着枚举最后一次挂载的一项故障。
pub(super) struct FaultUnit {
    pub(super) world: FaultWorld,
    pub(super) mount_number: usize,
    pub(super) user_publishes_left: usize,
    pub(super) menu_index: usize,
}

pub(super) struct FaultUnitCollector {
    pub(super) units: Vec<FaultUnit>,
}

impl FaultVisitor for FaultUnitCollector {
    fn reach_the_start_of_a_mount(
        &mut self,
        world: &FaultWorld,
        mount_number: usize,
        user_publishes_left: usize,
    ) {
        for entry in mount_fault_menu(world, mount_number) {
            self.units.push(FaultUnit {
                world: world.clone(),
                mount_number,
                user_publishes_left,
                menu_index: entry.menu_index,
            });
        }
    }

    fn visit_a_final_state(&mut self, _world: &FaultWorld) {
        panic!("收工作单元的那一遍停在最后一次挂载开始之前，走不到最后一次挂载的崩溃点");
    }
}

pub(super) struct FaultStateJudge<'run> {
    pub(super) judgement: &'run FaultJudgement,
    pub(super) tally: FaultTally,
    pub(super) histories_seen_in_this_unit: u64,
    pub(super) is_the_first_unit: bool,
}

impl FaultVisitor for FaultStateJudge<'_> {
    fn reach_the_start_of_a_mount(
        &mut self,
        _world: &FaultWorld,
        _mount_number: usize,
        _user_publishes_left: usize,
    ) {
        panic!("判状态的那一遍不设 stop_before_mount");
    }

    fn visit_a_final_state(&mut self, world: &FaultWorld) {
        let is_the_designated_history =
            self.is_the_first_unit && self.histories_seen_in_this_unit == 0;
        self.histories_seen_in_this_unit += 1;
        self.judgement
            .judge_a_history(world, is_the_designated_history, &mut self.tally);
    }
}

/// 一个 (域, M, U) 格上全部历史的计数，闸取甲（r4 档；只给单测用，跑法里一律带闸的取法）。
#[cfg(test)]
pub(super) fn fault_tally_of_a_cell(
    domain: FaultDomain,
    cell: RootChoiceCell,
    run_the_three_steps: bool,
    thread_count: usize,
) -> FaultTally {
    fault_tally_of_a_cell_under_the_gate(
        domain,
        cell,
        WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable,
        run_the_three_steps,
        thread_count,
    )
}

/// 一个 (域, M, U) 格上全部历史的计数，闸取 `witness_gate_rule`。合法历史与 KG 按工作单元分给几条线程；KL 不走这里。
pub(super) fn fault_tally_of_a_cell_under_the_gate(
    domain: FaultDomain,
    cell: RootChoiceCell,
    witness_gate_rule: WitnessGateRule,
    run_the_three_steps: bool,
    thread_count: usize,
) -> FaultTally {
    let collecting = FaultEnumeration {
        domain,
        mounts: cell.mounts,
        stop_before_mount: Some(cell.mounts),
    };
    let mut collector = FaultUnitCollector { units: Vec::new() };
    collecting.enumerate_a_mount(
        &FaultWorld::right_after_mkfs_under_the_gate(domain.geometry(), witness_gate_rule),
        1,
        cell.user_publishes,
        None,
        &mut collector,
    );
    let judgement = FaultJudgement::new(domain, cell.mounts, run_the_three_steps);
    let continuing = FaultEnumeration {
        domain,
        mounts: cell.mounts,
        stop_before_mount: None,
    };
    fault_tally_of_work_units(&collector.units, thread_count, &|unit_index, unit| {
        let mut judge = FaultStateJudge {
            judgement: &judgement,
            tally: FaultTally::new(),
            histories_seen_in_this_unit: 0,
            is_the_first_unit: unit_index == 0,
        };
        continuing.enumerate_a_mount(
            &unit.world,
            unit.mount_number,
            unit.user_publishes_left,
            Some(unit.menu_index),
            &mut judge,
        );
        judge.tally
    })
}

/// 登记第七节乙类（出自设计员的独立生成器 `anchors_r4.py`，输出 `anchors_r4.out` sha256 5a36b72237f84c52e6901ae5affcfc5431fd4c8b123f5b9e476907b749ba1fc7）：
/// 每行 = 域 M U，然后依次是：历史数、历史里有不罩故障的历史数、被判态数、有已确认的被判态数、被拒的挂载数、走了重读的挂载数、带撕裂系统配置槽的被判态数、
/// 同 txg 不同实例同在的被判态数、实例高而 txg 低同在的被判态数、视图里没有根的被判态数、视图里最大根之上的记录条数最大值、
/// 条款理想恢复丢的被判态按因在飞上限 / 罩着的其余 / 不罩三类的数；KF1 的 KJ1 KJ1T KJ2；KF2 的 KJ1 KJ1T KJ2 KJ3；KF4 的 KJ1 KJ1T；
/// 本地择根序 today v1823 v1824 v1825 v1826 v1822 各自的 KJ1 KJ1T KJ2。
const PINNED_FAULT_CELLS_TEXT: &str = "\
KF 1 0 10 0 50 0 0 0 0 0 0 0 1 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0\n\
KF 1 1 39 0 195 0 0 0 0 0 0 0 3 0 0 0 0 0 117 0 0 0 0 0 0 0 0 0 0 0 0 0 0 117 0 0 0 0 0 0 0 0 0\n\
KF 1 2 117 0 585 585 0 0 0 0 0 0 5 0 0 117 468 468 351 585 585 252 585 585 585 117 117 0 117 117 0 468 468 351 117 117 0 117 117 0 117 117 0\n\
KF 1 3 351 0 1755 1755 0 0 0 0 0 0 4 0 0 351 1404 1404 1053 1755 1755 756 1755 1755 1755 351 351 0 351 351 0 1404 1404 1053 351 351 0 351 351 0 351 351 0\n\
KF 2 0 1792 236 8960 0 212 1020 650 0 0 0 2 0 0 0 0 0 4827 0 0 0 0 0 0 0 0 0 0 0 0 0 0 4827 0 0 0 0 0 0 0 0 0\n\
KF 2 1 9401 1218 47005 0 647 5598 1625 0 0 0 3 0 0 0 0 0 40494 0 0 0 0 0 0 0 0 0 0 0 0 0 0 40494 0 0 0 0 0 0 0 0 0\n\
KF 2 2 48590 6207 242950 164835 1952 29529 4550 0 0 0 5 0 0 20655 145557 145557 213495 164835 164835 99999 164835 164835 164835 20655 20655 0 20655 20655 0 145557 145557 213495 20655 20655 0 20655 20655 0 20655 20655 0\n\
KF 2 3 175589 22353 877945 874710 5867 107217 13325 0 0 0 6 0 0 108198 785763 785763 804138 874710 874710 576648 874710 874710 874710 108198 108198 0 108198 108198 0 785763 785763 804138 108198 108198 0 108198 108198 0 108198 108198 0\n\
KF3 1 0 10 0 50 0 0 0 0 0 0 0 1 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0\n\
KF3 1 1 5 0 25 0 0 0 0 0 0 0 2 0 0 0 0 0 15 0 0 0 0 0 0 0 0 0 0 0 0 0 0 15 0 0 0 0 0 0 0 0 0\n\
KF3 1 2 5 0 25 25 0 0 0 0 0 0 3 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KF3 1 3 5 0 25 25 0 0 0 0 0 0 2 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KF3 2 0 1792 236 8960 0 212 1020 650 0 0 0 2 0 0 0 0 0 4827 0 0 0 0 0 0 0 0 0 0 0 0 0 0 4827 0 0 0 0 0 0 0 0 0\n\
KF3 2 1 1587 214 7935 0 317 810 975 0 0 0 2 0 0 0 0 0 6254 0 0 0 0 0 0 0 0 0 0 0 0 0 0 6254 0 0 0 0 0 0 0 0 0\n\
KF3 2 2 2292 307 11460 8275 422 1185 1300 0 0 0 3 0 0 991 7205 7205 9049 8275 8275 5623 8275 8275 8275 991 991 0 991 991 0 7205 7205 9049 991 991 0 991 991 0 991 991 0\n\
KF3 2 3 2477 335 12385 10800 527 1235 1625 0 0 0 3 0 0 1416 9545 9545 9964 10800 10800 7520 10800 10800 10800 1416 1416 0 1416 1416 0 9545 9545 9964 1416 1416 0 1416 1416 0 1416 1416 0\n\
KF3 3 0 275939 67845 1379695 0 46096 334980 111150 0 0 0 2 0 0 0 0 0 1091126 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1091126 0 0 0 0 0 0 0 0 0\n\
KF3 3 1 350403 86331 1752015 0 73710 411175 197925 0 0 0 2 0 0 0 0 0 1633188 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1633188 0 0 0 0 0 0 0 0 0\n\
KF3 3 2 549582 134997 2747910 1782245 118454 634670 323700 0 0 0 3 0 0 157713 1612806 1612806 2553380 1782245 1782245 1335924 1782245 1782245 1782245 157713 157713 0 157713 157713 0 1612806 1612806 2553380 157713 157713 0 157713 157713 0 157713 157713 0\n\
KF3 3 3 765601 187840 3828005 3371140 162278 884730 454675 0 0 0 3 0 0 277332 3110727 3110727 3647308 3371140 3371140 2653847 3371140 3371140 3371140 277332 277332 0 277332 277332 0 3110727 3110727 3647308 277332 277332 0 277332 277332 0 277332 277332 0\n\
KF9 1 0 10 0 50 0 0 0 0 0 0 0 1 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0\n\
KF9 1 1 39 0 195 0 0 0 0 0 0 0 3 0 0 0 0 0 117 0 0 0 0 0 0 0 0 0 0 0 0 0 0 117 0 0 0 0 0 0 0 0 0\n\
KF9 1 2 117 0 585 585 0 0 0 0 0 0 5 0 0 117 468 468 359 585 585 252 585 585 585 117 117 0 117 117 0 468 468 359 117 117 0 117 117 0 117 117 0\n\
KF9 2 0 1792 236 8960 0 212 1020 650 0 0 0 2 0 0 0 0 0 4827 0 0 0 0 0 0 0 0 0 0 0 0 0 0 4827 0 0 0 0 0 0 0 0 0\n\
KF9 2 1 9401 1218 47005 0 647 5598 1625 0 0 0 3 0 0 0 0 0 40494 0 0 0 0 0 0 0 0 0 0 0 0 0 0 40494 0 0 0 0 0 0 0 0 0\n\
KF9 2 2 48590 6207 242950 164835 1952 29529 4550 0 0 0 5 0 0 20655 145557 145557 217831 164835 164835 99999 164835 164835 164835 20655 20655 0 20655 20655 0 145557 145557 217831 20655 20655 0 20655 20655 0 20655 20655 0\n\
KFW 1 0 10 0 50 0 0 0 0 0 0 0 1 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0\n\
KFW 1 1 5 0 25 0 0 0 0 0 0 0 2 0 0 0 0 0 15 0 0 0 0 0 0 0 0 0 0 0 0 0 0 15 0 0 0 0 0 0 0 0 0\n\
KFW 1 2 5 0 25 25 0 0 0 0 0 0 3 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KFW 1 3 5 0 25 25 0 0 0 0 0 0 2 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KFW 1 4 5 0 25 25 0 0 0 0 0 0 2 0 0 5 20 20 15 25 25 11 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KFW 1 5 5 0 25 25 0 0 0 0 0 0 3 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KFW 1 6 5 0 25 25 0 0 0 0 0 0 2 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KFW 1 7 5 0 25 25 0 0 0 0 0 0 2 0 0 5 20 20 15 25 25 11 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KFW 1 8 5 0 25 25 0 0 0 0 0 0 3 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KFW 1 9 5 0 25 25 0 0 0 0 0 0 2 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KFW 1 10 5 0 25 25 0 0 0 0 0 1 2 0 0 4 16 16 12 25 25 11 25 25 25 4 4 0 4 4 0 16 16 12 4 4 0 4 4 0 4 4 0\n\
KFW 1 11 5 0 25 25 0 0 0 0 0 5 3 0 0 0 0 0 0 25 25 12 25 25 25 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n\
KFW 1 12 5 0 25 25 0 0 0 0 0 5 2 0 0 0 0 0 0 25 25 12 25 25 25 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n\
KFW 1 13 5 0 25 25 0 0 0 0 0 5 2 0 0 0 0 0 0 25 25 11 25 25 25 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n\
KFW 1 14 5 0 25 25 0 0 0 0 0 5 3 0 0 0 0 0 0 25 25 12 25 25 25 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n\
KG 1 0 10 0 50 0 0 0 0 0 0 0 1 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0\n\
KG 1 1 39 0 195 0 0 0 0 0 0 0 3 0 0 0 0 0 117 0 0 0 0 0 0 0 0 0 0 0 0 0 0 117 0 0 0 0 0 0 0 0 0\n\
KG 1 2 117 0 585 585 0 0 0 0 0 0 5 0 0 117 468 468 351 585 585 252 585 585 585 117 117 0 117 117 0 468 468 351 117 117 0 117 117 0 117 117 0\n\
KG 1 3 351 0 1755 1755 0 0 0 0 0 0 4 0 0 351 1404 1404 1053 1755 1755 756 1755 1755 1755 351 351 0 351 351 0 1404 1404 1053 351 351 0 351 351 0 351 351 0\n\
KG 2 0 4015 610 20075 0 0 0 925 0 450 0 2 0 0 0 0 0 10824 0 0 0 0 0 0 0 0 0 0 0 0 0 0 10824 0 0 0 0 0 36 0 0 0\n\
KG 2 1 22092 3648 110460 0 0 0 3120 0 3420 0 3 0 0 0 0 0 88296 0 0 0 0 0 0 0 0 0 0 0 0 0 0 88296 0 0 0 0 0 630 0 0 0\n\
KG 2 2 98892 14778 494460 334755 0 0 19035 0 21060 0 5 0 1512 51642 292302 292302 423162 334755 334755 188253 334755 334755 334755 53154 53154 0 53154 53154 0 292302 292302 423162 53154 53154 0 58824 58824 7236 53154 53154 0\n\
KG 2 3 400194 61506 2000970 2000970 0 0 56160 0 104490 0 6 0 19116 265491 1765179 1770093 1756485 2000970 2000970 1261629 2000970 1995300 2000970 265923 284607 0 265923 284607 0 1765179 1770093 1756485 265923 284607 0 309609 346005 49680 265923 284607 0\n\
";

/// 登记第七节「挂载 2 开头那次故障」表（M = 2 的各格合起来；每行 = 域、故障名、历史数、被拒数、走了重读的数）。
const PINNED_MOUNT_TWO_FAULT_TEXT: &str = "\
KF none 28776 310 0\n\
KF RN-T1 28776 310 3639\n\
KF RN-TP 25165 338 0\n\
KF SN0-T1 28776 310 28466\n\
KF SN0-TP 1238 1238 0\n\
KF SN1-T1 28776 310 28466\n\
KF SN1-TP 1238 1238 0\n\
KF SB-T1 28776 310 28466\n\
KF SB-TP 1238 1238 0\n\
KF RI-T1 28776 310 25861\n\
KF RI-TP 3823 1218 0\n\
KF RNSB-T1 28776 310 28466\n\
KF RNSB-TP 1238 1238 0\n\
KF3 none 910 70 0\n\
KF3 RN-T1 910 70 195\n\
KF3 RN-TP 743 98 0\n\
KF3 SN0-T1 910 70 840\n\
KF3 SN0-TP 182 182 0\n\
KF3 SN1-T1 910 70 840\n\
KF3 SN1-TP 182 182 0\n\
KF3 SB-T1 910 70 840\n\
KF3 SB-TP 182 182 0\n\
KF3 RI-T1 910 70 695\n\
KF3 RI-TP 307 162 0\n\
KF3 RNSB-T1 910 70 840\n\
KF3 RNSB-TP 182 182 0\n\
KF9 none 7268 105 0\n\
KF9 RN-T1 7268 105 1182\n\
KF9 RN-TP 6107 126 0\n\
KF9 SN0-T1 7268 105 7163\n\
KF9 SN0-TP 393 393 0\n\
KF9 SN1-T1 7268 105 7163\n\
KF9 SN1-TP 393 393 0\n\
KF9 SB-T1 7268 105 7163\n\
KF9 SB-TP 393 393 0\n\
KF9 RI-T1 7268 105 6313\n\
KF9 RI-TP 1228 378 0\n\
KF9 RNSB-T1 7268 105 7163\n\
KF9 RNSB-TP 393 393 0\n\
KG none 40921 0 0\n\
KG RN-T1 40271 0 0\n\
KG RN-TP 40271 0 0\n\
KG SN0-T1 40921 0 0\n\
KG SN0-TP 40921 0 0\n\
KG SN1-T1 40921 0 0\n\
KG SN1-TP 40921 0 0\n\
KG SB-T1 40921 0 0\n\
KG SB-TP 40921 0 0\n\
KG RI-T1 38831 0 0\n\
KG RI-TP 38831 0 0\n\
KG RNSB-T1 40271 0 0\n\
KG RNSB-TP 40271 0 0\n\
";

/// 登记 r5 修订 1 修1.七乙类（闸取 R2；出自设计员的独立生成器 `anchors_s5.py`，sha256 133e56358b88a284e9a5afcbb19197be3e6d2b6ce571c2f87549809144b83af7，
/// 输出 `anchors_s5.out` sha256 4578b08579af7e0198e23326b61ea4fe4711464533c72da930bfdfe8a42230c1，由第三段执行员的 `to_pinned_text.py cells` 只换形摊成这一张；
/// 同一份脚本对 `E165_S5_GATE=A` 那一遍的输出摊出来与 [`PINNED_FAULT_CELLS_TEXT`] 逐行相同）：列同 [`PINNED_FAULT_CELLS_TEXT`]。
/// 与取甲那一张不同的 15 行是 KF、KF3、KF9 的 M ≥ 2 各格，其余 34 行逐行相同。
const PINNED_FAULT_CELLS_TEXT_UNDER_THE_READ_ERROR_RULE: &str = "\
KF 1 0 10 0 50 0 0 0 0 0 0 0 1 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0\n\
KF 1 1 39 0 195 0 0 0 0 0 0 0 3 0 0 0 0 0 117 0 0 0 0 0 0 0 0 0 0 0 0 0 0 117 0 0 0 0 0 0 0 0 0\n\
KF 1 2 117 0 585 585 0 0 0 0 0 0 5 0 0 117 468 468 351 585 585 252 585 585 585 117 117 0 117 117 0 468 468 351 117 117 0 117 117 0 117 117 0\n\
KF 1 3 351 0 1755 1755 0 0 0 0 0 0 4 0 0 351 1404 1404 1053 1755 1755 756 1755 1755 1755 351 351 0 351 351 0 1404 1404 1053 351 351 0 351 351 0 351 351 0\n\
KF 2 0 3094 351 15470 0 99 1130 245 0 0 0 2 0 0 0 0 0 9265 0 0 0 0 0 0 0 0 0 0 0 0 0 0 9265 0 0 0 0 0 0 0 0 0\n\
KF 2 1 15639 1733 78195 0 318 5619 620 0 0 0 3 0 0 0 0 0 70332 0 0 0 0 0 0 0 0 0 0 0 0 0 0 70332 0 0 0 0 0 0 0 0 0\n\
KF 2 2 78888 8567 394440 275850 975 27900 1745 0 0 0 5 0 0 35820 247356 247356 354423 275850 275850 167166 275850 275850 275850 35820 35820 0 35820 35820 0 247356 247356 354423 35820 35820 0 35820 35820 0 35820 35820 0\n\
KF 2 3 282585 30518 1412925 1411335 2946 99414 5120 0 0 0 6 0 0 178317 1279296 1279296 1316046 1411335 1411335 919764 1411335 1411335 1411335 178317 178317 0 178317 178317 0 1279296 1279296 1316046 178317 178317 0 178317 178317 0 178317 178317 0\n\
KF3 1 0 10 0 50 0 0 0 0 0 0 0 1 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0\n\
KF3 1 1 5 0 25 0 0 0 0 0 0 0 2 0 0 0 0 0 15 0 0 0 0 0 0 0 0 0 0 0 0 0 0 15 0 0 0 0 0 0 0 0 0\n\
KF3 1 2 5 0 25 25 0 0 0 0 0 0 3 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KF3 1 3 5 0 25 25 0 0 0 0 0 0 2 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KF3 2 0 3094 351 15470 0 99 1130 245 0 0 0 2 0 0 0 0 0 9265 0 0 0 0 0 0 0 0 0 0 0 0 0 0 9265 0 0 0 0 0 0 0 0 0\n\
KF3 2 1 2543 299 12715 0 148 905 370 0 0 0 2 0 0 0 0 0 11002 0 0 0 0 0 0 0 0 0 0 0 0 0 0 11002 0 0 0 0 0 0 0 0 0\n\
KF3 2 2 3742 437 18710 14970 197 1340 495 0 0 0 3 0 0 1724 13116 13116 15849 14970 14970 10462 14970 14970 14970 1724 1724 0 1724 1724 0 13116 13116 15849 1724 1724 0 1724 1724 0 1724 1724 0\n\
KF3 2 3 3891 460 19455 18715 246 1370 620 0 0 0 3 0 0 2373 16712 16712 16946 18715 18715 13092 18715 18715 18715 2373 2373 0 2373 2373 0 16712 16712 16946 2373 2373 0 2373 2373 0 2373 2373 0\n\
KF3 3 0 885618 188613 4428090 0 42196 668541 74905 0 0 0 2 0 0 0 0 0 3783158 0 0 0 0 0 0 0 0 0 0 0 0 0 0 3783158 0 0 0 0 0 0 0 0 0\n\
KF3 3 1 1068016 228892 5340080 0 64571 796720 134805 0 0 0 2 0 0 0 0 0 5173977 0 0 0 0 0 0 0 0 0 0 0 0 0 0 5173977 0 0 0 0 0 0 0 0 0\n\
KF3 3 2 1665004 357011 8325020 5864165 104556 1233377 223455 0 0 0 3 0 0 507003 5333364 5333364 7951618 5864165 5864165 4481658 5864165 5864165 5864165 507003 507003 0 507003 507003 0 5333364 5333364 7951618 507003 507003 0 507003 507003 0 507003 507003 0\n\
KF3 3 3 2326572 498392 11632860 10967505 141571 1726053 314605 0 0 0 3 0 0 870431 10186388 10186388 11363419 10967505 10967505 8745782 10967505 10967505 10967505 870431 870431 0 870431 870431 0 10186388 10186388 11363419 870431 870431 0 870431 870431 0 870431 870431 0\n\
KF9 1 0 10 0 50 0 0 0 0 0 0 0 1 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0\n\
KF9 1 1 39 0 195 0 0 0 0 0 0 0 3 0 0 0 0 0 117 0 0 0 0 0 0 0 0 0 0 0 0 0 0 117 0 0 0 0 0 0 0 0 0\n\
KF9 1 2 117 0 585 585 0 0 0 0 0 0 5 0 0 117 468 468 359 585 585 252 585 585 585 117 117 0 117 117 0 468 468 359 117 117 0 117 117 0 117 117 0\n\
KF9 2 0 3094 351 15470 0 99 1130 245 0 0 0 2 0 0 0 0 0 9265 0 0 0 0 0 0 0 0 0 0 0 0 0 0 9265 0 0 0 0 0 0 0 0 0\n\
KF9 2 1 15639 1733 78195 0 318 5619 620 0 0 0 3 0 0 0 0 0 70332 0 0 0 0 0 0 0 0 0 0 0 0 0 0 70332 0 0 0 0 0 0 0 0 0\n\
KF9 2 2 78888 8567 394440 275850 975 27900 1745 0 0 0 5 0 0 35820 247356 247356 363767 275850 275850 167166 275850 275850 275850 35820 35820 0 35820 35820 0 247356 247356 363767 35820 35820 0 35820 35820 0 35820 35820 0\n\
KFW 1 0 10 0 50 0 0 0 0 0 0 0 1 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0\n\
KFW 1 1 5 0 25 0 0 0 0 0 0 0 2 0 0 0 0 0 15 0 0 0 0 0 0 0 0 0 0 0 0 0 0 15 0 0 0 0 0 0 0 0 0\n\
KFW 1 2 5 0 25 25 0 0 0 0 0 0 3 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KFW 1 3 5 0 25 25 0 0 0 0 0 0 2 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KFW 1 4 5 0 25 25 0 0 0 0 0 0 2 0 0 5 20 20 15 25 25 11 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KFW 1 5 5 0 25 25 0 0 0 0 0 0 3 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KFW 1 6 5 0 25 25 0 0 0 0 0 0 2 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KFW 1 7 5 0 25 25 0 0 0 0 0 0 2 0 0 5 20 20 15 25 25 11 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KFW 1 8 5 0 25 25 0 0 0 0 0 0 3 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KFW 1 9 5 0 25 25 0 0 0 0 0 0 2 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KFW 1 10 5 0 25 25 0 0 0 0 0 1 2 0 0 4 16 16 12 25 25 11 25 25 25 4 4 0 4 4 0 16 16 12 4 4 0 4 4 0 4 4 0\n\
KFW 1 11 5 0 25 25 0 0 0 0 0 5 3 0 0 0 0 0 0 25 25 12 25 25 25 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n\
KFW 1 12 5 0 25 25 0 0 0 0 0 5 2 0 0 0 0 0 0 25 25 12 25 25 25 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n\
KFW 1 13 5 0 25 25 0 0 0 0 0 5 2 0 0 0 0 0 0 25 25 11 25 25 25 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n\
KFW 1 14 5 0 25 25 0 0 0 0 0 5 3 0 0 0 0 0 0 25 25 12 25 25 25 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n\
KG 1 0 10 0 50 0 0 0 0 0 0 0 1 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0\n\
KG 1 1 39 0 195 0 0 0 0 0 0 0 3 0 0 0 0 0 117 0 0 0 0 0 0 0 0 0 0 0 0 0 0 117 0 0 0 0 0 0 0 0 0\n\
KG 1 2 117 0 585 585 0 0 0 0 0 0 5 0 0 117 468 468 351 585 585 252 585 585 585 117 117 0 117 117 0 468 468 351 117 117 0 117 117 0 117 117 0\n\
KG 1 3 351 0 1755 1755 0 0 0 0 0 0 4 0 0 351 1404 1404 1053 1755 1755 756 1755 1755 1755 351 351 0 351 351 0 1404 1404 1053 351 351 0 351 351 0 351 351 0\n\
KG 2 0 4015 610 20075 0 0 0 925 0 450 0 2 0 0 0 0 0 10824 0 0 0 0 0 0 0 0 0 0 0 0 0 0 10824 0 0 0 0 0 36 0 0 0\n\
KG 2 1 22092 3648 110460 0 0 0 3120 0 3420 0 3 0 0 0 0 0 88296 0 0 0 0 0 0 0 0 0 0 0 0 0 0 88296 0 0 0 0 0 630 0 0 0\n\
KG 2 2 98892 14778 494460 334755 0 0 19035 0 21060 0 5 0 1512 51642 292302 292302 423162 334755 334755 188253 334755 334755 334755 53154 53154 0 53154 53154 0 292302 292302 423162 53154 53154 0 58824 58824 7236 53154 53154 0\n\
KG 2 3 400194 61506 2000970 2000970 0 0 56160 0 104490 0 6 0 19116 265491 1765179 1770093 1756485 2000970 2000970 1261629 2000970 1995300 2000970 265923 284607 0 265923 284607 0 1765179 1770093 1756485 265923 284607 0 309609 346005 49680 265923 284607 0\n\
";

/// 登记 r5 修订 1 修1.七乙类「M = 2 的格按挂载 2 开头那一项故障合起来」（闸取 R2；同一份 `anchors_s5.out`，`to_pinned_text.py mount_two` 摊成）：
/// 列同 [`PINNED_MOUNT_TWO_FAULT_TEXT`]。
const PINNED_MOUNT_TWO_FAULT_TEXT_UNDER_THE_READ_ERROR_RULE: &str = "\
KF none 39931 0 0\n\
KF RN-T1 39931 0 4683\n\
KF RN-TP 35284 36 0\n\
KF SN0-T1 39931 0 6879\n\
KF SN0-TP 28776 310 0\n\
KF SN1-T1 39931 0 6879\n\
KF SN1-TP 28776 310 0\n\
KF SB-T1 39931 0 39931\n\
KF SB-TP 1238 1238 0\n\
KF RI-T1 39931 0 35760\n\
KF RI-TP 5377 1206 0\n\
KF RNSB-T1 39931 0 39931\n\
KF RNSB-TP 1238 1238 0\n\
KF3 none 1365 0 0\n\
KF3 RN-T1 1365 0 255\n\
KF3 RN-TP 1146 36 0\n\
KF3 SN0-T1 1365 0 315\n\
KF3 SN0-TP 910 70 0\n\
KF3 SN1-T1 1365 0 315\n\
KF3 SN1-TP 910 70 0\n\
KF3 SB-T1 1365 0 1365\n\
KF3 SB-TP 182 182 0\n\
KF3 RI-T1 1365 0 1130\n\
KF3 RI-TP 385 150 0\n\
KF3 RNSB-T1 1365 0 1365\n\
KF3 RNSB-TP 182 182 0\n\
KF9 none 10258 0 0\n\
KF9 RN-T1 10258 0 1524\n\
KF9 RN-TP 8761 27 0\n\
KF9 SN0-T1 10258 0 1857\n\
KF9 SN0-TP 7268 105 0\n\
KF9 SN1-T1 10258 0 1857\n\
KF9 SN1-TP 7268 105 0\n\
KF9 SB-T1 10258 0 10258\n\
KF9 SB-TP 393 393 0\n\
KF9 RI-T1 10258 0 8895\n\
KF9 RI-TP 1732 369 0\n\
KF9 RNSB-T1 10258 0 10258\n\
KF9 RNSB-TP 393 393 0\n\
KG none 40921 0 0\n\
KG RN-T1 40271 0 0\n\
KG RN-TP 40271 0 0\n\
KG SN0-T1 40921 0 0\n\
KG SN0-TP 40921 0 0\n\
KG SN1-T1 40921 0 0\n\
KG SN1-TP 40921 0 0\n\
KG SB-T1 40921 0 0\n\
KG SB-TP 40921 0 0\n\
KG RI-T1 38831 0 0\n\
KG RI-TP 38831 0 0\n\
KG RNSB-T1 40271 0 0\n\
KG RNSB-TP 40271 0 0\n\
";

/// 登记 r5 修订 1 修1.七乙类「拒因分计」（闸取 R2；同一份 `anchors_s5.out` 的 `refused_gate`、`refused_acq_device_without_self_verified`、
/// `refused_acq_undecidable` 三个键，`to_pinned_text.py refusals` 摊成）：每行 = 域 M U，然后是闸拒（重读那一遍仍真）、取号那一刻拒（某块盘一份自证过的都没有）、
/// 取号那一刻拒（判不出）。
const PINNED_REFUSALS_BY_REASON_TEXT_UNDER_THE_READ_ERROR_RULE: &str = "\
KF 1 0 0 0 0\n\
KF 1 1 0 0 0\n\
KF 1 2 0 0 0\n\
KF 1 3 0 0 0\n\
KF 2 0 91 8 0\n\
KF 2 1 298 20 0\n\
KF 2 2 919 56 0\n\
KF 2 3 2782 164 0\n\
KF3 1 0 0 0 0\n\
KF3 1 1 0 0 0\n\
KF3 1 2 0 0 0\n\
KF3 1 3 0 0 0\n\
KF3 2 0 91 8 0\n\
KF3 2 1 136 12 0\n\
KF3 2 2 181 16 0\n\
KF3 2 3 226 20 0\n\
KF3 3 0 38920 3276 0\n\
KF3 3 1 59499 5072 0\n\
KF3 3 2 96288 8268 0\n\
KF3 3 3 130347 11224 0\n\
KF9 1 0 0 0 0\n\
KF9 1 1 0 0 0\n\
KF9 1 2 0 0 0\n\
KF9 2 0 91 8 0\n\
KF9 2 1 298 20 0\n\
KF9 2 2 919 56 0\n\
KFW 1 0 0 0 0\n\
KFW 1 1 0 0 0\n\
KFW 1 2 0 0 0\n\
KFW 1 3 0 0 0\n\
KFW 1 4 0 0 0\n\
KFW 1 5 0 0 0\n\
KFW 1 6 0 0 0\n\
KFW 1 7 0 0 0\n\
KFW 1 8 0 0 0\n\
KFW 1 9 0 0 0\n\
KFW 1 10 0 0 0\n\
KFW 1 11 0 0 0\n\
KFW 1 12 0 0 0\n\
KFW 1 13 0 0 0\n\
KFW 1 14 0 0 0\n\
KG 1 0 0 0 0\n\
KG 1 1 0 0 0\n\
KG 1 2 0 0 0\n\
KG 1 3 0 0 0\n\
KG 2 0 0 0 0\n\
KG 2 1 0 0 0\n\
KG 2 2 0 0 0\n\
KG 2 3 0 0 0\n\
";

/// 一行钉值（`PINNED_FAULT_CELLS_TEXT` 里的一行）：一个 (域, M, U) 格上生成器一侧的计数与假结局、本地择根序的读数。
#[derive(Clone, Debug)]
pub(super) struct PinnedFaultCell {
    pub(super) domain: FaultDomain,
    pub(super) mounts: usize,
    pub(super) user_publishes: usize,
    pub(super) histories: u64,
    pub(super) histories_with_an_uncovered_fault: u64,
    pub(super) judged_states: u64,
    pub(super) confirmed_states: u64,
    pub(super) refused_mounts: u64,
    pub(super) reread_mounts: u64,
    pub(super) states_with_a_torn_system_slot: u64,
    pub(super) states_with_same_txg_different_instance_roots: u64,
    pub(super) states_with_higher_instance_lower_txg_roots: u64,
    pub(super) states_without_a_readable_root: u64,
    pub(super) above_newest_root_maximum: usize,
    /// 条款的理想恢复丢了的被判态：因在飞上限、罩着的其余、不罩。
    ideal_loss: [u64; 3],
    pub(super) kf1: [u64; 3],
    pub(super) kf2: [u64; 4],
    pub(super) kf4: [u64; 2],
    /// 本地择根序 today、v1823、v1824、v1825、v1826、v1822 各自的 KJ1、KJ1T、KJ2。
    pub(super) local_rules: [[u64; 3]; 6],
}

fn parse_the_pinned_fault_cells(text: &str) -> Vec<PinnedFaultCell> {
    text.lines()
        .map(|line| {
            let fields: Vec<&str> = line.split_whitespace().collect();
            let domain = *FaultDomain::ALL
                .iter()
                .find(|domain| domain.label() == fields[0])
                .expect("钉值表里的域名在六个域之内");
            let numbers: Vec<u64> = fields[3..]
                .iter()
                .map(|field| field.parse::<u64>().expect("钉值表里的数是非负整数"))
                .collect();
            assert_eq!(numbers.len(), 41, "钉值表每行 41 个数：{line}");
            let mut local_rules = [[0u64; 3]; 6];
            for (rule_index, rule) in local_rules.iter_mut().enumerate() {
                rule.copy_from_slice(&numbers[23 + 3 * rule_index..26 + 3 * rule_index]);
            }
            PinnedFaultCell {
                domain,
                mounts: fields[1].parse().expect("M 是非负整数"),
                user_publishes: fields[2].parse().expect("U 是非负整数"),
                histories: numbers[0],
                histories_with_an_uncovered_fault: numbers[1],
                judged_states: numbers[2],
                confirmed_states: numbers[3],
                refused_mounts: numbers[4],
                reread_mounts: numbers[5],
                states_with_a_torn_system_slot: numbers[6],
                states_with_same_txg_different_instance_roots: numbers[7],
                states_with_higher_instance_lower_txg_roots: numbers[8],
                states_without_a_readable_root: numbers[9],
                above_newest_root_maximum: usize::try_from(numbers[10])
                    .expect("记录条数装得进 usize"),
                ideal_loss: [numbers[11], numbers[12], numbers[13]],
                kf1: [numbers[14], numbers[15], numbers[16]],
                kf2: [numbers[17], numbers[18], numbers[19], numbers[20]],
                kf4: [numbers[21], numbers[22]],
                local_rules,
            }
        })
        .collect()
}

fn pinned_fault_cells() -> &'static Vec<PinnedFaultCell> {
    static PINNED: std::sync::OnceLock<Vec<PinnedFaultCell>> = std::sync::OnceLock::new();
    PINNED.get_or_init(|| parse_the_pinned_fault_cells(PINNED_FAULT_CELLS_TEXT))
}

/// 闸取 `witness_gate_rule` 时那一张钉值表：取甲是 [`PINNED_FAULT_CELLS_TEXT`]，R2 是 [`PINNED_FAULT_CELLS_TEXT_UNDER_THE_READ_ERROR_RULE`]。
fn pinned_fault_cells_under_the_gate(
    witness_gate_rule: WitnessGateRule,
) -> &'static Vec<PinnedFaultCell> {
    match witness_gate_rule {
        WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable => pinned_fault_cells(),
        WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots => {
            static PINNED_UNDER_THE_READ_ERROR_RULE: std::sync::OnceLock<Vec<PinnedFaultCell>> =
                std::sync::OnceLock::new();
            PINNED_UNDER_THE_READ_ERROR_RULE.get_or_init(|| {
                parse_the_pinned_fault_cells(PINNED_FAULT_CELLS_TEXT_UNDER_THE_READ_ERROR_RULE)
            })
        }
    }
}

pub(super) fn pinned_fault_cell_under_the_gate(
    witness_gate_rule: WitnessGateRule,
    domain: FaultDomain,
    cell: RootChoiceCell,
) -> Option<&'static PinnedFaultCell> {
    pinned_fault_cells_under_the_gate(witness_gate_rule)
        .iter()
        .find(|pinned| {
            pinned.domain == domain
                && pinned.mounts == cell.mounts
                && pinned.user_publishes == cell.user_publishes
        })
}

/// R2 那一张拒因钉值表里 (域, M, U) 那一格的三栏：闸拒、取号那一刻拒（某块盘一份自证过的都没有）、取号那一刻拒（判不出）。
fn pinned_refusals_by_reason_under_the_read_error_rule(
    domain: FaultDomain,
    cell: RootChoiceCell,
) -> Option<[u64; 3]> {
    PINNED_REFUSALS_BY_REASON_TEXT_UNDER_THE_READ_ERROR_RULE
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<&str>>())
        .find(|fields| {
            fields[0] == domain.label()
                && fields[1].parse::<usize>().ok() == Some(cell.mounts)
                && fields[2].parse::<usize>().ok() == Some(cell.user_publishes)
        })
        .map(|fields| {
            let numbers: Vec<u64> = fields[3..]
                .iter()
                .map(|field| field.parse::<u64>().expect("拒因钉值表里的数是非负整数"))
                .collect();
            assert_eq!(numbers.len(), 3, "拒因钉值表每行三栏：{fields:?}");
            [numbers[0], numbers[1], numbers[2]]
        })
}

/// 本地择根序 today 在 KG 上、被判时无故障的态里的 KJ1T 合计（登记 5.6「C331 的判别力自证」：本地理想恢复算出的数是 648）。
pub(super) const PINNED_KG_TODAY_DESCENT_VIOLATIONS_WITHOUT_A_JUDGED_FAULT: u64 = 648;

/// 每个跑到的格，装置数出的生成器一侧的计数与本地择根序的读数等于独立生成器的数；不符走作废条款 TV4（这些量只看生成器与真相，与被测实现无关）。
/// 条款的理想恢复在合法域里丢了的被判态（因在飞上限、罩着的其余）不是 0 时不当 TV4 停，交回登记的 TF9（条款或实现可能错）。
/// 闸取甲（只给单测用，跑法里一律带闸的取法）。
#[cfg(test)]
fn assert_the_pinned_fault_cell(
    domain: FaultDomain,
    cell: RootChoiceCell,
    tally: &FaultTally,
) -> Vec<String> {
    assert_the_pinned_fault_cell_under_the_gate(
        WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable,
        domain,
        cell,
        tally,
    )
}

/// 每个跑到的格的钉值比对（同上一个函数的说法），按闸的取法选表：取甲照 r4 登记第七节乙类，R2 照 r5 登记修订 1 修1.七乙类，另核拒因三栏（各栏之和等于被拒的挂载数）。
pub(super) fn assert_the_pinned_fault_cell_under_the_gate(
    witness_gate_rule: WitnessGateRule,
    domain: FaultDomain,
    cell: RootChoiceCell,
    tally: &FaultTally,
) -> Vec<String> {
    assert_eq!(
        tally.refusals_by_reason.iter().sum::<u64>(),
        tally.refused_mounts,
        "拒因各栏之和等于被拒的挂载数（{} M = {} U = {}）",
        domain.label(),
        cell.mounts,
        cell.user_publishes
    );
    match witness_gate_rule {
        WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable => {}
        WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots => {
            let pinned_refusals = pinned_refusals_by_reason_under_the_read_error_rule(domain, cell)
                .expect("每个跑到的格都在登记 r5 修订 1 修1.七乙类的拒因表里");
            assert_eq!(
                [
                    tally.refusals_by_reason[MountRefusalReason::TheGateAfterTheOneReread.index()],
                    tally.refusals_by_reason
                        [MountRefusalReason::AcquisitionDeviceWithoutASelfVerifiedSlot.index()],
                    tally.refusals_by_reason
                        [MountRefusalReason::AcquisitionWitnessUndecidable.index()],
                ],
                pinned_refusals,
                "PV12：{} M = {} U = {} 拒因三栏（闸、取号那一刻 (b)、取号那一刻 (a)）",
                domain.label(),
                cell.mounts,
                cell.user_publishes
            );
        }
    }
    let pinned = pinned_fault_cell_under_the_gate(witness_gate_rule, domain, cell)
        .expect("每个跑到的格都在登记第七节乙类的表里");
    let place = format!(
        "域 {} M = {} U = {}",
        domain.label(),
        cell.mounts,
        cell.user_publishes
    );
    assert_eq!(tally.histories, pinned.histories, "TV4：{place} 历史数");
    assert_eq!(
        tally.histories_with_an_uncovered_fault, pinned.histories_with_an_uncovered_fault,
        "TV4：{place} 历史里有不罩故障的历史数"
    );
    assert_eq!(
        tally.judged_states, pinned.judged_states,
        "TV4：{place} 被判态数"
    );
    assert_eq!(
        tally.confirmed_states, pinned.confirmed_states,
        "TV4：{place} 有已确认的被判态数"
    );
    assert_eq!(
        tally.refused_mounts, pinned.refused_mounts,
        "TV4：{place} 被拒的挂载数"
    );
    assert_eq!(
        tally.reread_mounts, pinned.reread_mounts,
        "TV4：{place} 走了重读那一遍的挂载数"
    );
    assert_eq!(
        tally.states_with_a_torn_system_slot, pinned.states_with_a_torn_system_slot,
        "TV4：{place} 带撕裂系统配置槽的被判态数"
    );
    assert_eq!(
        tally.states_with_same_txg_different_instance_roots,
        pinned.states_with_same_txg_different_instance_roots,
        "TV4：{place} 根环里同 txg 不同实例同在的被判态数"
    );
    assert_eq!(
        tally.states_with_higher_instance_lower_txg_roots,
        pinned.states_with_higher_instance_lower_txg_roots,
        "TV4：{place} 实例代号更高而 txg 更低同在的被判态数"
    );
    assert_eq!(
        tally.states_without_a_readable_root, pinned.states_without_a_readable_root,
        "TV4：{place} 读视图里一条根都没有的被判态数"
    );
    assert_eq!(
        tally.above_newest_root_maximum, pinned.above_newest_root_maximum,
        "TV4：{place} 读视图里版本最大的根之上同实例的记录条数的最大值"
    );
    assert_eq!(
        tally.ideal_loss_uncovered, pinned.ideal_loss[2],
        "TV4：{place} 条款的理想恢复丢了的被判态里「不罩」那一类"
    );
    let mut findings = Vec::new();
    if domain.is_a_legal_history() {
        if tally.ideal_loss_by_limit != pinned.ideal_loss[0]
            || tally.ideal_loss_covered_other != pinned.ideal_loss[1]
        {
            findings.push(format!(
                "TF9 {place} 条款的理想恢复丢了已确认发布的被判态：因在飞上限 {}、罩着的其余 {}（登记第七节甲类锚点都是 0）",
                tally.ideal_loss_by_limit, tally.ideal_loss_covered_other
            ));
        }
    } else {
        assert_eq!(
            tally.ideal_loss_by_limit, pinned.ideal_loss[0],
            "TV4：{place} 条款的理想恢复丢了的被判态里「因在飞上限」那一类"
        );
        assert_eq!(
            tally.ideal_loss_covered_other, pinned.ideal_loss[1],
            "TV4：{place} 条款的理想恢复丢了的被判态里「罩着的其余」那一类"
        );
    }
    for (rule_index, rule) in FaultLocalRule::ALL.iter().take(6).enumerate() {
        assert_eq!(
            tally.local_rules[rule_index],
            pinned.local_rules[rule_index],
            "TV4：{place} 本地择根序 {} 的 KJ1 / KJ1T / KJ2 读数",
            rule.label()
        );
    }
    findings
}

/// M = 2 的格合起来、按挂载 2 开头那项故障报的历史数、被拒数、重读数等于登记第七节那张表（TV4）；按闸的取法选表：取甲照 r4 登记第七节，
/// R2 照 r5 登记修订 1 修1.七乙类。
pub(super) fn assert_the_pinned_mount_two_table_under_the_gate(
    witness_gate_rule: WitnessGateRule,
    domain: FaultDomain,
    merged_over_the_mount_two_cells: &FaultTally,
) {
    let pinned_text = match witness_gate_rule {
        WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable => PINNED_MOUNT_TWO_FAULT_TEXT,
        WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots => {
            PINNED_MOUNT_TWO_FAULT_TEXT_UNDER_THE_READ_ERROR_RULE
        }
    };
    let mut checked_rows = 0;
    for line in pinned_text.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields[0] != domain.label() {
            continue;
        }
        let menu_index = MOUNT_FAULT_MENU_LABELS
            .iter()
            .position(|label| *label == fields[1])
            .expect("钉值表里的故障名在菜单之内");
        let pinned: Vec<u64> = fields[2..]
            .iter()
            .map(|field| field.parse::<u64>().expect("钉值表里的数是非负整数"))
            .collect();
        let count = merged_over_the_mount_two_cells.mount_two[menu_index];
        assert_eq!(
            [count.histories, count.refused, count.reread],
            [pinned[0], pinned[1], pinned[2]],
            "TV4：域 {} M = 2 挂载 2 开头故障 {} 的历史数 / 被拒数 / 重读数",
            domain.label(),
            fields[1]
        );
        checked_rows += 1;
    }
    assert_eq!(
        checked_rows, MOUNT_FAULT_MENU_LENGTH,
        "每个域的 M = 2 表 13 行"
    );
}

#[cfg(test)]
mod root_choice_fault_tests;
