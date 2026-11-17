# m3-close-unit-graph-wrapper-impls-r2 攻方腿（Opus）报告（2026-11-13）

立场：找反例，假设收严一、二与改法乙都是错的。前几轮判决 `research/prompts/m3-close-unit-graph-wrapper-impls-r1-main-verification.md`；第一轮九个夹具的形当已攻清单，没重复。禁读清单里的三处没读。被攻的是 `verify-copy` 里的改动（只读），今天的规则是主工作区 `crates/singlefs-unit-graph/`。这一腿一个数都没跑（子 agent 不交任务），下面所有「打中」「没打中」都是读代码推的，等主 agent 跑模型坐实。

## 复跑

模型目录 `research/prompts/m3-close-unit-graph-wrapper-impls-r2-opus-model/`，冻结副本是 `/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy`。复跑命令（主 agent 交，末节第 1 条）：

```
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r2-opus -- bash /home/fy5090/code/singlefs/research/prompts/m3-close-unit-graph-wrapper-impls-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy /home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r2-opus/model-run /home/fy5090/code/singlefs/research/prompts/m3-close-unit-graph-wrapper-impls-r2-opus-model
```

它做的（`tools/rerun.sh` 的 `rerun_steps`）：在仓根与 verify-copy 两棵树里各 `cargo build --release -p singlefs-unit-graph --bin unit-graph`，各照 `cargo metadata --format-version 1 --no-deps` 的 `target_directory` 找到二进制，编完马上拷进运行目录的 `bin/`（两次编可能落在服务给的同一个编译目录，后一次会盖掉前一次）；`tools/judge.py` 把 11 个夹具 crate 铺成两棵树（原样、按 `edits.tsv` 改了被判方法里一个乘数的），四种规则各收一趟闭包——today、verify_copy（两份 `unit-graph closure … --toolchain-line host: x86_64-unknown-linux-gnu`）、python_today、python_verify_copy（两份 `scripts/gate/lib_function_units.py`，经 `tools/python_closure.py`，照 `unit_graph_key` 算键）——记被改的方法在不在闭包里、改前改后键变不变；再用 `rustc` 编夹具跑驱动，记入口交回的值；逐行与 `out/expected.txt`（跑前写下的预期，推的）比；最后 `tools/measure_closure_sizes.py` 在仓根上用两份 unit-graph 量登记九组、random_histories 六个测试函数、七个观察者入口的闭包大小（第四条）。不是崩溃枚举，没有崩溃状态，3b 的状态数上限不适用。估时（推的）：两次 release 编译各几分钟；夹具 88 趟闭包各约 1–3 秒、44 次 rustc 各不到 1 秒；量闭包 44 趟各约 1–10 秒（主 agent 早先在真仓上收 GPU 判器组用了 1.65 秒，`m3-close-gpu-judge-closure/v2-out/gpu.v2.err`）。合计估 20 分钟以内，没到 40 分钟，不缩。

模型文件 sha256（`find . -type f | sort | xargs sha256sum`，在模型目录里跑的原样输出）：

```
238c2a29337bcfa93194dfc1979a3640b233dfa8822f70d946ead87898f89133  ./edits.tsv
d3a7e25d89fee6c5ee6a9a85e1cbb57644222c360a0a7d17b4460ff7b49e0196  ./fixture/Cargo.lock.fixture
c1e8f997805f05d064c553f0c4fe82849232ba2c82f7c121297b8ff2331f85cb  ./fixture/crates/bound_in_parentheses_on_an_opened_implementation/Cargo.toml.fixture
610890285c402a5cc20ad3cb1dcc2e2924f0d5d937e809d13c4070f1f0cd0639  ./fixture/crates/bound_in_parentheses_on_an_opened_implementation/src/lib.rs.fixture
0a0cedc15b14ce80f11782eec9940c508b57ee0c38e8ecec1570a8896b804efe  ./fixture/crates/higher_ranked_bound_on_an_opened_implementation/Cargo.toml.fixture
d3750e30dd78bd2c1534fa1d004efdc3c43d3327918aa24b75b99d554cb4651f  ./fixture/crates/higher_ranked_bound_on_an_opened_implementation/src/lib.rs.fixture
32c02a3f68670f072c90ada42ff590224ca8462f09330593146b8ab38edd1865  ./fixture/crates/receiver_of_a_provided_trait_method_entry/Cargo.toml.fixture
3aeb2963a3ec9489ed01a16749115dd0b4d4228fc684f9c46e8945838ef170d4  ./fixture/crates/receiver_of_a_provided_trait_method_entry/src/lib.rs.fixture
f87c5573d6430326bdcfbe21f7e7dfc856d285e6b8372ac4a4aa2036a898c51b  ./fixture/crates/self_type_written_as_a_raw_identifier/Cargo.toml.fixture
0438865891cd366fafa51af43f441fb3505cf332a31393313461849ad86f6f03  ./fixture/crates/self_type_written_as_a_raw_identifier/src/lib.rs.fixture
305e213f0742690c32c5d7770bf6d39245684f6789518f550dc38d94ce70fd45  ./fixture/crates/signature_type_also_defined_in_an_unlinked_file/Cargo.toml.fixture
651301aa25608cb7374c2ce24816df89801c7daae3371ecb2a06366784e299d3  ./fixture/crates/signature_type_also_defined_in_an_unlinked_file/src/lib.rs.fixture
4a10af8ae2fcc003b1119869ad4a3f8e939b75aea7e53e8bbf3c01b8523c93bc  ./fixture/crates/signature_type_also_defined_in_an_unlinked_file/src/stray.rs.fixture
7b3075dbb493f8734a80b13e623dc4020d02904a6fb0690fc0ba146552c54e2e  ./fixture/crates/struct_in_an_inactive_cfg_arm_alias_in_the_active_arm/Cargo.toml.fixture
cb933724f535083cb772e602acf6fe07f627970e8f2e4327420757bbb463f6b2  ./fixture/crates/struct_in_an_inactive_cfg_arm_alias_in_the_active_arm/src/lib.rs.fixture
6f42ab29d321040f9495b0596e39f822b1e40355f03be5cc3e04bf7b794f7b95  ./fixture/crates/trait_object_behind_a_dyn_any_parameter/Cargo.toml.fixture
5d46acd2d2bae3a93c11b28df893e290bfa8edf82ce42e159b8e4188689e5c60  ./fixture/crates/trait_object_behind_a_dyn_any_parameter/src/lib.rs.fixture
3b3fc00e650516fdbaf84270c2f4e75983052960854db78c46ef707d7a2522f3  ./fixture/crates/trait_object_field_of_an_opened_implementation/Cargo.toml.fixture
6d21b0c823aebd4d329d0d250ff7af3efbf2184dbb124dbaa0a6a0bf689b625b  ./fixture/crates/trait_object_field_of_an_opened_implementation/src/lib.rs.fixture
e522b0ceed97232b367be66b3b4d6d1f327b3d5643cb48479e1555dcf28e3f85  ./fixture/crates/trait_object_held_by_a_static/Cargo.toml.fixture
b41e044b30c8dc5c4c84101e0d7a5427b21812145aaf65ee3e892233f97b1968  ./fixture/crates/trait_object_held_by_a_static/src/lib.rs.fixture
fc59963ff8503154b217b6a8cbdab955e4cbe5c021bcb44adbc4fc4643507a19  ./fixture/crates/variant_imported_through_a_raw_identifier_path/Cargo.toml.fixture
8ef6451c2eceed2957499b851ac573e8c678ee17649d3811f34ed4284c9f145f  ./fixture/crates/variant_imported_through_a_raw_identifier_path/src/lib.rs.fixture
5732cea26f1beeb4d395f15675f250fbb39d37dfb61e3826c9ac06351eadb37a  ./fixture/crates/wrapper_named_in_the_entry_control/Cargo.toml.fixture
72f725b22dc312cdb4172e827723fd107b82e1ebd21725b55392b33e2090230f  ./fixture/crates/wrapper_named_in_the_entry_control/src/lib.rs.fixture
3bbfd3185df40652f794c70ae1fdf54d22c2f53e7b6921aa2895f92169b9cd34  ./out/expected.txt
f4932bec418eff1638b719e58a6635c3a9561aa2b876adbee7f0705432022251  ./tools/judge.py
5234b7b4d936235b9a47a13c84ba6996b75770e8a73eaa9dcd0a7707f2e25b66  ./tools/measure_closure_sizes.py
acb320569b34f90f4940a19f3377a30f84b5641d1ef063e2eddb4979f45a8104  ./tools/python_closure.py
e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b  ./tools/rerun-from-a-clean-work-directory.sh
85347d646acc8aedd468551914998f46c4d5c7d79fe95c8671c31b3975904fda  ./tools/rerun.sh
```

