# m3-close-reuse-closure-r2 判决（推论轮第二轮，2026-10-22）

被判对象：第一轮判决 `research/prompts/m3-close-reuse-closure-r1-main-verification.md` 第三节的形态（丙三件：HostLookup 异常只等于没有提示、判法去掉 `run-with-memory-cap:` 起头的行、只排一条边并让 `exec_module` 连边；甲：第二台跑每一趟同步过去的树副本里那一份）。正文 `research/prompts/_m3-close-reuse-closure-r2-body.md`。腿：正推（Sonnet）`research/prompts/m3-close-reuse-closure-r2-sonnet-output.md`（14e7902c…），攻方（Opus）`research/prompts/m3-close-reuse-closure-r2-opus-output.md`（5ee83323…）。

## 一、核查

1. 开工快照：`judged-sha256.txt` 8 行、`kb-sha256.txt` 3 行，`sha256sum -c` 全部 OK。
2. `python3 scripts/agents/cite-check.py <两份腿报告> --root . --background research/prompts/_m3-close-reuse-closure-r2-background.md`：核了 86 处引文、11 处裸行号，引文全部对上。
3. 两份模型拷进 `~/.local/share/singlefs/work/m3-close-b16-verify/closure-r2-rerun/` 复跑（第五节）：攻方末行 `name=rerun_compare status=same lines=147`；正推输出与腿的 `out/rerun-1.out` 去掉「用了几秒」那一档与排队提示后，只差模型目录路径两行与末尾一行退出码。

## 二、逐格判定

| 格 | 正推 | 攻方 | 主 agent 判 |
|---|---|---|---|
| W1 第 1 件（try 范围） | 冲突：要兜的还有 `begin`、`finish`、`run` 后半段（`size_bytes`、`human`、`shlex`）与 `cancel`；6 个注入点 5 个冒出主线程，兜完 0 个；排队锁与 join 没有时限 | 打中（量过）：`host-resources.py --json` 改一个字段写法，`HostLookup.run` 读完输出之后格式化时抛 TypeError，包装 252 变 251；照字面补的 try 仍是 251，整个 `run` 包住才回 252；判法单元指纹前后同一把 | 两腿同向。打中，挂起：try 要罩 HostLookup 的全部入口与它在主线程里调的全部步骤，锁加 `LOCK_NB`、join 加上限 |
| W1 第 2 件（按前缀去行） | 冲突：去多（升档证据要读的「撞了内存上限」「超过限时」两句也带这个前缀，`Runner.escalate_step` 每次升档判证据不齐）；去少（提示行、告警写不进、线程 traceback 共 12 行不带前缀）。改法：源头给 HostLookup 每行一个固定标记，判法只去标记行 | 打中（量过）：同样的去多两句，限时、内存两种升档从 0 条问题变 2 条；漏掉 `→ 第二台此刻放得下…` 那一行，形式化判法拿它当 252 那一处的「包装说」，句子随第二台可用内存变 | 两腿一致，打中，挂起。采正推的改法：标记打在源头，判法只去标记行 |
| W2 只排一条边 | 一致：35 个入口只 4 个变，其余 31 个逐个相同 | 没打中：同 | 站住 |
| W2 `exec_module` | 冲突：那条 load 边今天就连着；漏的是载入时一并执行的模块级 import。补上之后 `lib_long_run` 回到闭包，语法错变成 `python-syntax` 拒算，25 个入口的键变（多 18 到 45 个单元）、无新拒算 | 打中（量过）：照第 3 件字面做等于没改，它要的自证在切边树上是红的；包装静态 import 的 `memory_peaks_lib.py` 切边后掉 6 个模块级单元，改坏 `import subprocess` 一行四把键不变、包装 0 变 251；闭包算法文件头「import 时的副作用照算」不成立（check_proof 缺 164 个模块级单元、escalate_step 缺 290 个） | 两腿同指一处：闭包不收模块级语句。打中，挂起，而且是今天就有的盲区（不分辨丙甲）。改法：模块级 import 与赋值进闭包（攻方量过 1232 → 1655，拒算不回来；连 class 头会把拒算带回来，不连） |
| W2 别的起 python 的写法 | 一致：按路径载入、`runpy`、子进程、`import_module`、普通 import 五种都只带被起文件自己 | — | 并进上一格的改法 |
| W3 甲 | 冲突：程序按值送本来就保证第二台跑的是同一份文本，甲靠同步保证变弱；`source-stamp.py sync` 是本机到本机；排队探测时第二台没有树副本；要同步的文件在 `scripts/hosts/` 之外还有 5 处 | 打中：`sync` 两头都是本机路径，第二台的树是 `multi-host-run.sh` ④ 拷的；只拷 `scripts/hosts/` 起不来（仓内 10 份、7 份在外，`ModuleNotFoundError: project_preflight`，量过）；同步不带 `--delete`，量过 | 两腿一致，甲的第二轮写法打穿。改法交第三轮（第三节） |
| W3 键里不算第二台 | 一致；但广谱变异 `import_results` 只核键是 64 位十六进制、不重算键，「照已有做法按本机现算的键逐条核」不成立 | 没打中 | 采：键里不算第二台站住；`import_results` 不重算键另登欠账 |
| W4 条文 | 冲突，改写后自洽；`NOT_JUDGING_FILES` 是条文没写的文件级排除，要与边排除并列写进；候选句在报告 4.2 | 打中四处：去行与升档证据打架；兜底判据与三格自证认不出 W1 那一条（键不变、退出码变，落在已定项 17 第 433、434 行）；python 调用方的「声明进键」不成立（注释不进指纹）；`NOT_JUDGING_FILES` 两份经 import 与 load 进闭包（check_proof 里 776 个单元），候选的边判据装不下 | 采：条文要写两种排除并列（文件级 `NOT_JUDGING_FILES` 与边级），自证要加一格「键不变而退出码变」必红 |

