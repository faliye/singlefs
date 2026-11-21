use super::*;
use crate::crash_judge_gpu::recovery_kernel_source;

/// 展开之后的大小按调用点逐处累加：`leaf` 自己几个表达式、`middle` 调它两处、入口调 `middle` 三处，入口展开之后正好是
/// 入口自己 + 3 ×（middle 自己 + 2 × leaf 自己）；份数最多的是 `leaf`（6 份）。
#[test]
fn inlined_size_adds_one_copy_of_the_callee_at_every_call_site() {
    let source = "
            @group(0) @binding(0) var<storage, read_write> out: array<u32>;
            fn leaf(x: u32) -> u32 { return x * 3u + 1u; }
            fn middle(x: u32) -> u32 { return leaf(x) + leaf(x + 2u); }
            @compute @workgroup_size(1) fn main() {
                out[0] = middle(1u);
                out[1] = middle(2u);
                out[2] = middle(3u);
            }";
    let module = wgpu::naga::front::wgsl::parse_str(source).expect("解析得了");
    let own_of = |name: &str| -> u64 {
        let (_, function) = module
            .functions
            .iter()
            .find(|(_, function)| function.name.as_deref() == Some(name))
            .expect("有这个函数");
        u64::try_from(function.expressions.len()).expect("装得进 u64")
    };
    let entry_own =
        u64::try_from(module.entry_points[0].function.expressions.len()).expect("装得进 u64");
    let size = super::kernel_inlined_size(source);
    assert_eq!(
        size.expressions,
        entry_own + 3 * (own_of("middle") + 2 * own_of("leaf")),
        "入口自己 + 三份 middle，每份 middle 带两份 leaf"
    );
    assert_eq!(
        size.largest_contributors[0],
        ("leaf".to_string(), 6, own_of("leaf")),
        "leaf 被展开 6 份，占得最多"
    );
}

/// 判器的全部管线（三个内核，扫描内核源码另有扫描那两组、记账那四趟、引用与快照那一截、扫描重建那一截几个入口）都在预算之内；打各自展开之后的大小。
#[test]
fn every_kernel_stays_within_the_inlined_size_budget() {
    for (name, source, entry_point) in [
        ("recovery", recovery_kernel_source(), KERNEL_ENTRY_POINT),
        (
            "checker-scan",
            crate::crash_judge_gpu::checker_scan_kernel_source(),
            KERNEL_ENTRY_POINT,
        ),
        (
            "checker-scan-allocation-records-disjoint",
            crate::crash_judge_gpu::checker_scan_kernel_source(),
            CHECKER_SCAN_ALLOCATION_RECORDS_DISJOINT_ENTRY_POINT,
        ),
        (
            "checker-scan-allocation-generations-through-the-rollback-floor-raises",
            crate::crash_judge_gpu::checker_scan_kernel_source(),
            CHECKER_SCAN_ALLOCATION_GENERATIONS_THROUGH_THE_ROLLBACK_FLOOR_RAISES_ENTRY_POINT,
        ),
        (
            "checker-accounting-head",
            crate::crash_judge_gpu::checker_scan_kernel_source(),
            CHECKER_ACCOUNTING_HEAD_ENTRY_POINT,
        ),
        (
            "checker-accounting-slice",
            crate::crash_judge_gpu::checker_scan_kernel_source(),
            CHECKER_ACCOUNTING_SLICE_ENTRY_POINT,
        ),
        (
            "checker-accounting-decide",
            crate::crash_judge_gpu::checker_scan_kernel_source(),
            CHECKER_ACCOUNTING_DECIDE_ENTRY_POINT,
        ),
        (
            "checker-accounting-final",
            crate::crash_judge_gpu::checker_scan_kernel_source(),
            CHECKER_ACCOUNTING_FINAL_ENTRY_POINT,
        ),
        (
            "checker-references-and-snapshots",
            crate::crash_judge_gpu::checker_scan_kernel_source(),
            CHECKER_REFERENCES_AND_SNAPSHOTS_ENTRY_POINT,
        ),
        (
            "checker-scan-rebuild",
            crate::crash_judge_gpu::checker_scan_kernel_source(),
            CHECKER_SCAN_REBUILD_ENTRY_POINT,
        ),
        (
            "checker-walk",
            crate::crash_judge_gpu::checker_walk_kernel_source(),
            KERNEL_ENTRY_POINT,
        ),
    ] {
        let size = super::kernel_inlined_size_of_the_entry_point(&source, entry_point);
        println!(
            "GPU_KERNEL_INLINED kernel={name} expressions={} budget={} largest={:?}",
            size.expressions,
            super::LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL,
            size.largest_contributors
        );
        super::refuse_a_kernel_entry_point_too_large_for_the_driver(name, &source, entry_point);
    }
}

