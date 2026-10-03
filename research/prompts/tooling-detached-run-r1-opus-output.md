# 工具轮第一轮 攻方腿（Opus）报告：T2 脚本 / T3 闸的口子 / T4 自证与开关

立场：攻方。被判文件在冻结副本 `/tmp/claude-1000/tooling-detached-run-r1/tree/`（整份一字未改地读）。
下面引的 `research/scripts/detached-run.sh`、`.claude/hooks/bash-command-detector.sh`、`.claude/hooks/lib_heavy_tests.py`
三份，主仓与冻结副本的 sha256 相同（`research/prompts/tooling-detached-run-r1-snapshot/files-sha256.txt` 核过），行号两处一致。
T3 按派发办法「不真执行」：把命令写成 hook 输入 JSON 喂进副本里的 `bash-command-detector.sh` 与 `heavy-test-guard.sh`，
看退出码与 stderr（原样输出全在 `research/prompts/tooling-detached-run-r1-opus-model/t3-hook-probes.txt`）。
T2、T4 要真起单元的写成复现脚本放模型目录，判决时主 agent 带前缀跑；读代码推得了的自己判。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T2 脚本 | 没打中（读到的形状都对） | `--force` 剥除、`env --` 开头的 `名=值`、PATH 相对项都是文档化行为；退出码/竞争这类要真跑的格写成 repro 交主 agent；只找到一处无害死代码 |
| T3 子 agent 闸 | **打中** | `source` / `.` 起脚本、改名或改符号链接名的拷贝（basename 变了）都放过去（rc=0），真能起脱离单元；eval / python subprocess / 变量拼路径是继承的通用盲区 |
| T3 重型闸 | **打中** | 改名拷贝 `bash dr2.sh start -- cargo test --all` 把内层重型命令藏过前缀与角色判（rc=0），主 agent 不带前缀、子 agent 都放过去 |
| T4 自证 | **打中** | 自证 Ⓔ 只给 watch 发 TERM、从不发 HUP / INT；一个「watch 收到 HUP 时停单元」的错实现 11 格全绿——而 HUP 正是「会话关闭」那一路 |

跑前判据对照（正文第五节）：T3「造出子 agent 起得出脱离单元 / `start -- <重型命令>` 绕过前缀与角色 ⇒ 打中，必须改」——两条都中。
T4「写出一种错实现、自证与钩子自检全绿 ⇒ 打中，补自证格或开关」——中。

打中之后的四句（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」）：
① 不涉及臂，是闸的口子，不是分辨臂的格；② 判别子看得到——闸拿到的是整条命令文本，`source` 与改名在文本里都在，
重型闸用的 `lib_shell_words` 本来就认 `source`（见 T3 第 2 节），是子 agent 闸的 `wrapped_commands` 用了更弱的匹配，可改；
③ 满足的是「子 agent 起得出脱离单元」「绕过前缀与角色」两个字面分句；④ T3 判据只有「必须改」一个去向，没有互斥改法要逐格核。

## T2　脚本哪一格会错

读了全份 `research/scripts/detached-run.sh`（610 行），没找到确定的代码级错误。逐项：

- **退出码与 stop**：runner 把命令以 `exec env -- "$@"` 起在服务 cgroup 里；`systemctl --user stop` 发 TERM 给
  整个 control-group，命令直接收 TERM 死掉（143），runner 续写 `exit`，与自证 Ⓖ 判的一致。命令被 SIGKILL / 整 cgroup 被杀时
  runner 一起死、不写 `exit`，`list` 据此报「没留退出码」，是设计里的 vanished 分支，不算错。
- **`--force` 剥除**：`research/scripts/detached-run.sh:60`「`preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS`」
  经项目 preflight 把每个 `--force`（连 `--` 之后的）摘掉；文件头第 19 行已写明，并给出路 `-- bash -c '<命令> --force'`。文档化行为，不是 bug。
