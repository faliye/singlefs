# 工具轮第三轮 U1 正推腿报告（tooling-detached-run-r3，Sonnet，2026-10-01）

立场：正推。三件事：G1–G5 照第二轮判决做到了没有；改法与规则对不对得上；前两轮判决引的数在各自的日志里找不找得到。
被判文件读冻结副本 `/tmp/claude-1000/tooling-detached-run-r3/tree/`。下文 `detached-run.sh`、`lib_shell_words.py` 的行号都是那份副本里的行号。
模型、脚本与原样输出在 `research/prompts/tooling-detached-run-r3-sonnet-model/`（`README.md` 写了怎么搭副本）。

## 0 先打开的几样核（命令与原样输出）

```
$ sha256sum -c research/prompts/tooling-detached-run-r3-snapshot/files-sha256.txt   （在主仓根）
research/scripts/detached-run.sh: OK
.claude/hooks/bash-command-detector.sh: OK
.claude/hooks/lib_heavy_tests.py: OK
.claude/hooks/heavy-test-guard.sh: OK
.claude/hooks/lib_shell_words.py: OK
.claude/agent-common.md: OK
.claude/main-agent.md: OK
```

- 补丁与副本：把 `/tmp/claude-1000/m3-r1/detached-run-3/orig/` 的 5 份备份拷进草稿目录，`patch -p1 < research/prompts/tooling-detached-run-r3-diff.patch`，打上后 5 份与冻结副本逐字节相同（`cmp` 无输出）。
- 备份与第二轮冻结副本：5 份 `cmp` 逐字节相同，与正文「5 份备份与第二轮冻结副本逐字节相同」一致。
- `crates/` 里没有相关实现：`grep -rln 'systemd-run\|detached' crates/` 零命中（退出码 1），与正文一致。符合 `.claude/rules/implementation-first.md:13` 对「没有相关实现」的写法。
- 自证格数：副本里自证函数的 `detached_run_expect` 直接调用 30 处、`detached_run_selftest_watch_signal` 3 处，合计 33，与验收日志 `detached-accept3/self.log` 的「33 格都对」一致；第二轮副本同法数出 21，所以这一轮加 12 格，与实现员报告一致。
- 弄坏开关：验收 `summary.txt` 里 `break=` 行 27 条、27 个不同名字、`rc=1` 27 条；副本里 `detached_run_break_is <名>` 出现 25 个不同名字，加 `watch-stops-units-on-${1,,}` 模板展开的 hup、int 两个，共 27，对得上。
- 语法：`bash -n` 过三份 .sh；两份 .py 用 `ast.parse` 过（没写字节码）。

我没跑任何 `--selftest`、没起任何单元（按派发提示）。自证、27 个开关、`feed3.py` 的结果我只读主 agent 的验收日志（`scratchpad/detached-accept3/`），在判定里写「读日志」；能不起单元复核的我自己复核了（下文写明）。

## 1 G1：watch 按单元名核；systemd-run 报错而单元照起

判决原文（转述）：`research/prompts/tooling-detached-run-r2-main-verification.md:40-41`，两半：watch 按单元名核状态、登记被删而单元还 active 的报「还在跑（登记里没有）」不算结束；start 遇到 systemd-run 退非 0 回读单元状态，在跑就记成在跑并写明退出码。

做到了什么：
- watch 观察时连单元名一起记：`research/scripts/detached-run.sh:530`「DETACHED_RUN_OBSERVED_UNITS["$key"]="$(detached_run_field "$record" unit)"」。
- 登记不见的按单元名查：`research/scripts/detached-run.sh:534`「观察到的各条里登记不见了（被删、被挪走）而同名没有新登记的：按记下的单元名查状态。」，在跑的打一行并不算结束，`research/scripts/detached-run.sh:551`「watch 接着等它，退出码读不到」；查不到的另列「查不到（登记里没有）」（:552-554），单元不在了才进结局（:611-614）。
- start 回读：`research/scripts/detached-run.sh:488`「if ! detached_run_break_is start-failure-not-rechecked; then state="$(detached_run_unit_state "$unit")"; fi」，在跑就写 `research/scripts/detached-run.sh:490`「detached_run_write_field "$record" systemd-run-failed」（日期与 systemd-run 的退出码），stderr 打 `!` 警告，往下走、退 0；不在跑才写 start-failed、退 1（:493-498）。
- list：记了 start-failed 的先按单元名查（:290-300），在跑列成还在跑并注明 systemd-run 报过错（:311-313）。
- 读日志：验收的 w1、w2 在新版上 `watch 退 124；此刻单元 ActiveState=active`、`start 退 0`（`detached-accept3/repro-w1-…log`、`repro-w2-…log`）；自证 Ⓦ 三格与 `watch-trusts-registry`、`start-failure-not-rechecked`、`start-failed-trusted` 三个开关各红点名的格（`summary.txt`）。

