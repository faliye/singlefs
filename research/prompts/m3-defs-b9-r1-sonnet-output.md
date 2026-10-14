# 第二题第一轮：云端正推腿（Sonnet）报告

日期：2026-10-13。轮名 `m3-defs-b9-r1`。分到的格：Z1、Z3、Z4、Z6、Z7。立场：逐条核「定义与规则写的就是钩子、脚本、门禁做的」。

## 范围与读法

- 被比的句子：冻结副本 F=`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/` 对 M 形态之前那一版 `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/` 逐文件比出的零轮新写或改写的句子。命令 `diff -rq m3-defs-b6-r2/repo m3-defs-b9-r1/repo --exclude=target --exclude=.git`：36 份文件不同、15 处只在 F 里有（新增样本目录）。再把不同的行按「；」「。」切句、比两边的句集（草稿目录 `tools/sdiff.py`），得到下面表里的句子；定义九份「输入」里逐字相同的句子合成一行、列出各处位置。
- 一格一句：表里第二列「引文」整句抄、写那份文件自己的行号（F 里的绝对路径，行号由 `rows/locate.py` 在 F 里现取：片段按次序出现在恰好一行里，命中不是一行就报错）。句子里自带「」的，按 cite-check 的省略写法（`…`）切成不含「」的几段连起来；因此每段都是原文整段，段与段之间是原文里被「」括起的部分。
- 判定三种：一致 / 不一致 / 文字没写到。对子 agent 的要求（它起不起重活、读不读围栏里的行）源码里没有第二处执行，写成「一致（只有定义管，源码不判）」，另列文字没写到的部分。
- 喂法图例（全部在模型目录 `research/prompts/m3-defs-b9-r1-sonnet-model/`，复跑命令见末尾）：
  - 喂法 A=`out/z1-thread-prompt.tsv`：派发闸 `--check-prompt` 喂线程上限（⑮）的 56 例，只看「线程上限」这一项拒没拒。
  - 喂法 B=`out/z6-long-run-prompt.tsv`：同一入口喂预计挂钟与长活声明（⑭）的 32 例。
  - 喂法 C=`out/z3-heavy-guard.txt`：重型测试闸钩子入口喂 stdin JSON（仓副本里改过正文的自证），7 例带退出码与整条 stderr。
  - 喂法 D=`out/z4-abandoned.tsv`：叫停登记行的共用判法库（`lib_abandoned_rounds.py` 命令行）喂 34 例。
  - 喂法 E=`out/z6-script-header.tsv`：脚本文件头的多机一行经 `detached-run.sh long-run-check` 喂 10 例。
  - 喂法 F=`out/z7-exempt.tsv`：`lib_long_run.py exempt-check` 带不同基准喂 13 例。
  - 喂法 G=`out/real-abandoned-table.txt`、`out/real-exempt-table.txt`、`out/real-round-names.txt`：F 里真表真记录。
- 门禁格（`--check`）子 agent 起不了：Z4-13、Z7-07 对应的格没跑，命令在 `main-agent-to-run.md`，标「要主 agent 跑」。

## 越界读的披露（先于本报告交代）

开工时定义「开工先读」与共用约束点名我读 `.claude/agent-common.md`、`.claude/rules/three-way-inference.md`（主工作区的版本，整份）；这两份正是被判文件，禁读清单里「主工作区里被判的定义、规则」包含它们。我当时先读了，后来才对上这条禁读。影响：只用来知道规则怎么写；本报告所有句子、行号、判定都取自 F 的副本，没有引用主工作区的那两份。另外：①我对主工作区 `.claude/hooks/runner-dispatch-guard.sh` 只做了 `sha256sum`，得到 `765307b8…` 与 F 里 `135c7efa…`（`research/prompts/m3-defs-b9-r1-snapshot/frozen-copy-sha256.txt` 登记的就是 F 这一份）不同，由此定下「引文写 F 的绝对路径」；②为验 cite-check 的写法对主工作区的 `scripts/agents/cite-check.py`、`.claude/hooks/handback-guard.sh` 头部读了几十行，它们不在 Z 格里；③没有读 `multi-host.env` 的值（F 里那份文件我没打开，`rerun.sh` 只拷 `.claude` 与 `scripts`、不拷根下文件）。

## Z1　线程上限只照一个整数、围栏、已判行、clerk

### 专问答复

- CommonMark 围栏之外的写法（喂法 A 的例名在括号里）：波浪线围栏——不当围栏，里面的行照判（「波浪线围栏里的 16」拒）；缩进 4 格的代码块——不当代码块，照判（「缩进 4 格的代码块里的 16（前有空行）」拒）；列表项里的围栏、引用块里的围栏——不当围栏，照判（「列表项里的围栏里的 16」「引用块里的围栏里的 16」拒）；没闭合的围栏——不当围栏，后面的行照判（「没闭合的围栏后面的 16」拒）。闭合的三反引号块（至多缩进 3 格、信息串无反引号）里的行不判（「代码块里的 16 加块外的 31」放行）。⑭ 同一个函数、同一种结果（喂法 B 的 tilde、list、unclosed 三例拒）。
- 只有已判行时子 agent「不起重活」，钩子与定义谁判得到：只有定义。钩子 ⑮ 把这种提示放行（九类各一例，喂法 A 的 `九类-只有已判行-*`）；capped.sh 只验 N 是正整数或 default，不读派发提示（Z1-01）。定义一侧的文字在 agent-common.md:61 与 main-agent.md:53（Z1-02、Z1-06）；九份定义的「输入」只写已判行「也能顶替缺的行」，没写顶替后的后果。
- 最脆弱的一句（我的格内）：Z1-10「认围栏照 CommonMark」，说过头；换一个自然写法（把示例放进列表项里的围栏）就被误伤，见下表。

### 句表

