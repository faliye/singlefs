# tooling-detached-run-r2 辩方腿（Sonnet）：V1 复核第一轮判决

日期 2026-10-01。被复核：`research/prompts/tooling-detached-run-r1-main-verification.md` 第二节 12 行。立场：替被 F1–F7 收严的一方（F 之前的写法、主 agent 的合法用法）辩护。
被判文件读冻结副本 `/tmp/claude-1000/tooling-detached-run-r2/tree/`：七份 `sha256sum -c` 全 OK；主仓里同名的五份（detached-run.sh、两道钩子、两份共用库）此刻与冻结副本 sha256 相同，所以下面引文行号两处一致，引文写主仓路径。
钩子判定全是把命令写成 JSON 喂副本里的钩子、只看退出码，没执行任何被判命令、没起单元、没跑 `--selftest`。喂法与样本在 `research/prompts/tooling-detached-run-r2-sonnet-model/`（`feed.py`、`cases-v1*.json`、原样输出 `v1*.out`，探针重建见 `make-probes.sh`）。

## 一、一览表：判决条目 / 辩方判定 / 证据

判定列三个问：打中够不够得着真实用法；改法拒没拒合法用法；同样的打法打不打得中替代方案。

| # | 第一轮的条目 | 辩方判定 | 证据 | 什么现象会推翻 |
|---|---|---|---|---|
| 1 | T3 子 agent 闸：source / .、改名拷贝放过 | 字面够得着；真实危害窄；**同样打中别的臂**；F1 没拒合法用法 | 二.1、三.1、三.4 | 找到一个没被拒过的子 agent 在没有规则被拒记录时自然写出 source / 改名的例子 |
| 2 | T3 重型闸：改名拷贝藏 cargo test --all | 字面够得着；**主 agent 实际用法里够不着**（真实命令不走改名）；同样打中别的臂 | 二.2、三.1、三.4 | 主 agent 实际起过一条经改名拷贝的长活 |
| 3 | T4 自证只验 TERM（HUP 错实现全绿） | 站得住（测试缺口）；代码今天对，危害为零，F2 没拒任何合法用法 | 二.3 | HUP 路径上发现 watch 对单元有任何写动作 |
| 4 | T4 Ⓓ 只核 3/5 个变量 | 站得住（小）；不拒合法用法 | 二.4（变量表在 `research/scripts/detached-run.sh:102`） | 变量表再变而 Ⓓ 不跟 |
| 5 | T1 systemctl 失败当「单元不在」 | 站得住（真危害，但只影响上报） | 二.5；F3 只在失败时拒 start / stop | 失败时 systemctl 退 0 |
| 6 | T1 弄坏开关在生产调用也生效 | 字面够得着（要环境变量）；F4 没拒合法同名变量；**同样打中兄弟脚本** | 二.6、三.2 | 找到一处设了 DETACHED_RUN_BREAK 的合法调用方 |
| 7 | T1 stop 超 90 秒被 KILL | 够得着但窄：已知两条链收尾远小于 90 秒；F5 不拒合法用法 | 二.7、三.3 | 一条链的收尾在 90 秒到 180 秒之间 |
| 8 | T1 Linger=no 会话关闭 | 环境已解；F6 只警告、不拒 | 二.8 | Linger 被关而没有警告 |
| 9 | T1 规则两处字面 | 站得住；F7 两行没拒任何合法用法 | 二.9 | 新句子与同节别条矛盾 |
| 10 | T1 推的三种（登记被删、start 失败留登记、同名重起漏旧退出码） | 站得住；有一个副作用要主 agent 知道（见四.4） | 二.10 | — |
| 11 | T2 没打中 | 辩方无话可说：本腿不重打 T2；只抽一次样，不支撑「没有错」 | 第一轮判决自己写了 | — |
| 12 | T3 主 agent 夹带没打中 | 同上 | — | — |

## 二、逐条复核

### 二.1 条目 1：子 agent 闸

