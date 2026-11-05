# d137-gov-r3 攻方腿（Opus，格 R2）：已定撑不撑得住、未定清单、报出的数、零轮五件（2026-10-31）

开工快照 `research/prompts/d137-gov-r3-snapshot/files-sha256.txt` 17 份，开工时在主工作区 `sha256sum -c` 17 份 OK（量的）；冻结副本拷在草稿目录 `frozen/`，同样 17 份 OK。下文引被判文件写主工作区路径与行号，括号里是那份文件此刻 sha256 前 8 位；快照外的文件另写。

## 复跑命令与模型文件

```
bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/d137-gov-r3-opus-model/tools/rerun.sh --repo <仓根> --frozen <倒推出的开工快照树> --work <新的草稿目录>
```

rerun.sh 用仓根垫一棵树（`.claude` 去掉 kb、`scripts` 去掉 experiment-apparatus、各包清单），再把冻结副本盖上去，跑下表六个工具，只比以 `name=` 起头的行。我交回前在新 shell 里跑过一次（`--frozen` 给我自己的 `frozen/`）：rc=0，六项 `same=yes`。

| 文件 | sha256 |
|---|---|
| tools/rerun.sh | 87c05d5eac2b5485900346dd9945286d24d6e49a96542d40f2d30fc57aebaf35 |
| tools/count-switches.py | f3400595bb780401ecc8308bcea3e32fccf8b15dc3fcde5309cfa06496bd2eab |
| tools/lsw-coverage.py | f504aeff6271b054874f49a5f76c33b94911f9f0e92dd44900ccc21665d2be74 |
| tools/w2-parallel-close.py | b6d0d20d095b71f3422132508e901aed4041d3f11bd5c192cf1dd6ac76fa7aaa |
| tools/c1-boundary.py | 39081112234b351f07c3f499db8b48fee800fdb146e87bce7134dad455ad172e |
| tools/feed-dispatch-forms.py | 83334d1a249920108c0fada2584d874b41f33a47d1c751b856e892624c5687b8 |
| tools/alias-same-line-probe.sh | c5de494380adc9088bcac14788e18098b402d399e3b1c143e2cb329e025e0c6e |
| out/count-switches.txt | 8f67fb7b5cc2760a25e980f0606671b0b7e8f35ae96a8cf567b03da25851dfa5 |
| out/lsw-coverage.txt | 696946ef9dd8f914e19f8f2231f2f958364d5b99ba9100ba5c2ea9fc7811154d |
| out/w2-parallel-close.txt | 3a5824cbdf56a847688ff48756ffb8b623fc1df9feb040d9d2b787f0ad38326f |
| out/c1-boundary.txt | fae6d53e93093118e213d972e14ad313bdc56ac468314b6aa1efa02b8d9c8ff1 |
| out/feed-dispatch-forms.txt | d2caaae174dfd5e9efe7132beb3e4d2d541b05fd17d63dd55c4ce1f9e12500d6 |
| out/alias-same-line-probe.txt | c0ad33553666a4e1a0c2be66cce0e6aaee8016e1898c6a0b920d900b8beee102 |
| out/tool-timeout-probe-file.txt | b4668a66df0b920eba05c56ab998c375d1742279c86f4828f9893a8339dcb907 |
| out/tool-timeout-probe-tool-reply.txt | 86f3f616dab05b535b623d6de275976e46305e2848f38ee676d9fe0ccd00d14b |

`out/tool-timeout-probe-*.txt` 两份不进 rerun.sh（要的是 Bash 工具自己的超时行为，脚本里复现不了），是那一次探针的原样。

## 各格判定一览

