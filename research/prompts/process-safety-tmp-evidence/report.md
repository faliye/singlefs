# 进程安全闸 + 接手内存准入：交回报告（2026-09-25 JST 21:38 / UTC 12:38）

## 先看这一条：这一轮自己踩出来的一处，结局判不了
- `research/scripts/run-with-memory-cap.sh` 我用 `replace-once.py` 原地改（同一个 inode）。改的那一刻（约 UTC 12:18）别的 agent 正经它跑 `cargo test -p singlefs-harness --test s4_r2_candidates`（进程 2965927，上限 10G，UTC 11:57 起）。`/proc/2965927/fdinfo/255` 读到 `pos: 67774`：bash 按文件偏移往下读，它的命令跑完会从改后文件的第 67774 字节接着读——落在自检成功那句 echo 的中间，不是原来的 `exit $?`。
- 我想把原字节原地写回那个 inode，被权限分类器拒了（Irreversible Local Destruction），之后没再碰。UTC 12:38 查 `ps -p 2965927` 已经没有这个进程：**那一条 cargo test 的退出码与输出不可信，交主 agent 让它的属主重跑**。按我自己写的那几行推（没量过）：之后读到的是自检说明文字与 case 分派，没有 kill；最坏是报错退出、或把 run_capped 再跑一遍。
- 根因是流程：正在跑的 bash 脚本不能原地改，要写临时文件再改名换 inode（前任装 v2–v4 都是 cp + mv）。`replace-once.py` 与会话里的原地改法没有闸，记在提案第四十节第 32 行「欠」里，交主 agent 定要不要立规矩或改 replace-once.py（我没改，超出这一轮）。

## 第 1 步：进程安全闸（做完、证过）
1. 钩子：追加进 `.claude/hooks/bash-command-detector.sh`，当拒绝 ⑥（上游 `pattern-process-guard.sh` 管同类但不许就地改；`heavy-test-guard.sh` 管重型测试）。拒的写法与出路写在文件头 ⑥；拒绝信息出路是「记下它的 pid（`$!`）或任务号，一次停一个，单独 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`」。覆盖派发提示列的全部写法，外加补来的用户规矩（`kill $(pgrep …)`、`kill $(jobs -p)`、整批展开、`xargs kill` / `xargs -n1 kill`、`for … kill` 循环、`systemctl --user stop|kill` 带通配、`proc.py stop` 一次多个）。写死的进程号现读进程表：自己会话的祖先、pid 1、sshd/systemd/dbus/logind/claude、命令行带 `.vscode-server` 或 vllm、祖先里有另一个 Claude 会话的都拒。python -c 与喂给 python 的 heredoc 也判；shell 函数包着的 `bash -c` 递归判。
2. 脚本 lint：同一份判定经 `bash-command-detector.sh --scan-scripts <仓根> <目录>…` 交门禁 73 号，扫 research/scripts、.claude/hooks、.claude/scripts，另加 .claude/gate.d（样本目录不扫），按行号报。确实要按 cgroup 批量发的，那一行写 `# process-safety:own-scope <怎么核的>` 且往上 40 行有 `singlefs-memory-cap-*.scope` 的核对；只放行循环与按 cgroup 挑两类；标了却没东西可放行的判红；反引号里、写进文件的 heredoc、python 字符串里的标注不算。动不了的登记 `.claude/process-safety-pending`（指向不存在或一处没排到的判红）。
3. `run-with-memory-cap.sh` 逐处过、写进文件头「发信号的地方」：撞顶/超时由 systemd 停自己的 scope、reset-failed 自己的 unit、trap 只收不发、`kill -9 $$` 停自己。改了两处：风暴守卫改成核整条路径 `*/"$1"/singlefs-memory-cap-*.scope`（slice 名经 $1 传入；只核最后一段名字时，外面包着一层别的 singlefs-memory-cap scope 会打到外层）；slice 名只许 `^singlefs[-_A-Za-z0-9]*\.slice$`，别的退 2（自检加一项）。`research/scripts/check-staged.sh` 的 `kill -INT -- -"$job"` 改成 `kill -INT %1`（set -m 下按任务号发给那一组）。
4. 自证（字面输入只交给 hook 判，一条都不执行）：
   - hook 自检 230 → 352 种（终止进程单元例 101：该拒 68、放行 33；走真实入口 8；扫脚本 13）；`BASH_COMMAND_DETECTOR_ALLOW_PROCESS_SIGNALS=1` 红「终止进程」68、「stdin」5。改前 / 改后 hook 喂同一批（`compare-before-after.py`）：61 条改前退 0、改后退 2，`kill -TERM 1` 同样；放行 30 条里 29 条改后退 0，剩下那条前台 `while kill -0 … sleep` 是 ② 等待循环拒的，与 ⑥ 无关。`kill "$!"`、`proc.py stop 99999999` 经入口退 0。
   - 73 号经 `stage-selftest.sh` 喂（隔离的 gate 目录）：新 `✓ 73-research-gate-lint.sh red 判得对` / `green 判得对`；改前的 73 号红样本 7 条、绿样本 2 条 want 找不到。
   - 风暴守卫：发 TERM 换成往文件里记「想发给谁」（`extract-storm.py` 核过代码里只剩 `kill -0`）。在 session-1.scope 直接跑：新、旧守卫都记空；经包装 nocap：新守卫记空；外面包一层别的 singlefs-memory-cap scope：新守卫记空，**旧守卫记下外层外壳 1 个进程号**；这一例自己的 slice 底下自己的 scope（对照）：新守卫记下 1 个进程号。日志 `verify-storm-guard.log`。
   - check-staged：`--selftest` 通过；`CHECK_STAGED_NO_TRAP=1` 照旧报「关掉 trap 确认判红」。

