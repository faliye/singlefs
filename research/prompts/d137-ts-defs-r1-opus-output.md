# d137-ts-defs-r1 攻方腿报告（A 格：找漏洞）

日期：2026-10-31。被判 21 份定义、规则、skill 与对照文件照开工快照 `research/prompts/d137-ts-defs-r1-snapshot/files-sha256.txt` 核过：开工时全部对得上；写报告时 `.claude/agent-common.md` 已被别的会话改过（此刻 sha256 前 8 位 8858e562，快照里是 4f111ee5；行数 118，本报告引的第 3、46、59、62、63、70、90、104 行逐句现查仍在原行，第 74 行内容变了，引它时写的是此刻的原文）。其余被引文件此刻 sha256 与快照相同。

引文一律写「路径:行号（sha256 前 8 位）」。不在快照里的被引文件此刻的前 8 位：`task-server/task_server.py` d799e304、`scripts/hosts/multi-host-run.sh` df266c77、`scripts/agents/three-way-rerun-template.sh` d9447774、`.claude/hooks/write-guard.sh` f6259538、`.claude/hooks/completion-cleanup-guard.sh` 20e550e3。

## 复跑命令与模型文件

模型目录 `research/prompts/d137-ts-defs-r1-opus-model/`：`samples/` 是喂钩子的输入，`tools/feed.sh` 逐条喂、每条打一行 `name=… exit=… expected=…`（expected 是照被判定义原文该得的退出码，跑之前写死在脚本里），`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷、只改了 `rerun_steps`。`out/` 是这一次跑出的原样输出。

复跑之前先重建样本 h1、h2 点到的仓副本（我交回前已删，删前 190M）：
`mkdir -p /home/fy5090/.local/share/singlefs/work/d137-ts-defs-r1-attack/copy && rsync -a --exclude target --exclude .git scripts .claude crates Cargo.toml Cargo.lock .cargo task-server /home/fy5090/.local/share/singlefs/work/d137-ts-defs-r1-attack/copy/`（在仓根起）。

复跑（非重型）：`task-server/task submit --cwd <新草稿目录> -- bash <模型目录>/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/d137-ts-defs-r1-attack/copy --work <新的空输出目录>`；判读 `task-server/task log <号> | grep '^name='`，与 `out/feed-task-1786/summary.txt` 逐行比。

我这一次跑的：任务 1786，`task wait` 末行（量的）：
`TASK_SERVER_RESULT task=1786 type=succeeded exit=0 attempts=1 log=/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-1786.attempt-1.log`

| 文件 | sha256 前 16 位 |
|---|---|
| tools/rerun.sh | f09bd7193e1dded9 |
| tools/feed.sh | f03c8853dc4b9cbc |
| samples/h0-light-crash-main.json | 31f64c36fc895e98 |
| samples/h1-light-crash-in-copy.json | 8150d5775150d1bc |
| samples/h2-light-formal-in-copy.json | c3c22fa9e9cee285 |
| samples/h3-rerun-builds-and-runs-probe.json | a556d2bef7f11b95 |
| samples/h3b-rerun-feeds-hook-by-variable.json | 73524ea9f2ee9112 |
| samples/h4-kb-scribe-precheck-as-written.json | 56000635c11f3623 |
| samples/h5-writer-combined-clippy-build.json | 83cf0bd1885a767e |
| samples/w1-kb-scribe-precheck-in-scope.json | 97ae5a59bd616988 |
| samples/w2-kb-scribe-precheck-out-of-scope.json | 3e213b3ddf1ec653 |
| samples/w2b-kb-scribe-precheck-out-of-scope-with-cwd.json | bdf33eac36a68868 |
| samples/w2c-kb-scribe-precheck-out-of-scope-absolute.json | 47d9f81ba230f6e0 |
| samples/rerun-probe.sh | 48bde10a08c8d46b |
| samples/rerun-feeds-hook.sh | 13461ef47eadd988 |
| out/feed-task-1786/summary.txt | 2fc74e68acc26066 |
| out/feed-task-1786/h1-light-crash-in-copy.out | e0d4ee2f0bf71242 |
| out/feed-task-1786/w3-precheck-input-missing.out | 76c0ac772905a62f |
| out/placement-probes-1787-1791.txt | d8a813ab9806dbb7 |
| out/write-guard-retry-task-1804.txt | 82b1cc4b6544dd8e |
| out/README.txt | d44afd41348185c0 |

`out/feed-task-1786/` 下其余 8 份 `.out` 都是空文件（sha256 e3b0c44298fc1c14…，钩子放行时什么都不打）。

## 各格判定一览

| 号 | 漏洞（一句） | 种类 | 证据 |
|---|---|---|---|
| A1 | 从仓根交、不带 `--cwd` 的命令会被派到第二台；写进仓里或草稿目录的文件不拷回本机，任务照样报 `succeeded`；定义里没有一处提到 | 结局读错、产物丢 | 量的：任务 1787–1791 |
| A2 | 实现员照定义在自己的副本里交崩溃点重放轻型（`--cwd <副本>`），重型测试闸当重型拒，出路叫它写进「要主 agent 跑的命令」，与实现员定义「轻型自己交、不写进那一节」互相矛盾 | 撞拒、互相矛盾 | 量的：样本 h1 退 2；h0、h2 退 0 |
| A3 | 执行员、工具员定义这一批改成「命名检查同样交主 agent」，共用约束仍写它们自己交，交回闸照共用约束拒；执行员每次都改 `scripts/` 下的 `.rs`，每次都撞 | 撞拒、互相矛盾 | 读的 |
| A4 | 实现员照第 4 步交的 `cargo clippy --all-targets --all-features` 被服务认成要卡的任务：占一张卡、先清场、跑完不复原 | 资源误用、副作用 | 读的；样本 h5 证明钩子放它过去 |
| A5 | 「手跑的 GPU 测试分两条交（先 `--no-run`）」在服务下自相抵消：两条各占一张卡、各拿一份新的空编译目录，第二条照样从头编 | 互相矛盾、资源误用 | 读的 |
| A6 | `task wait` 起头 60 秒连不上服务就退 78、不打末行，出路写「服务好了再提交」；子 agent 照出路重交，原任务还在自己的单元里跑，同一件跑两份 | 结局读错、重复跑 | 读的 |
| A7 | 共用约束写「服务对子 agent 交的重型当场拒」，服务分不出子 agent；唯一的一道是钩子，钩子判不了时照常执行 | 漏判（说法错位） | 读的 |
| A8 | 经 `submit` 交的 cargo 不认合入锁：钩子明写不判，服务里也没有这一判；合入后验证打补丁时别的 agent 在主工作区交的编译照起 | 漏判 | 读的 |
| A9 | 子 agent 与主 agent 同一个会话号：「只许提交它的会话」的 `cancel`、`clean` 实际上谁都能对兄弟任务做；执行员跨轮要用的编译目录可被别的 agent `clean` 掉 | 漏判、目录误删 | 读的 |
| A10 | 写回员喂写范围闸的样本里没有 `cwd`、判据只认「退出码 2 就停」：任务的工作目录不是仓根时范围外的路径放行（量的），输入文件读不到时退 1、被当成过 | 漏判、结局读错 | 量的：w2 退 0、w2b、w2c 退 2、w3 退 1 |
| A11 | 分片任务的 `task log <号>` 给的是号最大的那个子任务的日志，不是没成的那一片；skill 叫主 agent 把它当「整份输出」存 | 结局读错 | 读的 |
| A12 | 服务不通时客户端出路是 `python3 task-server/control start`，子 agent 起它被重型测试闸拒；定义没写子 agent 遇到 78 怎么办 | 出路指不到合规写法 | 读的 |
| A13 | 共用约束「副本只拷要编的」拷不进 `scripts/`，而轻型三条要在副本里起 `scripts/crash/…` | 互相矛盾 | 读的 |

## A1　从仓根交的命令被派到第二台，写出的文件不回本机

**照定义原文做的那一步**（只举几处，同形的还有）：
- `.claude/agents/experiment-runner.md:38`（59d3f26f）：「4. 产物命令把输出写进 `research/results/`（照第 1 步交，输出文件名照下面的拼法写进命令）。」第 1 步（同文件第 28 行）是 `task-server/task submit [--cwd <目录>] -- <命令>`，`--cwd` 可省；第 31 行 1c：「照第 1 步经 `task-server/task submit -- bash <脚本>` 交」；第 30 行入库装置那一支：「执行员交回的产物命令只执行这一份（写成 `task-server/task submit -- <二进制…>`）」。
- `.claude/agents/implementation-writer.md:39`（b0466a2f）：验证命令「串在同一条 Bash 里，各段退出码各写进自己的文件、最后逐段读」；在主工作区改的实现员，第 1 步的 `<根>` 就是仓根。
- `.claude/agents/three-way-attack.md:40`（9d5aa05a）：一条命令里并行的几件「每件把退出码写进自己的文件」，`{ <命令>; echo "$?" > <草稿目录>/b<批号>-<件号>.rc; } &`。
- `scripts/agents/three-way-rerun-template.sh:7`（d9447774）的用法行不带 `--cwd`：`task-server/task submit -- bash <模型目录>/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录> [--model <模型目录>]`；第 11 行：`--work    新的草稿目录：不在、或是空目录；跑出来的全写在这里`。

**撞上的服务那几行**：
- `task-server/task_server.py:1264`（d799e304）：`if not snapshot.peer_available or os.path.realpath(task["cwd"]) != os.path.realpath(self.peer_root):` ——工作目录是仓根就有资格去第二台；不给 `--cwd`、Bash 的当前目录又是仓根（子 agent 每次调用都回到仓根），正好落进去。
- 同文件 `:1305`：`if not value.startswith("/"):` ——只看以 `/` 起头的整词；相对路径、`bash -c '…'` 里的字符串一律不看。
- 同文件 `:1319`：`return None   # 不在的（多半是要写的产物）、设备与管道：只在本机` ——只有「整词是绝对路径、此刻又不在」才钉本机。模板第 11 行许的「空目录」此刻在，照样随树送过去。
- `task-server/README.md:21`（89b2e7ee）：「参数里点到的仓外文件与目录（合计至多 2G、此刻在的）由服务随树送到第二台、换成那边的相对路径（送过去的是拷贝，第二台写进去的不拷回）；」
- `scripts/hosts/multi-host-run.sh:35`（df266c77）：拷树用 `git ls-files -co --exclude-standard`，列不出空目录，所以本机只有空目录、第二台连目录都没有。