/// 在扫描内核里把这七个调用方各自的多处调用并成了一处（调用方，它里面那个被调函数）。
const SCAN_KERNEL_JUDGES_MERGED_INTO_ONE_CALL_SITE: [(&str, &str); 7] = [
    (
        "judge_release_generation_and_tree_table_birth",
        "r_references_of_root",
    ),
    (
        "judge_allocation_records_disjoint",
        "judge_allocation_records_disjoint_of_a_version",
    ),
    (
        "judge_allocation_generations_against_unit_births",
        "judge_allocation_generations_under_tree_table",
    ),
    ("accounting_head", "c_quarantine_exempted"),
    (
        "judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark",
        "judge_first_data_unit_of_a_version",
    ),
    (
        "judge_deletion_state_of_the_versions",
        "d_judge_deletion_state_of_a_version",
    ),
    (
        "judge_tombstones_and_the_lineage_of_the_versions",
        "d_judge_tombstones_and_the_lineage_of_a_version",
    ),
];

/// 一份内核源码里每个（调用方, 被调方）有几个调用点：naga 解析之后逐个函数数调用语句，入口函数记成 `entry:<名字>`。
/// 与 [`super::kernel_inlined_size`] 各数各的（它不交回调用点表），这一份只给钉调用点数的用例用。
fn call_sites_by_caller_and_callee(
    source: &str,
) -> std::collections::BTreeMap<(String, String), u64> {
    use wgpu::naga::{Block, Function, Handle, Statement};
    #[allow(
        clippy::wildcard_enum_match_arm,
        reason = "naga 的语句是外部的枚举，只有这四种里还套着语句块，其余一律不含调用"
    )]
    fn callees_in(block: &Block, callees: &mut Vec<Handle<Function>>) {
        for statement in block.iter() {
            match statement {
                Statement::Call { function, .. } => callees.push(*function),
                Statement::Block(nested) => callees_in(nested, callees),
                Statement::If { accept, reject, .. } => {
                    callees_in(accept, callees);
                    callees_in(reject, callees);
                }
                Statement::Switch { cases, .. } => {
                    for switch_case in cases {
                        callees_in(&switch_case.body, callees);
                    }
                }
                Statement::Loop {
                    body, continuing, ..
                } => {
                    callees_in(body, callees);
                    callees_in(continuing, callees);
                }
                _ => {}
            }
        }
    }
    let module = wgpu::naga::front::wgsl::parse_str(source)
        .unwrap_or_else(|error| panic!("内核解析失败：\n{}", error.emit_to_string(source)));
    let mut sites = std::collections::BTreeMap::new();
    let callers = module
        .functions
        .iter()
        .map(|(_, function)| (function.name.clone().unwrap_or_default(), &function.body))
        .chain(
            module
                .entry_points
                .iter()
                .map(|entry| (format!("entry:{}", entry.name), &entry.function.body)),
        );
    for (caller, body) in callers {
        let mut callees = Vec::new();
        callees_in(body, &mut callees);
        for callee in callees {
            *sites
                .entry((
                    caller.clone(),
                    module.functions[callee].name.clone().unwrap_or_default(),
                ))
                .or_insert(0) += 1;
        }
    }
    sites
}

