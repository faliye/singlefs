# tooling-detached-run-r1 正推腿（Sonnet）：T1 题面与规则对实现

日期 2026-10-01。被判：`research/scripts/detached-run.sh`（与冻结副本 sha256 相同，前 16 位 7906ffada7a0d756）、`.claude/agent-common.md`、`.claude/main-agent.md`（与冻结副本 sha256 相同，前 16 位分别 fa48493bb64a797a、79f5e49058d33bd5）。`crates/` 里没有相关实现（正文第二节已列 grep），本腿读的全是脚本、钩子与规则。行号都是去那份文件里现查的。

方法：从用户三句话与 `records/2026-09-16-subagent拆分提案.md` 第四十二节「去向」推出「入口应当是什么」，再逐格与脚本比。没起任何 systemd 单元、没跑 `--selftest`。能只读复核的（cgroup、Linger、会话、bus 失败的退出码）贴了原样输出；需要真起单元才能验的标「推的」或「复核不了」。

## 一、题面逐句核

用户三句话与「去向」四项：

| 来源 | 要什么 | 实现里对上的地方 | 判定 |
|---|---|---|---|
| 用户「脚本后台限制停这个要改一下」 | 后台时限不再停长跑 | start 起 `systemd-run --user` 的服务单元，主进程是 runner bash，不在会话任务的进程树里 | 一致（推的，见 1.1） |
| 用户「后台不能中断我们运行中的脚本」 | 调用方被停、后台时限到点、会话关闭，三者都不停脚本 | 前两者靠单元不在调用方树里；第三者靠用户级 systemd 还活着 | 前两者一致，第三者冲突（见 1.3） |
| 用户「后台只能上报，统计目前运行未结束的脚本」 | 会话里的后台任务只读、报还在跑的 | `watch` 只读 systemctl 与登记 | 一致，一条环境变量路径例外（见二） |
| 第四十二节去向：脱离入口、后台只上报、钩子 ③ 放行、规则与主 agent 定义跟着写 | 四件事 | 入口、`watch`、detector diff、三处规则句子都在 | 四件都在；规则句子有三处毛病（见四） |

### 1.1 调用方被停：命令照跑

靠什么：`systemd-run --user … --service-type=exec --unit=… --slice=…` 把 runner 挂到用户级 systemd 之下，调用方只拿到 systemd-run 的返回。

`research/scripts/detached-run.sh:278`：「elif ! systemd-run --user --quiet --collect --service-type=exec --expand-environment=no」

本机实测（只读）我自己这条 Bash 所在的 cgroup：

```
$ cat /proc/self/cgroup
0::/user.slice/user-1000.slice/session-4.scope
```

单元会落在 `user@1000.service` 的 `singlefs-detached.slice` 之下，与上面这个 session scope 是两棵子树；停调用方（杀进程、停 scope）碰不到它。`systemctl --user list-units 'singlefs-detached*'` 当前只有一个 slice 行、零个 service。

反例：没找到。自证格 Ⓐ（调用方放进另一个单元再停它）是主 agent 已跑过的实测，我没跑，复核不了。

推翻它的现象：停调用方之后命令进程不在、或 runner 的 `/proc/<pid>/cgroup` 落在调用方 scope 里。

### 1.2 会话后台时限：命令照跑

靠什么：时限杀的是 `run_in_background` 那个任务的进程树；脚本里这个任务只会是 `watch`（`start` 十秒内就返回）。runner 不在这棵树里（理由同 1.1），`watch` 与单元之间没有父子关系。

`research/scripts/detached-run.sh:352`：「trap 'detached_run_watch_stopped TERM 143' TERM」

反例：没找到。残留的一个事实是 `watch` 被时限停掉之后没有谁再上报：脚本头只写了「被 KILL 时最后那一次清单打不出来」；规则句子没写「被停之后重起 `watch`」（见四）。

推翻它的现象：时限到点后 `systemctl --user list-units 'singlefs-detached-*'` 里该单元不在；或时限是 cgroup 级的杀而 Claude Code 的任务就在 runner 同一 cgroup。

