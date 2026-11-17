# 规则压缩复核第三轮（contradictions）正推腿报告

日期：2026-11-16。立场：正推，逐处核矛盾总表第三版。

约定：总表 = `research/prompts/rules-compress-r3-merged/contradictions.md`；合稿树 = `research/prompts/rules-compress-r3-merged/`，树里 `dot-claude/` 即 `.claude/`，`CLAUDE-merged.md` 即 `CLAUDE.md`；原件 = 仓里今天的文件（`judged-sha256.txt` 55 份开工时核过没变，我用 `sha256sum -c` 再核一遍，失败 0 份，量的）。总表里被引的文件只有三份不在那张清单里：`.claude/gate.d/lib-lag-baseline.py`、`.claude/hooks/selftest-registry.tsv`、调度记录（量的，脚本数的）。

## 一句结论

总表第三版还不能原样当换上时的对照表用：四项里引文回查、用户定案归属、一条用户定案落偏、三条新定案没进表，共七处换上之前必须改（见「要改的」）；其余都可以换上之后再改。第二轮修补本身落得对：新补与改过的 34 行（新补 26 行、表头点名更正标签 8 行；总表有两行都叫定义-11）里没有一行「改错了」，但有 6 行的标签、说明或依据与实际不符（入口-4、验证-19、入口-15、定义-13、常驻-22、用户定-20）。

## 三格

| 格 | 回 | 一句话 |
|---|---|---|
| 一 第二轮修补对不对 | 部分成立 | 验证-10 到验证-19、定义-11 到定义-17、入口-12 到入口-16、常驻-20 到常驻-22 对着文件头与原件现读，句子都对；总表自己的标签、说明或依据有 6 处不实（入口-4 与定义-13 的说明、验证-19 与入口-15 的依据、常驻-22 与用户定-20 的标签） |
| 二 修补引出的新问题 | 成立（有） | 编号重复一处（定义-11 两行）；共用约束 `.claude/agent-common.md:11` 把用户定案 25 点名要留的原话改了；回复只写短句一句在合稿树里 0 命中；其余轻 |
| 三 能不能换上 | 不成立（原样不行） | 总表表头回查结果不实；依据栏把主 agent 写给修稿的半句记在用户定案名下；用户定案 27 到 29 在总表之后才记入调度记录，总表没有，而且它们改变总表登记的几句 |

## 要改的

换上之前必须改：

| 编号 | 处 | 改什么 |
|---|---|---|
| 必-1 | 总表表头第 5 行 | 「problems=0」「refs_ok=455」不实：我逐字回查 456 处，455 处对上，1 处不逐字（用户定-14 一行的〔原话〕少了「已有的」「（只缩不涨）」，见项 4）；表头要改成真数，或改那一行的原话 |
| 必-2 | 依据栏归属 | 合稿-1、验证-19、入口-15 的依据，用户定-17、19、20、23 的合稿落点列，记在用户定案名下的半句，原话里没有（见项 2 的表）；调度记录 `records/2026-11-16-三方豁免取消与存量补轮.md:249` 已写明那两句「不是用户定案」，依据栏要改记「主 agent 处置」 |
| 必-3 | 用户定-25 与入口-4 | 用户定案 25 点名三处原话各留；合稿树 `.claude/agent-common.md:11` 写成「自己直接做，不派」，原件第 8 行是「简单任务不派发，自己直接改」；总表入口-4 一行的说明却写「各留原话」，树与表对不上 |
| 必-4 | 缺三条定案 | 调度记录 246、247、248 行的用户定案 27、28、29 不在总表里；27 删掉「那一轮只攻上一轮的修补」、28 定辩方腿在第二轮派、29 改设计推论轮的停法，直接改变总表用户定-2、17、23、26 与验证-19、合稿-1 登记的合稿落点，要加行、改行，树里对应句也要改 |
| 必-5 | 用户定-20 标签 | 「规则与共用约束样本进程那一条改回原判据」不实：合稿树 `.claude/rules/command-safety-by-review.md:9`、`.claude/agent-common.md:67` 比原件多了「只取自己用新会话起的那一个子进程的组、不在循环里逐个发」，是新判据；改标「改了判据」 |
| 必-6 | 定义-13 说明 | 「放回原件的工具名」不实：原件 `.claude/agent-common.md:100` 的直接跑名单带 `task-server/control` 的 install / start / stop / watch，合稿树 `.claude/agent-common.md:95` 没有；说明改成实情，或树里补上 |
| 必-7 | 定义-11 编号重复 | 总表第 78 行与第 118 行都叫「定义-11」（一条是调查员第 6 步改号，一条是弹窗闸词表），照编号查对照表会查到两行；第二条改号 |

换上之后可以再改的在各项表里逐条标注（「后改」）。

## 项 1 第三版新补与改过的条目逐条

判语：改得对 / 改错了 / 标签不对 / 核不动。「现读」写我读到的文件头那一句与合稿树落的那一句；原件两处原话与合稿落点的 文件:行〔片段〕 已由项 4 的脚本逐字回查（行号全对）。

### 验证-10 到验证-19（总表第 108 到 117 行）