重放（旧认法用 `BASH_COMMAND_DETECTOR_DETACHED_RUN_BY_NAME=1` 与 `LIB_HEAVY_TESTS_BREAK=detached-run-by-name-only` 开着，等于 F1 之前；新认法不开）。子 agent 身份 clerk，探针与命令在 `cases-v1f.json`、`cases-v1c.json`，输出 `v1f-*.out`、`v1c-*.out`：

| 子 agent 的写法 | 旧认法（名字） | F1 之后 |
|---|---|---|
| `source research/scripts/detached-run.sh start …` | 放过 rc=0 | 拒 rc=2 |
| `. research/scripts/detached-run.sh start …` | 放过 rc=0 | 拒 rc=2 |
| source 带签名行的改名拷贝 | 放过 rc=0 | 拒 rc=2 |
| 同一条命令 cp 再 bash 起 | 放过 rc=0 | 拒 rc=2 |
| 改名拷贝 / 符号链接 / CRLF 拷贝 / 直接执行改名拷贝 | 放过 rc=0 | 拒 rc=2 |
| 删掉签名行的拷贝、签名改一个字符、签名挪出前 4 KiB | 放过 rc=0 | **仍放过 rc=0** |
| `cat 原件 > 拷贝 && bash 拷贝 start`、变量里的路径、eval、python subprocess、xargs、`/usr/bin/time`、flock 包装 | 放过 rc=0 | **仍放过 rc=0** |

「够得着」这一半成立：旧认法确实放过，F1 确实拦住前五行。「同样打中」成立：同一个打法（换一种写法）在 F1 之后对后两行照样得手，对「按内容摘要认」「按文件名认」「禁拷贝」三种替代同样得手（改一个字符就换了摘要；`cat >` 不是 cp）。F1 把门槛从「改个名」抬到「改文件内容或换拷贝方式」，不是把口子堵死。第一轮攻方自己也写了这一点：「子 agent 闸是劝阻性的护栏不是安全边界」（`research/prompts/tooling-detached-run-r1-opus-output.md:132`）。

危害真不真：
- 走到改名拷贝，子 agent 得先被拒一次再换写法。共用约束写的是「被拒了照出路改写法，不换写法绕过去」（`.claude/agent-common.md:66`）。所以这条口子要先违反一条规则才走得到。
- 起出来的单元不是没人看得见：走默认登记目录的，`list` 与 `watch` 都报它（`list` 遍历全部登记）；登记在别处的，F3 加的核对按 `singlefs-detached-*` 前缀另列一行（`research/scripts/detached-run.sh:292`：「登记里没有  单元 $name」），`watch` 不等这一类。
- 同类先例（G3 做法）：`.claude/hooks/lib_heavy_tests.py:92`：「文件开头 BROAD_MUTATION_HEAD_BYTES 里有驱动的签名行」，同文件把罩不住的写法逐条记成清单，写明「multi-host-run.sh --、xargs、find -exec、eval（对每一类重型都在的老口子）」（`.claude/hooks/lib_heavy_tests.py:96`）。F1 的 9 类认不出清单是同一个做法，不是新开的洞。

### 二.2 条目 2：重型闸（主 agent 不带前缀、改名拷贝起 cargo test --all）

主 agent 身份、不带 `SINGLEFS_HEAVY_TESTS`，重型闸退出码（`cases-v1g.json`，输出 `v1g-new.out`、`v1g-old.out`）：

| 命令 | 旧认法 | F1 之后 |
|---|---|---|
| 带签名行的改名拷贝 `start --label p -- cargo test --all` | 0 放过 | 2 拒 |
| 真名 `detached-run.sh start … -- cargo test --all`、直接 `cargo test --all`、`bash -c 'cargo test --all'` | 2 拒 | 2 拒 |
| `bash research/scripts/multi-host-run.sh --peer -- cargo test --all` | **0 放过** | **0 放过** |
| `printf 'cargo test --all\n' > w.sh && bash w.sh` | 0 放过 | 0 放过 |
| `echo \| xargs cargo test --all` | 0 放过 | 0 放过 |

