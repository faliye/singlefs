# m3-close-unit-graph-wrapper-impls-r3 攻方腿（Opus）报告（2026-11-13）

立场：假设第二轮的三条零轮改法与前两轮判「站住」的形态都是错的，找前两轮没看过的地方。被攻的是 `/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/`（只读），今天的规则是主工作区 `crates/singlefs-unit-graph/` 与 `scripts/gate/lib_function_units.py`。禁读清单里的三处没读。前两轮攻方报告与模型目录当已攻清单读过，下面新造的形都不在里面；唯一的例外是第一条第三节：第二轮三个「没打中（量的）」的形，我判那次读数是夹具串根造出来的，原样拷来各自成根重量，标「复核」。

这一腿一个项目代码都没跑（子 agent 不交任务）。下面所有「打中」「没打中」都是读代码推的，模型的读数等主 agent 跑出来才算量的。

## 复跑

模型目录 `research/prompts/m3-close-unit-graph-wrapper-impls-r3-opus-model/`，冻结副本 `/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy`。复跑命令（主 agent 交，末节第 1 条）：

```
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r3-opus -- bash /home/fy5090/code/singlefs/research/prompts/m3-close-unit-graph-wrapper-impls-r3-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy /home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r3-opus/model-run /home/fy5090/code/singlefs/research/prompts/m3-close-unit-graph-wrapper-impls-r3-opus-model
```

它做的（`tools/rerun.sh` 的 `rerun_steps`，其余照 `scripts/agents/three-way-rerun-template.sh` 原样）：

1. 两份 unit-graph 各编进各自的编译目录：`CARGO_TARGET_DIR=$WORK/target-today` 在仓根编（今天的规则），`CARGO_TARGET_DIR=$WORK/target-verify-copy` 在 verify-copy 编（收严后的规则），拷进 `$WORK/bin/`，打 `name=binaries today_sha256=… verify_copy_sha256=…`（两个值相同就是编错了，整趟作废）。
2. `tools/judge.py`：16 个夹具每个各自成根（`$WORK/roots/<crate>/original` 与 `…/edited`，各只有 `Cargo.lock` 加 `crates/<crate>/`），四种规则（两份 unit-graph closure、两份 lib_function_units.py 经 `tools/python_closure.py`）各收一趟闭包，记被改的方法在不在闭包里、改前改后键变不变；再用 `rustc` 编夹具跑驱动记入口的值；逐行与 `out/expected.txt`（跑前写下的预期，推的）比。
3. 两份 unit-graph 各 `dump` 一次仓根（`--judgment-kind product-behavior`），比两份逐行相同；拿今天那一份交 `tools/scan_forms_the_rules_do_not_promise.py` 数第二条各类写法在 `crates/` 下的实例，全部行落 `$WORK/scan.txt`，日志里只打 `class=` 与 `name=` 行。

不是崩溃枚举，没有崩溃状态，3b 的状态数上限不适用。估时（推的）：两次 release 编译各几分钟；夹具 128 趟闭包（Rust 每趟约 1 秒、Python 每趟载入库约 2 秒）、32 次 rustc；两次 dump 各约十秒；扫描要切 `crates/` 下约 43.6 MB 的 Rust 源码（`find crates -name '*.rs' | xargs cat | wc -c` 量得 43643385），纯 Python 逐字切词估 2–5 分钟、内存估 1–2 GiB。合计估 20 分钟上下，没到 40 分钟，不缩。

模型文件 sha256（在模型目录里跑 `find . -type f | sort | xargs sha256sum` 的原样输出，40 份）：

```
1b75c6668fa6789686a98ff81c770a85399facf9101cc88916c558268b821202  ./edits.tsv
9b00500b1e190a5b39cfc8462553359812770508085ce3546fa58955244e9887  ./fixture/any_reached_through_a_type_alias/Cargo.toml.fixture
91fa08f2d56ccac8231d5a926911f30073efdfdba7a188a826e270af9c9c0e88  ./fixture/any_reached_through_a_type_alias/src/lib.rs.fixture
103397e8d31a574743b4551f051cffc0b108eb234c22215c023a6cb6c492911d  ./fixture/auto_trait_written_first_in_a_static_trait_object/Cargo.toml.fixture
ec5e892cd28528f803c17cb40a4b7346f25985555c92bc9a347f14c33c3f642c  ./fixture/auto_trait_written_first_in_a_static_trait_object/src/lib.rs.fixture
c1e8f997805f05d064c553f0c4fe82849232ba2c82f7c121297b8ff2331f85cb  ./fixture/bound_in_parentheses_on_an_opened_implementation/Cargo.toml.fixture
610890285c402a5cc20ad3cb1dcc2e2924f0d5d937e809d13c4070f1f0cd0639  ./fixture/bound_in_parentheses_on_an_opened_implementation/src/lib.rs.fixture
d3a7e25d89fee6c5ee6a9a85e1cbb57644222c360a0a7d17b4460ff7b49e0196  ./fixture/Cargo.lock.fixture
862cf083789dc9e99fa91521037d4627458b642313ec0cd5a89e48e154c242cc  ./fixture/downcast_to_an_alias_of_a_trait_object/Cargo.toml.fixture
4ca33dace714ef2187b8bde29ec7aab3d3832978f757a6c3805d56d4e63a29d3  ./fixture/downcast_to_an_alias_of_a_trait_object/src/lib.rs.fixture
0a0cedc15b14ce80f11782eec9940c508b57ee0c38e8ecec1570a8896b804efe  ./fixture/higher_ranked_bound_on_an_opened_implementation/Cargo.toml.fixture
d3750e30dd78bd2c1534fa1d004efdc3c43d3327918aa24b75b99d554cb4651f  ./fixture/higher_ranked_bound_on_an_opened_implementation/src/lib.rs.fixture
ee4833d587d36b5d73fcea17821bbee7ba2ed1e15bb751469c260e7e388d5bae  ./fixture/higher_ranked_supertrait_of_the_entry_trait/Cargo.toml.fixture
b48692187ab6797bb62d94fdf5f8c73645abdf75549d9e1014c84d404e247ecf  ./fixture/higher_ranked_supertrait_of_the_entry_trait/src/lib.rs.fixture
d0c317d71d42dd75e16cd3a9e57b3cf88fd250f0d802f58d9d6e1d9e3fe83d46  ./fixture/parenthesized_bound_on_the_entry_signature/Cargo.toml.fixture
0e7c54496ffe354a9d32a73b2f7f21a50b0d42794e6425b167f316dbf589ed77  ./fixture/parenthesized_bound_on_the_entry_signature/src/lib.rs.fixture
d5ec27af883e4abb31a837bb082e94e5849ae76ba149448971aa3433c651a0b3  ./fixture/raw_identifier_any_in_the_entry/Cargo.toml.fixture
a0546288f980e06282efe6b0c26191d335660c889c431ed80ebba1f88426c57f  ./fixture/raw_identifier_any_in_the_entry/src/lib.rs.fixture
5410874c6cb7d99054219e3d38d3e8c9ae492f2ac60e3967c5d2dfb8699649aa  ./fixture/raw_identifier_trait_in_the_entry_signature/Cargo.toml.fixture
978fc48969525301c5d687a813bc583374b27af48461b6735fe7299e6b779d6b  ./fixture/raw_identifier_trait_in_the_entry_signature/src/lib.rs.fixture
cfefbd33f07ba44ab19701f206e6db7c088ebdc331d6744c0ab9b802562e470a  ./fixture/struct_in_an_inactive_cfg_arm_alias_made_by_an_item_macro/Cargo.toml.fixture
af8ff7399484beaebddd7ac053a3683ef1a83e02ca050d039cf91dda6c3d8cb2  ./fixture/struct_in_an_inactive_cfg_arm_alias_made_by_an_item_macro/src/lib.rs.fixture
22d359dd17e56e818e21b317d3e07c8d57fe2599aefea846eaf5d2855ed6848b  ./fixture/struct_in_an_inactive_cfg_arm_glob_import_in_the_active_arm/Cargo.toml.fixture
f60bdb2dceff46e96002a3f69d983278c3780032f3f38a9237de7a17a9c5e1db  ./fixture/struct_in_an_inactive_cfg_arm_glob_import_in_the_active_arm/src/lib.rs.fixture
0ca19bb13341a4ea13fb315a9edce3e3025877b98a9fdca7d94d008a0b371c7d  ./fixture/trait_object_alias_held_by_a_static/Cargo.toml.fixture
114dd797d2d7353652c8d49e3f8af38da87e02fbdc9c29d0a0ff47b6a688337b  ./fixture/trait_object_alias_held_by_a_static/src/lib.rs.fixture
3b3fc00e650516fdbaf84270c2f4e75983052960854db78c46ef707d7a2522f3  ./fixture/trait_object_field_of_an_opened_implementation/Cargo.toml.fixture
6d21b0c823aebd4d329d0d250ff7af3efbf2184dbb124dbaa0a6a0bf689b625b  ./fixture/trait_object_field_of_an_opened_implementation/src/lib.rs.fixture
5f0b8af0ff0406bcba4c1d866d91cf39630ee9bd7758b093e20e3f34d18378de  ./fixture/trait_object_held_by_a_static_inside_a_function_body/Cargo.toml.fixture
98a1716bbe1d48e026fb6284dd94b0e839dcf000436fcc0afd8a26af44f9e875  ./fixture/trait_object_held_by_a_static_inside_a_function_body/src/lib.rs.fixture
f642099cfa6022ec113ca9ae874e8245d4a6771f16ca5b0404f4ae021340e9fa  ./fixture/variant_imported_by_a_use_made_by_an_item_macro/Cargo.toml.fixture
5d554c7f46089d572fb248241f260eafd478ce1cc6d46ac4c2eae454b4bd03ac  ./fixture/variant_imported_by_a_use_made_by_an_item_macro/src/lib.rs.fixture
5732cea26f1beeb4d395f15675f250fbb39d37dfb61e3826c9ac06351eadb37a  ./fixture/wrapper_named_in_the_entry_control/Cargo.toml.fixture
a624d72a6e394ecd5549375438b8a358c485293c1f4e4b131d2e946e3d208c95  ./fixture/wrapper_named_in_the_entry_control/src/lib.rs.fixture
cdb0d970a0f0a566ac423117e2437cb6f446f009a69c267ccb5d8310b4c720b9  ./out/expected.txt
201cf7daedc6df8ba5cc06e02c6e9ad1da70f4f06a2e07186f170de7754d40ba  ./tools/judge.py
4b0d3a39d24ba8e0f38174409579b805f306004df8f717f82a063ce33ecd27f7  ./tools/python_closure.py
e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b  ./tools/rerun-from-a-clean-work-directory.sh
0472435ad30bd44f141a4401a6bfb677965879bfb91a2841113ed363ab07150f  ./tools/rerun.sh
c9da11245444ad1f1e10f99bc421ad062e7ee743ab4552c4612206b36040281d  ./tools/scan_forms_the_rules_do_not_promise.py
```

