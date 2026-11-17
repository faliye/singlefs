# m3-close-unit-graph-wrapper-impls-r2：判决（2026-11-13）

正文 `research/prompts/_m3-close-unit-graph-wrapper-impls-r2-body.md`，开工快照 `research/prompts/m3-close-unit-graph-wrapper-impls-r2-snapshot/`。第二轮只攻第一轮判决第三节的三条零轮改法；上一轮判决 `research/prompts/m3-close-unit-graph-wrapper-impls-r1-main-verification.md`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 辩方（Sonnet） | `research/prompts/m3-close-unit-graph-wrapper-impls-r2-sonnet-output.md`，模型 `research/prompts/m3-close-unit-graph-wrapper-impls-r2-sonnet-model/` | 八格与形态十一「读下来都走得进闭包」；三条改法「站得住」；十格同根的量测对跨边界那一格没有分辨力；五条条件与三处开放入库测试几乎没碰；两份实现逐句对过没找到不等价；撤回改法甲的最强理由与它依赖的七个前提 |
| 攻方（Opus） | `research/prompts/m3-close-unit-graph-wrapper-impls-r2-opus-output.md`，模型 `research/prompts/m3-close-unit-graph-wrapper-impls-r2-opus-model/` | 收严后的改法甲「打中，7 个夹具」；两份实现不等价「打中，3 个夹具」；改法乙今天的树上没打中，有一处机制口子；收严二会不会放大「待量」 |

- 快照：主 agent 直接跑 `cat research/prompts/m3-close-unit-graph-wrapper-impls-r2-snapshot/*.txt | sha256sum -c`，主工作区的被判文件里 `scripts/gate/lib_function_units.py` 对不上（腿跑着的时候别的会话改过主工作区那一份；这一轮被攻的是草稿副本里的改动，腿引的是副本的行）。其余 OK。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/m3-close-unit-graph-wrapper-impls-r2-sonnet-output.md research/prompts/m3-close-unit-graph-wrapper-impls-r2-opus-output.md --root /home/fy5090/code/singlefs --background research/prompts/_m3-close-unit-graph-wrapper-impls-r2-background.md` 报对不上的都是腿引副本 `verify-copy` 里的行、行号差十几行：腿交回之后主 agent 往副本里写了再收严，行往后移了；逐处报的都是「原文在这份文件第 N 行」，引文本身在。判决引到的读数不取自这些行，取自复跑。
- 辩方模型复跑：模型拷到 `~/.local/share/singlefs/work/m3-close-gpu-judge-closure/defense-rerun/model`，任务 15820（再收严之前的副本）与任务 15909（再收严之后）都退 0，「name=summary lines=19 mismatches=0」。
- 攻方模型复跑：模型拷到 `~/.local/share/singlefs/work/m3-close-gpu-judge-closure/attack-two-rerun/model`。任务 15838 作废：两份单元图二进制 sha256 相同（两次编译落在同一个编译目录），Rust 两臂量的是同一条规则。主 agent 改副本里的 `tools/rerun.sh` 让两份各编进各自的目录（只动编译目录那四行），任务 15869 是腿交回那一刻的副本上的读数，任务 15908 是再收严之后的读数；两趟都退 1，因为读数与腿跑前写的预期不同。

## 二、判决

| 格 | 判 | 依据 |
|---|---|---|
| 第一轮八格与形态十一，收严后进不进得来 | **进得来（量的）**：辩方 19 格各自成根，形态十一的五种写法、第一轮十个夹具两份实现都收、闭包相同 | 任务 15820、15909「mismatches=0」，例「name=form11_field_dyn rust_members=1 python_members=1 closures_equal=yes」 |
| 任务 15787 对跨边界那一格有没有分辨力 | **没有**（辩方说得对）：十个夹具铺在同一个根里，三张索引互相串；那一格的结论改引各自成根的任务 15820 | 辩方报告条目 2 |
| 收严后的改法甲仍会让键过窄 | **打中四形（量的）**：值放在 `static` 的 trait 对象里、入口收 `&dyn Any` 再向下转型、入口是 trait 自带体的方法、cfg 两臂一臂 struct 一臂同名别名。两份实现都不收、改动后入口的值从 6 变 9 而键不变 | 任务 15869，例「name=trait_object_held_by_a_static rule=verify_copy impl_in_closure=0 key_changes_with_the_edit=no」，四形两份同形 |
| 攻方预期的另三形（开进来的 impl 字段里的 `dyn`、括号里的约束、高阶约束） | **没打中（量的）**：两份都收 | 任务 15869 那三组行都是「impl_in_closure=1 key_changes_with_the_edit=yes」 |
| 两份实现不等价 | **打中三形（量的）**：`r#` 路径引进的变体（Rust 收、Python 不收）、自身类型写成 `r#` 名字（Rust 不收、Python 收）、签名点名的类型另在不属于任何目标的文件里有定义（Rust 收、Python 不收）。辩方「逐句对过没找到」是读出来的，被这三格推翻 | 任务 15869 那三组行 |
| 收严二会不会把闭包放大 | **不会（量的）**：各登记组与测试入口的 Rust 单元数只降不升，gpu_judge 1361 对 1063、recording 1068 对 762、judgement_of_the_crash_images 3261 对 3023、observer.publish_boundary 580 对 250；两个对拍比法组与 gpu_facts_verifier 不变 | 任务 15869 的 `rust=` 行 |
| 辩方替撤回改法甲写的前提一「别的组的键是否也靠甲收缩」 | **也收缩**（上一行）：辩方写的推翻条件「别的组也因甲缩了一成以上」成立 | 同上 |
| 五条条件与各处开放的入库测试 | **不够**（辩方说得对）：删掉任何一条都不红 | 辩方报告条目 2；主 agent 现读副本里那四条测试 |
| 改法乙：按限定名比会不会误排同名单元 | **今天的树上没打中**（一次抽样）；机制口子（同标签的第二个 impl 块头会被一起排掉）成立，登记的 52 个名字今天各只对一个单元没核 | 攻方报告第三条 |
| 改法乙三条测试 | 各证得了自己的文档句；`mod gpu_memory_ledger;` 那一行从此不进这一组的键，是按单元排除的应有结果 | 攻方报告第三条；任务 15786「30 passed; 0 failed」 |
| 「显存记账的行为不改判定结局」 | **仍没有会红的检查**：攻方给的写法只做得成一部分（宽度扫描的蜕变断言、给编内核透传钉签名），设备丢失那一类做不成 | 攻方报告第三条 |

