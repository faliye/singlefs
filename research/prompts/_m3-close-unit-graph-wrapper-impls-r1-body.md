# m3-close-unit-graph-wrapper-impls-r1：键算法两处收窄会不会漏连（2026-11-13）

批次：`records/2026-10-18-里程碑三收尾与近期欠账.md` 第 2842–2847 行那一批。

## 一、题面

用户 2026-11-12 定：GPU 判器的块键「改动一处、其他全改」要修。量出来闭包大的两个来由各配一条改法，用户 2026-11-13 弹窗定「改，走一轮三方」「新立边级排除」。两条都动 D13（验证路线） 已定项 17 的键算法（Rust 一份 `crates/singlefs-unit-graph/src/closure.rs`，Python 一份 `scripts/gate/lib_function_units.py`）。这一轮只问一件事：两条改法会不会让某一份判定的键过窄（真依赖的单元不在闭包里），造成改了代码而判定照旧复用的假绿。主 agent 的倾向是两条都能做，不算结论。

## 二、实现今天的样子（主 agent 的观测，2026-11-13）

方案按 `crates/` 今天的实现来谈。读过的路径与看到的事实：

- blanket 的判法：`crates/singlefs-unit-graph/src/rust_items.rs` 第 1013 行，trait impl 的自身类型里任何深度出现这个 impl 自己的一个泛型参数就算 blanket；注释第 1010–1012 行写明为的是 `[T]` 这类自身类型里没有能被点名的具体名字的 impl。
- blanket 的三处放行：`crates/singlefs-unit-graph/src/closure.rs` 第 1438 行（trait 头进闭包，它的 blanket impl 连成员一起进）、第 1974 行（R-方法：`.x(` 落到 blanket impl 的成员 `x`）、第 2041 行（R-路径：以 `Q` 为 trait 写的 blanket impl 的成员）。全具体的自身类型等被点名：`name_type`（第 2122 行）把给被点名的类型写的 trait impl 连成员带进来（R-隐式），R-方法补上方法名见过的固有 impl 成员。
- 入口签名里的 trait 约束与 `dyn Trait` 走 R-开放（`open_trait`，第 2102 行）：链接集里这个 trait 的全部 impl 都进。
- 当初把「包装<泛型参数>」算进 blanket 的经过：`records/2026-10-13-复用键与全仓改名.md` 第 694、695、703 行（窄口径「只认 Self 是类型参数」会让 `[T]`、`(A, B)` 进不了任何闭包；主 agent 照「多连不少连」定成今天的宽口径；代价当时就量到：`FaultInjectingBlockDevice<Inner>` 一类包装类型的 impl 都算 blanket）。
- Python 一份同一口径：`scripts/gate/lib_function_units.py` 第 263–271 行的规则说明、第 3004 行的判法；弄坏开关 `rust-blanket-self-parameter-only`（第 1111 行）。两份实现由门禁 code-tooling 的 function-units-two-implementations-agree 格在真仓上逐单元对拍。
- 边级排除今天只有 Python 一侧的子进程边：`scripts/gate/lib_function_units.py` 的 `EXCLUDED_PROGRAM_EDGES`（第 13779 行）。`crates/singlefs-unit-graph/` 里没有 Rust 调用边的排除：`grep -rn "EXCLUDED" crates/singlefs-unit-graph/src/` 零命中（量的）。
- 判器组的入口登记：`crates/singlefs-verification/src/judging_units_entries.rs` 第 50 行起 `JUDGING_UNITS_ENTRY_GROUPS`，GPU 判器那一组 16 个入口；同一份文件第 814、823 行有两张「判器闭包里不许有的文件」表，只给测试断言用、不进键。
- 量的（任务 15310，单元图新加的只读子命令 `why` 打每个单元头一回进闭包的那条边；边链全文在 `~/.local/share/singlefs/work/m3-close-gpu-judge-closure/evidence/` 下三份 `group.*.txt`）：GPU 判器入口组的闭包 1361 个 Rust 单元。
  - 故障注入、读计数、核心层挂载会话是这样进来的：建判器表的函数拿 `MemoryPool`（`crates/singlefs-verification/src/memory_pool.rs` 第 414 行 `impl PoolReader for MemoryPool`）→ 它的成员按方法名 `journal_record_offsets_hint` 连到 trait `BlockDevice` 的同名成员 → `BlockDevice` 的 trait 头进闭包 → 第 1438 行把 `impl<Inner: BlockDevice> BlockDevice for FaultInjectingBlockDevice<Inner>`（`fault_injection.rs` 第 1108 行）、`ReadLoggingBlockDevice<Inner>`（`read_tally.rs` 第 206 行）、`DeviceReadOnceInThisMount<'_, '_, Device>`（`crates/singlefs-core/src/mounted_session.rs` 第 1410 行）连成员带进来，再各自铺开。闭包里没有任何单元点名这三个包装类型。
  - 显存记账与起卡一侧是真调用边：`GpuCrashJudge::judge_one_dispatch` 调 `gpu_unit_checks::take_the_readback_after_the_wait_or_record_the_device_lost`（设备丢了往显存记账记一笔）；`GpuCrashJudge::on_card_compiling_these_kernel_sources` 调 `GpuCard::pipelines_compiled_once_on_this_device` 再到 `gpu_memory_ledger` 的 `DeviceConditionOfACard::compile_for_the_work_of_this_thread`；`GpuCard` 结构体的字段点名显存记账的类型。显存记账 `gpu_memory_ledger.rs` 在闭包里 52 个单元，`gpu_unit_checks.rs` 22 个。
