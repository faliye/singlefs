# no-task-numbers-governance-r1 攻方腿报告（opus，2026-11-19）

攻击面：判据库 `task-and-session-numbers` 三条判据行的正则与上下文放行、写闸第七道、检出钩子 ⑰、`scripts/kb/lib_notation_check.py`、`scripts/kb/append-record.py` 的接法。被判文件在主工作区现读，五份的 sha256 与开工快照 `research/prompts/no-task-numbers-governance-r1-snapshot/judged-sha256.txt` 逐份相同（判据库 `0267903a…`、写闸 `c71d28e4…`、检出钩子 `02d0c3c8…`、`append-record.py` `fcf58d75…`、`lib_notation_check.py` `23b8137d…`，命令 `sha256sum`，量的）。

本报告里所有号的样例都放在反引号里，否则这份报告本身过不了 ⑰。

## 复跑命令与模型文件

复跑（主 agent 交，在仓根起；冻结副本直接给仓根，`rerun.sh` 先核判据库的 sha256 等于开工快照那一份，不等退 3）：

`task-server/task submit --cwd /home/fy5090/code/singlefs -- bash research/prompts/no-task-numbers-governance-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r1-opus-run /home/fy5090/code/singlefs/research/prompts/no-task-numbers-governance-r1-opus-model`

| 文件 | sha256 | 做什么 |
|---|---|---|
| `tools/probe_cases.py` | `3936d1c2d3fac87f4a9cdd41a7ef05f04ebdcc35261aa9d11ac6fc6c083c818f` | 48 条造的行与 19 条现存真行（跑时按 文件:行号 现取、核片段在）喂判据库 `all_hits`（当成 `records/probe.md` 判），逐条与腿用 `append-record.py --dry-run` 量到的退出码对；不一致退 1 |
| `tools/scan_real_lines.py` | `3b654cfdef34c119ca11ca1b4c6c68dbb42350763f7ade457b8eebbf38935b59` | 扫 `.claude/kb`、`.claude/rules`、`.claude/agents`、`.claude/skills`、`research/prompts`、`records` 下的 `.md`、`.tsv`、`.txt`，按目录与判据行数命中，另按五类启发式数「多半不是号」的命中、各列 8 行样本；禁读清单里的两处与这条腿自己的报告、模型目录不读 |
| `tools/rerun.sh` | `9901d51bc48f87821428d13477885a7ecad9a02074962407dccc600fd8123b30` | 从 `scripts/agents/three-way-rerun-template.sh` 拷、只换了 `rerun_steps`：核库 sha256、起上面两份、把输出写进运行目录 |
| `tools/rerun-from-a-clean-work-directory.sh` | `6c7a80a18e827845d4b027cba462e2a6b3865b6138453cd08cd2bf113bf3b033` | 先删 `-run` 收尾的那一个运行目录，再调 `rerun.sh` |

探针的内存与时间：推的，没量。最大一份被扫文件约 5.1 MB（`find … -printf '%s'` 量的），逐份读、逐份放，峰值推在 200 MB 以内；扫的总量约 79.5 万行、215 MB（`wc` 量的），三条正则逐行，推在一两分钟。

## 各格判定一览

| 格 | 判定 | 主要证据 |
|---|---|---|
| 关 1：漏拒（用户点名四种写法的近似变体） | 成立 | 现存记录里同形、整行放行的 7 种任务号写法、3 种会话短号写法、5 种批号写法，`append-record.py --dry-run` 逐条退 0（量的）；另有判据库没有任何提交时的格，漏过就是终局 |
| 关 2：误拒（kb、规则、判决里的正常行） | 成立 | kb 的 fsync 攒批参数 `seq 批 1`、`攒批 64`，泛指次序 `第一批字节`，步号 `（a6）`、`（a1）`，任务个数 `任务 1180 个`，判决里旧写法 `A/B 读数任务 12400`，逐条退 7（量的）；全仓出现次数由模型给，下面的 grep 近似数是推的 |
| 关 3：放行口子被借来写号 | 成立 | 「A/B 读数：」在行内任何位置都放过整行的任务号；表格行放过任何文件、任何一列的批号；反引号放过三条判据行；Edit 与 heredoc 的新内容以收尾围栏开头时其后整段放过（⑰ 实测写进了仓，见攻击面三） |
| 关 3 之二：拒绝说明的出路照着改能不能过闸 | 能过 | 四个用户例句照出路改成命令、日志行、那一批做了什么，`--dry-run` 退 0；误拒的几类照「放进反引号」改同样退 0（量的）。出路字面里「哪条命令（放进反引号）」「结局行原文」两种照写会把号原样带过闸，见攻击面三末段 |

