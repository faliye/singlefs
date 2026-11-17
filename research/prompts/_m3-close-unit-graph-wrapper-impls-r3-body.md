# m3-close-unit-graph-wrapper-impls-r3：前两轮站住的形态与第二轮的零轮改法（最后一轮，2026-11-13）

批次：`records/2026-10-18-里程碑三收尾与近期欠账.md` 第 2842–2859 行那一批。前两轮判决 `research/prompts/m3-close-unit-graph-wrapper-impls-r1-main-verification.md`、`research/prompts/m3-close-unit-graph-wrapper-impls-r2-main-verification.md`。这是第三轮，之后不再开轮：这一轮里新冒出来的零轮形态写进判决的交用户表。

## 一、题面

键算法（D13（验证路线） 已定项 17）的两处收窄：带泛型参数的 impl 等它的包装类型被点名才进闭包（改法甲，连同两轮的收严），与 GPU 判器那一组按单元登记不沿显存记账收（改法乙）。前两轮各打中一批漏连，都照腿的改法收严了；用户两次定「继续」。这一轮问三件事：前两轮判「站住」的那几样撑不撑得住；第二轮判决第三节的零轮改法还会不会让键过窄；两轮报出的数能不能核。

## 二、实现今天的样子（主 agent 的观测，2026-11-13）

方案按 `crates/` 今天的实现来谈。主工作区的单元图与判器组登记还是第一轮开工快照那一份；被攻的改动在草稿副本里：

- diff 全文：`~/.local/share/singlefs/work/m3-close-gpu-judge-closure/evidence/round-three-changes.diff`（1915 行，sha256 904112959a901234109fe5c07829deb2111a30a5e4e78cccefd04154eddfa70b）。
- 改后的整棵树（只读）：`~/.local/share/singlefs/work/m3-close-gpu-judge-closure/verify-copy/`。规则的现行写法在 `crates/singlefs-unit-graph/src/closure.rs` 的 `implementation_waits_for_its_wrapper_type`（五条，第三条分两句）、`open_the_traits_reaching_the_entry_from_outside`、`open_trait`、`visit_unit_for_target` 里 static 与 `Any` 那一段、`compute_closure_of_entries_not_following_the_listed_units`；Python 一份 `scripts/gate/lib_function_units.py` 的同名函数与 `without_the_raw_identifier_prefix`。测试与夹具：`crates/singlefs-unit-graph/tests/unit_graph_blanket_implementations_are_those_whose_self_type_mentions_a_generic_parameter.rs`（29 条）与 `tests/fixtures/blanket_self_types/fselftypes/src/lib.rs.fixture`；改法乙的四条测试在 `crates/singlefs-verification/src/crash_identity.rs`；十二行新变异在副本的 `crates/mutations.tsv` 末尾（原文另存 `~/.local/share/singlefs/work/m3-close-gpu-judge-closure/wrapper-mutations.tsv`）。

量的（任务号都在两份判决里有整行）：

- 第一轮十个夹具与形态十一的五种写法各自成根，两份实现都收、闭包相同（任务 15820、15909，`mismatches=0`）。
- 第二轮坐实的四形与三处两份实现不等价，再收严之后两份都收、读数相同（任务 15908）。
- 各登记组的 Rust 单元数，今天对副本：gpu_judge 1361 对 1063、judgement_of_the_crash_images 3261 对 3023、judgement_of_the_given_states 3165 对 2872、recording 1068 对 762、observer_conclusion_composition 4249 对 3930、observer.publish_boundary 580 对 250、observer.instance_table_sweep 632 对 338；gpu_facts_verifier 152、两个对拍比法组 218 与 2 不变（任务 15908）。
- 真仓上两份实现对拍格绿：「dump 比了 55869 个单元（Rust 53639、WGSL 2074、数据 156），逐个相同；登记入口 10 个（stage-inputs.tsv 18 个），闭包与键逐个相同」（任务 15924）。
- 单元图四个测试目标绿（任务 15911），新加的 12 条测试绿（任务 15917「29 passed; 0 failed」），判器键那一模块「31 passed; 0 failed」（任务 15942，含改法乙四条与「编进去的键与照树重算的相同」）。
- 没出结局的：十二行新变异的证红（任务 15952）。
- 没有会红的检查的：改法乙「显存记账里登记的 52 个单元的行为不改判定结局」（第二轮判决记欠账）。
- 今天就有、这套改法前后都不收的：全具体类型的 impl 上「引进变体造值」「impl 写在别名上」（任务 15820 两格都是 0）。