主 agent 此刻在跑的两条长活，登记里记的命令（只读 `~/.local/state/singlefs/detached-runs/*/command`、`cwd`、`environment`，原样）：

```
== broad-mutation-local/
command: nice -n 19 python3 research/scripts/broad-mutation.py --threads 31 --memory-cap 24G --skip-stage-two --shard 0/2
cwd: /tmp/claude-1000/m3-r1/mutants-wt
environment: SINGLEFS_HEAVY_TESTS PATH HOME
== broad-mutation-peer/
command: bash research/scripts/multi-host-run.sh --peer --memory-cap 4G --label broad-mutation-1001-detached -- bash -c SINGLEFS_HEAVY_TESTS=user-request\ python3\ research/scripts/broad-mutation.py\ --sync-tree\ \&\&\ SINGLEFS_HEAVY_TESTS=user-request\ nice\ -n\ 19\ python3\ research/scripts/broad-mutation.py\ --threads\ 31\ --memory-cap\ 24G\ --skip-stage-two\ --shard\ 1/2
```

把这两条原样写成主 agent 命令喂 F1 之后的钩子（`cases-v1d.json`、`v1d.out`，cwd 取 `/tmp/claude-1000/m3-r1/mutants-wt`，脚本取主仓绝对路径）：local 带前缀 detector 0、heavy 0；local 不带前缀 heavy 2（闸在工作）；local 不带前缀经改名拷贝 heavy 2；peer 形态不带任何前缀 heavy **0**，同一条 peer 命令不经 detached-run.sh 直接跑也是 0。
判定：改名拷贝不是主 agent 实际用法里会走到的路（实际命令都是真名加主仓绝对路径，F 之前就认），也不比实际 peer 形态那个老口子更宽；它自己把 `multi-host-run.sh --` 记成老口子，见 `.claude/hooks/lib_heavy_tests.py:96`：「对每一类重型都在的老口子」。「够得着」字面成立（旧认法 rc=0 已重放），「真的危害」在主 agent 自己起长活这条用法里不成立：要有意拷贝并改名才走得到，而不带前缀本来就是忘记前缀这种无心之失要防的，无心之失不会改名。F1 重型这一半没拒任何合法用法（见三.2）。

### 二.3 条目 3：HUP 错实现

代码本身对：`research/scripts/detached-run.sh:478`：「if detached_run_break_is watch-stops-units; then」之外，watch 的收尾函数里没有停单元的路径（第 472 到 489 行读过）。所以第一轮这一条是测试缺口，不是行为缺陷，今天的危害为零。F2 加的三格（`research/scripts/detached-run.sh:786` 到 `788`）只是起一条 sleep 单元、发信号、核单元还在，不拒任何用法；站得住。同样的打法（给 watch 注入一种对某个信号才发作的错）对「只测 TERM」的任何自证都得手，对三信号参数化的新写法得手不了，所以这条改法是唯一堵得上的写法，不是「同样打中别的臂」。

### 二.4 条目 4：Ⓓ 的变量

`research/scripts/detached-run.sh:102` 的变量表五项，Ⓓ 现在五项都核（`:725` 到 `:736`）。小，便宜，站得住。

### 二.5 条目 5：查询失败当「单元不在」

此刻只读复核（systemd 255.4）：
```
$ systemctl --user show -p ActiveState --value singlefs-detached-nonexistent-xyz.service ; echo rc=$?
inactive
rc=0
$ env XDG_RUNTIME_DIR=/nonexistent DBUS_SESSION_BUS_ADDRESS=unix:path=/nonexistent/bus systemctl --user show -p ActiveState --value x.service ; echo rc=$?
Failed to connect to bus: No such file or directory
rc=1
```
单元不在是 `inactive`、退 0，查询失败是空加退 1，两者分得开，所以旧写法把空串当「不在」确实是错的，第一轮这一条站得住。F3 的改法只在失败时拒 start / stop、让 watch 接着看，合法路径（systemctl 正常）上行为不变；详见四.1 对 watch 无上限等待的一个线索。

