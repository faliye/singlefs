# m3-close-unit-graph-wrapper-impls-r2：第一轮的三条零轮改法撑不撑得住（2026-11-13）

批次：`records/2026-10-18-里程碑三收尾与近期欠账.md` 第 2842–2847 行那一批。上一轮判决 `research/prompts/m3-close-unit-graph-wrapper-impls-r1-main-verification.md`，这一轮只攻它第三节的三条零轮改法，不重复第一轮攻过的角度。

## 一、题面

第一轮打中改法甲的原型（八格夹具反例，任务 15620），用户 2026-11-13 定「收严后继续改键算法」。主 agent 把两条腿各自提的收严合成了一套写法，连同改法乙的新写法，在草稿副本里写好了（主工作区没动）。这一轮问：这套写法还会不会让某个入口的键过窄；改法乙的登记与三条测试证不证得了它们要证的事。键算法是 D13（验证路线） 已定项 17。

## 二、实现今天的样子（主 agent 的观测，2026-11-13）

方案按 `crates/` 今天的实现来谈。主工作区的 `crates/singlefs-unit-graph/src/closure.rs`、`rust_items.rs`、`workspace.rs` 与 `crates/singlefs-verification/src/judging_units_entries.rs` 还是第一轮开工快照那一份（今天现核 sha256 相同）。被攻的改动是草稿副本相对主工作区的 diff：

- diff 全文：`~/.local/share/singlefs/work/m3-close-gpu-judge-closure/evidence/round-two-changes.diff`（1350 行，sha256 238247e7d699bea40095b2b701ca7b0ac6033ac62a459891aff2107e131d9731）。
- 改后的整棵树（只读）：`~/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/`，改到的文件是 `crates/singlefs-unit-graph/src/` 的 `closure.rs`、`workspace.rs`、`units.rs`、`rust_items.rs`、`rule_switches.rs`、`entry_groups.rs`，`crates/singlefs-verification/` 的 `src/judging_units_entries.rs`、`build.rs`、`src/crash_identity.rs`，`scripts/gate/lib_function_units.py`，与单元图一份测试、一份夹具。

三条改法各落在哪（都在 `verify-copy` 里）：

1. 甲收严一，等包装类型被点名要五条都满足：`closure.rs` 的 `implementation_waits_for_its_wrapper_type` 与 `is_released_as_a_blanket_implementation`；三张索引在 `workspace.rs`（`type_units_by_name`、`names_renamed_by_a_use`、`names_a_use_path_passes_through`）；impl 头的两个新字段在 `units.rs`、`rust_items.rs`（`generic_parameter_names`、`self_type_first_name`）。Python 一份：`lib_function_units.py` 的 `waits_for_its_wrapper_type`、`released_as_blanket`。
2. 甲收严二，入口外面传进来的值多开三处：`closure.rs` 的 `open_the_traits_reaching_the_entry_from_outside`、`names_written_where_a_trait_goes`，与 `open_trait` 里对被开进来的 impl 头上约束的递归。Python 一份同名函数与 `dyn_trait`。
3. 乙，按单元登记的排除（只有 Rust 一侧，判器组的键只由构建脚本经单元图算）：`closure.rs` 的 `compute_closure_of_entries_not_following_the_listed_units`，`entry_groups.rs` 的 `entry_group_key_not_following_the_listed_units`，登记 `judging_units_entries.rs` 的 `UNITS_OF_THE_MEMORY_LEDGER_THE_GPU_JUDGE_GROUP_DOES_NOT_FOLLOW`（52 个限定名），三条测试在 `crash_identity.rs`（`the_gpu_judge_group_reaches_no_unit_of_the_memory_ledger_beyond_the_listed_ones`、`not_following_the_listed_ledger_units_drops_no_unit_outside_the_memory_ledger`、`an_edit_of_a_listed_unit_of_the_memory_ledger_keeps_the_gpu_judge_key`）。

量的：

