//! 读扫描内核源码钉表的用法：已释放记录表的站点 29 走不到、记账进位槽读回分配记录计数、I-3.10 去重表满之后照判、I-5.4 与 I-3.16 排分配记录的次序。

use super::*;

// ───────── 已释放记录那张表：站点 29 走不到（等价变异留档，`.claude/rules/mutation-sampling.md`「没红的三类，判据不同」） ─────────

/// 扫描内核里最新根已释放记录的条数（`crash_judge_checker_scan.wgsl` 的 `var<private>`）。
const RELEASED_RECORD_COUNTER: &str = "c_released_count";

/// 扫描内核里一个根的分配记录条数（同上）。
const ALLOCATION_RECORD_COUNTER: &str = "c_alloc_record_count";

/// 已释放记录那张表的上限：站点 29 拿已释放条数比的那个数。
const RELEASED_RECORD_TABLE_CAPACITY: &str = "C_MAX_RELEASED";

/// 分配记录那张表的上限：站点 20 拿分配记录条数比的那个数。
const ALLOCATION_RECORD_TABLE_CAPACITY: &str = "C_MAX_ALLOC_RECORDS";

/// 读草稿缓冲一格的函数（`crash_judge_common.wgsl`）：扫描内核几趟派活之间的交接段经它读回。
const SCRATCH_READING_FUNCTION: &str = "sw";

/// 写草稿缓冲一格的函数（同上）：交接段经它存下。
const SCRATCH_WRITING_FUNCTION: &str = "sws";

/// 扫描内核的每个函数，连同入口：名字与中间表示。
fn every_function_of(module: &Module) -> Vec<(&str, &Function)> {
    module
        .functions
        .iter()
        .map(|(_, function)| (function.name.as_deref().unwrap_or("（没有名字）"), function))
        .chain(
            module
                .entry_points
                .iter()
                .map(|entry_point| (entry_point.name.as_str(), &entry_point.function)),
        )
        .collect()
}

/// 一串语句与它们里面套着的每一个语句块（`if` 两支、`switch` 每支、循环体与 continuing、裸的语句块），这一串自己在最前。
#[allow(
    clippy::wildcard_enum_match_arm,
    reason = "naga 的语句是外部的枚举，只有这四种里还套着语句块，其余一律不含语句块"
)]
fn statement_lists_nested_in(statements: &[Statement]) -> Vec<&[Statement]> {
    let mut lists = vec![statements];
    for statement in statements {
        match statement {
            Statement::Block(inner) => lists.extend(statement_lists_nested_in(inner)),
            Statement::If { accept, reject, .. } => {
                lists.extend(statement_lists_nested_in(accept));
                lists.extend(statement_lists_nested_in(reject));
            }
            Statement::Switch { cases, .. } => {
                for case in cases {
                    lists.extend(statement_lists_nested_in(&case.body));
                }
            }
            Statement::Loop {
                body, continuing, ..
            } => {
                lists.extend(statement_lists_nested_in(body));
                lists.extend(statement_lists_nested_in(continuing));
            }
            _ => {}
        }
    }
    lists
}

fn is_the_global_variable(
    module: &Module,
    function: &Function,
    expression: Handle<Expression>,
    name: &str,
) -> bool {
    matches!(
        function.expressions[expression],
        Expression::GlobalVariable(global) if module.global_variables[global].name.as_deref() == Some(name)
    )
}

fn is_a_load_of_the_global_variable(
    module: &Module,
    function: &Function,
    expression: Handle<Expression>,
    name: &str,
) -> bool {
    matches!(
        function.expressions[expression],
        Expression::Load { pointer } if is_the_global_variable(module, function, pointer, name)
    )
}

fn the_local_variable_loaded_by(
    function: &Function,
    expression: Handle<Expression>,
) -> Option<Handle<LocalVariable>> {
    let Expression::Load { pointer } = function.expressions[expression] else {
        return None;
    };
    let Expression::LocalVariable(local_variable) = function.expressions[pointer] else {
        return None;
    };
    Some(local_variable)
}

fn is_the_u32_literal(function: &Function, expression: Handle<Expression>, value: u32) -> bool {
    matches!(
        function.expressions[expression],
        Expression::Literal(Literal::U32(literal)) if literal == value
    )
}

fn is_the_named_constant(
    module: &Module,
    function: &Function,
    expression: Handle<Expression>,
    name: &str,
) -> bool {
    matches!(
        function.expressions[expression],
        Expression::Constant(constant) if module.constants[constant].name.as_deref() == Some(name)
    )
}

/// 一个 `u32` 模块常量的值：初值是字面量，或指到别的常量（顺着指过去，链长有限：WGSL 不许常量互相引用成环）。
fn value_of_the_u32_constant(module: &Module, name: &str) -> u32 {
    let (_, constant) = module
        .constants
        .iter()
        .find(|(_, constant)| constant.name.as_deref() == Some(name))
        .unwrap_or_else(|| panic!("扫描内核里没有常量 {name}"));
    let mut initializer = constant.init;
    // 迭代上界：模块常量的个数（每一步换到另一个常量的初值）
    for _step in 0..=module.constants.len() {
        if let Expression::Literal(Literal::U32(value)) = module.global_expressions[initializer] {
            return value;
        }
        let Expression::Constant(other) = module.global_expressions[initializer] else {
            panic!(
                "常量 {name} 的初值既不是 u32 字面量也不是别的常量：{:?}",
                module.global_expressions[initializer]
            );
        };
        initializer = module.constants[other].init;
    }
    panic!(
        "常量 {name} 顺着指了 {} 步还没到字面量",
        module.constants.len() + 1
    );
}