量法：除标「推的」的，下面每条结论的「退 0 / 退 7」都是这条腿直接跑 `python3 scripts/kb/append-record.py records/2026-11-19-进仓文字不写任务号与会话短号.md <草稿目录>/probes/<编号>.txt --dry-run` 的退出码（退 7 是「文本带…任务号、会话短号、批号」，退 0 是判过放行），原样输出在 `/home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r1-opus/probes/<编号>.out`。`append-record.py` 与写闸、⑰、`replace-once.py` 读同一份库、同样逐段调 `all_hits`：`.claude/hooks/write-guard.sh:613`：「found = [(where(number), name, matched) for text, where in places for number, _, name, matched in library.all_hits(form_key, text, relative)]」，`scripts/kb/lib_notation_check.py:97`：「hits = library.all_hits(form_key, new_text, relative)」。四个用户例句做阳性对照（C1–C4），都退 7、点名的命中与原例一致。

## 攻击面一：漏拒

判据行原文：`.claude/gate.d/lib-forbidden-notations.py:207`：「TASK_NUMBER = (rf'(?:(?<![{ASCII_WORD}])(?:任务\s*(?:号\s*[:：]?\s*)?|[Tt]ask\s+#?)\d{{4,}}{TASK_NUMBER_AFTER}'」，`.claude/gate.d/lib-forbidden-notations.py:206`：「TASK_NUMBER_AFTER = rf'(?![{ASCII_WORD}\-/:]|\.\d)'」，`.claude/gate.d/lib-forbidden-notations.py:208`：「rf'|(?<![相不共如协合])同\s*\d{{4,}}{TASK_NUMBER_AFTER}(?=\s*[、，,]\s*\d{{4,}}))'」，`.claude/gate.d/lib-forbidden-notations.py:210`：「SHORT_CODE_IN_PARENTHESES = r'[（(]\s*[0-9a-f]{2}\s*[)）]'」，`.claude/gate.d/lib-forbidden-notations.py:217`：「BATCH_NUMBER_MEASURE_WORDS = '个条份项次件只台组块页字道格趟片轮天周位行种处遍%'」，`.claude/gate.d/lib-forbidden-notations.py:220`：「rf'|(?<![#{ASCII_WORD}])第\s*[\d{CHINESE_NUMERALS}]+\s*批(?!\s*[{HEAVENLY_STEMS}])')」。

### 任务号

| 形 | 现存同形（整行喂库，量的） | 造的变体（量的） | 为什么放过 |
|---|---|---|---|
| `任务（N–N、N）` | 编号 a：`records/2026-11-01-判定库服务化.md:473` 整行退 0；同形另见 `records/2026-11-02-一百七十六节欠账.md:283`、`records/2026-11-10-全量提交门禁收尾调度.md:130`（grep 计 19 处，推的） | V02 退 0 | 「任务」与数字之间只许空白或「号：」，全角括号不认 |
| `任务：N` | 编号 b：`records/2026-11-12-subagent账本清欠.md:63` 整行退 0（「件 6 交的任务：」后跟四个号） | V01 退 0 | 冒号只许跟在「号」后面 |
| `任务 N/N` | 编号 c：`records/2026-11-02-文档代码逐项审核.md:549`；编号 d：`records/2026-11-16-三方豁免取消与存量补轮.md:595`，整行都退 0 | V04 退 0 | 数字后面接 `/` 被 `TASK_NUMBER_AFTER` 排除（本意是放过路径） |
| `任务 N-N`、`任务 N:` | 没在 `.md` 里找到现存同形 | V05、V06 退 0 | 同上，`-`、`:` 被排除（本意是放过日期与行号） |
| `同 N/N`、`同 N` 单个、`同 N 与 N` | 编号 e：`records/2026-11-01-判定库服务化.md:327`；编号 f：`records/2026-11-11-欠账表按代码重梳调度.md:88`，整行都退 0；`同 N` 单个在 `records/*.md` 里 grep 计 30 处（推的） | V07、V08、V09、V10 退 0 | 「同」那一支要两个以上、只认「、，,」连着 |
| `任务 #N` | 编号 g：`records/2026-10-31-task-server巡检.md:13`（表格行；任务号不放过表格行）整行退 0 | V03 退 0 | `#` 只在英文 `task` 后面认 |
| 英文复数与连写 | `.claude/kb/experiments/158-four-forks-in-root-selection-and-repair.en.md:116` 有 `Four tasks (` 后跟号的写法（同一行另有被拒的号，这一行整行不放） | V22 `tasks (…)`、V23 `task#N`、V24 `task=N` 退 0 | `[Tt]ask\s+#?` 要空白，复数的 s 与 `=` 都断开 |

