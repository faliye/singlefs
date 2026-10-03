# 工具轮第二轮 攻方腿（Opus）报告：V2 改法本身 / V3 第一轮没攻到的面 / V4 自证与开关（2026-10-01）

立场：攻方。被判文件读冻结副本 `/tmp/claude-1000/tooling-detached-run-r2/tree/`（`sha256sum -c` 7 份全 OK；主仓这 5 份脚本与钩子此刻与副本逐字节相同，下文行号两处一致）。
钩子只喂 JSON、不执行；要真起单元的写成复现脚本交主 agent 带前缀跑（本腿一份都没跑）。不起单元的模型（替身 systemctl、只跑 list / watch）本腿跑过，原样输出在模型目录。

## 复跑命令与文件

模型目录 `research/prompts/tooling-detached-run-r2-opus-model/`。不起单元、本腿跑过的（参数是 detached-run.sh 的路径；钩子探针的树是「冻结副本 7 份 + 主仓其余 `.claude` 与 `research/scripts`」，old 树换成实现员的备份 `orig/`）：

```
python3 feed.py <树> cases-v2a.json            # V2① 探针；cases-regress.json 是 148 条回归语料；cases-v4hooks.json 是 V4 钩子那一半
bash model-w1-watch-record-deleted.sh <detached-run.sh>
bash model-w2-start-failed-but-running.sh <detached-run.sh>
bash model-f4-forged-token.sh <detached-run.sh>
bash model-v3-runner-writes-fail.sh <detached-run.sh>
bash model-v3-list-scale.sh <detached-run.sh> 10 50 200 1000
bash model-v4g-watch-unknown-then-ended.sh <detached-run.sh>   # 对 control 与 g 两份各跑一次（v4_mutations.py 生成）
```

真起单元、交主 agent 带前缀跑（单元名都是 `singlefs-detached-opusr2-*`，登记目录都是临时的，收尾按单元名停、打残留数）：
`repro-w1-watch-record-deleted.sh`、`repro-w2-start-failed-but-running.sh`、`repro-v3-log-unwritable.sh`、`repro-b1-force-inner.sh`、
`repro-v4-selftest-wrong-impls.sh`（起自证）、`repro-v4-hook-selftests.sh`（不起单元，跑钩子 `--selftest`）；参数都是仓根，期望在 `repro-expected.txt`。

sha256（`sha256sum` 原样）：