**量的**（模型 `out/placement-probes-1787-1791.txt`）：从仓根、不带 `--cwd` 交五条只写一个文件的命令（`bash -c 'echo probe-N > research/prompts/d137-ts-defs-r1-opus-model/out/peer-probe-N.txt'`）。
- 1787–1789：目标目录 `out/` 当时是空的。三条都在第二台跑，`TASK_SERVER_RESULT task=1787 type=command-failed exit=1 attempts=1 log=/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-1787.attempt-1.log`（1788、1789 同形）；日志原话 `bash: line 1: research/prompts/d137-ts-defs-r1-opus-model/out/peer-probe-1.txt: No such file or directory`。
- 往 `out/` 放进一份 `README.txt` 之后交 1790、1791：两条都在第二台跑，`TASK_SERVER_RESULT task=1790 type=succeeded exit=0 attempts=1 log=/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-1790.attempt-1.log`（1791 同形，`type=succeeded exit=0`）；跑完 `ls research/prompts/d137-ts-defs-r1-opus-model/out/` 只有 `README.txt`，`peer-probe-4.txt`、`peer-probe-5.txt` 本机都没有。
- 同一时段我从草稿目录（`--cwd`）交的任务 1786 排着，`task status 1786` 原话「在等：本机：内存：预测 4.0 GiB + 余量 2.0 GiB、要记 4.0 GiB，此刻空闲 81 MiB、slice 放得下 0 MiB」：本机放不下时，从仓根交的去第二台。

**后果**：
1. 执行员的产物命令、1c 的长跑脚本：`type=succeeded`，`research/results/` 里没有新文件（或只有上一次的旧文件），续跑脚本存的进度同样丢；执行员第 4 步只叫它「跑完按这个新文件名认本轮才有的完成标记」，认不到就当没跑完再交，再派去第二台。
2. 实现员、攻方腿的 `.rc` 文件：写在第二台、或在第二台因目录不在而写不出，本机数到的份数对不上，照定义「整批作废」再跑，同样可能再去第二台。
3. 模板许 `--work` 是空目录：空目录被当「此刻在的」仓外目录送过去，复跑输出写进第二台那份拷贝。
4. 目标目录在本机是空目录时，第二台报 `command-failed` 退 1，看起来像命令自己错了（1787–1789 就是这样），会被当成代码问题去查。
5. 结局行 `type=succeeded` 本身是对的，读错的是「产物落盘了」这一层：被判的 21 份里没有一处说第二台写的文件不回本机，`grep -n '第二台\|不拷回\|拷回'` 在这 21 份里命中 9 行（量的），讲的是私有主机信息、分片、硬件运行条件、崩溃放量与广谱变异的跨台导入，没有一行讲写出的文件回不回本机。