## 三、去向

### 第三轮攻的形态（这一轮合成，被攻过零轮）

丙：

1. **HostLookup 整体兜底**：`begin`、`finish`、`run`、`query`、`cancel` 的函数体整段包在 try 里，任何异常只记一行带标记的告警、不改包装退出码；排队锁取 `LOCK_NB`、等线程的 join 加上限。自证：六个注入点（起线程、建临时文件、读输出、格式化、取锁、join）各注入一次，包装退出码与关掉 HostLookup 的那一臂逐个相同；弄坏开关拿掉 try，红在点名那一格。
2. **源头打标记，判法只去标记行**：HostLookup 打的每一行（提示、告警、它线程里的 traceback）以同一个固定标记起头；判法去掉标记行再读，包装别的行（升档证据要读的「撞了内存上限」「超过限时」）照读。自证：同一份 Verus 输出、标记行内容不同，存进库的判定逐字相同；升档证据两句照旧判得出；弄坏开关不去标记行，红。
3. **闭包收模块级语句，再只排一条边**：判法单元闭包把被载入、被 import 的文件的模块级 import 与赋值收进来（不收 class 头）；然后不走 `HostLookup.query` 起 `host-resources.py` 那一条子进程边。自证：改 `lib_long_run.py` 一个模块级 import，形式化入口的键变；改 `memory_peaks_lib.py` 的 `import subprocess`，包装入口的键变；切边前后少掉的单元只经那条边可达。

甲：第二台跑的是本机同一份 `host-resources.py`（连同它在仓内 import 的文件，按值打包送过去），由参数选机器；不依赖第二台有仓、也不依赖同步，送过去的就是本机那几份文件的原样。键里不算第二台（送过去的文件已在本机闭包里）。

条文：D13（验证路线） 已定项 17 与 `.claude/rules/implementation-workflow.md` 写两种排除并列——文件级（`NOT_JUDGING_FILES`，算键的库自己）与边级（只打提示、不改结局的一条子进程边，配上面三件自证）；自证另加一格「键不变而退出码变」必红。

### 另登欠账（不挡被判对象）

| 欠账 | 出处 |
|---|---|
| 判法单元闭包不收模块级 import 与赋值：check_proof 缺 164 个、escalate_step 缺 290 个模块级单元；改坏 `memory_peaks_lib.py` 的 import 键不变、包装退 251（今天就有，随丙第 3 件关） | 两腿 W2 |
| 广谱变异 `import_results` 只核键是 64 位十六进制、不按本机现算的键重算 | 正推 W3 |

## 四、多轮小结

第一轮打中「只打提示」在退出码与写进库的句子两层不成立、按边排除丢掉预测路径上的单元、甲的前提不存在。第二轮：try 范围不够（量出 TypeError 那一形）、按前缀去行去多去少、闭包不收模块级语句（今天就有的盲区）、甲的同步写法打穿；只排一条边、键里不算第二台两条站住。第三轮攻第三节的形态，分工照 `.claude/rules/three-way-inference.md` 第三轮：「已定」项撑不撑得住 / 未定项清单的完整性 / 那一轮报出的数字能不能核。

## 五、复跑

- 攻方模型：`bash scripts/hosts/run-with-memory-cap.sh 8G bash <副本>/opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work <副本>/opus-work`，退出码 0，末行 `name=rerun_compare status=same lines=147`。
- 正推模型：`bash scripts/hosts/run-with-memory-cap.sh 4G bash <副本>/sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work <副本>/sonnet-work`，退出码 0；与腿的 `out/rerun-1.out` 比，去掉「用了几秒」那一档后只差模型目录路径两行（腿在 `research/prompts/` 下跑、复跑在副本里）与末尾一行 `exit=0`。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 17 | 支撑 | 2026-10-22 不受影响：第二轮打中挂起，两种排除并列的条文交第三轮 |