| 编号 | 引文（F 里的位置） | 判定 | 依据；什么现象会推翻 |
|---|---|---|---|
| Z1-01 | 「N 只照派发提示的…那一个整数，提示里别处写的线程数一概不照（它把 cargo 与 crates/singlefs-harness 各装置读的线程变量一次设成 N）」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/agent-common.md:61`） | 一致（只有定义管，源码不判） | 这是对子 agent 的要求。钩子 ⑮ 只判行首「线程上限：」那一行，提示别处再写线程数照放（喂法 A「别处写了别的线程数」线程上限被拒=否）；capped.sh 只核 N 是正整数或 default，不对派发提示（scripts/hosts/capped.sh:56）。推翻：全仓找得到把子 agent 起的 capped.sh N 与派发提示的 N 对过的钩子或脚本。 |
| Z1-02 | 「派发提示里没有…一行、只有…的，不起重活，要起就停下交回，报告里写明要主 agent 补一行」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/agent-common.md:61`） | 一致（钩子放行这种提示；子 agent 照不照只有定义管） | 专问「只有已判行时子 agent 不起重活，钩子与定义谁判得到」：钩子判不到。九类各喂「只有已判行」都放行（喂法 A「九类-只有已判行-*」九行线程上限被拒=否），已判行没写理由或写在代码块里则拒（「九类-已判行没写理由-*」「代码块里的已判行顶不了缺行」）；子 agent 起不起重活没有任何源码核。九份定义的「输入」只写已判行「也能顶替缺的行」，没写顶替后不起重活，这一后果只在本句与 main-agent.md:53。推翻：找到读派发提示并拦子 agent 起重活命令的钩子。 |
| Z1-03 | 「三个反引号围起来的代码块里的行 ⑮ 不判」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/main-agent.md:53`） | 一致（闭合的三反引号块）；文字没写到（围栏种类、闭合、子 agent 怎么读块里的行） | 喂法 A：闭合块里的 16 加块外的 31 放行（「代码块里的 16 加块外的 31」）。文字没写到的三处：波浪线围栏里的 16 照判拒（「波浪线围栏里的 16」）；没闭合的围栏后面的 16 照判拒（「没闭合的围栏后面的 16」）；缩进 4 格的代码块里的 16 照判拒（「缩进 4 格的代码块里的 16（前有空行）」）。块外 31 加块内 16 时，子 agent 读到两个数，文字只说「那一个整数」，没说块里的不算。推翻：钩子对这几种写法放行，或文字补了。 |
| Z1-04 | 「要加说明另起一行，照普通正文写」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/main-agent.md:53`）；同句另见 `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/agents/clerk.md:24`、`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/agents/implementation-writer.md:27`、`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/agents/investigator.md:25`、`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/agents/mutation-triage.md:25`、`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/agents/three-way-attack.md:24`、`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/agents/three-way-defense.md:25`、`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/agents/three-way-forward.md:25`、`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/agents/tooling-writer.md:25`、`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/agents/experiment-runner.md:24` | 一致；文字没写到「普通正文」不能自己以「线程上限：」起头 | 喂法 A：说明行不以「线程上限：」起头放行；普通正文行也以「线程上限：」起头被当第二个线程上限行，数字不等于登记值则拒（「普通正文行也以线程上限冒号起头」）；旧写法「线程上限说明：」另起一行放行（「旧写法线程上限说明冒号另起一行」），全仓已无「线程上限说明」的残留写法（grep 全仓只剩钩子自检里的反向断言）。推翻：说明行以「线程上限：」起头被放行。 |
| Z1-05 | 「子 agent 起重活只照…那一个整数，提示里别处写的线程数一概不照」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/main-agent.md:53`）；同句另见 `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/agents/clerk.md:24`、`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/agents/implementation-writer.md:27`、`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/agents/investigator.md:25`、`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/agents/mutation-triage.md:25`、`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/agents/three-way-attack.md:24`、`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/agents/three-way-defense.md:25`、`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/agents/three-way-forward.md:25`、`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/agents/tooling-writer.md:25`、`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/agents/experiment-runner.md:24` | 一致（同 Z1-01，只有定义管） | 同 Z1-01 的依据。九份定义里这一句逐字相同。推翻：同 Z1-01。 |
| Z1-06 | 「这一行也能顶替缺的…行，顶替了的派发子 agent 不起重活，要起就停下交回、要主 agent 补一行…（`.claude/agent-common.md` 起重活那一条）」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/main-agent.md:53`） | 一致（同 Z1-02） | 后半括注里的「九类缺行也没有已判行则拒」与钩子相符：九类各喂「缺行也没有已判行」线程上限被拒=是（喂法 A「九类-缺行也没有已判行-*」）。推翻：九类里有一类缺行放行。 |
| Z1-07 | 「说明另起一行照普通正文写；三个反引号围起来的代码块里的行不判」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/agent-common.md:85`） | 一致（⑮）；文字没写到（⑭ 同样不判围栏内的行、同一标签第二行拒） | 这一句挂在 ⑮ 条下。⑭ 条（同一份第 85 行）只写多机一行的规范句，没写「围栏里的预计挂钟、分片、续跑、多机行不判」和「分片、续跑、多机同一标签第二行就拒」：喂法 B「fence-hosts-example-block-then-canon」放行、「wallclock-only-in-fence」拒、「dup-shards」「dup-resume」「canon-then-dup-label」拒。grep 「只写一行」在 agent-common.md、main-agent.md、九份定义、dispatch-prompt.py、detached-run.sh 里零命中，只在钩子文件头与 lib_long_run.py。推翻：这两条写进定义或共用约束。 |
| Z1-08 | 「要说明的另起一行照普通正文写，那一行不以…起头就不判」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/hooks/runner-dispatch-guard.sh:67`） | 一致 | 喂法 A：「说明行不以线程上限冒号起头」放行、「普通正文行也以线程上限冒号起头」拒。推翻：以「线程上限：」起头的普通正文行放行。 |
| Z1-09 | 「三个反引号围起来的代码块（围栏闭合了的）里的行不判（示例行、喂法样本放进去不被当成这一行，里面的…也不当放行行）；围栏没闭合的，后面的行照判」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/hooks/runner-dispatch-guard.sh:68`） | 一致 | 喂法 A：「代码块里的已判行顶不了缺行」拒、「没闭合的围栏后面的 16」拒、「N 不等于登记值而已判行在代码块里」拒。推翻：块里的已判行顶得了缺行，或没闭合的围栏遮住后面的行。 |
| Z1-10 | 「认围栏照 CommonMark，⑭ 与这一条经同一个函数（without_code_blocks）：围栏行至多缩进 3 个空格，开头那一行反引号后面的语言名里不许有反引号」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/hooks/runner-dispatch-guard.sh:69`） | 不一致（「照 CommonMark」说过头） | 同一个函数（钩子 without_code_blocks 在 1090 行，⑮ 在 1117 行用、⑭ 在 1278 行用）、缩进 3 格、信息串无反引号三处与代码相符（喂法 A「开头围栏缩进 4 格不是围栏」「开头围栏用制表符缩进不是围栏」「围栏信息串里有反引号」「围栏闭合行缩进 4 格」）。不符：CommonMark 里列表项内、引用块内的围栏是围栏，块里的行是代码；钩子不认，块里的 16 照判拒（喂法 A「列表项里的围栏里的 16」「引用块里的围栏里的 16」，对应 ⑭ 喂法 B「list-fence-hosts-example」拒）；波浪线围栏同（「波浪线围栏里的 16」「tilde-fence-hosts-example」）；没闭合的围栏 CommonMark 当块到文末，钩子当普通行（上一句 Z1-09 自己写了）。推翻：把列表项里、引用块里的三反引号围栏喂进去，块里的 16 放行。 |
| Z1-11 | 「是行内代码，不开块，后面一行照判；缩进 4 格或用制表符缩进的三个反引号也不是围栏）」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/hooks/runner-dispatch-guard.sh:70`） | 一致 | 喂法 A：「行首行内三反引号后面的 16」拒、「行内三反引号后真代码块里的 16」放行、两个缩进开头围栏的例子拒。推翻：这几例换结果。 |
| Z1-12 | 「每个标签只写一行：同一个标签出现第二行就拒（只读第一行，第二行写什么都没人判；lib_long_run.py 的 duplicate_prompt_problems）」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/hooks/runner-dispatch-guard.sh:63`） | 一致（预计挂钟超过 2 小时时） | 喂法 B：「dup-shards」「dup-resume」「canon-then-dup-label」「canon-then-dup-label-bold」「canon-then-dup-label-listmark」「canon-then-indented-label」「canon-then-halfwidth-colon-dup」都拒；「within-2h-dup-hosts」放行（2 小时内不判三行）；「dup-unshardable-ok」放行（分不了不在重复检查的标签里）；「canon-then-midline-label」放行（标签不在行首）。推翻：2 小时内的重复标签也拒。 |
| Z1-13 | 「与 ⑮ 一样不判三个反引号围起来的代码块（围栏闭合了的）里的行：预计挂钟、分片、续跑、多机的示例行放进代码块不被当成声明那一行，也不算重复；认围栏与 ⑮ 同一个函数、同一种认法」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/hooks/runner-dispatch-guard.sh:64`） | 一致 | 喂法 B：「fence-hosts-example-block-then-canon」放行、「canon-inside-fence-only」拒（块里的规范句不算声明）、「wallclock-only-in-fence」拒、「unclosed-fence-hosts-example」拒。推翻：块里的示例行被当成声明或重复。 |
| Z1-14 | 「（线程上限那张类型表，九种：实现员、调查员、工具员、执行员、变异分诊员、攻方腿、clerk、前向腿、防方腿）」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/scripts/agents/dispatch-prompt.py:20`） | 一致 | dispatch-prompt.py 的 HEAVY_WORK_TYPES 九个类型（该文件 83-84 行）与钩子 THREAD_LIMIT_REQUIRED_AGENTS 九个类型逐个相同（钩子 1052-1053 行）；喂法 A 九类缺行都拒。推翻：两张表差一个类型。 |
| Z1-15 | 「要说明的另起一行照普通正文写，那一行不判；三个反引号围起来的代码块里的行派发闸 ⑮ 同样不判」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/scripts/agents/dispatch-prompt.py:25`） | 一致 | 骨架 --check 经派发闸同一份判法（喂法 A）。推翻：同 Z1-09。 |
| Z1-16 | 「预计挂钟表少书记员、前向腿、防方腿」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/scripts/agents/dispatch-prompt.py:43`） | 不一致（clerk 改名没改全） | 同一处弄坏开关 types-before-nine 实际去掉的是 clerk、前向腿、防方腿（dispatch-prompt.py:90-91 的 LONG_RUN_TYPES 去掉 clerk）；全仓别处「书记员」指 kb-scribe（main-agent.md:73、钩子 ⑧ 文件头）。这一句里的「书记员」应是 clerk。推翻：把「书记员」换成 clerk 后与代码一致，说明只是名字没改全。 |