/// 并成一处调用的七个函数：每个被调函数在整个扫描内核里只有一个调用点、就在它的调用方里，调用方自己也只被调一处
/// （数是在主工作区的扫描内核上现量的）。别处再多调一次（哪怕总数还在预算里，例：`judge_accounting` 加回一处只到 392 383），
/// 先红在这里，不等上卡才撞展开预算；真要多一处，先量展开之后的大小，再改这里的钉值。
#[test]
fn each_judge_merged_into_one_call_site_keeps_that_single_call_site_in_the_scan_kernel() {
    const CALLEE_SITES_INSIDE_THE_CALLER: u64 = 1;
    const CALLEE_SITES_IN_THE_WHOLE_KERNEL: u64 = 1;
    const CALLER_SITES_IN_THE_WHOLE_KERNEL: u64 = 1;
    let sites =
        call_sites_by_caller_and_callee(&crate::crash_judge_gpu::checker_scan_kernel_source());
    let sites_calling = |callee: &str| -> u64 {
        sites
            .iter()
            .filter(|((_, called), _)| called == callee)
            .map(|(_, count)| *count)
            .sum()
    };
    for (caller, callee) in SCAN_KERNEL_JUDGES_MERGED_INTO_ONE_CALL_SITE {
        assert_eq!(
            sites
                .get(&(caller.to_string(), callee.to_string()))
                .copied()
                .unwrap_or(0),
            CALLEE_SITES_INSIDE_THE_CALLER,
            "{caller} 里调 {callee} 的地方只许一处"
        );
        assert_eq!(
            sites_calling(callee),
            CALLEE_SITES_IN_THE_WHOLE_KERNEL,
            "整个扫描内核里调 {callee} 的地方只许一处（就是 {caller} 里那一处）"
        );
        assert_eq!(
            sites_calling(caller),
            CALLER_SITES_IN_THE_WHOLE_KERNEL,
            "整个扫描内核里调 {caller} 的地方只许一处"
        );
    }
}