```
e1e3cfe60ec7e3b558b702e005e86636a6c276f339deafd0acc87137e40c8195  cases-regress.json
528286f6b530cd2a58ab2bceee1ef911a74eeac929750348a9041f25957fd9c9  cases-v2a.json
b5e2449940990f6f7fce695ab138424dd4b8b7b44ecac3d4918f1a865006e3d8  cases-v4hooks.json
11cf678e93618cd9fc78c57df436e08856fa4b5e52f0b92647a6d0fa76cc2777  feed.py
7c0672b08988a3367859780e0accb11b126595c012999808e0006bc26fe11986  hook-probes-v2.txt
47b0974ace4c7b5f1823520e23732eaffae0aa8689e12287c4e23a35d3a9c636  model-f4-forged-token.sh
c42902f9f8c31ebd17c180834543d936f3f12d97632a6d1f973f1bb20c29ac53  model-outputs.txt
4930e2daec9f30bc9dec1d2b3280a5df6e51587d81a01dae787f7426cc8a5143  model-v3-list-scale.sh
504703e776d157f1f0e45d9d2daed9cef9e7d177d050ff24db3353efa1f65949  model-v3-runner-writes-fail.sh
848e149c9cd6ebcb3e3f6f5942e78b7d4e0e2e1570d20994fa9fa11dea09eb24  model-v4g-watch-unknown-then-ended.sh
c92b25f2091d32e9a78bb52a368c6ee72e0eeeea2d6a0e349894e72ede4bc253  model-w1-watch-record-deleted.sh
6be23eefe317db5e5fbce67019cafa9975d021c1adca14e6cc978293cb5cacd5  model-w2-start-failed-but-running.sh
40435e1fdf5d67c8b7dfb550c0505dba6ffd2a3110eccbdf4b393403e73a3e9b  repro-b1-force-inner.sh
82cfd27a4438d5a624e9cf5bb9c4e20ca3318679465110d62c706b7b210ad57f  repro-expected.txt
d7b2e849955462f58be3ed4fb632fec7114bdeb86e5e9017a092746ed1b143b4  repro-lib.sh
7ad3427f1816af7ae7d3163d94ca83edf15880c9581d050ccf5ea168c0551378  repro-v3-log-unwritable.sh
59b4c7cb5b46251dca65838eb49a6e22a13514c5f03456037ff47e8c729e50f1  repro-v4-hook-selftests.sh
4926bd533ae187968c65b759459f16c95312de8f29f7393057d60768ba068203  repro-v4-selftest-wrong-impls.sh
8c5794c37cbcfd3d739fd0d556b8038617a67f0e157dac60440948e0d82e36f9  repro-w1-watch-record-deleted.sh
4d48744bcefc9e95da589376bd964ac52c7188f5bb383bb0f2d44ee0ea499653  repro-w2-start-failed-but-running.sh
3ca9a62892665d46ef71964c061278dfde24bd71069505ac81298022c52ab460  shim-lib.sh
1652f2ad888b07aadc3de91534667ec075304c0376dc1313b9794011e15beefd  v4_mutations.py
```

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| V2① 回归 | 没打中 | 148 条回归语料喂新旧两版钩子，判定只在 F1 / F5 有意改的那几行不同（`hook-probes-v2.txt`） |
| V2① 签名行伪造误拒 | 没打中 | 主仓里开头带签名行的只有 detached-run.sh 自己；主 agent 合法用法（worktree 里主仓绝对路径、`nice`、`cd … &&`、带日期的名）都放 |
| V2① `--force` 摘法对不上 | **打中**（字面落在第一轮 T3 的「绕过前缀与角色」，不在本轮 V2 两个分句里） | preflight 摘掉每个 `--force`，共用的 `detached_run_start_command` 不摘：主 agent 不带前缀 `start --label p -- --force cargo test --all` 重型闸放（rc=0）；`-- --force bash -c 'sleep 100 & disown'` ③ 放 |
| V2① 认不出清单之外的写法 | **打中**（都是旧版就有的，不是 F1 回归） | 子 agent `time -p bash …detached-run.sh start`、`env -S '…'`、`builtin source`、`bash --rcfile F`、`bash --`、`function f {…}`、`cp X{,.bak}`、`//tmp/拷贝`、第二行 500 层嵌套让钩子 RecursionError 后放行，九种都 rc=0；同一批口子里 `time -p cargo test --all`（主 agent 不带前缀）与子 agent `time -p cargo test -p singlefs-checker-tier` 重型闸也放 |
| V2② 登记被删 | **打中** | watch 观察到在跑的那条、登记目录随后被删：watch 退 0、报「单元已不在」「都结束了」，同一屏上一行 list 写着它 active（模型量过） |
| V2② start-failed 而单元在跑 | **打中**（触发是推的） | list 见 start-failed 就不问单元状态，列「没起成…命令一行都没跑」；watch 一轮退 0「观察到 0 条」（模型量过；systemd-run 报错而单元照起的触发没量） |
| V2② 查不到 / 同名重起 | 没打中 | Ⓠ、Ⓡ 读到的形状都对；list 先读 exit 后问状态，窗口里会把刚写完 exit 的报成「没留退出码」，watch 收尾时重读、报对（推的，轻） |
| V2③ F4 令牌 | **打中字面**，按「判别子观测不到」处理 | 自己在临时登记目录放 `.selftest-token`、设同值变量，`DETACHED_RUN_BREAK` 在自证外生效（模型量过）；同一用户造得出与自证一模一样的两样东西，脚本分不出来 |
| V2④ F5 | 没打中判据，记三笔 | 旧版两单元的真实时限是 1min 30s（只读 `systemctl --user show` 读到），stop 报「停机时限 ? 秒」；写不进 exit 时 stop 把原因写成「被 KILL」；D 状态下 KILL 段再等一个时限，合计可过 240 秒（推的） |
| V2⑤ F6 | 没打中 | 读不到就警告、照起；`loginctl` 对没登录又没 linger 的用户退 1 会被写成「读不到」而不是「没开」，但那时 preflight 先拒（用户级 systemd 不在） |
| V3 `--log` 写不进 | **打中** | `--log` 指到写不进的目录：start 退 0、打「✓ 起了…（已经跑完）」，命令一行没跑，登记 exit=1（模型量过单元那段；整条 start 写成复现） |
| V3 写满盘 | **打中**（机制量过，盘满是推的） | 单元那个 bash 写不进登记就丢了 exit，list / watch 把原因报成「主进程被 KILL、用户级 systemd 退出或机器重启」 |
| V3 登记很多 | 量了，不判打中 | list 每条约 10 ms、3 行、330–450 字节，1000 条 11 秒；watch 每轮打整张清单，挪走的旧登记从不清 |
| V3 watch 被 SIGKILL / 两个 watch | 没打中 | 被 KILL 留一个 `sleep <every>` 孤儿活到点；再起丢掉之前的观察，靠第一轮清单补；两个 watch 都只读、互不影响 |
| V4 detached-run 自证 | **打中**（按代码推，复现交主 agent） | 五种错实现自证 21 格照绿：读不到 Linger 不警告；默认 180 秒不设进单元；stop 不等停机任务；查不到的判挪到挪走旧登记之后；watch 不记查不到的（g 的后果模型量过） |
| V4 钩子自证与样本 | **打中**（样本量过，自证按代码推） | 子 agent 闸只认 cp / ln、认法不跟 cd、重型闸不认 heredoc 写出的拷贝：`cases-f1.json` 20 行三种都照旧，自证里没有 install、mv、cd、heredoc 那几格 |

跑前判据对照（正文第五节）：V2 第一条两个分句里「子 agent 用一种不在清单里的写法起得出单元」中（九种）；「主 agent 合法用法被误拒」没中。
V2 第二条「watch 把在跑的报成结束」中（登记被删、start-failed 而在跑）。V2 第三条「令牌在自证之外拿得到」中（伪造），「合法同名变量被误拒」归 V1、本腿没攻。
V4「错实现、自证与钩子自检全绿」：钩子样本那一半量过；自证那一半是推的，等主 agent 跑 `repro-v4-*.sh` 坐实。

## V2　改法本身

### ① F1 认法与共用的 start 参数切法