没做到、多做了什么：
- 没做到（文字上更宽的半句）：「按单元名核状态，不只信登记」在实现里只对「watch 观察到过的」生效。开跑前登记就不在的单元，list 列「登记里没有」但 watch 不等它，`research/scripts/detached-run.sh:55`「登记里没有、也没观察到的单元只在清单里列，不等它们」。判决点名的触发场景（观察之后被删）修了；这半句是实现员自己划的界，文件头写明了，不是漏。后果：登记在 watch 开跑前就被删、单元还在跑时，watch 第一轮就退 0 报「观察到 0 条」，同一份输出里有 list 的「登记里没有」行。
- 多做：`list` 对 start-failed 回查（判决没点名，但不做的话 w2 的 list 半边还是错的）；「查不到（登记里没有）」一类；`ran-without-pid-trusted` 等开关属于 G2。
- 回读窗口：systemd-run 报错、回读那一刻单元还没起来（排队中）的，记 start-failed 退 1，之后 list 按单元名回查会改判成在跑（:296 只在不是 active、不是查不到时才落到 start-failed）。文件头写了这一点。没有丢单元。

判定：做到（判决点名的触发场景全修，自证与开关能红）；上面「没做到」一条记作残留，不是 G1 没做。
推翻它的现象：在副本里让 watch 先观察到单元，再 `mv` 登记目录走而单元还在跑，watch 退 0 或报「都结束了」；或 start 在 systemd-run 退非 0 且单元在跑时退 1 并且不写 systemd-run-failed。

## 2 G2：写不进就不起；单元里写登记失败单列

判决原文（转述）：`research/prompts/tooling-detached-run-r2-main-verification.md:42`，start 起单元之前先核 --log 与登记目录都写得进，写不进退 1、不起单元、绝不打「✓ 起了」；单元里写登记失败的结局单列一类，不报成被 KILL。

做到了什么：
- --log：`research/scripts/detached-run.sh:393`「if ! detached_run_break_is log-unchecked && { [[ ! -w "$(dirname -- "$log")" ]] || [[ -e "$log" && ! -w "$log" ]]; }; then」，写不进退 1、什么都没起（:394-397）。
- 登记目录：建不了目录或锁文件退 1（:399-403）；登记各项与日志开头逐项带 `||` 记「写不进」（:451-464），有一样不成就删这一次的登记并挪回旧登记：`research/scripts/detached-run.sh:466`「rm -rf -- "${record:?}"」，`research/scripts/detached-run.sh:467`「if [[ -n "$archive" ]] && mv -- "$archive" "$record"; then」，退 1（:470）。
- 起完回读：有退出码没有命令进程号判没起成：`research/scripts/detached-run.sh:514`「if [[ -z "$pid" ]] && ! detached_run_break_is ran-without-pid-trusted; then」，退 1。
- 单元里写登记失败：旁路文件，`research/scripts/detached-run.sh:172`「mkdir -p -- "${side%/*}" && printf '%s\n' "$note" > "$side"」，路径在 `research/scripts/detached-run.sh:142`「detached_run_side_directory="${DETACHED_RUN_SIDE_DIRECTORY:-${XDG_RUNTIME_DIR:-/tmp}/singlefs-detached-runs}"」，由 start 在调用方环境里算好、作为第 3 个参数交给单元里的 bash（:483），所以单元里不必有 `XDG_RUNTIME_DIR`。list 列「登记写不进」（:302-303、:327-330），watch 结局报「登记写不进」（:623-624）；旁路也写不进的照旧是「没留退出码」，文件头第 28 行写明。
- 读日志：v3 在新版上 `start 退 1`、`marker：不在`（`detached-accept3/repro-v3-log-unwritable.log`）；自证 Ⓥ 三格与三个开关红点名的格（`summary.txt`）。

没做到、多做了什么：
- 小缺口：`mv -- "$record" "$archive"` 把同名旧登记挪走之后（:425），`mkdir -- "$record"` 失败的分支（`research/scripts/detached-run.sh:435`「if ! mkdir -- "$record"; then」）直接 `exit 1`，没有把旧登记挪回。旧登记仍在 `<名>@<起跑秒数>` 里，list 读得到、不丢退出码，只是原名下没有登记。触发条件是 mv 刚成功而 mkdir 立刻失败（inode 或空间耗尽），窄。G2 判决的字面「写不进就退 1、不起单元」做到了；「旧登记挪回」是实现员在 :467 那一支自己加的，这一支没带上。
- 小缺口：挪回的 `mv` 自己失败时（:467）没有任何输出，用户不知道旧登记在哪。同样不丢数据（list 读 `<名>@<秒数>`）。
- 多做：旁路文件、登记新项 `write-failure-file`、起完回读的「命令没跑起来」判断；这几样判决没要，但都是为「单列、不报成被 KILL」所需。
- 旁路目录在 `$XDG_RUNTIME_DIR`（tmpfs）：重启后旁路文件没了，登记里 `write-failure-file` 指向的路径不存在，list 退回「没留退出码」那一支，文案里本来就写着「机器重启」（:333），不会误报。这一格 U2 在打，我只记事实。

