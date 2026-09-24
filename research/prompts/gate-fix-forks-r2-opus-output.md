# gate-fix-forks-r2 云端攻方（Opus）：T1、T5、T6、T11

写于 2026-09-24 01:10 JST（2026-09-23 16:10 UTC）。只攻正文第四节分给这条腿的四格；T3、T4、T7、T8 与辩方的复核没碰。攻的是工作区里写好的代码与样本，不是正文的转述。

开工快照：`sha256sum -c research/prompts/gate-fix-forks-r2-snapshot/sha256sums.txt` 78 份全部 OK（`grep -v ': OK$'` 零行）；收工前再核一次，结果写在末尾「这条腿自己的限度」。

## 复跑

模型全在 `research/prompts/gate-fix-forks-r2-opus-model/`，只读真仓，现场建在第二个参数给的目录下的临时 git 仓里（`GIT_CONFIG_GLOBAL=/dev/null`、`GIT_CONFIG_NOSYSTEM=1`，与本机 git 配置隔开）：

```
M=research/prompts/gate-fix-forks-r2-opus-model; D=/tmp/claude-1000/gate-fix-forks-r2-opus/run
nice -n 19 bash    $M/t1-scenes.sh      . $D/t1 $M   # T1：戊（75 号 ⑤）与丙b* 在同一批改动上；10 号剩下那段与 75 号 ⑨
nice -n 19 python3 $M/t1-scan.py        .            # T1：真仓里 75 号 ①③ 都看不见的「决策提到实验、实验表里没这条决策」对子
nice -n 19 bash    $M/t5-added-lines.sh . $D/t5      # T5：gate_added_lines 的输入形状；40 / 61 / 88 新版与 HEAD 版同一现场；调用方吞退出码
nice -n 19 bash    $M/t5-fix.sh         . $D $M      # T5：本腿提的 gate_added_lines 补丁打在副本上，同一批现场重跑
nice -n 19 bash    $M/t5-64.sh          . $D/t5-64   # T5：64 号两条判据的漏判与误判写法
nice -n 19 bash    $M/t6-mutants.sh     . $D/t6      # T6：仓里 check-segment-registry.py 的九个变异 × 三道检查；本腿提的红样本
nice -n 19 bash    $M/t11-bases.sh      . $D/t11     # T11：八种仓状态下两种取法各取到哪个提交；两个候选各让哪道阶段判错
nice -n 19 bash    $M/t11-leak.sh       . $D/t11leak # T11：认 GATE_DIFF_BASE 时真仓的基准漏进样本自检
```

各 `.out` 是在仓根上照上面命令跑的原样输出（提交号每次跑不同，判定不变）。sha256：

```
bde51fa11f36998c8330852acfaed6e71ff37a8704a1b844d36e0ae0197ab96e  t11-bases.out
a553eb37ca085f37f2aa484904f7ed645970f40f621ef589f6b05be7241e4b14  t11-bases.sh
830f9b8bc31f2a3187608bc6dde2e4c381294c70a9eb38748abc971054279614  t11-leak.out
facb26d641b8a76766de6eec07fd1d60e6c4600a1250b375f1cd9484678edaa9  t11-leak.sh
4c2bf924fcb287fa957df493b21f9ca8dec51c675d016d46544e33dd89767552  t1-judge.py
b16177911d5cc1a1ae5e507eab679fb5ad4ca500ce50caa489801c0ad8cf7397  t1-scan.out
d64d39deb6ca6db70c145c6f1e8e9ce04c95b96c86ade30aa233d1e69023a77d  t1-scan.py
2bdeff47c5e686a0b4bf0028df988c87201a8ca276cc831a13d5ae7be53a7e83  t1-scenes.out
3aa28d3f7bdc332c9047122691cbcf6d78315020d7e59261253cf6b0b38a040d  t1-scenes.sh
89e21fea24d7a28b05ddfab181ab25668b08f8d1e9dda2acf46c690438f59f6f  t5-64.out
46b520d78dbf88b0cbf8fe7b8a57d5bd1c0d9b3ea9962026ec0febde5d08fff4  t5-64.sh
8e10144ad829825054fe6f4542301d3b217b507b817d89323998f7facd7146ae  t5-added-lines.out
6f978cea1d58839963708bdd8a4e9b5511bfd4583f643e55accfa594032c8295  t5-added-lines.sh
733b994fcf04ec121946a73752236424e3110caf26d745d77adcba7f2259165b  t5-fix.out
f5540d1be7de08488e9706cd316851f18c8a5e32d6ffaa9c6dd0ccd7ae8b5065  t5-fix.sh
911215dcfad33aa38fe33eaa0742f224230c03793923327cca493af6a6d1961f  t6-mutants.out
3c0598e3151a67bddebdbc8a6dfbcd804f467a43ac0bd1a3b2e1717dd0e5b8b0  t6-mutants.sh
```

## 各格判定一览

「打中」按正文第五节：具体可复现的失效（现跑确实如此），或逐字抄得出、与形态直接冲突的条款原文。