**推翻条件**：本机放得下时 `一样大取本机`，同一批命令都留在本机，这一格只在本机满载时打中；要推翻得给出一次「从仓根交、写相对路径、本机放不下」而任务仍留本机或文件回到本机的记录。

**四句**：分不分辨臂——这一题没有臂，打中的是「定义叫写文件」与「服务把写文件的命令派走」之间的空档，执行员、实现员、三方腿各份同受影响。系统看不看得到——服务看得到工作目录与整词绝对路径，看不到相对路径与 `bash -c` 里的写；定义作者看得到 README 第 21 行。满足的分句——A 格「会让任务结局被读错」「定义没覆盖到、照字面做会出错的边角」。跑前条款的改法——没有跑前条款；可能的改法见文末「改法表」，都是推的、被攻过零轮。

## A2　副本里交的崩溃点重放轻型被当重型拒，出路与实现员定义互相矛盾

**照定义原文做的那一步**：
- `.claude/rules/verification.md:50`（2687ec24）轻型那一行：「实现员、调查员在自己的副本里交（`--cwd <副本>`）」。
- `.claude/rules/implementation-workflow.md:43`（e24bbf10）：「各经 `task-server/task submit --cwd <自己的副本> -- <命令>` 交」。
- `.claude/agents/implementation-writer.md:34`（b0466a2f）第 1d 步：命令原样可抄，「`<根>` 填你改动所在的树，`--item` 按自己改到的文件点名」；交补丁的实现员改动所在的树就是副本。`.claude/agents/investigator.md:32`（9361c3aa）「查到就修」的调查员同样「命令照 `.claude/agents/implementation-writer.md` 第 1d 步自己交」。

**撞上的钩子那一行**：`.claude/agent-common.md:90`（8858e562）转述的重型测试闸判法：「放行靠参数的三份入口（`replay.sh`、`crash-point-replay.sh`、`crash-amplification.sh`）只认本仓那一份，别处的拷贝与经符号链接起的照读正文」。副本里那一份读正文，正文里有 `cargo test -p singlefs-checker-tier`，判成 checker 档。

**量的**（任务 1786，`out/feed-task-1786/summary.txt`）：
- `name=h0-light-crash-main hook=heavy-test-guard exit=0 expected=0`（同一条命令在仓根交：放行）
- `name=h1-light-crash-in-copy hook=heavy-test-guard exit=2 expected=0`（`--cwd <副本>`，副本里那份 `scripts/crash/crash-point-replay.sh` 与主仓逐字节相同，`cmp` 过）
- `name=h2-light-formal-in-copy hook=heavy-test-guard exit=0 expected=0`（形式化那一条在副本里交：放行）

h1 的拒绝说明（`out/feed-task-1786/h1-light-crash-in-copy.out` 首行与出路行原样）：
`✗ implementation-writer 经 task-server 提交的命令里有子 agent 不跑的一类：cargo test 跑到 singlefs-checker-tier 包的测试（checker 档，默认只在提交时跑；--test $target）（checker 档）`
`→ 怎么办：这一条不提交，把命令原样写进交回报告「要主 agent 跑的命令」一节（写成 `task-server/task submit -- <命令>`，标「重型」），交回之后由主 agent 提交。`

**后果**：出路要实现员把轻型标成「重型」写进「要主 agent 跑的命令」；`.claude/agents/implementation-writer.md:39` 却把「门禁阶段（第 1d 步那三条 `--quick` 与那两条命名检查除外）」划在自己交的一边，并要「红了接着改」。照出路办，轻型变成交回项，主 agent 收到一条标「重型」的轻型命令；照定义办，就得换成在仓根交——那跑的是主工作区，不是副本里的改动，量的不是自己的改动。两条路都不是定义写的那一条。崩溃与变异两条里只有崩溃这一条撞（形式化 h2 放行；变异那一道是门禁阶段，按文件名认，推的：与形式化同一种认法，未单独喂）。

**推翻条件**：同一份 h1 样本在钩子换版之后退 0；或者有一处定义已写明「崩溃点重放轻型在仓根交」。

**四句**：分不分辨——只打崩溃点重放入口，形式化与变异两道不受影响（h2 量过，变异推的）。看不看得到——钩子看得到是副本里的拷贝，它照设计读正文；定义作者看得到 `agent-common.md:90` 这一句。满足的分句——「会撞拒」「两份定义互相矛盾」「被拒之后出路指不到合规写法」三句都中。改法——见改法表。

## A3　执行员、工具员的「命名检查交主 agent」与共用约束、交回闸相反

**照定义原文做的那一步**：
- `.claude/agents/experiment-runner.md:42`（59d3f26f）：「6. 门禁阶段交回前不跑（命名检查同样交主 agent，共用约束「门禁」一节）」；同文件第 59 行「没做什么」也写「命名检查也交主 agent」。`git diff HEAD` 显示这一句是这一批从「（命名检查除外，共用约束「门禁」一节）」改过来的（量的：`git diff HEAD -- .claude/agents/experiment-runner.md` 的删除行 `-6. 门禁阶段交回前不跑（命名检查除外，…`）。
- `.claude/agents/tooling-writer.md:35`（4fc13d5e）：「7. 交回前不跑门禁阶段与各类 lint（命名检查同样交主 agent，共用约束「门禁」一节）」，同样是这一批改的。

**它指到的那一节说的相反**：`.claude/agent-common.md:104`（8858e562）「命名检查交回前跑」：「改了代码或实验源码（`crates/`、`scripts/` 下的 `.rs`）的子 agent（`implementation-writer`、`experiment-runner`、`tooling-writer`）交回前，在改动所在的树上自己经服务交两条命令」。

