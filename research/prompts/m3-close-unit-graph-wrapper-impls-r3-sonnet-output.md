# m3-close-unit-graph-wrapper-impls-r3：正推腿报告（Sonnet，2026-11-13）

轮名 m3-close-unit-graph-wrapper-impls-r3，批次 `records/2026-10-18-里程碑三收尾与近期欠账.md` 第 2842–2859 行那一批。立场：正推（「已定」的撑不撑得住、报出的数能不能核）。
方案按 `crates/` 今天的实现与 `verify-copy` 里的改动来谈；`verify-copy` = `/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy`（只读，这一趟没改）。
标记：「量的」= 照点名的任务日志或产物逐字核到；「推的」= 读代码得出、没经任务服务跑；「复核不了」写原因。
没读禁读清单里的任何文件（r3 攻方腿的报告、模型目录、草稿目录）。引 kb 条款时行号都是现查的 kb 文件自己的行号（见下面「键算法的条款」）。

## 开头：模型与复跑

问题 1 里找到的「五条之外值还能怎样到闭包」候选（推的）我写成了十二格夹具，没跑。模型目录 `research/prompts/m3-close-unit-graph-wrapper-impls-r3-sonnet-model/`，复跑命令在最后一节「要主 agent 跑的命令」第 1、2 项。
复跑命令（同最后一节第 1 项，在新 shell 里起，跑完看 `task wait` 末行，退非 0 的件数为 0 算过；运行目录每次先删再跑）：
`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r3-sonnet -- bash /home/fy5090/code/singlefs/research/prompts/m3-close-unit-graph-wrapper-impls-r3-sonnet-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy /home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r3-sonnet-run /home/fy5090/code/singlefs/research/prompts/m3-close-unit-graph-wrapper-impls-r3-sonnet-model`
探针的内存与时间我没量（没跑）；编的是 `singlefs-unit-graph` 一个包的调试档，十二个夹具各收两次闭包，前两轮同形装置在服务上的用时是 86 秒、90 秒量级（任务 15820 的 `seconds=86.0`），这个数拿来当量级参考、不是这套装置的读数。
每个文件的 sha256（写完模型之后算的）：

```
deb1fa8dcd5380ed57478286b4ff21636109f117c251113290577e3f8b8dc6f0  cases.tsv
d3a7e25d89fee6c5ee6a9a85e1cbb57644222c360a0a7d17b4460ff7b49e0196  fixture/Cargo.lock.fixture
7c66ca08bb71660301ff183fd80d3b5beed00eebb55b8cb80e99a842a52246e4  fixture/crates/fganyalias/Cargo.toml.fixture
94efd903af65221418789ee7edcd4a20a29ffc21344696ef48bcbfc4be57fd96  fixture/crates/fganyalias/src/lib.rs.fixture
ef0dda23e514bec7029cfdf11b3ecfe661f316ea7d51aa1a9ce22dc2309cfe39  fixture/crates/fganysplit/Cargo.toml.fixture
0f99f18d0cb101b7f07f4df1b5f76b66b245de91606450e77d2915268f6f975e  fixture/crates/fganysplit/src/lib.rs.fixture
61df3455499728a7ef421ba30a6d5f42aae841903de2df8654d8ac2e9909139c  fixture/crates/fgautofirst/Cargo.toml.fixture
9901cedbf2deb7a6e2be13127af184120ab12a2bc48cc789e25628750e23e61b  fixture/crates/fgautofirst/src/lib.rs.fixture
b66121f9424f94fecb04f471c5f1663ce699a3804943f2925053dd4ac8c6e2ba  fixture/crates/fgfnstatic/Cargo.toml.fixture
7a06ff3c30d110aaf72649c1d615b1a50ae59ddd9870024d7ff00cf9c5b3f4e6  fixture/crates/fgfnstatic/src/lib.rs.fixture
88bc302a414998a867d9391ab313c96a45fa9a946826342119fbd5f66606e61a  fixture/crates/fgitemmacro/Cargo.toml.fixture
f9be09e0f98f96ce7824943e184e76850e8b5b305023d3ed9c0bdf327e15b691  fixture/crates/fgitemmacro/src/lib.rs.fixture
23ef025566247bdc59fae5e933092b8e7dd39f71885456537759642a1d7e6287  fixture/crates/fgpathbound/Cargo.toml.fixture
c19da8ab96757efbef09897303f437975a5a20474e1671d1df13f71499c5f2ef  fixture/crates/fgpathbound/src/lib.rs.fixture
ff903c651ccaf1d06da01fe58bcb2c805f32fb6c9aeec5179b0a5db3a617c9c2  fixture/crates/fgpathdyn/Cargo.toml.fixture
a7e2720d3973e72eed80be6268e1aa2ba963a443c771c934be7464f27f80136c  fixture/crates/fgpathdyn/src/lib.rs.fixture
dfd1250d06fc06528beb80515871c70d1530c675a03503c254270a753ef98e82  fixture/crates/fgstaticalias/Cargo.toml.fixture
5519fedd05bf3bd99cbfb116e6e355fe6d4eb9fb7ffd6ee36dcab757bbb12bf9  fixture/crates/fgstaticalias/src/lib.rs.fixture
74dabe609e71e1ebdace159f8c4e9b37d0005aef99e50617ab69db5c3a9fdd64  fixture/crates/fgstaticdirect/Cargo.toml.fixture
06924a737ac846ce2a9b4677da6a9a2cba462c8dc2c739110d126babcc01d89f  fixture/crates/fgstaticdirect/src/lib.rs.fixture
a67c65fb8b4499e674413e45604946d20a67349938e9181fb09819bfe09047bb  fixture/crates/fgstructstatic/Cargo.toml.fixture
7bcc73d0d7a65f20662165c62cf9c80de51340b0253213489922f0fe4325df8a  fixture/crates/fgstructstatic/src/lib.rs.fixture
f9304b12c243983149dd79fb2dfb5a6810681fd55ce04d8ac7f344d829b05c61  fixture/crates/fgtwodown/Cargo.toml.fixture
7538c445174e18e4d04bf43375e85dd47651adc65caa0c3e1642b5f236178a23  fixture/crates/fgtwodown/src/lib.rs.fixture
4d0bed600c3da0d87be60a92fc7a7853294a8de850bbd83a80e94ffba305b510  fixture/crates/fgusebound/Cargo.toml.fixture
6636670ee8191230b48e92347fddfb2bea7b13d1b72ab24c53166011625b5895  fixture/crates/fgusebound/src/lib.rs.fixture
0450e62ff2bfa4a3366202d908f1fb3766a70f229b63ee4476313ea046a11074  out/expected.txt
cb25ac986418190b898cc32eb560a858889d73c2903649a4cccadcb5802895c5  tools/compare_two_implementations.py
e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b  tools/rerun-from-a-clean-work-directory.sh
f9b90733d785843463b9a6ab08d81c38cf16c154da28cac26489a8cee2c9ce2c  tools/rerun.sh
```

模型的做法：每个夹具单独放进一个根，Rust 一份（`unit-graph closure`）与 Python 一份（`lib_function_units.py closure`）各收一次闭包，数「被看的 impl 方法」（等包装类型的那一类 impl，包装类型在入口闭包里没人点名）进没进闭包；`out/expected.txt` 里的 0、1 是我读代码推的预期，没跑，主 agent 跑出来以实测为准。
`rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷来，只换了 `rerun_steps` 那一段（`# 自证` 起的尾部、前 76 行与模板逐行相同，已核）。`--selftest` 我没跑（第 2 项）。

## 一句话结论（先给）