| 编号 | 判 | 现读 |
|---|---|---|
| 验证-10 | 改得对 | 门禁文件头「toolchain、proof（整格）、table 各一项」（`.claude/gate.d/formal-verification.sh:51`）；钉全另有驱动：「钉全由 E165 第二段驱动」（`.claude/gate.d/formal-verification.sh:35`）；合稿树「toolchain、proof（整格）、table 各一项」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification-reference.md:138`），钉全句在 :171 |
| 验证-11 | 改得对（后改一处） | 文件头「不是服务分片起的这一道（gate.sh 整轮、手跑）与指定范围（--check gate-verdict）：不起驱动」（`.claude/gate.d/broad-mutation.sh:30`）；合稿树「经服务分片起的那一趟（不带 --check 经统一任务服务交）这一格先起驱动剪枝跑」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification-reference.md:149`）。后改：文件头合并那一步是「合并那一步驱动核账（分片没判到的条判红、一条不现判）」（`.claude/gate.d/broad-mutation.sh:28`），不是剪枝跑，合稿句把计划、各片、合并三步并成「先起驱动剪枝跑」；做法没变，射程读粗 |
| 验证-12 | 改得对 | 文件头「不给就七格按下面的次序全跑」（`.claude/gate.d/formal-verification.sh:16`）；合稿树参考件「格名表（七格）」在 :165，same-origin-counter 在 :170，七格都点到（verus 四格 :167、红例钉表两格 :169、计数器 :170，量的：数了一遍是 7） |
| 验证-13 | 改得对 | 三处各写各的：崩溃枚举用例带 gpu 的「录好的那几条判红、一条都不判」（`.claude/gate.d/crash-point-replay.sh:340`）；产品包对拍「领卡被拒那一格判红、那一趟不起」（`.claude/gate.d/crash-point-replay.sh:351`）；挂特性那一趟「领卡没成的测试目标记本次未跑、这一格判红」（`.claude/gate.d/crash-point-replay.sh:255`）。合稿树「领卡没成的各自怎么办在文件头同一段」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification-reference.md:21`）并逐一写了三支，与文件头一致 |
| 验证-14 | 改得对（门禁真实行为核不动） | 格名表那一行写「不为 0」（`.claude/gate.d/crash-point-replay.sh:17`），后一段写「挂特性的文件跑到加复用 0 条」报本次未跑、不判红（`.claude/gate.d/crash-point-replay.sh:249`），退出码表另写带特性那一格有本次未跑也退 0；合稿树照后一段：「跑到加复用 0 条」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification-reference.md:146`）。文件头自相矛盾，合稿选对了更具体的一段；门禁真跑出什么没跑，核不动 |
| 验证-15 | 改得对 | lkmm 文件头「每份 litmus 两条判定」（`.claude/gate.d/lkmm.sh:14`）；qemu 文件头「漏一道屏障与走页缓存两个对照必须判红」（`.claude/gate.d/qemu-device-streams.sh:5`）；合稿树「herd7 与绑定各一条」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification-reference.md:42`）与「漏一道屏障与走页缓存两个对照红在该红的地方」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification-reference.md:158`） |
| 验证-16 | 改得对 | 文件头五样：判定段（:16）、判器本体（:17）、标记、登记表 excluded（:21）、判器自测（:22），开头「一处调用点算合规，有一样成立就行」（`scripts/crash/cpu-judging-sites-check.py:15`）；合稿树参考件「一样都不成立的判红」并列全五样（`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification-reference.md:154`），常驻稿「合规的位置不止这两样」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification.md:25`） |
| 验证-17 | 改得对（与合稿-7 同一句登了两行） | 文件头「modules 验证包导入崩溃引擎模块的测试文件第一行声明它测哪几个模块」（`scripts/crash/crash-case-check.py:7`）；合稿树「验证包导入崩溃引擎模块的测试文件第一行声明它测哪几个模块」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification.md:23`）。射程收到脚本的射程，标「照脚本改」对；总表合稿-7（第 102 行）与这一行是同一件事，换上时对着查会看到两行 |
| 验证-18 | 改得对 | 文件头「特性里有 gpu 时这一趟的测试参数带 -- --nocapture、不带 --test-threads」（`.claude/gate.d/crash-point-replay.sh:256`）；合稿树两处同列法：「门禁 crash-point-replay 各趟」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification.md:27`、`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification-reference.md:81`） |
| 验证-19 | 句子对，依据栏标签不对 | 合稿树三方规则「开了第二轮的，第二轮与第三轮里至少一轮派一条辩方腿」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/three-way-inference.md:45`，只管设计推论轮）、实现流程规则「开了第二轮的，在第二轮派一条辩方腿复核已有的判决」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/implementation-workflow.md:30`）。依据栏写「用户定案第 23 条：开了第二轮的在第二轮派辩方腿」：第 23 条原话（`records/2026-11-16-三方豁免取消与存量补轮.md:211`）里没有辩方腿；这句话是之后才记入的用户定案 28（`records/2026-11-16-三方豁免取消与存量补轮.md:247`）。设计推论轮那一句（第二轮与第三轮至少一轮）没有任何用户定案，是修稿自己定的 |

### 定义-11 到定义-17（总表第 78、118 到 124 行）

总表有两行都叫「定义-11」：第 78 行是调查员定义「第 7 步」改「第 6 步」，第 118 行是弹窗闸词表。下面分写。

| 编号 | 判 | 现读 |
|---|---|---|
| 定义-11（第 78 行，调查员第 6 步） | 改得对 | 合稿树调查员定义第 6 步就是「派发提示写了「查到就修」的」（`research/prompts/rules-compress-r3-merged/dot-claude/agents/investigator.md:33`），第 15 行指到「第 6 步」；原第 6 步删掉后改号，指得到 |
| 定义-11（第 118 行，弹窗闸） | 改得对（后改：出处词表缩了两样） | 文件头词表「词表只认上面九个」（`.claude/hooks/ask-user-claim-guard.sh:43`），出处里有「research/results/ 下的文件名」（`.claude/hooks/ask-user-claim-guard.sh:25`）与「实测」「量过」「产物」「输出」旁边带数或路径（同文件 :26）；合稿树「词表只认这九个」（`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:104`）对，出处写成「产物行、量过的数」，比原件第 104 行少了「research/results/ 下的文件」与「「实测」「产物」「输出」旁边带数或路径」。读合稿的人少认两种出处，只会多补出处，不会漏判；总表说明写「改回」，实际是缩写 |
| 定义-12 | 改得对 | 写闸文件头「七、会话号与批次代号」（`.claude/hooks/write-guard.sh:39`）在 :41 列了带后缀的批号写法也算代号；合稿树「会话号或批次代号」（`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:96`），字母代号与带后缀的批号两处都写了，「第 N 批」纯批号照留；做法与文件头一致 |
| 定义-13 | 句子对，说明不实 | 合稿树「remind.sh 只给主 agent 直接跑」（`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:102`）与子 agent 一支的出路（:95）都有；但总表说明写「放回原件的工具名」：原件第 100 行直接跑名单带「task-server/control 的 install / start / stop / watch」，钩子库名单里 task-server/control 在主 agent 一支（`.claude/hooks/lib_project_code_runs.py:1108` 的「"task-server/control": ("first"」），合稿树 :95 的括号里没有它；同一名单里的 prune-server/control 也没有。做法差：照合稿树读，会以为 task-server/control 的起停主 agent 也得经服务交 |
| 定义-14 | 改得对 | 交回闸文件头「kb-scribe / experiment-runner：交回正文」（`.claude/hooks/handback-guard.sh:31`）；合稿树「写回员（kb-scribe）与执行员（experiment-runner）报了没数或 0 项的绿行」（`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:103`）；合稿树里「书记员」0 命中（量的，grep 了 dot-claude 整棵） |
| 定义-15 | 改得对 | 证红脚本文件头「--copy 必须是一份副本」（`scripts/mutation/prove-red.sh:171`），同行写「不是这份脚本所在的主工作区」；合稿树「在你改动所在的副本里证红」（`research/prompts/rules-compress-r3-merged/dot-claude/agents/implementation-writer.md:46`）：实现员在副本里改、证红在同一份副本；调查员「查到就修」改在主工作区，第 33 行写了另建带改动的副本再证红 |
| 定义-16 | 改得对（后改两处） | 合稿树「验证改动的命令照第 6 步填它改的那棵树」（`research/prompts/rules-compress-r3-merged/dot-claude/agents/investigator.md:28`）、「证红的 prove-red.sh 不认主工作区」（`research/prompts/rules-compress-r3-merged/dot-claude/agents/investigator.md:33`）与用户定案 19（`records/2026-11-16-三方豁免取消与存量补轮.md:171`「照旧在主工作区改」）一致。后改一：依据点名交回闸 ⑤，但 ⑤ 判的是「implementation-writer / experiment-runner / tooling-writer」（`.claude/hooks/handback-guard.sh:33`），不含调查员，这一支的命名检查没有钩子盯；后改二：「复现与二分的命令 --cwd 填副本」（合稿树第 28 行）把原件 `[--cwd <副本>]` 的可不给写成了必给 |
| 定义-17 | 改得对 | 四处我逐字比对原件整行与合稿整行（python difflib，量的）：三方攻方（原 :30、树 :30）与辩方（原 :33、树 :33）整行逐字相同；实现员原 :59 与树 :70 只差开头条件「派发提示写明「交补丁」…时：」（用户定案 1 一律交补丁，删得对）；执行员原 :36 与树 :42 只差「做，」一字。「合入后验证认这几张登记表、不当判不出测试目标的路径」（`research/prompts/rules-compress-r3-merged/dot-claude/agents/implementation-writer.md:70`）、「写进报告，主 agent 跑」（`research/prompts/rules-compress-r3-merged/dot-claude/agents/experiment-runner.md:42`）都在 |

### 入口-12 到入口-16、常驻-20 到常驻-22（总表第 125 到 132 行）

| 编号 | 判 | 现读 |
|---|---|---|
| 入口-12 | 改得对 | 原件「那是在提醒」「规矩变过了，先读再跑」在 `CLAUDE.md:43`；合稿「对不上就红——那是在提醒」带着「规矩变过了，先读再跑」（`research/prompts/rules-compress-r3-merged/CLAUDE-merged.md:42`），整句逐字放回 |
| 入口-13 | 改得对 | 原件「本项目只接其中一份，不需要知道别的」（`CLAUDE.md:41`），合稿同句同位（`research/prompts/rules-compress-r3-merged/CLAUDE-merged.md:40`），前一句「对外以 -en 为准」与它同一行，先行词不再落到别处 |
| 入口-14 | 改得对 | 脚本文件头退出码表退 2 有几种：「--fingerprint-of 那一趟开跑以来主工作区在它的起点上变了」（`scripts/gate/merge-and-verify.sh:209`），这一种的出路是「出路写整批重交（去掉 --item 与 --fingerprint-of）」（`scripts/gate/merge-and-verify.sh:181`）；合稿技能「第 1 步只重跑红的那一条退 2、说那一趟开跑以来主工作区变了的」（`research/prompts/rules-compress-r3-merged/dot-claude/skills/merge-verify/SKILL.md:14`）点名是哪一种，别的退 2（补丁核不过、用法错）不在这一句里；做法没变 |
| 入口-15 | 句子合理，依据栏错记 | 原件第 6 条「主工作区里没有打了而没验过的补丁，直接回那个会话」（`.claude/main-agent.md:51`）；合稿补「验证命令还没跑完的，先回在改」（`research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:51`）。用户定案 19 原话只有「照旧在主工作区改」（`records/2026-11-16-三方豁免取消与存量补轮.md:171`），没说先回在改、跑完再回已收尾；这半句是修稿为补洞加的，依据栏记第 19 条不实。后改（可留，要登成主 agent 处置） |
| 入口-16 | 改得对 | 合稿「实现员不往拒算表里登记，收的行只走拒算补丁目录」（`research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:68`）；原件第 68 行「实现员不登记，也不替它登记」；细则「理由至少 8 个字，写得出判的依据」搬到技能（`research/prompts/rules-compress-r3-merged/dot-claude/skills/merge-verify/SKILL.md:54`），入口句末写「做这一步之前先读」那份技能；三类罩不到的包、needs-attention、control check 回读等挪走的细则我没逐条找回（总表说明点名五条，抽了拒算那一条核到，其余四条没逐条核，见核不动） |
| 常驻-20 | 改得对 | 用户定案 4：「变异行之外 commit message 也写改坏了哪一行、哪条断言红」（`records/2026-11-16-三方豁免取消与存量补轮.md:121`）；合稿「脚本查不了 commit message 那一样，写不出来就是没做」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/show-me-test-by-review.md:7`），并补没有变异表收的测试（钩子、门禁、非装置脚本）用弄坏开关证红；落点与 `.claude/rules/test-script-discipline.md`「测试脚本自己也要被测试证明有效」那一节一致（该节有「每一处会判红的分支，配一个弄坏开关或一条变异」，原文带逗号，合稿引句无逗号，引的是合稿树里的同一节） |
| 常驻-21 | 改得对 | 原件「用户要求时 =user-request」（`.claude/skills/gate/staged-gate.md:16`）是退役前缀的残句；同一份技能第一节写「交给服务的命令不带 SINGLEFS_HEAVY_TESTS 前缀」（`.claude/skills/gate/staged-gate.md:8`）；合稿「这一条带 --full-gate，钩子弹窗要用户确认」（`research/prompts/rules-compress-r3-merged/dot-claude/skills/gate/staged-gate.md:17`）。仓里别处 `user-request` 命中只剩这一行（grep 技能、规则、范围声明闸、gate-staged.sh，量的） |
| 常驻-22 | 改得对，标签不全 | 原件「show-me-test.md（新测试先证明会红）」（`.claude/rules/test-script-discipline.md:7`）点名上游整份；合稿「show-me-test.md 整份」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/test-script-discipline.md:4`）放回，同时新加项目一侧「新增的测试必须先证明它会红」那一节的指针（总表按需-M4 已登）。标「改回原判据」漏了「另补一处指针」，后改 |

### 表头点名更正标签的几条（总表第 9 行）

| 编号 | 判 | 现读 |
|---|---|---|
| 入口-4（总表第 18 行） | 标签对，说明与树对不上 | 入口一处对：「探索性的 crates/ 改动、定案后主 agent 自己写得清的小处 kb 改动可以自己写」原件在 `.claude/main-agent.md:7`，合稿在 `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:9`，逐字相同，没有「只有」；`CLAUDE.md` 一处对：「修它们本身就是这一轮的出口时，直接改，不先问」在 `research/prompts/rules-compress-r3-merged/CLAUDE-merged.md:21`；共用约束一处落偏：原件「简单任务不派发，自己直接改」（`.claude/agent-common.md:8`），合稿「自己直接做，不派」（`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:11`）。用户定案 25 是「三处原话各留，不合并」（`records/2026-11-16-三方豁免取消与存量补轮.md:213`）；总表同一行说明写共用约束「各留原话」，树里没留。见必-3 |
| 合稿-7 | 改得对 | 标「照脚本改」对，依据同验证-17（`scripts/crash/crash-case-check.py:7`）；与验证-17 同一件事登了两行 |
| 合稿-10 | 改得对 | 原件第 22 行与合稿第 17 行整行逐字相同（difflib，量的）：「一律整份读，不受读入大小的限制」（`.claude/agent-common.md:22`、`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:17`），标「改回原判据」对 |
| 常驻-10 | 改得对 | 钩子 ⑥ 放行行「shell 里同一层先 setsid [--wait] <命令> &」（`.claude/hooks/bash-command-detector.sh:185`）；合稿规则「setsid [--wait] <命令> & 起（不带 -f），同一层紧跟着 名字=$!」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/command-safety-by-review.md:8`）；共用约束样本进程那条保留 `setsid --wait`（`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:67`）。标「照脚本改」对 |
| 常驻-11 | 改得对 | 用户定案 21：「检出卡住的工具（看门狗、钩子）只报不停」（`records/2026-11-16-三方豁免取消与存量补轮.md:199`）；合稿「报告的人或 agent 可以终止它」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/command-safety-by-review.md:18`），去掉了原件「或检出的工具」。依据改引用户定 21 对 |
| 常驻-12 | 改得对 | 合稿「主 agent 这边由看门狗复检、任务卡住由服务判」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/command-safety-by-review.md:17`），与入口「看门狗只报，不停子 agent 也不停进程」（`research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:74`）一致；原件「长活放后台，定期回来复检，不在前台干等」改成了不同的做法，标「改了判据」对 |
| 常驻-14 | 改得对 | 合稿「主 agent 自己就是调度的一方」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/command-safety-by-review.md:31`）并指到第 15 条，入口第 15 条「没有会话认领、或对应的会话已经不在，自己做」（`research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:38`）；主 agent 一侧从报给用户变成自己做，标「改了判据」对 |
| 用户定-20 | 标签不对 | 用户定案 20 原话是「只许对自己起的组发」（`records/2026-11-16-三方豁免取消与存量补轮.md:194`），点名工具员第 5 步照这个写清。工具员第 5 步与放行行标「改了判据」对（合稿 `research/prompts/rules-compress-r3-merged/dot-claude/agents/tooling-writer.md:33`，钩子放行 `.claude/hooks/bash-command-detector.sh:4114` 的「在循环里调也放行」）；规则与共用约束样本进程那一条标「改回原判据」不对：原件「超时与结束时对整组发信号」（`.claude/rules/command-safety-by-review.md:9`）后面没有限定，合稿同一句（`research/prompts/rules-compress-r3-merged/dot-claude/rules/command-safety-by-review.md:9`）后面加了「组号只取自己用新会话起的那一个子进程的」「不对别的进程组发信号，不先挑进程再发，不在循环里逐个发」，共用约束 :67 同样加了，是新判据。另：这一行落点列里 `research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:80` 的「Python 里 os.killpg 在循环里每一轮停自己起的那一组也放行」不是用户定案 20 说的（20 写「循环逐个发……一律拒」），是调度记录里主 agent 的处置（`records/2026-11-16-三方豁免取消与存量补轮.md:226`），行里已写「射程拿不准」，但落点列仍记在用户定名下；放行的依据在钩子代码的注释（`.claude/hooks/bash-command-detector.sh:4114`），不在文件头 ⑥ 那一段，而规则第 9 行说「以钩子文件头 ⑥ 为准」 |

## 项 2 「用户定」一节编号 1 到 26 对调度记录

调度记录里各条的行号：第 1 条在第 118 行，依次 2 到 5 在 119 到 124 行（5 在 124），6 在 125，7 在 126，8 在 127，9 在 136，10 在 137，11 在 138，12 在 139，13 在 150，14 在 151，15 在 152，16 在 153，17 在 169，18 在 170，19 在 171，20 在 194，21 在 199，22 在 200，23 在 211，24 在 212，25 在 213，26 在 214（grep 行首编号，量的，与总表「原话出处」列的行号逐条相同）。27 到 29 在 246 到 248 行，总表没有。

第一张表：总表记的与那一条原话是不是一回事，合稿落点是不是那一条说的。

| 条 | 总表记的 | 合稿落点 | 判 |
|---|---|---|---|
| 1 | 对（只派一个也在副本里改、交补丁） | `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:47`、`research/prompts/rules-compress-r3-merged/dot-claude/agents/implementation-writer.md:15`、`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:109`、`research/prompts/rules-compress-r3-merged/dot-claude/skills/dispatch/SKILL.md:14` 都写「一律」「只派一个也这样」 | 落对 |
| 2 | 对，原话末句「写成一处、另一处指过去」（`records/2026-11-16-三方豁免取消与存量补轮.md:119`） | 三方规则「开不开下一轮」写停法与三轮（`research/prompts/rules-compress-r3-merged/dot-claude/rules/three-way-inference.md:45`），「一条腿只抽一次样」又写一遍：「这一题停，不为它再派同立场腿」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/three-way-inference.md:72`），入口与实现流程是指针 | 大意落对，「写成一处」没做到：停的判据住 :45 与 :72 两处，总表补充（第 180 行）自己认了；第 29 条之后这一句整段要改 |
| 3 | 对 | `research/prompts/rules-compress-r3-merged/dot-claude/rules/command-safety-by-review.md:40`「没给时一律取登记的每台默认重活线程数」 | 落对 |
| 4 | 对；原话「变异行之外 commit message 也写改坏了哪一行、哪条断言红」（`records/2026-11-16-三方豁免取消与存量补轮.md:121`） | `research/prompts/rules-compress-r3-merged/dot-claude/rules/show-me-test-by-review.md:7`、`research/prompts/rules-compress-r3-merged/dot-claude/rules/implementation-workflow.md:9` 「两样都要」 | 落对；另补的脚本、钩子、门禁自证用弄坏开关证红不是第 4 条说的（项 1 常驻-20） |
| 5 | 对 | `research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:68` 与 `research/prompts/rules-compress-r3-merged/dot-claude/agents/experiment-runner.md:39` 写「上下文吃紧或卡住」；括号「（同一步反复做不过去）」是主 agent 在修稿补的，调度记录 200 行自己写了 | 落对 |
| 6、13 | 撤回，标对（第 20 条撤回，`records/2026-11-16-三方豁免取消与存量补轮.md:194`） | — | 对；合稿树对应两句我 grep 过：「逐个按进程号停」「按会话号在 /proc 里认」0 命中 |
| 7 | 对 | `research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:73`「逐条以它的文件头为准」；各钩子的写法细节已下沉成一条一条短句 | 落对；总表标「只改措辞」，实为把原第 82 到 110 行约 16000 字缩成约 7600 字（量的，awk 数字符），删的是拒的写法清单，保留放行写法与出路 |
| 8 | 对 | 变更史格式示例留（`research/prompts/rules-compress-r3-merged/dot-claude/rules/changelog-format.md:11` 起），`fs-design.md` 的 rust 围栏 0 处（grep 量的） | 落对 |
| 9 | 对 | `research/prompts/rules-compress-r3-merged/dot-claude/rules/verification.md:11`「依赖只有 std」；`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification-reference.md:153` 另写「门禁那一格还许 singlefs-format，收严记在欠账」 | 规则两处写只有 std 落对；括号里是对门禁此刻行为的事实陈述，文件头 `.claude/gate.d/checker-independence-and-sync.sh:32` 确写「⊆ {它自己, singlefs-format}」，真；这半句不是第 9 条说的，但不改变做法 |
| 10 | 对 | `research/prompts/rules-compress-r3-merged/dot-claude/agents/three-way-materials.md:29` 保留 quote-rust-items.py；`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:58` 写名单以钩子库的 TEXT_TOOLS_RUN_DIRECTLY 为准 | 落对；钩子库此刻 0 命中 quote-rust-items（grep -c，量的），总表补充已认；钩子改完之前材料员会被项目代码闸拒 |
| 11 | 对 | 各规则开头那句 0 命中（grep 量的）；全量 0 红实测 0 命中 | 落对 |
| 12 | 对 | `research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:83、85、89、90、91、97、98` 七行，各对钩子文件头，我对了 ⑨ ⑪ ⑯ ⑰ ⑱ 与写闸第六道、第十道：⑱ 里合稿写「git restore <路径…>（不带 --staged）」，文件头是「不带 --staged / -S，或带 --worktree / -W」（`.claude/hooks/bash-command-detector.sh:318`），合稿少了后半，后改 | 落对 |
| 14、15、16 | 没改（脚本一侧） | 无文字落点 | 对；第 16 条的「层 0」留在 `research/prompts/rules-compress-r3-merged/dot-claude/agents/implementation-writer.md:51`（受影响的层 0 流），与处置-2 同一处 |
| 17 | 对（「现查坐实就改，再攻一轮」） | `research/prompts/rules-compress-r3-merged/dot-claude/rules/implementation-workflow.md:10` 指到「代码轮与治理文件轮怎么收」；落点列的 :31「现查推翻了打中的那一条：不改，也不为它开一轮」不是第 17 条说的（见第二张表） | 主句落对；第 27 条之后，:30 的「那一轮只攻上一轮的修补」要删 |
| 18 | 对 | `research/prompts/rules-compress-r3-merged/dot-claude/rules/verification.md:38` 等四处；用户要求的那一趟也要确认（`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification.md:38`「包括用户要求的那一趟」） | 落对；24 条改定后是一处写全、别处指过去 |
| 19 | 对 | `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:47`、`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:109`、`research/prompts/rules-compress-r3-merged/dot-claude/agents/investigator.md:23`、`research/prompts/rules-compress-r3-merged/dot-claude/agents/implementation-writer.md:15`、`research/prompts/rules-compress-r3-merged/dot-claude/skills/dispatch/SKILL.md:14` 写「例外」 | 落对；落点列的 `research/prompts/rules-compress-r3-merged/dot-claude/agents/investigator.md:28`（验证命令写在改的那棵树上）是主 agent 处置 10，不是第 19 条说的 |
| 20 | 对 | 见项 1 用户定-20 | 点名的工具员第 5 步落对；规则、共用约束多加的限定是修稿加的，Python 循环里放行是主 agent 处置 11 |
| 21 | 对 | `research/prompts/rules-compress-r3-merged/dot-claude/rules/command-safety-by-review.md:18` | 落对 |
| 22 | 对 | 同 5 | 落对 |
| 23 | 对（「照代码轮收」） | `research/prompts/rules-compress-r3-merged/dot-claude/rules/implementation-workflow.md:25` 起一节；:43 的「设计推论轮」限定；落点列 :32 与 :45 | 「照代码轮收」「打中的现查坐实就改、改完再攻一轮、没新打中就收、最多三轮」落对；:32（第三轮又打中的改了收、写进交用户表、不开第四轮）与 :45（辩方腿第二轮与第三轮至少一轮）不是第 23 条说的，见第二张表；射程把「规则、定义、门禁、脚本」写成「改定义、共用约束、规则、技能、钩子、门禁、scripts/ 下的脚本、统一任务服务与判定库服务」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/implementation-workflow.md:27`），多了共用约束、技能、钩子、两个服务，与 `CLAUDE.md:22` 的治理文件清单一致，第 23 条括号里没有 |
| 24 | 对 | `research/prompts/rules-compress-r3-merged/dot-claude/rules/verification.md:38`、`research/prompts/rules-compress-r3-merged/dot-claude/skills/gate/staged-gate.md:17、18`；run-scope-guard.sh 文件头确写带两个标识的弹窗、不带的不判（`.claude/hooks/run-scope-guard.sh:7`、「不判」那一段在文件头「提交时才许不指定范围地跑」下） | 落对；补充（第 185 行）「跑全量工作区的 gate.sh 属于不带标记」我核到文件头写「不判：……gate-staged.sh、gate.sh……」，与合稿一致 |
| 25 | 对（原话「三处原话各留，不合并」，不写「只有」） | 入口、`CLAUDE.md` 两处落对；共用约束一处没留原话 | 一处落偏，见必-3 |
| 26 | 对 | `research/prompts/rules-compress-r3-merged/dot-claude/rules/three-way-inference.md:72`、:45 | 落对；第 29 条之后 :45 与 :72 的停法要改 |

第二张表：依据栏或落点列记在某条用户定案名下、而那一条原话里没有的半句，逐处列出（每一处我都在调度记录那一条的原话里找过，没有）。

| 总表行 | 编号 | 记在哪一条名下 | 那半句 | 实际是谁定的 |
|---|---|---|---|---|
| 96 | 合稿-1 | 第 23 条 | 「现查推翻的不改、不为它开轮」 | 主 agent 写给修稿的要求；调度记录 249 行写「不是用户定案」（`records/2026-11-16-三方豁免取消与存量补轮.md:249`） |
| 96 | 合稿-1 | 第 23 条 | 「第三轮又打中的改了收、写进交用户表、不开第四轮」 | 同上（249 行），第三轮判决的交用户表里列出 |
| 96 | 合稿-1 | 第 23 条 | 「开了第二轮的在第二轮派辩方腿」 | 第 23 条原话（`records/2026-11-16-三方豁免取消与存量补轮.md:211`）没有；之后用户定了第 28 条「开了第二轮就在第二轮派，复核第一轮判决」（`records/2026-11-16-三方豁免取消与存量补轮.md:247`），只管代码轮与治理文件轮 |
| 117 | 验证-19 | 第 23 条 | 「开了第二轮的在第二轮派辩方腿」 | 同上；而且合稿树设计推论轮那一句「第二轮与第三轮里至少一轮」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/three-way-inference.md:45`）没有任何用户定案，是修稿定的 |
| 128 | 入口-15 | 第 19 条 | 调查员例外「在主工作区改了、验证命令还没跑完的，先回在改，命令跑完再回「已收尾」」 | 修稿补洞，第 19 条（`records/2026-11-16-三方豁免取消与存量补轮.md:171`）只有「照旧在主工作区改」与「一律交补丁」的例外 |
| 156 | 用户定-17 落点列 | 第 17 条 | `research/prompts/rules-compress-r3-merged/dot-claude/rules/implementation-workflow.md:31`「现查推翻了打中的那一条：不改，也不为它开一轮」 | 同第 96 行第一半句（249 行） |
| 158 | 用户定-19 落点列 | 第 19 条 | `research/prompts/rules-compress-r3-merged/dot-claude/agents/investigator.md:28`「验证改动的命令照第 6 步填它改的那棵树」 | 主 agent 处置 10（调度记录 225 行），总表主 agent 处置-10 那一行已另登 |
| 159 | 用户定-20 落点列 | 第 20 条 | 「Python 里 os.killpg 在循环里每一轮停自己起的那一组也放行」（`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:80`、`research/prompts/rules-compress-r3-merged/dot-claude/rules/command-safety-by-review.md:9`）；规则与共用约束样本进程条里的「只取自己用新会话起的那一个子进程的组」 | 前者是主 agent 处置 11（226 行，「循环逐个发」的射程以钩子为准）；第 20 条原话是「循环逐个发……一律拒」（`records/2026-11-16-三方豁免取消与存量补轮.md:194`），连 Python 那一种也拒与否是总表自己写的「拿不准」；后者第 20 条只点名工具员第 5 步 |
| 162 | 用户定-23 落点列 | 第 23 条 | 实现流程规则第 32 行的「第三轮又打中的：现查坐实的改了就收，写进判决的交用户表，不开第四轮」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/implementation-workflow.md:32` 同一行），三方规则第 45 行的「开了第二轮的，第二轮与第三轮里至少一轮派一条辩方腿」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/three-way-inference.md:45` 同一行） | 前者 249 行，后者无用户定案（设计推论轮）；治理文件清单扩到共用约束、技能、钩子与两个服务也不是第 23 条括号里的「规则、定义、门禁、脚本」，与 `CLAUDE.md:22` 的治理文件清单一致 |
| 148 | 用户定-9 落点列 | 第 9 条 | 「收严记在欠账」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification-reference.md:153`） | 第 9 条原话是门禁格 ⑤ 收严并进治理文件那一批（`records/2026-11-16-三方豁免取消与存量补轮.md:136`），没写「记在欠账」；欠账号还没登 |
| 47 | 常驻-8 说明 | 第 18、24 条 | 「确认……只写一处」 | 第 18 条写的是「三份统一」（`records/2026-11-16-三方豁免取消与存量补轮.md:170`），第 24 条没写条数；一处写全、别处指过去是总表的压缩口径 |
| 101 | 合稿-6 依据 | 「用户定案删」（不带条号） | 「两处说同一件事以上游为准」删掉 | 调度记录里没有哪一条用户定案删它：只有主 agent 现查写的一句：合稿那份规则的第 4 行多了「两处说同一件事以上游为准」，原件 0 命中（`records/2026-11-16-三方豁免取消与存量补轮.md:181`）；这一行的原件栏〔无〕还会在 `.claude/rules/code-discipline-by-review.md:66` 里命中「几乎无开销」的「无」字，回查按「找得到」算，等于没回查 |

没有这类问题的条（原话与记录对得上）：用户定-1、3、4、5、7、8、10、11、12、21、22、24、26，入口-1、入口-4（只就归属说）、常驻-11、常驻-13、合稿-5。

第三张表：第 27 到 29 条（总表之后才记入调度记录，`records/2026-11-16-三方豁免取消与存量补轮.md:246` 到 249）与总表已登句子的冲突。

| 条 | 原话要点 | 与总表登记的合稿落点冲突处 |
|---|---|---|
| 27 | 攻上一轮的修补，加前一轮写明只抽查过、没看过的地方；已攻过而没打中的角度不重复；删掉修稿写的「那一轮只攻上一轮的修补」 | `research/prompts/rules-compress-r3-merged/dot-claude/rules/implementation-workflow.md:30` 仍写「那一轮只攻上一轮的修补」；总表用户定-17、23 行的落点句要连带改 |
| 28 | 代码轮与治理文件轮的辩方腿：开了第二轮就在第二轮派，复核第一轮判决 | 实现流程规则 :30 对；三方规则 :45 写「第二轮与第三轮里至少一轮」（管设计推论轮，28 没管到）；总表合稿-1、验证-19 依据栏要改记第 28 条 |
| 29 | 设计推论轮停法：「两条腿都没打中就停」只管第一轮；第一轮有打中的结论攻满三轮，第二轮没打中也照开第三轮 | `research/prompts/rules-compress-r3-merged/dot-claude/rules/three-way-inference.md:45` 写「前两轮不一致、无法收敛的开第三轮」，:72 写两条腿都没打中就停、没限第一轮；总表用户定-2、26 两行与补充（第 180 行）要连带改 |

## 项 3 标「只改措辞」「改指针」的行全部过一遍

范围：总表里标签恰好是「只改措辞」（43 行）或「改指针」（27 行）的 70 行（脚本数的），逐行对原件整行与合稿整行读过（原件与合稿的整行我用 dump 脚本取出看，不只看片段）；标「改指针（腿打中，现核后改）」的处置-6 另算，没逐句读。

动了做法的，列出来（5 行：入口-4、用户定-25 是同一处，另常驻-15、入口-3、入口-8）：

| 编号（总表行） | 原件 | 合稿 | 动了什么 |
|---|---|---|---|
| 入口-4 与用户定-25（18、164） | 「简单任务不派发，自己直接改」（`.claude/agent-common.md:8`） | 「自己直接做，不派」（`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:11`） | 用户定案 25 点名原话各留，共用约束一处改了字：「改」变「做」，并和派发的几条合进同一行；做法上「做」比「改」宽（查询、计划也算），总表同一行说明写「各留原话」不实。必-3 |
| 常驻-15（54） | 「第一段排他新建，回复只写短句」（`.claude/rules/three-way-inference.md:164`，派云端腿的提示里要写明的一句） | 合稿三方规则「云端腿的报告要分段落盘」那一节「第一段排他新建」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/three-way-inference.md:77`）后面没有「回复只写短句」，整棵 dot-claude 里 0 命中（grep，量的） | 提示里要写明的一句丢了；腿交回写什么由共用约束「交回只写文件路径、sha256sum 与判定一览」管（`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:121`），腿照样守，但主 agent 写派发提示时不再被提醒写这一句。总表只登了交回内容照共用约束，没登这一句。后改 |
| 入口-3（17） | 「fmt、clippy、build」（`.claude/main-agent.md:118`）与第 7 行的「fmt、clippy」 | 入口「编一次，只跑补丁改到的测试目标」（`research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:57`）不再写 fmt、clippy、build；技能「脚本编译之后另跑 fmt、clippy、build」（`research/prompts/rules-compress-r3-merged/dot-claude/skills/merge-verify/SKILL.md:18`）有 | 常驻读到的入口少了「合入后验证还跑 fmt、clippy、build」；做这一步前要读技能（入口句末写了），做法不变，常驻稿的可读性变了。后改 |
| 入口-8（22） | 「两版封存当历史，不再改」（`CLAUDE.md:22`） | 加「（除非要查来历，别读它们）」（`research/prompts/rules-compress-r3-merged/CLAUDE-merged.md:21`） | 新增一条读取限制，总表标「只改措辞」；依据是上游规则的「指向历史的链接要带劝阻句」，不是原件的话。后改 |

