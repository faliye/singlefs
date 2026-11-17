# m3-close-unit-graph-wrapper-impls-r1 正推腿报告（Sonnet，2026-11-13）

立场：从 Rust 的名字解析与类型规则、`crates/` 今天的实现推「应该是什么」，再与改法甲、改法乙逐格比。方案按 `crates/` 今天的实现来谈。
没读禁读清单里的任何文件。没跑项目代码、没往统一任务服务交任务：要跑的写在末尾「要主 agent 跑的命令」。
本腿没写模型目录；写了一份只读夹具（草稿目录，未跑过，所以下面凡写「夹具」的判定都是「推的」，等主 agent 跑）：

- 夹具根 `/home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r1-sonnet/fixture-tree`，驱动脚本 `/home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r1-sonnet/membership.sh`
- sha256（量的）：`membership.sh` 6fab6e5008b98ed5d6ac4bdbefd25e12daf48f5310f7989e513ad5ec04b6e11e；各夹具 `lib.rs`：fcfield dc654e62233a767ec6b20a60e04257c02c8748cf2d75096391bef226aa94df82、fcbound f577aacdfd3f6f84e9a0ac81deff4c9d0e3c16d6e0e213fb7955f6b8e0f4e9c1、fcrename 8a37f1a61711640b7f7522a688d97fb402fec608f8e25215fbf1e66025c72119、fcaliasimpl 264caf31867815349b5da702b9d2df948ef51823e74731da89948cc6849641b4、fccollide 5c7581a55665fea9a4174628e34ae00e3f7845ac5501de9c7f394bf80cfa4c9e、fccontrol 3db9674fbcf3205e26a1de9a8dda5d1aec43ff85af4dfa3b420d7a944c0ef370

## 〇、先把「改法甲」读成代码：文字与原型不是一回事

原型 diff（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/evidence/prototype.diff`，只读）里属于改法甲的只有环境变量 `PROTOTYPE_WRAPPER_IMPLEMENTATIONS_WAIT_FOR_THEIR_TYPE` 守着的三处；另一条「trait impl 等 trait」的试验代码同在 diff 里，由另一个变量守着，不在这一轮。

| 处 | 今天 | 原型 |
|---|---|---|
| trait 头进闭包带 blanket impl（`closure.rs:1435`） | 无条件 `push_container_and_members` | 自身类型里有「没被点名的工作区类型名」就不推，等 `name_type` 推 |
| R-方法放行（`closure.rs:1974`） | `shape.is_blanket_implementation` 或自身类型有被点名的名字 | blanket 且自身类型里没有任何工作区类型名，或自身类型有被点名的名字 |
| R-路径放行（`closure.rs:2041`） | 同上 | 同上 |

读到的事实：

- 原型判「工作区类型名」的写法是 `self_type_names.iter().any(|name| self.workspace_type_names.contains(name))`（`prototype.diff` 里 `self.workspace_type_names.contains(name)` 那几行），`workspace_type_names` 取全部 `unit.kind.is_type()` 单元的名字。没有「去掉 impl 自己的泛型参数」这一步。
- 这一步现在也做不了：`crates/singlefs-unit-graph/src/units.rs:381` 的 `ImplementationShape` 只存 `pub self_type_names: Vec<String>,`、`is_blanket_implementation` 等，泛型参数集合只活在 `rust_items.rs:932` 解析 impl 头的局部变量里，不进形状。背景材料写的是「自身类型的名字（去掉它自己的泛型参数之后）有一个是工作区里定义的类型」，原型做的是不去掉。两者不等价，后面形态六量这个差。
- 放行的粒度：`name_type`（`closure.rs:2123`：「if !self.named_types.insert(type_name.to_string()) {」）对 `implementations_by_self_type_name` 里登了这个名字的每一份 trait impl 全推（`workspace.rs:1311`：「for name in &implementation.self_type_names {」，登的是自身类型里的每一个名字，不止头一个）。所以原型实际是「自身类型里任一工作区类型名被点名就放行」，不是「头类型被点名」。对 `Outer<Wrapper<T>>` 这是更宽的一侧（多连不少连）。
- R-开放不动：原型没碰 `open_trait`（`closure.rs:2104`：「|| !self.dyn_traits_seen.insert(trait_name.to_string())」那一函数）。
- 点名的定义：一个类型「被点名」= 这个类型的单元进了闭包（`closure.rs:1414`：「if unit.kind.is_type() {」之后调 `name_type`），或条目级宏调用生成了它（`closure.rs:1418-1432`）；起始集是 `closure.rs:1274`：「named_types: ALWAYS_NAMED_TYPES」，即基本类型与 `String`、`Vec`、`Option`、`Result`、`Box`、`Self`（`closure.rs:895`）。类型单元怎么进闭包：只靠 R-裸名把某个单元里写的名字解到类型单元（`follow_bare_names`），或 R-结构把成员连到它的头（`closure.rs:1514-1517`）。

## 一、背景材料没写、但改法甲实际继承的设计句

`closure.rs:9`：「闭包内部写的 dyn Trait 不收——」；`closure.rs:10`：「值只能由某个具体类型经 unsized 转换而来，那个类型在闭包里被点名（R-隐式 带进它的 trait impl）或从入口签名传进来」。
这是今天对 `dyn Trait` 与全具体类型 impl 已经在用的那条论证，改法甲的论证与它同形：把它搬到泛型包装类型上。因此改法甲不引入新种类的缺口，它把今天对全具体 impl 与 `dyn` 已有的缺口，扩到泛型包装类型的 impl 上（推的；打中的缺口见下节各形态）。
代价对照：`records/2026-10-13-复用键与全仓改名.md:703`：「代价：放宽后泛型包装类型的 trait impl（`ReadStageCache<'_, Reader>`、`FaultInjectingBlockDevice<Inner>` 一类）都算 blanket」；当初要保的是 `[T]` 这类没有名字可等的形态（同一份 `:695`：「窄口径」），包装类型是被宽口径顺带带进来的。
现有的自证把「包装类型的 impl 不等点名」写成了期望：`scripts/gate/lib_function_units.py:18198`：「# Box、Option 与 u32 在 PRIMITIVE_TYPES 里、算被点名，这五份总是进闭包；Sealed<T> 没被点名，只有按 blanket 放行才进」；Rust 一侧 `crates/singlefs-unit-graph/tests/unit_graph_blanket_implementations_are_those_whose_self_type_mentions_a_generic_parameter.rs:165` 钉的是全具体类型没被点名就不进（「impl OnAConcreteType for Plain：自身类型全具体、Plain 没被点名，不算 blanket、不进闭包」）。改法甲落地时 python 的 `Sealed<T>` 那一格要翻成「没点名就不进」，两份实现的对拍样本同改。

## 问题 1：改法甲的论证成不成立

