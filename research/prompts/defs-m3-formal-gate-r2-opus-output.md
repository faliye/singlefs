# defs-m3-formal-gate-r2 攻方腿（Opus）报告：E1、E2、E3

日期：2026-10-01。被判的 22 份文件读主仓现行版本；开工时与快照 `research/prompts/defs-m3-formal-gate-r2-snapshot/sha256sums.txt` 逐份 `sha256sum -c` 全 OK。
干到一半主仓里有三份被别的会话改了（`sha256sum -c` 报 FAILED 的是 `.claude/hooks/heavy-test-guard.sh`、`.claude/hooks/lib_heavy_tests.py`、`.claude/gate.d/stage-inputs.tsv`；改的是自证登记表与 54 号那一行，不在 E1–E3 的判法里）。
所以全部探针都跑在草稿拷贝上，那份拷贝与快照 22 份逐份 `sha256sum -c` 全 OK；报告里的行号是主仓现行版本的行号（引的那几行两边逐字相同）。

被判的命令一条都没执行：钩子只喂 JSON、看退出码；判法脚本只在草稿拷贝里当模块载入、配假 Verus 与桩包装调（E3），或经 `admission.py` 的判定子命令间接起它的 `tool-versions`（E2）。
另外直接起过 Verus 验一份三行的玩具文件 5 次（不是被判的 crate；`.claude/hooks/lib_heavy_tests.py` 文件头写着直接起的 verus 不算重型），每次经 `run-with-memory-cap.sh 500M` 与 `capped.sh 4`。

## 复跑

```
bash research/prompts/defs-m3-formal-gate-r2-opus-model/rerun.sh <仓根> <一个不存在的草稿目录>
```

它拷一份仓（不带 target、.git），另拷两份打草稿改法（G9 打钩子、G10 打 tool-versions），依次跑 `e1-hook-cases.py`（钩子 JSON，40 种写法 × 2 个身份）、`e1b-shared-shapes.py`（同一批写法喂广谱变异与 checker 档 cargo test 对照）、`e2-verus-toy.sh`（玩具 Verus 5 次）、`e2-reuse.sh`（复用判定，原样与 G10 各一趟）、`e2-lkmm-hook.py`、`e3-mixed.py`（函数级探针），最后逐份与入库的输出比（去掉 `/tmp` 路径）。不跑门禁、不编。git 写只落在拷贝自己的 `.git` 里。

交回前照 `rerun.sh` 从主仓复跑过一趟（草稿 `rerun1`，退 0，挂钟没记，主仓那时已是别的会话改过三份的版本）：`e1-output`、`e1b-output`、`e1-G9-output`、`e1b-G9-output`、`e2-lkmm-hook-output`、`e3-mixed-output` 去掉 `/tmp` 路径后与入库的相同；`e2-reuse-output`、`e2-reuse-G10-output` 只差指纹值（`stage-inputs.tsv` 被改过，指纹跟着变），去掉十六进制串与路径之后相同；`e2-verus-toy-output` 五行退出码与入库的相同（0、128、0、0、1）。

模型目录 `research/prompts/defs-m3-formal-gate-r2-opus-model/` 的 `SHA256SUMS`：

```
8299ff57f565a0a97abf4be0201ed010fbbdc91380ad44e3c54a49996052560a  e1-hook-cases.py
bc37bfa3840e8dafee9590a725c95d1ca980fa1f45029d77d01d91843cbbd64e  e1b-shared-shapes.py
472dbba620ab17bc09a94e05f442ac930732b675d0f7f33823f13609e8d4f144  fix-draft-e1.py
dcd6432c7fa464741f3285549441006e4357e189edfcbe71cd4e25836205a23b  fix-draft-e2.py
e09240f9bb16a0d70315242dcc5b5aade42386f04048f8868b51454c634879a1  e2-verus-toy.sh
17dde58f19dea891b36188556c029c62091085a47c5d469cff3fd9856ed51911  e2-reuse.sh
1309daeab5de9c4988ea215accbd84c9c2aa3a5f96c206dfa529222b3cbdc534  e2-lkmm-hook.py
6140eba1ab7dec44b55ed17682e8648a43935e0adba7096adbda65841c394568  e3-mixed.py
2b082b473ae02847f9e658d4f44dbd0fcd5ee194a1163bf375721f0169123ccb  rerun.sh
cab855635ae0d83ceaadeee03335678b6f21f19c207d48577dd94af8d20ee051  e1-output.txt
8991e3ac3cf8d6e919588749795e9006b141bf23aaac6de3e321f771602abc20  e1b-output.txt
16c1e475952ed96690fab169ddc79c84cb42fd46c55c4a3e0a1b85db182a7f48  e1-G9-output.txt
0dc8656f44b63993233798258fca850c2caeba07a82dfacfd928a73a620989eb  e1b-G9-output.txt
39c8d9bccba99cfa004a6234986ee192b15f117fcbcb36f2cc19c9b9335235e8  e2-verus-toy-output.txt
8225c40e4e976b441585873de8512c234b90b9ceaf684fcee670a674061cc596  e2-reuse-output.txt
71dea229915890e191261237a612b37506cb0948ef1198820e355aeb9b81aaaa  e2-reuse-G10-output.txt
740263afb0ca0d655cef2acbe79c14dc37335dc804946435b3736ace4b167328  e2-lkmm-hook-output.txt
d88a1e4cc57bc3326878cd2e7fa0d5c85dc82b8a056261d69816ba659e03dd2e  e3-mixed-output.txt
```

## 各格判定一览