- 量的（任务 15312、15314，原型在草稿副本里、只读真仓，读数整行在 `~/.local/share/singlefs/work/m3-close-gpu-judge-closure/evidence/measure-15312.txt`、`measure-15314.txt`，原型相对主工作区的 diff 在同目录 `prototype.diff`）：
  - 改法甲单独用：入口组 1361 → 1063；`fault_injection.rs` 82 → 0、`read_tally.rs` 33 → 0、`mounted_session.rs` 30 → 0、`singlefs-core/src/mount.rs` 23 → 0、`history.rs` 35 → 2；`only_in_it=0`（没有新进来的单元）。
  - 另一条试过的规则（工作区 trait 的 impl 等 trait 进闭包才随类型进）入口组 1361 不变，不做，不在这一轮。

## 三、两条改法（都被攻过零轮）

### 改法甲：自身类型里有工作区类型名的带泛型参数 impl，等那个类型被点名才放行

今天算 blanket 的 impl 里，自身类型的名字（去掉它自己的泛型参数之后）有一个是工作区里定义的类型（struct、enum、union、type 别名）的，不再按 blanket 放行；它照全具体的 impl 办，等那个类型被点名（R-隐式）或方法名、路径落到它且类型已被点名。自身类型里一个工作区类型名都没有的（`T`、`&T`、`[T]`、`[T; N]`、`(T, u32)`、`Box<T>`、`Vec<T>`、`Option<Box<T>>`）照旧按 blanket 放行。R-开放（入口签名里的 trait 约束与 `dyn Trait`）不动：链接集里这个 trait 的全部 impl 照进。

主 agent 的论证（推的，要腿攻）：闭包里的代码要用到 `Wrapper<T>` 上的 trait impl，先要有一个 `Wrapper<…>` 的值或类型，它要么由闭包里某个单元点名 `Wrapper`（构造、签名、字段、别名、turbofish），要么从入口的参数传进来（入口签名点名 `Wrapper`，或写成泛型约束、`dyn Trait`，走 R-开放）。

### 改法乙：登记的边级排除（Rust 一侧）

在判器组的登记里（`judging_units_entries.rs`）给一组入口登记「不沿它收」的模块，GPU 判器那一组登记显存记账 `singlefs_verification::gpu_memory_ledger`；收这一组的闭包时，落到这个模块里的单元不收、不往下走。每条登记带理由。配的自证（照 D13（验证路线） 已定项 17 里边级排除的三格改写到 Rust 调用边上）：

1. 这个模块里的代码不改判定结局：GPU 判器对拍用例（GPU 与 CPU 参照判器逐态相同）在把显存记账里登记的几处行为改坏（变异行）之后仍然逐态相同，或者红在别的断言（记账断言）而不是判定不同。
2. 切边前后少掉的单元只经这个模块可达：切之后的闭包加上这个模块里原来在闭包里的单元，再从它们往下收，等于切之前的闭包。
3. 「键不变而判定变」必红：造一处改动，它落在被排除的模块里而又改了某个崩溃状态的判定字，自证要红。

Python 一份（`lib_function_units.py`）照同一张登记表判，两份实现照旧逐单元对拍。

## 四、分工

| 腿 | 立场 | 攻击面 |
|---|---|---|
| 正推（Sonnet） | 从 Rust 的名字解析与类型规则出发，逐条推改法甲的论证成不成立，列出「值或类型不经点名也不经入口签名就能到闭包里」的每一种语言形态，各判今天的规则与改法甲各收不收得到 | 改法甲的每一种自身类型形态；改法乙三格自证够不够 |
| 攻方（Opus） | 找反例：一段代码，改它会改某个入口的行为，而改法甲或改法乙之后它不在那个入口的闭包里（今天在）。优先在真仓里找，找不到再造最小夹具 | ① 类型不被点名而值到得了：返回 `impl Trait` 的函数、闭包与函数指针、宏展开里拼出的类型名、类型别名（别名被点名而原名没有）、关联类型（`type Output = Wrapper<T>`）、`Default::default()` 与推断出来的类型、`From`/`Into` 转换、迭代器适配器；② 经另一个 crate 的泛型函数把包装类型传回来；③ trait 对象在闭包内部产生（今天「闭包内部的 dyn Trait 不收」靠的就是具体类型被点名，改法甲之后包装类型没被点名时怎样）；④ 两份实现在「工作区类型名」上认得不一样的形态（同名的类型在两个 crate、条目级宏生成的类型、`Self`）；⑤ 改法乙：显存记账里有没有会改判定结局的路径（设备丢失之后的重试、每卡只编一次管线拿错管线、宽度封顶改了派活的切法进而改判定字），排除的写法（按模块）会不会把别的模块里经它转出来的真依赖一起切掉 |

两条腿都按 `crates/` 今天的实现来谈；引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。要跑的命令（单元图命令行、夹具、探针）一律写进报告「要主 agent 跑的命令」一节，不自己交任务。

## 五、判据（跑前写死）

- 腿给出一个反例（真仓或夹具里，一段代码的改动会改入口行为、而改法之后不在闭包里），主 agent 现查坐实的，那条改法照原样不做：要么收严到反例进不来，要么撤回。
- 两条腿都没给出反例的，只记「一次抽样没打中」，不写成「不会漏连」；改法照做，键算法这两条在决策正文里写明被攻过一轮、哪几面没打中。
- 改法乙三格自证里有一格腿判「证不了它要证的事」的，那一格重写之后再做。