- 问题 1：五条等待条件里四条各挡得住一种「写出名字而解不到这个类型单元」，成立；第二条（自身类型写成路径）作为独立条件不成立——它永远被第三条第一句先拦，十二行变异里它那一行没红（量的）。五条合起来之后，六处开放之外有四类值不经点名就到闭包的口子没有被任何一处开（推的，夹具已写、没跑；真仓里没找到实例）。
- 问题 2：两份判决引的数，日志里有整行的都核到了；对不上的两处：第一轮判决「测试任务 15776 排队中」（实际编译失败退 101）、第二轮判决「12 行变异的证红任务 15923 排队中」（15923 一条都没跑，结局在 15952）；核不到的三处：「一小时」「与原型同样从 1361 降到 1063」里的「原型」那一臂、「15820 是再收严之前的副本」；Python 行号 3013、18199 在今天的主工作区是 3017、18225（内容逐字在）。
- 问题 3：十二行变异 11 行红、1 行没红（条二那行，量的）；没有被任何一行罩到的条件分句与开放共九处，外加三处接线；Python 一份的十二处一行都没有。

## 键算法的条款（kb 文件自己的行号，现查）

这一轮判的是 D13（验证路线）已定项 17 的收闭包规则。条款三处跟这一题直接相关，我只当指路用（不在报告里摘改，原文见 kb）：
- `.claude/kb/decisions/13-验证路线.md` 第 431 行：「函数单元从这份判定的入口沿调用与引用收」（`.claude/kb/decisions/13-验证路线.md:431`）；
- 第 432 行把「键过窄（引用的常量、类型、宏漏收）」（`.claude/kb/decisions/13-验证路线.md:432`）列为违反；
- 第 433 行：「不静默少收」（`.claude/kb/decisions/13-验证路线.md:433`）。
问题 1 里「值不经点名到闭包而键不含它的 impl」就是「键过窄」；规则的收缩方向（blanket 一律放行 → 等包装类型被点名）是用「多连不少连」换键的收窄，所以口子的判据是「少连」。

## 问题 1：规则现行的写法逐条重推

