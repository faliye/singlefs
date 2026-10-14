# 第二题第二轮：云端辩方腿（Sonnet）报告

日期：2026-10-13。轮名 `m3-defs-b9-r2`。格：S1–S7。立场：复核第一轮判决 `research/prompts/m3-defs-b9-r1-main-verification.md`——P1–P7 落到冻结副本之后，是不是对第一轮两腿每一条打中都修到了，有没有把第一轮判对的行为改坏，第一轮哪一格判重、判轻。

## 读法与披露

- b10＝`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/`（after-b10-fixup），b9＝`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/`（after-b9-fixup）。两份只读；喂法跑在我从它们各拷出的小树（只含 `.claude`、`scripts`、`records` 与叫停登记、轮名材料；不含 `multi-host.env`、crates）上，每棵 69M，交回前删。
- 重喂的是第一轮两腿模型目录里的喂法：正推 `research/prompts/m3-defs-b9-r1-sonnet-model/` 的六个驱动（线程上限 56 例、长活声明 32 例、重型测试闸 7 例、叫停登记 34 例、脚本文件头 10 例、豁免表 13 例，外加真表真记录），攻方 `research/prompts/m3-defs-b9-r1-opus-model/` 的围栏 19 例、叫停表三种日常改动、doc-text 四格、豁免表重钉、shell 样本。每个喂法对 b9、b10 各跑一遍并排（`out/r1s-*`、`out/r1o-*`）。第一轮攻方的试改树（fix）不重做。
- 我自己的新喂法对应正文「三」的专问，文件名 `out/s1-*` 到 `out/s7-*`；驱动在 `tools/`。门禁格（`--check`）与样本档我没起，小树与命令在 `gate-samples/`，标「要主 agent 跑」。
- 越界披露：定义「开工先读」点名主工作区的 `.claude/agent-common.md` 与 `.claude/rules/three-way-inference.md`，这两份正是被判文件，与禁读清单相抵（第一轮两腿同）；我先读了，之后所有引文、行号、判定一律取自冻结副本，没引主工作区字面。禁读清单里的攻方报告、模型、草稿目录与 `multi-host.env` 的值都没读；样本里的私有值全是我编的（`sample-second-host-77` 一类，对不上任何真机器）。
- S1 真提示语料用了 `~/.local/share/singlefs/work/*/prompt.md` 共 49 份（排除 `m3-defs-b9-r2-opus` 与我自己的），只打汇总数，不打正文。
- 判定词：修到＝第一轮那条打中的喂法，b9 上中、b10 上不中；没修到＝b10 上仍中；改过头＝第一轮判对的行为，b10 上变了。每格另列判重、判轻。「站得住 / 够不着 / 同样打中别的臂」照三方定义用在末尾的判重判轻表里。
- 模型目录 `research/prompts/m3-defs-b9-r2-sonnet-model/`，`out/` 路径下面简称 `out/`；引到的行号写完整路径。

## 总表：判决条目 / 辩方判定 / 证据

喂法列的写法：`b9→b10` 是同一个喂法在两份冻结副本上的退出码或判定。

### S1（P1）认代码块改用 markdown-it

| 判决条目 | 辩方判定 | 证据 |
|---|---|---|
| 攻方 Z1-甲：波浪线块里露出三个反引号、列表项里没闭合的围栏、HTML 注释里的三个反引号，把真「线程上限：16」遮掉（T1、T1b、T2、T3） | 修到 | 派发闸 `--check-prompt` 四例 b9=0→b10=2，`research/prompts/m3-defs-b9-r2-sonnet-model/out/r1o-fence-feed.txt:2`：「T1-tilde-lone-fence-16 investigator b10=2 b9=0」，第 3–5 行同形；钩子 stdin 入口 T1、T2 退出 0→2，`research/prompts/m3-defs-b9-r2-sonnet-model/out/r1o-stdin-feed.txt:2`：「b10 退出=2」、`research/prompts/m3-defs-b9-r2-sonnet-model/out/r1o-stdin-feed.txt:3`：「b9 退出=0」 |
| 攻方 Z6-甲：同一形落在「多机：」（H2）；正推喂法 B 的 tilde、list 两例 | 修到 | H2 b9=0→b10=2，`research/prompts/m3-defs-b9-r2-sonnet-model/out/r1o-fence-feed.txt:13`：「H2-tilde-lone-fence-live investigator b10=2 b9=0」；两例拒→放行，`research/prompts/m3-defs-b9-r2-sonnet-model/out/r1s-long-run-prompt-b9.tsv:23`：「tilde-fence-hosts-example 预计挂钟与长活声明被拒=是」→`research/prompts/m3-defs-b9-r2-sonnet-model/out/r1s-long-run-prompt-b10.tsv:23`：「tilde-fence-hosts-example 预计挂钟与长活声明被拒=否」，第 24 行 list 同 |
| 攻方 Z1-乙：嵌套列表里的围栏示例被判（O2），同形的波浪线（O1）、引用块（O3） | 修到 | 三例 b9=2→b10=0，`research/prompts/m3-defs-b9-r2-sonnet-model/out/r1o-fence-feed.txt:8`：「O2-nested-list-fence-example investigator b10=0 b9=2」，第 7、9 行同形 |
| 正推 Z1-10：「认围栏照 CommonMark」说过头 | 修到 | 56 例里波浪线、列表项、引用块三例拒→放，`research/prompts/m3-defs-b9-r2-sonnet-model/out/r1s-thread-prompt-b9.tsv:40`：「波浪线围栏里的 16 clerk 线程上限被拒=是」→`research/prompts/m3-defs-b9-r2-sonnet-model/out/r1s-thread-prompt-b10.tsv:40`：「波浪线围栏里的 16 clerk 线程上限被拒=否」，第 41、42 行同形；其余 52 例逐行相同 |
| 正推 Z1-03、Z1-07：定义没写围栏种类、⑭ 的「代码块里不判、同一标签第二行拒」 | 修到（文字） | `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/agents/three-way-defense.md:24`：「这三个标签各只写一行，同一标签写第二行拒，代码块（照 CommonMark）里的行不算」；十份定义加共用约束同句（S7 行） |
| 正推 Z1-04：说明行不能自己以「线程上限：」起头 | 没动（第一轮没收进改法也没另立账） | `research/prompts/m3-defs-b9-r2-sonnet-model/out/r1s-thread-prompt-b10.tsv:52`：「普通正文行也以线程上限冒号起头 clerk 线程上限被拒=是」，b9 同；文字仍只写「照普通正文写」 |
| 第一轮判对的其余围栏行为（行内三反引号、缩进 4 格与制表符的围栏不是围栏、信息串带反引号、四个反引号夹三个、没闭合照判、CRLF） | 没改坏 | 56 例其余 52 例、32 例其余 30 例、攻方 19 例其余 11 例 b9 与 b10 逐行相同；CRLF 两例放放，`research/prompts/m3-defs-b9-r2-sonnet-model/out/s1-construct-matrix.txt:24` |
| **改过头（HTML 块）**：HTML 块里紧跟的围栏。`<div>` 后面紧跟三反引号围栏，示例行 b9 放（逐行认法遮了它），b10 拒（markdown-it 认成 html_block，不是 fence） | 改过头（轻，误拒方向）；第一轮没喂过这一形 | `research/prompts/m3-defs-b9-r2-sonnet-model/out/s1-construct-matrix.txt:3`：「HTML 块：<div> 里紧跟围栏 放 拒 html_block」；同形加空行的 `<details>` 放放，第 5 行 |
| **改过头（闭合行前缀）**：闭合行的前缀不按容器看。顶层围栏的闭合行写成缩进 4 格、制表符、或前面带 `> `：CommonMark 眼里围栏没闭合（到文末都是代码），钩子文件头说没闭合的照判，实现却遮了；b9 拒，b10 放 | 实现比文件头宽一格；CommonMark 读者与钩子一致，不当改坏，标出 | `research/prompts/m3-defs-b9-r2-sonnet-model/out/s1-construct-matrix.txt:13`：「围栏的闭合行缩进 4 格 拒 放 fence」，第 14 行制表符同；`research/prompts/m3-defs-b9-r2-sonnet-model/out/r1s-thread-prompt-b10.tsv:49`：「围栏闭合行缩进 4 格 clerk 线程上限被拒=否」。只在未闭合围栏块的最后一行恰好是这种闭合行时才遮：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/hooks/runner-dispatch-guard.sh:1162`：「closing = CODE_BLOCK_CLOSING_PREFIX.sub」 |
| **改过头（库退路）**：库载不进时的退路是一行都不遮，比 b9 更严：b9 认得的最普通的反引号围栏示例，退路里也被拒 | 改过头（轻，仅库载不进时）；试改量过 | `research/prompts/m3-defs-b9-r2-sonnet-model/out/s1-library-missing.txt:2`：「fenced-example PATH=shim 退出=2」（正常 PATH 下同一份放行，第 1 行）；退路改回逐行认法后 `research/prompts/m3-defs-b9-r2-sonnet-model/out/s1-fallback-trial.txt:6`：「fenced-example PATH=shim 退出=0」 |
| **带进来（文字过宽）**：P1 把定义里的「三个反引号围起来的代码块」改成「代码块（照 CommonMark）」，字面比钩子宽：CommonMark 里缩进 4 格的代码块、没闭合的围栏（到文末都是代码）也是代码块，钩子文件头写明它们不遮、里面的示例行照判 | 钩子拒而定义字面读成不判（行为 b9 同，b9 的文字没这个缺口）；站得住与否交主 agent 按跑前条款判 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/main-agent.md:53`：「代码块（照 CommonMark）里的行 ⑮ 不判」；钩子一侧 `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/hooks/runner-dispatch-guard.sh:74`：「到文末没闭合的、在列表项或引用块结束处结束的照判；缩进代码块与 HTML 块」；喂法 `research/prompts/m3-defs-b9-r2-sonnet-model/out/r1s-thread-prompt-b10.tsv:44`：「缩进 4 格的代码块里的 16（前有空行） clerk 线程上限被拒=是」、`research/prompts/m3-defs-b9-r2-sonnet-model/out/r1s-thread-prompt-b10.tsv:43`：「没闭合的围栏后面的 16 clerk 线程上限被拒=是」，b9 同；危害只是误拒（照字面写缩进示例会被拒一次，拒绝说明点名那一行），不涉及用户定案；改法是定义里写「闭合了的围栏代码块（照 CommonMark）」 |