| 编号 | 格 | 判定 | 量过 / 推的 | 一句话 |
|---|---|---|---|---|
| J1 | E1 | 打中 | 量过（钩子退出码） | 判法脚本的干活用法换六种写法闸全放：`PYTHONPATH=… python3 -m`、`python3 -c runpy`、仓外同名或改名的符号链接、子命令或模式放变量 / 命令替换 / shell 函数 / **for 循环逐格跑**。同样的写法起广谱变异驱动闸都拒，只漏这一道 |
| J2 | E1 | 打中，不分辨 | 量过 | `env -C`（`--chdir`）、脚本目录放变量、`script -qc`、`xargs` 也放，但广谱变异同样漏，病根在切词库，另一笔账 |
| J3 | E1 | 没打中 | 读代码 + 量过 | 只读用法（`toolchain`、`fingerprint`、`tool-versions`、`--check`、追加）一样都不起 Verus、不编装置；别的重阶段带 `--tool-versions` 闸放过，但那几道阶段的参数解析遇到认不出的选项先退 2 |
| J4 | E2 | 没打中 | 量过（草稿拷贝） | F1 之后这一道的指纹同一环境连算三次、在一棵 git worktree 里再算一次，四次逐字相同；没有一道因环境输出每次都变而永远复用不上（lkmm 那条只打 herd7 版本第一行，读代码，没跑 herd7） |
| J5 | E2 | 打中 | 量过（玩具 Verus + 草稿拷贝） | F2 的五样不够：只把安装里的 `libverus_builtin_macros.so` 换坏，Verus 判定从「验过」变成「编不过」，`tool-versions` 那一行一字不变，崩溃验证员与整轮两条问法都复用 |
| J6 | E2 | 打中 | 量过 | `VERUS_Z3_PATH` 没设与设成空串打的是同一行（都是「未设」），而 Verus 在空串下起不了 z3（退 128）；判绿之后改成空串，两条问法都复用 |
| J7 | E2 | 打中 | 量过 | `VERUS_Z3_PATH=z3`（裸名）时 Verus 照 PATH 找 z3（玩具跑里 PATH 最前的那份 z3 被起了 1 次），`tool-versions` 只打「z3:没有」；PATH 里换一份 z3，两条问法都复用 |
| J8 | E2 | 打中（轻） | 量过（钩子） | F1 之后 `stage-marker-check … checker-tier-lkmm.sh` 每次在里面现跑 `lkmm.sh --herd7-version`；同一条命令直接起，闸按 lkmm 重型拒（主 agent、崩溃验证员不带前缀都退 2），与这一道 `--tool-versions` 已放行不一致 |
| J9 | E3 | 没打中 | 读代码 + 量过 | 「没跑成」退 1，共用库把这一格记「红（退出 1）」，整道不写全绿标记、整轮不前移；没有一条路把它当通过或复用 |
| J10 | E3 | 打中 | 量过（判法输出）+ 推的（定义怎么记） | 同一格里一处包装退 252、另一处真红时，判法打两条 ✗；崩溃验证员第 3 步照字面「✗ 行写着『没跑成…』的，那一格…记『没跑成』，不记成红在证明、表或实现上」，真红跟着被记成没判出结果；门禁分诊一道只给一个归属，252 ⇒「并发」同样盖住 |
| J11 | E3 | 打中（轻） | 量过 | Verus 一侧把包装自己打的 ✗、→ 吞掉（捕获了 stderr 不打），门禁分诊要「原样抄 … 它上面包装自己打的 ✗、→」抄不到；252 有两种成因（排队超时、要的量比总上限大），出路都写「空出来再重跑」 |
| J12 | E3 | 没打中（分得开），附一处空格 | 读代码 | 253 在 Verus 一侧的 ✗ 行是「超过 N 秒没出结局，按红记（C5…）」，与「没跑成：内存包装…退 25x」字面分得开；但门禁分诊的 25x 规则只认后一种写法，Verus 的 253 落不进任何一条，空真格的超时行又点名了核的文件与行，会被按 diff 归属 |

## E1：判法脚本的干活与只读分界

### J1 换个写法，干活用法就被判成不重型（打中，只漏这一道）

被判的认法是「名字与位置」加「子命令在干活名单里」：

- `.claude/hooks/lib_heavy_tests.py:69`「research/scripts/e165-pin-red-examples.py 照它自己的参数切法落到 --replay 的（编 checker 档装置、逐行重放钉住的红例）；按名字与位置认（名字是它、又落在 research/scripts）。」
- `.claude/hooks/lib_heavy_tests.py:1161`「    if break_is_set("verus-judges-by-stage-only") or not located_in(word, directory, VERUS_JUDGE_DIRECTORY):」
- `.claude/hooks/lib_heavy_tests.py:1163`「    if name == VERUS_PROOF_CHECK_SCRIPT and arguments[:1] and arguments[0] in VERUS_PROOF_CHECK_WORKING_SUBCOMMANDS:」
- `.claude/hooks/lib_heavy_tests.py:1165`「    if name == E165_PIN_SCRIPT and e165_pin_mode(arguments) == "replay":」，模式的缺省是追加：`.claude/hooks/lib_heavy_tests.py:1147`「    mode, position = "append", 0」

也就是「认不出是干活的就放」。同一份文件给广谱变异驱动的认法反过来：名字与位置之外另按 PYTHONPATH 找（`.claude/hooks/lib_heavy_tests.py:1109`「    if os.path.basename(expanded) == BROAD_MUTATION_SCRIPT_NAME and any(located_in(expanded, base, BROAD_MUTATION_SCRIPT_DIRECTORY) for base in bases):」）、按真实路径认符号链接（`.claude/hooks/lib_heavy_tests.py:1117`「        real = os.path.realpath(resolved)」）、认 `python3 -c` 代码里的点名（`.claude/hooks/lib_heavy_tests.py:1227`「            if invocation.code_string is not None and BROAD_MUTATION_CODE_REFERENCE.search(invocation.code_string):」），参数只放过点名的只读几样。

喂钩子的原样退出码（实现员身份，主 agent 身份逐条相同；2 = 拒，0 = 放）：