**撞上的钩子**：`.claude/hooks/handback-guard.sh:485`（0c06912c）`NAMING_CHECKED_AGENTS = {"implementation-writer", "experiment-runner", "tooling-writer"}`；判法在同文件第 33 行起（⑤），共用约束第 99 行转述：「而那棵树最后一次改 `.rs` 之后没经 `task-server/task submit` 交过「命名检查交回前跑」那两条命令」就拒交回。

**后果**：执行员照第 2 步（同文件第 32 行「2. 写 `scripts/experiment-apparatus/e<号>_<英文名>.rs`」）每段都在主工作区写 `scripts/` 下的 `.rs`，照第 6 步不交命名检查——交回闸每次都拒。共用约束第 3 行又写「与定义冲突时以定义为准」，定义说不交、交回闸的出路说交，子 agent 两头为难：照出路改是违背自己的定义，照定义办交不回。工具员改 `scripts/` 下 `.rs` 的少见，撞的机会小，矛盾同形。这一格是读的：交回闸要会话记录，我造不出能喂它的真会话记录，没量。

**推翻条件**：交回闸对执行员改了 `scripts/experiment-apparatus/*.rs`、没交命名检查的交回放行；或者共用约束第 104 行与交回闸 `NAMING_CHECKED_AGENTS` 已改成不含这两类。

**四句**：分不分辨——只打执行员与工具员，实现员三份一致（实现员定义第 39 行把命名检查列在自己交的一边）。看不看得到——交回闸看得到会话记录里有没有那两条，判得出。满足的分句——「两份定义互相矛盾」「会撞拒」。改法——两份定义改回「命名检查除外」，或共用约束与交回闸同时去掉这两类（后一种要动钩子，推的）。

## A4　实现员的 clippy 带 `--all-features`，被服务认成要卡的任务

**照定义原文做的那一步**：`.claude/agents/implementation-writer.md:39`（b0466a2f）验证命令里有「`cargo clippy --all-targets --all-features -- -D warnings` 加 `.claude/singlefs-ai-sop/scripts/check.sh` 里 `CODE_DISCIPLINE_LINTS` 那几条 `-D`」，并要「clippy、build 与动到的测试二进制合成一条交」。

**撞上的服务那几行**：
- `task-server/task_server.py:544`（d799e304）`if word == "--all-features":` 下一行 `return True`（函数 `cargo_uses_gpu` 从第 538 行起，只看命令词是不是 `cargo` 与 `--` 之前的特性参数，不看子命令是 clippy 还是 test）。
- 同文件 `:607` `if cargo_uses_gpu(command.words):`，下一行把整件记成要卡（「带 gpu 特性的 cargo」），`bash -c` 包着的照样剥开认（第 598–605 行）。`crates/singlefs-checker-tier/Cargo.toml:43` 有 `gpu = ["dep:wgpu", "dep:pollster"]`，所以 `--all-features` 就是带 gpu。
- `.claude/rules/verification.md:63`（2687ec24）：带 `gpu` 特性的 cargo 算「凡用 GPU 先清场」，「跑完、跑红或被打断都不复原」。

**量的部分**：样本 h5（实现员把 clippy 与 build 合成一条 `bash -c` 交）`name=h5-writer-combined-clippy-build hook=heavy-test-guard exit=0 expected=0`：钩子放它过去，GPU 的认法在服务那一边。服务那一边我没交 cargo（这一题不起 cargo、不碰 GPU），是读代码推的。

**后果**（推的）：每个实现员每次交验证命令，整条（clippy 加 build 加测试二进制）都排进 GPU 那一队：等卡、占一张卡、先清场；清场回读没过就退 78（`run-condition-unmet`），实现员定义里没有一句说 clippy 退 78 怎么办；跑完卡不复原。clippy 自己一张卡都用不上。
**推翻条件**：交一条 `cargo clippy --all-targets --all-features` 的 `submit` 回显 `gpu=0`。这一条要起 cargo，按这一题的交付要求我没交，写进「要主 agent 跑的命令」。
**四句**：分不分辨——只打带 `--all-features` 的 clippy（也打主 agent 自己若照抄）；merge-and-verify 里的 clippy 在脚本里面、不在命令位置上，那条脚本本来就按名字算要卡的入口（`task-server/task_server.py:133` 的 `GPU_ENTRY_SCRIPT_NAMES`），与这一格无关。看不看得到——服务只看参数，分不出「编 gpu 代码」与「用卡跑」。满足的分句——「定义没覆盖到、照字面做会出错的边角」。改法——见改法表。

## A5　「手跑的 GPU 测试分两条交」在服务下两条都占卡、第二条从头编

**照定义原文做的那一步**：`.claude/rules/verification.md:63`（2687ec24）：「手跑的 GPU 测试分两条交：先交 `--no-run` 只编的那一条，等到结局再交只跑测试的那一条」。

**撞上的服务那几行**：
- `task-server/task_server_lib.py:122`（34dd46ae）：「调用方没给编译目录的，本机这一次尝试设 CARGO_TARGET_DIR=<TASK_SERVER_BUILD_DIRECTORY>/<尝试标签>/target」——每一次尝试一份新的空编译目录。第二条 `submit` 是另一个任务、另一个尝试标签，拿不到第一条编好的东西。
- `task-server/task_server.py:538` 起的 `cargo_uses_gpu`（d799e304）只看 `--features`、`-F`、`--all-features`，没有把 `--no-run` 除外：只编的那一条同样记成要卡、同样清场。
- 定义里没写把第一条的 `build=` 目录当 `CARGO_TARGET_DIR` 交给第二条（服务许调用方给仓外的编译目录，`task_server_lib.py:126`「落在仓外的照它、不删（归调用方）」），也没写这样给了之后第一条的 `task clean` 会删掉第二条要用的目录。

**后果**（推的，没交 cargo）：分两条本来是为了「编的时候不占卡」；在服务下两条各占一张卡、各清一次场，第二条在占着卡的时候把整棵依赖从头编一遍，比一条还慢。照字面做没撞拒，结局也读得对，是做法与目的自相抵消。
**推翻条件**：交 `cargo test --no-run --features gpu …` 回显 `gpu=0`；或第二条的 `task status` 显示用的是第一条的编译目录。
**四句**：分不分辨——只打手跑的 GPU 测试；门禁与入口脚本「照各自的起法」，不受影响。看不看得到——服务看得到 `--no-run`，没用它。满足的分句——「定义没覆盖到、照字面做会出错的边角」（资源）。改法——见改法表。