判定：做到；两处小缺口记作残留。
推翻它的现象：`--log` 指到不可写目录时 start 退 0 或打「✓ 起了」；命令退出后登记目录不可写时 list 仍报「没留退出码」且旁路文件存在。

## 3 G3：--force 摘法与预检一致，闸看的与真跑的逐字相同

判决原文（转述）：`research/prompts/tooling-detached-run-r2-main-verification.md:43`，共用的 start 解析与预检对 --force 的处理一致、闸看到的命令要与单元里真跑的逐字相同，做不到就让脚本在 -- 之后完全不经预检摘参数，选哪种写理由；判红样本是主 agent 不带前缀的 `-- --force cargo test --all` 要被重型闸拒。

做到了什么：
- 选「与预检一致」。预检的摘法只有一条：`.claude/singlefs-ai-sop/scripts/preflight.sh:31`「if [[ "$preflight_argument" == --force ]]; then PREFLIGHT_FORCE_GIVEN=1; else PREFLIGHT_ARGUMENTS+=("$preflight_argument"); fi」，逐个整词比。
- 共用库照抄：`.claude/hooks/lib_shell_words.py:518`「def detached_run_arguments_after_preflight(arguments):」，`.claude/hooks/lib_shell_words.py:523`「return [argument for argument in arguments if argument != "--force"]」；`detached_run_start_command` 在 :530 先调它；子 agent 闸 `detached_run_subcommand` 与重型闸都经同一份（`bash-command-detector.sh:357`、`lib_heavy_tests.py:864`）。
- 理由（实现员报告第二段 G3）我回原文核：「`--` 之后不经预检摘参数」在脚本里做不到。判据在 `.claude/singlefs-ai-sop/scripts/preflight-lint.py:66-67` 的整行正则，预检行之前只许 `SHELL_PROLOGUE_RES` 那几类行，且 `POSITIONAL_PARAMETER_RE`（:81）禁读位置参数，所以不能在预检之前把原始参数存起来。理由站得住。
- 判红样本：重型闸自检里加了判决点名的那一格与 `--label --force p` 一格（`heavy-test-guard.sh` 自检），探测器自检加 `-- --force setsid -f` 一格，`lib_heavy_tests` 自检加 `-- --force cargo test --all` 一格；自证 Ⓟ 把闸切出的词与登记里真跑的命令逐字比（`research/scripts/detached-run.sh:1227`）。
- 我在草稿副本里独立喂钩子输入（不执行命令，`feed_u1.py`，原样输出 `feed_u1-plain.out`、`feed_u1-break.out`）：

```
无开关：
G3 判决点名样本 -- 之后 --force 全量	detector=0	heavy=2
G3 带前缀 user-request 放行对照	detector=0	heavy=0
G3 --label 值位置 --force	detector=0	heavy=2
G3 --force 在最前	detector=0	heavy=2
G3 引号里的 --force	detector=0	heavy=2
LIB_SHELL_WORDS_BREAK=detached-run-force-kept：
G3 判决点名样本 -- 之后 --force 全量	detector=0	heavy=0
G3 --label 值位置 --force	detector=0	heavy=0
G3 引号里的 --force	detector=0	heavy=0
```

没做到、多做了什么：
- 判决字面（「逐字相同」）做到：切出的词与真跑的词一样。多做：开关 `LIB_SHELL_WORDS_BREAK=detached-run-force-kept`、自证 Ⓟ。
- 线索（超出 G3 字面，旧口子，不是这一轮引进的）：「逐字相同」不等于「同样理解」。真跑是 `research/scripts/detached-run.sh:161`「"$BASH" -c 'printf "%s\n" "$$" > "$0/.pid.tmp" && mv -f -- "$0/.pid.tmp" "$0/pid" && exec env -- "$@"' "$record" "$@" >> "$log" 2>&1 < /dev/null」，里面是 `exec env -- "$@"`，`env` 把 `--` 之后含 `=` 的头一个词当赋值。我在草稿里喂 `bash research/scripts/detached-run.sh start --label p -- --force=1 cargo test --all`，重型闸退 0（放行）；`env -- --force=1 echo ran` 实跑输出 `ran`、退出码 0，说明真跑会执行后面的命令。同类的 `-- a-b=1 cargo test --all` 重型闸也退 0，不涉及 --force，所以是共用库对「名字不是标识符的赋值」的旧盲区。这正是派发提示 U2 ③ 点名的 `--force=…`，交给 U2 与主 agent 判，我不下定论。`-- -- --force cargo test --all` 摘掉后是 `-- -- cargo …`，真跑 `env -- -- cargo` 报 `No such file or directory`（退 127），命令不会执行，闸放行无害。

判定：做到（字面）；「同样理解」那一半有旧口子，线索交 U2。
推翻它的现象：`-- --force cargo test --all`（主 agent 不带前缀）重型闸退 0；或自证 Ⓟ 里闸切出的词与登记的 command 不一致却不红。

## 4 G4：自证补格；钩子补 h4、h7、h5

