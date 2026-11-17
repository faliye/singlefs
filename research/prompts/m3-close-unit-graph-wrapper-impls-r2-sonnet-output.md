# m3-close-unit-graph-wrapper-impls-r2 辩方腿报告（Sonnet，2026-11-13）

立场：辩方。复核第一轮判决 `research/prompts/m3-close-unit-graph-wrapper-impls-r1-main-verification.md`：它判「打中」的八格与「按打中办」的形态十一，在收严之后是不是真的进不来；第三节三条改法各自够不够得着它声称修的格；Rust 与 Python 两份写没写成一样；替「撤回改法甲」辩一次。
方案按 `crates/` 今天的实现与 `verify-copy` 里的改动来谈。`verify-copy` 根是 `/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy`（只读）；下文引它的文件都写绝对路径。
没读禁读清单里的任何文件。没跑项目代码、没往统一任务服务交任务：所有「跑一下就知道」的都写进末节「要主 agent 跑的命令」；没经主 agent 跑出的读数一律标「推的」（读代码推的）。这一腿自己量的只有 `sha256sum`、行数、`grep` 与 `awk` 的计数、语法检查这几类命令的读数，闭包与键的读数一个都没有；下文凡标「量的」的，要么是这几类命令的读数，要么是转述背景材料或旧产物里的读数并写明出处。

## 先说三条要紧的

