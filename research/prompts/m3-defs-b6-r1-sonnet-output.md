# m3-defs-b6-r1 正推腿（Sonnet）报告

日期 2026-10-13。格 V1、V2、V4、V6、V8。判法：零轮新写或改写的每一句定义与规则文字，对着冻结副本里的钩子、脚本、门禁源码判「一致 / 不一致 / 文字没写到」；不一致的给喂法与原样输出。
带引号的引文，路径写新冻结副本里的绝对路径（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/…`，交回闸与主 agent 的 `--root` 不同也认得）；不带引号的「`路径` 第 N 行」是新冻结副本里的相对路径。零轮之前那一版（`m3-defs-b3-r2/repo/`）只用来比对喂法，不作行号引用。

## 0 对象、方法、边界

- 对象的数（命令原样）：`diff -rq <旧> <新> | grep -v multi-host | wc -l` 输出 46；再去掉 `/fixtures/` 的 `grep -v 'multi-host\|/fixtures/' | wc -l` 输出 27，与 `research/prompts/m3-defs-b6-r1-snapshot/frozen-copy-sha256.txt` 的 27 份相同；`sha256sum -c … | grep -vc ': OK'` 输出 0（新冻结副本 27 份 sha256 全对）。
- 九份定义与规则文件的改动按分句（`；` `。`）比出来，新分句数（`tools/clausediff.py` 输出里 `grep -c '^  NEW'`）：agent-common 11、main-agent 7、clerk 6、investigator 4、three-way-defense 6、three-way-forward 6、tooling-writer 4、three-way-inference 4、verification 5。agent-common 第 81 行新增的 `cd`/`env -C`/仓副本自证三句是 V5，不归本腿。
- 脚本与钩子里写成规则的注释与说明（钩子文件头、库的文档串、脚本文件头）按同一口径逐段判，放在各格的第二张表。
- 喂法：派发闸两个入口（`--check-prompt` 与 stdin JSON）各喂两份副本；判法函数直接调（`lib_abandoned_rounds.py`、`lib_long_run.py exempt-check`、`dispatch-prompt.py` 的 `guard_type_tables`、`host-dispatch.py` 的 `header_hosts`）；`capped.sh`、`detached-run.sh long-run-check` 按子命令起。没跑任何 `--selftest`，没起门禁格（`--check`）。
- 复跑：`research/prompts/m3-defs-b6-r1-sonnet-model/tools/rerun.sh`（从模板拷，只改 `rerun_steps`）；存的输出在 `…-model/out/`。命令与结果见第 7 节。
- 私有配置：冻结副本根下 `multi-host.env` 一个值都没读没写；只对它与 `.example` 跑过 `grep -c`（变量名出现次数，都是 0）。
- 开工先读 `.claude/agent-common.md` 时读的是主工作区那一份（定义要求先读），没有拿它当证据；下面 agent-common 的行号与原文都来自冻结副本。

## 1 V1：骨架占位、括注词表、九种类型、⑭⑮

### 1.1 规则与共用约束的逐句判定

判定：一=一致，不=不一致，缺=文字没写到。依据里的 F-n 是喂法编号（第 7 节）。

| 编号 | 句 | 判定 | 依据 |
|---|---|---|---|
| AC-1 | 「派 `implementation-writer`、`investigator`、`tooling-writer`、`clerk`、`three-way-forward`、`three-way-defense`、`experiment-designer`」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/agent-common.md:85`) | 一 | 十种与派发闸 `LONG_RUN_AGENTS`（`.claude/hooks/runner-dispatch-guard.sh` 第 1174 行）逐个相同（`out/tables.out`）；F-1：三种新类型缺预计挂钟退 2，旧副本退 0 |
| AC-2 | 「括注里不许有数字、中文数词、…分之…合计…减…各」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/agent-common.md:85`) | 缺 | 判在字符上不在词上：`31（统一取登记值）` `（唯一登记值）` `（十分稳定）` `（万一内存不够）` `（半角括号不认）` `（各自取）` `（不减半）` 都退 2，拒绝说明点名「一」「十」「半」「各」「减」（F-2；旧副本都退 0）。「统一」「万一」不是数词，句子只写了「中文数词」 |
| AC-3 | 「尖括号里只有 `<填 …>` 算骨架占位，别的尖括号写法按认不出数拒」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/agent-common.md:85`) | 不 | 值以 `<` 起头时一致（`<登记值>` 退 2，旧副本退 0）；括注里面不判：`线程上限：31（<登记值>）` 在 `--check-prompt` 与 stdin 两个入口、`31（<按机器忙闲定>）` 在 `--check-prompt` 都退 0（F-3、F-3s） |
| AC-4 | 「`three-way-attack`、`clerk`、`three-way-forward`、`three-way-defense` 没写这一行（⑮）」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/agent-common.md:85`) | 一 | 九种与 `THREAD_LIMIT_REQUIRED_AGENTS`（`.claude/hooks/runner-dispatch-guard.sh` 第 993 行）相同；三种新类型无行退 2（F-4）。例外：同提示另有 `线程上限已判：<理由>` 一行时缺行也放（退 0），本句没写这条路 |
| AC-5 | 「用户叫停、不写判决的轮在…除这一轮的轮名外没有别的轮名」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/agent-common.md:85`) | 不 | 归第 4 节（V6） |
| AC-6 | 「读这张表的门禁 doc-process-records 的 abandoned-rounds 格与 ⑥ 共用一份判法：日期、同一轮重复、出处」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/agent-common.md:85`) | 缺 | 归第 4 节（V6） |
| AC-7 | 「用户或别的会话在消息、弹窗里明说要让出 CPU 时（只看到负载忙不算）」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/agent-common.md:61`) | 缺 | 归第 2 节（V2） |
| AC-8 | 「不为改线程上限给在跑的子 agent 发消息，在跑的照原来的 N 跑完」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/agent-common.md:61`) | 一 | 归第 2 节（V2） |
| MA-1 | 「括注里不写数字，也不写中文数词和…分之…合计…减…各…，只写说明性的词」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/main-agent.md:53`) | 缺 | 同 AC-2；另：括注不套 ⑭ 的现算词表，`31（按核数取）` `31（忙时降低）` `31（负载高就少起）` 都退 0（F-5），「说明性的词」判不了内容 |
| MA-2 | 「半角括号不认」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/main-agent.md:53`) | 一 | `31(本机)` 退 2「整数后面接的不是一段中文括注」；全角数字 `３１` 退 2「认不出数」（F-6，新旧相同） |
| MA-3 | 「尖括号里只有 `<填 …>` 是骨架占位，别的尖括号写法按认不出数拒」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/main-agent.md:53`) | 不 | 同 AC-3 |
| MA-4 | 「派发闸 ⑮ 拒 N 不等于登记值又没写这一行的、写了 `÷`、`/`、…一半…除以…、中文数词、…分之…合计…减…各…或两个及以上数字的、整数后面接逗号或别的话的」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/main-agent.md:53`) | 一 | `31÷2` `31/2` `31 一半` `31，即 15` `31（本机），第二台 16`、括注里 `÷` `/` 全退 2（F-7）；括注数词那一半同 AC-2 |
| MA-5 | 「`three-way-attack`、`clerk`、`three-way-forward`、`three-way-defense` 没写…线程上限：…一行的也拒」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/main-agent.md:53`) | 一 | 同 AC-4 |
| MA-6 | 「用户或别的会话在消息、弹窗里明说要让出 CPU 时（只看到负载忙不算），只改新派的子 agent」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/main-agent.md:53`) | 缺 | 归第 2 节（V2） |
| MA-7 | 「不为改线程上限给在跑的子 agent 发消息，它们照原样跑完」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/main-agent.md:53`) | 一 | 归第 2 节（V2） |
| MA-8 | 「⑭ 判的类型：…`clerk`、`three-way-forward`、`three-way-defense`、`experiment-designer`，派这几种都写」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/main-agent.md:55`) | 一 | 同 AC-1 |

### 1.2 五份定义「输入」补行的逐句判定

三种新类型 clerk、three-way-forward、three-way-defense 各补两行（预计挂钟、线程上限），investigator、tooling-writer 把「线程上限、内存上限」拆成线程上限一行加内存上限一行。

| 编号 | 句（整条） | 判定 | 依据 |
|---|---|---|---|
| CL-1 | 「预计挂钟：一行…预计挂钟：2 小时内…或…预计挂钟：超过 2 小时…；超过的另有…分片：…续跑：…多机：…三行，分不了的那几样由一行…分不了：…点名。派发闸查，缺了派不出来。」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/agents/clerk.md:23`) | 一 | F-1：无挂钟退 2；「超过 2 小时」不带三行退 2；三行齐退 0；「分不了：」点名缺的放（F-8）。挂钟那一行不看内容真假 |
| CL-2 | 「线程上限：一行…线程上限：N…，N 取登记的每台默认重活线程数（`.claude/heavy-thread-default`），后面只许接一段中文括注；N 不是登记值的另写一行…线程上限已判：<理由>…。派发闸查，缺了派不出来。」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/agents/clerk.md:24`) | 缺 | 「后面只许接一段中文括注」「N 不是登记值的另写一行」「缺了派不出来」三分句都与钩子一致（F-2、F-4、F-9）；没写的是：缺行也能用 `线程上限已判：<理由>` 一行放行（F-4，钩子的拒绝说明自己写了这条出路「确实不起重活」，定义没写） |
| FW-1 | 「预计挂钟：一行…预计挂钟：2 小时内…或…预计挂钟：超过 2 小时…；超过的另有…分片：…续跑：…多机：…三行，分不了的那几样由一行…分不了：…点名。派发闸查，缺了派不出来。」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/agents/three-way-forward.md:24`) | 一 | F-1：无挂钟退 2；「超过 2 小时」不带三行退 2；三行齐退 0；「分不了：」点名缺的放（F-8）。挂钟那一行不看内容真假 |
| FW-2 | 「线程上限：一行…线程上限：N…，N 取登记的每台默认重活线程数（`.claude/heavy-thread-default`），后面只许接一段中文括注；N 不是登记值的另写一行…线程上限已判：<理由>…。派发闸查，缺了派不出来。」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/agents/three-way-forward.md:25`) | 缺 | 「后面只许接一段中文括注」「N 不是登记值的另写一行」「缺了派不出来」三分句都与钩子一致（F-2、F-4、F-9）；没写的是：缺行也能用 `线程上限已判：<理由>` 一行放行（F-4，钩子的拒绝说明自己写了这条出路「确实不起重活」，定义没写） |
| DF-1 | 「预计挂钟：一行…预计挂钟：2 小时内…或…预计挂钟：超过 2 小时…；超过的另有…分片：…续跑：…多机：…三行，分不了的那几样由一行…分不了：…点名。派发闸查，缺了派不出来。」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/agents/three-way-defense.md:24`) | 一 | F-1：无挂钟退 2；「超过 2 小时」不带三行退 2；三行齐退 0；「分不了：」点名缺的放（F-8）。挂钟那一行不看内容真假 |
| DF-2 | 「线程上限：一行…线程上限：N…，N 取登记的每台默认重活线程数（`.claude/heavy-thread-default`），后面只许接一段中文括注；N 不是登记值的另写一行…线程上限已判：<理由>…。派发闸查，缺了派不出来。」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/agents/three-way-defense.md:25`) | 缺 | 「后面只许接一段中文括注」「N 不是登记值的另写一行」「缺了派不出来」三分句都与钩子一致（F-2、F-4、F-9）；没写的是：缺行也能用 `线程上限已判：<理由>` 一行放行（F-4，钩子的拒绝说明自己写了这条出路「确实不起重活」，定义没写） |
| IN-1 | 「线程上限：一行…线程上限：N…，N 取登记的每台默认重活线程数（`.claude/heavy-thread-default`），后面只许接一段中文括注；N 不是登记值的另写一行…线程上限已判：<理由>…。派发闸查，缺了派不出来。」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/agents/investigator.md:25`) | 缺 | 同 CL-2（investigator 原有的预计挂钟行不变，只补线程上限行） |
| IN-2 | 「内存上限；」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/agents/investigator.md:26`) | 缺 | 「内存上限」没有任何钩子判（派发闸里无此标签），这一句只是把原来「线程上限、内存上限」的后半留下 |
| TW-1 | 「线程上限：一行…线程上限：N…，N 取登记的每台默认重活线程数（`.claude/heavy-thread-default`），后面只许接一段中文括注；N 不是登记值的另写一行…线程上限已判：<理由>…。派发闸查，缺了派不出来。」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/agents/tooling-writer.md:25`) | 缺 | 同 IN-1 |
| TW-2 | 「内存上限；」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/agents/tooling-writer.md:26`) | 缺 | 同 IN-2 |

### 1.3 钩子文件头与脚本文档串里的 V1 文字

| 编号 | 句 | 判定 | 依据 |
|---|---|---|---|
| HG-1 | 「⑮ 线程上限：提示里行首…后面只认一个十进制整数，整数后面只许接一段中文括注（全角括号，括注里没有数字、没有中文数词〇一二三四五六七八九十百千两半」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/hooks/runner-dispatch-guard.sh:62`) | 一 | 字符集与 `.claude/hooks/runner-dispatch-guard.sh` 第 991 行 的字符类逐字相同；「分之」「合计」「减」「各」同行 |
| HG-2 | 「经 scripts/hosts/heavy_threads.py 的 default_heavy_threads 取，不看 SINGLEFS_THREAD_CAP 这类临时覆盖」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/hooks/runner-dispatch-guard.sh:64`) | 一 | F-10：环境里设 `SINGLEFS_THREAD_CAP=16` 时 `线程上限：16` 仍退 2（16 不等于登记值 31），`线程上限：31` 退 0 |
| HG-3 | 「出现 ÷、/、…一半…两个及以上数字、整数后面接括注之外的话」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/hooks/runner-dispatch-guard.sh:65`) | 一 | F-7 |
| HG-4 | 「后三种的定义许编译、跑编译出来的代码…而提示里没有这一行也拒」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/hooks/runner-dispatch-guard.sh:67`) | 不 | 括注里的事实与定义不符：「要编译、跑编译出来的代码的」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/agents/clerk.md:29`)；「要改、要编译先拷进自己的草稿目录」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/agents/three-way-defense.md:36`)（讲的是冻结副本只读，不是许编译）；three-way-forward 定义里 `grep -n '编译'` 零命中（命令见第 7 节 C-1）。「没有这一行也拒」一致（F-4） |
| HG-5 | 「放行行…线程上限已判…用户给了参数、内存装不下这类…两种拒都放」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/hooks/runner-dispatch-guard.sh:68`) | 一 | F-4：N 不等于登记值加已判行、缺行加已判行都退 0；已判行理由不看内容（F-11） |
| HG-6 | 「N 是 <填 …> 占位…别的尖括号写法…按认不出数拒，不当占位；读不到登记时放行、stderr 写一句」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/hooks/runner-dispatch-guard.sh:69`) | 不 | 「读不到登记时放行」一致（C-2：`RUNNER_DISPATCH_GUARD_HEAVY_THREADS_LIBRARY` 指到不存在的文件，`线程上限：16` 退 0，stderr 一行「…这一次没判」）；「别的尖括号写法按认不出数拒」同 AC-3，括注里放行 |
| HG-7 | 「⑭ 任务带长活的类型（LONG_RUN_AGENTS：experiment-designer，加 ⑮ 那张类型表）」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/hooks/runner-dispatch-guard.sh:53`) | 一 | F-1；十种类型三处相同（`out/tables.out`） |
| DP-1 | 「线程上限那张类型表，九种：实现员、调查员、工具员、执行员、变异分诊员、攻方腿、书记员、前向腿、防方腿」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/scripts/agents/dispatch-prompt.py:20`) | 不 | 命名：「事务员（clerk）」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/agents/clerk.md:11`)；「书记员」在同一钩子里指 kb-scribe：「⑧ 书记员：提示里点名的工作根」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/hooks/runner-dispatch-guard.sh:46`)。九种里没有 kb-scribe，这一句的「书记员」应是「事务员」 |
| DP-2 | 「带重活的九种类型缺这一行也拒…逐个相同，自检解析派发闸那两行逐个比」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/scripts/agents/dispatch-prompt.py:25`) | 一 | `out/tables.out`：`guard_type_tables()` 解析派发闸两行（问题 None），骨架 `HEAVY_WORK_TYPES` 九种、`LONG_RUN_TYPES` 十种，两张表各自与派发闸、与 agent-common 和 main-agent 里列的两串相同；没跑自检本身 |
| DP-3 | 「登记读不到时这一行留占位，--check 拒」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/scripts/agents/dispatch-prompt.py:25`) | 一 | 骨架 `python3 scripts/agents/dispatch-prompt.py clerk` 打出 `线程上限：31` 与预计挂钟、分片、续跑、多机四行；forward、defense 同；旧副本 clerk 只有线程上限行，forward、defense 两样都没有（C-3） |

### 1.4 V1 专问

**中文数词表与「说明性的词」的边界。** 表是字符表：`〇一二三四五六七八九十百千两半` 加「分之」「合计」「减」「各」，按字符匹配括注全文（F-2、F-10）。
- 拒（符合句意）：`第二台十六`、`两台各取登记值`、`合计六十二`、`三分之一`、`与第二台同值`、`与另一台同值`、`两台同值`。
- 拒（句子没写）：含数字字的普通词——`统一取登记值`、`唯一登记值`、`十分稳定`、`万一内存不够`、`半角括号不认`、`各自取`、`不减半`。这些是「说明性的词」，被字符表误伤；规则文字没说判到字符。
- 放（句子想拦）：`按核数取`、`忙时降低`、`负载高就少起`（F-5）。⑭ 的现算词表（核数、忙闲、负载……）不套在括注上，「只写说明性的词」判不了内容。
- 放（符合句意）：`本机`、`登记值`、`本机，不看忙闲`、`每台默认`、`同上`。
**九种类型里哪种派发其实不带重活却被逼写线程上限。** clerk、three-way-forward、three-way-defense。clerk 的定义只在做什么第 2 条写「要编译、跑编译出来的代码的」才用线程上限，输入一节却无条件要这一行；forward、defense 的定义没有任何编译或起重活的步骤（forward 零命中，defense 只有「要改、要编译先拷进自己的草稿目录」）。出路是 `线程上限已判：<理由>` 一行（F-4，三种都退 0），钩子的拒绝说明写了这条路，三份定义没写。

### 1.5 V1 三问

- **过度外推或越位。** 一是 HG-4 的括注把「定义许编译」说给 forward 与 defense，定义里没有；二是括注字符表判到了普通词（AC-2）；三是括注判不了内容（MA-1）。判不了的、判过了头的都不影响九种类型的表本身（AC-1、AC-4 一致）。
- **内部矛盾或循环依赖。** 找到两处不严的对：AC-3/MA-3/HG-6 三处都写「别的尖括号写法按认不出数拒」，代码只对值以 `<` 起头的拒；缺行与 `线程上限已判` 的关系只在钩子的拒绝说明里写了（AC-4、CL-2）。⑬ 与 ⑮ 没有循环：值以 `<填` 起头的交 ⑬（F-3 里 `线程上限：<填 线程上限>` 退 2，说的是「骨架还有 1 处占位没填」）；登记读不到时 ⑮ 放行（C-2），骨架那一半是 dispatch-prompt.py 文档串的说法，没喂。
- **最脆弱的一条（本腿负责的五格里）。** V1 的括注字符表（AC-2、MA-1）：换一个最自然的写法就误伤（`统一`、`万一`、`十分`），也能换一个最自然的写法绕过去（`忙时降低`）。
- **什么现象会推翻。** AC-2「缺」：若括注里写「统一」仍退 0，或规则文字改成「含这些字符的括注一律拒」，则变一致；AC-3「不」：若 `31（<登记值>）` 退 2，则变一致；HG-4「不」：若 forward 定义里补上编译或起重活的步骤，则变一致。

## 2 V2：让出 CPU 与在跑的子 agent

| 编号 | 句 | 判定 | 依据 |
|---|---|---|---|
| AC-7 | 「用户或别的会话在消息、弹窗里明说要让出 CPU 时（只看到负载忙不算），主 agent 新派的写别的数，并带一行」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/agent-common.md:61`) | 缺 | 钩子只看 `线程上限已判` 行的理由非空、不以 `<` 起头、不是「无」「没有」这类（F-11）：理由写「机器忙，负载高，让出 CPU」退 0，写一个字 `x` 退 0，写 `<理由>` 或「无」退 2。「消息、弹窗里明说」「只看到负载忙不算」没有机器判，靠主 agent |
| AC-8 | 「不为改线程上限给在跑的子 agent 发消息，在跑的照原来的 N 跑完」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/agent-common.md:61`) | 一 | SendMessage 钩子 `continuation-guard.sh` 只判收件人交回过没有，`grep -c '线程'` 输出 0（C-4）；没有哪道钩子拦这句话本身。与同文件的 「干到一半收到主 agent 的消息：当成追加的输入并进这一轮做」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/agent-common.md:103`) 不冲突：那是收到消息之后的做法，这一句规定主 agent 不发 |
| MA-6 | 「用户或别的会话在消息、弹窗里明说要让出 CPU 时（只看到负载忙不算），只改新派的子 agent」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/main-agent.md:53`) | 缺 | 同 AC-7 |
| MA-7 | 「不为改线程上限给在跑的子 agent 发消息，它们照原样跑完」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/main-agent.md:53`) | 一 | 同 AC-8；与 「改了定义、共用约束或规则之后，当场给每个在跑的子 agent 发消息」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/main-agent.md:39`) 分工：那一句管定义与规则的改动，这一句管线程上限的数，两句各管一类，一句不覆盖另一句 |

**三问。**
- 过度外推或越位：没有。两句都只约束主 agent 的做法，没有往钩子里加判不了的东西。
- 内部矛盾或循环依赖：没找到。一个边界：改 `.claude/heavy-thread-default` 本身不是「改了定义、共用约束或规则」，不触发 `.claude/main-agent.md` 第 39 行 那一句的当场发消息；这一点文字没写，但不产生矛盾。
- 最脆弱：「只看到负载忙不算」。写一句「机器忙」的已判理由，钩子照放（F-11）；改判靠主 agent 自觉。
- 什么现象会推翻：若 `线程上限已判` 的理由写「忙」「负载」被钩子退 2，AC-7 变一致；若 SendMessage 的钩子开始拦含「线程上限」的消息，AC-8 的「无机器判」要改写。

## 3 V4：`--test-threads 1` 与 GPU 对拍 CPU 一侧的线程

### 3.1 规则文字

| 编号 | 句 | 判定 | 依据 |
|---|---|---|---|
| VF-1 | 「线程数：子 agent 照派发提示的…线程上限：N…，`scripts/hosts/capped.sh default`（登记的每台默认重活线程数）只给主 agent 与脚本内部用」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/rules/verification.md:26`) | 一 | `scripts/hosts/capped.sh` 第 53 行 的拒绝说明是同一句；C-5：`capped.sh abc true` 退 2 并打这句，`capped.sh default true` 谁起都退 0（无钩子拦），规则句。子 agent 在 `capped.sh N` 里再起脚本时，脚本内部的 `capped.sh default` 取到 N（C-5 第 4 条输出 `CARGO_BUILD_JOBS=7 SINGLEFS_THREAD_CAP=7`；「套在另一层 capped.sh <N> 里面的 default 取到的是 N」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/scripts/hosts/capped.sh:15`)），所以脚本内部用 default 不绕过派发的 N |
| VF-2 | 「门禁 `checker-tier-layer0-replay` 带 gpu 特性那一趟与 `scripts/crash/crash-point-replay.sh` 开了 GPU 的单机分支带」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/rules/verification.md:59`) | 一 | 门禁：`.claude/gate.d/checker-tier-layer0-replay.sh` 第 734 行；脚本：`scripts/crash/crash-point-replay.sh` 第 467 行，且只在 `gpu_switch` 为真时（同一函数 `if (( gpu_switch ))` 里）；单机路径的 `cargo test` 行带 `${test_threads_arguments[@]}` |
| VF-3 | 「该脚本的双机分支（只起 `scripts/crash/crash-amplification-two-hosts.sh`…cargo test` 的参数由 `scripts/crash/crash-amplification-lib.sh` 拼）」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/rules/verification.md:59`) | 一 | 双机分支 `scripts/crash/crash-point-replay.sh` 第 357 行 只起那一个脚本，参数只有 `--library --label --given-states --memory-cap --quiesced`；共用库拼参数 「cargo_test_arguments=(test --release -p singlefs-checker-tier --features "$features" --test "$test_target" -- --nocapture)」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/scripts/crash/crash-amplification-lib.sh:172`)，无 `--test-threads`；C-6：三个 crash-amplification 脚本里 `grep -c test-threads` 都是 0 |
| VF-4 | 「`scripts/gate/admission.py` 拼的 crash-case 命令、崩溃放量、双机驱动还没带，改成带时同时改这一条」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/rules/verification.md:59`) | 一 | 准入模块拼的命令 `scripts/gate/admission.py` 第 2471 行 以 `--nocapture` 收尾，无 `--test-threads`；`grep -n 'test-threads' scripts/gate/admission.py` 只有第 5273 行的临时目录前缀 `admission-selftest-threads-`（C-6） |
| VF-5 | 「GPU 对拍与内存对拍不压并发：用例内部的 CPU 线程数由用例自己读的变量定，GPU 对拍测试的 CPU 一侧读 `SINGLEFS_CRASH_JUDGE_CPU_THREADS`，没设取核数（`available_parallelism`），不读登记的每台默认重活线程数」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/rules/verification.md:60`) | 一 | `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 第 265 行 与变量名 `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 第 208 行；另一个 CPU 判器测试同写法 `crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs` 第 86 行；两份测试文件里 `heavy-thread-default`、`SINGLEFS_THREAD_CAP`、`RUST_TEST_THREADS` 零命中；`capped.sh 7` 起的进程里该变量是 unset（C-5 第 3 条输出）——所以「线程上限：N」管不到这一处，句子说的正是这件事 |
| VF-6 | 「代码批改成取登记值时同时改这一条」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/rules/verification.md:60`) | 一 | `.claude/heavy-thread-exempt` 已登记这两份测试：`.claude/heavy-thread-exempt` 第 76 行、`.claude/heavy-thread-exempt` 第 77 行 |