## 第 2 步：内存准入（接手核现场后做完）
- 现场：前任 v4 在仓里，四层都在；8 个后台任务已随会话断掉。守卫证过之后才跑的弄坏开关（`break-switches.log`，286 秒）：自检 29 项全过；nocap 21、fallback 1、exitcode 4、noresult 3、noslice 11、slicefallback 3、nolock 1、noledger 3、nodefault 4、ignoretable 3、noslicehit 1、notimelimit 1、keepstale 2 项红；第 28 行那一版包装跑新自证红 15 项；v4 跑新自证只红 slice 名那 1 项；跑完 0 个自证 slice 挂着，nocap 那一轮会话无恙。
- 59 号样本经 `stage-selftest.sh` 喂：`✓ 59-crates-mutation-replay.sh red 判得对`、`green 判得对`（21 秒）。
- 提案 `records/2026-09-16-subagent拆分提案.md` 第四十节插了第 30 行（内存准入，更新了前任草稿的数）与第 32 行（进程安全闸），用 insert-row.py，写前复核 sha256 未变。

## 门禁原样判定行
- 47（150 秒，rc=1）：`  ✗ python3 research/scripts/check-segment-registry.py --selftest 没过（退出码 1）：`——kb 段序列 `16+2+1+2` 对产物 `24+2+1+2`，别的会话的；包装与 mutate.sh 的自证都过。
- 62：`  ✓ 阶段归属表与门禁目录一致（75 个阶段，归 9 个 agent）`
- 63：`  ✓ 写范围闸、Bash 检出 hook、重型测试闸、续派闸、续做闸与弹窗断言闸注册着、自证通过，…共用切词模块的 14 个函数只在 lib_shell_words.py 里定义（查了 .claude/hooks/ 下另外 11 个文件），…`
- 73（rc=1）：`  ✓ 门禁自检通过：81 个脚本（.sh 与 .py）、239 条拒绝都带了出路`；`  ✓ 进程安全：查了 164 个脚本（.sh 126 个、.py 38 个），…own-scope 标注放行 1 处（research/scripts/run-with-memory-cap.sh:881）；没判的：.claude/process-safety-pending 里的 2 个文件 research/scripts/agent-watch.py（…:2160）；research/scripts/mutate.sh（…:351）`；shell-lint 三个目录 `✓`；红的只有 `  ✗ research/scripts/changed-paths.sh 在暂存区里是 100644，.sh 要可执行`——改前的 73 号在同一份仓上同样报这一行，是 67f2c78 提交时的执行位，不是这一轮。
- doc-lint：`  ✓ 文档铁律检查通过（检查 485，跳过 0；DOC_LINT_VERBOSE=1 看全部）`
- hooks-registered：`  ✓ 10 个钩子都注册着，其中 9 个的自检通过`
- gate-overlap：`  ✓ 相对 8d7bd9d23ae5：新加的门禁与钩子 3 个（…ask-user-claim-guard.sh、…heavy-test-guard.sh、…session-start.sh）都写明了比过谁，改过的 22 份脚本对照已有的 123 份没有整段相同`