- **`env --` 开头的 `名=值`**：GNU `env --` 之后开头的 `名=值` 仍当赋值，与文件头第 18 行所述一致；自证 Ⓓ 验了 `$` / `%` 透传。
- **一处无害死代码**：runner 的 `research/scripts/detached-run.sh:77`「`trap 'stop_requested=1' TERM INT HUP`」设了
  `stop_requested`，但这个变量全脚本再没被读过；TERM 经 cgroup 直达命令，功能不靠它，所以无害。只记一笔，不算打中。

要真跑才判的格（命令被 OOM / `--collect` 时序下 `exit` 写不写得出、两个同名 start 交错、`stop` 停不到 fork 出去的子进程、
用户级 systemd 不在时的退路）按派发办法不自己起单元：复现由主 agent 带前缀跑，脚本与期望在
`research/prompts/tooling-detached-run-r1-opus-model/`。本机 `loginctl show-user` 实测 `Linger=yes`、`systemd 255`，
与文件头「`Linger=no`」的前提不同——会话关闭后用户级 systemd 不随之退出，脱离单元照活，对本功能是更有利的一侧，不改判定。

## T3　闸的口子

喂 JSON 进两道钩子，退出码 2 = 拒、0 = 放。原样全在 `tooling-detached-run-r1-opus-model/t3-hook-probes.txt`。
先核基线：子 agent `bash research/scripts/detached-run.sh start --label m -- sleep 1` → 子 agent 闸 rc=2（认出调用）；
主 agent 同一条 → rc=0。闸的正路对。认得的写法：`nice` / `env` / `capped.sh` 前缀、`bash -c '…'`、`command` 内建都 rc=2。
行内赋值翻不了开关：`BASH_COMMAND_DETECTOR_ALLOW_SUBAGENT_DETACHED_RUN=1 bash … start` 子 agent 仍 rc=2（钩子读自己的 os.environ，不读命令里的赋值前缀）——这一处防得对。

### 1　子 agent 闸：`source` / `.` 与改名拷贝绕过（打中）

子 agent 闸 `detached_run_refusal`（`.claude/hooks/bash-command-detector.sh:521`「`def detached_run_refusal(command, agent_type):`」）
走 `detached_run_calls`（同文件 `:510`「`def detached_run_calls(text):`」），里面用
`:516`「`subcommand, rest = detached_run_subcommand(arguments) if name == DETACHED_RUN_SCRIPT_NAME else`」只在
命令词 basename 恰为 `detached-run.sh` 时才判。命令词是经 `wrapped_commands`（`:368`「`def wrapped_commands(words):`」）剥出来的，
它只认前缀、`WRAPPERS_TAKING_ONE_ARGUMENT`（`:380`）、`detached-run.sh … --` 自身与 `SHELL_NAMES`，**不认 `source` / `.`**。

实测（子 agent `clerk`）两种写法 bash 闸 rc=0（放过去）：
- `source research/scripts/detached-run.sh start --label m -- sleep 1` → rc=0；`. research/scripts/detached-run.sh start …` → rc=0。
- `cp research/scripts/detached-run.sh research/scripts/dr2.sh && bash research/scripts/dr2.sh start …` → rc=0；
  `ln -s detached-run.sh research/scripts/drl.sh; bash research/scripts/drl.sh start …` → rc=0。
  闸的注释 `:87`「`认的是命令词名叫 detached-run.sh，路径不论`」——路径不论，但名一变就不认，拷贝改名正好钻这个。

这两种真能起脱离单元：`source` 在当前 shell 里跑完 preflight 与派发，到 `detached_run_start` 起 `systemd-run`；
改名拷贝放在 `research/scripts/` 旁，`research/scripts/detached-run.sh:59`「`source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"`」
按 `BASH_SOURCE` 的目录解析得到 preflight，脚本照常跑。功能确认脚本 `repro-t3-rename-source-bypass.sh` 放模型目录，主 agent 带前缀跑。
继承的通用盲区（eval、变量拼路径、`python3 -c subprocess`）同样 rc=0，但那是整条 ③ 都看不到的（钩子注释自己列了），
不是本改动新开的；新开的是 `source` / `.` 与改名两类——`lib_shell_words` 本就认 `source`（见下节），本可堵。

### 2　重型闸：改名拷贝把内层重型命令藏过前缀与角色（打中）