/// 一次写一个计数（`u32` 的 `var<private>` 或循环变量）写成了什么。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CounterStore {
    SetToZero,
    IncrementedByOne,
    /// 写回之前读到的它自己的值（`let records_before = c_alloc_record_count;` 之后的 `c_alloc_record_count = records_before;`）。
    RestoredToAnEarlierReading,
    /// 从交接段读回一格（`c_alloc_record_count = sw(C_ACCOUNTING_CARRY_ALLOCATION_RECORD_COUNT);`）：读的那一格下标按常量算得出，
    /// 整个模块里下标按常量算得出、又是这一格的 `sws` 至少一处，每一处写的都是读这个计数自己
    /// （`sws(C_ACCOUNTING_CARRY_ALLOCATION_RECORD_COUNT, c_alloc_record_count);`），读回的是前一趟存下的它自己的值。
    /// 只认全局计数（[`store_to_the_global_counter`]）；下标按常量算不出的 `sws` 与不经 `sws` 写草稿缓冲的写法不在判的范围里。
    ReadBackFromACarrySlotHoldingOnlyTheCounter,
    WrittenOtherwise,
}

/// 一个计数被写成 `value`；`is_a_load_of_the_counter` 认一个表达式是不是读这个计数。
fn counter_store_of(
    function: &Function,
    value: Handle<Expression>,
    is_a_load_of_the_counter: &dyn Fn(Handle<Expression>) -> bool,
) -> CounterStore {
    if is_the_u32_literal(function, value, 0) {
        return CounterStore::SetToZero;
    }
    if is_a_load_of_the_counter(value) {
        return CounterStore::RestoredToAnEarlierReading;
    }
    if let Expression::Binary {
        op: BinaryOperator::Add,
        left,
        right,
    } = function.expressions[value]
    {
        if is_a_load_of_the_counter(left) && is_the_u32_literal(function, right, 1) {
            return CounterStore::IncrementedByOne;
        }
    }
    CounterStore::WrittenOtherwise
}

/// 一个 `u32` 表达式按常量算出的值：字面量、模块常量（顺着它的初值算）、或两者相加；读变量、调用这类算不出，交 `None`。
/// `expressions` 是这个表达式所在的那张表（函数自己的，或模块常量初值那张）；naga 的表里一个表达式的操作数都排在它前面，递归有底。
fn constant_u32_value_of(
    module: &Module,
    expressions: &Arena<Expression>,
    expression: Handle<Expression>,
) -> Option<u32> {
    if let Expression::Literal(Literal::U32(value)) = expressions[expression] {
        return Some(value);
    }
    if let Expression::Constant(constant) = expressions[expression] {
        return constant_u32_value_of(
            module,
            &module.global_expressions,
            module.constants[constant].init,
        );
    }
    let Expression::Binary {
        op: BinaryOperator::Add,
        left,
        right,
    } = expressions[expression]
    else {
        return None;
    };
    constant_u32_value_of(module, expressions, left)?.checked_add(constant_u32_value_of(
        module,
        expressions,
        right,
    )?)
}

/// `function` 里产出 `result` 那一次调用：被调的函数与交过去的参数。
fn the_call_producing(
    function: &Function,
    result: Handle<Expression>,
) -> Option<(Handle<Function>, &[Handle<Expression>])> {
    statement_lists_nested_in(&function.body)
        .into_iter()
        .flat_map(|nested| nested.iter())
        .find_map(|statement| {
            let Statement::Call {
                function: callee,
                ref arguments,
                result: Some(produced),
            } = *statement
            else {
                return None;
            };
            (produced == result).then_some((callee, arguments.as_slice()))
        })
}

fn is_the_function_named(module: &Module, function: Handle<Function>, name: &str) -> bool {
    module.functions[function].name.as_deref() == Some(name)
}

/// 整个模块里下标按常量算得出、又是 `slot` 的每一处 `sws`：写的是不是读全局计数 `counter`。
fn writes_of_the_scratch_slot(module: &Module, slot: u32, counter: &str) -> Vec<bool> {
    every_function_of(module)
        .into_iter()
        .flat_map(|(_, function)| {
            calls_in(&function.body)
                .into_iter()
                .filter_map(move |(callee, arguments)| {
                    let [index, value] = arguments else {
                        return None;
                    };
                    (is_the_function_named(module, callee, SCRATCH_WRITING_FUNCTION)
                        && constant_u32_value_of(module, &function.expressions, *index)
                            == Some(slot))
                    .then(|| is_a_load_of_the_global_variable(module, function, *value, counter))
                })
        })
        .collect()
}

/// `value` 是不是从交接段一格读回全局计数 `counter`（[`CounterStore::ReadBackFromACarrySlotHoldingOnlyTheCounter`] 的条件）。
fn is_a_read_back_from_a_carry_slot_holding_only_the_counter(
    module: &Module,
    function: &Function,
    value: Handle<Expression>,
    counter: &str,
) -> bool {
    let Expression::CallResult(callee) = function.expressions[value] else {
        return false;
    };
    let Some((_, [index])) = the_call_producing(function, value) else {
        return false;
    };
    let Some(slot) = constant_u32_value_of(module, &function.expressions, *index) else {
        return false;
    };
    let writes_of_the_slot = writes_of_the_scratch_slot(module, slot, counter);
    let every_write_stores_the_counter = writes_of_the_slot
        .iter()
        .all(|stores_the_counter| *stores_the_counter);
    is_the_function_named(module, callee, SCRATCH_READING_FUNCTION)
        && !writes_of_the_slot.is_empty()
        && every_write_stores_the_counter
}

/// 这一句写全局计数 `counter` 的话，写成了什么。
fn store_to_the_global_counter(
    module: &Module,
    function: &Function,
    statement: &Statement,
    counter: &str,
) -> Option<CounterStore> {
    let Statement::Store { pointer, value } = *statement else {
        return None;
    };
    is_the_global_variable(module, function, pointer, counter).then(|| {
        let store = counter_store_of(function, value, &|expression| {
            is_a_load_of_the_global_variable(module, function, expression, counter)
        });
        if store == CounterStore::WrittenOtherwise
            && is_a_read_back_from_a_carry_slot_holding_only_the_counter(
                module, function, value, counter,
            )
        {
            CounterStore::ReadBackFromACarrySlotHoldingOnlyTheCounter
        } else {
            store
        }
    })
}

