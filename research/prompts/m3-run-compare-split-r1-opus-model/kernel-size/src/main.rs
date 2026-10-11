// 攻方腿模型（m3-run-compare-split-r1）：量 WGSL 内核展开之后的表达式数，数法照
// crates/singlefs-checker-tier/src/crash_judge_dispatch.rs 的 kernel_inlined_size（函数自己的表达式数 + 每个调用点上被调函数展开后的表达式数）。
// 用法：kernel-size <入口函数名> <WGSL 文件>…（几份文件按给的次序拼起来解析）；另带 --each 打每个函数「展开后大小、在入口下被展开几份」。
// 不碰显卡，只用 naga 前端。
use std::collections::{HashMap, HashSet};

use naga::{Block, Function, Handle, Statement};

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
            Statement::Loop { body, continuing, .. } => {
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
        + calls[&function].iter().map(|callee| inlined(*callee, own, calls, memo)).sum::<u64>();
    memo.insert(function, size);
    size
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let each = arguments.iter().any(|argument| argument == "--each");
    let positional: Vec<&String> = arguments.iter().filter(|argument| *argument != "--each").collect();
    let entry_name = positional[0];
    let mut source = String::new();
    for file in &positional[1..] {
        source.push_str(&std::fs::read_to_string(file).expect("读得到 WGSL"));
        source.push('\n');
    }
    let module = match naga::front::wgsl::parse_str(&source) {
        Ok(module) => module,
        Err(error) => {
            println!("PARSE_FAILED\n{}", error.emit_to_string(&source));
            std::process::exit(3);
        }
    };
    let mut own = HashMap::new();
    let mut calls = HashMap::new();
    for (handle, function) in module.functions.iter() {
        own.insert(handle, function.expressions.len() as u64);
        let mut sites = Vec::new();
        calls_in(&function.body, &mut sites);
        calls.insert(handle, sites);
    }
    let mut memo = HashMap::new();
    let entry = module.entry_points.iter().find(|entry| entry.name == *entry_name).expect("入口在");
    let mut entry_calls = Vec::new();
    calls_in(&entry.function.body, &mut entry_calls);
    let total = entry.function.expressions.len() as u64
        + entry_calls.iter().map(|callee| inlined(*callee, &own, &calls, &mut memo)).sum::<u64>();
    println!("KERNEL_INLINED entry={entry_name} expressions={total} margin_to_400000={}", 400_000i64 - total as i64);
    if each {
        let mut copies: HashMap<Handle<Function>, u64> = HashMap::new();
        let mut order = Vec::new();
        let mut seen = HashSet::new();
        fn postorder(
            function: Handle<Function>,
            calls: &HashMap<Handle<Function>, Vec<Handle<Function>>>,
            seen: &mut HashSet<Handle<Function>>,
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
            *copies.entry(*callee).or_insert(0) += 1;
        }
        for function in order.iter().rev() {
            let caller_copies = copies.get(function).copied().unwrap_or(0);
            for callee in &calls[function] {
                *copies.entry(*callee).or_insert(0) += caller_copies;
            }
        }
        for (handle, function) in module.functions.iter() {
            let size = inlined(handle, &own, &calls, &mut memo);
            println!(
                "FUNCTION name={} own={} inlined={} copies_under_entry={}",
                function.name.clone().unwrap_or_default(),
                own[&handle],
                size,
                copies.get(&handle).copied().unwrap_or(0)
            );
        }
    }
}