## Z3　重型测试闸仓副本那一支

喂法 C 的七例（stdin JSON，`agent_type` 区分子 agent 与主 agent；`SINGLEFS_WORK_ROOT` 指到草稿目录里的工作根；副本里的脚本是登记成 light 的 `scripts/gate/apply-writer-patch.py`，改过的一份多了一行注释）结果：没改脚本的副本——子 agent 退 0；改过正文——子 agent 退 2、主 agent 不带前缀退 2、带 `SINGLEFS_HEAVY_TESTS=commit` 或 `=user-request` 退 0；主仓里同一份——退 0。

| 编号 | 引文（F 里的位置） | 判定 | 依据；什么现象会推翻 |
|---|---|---|---|
| Z3-01 | 「改过脚本的自证不在副本里跑，写进交回报告由主 agent 在集成副本里跑（什么时候跑、带什么前缀照 .claude/rules/implementation-workflow.md…那一节，拒绝说明指到那一节）」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/hooks/heavy-test-guard.sh:89`） | 一致 | 喂法 C 第二例（子 agent 在改过正文的副本里跑自证）退出码 2，stderr 末尾写「什么时候跑、带什么前缀照 .claude/rules/implementation-workflow.md「checker 档只在提交时跑，harness 随时跑」那一节的做法」，第三例（主 agent 不带前缀）同样。推翻：同一喂法 stderr 里没有指到那一节。 |
| Z3-02 | 「什么时候跑、带什么前缀照 .claude/rules/implementation-workflow.md…那一节的做法。」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/hooks/heavy-test-guard.sh:837`） | 一致 | 同 Z3-01，这一句是拒绝说明常量的收尾（喂法 C 第二例 stderr 最后一句）。指到的小节标题在 implementation-workflow.md:45，逐字相同。推翻：标题改名后这一句不跟。 |
| Z3-03 | 「什么时候跑、带什么前缀照 .claude/rules/implementation-workflow.md…那一节的做法，不在这里另写」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/hooks/lib_heavy_tests.py:1447`） | 一致 | 判法函数 selftest_work_copy_problem 的这一支只写指路，不写前缀（lib_heavy_tests.py 的 howto 字符串），喂法 C 第二例 stderr 里「内容不同……闸不读副本里的脚本正文」这一段即它。推翻：这一支的说明里出现 SINGLEFS_HEAVY_TESTS=。 |
| Z3-04 | 「仓副本：脚本正文被改过的拒绝说明指到 implementation-workflow.md…那一节的做法、不直接写 user-request 前缀…一格红（应当是 True，实际 False）」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/hooks/lib_heavy_tests.py:387`） | 一致（只就这一支的说明 reason 而言）；对整条拒绝输出不一致 | 自检只核 reason 这一句不含 user-request。喂法 C：子 agent 的整条 stderr 在这一句之后还有 COMMIT_CHECK_POLICY 常量，写着「主 agent 带 SINGLEFS_HEAVY_TESTS=commit（用户要求时 =user-request）跑」（/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/hooks/heavy-test-guard.sh:820）；主 agent 不带前缀的整条 stderr 以「→ 照抄……SINGLEFS_HEAVY_TESTS=commit python3 <路径> --selftest」起头。所以「拒绝说明不写带前缀跑」只在这一支成立，整条输出两种前缀都写了。推翻：喂法 C 第二、三例 stderr 里搜不到 SINGLEFS_HEAVY_TESTS=。 |
| Z3-05 | 「两档各是什么、谁跑、带什么前缀，在…；这里只写实现改动流程里的落点：」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/rules/implementation-workflow.md:47`） | 文字没写到 | 拒绝说明把读者指到这一节，这一节没写「改过正文的自证在集成副本里怎么跑」：节内没有「仓副本」「集成副本」，「自证」二字零次，只在讲重型清单时写了「登记为 light 的单独 --selftest 不算」（该文件第 56 行）；「提交时才跑的检查」这个词在 .claude/rules、main-agent.md、agent-common.md 里零命中，只在 .claude/skills/gate/staged-gate.md:16 与钩子的拒绝常量里。按这一节读，登记成 light 的 --selftest 不算重型、随时可跑；实际行为（喂法 C）相反：主 agent 在集成副本里不带前缀拒（退出码 2），带 =commit 或 =user-request 放行（退出码 0）。读者照指路只能推出「重型才要前缀」，推不出这一条自证在副本里被当成提交时才跑的检查。推翻：这一节补了改过正文的自证与前缀，或喂法 C 的主 agent 不带前缀放行。 |

## Z4　叫停登记行的出处：〔〕原话恰好找到一次

### 专问答复

- 原话命中「恰好一次」在记录被别的会话追加、改写之后稳不稳（喂法 D，全部是库的命令行输出，派发闸 ⑥ 与门禁 abandoned-rounds 格共用这一份）：
  - 仍收：在出处文件前面插两行（报「原话现在在第 4 行」）；登记行写错行号而原话对；别的会话改了那一行里原话以外的字；记录文件是回车换行；后面追加的一行只提到更长的轮名 `zz-ab-r10`（原话 `zz-ab-r1 叫停` 不是它的子串）。
  - 变成不收：后面追加了逐字重复的一行（命中第 2、4 行）；追加的一行里带了原话那几个字（同上）；那一行里的「叫停」被改成「喊停」（找不到）；那一行被折成两行（找不到）；同一行里原话出现两次（命中第 2、2 行）。
  - 无路可走：整行被逐字重复后，原话抄成整行仍命中两次；原话不能含换行，规则里的「命中多次就多抄几个字」在这种情形救不了（Z4-05）。
  - 所以「恰好一次」比行号稳（插行不怕），但对追加敏感：只要后来有人在同一份记录里原文再引一遍这句叫停，登记就失效，失效后门禁与派发闸一起红。
- 旧行名单的四个名字里三个已无行，名单与表的关系：冻结副本的登记表里名单中的名字只剩 `smalldata-ttime-r1`（喂法 G：`real-abandoned-table.txt` 两行都收，另一行 `m3-batch63-code-r1` 带出处）；`c196-r1`、`d16-item1-r2`、`c364-r3` 三个名字库里仍写死，今天新登记这三个名字之一、不写出处，派发闸与门禁照收（喂法 D「旧行名单里的名字可以再登记且没有出处」收）。库里写「只缩不涨」只是注释，没有检查它的格（Z4-12）。
- 最脆弱的一句：Z4-10、Z4-11 两处没跟着改的旧句子（见下），照它们写的人会以为 `-r<数字>` 形不算别的轮名。

### 句表

| 编号 | 引文（F 里的位置） | 判定 | 依据；什么现象会推翻 |
|---|---|---|---|
| Z4-01 | 「登记行里写出处，写成 records/<文件>.md 第 N 行〔<原话>〕（或 records/<文件>.md:N〔<原话>〕）」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/rules/three-way-inference.md:121`） | 一致；文字没写到「行号不能省」 | 喂法 D：「基线」「冒号写法」收；「没有行号只有原话」不收（报「没有出处」）。「行号只当提示」读起来像可省，实际正则必须有「第 N 行」或「:N」。推翻：没有行号的写法被收。 |
| Z4-02 | 「〔〕里逐字照抄那一行里含这一轮轮名的一段原话（不含〔〕、制表符与换行）」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/rules/three-way-inference.md:121`） | 不一致（文字比钩子严） | 代码只要求原话在那份文件里恰好出现一次、命中的那一行含轮名（lib_abandoned_rounds.py 的 source_problem），不要求原话本身含轮名。喂法 D：「原话不含轮名而那一行含」收、「原话只有一小段不含轮名」收。agent-common.md:85、main-agent.md:22 写的是同义句，同判。推翻：不含轮名的原话被拒。 |
| Z4-03 | 「路径里不带 ..，解到真路径后仍在 records/ 之内，文件存在」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/rules/three-way-inference.md:121`） | 一致 | 沿用旧判法；喂法 D：「出处指到 research 下」不收。..、符号链接、仓外路径由派发闸自检的既有格逐个钉（runner-dispatch-guard.sh 的 快照:出处路径里有 .. 一类格），本轮没改。推翻：出处写到 research/ 或 .claude/ 被收。 |
| Z4-04 | 「拿〔〕里的原话在那份文件里找，恰好命中一次才收，命中的那一行就是出处那一行，行号只当提示（对不上而原话找得到照收）」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/rules/three-way-inference.md:121`） | 一致 | 喂法 D：「行号写错原话对」「行号写成第 0 行」收并报原话现在在第几行；「别的会话在前面插了两行」收（报挪到第 4 行）；「同一行里原话命中两次」不收（命中第 2、2 行，同一行里两处也算两次）。推翻：行号对不上而原话唯一时不收，或命中两次时收。 |
| Z4-05 | 「没带原话、原话找不到（被删或改了字）、命中两次及以上都不收，命中多次就多抄几个字」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/rules/three-way-inference.md:121`） | 一致（不收）；文字没写到「多抄几个字」有时无路可走 | 喂法 D：「有行号没带原话」不收、「别的会话把叫停改成喊停」不收、「别的会话逐字重复了整行而原话已抄整行」不收且仍命中第 2、4 行：原话不能含换行，整行重复时多抄几个字无济于事。推翻：整行重复后换一段原话能让命中唯一（原话不跨行，办不到）。 |
| Z4-06 | 「出处那一行里要有这一轮的轮名（前后不连着别的轮名字符）与」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/rules/three-way-inference.md:121`） | 一致 | 喂法 D：「那一行有本轮名加一位数字」不收（zz-ab-r10 算别的轮名，本轮名前后不连着轮名字符的要求在代码里是 (?<![A-Za-z0-9_\-]) 与 (?![A-Za-z0-9_\-])）。推翻：本轮名连着数字也被当作本轮名。 |
| Z4-07 | 「括起来的用户原话，原话去掉空白后至少 6 个字，除这一轮的轮名外不许有别的轮名」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/rules/three-way-inference.md:121`） | 一致 | 喂法 D：「原话不到六个字」不收。推翻：不到 6 个字的原话被收。 |
| Z4-08 | 「别的轮名 = 仓里认得的轮名全集 ∪ 形如 <名>-r<数字> 的写法，两样都不许出现在那一行」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/rules/three-way-inference.md:121`）；同句另见 `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/agent-common.md:85` | 一致（小写字母、数字、连字符的名字）；文字没写到字符集 | 喂法 D：「那一行有小写的别的轮名」不收、「那一行有仓里以 -body.md 认出的非 r 形名」不收。但「形如 <名>-r<数字>」在代码里是 [a-z0-9][a-z0-9-]*-r[0-9]+（lib_abandoned_rounds.py 的 ROUND_NAME_FORM）：「那一行有大写的别的轮名」收、「那一行有带下划线的别的轮名」收、「那一行有带点的别的轮名」不收但只认出点后一段 bar-r2。仓里认得的 306 个轮名只有一个不以 -r<数字> 结尾（test-granularity-rename），都是小写字母、数字、连字符（喂法 G：real-round-names.txt）。推翻：大写或带下划线的 <名>-r<数字> 被当作别的轮名。 |
| Z4-09 | 「全集是 research/prompts/ 下 _<轮>-body.md、<轮>-snapshot/、判决文件与登记表里的轮名」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/rules/three-way-inference.md:121`）；同句另见 `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/agent-common.md:85` | 不一致（「判决文件」说宽了） | 代码只认 <轮>-main-verification.md 一种判决文件名（lib_abandoned_rounds.py 的 ROUND_MATERIAL_FORMS）。喂法 D：「那一行有仓里以 -main-verification.md 认出的非 r 形名」不收、「那一行有仓里只有 -verdict.md 的非 r 形名」收、「那一行有仓里只有腿产出的非 r 形名」收。推翻：别的判决文件名也被认。 |
| Z4-10 | 「除这一轮外没有别的轮名（按仓里认得的轮名全集判：research/prompts/ 下 _<轮>-body.md、<轮>-snapshot/、<轮>-main-verification.md 的轮名加登记表第一列，不按 -r<数字> 形」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/hooks/runner-dispatch-guard.sh:39`） | 不一致（旧文字没跟着改） | 同一份文件头往下的本轮新句和 lib_abandoned_rounds.py 都写「全集 ∪ 形如 <名>-r<数字> 的写法」，这一句仍写「不按 -r<数字> 形」。喂法 D：「那一行有小写的别的轮名」只靠 -r 形（仓里没有它的材料）就不收。推翻：-r 形不认时「那一行有小写的别的轮名」仍不收。 |
| Z4-11 | 「按仓里认得的轮名全集判（research/prompts/ 下 _<轮>-body.md、<轮>-snapshot/、<轮>-main-verification.md 的轮名，加登记表第一列的轮名），不按 -r<数字> 形。」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/gate.d/doc-process-records.sh:103`） | 不一致（旧文字没跟着改） | 同 Z4-10；这是门禁 abandoned-rounds 格文件头的 ④ 段，上两行（出处写法）已改，这一行没改。推翻：同 Z4-10。 |
| Z4-12 | 「免出处只给库里写死的四行旧登记（c196-r1、d16-item1-r2、c364-r3、smalldata-ttime-r1），不按日期判，名单外的行一律要出处，新叫停的轮都写出处。」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/rules/three-way-inference.md:121`） | 一致（名单外要出处）；文字没写到（名单里已无行的三个名字今天还能免出处） | （本句没改，为 Z4 专问收进来。）喂法 D：「名单外的名字没有出处」不收。冻结副本的登记表只剩 smalldata-ttime-r1 一个名单里的名字（喂法 G：real-abandoned-table.txt 两行都收）；c196-r1、d16-item1-r2、c364-r3 的行已删，但库里 LEGACY_SOURCELESS_ROUNDS 仍是四个名字，「旧行名单里的名字可以再登记且没有出处」收。「只缩不涨」只是库里的注释，不是检查。推翻：登记 c196-r1 且不写出处被拒。 |
| Z4-13 | 「恰好命中一次才收，行号只当提示：对不上而原话找得到照收、成功行后面逐个报挪到第几行；没带原话、找不到、命中两次及以上都红；」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/gate.d/doc-process-records.sh:101`） | 一致 | 门禁 abandoned-rounds 格与派发闸调同一个库；成功行报挪动的那段在 doc-process-records.sh:918-921（moved_rows），库的命令行入口同样报「（出处……写的第 N 行，原话现在在第 M 行）」（喂法 D 「行号写错原话对」）。门禁格本身子 agent 起不了：要主 agent 跑，命令在模型目录 main-agent-to-run.md。推翻：门禁格对同一张表给出与库的命令行不同的收不收。 |
| Z4-14 | 「〔〕里的原话在那份文件里恰好找到一次（行号只当提示），找到的那一行里有这一轮的名字和…括起来的用户原话（去掉空白后至少 6 个字）；」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/research/prompts/abandoned-rounds.tsv:5`） | 一致 | 喂法 G：冻结副本里真表两行（smalldata-ttime-r1 免出处、m3-batch63-code-r1 带 records/2026-10-01-删除与并发事务.md 第 1595 行〔……〕）都收。推翻：真表里有一行不收。 |
| Z4-15 | 「拿原话在那份文件里找，恰好命中一次（行号只当提示，对不上而原话找得到照收；没带原话、找不到、命中两次及以上都不收），命中的那一行里有这一轮的名字（前后不连着轮名字符）」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/hooks/runner-dispatch-guard.sh:38`） | 一致 | 同 Z4-04、Z4-05。推翻：同。 |
| Z4-16 | 「出处写 records/<文件>.md 第 N 行〔<原话>〕（或 records/<文件>.md:N〔<原话>〕，〔〕里逐字照抄那一行里含轮名的一段，在那份文件里要恰好找到一次，行号只当提示），找到的那一行要有轮名与 6 个字以上的」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/main-agent.md:22`） | 一致，同 Z4-01 至 Z4-05、Z4-07 | main-agent.md 对主 agent 的写法说明，判法同上；含轮名一段的要求同 Z4-02（钩子不查）。 |
| Z4-17 | 「出处要写 records/<文件>.md 第 N 行〔<原话>〕（或 :N〔<原话>〕，〔〕里逐字照抄那一行里含轮名的一段，在那份文件里要恰好找到一次，行号只当提示），找到的那一行有这一轮的轮名与 6 个字以上的」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/agent-common.md:85`） | 一致，同 Z4-16 | 同上。 |

