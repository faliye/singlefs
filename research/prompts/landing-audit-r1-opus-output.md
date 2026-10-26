# landing-audit-r1 攻方腿（Opus）报告：K2 合并与重合、K3 矛盾、K4 数（2026-10-22）

立场：假设这一批改动在三面上都没毛病，造输入去打穿它。被判的改动按 `research/prompts/landing-audit-r1-snapshot/judged-vs-index.diff` 里正文与调度记录点名的那一批认；diff 里别的会话的改动（`replay.sh` 按名字判、TaskStop 找不到、几行阶段归属）不判。
开工时 `sha256sum -c research/prompts/landing-audit-r1-snapshot/judged-sha256.txt`：48 份对得上，`.claude/gate.d/stage-owners.tsv`、`.claude/kb/checks-owed.md`、`.claude/kb/checks-owed.en.md` 三份此后被改过（别的会话），引它们的行号按今天的文件取。

## 复跑

模型目录 `research/prompts/landing-audit-r1-opus-model/`。两条命令，工作目录是仓根：

```
RUN_WITH_MEMORY_CAP_WAIT_SECONDS=60 nice -n 19 bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 7G \
  bash research/prompts/landing-audit-r1-opus-model/tools/rerun.sh --repo "$PWD" --frozen "$PWD/research/prompts/landing-audit-r1-snapshot" --work <新的草稿目录>
bash research/prompts/landing-audit-r1-opus-model/tools/naming_copy_probe.sh "$PWD" <新的空草稿目录>
```

第一条约 40 到 70 秒（结局账那一步读约 12 GB 会话记录，峰值 5.9 GiB，包装报的），输出与 `out/` 下同名文件逐行比；结局账随会话目录与此刻变，只比口径。第二条约 55 秒，输出比 `out/naming-roots.txt`。

| 文件 | sha256 |
|---|---|
| tools/rerun.sh | c3ae74b49de9c3d9371c978e6f3efc367f955b4d08826df2abbe08add103a9a6 |
| tools/clone_check.py | bf8b48c5464b640e089d48ca6347b5f2260c3b5e841b44549e13ed1f49e624a7 |
| tools/report_label_probe.sh | 172341f9b543b9771aaf244ee12ab9bc127c7bb9a4deaaac7f975cd1b3fd71b6 |
| tools/fill_skeleton.py | d9e1618685c49f21c5ada3f258979cfddff3e8b83142b72ce68c84dc60b44624 |
| tools/hook_probe.sh | 2119abda0b3cd5193621955c50afd813ed8c4cb28ed285f30735a92f12073a25 |
| tools/naming_copy_probe.sh | 727bbbe68dd0ab6ba93f3364d3d3a8d90ca94279dfcac487fdd49e96c55e2838 |
| tools/blind_agreement.py | 68df1aa920be631948e6955f9a3306e3664c54cda067c62bf976308b799d371e |
| tools/deferral_hits.py | 91a2381a1fe8f166e5a13d4fbd81ebe156fbe3aa1200ab53e1ab5a9ab581e2b4 |
| tools/outcome_check.py | 035885455161de93d9ccf4299ff7301692b96c0ed24c17cacc0afa263c94a8df |
| out/judged-code.tsv | 76aa887d26dd5d7f2738c31b97f6a9c3f76faa1249c46969d1597fe27590b02d |
| out/clone.txt | f2a45d0368884c2f56453781a8afd9163be26ef38b74a04bf105571e62df1890 |
| out/k2-report-labels.txt | 661eb8587c4bfcc3241f3bc2e71982c3ae9fbefa9be55b51cf98b9e81be8e19f |
| out/k3-dispatch.txt | 09545183b4167524cee7f1949a3b7c53d7563d27db701f7944a6ee4fab3b1370 |
| out/k3-handback.txt | 613b94abcd00b986d2bfe982ca35c33febd8d2e5d231e337a5eb5526333772d6 |
| out/k3-hooks.txt | efe73b9d1da6e6a89c5a61659ff428be5d78484e81b20f44a89b247535c8c42e |
| out/naming-roots.txt | 7e36ff3d63f8fcde9d763c937cecf00fda86752d42655d0cfa917a0394d75f14 |
| out/k4-blind.txt | 42723cb9635224f19db7cc5a22f317fbd5b2c2921f8e530d6c10f19d8b842519 |
| out/k4-deferral.txt | 874d1e9b334974231b5338bcfa06fe54934e17bcd30163f20e057c6257006a6f |
| out/k4-outcome.txt | 576372639018ae843d87d1de53d5595bb795ddad02b3081d6baa8c8576e9c586 |

## 各格判定一览

| 格 | 判定 | 打中几条 | 一句话 |
|---|---|---|---|
| K2 合并与重合 | 打中 | 6 条（另 2 条低值） | 字面整段相同 0 段（逐字、引号归一两档都 0），重合在「换了变量名与引号的同一段判法」：基准版滞后表读法第 7、8 份，复跑表解析在同一道里第 4 份；滞后表本身是登记表 none: 行的镜像；派发闸 ④ 与 ⑰ 各读一半「报告」行；不变量清单路径写死，旁边已有 `.claude/kb-directories`；结局账的「交回」口径与续做闸、看门狗不同 |
| K3 矛盾 | 打中 | 9 条 | 三方正推腿骨架默认 `<草稿目录>/result.md` 能过派发闸、过不了交回闸（实测）；main-agent「一律 result.md」没给三方腿留例外；命名自查与执行员、工具员定义「交回前不跑门禁阶段」相反且定义优先；副本根上命名四格判不到；延后登记的三处与 CLAUDE.md 子 agent 欠账的去处相反；比对登记的写者与写范围对不上；另三条较弱 |
| K3 没打中的 | 没打中 | — | 命名自查两条命令照抄，四种类型 × 四道 Bash 钩子 64 格全放（对照格被拒）；全仓按名字要求子 agent 写 `report.md` 的 0 处 |
| K4 数 | 部分打中 | 4 条 | 2268、各桶、353 / 15.6% / 14.1% 在截止时刻上复得出到个位附近、复不出原样（没落产物、结果随此刻变）；记录第 42 行的 337 / 13.8% 写成五个词而五个词只有 156；37/42、33/42、6/8 复得出，「另多 3 次」里 2 次是 rework.md 归给 K10 的；8/54、16/129 算术对、复不出（输入目录已删） |

四句（evidence-discipline「打中之后先判是哪一种」）对这一轮的适用：被判的是治理改动不是实验臂，「分不分辨臂」一句逐条答成「打中的是哪一处改法、别的改法同样中不中」；「系统当时看不看得到」答成「钩子或定义在那一刻读得到的输入里有没有判别它的东西」。

## K2　能不能合并、和已有脚本有没有重合

### 现查的清单与整段相同

`python3 .claude/singlefs-ai-sop/scripts/gate-overlap.py --list`（退 0，58 行）里与这一批有关的行原样（说明列是脚本自己截的）：