- 任务 15787：收严后的单元图在第一轮攻方的十个夹具上每格都收、键都变（十行同形「impl_in_closure=1 key_changes_with_the_edit=yes」）；真仓 GPU 判器入口组「闭包 2932 个单元（Rust 1063、WGSL 1869、数据 0），拒算 0 处」，与原型同样从 1361 降到 1063，`fault_injection.rs`、`read_tally.rs`、`mounted_session.rs`、`singlefs-core/src/mount.rs` 各 0 个。
- 任务 15768：副本里单元图库单测 70 条、blanket 那一份 17 条、逐开关那一份 46 条都绿。任务 15766：Python 自证 617 格里红 2 格，都是最小副本里缺 `records/` 与几份钩子造成的真仓格。
- 任务 15784：判器键那一模块 30 条里 28 条绿，含「编进去的键与照树重算的相同」「改一个登记了的显存记账单元 GPU 判器的键不变」；乙的两条测试头一版断言写错（真从别的模块跨进显存记账的只有 5 个单元，其余 47 个经它们才够到；显存记账之外少掉的是 `lib.rs` 里 `mod gpu_memory_ledger;` 那一行），已照读数改，重跑任务 15786「test result: ok. 30 passed; 0 failed」，三条测试都绿。
- 任务 15767：副本里门禁 code-tooling 的两份实现对拍格绿，「dump 比了 55857 个单元（Rust 53627、WGSL 2074、数据 156），逐个相同；登记入口 10 个（stage-inputs.tsv 18 个），闭包与键逐个相同」。
- 没量的：正推腿第一轮形态十一的夹具（入口接收者字段里的 `dyn`、入口所在 impl 头上的约束）。

## 三、分工

| 腿 | 立场 | 攻击面 |
|---|---|---|
| 辩方（Sonnet） | 复核第一轮判决：它判「打中」的八格与「按打中办」的形态十一，收严之后是不是真的进不来；判决第三节三条改法各自够不够得着它声称修的那几格；替「撤回改法甲」这一方辩护一次——收严之后还剩的收缩值不值这套规则的复杂度与残余风险 | 逐格对着 `verify-copy` 的代码核，不靠任务 15787 的结论句；Rust 与 Python 两份的写法有没有不等价的地方（各列出文件:行） |
| 攻方（Opus） | 找新的反例，不重复第一轮九个夹具的形 | ① 五条里每一条的边角：头一个名字的取法（`&'a mut Wrapper<T>`、`(Wrapper<T>, u32)`、`[Wrapper<T>; N]`、`dyn Tr + 'a`、`<T as Tr>::Out`、宏里的 `$name<T>`）、「同一份文件同一段行内模块」在 `#[path]`、`include!`、同一文件被两个目标各编一次、cfg 两臂各定义一个同名类型时还成不成立；本模块里既有 `struct Wrapper` 又 `use other::Wrapper`（编不过）之外，有没有编得过而名字不解到本模块那个类型的写法（宏卫生、`Self`、关联类型、泛型参数遮蔽、嵌套函数里的同名条目）；② 包装值不写包装类型名就造出来的别的路：`Default`/`From`/`Into`/`FromIterator`、返回 `impl Trait` 的函数在闭包外、闭包捕获、`transmute`/指针转换、`static` 里现成的值、测试夹具函数；③ 收严二的三处开放漏了什么：入口参数类型经类型别名、关联类型、泛型实参（`Vec<Box<dyn Tr>>` 在别名后面）、trait 对象在 `static` 或线程局部里、返回值位置的 `impl Trait`；以及它会不会把闭包放大到失去意义（真仓上别的登记组与按条复用的测试函数各多收多少，写成要主 agent 跑的命令）；④ 两份实现不等价：`names_written_where_a_trait_goes`、签名到头一个 `{` 为止、`type_units_by_name` 遍历的文件集合、`use` 路径段的认法，找一段输入让两份给出不同的闭包；⑤ 乙：52 个登记名按限定名比，同名的两个单元（cfg 两臂、两个 impl 块里的同名方法）会不会让没登记的那个被一起排掉；三条测试各证不证得了它的文档句；「显存记账的行为不改判定结局」今天没有会红的检查，给出一个做得成的写法或写明做不成 |

两条腿都按 `crates/` 今天的实现与 `verify-copy` 里的改动来谈；引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。要跑的命令一律写进报告「要主 agent 跑的命令」一节，不自己交任务。

## 四、判据（跑前写死）

- 攻方给出新反例（今天的规则收、收严后不收、改它入口结果变），主 agent 复跑坐实的：那一形要么再收严，要么这套规则撤回；第三轮只攻前两轮站住的形态。
- 两份实现给得出不同闭包的输入，坐实的：两份对齐之前不合。
- 辩方判某条改法够不着它声称修的格的：那一格重做。
- 两条腿都没打中的：记「第二轮一次抽样没打中」，不写成「不会漏连」。