论证（背景材料第三节）：`Wrapper<T>` 值要进闭包，要么闭包里某个单元点名 `Wrapper`，要么从入口传进来（入口签名点名 `Wrapper`，或写成泛型约束、`dyn Trait`，走 R-开放）。
先给结论，再逐形态：**对「值在闭包代码里被造出来」成立（推的）；对「值在闭包外造好、经入口签名以外的通道进来」不成立；对「名字写了但没解到类型单元」不成立。** 后两类各有最小片段，夹具已写好、待跑。

判「收不收得到它的 trait impl」时，今天的规则收 `Wrapper<T>` 上的 trait impl 靠三条路：trait 头进闭包带 blanket impl（`closure.rs:1435`）、R-方法按方法名放行 blanket（`closure.rs:1974`）、R-路径按 trait 名放行 blanket（`closure.rs:2041`）；另有 R-隐式（`name_type`，类型被点名才带）与 R-开放（入口签名）。改法甲拿掉前三条对「自身类型里有工作区类型名」的 impl 的放行，只留后两条。

### 甲类：论证成立的形态（值在闭包代码里造出来，名字必在某个闭包单元里）

下表「今天」「甲后」两列：收 = 收得到。成立的形态两列都是收（甲后靠 R-隐式）。

| # | 形态 | 今天 | 甲后 | 判 | 依据 |
|---|---|---|---|---|---|
| 1 | 构造点写名字：结构体字面量、`Wrapper::new`、元组结构体 `Wrapper(x)`、`Self` | 收 | 收 | 成立 | 单元的名字走 R-裸名到类型单元（`closure.rs:1556` 起的 `follow_bare_names`），类型单元进闭包即 `closure.rs:1414` 的 `name_type` |
| 2 | 类型别名被点名、原名没有（`type Dev = Wrapper<Mem>`） | 收 | 收 | 成立 | `TypeAlias` 的 `resolves_its_own_names` 为真（`units.rs:129-149`，只有 `use`、`mod`、`extern crate` 三类为假，`UnitKind::ExternCrate` 那一行在 `units.rs:134`），别名单元自己的名字里有 `Wrapper`，一进闭包就把 `Wrapper` 连进来。真仓实例：`crates/singlefs-verification/src/gpu_unit_checks.rs:295` 「type CompiledPipelinesBySourceDigest = CompiledOnceBySourceDigest<Vec<wgpu::ComputePipeline>>;」 |
| 3 | 返回 `impl Trait` 的函数、闭包、函数指针（在闭包里造出来的） | 收 | 收 | 成立 | 造值的函数体或闭包体在闭包里，体内写了 `Wrapper`；整个函数体的词都是这个单元的名字（`rust_items.rs:210` 的 `references_of_tokens`） |
| 4 | 宏：`macro_rules!` 体里拼出类型名；条目级宏调用生成的类型 | 收 | 收（宏生成的类型原型不收窄，见形态九） | 成立 | 宏体里的名字都进该单元名字（`closure.rs` 头注 R-宏）；条目级宏生成的类型进闭包时算被点名（`closure.rs:1418-1432`）；proc-macro crate 在 `finish` 里整个拒算（`closure.rs:2791` 起），不会静默 |
| 5 | 关联类型 `type Out = Wrapper<T>`、`<X as Tr>::Out` | 收 | 收 | 成立（有前提） | `type Out = …` 写在 impl 里，它的名字属于那份 impl（头或成员）；impl 要在闭包里才有这些词，`X` 被点名则 `name_type(X)` 带进这份 impl。前提：X 本身被点名；X 是泛型参数时，实例化点（`run::<X>()`）写了名字 |
| 6 | `Default::default()`、`From`/`Into`、`?`、`collect`、`parse`、`sum` 的推断 | 收 | 收 | 成立（有前提） | Rust 推断要有决定目标类型的一处写法：字面注解、被调函数的签名、字段类型、impl 的关联类型；这一处是个会执行的单元，在闭包里，名字就在。前提是决定它的那一项在闭包里——决定它的项在闭包外就是形态十一 |
| 7 | 扩展 trait：`impl<I: Iterator> Ext for I { fn adapted(self) -> Adapter<Self> }` | 收 | 收 | 成立 | 这份 impl 的自身类型只有泛型参数，仍是 blanket（`rust_items.rs:1013`），按方法名放行；体里写 `Adapter`，`Adapter` 的 `Iterator` impl 走 `name_type`。依赖 R-隐式，不依赖今天的 blanket 放行 |
| 8 | 隐式调用：运算符、`Deref`、`Drop`、`Display`、`for` 循环、`?` | 收 | 收 | 成立 | 没有方法名可匹配，今天与甲后都只靠类型被点名走 `name_type`；甲对它们没有改动 |
| 9 | trait 对象在闭包内部产生（具体类型转 `dyn`） | 收 | 收 | 成立 | 转换点写了具体类型的名字，`closure.rs:10` 的论证；今天对全具体类型也只靠这一条 |
| 10 | 外部（工作区外）依赖里的泛型函数把值传回来 | 收 | 收 | 成立（推） | 外部 crate 写不出本仓类型的名字；类型实参由闭包里的调用点定，那一处写了名字（同形态六）。工作区外 path 依赖按调用关系连（`closure.rs` 头注），也是调用点点名 |

形态九补一句：原型的 `workspace_type_names` 只含单元种类是 struct、enum、union、type 的名字；条目级宏生成的类型不在里面。这类类型上的 blanket-形 impl 在原型里照旧按 blanket 放行，不收窄也不漏，但 python 一份要与它对拍，宏生成类型的名字表两边要同口径（攻方 ④ 的面）。

什么现象会推翻甲类：找到一个能在闭包里执行、其 `Wrapper<T>` 值的类型只由「不在闭包里、也不是入口签名」的东西决定的程序。下面乙类就是这样的东西。

### 乙类：论证不成立的形态（今天收，甲后收不到）

下面每一种，今天都是靠 trait 头进闭包时 blanket impl 连同成员一起进（`crates/singlefs-unit-graph/src/closure.rs:1435`：「// trait 头进闭包时它的 blanket impl 连同成员一起进」）收到的；甲之后这条路对工作区包装类型关掉，而名字或值的来路不在甲的两条前提里。夹具各一份（草稿目录，未跑），预期读数写在「待跑」一格。

**形态十一：值从入口的接收者、参数类型的字段、或泛型 impl 头上的约束进来（入口签名里没有 `dyn`、没有约束）。**

- 最小片段（夹具 `fcfield`、`fcbound`）：

```rust
pub trait FieldDev { fn field_read(&self) -> u8; }
pub struct FieldFaulty<Inner>(pub Inner);
impl<Inner: FieldDev> FieldDev for FieldFaulty<Inner> { fn field_read(&self) -> u8 { self.0.field_read() ^ 1 } }
pub struct Session { pub dev: Box<dyn FieldDev> }
impl Session { pub fn entry_field_dyn(&self) -> u8 { self.dev.field_read() } }   // 入口 fcfield::Session::entry_field_dyn
```

```rust
pub struct Holder<Device> { pub dev: Device }
impl<Device: BoundDev> Holder<Device> { pub fn entry_bound(&self) -> u8 { self.dev.bound_read() } }   // 入口 fcbound::Holder::entry_bound
```