| 格 | 被判的东西 | 判定 | 打中的东西（一句） | 分辨臂 |
|---|---|---|---|---|
| T1 | 戊：撤 10 号 §3，射程交 75 号 ⑤ | **打中但改法在手** | 决策正文在「**依据**」段之外（例「**欠**」块）提到一个实验、而实验的影响表里没有这条决策时，75 号 ①③ 都看不见这一对；这一批把实验改成已跑、只回看表里别的行，75 号绿、丙b* 红（t1-scenes B）。真仓今天有 25 对这样的对子，其中 D26 已定项里「等 E10（老化：结构抗老化 vs 事后整理）」那一句就是（E10 未跑） | 是（丙b* 判红） |
| T1 | 同上，表里有那条决策、只回看了另一行 | 打中（窄） | A：只回看另一行，75 号绿；这一批在页内历史节记了带日期的条目时，④ 会抓到（F）；不记就抓不到 | 是 |
| T1 | 10 号剩下那段与 75 号 ⑨「是不是同一件事」 | **不是同一件事** | 三页分得开：已跑且作废、没人引（10 红、75 绿）；表里一行「备料」、正文没写「**备料**：」（10 红、75 绿）；决策只在自己的历史版本里提到它、表里只有「不影响」（10 绿、75 红）。真仓今天没有分歧的页 | — |
| T1 | 75 号本身（戊把射程交给它） | **打中（不分辨臂）** | 75:53 不判 `gate_changed_paths` 的退出码：git 失败时 ⑤ 什么都没比，照样报绿（t1-scenes 第一族最后一格：同一改动 75 号从退 1 变成退 0） | 否（丙b* 走同一个共用脚本，判了退出码就不中；戊、丙共用 75 这一处） |
| T5 | `gate_added_lines` | **打中但改法在手** | 四种输入给错：路径带空格（61 号新版退 77、HEAD 版退 1）；新增一行以「++ 」开头，之后的行都记到假路径上（88 号新版退 77、HEAD 版退 1）；同目录一份未跟踪的二进制文件（vim 交换文件）让 40 号第三道一行都没判（新版退 0、HEAD 版退 1）；`mv` 与 `git mv`、`diff.renames` 配置三种情形判得不一样 | 否（形态内的实现缺陷） |
| T5 | 40 / 88 改写前后 | **打中** | 没有上游（功能分支没设 upstream、游离 HEAD、只在本地的仓）时，HEAD 版「取不到基准就判全部」，新版取到 HEAD，已提交没推的页里点名的假产物、抄错的产物行都不判了（C7：40 号 1→0，88 号 1→77）；88:22 头部那句「拿不到 diff 基准时退回全量判」在这种状态下不成立 | 是（HEAD 版判红） |
| T5 | `gate_changed_paths` 失败外传 | **打中** | 共用脚本失败退非 0 了，十处调用只有 61、92 判；11、56（两处）、68、69（两处）、72（两处，别的会话的）、75、97 不判。索引写坏时 56 号从退 1 变成退 77（C8） | 否 |
| T5 | 64 号判据 ① ② | **打中（漏判 8 种、误判 3 种）** | 漏：`diff … @{u}...`（三点号隐含 merge-base）、`HEAD~1` 写死、参数列表换行后以引号开头、`core.quotepath=true`、同文件里有个叫 `git` 的包装函数、同一行有别的 `-z`、`printf` 开头、`@{u}` 短写；误：`unset GATE_BASE`、`merge-base --is-ancestor` 核登记提交、读 `refs/sop/gate-ok` 报上次全绿 | 否（判据写法） |
| T6 | 乙：按阶段位置调仓里的脚本、合成样本三份 | **打中但改法在手**（第二次抽样打中，与第一轮「没打中」不一致） | 九个变异里五个让三道一起放过：产物行不比种类 / 不比操作数 / 不比状态数 / 产物里没这条 path 就跳过 / 只比第一行与整条流。47 号的 `--selftest` 改坏的两刀都落在「mkfs 种根」那一行，它走钉活代码的用例、不走产物比对；乙的红样本只改段序列。红样本改成五行各一种错之后，九个变异全部判得出（本腿提的，量过、被攻过零轮） | 否（甲也放过；是样本内容，不是位置） |
| T11 | 第一轮判决「两者只在『全推出去了』这一种仓状态下不同」 | **打中（前提）** | 八种仓状态里六种不同：还有 master 没配上游、功能分支没配上游、功能分支上游是它自己、游离 HEAD、有 gate-ok、落后上游 | — |
| T11 | 候选「不认」 | **打中** | 没有上游的三种状态（S3、S4、S6）下，定案已提交、未定项没碰，61 号退 77；61 号红样本注释说修掉的正是这个形状，只在配了上游时成立 | 是（「认」判红） |
| T11 | 候选「认」 | **打中** | 全推出去了（S1）时窗口放到 HEAD~1：这一批照归档规则删掉上一轮的产物，88 号就判上一轮提交里抄的那行（退 1；「不认」退 77），与 88:19「只判这次改动新增或改写的行」冲突；第一轮已报的 68 号放过另算 | 是 |
| T11 | 认它时怎么防漏进样本 | **打中「解析得到提交就认」；「只认 40 位十六进制」没打中** | `GATE_BASE=master bash gate.sh` 时 gate.sh 导出 `GATE_DIFF_BASE=master`（lib.sh:173 原样交回），样本自检只清 GATE_BASE；样本仓里 master 就是它自己的 HEAD，61 号红样本判错（期望 1、实测 77），共用脚本自己的 `--selftest` 也红 | — |

## T1　戊：撤掉 10 号 §3，射程交 75 号 ⑤

### 被判的原文（行号现查）

- `.claude/gate.d/10-kb-rot.sh:16` `# 「实验改成已跑、引用它的决策有没有同批回看」不在这里判：门禁 75 号 ⑤ 把实验页标题行的变动算作正文改了，`
- `.claude/gate.d/10-kb-rot.sh:17` `# 要求同一批回看影响的决策表、写「改了」的那条决策文件在同一批里（三方判决 gate-fix-forks-r1 的 T1）。`
- 被撤掉的那一段的出路（`git show HEAD:.claude/gate.d/10-kb-rot.sh` 第 80 行）：`    howto "把这个实验的结论写回它支撑的那条决策，与状态变动同批提交；" \`
- `.claude/gate.d/75-decision-experiment-links.sh:6` `#      实验正文（历史版本与这一节之外）提到的每条决策（`D<号>（`）都要有一行。`（①，从实验一侧看）
- `.claude/gate.d/75-decision-experiment-links.sh:10` `#      分项「**依据**」段引的每个实验，那个实验页里要有这一行、关系是支撑或推翻。待回填清单里的决策不判这一条。`（③，决策一侧只看「**依据**」段）
- `.claude/gate.d/75-decision-experiment-links.sh:15` `#      这次改动要在那张表里改过至少一行；新加的三方判决 research/prompts/*-main-verification.md`（⑤）

### 造出来的批（t1-scenes.sh；跑的是真仓现在的 75 号、10 号，丙b* 是 t1-judge.py）

基线：D1 已定项 1 的「**依据**」引 E1、「**欠**」块写「E2（样本实验二） 跑完再核射程；E4（样本实验四） 跑完再核定案」；E1 已跑、表里 D1 支撑；E2 待跑、表里只有 D3 备料；E4 待跑、表里 D1 备料与 D3 不影响两行。基线上 75、10 都退 0，丙b* 绿。

| 批 | 75 号 | 10 号 | 丙b* |
|---|---|---|---|
| A　E4 改成已跑、加一句结论，只回看 D3 那一行（D1 一个字没动） | 退 0 | 退 0 | 红：`E4 改成已跑，引用它的 .claude/kb/decisions/01-甲.md 不在这一批里` |
| F　同 A，另在 E4 页内历史节记 `### 2026-09-24` | 退 1：`[回看过期] … E4 最近一次变动在 2026-09-24，对 D1 的回看停在 2026-09-18` | 退 0 | 红 |
| B　E2 改成已跑（页内也记了 2026-09-24），回看 D3 那一行；D1 只在「**欠**」块里点名 E2 | **退 0** | 退 0 | 红 |
| 对照　同 B，D1 的「**欠**」块跟着改 | 退 0 | 退 0 | 绿 |
| 对照　E4 只改标题成已跑，表一行都没回看 | 退 1：`[没回看]` | 退 0 | 红 |
| 同上一格，另把 `.git/index` 写坏（`git diff` 退 128） | **退 0**（`这次改动触发回看 0 处`） | 退 0 | 红：`取不到改动范围（共用脚本退 1）` |

**B 是这一格要找的那一批**：D1 在等 E2（「**欠**」：E2 跑完再核射程），E2 跑完了，这一批没回头改 D1，而 75 号三条都看不见——① 只从实验正文看（E2 正文没写 `D1（`），③ 只看「**依据**」段，⑤ 只要表里任一行被改过。10 号旧出路第一句要的正是「把结论写回它支撑的那条决策」。F 说明 ④ 能补 A 那一种（有那一行、页内记了带日期的条目时），补不了 B（表里根本没有那一行）。

**真仓里有这种对子**（t1-scan.py 原样输出）：

```
一、对子（D 不在待回填、E 不在待回填、E 的表里没有 D）：D 的文件里任一处按 10 号的正则提到 E 的 25 对；其中 D 的正文（历史版本之前）用「E<号>（」形态提到的 25 对，这里面 E 的标题还没有「已跑」的 3 对
   正文形态的前 12 对：D2→E47、D2→E55、D8→E28、D8→E54、D8→E71、D8→E90、D9→E17、D11→E103、D13→E3、D14→E103、D14→E20、D16→E58
```

还没跑的 3 对是 D13→E3、D25→E21、D26→E10。D26 那一对就是 B 的形状，`.claude/kb/decisions/26-后台整理与放置回收.md:29`：

