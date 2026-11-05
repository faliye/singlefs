# d137-gov-r2 辩方腿（Sonnet）报告：复核第一轮判决、找过度修补

开工状态（2026-10-29）：被判文件 sha256 与快照 `research/prompts/d137-gov-r2-snapshot/files-sha256.txt` 逐份相符（开工时 `sha256sum -c` 18 份全 OK，量的）。跑到后面 `tools/rerun.sh` 的快照核对报 `scripts/mutation/crates-mutation-rows.py` 此刻 sha256 前 8 位 7632abbb、快照 5f83450d：别的会话在我开工之后改了这一份；本报告没用它，其余 17 份到末尾仍与快照相符（rerun 的核对只报了这一份不同）。
模型目录 `research/prompts/d137-gov-r2-sonnet-model/`，草稿目录 `~/.local/share/singlefs/work/d137-gov-r2-sonnet`。
分片进度（每片做完记一行）：

- 片 1（G1 表）：做完（见「片 1」）
- 片 2（G2 表）：做完（见「片 2」）
- 片 3（G3 表）：做完（见「片 3」）
- 片 4（过度修补）：做完（见「片 4」及「片 4 续」）

结果一览（第一轮判决共 24 条：H1–H13、S1–S7、A1–A4；条数按我的判定数，量的是表里的行）：

- 站得住 18 条：H1、H2、H4、H6、H7、H8、H9、H10、H11、A2、S1–S7、A3。
- 站得住但带保留 3 条：H3（判决里主 agent 现喂的一行写法在真 bash 里不起 cargo，换行写法才是真漏拒）、H5（`;` 写法站得住，`&&`、`||` 守着的失败 cd 是修补造出来的误拒）、A1（条目站得住，修补的上界偏松一格）。
- 够不着或今天只是潜在 3 条：H12（极窄，且修补改了判决「撑住」的一格）、H13（本机有 python3，我没复现）、A4（今天仓里没有真实调用 `detached-run.sh start` 的脚本）。
- 判决「撑住」的有没有被修补改坏：R1 正推腿两组电池 171+232 格两棵树对跑，零格由符合变不符合；准入退码一组 14 行里 3 行有意由退 2 变退 0（H12）；其余几项没验（见「没做什么」）。
- 修补后仍漏 3 条（R1 电池里 Z021、Y014、Y029，真 bash 起 cargo、钩子退 0）。
- 过度修补：口径外 6 类（片 4 的类 A–F），另有登录 shell 噪声一类（类 G，不拒、变慢并多记检出）；真实命令语料 30,595 条里修补前放行、修补后拒 1,893 条（扣掉 272 条回放假象后 1,621 条，其中 sed 1,262 条、四分之三是算术展开）。

复跑命令（整条经内存包装；`--frozen` 本轮没有冻结副本，给开工快照目录，脚本不读它；环境变量 `R1_STUBS_DIRECTORY`、`REAL_COMMANDS` 可选，没给就跳过那两步）：

```
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/d137-gov-r2-sonnet-model/tools/rerun.sh --repo <仓根> --frozen <仓根>/research/prompts/d137-gov-r2-snapshot --work <新的草稿目录> --model <仓根>/research/prompts/d137-gov-r2-sonnet-model
```

模型目录 `research/prompts/d137-gov-r2-sonnet-model/` 每个文件的 sha256（前 16 位，写报告时的状态；全长用 `find . -type f | sort | xargs sha256sum` 取）：

```
1a7647f66ff5a0f7  baseline/bash-command-detector.sh
83aacdab44a377eb  baseline/heavy-test-guard.sh
ce3b4387e5ba601c  baseline/lib_heavy_tests.py
0ea575816c81ad3b  baseline/lib_shell_words.py
472145f2b6a825ed  baseline/runner-dispatch-guard.sh
0e380ffc8f3813ad  baseline/sha256.txt
becb4693a2903f1d  cases/build2.py
b5d97d64ab547efa  cases/build.py
5f988c7de181de69  cases/cdcases.py
1ae8f419f446fb9b  cases/everyday2.py
510c3a892a93b116  cases/everyday.py
40bf888413d2ef1b  cases/r1-extra.py
3998474c56b2bbe1  cases/r1-hits.py
547ea7439617bc69  cases/real.py
e07ef9ff54f5a0f2  cases/repo-scripts.py
f51173ba0557d4fb  cases/shellish.py
1dd0799cc6fde5a1  out/dispatch-base.txt
f158b43a7c429e65  out/dispatch-now.txt
3943fcb4efccca73  out/fuzz-base.tsv
34b5fb9d50e9e158  out/fuzz-now.tsv
2cefa469a0bde63a  out/heavy-base.tsv
2791b74e2fa6f081  out/heavy-now.tsv
688cd24aeb40dd06  out/probe-r1-extra.txt
8fced652e22965fd  out/probe-r1-hits.txt
ff8f929bd033c449  out/run-cd.txt
d07aabab19377b39  out/run-everyday-1.txt
da76146e315417e7  out/run-everyday-2.txt
a21813f991d350e0  out/run-repo-scripts.txt
7a7e12a0531cd985  out/truth-extra.txt
1ce0954d06d2cd28  out/truth-r1.txt
cdf0c0ebe17db8bd  tools/analyze.py
64253d20b76fefd9  tools/analyze-real.py
582c1df706b33a4b  tools/classify-rest.py
47fad3588cbbbf40  tools/classify-sed.py
21a5dacaba4d311c  tools/decl_probe.py
d327689a80e4b75e  tools/extract-commands.py
e0f99f4c76ba558a  tools/mkroots.py
89f6b7dd1222678b  tools/probe.py
4a0d684ef2634dc0  tools/rerun.sh
665bbd99342aa52e  tools/truth.py
```

## 片 4：过度修补（把读而不起、可静态证明无害的正当命令拒掉）

### 方法与范围（量的）