与用户例句的关系：用户原例在 `records/2026-11-18-等GPU拆到锁外与前台等锁让路.md:80`，同一行里另有一串裸号（`clippy 21002`、`构建 21003`、`缩域单跑 21005、21006、21009、21011 退 0` 这些），工具员已写明管不到（`.claude/gate.d/lib-forbidden-notations.py:77`：「管不到的：没带「任务」「task」「同」的裸任务号」），这里不再算新打中，只记：用户点名的那一行里，被闸认得的是三处，认不得的裸号是十来处。

### 会话短号

| 形 | 现存同形（量的） | 造的变体（量的） | 为什么放过 |
|---|---|---|---|
| 括号里短号后面跟话 `（c4 在改）`、`（b6 推的）`、`（5a 接了）` | 编号 j：`records/2026-10-18-里程碑三收尾与近期欠账.md:166` 整行退 0；同形 grep 计 24 处（推的），例 `records/2026-11-01-判定库服务化.md:322`、`records/2026-10-31-task-server巡检.md:397`、`records/2026-11-05-删轻型并入重型.md:770` | V11 `（5b 在修）`、V12 `（5b、10 在修）` 退 0 | 括号里只许两位十六进制加空白，后面紧跟括号 |
| 四位十六进制短号 `（e152）` | 记录里有 `（e198）` 一处，判不出是会话还是实验号，不算 | V13 退 0 | 只认两位；会话名本身是 2 到 6 位 |
| 大写 `（5B）` | 没找 | V14 退 0 | 只认小写 |

这几种都在 `.claude/agent-common.md:41` 那条「会话全名与两位短号都不写」的射程里。

### 批号

| 形 | 现存同形（量的） | 造的变体（量的） | 为什么放过 |
|---|---|---|---|
| `第 N–M 批`（区间） | 编号 m：`records/2026-10-13-Verus证明审计.md:26` 整行退 0（同一行另有 `b16–b37 各批`）；同形 grep 计 49 处（推的） | V16 退 0 | 「第」后面只许数字串，碰到 `–` 断开，后一个数前面没有「第」 |
| `第 N、M 批`（并列） | 同形 grep 计 74 处（推的），例 `records/2026-10-26-checker欠账C896-C897-C914.md:406`（那一行另有 `第 10 批` 被拒） | V17 退 0 | 同上，`、` 断开 |
| `批 #N` | 编号 k：`records/2026-10-19-里程碑四GPU垃圾回收开工.md:22`；编号 l：`records/2026-11-10-全量提交门禁收尾调度.md:117`，整行都退 0 | V15 退 0 | `批` 与数字之间只许空白与冒号 |
| `批 N 只…`（副词「只」） | `records/2026-11-06-门禁判定全部入库.md:409` 有 `批 6 只在副本里改`、`records/2026-10-23-doc-text两格红行清理.md:143` 有 `批 11 只改日语`（两行另有别的批号被拒） | V19、V20 退 0 | 量词表里有「只」「处」「行」「位」「道」这些也常作副词、名词的字 |
| `批次号 N` | 没找到现存同形 | V18 退 0 | `批[次号]?` 只吃一个字 |
| `批次：<记录>#第 N 批` | 仓里文本零处（grep 量的） | V21 退 0 | `#第` 一律当锚点放过；`.claude/agent-common.md:43` 写着这一行不进仓 |

