# 工具轮第三轮 攻方腿（Opus）报告：U2 改法本身 / U3 第二轮还开着的格（2026-10-01）

立场：攻方。被判文件读冻结副本 `/tmp/claude-1000/tooling-detached-run-r3/tree/`（`sha256sum -c files-sha256.txt` 7 份全 OK；主仓这 5 份脚本与钩子此刻与副本逐字节相同，下文行号两处一致）。
钩子只喂 JSON 输入判、不执行（`feed.py`，与第二轮同一写法）；要真起单元的写成复现脚本交主 agent 带前缀跑（单元名前缀 `singlefs-detached-opusr3-`、登记目录临时、收尾核零残留），本腿一份都没跑。
不起单元、只在 `/tmp` 的仓副本上直接对内存包装发信号量机制的，本腿跑过（U3 scope 活下来那一条）。
不碰正在跑的 `singlefs-detached-broad-mutation-*` 两个单元与 `~/.local/state/singlefs/detached-runs/`。

## 复跑命令与文件

模型目录 `research/prompts/tooling-detached-run-r3-opus-model/`。钩子探针的树是草稿目录里 `tree/`（冻结副本 7 份 + 主仓其余 `.claude` 与 `research/scripts`，逐字节同主仓）。

```
# 喂 JSON 判钩子（不执行命令）：参数是副本树根
OPUSR3_SCRATCH=<草稿> python3 feed.py <树> cases-g3.json           # U2③ G3 摘 --force 的变体
OPUSR3_SCRATCH=<草稿> python3 feed.py <树> cases-g3-generic.json   # 同一盲区在不经 detached-run 的直接命令上
# 开关对照（证明新开关真的卡东西，不是摆设）：
OPUSR3_SCRATCH=<草稿> python3 feed.py <树> cases-g3.json LIB_SHELL_WORDS_BREAK=detached-run-force-kept
# 单元里 exec env -- "$@" 的语义（把带 = 的词当赋值、照跑后面那条）：
bash model-g3-env-assignment.sh <树>/research/scripts/detached-run.sh
# U3 list 规模与自证噪声（替身 systemctl，不起单元）：
bash model-u3-list-scale-and-noise.sh <树>/research/scripts/detached-run.sh 10 100 1000
# U3 scope 活下来：真起单元交主 agent 跑
bash repro-u3-scope-survives-unit-kill.sh <仓根>
```

本腿在 `/tmp` 仓副本上直接量 scope 活下来（不经 detached-run 单元，等价地 kill 内存包装自己）：
`run-with-memory-cap.sh 64M bash -c 'exec sleep 90'` 起在后台，`kill`（TERM）与 `kill -KILL` 包装进程，看 scope 存活——原样在下文 U3。

sha256（`sha256sum` 原样）：