```
门禁	.claude/gate.d/code-source-discipline.sh		代码类的源码纪律：名字不只由空泛词拼成、测试文件与 #[test] 函数不按里程碑、欠账号起名；kb 登记的格式常量、条文集合与 feature bit 位号与代码逐个对得上；实验二进制有成形的变异表且锚点唯一、有钉绝对值的断言、种子不折叠与读数不恒为字面量 0；crates/ 里放开 unsafe 的地方与登记表逐处对上；crates 变异表的轻型登记在改坏对象在 crates/singlefs-core/src/ 下的行里每个测试目标一行；不走 GPU 的计算与豁免表逐项对上；崩溃点重放的按段登记成形；代码里我们自己的名字不带序号与位置词
门禁	.claude/gate.d/code-tooling.sh		工具层自检（二十七格：研究脚本的自证都过、有阶段在跑；带判定库的脚本自证里复用三问都过；项目规则清单；阶段归属表按格登记；钩子注册与写范围表与定义头；改动范围只许一份取法；研究脚本与钩子的拒绝出路与 shell 纪律；阶段声称的样本与编造的日期；带编号登记位的样本都排在 doc-lint 之外；kb 登记表；门禁结构；环境检查排在跑测试的阶段之后；目录形态；提交里没有超过单文件上限的文件；阶段在 cd 之前取好自己的位置；并行度默认值只经每台默认重活线程数的登记位取；长活声明按现算读的脚本豁免表指得到、真要靠现算才齐、只缩不涨；函数单元两份实现对拍；heavy 与 heavy-if-set 的自证登记成偏离；变异门禁的 --list-items 在被判的仓上跑得通；门禁阶段的弄坏开关只在样本根上认；入口脚本带列项、自证与每项一行；弄坏开关文件头三样写全；门禁阶段的弄坏开关逐条复跑、判错的样本与头注点名的相等；研究脚本与钩子平常跑时设了弄坏开关要被拒；登记的脚本与钩子的弄坏开关逐条复跑、点名的项都在失败行里）
门禁	.claude/gate.d/doc-experiments.sh		实验页与产物（十三格：索引行与正文标题、产物写回、产物与依据落在仓里、决策与实验双向登记、判决行的 false 被点名、复跑命令、实验号有正文、整行抄的产物行、上一轮记录已归档、多路径互证的登记、点名的路径找得到、跑前登记的并行与挂钟、重跑登记的比对对象在单元表与复跑表里找得到）
门禁	.claude/gate.d/doc-process-records.sh		三方论证与阶段同步有没有留下该留的记录：批次范围、判决点名、正文前提、轮次收尾、同步记录、里程碑收口时的欠账
钩子	.claude/hooks/continuation-guard.sh	PreToolUse[SendMessage]	PreToolUse hook（SendMessage）：给一个已经交回过、或被中断过（撞限额、被停、报错）的子 agent 发消息之前拦一
钩子	.claude/hooks/handback-guard.sh	PreToolUse[SubagentHandback]	PreToolUse hook（SubagentHandback）：子 agent 交回之前拦四样——自己起的后台任务还在跑（全部子 age
钩子	.claude/hooks/heavy-test-guard.sh	PreToolUse[Bash]	PreToolUse hook（Bash）：重型测试只在提交代码时、或用户要求时跑，子 agent 只跑自己那一份；不合的当场拒绝（退出 2
钩子	.claude/hooks/runner-dispatch-guard.sh	PreToolUse[Agent|Task]	PreToolUse hook（Agent）：派发那一刻查派发提示齐不齐、写得对不对，缺一样当场拒（退出 2，stderr 写原因与出路）。
钩子	.claude/hooks/write-guard.sh	PreToolUse[Write|Edit|MultiEdit]	PreToolUse hook（Write、Edit、MultiEdit：.claude/settings.json 里注册在 matche
共 57 份：门禁 40、钩子 17
```

`gate-overlap.py` 判「整段相同」的那一半（不带 `--list`）被重型测试闸当「提交时才跑的检查」拒（攻方腿不跑）。我另写了一份照它取法的核对 `tools/clone_check.py`：去空行、注释、短行与只剩关键字的行，连续 8 行相同算一段（「CLONE_MINIMUM_LINES = 8」（`.claude/singlefs-ai-sop/scripts/gate-overlap.py:75`））；与它不同的三处是对照范围加上 `scripts/` 全部、同一份文件里不重叠的两处也比、另开一档把单引号换成双引号再比。它只比别的文件：「if other_path != path and other_path not in first_position_by_other:」（`.claude/singlefs-ai-sop/scripts/gate-overlap.py:727`），对照范围是门禁、钩子与上游包的脚本：「那一项的对照范围再加上本包 scripts/ 下其余的脚本，以及项目登记的门禁目录、钩子目录下的 .py 助手」（`.claude/singlefs-ai-sop/scripts/gate-overlap.py:31`），`scripts/agents/`、`scripts/kb/` 不在里面。

结果（`out/clone.txt` 原样）：两档都是「== 小计：0 段」，对照 387 份。核对自己会红：两份各 25 行的临时文件，一份把双引号换成单引号，逐字档 0 段、引号归一档报「scripts/b.py:2-26 与 scripts/a.py:1 起，连续 20 行相同」（草稿里跑的，命令与输出没进模型）。
所以这一批没有字面上的整段抄；下面几条都是「同一段判法换了变量名、换了引号、换了折行再写一遍」，`gate-overlap.py` 的逐行比对按设计看不见。

### K2-1　「基准那一版的滞后表」读法：这一批写了第 7、8 份

同一件事（取 gate 基准提交里那张表，分清「基准里没有这张表」与「git 取不到」，取不到判红）现在在七处各写一遍：

