# landing-audit-r7 攻方腿（Opus）：K2 内部矛盾与越位、K3 哪一条最脆

日期：2026-10-24。被判对象：开工快照 `research/prompts/landing-audit-r7-snapshot/` 倒推出的那一版（`snapshot-tree.py landing-audit-r7` 退 0，清单点名 80 份全对上，见 K2-1）。
这一条腿没写模型目录：全部现试是在草稿目录 `/home/fy5090/.local/share/singlefs/work/landing-audit-r7-opus/` 里喂造出来的输入或真派发提示，探针与输入的 sha256 列在 `/home/fy5090/.local/share/singlefs/work/landing-audit-r7-opus/probe-sha256.txt`（28 份）；每一格下面给复跑命令与原样输出。
提的改法都只在我的探针上量过、被攻过零轮。

## 各格判定一览

| 格 | 编号 | 一句话 | 量的 / 推的 |
|---|---|---|---|
| K2 snapshot-tree 与规则第 1 步 | K2-1 | 第 1 步只倒推 `judged-sha256.txt`，`kb-sha256.txt` 那几份不进树也不报「倒推不出」；cite-check 在倒推树上按文件名找不到任何文件 | 量的（r6 真腿报告：114 处「仓里没有这个文件名」，r4/r5/r6 的 kb 清单各 3/2/4 份倒推不出） |
| K2 弄坏开关守卫 | K2-2 | 「平常跑设了就拒」的守卫被自己的关守卫项借道：开关值按逗号拆，关守卫那一项与别的项一起写，平常跑照认 | 量的（snapshot-tree.py 真跑 r6：sha256 不核、HEAD 那一份当快照铺进树并打 ✓）；三份钩子与 stage-cells.sh 是推的 |
| K2 break-switch-replay 与自带开关轮 | K2-3 | 基准趟的限时用「每条开关最多几秒」，自带开关轮的两份基准趟里整轮开关都跑一遍；两套判法读同一张表的不同部分 | 推的（基准趟 535/680/895 秒取自任务 54 报告，门禁负载下会不会超时没量） |
| K2 deferred-items-registered 与主 agent 定义第 2 条 | K2-4 | 两边认的登记处不是同一组：门禁另认两份流程文件、任何文件的任何一行（含调度记录自己那一行）、已还清的 C 号 | 推的（照代码读；门禁阶段在副本小仓里起被重型闸拒，原样贴在 K2-4） |
| K2 看门狗内存过线 | K2-5 | 共用 `registered_light_selftest` 之后，工作根副本里入口同 sha256、它 source 的库改过的自证，底下整棵不报内存过线 | 量的（认法那一半：副本那一份照认成轻的自证）；「真过线进程被排掉」是推的 |
| K2 定义「输入」与派发闸 ⑭ | K2-6 | 七份定义的输入一节写超过 2 小时要三行，派发闸另要「复用：」一行；照定义写的提示被拒 | 量的 |
| K2 stage-cells.sh 认样本根 | — | 没打中样本根那一判本身；关守卫项见 K2-2 | — |
| K2 交回闸 ⑤ ⑦ 与共用约束 | — | 没打中 | — |
| K3 最脆的一格 | K3-1 | 派发闸 ⑱ 写死主机资源（HOST_QUOTA）按字面挨着认：同一句意思换个说法就放，观测句反而拒 | 量的（14 种造出来的写法；真派发提示 11963 份里按宽正则挑出 212 份，2026-10-14 起去重 18 份过 --check-prompt，这一判拒 0 份） |
| K3 E171 修订 11 槽数与线程合计 | K3-2 | 第二段修订 11（经修订 10 的让路）与第一段修订 8 逐行编排各按登记值分槽，同一台线程合计 60，登记 31；不在一百一十九节 ①② 的射程里 | 量的（allocate 两次的算术）；两段同时在跑的时间段是推的 |

## K2　内部矛盾与越位

### K2-1　snapshot-tree.py 与三方规则第 1 步：kb 清单不倒推，按文件名的引文在倒推树上一律认不出