- 为什么今天收：`self.dev.field_read()` 有 `.field_read(` 方法名，R-方法见到后把 trait 里同名成员连进来（`closure.rs:1969` 起 `container_unit.kind == UnitKind::TraitHead` 一支恒放行），成员连头（R-结构）、trait 头进闭包，`FieldFaulty<Inner>` 的 impl 是 blanket（自身类型里有泛型参数，`rust_items.rs:1013`：「let is_blanket_implementation = self_type_names」），随 trait 头进来。
- 为什么甲后收不到：入口收闭包时只对入口函数**签名里**的 trait 名跑 R-开放：`closure.rs:3177` 「run.open_trait(trait_name);」，`signature_trait_names` 只在函数体 `{` 之前扫 `impl`、`dyn`、`:`、`+` 后面的名字（`rust_items.rs:283` 「if matches!(text_at(position), "impl" | "dyn" | ":" | "+")」）。`&self` 的方法签名里一个这样的词都没有；`Session` 字段里的 `dyn FieldDev` 在 `Session` 结构体单元里，不是入口签名；泛型 impl 头上的 `Device: BoundDev` 在 impl 头单元里，也不是入口函数签名。闭包内部的 `dyn Trait` 又只在弄坏开关打开时才叫 R-开放（`closure.rs:1403` 「.is_intact(BrokenRule::DynTraitsOpenEveryImplementation)」）。`FieldFaulty` 的名字没有任何闭包单元写，甲让它的 impl 等被点名，永远等不到。
- 这不是「调用者造值」的假设：入口由主 agent 点名交的 `<入口 fn>`，调用者（测试、harness、另一个 crate）在闭包外造出 `Session { dev: Box::new(FieldFaulty(..)) }` 或 `Holder { dev: FieldFaulty(..) }` 再传进来，是合法的 Rust。今天的头注 `closure.rs:10`「……或从入口签名传进来」只承诺了签名。
- 真仓里有没有：登记的 16 个 GPU 判器入口里我读了 `GpuCrashJudge`（`crates/singlefs-verification/src/crash_judge_dispatch.rs:396` 「pub struct GpuCrashJudge<'card> {」），读到的字段（到 `dispatch_width_schedule` 为止）全是具体类型（`card: &'card GpuCard`、`wgpu::ComputePipeline`、缓冲），没有 `dyn`、没有带约束的泛型字段；这一个入口对这个形态打不中（量的，只读了这一个结构体）。其余 15 个入口的接收者与参数类型、`GpuCard` 之外的字段类型没逐个读。观察者登记表（`crates/singlefs-verification/src/judging_units_entries.rs:159` 起的 `OBSERVER_ENTRIES`）我读了两个入口的签名（`observe_the_mount_continuation`、`observe_the_publish_boundary`），参数是 `&CrashImage<'_>`、`&RecoveryReport`、`&mut CrashPointReplayObserverCounts` 与各自的 `inputs`/`prepared`，没展开它们的字段。别的组、按条复用的测试函数（入口是无参的 `#[test] fn`）没查。
- 判：**论证第二肢（「入口签名点名……走 R-开放」）过窄，对形态十一不成立。** 推翻它的现象：夹具两份在甲臂的闭包里仍有 `field_read`、`bound_read` 的 `FieldFaulty`、`BoundFaulty` 成员行（那就是我读错了某条边）。
- 待跑：`fcfield`、`fcbound` 今天臂预期 `FieldFaulty` 与 `BoundFaulty` 的成员行在闭包里，甲臂预期都不在。

**形态十二：值经全局量、线程局部量、链接期登记（`inventory`、`linkme`、`ctor` 一类）进来，登记的代码在闭包外。**

- 今天：trait 头在闭包里时，同形态十一收到包装类型的 blanket impl；甲后：收不到，同一原因（没有闭包单元点名它）。
- 判：形态存在，论证对它不成立；仓里无实例（量的：`grep -rnE 'inventory|linkme|ctor|typetag' --include=Cargo.toml` 只在实验装置清单里命中一个目录名子串，没有这类依赖；全局分配器由 `closure.rs:2075` 的 `push_global_allocators_of_the_linked_targets` 单独收）。

**形态十三：名字写了，但 R-裸名没把它解到类型单元（`use … as …` 在别的模块里改名）。**

- 最小片段（夹具 `fcrename`）：

```rust
pub mod devices { /* RenameFaulty<Inner> 与它的 impl */ }
pub mod prelude { pub use crate::devices::RenameFaulty as Wrap; }
pub fn entry_renamed() -> u8 { let device = crate::prelude::Wrap(crate::devices::RenameMem); device.rename_read() }
```

- 为什么解不到：`use` 单元不解自己的名字（`units.rs:129-134`，`UnitKind::Use` 在 `resolves_its_own_names` 的假一支），也不按名字被找到（`units.rs:118-123` 的 `is_found_by_bare_name` 把 `UnitKind::Use` 放在假一支），别名只在「引用它的单元与那条 use 在同一个文件、同一段行内模块」时才还原成原名（`closure.rs:1526-1553`，`uses_by_scope` 以 `(unit.file, unit.local_module_path)` 取）。`entry_renamed` 在入口所在模块，`pub use … as Wrap` 在 `prelude` 模块，不同作用域，`Wrap` 解到零个候选，`RenameFaulty` 的结构体与 impl 都没进来。
- 今天收：同形态十一，`device.rename_read()` 触发 trait 头，blanket impl 的头里写了 `RenameFaulty`，结构体单元跟着进。甲后收不到。
- 这是今天就有的缺口（跨模块 `use as` 改名的类型，它自己的结构体、固有 impl、全具体 trait impl 今天就进不了闭包），今天被泛型包装类型的 blanket 放行碰巧盖住；甲把它盖不住的面扩大到泛型包装类型的 trait impl。
- 真仓里有没有：`grep -rnE '^\s*(pub(\([a-z]+\))? )?use [^;]* as [A-Za-z_]+' --include=*.rs crates`（排除单元图自己、排除 `as _`）命中 12 行：函数、常量、模块与 crate 名改名，`crates/singlefs-checker/src/walk.rs:1363` 与 `:1364` 在函数体里把两个枚举改名成 `Exit`、`Page`（与引用它们的单元同函数，走 `closure.rs:1547-1550` 的还原），一处在字符串字面量里（`crates/singlefs-verification/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs:931`）；没有一条把泛型包装类型跨模块改名转出（量的：命中行读过，`history.rs:108` 那一行转出的是 `macro_rules!`，定义在 `crates/singlefs-core/src/admission.rs:1122`）。
- 判：形态存在，真仓现无实例；论证对它不成立。推翻它的现象：夹具 `fcrename` 甲臂的闭包里有 `rename_read` 的 `RenameFaulty` 成员行。

**形态十四：impl 写在类型别名上，值用原名构造。**

- 最小片段（夹具 `fcaliasimpl`）：