| 名字 | 格 | 判 | 依据（量的 / 推的） |
|---|---|---|---|
| 并行行错收 | 已定 W2 | 打中：W2 修补撑不住。`declare-parallel` 不写写者，「此刻并行」收的是最近开的那一段；程序乙的「此刻并行」收掉程序甲的段，程序乙自己的「预计 7200 秒」那一段一直开着，替卡住的程序甲顶到第 7200 秒 | 量的：`out/w2-parallel-close.txt`，今天写法第 2400、4800、7000 秒都「声明解释得了」，对照（并行行带写者）同三刻都「解释不了」 |
| 跨 7200 那一窗口 | 已定 C1 | 边角，判错一窗口：拖过 7200 秒那一窗口里，同一行既写「声明解释得了」又写「拖过了它算数的 7200 秒」，照定义归第一种（只记），晚一个窗口（600 秒）才叫醒 | 量的：`out/c1-boundary.txt`，4.5 核、5.5 核第 7500 秒 `both_phrases=True`，第 8100 秒照叫醒 |
| verus 预计秒数不含排队 | 已定 M6 那一处 | 边角，偏叫醒那一侧：`min(7200, ⌈处数÷路数⌉×限时)` 只罩住跑的时间，包装限时「不算排队的时间」，排队改成不限时之后这个数不再是上界；后果是多叫醒，不是漏叫醒 | 推的（代码对读，没跑 Verus） |
| 派发闸退 2 一律拒、`sha256=refused` | 已定 H12、D1、D2 | 没打中 | 只读了代码与前两件报告，没喂派发闸；见「没打中的形状」 |
| 看门狗单元归属 U1、A1 上界 | 已定 | 没打中 | 只跑了上面两个模型，U1 没造多会话历史 |
| lsw 十四条开关无人复跑 | 已定 M3 | 打中：M3「三份钩子登进复跑表」的性质被改了。`lib_shell_words.py` 那一行删掉之后，它文件头 76 条开关里有 14 条不在任何一份登记脚本的文件头开关表里，提交前门禁一条都不复跑；其中有第一轮最重的 H1（`shell-option-clusters-unvalued`）与 H2、H3、H4、H6、H8 的开关 | 量的：`out/lsw-coverage.txt`，读法是从 `code-tooling.sh` 原样抽出的 `switch_entries` |
| 包装不在复跑表 | 已定 M3 同形 | 打中（漏登）：`run-with-memory-cap.sh` 登记成 light 却不在复跑表；件 11 的五条新开关与改了条数的四条旧开关「整份没重跑」，没登进收尾那一节 | 量的：复跑表 grep 0 行；调度记录第 178 行 |
| 检出钩子 118 条复跑范围 | 数 | 撑不住那句话：118 条里只有第 8 批 7 条在最终切词上复跑过；另 111 条是件 15、件 16 改共用切词之前跑的 | 量的：件 16 报告开关表；条数 118 用门禁读法数得出 |
| 2502 已过时 | 数 | 2502 是件 13 那一版重型闸的自证种数；件 16 之后是 2577，快照那一版又多了别的会话的格 | 量的：两份报告的原样行 |
| 161 已过时 | 数 | 161 是件 11 那一版包装的自证项数；快照那一版带别的会话加的「跑完查下一条」，那个会话量的是 163 | 量的：两份报告与调度记录 |
| legB-r2 30 与 38 | 数 | 正文「第二轮腿乙 30 条与登记不同、与上一趟逐条相同」只看了 compare 打出来的前 30 条；分类报告读头行是 38 | 量的：分类报告原样行 |
| 30,595 / 326 | 数 | 对得上，一条命令数得出 | 量的：`grep -c` 两条 |
| 592 秒 | 数 | 数对，成因只说对一半：最后 231 秒那一条自己就是排头、自己放不下，与回填无关 | 量的：`selftest-batch-1.out` 的排队行 |
| 前台超时转后台 | 零轮：共用约束第 62 行与第 75 行 | 打中（自相矛盾，比调度记录第 180 行登的更重）：Bash 工具的前台超时不杀命令，把它转成后台任务；「在队里等就是」的子 agent 一超时就成了第 75 行禁止的「要放后台等的命令」 | 量的：探针，进程活过 4 秒超时、12 秒后照写了一行 |
| 均衡句是先本机 | 零轮：主 agent 定义均衡句 | 打中（不是用户定的那件事）：`host-dispatch.py` 挑机器是「本机在前、第一台放得下的」，本机放得下就永远不去第二台；用户定的是「均分」「均衡的跑」 | 量的：代码行；推的：`--plan` 照抄「这条命令」时键带上了赋值词 |
| 声明闸不剥派发器 | 零轮：第 63 行与范围声明闸 | 打中（把别的规矩改坏）：照第 63 行经 `host-dispatch.py` 起 `cargo test -p … --test …`，不带范围声明范围声明闸照放；同一条走本机内存包装照拒 | 量的：`out/feed-dispatch-forms.txt` 样本 2、3 |
| 派发提示与第 63 行 | 零轮 | 打中（自相矛盾，轻）：第 63 行要本仓脚本单独一个 `--selftest` 经 `host-dispatch.py` 起、外面不再套包装；这一轮的正文与派发提示叫腿「自证与探针经 run-with-memory-cap.sh」 | 量的：两处原文 |
| 主 agent `--plan` 不带前缀被拒 | 零轮 | 边角：只查不跑的 `--plan` 不带 `SINGLEFS_HEAVY_TESTS` 被重型闸拒 | 量的：样本 7 |
| 闸剥开两种起法的选项表 | 零轮 | 没打中：两份派发器的选项与值的个数和剥法逐个对上 | 代码对读 |
| E4 同一行别名 | 零轮：误拒修补 | 没打中：在 Bash 工具同样的 `eval` 包法下照样不展开 | 量的：`out/alias-same-line-probe.txt` |
| 排队判法的并发、同一秒、死条 | 零轮 | 没打中 | 代码对读，见第四节 |

## 一、已定撑不撑得住

### 并行行错收（W2，第二轮判决「同一份声明文件里几个程序的声明互相截断、互相续命」）

修补的读法写在 `scripts/hosts/core_usage.py:460`：「串行行结束同一个写者开着的那一段；并行行结束带的写者那一段，没带写者的结束最近开的那一段」（a7d613f8）。写者只在串行行上写：`scripts/hosts/core_usage.py:840`：「line += f"\t{WRITER_MARK}{os.getppid()}"」；`declare-parallel` 那一支只写时刻与标记：`scripts/hosts/core_usage.py:844`：「line = f"{time.time():.3f}\t{PARALLEL_MARK}"」。verus 自己写声明文件，串行、并行两种都不带写者：`scripts/gate/verus-proof-check.py:3339`：「handle.write(f"{time.time():.3f}\t{body}\n")」（608296e2）。所以今天写出来的每一条「此刻并行」都走「没带写者的结束最近开的那一段」。

造的历史（`tools/w2-parallel-close.py`，只调 `parse_declarations`、`judge_series`、`verdict_lines`，整棵树一直 0.2 核、登记 31）：
- 第 0 秒程序乙（写者 222）写「此刻串行（预计 7200 秒）」；
- 第 1000 秒程序甲（写者 111）写「此刻串行：…」，没写预计秒数（算一个窗口 600 秒）；
- 第 1100 秒程序乙写「此刻并行」（照 `declare-parallel` 的写法，不带写者）；之后程序甲卡住不收。

原样输出（`out/w2-parallel-close.txt`，摘 `name=` 行）：

```
name=w2_parallel_close case=此刻并行不带写者（declare-parallel 今天的写法） now=2400 status=low declared_in_window=600 excused=True reasons=['程序乙的计时窗口必须单线程']
name=w2_parallel_close case=此刻并行不带写者（declare-parallel 今天的写法） now=4800 status=low declared_in_window=600 excused=True reasons=['程序乙的计时窗口必须单线程']
name=w2_parallel_close case=此刻并行不带写者（declare-parallel 今天的写法） now=7000 status=low declared_in_window=700 excused=True reasons=['程序乙的计时窗口必须单线程']
name=w2_parallel_close case=此刻并行不带写者（declare-parallel 今天的写法） now=7500 status=low declared_in_window=300 excused=False reasons=['程序乙的计时窗口必须单线程']
name=w2_parallel_close case=对照：此刻并行带写者 222 now=2400 status=low declared_in_window=0 excused=False reasons=[]
name=w2_parallel_close case=对照：此刻并行带写者 222 now=4800 status=low declared_in_window=0 excused=False reasons=[]
name=w2_parallel_close case=对照：此刻并行带写者 222 now=7000 status=low declared_in_window=0 excused=False reasons=[]
name=w2_parallel_close case=对照：此刻并行带写者 222 now=7500 status=low declared_in_window=0 excused=False reasons=[]
```