### 打中之后的四句（关 1）

- 分不分辨臂：这一题没有臂，比的是「库现在的判法」与「规则字面」；每一格都只打这份库，与别的改法无关。
- 被判的系统当时看不看得到判别它的东西：看得到。号就在交进来的那一段文字里，库只是没认这些形。
- 满足的是判据字面的哪一句：派发提示关 1「造出一条放行的、在现存记录里出现过同形的写法」。编号 a、b、c、d、e、f、g、j、k、l、m 是现存记录里整行原样放行（不是造的），满足字面；V 系列是同形的造句。
- 改法在打中的格上还中不中：见末尾「改法与修哪一格」。

## 攻击面二：误拒

| 类 | 现存真行（整行喂库，量的） | 命中 | 为什么拒 | 出现次数（推的，grep 近似；准数待主 agent 跑模型） |
|---|---|---|---|---|
| fsync 攒批的大小（参数，不是批号） | 编号 r：`.claude/kb/decisions/16-发布语义.md:177` 退 7；编号 s：`.claude/kb/experiments/08-大文件与小文件是否该走不同写路径.md:43` 退 7 | `批 1`、`批 10`、`批 64` | 批号那一条只放过前面是「每」「分」与后面跟量词的；`seq 批 1 是`、`攒批 64 /` 都不在内 | 非表格行：kb 34、`research/prompts` 49、`records` 2（只数前面紧挨 seq / 追加 / 覆写 / 攒 / 写入 / 流数 N 的） |
| 泛指次序「第一批 <名词>」 | 编号 t：`.claude/kb/experiments/100-系统配置的三段几何.md:6` 退 7（`mkfs 写的第一批字节`） | `第一批` | 「第 <中文数字> 批」一律算批号 | 非表格行「第一 / 二 / 两 / 三批」：kb 73、`research/prompts` 321、`records` 214，里面混着调度批号（如 kb 引记录小节名），要看样本分 |
| 括号里的步号与标签 | 编号 u：`.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:416`（`（a6）`）退 7；编号 v：`.claude/kb/layout/01-first-txn.md:441`（表格行，`（a1）`）退 7 | `（a6）`、`（a1）` | 数字字母都有的两位括号一律判，表格行不放过这一条 | kb 22、`research/prompts` 54（逐条看过，`research/prompts` 那 54 处里两处是真短号：`research/prompts/gpu-tests-placement-design-r2-sonnet-output.md` 一处会话列举、`research/prompts/m4-close-c1040-c1041-code-r1-main-verification.md` 一处「以 `(5b)` 借卡那一批」；其余是 `（1a）`、`（2c）`、`（f3）`、`（e1）` 这类步号；三方腿报告给攻击项编号就用这种写法） |
| 任务的个数 | 编号 x4：`records/2026-11-10-全量提交门禁收尾调度.md:276` 退 7（`任务 1180 个`） | `任务 1180` | 任务号那一条没有量词放过 | 全仓 1 处 |
| 判决里零轮改法的旧写法 | 编号 w：`research/prompts/device-library-write-many-code-r3-main-verification.md:42` 退 7（`A/B 读数任务 12400`，没有冒号） | `任务 12400` | 放行要「A/B 读数」后面跟冒号 | 2 行（同一份判决第 42、43 行） |
| 缩进四格的代码块 | 造的 F12 退 7 | `任务 21036` | 只认围栏，不认缩进代码块 | 没数 |

关于工具员已写明的「括号里像 `(a1)`、`(2c)` 的步号会被误判」：这一类这里给了次数与真行，不算新形状；新的是前四类里的 fsync 攒批参数、泛指次序与任务个数，与「判决旧写法」那一类。

