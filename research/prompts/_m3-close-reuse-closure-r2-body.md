# m3-close-reuse-closure-r2：判法单元闭包不走内存包装查机器那条提示路径，第一轮修补之后的形态站不站得住（推论轮第二轮）正文（2026-10-22）

## 一、题面

第一轮判决 `research/prompts/m3-close-reuse-closure-r1-main-verification.md`。被判对象是用户 2026-10-22 定的「丙加甲」（原话在调度记录 `records/2026-10-18-里程碑三收尾与近期欠账.md`）：形式化门禁三个判法入口（`scripts/gate/verus-proof-check.py` 的 check_proof、check_vacuity、check_must_red_worlds）与广谱变异的 `Runner.escalate_step` 的判法单元闭包，今天经内存包装 `scripts/hosts/run-with-memory-cap.sh` 的 `HostLookup` → `scripts/hosts/host-resources.py` → `scripts/hosts/peer-host-lib.sh` 的 `peer_host_run`（`bash -c "$2"`）拒算，于是每次现判、不复用。

第一轮挂起：「只打提示」在两层不成立——包装退出码（前台建临时文件与起线程没包在 try 里，252 变 251）与写进库的判定句子（判法把包装整份 stderr 当诊断读，HostLookup 那一行带着第二台 stderr 末行）；按调用边排除会顺带丢掉预测路径上会改结局的单元（`lib_long_run.py`，今天只经那条边进闭包，`predicted_bytes` 的 `exec_module` 不连边）；甲要的「第二台常驻同版本仓」今天不存在。

这一轮攻的是判决第三节「第二轮攻的形态」（下面第三节照抄）。分工照 `.claude/rules/three-way-inference.md` 第二轮：第一轮的修补本身 / 产生结论的方法 / 从未被看过的地方。

## 二、实现今天的样子（主 agent 的观测，2026-10-22，行号按主工作区）

- `scripts/hosts/run-with-memory-cap.sh`：内嵌 python 的 `class HostLookup`（第 920 行附近），`begin` 里第 939–940 行起线程、`finish` 与 `query` 里第 1007 行前台建两份 `tempfile.TemporaryFile`，都不在 try 里；排队退 252 之前前台再查一次。
- `scripts/hosts/host-resources.py`：第 227 行 `remote_probe_program`、第 312–329 行 `run_on_machine`（program 经 base64 编码送 `peer_host_run`）；`predicted_bytes` 经 `importlib` 的 `exec_module` 起 `scripts/hosts/host-dispatch.py`，后者模块体 import `lib_long_run`。
- `scripts/hosts/host-dispatch.py` 第 315–316 行同样经 `peer_host_run` 送 base64 程序。
- `scripts/hosts/peer-host-lib.sh` 第 76 行 `peer_host_run`。
- `scripts/hosts/source-stamp.py`：`sync <源树> <副本>` 把树拷到副本（不拷 `.git` 与 `target`）。`multi-host.env.example` 第 11 行：第二台每一趟在专用目录下建子目录放树副本、跑完删。
- `scripts/gate/lib_function_units.py`：脚本一侧的判法单元算法；`lib_verdict_library.py` 有文件级排除 `NOT_JUDGING_FILES`；`scripts/mutation/broad-mutation.py` 第 1466 行 `library.closure_key(closure, allow_refused=True)`。
- 判法读包装 stderr 的地方：`scripts/gate/verus-proof-check.py`（diagnostics）、`scripts/mutation/broad-mutation.py`、门禁 checker-tier-crates-mutation-replay（遇 251 整张表停）。
- 第一轮两条腿的模型可以拿来改臂再量：`research/prompts/m3-close-reuse-closure-r1-opus-model/`、`research/prompts/m3-close-reuse-closure-r1-sonnet-model/`（只读，要改拷进自己这一轮的模型目录）。

## 三、被判的形态（照抄判决第三节）

丙改成三件，前两件是前提：