## Z6　多机一行：逐字等于规范句，后面不接说明

### 专问答复

- 说明另起一行之后，现算的说法挪到下一行照样送到子 agent 手里：成立，实测放行。派发提示：规范句一行加下一行写「每台线程按此刻空闲现算，机器忙时按负载降」，⑭ 放行（喂法 B 的 `canon-then-next-line-live`、`canon-then-next-line-label-live`）；说明夹在括号里写在别的行中间也放行（`canon-then-midline-label`）；脚本文件头：规范句一行加下一行不带键的注释写现算，`long-run-check` 退 0（喂法 E 的 `canon-then-plain-comment-live`）。新规则对说明行的内容一个字不判（旧规则「不含数字」随这一批删了，Z6-12）；定义只写「说明另起一行写」，没限说明行的内容。
- 重复标签：派发提示里「分片：」「续跑：」「多机：」超过 2 小时时同一标签第二行就拒（Z1-12），脚本文件头里同一个键写几行是接起来、分片键写两行不拒（Z6-07）；这两条只在钩子文件头与库的文件头里有，定义与共用约束没写（Z1-07）。
- 最脆弱的一句：Z6-12 的「说明另起一行」没限内容，换个自然写法（下一行写「实际按此刻空闲算」）就绕过「不现算」。