## 改了哪些文件（都没暂存）
- `.claude/hooks/bash-command-detector.sh`（sha256 b07ddbfefd9d5718…）、`.claude/gate.d/73-research-gate-lint.sh`（067e28d34965a292…）
- 73 号样本：red 加 `research/scripts/stop-processes.sh`、`research/scripts/stop.py`、`.claude/gate.d/50-stop-group.sh`，green 加 `research/scripts/stop-own.sh`，两份 expect 加 want
- `research/scripts/run-with-memory-cap.sh`（c8db93d7b6e914aa…，未跟踪）、`research/scripts/check-staged.sh`（0b4320a77f22f8d6…）
- 新建 `.claude/process-safety-pending`（0e3f0cd73c122390…）；`records/2026-09-16-subagent拆分提案.md` 第四十节第 30、32 行
- 草稿：`/tmp/claude-1000/process-safety/`（改前副本 `before/`、补丁 `patches/`、各日志；只是在说当时草稿放哪，不是这一轮要核的依据）

## 欠（交主 agent）
- 待改清单两份：`mutate.sh` 收尾循环 kill 自己的工作进程（归在改它并行的 agent）；`agent-watch.py` 自检收尾循环 os.kill 自己的子树（有别的会话没提交的改动）。改法：按任务号，或整批放进一个自己开的 scope 停那一个 scope。
- `.claude/agent-common.md` 第 57 行与 `.claude/main-agent.md`「派出去之后」列 hook 拒哪几种写法，还是五种、没写 ⑥（改定义走门禁 72 号那一轮三方）。
- 正在跑的脚本被原地改（见开头）：没有闸。
- 判不到的写法在 hook 文件头 ⑥ 末尾（变量里拼的命令、eval、`kill $pids`、先写文件下一条再读来 kill、循环里 `bash -c "kill $p"`、subprocess 起的 kill 与 Popen.terminate/kill、进程表读不到的写死进程号）。

## 推翻条件
- 会话里有一条打得到别人进程的终止写法经 hook 退 0，或脚本里有一处这类写法 73 号不报、也不在待改清单里，第 1 步就不成立。
- 风暴守卫：在不是「这一例自己 slice 底下自己的 scope」的 cgroup 里，记录版记下了任何进程号，守卫就不成立。

## 没做什么
- 没跑层 0、QEMU、herd7、crates 变异整表、全量 cargo test、gate.sh 整轮；没提交、没暂存。
- 没改 agent-common.md、main-agent.md、agent 定义、replace-once.py、mutate.sh、agent-watch.py。
- 没修 47 号的 check-segment-registry 红与 73 号的 changed-paths.sh 执行位红（都不是这一轮的）。
- 被拒的那一步（把原字节写回在跑的包装读的那个 inode、读那段偏移之后的内容）没再做，交主 agent 或用户定。