> **阈值数值不进格式**：等 E10（老化：结构抗老化 vs 事后整理），在那之前取保守默认、做成挂载可调。改第一个事务的字节：否——三个量已由已定项 6 定为记账树里的统计量、第一个事务已经在写它们；起跑与停机是判定的形式，`segs0` 住意图记录而第一个事务不含整理意图。

E10 的影响表只有 D3、D10 两行（`grep -n '^| \**D' .claude/kb/experiments/10-*.md` 命中第 25、26 行），没有 D26。E10 跑完的那一批只要回看 D3 或 D10 一行，75 号就放行，D26「等 E10」那句留着。

用户决定的几步放开扫过：回看哪几行（只回看别的行 / 一行不回看）、页内记不记带日期的条目、改不改那份决策。只固定前缀（D 在「**依据**」段外提到 E、E 的表里没有 D）与「E 改成已跑」这一步；B 在「记 / 不记日期条目」两种下 75 号都退 0。

### 10 号剩下的「已跑实验要有决策引用」与 75 号 ⑨：不是同一件事

`.claude/gate.d/10-kb-rot.sh:99` 按子串认已跑：`  mapfile -t ran_experiment_lines < <(cat "${experiment_body_files[@]}" | grep -E "^## E[0-9]+ " | sed 's/^## //' | grep "已跑")`；`.claude/gate.d/75-decision-experiment-links.sh:25` `#      标题状态里写着「作废」或「退役」的实验不判（它们留着是为了记下那条路不通）。`

| 页（各在基线上加一页，t1-scenes 第二族） | 10 号 | 75 号 |
|---|---|---|
| C　`—— 已跑，结论作废`，没有决策提到它，表里一行 D3 不影响 | 退 1：`E5 已跑，但决策正文一次都没引用它——它的结论悬空了` | 退 0（⑨ 按「作废」不判） |
| D　已跑，表里一行 `D3 … | 备料 |`，正文没写「**备料**：」、没有决策提到它 | 退 1 | 退 0 |
| E　已跑，表里只有「不影响」，D3 只在自己的「## 历史版本」里提到它 | 退 0 | 退 1：`[没对应决策]` |

C 与 75 号 ⑨ 直接冲突：10 号给作废的实验留的两条出路（`.claude/gate.d/10-kb-rot.sh:136` 那句「确实谁也不支撑的话，在实验正文里写一行「**备料**：……」，点名它等着的决策或欠账」，或者去某份决策里提它一次），对一个「那条路不通」的实验都是假话。D 是同一个事实（这个实验是某条决策的备料）要登记两遍、两种写法。真仓今天没有分歧页（t1-scan 第二段：138 + 6 + 4 + 1 页都两边过），E45 这类作废页今天过 10 号是因为恰好有决策提到它。

### 四句（T1）

| 打中 | 分不分辨臂 | 被判系统当时看不看得到 | 满足判据哪一分句 | 跑前条款的改法在这几格上 |
|---|---|---|---|---|
| B（D 在「**依据**」外提到 E，E 表里没 D） | 分辨：戊放行、丙b* 红 | 看得到：D 的文件与 E 的标题变动都在工作区 | 第一分句（t1-scenes B；真仓 D26→E10） | 正文没给改法。本腿提的：75 号 ① 加反向——决策正文（历史版本之前）用 `E<号>（` 提到的实验，表里要有这条决策一行（推的，没实现）；它会让真仓今天那 25 对当场红，要先回填 |
| A（表里有 D，只回看了别的行） | 分辨 | 看得到 | 第一分句 | 本腿提的：⑤ 在标题状态变了的那一批里，要求表里每一行都回看（推的）；F 说明 ④ 在记了日期条目时已经能抓 |
| 10 剩下那段与 ⑨ 不同 | 不分辨（戊、丙b* 都留着这段） | 看得到 | 第一分句（C、D、E） | 另记一笔：要么 10 号认「作废 / 退役」并认表里的「备料」行，要么把这段并进 75 号 ⑨ |
| 75:53 吞退出码 | **不分辨**（戊、丙都要靠 75 或共用脚本；丙b* 模型判了退出码所以不中，不是丙的性质） | 看得到（git 报了 fatal） | 第一分句 | 见 T5「调用方吞退出码」 |

**推翻条件**：75 号 ① 或 ③ 已经从决策一侧看「**依据**」段之外的 `E<号>（`（现查 75:6、75:10 与代码第 136–165 行，没有）；或者 kb 规则明写「决策正文只许在「**依据**」段里提实验」（那样 D26:29 的写法本身违规，B 的前提不成立）。

## T5　改动范围只许一份取法

### `gate_added_lines` 在哪种输入上给错（t5-added-lines.sh；「新版」是真仓现在的阶段，「HEAD 版」是 `git show HEAD:` 取出的旧阶段，同一现场）

被判的解析（`research/scripts/changed-paths.sh:98`）：`    /^\+\+\+ / { path = substr($0, 5); if (path == "/dev/null") path = ""; else sub(/^b\//, "", path); next }`。它在 diff 的任何位置都把「+++ 」当文件头，也不去掉 git 给带空格路径补在文件头末尾的制表符。

| 现场 | `gate_added_lines` 原样（`sed -n l` 转义） | 新版 | HEAD 版 |
|---|---|---|---|
| C1 决策文件叫 `97 样本.md`（带空格），工作区加「### 已定项 2 —— 已定」，未定项没碰 | `…/97 样本.md\t\t### 已定项 2 …`：路径后多一个制表符，行文开头多一个制表符，61 号 `cut -f2-` 之后的 `^#{2,4} ` 认不出 | 61 号**退 77**（`本次 diff 没有新增「已定」小节`） | 61 号退 1（`97 样本.md:7 本次新增了「已定」小节，而这条未定项一个字都没动`） |
| C2 实验页新增「++ 注：下一行是抄的产物」，下一行新增一条对不上的 `E7RESULT` | 第一行被当成文件头吞掉；下一行记在路径「注：下一行是抄的产物」名下 | 88 号**退 77** | 88 号退 1 |
| C3 实验页新点名 `e1-never-2026-09-24.out`（树里、历史里都没有）；同目录一份未跟踪的 vim 交换文件 `.01-样本.md.swp`（`.gitignore` 不管它，内含 NUL） | 交换文件整份按行吐出，含 NUL；40 号 `cut -f2- … \| grep -oE` 把整段当二进制，一个名字都不交（GNU grep 3.11，门禁用的就是它） | 40 号**退 0**（`新增或改写的行点名的 0 份产物`）；没有交换文件时退 1 | 40 号退 1（两种都是） |
| C4 页里一行早先抄下、已对不上产物的 `E7RESULT`，只把页 `mv` 改个名 | 未跟踪的新名整份算新增 | `mv` 后退 1；`git add -A` 后（暂存区是一次改名）**退 77**；再配 `diff.renames=false` 又退 1 | `mv` 后退 77；`git add -A` 后退 1 |
| C5 路径带制表符（跟踪的 / 未跟踪的） | `"b/k/t\\tx.md"\tb`（git 加了引号，路径对不上）；`k/u\ty.md\tc`（记录里多一个制表符，路径切错）；退 0 | — | — |
| C6 十四种 git 配置逐个配上，比输出与不配时是否逐字相同 | 只有 `diff.renames=false` 不同（多出改名后那份的整份旧行）；`diff.noprefix`、`diff.mnemonicPrefix`、`diff.relative`、`color.*`、`diff.algorithm`、`diff.context`、`diff.interHunkContext`、`diff.external`、`core.quotepath=true` 都相同 | — | — |

