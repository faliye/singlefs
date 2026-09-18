# bg-notify-r1 云端攻方腿（Opus，`three-way-attack`）报告

写于 2026-09-18 15:28–15:36 UTC（东京 2026-09-19 00:28–00:36 JST）。攻击面：A1（两句新指令，四问全答）、A2（附录三 L1–L14 以外的命令，与误检叫醒主 agent 的代价）。L1–L14、A3、A4 不判。前几轮判决：无（第一轮）。

## 复跑与文件

在仓根下跑（约 15 秒，只写草稿目录；hook 一律喂自己的临时检出文件，不碰 `/tmp/claude-1000/agent-hook-detections.jsonl`）：

```
bash research/prompts/bg-notify-r1-opus-model/run.sh /tmp/claude-1000/bg-notify-r1-opus
```

本次原样输出存为 `research/prompts/bg-notify-r1-opus-model/run-output.txt`（79 行）。各文件 sha256（`sha256sum research/prompts/bg-notify-r1-opus-model/*` 原样）：

```
ed53ee0ffccfeed67e30483ad8c9b2ce58959399488b81524dd550304c3a8e9e  research/prompts/bg-notify-r1-opus-model/hook-variant.sh
ac4b2dff4a77b0d5a59cabb1d3873a9cecc7394c4ec9a6c88a794da10b511d09  research/prompts/bg-notify-r1-opus-model/l-regress.py
d5c0efe309caab191229e6d9ace40844f837cab65b9328c00f3c3866f5e80490  research/prompts/bg-notify-r1-opus-model/probe.py
a490a759353de7751f560c68cd9f42eff0dc2aae3d901834a596a19438b96e02  research/prompts/bg-notify-r1-opus-model/run-output.txt
6aabf96990b5019144e8b697f57bf444b2da31904a0275c96889877850fe6b56  research/prompts/bg-notify-r1-opus-model/run.sh
035e8bd2aa84f8101cb9f4b2bb2f5355e8fa80a2dd307aa72c90e41ea968edd8  research/prompts/bg-notify-r1-opus-model/shapes.py
```

| 文件 | 是什么 |
|---|---|
| `shapes.py` | 33 条表外形态：X（可能漏检）13 条、F（可能误检）7 条、C（对照）13 条。每条两列：`hook` 是喂给真 hook 的原文（真实会写的样子），`exec` 是真跑的那一条，只把真活换成 `work.sh`（睡几秒后写 done 标记），`&`、`wait`、nohup/setsid/disown、引号、heredoc 的位置逐字不变 |
| `probe.py` | 每条问两件事：仓里真 hook 记不记第三种；在模拟 harness 的外层 shell（`bash -c 'set -m; eval "$命令" < /dev/null && pwd -P >| cwd.out'`，新会话起、标准输出进文件）里真跑，外层 shell 退出之后还有没有活没完（= 脱钩） |
| `hook-variant.sh` | 本腿提的 hook 改法（副本，**只在我的模型上量过、被攻过零轮**） |
| `l-regress.py` | 变体与仓里 hook 在 L1–L14 上第三种判定差几条（只为说明变体动了哪几格，不判 L 表） |
| `run.sh`、`run-output.txt` | 复跑入口与本次原样输出 |

模拟 harness 的依据（本腿在真 harness 里现查，前台 Bash，2026-09-18 约 15:09 UTC）：外层 shell 是 `/bin/bash -c source <快照> … && eval '<命令>' < /dev/null && pwd -P >| /tmp/claude-XXXX-cwd`，`ps` 显示它 PID = PGID = SID（会话首进程），`$-` 为 `hmtBc`（同机普通 `bash -c` 为 `hBc`，多出的 `m` 即作业控制开着），标准输出与标准错误都指向 `tasks/<id>.output` 文件而不是管道。所以 harness 判「完成」只看外层 shell 退出，不等继承了输出的子进程。

开工快照在开工与写报告前（15:27:58 UTC）各核一次，`sha256sum -c research/prompts/bg-notify-r1-start-snapshot.sha256` 两次都是 8 行 OK，腿跑着的时候被判的 8 个文件没变。

## 各格判定一览

| 格 | 判定 | 打中的形状（编号见 `shapes.py`） | 依据的量 |
|---|---|---|---|
| A1① 漏禁 | **打中** | X01（事故那条删掉 nohup、disown 后剩下的样子：`&` 后面跟 `echo`，不在末尾）、X02/X06/X07（`&` 在子 shell 或命令替换里）、X04/X05（「并行起几个再 `wait`」字面放行的 `wait $!`、`wait -n`）、X08–X11（coproc、`tmux -d`、`screen -dm`、`systemd-run`，不在三个词里）——字面都允许，都脱钩 | 模型表（量过，X11 推的）；X06/X07/X08 真 harness 前台复核，X06 真 harness `run_in_background` 端到端 |
| A1① 误禁 | 没打中 | 字面禁掉而其实不脱钩的只有 `nohup` 不带 `&`（F01）与 `setsid -w`（F07），两种都没有非用不可的场合 | 模型表 |
| A1② | **打中（弱）** | C13：长活与缓存计时器塞进同一条 `… & … & wait`，两句字面同时满足，计时器那次叫醒被吞掉（外层 6.0 秒才退，计时器 2.0 秒就完）；X04 反过来 `wait $!` 等计时器，长活脱钩 | 模型表（量过） |
| A1③ | **打中（在判据两种之外）** | 第三种 completed：子 agent 结束本轮时手里没有活着的后台任务、也没交回（表外漏检 X03、X06–X12，前台放后台 X13，之后结束本轮又没另起计时器，都走到这里；X04、X05 里并着计时器，计时器完时照常叫醒，不走到这里）。入口那段只给两种读法，第三种没说怎么办；看门狗对它与「等后台」报同一个状态 | 通知原文（第三种的 result 原文本腿看不到，推的）；`agent-watch.py` 状态判法 |
| A1④ | **打中 2 处（范围外同形）＋ 1 处弱** | `.claude/main-agent.md:23` 主 agent 自己「同样放后台，由看门狗盯着进程」、`:21` 临时派的 agent 只带计时器那一条，都没带新指令；放后台的进程 PPID 变 1，看门狗的进程那一半看不见。弱：`three-way-local-attack.md:27`、`three-way-inference.md:302` 的「前台跑」与 240 秒前台上限 | 逐处 grep；PPID=1 真 harness 实测三次 |
| A1④ 其余各处 | 没打中 | `gate-triage.md:24–25`、`crash-verifier.md:23–24`、`command-safety.md:47–48`、`cache-keepalive.sh:16` 与新指令相容（crash-verifier 给的写法 C10 量过：不记、不脱钩） | 逐处 grep；模型表 |
| A2 漏检 | **打中** | 仓里 hook 在 33 条表外形态上：漏检 10 条（X03–X10、X12 量过；X13 是前台，在判据之外），另 X11 推的 | 模型表 |
| A2 误检 | **打中** | 7 条（F01–F07，量过）；F02–F05（起了再按 pid 轮询、`tail --pid`、heredoc 里 Python 或 Rust 的 `&`）算常见，F01、F06、F07 少见 | 模型表 |
| A2 代价 | 单次漏检贵于单次误检；频率没有数 | 误检 ≈ 叫醒主 agent 一轮；漏检至少同样叫醒一次（10 分钟「无动静」），外加子 agent 停摆、缓存过期整份重写、退出码丢失、进程掉出看门狗视野 | 见 A2 第四节；频率本腿量不到 |