**哪一处。** 规则第 1 步只点名一份清单：「照 `research/prompts/<轮>-snapshot/` 的 `judged-sha256.txt`」（`.claude/rules/three-way-inference.md:125`），命令是「先跑 `python3 scripts/agents/snapshot-tree.py <轮> --out <草稿目录>/<轮>-tree`」（`.claude/rules/three-way-inference.md:125`）。脚本不给 `--list` 时「清单：默认 judged-sha256.txt」（`scripts/agents/snapshot-tree.py:12`）。快照目录里另有 `kb-sha256.txt`（这一轮 4 份 kb），它不进树，也不进「倒推不出」的点名；规则要判决的人写明的只有「倒推不出、sha256 对不上的那几份，判决里写明是哪几份」（`.claude/rules/three-way-inference.md:125`），kb 那几份因此一声不出。
第二处：cite-check 只写文件名的引文在 `--root` 那一棵里跑 `git ls-files` 找（「output = subprocess.run(["git", "-C", root, "ls-files"]」（`scripts/agents/cite-check.py:108`）），倒推树不是 git 仓，任何只写文件名的引文都报「仓里没有这个文件名」，连树里就有的那份也一样。

**怎么复出来**（草稿目录里，只读主仓）：

```
python3 scripts/agents/snapshot-tree.py landing-audit-r7 --out <草稿目录>/r7-tree
  ✓ 快照 landing-audit-r7：清单点名 80 份，倒推出 80 份，sha256 对上 80 份（基底取自 HEAD 37 份、diff 头记的 blob 0 份、新文件 43 份；diff 用的是 judged-vs-head.diff）；树在 /home/fy5090/.local/share/singlefs/work/landing-audit-r7-opus/r7-tree
python3 scripts/agents/cite-check.py <草稿目录>/probe-report.md --root <草稿目录>/r7-tree
  - 没判：probe-report.md:2 引 .claude/kb/checks-owed.md:3：路径认不出（在给的根下都找不到）
  - 没判：probe-report.md:5 引 snapshot-tree.py:4：路径认不出（仓里没有这个文件名）
  ✓ 核了 4 处引文、0 处裸行号，对上 2 处，没判 2 处（其中裸行号没有引文可比 0 处；逐处列在上面）
```

探针报告四处引文都是此刻逐字对得上的（同一份不给 `--root` 跑报「对上 4 处，没判 0 处」）；第 5 行引的 `snapshot-tree.py` 就在树里。

真输入（r6 两条腿的报告，照规则第 1 步）：

```
python3 scripts/agents/snapshot-tree.py landing-audit-r6 --out <草稿目录>/r6-tree        （退 0，40/40）
python3 scripts/agents/cite-check.py research/prompts/landing-audit-r6-opus-output.md research/prompts/landing-audit-r6-sonnet-output.md --root <草稿目录>/r6-tree --background research/prompts/_landing-audit-r6-background.md
  ✓ 核了 15 处引文、144 处裸行号，对上 11 处，没判 148 处（其中裸行号没有引文可比 20 处；逐处列在上面）
```

148 处没判按原因数（`grep -o '：[^：]*$' | sort | uniq -c`）：114 处「路径认不出（仓里没有这个文件名）」、20 处「行号在文件范围内，没有引文可比」、14 处「路径认不出（在给的根下都找不到）」。不给 `--root` 时这 114 处里的裸行号都能核行号越不越界；给了倒推树就一处也核不了。加上 kb 清单再倒推：

```
python3 scripts/agents/snapshot-tree.py landing-audit-r6 --out <草稿目录>/r6-tree-kb --list judged-sha256.txt --list kb-sha256.txt
  ✗ 快照 landing-audit-r6：清单点名 44 份，倒推出 40 份、sha256 对上 40 份；倒推不出 4 份、对不上 0 份（逐份列在上面；没对上的不铺进树；diff 用的是 judged-vs-head.diff）
```

r4、r5 同样加 kb 清单：「倒推不出 3 份」「倒推不出 2 份」。kb 在快照之后多半被改过、又不在 diff 里，倒推不出是常态；照规则今天的写法，判决的人只看到 kb 引文落进「没判」，不知道那几份 kb 快照那一刻的样子已经取不回、人工核时拿的是此刻的 kb。两根并给（`--root <树> --root .`）也不行：r6 那一趟出 1 处假红，引的是快照之后改过的 `.claude/rules/three-way-inference.md:125`，它不在 r6 清单里，从仓根取到的是此刻那一版。

**改法（被攻过零轮）：** ① 规则第 1 步与 `snapshot-tree.py` 成功句里打的命令都带上快照目录里的每一份清单（`--list judged-sha256.txt --list kb-sha256.txt`，或脚本默认读目录里全部 `*-sha256.txt`），倒推不出的 kb 照现有那一句写进判决；② cite-check 给了 `--root` 而那棵不是 git 仓时，按文件名在那棵树里走目录找（或倒推时在树根写一份文件清单给它读），不要一律报「仓里没有这个文件名」。
**推翻条件：** 判决的人另有一处规则要求逐份写 kb 清单的倒推结局（我没找到）；或 cite-check 在非 git 的根上本来就按目录找，114 处是别的原因。

### K2-2　「平常跑设了就拒」被关守卫那一项借道

**哪一处。** 存量表的改法：「设了就退 2 并点名它只在自证里认」（`.claude/gate.d/break-switch-outside-selftest-pending.tsv:3`）。这一批改成守住的几份，都留着一项「关掉守卫」的开关，而开关值按逗号拆成几项：`snapshot-tree.py` 拆法「.split(",") if item)」（`scripts/agents/snapshot-tree.py:77`），守卫在关守卫项在时放行「or "guard-off" in BROKEN:」（`scripts/agents/snapshot-tree.py:146`）；续做闸「"honor-outside-selftest" in value.split(",")」（`.claude/hooks/continuation-guard.sh:106`）；检出钩子「return item in os.environ.get(DETECTOR_BREAK_VARIABLE, "").split(",")」（`.claude/hooks/bash-command-detector.sh:700`）。门禁阶段那一侧，`stage-cells.sh` 在判样本根之前先认它自己的关守卫项「== break-honored-everywhere ]] && return 0」（`.claude/gate.d/lib/stage-cells.sh:233`），之后整份环境里别的 `*_BREAK` 照认。门禁 code-tooling 的 break-switch-outside-selftest 格只用一个值探：「文件头开关表里这个变量的第一个值」（`.claude/gate.d/code-tooling.sh:4345`），探不到「关守卫项加别的项」这种写法。