### 3.2 脚本与门禁文件头里的 V4 文字

| 编号 | 句 | 判定 | 依据 |
|---|---|---|---|
| CP-1 | 「开了 GPU 时单机路径的每一趟快档测试参数带…-- --nocapture --test-threads 1」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/scripts/crash/crash-point-replay.sh:43`) | 一 | 同 VF-2；调用记录核对在同文件的 quick-gpu 格（`scripts/crash/crash-point-replay.sh` 第 918 行），没跑 |
| CP-2 | 「还没带…--test-threads 1…的（不是无例外）：双机分支起的 crash-amplification-two-hosts.sh」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/scripts/crash/crash-point-replay.sh:44`) | 一 | 同 VF-3 |
| CP-3 | 「准入模块 admission.py 拼的 crash-case 命令、崩溃放量 crash-amplification.sh；这几处等代码批与下一批改动同时改成带」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/scripts/crash/crash-point-replay.sh:45`) | 一 | 同 VF-4 |
| CP-4 | 「这个参数只管同时起几个测试，不管一个测试里 CPU 判器自己开的线程：GPU 对拍的测试读 SINGLEFS_CRASH_JUDGE_CPU_THREADS，没设取本机可用并行度」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/scripts/crash/crash-point-replay.sh:46`) | 一 | 同 VF-5；旧副本这一处写的是「CPU 一侧判器的线程照 capped.sh default 交的登记值」，`capped.sh` 的变量清单（`scripts/hosts/capped.sh` 第 39 行 起）里没有这个变量，旧句不对，新句对 |
| CP-5 | 「单机路径带 gpu 特性的每一趟测试线程只开 1 个」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/scripts/crash/crash-point-replay.sh:465`) | 一 | 同 VF-2；同一段注释的后半「双机分支……起的双机驱动还不接这个参数」同 VF-3 |
| L0-1 | 「特性里有 gpu 时这一趟的测试参数带…-- --nocapture --test-threads 1」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/gate.d/checker-tier-layer0-replay.sh:59`) | 一 | `.claude/gate.d/checker-tier-layer0-replay.sh` 第 734 行 |
| L0-2 | 「还没带这个参数的（不是无例外）：准入模块 admission.py 拼的 crash-case 命令（崩溃枚举用例）、崩溃放量 crash-amplification.sh、双机驱动 crash-amplification-two-hosts.sh（含 crash-point-replay.sh 的双机分支）」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/gate.d/checker-tier-layer0-replay.sh:60`) | 一 | 同 VF-3、VF-4 |
| L0-3 | 「一个测试里 CPU 判器自己的线程不受这个参数管：读 SINGLEFS_CRASH_JUDGE_CPU_THREADS，没设取本机可用并行度」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/gate.d/checker-tier-layer0-replay.sh:733`) | 一 | 同 VF-5 |
| CAP-1 | 「子 agent 照派发提示里…线程上限：N…写 N；default（取 .claude/heavy-thread-default 的登记值）只给主 agent 与脚本内部用」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/scripts/hosts/capped.sh:53`) | 一 | C-5；这句是拒绝说明，只在上限写错时打 |

### 3.3 V4 三问

- 过度外推或越位：没有。V4 的句子只描述「哪几处带了、哪几处没带、CPU 一侧读什么变量」，都是对源码的陈述，没有往钩子里加判断。「改成带时同时改这一条」是过程承诺，没有机器判，靠门禁 checker-tier-layer0-replay 的 `quick-features-green` 样本只钉了「带」的一侧（文件头 `.claude/gate.d/checker-tier-layer0-replay.sh` 第 62 行 提到的 expect 核这一行），没钉「还没带」的几处。
- 内部矛盾或循环依赖：没找到。`capped.sh N` 不设 `SINGLEFS_CRASH_JUDGE_CPU_THREADS`（C-5），所以「N 管 CPU 一侧」那句话没有被写进任何规则；VF-5 把它明说成「不读登记值」，与 agent-common 第 61 行「各装置读的线程变量一次设成 N」不冲突，因为这个变量不在清单里。
- 最脆弱：VF-3、VF-4 的「还没带」清单是手写的名单，没有哪道检查把它与 `crash-amplification-lib.sh` 的 `cargo_test_arguments` 绑在一起；哪天共用库加了 `--test-threads 1`，这三处文字（verification.md、两份文件头）同时过期而门禁不红。
- 什么现象会推翻：`grep -c test-threads scripts/crash/crash-amplification-lib.sh` 出现非 0，则 VF-3、CP-2、L0-2 变不一致；`capped.sh N` 起的进程里 `SINGLEFS_CRASH_JUDGE_CPU_THREADS` 不再是 unset，则 VF-5、CP-4、L0-3 变不一致。

## 4 V6：叫停轮登记行的判法（⑥ 与门禁 abandoned-rounds 格）

### 4.1 规则文字

| 编号 | 句 | 判定 | 依据 |
|---|---|---|---|
| TI-1 | 「第 N 行里要有这一轮的轮名（前后不连着别的轮名字符）与…括起来的用户原话，原话去掉空白后至少 6 个字，除这一轮的轮名外不许有别的轮名。」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/rules/three-way-inference.md:121`) | 不 | 判法只认两种别的轮名：`.claude/hooks/lib_abandoned_rounds.py` 第 64 行 与登记表里出现过的名字。出处那一行写 `m3-close-b1`（无 -r、没登记）或 `OTHER-R2`（大写）都收（V-1、V-2，库默认读法与 --legacy-ok 读法都输出 `收`，派发闸端到端 ⑥ 放行）；写 `other-r2` 或登记过的 `oldround` 则不收。规则句说「不许有别的轮名」，没说认哪几种写法 |
| TI-2 | 「日期写 YYYY-MM-DD，同一轮只登记一行。」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/rules/three-way-inference.md:121`) | 一 | V-3：`2026/10/13` 不收、同一轮两行（都合格）不收、两行里一行日期写坏也不收；日期只验形状，`2026-13-45` 收（V-4），句子只说了写法 |
| TI-3 | 「派发闸 ⑥ 与门禁 doc-process-records 的 abandoned-rounds 格共用一份登记行判法（日期、同一轮重复、出处三样），出处不合这几条的 ⑥ 不收，那一轮照旧算开着，拒绝说明写没收的原因；」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/rules/three-way-inference.md:121`) | 缺 | 共用的是同一份库（门禁格 `.claude/gate.d/doc-process-records.sh` 第 261 行，钩子 `.claude/hooks/runner-dispatch-guard.sh` 第 393 行）。「三样」没数四列不空（同段前文有）；没写的是：门禁传 `legacy_sourceless_ok=True`（`.claude/gate.d/doc-process-records.sh` 第 830 行），⑥ 不传（`.claude/hooks/runner-dispatch-guard.sh` 第 831 行），共用的判法两边出处要求不一样（4.3 的 A、C 形）。拒绝说明写原因：一致（C-7 输出里有「登记了 mtest-r1，但收不了这一轮：第 1 行没有出处……」） |
| TI-4 | 「第 N 行是开轮行、引规则行、…这类，或那句话是不是用户为这一轮说的，由主 agent 核。」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/rules/three-way-inference.md:121`) | 一 | V-5：records 里那一行是「开 mtest-r1 第一轮，规则写『用户叫停的轮要登记』」，库收、⑥ 放行——钩子认不出，句子说的正是这一半靠主 agent |
| AC-5 | 「那一行有这一轮的轮名与 6 个字以上的…原话、除这一轮的轮名外没有别的轮名、真路径在 `records/` 之内，不合的照旧挡」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/agent-common.md:85`) | 不 | 同 TI-1；「真路径在 records/ 之内」一致（V-6：`records/link.md` 符号链接指到 records 外，库输出「不收：…解到…不在…records 之内」；`records/sub/deep.md` 收；`records/../` 写法不收） |
| AC-6 | 「读这张表的门禁 doc-process-records 的 abandoned-rounds 格与 ⑥ 共用一份判法：日期、同一轮重复、出处」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/agent-common.md:85`) | 缺 | 同 TI-3 |

### 4.2 钩子、库、门禁文件头里的 V6 文字

| 编号 | 句 | 判定 | 依据 |
|---|---|---|---|
| HG6-1 | 「一行收不收的判法只在同目录的 lib_abandoned_rounds.py 一份，门禁那一格调同一份（不收就是红）」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/hooks/runner-dispatch-guard.sh:35`) | 不 | 日期早于 2026-10-12 的行：门禁读法收、⑥ 读法不收（`out/lib-table.out`：冻结副本自己的登记表，⑥ 读法 4 行不收、`--legacy-ok` 读法 5 行全收；V-7）。「不收就是红」对这些行不成立 |
| HG6-2 | 「除这一轮外没有别的轮名（形如 <名>-r<数字> 的，或登记表里出现过的轮名；同一行里别的轮的叫停原话不收）」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/hooks/runner-dispatch-guard.sh:38`) | 一 | 这一句把认法写全了，与 `.claude/hooks/lib_abandoned_rounds.py` 第 127 行 一致；V-1、V-2 的收与 V-8（`other-r2`、`oldround` 不收）都按它。TI-1 没写认法 |
| HG6-3 | 「指到这一轮自己的材料（research/prompts/<轮>*、_<轮>*）、规则与定义文件、仓外路径、别的轮的叫停原话、日期写坏、同一轮两行、」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/hooks/runner-dispatch-guard.sh:39`) | 一 | V-9：出处指到 `research/prompts/_mtest-r1-body.md` 不收（「没有出处：要写 records/……」，只认 records/）；日期写坏、同一轮两行、行号越界、`..` 路径都不收；`.claude/` 下规则文件同样不认，因为 `.claude/hooks/lib_abandoned_rounds.py` 第 59 行 的正则只匹配 records/ 起头（这一条没单独喂） |
| HG6-4 | 「读不到 lib_abandoned_rounds.py 时 ⑥ 不收任何叫停的轮（登记表当没有），stderr 写一句。」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/hooks/runner-dispatch-guard.sh:41`) | 一 | C-8：同一份提示，完整钩子放行，去掉库的钩子拒（⑥ 的拒绝说明 + 一行「! 派发闸读不到同目录的 lib_abandoned_rounds.py……叫停的轮这一次都没收」） |
| HG6-5 | 「门禁 doc-process-records 的 abandoned-rounds 格读同一张表、判同一份，这里不收那一格就红」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/hooks/runner-dispatch-guard.sh:981`) | 不 | 拒绝说明里的这一句同 HG6-1 |
| LIB-1 | 「同一张表只有一份判法，三样两边都判，…不收…与…红…是同一个结局」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/hooks/lib_abandoned_rounds.py:7`) | 不 | 同 HG6-1；同一个文档串 `.claude/hooks/lib_abandoned_rounds.py` 第 18 行 自己写了例外 |
| LIB-2 | 「派发闸不传（⑥ 收一个开着的轮，出处一直要）。形状与重复两样不分日期。」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/hooks/lib_abandoned_rounds.py:19`) | 一 | V-3：形状与重复两样两种读法输出相同；出处一样：旧日期行两种读法不同 |
| DPR-1 | 「登记行一行收不收，判法只在 .claude/hooks/lib_abandoned_rounds.py 一份，派发闸 ⑥ 调同一份（不收 ⟺ 红）」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/gate.d/doc-process-records.sh:99`) | 不 | 同 HG6-1 |
| DPR-2 | 「日期早于 2026-10-12（那份库的 SOURCE_REQUIRED_FROM）的行是出处这条规矩立起来之前登记的，没有出处可补，不要求出处（这一格传 legacy_sourceless_ok）；派发闸收一个开着的轮，出处一直要。」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/gate.d/doc-process-records.sh:101`) | 缺 | 例外写明了，一致于代码；缺的是前提：「日期」在登记表头里的定义是腿发出那天：「日期（腿发出那天，YYYY-MM-DD）」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/research/prompts/abandoned-rounds.tsv:2`)，不是登记那天。一轮在 2026-10-11 或更早发腿、今天撂下，登记行的日期是旧的，门禁不要出处，⑥ 要（4.3 的 A 形） |
| DPR-3 | 「管不到的：判决写得对不对、登记的理由真不真、出处那句话是不是用户为这一轮说的（靠主 agent 核）」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/gate.d/doc-process-records.sh:107`) | 一 | 同 TI-4 |
| DPR-4 | 「叫停轮登记行怎么判：判法只在这一份，派发闸 ⑥ 调同一份」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/gate.d/doc-process-records.sh:260`) | 一 | `.claude/gate.d/doc-process-records.sh` 第 261 行 取自 `$STAGE_DIR/../hooks/`，与钩子同目录同一份文件 |
| DPR-5 | 「这些行的出处收不了（派发闸 ⑥ 读同一张表、判同一份，不收就是红；日期」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/gate.d/doc-process-records.sh:877`) | 不 | 门禁红的提示里的「不收就是红」同 HG6-1；后半「起登记的行都要出处」同 DPR-2（日期是腿发出日不是登记日） |