/// 一串语句（连同里面套着的）里每一次写全局计数 `counter`。
fn stores_to_the_global_counter_in(
    module: &Module,
    function: &Function,
    statements: &[Statement],
    counter: &str,
) -> Vec<CounterStore> {
    statement_lists_nested_in(statements)
        .into_iter()
        .flat_map(|nested| nested.iter())
        .filter_map(|statement| store_to_the_global_counter(module, function, statement, counter))
        .collect()
}

/// 一串语句（连同里面套着的）里每一次写循环变量 `loop_variable`。
fn stores_to_the_local_variable_in(
    function: &Function,
    statements: &[Statement],
    loop_variable: Handle<LocalVariable>,
) -> Vec<CounterStore> {
    statement_lists_nested_in(statements)
        .into_iter()
        .flat_map(|nested| nested.iter())
        .filter_map(|statement| {
            let Statement::Store { pointer, value } = *statement else {
                return None;
            };
            matches!(function.expressions[pointer], Expression::LocalVariable(stored) if stored == loop_variable)
                .then(|| {
                    counter_store_of(function, value, &|expression| {
                        the_local_variable_loaded_by(function, expression) == Some(loop_variable)
                    })
                })
        })
        .collect()
}

/// 一串语句（连同里面套着的）里每一次调用：被调的函数与交过去的参数。
fn calls_in(statements: &[Statement]) -> Vec<(Handle<Function>, &[Handle<Expression>])> {
    statement_lists_nested_in(statements)
        .into_iter()
        .flat_map(|nested| nested.iter())
        .filter_map(|statement| {
            let Statement::Call {
                function,
                ref arguments,
                ..
            } = *statement
            else {
                return None;
            };
            Some((function, arguments.as_slice()))
        })
        .collect()
}

/// 直接或经调用写全局计数 `counter` 的函数（WGSL 不许递归，调用关系无环）。
fn functions_storing_the_global_counter(
    module: &Module,
    counter: &str,
) -> HashSet<Handle<Function>> {
    let mut storing: HashSet<Handle<Function>> = module
        .functions
        .iter()
        .filter(|(_, function)| {
            !stores_to_the_global_counter_in(module, function, &function.body, counter).is_empty()
        })
        .map(|(handle, _)| handle)
        .collect();
    // 迭代上界：函数的个数（每一轮至少多认出一个，认不出就停）
    for _round in 0..=module.functions.len() {
        let before = storing.len();
        for (handle, function) in module.functions.iter() {
            if calls_in(&function.body)
                .iter()
                .any(|(callee, _)| storing.contains(callee))
            {
                storing.insert(handle);
            }
        }
        if storing.len() == before {
            return storing;
        }
    }
    panic!("调用关系里有环：WGSL 不许递归，naga 解析得过的源码走不到这里");
}

/// 这一句是不是「计数到了上限就返回」那一判：`if (<计数> >= <上限常量>) { …; return; }`，没有 else。
fn is_a_guard_returning_once_the_counter_reaches_the_capacity(
    module: &Module,
    function: &Function,
    statement: &Statement,
    counter: &str,
    capacity: &str,
) -> bool {
    let Statement::If {
        condition,
        ref accept,
        ref reject,
    } = *statement
    else {
        return false;
    };
    let Expression::Binary {
        op: BinaryOperator::GreaterEqual,
        left,
        right,
    } = function.expressions[condition]
    else {
        return false;
    };
    is_a_load_of_the_global_variable(module, function, left, counter)
        && is_the_named_constant(module, function, right, capacity)
        && matches!(accept.last(), Some(Statement::Return { .. }))
        && reject.is_empty()
}

/// 挑最新根已释放记录的那个 for 循环。naga 把 `for (<初值>; <条件>; <每轮加>) { <循环体> }` 拆成：初值在外面那个语句块里，
/// `loop { if (<条件>) {} else { break; } { <循环体> } continuing { <每轮加> } }`。
struct ReleasedRecordPickingLoop<'module> {
    function_name: &'module str,
    function: &'module Function,
    enclosing_block: &'module [Statement],
    position_in_the_enclosing_block: usize,
    loop_variable: Handle<LocalVariable>,
    whole_body: &'module [Statement],
    for_body: &'module [Statement],
    continuing: &'module [Statement],
}

/// 条件是「循环变量 < 分配记录条数」、循环体顶层有一句已释放条数加一的 for 循环，扫描内核里的每一个。
fn every_released_record_picking_loop(module: &Module) -> Vec<ReleasedRecordPickingLoop<'_>> {
    let mut loops = Vec::new();
    for (function_name, function) in every_function_of(module) {
        for enclosing_block in statement_lists_nested_in(&function.body) {
            for (position_in_the_enclosing_block, statement) in enclosing_block.iter().enumerate() {
                let Statement::Loop {
                    body: ref whole_body,
                    ref continuing,
                    break_if: None,
                } = *statement
                else {
                    continue;
                };
                let mut statements_other_than_emit = whole_body
                    .iter()
                    .filter(|body_statement| !matches!(body_statement, Statement::Emit(_)));
                let (Some(first), Some(second)) = (
                    statements_other_than_emit.next(),
                    statements_other_than_emit.next(),
                ) else {
                    continue;
                };
                let Statement::If {
                    condition,
                    ref accept,
                    ref reject,
                } = *first
                else {
                    continue;
                };
                let Statement::Block(ref for_body) = *second else {
                    continue;
                };
                let is_the_condition_of_a_for_loop = accept.is_empty()
                    && reject.len() == 1
                    && matches!(reject.first(), Some(Statement::Break));
                let Expression::Binary {
                    op: BinaryOperator::Less,
                    left,
                    right,
                } = function.expressions[condition]
                else {
                    continue;
                };
                let Some(loop_variable) = the_local_variable_loaded_by(function, left) else {
                    continue;
                };
                let increments_the_released_record_counter =
                    for_body.iter().any(|for_body_statement| {
                        store_to_the_global_counter(
                            module,
                            function,
                            for_body_statement,
                            RELEASED_RECORD_COUNTER,
                        ) == Some(CounterStore::IncrementedByOne)
                    });
                if is_the_condition_of_a_for_loop
                    && is_a_load_of_the_global_variable(
                        module,
                        function,
                        right,
                        ALLOCATION_RECORD_COUNTER,
                    )
                    && increments_the_released_record_counter
                {
                    loops.push(ReleasedRecordPickingLoop {
                        function_name,
                        function,
                        enclosing_block,
                        position_in_the_enclosing_block,
                        loop_variable,
                        whole_body,
                        for_body,
                        continuing,
                    });
                }
            }
        }
    }
    loops
}