**怎么复出来**（真仓上平常跑，不是 `--selftest`）：

```
SNAPSHOT_TREE_BREAK=skip-sha-check python3 scripts/agents/snapshot-tree.py landing-audit-r6 --out <草稿目录>/r6-g1 --list judged-sha256.txt --list kb-sha256.txt
  ✗ SNAPSHOT_TREE_BREAK=skip-sha-check 是弄坏开关，只在自证（--selftest）里认；平常跑设了就拒，这一次什么都没做
（退 2）
SNAPSHOT_TREE_BREAK=guard-off,skip-sha-check python3 scripts/agents/snapshot-tree.py landing-audit-r6 --out <草稿目录>/r6-g2 --list judged-sha256.txt --list kb-sha256.txt
  ✓ .claude/kb/checks-owed.md  来源：HEAD（diff 里没有它）  sha256 a109e9f0ea79
  ✗ 快照 landing-audit-r6：清单点名 44 份，倒推出 42 份、sha256 对上 42 份；倒推不出 2 份、对不上 0 份（逐份列在上面；没对上的不铺进树；diff 用的是 judged-vs-head.diff）
（退 1）
```

K2-1 里同一份 `checks-owed.md` 照实判是「此刻 HEAD 那一份 sha256 b34a284931af 不是快照的 a109e9f0ea79」；第二条命令把此刻那一份铺进树、打 ✓、写的却是快照的 sha256。续做闸、检出钩子、`stage-cells.sh` 三处没起（钩子平常跑要 hook 的 JSON，门禁阶段子 agent 起被重型闸拒），是照代码推的。
钩子那两份的变量要漏进 Claude Code 自己的环境才会生效，实际风险比主 agent 手跑的 `snapshot-tree.py`、门禁阶段小。

**改法（被攻过零轮）：** 关守卫那一项只在它是唯一一项时认（与别的项同写时照拒、点名），或只在带自证标记的子进程里认；break-switch-outside-selftest 格的探针另起一次「关守卫项,第一个值」。
**推翻条件：** 自证里有一格造了「关守卫项加别的项」、平常跑判它被拒（我在四份文件头的开关表里没见到这样的格）。

### K2-3　break-switch-replay 格与自带开关轮的两份：限时口径与读表都不是一套

**哪一处。** 格的注释说自带开关轮的是外面设开关被拒的那种：「退 2 并点名这个变量只在自证里认的是」（`.claude/gate.d/code-tooling.sh:4435`）自证自带开关轮。可登记表里写着「自证自带开关轮」的 `crash-point-replay.sh`、`crash-amplification.sh` 都不拒外面设的开关，被逐条复跑（`.claude/gate.d/break-switch-replay-files.tsv:52`：「四十一条开关逐个跑过两趟」，同一行另写单跑一条最慢 490 秒）。于是同一张文件头开关表有两个判法：自证里的开关轮读括号外的格名、比格名集合相等，括号里的失败行原文不读（`scripts/crash/crash-amplification-lib.sh` 的 `crash_amplification_read_switch_table` 去掉括号再切）；门禁这一格读「…」里的失败行原文、减基准趟、要求每一行 ✗ 都被点名。两边都绿才算对，各自都发现不了另一边那一半写错。
限时：基准趟与开关趟用同一个数，「run_selftest(path, seconds_of[path])」（`.claude/gate.d/code-tooling.sh:4631`），登记表那一列的意思是每条开关最多几秒。自带开关轮的两份，基准趟不设开关、走完整份自证再把全部开关轮一遍（四十一条、五十条），开关趟在格红之后就返回、不起开关轮。任务 54 报告记的基准趟是「基准趟 535、680、895 秒」（`research/prompts/subagent-landing-audit-2026-10-22/b12-switch-naming-final.md:102`），登记的限时是 800、1000、1100：基准趟没超时是余量碰巧够，不是这一列算的。门禁里各份并行至多 8 路、每份开关轮里再按登记线程数并行，负载比量那几趟时高，基准趟超时就是「不设开关的 --selftest 就没过（超时）」、这一份一条开关都不起。

