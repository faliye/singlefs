//! 一个内核全部调用逐处展开之后多大（naga 中间表示里的表达式个数）与驱动编得过的预算：编内核之前超预算的当场拒。

use super::*;

/// 一个内核全部调用逐处展开之后最多多少个表达式（naga 中间表示里的表达式个数，[`kernel_inlined_size`]）。
/// GPU 程序没有真正的函数调用，驱动的编译器把每个调用点原地展开成被调函数的一份拷贝，编译的用时与内存随展开后的大小涨得比正比快：
/// 走读内核的「走一遍根」有六个调用点时（展开之后 98 万个表达式），第二台 580 版驱动在 48G 里编不完；改成一个调用点之后
/// （22 万）那台 94 秒、3.4 GiB 编完，恢复与扫描两个内核（各 27 万）在那台也编得过。预算取已知编得过的最大那个的一倍半左右，
/// 编不编得过的真实边界在 27 万与 98 万之间，没量过。内核超出预算先压展开量，不抬预算。
pub const LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL: u64 = 400_000;

/// 一个内核展开之后多大、谁占得最多。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelInlinedSize {
    /// 入口函数展开之后的表达式个数。
    pub expressions: u64,
    /// 展开后占得最多的几个函数：名字、被展开几份、自己几个表达式，按「份数 × 自己的表达式数」从大到小。
    pub largest_contributors: Vec<(String, u64, u64)>,
}

/// 量一份内核源码全部调用逐处展开之后多大：函数自己的表达式数，加上它每个调用点上被调函数展开后的表达式数（WGSL 不许递归，调用关系无环）。
/// 量的是入口 `main`（[`KERNEL_ENTRY_POINT`]）；一份源码里的别的入口用 [`kernel_inlined_size_of_the_entry_point`]。
///
/// # Panics
/// 源码解析不了（内核源码自己的单测先判这一条）。
#[must_use]
pub fn kernel_inlined_size(source: &str) -> KernelInlinedSize {
    kernel_inlined_size_of_the_entry_point(source, KERNEL_ENTRY_POINT)
}