门禁样本相关的 DPR 头注（abandoned-rounds-source-red / -green 样本与 `LIB_ABANDONED_ROUNDS_BREAK` 各开关的对应表）是 `--selftest` 与样本档的说明，本腿不跑，没判。

### 4.3 V6 专问：2026-10-12 之前免出处的行，⑥ 与门禁格一收一拒还剩哪些形状

判法差别只有一处：门禁传 `legacy_sourceless_ok=True`，`.claude/hooks/lib_abandoned_rounds.py` 第 175 行 对日期字符串早于 `2026-10-12` 的行整行跳过出处检查；⑥ 不传。另一处是判的范围：门禁判整张表，⑥ 只判开着的轮的行（`.claude/hooks/runner-dispatch-guard.sh` 第 831 行）。

| 形 | 表里的行 | 门禁读法 | ⑥ 读法 | 现在可达吗 |
|---|---|---|---|---|
| A | 日期 ≤ 2026-10-11、没出处、轮开着（有快照目录、有 `_<轮>-body.md`、没有判决） | 收（绿） | 不收，快照照挡（V-7：`out/gate-trees-lib.out` 里 legacy-sourceless 树，库默认读法 `lg-r1 不收`、`--legacy-ok` 读法 `lg-r1 收`；`out/v6.out` 里同形用例派发闸端到端 `⑥ 拒`） | 可达：表头定义日期是腿发出日，2026-10-11 之前发腿、今天撂下的轮就是这一形。冻结副本里开着的轮只有 `m3-batch63-code-r1`，它的行带出处，不落进这一形 |
| B | 日期随便写成更早（`2000-01-01`）、没出处 | 收 | 不收（V-7b） | 可达，但只能让门禁绿，让不了 ⑥ 收，所以不是绕过，是写了一行 ⑥ 永远不认的死行 |
| C | 旧日期、写了出处但出处指错（冻结副本 `c196-r1`：出处那一行没有轮名） | 收（不查） | 不收（`out/lib-table.out`：第 7 行的出处……里没有这一轮的名字） | 同 A 的条件；现在 `c196-r1` 没有快照目录，不被 ⑥ 读 |
| D | 新日期、出处坏、轮不开着（没有快照目录，或已有判决） | 红（判整表） | 不读这一行 | 可达：门禁红而 ⑥ 无事。这是读代码的结论（上面两处调用），没造样本跑门禁格 |
| E | 日期正好 `2026-10-12` | 要出处 | 要出处 | 两边相同（V-7c） |
| F | 同一行 ⑥ 收、门禁拒 | — | — | 没找到。同库同模式，门禁只多一个豁免；门禁另有的「登记了撂下又有判决 ⇒ 红」落在有判决的轮上，⑥ 不把有判决的轮当候选。读代码的结论，没造出 |

