# m3-close-unit-graph-wrapper-impls-r3：判决（2026-11-13，最后一轮）

正文 `research/prompts/_m3-close-unit-graph-wrapper-impls-r3-body.md`，开工快照 `research/prompts/m3-close-unit-graph-wrapper-impls-r3-snapshot/`。前两轮判决 `research/prompts/m3-close-unit-graph-wrapper-impls-r1-main-verification.md`、`research/prompts/m3-close-unit-graph-wrapper-impls-r2-main-verification.md`。这一题不再开轮。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推（Sonnet） | `research/prompts/m3-close-unit-graph-wrapper-impls-r3-sonnet-output.md`，模型 `research/prompts/m3-close-unit-graph-wrapper-impls-r3-sonnet-model/` | 五条里四条成立、「自身类型写成路径」不独立起作用；五条都满足后还有四处缺口（static 与 `Any` 那一支）；两份判决的数核到了、两句与事实不符、三处核不到；九处条件分句与开放没有变异行 |
| 攻方（Opus） | `research/prompts/m3-close-unit-graph-wrapper-impls-r3-opus-output.md`，模型 `research/prompts/m3-close-unit-graph-wrapper-impls-r3-opus-model/` | 零轮改法的边角「打中 10 形」；第二轮判没打中的三形是夹具串根造出来的，重量预期都打中；两份实现「打中 2 形」；改法乙「没打中」；未定项清单 16 类 |

- 快照：这一轮被攻的是草稿副本 `verify-copy`，腿跑着的时候主 agent 没改它（十二行变异里没红的那一行的夹具改动留到腿交回之后，此刻仍没改）。
- 正推腿模型复跑：拷到 `~/.local/share/singlefs/work/m3-close-gpu-judge-closure/forward-three-rerun/model`，任务 16006 退 0，「name=summary lines=12 mismatches=0」。
- 攻方模型复跑：拷到 `~/.local/share/singlefs/work/m3-close-gpu-judge-closure/attack-three-rerun/model`，任务 16025 退 0，「name=summary lines=80 mismatches=0 failed_steps=0」，「name=binaries」两份 sha256 不同，「name=dump same=yes」。
- 前两轮判决里与事实不符的两句（正推腿核出，主 agent 认）：第一轮「测试任务 15776 排队中」——15776 编译失败退 101，绿的是任务 15786（改过断言）与任务 15942；第二轮「12 行变异的证红任务 15923 排队中」——15923 一条没跑，结局在任务 15952：「计数：抓到 11 条、没红 1 条、无效 0 条、锚点失效 0 行（共 12 条）」。正推腿核不到的三处：第一轮「原型」那一臂的 1063 出自任务 15314 的「MEASURE gpu rule=wrapper … Rust 1063」；「15820 是再收严之前的副本」是主 agent 交任务的次序，日志里没有标记；「一小时」是排队时长的约数，不承重。

## 二、判决

| 格 | 判 | 依据 |
|---|---|---|
| 收严后（副本此刻的写法）仍会让键过窄 | **打中（量的），前后两条腿合计十九格**。正推腿六格：static 的类型是 struct 而 `dyn` 写在 struct 里、static 的类型是 `dyn` 的别名、`Any` 向下转型成 `dyn` 的别名、擦除与向下转型分在两个单元、函数体里的 static、`dyn Send + Tr` 的 `Tr`。攻方十三格：cfg 一臂 glob 引进同名、条目级宏造的同名别名、父 trait 写成高阶约束、static 的类型写成别名、自动 trait 写在前头的 trait 对象、函数体里的 static、转型目标写成别名、`Any` 经别名到入口、入口签名里带括号的约束、条目级宏造的 `use` 引进变体，与重量的三格 | 任务 16006 六格「rust_members=0 python_members=0」；任务 16025 十三格「rule=verify_copy impl_in_closure=0 key_changes_with_the_edit=no」，Python 同 |
| 第二轮判「没打中（量的）」的三形（开进来的 impl 字段里的 `dyn`、括号里的约束、高阶约束） | **第二轮判错，改判打中**：第二轮的夹具铺在同一个根里互相串；各自成根重量三格都不收 | 任务 16025 `trait_object_field_of_an_opened_implementation`、`bound_in_parentheses_on_an_opened_implementation`、`higher_ranked_bound_on_an_opened_implementation` 三行都是「impl_in_closure=0」 |
| 两份实现不等价 | **打中两形（量的）**：`r#Any`、入口签名里的 `r#Store`，Rust 收、Python 不收 | 任务 16025 那两组行 |
| 把「值经全局状态或类型擦除进来」整类盖住 | **代价量到了，盖住之后没有收缩**：闭包里任何单元写的 `dyn` 都开放时 gpu_judge 1063 变 1462（今天 1361），`fault_injection.rs` 82、`read_tally.rs` 33、`mounted_session.rs` 48 都回来 | 任务 16008「EVERY_DYN gpu_judge rule=narrow rc=0 Rust 1063」「EVERY_DYN gpu_judge rule=every rc=0 Rust 1462」 |
| 「自身类型不写成路径」那一条 | **不独立起作用**（正推腿说得对）：头一个名字取的就是路径根，被第三条先拦 | 任务 15952 那一行没红 |
| 真仓 `crates/` 里今天有没有这些写法的实例 | 按单元图的切分扫：十六类里有实例的五类。static 经别名持有 trait 对象 3 处（都在 `crates/singlefs-verification/src/verdict_store.rs` 第 2160、2163、2166 行，持的是回调）、`Any` 的别名 1 处（实验装置里的 panic 载荷）、`use` 引进变体 1 处（那个 enum 的 trait impl 数 0）、trait impl 的自身类型里有 trait 对象 11 处、登记入口外的调用方给的回调 11 处。主 agent 现读前三类的五处：没有一处后面是工作区 trait 的包装类型 impl；后两类 22 处没逐处判 | 任务 16025 的 `class=` 行与运行目录 `scan.txt` |
| 改法乙：登记的 52 个显存记账单元改不改判定字 | **第二次没打中**（换读法追返回值流向），连第一轮共两次抽样；其中 `compile_for_the_work_of_this_thread`、`states_in_one_dispatch_capped_by_the_state_count` 两个靠「编内核透传是参数化的」「切法不进结论」两个前提，没有会红的检查守着 | 攻方报告第三条 |
| 改法乙的机制与四条测试 | **站住**：按单元登记、带登记算键、四条测试两轮都没被打中 | 任务 15942「31 passed; 0 failed」 |
| 测试与变异的覆盖 | 十二行里十一行真靠那一处才绿；九处条件分句与开放没有变异行，Python 一份没有按条的开关 | 正推腿报告问题 3 |