### 句表

| 编号 | 引文（F 里的位置） | 判定 | 依据；什么现象会推翻 |
|---|---|---|---|
| Z6-01 | 「其中 # long-run-hosts: 逐字等于骨架那一句（python3 scripts/agents/dispatch-prompt.py <类型> 打的…一行里的规范句，原文是 .claude/hooks/lib_long_run.py 的 CANONICAL_HOSTS_SENTENCE，可带句末句号，后面不接任何说明，说明另起一行写）」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/agent-common.md:67`） | 一致 | 脚本文件头一侧（喂法 E，经 detached-run.sh long-run-check）：规范句放行、带全角或半角句末句号放行（退 0）；规范句后接「；」加说明拒（退 5）。派发提示一侧同理（喂法 B）。推翻：规范句加说明被放行，或带句号被拒。 |
| Z6-02 | 「一行逐字等于骨架（scripts/agents/dispatch-prompt.py）打的那一句规范写法，即 .claude/hooks/lib_long_run.py 的 CANONICAL_HOSTS_SENTENCE，可带句末句号，后面不接任何说明，说明另起一行写，别的写法拒）」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/agent-common.md:85`） | 一致 | 喂法 B：「canon」「canon-period-zh」「canon-period-en」「canon-period-trailing-space」放行；「canon-double-period」「canon-semicolon-note」「canon-comma-note」「canon-in-backticks」「canon-in-quote-marks」「canon-with-trailing-markdown-bold」拒。推翻：这几例换结果。 |
| Z6-03 | 「行与脚本文件头 # long-run-hosts: 只认一句规范写法，都逐字等于骨架那一句（python3 scripts/agents/dispatch-prompt.py <类型> 打的…一行里的规范句，原文是 .claude/hooks/lib_long_run.py 的 CANONICAL_HOSTS_SENTENCE，可带句末句号，后面不接任何说明，说明另起一行写）」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/main-agent.md:55`） | 一致 | 同 Z6-01、Z6-02：派发提示与脚本文件头两处都是同一个 canonical_hosts_problem 判（lib_long_run.py 的 judge，hosts_reading=registered）。推翻：两处判法分叉。 |
| Z6-04 | 「照骨架那一句逐字写（python3 scripts/agents/dispatch-prompt.py <类型> 打的…一行里的规范句，原文是 .claude/hooks/lib_long_run.py 的 CANONICAL_HOSTS_SENTENCE，可带句末句号，后面不接任何说明，说明另起一行写）」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/agents/experiment-designer.md:34`）；同句另见 `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/agents/experiment-runner.md:33` | 一致 | 设计员写进跑前登记、执行员写进脚本文件头，都落到同一个判法（喂法 E）。推翻：同 Z6-01。 |
| Z6-05 | 「登记值读法下 hosts 一行是不是规范写法：逐字等于 CANONICAL_HOSTS_SENTENCE，可带句末句号（HOSTS_SENTENCE_ENDINGS），后面不许接任何说明。」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/hooks/lib_long_run.py:127`） | 一致 | 判法函数 canonical_hosts_problem 的 docstring；喂法 B、E 全部以它为准。句末只认空串、全角句号、半角句点（lib_long_run.py 的 HOSTS_SENTENCE_ENDINGS），规整只做 strip（含不换行空格，喂法 B「canon-nbsp-trailing」放行）。推翻：句末再多一个字符被放行。 |
| Z6-06 | 「要说明的另起一行写：脚本文件头里写成不带 long-run-hosts: 的普通注释行（同一个键写第二行会按次序接在后面，接出来就不再逐字等于规范句），」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/hooks/lib_long_run.py:20`） | 一致 | 喂法 E：「canon-then-second-hosts-key」「canon-then-second-hosts-key-same」「split-canon-over-two-keys」拒（退 5）；「canon-then-plain-comment-live」「canon-then-blank-then-comment」放行（退 0）。拒绝说明里把第二行也叫「接了说明」，第二行本身是规范句时说法有点怪，判对。推翻：第二个 long-run-hosts: 键被放行。 |
| Z6-07 | 「（REQUIRED_KEYS 那三样）哪一个写了两行及以上：prompt_declaration 只读第一行，第二行写什么都没人判，所以同一标签出现两次判红。」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/hooks/lib_long_run.py:369`） | 一致（派发提示）；文字没写到（脚本文件头的重复键走另一条路，分片、续跑键写两行是接起来、不拒） | 派发提示一侧见 Z1-12。脚本文件头一侧：同一个键写几行是按次序用空格接起来（lib_long_run.py 的 script_header_declaration），多机键接出来不是规范句才拒；分片键写两行放行（喂法 E「canon-dup-shards-key」退 0）。体现在钩子文件头、lib 文件头，定义与共用约束都没写重复键。推翻：文件头分片键写两行被拒。 |
| Z6-08 | 「逐字（可带句末句号），后面不接任何说明（说明另起一行写），别的写法一律拒，判在派发闸 ⑭；骨架的占位引那一句，不另抄一份）。」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/scripts/agents/dispatch-prompt.py:21`） | 一致 | 骨架 python3 scripts/agents/dispatch-prompt.py clerk 打的多机行引规范句（dispatch-prompt.py 的 LONG_RUN_LINES）。推翻：骨架多机行与 lib 的规范句不同。 |
| Z6-09 | 「这一行逐字等于它（可带句末句号），后面不许接任何说明（…都拒），说明另起一行照普通正文写，」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/hooks/runner-dispatch-guard.sh:60`） | 一致 | 喂法 B：「canon-semicolon-note」「canon-comma-note」拒。推翻：规范句后接「，」加说明被放行。 |
| Z6-10 | 「（多机这一行只认这一句，逐字，可带句末句号；后面不接任何说明，要说明另起一行写，别的说法一律不认）；」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/hooks/runner-dispatch-guard.sh:1264`） | 一致 | 拒绝说明里的写法提示（long_run_lines，派发闸拒绝输出里的「→ 怎么办」）与判法相符。推翻：按这句改写后仍被拒。 |
| Z6-11 | 「逐字（可带句末句号），后面不许接任何说明，说明另起一行写普通注释，别的写法一律拒，不再用现算词表；规范句只写在那一处，这里不抄。」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/scripts/hosts/detached-run.sh:55`） | 一致 | detached-run.sh 的 start（over-2h，591 行）与 long-run-check（1967 行）都调同一个 detached_run_long_run_check（547 行），它起 lib_long_run.py 的 script 子命令，该子命令经 judge_script_header 判（lib_long_run.py:400）；喂法 E 即 long-run-check 的结果。host-dispatch.py 同一个库同一个函数（host-dispatch.py:573、577），见 Z6-13。推翻：start 对规范句加说明放行。 |
| Z6-12 | 「要说明的另起一行写：脚本文件头里写成不带 long-run-hosts: 的普通注释行」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/hooks/lib_long_run.py:20`） | 一致；专问：说明另起一行之后，现算的说法挪到下一行照样送到子 agent 手里——成立，且文字没写到说明行里能写什么 | 喂法 B：「canon-then-next-line-live」「canon-then-next-line-label-live」「canon-then-midline-label」放行（下一行写「每台线程按此刻空闲现算，机器忙时按负载降」照放）；喂法 E：「canon-then-plain-comment-live」放行。旧规则「不含数字的说明」随这一批删了，新规则对说明行内容一个字不判。定义只说「说明另起一行写」。推翻：下一行的现算说法被拒。 |
| Z6-13 | 「逐字（可带句末句号），后面不许接任何说明（说明另起一行写普通注释），别的写法一律拒（规范句只写在那份库里，这里不抄）；」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/scripts/hosts/host-dispatch.py:23`） | 一致 | host-dispatch.py 经 judge_script_header 判文件头（host-dispatch.py:573、577），与 detached-run.sh 同一个库函数；喂法 E 的判法即它。推翻：派发器对规范句加说明放行。 |