```
6d984e87182266c1deeac9c3b340c4193bbe3186024d3f41e5d7c517706d17e3  feed.py
8a9e46f6181901332c4646acb0eb97626e6071eea11886d6b8547f1620cd6a1f  cases-g3.json
a0559fbb9c1786512650adb8791200235ae104767b7e3d7398aa687e1e7e4d61  cases-g3-generic.json
d62574da95cb97ca28db2843d4b583cfb847ab69f2da860b7a65763c7f41ca88  model-g3-env-assignment.sh
e622d7564ffa9712e87efe79a939c471705812f26245812d78234b852081fa0b  model-u3-list-scale-and-noise.sh
32fae71f3781c292c402149aa840dbc66d24273ced64f4446270eebaec5f04aa  repro-u3-scope-survives-unit-kill.sh
4dea808633420d3d447bf4726907fe8238d957903cc728eb7926b0afd0e23241  model-outputs.txt
039762ed35e30efa863c649a572ac320f09b391ff5334c5c9cb5a5319f5c5ec6  repro-expected.txt
```

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| U2③ G3 摘 `--force` 与预检逐字一致 | **打中** | `--force=1` / `1=1` / `a-b=1` / `a.b=1` 当 `-- ` 之后第一个词：重型闸 heavy=0（看不到后面的 cargo），而单元里 `exec env -- "$@"` 把它当赋值、照跑 `cargo test --all`。G3 只摘整词 `--force`，这几种预检与 G3 都不摘，闸的切词把它当命令词停在那里 |
| U2③ 引号里的 `--force`：`$'--force'` / `{--force,}` / `$F` | **打中**（讨论见正文，`$F` 归旧盲区） | `$'--force'` heavy=0：bash 把它展成 `--force`、预检摘掉、单元跑 cargo，而闸不认 ANSI-C 引号，当它是命令词；`--for''ce`、`"--force"`、`\--force` 闸认得、heavy=2，不中 |
| U2② G1 把旧残留单元认成这一次起的 | 没打中 | start 在 systemd-run 之前持锁、已拒同名 active、reset-failed 同名 failed；systemd-run 失败后回读到的 active 单元只能是本次 `--unit=$unit` 起的 |
| U2① G2 回滚丢旧登记退出码 | 没打中（一处弱化，推的） | 挂掉在挪走之后、回滚之前：旧登记的退出码仍在 `<名>@<秒数>` 里，list 按 rc 先判 ended、照报；旁路文件在 `$XDG_RUNTIME_DIR`（tmpfs），重启后「登记写不进」降级成「没留退出码」、报因写错 |
| U2④ G5 报警计数 | 没打中 | 查得到那一轮把该键的连续数清 0（`detached_run_watch_alarm` 的 else 分支），交替不累积，只有连续 3 次才报 |
| U2⑤ 新开关在实际调用里生效 | 没打中（四个都真卡东西） | force-kept 把 heavy 2→0、COPY_BY_CP_LN 把 bash 2→0、START_DIRECTORY 把 bash 2→0、HEREDOC 把 heavy 2→0，各自对照见 `model-outputs.txt` 与正文 |
| U3 内存包装的 scope 单元被停时活下来 | **造出**（记欠账，这一轮不修） | detached-run 单元里跑内存包装时，scope 在 `singlefs-heavy.slice`、不在 service 的 cgroup；停 service 只停它自己 cgroup 里的进程，scope 连同命令活成孤儿。本腿直接 kill 包装量到：scope ActiveState=active、命令还在、账上留一笔 |
| U3 别的会话自证单元在 list 里是噪声 | 复现（旧欠账，这一轮不修） | 替身 25 个 `singlefs-detached-selftest-*` 单元、本会话登记 0 条：list 打 25 行「登记里没有」 |
| U3 1 000 条登记 list 耗时 | 量了，不判打中 | 本机 10 条 0.13 秒、100 条 0.91 秒、1 000 条 15.4 秒（第二轮报 11 秒，机器与负载不同；崩溃点测试不衡量时间成本，这里也不判） |

跑前判据对照（正文第五节）：U2「G1–G5 引进的新错」中 U2③（闸看的与单元里真跑的不一致：`--force=1` 等赋值形与 `$'--force'`）。U2①②④⑤没中。
U3「造得出 scope 活下来」中 ⇒ 记欠账。

## U2　G1–G5 改法本身哪一格会错

不重复第一轮 T2–T4、第二轮 V2–V4：第二轮打中的是**整词** `--force`（`-- --force cargo test --all`），G3 已修——本腿先核：C0 `-- --force cargo test --all` 现在 heavy=2（闸切出的词里有 cargo，被拒），G3 对整词有效（`model-outputs.txt` 的 cases-g3 C0 行）。这一轮攻 G3 没覆盖的**带 `=` 的赋值形**与**ANSI-C 引号形**。

### 打中：`--force=1` 与任何「不合法变量名的赋值」写在 `-- ` 之后第一个词，重型闸看不到后面的重型命令，而单元照跑它（U2③）

闸这一侧（`feed.py tree cases-g3.json`，主 agent 不带前缀，原样在 `model-outputs.txt`）：