两条与正推腿分工交界的，只记不展开：派发闸认的「A/B 读数」一行是 `.claude/hooks/runner-dispatch-guard.sh:2566`：「ZERO_ROUND_READING_LINE = re.compile(LINE_START + r"A/B[ \t]*读数[* \t]*[：:][* \t]*(?P<value>.*?)[ \t]*$", re.M)」，认 `**A/B 读数**：` 与 `A/B读数：`；判据库只认 `A/B 读数` 中间恰好一个空格、后面紧跟冒号，F10、F11 两种写法退 7（量的）。

### 打中之后的四句（关 2）

- 分不分辨臂：没有臂；每一格只打判据库的一条判据行。
- 系统看不看得到判别它的东西：fsync 攒批那一类看得到（前面是 `seq`、`攒`、`追加` 这些词），泛指次序看得到一半（后面跟的是名词还是「交回」「合入」要读语义），步号与真短号在字面上分不开（同一个 `(5b)` 可以是步号也可以是会话），任务个数看得到（后面跟「个」）。分不开的那一类改判据也救不了，只能靠写的人放反引号。
- 满足的是判据字面的哪一句：关 2「拿真行喂判据库……拒了就打中，报出现次数」。编号 r、s、t、u、v、w、x4 是真行原样被拒，满足字面；次数这里是 grep 近似（推的），准数是模型的 `name=scan_class` 行。
- 改法在打中的格上还中不中：见末尾「改法与修哪一格」。

## 攻击面三：放行口子被借来写号，与出路照着改

放行原文：`.claude/gate.d/lib-forbidden-notations.py:221`：「RESERVED_TASK_NUMBER_LINE = re.compile(r'A/B 读数\s*[:：]|^\s*(?:[-*+]\s+|\d+[.)、]\s*)?长跑拆分\s*[:：]')」，`.claude/gate.d/lib-forbidden-notations.py:329`：「return inside_code_span(line, match.start()) or RESERVED_TASK_NUMBER_LINE.search(line) is not None」，`.claude/gate.d/lib-forbidden-notations.py:334`：「return inside_code_span(line, match.start()) or line.lstrip().startswith('|')」，`.claude/gate.d/lib-forbidden-notations.py:619`：「'skips_fenced_blocks': True,」。规则那一边：`.claude/agent-common.md:42`：「照写任务号的只有两处：判决里零轮改法的「A/B 读数：」那一行」。

| 口子 | 借法（量的） | 借来的范围 |
|---|---|---|
| 「A/B 读数：」 | B1：`- 九项崩溃点重放连撞三次上限，A/B 读数：见上；同 20461、20814，任务 21036 已认领` 退 0 | `search` 不锚行首：一行里任何位置出现「A/B 读数：」，这一行所有任务号都放过；也不看文件，记录、kb 里写都放（规则说的是判决里的零轮改法） |
| 「长跑拆分：」起头 | B2：`- 长跑拆分：无；任务 21036 同 20461、20814 在跑` 退 0 | 锚了行首，但整行放过，不管后面写的是不是长跑拆分 |
| 以 `|` 起头的表格行 | B3：`| 备注 | 它的批 105 在修，第 6 批交回 |` 退 0；现存真行编号 o：`records/2026-10-23-lkmm绑定复用与依赖脚本修正.md:18` 第三列 `批 3 交回报告「没做的」` 整行退 0 | 任何文件、任何表、任何一列都放（规则只留调度记录依赖表的批次那一列）；kb 的表格行里批号命中 grep 计 279 行（推的） |
| 反引号 | B4：`- 对照（`+反引号包住的任务号+`）同样退 78`，三条判据行的号都包进反引号，退 0 | 三条判据行都放过反引号里的；出路第三条明写「放进反引号」 |
| 围栏，新内容以收尾围栏开头 | B6 在库上退 0；**⑰ 实测写进了仓**：先排他新建一份以开着的围栏收尾的 `.md`（模型目录下，已删），再 `cat >> 那一份 <<'EOF'`，heredoc 第一行是收尾的三个反引号、第三行是 `摘录之后的正文：` 加用户三种写法，⑰ 放行、退 0；同一句不带围栏的 heredoc 被 ⑰ 拒（拒绝原文点名 `任务 21036`、`(5b)`、`批 105` 三处）。写进去之后的文件存在 `/home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r1-opus/probes/heredoc-half-fence.after.md`：那一行在文件里是围栏外的正文 | 写闸逐个 new_string、⑰ 逐个字面、`replace-once.py` 逐个新串认围栏；Edit 把「代码块收尾 + 后面的正文」当一个 new_string 写，是改记录时常见的一刀。工具员写明的是「半截围栏认不出」的误拒那一面，没写这一面会漏拒 |
| 围栏开头行带信息串 | B7：`~~~` 后面直接跟用户三种写法，单独一行，退 0 | 开围栏那一行自己算在围栏里，且没收的护到末尾 |
| `#第 N 批` 锚点 | V21 退 0 | 见攻击面一 |