仓里 hook 在 33 条上的判定分布（`run-output.txt` 第一张表，命令数出）：漏检 10、误检 7、记且脱钩 6、不记不脱钩 9、没真跑 1。变体（副本）在同 33 条上：漏检 1（X13，前台）、误检 1（F06）、记且脱钩 15、不记不脱钩 15、没真跑 1；变体自检通过（17 种）；变体与仓里 hook 在 L1–L14 上第三种判定只差 1 条（L13）。

## A1 两句新指令

### ① 字面禁的与该禁的对不上（漏禁：打中；误禁：没打中）

被判的那一句，`.claude/agent-common.md:40` 整行：

> - 长活可以等，不给它设超时、不自己中途杀掉：编译、变异表、复跑这类长活用 Bash 的 `run_in_background` 起（命令照前台的写法原样交给它，不再加 `nohup`、`setsid`、`disown`，也不在末尾加 `&`；并行起几个再 `wait` 的照常写），起完结束本轮，完成时会通知你；结束本轮就是这一条回复只写一句在等什么、不再调工具；前台命令的 `timeout` 不超过 240000 毫秒。

字面只禁两样：三个词（`nohup`、`setsid`、`disown`）与「在末尾加 `&`」；另明文放行一样：「并行起几个再 `wait` 的照常写」。下表每条都照这三处字面放得过去，而在模拟 harness 里外层 shell 在真活完之前就退了（harness 在那一刻发完成通知）：

| 编号 | 命令（喂 hook 的原文，路径 `G=/tmp/claude-1000/gate-triage-commit-0918`） | 字面上为什么放得过 | 外层退出 / 真活完成（秒，量过） | 仓里 hook 记不记 |
|---|---|---|---|---|
| X01 | `nice -n 19 bash .claude/scripts/gate.sh --staged > $G/gate-run.log 2>&1 & echo "started pid $!"` | 三个词都没有；`&` 后面还有 `echo`，不在末尾。这正是事故那条（附录三 L1）照新指令删掉 `nohup`、`disown` 之后剩下的样子 | 0.0 / 6.0 | 记 |
| X02 | `(nice -n 19 bash .claude/scripts/gate.sh --staged > $G/gate-run.log 2>&1 &)` | 末尾是右括号，不是 `&` | 0.0 / 6.0 | 记 |
| X06 | X02 后面加 `; wait` | 同上，且以 `wait` 收尾 | 0.0 / 6.0 | 不记 |
| X07 | `pid=$(nice -n 19 bash … > $G/gate-run.log 2>&1 & echo $!); echo "gate pid $pid"; wait "$pid"` | `&` 在命令替换里、不在末尾；以 `wait` 收尾 | 0.0 / 6.0 | 不记 |
| X04 | `nice -n 19 bash … > $G/gate-run.log 2>&1 & bash research/scripts/cache-keepalive.sh & wait $!` | 「并行起几个再 `wait`」 | 2.0 / 6.0 | 不记 |
| X05 | 同 X04，`wait $!` 换成 `wait -n` | 同上 | 2.0 / 6.0 | 不记 |
| X08 | `coproc GATE { nice -n 19 bash … > $G/gate-run.log 2>&1; }` | 没有 `&`、没有三个词 | 0.0 / 6.0 | 不记 |
| X09 | `tmux new-session -d -s gate 'nice -n 19 bash … > $G/gate-run.log 2>&1'` | 同上 | 0.0 / 6.0 | 不记 |
| X10 | `screen -dmS gate bash -c 'nice -n 19 bash … > $G/gate-run.log 2>&1'` | 同上 | 0.0 / 6.0 | 不记 |
| X11 | `systemd-run --user --collect --unit gate-run nice -n 19 bash .claude/scripts/gate.sh --staged` | 同上 | 没真跑（推的：不带 `--wait` 起完单元就返回） | 不记 |

真 harness 复核（不是模拟，本腿的真 Bash 工具）：

- 前台一条命令里连起 X06、X07、X08 三种（各睡 25 秒），原样输出：

```
/bin/bash: line 1: wait: pid 428797 is not a child of this shell
wait on x07 pid rc=127
coproc pid=428798
```

  下一次调用原样：`elapsed since start 4.9`、`ls: cannot access 'done.*': No such file or directory`，`ps` 显示 428797（X07 的活）与 428798（X08 的 coproc）两行 PPID 都是 `1`。工具调用在真活完之前约 20 秒就返回了。
- `run_in_background` 端到端跑 X06（`(bash …/work.sh e2e 40 > …/e2e.log 2>&1 &); wait`，先离线喂 hook 确认记 0 条才跑，免得叫醒主 agent）：起步时刻 15:20:02.230 UTC；harness 在 15:20:02.238 往任务输出文件写 `[exited with code 0]`，随后送来 `<status>completed</status>`「completed (exit code 0)」的通知；真活 15:20:42.233 才写完成标记，之后再没有通知；`/tmp/claude-1000/agent-hook-detections.jsonl` 前后都是 24 行（hook 没记）。

四句：
1. 分不分辨臂：分辨。这一格的臂是「保留这句的字面」与「改写」；上表每条在现字面下放行，在下面的改写下被禁。
2. 被判的系统看不看得到：写命令的子 agent 看得到自己的命令；判别它要的只是「外层 shell 会不会在真活完之前退出」，这是命令文本的性质。
3. 满足判据字面哪一句：A1 触发的观测「会脱钩却允许的」；问法里的「漏禁会脱钩的写法」。
4. 跑前条款的改法在这些格上中不中：条款写「A1 打中 ⇒ 改那句指令」。只补词表（加 coproc、tmux、screen、systemd-run）修不了 X01、X02、X06、X07、X04、X05；要改成讲原则的写法，见下。hook 那一侧（A2 条款）的补法对 X01、X02 本来就记，对其余见 A2。

改写（本腿提的，**只在我的模型上量过、被攻过零轮**；是指令文字，能不能让子 agent 少写上表那些，没法在这里量，全格是「推的」）：

> 长活用 Bash 的 `run_in_background` 起，外层 shell 要一直活到真活跑完：命令里不再放后台——不用 `nohup`、`setsid`、`disown`、`coproc`、`tmux`/`screen` 的分离模式、`systemd-run`，不在子 shell `( … &)` 或命令替换 `$( … &)` 里放后台；`&` 只用来在同一个 shell 里并行起几件，最后跟一个不带参数的 `wait`。缓存计时器不并进去，另起一条。

误禁一侧（没打中）：字面禁掉的写法里，不脱钩的只有 `nohup` 不带 `&`（F01：真 harness 前台跑 `nohup bash …/work.sh f01 12 > f01.log 2>&1`，原样 `call returned after 12.0 s; done.f01 at 12.0`，即工具等它跑完才返回）与 `setsid -w`（F07，模型量过）；两种都没有非用不可的场合，禁了不损失什么。「末尾加 `&`」若按行读，会碰到 C05 那种 heredoc 里逐行 `&`、最后 `wait` 的写法，但同一句的「并行起几个再 `wait` 的照常写」放行了它。F02、F03（起了再按 pid 轮询、`tail --pid`）字面放行，不脱钩，这一侧也对。

### ② 「结束本轮只写一句、不再调工具」与起计时器、写进度记录（打中一处，弱）

与之同段的计时器那一句，`.claude/agent-common.md:41` 整行：