其余 65 行做法没动，抽几类说明（逐行读过，不全列）：

- 只改指针、指针指到真有那句话的地方：常驻-1 到常驻-6、常驻-16、常驻-19、按需-M2、M4 到 M10、合稿-3、合稿-12、定义-2、定义-7、定义-9、入口-6、入口-9、入口-10，原件指针指不到或指错，合稿改到真有的节；常驻-19 把「它认 ### 日期 下写成（其 <序号>）、新增撞号就红」换成指上游「8.」，上游那一节同样写撞号由门禁判（`.claude/singlefs-ai-sop/rules/kb-discipline.md:87`）。
- 只改措辞：入口-2、5、7、11、12、13、14、16，验证-6、8，常驻-9、18、21，共用-1、2、6、7、8、9，定义-3、4、5、6、8、10、14、17，按需-M1、M3、M12、M15、M16、M17，合稿-2，用户定-7、8、11，处置-8、9：说法换了、判据没动。其中按需-M15 原件那句「判法比之前把」读不通，合稿改「判法比对前把」是猜：门禁文件头写「链接目标里的 .en.md / .ja.md 不归成 <名>.md」是弄坏开关（`.claude/gate.d/doc-text.sh:344`），与合稿一致，猜得对但原件那一句本身是错字。
- 按需-M1：原件处置「留下」改「改写」，写法不变（总表自己写明）；`.claude/rules/code-discipline-by-review.md:77` 那一行的「按一件能单独验证的事切，不按行数」在合稿同条保留。
- 共用-7：原件「Write / Edit 由 write-guard.sh 当场拒」改「写进仓里的，写闸当场拒」，不再点名工具与脚本名；写闸的钟点与会话号那一道只判仓里的（文件头「仓外（草稿目录、报告目录）不判」在 `.claude/hooks/write-guard.sh:32`），读起来更准，做法没动。