夹具文件带 `.fixture` 后缀（照前两轮的做法），`judge.py` 铺进运行目录时去掉后缀。`tools/rerun-from-a-clean-work-directory.sh` 与第二轮那一份逐字相同（sha256 同为 e3294c15…）。

## 各格判定一览

| 关的那一条 | 格 | 判定 | 依据一句话 | 量的 / 推的 |
|---|---|---|---|---|
| 1 零轮改法的边角 | 第三条「同一段模块里没有 `use` 引进同名」对 glob | **打中（夹具）** `struct_in_an_inactive_cfg_arm_glob_import_in_the_active_arm` | 只比 use 引进的名字，`pub use real::*;` 的名字表是空的 | 推的，待跑 |
| 1 | 第三条「个个是 struct、enum 或 union」对条目级宏造的同名别名 | **打中（夹具）** `struct_in_an_inactive_cfg_arm_alias_made_by_an_item_macro` | 宏造的别名不是类型单元，同名候选只剩那个 struct | 推的，待跑 |
| 1 | 第五条「没有 `use` 的路径从它身上穿过去」对写在 `macro_rules!` 里的 use | **打中（夹具）** `variant_imported_by_a_use_made_by_an_item_macro` | 「穿过的名字」只从 use 单元收 | 推的，待跑 |
| 1 | 第三条对写在外层模块或函数体里的 use、对 `extern crate`、`pub(crate) use` 重导出 | 没打中 | 外层模块的 use 改不了内层的名字；函数体里的 use 是那个函数单元自己的词，写出的名字照点名；改名转出走第四条（全工作区一张表） | 推的 |
| 1 | 「入口在 trait 头下时开这个 trait」对父 trait | **打中（夹具）** `higher_ranked_supertrait_of_the_entry_trait` | 父 trait 写成 `for<'a> Store<'a>`，开 trait 时取约束只认紧跟 `:` 的那一个词 | 推的，待跑 |
| 1 | 同上，对关联函数（没有接收者） | 没打中 | 开入口所在 trait 时把它的 impl 头连约束一起开，约束照常写时 `Store` 照开 | 推的 |
| 1 | static 与 `Any` 那一条：static 的类型写成别名 | **打中（夹具）** `trait_object_alias_held_by_a_static` | 只开 static 单元自己写的 `dyn` | 推的，待跑 |
| 1 | 同上：`dyn Send + Sync + Store` | **打中（夹具）** `auto_trait_written_first_in_a_static_trait_object` | `dyn_trait_names` 只记 `dyn` 后面头一个 trait | 推的，待跑 |
| 1 | 同上：函数体里的 `static` | **打中（夹具）** `trait_object_held_by_a_static_inside_a_function_body` | 函数体里的条目不切单元，函数单元不是 static | 推的，待跑 |
| 1 | 同上：向下转型的目标写成别名 | **打中（夹具）** `downcast_to_an_alias_of_a_trait_object` | `dyn Store` 在别名里，别名不提 `Any` | 推的，待跑 |
| 1 | 同上：`Any` 经别名到入口 | **打中（夹具）** `any_reached_through_a_type_alias` | 入口自己的词里没有 `Any` | 推的，待跑 |
| 1 | 入口签名里的约束写成 `(Store)` | **打中（夹具）** `parenthesized_bound_on_the_entry_signature` | 签名的 R-开放 只认紧跟的那一个词（今天的规则同样不开，今天靠 blanket 兜住） | 推的，待跑 |
| 1 | `const` 里的函数指针表、`Box<dyn Fn>` 回调捕获的包装值、`&dyn Debug`/`Display` 后面的包装类型、`impl Trait` 返回值经另一个 crate 转手 | 没打中（不分辨两套规则，或都收） | 见第一条「没打中的形」 | 推的 |
| 1 复核 | 第二轮判「没打中（量的）」的三形：开进来的 impl 字段里的 `dyn`、`(Store)`、`for<'a>` | **那次读数不作数**：十一个夹具铺在同一个根里，`receiver_of_a_provided_trait_method_entry` 的 `impl<Inner: Store> Source for Wrapper<Inner>` 头上的 `Store` 被开，把同根每个夹具的 `Wrapper` impl 带进来；三形各自成根重量，预期都打中 | 推的，待跑 |
| 2 未定项清单 | 规则不保证收到的写法 | **列了 16 类**（第二条表），每类一个扫描类名；第 1 类（入口外调用方给的回调）预期今天真仓有实例 | 扫描脚本读 `unit-graph dump` 的切分 | 推的，待跑 |
| 3 改法乙 | 52 个登记单元的返回值流到哪 | **没打中**：两个的返回值进判定路径（编好的管线、派活切法与草稿区），都改不了判定字；其余交 `()`、`bool` 或打印用的串 | 逐个追到文件:行，见第三条 | 推的（读源码） |
| 3 | `mod gpu_memory_ledger;` 与 use 不进键之后漏掉的 cfg 或特性依赖 | **没打中** | 那一行没有属性；四条 use 都是 std；登记的单元里没有 cfg 条件与 `cfg!`，不点外部 crate | 推的（读源码） |
| 4 两份实现 | 这一轮新加的代码 | **打中（夹具两个）** `raw_identifier_any_in_the_entry`、`raw_identifier_trait_in_the_entry_signature` | Python 判「提到 `Any`」与签名 trait 名都拿原文比、带着 `r#`；Rust 两处都去前缀 | 推的，待跑 |