### 二.6 条目 6：弄坏开关在生产调用里生效，F4

- 够得着的前提是环境里有人设了 `DETACHED_RUN_BREAK`。全仓（含 `.claude/`、`research/`，不含 `.git`、`target`、`prompts`、`records`）只有 `research/scripts/detached-run.sh` 自己引用它；runner 表项在 `.claude/gate.d/code-tooling.sh:245`：「bash research/scripts/detached-run.sh --selftest」只起 `--selftest`，F4 的判在 `research/scripts/detached-run.sh:910` 放行 `--selftest`。所以没有合法的同名设置被拒：门禁、别的脚本都不设它。
- 同类先例（F4 的形状）：`research/scripts/broad-mutation.py:297`：「只供自证的变量出现在自证之外、或指着自证工作目录之外的库：拒，一步都不做。」，它的 TEST_ONLY_VARIABLES 里就有 BROAD_MUTATION_BREAK（`:205` 到 `:206`）。
- 反面：兄弟脚本不判。`research/scripts/multi-host-run.sh:67`：「multi_host_break_is() { [[ ",${MULTI_HOST_RUN_BREAK:-}," == *",$1,"* ]]; }」直接读环境变量，不问是不是自证；`research/scripts/mutation-shard-run.sh:69` 同形。`research/scripts/` 下带 `_BREAK` 字样的 .sh / .py 共 47 份（`grep -l` 数的），我只核了这三份的判法，别的没读。所以「开关在生产里生效」这个打法对这些兄弟脚本同样得手，F4 只收了 detached-run.sh 一份，这是同样打中别的臂，不是 F4 错。
- 退 4 的代价：持久环境里留着开关时 `stop` 也被挡（出路写在 stderr：`env -u DETACHED_RUN_BREAK …`）。Bash 工具每次调用 shell 状态不持久，这只会在有人在自己的终端 export 过时遇到。判：站得住，窄。

### 二.7 条目 7：停机时限，F5

- `research/scripts/detached-run.sh:389`：「if ! detached_run_break_is stop-timeout-ignored; then properties+=("--property=TimeoutStopSec=$stop_timeout"); fi」把时限设进单元；默认见 `:105`。systemd 手册（本机 `man systemd.service`，原样）：「If it doesn't terminate in the specified time, it will be forcibly terminated by SIGKILL」；单元没有 ExecStop 时「the service gets the SIGTERM immediately」。
- 本机用户级默认：两个在跑的旧单元 `systemctl --user show -p TimeoutStopUSec --value` 打 `1min 30s`，登记里没有 stop-timeout。
- 收尾要多久（逐段数，来源行）见三.3。结论：已知两条链都在 90 秒内，所以第一轮这一条的「真实危害」要等收尾超过 90 秒才碰上；F5 是便宜的保险，不拒任何合法用法。仓里同类先例只找到一处：`research/scripts/run-with-memory-cap.sh:805` 给内存包装的 scope 设 TimeoutStopSec（值取 `:125` 的 DEFAULT_KILL_GRACE_SECONDS=30）；detached-run.sh 的 180 秒没有先例可抄，文件头自己写了「这几段收尾要多久是推的，没量过」。

### 二.8 条目 8：Linger，F6

此刻只读：`loginctl show-user fy5090 -p Linger --value` 打 `yes`。环境已解。`research/scripts/detached-run.sh:176`：「elif [[ "$linger" == no ]]; then」只在 stderr 打一行警告、照起，不拒；站得住，便宜。

### 二.9 条目 9：规则字面，F7

现在写 `.claude/agent-common.md:66`：「没另写的主 agent 与子 agent 一样拒；只挂在一方才用的工具上的、按角色分开判的（同一种写法放行主 agent、拒子 agent），在条目里写明」，末句 `.claude/main-agent.md:40`：「脱离单元由 `detached-run.sh watch` 上报，`watch.sh --processes` 看不到它们。」。两处都是加限定语，没有拒任何用法，与同节 ③ 的并排不再冲突；站得住。