重型闸经 `command_under_detached_run`（`.claude/hooks/lib_heavy_tests.py:851`「`def command_under_detached_run(words):`」）
剥 `start -- <命令>`，而它 `:854`「`if (os.path.basename(words[0]) != DETACHED_RUN_SCRIPT_NAME`」同样只认 basename。
改名拷贝的 basename 不是 `detached-run.sh`，于是 `bash dr2.sh start -- cargo test --all` 不被剥开：重型闸把 `dr2.sh` 当普通脚本
读正文（里面没有 cargo），`-- cargo test --all` 当脚本的位置参数、不再当命令判。

实测对照（`t3-hook-probes.txt` 的 cases-heavy）：
- 基线真名：主 agent 不带前缀 `bash research/scripts/detached-run.sh start --label m -- cargo test --all` → heavy rc=2（认出全量测试、要前缀）。闸的正路对。
- 改名：主 agent 不带前缀 `bash research/scripts/dr2.sh start --label m -- cargo test --all` → heavy rc=0；子 agent 同一条 → heavy rc=0 且 bash 闸 rc=0。
  即改个脚本名，主 agent 不带 `SINGLEFS_HEAVY_TESTS` 就跑了全量测试，子 agent 双闸全过、跑了 checker 档。满足判据「`start -- <重型命令>` 绕过前缀与角色」。
- 对称一笔（记录，不是洞）：`source research/scripts/detached-run.sh start -- cargo test --all` 重型闸 rc=2（认出）——
  因为重型闸用 `lib_shell_words` 的 scan，它本就把 `source` / `.` 当 launcher；子 agent 闸的 `wrapped_commands` 没用这条，才漏了 `source`。
  两道闸对同一种写法判得不一样，正说明「用 scan 的那套能堵 `source`」。

### 3　主 agent 经入口藏放出追踪 / 终止进程：`--` 之后那条都接着剥（没打中）

主 agent `start -- <内层>`，内层照 ③⑥ 判，实测都 rc=2：`bash -c 'long & disown'`、`systemd-run --user bash gate.sh`（且重型闸 rc=2）、
`bash -c 'kill -- -1'`、`setsid -f sleep 100`、`bash -c 'nohup sleep 100 &'`、`pkill -f cargo` 全部拒。
`-- bash -c 'x' -- disown` 的 `disown` 落到 `bash -c` 的位置参数、不在命令位，rc=0 是对的。
内层整份覆盖 `research/results/` 那一类我这份非 git 副本判不了（控制组直接写同一文件也 rc=0，是副本不是 git 仓所致，不是洞）——交主 agent 在真仓里复核。

## T4　自证与开关

自证共 11 格（Ⓗ 三条 + Ⓑ Ⓓ Ⓕ Ⓒ Ⓖ Ⓐ Ⓔ + 收尾 leftover）。先核它防得住的几种错实现：
- `start` 起成 `systemd-run --scope`：scope 在调用方 cgroup 里，Ⓐ 把调用方放进自己的单元再停整个 cgroup，scope 会一起死 ⇒ Ⓐ 判红，防住。
- `exit` 只在正常退出写：Ⓖ 验 `stop` 之后 `exit==143` ⇒ 判红，防住。
- 不带环境：Ⓓ 验 `SINGLEFS_HEAVY_TESTS` / `CARGO_HOME` / `RUSTUP_HOME` ⇒ 判红，防住。

### 1　watch 收到 HUP / INT 时停单元：11 格全绿（打中）

watch 的三个 trap（`research/scripts/detached-run.sh:352`「`trap 'detached_run_watch_stopped TERM 143' TERM`」、
`:354`「`trap 'detached_run_watch_stopped HUP 129' HUP`」、INT 同理）都进同一个 `detached_run_watch_stopped`，
里面 `:332`「`if detached_run_break_is watch-stops-units; then`」只有弄坏开关那一条会停单元。但自证 Ⓔ 只测 TERM：
`:566`「`bash "$detached_run_script" start --label "${prefix}e" -- sleep 300`」起一条，`:576`「`kill -TERM "$watch_pid"`」只发 TERM，
再核单元还在。**HUP 与 INT 这两条路，自证一次都没验「不停单元」。**