> `.claude/agent-common.md:41`   **结束本轮去等之前，再用 `run_in_background` 起一次 `bash research/scripts/cache-keepalive.sh`**：它 230 秒后退出，那条完成通知把你叫醒。被它叫醒就看一眼等的东西跑完没有：没跑完再起一次、结束本轮接着等，跑完了接着干、不用再起。自己已经交回或被停的不用管。预计单段要等 15 分钟以上的（层 0 全量、E152（按里程碑对比六家文件系统的文件性能） 那类），不起计时器。

打中的场景 C13：`bash research/scripts/cache-keepalive.sh & nice -n 19 bash .claude/scripts/gate.sh --staged > $G/gate-run.log 2>&1 & wait`，整条用 `run_in_background` 起，然后照第 40 行结束本轮。第 40 行「并行起几个再 `wait` 的照常写」放行它，第 41 行「结束本轮去等之前，再用 `run_in_background` 起一次」也算做到了（计时器确实是用 `run_in_background` 起的，只是和长活同一条）。模型量过：外层 shell 6.0 秒才退（= 长活），计时器 2.0 秒就完，那一次叫醒被吞进长活的完成通知里。换成真尺寸，计时器 230 秒、全量门禁十几分钟，子 agent 在 230 秒上不会醒，5 分钟档缓存过期；续上时的代价见 `records/2026-09-16-subagent拆分提案.md:579` 整行：

> `records/2026-09-16-subagent拆分提案.md:579` - **单次成本**：一次计时器叫醒 ≈ 2–3 次调用 × 13.5 万读缓存 ≈ 4 万折基础输入；一次整份重写 ≈ 14 万。⇒ 单段等待超过约 14 分钟时，计时器比让缓存过期更贵；这条写进了共用约束。

hook 不记 C13（它不脱钩，按 hook 的判据不该记），所以只有指令能挡。X04（`… & 计时器 & wait $!`）是反过来的一种：计时器照常叫醒，长活脱钩。

四句：
1. 分不分辨臂：分辨。现字面放行，「缓存计时器不并进去，另起一条」的改写禁掉它。
2. 看不看得到：写命令的子 agent 看得到。
3. 满足判据哪一句：问法 ②「与别处要求结束本轮之前先做的事（起缓存计时器……）冲不冲突」；触发「按新指令字面会做错的具体场景」。
4. 改法中不中：条款「改那句指令」——在第 40 行的括注里补「缓存计时器不并进去，另起一条」即修到（推的）；A2 的 hook 补法修不到（不是脱钩）。

弱在哪里：要子 agent 自己决定把两件并成一条才会走到；本腿没有频率数。

同一问里没打中的：
- 次序：第 40 行「起完结束本轮……结束本轮就是这一条回复只写一句在等什么、不再调工具」读在前，第 41 行「结束本轮去等之前，再……起一次」读在后。按顺序读是「起长活 → 起计时器 → 只写一句结束」，不矛盾。
- 进度记录：`.claude/agent-common.md:42` 要写进度记录的时点是「等的时候看到进程不动、报错」，那是被叫醒之后的一轮，不是结束本轮的那条回复，不冲突。
- 「预计单段要等 15 分钟以上的……不起计时器」：不起计时器时，结束本轮的回复本来就只剩一句，不冲突。

### ③ 入口那段对 completed 通知的读法（两种读对了；第三种没有读法：打中，在判据字面两种之外）

被判的那一段，`.claude/main-agent.md:29` 整行：

> `.claude/main-agent.md:29` 交回按子 agent 的 SubagentHandback 消息判。后台派的子 agent 每结束一轮，harness 发一条 status 为 completed 的通知；结果写「This agent has not reported yet: it is waiting on its own background work」、或 note 写「the result below may be interim」的，是它结束本轮去等自己起的后台任务：不当交回、不当出错，照旧等它的交回消息与看门狗。

主 agent 记下的通知原文在 `research/prompts/_bg-notify-r1-background.md:39`（整行）：

> `research/prompts/_bg-notify-r1-background.md:39` | 主 agent 收到的通知原文 | 子 agent 结束本轮而后台任务还在时：status `completed`，note「This agent stopped with background work of its own still running. It may resume on its own when that work completes or reports, and the same task-id notifies again if it does; the result below may be interim.」，result「This agent has not reported yet: it is waiting on its own background work and will deliver its report through SubagentHandback when that finishes.」；这一次收到两次（13:53:44、13:58:29 各结束一次本轮）。子 agent 交回之后：status `completed`，note「A task-notification fires each time this agent stops with no live background children of its own. The user can send it another message and resume it, so the same task-id may notify more than once.」，result「This agent's report was delivered to you as a message from "<id>" (its SubagentHandback call). Read it there; it is not repeated here.」（sweep a78b10a27371fb5a5 那一次） | 主会话里收到的通知逐字抄 |

两种都读对了：等后台那种按「may be interim」或 result 那句认，不当交回；交回那种按 SubagentHandback 消息认。

第三种：交回那种的 note 自己写着「A task-notification fires each time this agent stops with no live background children of its own.」——这是「每次停下、手里没有活着的后台子任务」都发，不限于交回过的。于是「结束本轮、手里没有活着的后台任务、也没交回」必然也有一条 completed 通知：按那句的字面，note 应与交回那种同一句，result 不是「This agent's report was delivered…」（这两样的原文本腿都看不到，推的；主 agent 能在主会话记录里现查）。走到这里的路：表外漏检的 X03、X06–X12，或前台放后台的 X13，之后照第 40 行结束本轮、又没有另起计时器；或者子 agent 忘了交回。（X04、X05 把计时器并在同一条里，计时器一完外层就退、照常叫醒，不走到这里。）

按这段做：它既不含那两句，也没有交回消息；这段只对前一种说「照旧等」。若主 agent 把「completed 而没交回」一概当成在等后台（这段要纠正的正是「completed 就是结束」的误读，最顺手的泛化就是这一种），它会等一个不会来的叫醒——子 agent 手里没有活着的后台任务，不会自己醒，note 也写着要「send it another message」才会续。看门狗帮不上分辨：`research/scripts/agent-watch.py:214` 对最后一条是结束本轮的一律判「本轮结束、没交回」，与等后台那种同一个状态；要等 `--idle-minutes` 默认 10 分钟（第 809 行）报「无动静」才叫醒；脱钩的真活 PPID 是 1，不在看门狗进程那一半的视野里（④ 里给出处）。代价：至少 10 分钟停摆，外加子 agent 5 分钟档缓存过期、续做时整份重写（第 579 行那条）。

四句：
1. 分不分辨臂：分辨。加一句第三种的读法就修到，不加就不修。
2. 看不看得到：看得到——主 agent 收通知那一刻，note 写「still running」还是「no live background children」、有没有交回消息，都在眼前；看门狗的状态分不开（同为「本轮结束、没交回」），只能靠通知。
3. 满足判据哪一句：A1 的问法 ③ 字面只点了两种 completed（等后台、已交回），这两种判得对；打中的是第三种，在字面之外，也不在 A3 的三种情形（等后台、交回、被 TaskStop 停掉）里。算 A1③ 还是另立一格，由主 agent 定。
4. 改法中不中：条款「改那句指令」——在这段末尾补一句即修到（推的）；A2 的 hook 补法只减少走到这里的次数（X03、X06–X12），修不到 X13 与忘了交回。

补的一句（本腿提的，**被攻过零轮**）：