**怎么复出来：** 这两份自证一次 535 到 895 秒，前台超过 240 秒的限时，我没跑；是照代码与任务 54 报告推的。复跑要主 agent 起：`bash .claude/gate.d/code-tooling.sh --check break-switch-replay <根>` 带 `SINGLEFS_HEAVY_TESTS=commit`，看那两份有没有「不设开关的 --selftest 就没过（超时）」。
**改法（被攻过零轮）：** 登记表加一列基准趟限时（或格里对自带开关轮的那几份把基准趟限时乘上开关条数除以并行数）；注释里「自带开关轮」两种情形分开写，并写明开关轮读格名、这一格读原文。
**推翻条件：** 门禁整轮里这两份的基准趟在提交前那一趟实测都远低于登记限时（读整轮门禁日志里这一格的耗时）。

### K2-4　deferred-items-registered 与主 agent 定义第 2 条：认的登记处不是同一组

**主 agent 定义那一边：** 延后的登在里程碑收口表、欠账表或决策正文，子 agent 的事登拆分提案编号小节，「只写进调度记录不算延后，算没去向。」（`.claude/main-agent.md:21`）。
**门禁那一边**（`.claude/gate.d/doc-process-records.sh` deferred-items-registered 格）：① C 号「（开着的与已还清的都算）」（`.claude/gate.d/doc-process-records.sh:269`）；③ 任意文件:行，只要「路径从仓根起、文件在树里、行号不超过它的行数」（`.claude/gate.d/doc-process-records.sh:274`），判的是同一行的原文「found = pointers(text)」（`.claude/gate.d/doc-process-records.sh:2135`），不排除调度记录自己、不排除这一行自己；④ 登记处表里两份流程文件（「提交前的整轮门禁由主 agent 每次提交时照它跑」（`.claude/gate.d/deferral-registries.tsv:4`））；两份拆分提案（新份、旧份）里的行都不判。
**矛盾的几处：**
- 写「这一件延后，见 `records/<调度记录>.md:<这一行自己的行号>`」，门禁照 ③ 认登记了；主 agent 定义说这是没去向。
- 写「延后（C10）」而 C10 早已还清、讲的是别的事，门禁认；定义要「登一行」。
- 延后到「提交前跑」写 `staged-gate.md`，门禁认；定义的四种登记处里没有流程文件。登记处表自己的说法是「要另登一行才有人接的（欠账表、拆分提案、决策正文）不进这张表」（`.claude/gate.d/deferral-registries.tsv:3`），定义第 2 条没跟上这一种。

**怎么复出来：** 我在草稿目录里搭了带两次提交的小仓，同一条命令里写进五行带延后词的探针（甲到戊）再起 `bash <小仓>/.claude/gate.d/doc-process-records.sh --check deferred-items-registered <小仓>`，被重型闸执行前拒，原样：「✗ 提交时才跑的检查被拒：门禁阶段 doc-process-records.sh（扫仓的门禁归提交前主 agent 跑的 scripts/gate/gate-staged.sh）」。没换写法绕过去，上面三条是照代码读的。整条命令在执行前被拒，探针五行也没写进小仓；五行原样另存在 `<草稿目录>/defer-probe-lines.md`（乙那一行指到它自己所在的第 4 行），主 agent 可放进带 C1 开着、两份拆分提案的小仓里跑（小仓交回前删了）。
**改法（被攻过零轮）：** ③ 排除指到调度记录（`records/` 下不是两份拆分提案的文件）的 文件:行，或至少排除指到这一行自己的；① 只认开着的 C 号；定义第 2 条补一句「延后到提交时那一套流程的，写登记处表里的那份文件」，两边同一组。
**推翻条件：** 门禁真跑时 ③ 对指到 `records/` 自己的行另有排除（我在 `pointers` 与调用它的循环里没找到）。

### K2-5　看门狗内存过线共用 registered_light_selftest：工作根副本里改过的库照认成轻的自证