- 喂法：`research/prompts/d137-gov-r2-sonnet-model/tools/probe.py`，cwd 放草稿目录里的树，每条样本一个进程组、超时杀整组；每条命令喂 `heavy-test-guard.sh` 与 `bash-command-detector.sh`，主 agent 与 `three-way-defense` 子 agent 各一遍，各喂两棵树：`now`（此刻主工作区 `.claude/hooks` 整份拷）与 `base`（同，四份被修补的钩子换回修补前备份：`lib_shell_words.py` 0ea57581、`heavy-test-guard.sh` 83aacdab、`lib_heavy_tests.py` ce3b4387、`bash-command-detector.sh` 1a7647f6，备份在 `d137-gov-r2-sonnet-model/baseline/`，来自 `d137-r1fix-hooks/originals/`）。此刻文件 sha256 前 8 位：`lib_shell_words.py` 2d29ef89、`heavy-test-guard.sh` 6f760174、`bash-command-detector.sh` 14538fbf、`lib_heavy_tests.py` 3a022885（与快照相符）。
- 语料是我自己列的日常写法，不是随机抽样：`cases/everyday.py` 与 `cases/shellish.py`（421 条命令，1684 次喂法）、`cases/everyday2.py`（302 条命令，1208 次）、`cases/cdcases.py`（42 条）、`cases/repo-scripts.py`（本仓 121 份 shell 脚本各 `bash <脚本> --help`，484 次）。所以下面的数是「语料里」的数，不是日常命令里的拒绝率。
- 结果文件：`out/run-everyday-1.txt`、`out/run-everyday-2.txt`（原样）；汇总 `tools/analyze.py`。语料里修补前放行、修补后拒的不同命令 65 条（量的，`analysis12.txt`，按钩子加命令去重），全部出在 `heavy-test-guard.sh`；`bash-command-detector.sh` 在这些命令上修补前后退出码逐条相同（量的）。主 agent 与子 agent 的新拒绝逐条相同（量的，无「只拒一方」的差）。
- 本仓 121 份 shell 脚本：`bash <脚本> --help` 在两棵树上退出码逐份相同（`heavy-test-guard.sh` 主 agent 44 份拒、子 agent 46 份拒，修补前后同；`bash-command-detector.sh` 1 份拒，同），修补只让 2 份多记了检出（量的，`run3.txt`）。就是说「读进来的脚本正文宽判、只记」这一支在本仓脚本上没造出新拒绝。

### 过度修补清单（喂法与原样输出；每条先写是哪一类）

下面每一条的「原样输出」是 `probe.py` 打出的行与它的头一条拒绝说明，主 agent 身份，`heavy-test-guard.sh`。`base:rc=0` 即修补前放行。

**类 A：sed 脚本里的算术展开 `$((…))`、`$[…]`（算术展开的结果只能是整数，不会是 `e` 命令）。** 主 agent 撞上的那条的同类，我的语料里共 9 条拒。
```
now:rc=2/n=1/0.1s base:rc=0/n=0/0.1s CHANGED | heavy main | sed | 'n=40; sed -n "$((n-3)),$((n))p" README.md'
    now> ✗ 认不出起什么的写法被拒：sed 脚本解析不下去，认不出里面起不起命令（sed 里起的命令）：主 agent 跑「认不出起什么的写法」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
now:rc=2/n=1/0.1s base:rc=0/n=0/0.1s CHANGED | heavy main | sed2 | 'sed -n "$(( n ))p" README.md'
now:rc=2/n=1/0.1s base:rc=0/n=0/0.1s CHANGED | heavy main | sed2 | 'sed -n "$[n-3],$[n+3]p" README.md'
```
同类还有 `sed -n "$((n - 3)),$((n + 3))p"`、`sed -n "$((n-3)),${m}p"`、`sed -n "$((${n}-3)),$((${n}+3))p"`、`sed -n "$(( 10 + 5 )),30p"`（都在 `out/run-everyday-2.txt` 与 `out/run-everyday-1.txt`）。`sed -n "${a},${b}p"`、`sed -n "${n}p"`（变量展开）修补前后都放行，所以同一件事「变量放、算术拒」是不一致的：算术比变量更确定是整数。

**类 B：sed 的 `a\`、`i\`、`c\` 多行文本写法（标准 sed 语法，不是认不出）。** 3 条拒。
```
now:rc=2/n=1/0.1s base:rc=0/n=0/0.1s CHANGED | heavy main | sed2 | "sed '/x/c\\\nfoo' README.md"
    now> ✗ 认不出起什么的写法被拒：sed 脚本解析不下去，认不出里面起不起命令（sed 里起的命令）：…
now:rc=2/n=1/0.1s base:rc=0/n=0/0.1s CHANGED | heavy main | sed2 | "sed '/x/a\\\nfoo\\\nbar' README.md"
now:rc=2/n=1/0.1s base:rc=0/n=0/0.1s CHANGED | heavy main | sed | "sed '/x/i\\\nfoo' README.md"
```
（命令是 `sed '/x/c\` 换行 `foo'`。）根因是解析器 bug，不是「认不出」：`.claude/hooks/lib_shell_words.py:3324` 的 `if command in "aicrRwW#":` 只跳到第一个换行为止；`c\` 之后紧跟换行，文本在下一行，下一行的 `foo` 被当 sed 命令读，`f` 不在认得的命令里，落到 `.claude/hooks/lib_shell_words.py:3352` 的「sed 脚本解析不下去」。`sed '/x/a foo'`（一行写法）放行，只拒多行写法。

**类 C：python 里 `subprocess.run([...])`，命令词是字面的、只有后面某个参数不是字面的**（正文点名的两条之一的同类，语料里 python 类新拒 24 条，其中我判为过度的 14 条，范围：`-c` 与 `python3 - <<'EOF'` 两种）。
```
now:rc=2/n=1/0.1s base:rc=0/n=0/0.1s CHANGED | heavy main | python | 'python3 -c \'import subprocess,sys; subprocess.run(["ls", sys.argv[1]])\' /tmp'
    now> ✗ 认不出起什么的写法被拒：python 代码里 subprocess.run(…) 的命令不是字面的，认不出起的是什么（python3 里起的命令）：…