漏过就是终局：这一形态没有提交时的格（`.claude/gate.d/lib-forbidden-notations.py:126`：「任务号、会话短号与批号一形态（没有 doc-text.sh 的格」；`grep -c task-and-session-numbers .claude/gate.d/doc-text.sh` 输出 `0`，量的）。写闸、⑰ 与 `scripts/kb/` 几份写入工具是唯一的几道；⑰ 自己写明不判 `cp`、`mv` 与别的命令的输出（`.claude/hooks/bash-command-detector.sh:357`：「内容不是字面的（cat 文件、cp、mv、别的命令的输出）」），草稿里写好再 `cp` 进 `research/prompts/` 的不过任何一道。`records/` 下的 `.json`（台账一类）不在 `.md`、`.tsv`、`.txt` 里，grep 计 145 行带「任务 N」（推的）。

### 出路照着改

| 拒的写法 | 照出路改成 | 结果（量的） |
|---|---|---|
| 用户例句 C1 那一句 | R3：写成那条命令（反引号里的 `bash scripts/crash/crash-amplification.sh --item x`）、日志里那一行、「改 GPU 服务交回路径那一路的会话在修」 | 退 0 |
| C4 那一句 | R4：`改算键库那一批量读数：退 0` | 退 0 |
| 误拒的 `seq 批 1` | R5：放进反引号 | 退 0 |
| 误拒的 `（a6）` | R6：括号里的放进反引号 | 退 0 |
| 判决旧写法 | R7：`A/B 读数：` 加任务号 | 退 0 |
| 误拒的任务个数 | R8：改语序成「1180 个任务」 | 退 0 |
| 误拒的泛指次序 | R9：`头一批字节` | 退 0 |