> status 为 completed、没有交回消息、note 与 result 里也没有那两句的：它结束本轮时手里没有还在跑的后台任务，不会自己醒；看它最后在等什么，发消息让它接着做、交回，或停掉。

### ④ 与 16 份定义、`.claude/rules/` 里现有后台跑法的逐处对照（打中 2 处范围外同形 + 1 处弱；其余相容）

逐处找的命令与原样输出（16 份定义 + 7 份规则，关键词：`run_in_background|后台|结束本轮|nohup|setsid|disown|前台|单次上限|keepalive`）：

```
.claude/agents/gate-triage.md:1
.claude/agents/mutation-triage.md:0
.claude/agents/crash-verifier.md:1
.claude/agents/experiment-runner.md:0
.claude/agents/three-way-attack.md:0
.claude/agents/three-way-materials.md:0
.claude/rules/path-moves.md:0
.claude/agents/prior-art.md:0
.claude/rules/implementation-first.md:0
.claude/agents/sweep.md:0
.claude/agents/three-way-local-attack.md:1
.claude/agents/experiment-designer.md:0
.claude/rules/mutation-sampling.md:0
.claude/agents/kb-scribe.md:0
.claude/agents/three-way-forward.md:0
.claude/agents/implementation-writer.md:0
.claude/agents/three-way-verifier.md:0
.claude/rules/implementation-workflow.md:0
.claude/agents/three-way-local-defense.md:0
.claude/agents/three-way-defense.md:0
.claude/rules/fs-design.md:1
.claude/rules/format-evolution.md:0
.claude/rules/three-way-inference.md:3
```

非 0 的 5 个文件逐行看过（`fs-design.md:25` 讲文件系统的后台整理，与跑命令无关），另把 `gate-triage.md:24`、`crash-verifier.md:23`（「等它结束」）、`.claude/singlefs-ai-sop/rules/command-safety.md:47–48`（共用约束第 42 行点名的那份）、`research/scripts/cache-keepalive.sh:16` 与主 agent 入口 `.claude/main-agent.md:21`、`:23` 一并对了。

**打中 1（范围外同形）：`.claude/main-agent.md:23`。**

> `.claude/main-agent.md:23` 叫醒之后主 agent 判断：只是慢就再起一个看门狗接着盯；是问题就决定发消息让它改、停掉重派、还是报给用户。主 agent 自己起的长命令同样放后台，由看门狗盯着进程。

主 agent 不读共用约束，新加的那句管不到它；「同样放后台」没说怎么放。若照事故那样 `nohup … &`（或表外任一脱钩写法）起，真活的 PPID 变成 1——本腿在真 harness 里三次看到：15:09 前台 `( … ) &` 起的 425807、X07/X08 复核里的 428797 与 428798、`run_in_background` 端到端里的 429079，事后 `ps` 的 PPID 列都是 `1`。而「由看门狗盯着进程」靠的是这一半：

> `research/scripts/agent-watch.py:26` 进程这一半只看运行这个脚本的那个 Claude 实例的后代进程（主 agent 与它派出的子 agent 起的命令都在里面），不看别的会话。
> `research/scripts/agent-watch.py:404`     descendants, frontier = [], [root_pid]

PPID 为 1 的进程不是 Claude 实例的后代，进程这一半看不见它，「进程无输出」「进程过长」都报不出来，这一句就落空。另外主 agent 的命令与子 agent 同一个会话 id（本腿自己的一条命令在检出记录里的 `session_id` 就是主会话那个），hook 记下主 agent 自己的放后台命令，看门狗照样退出、叫醒主 agent 自己。

**打中 2（范围外同形）：`.claude/main-agent.md:21`。**

> `.claude/main-agent.md:21` 子 agent 的提示缓存是 5 分钟档，由它自己续（`research/scripts/cache-keepalive.sh`，共用约束「不做」一节），主 agent 不定时 ping 它。临时派不读共用约束的 agent（general-purpose 这类）干带长等待的活时，派发提示里写上这一条。

临时派的 general-purpose 只被要求在提示里写上计时器那「这一条」，新加的「命令里不再放后台」没带上；这类 agent 起的命令照样过 hook、照样叫醒主 agent，而它从没被告知这条。

四句（两处同答）：分辨臂——补一句即修到，不补不修；看得到——主 agent 写命令、写派发提示时看得到；满足判据哪一句——A1 问法 ④「与……现有的后台跑法指令有没有打架」，但字面范围是「16 份定义与 `.claude/rules/`」，主 agent 入口不在里面，所以记「范围外同形」，归不归这一格由主 agent 定；改法——条款「与之打架的定义一并改」：`:23` 改成「主 agent 自己起的长命令同样用 `run_in_background` 起、命令里不再放后台（放后台的进程挂到 1 号底下，看门狗看不见）」，`:21` 的「这一条」扩成「计时器与『命令里不再放后台』这两条」（推的，零轮）。

**弱（推的）：`.claude/agents/three-way-local-attack.md:27` 与 `.claude/rules/three-way-inference.md:302–304`。**

> `.claude/agents/three-way-local-attack.md:27` 3. 前台跑 `bash research/scripts/ask-local.sh <提示文件> > <前缀>-output-s<n>.md`，不许 `setsid`、`&`、`disown`。
> `.claude/rules/three-way-inference.md:302` ⚠️ **本地腿要前台跑，不要 `setsid ... & disown`。** 实测两次：后台进程消失而
> `.claude/rules/three-way-inference.md:303` stdout / stderr **都是 0 字节**，同一条命令前台 56 秒跑完。
> `.claude/rules/three-way-inference.md:304` 0/0 正是「缺席必须显式报告」要禁的形态，后台跑法让它无法诊断。
> `research/scripts/ask-local.sh:14` TIMEOUT="${ASK_LOCAL_TIMEOUT:-900}"
> `research/prompts/agent-defs-r1-local-defense-runlog.md:6` 开跑前 `ps` 未见本机有 `cargo`、`qemu-system`、`gate.sh` 在跑影响本轮（本轮不编译、不跑门禁，`ps` 检查不适用）；本机本地模型此刻同时被别的会话的三方轮次在用，实测两次调用分别耗时 93 秒与 102 秒（首次含一次判红），比派发提示预告的「可能排队更慢」量级内。

与新指令的字面不冲突（两边都不许放后台）。冲突在旁边：共用约束第 40 行把「用 `run_in_background` 起、命令里不放后台」立成长活的跑法，并限前台 timeout 不超过 240000 毫秒；本地攻方定义写死「前台跑」，而 `.claude/agent-common.md:3` 说与定义冲突时以定义为准，于是本地腿只能前台跑，`ask-local.sh` 自己最多等 900 秒，超过 240 秒的那一次会被工具超时截掉。有记录的时长是 56 秒、93 秒、102 秒，都在 240 秒内，本腿没有超过 240 秒的实例，只记为弱。改法（零轮）：两处的「前台跑」改成「前台或 `run_in_background` 跑，命令里不放后台」。

**相容、没打中的各处：**