### 1.3 会话关闭：只在「用户级 systemd 还活着」时成立，脚本不查

靠什么：只靠用户级 systemd 不退出。本机现状（原样输出）：

```
$ loginctl show-user $(id -u) | grep -E 'Linger|State|Sessions'
State=active
Sessions=218 211 4 1
Linger=no
$ for s in $(loginctl list-sessions --no-legend | awk '{print $1}'); do loginctl show-session $s -p Type -p Remote -p State -p Class | tr '\n' ' '; echo; done
Remote=yes Type=tty Class=user State=closing 
Remote=yes Type=tty Class=user State=active 
Remote=yes Type=tty Class=user State=active 
Remote=yes Type=tty Class=user State=closing 
```

四个会话全是 SSH（Remote=yes），其中两个已是 closing；我这条 Bash 就在 closing 的 session-4 里。`man loginctl`（本机）与 `man logind.conf` 的 UserStopDelaySec 写明：不开 linger 时，用户的最后一个会话结束后用户级 systemd 被终止（默认延迟 10 秒；`/etc/systemd/logind.conf` 本机没改）。用户级 systemd 一停，里面的单元一起停。

`research/scripts/detached-run.sh:33`：「用户级 systemd 没开 linger（loginctl show-user 的 Linger=no）时，这个用户的最后一个登录会话结束，用户级 systemd 连同这里起的单元一起停」

脚本对这件事只写了注释：`grep -n -i linger research/scripts/detached-run.sh` 命中第 33 行注释与第 188 行的提示文字，没有一行在 start 时读 Linger、也没有拒绝或警告；运行条件只判 `research/scripts/detached-run.sh:57`：「run-condition: check systemctl --user show-environment >/dev/null 2>&1」。

判定：与用户第二句「后台不能中断我们运行中的脚本」、第四十二节现象里「会话一关也跟着停」比，会话关闭这一项只在还有别的登录会话时才不停。主 agent 打算起的是按天算的广谱变异，登出所有 SSH 就会被停，而 `list` 事后只会写「没留退出码」。我没登出过，不能复现，所以是推的。按正文第五节第一条，这一项是「条件式打中」，建议 start 读 Linger，Linger=no 时拒起或至少打一句警告并写进登记。

推翻它的现象：Linger=no 下登出所有会话、过了 UserStopDelaySec，单元仍在 `systemctl --user list-units` 里。

## 二、`watch` 是不是真的只上报

逐条核脚本里 `watch` 走到的每个会碰进程的调用：

| 调用 | 位置 | 碰到什么 |
|---|---|---|
| `systemctl --user show -p ActiveState --value`、`show -p MainPID` | `research/scripts/detached-run.sh:109`、`research/scripts/detached-run.sh:174` | 只读单元属性 |
| `kill "$DETACHED_RUN_WATCH_SLEEP_PID"` | `research/scripts/detached-run.sh:330` | 只有 `sleep "$every" &`（第 367 行）起的那一个，收尾后置空 |
| `trap '' TERM INT HUP` | 第 329 行 | 先挡掉后续信号再打最后一次清单，不转发任何信号 |

`research/scripts/detached-run.sh:330`：「if [[ -n "$DETACHED_RUN_WATCH_SLEEP_PID" ]]; then kill "$DETACHED_RUN_WATCH_SLEEP_PID" 2>/dev/null; fi」

`watch` 不是 runner 的父进程，runner 也不在它的 cgroup 里，所以它被 TERM、INT、HUP、KILL 时单元收不到任何东西（KILL 时只是最后一次清单打不出来，脚本头第 28 行已写）。默认环境下没找到会给单元或命令发信号的路径。

