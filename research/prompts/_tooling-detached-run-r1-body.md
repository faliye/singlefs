# 长跑脱离会话入口：工具轮第一轮正文（2026-10-01）

<!-- doc-lint:not-numbers T1 T2 T3 T4 -->

## 一、题面

用户 2026-10-01：「脚本后台限制停这个要改一下。后台不能中断我们运行中的脚本。」「后台只能上报，统计目前运行未结束的脚本。」「写完后 走三方捶打代码 从这几次的反馈来看 这部分代码质量一般」。欠账登记在 `records/2026-09-16-subagent拆分提案.md` 第四十二节。工具实现员交了一个脱离入口与三处闸的改动，主 agent 带前缀跑过验收（自证 11 格都对；`DETACHED_RUN_BREAK` 五个开关逐个退 1；`bash-command-detector.sh`、`heavy-test-guard.sh`、`lib_heavy_tests.py` 三份自检退 0，三个新开关各自退 1；跑完 `singlefs-detached-selftest-*` 单元零条；日志 `/tmp/claude-1000/-home-fy5090-code-singlefs/8adc3bc8-1d0b-43bd-9a3f-9e63dae04aaf/scratchpad/detached-accept/`）。这一轮攻代码：它做的是不是题面要的、哪一格会错、自证与开关放不放得过错的实现、闸有没有口子。

## 二、实现今天的样子（主 agent 的观测，2026-10-01）

被判的是脚本、钩子与定义，不是 `crates/`：`grep -rln 'systemd-run\|detached' crates/` 零命中，`crates/` 里没有相关实现。腿读冻结副本 `/tmp/claude-1000/tooling-detached-run-r1/tree/`（sha256 在 `research/prompts/tooling-detached-run-r1-snapshot/files-sha256.txt`）。这一轮的改动只有：新文件 `research/scripts/detached-run.sh`（610 行），实现员自己的两份 diff `tree/detector.diff`（`bash-command-detector.sh`）与 `tree/others.diff`（`lib_heavy_tests.py`、`heavy-test-guard.sh`、`code-tooling.sh`、`agent-common.md`、`main-agent.md`）。`research/prompts/tooling-detached-run-r1-diff.patch` 是这些文件对提交 fcc4d340 的整份 diff，里面还含别的会话早先暂存的改动（广谱变异那一批，已由 `research/prompts/tooling-broad-mutation-r1-main-verification.md` 判过），那部分不在这一轮。实现员报告（线索，结论句不当证据）：`/tmp/claude-1000/m3-r1/detached-run/report.md`。

- `detached-run.sh start --label <名> [--log] -- <命令>`：`systemd-run --user --quiet --collect --service-type=exec --expand-environment=no --unit=singlefs-detached-<名>.service --slice=singlefs-detached.slice --working-directory=$PWD --setenv=…`（带 `SINGLEFS_HEAVY_TESTS`、`PATH`、`HOME`、`CARGO_HOME`、`RUSTUP_HOME`，设了才带）；同名单元在跑退 3；判与起在登记目录的 `flock` 里；单元主进程是一个 bash，写 runner-pid、起子 bash 写命令 pid 再 `exec env -- "$@"`，命令退出后先写临时文件再 `mv` 成 ended / exit；它 trap TERM。登记目录 `${XDG_STATE_HOME:-~/.local/state}/singlefs/detached-runs/<名>/`，同名旧登记挪成 `<名>@<秒数>`。
- `list`：还在跑 / 已结束 / 没留退出码三类逐条列。`watch [--every 秒]`：定时 list；全部结束退 0 并报各条退出码；TERM / INT / HUP 先打最后一次清单再退 143 / 130 / 129，只停它自己起的 sleep。`stop <名>`：只按登记的单元名 `systemctl --user stop`，回读状态、至多等 30 秒退出码。
- `bash-command-detector.sh`：③ 放行 `detached-run.sh start`，`--` 之后那条接着剥、照判；子 agent（hook 输入里有 `agent_type`）的 start / stop 退 2；list / watch 谁都放。开关 `BASH_COMMAND_DETECTOR_ALLOW_SUBAGENT_DETACHED_RUN`、`BASH_COMMAND_DETECTOR_OPAQUE_DETACHED_RUN`。
- `lib_heavy_tests.py`：新函数 `command_under_detached_run`，`command_under_launcher` 先调它，重型闸据此看穿 `start -- <命令>`；开关 `LIB_HEAVY_TESTS_BREAK=detached-run-opaque`。`heavy-test-guard.sh` 补自检 6 格。
- `code-tooling.sh` 的自证表加一条；`agent-common.md`「长活可以等」与「执行前拒绝的写法」③、`main-agent.md`「派出去之后」各加一句。
- 已知事实：本机 `Linger=no`；主 agent 打算在这一轮判完之前就用它跑两台广谱变异（本机直接起，第二台经 `research/scripts/multi-host-run.sh --peer` 整个挂进单元）。