错实现：在 `detached_run_watch_stopped` 里加一句「`$1 == HUP` 时逐个 `systemctl --user stop` 还在跑的单元」，与 `watch-stops-units` 开关无关。
它让 watch 在收到 HUP 时停掉所有脱离单元——而 HUP 正是登录会话挂断（会话关闭）发给会话内进程的信号，本功能的头号目标就是「会话关闭不影响它」。
这个错实现跑 `--selftest` 仍 11 格全绿：Ⓔ 发的是 TERM、不触发 HUP 分支，别的格不涉及 watch 的信号。满足判据「错实现 + 自证全绿」。
复现 `repro-t4-watch-hup-gap.sh`（注入 HUP-停单元、跑 `--selftest`，期望退 0）放模型目录，主 agent 带前缀跑。
改法：补一格「watch 被 HUP / INT 停时单元与命令都还在」（照 Ⓔ 改信号）。

### 2　次要：Ⓓ 只核 3/5 个带入变量（没单独打中）

`DETACHED_RUN_CARRIED_VARIABLES` 有 5 个（`SINGLEFS_HEAVY_TESTS PATH HOME CARGO_HOME RUSTUP_HOME`），Ⓓ 只验了后 3 个，
没验 `PATH` / `HOME` 带进单元。丢 `HOME` 的错实现 Ⓓ 照过。丢 `PATH` 会让多数命令找不到、别处容易暴露，列为次要一笔，建议 Ⓓ 一并核 `PATH` / `HOME`。

## 没打中的形状

- T2 脚本：读了全份没找到确定的代码级错误；要真跑才判的退出码 / 竞争 / 停不到子进程 / 用户级 systemd 不在，写成 repro 交主 agent（不是「没打中」，是按办法没自己跑）。
- T3 主 agent 经入口藏 ③⑥ 写法：`bash -c`、`systemd-run`、`kill -- -1`、`setsid -f`、`nohup &`、`pkill -f` 全部被 `--` 之后接着剥、照拒，没找到绕过。
- T3 行内赋值翻开关：`BASH_COMMAND_DETECTOR_ALLOW_SUBAGENT_DETACHED_RUN=1 …` / `BASH_COMMAND_DETECTOR_DISABLE_CHECK=1 …` / `export …; …` 子 agent 仍 rc=2，防住。
- T3 嵌套 detached（`start -- bash detached-run.sh start -- …`）：两层都认出、拒。
- 取样范围：T3 共 36 条 hook 输入（5 个 cases-*.json），覆盖前缀 / 包装 / `bash -c` / `source` / `.` / 改名 / 符号链接 / eval / 变量 / python / 行内赋值 / 嵌套 / 内层 ③⑥ / 内层重型；一次抽样。

## 这条腿自己的限度

- 「没打中」只抽了一次样，不支撑否定结论（`.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」）：T2 的「没找到 bug」要再抽一次或主 agent 真跑 repro 才算。
- T3 的 `source` / 改名、T4 的 HUP 错实现，我只喂了 JSON / 读了代码，**没真起单元、没真跑 `--selftest`**（钩子对子 agent 拒 `start` / `stop` / `--selftest`，按派发办法不绕）。功能确认靠模型目录里的两个 repro，主 agent 带前缀跑才坐实。
- 子 agent 闸是劝阻性的护栏不是安全边界：eval / python subprocess 这类通用盲区任何护栏都堵不住，所以 `source` / 改名的改法目标是堵掉「省事又像正常用法」的那几种，可考虑按脚本签名行认而非只认 basename。改法只在我读的这份上推过、被攻过零轮。
- 副本是非 git 仓，`research/results/` 整份覆盖那一类（⑤）判不了，已在 T3 第 3 节标明交主 agent。

## 没做什么

- 不判正推（T1）那几格；不替主 agent 采纳；副本 / JSON 喂出来的退出码不算入库装置上的数。
- 没跑任何单元、`--selftest`、门禁阶段、lint；没编译 Rust。