**回归：没打中。** 把第一轮攻方的 44 条、实现员的 20 条样本与 84 条常见写法（cargo 各档、capped / 内存包装、source、nohup、systemd-run、kill、带前缀与不带、三种身份）共 148 条，
喂进 old（实现员开工时的备份）与 new（冻结副本）两棵树，判定只差在 F1 / F5 有意改的那些行（source、改名拷贝、符号链接、`--stop-timeout`、heredoc 写出带签名行再起）。原样在 `hook-probes-v2.txt` 第二段。
`lib_shell_words.py` 这次只加函数与常量、没改原有函数；新函数名在 `.claude/hooks/` 与 `research/scripts/` 里没有第二处 def（`grep -rn 'def detached_run_script\|def detached_run_start_command\|def carries_detached_run_signature'` 只命中它自己）。
用到它的还有 `research/scripts/agent-watch.py`（经 `classify_process`），这次只多传了当前目录。

**签名行伪造误拒：没打中。** 仓里行首带 `# detached-run-script-signature:` 的只有 `research/scripts/detached-run.sh` 本身（`grep -rlE` 全仓，排除 target、.git）。
造一份带签名行的别的脚本当然会被当成 detached-run，但那不是合法用法。主 agent 的合法写法 `cd /tmp && nice -n 19 bash /home/fy5090/code/singlefs/research/scripts/detached-run.sh start --label m-2026-10-01 -- sleep 1` 两道都退 0。
子 agent `bash -n`、拷贝后 `bash -n`、`list` 都退 0。

**打中一：preflight 摘 `--force`，共用的切法不摘。** 脚本先经 preflight：`.claude/singlefs-ai-sop/scripts/preflight.sh:31`「if [[ "$preflight_argument" == --force ]]; then PREFLIGHT_FORCE_GIVEN=1; else PREFLIGHT_ARGUMENTS+=("$preflight_argument"); fi」，
每个 `--force` 不论在哪都摘掉（实测 `detached-run.sh list --force` 退 0、`list extra` 退 2，`hook-probes-v2.txt` 末段）。
共用的切法只在子命令前与选项位上跳过它：`.claude/hooks/lib_shell_words.py:519`「while position < len(arguments) and arguments[position] == "--force":」；
遇到 `--` 就把后面原样交出：`.claude/hooks/lib_shell_words.py:527`「return arguments[position + 1:] or None」。于是（主 agent、不带前缀，`cases-v2a.json`）：

| 写法 | 脚本实际跑的 | heavy | bash |
|---|---|---|---|
| `start --label p -- cargo test --all`（对照） | 同左 | 2 | 0 |
| `start --label p -- --force cargo test --all` | `cargo test --all` | **0** | 0 |
| `start --label --force p -- cargo test --all` | `--label p`，同上 | **0** | 0 |
| `start --label p --log --force -- -- cargo test --all` | `--log --`、命令 `cargo test --all` | **0** | 0 |
| `start --label p -- --force bash -c 'sleep 100 & disown'` | `bash -c …disown` | 0 | **0**（对照 2） |
| `start --label --force p -- bash -c 'sleep 100 & disown'` | 同上 | 0 | **0** |

第二行：钩子把 `--force` 当命令词，看不到后面的 cargo；第三行：`--label` 吃掉 `--force`，`p` 成了认不出的选项，整条不当 start 剥。
`kill -- -1` 那一类不受影响（⑥ 在全文上找，照拒）。旧版钩子同样放（old 树同判），不是 F1 引进的，第一轮 T2「`--force` 剥除是文档化行为」没看钩子这一侧。复现 `repro-b1-force-inner.sh`（只跑 `bash -c 'echo ran'`）坐实「`-- --force` 后面那条照跑」。

四句：① 不分辨臂，是闸的口子；② 判别子看得到：钩子拿到整条文本，`--force` 在里面，照 preflight 的语义先把参数里每个 `--force` 去掉再切就对上了；
③ 本轮 V2 第一条的两个分句都不中（不是误拒、不是子 agent——子 agent 那条 bash 闸照拒），落在第一轮 T3 判据「`start -- <重型命令>` 绕过前缀与角色」那一句；④ 正文给的去向只有「改」，没有互斥改法要逐格核。

**打中二：认不出清单之外的九种写法（子 agent 起得出单元）。** `cases-v2a.json`，clerk 身份，bash 闸都退 0（对照 `bash …detached-run.sh start` 退 2）；每种在本机用一个只打参数的探针脚本核过 bash 真会去执行它（`/tmp/claude-1000/tooling-detached-run-r2-opus/sem/`）：