### S2（P2）缺行出路两种写法

| 判决条目 | 辩方判定 | 证据 |
|---|---|---|
| 攻方 Z1-丙：钩子缺行出路只给已判行，与共用约束「只有已判行时子 agent 不起重活」相抵 | 修到（拒绝说明） | b9 的说明：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/hooks/runner-dispatch-guard.sh:1160`：「这件活确要别的数（用户给了参数、内存装不下），或确实不起重活，提示里单起一行」；b10 分两种写：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/hooks/runner-dispatch-guard.sh:1217`：「这件活确要别的数（用户给了参数、内存装不下），写…线程上限：N…再加一行…线程上限已判：<理由>」；两处 15 行放拒结论逐行相同 |
| 正推 Z1-16：`dispatch-prompt.py` 弄坏开关说明里的「书记员」应为 clerk | 修到 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/scripts/agents/dispatch-prompt.py:43`：「预计挂钟表少书记员、前向腿、防方腿」→`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/scripts/agents/dispatch-prompt.py:43`：「预计挂钟表少 clerk、前向腿、防方腿」 |
| 攻方 Z1-丙 的另一半：钩子放而子 agent 按定义读成相反（单独已判行、理由里写要用的数） | 没修到（P2 本意只改文字，钩子仍放；与 b9 同） | `research/prompts/m3-defs-b9-r2-sonnet-model/out/s2-exit-matrix.txt:9`：「单独已判行，理由里写了要用的数（旧出路的写法） 放 放」；结局是子 agent 不起重活、停下交回，白派一次，不违反用户定案；展开见「专问 S2」 |
| 钩子行为有没有被 P2 动 | 没动，没改坏 | 15 个形态 b9 与 b10 的放/拒逐行相同，`research/prompts/m3-defs-b9-r2-sonnet-model/out/s2-exit-matrix.txt:2`：「缺行也没有已判行 拒 拒」，其余行同 |

### S3（P3）`|| X=$var` 的变量来源

| 判决条目 | 辩方判定 | 证据 |
|---|---|---|
| 攻方 Z2-乙：先从登记位取进变量、再 `\|\| X=$var`，照格的出路写仍被点名成写死的默认 | 修到 | 攻方 shell 样本，b9 点名 5 处且点名 `var_from_registry.sh:5`，b10 点名 4 处且不点名它：`research/prompts/m3-defs-b9-r2-sonnet-model/out/r1o-z2-shell-b9.txt:1`：「5 处自己定并行度默认值的写法没经登记位取」→`research/prompts/m3-defs-b9-r2-sonnet-model/out/r1o-z2-shell-b10.txt:1`：「4 处自己定并行度默认值的写法没经登记位取」；b10 样本档 `heavy-thread-defaults-var-source-green`：b9 点名三份、b10 零份，`research/prompts/m3-defs-b9-r2-sonnet-model/out/s3-fixtures-both.txt:15`：「heavy-thread-defaults-var-source-green b9 退出=1 点名：scripts/hosts/var_after_if.sh:4 scripts/hosts/var_from_cap.sh:3 scripts/hosts/var_from_registry.sh:4」→`research/prompts/m3-defs-b9-r2-sonnet-model/out/s3-fixtures-both.txt:16`：「heavy-thread-defaults-var-source-green b10 退出=1 点名：」（退出 1 是我的装配把 `heavy_threads.py` 两行带进来，两边同，不是样本） |
| 第一轮判对的其余钉数写法有没有被放宽 | 没改坏 | 同一批 10 份 heavy-thread-defaults 样本，用这一轮样本目录分别用 b9、b10 的格代码判，点名的文件与行逐个相同，只有 var-source-green 一份变了，`research/prompts/m3-defs-b9-r2-sonnet-model/out/s3-fixtures-both.txt` 其余 9 份（18 行）两两相同；`var-source-red` 四份（字面数、一处登记一处字面、算术、`\|\| X=8`）b10 仍点名，第 17、18 行 |
| P3 带进来的新边界（变量在函数里赋、在 source 进来的文件里赋、赋两次、条件赋） | 够不着的盲区七份，其余按设计 | 二十九份样本 `research/prompts/m3-defs-b9-r2-sonnet-model/feeds/s3/sample/EXPECT.tsv`：该红的十三份 b10 全点名、该绿的八份全不点名；七份「预期＝盲区」的 b10 都不点名而 b9 点名：`research/prompts/m3-defs-b9-r2-sonnet-model/out/s3-cell-b9.txt:2`：「scripts/experiments/s3_arith_spaces.sh:5 写死的线程数默认」，b10 的点名行里没有这七份（`research/prompts/m3-defs-b9-r2-sonnet-model/out/s3-cell-b10.txt:1`：「15 处自己定并行度默认值的写法没经登记位取」，十三份文件）。盲区＝同一个变量在本文件里有一处登记位读法的赋值，另一处用 `(( x = 8 ))`、`eval`、`read`、`printf -v`、`for x in 8`、`x+=0`，或 source 进来的别的文件里把它赋成字面数；格只认 `名=值` 一种写法，见下「专问 S3」 |

### S4（P4）仓副本改过正文的自证

| 判决条目 | 辩方判定 | 证据 |
|---|---|---|
| 正推 Z3-05：规则那一节没写改过正文的自证在副本里怎么跑，按那一节读登记为 light 的 `--selftest` 不算重型、随时可跑，钩子却拒 | 修到（文字） | `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/rules/implementation-workflow.md:54`：「登记为 light 的 --selftest 也按提交时的检查办：子 agent 不在副本里跑，把命令写进交回报告；主 agent 在集成副本里跑，命令带 SINGLEFS_HEAVY_TESTS=commit，不弹窗」 |
| 正推 Z3-04：这一支说明不写前缀、同一条 stderr 末尾的常量却写两种前缀 | 修到（两处同义） | `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/hooks/lib_heavy_tests.py:1447`：「由主 agent 在集成副本里跑，命令带 {HEAVY_TESTS_OCCASION_VARIABLE}=commit、不弹窗」；`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/hooks/heavy-test-guard.sh:837`：「由主 agent 在集成副本里跑，命令带 SINGLEFS_HEAVY_TESTS=commit、不弹窗」 |
| 钩子行为（正推喂法 C 七例）有没有被动 | 没动，没改坏 | 七例退出码两版逐个相同：0、2、2、0、0、0、0（`research/prompts/m3-defs-b9-r2-sonnet-model/out/r1s-heavy-guard-b10.txt:3`：「退出码：0」，第 8、16 行「退出码：2」，其余 0）；变的只有说明文字 |
| 用户定案「仓副本自证收回放宽」有没有被新说明重新放宽 | 没有 | 子 agent 照新说明抄前缀，仍拒：`research/prompts/m3-defs-b9-r2-sonnet-model/out/s4-heavy-guard-b10.txt:1`：「A1 子 agent 在改过正文的副本里自加 =commit 前缀 退出码=2」，第 2 行 `=user-request` 同；b9 同 |

### S5（P5）叫停登记出处按原话找、行号当提示

| 判决条目 | 辩方判定 | 证据 |
|---|---|---|
| 攻方 Z4-甲：已收的叫停轮，出处那份调度记录追加一行抄了原话的补记，原话命中两次，⑥ 与 abandoned-rounds 翻回「开着」 | 修到 | 真树 m3-batch63-code-r1，`research/prompts/m3-defs-b9-r2-sonnet-model/out/r1o-z4-real.txt:2`：「b10 追加之后 叫停表：m3-batch63-code-r1 收 ⑥ 派实现员退出=0」；b9 `research/prompts/m3-defs-b9-r2-sonnet-model/out/r1o-z4-real.txt:4`：「b9 追加之后 叫停表：m3-batch63-code-r1 不收」，⑥ 退出 2；合成小树 `research/prompts/m3-defs-b9-r2-sonnet-model/out/r1o-z4-drift.txt:1`：「b10 excerpt self-quote 改动前：hx-quote-r1 收 改动后：hx-quote-r1 收」→第 4 行 b9 改动后不收 |
| 攻方 Z4-乙：登记后照 path-moves 跑术语改名，原话找不到 | 修到 | `research/prompts/m3-defs-b9-r2-sonnet-model/out/r1o-z4-drift.txt:2`：「b10 excerpt term-sweep 改动前：hx-term-r1 收 改动后：hx-term-r1 收」；b9 第 5 行改动后不收 |
| 攻方 Z4-丙：照 date-once 的出路删日期，原话找不到 | 修到 | `research/prompts/m3-defs-b9-r2-sonnet-model/out/r1o-z4-drift.txt:3`：「b10 excerpt date-once 改动前：hx-date-r1 收 改动后：hx-date-r1 收」；b9 第 6 行改动后不收 |
| 攻方 Z4-丁、正推 Z4-10、Z4-11：两份文件头仍写「不按 -r<数字> 形」，与库相反 | 修到 | 命令 `grep -rnF '不按 -r<数字> 形' <冻结副本>/.claude <冻结副本>/scripts`：b9 两处（钩子文件头第 39 行、门禁 `doc-process-records.sh` 第 103 行），b10 零处；`research/prompts/m3-defs-b9-r2-sonnet-model/out/r1o-z4-misc.txt:1` 的 ghost-r2 一例两版都不收 |
| 正推 Z4-02、Z4-09：规则写原话要含轮名、「判决文件」说宽了 | 修到（文字） | `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/rules/three-way-inference.md:121`：「原话所在的那一行含这一轮的名字」；`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/hooks/lib_abandoned_rounds.py:23`：「判决文件只认 <轮>-main-verification.md 这一种文件名」 |
| 正推 Z4-05：整行被逐字重复之后「多抄几个字」无路可走 | 修到 | `research/prompts/m3-defs-b9-r2-sonnet-model/out/r1s-abandoned-b9.tsv:32` 不收 → `research/prompts/m3-defs-b9-r2-sonnet-model/out/r1s-abandoned-b10.tsv:32` 收（「别的会话逐字重复了整行而原话已抄整行」）；同表另五例 拒→收：第 10、13、18、19 行等；七例变化里六例是 拒→收，一例（第 14 行，折成两行）仍不收、只换了原因 |
| 第一轮判对的收法（插行漂移、写错行号原话对、回车换行、更长轮名、小写的别的轮名、旧行名单、写死的四个旧名） | 没改坏 | 34 例其余 27 例逐行相同；插行漂移 `research/prompts/m3-defs-b9-r2-sonnet-model/out/s5-drift-matrix.txt:4`：「漂移：在前面插一行（行号对不上，原话唯一） b10 退出=0」同 b9；旧行名单再登记不写出处仍收（Z4-12，另立账）`research/prompts/m3-defs-b9-r2-sonnet-model/out/r1o-z4-misc.txt` 最后两行 |
| P5 的新窄口（攻方写的「弱点」） | 能造出来；比 b9 多一条无声路径 | 见「专问 S5」：改字加漂移、补记插在前面、原话瞎编三种，b9 不收、b10 收，行号提示那一行是另一句带轮名与「」的话；b9 也能靠「把原话抄成那一句」造出同一类（对照行，`research/prompts/m3-defs-b9-r2-sonnet-model/out/s5-drift-matrix.txt:35`） |

### S6（P6）doc-text 输出通道

| 判决条目 | 辩方判定 | 证据 |
|---|---|---|
| 攻方 Z5-甲：date-once 红行、term-renames 转出的 sweep-term 输出原样打路径 | 修到 | 攻方样本（假配置），b9 `research/prompts/m3-defs-b9-r2-sonnet-model/out/r1o-z5-cells-b9.txt:12`：「✗ records/2026-10-13-demo-peer-host-42-对拍.md：正文里重复所属日期 1 处」→b10 `research/prompts/m3-defs-b9-r2-sonnet-model/out/r1o-z5-cells-b10.txt:12`：「✗ records/〔文件名含私有值〕：正文里重复所属日期 1 处」；term-renames 第 17 行 `research/results/from-peer/srv/demo-private/peer-checkout/notes.md`→`research/results/〔文件名含私有值〕/notes.md` |
| 攻方 Z5-乙：clock-times 片段截在私有值中间，半截原样打出 | 修到 | `research/prompts/m3-defs-b9-r2-sonnet-model/out/r1o-z5-cells-b9.txt:3`：「records/2026-10-13-样本调度.md:6 ISO 时间戳 …ate/peer-checkout 上 2026-10-13T<时分秒> 收尾。…」→`research/prompts/m3-defs-b9-r2-sonnet-model/out/r1o-z5-cells-b10.txt:3`：「records/2026-10-13-样本调度.md:6 ISO 时间戳 …〔私有值〕 上 2026-10-13T<时分秒> 收尾。…」 |
| P6 只写了 clock-times，prime-marks、session-names 同一函数有没有吃到 | 修到（三格片段两头都挪） | 我的样本左边切在值里（撇号类、第二台仓路径）、右边切在值里（会话号后紧跟主机名）：b9 `research/prompts/m3-defs-b9-r2-sonnet-model/out/s6-cells-b9.txt:13`：「records/2026-10-13-样本调度.md:8 …checkout 之后五个字再出现 K9<撇号> 这个角标。…」→b10 `research/prompts/m3-defs-b9-r2-sonnet-model/out/s6-cells-b10.txt:13`：「records/2026-10-13-样本调度.md:8 …〔私有值〕 之后五个字再出现 K9<撇号> 这个角标。…」；右边 `research/prompts/m3-defs-b9-r2-sonnet-model/out/s6-cells-b10.txt:19`：「records/2026-10-13-样本调度.md:10 …- 会话 singlefs-<十六进制> 之后紧跟 〔私有值〕…」 |
| 第一轮没列出的第三条通道：term-renames 按行豁免对不上时 sweep-term 把行内锚原样打出 | 修到（顺带） | b9 `research/prompts/m3-defs-b9-r2-sonnet-model/out/s6-anchor-b9.txt:3`：「records/2026-10-13-样本调度.md:「不存在的锚 sample-second-host-77 之类」命中 0 行」→b10 `research/prompts/m3-defs-b9-r2-sonnet-model/out/s6-anchor-b10.txt:3`：「records/2026-10-13-样本调度.md:「不存在的锚 〔私有值〕 之类」命中 0 行」 |
| 第一轮判对的打码（private-values 自己的红行、私有值整个落进片段） | 没改坏 | `research/prompts/m3-defs-b9-r2-sonnet-model/out/r1o-z5-cells-b10.txt:4`：「records/2026-10-13-样本调度.md:7 ISO 时间戳 …〔私有值〕 上 2026-10-13T<时分秒> 开跑。…」，b9 同 |

### S7（P7）多机只照规范句、豁免表基准

| 判决条目 | 辩方判定 | 证据 |
|---|---|---|
| 攻方 Z6-乙、正推 Z6-12：规范句下一行写「每台线程按此刻空闲现算」三版都放，线程上限有「别处写的数一概不照」、多机没有对应一句 | 修到（文字，一半）；钩子行为没动 | 共用约束与十份定义补了一句：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/agent-common.md:85`：「子 agent 跑多机只照这一句规范句，提示里别处写的台数、线程取法一概不照」，`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/agents/three-way-defense.md:24`：「子 agent 跑多机只照「多机：」那一句规范句，提示里别处写的台数、线程取法一概不照」；钩子仍放：`research/prompts/m3-defs-b9-r2-sonnet-model/out/s7-prose-feed.txt:4`：「下一行：每台线程按此刻空闲现算 experiment-runner 放 放」 |
| 攻方 Z7-甲、正推 Z7-01：基准那一版没有这张表时不比，stage-owners 那一行写「判红」；重钉提交进基准之后不再红没写 | 修到（文字） | `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/gate.d/stage-owners.tsv:48`：「基准那一版没有这张表时不比；重钉提交进基准之后不再红」；共用约束 `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/agent-common.md:67`：「基准那一版没有这张表时不比；重钉提交进基准之后不再红」与主 agent 定义 `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/main-agent.md:55`：「基准那一版没有这张表时不比；重钉提交进基准之后不再红」同句；判法没动：`research/prompts/m3-defs-b9-r2-sonnet-model/out/r1o-z7.txt:3`：「b10 repin-基准没有表 exempt-check 退出=0」，b9 第 8 行同；基准有表重钉仍红，第 2 行退出=1 |
| 正推 Z7-01 其余两处：基准是旧写法只比路径、空基准文件逐行报涨与没有基准不比相反；Z7-06 无基准时新路径放行 | 没动（第一轮没收进改法也没另立账） | 豁免表 13 例两版逐行相同，`research/prompts/m3-defs-b9-r2-sonnet-model/out/r1s-exempt-b9.tsv` 与 `research/prompts/m3-defs-b9-r2-sonnet-model/out/r1s-exempt-b10.tsv` 的 `cmp` 零差；文字仍没写 |
## 第一轮判决的判重与判轻