- 对照：`research/prompts/defs-m3-formal-gate-r2-opus-model/e1-output.txt:1`「2	2	implementation-writer	C1 原样」（`cd` 进去再 `-m`、`-m research.scripts.…`、直接执行、`bash -c`、`timeout`、子 shell 里 `cd`、`capped.sh`、`/usr/bin/time`、heredoc、`run-with-memory-cap.sh auto`、`python3 -I` 这 12 种对照全拒）
- `research/prompts/defs-m3-formal-gate-r2-opus-model/e1-output.txt:31`「0	2	implementation-writer	B4 PYTHONPATH= 前缀再 -m  <- 与期望不同」
- `research/prompts/defs-m3-formal-gate-r2-opus-model/e1-output.txt:35`「0	2	implementation-writer	B6 python3 -c runpy.run_path  <- 与期望不同」
- `research/prompts/defs-m3-formal-gate-r2-opus-model/e1-output.txt:41`「0	2	implementation-writer	B9 子命令放变量  <- 与期望不同」
- `research/prompts/defs-m3-formal-gate-r2-opus-model/e1-output.txt:45`「0	2	implementation-writer	B11 shell 函数转参数  <- 与期望不同」
- `research/prompts/defs-m3-formal-gate-r2-opus-model/e1-output.txt:47`「0	2	implementation-writer	B12 红例重放的模式放变量  <- 与期望不同」
- `research/prompts/defs-m3-formal-gate-r2-opus-model/e1-output.txt:65`「0	2	implementation-writer	B21 for 循环逐格跑三格  <- 与期望不同」
- `research/prompts/defs-m3-formal-gate-r2-opus-model/e1-output.txt:67`「0	2	implementation-writer	B22 for 循环先核钉全再重放  <- 与期望不同」

同一批写法喂广谱变异驱动（`e1b-output.txt`，符号链接先建好在草稿目录里，因为广谱变异那一份读盘认）：PYTHONPATH 加 `-m`、`-c runpy`、仓外同名与改名的符号链接、参数放变量这五种，广谱变异全拒、这一道全放，例：`research/prompts/defs-m3-formal-gate-r2-opus-model/e1b-output.txt:12`「2	广谱变异	S4 仓外同名符号链接（先建好）」对 `research/prompts/defs-m3-formal-gate-r2-opus-model/e1b-output.txt:11`「0	Verus 判法	S4 仓外同名符号链接（先建好）」；`research/prompts/defs-m3-formal-gate-r2-opus-model/e1b-output.txt:21`「2	广谱变异	S8 参数放变量」对 `research/prompts/defs-m3-formal-gate-r2-opus-model/e1b-output.txt:20`「0	Verus 判法	S8 参数放变量」。

这些写法真的会跑起那份脚本：草稿目录里拿一份同样带连字符、同目录导入辅助模块的假脚本试过五种（`PYTHONPATH=… python3 -m`、`env -C`、仓外同名符号链接、改名符号链接、`python3 -c runpy.run_path`），五种都以 `__main__` 跑起、导入成功（只在草稿里跑，没入库，原样没存；符号链接下 `sys.path[0]` 落在真实目录，`e165-pin-red-examples.py` 的同目录导入照样成）。

一段具体的历史：实现员在写证明，派发提示有「重型测试：不跑」；它改完想一次看三格，写 `for c in proof vacuity must-red-worlds; do python3 research/scripts/verus-proof-check.py $c .; done`。闸放行，三格逐个起 Verus（必红世界逐行起），正是 3-11 这条认法要拦的那一件。for 循环是这几种里最顺手、最不像「绕」的写法。

四句：
1. 分不分辨臂：分辨。这五种写法让这一道的认法与广谱变异的认法给出相反判定（`e1b-output.txt` 第 7–14、20–21 行），病根是这一轮新写的 `verus_judge_use`（白名单式「干活子命令才算」、只按名字与位置认），不是共用的切词库。
2. 系统看不看得到：看得到。PYTHONPATH 在命令文本里（广谱变异那一份已经读它）；符号链接在盘上（广谱变异读 `realpath`）；`-c` 的代码串在手里；变量与 for 循环的值它不必展开——缺省判重型、只放过字面写明的只读子命令，就不用看见值（`cargo -p $P` 与广谱变异都是这样判红的，`e1b-output.txt` 第 21、22 行）。
3. 满足的分句：正文 E1 一格「一种干活用法换个写法（参数次序、`python3 -m`、符号链接、经别的包装）就被判成只读」，字面命中其中的 `python3 -m`、符号链接两项，另加变量、函数、for 循环三种。
4. 改法：3-11 报告自己写了符号链接、改名拷贝与 `python3 -c` 罩不住、「判决没要求，没做」；它没提变量与 for 循环。G9（草稿，见改法表）把这一道的认法改成广谱变异那一种：认出是这两份脚本就算重型，只有字面写明的 `toolchain`、`fingerprint`、`tool-versions` 与 e165 的追加、`--check` 放过；认法加 PYTHONPATH、真实路径与 `-c` 代码点名。量过：G9 下 B3–B6、B9–B12、B21、B22 全变成拒（`e1-G9-output.txt` 里与期望不同的只剩 16 条，见 J2），只读的 R1–R4 仍放；`e1b-G9-output.txt` 第 7–14、20 行这一道变成 2。**G9 只在我的模型上量过、被攻过零轮。**

### J2 `env -C`、目录放变量、`script -qc`、`xargs`（打中，不分辨）

- `research/prompts/defs-m3-formal-gate-r2-opus-model/e1-output.txt:25`「0	2	implementation-writer	B1 env -C 换目录  <- 与期望不同」：切词库把 `-C` 的值当选项值跳过、不跟着换目录：`.claude/hooks/lib_shell_words.py:48`「    "timeout": {"-s", "-k", "--signal", "--kill-after"}, "env": {"-u", "--unset", "-C", "--chdir"},」。
- `research/prompts/defs-m3-formal-gate-r2-opus-model/e1-output.txt:49`「0	2	implementation-writer	B13 脚本目录放变量  <- 与期望不同」；`research/prompts/defs-m3-formal-gate-r2-opus-model/e1-output.txt:59`「0	2	implementation-writer	B18 script -qc  <- 与期望不同」；`research/prompts/defs-m3-formal-gate-r2-opus-model/e1-output.txt:51`「0	2	implementation-writer	B14 xargs 喂子命令（文件头列过的老口子）  <- 与期望不同」。