- 「listed = subprocess.run(['git', '-c', 'core.quotepath=false', 'ls-tree', '--name-only', base, '--', lag_path],」（`.claude/gate.d/doc-experiments.sh:2103`，multipath-registry 格）
- 「listed = subprocess.run(["git", "-c", "core.quotepath=false", "ls-tree", "--name-only", base, "--", lag_path], capture_output=True, text=True)」（`.claude/gate.d/doc-experiments.sh:2546`，新格 replay-comparison-units）
- 「listed = subprocess.run(["git", "-c", "core.quotepath=false", "ls-tree", "--name-only", base, "--", table], capture_output=True, text=True)」（`.claude/gate.d/doc-process-records.sh:848`）
- 「listed = subprocess.run(["git", "-c", "core.quotepath=false", "ls-tree", "--name-only", base, "--", lag_path], capture_output=True, text=True)」（`.claude/gate.d/code-source-discipline.sh:1591`），同一道另一处 `.claude/gate.d/code-source-discipline.sh:2097`
- bash 写的两处：「ls-tree --name-only "$base_name" -- .claude/heavy-thread-exempt」（`.claude/gate.d/code-tooling.sh:2747`）、「ls-tree --name-only "$base_name" -- .claude/long-run-live-hosts-exempt」（`.claude/gate.d/code-tooling.sh:3295`）
- 钩子库里按参数给基准表的一处：「基准里还没有这张表就不给 --base-table」（`.claude/hooks/lib_long_run.py:425`）

新格那 14 行与 multipath-registry 格那 14 行，把单引号换成双引号、去掉行首空白之后 `diff` 只差三处：变量名 `baseline_keys` 对 `baseline_numbers`、两行折行、最后取键的那一行（草稿里的 `diff` 命令，输出没进模型）。读表的小函数也是第三份：「def rows_of(text):」（`.claude/gate.d/doc-experiments.sh:2431`）与「def lag_rows(text):」（`.claude/gate.d/doc-experiments.sh:2032`）同一个语义（注释行与空行不算、按制表符切），`.claude/gate.d/doc-process-records.sh:817` 又有一份 `rows_of`。

- 为什么算重合：管同一件事（滞后表与基准比），同一个触发点（gate 基准 `shared_base`），共用逻辑没抽成库；`.claude/gate.d/` 已有 `lib-*.py` 的先例（`lib-item-ref-status.py` 等）。
- 并成什么样：加 `.claude/gate.d/lib-lag-baseline.py`，两个函数：`content_rows(text)` 与 `baseline_rows(base, path)`（返回「基准里的行或 None」与「取不到时的那一句问题」）；七处各自只留「拿哪几列当键、比出来怎么报」。删的：七处各自那 10 到 14 行的 `ls-tree` / `show` 与三份 `rows_of` / `lag_rows`；bash 那两处改成调这份库的命令行入口。
- 对缺口：这一条不碰「丢掉的那一格」的判据，并了之后缺口照样关住。
- 推翻条件：七处里有一处对「git 取不到基准」必须给出与别处不同的结局（例如当初次登记放过）。我核过的四处（multipath、新格、doc-process-records、code-source-discipline 那两处文件头）都写「git 取不到基准那一版判红」，code-tooling 的两处没逐行核。

### K2-2　新格的滞后表是登记表 none: 行的镜像，可以删掉

登记表里复跑写 none: 的行本身就带理由：「none:replay.sh 里没有复跑行，r2 直接建表的合法一族」（`.claude/gate.d/replay-comparison-units.tsv:16`）；滞后表同一件事再写一行：「r2 直接建表的合法一族，复跑表里没有这条正式跑的驱动行」（`.claude/gate.d/replay-comparison-units-lag.tsv:5`）。新格对滞后表判四样（形状、每行指到 none 行、每个 none 行都在滞后表、同一对不重复）全是在核两份是不是同一张表，只有「与基准比只缩不涨」一样带新信息：「只缩不涨：在 replay.sh 复跑表补了驱动行、登记表复跑列改写驱动名之后，同一次改动里删这里的行；不往这里加新行。」（`.claude/gate.d/replay-comparison-units-lag.tsv:3`）。

- 与 multipath-registry 那张滞后表不是一回事：那张表登记的是「还没写登记节的实验页」，这个信息别处没有；这张表登记的东西在登记表的复跑列里已经有了。两格判的对象不同（实验页里的表对机读登记），读的输入不同，不能并成一格；能共用的只有 K2-1 那一段。
- 并成什么样：删 `replay-comparison-units-lag.tsv`；「只缩不涨」改判登记表自己——此刻登记表里复跑写 none: 的（登记出处，留存产物名），基准那一版登记表里没有的判红。删的判法：滞后表形状、指到 none 行、none 行都登记、同一对不重复四样，开关 `comparison-none-lag-unchecked`；`comparison-lag-growth-unchecked` 改成不比登记表的基准；红样本里四种滞后表的红换成一种「none 行比基准多」。
- 对缺口：缺口的三样判据（登记出处在、单元名恰好一次、复跑驱动与产物名相符）不动，none 行照样只缩不涨，缺口照样关住；少一张要同步的表、少一处写者（K3-6 的写者问题也跟着小一半）。
- 推翻条件：用户或主 agent 要「none 行能不能加」由与登记表不同的人批（滞后表另有归属）。今天阶段归属表里这一格只有一行、整格同一组归属，没有这种分工。

### K2-3　复跑表 `scripts/experiment-apparatus/replay.sh` 在同一道 doc-experiments.sh 里被解析第四遍，取法不同

- 「REGISTERED="$(sed -nE 's/^E[0-9]+\|[^|]*\|[^|]*\|([^|]*\/)?([^|/]*)\|(exact|timing)$/\2/p' "$REPLAY_TABLE")"」这类 sed 取法在 `.claude/gate.d/doc-experiments.sh:334`（results-cited）；
- 「ROW_RE = re.compile(r'^(E[0-9]+)\|([^|]*)\|([^|]*)\|([^|]*)\|(exact|timing)$')」在 `.claude/gate.d/doc-experiments.sh:1459` 与 `:1822` 各一份（verdict-false-named、quoted-result-lines），两处注释都说与别的格「同一份形状」；
- 新格另写一份：「if len(fields) == 5 and re.fullmatch(r"[A-Z][0-9]+", fields[0]):」（`.claude/gate.d/doc-experiments.sh:2456`），只读 `TABLE=$(cat <<'TSV'` 那一段（`.claude/gate.d/doc-experiments.sh:2445`），实验号认任意大写字母开头、第 5 列不限 exact / timing。

今天两种取法在 `replay.sh` 上都取到 254 行、逐行相同（草稿里的一次 python 比对）；取法不同，`replay.sh` 加第二段同形的表、或第 5 列加新值时会分岔（推的，没造出这样的 `replay.sh`）。`scripts/gate/admission.py` 另有 `REPLAY_TABLE_START` 一份。

- 并成什么样：一个 `replay_rows(path)`（放进 K2-1 的那份库或 `lib-item-ref-status.py`）返回（实验号、第 2 列、参数、产物、方式），四格都调它；删新格的 `replay_table()` 与两处 `ROW_RE` 读表循环，sed 那一处改成调库。
- 对缺口：不变。推翻条件：四格对「哪些行算复跑行」确有不同的需要（例如新格要认 TABLE 段之外的行）。

### K2-4　派发闸 ④ 与新判法 ⑰ 各读一半「报告」行，标签集合互不相交

④ 认的标签：「SCOPE_MENTION = re.compile(r"(?:报告路径|报告写进|报告写到|写进|写到|更新)」（`.claude/hooks/runner-dispatch-guard.sh:593`），没有裸的「报告：」；⑰ 只认「报告：」：「REPORT_LABEL_LINE = re.compile(LINE_START + r"报告[* \t]*[：:][* \t]*(.*?)[ \t]*$", re.M)」（`.claude/hooks/runner-dispatch-guard.sh:1214`）。骨架打的正是「报告：」一行，所以骨架的报告路径从来不过 ④。三条提示经 `dispatch-prompt.py --check` 实测（`out/k2-report-labels.txt` 原样）：

```
== probe-writer：报告：research/prompts/probe-impl-result.md
  ✓ 派发闸照这份提示放行（implementation-writer，查了 21 行提示）；…
check-rc=0
== probe-writer-control：报告写进 research/prompts/probe-impl-result.md
✗ 提示要 implementation-writer 写进它写范围之外的路径：research/prompts/probe-impl-result.md（.claude/hooks/agent-write-scope.tsv）。
check-rc=3
== probe-clerk：报告路径：…/probe-clerk/report.md
  ✓ 派发闸照这份提示放行（clerk，查了 18 行提示）；…
check-rc=0
```

（`…` 是我截的长行尾巴与绝对路径前缀，原文在 `out/k2-report-labels.txt`。）实现员的报告写到 `research/prompts/` 下，用「报告写进」被 ④ 拒、用「报告：」放行；clerk 的报告名是 `report.md`，用「报告路径：」被 ⑰ 放过。⑰ 自己的样本把后一种钉成放行（「报告名:「报告路径：」一行不是这一种标签，不判」那一格）。

- 为什么算重合：同一个触发点（Agent 派发）、同一批对象（提示里点名报告文件的那一行），两个判法各写一份行解析，标签切得互补，两边都漏。
- 并成什么样：一个报告行解析，标签认「报告」「报告路径」「报告写进」「报告写到」四种，取出的路径同时交给 ④（写范围）与 ⑰（文件名）；删 ⑰ 的 `REPORT_LABEL_LINE` 与 `SCOPE_MENTION` 里的三个报告标签；⑰ 那一格样本改成「报告路径：…/report.md 拒」。
- 对缺口：报告文件名那条缺口原来只在「报告：」一种写法上关住，并了之后四种写法都关住；④ 顺带补上骨架报告行不判写范围的洞（这个洞早于这一批）。
- 推翻条件：有派发提示要在「报告路径：」后面写一个不归子 agent 写的路径（只读参照），并了之后会被 ④ 误拒——我在 `.claude/agents/` 的「输入」各节里没找到这种用法。

### K2-5　候选扫描把不变量清单的路径写死，旁边已有登记位 `.claude/kb-directories`

新加的那一行：「r'^crates/singlefs-checker/invariants/[^/]+\.md$',」（`scripts/kb/stale-candidates.py:97`）。仓里已有一份登记：「crates/singlefs-checker/invariants  # 不变量清单按类拆开挪到这里」（`.claude/kb-directories:2`），已经在读它的有「KB_DIRECTORIES_REGISTRY = '.claude/kb-directories'」（`scripts/kb/quote-kb.py:435`）、「if os.path.isfile('.claude/kb-directories'):」（`.claude/gate.d/doc-experiments.sh:2196`，cited-paths-exist 格）与 `scripts/agents/three-way-materials.py:374`。

- 为什么算重合：同一件事（kb 内容住在哪几个目录）两处登记，候选扫描不读已有的那一份。
- 并成什么样：`CARRIER_PATTERNS` 里 `.claude/kb/` 与不变量清单这两行换成「`.claude/kb-directories` 登记的每个目录下的 `.md`」；开关 `invariants-carrier-missing` 改成「不读登记位、退回写死的 `.claude/kb/`」。
- 对缺口：调度记录写的病根是「候选扫描与写回员写范围都没跟着搬家改」。写死一行只关住这一次搬家；下一次搬家照 `.claude/kb-directories` 的说法登记（它第 1 行写「登记是替换不是追加」），候选扫描照旧漏，缺口复发。读登记位才关得住扫描这一侧；写范围那一侧（`.claude/hooks/agent-write-scope.tsv` 的写回员行）仍是写死的模式，要关住得在 code-tooling 加一格「登记位里每个目录在写回员写范围里有一行」（推的，没写）。
- 推翻条件：`.claude/kb-directories` 有意不含某些现状载体、或含了不该进候选的目录（例如只为不变量类号查找而登）。它两行今天都是现状载体。

### K2-6　结局账的「交回」与「起点」口径，和续做闸、看门狗、同一份脚本里的另三个子命令各不相同

- 新的 `outcome` 只看调没调过交回工具：「if name == "SubagentHandback":」（`scripts/agents/agent-cost.py:654`），调用被交回闸拒了也算交回。续做闸要送达：「"""子 agent 的会话记录里有没有一次送达的交回：assistant 调过 SubagentHandback，且后面有它对应、不是错误的 tool_result。"""」（`.claude/hooks/continuation-guard.sh:133`）；看门狗同样：「「交回」按交回工具（SubagentHandback）成功返回判」——这一句里有内层引号，cite-check 认不了，行号是 `scripts/agents/agent-watch.py:27`。
- 同一份 `agent-cost.py` 里，原有三个子命令按文件修改时刻选子 agent（`scripts/agents/agent-cost.py:160` 的 `recent(transcript, since)`），`outcome` 再按首条记录筛（「if not scan["start"] or scan["start"] < since:」（`scripts/agents/agent-cost.py:963`））。`--since 2026-10-01` 时前者 2290 个、后者 2287 个，差 3 个（草稿里一段读首行时间戳的 python，量的，没进模型）。
- 量到的差：`tools/outcome_check.py` 照续做闸的认法另写一份「送达」，在全部 2287 个上数「outcome 当成交回、交回一次都没送达的」：「outcome 当成交回、交回一次都没送达的 0 个，按桶：；按类型：」（`research/prompts/landing-audit-r1-opus-model/out/k4-outcome.txt:13`）。认法自己会红：拿一份交回先被拒、后来送达的真会话记录，整份判送达、截到第一次被拒那一行判没送达（草稿里跑的）。会话目录里带「PreToolUse:SubagentHandback」拒绝的子 agent 会话 723 份，全都后来送达了。
- 判：口径重了三份、彼此不一致，今天数上没差；交回闸拒的多（交回闸文件头写 2000 字时 8 天拒 67 次），哪天出现「被拒之后被停」的子 agent，结局账会把它记成交回、续做闸与看门狗记成没交回。
- 并成什么样：`scripts/agents/` 下加一份子 agent 会话记录库（`handback_delivered`、按首条记录取起点），`agent-cost.py` 的 `read_transcript`、`scan_subagent` 与 `continuation-guard.sh`、`agent-watch.py` 都从它取；删 `scan_subagent` 里只认调用的那一支与 `subagent_files` 的修改时刻筛选。读会话记录的主循环（`scan_subagent` 为快只解析部分行，`read_transcript` 逐行解析）不必合：两边要的字段不同。
- 对缺口：「结局没有常设的账」那条缺口照样关住；并了之后账上的「交回 / 没交回」与看门狗、续做闸同一个定义。推翻条件：主 agent 有意要「试过交回」这个口径（例如数交回闸的拦截量）——那该另起一个字段，不顶「交回」这个名字。

### K2-7、K2-8（低值，写明不必并）

- K2-7：命名自查放行与广谱变异 gate-verdict 放行是同一种形状——「and checked_cells(arguments) == {BROAD_MUTATION_READ_ONLY_CELL}):」（`.claude/hooks/lib_heavy_tests.py:1603`）与「if naming_cells and (break_is_set("naming-self-check-any-cell") or naming_cells <= NAMING_SELF_CHECK_CELLS):」（`.claude/hooks/lib_heavy_tests.py:1607`）。可以并成一张「阶段 → 只读格集合」的表、一个分支；一个是等于、一个是子集，并了要带上比法；两处各自的弄坏开关要一起改。值不大，不并也不出错。
- K2-8：`git worktree list --porcelain` 的解析三份：`scripts/kb/claim-experiment.sh:35`、`scripts/agents/agent-cost.py:877`（都是这一批加的）、`scripts/hosts/clean-session-scratch.py:141`；各两三行、一份 bash 两份 python，抽库不划算。另一处更值得看：占号查「哪儿出现过这个号」的目录表（`scripts/kb/claim-experiment.sh:53` 那一组 `ls`）与 experiment-orphans 格的「SCOPE_DIRS=(research scripts/experiment-apparatus scripts/mutation/tables crates/singlefs-checker-tier/src/bin)」（`.claude/gate.d/doc-experiments.sh:1674`）不是同一份：占号不看 `crates/singlefs-checker-tier/src/bin/` 下的入库装置 `e<号>_*.rs`，孤儿格不看 kb。另一棵工作树里只有入库装置、还没写登记与实验页的号，占号查不到（推的，没造这样的工作树）。并法：一份「实验号住在哪」的目录登记，两边读。对缺口（K9 跨工作树撞号）：入库装置一般与登记同批出现，漏的窗口小。

### K2 没打中的

- 新判法 ⑰ 与写闸、交回闸「管报告文件」的部分：写闸里没有按文件名判报告的判法（`grep -ni report .claude/hooks/write-guard.sh` 去掉样本里的 `result.md` 之外 0 行），交回闸只认路径落在哪个根、不认文件名；拦 `report.md` 的是工具层自带的那一道。不算同一件事判两处。
- 新格与 multipath-registry 能不能并成一格：不能（K2-2 第一条）。
- `stage-owners.tsv`、`selftest-registry.tsv` 加的行：一格一行、一份脚本一行，与已有行不重。

## K3　有没有矛盾

### K3-1　三方正推腿的骨架默认报告路径：派发闸放行、交回闸拒（主 agent 派之前看到的那一处）

- 骨架现在给「报告：」一行填默认值：「default = f"{draft.rstrip('/')}/{REPORT_FILE_NAME}" if draft else f"<填 草稿目录，与上一行一致>/{REPORT_FILE_NAME}"」（`scripts/agents/dispatch-prompt.py:241`）。改之前是占位，主 agent 必须自己填：「-        wanted = ["重型测试：不跑", "批次：<填 调度记录#批号>", "草稿目录：~/.local/share/singlefs/work/x/", "报告：<填 报告|报告路径>", "条款：<填 条款>", "## 出口"]」（`research/prompts/landing-audit-r1-snapshot/judged-vs-index.diff:3062`）。
- 三方腿的报告是另一种：「报告路径（形态 `research/prompts/<轮>-sonnet-output.md`）」在 `.claude/agents/three-way-forward.md:22`；「云端腿按提示先把报告写进 `research/prompts/*-output.md`、最后才回复。」（`.claude/rules/three-way-inference.md:158`）；交回闸只认这种：「return ["交回里没点名报告文件（research/prompts/<轮>-…-output.md）"]」（`.claude/hooks/handback-guard.sh:119`）。
- 实测一条链（`out/k3-dispatch.txt`、`out/k3-handback.txt` 原样）：照骨架打 three-way-forward，只填占位、不动「报告：」那一行（`/…/probe-forward/result.md`），`dispatch-prompt.py --check` 判「✓ 派发闸照这份提示放行（three-way-forward，查了 20 行提示）」、`dispatch-check-rc=0`；腿照提示交回 `报告 …/probe-forward/result.md`，交回闸退 2：「交回里没点名报告文件（research/prompts/<轮>-…-output.md）」、`handback-rc=2`。派发闸放过去的提示，腿照做必在交回时被拒。
- 是不是这一批带出来的：是。改之前骨架不填，`--check` 因占位退 3，主 agent 必须照定义填；这一批的骨架改动让错的路径成了默认、且能过 `--check`。别的类型：类型定义 frontmatter 里带「报告」的只有 clerk、experiment-runner、implementation-writer、investigator、kb-scribe、mutation-triage、prior-art、sweep、tooling-writer、three-way-forward 十种，前九种的定义对报告只写「报告路径与草稿目录」或要它落在工作根（例 `.claude/agents/investigator.md:26`），没有别的形态，默认值对得上；三方腿里只有正推腿 frontmatter 带「报告」（「required-inputs: 禁读清单, 草稿目录, 背景材料, 报告」（`.claude/agents/three-way-forward.md:8`）），攻方、辩方没带（「required-inputs: 禁读清单, 草稿目录, 背景材料, 攻击面」（`.claude/agents/three-way-attack.md:8`）），骨架里根本没有「报告：」一行，要主 agent 自己加——这一处早于这一批，不是这一批带出来的，但同样没人判。
- 四句：分不分辨——只打中三方腿，别的九种类型的默认对；系统看得见判别它的东西——派发闸 ⑰ 那一刻读得到类型（three-way-forward）与路径，却只判文件名不判形态；字面分句——⑰「报告：」一行；改法——见下。
- 改法：每个类型的报告路径形态写进定义 frontmatter（例 `report-path: research/prompts/<轮>-sonnet-output.md`），骨架照它填默认、三方腿填 `research/prompts/<填 轮>-<模型>-output.md` 占位，派发闸 ④⑰ 并成的那一处（K2-4）按它判形态。只在骨架里给三方腿留占位也行，但派发闸照旧放过「报告：<草稿目录>/result.md」的三方腿提示。
- 对缺口（报告写法矛盾）：这条缺口要关的是「派发写的报告名与工具层、交回闸对不上」，改名到 `result.md` 时把三方腿一起带偏，等于在三方腿上新开了同一种矛盾（这一次对不上的是交回闸）；照上面的改法改了之后两头都关住。
- 推翻条件：交回闸把三方腿「草稿目录下 result.md」也认作报告（今天的 `LEG_REPORT_MENTION` 只认 `research/prompts/…-output.md`）；或三方腿的定义改成报告写在草稿目录。

### K3-2　main-agent「报告路径一律 result.md」没给三方腿留例外

「派发提示里的报告路径，文件名一律写 `result.md`（例：`<草稿目录>/result.md`），不用名字里带 report 的文件：工具层拒子 agent 用 Write 写这类文件。」（`.claude/main-agent.md:86`）。三方腿的报告名是 `<轮>-<模型>-output.md`（上一条三处出处）。这一轮主 agent 自己派这条腿时写的就是「报告：research/prompts/landing-audit-r1-opus-output.md」，照字面违反第 86 行、照三方规则是对的。共用约束那一句放在「其余定义」之后：「报告文件名一律 `result.md`，不用名字里带 report 的文件」（`.claude/agent-common.md:108`），前文刚写「三方论证的云端腿，报告就是派发提示给的 `research/prompts/<轮>-*-output.md`」（`.claude/agent-common.md:108`），同一行里读得出例外，主 agent 那一行读不出。

- 改法：main-agent 那一句改成「名字里不带 report；没有别的形态要求的写 `<草稿目录>/result.md`，三方腿照 three-way-inference 写 `research/prompts/<轮>-*-output.md`」。对缺口：「报告写法矛盾」这条缺口要的是不撞工具层的拦截，`*-output.md` 不撞，改了照样关住。
- 推翻条件：主 agent 那一句有意只管非三方腿（那就该写出来，今天的字面是「一律」）。

### K3-3　「命名检查交回前跑」与执行员、工具员定义的「交回前不跑门禁阶段」相反，而定义优先

- 共用约束新写：「改了代码或实验源码（`crates/`、`scripts/` 下的 `.rs`）的子 agent（`implementation-writer`、`experiment-runner`、`tooling-writer`）交回前」（`.claude/agent-common.md:97`）跑 code-source-discipline 的四格；同一节把门禁一句改成「命名检查除外」（`.claude/agent-common.md:96`）。
- 两份定义没跟着改：「7. 交回前不跑门禁阶段与各类 lint；」（`.claude/agents/tooling-writer.md:36`）；「6. 门禁阶段交回前不跑，由提交前主 agent」（`.claude/agents/experiment-runner.md:45`），它固定的「没做什么」还写「没跑门禁阶段」（`.claude/agents/experiment-runner.md:62`）。code-source-discipline 那四格是门禁阶段的格。
- 冲突怎么判由共用约束自己定：「`.claude/agents/` 下每个定义开工前先读这一份；与定义冲突时以定义为准。」（`.claude/agent-common.md:3`）。照字面，执行员与工具员不跑命名检查——这条改法在三种点名的类型里有两种不起作用。
- 对缺口：K1（命名 lint 交回前不跑）的三次全是执行员（rework.md 的 K1 行：E187 #2、E185 #6 #7），正好落在被定义盖掉的那一种，缺口没关住。
- 改法：两份定义那一步各加「命名检查除外（共用约束「门禁」一节）」，执行员固定的「没做什么」那一行改成「没跑命名检查之外的门禁阶段」。
- 推翻条件：主 agent 认为「门禁阶段」不含「只点名四格的 `--check`」——那要在共用约束里写明，今天的字面没有这个区分；重型测试闸放行它也是按「阶段的格」认的。

### K3-4　「在副本根上跑」与「副本只拷要编的」：最小副本上四格判不到

共用约束让在「在自己改动所在的树（副本根；在主工作区里改的写仓根）上跑两条」（`.claude/agent-common.md:97`），同一份的建副本规矩是「副本只拷要编的（`crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default`，要门禁表的再加 `.claude/gate.d`）」（`.claude/agent-common.md:46`）。四格读的词表在 `<根>/.claude/` 下：「WORDS_FILE = ".claude/naming-vague-words"」（`.claude/gate.d/code-source-discipline.sh:294`），ordinal-names 还只扫 git 列出的文件（`.claude/gate.d/code-source-discipline.sh:46`）。`tools/naming_copy_probe.sh` 在三种根上照抄那两条命令（`out/naming-roots.txt` 原样摘）：

```
== 根：copy-root
  ✗ 读不到空泛词表 .claude/naming-vague-words
  ✗ 读不到序号与位置词表 .claude/naming-ordinal-tokens
  ✗ 数据形态的名字不合命名纪律：735 处违规、排除表 0 处不合规（扫了 131 份文件、10080 个串）
  ✗ 2823 处名字不合命名纪律（查了 599 个 .rs 文件、154142 个声明的名字）
== 根：copy-root-wordlists
  ! 本次无对象可判：crates/ 与 scripts/experiment-apparatus/ 下一份 .rs 都没列出来
  ✗ git ls-files 退 128：fatal: not a git repository (or any of the parent directories): .git
  ✗ 数据形态的名字不合命名纪律：29 处违规、排除表 2257 处不合规（扫了 131 份文件、10080 个串）
== 根：/home/fy5090/code/singlefs
  ✗ 11 个函数名或类型名只由空泛词拼成（每个词都在 .claude/naming-vague-words 里）
  ✗ 2 处名字带序号或位置词（词表 .claude/naming-ordinal-tokens，按词边界认，复合名也算）
  ✗ 数据形态的名字不合命名纪律：742 处违规、排除表 0 处不合规（扫了 1046 份文件、33990 个串）
  ✗ 3448 处名字不合命名纪律（查了 765 个 .rs 文件、224191 个声明的名字）
```

最小副本（rsync、不带 `.git`，证红那种副本就是这样建的）上 vague-names 与 ordinal-names 一个名字都没判；补上词表也只换成「无对象可判」与 git 报错。仓根上四格与 naming-lint 一共 4203 处存量红，交回的子 agent 要自己从里面挑出自己改的文件。

- 对缺口：在主工作区改的（执行员多半如此）不受影响；交补丁、证红副本上改的实现员，这条改法判不到 vague / ordinal。git worktree 建的副本应当判得到（推的，没建 worktree 量）。
- 改法：命令的根写死成仓根，用 `--files <自己改的文件>` 一类的参数只看自己的文件（四格今天没有这个参数，要加）；或把「副本根」改成「git worktree 的根，rsync 副本不算」。
- 推翻条件：四格对缺词表、非 git 根有回落（今天没有，上面的输出是现跑的）。

### K3-5　「延后的当时就登记」只列三处登记位，与 CLAUDE.md 子 agent 欠账的去处相反

- 新句：「同一步在里程碑收口表、`.claude/kb/checks-owed.md` 或决策正文里登一行」，又「只写进调度记录不算延后，算没去向。」（`.claude/main-agent.md:21`）。
- CLAUDE.md：「subagent 与协作工具的欠账记在 `records/2026-09-16-subagent拆分提案.md`，不占 `.claude/kb/` 的 C / D / E 编号，kb 只管文件系统本身。」（`./CLAUDE.md:22`）。
- 这一批自己就是这么延的：「子 agent 与协作工具的五条：`records/2026-09-16-subagent拆分提案.md` 第九十节 ①到⑤」（`records/2026-10-22-子agent结果落地审计.md:142`）。照第 21 行的字面这五条「算没去向」；照 CLAUDE.md 第 22 行它们不许进 `checks-owed.md`。第 7 条回看的也只是那三处（`.claude/main-agent.md:26`）。handoff-gaps.md 在判去向时自己另加了「常设清单」一类，等于承认三处不够。
- 四句：打中的是第 2 条的字面射程（漏了第四处登记位），不分辨别的改法；主 agent 在登记那一刻看得到延后项是不是子 agent / 协作工具的事。
- 改法：第 2 条与第 7 条的登记位列表加「子 agent 与协作工具的事记进 `records/2026-09-16-subagent拆分提案.md` 的编号小节（CLAUDE.md「经验与约定写在哪」）」；九十节 ③ 要立的「延后项带编号的格」照四处认。对缺口：「延后项没进欠账表」要关的是「没去向」，第四处也是有去向，加上之后照样关住。
- 推翻条件：用户定过子 agent 欠账也要进 `checks-owed.md`（那 CLAUDE.md 第 22 行要改）。

### K3-6　重跑比对登记：归属表说设计员加行，设计员写不了；写范围表说定义会写，执行员定义没写

- 阶段归属表：「登记行由写重跑登记的设计员与执行员加」（`.claude/gate.d/stage-owners.tsv:106`）；新格文件头也说同一次改动里登记（`.claude/gate.d/doc-experiments.sh:2395`）。设计员的写范围：「只写 `research/prompts/e<号>-preregistration.md`（由 `claim-experiment.sh` 排他新建」（`.claude/agents/experiment-designer.md:61`），写范围表里也没有设计员写 `.claude/gate.d/replay-comparison-units.tsv` 的行。设计员照归属表去加行，写闸当场拒。
- 写范围表给执行员加了一行「重跑比对对象登记：执行员实现单元时往里登记行」（`.claude/hooks/agent-write-scope.tsv:20`），同一行出处那一列写着定义的写范围由第 11 号任务补；第 11 号任务的 diff 里执行员定义的写范围没动，今天那一段（`.claude/agents/experiment-runner.md:52`）没有这份表，执行员的步骤里也没有一步说什么时候往里登记。表头声称「只登记 tools 里有 Write 或 Edit 的定义；表与定义逐项一致由 code-tooling.sh 的 agent-write-scope 格判。」（`.claude/hooks/agent-write-scope.tsv:7`），那一格实际只判到类型名一级：「③ tools 含 Write 或 Edit 的每个定义，在 .claude/hooks/agent-write-scope.tsv 里至少一条路径模式；④ 表里每个 agent 名都有定义，每行至少两列；」（`.claude/gate.d/code-tooling.sh:1040`），这一处不一致门禁看不见。
- 对缺口（「丢掉的那一格」）：格本身立住了，但「新的重跑登记谁往表里登」没有一个写得了、又被定义告知要写的人，新登记漏登机读表时格看不见（格自己的「管不到」一段也这么写）。缺口只关住了 E165 这 14 行。
- 改法：二选一写死——登记行归设计员（写范围表加一行、设计员第 4 步「差分重跑登记」加一句）；或归执行员（执行员第 5 步登记复跑那一步加一句、写范围一段补这份表），归属表跟着只留一种。
- 推翻条件：主 agent 打算自己登记这张表（那归属表与写范围表那两行都该删）。

### K3-7　设计员 3a「预期的差」「会发生的结论」与「实验开跑之前，答案不许已经存在」

- 3a 要设计员在登记写死之前写出「② 每个对照点（阳性对照、几何敏感性的反向取样点、每个判据比较的双方）写出它在被测臂上预期产生多大的差」与「③ 登记点上要触发的事件（搬空发布、回收、淘汰、写满这类）写出触发条件，并用登记的几何代进去算，写出算式和它会发生的结论」（`.claude/agents/experiment-designer.md:33`）；分辨率分不出这个差的，改判据。
- 规则：「**不许预设结论，设计阶段也不许参考任何已有结论**：这个项目上一轮的结论、」（`.claude/singlefs-ai-sop/rules/test-discipline.md:26`）「别处的定论、自己的直觉，一概不看。」（`.claude/singlefs-ai-sop/rules/test-discipline.md:27`）；「判据、阈值、作废条款都在跑之前写死，写完再去看已有结论。」（`.claude/singlefs-ai-sop/rules/test-discipline.md:33`）。设计员自己的第 2 步也要它把「判问法时自己算出了答案，列进登记固定的一节」（`.claude/agents/experiment-designer.md:31`），3a 却叫它把算出的差写进第六节与第十四节。
- 判：「判据比较的双方」在被测臂上的预期差，就是这个判据要量的答案；先算出它、再按它改判据，是规则禁的「先有答案再定判据」。③ 里「它会发生的结论」在触发事件本身就是被问的东西时（例「回收会不会在这个几何上发生」）同样是答案。可跑性（态数、挂钟、内存）那一半不冲突。
- 改法：② 改成「每个判据要分辨的差取岔路单那一行的翻面观测（量出什么数会让选择翻面），写测量手段分不分得出它」——翻面观测是问题定义，「岔路单只写问题与候选定义，不写倾向、不写已有的数。」（`.claude/rules/three-way-inference.md:140`）；③ 只对「触发是前提、不是被问的量」的事件做，算出的结论照第 2 步列进「跑之前已经存在的数」。对缺口（K3 设计员不核可跑性与判别力）：判别力照样核，只是核的对象从「预期答案」换成「要分辨的差」，缺口照样关住。
- 推翻条件：test-discipline 那一条只管「读已有结论」、不管「自己现算」（它的字面写了「自己的直觉，一概不看」，我按包含现算读）。

### K3-8（弱）　执行员 3c 的证红要求与入库装置那一支不许执行员跑 checker 档测试

3c：一行里的用例「用例要靠改坏该函数一处证红的，照」show-me-test 办（`.claude/agents/experiment-runner.md:40`）；入库装置那一支（装置在 `crates/singlefs-checker-tier/src/bin/`）：「执行员不对它起 `cargo test`（checker 档，重型闸拒）」（`.claude/agents/experiment-runner.md:32`）。入库装置的用例要证红只能交主 agent 跑，3c 没写这一支怎么办，照字面执行员只能「写不出来，停下交回」，每次都多一派。改法：3c 加一句「入库装置那一支的证红写进『要主 agent 跑的命令』，清单那一格写待主 agent 证红」。推翻条件：入库装置的用例全在 `scripts/experiment-apparatus` 那边有对拍用例可证（我没逐个查）。

### K3-9（弱）　回扫「交实现员」并进「下一批」与主 agent 第 1 条的截止批次、第 4 条的「条款已定」

回扫员把 `crates/` 行交出去，「的行并进下一批交 `implementation-writer`。」（`.claude/agents/sweep.md:52`）；主 agent 第 1 条「批内冒出的发现只进下一轮的清单，不当场派。」（`.claude/main-agent.md:20`），第 4 条聚簇的前提是「把这一轮已抓到、条款已定、要改 `crates/` 的改动全部列出来」（`.claude/main-agent.md:23`）。阶段同步排在一批的末尾，常常就是截止批次，「下一批」在这一轮里不存在；回扫员判的「要改」（尤其测试里的钉值）是它的判断，不是定了的条款。两处可以同时读成对的（下一批 = 下一轮的第一批），但字面不对齐，调度记录里「回扫判出的 `crates/` 行没人接」那条缺口正是这样漏的。改法：sweep 与主 agent 派发表那一行都写「进下一轮的清单」，并说明钉值类的行按第 4 条先定条款再派。推翻条件：主 agent 定过阶段同步的交实现员行不受截止批次限。

### K3 另一处（与 K2-4 同一个根）：⑰ 放行「报告路径：…/report.md」与「一律不用名字里带 report」

共用约束（`.claude/agent-common.md:108`）与主 agent（`.claude/main-agent.md:86`）说一律不用，⑰ 只判「报告：」一行，它的样本把「报告路径：…/report.md」钉成放行；实测见 K2-4。改法同 K2-4。

### K3 没打中的

- 命名自查两条命令照抄能不能过钩子：`tools/hook_probe.sh` 把共用约束第 97 行的两条命令（各带不带 `nice -n 19` 两种写法）按 implementation-writer、experiment-runner、tooling-writer、three-way-attack 四种类型喂给 heavy-test-guard、run-scope-guard、two-logs-guard、bash-command-detector，64 格全退 0；对照「夹着 format-constants」那一条重型测试闸退 2（`out/k3-hooks.txt` 末行「对照（夹着 format-constants）heavy-test-guard rc=2」）。不被拒；两条命令都没有 `.log` 重定向，two-logs-guard 不管。
- 全仓仍要求或按名字认 `report.md` 的地方：
  - `grep -rn --exclude-dir=target --exclude-dir=.git 'report\.md' .` 共 7909 行（含 `xxx-report.md` 这类名字）：research/prompts 5738、`.claude/kb` 522、records 合计 1515、research/results 31、`.claude/hooks` 19、`.claude/agent-def-review-exempt` 17、crates 31、`.claude/gate.d` 12、scripts 22、`.claude/warnings` 1、`.claude/heavy-thread-exempt` 1（按目录数，各数是那一条命令的输出用 `uniq -c` 数的，records 是各份合计）。
  - 裸名 `report.md`（前面是路径分隔、空白、引号或反引号）：`grep -rnE '(^|[/`"'"'"' （「(])report\.md'` 共 3973 行；落在定义、共用约束、主 agent、规则、技能、CLAUDE.md 里的 0 行。
  - 落在钩子、门禁、脚本、crates 里的 47 行逐行看过：派发闸 ⑰ 本身与它的样本 15 行、骨架的说明与样本 4 行（都是为了拒它）；evidence-in-repo 样本里写的 `/tmp/…/report.md` 11 行；旧报告的出处注释 10 行（crates 3、`replay.sh` 4、e142 装置 1、`verus-pins.tsv` 1、`doc-consolidate.py` 1）；test-environment-check 自证里它自己的输出文件名 1 行；clean-session-scratch 自证里造的「草稿目录里的报告不碰」3 行；`agent-cost.py` 新写的结局账自证 3 行用 `report.md` 造历史会话（造的是改名之前的会话，对）。没有一处要求子 agent 把报告写成 `report.md`，也没有一处按这个名字找报告（`apply-writer-patch.py` 已改认 `result.md`）。