「够得着」＝打中是真的；「同样打中别的臂」＝把同一段历史喂给替代方案，替代方案也中。

| 格 | 辩方判 | 理由与喂法 |
|---|---|---|
| Z1 | 判得对；P1 的代价判轻 | 站不住的两形（遮真声明、误判嵌套示例）都真，见 S1 前三行。P1 引进 markdown-it，第一轮只在攻方 19 例上量过，判决自己写了派发闸自证与 195 个标签没重跑（`research/prompts/m3-defs-b9-r1-main-verification.md:36`：「派发闸自证与上一题第二轮辩方 195 个标签没重跑」）；这一轮量出四处副作用（S1 的四条：改过头（HTML 块）、改过头（闭合行前缀）、改过头（库退路）、带进来（文字过宽）），第一轮没提。替代方案：整份照判（r2 形）不中这几形，但把代码块里的示例也判了，违反「代码块里不判」；所以它不是顶替 |
| Z2 | 判得对 | Z2-乙 真（b9 点名 var_from_registry.sh）；另立账的 Z2-甲、Z2-丙 这一轮没动（不在射程） |
| Z3 | 判得对；治法只有文字 | 另一种治法是放宽钩子（主 agent 不带前缀也放登记为 light 的副本自证），撞用户定案（仓副本自证收回放宽；`research/prompts/m3-defs-b9-r2-sonnet-model/out/s4-heavy-guard-b10.txt:1`：「A1 子 agent 在改过正文的副本里自加 =commit 前缀 退出码=2」，子 agent 这条没松）；文字补是唯一不撞的 |
| Z4 | 判得对，是最脆弱的一格；P5 的弱点判轻 | 三种日常改动 b9 全中、b10 全修到（S5）。判决把弱点写成「推的」，这一轮量过：真记录里造得出（S5 专问），而且同一弱点也打中按行号读的那一版（`research/prompts/m3-defs-b9-r2-sonnet-model/out/s5-line-number-read.txt:14`，弱点四行全收），所以是「同样打中别的臂」，不是 P5 独有；P5 比 b9 多出来的是无声路径——b9 要靠把原话抄成那一句才收，b10 的日常改动就会无声改指 |
| Z5 | 判得对；漏列第三条通道 | 判决列了 date-once、term-renames 路径与 clock-times 片段；term-renames 按行豁免对不上时 sweep-term 把行内锚原样打出是第三条（`research/prompts/m3-defs-b9-r2-sonnet-model/out/s6-anchor-b9.txt:3`），P6 的路径打码顺带堵了 |
| Z6 | 判得对（部分站不住） | 围栏那一半修到；「每台线程按此刻空闲现算」写在下一行那一半只是文字补一句，钩子仍放（S7），判决写的是文字，没夸大 |
| Z7 | 判得对 | 判法没动、文字补齐；重钉提交进基准之后不再红是读代码推的，门禁格我没起（要主 agent 跑） |
| 第一轮正推「文字没写到」里既没进改法也没另立账的 | 判轻 | Z1-04（说明行不能以标签起头）、Z4-01（行号不能省）、Z6-07（脚本文件头分片键写两行放行）、Z7-01 其余两处（基准旧写法只比路径、空基准逐行报涨）、Z7-06：这一轮喂法两版逐行相同（`research/prompts/m3-defs-b9-r2-sonnet-model/out/r1s-exempt-b9.tsv`、`research/prompts/m3-defs-b9-r2-sonnet-model/out/r1s-exempt-b10.tsv`、`research/prompts/m3-defs-b9-r2-sonnet-model/out/r1s-script-header-b10.tsv`），都是改前就有的文字缺口，危害只是误拒或认错，不涉用户定案；列在这里让主 agent 决定记不记账 |