/// 扫描内核 `source` 里站点 29（已释放记录那张表满了、记判不了）走不到，推理逐步落成断言：
/// ① 已释放条数只归零与加一，加一只有一处，两个计数都不当指针交给别的函数；
/// ② 那一处在挑已释放记录的循环里，进循环之前同一个语句块里归零；循环变量每轮只在 continuing 里加一，条件是「循环变量 < 分配记录条数」，
///    循环体里不改分配记录条数（直接写与经调用写都没有）；站点 29 那一判在加一之前。所以走到第 k 轮（从 0 数）那一判时，
///    已释放条数 ≤ k ≤ 循环变量 < 分配记录条数；
/// ③ 分配记录条数只归零、写回之前读到的值、从交接段读回只存它自己的那一格（[`CounterStore::ReadBackFromACarrySlotHoldingOnlyTheCounter`]：
///    读回的是前一趟存下的它自己的值）、或在同一个函数顶层先判「≥ 分配记录表上限就返回」再加一（中间不经调用改它），所以它不超过那个上限；
/// ④ 分配记录表上限 ≤ 已释放记录表上限。
/// 合起来那一判的条件「已释放条数 ≥ 已释放记录表上限」在每个输入上都不成立：指这一判「抬过上限报判不了」的变异（上限抬过分配记录表、
/// 删掉这一判、`>=` 改 `>`）是等价变异，不进变异表；会改行为的那一条是把上限压到已释放条数之下（2048 那条流红）。
fn assert_site_twenty_nine_of_the_scan_kernel_is_unreachable(source: &str) {
    let module = wgpu::naga::front::wgsl::parse_str(source)
        .unwrap_or_else(|error| panic!("扫描内核解析失败：\n{}", error.emit_to_string(source)));
    // ①
    let mut released_record_counter_stores: Vec<(&str, CounterStore)> = Vec::new();
    for (function_name, function) in every_function_of(&module) {
        for store in stores_to_the_global_counter_in(
            &module,
            function,
            &function.body,
            RELEASED_RECORD_COUNTER,
        ) {
            released_record_counter_stores.push((function_name, store));
        }
    }
    assert!(
        released_record_counter_stores
            .iter()
            .all(|(_, store)| matches!(
                store,
                CounterStore::SetToZero | CounterStore::IncrementedByOne
            )),
        "已释放条数只该归零与加一：{released_record_counter_stores:?}"
    );
    assert_eq!(
        released_record_counter_stores
            .iter()
            .filter(|(_, store)| *store == CounterStore::IncrementedByOne)
            .count(),
        1,
        "已释放条数加一只有一处：{released_record_counter_stores:?}"
    );
    for counter in [RELEASED_RECORD_COUNTER, ALLOCATION_RECORD_COUNTER] {
        let functions_handing_the_counter_away: Vec<&str> = every_function_of(&module)
            .into_iter()
            .filter(|(_, function)| {
                calls_in(&function.body).iter().any(|(_, arguments)| {
                    arguments.iter().any(|argument| {
                        is_the_global_variable(&module, function, *argument, counter)
                    })
                })
            })
            .map(|(function_name, _)| function_name)
            .collect();
        assert!(
            functions_handing_the_counter_away.is_empty(),
            "{counter} 被当指针交给别的函数（那边写它这里看不见）：{functions_handing_the_counter_away:?}"
        );
    }
    // ②
    let picking_loops = every_released_record_picking_loop(&module);
    assert_eq!(
        picking_loops.len(),
        1,
        "挑已释放记录的循环（条件「循环变量 < {ALLOCATION_RECORD_COUNTER}」、循环体顶层 {RELEASED_RECORD_COUNTER} 加一）恰好一个，找到 {} 个：{:?}",
        picking_loops.len(),
        picking_loops
            .iter()
            .map(|picking_loop| picking_loop.function_name)
            .collect::<Vec<_>>()
    );
    let picking_loop = &picking_loops[0];
    let picking_function = picking_loop.function;
    assert!(
        picking_loop
            .enclosing_block
            .iter()
            .take(picking_loop.position_in_the_enclosing_block)
            .any(|statement| {
                store_to_the_global_counter(
                    &module,
                    picking_function,
                    statement,
                    RELEASED_RECORD_COUNTER,
                ) == Some(CounterStore::SetToZero)
            }),
        "{}：进挑已释放记录的循环之前，同一个语句块里先把 {RELEASED_RECORD_COUNTER} 归零",
        picking_loop.function_name
    );
    assert_eq!(
        stores_to_the_local_variable_in(
            picking_function,
            picking_loop.continuing,
            picking_loop.loop_variable
        ),
        vec![CounterStore::IncrementedByOne],
        "{}：循环变量在 continuing 里每轮加一、只写这一次",
        picking_loop.function_name
    );
    assert!(
        stores_to_the_local_variable_in(
            picking_function,
            picking_loop.whole_body,
            picking_loop.loop_variable
        )
        .is_empty(),
        "{}：循环体里不写循环变量",
        picking_loop.function_name
    );
    let functions_storing_the_allocation_record_counter =
        functions_storing_the_global_counter(&module, ALLOCATION_RECORD_COUNTER);
    for block in [picking_loop.whole_body, picking_loop.continuing] {
        assert!(
            stores_to_the_global_counter_in(
                &module,
                picking_function,
                block,
                ALLOCATION_RECORD_COUNTER
            )
            .is_empty(),
            "{}：挑已释放记录的循环里不写 {ALLOCATION_RECORD_COUNTER}",
            picking_loop.function_name
        );
        assert!(
            calls_in(block).iter().all(|(callee, arguments)| {
                !functions_storing_the_allocation_record_counter.contains(callee)
                    && arguments.iter().all(|argument| {
                        !matches!(
                            picking_function.expressions[*argument],
                            Expression::LocalVariable(handed) if handed == picking_loop.loop_variable
                        )
                    })
            }),
            "{}：挑已释放记录的循环里不调用写 {ALLOCATION_RECORD_COUNTER} 的函数，也不把循环变量交出去",
            picking_loop.function_name
        );
    }
    let position_of_site_twenty_nine = picking_loop.for_body.iter().position(|statement| {
        is_a_guard_returning_once_the_counter_reaches_the_capacity(
            &module,
            picking_function,
            statement,
            RELEASED_RECORD_COUNTER,
            RELEASED_RECORD_TABLE_CAPACITY,
        )
    });
    let position_of_the_released_record_increment =
        picking_loop.for_body.iter().position(|statement| {
            store_to_the_global_counter(
                &module,
                picking_function,
                statement,
                RELEASED_RECORD_COUNTER,
            ) == Some(CounterStore::IncrementedByOne)
        });
    assert!(
        matches!(
            (position_of_site_twenty_nine, position_of_the_released_record_increment),
            (Some(check), Some(increment)) if check < increment
        ),
        "{}：循环体顶层先有站点 29 那一判（{RELEASED_RECORD_COUNTER} >= {RELEASED_RECORD_TABLE_CAPACITY} 就返回）、再加一；\
         那一判在第 {position_of_site_twenty_nine:?} 句，加一在第 {position_of_the_released_record_increment:?} 句",
        picking_loop.function_name
    );
    // ③
    for (function_name, function) in every_function_of(&module) {
        let stores = stores_to_the_global_counter_in(
            &module,
            function,
            &function.body,
            ALLOCATION_RECORD_COUNTER,
        );
        assert!(
            stores.iter().all(|store| *store != CounterStore::WrittenOtherwise),
            "{function_name}：{ALLOCATION_RECORD_COUNTER} 只该归零、写回之前读到的值、从交接段读回只存它自己的那一格、或加一：{stores:?}"
        );
        let increments_anywhere = stores
            .iter()
            .filter(|store| **store == CounterStore::IncrementedByOne)
            .count();
        let increments_on_the_top_level: Vec<usize> = function
            .body
            .iter()
            .enumerate()
            .filter(|(_, statement)| {
                store_to_the_global_counter(&module, function, statement, ALLOCATION_RECORD_COUNTER)
                    == Some(CounterStore::IncrementedByOne)
            })
            .map(|(position, _)| position)
            .collect();
        assert_eq!(
            increments_on_the_top_level.len(),
            increments_anywhere,
            "{function_name}：{ALLOCATION_RECORD_COUNTER} 加一只写在函数体顶层"
        );
        for position_of_the_increment in increments_on_the_top_level {
            let position_of_the_guard = function.body[..position_of_the_increment]
                .iter()
                .rposition(|statement| {
                    is_a_guard_returning_once_the_counter_reaches_the_capacity(
                        &module,
                        function,
                        statement,
                        ALLOCATION_RECORD_COUNTER,
                        ALLOCATION_RECORD_TABLE_CAPACITY,
                    )
                })
                .unwrap_or_else(|| {
                    panic!(
                        "{function_name}：{ALLOCATION_RECORD_COUNTER} 加一之前，函数体顶层先判「>= {ALLOCATION_RECORD_TABLE_CAPACITY} 就返回」"
                    )
                });
            let between_the_guard_and_the_increment =
                &function.body[position_of_the_guard + 1..position_of_the_increment];
            assert!(
                stores_to_the_global_counter_in(
                    &module,
                    function,
                    between_the_guard_and_the_increment,
                    ALLOCATION_RECORD_COUNTER
                )
                .is_empty()
                    && calls_in(between_the_guard_and_the_increment)
                        .iter()
                        .all(|(callee, _)| {
                            !functions_storing_the_allocation_record_counter.contains(callee)
                        }),
                "{function_name}：判过上限到加一之间不改 {ALLOCATION_RECORD_COUNTER}（直接写与经调用写都没有）"
            );
        }
    }
    // ④
    let allocation_record_table_capacity =
        value_of_the_u32_constant(&module, ALLOCATION_RECORD_TABLE_CAPACITY);
    let released_record_table_capacity =
        value_of_the_u32_constant(&module, RELEASED_RECORD_TABLE_CAPACITY);
    assert!(
        allocation_record_table_capacity <= released_record_table_capacity,
        "{ALLOCATION_RECORD_TABLE_CAPACITY} = {allocation_record_table_capacity} 不该大于 {RELEASED_RECORD_TABLE_CAPACITY} = {released_record_table_capacity}：\
         已释放记录是从分配记录表里挑的，已释放表窄了站点 29 就走得到"
    );
}