```
（另一条原样见 `out/run-everyday-2.txt` 里 `check=False` 那一行。）同类还有 `["git","diff","--stat",path]`、`["git","-C",root,"status"]`、`["wc","-l",f]`、`["git","show",sha]`、`["ls","-l",f"{d}/x"]`。根因：`.claude/hooks/lib_shell_words.py:3054` 的 `python_literal(positional[0])` 对整张 argv 列表要求每个元素都是字面，一个元素不是就整条记成命令不是字面的（`.claude/hooks/lib_shell_words.py:3059` 的 `if arguments is None:`），不单独看命令词。命令词是 `ls`、`wc`、`git status` 这类不会再起别的程序的命令时，参数是什么都不会让它起 cargo。

**类 D：python 里把字面列表先赋给变量再 `subprocess.run(变量)`、字面字符串的 `.split()`。** 2 条拒（比类 C 弱：需要常量传播；列在这里是因为它们和日常写法一一对应）。
```
now:rc=2/n=1/0.1s base:rc=0/n=0/0.1s CHANGED | heavy main | python | 'python3 - <<\'EOF\'\nimport subprocess\nargs = ["git", "status"]\nsubprocess.run(args)\nEOF'
now:rc=2/n=1/0.1s base:rc=0/n=0/0.1s CHANGED | heavy main | python2 | 'python3 -c \'import subprocess; subprocess.run("ls -l".split())\''
```

**类 E：awk 里命令赋给变量再 `| getline`、或用命令行 `var=值` 赋命令。** 2 条拒（赋的是字面）。
```
now:rc=2/n=1/0.1s base:rc=0/n=0/0.1s CHANGED | heavy main | awk2 | 'awk \'BEGIN { cmd = "date"; cmd | getline d; close(cmd); print d}\''
    now> ✗ 认不出起什么的写法被拒：awk 程序里 … | getline 的命令不是字面字符串，认不出起的是什么（awk 里起的命令）：…
now:rc=2/n=1/0.1s base:rc=0/n=0/0.1s CHANGED | heavy main | awk2 | "awk '{ print $1 | cmd }' cmd='sort' README.md"
```

**类 F：`cd` 之后起相对路径脚本（H5 的修补）。** 语料里 34 条拒（两份 `everyday` 语料 15 条，`cdcases.py` 19 条，`out/run-cd.txt`），拒的写法里包括明显无害的：
```
now:rc=2/n=1/0.1s base:rc=0/n=0/0.1s CHANGED | heavy main | trap-cd-exec | 'cd $DIR && ./run.sh'
    now> ✗ 认不出起什么的写法被拒：直接执行的脚本 ./run.sh 的当前目录算不出，而这一条的当前目录算不出，认不出起的是哪一份：…
now:rc=2/n=1/0.1s base:rc=0/n=0/0.1s CHANGED | heavy main | misc2 | 'cd "$PWD" && ./x.sh'
now:rc=2/n=1/0.1s base:rc=0/n=1/0.1s CHANGED | heavy main | misc2 | 'cd /tmp/nonexist-xyz && ./x.sh'
    now> ✗ 认不出起什么的写法被拒：直接执行的脚本 ./x.sh 要先 cd 到 /tmp/nonexist-xyz，那个目录不在、cd 会失败，认不出起的是哪一份，而这一条的当前目录算不出，认不出起的是哪一份：…