## 正文「三」专问逐条答

### S1　markdown-it 与「人读派发提示」不一致的写法

喂法 `research/prompts/m3-defs-b9-r2-sonnet-model/tools/s1_construct_matrix.py`：真声明「线程上限：31」加一段含「线程上限：16」的构造，列钩子 b9→b10 拒没拒、markdown-it 眼里那一行落在哪种块里。「预期读者」是人工写的读法，不是实测。

| 写法 | markdown-it 眼里 | 预期读者 | b9→b10 | 方向与证据 |
|---|---|---|---|---|
| HTML 块（`<div>` 后紧跟围栏） | html_block | 代码示例 | 放→拒 | 更严，误拒；`research/prompts/m3-defs-b9-r2-sonnet-model/out/s1-construct-matrix.txt:3` |
| HTML 块（`<pre>`、HTML 注释里的行） | html_block | 示例、被注释的旧写法 | 拒→拒 | 更严；第 4、6 行；注释里的行钩子判，读者也多半当旧写法，不算漏 |
| 链接引用定义（标题跨行含声明行） | 正文 | 引文 | 拒→拒 | 一致；第 7 行 |
| setext 标题（声明行后接 `---`） | 正文 | 标题文字，仍是一行声明 | 拒→拒 | 一致；第 8 行 |
| 表格里的反引号 | commonmark 预设无表格，行首是竖线不是围栏 | 单元格 | 放→放 | 无影响（那一行本来不以「线程上限」起头）；第 9 行；无首尾竖线且以三个反引号起头的表格行造得出，没喂 |
| 制表符缩进（顶层） | 缩进代码块，不是围栏 | 示例 | 拒→拒 | 更严；第 10 行 |
| 制表符缩进（列表项里） | fence | 示例 | 拒→放 | 修到；第 11 行 |
| `\r\n` | 与 `\n` 同 | 同 | 放→放 | 一致；闭合围栏照遮、只有真声明照放，第 24、25 行；单独的 `\r` 没喂 |
| 闭合行缩进 4 格、制表符、前面带 `> ` | 围栏没闭合 | 示例或没闭合 | 拒→放 | 实现宽一格，见 S1 改过头（闭合行前缀）；第 13、14 行 |