C4 的同一次搬家，88 号判不判取决于跑门禁前有没有 `git add`、本机配没配 `diff.renames`——与第一轮 T1 S4 / S5（`git mv` 判绿、git 认不出改名的拆分判红）是同一类。`changed-paths.sh:16` 自己写了「路径里带制表符、换行或引号的，git 照样转义，这一条管不到」，那是 `gate_changed_paths` 的限度；带空格的路径不在这句里，C1 是它没写到的一格。真仓今天没有带空格、带制表符的路径，也没有以「++ 」开头的 md 行（`git ls-files | grep -c ' '` → 0；`git grep -c -E '^\+\+ ' -- '*.md'` 零命中），C1、C2、C5 是以后会中的形状；C3 只要有人在 `.claude/kb/experiments/` 下用 vim 开着一页就中。

### 40、61、88、92 改写前后判得不同的输入

上面 C1（61）、C2（88）、C3（40）、C4（88，两版在 `git add` 前后翻向相反）都是。另一格与输入无关、与仓状态有关：

| C7 功能分支没配上游、没有 gate-ok；实验页已提交，点名一份不存在的产物、抄一行对不上的 E7RESULT；工作区干净 | 新版 | HEAD 版 |
|---|---|---|
| 40 号 | 退 0（`这次改动（基准 HEAD）新增或改写的行点名的 0 份产物…`） | 退 1 |
| 88 号 | 退 77 | 退 1（`全量 kb 正文里有 1 行…`） |

HEAD 版在「gate-ok、上游都没有」时基准为空，判全部；新版的 `gate_diff_base gate` 此时交回 HEAD（`changed-paths.sh:10` `#     gate：GATE_BASE 是个提交就用它；否则 @{upstream} 的 merge-base；都没有（或 merge-base 取不到）就 HEAD。`），只判工作区。88 号头部 `.claude/gate.d/88-quoted-result-lines.sh:22` `#      拿不到 diff 基准时**退回全量判**（保守：宁可多判，不可漏判）。` 与 40 号 `.claude/gate.d/40-results-cited.sh:86` 那句「不在 git 仓里、或取不到改动范围，就判全部（保守）」在这种状态下不再成立：没有上游不算「取不到」，窗口悄悄收成了工作区。同一时刻上游 `diff_base` 取到的是 master 的 merge-base（C7 原样：`上游版 lib.sh diff_base 在这里取到 …（master 的 merge-base），共用脚本 gate 取法取到 HEAD`）。这一格与 T11 同根。

92 号：新旧两版的基准取法相同（GATE_BASE → 上游 merge-base → HEAD），改的是路径带 quotepath、merge-base 取不到时不再交空串、git 失败判红；除了中文 checker 路径（新版的绿样本就是它）之外，没找到两版判得不同的输入。

### 共用脚本失败退非 0 了，调用方大多不判（C8）

`research/scripts/changed-paths.sh:20` `#     路径可以是目录。git 失败时退出码非 0、不输出半截结果，调用方按「拿不到改动范围」处理（通常退回全量判）。` 真仓现查（C8 原样），调 `gate_changed_paths` 而不判它退出码的：

```
    .claude/gate.d/11-batch-scope.sh:50:changed="$(gate_changed_paths "$(gate_diff_base head)" untracked)"
    .claude/gate.d/56-crates-adversarial-review.sh:24:changed="$(gate_changed_paths "$base" untracked)"
    .claude/gate.d/56-crates-adversarial-review.sh:30:added="$(gate_changed_paths "$base" untracked A)"
    .claude/gate.d/68-knowledge-sync.sh:73:changed="$(gate_changed_paths "$base" untracked)"
    .claude/gate.d/69-evidence-in-repo.sh:61:changed="$(gate_changed_paths "$base" untracked)"
    .claude/gate.d/69-evidence-in-repo.sh:62:added="$(gate_changed_paths "$base" untracked A)"
    .claude/gate.d/72-agent-def-adversarial-review.sh:24:changed="$(gate_changed_paths "$base" untracked)"
    .claude/gate.d/72-agent-def-adversarial-review.sh:30:added="$(gate_changed_paths "$base" untracked A)"
    .claude/gate.d/75-decision-experiment-links.sh:53:  changed="$(gate_changed_paths "$base" untracked)"
    .claude/gate.d/97-invariant-field-anchors.sh:36:added="$(gate_changed_paths "$base" untracked A)"
```

判的只有 61:41 与 92:56。复现：改一份 `crates/a/src/lib.rs`、把 `.git/index` 写坏（`git diff --name-only HEAD` 退 128，`gate_changed_paths` 退 1），56 号从退 1（`crates 改动里 1 个文件没有三方判决点名`）变成**退 77**（`这次改动没碰 crates/*/src`）；75 号从退 1 变成退 0（见 T1 表最后一格）。第一轮判决给 92 号定的洞「git 失败时集合为空，被读成『这次什么都没碰』」，今天还开在这十处里。64 号不判这一条。

### 64 号的判据：漏判与误判的写法（t5-64.sh，跑真仓现在的 64 号，每种写法一份阶段文件）

| 写法 | 64 号点名 | 该不该点名 |
|---|---|---|
| `changed="$(git -c core.quotepath=false diff --name-only @{u}... --)"`（三点号就是 merge-base） | 0 行 | 该：自己算了基准 |
| `… diff --name-only HEAD~1 --` | 0 行 | 该：自己定了基准 |
| python 参数列表换行，`'--name-only', base, '--'],` 那一行以引号开头 | 0 行 | 该：列路径没带 quotepath |
| `git -c core.quotepath=true ls-files --others …` | 0 行 | 该：照样转义 |
| 同一文件里有 `def git(*args):`（带 quotepath 的包装），另一行直接 `subprocess.run(['git', 'ls-files', '--others', …])` | 0 行 | 该：这一行没走包装 |
| `[[ -z "${SKIP:-}" ]] && git ls-files --others … > "$list"` | 0 行 | 该：`-z` 是 test 的 |
| `printf '%s\n' "$(git ls-files --others …)" > "$list"` | 0 行 | 该：以 printf 开头被当成出路文字 |
| `base="$(git rev-parse @{u})"` | 0 行 | 该 |
| `unset GATE_BASE GATE_STAGED_FROM`（阶段嵌套跑样本前清环境） | 1 行 | 不该：没读它 |
| `git merge-base --is-ancestor "$recorded_commit" HEAD \|\| bad …`（核登记的提交在历史里） | 1 行 | 不该：不是改动范围 |
| `last_green="$(git rev-parse -q --verify refs/sop/gate-ok \|\| true)"` | 1 行 | 不该（可争） |

机制，`.claude/gate.d/64-change-range-single-source.sh` 现查：第 30 行 `BASE_PATTERN = re.compile(r'merge-base|@\{upstream\}|gate-ok|\bGATE_BASE\b|\bGATE_DIFF_BASE\b')` 只认字面，不认 `@{u}`、`...`、`HEAD~N`，也不分读与写；第 32 行 `TEXT_PREFIX` 把以引号、`printf` 开头的行整行剔掉；第 70 行 `                and not any(re.search(r'(\$\{?|\b)' + re.escape(name) + r'\b', line) for name in names):` 只看包装函数的名字在不在这一行，名字恰好是 `git` 时（75 号第 81 行 `def git(*args):` 就是）同文件里每一行裸 git 都算过；第 36 行 `NO_QUOTING` 的 `-z` 不问是谁的参数；quotepath 只查出现，不查取值。真仓 gate.d 今天没有这些写法的实例（`grep` 引号开头的列路径行、`@{u}`、`...`、`HEAD~1` 都零命中），64 号今天判得对，漏的是以后的写法。