| 位置 | 为什么相容 |
|---|---|
| `gate-triage.md:25`「超过 Bash 单次上限时后台跑、结束后读输出。」 | 没说怎么放后台；共用约束第 40 行现在说了，两边不矛盾。事故就出在这一句没说清、共用约束当时也没说 |
| `gate-triage.md:24`、`crash-verifier.md:23`「等它结束」 | 等别人起的 `gate.sh`，按写死 pid 等，用 `run_in_background` 起、不放后台即可 |
| `crash-verifier.md:24` 给的 `{ nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1` | 模型 C10：hook 不记、不脱钩；多一个 `&` 的 C11：记且脱钩。「不认起后台的那条命令自己的 `$?`」在 `run_in_background` 下也成立（harness 通知里的退出码是 `echo` 的） |
| `command-safety.md:47–48`「等自己起的后台任务用 `wait`」与按写死 pid 等 | 与「并行起几个再 `wait`」同向；「起了再按 pid 轮询」（F02）字面放行、不脱钩，但 hook 记它，见 A2 误检 |
| `cache-keepalive.sh:16` 到点那句 | 「没跑完就再用 run_in_background 起一次本脚本、结束本轮接着等」，与第 40、41 行同向 |
| 其余 13 份定义、5 份规则 | 计数为 0，不写后台跑法 |

## A2 hook 第三种检出：表外命令的漏检与误检

被判的判法，`.claude/hooks/bash-command-detector.sh` 第 29、31、36 行整行：

> `.claude/hooks/bash-command-detector.sh:29` SELF_DETACH = re.compile(r"\bnohup\b|\bsetsid\b|\bdisown\b")
> `.claude/hooks/bash-command-detector.sh:31` LONE_AMPERSAND = re.compile(r"(?<![&>|<])&(?![&>])")
> `.claude/hooks/bash-command-detector.sh:36`     return any(not re.search(r"\bwait\b", command[match.end():]) for match in LONE_AMPERSAND.finditer(command))

### 一、量到的（`run-output.txt` 第 2–36 行原样，仓里的 hook）

| 编号 | run_in_background | hook 第三种记了没有 | hook 全部检出 | 外层 shell 退出（秒） | 各件活完成（秒） | 外层退出之后才完的活 | 判定 |
|---|---|---|---|---|---|---|---|
| X01 | True | 记 | run_in_background 里又自己放后台 | 0.0 | gate=6.0 | gate | 记且脱钩（对） |
| X02 | True | 记 | run_in_background 里又自己放后台 | 0.0 | gate=6.0 | gate | 记且脱钩（对） |
| X03 | True | 不记 | 无 | 0.0 | gate=6.0 | gate | 漏检 |
| X04 | True | 不记 | 无 | 2.0 | gate=6.0, keepalive=2.0 | gate | 漏检 |
| X05 | True | 不记 | 无 | 2.0 | gate=6.0, keepalive=2.0 | gate | 漏检 |
| X06 | True | 不记 | 无 | 0.0 | gate=6.0 | gate | 漏检 |
| X07 | True | 不记 | 无 | 0.0 | gate=6.0 | gate | 漏检 |
| X08 | True | 不记 | 无 | 0.0 | gate=6.0 | gate | 漏检 |
| X09 | True | 不记 | 无 | 0.0 | gate=6.0 | gate | 漏检 |
| X10 | True | 不记 | 无 | 0.0 | gate=6.0 | gate | 漏检 |
| X11 | True | 不记 | 无 | — | — | — | hook 不记；没真跑 |
| X12 | True | 不记 | 无 | 0.0 | gate=6.0 | gate | 漏检 |
| X13 | False | 不记 | 无 | 0.0 | gate=6.0 | gate | 漏检 |
| F01 | True | 记 | run_in_background 里又自己放后台 | 4.0 | gate=4.0 | 无 | 误检 |
| F02 | True | 记 | 没有 timeout 的等待循环；run_in_background 里又自己放后台 | 4.0 | gate=4.0 | 无 | 误检 |
| F03 | True | 记 | run_in_background 里又自己放后台 | 4.0 | gate=4.0 | 无 | 误检 |
| F04 | True | 记 | run_in_background 里又自己放后台 | 4.0 | gate=4.0 | 无 | 误检 |
| F05 | True | 记 | run_in_background 里又自己放后台 | 4.0 | gate=4.0 | 无 | 误检 |
| F06 | True | 记 | run_in_background 里又自己放后台 | 4.0 | gate=4.0 | 无 | 误检 |
| F07 | True | 记 | run_in_background 里又自己放后台 | 4.0 | gate=4.0 | 无 | 误检 |
| C01 | True | 记 | run_in_background 里又自己放后台 | 0.0 | gate=4.0 | gate | 记且脱钩（对） |
| C02 | True | 不记 | 无 | 4.0 | gate=4.0 | 无 | 不记不脱钩（对） |
| C03 | True | 不记 | 无 | 4.0 | a=4.0, b=4.0, c=4.0 | 无 | 不记不脱钩（对） |
| C04 | True | 记 | run_in_background 里又自己放后台 | 0.0 | gate=4.0 | gate | 记且脱钩（对） |
| C05 | True | 不记 | 无 | 4.0 | gate=4.0, keepalive=2.0 | 无 | 不记不脱钩（对） |
| C06 | True | 不记 | 无 | 4.0 | gate=4.0 | 无 | 不记不脱钩（对） |
| C07 | True | 记 | run_in_background 里又自己放后台 | 0.0 | gate=4.0 | gate | 记且脱钩（对） |
| C08 | True | 不记 | 无 | 4.0 | gate=4.0 | 无 | 不记不脱钩（对） |
| C09 | True | 不记 | 无 | 4.0 | gate=4.0 | 无 | 不记不脱钩（对） |
| C10 | True | 不记 | 无 | 4.0 | gate=4.0 | 无 | 不记不脱钩（对） |
| C11 | True | 记 | run_in_background 里又自己放后台 | 0.0 | gate=4.0 | gate | 记且脱钩（对） |
| C12 | True | 不记 | 无 | 4.0 | a=4.0, b=4.0, c=4.0 | 无 | 不记不脱钩（对） |
| C13 | True | 不记 | 无 | 6.0 | gate=6.0, keepalive=2.0 | 无 | 不记不脱钩（对） |

「外层退出」即 harness 发完成通知的时刻；「外层退出之后才完的活」非空即脱钩。F02 的 hook 检出里另带「没有 timeout 的等待循环」，是第二种检出，照录。

### 二、漏检（会脱钩、hook 不记）

| 编号 | hook 为什么放过 | 会不会真写出来（本腿判断，没有频率数） |
|---|---|---|
| X03 | 第 36 行只问 `&` 之后的文本里有没有 `wait` 这个词；`echo "… I will wait for the notification"` 的字面就算 | 中：模型起完常 `echo` 一句说明 |
| X04 | 同上；`wait $!` 只等最后一个（计时器），长活脱钩 | 中低：要把长活与计时器并成一条 |
| X05 | 同上；`wait -n` 等最先完的那个 | 低 |
| X06 | 同上；`( … &)` 里的活是子 shell 的孩子，外层 `wait` 没有孩子可等，当场返回 | 中低：双 fork 是常见的「放后台」惯用法，后面加 `wait` 少见 |
| X07 | 同上；`$( … & echo $!)` 起的不是外层 shell 的孩子，真 harness 原样报 `wait: pid 428797 is not a child of this shell`、退出码 127 | 低 |
| X08 | 第 29 行只有三个词、没有 `&`；coproc 异步，外层不等 | 低 |
| X09、X10 | 同上；`tmux new-session -d`、`screen -dmS` | 中：模型「长活放后台」的惯用写法之一 |
| X11 | 同上；`systemd-run` 不带 `--wait`（推的，没真跑） | 低 |
| X12 | heredoc 在外层 `&` 之后，正文里的 `p.wait()` 被当成 `wait` | 低 |
| X13 | 前台起（`run_in_background` 为假），第三种检出按设计不看；起完不会有任何完成通知，子 agent 之后照第 40 行结束本轮就等不到 | 高：前台 `… &` 是模型最顺手的写法；但在判据之外（见四句 2） |