## A6　`task wait` 起头连不上就退 78，出路叫子 agent 重交，原任务还在跑

**照定义原文做的那一步**：子 agent 交完用 run_in_background 起 `task-server/task wait <号>`；被拒或报错时，`.claude/agent-common.md:74`（8858e562）此刻原文：「是写法的问题就照出路把命令或脚本改成合规的写法再交」；第 73 行只说「`task wait` 报了错、或等的条件不会成立，把看到的命令与输出写进草稿目录里的进度记录，接着做定义里还能做的」。

**撞上的客户端那几行**：
- `task-server/task:229`（f351d875）：`if (connected_once and lib.broken("wait-gives-up-on-disconnect")) or (not connected_once and time.monotonic() > startup_deadline):`，第 44 行 `STARTUP_RETRY_SECONDS = 60.0`：一次都没连上、过了 60 秒就退出，第 230 行用的是 `unreachable_error`。
- `task-server/task_server_lib.py:1243`（34dd46ae）那条出路的结尾：`"不退回直接执行命令，服务好了再提交")`——wait 用的是同一条出路。
- 退出码是 78（`report_unavailable_and_exit`），没有 `TASK_SERVER_RESULT` 末行。

**后果**（推的）：子 agent 交完、服务在它起 wait 之前停了一分钟以上（重启、装新版），wait 退 78、出路写「服务好了再提交」。子 agent 照出路再交一次，而原任务在它自己的单元里接着跑（服务停不杀任务）：同一条命令跑两份——内存翻倍记账，两份同时写同一份判定库、同一个 `research/results/` 文件名、同一个副本。只有 `.claude/main-agent.md:64`（41e4163e）写着「服务不通时在跑的任务在各自的单元里接着跑、不重投」，子 agent 不读 `main-agent.md`。
**推翻条件**：wait 起头连不上时的出路不含「再提交」；或某份子 agent 定义写了「wait 退 78 不重交，先 `task status <号>`」。
**四句**：分不分辨——打所有自己交任务的子 agent。看不看得到——wait 知道自己拿着号，原任务在不在要等服务回来才查得到；出路却不分「submit 没交上」与「wait 没等上」两种。满足的分句——「会读错任务结局」「被拒之后出路指不到合规写法」。改法——见改法表。

## A7　「服务对子 agent 交的重型当场拒」：服务分不出子 agent

**照定义原文做的那一步**：`.claude/agent-common.md:59`（8858e562）：「重型（「重型测试就是 checker 档那一批」那一条）子 agent 不交，写进报告「要主 agent 跑的命令」一节交回；服务对子 agent 交的重型当场拒。」

**对照**：
- `task-server/task_server_lib.py:157`（34dd46ae）「子 agent 号：harness 给子 agent 起的 shell 的环境与进程树上都读不到能分开子 agent 的标识」，随后写「不记」。`grep -n '子 agent\|subagent\|agent_id\|HEAVY_TESTS' task-server/task_server.py` 零命中（量的）：服务那一边没有按子 agent 拒的判法。
- `.claude/hooks/heavy-test-guard.sh:325`（cc9fd871）：「服务认不出提交的是不是子 agent（task-server/task_server_lib.py 文件头「身份」），这一判只在钩子里」。
- 钩子判不了时照常执行：`.claude/agent-common.md:90`「判不了的（hook 输入不是 JSON 对象、判的时候抛异常、读不到共用模块、读不到或看不全的脚本……）照常执行并记一条检出（只记）」，同一条又写「python / perl / ruby / tclsh 起的脚本文件不读」。

**后果**：定义把最后一道写成服务，实际只有钩子一道；钩子判不了（只记）或读不到的那一层（python 起的脚本里再起的重型），任务照样进队、照样跑，没有第二道接住。子 agent 照这一句会以为「交错了服务会拒」，`agent-common.md:62` 那条「里面有重型的，子 agent 不交」就成了唯一的自守。读的；不造绕过写法，没量。
**推翻条件**：服务端有按会话或 agent 拒重型的判法（上面 grep 命中）。
**四句**：分不分辨——是说法错位，不分 agent。看不看得到——服务看不到子 agent 身份，这是 `task_server_lib.py:157` 现查过的事实。满足的分句——「会漏判」。改法——把那半句改成「钩子对子 agent 交的重型当场拒（服务分不出子 agent）」，推的、被攻过零轮。

## A8　经 `submit` 交的 cargo 不认合入锁

**照定义原文做的那一步**：`.claude/agent-common.md:90`（8858e562）：「经 submit 交给服务的那一条不判内存包装（套了包装的由服务当场拒），合入锁与编译目录那两道也不判它（编译目录归服务）」。理由只交代了编译目录，没交代合入锁。

**对照**：`.claude/hooks/heavy-test-guard.sh:327`（cc9fd871）「合入锁与编译目录那两道（下面「再一道」）不判交给服务的那一条（编译目录归服务）」；服务三份文件里 `grep -c workspace_lock` 都是 0（量的：`task_server.py`、`task_server_lib.py`、`attempt_runner.py` 各 0）。合入锁原来的用处（同文件第 337 行起「再一道」）是：`merge-and-verify.sh` 与 `apply-writer-patch.py` 打补丁的时候，主工作区里别的 cargo 不起。

**后果**（推的）：今天所有 cargo 都经 `submit`，这道锁对谁都不再生效。主工作区里改的实现员、`查到就修` 的调查员交 `cargo test -p singlefs-harness --test …`（工作目录是仓根），与另一个会话的 `merge-and-verify.sh` 同时进行时，编的是打了一半补丁的树，红了按代码红去查；`.claude/agent-common.md:68` 自己写着「本机常有别的会话在同一个仓里干活」。换了编译目录只解决了 `target/` 的争用，解决不了源码树在打补丁途中被读。
**推翻条件**：服务或钩子在 `submit` 那一刻认合入锁（上面 grep 命中，或钩子第 327 行改了）。
**四句**：分不分辨——打所有在主工作区经服务起 cargo 的。看不看得到——锁文件在本机，钩子与服务都读得到，只是不读。满足的分句——「会漏判」。改法——见改法表。

## A9　子 agent 与主 agent 同一个会话号：「只许提交它的会话」管不住兄弟