## 项 4 表头回查「rows=118 users=26 refs_ok=455 problems=0」

我自己写了只读脚本 `/home/fy5090/.local/share/singlefs/work/rules-compress-contradictions-r3-sonnet/recheck.py`（python3 -I，只读仓与合稿树），对总表三张表全部 `文件:行〔片段〕` 逐字回查：原件栏与调度记录栏读仓里今天的文件，合稿栏读合稿树（.claude/ 换 dot-claude/、CLAUDE.md 换 CLAUDE-merged.md）；表格按「不在 〔〕 与反引号里的竖线才切格」切，片段里带竖线的（按需-M1）不会切错。输出原样：

```
rows=156 users=26 refs=456 found=455 notfound=1 line_off=0 multi_hit=33
== 找不到 ==
(153, '用户定-14（用户定）', 1, 'orig', 'records/2026-11-16-三方豁免取消与存量补轮.md', '149-151', '`nproc` 存量：取并行度缺省的那几处登进 `.claude/heavy-thread-exempt` 逐批改', '片段在文件里 0 命中')
== 找得到但行号对不上（片段在别的行）==
```

- 总表三张表共 156 行：第一张 118 行（与表头 rows=118 对上）、用户定 26 行、主 agent 处置 12 行；引文 456 处，455 处在所引文件里逐字找到，且都在写的那一行（line_off=0，量的）；1 处找不到。表头写 refs_ok=455、problems=0：数对上 455，但漏掉了 1 处不对的，不是零问题。
- 找不到的那一处：总表第 153 行用户定-14 的〔原话〕写「`nproc` 存量：取并行度缺省的那几处登进 `.claude/heavy-thread-exempt` 逐批改」，调度记录那一行（`:151`）是「`nproc` 存量：取并行度缺省的那几处登进已有的 `.claude/heavy-thread-exempt`（只缩不涨）逐批改」，少了「已有的」与「（只缩不涨）」。意思没变，但「引号里只放逐字原文」没做到，表头的「problems=0」不实。
- 回查力度不足的一处：合稿-6（总表第 101 行）原件栏〔无〕，是一个字的片段，在 `.claude/rules/code-discipline-by-review.md:66` 里命中「几乎无开销」的「无」，回查通过但那一行本来就是要说「原件整份没有这句」，通过没有意义。
- 有 33 处片段在所引文件里不止出现在一行（multi_hit），行号都对，所以不影响结果，但写手的脚本「取离写的行号最近的一处命中」会盖住行号错；这一次我把行号也核了，全对。
- 总表依据栏里不带〔〕的 文件:行 共 42 处，文件都在、行号都不越界（量的）；抽查 9 处（task_server_lib.py:2084、write-guard.sh:31 到 32、completion-cleanup-guard.sh:104、admission.py:163 到 164、lib_project_code_runs.py:1091 到 1094、handback-guard.sh:44 到 45、stage-cells.sh:52 到 56 等）内容与依据说的相符，表头说这些行号换上时以文件头为准，我认同。