另在草稿里现量了三条（不在模型文件里，命令与原样输出如下），变体也补不上 V2、V3：

```
V1 命令替换里 sed 的 & | 仓里=记 变体=记
V2 子 shell 里的 wait 冒充 | 仓里=不记 变体=不记
V3 & 在单引号里的 bash -c 且有 wait | 仓里=不记 变体=不记
```

（V1 `name=$(printf '%s' "$x" | sed 's/&/and/'); nice -n 19 bash gate.sh > "$name.log" 2>&1`，不脱钩，两版都误检；V2 `… & (wait)`；V3 `bash -c '… &'; wait`，是 L14 的机制后面加一个 `wait`。）

### 三、误检（不脱钩、hook 记了）

| 编号 | 为什么记 | 常不常见（本腿判断） |
|---|---|---|
| F01 | `nohup … > log 2>&1` 不带 `&`：第 29 行见词就记；nohup 不 fork，真 harness 前台跑到活完才返回（12.0 秒，见 A1①） | 中：习惯性加 nohup；新指令本来就禁它，记下与指令一致，但检出那句「完成通知当场发出」是错的 |
| F02 | `… & pid=$!; while kill -0 "$pid" …; do sleep 20; done; tail -3 …`：外层 shell 轮询到活完才退，`&` 之后没有 `wait` 这个词 | 常见：`command-safety.md:47` 教的正是按写死 pid 等；而且它同时带第二种检出，不再是「只有等待循环」，`agent-watch.py:494` 那条「只检出等待循环的不立刻叫醒」救不了它，会立刻叫醒 |
| F03 | `… & tail --pid=$! -f log > /dev/null` | 中 |
| F04 | heredoc 里的 Python `row['flags'] & 0x4` | 中：执行员、攻方腿在草稿里用 heredoc 跑分析 |
| F05 | heredoc 写一段 Rust（`&[u8]`、`|&b|`）再 `cargo test` | 中：实现员、攻方腿在仓副本里常这么做，编译又长，正该后台起 |
| F06 | `sed -E 's/…/!! &/'` 的替换串 | 低 |
| F07 | `setsid -w`（等它结束） | 低 |

hook 文件头第 15 行已写 heredoc 会误记：

> `.claude/hooks/bash-command-detector.sh:15` # 只看顶层命令文本；命令里只是带着这些字（heredoc 里的测试数据）也会被记一条，主 agent 看了判断即可。

那一句写的时候，触发词是 `pgrep`、`until` 这类词；第三种的触发字符是 `&`，Rust 的引用、Python 与 shell 算术的按位与、sed 的整段匹配都用它，比那两种常见得多。

### 四、误检叫醒主 agent 的代价与漏检的代价

误检这一路：hook 记一行 → 看门狗 15 秒内读到（第 580 行按 `--detection-poll-seconds` 睡，默认 15）→ 不是「只有等待循环」就退出码 3（`agent-watch.py:586`）→ harness 叫醒主 agent → 主 agent 读报告、判、再起一个看门狗。代价的锚：

> `records/2026-09-16-subagent拆分提案.md:577` - **第一次 A/B（主 agent ping）**：两组同类型同提示，18 分钟各等两段 9 分钟。主 agent 四轮 ping 多花 9 次调用、读缓存 296.8 万、1 小时写 1.6 万、输出 2,878 ≈ $1.71（Opus 5，上下文约 30 万）；B 组回应四次 ping 读缓存 53.3 万 ≈ $0.11；能省的上限是 B 组两次醒来的整份重写 ≈ $0.61。⇒ 主 agent 侧一轮 ≈ $0.43，高于一次重写省下的 $0.31。**这一轮 A 组的数作废**：两组起步上下文逐字相同，缓存按前缀共享，B 组的 ping 把 A 组的 12.8 万前缀也续住了，A 组两次隔 9 分钟醒来都读到缓存、只重写自己那 4–5k。
> `records/2026-09-16-subagent拆分提案.md:540` | 看门狗每遇到一个正常的等待循环（等编译、等全量跑写出 `.done`）就立刻叫醒主 agent，一个核查员一轮里叫醒了三次，三次主 agent 核下来都没事 | `research/scripts/agent-watch.py`：hook 只检出「没超时的等待循环」的记录不再立刻叫醒，交给会话记录那一路满 3 分钟再报；按模式找进程与写被拒照旧立刻报。自检加一格，`AGENT_WATCH_BREAK=loopnoise` 判红 |
> `records/2026-09-16-subagent拆分提案.md:576` - **主 agent 忙时通知卡住**：主 agent 在一轮里忙着时，发给子 agent 的后台完成通知只入队不出队（主会话记录的 `queue-operation` 行），三个探针分别卡了 10 分 18 秒、约 1 分、约 1 分，主 agent 空闲后全部送到并交回；同一时段主 agent 发的 SendMessage 即时送到（探针 17:28:44 UTC 收到）。ping 过与没 ping 过的探针都卡，所以卡住与 ping 无关。

即一次误检 ≈ 主 agent 一轮（那次量的是约 30 万上下文、$0.43 一轮；第 577 行作废的是 A 组子 agent 醒来读缓存的数，主 agent 侧这一笔按那行字面不在作废之列）；同类的「只有等待循环」误检曾一轮叫醒三次、三次都没事，已为此改过看门狗。主 agent 忙时通知排队，误检的叫醒会晚到、不会丢。还有一笔推的：检出那句写着「完成通知当场发出，跑完的那个不会叫醒你」，对 F01–F07 是错的；主 agent 若照它让子 agent 改了重起，白跑一次长活。

漏检这一路分两种：
- 子 agent 没有另起、还活着的计时器（单段 15 分钟以上的活按第 41 行本来就不起，或它忘了）：真活完了不叫醒它；看门狗对它报「本轮结束、没交回」，10 分钟后才报「无动静」（第 809 行默认 10）——这本身就是一次叫醒，至少等于一次误检；另加至少 10 分钟停摆、子 agent 5 分钟档缓存过期后续做的整份重写（第 579 行：约 14 万折基础输入）、真活退出码丢失、真活 PPID 为 1 看门狗进程那一半看不见（A1④）。每再起一个看门狗，它最后一条记录越来越旧，立刻又报「无动静」，主 agent 只能出手。
- 另有一条独立的计时器活着：230 秒内被叫醒、照第 41 行看一眼就能补上；代价只剩退出码丢失与进程看不见。

结论：单次漏检（前一种）严格贵于单次误检；后一种便宜。总账要乘频率，本腿没有频率数（检出记录里第三种只有一行，是主 agent 14:45:56 自己的探针）。推翻条件：量出 F02–F05 这类命令在 `run_in_background` 里出现得比表外漏检多一个数量级以上。

### 五、改法各修哪一格（本腿提的，**只在我的模型上量过、被攻过零轮**）

变体 `hook-variant.sh` 在副本上跑同一张表，`run-output.txt` 第 38–73 行原样：

