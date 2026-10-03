# test-script-align-r1 正推腿（Sonnet）报告：T1、T4

日期：2026-10-01。批次：`records/2026-10-01-对接上游测试脚本规范.md#2`。立场：正推——从上游规则与本仓规则逐句推；欠账每一笔照「怎么拦」写出一个会红的输入。
范围：只判 T1、T4；T2、T3 归攻方腿，不在这里判。重型测试一样没跑。探针与复跑命令在 `research/prompts/test-script-align-r1-sonnet-model/rerun.sh`（只读 grep 与 git show）。

读过的 `crates/` 与事实（方案按 `crates/` 今天的实现来谈）：
- `crates/singlefs-checker-tier/Cargo.toml` 的 `[features]`、`tests/` 顶层七份挂特性的文件（下面 T4 C604 列行号）、`src/crash_identity.rs` 的 `reuse_key`、`crates/mutations.tsv`（1525 行变异；开 `gpu` 的 12 行）。
- `crates/singlefs-checker-tier/tests/common_gpu_adapter/` 与 `decide_gpu_test_start`：今天 `crates/` 里没有（`ls` 报不存在、`grep -rn decide_gpu_test_start crates` 零命中）。凡是「补丁里才有」的东西，下面都标「补丁未合入，只能按字面推」。

## 一、T1：规则去重、进度间隔、删「夜间」

### T1-1 本仓删掉的句子，每一句去向哪里

旧文用 `git show HEAD:<文件>` 取（工作区已改，旧句不在工作区文件里，所以下面的旧句都是转述，附取旧文的命令）。

| 被删的旧句（转述） | 去向 | 判定 |
|---|---|---|
| `verification.md`「定义与名字」旧句：文件名「不带里程碑、步号、增补号、并行线号、欠账号」。命令：`git show HEAD:.claude/rules/verification.md \| grep -c '并行线号'` 输出 1；工作区同命令 `grep -c '并行线号' .claude/rules/verification.md` 输出 0 | 上游那一节只说「用例名……不带里程碑、步号、批次这类来历」；本仓留下的例子只剩里程碑序数、step_four、supplement_two、c519 四种 | **落空一句**：判它的脚本仍判并行线号，规则正文不再提它。见 T1-1a |
| 同一句后半：「来历写进文件头的文档注释」 | 上游「来历进 commit message 或项目 kb」 | 方向相反，两边冲突，本仓删掉本仓那半句是对的；但存量与脚本出路仍按旧句，见 T1-1b |
| 「修完代码也只重跑红的那几条」（`git show HEAD:.claude/rules/verification.md \| grep -c '修完代码也只重跑红的那几条'` 输出 1，工作区 0） | 上游只有「红了只重跑红的那几条」，没说修完代码之后 | 低：上游罩住大意，没罩住「修完之后」这半句；整份重跑归提交时整轮门禁那半句本仓还留着 |
| 「红的是耗时用例也跑得到」（`--include-ignored` 的理由） | 上游「用例原子化」没有这条；本仓只留了「一律带 `--include-ignored`」 | 低：命令还在，理由没了；不影响照做 |
| 「按参数循环、每一轮新建一个池……拆成一个带参数的函数加每个取值一条 `#[test]`，红了只重跑那一条」 | 上游「一条用例一个场景。按参数循环、每一轮重建被测状态的，拆成一个带参数的函数加每个取值一条用例」 | 落了；本仓留下的是「每一轮新建一个池」的本仓认法与标记名 |
| `implementation-workflow.md` 多线程旧句「输出与线程数 = 1 时逐字相同，并且在同一份代码上核过一次」（`git show HEAD:.claude/rules/implementation-workflow.md \| grep -c '在同一份代码上核过一次'` 输出 1，工作区 0） | 上游「多线程与单线程的对拍逐字相同，进快档」 | 上游更严（要进快档）。落了，但「进快档」这一半本仓没有任何一处判它，见 T4-缺口 3 |
| 旧句「每片报区间与已跑的数，长用例的日志一直在涨」 | 上游「单项跑得超过项目定的进度间隔的，每过一个间隔至少打一行进度」加本仓新加的 5 分钟 | 落了，且更具体 |
| 旧句「做不到……报错退出，不退回少一台或 CPU」（崩溃放量那一行） | 上游「不悄悄退回 CPU 或单机」；本仓留下退 78 / 248 的分法 | 落了。分法与上游有一处出入：上游 78 的例子含「内核编不过、显存不够」，本仓把「内核展开超预算」归 248（`crash-amplification-item.sh` 文件头第 15 行）。上游允许别的拒绝形态另用退出码、在文件头列明，且本仓文件头已列，所以不算冲突，只是边界要靠人读 |
| 旧句「重跑同一条命令就是续跑（同一判法版本下判过的块复用，判法版本变了旧行留着、这一版从头判）」 | 上游「判定可复用」几条加 `crates/singlefs-checker-tier/src/verdict_store.rs:23` 的模块注释 | 落了（在代码注释里，不在规则里） |
| `CLAUDE.md` 旧句「门禁、样本与脚本自证归提交时……改完一样都不跑」 | 新句「登记为 light 的改了就跑，登记为 heavy 的归提交时」 | 落了，但新句比 `agent-common.md` 少一类，见 T1-5 |

#### T1-1a 并行线号落空（会红的输入）
- 判它的脚本：`research/scripts/crash-case-check.py:192` 含「(^|_)parallel_line_(one|two|three)(_|$)」。
- 规则正文今天的写法：`.claude/rules/verification.md:18` 写「本仓判的形态是以里程碑序数起头的、`*_step_four_*`、`*_supplement_two_*`、`*_c519_*`」。
- 输入：在 `crates/singlefs-harness/tests/` 放一份 `parallel_line_one_x.rs`。脚本判红，规则正文没有一个字说得出它为什么红。
- 推翻它的现象：`grep -rn '并行线号\|parallel_line' .claude/rules CLAUDE.md` 在规则里命中任一处。今天命中 0（命令见 rerun.sh 第 1 段）。

#### T1-1b 来历写进文件头：规则改了，欠账 C610 的现状栏跟着过期
- `research/scripts/crash-case-check.py:286` 现在写「来历进 commit message 或 .claude/kb/」，本批已改掉旧出路（`git diff HEAD -U0 -- research/scripts/crash-case-check.py` 两行加两行减）。
- `.claude/kb/checks-owed.md:481` C610 却仍写「`research/scripts/crash-case-check.py` 的 file-names 格出路还叫人把来历写进文件头」：这一句今天对不上工作区，提交后会成为一句假话。
- 推翻它的现象：`grep -nF '来历写进文件头' research/scripts/crash-case-check.py` 命中（今天 0 行，见 rerun.sh）。