判决原文（转述）：`research/prompts/tooling-detached-run-r2-main-verification.md:44-46`，攻方 b、c、f、g 四种错实现各成开关、各补一格、各喂一次必须红；钩子自检补 h4（install / mv）、h7（cd 之后的相对路径），h5（heredoc 写出的拷贝）在重型闸自检里造一格、对照组先跑绿。

做到了什么：
- 四个开关与格：b `default-stop-timeout-unset`（`research/scripts/detached-run.sh:477`「if detached_run_break_is default-stop-timeout-unset && [[ -z "$stop_timeout_given" ]]; then properties=(); fi」，格 Ⓒ 没写 --stop-timeout）；c `stop-nonblocking`（:725，格 Ⓝ，命令收到 TERM 后 7 秒才退 9、比 stop 的 5 秒回读长）；f `query-check-after-archive`（:411、:432，格 Ⓠ 同名）；g `watch-skips-unknown`（:672，格 Ⓠ 连续 3 次与格 Ⓙ）。
- 读日志（主 agent 验收）：`summary.txt` 里四个开关各 `rc=1` 且红的格就是上面点名的；攻方 `v4_mutations.py` 的原文本改法在新版上 `repro-v4-selftest-wrong-impls.log`：control 33 格都对，a、b、c、f、g 各退 1。
- 钩子三格我独立复现（`feed_g4.py`，原样输出 `feed_g4.out`，不执行命令）：

```
无开关：    h4 install 改名再起 detector=2；h4 mv 改名再起 detector=2；h7 cd 之后相对路径 detector=2；h5 heredoc 写出拷贝再起全量(主agent) heavy=2
..._COPY_BY_CP_LN=1：     h4 install 改名再起 detector=0；h4 mv 改名再起 detector=0；h4 cp 改名再起（旧）detector=2
..._START_DIRECTORY=1：   h7 cd 之后相对路径 detector=0
HEAVY_TEST_GUARD_IGNORE_HEREDOC_DETACHED_RUN=1：  h5 heredoc 写出拷贝再起全量(主agent) heavy=0
```

- h5 的对照组：验收 `guard.log` 里 `✗` 0 行、末行「自检通过」，开关下红两行，都属同一格（那一格有「照拒」与「读进去几份脚本」两句断言）。

没做到、多做了什么：
- 没有没做到的。判决说「对照组先跑绿」：第二轮攻方副本里对照组退 1，实现员没查原因（报告第七段）；这一轮验收在主仓里对照组退 0（`summary.txt` 的 `3 guard rc=0`），所以那次退 1 是攻方副本的环境问题，不是 h5 格的问题。我没法在副本里重跑 `--selftest` 来排，写「复核不了」。
- 实现员报告的预期与验收日志有一处不符：报告写 b 退 1「1 格不对」，验收 `repro-v4-selftest-wrong-impls.log` 是「b-default-stop-timeout-unset：--selftest 退 1；2 格不对」，多红一格；方向没错（更红），只是预期没对上，写出来备查。
- 多做：三个钩子开关各自成对（`BASH_COMMAND_DETECTOR_DETACHED_RUN_COPY_BY_CP_LN`、`…_START_DIRECTORY`、`HEAVY_TEST_GUARD_IGNORE_HEREDOC_DETACHED_RUN`），G3 的两格重型闸自检。

判定：做到。
推翻它的现象：把 b、c、f、g 之一的错实现喂给新自证仍退 0；或 h4、h5、h7 三类命令在对应开关关闭时探测器（重型闸）退 0。

## 5 G5：连续查不到报警；stop 怎么等

判决原文（转述）：`research/prompts/tooling-detached-run-r2-main-verification.md:47`，watch 对「查不到」连续若干次（照 `research/scripts/peer-host-lib.sh:189` 的连续次数上限写法，次数实现员定）后在清单里单列报警，仍不退出、不停单元；`--stop-timeout` 默认值与 stop 的等待写进文件头，并写明前台调 stop 时 Bash 工具的超时要给够，或改成 stop 只发停机；二选一、写理由。

做到了什么：
- 报警：每轮 `detached_run_watch_alarm`（:560）对观察到的每一条记连续次数：`research/scripts/detached-run.sh:568`「DETACHED_RUN_UNKNOWN_ROUNDS["$key"]=0」是这一轮查得到时清零，查不到时加一（:566），到 `DETACHED_RUN_UNKNOWN_ALARM_ROUNDS`（3，:140）单列「报警」并带出路（:577-582），不退出、不停单元。
- 我用替身 systemctl 在草稿里跑 watch（`alarm_model.sh`，只读，不起单元，原样输出 `alarm_model.out`），ActiveState 查询按计划逐次给 F（失败）或 A（active），计划 `F F A F F A F F F F A A`：