冻结副本自己的登记表现在就是 A、C 两形的静态样子：5 行里 4 行（`c196-r1`、`d16-item1-r2`、`c364-r3`、`smalldata-ttime-r1`）门禁收、⑥ 不收；这 4 轮在 `research/prompts/` 下 `ls -d <轮>-snapshot` 与 `_<轮>-body.md` 都不存在（C-9），⑥ 从不读它们，所以今天没有可达的拒绝。

### 4.4 V6 三问

- 过度外推或越位：没有；TI-4、DPR-3 明说认不出的那一半交主 agent。
- 内部矛盾：有两处。一是「不收就是红」（HG6-1、HG6-5、DPR-1、DPR-5、LIB-1）对 A、B、C 形不成立；二是登记表头把日期定义成腿发出日（DPR-2 里引的那一行），库把同一列当「登记日」用来分界（`SOURCE_REQUIRED_FROM`「这一天起登记的行」）。两处都是一收一拒的同一个根。
- 最脆弱：别的轮名的认法（TI-1）。换一个最自然的写法就绕过：出处那一行带一个没有 `-r<数字>` 结尾的名字（`m3-close-b1`）或大写名字，库照收；规则文字说「不许有别的轮名」。
- 什么现象会推翻：TI-1「不」——若 `m3-close-b1` 那一行被库判不收，则变一致；HG6-1「不」——若 `--legacy-ok` 读法下 A 形也不收（即门禁不再传 `legacy_sourceless_ok`），则变一致。