### 本腿给 `gate_added_lines` 的改法，打在副本上重跑同一批现场（t5-fix.sh；只在我的模型上量过、被攻过零轮）

改法：「+++ 」只在 `diff --git` 与第一个 `@@` 之间认；剥掉文件头末尾的制表符；文件头路径被 git 加了引号、或未跟踪路径含制表符，就退非 0（调用方退回全量判或判红）；diff 写死 `--no-renames`；未跟踪文件 `grep -Iq .` 认成二进制（或空）的不读。补过的共用脚本 `--selftest` 11 项全过，`LIB_CHANGED_PATHS_BREAK=quotepath`、`=untracked-lines` 两种弄坏仍判红（t5-fix.out 前三行）。

| 格 | 补丁前 | 补丁后（t5-fix.out） | 量过 / 推的 |
|---|---|---|---|
| C1 路径带空格 | 61 号退 77 | 61 号退 1 | 量过 |
| C2 「++ 」开头的新增行 | 88 号退 77 | 88 号退 1 | 量过 |
| C3 未跟踪的二进制文件 | 40 号退 0 | 40 号退 1 | 量过 |
| C4 `mv` / `git add` / `diff.renames=false` | 退 1 / 77 / 1 | 退 1 / 1 / 1 | 量过。代价：搬一次家，页里早先抄下的行全算新增；88:19 的「只判这次改动新增或改写的行」按字面不该判它们。另一个方向（写死 `-M`）要给未跟踪的新名找它删掉的旧名，没实现 |
| C5 路径带制表符 | 退 0、路径给错 | 退 1 | 量过 |
| C6 配置扫描 | `diff.renames=false` 不同 | 十四种全相同 | 量过 |
| C7 没有上游时窗口收成工作区 | — | 不碰（基准取法的事，见 T11） | — |
| C8 调用方不判退出码 | — | 不碰。改法：十处都照 61:41 的写法判；64 号加第 ③ 条「`gate_changed_paths` / `gate_added_lines` 的调用要判退出码」 | 推的 |
| 64 号漏判 8 种 | — | 不碰。改法：① 加 `@\{u\}`、`\.\.\.`、`HEAD~\d`；② 查 `quotepath=false` 的取值、包装函数认「这一行调的是它」而不是「这一行有它的名字」、多行调用按语句拼起来再判 | 推的 |
| 64 号误判 3 种 | — | 不碰。改法：① 只认读（`$GATE_BASE`、`${GATE_BASE`、`environ`），`--is-ancestor` 与只读 `gate-ok` 的放行 | 推的 |

### 四句（T5）

| 打中 | 分不分辨臂 | 被判系统当时看不看得到 | 满足判据哪一分句 | 跑前条款的改法在这几格上 |
|---|---|---|---|---|
| C1、C2、C3、C5（`gate_added_lines` 解析） | 不分辨甲乙丙（第一轮判的是「收成一份」，这是那一份自己的缺陷） | 看得到：diff 与文件都在 | 第一分句 | 上表补丁量过修得了；64 号修不了（它查的是阶段，不是共用脚本） |
| C4（同一批，`git add` 前后判得相反） | 不分辨 | 看得到 | 第一分句 | 补丁写死 `--no-renames` 修得了翻面，代价见上表 |
| C7（没有上游） | 分辨：HEAD 版判全部 | 看得到：没有上游是 git 答得出的 | 第一分句；另与 88:22 原文直接冲突（第二分句） | 64 号修不了；见 T11 |
| C8（十处吞退出码） | 不分辨 | 看得到：git 报 fatal | 第一分句 | 第一轮判决只修了 61、92 两处 |
| 64 号漏判、误判 | 不分辨（丙本身就是 64 号；甲乙没有判据） | — | 第一分句（t5-64.out） | 推的改法见上表 |

**推翻条件**：门禁跑时 `grep` 不是 GNU grep ≥ 3.5 或别的遇 NUL 仍逐行输出的实现（那样 C3 在 40 号上不中；`gate_added_lines` 仍交出带 NUL 的行）；`.gitignore` 或全局 excludes 里加了 `*.swp`（C3 要换一种未跟踪的二进制文件才中）；gate.sh 在「没有上游」时给阶段设了 GATE_BASE（现查 gate.sh 只在 `--staged` 里层设，C7 不成立时要看到别的设法）。

## T6　52 号：乙形态的第二次抽样

### 被判的原文

- `.claude/gate.d/52-segment-registry.sh:21` `#   钉活代码的那几句与真产物的解析由脚本自己的 --selftest（47 号跑）拿真文件测，样本不再测一遍。`
- 第一轮攻方的依据（`research/prompts/gate-fix-forks-r1-opus-output.md:299`）说 `--selftest` 拿真文件各改坏一处在测。
- `research/scripts/check-segment-registry.py:385` `    """--selftest 用：把八节表格第一条可比对的行，段序列的最后一项加 1。`

**机制**：今天八节表格第一条「不是预想、不是装置钉住、带段序列」的行是「mkfs 种根」，它的出处栏指到一份钉活代码的用例，走 `compare_against_pinning_test`，不走 E142 产物比对。真仓 `--selftest` 原样：`把「mkfs 种根」的段序列改成 `12+1+1+1+5` 之后判红（退出码 1）；只把「mkfs 种根」第一段的种类 `zero_fill×8` 改成 `zero_fill×9` 也判红`。所以 `--selftest` 的两刀都不经过产物比对那一段（`check-segment-registry.py` 第 465–534 行的 `for entry in entries:` 循环里 `product_path_name` 不为空的那一支）；那一支今天只由乙的红样本测，而红样本只改了段序列。

### 九个变异 × 三道检查（t6-mutants.sh；现场是真 layout、replay.sh、E142 产物、第八节点名的 4 份 .rs 拷进草稿目录，52 号与乙的样本原样拷过去）

「埋错」是在同一份真输入上改一处，原脚本判红、变异后看判不判得出。原样输出：

```
无                            | 47 --selftest 退 0 | 样本自检退 0（判错 0 条）| 埋错「无」：原脚本 52 号退 0，变异后 52 号退 0
M0 main 不传退出码（第一轮的变异） | 47 --selftest 退 0 | 样本自检退 1（判错 1 条）| 埋错「普通发布段序列」：原脚本 52 号退 1，变异后 52 号退 0
M1 产物行不比种类       | 47 --selftest 退 0 | 样本自检退 0（判错 0 条）| 埋错「取号行种类」：原脚本 52 号退 1，变异后 52 号退 0
M2 产物行不比操作数    | 47 --selftest 退 0 | 样本自检退 0（判错 0 条）| 埋错「取号行操作数」：原脚本 52 号退 1，变异后 52 号退 0
M3 产物行不比状态数    | 47 --selftest 退 0 | 样本自检退 0（判错 0 条）| 埋错「取号行状态数」：原脚本 52 号退 1，变异后 52 号退 0
M4 不比整条流那句       | 47 --selftest 退 0 | 样本自检退 1（判错 1 条）| 埋错「整条流状态数」：原脚本 52 号退 1，变异后 52 号退 0
M5 产物里没有这条 path 就跳过 | 47 --selftest 退 0 | 样本自检退 0（判错 0 条）| 埋错「产物缺取号行」：原脚本 52 号退 1，变异后 52 号退 0
M6 没写种类也放过       | 47 --selftest 退 0 | 样本自检退 0（判错 0 条）| 埋错「取号行没写种类」：原脚本 52 号退 1，变异后 52 号退 1
M7 只比第一行与整条流 | 47 --selftest 退 0 | 样本自检退 0（判错 0 条）| 埋错「普通发布段序列」：原脚本 52 号退 1，变异后 52 号退 0
M8 产物行不比段序列    | 47 --selftest 退 0 | 样本自检退 1（判错 3 条）| 埋错「普通发布段序列」：原脚本 52 号退 1，变异后 52 号退 0
```