```
第 1-2 次 F、第 3 次 A、第 4-5 次 F、第 6 次 A：没有「报警」行
第 7-9 次 F：第 9 次出现「! 报警  u1 …连续 3 次查不到单元状态」
第 10 次 F：「! 报警  u1 …连续 4 次查不到单元状态」
第 11 次 A 起：回到「还在跑」，不再报警；watch 一直没自己退（退 124 是我设的 timeout 15）
```

  交替 F、A 时每次清零，不累计；连续 3 次才报。这与实现员报告「查得到一次就清零」一致；与 `research/scripts/peer-host-lib.sh:189` 同形（那边 `unreachable_rounds` 在查得到时清零）。
- stop：选「写明前台 timeout」。`research/scripts/detached-run.sh:60`「前台调用 stop 时，Bash 工具的 timeout 至少给到（停机时限 + 30）× 1000 毫秒」，默认 210000；start 成功时按这一次的时限算好毫秒数再打一行（:522）；文件头 :63 写了不改成只发停机的理由（同名重起要旧单元已停下），回原文核：start 判同名在跑就拒（:415-419），属实。
- 默认值的口径：`research/scripts/detached-run.sh:35`「这几段收尾要多久是推的，没量过」，对应 `.claude/singlefs-ai-sop/rules/evidence-discipline.md@进结论的每个数，先问它是量的还是拍的` 要求的「拍的要标出来」，标了。次数 3 与「总线一两秒的抖动」那句（:53-54）没标推的，是选取值，不是量出的数；判决说「次数你定」，不算违规，记一笔。

没做到、多做了什么：
- 一处算术不一致：文件头 `research/scripts/detached-run.sh:62`「停机时限超过 205 秒的（加 30 秒超过前台命令 240000 毫秒的上限）」。（时限 + 30）× 1000 超过 240000 要时限大于 210，不是 205：205 加 30 是 235 秒、不超；206 到 210 秒这一档，头注要求用 run_in_background，而 start 打的那句 `research/scripts/detached-run.sh:522`「超过 240000 的用 run_in_background 起 stop）」按算出的毫秒数判，不会这么说。方向保守（多要求后台），不会误伤，但两处写法不同，应统一成 210 或把公式改成（时限 + 35）。
- 240000 这个数在文件头被说成「前台命令的上限」，它是规则的数（`.claude/agent-common.md:63`「前台命令的 `timeout` 不超过 240000 毫秒」），不是 Bash 工具自己的上限；头注没指明出处。
- 「工具不写 timeout 时默认 120 秒」（:61）是环境事实，规则文件里没有这个数，我只能对着本会话 Bash 工具说明里的默认 120000 毫秒认，不是在仓里能查的出处。
- 被截断后 `list` 能看到什么：头注 :61-62 说「之后用 list 看」。若 stop 被截断而命令到点被 KILL，`stop-killed` 没来得及写，list 走的是 `research/scripts/detached-run.sh:333` 那句通用说明（「主进程被 KILL、用户级 systemd 退出…或机器重启」）而不是「stop 停的」；若命令在时限里退出了，退出码在登记里、list 报「已结束 退出码 N（detached-run.sh stop 停的）」。所以「退出码报不出来，之后用 list 看」对后一种成立、对前一种只看得到通用说明。
- 多做：stop 开头打一行时限与要等多久（:722）。

判定：做到（报警我复现；stop 选「写明」与规则对得上，见下一节）；算术不一致与出处缺失记作小问题。
推翻它的现象：替身 systemctl 下，交替 F、A 的计划里出现「报警」行，或连续 3 次 F 之后没有「报警」行，或 watch 在报警后自己退出。

## 6 G5「stop 的超时只写在脚本文件头」与规则对不对得上，要不要写进规则

我读的条款：
- `.claude/main-agent.md:40`「要停只用 `detached-run.sh stop <名>`」，同一行：「脱离单元由 `detached-run.sh watch` 上报，`watch.sh --processes` 看不到它们。」
- `.claude/agent-common.md:63`「前台命令的 `timeout` 不超过 240000 毫秒」，同一行：「子 agent 不用这个入口。」
- 上一轮判决 F7 要的两处字面都在：`.claude/agent-common.md:66` 有「只挂在一方才用的工具上的、按角色分开判的（同一种写法放行主 agent、拒子 agent），在条目里写明」，`.claude/main-agent.md:40` 有 watch.sh --processes 那一句。

对得上吗：
1. 不冲突。默认 stop 要 185 秒，前台 timeout 210000 毫秒，落在 `.claude/agent-common.md:63` 的 240000 毫秒之内；时限更长的 stop 用 run_in_background 起、结束本轮等通知，正是同一行写的做法。两条规则都没写 stop 怎么调，文件头补的是规则之外的调用细节，没有改规则任何一句的意思。
2. 「长活可以等，不给它设超时」管的是长活：stop 至多一百多秒，不是长活；给 stop 一个够长的 Bash timeout，与「不给长活设超时」方向一致，没有冲突。
3. 受众：start 与 stop 只给主 agent（探测器拒子 agent，`.claude/agent-common.md:63` 同一行「子 agent 不用这个入口」），所以这条调用细节只涉及 `.claude/main-agent.md`，不涉及子 agent 共用约束。
4. 缺口（不是矛盾）：`.claude/main-agent.md` 里「一轮怎么开、怎么收」第 11 条规定会话过 30 万上下文就换新会话。换了会话之后的主 agent 读 `.claude/main-agent.md:40` 只知道「要停只用 stop」，没见过 start 当时打的那一行；它会先读 stop 的文件头吗，规则没要求。后果有限：stop 被 Bash 的 120 秒默认 timeout 截断时，systemd 里的停机任务不受影响（头注 :61 的说法，我没起单元验）；丢的是 stop 这一次的结局输出，`list` 仍读得到「已结束 退出码 N（detached-run.sh stop 停的）」，只有命令到点被 KILL 的那一种丢成通用说明（第 5 节末）。