| 写法（`start --label p -- <这里> cargo test --all`） | heavy | 对照 |
|---|---|---|
| `--force`（C0，G3 修的那格） | **2** | 被拒 |
| 合法名 `FOO=1`（C1；env 认它） | **2** | 被拒 |
| `--force=1`（E1） | **0** | 放行 |
| `1=1`（E2） | **0** | 放行 |
| `a-b=1`（E3） | **0** | 放行 |
| `a.b=1`（E5，里面还藏 `bash -c 'sleep 100 & disown'`） | **0**（bash 闸也 0） | 放行 |

单元那一侧（`model-g3-env-assignment.sh`，把单元里 `DETACHED_RUN_RUNNER_*` 三段原样抠出来跑）：命令开头是 `--force=1` / `1=1` / `a-b=1` / `a.b=1` 时，单元那个 bash 退 0、`exit=0`，**后面那条 `touch marker` 跑了**。根由是 `research/scripts/detached-run.sh:161`「"$BASH" -c 'printf "%s\n" "$$" > "$0/.pid.tmp" && mv -f -- "$0/.pid.tmp" "$0/pid" && exec env -- "$@"' "$record" "$@" >> "$log" 2>&1 < /dev/null」——`env -- a-b=1 cargo …` 里 env(1) 接受带 `-` 带数字开头的赋值名，设掉它再跑 `cargo test --all`。

为什么 G3 不摘它：预检 `.claude/singlefs-ai-sop/scripts/preflight.sh:31`「if [[ "$preflight_argument" == --force ]]; then PREFLIGHT_FORCE_GIVEN=1; else PREFLIGHT_ARGUMENTS+=("$preflight_argument"); fi」只摘**整词** `--force`，`--force=1` 不等于 `--force`；G3 的 `.claude/hooks/lib_shell_words.py:523`「return [argument for argument in arguments if argument != "--force"]」同样只去整词。所以闸切出的参数与预检摘出的逐字相同（G3 做到了它写的目标），可重型闸仍漏——漏在再下一层：闸的切词只把**合法变量名**的 `名=值` 当赋值跳过（`.claude/hooks/lib_shell_words.py:53`「ASSIGNMENT = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)=(.*)$", re.S)」），`a-b=1`、`--force=1`、`1=1` 不匹配 ⇒ 当成命令词、切词停在那里、看不到后面的 cargo；而 env(1) 比这个正则宽，照样把它当赋值、跑后面那条。于是**闸判的命令（停在 `a-b=1`）与单元里真跑的命令（`cargo test --all`）不一致**——正是 G3 要堵的那一类，只是换了个赋值名的壳。

这不是 detached-run 独有：`feed.py tree cases-g3-generic.json` 里 `env -- a-b=1 cargo test --all`（不经 detached-run，G1 行）heavy=0、`env -- FOO=1 cargo test --all`（G7 对照）heavy=2——是共用切词/重型判定对「不合法变量名赋值」的盲区。但 detached-run 的 `exec env -- "$@"` 把这个盲区变成**真会跑起来**的重型命令，所以在这条入口上它是功能性绕过，不只是语法噪声。

四句：① 不分辨臂，是闸的口子；② 判别子看得到：整条命令文本在闸手里，env(1) 对赋值名的宽松是可知的，闸能照 env 的口径判；③ 中 U2③「`--force=…`」那一分句，归「闸看的命令与单元里真跑的命令不一致」（与第二轮第二节第 5 条同一类）；④ 正文去向只有「改」，没有互斥改法要逐格核。

改法（只在我的模型上量过、**被攻过零轮**，下面都是推的、一格没实现）：

| 改法 | 修哪几格 | 量过 / 推的 |
|---|---|---|
| 闸的 leading-assignment 识别放宽到 env(1) 的口径：`-- ` 之后（或 env 参数里）凡是 `<非空且不以 - 开头之外的 bareword>=…`、直到第一个不带 `=` 的词，都当赋值跳过，接着往后找真命令词 | `--force=1`、`1=1`、`a-b=1`、`a.b=1`，连同 generic 的 `env -- a-b=1 cargo` | 推的（放宽只会多判重型、更严，方向安全；要核会不会误伤 `foo=bar` 这类单独赋值——那本就没有重型命令，判了也只是放行照旧） |
| 或：detached-run 的 runner 不经 `exec env -- "$@"` 吞赋值名，`-- ` 之后第一个词若含 `=` 而不是合法变量名就按用法错拒，逼用户写 `-- env 名=值 <命令>` | 同上，且与闸口径天然一致 | 推的 |