差分模糊（`research/prompts/m3-defs-b9-r2-sonnet-model/tools/s1_differential_fuzz.py`，种子 1，20000 份随机拼的 markdown 碎片，两边遮的行不同的缩到最短）：b10 比 b9 少遮 54 类（HTML 注释里的围栏、列表项与围栏的错位）、多遮 53 类（列表项、引用块、波浪线、顶层围栏被缩进或 `> ` 闭合），`research/prompts/m3-defs-b9-r2-sonnet-model/out/s1-differential-fuzz.txt:1`：「b10 比 b9 少遮（b10 更严）：缩到最短后不同的例子 54 个，列最短的 12 个」、第 14 行「b10 比 b9 多遮（b10 更宽）：缩到最短后不同的例子 53 个，列最短的 12 个」。多遮里除容器内的围栏（有意）之外，就是改过头（闭合行前缀）那一类。

解析会不会抛异常：`parser.parse` 在 try 之外（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/hooks/runner-dispatch-guard.sh:1156`：「for token in parser.parse(parsed):」），抛了就是钩子崩。喂了九个种子各 3000 份随机碎片（含 NUL、`\r\n`、深嵌套），每个种子之后各跑八个极端形状（2000 层引用块、400 层列表、孤立代理字符、1MB 正文、2MB 单行等）：0 份异常，`research/prompts/m3-defs-b9-r2-sonnet-model/out/s1-fuzz-b10.txt:1`：「随机 3000 份（种子 1）：异常 0 份」；1MB 普通正文约 1 秒（耗时写在 stderr，没落盘）。

库载不进的退路：
- 触发：`import markdown_it` 或建解析器抛异常（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/hooks/runner-dispatch-guard.sh:1151`：「派发闸载不进 markdown-it-py（」），环境上＝PATH 里排在 `/usr/bin/python3` 前面的 python3 没有 dist-packages（venv、conda、pyenv、`python3 -S`）。用垫片（`research/prompts/m3-defs-b9-r2-sonnet-model/feeds/shim/python3`，`exec /usr/bin/python3 -S`）让导入真的失败，三份提示的结果在 `research/prompts/m3-defs-b9-r2-sonnet-model/out/s1-library-missing.txt`：普通围栏示例与波浪线示例在正常 PATH 下放行、垫片下退 2。
- 这一台的依赖是传递依赖：命令 `apt-mark showauto | grep -i 'markdown\|python3-rich'` 输出 `python3-markdown-it`、`python3-rich`，`apt-mark showmanual | grep -ic markdown` 输出 0，`apt-cache rdepends --installed python3-rich` 的反依赖是 `netplan.io`。链是 netplan.io → python3-rich → python3-markdown-it，不是任何项目登记的依赖（仓里只有钩子自己提到 markdown-it）；钩子头部没有 preflight 的依赖声明。卸掉 netplan.io 之后 `apt autoremove` 会带走它，退路就无声触发（推的，没卸过）。
- 无声：库载不进时那一句只写 stderr；放行（退 0）时 stderr 不回给模型（Claude Code 钩子约定，推的，没量过），拒的时候才看得到。
- 误伤：49 份真派发提示（`research/prompts/m3-defs-b9-r2-sonnet-model/out/s1-real-prompts.txt:1`：「份数 49」）b9、b10、b10 无库三种配置的 ⑭ ⑮ 判定逐份相同（第 2、3 行「b9 与 b10 的两项不同 0」「b10 有库 与 无库 的两项不同 0」），因为 49 份里没有一份含围栏行（反引号围栏、波浪线围栏、列表项里的、引用块里的，逐份 `grep -q` 判，各 0 份）；这个语料测不出退路的代价。能说的是触发后会误伤的集合：围栏里写了「线程上限：」「预计挂钟：」「分片：」「续跑：」「多机：」示例的提示；五种类型的标准骨架（`python3 scripts/agents/dispatch-prompt.py <类型>`）围栏行数都是 0，不受影响。
- 比 b9 更严（改过头（库退路））：退路一行都不遮，b9 认得的最普通的围栏示例也被拒。试改把退路改回逐行认法（钩子里留着的 `without_code_blocks_by_lines`），`research/prompts/m3-defs-b9-r2-sonnet-model/out/s1-fallback-trial.txt:2`：「fenced-example PATH=shim 退出=2」→第 6 行「fenced-example PATH=shim 退出=0」，波浪线示例仍拒（第 3、7 行），与 b9 同；一行改动，但 `code-block-library-missing` 开关的自证格要跟着改（我没动冻结副本，也没跑自证）。

