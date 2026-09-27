# 项目 subagent 的共用约束

`.claude/agents/` 下每个定义开工前先读这一份；与定义冲突时以定义为准。
这份与各份定义只写怎么做：步骤、约束、判据、交付。「为什么」「实测」「经过」与带日期的记录写进 `records/2026-09-16-subagent拆分提案.md` 或 `.claude/kb/`，这里不写、也不留解释的尾巴；要用到那些内容时写成一条读取指令（「开工先读：`文件`「小节」」）。上游门禁的「规则纪律（项目本地）」阶段判这一条（规则：`.claude/rules/implementation-workflow.md`「改 agent 定义与共用约束，走同一条三步」）。

## 派发

- 只由主 agent 点名派发；你手里没有 Agent 工具，不派 subagent，也不从 Bash 里起 `claude` 会话。
- 轮名、产出文件路径、草稿目录、这一轮的禁读清单都由主 agent 在派发提示里给。定义里写的路径只是形态（`<轮>`），不是实际路径。
- 输入缺一样就不开工，回复只写缺什么。
- **所有派发的subagent的effort不超过high**。如果需要超过那么弹窗告知。

## 规则怎么读

`.claude/agents/` 下的定义都开了 `omitClaudeMd`：项目 CLAUDE.md、它 `@` 的规则、用户级 CLAUDE.md 与主 agent 的私有记忆都不进你的上下文。

- 定义「开工先读：」一行点名的规则，只读点名的那几个小节：先 `grep -n '^#' 文件` 找到小节的起止行，再按行读。点的是整份文件的，先看它的节标题，挑与这件活有关的节读。
  不整份读。
- 每个定义都照守、不再写进各自「开工先读：」一行的三处：跑命令照 `.claude/singlefs-ai-sop/rules/command-safety.md`「退不回去的操作，动手前先想一遍」「`pkill -f` / `killall` 一律禁用」两节；
  给人看的文字照 `.claude/singlefs-ai-sop/rules/writing-discipline.md`「说人话」一节；说外部状态之前现查（`.claude/singlefs-ai-sop/rules/verify-before-claiming.md` 开头一节）。
- 本机时钟是 UTC，人在东京（JST，UTC+9）；报告里的时刻写清是哪个时区。
- 候选、臂、方案、判据、提问编号有了变体，起一个新名字（那一族里下一个没用过的号，或一个短的描述性名字），不在原名后面加撇号类角标（U+2032、U+2033、U+2034、U+02B9、U+02BA）；全仓由门禁 12 号判，写法见 `.claude/rules/path-moves.md`「变体起新名字，不用角标」。
- 派发提示里没给、定义里也没写的项目事实（某份 kb 在哪、某条决策的原文），去仓里现查，不凭印象补。
- **找不到历史实验的数据、提示或产物，去 `git log` 里看。** 上一轮及更早的实验记录不留在工作区：
  这一轮提交之后由下一次提交删掉上一次那批，本轮的留着（`.claude/gate.d/91-archive-past-rounds.sh` 判这一条）。
  仓里因此有一批引用只写文件名、不写路径，那不是坏链接，是已经归档的东西。
  查法：`git log --all --diff-filter=D --name-only -- "*<文件名>*"` 找到删它的那次提交，
  `git show <提交>^:<路径>` 读当时的内容。**读到的是当时的数，不是今天的结论**——
  拿它支撑新结论之前先重新跑一遍（`.claude/singlefs-ai-sop/rules/kb-discipline.md`「2. 每条带出处与状态」一节里「所有旧数据都只是参考」那一条）；
  要推翻早先的结论，按 `.claude/rules/three-way-inference.md` 重走一轮，不是拿旧文件对质。

## 写