### 讨论：引号里的 `--force`（U2③ 第三分句）

`feed.py tree cases-g3.json` 的 Q 组：`$'--force'`（Q1）heavy=0、`{--force,}`（Q2）heavy=0、`$F`（Q6，`F=--force`）heavy=0；而 `--for''ce`（Q3）、`\--force`（Q4）、`"--force"`（Q5）heavy=2。
- `$'--force'`：bash 展成 `--force`，预检摘掉、单元跑 cargo；闸不做 ANSI-C 展开，当它是命令词 ⇒ 闸看不到 cargo，与真跑不一致。**算打中**，与上面赋值形同一个病：闸的词法模型比真 shell/env 窄。改法同上（给闸补 ANSI-C 展开，或 runner 侧收严）。
- `{--force,}`：brace 展开成 `--force` 与空串，真跑时空串首参可能让 `env --` 报错，不稳，不单独计。
- `$F`：变量值闸看不到，是 `lib_shell_words` 一直就有、第二轮已列的「变量里拼出来」盲区，不算这一轮新引进的，归旧欠账。
- `--for''ce`、`\--force`、`"--force"`：闸认得、heavy=2，**没打中**——G3/切词对这几种简单引号是对的。

### 没打中：G1 把旧残留单元认成这一次起的（U2②）

要 start 在 systemd-run 失败后回读到的 active 单元**不是**本次起的，才成立。读代码（冻结副本）：start 先拿登记目录的锁（`research/scripts/detached-run.sh:404`「exec 9>>"$detached_run_state/.lock"」、`:405`「if ! flock -w 60 9」），锁一直持到 systemd-run 之后才关（`:501`「exec 9>&-」）；拿锁后、起单元前已判同名 active 就退 3（`:415`）、同名 failed 就 reset-failed（`:420`）。所以到 `systemd-run`（`:480`）那一刻，不存在同名 active 的残留；systemd-run 退非 0 后回读（`:488`「state="$(detached_run_unit_state "$unit")"」）到的 active 只能是本次 `--unit=$unit` 起的。锁又挡住并发同名 start。构造不出「旧残留被认成这一次」——没打中（一次抽样，按规则不支撑「一定没有」，只报没造出来）。

### 没打中：G2 回滚丢旧登记的退出码（U2①），一处弱化

挂在「挪走旧登记（`research/scripts/detached-run.sh:425`「if ! mv -- "$record" "$archive"」）之后、回滚（`:467`「mv -- "$archive" "$record"」）之前」：旧登记此刻在 `<名>@<起跑秒数>`，它的 `exit` 文件还在，list 先按 rc 判 ended（`:284`「if [[ -n "$rc" ]]; then status=ended」），照报退出码，只是挂在带 `@` 的名下。退出码没丢——没打中。
一处弱化（推的，没真造）：旁路文件默认在 `$XDG_RUNTIME_DIR/singlefs-detached-runs`（`:142`），tmpfs、重启清空；而登记（`~/.local/state`）持久。重启后一条「登记写不进」的记录，它的 `write-failure-file` 指向已没的旁路文件，list 的 `:303`「if [[ "$status" == vanished && -n "$side" && -f "$side" ]]」因 `-f side` 不成立而降级成 vanished，报成「没留退出码：主进程被 KILL、用户级 systemd 退出或机器重启」（`:333`），把真因（登记写不进）说错。重启后多半已无所谓，低危，列这里备查。

### 没打中：G5 报警「连续 3 次」的计数（U2④）

`detached_run_watch_alarm`（`research/scripts/detached-run.sh:560`）对每个观察到的键：这一轮在 unknown 集里就加一（`:566`），不在就清 0（`:568`「DETACHED_RUN_UNKNOWN_ROUNDS["$key"]=0」）。查不到与查得到交替时清零、不累积，只有连续到 `DETACHED_RUN_UNKNOWN_ALARM_ROUNDS`（3）才报（`:574`）。计数对——没打中。