要不要写进规则：
- 不要，G5 本身不需要。第二轮判决原文要的就是「写进文件头」（`research/prompts/tooling-detached-run-r2-main-verification.md:47`），而且 `.claude/singlefs-ai-sop/rules/rules-discipline.md` 的表里「怎么做：步骤、次序、判据、阈值」才是规则的内容，调用某个脚本时的超时数属于脚本自己的用法，放脚本头合乎「什么规则在哪里：指到别的规则、脚本或 kb」的分工。
- 想补缺口的话，最小的写法是在 `.claude/main-agent.md:40` 那句「要停只用 `detached-run.sh stop <名>`」后面加一个指向：前台调 stop 的 Bash timeout 见 `research/scripts/detached-run.sh` 文件头 stop 一段。只指路，不写数，数变了规则不用跟着改。这仍是改 `.claude/main-agent.md`，要照 `.claude/rules/implementation-workflow.md:20`「**改它们与改 `crates/` 同规矩**」走三方或由用户逐份豁免；我不替主 agent 定，列进交用户表由主 agent 带上。
- 推翻「不要」的现象：本机上真有主 agent 前台调 stop 被 120 秒 timeout 截断而丢了 stop-killed 的记录（我没见到这样的事，也没法复核，要主 agent 实测）。

## 7 前两轮判决引的数，在各自的日志里逐字找不找得到

核法：命令在上面各段与模型目录里；「找到」指在被引文件里逐字（或逐行号）命中。判决文件：`research/prompts/tooling-detached-run-r1-main-verification.md`（下称 R1）、`research/prompts/tooling-detached-run-r2-main-verification.md`（下称 R2）。

### 7.1 第一轮（R1）