- `.claude/main-agent.md` 派发表写回员「不改 `crates/`」与不变量清单例外：派发表那一行已经写明例外，写回员定义的写范围与输入两处、写范围表一行、写闸样本四格一致；没找到别处写「只有实现员改 `crates/`」。

---

## K4　数与读数能不能核

### K4-1　结局账 2268 个、各桶、返工字样 353：口径复得出，原样复不出

记录写的：「查了 2268 个子 agent，折算合计 7623.33 百万」（`records/2026-10-22-子agent结果落地审计.md:138`），同一行各桶 888 / 49.8%、90 / 4.3%、473 / 27.5%、661 / 15.3%、91 / 2.0%、其余 65 / 1.2%，返工字样 353 个、15.6%、14.1%，没交回 116（36、2、73、5）。

照记录那条命令，脚本拷进草稿目录跑（`agent-cost.py` 拷贝的 sha256 与开工快照里那一份相同，`8ab2757e…`）。拷贝不在仓里时默认会话目录推错，要加 `--root ~/.claude/projects/<项目>`（不加退 2：「会话目录不在」），别的照抄。`--summary` 原样：「## 按结局桶（2026-10-01 起，2287 个，折算合计 7713.93 百万）」，五行 897 / 50.4%、90 / 4.0%、474 / 27.2%、667 / 15.2%、92 / 2.0%、其余 67 / 1.3%（草稿 `outcome-summary-run2.txt`）。