## 5 V8：长活声明文件头、豁免表、⑭ 与文件头共用的现算词表

⑭ 的类型表（十种）已在 AC-1、MA-8 判过。下面是文字与词表、文件头读法、豁免表。

| 编号 | 句 | 判定 | 依据 |
|---|---|---|---|
| MA-9 | 「多机：…行写台数取多机配置（…每台线程取登记的每台默认重活线程数，不现算」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/main-agent.md:55`) | 缺 | 本轮没改这一句，词表落在它上面。照字面写（不点名 `.claude/heavy-thread-default`）退 2「一行没点名 heavy-thread-default」；标准写法后面加「不按核数现算」退 2（核数）；「不看忙闲」「机器空闲时也不变」「负载高也不降」「别的会话在跑也不降」「不除以台数」各退 2；「包含全部卡」退 0（F-12）。句子没写词表、没写「点名」、没写「否定句也算」 |
| HG-8 | 「不许写现算（host-resources、available_threads）与按核数、忙闲、负载折算的说法（available_parallelism、nproc、除以、在跑、核数、忙闲、空闲、负载；否定句里写也算）」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/hooks/runner-dispatch-guard.sh:57`) | 一 | F-12：八个词各自单独写都退 2，`host-resources` `nproc` `available_parallelism` 在否定句里写也退 2；`31 线程` 写死退 2（「写死了额度」）。词表四处逐字相同：`.claude/hooks/lib_long_run.py` 第 78 行、`scripts/agents/dispatch-prompt.py` 第 21 行、本行、`scripts/hosts/detached-run.sh` 第 55 行 |
| HG-9 | 「脚本文件头的 # long-run-hosts: 与派发提示同一种登记值读法…detached-run.sh 与 host-dispatch.py 经 judge_script_header 判」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/hooks/runner-dispatch-guard.sh:60`) | 一 | C-11：豁免表里的脚本 `long-run-check` 退 0、`header_hosts` 判 multi；同一份内容拷到表外：`long-run-check` 退 5（点名「没点名 heavy-thread-default」「写了现算（host-resources）」），`header_hosts` 判 single；旧副本里同一份拷贝判 multi |
| HG-10 | 「只有 .claude/long-run-live-hosts-exempt 登记的脚本文件头按现算读。」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/hooks/runner-dispatch-guard.sh:61`) | 一 | 同 HG-9；符号链接指到表内脚本也按表内读（realpath 后比相对路径），不是文字要拦的东西 |
| DP-4 | 「也不写 available_parallelism、nproc、除以、在跑、核数、忙闲、空闲、负载这些按核数、忙闲、负载折算的说法（否定句里写也算，判在派发闸 ⑭，词表在 lib_long_run.py 的 COMPUTE_PHRASES）」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/scripts/agents/dispatch-prompt.py:21`) | 一 | 同 HG-8；骨架 `多机：` 一行自己的提示句（`dispatch-prompt.py clerk` 打出）不含八个词 |
| DR-1 | 「不许出现现算词（host-resources、available_threads、available_parallelism、nproc、除以、在跑、核数、忙闲、空闲、负载，否定句也不行）」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/scripts/hosts/detached-run.sh:55`) | 一 | 同 HG-9（文件头走 `judge_script_header`，与提示共用 `judge`） |
| DR-2 | 「只有 .claude/long-run-live-hosts-exempt 登记的脚本（被 sha256 钉住、真按此刻空闲现算的 E171 六份入口）这一行按现算读」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/scripts/hosts/detached-run.sh:56`) | 缺 | 「六份」「表里的才按现算读」一致（`exempt-check` 6 行；C-11）。「被 sha256 钉住」没有机器核：`exempt_table_problems` 不读 sha256；我在冻结副本里找到的钉是 `research/prompts/m3-leaf-death-r3-snapshot/tree-sha256.txt` 里这六个路径的行，六份现行文件的 sha256 与它**逐份都不同**（`out/lr.out` 末六行 `DIFF`，C-10），那三轮 m3-leaf-death 都已有判决 |
| HD-1 | 「judge_script_header 判不齐（一行都没写、半截声明、hosts 一行没点名 --emit-hosts 与登记值、写了现算词或写死额度）⇒ 只占一台，判出的问题写进理由」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/scripts/hosts/host-dispatch.py:26`) | 一 | C-11：表外拷贝 `header_hosts` 返回 single，理由首句「文件头的长活声明不齐：# long-run-hosts: 一行没点名 heavy-thread-default……」；表内脚本返回 multi |
| HD-2 | 「与派发提示里的…多机：…一行同一种读法，按登记值读：台数点名 --emit-hosts、每台线程点名 .claude/heavy-thread-default」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/scripts/hosts/host-dispatch.py:22`) | 一 | 同 HD-1 |
| HD-3 | 「出路：改成台数按 layer0-shard-configuration-check.sh --emit-hosts 读、每台线程取登记值」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/scripts/hosts/host-dispatch.py:29`) | 一 | 出路文字与 `LONG_RUN_HOWTO`（`scripts/hosts/host-dispatch.py` 第 613 行）同一写法；与 `lib_long_run.py` 报的合规写法一致 |
| LR-1 | 「脚本文件头与派发提示都按…登记值…读：台数点名 --emit-hosts，每台线程点名 heavy-thread-default，不许现算词；」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/hooks/lib_long_run.py:10`) | 一 | 同 HG-9 |
| LR-2 | 「"registered"（默认：派发提示与脚本文件头走这一种，用户 2026-10-12 定、2026-10-13 把文件头也并过来）」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/hooks/lib_long_run.py:18`) | 一 | `.claude/hooks/lib_long_run.py` 第 116 行 默认值就是 registered；调用方里 `judge(` 的位置都显式传读法或经 `judge_script_header`（`grep` 全仓 `long_run.*judge(` 只有派发闸传 `hosts_reading="registered"` 与 host-dispatch 的两个弄坏分支传读法） |
| LR-3 | 「脚本相对本库所在仓根的路径在 LIVE_HOSTS_EXEMPT_TABLE…里的按 "live" 读，别的一律 "registered"。」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/hooks/lib_long_run.py:25`) | 一 | C-11 的表内 / 表外对照；本库所在仓根 = `PROJECT_ROOT`（`.claude/hooks/lib_long_run.py` 第 82 行），在副本里跑的库查副本的表 |
| LR-4 | 「表里只有按现算写、被 sha256 钉住不能改的脚本；」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/hooks/lib_long_run.py:26`) | 缺 | 同 DR-2：表的写法、路径、读法、只缩不涨有机器判，「被 sha256 钉住」没有 |
| EX-1 | 「指向不存在的路径、脚本文件头已经按登记值写齐、按现算读法也不齐、比改动范围的基准那一版多出的行，都判红」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/long-run-live-hosts-exempt:4`) | 一 | `exempt-check` 对 `gate-trees/exempt-table`（三行：缺路径、头已按登记值写齐、E171 式现算头）输出两处不对（缺路径、已按登记值写齐），现算头那一行放（`out/gate-trees-lib.out`）。「多出的行」要基准表：库的 `--base-table` 我没喂，门禁格 `.claude/gate.d/code-tooling.sh` 第 2405 行 取基准那一半归主 agent 跑 |
| EX-2 | 「只缩不涨：改成登记值读法或脚本不再需要就删掉那一行，新脚本不往这里加。」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/long-run-live-hosts-exempt:5`) | 一 | 同 EX-1；基准里没有这张表时成功句写明「只缩不涨这一条没比」（冻结副本自己的表：`exempt-check` 输出，`out/lr.out`） |
| EX-3 | 「# E171 的六份多机入口被 sha256 钉住不能改，它们真按此刻空闲现算每台槽数与线程，固定每台 31 线程归 E171 下一次修订」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/long-run-live-hosts-exempt:7`) | 缺 | 六份：对。「被 sha256 钉住」同 DR-2。「真按此刻空闲现算」：入口头自己写 「每台槽数与每槽线程从 scripts/hosts/host-resources.py --json 现查现算」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes/run_segment_one_revision_11_multi_host.sh:9`)。「固定每台 31 线程归 E171 下一次修订」是一句承诺，没有任何一道检查把表里的行与那次修订连起来 |
| CT-1 | 「判法只在 .claude/hooks/lib_long_run.py 的 exempt-check（用这一道所在仓里那一份，被判的根照传）」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/gate.d/code-tooling.sh:2379`) | 一 | `.claude/gate.d/code-tooling.sh` 第 2426 行 起的就是 `$STAGE_REPOSITORY/.claude/hooks/lib_long_run.py`；门禁格本身（`--check long-run-header-exempt`）我没起，归主 agent（第 8 节） |
| CT-2 | 「里那一版比，基准里没有的行判红；基准里还没有这张表、不在 git 仓里，这一条不比、成功句里写明。」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/gate.d/code-tooling.sh:2381`) | 一 | 库的成功句里有「没给基准……只缩不涨这一条没比」（`out/lr.out`）；格里 git 与基准的分支（`.claude/gate.d/code-tooling.sh` 第 2410 行 取基准那一版）没跑 |
| SO-1 | 「长活声明按现算读的脚本豁免表与它的判法（lib_long_run.py exempt-check）由工具实现员改；表里的脚本被 sha256 钉住，红了删行或改脚本归实验那一边」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/gate.d/stage-owners.tsv:48`) | 缺 | 「红了删行或改脚本归实验那一边」与「表里的脚本被 sha256 钉住」同时写：钉住就不能改脚本，只剩删行；同 DR-2 |