/// 同 [`kernel_inlined_size`]，量的是名叫 `entry_point` 的那个入口（扫描内核源码里的第二个入口，引用与快照那一截，
/// [`CHECKER_REFERENCES_AND_SNAPSHOTS_ENTRY_POINT`]）：驱动按入口编一条管线，展开预算各量各的。
///
/// # Panics
/// 源码解析不了；源码里没有这个入口。
#[must_use]
pub fn kernel_inlined_size_of_the_entry_point(
    source: &str,
    entry_point: &str,
) -> KernelInlinedSize {
    use std::collections::HashMap;
    use wgpu::naga::{Block, Function, Handle, Statement};
    #[allow(
        clippy::wildcard_enum_match_arm,
        reason = "naga 的语句是外部的枚举，只有这四种里还套着语句块，其余一律不含调用"
    )]
    fn calls_in(block: &Block, calls: &mut Vec<Handle<Function>>) {
        for statement in block.iter() {
            match statement {
                Statement::Call { function, .. } => calls.push(*function),
                Statement::Block(inner) => calls_in(inner, calls),
                Statement::If { accept, reject, .. } => {
                    calls_in(accept, calls);
                    calls_in(reject, calls);
                }
                Statement::Switch { cases, .. } => {
                    for case in cases {
                        calls_in(&case.body, calls);
                    }
                }
                Statement::Loop {
                    body, continuing, ..
                } => {
                    calls_in(body, calls);
                    calls_in(continuing, calls);
                }
                _ => {}
            }
        }
    }
    fn inlined(
        function: Handle<Function>,
        own: &HashMap<Handle<Function>, u64>,
        calls: &HashMap<Handle<Function>, Vec<Handle<Function>>>,
        memo: &mut HashMap<Handle<Function>, u64>,
    ) -> u64 {
        if let Some(size) = memo.get(&function) {
            return *size;
        }
        let size = own[&function]
            + calls[&function]
                .iter()
                .map(|callee| inlined(*callee, own, calls, memo))
                .sum::<u64>();
        memo.insert(function, size);
        size
    }
    let module = wgpu::naga::front::wgsl::parse_str(source)
        .unwrap_or_else(|error| panic!("内核解析失败：\n{}", error.emit_to_string(source)));
    let entry = module
        .entry_points
        .iter()
        .find(|entry| entry.name == entry_point)
        .unwrap_or_else(|| panic!("内核源码里有入口 {entry_point}"));
    let mut own: HashMap<Handle<Function>, u64> = HashMap::new();
    let mut calls: HashMap<Handle<Function>, Vec<Handle<Function>>> = HashMap::new();
    for (handle, function) in module.functions.iter() {
        own.insert(
            handle,
            u64::try_from(function.expressions.len()).expect("装得进 u64"),
        );
        let mut sites = Vec::new();
        calls_in(&function.body, &mut sites);
        calls.insert(handle, sites);
    }
    let mut entry_calls = Vec::new();
    calls_in(&entry.function.body, &mut entry_calls);
    let mut memo = HashMap::new();
    let expressions = u64::try_from(entry.function.expressions.len()).expect("装得进 u64")
        + entry_calls
            .iter()
            .map(|callee| inlined(*callee, &own, &calls, &mut memo))
            .sum::<u64>();
    // 每个函数被展开几份：从入口往下，调用方的份数乘以调用点数，按「先调用方、后被调方」的次序累加
    let mut order: Vec<Handle<Function>> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    fn postorder(
        function: Handle<Function>,
        calls: &HashMap<Handle<Function>, Vec<Handle<Function>>>,
        seen: &mut std::collections::HashSet<Handle<Function>>,
        order: &mut Vec<Handle<Function>>,
    ) {
        if !seen.insert(function) {
            return;
        }
        for callee in &calls[&function] {
            postorder(*callee, calls, seen, order);
        }
        order.push(function);
    }
    for callee in &entry_calls {
        postorder(*callee, &calls, &mut seen, &mut order);
    }
    let mut copies: HashMap<Handle<Function>, u64> = HashMap::new();
    for callee in &entry_calls {
        *copies.entry(*callee).or_insert(0) += 1;
    }
    for function in order.iter().rev() {
        let copies_of_caller = copies.get(function).copied().unwrap_or(0);
        for callee in &calls[function] {
            *copies.entry(*callee).or_insert(0) += copies_of_caller;
        }
    }
    let mut largest_contributors: Vec<(String, u64, u64)> = order
        .iter()
        .map(|function| {
            (
                module.functions[*function].name.clone().unwrap_or_default(),
                copies[function],
                own[function],
            )
        })
        .collect();
    largest_contributors.sort_by_key(|(_, copies_of_function, own_expressions)| {
        std::cmp::Reverse(copies_of_function * own_expressions)
    });
    largest_contributors.truncate(5);
    KernelInlinedSize {
        expressions,
        largest_contributors,
    }
}

/// 内核展开之后超过 [`LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL`] 就拒绝，不交给驱动去编（量入口 `main`）。
///
/// # Panics
/// 超过预算：消息里给展开份数最多的几个函数。
pub fn refuse_a_kernel_too_large_for_the_driver(label: &str, source: &str) {
    refuse_a_kernel_entry_point_too_large_for_the_driver(label, source, KERNEL_ENTRY_POINT);
}

/// 同 [`refuse_a_kernel_too_large_for_the_driver`]，量的是名叫 `entry_point` 的那个入口。
///
/// # Panics
/// 超过预算；源码里没有这个入口。
pub fn refuse_a_kernel_entry_point_too_large_for_the_driver(
    label: &str,
    source: &str,
    entry_point: &str,
) {
    let size = kernel_inlined_size_of_the_entry_point(source, entry_point);
    assert!(
        size.expressions <= LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL,
        "内核 {label} 全部调用展开之后 {} 个表达式，超过预算 {}：驱动的编译器把每个调用点展开成一份拷贝，\
         编译用时与内存随之猛涨（第二台 580 版驱动编一个超了三四倍的内核，48G 里编不完）。\
         把被多处调用的大函数改成只有一个调用点（例：走读内核的 w_walk_one 由 walk_and_hand_off 的阶段循环驱动）。\
         展开后占得最多的函数（名字、份数、自己的表达式数）：{:?}",
        size.expressions,
        LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL,
        size.largest_contributors
    );
}

#[cfg(test)]
mod tests;