一条条件路径：弄坏开关不是只在 `--selftest` 里读。`research/scripts/detached-run.sh:89`：「detached_run_break_is() { [[ ",${DETACHED_RUN_BREAK:-}," == *",$1,"* ]]; }」；`research/scripts/detached-run.sh:332`：「if detached_run_break_is watch-stops-units; then」之后第 335 行对每个还在跑的单元 `systemctl --user stop`；`research/scripts/detached-run.sh:276`：「if detached_run_break_is in-caller-tree; then」让 start 把命令起在调用方树里。这些在生产调用里同样生效：主 agent 的环境里留着一个 `DETACHED_RUN_BREAK`（例如 export 过一次）就会让 `watch` 停单元、让 `start` 不脱离。钩子不拦这个环境变量。这是正推读出来的路径，我没跑（跑它要起单元）。

判定：默认环境一致；`DETACHED_RUN_BREAK` 设着时冲突。正文第五节第二条「`watch` 在任何路径上停了或发信号给被观察单元」按字面含这条路径，由主 agent 判它算不算。

推翻它的现象：`DETACHED_RUN_BREAK` 没设时 `watch` 被 TERM 之后，被观察单元的 `ActiveState` 变了或命令进程收到信号（看 `journalctl --user -u <单元>` 里有没有 Stopping 之类的停止记录、命令进程有没有收到信号）。

## 三、`list` 的「还在跑、没结束」与真实状态

判法在 `research/scripts/detached-run.sh:162`–`research/scripts/detached-run.sh:169`：有 exit 文件就算已结束；没有就按单元 ActiveState 分「还在跑」与「没留退出码」。关键两行：

`research/scripts/detached-run.sh:109`：「systemctl --user show -p ActiveState --value "$1" 2>/dev/null」

`research/scripts/detached-run.sh:168`：「if detached_run_is_active_state "$state"; then status=running; else status=vanished; fi」