/// 扫描内核挑最新根已释放记录的那张表（`C_MAX_RELEASED`）装得下每一条已释放记录，站点 29 走不到：推理在
/// [`assert_site_twenty_nine_of_the_scan_kernel_is_unreachable`]，判的是库编进去的那一份扫描内核源码，只用 naga 解析、不用卡。
#[test]
fn site_twenty_nine_of_the_scan_kernel_is_unreachable_because_released_records_are_picked_from_the_allocation_record_table(
) {
    assert_site_twenty_nine_of_the_scan_kernel_is_unreachable(&checker_scan_kernel_source());
}

/// 扫描内核记账那几趟开头从交接段读回分配记录条数的函数（`crash_judge_checker_scan.wgsl`）。
const ACCOUNTING_CARRY_LOADING_FUNCTION: &str = "accounting_carry_load";

/// 记账头那一趟把分配记录条数存进交接段的那一句（`crash_judge_checker_scan.wgsl`，扫描内核里恰好一处）。
const STORING_THE_ALLOCATION_RECORD_COUNTER_INTO_THE_CARRY: &str =
    "sws(C_ACCOUNTING_CARRY_ALLOCATION_RECORD_COUNT, c_alloc_record_count);";

/// 扫描内核 `source` 里函数 `function_name` 每一次写分配记录条数写成了什么。
fn allocation_record_counter_stores_in_the_function(
    source: &str,
    function_name: &str,
) -> Vec<CounterStore> {
    let module = wgpu::naga::front::wgsl::parse_str(source)
        .unwrap_or_else(|error| panic!("扫描内核解析失败：\n{}", error.emit_to_string(source)));
    let (_, function) = every_function_of(&module)
        .into_iter()
        .find(|(name, _)| *name == function_name)
        .unwrap_or_else(|| panic!("扫描内核里没有函数 {function_name}"));
    stores_to_the_global_counter_in(&module, function, &function.body, ALLOCATION_RECORD_COUNTER)
}