**照定义原文做的那一步**：`.claude/agent-common.md:63`（8858e562）：「要停经服务交的任务，用 `task-server/task cancel <号>`（只许提交它的会话）」；各份定义都写「`task wait` 末行带 `build=` 的，用完 `task-server/task clean <号>` 删它的编译目录」。执行员要跨轮用编译目录：`.claude/agents/experiment-runner.md:38`（59d3f26f）「生成产物用的二进制取编它的那一次任务 `task wait` 末行 `build=` 目录下那一份」。

**对照**：`task-server/task_server_lib.py:155`–`158`（34dd46ae）身份取进程树上最外层的 `CLAUDE_CODE_SESSION_ID`，子 agent「不记」；`task-server/task_server.py:1896`（clean）与 `:1932`（cancel）只比 `task.get("session") != session`。我这条腿交的任务回显的会话号（量的：任务 1786–1791、1804 的 `TASK_SERVER submit` 行）与 harness 给本会话的临时目录名里的会话号相同，按上面的取法是进程树最外层那一个；会话号不进仓，此处不抄。它是不是主 agent 的那一个，我从子 agent 这一侧没直接核；`.claude/hooks/completion-cleanup-guard.sh:20`（20e550e3）写着「子 agent 认领的会话号与主 agent 相同，也归本会话」，与之相合。

**后果**（推的）：任何一个子 agent 写错号、或照 `task status`（不带号，列的是整个会话的任务，含兄弟与主 agent 的）挑号，`cancel` 与 `clean` 都会成功：停掉兄弟正在跑的产物、删掉执行员下一轮要执行的那份二进制所在的编译目录。被删的一方下一轮执行二进制得到「没有这个文件」、`command-failed`，看起来像自己的命令错了。定义把「只许提交它的会话」写成保护，子 agent 会当成「删错了服务会拒」。
**推翻条件**：服务能分开子 agent（`task_server_lib.py:157` 那一段改了），或 `cancel`、`clean` 改成比提交时记下的 agent 号。
**四句**：分不分辨——打所有同会话的 agent。看不看得到——服务看不到子 agent 身份（同 A7）。满足的分句——「会留下或误删编译目录」「会读错任务结局」。改法——定义里把「只许提交它的会话」改成「同一会话的主 agent 与各子 agent 都过得了，只动自己交回过号的任务」，推的、被攻过零轮。

## A10　写回员喂写范围闸：样本里没有工作目录，判据只认退出码 2

**照定义原文做的那一步**：`.claude/agents/kb-scribe.md:31`（da237b44）：每个路径写一份 `<草稿目录>/precheck-<n>.json`（内容 `{"tool_name":"Edit","agent_type":"kb-scribe","tool_input":{"file_path":"<路径>"}}`），交 `task-server/task submit -- env AGENT_HOOK_DETECTIONS=<草稿目录>/precheck.jsonl bash -c 'bash .claude/hooks/write-guard.sh < <草稿目录>/precheck-<n>.json'`，「有一条退出码 2 就整份不写、停下报告」。

**撞上的钩子那一行**：`.claude/hooks/write-guard.sh:1559`（f6259538）`project_root = os.environ.get("CLAUDE_PROJECT_DIR") or hook_input.get("cwd") or os.getcwd()`。服务起的任务里没有 `CLAUDE_PROJECT_DIR`（`CLAUDE_*` 是「会话自带、不带进去」那一种，`task_server_lib.py:150`），样本里又没有 `cwd`，仓根就取任务的工作目录。

**量的**：
- 任务 1786（`--cwd` 是我的草稿目录）：`name=w1-kb-scribe-precheck-in-scope hook=write-guard exit=0 expected=0`；`name=w2-kb-scribe-precheck-out-of-scope hook=write-guard exit=0 expected=2`——照写回员定义的样本形态写 `CLAUDE.md`（范围外），放行。
- 任务 1804（同一个工作目录，`out/write-guard-retry-task-1804.txt`）：样本加上 `"cwd": "/home/fy5090/code/singlefs"` 的 `name=w2b exit=2 expected=2`；路径写成绝对路径、不加 `cwd` 的 `name=w2c exit=2 expected=2`，但拒的理由是「kb-scribe 没有项目定义（.claude/agents/kb-scribe.md 不在…）」——仓根被当成草稿目录，判的路子已经错了；原样再喂 w2 `name=w2-again-cwd-scratch exit=0 expected=2`。
- 任务 1786 `name=w3-precheck-input-missing hook=write-guard exit=1 expected=not-2`：输入文件读不到时 bash 退 1（`bash: line 1: /nonexistent-d137-precheck.json: No such file or directory`），钩子一行没跑。

**后果**：
1. 写回员照定义原样从仓根交、任务也在本机跑时，工作目录是仓根，判得对（推的：与 w2b 同一个仓根）。但只要工作目录不是仓根——带了 `--cwd`（三方腿的定义、这一题正文「五、交付要求」都叫喂钩子时 `--cwd <自己的草稿目录>`）——范围外的路径放行，整份照写。
2. 这条命令的工作目录是仓根、参数里没有此刻在的绝对路径整词（`precheck-<n>.json` 藏在 `bash -c` 的字符串里，`AGENT_HOOK_DETECTIONS=…` 那一词从第二份起文件已在、会被送过去），按 A1 那几行可以去第二台；第二台没有 `<草稿目录>/precheck-<n>.json`，bash 退 1（w3 的形态），钩子没跑。
3. 定义只说「退出码 2 就停」，退 1、退 78、`type=` 不是 `succeeded` 的都没说怎么办，照字面就是「没有退 2，照写」。
**推翻条件**：写回员定义的样本形态在 `--cwd` 不是仓根时喂出退 2；或定义写明「`type=succeeded exit=0` 才算过」。
**四句**：分不分辨——只打写回员的预检这一步，真写的时候写闸在会话里照判（会话里有 `cwd`），所以真越界的写仍会在写的那一刻被拒，打中的是「预检报过而真写被拒」的那种中途停摆与预检本身失效。看不看得到——钩子看得到样本里没有 `cwd`，退回取进程工作目录，没说。满足的分句——「会漏判」「会读错任务结局」。改法——见改法表。

## A11　分片任务的 `task log <号>` 给的不是没成的那一片