- 只写派发提示与定义「写范围」一节给的路径。写范围闸（项目 settings 里的 hook，表在 `.claude/hooks/agent-write-scope.tsv`）只看 Write / Edit：目标不在表里你那几行模式内的，当场拒掉；被拒之后不换 Bash 或别的办法写过去，写进报告交回主 agent。Bash 里的写闸看不见，全靠你守。
- tools 里有 Write / Edit 的：手写的改动一律用 Edit（旧串必须在文件里恰好命中一次，与 `replace-once.py` 同义；这一轮自己新建的文件可以 `replace_all`），新建文件用 Write（先 `ls` 确认不存在），让闸看得见；Bash 里只许定义点名的脚本自己写（`relabel-item.py`、`kb-scribe` 的 `replace-batch.py`、门禁阶段的 `--write`、`mutate.sh`、`replay.sh`、cargo、跑产物的重定向），不用 python / sed 就地改仓内文件（执行前被拒，见「执行前拒绝的写法」⑧）。草稿目录里（仓副本、日志、自己的辅助脚本）用什么写都行，那里只归你。
- tools 只有 Read / Bash 的：新建文件一律排他，`set -o noclobber` 后 `cat > 文件 <<'EOF'`，文件已存在就停下报告；之后 `>>` 追加。
- 报告分段写，每一次写进文件的内容不超过 150 行（`.claude/rules/three-way-inference.md`「云端腿的报告要分段落盘」）；表格与代码块整块放进同一段，不从中间切。
- tools 里没有 Write、Edit 的，改已有文件只用定点替换：`research/scripts/replace-once.py` 或 `research/scripts/replace-batch.py`（先 `--dry-run`），不整份重写。
- 草稿放派发提示给的草稿目录，不放会话共用的暂存目录；用不上可以空着。

## 不做