### 二.10 条目 10：三种推的情形

登记被删、start 失败留登记、同名重起，F3 各给了一格自证（`research/scripts/detached-run.sh:822` 起 Ⓢ、`:844` 起 Ⓡ、Ⓧ 在 `:855`）。它们在日常用法里都要先有一个异常（有人删登记、systemd-run 失败、同名重起）才触发；改法不拒合法用法。站得住。副作用见四.4。

## 三、正文第三节 V1 的逐问

### 三.1 F1–F7 有没有把合法用法也拒掉

喂 F 之后的两道钩子，退出码写「detector / heavy」（0 放行、2 拒；`cases-v1a/b/e/h.json`，输出 `v1a/b/e/h.out`）。主 agent：

| 用法 | detector / heavy |
|---|---|
| worktree 里、主仓绝对路径、`nice -n 19`、`SINGLEFS_HEAVY_TESTS=user-request`、label `bm-local-2026-10-01`、`--log`，起 broad-mutation.py | 0 / 0 |
| 同一形状把内层换成 `cargo test --all`：不带前缀 / `=commit` | 0 / 2（该拒） / 0 / 0 |
| `cd /tmp && bash <绝对路径> start …`、`capped.sh 8` 前缀、子 shell 里 cd 加前缀、`export SINGLEFS_HEAVY_TESTS=commit; …`、`env SINGLEFS_HEAVY_TESTS=commit …` | 全部 0 / 0 |
| `--stop-timeout 600`、`list`、`watch --every 600`、`stop` | 全部 0 / 0 |
| 在 worktree 里用相对路径 `research/scripts/detached-run.sh start`（worktree 里没有该文件） | 0 / 0 |
| 内层非重型 `cargo test -p singlefs-harness --lib`；内层 `bash -c 'a & b & wait'` | 0 / 0 |
| 内层 `& disown`、`setsid -f`、`pkill -f`、`systemd-run`（`--stop-timeout`、`--log` 在前、`--label=` 写法都试了） | 2 / —（③ 对 start 之后那条命令的剥开没变） |
| 起 `cargo test -p singlefs-checker-tier`，不带前缀 | 0 / 0，与不经 detached-run.sh 直接跑一致（经 start 与直接跑判得一样） |

label 的写法（`research/scripts/detached-run.sh:103` 的 `^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$`，这一行不在 F1–F7 的 diff 里）现跑 `[[ =~ ]]`：`bm-2026-10-01`、`2026-10-01-bm`、`bm_2026.10.01`、64 字的名 ok；含空格、含斜杠、点起头、减号起头、65 字的拒。带日期的 label 不被拒。
子 agent 侧（detector 只管 start / stop）：`cp` 备份 detached-run.sh、`cp` 之后 `bash -n`、`cp` 之后 `list`、`grep` / `diff` / `bash -n` 原件、别的脚本的 `start` 子命令、`systemctl --user start foo.service`、heredoc / echo / `git commit -m` / `python3 -c` 里只是写着 `detached-run.sh start` 的文字，全部 0。
查到的唯一一处误拒：同一条命令里先把 detached-run.sh 拷到 `/tmp/t.sh`、再把 capped.sh 拷到同一个目标、最后 `bash /tmp/t.sh start …`，detector 拒（2）。copy 集合记的是目标名、不是最后写入者。这条是人造的，被拒的又恰是子 agent 的 `start`，无害。
没有找到被 F1–F7 拒掉的主 agent 合法用法。

钩子耗时（5 次取最小）：detector 0.07 到 0.09 秒，heavy 0.06 秒；`detached_run_script` 读盘的写法是 `os.path.isfile` 之后读 4 KiB（`.claude/hooks/lib_shell_words.py:500`），FIFO、目录不会卡住。名字判在读盘之前（`.claude/hooks/lib_shell_words.py:495`），所以真名脚本不多一次读。

### 三.2 F4「设了开关就拒」