== 变体 hook-variant.sh（sha256 ed53ee0ffccfeed67e30483ad8c9b2ce58959399488b81524dd550304c3a8e9e；本腿提的改法，被攻过零轮）
| 编号 | run_in_background | hook 第三种记了没有 | hook 全部检出 | 外层 shell 退出（秒） | 各件活完成（秒） | 外层退出之后才完的活 | 判定 |
|---|---|---|---|---|---|---|---|
| X01 | True | 记 | run_in_background 里又自己放后台 | 0.0 | gate=6.0 | gate | 记且脱钩（对） |
| X02 | True | 记 | run_in_background 里又自己放后台 | 0.0 | gate=6.0 | gate | 记且脱钩（对） |
| X03 | True | 记 | run_in_background 里又自己放后台 | 0.0 | gate=6.0 | gate | 记且脱钩（对） |
| X04 | True | 记 | run_in_background 里又自己放后台 | 2.0 | gate=6.0, keepalive=2.0 | gate | 记且脱钩（对） |
| X05 | True | 记 | run_in_background 里又自己放后台 | 2.0 | gate=6.0, keepalive=2.0 | gate | 记且脱钩（对） |
| X06 | True | 记 | run_in_background 里又自己放后台 | 0.0 | gate=6.0 | gate | 记且脱钩（对） |
| X07 | True | 记 | run_in_background 里又自己放后台 | 0.0 | gate=6.0 | gate | 记且脱钩（对） |
| X08 | True | 记 | run_in_background 里又自己放后台 | 0.0 | gate=6.0 | gate | 记且脱钩（对） |
| X09 | True | 记 | run_in_background 里又自己放后台 | 0.0 | gate=6.0 | gate | 记且脱钩（对） |
| X10 | True | 记 | run_in_background 里又自己放后台 | 0.0 | gate=6.0 | gate | 记且脱钩（对） |
| X11 | True | 记 | run_in_background 里又自己放后台 | — | — | — | hook 记；没真跑 |
| X12 | True | 记 | run_in_background 里又自己放后台 | 0.0 | gate=6.0 | gate | 记且脱钩（对） |
| X13 | False | 不记 | 无 | 0.0 | gate=6.0 | gate | 漏检 |
| F01 | True | 不记 | 无 | 4.0 | gate=4.0 | 无 | 不记不脱钩（对） |
| F02 | True | 不记 | 没有 timeout 的等待循环 | 4.0 | gate=4.0 | 无 | 不记不脱钩（对） |
| F03 | True | 不记 | 无 | 4.0 | gate=4.0 | 无 | 不记不脱钩（对） |
| F04 | True | 不记 | 无 | 4.0 | gate=4.0 | 无 | 不记不脱钩（对） |
| F05 | True | 不记 | 无 | 4.0 | gate=4.0 | 无 | 不记不脱钩（对） |
| F06 | True | 记 | run_in_background 里又自己放后台 | 4.0 | gate=4.0 | 无 | 误检 |
| F07 | True | 不记 | 无 | 4.0 | gate=4.0 | 无 | 不记不脱钩（对） |
| C01 | True | 记 | run_in_background 里又自己放后台 | 0.0 | gate=4.0 | gate | 记且脱钩（对） |
| C02 | True | 不记 | 无 | 4.0 | gate=4.0 | 无 | 不记不脱钩（对） |
| C03 | True | 不记 | 无 | 4.0 | a=4.0, b=4.0, c=4.0 | 无 | 不记不脱钩（对） |
| C04 | True | 记 | run_in_background 里又自己放后台 | 0.0 | gate=4.0 | gate | 记且脱钩（对） |
| C05 | True | 不记 | 无 | 4.0 | gate=4.0, keepalive=2.0 | 无 | 不记不脱钩（对） |
| C06 | True | 不记 | 无 | 4.0 | gate=4.0 | 无 | 不记不脱钩（对） |
| C07 | True | 记 | run_in_background 里又自己放后台 | 0.0 | gate=4.0 | gate | 记且脱钩（对） |
| C08 | True | 不记 | 无 | 4.0 | gate=4.0 | 无 | 不记不脱钩（对） |
| C09 | True | 不记 | 无 | 4.0 | gate=4.0 | 无 | 不记不脱钩（对） |
| C10 | True | 不记 | 无 | 4.0 | gate=4.0 | 无 | 不记不脱钩（对） |
| C11 | True | 记 | run_in_background 里又自己放后台 | 0.0 | gate=4.0 | gate | 记且脱钩（对） |
| C12 | True | 不记 | 无 | 4.0 | a=4.0, b=4.0, c=4.0 | 无 | 不记不脱钩（对） |
| C13 | True | 不记 | 无 | 6.0 | gate=6.0, keepalive=2.0 | 无 | 不记不脱钩（对） |

同一文件第 75–79 行原样（变体自检；变体与仓里 hook 在 L1–L14 上）：

    == 变体自检
      ✓ 自检通过：按模式找进程、没超时的等待循环、run_in_background 里又自己放后台记进检出记录，普通命令、重定向里的 & 与前台的 & 不记，入口一律放行（查了 17 种）
    
    == 变体与仓里 hook 在 L1–L14 上
    L1–L14 共 14 条，变体与原 hook 第三种判定不同的 1 条：L13（原 记 → 变体 不记）

同一份变体设 `BASH_COMMAND_DETECTOR_DISABLE_SELF_BACKGROUND=1` 跑自检（草稿副本，sha256 与 `hook-variant.sh` 相同），原样末两行 `  ✗ 自检：后台起的 & 在 wait 之后 应当是 1，实际 0`、`    → 看 findings_for() 与 record()；…`，退出码 1。

| 变体里的一条 | 修哪几格 | 量过 / 推的 |
|---|---|---|
| 放后台的词加 `coproc`、`tmux … new(-session) … -d`、`screen -dm…`、`systemd-run`（不带 `--wait/--pipe/--pty/--scope`）、`start-stop-daemon -b` | X08、X09、X10：量过（上表）；X11：hook 记是量过，脱钩是推的 | 量过 / 推的 |
| `wait` 要在命令位置（行首或 `; & \| ( {`、换行、`then/do/else` 之后），后面不接 `.`、`(`、`-`、字母 | X03：量过 | 量过 |
| 起了两个以上 `&` 时只认不带参数的 `wait`；只起一个时带 pid 参数也算；`-n` 不算 | X04、X05：量过；L12 不变：量过（L 表只差 L13 一条） | 量过 |
| `… &)` 与 `$( … & … )` 一律记 | X06、X07：量过 | 量过 |
| 喂给非 shell 解释器的 heredoc，正文剥掉再找 `&`；喂给 `bash`/`sh` 的保留 | F04、F05、X12：量过；C04、C05 不变：量过 | 量过 |
| `&` 之后有 `tail --pid` 或 `while/until … kill -0` 的算在等 | F02、F03：量过 | 量过 |
| `nohup` 只在同一行还有单独 `&` 时算；`setsid -w/--wait` 不算 | F01、F07：量过；副作用 L13 由记变不记：量过 | 量过 |
| 修不到：X13（前台） | hook 在那一刻看不到子 agent 之后会不会结束本轮去等；要在「结束本轮、手里没有活着的后台任务、没交回」那一刻判（看门狗那一侧，A4 的地盘，本腿不判） | 推的 |
| 修不到：F06、V1（引号里的 `&`） | 不能一概跳过引号：L14 那种 `bash -c '… &'` 引号里的 `&` 是真放后台；照条款「少见的记下由主 agent 看了判」写进 hook 文件头 | 推的 |
| 修不到：V2、V3 | 变体照样漏（草稿现量，见第二节） | 量过 |