读的位置：`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs` 的 `implementation_waits_for_its_wrapper_type`（函数头「fn implementation_waits_for_its_wrapper_type(&self, head: UnitIndex) -> bool {」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2111`）），
`rust_items.rs` 里 `self_type_first_name`、`self_type_path_root`、泛型参数与 `dyn` 的取法，`workspace.rs` 里三张索引，`open_trait` 与 `open_the_traits_reaching_the_entry_from_outside`。
先把「等」的含义钉死：`name_type` 按名字字符串触发（类型单元进闭包时登一个名字，所有「自身类型名字里有它」的 impl 头与成员随之进来），所以一个 impl 等的是「闭包里某个叫这个名字的类型单元」。
「写出名字而解不到这个类型单元」= 值真是这个 impl 的自身类型，却在闭包里没有任何叫这个名字的类型单元进来。

### 1.1 五条（第三条分两句）各挡什么

| 条 | 代码 | 挡的那一种 | 判 | 依据 |
|---|---|---|---|---|
| 一 | 「shape.generic_parameter_names.contains(first_name)」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2121`） | 头一个名字是这个 impl 自己的泛型参数：`impl<Reader: Tr> Tr for &Reader` 里写的 `Reader` 是类型变量，工作区里另有 `struct Reader` 也与它无关 | **成立** | 夹具 `Shadowed` 只有这一条拦路（同模块有 `struct Shadowed`、无 `use`、没改名、没穿过），变异行去掉这一条测试红（任务 15955 一片的计数「抓到 4 条、没红 0 条」里有它）；泛型参数名取的是深度 1 上跟在 `<`、`,`、`const` 后的名字，生命周期是另一种词法单元、不会混进来 |
| 二 | 「if shape.self_type_path_root.is_some()」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2120`） | 自身类型写成路径：`impl<T> Tr for std::cell::Cell<T>`，工作区另有 `struct Cell` | **不成立（作为独立条件）**：目的对，没有独立作用 | `self_type_first_name` 取的是跳过 `dyn mut const impl for where as unsafe` 之后的**头一个名字**（「.map(」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/rust_items.rs:1046`）），路径时就是根段（`std`、`crate`、`elsewhere`），不是最后一段；`self_type_path_root` 恰好是「头一个名字后面紧跟 `::`」时的同一个名字（函数头在 `/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/rust_items.rs:1049`）。所以路径写法下 `type_units_by_name` 里查的是根段，根段与同模块里一个 struct 同名在合法 Rust 里基本不会发生（模块、外部 crate 名与类型同在类型命名空间，同名会冲突或遮住路径；推的），第三条第一句先拦下，第二条从不单独起作用。量的：十二行变异里它那一行「没红（退出码 0）」，点名的测试 `an_implementation_whose_self_type_is_written_as_a_path_is_released_as_blanket` 在改坏的副本上照样绿（合并任务 15964 日志第 85 行，一片 15954 日志第 88 行 `exit=1` 同一结论）。安全方向不变（多连），但夹具里「本模块另有 struct Pathed 也不等」那句说明与代码注释「哪怕工作区里另有类型叫 `Cell`」说的是最后一段，代码取的是根段 |
| 三之一 | 「if type_units_of_the_name_in_the_module_of_the_head.is_empty()」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2146`）加「.all(」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2149`） | 名字在同一份文件、同一段行内模块里**没有**类型单元（类型是 `use` 引进来的、外部的、宏生成的、写在别的文件里的），或有叫这个名字的单元却不个个是 struct、enum、union（type 别名：`impl` 写在别名上、值照原名造；cfg 两臂一臂 struct 一臂别名） | **成立** | 夹具 `Armed` 只有 `.all` 这一句拦（同模块两个类型单元：cfg 不编的 struct 与别名），变异 `.all`→`.any` 红（任务 15955 一片，合并 15964 里「复用哨兵强制现判」又判了一次，同为红）；「至少一个」那半句（`is_empty()`）没有变异行，`Box<Wrapper<…>>`、`Vec<…>` 这类夹具靠它放行，推的：去掉它 `.all` 对空集为真，这些 impl 会变成「等」，对应测试会红，但没有变异行证过 |
| 三之二 | 「name == first_name))」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2166`） | 同一段模块里有 `use` 引进同名（cfg 两臂一臂定义 struct、一臂 `use other::Wrapper`；外部类型同名） | **成立**（只认模块里的 `use` 单元） | 夹具 `Imported` 只有这一条拦路；变异行红（任务 15956 一片）。推不出的一格：条目级宏调用展开出来的 `use`、`extern crate … as 名字` 不是 `Use` 单元，`uses_by_scope` 看不到；写法太偏，没夹具、没真仓实例的证据 |
| 四 | 「.names_renamed_by_a_use」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2124`） | 有 `use a::Wrapper as W`：别的模块按别名用它，原名不出现，这个名字解不到类型单元 | **成立** | 全工作区按名字一张集合（比按模块更保守，多连）；夹具 `Renamed` 只有这一条拦路，变异行红（任务 15956 一片）；函数体里的 `use … as` 不是 `Use` 单元，但那个函数单元自己的词法单元里写了原名，R-裸名照原名解，不靠这一条 |
| 五 | 「.names_a_use_path_passes_through」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2128`） | `use a::Layered::Doubled`、`use a::Layered::*`：变体引进来之后造值不写 `Layered` | **成立** | 全工作区按名字、任何后面接 `::` 的名字（模块名、crate 名也进，多连）；夹具 `Layered` 只有这一条拦路，变异行红（任务 15954 一片） |

「夹具里只有这一条拦路」是对着 `/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/tests/fixtures/blanket_self_types/fselftypes/src/lib.rs.fixture` 逐个 impl 数过的（推的：读夹具；十二行里 11 行点名的测试在改坏的副本上红是量的，计数行见问题 3）。
五条各自撑得住自己那一种，第二条除外。
推翻「第二条没有独立作用」的现象：构造一个自身类型写成路径、根段又恰好与同模块一个 struct 同名的合法 Rust 程序（我推它不存在），或第二条那行变异改坏后哪个夹具的闭包里 impl 方法进了闭包。
推翻「一、三、四、五各挡得住」的现象：十二行变异里它们那几行任何一行没红（15954、15955、15956 的计数行逐片是「抓到 3、4、4」）。

### 1.2 五条都满足、impl 在等的时候，值还有没有不经点名就到得了闭包的路

推法：闭包里跑的代码要拿到一个自身类型是 `Wrapper<…>` 的值，只有两种来路。其一，闭包里某个单元造它：造它要写 `Wrapper`（或经别名、变体、改名转出），写出来的名字按名字字符串进 `name_type`，五条就是这条路上「写了却解不到」的五种；
其二，从闭包外面进来：入口调用方传进来（入口签名、入口所在 impl 头、入口所在 trait 头、入口签名点名的类型的字段、被开进来的 impl 头上的约束都在 `open_the_traits_reaching_the_entry_from_outside` 与 `open_trait` 里开），
或闭包外的代码放进全局状态、经类型擦除取出（static、`Any` 两处）。其余来路我没找到，但下面四类缺口在代码里是真的有口子（推的，夹具已写、没跑）：

| 编号 | 缺口 | 代码依据 | 夹具与预期（推的） | 真仓实例（grep 近似，不是 `unit-graph dump`） |
|---|---|---|---|---|
| 一 | static 与 `Any` 那一支只读这个单元自己词法单元里的 `dyn`，不像入口那样沿类型单元往字段里找：`static REG: Registry`、`type Registered = Mutex<Vec<Box<dyn Tr>>>`、向下转型到别名 `HeldBox`，dyn 都写在另一个类型单元里 | 「if matches!(unit.kind, UnitKind::Static」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:1415`）起的那一支下面只迭代「for trait_name in &unit.references.dyn_trait_names {」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:1423`）；沿字段找的是入口那一支（「type_names_to_look_into.extend(shape.self_type_names.iter().cloned());」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2210`）起的那一段） | `fgstructstatic`、`fgstaticalias`、`fganyalias` 预期被看的 impl 方法不进闭包（0），对照 `fgstaticdirect` 进（1） | 模块级 static 53 个，它们的声明行与随后四行里的 `dyn` 只有一处，是紧跟在某个 static 后面的另一个函数（`payload_text`）的参数，不是 static 的类型；抽 6 个 static 的结构类型（`StateProbe`、`LedgerDiagnostics`、`EffectiveRootDiagnostics`、`DisposableLibraryChosen`、`GpuConfigurationOfTheCaller`、`MultiHostAddresses`）看字段，没有 `dyn`、`Box`。没找到实例 |
| 二 | 擦除与向下转型分在两个单元：只有「提到 `Any`」的那个单元里写的 `dyn` 才开，写 `Box<dyn Tr>` 的调用单元不提 `Any` 就不开 | 「.cmp("Any"))」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:1419`）按单个单元的 `plain_names` 判 | `fganysplit` 预期 0 | 真仓 `downcast`、`downcast_ref`、`downcast_mut` 共 57 处，目标类型只有 `String`、`&str`、`StopInsideASlice` 三种（`grep -rnoE` 统计：29、25、1+2），没有一处向下转型成 `Box<dyn …>`；没找到实例 |
| 三 | 全局状态写在函数体里：`fn registry() -> &'static Mutex<…dyn Tr…> { static LOCAL: OnceLock<…> … }` | `rust_items.rs` 文件头「自由 fn（函数体里的嵌套条目算它的）」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/rust_items.rs:3`）：函数体里的 `static` 不是 `Static` 单元，函数单元里的 `dyn` 不开 | `fgfnstatic` 预期 0 | 缩进的 `static` 69 行（函数体里或行内模块里），类型是 `OnceLock<MemoryPool>`、`AtomicU64` 这类，同一行没有 `dyn`；值只能经那个函数放进去，没找到闭包外放值的实例 |
| 四 | `dyn` 后面只记头一段路径：`dyn Send + Held` 记成 `Send`，`Held` 漏掉；`dyn Held + Send` 没事 | 「if name == "dyn" {」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/rust_items.rs:258`）起那一段循环只在 `::` 上往下走，遇 `+` 就停；Python 一份同（「if name == "dyn":」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/scripts/gate/lib_function_units.py:2338`）起，`while` 只在 `::` 上往下走） | `fgautofirst` 预期 0 | 真仓里 `dyn Send +`、`dyn Sync +` 在 `crates/` 下 3 处，都是注释或测试字符串，没有 static |

没有会推翻前面推法的「别的来路」被我找到，但这是读出来的，不是穷尽：我没做宏展开产生的 impl、`extern crate … as`、`include!` 把类型与 impl 放进不同文件这几种的夹具，写「推不出」。
我核过、不是缺口的几处（免得重查）：
- 带路径的 trait 名（`D: dev::Device`、`&dyn dev::Device`）：`::` 的第二个冒号后面紧跟 `Device`，而取 trait 名的规则是「任何 `impl`、`dyn`、`:`、`+` 后面紧跟的名字」（「"impl" | "dyn" | ":" | "+") {」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2181`）），所以 `Device` 会被取到（还多取一个路径根段 `dev`，多连不少连）。`fgpathbound`、`fgpathdyn` 我写成对照，预期 1。
- 被开进来的 impl 头上的约束与入口接收者类型：开的是 trait 名，每个被开的 impl 头进闭包时头上写着自身类型名（`impl … for Adapter<Inner>` 的 `Adapter`），R-裸名把它解到类型单元、`name_type` 把这个类型的别的 trait impl 一并带进来；所以「入口开了 trait A，等着的 impl 是同一个包装类型的 trait B」这一格不漏（推的，读 `open_trait` 与 `name_type`）。
- 靠链接期登记收值的写法（`inventory`、`linkme`、`ctor`、`lazy_static`）：`Cargo.lock` 里没有这几个包，`crates/` 下条目级宏调用里的 static 只有 `thread_local!` 一种，花括号的 `thread_local!` 在 `rust_items.rs` 里被拆成里面的 `Static` 单元（「"thread_local" if is_braced => {」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/rust_items.rs:2579`）），走的是 `Static` 那一支，不是 `ItemMacroInvocation` 那一支。`ItemMacroInvocation` 那一支仓里没有用户（夹具 `fgitemmacro` 是对照，预期 1）。
- 入口签名点名的类型沿字段找的传递（Outer→Middle→dyn）：代码里有（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs` 第 2241–2247 行），夹具 `fgtwodown` 是对照，预期 1；入库测试只有一层，这一格没有测试罩（见问题 3）。

推翻缺口一至四的现象：`fgstructstatic`、`fgstaticalias`、`fganyalias`、`fganysplit`、`fgfnstatic`、`fgautofirst` 任何一格实测被看的 impl 方法进了闭包（members=1）。推翻「不是缺口」的几处：`fgpathbound`、`fgpathdyn`、`fgitemmacro`、`fgtwodown` 任何一格实测不进（members=0）。
这些缺口是不是「今天就有」：缺口一、二、三、四对应的开放（static、`Any`）都是第二轮零轮改法新加的，在改法之前（blanket 一律放行）这些 impl 会随 trait 头进闭包，所以是这套改法新造的窄口，不是原有的。真仓里没有实例，所以按判据「真仓里今天就有实例的漏连」不触发；按「攻方给出新反例」那一条要由主 agent 复跑坐实后记零轮。

## 问题 2：两份判决引的每一个数，照任务日志逐个核

日志路径 `/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-<号>.attempt-<次>.log`；同一任务有几次尝试的取最后一次（只有 15787 有两次：第一次尝试撞内存上限退 250、输出不算，取第二次）。结局另用 `task-server/task status <号>` 读（只读）。
下面「核到」= 判决那一句里的数或行原文在日志里逐字找到；「对不上」= 句子与日志不符；「核不到」= 日志里没有这个数。

### 2.1 第一轮判决（`research/prompts/m3-close-unit-graph-wrapper-impls-r1-main-verification.md`）

| 判决的哪一句 | 任务号 | 核到的整行（或「核不到」） | 判 |
|---|---|---|---|
| 攻方模型复跑「退 1：30 行里 29 行与腿的预期相同、1 行不同（`name=summary lines=30 mismatches=1`）」 | 15620 | 「name=summary lines=30 mismatches=1」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15620.attempt-1.log:59`）；「第二台那条命令退 1」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15620.attempt-1.log:74`） | 核到 |
| 「关联类型名被当成工作区类型名」那一形原型照收 | 15620 | 「name=associated_type_name_in_the_self_type rule=wrapper_waits_for_its_type impl_in_closure=1 key_changes_with_the_edit=yes」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15620.attempt-1.log:44`） | 核到 |
| 改法甲八组行，例「name=impl_written_on_a_generic_alias rule=wrapper_waits_for_its_type impl_in_closure=0 key_changes_with_the_edit=no」 | 15620 | 同一行原文在「/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15620.attempt-1.log:29」；这一格 `impl_in_closure=0` 的行日志里共 8 组（第 29、32、35、38、41、47、50、53 行），与「八格」相符 | 核到 |
| 「改那一行入口交回的值从 6 变 9 而键不变」 | 15620 | 「name=impl_written_on_a_generic_alias value_before=6 value_after=9」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15620.attempt-1.log:30`） | 核到 |
| 「原交的 15763 排队一小时没起、取消后同一件量测重交」 | 15763、15787 | 15763 没有日志文件；`task-server/task status 15763` 读到「在队里时被取消，没起」、结局 cancelled 退 239，命令是 `v2-measure.sh`；15787 的命令是 `proto-measure.sh`，脚本名不同；「一小时」这个数 status 里没有 | 核到取消与没起；「一小时」核不到；「同一件量测」没核两份脚本的内容 |
| 收严后在攻方十个夹具上每格都收、键都变，例「name=impl_written_on_a_generic_alias rule=wrapper_waits_for_its_type impl_in_closure=1 key_changes_with_the_edit=yes」 | 15787（第二次尝试） | 「name=impl_written_on_a_generic_alias rule=wrapper_waits_for_its_type impl_in_closure=1 key_changes_with_the_edit=yes」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15787.attempt-2.log:33`）；十个夹具名各有 `impl_in_closure=1` 一行 | 核到 |
| 「name=summary lines=30 mismatches=9」 | 15787 | 「name=summary lines=30 mismatches=9」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15787.attempt-2.log:71`） | 核到 |
| 「V2_GPU rc=0 unit-graph：闭包 2932 个单元（Rust 1063、WGSL 1869、数据 0），拒算 0 处」 | 15787 | 「V2_GPU rc=0 unit-graph：闭包 2932 个单元（Rust 1063、WGSL 1869、数据 0），拒算 0 处」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15787.attempt-2.log:73`） | 核到 |
| `fault_injection.rs`、`read_tally.rs`、`mounted_session.rs`、`singlefs-core/src/mount.rs` 各 0 个、`history.rs` 2 个 | 15787 | 「V2_FILE fault_injection.rs 0」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15787.attempt-2.log:74`）、「V2_FILE read_tally.rs 0」（第 75 行）、「V2_FILE mounted_session.rs 0」（第 76 行）、「V2_FILE singlefs-core/src/mount.rs 0」（第 77 行）、「V2_FILE /history.rs 2」（第 78 行） | 核到 |
| 「V2_ONLY_IN_V2 0」 | 15787 | 「V2_ONLY_IN_V2 0」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15787.attempt-2.log:81`） | 核到 |
| 「与原型同样从 1361 降到 1063」 | 15787 | 15787 日志只有 1063；1361 在 15869、15908 的 `rule=today` 行：「name=gpu_judge rule=today rc=0 rust=1361 wgsl=1869 data=0 refused=0」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15908.attempt-1.log:87`）。「原型」那一臂的 1063 我没找到对应日志 | 1063 核到；1361 在别的任务里核到；「与原型同样」核不到 |
| 乙的测试「测试任务 15776 排队中」 | 15776 | 日志退 101，编译失败：「error: could not compile `singlefs-verification` (build script) due to 1 previous error」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15776.attempt-1.log:180`），「exit=101」（第 215 行同一行里） | 对不上：不是「排队中」，也没跑出测试（句子写的是当时的状态，已过时；R2 判决才用 15786） |
| 「第 3013 行『blanket = split is not None and bool(self_types & generics)』、第 18199 行『BLANKET_SHAPES_SELF_TYPES = (…)』」 | 非任务（读主工作区文件） | 今天主工作区该文件里两句都在，行号是 3017 与 18225（`grep -nF` 命中）；副本里第一句在第 3027 行 | 内容核到，行号对不上（文件被别的会话改过，判决自己已声明） |
| 「核了 48 处引文、76 处裸行号，对上 48 处，没判 76 处（其中裸行号没有引文可比 70 处）」 | 非任务（`cite-check.py`） | 今天重跑同一条命令：「核了 48 处引文、76 处裸行号，1 处对不上，76 处没判（其中裸行号没有引文可比 70 处；逐处列在上面）」；那 1 处是 `lib_function_units.py:18198` 的引文，原文今天在第 18224 行（同一个文件漂移） | 48、76、70 核到；「对上 48」今天是 47，差的那一处就是行号漂移 |

### 2.2 第二轮判决（`research/prompts/m3-close-unit-graph-wrapper-impls-r2-main-verification.md`）

| 判决的哪一句 | 任务号 | 核到的整行（或「核不到」） | 判 |
|---|---|---|---|
| 辩方模型复跑「name=summary lines=19 mismatches=0」，任务 15820（再收严之前）与 15909（再收严之后）都退 0 | 15820、15909 | 「name=summary lines=19 mismatches=0」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15820.attempt-1.log:111`）、同一行在 15909（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15909.attempt-1.log:26`）；`task status` 两个都 succeeded 退 0。两份日志 19 行读数逐行相同，且都编的是 `defense-rerun/compare-run/tree`，日志里没有任何一处写明哪一趟是再收严之前的树 | 数核到；「15820 是再收严之前」核不到 |
| 例「name=form11_field_dyn rust_members=1 python_members=1 closures_equal=yes」；形态十一五种、第一轮十个夹具两份都收 | 15820 | 「name=form11_field_dyn rust_members=1 python_members=1 closures_equal=yes」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15820.attempt-1.log:92`）；十个第一轮夹具名各一行 `rust_members=1 python_members=1` | 核到 |
| 全具体类型的 impl 上两形今天就漏：「任务 15820 `status_quo_concrete_enum_variant`、`status_quo_concrete_alias_impl` 两份都 0」 | 15820 | 「name=status_quo_concrete_enum_variant rust_members=0 python_members=0 closures_equal=yes」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15820.attempt-1.log:99`）、「name=status_quo_concrete_alias_impl rust_members=0 python_members=0 closures_equal=yes」（第 100 行） | 核到 |
| 「任务 15838 作废：两份单元图二进制 sha256 相同」 | 15838 | 「name=binaries today_sha256=fe0dc87413f74b2453c68ab6ea323db37aeaad73c8c476ff781bba275e23e10a verify_copy_sha256=fe0dc87413f74b2453c68ab6ea323db37aeaad73c8c476ff781bba275e23e10a」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15838.attempt-1.log:33`） | 核到 |
| 四形打中，例「name=trait_object_held_by_a_static rule=verify_copy impl_in_closure=0 key_changes_with_the_edit=no」，两份都不收、值 6 变 9 | 15869 | 「name=trait_object_held_by_a_static rule=verify_copy impl_in_closure=0 key_changes_with_the_edit=no」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15869.attempt-1.log:63`）；另三形 `trait_object_behind_a_dyn_any_parameter`（第 70 行 python）、`receiver_of_a_provided_trait_method_entry`（第 73 行 rust）、`struct_in_an_inactive_cfg_arm_alias_in_the_active_arm`（第 80 行 python）各 `impl_in_closure=0` | 核到 |
| 另三形（开进来的 impl 字段里的 `dyn`、括号里的约束、高阶约束）两份都收「impl_in_closure=1 key_changes_with_the_edit=yes」 | 15869 | 第 58、85、88 行（rust 与 python 各一，如「name=bound_in_parentheses_on_an_opened_implementation rule=python_verify_copy impl_in_closure=1 key_changes_with_the_edit=yes」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15869.attempt-1.log:85`）） | 核到 |
| 不等价三形：`r#` 路径变体（Rust 收、Python 不收）、自身类型写成 `r#` 名字（Rust 不收、Python 收）、签名类型另在不属于任何目标的文件里有定义（Rust 收、Python 不收） | 15869 | 「name=variant_imported_through_a_raw_identifier_path rule=verify_copy impl_in_closure=1 key_changes_with_the_edit=yes」（第 93 行）对「name=variant_imported_through_a_raw_identifier_path rule=python_verify_copy impl_in_closure=0 key_changes_with_the_edit=no」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15869.attempt-1.log:95`）；自身类型 `r#`：rust 0（第 98 行）、python 1（第 100 行）；不属于目标的文件：rust 1（第 103 行）、python 0（第 105 行） | 核到 |
| 「各登记组 Rust 单元数只降不升，gpu_judge 1361 对 1063、recording 1068 对 762、judgement_of_the_crash_images 3261 对 3023、observer.publish_boundary 580 对 250；两个对拍比法组与 gpu_facts_verifier 不变」 | 15869 | 「name=gpu_judge rule=today rc=0 rust=1361 wgsl=1869 data=0 refused=0」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15869.attempt-1.log:185`）对「name=gpu_judge rule=verify_copy rc=0 rust=1063 wgsl=1869 data=0 refused=0」（第 186 行）；recording 1068 对 762（第 200–201 行）、3023（第 189 行）、250（第 285 行）；「name=gpu_facts_verifier rule=verify_copy rc=0 rust=152 wgsl=0 data=0 refused=0」（第 198 行）；我另按 22 个量到的行逐行比 today 与 verify_copy：19 降 3 平 0 升，wgsl 与数据数两臂都相同、refused 都是 0 | 核到（「只降不升」全 22 行都核了） |
| 「两趟都退 1，因为读数与腿跑前写的预期不同」 | 15869、15908 | 「name=rerun judge_exit=1 measure_exit=0」（15869 第 306 行、`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15908.attempt-1.log:154`）；summary 行 15869 是 mismatches=6、15908 是 mismatches=17 | 核到 |
| 再收严的 A/B：「name=struct_in_an_inactive_cfg_arm_alias_in_the_active_arm rule=verify_copy impl_in_closure=1 key_changes_with_the_edit=yes」，Python 同；`receiver_of_a_provided_trait_method_entry`、`trait_object_held_by_a_static`、`trait_object_behind_a_dyn_any_parameter` 两份都 `impl_in_closure=1` | 15908 | 「name=trait_object_held_by_a_static rule=verify_copy impl_in_closure=1 key_changes_with_the_edit=yes」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15908.attempt-1.log:20`）、provided（第 30 行）、any（第 25 行）、cfg 臂（第 35 行）各 rust 一行（第 20、25、30、35 行）、python 一行（第 22、27、32、37 行） | 核到 |
| 「那三组两份读数相同」（`r#` 变体、`r#` 自身类型、不属于目标的文件） | 15908 | 「name=variant_imported_through_a_raw_identifier_path rule=python_verify_copy impl_in_closure=1 key_changes_with_the_edit=yes」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15908.attempt-1.log:52`）；`self_type_written_as_a_raw_identifier` 两份都 0（第 55、57 行）；`signature_type_also_defined_in_an_unlinked_file` 两份都 0（第 60、62 行） | 核到 |
| 「各组 Rust 单元数与再收严之前相同（gpu_judge 1063、recording 762）」；背景材料里 3165 对 2872、4249 对 3930、632 对 338、152、218 与 2 | 15908 | 「name=gpu_judge rule=verify_copy rc=0 rust=1063 wgsl=1869 data=0 refused=0」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15908.attempt-1.log:88`）、recording 762（第 103 行）、「name=judgement_of_the_given_states rule=verify_copy rc=0 rust=2872 wgsl=0 data=0 refused=0」（第 94 行）、「name=observer_conclusion_composition rule=verify_copy rc=0 rust=3930 wgsl=1869 data=2 refused=0」（第 112 行）、「name=observer.instance_table_sweep rule=verify_copy rc=0 rust=338 wgsl=0 data=0 refused=0」（第 148 行）、152（第 100 行）、218（第 106 行）、2（第 109 行）；我把 15869 与 15908 的 `rust=` 行逐行 `diff`，两份完全相同（22 行 today、22 行 verify_copy） | 核到 |
| 「任务 15786『30 passed; 0 failed』」；改法乙三条测试 | 15786 | 「test result: ok. 30 passed; 0 failed」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15786.attempt-1.log:286`）；三条测试 `the_gpu_judge_group_reaches_no_unit_of_the_memory_ledger_beyond_the_listed_ones`（第 264 行）、`the_judging_units_keys_compiled_in_are_the_ones_computed_from_the_tree`（第 265 行）等都 ok | 核到 |
| 「真仓对拍格任务 15924『dump 比了 55869 个单元（Rust 53639、WGSL 2074、数据 156），逐个相同；登记入口 10 个（stage-inputs.tsv 18 个），闭包与键逐个相同』」 | 15924 | 「两份实现对拍：dump 比了 55869 个单元（Rust 53639、WGSL 2074、数据 156），逐个相同；登记入口 10 个（stage-inputs.tsv 18 个），闭包与键逐个相同」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15924.attempt-1.log:9`）；status 退 0 | 核到。注：这一趟的判定库在副本里（`verify-copy/.git/…`），不是 prune-server 那份，只作这一趟的证据 |
| 「测试任务 15917『29 passed; 0 failed』」 | 15917 | 「test result: ok. 29 passed; 0 failed」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15917.attempt-1.log:61`） | 核到 |
| 「12 行变异的证红任务 15923 排队中，结局出来补在这一行之后」 | 15923 | 日志退 1，一条都没跑：「✗ 12 行在判定库里没有键相同的记录，这一趟不现判（键对不上的行只在分片里判），一条都没跑：」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15923.attempt-1.log:209`）；该走分片的那条是 15952，它的合并（15964）有结局：「计数：抓到 11 条、没红 1 条、无效 0 条、锚点失效 0 行（共 12 条）」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15964.attempt-1.log:79`） | 对不上：15923 不是证红的结局；真正的结局在 15952 与它的子任务，11 红 1 没红 |
| 背景材料「没出结局的：十二行新变异的证红（任务 15952）」 | 15952 | 15952 此刻 finished、command-failed、退 1，分片 15954、15955、15956 与合并 15964 都有日志 | 背景材料的句子已过时，结局见问题 3 |
| 背景材料「任务 15911 四个测试目标绿」「任务 15942 判器键那一模块『31 passed; 0 failed』」 | 15911、15942 | 15911：70、17、7、46 条，各行「test result: ok. 70 passed; 0 failed」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15911.attempt-1.log:81`）等；注意其中 blanket 那个测试目标只有 17 条，是加 12 条之前的。15942：「test result: ok. 31 passed; 0 failed」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15942.attempt-1.log:140`），含 `every_listed_unit_of_the_memory_ledger_names_exactly_one_unit_of_the_workspace` | 核到 |

没核的数（不是任务日志里的数）：R1 判决的「攻方扫了 62 个带泛型参数的 impl」（来自攻方腿自己的草稿脚本，近似切法，报告里就写了「不是单元图的数」）、「正推腿读了 13 条 trait impl」（我只核了 `proto-out` 里 `comm -23` 得到 299 行、其中含 `impl` 的 181 行，与「298 行不含键行」相容，13 条没逐条核）、R1 快照 sha256（读的是开工快照与主工作区的比对，不是日志）。

## 问题 3：十二行变异与 29 条测试

读的位置：`~/.local/share/singlefs/work/m3-close-gpu-judge-closure/wrapper-mutations.tsv`（12 行）、副本 `crates/mutations.tsv` 末尾 12 行（`tail -n 12 … | diff - wrapper-mutations.tsv` 无差别，量的）、
测试文件 `unit_graph_blanket_implementations_are_those_whose_self_type_mentions_a_generic_parameter.rs`（`grep -c '^#\[test\]'` = 29，量的）、夹具 `/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/tests/fixtures/blanket_self_types/fselftypes/src/lib.rs.fixture`（356 行逐行读）。
证红的结局：任务 15923（整道直接跑）一条都没跑（见问题 2）；按分片协议交的 15952 有结局，三个分片与合并的计数行：15954「计数：抓到 3 条、没红 1 条、无效 0 条、锚点失效 0 行（共 4 条）」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15954.attempt-1.log:140`）、15955「计数：抓到 4 条、没红 0 条、无效 0 条、锚点失效 0 行（共 4 条）」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15955.attempt-1.log:39`）、15956 同形（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15956.attempt-1.log:118`）、合并 15964「计数：抓到 11 条、没红 1 条、无效 0 条、锚点失效 0 行（共 12 条）」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15964.attempt-1.log:79`）；基线「没改坏的副本上点名的测试恰好对上一个用例、结果是 ok 的 12 行，基线不绿 0 行」（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-15964.attempt-1.log:80`）。以下「红」都指这几份日志。

### 3.1 十二行各点到哪一处、点名的测试是不是真靠那一条

| 变异行（mutations.tsv 第几行） | 点到的条件分句或开放 | 点名的测试 | 夹具里别的元素会不会让它提前绿 | 结局 |
|---|---|---|---|---|
| 7834 | 条二 `self_type_path_root` | `an_implementation_whose_self_type_is_written_as_a_path_is_released_as_blanket` | **会**：impl 写在 `elsewhere::Pathed<Inner>`，头一个名字是 `elsewhere`，同模块没有叫它的类型单元（条三第一句先拦）；而且同一夹具里别的一行 `use elsewhere::Imported;` 把 `elsewhere` 放进全工作区的「use 路径穿过的名字」集合（条五再拦一次）。三重拦路，去掉条二照绿 | **没红**（等价变异）；15954 一片、合并 15964 第 85 行 |
| 7835 | 条一 泛型参数同名 | `an_implementation_whose_first_self_type_name_is_its_own_generic_parameter_is_released_as_blanket` | 不会：`Shadowed` 同模块有 struct、无 use、没改名、没穿过，只有条一拦 | 红（15955） |
| 7836 | 条四 改过名 | `an_implementation_on_a_type_renamed_by_a_use_is_released_as_blanket` | 不会：`Renamed` 只被 `pub use super::Renamed as Other;` 改名，其余四条都不拦 | 红（15956） |
| 7837 | 条五 路径穿过去 | `an_implementation_on_a_type_a_use_path_passes_through_is_released_as_blanket` | 不会：`Layered` 只被 `pub use super::Layered::Plain;` 穿过 | 红（15954） |
| 7838 | 条三第一句 「个个是 struct、enum、union」 | `an_implementation_on_a_name_that_is_an_alias_in_another_cfg_arm_is_released_as_blanket` | 不会：`Armed` 同模块 cfg 两臂一臂 struct 一臂 type 别名，无 use | 红（15955；合并 15964 里强制现判又红一次） |
| 7839 | 条三第二句 同模块 `use` 引进同名 | `an_implementation_on_a_name_that_a_use_in_the_same_module_brings_in_is_released_as_blanket` | 不会：`Imported` 的 `use elsewhere::Imported;` 在根模块，是唯一拦路者 | 红（15956） |
| 7840 | 开放：被开进来的 impl 头上的约束 | `the_bounds_on_an_implementation_opened_from_the_entry_signature_are_opened_too` | 不会：`Adapter` 的 impl 被入口签名的 `dyn Source` 开进来，`Doubling` 只经它头上的 `Store` 约束才到 | 红（15954） |
| 7841 | 开放：入口所在 impl 头（或 trait 头）上的约束 | `the_bounds_on_the_implementation_head_of_the_entry_are_opened` | 不会：入口是 `Holder::entry_on_a_bounded_holder`，`Faulty` 只经 `impl<Held: Device>` 头上的约束到 | 红（15955） |
| 7842 | 开放：入口签名点名的类型沿字段找 | `a_trait_object_in_a_field_of_a_type_the_entry_signature_names_is_opened` | 不会：`FieldFaulty` 只经 `Session` 字段里的 `Box<dyn FieldDevice>` 到 | 红（15956） |
| 7843 | 开放：入口在 trait 头下时开这个 trait | `the_trait_of_an_entry_that_is_a_provided_method_is_opened` | 不会：`ProvidedWrapper` 只经入口所在 trait `Provided` 被开 | 红（15954） |
| 7844 | 开放：提到 `Any` 的单元里写的 `dyn` | `a_trait_object_written_in_a_unit_mentioning_any_is_opened` | 不会：`Erased` 只经 `entry_downcasting` 里同一个单元的 `dyn Erased` 与 `Any` 到 | 红（15955） |
| 7845 | 开放：`static` 单元里写的 `dyn` | `a_trait_object_written_in_a_static_of_the_closure_is_opened` | 不会：`HeldInAStatic` 只经 `HELD` 的类型到；改坏后保留 `ItemMacroInvocation`，`Static` 这一支没了 | 红（15956） |

「别的元素提前绿」是读夹具逐个 impl 数的拦路者（推的）；红的结局是量的。7834 是唯一没红的一行。

### 3.2 29 条测试按「靠哪一处才绿」分

- 12 条被上面十二行点名（表里第三列）：11 条靠它点名的那一处才绿（改坏它红）；1 条（条二那条）不靠它。
- 10 条是「blanket 形状」：`ForTheParameter`、`ByReference`、`ByMutableReference`、`InABox`、`InAVector`、元组首尾各一、`InASlice`、`InAnArray`、`InAnOptionalBox`。它们断言「trait 头进闭包就带这个 impl」，靠 `is_blanket_implementation` 的形状判法，由更早的变异行（`mutations.tsv` 第 3766、3797、3798、3799、3800 行）点名，不是这次十二行的事；但它们也同时是「条三第一句『至少一个』」的间接守卫：这些 impl 的头一个名字（`Element`、`Box`、`Vec`、`Option`、`u32`）要么是泛型参数、要么同模块没有类型单元，靠 `is_empty()` 放行，没有变异行直接点它。
- 2 条是全具体类型（`Plain` 没被点名 / 被入口点名），靠更早的行（3800、3801）。
- 1 条 `editing_the_method_of_a_slice_implementation_changes_the_key_of_its_caller`：改 blanket impl 的方法体键变。
- 1 条 `an_implementation_for_a_workspace_type_wrapping_its_generic_parameter_waits_for_the_type_to_be_named`：断言 `Wrapper<Inner>` 的 impl 在没点名时不进（等）、`Box<Wrapper<Inner>>` 进（放）；它是「等」这一半唯一的守卫（五条都不满足的反方向），十二行里没有任何一行改成「总是等」或「去掉 `!(…等…)` 的接线」。
- 1 条 `…_is_in_the_closure_once_the_type_is_named`：入口点名 `Wrapper` 后 impl 进。
- 1 条 `an_entry_whose_signature_names_the_trait_takes_the_implementation_for_the_wrapper_nobody_names`：入口签名里写 `dyn OnAWrapper`，靠更早的行（`mutations.tsv` 第 3628 行）。
- 1 条 `with_the_waiting_rule_broken_the_implementation_for_the_wrapper_nobody_names_is_released_as_blanket`：弄坏开关 `implementations-on-workspace-types-wait-for-the-type`，整条规则，不分条。
合计 12 + 10 + 2 + 1 + 1 + 1 + 1 + 1 = 29。

### 3.3 没被任何一行罩到的条件分句与开放

「罩到」= 有一行变异改坏它、点名的测试在改坏的副本上红。下面按这个口径列；「推的」= 读夹具与代码判断没有任何入库测试会因它红，没有任何变异行去试。

| 编号 | 条件分句或开放 | 位置 | 为什么没罩到 | 级别 |
|---|---|---|---|---|
| 条件一 | 条二 `self_type_path_root` | 「if shape.self_type_path_root.is_some()」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2120`） | 有一行（7834），改坏后点名的测试仍绿：等价变异（问题 1 表第二行） | 量的 |
| 条件二 | 条三第一句「至少一个」（没有叫这个名字的类型单元就不等） | 「if type_units_of_the_name_in_the_module_of_the_head.is_empty()」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2146`） | 没有变异行；`.all` 对空集为真，去掉它 `Box<Wrapper<Inner>>`、`Vec<Element>` 这类会变成「等」，应有测试红，但没有行去证 | 推的 |
| 条件三 | 条三第一句的限定「同一份文件」 | 「unit.file == head_unit.file && unit.local_module_path == head_unit.local_module_path」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2143`）的前半 | 夹具里没有「同名类型单元在别的文件」的 impl；`fselftypes` 只有一份 lib.rs，`fslice` 里没有同名；改成不比文件，所有夹具的结果不变 | 推的 |
| 条件四 | 条三第一句的限定「同一段行内模块」 | 同一行的后半 | 夹具里有行内模块 `elsewhere`（里面有 `Pathed`、`Imported`），但这两个 impl 都被别的条件先拦（条三第二句、条二、条五、条三之空集），模块路径比不比都不改结果 | 推的 |
| 条件五 | 条三第二句的限定「同一段模块里的 `use`」（`uses_by_scope` 的键是（文件，行内模块路径）） | 「.get(&(head_unit.file, head_unit.local_module_path.clone()))」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2162`）所在语句（第 2159–2166 行） | 夹具里唯一引进同名的 `use` 就在根模块，键换成「全工作区的 use」所有夹具结果不变 | 推的 |
| 开放一 | 入口所在 impl 的自身类型点名的类型也沿字段找 | 「type_names_to_look_into.extend(shape.self_type_names.iter().cloned());」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2210`） | 十二行里的 7842 改的是内层循环（对每个找到的类型单元取 trait 名），入口签名那一支与这一支共用它；这一支自己的种子（入口在 `impl Session { fn entry(&self) }` 里、`Session` 字段是 `dyn`）没有夹具：夹具里入口所在 impl 的自身类型 `Holder<Held>` 没有 `dyn` 字段 | 推的；r2 辩方腿的 `form11_field_dyn` 量过这一形（任务 15820 第 92 行 `rust_members=1`），但那不是入库测试 |
| 开放二 | 沿字段找的传递（找到的类型单元的 `plain_names` 再往下找） | `/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs` 第 2241–2247 行 | 入库夹具 `Session` 只有一层（`Session` 的字段直接是 `Box<dyn FieldDevice>`）；去掉「再往下找」，7842 那条测试照绿 | 推的（`fgtwodown` 写了两层的对照，没跑） |
| 开放三 | `static` 那一支里的 `ItemMacroInvocation` | 「if matches!(unit.kind, UnitKind::Static」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:1415`）的第二个选项 | 7845 把条件改成只剩 `ItemMacroInvocation`、去掉的是 `Static`；没有一行改成只剩 `Static`，也没有夹具里有「花括号条目级宏调用里写 static」的形态（`thread_local! { … }` 花括号形态被拆成 `Static` 单元，走的是 `Static` 这一支） | 推的（`fgitemmacro` 对照没跑） |
| 开放四 | 取 trait 名的词表 `impl`、`:`、`dyn`、`+` 里的 `impl` 与 `+` 两个 | `/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs` 第 2181 行取词表那一句（`impl`、`dyn`、`:`、`+` 四个词） | 入库夹具的约束只用 `:`（`Inner: Store`、`Held: Device`）与 `dyn`（`Box<dyn FieldDevice>`）；`T: A + B` 的第二个约束、`impl Tr` 这两种写法没有夹具。7840、7841、7842 三行都把整个返回清空，不分词。（r2 攻方腿的括号约束、高阶约束两形不是入库测试） | 推的 |

不算上面九处、但同样没有一行变异点到的接线（列出来给主 agent 取舍）：
- 「等」的判定在三处被用：trait 头那一处（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:1468` 起）有测试守（入口只经泛型约束点到 trait 头）；R-方法那一处与 R-路径那一处（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs` 第 2002 行、第 2069 行的 `is_released_as_a_blanket_implementation`）没有夹具：入库夹具的入口都不调 trait 的方法、不写 `Tr::x` 路径，把这两处改成直接看 `is_blanket_implementation`（绕过「等」）所有入库测试结果不变（推的）。
- `is_released_as_a_blanket_implementation` 里「并且不是等包装类型的那一种」的接线（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs` 第 2090–2099 行）没有变异行；只有弄坏开关那一条测试守它，改成「永远不等」会红的是 `…waits_for_the_type_to_be_named` 那一条，但没有变异行去证。
- 开头两个提前返回（`head_unit.implementation` 为空、`self_type_first_name` 为空，函数头下面的「let Some(first_name) = shape.self_type_first_name.as_ref() else {」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2117`））：对带泛型参数的 trait impl 都走不到（自身类型里有泛型参数就至少有一个名字），是死分支，不值得一行变异。

Python 一份（`scripts/gate/lib_function_units.py` 的 `waits_for_its_wrapper_type`、`names_written_where_a_trait_goes`、static 与 `Any` 那一段）：十二行变异全在 `closure.rs`，一行都不碰 Python；Python 一份的弄坏开关只有整条规则一个（`rust-blanket-workspace-self-types-released`，自证那一格在「("rust-blanket-workspace-self-types-released"」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/scripts/gate/lib_function_units.py:23005`）），没有按条的开关。
所以 Python 一份的五条与六处开放，逐条都没有一行变异或一个弄坏开关罩着；它们靠的是真仓上两份实现对拍（任务 15924，真仓里没有触发这些条件的实例，对拍对条件分句没有分辨力，这是推的）与第二轮辩方、攻方腿的夹具对拍（不在入库装置里）。

主 agent 要核的一条：问题 3 的第一格（7834）是 `verify-copy` 的 `crates/mutations.tsv` 里的一行，合进主工作区时会进门禁，有一条「没红」整格就红（15964 日志「crates-mutation-rows：红（退出 1）」）。处置有两条路（不替主 agent选）：删掉条二的代码与这一行变异；或让条二真起作用（`self_type_first_name` 在路径写法下取最后一段、再配一个夹具：模块里有 `struct Cell` 又有 `impl … for std::cell::Cell<T>`，且同一个夹具文件里没有别的 `use` 把根段放进穿过集合）。

推翻问题 3 的结论的现象：重跑 15952 那一条（命令见最后一节第 3 项），7834 这一行红了，或表里其余 11 行任何一行没红；或在夹具里加一个只被「同一份文件」或「同一段行内模块」那一句拦路的 impl，对应的变异行红了。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 1 条一（头一个名字是 impl 自己的泛型参数） | 成立 | `Shadowed` 夹具只有这条拦路，变异行红 |
| 1 条二（自身类型写成路径） | 不成立（作为独立条件） | 头一个名字取的是路径根段，条三第一句先拦；变异行没红（量的） |
| 1 条三第一句（同模块有类型单元且个个是 struct、enum、union） | 成立 | `Armed` 夹具只有 `.all` 拦路；「至少一个」半句没有变异行（推的） |
| 1 条三第二句（同模块没有 `use` 引进同名） | 成立，一格推不出 | 宏展开出来的 `use`、`extern crate … as` 看不到，没夹具 |
| 1 条四（没有 `use … as` 改过名） | 成立 | 全工作区按名字，多连；`Renamed` 夹具只有这条拦路 |
| 1 条五（没有 `use` 路径穿过它） | 成立 | 全工作区按名字，多连；`Layered` 夹具只有这条拦路 |
| 1 五条合起来后值还有没有不经点名的路 | 推不出，有四类口子（推的） | static 与 `Any` 不沿类型单元找 `dyn`、擦除与转型分单元、函数体里的 static、`dyn Send + Tr`；真仓没找到实例；夹具十二格已写，没跑 |
| 1 带路径的 trait 名、被开进来的 impl 的别的 trait、链接期登记 | 不是缺口 | `::` 第二个冒号后的名字也取；包装类型被点名带全部 impl；仓里没有 `inventory`、`linkme`、`ctor` |
| 2 第一轮判决的数 | 核到，三处例外 | 15776 句子过时（实际编译失败退 101）、「一小时」与「与原型同样」核不到、Python 行号漂移（内容在） |
| 2 第二轮判决的数 | 核到，两处例外 | 15923 不是证红的结局（一条都没跑，15952 才是）、「15820 是再收严之前」日志里看不出 |
| 2 22 个登记组「只降不升」 | 核到 | today 对 verify_copy：19 降 3 平 0 升，15869 与 15908 两趟 `rust=` 行逐行相同 |
| 3 十二行变异 | 11 红、1 没红（量的） | 没红的是条二那行（7834），等价变异，整格红 |
| 3 29 条测试 | 12 条点名、11 条真靠那一处 | 其余 17 条靠更早的行或只守反方向；条二那条被三重拦路遮住 |
| 3 没被罩到的条件分句与开放 | 九处，外加三处接线与 Python 一份整个 | 条二（等价）、条三「至少一个」「同文件」「同模块」、条三第二句的模块键、入口 impl 自身类型的字段种子、字段传递、`ItemMacroInvocation` 一支、取词表的 `impl` 与 `+` |

## 要主 agent 跑的命令

每项一条，`task-server/task submit` 整条；跑完新派同立场的腿（派发提示写「承接：research/prompts/m3-close-unit-graph-wrapper-impls-r3-sonnet-output.md」并给任务号）读结果，或主 agent 自己读 `task-server/task log <号>`。

1. 十二格夹具的 A/B（问题 1.2 的缺口一至四与四个对照；Rust 一份与 Python 一份各收一次闭包）
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r3-sonnet -- bash /home/fy5090/code/singlefs/research/prompts/m3-close-unit-graph-wrapper-impls-r3-sonnet-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy /home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r3-sonnet-run /home/fy5090/code/singlefs/research/prompts/m3-close-unit-graph-wrapper-impls-r3-sonnet-model`
   - 在哪起：`--cwd` 如上（草稿目录）；运行目录 `…-r3-sonnet-run` 在工作根下、名字以 `-run` 收尾，脚本开头先删它再跑。
   - 看哪几行：`TASK_SERVER_RESULT` 的 `type=` 与 `exit=`；日志里 `name=<格名> rust_members=<n> python_members=<n> closures_equal=<yes|no>` 共 12 行，末行 `name=summary lines=12 mismatches=<n>`。`out/expected.txt` 的预期是我推的；`mismatches=0` 表示我推的全对（六个 `gap_` 格 members=0 即缺口成立，六个 `control_` 格 members=1）；有 `✗ 与 expected 不同` 的行就是要回头核的格，以实测为准。`rust_rc=` 或 `python_rc=` 出现说明那一格某一份实现没跑成（夹具解析出错），要读 stderr。
2. `rerun.sh` 自证（模板的 `--selftest`，六格）
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r3-sonnet -- bash /home/fy5090/code/singlefs/research/prompts/m3-close-unit-graph-wrapper-impls-r3-sonnet-model/tools/rerun.sh --selftest`
   - 看哪几行：退出码 0，日志里「✓ 自检通过（查了 6 格）」。
3. 条二那一行变异（7834）独立再证一次（问题 3，主 agent 要引这个数时自己跑）
   - 命令：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 crates-mutation-replay.sh 带 --item; 范围=点名的 1 项; 为什么=核第三轮正推腿报的「条二那一行变异没红」，只判这一行，别的 11 行在上一趟已有结局' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy -- bash .claude/gate.d/crates-mutation-replay.sh --item '单元图 等包装类型：自身类型写成路径的也等' /home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy`
   - 看哪几行：日志里「没红（退出码 0）」那一条是 7834；`计数：抓到 0 条、没红 1 条`；`TASK_SERVER_RESULT` 退 1。15954 与 15964 已是两次独立现判，同一结论。

没有需要跑的整包测试，没有不指定范围的命令。

## 没做什么

- 不判别：没替主 agent 采纳或出判决；没往统一任务服务交任务，没跑任何项目代码（模型、夹具、自证、变异证红都在上面三项里等主 agent 跑）；判决要引的数没复跑，凡「推的」都是读代码得出的。
- 问题 1.2 的四类缺口与四个对照只写了夹具、预期是我推的，没有实测；真仓里有没有实例我用的是 `grep -rnE` 近似，不是 `unit-graph dump`（那是攻方腿的活）。
- 没做的夹具：宏展开产生的 `use`、`extern crate … as` 与「类型和 impl 经 `include!` 放进不同文件」三种对条三第二句与条三第一句的影响（写「推不出」）；Python 一份的五条与六处开放没有逐条夹具，只在问题 3 里按「没有变异行」列。
- 问题 2 里「原型那一臂的 1063」「15763 排队一小时」「13 条 trait impl」「62 个 impl」「R1 快照 sha256」核不到或没核，已逐条写明。
- 没读禁读清单（r3 攻方腿的报告、模型目录、草稿目录）。