**照定义原文做的那一步**：`.claude/skills/gate/staged-gate.md:15`（fc5a89f5）：「整份输出 `task-server/task log <号> --all` 写进草稿目录」；各份子 agent 定义都写「读 `task-server/task log <号>` 接着做」。分片的入口登记在 `task-server/task_server_lib.py:511` 起的 `SHARDED_ENTRIES`（`crash-point-replay-shard-run.sh --merged-log`、`mutation-shard-run.sh`、`crash-amplification-two-hosts.sh`、E171 的 `rows_orchestrator.py run`），`.claude/rules/implementation-workflow.md:66` 叫主 agent 交其中两份。

**客户端那几行**：`task-server/task:367`（f351d875）`attempts = [attempt for _, attempt in every_attempt(reply)] if code == 200 else []   # 分片的父任务：各子任务的尝试，按子任务号排`，第 371 行 `path = chosen[-1].get("log", "")`——取号最大的那个子任务的最后一次尝试。`task wait` 的末行另有一段（第 145–146 行）专门换成没成的那一步（`failed.get("task")`）的日志，`task log` 没有这一段。

**后果**（推的，没交分片任务）：一片没成时合并不起，号最大的是排在后面、被记 `cancelled` 而一次没起的兄弟（没有尝试，不在列表里）或在跑、跑完的另一片；`task log <父号>` 打出的是那一片，`--all` 也只是那一片的整份，不是「整份输出」。主 agent 照 skill 把它存成这道门禁的输出，读到的是别的片的日志。`--attempt <n>` 在父任务上按各子任务各自的次数号筛，几片都有第 1 次时取最后一片。
**推翻条件**：`task log <父号>` 打出没成的那一片（或全部子任务）的日志。
**四句**：分不分辨——只打分片的父任务。看得到——客户端手里有 `failed_step`，`wait` 用了、`log` 没用。分句——「会读错任务结局」。改法——定义写「分片的读 `task wait` 末行的 `log=` 与 `shard=`」，推的、被攻过零轮。

## A12　服务不通时客户端的出路，子 agent 起不了

`task-server/task_server_lib.py:1242`（34dd46ae）不通时的出路：`f"看服务在不在：systemctl --user status {UNIT_NAME}；没起就 {START_COMMAND}；没装就 {INSTALL_COMMAND}；"`，第 494 行 `START_COMMAND = "python3 task-server/control start"`。`.claude/agent-common.md:90` 把「`task-server/control` 的 `install` / `start` / `stop` / `watch`」列在只给主 agent 的名单里，子 agent 起它被重型测试闸拒。被判的 13 份定义里没有一句写「`submit` 或 `wait` 退 78 时子 agent 怎么办」（量的：`grep -c 78` 在共用约束、`main-agent.md` 与 11 份被判定义里全是 0）。第 74 行此刻写「子 agent 改不成写法、或判出是机制问题的，写进报告交主 agent」，接得住，所以这一格只是出路的第一步指错（读的）。

## A13　「副本只拷要编的」与「轻型在副本里交」

`.claude/agent-common.md:46`（8858e562）：「副本只拷要编的（`crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default`，要门禁表的再加 `.claude/gate.d`）」。照这一句建的副本里没有 `scripts/`，`.claude/rules/verification.md:50` 要在副本里交的 `bash scripts/crash/crash-point-replay.sh --quick` 在副本里找不到脚本，`command-failed`（推的；`bash` 起不在的脚本退 127）；形式化与变异两道在 `.claude/gate.d/` 下，可它们 source 的 `scripts/` 下的库同样不在（推的，没逐份查）。实现员定义第 36 行另写「`rsync -a --exclude target --exclude .git`」整仓拷，两句不一致；照整仓拷的那一种走到 A2。

## 改法表（全部只在我的推理上、被攻过零轮）

| 格 | 改法 | 修哪一格 | 状态 |
|---|---|---|---|
| A1 | 定义里写一句：要往仓里或草稿目录写文件的命令，`--cwd` 给草稿目录（工作目录不是仓根就只在本机）、或输出路径写成此刻不在的绝对路径整词；共用约束「子 agent 不交重型」那一条补「第二台写的文件不回本机」 | A1 的产物、`.rc`、复跑输出 | 推的；放置规则量过（1787–1791 在第二台），改了之后留不留本机没量 |
| A1 | 服务那一侧：`bash -c` 字符串与相对路径参数也当「可能要写」，钉本机 | 同上 | 推的 |
| A2 | 钩子：副本里与主仓逐字节相同的 `crash-point-replay.sh` 带 `--quick` 照轻型认（与 `--selftest` 在副本里放行的三条件同一套）；或三份定义改成「崩溃点重放轻型在仓根交，测的是主工作区」 | A2 | 推的；h1 退 2 是量的 |
| A3 | 执行员第 42、59 行与工具员第 35 行改回「命名检查除外」 | A3 | 推的 |
| A4 | 实现员第 39 行的 clippy 不写 `--all-features`（写点名的非 gpu 特性），gpu 代码的 clippy 单列；或服务的 `cargo_uses_gpu` 只认 test / run / bench | A4 | 推的 |
| A5 | 第 63 行改：第二条带 `CARGO_TARGET_DIR=<第一条的 build= 目录>`，第二条完了再 `task clean <第一条的号>`；或服务把 `--no-run` 记成不要卡 | A5 | 推的 |
| A6 | wait 起头连不上时出路改「先 `task-server/task status <号>`，在跑就再起 wait，不重交」；子 agent 定义补同一句 | A6 | 推的 |
| A7、A9 | 措辞：「钩子拒」「同一会话的 agent 都过得了」 | A7、A9 | 推的 |
| A8 | 钩子或服务在 `submit` 那一刻、工作目录落在主工作树时认合入锁 | A8 | 推的 |
| A10 | 写回员样本加 `"cwd": "<仓根>"`；判据改成「`type=succeeded exit=0` 算过，`exit=2` 停下，别的一律停下报告」 | A10 | w2b 退 2 是量的；判据那半是推的 |
| A11 | 定义写「分片任务读 `task wait` 末行的 `log=`、`shard=`」 | A11 | 推的 |
| A12 | 子 agent 定义补「`submit`、`wait` 退 78：不起 `control`，写进进度记录交主 agent」 | A12 | 推的 |
| A13 | 第 46 行与实现员第 36 行取一种，轻型要在副本里起的把 `scripts/` 列进去 | A13 | 推的 |

## 没打中的形状