| 写法 | 卡在哪 |
|---|---|
| `time -p bash research/scripts/detached-run.sh start …` | `time` 是关键字（`.claude/hooks/lib_shell_words.py:40`「COMMAND_KEYWORDS = {"if", "then", "else", "elif", "do", "while", "until", "!", "{", "}", "time"}」），后面的 `-p` 被当命令词 |
| `env -S 'bash research/scripts/detached-run.sh start …'` | env 的 `-S` 不当带值选项（`.claude/hooks/lib_shell_words.py:45`「"env": {"-u", "--unset", "-C", "--chdir"},」），整串成了一个命令词 |
| `builtin source research/scripts/detached-run.sh start …` | `builtin` 不是前缀 |
| `bash --rcfile /dev/null research/scripts/detached-run.sh start …` | `--rcfile` 的值被当脚本（`--init-file` 同） |
| `bash -- research/scripts/detached-run.sh start …` | `.claude/hooks/lib_shell_words.py:412`「if not argument.startswith(("-", "+")) or argument in ("-", "--"):」把 `--` 自己当脚本 |
| `function f { bash research/scripts/detached-run.sh start …; }; f` | `function` 不是关键字 |
| `cp research/scripts/detached-run.sh{,.bak} && bash research/scripts/detached-run.sh.bak start …` | 花括号没展开，cp 只有一个操作数（`.claude/hooks/bash-command-detector.sh:534`「if target_directory is None and len(operands) < 2:」） |
| `cp … /tmp/opusr2-x.sh && bash //tmp/opusr2-x.sh start …` | `os.path.normpath` 保留开头的 `//`，与记下的 `/tmp/opusr2-x.sh` 对不上 |
| 第一行 `bash …detached-run.sh start …`，第二行 `echo "` 加 700 个 `$("` | 切词递归过深抛 RecursionError，`.claude/hooks/bash-command-detector.sh:2870`「print(f"  ! bash-command-detector.sh 没判成 detached-run.sh 的起法（{error!r}），这条命令照常执行", file=sys.stderr)」；bash 先执行第一行再在第二行报语法错（核过） |

同一批口子打到重型闸（`cases-v2a.json` B6–B8）：主 agent 不带前缀 `time -p cargo test --all` heavy=0、`bash -- research/scripts/detached-run.sh start --label p -- cargo test --all` heavy=0、
子 agent `time -p cargo test -p singlefs-checker-tier` heavy=0；RecursionError 那条两道钩子的每一格都「没判成…照常执行」（本腿逐行看过 stderr：bash 闸 8 行、heavy 1 行）。
这些 old 树全部同判：是 `lib_shell_words.py` 早就有的盲区，不是 F1 的回归，但都不在实现员「认不出」那 9 类里（`time -p` 与第 5 类的 `/usr/bin/time` 不是同一个东西：后者是外部命令，前者是关键字带选项）。
自然程度不一样：`time -p`、`cp X{,.bak}` 是常见手法；`env -S`、`--rcfile`、`function`、`//`、500 层嵌套是有意绕。

四句：① 不分辨臂；② 判别子看得到（都在命令文本里；RecursionError 那条闸自己知道没判成）；③ 中 V2 第一条第二分句「子 agent 用一种不在…清单里的写法起得出单元」；
④ 去向只有「改」。改法（只在我的模型上量过、被攻过零轮——其实一格都没实现，下表全是推的）：

| 改法 | 修哪几格 | 量过 / 推的 |
|---|---|---|
| `detached_run_start_command` 先删掉参数里每个 `--force` 再切（照 preflight） | `--force` 三行重型、两行 ③ | 推的 |
| `skip_prefixes` 认 `time` 的 `-p` / `--`、`builtin`、`function` 关键字，env 认 `-S` / `--split-string` 并按词切开 | `time -p`、`builtin`、`function`、`env -S`，连同重型闸的 `time -p cargo` | 推的 |
| `shell_invocation` 认 `--` 为选项结束、`--rcfile` / `--init-file` 带值 | `bash --`、`--rcfile`，连同重型闸的 `bash --` | 推的 |
| 认拷贝时展开 `{a,b}`；路径比对前把开头的 `//` 并成 `/` | 花括号、`//` | 推的 |
| 切词抛异常时对子 agent 判拒（或 detached_run_refusal 那一格对含 detached-run 字样的命令判拒），不「照常执行」 | RecursionError | 推的；对主 agent 照放要另想 |

### ② F3：查不到、登记被删、start-failed、同名重起

**打中三：登记被删之后，watch 把还在跑的报成结束（W1）。** watch 每轮只看 list 给的「还在跑」与「查不到」；登记目录没了，那条就不在任何一类里，
还在跑的条数归零，watch 走收尾。收尾找观察到的那一次：`research/scripts/detached-run.sh:433`「if detached_run_break_is watch-keys-by-path || [[ "$(detached_run_field "$record" started)" == "$started" ]]; then」
对不上（登记没了、读出空），去找挪走的 `<名>@<秒数>` 也没有，最后落到 `research/scripts/detached-run.sh:466`「echo "  · $label 没留退出码（单元已不在：主进程被 KILL、用户级 systemd 退出或机器重启）$moved"」。
模型（替身 systemctl 让单元一直 active，`model-w1-watch-record-deleted.sh`，原样在 `model-outputs.txt`）：

```
watch 退出码 0；替身单元仍 active：active
  · 登记里没有  单元 singlefs-detached-opusr2w1.service（loaded / active / running）：它的登记被删了、或在别的登记目录里；…
  · opusr2w1 没留退出码（单元已不在：主进程被 KILL、用户级 systemd 退出或机器重启）（之后同名又起过一次，这一次的登记挪到了 …/state/opusr2w1@1790821017）
  ✓ detached-run.sh watch：观察到的 1 条都结束了，各自的退出码在上面
```