### S2　两种出路之间有没有一种写法两头都不落

喂法 `research/prompts/m3-defs-b9-r2-sonnet-model/tools/s2_exit_matrix.py`，15 个形态，b9 与 b10 的放拒逐行相同（`research/prompts/m3-defs-b9-r2-sonnet-model/out/s2-exit-matrix.txt`）。出路一＝「线程上限：N」再加已判行；出路二＝单独已判行。

- 钩子自己的两条出路都过：出路一（N 不等于登记值加有理由的已判行）放，第 5 行；出路二（单独已判行，理由说不起重活）放，第 8 行；缺行与 N 不对的拒绝说明在 b10 都写了两条，`research/prompts/m3-defs-b9-r2-sonnet-model/out/s2-exit-matrix.txt:20`：「→ 怎么办：写成单独一行…线程上限：31…确实不起重活的，只写一行」（第 24 行同，b9 的第 18、22 行是一条合写「或确实不起重活」）。
- 两头都不落的写法有，但是在钩子看不到的一侧：钩子只看已判行非空，不看理由里写了什么，也不看 N 在不在代码块里。放行但按定义读出来的结果和主 agent 要的相反：单独已判行、理由里写要用 16 线程（子 agent 读成不起重活，第 9 行）；N 等于登记值加已判行写要用 8 线程（子 agent 按 31，第 10 行）；N 不等于登记值写在代码块里加已判行写在外面（N 被遮，子 agent 读成只有已判行，第 12 行）；N 带括注加已判行（括注被已判行放行，第 14 行）。读出来的结局是子 agent 停下交回或按登记值跑，没有一条违反用户定案（默认 31、固定不看忙闲），代价是白派一次；与 b9 同，不是 P2 带进来的。
- 所以 P2 修到的是说明文字（Z1-丙），没有也不能让钩子分辨两条出路；判决写的就是文字，没有夸大。

### S3　变量来源判法的边界

喂法：二十九份单情形脚本 `research/prompts/m3-defs-b9-r2-sonnet-model/feeds/s3/sample`（`EXPECT.tsv` 是预期），经攻方 `z2_htd_cell.py` 抽出格的 python 原样跑（`research/prompts/m3-defs-b9-r2-sonnet-model/out/s3-cell-b9.txt`、`research/prompts/m3-defs-b9-r2-sonnet-model/out/s3-cell-b10.txt`）。判法在 `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/gate.d/code-tooling.sh:2025`：「def registry_sourced_variables(shell_code):」，它按整份文件取每个变量名的所有 `名=值` 赋值，全是登记位读法才算（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/gate.d/code-tooling.sh:2026`：「每一处赋值都是登记位读法的变量名」）。

| 情形 | 结果 | 说明 |
|---|---|---|
| 变量在函数里 `local` 赋 | 绿 | 不分作用域，整份文件一个集合；函数从没被调用、或赋在子 shell 里、或赋在使用之后，变量到使用处是空的，也绿（三份，语义洞，不是钉数） |
| 变量赋在另一份 source 进来的文件里，本文件没赋 | 红 | 保守（`s3_sourced_only.sh`） |
| 本文件从登记位赋一次，source 进来的文件又把它赋成字面数 | 绿（盲区） | 只看本文件 |
| 赋两次：登记位在先字面在后 / 字面在先登记位在后 | 红 / 红 | 不看次序，后一种是误报（最终值是登记值） |
| 条件赋 `[[ … ]] && x=8`、`let x=8`、`declare -i x=8` | 红 | `名=值` 写法都认 |
| `(( x = 8 ))`（等号两边有空格）、`eval "x=8"`、`read x <<< 8`、`printf -v x %s 8`、`for x in 8`、`x+=0` | 绿（盲区，六份） | 不是 `名=值`；前提是本文件里还有一处登记位读法的赋值，否则变量不在集合里、红 |
| `${SINGLEFS_THREAD_CAP}` 取来 / 带字面回退 `:-8` | 绿 / 红 | 回退数字照算 |
| 登记位读法里夹数字（`heavy_thread_default \|\| echo 8`、`/ 2`） | 红 | 去掉登记位读法之后还有数字就不算 |

盲区七份在 b9 全被点名（b9 对任何 `$变量` 一律点名），是 P3 放宽带进来的，要故意拐弯才碰得到：同一个变量既从登记位取过、又用 `(( ))`、`eval`、`read` 这类写法赋了字面数。不涉用户定案；它是防笔误的探测、不是封口，但定义字面「每一处赋值」比实现宽（实现只认 `名=值`），建议改成「每一处 `名=值` 形式的赋值」。

### S4　「按提交时的检查办、不弹窗」与「重型提交之外先弹窗」在哪种命令上撞

两条各管的范围：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/rules/implementation-workflow.md:54` 那一行只管「登记为 light 的 --selftest」；`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/main-agent.md:21`：「提交之外任务确实要跑，先弹窗问用户，同意了才跑」管重型测试。靠登记表分界，钩子也只看登记表（不读副本里的脚本正文）。喂法 `research/prompts/m3-defs-b9-r2-sonnet-model/tools/s4_heavy_guard_feed.py`，b9、b10 退出码逐个相同（`research/prompts/m3-defs-b9-r2-sonnet-model/out/s4-heavy-guard-b9.txt`、`research/prompts/m3-defs-b9-r2-sonnet-model/out/s4-heavy-guard-b10.txt`）。撞的有两类：