**哪一处。** 内存过线那一支找上层的认法：「里最近的登记成轻的自证（registered_light_selftest，与」（`scripts/agents/agent-watch.py:2755`），与重型测试那一判同一份，认出来就整棵不报。这份认法照自证登记表的副本规则：「副本里那份脚本与主仓那一份 sha256 相同的放行」（`.claude/hooks/selftest-registry.tsv:12`），只核入口那一份。入口 source 的库不核：`crash-amplification-lib.sh` 一开头就「source "$(dirname "${BASH_SOURCE[0]}")/../hosts/heavy-threads.sh"」（`scripts/crash/crash-amplification-lib.sh:119`）。

**怎么复出来：** 在 `<草稿目录>/wcopy/` 放 `scripts/crash/crash-amplification-lib.sh`（与主仓同 sha256）、登记表、往 `scripts/hosts/heavy-threads.sh` 末尾追加一行；`python3 -I <草稿目录>/probe_watch.py <草稿目录>/wcopy`：

```
本仓那一份 → scripts/crash/crash-amplification-lib.sh --selftest
工作根副本、入口同 sha256、它 source 的库改过 → scripts/crash/crash-amplification-lib.sh --selftest
bash -c 里起自证再起别的 → None
```

第二行：副本里库被改过的那一趟照认成轻的自证，它底下不论吃多少内存都不进「进程常驻内存过线」。重型闸那一半是拆分提案一百一十八节（不在这一轮不重复报的四节里）已记的口子；这一批让内存告警也共用它，告警一侧多出一处盲区。「真有过线进程被排掉」没造（要在副本里改库起一个吃内存的进程），是推的。第三行是没打中的形状：`bash -c` 那一层不当自证。
**改法（被攻过零轮）：** 内存过线不排登记成轻的自证底下的进程，改成只排它的探针格（照登记表理由里写的上限，例如 `run-with-memory-cap.sh` 那份写的 64M、`agent-watch.py` 那份写的 256M，超过照报）；或副本那一种只在入口 source 的库也逐份同 sha256 时认。
**推翻条件：** 内存过线那一支另有一道「登记成轻的自证底下超过某个上限照报」（我在 `memory_report` 里没找到）。

### K2-6　七份定义的输入一节与派发闸 ⑭：超过 2 小时少写「复用：」

主 agent 定义说定义的输入一节「一节是派发时必须给的东西，缺一样它不开工」（`.claude/main-agent.md:7`）。攻方定义的输入写超过 2 小时另有分片、续跑、多机三行，「派发闸查，缺了派不出来」（`.claude/agents/three-way-attack.md:24`）；派发闸 ⑭ 另要「复用：」一行。数一遍：带预计挂钟那一条的十份定义里，clerk、implementation-writer、investigator、three-way-attack、three-way-defense、three-way-forward、tooling-writer 七份一处「复用：」都没有。照攻方定义写三行的提示（`<草稿目录>/p/attack-long.txt`）喂拷进草稿目录的派发闸：

```
bash <草稿目录>/gcopy/.claude/hooks/runner-dispatch-guard.sh --check-prompt <草稿目录>/p/attack-long.txt --type three-way-attack --root .
✗ 派 three-way-attack 的提示写了预计挂钟超过 2 小时，长活声明不齐：
  缺 复用（复用：）：超过 2 小时的活要写明判定按什么入库、键相同的怎么沿用；这一件不产出判定的写「复用：无判定：<为什么>」，「分不了：」顶不了这一行
（退 2）
```

照骨架写的不撞（骨架打这一行）。**改法：** 七份定义的输入一节那一条补「复用：」，或改成「超过 2 小时的几行照骨架」。**推翻条件：** 定义输入一节本来就不当派发的全集（主 agent 定义第 7 行那一句另有限定，我没找到）。

### 没打中的 K2 子项

- 交回闸 ⑤ 认命名检查跑完：喂 8 种写法（骨架落盘写法、`nice -n 19` 在声明前后、`env` 带声明、上游 lint 落盘、`two-logs.sh` 包着、`echo rc=$?` 追加进同一份文件再 tail），7 种认成跑完；`cd <根> && <检查> > 文件; echo rc=$?` 打出 rc=1 那一种不收（原因 and-nonzero），是文件头写明的「前面 && 连着 cd 或命名检查的只认 0」，不算矛盾。探针 `<草稿目录>/hb/probe_naming.py`（从 `handback-guard.sh` 第 380–2271 行抽出 python 进程内调 `completed_naming_text`）。
- `stage-cells.sh` 认样本根：照代码读，阶段所在的仓要「第一行逐字那一句接 pwd -P」，`--staged` 有 `GATE_STAGED_FROM` 拒，拷到别处的小仓第一行路径对不上拒，仓副本根上没有标记拒；能让平常跑也认开关的只有 K2-2 那一项关守卫开关。
- snapshot-tree.py 与搬迁规则第 6 步：r4、r5、r6 照默认清单倒推 80/80、27/27、40/40 都对上（含 r4 12 份、r5 11 份照搬迁登记换回旧路径），两边说的是同一件事。
- 一份真提示照骨架写能不能同时过所有判：攻方骨架填满（`<草稿目录>/p/attack-ok.txt`）派发闸放行；交回闸 ② 对三方腿认的报告形态与派发闸 ⑰ 同一个库函数，没找到一处要求、一处拒（K2-6 是定义与派发闸之间，不是两道闸之间）。