```rust
pub struct AliasInner<Inner>(pub Inner);
pub type AliasFaulty<Inner> = AliasInner<Inner>;
impl<Inner: AliasDev> AliasDev for AliasFaulty<Inner> { fn alias_read(&self) -> u8 { self.0.alias_read() ^ 1 } }
pub fn entry_alias_impl() -> u8 { let device = AliasInner(AliasMem); device.alias_read() }
```

- Rust 允许 `impl … for <别名>`，别名展开后就是原类型上的 impl。单元图只按 impl 头里写出来的标识符登记自身类型（`rust_items.rs:1009`：「let self_type_names = names_in(for_position + 1, where_index);」，不展开别名），索引按这些名字登（`workspace.rs:1311`）。这份 impl 登在 `AliasFaulty` 与 `Inner` 名下，不登在 `AliasInner` 名下。
- 今天收：自身类型里有泛型参数，blanket，随 trait 头进。甲后：`AliasFaulty` 是工作区类型名（别名单元，原型的 `is_type()` 包括 `TypeAlias`），没人点名它，`name_type("AliasInner")` 又不带这份 impl → 收不到。
- 真仓里有没有：别名在测试文件里很多（如 `crates/singlefs-verification/tests/write_accounting_per_structure.rs:39` 的 `type MemoryDevice = FaultInjectingBlockDevice<RecordingBlockDevice<SparseBlockDevice>>;`），但我拿 188 个 `type 名` 行里取出的名字（含 trait 里的关联类型，所以偏宽）逐个 grep 单行写法的 `impl … for <名>`，命中 5 行，5 行所在文件里那个名字的定义都是 `struct`（量的，逐个核了），没有写在别名上的 impl；多行写法的 impl 头（`for` 另起一行）这一遍没覆盖。
- 判：形态存在，真仓现无实例；论证对它不成立。非泛型的 `impl Tr for 别名` 今天就收不到，同一缺口。推翻它的现象：夹具 `fcaliasimpl` 甲臂闭包里有 `alias_read` 的别名 impl 成员行。

**形态十五：impl 自己的泛型参数与工作区里某个类型同名（纯 blanket 的 impl）。**

- 最小片段（夹具 `fccollide`）：

```rust
pub struct Reader;                                   // 工作区里有个类型叫 Reader
impl<Reader: CollideDev> CollideDev for &Reader { … }  // 泛型参数也叫 Reader，自身类型里只有这个名字
```

- 原型的判法不去掉泛型参数（见第〇节），`&Reader` 的自身类型名字集是 `{Reader}`，`Reader` 在 `workspace_type_names` 里，被当成「带工作区类型名的包装」，等那个结构体 `Reader` 被点名；调用 `call_through(&CollideMem)` 实际执行的是 `<&CollideMem as CollideDev>`，也就是这份 impl。→ 甲后收不到，今天收（blanket）。
- 真仓这个同名并不空：`crates/singlefs-verification/src/verification_read.rs:64` 「struct Reader<'pool>(&'pool dyn PoolReader);」，测试里还有 `enum Reader`、`struct Reader<'bytes>`（`grep -rnE '^\s*(pub(\([a-z]+\))? )?(struct|enum|union|type) Reader\b' --include=*.rs crates`（排除单元图自己）命中 3 处，量的）；而泛型参数叫 `Reader` 的 impl 有十几份（如 `crates/singlefs-verification/src/fault_injection.rs:391` 「impl<Reader: PoolReader + ?Sized> PoolReader for PoolReaderWithUnreadableRootRingSlots<'_, Reader> {」）。这些份的自身类型里另有真正的包装类型名，所以放行与否由那个名字定，同名只会多放行（多连，不漏）。漏的只是「自身类型里只剩泛型参数名」的 impl；真仓现在没有这种形状（量的：对 `crates/` 下单行写法的 impl 头与 `for` 另起一行的写法各 grep 一遍，自身类型是 `&泛型参数`、`&mut 泛型参数`、`Box<泛型参数>`、裸泛型参数的零命中，仅有的纯泛型形是 `Option<Element>`、`Vec<Element>`（`e171_three_oracle_collusion.rs:4741`、`:4760`）与 `[(DeviceIdentity, Device)]`（`crates/singlefs-core/src/recovery.rs:546` 「impl<Device: BlockDevice> PoolReader for [(DeviceIdentity, Device)] {」））。
- 判：**原型（不去掉泛型参数）在这个形态上漏连；背景材料的文字（去掉泛型参数之后）不漏。** 要落地，`ImplementationShape` 得存泛型参数集合（`crates/singlefs-unit-graph/src/units.rs:381` 起的形状），两份实现同改；或者「工作区类型名」的判法改成「自身类型的名字集合减去 impl 自己的泛型参数集合」。推翻它的现象：夹具 `fccollide` 甲臂闭包里有 `collide_read` 的 `&Reader` 成员行。

**对照（预期两臂的差，夹具 `fccontrol`）**：入口点名 `ControlNamedFaulty`；`ControlNeverNamed<Inner>`、`Box<Inner>`、`[Inner]` 三份 impl 没人点名。预期今天臂四份 `control_read` 的 impl 成员都在；甲臂 `ControlNeverNamed` 那份不在，`ControlNamedFaulty`、`Box<Inner>`、`[Inner]` 与 `ControlMem` 的仍在。这一格证甲的收窄真发生、且只发生在该发生的地方。

### 真仓对账：GPU 判器入口组里甲收掉的 trait impl 有没有走形态十一、十二的通道

读的是主 agent 留的闭包读数：`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/proto-out/` 里 `gpu.today.txt` 与 `gpu.wrapper.txt`，`comm -23 <(sort gpu.today.txt) <(sort gpu.wrapper.txt)`（量的，只读）：今天臂有、甲臂没有的单元行共 298 行（不含键行），其中 `impl` 行 36 条，其中 trait impl 13 条：`BlockDevice` 6 条（`DeviceReadOnceInThisMount`、`FaultInjectingBlockDevice`、`DeviceLosingPowerAtTheFirstJournalRecordWrite`、`DeviceReadingZerosOverHiddenRanges`、`RecordingBlockDevice`、`ReadLoggingBlockDevice`），`PoolReader` 7 条（`ReadStageCache`、`SystemConfigurationSlotReadsRecordedByTheCheck`、`PoolReaderCountingUnitReadsWithOneCopyFailing`、`PoolReaderOfOneReadPass`、`PoolReaderStraightFromTheDisk`、`PoolReaderWithUnreadableRootRingSlots`、`ReadCountingPoolReader`），其余 23 条是这些包装类型的固有 impl 与别的类型的固有 impl。
这 13 条的 trait 只有两个：`BlockDevice`、`PoolReader`。要走形态十一的通道，得有一个入口接收者或参数类型的字段、或入口所在 impl 头的约束，装着 `dyn BlockDevice`、`dyn PoolReader` 或这两个 trait 的泛型约束。我读到的：
- `&dyn PoolReader` 在验证包里只作函数参数出现（`crates/singlefs-verification/src/verification_read.rs:228` 起十多处、`mount_transparency.rs:155`、`:258`、`:2405`，`grep -rnE 'dyn (PoolReader|BlockDevice)'` 排除 `bin/`，量的），没有出现在结构体字段里；`dyn BlockDevice` 零命中。
- 两个建表入口的参数是 `&JudgeFlow<'_>` 等纯数据，`JudgeFlow` 的字段是 `&MemoryPool`（具体类型）、切片与一个 `&dyn Fn`（`crates/singlefs-verification/src/crash_judge_tables.rs:306-313`），`Fn` 在本仓没有 impl 可连。
- `GpuCrashJudge` 的字段读过（见形态十一）。
没读：其余入口的参数类型逐个展开；那 13 个包装类型在真运行里从哪造（它们是核心层与验证包里的故障注入与计数包装，我没追构造点）。对 GPU 判器这一组，形态十一、十二这两条通道在已读的入口上没有打中；这是读源码得到的，不是对闭包的实测，也不覆盖别的组。