- 不做 git 写操作（`add`、`commit`、`stash`、`checkout`、`restore`、`reset`、`worktree`……），不提交，不推送。门禁自带的 git 写只有 `gate-triage` 例外，见它的定义。
- 不建、不改 `.claude/agents/`、`.claude/settings*.json`、`.claude/hooks/`、`.claude/rules/`、`.claude/singlefs-ai-sop/`，也不碰 `~/.claude/`。`tooling-writer` 例外：按它定义「写范围」一节改 `.claude/agents/`、`.claude/hooks/`、`.claude/rules/` 与 `.claude/settings.json` 的 `hooks` 一节；`.claude/singlefs-ai-sop/` 与 `~/.claude/` 它同样不碰。
- 不读派发提示给的禁读清单里的文件。
- 不编译 Rust、不跑 `gate.sh` 全量，除非定义明写要做。跑脚本、跑模型一律加 `nice -n 19`。
- 重型测试（名字含 `layer0` 的测试二进制与 54 号、QEMU、herd7、crates 变异整表、全量 `cargo test`（`--all` / `--workspace` / 工作区根裸跑）与 `check.sh`、`gate.sh` 整轮、87 号全部实验复跑、E152 装置）只在提交代码时跑一次、或用户要求时跑；子 agent 一律不跑，只跑自己动到的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）、fmt / clippy / build 与 `.claude/gate.d/` 下 54、55、57、59、87 之外的阶段。`crash-verifier` 只跑 54、55、57、59 号那几道，`gate-triage` 只跑 `gate.sh` 整轮与 87 号，都在提交时或用户要求时跑、命令带 `SINGLEFS_HEAVY_TESTS=commit`（用户要求时 `=user-request`）。项目 settings 里的 `.claude/hooks/heavy-test-guard.sh` 在执行前拒绝越出这些的命令；被拒了不换写法绕过去，在交回里写明要跑什么、为什么，交主 agent 去问用户。
- 写进脚本文件、放进 `python3` 的 subprocess、`make` 这类间接起法同样算跑：自己写的验证链里不许出现重型测试的任何一步，名字含 `layer0` 的测试二进制也不许。
- 跑编译出来的代码经内存包装：`cargo test` / `run` / `bench`（带 `--no-run` 只编不跑的不算）与直接执行 cargo 编出来的二进制（实验装置、测试二进制），一律经 `bash research/scripts/run-with-memory-cap.sh <上限> <命令…>` 跑；写进自己的脚本再跑的，整条脚本经它跑（`bash research/scripts/run-with-memory-cap.sh <上限> bash <脚本>`）。上限取派发提示给的（定义另写了来源的照定义），都没给就取 `research/scripts/replay.sh` 文件头 `REPLAY_MEMORY_CAP` 的默认值。`research/scripts/mutate.sh`、`research/scripts/replay.sh` 与门禁 59 号在里面逐条套了，门禁 15、74 号在阶段里面经包装（派发提示给了上限的，设进阶段文件头写的变量：15 号 `GATE_RESEARCH_BUILD_MEMORY_MAX`，74 号 `GATE_MODEL_DIFFERENTIAL_MEMORY_MAX`），都直接跑，外面不再包一层。退出码 250–254 是包装自己的结局（含义见 `research/scripts/run-with-memory-cap.sh` 文件头「退出码」一段）：那一次的输出不算结果，命令与退出码写进报告，不绕开包装重跑。`cargo build`、`cargo clippy`、`cargo fmt` 不跑编出来的代码，不要求经它。
- 要停自己起的一条链（脚本、后台任务）时，先列出它的整棵子进程（`ps -o pid,ppid,args --ppid <pid>` 逐层往下），从最底层起逐个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`（`proc.py` 只停给定的那一个，不带子进程），停完再列一遍，确认一个不剩；不许留下正在跑的那一步「让它跑完」。
- 要编译、跑门禁阶段、跑变异或产物的，开跑前 `ps -o pid,args -u "$(id -u)"` 看负载：有性能测量在跑（`qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`）就报「在等 pid …（命令）」停下；只有别的 `cargo`、`gate.sh` 在跑的，加 `nice -n 19` 照常跑（cargo 自己排队拿文件锁），报告里记下 `ps` 看到了什么、等锁等了多久。定义另有更严要求的照定义。
- 读大文件（超过 500 行）先 grep 定位、再按行段读（Read 的 offset / limit），不整份读；同一份文件读过一次、之后没改过，就不再整份重读。长命令的输出落进草稿目录的文件，只读汇总行（`tail -n 20`、`grep -c`、`grep ✗`），不把整份日志、整份 diff 读进上下文。上下文到 600k 就停在一个能交接的点交回（定义另写了线的照定义）：报告写做完的、做到一半的（文件与差哪一步）、没开的。
- 派发提示里有「线程上限：N」的，编译、测试、变异、产物、原型扫描的每条命令都用 `bash research/scripts/capped.sh N <命令>` 起（它把 cargo 与 `crates/singlefs-harness` 各装置读的线程变量一次设成 N）；实验装置自己的线程变量（`KS_THREADS` 这类）同样设成不超过 N。主 agent 发消息改了 N，从下一条命令起照新的。
- **崩溃点测试不衡量时间成本，也不为省时间缩范围**：要加崩溃点、要多枚举一档，就加。挂钟长不是收窄它的理由，也不用先算它值不值。
- 禁止自行扩大任务范围，只改自己的部分。一个任务一个出口，达成出口即终止任务，扩大任务必须弹窗。禁止因为产物变化无限继续任务。尤其是门禁检测等任务，不属于自己任务的验证，如果验证为红给其他任务发消息，严禁自行修改扩大任务范围。
- 本机常有别的会话在同一个仓里干活：只动这一轮自己的文件；看到别人没提交的改动不碰、不修。
- 长活可以等，不给它设超时、不自己中途杀掉：编译、变异表、复跑这类长活用 Bash 的 `run_in_background` 起，起完结束本轮，完成时会通知你。交给它的命令要一直跑到真正的活结束才退出：不在里面再把活放到后台——不用 `disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux`/`screen` 的分离模式、`systemd-run`，子 shell 或 `$( … )` 里也不写 `&`；`&` 只在同一条命令随后用不带参数的 `wait` 等齐时用，每件的退出码照「执行前拒绝的写法」那一条 ④ 的写法收。结束本轮就是这一条回复只写一句在等什么、不再调工具；前台命令的 `timeout` 不超过 240000 毫秒。等自己起的后台任务一律结束本轮等完成通知，不写轮询它输出文件的循环（run_in_background 里也拒，见「执行前拒绝的写法」②）。
  等长活时不起缓存计时器，结束本轮直接等完成通知。
  等的时候看到进程不动、报错、或等的条件不会成立，把看到的命令、输出与时刻写进草稿目录里的进度记录，接着做定义里还能做的；结不结束、怎么处置由主 agent 定。主 agent 的看门狗（`research/scripts/agent-watch.py`）定时复检你；前台没有超时的等待循环、run_in_background 里轮询本会话后台任务输出文件的循环、把活放出追踪、run_in_background 里后面没有 `wait` 的单独 `&` 在执行前被拒（「执行前拒绝的写法」那一条的 ②③④），run_in_background 里别的等待循环只记检出，交给主 agent 判断。跑超过 30 分钟的量，每完成一格往草稿目录的 `progress.md` 追加一行（格名、耗时、下一格预计多久），看门狗的整点询问会附上它的末几行。等日志里的一行字之前，先确认那一行真会写进那个文件；找进程用写死的 pid，不用 `pgrep -f` / `pkill -f`（command-safety.md）。
- 执行前拒绝的写法：项目 settings 里注册的这几道 hook 在执行前（收工闸在收工前）拒绝这几类写法（退出 2，stderr 写原因与出路），主 agent 与子 agent 一样拒，只挂在一方才用的工具上的在条目里写明；拒的只是这一次调用，不停已经在跑的东西。被拒了照出路改写法，不换写法绕过去；改不成的写进报告交主 agent。
  - Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`）：② 只在前台拒（run_in_background 里的等待循环只记检出、不拒），④ 只在 run_in_background 里拒，其余前台与 run_in_background 一样拒：
    ① 起看门狗的错误写法：`research/scripts/watch.sh`、`research/scripts/agent-watch.py watch` 不用 run_in_background 起，或命令里带单独的 `&`、`nohup`、`disown`、`setsid`、把输出丢进 `/dev/null`。看门狗只有主 agent 起。
    ② 前台没有超时的等待循环：`until` / `while` 里有 `sleep`，外面没套 `timeout`；run_in_background 里条件或循环体点名了本会话后台任务输出文件（`tasks/<id>.output`）的等待循环，有没有 timeout 都拒。要等就用 run_in_background 起、结束本轮等完成通知，或 `python3 .claude/singlefs-ai-sop/scripts/proc.py wait <pid> --timeout <秒>`。
    ③ 把活放出追踪：`disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux` / `screen` 的分离模式、不等到结束的 `systemd-run`。
    ④ run_in_background 里有作业以单独的 `&` 收尾，之后同一条命令里没有 `wait`。要并行就每件把退出码写进自己的文件（`{ <命令>; echo "$?" > <文件>; } &`，文件名带批号与件号，每批开跑前先删掉这一批的文件），最后单独一个不带参数的 `wait` 等齐，再按派活的次序逐个读那几个文件；读之前数一遍，文件数与派出去的件数对不上，整批作废；不用 `wait "$pid"` 收。
    ⑤ 整份覆盖 `research/results/` 下已存在又没进 git 的产物：`>`、`>|`、`&>`、不带 `-a` 的 `tee`、`cp` / `mv` / `install` 的目标、`dd of=`、`truncate`。追加（`>>`、`tee -a`）与新文件名不拦；要换就按日期另存新文件名，旧的留着。
    ⑥ 终止进程不是点名一个自己起的进程号或任务号：`kill` 的目标带负号、是 0 或 `$PPID`、一次给几个、是命令替换或通配，在循环里逐个发（`proc.py stop` 同样），按名字、按 cgroup 或 `/proc` 挑进程，写死的进程号指到自己会话的祖先、别的会话起的进程或 SSH / VSCode / 本地模型服务，`systemctl` 停 SSH、登录会话、本地模型服务这类单元，关机重启。放行的是点名一个：`kill "$!"`、`kill %1`、单独一条 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`。
    ⑧ 有 Edit 工具的项目子 agent 在 Bash 里就地改仓内文件：命令位置上的 `sed -i`（目标不在 `/tmp/` 下），python 代码里 `open(<仓内路径>, 'w')` 这一类、`Path(<仓内路径>).write_text(`。仓内文件用 Edit 改，草稿目录照写。
    ⑦ 在同一个 inode 上改已经存在的脚本（`.sh`、`.py` 或带执行位的文件，在仓里或 `/tmp/claude-1000/` 下）：`>`、`>|`、`&>`、不带 `-a` 的 `tee`、`cp` 的目标、`dd of=`、`truncate`、python 的 `open(…, 'w')` 这一类。改脚本写到同目录临时文件再 `mv` 换上，或用 `research/scripts/replace-once.py` / `insert-row.py` 定点改；追加与新建不拦。
  - 上游的 `.claude/singlefs-ai-sop/scripts/claude-hooks/pattern-process-guard.sh`：命令位置上的 `pgrep -f`、`pkill -f`、`killall`。
  - 重型测试闸（`.claude/hooks/heavy-test-guard.sh`）：越出重型测试那一条的命令（主 agent 不带 `SINGLEFS_HEAVY_TESTS` 前缀也拒）；子 agent 不经内存包装跑编译出来的代码（「跑编译出来的代码经内存包装」那一条）。
  - 写闸（`.claude/hooks/write-guard.sh`，只看 Write / Edit 工具）：Write 整份覆盖仓里已存在又没进 git 的文件；项目子 agent 写到写范围表（`.claude/hooks/agent-write-scope.tsv`）里自己那几行之外；写进的内容里有撇号类角标（字符与起名的写法见 `.claude/rules/path-moves.md`「变体起新名字，不用角标」）。
  - 交回闸（上游的 `.claude/singlefs-ai-sop/scripts/claude-hooks/handback-scratch-check.sh`，挂交回工具 SubagentHandback 与 SubagentStop，只判子 agent）：临时目录里自己建的编译目录、工作树与仓副本还在，交回报告里又没逐个写全路径与为什么不删（`.claude/singlefs-ai-sop/rules/session-wrapup.md`「5. 子 agent 交回之前，删掉自己建的编译目录与仓副本」）。
  - 收工闸（上游的 `.claude/singlefs-ai-sop/scripts/claude-hooks/gate-reuse-check.sh`，挂 Stop 与 SubagentStop）：这个会话新建或改过的门禁与钩子没写 `# gate-similar:` / `# hook-events:`、该点名的已有门禁与钩子没点全、或整段抄了已有的一份（`.claude/singlefs-ai-sop/rules/sop-first.md`「加门禁或钩子之前，先找已有的」）。
  - 弹窗闸（`.claude/hooks/ask-user-claim-guard.sh`，挂 AskUserQuestion，主 agent 用）：问句或选项说明里一句话带断言词（不可能、造不出、从来不、从来没有、一定、必然、永远不、绝不会、恒为），同一句里没有出处（反引号里的路径或命令、文件:行号、`research/results/` 下的文件、`name=` 开头的产物行、「实测」「量过」「产物」「输出」旁边带数或路径），也没写「推的」「没量过」「推测」「估计」「粗估」之一。
  - 派发闸（`.claude/hooks/runner-dispatch-guard.sh`，挂 Agent / Task，主 agent 用）：派 crash-verifier、gate-triage 之外的类型，提示里没有一行「重型测试：不跑」；不读共用约束的类型没写「开工先读 `.claude/agent-common.md`」；缺定义 frontmatter `required-inputs:` 要的输入；提示要子 agent 写进它写范围之外的路径；派 `implementation-writer` 没写「要动的 crates 文件：…」、超过 8 个、或与在跑的实现员撞文件；在跑的 opus 子 agent 满 8 个、或同一族还在限额窗口里；派 `kb-scribe` 的规格过不了 `research/scripts/kb-spec-check.py`；派 `experiment-designer` 写准入判输入没变的重跑登记；派 `mutation-triage` 没给变异表；派 `experiment-runner` 没写「这一段回答的岔路：…」（只修锚点、只补落产物除外），续做没写「上一段岔路表里还差：…」或写了一行都不差；派 `kb-scribe`、`implementation-writer` 要改的文件落在还没写判决的三方轮的开工快照里。
  - 续做闸（`.claude/hooks/continuation-guard.sh`，挂 SendMessage，主 agent 用）：给已经交回过（它自己的会话记录，或主会话记录里的交回消息）、或最近一次任务通知是 failed / killed 的子 agent 发消息。
  - 交回正文闸（`.claude/hooks/handback-guard.sh`，挂交回工具 SubagentHandback，只判项目子 agent）：交回正文超过 2000 字；三方腿与核查员交回里点名的报告过不了 `research/scripts/cite-check.py`；书记员与执行员的交回或报告里有没报数、或报 0 项的绿行。

## 门禁

- 哪个门禁阶段该由谁在干完自己的活之后先跑，登记在 `.claude/gate.d/stage-owners.tsv`（第二列是 agent 名，逗号分隔）。列出登记给你的：
  `awk -F'\t' -v me=<你的名字> '$1 !~ /^#/ && NF == 3 { n = split($2, owners, ","); for (i = 1; i <= n; i++) if (owners[i] == me) print $1 }' .claude/gate.d/stage-owners.tsv`
  逐个 `nice -n 19 bash .claude/gate.d/<文件>`，贴每个的原样末行与退出码。
- 退出码 77 是「本次未跑」，不是通过。贴绿行时连它报的「查了多少项」一起贴；报「查了 0 项」或根本没报数的，按没判写。红了先看它点名的文件与行在不在这一轮的改动里；不在的不修，照写。
- 提交前的整轮门禁归 `gate-triage`，不归你；表里没登记给你的阶段不用跑。

## 报告

- 交回与报告在不影响正确性的前提下写简练：写结论、依据与没做什么，不写做事的经过、不复述派发提示与定义；下面几条要求的命令与原样输出、行号、推翻条件照样给，嫌长就放进报告文件、交回里指过去。
- 引规则、kb、脚本、代码，写那份文件自己的行号，行号去原文件里现查，不从背景材料里数；没现查过的行号不写进报告，先写「行号待查」；行号用 `grep -n` 或 `awk 'NR==行号'` 现取，不从 `sed -n 'A,Bp'` 的输出里数偏移；贴的命令输出必须是这一次真跑出来的原样；引原文整行抄，不许摘句。
- 能用命令核的事实，贴命令与原样输出；输出不截断，嫌长先数。报告里的条数、阶段数、命中数用命令数出来，不手数。
- 结论写「什么现象会推翻它」。
- 报告末尾有「没做什么」一节：跑不动的、没验的、按定义不归你的，照实列。
- 跑出来的产物先落盘进仓里该在的位置（实验产物与重跑日志进 `research/results/`，报告、提示与判决进 `research/prompts/`），再往下做。停机条款没过、按定义不写实验页、或者主 agent 还没定要不要留的，也要把草稿产物连同「为什么没入库」写进报告——它在哪个 `/tmp` 路径、跑了什么、为什么现在不入库，主 agent 才知道它在哪、还来不来得及拷。门禁 69 号判这一条的形式：装置或变异表改了而 `research/results/` 里没有一份不比它旧的产物、kb 与这一轮新写的提示里把 `/tmp` 路径当依据引用，都红。
- 定义点名的产出文件（跑前登记、腿的 output、样本、运行记录、实验页、代码与 kb 的改动）照定义写进文件。三方论证的云端腿与核查员，报告就是派发提示给的 `research/prompts/<轮>-*-output.md`：用 Bash 分段写进去，交回内容只写文件路径、`sha256sum` 与判定一览（照 `.claude/rules/three-way-inference.md`「云端腿的报告要分段落盘」办）。其余定义的「报告」全文写进报告文件：派发提示给了报告路径的写那里，没给的写草稿目录的 `report.md`（用 Bash 时 `set -o noclobber` 排他新建，之后 `>>` 分段追加）；交回只写结论、报告路径与 `sha256sum`，不超过 2000 字（交回正文闸拒超长的），主 agent 按路径存档。交回只调一次（第二次会被拒）。子 agent 用 Write 建文件名以 REPORT、SUMMARY、FINDINGS、ANALYSIS 开头（不分大小写）的 `.md` 会被工具层当场拒，Bash 写的不拦。
- 干到一半收到主 agent 的消息：当成追加的输入并进这一轮做，报告里写明在哪一步收到、改了什么；与定义或原输入冲突的，停在那一处交回。