```
- `cd "$PWD"` 是空操作，目录就是当前目录，钩子自己的 `expand_known_variables` 认 `$PWD`，仍拒。
- `cd 不存在的字面目录 && ./x.sh`、`cd 不存在的字面目录 || exit 1; bash x.sh`：`&&`、`|| exit` 之后 cd 失败时脚本根本不会起，仍拒（`cdcases.py`：`cd /tmp/nonexist-xyz || exit 1; bash benign.sh` 退 2，修补前退 0 记 1 条检出）。只有 `cd 不存在的目录; bash x.sh`（分号）才真会在原目录起脚本。
- `cd "$(git rev-parse --show-toplevel)" && ./benign.sh`、`for d in sub .; do (cd $d && ./benign.sh); done`、`cd "$d" && make`、`cd "$d" && ./configure` 都拒。
- 对照：同样的 `cd "$(git rev-parse --show-toplevel)" && bash scripts/hosts/run-with-memory-cap.sh --status` 与 `cd "$REPO" && bash scripts/…` 修补前后都放行（相对脚本恰好在当前目录下也有一份，按当前目录那一份判）。就是说拒不拒取决于相对路径在当前目录下碰巧有没有同名文件，不取决于命令写得好不好。

## 片 1：G1 表（钩子）逐条复核

方法：每条「打中」的喂法拿两棵树（`now` 此刻、`base` 修补前）喂 `heavy-test-guard.sh`，主 agent 与 `implementation-writer` 各一遍（`cases/r1-hits.py`、`cases/r1-extra.py`，原样输出 `out/probe-r1-hits.txt`、`out/probe-r1-extra.txt`）；同一条命令另在真 bash 里起一遍，PATH 上放一个只记日志的假 `cargo`，看它有没有被起（`tools/truth.py`，每条样本 `setsid --wait` 起、超时与结束对整组发 KILL，`out/truth-r1.txt`、`out/truth-extra.txt`，跑完 `ps` 核无残留）。「够得着」= 真 bash 里假 cargo 被起而修补前钩子放行。「换成判不了放行并记检出还打中吗」看 `base` 的检出条数 n：n=0 是钩子压根没认出这里有东西可判，记检出这一臂也没有东西可记，打中同样成立；n≥1 是钩子认出了判不了、只是放行，一律拒是对症的修。「真实频率」= 会话记录里抽的 162,809 条不同命令（2026-10-10 到 2026-10-28，不含 tooling-writer、不含命令里提到 hooks/ 与 heavy-test 的；量的，`tools/extract-commands.py`，语料含私有路径只放草稿目录）里出现该形状的条数，用来估账面上的触及面，不用来判对错。

| 条 | 辩方判定 | 证据（量的） |
|---|---|---|
| H1 `bash -euo pipefail -c` | 站得住 | 真 bash 起 cargo；`base` 退 0、n=1；`now` 退 2，同形无重活 `bash -euo pipefail -c 'echo hi'` 退 0 且 n 从 1 降到 0。R1 模型 Y064–Y073 十格全 漏拒→符合。真实频率 1 条（还是 shell-words 轮的 fuzz 脚本）。 |
| H2 trap 字符串 | 站得住 | 真 bash 起 cargo；`base` 退 0、n=0（静默，换成「判不了放行并记」这一臂同样漏）；`now` 退 2，`trap 'rm -f /tmp/x' EXIT` 退 0。真实频率：带引号动作的 trap 13 条，动作里有重型词的 2 条（一条是 forward 腿写的样本）。 |
| H3 alias + expand_aliases | 站得住，但判决里主 agent 现喂的那一条不是真漏拒 | 判决现喂 `shopt -s expand_aliases; alias t='cargo test --all'; t` 在真 bash 里没起 cargo（量的：`out/truth-r1.txt` H3-红 `cargo_called=no`，bash 读完整行才执行，同一行刚定义的别名不生效）；换行写法起了（`out/truth-extra.txt` H3-换行-红 yes）。R1 正推腿自己的样本是换行写法，所以条目站得住，判决里那一格的「现喂退 0」不能当漏拒证据。修补后一行与换行都拒（一行写法属偏严误拒，无害）；但 R1 模型 Z021 `alias c=…; eval c` 与 Y014 `bash -O expand_aliases -c "alias…\nc"` 修补后仍退 0 且真起（见下「修补后仍漏」）。 |
| H4 标准输入一族 | 站得住 | 真 bash 起 cargo；`base` 退 0、n=0（静默）；`now` 对 `/dev/stdin`、`exec <`、`{ bash; } <`、`( bash ) <`、`while…done <` 都退 2，`bash /dev/stdin < benign.sh` 退 0。R1 模型 Y015–Y028 里变化的 11 格都由漏拒转符合。仍漏：Y029 `if true; then bash; fi < heavy.sh`（真起、退 0）。真实频率 27 条（多半是 `/dev/stdin` 当 diff 输入）。 |
| H5 cd 失败/算不出之后的相对脚本 | 分叉：`;` 写法站得住，`&&`、`\|\| exit` 写法是修补造出来的误拒 | `cd missing-dir; bash heavy.sh` 真起 cargo、`base` 退 0、`now` 退 2。但 `cd missing-dir && bash heavy.sh` 与 `cd missing-dir \|\| exit 1; bash heavy.sh` 真 bash 里假 cargo 没被起（`out/truth-extra.txt`），`now` 仍退 2、`base` 退 0。详见片 4 类 F。 |
| H6 `~+` | 站得住 | 真起；`base` 退 0、n=1；`now` 退 2。真实频率 2 条。 |
| H7 先写出再起（多源 cat、`>>`） | 站得住 | 真起；干净沙箱里 `base` 退 0、n=0（静默），`now` 退 2。（我第一遍喂时沙箱里留着上一步真起写出的 d7，`base` 因此读到文件而退 2；清掉重喂得 n=0，写在这里是因为这是装置造出来的偏差，已清掉重喂。） |
| H8 改名的 shell | 站得住 | `cp /bin/bash ./mysh && ./mysh -c …`、`ln -sf`、`rbash`、`busybox sh` 真起；`base` 退 0、n=0；`now` 退 2；`cp /bin/ls ./myls && ./myls` 退 0。真实频率 0 条。 |
| H9 别的解释器里起 shell | 站得住（真起）；修补面过大 | `python3 -c 'os.system(…)'`、`perl -e system`、`awk system`、`sed 1e`、`git -c alias.x=!…` 五种真起；`base` 退 0、n=0；`now` 全退 2，同形 `echo hi` 退 0。R1 模型 Z044–Z053 里 9 格、Y082–Y088 与 Y092 由漏拒转符合（Y089、Y090 是「放过且没起」转成「拒了但没起」）。代价见片 4：同一支代码把 `sed -n "$((n-3))…p"` 之类拒了，真实语料里新拒 1,262 条 sed、264 条 python。 |
| A2 BASH_ENV 包进转发函数 | 站得住 | `f(){ bash -c true; }; BASH_ENV=e.sh f` 真起；`base` 退 0、n=0；`now` 退 2；`f(){ bash -c true; }; f` 退 0。真实频率 2 条（BASH_ENV 字样）。 |
| H10 引号花括号作命令词（误拒） | 站得住 | `'{cargo,test,--all}'` 真 bash 没起 cargo；`base` 退 2，`now` 退 0；`{cargo,test,--all}`（无引号，真展开）两边都退 2。 |
| H11、S4 说明过时 | 站得住 | 旧句「分不出花括号」在四份钩子里现为 0 处（`grep -c`），新句在 `.claude/hooks/bash-command-detector.sh:1963` 一带。 |
| H12 准入判法不在/自己退 2 | 够得着但极窄；且修补改动了判决「撑住」的一格 | 仓里 `scripts/gate/admission.py` 一直在；够得着要 python 起不来。R1 模型的 `dispatch_probe.py` 两棵树各跑：`adm2.py`（只 `sys.exit(2)`、不打字）、`nonexistent.py`、「不是重跑登记」三行由 `base` 退 2 变成 `now` 退 0 并记 1 条检出，别的 11 行（共 14 行，不是判决写的 15 格，我这边的探针脚本打 14 行）逐行相同（`out/` 里由 `rerun.sh` 重生）。判决「撑住」写的是「准入退码 15 格里 14 格」，用户口径字面是「退 2、退 3 照拒」，修补把「退 2 而没打两句准入句子」改成放行，是一次有意的口径细读（`.claude/hooks/runner-dispatch-guard.sh:1607` 的 `ADMISSION_REGISTRATION_SENTENCE`），但不是用户原话；真 `admission.py` 的退 2 路径（`scripts/gate/admission.py:3154` 起的 `admit_experiment`）打的两句都能被它认出，用法错与弄坏开关的退 2 则放行。建议主 agent 向用户确认这一句读法。 |
| H13 python3 不在 PATH | 够不着（本机有 python3），我没自己复现 | 取 `d137-r1fix-hooks-4/result.md` 的读数（改前退 78、0 条检出，改后退 0、1 条检出）；判决自己也写「偏窄」。 |

### 判决「撑住」的有没有被修补改坏（量的）

- R1 正推腿 G1 样本 171 格（`cases_heavy.py`、`cases_heavy2.py`，沙箱照 `setup_sandbox.sh` 重建，两棵树各跑，`out/heavy-now.tsv`、`out/heavy-base.tsv`）：166 格修补前后都符合；1 格由不符变符合（A43 `{cargo,test,--all}`）；4 格标「漏拒」的（M02、M06、M11、M13）是主 agent 带 `SINGLEFS_HEAVY_TESTS=commit` 前缀、期望放行的格，两边相同。没有一格由符合变不符。
- R1 正推腿 fuzz 样本 232 格（`run_fuzz.py`，`out/fuzz-now.tsv`、`out/fuzz-base.tsv`）：142 格修补前后都符合；78 格漏拒→符合；3 格漏拒→漏拒（Z021、Y014、Y029，见下）；5 格由「放过且没起」变成「拒了但没起」（Y005 顶层 `trap … RETURN`、Y006 `trap … SIGTERM`、Y031 `bash ~-/heavy.sh`、Y089 `git -c core.pager=… diff --no-index --stat`、Y090 `less +'!…' < /dev/null`）：偏严的误拒，都是我所说的「口径内」，列出来只为数清。
- 同族脚本拷贝边界、⑫ 写范围、撤旧根 29 格：我没重跑（`battery_other.py` 往 HOME 与会话临时目录根写样本，出了我的写范围），所以这三项「没验改坏没有」。

### 修补后仍漏（修补不完全；不是新打中，是判决已打中的类里剩下的）

R1 的 fuzz 样本喂 `now` 仍退 0 而真 bash 起 cargo 的三条：Z021 `shopt -s expand_aliases; alias c='cargo test --all'; eval c`；Y014 `bash -O expand_aliases -c "alias c='cargo test --all'` 换行 `c"`；Y029 `if true; then bash; fi < heavy.sh`。另我自己的 `r1-extra.py` 里 `H3-eval-红` 同 Z021（`now` 退 0，真 bash yes）。推翻条件：这三条喂 `now` 任一条退 2。

## 片 2：G2 表（定义、共用约束、规则）逐条复核

此刻文件 sha256 前 8 位：`.claude/agent-common.md` 65769f7a、`.claude/main-agent.md` 612a3de2（与快照相符）。

| 条 | 辩方判定 | 证据 |
|---|---|---|
| S1 共用约束理由句 | 站得住，已改 | `.claude/agent-common.md:81` 现为「记一条检出；写成 `setsid --wait`。」，理由已删。 |
| S2 `lib_heavy_tests.py` 文件头说 case 模式行被切成命令 | 站得住，已改 | `.claude/hooks/lib_heavy_tests.py:218`、`:256` 现写由 `case_patterns_and_conditionals_folded` 折叠、不当命令位置；`grep -n '被切成命令' .claude/hooks/lib_heavy_tests.py` 现只剩 :844 一处（讲通配字符，不是 case 模式行）。 |
| S3 文件头写「照拒」、代码只在已拒之后加说明 | 站得住，已改；且与用户口径同向 | `.claude/hooks/heavy-test-guard.sh:1996` 起 `missing_cat_sources` 成为拒绝说明；`:937` 的注释自己写明「`&&` 后面其实不跑」，修补仍拒，是因为用户在第一轮定了「cat 不看源文件在不在：照拒」，不是过度修补。 |
| S4 检出钩子说明过时 | 站得住，已改 | 同 H11。 |
| S5 「会话临时目录」与「会话共用的暂存目录」并存 | 站得住，已改 | `grep -c '会话共用的暂存目录' .claude/agent-common.md` 为 0；统一叫「会话临时目录根」，在第 39、44、85、96 行。 |
| S6 派发闸那一句 | 站得住，已改 | `.claude/agent-common.md:96` 现带「且标准错误里有它自己报」；与 `.claude/hooks/runner-dispatch-guard.sh:1607` 一致。口径细读见 H12。 |
| S7 「钩子 bash 那一层起不来」 | 站得住，已改 | `.claude/agent-common.md:76`、`:91` 现带「含 PATH 上没有 python3」。 |

撑住的有没有被改坏：
- 进程组一句：`.claude/agent-common.md:63` 与三份 `three-way-*.md` 里那一句，从「装置与判法里起样本进程」到「那一条的 ⑥）」逐字符取出来 sha256 前 8 位都是 bf4841d2、169 字符（量的，`python3 -I` 取出）。S1 改的是第 81 行，没碰这一句。
- 定义与代码对得上的几处：`.claude/main-agent.md:70` 的声明处置句（「扣掉算数的串行秒数，剩下的时间把这一窗口的 CPU 全算给它也到得了阈值」）与 `scripts/hosts/core_usage.py:489` 起的 `declaration_excuses_low` 同一判法（`:502` 一行 `return undeclared <= 0 or cores * span / undeclared >= threshold`）。
- 我没重核「开工先读引用 74 处」「`--ack` 四种键三处相同」「verification、commit-stages、CLAUDE.md 各一句」：这一轮修补没碰这些文件或这些句子（`d137-r1fix-defs/own.diff` 只动 `agent-common.md` 与 `main-agent.md`），没验。

## 片 3：G3 表（看门狗、core_usage）逐条复核

此刻文件 sha256 前 8 位：`scripts/agents/agent-watch.py` 0c41175d、`scripts/hosts/core_usage.py` 33016c5b、`scripts/hosts/detached-run.sh` 0a9d3e70（与快照相符）。

| 条 | 辩方判定 | 证据 |
|---|---|---|
| A1 声明一次永不闭合，真卡住叫不醒 | 站得住（够得着），修补有效；修补的上界偏松一格 | 改前模型读数在 `d137-r1fix-watch/result.md` 里（改前「永不闭合」静默，改后 7200 秒封顶，没写预计秒数的 600 秒窗口）。修补后的判法是 `scripts/hosts/core_usage.py:489` 的 `declaration_excuses_low`，把窗口里全部 CPU 秒都算给没声明的那一截；声明的串行段至少占一个核，所以这个上界偏松。`tools/decl_probe.py` 量的：cores=4、窗口 600 秒、声明 300 秒、阈值 7.75，函数交 True（只记不叫醒），而声明段用一个核时没声明那一截的平均只有 7.00，低于阈值。五组读数里按「声明段占一个核」重算，第 1 组（上面这一组）与第 5 组（1 核、声明 590 秒，剩 10 秒）和函数的结论不同；第 5 组声明几乎盖满窗口，放是合理的；第 2、3、4 组相同。所以松的主要是第 1 组这种「声明约占一半、其余时间离阈值只差一点」的读数。不是大漏，列出来给主 agent 决定要不要收紧（推翻条件：同一读数下按「扣一个核」算也到阈值）。 |
| A3 两处门槛不一 | 站得住，已并成一处 | `scripts/agents/agent-watch.py:3375` 的 `declaration_quiets_low` 在 `:3383` 只调 `core.declaration_excuses_low(reading)`，没有第二个门槛。 |
| A4 看门狗按主会话记录认 `--label` | 站得住是潜在的，今天仓里没有真实调用者；改法本身无害 | 在 `scripts` 与 `.claude/gate.d` 里找非注释、非自证的 `detached-run.sh start` 调用：0 处（`grep -rnE` 出来的 `detached-run.sh` 内部几处都是它自己的自证与说明）。攻方腿的场景「脚本里起、主会话记录里没有 `--label`」要主 agent 或子 agent 另写一份调用它的脚本才成立，修补报告也承认「没在真多会话、真 systemd 单元下复现」。修补后单元自己带会话号（`scripts/hosts/detached-run.sh:843` 写 `owner-session`），认不出的照叫醒；副作用是改动之前起的单元登记里没有这个字段，它们的低核记录从此照叫醒（修补报告已写）。 |

### 第一轮判决方法有没有漏

1. 判决现喂了 8 行，其余 H5–H13 采为线索。我把线索里的 H5、H6、H7、H9、H10、H12 都喂了：H5/H6/H7/H9 在真 bash 里真起 cargo 而 `base` 放行，H10 是真误拒，H12 在假准入判法下复现；H11 只是说明文字；H13 我没自己复现。线索没有一条是空的，所以「只现喂 7 条」没有漏掉假阳性。
2. 判决漏的是另一头：整份判决没有一条量了偏严的代价。R1 正推腿自己量过一次（20 条手挑的命令里 7 条被拒，它写「不算误拒」），判决把这一档记作「撑住」，没有把它转成任何一条要查的东西。我用 30,595 条真实命令补了这头（片 4）：修补前放行、修补后拒的有 1,893 条，扣掉 272 条因旧草稿目录已被删而造成的回放假象（拒绝说明写「要先 cd 到」一个现在不在盘上的字面目录；量的），仍有 1,621 条，其中 1,262 条是 sed 脚本解析不下去。
3. 判决里 H3 一行的现喂不是真漏拒（见 H3 行）。判决没对每条现喂都配「真 bash 起没起」，所以这一格的证据站不住，虽然条目本身站得住。
4. 「撑住」那几项判决写的是「腿量过、主 agent 没逐条复跑」。我复跑了其中 G1 的两组（171+232 格，两棵树）：没有一格由符合变不符合。其余（准入退码 15 格、撤旧根、⑫ 写范围、同族拷贝边界）：准入退码我跑了（14 行，3 行有意改变，见 H12）；别的我没跑。

### 片 4 续：真实命令语料上的量（无偏的一份加形状命中的全部）

`cases/real.py`：从会话记录抽的 Bash 命令（见片 1 的语料说明；不同命令 162,685 条，长度不超过 20,000 字符），形状命中（含 `sed … $(`/`$[`、`subprocess.`、`os.system`、`awk`、`cd "$…`、`trap`、`bash -l`、`expand_aliases`、`/dev/stdin`、`BASH_ENV`、`git -c`、`perl -`、`xargs`、`find … -exec`、`make`）的 25,595 条全喂，没命中的 137,090 条里取固定种子（137）的随机 5,000 条，主 agent 的命令按主 agent、子 agent 的按 `implementation-writer` 喂 `heavy-test-guard.sh`，`now` 与 `base` 两棵树（共 30,595 行，`tools/analyze-real.py`、`tools/classify-sed.py`、`tools/classify-rest.py`；语料含私有路径，只有条数进仓）。

| 量 | 条数（量的） |
|---|---|
| 修补前放行、修补后拒 | 1,893 |
| 修补前拒、修补后放（H10 那一类） | 0 |
| 两边都拒 | 999 |
| 其中拒绝说明是「要先 cd 到〈字面目录〉，那个目录不在」 | 272（回放假象：这些命令 cd 的草稿目录后来被删了，当时在盘上的会放行；不计入下面的过度修补数） |
| 其中 sed 脚本解析不下去 | 1,262：只含算术展开 `$((…))`/`$[…]` 的 951；含 `$(…)`/反引号的 285；算术与命令替换并存的 10；其他 16（多是 `xargs -I{} sed -n '{}p'` 的占位符） |
| 其中 python 里 `subprocess.*`/`os.*` 的命令不是字面的 | 264 |
| 其中 python `exec(…)`、`python3 - <<EOF` 结束词没带引号 | 17、37 |
| 其中 awk、`sed s///e` | 5、2 |
| 其中 `cd` 目标算不出之后的相对脚本 | 5 |
| 其余未分类（多是整条带 `SINGLEFS_HEAVY_TESTS=commit` 前缀的长命令） | 29 |
| 无偏随机样本 5,000 条里修补前放行、修补后拒 | 22；其中 17 条是上面的 cd 回放假象，剩 5 条（`exec(…)` 4 条、`python3 - <<EOF` 结束词没带引号 1 条），即 0.1% |

读法：
- 在「没有这些形状」的命令里（随机样本），修补几乎不碰；新拒绝几乎全落在 sed、python 两族，而 sed 一族里四分之三（951/1,262）是 `$((…))` 算术展开：算术展开的结果只能是整数，不会是 `e` 命令，这一支的拒绝没有一个能落在「起命令」上。
- 这个数不是「日常命令被误拒的比率」：形状命中那 25,595 条是我按形状挑的，只有 sed/python/cd 三族可以按条数读。会话里的 agent 在老钩子下写这些命令，新钩子下会改写；真实代价是改写次数，子 agent 没有 `SINGLEFS_HEAVY_TESTS=commit` 可用（只能改写，改不成交回主 agent），主 agent 可带前缀绕过。
- 推翻条件：把 `real.py` 的语料换成别的时间段（或别的会话记录），sed 算术一支的新拒数明显变少，或这些命令在老钩子下本来就被别的钩子（检出钩子、写闸）拒掉。

### 过度修补清单汇总（先分口径内与口径外）

「口径内」指拒绝落在用户定的「认不出的一律拒（偏严）」之内、命令里确有一处钩子无法静态定下的内容；「口径外」指命令里不起命令的内容已经可以静态定下，钩子仍拒。下面只有口径外的算过度修补。

| 类 | 写法 | 口径 | 语料里新拒 | 真实语料里新拒（量的） | 根因 |
|---|---|---|---|---|---|
| A | sed 脚本里的 `$((…))`、`$[…]` | 口径外 | 9 | 951 | 算术展开结果只能是整数，解析器没当整数处理（`.claude/hooks/lib_shell_words.py:3352` 落到「解析不下去」） |
| B | sed 的 `a\`、`i\`、`c\` 多行文本 | 口径外 | 3 | 0（sed 解析不下去的新拒里「其他」16 条我逐条看过，是 `xargs -I{} sed -n '{}p'` 的占位符与变量地址，没有一条是多行文本写法） | `.claude/hooks/lib_shell_words.py:3324` 只跳到第一个换行，文本的下一行被当 sed 命令读 |
| C | python `subprocess.run([字面命令词, …, 变量])` | 口径外（命令词是 `ls`/`wc`/`git` 这类不会再起别的程序的字面词时） | 14 | 264 条里的一部分（含命令词字面、参数含变量的），具体份额我没拆 | `.claude/hooks/lib_shell_words.py:3054` 对整张 argv 要求每个元素字面，不单看命令词 |
| D | python 字面列表先赋给变量再 `subprocess.run(变量)`、`"ls -l".split()` | 口径外但需要常量传播 | 2 | 未拆 | 同 C |
| E | awk `cmd = "date"; cmd \| getline`、命令行 `cmd=sort` 赋命令 | 口径外但需要常量传播 | 2 | 5 条 awk 里的一部分，未拆 | 只认字面字符串，不认赋过字面值的变量 |
| F | `cd` 之后起相对路径脚本：`cd "$PWD"`、`cd 不存在的字面目录 && …`、`\|\| exit`、`cd "$(git rev-parse --show-toplevel)" && ./x.sh`、`cd "$d" && make` | `cd "$PWD"` 与 `&&`/`\|\| exit` 守着的失败 cd 是口径外；`cd "$d"` 目标算不出是口径内 | 34（`everyday` 两份语料 15 条加 `cdcases.py` 19 条，各自去重；`out/run-cd.txt`） | 5（另 272 为回放假象） | `.claude/hooks/heavy-test-guard.sh:1388` 起把「目录不在」一律当「cd 失败、留在原目录」，没分 `;` 与 `&&`/`\|\| exit`；`$PWD`、`$OLDPWD` 的已知值没用上 |
| G | 登录 shell（`bash -lc`、`sh -l`、`exec -l`） | 不是拒绝：每条多 2.8 秒、多记 1 条检出 | 6 条命令（退出码仍 0） | 22 条命令有这形状 | 现在会读 profile 与 bashrc，本机 bashrc 里 source 一个不存在的 `~/.bash_aliases`，每次记一条「脚本不存在」检出（量的：喂法见 `tools/probe.py`，量 `out/run-everyday-1.txt` 里 `bash -lc 'echo hi'` 一行，`now` 2.8 秒、n=1，`base` 0.1 秒、n=0） |

口径内的例子（不算过度，列出来免得被误当过度）：`sed -n "$(wc -l < f)p"`、`sed -n "$LINE p"`、`awk '{system("echo " $1)}'`、`sed 's/a/b/e'`、`python3 -c 'exec(open(…).read())'`、python 里自定义 `run(*a)` 包装、`subprocess.run(cmd, shell=True)` 的 `cmd` 无定义、`git -c core.editor="$EDITOR" commit`：命令里确有一处值钩子定不下来。

活体观察（量的，不是修补引入的）：本会话里我自己写的 `M=$PWD/research/x; python3 $M/tools/mkroots.py` 被活钩子当「python 脚本路径由变量拼出」拒了；同一条喂 `base` 同样退 2（`smoke.py` 的 `live-M-pwd` 行），所以是老规则，不是这一轮的。

## 引文原样（行号现取，`grep -nF` 或 `awk 'NR==行号'`）

`.claude/hooks/lib_shell_words.py:3324`：「if command in "aicrRwW#":」
`.claude/hooks/lib_shell_words.py:3352`：「unjudged.append("sed 脚本解析不下去，认不出里面起不起命令")」
`.claude/hooks/lib_shell_words.py:3054`：「arguments = python_literal(positional[0]) if positional」
`.claude/hooks/lib_shell_words.py:3059`：「if arguments is None:」
`.claude/hooks/heavy-test-guard.sh:1275`：「DIRECTORY_UNRESOLVED_SUFFIX = "，而这一条的当前目录算不出"」
`.claude/hooks/heavy-test-guard.sh:1388`：「cd 到的目录是字面路径、盘上不在：cd 会失败、留在原来的目录」
`.claude/hooks/heavy-test-guard.sh:937`：「cat 报错退 1，`&&` 后面其实不跑」
`.claude/hooks/runner-dispatch-guard.sh:1607`：「ADMISSION_REGISTRATION_SENTENCE = re.compile」
`scripts/gate/admission.py:417`：「EXIT_REGISTRATION_ERROR = 2」
`scripts/hosts/core_usage.py:489`：「def declaration_excuses_low(reading):」
`scripts/hosts/core_usage.py:502`：「return undeclared <= 0 or cores * span / undeclared >= threshold」
`scripts/agents/agent-watch.py:3383`：「return core.declaration_excuses_low(reading)」
`scripts/hosts/detached-run.sh:843`：「detached_run_write_field "$record" owner-session」
`.claude/agent-common.md:81`：「记一条检出；写成 `setsid --wait`。」
`.claude/hooks/bash-command-detector.sh:1963`：「这道闸切词已分得出引号里外」

## 没做什么

- 没撤销或维持第一轮判决，只交复核结果（定义固定写法）。
- 没重跑第一轮的「同族脚本逐字节相同拷贝边界」「撤旧根两样 29 格」「⑫ 写范围 24 格」「检出钩子 ②④⑦⑬⑮ 30 格」「开工先读引用 74 处」「`--ack` 四种键三处相同」：它们要么往 HOME 与会话临时目录根写样本（`d137-gov-r1-sonnet-model/tools/battery_other.py`，出了我的写范围），要么这一轮修补没碰对应文件。这些项「没验改坏没有」。
- 没复现 H13（python3 不在 PATH），取各件报告的读数。
- 没验检出钩子 ⑬ 读脚本正文那一支在真实 shell 脚本语料上的误拒：本仓 121 份 shell 脚本的 `bash <脚本> --help` 两棵树退出码逐份相同，但那是本仓脚本，不是 agent 现写的脚本。
- 真实命令语料只喂了 `heavy-test-guard.sh`（主 agent 与 `implementation-writer` 身份）；检出钩子在日常命令语料（`everyday*.py`）上修补前后逐条相同，在真实语料上没喂。
- 真实语料来自会话记录，agent 在老钩子下写的命令；新钩子下他们会改写，所以「新拒绝条数」是上界意义上的改写次数，不是失败次数；我没量改写的成本。
- 没起 cargo、没碰 GPU、没跑重型测试；喂钩子与复跑都经 `scripts/hosts/run-with-memory-cap.sh 4G`（排得上），真 bash 对照里的 `cargo` 是只记日志的假脚本。
- 没改主工作区任何文件。草稿与模型目录以外只读。`~/.claude` 下会话记录只读（抽命令），抽出的 JSON 行含私有路径，只放草稿目录，不入库。
- 没提交。

## 什么现象会推翻这里的结论

- 片 1：把 `cases/r1-hits.py` 与 `cases/r1-extra.py` 的任一条喂回此刻的钩子，退出码与本报告表里写的不同；或真 bash 对照里假 cargo 的起/没起与表里写的不同。
- 片 2：上面「引文原样」任一条在被引文件里 `grep -nF` 命中 0 次，或进程组一句四处的 sha256 前 8 位不再相同。
- 片 3：`tools/decl_probe.py` 在此刻 `core_usage.py` 上打出的 `excused=` 与表里写的不同；`scripts` 与 `.claude/gate.d` 里出现了真调用 `detached-run.sh start` 的脚本（A4 就不再是潜在的）。
- 片 4：`tools/rerun.sh` 的摘要行里「修补前放行、修补后拒」的条数明显变小（被下一次修补收窄），或 sed 算术一支在真实语料上不再被拒。

## 草稿目录清理（交回前）

删前 `du -sh`（量的）：喂钩子用的两棵树 `now` 4.2M、`base` 4.0M，第一轮电池用的 `r1leg` 684K，整条复跑的 `rerun1` 11M，真实语料的喂钩子输出 `run-real.txt` 33M，从会话记录抽出的 `real-commands.jsonl` 210M（含私有路径）。这些已删；草稿目录现 624K，只剩小文本输出与样本目录。真实语料那一步的数（30,595 行、1,893 条、1,262 条 sed 等）要重生：先 `python3 -I tools/extract-commands.py ~/.claude/projects/-home-fy5090-code-singlefs <输出> --since 2026-10-10`，再给 `rerun.sh` 设 `REAL_COMMANDS=<输出>`（整步约 30 分钟，超出前台上限，主 agent 自己起）。交回前 `ps` 核过：喂钩子、真 bash 对照、第一轮电池的进程一个不剩（输出 0 行）。