### 问题 1 小结：改法甲的论证成不成立

- 成立的部分：值在闭包代码里造出来的每一种写法（形态 1-10），名字必在某个会执行的闭包单元里，R-隐式（`name_type`）把它的 trait impl 带进来。这一部分的依据是 Rust 的类型必须经由某处写法确定（推的，没有反例；构造不出反例不等于没有，只抽了这一轮的样）。
- 不成立的部分：(a) 值在闭包外造好、经入口签名以外的通道进来（形态十一、十二）；(b) 名字写了但没解到类型单元（形态十三、十四）；(c) 原型把泛型参数名与工作区类型名混了（形态十五）。(a) 是背景材料论证第二肢写窄了：「入口签名……走 R-开放」只读函数签名，不读入口所在 impl 头的约束、不读入口参数与接收者类型的字段（`closure.rs:3172-3178`、`rust_items.rs:280-293`）。(b)(c) 是单元图解析与形状的现状。
- 都继承自今天对全具体类型 impl 与 `dyn` 的同一条论证（`closure.rs:10`）：甲不增加缺口的种类，把已有缺口的暴露面扩到泛型包装类型。真仓对这一组入口：形态十一读了一个入口没打中，十二到十五没有实例；这是一次抽样，不写成「不会漏连」。
- 收严到反例进不来的写法（都是对 `crates/` 的改动，零轮，没攻过）：
  1. 形态十一：入口收闭包时，对入口函数所在 impl 头的泛型约束名、与入口参数、接收者类型（沿字段、不展开别名外的泛型实参）里写的 `dyn Trait` 与约束名，也跑 `open_trait`（`closure.rs:3170-3178` 那一段加两个来源）；
  2. 形态十三：`name_type` 的「被点名」再加一条：任一 `use … as …` 单元的原名是工作区类型名时，那个原名算被点名（只多连）；
  3. 形态十四：`is_type()` 里的 `TypeAlias` 不算「等被点名」的类型名（impl 自身类型里只有别名与泛型参数时，仍按 blanket 放行）；
  4. 形态十五：`ImplementationShape` 存 impl 自己的泛型参数集合，工作区类型名的判法先减去它（Rust 在 `crates/singlefs-unit-graph/src/units.rs:381`、`rust_items.rs:1009-1020`，python 在 `scripts/gate/lib_function_units.py:3013`）。

## 问题 2：十二种自身类型形态，改法甲各放行不放行、对不对

「放行」指不等自身类型里的工作区类型名被点名（trait 头带、R-方法、R-路径照今天放）。名字集是 `rust_items.rs:1009` 取的自身类型里全部标识符（去掉 `dyn mut const impl for where`）；生命周期不是标识符：词法器把 `'a` 标成 `TokenKind::Lifetime`（`crates/singlefs-unit-graph/src/lexer.rs:317` 「kind = TokenKind::Lifetime;」），impl 头解析用的 `is_identifier` 只认 `TokenKind::Identifier`（`crates/singlefs-unit-graph/src/rust_items.rs:718` 「self.kind_of(index) == Some(TokenKind::Identifier)」），所以它不进名字集，也不进泛型参数集。
先说一个读原型读出来的偏差，它影响第 5、12 行：R-方法与 R-路径的第二个放行条件是自身类型里有任一名字在 `named_types` 里（`crates/singlefs-unit-graph/src/closure.rs:1978` 与 `:2045` 的 `.any(|type_name| self.named_types.contains(type_name))`），而 `named_types` 起始集含 `u32`、`Vec`、`Option`、`Box` 等（`closure.rs:895`）。所以自身类型里带这几个名字的 impl，按方法名放行这条路原型没有收窄；只有「trait 头带它」那一处收窄了。

| # | 自身类型 | 名字集（工作区类型名加粗） | 原型放行？ | 文字规则放行？ | 对不对 |
|---|---|---|---|---|---|
| 1 | `T` | {T} | 放行 | 放行 | 对：没有任何名字可等，不放行就进不了任何闭包（`records/2026-10-13-复用键与全仓改名.md:695`）。例外：`T` 与工作区某类型同名时原型不放行（形态十五），文字规则放行 |
| 2 | `&T` | {T} | 放行 | 放行 | 对，同 1 |
| 3 | `[T]` | {T} | 放行 | 放行 | 对：这是当初宽口径要保的形态 |
| 4 | `[T; N]` | {T, N}，`N` 是 const 泛型参数（`rust_items.rs:949` 里 `const` 后的名字算泛型参数） | 放行 | 放行 | 对 |
| 5 | `(T, u32)` | {T, u32} | 放行 | 放行 | 对（没有工作区名可等） |
| 6 | `Box<T>` | {Box, T} | 放行 | 放行 | 对；`Box` 恒被点名（`closure.rs:895`），本来就走方法名那条 |
| 7 | `Vec<T>` | {Vec, T} | 放行 | 放行 | 对，同 6 |
| 8 | `Option<Box<T>>` | {Option, Box, T} | 放行 | 放行 | 对，同 6 |
| 9 | `Wrapper<T>` | {**Wrapper**, T} | 不放行，等 `Wrapper` 点名 | 不放行 | 对，条件是形态十一到十五被收住；今天这一格的论证只对「值在闭包里造出来」成立 |
| 10 | `Wrapper<'a, T>` | {**Wrapper**, T}，`'a` 不入名字集 | 不放行 | 不放行 | 对，同 9。顺带：`impl<'a> Tr for Wrapper<'a>`（只有生命周期参数）今天就不是 blanket（`rust_items.rs:945-951` 只收标识符，`'a` 是生命周期），今天就等点名，甲不改它 |
| 11 | `Outer<Wrapper<T>>` | {**Outer**, **Wrapper**, T} | 不放行，`Outer`、`Wrapper` 任一被点名就放（`name_type` 对每个名字都登，`workspace.rs:1311`） | 文字没说「哪一个」 | 对且偏宽：值要两个类型同时存在，「任一」比「全部」多放；多连不漏连。落地时在决策正文里写明是「任一」 |
| 12 | `(Wrapper<T>, u32)` | {**Wrapper**, T, u32} | **只收窄了 trait 头那一处**：`.method(` 照放（`u32` 在 `named_types` 起始集里） | 不放行 | 偏差：原型对这一形态的收窄不完整，只是多连，不漏连；文字规则（等 `Wrapper` 点名）与原型不等价。含 `Vec`、`Option`、`Box`、基本类型名的复合形态（真仓有 `Vec<(DeviceIdentity, Device)>`：`crates/singlefs-core/src/recovery.rs:614`）同样。要让它与文字一致，「被点名」的第二个放行条件要把起始集那些名字排除，只看工作区类型名 |