## 三、分工

| 腿 | 立场 | 攻击面 |
|---|---|---|
| 正推（Sonnet） | 「已定」的撑不撑得住、数能不能核 | ① 逐条对着副本的代码把规则现行的写法重推一遍：五条等待条件各自挡的是哪一种「写出名字而解不到这个类型单元」，五条合起来之后「等」的那一类 impl 里还有没有值不经点名就到得了闭包的路（只在 static、`Any`、入口签名与 impl 头、被开进来的 impl 头这几处开放之外找）；② 两份判决引的每一个数，照它点名的任务日志或产物逐个核：日志路径 `~/.local/state/singlefs/task-server/logs/task-b0f6b921-<任务号>.attempt-<次>.log`，核不到的列出来；③ 十二行变异与 29 条测试：每一条条件、每一处开放是不是各有一行变异点到、点名的测试是不是真靠那一条才绿（读夹具，逐行判），列出没被任何一行罩到的条件分句 |
| 攻方（Opus） | 找前两轮与第二轮零轮改法都没看过的地方 | ① 第二轮的三条零轮改法各自的边角：第三条的「同一段模块里没有 `use` 引进同名」对 glob 引进、对 `use` 写在外层模块或函数体里、对 `extern crate`/`pub(crate) use` 重导出成不成立；「入口在 trait 头下时开这个 trait」对 trait 的父 trait、对关联函数（没有接收者）成不成立；static 与 `Any` 那一条认的三类位置之外，值还能经哪种全局状态或类型擦除进来（`const` 里的函数指针表、`thread_local!` 之外的宏、`Box<dyn Fn>` 回调捕获的包装值、`&dyn Debug`/`Display` 这类标准库 trait 对象后面的包装类型、`impl Trait` 返回值经另一个 crate 转手）；② 「未定项清单的完整性」：把规则不保证收到的每一类写法列全（包括今天就漏的），判哪些在真仓 `crates/` 里今天有实例——这一轮真仓扫描要用单元图自己的切分（`unit-graph dump`），不用 grep 近似；③ 改法乙：52 个登记单元里有没有哪一个的行为今天就改判定字（第一轮攻方读过一遍，这一轮换读法：从 `judge_one_dispatch` 与 `on_card_compiling_these_kernel_sources` 两个入口往下，追登记单元的返回值流到哪）；登记之后 `mod gpu_memory_ledger;` 那一行与这个模块的 `use` 不进键，有没有因此漏掉的 cfg 或特性依赖；④ 两份实现：第二轮对齐之后再找一次不等价，重点在这一轮新加的 Python 代码（static、`Any`、trait 头入口、第三条的两句）与 Rust 的判定次序 |

两条腿都按 `crates/` 今天的实现与 `verify-copy` 里的改动来谈；引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。要跑的命令一律写进报告「要主 agent 跑的命令」一节，不自己交任务。腿跑着的时候主 agent 不改 `verify-copy`。

## 四、判据（跑前写死）

- 攻方给出新反例（今天的规则收、副本的规则不收、改它入口结果变），主 agent 复跑坐实的：那一形写进判决的交用户表、标零轮，由用户在「补这一形后合」「撤回改法甲」之间定；不再为它开轮。
- 真仓 `crates/` 里今天就有实例的漏连（不论新旧）：合进主工作区之前必须收到，收不到这套规则不合。
- 两份实现给得出不同闭包的输入，坐实的：对齐之前不合。
- 正推腿核不到的数：从判决里撤掉那一句或重量。
- 两条腿都没打中的：记「第三轮一次抽样没打中」，连同前两轮被打中又收严的形态清单一起写进决策正文，不写成「不会漏连」。