见二.6：全仓没有别的调用方设 `DETACHED_RUN_BREAK`；门禁只起 `--selftest`；先例是 `research/scripts/broad-mutation.py:297`；兄弟脚本 `research/scripts/multi-host-run.sh:67` 不判。

### 三.3 F5 的 180 秒与两条收尾链

systemd 的做法：停单元时 KillMode 默认 control-group（本机 `man systemd.kill`：「Defaults to control-group.」），单元里每个进程同时收 TERM，到 TimeoutStopSec 还在的收 KILL。runner 的 bash 把 TERM 记成标志、等命令退出再写退出码（`research/scripts/detached-run.sh:118`）。

| 链 | 收尾上界（已读的代码） | 出处 |
|---|---|---|
| 内存包装 scope | TimeoutStopSec 默认 30 秒 | `research/scripts/run-with-memory-cap.sh:125`、`:805` |
| broad-mutation.py 被 TERM | 先 `systemctl --user stop` 自己的 scope（超时取 stop-grace，默认 10），再 `child.wait(10)`，到点 KILL；约 20 秒，再拷回原件 | `research/scripts/broad-mutation.py:146`、`:1273` 到 `:1291` |
| multi-host-run.sh 本机模式 | 停命令的整棵树至多 30 秒（加 0.5 秒回读） | `research/scripts/multi-host-run.sh:31`、`research/scripts/peer-host-lib.sh:155` |
| multi-host-run.sh --peer 模式 | 等进程号文件至多 10 秒，加第二台那棵树至多 30 秒，加 `--fetch` 取回（有的话），加一次 ssh 删树 | `research/scripts/peer-host-lib.sh:213`、`research/scripts/multi-host-run.sh:31` |

主 agent 在跑的 peer 单元登记里没有 `--fetch`（二.2 的原样输出），所以它的收尾约 41 秒加一次 ssh 删树，估不出的只有 ssh 往返。结论：
- 已知两条链都在旧默认 90 秒内；180 秒对它们有 4 倍以上余量，对 peer 链在有 `--fetch` 时给取回留约 139 秒（取回体量没量过）。
- 合得上；「合得上」是纸面算术，不是量出来的：180 本来就是推的，文件头自己写了没量过。
- 反方向的打法：任何常数都会被「某条链收尾更长」打中，`infinity` 会被「stop 永不返回、关机拖死」打中；参数化加登记（`--stop-timeout`、登记里的 stop-timeout）是不被这两类打法打中的写法，F5 取的就是它。

### 三.4 同样的打法打不打得中所有替代方案

| 被收严的条目 | 第一轮的打法 | 对 F 之前 | 对 F | 对替代 |
|---|---|---|---|---|
| T3 子 agent 闸 / 重型闸 | 换一种写法 | 得手 | 对「删签名、改一字、cat 拷、变量路径、eval」仍得手 | 按摘要认：改一字即脱；禁 cp：`cat >` 绕过；只按名字：改名即脱。能分辨调用者身份的只有钩子（`.claude/hooks/bash-command-detector.sh:2866` 把 hook_input 的 agent_type 交给 detached_run_refusal），脚本自己拿不到，所以「放进脚本里判」不是替代 |
| 弄坏开关在生产里生效 | 设环境变量 | 得手 | 得手不了（退 4） | 对 multi-host-run.sh 等兄弟脚本得手，是同类问题，不是 F4 错 |
| stop 超时被 KILL | 收尾更长的命令 | 90 秒下得手 | 180 秒下得手要更长的收尾 | 任何常数同样被打中；参数化的写法最好 |
| 查询失败当不在 | 总线断 | 得手 | 得手不了；但换了方向（见四.1） | 「N 次连续失败后放弃」会被「恰好 N 次抖动」打中，要选 N |

## 四、顺手读到的线索（不是 V1 的判定，交攻方腿或主 agent 现查；都没起单元重放）