- 样本 h3（三方腿照第 3b、8 步写的 `rerun.sh`：`cargo build --manifest-path "$WORK/…"` 再直接执行 `"$WORK/…/crash-probe"`，路径全经变量）：`exit=0`，钩子没拒。h3b（钩子路径经 `$REPO_ROOT` 引）：`exit=0`。我自己的 `tools/rerun.sh` 经 `$MODEL` 起 `feed.sh`，任务 1786 照常起、跑完退 0。模板「路径只用四个变量」与共用约束第 62 行「每一条写字面路径」字面上不一致，但这三种形态都没撞拒，没打中后果。
- 样本 h4（写回员定义第 31 行那条预检命令原样）：`exit=0`，重型测试闸放行；问题在 A10 的另一层。
- 样本 h5（实现员把 clippy 与 build 合成一条 `bash -c` 交）：`exit=0`；问题在服务那一侧（A4）。
- 对照 h0（仓根交崩溃点重放轻型）、h2（副本里交形式化轻型）、w1（范围内路径）：都与定义一致。
- 读了、没打中：重型测试闸照 `--cwd` 换目录解脚本路径（`.claude/hooks/lib_heavy_tests.py:2910` 一带）；交回闸认 run_in_background 的输出文件里的 `TASK_SERVER_RESULT`（`.claude/hooks/handback-guard.sh:13`）；完工清理闸对主 agent 按整个会话列服务编译目录（`.claude/hooks/completion-cleanup-guard.sh:25`–`26`），中途停掉的腿留下的编译目录主 agent 收工时会被拦下；`wait` 对已取过结局的任务照样回结局（`task-server/task_server.py:2155` 起只在同会话时记已取，不挡第二次）；带判定库的入口经 `--verdict-library-route` 钉本机，门禁产物不受 A1 影响（`task-server/task_server.py:1281` 起）。
- 取样范围：钩子样本 10 条（任务 1786）加写闸复喂 3 条（任务 1804），放置探针 5 条（1787–1791）。放置只在一个时段、本机满载时量过，本机空闲时没量。

## 要主 agent 跑的命令

1. 复跑模型（非重型）。在仓根起，先重建副本：`mkdir -p /home/fy5090/.local/share/singlefs/work/d137-ts-defs-r1-attack/copy && rsync -a --exclude target --exclude .git scripts .claude crates Cargo.toml Cargo.lock .cargo task-server /home/fy5090/.local/share/singlefs/work/d137-ts-defs-r1-attack/copy/`；再 `task-server/task submit --cwd <主 agent 的草稿目录> -- bash <模型目录拷贝>/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/d137-ts-defs-r1-attack/copy --work <新的空输出目录>`（新 shell 里起）。看：末行 `type=succeeded exit=0`；`task-server/task log <号> | grep '^name='` 与 `out/feed-task-1786/summary.txt` 逐行同（h1 `exit=2`、w2 `exit=0` 是打中的两行）。样本 JSON 里写死了仓根与上面这个副本路径，副本放在别处要改样本。
2. 写闸复喂（非重型）：在仓根起 `task-server/task submit --cwd <主 agent 的草稿目录> -- bash -c '<任务 1804 的命令原样，见 out/write-guard-retry-task-1804.txt 上面那三条>'`；任务 1804 的整条命令可用 `task-server/task status 1804` 看。看：`name=w2b exit=2`、`name=w2c exit=2`、`name=w2-again-cwd-scratch exit=0`。
3. A1 放置（非重型，结果随负载变）：在仓根起 `task-server/task submit -- bash -c 'echo probe > research/prompts/d137-ts-defs-r1-opus-model/out/peer-probe-main.txt'`。看：`task-server/task status <号>` 的「第 1 次：」是哪台；在第二台的，本机 `out/` 下没有那个文件而结局 `succeeded`。本机空闲时多半留本机，那一次不算推翻。
4. A4 推翻条件（起 cargo，这一题不许腿起；只看提交回显，不必跑完）：在仓根起 `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=clippy; 范围=1 个测试目标; 为什么=核实现员定义第 39 行的 clippy 会不会被服务记成要卡' task-server/task submit -- cargo clippy --all-targets --all-features -- -D warnings`。看：`TASK_SERVER submit` 那一行 `gpu=1` 即坐实；随后 `task-server/task cancel <号>`。
5. A5 推翻条件（同上，只看回显）：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-checker-tier 只编; 范围=1 个测试目标; 为什么=核 --no-run 那一条会不会被记成要卡' task-server/task submit -- cargo test --release -p singlefs-checker-tier --features gpu --no-run`。这一条点名 checker 档包，按 `classify` 是重型，标「重型」，主 agent 定要不要交；看回显 `gpu=`，随后 `cancel`。

## 这条腿自己的限度

- A3、A4、A5、A6、A7、A8、A9、A11、A12、A13 是读代码与定义推的，没量；A4、A5 要起 cargo、碰 GPU 队列，这一题不许腿做。
- A1 的放置只在本机 slice 满载的一个时段量过 5 条；本机有余量时的放置没量。
- A3 没喂交回闸：它要会话记录，我没造。
- 样本 h1、h2 的副本我交回前删了，复跑要照「要主 agent 跑的命令」第 1 条重建。
- 只攻 A 格；正推腿那一格（逐份对照）没做，也没读正推腿的报告。

## 没做什么

- 没判正推那一格，没替主 agent 采纳；没提交；没跑重型、没起 cargo、没碰 GPU；判决要引的数没复跑（命令在上一节）。
- 没改任何被判文件；改法都是推的、被攻过零轮。

## 交回前的核对

- `python3 scripts/agents/cite-check.py research/prompts/d137-ts-defs-r1-opus-output.md` 末行（量的）：`  ✓ 核了 2 处引文、58 处裸行号，对上 2 处，没判 58 处（其中裸行号没有引文可比 56 处；逐处列在上面）`；`✗` 行 0 条。正文里的引文原话都在被引文件里用 `grep -nF` / `grep -n -o` 现查过，行号取命中的那一行。
- 草稿目录里的仓副本 `/home/fy5090/.local/share/singlefs/work/d137-ts-defs-r1-attack/copy` 删前 `du -sh` 190M，已删。草稿目录里留着 `rerun-out-1/`（任务 1786 的原样输出，已拷进模型 `out/feed-task-1786/`）、`w2-retry.jsonl`、`cite.txt` 与两份生成样本的小脚本，不含副本与编译目录。我交的任务都没有会编的 cargo，没有 `build=` 目录要 `clean`。