**打中**：M1、M2、M3、M5、M7 五个变异让仓里的脚本退化（真输入上埋的错判不出），而 `--selftest` 绿、乙形态的样本判对，三道一起放过。M1 是「每段步骤种类多重集」——等价类判据的一半（脚本第 515 行自己写着「每段的步骤种类多重集是等价类判据的一半（D17 已定项 2）」）；M7 让表格第二行起的产物比对全部失效（真表里是「取号」「暖机」「普通发布」三行）。M6 不算退化：后面的种类比对照样判红。M8 说明乙的样本确实补上了 `--selftest` 够不着的段序列那一格——第一轮说乙抓得到 M0，这一轮复现了。

**改法（本腿提的，只在我的模型上量过、被攻过零轮）**：乙的红样本改成五行，每行一种错（段序列 / 种类 / 操作数 / 状态数 / 产物里没有这条 path），`expect` 逐条 `want`；仍然只是 layout、replay.sh、产物三样合成输入。原样输出：

```
  无                            | 改过的样本自检退 0（判错 0 条）
  M0 main 不传退出码（第一轮的变异） | 改过的样本自检退 1（判错 1 条）
  M1 产物行不比种类       | 改过的样本自检退 1（判错 2 条）
  M2 产物行不比操作数    | 改过的样本自检退 1（判错 2 条）
  M3 产物行不比状态数    | 改过的样本自检退 1（判错 2 条）
  M4 不比整条流那句       | 改过的样本自检退 1（判错 1 条）
  M5 产物里没有这条 path 就跳过 | 改过的样本自检退 1（判错 2 条）
  M7 只比第一行与整条流 | 改过的样本自检退 1（判错 5 条）
  M8 产物行不比段序列    | 改过的样本自检退 1（判错 2 条）
```

五行的红样本全文在 t6-mutants.sh 第 93–120 行（layout 与 expect 两段 heredoc，产物两行在第 110–111 行）。

### 四句（T6）

| 打中 | 分不分辨臂 | 被判系统当时看不看得到 | 满足判据哪一分句 | 跑前条款的改法在这几格上 |
|---|---|---|---|---|
| M1、M2、M3、M5、M7 三道一起放过 | **不分辨甲乙**：甲（拷全真输入、红样本用 `mutate_one_segment_number`）改的也是 mkfs 那一行，同样放过；病根在「红样本只测一个分支」，不在样本指向哪一份脚本 | 看得到：埋的错就在 layout 与产物里 | 第一分句（t6-mutants.out） | 正文没给改法；上面的五行红样本量过修得了全部五格 |

它与第一轮的「没打中」不一致：按 `three-way-inference.md`「一条腿只抽一次样不算一次观测」那一节，两次不一致记「不稳定」，而这一次给的是可复现的具体变异——打中那一侧一次就值得去核（同一节的表）。乙的「按阶段位置调仓里的脚本」这一半两次都没被打中；打中的是「样本只放三样合成输入、红样本只改段序列」里「只改段序列」那一半。

**推翻条件**：有别的检查在 M1–M5、M7 下会红——我在 gate.d 里只找到 47、52 两处调这份脚本（第一轮同一条 `grep -ln check-segment-registry .claude/gate.d/*.sh` 的结果）；或者 `mutate_one_segment_number` 选中的第一行不是钉活代码的那一行（表格行序改了，`--selftest` 那一刀就会落到产物比对上，M8 那一格改由它抓）。

## T11　共用脚本认不认 `GATE_DIFF_BASE`

### 第一轮判决的前提不成立：两种取法不止在「全推出去了」时不同

`research/prompts/gate-fix-forks-r1-main-verification.md:78` 整行：

> 主 agent 现查坐实：`gate.sh:115–123` 算好 `GATE_DIFF_BASE` 导给每道阶段，注释的起因是「同一轮里两个阶段判的不是同一批改动……审计实测于使用者项目的一个本地阶段」；`.claude/gate.d/`、`research/scripts/`、`.claude/scripts/`、`.claude/hooks/` 里读它的 0 处。今天 `diff_base` → `896b73f`（HEAD~1），`gate_diff_base gate` → `3b60f09`（HEAD）。两者只在「全推出去了」这一种仓状态下不同：上游在 `merge-base` 等于 HEAD 时往回退到 `gate-ok` 或 HEAD~1。

t11-bases.sh 原样（三个提交 c1–c3，提交名是提交说明；`diff_base` 是上游 lib.sh 的，`gate_diff_base gate` 是共用脚本的）：

```
  S1 master 全推出去了，工作区有改动 diff_base → c2        gate_diff_base gate → c3
  S2 master 本地多一个没推的提交 c3  diff_base → c2        gate_diff_base gate → c2
  S3 master 没配上游                       diff_base → c2        gate_diff_base gate → c3
  S4 功能分支 c4，没配上游            diff_base → c3        gate_diff_base gate → c4
  S5 功能分支 c4，上游是它自己、全推出去了 diff_base → c3        gate_diff_base gate → c4
  S6 游离 HEAD（在 c3 上）               diff_base → c2        gate_diff_base gate → c3
  S7 master 全推出去了，gate-ok 在 c2   diff_base → c2        gate_diff_base gate → c3
  S8 master 落后上游一个提交（fetch 了没合） diff_base → c1        gate_diff_base gate → c2
```

八种里只有 S2 相同。差在两处回退：共用脚本没有上游、或上游 merge-base 就是 HEAD 时都交 HEAD；上游 `diff_base` 先比 master / main 的 merge-base（功能分支取分叉点），在默认分支上再退到 gate-ok 或 HEAD~1（`.claude/singlefs-ai-sop/scripts/lib.sh:188` `        # 攒多少次都躲不掉。没有远端跟踪分支时才退回 HEAD~1。`）。

### 两个候选各让哪一道判错

「认」这一臂用 `GATE_BASE="$(diff_base)"` 跑真阶段模拟（共用脚本第一档就认 GATE_BASE，取到的提交与「先认 GATE_DIFF_BASE」相同）。

| 仓状态与历史 | 不认（今天的代码） | 认 |
|---|---|---|
| S3 / S4 / S6：决策文件先提交；再提交一次，加「已定」小节、未定项没碰；工作区另改一个无关文件 | 61 号**退 77**（三种状态都是：`本次 diff 没有新增「已定」小节，本阶段无对象可判`） | 61 号退 1（`97-样本.md:7 本次新增了「已定」小节，而这条未定项一个字都没动`） |
| S1：上一轮提交抄了产物行并已推；这一批照归档规则删掉上一轮的产物、换上新产物、页里点名新产物 | 88 号退 77 | 88 号**退 1**（`这次改动新增或改写的 kb 正文里有 1 行整行抄的产物行…一份都找不到`：判的是上一轮已推的那一行，产物的删除还没提交，`git log --diff-filter=D` 看不到它） |
| S1：第一轮已报的 68 号（HEAD~1 把上一次提交的同步记录算进这一批，顶了 7 个文件的账） | 判对 | 放过 |
| 另见 T5 C7：S4 下 40、88 号新版只判工作区 | 40 退 0、88 退 77 | （推的，没跑：窗口是 master 的分叉点，已提交的那一页进窗口，应判红） |