| R1 里的数或引文（行） | 怎么核 | 结果 |
|---|---|---|
| 第 10、11 行：正推腿 210 行、sha256 前 8 位 dd4b4481；攻方腿 138 行、d594df29 | `sha256sum`、`wc -l` | 对上：dd4b4481 / 210，d594df29 / 138 |
| 第 14 行：被判 7 份文件 `sha256sum -c` 全 OK | 在 `/tmp/claude-1000/tooling-detached-run-r1/tree/` 里对 `tooling-detached-run-r1-snapshot/files-sha256.txt` | 7 行 OK |
| 第 15 行：cite-check「核了 23 处引文，对上 21 处，没判 2 处」；没判的是 `rules-discipline.md:13`、`watch.sh:11` | 用 R1 副本做根、R1 背景材料重跑（`cite-r1.out`） | 末行 `✓ 核了 23 处引文，对上 21 处，没判 2 处（逐处列在上面）`，没判的就是这两条；两条在主仓现查命中 |
| 第 21、22 行：clerk 真名 start 退 2；source、`.`、cp 改名三种退 0；主 agent 不带前缀经改名拷贝起全量重型闸退 0，clerk 同样退 0 | 把 R1 冻结的三份钩子放进草稿副本，喂钩子输入（`feed_r1.py`、`feed_r1.out`） | detector 2 / 0 / 0 / 0，heavy 0 / 0，全部对上 |
| 第 23 行：`detached-run.sh` 第 352–354 行 TERM / INT / HUP 同一个收尾函数；坏脚本自证「11 格都对」、退 0；「残留单元 0」 | 读 R1 副本第 352–354 行；读 `scratchpad/t4repro.log` | 352–354 行对上；「✓ detached-run.sh 自证 11 格都对」与「--selftest 退出码 0」在日志里；「残留单元 0」那一句在这份日志里逐字找不到（日志不含此句，同义的是自证末格「收尾之后 … 一条不剩」，在倒数第三行） |
| 第 25 行：第 109 行吞掉 stderr，第 168–169 行空串判 vanished | 读 R1 副本 | 109 行是 `systemctl --user show -p ActiveState --value "$1" 2>/dev/null`；168 行 `status=vanished`，对上 |
| 第 26 行：第 332 行判开关不看是不是在自证里；第 548 行把开关带进单元 | 读 R1 副本 | 332 行 `if detached_run_break_is watch-stops-units; then`；548 行 `${DETACHED_RUN_BREAK:+"--setenv=DETACHED_RUN_BREAK=$DETACHED_RUN_BREAK"} \`，对上 |
| 第 27 行：`grep -c TimeoutStopSec` 0 次 | 在 R1 副本数 | 0 |
| 第 28 行：用户 linger 定案在 `records/2026-09-30-里程碑三开工第一轮.md` 第五节 | `grep -n` | 第 167 行是第五节标题，第 194 行有「理论上不会，你去核实一下。如果需要就开启 两台本地主机都开启」 |
| 第 29 行：`agent-common.md` 第 66 行「主 agent 与子 agent 一样拒」 | 读 R1 副本第 66 行 | 对上 |

结论：R1 的数全部找得到；只有「残留单元 0」一句在引用的那份日志里没有逐字（结论「打中」不受影响，那一格的证据是自证的 11 格与退 0，都在）。按跑前写死的判据，「找不到 ⇒ 那一条判决改写」：只需把那半句改成「自证末格收尾后单元一条不剩」或删掉。

### 7.2 第二轮（R2）

| R2 里的数或引文（行） | 怎么核 | 结果 |
|---|---|---|
| 第 10、11 行：辩方腿 189 行 6e8335d9；攻方腿 298 行 e439761d | `sha256sum`、`wc -l` | 对上：6e8335d9 / 189，e439761d / 298 |
| 第 14 行：7 份 `sha256sum -c` 全 OK | 在 `/tmp/claude-1000/tooling-detached-run-r2/tree/` 里对 R2 的 `files-sha256.txt` | 7 行 OK |
| 第 15 行：cite-check「核了 52 处引文，对上 44 处，没判 8 处」及那 8 处 | 重跑（`cite-r2.out`） | 末行 `✓ 核了 52 处引文，对上 44 处，没判 8 处（逐处列在上面）`，8 处与 R2 列的逐条相同 |
| 同行：8 处没判「内容与报告所述对得上」 | 今天在主仓逐条 `awk` 取行、`grep -F` 原文 | `code-tooling.sh:245`、`broad-mutation.py:297`、`multi-host-run.sh:67`、`peer-host-lib.sh:189`、`broad-mutation.py:1271`、`preflight.sh:31`（在 `.claude/singlefs-ai-sop/scripts/`）、`evidence-discipline.md:122`、`tooling-detached-run-r1-opus-output.md:132` 全部命中 |
| 第 22–24 行：w1、w2、v3 的输出行 | 读 `scratchpad/r2repro/*.log` | w1：`start 退 0`、`watch 退 0；此刻单元 ActiveState=active`、`✓ detached-run.sh watch：观察到的 1 条都结束了`；w2：`start 退 1`、`单元 ActiveState=active`、`没起成 … 命令一行都没跑`、`开跑时就没有还在跑的，观察到 0 条`；v3：`start 退 0`、`Permission denied`、`marker：不在`、`已结束 … 退出码 1`，都在 |
| 第 26 行：b1 `marker：ran`、登记里的命令不带 --force；`-- --force cargo test --all` 主 agent 不带前缀重型闸放行 | 读 `repro-b1-force-inner.log`；在 R2 冻结钩子上喂输入（`feed_r2.py`、`feed_r2.out`） | 日志有 `marker：ran；登记里的命令：bash -c echo\ ran…` 不带 --force；钩子 detector=0、heavy=0（放行），对上；同一输入在这一轮钩子上 heavy=2 |
| 第 27 行：v4 的 a 红 1 格、b / c / f / g 退 0 且「0 格不对」；b 单元 `TimeoutStopUSec=1min 30s` 而登记 180；c 在 5 秒回来、10 秒后 exit=9 与 stop-killed 并存 | 读 `repro-v4-selftest-wrong-impls.log` | 逐行都在；c 那行 stop 输出被脚本的 `cut -c` 截在「停机�」，「被 KILL」字样在紧跟的 `stop-killed=… 被 KILL，没留退出码` 一行里 |
| 第 28 行：h4、h7 的 `bash-command-detector --selftest` 退 0；`heavy-test-guard` 对照组在副本里退 1 | 读 `repro-v4-hook-selftests.log` | 都在 |
| 第 32 行：`multi-host-run.sh:67` 属实，`mutation-shard-run.sh:69` 同形 | 今天现查 | 67 行 `multi_host_break_is() { [[ ",${MULTI_HOST_RUN_BREAK:-}," == *",$1,"* ]]; }`；69 行 `mutation_shard_break_is() { [[ ",${MUTATION_SHARD_RUN_BREAK:-}," == *",$1,"* ]]; }` |
| 第 35 行：1 000 条登记 list 要 11 秒 | 读攻方 `model-outputs.txt` | `N=1000 类=ended list 退 0 耗时 11415 ms`、`类=vanished … 10264 ms`；我没重跑（U3 的格） |
| 第 32、34 行：记欠账 | 读 `records/2026-09-16-subagent拆分提案.md` | 第四十三节第 3、4 行就是这几条（弄坏开关两份脚本；scope、自证单元噪声） |

结论：R2 的数全部找得到、cite-check 与钩子退出码重跑一致。没有一条需要改写。

### 7.3 正文里的数（顺带）

- 正文说 `main-agent.md` 第 52、61、63 行在第二轮之后被别的会话改过。`diff` 第二轮副本与这一轮副本：改的是 52、61、63、66 四行（66 行是「这四道」改「这五道」）。第 66 行与这件事无关，但「三行」的数对不上。
- 正文说验收末行「自证 33 格都对」、27 个开关各退 1、`feed3.py` 9 行全对、残留单元 0：读 `detached-accept3/summary.txt` 与 `self.log`，都在。

## 8 留给主 agent 与 U2 的线索（我坐实了事实，不下定论）

| # | 线索 | 事实依据 | 与哪一格有关 |
|---|---|---|---|
| 1 | `start --label p -- --force=1 cargo test --all`：重型闸看成命令词 `--force=1`，放行；真跑是 `exec env -- "$@"`，`env` 把它当赋值、执行后面的 cargo | `feed_u1-plain.out` 里该行 `heavy=0`；`env -- --force=1 echo ran` 输出 `ran`、退出码 0；`-- a-b=1 cargo test --all` 同样 `heavy=0`，说明是共用库对非标识符赋值的旧盲区，不是 G3 引进的 | U2 ③、第四十三节第 1 行那一批 |
| 2 | `mkdir -- "$record"` 失败的分支不把挪走的旧登记挪回 | `research/scripts/detached-run.sh:425`、`:435`、`:467` | G2 回滚（U2 ①） |
| 3 | 头注 205 秒与公式（时限 + 30）对不上：应是 210，或公式改成（时限 + 35） | `research/scripts/detached-run.sh:62`、`:522` | G5 |
| 4 | watch 开跑前登记就不在的单元，list 列、watch 不等，第一轮就退 0 | `research/scripts/detached-run.sh:55`、`:601` | G1 |
| 5 | R1 第 23 行「残留单元 0」在引用的日志里逐字找不到 | `scratchpad/t4repro.log` | R1 改写（一句） |
| 6 | 正文说 `main-agent.md` 被改 3 行，实际 4 行 | 第二轮、这一轮副本 `diff` | 正文 |

## 9 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| G1 | 做到（残留 1） | 触发场景（观察之后被删、systemd-run 报错而单元照起）全修，自证与开关读日志都红；开跑前就没登记的单元 watch 不等，是实现员划的界 |
| G2 | 做到（残留 2） | 起之前核日志与登记、写不进回滚退 1、旁路单列；mkdir 失败的分支不挪回旧登记，挪回失败无输出，都不丢数据 |
| G3 | 做到（字面） | 与预检逐词一致，理由回原文站得住，判红样本我独立喂出 heavy=2；「同样理解」有旧盲区（线索 1）交 U2 |
| G4 | 做到 | b、c、f、g 四个开关各红点名的格；h4、h5、h7 我独立复现，开关翻转都对 |
| G5 报警 | 做到 | 替身 systemctl 下交替清零、连续 3 次报警、不退不停，我跑过 |
| G5 stop | 做到，有小问题 | 选「写明前台 timeout」，理由属实；205 与 210 不一致，240000 没写出处 |
| G5 与规则 | 对得上，不必写进规则 | 不冲突；缺口只在换新会话后的主 agent；最小补法是 `.claude/main-agent.md:40` 加一句指路，要走三方或用户豁免，由主 agent 定 |
| R1 的数 | 全部找到，一处例外 | 「残留单元 0」逐字找不到，其余逐字或重跑对上 |
| R2 的数 | 全部找到 | 52 / 44 / 8 重跑一致，8 条主仓行今天全命中，复现日志逐行在 |
| 正文的数 | 一处对不上 | `main-agent.md` 实际被改 4 行不是 3 行 |

## 10 没做什么

- 没跑任何 `--selftest`，没起任何 systemd 单元（派发提示不许；子 agent 闸拒 `start` / `stop`）。自证 33 格、27 个开关各红的格、`feed3.py` 9 行、攻方 6 份复现的新版结果，我只读了主 agent 的验收日志，没有自己复跑。
- 没造 G2 的 mkdir 失败分支（要磁盘或 inode 耗尽），那一条是读代码推出来的，推的。
- 没验 Bash 的 120 秒默认 timeout 截断 stop 之后 systemd 的停机任务是否照做完（要起单元），头注的这句话我只核到了它写了什么。
- 没重跑 1 000 条登记的 list 耗时（U3 的格）；没查两台的 `Linger=yes`。
- 没判 U2、U3 的格，不采纳、不出判决；`main-agent.md` 第 52、61、63、66 行改了什么我不判。
- 钩子判定只在草稿副本里喂 JSON，没改主仓任何文件。草稿副本 `work/`、`work-r1/`、`work-r2/`（各约 6 MB，含 `.claude/singlefs-ai-sop` 的拷贝）交回前删，配方在模型目录 `README.md`；草稿目录里留脚本与输出。