三句都错：单元在跑、没有同名重起、那个挪走的路径不存在。自证 Ⓧ 只核 list 的那一行（`research/scripts/detached-run.sh:856`「"$(grep -q "登记里没有  单元 $unit_x" "$work/x.out" && echo 0 || echo 1)"」），不跑 watch。
真单元复现 `repro-w1-watch-record-deleted.sh`。

四句：① 不分辨臂；② 判别子看得到：watch 观察时读得到单元名（登记里的 unit），同一轮 list 也从 list-units 读到它 active，收尾前按单元名问一次 ActiveState 就分得开；
③ 中 V2 第二条「watch 仍把在跑的报成结束」；④ 去向只有「改」。改法（推的）：观察表按单元名记，登记不见了的按单元名接着问，单元 active 就算还在跑、清单里写明「登记没了」。

**打中四：start-failed 而单元在跑（W2，触发是推的）。** list 见 start-failed 就不问单元：`research/scripts/detached-run.sh:237`「elif [[ -f "$record/start-failed" ]]; then」排在问状态那一支之前；
这条登记的单元名在 registered_units 里，也不进登记里没有那一类，因为 `research/scripts/detached-run.sh:290`「[[ "$name" == "${DETACHED_RUN_UNIT_PREFIX}"*.service && -z "${registered_units[$name]:-}" ]] || continue」。
模型（`model-w2-start-failed-but-running.sh`）：

```
watch 退 0；替身单元：active
  · 没起成  opusr2w2  单元 singlefs-detached-opusr2w2.service：2026-10-01 systemd-run 退 1，命令一行都没跑  …
  ✓ detached-run.sh watch：开跑时就没有还在跑的，观察到 0 条
```

触发要 systemd-run 报失败而单元其实起了：D-Bus 调用超时（默认 25 秒）而 systemd 随后照起，是推的、没量；`repro-w2-start-failed-but-running.sh` 用「先转给真的 systemd-run、再退 1」的替身造出这个后状态。
四句：② 看得到（systemd-run 失败之后、list 判 start-failed 之前问一次 ActiveState 就分得开）；③ 中 V2 第二条；④ 改法（推的）：start 在 systemd-run 失败后先问状态，active 就不写 start-failed；list 对 start-failed 的也问状态。

**没打中：查不到、同名重起。** Ⓠ 的拒与不退、Ⓡ 的按「目录@秒数」找旧一次，读到的形状都对。有一个轻的窗口（推的、没量）：list 先读 exit（`research/scripts/detached-run.sh:233`「rc="$(detached_run_field "$record" exit)"」）、后问状态，
单元在两步之间写完 exit 退出，这一轮清单把它列成「没留退出码」；watch 收尾时重读 exit，报对。同一秒同名重起两次、两次轮询之间跑完的短命令，实现员已列为没修。

### ③ F4 令牌

**打中字面：令牌在自证之外造得出来。** 认法只比两样：`research/scripts/detached-run.sh:133`「[[ -n "${DETACHED_RUN_SELFTEST_TOKEN:-}" && -f "$detached_run_state/.selftest-token" ]]」再比内容；
登记目录由 `DETACHED_RUN_STATE_DIRECTORY` 定，谁都能指到自己建的目录。模型（`model-f4-forged-token.sh`，只跑 list，替身让 show 失败）：不设开关列「查不到」；设开关不带令牌退 4；
自己放一个 `.selftest-token` 再设同值变量，`query-failure-as-gone` 生效，同一条登记改列「没留退出码」、退 0。

四句：① 不分辨臂；② **判别子观测不到**：同一用户造出的目录与变量，与自证造的逐字节一样，脚本在做决定的那一刻分不出来（`.claude/singlefs-ai-sop/rules/evidence-discipline.md:122`「| **判别子观测不到** |」那一行：改判据）；
③ 中 V2 第三条前一分句；④ 正文给「改」，但没有改法能让脚本分出同用户的有意伪造。建议改判据为「开关误设（忘了 unset、从外层环境带进来）时不生效」——这一条现在成立：
真登记目录里没有令牌文件，`--selftest` 把登记目录换成自己的临时目录；自证起的每个子调用都在带令牌的目录里（Ⓠ、Ⓢ、Ⓡ、Ⓧ、Ⓣ、Ⓛ 各自 `detached_run_selftest_state_directory`，Ⓐ 的调用方单元经 `--setenv` 带令牌），开关照生效（读代码，没跑自证）。

### ④ F5 停机时限

没打中判据，记三笔：
- 旧版起的两条在跑单元，只读 `systemctl --user show -p TimeoutStopUSec` 读到 `1min 30s`（用户级默认也是 `1min 30s`），登记里没有 stop-timeout；新 stop 停它们时消息与 stop-killed 写「停机时限 ? 秒」，真实时限其实读得到。
- stop 读不到 exit 就写 stop-killed 并说「被 KILL」（`research/scripts/detached-run.sh:566`「detached_run_write_field "$record" stop-killed」那一行），而写不进 exit 还有一种原因：单元那个 bash 写不进登记（见 V3 写满盘），这时 stop 的说法错，连 stop-killed 自己也写不进，stop 照打「登记里记了 stop-killed」。
- systemd 对服务到点发 KILL 之后，再给一个同样长的时限等进程消失（推的，没量）：进程卡在 D 状态时 stop 至多约 360 秒，过了会话前台的 240 秒。

### ⑤ F6 Linger