1. watch 对「查不到」没有上限。`research/scripts/detached-run.sh:511`：「if ((DETACHED_RUN_UNKNOWN_COUNT > 0)); then」只打一行、接着睡下一轮，总线一直不通就一直等，失败方向从「误报全结束」翻成「静默等到会话的后台时限把 watch 停掉」（停掉时 watch 会打最后一次清单）。兄弟脚本的写法是有上限的确认，写在 `research/scripts/peer-host-lib.sh:189`：「vanished（进程没了、没留退出码，连续 3 次）；unreachable（连续 60 次 ssh 不通）」。F3 方向对，上限没定，推的。
2. F5 默认 180 秒比 Bash 工具默认前台超时（工具说明给的默认是 120000 毫秒）长。文件头只拿它与 240 秒比（`research/scripts/detached-run.sh:28`：「又要比会话里前台命令的 240 秒时限短」），共用约束写的是上限：`.claude/agent-common.md:63`：「前台命令的 timeout 不超过 240000 毫秒」。收尾真超过 120 秒、而主 agent 没显式给 timeout 时，`stop` 会被工具截断，登记里的 stop-killed 标记写不上。已知链都远小于 120 秒，所以只有未知的长收尾会碰上，推的。
3. 单元被 KILL 时内存包装的 scope 不一定跟着没：`research/scripts/run-with-memory-cap.sh:811` 用 `systemd-run --user --scope` 起 scope，scope 是单元 cgroup 之外的另一个 cgroup，注释在 `research/scripts/broad-mutation.py:1271`：「另停它开的那一个 scope」。所以到点 KILL 只杀单元 cgroup，scope 里的 cargo 理论上还活着。180 秒降低碰上的概率，没有消除。推的，没量。
4. 自证期间主 agent 的 `list` 会多出噪声行：自证用临时登记目录（`research/scripts/detached-run.sh:674`），单元前缀仍是 `singlefs-detached-`（`:100`），`list` 的核对只按前缀（`:288`），所以别的会话在跑自证（例如门禁 `.claude/gate.d/code-tooling.sh:245` 那一格）时，这边 `list` 把自证的单元列成「登记里没有」。`watch` 不等这一类（只有登记里还在跑的与查不到的进观察表，`research/scripts/detached-run.sh:509`），所以只是噪声，不影响何时退出。推的，没重放。
5. 重型闸看不穿 `multi-host-run.sh --`：二.2 的表，主 agent 实际的 peer 单元形状不带任何前缀也放行，F1 前后一样，是 `.claude/hooks/lib_heavy_tests.py:96` 记着的老口子，不在 F1 的射程里。

## 五、没做什么

- 没替主 agent 撤销或维持判决，没判哪条该留哪条该撤：第一轮判决里没有一条我判成「够不着，所以那条改法该交用户定留不留」的完整够不着；条目 1、2 是字面够得着、真实危害窄，留不留是个成本判断，交主 agent 与用户。
- 没起任何 systemd 单元，没跑任何 `--selftest`，没执行任何被判命令。钩子判定全是 JSON 喂副本；`systemctl --user list-units / show`、`loginctl show-user`、`man systemd.service`、`man systemd.kill` 是只读。
- 没读禁读清单里的任何文件。没碰 T2（第一轮没打中、只抽一次样的格），没重做 V2、V3、V4。
- 没量过：180 秒收尾预算（peer 链有 `--fetch` 时的取回体量）、自证 21 格的耗时、KILL 路径上 scope 是不是真的活下来（四.3）、watch 在总线长时间不通下的实际行为（四.1）。
- 钩子行为是副本里的结果（五份文件与主仓 sha256 相同）；判决要引的数按三方规则应由主 agent 在入库装置上重做，副本里的数不进 kb。
- 临时目录：`/tmp/claude-1000/tooling-detached-run-r2-sonnet/`（`repo/` 是冻结副本七份文件加 `.claude/singlefs-ai-sop` 的拷贝，`probe/` 是探针、`out/` 是原样输出）。没有编译目录、没有 git 工作树；`repo/` 与 `probe/` 是仓副本一类的东西，保留的理由：主 agent 复核时要在同一棵树上重放 `feed.py`，复核完可整个删。