登记目录那一圈：`research/scripts/detached-run.sh:146`：「records=("$detached_run_state"/*/)」，只遍历登记，不看 systemd 里实际有哪些 `singlefs-detached-*` 单元。

逐种情形（「实测」是我这一轮真跑出输出的，「推的」是从代码读出来、没起单元验）：

| 情形 | list 报 | 真实 | 一致？ | 依据 |
|---|---|---|---|---|
| 单元已被 `--collect` 卸掉、exit 在 | 已结束 + 退出码 | 同 | 一致 | 第 162–163 行；runner 在命令退出后写 exit（第 80、85 行），推的 |
| 命令被别人 SIGKILL（runner 活着） | 已结束，退出码 137 | 同 | 一致 | 第 79–85 行 `rc=$?` 再写 exit，推的 |
| runner 先死 | 没留退出码 | 命令随单元 cgroup 一起被停（systemd 的默认 KillMode），单元不在 | 一致（原因文案只列了三种，没列这一行） | 第 188 行文案，推的 |
| 登记目录被删（单元还在跑） | 不列这条；「还在跑 0 条」 | 单元与命令还在跑 | **不一致** | 第 146、152 行只认登记；`stop` 也要登记（第 380 行），登记没了连 `detached-run.sh stop` 都停不到，推的 |
| `systemctl --user show` 中途失败（bus 断、用户级 systemd 重启） | 没留退出码；`watch` 退 0 并报「都结束了」 | 单元与命令还在跑 | **不一致，已复现** | 见下 |
| start 的 `systemd-run` 失败（第 282 行 `exit 1`），登记已写 | 没留退出码，文案说「主进程被 KILL、用户级 systemd 退出…」 | 命令一行都没跑 | 不一致 | 第 264–270 行先写登记，第 278–285 行失败直接退出、不清登记，推的；我想用 shim 复现被钩子拒了（见下） |
| bus 不通时 start 判同名（第 233、234 行） | 状态为空判成「不在」，把可能还在跑的旧登记挪走（第 240–250 行），再因 systemd-run 同样失败退 1 | 旧单元还在跑，它写 exit 的路径是旧登记的绝对路径，现在指向新建的空登记 | 不一致 | 推的，两个失败要同时发生 |
| 同名重起 + `watch` | 报的是新一次的退出码 | 旧一次的退出码在 `<名>@<秒数>` 里，`watch` 的报告里没有 | 不一致（上报漏一次） | `watch` 的观察表以登记路径为键，`research/scripts/detached-run.sh:361`：「DETACHED_RUN_OBSERVED["$record"]=1」，旧登记被挪走（第 242–244 行）之后路径指向新的，推的 |
| `stop` 时命令在 `TimeoutStopSec` 内不退 | 没留退出码（`stop-requested` 只在已结束行上显示） | stop 记过 | 不一致 | `systemd-run` 调用（第 278–281 行）没带 `-p TimeoutStopSec=`，`grep -n 'TimeoutStopSec\|KillMode' research/scripts/detached-run.sh` 零命中，本机用户级默认 `DefaultTimeoutStopUSec=1min 30s`（原样输出，见下）；超时后 systemd 对 cgroup 里全部进程发 KILL，runner 一起没了，exit 写不出来，推的 |
| 在两次轮询之间起了又跑完的短命令 | `watch` 不报（没进观察表） | 跑过 | 不一致（上报漏一次） | 第 361 行只收「某一轮里还在跑」的，推的 |

bus 失败那一行的复现。真实形态（只读，不起单元）：

```
$ env XDG_RUNTIME_DIR=/nonexistent DBUS_SESSION_BUS_ADDRESS=unix:path=/nonexistent/bus systemctl --user show -p ActiveState --value x.service
Failed to connect to bus: No such file or directory
rc=1
```

stdout 空、退 1；而单元不存在时同一条命令打 `inactive`、退 0（本轮实测 `systemctl --user show -p ActiveState --value nonexistent-abc.service` 打 inactive、rc=0）。所以「空串」只能是查询失败，脚本第 109 行吞掉退出码，第 168 行把空串当作「不在」。

端到端复现（顶替的 systemctl，不起单元；脚本与输出在 `research/prompts/tooling-detached-run-r1-sonnet-model/`，先两次答 active、第三次起退 1）：

```
$ bash research/prompts/tooling-detached-run-r1-sonnet-model/repro-bus-flake.sh <detached-run.sh>
watch 退出码 0
2:  == 第 1 次（2026-10-01）
6:  ✓ detached-run.sh list：还在跑、没结束的 1 条，已结束 0 条，没留退出码 0 条（…）
7:  == 第 2 次（2026-10-01）
11:  ✓ detached-run.sh list：还在跑、没结束的 1 条，已结束 0 条，没留退出码 0 条（…）
12:  == 第 3 次（2026-10-01）
16:  ✓ detached-run.sh list：还在跑、没结束的 0 条，已结束 0 条，没留退出码 1 条（…）
18:  · fake 没留退出码（单元已不在：主进程被 KILL、用户级 systemd 退出或机器重启）
19:  ✓ detached-run.sh watch：观察到的 1 条都结束了，各自的退出码在上面
```

（节选行，全文 20 行在模型目录的 `watch-bus-flake.out`；那一次的「已不在」文案原样来自第 188 行。）本机用户级 systemd 的停单元超时：

```
$ systemctl --user show --property=DefaultTimeoutStopUSec,DefaultCPUAccounting | head
DefaultTimeoutStopUSec=1min 30s
DefaultCPUAccounting=yes
```

用 shim 复现「start 失败留登记」时，我对 `detached-run.sh start` 的那一条命令被 bash-command-detector 拒了（退出码 2，stderr 第一行：「✗ three-way-forward 用 research/scripts/detached-run.sh 起或停脱离会话的长跑」）。这是子 agent 角色拦截生效的一次观测，我没绕，该行只靠读代码判。

判定：「已结束」一侧（exit 在）可靠；「还在跑」与「没留退出码」两侧各有一处与真实不符：查询失败当成「不在」（已复现）、遍历只认登记。打中，按正文第五节第三条，改法进下一批工具改动：`detached_run_unit_state` 在 `systemctl` 退非 0 时报「查不到」，`list` 单列、`watch` 遇到「查不到」不当结束；`list` 再列一遍 `systemctl --user list-units 'singlefs-detached-*'` 里没有登记的。

推翻它的现象：bus 断时 `systemctl --user show -p ActiveState --value <单元>` 的退出码是 0，或 `list` 在查询失败时不打「没留退出码」。

## 四、三处规则与定义的新句子

照 `.claude/singlefs-ai-sop/rules/rules-discipline.md:13`：「怎么做：步骤、次序、判据、阈值」只写这一类，理由、日期、旧做法不写；指到别处的写文件名。`.claude/agent-common.md:4`：「这份与各份定义只写怎么做」。三句新句子里没有理由、日期、旧做法，也没有位置指代（对三行各 grep 「上一节、下面、前面提到、上述、如下：、同上、见下节、本条、本节、本表、本文档」，命中 0 次）。「只写怎么做」这一半过；与原有句子是否矛盾，三处各有问题。

### 4.1 `.claude/agent-common.md:63`「长活可以等」

新句子：「主 agent 跑得过后台时限的长活经 `bash research/scripts/detached-run.sh start --label <名> -- <命令>` 起」。

1. 缺判据：「跑得过后台时限」没有数。后台时限（约 30 分钟、最长 2 小时）只写在 `records/` 第四十二节里，规则正文里读不到，主 agent 读规则时判不出一条命令该走哪一边；判据与阈值属于正文（`.claude/singlefs-ai-sop/rules/rules-discipline.md:13`）。
2. 没写 `start` 在前台跑：同一条里更前面是「交给它的命令要一直跑到真正的活结束才退出」，写的是 `run_in_background` 的命令；`start` 十秒内就返回，放进 `run_in_background` 就是「命令退出、活还在」，完成通知当场发出。新句子只写「会话里只用 run_in_background 起 `bash research/scripts/detached-run.sh watch`」，靠「只」字暗示 `start` 不进后台，没有明说。
3. 同一条的禁用清单里仍写着裸的 `systemd-run`，新句子紧跟其后要求用一个内部就是 `systemd-run` 的入口；两句的次序与互相的例外关系要靠 ③ 那一条才读得通（见 4.2），这一条自己没写。
4. 没写 `watch` 被后台时限停掉之后怎么办（重起、或读 `list`）。`watch` 是唯一会被时限停的东西（1.2），停了之后没人叫醒主 agent；`.claude/main-agent.md:34` 的告警「结束本轮而手里没有在跑的后台任务」才会兜到，规则句子没把它写成一步。

与原句「不自己中途杀掉」不矛盾：「要停只用 `detached-run.sh stop <名>`」是「要停」时的唯一入口。

### 4.2 `.claude/agent-common.md:70` ③

新句子：「主 agent 经 `research/scripts/detached-run.sh start` 起的放行（`--` 之后那条命令照判）；子 agent 用它的 `start`、`stop` 照拒，`list`、`watch` 谁都放」。

与同一节开头 `.claude/agent-common.md:66`「主 agent 与子 agent 一样拒，只挂在一方才用的工具上的在条目里写明」矛盾：这一节的前提是两种身份一样被拒，例外只留给「只挂在一方才用的工具上」的那种；`detached-run.sh` 不是只挂在一方的工具，③ 现在却对主 agent 放行、对子 agent 拒。③ 自己写了差别，但前提句没改，两句并排读，前提句为假。改法是把前提句的例外说成「按身份分的写在条目里」。另外「`list`、`watch` 谁都放」说的是 ③ 从来没拒过的东西，不是怎么做。

### 4.3 `.claude/main-agent.md` 小节「派出去之后」里的新句子（第 40 行）

新句子：「跑得过后台时限的长命令经 `bash research/scripts/detached-run.sh start --label <名> -- <命令>` 起」，紧跟在「主 agent 自己起的长命令同样放后台，命令照共用约束「长活可以等」那一条写（不在里面再把活放到后台），由看门狗盯着进程：没有子 agent 在跑时用 `bash research/scripts/watch.sh --processes`」之后。

两句并排没说谁覆盖谁：`research/scripts/watch.sh:11`：「只盯这个 Claude 实例底下已经在跑的长进程」，经 `systemd-run --user` 起的 runner 的父进程是用户级 systemd，不在这个实例底下，`watch.sh --processes` 看不到它；`.claude/main-agent.md:34` 的告警「本实例底下有没带 `SINGLEFS_HEAVY_TESTS` 前缀的重型测试进程」也就看不到脱离单元里的重型命令（那一道只靠 heavy-test-guard 在起的时候判）。新句子没写「只有脱离单元在跑时还要不要起 `watch.sh --processes`」「两只看门狗各管什么」。「同样放后台」对跑得过时限的命令现在不再成立，没标例外。

推翻它们的现象：把三处句子给一个没读过背景的主 agent，让它起一条 3 小时的命令，它仍然把 `start` 放进 `run_in_background`、或不知道 `watch` 被时限停掉要重起，则 4.1 第 2、4 点成立；4.2 的矛盾是纯文字并排，无需实测。

## 五、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 1.1 调用方被停命令照跑 | 一致（推的） | 单元在 `user@1000.service` 之下、不在调用方 cgroup；我没起单元，自证格 Ⓐ 复核不了 |
| 1.2 会话后台时限命令照跑 | 一致（推的） | 时限只杀 `watch`，与单元无父子关系；`watch` 被停后无人叫醒的缺口见 4.1 第 4 点 |
| 1.3 会话关闭命令照跑 | 冲突（条件式，推的） | Linger=no、四个会话全是 SSH，最后一个会话结束用户级 systemd 连单元一起停，start 不查 Linger |
| 二 `watch` 只上报 | 默认环境一致；`DETACHED_RUN_BREAK=watch-stops-units` 路径冲突 | 弄坏开关在生产调用里同样生效，钩子不拦 |
| 三 `list` 已结束一侧 | 一致 | exit 文件写入是先写临时文件再改名 |
| 三 `list` 查询失败 | 冲突，已复现 | 失败当成「单元不在」，`watch` 退 0 宣布全部结束 |
| 三 `list` 登记被删 / start 失败残留 / 同名重起 / stop 超时 | 冲突（推的） | 遍历只认登记；失败残留被报成被 KILL；`watch` 以路径为键漏旧一次；90 秒后 KILL 丢退出码 |
| 4.1 agent-common「长活可以等」 | 只写怎么做：过；与原句：缺判据与前后台说明 | 缺阈值、没写 `start` 在前台、没写 `watch` 被停后重起 |
| 4.2 agent-common ③ | 与同节前提句矛盾 | 前提句「主 agent 与子 agent 一样拒」没改 |
| 4.3 main-agent「派出去之后」 | 与上一句并排未定谁覆盖谁 | `watch.sh --processes` 看不到脱离单元 |

## 六、没做什么

- 没起任何 systemd 单元、没跑任何 `--selftest`（钩子对子 agent 拒）；所有「推的」都等主 agent 或攻方腿在真起单元时核。
- 没复核主 agent 带前缀的验收（自证 11 格、五个开关、三份钩子自检）；本腿只读了脚本与规则。
- 没判 T2 的竞争与参数环境、T3 的钩子口子、T4 的自证放不放过错的实现；`--selftest` 里各格与 diff 里钩子格我只读到够判 T1 为止。
- 没验「Bash 工具的沙箱开着时 `systemd-run --user` 还能不能连上 bus」：我这一次 `systemctl --user` 能连，主 agent 的会话是否同样没验。
- 没验 `multi-host-run.sh --peer` 整个挂进单元之后 ssh 的认证来源（用户级 systemd 的环境里有 `SSH_AUTH_SOCK`，是不是活的没查）。
- 未做 `grep -n 'TimeoutStopSec\|KillMode' research/scripts/detached-run.sh` 之外的单元参数核查；它零命中（上面「stop 时命令在 TimeoutStopSec 内不退」一行的依据）。
- 临时副本与草稿：`/tmp/claude-1000/tooling-detached-run-r1-sonnet/` 下有 detached-run.sh 与 preflight 的副本（`tree/`）和 shim；交回前已删 `tree/`。
- 不采纳、不出判决。