没打中。读不到时警告一行、照起，与文件头一致。本机只读：`loginctl show-user <自己> -p Linger --value` 打 `yes`；对没登录又没 linger 的用户（root）退 1、打「not logged in or lingering」，
脚本会把它写成「读不到」而不是「没开」，但那种处境下用户级 systemd 不在，preflight 的 `systemctl --user show-environment` 先拒（退 78），走不到这一句。`loginctl` 卡住时由 D-Bus 的默认超时兜住（推的）。
自证 Ⓛ 只用替身打 `no` / `yes`，「读不到」那一支没有格——见 V4-a。

## V3　第一轮没攻到的面

### `--log` 指到写不进的地方：打中

start 只核日志所在的目录在不在：`research/scripts/detached-run.sh:328`「[[ -d "$(dirname -- "$log")" ]] || detached_run_usage_error "日志 $log 所在的目录不在"」。
目录在而写不进（权限、`--log` 本身是个目录），单元那个 bash 的重定向失败，命令根本不起：`research/scripts/detached-run.sh:120` 那一行以「"$record" "$@" >> "$log" 2>&1 < /dev/null」收尾，失败时 rc=1、不写 pid，接着照写 exit=1。
start 回读时只要 exit 在就算起了：`research/scripts/detached-run.sh:414`「if [[ -z "$pid" && ! -f "$record/exit" ]]; then」不进，落到 `research/scripts/detached-run.sh:419`「echo "  ✓ 起了「$label」」，退 0。
模型（`model-v3-runner-writes-fail.sh` 乙，把单元那段 bash 原样抠出来在本进程跑）：

```
单元那个 bash 退 1；登记里 pid：没有  exit：1；命令跑没跑（marker）：没跑；它的报错：detached-run-runner: line 4: …/ro/run.log: Permission denied
start 回读（照第 414–419 行）：✓ 起了…命令进程 （已经跑完），退 0
```

整条 start 的复现 `repro-v3-log-unwritable.sh`（期望 start 自己的输出里另有一行写开头时的 Permission denied，这是唯一的提示；单元里那句报错只进 journal）。
改法（推的）：start 先 `: >> "$log"` 试写，写不进按用法错退 2；回读时有 exit 而没有 pid 的，判「命令没起成」、退 1。
`--log` 指到已存在的文件：照追加，两条共用一个日志时 list 的「日志最后一行」是另一条的，轻，不判。

### 写满盘 / 登记写不进：打中（机制量过，盘满本身是推的）

单元那个 bash 写退出码只有一次、不核结果：`research/scripts/detached-run.sh:126`「printf '%s\n' "$rc" > "$record/.exit.tmp" && mv -f -- "$record/.exit.tmp" "$record/exit"」。
盘满时 printf 写不进、`&& mv` 不跑，exit 就没了（推的，没在满盘上量）；模型用「命令退出前把登记目录改成只读」走同一条路（甲）：命令正常退 3，登记里没有 exit，list 报
「没留退出码 … 主进程被 KILL、用户级 systemd 退出（没开 linger 时最后一个登录会话结束）或机器重启」——原因报错了，退出码丢了。stop 那一侧见 V2④ 第二笔。
命令自己把日志写满盘（日志默认在登记目录里，同一个文件系统）正好走这一条。改法（推的）：写不进时往 journal 打一行带退出码的、退出码作为单元自己的退出状态留着（runner 已经 `exit "$rc"`，list 可以再读 `ExecMainStatus`，前提是单元没被 `--collect` 收掉，所以还要换收法）。

### 登记很多：量了，不判打中

`model-v3-list-scale.sh`，临时登记目录造 N 条，list 一次（真 systemctl 只读）：

| N | 已结束：耗时 / 行 / 字节 | 没留退出码：耗时 / 行 / 字节 |
|---|---|---|
| 10 | 121 ms / 33 / 4 093 | 340 ms / 33 / 5 293 |
| 50 | 583 ms / 153 / 17 093 | 457 ms / 153 / 23 093 |
| 200 | 1 620 ms / 603 / 66 249 | 2 283 ms / 603 / 90 249 |
| 1000 | 11 415 ms / 3 003 / 329 455 | 10 264 ms / 3 003 / 449 455 |

（机器上同时有别的 cargo 在跑，耗时只看量级。）每条约 10 ms、3 行。watch 每一轮把整张清单打一遍（`research/scripts/detached-run.sh:508` 起的循环），同名重起挪走的 `<名>@<秒数>` 从不清；
50 条历史时每轮约 17–23 KB，默认 300 秒一轮跑 10 小时约 120 轮、2–2.8 MB（算的）。会话里读 watch 输出要读进上下文的话，这是个实打实的量；正文没给判据，不判打中。改法（推的）：watch 的定时轮只打还在跑与查不到的，结束时才打全表。

### watch 被 SIGKILL、两个 watch 同时跑：没打中

- 被 KILL：它起的 `sleep <every>` 没人停，PPID 变 1、活到点（实测 `--every 40`，`model-outputs.txt` 末段；本腿随后 `proc.py stop` 停了它）。再起的 watch 丢掉之前的观察表：被 KILL 到再起之间结束的那几条，只在第一轮清单里以「已结束 退出码」出现，收尾那段不报——信息还在，轻。
- 两个 watch：watch 与 list 不写任何文件（读代码），互不影响。
- 命令把 stdout 关掉：stdout 是单元那个 bash 给它接的日志，关掉只是不再写，runner 照写 exit（读代码）。