### 5.1 V8 专问

**豁免表与 E171 下一次修订的衔接。**
- 表就是 E171 六份入口：`exempt-check` 报 6 行都在、都得靠现算读法才齐（`out/lr.out`）；`ls scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes/ | grep multi_host` 除两份 `fake_multi_host_run.*` 外正好这六份。
- 下一次修订新增的入口不在表里：文件头按登记值读，点名 `--emit-hosts` 与 `heavy-thread-default`、不许八个现算词（C-11 的拷贝对照：把旧入口原样拷到表外，`long-run-check` 退 5）。表只缩不涨、新脚本不往里加（「新脚本不往这里加」(`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo/.claude/long-run-live-hosts-exempt:5`)），所以新入口只有两条路：真改成登记值（EX-3 说的「固定每台 31 线程归 E171 下一次修订」），或写 `# long-run-unshardable:` 点名缺多机。以旧入口为蓝本整份拷贝而文件头不改，一定被拒。
- 派发提示一侧没有豁免：⑭ 对 experiment-runner 等的「多机：」一行只认登记值读法（HG-8、F-12）。派发这六份旧入口时，提示里写的「台数取 --emit-hosts、每台线程取 .claude/heavy-thread-default」与入口实际做的（现查现算每台槽数与线程，EX-3 里引的那一行）不符；提示与文件头各自都过判法。MA-9 与 HG-9 的文字对这六份不成立。
- 表行不会随 E171 修订自动变：「被 sha256 钉住」没有机器核（DR-2，C-10 六份都与找得到的钉不同），「只缩不涨」要靠人删行。