## 核不动的

| 处 | 为什么核不动 |
|---|---|
| 验证-13、验证-14 的门禁真实行为（挂特性的文件跑到 0 条整道退 0、领卡没成各支的判红与否） | 我只读了 `.claude/gate.d/crash-point-replay.sh` 的文件头与退出码表；要起门禁、造样本才能证行为，我不交任务，写成「待主 agent 跑」，命令见下一节第 2 条 |
| 用户定-20 的 Python 循环里 os.killpg 放行 | 依据在钩子代码注释（`.claude/hooks/bash-command-detector.sh:4114`），文件头 ⑥ 里没有；这一句真被钩子放行要喂钩子才知道，没喂（命令见下一节第 3 条） |
| 用户定-24 服务端对提交之外带 --full-gate 的命令收不收 | 修稿报告拿不准第 2 条自己写没核；我只核到 `.claude/hooks/run-scope-guard.sh:7` 文件头写钩子弹窗，没有读服务端那一半 |
| 入口-16 说明里「五条判据各收成入口或技能一处」 | 我只逐字对了拒算罩不到那一条与拒算补丁细则；needs-attention 不是结局、control check 回读、--check-facts、调查员例外四条没逐条找回合稿落点 |
| 常驻-12 依据里「任务卡住由服务判（文件头「错误类型」表的 stalled 一行）」 | 那张表在 `task-server/task_server_lib.py` 的 2072 到 2144 行之间一段模块级说明里（stalled 在第 2141 行附近），不在文件最开头的模块文档串里；原件共用约束就这么写，我没把它当错，只记一笔 |
| 合稿树里挪走的细则是否都能经「先读技能」读到 | 派发提示说这一格归别的腿（入口与常驻腿），我只在三处抽查（入口-3、入口-16、常驻-15）看过 |

