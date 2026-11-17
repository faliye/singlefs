# m3-close-unit-graph-wrapper-impls-r1 攻方腿（Opus）报告（2026-11-13）

立场：找反例。两条改法都按「主 agent 的倾向是错的」去打。前几轮判决：无。禁读清单里的三处没读。

## 复跑

模型目录 `research/prompts/m3-close-unit-graph-wrapper-impls-r1-opus-model/`，冻结副本是改法甲的原型树 `/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/prototype-root`（只读，开头按 `prototype-sha256.txt` 的 23 份 sha256 核它没变）。复跑命令（主 agent 交，见末节「要主 agent 跑的命令」）：

```
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r1-opus -- bash /home/fy5090/code/singlefs/research/prompts/m3-close-unit-graph-wrapper-impls-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/prototype-root /home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r1-opus/fixture-run /home/fy5090/code/singlefs/research/prompts/m3-close-unit-graph-wrapper-impls-r1-opus-model
```

它做的：核原型 sha256 → 拷原型树进运行目录、`cargo build --release` 编 `unit-graph` → `tools/judge.py` 把 11 个夹具 crate 铺成两棵树（原样、按 `edits.tsv` 改了被判 impl 方法里一个乘数的），对 10 个入口各跑两种规则（today：不设环境变量；wrapper_waits_for_its_type：`PROTOTYPE_WRAPPER_IMPLEMENTATIONS_WAIT_FOR_THEIR_TYPE=1`）的 `unit-graph closure`，记被改的那个 impl 方法在不在闭包里、改前改后 `key` 行变不变；再用 `rustc` 把每个夹具编出来跑入口，记改前改后入口交回的值；逐行与 `out/expected.txt`（跑前写下的预期，推的）比，不同退 1。不是崩溃枚举，没有崩溃状态，3b 的状态数上限不适用；估时：原型 release 编译约几分钟（推的），之后 40 次单元图各约 1 秒、40 次 rustc 各不到 1 秒（推的）。

模型文件 sha256（`find . -type f | sort | xargs sha256sum`，在模型目录里跑的原样输出）：

```
ad6b3fab8745276b1fd2246a01ba18c1c9807a5e01468a58b5b03c481c2259e6  edits.tsv
d3a7e25d89fee6c5ee6a9a85e1cbb57644222c360a0a7d17b4460ff7b49e0196  fixture/Cargo.lock.fixture
bf98c29fa4570257c1a40454cddae7578fd755bba9d43ddbe80e9cbddf323e89  fixture/crates/associated_type_name_in_the_self_type/Cargo.toml.fixture
da778c1d3da251a413a1f43f6eae105451391ed37e92d904e038b6b04da86f34  fixture/crates/associated_type_name_in_the_self_type/src/lib.rs.fixture
1c210fc27567f6c00c3603705af54a8d9a466bf4bd5fbb0f37968e1f8b82b245  fixture/crates/crate_with_a_private_cursor/Cargo.toml.fixture
01c774dfe3cf0517e4a2a1d10db7237cb33a387f247f2192145dfa1661d4a1fb  fixture/crates/crate_with_a_private_cursor/src/lib.rs.fixture
9094c5ac5c30b52089377d717b0eb71c974de245d014c91109af124f629c671a  fixture/crates/enum_built_from_a_glob_imported_variant/Cargo.toml.fixture
947fd6dc6b6a9aff1474471943d840bb74c602b6687265a41eb954862e4fe4c2  fixture/crates/enum_built_from_a_glob_imported_variant/src/lib.rs.fixture
f69532df2354376c654d2bb9569c80099ae891ec2d2b6d1db9ca8919f26e4130  fixture/crates/enum_built_from_an_imported_variant/Cargo.toml.fixture
20b818e0a0b798dfb8a5f9236fdad6d18f2e88ff581132e826922e9227ec45b1  fixture/crates/enum_built_from_an_imported_variant/src/lib.rs.fixture
ac20d4610c0d8f97c51452371936033fef84bf45054b265fe977444d51d01739  fixture/crates/external_type_sharing_a_private_type_name/Cargo.toml.fixture
40b61e6039fc1a1341f33f4087790d06aaafed0543be629259e0e49b06112363  fixture/crates/external_type_sharing_a_private_type_name/src/lib.rs.fixture
07db16537380e188dda0cd3a1b3edb249911fdbdc936b7bb00507f2e8a95dcb2  fixture/crates/external_type_sharing_a_type_name_in_another_crate/Cargo.toml.fixture
a670bd92fcd0e9ef37bb203ab0746e8535d7d2f666eab5b69d564e507b885670  fixture/crates/external_type_sharing_a_type_name_in_another_crate/src/lib.rs.fixture
b4cc096728502cc86b2527f3cfa91432d86b8dae881e29e6ccc8d9b701f7998f  fixture/crates/generic_parameter_sharing_a_type_name/Cargo.toml.fixture
da58686dba3741ffdfc2852649a0bb0d911a870d13a1b4f73b3b9aaf1b58f8d2  fixture/crates/generic_parameter_sharing_a_type_name/src/lib.rs.fixture
a13380a414b79b8ff49b33cac22334ac38cec622f4bf9d75b554ea7d908f7a40  fixture/crates/impl_written_on_a_generic_alias/Cargo.toml.fixture
1221a95e137510779a7cd2b652c218a8da27ace417fb75051048ff31b11a2cfd  fixture/crates/impl_written_on_a_generic_alias/src/lib.rs.fixture
5223d157737ff1bc052cd92a4bd97b921838bc338abb65c8652c412352d11e08  fixture/crates/wrapper_built_by_the_caller_of_an_open_entry/Cargo.toml.fixture
40b8dc08167942c992032b1a7f2b4fb93a14aabefeca38a0baf13a3724d95166  fixture/crates/wrapper_built_by_the_caller_of_an_open_entry/src/lib.rs.fixture
e1b5844ab120d95f267f8c400264fadca6da66db658d57b7d4a95dc9834f21c1  fixture/crates/wrapper_named_in_the_entry/Cargo.toml.fixture
4ba95db03e2a77e0e65de53efd781230af9ae60139cb8263f1a2b4f7fd22c71c  fixture/crates/wrapper_named_in_the_entry/src/lib.rs.fixture
51315d3be33910dff86ad140ec9d501abc61502f2331688c7f866c706eef7237  fixture/crates/wrapper_reached_through_a_renamed_reexport/Cargo.toml.fixture
7998e56b38d5436640fc92a6add1d3093b802970ae6c0c26076f827c9ec3ad85  fixture/crates/wrapper_reached_through_a_renamed_reexport/src/lib.rs.fixture
c37d116734b861894c753ca55781fcbeca0a58398438af4328d3c5fe03e3b103  out/expected.txt
b33ce92908cc3fd3b3cc0eadc0d269b68df55936982e26c8b768a7d8faabb9a3  prototype-sha256.txt
0a7c80f00ce7cc9c800d71007e23b79e5446b364f431f338640b465a7f2cdd4b  tools/judge.py
e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b  tools/rerun-from-a-clean-work-directory.sh
e369806ada88096355d238d24cafe504bd852b9b8d9b23b520a24873872418ba  tools/rerun.sh
```