**词表与「台数取多机配置」那一句的交集。** 实测见 F-12：`核数`、`忙闲`、`空闲`、`负载`、`在跑`、`除以` 各自出现在「多机：」行里都拒，包括前面带「不」的写法。最自然的合规写法「……不按核数现算」「……不看忙闲」（MA-9 与 main-agent 第 53 行用的就是「不看忙闲」这个词）被拒。拒绝说明自己写了「否定句也不行」，但 main-agent.md、agent-common.md 里没有这一句，也没有列词表，词表只在钩子文件头、`dispatch-prompt.py` 文档串、`lib_long_run.py`、`detached-run.sh` 文件头。

### 5.2 V8 三问

- 过度外推或越位：词表按子串匹配文本，判的是用词，不是行为；否定句也拒，使一行里不能解释「为什么不这么取」。这是故意的（拒绝说明明说），但规则文字没跟上，所以记「缺」。
- 内部矛盾：一是 E171 六份入口——文件头按现算读而提示必须按登记值读（见专问）；二是 DR-2、LR-4、SO-1 三处同写「被 sha256 钉住」，没有一道检查核它，且找到的钉与现行文件不符。没有循环依赖：`judge_script_header` 查表、表的检查调 `judge`，表自己不读自己。
- 最脆弱：「多机：」行的现算词表。换一个最自然的写法就误伤（`不按核数现算`），也无法区分「写了现算」与「写了不现算」。
- 什么现象会推翻：HG-8「一」——若 `不按核数现算` 退 0，则「否定句里写也算」不成立；DR-2「缺」——若 `exempt-check` 开始比 sha256（或 C-10 的六份变为 same），则变一致；MA-9「缺」——若 main-agent.md 补上词表与「否定句也算」，则变一致。

## 6 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| V1 | 大体一致，三处不严 | 九种类型表（⑭ 十种、⑮ 九种）在派发闸、骨架、两份规则、各定义里逐个相同，三种新类型的预计挂钟与线程上限行钩子真拒、骨架真带；不严的：括注数词表按字符判（`统一` `万一` `十分` 被拒，`按核数取` `忙时降低` 被放）；括注里的尖括号不判；`线程上限已判` 能顶替缺行而定义只写了「N 不是登记值」；钩子注释说 forward 与 defense 的定义许编译，forward 定义里没有；dispatch-prompt.py 把 clerk 叫「书记员」（那是 kb-scribe 在钩子里的叫法） |
| V2 | 一致（规则句，无机器判） | 两句都只约束主 agent；「明说」「只看到负载忙不算」钩子判不了，已判行理由写什么都放；SendMessage 钩子不看线程上限 |
| V4 | 一致 | `--test-threads 1` 带与没带的名单、CPU 一侧读 `SINGLEFS_CRASH_JUDGE_CPU_THREADS` 没设取核数、`capped.sh` 不设这个变量，逐项与源码相同；旧副本「CPU 一侧照 capped.sh default 交的登记值」是错的，新句对。名单是手写的，没有检查把它与共用库绑在一起 |
| V6 | 不一致，两个根（七句） | 一是「除这一轮的轮名外不许有别的轮名」，库只认 `<名>-r<数字>` 与登记过的名字，`m3-close-b1`、`OTHER-R2` 照收；二是「不收就是红」（钩子注释、库文档串、门禁头注、门禁与派发闸的提示）对日期早于 2026-10-12 的行不成立：门禁读法收、⑥ 读法不收，而表头把日期定义成腿发出日，不是登记日 |
| V8 | 大体一致，缺五句 | 现算词表在 ⑭、文件头、骨架文字、库四处逐字相同，文件头与提示共用 `judge`，豁免表真放 E171 六份、拒表外拷贝；缺：main-agent 第 55 行没写词表与「否定句也算」（`不按核数现算` 被拒）；「被 sha256 钉住」无人核且六份现行文件与找得到的钉都不同；E171 六份按现算而派发提示必须写登记值读法 |

逐句判定的总数（对第 1 至 5 节的表数行首编号，同一编号去重）：87 条，一致 56、不一致 12、文字没写到 19；表里共 93 行，AC-5、AC-6、AC-7、AC-8、MA-6、MA-7 在第 1 节和各自的格里各出现一次（判定相同）。V5 的句子（agent-common.md 第 81 行三句）与两份文件头 / 样本说明（`--selftest` 的开关对应表）不在这个数里。

## 7 喂法与复跑

复跑（整条经内存包装，工作目录随意，约 2 分钟）：

```
cd /home/fy5090/code/singlefs
OLD_FROZEN=~/.local/share/singlefs/work/m3-defs-b3-r2/repo bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/m3-defs-b6-r1-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen ~/.local/share/singlefs/work/m3-defs-b6-r1/repo --work <新的草稿目录>
```

结果：退出 0；最后 12 行 `name=rerun_compare file=<文件> result=same`（`grep -c 'result=same'` 输出 12，`grep -vc` 输出 0）。我跑了两遍都是 12 行 same。存的输出在 `research/prompts/m3-defs-b6-r1-sonnet-model/out/`，路径已换成 `<WORK>` `<NEW>` `<OLD>` 再比。

