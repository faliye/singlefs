# shell-words-r3 正推腿乙报告（格 X2：留到下一轮的清单全不全）

日期 2026-10-28。立场：只从两份钩子文件头写明的判据与 `.claude/main-agent.md`、`.claude/agent-common.md` 里照抄的起法出发，造正当命令与各类该拒写法，喂冻结副本里的 `heavy-test-guard.sh`、`bash-command-detector.sh`，起真 bash 核每条命令真跑起来做什么，判错的逐条归类。

## 零、复跑命令与模型目录 sha256

模型目录：`research/prompts/shell-words-r3-sonnet-s2-model/`（47 个文件）。复跑命令（分块，每块前台几分钟；`--stride 8` 每 8 条取 1 条，全量去掉 `--stride`，机器忙时全量要半小时以上）：

```
M=research/prompts/shell-words-r3-sonnet-s2-model
W=<新的草稿目录>
bash scripts/hosts/run-with-memory-cap.sh 4G bash $M/tools/rerun.sh --repo <仓根> --frozen ~/.local/share/singlefs/work/shell-words-r3-frozen --work $W --part setup
for p in L F S D M G; do bash scripts/hosts/run-with-memory-cap.sh 4G bash $M/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work $W --part $p --stride 8; done
bash $M/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work $W --part report   # 末行「对比 N 条…不一致 0」，退出码 0
```

我自己跑的：内存包装排队排不上（`run-with-memory-cap.sh` 报"排队 99 秒"，机器此刻负载 100–200），按派发提示改用 `nice -n 19` 加 `ulimit -v 4194304`（4G）直接跑，每一块都是。上面这串我在新 shell 里按块各跑了一遍：`setup`、`L F S D M G`、`report` 每块退出码都是 0，`report` 末行「对比 341 条（新跑 341，报告里 2719），不一致 0」（341 = 全量 2719 条的每 8 条取 1 条；该判与钩子判逐条同）。**没有整份 `--part all` 全量复跑过**（机器负载下太慢），这一点算复核不了的一部分。

`tools/` 是装置（桩树、起真 bash、该判算法、语料、喂钩子、归类、报告生成），`out/` 是全量那一趟的原始记录（`*.jsonl` 每条样本一行：命令、角色、该判与依据、两个钩子的退出码与 stderr 头一句、真 bash 观测）与汇总（`summary.txt`、`table.md`、`znotes.md`、`mismatches.tsv`）。

```
066b786246e1b53b924849feb3508d28bb984437002551faa77aa5e915a3a61e  ./out/D-a.jsonl
8aee45da3d2598ee53aa6cd0b8133c29303e6cf2e325f052446fb4bc4597c466  ./out/F-a.jsonl
f08065a2698b3d55b4adeba809fe395a6ee1e211057b5a402c02cd5a806ee903  ./out/G-a.jsonl
992145a0b8f9960a9daba5d8e0fe28d66f31932f8a1c0adb18423a930410ef23  ./out/G-z.jsonl
7825ce329ff9b7f986246fb5ed2b795faba0ca7366a496749357c25d7d07c5dd  ./out/L-a.jsonl
c140954e066f2d23d68e512c41629f65e501db8556f6b5a5270eca3295da5cf6  ./out/M-a.jsonl
ca21a39c1cf542a33d4ce4a921e693aad91f5534f8983b2ccfb547e9ca9f9bb6  ./out/mismatches.tsv
5a51dad2c15f20b598fbc92e453588b32cc421eab644458ff30acfb9808b2243  ./out/M-z.jsonl
eb78e9017685592c8403e1298f4c836a846ee75cc59d83a7021d866aeccf1717  ./out/S-a.jsonl
f63cc6a5e4e117d965e76d8b5a48fdda4e4c17db327c96f35ce0d8c8a3a419b2  ./out/summary.txt
4aa4fb46a0510030df9ed40c21b0660c867dd2850cd1ebcfb3307bb6d1b872f0  ./out/table.md
e276f1cd35653baa26819869cf1ea5d9df050caa8a56332267bd71c7aef1885b  ./out/znotes.md
f404f475beec7ec9a5ca6f364648d215c14604b4a03f6fabaa73063ac9e5687e  ./tools/analyze.py
f9a4db95e44de96a060c52e848d434276b49cb0180f16447bb3d01b7d1a6c049  ./tools/bin/cargo
30ad1d99f661341ee8c56f0a99b1339ddcd558d0c27edde69856acff02a4ee60  ./tools/bin/claude
546a5cc46b7c4e7cac51309dc3dc7ed90e14e9b1ad6b8876674a43b34d097002  ./tools/bin/doas
6724575f241fe6409a64069bf6b2b47ce143f9138827fa2a304e74ded5425e2e  ./tools/bin/herd7
0cc0385b933e1ebcb305bf83ddb4419c3c4a3f35cc48d59363744aaf3f83eafb  ./tools/bin/numactl
1479f3fca0b25f1936a9e2b0ebb9b236c83dafcb8e448e5583bf2d81a79f2a76  ./tools/bin/perf
16ef32a6bd8546d9935d8dfb1c30100c3909944befc575a3a015b63d8218705e  ./tools/bin/qemu-system-x86_64
d0129a0f7310cd514221526d8aa151cfba1d887ec8be26d2378fc38fd78c241b  ./tools/bin/rustup
4b06d3592812f54b223b0ccf834865c109eafd94ff1622f9303e064abecb7cc0  ./tools/bin/screen
b1e6009c94b2b888a6b56466e5971c84269502a34997d4965f5db68e3d43c708  ./tools/bin/ssh
6fbfb60867fd73844812ee1282f3857a2668d3dd000f52359264b1659107c9cb  ./tools/bin/strace
ac74be90cbda3eaf41945a8551c4ad1fadc46432199211174bb64c4e97cec310  ./tools/bin/sudo
b56da14eb90cba95d756f1fef106619f506666d600afcfe8b644e215576bc701  ./tools/bin/systemd-run
98deeec01617239377c58178a21630d66dd1383a880e0a4cf994c04770829d72  ./tools/bin/tmux
f05036bb4e652d9c411125d5e5d947d36a99c1e416ac0160c8c71c097579ff86  ./tools/bin/verus
aa049e6c6965e17c9647048dc536940a17ee96d465b09e640a4d27753b87bf2a  ./tools/build_tree.sh
d9fc1807ea68d5343127fa2328acd7d4dd77036243ba2b49a79ee4918d3591cc  ./tools/classify.py
ae60696bb5bf8a5200ce281161df1a6cd89648ac1c58ba107fd4724e66b9a3bb  ./tools/compare.py
1e07eccc1fa2ee612f0948d2051021aed4ecc267ea3f84f2fadc3312442d696c  ./tools/corpus_base.py
f338041d4a001719a239071af4e5077cce14921b4b20460323469f0f04a813c5  ./tools/corpus_detect.py
fa326e6675ad2c04704cc85c1424d398c89e3ef664b6c105c2c765fae3182072  ./tools/corpus_forms.py
cef93b6ba643553227250bf78a39bd4a69628187b464fa92c6dbbd4a30b3a34b  ./tools/corpus_launch.py
8562797260be1eeeb7f9c4c482c9289b8d213f1b5bdd872de4b4f14e4c49e807  ./tools/corpus_misc.py
abe6c8249b6f9b9d625af03a13ceff50ef6b4b5e87411d9db6de51922a4f385d  ./tools/corpus_more.py
746bc4839b129045beef9e114e55d5bc1e4205bbfbfe89fa0399b5baee834193  ./tools/corpus_scripts.py
df98b62ee9a50feb20dee69be58d05c2698d956b0f5c15e86339fc7500407ad7  ./tools/feed.py
98812aaa512190cc065cb3b64eb857b4025c49b0ed3740ec5fde7b11b6bf41a5  ./tools/mkreport.py
f3691798d056a3a0c4bf9ca475530857fe5eaa5d677928d14dc3afc264636a89  ./tools/oracle.py
fa24850a42de1806c28a9ad2dc138c4fc7a6e5665add7e843ee2db369affba9a  ./tools/policy.py
c8ec33233686cf2f78816171e6c2887dedd78f0798a36af65ffff763fc24261a  ./tools/report.py
cb6003889d3670384ed332c218a94985b7859961ce702d6833a6fab877b67376  ./tools/rerun.sh
64630d258eba2887a5bb655653d29a706f9ae727a8f3c6d2a9906196e0605e40  ./tools/run_all.py
b37d0c3d6bed2f9858e9aa826c9ad442afc3b8bac7807f31d9f4e23db7e30b2c  ./tools/run_shards.py
2f7e60fd46512102ed29e6c30b64402406a5754fecf88d8597b25f2a3c48119b  ./tools/stublib.sh
```
## 一、做法（只从钩子文件头与两份定义出发）