今天的写法下，程序甲第 1600 秒起就该过期，看门狗却到第 7200 秒之后才叫醒，晚了约 5600 秒；理由一栏写的是程序乙的理由。对照组（同一段历史，只给并行行补上写者）第 2400 秒就叫醒，那一行点出程序甲「拖过了它算数的 600 秒」。

四句：
- 分不分辨臂：分辨。同一段历史只差并行行带不带写者，判定翻面。
- 系统看不看得到判别它的东西：看不到。`declare-parallel` 写下的行里没有写者，`declaration_segments` 在判的那一刻分不出这一行是谁写的；病根在写声明那一侧，不在判法。
- 满足字面哪一句：`.claude/main-agent.md:71`：「同一份声明文件里几个程序的声明按写者分段，一个程序拖过它算数的秒数，别的程序的声明不替它续命」（9e88ebfb）——这里程序乙的声明替程序甲续了命，命中的是这一句的后半。
- 改法在打中格上还中不中：改法「`declare-parallel` 也写 `\t写者进程 <进程号>`，verus 的 `declare_serial_section` 两种行都写写者」在这几格上不中（对照组就是它，量过：只在我的模型上量过、被攻过零轮）。只改判法、不改写法的（例：没带写者的并行行一段都不收）推的会把只写并行行收尾的旧脚本的段一直留到算数秒数，偏叫醒，没量。

推翻条件：仓里没有哪一棵树里有两个程序写同一份声明文件（W2 的前提本身不成立），或 `declare-parallel` 在别处另有带写者的写法。我只查了 `core_usage.py` 与 `verus-proof-check.py` 两处写声明的代码。

### 跨 7200 那一窗口（C1）

`.claude/main-agent.md:72`：「写了「声明解释得了」」那一种只记不叫醒；`.claude/main-agent.md:74`：「「声明解释得了」「拖过了它算数的 7200 秒」都没写」才照出路停。一行同时写两样时归第一种。造的历史（`tools/c1-boundary.py`）：程序甲第 0 秒写「此刻串行（预计 7200 秒）」、从不收，整棵树 6 条线程、r 核，判第 7500、8100 秒两个窗口：

```
name=c1_boundary rate=3.5 now=7500 status=low excused=False both_phrases=False
name=c1_boundary rate=3.5 now=8100 status=low excused=False both_phrases=False
name=c1_boundary rate=4.5 now=7500 status=low excused=True both_phrases=True
name=c1_boundary rate=4.5 now=8100 status=low excused=False both_phrases=False
name=c1_boundary rate=5.5 now=7500 status=low excused=True both_phrases=True
name=c1_boundary rate=5.5 now=8100 status=low excused=False both_phrases=False
```

第 7500 秒那一窗口里声明还算 300 秒、另 300 秒过期被扣掉，按「声明段占一个核」算解释得了，同一行又点出拖过 7200 秒：归只记那一种。下一窗口照叫醒。用户定的「超过 7200叫醒 检查」（调度记录第 134 行）在这一种读数下晚一个窗口。判错只有一窗口，偏迟不偏漏。改法（推的、没量）：第二种的条件改成「写了拖过 7200 秒」不论有没有「解释得了」，第一种只收没写拖过的。

### verus 预计秒数不含排队（M6 那一处）