## 第一条：第二轮三条零轮改法的边角（①）

### 核过的规则（verify-copy 的代码，全部现读）

- 第三条的两句：同一段模块里的类型单元按文件与行内模块路径筛「unit.file == head_unit.file && unit.local_module_path == head_unit.local_module_path」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2143`），候选取自 `type_units_by_name`，那张表只收类型单元（`unit.kind.is_type()`）；use 那一句只比 use 单元交的名字表「.any(|imports| imports.names.iter().any(|name| name == first_name))」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2166`）。glob 的 use 在名字表里一个名字都没有（`use_imports` 见 `*` 只记 `is_glob`）。
- 第四、第五条的两张表只从 use 单元收：「if unit.kind != UnitKind::Use {」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/workspace.rs:1262`）之后才记改名与「路径穿过的名字」。写在 `macro_rules!` 展开式里的 use 是宏定义单元的词，不是 use 单元；`macro_rules!` 展开式顶层登名字只认这几个关键字「["const", "static", "fn", "struct", "enum", "union", "type"];」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/workspace.rs:261`），没有 `use`。
- 入口在 trait 头下：「if workspace.units[container.0].kind == UnitKind::TraitHead {」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2206`）之后开这个 trait；开 trait 时把它每个 impl 头上的约束也开，取约束的写法是紧跟 `impl`、`dyn`、`:`、`+` 的那一个词「if !matches!(token_text(&file.source, token), "impl" | "dyn" | ":" | "+") {」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2181`）。`for<'a> Store<'a>` 里紧跟 `:` 的是 `for`（按标识符收进去，`open_trait("for")` 找不到 impl）。
- static 与 `Any` 那一条：「if matches!(unit.kind, UnitKind::Static | UnitKind::ItemMacroInvocation)」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:1415`），开的是这个单元的 `dyn_trait_names`；`dyn_trait_names` 在 `dyn` 后面只走一条路径、记它的末段「last_segment = Some(identifier_name(text_at(path_position)));」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/rust_items.rs:265`），`+` 后面的不记。

### 打中的形（每个都是模型里各自成根的夹具；预期今天两份收、verify-copy 两份不收、改那一行入口值 6 变 9，全部推的，待主 agent 跑）

| 夹具 | 改法 | 形 | 许可它的那一句（verify-copy） |
|---|---|---|---|
| `struct_in_an_inactive_cfg_arm_glob_import_in_the_active_arm` | 第三条后一句 | 根模块 `#[cfg(not(unix))] pub struct Wrapper<Inner>(pub Inner);` 与 `#[cfg(unix)] pub use real::*;`，`real` 里有 `pub type Wrapper<Inner> = Real<Inner>;`；`impl<Inner: Store> Store for Wrapper<Inner>`；入口 `real::Real(Base).put(3)` | closure.rs:2166 那一句只比名字表；别名在 `real` 模块，被 closure.rs:2143 筛掉；同模块候选只剩不编进来的 struct |
| `struct_in_an_inactive_cfg_arm_alias_made_by_an_item_macro` | 第三条前一句 | 同上，`#[cfg(unix)]` 那一臂换成条目级宏调用 `alias_of_a_type_in_the_real_module!(Wrapper, Real);`（展开式 `pub type $alias<Inner> = real::$target<Inner>;`）；值由 `real::make(Base)` 造 | 宏造的别名不是类型单元，`type_units_by_name` 里叫 `Wrapper` 的只有那个 struct；入口不写宏调用里出现的名字，宏调用不进闭包 |
| `variant_imported_by_a_use_made_by_an_item_macro` | 第五条 | `pub enum Layered<Inner> { Plain(Inner), Doubled(Inner) }`、`impl<Inner: Store> Store for Layered<Inner>`；`macro_rules! bring_in_the_doubling_variant { () => { use crate::Layered::Doubled; }; }` 在根模块调一次；入口 `Doubled(Base).put(3)` | workspace.rs:1262 只从 use 单元记「路径穿过的名字」；宏展开里的 use 不是 use 单元，`Layered` 不进那张表，这个 impl 等 `Layered` 被点名 |
| `higher_ranked_supertrait_of_the_entry_trait` | 入口所在 trait | `pub trait Source: for<'a> Store<'a> { fn total(&self) -> u32 { self.put(&3) } }`、`impl<Thing> Source for Thing where Thing: for<'a> Store<'a> {}`；入口 `Source::total`，`caller` 是 `Wrapper(Base).total()` | closure.rs:2206 开 `Source`，它唯一的 impl 头与 trait 头的父 trait 段在 closure.rs:2181 那一句下都只交 `for` |
| `trait_object_alias_held_by_a_static` | static 与 `Any` | `pub type Installed = Box<dyn Store + Send + Sync>;`、`pub static INSTALLED: OnceLock<Installed>`；入口从 `INSTALLED.get()` 取盒子调 `put(3)`，`caller` 先装 `Wrapper(Base)` | closure.rs:1415 只开 static 单元自己的 `dyn_trait_names`（空的）；别名是类型单元，它写的 `dyn Store` 只在弄坏开关下开 |
| `auto_trait_written_first_in_a_static_trait_object` | static 与 `Any` | `pub static INSTALLED: OnceLock<Box<dyn Send + Sync + Store>>`，其余同上 | rust_items.rs:265 只记 `Send` |
| `trait_object_held_by_a_static_inside_a_function_body` | static 与 `Any` | `fn installed() -> &'static Mutex<Option<Box<dyn Store + Send>>> { static INSTALLED: … = Mutex::new(None); &INSTALLED }`；入口经 `installed()` 取 | 函数体里的条目不切单元，`installed` 是函数单元，既不是 static 也不提 `Any` |
| `downcast_to_an_alias_of_a_trait_object` | static 与 `Any` | `pub type Boxed = Box<dyn Store>;`，入口 `entry(thing: &dyn Any)` 里 `thing.downcast_ref::<Boxed>()` 再 `put(3)` | 入口提 `Any`，它自己的 `dyn` 只有 `Any`；`dyn Store` 在别名里，别名不提 `Any` |
| `any_reached_through_a_type_alias` | static 与 `Any` | `pub type Erased = dyn std::any::Any;`，入口 `entry(thing: &Erased)` 里 `downcast_ref::<Box<dyn Store>>()` | 入口的词里没有 `Any`；别名提 `Any`，但它的 `dyn` 只有 `Any` |
| `parenthesized_bound_on_the_entry_signature` | 入口签名的 R-开放（今天的规则） | `pub fn entry<Thing: (Store)>(thing: &Thing) -> u32 { thing.put(3) }` | 签名 trait 名紧跟 `:` 才收，`:` 后面是 `(`；第二轮只在开进来的 impl 头上试过这种写法 |

这十个形都只用标准库、两份规则各一份写法；对照夹具 `wrapper_named_in_the_entry_control`（入口写 `Wrapper(Base).put(3)`）预期四种规则都收、键都变。

### 复核：第二轮那三个「没打中（量的）」是夹具串根造出来的（推的，待跑）

第二轮判决把开进来的 impl 字段里的 `dyn`、`(Store)`、`for<'a>` 三形记成「没打中（量的）」，依据是任务 15869 那三组行，例「name=bound_in_parentheses_on_an_opened_implementation rule=verify_copy impl_in_closure=1 key_changes_with_the_edit=yes」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15869.attempt-1.log:83`）。那一趟的判法把十一个夹具铺进同一个根：「for directory, _, names in os.walk(source_root):」（`research/prompts/m3-close-unit-graph-wrapper-impls-r2-opus-model/tools/judge.py:68`），`source_root` 是整个 `fixture/`。串的路子：

1. 三形的入口都收 `&dyn Source`，入口签名开 `Source`。
2. `open_trait` 按 trait 名查的索引是全工作区一张表「.trait_implementations_by_trait_name」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/workspace.rs:1381`，建表时不看目标），开到的每个 impl 头，不论在不在链接集里，都照「for bound_trait_name in self.names_written_where_a_trait_goes(*head) {」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2311`）再开它头上的约束；不在链接集里的头只是不进闭包（`push_with_a_linked_target` 那一层筛）。
3. 同根的 `receiver_of_a_provided_trait_method_entry` 里有「impl<Inner: Store> Source for Wrapper<Inner> {」（`research/prompts/m3-close-unit-graph-wrapper-impls-r2-opus-model/fixture/crates/receiver_of_a_provided_trait_method_entry/src/lib.rs.fixture:24`）：它被开到，`Store` 跟着开，按名字查到每个夹具的 `impl<Inner: Store> Store for Wrapper<Inner>`，链接集里那一份进闭包。

所以那三组 `impl_in_closure=1` 不说明收严二的取名法收得到 `(Store)`、`for<'a>`、开进来的 impl 的字段。这一腿把三个夹具原样拷来（sha256 与第二轮那几份相同，见上），各自成根重量，预期三形都是 verify-copy 两份不收、今天两份收（`out/expected.txt`）。推翻它的现象：各自成根之后三形仍是 `rule=verify_copy impl_in_closure=1`。上面第 2 步的机理同时意味着真仓里一个 crate 的 impl 头上的约束会开别的不相干 crate 里的 impl（只多连，不是过窄）。

### 打中之后的四句（照 `.claude/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种`）

1. **分不分辨臂**：分辨。每一形上今天的规则收（带泛型参数的 impl 一律按 blanket 放行，R-方法 落到 `put` 就带它）、verify-copy 不收，对照夹具两边都收。其中 static 别名、自动 trait 在前、函数体里的 static、向下转型的两形、`(Store)` 与高阶约束三处，换成全具体的 impl（`struct Doubled(Base)` 加 `impl Store for Doubled`）今天也漏，病根是两边共用的「被点名 = 闭包里有单元写出类型名」，收严把它的射程扩到包装类型；第三条与第五条那三形（glob、宏造别名、宏里的 use）是收严自己新写的条件的边角，不是共用前提。
2. **被判的系统看不看得到判别它的东西**：看得到。单元图手里有：同一段模块里的 glob use 与条目级宏调用单元；`macro_rules!` 单元的词（里面的 `use 路径::`）；约束区里 `(`、`for<…>` 后面的词；static 单元点名的别名单元的词；函数单元词里的 `static` 关键字；单元里的 `downcast_ref` 方法名。verify-copy 没用这些。
3. **满足的是判据字面的哪一个分句**：`research/prompts/_m3-close-unit-graph-wrapper-impls-r3-background.md@四、判据（跑前写死）` 第一条「攻方给出新反例（今天的规则收、副本的规则不收、改它入口结果变），主 agent 复跑坐实的：那一形写进判决的交用户表、标零轮，由用户在「补这一形后合」「撤回改法甲」之间定；不再为它开轮。」整句，十形都按它造，入口值 6 变 9 是推的。入口执行的是闭包外造的值的方法，「改它入口结果变」照字面成立；这一类算不算这一题要管的「入口行为」，前两轮判决都按打中办了，这里照办。真仓有没有实例由第二条的扫描回答，落到同一节第二条「真仓 `crates/` 里今天就有实例的漏连（不论新旧）：合进主工作区之前必须收到，收不到这套规则不合。」。
4. **跑前条款给的改法在打中的格上还中不中**：判据只给两条出路「补这一形后合」「撤回改法甲」。撤回在十形与复核三形上都不中（回到今天的 blanket）。补形逐格见下表；没有一种同时修全部。

| 补形的写法（只在我的推演上过、被攻过零轮；每格都是「推的」，没实现没跑） | glob | 宏造别名 | 宏里的 use | 高阶父 trait | static 别名 | 自动 trait 在前 | 函数体 static | 转型目标别名 | `Any` 经别名 | 签名 `(Store)` | 复核三形 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 甲一：第三条再加「同一段模块里有 glob use 或条目级宏调用（含 `include!`）就不等」 | 修 | 修 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 |
| 甲二：第四、第五条的两张表另从 `macro_rules!` 单元的词里收（`use` 到 `;` 之间的「穿过的名字」与 `as` 的原名） | 不修 | 不修 | 修 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 |
| 甲三：取约束名时 `:`、`+` 后面跳过 `(`、`?`、`for<…>` 再取路径末段（`names_written_where_a_trait_goes` 与签名的 `signature_trait_names` 两处，两份实现同改） | 不修 | 不修 | 不修 | 修 | 不修 | 修（static 那一处改用它时） | 不修 | 不修 | 不修 | 修 | 修 `(Store)`、`for<'a>` 两形 |
| 甲四：static 与 `Any` 那一条改开 `names_written_where_a_trait_goes(单元)`，并沿这个单元点名的类型单元（别名）照入口那样往下找 | 不修 | 不修 | 不修 | 不修 | 修 | 修 | 不修 | 修 | 不修 | 不修 | 不修 |
| 甲五：函数单元的词里有 `static` 条目或 `thread_local!` 的，照 static 单元开 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 修 | 不修 | 不修 | 不修 | 不修 |
| 甲六：「提到 `Any`」改成「提到 `Any`，或点名的别名提 `Any`，或调了 `downcast`、`downcast_ref`、`downcast_mut`」 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 修 | 不修 | 不修 |
| 甲七：被 R-开放 开进来的 impl，自身类型的名字也进 `type_names_to_look_into` 沿字段找（第二轮攻方表里那一行） | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 修字段一形 |
| 撤回改法甲 | 修 | 修 | 修 | 修 | 修 | 修 | 修 | 修 | 修 | 修 | 修 |

甲一到甲七合起来才盖住这十三格，而每一条都是「再开一处」：第二轮与这一轮各打中一批，补的都是同一个前提（值不经点名就到得了闭包）的又一个口子。我推不出这一串会在哪一条收住；第二条的清单里还有今天就漏、这几条都不修的类（入口外调用方给的回调、静态里的回调、过程宏）。甲一会让同一段模块里有 `use super::*;` 的 impl 全照 blanket 放行，真仓上会放大多少要量（没量）。

### 试过、没打中的形（推的，没有夹具）

| 改法 | 形 | 为什么没打中 |
|---|---|---|
| 第三条 | `use` 写在外层模块（`mod outer { use crate::other::Wrapper; mod inner { struct Wrapper… impl … for Wrapper<T> } }`） | 2018 版以后外层模块的 use 不进内层的作用域，内层的 `Wrapper` 只能是内层自己的条目、内层的 use 或内层的 glob（glob 那一形已打中） |
| 第三条 | `use` 写在函数体里（`fn entry() { use crate::Wrapper as W; W(Base).put(3) }`、`use crate::Layered::Doubled;`） | 函数体里的 use 是函数单元自己的词：`Wrapper`、`Layered` 照 R-裸名 解（单元里另有 use 时按模块筛退回全连），类型被点名 |
| 第三条 | `extern crate x as Wrapper;` 放在另一臂 | 编不过：`Wrapper<T>` 指一个 crate，不是类型 |
| 第三、四条 | `pub(crate) use inner::Wrapper as W;`、`pub(crate) use self::Layered::*;`、`pub(crate) use inner::*;` 重导出 | 前两种是 use 单元，进全工作区的改名表与「穿过的名字」表；第三种不改名，别处用它还写 `Wrapper` |
| 入口所在 trait | 关联函数入口（`trait Make { fn total() -> u32 where Self: Sized { Self::build().put(3) } fn build() -> Self; }`），实现写成 `impl<T: Store + Default> Make for T` 或 `impl<I: Store + Default> Make for Wrapper<I>` | 开 `Make` 时它的 impl 头进闭包：前一种头上的 `: Store` 照开，后一种头点名 `Wrapper` 照 R-隐式；只有约束写成 `(…)`、`for<…>` 才漏（归高阶父 trait 那一格同一个机理） |
| 入口所在 trait | 父 trait 照常写（`trait Source: Store`）或写在 where 里（`where Self: Store`） | trait 头的词里 `:` 后面紧跟 `Store`，照开 |
| static 与 `Any` | `const` 里的函数指针表（`const TABLE: [fn(u32) -> u32; 1] = [make_wrapped];`） | 表里点名的函数照 R-裸名 进闭包，它写了 `Wrapper` |
| static 与 `Any` | `const HELD: &dyn Store = &Wrapper(Base);` | const 单元自己写出 `Wrapper` |
| static 与 `Any` | `thread_local!` 之外的条目级宏（自写 `macro_rules! global { … static … }` 再调） | 条目级宏调用单元在那三类位置里，调用里写的 `dyn Store` 照开 |
| static 与 `Any` | `Box<dyn Fn() -> u32>` 回调捕获包装值、交给入口调 | 回调体在调用方，两套规则都不收（第二条 A1），不分辨 |
| static 与 `Any` | 入口收 `&dyn Debug` / `&dyn std::fmt::Display`，后面是 `Wrapper<Base>` | 签名开 `Debug`/`Display`（`dyn std::fmt::Display` 里第二个 `:` 后紧跟 `fmt`，再一个后紧跟 `Display`）；格式化走 `format!`，两套规则都不把 `fmt` 当方法调用收，不分辨 |
| static 与 `Any` | 另一个 crate 的 `pub fn wrap<S: Store>(s: S) -> impl Store { Doubler(s) }` 转手 | 那个函数在闭包里，它写了 `Doubler` |
| static 与 `Any` | `static PTR: AtomicPtr<()>` 存擦掉类型的指针、入口 `unsafe` 转回 `*const Box<dyn Store>` | 机理同函数体 static：入口单元写 `dyn Store` 而既不是 static 也不提 `Any`，两份都不开，会打中；没造夹具（要 `unsafe`，真仓的判器一侧没有，第二条 B6 那一类列了它，没扫） |

## 第二条：未定项清单的完整性（②）

### 规则不保证收到的写法，列全（今天就漏的连同收严后新漏的）

分类按「值或代码怎样不经点名到达入口执行的地方」切：A 是代码本身在闭包外（两套规则都不收）；B 是类型名不出现（全具体的 impl 今天就漏，带泛型参数的今天靠 blanket 兜住、收严后漏）；C 是收严新写的条件的边角（只收严后漏）；D 是两份实现读法不同。每类「扫描类名」是 `tools/scan_forms_the_rules_do_not_promise.py` 输出里的 `class=` 名。预期读数全是推的（我只用 grep 粗看过几类，不当量的）。

| 类 | 写法 | 今天 | 收严后 | 扫描类名 | 真仓实例数的预期读法 |
|---|---|---|---|---|---|
| A1 | 登记入口的签名，或沿签名点名的类型（按名字、连字段一路下去）里有 `dyn Fn…`、`impl Fn…`、`: Fn…`、`fn(…)`：回调体由入口外的调用方写 | 漏 | 漏 | `code-supplied-by-the-caller-of-a-registered-entry`（计数 `in_an_entry_signature`、`in_a_type_reached_from_an_entry`、`entries_not_found`） | 预期大于 0（第一轮攻方读到 `crash_judge_tables.rs` 的 `expansion: &'flow dyn Fn(…)` 字段）。非 0 不自动等于漏连：每一处要主 agent 判回调交回的是不是这份判定自己的内容输入（已定项 17 那一句「加这份判定自己的内容输入（段写表与起步镜像摘要、litmus 内容、变异原文与替换文这类）」），不是的就是今天就有的漏连。`entries_not_found` 大于 0 说明脚本认入口的写法没对上，那几个入口没扫 |
| A2 | static、`thread_local!`、条目级宏里放回调或函数指针 | 漏 | 漏 | `code-held-by-a-static` | 不知道（没粗看）；非 0 的每一处看它放的回调是谁写的、在不在登记组的闭包里 |
| A3 | 工作区里的过程宏（`#[derive(...)]` 生成的 impl 不是单元） | 漏 | 漏 | `procedural-macro-crate` | 预期 0（`crates/*/Cargo.toml` 粗看没有 `proc-macro = true`） |
| B1 | 引进变体造值：`use E::V;`、`use E::*;`，`E` 是全具体的 enum、有 trait impl | 漏 | 漏（带泛型参数的 enum 由第五条接住） | `variant-brought-in-by-a-use`（计数 `enum_with_a_concrete_trait_implementation`） | 不知道；计数非 0 的那几处，要用 `unit-graph why` 看那几个 impl 在不在点到它的登记组闭包里 |
| B2 | impl 写在别名上（`impl Tr for Alias`，`Alias = Struct`） | 漏 | 漏（带泛型的由第三条接住） | `implementation-written-on-a-type-alias` | 预期 0（第一轮攻方的近似扫描没见过） |
| B3 | `use X as Y` 改名、别的模块按 `Y` 用 | 漏（第一轮那一形） | 漏（带泛型的由第四条接住） | `type-renamed-by-a-use`（计数 `type_with_a_trait_implementation`） | 预期总数个位数（第一轮攻方数过 5 处 `use … as 大写名`），计数 0 |
| B4 | 被 R-开放 开进来的 impl，自身类型的字段里放工作区 trait 的 `dyn` | 漏 | 漏 | `trait-object-inside-the-self-type-of-a-trait-implementation` | 不知道；扫描只要求「类型有 trait impl、字段里是工作区 trait 的 `dyn`」，是上界，每一处要看那个 impl 的 trait 有没有被某个登记组的签名开 |
| B5 | 约束写成 `(Trait)`、`for<'a> Trait<'a>`、`impl (Trait)`、`dyn (Trait)`：入口签名、开进来的 impl 头、父 trait、字段类型的约束 | 漏（签名那一处；带泛型参数的靠 blanket 兜住） | 漏 | `bound-after-a-parenthesis-or-for`（计数 `naming_a_workspace_trait`） | 预期 `naming_a_workspace_trait=0`（粗看没有 `: for<`、`: (`）；标准库 `Fn` 上的高阶约束可能有几处，不相干 |
| B6 | 函数体里的 `static` 或 `thread_local!` 放 trait 对象；`AtomicPtr<()>` 一类擦掉类型的指针 | 漏 | 漏 | `static-inside-a-function-body-holding-a-trait-object`（指针那一种没扫） | 预期 0（粗看没有） |
| C1 | cfg 两臂：一臂本模块的 struct/enum/union，另一臂是 glob use 或条目级宏调用（含 `include!`）造的同名 | 收（blanket） | 漏 | `cfg-arm-name-supplied-by-a-glob-or-an-item-macro`（计数 `with_a_waiting_implementation`） | 预期 0 |
| C2 | use 写在 `macro_rules!` 展开式里，路径穿过类型名或改名 | 收 | 漏 | `use-written-inside-a-macro-expansion`（计数 `passing_through_or_renaming_a_type`） | 预期计数 0 |
| C3 | static 与 `Any` 那一条认的位置之外：static 的类型写成别名 | 收 | 漏 | `static-typed-through-an-alias-holding-a-trait-object` | 预期 0 |
| C4 | 同上：`dyn` 后面先写自动 trait（`dyn Send + Store`） | 收 | 漏（只在 static、条目级宏、提 `Any` 的单元里） | `trait-object-with-an-auto-trait-first`（计数 `in_a_static_or_any_unit`） | 预期计数 0 |
| C5 | 同上：向下转型的目标是别名、或转型的单元不提 `Any`；`Any` 经别名到入口 | 收 | 漏 | `downcast-to-a-trait-object-outside-the-any-rule`（计数 `downcasts_seen`、`unit_does_not_name_any`、`target_is_an_alias`）、`alias-of-any` | 预期 `downcasts_seen` 几十（粗看 57 处，目标是 `String`、`&str`、`StopInsideASlice`），后两个计数 0，`alias-of-any` 0 |
| C6 | 入口所在 trait 的父 trait 写成高阶或带括号 | 收 | 漏 | 并在 B5 里（`trait` 单元的父 trait 段） | 同 B5 |
| D1 | 原始标识符：`r#Any`、入口签名里 `dyn r#Trait` / `: r#Trait` | 两份都收 | Rust 收、Python 不收 | `raw-identifier-read-differently-by-the-two-implementations`（计数 `raw_identifiers_seen`、`raw_any`、`raw_signature_trait`） | 预期 `raw_any=0`、`raw_signature_trait=0`；`raw_identifiers_seen` 是代码里的 `r#` 总数（粗看 `r#mod`、`r#type` 一类在测试的串里，串里的不算） |

清单之外，我知道而判为「收得到」的：`macro_rules!` 体里写类型名（体按调用处解、全连）、条目级宏生成的 impl 与类型（宏调用登它生成的类型名）、别名被点名（别名单元的词里有原名）、关联类型 `type Out = Wrapper<T>`（impl 连成员进闭包）、`Default`/`From`/推断（目标类型总要在闭包里某处写出）。这几条是前两轮攻方读出来的，我没重核。

### 扫描怎么做、读数怎么读

- 切分用单元图自己的：`unit-graph dump <仓根> --judgment-kind product-behavior`（今天的那一份；收严后的那一份也 dump 一次，`name=dump same=yes` 才说明两份切法相同），每行的种类、限定名、`文件:起-止` 原样取，只看 `crates/` 下的 `.rs`、不看 doctest 与数据单元。每个单元的词从它那几行的源码现切（去注释，串、字符字面量各算一个词），所以「串里写着 `dyn Fn`」不算。模块按限定名里 `<impl`、`<trait` 之前的那几段认，等价于单元图的「同一份文件、同一段行内模块」。
- 输出：每类一行 `class=<类名> instances=<处数> <计数>=<n>…`，接着每一处 `instance class=<类名> at=<文件>:<行> unit=<种类> <限定名> detail=<…>`（全部在运行目录的 `scan.txt`；日志里只有 `class=` 行与末行）；末行 `name=scan rust_units=<n> files=<n> classes=16 registered_entries=<n> entries_found=<n>`。`registered_entries` 是从 `crates/singlefs-verification/src/judging_units_entries.rs` 的 `JUDGING_UNITS_ENTRY_GROUPS` 里读到的入口路径个数（观察者那一张表没读，见限度），`entries_found` 是在 dump 里认到的个数。
- 判据（跑前写下）：C1–C5、D1 各自的计数（不是总处数）非 0 的那一处，是收严之后真仓就有的漏连或不等价候选，要主 agent 用 `unit-graph why` 核它在不在某个登记组的闭包里；B1–B6 计数非 0 是今天就有的候选，同样核；A1、A2 非 0 要逐处判是不是内容输入。推翻「真仓里没有收严新造的实例」的现象：C1–C5、D1 任一计数非 0 且 `why` 核实那个 impl 本该在某组闭包里。
- 扫描的限度：impl 头的形状、五条等待条件、use 的名字表都是照 Rust 那一份的写法在脚本里重写的（`implementation_shape`、`use_shape`、`Index.waits_for_its_wrapper_type`），不是调单元图；A1 沿类型名往下找按名字、不按模块（与 `open_the_traits_reaching_the_entry_from_outside` 同一个松法，多报不少报）。

## 第三条：改法乙，从两个入口往下追登记单元的返回值（③）

判定：**没打中**（推的，读源码）。52 个登记单元按怎么被够到分四路（第一轮主 agent 留的 `why` 清单 `/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/evidence/group.gpu_memory_ledger.rs.txt` 里从别的模块跨进来的 6 条链，其余 46 个是这 6 个往下的 R-隐式、R-方法 与成员）。52 个里 17 个类型、3 个常量与静态、8 个 impl 头、4 条 use 没有返回值；20 个函数逐个看返回值去了哪：

| 单元（函数） | 怎么被够到 | 返回值流到哪 | 改它会不会改判定字 |
|---|---|---|---|
| `states_in_one_dispatch_capped_by_the_state_count` | `on_card_compiling_these_kernel_sources` 直接调：「let states_per_dispatch = states_in_one_dispatch_capped_by_the_state_count(」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-verification/src/crash_judge_dispatch.rs:874`） | 三处：草稿区开几条、每条多大「let scratch_lanes: Vec<wgpu::Buffer> = (0..states_per_dispatch.div_ceil(states_per_lane))」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-verification/src/crash_judge_dispatch.rs:887`）；调度的容量「dispatch_width_schedule: DispatchWidthSchedule::starting_narrow(」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-verification/src/crash_judge_dispatch.rs:931`）；一次派活的上限「assert!(count <= self.states_per_dispatch);」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-verification/src/crash_judge_dispatch.rs:1085`）。进派活参数的只有每一份的起始序号与状态数「let dispatch = dispatch_words(」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-verification/src/crash_judge_dispatch.rs:1118`；参数是起始序号高低两半、状态数、每状态草稿字数，`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-verification/src/crash_judge_gpu.rs:1002`），内核按调用内序号取草稿「scratch_base = local * dispatch.scratch_words;」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-verification/src/crash_judge_common.wgsl:4495`），不读缓冲长度；结论按派活内序号接回「assert_eq!(entries.len(), count * VERDICT_WORDS, "回读的字数与派的相同");」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-verification/src/crash_judge_dispatch.rs:1322`） | **不改**，前提是「切法不进结论」（与调度不登进组同一个前提）。它的值只改一次派几个、草稿区多大；返回 0 被自己那一句「.max(1)」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-verification/src/gpu_memory_ledger.rs:93`）挡，改掉那一句让它交 0 时 `count <= 0` 的断言让这一次派活 panic；交得比显存大，建缓冲失败 panic |
| `compile_for_the_work_of_this_thread` | `pipelines_compiled_once_on_this_device` 里「.compile_for_the_work_of_this_thread(compile)」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-verification/src/gpu_unit_checks.rs:1144`），后者由「.pipelines_compiled_once_on_this_device(kernel_sources_digest(kernel_sources), || {」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-verification/src/crash_judge_dispatch.rs:903`）调 | **它的返回值就是判器的 11 条管线**，每个判定字都出自它们 | **不改**，靠参数化：签名「pub fn compile_for_the_work_of_this_thread<Output>(」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-verification/src/gpu_memory_ledger.rs:2254`）与「compile: impl FnOnce() -> Output,」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-verification/src/gpu_memory_ledger.rs:2256`），`Output` 没有约束，文件里测试模块之前没有 `unsafe`（`awk 'NR<3065' gpu_memory_ledger.rs \| grep -c unsafe` 量得 0，量的），造得出 `Output` 的只有调 `compile`（「compile()」，`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-verification/src/gpu_memory_ledger.rs:2271`）；能改的只是交不交（panic）。没有会红的检查盯着这一条（第二轮攻方给的钉签名测试没做） |
| `is_lost` | `take_the_readback_after_the_wait_or_record_the_device_lost` 与 `judge_one_dispatch` 里等卡没等成的那一支 | 只决定记不记设备丢失：「if !condition.is_lost() {」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-verification/src/gpu_unit_checks.rs:406`、`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-verification/src/crash_judge_dispatch.rs:1486`），两处之后都无条件「panic!("{failure}");」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-verification/src/gpu_unit_checks.rs:412`、`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-verification/src/crash_judge_dispatch.rs:1492`） | 不改：这一次派活不交结论 |
| `record_the_device_lost`、`record_the_device_lost_telling_the_marker_line`、`tell_the_device_lost_marker_line_the_first_time`、`print_the_device_lost_marker_line`、`gpu_device_lost_line`、`text_on_one_line`、`is_a_line_break` | 同上那两处的记账一路往下 | `()`、打标记行用的串与判断换行的 `bool` | 不改：只改记什么、打不打那一行，随后照样 panic |
| `books_locked`、`release_a_device`、`return_the_leases_kept_after_a_failed_wait_on`、`devices_marked_by_bus_locked`、`take_one_device_off_the_bus`、`new`、`default`、三个 `drop` | `GpuCard` 的字段点名 `DeviceStartedOnTheBooks`、`DeviceOutsideTheBooksMarkedOnTheBus`，R-隐式 带 `Drop` impl 与它们调的 | 锁卫、`()`、销账时的计数 | 不改：只在卡被丢掉时销账、减总线计数，影响的是后面借不借得到卡（回到「本次未跑」与派活宽度） |

判表本身不经这 52 个：表是 `PackedJudgeTablesReadBack` 的字（从判定库读回）与 `JudgeTables::words`（组里的入口），我追的两条入口里没有一处把登记单元的返回值写进表或改表的字。推翻它的现象：派活宽度不同（同一张卡、同一条流、`requested_states_per_dispatch` 取 1 与取状态数）时 GPU 判器对同一个状态交回的结论字不同；或在 `gpu_memory_ledger.rs` 测试模块之前出现 `unsafe`、或 `compile_for_the_work_of_this_thread` 的 `Output` 加了约束。

### `mod gpu_memory_ledger;` 那一行与这个模块的 use 不进键之后

判定：**没打中**（推的，读源码）。

- 那一行没有属性：「pub mod gpu_memory_ledger;」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-verification/src/lib.rs:53`），上一行是另一个模块的声明，不带 `#[cfg]`；同一份文件里 GPU 那几个模块（`crash_judge_dispatch`、`crash_judge_gpu`、`gpu_unit_checks`）的声明带 `#[cfg(feature = "gpu")]`，它们照进 GPU 判器组的闭包，`feature="gpu"` 这个原子照旧由它们记进键。
- 登记的四条 use 是 std：「use std::cell::Cell;」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-verification/src/gpu_memory_ledger.rs:75`）起的 75、76、78、79 四行；不点外部 crate，不影响依赖行。测试模块之前这份文件不写 `crate::`、`wgpu::`（`awk 'NR<3065' … \| grep -n 'wgpu::\|pollster\|singlefs_core::\|singlefs_format::\|crate::'` 无输出，量的），所以排掉这些单元不会让键少一行依赖。
- 文件里的 `#[cfg` 只有 `#[cfg(test)]`（1239、1286、1677 三个测试用函数与 3065 起的测试模块），都不在登记里、也不在 GPU 判器的闭包里；没有 `cfg!`。排掉登记的单元，键里少掉的 cfg 原子是零个。
- 将来会漏的只有这一种：有人给那一行加 `#[cfg_attr(…, path = …)]` 换文件，测试一按文件过滤会红（第二轮攻方说过）；给登记的单元加 cfg 条件，它的原子不进键——但登记的前提就是这些单元的行为不进判定字，cfg 只能改它们编不编进来，编不进来时判器自己编不过（推的）。
- 登记的 17 个类型里有两个是回调别名：「pub type CardMemoryReader = Box<dyn Fn() -> Result<Vec<CardMemoryReading>, String> + Send + Sync>;」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-verification/src/gpu_memory_ledger.rs:817`）与 `CardCapacityForThisProcess`（同一份文件第 991 行起，`Box<dyn Fn(usize, u64) -> u64 + Send + Sync>`）。它们由卡池装、在借显存时调；调它们的函数不在登记里，也就不在 GPU 判器的闭包里（测试一要求显存记账里进闭包的恰好是登记的那几个），所以这两条回调的返回值不进这两个入口的任何一步（推的）。第二条 A1 的扫描不认登记，沿 `GpuCard` 的字段会把它们报成 `gpu_judge` 组的回调实例，读数时按这一条排掉。

## 第四条：两份实现，第二轮对齐之后再找不等价（④）

判定：**打中（夹具两个，推的，待跑）**。两处都是「这一轮新写的规则让一处原文与去前缀的差别第一次影响闭包」：今天两份都靠 blanket 放行收 `Wrapper` 的 impl，读法差在哪都不显；收严之后 Rust 一份照开 `Store`、Python 一份不开。

| 夹具 | 不同在哪（两份各引一行） | 预期 |
|---|---|---|
| `raw_identifier_any_in_the_entry`：入口 `entry(thing: &dyn std::any::r#Any)`，里面 `downcast_ref::<Box<dyn Store>>()` 再 `put(3)` | Rust 的裸名去掉 `r#`：「let name = identifier_name(token_text(source, token));」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/rust_items.rs:233`），所以 `plain_names` 里有 `Any`，closure.rs:1415 那一条开入口的 `dyn Store`；Python 那一句拿 `unit.plain` 原样比：「if unit.kind in ("static", "item_macro") or "Any" in (unit.plain or ()):」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/scripts/gate/lib_function_units.py:5327`），`unit.plain` 里是 `r#Any`（`token_references` 收裸名不去前缀） | verify_copy 收、键变；python_verify_copy 不收、键不变；今天两份都收。第二轮对齐 `r#` 时这一句的 `dyn_trait` 参数去了前缀，判「提到 `Any`」的那一半没去 |
| `raw_identifier_trait_in_the_entry_signature`：入口 `entry(store: &dyn r#Store)` 调 `store.put(3)` | Rust 的签名 trait 名去前缀：「let candidate = identifier_name(text_at(position + 1));」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/rust_items.rs:288`）；Python 的 `entry_signature_traits` 照原文收：「out.add(rust_file.t(i + 1))」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/scripts/gate/lib_function_units.py:5107`），`dyn_trait("r#Store")` 在 `impls_of_trait` 里查不到 | 同上一行。这一句是旧代码，收严之后才让它分出两份闭包；新加的 `open_the_traits_reaching_the_entry_from_outside` 两份都去前缀，盖不到入口自己的签名 trait |

改法（只在我的推演上过、被攻过零轮）：Python 那两句各套一层 `without_the_raw_identifier_prefix`（判「提到 `Any`」时比去前缀的名字；`entry_signature_traits` 收去前缀的名字，并照 Rust 一份跳过关键字）。两格各修各的；撤回改法甲也让两格回到「两份都收」。

真仓里有没有这两种写法由第二条 D1 的计数回答（预期 0）。门禁 code-tooling 的两份对拍格只比真仓登记的入口，真仓没有这两种写法时它照绿（推的），所以这两格它今天抓不到。

### 背景材料点名的另几处，逐句对过、没找到分歧的（推的）

| 处 | 两份的写法 | 为什么相同 |
|---|---|---|
| static 与 `Any` 那一条的种类 | Rust 在 `resolves_its_own_names()` 为真的种类里判 `Static`、`ItemMacroInvocation`；Python 在 `structure_only` 之外判 `static`、`item_macro` | 两边排掉的都是 use、mod 声明、模块头、extern crate；文档测试两边都判 |
| static 与 `Any` 开的名字 | Rust 开 `dyn_trait_names`（去前缀的末段）；Python 开 `unit.dyn_traits` 再去前缀 | Python 的 `token_references` 也是走路径取末段（「dyn_traits.add(texts[j])」，`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/scripts/gate/lib_function_units.py:2343`），都只记 `dyn` 后面头一个 trait（第一条「自动 trait 在前」那一形两份同漏） |
| trait 头入口 | Rust 开 `container.name`（`name_at` 去前缀）；Python 开 `without_the_raw_identifier_prefix(root.container.short)` | 相同 |
| 第三条的两句 | Rust 按（文件, 行内模块路径）筛、比 `imports.names`（`name_at` 去前缀，`a as b` 只记 b）；Python 按（文件, scope）筛、比去前缀的 `use.imports`（`a as b` 两边都记） | 只在「同一段模块里 `use other::X as Wrapper` 与 struct `Wrapper` 两臂」时名字表不同：Python 多记原名 `X`，比的是 `Wrapper`，两边都命中；原名恰好也叫 `Wrapper`（`use other::Wrapper as Wrapper`）时第四条的改名表两边都让它不等。结果相同 |
| 判定次序 | Rust 在 `visit_unit_for_target` 里先判登记的「不沿它收」、再判 cfg、再收名字；static 与 `Any` 在 R-隐式 之前；Python 在名字、方法、路径之后才判 static 与 `Any` | 闭包是不动点：`open_trait` 与 `name_type` 都按名字去重、与先后无关；R-方法 先见方法名、类型后被点名的，`name_type` 那一支照补（trait impl 整个带进来、固有 impl 按见过的方法名补）。登记的「不沿它收」只有 Rust 有，Python 那一份没有对应（对拍格比的是 `unit-graph closure`，它也不认登记，两边一致） |

## 没打中的形状（汇总）

- ①：第一条「试过、没打中的形」那张表，13 形（第三条 4 形、入口所在 trait 2 形、static 与 `Any` 7 形），都是读代码推的、没有夹具。
- ③：52 个登记单元逐个分到四路、20 个函数逐个追返回值（第三条表）；`mod` 那一行与四条 use 的 cfg、依赖（第三条第二节），读源码，没跑。
- ④：背景材料点名的 static、`Any`、trait 头入口、第三条两句与判定次序五处逐句对过（第四条第二张表）；新加的 `open_the_traits_reaching_the_entry_from_outside` 两份的签名取法（到头一个 `{`、从单元头一个词起、收标识符去前缀）、沿类型名往下找、`names_written_where_a_trait_goes` 的范围也对过，没找到分歧。
- 取样范围：夹具 16 个（10 个新形、3 个复核、2 个两份不等价、1 个对照），各自成根；真仓扫描 16 类，`crates/` 下 dump 出来的全部 Rust 单元（只切 fn、impl、trait、const、static、struct、enum、union、type、macro_rules、use、item_macro 这几种的词）。

## 这条腿自己的限度

- 一个数都没跑：单元图、两份 `lib_function_units.py`、rustc、扫描脚本一次都没起（子 agent 不交任务，扫描脚本是本仓里的脚本，也不许直接起）。13 个打中形、2 个不等价形、对照的预期全是读代码推的；夹具没编过，写错 Rust 的那一格（最可能是 `trait Source: for<'a> Store<'a>` 高阶父 trait 与 `Box<dyn Send + Sync + Store>` 自动 trait 在前这两处）`judge.py` 写 `value error=…`、照跑别的格、最后退 2；主 agent 跑出来与 `out/expected.txt` 不同的格，按跑出来的算。
- 扫描脚本没跑过一次：它的切词器（`lex`）、impl 头与 use 的取法、五条等待条件的近似、A1 的找法都是照 Rust 那一份重写的，有写错的地方会在 `class=` 读数上显出来（例：`entries_not_found` 非 0、某一类数到上千），读数要先抽几处 `instance` 行对源码。它不调单元图，切分用 dump、判法是我写的。
- A1 只读了 `JUDGING_UNITS_ENTRY_GROUPS` 一张登记表；观察者入口与按条复用的测试函数入口没扫。
- 复核三形的机理（第一条第三节）是读 `workspace.rs`、`closure.rs` 与第二轮 `judge.py` 推的；15869、15908 两次读数是主 agent 跑出来的，我只读了日志里那几行。
- 第三条「不改判定字」的两处都靠前提：派活切法不进结论（没有另证，第二轮攻方给的宽度扫描还没做）、`Output` 的参数化（没有会红的检查）。
- 甲一到甲七那张改法表每一格都是推的，没实现、没量放大了多少闭包。

## 要主 agent 跑的命令

1. 跑模型（编两份 unit-graph、16 个夹具各自成根比四种规则、编夹具看值、两份 dump 真仓并扫描）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r3-opus -- bash /home/fy5090/code/singlefs/research/prompts/m3-close-unit-graph-wrapper-impls-r3-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy /home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r3-opus/model-run /home/fy5090/code/singlefs/research/prompts/m3-close-unit-graph-wrapper-impls-r3-opus-model`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r3-opus`（命令里全是绝对路径；判决复跑时照规则把模型目录拷进主 agent 自己的草稿目录，最后一个参数换成拷贝，运行目录换成那个草稿目录下一个以 `-run` 收尾的名字）。
   - 看哪几行：末行 `TASK_SERVER_RESULT` 的 `exit=`：0 是夹具每一行都与 `out/expected.txt` 相同且扫描跑成；1 是有夹具格与预期不同（`task-server/task log <号> --all` 里 `  ✗ 与 expected 不同` 那几行点名）；2 是有格跑不成（`error=` 行，或 `name=summary … failed_steps=<n>` 非 0）、dump 退非 0、两份 dump 不同（`name=dump same=no`）或扫描退非 0。夹具读数是 `name=<夹具> rule=<today|verify_copy|python_today|python_verify_copy> impl_in_closure=… key_changes_with_the_edit=…` 与 `name=<夹具> value_before=… value_after=…`，汇总 `name=summary lines=80 mismatches=<n> failed_steps=<n>`；`name=binaries today_sha256=… verify_copy_sha256=…` 两个值要不同；扫描读数是 `class=<类名> instances=<n> <计数>=<n>` 那 16 行与 `name=scan …` 末行，逐处的 `instance` 行在 `/home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r3-opus/model-run/scan.txt`（读法见第二条「扫描怎么做、读数怎么读」）；`name=rerun judge_exit=… scan_exit=…` 是两步各自的退出码。
   - 跑完：运行目录里有两份编译目录（`target-today`、`target-verify-copy`，release 编的 unit-graph 连依赖）、`bin/`、`roots/`、`build-original/`、`build-edited/`、两份 dump 与 `scan.txt`，读完由主 agent 删整个运行目录。
2. 扫描读数里 C1–C5、D1 任一计数非 0、或 B1、B4 的那一处要核在不在登记组闭包里的，逐处用 `unit-graph why` 核（命令要等第 1 条的读数出来才写得出具体的入口与 `--unit`，由承接的腿写）。

## 没做什么

- 不判正推那几格；不替主 agent 采纳；没往统一任务服务交任务，判决要引的数没复跑（命令在上一节）。
- 副本上量出的数这一腿没有；以后跑出的夹具数是在夹具与 verify-copy（草稿树）上的数，不算入库装置上的数。
- 没建仓副本与编译目录：草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r3-opus` 里这一腿只有认领留下的 `.owner`、写者锁与 `cite-check.py` 的输出 `cite-check.txt`；模型目录只有源码、夹具与预期。
- 没造 `AtomicPtr<()>` 擦掉类型那一形的夹具（第一条没打中表末行）；没扫观察者入口的回调（限度）。
- 没写 `unit-graph why` 的具体命令（要等扫描读数）。