## V4　自证与开关：看起来对、其实错的实现

### detached-run.sh 自证（21 格）放得过的五种（按代码推，`repro-v4-selftest-wrong-impls.sh` 交主 agent 坐实）

改法写在 `v4_mutations.py`（每种先把自证前缀换成 `opusr2-<pid>-`，单元名落在 `singlefs-detached-opusr2-*`）；本腿只核过六份都改得上、`bash -n` 都过，没跑 `--selftest`。

| 名 | 错在哪 | 为什么自证看不见 | 后果 |
|---|---|---|---|
| a 读不到 Linger 不警告 | `research/scripts/detached-run.sh:175` 以「echo "  ! 读不到 loginctl show-user $user 的 Linger」起头的那句换成 `:` | Ⓛ 只放两种替身：`research/scripts/detached-run.sh:880`「detached_run_selftest_shim "$work/shim-linger-no" loginctl "echo no"」与 `:881` 打 yes 的那一个 | loginctl 不在或报错时一声不吭 |
| b 默认 180 秒不设进单元 | 只有明写 `--stop-timeout` 才走 `research/scripts/detached-run.sh:389`「then properties+=("--property=TimeoutStopSec=$stop_timeout"); fi」，登记照写 180 | Ⓣ 只测明写的 2 秒（`research/scripts/detached-run.sh:861`「start --label "${prefix}t" --stop-timeout 2」） | 默认起的单元真实时限是用户级默认 1min 30s，登记与日志写 180；broad-mutation / multi-host-run 的收尾按 90 秒被 KILL |
| c stop 不等停机任务 | `research/scripts/detached-run.sh:547`「if ! systemctl --user stop "$unit"; then」加 `--no-block`，之后至多等 5 秒「exit 在且单元停了」，等不到就写 stop-killed | Ⓖ 的 sleep 收 TERM 立刻退；Ⓣ 的时限 2 秒短于 5 秒 | 默认 180 秒、收尾要 8 秒以上的命令：stop 5 秒就回来说「被 KILL」，单元还在 deactivating，后来登记里出现真退出码而 stop-killed 也在 |
| f 查不到的判挪到挪走旧登记之后 | 把 `research/scripts/detached-run.sh:343`「detached_run_query_failed "$unit" "判不出同名的还在不在跑：这一次什么都没起，旧登记没动"」那三行挪到建新登记之前 | Ⓠ 的 start 用的是新名 q2（`research/scripts/detached-run.sh:809`），没有旧登记可挪 | 用户级 systemd 连不上时对一个还在跑的名 start：旧登记被挪成 `<名>@<秒数>`、退 1；之后 list 对带 @ 的不问状态，把在跑的列成「没留退出码」，`stop <名>` 退 3 停不到 |
| g watch 不记查不到的 | `research/scripts/detached-run.sh:510`「${DETACHED_RUN_UNKNOWN_RECORDS[@]+"${DETACHED_RUN_UNKNOWN_RECORDS[@]}"}」从观察里拿掉 | Ⓠ 只核 watch 不退、没报「都结束了」 | watch 开跑时连不上、期间那条跑完：对的实现报「opusr2g 退出码 0」，g 报「观察到 0 条」（`model-v4g-watch-unknown-then-ended.sh`，两份都跑过，原样在 `model-outputs.txt`） |

四句（五种同）：① 不分辨臂；② 判别子看得到（各格要的观测都在单元属性、登记与输出里）；③ 中 V4「写出一种错实现、自证与钩子自检全绿」——前提是主 agent 跑出五行「21 格都对」；④ 去向「补格」。
补格（推的）：Ⓛ 加「替身 loginctl 退 1」那一支；Ⓑ 或 Ⓒ 起的默认单元读 TimeoutStopUSec 要是 `3min`；Ⓣ 之外加一条「命令收 TERM 后 3 秒才退、`--stop-timeout 10`」核 stop 至少等 3 秒、报真退出码、没有 stop-killed；
Ⓠ 对一个已有登记的名 start、核旧登记原地不动；Ⓠ 的 watch 开跑后恢复替身、让那条写 exit，核收尾报它的退出码。另把 W1 做成一格（Ⓧ 里加 watch）。

### 钩子自证与样本放得过的三种（样本量过，自证按代码推）