/// 库编进去的那一份扫描内核里，`accounting_carry_load` 从交接段读回分配记录条数记「从交接段读回只存它自己的那一格」：
/// 交接段那一格只有记账头那一趟的一处 `sws`，写的是分配记录条数自己。
#[test]
fn the_accounting_carry_load_reads_the_allocation_record_counter_back_from_a_carry_slot_holding_only_the_counter(
) {
    assert_eq!(
        allocation_record_counter_stores_in_the_function(
            &checker_scan_kernel_source(),
            ACCOUNTING_CARRY_LOADING_FUNCTION
        ),
        vec![CounterStore::ReadBackFromACarrySlotHoldingOnlyTheCounter]
    );
}

/// 交接段那一格的那一处 `sws` 改成存别的计数（`c_occupied_count`）之后，`accounting_carry_load` 从那一格读回分配记录条数记 `WrittenOtherwise`：
/// 站点 29 那条推理的 ③ 不放行（读回的不一定是分配记录条数自己的值）。改的是测试里的源码串，库里的扫描内核不动。
#[test]
fn reading_the_allocation_record_counter_back_from_a_carry_slot_written_with_another_counter_is_written_otherwise(
) {
    let source = checker_scan_kernel_source();
    assert_eq!(
        source
            .matches(STORING_THE_ALLOCATION_RECORD_COUNTER_INTO_THE_CARRY)
            .count(),
        1,
        "扫描内核里「{STORING_THE_ALLOCATION_RECORD_COUNTER_INTO_THE_CARRY}」恰好一处"
    );
    let altered_source = source.replace(
        STORING_THE_ALLOCATION_RECORD_COUNTER_INTO_THE_CARRY,
        "sws(C_ACCOUNTING_CARRY_ALLOCATION_RECORD_COUNT, c_occupied_count);",
    );
    assert_eq!(
        allocation_record_counter_stores_in_the_function(
            &altered_source,
            ACCOUNTING_CARRY_LOADING_FUNCTION
        ),
        vec![CounterStore::WrittenOtherwise]
    );
}

// ───────── I-3.10 的去重表：表满之后照判，站点 31 没有了 ─────────

/// 扫描内核里判 I-3.10 一棵树的那个函数（`crash_judge_checker_scan.wgsl`）。
const ALLOCATION_GENERATION_JUDGING_FUNCTION: &str = "judge_allocation_generations_of_the_records";

/// 扫描内核里 I-3.10 去重表已记的条数（`var<private>`）。
const EXAMINED_RECORD_COUNTER: &str = "c_examined_count";

/// I-3.10 去重表的上限（`crash_judge_checker_common.wgsl`）。
const EXAMINED_RECORD_TABLE_CAPACITY: &str = "C_MAX_EXAMINED";

/// 判不了的原因字（`crash_judge_common.wgsl` 的 `var<private>`）：写了它，这个状态就判不了。
const UNDECIDABLE_REASON_WORD: &str = "undecidable";

/// 置评估位与违例位的那个函数（`crash_judge_checker_common.wgsl`）。
const JUDGING_FUNCTION: &str = "judge";

/// I-3.10 在评估位、违例位里的下标常量（`crash_judge_checker_common.wgsl`）。
const ALLOCATION_GENERATION_INVARIANT_CONSTANT: &str = "INVARIANT_I_3_10";

/// 这一句是不是「去重表还没满才往里记」那一判：`if (<计数> < <上限常量>) { … }`，没有 else。
fn is_a_guard_recording_only_while_the_examined_table_has_room(
    module: &Module,
    function: &Function,
    statement: &Statement,
) -> bool {
    let Statement::If {
        condition,
        ref reject,
        ..
    } = *statement
    else {
        return false;
    };
    let Expression::Binary {
        op: BinaryOperator::Less,
        left,
        right,
    } = function.expressions[condition]
    else {
        return false;
    };
    is_a_load_of_the_global_variable(module, function, left, EXAMINED_RECORD_COUNTER)
        && is_the_named_constant(module, function, right, EXAMINED_RECORD_TABLE_CAPACITY)
        && reject.is_empty()
}