- 登记成 light、副本里正文被改成起 `cargo test`（钩子不读正文）：主 agent 带 `=commit` 退 0，`research/prompts/m3-defs-b9-r2-sonnet-model/out/s4-heavy-guard-b10.txt:4`：「B1 主 agent 带 =commit，副本正文被改成起 cargo test（闸不读正文） 退出码=0」。规则那一行说不弹窗，重型那一条说重型要先弹窗；两边引用的都是「登记为 light」，登记是旧的，判官只剩主 agent 读交回报告里的脚本改动。b9 同（`research/prompts/m3-defs-b9-r2-sonnet-model/out/s4-heavy-guard-b9.txt:4`），P4 的变化是把「不弹窗」写明了，让这条路更像是被许可的。
- 登记成 heavy 的自证在副本里：带 `=commit` 退 0（第 7 行）、带 `=user-request` 退 0（第 8 行）、不带前缀退 2（第 6 行）。规则那一行不覆盖它，应当弹窗；钩子分不出这条命令是按哪一行办的。b9 同。
- 子 agent 一侧没有撞：自加前缀仍拒（第 1、2 行）。

### S5　行号提示参与之后，漂移、重复、改字各回到什么判定

喂法 `research/prompts/m3-defs-b9-r2-sonnet-model/tools/s5_drift_matrix.py`（19 个形态，用各自的 `lib_abandoned_rounds.py` 命令行，派发闸 ⑥ 与门禁 abandoned-rounds 格共用这一份），结果 `research/prompts/m3-defs-b9-r2-sonnet-model/out/s5-drift-matrix.txt`：

| 日常改动 | b9 | b10 | 行 |
|---|---|---|---|
| 漂移：前面插一行、三行 | 收（报原话现在在第 M 行） | 同 | 3–6 |
| 重复：后面追加一行抄了原话的补记 | 不收（命中 2 次） | 收（取提示那一行） | 7→8 |
| 重复：补记插在原行前面，提示行落在命中的两处之一 | 不收 | 收（提示落在补记上就取补记那一行） | 9→10 |
| 重复：提示行不在命中的两处里（补记插前、或插行再追加） | 不收 | 不收，说明点名「行号提示的第 N 行不在其中」 | 11→12、21→22 |
| 重复：整行逐字重复、原话抄成整行 | 不收（无路可走） | 收 | 13→14 |
| 改字：原话里的字被改、被术语改名换词，提示行不动 | 不收 | 收 | 15→16、17→18 |
| 改字加漂移：插一行再改字 | 不收 | 收，当且仅当移到提示位置的那一行带这一轮的名字、有 6 个字以上的「」、没有别的轮名；否则不收（攻方的 hx-drift-r1 一例不收，`research/prompts/m3-defs-b9-r2-sonnet-model/out/r1o-z4-misc.txt:7`：「drift 插行又改字 b10 hx-drift-r1 不收」） | 19→20 |
| 折行、删行 | 不收 | 不收（折行换了原因） | 23–26 |