- 「不认」在没有上游的三种状态下，放过的正是 61 号红样本注释说修掉的那个形状：`.claude/gate.d/fixtures/61-settled-same-file.sh/red/setup.sh:4` `# 定案已经提交在本地而没推，正是「在默认分支上 merge-base 就是 HEAD」那种基准会漏掉的形状（三方判决 gate-fix-forks-r1 的 T5）。` 红样本配了上游（setup.sh 第 8 行 `git branch 上游 && git branch -q --set-upstream-to=上游 master`），所以判得对；拿掉上游那一行就是 S3。上游自己的理由 `lib.sh:182` `        # 退一格（HEAD~1）不够：**分两次提交就绕过去了**——第一次改代码不带测试，` 说的也是这一种。
- 「认」在 S1 下与 `.claude/gate.d/88-quoted-result-lines.sh:19` `#   4. **只判这次改动新增或改写的行**（用户 2026-09-21 定）：正文里早先抄下的行是历史，仅作参考——` 冲突：上一轮已推的那一行不是这一批写的，而这一批删上一轮的产物是归档规则要它做的事。它能靠先提交这次删除消掉（提交之后 `--diff-filter=D` 找得到），改工作区里任何一个文件都消不掉。

用户决定的几步放开扫过：提交几次、推没推、在哪个分支、游离与否（八种状态）；「认」的 88 号那一格，另在 t11-bases.sh 留下的现场上手跑了「先提交删除」这一步（不在模型脚本里）：`git commit -qam 归档上一轮产物 && git add -A && git commit -qm 第二轮` 之后 `diff_base` 取到「第一轮抄行」，88 号原样 `  ! 这次改动新增或改写的 kb 正文里没有整行抄的 E7RESULT 行，本阶段无对象可判`、退 77；再把上游挪到 HEAD（推出去）之后 `diff_base` 取到「归档上一轮产物」，仍退 77。所以这一格的红只在「删除还没提交」时有，提交就消。

### 认它时怎么防真仓的基准漏进样本（t11-leak.sh：40、61、88 三道与样本、补丁后的共用脚本拷进草稿目录，跑上游 stage-selftest）

漏法：`.claude/singlefs-ai-sop/scripts/stage-selftest.sh:65` `    out="$(cd "$work" && env -u GATE_BASE -u GATE_STAGED_FROM bash "$stage" "$work" 2>&1)" || got=$?` 只清两个；`gate.sh:122` `GATE_DIFF_BASE="$(diff_base "$ROOT")"`；而人指定窗口时 `lib.sh:173` `    printf '%s' "$GATE_BASE"; return 0` 原样交回，`gate.sh:334` 的出路文字就教人写 `GATE_BASE=<ref> bash .claude/scripts/gate.sh`。原样输出：

```
  认法甲  GATE_DIFF_BASE=（不设）                              stage-selftest 退出码 0；判错：
  认法甲  GATE_DIFF_BASE=3b60f098e97dc4c4f3ed9c6355422b607db1c34c  stage-selftest 退出码 0；判错：
  认法甲  GATE_DIFF_BASE=master                                    stage-selftest 退出码 1；判错：61-settled-same-file.sh red 期望退出 1，实测 77;61-settled-same-file.sh red 输出里找不到「本次新增了「已定」小节，而这条未定项一个字都没动」;
  认法甲  GATE_DIFF_BASE=HEAD                                      stage-selftest 退出码 1；判错：61-settled-same-file.sh red 期望退出 1，实测 77;61-settled-same-file.sh red 输出里找不到「本次新增了「已定」小节，而这条未定项一个字都没动」;
  认法乙  GATE_DIFF_BASE=（不设）                              stage-selftest 退出码 0；判错：
  认法乙  GATE_DIFF_BASE=3b60f098e97dc4c4f3ed9c6355422b607db1c34c  stage-selftest 退出码 0；判错：
  认法乙  GATE_DIFF_BASE=master                                    stage-selftest 退出码 0；判错：
  认法乙  GATE_DIFF_BASE=HEAD                                      stage-selftest 退出码 0；判错：
```

认法甲 =「`rev-parse --verify` 解析得到提交就认」；认法乙 = 另要求值是 40 位十六进制。真仓的提交号在样本仓里解析不到，两种认法都退回去，没事；**符号名**（`master`、`HEAD`）在样本仓里解析得到——样本仓都是 `git init -b master`，master 就是它自己的 HEAD——认法甲当场把 61 号红样本判成 77。同一条漏法走 47 号：补丁后的 `changed-paths.sh --selftest` 在认法甲、`GATE_DIFF_BASE=master` / `HEAD` 下退 1（`GATE_BASE 是个提交就用它：期望「…」`），认法乙退 0。

认法乙的代价（推的）：人指定 `GATE_BASE=master` 时，阶段第一档照旧认 GATE_BASE，窗口不变；只有「人没指定、diff_base 又交回字面 HEAD」（仓里既没有 master 也没有 main）那一种，认法乙不认它、退回共用脚本自己的取法。另一种防法是由样本自检清掉 `GATE_DIFF_BASE`，那要改上游 stage-selftest.sh，照正文第六节只能写成「要等上游」。

### 四句（T11）

| 打中 | 分不分辨臂 | 被判系统当时看不看得到 | 满足判据哪一分句 | 跑前条款的改法在这几格上 |
|---|---|---|---|---|
| 前提「只在全推出去了时不同」 | —（判决前提） | 看得到 | 第一分句（t11-bases.out 八行） | — |
| 「不认」：S3 / S4 / S6 下 61 退 77 | 分辨（认判红） | 看得到：没有上游、游离都是 git 答得出的 | 第一分句；与 61 红样本 setup.sh:4 的说法冲突 | 「认」修得了这一格 |
| 「认」：S1 下 88 判上一轮的行 | 分辨（不认退 77） | 看得到 | 第一分句；与 88:19 原文冲突（第二分句） | 「不认」修得了这一格 |
| 认法甲漏进样本 | 分辨认法甲 / 乙 | 看得到：样本仓里 master 是它自己的 | 第一分句（t11-leak.out） | 认法乙量过修得了（只在我的模型上，被攻过零轮） |

两个候选各有一格打中、互不相修，第一轮写的二选一在这一格上就是「改法碰不到打中的格」那一种。本腿提的第三种（推的，没实现、没跑）：共用脚本保留「有上游就取上游 merge-base」，只把两处回退改掉——没有上游时先取 master / main 的 merge-base（功能分支的分叉点），在默认分支上且 merge-base 就是 HEAD 时用 gate-ok（是祖先才认），都没有才用 HEAD，并且在成功行里写明「没有上游，窗口只含工作区」。它在 S1 上与「不认」同（HEAD），在 S4 上与「认」同；S3 / S6 没有 gate-ok 时仍是 HEAD，61 仍退 77，只是说出来了——这一格它不修。

**推翻条件**：本仓的门禁只在配了上游的 master 上跑（没有功能分支、游离 HEAD、只在本地的仓这三种用法）——那样「不认」那一格没有对象；或者 88 号把 `git log --diff-filter=D` 换成也认工作区里已删的产物（那样「认」在 S1 下不红）。