`tools/outcome_check.py` 只留首条记录不晚于某个时刻的子 agent，复出最近的一组（`out/k4-outcome.txt` 第 16 至 26 行，截止取 `tw-outcome-ledger.md` 的修改时刻 unix 1792668941）：

| 量 | 记录 | 复算（截止那一刻之前开工的） | 此刻全部 |
|---|---|---|---|
| 子 agent | 2268 | 2269 | 2287 |
| 折算合计（百万） | 7623.33 | 7654.44 | 7723.82 |
| 写了、不低于 70% | 888 / 49.8% | 894 / 50.5% | 899 / 50.4% |
| 写了、30% 到 70% | 90 / 4.3% | 90 / 4.1% | 91 / 4.1% |
| 三方腿 | 473 / 27.5% | 473 / 27.3% | 475 / 27.2% |
| 只交报告、主 agent 点到 | 661 / 15.3% | 661 / 15.2% | 667 / 15.2% |
| 看起来没效果的几桶 | 91 / 2.0% | 91 / 2.0% | 92 / 2.0% |
| 其余 | 65 / 1.2% | 60 / 0.9% | 63 / 1.1% |
| 返工字样 | 353、15.6%、14.1% | 353、15.6%、14.1% | 354、15.5%、14.1% |
| 没交回 | 116（36、2、73、5） | 111（36、2、73、0） | 114（36、2、73、3） |