| 名 | 错实现 | 样本 `cases-f1.json` 20 行 | 自证里有没有那一格 | 探针（应拒而放） |
|---|---|---|---|---|
| h4 | `.claude/hooks/bash-command-detector.sh:349`「DETACHED_RUN_COPYING_COMMANDS = {"cp", "install", "ln", "mv"}」改成只有 cp、ln | 20 行照旧 | 六格里只有「先 cp 改名再起」「先 ln -s 再起」（`.claude/hooks/bash-command-detector.sh:2644`「("同一条命令里先 ln -s 再起"」） | `install -m 755 … /tmp/opusr2-i.sh && bash /tmp/opusr2-i.sh start …` 与 `cp … && mv … && bash … start …` 由 2 变 0 |
| h5 | `written_detached_run` 只认 cp 拷来的，不认 heredoc 写出的（`.claude/hooks/heavy-test-guard.sh:485`「        head = (entry.text or "")[:shell_words.DETACHED_RUN_HEAD_BYTES].encode("utf-8")」那一支不走） | 20 行照旧 | 重型闸那五格里只有 cp（`.claude/hooks/heavy-test-guard.sh:858`「("主 agent 不带前缀同一条命令里先 cp 改名再起全量照拒", None,」） | 主 agent 不带前缀 heredoc 写出带签名行的脚本再 `start -- cargo test --all`，heavy 由 2 变 0 |
| h7 | 认法解相对路径用命令开头的目录、不跟 cd（`.claude/hooks/bash-command-detector.sh:559`「recognized = shell_words.detached_run_script(word, command.directory)」改成用 directory） | 20 行照旧 | 没有一格在 cd 之后用相对路径起改名拷贝 | `cd <草稿>/probe && bash ./rn.sh start …` 由 2 变 0 |

原样在 `hook-probes-v2.txt` 的 V4 段（三份错实现各自与 new 的差只在那条探针上）。三处都是实现员文件头写明「认」的行为：`install` / `mv`、heredoc、cd 跟着算。
四句：① 不分辨臂；② 看得到；③ 中 V4——样本那一半量过，自证那一半要主 agent 跑 `repro-v4-hook-selftests.sh` 看到四行 ✓ 才算；④ 补格：三条探针原样进自证与 `cases-f1.json`。

## 没打中的形状

- V2① 回归：148 条语料（第一轮攻方 44、实现员 20、常见写法 28 × 三种身份 84），old / new 两版判定只差在有意改的行。一次抽样。
- V2① 签名行误拒：全仓找带签名行的文件只有本尊；主 agent 合法写法 5 种（主仓绝对路径、`cd … &&`、`nice`、带日期的名、`--stop-timeout`）放。
- V2① 子 agent 闸照拒的：`exec -a x bash …`、`bash … --force stop m`、nice / env / capped 前缀、`bash -c`、source / .、改名拷贝、符号链接（含旧样本）。
- V2② 查不到 / 同名重起的拒与报：读 Ⓠ、Ⓡ 与对应代码，没找到错；只有一个推的轻窗口（先读 exit 后问状态）。
- V2③ 自证里的子调用开关照生效：读代码，每个临时登记目录都放了令牌。
- V2⑤ Linger：读不到照起、警告；没登录又没 linger 的「读不到」措辞到不了（preflight 先拒）。
- V3 两个 watch、stdout 关掉：读代码，watch / list 不写文件。
- 取样范围：钩子探针 29 条（`cases-v2a.json`）+ 148 条回归 + 24 条 V4（`cases-v4hooks.json`）；模型 7 份；真单元的复现 6 份都没跑。

## 这条腿自己的限度

- 「没打中」都只抽了一次样，不支撑否定结论（`.claude/rules/three-way-inference.md:159`「| **没打中**（「构造不出反例」「没发现问题」） |」那一行）。
- 没起过一个单元、没跑过任何 `--selftest`（钩子拒子 agent 的 start / stop / `--selftest`，没绕）：W1、W2 的真单元版本、`--log` 那条的整条 start、`--force` 那条的单元里实跑、V4 自证那一半，全靠主 agent 跑模型目录里的 `repro-*.sh`。期望写在 `repro-expected.txt`，是按代码推的。
- 模型用替身 systemctl：它只答 ActiveState / MainPID / list-units 三样，真 systemd 在登记被删、单元 failed 时的别的表现没进模型。
- 改法表里每一格都是推的、一格都没实现，被攻过零轮（`.claude/rules/three-way-inference.md:117` 那一条）。
- 钩子探针的树是「冻结副本 7 份 + 主仓此刻其余文件」，副本上的退出码不算入库装置上的数。
- 禁读清单：我用 `grep -rlE '^# detached-run-script-signature:' /tmp/claude-1000` 找带签名行的文件时，输出里列出了 `/tmp/claude-1000/tooling-detached-run-r2-sonnet/` 下 4 个文件名（只见到文件名，没打开、没读内容）；之后没再碰那个目录。主 agent 判这一点时知道就行。

## 没做什么

- 不判 V1（辩方那几格），F4「合法同名变量被误拒」也归 V1、没攻；不替主 agent 采纳。
- 没编译、没跑门禁、lint、重型测试与任何 `--selftest`；开工时 `ps` 看到别的会话的 3 条 cargo（mutants-wt 里的 nextest、两条 `cargo test -p singlefs-harness`），本腿不编译，没等锁。
- 没停、没碰 `singlefs-detached-broad-mutation-*` 两个单元与 `~/.local/state/singlefs/detached-runs/`：只 `ls` 过那两个登记目录的文件名、只读 `systemctl --user show -p TimeoutStopUSec` 两个单元。
- 本腿自己起的进程：watch 被 SIGKILL 那次留下的 sleep（pid 2481274），已 `proc.py stop`、核过不在。
- 交回前删掉的仓副本（都在 `/tmp/claude-1000/tooling-detached-run-r2-opus/`）：`new/`、`old/`、`mut-h4/`、`mut-h5/`、`mut-h7/` 各 34M（`.claude` 与 `research/scripts` 的拷贝），`lay/` 224K、`v4/` 412K（错实现脚本），`probe/` 72K（一份改名拷贝）；草稿目录剩 320K（喂钩子的输出与模型输出），没有编译目录。