### T1-2 指向的节名在副本里是不是逐字存在

命令：rerun.sh 第 2 段（脚本读 `.claude/singlefs-ai-sop/rules/test-script-discipline.md` 的九个二级标题，扫仓里每一处「`test-script-discipline.md`」后紧跟的「…」）。原样输出末行：`total 20 bad 1`。
- 20 处里 19 处逐字存在；1 处不是：`.claude/kb/checks-owed.md:478` C607 写「用例原子化」，副本标题是「用例原子化：每一条都能单独跑」（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:7`）。截短写法，`cite-check.py` 的小节写法认不出，也无法 grep 到。
- 被判的规则文件里的指向（`verification.md:18/24/27/35`、`implementation-workflow.md:72`、`CLAUDE.md:33`）全部逐字存在。
- 推翻它的现象：副本改了标题（上游下一版重命名节）。副本版本戳今天仍是 0.0.62，同步之后要重跑这一段。

### T1-3 「用户要求时」之后，还有没有一处按夜间安排或互相矛盾

命令：rerun.sh 第 3 段（`grep -rnI '夜间'` 扫 `.claude`、`CLAUDE.md`、`research/scripts`、`crates`，去掉副本目录与 `decisions-history.md`）。原样输出共 6 行：
- `.claude/gate.d/doc-decisions.sh:1348`（注释里引 D13 旧句）
- `.claude/gate.d/fixtures/doc-text.sh/prime-marks-green/setup.sh:19` 与 `.claude/gate.d/fixtures/doc-text.sh/clock-times-green/setup.sh:19`（样本数据，拿「全量由用户要求或夜间跑」当一句「不是钟点」的例句）
- `.claude/hooks/write-guard.sh:581`（同一个例句的自证用例）
- `.claude/kb/decisions/13-验证路线.md:196` 与 `:326`（定案说明，写「删掉「夜间」」）

规则、定义、共用约束、技能、钩子正文、门禁阶段的出路文字、研究脚本里没有一处再按夜间安排：`.claude/gate.d/checker-tier-layer0-replay.sh:616` 已写「要跑全量（用户要求时）」。没有一处与「全量只在用户要求时」互相矛盾。分项判定：
1. 样本数据里的「夜间」是有意留着的（拿它验钟点判法不误伤），不算残留。
2. **一处过期引文**：`.claude/gate.d/doc-decisions.sh:1348` 的注释引「一个不可复现的夜间失败价值接近 0，甚至为负，它训练人忽略红灯」并说它在 D13 已定项 9 里；该句在 `.claude/kb/decisions/13-验证路线.md:186` 现在写成「log2(N) × 一趟全量的挂钟」那一段里的「全量档失败」。只是注释，不影响任何判定；但按「引一句要与原条一字不差」它已经对不上。
3. **一处隐含冲突（没改字，改了意思）**：`.claude/kb/checks-owed.md:23` C20 的怎么拦写「定期全量扫（不挂在提交上）」，要求每套布局「至少每 N 天被完整跑过一次，最久未跑的那套超期即判红」。全量既然只在用户要求时跑，这一格超期之后只会红、没有任何一处会去跑它。这条不是这一批引入的，但这一批把「夜间」这个唯一的定期时机删了，它从「靠夜间跑兜」变成「只能靠用户开口」；C20 的现状栏没有跟着写。
4. D13「合并前 = 全量档全绿（准入条件）」（`.claude/kb/decisions/13-验证路线.md:184` 一带）今天没有任何触发者：`grep -rn '合并前' .claude/rules .claude/agents .claude/main-agent.md .claude/agent-common.md` 零命中。夜间原本也只是纸面时机（仓里没有定时任务：`grep -rniI 'cron\|定时' .claude/gate.d research/scripts` 命中 4 份文件：mutate.sh 与 corruption-check.py 是子串误撞、detached-run.sh 与 agent-watch.py 讲的是看门狗的定时回报，没有一处是跑全量的定时任务），所以不是新漏洞；但变更史标题写「准入语义不变」（`.claude/kb/decisions-history.md:6005`），结论写「全量档仍是异步准入判据」（`.claude/kb/decisions-history.md:6006`），这两句对「谁来触发」都没有交代。交主 agent 判是否值得补一句。

推翻 2 的现象：`doc-decisions.sh:1348` 引的整句在 13-验证路线.md 里 `grep -nF` 命中；推翻 3：C20 现状栏出现「全量不再定期跑」之类的交代。

### T1-4 新加的进度间隔 5 分钟

- 落点：`.claude/rules/implementation-workflow.md:73` 写「**进度间隔**：5 分钟。」，后一句「单项跑得超过 5 分钟的，每过 5 分钟至少打一行进度」。上游句子在 `.claude/singlefs-ai-sop/rules/test-script-discipline.md:74`「单项跑得超过项目定的进度间隔的」，要求「每过一个间隔至少打一行进度」，两边接得上，数字只有一个来源。
- 仓里别处的数：`research/scripts/e165-replay-mutation-kill-bounds.sh:120` 与 `research/scripts/broad-mutation.py:154` 都写 300 秒，对得上；`research/scripts/agent-watch.py:33` 的「10 分钟无动静」是看门狗的，间隔比 5 分钟长，不冲突。
- 没有落地的几处不在 T1：`crates/singlefs-checker-tier/src/crash.rs:3169` 的 `LAYER0_PROGRESS` 每片一行、不按挂钟（C611 讲它，见 T4）。
- 推翻它的现象：规则正文里另有一个不同的进度数（今天 `grep -n '进度间隔' .claude/rules/*.md` 只命中这一处）。

### T1-5 定义与共用约束改后的字面（只列与 T1「互相矛盾」那一问相关的，字面会不会做错归 T2）

三处写「脚本自证与门禁样本」谁跑谁不跑，口径不一：
- `./CLAUDE.md:22`：「登记为 light 的改了就跑，登记为 heavy 的归提交时」。只有两类，没有「没登记的」。
- `.claude/agent-common.md:87`：「登记为 heavy 或没登记的照旧归提交前」。多一类「没登记」。
- `.claude/agents/tooling-writer.md:34`：「登记为 heavy 或没登记的写明命令交主 agent」。
- 登记表 `.claude/hooks/selftest-registry.tsv:6` 另有第三类「heavy-if-set:<变量>」，表里 `awk` 数出 light 72、heavy 3、heavy-if-set 1、samples-light 10、samples-heavy 3、samples-light-marker 3（rerun.sh 第 4 段）。三份文字都没点名 heavy-if-set 与 samples-* 各自算哪一边。
- 输入（假设的）：某个子 agent 改了 `research/scripts/layer0-shard-run.sh`（登记表 `.claude/hooks/selftest-registry.tsv:51` 登记为 heavy-if-set）。按 `CLAUDE.md:22` 它既不是 light 也不是 heavy，字面读是「没说」；按 `agent-common.md:87` 它不是「heavy 或没登记」，也不算 light，同样「没说」。
- 推翻它的现象：三份文字改成同一个口径并点名 heavy-if-set，或登记表头注释被其中一份指过去。

## 二、T4：欠账 C604–C613、C620 的「怎么拦」与 D13 改写

做法：每一笔先现查「要拦什么」栏里的事实在工作区还对不对，再把「怎么拦」栏推成一个会红的输入，判能不能落成。行号都是 `.claude/kb/checks-owed.md` 里那一行所在的行（C604=475 … C613=484，C620=491，用 `grep -n '^| C60[4-9] ' …` 取）。**一个总的发现先说：这一批同一趟里 tooling-writer 已经落了 C604、C606、C611、C612 的脚本那一半，这四行的「要拦什么」栏还按落地前（HEAD）写，提交之后会有几句假话**，逐条见下。

### C604（475）快档不开特性
- 事实：三份文件的 cfg 行号对得上：`crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs:6`、`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:9`、`crates/singlefs-checker-tier/tests/crash_facts_gpu_verifier_agrees_with_the_cpu_reference.rs:5`；`grep -n '^#!\[cfg(' crates/singlefs-checker-tier/tests/*.rs` 共 7 份挂特性的文件。
- 会红的输入：在 `crates/singlefs-checker-tier/tests/` 放一份 `#![cfg(feature = "gpu")]` 的测试文件，而 54 号快档两趟都没点它。**这个格已经落了**：`.claude/gate.d/checker-tier-layer0-replay.sh:9` 写「gate-cell: feature-gated-tests」，从 `tests/` 顶层 `#[cfg(…)]` 现读特性、挂特性的文件逐个点名（`:70` 起的注释）。读得到就点得到，不用抄名单，输入可行。
- 现状栏过期：栏里「一格门禁……没有就判红」那一半今天已有；还欠的只剩「没卡时 GPU 对拍报本次未跑」，而 `grep -rn NOT_RUN_NO_GPU_ADAPTER crates` 输出 0 行：门禁认它，`crates/` 里没人打它（补丁未合入）。今天在有卡的这台机器上 GPU 用例照真卡跑；没卡的机器上开 `gpu` 特性仍是 panic 退 101，这一格会红，不是假绿。
- 结论：能落成，门禁半已落；该行应改成「已落：feature-gated-tests；仍欠：GPU 没卡打 NOT_RUN（crates）」。

### C605（476）复用与剪枝没有关掉再判的对照
- 会红的输入：栏里写「快档用例：同一小输入开、关复用与剪枝各跑一次，逐态判定相同；变异：复用键少罩一项输入，那条用例红」。
- **跨趟复用那一半：同一输入开、关各跑一次，抓不到「复用键少罩一项输入」**：跨趟读到的是上一趟存下的判定；上一趟与这一趟若是同一份输入，库里存的就是它自己的判定，键漏不漏都读到同一个值，逐态相同，变异存活。要红，库里必须先存着一份「与这一份只差被漏掉那一项」的输入的、真判定与这一份不同的判定。上游 `.claude/singlefs-ai-sop/rules/test-script-discipline.md:63` 只写「剪枝要自证：在小规模输入上关掉剪枝与复用跑一遍」，没说怎么造这种输入。
- 会红的输入（跨趟那半，修正后）：先拿输入 A 填库，再拿「与 A 只差一项被键漏掉的输入」B 在同一个库上跑，与 B 关复用的结果逐态比；键少罩那一项，B 读到 A 的判定，两边不同，红。前缀共享那半不受此限：同一趟里两个只差被漏掉那一项的点会被并成一个，开关各一次够用，前提是输入里真有这样一对点（`crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs` 的两条流尾巴同名不同数据，是现成的一对候选，没有核过它们差的是不是键里的那一项）。
- 前置栏缺一句：关复用与剪枝的开关今天要先加。`grep -rn 'no_reuse\|without_reuse\|reuse_off\|disable_reuse' crates` 零命中（只列了这四种名字，不排除别的叫法）。键层已有一条用例：`crates/singlefs-checker-tier/src/crash_identity.rs:1092` 的 `fn each_attribute_changes_the_reuse_key() {`，两项属性各改一次键要变；`crates/mutations.tsv` 里没有一行点它的名字（`grep -c each_attribute_changes_the_reuse_key crates/mutations.tsv` 输出 0）。所以变异那半在键层上今天加一行就够，不用等新用例。
- 结论：跨趟复用那一半的「怎么拦」两句互相够不着（用例抓不到变异），要改写；前缀共享那一半可用；前置要补开关。

### C606（477）入口不写展开深度
- 事实：`crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_rollback.rs:31` 与 `…_through_unmount.rs:38` 仍是 `const DEFAULT_EXPAND_UP_TO: usize = 6;`（crates 补丁未合入）。
- 栏里四个症状（默认深度 6、exhaustive=false 照样退 0、变量写错退回默认、直接带 `SHARE=i/n` 只判一部分照样绿），「怎么拦」栏只有三句，**没有一句对着第四个症状**：栏里写的是「入口不带参数就按全量的展开深度跑；ITEM 行带 `expand_up_to=` 与 `exhaustive=`，不带参数的那一趟有一项 `exhaustive=false` 就判红；规模变量的值认不出判红、不退回默认」。入口那一头已经认 `share=<i>/<n>` 的行（`research/scripts/crash-amplification.sh:49`、`:56`），但这在脚本注释里，行里没有。
- 会红的输入：入口不带参数跑，假驱动对一项打 `exhaustive=false`。入口侧已落（`research/scripts/crash-amplification.sh:112` 写 `FULL_EXPAND_UP_TO=28`，`:46` 的注释写判红条件）；Rust 侧（规模变量写错判红、GPU 那一行写展开深度）补丁未合入，按字面只能推到这里。
- 现状栏过期：前置栏「全量的展开深度登记在哪一处（今天只在用户定案与调度记录里）」今天不对，登记在入口 `:112`。

### C607（478）随机取样点用例凭种子复现不出
- 会红的输入：栏里写「每种随机用例在快档里有一条自检：拿它判红时打出的复现行原样重跑，逐态与原趟相同，不同判红」。输入：让一份随机用例打出缺「比重」的复现行，原样重跑，逐态不同，自检红。
- 缺的前置：要「判红时打出的复现行」得有一个判红，快档里没有红可用，要给四个随机二进制各加一个注入失败的入口（或弄坏开关）；前置栏写的是「无」。
- 另：栏里 `.claude/singlefs-ai-sop/rules/test-script-discipline.md`「用例原子化」是截短的节名（见 T1-2）。

### C608（479）GPU 变异不罩内核
- 事实全对：`awk` 数出开 `gpu` 的变异行 12 行（`crates/mutations.tsv` 1461–1475 间）；`grep -c wgsl crates/mutations.tsv` 输出 0；这 12 行改的宿主文件只有 `crash_judge_gpu.rs`、`crash_judge_dispatch.rs`、`crash_amplification.rs`，没有 `crash_verify_gpu.rs`、`gpu_unit_checks.rs`。
- 会红的输入：变异表头注释（`crates/mutations.tsv:2`）写「原文在文件里必须恰好命中一次」，是纯文本替换，不限扩展名，所以 `.wgsl` 可以进表：把 `crates/singlefs-checker-tier/src/crash_judge_common.wgsl:381` 的 `fn version_of_location(location: u32) -> u32 {` 的函数体改成只比低 32 位，点名 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state` 要红。
- 前置栏已写 C601：它说这一改在 σ 对拍上今天照样绿，所以这一行变异在 C601 的合成输入合入之前会「没红」；没卡的机器上这行不会假绿，会被门禁认成 `NOT_RUN_NO_GPU_ADAPTER`（本批新加的认法）。本机有两块 NVIDIA（`/sys/bus/pci/devices/0000:01:00.0` 与 `0000:10:00.0` 的 vendor 是 0x10de、class 0x030000、driver nvidia），所以在这台机器上能真红。
- 结论：能落成，前置齐；要补一条：这些行只在有卡的机器上算数，门禁里该写明没卡时这几行记「本次未跑」而不是「没红」。

### C609（480）全量出的红没有统一的钉表
- 会红的输入：一份 54 号 `--full` 的产物里有一条 `test … FAILED` 行，钉表里没有「整行抄下、带产物文件名」的对应行，这一格要判红。栏里的匹配规则可以直接用上游 `.claude/singlefs-ai-sop/rules/test-script-discipline.md:89`「全量产物里的每一个红，在钉表里都找得到对应的那一行，才算关掉」：按整行文本找。
- 栏里没写、落成检查前要先定的三件：① 找哪一份产物（`research/results/` 里同一道门禁有多份，取最新还是全部）；② 变异整表的红是「没抓到」而不是一个测试失败，QEMU 的红是设备流不一致，它们的「出红的那一行」是什么、钉回快档钉成什么，栏里只写了崩溃枚举这一形态；③ 上游 `:87`「钉完核一次：在出红的那份代码上」，栏里的怎么拦只覆盖「整行抄、带产物文件名、改回哪一处它就红」，没有「在出红的那份代码上，钉住的用例同样红、红在同一条断言上」，E165 那一格的逐行重放是它的现成形态（`.claude/gate.d/e165-pinned-red-examples.tsv` 头部写「⑥ 点名的「改回去就红」的变异要还在它的表里」）。
- 前置栏写「无」不准：E165 钉表自己离上游「整行抄」还差一步——`.claude/gate.d/e165-pinned-red-examples.tsv` 头注释仍是「八段」，而 `records/2026-10-01-对接上游测试脚本规范.md:238` 记着「第 1681 行把钉表段数定死为 8」、crates 一半（补丁）没合入。要把这张表的形态推到别的慢档，先要它自己合上游，这一层没有一行欠账。
- 结论：能落成，但「怎么拦」只够崩溃枚举一类；前置要补 H6（见缺口 6）。

### C610（481）测试文件头还写着来历
- 会红的输入：栏里写「测试文件头的文档注释里出现日期加「定」「用户」这类来历词就判红」。输入：给任一测试文件加一行 `//! 用户 2026-10-02 定：……`，门禁红。
- 命令（rerun.sh 第 5 段）实测今天的量：`crates/*/tests/*.rs` 里写日期的 `//!` 行 87 行、分布在 65 份文件；其中日期之后跟着「用户」或「定」（栏里说的形态）的 68 行、55 份文件；严格写成「用户 <日期> 定」（中间可夹空格与「同日」）的 47 行。
- **两个方向都不准**：假红——68 行里 23 行同一行还带 `已定项`、`D<数>（`、`C<三位数>`、`E<数>` 这类 kb 指路（例：`crates/singlefs-checker-tier/tests/crash_enumeration_position_addressed_trees.rs:2` 的「D8（核心索引结构） 已定项 14，用户 2026-09-24 定 K1–K4」），这是指向条款，不是来历叙事，规则「来历进 kb」恰好允许文件头指过去；假绿——87 行带日期的行里 14 行既没有「用户」也没有「定」，来历叙事用别的词写（例：`crates/singlefs-harness/tests/acquisition_refuses_a_device_whose_witness_slots_turn_unreadable_and_the_remaining_writer_panics.rs:1` 的「实审 A3c（规格 `/tmp/claude-1000/impl-rev-a3c/spec.md`，写于 2026-09-27）」）。判的对象只能是「叙事」，字面词表判不出叙事。
- 现状栏过期：同 T1-1b，脚本出路已改。
- 结论：「回扫一次」那半能做；门禁那格按栏里的词表落成会既红到指路行、又放过「写于 / 第三批 / 实审」这类写法，要换判据（例如只许文件头出现 kb 编号而不许出现日期）。

### C611（482）长跑的项不按挂钟打进度
- 事实对不上工作区：栏里「入口把一项的整段输出装进变量、跑完才转打（`item_output="$(…)"`）」今天已改：`grep -n 'item_output=' research/scripts/crash-amplification.sh` 零命中，`research/scripts/crash-amplification.sh:33` 写「输出：每项的驱动输出边跑边转打，同时 tee 进这一项自己的文件」，`:647` 是 `| tee -- "$item_output_file"`。
- 仍欠的两处：`CRASH_AMPLIFICATION_BLOCK` 每判完一块打一行（crates 补丁未合入）；`crates/singlefs-checker-tier/src/crash.rs:3169` 的 `LAYER0_PROGRESS` 是收到一片才打一行，不按挂钟。
- 会红的输入：栏里写「一条快档用例用假时钟核「超过间隔必有一行」」。输入：假时钟走 301 秒、这期间没有任何一片完成，要有一行进度。这是**可落成的**，而且仓里有现成形态可抄：`research/scripts/e165-replay-mutation-kill-bounds.sh:1083` 用 `Progress(3, PROGRESS_INTERVAL_SECONDS, clock=lambda: clock[0], sink=sink)` 注入假时钟与假出口；`crates/` 里的发射点现在是「收到片才打」，要先改成能被时钟驱动，这一条栏里没写。
- **记账没记全**：`records/2026-10-01-对接上游测试脚本规范.md:240` 写「这些并进 C611（长跑的项不按挂钟打进度），进第二波」，所指是：E165 第二段驱动、`broad-mutation.py`、形式化验证 pinned-red-examples-replay 格（`capture_output`）、包装每 60 秒的排队行被收 stderr 的调用方吞掉、54 号 `--full` 只转发 `LAYER0_PROGRESS`、crates 变异整表一格 6–8 分钟、QEMU 跑完前不打；`:494` 又把「54 号 `--full` 转发的进度行只认 `LAYER0_PROGRESS`、QEMU 跑完前不打、crates 变异整表格内不打」记成「进下一批」。C611 这一行（`.claude/kb/checks-owed.md:482`）一个字没提它们。其中 E165 与 `broad-mutation.py` 今天脚本里已有 300 秒（`research/scripts/e165-replay-mutation-kill-bounds.sh:120`、`research/scripts/broad-mutation.py:154`）；pinned-red-examples-replay 的 `capture_output` 今天还在：`research/scripts/e165-pin-red-examples.py:388` 与 `:401`（它们跑多久没量，是不是超过 5 分钟没核）。
- 结论：能落成；要把 `:240`、`:494` 点名的几处写进这一行，现状栏改到工作区的样子。

### C612（483）变异自证把包装的排队时间算进 2 秒限时
- 事实对不上工作区：栏里的行号今天都指到别处——`research/scripts/mutate.sh:131` 现在是一行 `if [[ ",$enabled," != *",$feature,"* …`，不是 `MUTATE_TIMEOUT=2`；`:572` 是一个 `}`。限时已改交包装：`research/scripts/mutate.sh:49` 写「每条变异限时 MUTATE_TIMEOUT（正整数秒，默认 120），交给包装的 RUN_WITH_MEMORY_CAP_TIME_LIMIT」，`:678` 注释写「不在包装外面套 timeout」，`:340` 起有 `queued-admission` 一例，用 `RUN_WITH_MEMORY_CAP_SELFTEST_ADMIT_PAUSE`，每例各用自己的 `case_directory`（`:168`）。
- 栏里「怎么拦」的三句（限时用包装的、各用例各用一份临时目录、造包装排队 2 秒仍判对）三句今天脚本里都有对应；本行等于已落地。是否能当「已还清」由主 agent 带前缀复核，我没有跑 `mutate.sh --selftest`（登记为 light，可以在草稿仓拷贝上跑，但这一批主 agent 已跑过两份并行都退 0，报在正文第二节）。
- 另有一句没落地且没记账：`records/2026-10-01-对接上游测试脚本规范.md:494` 写「`mutate.sh` 不分类包装的 252/254 结局」。`grep -n '252\|254' research/scripts/mutate.sh` 零命中（只有 250、251、253 三个常量，`:84`–`:86`）。包装退 252（排不上）时命令一行都没跑，输出里没有 `running N test`，走到 `:716` 一带「变异导致编译失败，本条无效」那一支，报成无效变异，不是假绿，但是误导。这一条没有 C 编号（缺口 7）。

### C613（484）自证登记表靠读代码判轻重
- 会红的输入：栏里写「跑登记成 light 的自证时，在它的进程树里查有没有起 cargo、rustc、qemu、herd7、verus 这类可执行文件，起了就判红并点名那一行」。
- **按字面落成会同时假红与假绿**：
  1. 假红：登记为 light 的自证大量用「名叫 cargo」的假工具。`research/scripts/mutate.sh:12` 写「拿一个假的 cargo 走一遍各种结局」，`:96` 的注释写「自证：假的 cargo 按被测源码里写的 BEHAVIOR 演一种结局，不编译任何东西」；54 号样本目录里有 `.layer0-sample-tools/cargo`（登记为 samples-light-marker，`.claude/hooks/selftest-registry.tsv` 里那三行）。按可执行文件名认，这些全红；要按「`/proc/<pid>/exe` 解到的真路径是不是工具链里的」认，栏里没写。
  2. 假绿：「在进程树里查」是采样，一个 50 毫秒的 `cargo --version` 采样间隔漏掉就绿；用绝对路径起的（`~/.cargo/bin/cargo`）同样要靠采到。确定的做法是把 PATH 前面放一个只记录的桩目录、加上起进程的系统调用跟踪，两样栏里都没写。
- 输入（要同时判对的一红一绿）：红——登记为 light 的一份假脚本，`--selftest` 里 `~/.cargo/bin/cargo --version`；绿——`mutate.sh --selftest` 起它自己写的假 cargo。落成之后两个都得判对。
- 结论：「怎么拦」写得出会红的输入，但按栏里的字面落成不可靠；要先定「怎么认真工具」。

### C620（491）GPU 对拍认「没卡」只看 Vulkan 枚举
- 事实：栏里点名的 `decide_gpu_test_start` 与 `common_gpu_adapter/mod.rs` 今天 `crates/` 里没有（`ls crates/singlefs-checker-tier/tests/common_gpu_adapter` 报不存在，`grep -rn decide_gpu_test_start crates` 零命中）。栏里没写「`crates/` 里没有 X」加 grep（`.claude/rules/implementation-first.md`「规矩」第 2 条），读者核不到。
- 会红的输入：一条单测喂「PCI 有卡、Vulkan 零个」要判红，纯函数形态，补丁合入后可落成。
- 栏里「或 `nvidia-smi`」这半**与它要拦的故障相反**：驱动坏了、枚举不出时，`nvidia-smi` 同样报错，用它判「有没有卡」恰好得出「没卡」，就是 C620 要拦的那个误判；独立的证据只有 PCI。
- PCI 厂商号 0x10DE 单查会把同一块卡的音频功能也数成一张卡：本机 `/sys/bus/pci/devices` 里 vendor 为 0x10de 的有 4 个：两个 class 0x030000（显卡，driver nvidia），两个 class 0x040300（HDA 音频，driver snd_hda_intel），数卡数时翻倍，也会数到别的 NVIDIA 非显示设备。要加 class 前缀 0x03 的条件（读每个设备目录下的 class 文件）。
- 结论：能落成，栏里补 class 过滤、去掉「或 nvidia-smi」、补 grep 零命中。

### D13「夜间档」改「全量档」有没有顺带改掉别的意思

逐处比对（`git diff HEAD -U0 --word-diff=plain` 在 `.claude/kb/decisions/13-验证路线.md`、`06-快照实现模型.md`、`16-发布语义.md`，命令在 rerun.sh 第 6 段）：
- 纯改名：已定项 9 索引行、层 0 与层 2 两格触发、「夜间档红了」改「全量档红了」、已定项 15 索引行与定案里「由用户要求或夜间跑」改「由用户要求时跑」、D6 一行与依据、D16 净代价那句。
- 不止改名的两处，**变更史都写明了**（`.claude/kb/decisions-history.md:6006`：「的「一天」是夜间节奏的产物」、「全量档提交时不默认跑、不限时刻，由用户要求时跑，也算准入判据」）：
  1. `.claude/kb/decisions/13-验证路线.md:186` 把「修一个夜间档 bug 的挂钟 ≈ log2(N) × 一天」改成「log2(N) × 一趟全量的挂钟」。单位变了：旧的是夜间节奏，一天一跑；新的是一趟全量的挂钟，今天那一趟是七个多小时（`CLAUDE.md` 里写的数）。这是改了量，不是改了名，但依据成立、变更史交代了。
  2. `.claude/kb/decisions/13-验证路线.md:183` 小标题从「只能夜间跑的也算准入判据，但只能是异步准入判据」改成「全量档提交时不默认跑、不限时刻，由用户要求时跑，也算准入判据，但只能是异步准入判据」。结论没变（还是异步准入判据）；前提从「只能夜间跑」（能力限制）换成「提交时不默认跑」（安排），为什么不能同步放进提交的理由仍在同一节的射程里（`:189`「不再是「写请求数两位数」的冒烟体量，所以提交时默认只跑快档」）。
- 要连同一起答的那一句没动：`.claude/kb/decisions/13-验证路线.md:184`「提交前 = 快速档全绿（必要条件）；合并前 = 全量档全绿（准入条件）。」整行在；`:185`「全量档红了，处置不是回退，是「之后所有提交都带着一个未结的红」直到修掉。」在。
- 没有别的意思被改：`git diff` 这三份文件里属于这一批的词级改动逐处都在上面；`.claude/kb/decisions/13-验证路线.md:196` 与 `:326` 两条新加的定案行只是指路。D16 那一份 diff 里同时有另一件事（C314 题面改写），不属于这一批，没判。
- 一个没改字、但读者会问的点：「合并前 = 全量档全绿」今天没有触发者，见 T1-3 第 4 条。
- 推翻它的现象：`git diff` 里属于这一批的词级改动出现上面没列的一处。

### T4-缺口：这一批该记而没记的

每一条给「会红的输入」与已有的记账位置。

1. **崩溃放量入口对「取不出复用数」永远不红，驱动行以后少打字段也不会有东西响。** `research/scripts/crash-amplification.sh:62` 写「有一行取不出，这一项记 reused=unknown、计进 TOTAL 行的 reuse_unknown，不记 failed、不改退出码」，`:84` 的弄坏开关 `accept-unknown-reuse` 证的是「取不出不被记成 0」，不是「该有时必须有」。`grep -rn states_reused crates` 今天零命中，所以补丁合入前每一项都是取不出，退 0。输入：合入后某天驱动行被改名成 `reused_states=`，入口退 0、成功句写「复用数取不出」，没有一格红。落成要一个棘轮：驱动合入 `states_reused=` 之后，有复用层的项取不出判红。记账位置：`records/2026-10-01-对接上游测试脚本规范.md:100`（派件表一行），没有 C 号。
2. **C611 没把记录里「并进 C611」的几处写进去**：见 C611 一节，出处 `:240`、`:494`。
3. **上游「一律进快档」那几类（多线程与单线程对拍、优化前后对拍、慢档判法的小输入、钉回的红例）没有逐类核，也没有 C 号。** 出处 `records/2026-10-01-对接上游测试脚本规范.md:74`（H3「每一种在不在快档没逐条核」，去向写「等调查交回（G7、G9），缺的记欠账」）；G7、G9 回来只落成了复用/剪枝（C605）与 GPU（C604、C608、C620）。上游 `.claude/singlefs-ai-sop/rules/test-script-discipline.md:94` 把「一律进快档的那几类对拍在不在快档里」列为项目可接的本地门禁，`:52` 写「档位标在用例上或登记表里，不在脚本里另抄一份名单」。输入：一条「多线程与单线程逐字对拍」的用例被标 `#[ignore]` 进了慢档，今天没有任何一格红；落成前置是先有一张「一律进快档的类」的登记表（今天没有）。
4. **「文件头列明的每个弄坏开关打开之后自证红在点名的那一项」没有门禁、没有 C 号。** 上游 `:94` 列为可接的本地门禁；`records/2026-10-01-对接上游测试脚本规范.md:107` 的 F2 盘点写 54 份文件的开关文件头缺「打开后哪一项该红」，派给第二波；本仓 `.claude/gate.d/code-tooling.sh` 管的是门禁阶段的「开关 → 样本」，研究脚本与钩子的开关没有同类格。输入：一份文件头写「X_BREAK=foo 时必须判红」而不点名哪一项的脚本，今天没有格拦。
5. **走加速硬件的计算类登记（H9）没有 C 号。** `records/2026-10-01-对接上游测试脚本规范.md:238` 写「H9 整件延后进第二波……仓根模板不在 `tooling-writer` 的写范围里，谁来写交用户定」；上游 `:79` 要求登记「哪几类计算走加速硬件、用哪几台机器、各给多少额度」，本仓 `gpu-cards.env` 只登记显卡先后与额度。输入：往 `crates/` 加一类新走 GPU 的计算，登记里没有它，今天没有一格红。
6. **E165 钉表整行抄（H6）没有 C 号，而 C609 的前置栏写「无」。** `.claude/gate.d/e165-pinned-red-examples.tsv` 头注释仍是八段，`records/2026-10-01-对接上游测试脚本规范.md:238` 记着装置把段数定死为 8、crates 补丁未合入。
7. **`mutate.sh` 不分类包装 252/254 结局**：`records/2026-10-01-对接上游测试脚本规范.md:494` 记「进下一批」，没有 C 号，影响见 C612 一节。
8. **五行现状栏与工作区不符**（C604、C606、C610、C611、C612，每一行的具体句子见上）：写进 kb 的「现状」要么在提交前改到工作区的样子，要么写上「已落：…；仍欠：…」。

## 三、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T1-1 删句去向 | 一句落空、几句低风险 | 并行线号的例子只剩脚本在判，规则里没有；来历写进文件头的旧句删得对，但 C610 现状栏与它对不上 |
| T1-2 节名逐字 | 19 处存在、1 处不是 | `.claude/kb/checks-owed.md:478` 的「用例原子化」是截短，副本标题带「：每一条都能单独跑」 |
| T1-3 夜间 | 无按夜间安排的残留 | 6 行命中里 3 行是有意的例句、2 行是定案说明，1 行是一条过期的注释引文，C20 与「合并前」两处是改字之外的隐含冲突，交主 agent |
| T1-4 进度间隔 | 一致 | 5 分钟与 300 秒两处脚本、看门狗 10 分钟都不冲突 |
| T1-5 定义字面 | 口径不一 | `CLAUDE.md:22`、`.claude/agent-common.md:87`、`.claude/agents/tooling-writer.md:34` 对「没登记」与 heavy-if-set 的写法不同（字面会不会做错归 T2） |
| T4 C604 | 能落成，门禁半已落 | 7 份挂特性文件、feature-gated-tests 已在；仍欠 GPU 没卡打 NOT_RUN（crates 零命中） |
| T4 C605 | 要改写 | 跨趟复用那半，同一输入开关各一次抓不到键漏项；前置缺关开关；键层变异一行就能加 |
| T4 C606 | 入口半已落、栏里一个症状无对应句 | `SHARE=i/n` 没有怎么拦；前置栏「展开深度只在调度记录里」已不对（入口 `:112`） |
| T4 C607 | 能落成，前置不是「无」 | 要先给四个随机二进制各加注入失败的入口 |
| T4 C608 | 能落成 | 事实全对（12 行、0 行 wgsl）；本机有卡能真红，没卡记本次未跑 |
| T4 C609 | 只够崩溃枚举一类 | 产物怎么选、变异与 QEMU 的红钉成什么、「在出红的代码上同样红」三句没写；前置要补 H6 |
| T4 C610 | 按词表落成会既假红又假绿 | 68 行日期后跟用户/定里 23 行同时带 kb 指路；87 行日期行里 14 行用别的词写来历 |
| T4 C611 | 能落成，脚本半已落 | 入口半已落；记录点名的几处没进这一行；crates 发射点要先能被时钟驱动 |
| T4 C612 | 脚本已落 | 栏里行号都对不上了；252/254 没分类另无 C 号 |
| T4 C613 | 会红的输入写得出，字面落成不可靠 | 假 cargo 会假红、采样会假绿，「怎么认真工具」栏里没写 |
| T4 C620 | 能落成 | 「或 nvidia-smi」与要拦的故障同向；PCI 要加 class 过滤；crates 零命中没写 |
| T4 D13 | 无多余的意思改动 | 「一天」改量与前提换写都在变更史交代；合并前没有触发者一点隐含 |
| T4 缺口 | 记了 8 条 | 取不出复用数没有棘轮、C611 少记、一律进快档几类、弄坏开关文件头、H9、H6、252/254、现状栏过期 |

## 四、没做什么

- 没判 T2、T3（攻方腿的格）：钩子放行条件能不能被绕过、各入口脚本的假绿假红与退出码表，一格没判；T1-5 只列了三份文字口径不一。
- 没跑任何重型测试、`--selftest`、门禁阶段或脚本；所有「会红的输入」都是从字面与代码推的，没有真的喂进去看它红。能核的事实（行号、命中数、`awk` 与 `grep` 计数）贴在 rerun.sh 里，只读。
- `crates/` 里还没有的东西（`decide_gpu_test_start`、`NOT_RUN_NO_GPU_ADAPTER` 的打印、`states_reused=`、关复用的开关、规模变量写错判红）都按补丁不在树里推，补丁在 `/tmp/claude-1000/impl-*/patch/`，没有读。
- 没核各件的交回报告（`/tmp/claude-1000/tooling-writer-*/report.md` 等）里的自证原样输出，也没复跑主 agent 报的「21 格、283 格 1 格红」那些数。
- 没判「每项结束一行」（项名、退出码、判了多少、红了多少、复用、秒）在 54 号、crates 变异、QEMU 入口上写没写「不复用」：只看了崩溃放量入口。
- C20 与「合并前」没有触发者两点只写了现象，没替主 agent定要不要补句。
- `mutate.sh` 把 252 记成「无效变异」之后，门禁把「无效」当红还是当绿，我没核。

## 五、引文原句（上面各节引到的行，原句在被引文件里现查过，行号是那份文件自己的行号）

规则与上游：
- `.claude/rules/verification.md:18`：「本仓判的形态是以里程碑序数起头的、`*_step_four_*`、`*_supplement_two_*`、`*_c519_*`」
- `.claude/rules/verification.md:24`：「它判红、一条命令都不打时才整份重跑，完整的一遍归提交时的整轮门禁」
- `.claude/rules/implementation-workflow.md:73`：「单项跑得超过 5 分钟的，每过 5 分钟至少打一行进度：每片报区间与已判的数」
- `.claude/singlefs-ai-sop/rules/test-script-discipline.md:14`：「来历进 commit message 或项目 kb」
- `.claude/singlefs-ai-sop/rules/test-script-discipline.md:49`：「跟着全量跑：提交前或用户要求时；别的时机由项目定」
- `.claude/singlefs-ai-sop/rules/test-script-discipline.md:52`：「档位标在用例上或登记表里，不在脚本里另抄一份名单」
- `.claude/singlefs-ai-sop/rules/test-script-discipline.md:63`：「剪枝要自证：在小规模输入上关掉剪枝与复用跑一遍」
- `.claude/singlefs-ai-sop/rules/test-script-discipline.md:73`：「多线程与单线程的对拍逐字相同，进快档」
- `.claude/singlefs-ai-sop/rules/test-script-discipline.md:74`：「单项跑得超过项目定的进度间隔的，每过一个间隔至少打一行进度」
- `.claude/singlefs-ai-sop/rules/test-script-discipline.md:80`：「不悄悄退回 CPU 或单机」
- `.claude/singlefs-ai-sop/rules/test-script-discipline.md:87`：「钉完核一次：在出红的那份代码上」
- `.claude/singlefs-ai-sop/rules/test-script-discipline.md:89`：「全量产物里的每一个红，在钉表里都找得到对应的那一行，才算关掉」
- `.claude/singlefs-ai-sop/rules/test-script-discipline.md:94`：「文件头列明的每个弄坏开关打开之后自证红在点名的那一项」

kb（欠账行很长，只抄栏里与结论相关的整句）：
- `.claude/kb/checks-owed.md:475`：「一格门禁：checker 档里挂特性的测试文件，快档至少有一趟把它编进来，没有就判红」
- `.claude/kb/checks-owed.md:476`：「快档用例：同一小输入开、关复用与剪枝各跑一次，逐态判定相同；变异：复用键少罩一项输入，那条用例红」
- `.claude/kb/checks-owed.md:477`：「展开上限变量值写错时悄悄退回默认；直接带 `SHARE=i/n` 跑、只判了一部分也照样绿（后两样是调查员读代码推的，没跑）」
- `.claude/kb/checks-owed.md:477`：「全量的展开深度登记在哪一处（今天只在用户定案与调度记录里）」
- `.claude/kb/checks-owed.md:478`：「每种随机用例在快档里有一条自检：拿它判红时打出的复现行原样重跑，逐态与原趟相同，不同判红」
- `.claude/kb/checks-owed.md:479`：「没有一行改 `.wgsl` 内核、`crash_verify_gpu.rs` 或 `gpu_unit_checks.rs`」
- `.claude/kb/checks-owed.md:480`：「全量产物里的红逐个在钉表里找得到，找不到判红」
- `.claude/kb/checks-owed.md:481`：「一格门禁：测试文件头的文档注释里出现日期加「定」「用户」这类来历词就判红」
- `.claude/kb/checks-owed.md:482`：「入口边跑边转打；内层按挂钟每 5 分钟至少一行；一条快档用例用假时钟核「超过间隔必有一行」」
- `.claude/kb/checks-owed.md:483`：「限时改用包装自己的 `RUN_WITH_MEMORY_CAP_TIME_LIMIT`（从起跑算、不算排队）」
- `.claude/kb/checks-owed.md:484`：「在它的进程树里查有没有起 cargo、rustc、qemu、herd7、verus 这类可执行文件，起了就判红并点名那一行」
- `.claude/kb/checks-owed.md:491`：「厂商号 0x10DE）或 `nvidia-smi`：PCI 上有卡而 Vulkan 枚举不出判红」
- `.claude/kb/checks-owed.md:23`：「定期全量扫（不挂在提交上）」
- `.claude/kb/decisions/13-验证路线.md:184`：「提交前 = 快速档全绿（必要条件）；合并前 = 全量档全绿（准入条件）」
- `.claude/kb/decisions/13-验证路线.md:186`：「log2(N) × 一趟全量的挂钟」
- `.claude/kb/decisions/13-验证路线.md:196`：「用户定案（2026-10-01）：全量档不限时刻，用户要求时随时跑，删掉」
- `.claude/kb/decisions-history.md:6005`：「夜间档改叫全量档，全量不限在夜间、随时可跑，由用户要求时跑；准入语义不变」
- `.claude/kb/decisions-history.md:6006`：「全量档仍是异步准入判据、仍先保失败可复现」
- `records/2026-10-01-对接上游测试脚本规范.md:100`：「驱动行补 `states_reused=`」
- `records/2026-10-01-对接上游测试脚本规范.md:238`：「第 1681 行把钉表段数定死为 8」
- `records/2026-10-01-对接上游测试脚本规范.md:240`：「这些并进 C611（长跑的项不按挂钟打进度），进第二波」

代码与门禁：
- `crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs:6`：「#![cfg(feature = "verdict-store")]」
- `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:9`：「#![cfg(all(feature = "verdict-store", feature = "gpu"))]」
- `crates/singlefs-checker-tier/tests/crash_facts_gpu_verifier_agrees_with_the_cpu_reference.rs:5`：「#![cfg(all(feature = "verdict-store", feature = "gpu"))]」
- `crates/singlefs-checker-tier/src/crash_identity.rs:1092`：「fn each_attribute_changes_the_reuse_key() {」
- `crates/singlefs-checker-tier/src/crash_judge_common.wgsl:381`：「fn version_of_location(location: u32) -> u32 {」
- `crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_rollback.rs:31`：「const DEFAULT_EXPAND_UP_TO: usize = 6;」
- `crates/mutations.tsv:2`：「原文在文件里必须恰好命中一次」
- `.claude/gate.d/checker-tier-layer0-replay.sh:9`：「gate-cell: feature-gated-tests」
- `.claude/gate.d/checker-tier-layer0-replay.sh:616`：「要跑全量（用户要求时）」
- `.claude/gate.d/doc-decisions.sh:1348`：「一个不可复现的夜间失败价值接近 0，甚至为负，它训练人忽略红灯」
- `research/scripts/crash-case-check.py:192`：「(^|_)parallel_line_(one|two|three)(_|$)」
- `research/scripts/crash-case-check.py:286`：「来历进 commit message 或 .claude/kb/」
- `research/scripts/crash-amplification.sh:33`：「输出：每项的驱动输出边跑边转打，同时 tee 进这一项自己的文件」
- `research/scripts/crash-amplification.sh:62`：「有一行取不出，这一项记 reused=unknown、计进 TOTAL 行的 reuse_unknown，不记 failed、不改退出码」
- `research/scripts/crash-amplification.sh:112`：「FULL_EXPAND_UP_TO=28」
- `research/scripts/mutate.sh:12`：「拿一个假的 cargo 走一遍各种结局」
- `research/scripts/mutate.sh:49`：「每条变异限时 MUTATE_TIMEOUT（正整数秒，默认 120），交给包装的 RUN_WITH_MEMORY_CAP_TIME_LIMIT」
- `research/scripts/e165-replay-mutation-kill-bounds.sh:120`：「PROGRESS_INTERVAL_SECONDS = 300」
- `research/scripts/broad-mutation.py:154`：「PROGRESS_INTERVAL_SECONDS = 300」
- `research/scripts/e165-pin-red-examples.py:388`：「completed = subprocess.run(command, cwd=root, capture_output=True, text=True, env=environment)」
- `.claude/hooks/selftest-registry.tsv:51`：「research/scripts/layer0-shard-run.sh」
- `./CLAUDE.md:22`：「登记为 light 的改了就跑，登记为 heavy 的归提交时」
- `.claude/agent-common.md:87`：「登记为 heavy 或没登记的照旧归提交前」
- `.claude/agents/tooling-writer.md:34`：「登记为 heavy 或没登记的写明命令交主 agent」