### 没打中：新开关都真卡东西（U2⑤，F4 那一类）

四个新开关各跑「设 / 不设」一对（`model-outputs.txt` 与下表；判定翻面即证明它不是摆设）：

| 开关 | 探针 | 不设 | 设上 |
|---|---|---|---|
| `LIB_SHELL_WORDS_BREAK=detached-run-force-kept` | 主 agent `-- --force cargo test --all` | heavy=2 | heavy=0 |
| `BASH_COMMAND_DETECTOR_DETACHED_RUN_COPY_BY_CP_LN=1` | clerk `install … dr.sh && bash dr.sh start` | bash=2 | bash=0 |
| `BASH_COMMAND_DETECTOR_DETACHED_RUN_START_DIRECTORY=1` | clerk `cp … ; cd /tmp && bash ./dr.sh start` | bash=2 | bash=0 |
| `HEAVY_TEST_GUARD_IGNORE_HEREDOC_DETACHED_RUN=1` | 主 agent heredoc 写带签名行拷贝再起全量 | heavy=2 | heavy=0 |

四个都翻面——没有一个是 F4 式的摆设。附带坐实 G4 的正向：不设开关时 clerk 的 `install` 改名拷贝（bash=2）、`mv` 改名拷贝、`cd` 后相对路径拷贝都被拒，主 agent heredoc 写出的拷贝起全量被重型闸拒（heavy=2）——G4 补的这几格确实在认。

## U3　第二轮还开着的格

第二轮判决第二节第 13、14 条（`research/prompts/tooling-detached-run-r2-main-verification.md`：「单元被 KILL 时内存包装的 scope 可能活下来」「别的会话的自证单元在 list 里是噪声」「1 000 条登记 list 要 11 秒」都记成推的/未判）。

### 造出：内存包装的 scope 在单元被停之后活下来（记欠账，这一轮不修）

机制：detached-run 的单元里若跑 `run-with-memory-cap.sh`，它经 `research/scripts/run-with-memory-cap.sh:811`「systemd-run --user --scope --unit="$unit" …」把命令放进**另一个** scope，挂在 `singlefs-heavy.slice`（`:132`「SLICE="${RUN_WITH_MEMORY_CAP_SLICE:-singlefs-heavy.slice}"」），不在 detached-run 单元（`singlefs-detached.slice` 下的 service）的 cgroup 里。停 service 单元时 systemd（KillMode 默认 control-group）只停 service 自己 cgroup 里的进程（runner bash 与包装 bash）；内存包装收到 TERM 走 `research/scripts/run-with-memory-cap.sh:800`「trap 'rm -f -- "$started_marker" "$STATE_DIRECTORY/ledger/$unit.json" "$peak_file" "$peak_file.partial"; exit 143' TERM」——只删账与标记、**不停 scope**，exit 走人。scope 在别的 slice 里没人动，命令成孤儿继续占内存预算，detached-run 看不到它（不是它登记的单元）。

本腿在 `/tmp` 仓副本上**直接量**（不碰正在跑的 broad-mutation 单元，等价地对内存包装发信号）：
`run-with-memory-cap.sh 64M bash -c '… exec sleep 90'` 起在后台，sleep 的 cgroup 落在 `singlefs-memory-cap-<包装pid>-*.scope`；`kill`（TERM）包装 2 秒后包装还在、sleep 还在；`kill -KILL` 包装后 `wait` 回来，**包装不在、sleep（命令）还在、父进程被 reparent、scope ActiveState=active、账上这一笔还留着 1 个**。原样（这一次真跑的输出）：

```
包装 … cgroup：session-4.scope；命令 …（sleep）cgroup：singlefs-memory-cap-…-.scope
TERM 包装 2 秒后：包装 bash；命令 sleep
KILL 包装之后（wait 回来了）：包装 不在；命令 sleep，父进程 …；scope singlefs-memory-cap-…-.scope ActiveState=active
账上这一笔：1 个
```