/// 扫描内核判 I-3.10 一棵树的那个函数满足三样，合起来就是「去重表满了之后新来的记录不进表、照判，这个函数不让状态判不了」：
/// ① 它自己（连同里面套着的语句块）一处都不写判不了的原因字；② 去重表的计数只在「还没满才往里记」那一判里面加一，那一判恰好一处；
/// ③ 判 I-3.10 的那一次 `judge` 恰好一处，不在那一判里面（表满了照样走到）。
/// 只看这个函数自己：它调的读盘那几个函数各自报不报判不了，是读不读得出单元头的事，与去重表满不满无关。
fn assert_the_allocation_generations_are_judged_on_once_the_examined_table_is_full(source: &str) {
    let module = wgpu::naga::front::wgsl::parse_str(source)
        .unwrap_or_else(|error| panic!("扫描内核解析失败：\n{}", error.emit_to_string(source)));
    let (_, function) = module
        .functions
        .iter()
        .find(|(_, function)| {
            function.name.as_deref() == Some(ALLOCATION_GENERATION_JUDGING_FUNCTION)
        })
        .unwrap_or_else(|| panic!("扫描内核里没有 {ALLOCATION_GENERATION_JUDGING_FUNCTION}"));
    // ①
    let undecidable_stores =
        stores_to_the_global_counter_in(&module, function, &function.body, UNDECIDABLE_REASON_WORD);
    assert!(
        undecidable_stores.is_empty(),
        "{ALLOCATION_GENERATION_JUDGING_FUNCTION} 不该写判不了的原因字（表满了照判、不记站点 31）：写了 {} 处 {undecidable_stores:?}",
        undecidable_stores.len()
    );
    // ②
    let guards: Vec<&Statement> = statement_lists_nested_in(&function.body)
        .into_iter()
        .flat_map(|nested| nested.iter())
        .filter(|statement| {
            is_a_guard_recording_only_while_the_examined_table_has_room(
                &module, function, statement,
            )
        })
        .collect();
    assert_eq!(
        guards.len(),
        1,
        "{ALLOCATION_GENERATION_JUDGING_FUNCTION} 里「{EXAMINED_RECORD_COUNTER} < {EXAMINED_RECORD_TABLE_CAPACITY} 才往表里记」那一判恰好一处"
    );
    let Statement::If {
        accept: ref recorded_while_there_is_room,
        ..
    } = *guards[0]
    else {
        unreachable!("上面只挑了 if");
    };
    let increments_in_the_whole_function: Vec<CounterStore> =
        stores_to_the_global_counter_in(&module, function, &function.body, EXAMINED_RECORD_COUNTER)
            .into_iter()
            .filter(|store| *store == CounterStore::IncrementedByOne)
            .collect();
    let increments_under_the_guard: Vec<CounterStore> = stores_to_the_global_counter_in(
        &module,
        function,
        recorded_while_there_is_room,
        EXAMINED_RECORD_COUNTER,
    )
    .into_iter()
    .filter(|store| *store == CounterStore::IncrementedByOne)
    .collect();
    assert_eq!(
        (
            increments_in_the_whole_function.len(),
            increments_under_the_guard.len()
        ),
        (1, 1),
        "{EXAMINED_RECORD_COUNTER} 只在那一判里面加一，恰好一处"
    );
    // ③
    let (judging_function, _) = module
        .functions
        .iter()
        .find(|(_, candidate)| candidate.name.as_deref() == Some(JUDGING_FUNCTION))
        .unwrap_or_else(|| panic!("扫描内核里没有 {JUDGING_FUNCTION}"));
    let judges_the_allocation_generation_invariant =
        |(callee, arguments): &(Handle<Function>, &[Handle<Expression>])| {
            *callee == judging_function
                && arguments.first().is_some_and(|invariant| {
                    is_the_named_constant(
                        &module,
                        function,
                        *invariant,
                        ALLOCATION_GENERATION_INVARIANT_CONSTANT,
                    )
                })
        };
    assert_eq!(
        calls_in(&function.body)
            .iter()
            .filter(|call| judges_the_allocation_generation_invariant(call))
            .count(),
        1,
        "{ALLOCATION_GENERATION_JUDGING_FUNCTION} 里判 {ALLOCATION_GENERATION_INVARIANT_CONSTANT} 的 {JUDGING_FUNCTION} 恰好一处"
    );
    assert_eq!(
        calls_in(recorded_while_there_is_room)
            .iter()
            .filter(|call| judges_the_allocation_generation_invariant(call))
            .count(),
        0,
        "判 {ALLOCATION_GENERATION_INVARIANT_CONSTANT} 的那一次不在「表还没满才记」那一判里面：表满了照样判"
    );
}

/// I-3.10 的去重表（`C_MAX_EXAMINED`）满了之后不再记判不了：新来的记录不进表、照判（I-3.10 是置位，同一条记录判两遍结论相同；
/// CPU 判器去重只为不重复判）。合法的流造不出越过那张表的状态（文件头），所以落成扫描内核 naga 中间表示上的断言
/// （[`assert_the_allocation_generations_are_judged_on_once_the_examined_table_is_full`]），判的是库编进去的那一份扫描内核源码，只用 naga 解析、不用卡。
/// 判别力（`crates/mutations.tsv`）：表满时退回记站点 31、返回，① 与 ② 红。
#[test]
fn judging_the_allocation_generations_never_leaves_the_state_undecidable_and_judges_on_once_the_examined_table_is_full(
) {
    assert_the_allocation_generations_are_judged_on_once_the_examined_table_is_full(
        &checker_scan_kernel_source(),
    );
}

// ───────── I-5.4 与 I-3.16 排分配记录借 ⑮ 的次序表：钉两处调用点 ─────────

/// 把一张表的行按前四个字排出行号次序的那个函数（`crash_judge_checker_common.wgsl`）：参数依次是表的起点、一行几个字、几行、次序写到哪。
const ROW_ORDERING_FUNCTION: &str = "order_rows_by_their_first_four_words";

/// 扫描内核一条根的分配记录那张表的起点（`crash_judge_checker_common.wgsl` ⑫）。
const ALLOCATION_RECORD_TABLE_START: &str = "C_ALLOC_RECORDS";

/// 排分配记录借的次序表（`crash_judge_checker_common.wgsl` ⑮，`C_MAX_ALLOC_RECORDS` 个字）。
const ALLOCATION_RECORD_ROW_ORDER: &str = "C_ALLOCATION_RECORD_ROW_ORDER";

/// 扫描内核里排分配记录的两处所在的函数：I-5.4（相邻两条比区间）与 I-3.16（按记录口径数全空段）。
const FUNCTIONS_ORDERING_THE_ALLOCATION_RECORDS: [&str; 2] = [
    "judge_allocation_record_ranges",
    "judge_fragmentation_and_empty_segment_statistics",
];