四句：不分辨——广谱变异同样漏（`research/prompts/defs-m3-formal-gate-r2-opus-model/e1b-output.txt:5`「0	广谱变异	S1 env -C 换目录」、`research/prompts/defs-m3-formal-gate-r2-opus-model/e1b-output.txt:18`「0	广谱变异	S7 script -qc」），`script -qc` 连 checker 档 `cargo test` 也漏（第 19 行），病根在 `lib_shell_words.py` 的前缀与包装表，另立一笔账；`xargs` 已在文件头老口子清单里。系统看得到（`-C` 的值、`script -c` 的代码串都在命令文本里）。分句同 J1「经别的包装」。改法推的：`env -C` / `--chdir` 跟着换目录，`script -c` 当 `bash -c` 递归切。G9 碰不到这几格（`e1-G9-output.txt` 剩下的 16 条就是 B1、B2、B7、B8、B13、B14、B18、B20 各两个身份；B7、B8 剩下是因为链接在同一条命令里才建、钩子判的时候盘上还没有，先建好的 S4、S5 在 G9 下已拒）。

### J3 只读用法与 `--tool-versions`（没打中）

- 只读子命令一样都不起 Verus、不编：`toolchain` 只读 `version.json`、`verus.zip`、`Cargo.lock` 与 cargo 缓存（`check_toolchain` 全文）；`tool-versions` 只哈希文件；`fingerprint` 只走一遍包目录；e165 的 `--check` 与追加只读写点名的表。子命令认不出的照用法错退 2：`research/scripts/verus-proof-check.py` 的 `main` 先判 `arguments[0]`，选项只认六个带值的，别的 `--` 起头一律退 2，参数次序没有别的写法。
- `--tool-versions` 进了闸的查询参数表，对每一道阶段都算：`.claude/hooks/lib_heavy_tests.py:184`「STAGE_QUERY_ARGUMENTS = frozenset({"--list", "--list-items", "--tool-versions"})」，所以 `research/prompts/defs-m3-formal-gate-r2-opus-model/e1-output.txt:77`「0	-	implementation-writer	R5 54 号带 --tool-versions」。读五道重阶段的参数解析：54 号、QEMU、lkmm、crates 变异整表、research 构建与复跑遇到认不出的 `-*` 都先退 2（各自 `while (($#))` 里的 `-*)` 分支，在起任何重活之前），只有这一道认 `--tool-versions` 并 `exec` 到 `tool-versions`。没跑那几道，读代码判的。推翻条件：哪一道往后在参数解析之前起了重活，或者把 `--tool-versions` 认成别的意思。

## E2：F1（环境进全绿标记的指纹）与 F2（tool-versions 多哈希四样与 VERUS_Z3_PATH）

装置（`e2-reuse.sh`）：在与快照逐份相同的草稿拷贝里 `git init`，只调 `admission.py` 的 `stage-fingerprint`、`stage-marker-write`、`stage-marker-check`、`gate-reuse` 与它经 environment= 起的阶段 `--tool-versions`。固定的前缀：崩溃验证员在工作区判绿、写这批输入的全绿标记（环境 W），之后输入一字不改；放开扫的是之后换成的环境 Q 与两条问法（q1 崩溃验证员第 1 步 `stage-marker-check`，退 0 = 复用；q3 `gate.sh --staged` 里阶段问 `gate-reuse`，退 10 = 复用）。Verus 的判定会不会因 W → Q 变，另由 `e2-verus-toy.sh` 直接起 Verus 验玩具文件量。

### J4 环境输出稳不稳（没打中）

`research/prompts/defs-m3-formal-gate-r2-opus-model/e2-reuse-output.txt:2`「  第 1 次：ae49cf42bacbcb13e24198cea1938862d36baff6062190add96e6d704bfadb6d 566」，第 3、4 行两次相同，`research/prompts/defs-m3-formal-gate-r2-opus-model/e2-reuse-output.txt:5`「  worktree：ae49cf42bacbcb13e24198cea1938862d36baff6062190add96e6d704bfadb6d 566」。环境摘要取整份 stdout（`research/scripts/admission.py:1010`「        digest.update(token.encode("utf-8", "surrogateescape") + b"\n" + output + b"\0")」），而这一道的 `tool-versions` 那一行里没有时刻、进程号与根路径（安装目录只在「没装」那一句里出现，那条路径由 `SINGLEFS_VERUS_ROOT` 或 `XDG_DATA_HOME` 定，不随趟变）。lkmm 那一条只打 herd7 版本输出的第一行：`.claude/scripts/lkmm.sh:78`「  printf '%s\n' "${herd7_version_output%%$'\n'*}"」——读代码判的，没跑 herd7（按名字归重型）。上游 `preflight.sh` 一行都不往 stdout 打（`grep` 过，没有不带 `>&2` 的 `echo` / `printf`）。
推翻条件：herd7 的 `-version` 第一行带构建时刻以外的随趟变量，或别的会话往这两道的 environment= 命令里加了打到 stdout 的东西。

### J5 F2 的五样不够：Verus 还读安装里别的文件（打中）

F2 哈希的是 `research/scripts/verus-proof-check.py:113`「TOOL_VERSION_FILES = ("rust_verify", "z3", "vstd.vir", "libvstd.rlib")」加 `verus`。装的 Verus 目录顶层有 54 个普通文件（`find -maxdepth 1 -type f | wc -l`）；两份二进制里点名、这五样之外的有 `libverus_builtin.rlib`、`verus_builtin_macros`、`verus_state_machines_macros`（`strings` 读的，`rust_verify` 里有一串「verus_builtin_macrosverus_state_machines_macroslibverus_builtin」），`rust_verify` 还动态链接 rustup 工具链里的 `librustc_driver-eaf3ed1d23ca5027.so`（`readelf -d` 的 NEEDED），环境变量另有 `VERUS_ROOT`、`VERUS_CVC5_PATH`、`VERUS_SINGULAR_PATH`、`RUSTUP_TOOLCHAIN`、`RUST_MIN_STACK`。这几样里我只量了一样：