## Z7　长活声明豁免表只缩不涨按（路径、sha256）成对比

### 专问答复

- （路径、sha256）成对比在基准那一版没有这张表时做什么：不比。门禁格先看 git 仓与 HEAD 在不在，在就取基准、问基准里有没有这张表，列不到就不给 `--base-table`（`code-tooling.sh` 的 `base_arguments` 一段，2439-2455 行）；库的 `exempt-check` 不带基准时退 0，成功句写「没给基准（基准那一版里还没有这张表，或不在 git 仓里），只缩不涨这一条没比」（喂法 F 的 `不给基准`）。所以整张表在没有基准的那一刻新建，所有行放行，包括新路径与新 sha256。基准文件存在但是空文件或只有注释，则每一行都报涨（喂法 F 的 `基准是空文件`、`基准只有注释`）：「没有表」和「有表但空」两种结果相反。
- 基准是不带 sha256 的旧写法：只比路径，重钉放行（喂法 F 的 `基准是不带 sha256 的旧写法而现在重钉`）。冻结副本里真表六行都带 sha256，当前的基准下用不上这条放行，但定义写的是无条件的「同一路径 sha256 变了也算涨」。
- 基准是什么：`gate_diff_base gate`，GATE_BASE 是提交就用它，否则上游的 merge-base，都没有就 HEAD（Z7-11）。重钉进了基准之后再跑不红，这一条是读代码推的、门禁格没跑，命令在 `main-agent-to-run.md` 第 3 条。
- 最脆弱的一句：Z7-01「同一路径 sha256 变了也算涨，判红」——「判红」成立的窗口是「基准还没吃进这次重钉」，文字没写。