夹具文件带 `.fixture` 后缀（照 `crates/singlefs-unit-graph/tests/fixtures/` 的做法，免得仓里多出 Cargo 包与 `.rs`），`judge.py` 铺进运行目录时去掉后缀。

## 各格判定一览

| 格 | 判定 | 依据一句话 | 量的 / 推的 |
|---|---|---|---|
| 第一条（改法甲）总判 | **打中（夹具），真仓没找到** | 改法甲把「名字是不是被点名」从只管全具体 impl 扩到带泛型参数的 impl；单元图今天点名靠的是「类型定义单元进闭包」，有几种合法 Rust 写法造得出值而定义单元不进闭包，这几种写法上今天靠 blanket 放行兜住，改法甲之后漏 | 推的，待主 agent 跑模型 |
| ① 别名（impl 写在泛型别名上，值照原名造） | 打中（夹具 `impl_written_on_a_generic_alias`） | 别名名字算工作区类型名，没人点名别名 | 推的 |
| ① 引进的 enum 变体造值（use 点名变体、glob） | 打中（夹具两个） | use 单元不往下解名字，变体不是单元，enum 名一处都不被点名 | 推的 |
| ① 改名转出（`pub use … as`）从别的模块引用 | 打中（夹具 `wrapper_reached_through_a_renamed_reexport`） | 别名原名只按同一模块的 use 解 | 推的 |
| ① `impl Trait` 返回、闭包、函数指针、`Default::default()`、推断、`From`/`Into`、迭代器适配器 | 没打中 | 每一种都要在闭包里某个单元的词里写出包装类型的名字（构造、签名、注解、turbofish、字段） | 推的 |
| ① 宏拼出的类型名（`paste!`、`concat_idents!`） | 没打中 | 单元图对这两种拒算（照跑），不是静默少收 | 推的（读的拒算代码与测试） |
| ② 经另一个 crate 的泛型函数把包装类型传回 | 没打中（另成一形归 ①、③） | 那个函数自己点名包装类型，它在闭包里；只有经改名转出时漏（归 ①） | 推的 |
| ③ 闭包内部产生 trait 对象 | 闭包内部造的：同 ①，没有新形；跨入口边界：打中（夹具 `wrapper_built_by_the_caller_of_an_open_entry`） | R-开放只开入口签名上的 trait，开进来的泛型适配器的类型参数由闭包外的调用方定 | 推的；真仓登记的入口签名全是数据类型，没有这一形 |
| ④ 两份实现认「工作区类型名」不一样 | 打中两处口径（夹具 `generic_parameter_sharing_a_type_name`、`associated_type_name_in_the_self_type`、`external_type_sharing_*` 两个） | 原型不去泛型参数（与题面不同）；关联类型 `type Item` 是类型别名单元、名字进集合；按全工作区的裸名认，撞上工作区外类型的同名 | 推的；真仓 11 个 impl 落在泛型参数撞名那一形，被 R-隐式兜住、闭包不变 |
| 第二条（改法乙）52 个单元 | **没打中**：一个都没找到会改崩溃状态判定字的 | 52 个单元分四类：派活宽度封顶（只改切法与草稿区大小）、设备丢失与显存不够的记账（只改记录与 panic 的说法）、借据与总线计数（只改后续借卡）、编内核的透传（参数化，交回的就是 `compile()` 的结果） | 推的（逐个读源码） |
| ⑤ 按模块排除会不会把经它转出来的真依赖切掉 | 今天没有；**模块粒度有隐患** | 同一个模块里有判定结局相关的块序写库 `BlocksStoredInOrder`、卡上判块循环 `judge_blocks_on_one_card`，今天不在 GPU 判器闭包里，将来被这一组够着时按模块排除会静默切掉 | 推的 |
| 改法乙自证第 1 格 | 证不全 | 「仍然逐态相同」分不出「对拍判了且相同」与「对拍报本次未跑、早退」，而本次未跑的判法正好在显存记账里 | 推的 |
| 改法乙自证第 2 格 | 证不了它字面要证的事 | 「切之后的闭包加模块里的单元再往下收等于切之前」对任何一个按单元截断的切法恒成立；只证实现是纯截断，不证切掉的不改判定 | 推的 |
| 改法乙自证第 3 格 | 证得了「网会红」，证不了「环境相关的改动会红」 | 造的那一处改动只在对拍用例的那几条流、那张卡上红；显存记账里的行为本来就按卡、额度、设备丢失分支 | 推的 |