## 要主 agent 跑的命令

1. 项 4 的回查脚本（只读，我已直接跑过一遍；主 agent 复跑取数）：
   - 命令：`task-server/task submit --cwd /home/fy5090/code/singlefs -- python3 -I /home/fy5090/.local/share/singlefs/work/rules-compress-contradictions-r3-sonnet/recheck.py /home/fy5090/code/singlefs research/prompts/rules-compress-r3-merged research/prompts/rules-compress-r3-merged/contradictions.md`
   - 目录：仓根。
   - 看哪几行：任务末行 `TASK_SERVER_RESULT` 的 `exit=0`；输出第一行应是 `rows=156 users=26 refs=456 found=455 notfound=1 line_off=0 multi_hit=33`，「找不到」一节只有总表第 153 行用户定-14 那一条。脚本 sha256 `6f1076b0937856fc02fad6cd84a659796e179b0ce4512623634456cf860d28ee`，我这一遍的输出 sha256 `8f0855da0f79a362d36ff98187a16dfbfc48de11be245620708af1b295da2f3d`。
2. 待主 agent 跑：验证-14 的门禁真实行为，需要主 agent 判要不要起。我写不出一条不起整道门禁的命令（`crash-point-replay.sh` 的 feature-gated-tests 格带点名项属于指定范围，要范围声明与项名，我没有查到哪个挂特性的测试文件会在没有 GPU 的机器上跑到 0 条），所以标「待主 agent 跑」，不给假命令。
3. 待主 agent 跑：用户定-20 钩子行为，要用钩子自证里那一格「python -c 对 start_new_session=True 起的子进程的进程组 os.killpg 与 os.kill(-组号)」（钩子文件头弄坏开关一段点名，`.claude/hooks/bash-command-detector.sh:604`）；起法照钩子自证入口，我没有写出整条命令，不给假命令。