变体没给新形态加自检样本；要采纳，X03–X12、F01–F07 各应进自检一格（推的）。

四句（A2 整格）：
1. 分不分辨臂：分辨。上表：仓里判法漏 10、误 7，变体漏 1、误 1，同一批命令逐条对得上。
2. 被判的系统看不看得到：X03–X12、F01–F07 的判别全在命令文本里，hook 看得到。X13 看不到——前台 `&` 之后子 agent 是在本轮里轮询（合法）还是结束本轮去等（出错），hook 触发那一刻还没发生，属「判别子观测不到」，不该拿它判 hook 的判据。F06、V1 半看得到：要区分「sed 替换串里的 `&`」与「`bash -c` 里的 `&`」，正则分不开，得真解析 shell。
3. 满足判据哪一句：漏检是 A2 触发的「会让等的东西与完成通知脱钩、而 hook 不记的命令」；误检是「不脱钩、而 hook 记了的常见命令」——「常见」只对 F02–F05 成立，F01、F06、F07 按少见记。
4. 跑前条款的改法中不中：「漏检 ⇒ 判据能补就补」对 X03–X12 中（上表量过），对 X13、V2、V3 不中，照条款写进文件头「判不到的」；「误检 ⇒ 常见的就收窄」对 F02–F05 中，F01、F07 顺带中，F06 照「少见的记下」处理。

## 推翻条件（各打中一条）

| 结论 | 什么现象会推翻它 |
|---|---|
| A1① 表外写法字面放行且脱钩 | harness 的后台任务改成等整个进程组或全部后代才算完成（本次端到端：外层 15:20:02.238 就写了 `[exited with code 0]`，真活 15:20:42 才完，之后没有第二条通知）；或第 40 行被读成「凡是后面没有 `wait` 的 `&` 都禁」 |
| A1② C13 吞掉计时器的叫醒 | harness 对同一条命令里每个子进程结束各发一条通知（本次观测是只看外层 shell 退出） |
| A1③ 第三种 completed 没有读法 | 主会话记录里查到第三种通知的 note 或 result 也带「may be interim」或「has not reported yet」那句；或 harness 在没交回、手里又没有后台任务时根本不发通知 |
| A1④ `main-agent.md:23` 落空 | 看门狗的进程那一半另有办法认出挂到 1 号底下的进程（按现有代码只走 Claude 实例的后代）；或主 agent 从不在命令里放后台 |
| A2 漏检 10、误检 7 | 真 harness 的外层 shell 与模拟在这些形态上表现不同（已在真 harness 复核 X06、X07、X08、F01 前台，X06 后台） |
| A2 单次漏检贵于单次误检 | 量出主 agent 被叫醒一轮远贵于子 agent 停摆十分钟加一次整份重写；总账还要看频率，频率数能改排序 |

## 没打中的形状

- A2 对照 13 条全对（C01–C13）：`timeout … &` 记且脱钩、`timeout` 不带 `&` 不记；`xargs -P` 并行不记；heredoc 喂给 `bash` 里放后台的记、里面并行加 `wait` 的不记；`exec` 顶替不记、`exec … &` 记；进程替换接 `tee` 不记；`cd … && … > log; echo "exit=$?" >> log` 长命令链不记；`crash-verifier.md:24` 的写法不记、多一个 `&` 记；`for … & done; wait` 不记；计时器与长活并行加 `wait`（C13）不记（按 hook 判据对，问题在 A1②）。
- 仓里脚本自己放后台：`.claude/`、`research/scripts`、`scripts` 下命令位置上的单独 `&` 找到 3 处（`.claude/singlefs-ai-sop/scripts/selftest.sh:665`、`.claude/gate.d/55-qemu-first-transaction.sh:47`、`research/scripts/check-staged.sh:123`），逐处看过都跟着 `wait`；没有「起一个脚本、脚本自己脱钩」的现成路径。
- kb、实验页里给的跑法带 `nohup`/`setsid`/`disown`：在 `.claude`、`research`（除 `research/prompts/`）、`crates`、`scripts` 下找，只有共用约束、本地攻方定义、hook、三方规则 4 个文件提到，都是禁令或判法，没有叫人这么跑的。
- A1① 误禁、A1② 次序与进度记录、A1④ 五处相容：见各节。
- 取样范围：模型 33 条（X 13、F 7、C 13）＋草稿现量 3 条（V1–V3）＋真 harness 复核 5 次（前台 `( … ) &`、前台 X06/X07/X08 一次、前台 F01、后台 X06 端到端）。

## 这条腿自己的限度

- 模拟 harness 只复刻了 `set -m`、新会话、标准输出进文件、`eval … < /dev/null && pwd -P >| …`，没有 source harness 的 shell 快照（在 `~/.claude/` 底下，不碰），`$-` 里的 `t` 没复刻。真 harness 只复核了上面 5 次，其余形态只在模拟里跑过。
- 真活用 2–6 秒的 `work.sh` 顶替；量的是「外层 shell 先退还是真活先完」，与时长无关，只有 `wait -n` 取决于谁先完。
- 「会不会真写出来」「常不常见」全是本腿的判断，没有频率数；第三种检出上线以来检出记录里只有主 agent 自己的一行探针。
- 第三种 completed 通知的 result 原文看不到（要读主会话记录，腿不碰 `~/.claude/`）。
- X11 `systemd-run` 没真跑（不想在 user systemd 里留单元），脱钩是推的。
- 变体被攻过零轮，只拿 V1–V3 三条试过它；A1 的两句改写是文字，子 agent 照不照做在这里量不了。
- 本轮本地攻方的运行记录 `research/prompts/bg-notify-r1-local-attack-runlog.md` 不在禁读清单字面里，为了不串腿也没读。

## 没做什么

- 不判 L1–L14、A3、A4；只为说明变体动了哪几格，报了「变体与仓里 hook 在 L1–L14 上只差 L13 一条」。
- 不判正推、辩方的格，不替主 agent 采纳；副本（草稿里的 hook 副本、`hook-variant.sh`）上量的数不算入库装置上的数。
- 只写了本报告、`research/prompts/bg-notify-r1-opus-model/` 与草稿目录 `/tmp/claude-1000/bg-notify-r1-opus/`（每格的临时目录、hook 副本、真 harness 复核的标记文件）；没改仓里别的文件，没编译 Rust，没跑门禁阶段（`stage-owners.tsv` 里登记给 `three-way-attack` 的阶段数为 0）。
- 真 harness 复核起的测试进程都已自己跑完（15:33 UTC 用 `ps` 看过，没有残留）；`tmux`、`screen` 只在模拟里用带本腿名字的会话起，跑完即退。
- 本腿自己的一条前台命令在检出记录里留了一行：15:24:23 UTC，`three-way-attack`，「没有 timeout 的等待循环」——那是我 `grep -nF` 核 `command-safety.md:47` 引文时，命令串里带着那句按写死 pid 等的示例循环；只有等待循环一种，按 `agent-watch.py:494` 不立刻叫醒主 agent。其余命令都是前台起的（第三种按设计不看前台）；唯一一条 `run_in_background`（X06 端到端）先离线喂 hook 确认不记才跑。检出记录从本腿开工时的 24 行变成 25 行，就这一行。
- 跑的中途没有收到主 agent 的消息。
