# m3-close-reuse-closure-r3：判法单元闭包不走内存包装查机器那条提示路径，第二轮修补之后的形态站不站得住（推论轮第三轮，最终轮）正文（2026-10-22）

## 一、题面

前两轮判决：`research/prompts/m3-close-reuse-closure-r1-main-verification.md`、`research/prompts/m3-close-reuse-closure-r2-main-verification.md`。被判对象是用户 2026-10-22 定的「丙加甲」（原话在调度记录 `records/2026-10-18-里程碑三收尾与近期欠账.md`）：形式化门禁三个判法入口与广谱变异 `Runner.escalate_step` 的判法单元闭包，今天经内存包装 `scripts/hosts/run-with-memory-cap.sh` 的 `HostLookup` → `scripts/hosts/host-resources.py` → `scripts/hosts/peer-host-lib.sh` 的 `peer_host_run` 拒算。

第二轮挂起：try 只包三处不够（`run` 读完输出后格式化抛 TypeError 仍让包装 252 变 251）；按 `run-with-memory-cap:` 前缀去行，去多（升档证据两句）也去少（不带前缀的提示行与 traceback）；闭包不收模块级 import 与赋值（今天就有的盲区，`lib_long_run.py`、`memory_peaks_lib.py` 改坏而键不变）；甲靠同步那一写法打穿（`source-stamp.py sync` 是本机到本机、只拷 `scripts/hosts/` 起不来）。

这一轮攻第二轮判决第三节「第三轮攻的形态」（下面第三节照抄）。这是最后一轮：分工照 `.claude/rules/three-way-inference.md` 第三轮——「已定」项撑不撑得住 / 未定项清单的完整性 / 那一轮报出的数字能不能核。这一轮新冒出来的改法算零轮，写进交用户表，不开第四轮。

## 二、实现今天的样子（主 agent 的观测，2026-10-22，行号按主工作区）

- `scripts/hosts/run-with-memory-cap.sh`：内嵌 python 的 `class HostLookup`（第 920 行附近），`begin` 第 939–940 行起线程、`finish` 与 `query` 第 1007 行前台建临时文件，都不在 try 里；排队锁与 join 没有时限。
- `scripts/hosts/host-resources.py`：`remote_probe_program`（第 227 行附近）、`run_on_machine`（第 312–329 行附近，program 经 base64 送 `peer_host_run`）；它在仓内 import 的文件 10 份、7 份不在 `scripts/hosts/`（第二轮攻方量的）。
- `scripts/hosts/host-dispatch.py` 第 315–316 行同样经 `peer_host_run` 送 base64 程序；模块体 import `lib_long_run`。
- `scripts/gate/lib_function_units.py`：脚本一侧的判法单元算法，文件头写「import 时的副作用照算」，第二轮攻方量出 check_proof 缺 164 个、escalate_step 缺 290 个模块级单元。
- `scripts/crash/lib_verdict_library.py` 的 `NOT_JUDGING_FILES`：文件级排除，两份经 import 与 load 进 check_proof 闭包（776 个单元）。
- 广谱变异 `scripts/mutation/broad-mutation.py`：升档证据读包装的「撞了内存上限」「超过限时」两句；`import_results` 只核键的形状。
- 前两轮的模型可以拿来改臂再量：`research/prompts/m3-close-reuse-closure-r1-*-model/`、`research/prompts/m3-close-reuse-closure-r2-*-model/`（只读，要改拷进自己这一轮的模型目录）。

## 三、被判的形态（照抄第二轮判决第三节）

丙：

1. **HostLookup 整体兜底**：`begin`、`finish`、`run`、`query`、`cancel` 的函数体整段包在 try 里，任何异常只记一行带标记的告警、不改包装退出码；排队锁取 `LOCK_NB`、等线程的 join 加上限。自证：六个注入点（起线程、建临时文件、读输出、格式化、取锁、join）各注入一次，包装退出码与关掉 HostLookup 的那一臂逐个相同；弄坏开关拿掉 try，红在点名那一格。
2. **源头打标记，判法只去标记行**：HostLookup 打的每一行（提示、告警、它线程里的 traceback）以同一个固定标记起头；判法去掉标记行再读，包装别的行（升档证据要读的「撞了内存上限」「超过限时」）照读。自证：同一份 Verus 输出、标记行内容不同，存进库的判定逐字相同；升档证据两句照旧判得出；弄坏开关不去标记行，红。
3. **闭包收模块级语句，再只排一条边**：判法单元闭包把被载入、被 import 的文件的模块级 import 与赋值收进来（不收 class 头）；然后不走 `HostLookup.query` 起 `host-resources.py` 那一条子进程边。自证：改 `lib_long_run.py` 一个模块级 import，形式化入口的键变；改 `memory_peaks_lib.py` 的 `import subprocess`，包装入口的键变；切边前后少掉的单元只经那条边可达。

甲：第二台跑的是本机同一份 `host-resources.py`（连同它在仓内 import 的文件，按值打包送过去），由参数选机器；不依赖第二台有仓、也不依赖同步，送过去的就是本机那几份文件的原样。键里不算第二台（送过去的文件已在本机闭包里）。

条文：D13（验证路线） 已定项 17 与 `.claude/rules/implementation-workflow.md` 写两种排除并列——文件级（`NOT_JUDGING_FILES`，算键的库自己）与边级（只打提示、不改结局的一条子进程边，配上面三件自证）；自证另加一格「键不变而退出码变」必红。

## 四、格

| 格 | 问什么 |
|---|---|
| X1 | 「已定」项撑不撑得住：被判形态与 D13 已定项 17（逐句读原文，尤其「键过窄」「不静默少收」「拒算的照跑、不复用」）、`.claude/rules/implementation-workflow.md`「复用重型门禁的判定」一节、`.claude/rules/verification.md` 判定库那几句是否字面或语义打架；「收模块级 import 与赋值」之后，已登记的 35 个入口里哪些键变、会不会冒出新的拒算 |
| X2 | 未定项清单的完整性：被判形态与第二轮「另登欠账」漏了哪些要随这一改一起动的地方（判法读包装输出的全部调用点、按值打包要带的文件、`host-dispatch.py` 那一路、`selftest-registry.tsv` 与门禁 code-tooling 的 reuse-selftests 格、文件头说明）；甲「按值打包送同一份文件」在第二台缺 python 包或版本不同时会怎样 |
| X3 | 前两轮报出的数能不能核：第一轮「单元 1579 → 1232」「792 → 436」「35 个入口只 4 个变」；第二轮「check_proof 缺 164、escalate_step 缺 290 个模块级单元」「1232 → 1655」「776 个单元」「25 个入口的键变、多 18 到 45 个单元」「仓内 10 份、7 份在外」。逐个用独立于原腿的办法复核 |

## 五、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | X1–X3 | 复核两份判决：替被判出局或改掉的写法辩护（按前缀去行、「认 `exec_module` 字面路径」、甲靠同步），判打中够不够得着、是不是同样打中所有替代写法；X3 的数逐个复核 |
| 云端攻方（Opus） | X1–X3 | 找反例：X1 被判形态与已定条款打架、收模块级语句之后冒出的新拒算或错键；X2 漏掉的落点、按值打包在第二台环境不同时的失败；X3 前两轮数字核不上的。不重复前两轮攻过的角度（TMPDIR 被删退 251、stderr 末行进句子、`lib_long_run` 经那条边进闭包、第二台没有常驻仓、ssh 串不连边、try 只包三处、按前缀去行、闭包不收模块级语句这件事本身、同步写法） |

方案按 `crates/` 与 `scripts/` 今天的实现来谈。引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