/// 一个参数在哪一种情形下取哪个值：直接给的，或 `select` 的条件成立、不成立两边。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ArgumentSide {
    Unconditional,
    WhenTheConditionHolds,
    WhenTheConditionFails,
}

/// 一个参数表达式在各情形下是哪个模块常量（不是模块常量的那一边记 `None`）。
#[allow(
    clippy::wildcard_enum_match_arm,
    reason = "naga 的表达式是外部的枚举，这里只认常量与 select 两种，其余一律当不是模块常量"
)]
fn named_constants_of_the_argument<'module>(
    module: &'module Module,
    function: &Function,
    argument: Handle<Expression>,
) -> Vec<(ArgumentSide, Option<&'module str>)> {
    let constant_name = |expression: Handle<Expression>| match function.expressions[expression] {
        Expression::Constant(constant) => module.constants[constant].name.as_deref(),
        _ => None,
    };
    match function.expressions[argument] {
        Expression::Select { accept, reject, .. } => vec![
            (ArgumentSide::WhenTheConditionHolds, constant_name(accept)),
            (ArgumentSide::WhenTheConditionFails, constant_name(reject)),
        ],
        _ => vec![(ArgumentSide::Unconditional, constant_name(argument))],
    }
}

/// 排分配记录那张表的一处调用：在哪个函数里、次序那个参数在同一种情形下是哪个模块常量。
#[derive(Debug, PartialEq, Eq)]
struct AllocationRecordOrderingSite<'module> {
    function_name: &'module str,
    row_order: Option<&'module str>,
}

/// 扫描内核里每一处「表的起点在某一种情形下是 `C_ALLOC_RECORDS`」的排行号次序调用，连同那一种情形下次序参数是哪个常量
/// （次序参数直接给的就是它；也是 `select` 的取同一边）。
fn allocation_record_ordering_sites_of(module: &Module) -> Vec<AllocationRecordOrderingSite<'_>> {
    let (row_ordering_function, _) = module
        .functions
        .iter()
        .find(|(_, function)| function.name.as_deref() == Some(ROW_ORDERING_FUNCTION))
        .unwrap_or_else(|| panic!("扫描内核里没有 {ROW_ORDERING_FUNCTION}"));
    let mut sites = Vec::new();
    for (function_name, function) in every_function_of(module) {
        for (callee, arguments) in calls_in(&function.body) {
            if callee != row_ordering_function {
                continue;
            }
            let [table_start, _row_words, _row_count, row_order] = arguments else {
                panic!(
                    "{function_name} 调 {ROW_ORDERING_FUNCTION} 给了 {} 个参数，它收四个",
                    arguments.len()
                );
            };
            let row_orders = named_constants_of_the_argument(module, function, *row_order);
            for (side, table) in named_constants_of_the_argument(module, function, *table_start) {
                if table != Some(ALLOCATION_RECORD_TABLE_START) {
                    continue;
                }
                let row_order_on_the_same_side = row_orders
                    .iter()
                    .find(|(row_order_side, _)| {
                        *row_order_side == ArgumentSide::Unconditional || *row_order_side == side
                    })
                    .and_then(|(_, name)| *name);
                sites.push(AllocationRecordOrderingSite {
                    function_name,
                    row_order: row_order_on_the_same_side,
                });
            }
        }
    }
    sites
}

/// 扫描内核排分配记录的调用点恰好是 I-5.4、I-3.16 那两处，各一处，次序参数都是 `C_ALLOCATION_RECORD_ROW_ORDER`。
fn assert_the_allocation_records_are_ordered_in_their_own_row_order(source: &str) {
    let module = wgpu::naga::front::wgsl::parse_str(source)
        .unwrap_or_else(|error| panic!("扫描内核解析失败：\n{}", error.emit_to_string(source)));
    let mut sites = allocation_record_ordering_sites_of(&module);
    sites.sort_by_key(|site| site.function_name);
    let mut expected: Vec<AllocationRecordOrderingSite<'_>> =
        FUNCTIONS_ORDERING_THE_ALLOCATION_RECORDS
            .iter()
            .map(|function_name| AllocationRecordOrderingSite {
                function_name,
                row_order: Some(ALLOCATION_RECORD_ROW_ORDER),
            })
            .collect();
    expected.sort_by_key(|site| site.function_name);
    assert_eq!(
        sites, expected,
        "扫描内核里拿 {ALLOCATION_RECORD_TABLE_START} 调 {ROW_ORDERING_FUNCTION} 的恰好是 I-5.4、I-3.16 那两处（按函数名排），次序都写进 {ALLOCATION_RECORD_ROW_ORDER}（一棵分配记录树至多 C_MAX_ALLOC_RECORDS 条，C_ROW_ORDER 只有 C_MAX_REFERENCES 个字）"
    );
}

/// I-5.4 与 I-3.16 排分配记录借 ⑮ 的次序表（`C_ALLOCATION_RECORD_ROW_ORDER`，`C_MAX_ALLOC_RECORDS` 个字）。借回 `C_ROW_ORDER`（4096 个字）时
/// 一版多于 4096 条的分配记录排出的次序越过那一段，越过的那一截落进扫描内核不读的空段，卡上的判定看不出（整理夹着回退那 27 张照样判对），
/// 所以落成扫描内核 naga 中间表示上的断言（[`assert_the_allocation_records_are_ordered_in_their_own_row_order`]），判的是库编进去的那一份扫描内核源码，
/// 只用 naga 解析、不用卡。判别力（`crates/mutations.tsv`）：两处调用的次序参数各自改回 `C_ROW_ORDER`，这条红。
#[test]
fn the_scan_kernel_orders_the_allocation_records_in_the_allocation_record_row_order_at_both_call_sites(
) {
    assert_the_allocation_records_are_ordered_in_their_own_row_order(&checker_scan_kernel_source());
}