1. **查机器那一支出了异常只等于没有提示**：`HostLookup` 的起线程、前台建临时文件、读输出整段包在 try 里，任何异常都只记一行告警、不改包装的退出码（该退 252 照退 252）。自证加两格：注入「起不了线程」与「建不了临时文件」，包装退出码与关掉 HostLookup 的那一臂逐个相同；弄坏开关把 try 拿掉，这两格红。
2. **判法不读包装自己打的行**：凡读包装标准错误当诊断的判法（形式化三个入口、广谱变异、crates 变异），先去掉以 `run-with-memory-cap:` 起头的行再读。自证加一格：同一份 Verus 输出，HostLookup 那一行带不同的第二台末行，存进库的判定逐字相同；弄坏开关不去掉这些行，这一格红。
3. **按调用边排除，只排一条边**：判法单元闭包不走 `HostLookup.query` 起 `host-resources.py` 那一条子进程边；同时 `predicted_bytes` 的 `exec_module` 起 `host-dispatch.py` 要连成边（闭包算法认 `exec_module` 的字面路径，或改成静态 import），让 `lib_long_run.py` 等预测路径上的单元留在闭包里。自证：切边前后闭包里少掉的单元恰好是只经那条边可达的那几个；`lib_long_run.py` 的一个单元改了，形式化入口的键变。

甲改成：第二台跑的是同一份脚本，取自每一趟同步过去的树副本（`scripts/hosts/source-stamp.py sync` 拷的那一份），探测前先把 `scripts/hosts/` 下用到的几份同步过去，由参数选机器；host-resources 与 host-dispatch 不再送 base64 程序。键里不算第二台那一侧，第二台判出的东西导入本机时照已有做法按本机现算的键逐条核。`bash -c "$N"` 的折法这一题不做（丙之后已量的入口不再需要它）。

## 四、格

| 格 | 问什么 |
|---|---|
| W1 | 丙第 1、2 件之后，HostLookup 那条路还有没有改判定结局（退出码、判红判绿、写进库的内容）的地方：try 之外还剩哪几处（告警记录、节流文件、第二台挂死时的时限）；「去掉 `run-with-memory-cap:` 起头的行」会不会把判法真要读的行一起去掉，或漏掉 HostLookup 打出的不以它起头的行 |
| W2 | 丙第 3 件：「只排那一条边」加「`exec_module` 连边」之后，闭包里少掉的单元是不是都不改结局；别的经 `importlib`、`runpy`、`subprocess` 起 python 文件的地方今天是不是同样不连边；按边排除写进闭包算法会不会让别的判定入口（登记表里 35 个）的闭包变 |
| W3 | 甲：每一趟同步过去的树副本里那份脚本与本机那份是不是同一份（同步时机、探测前同步几份、同步失败时怎么办）；HostLookup 探测发生在本机排队时，那一刻第二台上有没有树副本；「键里不算第二台」在第二台跑的是同一份脚本时站不站得住 |
| W4 | 丙加甲合起来，D13（验证路线） 已定项 17 与 `.claude/rules/implementation-workflow.md`「复用重型门禁的判定：按每条判定的单元键判」一节要补的条文（逐字候选），与 `lib_verdict_library.py` 的 `NOT_JUDGING_FILES` 文件级排除、与已定项 17 第 432–435 行是否自洽 |

## 五、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | W1–W4 | 从附录条款与第二节的脚本今天的代码正推被判形态每一件该不该这样做；W1、W2 写出改法各落在哪个函数，自证格怎么造 |
| 云端攻方（Opus） | W1–W4 | 找反例：W1 改完之后仍改结局的机器状态；W2 只排一条边仍丢掉、或连了 `exec_module` 仍漏掉的会改结局的单元；W3 同步那一步失败或时机不对时第二台跑的不是同一份；W4 候选句与已定条款打架。不重复第一轮攻过的角度（TMPDIR 被删退 251、stderr 末行进判定句子、`lib_long_run` 经那条边进闭包、第二台没有常驻仓、ssh 串不连边这几件事本身） |

方案按 `crates/` 与 `scripts/` 今天的实现来谈。引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