量完本腿按 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <sleep pid>` 停掉自己起的 sleep，收尾核残留为 0（scope 随命令退出转 inactive、账清 0、peak 文件留一个空壳）。真起 detached-run 单元、经 `detached-run.sh stop` 停 service 看 scope 存活的复现写成 `repro-u3-scope-survives-unit-kill.sh`（期望在 `repro-expected.txt`），交主 agent 带前缀跑坐实——子 agent 钩子拒 start/stop，本腿没跑。

四句：① 不分辨臂，是内存包装/脱离入口的交互；② 判别子看得到（scope 的 cgroup 路径、所属 slice 都读得到）；③ 合第二轮第 13 条、U3 题面；④ 跑前判据给「造得出 ⇒ 记欠账（第四十三节），这一轮不修」。未实现改法，只记一个可能方向：内存包装收到 TERM/撤销时同时 `systemctl --user stop <自己的 scope>`，或 detached-run 的停机把内层 scope 一并停——两者都没量，被攻过零轮。

### 复现：别的会话的自证单元是 list 的噪声（旧欠账，这一轮不修）

`model-u3-list-scale-and-noise.sh` 乙段：替身 systemctl 列 25 个 `singlefs-detached-selftest-99999-c*.service`、本会话登记 0 条，list 打 **25 行**「登记里没有  单元 …」。根由 `research/scripts/detached-run.sh:354`「[[ "$name" == "${DETACHED_RUN_UNIT_PREFIX}"*.service && -z "${registered_units[$name]:-}" ]] || continue」——凡前缀 `singlefs-detached-` 又不在本会话登记里的，一律列成「登记里没有」，别人的自证单元照进。第二轮已记欠账，钉住现状、这一轮不修。

### 量了不判：1 000 条登记 list 的耗时

`model-u3-list-scale-and-noise.sh` 甲段（本机，替身 systemctl、登记都已结束）：10 条 0.13 秒、100 条 0.91 秒、1 000 条 15.44 秒、输出 3 001 行 270 839 字节。约每条 15 ms，近线性。第二轮报 11 秒，机器/负载不同。按共用约束「崩溃点测试不衡量时间成本」与第二轮判决第 14 条，耗时不判打中，列此备查。

## 没打中的形状

- U2② G1 旧残留被认成这一次：锁与前置判挡住，构造不出（一次抽样）。
- U2① G2 回滚丢退出码：挪走的旧登记仍带 exit、list 照报；只量到重启后旁路文件没了导致「登记写不进」降级成「没留退出码」的报因错误（推的，低危）。
- U2④ G5 报警计数：交替清零、只连续计数，对。
- U2⑤ 四个新开关：都真翻面，无摆设。
- 引号 `--for''ce` / `"--force"` / `\--force`：闸认得、heavy=2。

## 这条腿自己的限度

- 钩子只喂 JSON 判、没执行；U2 的「单元里真跑什么」用抠出 RUNNER 段在本进程跑的模型量（`model-g3-env-assignment.sh`），不是真单元——真单元的 `env`、PATH 行为由 `repro-*` 交主 agent 坐实。
- U3 scope 活下来在 `/tmp` 仓副本上对内存包装直接发信号量到，等价于「调用方（单元里的 runner）被停」；真经 detached-run 单元 + `detached-run.sh stop` 的链路写成复现脚本、未亲跑。
- 所有改法只在我的模型/读码上推过，被攻过零轮，一格没实现没跑。
- 否定结论（G1/G5/开关没打中）只抽了一次样，按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」不支撑「一定没问题」，只报「这一轮没造出来」。

## 没做什么

- 不判正推（U1）、辩方那几格；不替主 agent 采纳；副本/模型上量出的数不算入库装置上的数。
- 没真起 detached-run 单元（钩子拒子 agent start/stop），真单元复现交主 agent。
- 没跑任何重型测试、没跑门禁阶段与 `--selftest`。
- 没碰正在跑的 `singlefs-detached-broad-mutation-*` 两个单元与 `~/.local/state/singlefs/detached-runs/`。