三轮合起来：改法甲每一轮都在「写出名字而解不到类型单元」与「值经全局状态或类型擦除进来」两类上被打中新形，照腿的改法收严之后下一轮在同两类上再被打中；后一类整类盖住的代价是闭包比今天还大。改法乙两轮没打中。

## 三、改法

| 改法 | 状态 |
|---|---|
| 改法乙（GPU 判器那一组按单元登记不沿显存记账收） | 两轮站住，可以合。合之前补：登记单元里靠前提的那两个各配一条会红的检查（派活宽度的蜕变断言、编内核透传的签名），设备丢失那一类写明做不成 |
| 改法甲（副本此刻的写法） | **不照原样合**：这一轮坐实十九格、两份实现两处不等价。去向交用户（下一节） |
| 十二行变异里没红的那一行 | 随改法甲的去向办：撤回则连同那一条条件删掉；保留则删掉「不写成路径」那一条与它的变异（不独立起作用），或改夹具让它起作用 |

## 交用户的

| 岔路 | 候选 | 各自量到的 | 被攻过几轮 |
|---|---|---|---|
| 改法甲的去向 | 撤回改法甲，全仓键算法不动；要切的几类（故障注入、读计数、核心层挂载）在 GPU 判器那一组走改法乙的按单元登记 | 键算法没有新的漏连；GPU 判器那一组能切到与改法甲相同的范围（登记多少排多少），别的组不变 | 机制两轮站住；这几类的登记本身零轮 |
| | 照副本此刻的写法合，把规则不保证收到的十六类写法登记成清单，用单元图切分的扫描做成门禁格：真仓里出现这类写法且后面是工作区 trait 的包装类型 impl 就红 | 各组收缩保住（gpu_judge 1361 对 1063、recording 1068 对 762）；十九格已知漏连不靠闭包收、靠扫描拦；今天真仓前三类五处都不涉及包装类型 impl | 规则三轮每轮被打中；扫描脚本零轮 |
| | 照副本此刻的写法合并整类盖住（闭包里任何 `dyn` 都开放） | gpu_judge 1462，比今天的 1361 还大，没有收缩 | 零轮 |

## 这一轮被判过的文件

- crates/singlefs-unit-graph/src/closure.rs：副本里五条等待条件、入口与 static、`Any` 的开放。第三条看不到 glob 与条目级宏造出的同名；开放只读单元自己写的 `dyn`、取约束名只认紧跟 `:`、`+` 的名字，十九格由此漏
- crates/singlefs-unit-graph/src/rust_items.rs：副本里 `dyn_trait_names` 只记 `dyn` 后头一个 trait，`dyn Send + Tr` 的 `Tr` 不记
- crates/singlefs-unit-graph/src/workspace.rs：副本里三张索引，`use` 路径穿过的名字只认 use 单元里的，条目级宏造的 `use` 不认
- crates/singlefs-unit-graph/src/units.rs：副本里 impl 头的两个字段，头一个名字取路径根，使「不写成路径」那一条不独立起作用
- crates/singlefs-unit-graph/src/rule_switches.rs：副本里只有整条规则一个弄坏开关
- crates/singlefs-unit-graph/src/entry_groups.rs：副本里带登记算键的入口，两轮站住
- crates/singlefs-verification/src/judging_units_entries.rs：副本里显存记账的 52 个登记名，两轮没找到改判定字的
- crates/singlefs-verification/build.rs：副本里构建脚本带登记算键，站住
- crates/singlefs-verification/src/crash_identity.rs：副本里改法乙的四条测试，站住
- scripts/gate/lib_function_units.py：副本里 Python 一份，`r#Any` 与入口签名里的 `r#` 名字照原文取，与 Rust 两处不等价

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 17 | 支撑 | 2026-11-13 不受影响：三轮都没改条款；用户定去向之后，合进主工作区的那一部分再改边级排除那一句 |