## 三、格

### T1　题面对实现（云端正推）
逐句核用户三句话与第四十二节的「去向」：入口起的命令真的不随调用方、不随会话后台时限、不随会话关闭停吗（各自靠什么、有没有反例）；`watch` 真的「只上报」吗（任何路径上会不会给被观察单元或它的命令发信号、会不会因为自己被停而让单元收到什么）；`list` 报的「还在跑、没结束」与真实状态在哪些情形下不一致（单元已卸、命令进程被别人杀、runner 先死、登记目录被删、同名重起）。三处规则与定义的新句子照 `.claude/singlefs-ai-sop/rules/rules-discipline.md` 只写怎么做了没有、与原有句子矛不矛盾。

### T2　脚本哪一格会错（云端攻方）
在副本上真跑造反例（只起 sleep / true / bash 这一类，单元名用你自己的前缀，收尾停干净、核零残留）：① 退出码：命令被 SIGKILL、OOM、单元被 `systemctl stop`、runner 被杀、机器级别的 `--collect` 时序下，exit 文件写不写得出、写出的对不对、list 怎么报；② 竞争：两个 start 同名同时起、start 与 stop 交错、watch 与 start 交错；③ 参数与环境：label 带特殊字符、命令带引号 / `$` / `%` / 换行、`--` 之后开头的 `名=值`、工作目录带空格、没设的变量、`PATH` 里有相对路径；`--force` 被 preflight 从 `--` 之后摘掉的后果；④ `stop` 停不到命令（命令自己 fork 出去、改进程组）时单元与退出码怎么样；⑤ 用户级 systemd 不在（`XDG_RUNTIME_DIR` 缺、D-Bus 连不上）时报什么、会不会退回在调用方树里跑。

### T3　闸的口子（云端攻方，同一条腿、另成一节报告）
① 子 agent 能不能经别的写法起脱离单元而不被拒（`bash -c`、`eval`、变量里拼脚本路径、拷一份改名、符号链接、`python3 -c subprocess`、直接 `systemd-run`、`--` 之后再套一层 detached-run）；② 重型闸：`start -- <重型命令>` 的各种写法（前缀写在外面 / 里面 / 环境文件里、命令写进脚本再起、`--log` 的值长得像命令）有没有绕过前缀或角色判定的；③ 主 agent 经入口起的命令里藏放出追踪、终止进程（③⑥）的写法，`--` 之后那条接着剥是不是每一种都认。

### T4　自证与开关（云端攻方，同一条腿、再另成一节报告）
对自证 11 格与新加的钩子自检格，各写至少一种「看起来对、其实错」的实现（不是那五个与三个弄坏开关），看放不放得过：例如 start 起在 `systemd-run --scope`（仍在调用方 cgroup 外但随调用方退出？）、watch 收 TERM 时只停一个单元、exit 只在正常退出时写、环境只带 `SINGLEFS_HEAVY_TESTS`。放得过的写出最短的那一种。

## 四、分工

| 腿 | 格 | 立场 | 理由 |
|---|---|---|---|
| 云端正推（Sonnet） | T1 | 题面与规则对实现 | 第一轮，没有前一轮判决可复核，不派辩方 |
| 云端攻方（Opus） | T2、T3、T4 | 在副本上真跑造反例 | 要真跑；三节分开报 |

## 五、跑前写死的判据

- T1 或 T2 造出入口起的命令在调用方被停、后台时限、会话关闭三者任一之下被停 ⇒ 打中，必须改，改完之前不用它跑长活。
- T1 或 T2 造出 `watch` 在任何路径上停了或发信号给被观察单元 ⇒ 打中，必须改。
- T2 造出退出码丢失、写错、或 `list` 把已结束报成还在跑（或反过来） ⇒ 打中，改法并进下一批工具改动。
- T3 造出子 agent 起得出脱离单元、或 `start -- <重型命令>` 绕过前缀与角色 ⇒ 打中，必须改。
- T4 写出一种错实现、自证与钩子自检全绿 ⇒ 打中，补自证格或开关。
- 一条腿报「没打中」的格只抽了一次样，不支撑否定结论。

交付：腿的报告写 `research/prompts/tooling-detached-run-r1-<模型>-output.md`（`-sonnet-` / `-opus-`），分段落盘，每次写进文件不超过 150 行，第一段排他新建，回复只写短句；攻方的副本改法、复现脚本与日志放 `research/prompts/tooling-detached-run-r1-opus-model/`。草稿与临时文件放腿自己的目录 `/tmp/claude-1000/<腿名>/`。引规则写规则文件名加那份文件自己的行号。不在主仓里改任何文件，副本拷进自己的草稿目录再改。