出路字面能把号原样带过闸的两种：`.claude/gate.d/lib-forbidden-notations.py:626`：「任务号换成哪条命令（放进反引号）、结局行原文（TASK_SERVER_RESULT 那一行）或日志里的那一行」。R1：反引号里的 `task-server/task log` 加号，退 0；R2：结局行原文，退 0，结局行本身带号（`task-server/task_server_lib.py:4615`：「line = (f"{RESULT_MARK} task={task.get('id')} type={task.get('outcome')} exit={task.get('exit_code')} attempts={len(pairs)} "」）。现存记录里这两种都有：反引号里的 `task-server/task log`、`status`、`clean` 加号 grep 计 17 处（推的，例 `records/2026-11-01-判定库服务化.md:75`）；`TASK_SERVER_RESULT task=` 加号的结局行例 `records/2026-10-18-里程碑三收尾与近期欠账.md:2815`。用户原话准写「结局行原文」，所以 R2 算不算违规推不出，交主 agent 判；R1 是借反引号写号，算关 3 打中。

⑰ 的拒绝说明只取 remedy 第一条（`.claude/hooks/bash-command-detector.sh:5627` 起的 `literal_notation_remedies`），误拒时该看的「放进反引号」那一条 ⑰ 不给；拼接处多出一个「；。」（上面 ⑰ 拒绝原文里可见）。

### 打中之后的四句（关 3）

- 分不分辨臂：没有臂；每个口子单独一格。
- 系统看不看得到判别它的东西：「A/B 读数：」行内位置、表格是不是调度记录依赖表、文件是不是判决，库都看得到（行文、相对路径都交进来了），只是没判；围栏那一格，写闸与 ⑰ 只拿到新内容、看不到文件里原来开着的围栏，单看 new_string 判不出（判别子观测不到），要改得读目标文件。
- 满足的是判据字面的哪一句：关 3「能不能被正文借来写任务号」——B1、B2、B4、B6（⑰ 实写）、R1 是借来写任务号；B3 与编号 o 是借表格行写批号，同一条的批号那一面。
- 改法在打中的格上还中不中：见下一节。

## 改法与修哪一格

全是这条腿提的，没实现、没跑，每一格都是「推的，待主 agent 跑」；只在探针的样例上推过、被攻过零轮。

| 改法 | 修哪几格 | 不修、可能新误拒的 |
|---|---|---|
| 任务号那一条：「任务」与数字之间另认全角括号、冒号、`#`；数字后面接 `/` 或 `-` 再接四位以上数字的照判（日期 `-\d\d-`、行号 `:\d` 仍放） | 编号 a、b、c、d、g，V01–V05（推的） | V06（`任务 N:` 加 ASCII 冒号与行号分不开，推的） |
| 「同」那一支另认「与」「和」「/」连着的号；单个 `同 N` 要不要判交主 agent 定 | 编号 e，V07、V08、V10（推的） | 单个的编号 f、V09；判单个会撞「同 4096 字节」这类（推的，没数过） |
| 任务号那一条加量词放过（后面跟「个」「条」） | 编号 x4（推的） | 无 |
| 会话短号：括号里两位短号后面跟空白与「在修 / 在改 / 接了 / 已接 / 推的 / 量的 / 转述 / 范围 / 判过」这类词也判 | 编号 j，V11、V12（推的） | 步号后面跟说明的（`（1b） 两种取法` 这类）会新误拒，次数没数 |
| 批号：`第 N[–、，]M 批`、`批 #N`、`批次号 N` 另认；量词表去掉「只」「处」「行」「位」「道」这几个常作副词与名词的字 | 编号 k、l、m，V15–V20（推的） | 去掉量词之后「批 3 处」这类真计数会新误拒，次数没数 |
| 「A/B 读数：」改成锚行首（同派发闸 `LINE_START` 那一认法），并只在 `research/prompts/*-main-verification.md` 里放 | B1（推的） | 现存记录里没有行内写「A/B 读数：」的，不新增误拒（推的） |
| 表格行放过批号只在 `records/` 下 | kb 与判决里借表格行写批号的（推的） | 编号 o 在 `records/` 下，不修 |
| 写闸、⑰、定点写入工具认围栏时先读目标文件，算出插入点前围栏开没开着，再判新内容 | B6（⑰ 实写那一格）、工具员写明的「半截围栏」误拒（推的） | B5、B7（整段新文字自己就是开着的围栏，读文件也一样放，推的） |
| 批号那一条放过前面紧挨 `seq`、`攒`、`追加`、`覆写`、`写入` 的 | 编号 r、s（推的） | 泛指次序编号 t 不修；它与调度批号字面同形，改判据救不了，只能靠写的人换「头一批」或放反引号 |

出路的字面：把「任务号换成哪条命令（放进反引号）」改成写交给服务的那条命令（不带号），并写明 `task-server/task log`、`status` 加号的不算，修 R1（推的）；R2 留给主 agent 对用户原话判。

## 要主 agent 跑的命令

1. `task-server/task submit --cwd /home/fy5090/code/singlefs -- bash research/prompts/no-task-numbers-governance-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r1-opus-run /home/fy5090/code/singlefs/research/prompts/no-task-numbers-governance-r1-opus-model`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`。
   - 看哪几行：末行 `TASK_SERVER_RESULT` 的 `type=succeeded exit=0` 算过（退 1 是探针里有一条库判定与腿量到的不一致，退 3 是判据库不是开工快照那一份）；`task-server/task log <号> --all` 里 `name=rerun_probe_cases exit=0` 与它下一行 `name=probe_summary cases=67 disagreements=0 rule_vs_library_differs=…`；`name=scan_class directory=… class=… hits=…` 各行是关 2 的准数，样本在运行目录的 `scan-real-lines.out` 的 `name=scan_sample` 行。
   - 被范围声明闸拒的话：这一条是模型复跑、不是门禁与崩溃入口，推它不要声明；拒了照拒绝说明补 `SINGLEFS_RUN_SCOPE`。

## 没打中的形状

- 任务号：`任务21036`（不带空格）、`子任务 N`、`任务号：N`、`任务 N、N`、`任务 N–N`（en dash）、`连同 N、N`、`（任务 N）` 都被拒（前三种推的，读正则；后几种 grep 现存行与 C1、C3 量的）；「相同 / 不同 / 共同 / 如同 / 协同 / 合同」后面跟号的放过是设计如此，现存行里没找到被它放过的真任务号（grep `(?<![相不共如协合])同` 那一支全仓只 3 处命中，都是真号且都被拒，推的）。
- 会话短号：`给 (10)`、`(10) 说` 这类挨会话词的纯数字短号，在 kb、规则、`research/prompts` 里 grep 零处（推的），没有误拒；`Some(22)` 这类挨 ASCII 字母的放过是对的。
- 批号：`每批 64`、`分批`、`一批 10 个`、`批 0.62 批/行`（`.\d` 放过）没有误拒；`#第 N 批` 在仓里文本零处，没有现存误拒；`批量 N` 不命中。
- 接法：写闸对主 agent 与子 agent 一样判（`decide` 里这一判不看 agent 类型，读的是 `.claude/hooks/write-guard.sh` 第 969 行那一项）；`lib_notation_check.py` 的会话号、批次代号、任务号三判按次序报第一样，没找到漏判的次序；`append-record.py` 只判追加的文本，记录里已有的旧号不挡（工具员自证覆盖，这里没另造）。
- 取样范围：现存行只在 `records/*.md`、`.claude/kb`、`.claude/rules`、`.claude/agents`、`.claude/skills`、`research/prompts` 里 grep；整行喂库量的共 19 行真行、48 行造的。

## 这条腿自己的限度

- 量的只有 `append-record.py --dry-run` 的退出码与一次 ⑰ 实写；写闸（Write / Edit）这条腿手里没有那两个工具，没能实测，写闸与 ⑰、`append-record.py` 同调 `all_hits` 这一点是读代码（推的，引文见开头）。
- 全仓次数都是 `grep -P` 照库里正则改写的近似：不认围栏、不认反引号、不认排除前缀，`\d` 只认 ASCII 数字；准数要等模型复跑。
- 启发式分类（fsync 攒批、泛指次序、步号）是这条腿自己的分法，样本要人看；泛指次序那一类里混着调度批号，没逐条分。
- 「现存同形」只证明这种写法有人写过、写闸放过；那几行是旧行，规则不回改，打中的是「以后照这种写法写会放过」。
- 改法全是推的、被攻过零轮。
- 一次 grep 扫 `research/prompts` 时，命中行里带出了禁读清单里 `research/prompts/no-task-numbers-governance-r1-sonnet-output.md` 的几行（找「任务 <号>」占位那一次）；之后的 grep 都加了 `--exclude`，那几行的内容没用进这份报告。
- 这条腿没有派发提示要的 `crates/` 崩溃枚举原型（攻击面全是文本判据），第 3b 步的崩溃状态取样不适用。

## 没做什么

- 不判正推那几格（规则文字对用户原话、全仓还要求写号的句子、保留两处的固定写法与库认法是不是同一个形态；后者只在攻击面二末段记了一句交界）。
- 没往统一任务服务交任务；模型复跑命令在「要主 agent 跑的命令」，判决要引的数等主 agent 复跑。
- 写闸没实测（没有 Write / Edit 工具）。
- 没改被判文件，没建仓副本、没编译。

## 清理

- 没建仓副本、编译目录与库拷贝。
- 模型目录里试 ⑰ 用的 `probes/heredoc-half-fence.md` 写进过一行带号的正文，拷进草稿目录留证之后已删（`rm`，退 0），空的 `probes/` 目录已删；`py_compile` 在模型目录 `tools/` 下生成的 `__pycache__` 已删。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r1-opus`（628K）里只有探针的文本输入、`append-record.py` 的原样输出、⑰ 实写后的文件副本与 `rerun.sh` 的草稿，是这份报告引的证据，留着。