读过的：`heavy-test-guard.sh` 与 `bash-command-detector.sh` 的文件头判据（冻结副本 `~/.local/share/singlefs/work/shell-words-r3-frozen/.claude/hooks/`），`.claude/main-agent.md`、`.claude/agent-common.md` 里照抄的起法（`bash scripts/hosts/capped.sh N bash scripts/hosts/run-with-memory-cap.sh 4G cargo test …`、`SINGLEFS_HEAVY_TESTS=commit|user-request` 前缀、`detached-run.sh start … -- <命令>`、`watch.sh` 用 run_in_background 起、拒绝写法 ①–⑮）。没读 diff、前两轮腿报告与反例、修补报告；没读钩子的实现代码（`lib_*.py` 与 `.sh` 里 `python3 /dev/fd/3` 之后的部分）——判错之后只看钩子的 stderr 头一句来分是哪一道拒的。

1. **桩树**（`tools/build_tree.sh`）：把仓里钩子会认名字的脚本（`capped.sh`、`run-with-memory-cap.sh`、`detached-run.sh`、`gate.sh`、`.claude/gate.d/*.sh`、`crash-point-replay.sh` 等）换成"只记录自己被谁带什么参数、什么前缀起了"的桩，包装类的桩照常把后面的命令跑起来（不 exec，留在祖先链上，所以 `cargo` 桩能认出自己是不是在 `run-with-memory-cap.sh` 底下）。`PATH` 前面放假的 `cargo`、`claude`、`tmux`、`screen`、`systemd-run`、`sudo`、`doas`、`ssh`（把后面当本地命令跑）、`perf`、`numactl`、`rustup`、`strace`、`herd7`、`qemu-system-x86_64`。钩子读到的就是这棵树（`.claude/hooks/` 下五份换成冻结副本里的，另外的 hook 辅助模块照仓里的拷）。
2. **起真 bash**（`tools/oracle.py`）：`bash -c <命令>`，每条放进自己的会话（`start_new_session`），超时与结束都杀整组；环境里带一个 `ORACLE_MARK`，结束后按 `/proc/*/environ` 扫带标记的残留进程（`setsid -f` 逃出会话的也扫得到）并杀掉；写盘用 `git status` 与硬链接钉住的 inode 号看。登录 shell（`bash -lc`）读的启动文件在假家目录里，把桩目录放回 `PATH`。
3. **该判什么**（`tools/policy.py` 与 `tools/run_all.py` 的 `extra_reasons`）：对真 bash 留下的痕迹（哪个桩被起、带什么参数、什么前缀、有没有内存包装在祖先里、命令跑没跑满超时、有没有进程残留、仓里哪些文件被改、暂存区有没有被清空）按两份定义写明的规矩算。**判据是真跑出来的事实，不是命令的字面**：例如 `false && cargo test --workspace` 真 bash 里没起 cargo，该放；`xargs cargo test --workspace` 真起了，该拒。放出追踪（③）、`&` 与 `wait`（④）、看门狗（①）、kill（⑥）这几类真 bash 只用来核"命令跑完有没有进程留下"，该判是手标的，依据写在样本 `note` 里。
4. **喂钩子**（`tools/feed.py`）：每条样本按 hook 的 JSON 输入喂草稿目录里那两份 PreToolUse[Bash] 钩子（`heavy-test-guard.sh`、`bash-command-detector.sh`），角色三种：主 agent（不带 `agent_type`）、`three-way-forward`（子 agent，无 Edit）、`implementation-writer`（子 agent，有 Edit）；`run_in_background` 真假都有。`SINGLEFS_WORK_ROOT` 指到草稿目录下的 `wr/`，保证桩树不在工作根里面（否则写范围判不准）；仓副本放在 `wr/copy`。任一钩子退 2 记"拒"。
5. **范围**：2 个钩子 × 3 种角色 × 2719 条样本（每条按角色展开）（启动器矩阵 L、语法形态 F、脚本写出与执行 S/SW、等待循环与放出追踪 D、claude/detached-run/看门狗/kill/git 索引/覆盖/同 inode/写范围 M、间接启动与按名字判的重型脚本与花括号 G/N/B）。语料全部在 `tools/corpus_*.py`。

**局限**（先写在前面）：①ssh/doas/script/unshare 等名单外启动器用的是假的或系统自带的，ssh 只当"本地再起一个 bash"；②`kill`/`pkill` 这类能伤别人的只跑了点名单个 pid 的几种，别的按文件头手标不跑；③`env -i` 之类清掉前缀的，按"日志里更早的任何一条桩记录带过前缀"算带了；④上游 `pattern-process-guard.sh`、`run-scope-guard.sh` 等不在这一轮的五份钩子里，没喂；⑤机器负载 100+，超时给到 30 秒（等待循环 8 秒），跑满超时的才算"没停"。

## 二、总数与结论（X2）

跑了 2719 条样本（语料 `tools/corpus_*.py`，家族 L 启动器矩阵 456、F 语法形态 564、S/SW 脚本写出与执行 330+228、D 等待循环与放出追踪 336、M claude/detached-run/看门狗/kill/git 索引/覆盖/同 inode/写范围 280、G 间接启动 225、N 按名字判的重型脚本 246、B 花括号与 cat 源文件不在 51）。真 bash 与钩子一致的 2507 条；**判错 212 条（逐样本），合并成 116 条不同的命令**（几个角色、带不带前缀的同一条命令并成一行）。

| 归类 | 命令数 | 一句话 |
|---|---|---|
| A1 拷出写法（`>>`、`&>`、`>&`、`tee`） | 9 | 清单里的，复现（文件头 `heavy-test-guard.sh:176` 已写明认不出） |
| A2 `bash -s < 文件` | 7 | 清单里的，复现；同一原理的 `bash < 文件`、`sh -s < 文件`、`cat 文件 \| bash` 也漏 |
| A3 xargs（含 `xargs env`） | 13 | 清单里的，复现；`xargs claude`、`xargs bash 脚本` 同漏 |
| A4 名单外启动器 | 15 | 清单里的，复现：`find -exec/-execdir`、`ssh`、`doas`、`script -qec`、`unshare`、`setpriv`、`numactl` 漏；`sudo`、`ionice`、`taskset`、`stdbuf`、`flock`、`chrt`、`prlimit`、`nohup`、`setsid`、`strace`、`perf`、`rustup run`、`/usr/bin/time`、`env`、`nice`、`timeout`、`exec`、`command`、`time` 全都认得 |
| B1 cat 不看源文件在不在 | 0 | 没复现：我造的 `cat 不存在的文件 > t.sh; bash t.sh` 三种，钩子判得对（放） |
| B2 重定向目标花括号展开成几个词 | 3 | 清单里的，复现（detector ⑫ 对有 Edit 的子 agent：`> o.{a,b}`、`> o{1..2}.txt`、**加引号的** `> 'o.{a,b}'`） |
| B3 函数体之外引号里的花括号 | 0 | 没复现：四种写法（`f() { :; }; echo '{ x }'; f` 等）都判对 |
| C 函数当启动器（`run_logged` 一类） | 6 | 清单里的，复现：`run() { "$@"; }; run cargo …`、`run_logged log cargo …`、`function run { "$@"; }`、`w() { nice "$@"; }; w cargo …`、`r() { env "$@"; }`、`m() { …run-with-memory-cap.sh 4G "$@"; }` |
| **Z 不在清单里** | **63** | 下面「五」逐子类写真 bash 行为与钩子退出码；其中文件头已写明看不见或接受的 38 条（Z1、Z2、Z3、Z4、Z9、Z10、Z11、Z12），**文件头写了认、实际不认的 1 类 2 条**（Z13），文件头没写的 23 条（Z5、Z6、Z7、Z8、Z14、Z15、Z16） |