## K3　哪一条最脆

### K3-1　派发闸 ⑱「写死主机资源的数」（HOST_QUOTA）：换个说法就放，观测句反而拒

**哪一处。** 共用约束第 90 行写的判据是「本机与第二台成对的线程数、每台或两台各 N 线程或 NG、两台或内存合计 NG」（`.claude/agent-common.md:90`），钩子的注释同一句「写死的主机资源数：本机与第二台成对的线程数、每台 / 两台各 N 线程或 NG」（`.claude/hooks/runner-dispatch-guard.sh:2318`），正则只认数字紧挨在量词后面的字面：「r"|(?:每台|两台各|各台)\s*\d+\s*(?:个)?(?:线程」（`.claude/hooks/runner-dispatch-guard.sh:2320`），成对那一支要「本机 N 线程」与「第二台 N 线程」两段都写成数字接「线程」。

**真输入里找过：** 主会话记录里的 Agent / Task 调用 11963 份，按宽正则（本机 / 第二台 / 两台 / 每台 后 24 字内接数字与线程、路、核、G）挑出 212 份，2026-10-14 起去重 18 份（`<草稿目录>/real/`，`scan_prompts.py` 抽的），逐份经拷进草稿目录的派发闸 `--check-prompt`，这一判拒 0 份；拒的那些是缺新加的几行，与这一判无关。真提示上没撞上，误放误拦都在下面造的输入上。

**怎么复出来：** 派发闸与它要的库拷进 `<草稿目录>/gcopy/`（`.claude/hooks`、`.claude/agents`、`.claude/heavy-thread-default`、`.claude/singlefs-ai-sop/scripts`、`scripts/hosts/heavy_threads.py`、`heavy-threads.sh`），一份 2 小时内的 tooling-writer 提示（`<草稿目录>/p/b4-<号>.txt`），只换「关的是哪几条」第 1 条那一句；`bash <草稿目录>/gcopy/.claude/hooks/runner-dispatch-guard.sh --check-prompt <草稿目录>/p/b4-<号>.txt --type tooling-writer --root .`：

| 号 | 那一句 | 退出码 | 该不该拒（照第 90 行） |
|---|---|---|---|
| 11 | 两台各 31 线程跑。 | 2 | 拒 |
| 31 | 两台各跑 31 线程。 | 0 | 拒（同一个派法，中间多一个「跑」） |
| 12 | 每台 31 线程跑。 | 2 | 拒 |
| 32 | 每台跑 31 线程。 | 0 | 拒 |
| 13 | 本机与第二台各 31 线程跑。 | 0 | 拒（两台各 N 线程） |
| 14 | 本机和第二台都用 31 线程跑。 | 0 | 拒 |
| 15 | 本机 31、第二台 31 线程跑。 | 0 | 拒（成对） |
| 16 | 本机用 31 线程、第二台用 16 线程起。 | 0 | 拒（成对） |
| 20 | 第二台 16 线程，本机 31 线程。 | 2 | 拒 |
| 17 | 每台 24G。 | 2 | 拒 |
| 33 | 每台都给 24G。 | 0 | 拒 |
| 18 | 两台各 36G 内存跑。 | 2 | 拒 |
| 19 | 本机与第二台各给 36G。 | 0 | 拒 |
| 21 | 本机跑 31 路并行，第二台跑 31 路并行。 | 0 | 拒（成对） |
| 1 | 上一趟主 agent 复跑两趟（本机并行 31，第二台并行 31），一趟不同 3。 | 0 | 放（观测） |
| 7 | 本机 31 线程、第二台 31 线程的那一趟里 3 条不同。 | 2 | 放（观测，与第 1 号同一件事） |