### 句表

| 编号 | 引文（F 里的位置） | 判定 | 依据；什么现象会推翻 |
|---|---|---|---|
| Z7-01 | 「豁免表只缩不涨，按（路径、sha256）成对比基准那一版，同一路径 sha256 变了也算涨，判红」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/agent-common.md:67`）；同句另见 `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/main-agent.md:55` | 一致；文字没写到三处（基准是什么、基准旧写法、没有基准） | 喂法 F：「改了脚本再重钉」拒（退 1，点名「豁免表重钉了」）、「基准里没有这个路径」拒。文字没写到：①基准旧写法不带 sha256 的行只比路径，重钉放行（「基准是不带 sha256 的旧写法而现在重钉」退 0）；②基准那一版没有这张表、或不在 git 仓里：不比，成功句里写「没比」（「不给基准」退 0），整张表新建全部放行；③「基准」是 gate_diff_base gate：GATE_BASE，否则 @{upstream} 的 merge-base，否则 HEAD（changed-paths.sh 的 gate_diff_base，行号见 Z7-11），重钉进了基准之后再跑就不红——这一条靠读代码，门禁格要主 agent 跑。推翻：基准是旧写法时重钉被拒，或没有基准时新行被拒。 |
| Z7-02 | 「基准那一版里同一路径的 sha256 变了的行（改了脚本正文再把表里的 sha256 重钉，也算涨），都由 exempt-check 判红（只缩不涨按…成对比）。」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/hooks/lib_long_run.py:31`） | 一致 | 判法函数 exempt_table_problems：对基准表逐行取（路径，sha256 集合），当前行路径不在基准里报涨、在而 sha256 不在集合里报重钉（lib_long_run.py:311-322）。喂法 F：「基准里同一路径两行不同 sha256 而现在是其中一个」放行、「……而现在是第三个」拒。推翻：重钉放行。 |
| Z7-03 | 「给了 --base-table 的再比只缩不涨（按路径 + sha256 成对比：基准那一版里没有的路径、同一路径的 sha256 换了的都判红；」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/hooks/lib_long_run.py:39`） | 一致 | 喂法 F：「基准是空文件」「基准只有注释」都报每一行涨（空基准不等于没基准）。推翻：空基准放行。 |
| Z7-04 | 「只缩不涨：按（路径，sha256）成对跟基准那一版比，基准里没有这一对的行就算涨——新路径算涨，同一路径的 sha256 变了（改了脚本再换表里的 sha256 重钉）也算涨；」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/long-run-live-hosts-exempt:7`） | 一致 | 同 Z7-01、Z7-02。推翻：同。 |
| Z7-05 | 「基准那一版里该路径还是不带 sha256 的旧写法的，只比路径。改成登记值读法或脚本不再需要就删掉那一行，新脚本不往这里加。」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/long-run-live-hosts-exempt:8`） | 一致 | 喂法 F：「基准是不带 sha256 的旧写法而现在重钉」退 0、「基准是不带 sha256 的旧写法而路径没变」退 0。冻结副本里的真表六行都带 sha256，真表对得上（喂法 G：real-exempt-table.txt）。推翻：旧写法基准下重钉被拒。 |
| Z7-06 | 「E171 下一次修订的入口不进这张表，文件头按规范句写。」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/long-run-live-hosts-exempt:5`） | 一致（靠「新路径算涨」兜住）；文字没写到：只在有基准时才拦 | 表里没有「E171 入口」这类特判，靠基准比较拦新路径（喂法 F：「基准里没有这个路径」拒）；基准没有这张表或不在 git 仓里时不比，新行放行（「不给基准」退 0）。推翻：无基准时新路径被拒。 |
| Z7-07 | 「成对比，基准里没有的路径、基准里同一路径的 sha256 换了的行（改了脚本正文再重钉，也算涨）都判红；」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/gate.d/code-tooling.sh:2418`） | 一致 | 格函数 cell_long_run_header_exempt 取基准的做法：在 git 仓里且 HEAD 在，用 gate_diff_base gate 取基准，基准里列得到这张表就 git show 成临时文件传 --base-table，列不到就不传（code-tooling.sh:2439-2455 的 base_arguments 一段）；然后调库的 exempt-check。同一份判法，喂法 F 即库这一层。门禁格本身子 agent 起不了，造小树的命令在 main-agent-to-run.md，标「要主 agent 跑」。推翻：格与库给出不同结论。 |
| Z7-08 | 「基准那一版还是不带 sha256 的旧写法的行只比路径；基准里还没有这张表、不在 git 仓里，这一条不比、成功句里写明。」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/gate.d/code-tooling.sh:2419`） | 一致 | 专问「（路径、sha256）成对比在基准那一版没有这张表时做什么」：不比，exempt-check 退 0，成功句写「没给基准（基准那一版里还没有这张表，或不在 git 仓里），只缩不涨这一条没比」（lib_long_run.py exempt_main，喂法 F「不给基准」；格里 base_arguments 为空）。与「基准是空文件」不同：后者逐行报涨。推翻：没有基准时报红或报涨。 |
| Z7-09 | 「红了删行；改被钉的脚本要重钉 sha256，按只缩不涨判红，交用户」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/gate.d/stage-owners.tsv:48`） | 一致 | 喂法 F：脚本改了、表没动 →「表里 sha256 与脚本不符而基准相同」拒，怎么办写「把改动撤回去；确要改它……删掉这一行」；脚本改了、表重钉 →「改了脚本再重钉」拒，怎么办写「确要保留改动并重钉的，交主 agent 问用户」。三者相接。推翻：重钉后绿。 |
| Z7-10 | 「与基准那一版比没涨…没给基准（基准那一版里还没有这张表，或不在 git 仓里），只缩不涨这一条没比」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/.claude/hooks/lib_long_run.py:441`） | 一致 | 成功句的两种写法即 Z7-08 的两条路；喂法 F「基准与现在逐字相同」打前者，「不给基准」打后者。推翻：两种写法对调。 |
| Z7-11 | 「gate：GATE_BASE 是个提交就用它；否则 @{upstream} 的 merge-base；都没有（或 merge-base 取不到）就 HEAD。」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo/scripts/gate/changed-paths.sh:10`） | 一致（本句在代码里；定义、共用约束没写基准是什么） | Z7-01 的第③处：基准随上游推进，重钉进了基准之后同一张表再跑不红。取法在 changed-paths.sh:67-91 的 gate_diff_base。要看门禁格的实际基准要主 agent 跑。推翻：门禁格不用这一取法取基准。 |

## 判定一览

句数是表里的行数（九份定义里逐字相同的句子合成一行）。

| 格 | 判定 | 一句话 |
|---|---|---|
| Z1 | 16 句：11 句一致；3 句一致但有文字没写到（Z1-03、Z1-04、Z1-07）；2 句不一致（Z1-10、Z1-16） | 线程上限只照一个整数、已判行、说明另起一行、闭合的三反引号块都与钩子相符；「认围栏照 CommonMark」说过头（列表项、引用块里的围栏、波浪线围栏钩子都不认，Z1-10）；dispatch-prompt.py:43 的「书记员」是 clerk 没改全（Z1-16）；围栏内不判、同一标签第二行拒这两条 ⑭ 的规矩在定义与共用约束里没写（Z1-07） |
| Z3 | 5 句：3 句一致；1 句一致但对整条拒绝输出不一致（Z3-04）；1 句文字没写到（Z3-05） | 仓副本那一支的说明确实只指路、不写前缀，但整条 stderr 还有常量写着 commit 与 user-request 前缀；指到的小节没写改过正文的自证怎么跑，且按它读登记成 light 的 --selftest 不算重型，与钩子相反（主 agent 不带前缀退 2） |
| Z4 | 17 句：9 句一致；4 句一致但有缺口（Z4-01、Z4-05、Z4-08、Z4-12）；4 句不一致（Z4-02、Z4-09、Z4-10、Z4-11） | 〔〕原话恰好一次、行号只当提示、别的轮名全集 ∪ -r 形的判法与文字相符；不稳在追加（整行重复后无路可走）；文字要求原话含轮名而钩子不查；「判决文件」说宽了；钩子文件头与门禁文件头各留一句「不按 -r<数字> 形」没改；旧行名单三个名字已无行但仍能免出处登记 |
| Z6 | 13 句：11 句一致；2 句一致但有文字没写到（Z6-07、Z6-12） | 规范句加可选句末句号、后面不接说明，派发提示与脚本文件头同判；说明另起一行之后现算的说法放到下一行照放（实测），新规则对说明行内容一个字不判；脚本文件头重复键与提示重复标签是两条路，定义没写 |
| Z7 | 11 句：9 句一致；2 句一致但有文字没写到（Z7-01、Z7-06） | （路径、sha256）成对比与重钉判红相符；基准是旧写法只比路径、没有基准不比（整张表新建全放行）、基准随上游推进这三处文字没写到；空基准文件与没有基准结果相反 |

## 没做什么

- 没判 Z2、Z5（不是我的格）；没替主 agent 采纳或出判决；没跑攻方立场的「造工作流历史」。
- 门禁格（`code-tooling.sh --check long-run-header-exempt`、`doc-process-records.sh --check abandoned-rounds`）子 agent 起不了，没跑；它们调的库我在库这一层喂过。基准随上游推进（Z7-01 第三处、Z7-11）是读代码推的，没喂。
- 叫停登记行的派发闸 ⑥ 没做端到端喂（要造开着的三方轮快照）；⑥ 与门禁共用的判法库我直接喂了命令行，没喂钩子入口。
- 重型测试不跑；没跑任何登记的 `--selftest`；没编译 Rust。
- 对子 agent 行为的句子（Z1-01、Z1-02、Z1-05、Z1-06）没有第二处源码可核，只写了「钩子放行」与「别处没有核它的脚本」。
- 正文「三」要每格答的三件（过度外推与越位、内部矛盾与循环依赖、最脆弱）我没有逐格单列：最脆弱的一句写在各格的专问答复里；内部矛盾只在 Z4-10、Z4-11（同一份文件头里新旧两句相反）、Z3-05（指路与钩子相反）、Z1-16（同一类型两个名字）出现；过度外推与越位在我的立场（句子对源码）里没有单独的判法。

## 要主 agent 跑的命令

见 `research/prompts/m3-defs-b9-r1-sonnet-model/main-agent-to-run.md`：三条，门禁 code-tooling 与 doc-process-records 的样本档各一条，加 Z7-01 第三处的小树验证。

## 复跑与交回核对

- 模型目录 `research/prompts/m3-defs-b9-r1-sonnet-model/`：`tools/rerun.sh`（从 `scripts/agents/three-way-rerun-template.sh` 拷来，只换 `rerun_steps`）、`tools/` 下六个喂法脚本、`out/` 下九份原输出、`main-agent-to-run.md`。
- 复跑命令（经内存包装，上限 2G；仓根、冻结副本、草稿目录都从参数传）：`bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-defs-b9-r1-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m3-defs-b9-r1/repo --work <新的草稿目录> --model /home/fy5090/code/singlefs/research/prompts/m3-defs-b9-r1-sonnet-model`。
- 我最后一次跑（草稿目录 `run4`）：九份输出都报「与模型目录 out/ 里同名文件逐行相同」；`out/` 的 sha256：

```
0e62d9ae128a6ba5b6c4a8b642ea72fd23f7bbef73cc2a6435fdb058a6d9b418  out/real-abandoned-table.txt
c4c45ea14cc9c67513f5ad6e02edf8654a0284549aac28874637a3a498631e84  out/real-exempt-table.txt
858cfe5128e6319e9e04b13037c1fa7452970cabaeda6b567641aff0a1d5a36d  out/real-round-names.txt
1e765ef50f585f93a71ef7bf0d012142870629b0a819425fce1a8012a6d6b069  out/z1-thread-prompt.tsv
ef5e15f35066165037b0b2ebc811c757fe58078f388e07dbfc92fe558fde4297  out/z3-heavy-guard.txt
93b5d6827ffff6746045c870b85a9129135e5889a7e47833708b95c9495e8cd2  out/z4-abandoned.tsv
1f50a66b947e199f636816270e44ea724ed3bc53c6208ccd569e4f127a1598e4  out/z6-long-run-prompt.tsv
6e3ea72caa7f6cb91418a2a5cda01061ee26a62155591aaa33eec3837a5a193a  out/z6-script-header.tsv
5a74698c42e4707e10af6beae595c5d8c3c38169e0fd24a13c49bf47ad408685  out/z7-exempt.tsv
```
- 喂法脚本的判别力：每个脚本里放行例与拒绝例成对（例名在 out/ 里），一个例变了结果就在 out/ 的逐行比里显出来；没单独做必红的弄坏开关。
- 草稿目录里的冻结副本拷贝（`b9/`、`wr/`、`z4out/` 等）交回前删掉。

## cite-check 末行

`python3 scripts/agents/cite-check.py research/prompts/m3-defs-b9-r1-sonnet-output.md --root /home/fy5090/code/singlefs` 的末行：

```
  ✓ 核了 60 处引文，对上 60 处，没判 0 处（逐处列在上面）
```