弱点（提示那一行恰好是另一句带轮名与「」的话）在真记录里造得出来：
- 数：`records/` 96 份里，恰有一个 `<名>-r<数字>` 轮名且带去掉空白后至少 6 个字的「」的行 458 行、18 份文件；同一份记录里同一个轮名有两行及以上这样的行的（文件, 轮名）79 对，`research/prompts/m3-defs-b9-r2-sonnet-model/out/s5-real-records.txt:1`：「records 下 96 份；满足条件的行 458 行，分布在 18 份文件；同一文件里同一轮名有两行及以上的 (文件, 轮名) 对 79 对」。
- 造：取第一对（c381-r1，行号 793、800、801），登记行写成原话是记录里没有的一段话、行号写其中一行，b9 不收、b10 收，`research/prompts/m3-defs-b9-r2-sonnet-model/out/s5-real-records.txt:3`：「b9 原话瞎编（记录里没有），行号写成那一对的第一行 退出=1」、`research/prompts/m3-defs-b9-r2-sonnet-model/out/s5-real-records.txt:5`：「b10 原话瞎编（记录里没有），行号写成那一对的第一行 退出=0」。
- 不用故意瞎编也会遇到：改字加漂移（`research/prompts/m3-defs-b9-r2-sonnet-model/out/s5-drift-matrix.txt:20`）、补记插在前面（第 10 行）这两种日常改动会无声改指到开轮行或补记。门禁成功行会报「照行号提示取」的行（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/gate.d/doc-process-records.sh:930`：「hinted_rows = [」），派发闸 ⑥ 放行时不报。
- 是不是 P5 独有：不是。库自带的 `line-number-read` 开关就是按行号读的那一版（第一轮 r3 形），同一批形态它全收，`research/prompts/m3-defs-b9-r2-sonnet-model/out/s5-line-number-read.txt:17`：「弱点：原话瞎编，提示行是开轮行 b10-line-number-read 退出=0」；b9 上把原话抄成那一句也收，`research/prompts/m3-defs-b9-r2-sonnet-model/out/s5-drift-matrix.txt:35`：「对照：原话抄成另一句带轮名与…的话（它在记录里恰好一次），行号写叫停那一行 b9 退出=0」。所以不拿它否这批改动（跑前条款「打中也打中第一轮冻结副本」）；新的只有「日常改动无声触发」这一条路。
- 收严的试改（辩方腿提的，只在本腿 19 例上量过，没被攻过）：原话找不到而取提示行时，要求提示那一行与原话共有一段不短于原话六成的连续文字。19 例里四个例子由收变不收（改字加漂移、两个瞎编原话、开轮行），其余 15 例不变，`research/prompts/m3-defs-b9-r2-sonnet-model/out/s5-similarity-trial.txt:1`：「19 个例子里结论变了的 4 个（其余 15 个改前改后相同）：」。它挡不住「补记插在前面、提示落在补记上」（补记里抄着原话，和原话共有的文字够长）。

### S6　除挪片段与路径打码之外还剩哪些输出通道

列 `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/gate.d/doc-text.sh` 里往外打字的点，看哪些没经 `shown_path` / `shown_text` / `shown_lines_main`：
- 经打码的：三格的红行路径、片段、「没扫的」三个清单（二进制、不是普通文件、读不了），private-values 的路径与键名行，date-once 的红行路径（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/gate.d/doc-text.sh:40`：「红行里的路径经 private-values 那一份打码（shown_path）」），term-renames 转出的 sweep-term 全部输出（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/gate.d/doc-text.sh:472`：「if sweep_output="$(python3 "$SWEEP_TERM" --check "$ROOT" 2>&1)"; then sweep_status=0; else sweep_status=$?; fi」，stderr 并进来一起打码）。
- 没经打码、也不带仓内值的：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/gate.d/doc-text.sh:329`：「echo "  ✗ 认不出的选项 $1"」（调用方给的参数）、`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/gate.d/doc-text.sh:350`：「print(f'  ✗ 读不到禁用写法的登记 {library_path}：{error}')」（登记库路径与 OSError 文字）、`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/gate.d/doc-text.sh:336` 的 `进不去项目根 $ROOT`（本机仓根）。值本身不在这几条里。
- 门禁管不到的（文件头已写）：commit message、换了写法的值（大小写、拆开写、别称）、不足 8 个字的值、用户原话里的别称。私有值取词是按空白切开的（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/gate.d/doc-text.sh:197`：「for word in split_words(value.strip()):」），所以 `shown_line` 按空白切开再打码对得上：值里不会有空白。
- 实测：假配置样本五格加按行豁免锚，b10 输出里假主机名、假仓路径整串与半截都是 0 处（`grep -c 'second-checkout\|host-77'` 对 `research/prompts/m3-defs-b9-r2-sonnet-model/out/s6-cells-b10.txt` 输出 0，b9 同一份有）。除上面已堵的三条（路径、片段、行内锚）之外没找到别的通道；`sweep-term.py` 的 `--apply` 输出不在门禁里。

### S7　定义说「一概不照」之后，钩子判不到的那一半由谁判

钩子（⑭）判：「多机：」一行逐字是规范句、三个标签各只写一行、2 小时内的不判三行；正文别处写的台数与线程取法它读不到，喂法 `research/prompts/m3-defs-b9-r2-sonnet-model/tools/s7_prose_feed.py`：下一行写现算、两台各 64 与 4 线程、nproc 的一半、只用一台开 8 个 worker、引用块里转述用户、括号里台数、KS_THREADS，七种写法两种类型都放，b9、b10 同（`research/prompts/m3-defs-b9-r2-sonnet-model/out/s7-prose-feed.txt:4`：「下一行：每台线程按此刻空闲现算 experiment-runner 放 放」）；写进「多机：」那一行本身的写死台数拒（第 18、19 行）。判者：
- 读的时候：只有子 agent，照共用约束与十份定义那一句不照（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/agent-common.md:85`）。没有第二处源码核它。
- 写的时候：主 agent 自律，`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/main-agent.md:55`：「派发提示里不写死台数与额度」，同样没有源码核正文别处。主 agent 定义里没有对应「一概不照」的那一句（`grep` 该句只在共用约束和十份定义里），那是写给子 agent 读的，合理。
- 兜底：台数取多机配置（规范句的内容）、脚本文件头的读法由写的人保证（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/agent-common.md:67`：「文件头写的读法与脚本做的事是否一致由写的人保证，钩子只判写法」）。子 agent 若真照正文别处的数起命令，`capped.sh` 只核 N 是正整数或 default，不核等于登记值，没有机器兜底（第一轮 Z1-01 已记）。
- 所以这一半的判者是「子 agent 的自守」，P7 把它从没写变成写了，仍不可机器判；不是新的洞，Z6-乙 判决写的就是文字。
## 要主 agent 跑的命令

全部在 `research/prompts/m3-defs-b9-r2-sonnet-model/gate-samples/要主agent跑.txt`，造树脚本 `gate-samples/make_trees.sh`（对这一轮冻结副本造小树；第一轮攻方七棵树照原脚本再造一遍）。要点：
- 三道样本档（`code-tooling.sh`、`doc-process-records.sh`、`doc-text.sh`，`stage-selftest.sh --item`）：看批 10 新加的四份样本（`heavy-thread-defaults-var-source-green/red`、`private-values-clock-snippet-red`、`private-values-date-once-path-red`）与改过的两份（`abandoned-rounds-source-green/red`）判对。库这一层我替它们验过：`research/prompts/m3-defs-b9-r2-sonnet-model/out/s5-fixtures-lib.txt` 与 `research/prompts/m3-defs-b9-r2-sonnet-model/out/s3-fixtures-both.txt`。
- 叫停表四棵小树（追加补记、改字、插行再改字、插行再追加）：预计退 0、0、0、1；b9 同四棵预计 1、1、1、1。
- `heavy-thread-defaults` 对我造的二十九份脚本：预计点名十三份（十五行），七份盲区不点名。
- `doc-text` 两棵样本（假配置）：预计输出里 `sample-second-host-77`、`/srv/sample-private/second-checkout` 整串与半截零处。
- 第一轮攻方留下的七棵：只有叫停表追加补记（退 1→预计 0）、`htd-shell`（5 处→预计 4 处）、doc-text（原样→打码）三棵预计变，其余不变。
- 预计都是抽出格里的判法在小树上跑出来的，没起过格本身。

## 没做什么

- 没替主 agent 撤销或维持判决，只交复核结果；「改过头」「带进来」只写事实与方向，算不算按跑前条款由主 agent 判。
- 没起门禁格（`--check`）、没跑 `stage-selftest.sh`、没跑任何登记的 `--selftest`、没跑 `gate.sh`、没编译、没跑重型测试。
- 没读禁读清单（攻方报告、模型、草稿目录、`multi-host.env` 的值）；没改冻结副本；试改（S1 退路、S5 相似度闸）都在草稿目录拷贝里改，不在冻结副本，也没写进仓里的钩子或库。
- 没喂：无首尾竖线且以三个反引号起头的表格行、单独的 `\r`、真 agent 读提示的方式（markdown-it 与人读法的差异是我按 CommonMark 写的预期，不是实测）、门禁格本身的输出；派发闸放行时 stderr 是否回给模型是推的。
- 第一轮另立账的 Z2-甲、Z2-丙、Z7-乙、Z4-08、Z4-12 与上一题的另立账不在射程里，没复核。
- 第一轮攻方试改树（fix）没重做；第一轮攻方的 Z2-甲 真树喂法（`z2_real_tree.sh`）没重跑（另立账）。

## 复跑与核对

- 整条经内存包装（上限 2G），仓根、冻结副本、草稿目录都从参数传，草稿目录要在 `~/.local/share/singlefs/work/` 底下且是新的空目录；全跑约 5 分钟，前台命令不超过 240 秒时按 `RERUN_PARTS` 分几次：

```
bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-defs-b9-r2-sonnet-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo --work <新的草稿目录>
```

- 我最后一次从仓里的模型目录复跑：`r1sonnet r1opus`、`s1 s2 s3 s4 s5 s6 s7` 分次跑，换了三个长短不同的草稿目录名，输出全部与 `out/` 同名文件 `same`。`out/s1-real-prompts.txt` 不在复跑里（语料随时间变）。
- 模型目录每个文件的 sha256 在 `research/prompts/m3-defs-b9-r2-sonnet-model/SHA256SUMS`。
- 草稿目录里的冻结副本拷贝（`t9`、`t10` 与各趟 `run*`、`ver*`、`s*` 下的小树）交回前删掉。

## cite-check 末行

`python3 scripts/agents/cite-check.py research/prompts/m3-defs-b9-r2-sonnet-output.md --root /home/fy5090/code/singlefs` 的末行：

```
  ✓ 核了 89 处引文、17 处裸行号，对上 89 处，没判 17 处（其中裸行号没有引文可比 17 处；逐处列在上面）
```