- 复不出原样的原因，三样都是命令本身的性质：只有下界 `--since`、没有上界，后来开工的子 agent 进来；「还在跑」按此刻 30 分钟算，当时在跑的后来结束、花费继续涨（合计 7623 → 7654）；落地按 git 历史与工作树现比，后来提交的行让当时的「写了」桶升级（888 → 894）。
- 记录没留这次跑的输出：调度记录第 138 行只写命令与数，没点名产物文件；第三批的那次（2268 个、7617.31）也只留在草稿里：「没把真跑的 2268 行账存进」（`research/prompts/subagent-landing-audit-2026-10-22/tw-outcome-ledger.md:150`），那个目录今天已经不在（`ls ~/.local/share/singlefs/work/landing-audit-tw-outcome-ledger/` 报不存在）。共用约束要「跑出来的产物先落盘进仓里该在的位置」（`.claude/agent-common.md:107`）。
- 判：记录的数是某一刻的快照，对得上到个位附近，但没有一份产物能逐字核；推翻条件：找到那次 `--summary` 的原样输出且与第 138 行逐字相同。

### K4-2　返工字样：第 42 行的 337 / 13.8% 还写着五个词，严格词表里有单字「修」

- 第 42 行：「的派发 337 个，占花费 13.8%」（`records/2026-10-22-子agent结果落地审计.md:42`），同一句把词写成「修、重跑、续做、改名、改号」五个。第 137 行说 337 其实是宽词表数的，第 138 行说新数「取代按结局分那张表」，第 42 行那句不在表里，没改也没注明。按五个词复算是 156 个、6.9%、5.4%（`out/k4-outcome.txt` 第 25 行）。这是第 42 行自己的口径错，推翻条件：第 42 行改过或加了注。
- 353 用的「严格词表」：「REWORK_WORDS = re.compile(r"修|重跑」（`scripts/agents/agent-cost.py:341`）含单字「修」。此刻 354 个命中里只靠一个词命中的，fix 94、修 77 最多（`out/k4-outcome.txt` 第 14 行）；只靠「修」的 77 个里「修订」11 个（例「E177 跑前登记修订」「E175 登记修订 2」，草稿里的描述清单），登记修订是设计员与执行员的正常步骤，不一定是返工；「修补」13、「修红」8 是返工。记录第 137 行说宽词表的毛病是单字「续」，严格词表又放进了单字「修」。353 是「这份词表数出来的」，不是「返工的派发」——占花费 14.1% 里有多少是修订，没逐条判（推的，至少 11 个描述不是返工）。

### K4-3　盲判一致率 37/42、33/42：复得出；「另多 3 次」的读法有争

记录：「主因一致 37 次；可避免与否（Y 或 P 对 N）一致 33 次」（`records/2026-10-22-子agent结果落地审计.md:93`），没写怎么比的。`tools/blind_agreement.py` 读两份报告的逐次表（rework.md 的「归类」「本可避免」两列，blind-rework.md 的「主因」「可避免」两列），主因按类名归一、去掉首派，复出「续派 42 次；主因一致 37；可避免（Y 或 P 对 N）一致 33」（`research/prompts/landing-audit-r1-opus-model/out/k4-blind.txt:2`）。可避免不一致的 9 次里 rework 判 N、盲判判 P 的 7 次（E171 #7 #29 #30 #31 #33 #34 #35），与记录同；另两次是 E171 #11（P 对「拿不准：N 或 P」）与 E188 #8（N 对 Y）。

- K4 的 6/8：rework.md 点 K4 的 8 行（「| E171#3(P) #5 #8(P) #17(P) #18(P) #25」（`research/prompts/subagent-landing-audit-2026-10-22/rework.md:125`）在两条链里是 E171 #3 #5 #8 #17 #18 #25、E188 #4 #5），盲判甲类 6 行都在里面，E171 #17 #18 盲判归给主 agent 那一步（丁类）。对得上。
- K3 的「另多 3 次」：盲判丙类是「E171 #13、#14、#15、#16；E188 #6、#7、#8」（`research/prompts/subagent-landing-audit-2026-10-22/blind-rework.md:101`）。E188 #6 #7 rework.md 归的是另一条缺口：「| K10 登记判据当场报了问题仍先跑整格 | 2 | 3.65 | E188#6 #7 |」（`research/prompts/subagent-landing-audit-2026-10-22/rework.md:132`），E188 #8 rework.md 判计划内、N。所以三次里两次是「同一行换了一条缺口」，一次是可避免性判法不同，不是盲判独立找出 rework 漏掉的三次。记录拿它当 K3 的加分：「盲判独立复现 4/4，另多 3 次」（`records/2026-10-22-子agent结果落地审计.md:101`）。K3 判「真」靠 4/4 已够，这一条只改「另多 3 次」的读法，不翻判定。推翻条件：K10 在主 agent 那里本来就算 K3 的一部分。

### K4-4　延后项 8/54、回扫 `crates/` 行 16/129：算术对，复不出

- 算术：「无去向 8/54 = 14.8%」（`research/prompts/subagent-landing-audit-2026-10-22/handoff-gaps.md:59`），表里 A 层 7 + B 层 1；6/54 = 11.1%；加权 12.0%（A 层 646 × 7/50 ≈ 90.4，B 层 632 × 1/15 ≈ 42.1，分母 646 × 40/50 + 632 × 14/15 ≈ 1106.7，比是 11.97%）；回扫表 102 + 15 + 8 + 4 = 129，原样 4 + 8 + 0 + 4 = 16，16/129 = 12.4%。都对。
- 复不出：两组数的输入都在 `~/.local/share/singlefs/work/subagent-landing-audit-2026-10-22/` 下（例 `research/prompts/subagent-landing-audit-2026-10-22/handoff-gaps.md:110` 点名的 `inputs/all-agents-classified.json`），这个目录今天不在；抽样「命中太多（1350 处），按层固定种子抽」（`research/prompts/subagent-landing-audit-2026-10-22/handoff-gaps.md:32`）只写了种子，没写抽之前命中怎么排序，样本表与 `tools/q1_hits.py` 都随目录没了。能复的只有命中数：`tools/deferral_hits.py` 照第一节那张词表数（按长的先匹配），今天工作区 A 层 661、B 层 650，HEAD 那一版 A 层 612、B 层 575（`out/k4-deferral.txt` 第 3、7 行与第 2、6 行的合计），报告写的是 A 646、B 632（`research/prompts/subagent-landing-audit-2026-10-22/handoff-gaps.md:24`）；逐词看，下轮开 186、下一轮 135、暂不 10、另开 21、下次 17、继续留 25、另起 51、留作 21 与报告逐个相同，差的只在延后、留到、留给、之后再与下一批——当天记录一直在长，数落在 HEAD 与此刻之间，取法对得上。
- 判：两个比例「算术对、复不出」，复核只能逐条核报告里列出的 8 条无去向与 16 行原样（我没做）。推翻条件：那个目录在别处留了副本（`git log` 里找不到，它不进仓）。

### K4-5　顺带核的几个数

- 定义与共用约束「对暂存区 20 行加、12 行减」：`git diff --numstat` 七份合计 20 / 12，对上。
- 新格「14 行，10 行复跑相符，4 行在滞后表」：另写一段 python 不经门禁、照登记表读单元表源码与 `replay.sh` 复跑表，14 行里 10 行驱动行在且产物名相符、4 行 none（草稿里跑的）。对上。
- 自证读数：runner-dispatch-guard「✓ 自检通过（查了 437 种）」、write-guard「查了 228 种情形」、agent-cost「查了 12 项」与记录相同；heavy-test-guard 今天「查了 1395 种，其中该拒 264 种」，记录写 1386（`records/2026-10-22-子agent结果落地审计.md:135`）。开工快照里 heavy-test-guard 与 lib_heavy_tests 的 diff 混着别的会话加的 `replay.sh` 按名字判那几格（快照 diff 第 598 至 602 行等），差的 9 种应当是它们（推的，没逐格对）。四份输出在草稿目录，没进模型。

## 没打中的形状

- K2：新格与 multipath-registry 并成一格（对象、输入都不同）；⑰ 与写闸、交回闸重复（写闸不按名字判报告，交回闸只认根）；`stale-candidates.py` 认不变量清单与别的载体表重复（`lib-governance-refs.py` 的 `CARRIER_PATTERNS` 管的是治理文档，不是同一批对象）；`claim-experiment.sh` 跨工作树与别的脚本重复（只有三份两三行的 porcelain 解析，见 K2-8）；整段相同（自己的核对在 387 份对照上两档 0 段）。
- K3：命名自查两条命令被别的钩子拒（64 格全放，对照格被拒）；仍有定义、规则、技能要求 `report.md`（0 处）；写回员「不改 `crates/`」与不变量清单例外（派发表、定义两处、写范围表、写闸样本都一致）；kb-scribe 新加的「找不到的目标」一节与规格检查、`replace-batch.py` 冲突（两个脚本都不限目标在 `.claude/kb/` 下，`scripts/kb/kb-spec-check.py` 里没有路径前缀的判法）；two-logs-guard 拒命名自查（两条命令都没有 `.log` 重定向）；`apply-writer-patch.py` 改认 `result.md` 之后旧习惯交 `report.md` 会静默放过 checker 那一节（不会：补丁动了 `crates/singlefs-checker/src/` 而 `result.md` 不在，报告读成空串，照样判缺那一节、拒）。
- K4：结局账的「交回」口径差（K2-6，今天 0 个）；盲判 37、33、7、6/8 四个数（都复得出）；定义 diff 的 20 / 12；三份自证读数。

取样范围：K2 的整段相同对照 387 份（`.claude/gate.d`、`.claude/hooks`、`.claude/scripts`、`.claude/singlefs-ai-sop/scripts`、`scripts/` 下 git 列出的 `.sh` / `.py`，不含样本）；K3 的钩子探针 4 种类型 × 4 种命令写法 × 4 道钩子；骨架探针只做了 three-way-forward、implementation-writer、clerk 三种类型；结局账在 2287 个子 agent 上全量跑了三次，截止时刻取了三个。

## 这条腿自己的限度

- 整段相同用的是我照 `gate-overlap.py` 写的另一份取法，不是它本身（它的判定那一半被重型测试闸当提交时的检查拒了）；两边的规范化应当相同，没逐行对过它的 `normalized_lines` 与我的。
- 结局账的截止时刻是我拿文件修改时刻推的，记录那次真跑的时刻不知道；「复到个位附近」只说明口径一致。
- 盲判一致率的主因归一（`CAUSES` 那张对照）是我定的，记录没写比法；换一种归一（例如把「承 #n」单列）数会变。
- K3-7 对 test-discipline 的读法（「自己的直觉」含现算）是我的读法，规则作者可能只指「读已有结论」。
- K3-8、K3-9 是字面不对齐，没找到一次真撞上的派发。
- 只在我自己的模型上量过、被攻过零轮的改法：K2-1 到 K2-6 的「并成什么样」、K3-1 到 K3-9 的「改法」都是。下表每一格标「量过」或「推的」：

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| ④⑰ 合成一个报告行解析（K2-4） | 「报告：」写到范围外放行、「报告路径：…/report.md」放行 | 推的（两个洞是量过的，`out/k2-report-labels.txt`；改法没实现） |
| 骨架按类型给报告形态（K3-1） | 三方正推腿骨架过派发闸、过不了交回闸 | 推的（洞是量过的，`out/k3-dispatch.txt`、`out/k3-handback.txt`） |
| 两份定义加「命名检查除外」（K3-3） | 执行员、工具员按定义不跑 | 推的 |
| 命名检查固定在仓根、只看自己的文件（K3-4） | 最小副本上两格判不到 | 推的（判不到是量过的，`out/naming-roots.txt`） |
| 删滞后表、按基准比 none 行（K2-2） | 双份数据、四样同步判法 | 推的 |
| 滞后基准读法抽库（K2-1） | 七份同一段判法 | 推的 |
| 候选扫描读 `.claude/kb-directories`（K2-5） | 下一次载体搬家复发 | 推的 |
| 交回按送达判（K2-6） | 三份口径不同 | 量过「今天差 0 个」（`out/k4-outcome.txt` 第 13 行）；改法没实现 |
| 3a ② 改用岔路单的翻面观测（K3-7） | 设计阶段先算答案 | 推的 |

## 没做什么

- 没跑 `gate-overlap.py` 的判定那一半与新格的真仓单跑（重型测试闸把门禁阶段与 gate-overlap 判定当提交时的检查，攻方腿不跑）；新格的数是我另写的 python 核的。
- 没逐条核 handoff-gaps.md 的 8 条无去向与 16 行原样（输入目录已删，只能逐条翻记录与代码，超出这一轮的挂钟）。
- 没判 K1（归正推腿），没判调度记录里九条缺口真不真；没替主 agent 采纳任何改法。
- 没改仓里被判的文件；写了的只有报告、模型目录 `research/prompts/landing-audit-r1-opus-model/` 与草稿目录。
- 副本上量出的数（结局账、命名四格在副本根上）是在草稿目录的拷贝上量的，不算入库装置上的数。

## 要主 agent 跑的命令

- 要「整段相同」的正式读数：`SINGLEFS_HEAVY_TESTS=commit python3 .claude/singlefs-ai-sop/scripts/gate-overlap.py <仓根>`（判这一次改动），看末行有没有「整段相同」；预期与我的核对一样 0 段。
- 要新格真仓读数：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-experiments.sh 的 replay-comparison-units 格; 范围=1 格; 为什么=核攻方腿另算的 14/10/4' SINGLEFS_HEAVY_TESTS=user-request bash .claude/gate.d/doc-experiments.sh --check replay-comparison-units <仓根>`，看成功行的三个数。

草稿目录 `/home/fy5090/.local/share/singlefs/work/landing-audit-r1-opus/` 里留着：`rerun2/`（第二次复跑的输出；复跑时拷进去的 `agent-cost.py` 与两份 preflight 脚本已删）、几份文本输出（`outcome-*.txt`、`naming-*.txt`、`selftest-*.txt`、`grep-*.txt`、`gate-overlap-list.txt`），留给主 agent 核；没有仓副本与编译目录（两份最小副本跑完已删）。