## 第一条：改法甲

### 「被点名」今天到底是什么（核过的代码）

- 类型被点名 = 那个类型的定义单元进了闭包：「self.name_type(&unit.name);」（`crates/singlefs-unit-graph/src/closure.rs:1416`），只在 `unit.kind.is_type()` 那一支调。定义单元进闭包靠 R-裸名、R-路径等按名字解到它。
- R-隐式按自身类型里的每一个名字建索引，泛型参数名也在里面：「for name in &implementation.self_type_names {」（`crates/singlefs-unit-graph/src/workspace.rs:1311`）；`name_type` 从这张索引取：「.implementations_by_self_type_name」（`crates/singlefs-unit-graph/src/closure.rs:2129`）。所以包装类型一旦被点名，它的 trait impl 不论 trait 在不在闭包里都进来，原型里 trait 头那一支「等类型」不会因为次序漏掉（核过，不是攻击点）。
- 自身类型名字是自身类型那一截里的每一个标识符（路径段、关联类型名、泛型参数都算）：「if !is_keyword(name) {」那一套之外，impl 头另有 `names_in`；blanket 的判法「.any(|name| generic_parameters.contains(name));」（`crates/singlefs-unit-graph/src/rust_items.rs:1015`）。
- use 单元不往下解名字：「| UnitKind::ExternCrate => false,」（`crates/singlefs-unit-graph/src/units.rs:134`），use 引进的名字只用来判连不连那条 use；改名的原名只按本模块的 use 解：「for (alias, original) in &imports.aliases {」（`crates/singlefs-unit-graph/src/closure.rs:1547`）。
- impl 与 trait 的体照条目切，关联类型 `type Item = …;` 切成类型别名单元：「self.scan_items(head_end + 1, body_close, &member_context);」（`crates/singlefs-unit-graph/src/rust_items.rs:2347`）、「"type" => UnitKind::TypeAlias,」（`crates/singlefs-unit-graph/src/rust_items.rs:2438`）。
- 原型的「工作区类型名」是全工作区每个类型单元的裸名，不分 crate、不分是不是在链接集里、不分是不是关联类型：「workspace.units.iter().filter(|unit| unit.kind.is_type()).map(|unit| unit.name.clone()).collect()」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/prototype-root/crates/singlefs-unit-graph/src/closure.rs:1298`）；判「等」时自身类型的名字整份拿去比，泛型参数没去掉：「(shape.is_blanket_implementation && !shape.self_type_names.iter().any(|name| self.workspace_type_names.contains(name)))」（同一文件:1996）。
- 带路径根的写法跳过：路径根是 `std`、`core`、`alloc` 或别的 crate 名的，按模块筛候选时不点本目标里的哪一份：「.is_some_and(|root| roots_naming_another_crate.contains(root))」（`crates/singlefs-unit-graph/src/closure.rs:1913`）。

主 agent 的论证「先要有一个 `Wrapper<…>` 的值或类型，它要么由闭包里某个单元点名 `Wrapper`，要么从入口的参数传进来」里有两个缺口：一是「点名」在单元图里不等于「源码里写了一次能解到它的名字」，下面 ① 的三形就是 Rust 里合法、单元图解不到定义单元的写法；二是 R-开放只开入口签名上的 trait，不开被开进来的 impl 的类型参数上的约束（③）。

### 打中的形（每一个都在夹具里；今天收、改法甲原型不收，改那一行入口交回的值从 6 变 9 而键不变，全部是推的，待主 agent 跑 `out/expected.txt` 那几行）

| 夹具 crate | 形 | 被改的那一行（夹具源码） | 许可它的那一句 |
|---|---|---|---|
| `impl_written_on_a_generic_alias` | `impl<Inner: Store> Store for Doubling<Inner>`，`pub type Doubling<Inner> = Wrapper<Inner>;`，入口 `Wrapper(Base).put(3)` | `self.0.put(amount) * 2` | Rust 许 impl 写在别名上；R-隐式按 `Doubling` 建索引，`Wrapper` 被点名不带它；`Doubling` 是工作区类型别名单元，原型等它被点名，没人写它 |
| `enum_built_from_an_imported_variant` | `pub enum Layered<Inner> { Plain(Inner), Doubled(Inner) }`，`use crate::Layered::Doubled;`，入口 `Doubled(Base).put(3)` | `inner.put(amount) * 2` | 变体名不是单元，`Doubled` 解不到任何单元；use 单元不解名字；`Layered` 只在 use 那一行与 impl 体里 |
| `enum_built_from_a_glob_imported_variant` | 同上，`use crate::Layered::*;` | 同上 | 同上 |
| `wrapper_reached_through_a_renamed_reexport` | 根模块 `pub use wrappers::Wrapper as Doubling;`，子模块 `make` 里 `crate::Doubling(crate::Base).put(3)` | `self.0.put(amount) * 2` | 改名的原名只在单元所在模块的 use 里找，`make` 模块没有这条 use |
| `wrapper_built_by_the_caller_of_an_open_entry` | 入口 `pub fn entry(source: &dyn Source) -> u32`，`impl<Inner: Store> Source for Adapter<Inner>` 调 `self.0.put(3)`，闭包外的 `caller` 造 `Adapter(Wrapper(Base))` | `self.0.put(amount) * 2`（`Wrapper` 的 impl） | R-开放开 `Source` 的全部 impl；`Adapter` 的 `Inner` 由调用方定，`Wrapper` 只在调用方里写 |
| `external_type_sharing_a_private_type_name` | `impl<Inner: Copy + Into<u32>> Store for std::cell::Cell<Inner>`，同一 crate 另一个模块有私有 `struct Cell`，入口 `std::cell::Cell::new(1_u8).put(3)` | `value * amount * 2` | 按裸名认工作区类型名；入口写的 `std::cell::Cell` 路径根是 `std`，按模块筛不连那个私有 `Cell` |
| `external_type_sharing_a_type_name_in_another_crate` | `impl<Bytes: AsRef<[u8]>> Store for std::io::Cursor<Bytes>`，另一个不在链接集里的 crate `crate_with_a_private_cursor` 有私有 `struct Cursor` | `.unwrap_or(0) * amount * 2` | 原型的类型名集合取全工作区，包括不链接的 crate 的私有类型 |
| `associated_type_name_in_the_self_type` | `impl<Element: 'static> Store for Vec<Box<dyn Iterator<Item = Element>>>`，另一个模块里 `impl Iterator for Counter { type Item = u32; … }` | `.unwrap_or(0) * amount * 2` | 关联类型是类型别名单元，`Item` 进类型名集合；入口里的 `Item` 按模块筛连不到那个关联类型 |
| `generic_parameter_sharing_a_type_name` | `impl<Reader: Source> Store for Reader`，另一个模块里有私有 `struct Reader` | `self.amount() * amount * 2` | 原型没去泛型参数；题面写「去掉它自己的泛型参数之后」，题面口径下这一形应当照 blanket 放行——这一格打的是原型，不是题面 |
| `wrapper_named_in_the_entry`（对照） | 入口 `Wrapper(Base).put(3)` | `self.0.put(amount) * 2` | 预期两种规则都收、键都变：证判法不是一律判「不在」 |

### 打中之后的四句（照 `.claude/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种`）

1. **分不分辨臂**：分辨。每个夹具上今天的规则收、改法甲不收（对照夹具两种都收）。不过 ①（别名、引进的变体、改名转出）与 ③ 跨边界那一形在**全具体**的类型上今天就漏（`impl Store for Mode` 加 `use Mode::A` 一样不被点名），病根是「点名 = 定义单元进闭包」这一条共用前提；改法甲把它从全具体类型扩到包装类型。判臂只看这一格分辨得出：今天包装 impl 靠 blanket 兜住，改法甲之后不兜。
2. **被判的系统看不看得到**：看得到。单元图收闭包时手里有 use 的原文（`imports`、改名表）、impl 头的原文与泛型参数表、每个类型单元的种类与所在模块；分得出「别名」「关联类型」「泛型参数」「路径根是 std」。原型没用这些。
3. **满足的是判据字面的哪一个分句**：背景材料第五节判据第一条「腿给出一个反例（真仓或夹具里，一段代码的改动会改入口行为、而改法之后不在闭包里）」的「夹具里」那一分句。跨边界那一形（`wrapper_built_by_the_caller_of_an_open_entry`）改的是闭包外调用方造的值的行为，算不算「入口行为」由主 agent 定：入口签名收 `&dyn Source`，入口里实际跑的是 `Wrapper::put`；真仓登记的判器、录入、观察者入口我读过的签名都是数据类型（见下一节），没有这一形。
4. **跑前条款给的改法在打中的格上还中不中**：判据只给两个改法：「收严到反例进不来」与「撤回」。撤回在每一格上都不中（回到今天的 blanket）。收严：下面「改法表」逐格写哪种收严修哪一格；没有一种收严同时修全部九格。

### 真仓里找过什么（量的是源码扫描，单元图没跑）

用草稿脚本 `/home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r1-opus/scan/scan_impls2.py`（照 `rust_items.rs` 的 impl 头切法近似：取 `impl<…>` 的泛型参数、`for` 之后到 `where`/`{` 的自身类型标识符）扫 `crates/` 与 `prune-server/`，输出 `scan_impls2.py.out.json`：`4510 types 62 blanket impls`（量的，近似解析，不是单元图的数）。62 个里：

- 60 个自身类型里有工作区类型名，名字都是工作区里真定义的包装 struct 或 enum，或 `DeviceIdentity`（`Vec<(DeviceIdentity, Device)>` 那两个）；没有一个是别名（①别名形没有）、没有一个的外层是工作区外类型（④同名形没有）、自身类型里没有关联类型名（`Item`、`Output`）。
- 唯一的泛型 enum 是 `UnmountRefused<Device>`（`crates/singlefs-mount-runtime/src/lib.rs:340`，只有 `Debug` 的 impl），用到它的地方都写 `UnmountRefused::Failed` 这类带 enum 名的路径（①变体形没有）。
- 改名引进（`use … as 大写名`）全仓 5 处，没有一处是包装类型（①改名转出形没有）。
- 泛型参数名撞上工作区类型名：11 个 impl 的泛型参数叫 `Reader`，工作区有三个 `Reader`（`crates/singlefs-verification/src/verification_read.rs:64` 等）。原型下它们在 trait 头那一支要多等 `Reader` 被点名，但自身类型里的包装名一被点名，R-隐式照推进来，闭包不变（推的）；照题面去泛型参数的另一份实现（Python 那份）与原型在这 11 个上会不一样，两份实现的对拍格会红——这一处是「发现得了」的分歧，不是假绿。
- 工作区类型名里有与工作区外泛型类型同名的：`Cell`（`prune-server/src/selftest.rs:101`）、`Cursor`（`crates/singlefs-verification/src/crash_facts.rs:1175`）、`Buffer`、`Output`、`Command`、`Arguments`、`Table`、`Pool`、`Reader`（扫描量的）。今天没有 blanket impl 写在 `std::cell::Cell<T>`、`std::io::Cursor<T>` 上，所以没有现成的真仓反例；往后谁写一个，改法甲下它进不了任何闭包。
- 登记的入口签名（`crates/singlefs-verification/src/judging_units_entries.rs` 里 GPU 判器、镜像判定、给定状态、两种核对、录入、观察者拼块各组，加观察者登记表前几行）逐个读过：参数都是数据（`&JudgeFlow`、`&CrashImage`、`&FlowFactsReadBack`、`&[ImageAfterTheObserverToJudge]`），带 `dyn` 的只有 `dyn Fn`/`dyn FnMut` 回调与「pub expansion: &'flow dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,」（`crates/singlefs-verification/src/crash_judge_tables.rs:312`）这类，回调交回的是数据；`CrashImage` 里「pub base: &'base MemoryPool,」（`crates/singlefs-verification/src/memory_pool.rs:462`）是具体类型。所以 ③ 跨边界那一形真仓没有。
- 背景材料第二节量到的四组收缩（GPU 判器组 1361 → 1063、`JudgeTables::words` 569 → 240、建表 974 → 671）里掉的 `fault_injection.rs`、`read_tally.rs`、`mounted_session.rs`、`mount.rs` 单元：这几个入口直接调到的函数里没有一个点名这些包装类型，造这些包装值的函数（挂载、故障注入的流）不在这几个入口的闭包里，我推不出入口执行时会有这些值（推的，没跑 `why`）。

### 改法表（自提的收严只在我的模型上推过、被攻过零轮；每格都是「推的，待主 agent 跑」）

| 收严 | 别名 | 引进的变体 | 改名转出 | 跨边界 | 撞工作区外类型名 | 关联类型名 | 泛型参数名 |
|---|---|---|---|---|---|---|---|
| 收严「只认最外层」：「工作区类型名」只认自身类型**最外层**那个类型构造器的名字，而且照 R-裸名的模块筛能从 impl 所在模块解到一个 struct/enum/union 单元（别名、关联类型、路径根是 std 或别的 crate 的不算），解不到照 blanket 放行 | 修（别名不算，照 blanket） | 不修 | 不修 | 不修 | 修 | 修 | 修 |
| 收严「use 算点名」：use 单元引进的路径里（`use a::Layered::Doubled`、`use a::Layered::*`）倒数第二段起的名字，凡是类型单元，use 进闭包时算被点名；`pub use X as Y` 进闭包时 X 算被点名（不论从哪个模块引用 Y） | 不修 | 修 | 修（要再加：按名字找不到单元的裸名去全目标的改名表里找原名） | 不修 | 不修 | 不修 | 不修 |
| 收严「约束照开」：R-开放开进来的 impl，它的泛型参数上的 trait 约束也照 R-开放开（只开一层还是传递到底要另定） | 不修 | 不修 | 不修 | 修（今天全具体类型那一半也一起修了） | 不修 | 不修 | 不修 |
| 撤回改法甲 | 不中 | 不中 | 不中 | 不中 | 不中 | 不中 | 不中 |

三种收严合起来才把九格都盖住；「只认最外层」单做时 GPU 判器组的收缩还在不在要重量（`FaultInjectingBlockDevice<Inner>` 的最外层是工作区 struct、能解到，仍然等被点名，推的是收缩照旧）。

## 第二条：改法乙（⑤）

### 52 个单元逐个读（清单：`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/evidence/group.gpu_memory_ledger.rs.txt`，`grep -cE '^(why|inside)'` 量得 52）

判「它的行为变了会不会改某个崩溃状态的判定字」，读的范围：清单里每个单元在 `crates/singlefs-verification/src/gpu_memory_ledger.rs` 的行区间，加它在 GPU 判器闭包里的调用处（`crates/singlefs-verification/src/crash_judge_dispatch.rs` 的 `on_card_compiling_these_kernel_sources`、`judge_one_dispatch`、等卡那一段；`crates/singlefs-verification/src/gpu_unit_checks.rs` 的 `GpuCard`、`take_the_readback_after_the_wait_or_record_the_device_lost`、`pipelines_compiled_once_on_this_device`）。四类：

| 类 | 单元（`gpu_memory_ledger.rs` 行） | 在判器里干什么 | 改它会怎样 | 改判定字吗 |
|---|---|---|---|---|
| 派活宽度封顶 | `states_in_one_dispatch_capped_by_the_state_count`（84–94） | 「let states_per_dispatch = states_in_one_dispatch_capped_by_the_state_count(」（`crates/singlefs-verification/src/crash_judge_dispatch.rs:874`）：草稿区开几条、一次派活最多几个状态、宽度调度的上界 | 返回 0：草稿区一条都不开，派活时越界或断言 panic；返回大于显存装得下的：建缓冲时显存不够 panic 或报本次未跑；返回别的正数：只改切法。「assert!(count <= self.states_per_dispatch);」（同一文件:1085）挡着一次派活超出它 | 不改，**前提**是「派活切法不进结论」——与登记表里 `DispatchWidthSchedule` 不登进来是同一个前提（`crates/singlefs-verification/src/judging_units_entries.rs` 文件头那一段），我没另证 |
| 设备丢失与显存不够的记账 | `DeviceConditionOfACard`（2147–2159）及其 impl 头、`record_the_device_lost`（2276–2282）、`record_the_device_lost_telling_the_marker_line`（2293–2306）、`tell_the_device_lost_marker_line_the_first_time`（2308–2325）、`is_lost`（2341–2347）、`CauseOfTheDeviceLoss`、`LossOfTheDevice`、`OutOfMemoryReport`、`WorkOnTheCardIdentifier`、`WORK_ON_THE_CARD_OF_THIS_THREAD`、`GPU_DEVICE_LOST_MARKER`（704）、`THREAD_WITHOUT_A_NAME`（707）、`is_a_line_break`、`text_on_one_line`、`gpu_device_lost_line`、`print_the_device_lost_marker_line`（711–737） | 等卡没等成时记一笔再 panic：「panic!("{failure}");」（`crates/singlefs-verification/src/gpu_unit_checks.rs:412`、`crates/singlefs-verification/src/crash_judge_dispatch.rs:1492`），记不记都 panic | 改的是记不记、记什么、打不打那一行；判器这一次照样 panic，交不出结论 | 不改。`GPU_DEVICE_LOST_MARKER` 改了会改**变异判法**认「卡出事」还是「抓到」，那是变异的判定，不是崩溃状态的判定字，也不归 GPU 判器组的键 |
| 编内核的透传 | `compile_for_the_work_of_this_thread`（2254–2272）、`CompilingForAWork` 及其 `Drop`（2186–2204）、use 75/76/78/79 | `GpuCard::pipelines_compiled_once_on_this_device` 里「.compile_for_the_work_of_this_thread(compile)」（`crates/singlefs-verification/src/gpu_unit_checks.rs:1144`） | 签名 `fn …<Output>(&self, compile: impl FnOnce() -> Output) -> Output`：不改签名就只能交 `compile()` 的结果或 panic、不返回；按源码摘要缓存管线的那一格在 `gpu_unit_checks.rs`，不在这个模块 | 不改（「每卡只编一次拿错管线」那一形的代码不在显存记账里） |
| 借据、总线计数与 `Drop` | `BooksOfACard`（999–1015）与 impl 头、`return_the_leases_kept_after_a_failed_wait_on`（1057–1067）、`release_a_device`（1092–1109）、`GpuMemoryLedger`（1132–1138）与 impl 头、`books_locked`（1170–1172）、`DeviceStartedOnTheBooks`（1817–1821）及 `Drop`（1860–1863）、`BusesWithDevicesStartedOutsideTheBooks` 与 `Default`、`new`、`devices_marked_by_bus_locked`、`take_one_device_off_the_bus`（1987–2051）、`DeviceOutsideTheBooksMarkedOnTheBus` 及 `Drop`、`DeviceNumberOnTheCard`、`CardMemoryReading`、`OccupiedByThisProcess`、`DriverShareOnTheCard`、`CardMemoryReader`、`CardCapacityForThisProcess` | 只因 `GpuCard` 结构体的字段点名这些类型而进闭包；最后一个克隆丢掉时销账、减总线计数 | 改的是后面借卡、借多少显存：最多让后面的判器分到更少或更多的草稿区（又回到「切法不进结论」那个前提）、显存不够报本次未跑、或账坏了断言 panic | 不改 |

结论：一个都没找到会改崩溃状态判定字的单元；读过的范围是上面列的 52 个单元与它们在 GPU 判器闭包里的调用处。推翻它的现象：派活宽度不同时 GPU 内核对同一个状态交回的结论字不同（那时第一类改判定字），或显存记账里出现一条路让设备丢失之后不 panic、接着读回读缓冲。

### 设备丢失之后的重试、每卡只编一次拿错管线、宽度封顶改切法（攻击面 ⑤ 点名的三种）

- 重试：判器里没有重试；等不成就记下、panic，卡池下一次借这张卡时重起设备。重起看 `is_lost`（`DeviceStartedOnDemand`，1898 起），那一段不在 GPU 判器组的闭包里。改坏 `is_lost` 让丢了的设备照用：下一次提交等卡不成，再 panic，不交出结论（推的，依赖 wgpu 在设备丢失后让 poll 或映射报错——这是 wgpu 的行为，没在本仓验）。
- 拿错管线：缓存与按摘要取在 `gpu_unit_checks.rs`，显存记账只有透传，见上表。
- 宽度封顶：见上表第一类，不改判定字的前提是切法不进结论。

### 按模块排除的写法

- 今天：52 个单元往外够到的只有标准库（清单里没有别的模块的单元经它们首次进闭包，按清单读的），排掉之后没有别的模块的真依赖跟着掉。
- 隐患：这个模块不只是记账。它有判定结局相关的代码：判完的块按块序写库「/// 判完的块按块序写库（判定向量的编号按块序登记，与用了几张卡无关）」（`crates/singlefs-verification/src/gpu_memory_ledger.rs:2758`）的 `BlocksStoredInOrder`（2761），与卡上判块的循环 `judge_blocks_on_one_card`（2709）。这两样把错了块号，库里同一把块键下记的就是别的块的结论。它们今天不在 GPU 判器组的闭包里（崩溃放量的调度没登进任何一组），所以今天不是打中；但「按模块登记」意味着将来 GPU 判器组一旦够着它们（比如把派活接回的那一段挪进来），会被这条登记静默排掉，而三格自证里没有一格会在那时报出来（下一节）。按单元登记（列出排的那几个单元名）或按「本模块里现在进闭包的那几个单元」生成登记并在每次算键时比对新进来的单元，才挡得住这一种。

### 背景材料第三节改法乙的三格自证，各证不证得了它要证的事

条款出处：`.claude/kb/decisions/13-验证路线.md:434`「边级排除要配三格自证」（kb 文件自己的行号，现查）。

1. **第 1 格（变异之后对拍仍逐态相同，或红在别的断言）**：证不全。
   - 对拍用例拿不到卡或显存不够时报本次未跑、直接返回：「let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {」（`crates/singlefs-verification/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:2361`），而判「借得到借不到」「报不报本次未跑」的代码正好在显存记账里（变异表里「一张都没借到而空着时放得下的也判红（不报本次未跑）」一类 171 行里有一批就是改这一分支的，`grep -c 'gpu_memory_ledger' crates/mutations.tsv` 量得 171）。一条让每张卡都借不到的变异，对拍用例「没红」，这一格会读成「仍然逐态相同」。判别子要补：每一条变异下对拍真判了的状态数大于 0、且在 GPU 上判的，才算「相同」。
   - 只抽登记了的变异行：171 行里落在 GPU 判器闭包那 52 个单元上的，我只认出一行（「GPU 显存账 草稿区不按流的状态数封顶」，改的是 `.min(state_count_as_usize)`），别的多半改借卡那一半（不在闭包里）。这一格要证「这个模块里的代码不改判定结局」，抽的样本基本不在它要证的那些单元上。
2. **第 2 格（切之后的闭包加这个模块里原来在闭包里的单元，再往下收，等于切之前的闭包）**：证不了它字面要证的「少掉的单元只经这个模块可达」。设切之前的闭包是 C、切掉的单元是 L、切之后是 切后闭包：C 里任何一个单元，从入口到它的路上不经 L 的就在 切后闭包 里，经 L 的就在「从 L 往下收」里，所以「切后闭包 加 L 再往下收 = C」对任何按单元截断的切法都成立（闭包是按名字的全局集合收的，`named_types`、`methods_seen` 这两样也是随 L 一起加回来的，推的结论不变）。它能抓的是实现错（切的时候连带切掉了不经 L 的单元），抓不到「经 L 才够得着的那几个单元里有改判定的」。要证后一样得配一张「L 往下收出来、不在 L 里的单元」清单，逐个判，第 1 格的变异也要落到它们上。
3. **第 3 格（造一处落在被排除模块里、又改了某个崩溃状态判定字的改动，自证要红）**：证得了「网会红」——上一节判定那 52 个单元在不改签名时改不了判定字，所以这一处改动得是加出来的耦合（例如给 `compile_for_the_work_of_this_thread` 的 `Output` 加 `'static` 约束、在里面把管线次序对调），对拍用例会红，这一格成立；证不了「显存记账里依赖环境的改动会红」：记账的分支按卡、额度变量、设备丢失走，对拍只在跑它的那台机器、那几张卡、那几条流上判，块键又不分卡型与额度（`.claude/kb/decisions/13-验证路线.md:440`「显卡驱动、卡型、几张卡怎么分活留作块的属性或跑的参数，不进键」），一处只在另一台的卡型或额度下才改判定的记账改动这一格抓不到。
4. **三格都没管的**：按模块登记的将来新进闭包的单元（上一节的隐患）。

## 没打中的形状

| 攻击面 | 试过的形 | 取样范围 | 为什么没打中 |
|---|---|---|---|
| ① | 返回 `impl Trait` 的函数造包装值 | 读规则推的 | 那个函数体写了包装类型名，函数在闭包里 |
| ① | 闭包、函数指针、元组 struct 构造器当函数传（`.map(Wrapper)`） | 推的 | 构造器名就是类型名 |
| ① | 类型别名被点名、原名没写（`type CountedDevice<Inner> = RecordingBlockDevice<FaultInjectingBlockDevice<Inner>>`，真仓 `crates/singlefs-verification/src/bin/new_pool_file_creation_on_device.rs:560`） | 真仓一处 | 别名单元进闭包、它的词里写着原名，原名照点名；漏的是反方向（impl 写在别名上，已列打中） |
| ① | 关联类型 `type Output = Wrapper<T>` 再经 `Self::Output` 用 | 推的 | 写那一行的 impl 头连成员进闭包时成员（关联类型单元）的词里有 `Wrapper` |
| ① | `Default::default()`、`From`/`Into`、推断、迭代器适配器 | 推的 | 目标类型总要在闭包里某处写出（签名、字段、注解、turbofish），否则编不过 |
| ① | `paste!` / `concat_idents!` 拼类型名 | 读 `crates/singlefs-unit-graph/tests/unit_graph_refusals_name_the_unit_and_the_form.rs` 的 `an_identifier_pasting_macro_is_refused` | 拒算照跑，不假绿 |
| ① | `macro_rules!` 体里写包装类型名 | 读 `follow_bare_names` | 宏体按调用处解、一律全连 |
| ② | 别的 workspace crate 的泛型函数返回 `impl Trait`（包装类型私有） | 推的 | 同一目标里私有的照连 |
| ② | 工作区外 crate 的泛型函数 | 推的 | 它不能造工作区类型的值，只能经 trait 转，转的目标类型要在闭包里写出 |
| ③ | 闭包内部 `Box<dyn Store>` 从具体值转 | 推的 | 具体值的类型在闭包里写出 |
| ③ | `static`/`thread_local!` 里存 `Box<dyn Trait>`，由入口外的代码放进去 | 推的；真仓 `thread_local!` 块 12 处（`grep -rnE '^\s*thread_local!\s*\{'` 量的），里面的 static 没有一个是这几个 trait 的对象 | 形态成立，与跨边界那一形同一类；真仓没有 |
| ④ | 同名类型在两个 crate（真仓 `ReadLoggingBlockDevice` 两份） | 真仓 | 按名字算，点名一份两份都算，只多连 |
| ④ | 条目级宏生成的类型 | 推的 | 原型的类型名集合只取类型单元，宏生成的不在里面，照 blanket，只多连；Python 那份若把宏生成的类型名也算进去，两份会不一样，对拍格会红（发现得了） |
| ④ | `Self` 出现在自身类型里 | 推的 | impl 头里写不了 |
| ⑤ | 52 个单元逐个（见上） | 清单全量 | 见上 |

## 这条腿自己的限度

- 一个数都没跑：单元图、rustc、夹具一次都没起（子 agent 不交任务）。夹具今天收、原型不收、值 6 变 9，全是读代码推的；夹具也没编过，有一处写错 Rust 就编不过（`judge.py` 会退 2 并贴 rustc 的报错）。`out/expected.txt` 是跑前写下的预期，主 agent 跑出来与它不同的格，按跑出来的算。
- 「今天」那一臂用的是原型不设环境变量的那一份，不是主工作区此刻的 `crates/singlefs-unit-graph`；两者差多少只看了背景材料给的 `prototype.diff`（只加了两个环境变量开关下的分支）。
- 原型只做了 Rust 一份；Python 一份（`scripts/gate/lib_function_units.py`）的改法甲还没写，④ 里「两份会不一样」是按题面与原型的差推的。
- 真仓扫描是我自己的近似切法（草稿脚本，`scan/scan_impls2.py`），不是单元图的 `dump`；它漏认的 impl 写法（比如宏生成的 impl）不在 62 个里。
- 改法乙只读了清单里的 52 个单元与它们的调用处；「派活切法不进结论」没另证，判定「不改判定字」以它为前提。
- 跨边界那一形算不算「入口行为」，以及别名、引进的变体、改名转出三形在全具体类型上今天就漏（不分辨改法甲与今天），这两处的定性交主 agent。
- 改法表里三种收严只在纸上推过，被攻过零轮，没在模型上量。

## 要主 agent 跑的命令

1. 跑模型（编原型的 unit-graph、在夹具上比两种规则、编夹具看值）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r1-opus -- bash /home/fy5090/code/singlefs/research/prompts/m3-close-unit-graph-wrapper-impls-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/prototype-root /home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r1-opus/fixture-run /home/fy5090/code/singlefs/research/prompts/m3-close-unit-graph-wrapper-impls-r1-opus-model`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r1-opus`（命令里的路径都是绝对路径；判决复跑时照规则把模型目录拷进主 agent 自己的草稿目录，最后一个参数换成拷贝）。
   - 看哪几行：末行 `TASK_SERVER_RESULT` 的 `exit=0` 是每一行都与 `out/expected.txt` 相同；`exit=1` 是有格不同，`task-server/task log <号> --all` 里 `  ✗ 与 expected 不同` 那几行点名是哪一格；`exit=2` 是某一步跑不成（原型 sha256 对不上、编不过、单元图退非 0 或拒算），看 `✗` 那一行。每格的读数在 `name=<夹具> rule=… impl_in_closure=… key_changes_with_the_edit=…` 与 `name=<夹具> value_before=… value_after=…`，汇总行 `name=summary lines=30 mismatches=<n>`；整份结果另落在运行目录的 `result.txt`。
   - 跑完删运行目录 `/home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r1-opus/fixture-run`（里面有原型源码的拷贝与夹具的编出物），编译目录照服务 `task-server/task clean <号>`。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；没往统一任务服务交任务，判决要引的数没复跑（命令在上一节）。
- 副本上量出的数这一腿没有；以后跑出的也是原型（草稿树）上的数，不算入库装置上的数。
- 没有建仓副本与编译目录：草稿目录里只有扫描脚本与它的输出（`scan/`），交回前 `du -sh` 记大小写进交回。
- 没跑 `unit-graph why` 去核 GPU 判器组掉的那 298 个 Rust 单元每一个在入口执行时够不够得着；第一条「真仓没找到」只按源码扫描与读入口签名下的。