**结论一句**：留到下一轮的清单不全。清单里 A1–A4、B2、C 都复现了，B1、B3 在我造的写法里没复现；**清单之外还有 16 个子类**，其中最该进下一轮的是三类——(1) 文件头写了会认、实际不认的：bg 里 `tail --pid` / `kill -0` 轮询等它（Z13，文件头 `bash-command-detector.sh:29` 写了算等，实际按没等拒），以及文件头没写到的 ⑬ 的 here-string 与 `env -S`（Z6：`bash <<< 'claude -p hi'`、`env -S 'claude -p hi'` 漏，重型闸与 ① 对同一写法认得）；(2) 与 A4 同类的新启动器，加上 `bash -c '…"$@"…' _ cargo test`、`sh -c 'exec "$@"' sh cargo test` 这种位置参数拼命令（Z16）、`cargo test $F`、`-p $P`、`cargo $(echo test …)` 旗标由变量拼（Z15）；(3) 没被调用的函数体、永远走不到的分支按字面拒（Z5，误拒方向）。

**什么现象会推翻这一条**：下一轮把清单里的类修完之后，用本模型目录的 `tools/rerun.sh` 重跑，Z 子类里任何一条变成"钩子判对"，该子类就不再是清单外；或者我造的某条样本其实不能照真 bash 的行为解释（见「七」里列的我自己改掉的样本毛病），则该条撤回。

## 三、文件头原文（引文核对用）

下面各条是正文里用到的文件头原句。冻结副本与仓里这五份逐字节相同（主 agent 的 `cmp`，我也 `cmp` 过），行号按 `.claude/hooks/` 下的写：

- `.claude/hooks/heavy-test-guard.sh:176`：「认不出的写法（printf、echo、python 写文件、带选项的 cat、>> / &> 写出的 cat 拷贝、管道 cat <<EOF | tee x.sh、变量拼出的目标）照旧」
- `.claude/hooks/heavy-test-guard.sh:182`：「标准输入从文件来）判不了：照常执行，stderr 报一句、记一条检出」
- `.claude/hooks/heavy-test-guard.sh:195`：「看不见的（照常放行、不记检出）：变量里拼出来的命令与脚本路径」
- `.claude/hooks/heavy-test-guard.sh:195`：「通配与花括号写出来的脚本里的正文、eval」
- `.claude/hooks/heavy-test-guard.sh:197`：「make（Makefile 里起的命令）与 xargs 起的命令」
- `.claude/hooks/heavy-test-guard.sh:199`：「命令词不带斜杠、从 PATH 里找到的脚本」
- `.claude/hooks/bash-command-detector.sh:29`：「（wait -n 不算），或之后有 tail --pid / while|until … kill -0 轮询」
- `.claude/hooks/bash-command-detector.sh:67`：「判不到的：变量里拼出来的命令、eval、喂给 shell 的不是字面」
- `.claude/hooks/bash-command-detector.sh:109`：「带计数上限的 while 循环机器分不出上限，照拒，外面套一层 timeout 就放行」

## 四、判错表（每条命令一行；几个角色、带不带前缀的同一条命令并成一行）

读法：`{U}` = 每条样本自己的草稿子目录，`{C}` = 仓副本 `wr/copy`，`{R}` = 桩树根，`{W}` = 工作根 `wr/`，`{D}` = 草稿目录；`↵` = 命令里的换行。「该判」括号里是依据：H1 重型、H2 子 agent 不经内存包装跑编译出来的代码、D2/D10/D13 等对应 `bash-command-detector.sh` 的 ② ⑩ ⑬、W5/W7/W8/W12 写盘、G15 索引。「钩子判」括号里两个数是 `heavy-test-guard.sh` / `bash-command-detector.sh` 的退出码。「角色」：主 = 主 agent（无前缀/带前缀），子 = `three-way-forward`，子(Edit) = `implementation-writer`，(bg) = run_in_background。

