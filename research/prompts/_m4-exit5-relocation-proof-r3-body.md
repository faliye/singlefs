# m4-exit5-relocation-proof-r3：出口 5 全验那一形的最后一轮（2026-10-26）

<!-- doc-lint:not-numbers Z1 Z2 Z3 Z4 Z5 G7 P4 -->

## 一、题面

同一题的第三轮，也是最后一轮（`.claude/rules/three-way-inference.md`「第三轮之后停，不开第四轮」）。第二轮判决 `research/prompts/m4-exit5-relocation-proof-r2-main-verification.md` 第二节给出整形；用户 2026-10-26 定「里程碑四全验」：发布、回退、崩溃后读映射三个闭包的 A、B、C、D 四类函数都在里程碑四由 Verus 直接验 core 的可执行代码。这一轮照「三轮分工」第三轮：「已定」项撑不撑得住、未定项清单的完整性、那一轮报出的数字能不能核。只攻第二轮站住的形态；这一轮新冒出的零轮形态写进判决的交用户表，不再为它开一轮。

## 二、实现今天的样子（主 agent 的观测，2026-10-26）

- 冻结副本：`~/.local/share/singlefs/work/m4-close-rounds-frozen/`（与第一、二轮同一份）；主工作区此刻另有这一批打进去的多 inode 改大小与池级 checker 改动，引它们写主工作区的行号并注明。
- 第二轮两条腿的模型目录：正推 `research/prompts/m4-exit5-relocation-proof-r2-sonnet-model/`（改后协议、区间 P4、core 里接上的第一阶段原型、MIR 比对、std 规格覆盖），攻方 `research/prompts/m4-exit5-relocation-proof-r2-opus-model/`（cfg 分叉、axiom、错规格、区间 P4 与两槽第二槽两个反例）。
- 门禁判法：`scripts/gate/verus-proof-check.py`、`.claude/gate.d/verus-pins.tsv`、`crates/singlefs-replay-prefix/verus-must-red-worlds.tsv`；函数单元图 `crates/singlefs-unit-graph`；判器单元键 `crates/singlefs-checker-tier/src/judging_units_entries.rs` 与 `build.rs`。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### Z1　改后协议与区间 P4 撑不撑得住
在第二轮正推那份改后协议与区间 P4 上复跑第二轮攻方两个反例的变体：搬迁写进回退目标还在读的槽；补了回退视图之后两槽单元的第二槽照样可写。格里要问：这两形在改后协议上是否被排除（给 Verus 的 verified / errors 原样输出）；G7 改写成「回退候选窗口移一格」之后，带回退与抬 F 的历史上「带 GC 与不跑 GC 逐项相同」在那一格的口径下是否成立；改后协议的前提里还有没有一条在 core 今天的代码里不成立（例：`set_reuse_window` 拨到 `ForcedToZero`）。

### Z2　信任清单与门禁的判法撑不撑得住
第二轮定的：去掉 `--no-cheating`；被验区域里每一处 `assume_specification`、`external_body`、`axiom`、`uninterp` 都要在信任清单里有一行（点名的 std 函数、规格原文 sha256、一条差分测试）；vstd 自带的按版本整体登一行；`axiom fn` 一律不许；按 `verus_keep_ghost` 分叉的函数体判红；产品构建与 Verus 的 MIR 逐函数相同进门禁；Verus 与产品的 rustc 版本差进工具链钉表。格里要问：在这一套下还有没有一种写法能让 Verus 验过、产品行为违 ensures 而门禁不红（宏展开、`#[cfg]` 别的原子、`unsafe`、trait 默认方法、泛型单态化之后的差异）；差分测试能不能判出一条规格写错了的 std 函数（给一例）。

### Z3　射程的完整性
第二轮定射程是发布、回退、崩溃后读映射三个闭包的并集。格里要问：出口 5「GC 在抽象文件系统状态上是恒等变换」要成立，还有哪条 core 路径会改抽象状态而不在这三个闭包里（挂载续做整理意图、整理开关的运行期切换、GPU 垃圾回收第 42 批之后的 trait 那一路 `BackgroundGpuCompute`、区域扫描）；列清单与判法（闭包现量命令与原样输出）。

### Z4　判器键与重判代价
第二轮定判器闭包键去掉 Verus 的规格与证明块。格里要问：这样做之后「只改证明、产品代码不变」的改动是不是键不变（在副本上改一处 ensures 量键）；会不会让一处真改了产品行为的改动被当成只改证明（例：proof 块里调了一个有副作用的 exec 函数、`ghost` 变量参与了 exec 分支）；做不到时整库重判的代价按 549 个 fn、崩溃判定库现有块数推一个数。

### Z5　第二轮报的数能不能核
第二轮正推报：闭包 1073 个 fn、24970 行；A 472 / B 190 / C 291 / D 120；std 调用键 456 个、vstd 按名字配上 49%；全验约 11 万–12.5 万行（推的）。攻方报：1071 个 fn、598 个在 impl / trait 里、82 个名字被 274 个 fn 共用、549 个 fn 同在判器闭包里。格里要问：两边 1073 与 1071 差在哪；分类 A–D 的判法照它写的命令现量能不能复出；「11 万–12.5 万行」的推法用的比例（证明行对代码行）取自哪几个现成核（`compaction_plan_proofs.rs`、`gc_crash_proofs.rs`、`replay_core` 那几份），按那几份现数重算一遍。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | Z1–Z5 | 复核第二轮判决第二节各格够不够得着：逐格对判决原文、两腿报告与模型目录，核改后协议对攻方两个反例变体的排除、信任清单判法的漏洞、射程清单的完整性、键的做法与数；判决的结论是不是同样打中被否的「核外提」那一叠；每条结论写文件:行号 |
| 云端攻方（Opus） | Z1–Z5 | 在冻结副本与第二轮两份模型的拷贝上造反例：改后协议上仍成立的反例；信任清单下 Verus 验过而产品违 ensures、门禁不红的写法；三个闭包之外改抽象状态的 core 路径；键不变而产品行为变的改动；第二轮报的数对不上。不许重复第一、二轮已攻过的角度（两轮判决第二节逐格写着）。每个打中给可复现的命令与原样输出 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本或主工作区里的路径与行号，写明是哪一份。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；拷冻结副本，不在冻结副本里改；每棵树各用一个编译目录，跑完删。直接起的 `verus` 与 `cargo verus` 在本机副本里跑（第二台没有 Verus）；门禁 checker-tier-formal-verification 与 `scripts/gate/verus-proof-check.py` 不起。
- 每条编译与 Verus 命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 8G 的上限，同一时刻只起一条 cargo 或 verus；线程上限照派发提示。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 模型与原型放 `research/prompts/m4-exit5-relocation-proof-r3-<模型>-model/`，带 `tools/rerun.sh`（照 `scripts/agents/three-way-rerun-template.sh`），整份复跑超过 240 秒的分段落盘复跑命令。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试，不起 GPU。