`scripts/gate/verus-proof-check.py:3355`：「expected_seconds = min(7200, -(-len(edits) // jobs) * options["timeout"])」。每处 Verus 经内存包装起、限时交给包装；包装的限时 `scripts/hosts/run-with-memory-cap.sh:323`：「从起跑算、不算排队的时间」（1e85c49c）。件 11 之后排队不设上限，所以 ⌈处数÷路数⌉×限时 只是跑的时间的上界，加上排队就不是了。后果推的：声明在排队拖长时提前过期 → 多叫醒一次「拖过了它算数的 N 秒」，主 agent 查进度、在走就照跑；看门狗另有「排内存队的只记」那一条，排着的那一刻不叫。偏叫醒，不偏漏。没跑 Verus，没量。

### lsw 十四条开关无人复跑（M3 的性质被改）

第二轮判决 M3 点名三份钩子（`heavy-test-guard.sh`、`lib_shell_words.py`、`bash-command-detector.sh`）不在复跑表。登记件交回后主 agent 删了 `lib_shell_words.py` 那一行，理由写在调度记录 `records/2026-10-29-一百三十七节那一轮调度记录.md:174`：「主 agent 判删掉那一行（它的开关由导入它的几份自证各自带着跑，文件头第 102–116 行）」（27936bdf）。导入它的自证确实会红，但复跑门禁只复跑登记脚本文件头开关表里列出的那几条（`code-tooling.sh` 的 `switch_entries` 读法）。我用同一份读法数（`tools/lsw-coverage.py`，原样抽 `.claude/gate.d/code-tooling.sh` 里那段库）：

```
name=lsw_coverage registered_rows=60 lib_shell_words_switches=76 listed_in_some_registered_header=62 not_listed=14
```

没被任何登记脚本列出的 14 条（`out/lsw-coverage.txt` 原样）：aliases-unexpanded、brace-sequence-width-padded-ends-only、conditional-expression-split、coproc-as-command、env-ignore-environment-kept、extended-assignments-as-command、quoted-braces-as-unknown、renamed-shells-unseen、shell-option-clusters-unvalued、shell-standard-input-file-unread、standard-input-device-script-unread、substitution-case-and-comment-unseen、tilde-plus-minus-unexpanded、trap-action-unread。其中 `shell-option-clusters-unvalued` 是第一轮判决 H1（标「最重」）的开关，`.claude/hooks/lib_shell_words.py:347`（d33ec04b）写着它打开后重型闸与 lib_heavy_tests 两份自证各红几格，可这两份的文件头开关表里没有这一条。`trap-action-unread`、`aliases-unexpanded`、`renamed-shells-unseen` 在 `heavy-test-guard.sh` 里只出现在第 3498–3529 行的代码注释里，门禁读不到。M3 原意是「提交前门禁替这几份整表复跑」，删行之后这 14 条没有任何门禁复跑，调度记录也没登。

四句：分辨（登记与否决定门禁复不复跑这 14 条）；系统看得到（门禁读的就是文件头表）；满足的是第二轮判决 M3 那一句「提交前门禁不会替它们整表复跑」，换了一份文件照样成立；改法「把这 14 条补进导入它的某一份登记脚本的文件头表（点名它们在那份自证里红的格）」或「给 `lib_shell_words.py` 一份能被门禁起的入口」，两样都推的、没量。

推翻条件：门禁 break-switch-replay 另有一处读 `lib_shell_words.py` 文件头、逐条设开关跑导入它的自证（我只读了 `switch_entries` 与登记表，没读完那一格的全部代码）。

### 没打中的已定格（H12、D1、D2、U1、A1 上界、M4）

只读代码与前两件交回报告，没喂派发闸、没造多会话的看门狗历史。H12「退 2 一律拒」是用户定的口径，判的是写法与口径一致，造不出与口径相反的格；D1 的「退 0 而产物头 `sha256=refused`」与 D2 的清 `SINGLEFS_EXPERIMENT_RERUN_REASON` 我没构造出边角。M4（主工作区重核开关）被下面第三节「检出钩子 118 条复跑范围」部分顶掉：重核之后共用切词又改了两次。

## 二、未定清单完不完整

调度记录里还没有一节叫「收尾」；登进收尾、留用户、下一轮的条目只散在进度行里。我按关键词 grep 一遍（`grep -n '收尾那一节\|收尾时登账\|登进账本\|留用户亲自处理\|留你亲自处理\|下一轮\|归提交前\|延后到提交前\|还没复跑\|没复跑\|整份没重跑' records/2026-10-29-一百三十七节那一轮调度记录.md`），命中第 22、53、107、131、142、154、156、166、174、177、178、180、183、184、185、190 行（量的）。

### 已登的（照原文没被改性质的）

| 记录行 | 登的是什么 | 去向 | 性质变没变 |
|---|---|---|---|
| 154、166、180 | 子 agent 前台 timeout 600000 毫秒（检出 ③ 改不改成拒）与「在队里等」怎么并 | 收尾那一节、交用户 | 变了，见下面「前台超时转后台」 |
| 156、184 | Y014、Y029、Z021、两处变量口子；子 shell 与管道里的赋值不分作用域 | 留用户亲自处理 | 没变 |
| 174 | 检出钩子文件头 20 行短写法开关、重型闸 `cat-concatenation-unseen` 没复跑 | 收尾那一节 | 没变 |
| 177 | 看门狗把包装的盯账、采峰值两个辅助进程当成「进程无输出」 | 收尾那一节 | 没变 |
| 178 | C631（每一轮各等满 3600 秒再退 252）前提变了，三语 kb 跟着改 | 收尾写回那一批 | 没变 |
| 183 | 派发提示把出处写进「要改的文件」一行被闸当成要改的文件 | 收尾那一节 | 没变 |
| 185 | 重型闸文件头 18 条散文写法的开关点名与实红对不上 | 收尾那一节 | 没变 |
| 190 | E4 落在检出钩子的四处没换；`unrecognized-launches-allowed` 下重型闸两格不再红（推的是别的会话那一道先拒） | 收尾那一节 | 没变 |
| 53、107、131、142 | 广谱变异、agent-watch、detached-run、lib_heavy_tests 整表复跑归提交前 | 提交前 break-switch-replay 格 | 没变（但见下一表 lsw 一行：这一格读不到的不在「整表」里） |

### 漏登的

| 名字 | 出处 | 为什么算漏 |
|---|---|---|
| lsw 十四条开关无人复跑 | `records/2026-10-29-一百三十七节那一轮调度记录.md:174`：「主 agent 判删掉那一行」 | 删行的理由（导入它的自证各自带着跑）只对一半：门禁只复跑登记文件头列出的，14 条不在任何一份里（第一节）。第二轮 M3 被悄悄改成「两份钩子登记」 |
| 包装不在复跑表 | `records/2026-10-29-一百三十七节那一轮调度记录.md:178`：「旧开关 nocap、noslice、noledger、nodefault 的条数照新加的格改了，整份没重跑」；复跑表里 `run-with-memory-cap.sh` 0 行，`.claude/hooks/selftest-registry.tsv:135` 登记它是 light | 件 11 的五条新开关只在件 11 自己跑过，四条旧开关的新条数没跑过；提交前门禁不替它复跑，记录也没登。别的会话之后又加了两条（nextunchecked、nextalwaysreminds），它自己的记录写「还欠」 |
| 检出钩子 118 条复跑范围 | 复跑表第 68 行与件 16 报告 | 见第三节；复跑表那一行写「逐个跑过」，没登「111 条跑在最终切词之前」 |
| 第二轮 M5 的前提 | 第二轮判决 M5：「主 agent 在第三轮开工前用最终钩子跑四份全量复跑」 | 第三轮开工时最终钩子上的复跑没跑（正文第二节写「要等统一任务启动器」），从开工前提改成判决后补；正文写了现状，没有一条把 M5 记成没兑现 |
| 声明闸不剥派发器 | 第四节 | 件 13 只改了重型闸；范围声明闸 `.claude/hooks/run-scope-guard.sh`（b228eefd）里 `host-dispatch` 出现 0 次（`grep -c`，量的），第 63 行写的「范围声明照旧写在最前面」在这条路上没人管 |
| 并行行错收 | 第一节 | W2 修补没碰写声明那一侧，没登 |
| 复跑表 agent-watch 那一行的条数 | 复跑表第 66 行写「一百三十七条开关（文件头现数…）」 | 门禁读法数出 146 条（HEAD 那一版 125 条，用 `git show HEAD:scripts/agents/agent-watch.py` 取出来照同一个工具数，取出的那一份交回前已删；量的）；多出来的 9 条里有别的会话加的，条数说明已不对。门禁按表头逐条复跑，不按说明里的数，所以只是说明过时 |
| 前台超时转后台 | 第四节 | 第 180 行登的是「一直撞」，实测是转后台，后果不同（第 75 行与交回闸都牵进来） |

### 采为线索而没现喂的（前两轮）

第一轮的 H4 其余、H5、H6、H7、H9、H10、H11、H12、H13、S2、S3、S5、A4，第二轮的 Z021、B、D、E、G、M2、D2 后一半、A4：按调度记录第 128–143、153–157 行，各自由修补件复现后改或写明（H10 第一次撤回、第二次落地；A4 采为潜在；Z021 归用户）。没找到哪一条既没复现也没登的。第二轮 M1（看门狗修补核开关只比失败行前 48 个字）判「整表复跑归提交前门禁」，我没核门禁那一格比不比全行，算没核。

## 三、报出的数能不能核

| 数 | 出处 | 一条命令 | 数出 | 对不对得上 | 量的范围撑不撑那句话 |
|---|---|---|---|---|---|
| 重型闸开关 79、lib_heavy_tests 74、检出钩子 118、core_usage 22 | 复跑表第 62、67、68、69 行 | `python3 -I research/prompts/d137-gov-r3-opus-model/tools/count-switches.py <树> <路径>…`（抽 `code-tooling.sh` 的 `switch_entries` 原样执行） | 79、74、118、22（`out/count-switches.txt`） | 对得上 | 79：76 条复跑过、3 条没复跑，表里写明了，撑得住。74：72 加 2，写明了，撑得住。118：撑不住「逐个跑过」，见下。22：件 registry-2 在 2026-10-30 跑过，之后 core_usage 没进这一轮别的修补，撑得住（推的：没比当时那一份的 sha256） |
| 检出钩子 118 条「逐个跑过」 | 复跑表第 68 行 | 件 16 报告「开关」表一行原样 | 只有件 15 的 7 条在件 16 换上之后跑过；其余 111 条是登记续做在件 15 之前跑的 | 条数对，「逐个跑过」的时点不对 | 撑不住：件 15、件 16 都改了 `lib_shell_words.py`（检出钩子导入它），111 条没在最终切词上重跑。件 16 只另核了两条旧开关（extra-launchers-unseen、unrecognized-launches-allowed） |
| 重型闸「查了 2502 种」 | 件 13 报告第 98 行 | 读报告：`grep -nF '查了 2502 种'` | 件 13 那一版 2502；件 16 那一版 2577（件 16 报告第 111 行：「✓ 自检通过（查了 2577 种，其中该拒 552」） | 对那一版对得上 | 撑不住「被判的那一版」：快照里的重型闸在件 16 之后，又有别的会话加的「起编译不经内存包装」那一道与三条开关，此刻种数没人数过。要数只能跑一次自证（登记 light），静态数不出 |
| 包装自证 161 项 | 件 11 报告第 9 行 | 读报告 | 161 是件 11 那一版；快照那一版带别的会话的「跑完查下一条」，那个会话串着单跑报「查了 163 项」（`records/2026-10-19-里程碑四GPU垃圾回收开工.md:627`） | 对件 11 那一版对得上 | 撑不住「快照那一版 161 项」；件 11 的 own.diff 打回件 11 的 base 之后与此刻比，还差 415 行 diff（量的；那份 diff 交回前已删，重做：把 `d137-queue-order/own.diff` 前 581 行打回它 `base/` 里的那一份包装，再与此刻的包装 `diff -u`） |
| crates-mutation-rows 187 格、dispatch-prompt 119 项 | 件 11、件 12 报告 | 读报告 | 187、119 | 对得上那一趟 | 没核此刻那两份与那一趟是不是同一份：快照 sha256 有，交回时的没留 |
| 复跑差 294 | 分类报告总表 | 加法 | 181+84+4+25=294 | 对得上 | 撑得住（分类按头行 `differing=` 读，不受 compare 只打 30 条影响） |
| 复跑差 38 与正文「30 条」 | 分类报告第 29 行；正文第二节 | 读 legB-r2 头行 | 38（33+2+3） | 分类与正文不一致 | 正文 `research/prompts/_d137-gov-r3-body.md@二、实现今天的样子（主 agent 的观测，2026-10-31）`：「第二轮腿乙 30 条与登记不同、与上一趟逐条相同」——30 是 compare 只打前 30 条的数，「与上一趟逐条相同」比的也只是两趟各自打出的前 30 条；另 8 条（分类报告点名 o.O30-pkill 等）两趟比没比过不知道 |
| 语料 30,595 / 新拒 326 | 正文第二节 | `grep -c '^now:' d137-r3-prep/real-work/out/run-real.txt`；`grep -c '^now:rc=2.*base:rc=0' 同一份` | 30595、326 | 对得上 | 撑得住，范围是件 13、15、16 之前的钩子（正文写明了）。同目录另有 `run-cd.txt` 29、`run-everyday-1.txt` 18、`run-everyday-2.txt` 44 条新拒（同一条 grep，量的），正文与分类都没提 |
| 随机样本 5000 里 16 | 正文第二节 | 没找到能一条命令数的产物 | — | 没核 | — |
| 排队 592 秒 | 件 11 报告第 165 行 | `grep -n 'run-with-memory-cap' d137-queue-order/selftest-batch-1.out` | 592 | 对得上 | 成因只说对一半，见下 |

592 秒那一条的排队行原样（每行截到「这一条要 4.0 GiB」为止，中间括注省去）：

```
排队（已等 0 秒，不设上限）：第 5 位 / 共 5 条，排头要 8.0 GiB；slice 已占 32.1 GiB，这一条要 4.0 GiB
排队（已等 60 秒，不设上限）：第 4 位 / 共 5 条，排头要 4.0 GiB；slice 已占 36.2 GiB，这一条要 4.0 GiB
排队（已等 121 秒，不设上限）：第 3 位 / 共 4 条，排头要 8.0 GiB；slice 已占 32.2 GiB，这一条要 4.0 GiB
排队（已等 301 秒，不设上限）：第 3 位 / 共 6 条，排头要 8.0 GiB；slice 已占 32.2 GiB，这一条要 4.0 GiB
排队（已等 361 秒，不设上限）：第 1 位 / 共 4 条，排头要 4.0 GiB；slice 已占 36.2 GiB，这一条要 4.0 GiB
排队（已等 543 秒，不设上限）：第 1 位 / 共 5 条，排头要 4.0 GiB；slice 已占 36.2 GiB，这一条要 4.0 GiB
排了 592 秒，起跑
```

件 11 报告与调度记录第 178 行把 592 秒说成「排头要 8 GiB、slice 已占 32 GiB 时，后面 49 MiB 与 4 GiB 的都回填不进」。按行读：第 0、121–301 秒那几段排头要 8 GiB、已占 32 GiB，旧判法（谁放得下谁进）下这一条 32.2+4.0=36.2 ≤ 40.1 会当场起，这一段确是先来后到带保守回填的代价；第 361 秒起它自己是排头（「第 1 位」，报行的「排头要」此时是它自己），已占 36.2 + 4.0 = 40.2 > 40.1，旧判法也不放，这最后至少 182 秒（361→543，报行间隔 60 秒，到 592 可能是 231 秒）与回填无关。「592 秒是用户定的式子的代价」要改成「其中第 0 秒到第 361 秒之间那一段是」（推的：旧判法在那一刻放进去之后后面的走法会变，这只是第一步的反事实）。

## 四、零轮五件

### 排队判法（件 11）：并发、同一秒、包装被 KILL、slice 读数抖

没打中。对读的几处（`scripts/hosts/run-with-memory-cap.sh`，1e85c49c）：
- 判与登记在同一把锁里：`scripts/hosts/run-with-memory-cap.sh:1463`：「fcntl.flock(handle, fcntl.LOCK_EX)」；登记在锁里：`scripts/hosts/run-with-memory-cap.sh:2144`：「own_queue_entry = register_in_queue(unit, wrapper_pid, wrapper_start_time, need, key, queue, lookup.command)」。同一秒到的两条排成先后，序号 `scripts/hosts/run-with-memory-cap.sh:1514`：「"arrival_sequence": max((queue_entry_sequence(other) for other in queue), default=0) + 1,」，不看时钟。
- 包装被 KILL：排队那一步每轮先看父进程（`scripts/hosts/run-with-memory-cap.sh:2088`：「if os.getppid() != wrapper_pid:」），它自己退；登记里记的是包装的进程号与起始时刻，别人下一次进锁按死条删，进程号被复用也认得出。
- 保守回填：`scripts/hosts/run-with-memory-cap.sh:1538`：「return occupied + need + queue_entry_need(head) <= total_bytes」。放进去之后排头在同一份读数下仍放得下；读数往下抖时排头与后面的同样被放大，往上抖时谁都不进。没造出排头被回填一直挤着的历史（排头不放得下时回填条件必然不成立，`need > 0`）。
- 我试过的反向：排头的包装活着却不再重判（被 STOP、冻结的 cgroup）时，后面放不进回填的一律跟着等，而等待不再设上限。这要外力停住包装，不是包装自己会走到的状态，没算打中。

### 前台超时转后台（共用约束第 62 行与第 75 行）

`.claude/agent-common.md:62`：「它一直等、先来后到，在队里等就是」（a2ad83ba）；`.claude/agent-common.md:75`：「子 agent 不起要放后台等的命令」与「前台命令的 `timeout` 不超过 240000 毫秒」。调度记录第 180 行登的是两句「一直撞」。我量了 Bash 工具超时时做什么：前台起 `python3`，睡 12 秒后往文件追加一行，`timeout` 给 4000 毫秒。工具回复原样（`out/tool-timeout-probe-tool-reply.txt`）：「Command did not complete within its 4s timeout and was moved to the background」；文件原样（`out/tool-timeout-probe-file.txt`）两行，第二行「late: survived the tool timeout」，随后任务通知 exit code 0。

所以子 agent 照第 62 行在前台排内存队、排过 timeout 时，命令不会被杀，而是自己变成一条后台任务：正是第 75 行不许子 agent 起的那种；交回闸又拒「全部子 agent 自己起的后台任务还有进程开着输出文件」（`.claude/agent-common.md:104`），出路是「结束本轮等完成通知再交回，或逐个停掉再交回」，而第 62 行不许「停掉排队的那一条换别的起法」。一个子 agent 照这两句走，只剩结束本轮等通知这一条路，与第 75 行「长活交回主 agent」相反。调度记录第 180 行的登法（「会一直撞」「排队多久算长活」）没写这一层，交用户时要连这条实测一起交。

四句：分辨（工具杀不杀命令决定两句能不能同时守）；系统看得到（工具回复里明写 moved to the background）；满足的是第 75 行「子 agent 不起要放后台等的命令」——这里不是子 agent 起的，是工具转的，字面上没违反、实质一样；改法推的：第 62 行对子 agent 改成「排不上就交回主 agent，命令写进要主 agent 跑的命令」，或第 75 行写明「工具超时转成的后台任务照后台任务办：结束本轮等通知」，两样都没量。

### 均衡句是先本机（主 agent 定义第 53、104 行）

用户原话 `records/2026-10-29-一百三十七节那一轮调度记录.md:172`：「我定的是均分。 那边有资源取那边 不是必须要去那边。两台机器环境基本一致 因此均衡的跑比较好」。定义写成 `.claude/main-agent.md:53`：「两台按资源均衡着跑：每条命令起之前先跑 `python3 scripts/hosts/host-dispatch.py --plan -- <这条命令>`，它挑第二台就加 `--peer`」。`--plan` 挑机器的规矩：`scripts/hosts/host-dispatch.py:51`：「④ 挑一台：本机在前、第二台在后，挑第一台「预测量 + 余量放得下」的」（11675290）；代码 `scripts/hosts/host-dispatch.py:767`：「for host in (["peer", "local"] if broken("peer-first") else ["local", "peer"]):」与 `scripts/hosts/host-dispatch.py:773`：「chosen = next((verdict for verdict in verdicts if verdict.fits), None)」。本机放得下就挑本机，第二台只在本机放不下时才轮到：这是「本机优先、溢出到第二台」，不是「均分」。今天本机 slice 余量常有几十 GiB（我开工时 `--status`：总上限 40.1 GiB、回收不掉的 3.0 GiB，量的），照这句跑，主 agent 的重活几乎全落本机。

另一处推的：第 53 行的「这条命令」开头就是 `SINGLEFS_RUN_SCOPE='…' SINGLEFS_HEAVY_TESTS=user-request bash scripts/gate/merge-and-verify.sh …`。照抄进 `--plan -- <这条命令>`，`--plan` 算预测用的键是整串词连起来（`scripts/hosts/memory_peaks_lib.py:107`：「joined = TEST_BINARY_HASH.sub("", " ".join(words))」（f54718be），不剥赋值词），峰值表里不会有这把键，预测落到默认 8G；合入后验证本身也不整条经内存包装、峰值表里本来就没有它。`--plan` 的读数因此和这条命令真要多少无关（没跑 `--plan`，推的）。

四句：分辨（先本机与均分在本机有余量时挑法相反）；系统看得到（`--plan` 两台的余量都现查）；满足的是用户原话里「均分」「均衡的跑」那一句，「那边有资源取那边」若读成「第二台有资源就去」也与先本机相反；改法推的：`--plan` 加一种挑法（两台都放得下时挑在用的少的、或轮流），定义照它写。没量。

### 派发提示与第 63 行

`.claude/agent-common.md:63`：「外面不再套 `run-with-memory-cap.sh`，派发提示给的内存上限这时不用」，管的是「本仓脚本单独一个 `--selftest`」这类只读仓树的命令。这一轮的正文 `research/prompts/_d137-gov-r3-body.md@五、交付要求`：「自证与探针经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 4G 的上限」，派发提示同句。写新句的人在同一轮的派发里没照它。轻：探针多半读草稿目录，归第 64 行；自证归第 63 行。我照派发提示办（探针都读草稿目录），没跑自证。

### 重型测试闸剥开两种起法（件 13）

选项表 `.claude/hooks/lib_heavy_tests.py:1266-1267`（4ab8f57d）与两份派发器切参数的代码逐个对上：`host-dispatch.py` 的 `--gpu`、`--plan` 不带值，`--label`、`--fetch`、`--shard`、`--estimated-hours` 各带一个（`scripts/hosts/host-dispatch.py:972`：「elif word in ("--label", "--fetch", "--shard", "--estimated-hours"):」）；`multi-host-run.sh` 的 `--send` 吃两个值（`scripts/hosts/multi-host-run.sh:1185`：「--send) (( $# >= 3 ))」（e6058a94））。剥出来的那条算经内存包装，两份派发器确实都经包装起（`host-dispatch.py` 文件头 ⑤，`multi-host-run.sh` 不带 `--peer` 时缺省上限照旧走本机包装）。剥法没打中。

剥法之外，两处与别的规矩不齐，喂冻结副本的钩子（`tools/feed-dispatch-forms.py`，只喂 JSON；原样 `out/feed-dispatch-forms.txt`）：

```
name=dispatch_form case=2 hook=run-scope-guard.sh agent=three-way-attack rc=0 wanted=拒（target 声明缺） label=同一条不带声明
name=dispatch_form case=3 hook=run-scope-guard.sh agent=three-way-attack rc=2 wanted=拒（target 声明缺） label=对照：本机经内存包装、不带声明
name=dispatch_form case=7 hook=heavy-test-guard.sh agent=主 agent rc=2 wanted=放（只查不跑） label=主 agent：--plan、什么都不带
```

样本 2 是第 63 行例子去掉范围声明（`python3 scripts/hosts/host-dispatch.py --label probe-b -- bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-build-directory.sh cargo test -p singlefs-harness --test overwrite_in_one_instance`）：范围声明闸放。样本 3 同一条换成本机内存包装：拒。范围声明闸里 `host-dispatch` 出现 0 次。这不是我造的写法，是第 63 行指定的那条路本身；用户口径「不攻钩子漏拒的写法」管不管这一条由主 agent 定，我没在它上面再变写法。样本 7：主 agent 只查不跑的 `--plan` 不带 `SINGLEFS_HEAVY_TESTS` 被重型闸拒（剥开照判是件 13 有意的严）；带上前缀（样本 5、6）两道都放。第 53 行没写 `--plan` 要带前缀。

### 两件误拒修补（件 15、件 16）：放开的写法里有没有真起重型的

只用分类报告 `d137-replay-classify/result.md` 里已有的同形写法，不造新的。E4 我用真 shell 量了（`tools/alias-same-line-probe.sh`，样本把 cargo 换成 echo，每条 `setsid --wait`、10 秒杀整组）。Bash 工具真实的起法我先现查了本会话那一层 shell 的命令行：`bash -c "source <快照> … && eval '<命令>' && pwd -P …"`，快照里有 `shopt -s expand_aliases`。原样（`out/alias-same-line-probe.txt`）：

```
name=alias_same_line case=plain-bash-c-same-line rc=127 expanded=no
name=alias_same_line case=plain-bash-c-eval-same-line rc=127 expanded=no
name=alias_same_line case=tool-wrapper-expand-aliases-state rc=0 expanded=no
name=alias_same_line case=tool-wrapper-eval-same-line-with-shopt rc=127 expanded=no
name=alias_same_line case=tool-wrapper-eval-same-line-without-shopt rc=127 expanded=no
name=alias_same_line bash_version=5.2.21(1)-release
```

第三行的输出是 `expand_aliases 	on`：工具那一层的 shell 本来就开着别名展开。同一行先 alias 再用，在 `eval` 包法下照样不展开（rc=127，`ct: command not found`），E4 放开站得住。工具 shell 默认开着 `expand_aliases` 这件事只记在这里，换行写法归用户口径「不攻漏拒的写法」，我没喂、没试。

其余几类按分类报告那一行的同形样本对读（没跑真 shell）：E1（`print("$D")`，D 是字面目录）、E6（`du -sch` 接字面列表）、E7（只 `compile` 不 `exec`）、E8（sed 标志区有 GNU sed 不认的字、整个脚本不跑）、E9（`cd 不在的目录 && ( bash x.sh )`，cd 失败不跑）、E3（cat 写出的轻脚本再 `bash $t`）、E5（`awk -f` 字面目录下的程序文件）、E2（`systemd-run --user --wait` 等包着一行 `echo hi` 的轻脚本）：同形样本起的都是 du、print、awk、echo 一类，没有一样起重型。这几类放开的边界（变量里有命令替换、heredoc 正文里有别的展开）要造新写法才试得到，按口径不试。

## 没打中的形状

| 格 | 试过的形状 | 取样范围 |
|---|---|---|
| 排队判法 | 同一秒到达、包装被 KILL、进程号复用、slice 读数上下抖、回填挤排头 | 代码对读 2060–2180、1455–1540 行一带；没起真的包装排队（要真 slice 与几条并发包装，挂钟与别的会话的活抢） |
| 派发闸 H12、D1、D2 | 退 2 的几种来路、`sha256=refused`、准入超时 | 只读第二轮修补件报告与正文，没喂派发闸 |
| 看门狗 U1、A1 上界 | — | 没造；只跑了 W2 与 C1 两个模型 |
| 闸剥开两种起法的选项 | 两份派发器每个选项与值的个数、`--plan`、`--selftest`、别处的同名拷贝 | 代码对读加喂 7 条 |
| E4 | 四种起法 | 真 shell 5 格 |
| 别的误拒修补 | 只对读同形样本 | 8 类各 1 条 |

「没打中」只有这一次取样，照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」，不能拿来支撑「撑住」。

## 这条腿自己的限度

- 没跑任何自证：用户经别的会话转来「新的脚本任务（含自证）先不起、等统一任务启动器」，正文第二节也照它等；派发提示叫自证与探针经包装跑，我只跑了读草稿目录的探针（模型、喂钩子、真 shell 样本，各经 `run-with-memory-cap.sh` 1G–2G），自证种数只能读报告，所以 2502、161、187、119 都只核到「是哪一版的数」。
- 喂钩子用的是冻结副本里的钩子，cwd 是冻结副本根（在草稿目录下）。冻结副本缺 `.claude/kb`、`gate.d/fixtures`、`scripts/experiment-apparatus`（只拷了 `replay.sh`）与 `.git`，钩子判「主工作区」那一道在副本里与主工作区可能不同；我喂的 7 条都不碰那一道。
- W2、C1 两个模型只调 `core_usage.py` 的函数、造的样本序列，不是看门狗读真进程树的整条路；看门狗读的是同一份 `declaration_excuses_low`（第二轮修补写明统一用它），中间的取样没走。
- 主 agent 定义均衡句那一条，「本机常有余量」是我开工那一刻 `--status` 的一个读数，不是长期分布。
- 第二轮 M1 门禁比不比全行没核。

## 推翻条件

- 并行行错收：仓里没有两个程序写同一份声明文件的树；或 `declare-parallel` 另有带写者的写法。
- lsw 十四条：break-switch-replay 格另有逐条设 `LIB_SHELL_WORDS_BREAK` 跑导入它的自证的那一段（我只读了 `switch_entries` 与登记表）。
- 均衡句：用户的「均分」本意就是「本机优先、溢出才去第二台」（那样定义与代码都对）。
- 前台超时转后台：子 agent 的工具在超时时杀命令（我量的是本会话这一版工具；子 agent 若是另一版工具会不同）。
- 声明闸不剥派发器：范围声明另由别处（派发器自己、看门狗）查；我只 grep 了 `run-scope-guard.sh`。
- 118 条：件 16 之后有人在最终切词上整表跑过检出钩子的开关、只是没写进报告。

## 哪一条最脆弱

均衡句是先本机。它是这一轮用户亲口定的事，定义写的是「均衡」，执行它的唯一工具 `--plan` 照代码是先本机，本机有余量就永远不去第二台；加上「这条命令」照抄进 `--plan` 时键对不上、预测落默认值，`--plan` 的读数与这条命令真要的量也没关系。它不靠边角输入，日常每一次都这样走。其次是前台超时转后台：子 agent 每一次照第 62 行在前台排队超过 timeout 都会撞上。

## 没做什么

- 没跑重型测试、没起 cargo、没碰 GPU、没跑任何 `--selftest` 与门禁阶段。
- 没造新的绕过写法，没复现 Y014、Y029、Z021 与变量拼命令词那两处；E4 的换行写法没试。
- 没在主工作区改任何文件；冻结副本、`rerun-1/`、`head/`、`qo/` 交回前已删（大小见交回）。
- 不判辩方那一格；副本上的读数不算入库装置上的数；自己提的改法都是零轮、只在我的模型上量过（并行行带写者那一条）或推的。