## 我自己提的改法（全部只在我的模型上量过、被攻过零轮）

| 格 | 改法 | 修哪几格 | 量过 / 推的 |
|---|---|---|---|
| T1 | 75 号 ① 加反向：决策正文（历史版本之前）用 `E<号>（` 提到的实验，实验的影响表里要有这条决策一行 | B（真仓 D26→E10 那种） | 推的；真仓今天 25 对当场红，要先回填 |
| T1 | 75 号 ⑤：标题状态变了的那一批，表里每一行都要回看 | A | 推的 |
| T1 | 10 号剩下那段认「作废 / 退役」、认表里的「备料」行，或整段并进 75 号 ⑨ | C、D、E | 推的 |
| T5 | `gate_added_lines`：文件头状态机、剥尾部制表符、引号路径与带制表符的未跟踪路径退非 0、写死 `--no-renames`、不读未跟踪的二进制文件 | C1–C6 | **量过**（t5-fix.out）；C4 的代价见 T5 |
| T5 | 十处 `gate_changed_paths` 调用照 61:41 判退出码；64 号加一条判它 | C8、T1 最后一格 | 推的 |
| T5 | 64 号 ① 认 `@{u}`、`...`、`HEAD~N`，只认读 `GATE_BASE`、放行 `--is-ancestor`；② 查 quotepath 取值、包装函数按「这一行调它」认、多行调用拼成语句再判 | t5-64 的 11 种写法 | 推的 |
| T6 | 乙的红样本改成五行各一种错，`expect` 逐条 want | M1、M2、M3、M5、M7 | **量过**（t6-mutants.out 末段） |
| T11 | 认 `GATE_DIFF_BASE` 时只认 40 位十六进制且在本仓是提交 | 认法甲漏进样本 | **量过**（t11-leak.out） |
| T11 | 共用脚本两处回退改成：无上游先取 master / main 的分叉点，默认分支上 merge-base 是 HEAD 时认祖先 gate-ok，都没有才 HEAD 并在成功行写明 | S4、S5 的「不认」那一格 | 推的；S3 / S6 不修 |

## 没打中的形状

- **T1**：想找「⑤ 放行、而 75 号其余几条也都放行、丙b* 却放行」的反向格（戊比丙严）——只找到第一轮已知的一种：只回看了一行「不受影响：理由」、没动决策文件，75 绿、丙b* 红，那是 75 号给的合法出路，不算打中戊。想找 ⑤ 把标题行变动漏算的写法：标题从「部分已跑」改成「已跑」、只改括注、只改标题里的简称，按代码读都算「正文改了」（第 400–403 行按新增行在不在历史节与表之外判），没漏；这三种没跑。
- **T5 `gate_added_lines`**：没有结尾换行（最后一行照出，`\ No newline` 行以反斜杠开头不进）、CRLF（`\r` 两边一致，88 号 `strip()` 掉）、删掉的文件（`+++ /dev/null` 置空）、纯改名 100% 相似（没有 `+` 行，与 `git add` 前的未跟踪整份不一致，已归进 C4）、子模块（`+Subproject commit …` 会被当一行新增交出去，今天没有子模块，没跑）、十四种配置（C6，只有 `diff.renames` 有差）。
- **T5 40 号「三道都判完再退出」**：想找第一道或第二道红时第三道没跑到的输入，没找到；awk 失败当场退 1 是写明的。
- **T5 92 号**：见 T5 节，除中文 checker 路径外没找到新旧两版判得不同的输入。
- **T6**：M6（没写种类也放过）不是退化，后面的种类比对照样红；「按阶段位置调仓里的脚本」这一半两次抽样都没打中。
- **T11**：想找认法乙在真门禁里改变窗口的情形，只推出「仓里既没有 master 也没有 main」那一种（没跑）。

取样范围：T1 六批加三页，外加真仓 `.claude/kb/experiments/` 下全部 156 份、全部决策文件；T5 八种现场、十四种配置、11 种 64 号写法、真仓 gate.d 全部 `gate_changed_paths` 调用点；T6 九个变异、七种埋错；T11 八种仓状态、两种认法 × 四个值。模型是确定性的，每个跑了一遍；复跑命令在开头。

## 这条腿自己的限度

- **收工时的快照复核**（2026-09-23 16:11 UTC）：`sha256sum -c` 78 份里 1 份对不上：`.claude/rules/implementation-workflow.md: FAILED`。那是 T3 的文件，不归这条腿；我的模型没读它，报告里也没引它。
- 开跑前与收工时 `ps` 看到的：开工时一个别的会话在逐个跑 22、44、10、36 号与 doc-lint（pid 3395268）；收工时 `ask-local.sh research/prompts/gate-fix-forks-r2-local-attack.md` 在跑（本地攻方，pid 2439515）。没有 qemu、vm-bench、e152、fio，也没看到 cargo。模型全加了 `nice -n 19`。
- 「认」那一臂是拿 `GATE_BASE="$(diff_base)"` 跑真阶段模拟的，不是改了共用脚本再跑（t11-leak 那一段才是补丁后的副本）。两者在「GATE_BASE 不设」时取到同一个提交；GATE_BASE 被人指定时两者不同，那一格没跑。
- T5 的 HEAD 版阶段是 `git show HEAD:` 取出来放在草稿目录里跑的，它们的 `$(dirname "$0")` 指向草稿目录——这三份 HEAD 版不 source 任何东西，不受影响。
- C3 依赖门禁用的 `grep` 是 GNU grep 3.11（`bash --norc --noprofile -c 'readlink -f "$(command -v grep)"'` → `/usr/bin/grep`，`/usr/bin/grep --version` → `grep (GNU grep) 3.11`）；我交互 shell 里的 `grep` 是 ugrep 7.8.4，遇 NUL 同样一行都不交，但门禁不走它。
- T6 的现场是拷过去的真输入（layout、replay.sh、`e142-first-txn-dry-run-2026-09-23-decisions-catchup.out`、四份 .rs），`check-segment-registry.py` 在别的会话的未提交改动里，拷的是今天工作区那一份。
- T1 的真仓扫描按 75 号的 `D<号>（` 行首形态认影响表的行、按 10 号的正则认「提到」，与两道阶段的解析不是同一份代码；25 对里只手核了 D8→E28 与 D26→E10 两对。
- 副本上量出的数（t5-fix、t6 改法、t11-leak 认法乙）不算入库装置上的数。

## 没做什么

- 没改仓里任何文件；只写了这份报告与模型目录 `research/prompts/gate-fix-forks-r2-opus-model/`。现场都在 `/tmp/claude-1000/gate-fix-forks-r2-opus/` 下（`run/` 是 `.out` 那一次，`work/` 是写模型时的草稿，`t1`、`t5`、`t6`、`t11`、`patched` 等是调试时的现场），全是临时 git 仓与拷贝，没入库，靠复跑命令重建。
- 没跑 54、55、57、59、87 号，没碰 cargo；跑过的真阶段只有 10、40、56、61、64、75、88 号（在临时仓里）与 52 号（在拷过去的现场上），加上 `check-segment-registry.py`（真仓只读，含 `--selftest`）与 64 号（真仓只读，退 0）。
- T3、T4、T7、T8 与辩方的复核没碰。T6 模型里 52 号「找不到脚本退 1」那一支没走到，不评。
- 没读禁读清单里的文件；全仓 grep 都排除了 `research/prompts/`，要读的第一轮报告与判决按路径直接读。
- 没派 subagent，没跑本地模型。