| 编号 | 喂的是什么 | 输出在哪 |
|---|---|---|
| F-1 | clerk、three-way-forward、three-way-defense 各三例：无预计挂钟；超过 2 小时不带三行；三行齐（`feed.py`，`--check-prompt`，新旧两份副本） | `out/feed.out` 的 `V1 <类型> 没有预计挂钟` 等；全文 `out/feed-raw.txt` |
| F-2 | clerk 线程上限行 `31（…）` 的括注二十余种（数词、统一、唯一、十分、万一、半角括号不认、各自、不减半、合计、分之、同上、每台默认、任意说明） | `out/feed.out` 的 `V1 clerk 线程上限：…` |
| F-3、F-3s | `31（<登记值>）` `31（<按机器忙闲定>）` 与顶格 `<登记值>` `<填 线程上限>`；stdin 入口的同例 | `out/feed.out` 的 `V1 clerk 线程上限行写 括注里的尖括号 …`、`线程上限：<…>`；`out/stdin.out` 第四例 |
| F-4 | 五类（clerk、forward、defense、tooling-writer、investigator）无线程上限行；无行加已判；30 不加已判；30 加已判 | `out/feed.out` 的 `V1 <类型> 没有线程上限行…` |
| F-5 | `31（按核数取）` `（忙时降低）` `（负载高就少起）` `（nproc）` `（host-resources 现算）` | `out/feed.out` 的 `V8x clerk ⑮ …`（全部退 0） |
| F-6 | 半角括号、全角数字、两段括注、整数后接逗号 | `out/feed.out` |
| F-7 | `31÷2` `31/2` `31 一半` `31，即 15` `31（本机），第二台 16`、括注里 `÷` `/` | `out/feed.out` 的 `V1b …` |
| F-8 | 超过 2 小时、「分不了：」点名多机 | `out/feed.out` 的 `V8 clerk 分不了 点名多机` |
| F-9 | tooling-writer、investigator 同 F-1、F-4 | `out/feed.out` |
| F-10 | `SINGLEFS_THREAD_CAP=16` 时 `线程上限：16` 退 2、`31` 退 0；`与第二台同值` `与另一台同值` `两台同值` 退 2，`本机，不看忙闲` 退 0 | `out/feed.out` 的 `V1e …`、`V1n …` |
| F-11 | `线程上限：16` 加 `线程上限已判：<理由>`：机器忙、用户明说、`x`、随便、`<理由>`、`无`；已判写在句中 | `out/feed.out` 的 `V2 …` |
| F-12 | 超过 2 小时，「多机：」一行 14 种写法（标准、加不按核数现算、核数、不看忙闲、空闲、负载、在跑、除以、字面照 main-agent、available_parallelism、host-resources、写死 31 线程、包含全部卡、nproc） | `out/feed.out` 的 `V8 clerk 多机 …` |
| stdin | 七例按 hook 的 JSON 喂 stdin 入口（`feed_stdin.py`，设 `DISPATCH_GUARD_STATE_ROOT` 指到草稿目录，每例自己的草稿目录） | `out/stdin.out` |
| C-1 | `grep -n '编译'` 三份定义；`grep -c test-threads` 三个 crash-amplification 脚本；admission.py 里的 test-threads | `out/capped.out` 后半 |
| C-2、C-4、C-7、C-9 | 登记读不到时 ⑮ 放行；continuation-guard.sh 里「线程」计数；legacy 树派发闸说明行；旧行的轮有没有快照 | `out/adhoc.out` |
| C-3 | `dispatch-prompt.py <类型>` 的骨架里线程上限与预计挂钟四行，新旧两份副本 | `out/adhoc.out` 末段（旧副本 clerk 只有线程上限行，forward、defense 一行都没有） |
| C-5 | `capped.sh abc true` 退 2 并打出路；`capped.sh default true` 退 0；`capped.sh 7` 起的进程里 `SINGLEFS_CRASH_JUDGE_CPU_THREADS=unset`；`capped.sh 7 capped.sh default` 里 `SINGLEFS_THREAD_CAP=7` | `out/capped.out` 前半 |
| C-8 | 完整钩子对 ot-r1 放行；去掉 `lib_abandoned_rounds.py` 的钩子拒并写读不到库 | `out/nolib.out` |
| C-10、C-11 | 六份脚本现行 sha256 与 `m3-leaf-death-r3-snapshot/tree-sha256.txt` 的钉比；`long-run-check` 表内、表外；`header_hosts` 表内、表外（新旧）；`exempt-check` 冻结副本自己的表 | `out/lr.out` |
| V-1…V-9 | 小树里的叫停轮登记行二十余种写法，库两种读法加派发闸端到端（`v6.py`）：V-1 `m3-close-b1`、V-2 `OTHER-R2`、V-3 日期与重复、V-4 `2026-13-45`、V-5 开轮行、V-7 旧日期无出处（V-7b `2000-01-01`、V-7c 边界日）、V-8 `other-r2` 与登记过的名字、V-9 自己的材料 / 越界 / `..` | `out/v6.out` |
| V-6 | 符号链接出处、records 子目录出处 | `out/gate-trees-lib.out` 的 symlink-source 两段 |
| V-7 | 冻结副本自己的登记表两种读法 | `out/lib-table.out` |
| tables | 派发闸两张表（经 `dispatch-prompt.py` 的 `guard_type_tables`）、骨架两张表、agent-common 与 main-agent 里列的两串、各定义「输入」里有没有两行 | `out/tables.out` |

`out/` 里 sha256（`cd research/prompts/m3-defs-b6-r1-sonnet-model/out && sha256sum *`）：

```
c0bb7a39fd1bb0291d84e88570919f73d207908877f7bacebb1d9ca35ee0a61f  adhoc.out
30b8035a6dd7f9c6cbf6517f91d3510b46b0872ae38d619cb2083b5b5583a824  capped.out
476a8d8fd9f546f735b4e7b89c51192d92f2aff3e069a39762441b1656b8ca8d  feed-raw.txt
6051a58883e7eb2b57e974757cd5335f82a9bf41fbc2e649a5f89e6000df61f6  feed.out
ea1233bf008ca19441f9a705dc9ddd2f8b1f18add8c3e0474855537536563a16  gate-trees-lib.out
8676f082480ae7b52fcfb6f624f0036a6d9eeb832d2b97847f7ec7cd0ad8eaea  gate-trees.out
4798505154dbc6bd3dbeb5e8330ac7e1de2a6259039a04a9737236751b5d7ffe  lib-table.out
3af5976990e5a4ae381a2b689a9eda242a727f589d231b6ac62d56725aaaec84  lr.out
7c3edfeffbb09f336bba5230fd11d1941d5a2dfbdc936a00e1f599def8e15d31  nolib.out
ab33073cca01ea45d9bb1233625acf1422837b587f5edf7968a3a688b42c24f3  stdin.out
a21668a36dc7c1edfd6ca996194f35f03a0580c412f68a56e28efb21671922fa  tables.out
2bc1ab6f81e1a08268ee9983d65b9c9f0b4c385ede0055c6ad9a5694615a1704  v6.out
```

## 8 要主 agent 跑的命令（门禁格，子 agent 起不了 `--check`）

树由 `research/prompts/m3-defs-b6-r1-sonnet-model/tools/make_gate_trees.sh <新目录>` 造（`gate-trees` 四棵：`legacy-sourceless` `other-name-unrecognised` `exempt-table` `symlink-source`）。下面「预期」是我读格的代码推出来的，没起过格。

1. `cd <仓根或冻结副本> && bash .claude/gate.d/doc-process-records.sh --check abandoned-rounds <树>/legacy-sourceless`。预期（推）：绿；成功行报「出处……查了 0 行，1 行日期早于 2026-10-12、不要求出处」。同一棵树我已让库按两种读法判过（`out/gate-trees-lib.out`：⑥ 读法不收、门禁读法收）和派发闸端到端（`out/adhoc.out` 的 C-7：⑥ 拒）。要看的是：格绿而 ⑥ 拒，坐实 4.3 的 A 形。
2. 同命令，树换成 `other-name-unrecognised`。预期（推）：绿；库输出 `ot-r1 收`（`out/gate-trees-lib.out`）。要看的是：规则句「不许有别的轮名」下这棵树仍绿，坐实 TI-1。
3. `cd <仓根或冻结副本> && bash .claude/gate.d/code-tooling.sh --check long-run-header-exempt <树>/exempt-table`。预期（推）：红，两处（缺路径、已按登记值写齐），格不是 git 仓，只缩不涨那一半没比；库直接调的结果在 `out/gate-trees-lib.out`。格里取基准那一半（`gate_diff_base gate`、`git show`）只有在 git 仓里跑才走到，归主 agent。
4. 想看「只缩不涨」红：在一个 git 仓里提交一版只有 `scripts/live.sh` 一行的表，再加回 `scripts/other.sh` 一行再跑第 3 条，预期红「是基准那一版里没有的（豁免表涨了）」（推，代码在 `lib_long_run.py` 的 `exempt_table_problems`）。

## 9 没做什么

- 没起门禁格（`abandoned-rounds`、`long-run-header-exempt`、`heavy-thread-defaults` 等），没跑任何 `--selftest`、样本档；样本说明（弄坏开关对应表）没判。第 8 节是这一批要主 agent 跑的。
- 没判 V3、V5、V7 的句子，也没判 agent-common.md 第 81 行新增的三句（V5）。
- 没替主 agent 采纳或出判决；表里的「不」「缺」是对照结果，不是改法。
- 引文口径：规则文件里的长行（agent-common.md 第 85 行 1724 个字符）按分句抄，行号是分句所在的整行；cite-check 的引文上限 400 字，且引文里不能有「」，含「」的句子切成几段用 `…` 连。表里「整条」抄的是定义里的整条输入行。
- 旧冻结副本（`m3-defs-b3-r2`）只喂了钩子与判法函数，没引它的行号；「打中也打中零轮之前」的账是攻方腿与主 agent 的，本报告只在喂法里并排列了新旧退出码。
- stdin 入口只喂了七例，其余走 `--check-prompt`（同一个 `judge_dispatch`，`check_only` 不读会话记录、不建写者锁）；⑤ 并发撞文件、⑦ 并发与限额、⑯ 写者锁这三条没喂，也不在本腿的格里。
- 没读禁读清单里的文件（主工作区被判的定义、规则、钩子、脚本与门禁；调度记录；`m3-tooling-*` 实现员自报；攻方腿的报告与模型目录）。唯一例外是开工先读的主工作区 `.claude/agent-common.md`，见第 0 节。
- `multi-host.env` 的值没读。
- D 形（门禁红而 ⑥ 无事）与 F 形（⑥ 收而门禁拒不存在）是读代码的结论，没造样本跑门禁格。
- 内存上限（`内存上限；` 两句）没有钩子判，没法验。
- 重型测试：没跑。

## 10 交回前核

命令：`python3 scripts/agents/cite-check.py research/prompts/m3-defs-b6-r1-sonnet-output.md`（默认根）与加 `--root ~/.local/share/singlefs/work/m3-defs-b6-r1/repo --background research/prompts/_m3-defs-b6-r1-background.md --background research/prompts/_m3-defs-b6-r1-body.md` 各跑一遍，末行相同：

```
  ✓ 核了 100 处引文，对上 100 处，没判 0 处（逐处列在上面）
```