1. **「十格都收」那次量测（任务 15787）如果用的是攻方的判法，十个夹具放在同一个根里跑，对其中两格（跨边界、点名包装类型的对照）没有分辨力。**（背景材料没写 15787 的命令；下面是「若它用攻方判法」的推论。）单元图里「有没有 `use … as …` 改过这个名字」「有没有 `use` 路径从这个名字穿过去」「同名类型」三张索引是整个工作区一张：「for (position, unit) in units.iter().enumerate() {」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/workspace.rs:1251`）；攻方判法把十个夹具的 crate 全部铺进一个运行根：「original_root = os.path.join(arguments.work, "fixture-original")」（`/home/fy5090/code/singlefs/research/prompts/m3-close-unit-graph-wrapper-impls-r1-opus-model/tools/judge.py:144`）、「key_before, holding = closure_reading(arguments.binary, original_root, row, rule_environment, edit_lines[row["crate"]])」（`/home/fy5090/code/singlefs/research/prompts/m3-close-unit-graph-wrapper-impls-r1-opus-model/tools/judge.py:151`），其中改名转出那一格写着 `pub use wrappers::Wrapper as Doubling;`，于是「Wrapper」这个名字对别的夹具也进了「改过名」的集合，跨边界那一格里本来该等被点名的 `impl<Inner: Store> Store for Wrapper<Inner>` 就按 blanket 放行了——不靠入口外面传进来那一处开放，它也在闭包里。所以「十格每格都收」不能当作「收严二修了跨边界」的证据（推的，读代码推的）。修了没有，要每个夹具单独一个根再量一遍（末节命令 1，模型已写好）。
2. **收严写进来的五条条件与三处新开放，除 `Box<Wrapper<T>>`（头一个名字是 `Box`）与入口签名里写 `dyn` 两种之外，入库的测试里一格都没有：** 删掉其中任何一条条件、或删掉 `open_trait` 里对约束的递归、或删掉入口所在 impl 头与字段的那一路，新加的四条单元图测试与 Python 一格自证都不会红（只有整条规则的弄坏开关一个）。详见条目 2。
3. 读代码这一步，八格与形态十一在收严之后**都走得进闭包**（条目 1 逐格写了经哪条规则、文件:行）；Rust 与 Python 两份没找到不等价的写法，只有一处文件集合的范围差（条目 3）；撤回改法甲的最强辩护我写了（条目 4），它的结论是「撤回并改用按单元登记」，依赖七个我没核过的前提。

## 判定一览

判定词：站得住 = 判决这一条在收严之后的代码上读下来成立；够不着 = 判决说修了、代码走不到；同样打中别的臂 = 同一个形状对别的方案（今天的规则、撤回改法甲）同样成立。全部是读代码推的，没有这一腿量的闭包或键的读数。

| 判决条目 | 辩方判定 | 证据 |
|---|---|---|
| 八格之一：impl 写在别名上（`impl_written_on_a_generic_alias`） | 站得住（收严后进闭包）；同样打中别的臂（全具体类型的 impl 写在别名上，今天与撤回甲之后同样漏） | 条目 1；模型 `status_quo_concrete_alias_impl`（预期 0） |
| 八格之二、三：变体经 use / glob 引进 | 站得住；同样打中别的臂（全具体枚举同形今天同样漏） | 条目 1；模型 `status_quo_concrete_enum_variant`（预期 0） |
| 八格之四：改名转出 | 站得住；同样打中别的臂（判决第二节第 24 行的说法，我没另设对照格，推的） | 条目 1 |
| 八格之五：跨边界（入口外面造的包装值） | 站得住（靠开放里的递归）；同样打中别的臂（同上）；十格同根的量测对这一格没有分辨力 | 条目 1、开头第 1 条 |
| 八格之六、七、八：撞工作区外类型名两格、泛型参数同名 | 站得住；只中原型（原型整份按名字比，收严后的第二、第一条条件让它们照放行） | 条目 1 |
| 「关联类型名被当成工作区类型名」没打中 | 站得住（头一个名字是 `Vec`，按 blanket 放行） | 条目 1 |
| 形态十一（接收者字段里的 `dyn`、入口所在 impl 头上的约束）按打中办 | 站得住（收严二够得着，推的，没量） | 条目 1；模型 `form11_*` |
| 「缺口不是改法甲新造的」 | 站得住（前四形）；撞名三格是原型新造 | 条目 1 末段 |
| 「真仓里没找到现成反例」（两条腿各一次抽样） | 没复核（我没扫真仓） | 没做什么 |
| 改法甲收严一声称修的七格 | 站得住（够不着的格 0 个）；但五条条件的反面没有入库的会红检查 | 条目 2 |
| 改法甲收严二声称修的两格 | 站得住（够不着的格 0 个）；三处开放没有入库的会红检查 | 条目 2 |
| 改法乙（按单元登记、第二格） | 站得住；第一格没做，判决自己记了欠账 | 条目 2 |
| Rust 与 Python 两份「同一口径」 | 逐句对过，没找到不等价；范围差一处（索引遍历的文件集合）、原始标识符一处；没有同一输入上的比较覆盖条件的反面 | 条目 3；模型 `closures_equal` |
| 撤回改法甲「不选」 | 用户已定；辩方最强理由与七个未核前提见条目 4 | 条目 4 |

## 模型

写了一个小模型，用途两个：把十个攻方夹具与形态十一的夹具各放进自己的根，Rust 与 Python 两份各收一次，比闭包逐行是否相同、「被看的 impl 方法」进没进；再给两个今天就漏的对照格。模型目录 `research/prompts/m3-close-unit-graph-wrapper-impls-r2-sonnet-model/`，没跑过，`out/expected.txt` 是读代码推的预期（「推的」）。
复跑命令（主 agent 交，整条写在末节）：`task-server/task submit --cwd <草稿目录> -- bash <模型目录>/tools/rerun-from-a-clean-work-directory.sh <仓根> <冻结副本> <运行目录> <模型目录>`；冻结副本填 `verify-copy`。
每个文件的 sha256（`sha256sum` 量的，模型目录相对路径）：

```
cdd5f6a48c23f8c0aed8433bab185a7c0a905b23df21c26449ea24c73b29a5e6  ./cases.tsv
d4273af89d87567278617c859155491af80eac9ad2db636339bdcf1efd236512  ./out/expected.txt
41c2d1c48dadbb5dfdd573a3a61bc05ac858ac7ffc1daf66450a0104a45d9fb8  ./tools/compare_two_implementations.py
e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b  ./tools/rerun-from-a-clean-work-directory.sh
d6cbed4e396c98d9a14449a172330ed094a356c44840ccfde556ae19172521a5  ./tools/rerun.sh
d3a7e25d89fee6c5ee6a9a85e1cbb57644222c360a0a7d17b4460ff7b49e0196  ./fixture/Cargo.lock.fixture
ca1c5aa66ae58bcc0e15786d5e8eaef5160378712cbf0c3e4bc8efac762413b5  ./fixture/crates/fcalias/Cargo.toml.fixture
1469fb7d5c5850b7e518a6f9a752a14b783a3d37c29dbb18b494534995307071  ./fixture/crates/fcalias/src/lib.rs.fixture
2193220f80def38438e91daecf8190c07a96fa5a565ca6707ac266b429ddcf70  ./fixture/crates/fcbound/Cargo.toml.fixture
55f2d60c67b0d06d19b851cac7316ac0c220d02405f6d2c7db0c33b8738c8611  ./fixture/crates/fcbound/src/lib.rs.fixture
c7c801cb8f2e2ef2633b73febaa65c9cfd75873376d8dca790e3168c05bfc985  ./fixture/crates/fcconcretealias/Cargo.toml.fixture
eb8907f3794a95726870e126f3bd44206c58a6a0c79ab7ac7fe266ee8718e052  ./fixture/crates/fcconcretealias/src/lib.rs.fixture
f6ad68c628e11052cd50dc8497e4ecd6e28a3e26027b822faf2f18a03cc85445  ./fixture/crates/fcconcreteenum/Cargo.toml.fixture
a264b55e35e32dbff738ed2890c5e179194f17fd5729f64910536c4aae294245  ./fixture/crates/fcconcreteenum/src/lib.rs.fixture
2338500464b5dcbfb54ee3ba0d9080964b225d2248673e8053626fe3ee5de62e  ./fixture/crates/fccontrol/Cargo.toml.fixture
301004a47cbfb9a9395bfd6f76aac5b59c8959599ff83e84d20a48150c146b9b  ./fixture/crates/fccontrol/src/lib.rs.fixture
64b7ab46a1d3c1dd69940c628a0f2929b835b1f08a2cb00cd5ff670125849c80  ./fixture/crates/fcfield/Cargo.toml.fixture
9fc525677a6375bdca8b83ab33935ac21486d5202fe2467f0da3905ddc6e0ced  ./fixture/crates/fcfield/src/lib.rs.fixture
05679d4e2fa8b441df55a23bf04ac65d8d74270e9873505a0b2052f4afdb06dd  ./fixture/crates/fcnested/Cargo.toml.fixture
1b032bd5092259175d6dbb49e69d07b6016ee43f2cd081e36ff12d883a6b56c3  ./fixture/crates/fcnested/src/lib.rs.fixture
4cdb1758610450c57fc05c7a02f765c6399d0ed18c02a085ec4d7e95a3c9515a  ./fixture/crates/fctrait/Cargo.toml.fixture
1c99347d86e98f542d07888c0405b57c194e9c51cdb252eaadf2f6fecf868e2e  ./fixture/crates/fctrait/src/lib.rs.fixture
```

语法检查（量的）：`python3 -m py_compile` 过了 `compare_two_implementations.py`，`bash -n` 过了 `rerun.sh` 与 `rerun-from-a-clean-work-directory.sh`。模型里 `rerun.sh` 的 `--selftest`（模板自带的那一格）因为换了 `rerun_steps` 不再适用，没交、也不需要交。
本腿开工前认领了草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r2-sonnet`，里面没有写任何东西（没有仓副本、编译目录）。

约定：下文「够得着」指读代码走下来，这一格里被看的那个 impl 方法在收严之后的闭包里（改它键会变，因为键含闭包里每个单元的指纹，`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:3438` 一带）；「够不着」指走不到、写断在哪一行；「推不出」指走不完。三个词都是读代码的读数，等模型跑出来再改成量的。

## 条目 1：八格与形态十一，收严之后进不进闭包（逐格走代码）

简写（只在下文用，不带扩展名的 `名:行` 不是引文）：`C` = `/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs`；`W` = 同目录的 `workspace.rs`；`R` = 同目录的 `rust_items.rs`；`P` = `/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/scripts/gate/lib_function_units.py`。夹具是攻方模型目录 `research/prompts/m3-close-unit-graph-wrapper-impls-r1-opus-model/fixture/crates/<格名>/src/lib.rs.fixture`。

### 各格共用的那一段路（读代码）

八个夹具的入口都在函数体里调 `.put(…)`（跨边界那一格调 `.total()`）。R-方法把 `.put(` 连到所有叫 `put` 的成员，trait 里的成员恒放行：「let is_eligible = if container_unit.kind == UnitKind::TraitHead {」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:1983`）；成员连头，trait 头进闭包时带它的 blanket impl：「// trait 头进闭包时它的 blanket impl 连同成员一起进」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:1447`），放不放行由这一句定：「if self.is_released_as_a_blanket_implementation(*head) {」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:1454`）。
放行函数是「自身类型里出现它自己的泛型参数」且「不是等包装类型的」：「.is_some_and(|shape| shape.is_blanket_implementation)」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2080`）与「implementation_waits_for_its_wrapper_type(head))」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2084`）。R-方法与 R-路径两处同样调它（`C:1988`、`C:2055`）。
所以一格要「进」，等价于「它那份 impl 的 `implementation_waits_for_its_wrapper_type` 为假」，或「走 R-开放」，或「自身类型里的名字被点名」。下面逐格看是哪一条。

### 八格

- **别名（`impl_written_on_a_generic_alias`）：够得着。** impl 头 `impl<Inner: Store> Store for Doubling<Inner>`，自身类型头一个名字取 `Doubling`（`R:1040` 的 `self_type_first_name`）。叫 `Doubling` 的类型单元只有一个类型别名，类型索引里别名也在「if unit.kind.is_type() {」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/workspace.rs:1252`）；第五条只认三种：「UnitKind::Struct | UnitKind::Enumeration | UnitKind::Union」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2127`），别名不在其中，等待判假，按 blanket 放行。推翻它的现象：这一格的闭包里没有 `<impl Store for Doubling < Inner >>::put`。
- **引进的变体（`enum_built_from_an_imported_variant`）与 glob 引进（`enum_built_from_a_glob_imported_variant`）：两格都够得着。** `use crate::Layered::Doubled;` 与 `use crate::Layered::*;` 里 `Layered` 后面都紧接 `::`，索引把它收进「路径从它身上穿过去」的集合：「names_a_use_path_passes_through.insert(」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/workspace.rs:1282`）；等待判里它是第四条：「.names_a_use_path_passes_through」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2113`），命中就返回假。入口只写变体名 `Doubled`，变体名不是任何单元的名字（枚举变体不进 `declared_names`），所以枚举 `Layered` 不会从入口那边被点名；它进闭包全靠这份 impl 被放行后 impl 头里的 `Layered` 连过去。
- **改名转出（`wrapper_reached_through_a_renamed_reexport`）：够得着。** `pub use wrappers::Wrapper as Doubling;` 在 use 解析里记一条别名：「imports.aliases.push((self.name_at(index + 1), original));」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/rust_items.rs:1104`），原名进「改过名」的集合：「names_renamed_by_a_use.extend(」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/workspace.rs:1262`），等待判第三条命中（`C:2109`）。
- **撞工作区外类型名两格（`external_type_sharing_a_private_type_name`、`external_type_sharing_a_type_name_in_another_crate`）：两格都够得着。** 自身类型写成 `std::cell::Cell<Inner>`、`std::io::Cursor<Bytes>`，路径头一段是 `std`：「if shape.self_type_path_root.is_some()」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2105`）命中就返回假。第一格还有第二个独立的理由：同名的私有 `Cell` 在 `mod unrelated` 里，与 impl 不在同一段行内模块（`C:2129`）；第二格的同名类型在别的 crate，文件不同（`C:2128`）。
- **泛型参数同名（`generic_parameter_sharing_a_type_name`）：够得着。** `impl<Reader: Source> Store for Reader`：泛型参数在 `R:951` 收，头一个名字 `Reader` 在里面：「|| shape.generic_parameter_names.contains(first_name)」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2106`）。
- **跨边界（`wrapper_built_by_the_caller_of_an_open_entry`）：够得着，而且在单独一个根里只有一条路。** 入口签名 `&dyn Source`，`signature_trait_names` 里有 `Source`：「run.open_trait(trait_name);」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:3365`）；开 `Source` 把 `Adapter<Inner>` 那份 impl 连成员推进来，再对这份 impl 头里 `:` 后面的名字 `Store` 递归开：「self.open_trait(bound_trait_name);」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2273`），`Store` 的全部 impl（含 `Wrapper<Inner>` 那份）进闭包。`Wrapper` 的 impl 与它的 struct 在同一段模块、没人给它改名，单独成根时它等被点名（五条都满足），放行只能靠这条递归。这一格单元图今天的判法（收严前）靠 trait 头带 blanket impl 进来，收严后靠递归进来。
- **对照 `associated_type_name_in_the_self_type`（判决记「没打中」）：站得住。** 自身类型 `Vec<Box<dyn Iterator<Item = Element>>>`，头一个名字 `Vec`，工作区里没有叫 `Vec` 的类型单元，等待判在 `C:2121-2122` 的 `.get(first_name)` 上得假，按 blanket 放行。

### 形态十一（正推腿报告里的形态十一，判决记「推的，没量，按打中办」）

- **接收者字段里的 `dyn`（`fcfield`）：够得着。** 入口 `Session::entry_field_dyn` 在 `impl Session` 里：「run.open_the_traits_reaching_the_entry_from_outside(*entry);」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:3367`）。函数里先取入口所在 impl 头：「for trait_name in self.names_written_where_a_trait_goes(container) {」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2167`），再把这份 impl 的自身类型名 `Session` 当要看的类型：「type_names_to_look_into.extend(shape.self_type_names.iter().cloned());」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2171`）；看 `Session` 这个 struct 单元：「for trait_name in self.names_written_where_a_trait_goes(*type_unit) {」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2199`）取到 `dyn` 后面的 `FieldDev`，`open_trait("FieldDev")`（`C:2200`）把 `FieldFaulty<Inner>` 那份 impl 推进来。
- **入口所在 impl 头上的约束（`fcbound`）：够得着。** `impl<Device: BoundDev> Holder<Device>`，同一个 `C:2167`：impl 头词法单元里 `:` 后面是 `BoundDev`。
- **模型另加三种变体（`fcnested`、`fcalias`、`fctrait`），读代码都够得着：** 字段经两层结构体（`C:2202-2208` 把每个类型单元里的 `plain_names` 接着往下看）；字段类型是写着 `Box<dyn Trait>` 的别名（别名单元也在类型索引里，`W:1252`）；入口是 trait 里的缺省方法、约束写在 trait 头的超 trait 上（`C:2166` 的 `container` 可以是 trait 头）。Python 一份同形：`P:5059-5082`。
- 形态十二（全局量、线程局部、链接期登记）与「值经入口以外的通道进来」不在这一份开放的射程里，判决没有把它们记成修了的格；仓里无实例（正推腿报告读过依赖清单）。这一格我没复核，不算在上面。

### 这一节的判与推翻条件

八格与形态十一在收严之后的代码上读下来都走得进闭包，没有一格走到「够不着」。**推翻它们的现象**：末节命令 1 跑出来，某一格的 `rust_members` 或 `python_members` 是 0（与 `out/expected.txt` 不同）。在那之前这些都是读代码推的。
**同一份判法对「今天就漏」的全具体类型 impl 不起作用**（判决第二节第四行说的那一半）：全具体类型的 impl 不是 blanket（`C:2080`），trait 头不带它，R-方法只在自身类型被点名时放行（`C:1992`），所以全具体类型上的别名、引进变体两形今天照漏、收严后也照漏。模型里 `status_quo_concrete_enum_variant`、`status_quo_concrete_alias_impl` 两格量的就是这个（读代码推的预期是两份实现都是 0 个成员）。这两格判「同样打中别的臂」：它们打中的不是改法甲，是键算法在全具体类型 impl 上本来就有的缺口。

## 条目 2：判决第三节三条改法，够不够得着它们声称修的格

判决第三节原文三行：甲收严一（「别名、引进的变体两格、改名转出、撞工作区外类型名两格、泛型参数同名」）、甲收严二（「跨边界、正推腿形态十一」）、乙改成按单元逐个登记（「按模块登记的隐患；第二格」）。逐行核：

| 改法 | 声称修的格 | 辩方判定 | 依据 |
|---|---|---|---|
| 甲收严一 | 别名、引进的变体两格、改名转出、撞工作区外类型名两格、泛型参数同名（共七格） | 站得住（读代码推的，没量） | 七格各自走到哪一条条件，条目 1 逐格写了文件:行：泛型参数同名 `C:2106`；路径起头的自身类型两格 `C:2105`；别名 `C:2127`；改名转出 `C:2109`；引进变体两格 `C:2113`。判决文字里的五条与代码里的五条逐条对得上（`C:2105-2129`），没有哪一条文字写了而代码没写 |
| 甲收严二 | 跨边界、形态十一 | 站得住（读代码推的，没量；跨边界那一格任务 15787 的读数没有分辨力，见开头第 1 条） | 跨边界靠 `open_trait` 里对被开进来的 impl 头上约束的递归（`C:2273`）；形态十一靠入口所在 impl 头或 trait 头、入口签名与入口所在 impl 自身类型点名的类型的字段（`C:2167`、`C:2171`、`C:2199`）。判决文字里三处开放与代码里三处一一对上 |
| 乙 | 按模块登记的隐患；排除之后显存记账之外没有单元跟着少掉 | 站得住，第一格没做（判决自己记了欠账） | 登记现在逐个单元：「if self.units_not_followed.contains(&unit.qualified_name) {」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:1389`），登记表 52 个限定名（`awk` 数 `judging_units_entries.rs` 里那张表的 `gpu_memory_ledger` 行，量的：52）。第二格由 `crash_identity.rs` 里的测试 `not_following_the_listed_ledger_units_drops_no_unit_outside_the_memory_ledger` 断言少掉的恰是 `["mod singlefs_verification::gpu_memory_ledger"]`（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-verification/src/crash_identity.rs:1133`）；这是断言具体清单，不是对任何切法都成立的恒真句 |

### 判决说修了而代码没修的格

没找到。五条条件、三处开放、按单元登记，判决文字与代码一一对得上。

### 但「修了」没有会红的入库检查：这是这一节要交的主要东西

读入库的测试：新加的四条单元图测试是 `an_implementation_for_a_workspace_type_wrapping_its_generic_parameter_waits_for_the_type_to_be_named`（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/tests/unit_graph_blanket_implementations_are_those_whose_self_type_mentions_a_generic_parameter.rs:216`）、`…_is_in_the_closure_once_the_type_is_named`（`T:235`）、`an_entry_whose_signature_names_the_trait_takes_the_implementation_for_the_wrapper_nobody_names`（`T:247`）、`with_the_waiting_rule_broken_…`（`T:259`，整条规则的弄坏开关）；夹具 `fselftypes` 里新加的是 `Wrapper<Inner>`、`Box<Wrapper<Inner>>` 与一个签名里写 `dyn OnAWrapper` 的入口（`fixtures/blanket_self_types/fselftypes/src/lib.rs.fixture` 第 138 行起，整份没有一行 `use`：`grep -nE '^\s*(pub )?use '` 零行，量的）。
Python 一侧新加一格自证 `blanket_workspace_self_type_cell`（`P:18315`），用的是同一个 `Sealed<T>`（点名与不点名各一个入口）与同一个整条规则的弄坏开关 `rust-blanket-workspace-self-types-released`（`P:4928`）。

由此推出（推的，没拿弄坏版本去跑）：

- 五条条件里，只有「头一个名字不是工作区类型」（`Box<…>`，第三条的反面）被入库测试碰到；**泛型参数同名、路径起头的自身类型、`use … as`、`use` 路径穿过去这四条，删掉任何一条，Rust 的四条新测试与 Python 的一格自证都不会红**，因为夹具里没有这几种写法。它们现在只靠攻方腿研究目录里的夹具（`research/prompts/…-r1-opus-model/fixture`，不进门禁）。
- 三处新开放里，只有「入口签名里写 `dyn`」（`T:247`，开放本来就有）被碰到；`open_trait` 里的递归（`C:2273`）、入口所在 impl 头或 trait 头（`C:2167`）、沿字段找（`C:2199`）三处，删掉任何一处，入库测试都不会红。
- 这与证据纪律「改法碰不到打中的格」那条的要求相反：`.claude/rules/evidence-discipline.md:127`「写条款时每个改法都写明它修的是哪一格；打中之后先在打中的格上逐个核改法还中不中」——读代码这一步核了，但没有把核过的格留成会红的检查。
- 判决「改法」一节要求零轮改法写进产品代码前先做 A/B：「写进产品代码之前先在探针里对它做 A/B」（`.claude/rules/three-way-inference.md:112`）；A/B 读数任务 15787 给的是真仓 GPU 判器入口组的单元数与十个夹具的收不收，没有「删掉某一条条件之后哪一格红」的对照。

**要补的（交主 agent 判，不是我改）：** 每条条件与每处开放各一个夹具格 + 各一个弄坏开关（或至少一行变异），Rust 与 Python 各一份；夹具直接取攻方模型目录里的十个，每格单独成根（理由见开头第 1 条）。

### 乙的三条测试里我顺带看到的两处（不属于「修没修」，记一笔）

- 登记里有四个按行号取名的 use 单元「"singlefs_verification::gpu_memory_ledger::use@75",」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-verification/src/judging_units_entries.rs:877`）：往 `gpu_memory_ledger.rs` 第 75 行之上加一行，名字就对不上、测试 `the_gpu_judge_group_reaches_no_unit_of_the_memory_ledger_beyond_the_listed_ones`（`crash_identity.rs:1078`）当场红、要改登记；这是吵闹的失败不是静默的，不影响正确性，影响的是这份登记要不要随那个文件头部的改动一起改。
- 第三条测试 `an_edit_of_a_listed_unit_of_the_memory_ledger_keeps_the_gpu_judge_key`（`crash_identity.rs:1141`）改的是 `.min(state_count_as_usize)`（`gpu_memory_ledger.rs:92`，落在登记里的 `states_in_one_dispatch_capped_by_the_state_count` 里，该函数从第 84 行起），52 个单元里只抽了这一个；排除的机制在 `closure.rs:1389` 一处，所以抽一个够证机制，证不了「52 个每一个都真改不动判定结局」（后者是第一格，没做）。

## 条目 3：Rust 与 Python 两份写没写成一样（逐句对照）

对照用的是 `verify-copy` 里的两份：Rust 在 `crates/singlefs-unit-graph/src/`（`C`、`W`、`R` 简写同上），Python 在 `P`。两份的单元切分本来由门禁 code-tooling 的对拍格逐单元比过（指纹相同就是词法单元区间相同，背景材料写的任务 15767 读数：「dump 比了 55857 个单元（Rust 53627、WGSL 2074、数据 156），逐个相同」，转述，没复跑），所以下面比较的单元区间是同一个区间。

| 对照的一对 | 两边 | 判 |
|---|---|---|
| `implementation_waits_for_its_wrapper_type` ↔ `waits_for_its_wrapper_type` | `C:2096-2132` ↔ `P:4930-4938` | 逐句对过，没找到不等价。五条条件同序同口径：路径起头（`C:2105`）、泛型参数（`C:2106`）、改过名（`C:2109`）、路径穿过去（`C:2113`）、同文件同行内模块的 struct / enum / union（`C:2127-2129`），另有一句头一个名字取不到就返回假（`C:2102` ↔ `P:4934` 的 `first_name is None`）；Python 对应 `P:4934`、`P:4935`、`P:4937`「or first_name in workspace.names_renamed_by_a_use or first_name in workspace.names_a_use_path_passes_through):」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/scripts/gate/lib_function_units.py:4935`） |
| 头一个名字与泛型参数的收法（喂给上一行） | `R:1040-1047`、`R:933-957` ↔ `P:3012-3026`、`P:2970-2983` | 逐句对过。跳过的词都是同样八个「const WORDS_BEFORE_THE_SELF_TYPE_PATH_ROOT: [&str; 8] = [」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/rust_items.rs:41`）↔ `P:2378` 的 `SELF_TYPE_SKIPPED`；泛型参数都是「深度 1、紧跟 `<`、`,` 或 `const`」（「&& matches!(self.text_of(index - 1)」，`R:949` ↔ 「elif depth == 1 and self.kind(index) == IDENT and self.t(index - 1) in ("<", ",", "const") and text != "const":」，`P:2978`），两边都不数圆括号（`impl<F: Fn(A, B)>` 里 `B` 被当成泛型参数，两边一样多放行）。唯一写法差：箭头 `->` 里的 `>`，Rust 要求前一个词法单元是 `-` 且字节紧挨（`R:923`），Python 只看前一个词是 `-`（`P:2976`）；`- >` 中间有空白在 Rust 里不是箭头，两边只在这种编不过的写法上不同 |
| `names_written_where_a_trait_goes` | `C:2136-2156` ↔ `P:4943-4949` | 逐句对过。四个词 `impl`、`dyn`、`:`、`+`，下一个词法单元是标识符才收，范围都是这个单元的词法区间（止不含），都交原文（`C:2151` ↔ `P:4949`），两边都不筛关键字 |
| `open_the_traits_reaching_the_entry_from_outside` | `C:2162-2211` ↔ `P:5059-5082` | 逐句对过。七处一一对上：入口所在单元的头（`C:2167` ↔ `P:5064`）、头的自身类型名当要看的类型（`C:2171` ↔ `P:5066`；trait 头没有自身类型名，两边都空）、入口签名到头一个 `{`（`C:2180` ↔ `P:5070`，「没有 `{` 整个算签名」两边都是）、按名字找类型单元（`C:2192` ↔ `P:5076`）、每个类型单元至多看一次（`C:2189` ↔ `P:5077`）、类型单元里 `impl`/`dyn`/`:`/`+` 后面的名字（`C:2199` ↔ `P:5080`）、再沿单元里的裸名往下（`C:2205` ↔ `P:5082`） |
| `open_trait` 里对被开进来 impl 头上约束的递归 ↔ `dyn_trait` | `C:2268-2274` ↔ `P:5053-5057` | 逐句对过（推开 impl 连成员，再对 impl 头里的名字递归；每个 trait 名只开一次，`C:2258` ↔ `P:5050`） |
| 入口那一处的触发 | `C:3365-3367` ↔ `P:5179-5182` | 等价：Rust 无条件调，靠 `open_trait` 开头查「类的层次」开关；Python 在调用点查 `no_open`（`P:4911`）。同一个开关关掉时两边都不开 |
| 三张索引的建法 | `W:1248-1292` ↔ `P:4305-4318` | 逐句对过，只有下面「不等价」第一条的范围差。类型索引取 struct / enum / union / 别名（`W:1252` ↔ `P:2358`、`P:4312`），改名索引取 `imports.aliases` 的原名（`W:1262` ↔ `P:4315`），穿过去索引取 `use` 单元里后面紧接 `::` 的标识符（`W:1282`）↔ 「if rust_file.kind(index) == IDENT and rust_file.t(index + 1) == ":" and rust_file.t(index + 2) == ":":」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/scripts/gate/lib_function_units.py:4317`） |

### 不等价的地方（两边文件:行）

1. **索引遍历的文件集合不同。** Rust 遍历装进来的全部单元：「for (position, unit) in units.iter().enumerate() {」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/workspace.rs:1251`），装进来的是每个 crate 目录下的全部 `.rs`（`W:679` 的 `collect_source_files` 不看模块树）；Python 只遍历进了某个目标模块树的文件：「paths = sorted({path for target in self.targets.values() for path in target.files})」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/scripts/gate/lib_function_units.py:4269`）。同一个差别在老的索引上早就有（`W:1319` ↔ `P:4216`），不是这一批引进的。对等待判本身没有影响（同文件过滤，`C:2128` ↔ `P:4937`）；对「改过名」「穿过去」两张集合与入口那一步的按名字找类型（`C:2192` ↔ `P:5076`）有影响：模块树之外的 `.rs` 里要是定义了同名类型或写了 `use … as`，Rust 多放行 / 多开，Python 少。真仓里有没有这样的文件我没查（推的：登记入口上对拍绿，所以这些入口上没有）。
2. **原始标识符 `r#…`。** Rust 的 `name_at` 去掉 `r#`（`R:721-722`），Python 取原文（`P:3026`、`P:4317`）。只影响名字写成 `r#类型名` 的类型；没查仓里有没有（推的：没有）。
3. （老的、不是这一批引进的，记一笔）Rust 的 `signature_trait_names` 筛关键字「if !is_keyword(candidate) {」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/rust_items.rs:289`），Python 的 `entry_signature_traits` 不筛（`P:5084-5093`）；关键字没有 impl，不影响闭包。

### 对照的证据有多强

入库里两份实现在**同一个输入**上逐项比的，除了仓整份与登记入口之外，只有一份样本 `BLANKET_SHAPES_FIXTURE`：「BLANKET_SHAPES_FIXTURE = {」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/scripts/gate/lib_function_units.py:18235`），上一行的注释写明它在 code-tooling 的 function-units-two-implementations-agree-green 样本里与 unit-graph 对拍（`P:18234`）。新加的是 `Sealed<T>` 点名 / 不点名与 `Rc<T>` 放行，同样没有条目 2 说的四条条件的反面、没有三处开放。所以「两份逐句对过」是我读出来的，不是比出来的；模型里的 `closures_equal` 一栏就是补这个（末节命令 1），在十九格上一格一格比两份闭包。

## 条目 4：替「撤回改法甲」辩一次

判决第三节最后一行：「撤回改法甲 | 判据的另一条出路 | 不选：用户 2026-11-13 弹窗定」（`research/prompts/m3-close-unit-graph-wrapper-impls-r1-main-verification.md:38`），后面接「收严后继续改键算法」。我替撤回这一方辩护，只辩论据，不碰用户已定的事。

### 最强的一版理由

**撤回通用规则（甲收严一、二），GPU 判器那一组的收缩改走乙已经建好的按单元登记。** 理由四句：

1. **失败方向不对称。** 键过窄是违规「键过窄（引用的常量、类型、宏漏收）、库放在会丢的位置、改名丢库、两份库重复判同一批，同样算违反」（`/home/fy5090/code/singlefs/.claude/kb/decisions/13-验证路线.md:432`），键过宽只是多判。甲是一条自动的「可以少连」的证明：它对全仓每一把键、每一种判定生效，论据是一句关于 Rust 名字解析的话「值要么由闭包里某个单元写出 `Wrapper` 这个名字造出来，要么从入口外面传进来」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/crates/singlefs-unit-graph/src/closure.rs:2088`），这句话的例外是靠逐个列出来补的：原型一轮被打中八格，补成五条条件加三处开放，列表不是对 Rust 语法封闭的（还没排除的候选：同名类型在 cfg 的两臂里一臂本地定义、一臂是工作区外引进；模块树之外的 `.rs`；条目级宏生成的 `use`；`r#` 名字）。按单元登记的方向相反：新进闭包的单元默认进键，要排掉得有人点名、测试逼着判一遍（`crash_identity.rs` 里 `the_gpu_judge_group_reaches_no_unit_of_the_memory_ledger_beyond_the_listed_ones`，`crash_identity.rs:1078`）；出错的方向是多判，不是漏判。
2. **「第三轮之后停」把残余风险固定进键里。** 「第三轮之后停，不开第四轮」（`/home/fy5090/code/singlefs/.claude/rules/three-way-inference.md:122`）；腿提的收严按零轮算（同文件 `:110`）。第二轮攻方找得到的形，第三轮只攻站住的；第三轮新冒出的零轮形不再开轮。漏了的形没有任何东西会红，键只是不变。
3. **收严的证据自己不独立，覆盖也没入库。** 十个夹具是设计收严用的那十个（收严是照它们补的），其中一格的量测有分辨力问题（开头第 1 条）；五条条件的反面、三处开放没有入库的会红的检查（条目 2）；两份实现的对拍没有输入覆盖这几条（条目 3）。
4. **复杂度。** `round-two-changes.diff` 里 `closure.rs` 一处新增就是 140 行「@@ -2056,6 +2070,146 @@」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/evidence/round-two-changes.diff:72`），加 `workspace.rs` 三张索引、`units.rs` 与 `rust_items.rs` 两个字段、一个弄坏开关、四条测试、Python 一份同形，甲这一半涉及 8 个文件（`closure.rs`、`workspace.rs`、`rule_switches.rs`、`units.rs`、`rust_items.rs`、一份测试、一份夹具、`lib_function_units.py`；整份 diff 共 12 个 `+++` 文件，量的，另 4 个是乙的），而被它换来的是下面这些读数。
   另有一处没算在 GPU 判器那一组上的代价：入口那一步的新开放会沿 impl 头里的约束一路递归开（`C:2273`），一个 impl 头写了 `T: Debug` 就开所有 `impl … Debug for` 的 impl 与成员。仓里这类 impl 行数（`grep -rnE 'impl[^{]*Debug for ' crates --include=*.rs | wc -l`，量的：8；`Debug`、`Display`、`Clone`、`Default`、`Iterator`、`Drop`、`Error` 七个合计 88，量的，是行数不是单元数）。任务 15787 只量了 GPU 判器那一组，没量别的组。

### 它换来的是什么（转述，没复跑）

- 收严后真仓 GPU 判器入口组：背景材料第二节转述任务 15787「闭包 2932 个单元（Rust 1063、WGSL 1869、数据 0），拒算 0 处」，原型同样降到 1063；原型两趟的读数我读了产物：「MEASURE gpu rule=wrapper rc=0 lines=2933 unit-graph：闭包 2932 个单元（Rust 1063、WGSL 1869、数据 0）」（`/home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/evidence/measure-15314.txt:2`），单入口 `JudgeTables::words` 569 → 240、建判表一个入口 974 → 671（同文件 `:7`、`:17`）。整组 Rust 1361 → 1063，少 298 个，整组含 WGSL 从 3230 到 2932，约少 9%。
- 这 298 个里点名的五个文件各占：「`fault_injection.rs` 78 到 81、`history.rs` 31 到 35、`read_tally.rs` 33、核心层 `mount.rs` 23、`mounted_session.rs` 30」（`/home/fy5090/code/singlefs/records/2026-10-18-里程碑三收尾与近期欠账.md:2841`，任务 15142 的逐入口读数），合计 195 到 202 个；其余约 100 个在哪几个文件里，我这里没有读数。
- 它们进闭包的来由：「`MemoryPool` 的 `impl PoolReader` 成员按名字连到 `BlockDevice`，trait 头一进闭包，自身类型带泛型参数的 impl 全按 blanket 放行」（`/home/fy5090/code/singlefs/records/2026-10-18-里程碑三收尾与近期欠账.md:2845`，任务 15310 边链）：是放大，不是真调用边。
- 一次键变的代价：GPU 判器那一组任一单元变，镜像判定与给定状态判定的每一块都作废：「改 GPU 判器那一组的任一单元，这一类的每一块（CPU 判过的也在内）都作废、现判」（`/home/fy5090/code/singlefs/.claude/kb/decisions/13-验证路线.md:440`）。用户要修的就是这个：「改动一处其他全改的问题你要修复」（`/home/fy5090/code/singlefs/records/2026-10-18-里程碑三收尾与近期欠账.md:2839`）。

### 辩护的做法：乙的机制再用一次，不动通用算法

`compute_closure_of_entries_not_following_the_listed_units` 与三条测试那一套（`C:1389`、`crash_identity.rs:1078`、`:1115`、`:1141`）对 GPU 判器那一组再登记一张表：把 195 到 202 个放大进来的单元（连同其余那约 100 个）按限定名点名，依据是 `unit-graph why` 的边链（`records:2845` 说的 `why` 子命令已经有）里它们的头一条边是「trait 头带它的 impl」「R-方法」，而不是真调用边。别的所有组、所有按条复用的测试函数、变异、形式化的键照旧用今天那条过宽的规则，没有任何新的静默少连的可能。

### 这条辩护依赖的、我没核过的前提

1. **别的键不靠甲收缩。** 甲是通用规则，同一种放大对任何闭包里带 `BlockDevice` 一类 trait 的键都在；要是 CPU 判定那几组、按条复用的测试函数的键也因此缩了很多，撤回甲就丢掉了一大片，辩护不成立。我没有 GPU 判器那一组以外任何一组的收严前后读数（任务 15787 只量了这一组；末节命令 2 给了量法）。
2. **那约 200 到 300 个单元多久被改一次。** git 历史是压平过的（这一批文件各只有一条提交：`git log --oneline -- crates/singlefs-verification/src/fault_injection.rs | wc -l` 得 1，量的），量不出改动频率。一次键变就是整库作废、几个小时的卡时间，频率高则收缩值钱。
3. **这些单元确实不改 GPU 判定结局。** 乙的第一格（显存记账的行为不改判定结局）今天就没有会红的检查（判决第三节第 37 行，`research/prompts/m3-close-unit-graph-wrapper-impls-r1-main-verification.md:37`）；按单元登记这约 200 到 300 个，同样要有人判，且这一格同样没有会红的检查。「头一条边是放大」是机械可查的，「不改结局」不是。
4. **登记这么长一张表（约 300 个限定名）维护得动。** 乙的 52 个里已经有四个按行号取名的 `use@行`；fault_injection、history 这些文件改得比显存记账勤，登记会频繁对不上、测试频繁红（吵闹的失败，不是静默的）。
5. **我对残余形态的估计。** 「同名类型在 cfg 两臂里一臂本地定义、一臂工作区外引进」「模块树之外的 `.rs`」「条目级宏生成的 `use`」这三种我只读代码推到「等待判会判错」或「两份实现会不一致」，没有夹具、没有在真仓里找过。真仓里有没有，两条腿第一轮各抽一次样没找到（判决第二节「真仓里有没有现成的反例」，`research/prompts/m3-close-unit-graph-wrapper-impls-r1-main-verification.md:25`），不是「没有」。
6. **第三轮攻方不会再打中。** 辩护要成立，得第三轮仍有人能打中残余形；要是第二轮与第三轮都没打中、且条目 2 补的会红的检查补齐，辩护的第 1、3 句就弱了。
7. **开放递归在别的组里不膨胀。** 没量（上面第 4 句）。

### 推翻这条辩护的现象

任一条：（a）末节命令 2 量出别的组的闭包也因甲缩了一成以上；（b）改动频率读数显示那约 200 到 300 个单元在一周里被改过不止一次；（c）模型命令 1 十九格全绿、条目 2 的会红检查补齐后第三轮仍没有打中。（a）或（b）成立，撤回就不划算；（c）成立，残余风险一句弱了。

## 要主 agent 跑的命令

1. **模型：每个夹具单独成根，Rust 与 Python 两份各收一次闭包，比相同与否、数被看的 impl 方法**（条目 1 的八格与形态十一、条目 3 的两份对拍、条目 2 开头那条「十格同根」要的重量都在这一条里）。
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r2-sonnet -- bash /home/fy5090/code/singlefs/research/prompts/m3-close-unit-graph-wrapper-impls-r2-sonnet-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy /home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r2-sonnet/compare-run /home/fy5090/code/singlefs/research/prompts/m3-close-unit-graph-wrapper-impls-r2-sonnet-model`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r2-sonnet`（命令里路径都是绝对路径；判决复跑时照规则把模型目录拷进主 agent 自己的草稿目录，最后一个参数换成拷贝，第一个参数仓根不换——攻方夹具要从仓里的第一轮攻方模型目录取）。`rerun.sh` 在这棵 `verify-copy` 上编 `unit-graph`（`cargo build -p singlefs-unit-graph --bin unit-graph`），再起 `tools/compare_two_implementations.py`。
   - 看哪几行：末行 `TASK_SERVER_RESULT` 的 `exit=0` 是十九行都与 `out/expected.txt` 相同；`exit=1` 是有格不同，`task-server/task log <号> --all` 里 `  ✗ 与 expected 不同` 那几行点名哪一格；`exit=2` 是某一步跑不成（编不过、某个夹具原文不是恰好一次、单元图或 Python 退非 0），看 `✗` 那一行。每格读数：`name=<格名> rust_members=<个数> python_members=<个数> closures_equal=<yes|no>`；汇总行 `name=summary lines=19 mismatches=<n>`；两份闭包不同时另有 `  diff <格名> 只在 Rust：…` 或 `只在 Python：…` 行（不进与 expected 的比较，逐行是闭包里多出来的单元）。整份结果也落在运行目录的 `result.txt`。
   - 各格对条目的对应：`form11_*` 五格是形态十一与三种变体（预期都是 1）；`control_named_wrapper` 预期 1、`control_never_named_wrapper` 预期 0（等包装类型的规则让没点名的那份出去）；`status_quo_concrete_enum_variant`、`status_quo_concrete_alias_impl` 预期 0（全具体类型 impl 今天就漏的两形，「同样打中别的臂」）；其余十格是攻方十个夹具（八格加关联类型名那一格加点名包装类型的对照），预期都是 1。`out/expected.txt` 全是读代码推的，对不上的格就是要回头核的格。
   - 任何一格 `python_members` 与 `rust_members` 不同、或 `closures_equal=no`，就是条目 3 里两份对不上的坐实：两份对齐之前不合（判据，背景材料第四节）。
   - 跑完删运行目录 `/home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r2-sonnet/compare-run`（里面有 `verify-copy` 的拷贝与编译产物，删前 `du -sh`），编译目录照服务 `task-server/task clean <号>`。

2. **条目 4 前提 1 与 7 要的读数：别的组的闭包大小，收严前后各一趟。** 构建脚本会把每一组的组成清单写进 `OUT_DIR/judging_units_keys.tsv`（`unit\t<组名>\t…` 一单元一行，`entry_groups.rs:357-361`）。两棵树各编一次验证包，再数每组的单元行。
   - 命令一（今天的主工作区）：`task-server/task submit --cwd /home/fy5090/code/singlefs -- cargo build -p singlefs-verification --target-dir /home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r2-sonnet/target-main`
   - 命令二（收严后的副本）：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy -- cargo build -p singlefs-verification --target-dir /home/fy5090/.local/share/singlefs/work/m3-close-unit-graph-wrapper-impls-r2-sonnet/target-copy`
   - 在哪起：各自的 `--cwd`。注意主工作区里别的会话也在改文件：命令一编出来的「今天」混着别的改动，只拿两边都没动的那些组比，或改用第一轮开工快照树当命令一的 `--cwd`。
   - 看哪几行：两条各自末行 `TASK_SERVER_RESULT` 的 `exit=0`；然后（只读命令，主 agent 直接跑）`find <target 目录> -name judging_units_keys.tsv`，对每份 `awk -F'\t' '$1=="unit"{count[$2]++} END{for (group in count) print group, count[group]}' <文件> | sort`，两份并排比每组的单元数；`group\t<组名>\trefused` 的行要单独列（有组在收严后拒算就是新问题）。
   - 要回答的：除 `gpu_judge` 之外，有没有哪一组收严后少了一成以上的单元（条目 4 前提 1），有没有哪一组多了单元（开放递归膨胀，前提 7）。

3. 条目 2 要补的会红检查（每条条件、每处开放一格夹具加一个弄坏开关）是实现活，不是命令；我没写、也没交，交主 agent 判要不要开一批实现。

## 没做什么

- **没跑任何项目代码、没往统一任务服务交任务**；上面所有「够得着 / 够不着」「预期」「推的」都是读代码推的，没有一个闭包或键的读数是这一腿量的；判决要引的数没复跑（命令在上一节）。背景材料转述的任务读数（15787、15767、15310、15312、15314 等）都没复跑，只读了 `evidence/measure-15314.txt` 与 `round-two-changes.diff`（sha256 238247e7d699bea40095b2b701ca7b0ac6033ac62a459891aff2107e131d9731，量的，与背景材料一致）。
- 没看到任务 15787 实际交的命令（背景材料没写）：「十格同根」只是「若它用攻方的 `judge.py`」的推论。
- 没对真仓扫残余形态：cfg 两臂里一臂本地定义一臂工作区外引进同名类型、模块树之外的 `.rs`、条目级宏生成的 `use`、`r#` 类型名。没有夹具、没有真仓命中数；条目 4 把它们当前提 5 列出来。
- 没复核形态十二（全局量、线程局部、链接期登记）与「值经入口以外的通道进来」：判决没把它们记成修了的格。
- 乙：没判三条测试的判别力（攻方那一腿的活）；没读 52 个单元的语义，第一格（显存记账的行为不改判定结局）没有会红的检查，我也没提一个。
- 没拿弄坏版本（删掉某一条条件、某一处开放）去证入库测试不红：条目 2 开头那几条是读夹具与测试推的。
- 没改 `crates/`、`scripts/`、kb、规则与定义；没做 git 写操作；没往草稿目录写任何东西（认领的草稿目录是空的，没有仓副本与编译目录要删）。模型目录在 `research/prompts/` 下是这一腿的产出，没删。
- 不替主 agent 撤销或维持判决，只交复核结果。