第 7 号的拒绝说明原样：「✗ 派 tooling-writer 的提示正文写死了主机资源的数：「本机 31 线程、第二台 31 线程」。」第 1 号与第 7 号说的是同一件事，一放一拒；第 13 到 16、19、21、31 到 33 号都是照第 90 行该拒的派法，放了。主 agent 调度记录里的原话就有第 1 号那种写法（第十二批节「本机并行 31，第二台经 `multi-host-run.sh --peer` 并行 31」），往提示里抄观测时撞不撞全看用的是「并行 31」还是「31 线程」。
四句：分不分辨——这一判本身只有一条臂，不涉及；钩子看不看得到判别它的东西——派法与观测钩子在字面上分不出（注释自己写了「写死的是派法还是观测，钩子分不出」），所以误拦那一半是判别子观测不到；误放那一半是字面覆盖不全，看得到；满足的分句——第 13、19 号满足第 90 行「两台各 N 线程或 NG」那一分句，第 15、16、21 号满足「本机与第二台成对的线程数」；改法在这几格上——见下。
**改法（被攻过零轮）：** 量词与数之间、两台之间允许插几字（「各跑」「都用」「用」「各给」），「路」「路并行」与「线程」同等；观测那一半另给一条放行写法（例如句子里有「上一趟」「量到」「报在用」时不判，或要求观测写进「」引文，钩子已经不判引文）。改宽要回到真语料 3484 份上再量一遍新拒新放（任务 49 那一套）。
**推翻条件：** 用户定过只拦今天这几种字面、别的写法交主 agent 自觉（我在第九十节 ⑤ 与任务 49 报告里只读到「N 核」与单写一台不判这两条代价）。

### K3-2　E171 修订 11：第二段与第一段逐行编排各按登记值分槽，同一台合计 60 线程

**哪一处。** 编排器分配：「每台线程 = 登记值（threads 不给就现读登记，不看忙闲）」（`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator.py:287`）。改前每台线程读的是此刻空闲（判的 diff 里删掉的豁免行「HostResources 的字段读 host-resources.py 此刻空闲线程」（`research/prompts/landing-audit-r7-snapshot/judged-vs-head.diff:8634`））。第二段修订 11 原样套修订 10，修订 10 与第一段的协调是让路：「第二段在各台都不起新行，在跑的跑完」（`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator_segment_two_revision_10.py:18`），只在第一段第 11、12 步与跨机对照还有排着没起的行时停派；第一段的行都派出去了（在跑）、或在跑第 13 步超时重跑时，第二段照自己的满额分配派新行。改前两边都看此刻空闲，后起的一边只拿得到剩下的线程；改后两边各拿满额。一百一十九节写的是「都只在修订 8 那几层」（`records/2026-10-23-subagent拆分提案.md:1124`）：① 是修订 8 逐行编排自己的加槽，② 是修订 8 第 0–10 步与第 11–13 步重叠，第一段与第二段之间这一处不在里面。

**怎么复出来：** `python3 -I <草稿目录>/probe_e171.py`（只载入 `rows_orchestrator.py` 调 `allocate`，两台、内存给足，模拟两段各分一次）：

```
第一段逐行编排 [(1, 31, 6, 5, 30), (2, 31, 6, 5, 30)]
第二段（修订 10 让路那一层同一个 allocate） [(1, 31, 6, 5, 30), (2, 31, 6, 5, 30)]
台 1：两段各按登记值分槽，线程合计 60，登记值 31
台 2：两段各按登记值分槽，线程合计 60，登记值 31
```

元组是（台号、每台线程、槽数、每槽线程、槽数乘每槽线程）。修订 10 的分配只在本机闸没开时把本机记 0（`allocate_behind_the_local_gate`），本机闸开了之后与别的台一样满额。两段真在同一台上同时跑的时间段是照让路那几行推的，没起装置。修订 11 第一段经修订 8 那一层跑，它自己的加槽与重叠段是一百一十九节 ①②，这里不重复。
**改法（被攻过零轮）：** 让路那一层在派第二段新行之前，按第一段行目录里在跑的行数乘每槽线程扣掉这一台的线程，余下的不到一槽就不派；或两段共用一份「这一台已分出去的线程」账（与一百一十九节 ① ② 同一个改法，合成一件），自证造一格两段同时在跑、核每台合计不超过登记值，带弄坏开关。修订 10 那一层被修订 11 按 sha256 钉住，改它要连同重钉。
**推翻条件：** 修订 10 的让路另有一处按第一段在跑的槽扣线程（我在它文件头 ⑤ ⑦ 与 `allocate_behind_the_local_gate` 里没找到），或修订 11 第二段的起跑闸要求第一段第 11–13 步全部做完才起（它写的是第 0、1、2 步绿）。

### 没打中的 K3 形状