- 把安装目录顶层文件拷成 A，再拷一份 B、只把 `libverus_builtin_macros.so` 截成前 4096 字节。玩具文件在 A 上验过、在 B 上编不过：`research/prompts/defs-m3-formal-gate-r2-opus-model/e2-verus-toy-output.txt:4`「V4-拷贝A	退 0	stdout：{…」，`research/prompts/defs-m3-formal-gate-r2-opus-model/e2-verus-toy-output.txt:5`「V5-拷贝B-宏库截断	退 1	stdout：{  "func-details": {},  "verification-results": {    "encountered-error": …」（stderr 是 `error[E0463]: can't find crate for vstd`）。
- 两份打的那一行摘要相同：`research/prompts/defs-m3-formal-gate-r2-opus-model/e2-reuse-output.txt:11`「  A（安装拷贝 A）：8375c67c95977449 verus version=…VERUS_Z3_PATH=未设」，`research/prompts/defs-m3-formal-gate-r2-opus-model/e2-reuse-output.txt:12`「  B（安装拷贝 B（宏库截断））：8375c67c95977449 verus version=…VERUS_Z3_PATH=未设」。
- 在 A 上判绿写标记，换到 B 再问：`research/prompts/defs-m3-formal-gate-r2-opus-model/e2-reuse-output.txt:18`「Y2 安装拷贝 A → 拷贝 B（宏库截断，Verus 编不过）	q1 stage-marker-check 退 0（0 = 复用）	q3 gate.sh --staged 里 gate-reuse 退 10（10 = 复用）」。对照：只改 `version.json` 的 commit，两条都不复用（`research/prompts/defs-m3-formal-gate-r2-opus-model/e2-reuse-output.txt:16`「对照 版本变（拷贝 A → 拷贝 C：version.json 的 commit 改）	q1 stage-marker-check 退 1（0 = 复用）	q3 gate.sh --staged 里 gate-reuse 退 0（10 = 复用）」），比对本身起作用。

截断是为了在玩具上看得见判定变了；真实的形态是安装目录里重编或换了宏库（与第一轮 B3「只换 z3」同一类：版本号与外壳不变、判定依赖的另一个文件变了），复用照旧作数，而真跑会是另一个结论。

### J6 `VERUS_Z3_PATH` 没设与设成空串是同一个指纹，Verus 却不同（打中）

`research/scripts/verus-proof-check.py:752`「        z3_override = os.environ.get("VERUS_Z3_PATH", "")」与 `research/scripts/verus-proof-check.py:754`「        extra += f" VERUS_Z3_PATH={z3_override + ':' + file_sha256(z3_override) if z3_override else '未设'}"」把空串并进「未设」：`research/prompts/defs-m3-formal-gate-r2-opus-model/e2-reuse-output.txt:8`「  E（VERUS_Z3_PATH 空串）：8375c67c95977449 verus version=…VERUS_Z3_PATH=未设」，与第 7 行没设的那一行同一个摘要。Verus 不这样看：`research/prompts/defs-m3-formal-gate-r2-opus-model/e2-verus-toy-output.txt:2`「V2-空串	退 128	stdout：	stderr 末行：help: z3 needs to be in the PATH, or the environment variable VERUS_Z…」（上一行 stderr 是 `could not execute z3 process (No such file or directory)`）。判绿之后环境里多一个空的 `VERUS_Z3_PATH`：`research/prompts/defs-m3-formal-gate-r2-opus-model/e2-reuse-output.txt:17`「Y1 未设 → 空串（Verus 起不了 z3）	q1 stage-marker-check 退 0（0 = 复用）	q3 gate.sh --staged 里 gate-reuse 退 10（10 = 复用）」。这正是正文 E2 问的那一句「`VERUS_Z3_PATH` 没设与设成空串是不是同一个指纹」：是同一个，而判定不同。

### J7 `VERUS_Z3_PATH` 写裸名时 Verus 照 PATH 找，指纹看不见（打中）

`file_sha256(z3_override)` 按 `tool-versions` 那个进程的当前目录（阶段先 `cd "$ROOT"`）找这个名字，找不到就写「没有」；Verus 用的是 PATH 里的那一份。玩具跑里 PATH 最前放一个记日志再转给真 z3 的 z3：`research/prompts/defs-m3-formal-gate-r2-opus-model/e2-verus-toy-output.txt:3`「V3-裸名z3	退 0	stdout：{…	PATH 里的 z3 被起了 1 次」。两种 PATH 打的那一行相同：`research/prompts/defs-m3-formal-gate-r2-opus-model/e2-reuse-output.txt:9`「  P1（VERUS_Z3_PATH=z3、PATH 最前是记日志的 z3）：75a4b9f39795f21f verus version=…VERUS_Z3_PATH=z3:没有」与第 10 行 P2 同一个摘要。复用：`research/prompts/defs-m3-formal-gate-r2-opus-model/e2-reuse-output.txt:19`「Y3 裸名 z3：PATH 里换一份 z3	q1 stage-marker-check 退 0（0 = 复用）	q3 gate.sh --staged 里 gate-reuse 退 10（10 = 复用）」。相对路径（含 `/`）同病更重：`tool-versions` 在仓根解、Verus 在临时副本里解（`research/scripts/verus-proof-check.py:311`「        completed = subprocess.run(command, cwd=crate_copy, capture_output=True, text=True, env=environment, timeout=subprocess_timeout)」），两边指的不是同一个文件（推的，没跑）。

J5–J7 的四句（同一组）：
1. 分不分辨臂：分辨 F2 的写法与「哈希整个安装、照 Verus 的找法解 z3」那种写法（G10，量过）；不碰 F1——F1 只是把这一行的摘要搬进标记，行里没有的东西，搬进去照样看不见。旧的环境记录那条路（标记不在、走 `judge_gate_environment`）用同一个 `current_gate_environment`，同样看不见（推的，没跑 H3、H4 那两段历史）。
2. 系统看不看得到：看得到。安装目录在手里（`tool-versions` 已经在读它），`VERUS_Z3_PATH` 在不在 `os.environ` 里分得出，PATH 查找 `shutil.which` 一步就有。
3. 满足的分句：正文 E2「F2 哈希的五样够不够（Verus 还读别的东西吗）」与「`VERUS_Z3_PATH` 没设与设成空串是不是同一个指纹」两个分句。
4. 改法：G10（草稿）：另打安装目录顶层全部普通文件的整体摘要，`VERUS_Z3_PATH` 分「未设」「空串」，值里没有 `/` 的照 PATH 找到再哈希。量过：`research/prompts/defs-m3-formal-gate-r2-opus-model/e2-reuse-G10-output.txt:17`「Y1 未设 → 空串（Verus 起不了 z3）	q1 stage-marker-check 退 1（0 = 复用）	q3 gate.sh --staged 里 gate-reuse 退 0（10 = 复用）」，第 18、19 行 Y2、Y3 同样两条都不复用；对照第 14、15 行仍复用、第 16 行仍不复用；三次加 worktree 的指纹仍相同（第 2–5 行）。代价：每次算指纹多哈希约 253 MiB（安装顶层，`du -sh` 量的），挂钟没量。G10 罩不住的：rustup 工具链里的 `librustc_driver`、`VERUS_ROOT` 等环境变量、z3 是个转发脚本时它转去的那一份（只哈希脚本本身）。**G10 只在我的模型上量过、被攻过零轮。**

### J8 F1 让 lkmm 那一道的标记查询在里面起 `lkmm.sh`（打中，轻）

F1 之后 `stage_fingerprint` 对登记了 environment= 的阶段现跑环境（`research/scripts/admission.py:2224`「        _environment_lines, environment_digest = current_gate_environment(root, stage_rows)」），lkmm 那一条是 `lkmm.sh --herd7-version`，经 `bash <根>/<脚本>` 起（`research/scripts/admission.py:958`「    command = ["bash", os.path.join(root, parts["script"]), *parts["arguments"]]」）。闸对 `lkmm.sh` 不看参数一律算重型：`.claude/hooks/lib_heavy_tests.py:1197`「    if name == "lkmm.sh":」。喂钩子：`research/prompts/defs-m3-formal-gate-r2-opus-model/e2-lkmm-hook-output.txt:1`「2	主 agent	主 agent 直接打 herd7 版本（lkmm 的 environment= 那一条）」，而 `research/prompts/defs-m3-formal-gate-r2-opus-model/e2-lkmm-hook-output.txt:3`「0	crash-verifier	崩溃验证员第 1 步问 lkmm 的标记（F1 之后它在里面现跑上一条）」。按共用约束 `.claude/agent-common.md:53`「写进脚本文件、放进 `python3` 的 subprocess、`make` 这类间接起法同样算跑」的字面，崩溃验证员第 1 步不带前缀就起了一次被归成重型的命令；实际只是打版本，没有害处。与第一轮 A4 同形，A4 只给这一道的 `--tool-versions` 开了口（第 4 行放），lkmm 的版本查询没跟。四句：只管 lkmm；看得到（参数就在命令里）；分句是正文 E2「F1 动了每一道登记 `environment=` 的阶段」；改法推的：`lkmm.sh --herd7-version` 照 `--tool-versions` 算查询。

## E3：包装 250–254 判「没跑成」、253 判红

装置（`e3-mixed.py`）：草稿拷贝里把 `verus-proof-check.py` 当模块载入，Verus 换成它自证用的 `FAKE_VERUS`，`MEMORY_CAP_RUNNER` 换成一个桩：副本源码里有 `STUB_252` 就往 stderr 打一行「✗ 桩：内存不够排不上」、退 252、不跑；没有就照原样起那条命令。不起真 Verus、不编、不跑门禁（与 3-11 的 D3 探针同一种做法）。

### J9 「没跑成」会不会被当通过或被复用（没打中）

判法里「没跑成」照样退 1，格的共用库把非 0、非 77 记红：`.claude/gate.d/lib/stage-cells.sh:250`「      red+=("$name"); verdict_of[$name]="红（退出 $cell_exit）"」；整道非 0 就不记环境、不写全绿标记（阶段第 144 行起那一段只在 `stage_exit == 0` 时走），整轮红则 `gate-staged.sh` 不前移 `refs/sop/staged-green`。崩溃验证员下一趟第 1 步的 `stage-marker-check` 找不到这批输入的标记，照跑。量过的只有判法一侧（M1–M3 三格都退 1）；整轮与标记一侧是读代码判的。
推翻条件：哪一条路把这一道的退 1 改成 77，或判绿写标记的那一段不再看 `stage_exit`。

### J10 同一格里「没跑成」与真红混在一起，定义按格记会盖掉真红（打中）

判法把一格里的问题逐条打，没跑成的那一处与真红的那一处各一条 ✗：

- `research/prompts/defs-m3-formal-gate-r2-opus-model/e3-mixed-output.txt:2`「  ✗ 第 2 行「改坏了也验得过」改坏之后 Verus 照样验过（verified 2、errors 0）：证明没盯住这一处」与 `research/prompts/defs-m3-formal-gate-r2-opus-model/e3-mixed-output.txt:4`「  ✗ 第 3 行「这一行排不上」：没跑成：内存包装 research/scripts/run-with-memory-cap.sh 退 252（内存不够排不上，上限 1G），这一次的输出不算判定」同在必红世界一格（M1）。
- `research/prompts/defs-m3-formal-gate-r2-opus-model/e3-mixed-output.txt:7`「  ✗ 核里有不许的标记 crates/core/src/kernel.rs:10：assume」与 `research/prompts/defs-m3-formal-gate-r2-opus-model/e3-mixed-output.txt:9`「  ✗ 没跑成：内存包装 research/scripts/run-with-memory-cap.sh 退 252（内存不够排不上，上限 1G），这一次的输出不算判定」同在证明一格（M2）：作弊标记是开跑前静态判的，与 Verus 跑没跑成无关，是确定的红。

定义按格（或按阶段）记：`.claude/agents/crash-verifier.md:32`「「✗」行写着「没跑成：内存包装 research/scripts/run-with-memory-cap.sh 退 <码>」的，那一格这一趟没判出结果：表里那一格记「没跑成」，不记成红在证明、表或实现上」；它的产出表一道一行（第 43 行「阶段 / 退出码（0、非 0、77）/ 耗时 / 原样的 ✓，或 ✗ 与 →」）。门禁分诊同样一道一个归属：`.claude/agents/gate-triage.md:38`「- 一张表：阶段 / 绿·红·77 / 归属（这一轮 · 不是这一轮 · 环境 · 并发 · 分不清）/ 门禁原样的 ✗ 与 → ；然后未实现清单原样。」，25x 那一条又说「不按它点名的文件判：…252 与 254 ⇒「并发」」。

一段具体的历史：主 agent 暂存了一处证明改动（必红世界表里某一行改坏后不再红），派崩溃验证员；同时别的会话占着 slice。这一道必红世界 94 行里几行排满 3600 秒退 252、其余照跑，其中那一行真没红。阶段输出两类 ✗；崩溃验证员照字面把必红世界那一格（或整道）记「没跑成」、「不记成红在证明」，主 agent 读到的是「环境没跑成，空出来再跑」，不是「证明盯不住了」。之后整轮会再跑一次，真红会在那时出来——所以后果是误导与推迟，不是放过；但若分诊也照 252 记「并发」，主 agent 手里两份报告都没有「这一轮的红」。

四句：
1. 分不分辨臂：分辨「按格 / 按道记没跑成」与「按每一条 ✗ 记」两种记法；病根是这一轮新写的定义句，不是共用前提。
2. 系统看不看得到：看得到，两类 ✗ 各占一行、字面分得开（M1、M2 的输出）。
3. 满足的分句：正文 E3「『没跑成』与『红』在 crash-verifier、gate-triage 与 `gate.sh` 汇总里各怎么记，会不会被当成通过」——不是被当通过，是真红被记成没跑成。定义的字面「那一格…记『没跑成』」是格粒度，判法的输出是行粒度，两者对不上。
4. 改法（推的，没量）：G11 定义改成「一格里每一条 ✗ 都写着『没跑成』，那一格才记没跑成；还有别的 ✗ 的照红记，两类原样都抄」，门禁分诊同改；或判法一侧在有别的问题时不再让「没跑成」那一条单独定这一格的出路（比如末尾另打一行「这一格另有 N 条真红」）。只改判法、不改定义，碰不到崩溃验证员按格记的那一句。

### J11 Verus 一侧吞掉包装自己的 ✗、→；252 的两种成因同一个出路（打中，轻）

- 包装往 stderr 打它自己的 ✗、→，Verus 一侧捕获了 stderr（`research/scripts/verus-proof-check.py:311` 那一行 `capture_output=True`）却不打出来：M2 里桩打的「✗ 桩：内存不够排不上」在整格输出里没有（`e3-mixed-output.txt` 第 6–10 行全文）。门禁分诊第 3 步要「原样抄阶段的 → 与它上面包装自己打的 ✗、→」（`.claude/agents/gate-triage.md:29`「…250 ⇒「环境（上限）」，252 与 254 ⇒「并发」，251 与 253 ⇒「环境」，原样抄阶段的 → 与它上面包装自己打的 ✗、→…」），这一道的 Verus 那三格抄不到。红例重放那一侧把 stderr 末 4 行截 300 字塞进 ✗ 行（`research/scripts/e165-pin-red-examples.py:399`「        tail = " | ".join(line for line in completed.stderr.strip().split("\n")[-4:] if line.strip())」），两侧不一致。
- 252 有两种成因：`research/scripts/run-with-memory-cap.sh:27`「#   252    内存不够排不上：等满 RUN_WITH_MEMORY_CAP_WAIT_SECONDS 秒还放不下，或这一条要的量比总上限还大（等多久都放不下，不等）。命令一行都没跑；」。后一种重跑多少次都一样，判法给的出路却只有一种：`research/scripts/verus-proof-check.py:111`「                 "251、252、254 先跑 bash research/scripts/run-with-memory-cap.sh --status 看 slice 被谁占着，空出来再重跑这一格；不拿掉包装跑，不改证明、核或必红世界表")」，分诊照 252 记「并发」。能不能走到后一种（推的）：要的量取峰值表里的峰值，表里没有就取上限（`research/scripts/run-with-memory-cap.sh:46`「#   这一条要的量：峰值表里这条命令（键见下）上一次实测的峰值，不超过它的上限；表里没有的按它的上限算——scope 的 MemoryMax 就是它最多能占的量，按它算不会少算。」）；装置那条的键带根路径（第一轮 C4），`--staged` 的 worktree 每趟一个新键，于是 `GATE_E165_PINNED_MEMORY_MAX` 给得比 slice 总上限（本机 40.1 GiB，这一次 `--status` 读的）大时，每趟当场 252。Verus 那三条的键是固定的、表里有峰值，走不到。
- 四句：分辨两侧判法（红例那一侧带了 stderr 末几行，Verus 一侧没带）；系统看得到（stderr 就在 `outcome["diagnostics"]` 里）；分句是正文 E3「交给分诊时分得开吗」；改法推的：Verus 一侧照红例那一侧把包装 stderr 的末几行并进「没跑成」那一条。

### J12 253 与 25x 分得开吗（没打中，附一处空格）

分得开：Verus 一侧 253 打 `research/scripts/verus-proof-check.py:366`「        return f"超过 {options['timeout']} 秒没出结局，按红记（C5：超时不当通过）"」，`WRAPPER_OWN_EXITS` 里没有 253（`research/scripts/verus-proof-check.py:107`「WRAPPER_OWN_EXITS = {250: "撞了这一条自己的内存上限", 251: "包装用法错或起不了 scope", 252: "内存不够排不上", 254: "被总上限挤掉"}」），字面与「没跑成：内存包装…」不同。
空格（推的，读定义）：门禁分诊的 25x 规则只认「…退 <码>」那一种写法，Verus 的超时行不含它，就落回「看它点名的文件与行在不在暂存区 diff 的块里」。证明格的超时行不点名文件，归不了（只能「分不清」）；空真格的超时行带着「<核文件> fn <名>（第 N 行的 requires）」，会按那一行在不在 diff 里归「这一轮」或「不是这一轮」——超时与那一行有没有改无关。红例那一侧的 253 判「没跑成」，但包装在传给命令前清掉限时变量、`e165-pin-red-examples.py` 自己不设，这一支今天走不到（读代码）。推翻条件：分诊定义里另有一句接住「超过 N 秒没出结局」。

## 改法表（全部只在我的模型上量过或只是推的，被攻过零轮）

| 改法 | 改哪 | 修哪几格 | 状态 |
|---|---|---|---|
| G9 | `lib_heavy_tests.py` 的 `verus_judge_use`：认出是两份判法脚本就算重型，只有字面写明的 `toolchain`、`fingerprint`、`tool-versions` 与 e165 的追加、`--check` 放过（参数里有 `$` 或反引号的一律算）；认法加 PYTHONPATH、真实路径、`python3 -c` 代码点名（`fix-draft-e1.py`） | J1 | 量过（草稿拷贝）：`e1-G9-output.txt` 与期望不同的由 36 条降到 16 条，降掉的就是 J1 那十种写法；只读的 R1–R4 仍放；`e1b-G9-output.txt` 第 7、9、11、13、20 行变成 2。对 J2 不起作用（量过） |
| — | `lib_shell_words.py`：`env -C` / `--chdir` 跟着换目录，`script -c` 当 `bash -c` 递归切 | J2 | 推的；各类重型共用 |
| G10 | `verus-proof-check.py` 的 `tool_versions`：另打安装目录顶层全部普通文件的整体摘要，`VERUS_Z3_PATH` 分未设与空串，裸名照 PATH 找到再哈希（`fix-draft-e2.py`） | J5、J6、J7 | 量过（草稿拷贝）：`e2-reuse-G10-output.txt` 第 17–19 行 Y1–Y3 两条问法都不复用，第 14–16 行对照不变，第 2–5 行四次指纹相同。罩不住 rustup 工具链里的库与 `VERUS_ROOT` 等环境变量（推的） |
| — | `lib_heavy_tests.py`：`lkmm.sh --herd7-version` 照 `--tool-versions` 算查询 | J8 | 推的 |
| G11 | `crash-verifier.md` 第 3 步、`gate-triage.md` 第 3 步：一格（一道）里每一条 ✗ 都是「没跑成」才记没跑成，另有别的 ✗ 照红记、两类都抄 | J10 | 推的（文字改法）；只改判法碰不到按格记的那一句 |
| — | `verus-proof-check.py` 的 `outcome_problem`：「没跑成」那一条带包装 stderr 末几行 | J11 | 推的 |
| — | `gate-triage.md` 第 3 步：接住 Verus 的「超过 N 秒没出结局」，不按点名的核行归属 | J12 的空格 | 推的 |

## 没打中的形状与取样范围

- E1：钩子 JSON 40 种写法 × 2 个身份（实现员、主 agent），另 9 种写法喂这一道与广谱变异、其中 4 种再喂 checker 档 `cargo test` 对照。没打中的：12 种对照写法（全拒，见 J1）、`strace -f`、`flock`、`systemd-run --scope`、`chrt`（这四种也拒）；只读用法不起 Verus、不编（J3）。没试：`uv run` / `pypy` / `eval` / `find -exec`（文件头已列为老口子）、`detached-run.sh start --`（子 agent 照拒）、别的身份（崩溃验证员带前缀、门禁分诊，第一轮 A5 与 3-11 的样本都跑过）。
- E2：环境六种（未设、空串、裸名两种 PATH、安装拷贝 A、宏库截断的 B）、对照三种（不换、A 换回原安装、只改 commit）、问法两条（q1、q3）。没扫：标记不在、走旧环境记录那两段历史（第一轮 H3、H4），F1 之后同一个 `current_gate_environment` 判，推的同样看不见；rustup 工具链换库、`VERUS_ROOT`、`VERUS_CVC5_PATH`、`RUST_MIN_STACK` 只读了二进制里的串，没造历史。lkmm 的环境没跑（herd7 按名字归重型）。
- E3：判法一侧三格（必红世界混合、证明混合、空真全没跑成）。没试：红例重放那一侧的混合（它在没跑成时直接抛 Red，一格只有一条 ✗，走不到混合）；包装自己退 250、251、254 各一次（与 252 同一条代码路，`WRAPPER_OWN_EXITS` 查表）；真包装在真 slice 上排满 3600 秒（要占 slice 一小时，没做）。
- 命令行参数次序：`verus-proof-check.py` 只认子命令在第一位，`e165-pin-red-examples.py` 的切法与闸的 `e165_pin_mode` 逐项对过（带值选项同一组、`--check` / `--replay` 后写的定模式），没找到错位。

## 这条腿自己的限度

- 第二轮、单次抽样；「没打中」的几格（J3、J4、J9、J12）照规矩不算数。
- 量 Verus 判定的只有一份三行的玩具文件，没在被判的 crate 上换环境；J5 用的是截断的宏库（看得见的坏法），真实里重编出来的宏库让结论怎么变没量。
- J10 的「崩溃验证员会把整格记成没跑成」是照定义字面推的，没派崩溃验证员去走一遍；判法打两条 ✗ 是量过的。
- J11 的「要的量比总上限还大」那一种只按包装文件头与第一轮 C4 推，没造出来。
- 副本上量出的数（G9、G10 那几行）不算入库装置上的数；`e2-verus-toy.sh` 的峰值表键带了草稿路径，复跑时第一次可能被包装按「上限超过实测峰值 2 倍」拒，脚本里已设 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON`。
- 主仓有三份被判文件在这一轮里被别的会话改过（开头那一段），引文行号取的是主仓此刻的版本；我核过引的那几行在快照拷贝里逐字相同，没核别的行是否挪动。

## 没做什么

- 没跑重型测试、门禁阶段、`--selftest`、样本、lint；判法脚本的干活用法一条都没在真 Verus 上起；没编 Rust。
- 不判 E4 与第一轮判决的复核（归辩方）；不替主 agent 采纳；没读禁读清单里的两样。
- 除这份报告与模型目录之外，仓里一个文件都没写；G9、G10 只打在草稿拷贝里。
- 草稿目录 `/tmp/claude-1000/defs-m3-formal-gate-r2-opus/` 里的仓拷贝、Verus 安装拷贝与 worktree 在交回前删掉（交回里写明）。
