# m3-close-unit-graph-wrapper-impls-r1：判决（2026-11-13）

正文 `research/prompts/_m3-close-unit-graph-wrapper-impls-r1-body.md`，开工快照 `research/prompts/m3-close-unit-graph-wrapper-impls-r1-snapshot/`。这是键算法两处收窄的第一轮。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推（Sonnet） | `research/prompts/m3-close-unit-graph-wrapper-impls-r1-sonnet-output.md` | 改法甲的论证「部分成立」：值在闭包里造出来的十种写法成立；入口签名以外进来的值、`use … as` 跨模块改名、impl 写在别名上、泛型参数与工作区类型同名四形不成立。改法乙三格里第一格证不了 |
| 攻方（Opus） | `research/prompts/m3-close-unit-graph-wrapper-impls-r1-opus-output.md`，模型 `research/prompts/m3-close-unit-graph-wrapper-impls-r1-opus-model/` | 改法甲「在夹具上打中，真仓里没找到反例」，九个夹具；改法乙「没打中」，按模块排除有隐患，三格自证各有证不了的 |

- 快照：写判决之前主 agent 直接跑 `cat research/prompts/m3-close-unit-graph-wrapper-impls-r1-snapshot/*.txt | sha256sum -c`，六份里五份 OK，`scripts/gate/lib_function_units.py` 对不上（腿跑着的时候别的会话改过它；没倒推）。腿引到这一份的只有两处行号（正推腿 `lib_function_units.py:3013`、`:18198`），主 agent 照今天的文件重读：第 3013 行「blanket = split is not None and bool(self_types & generics)」、第 18199 行「BLANKET_SHAPES_SELF_TYPES = ("(T, u32)", "[T]", "[T; N]", "Box<T>", "Option<Box<T>>", "Sealed<T>")」（差一行），内容与腿说的相同。其余五份就是开工那一刻的文件，`cite-check.py` 的 `--root` 直接给仓根。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/m3-close-unit-graph-wrapper-impls-r1-sonnet-output.md research/prompts/m3-close-unit-graph-wrapper-impls-r1-opus-output.md --root /home/fy5090/code/singlefs --background research/prompts/_m3-close-unit-graph-wrapper-impls-r1-background.md`，末行「✓ 核了 48 处引文、76 处裸行号，对上 48 处，没判 76 处（其中裸行号没有引文可比 70 处；逐处列在上面）」。没判的裸行号里，判决引到的几处主 agent 现读过（下表依据栏）。
- 攻方模型复跑：模型目录拷到 `~/.local/share/singlefs/work/m3-close-gpu-judge-closure/attack-rerun/model`，任务 15620 在副本里跑腿报告里的命令，退 1：30 行里 29 行与腿的预期相同、1 行不同（`name=summary lines=30 mismatches=1`）。不同的是 `associated_type_name_in_the_self_type`：腿预期原型不收，量到「rule=wrapper_waits_for_its_type impl_in_closure=1 key_changes_with_the_edit=yes」，原型收了。
- 正推腿的夹具（`membership.sh`）没复跑：它的五形与攻方的夹具重合四形，第五形（入口接收者字段里的 `dyn`、impl 头上的约束）没有量过，下表记「推的」。

## 二、判决

| 格 | 判 | 依据 |
|---|---|---|
| 改法甲（原型的写法）会让键过窄 | **打中（量的）**，八格。今天的规则收、原型不收、改那一行入口交回的值从 6 变 9 而键不变 | 任务 15620 的八组行，例：「name=impl_written_on_a_generic_alias rule=wrapper_waits_for_its_type impl_in_closure=0 key_changes_with_the_edit=no」「name=impl_written_on_a_generic_alias value_before=6 value_after=9」；其余七格 `enum_built_from_an_imported_variant`、`enum_built_from_a_glob_imported_variant`、`external_type_sharing_a_private_type_name`、`external_type_sharing_a_type_name_in_another_crate`、`generic_parameter_sharing_a_type_name`、`wrapper_built_by_the_caller_of_an_open_entry`、`wrapper_reached_through_a_renamed_reexport` 同形 |
| 关联类型名被当成工作区类型名那一形 | **没打中（量的）**：原型在这个夹具上照收 | 任务 15620 那一行，见上 |
| 入口接收者字段里的 `dyn`、入口所在 impl 头上的约束（正推腿形态十一） | **推的，没量**；机理与攻方「跨边界」那一格同一条（R-开放只开入口函数自己的签名），按打中办 | 正推腿报告「形态十一」；主 agent 现读 `crates/singlefs-unit-graph/src/closure.rs` 入口那一段只对 `signature_trait_names` 调 `open_trait` |
| 这些缺口是不是改法甲新造的 | **不是**：别名、引进的变体、改名转出、跨边界四形在全具体类型的 impl 上今天就漏，今天对包装类型靠 blanket 放行盖住；改法甲把盖不住的面扩到包装类型 | 两条腿各自的论述，互不冲突 |
| 真仓里有没有现成的反例 | **两条腿都没找到**，各一次抽样，不写成「没有」：攻方扫了 62 个带泛型参数的 impl 与登记入口的签名；正推腿读了 GPU 判器组掉的 13 条 trait impl 的两个 trait | 两份报告 |
| 改法乙：显存记账里今天在 GPU 判器闭包里的 52 个单元改不改判定字 | **没打中**（攻方逐个读过，一次抽样）；前提是「派活的切法不进结论」，与调度不登进组同一个前提，攻方没另证 | 攻方报告「52 个单元逐个读」 |
| 改法乙按模块登记 | **不成立**：同一个模块里有按块序写库这类改判定结局的代码（`BlocksStoredInOrder`、`judge_blocks_on_one_card`），今天不在闭包里，按模块登记将来会把它们静默切掉 | 攻方报告「按模块排除的写法」；主 agent 现读 `crates/singlefs-verification/src/gpu_memory_ledger.rs` 有这两个名字 |
| 改法乙三格自证 | **三格都要重写**：第一格分不出「对拍判了且相同」与「对拍报本次未跑」，抽的变异也不落在要证的单元上；第二格对任何按单元截断的切法恒成立；第三格只证得了网会红 | 两份报告各自的问题 3 与第二条，互不冲突 |

## 三、改法

| 改法 | 修哪一格 | 状态 |
|---|---|---|
| 甲收严一：等包装类型被点名要五条都满足（头一个名字不是 impl 自己的泛型参数；自身类型不写成路径；同一份文件同一段行内模块里有叫这个名字的 struct、enum 或 union；没有 `use … as …` 给它改过名；没有 `use` 的路径从它身上穿过去），差一条照 blanket 放行 | 别名、引进的变体两格、改名转出、撞工作区外类型名两格、泛型参数同名 | **零轮**（两条腿各提了一半，主 agent 合成的写法）。A/B 读数：任务 15787（原交的 15763 排队一小时没起、取消后同一件量测重交）。收严后的单元图在攻方十个夹具上每格都收、键都变，例「name=impl_written_on_a_generic_alias rule=wrapper_waits_for_its_type impl_in_closure=1 key_changes_with_the_edit=yes」，十格同形（这份二进制上两臂是同一条规则，与腿给原型写的预期差 9 行是应有的：「name=summary lines=30 mismatches=9」）；真仓 GPU 判器入口组「V2_GPU rc=0 unit-graph：闭包 2932 个单元（Rust 1063、WGSL 1869、数据 0），拒算 0 处」，`fault_injection.rs`、`read_tally.rs`、`mounted_session.rs`、`singlefs-core/src/mount.rs` 各 0 个、`history.rs` 2 个，「V2_ONLY_IN_V2 0」：与原型同样从 1361 降到 1063，没有新进来的单元 |
| 甲收严二：被 R-开放 开进来的 impl 头上的约束照开；入口所在 impl 头或 trait 头上的约束、入口签名与入口所在 impl 的自身类型点名的类型沿字段找到的约束与 `dyn Trait` 照开 | 跨边界、正推腿形态十一 | **零轮**。同上一行的任务（跨边界那一格 `wrapper_built_by_the_caller_of_an_open_entry` 在里面；正推腿形态十一的夹具没跑） |
| 乙改成按单元逐个登记（52 个限定名），配三条测试：够到的显存记账单元恰好等于登记的那几个（陈旧登记红、新单元进闭包红）；排除之后显存记账之外没有单元跟着少掉；改一个登记了的单元 GPU 判器的键与镜像判定那一把都不变 | 按模块登记的隐患；第二格 | **零轮**。测试任务 15776 排队中 |
| 乙的第一格（显存记账的行为不改判定结局）改成什么 | 第一格 | **没做**：攻方要的判别子是「每一条变异下对拍真在 GPU 上判了的状态数大于 0 才算相同」，落到登记的 52 个单元上的变异行今天只认出一行；这一格记欠账，不拿今天的写法当证过 |
| 撤回改法甲 | 判据的另一条出路 | 不选：用户 2026-11-13 弹窗定「收严后继续改键算法」 |

收严一、二与乙的改法都在草稿副本里写好了 Rust 一份，甲另有 Python 一份；主工作区里这一轮被判的文件没动。合进主工作区之前开第二轮，只攻这三条零轮改法。

## 交用户的

无新岔路：改法甲的去向用户已定（收严后继续）；改法乙第一格没有会红的检查，记欠账。

## 这一轮被判过的文件

- crates/singlefs-unit-graph/src/closure.rs：blanket 的三处放行与入口的 R-开放。改法甲原型的写法在八个夹具上让键过窄；R-开放只开入口函数自己的签名这一条对全具体类型的 impl 今天就有同样的缺口
- crates/singlefs-unit-graph/src/rust_items.rs：impl 头的自身类型名字与 blanket 的判法。自身类型名字集合不分泛型参数、别名、路径根，原型拿它整份去比是八格里五格的来由
- crates/singlefs-unit-graph/src/workspace.rs：按名字建的索引。原型的「工作区类型名」取全工作区每个类型单元的裸名，连别名与关联类型
- crates/singlefs-verification/src/judging_units_entries.rs：判器组的入口登记。改法乙的登记放这里，按单元登记
- scripts/gate/lib_function_units.py：Python 一份同一口径。腿引到的两处现读与腿说的相同；改法甲的 Python 一份这一轮还不存在，涉及它的都是推的

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 17 | 支撑 | 2026-11-13 不受影响：这一轮没改条款；收严后的规则与 Rust 一侧的边级排除合进主工作区时再改「函数单元从这份判定的入口沿调用与引用收」与边级排除那两句 |