- 交回闸 ⑤：真会话记录里子 agent 跑命名检查最常见的写法（骨架落盘、带 `nice -n 19`、经 `two-logs.sh`）都认得出，见 K2 没打中那一节。
- 看门狗「重型测试没带前缀」与内存过线：子 agent 的 Bash 都是 `/bin/bash -c "source … && eval '<命令>'"` 起的，`launched_script` 对 `bash -c` 交 None，命令串里先跑一条登记成轻的自证再接 cargo，不会把 cargo 排掉。
- 派发闸 ⑯ 两种围栏认法、⑰ 报告名大小写与 `$SINGLEFS_WORK_ROOT/`：任务 40、48 已在 4006 / 4026 份真提示上量过 0 份撞上，我没再造输入。

## 没打中的形状（汇总）

- K2：交回闸 ⑤ 的 8 种跑法写法；派发闸照骨架填满的攻方提示；交回闸 ② 与派发闸 ⑰ 的三方报告形态（同一个库函数）；`stage-cells.sh` 在真仓、`--staged`、拷走的小仓、仓副本根上照拒；`snapshot-tree.py` 照搬迁登记换回在 r4、r5、r6 上全对上；看门狗 `bash -c` 那一层不当自证。
- K3：真派发提示上 HOST_QUOTA 这一判 0 份误拦误放（18 份）；E171 修订 11 的第二段加槽只给起跑时一槽都没有的台（`rows_orchestrator_segment_two.py` 的 `make_regrow`，`waiting` 只收没槽的台），不重复分；第一段 `rows_orchestrator.py` 本身没有加槽。
- 取样范围：派发闸 ⑱ 造了 16 种句子（14 种派法、2 种观测）；交回闸 ⑤ 8 种；真派发提示 11963 份按宽正则挑 212 份、10-14 起去重 18 份；快照倒推 r4–r7 四轮，cite-check 只在 r6 两份真腿报告上比了三种根。

## 这条腿自己的限度

- K2-3、K2-4 与 K2-2 里钩子、`stage-cells.sh` 那几处是照代码推的：门禁阶段子 agent 起被重型闸拒（K2-4 贴了拒绝原文），两份慢自证前台跑不完。
- K3-1 的「该不该拒」一列是我照共用约束第 90 行字面判的，钩子注释承认派法与观测字面分不出，误拦那一半改不改是取舍，不是对错。
- K3-2 只量了分配的算术，没起装置看两段真在同一台同时跑多久。
- K2-5 只量了认法那一半，没在副本里造一个吃内存的进程看告警漏报。
- 交回报告与调度记录里写的数（任务 54 的基准趟秒数、任务 40/48 的真提示份数）我直接引了，没复算。

## 要主 agent 跑的命令

1. K2-3：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 break-switch-replay 格; 范围=1 格; 为什么=核自带开关轮那两份基准趟在门禁负载下超不超登记限时' SINGLEFS_HEAVY_TESTS=commit bash .claude/gate.d/code-tooling.sh --check break-switch-replay <仓根>`，工作目录仓根，读输出里 `scripts/crash/crash-point-replay.sh`、`scripts/crash/crash-amplification.sh` 两行有没有「不设开关的 --selftest 就没过（超时）」。
2. K2-4：把 `<草稿目录>/defer-probe-lines.md` 放进一个带 `.claude/kb/checks-owed.md`、两份拆分提案、`.claude/gate.d/deferral-registries.tsv` 与两份流程文件的两次提交小仓，路径写成 `records/2026-10-24-探针.md`，起 `bash .claude/gate.d/doc-process-records.sh --check deferred-items-registered <小仓>`，看甲、乙两行（指到调度记录、指到自己那一行）是不是没报。
3. K2-5：在工作根下的副本里放 `scripts/crash/crash-amplification-lib.sh`（与主仓同 sha256）、`.claude/hooks/selftest-registry.tsv`，改 `scripts/hosts/heavy-threads.sh`，`python3 -I <草稿目录>/probe_watch.py <副本>` 看第二行是不是照认成轻的自证。
4. K3-1：照 K3-1 那一段的拷法建 `gcopy`，逐份喂 `<草稿目录>/p/b4-*.txt`。

## 没做什么

- 不判 K1（正推那一格），没读正推腿报告（禁读清单）。
- 没编译、没跑重型、没跑门禁阶段与各类 lint；没写模型目录。
- 草稿目录里删了的副本与树（交回前 `du -sh`）：门禁小仓 `defer-repo` 49M、`gcopy` 9.8M、`wcopy` 156K、倒推树 r4 4.7M、r5 6.3M、r6 3.1M、r7 6.9M、加 kb 清单的 r4 5.0M、r5 6.6M、r6 3.1M、关守卫那一趟 r6 4.6M。留着的是探针、输入与输出文本（`<草稿目录>/p/`、`real/`、`hb/`、`*.txt`），不含仓副本与编译目录。