/// 每个入口此刻展开之后的表达式数与离预算的余量（用 naga 解析、校验，数法同 `kernel_inlined_size`：调用点展开之后数表达式；预算见 `LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL`，五份以上 WGSL 照库里的拼法拼）。
/// 改了哪个内核的源码，这里先红，量过新数、确认余量还够之后再改钉值，免得余量悄悄耗光、等上卡时才撞预算。
/// 全部入口逐个量完再判：对不上的一次列全（名字、量到的数、钉值、离预算的余量、钉的余量），一趟就拿得到重钉要的全部数；
/// 每个入口另打一行 `GPU_KERNEL_INLINED_PIN`，对得上的也打。
#[test]
fn each_kernel_keeps_its_pinned_inlined_size_and_margin_under_the_budget() {
    let budget = super::LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL;
    let mut kernels_off_their_pins: Vec<String> = Vec::new();
    for (name, source, entry_point, pinned_expressions, pinned_margin) in [
        (
            "recovery",
            recovery_kernel_source(),
            KERNEL_ENTRY_POINT,
            288_999,
            111_001,
        ),
        (
            "checker-scan",
            crate::crash_judge_gpu::checker_scan_kernel_source(),
            KERNEL_ENTRY_POINT,
            7_885,
            392_115,
        ),
        (
            "checker-scan-allocation-records-disjoint",
            crate::crash_judge_gpu::checker_scan_kernel_source(),
            CHECKER_SCAN_ALLOCATION_RECORDS_DISJOINT_ENTRY_POINT,
            27_914,
            372_086,
        ),
        (
            "checker-scan-allocation-generations-through-the-rollback-floor-raises",
            crate::crash_judge_gpu::checker_scan_kernel_source(),
            CHECKER_SCAN_ALLOCATION_GENERATIONS_THROUGH_THE_ROLLBACK_FLOOR_RAISES_ENTRY_POINT,
            278_009,
            121_991,
        ),
        (
            "checker-accounting-head",
            crate::crash_judge_gpu::checker_scan_kernel_source(),
            CHECKER_ACCOUNTING_HEAD_ENTRY_POINT,
            34_467,
            365_533,
        ),
        (
            "checker-accounting-slice",
            crate::crash_judge_gpu::checker_scan_kernel_source(),
            CHECKER_ACCOUNTING_SLICE_ENTRY_POINT,
            7_350,
            392_650,
        ),
        (
            "checker-accounting-decide",
            crate::crash_judge_gpu::checker_scan_kernel_source(),
            CHECKER_ACCOUNTING_DECIDE_ENTRY_POINT,
            3_077,
            396_923,
        ),
        (
            "checker-accounting-final",
            crate::crash_judge_gpu::checker_scan_kernel_source(),
            CHECKER_ACCOUNTING_FINAL_ENTRY_POINT,
            51_158,
            348_842,
        ),
        (
            "checker-references-and-snapshots",
            crate::crash_judge_gpu::checker_scan_kernel_source(),
            CHECKER_REFERENCES_AND_SNAPSHOTS_ENTRY_POINT,
            191_277,
            208_723,
        ),
        (
            "checker-scan-rebuild",
            crate::crash_judge_gpu::checker_scan_kernel_source(),
            CHECKER_SCAN_REBUILD_ENTRY_POINT,
            69_419,
            330_581,
        ),
        (
            "checker-walk",
            crate::crash_judge_gpu::checker_walk_kernel_source(),
            KERNEL_ENTRY_POINT,
            370_376,
            29_624,
        ),
    ] {
        let size = super::kernel_inlined_size_of_the_entry_point(&source, entry_point);
        let measured_margin = budget.checked_sub(size.expressions);
        let measured_margin_text = measured_margin.map_or_else(
            || "没有（超了预算）".to_string(),
            |remaining_expressions| remaining_expressions.to_string(),
        );
        println!(
            "GPU_KERNEL_INLINED_PIN kernel={name} expressions={} pinned_expressions={pinned_expressions} \
                 margin={measured_margin_text} pinned_margin={pinned_margin} budget={budget}",
            size.expressions
        );
        if size.expressions != pinned_expressions || measured_margin != Some(pinned_margin) {
            kernels_off_their_pins.push(format!(
                "内核 {name}：量到 {} 个表达式、钉值 {pinned_expressions}；离预算 {budget} 的余量 {measured_margin_text}、钉的余量 {pinned_margin}",
                size.expressions
            ));
        }
    }
    assert!(
        kernels_off_their_pins.is_empty(),
        "{} 个内核展开之后的表达式数或离预算的余量与钉值对不上（全部入口都量完了，下面逐个列全）：\
             量过、确认余量还够之后改这里的钉值\n{}",
        kernels_off_their_pins.len(),
        kernels_off_their_pins.join("\n")
    );
}

/// 走读内核里「走一遍根」回到六个调用点（第二台 580 版驱动 48G 里编不完的那个形态），编之前就被拒。
#[test]
fn the_walk_kernel_with_the_root_walk_called_from_six_sites_is_refused() {
    const CALL: &str = "w_walk_one(walk_kind, walk_base, walk_is_newest, fsid_low);";
    let source = crate::crash_judge_gpu::checker_walk_kernel_source();
    assert_eq!(
        source.matches(CALL).count(),
        1,
        "走读内核里「走一遍根」只有一个调用点"
    );
    let six_sites = source.replacen(CALL, &CALL.repeat(6), 1);
    let size = super::kernel_inlined_size(&six_sites);
    println!(
        "GPU_KERNEL_INLINED kernel=checker-walk-six-sites expressions={}",
        size.expressions
    );
    let refused = std::panic::catch_unwind(|| {
        super::refuse_a_kernel_too_large_for_the_driver("checker-walk-six-sites", &six_sites);
    });
    assert!(
        refused.is_err(),
        "六个调用点展开之后 {} 个表达式，该被拒",
        size.expressions
    );
}