## 什么现象会推翻这些结论

- 「新补与改过的条没有一条改错」：主 agent 若在同一份文件头里找到一处我读成「一致」而其实不一致的段（我读的是钩子、门禁、脚本文件头今天的原文，工作区版本，不是 HEAD），该条改判。
- 「总表表头回查不实」：复跑脚本时「找不到」一节为空，则我判错；脚本对竖线的切法有误时 notfound 会多出，不会少。
- 「用户定案归属记错」：调度记录 246 到 249 行之后若还有别的行写明那几句是用户原话，第二张表对应行撤回。
- 「共用约束 11 行落偏」：用户若说「自己直接做」就是原话的意思，必-3 撤回（仍是总表说明与树不一致，要改一头）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 项 1 验证-10 到验证-19 | 改得对 9 条，标签不对 1 条（验证-19 依据） | 句子都对着文件头；验证-11 后改（合并那一步是核账）；验证-14 门禁真实行为核不动 |
| 项 1 定义-11 到定义-17 | 改得对 7 行，说明不实 1 行（定义-13） | 定义-11 编号重复；弹窗闸出处词表缩了两样（后改）；定义-13 没放回 task-server/control |
| 项 1 入口-12 到入口-16 | 改得对 4 条，依据错记 1 条（入口-15） | 入口-15 补的半句不是用户定案 19 说的 |
| 项 1 常驻-20 到常驻-22 | 改得对 2 条，标签不全 1 条（常驻-22） | 常驻-22 另补了一处指针，标「改回原判据」漏了 |
| 项 1 表头点名更正标签 | 对 6 条，不对 2 条（入口-4 说明、用户定-20 标签） | 合稿-7、合稿-10、常驻-10、常驻-11、常驻-12、常驻-14 标签对 |
| 项 2 用户定 1 到 26 | 落偏 1 条（25）；2、9、17、19、20、23 六条的落点列记了不是原话的半句（第 2 条另有没做到「写成一处」）；其余 19 条落对（含撤回 2 条、没改 3 条） | 25 的共用约束一处落偏；半句列在第二张表；27 到 29 总表缺，必-4 |
| 项 3 只改措辞与改指针 70 行 | 做法没动 65 行，动了 5 行 | 动了的：入口-4 与用户定-25（同一处）、常驻-15、入口-3、入口-8 |
| 项 4 回查 | 表头不实 | 456 处引文，455 处对上，1 处不逐字（用户定-14，总表第 153 行）；合稿-6 的回查没有意义 |
| 总判 | 原样不能当对照表 | 必改 7 处（必-1 到必-7），后改的在各表里标；改完这 7 处可以换上 |

## 没做什么

- 不替主 agent 采纳或出判决；没往统一任务服务交任务、没跑门禁与钩子自证，判决要引的数（项 4 的 455 与 456）由主 agent 照「要主 agent 跑的命令」第 1 条复跑。
- 没读禁读清单里的任何文件（攻方腿这一轮的报告与模型目录、草稿目录，别的几题第三轮各腿的产出）。
- 没判总表该不该这样定（用户定案只核落没落对）；没判合稿树里不在总表登记范围内的句子，只在与总表登记的句子相撞时指出（第三张表）。
- 项 3 的 70 行我逐行读了原件与合稿整行，但没有逐行贴出，只列了动了做法的与抽的几类；处置-6（改指针，腿打中、现核后改）没读。
- 项 1 里依据栏不带〔〕的 42 处 文件:行 只核了文件在、行号不越界，抽了 9 处读内容，没有逐处核内容（总表自己说换上以文件头为准）。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/rules-compress-contradictions-r3-sonnet` 里只有三份脚本与输出（`recheck.py`、`dump.py`、`recheck.out`），共 24K，没有仓副本与编译目录，不需要主 agent 删仓副本。