夹具文件带 `.fixture` 后缀（照第一轮模型目录的做法），`judge.py` 铺进运行目录时去掉后缀。`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps` 那一段（连同它上面的说明注释）。

## 各格判定一览

| 关的那一条 | 格 | 判定 | 依据一句话 | 量的 / 推的 |
|---|---|---|---|---|
| 1 甲过窄 | ① 五条的边角 | **打中（夹具）**：`struct_in_an_inactive_cfg_arm_alias_in_the_active_arm` | 第三条只问同一段模块里「有没有」叫这个名字的 struct，不问那一臂编不编进来；cfg 两臂一臂 struct、一臂别名，编进来的是别名，值照原名造 | 推的，待跑模型 |
| 1 甲过窄 | ① 其余（`&'a mut W<T>`、`(W<T>, u32)`、`[W<T>; N]`、`dyn Tr + 'a`、`<T as Tr>::Out`、`$name<T>`、`#[path]`、`include!`、两个目标各编一次、宏卫生、`Self`、关联类型、泛型遮蔽、嵌套函数条目） | 没打中 | 头一个名字取到的是包装类型或不是类型；不编进来的写法两份规则一样漏；按名字点名对路径、文件归属不敏感 | 推的 |
| 1 甲过窄 | ② 不写包装类型名造值 | **打中（夹具）**：`trait_object_held_by_a_static`、`trait_object_behind_a_dyn_any_parameter` | 值从入口签名以外进来：`static` 里的 trait 对象、`&dyn Any` 擦掉类型再 `downcast_ref`；收严二只看入口签名与入口所在 impl | 推的，待跑模型 |
| 1 甲过窄 | ② 其余（`Default`/`From`/`Into`/`FromIterator`、闭包外返回 `impl Trait`、闭包捕获、`transmute`、测试夹具函数） | 没打中 | 要么闭包里某个单元写出了类型名（R-隐式 把那个 impl 的全部成员带进来），要么今天也不收（回调里调的代码两份规则都看不见） | 推的 |
| 1 甲过窄 | ③ 收严二三处开放漏的 | **打中（夹具四个）**：`trait_object_field_of_an_opened_implementation`、`receiver_of_a_provided_trait_method_entry`、`bound_in_parentheses_on_an_opened_implementation`、`higher_ranked_bound_on_an_opened_implementation` | 沿字段找 trait 对象只从入口签名点名的类型起，不从 R-开放 开进来的 impl 的自身类型起；入口自己所在的 trait 不开；约束只认紧跟 `:`/`+` 的名字，`(Store)`、`for<'a> Store<'a>` 漏 | 推的，待跑模型 |
| 1 甲过窄 | ③ 别名、关联类型、`Vec<Box<dyn Tr>>` 在别名后面、返回值位置 `impl Trait` | 没打中 | `type_units_by_name` 收别名与关联类型单元（按名字，连关联类型名一起），沿它们的词找得到 `dyn` | 推的 |
| 1 甲过窄 | 真仓里找新反例 | **没找到**（一次抽样） | 没跑单元图；只核了登记入口的签名形状与 `static`、`thread_local!` 里的 trait 对象（见第一条末节） | 推的 |
| 2 两份不等价 | ④ | **打中（夹具三个）**：`variant_imported_through_a_raw_identifier_path`、`self_type_written_as_a_raw_identifier`、`signature_type_also_defined_in_an_unlinked_file` | Rust 一份认「use 穿过」与「头一个名字」时去掉 `r#`、Python 一份照原文；Rust 的 `type_units_by_name` 收工作区每一个单元（连不属于任何目标的文件），Python 只收进了目标的文件 | 推的，待跑模型 |
| 3 乙 | ⑤ 按限定名排会不会排掉没登记的同名单元 | 没打中（今天的树） | 显存记账里 52 个登记名今天各只对一个单元；机制上同名的两个单元（cfg 两臂、同一个类型的第二个固有 impl 块头）会一起排，测试一用 `BTreeSet<String>` 比、看不出多了一个同名的 | 推的（读源码数的） |
| 3 乙 | 三条测试各证不证得了它的文档句 | 证得了（各有一处限度） | 测试一的主断言证成员相等；测试二证今天的树上只多掉 `mod gpu_memory_ledger;` 那一行（那一行从此不进键）；测试三只证那一处改动 | 推的 |
| 3 乙 | 「显存记账的行为不改判定结局」的会红检查 | **做得成一部分** | 宽度封顶归「切法不进结论」，可以扫宽度做蜕变断言；编内核透传可以钉签名，靠参数化挡；设备丢失与显存不够的记账在真卡上造不出，做不成 | 推的 |
| 4 闭包放大 | 别的登记组与测试函数入口各多收多少 | **待量** | 命令写好（末节第 1 条，`measure_closure_sizes.py` 那一段）；收严二只增不减，收严一只减不增，净变化要量 | 推的 |