十二行里「对」的判据是：它的 trait impl 在闭包里要执行，就得有一个这个自身类型的值；放行的前六行是因为没有名字能证明「值没有」，所以只能多放；后六行是有名字可等。这一判据对后六行的成立范围就是问题 1 的甲类，不含乙类。
还要写的一条（推的）：第 1-8 行放行之后，trait 头在闭包里时，这些 impl 的成员全进。这一部分的收益（量的：入口组 1361 → 1063，`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/evidence/measure-15314.txt` 的 `MEASURE gpu` 两行；`measure-15312.txt` 是另一条「等 trait」规则，1361 不变）来自第 9-12 行里的包装类型，不来自 1-8；也就是甲收走的全是后六行，后六行的论证好坏就是甲的全部风险。

什么现象会推翻第 1-8 行「对」：一份闭包里要执行的 `impl<T> Tr for [T]` 一类 impl，在甲臂闭包里不在——不会出现，因为这几行的放行条件甲没动。什么现象会推翻第 9-11 行「对」：同问题 1 乙类。

## 问题 3：改法乙三格自证各自证不证得了它要证的事

先对账：D13 已定项 17 里边级排除的原话是 `.claude/kb/decisions/13-验证路线.md:434`：「边级：一条只打提示、不改判定结局的子进程边，按实测排，不沿它收」，三格自证写在同一行：「边级排除要配三格自证：这条路上的任何异常不改包装退出码；判法读包装输出前去掉这条路打的行；切边前后少掉的单元只经这条边可达」，后面另有一格「键不变而退出码变」必红。共四格。背景材料第三节的乙案三格是：格一 ≈ 原第一格（异常不改结局）、格二 ≈ 原第三格（少掉的单元只经这条边可达）、格三 ≈ 后面另起的那一格（键不变而结局变必红）。**原第二格「判法读包装输出前去掉这条路打的行」在乙案里没有对应的一格**，下面第四点补。
还有一个类别差：原边级排除管的是一条**子进程边，它的输出只是提示**（`kb:434` 那句里的「只打提示」）；乙案排的是一个**被判器代码直接调用、还会经闭包参数跑判器关键工作的模块**。证据（量的）：`crates/singlefs-verification/src/gpu_unit_checks.rs:1134` 起的 `pipelines_compiled_once_on_this_device` 把编管线的闭包 `compile` 交给 `self.device_condition.compile_for_the_work_of_this_thread(compile)`，后者在 `crates/singlefs-verification/src/gpu_memory_ledger.rs:2254` 起的签名是 `compile_for_the_work_of_this_thread<Output>(&self, compile: impl FnOnce() -> Output) -> Output`（`:2256` 「compile: impl FnOnce() -> Output,」）：判器要用的管线是**穿过显存记账的函数返回的**；`GpuCard` 有字段点名记账里的类型（`crates/singlefs-verification/src/gpu_unit_checks.rs:323` 「device_condition: std::sync::Arc<DeviceConditionOfACard>,」）；`take_the_readback_after_the_wait_or_record_the_device_lost`（`crates/singlefs-verification/src/gpu_unit_checks.rs:398` 起）读 `condition.is_lost()` 决定要不要记、再 panic。排它之后，判器代码调记账的调用点的返回值有没有被判定字依赖，要有一格去核，不能靠「它叫账」。这不是说它会改判定，是说它不属于原边级排除那一类「只打提示」。

### 格一：「GPU 判器对拍用例在把显存记账里登记的几处行为改坏之后仍然逐态相同，或者红在别的断言」

它证得了什么：被改坏的那几处、在这条对拍用例的输入上，GPU 与 CPU 参照逐态相同（或另有断言先红）。证不了「这个模块里的代码不改判定结局」，缺五样：

1. **执行证据**。要求写的是「仍然逐态相同」，没要求被改坏的那一处在对拍里真被执行到。改坏的位置没执行到时，「仍然相同」是空真。设备丢失后的重试、每卡只编一次管线、宽度封顶这几条路在对拍里走不走，材料里没有读数（没读到，不断言）。缺：每个改坏点各带一条「对拍这一趟跑过它」的观测（如这一处改坏之后记账的单测红，且对拍同一趟绿）。
2. **「或者红在别的断言」这一支把结论放空**。改坏的位置若既改了判定、又触发了记账断言，两件事同时发生，按字面这一格判过。要证「判定不变」，要的是「对拍里逐态比较那条断言在同一趟是绿的」，与别的断言红不红无关，两者要合取，不是析取。更要紧的是它与复用的关系：复用的是被判过的块，改坏记账之后别的断言红，并不阻止块在键不变时被复用；这支只在「对拍用例自己的键含记账」时才起兜底作用，那一条前提在三格里没有（见格三）。
3. **覆盖面**。乙案排的是整个模块 `singlefs_verification::gpu_memory_ledger`；那份文件 9074 行（量的：`grep -c '' crates/singlefs-verification/src/gpu_memory_ledger.rs`），闭包里 52 个单元（背景材料的观测）。`crates/mutations.tsv` 里含 `gpu_memory_ledger` 字样的行 171 行、含 `gpu_memory_ledger.rs` 的 166 行（量的：`grep -c 'gpu_memory_ledger' crates/mutations.tsv`、`grep -n 'gpu_memory_ledger.rs' crates/mutations.tsv | wc -l`；行里是文件列还是测试名列没逐行分）。「登记的几处」没说是哪几处、占模块里多大比例；整个模块按路径排除，往后任何新加进这个模块的函数自动免键，没有一格拦它。缺：把格一的集合定成「模块里每个在闭包里的单元至少一行被改坏的变异行」，并有一条门禁在模块多出单元时红。
4. **闭包参数直通的值**。`compile_for_the_work_of_this_thread` 返回的是调用者交进来的闭包的返回值；改坏它（调两次、返回缓存里别的摘要对应的那份、吞掉 panic）不改任何「账」，却会改拿到的管线。「行为改坏」要包括这类直通函数，而不只是改账上数字的变异。
5. **形状改动**。排除的是单元，不只是行为：记账里的类型（`DeviceConditionOfACard`、`DeviceStartedOnTheBooks` 等）是 `GpuCard` 的字段类型，它们的 `Drop`、布局、字段顺序（`crates/singlefs-verification/src/gpu_unit_checks.rs:325` 的注释写着「字段照声明的次序丢」）改了也不进键。格一的「行为」要覆盖这些。

判：**证不了**。缺的是上面五样；其中 2 是写法问题，改成合取就行，1、3、4 要补集合与观测。