| # | 命令原样 | 角色 | 该判 | 钩子判（重型闸/检出钩子退出码） | 归类 | 文件头写明看不见/接受 |
|---|---|---|---|---|---|---|
| 1 | `cat <<'EOF' \| tee -a {U}/w.sh >/dev/null↵cd {C} && cargo test --workspace↵EOF↵bash {U}/w.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A1 脚本拷出用 >> / &> / >& / tee 再执行 | 是 |
| 2 | `cat <<'EOF' \| tee {U}/w.sh >/dev/null↵cd {C} && cargo test --workspace↵EOF↵bash {U}/w.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A1 脚本拷出用 >> / &> / >& / tee 再执行 | 是 |
| 3 | `cat > {U}/w.sh <<'EOF'↵cd {C} && cargo test --workspace↵EOF↵cat {U}/w.sh &> {U}/w2.sh; bash {U}/w2.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A1 脚本拷出用 >> / &> / >& / tee 再执行 | 是 |
| 4 | `cat > {U}/w.sh <<'EOF'↵cd {C} && cargo test --workspace↵EOF↵cat {U}/w.sh >> {U}/w2.sh; bash {U}/w2.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A1 脚本拷出用 >> / &> / >& / tee 再执行 | 是 |
| 5 | `cat {U}/s.sh &> {U}/cat.sh; bash {U}/cat.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A1 脚本拷出用 >> / &> / >& / tee 再执行 | 是 |
| 6 | `cat {U}/s.sh >& {U}/cat.sh; bash {U}/cat.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A1 脚本拷出用 >> / &> / >& / tee 再执行 | 是 |
| 7 | `cat {U}/s.sh >> {U}/cat.sh; bash {U}/cat.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A1 脚本拷出用 >> / &> / >& / tee 再执行 | 是 |
| 8 | `cat {U}/s.sh \| tee -a {U}/t.sh >/dev/null; bash {U}/t.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A1 脚本拷出用 >> / &> / >& / tee 再执行 | 是 |
| 9 | `cat {U}/s.sh \| tee {U}/t.sh >/dev/null; bash {U}/t.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A1 脚本拷出用 >> / &> / >& / tee 再执行 | 是 |
| 10 | `bash -s < {U}/s.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A2 标准输入从文件来的 shell（`bash -s < 文件`、`bash < 文件`、`cat 文件` 经管道进 bash） | 是 |
| 11 | `bash < {U}/s.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A2 标准输入从文件来的 shell（`bash -s < 文件`、`bash < 文件`、`cat 文件` 经管道进 bash） | 是 |
| 12 | `cat > {U}/w.sh <<'EOF'↵cd {C} && cargo test --workspace↵EOF↵bash -s < {U}/w.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A2 标准输入从文件来的 shell（`bash -s < 文件`、`bash < 文件`、`cat 文件` 经管道进 bash） | 是 |
| 13 | `cat > {U}/w.sh <<'EOF'↵cd {C} && cargo test --workspace↵EOF↵cat {U}/w.sh \| bash` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A2 标准输入从文件来的 shell（`bash -s < 文件`、`bash < 文件`、`cat 文件` 经管道进 bash） | 是 |
| 14 | `cat {U}/s.sh \| bash` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A2 标准输入从文件来的 shell（`bash -s < 文件`、`bash < 文件`、`cat 文件` 经管道进 bash） | 是 |
| 15 | `sh -s < {U}/s.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A2 标准输入从文件来的 shell（`bash -s < 文件`、`bash < 文件`、`cat 文件` 经管道进 bash） | 是 |
| 16 | `tee {U}/t.sh < {U}/s.sh >/dev/null; bash {U}/t.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A2 标准输入从文件来的 shell（`bash -s < 文件`、`bash < 文件`、`cat 文件` 经管道进 bash） | 是 |
| 17 | `cat > {U}/w.sh <<'EOF'↵cd {C} && cargo test --workspace↵EOF↵echo {U}/w.sh \| xargs bash` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A3 xargs 起的命令/脚本（含 xargs env、xargs bash） | 是 |
| 18 | `echo FOO=1 \| xargs env cargo test --workspace` | 主带前缀、主无前缀、子 | 拒（H1; H2） | 放（0/0） | A3 xargs 起的命令/脚本（含 xargs env、xargs bash） | 是 |
| 19 | `echo FOO=1 \| xargs env cargo test -p singlefs-harness --test t1` | 子 | 拒（H2） | 放（0/0） | A3 xargs 起的命令/脚本（含 xargs env、xargs bash） | 是 |
| 20 | `echo extra \| bash {R}/scripts/hosts/run-with-memory-cap.sh 4G xargs cargo test --workspace` | 主带前缀、主无前缀、子 | 拒（H1） | 放（0/0） | A3 xargs 起的命令/脚本（含 xargs env、xargs bash） | 是 |
| 21 | `echo extra \| xargs -I{} cargo test --workspace` | 主带前缀、主无前缀、子 | 拒（H1; H2） | 放（0/0） | A3 xargs 起的命令/脚本（含 xargs env、xargs bash） | 是 |
| 22 | `echo extra \| xargs -I{} cargo test -p singlefs-harness --test t1` | 子 | 拒（H2） | 放（0/0） | A3 xargs 起的命令/脚本（含 xargs env、xargs bash） | 是 |
| 23 | `echo extra \| xargs -I{} sh -c 'cargo test --workspace'` | 主带前缀、主无前缀、子 | 拒（H1） | 放（0/0） | A3 xargs 起的命令/脚本（含 xargs env、xargs bash） | 是 |
| 24 | `echo extra \| xargs -I{} sh -c 'cargo test -p singlefs-harness --test t1'` | 子 | 拒（H2） | 放（0/0） | A3 xargs 起的命令/脚本（含 xargs env、xargs bash） | 是 |
| 25 | `echo extra \| xargs cargo test --workspace` | 主带前缀、主无前缀、子 | 拒（H1; H2） | 放（0/0） | A3 xargs 起的命令/脚本（含 xargs env、xargs bash） | 是 |
| 26 | `echo extra \| xargs cargo test -p singlefs-harness --test t1` | 子 | 拒（H2） | 放（0/0） | A3 xargs 起的命令/脚本（含 xargs env、xargs bash） | 是 |
| 27 | `echo hi \| xargs claude -p` | 子 | 拒（D13） | 放（0/0） | A3 xargs 起的命令/脚本（含 xargs env、xargs bash） | 是 |
| 28 | `echo {U}/s.sh \| xargs bash` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A3 xargs 起的命令/脚本（含 xargs env、xargs bash） | 是 |
| 29 | `printf '%s\0' cargo test --workspace \| xargs -0 echo >/dev/null; printf '%s\0' cargo test --workspace \| xargs -0 -n3 env` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A3 xargs 起的命令/脚本（含 xargs env、xargs bash） | 是 |
| 30 | `bash {R}/scripts/hosts/run-with-memory-cap.sh 4G doas cargo test --workspace` | 主无前缀、子 | 拒（H1） | 放（0/0） | A4 名单外启动器：script | 否 |
| 31 | `bash {R}/scripts/hosts/run-with-memory-cap.sh 4G find . -maxdepth 0 -exec cargo test --workspace \;` | 主无前缀、子 | 拒（H1） | 放（0/0） | A4 名单外启动器：script | 否 |
| 32 | `bash {R}/scripts/hosts/run-with-memory-cap.sh 4G numactl -N0 cargo test --workspace` | 主无前缀、子 | 拒（H1） | 放（0/0） | A4 名单外启动器：script | 否 |
| 33 | `bash {R}/scripts/hosts/run-with-memory-cap.sh 4G script -qec 'cargo test --workspace' /dev/null` | 主无前缀、子 | 拒（H1） | 放（0/0） | A4 名单外启动器：script | 否 |
| 34 | `bash {R}/scripts/hosts/run-with-memory-cap.sh 4G setpriv --reuid=$(id -u) cargo test --workspace` | 主无前缀、子 | 拒（H1） | 放（0/0） | A4 名单外启动器：script | 否 |
| 35 | `bash {R}/scripts/hosts/run-with-memory-cap.sh 4G ssh localhost 'cargo test --workspace'` | 主无前缀、子 | 拒（H1） | 放（0/0） | A4 名单外启动器：script | 否 |
| 36 | `bash {R}/scripts/hosts/run-with-memory-cap.sh 4G unshare --user cargo test --workspace` | 主无前缀、子 | 拒（H1） | 放（0/0） | A4 名单外启动器：script | 否 |
| 37 | `find . -maxdepth 0 -exec cargo test --workspace \;` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A4 名单外启动器：find | 否 |
| 38 | `find . -maxdepth 0 -exec cargo test -p singlefs-harness --test t1 \;` | 子 | 拒（H2） | 放（0/0） | A4 名单外启动器：find | 否 |
| 39 | `find . -maxdepth 0 -execdir cargo test --workspace \;` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A4 名单外启动器：find | 否 |
| 40 | `find . -maxdepth 0 -execdir cargo test -p singlefs-harness --test t1 \;` | 子 | 拒（H2） | 放（0/0） | A4 名单外启动器：find | 否 |
| 41 | `find {U} -name 's.sh' -exec bash {} \;` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A4 名单外启动器：find | 否 |
| 42 | `ssh localhost 'cargo test --workspace'` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A4 名单外启动器：ssh | 否 |
| 43 | `ssh localhost 'cargo test -p singlefs-harness --test t1'` | 子 | 拒（H2） | 放（0/0） | A4 名单外启动器：ssh | 否 |
| 44 | `ssh localhost 'claude -p hi'` | 子 | 拒（D13） | 放（0/0） | A4 名单外启动器：ssh | 否 |
| 45 | `echo hi > '{D}/o.{a,b}'` | 子(Edit) | 放（-） | 拒（0/2） | B2 重定向目标花括号展开成几个词 | 否 |
| 46 | `echo hi > {D}/o.{a,b}` | 子(Edit) | 放（-） | 拒（0/2） | B2 重定向目标花括号展开成几个词 | 否 |
| 47 | `echo hi > {D}/o{1..2}.txt` | 子(Edit) | 放（-） | 拒（0/2） | B2 重定向目标花括号展开成几个词 | 否 |
| 48 | `function run { "$@"; }; run cargo test --workspace` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | C 函数当启动器（run_logged 一类） | 否 |
| 49 | `m() { bash {R}/scripts/hosts/run-with-memory-cap.sh 4G "$@"; }; m cargo test --workspace` | 主无前缀、子 | 拒（H1） | 放（0/0） | C 函数当启动器（run_logged 一类） | 否 |
| 50 | `r() { env "$@"; }; r cargo test --workspace` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | C 函数当启动器（run_logged 一类） | 否 |
| 51 | `run() { "$@"; }; run cargo test --workspace` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | C 函数当启动器（run_logged 一类） | 否 |
| 52 | `run_logged() { local l=$1; shift; "$@" > "$l" 2>&1; }; run_logged {D}/g.log cargo test --workspace` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | C 函数当启动器（run_logged 一类） | 否 |
| 53 | `w() { nice "$@"; }; w cargo test --workspace` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | C 函数当启动器（run_logged 一类） | 否 |
| 54 | `bash -c "$(cat {U}/s.sh)"` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z1 进程替换 / 命令替换里拼出命令或脚本 | 是 |
| 55 | `bash -c "$(printf '%s' 'cargo test --workspace')"` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z1 进程替换 / 命令替换里拼出命令或脚本 | 是 |
| 56 | `bash <(echo 'cargo test --workspace')` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z1 进程替换 / 命令替换里拼出命令或脚本 | 是 |
| 57 | `c='cargo test --workspace'; eval "$c"` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z1 eval 拼出命令 | 是 |
| 58 | `eval "$(cat {U}/s.sh)"` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z1 eval 拼出命令 | 是 |
| 59 | `source <(cat {U}/s.sh)` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z1 进程替换 / 命令替换里拼出命令或脚本 | 是 |
| 60 | `source <(echo 'cargo test --workspace')` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z1 进程替换 / 命令替换里拼出命令或脚本 | 是 |
| 61 | `i=0; while [ $i -lt 3 ]; do sleep 0.1; i=$((i+1)); done` | 主、子 | 放（-） | 拒（0/2） | Z10 真会结束的前台等待循环按字面拒（计数上限、文件已在、read 循环、break、kill -0 轮询） | 是 |
| 62 | `sleep 1 & p=$!; while kill -0 $p 2>/dev/null; do sleep 0.2; done` | 主、子 | 放（-） | 拒（0/2） | Z10 真会结束的前台等待循环按字面拒（计数上限、文件已在、read 循环、break、kill -0 轮询） | 是 |
| 63 | `until [ -f {U}/ready ]; do sleep 1; done` | 主、子 | 放（-） | 拒（0/2） | Z10 真会结束的前台等待循环按字面拒（计数上限、文件已在、read 循环、break、kill -0 轮询） | 是 |
| 64 | `while ! [ -f {U}/ready ]; do sleep 1; done` | 主、子 | 放（-） | 拒（0/2） | Z10 真会结束的前台等待循环按字面拒（计数上限、文件已在、read 循环、break、kill -0 轮询） | 是 |
| 65 | `while read -r l; do sleep 0.1; done <<< foo` | 主、子 | 放（-） | 拒（0/2） | Z10 真会结束的前台等待循环按字面拒（计数上限、文件已在、read 循环、break、kill -0 轮询） | 是 |
| 66 | `while sleep 0.1 && false; do :; done` | 主、子 | 放（-） | 拒（0/2） | Z10 真会结束的前台等待循环按字面拒（计数上限、文件已在、read 循环、break、kill -0 轮询） | 是 |
| 67 | `while true; do sleep 0.2; break; done` | 主、子 | 放（-） | 拒（0/2） | Z10 真会结束的前台等待循环按字面拒（计数上限、文件已在、read 循环、break、kill -0 轮询） | 是 |
| 68 | `sleep 1 & p1=$!; sleep 2 & p2=$!; wait $p1` | 主(bg) | 拒（D3/4） | 放（0/0） | Z11 两个 & 只等其中一个（wait $p1），另一个留下 | 是 |
| 69 | `sleep 1 & wait -n` | 主(bg) | 放（-） | 拒（0/2） | Z12 bg 里 `& … wait -n`（单个作业，真等到了）按没等算 | 是 |
| 70 | `sleep 1 & p=$!; while kill -0 $p 2>/dev/null; do sleep 0.2; done` | 主(bg) | 放（-） | 拒（0/2） | Z13 bg 里 `tail --pid` / `kill -0` 轮询等它，文件头写了算等，实际仍按没等拒 | 相反 |
| 71 | `sleep 2 & tail --pid=$! -f /dev/null` | 主(bg) | 放（-） | 拒（0/2） | Z13 bg 里 `tail --pid` / `kill -0` 轮询等它，文件头写了算等，实际仍按没等拒 | 相反 |
| 72 | `w() { wait; }; sleep 1 & w` | 主(bg) | 放（-） | 拒（0/2） | Z14 wait 写在函数里、函数被调用，认不出 | 否 |
| 73 | `F=--workspace; cargo test $F` | 主无前缀 | 拒（H1） | 放（0/0） | Z15 cargo 的旗标 / 包名 / 子命令由变量或命令替换拼出 | 否 |
| 74 | `P=singlefs-checker-tier; cargo test -p "$P"` | 主无前缀 | 拒（H1） | 放（0/0） | Z15 cargo 的旗标 / 包名 / 子命令由变量或命令替换拼出 | 否 |
| 75 | `P=singlefs-checker-tier; cargo test -p $P` | 主无前缀 | 拒（H1） | 放（0/0） | Z15 cargo 的旗标 / 包名 / 子命令由变量或命令替换拼出 | 否 |
| 76 | `cargo $(echo test --workspace)` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z15 cargo 的旗标 / 包名 / 子命令由变量或命令替换拼出 | 否 |
| 77 | `cargo $(echo test) --workspace` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z15 cargo 的旗标 / 包名 / 子命令由变量或命令替换拼出 | 否 |
| 78 | `t={U}/w.sh; cat > $t <<'EOF'↵cd {C} && cargo test --workspace↵EOF↵bash $t` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z15 cargo 的旗标 / 包名 / 子命令由变量或命令替换拼出 | 否 |
| 79 | `bash -c '$0 test --workspace' cargo` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z16 bash -c / sh -c 的脚本里用位置参数 $0 $1 "$@" 拼出命令 | 否 |
| 80 | `bash -c 'cargo "$1" "$2"' _ test --workspace` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z16 bash -c / sh -c 的脚本里用位置参数 $0 $1 "$@" 拼出命令 | 否 |
| 81 | `bash -c 'cargo "$@"' _ test --workspace` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z16 bash -c / sh -c 的脚本里用位置参数 $0 $1 "$@" 拼出命令 | 否 |
| 82 | `sh -c 'exec "$@"' sh cargo test --workspace` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z16 bash -c / sh -c 的脚本里用位置参数 $0 $1 "$@" 拼出命令 | 否 |
| 83 | `"$(command -v cargo)" test --workspace` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z2 命令词由命令替换 / set -- 算出 | 是 |
| 84 | `$(which cargo) test --workspace` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z2 命令词由命令替换 / set -- 算出 | 是 |
| 85 | `a=(cargo test --workspace); "${a[@]}"` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z2 变量里拼出命令词 / 脚本路径（含 for 循环变量、命令替换算出路径） | 是 |
| 86 | `bash $(echo {U}/s.sh)` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z2 变量里拼出命令词 / 脚本路径（含 for 循环变量、命令替换算出路径） | 是 |
| 87 | `c='cargo test --workspace'; $c` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z2 变量里拼出命令词 / 脚本路径（含 for 循环变量、命令替换算出路径） | 是 |
| 88 | `c=claude; $c -p hi` | 子 | 拒（D13） | 放（0/0） | Z2 变量里拼出命令词 | 是 |
| 89 | `for f in {U}/s.sh; do bash $f; done` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z2 变量里拼出命令词 / 脚本路径（含 for 循环变量、命令替换算出路径） | 是 |
| 90 | `set -- cargo test --workspace; "$@"` | 主无前缀、子 | 拒（H1） | 放（0/0） | Z2 命令词由命令替换 / set -- 算出 | 是 |
| 91 | `x=reset; git $x` | 主、子 | 拒（G15） | 放（0/0） | Z2 变量里拼出命令词（git $x） | 是 |
| 92 | `PATH={U} s.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z3 命令词不带斜杠、经 PATH 找到的脚本 | 是 |
| 93 | `dd if={U}/s.sh of={U}/dd.sh 2>/dev/null; bash {U}/dd.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z4 脚本用 dd / sed / head / printf / echo / install / python 写出再执行 | 是 |
| 94 | `dd of={U}/w.sh 2>/dev/null <<'EOF'↵cd {C} && cargo test --workspace↵EOF↵bash {U}/w.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z4 脚本用 dd / sed / head / printf / echo / install / python 写出再执行 | 是 |
| 95 | `echo 'cd {C} && cargo test --workspace' > {U}/w.sh; bash {U}/w.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z4 脚本用 dd / sed / head / printf / echo / install / python 写出再执行 | 是 |
| 96 | `echo 'cd {C} && cargo test --workspace' >> {U}/w.sh; bash {U}/w.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z4 脚本用 dd / sed / head / printf / echo / install / python 写出再执行 | 是 |
| 97 | `echo -e 'cd {C} && cargo test --workspace' > {U}/w.sh; bash {U}/w.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z4 脚本用 dd / sed / head / printf / echo / install / python 写出再执行 | 是 |
| 98 | `head -n 9 {U}/s.sh > {U}/h.sh; bash {U}/h.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z4 脚本用 dd / sed / head / printf / echo / install / python 写出再执行 | 是 |
| 99 | `install -m755 /dev/stdin {U}/w.sh <<'EOF'↵cd {C} && cargo test --workspace↵EOF↵bash {U}/w.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z4 脚本用 dd / sed / head / printf / echo / install / python 写出再执行 | 是 |
| 100 | `printf '%s\n' 'cd {C} && cargo test --workspace' > {U}/w.sh; bash {U}/w.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z4 脚本用 dd / sed / head / printf / echo / install / python 写出再执行 | 是 |
| 101 | `python3 - <<'PY'↵open('{U}/w.sh','w').write('cd {C} && cargo test --workspace')↵PY↵bash {U}/w.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z4 脚本用 dd / sed / head / printf / echo / install / python 写出再执行 | 是 |
| 102 | `python3 -c "open('{U}/w.sh','w').write('cd {C} && cargo test --workspace')"; bash {U}/w.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z4 脚本用 dd / sed / head / printf / echo / install / python 写出再执行 | 是 |
| 103 | `sed 's/x/x/' {U}/s.sh > {U}/se.sh; bash {U}/se.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z4 脚本用 dd / sed / head / printf / echo / install / python 写出再执行 | 是 |
| 104 | `case a in b) cargo test --workspace;; esac` | 主无前缀、子 | 放（-） | 拒（2/0） | Z5 永远不执行的分支 / 没被调用的函数体按字面拒 | 否 |
| 105 | `f() { cargo test --workspace; }; echo done` | 主无前缀、子 | 放（-） | 拒（2/0） | Z5 永远不执行的分支 / 没被调用的函数体按字面拒 | 否 |
| 106 | `f() { while true; do sleep 1; done; }; echo ok` | 主、子 | 放（-） | 拒（0/2） | Z5 没被调用的函数体按字面判（循环） | 否 |
| 107 | `false && cargo test --workspace` | 主无前缀、子 | 放（-） | 拒（2/0） | Z5 永远不执行的分支 / 没被调用的函数体按字面拒 | 否 |
| 108 | `for i in; do cargo test --workspace; done` | 主无前缀、子 | 放（-） | 拒（2/0） | Z5 永远不执行的分支 / 没被调用的函数体按字面拒 | 否 |
| 109 | `if false; then cargo test --workspace; fi` | 主无前缀、子 | 放（-） | 拒（2/0） | Z5 永远不执行的分支 / 没被调用的函数体按字面拒 | 否 |
| 110 | `until true; do cargo test --workspace; done` | 主无前缀、子 | 放（-） | 拒（2/0） | Z5 永远不执行的分支 / 没被调用的函数体按字面拒 | 否 |
| 111 | `while false; do cargo test --workspace; done` | 主无前缀、子 | 放（-） | 拒（2/0） | Z5 永远不执行的分支 / 没被调用的函数体按字面拒 | 否 |
| 112 | `bash <<< 'claude -p hi'` | 子 | 拒（D13） | 放（0/0） | Z6 ⑬ claude：here-string / env -S 里写 claude 认不出（①与重型闸对同一写法认得） | 否 |
| 113 | `env -S 'claude -p hi'` | 子 | 拒（D13） | 放（0/0） | Z6 ⑬ claude：here-string / env -S 里写 claude 认不出（①与重型闸对同一写法认得） | 否 |
| 114 | `bash {R}/scripts/agents/watch.sh \| tee /dev/null` | 主(bg) | 拒（W1） | 放（0/0） | Z7 ① 看门狗输出经 `| tee /dev/null` 丢掉 | 否 |
| 115 | `echo '# x' > {U}/n.sh; rsync --inplace {U}/n.sh scripts/hosts/capped.sh` | 主 | 拒（W7） | 放（0/0） | Z8 ⑦ 没点名 rsync --inplace（同 inode 改脚本） | 否 |
| 116 | `bash {U}/*s.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z9 bash 的脚本路径参数里带通配，没按 bash 展开去读 | 是 |

## 五、不在清单里的（这一格要交的）：每个子类的真 bash 行为与钩子退出码

每个子类给最多 4 条代表（同子类其余的在上表）；「文件头怎么说」写这一类文件头有没有写明看不见/接受。「真 bash」一栏是桩日志：哪个桩被起、带什么前缀、有没有内存包装在祖先里，加上超时/残留进程/仓内改动。钩子退出码：`heavy` = `heavy-test-guard.sh`，`bash` 或 `detector` = `bash-command-detector.sh`（0 放、2 拒）。

### Z1 进程替换 / 命令替换里拼出命令或脚本（14 个角色×写法，7 条命令）

文件头怎么说：是（heavy-test-guard.sh:195/182「喂给 shell 的不是字面」）

- `source <(echo 'cargo test --workspace')`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- `bash <(echo 'cargo test --workspace')`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- `bash -c "$(printf '%s' 'cargo test --workspace')"`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- `c='cargo test --workspace'; eval "$c"`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- 同子类另有 3 条，见判错表

### Z10 真会结束的前台等待循环按字面拒（计数上限、文件已在、read 循环、break、kill -0 轮询）（14 个角色×写法，7 条命令）

文件头怎么说：是（detector:109「带计数上限的 while 循环机器分不出上限，照拒」；其余同一原理）

- `i=0; while [ $i -lt 3 ]; do sleep 0.1; i=$((i+1)); done`（主、子）：该判**放**；真 bash：没有桩被起；bash 退出码 0；钩子退出码 heavy=0 detector=2
- `while ! [ -f {U}/ready ]; do sleep 1; done`（主、子）：该判**放**；真 bash：没有桩被起；bash 退出码 0；钩子退出码 heavy=0 detector=2
- `while read -r l; do sleep 0.1; done <<< foo`（主、子）：该判**放**；真 bash：没有桩被起；bash 退出码 0；钩子退出码 heavy=0 detector=2
- `sleep 1 & p=$!; while kill -0 $p 2>/dev/null; do sleep 0.2; done`（主、子）：该判**放**；真 bash：没有桩被起；bash 退出码 0；钩子退出码 heavy=0 detector=2
- 同子类另有 3 条，见判错表

### Z11 两个 & 只等其中一个（wait $p1），另一个留下（1 个角色×写法，1 条命令）

文件头怎么说：是（detector 文件头 ② 末「之后的 wait "$pid" 只等了其中一个」只记不拒）

- `sleep 1 & p1=$!; sleep 2 & p2=$!; wait $p1`（主(bg)）：该判**拒**；真 bash：没有桩被起；残留进程 sleep 2；bash 退出码 0；钩子退出码 heavy=0 detector=0

### Z12 bg 里 `& … wait -n`（单个作业，真等到了）按没等算（1 个角色×写法，1 条命令）

文件头怎么说：是（detector:29「`wait -n` 不算」）

- `sleep 1 & wait -n`（主(bg)）：该判**放**；真 bash：没有桩被起；bash 退出码 0；钩子退出码 heavy=0 detector=2

### Z13 bg 里 `tail --pid` / `kill -0` 轮询等它，文件头写了算等，实际仍按没等拒（2 个角色×写法，2 条命令）

文件头怎么说：相反（detector:29「或之后有 `tail --pid` / `while|until … kill -0` 轮询」）

- `sleep 2 & tail --pid=$! -f /dev/null`（主(bg)）：该判**放**；真 bash：没有桩被起；bash 退出码 0；钩子退出码 heavy=0 detector=2
- `sleep 1 & p=$!; while kill -0 $p 2>/dev/null; do sleep 0.2; done`（主(bg)）：该判**放**；真 bash：没有桩被起；bash 退出码 0；钩子退出码 heavy=0 detector=2

### Z14 wait 写在函数里、函数被调用，认不出（1 个角色×写法，1 条命令）

文件头怎么说：否

- `w() { wait; }; sleep 1 & w`（主(bg)）：该判**放**；真 bash：没有桩被起；bash 退出码 0；钩子退出码 heavy=0 detector=2

### Z15 cargo 的旗标 / 包名 / 子命令由变量或命令替换拼出（9 个角色×写法，6 条命令）

文件头怎么说：否（文件头只写命令词与脚本路径）

- `cargo $(echo test --workspace)`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- `P=singlefs-checker-tier; cargo test -p $P`（主无前缀）：该判**拒**；真 bash：cargo test -p singlefs-checker-tier（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- `F=--workspace; cargo test $F`（主无前缀）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- `P=singlefs-checker-tier; cargo test -p "$P"`（主无前缀）：该判**拒**；真 bash：cargo test -p singlefs-checker-tier（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- 同子类另有 2 条，见判错表

### Z16 bash -c / sh -c 的脚本里用位置参数 $0 $1 "$@" 拼出命令（8 个角色×写法，4 条命令）

文件头怎么说：否

- `bash -c 'cargo "$@"' _ test --workspace`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- `sh -c 'exec "$@"' sh cargo test --workspace`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- `bash -c '$0 test --workspace' cargo`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- `bash -c 'cargo "$1" "$2"' _ test --workspace`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0

### Z2 变量里拼出命令词 / 脚本路径（含 for 循环变量、命令替换算出路径）（17 个角色×写法，9 条命令）

文件头怎么说：是（heavy-test-guard.sh:195「变量里拼出来的命令与脚本路径」）

- `a=(cargo test --workspace); "${a[@]}"`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- `c='cargo test --workspace'; $c`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- `set -- cargo test --workspace; "$@"`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- `"$(command -v cargo)" test --workspace`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- 同子类另有 5 条，见判错表

### Z3 命令词不带斜杠、经 PATH 找到的脚本（2 个角色×写法，1 条命令）

文件头怎么说：是（heavy-test-guard.sh:199「命令词不带斜杠、从 PATH 里找到的脚本」）

- `PATH={U} s.sh`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0

### Z4 脚本用 dd / sed / head / printf / echo / install / python 写出再执行（22 个角色×写法，11 条命令）

文件头怎么说：是（heavy-test-guard.sh:176「认不出的写法（printf、echo、python 写文件」；dd/sed/head/install 没点名，同一原理）

- `sed 's/x/x/' {U}/s.sh > {U}/se.sh; bash {U}/se.sh`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- `echo 'cd {C} && cargo test --workspace' >> {U}/w.sh; bash {U}/w.sh`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- `python3 - <<'PY'↵open('{U}/w.sh','w').write('cd {C} && cargo test --workspace')↵PY↵bash {U}/w.sh`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- `head -n 9 {U}/s.sh > {U}/h.sh; bash {U}/h.sh`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- 同子类另有 7 条，见判错表

### Z5 没被调用的函数体按字面判（循环）（16 个角色×写法，8 条命令）

文件头怎么说：否

- `f() { while true; do sleep 1; done; }; echo ok`（主、子）：该判**放**；真 bash：没有桩被起；bash 退出码 0；钩子退出码 heavy=0 detector=2
- `false && cargo test --workspace`（主无前缀、子）：该判**放**；真 bash：没有桩被起；bash 退出码 1；钩子退出码 heavy=2 detector=0
- `until true; do cargo test --workspace; done`（主无前缀、子）：该判**放**；真 bash：没有桩被起；bash 退出码 0；钩子退出码 heavy=2 detector=0
- `f() { cargo test --workspace; }; echo done`（主无前缀、子）：该判**放**；真 bash：没有桩被起；bash 退出码 0；钩子退出码 heavy=2 detector=0
- 同子类另有 4 条，见判错表

### Z6 ⑬ claude：here-string / env -S 里写 claude 认不出（①与重型闸对同一写法认得）（2 个角色×写法，2 条命令）

文件头怎么说：否（detector ① 写了 here-string 字面照命令判；⑬ 那一条只写了 heredoc）

- `bash <<< 'claude -p hi'`（子）：该判**拒**；真 bash：claude -p hi（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- `env -S 'claude -p hi'`（子）：该判**拒**；真 bash：claude -p hi（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0

### Z7 ① 看门狗输出经 `| tee /dev/null` 丢掉（1 个角色×写法，1 条命令）

文件头怎么说：否

- `bash {R}/scripts/agents/watch.sh | tee /dev/null`（主(bg)）：该判**拒**；真 bash：scripts/agents/watch.sh （前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0

### Z8 ⑦ 没点名 rsync --inplace（同 inode 改脚本）（1 个角色×写法，1 条命令）

文件头怎么说：否

- `echo '# x' > {U}/n.sh; rsync --inplace {U}/n.sh scripts/hosts/capped.sh`（主）：该判**拒**；真 bash：没有桩被起；仓内改动 scripts/hosts/capped.sh；bash 退出码 0；钩子退出码 heavy=0 detector=0

### Z9 bash 的脚本路径参数里带通配，没按 bash 展开去读（2 个角色×写法，1 条命令）

文件头怎么说：是（heavy-test-guard.sh:195「通配与花括号写出来的脚本里的正文」）

- `bash {U}/*s.sh`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0


## 六、新形态（这一轮冒出来的，不展开）

- ⑮：`git stash`、`git rm -r --cached -q .`、`git checkout HEAD -- .` 同样清空整个索引，文件头只点名 `git reset` 与 `git restore --staged`；早期一趟里钩子放行（真 bash 里索引确实被清空）。这几条会改工作树或很慢，后来从语料里删了，只留这一次观测。
- ⑦：`rsync --inplace 源 已有脚本` 在同一个 inode 上改脚本（真 bash 里 inode 没换），⑦ 的清单（`>`、`>|`、`&>`、`tee`、`cp` 目标、`dd of=`、`truncate`、python `open`）没点名；钩子放行。
- ①：`watch.sh | tee /dev/null` 把看门狗的输出丢掉，效果同重定向到 `/dev/null`，钩子放行（Z7）。
- B2 的外延：**加了引号**的 `> 'o.{a,b}'` 在 bash 里就是一个字面文件名，只写一个文件，detector 对有 Edit 的子 agent 仍报"目标算不出最终落点"。
- ⑬：`bash <<< 'claude -p hi'`、`env -S 'claude -p hi'` 漏（Z6）；`xargs claude`、`ssh host claude`、`c=claude; $c` 漏。
- 前缀只落在管道前一段：`SINGLEFS_HEAVY_TESTS=commit echo extra | xargs cargo test --workspace`，前缀给了 `echo`，cargo 没带；钩子放行（跟着 A3）。
- `env -i` 清掉前缀：`SINGLEFS_HEAVY_TESTS=commit … env -i PATH=$PATH cargo test --workspace`，钩子按命令文字放行；按"真 cargo 进程环境里有没有前缀"算该拒，我没把它记成判错（见「七」第 3 条）。

## 七、我自己的样本/该判毛病（已改，不算钩子判错）

跑第一遍之后看钩子的 stderr，发现下面这些是我的装置或"该判"写错，钩子是对的，改完重跑（改的在 `tools/` 里，留着）：
1. `bash -lc` 登录 shell 重设 `PATH`，假 cargo 找不到，真 bash 里没起 cargo——给假家目录放了 profile 把桩目录放回 `PATH`。
2. `env -0 cargo …`（GNU env 的 `-0` 带命令是错）、`command -p cargo`（默认 `PATH` 里没有）、`run-with-memory-cap.sh 4G exec cargo …`（桩里的 `exec` 把包装自己替掉了，祖先链上看不到内存包装，装置没法认；真包装脚本是否这样我没读）：删了。前两条真 bash 没起 cargo（`env -0` 退 125、`command -p` 找不到），钩子拒它们没有害处，语料里删了。
3. `env -i` 清前缀：该判改成"日志里更早的任何一条桩记录带过前缀就算带了"，否则一律算漏放。
4. 化合命令前面直接写 `SINGLEFS_HEAVY_TESTS=commit`（`VAR=1 f() {…}`、`VAR=1 if …`、`VAR=1 ( … )`）是 bash 语法错，改成 `export SINGLEFS_HEAVY_TESTS=commit; …`。
5. 桩树在工作根里面，写范围判不准；`SINGLEFS_WORK_ROOT` 改指到草稿目录下的 `wr/`，仓副本放到 `wr/copy`。
6. 草稿里共用的 `n.sh`、`n3.py` 第二次被写时已经是"已有的脚本"，⑦ 正确地拒了——改成每条样本一个目录。
7. 钩子自己在桩树里写出 `__pycache__`，被我当成"仓内改动"——观测里排除。
8. 第一版 policy 的错：①`--list` 的 `cargo test` 仍起测试二进制，子 agent 要经内存包装；②轻型三条 `--quick` 子 agent 也要经内存包装（`heavy-test-guard.sh` 文件头"三条在 lib_heavy_tests.runs_compiled_code 里算跑编译出来的代码"）；③`checker-tier-crash-point-replay.sh` 不在轻型三条里，`--quick` 也算重型；④`-p 'singlefs-checker*'` 通配能匹配到 checker 档包；⑤`cd 副本/crates/singlefs-checker-tier && cargo test` 在副本里也是 checker 档；⑥`gate.sh` 在 `.claude/scripts/` 下。每一条都是钩子对我错，改了 policy 重跑受影响的样本。

## 八、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| X2 清单全不全 | 冲突 | 不全：清单外 63 条命令、16 个子类（Z1–Z16），见「五」 |
| A1 拷出写法 | 一致（清单里的） | 9 条复现；文件头 `heavy-test-guard.sh:176` 已写明认不出 |
| A2 `bash -s < 文件` | 一致 | 7 条复现；同一原理的 `bash < 文件`、`sh -s < 文件`、`cat 文件 \| bash` 同漏，应并入 |
| A3 xargs | 一致 | 13 条复现；`xargs bash 脚本`、`xargs claude` 同漏 |
| A4 名单外启动器 | 一致 | 15 条复现；漏的是 `find -exec/-execdir`、`ssh`、`doas`、`script -qec`、`unshare`、`setpriv`、`numactl` |
| B1 cat 不看源文件在不在 | 规则没说 | 我的三种写法没复现误拒；是否别处还有，复核不了 |
| B2 重定向花括号 | 一致，外延更大 | 3 条；`{1..2}` 与加引号的字面花括号也中 |
| B3 函数体之外引号里的花括号 | 规则没说 | 四种写法没复现 |
| C 函数当启动器 | 一致 | 6 条复现，含 `run_logged` |
| Z（清单外） | 冲突 | 63 条，其中文件头没写的 23 条、写了认实际不认的 2 条（Z13） |

**什么现象会推翻**：①清单里任一类修掉后用 `tools/rerun.sh` 重跑，该类样本钩子判对，则该类不再是判错；②Z 里任一子类，在真 bash 里查出那条命令其实没起我说的进程（样本毛病），则撤回；③把我标"文件头写了认实际不认"的 Z13 对着 `bash-command-detector.sh:29` 复核，若那一句只管 `run_in_background` 之外的写法，则 Z13 退回"文件头没写"。

## 九、复核不了的

- 真 `ssh` 到别的机器、真 `doas`、真 `systemd-run`/`tmux`/`screen` 的行为：用的是本机假的（`tools/bin/`），A4 里的 `ssh`/`doas` 只证明"本机再起一个 bash 时钩子看不见"。
- 上游 `pattern-process-guard.sh`、`run-scope-guard.sh` 不在这一轮五份钩子里，没喂。
- 机器负载 100–200 时超时给到 30 秒（等待循环 8 秒）；跑满超时才算"没停"，没有"恰好卡在边上"的样本被计入。
- `kill` 类只跑了点名单个 pid 的几种和 `kill 0`、`kill -- -$$`、命令替换/循环里的 kill（都只杀自己会话的进程）；`kill -1`、`$PPID`、按名字杀都不跑，⑥ 其余的没覆盖。

## 十、没做什么

- 不判 X1、X3；不替主 agent采纳或出判决。
- 不读 diff、前两轮的腿报告与反例、修补报告；不读钩子实现。
- 不起 cargo、不碰 GPU、不跑重型测试；假的 `cargo` 只是一个写日志的脚本。

## 十一、进程与草稿目录核对

- 交回前核：按 `/proc/*/environ` 里的 `ORACLE_MARK=` 扫，我起的真 bash 样本留下的活进程 0 个；`ps` 里还看得到的 `shell-words-r3-sonnet-model/tools/rerun.sh`（pid 2971738 一带）是另一条腿（正推腿甲）的，不是我的。过程中有一趟误把 `git rm -r --cached .` 之类大改动样本跑进去、拖慢了六个 worker，我对那一趟的 python 进程逐个点名 `kill <pid>` 停掉（没用按名字杀），随后清了仓状态。
- 草稿目录 `~/.local/share/singlefs/work/shell-words-r3-sonnet-s2/`：桩树 8 份（各约 19–29 MB）、`rerun-work`（109 MB）、`oraclelog`（25 MB）、仓副本与每条样本的草稿子目录，交回前都已删（删前合计 325 MB，删后 1.4 MB）；模型目录的 `tools/rerun.sh --part setup` 能再造出来。草稿目录里只剩 `bin/`、`detections.jsonl` 等小文件。
- 没碰主工作区：所有 `cargo`、`claude`、`ssh`、`tmux`、`systemd-run` 都是 `tools/bin/` 里写日志的假脚本；`kill` 类只杀了自己会话的进程。