## 第一条：收严后的改法甲会不会让某个入口的键过窄（①②③）

### 核过的规则（verify-copy 的代码）

- 等包装类型的第三条只看类型单元在不在同一份文件、同一段行内模块：「) && unit.file == head_unit.file」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2128`），候选取自 `type_units_by_name`，那张表收工作区里每一个类型单元、不看 cfg：「for (position, unit) in units.iter().enumerate() {」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/workspace.rs:1251`）。`implementation_waits_for_its_wrapper_type` 里没有一处调 `activity`。
- 收严二的三处开放都经同一个取名法：「if !matches!(token_text(&file.source, token), "impl" | "dyn" | ":" | "+") {」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2146`），只收紧跟这四个词的那一个标识符。
- 沿字段找只从两处起：入口签名到头一个 `{` 的标识符、入口所在 impl 的自身类型名字；之后沿每个类型单元的 `plain_names` 往下：「for trait_name in self.names_written_where_a_trait_goes(*type_unit) {」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2199`）。被 R-开放 开进来的 impl 只再开它头上的约束：「for bound_trait_name in self.names_written_where_a_trait_goes(*head) {」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2272`），不沿它的自身类型往字段里找。
- 这一趟只在入口那里调一次：「run.open_the_traits_reaching_the_entry_from_outside(*entry);」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:3367`）。闭包里别的单元写的 `dyn Trait` 照旧只在弄坏开关下开：「for trait_name in &unit.references.dyn_trait_names {」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:1418`）。

主 agent 的论证「值要么由闭包里某个单元写出 `Wrapper` 造出来，要么从入口外面传进来（入口那一侧由 `open_the_traits_reaching_the_entry_from_outside` 与 R-开放 收）」里，「从入口外面传进来」只罩到入口签名与入口所在 impl 这一层；值经全局状态、经类型擦除、经开进来的 impl 的字段、经入口自己所在 trait 的 `self` 进来时，这一侧没收，而今天的 blanket 放行把它们兜住了。

### 打中的形（每个都在模型的夹具里；预期今天收、verify-copy 不收、改那一行入口交回的值 6 变 9，全部是推的，待主 agent 跑）

| 夹具 crate | 攻击面 | 形 | 被改的那一行 | 许可它的那一句（verify-copy） |
|---|---|---|---|---|
| `struct_in_an_inactive_cfg_arm_alias_in_the_active_arm` | ① | 同一段模块里 `#[cfg(not(unix))] pub struct Wrapper<Inner>(pub Inner);` 与 `#[cfg(unix)] pub type Wrapper<Inner> = real::Real<Inner>;`，`impl<Inner: Store> Store for Wrapper<Inner>`；入口 `real::Real(Base).put(3)` | `self.0.put(amount) * 2` | `/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2128` 那一条只问同文件同模块里有没有叫 `Wrapper` 的 struct；不编进来的那一臂照算。编进来的是别名，值照 `Real` 造，`Wrapper` 没人写 |
| `trait_object_held_by_a_static` | ②③ | `pub static INSTALLED: OnceLock<Box<dyn Store + Send + Sync>>`，闭包外的 `caller` 先 `set(Box::new(Wrapper(Base)))`；入口没有参数，从 `INSTALLED.get()` 取到的盒子上调 `put(3)`（取不到交 0） | `self.0.put(amount) * 2` | 入口签名里没有类型；`static` 单元里的 `dyn Store` 只在弄坏开关下开（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:1418`）；`.put(` 落到 trait 头，trait 头只带「放行」的 blanket |
| `trait_object_behind_a_dyn_any_parameter` | ② | 入口 `entry(thing: &dyn Any)`，函数体里 `downcast_ref::<Box<dyn Store>>()` 再 `put(3)`；盒子由闭包外的 `caller` 造 | `self.0.put(amount) * 2` | 签名里只有 `Any`（工作区里没有它的 impl，也不是类型单元）；`dyn Store` 写在函数体里，不在签名里 |
| `trait_object_field_of_an_opened_implementation` | ③ | 入口 `entry(source: &dyn Source)`；`impl Source for Holder`（全具体、头上没有约束），`struct Holder { pub store: Box<dyn Store> }`；`caller` 造 `Holder { store: Box::new(Wrapper(Base)) }` | `self.0.put(amount) * 2` | R-开放 开 `Source`，开进来的 impl 头上只有 `Source`（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2272`）；`Holder` 不在入口签名里，不沿它的字段找 |
| `receiver_of_a_provided_trait_method_entry` | ③ | 入口是 trait 自带体的方法 `Source::total`（`self.amount() + 0`）；`impl<Inner: Store> Source for Wrapper<Inner>` 的 `amount` 调 `self.0.put(3) * 2`；`caller` 是 `Wrapper(Base).total()` | `self.0.put(3) * 2` | 入口所在的 trait 头上没有约束、签名只有 `&self`；入口自己所在的 trait 不开，`Self` 由调用方定 |
| `bound_in_parentheses_on_an_opened_implementation` | ③ | 第一轮跨边界那一形，约束写成 `impl<Inner: (Store)> Source for Adapter<Inner>` | `self.0.put(amount) * 2` | `:` 后面是 `(`，`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2146` 只收紧跟着的那一个标识符 |
| `higher_ranked_bound_on_an_opened_implementation` | ③ | 同上，约束写成 `where Inner: for<'a> Store<'a>`（`Store` 带生命周期参数） | `self.0.put(amount) * 2` | `:` 后面是 `for`；`Store` 跟在 `>` 后面 |
| `wrapper_named_in_the_entry_control`（对照） | — | 入口 `Wrapper(Base).put(3)` | `self.0.put(amount) * 2` | 预期四种规则都收、键都变：证判法不是一律判「不在」 |

后两形与第一轮 `wrapper_built_by_the_caller_of_an_open_entry` 同一个机理（值由调用方经开放的入口给），攻的是收严二新写的取名法，不是第一轮那一格本身。`receiver_of_a_provided_trait_method_entry` 里 `Store` 只是 `amount` 用到的 trait；被判的是 `Source for Wrapper<Inner>` 这个 impl，它自身类型的头一个名字是本模块的 `Wrapper`，五条都满足，等 `Wrapper` 被点名。

### 打中之后的四句（照 `.claude/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种`）

1. **分不分辨臂**：分辨。七个打中形上今天的规则收（带泛型参数的 impl 一律按 blanket 放行，trait 头进闭包就带它）、verify-copy 不收，对照夹具两边都收。不过其中四形（`static`、`dyn Any`、开进来的 impl 的字段、入口自己所在 trait 的 `self`）在**全具体**的 impl 上今天就漏（把 `Wrapper<Inner>` 换成 `struct Doubled(Base)` 加 `impl Store for Doubled`，今天也等 `Doubled` 被点名），病根是「被点名 = 闭包里有单元写出类型名」这一条两边共用的前提；收严一把它的射程从全具体 impl 扩到带泛型参数的 impl。判臂只看这一格分辨得出：今天靠 blanket 兜住，收严后不兜。cfg 两臂那一形与两种约束写法是收严一、二自己新写的判法的边角，不是共用前提。
2. **被判的系统看不看得到判别它的东西**：看得到。单元图手里有每个单元的条件链（`activity` 判得出那一臂编不编进来）、每个 `static` 单元与开进来的 impl 自身类型的词、入口单元的 `container` 是 trait 头、约束里 `(`、`for<…>` 的词法单元；收严一、二没用这些。
3. **满足的是判据字面的哪一个分句**：背景材料第四节判据第一条「攻方给出新反例（今天的规则收、收严后不收、改它入口结果变），主 agent 复跑坐实的」的整句，七形都按它造（入口值 6 变 9 是推的）。四个「调用方造值」的形（`static`、`dyn Any`、开进来的 impl 的字段、两种约束写法）里入口执行的代码是闭包外造的值的方法，「改它入口结果变」照字面成立（入口交回的数变）；算不算这一题要管的「入口行为」由主 agent 定——真仓登记的判器、录入、观察者入口的签名都是数据，下一节说真仓里没找到这几形。
4. **跑前条款给的改法在打中的格上还中不中**：判据只给两条出路「那一形再收严」或「这套规则撤回」。撤回在七格上都不中（回到今天的 blanket）。再收严各格怎么修见下表；没有一种同时修七格。

| 再收严的写法（只在我的推演上过、被攻过零轮；每格「推的」） | cfg 两臂 | `static` | `dyn Any` | 开进来的 impl 的字段 | 入口所在 trait 的 `self` | `(Store)` | `for<'a> Store<'a>` |
|---|---|---|---|---|---|---|---|
| 第三条只认这个目标下编进来的那一臂（`activity` 判 Active），或「同名类型单元里有任何一个不是本模块 struct/enum/union（别名、另一臂）就照 blanket 放行」 | 修 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 |
| 取约束名改成「`:`、`+` 之后跳过 `(`、`?`、`for<…>` 再取头一个名字」，或干脆取约束区里每一个标识符（多开不少开） | 不修 | 不修 | 不修 | 不修 | 不修 | 修 | 修 |
| 被 R-开放 开进来的 impl：自身类型名字也进 `type_names_to_look_into` 沿字段找 | 不修 | 不修 | 不修 | 修 | 不修 | 不修 | 不修 |
| 入口是 trait 头下的方法时开这个 trait 自己（R-开放 照入口签名里写了 `dyn ThisTrait` 办） | 不修 | 不修 | 不修 | 不修 | 修 | 不修 | 不修 |
| 闭包里任何 `static`（含 `thread_local!` 切出来的）与任何单元里写的 `dyn Trait` 都照 R-开放 开 | 不修 | 修 | 修 | 修 | 不修 | 不修 | 不修 |
| 撤回收严一（回到今天的 blanket） | 修 | 修 | 修 | 修 | 修 | 修 | 修 |

倒数第二行（闭包里任何 `dyn` 都开）就是今天弄坏开关 `dyn-traits-open-every-implementation` 打开时的收法（规则把它当弄坏、只多连），会把闭包放得很大，要第四条那种量法量过才谈得上；前四行各修一格，合起来仍有「值经全局状态或类型擦除进来」这一类没盖住。

### 真仓里找过什么（量的是 grep，单元图没跑）

- `static` 里放 trait 对象：`grep -rnE '^\s*(pub(\([a-z]+\))? )?static [A-Z_0-9]+: .*dyn ' crates/ --include=*.rs | wc -l` 量得 0；`thread_local!` 之后四行里带 `dyn ` 的 0 处；`OnceLock`/`LazyLock`/`Mutex` 等包着 `dyn` 的 `static` 0 处（量的，`crates/` 下 `.rs`）。
- `dyn Any`：`crates/` 下 15 处，逐个看过，都是 panic 的载荷（`&(dyn std::any::Any + Send)` 取文字）或装 panic 载荷的枚举字段，没有一处 `downcast_ref` 到 trait 对象（量的）。
- 带括号的约束 `<T: (…`、`, T: (…`：0 处；`: for<` 高阶约束：0 处（量的，`crates/` 下 `.rs`，正则见末节没做什么）。
- 入口是 trait 自带体的方法：登记表里九组与观察者那几行的入口都写成 `模块::函数` 或 `模块::类型::方法`，我按名字核了 GPU 判器组的 16 个与判镜像定义组的 6 个，都是自由函数或固有方法（推的，没逐个核 impl 头是不是 trait impl）。
- cfg 两臂一臂 struct、一臂同名别名或改名 `use`：没扫（写不出可靠的 grep），记在限度里。

所以真仓今天没有现成的反例（一次抽样，不写成「没有」）；往后谁在显存记账、判器那一侧写一个全局的 trait 对象或把约束写成 `(Trait)`，收严后它进不了那一组的键。

### ①②③ 试过、没打中的形（每一形都是推的，没有夹具）

| 面 | 形 | 为什么没打中 |
|---|---|---|
| ① | `&'a mut Wrapper<T>`、`(Wrapper<T>, u32)`、`[Wrapper<T>; N]` | 生命周期是 `Lifetime` 词法单元、不算名字，头一个名字取到 `Wrapper`；这几种值都要先有一个 `Wrapper` 的值，与 `Wrapper<T>` 本身同一个点名条件 |
| ① | `dyn Tr + 'a`、`fn(Wrapper<T>) -> u32` | 头一个名字是 trait 名或 `fn`，`type_units_by_name` 里没有，照 blanket 放行（多连） |
| ① | `<T as Tr>::Out` | 头一个名字是泛型参数，照 blanket；写成 `<Wrapper<T> as Tr>::Out` 的 impl 编不过（泛型参数不受约束） |
| ① | `macro_rules!` 里的 `$name<T>`，条目级宏调用生成的 impl | impl 写在宏体里不切成 impl 头单元，两份规则一样（今天也不收），不分辨 |
| ① | `#[path]` 同一文件当两个模块、`include!`、同一文件被两个目标各编一次 | 单元按文件切一次；第三条要「同一份文件」，包装类型与 impl 分在两份文件时照 blanket（多连）；同一份文件两个身份时点名按名字（`named_types` 是全局的名字集合），哪一份被写出都点到它 |
| ① | 宏卫生、`Self`、关联类型、泛型参数遮蔽、嵌套函数里的同名条目 | impl 头里写不了 `Self`；泛型参数同名由第一条挡；函数体里的条目不切成单元（`scan_items` 只进模块、impl、trait、`verus!`、`thread_local!` 的体），不进 `type_units_by_name`；宏生成的同名 struct 与本模块 struct 同名编不过 |
| ① | cfg 两臂各一个同名 **struct**（`#[cfg(unix)] struct W` / `#[cfg(not(unix))] struct W`） | 哪一臂编进来都叫 `W`，被写出时按名字点名，不漏；漏的只是「一臂 struct、一臂别名或改名 use」（已打中） |
| ② | `Default::default()`、`From`/`Into`、`FromIterator`、推断 | 目标类型总要在闭包里某处写出；写在 impl 的关联类型 `type Product = Wrapper<Base>` 里时，R-路径 `Self::Product` 与 R-隐式（全部成员）把它带进来 |
| ② | 闭包外的函数返回 `impl Trait`、闭包捕获、回调 | 值经回调进入口（`&dyn Fn() -> u32`）时，回调里调的代码今天也不在闭包里（两边都漏，不分辨）；回调签名里写了 `dyn Store` 的，今天的 R-开放 就开 |
| ② | `transmute`、指针转换 | 要写出目标类型；推断出的类型同上 |
| ② | 测试夹具函数造值 | 夹具函数在测试入口的闭包里，它写了类型名 |
| ③ | 入口签名经类型别名（`type Sources = Vec<Box<dyn Source>>`）、别名在别的 crate | `type_units_by_name` 收别名单元，沿它的词找到 `dyn`；按名字、跨 crate |
| ③ | 关联类型（`&<Engine as Config>::Backend`，`type Backend = dyn Store`） | 签名里的 `Backend` 按名字找到关联类型那个别名单元 |
| ③ | `Vec<Box<dyn Tr>>`、`HashMap<u32, Box<dyn Tr>>` 写在签名里 | 今天的签名 R-开放 就开 |
| ③ | 返回值位置的 `impl Trait`（入口 `-> impl Store`） | 入口自己只造值，`put` 在闭包外调，不是入口的行为 |
| ③ | 约束经超 trait（`trait Both: Store`、`impl<T: Both> …`） | 开 `Both` 时它的 impl 头被推进来，要么头上写着 `Store` 约束（递归开），要么自身类型点名包装类型（R-隐式） |
| ③ | `impl<Inner: Deref<Target: Store>>`、`where Inner::Target: Store`、`?Sized + Store`、`crate::Store` | `:` 或 `+` 后面紧跟 `Store`（路径写法里第二个 `:` 后面是 `Store`），照开 |

## 第二条：两份实现不等价（④）

判定：**打中（夹具三个，推的，待跑）**。三处都是这一轮新写的代码里两份的取法不同，不是旧代码的差。

| 夹具 crate | 不同在哪（两份各引一行） | 预期（`out/expected.txt`） |
|---|---|---|
| `variant_imported_through_a_raw_identifier_path`：`use crate::r#Layered::Doubled;`，`impl<Inner: Store> Store for Layered<Inner>`，入口 `Doubled(Base).put(3)` | Rust 记「use 穿过的名字」去掉 `r#`：「crate::lexer::identifier_name(crate::lexer::token_text(」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/workspace.rs:1283`）；Python 照原文：「self.names_a_use_path_passes_through.add(rust_file.t(index))」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/scripts/gate/lib_function_units.py:4318`），Python 的词带着 `r#`：「# 原始标识符的词按源码原样取、带着 r#（与 Rust 那一份 lexer.rs 同一条：词是源码里的那一段）」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/scripts/gate/lib_function_units.py:1726`） | verify_copy 收、键变；python_verify_copy 不收、键不变（Python 认为 `Layered` 没有被 use 穿过，等它被点名）。入口值 6 变 9：这一格同时是 Python 一份的键过窄 |
| `self_type_written_as_a_raw_identifier`：`impl<Inner: Store> Store for r#Wrapper<Inner>`，入口 `Base.put(3)` | Rust 的头一个名字去掉 `r#`：「identifier_name(self.text_of(index)).to_string()」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/rust_items.rs:722`，`name_at`）；Python 照原文：「"generics": frozenset(generics), "self_first_name": self.t(first_name) if first_name is not None else None,」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/scripts/gate/lib_function_units.py:3026`），`type_units_by_name.get("r#Wrapper")` 是空的 | verify_copy 不收（等 `Wrapper`）；python_verify_copy 收（照 blanket）。入口值 3 不变，比的只是两份的闭包 |
| `signature_type_also_defined_in_an_unlinked_file`：入口签名点名 `Holder`；同一 crate 另有一份没有 `mod` 声明的 `src/stray.rs`，里面的 `Holder` 有字段 `Box<dyn Extra>` | Rust 的 `type_units_by_name` 收工作区里每一个单元（「for (position, unit) in units.iter().enumerate() {」，`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/workspace.rs:1251`），工作区的文件是 crate 目录下每一份 `.rs`（引文见表下）；Python 只收进了目标的文件：「paths = sorted({path for target in self.targets.values() for path in target.files})」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/scripts/gate/lib_function_units.py:4269`） | verify_copy 开 `Extra`、收 `impl Extra for Plain` 的方法、键变；python_verify_copy 与两边的今天都不收。入口值 3 不变 |

表下的两行引文（整行）：

- Rust 收工作区文件：「} else if file_name.ends_with(".rs") || file_name.ends_with(".wgsl") {」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/workspace.rs:697`）。
- Python 的 impl 头单元只到 `{` 之前：「unit = self.add("impl", head["label"], start, head_end, context, module_path, container, conditions, True)」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/scripts/gate/lib_function_units.py:2853`）。

背景材料点名的另三处我逐行对过，没找到分歧：`names_written_where_a_trait_goes` 两份都照原文取词、都只看紧跟的一个标识符、都在单元自己的词法范围里（Rust 用切片 `tokens.get(position + 1)`，Python 用 `index + 1 < unit.last_token`）；签名到头一个 `{` 两份都从单元头一个词法单元（连属性）起、照原文收标识符；impl 头单元两份都只到 `{` 之前（Python 见下一段引文，Rust `(item_start, head_end)`），所以被开进来的 impl 只看头上的约束、不看成员体，两份一样。

要主 agent 跑的：末节第 1 条整趟跑完之后，这一格的读数在 `task-server/task log <号> --all` 的 `name=<夹具> rule=verify_copy …` 与 `name=<夹具> rule=python_verify_copy …` 两行；要单独复现第一格，在那一趟的运行目录里（`fixture-original/` 由 `judge.py` 铺好）各起一条（末节第 2、3 条）：

- Rust：`<运行目录>/bin/unit-graph-verify-copy closure <运行目录>/fixture-original variant_imported_through_a_raw_identifier_path::entry --toolchain-line 'host: x86_64-unknown-linux-gnu'`，看输出里有没有一行 `fn` 的位置是 `crates/variant_imported_through_a_raw_identifier_path/src/lib.rs:<起>-<止>` 且盖住第 24 行（`inner.put(amount) * 2` 那一行）。
- Python：`python3 <模型目录>/tools/python_closure.py --library /home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/scripts/gate --root <运行目录>/fixture-original --file crates/variant_imported_through_a_raw_identifier_path/src/lib.rs --path entry --edit-line 24`，看 `holding=` 那一行（预期 0，Rust 那一条预期有 1 行盖住）。

推翻它的现象：两份在这三个夹具上 `impl_in_closure` 与 `key_changes_with_the_edit` 逐行相同。门禁 code-tooling 的两份对拍格只比真仓登记的入口，真仓里没有原始标识符写的类型名与不属于任何目标的 `.rs` 时它照绿（推的，没扫真仓有没有这两样），所以这三格它今天抓不到。

## 第三条：改法乙（⑤）

### 52 个登记名按限定名比，会不会把没登记的同名单元一起排掉

判定：**今天的树上没打中**（推的，读源码数的）；机制上有一处看不见的口子。

- 比法：够到一个单元先看它的限定名在不在登记里：「if self.units_not_followed.contains(&unit.qualified_name) {」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:1389`），在就不进闭包、不往下收。限定名是 crate 名、模块路径、impl 标签、名字连起来：「unit.qualified_name = segments.join("::");」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/workspace.rs:1215`）；impl 标签去掉泛型参数表与 `where` 子句（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/rust_items.rs:2975` 起的 `implementation_label`）。所以同一个限定名能落到几个单元上的只有：同一条目的 cfg 两臂；同一个类型写了两个标签相同的 impl 块（第二个 `impl BooksOfACard {`）时的两个 impl 头。
- 今天的显存记账：四个登记了 impl 头的类型各只有一个固有 impl 块（`grep -nE '^\s*impl(<[^>]*>)? *(BooksOfACard|GpuMemoryLedger|DeviceConditionOfACard|BusesWithDevicesStartedOutsideTheBooks)\b'` 在 `crates/singlefs-verification/src/gpu_memory_ledger.rs` 命中 1017、1150、2011、2207 四行，量的）；文件里的 `#[cfg` 只有五处，都是 `#[cfg(test)]`，挂在不在登记里的条目上（量的）。没有同名的两个单元。
- cfg 两臂：一次构建只编进一臂，GPU 判器组的键按那一次的特性算，排掉不编进来的那一臂不改这一把键（推的）。
- 第二个同标签 impl 块：它的头单元（`impl` 到 `{` 的那几个词，连属性）跟着被排掉，测试一看不出——它把显存记账里够到的单元取成 `BTreeSet<String>` 再比（「let ledger_units_reached_following_everything: std::collections::BTreeSet<String> =」，`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-verification/src/crash_identity.rs:1083`），两个同名头并成一个名字。那个块里的方法名字不同、不在登记里，够到了照进；但 `BooksOfACard` 这个 struct 登记了，从来不被点名，R-方法 按「自身类型已被点名」放行固有 impl 的方法这一路对它永远不通，新块里的方法经 `.x(` 够到时进不来——这一种测试一抓得到（不带登记那一趟把它列进显存记账、名字不在登记里，红）。剩下抓不到的只有那个头单元自己的词，影响是 impl 头上的属性改了不变键（推的）。

### 三条测试各证不证得了它的文档句

| 测试 | 文档句要证的 | 证得了吗 | 依据与限度 |
|---|---|---|---|
| `the_gpu_judge_group_reaches_no_unit_of_the_memory_ledger_beyond_the_listed_ones`（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-verification/src/crash_identity.rs:1078`） | 不带登记收时显存记账里进这一组的单元恰好是登记的那几个；带登记收时显存记账一个不在、真够到的都在登记里 | 证得了（推的） | 主断言按限定名集合比，陈旧登记、新进来的单元都红。第二个断言（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-verification/src/crash_identity.rs:1098` 那一行，引文见表下）后半句按构造恒真（只有登记的名字会被记进「够到又没沿它收」），有判别力的是前半句「不空」；它与第三个断言（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-verification/src/crash_identity.rs:1106`）一起抓的是「排除没生效」这种实现错，不是登记对不对。同名两个单元并成一个名字（上一节） |
| `not_following_the_listed_ledger_units_drops_no_unit_outside_the_memory_ledger`（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-verification/src/crash_identity.rs:1115`） | 不沿登记收之后，显存记账之外少掉的只有 `mod gpu_memory_ledger;` 那一行 | 证得了（推的） | 比的是整行（种类、限定名、位置、指纹），抓得到「经登记的单元才够得着的外面单元」被切掉。限度：放过的那一行「["mod singlefs_verification::gpu_memory_ledger"],」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-verification/src/crash_identity.rs:1133`）从此不进 GPU 判器组的键——`crates/singlefs-verification/src/lib.rs` 第 53 行 `pub mod gpu_memory_ledger;` 上加属性不再变这一把键；加 `#[path = …]` 换文件会让测试一红（文件过滤对不上），加 cfg 让模块不编进来时判器自己编不过（推的） |
| `an_edit_of_a_listed_unit_of_the_memory_ledger_keeps_the_gpu_judge_key`（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-verification/src/crash_identity.rs:1141`） | 改登记了的一个单元，GPU 判器组的键与镜像判定那一把都不变 | 证得了，只证那一处 | 只改 `states_in_one_dispatch_capped_by_the_state_count` 一处（原文在 `gpu_memory_ledger.rs` 里恰好一次，`grep -c` 量得 1）；登记清空时它红（那个单元回到闭包），所以它罩得住「排除没接上」。它不证改了不改判定字（文档句也没这么说） |

表下引文（整行）：「.all(|unit| listed.contains(unit)),」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-verification/src/crash_identity.rs:1098`）。

三条都只管「键里有没有这些单元」，没有一条管「这些单元的行为不改判定字」——那一句由下一节回答。

### 「显存记账的行为不改判定结局」：做得成的会红检查

判定：**做得成一部分**（两块写法，全部是推的、没实现没跑）；剩下一块做不成，理由在后面。思路是把这一句按第一轮攻方分的四类拆开，每一类找出它能碰到结论的那唯一一条路，在那条路上立会红的断言。

1. **宽度封顶（`states_in_one_dispatch_capped_by_the_state_count`）**：它交出的只是一个宽度，判器按宽度开草稿区、切派活。不改判定字 ⇔ 判器的结论与派活宽度无关（「切法不进结论」）。今天有一条两档宽度的对拍：「fn verdicts_are_the_same_whether_one_dispatch_spans_one_scratch_lane_or_several() {」（`/home/fy5090/code/singlefs/crates/singlefs-verification/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:4595`），只比 256 与三条草稿缓冲两档。
   - 改哪份文件：同一个测试文件，加一条（或把这一条扩成）宽度扫描：同一条流、同一张卡，`GpuCrashJudge::on_card_with_packed_tables(card, &packed, w)` 依次取 w ∈ {1, 2, 3, 7, 255, 状态数 − 1, 状态数, 状态数 + 5}，每一档 `judge_states(0..状态数)`。
   - 断言什么：（甲）每一档逐状态的结论字与 w = 状态数那一档相同，也与 CPU 参照判器相同（用这份文件已有的逐状态比法）；（乙）每一档的 `states_per_dispatch()` 等于 `min(w, 状态数, 显存装得下的)` 再取不小于 1——证判器真按这一档开了宽度、不是全被显存压成同一档；（丙）这一条拿不到卡、显存不够时照文件里已有的写法报本次未跑，但打一行 `GPU_WIDTH_SWEEP judged_states=<判了的状态数>`，变异判法读到 0 或读不到这一行按「没判」算，不按「相同」算（第一轮攻方说的那个判别子）。
   - 什么输入会红：宽度不整除状态数时尾巴那一次派活的序号或回读位置错一位的判器（例：`crates/singlefs-verification/src/crash_judge_dispatch.rs` 里 `judge_ordinals` 那一段把最后一次不满宽的派活当满宽回读）；w = 1 那一档对每个状态各派一次，最容易暴露「草稿区下标按派活内序号算错」这一类。今天两档宽度都是 256 的倍数附近，这几类在那两档上不一定红（推的）。
   - 罩的范围：显存记账改这个函数只能改宽度（返回 0 的由 `.max(1)` 与断言挡、返回比显存大的由建缓冲报本次未跑），只要判器在这一串宽度上与宽度无关，这个函数怎么改都改不了结论字。
2. **编内核的透传（`compile_for_the_work_of_this_thread`）**：签名是「pub fn compile_for_the_work_of_this_thread<Output>(」（`/home/fy5090/code/singlefs/crates/singlefs-verification/src/gpu_memory_ledger.rs:2254`）加 `compile: impl FnOnce() -> Output` 交 `Output`。`Output` 不带任何约束、`compile` 只能调一次、这份文件里没有 `unsafe`（`grep -c unsafe` 量得 0），所以它要么交 `compile()` 的结果、要么不返回（参数化）。
   - 改哪份文件：`gpu_memory_ledger.rs` 自己的测试模块，加一条只编不跑的钉签名测试：定义一个不实现任何 trait、带借用生命周期的标记类型 `struct OnlyFromTheClosure<'a>(&'a u8);`，把这个方法赋给 `fn(&DeviceConditionOfACard, fn() -> OnlyFromTheClosure<'_>) -> OnlyFromTheClosure<'_>` 这样的函数指针（具体写法由实现员照编译器调）。
   - 断言什么：编得过就是断言成立；另加一条源码断言「这份文件里没有 `unsafe`」（读自己的源码数）。
   - 什么输入会红：给 `Output` 加任何约束（`Clone`、`'static`、`Default`、`Send`）——标记类型都不满足，编不过；把返回类型改成别的（`Option<Output>`、缓存里的管线）——函数指针类型对不上，编不过；在文件里写 `unsafe` 造一个 `Output`（`mem::zeroed`、`transmute`）——源码断言红。
3. **借据、总线计数与 `Drop`**：它们只决定后面借不借得到卡、借多少，落到判器上就是第 1 条的宽度或「本次未跑」，由第 1 条的（乙）（丙）罩着，不另立。
4. **设备丢失与显存不够的记账、标记行**：它们只改记不记、打不打那一行，判器这一次照样 panic（第一轮攻方读到的两处 `panic!("{failure}");`）。会红的写法要在真卡上造一次设备丢失，看判器不交结论字——**做不成**：wgpu 没有可移植的「让这个设备现在丢失」的入口，真卡上造不出可重复的设备丢失；软件后端上丢失的路径与真卡不同，量出来不算数（推的，没在本仓验）。退一步能做的只是结构断言（这几个函数的返回类型是 `()` 或 `bool`、调用处紧跟 `panic!`），那是读源码，不是会红的行为检查，不建议当它证过。
   另一个做不成的面：块键不分卡型、额度与卡数（`.claude/kb/decisions/13-验证路线.md` 第 440 行，已定项 17 的射程段），记账里按卡型或额度才走到的分支，第 1 条只在跑它的那台机器、那几张卡上判，另一台的卡型下才改结论的记账改动抓不到；要罩它得在两台各跑一次第 1 条，并把卡型写进 `GPU_WIDTH_SWEEP` 那一行，靠人对两台的读数。

这几条只在我的推演上过、被攻过零轮；修的是哪一格：第 1 条修「宽度封顶」与「借据」两类，第 2 条修「编内核透传」一类，第 4 类没有修。

## 第四条：收严二会不会把闭包放大到失去意义

判定：**待量**（命令写好，没跑）。收严一只会让闭包变小（多一个「等」的条件），收严二只会让闭包变大（多开 trait），两者在同一趟里一起起作用，净变化要量；GPU 判器组已量过一次（背景材料第二节，任务 15787：Rust 1361 → 1063），别的组没量。

量法在末节第 1 条那一趟的最后一步（`tools/measure_closure_sizes.py`）：同一棵树（仓根）、两份 unit-graph（主工作区编的是今天的规则，verify-copy 编的是收严后的规则），对下面每一项各收一趟闭包（都给 `--toolchain-line 'host: x86_64-unknown-linux-gnu' --judgment-kind product-behavior`，不给依赖行；`unit-graph closure` 不认 GPU 判器组的「不沿它收」登记，量的是收严一、二本身）：

- 入口登记表九组（`crates/singlefs-verification/src/judging_units_entries.rs` 的 `JUDGING_UNITS_ENTRY_GROUPS`，入口照抄进脚本），都加 `--cfg 'feature="gpu"'`（照 gpu 构建收，两个对拍组照各自的测试目标）；
- 按条复用的测试函数入口：`.claude/gate.d/stage-inputs.tsv` 里 `model-differential-and-scenarios.sh` 那一行点名的 `random_histories` 六个测试函数，各收一趟（`--target test:random_histories`）；
- 七个观察者入口（同一份登记表观察者那几行，各自的测试目标）。

读法：每一项三行，`name=<项> rule=today rc=… rust=… wgsl=… data=… refused=…`、同形的 `rule=verify_copy` 一行、`name=<项> rust_only_in_verify_copy=<个> rust_only_in_today=<个> top_files_only_in_verify_copy=<文件:个,…> top_files_only_in_today=<…>`；末行 `name=summary measured=22 broken_runs=<n>`，`broken_runs=0` 才算每一趟都跑成（退 0 或 3，3 是有拒算、照列）。两边的组成清单各落在运行目录的 `sizes/<项>.<规则>.txt`。

跑前写下的翻面观测（推的）：某一项 `rust_only_in_verify_copy` 明显大于 `rust_only_in_today`、多出来的集中在与那一项无关的模块（例：判器组多出挂载、故障注入、实验装置那几份文件），就是收严二把闭包放大了；收严二多开的只会是入口签名点名的类型（沿字段）与入口所在 impl 头上的约束能到的 trait，登记的入口签名多是数据类型加 `dyn Fn` 回调（第一轮攻方读过），我预期大多数项 `rust_only_in_verify_copy` 是 0 或个位数——量出来上百个就推翻这个预期。

## 没打中的形状（汇总）

①②③ 的逐形表在第一条「试过、没打中的形」；④ 背景材料点名的另三处在第二条表下一段；⑤ 的同名单元在第三条第一节。取样范围：① 十四形、② 六形、③ 九形（都是读代码推的，没有夹具），真仓 grep 五种（第一条「真仓里找过什么」），显存记账 52 个登记名与那份文件里的 impl 块、`#[cfg` 全数（量的）。

## 这条腿自己的限度

- 一个数都没跑：单元图、两份 `lib_function_units.py`、rustc、夹具一次都没起。七个打中形、三个不等价形、对照的预期全是读代码推的；夹具也没编过，某一个写错 Rust（最可能是 `(Store)` 带括号的约束与高阶约束那两个），`judge.py` 在那一格写 `value error=…`、照跑别的格、最后退 2。主 agent 跑出来与 `out/expected.txt` 不同的格，按跑出来的算。
- Python 一侧经我自己写的 `tools/python_closure.py` 找入口单元（按 fn 名与所在 trait 或 impl 自身类型认），不是门禁 code-tooling 两份对拍格那一套换算；它认错入口时报「命中 0 个或几个」退 2，不会静默认错。
- 「今天」那两臂用的是主工作区此刻的 `crates/singlefs-unit-graph/` 与 `scripts/gate/lib_function_units.py`；后一份别的会话在改（第一轮判决第一节说过），跑的那一刻是什么就是什么。
- 真仓没扫「cfg 两臂一臂 struct、一臂别名或改名 use」；真仓里有没有原始标识符写的类型名、有没有不属于任何目标的 `.rs` 也没扫（第二条那三格在真仓上会不会让对拍格红，没核）。
- 第三条第三节的两块会红检查只在纸上推过，被攻过零轮；第 1 块的「什么输入会红」举的判器缺陷是假想的，不是我在 `judge_ordinals` 里读到的缺陷。
- 「跨边界的值算不算入口行为」、四形在全具体 impl 上今天就漏，这两处的定性交主 agent。

## 要主 agent 跑的命令

1. 跑模型（编两份 unit-graph、在夹具上比四种规则、编夹具看值、量两份规则下的闭包大小）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r2-opus -- bash /home/fy5090/code/singlefs/research/prompts/m3-close-unit-graph-wrapper-impls-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy /home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r2-opus/model-run /home/fy5090/code/singlefs/research/prompts/m3-close-unit-graph-wrapper-impls-r2-opus-model`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r2-opus`（命令里的路径都是绝对路径；判决复跑时照规则把模型目录拷进主 agent 自己的草稿目录，最后一个参数换成拷贝，运行目录换成那个草稿目录下一个以 `-run` 收尾的名字）。
   - 看哪几行：末行 `TASK_SERVER_RESULT` 的 `exit=`：0 是夹具每一行都与 `out/expected.txt` 相同且量闭包每一趟都跑成；1 是夹具有格与预期不同（`task-server/task log <号> --all` 里 `  ✗ 与 expected 不同` 那几行点名）；2 是有格跑不成（`error=` 那几行，或 `name=summary … failed_steps=<n>` 不为 0，或量闭包 `broken_runs` 不为 0）。夹具读数是 `name=<夹具> rule=<today|verify_copy|python_today|python_verify_copy> impl_in_closure=… key_changes_with_the_edit=…` 与 `name=<夹具> value_before=… value_after=…`，汇总 `name=summary lines=55 mismatches=<n> failed_steps=<n>`；量闭包的读数照第四条「读法」；`name=binaries today_sha256=… verify_copy_sha256=…` 记两份二进制；`name=rerun judge_exit=… measure_exit=…` 是两步各自的退出码。
   - 跑完：运行目录 `/home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r2-opus/model-run` 里有两份 unit-graph 的拷贝（`bin/`）与夹具的 rustc 编出物（`build-original/`、`build-edited/`），第 2、3 条用完之后由主 agent 删；两次 `cargo build` 的编译目录照服务 `task-server/task clean <号>`。
2. 单独复现第二条第一格的 Rust 一侧（第 1 条跑完之后、运行目录还在时）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r2-opus/model-run -- /home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r2-opus/model-run/bin/unit-graph-verify-copy closure /home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r2-opus/model-run/fixture-original variant_imported_through_a_raw_identifier_path::entry --toolchain-line 'host: x86_64-unknown-linux-gnu'`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r2-opus/model-run`。
   - 看哪几行：`exit=0`；`task-server/task log <号>` 里有一行以 `fn` 起头、第三段是 `crates/variant_imported_through_a_raw_identifier_path/src/lib.rs:<起>-<止>` 且起止盖住第 24 行（预期有，推的）。
3. 单独复现第二条第一格的 Python 一侧：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r2-opus/model-run -- python3 /home/fy5090/code/singlefs/research/prompts/m3-close-unit-graph-wrapper-impls-r2-opus-model/tools/python_closure.py --library /home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/scripts/gate --root /home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r2-opus/model-run/fixture-original --file crates/variant_imported_through_a_raw_identifier_path/src/lib.rs --path entry --edit-line 24`
   - 在哪起：同上。
   - 看哪几行：`exit=0`；日志里 `holding=0`（预期，推的；与第 2 条的「有一行盖住」对不上就是两份闭包不同）与 `key=<十六进制>`。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；没往统一任务服务交任务，判决要引的数没复跑（命令在上一节）。
- 副本上量出的数这一腿没有；以后跑出的夹具数是在夹具与 verify-copy（草稿树）上的数，不算入库装置上的数。
- 没建仓副本与编译目录：草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r2-opus` 里这一腿只写了改报告引文路径的替换规格 `fix-paths.json` 与 `cite-check.py` 的输出 `cite-check.txt`；模型目录只有源码与预期。
- 没跑 `unit-graph why` 核真仓上任何一个单元；真仓「没找到反例」只按 grep 与读登记入口的签名下的。
- 第三条第三节的会红检查没写成代码，只给了写法。