### 格二：「切之后的闭包加上这个模块里原来在闭包里的单元，再从它们往下收，等于切之前的闭包」

它证得了什么：这不是恒等式，而是一个有内容的等式——`closure.rs` 的规则带全局状态：R-方法的 `methods_seen`（`closure.rs:1958` 「if !self.methods_seen.insert(method_name.clone()) {」）与 R-隐式的 `named_types`（`closure.rs:2123` 「if !self.named_types.insert(type_name.to_string()) {」）是整次收闭包共用的集合，所以「从入口 ∪ M 一起收」可以严格大于「从入口收」∪「从 M 收」（模块里点名一个类型、模块外调它的方法名，只有两边一起收才会连到那个方法体；这是读规则推的，没跑）。等式不成立说明模块外的单元在消费模块里造出的带类型的值，那是真耦合。所以这一格能抓到耦合。

它证不了：**少掉的单元里模块外的那些，对判定字无关。** 模块外只经模块可达的单元（例如只被记账函数调用的 `gpu_unit_checks` 辅助函数）属于「M 里的单元往下收」那一项，等式照成立，切掉它们却让它们出了键。乙案的格一只改坏记账里的行为，覆盖不到这批单元。缺：把「少掉的单元减去 M 自己的单元」这个集合打出来，要么为空，要么逐个登记理由并进格一的改坏集合。目前没有这个集合的读数（乙案没量过：背景材料第二节量的是甲案 1361 → 1063，乙案切之后少多少没有数）。
另一条要写死：块键是 CPU 参照判器那一组与 GPU 判器那一组并成的一把（`crates/singlefs-verification/src/judging_units_entries.rs:41` 「是这一把与 GPU 判器那一组并成的一把」）；乙案的排除登记在哪几组上，背景材料只写了「GPU 判器那一组」。格二的等式要对每一个被登记的组各做一遍。

判：**证得了「切法没有因耦合多切或少切」，证不了「少掉的单元无关」。** 要补「模块外被少掉的单元表为空或逐项覆盖」这一条。

### 格三：「键不变而判定变」必红：造一处落在被排除的模块里、又改了某个崩溃状态判定字的改动，自证要红

它证得了什么：装置对「模块里有会改判定字的改动」这种已知坏有反应（负控）。这一类自证本身是对的，没有它前面两格全作废（`.claude/rules/evidence-discipline.md` 开头「校验路径本身也要证明它会红」）。

它证不了、缺什么：

1. **能不能造出来没证**。要「落在被排除的模块里」又「改了判定字」，改动只能写在记账里。如果记账到判定字没有任何数据通路，只改记账是造不出来的，要改判器代码才行，那就不是「落在被排除的模块里」；如果造得出来，说明模块里本就有通路，那是格一该红的证据，不是格三的负控。我没有找到只改记账内部就能改判定字的位置（读了 `compile_for_the_work_of_this_thread`、`wait_for_the_device_and_take_in_the_outcome`（`gpu_memory_ledger.rs:1871`）、`take_the_readback_after_the_wait_or_record_the_device_lost`，这三处可造出的坏是丢管线、吞 panic、改记账数字，哪个会落到判定字没验证；找不到不等于没有）。格三要先给一个具体的、能造出来的坏，不然自证只是「装置能抓到我们想像出来的坏」。
2. **谁来红**。「自证要红」没点名由谁判：对拍用例单跑时，GPU 与 CPU 不同它一定红，与键无关；格三要证的是「键不变、复用会命中、而结局变了」时，**复用这一道没把坏放过去**，观察者应该是：改坏之后 ① GPU 判器那一组的键不变，② 对拍那一组的键变了（因而对拍重跑）并红。第 ② 条在格一、格二、格三里都没有出现。
3. **对拍组的键仍含记账**（上一条的前提，也是格一、格三的共同兜底）：对拍用例是单独的一组（`crates/singlefs-verification/src/judging_units_entries.rs:126` 「group_name: "gpu_and_cpu_judge_pairing_comparison",」，入口 `compare_the_judgements_state_by_state`）。乙案若只排 GPU 判器那一组，这一组仍含记账，记账一改它重跑，兜住了（推的：这一组的闭包是否真含记账没量）；若排除写成对整个模块按路径全局排、不分组，这一组也被排掉，兜底就断了。三格里需要有一格明确断言：记账改动时对拍那一组的键必变。

判：**证得了「装置对已造出来的坏有反应」，证不了乙案要的结论**，缺的是第 1（造得出来）、第 2（观察者）、第 3（对拍组键含记账）。

什么现象会推翻问题 3 的三条判：格一「证不了」——乙案把「每个改坏点对拍真执行到、同一趟逐态断言为绿、覆盖模块全部在闭包里的单元并有门禁盯着新增」写成格一的判据，并给出读数；格二「证不了少掉的单元无关」——乙案原型量出「模块外被少掉的单元」为空集（或逐项有改坏行覆盖）；格三「证不了」——给出一个只改记账内部、会改某个崩溃状态判定字的具体改动，跑出「对拍组键变且红、GPU 判器组键不变」的读数。这几样我这一轮都没有读数，所以三条判是「按写法与已读代码推的」，不是实测。

### 格四（原第二格的类比，乙案里缺的一格）

原第二格是一个**构造性的保证**：判法读包装输出之前先去掉子进程打的行，所以判定在构造上不读它。Rust 一侧的类比是静态审计：列出闭包里每一个从模块外落到记账里的调用或引用点（`unit-graph why` 对每个记账单元打「头一回进闭包的那条边」，同一个单元可能有多条边，`why` 只给头一条；要全部边要另起），逐点写明返回值与共享状态不进判定字（返回 `()`、只更新账、或经 `Output` 直通且被一格改坏集合覆盖）。没有这一格，格一的改坏集合只能是抽样；有了它，格一变成对这张清单的复核。判：乙案缺它；它做不做、做成什么样由主 agent 与用户定。

## 要主 agent 跑的命令

没有不交任务就核得了的格以外的命令要跑；下面这一条是形态十一到十五与对照的夹具读数，本腿没跑过（`membership.sh` 只过了 `bash -n`）。

1. 夹具六案，今天臂与改法甲臂各收一次闭包（同一个二进制，改法甲靠环境变量 `PROTOTYPE_WRAPPER_IMPLEMENTATIONS_WAIT_FOR_THEIR_TYPE`）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r1-sonnet -- bash /home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r1-sonnet/membership.sh <原型 unit-graph 二进制> /home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r1-sonnet/fixture-tree /home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r1-sonnet/membership-out`
   - `<原型 unit-graph 二进制>`：`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/proto-measure.sh` 在 `prototype-root` 里编出的 `<target_directory>/release/unit-graph`（主 agent 自己的编译产物，我没找到现成的路径）；要先有一次编译任务。
   - 在哪个目录起：上面 `--cwd` 的草稿目录。
   - 跑完看哪几行：任务末行 `TASK_SERVER_RESULT` 的 `exit=0`（`membership.sh` 里有一趟闭包退非 0 就退非 0，刻意如此：夹具不该有拒算）；`task-server/task log <号>` 里每个案两行 `CASE <名> arm=<today|wrapper_waits_for_its_type> rc=0 method_lines=<n>`，以及按臂缩进打出的成员行，末行 `RESULT rc_nonzero_runs=0`。
   - 预期读数（推的，写在跑之前；与预期相反的读数我怎么改结论见各形态「推翻它的现象」）：

| 案 | 今天臂（含包装类型 impl 成员行） | 甲臂 |
|---|---|---|
| `fcfield`（形态十一，字段里的 `dyn`） | `FieldFaulty` 的 `field_read` 成员在 | 不在 |
| `fcbound`（形态十一，impl 头约束） | `BoundFaulty` 的 `bound_read` 成员在 | 不在 |
| `fcrename`（形态十三，`pub use … as …`） | `RenameFaulty` 的 `rename_read` 成员在 | 不在 |
| `fcaliasimpl`（形态十四，impl 写在别名上） | 别名上那份 impl 的 `alias_read` 成员在 | 不在 |
| `fccollide`（形态十五，泛型参数与类型同名） | `&Reader` 那份的 `collide_read` 成员在 | 不在（原型漏连；文字规则不漏） |
| `fccontrol`（对照） | 四份 impl 成员都在（`ControlMem`、`ControlNamedFaulty`、`ControlNeverNamed`、`Box<Inner>`、`[Inner]` 各一行 `control_read`） | `ControlNeverNamed` 那份不在，其余仍在 |

   `fccontrol` 一格不照预期（今天臂缺行、或甲臂 `ControlNeverNamed` 仍在），说明夹具或二进制有问题，其余五格的读数不能用。

别的我想跑但跑不了或没必要的：
- 乙案没有原型，格二「少掉的单元减去模块自己的单元」的集合无从量；等乙案原型出来，用 `unit-graph closure` 两臂 `comm -23` 打出模块外被少掉的单元（命令形态同 `proto-measure.sh`）。写不出整条，写「复核不了」。
- 十二种自身类型里第 12 行（`(Wrapper<T>, u32)` 在原型里只收窄了 trait 头那一处）不另起夹具：它由 `closure.rs:895`、`:1978`、`:2045` 三处读出来；要核可往 `fccontrol` 加一个 `(ControlNeverNamed<Inner>, u32)` 的 impl 与一个带 `.control_read(` 的入口，两臂比。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 问题 1 形态 1-10（值在闭包里造出来：构造点、别名被点名、`impl Trait`、宏、关联类型、推断、扩展 trait、隐式调用、闭包内部 `dyn`、外部泛型函数） | 成立（推） | 值要存在就要有会执行的单元写它的名字，R-隐式带进 impl；没造出反例，一次抽样 |
| 问题 1 形态十一（值经接收者字段、泛型 impl 头约束进来） | 不成立 | R-开放只读入口函数签名（`closure.rs:3177`、`rust_items.rs:283`），夹具 `fcfield`、`fcbound` 待跑；GPU 判器组已读入口上没打中 |
| 问题 1 形态十二（全局量、链接期登记） | 不成立（形态存在） | 同一原因；仓里无这类依赖 |
| 问题 1 形态十三（`use … as …` 跨模块改名） | 不成立（形态存在） | `use` 单元不解名字、别名只在同作用域还原（`closure.rs:1526-1553`）；今天就有的缺口，甲把它扩到泛型包装 impl；真仓无实例 |
| 问题 1 形态十四（impl 写在类型别名上） | 不成立（形态存在） | 自身类型名不展开别名（`rust_items.rs:1009`）；真仓无实例 |
| 问题 1 形态十五（泛型参数与工作区类型同名） | 原型漏连，文字规则不漏 | 原型不去掉泛型参数；`ImplementationShape` 没存泛型参数集合（`crates/singlefs-unit-graph/src/units.rs:381`）；真仓有同名的 `Reader`，无纯 blanket 形状 |
| 问题 1 总判 | 部分成立 | 第一肢（点名）成立；第二肢（入口签名与 R-开放）写窄了；名字解析与形状各有三处缺口；与今天对全具体 impl、`dyn` 的论证同形，甲不增加缺口种类，扩大暴露面 |
| 问题 2 第 1-8 行（`T`、`&T`、`[T]`、`[T; N]`、`(T, u32)`、`Box<T>`、`Vec<T>`、`Option<Box<T>>`） | 放行，对 | 没有名字可等；放行条件甲没动 |
| 问题 2 第 9、10 行（`Wrapper<T>`、`Wrapper<'a, T>`） | 不放行，对（有条件） | 条件是形态十一到十五被收住；生命周期不入名字集 |
| 问题 2 第 11 行（`Outer<Wrapper<T>>`） | 不放行，对，偏宽 | 任一名字被点名就放（`workspace.rs:1311`），多连不漏连 |
| 问题 2 第 12 行（`(Wrapper<T>, u32)`） | 原型与文字不一致 | 原型经 `u32` 恒被点名（`closure.rs:895`）让方法名那条路照放，只收窄了 trait 头一处；多连，不漏 |
| 问题 3 格一（改坏记账、对拍仍逐态相同或红在别的断言） | 证不了 | 缺执行证据、合取、覆盖整个模块与以后新增、直通函数、形状改动五样 |
| 问题 3 格二（切之后闭包加模块单元再收等于切之前） | 证得了「没因耦合多切」，证不了「少掉的单元无关」 | 缺「模块外被少掉的单元表为空或逐项覆盖」；规则带全局状态所以等式有内容（`closure.rs:1958`、`:2123`） |
| 问题 3 格三（键不变而判定变必红） | 证得了装置对已造出的坏有反应，证不了乙案要的结论 | 缺造得出来的坏、谁来红、对拍组键含记账三样 |
| 问题 3 原第二格的类比 | 乙案缺 | 判法读包装输出前去掉那条路打的行，Rust 一侧是对每个调用点的静态审计；乙案没有 |

## 没做什么

- 不判别的腿的格，不替主 agent 采纳或出判决；没往统一任务服务交任务、判决要引的数没复跑（夹具读数、乙案模块外被少掉的单元表，命令在上一节）。
- 没读禁读清单里的文件；没跑项目代码（只用了 `grep`、`sed`、`comm` 与读文件）；夹具与 `membership.sh` 未跑。
- 没核 python 一份（`scripts/gate/lib_function_units.py:3013`、`:5195`、`:5213`、`:5222`）按改法甲落地后与 Rust 的对拍；没核宏生成类型、同名类型在两个 crate 里两份实现的认法（攻方 ④ 的面）。
- 没读真仓里别的入口组与按条复用的测试函数的入口签名；没读 `crash_judge_conclusions`、`crash_facts` 等入口的参数结构体字段。
- 没量乙案切之后少多少单元、没读 GPU 与 CPU 对拍用例的源码，所以格一里「对拍是否执行到每个改坏点」只是缺这一条观测，没有读数。