## 三、改法

用户 2026-11-13 弹窗定「再收严，进第三轮」。下面的再收严都写进了草稿副本的两份实现，主工作区没动。

| 改法 | 修哪一格 | 状态 |
|---|---|---|
| 等包装类型的第三条加严：同一段模块里叫这个名字的类型单元个个是 struct、enum 或 union，且这一段模块里没有 `use` 引进同名 | cfg 两臂 | **零轮**。A/B 读数：任务 15908「name=struct_in_an_inactive_cfg_arm_alias_in_the_active_arm rule=verify_copy impl_in_closure=1 key_changes_with_the_edit=yes」，Python 同 |
| 入口在 trait 头下时开这个 trait 自己 | 入口是 trait 自带体的方法 | **零轮**。A/B 读数：任务 15908 `receiver_of_a_provided_trait_method_entry` 两份都「impl_in_closure=1」 |
| 闭包里的 static 单元、条目级宏调用与提到 `Any` 的单元里写的 `dyn Trait` 照 R-开放 开 | `static` 里的 trait 对象、`dyn Any` | **零轮**。A/B 读数：任务 15908 `trait_object_held_by_a_static`、`trait_object_behind_a_dyn_any_parameter` 两份都「impl_in_closure=1」；各组 Rust 单元数与再收严之前相同（gpu_judge 1063、recording 762） |
| 两份实现对齐：`r#` 两边都去前缀再比；类型索引两边都只收属于某个编译目标的文件 | 三处不等价 | **零轮**。A/B 读数：任务 15908 那三组两份读数相同；真仓对拍格任务 15924「dump 比了 55869 个单元……逐个相同；登记入口 10 个……闭包与键逐个相同」 |
| 五条条件（第三条拆两条）与六处开放逐条配夹具、测试与变异 | 入库测试不够 | 测试任务 15917「29 passed; 0 failed」；12 行变异的证红任务 15923 排队中，结局出来补在这一行之后 |
| 改法乙登记的 52 个名字各只对一个单元 | 机制口子 | **没做**：进第三轮之前补一条测试（登记的每个限定名在工作区里恰好一个单元） |
| 「显存记账的行为不改判定结局」的检查 | 改法乙第一格 | **没做，记欠账**：做得成的那一部分（派活宽度的蜕变断言、编内核透传的签名）排在第三轮之后；设备丢失那一类写明做不成 |

全具体类型的 impl 上「引进变体造值」「impl 写在别名上」两形今天就漏、这套改法前后都不收（任务 15820 `status_quo_concrete_enum_variant`、`status_quo_concrete_alias_impl` 两份都 0）：是键算法现有的缺口，另立欠账，不并进这一题。

第三轮是最后一轮：只攻前两轮站住的形态与上表的零轮改法；第三轮里新冒出来的零轮形态不再开轮，写进判决的交用户表。

## 交用户的

无新岔路：第二轮之后的去向用户已定（再收严，进第三轮）。

## 这一轮被判过的文件

- crates/singlefs-unit-graph/src/closure.rs：副本里的 `implementation_waits_for_its_wrapper_type`、`open_the_traits_reaching_the_entry_from_outside`、`open_trait` 的递归、不沿登记单元收的收法。五条条件的第三条在 cfg 两臂上不够严；入口在 trait 头下、static 与 `Any` 里的 `dyn` 两处没开
- crates/singlefs-unit-graph/src/rust_items.rs：副本里 impl 头的两个新字段。取头一个名字时去 `r#` 前缀，Python 那一份照原文取，两份不等价
- crates/singlefs-unit-graph/src/workspace.rs：副本里的三张索引。类型索引收不属于任何目标的文件，Python 那一份只收目标里的，两份不等价
- crates/singlefs-unit-graph/src/units.rs：副本里 `ImplementationShape` 的两个新字段，写法与用处对得上
- crates/singlefs-unit-graph/src/rule_switches.rs：副本里新加的弄坏开关，只罩整条规则，罩不到五条里的每一条
- crates/singlefs-unit-graph/src/entry_groups.rs：副本里带登记算键的入口，登记为空时与原来逐字相同
- crates/singlefs-verification/src/judging_units_entries.rs：副本里显存记账的 52 个登记名，按限定名比
- crates/singlefs-verification/build.rs：副本里构建脚本改成带登记算键，编进去的键与照树重算的相同（任务 15786）
- crates/singlefs-verification/src/crash_identity.rs：副本里改法乙的三条测试，各证得了自己的文档句
- scripts/gate/lib_function_units.py：副本里 Python 一份的同名函数，三处与 Rust 不等价（上表）

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 17 | 支撑 | 2026-11-13 不受影响：这一轮没改条款；第三轮站住、合进主工作区时再改收闭包与边级排除那两句 |
