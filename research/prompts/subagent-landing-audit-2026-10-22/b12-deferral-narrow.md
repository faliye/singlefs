# 第十二批任务 47：deferred-items-registered 格收窄误判（2026-10-23）

## 结论

- 派发给的两类误判都收窄了：标题行不判；「延后项」当名词用、同一分句里带 N/M 统计数的「延后」不算延后（统计数这一条是真仓逐行判时补的：第 94 行后半句「延后当时就登进欠账表……的 6/54」只靠名词那一条收不掉）。另一种「去向」：表格行里延后类词那一格之后的某一格点名了登记处表 `.claude/gate.d/deferral-registries.tsv` 里登记的文件，就算登记了。表是新建的，两行：`staged-gate.md`、`commit-stages.md`，每行写了为什么算登记处。
- 真仓逐行判的时候发现收窄过了头，各修了一处：① 「进延后项」「记进延后项」是在把这一件延后，照旧算（不然里程碑四预开发:838 与里程碑三收尾:1289 这两条真延后会被放过）；② 登记处只认表格行、而且要在延后类词那一格之后（去向格），不然里程碑三收尾:1418 会被放过。那一行把 `commit-stages.md` 当成要改的对象提到，并不是去向。
- 新增四份样本：topic-green / topic-red / registry-green / registry-red；两份旧样本的 expect 各改 1 行，跟着改了措辞。新增八个弄坏开关。草稿里把每个开关逐个打开喂六份样本，每个开关都只让点名的那几份判错；拿任务 41 那一版判法喂两份新绿样本，两份都判红（5 行、2 行）。
- 真仓读数（同一条命令，records/ 这期间被别的会话写长了，新增行从 2531 行变成 2641 行）：任务 41 那一版判红 109 行，收窄后 96 行。转绿的 13 行逐行判过，都是该转的：话题名词 8 行、去向在登记处 3 行、标题 2 行。剩下 96 行逐行判过：**真延后没登记 89 行，还是误判 7 行**。这 7 行分三类：「留给」作轮内交接或保留给某人用、撤回延后（提前做）、转述流程或说清单在哪。三类都写不出一条不放过真延后的文字规则（理由见第五节），没再收窄，写进了格的文件头「管不到的」，交主 agent 改写这 7 行，或者另定判法。
- 什么现象会推翻这个结论：stage-selftest 喂样本时有一份判错（我的草稿只单跑了格里那段 python 和那段 shell 函数，没经阶段起）；stage-break-switch-replay 格对新开关实测的判错集合与头注点名的不等；真仓里有只靠「延后项」或统计数说法写的真延后被放过（第四节列了转绿的全部 13 行，可以逐行复核）。

## 改过的文件

| 文件 | 改了什么 |
|---|---|
| `.claude/gate.d/doc-process-records.sh` | 只动 deferred-items-registered：格名表那一行与 `stage_cell` 的描述（两处逐字相同，`--list` 打出新描述）、出路句、文件头那一段（对象、判据 ④、管不到的、开关表）、样本清单加四行、共用库 `STAGE_CELLS_BREAK=red-swallowed` / `=marker-ignored` 的点名清单补上新样本、常量 `DEFERRAL_REGISTRY_TABLE`、格里那段 python |
| `.claude/gate.d/deferral-registries.tsv`（新建） | 登记处表：两行加三行注释 |
| `.claude/gate.d/fixtures/doc-process-records.sh/deferred-items-registered-{topic-green,topic-red,registry-green,registry-red}/`（新建） | 每份三个文件：`setup.sh`、`expect`、`.gate-cells` |
| `.claude/gate.d/fixtures/doc-process-records.sh/deferred-items-registered-red/expect` | 第 5 行那条 want 改成新说法「…、文件:行或登记处」 |
| `.claude/gate.d/fixtures/doc-process-records.sh/deferred-items-registered-green/expect` | 成功句 want 改成新说法，句尾加上新的两个计数（各 0 行） |

`git diff --stat`（工作区对暂存区，这份阶段同时带着任务 41 还没暂存的改动）原样：
```
 .claude/gate.d/doc-process-records.sh | 352 +++++++++++++++++++++++++++++++++-
 1 file changed, 345 insertions(+), 7 deletions(-)
```
只算这一批自己的改动（对开工时留的底 `originals/`，`git diff --no-index --stat`）：
```
 .../gate.d/doc-process-records.sh                  | 159 ++++++++++++++++++---
 1 file changed, 136 insertions(+), 23 deletions(-)
```
两份旧 expect 各 `1 insertion(+), 1 deletion(-)`。新建的文件都没进 git（`??`），行数：表 5 行；setup.sh 39 / 35 / 37 / 43 行；expect 4 / 11 / 4 / 19 行。阶段的改动段（`diff originals 现在`）落在第 22 行、头注 259–306、样本清单 378–383、共用库清单 418 与 425–426、`stage_cell` 465–466、常量 504–505、格函数 1891–2134，别的格一行没动。`bash -n` 通过；从阶段里抽出来的 python 与草稿 `cell.py` 逐字节相同（`cmp`）。

没写新的 `# gate-similar:` / `# hook-events:` 行：没加新阶段，也没加新钩子。`gate-overlap.py --list` 里管延后项登记的只有 doc-process-records 这一道（`gate-overlap-list.txt` 第 36 行）。

## 一、判法改了什么（格里那段 python）

- 判哪些行：先去掉「」与反引号里的字，再去掉只在点话题的「延后」，剩下的字里还有延后类词的才判。只在点话题的「延后」有两种：名词「延后项」（正则 `(?<!进)延后项`，「进延后项」照旧算）；以及和 `N/M` 落在同一个分句里的「延后」（分句按 `，。；;：:|` 切，只管「延后」这一个词）。这两种去掉之后没词了的，计进「只在点话题的」。
- 标题行（`^ {0,3}#{1,6}(?:[ \t]|$)`）不判，计进「标题」；先判围栏，围栏里的标题计进围栏。
- 判据 ④ 登记处：只看表格行。从带延后类词的第一格往后，后面任何一格点名了表里登记的路径，就算登记了。路径可以写在反引号里，前后不能紧挨路径里的字，所以 `staged-gate.md.old` 不算。
- 登记处表逐行核四样：是不是两段、文件在不在树里、理由够不够 8 个字、有没有同一份登两次。写坏的行逐行报 `✗ <表>:<行>：…`，后面跟一条汇总的 ✗ 和 →，那一行不当登记处认，这一格判红。所有行都不判、而表写坏了的，退 1，不退 77。表不在就当一份登记处都没有。
- 措辞：「没写」那句改成「同一行没写 C 号、拆分提案的节号与行首序号、文件:行或登记处」；成功句改成「…、文件:行或登记处」，不判的计数后面加「「延后」只在点话题的（名词「延后项」、统计数）N 行、标题 N 行」；出路句后面加一句登记处的写法。

## 二、样本：红绿各两份新的（草稿里单跑格里那段 python，基准取 HEAD，与 stage-selftest 去掉 GATE_BASE 之后的取法一样）

单跑命令：`bash run-cell.sh <样本目录> <开关> [<格的 python>]`。它在一棵临时树里起 setup.sh，经 `scripts/gate/changed-paths.sh` 的 `gate_added_lines HEAD "records/*.md"` 取新增行，再起格的 python。另用 `run-function.sh` 起从阶段原样抽出来的那段 shell 函数（连同 `DEFERRAL_WORDS` 与 `DEFERRAL_REGISTRY_TABLE` 两个常量），六份样本的退出码与红行数都对：topic-green 0/0、topic-red 1/5、registry-green 0/0、registry-red 1/7、旧红 1/6、旧绿 0/0。

topic-green（rc=0）：
```
  ✓ records/ 下这次新增的带延后类词的行：判了 1 行，都写着指得到的登记（C 号、拆分提案的节号与序号、文件:行或登记处；records/ 新增的 .md 行共 9 行）；不判的：词只在「」或反引号里的 0 行、围栏里的 0 行、拆分提案（登记处本身）里的 0 行、「延后」只在点话题的（名词「延后项」、统计数）4 行、标题 1 行
rc=0
```
topic-red（rc=1）：
```
  ✗ records/2026-10-22-样本调度.md:4：带「延后」，同一行没写 C 号、拆分提案的节号与行首序号、文件:行或登记处
  ✗ records/2026-10-22-样本调度.md:5：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号、文件:行或登记处
  ✗ records/2026-10-22-样本调度.md:6：带「延后」，同一行没写 C 号、拆分提案的节号与行首序号、文件:行或登记处
  ✗ records/2026-10-22-样本调度.md:7：带「延后」，同一行没写 C 号、拆分提案的节号与行首序号、文件:行或登记处
  ✗ records/2026-10-22-样本调度.md:8：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号、文件:行或登记处
  ✗ records/ 下这次新增的带延后类词的行里 5 行没登记去向（判了 6 行）
     → 怎么办：照 .claude/main-agent.md「一轮怎么开、怎么收」第 2 条在同一行写登记的编号或 文件:行（…），先在那里登一行再写延后；去向是提交时那一套流程的，同一行点名 .claude/gate.d/deferral-registries.tsv 里登记的那份文件（写全路径）
rc=1
```
registry-green（rc=0）：
```
  ✓ records/ 下这次新增的带延后类词的行：判了 3 行，都写着指得到的登记（C 号、拆分提案的节号与序号、文件:行或登记处；records/ 新增的 .md 行共 3 行）
rc=0
```
registry-red（rc=1）：
```
  ✗ .claude/gate.d/deferral-registries.tsv:3：.claude/skills/gone.md 不在树里（写从仓根起的路径）
  ✗ .claude/gate.d/deferral-registries.tsv:4：.claude/skills/crash-test/commit-stages.md 的理由不到 8 个字（写为什么这份文件本身就是登记处）
  ✗ .claude/gate.d/deferral-registries.tsv:5：不是「路径<TAB>理由」两段（读出 1 段）
  ✗ .claude/gate.d/deferral-registries.tsv:6：.claude/skills/gate/staged-gate.md 登记了两次
  ✗ 登记处表 .claude/gate.d/deferral-registries.tsv 里 4 行写坏了，这几行不当登记处认
     → 怎么办：照 .claude/gate.d/deferral-registries.tsv 文件头写成「从仓根起的路径<TAB>为什么它本身就是登记处（8 个字以上）」，一份一行、文件要在树里；不再是登记处的整行删掉
  ✗ records/2026-10-22-样本调度.md:4：带「下轮开」「下一批」，同一行没写 C 号、拆分提案的节号与行首序号、文件:行或登记处
  ✗ records/2026-10-22-样本调度.md:5：带「下轮开」，同一行没写 C 号、拆分提案的节号与行首序号、文件:行或登记处
  ✗ records/2026-10-22-样本调度.md:6：带「下轮开」，同一行没写 C 号、拆分提案的节号与行首序号、文件:行或登记处
  ✗ records/2026-10-22-样本调度.md:7：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号、文件:行或登记处
  ✗ records/2026-10-22-样本调度.md:8：带「下轮开」，同一行没写 C 号、拆分提案的节号与行首序号、文件:行或登记处
  ✗ records/2026-10-22-样本调度.md:9：带「下轮开」，同一行没写 C 号、拆分提案的节号与行首序号、文件:行或登记处
  ✗ records/2026-10-22-样本调度.md:10：带「下轮开」，同一行没写 C 号、拆分提案的节号与行首序号、文件:行或登记处
  ✗ records/ 下这次新增的带延后类词的行里 7 行没登记去向（判了 8 行）
     → 怎么办：（同上）
rc=1
```
（registry-red 第 4–10 行依次是：「九十节各行」、没登记的 other-process.md、登记处只在延后类词那一格之前、不是表格行、`staged-gate.md.old`、commit-stages.md（它在表里那一行的理由太短）、gone.md（它在表里那一行写的文件不在树里）；第 11 行去向格点名 staged-gate.md，不报。）

拿任务 41 那一版判法（`cell-task41.py`，从阶段里原样抽出，与任务 41 草稿里那份逐字节相同）喂两份新的绿样本：topic-green 判红 5 行（「判了 6 行」），registry-green 判红 2 行（「判了 3 行」）。

## 三、弄坏开关（`DEFERRED_ITEMS_BREAK=<项>`，写在文件头那一段，一次设一个）

新加的八个。头三个关掉收窄、回到任务 41 那一版的判法；`registry-table-ignored` 同样是回到改前；另外四个把收窄放宽过头，用来证红样本里那几行确实有人在判：

| 开关 | 弄坏哪一处 | 判错的样本（头注点名的） |
|---|---|---|
| `headings-judged` | 标题行也照判 | topic-green |
| `deferral-noun-counted` | 名词「延后项」也算延后 | topic-green、topic-red |
| `statistics-counted` | 统计数分句里的「延后」也算 | topic-green |
| `into-noun-skipped` | 「进延后项」也当名词去掉 | topic-red |
| `noun-line-skipped` | 带「延后项」的整行都不判 | topic-red |
| `registry-table-ignored` | 不读登记处表 | registry-green、registry-red |
| `registry-cell-unchecked` | 登记处在行里任何位置都算 | registry-red |
| `registry-rows-unchecked` | 登记处表的行不核 | registry-red |

草稿里逐个打开，六份样本（两份旧的加四份新的）各喂一遍，只判格里那段 python 打出的 want（stage-cells 库打的三条跳过），一共 78 趟。`judge-switches.txt` 原样：
```
（不设开关）: 全判对
pointers-unresolved-accepted: old-red（rc=1，期望 1；缺 6 条 want：records/2026-10-22-样本调度.md:6：带「下一批」，写的登记指不到——C 号「C99」指不到：.cl）
quotes-counted: old-green（rc=1，期望 0；缺 1 条 want：records/ 下这次新增的带延后类词的行：判了 5 行，都写着指得到的登记（C 号、拆分提案的节号与序号、文件:行或）
registry-rows-judged: old-green（rc=1，期望 0；缺 1 条 want：records/ 下这次新增的带延后类词的行：判了 5 行，都写着指得到的登记（C 号、拆分提案的节号与序号、文件:行或）
fenced-lines-judged: old-green（rc=1，期望 0；缺 1 条 want：records/ 下这次新增的带延后类词的行：判了 5 行，都写着指得到的登记（C 号、拆分提案的节号与序号、文件:行或）
headings-judged: topic-green（rc=1，期望 0；缺 1 条 want：records/ 下这次新增的带延后类词的行：判了 1 行，都写着指得到的登记（C 号、拆分提案的节号与序号、文件:行或）
deferral-noun-counted: topic-green（rc=1，期望 0；缺 1 条 want：records/ 下这次新增的带延后类词的行：判了 1 行，都写着指得到的登记（C 号、拆分提案的节号与序号、文件:行或）；topic-red（rc=1，期望 1；缺 1 条 want：records/2026-10-22-样本调度.md:5：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号、文）
statistics-counted: topic-green（rc=1，期望 0；缺 1 条 want：records/ 下这次新增的带延后类词的行：判了 1 行，都写着指得到的登记（C 号、拆分提案的节号与序号、文件:行或）
into-noun-skipped: topic-red（rc=1，期望 1；缺 2 条 want：records/2026-10-22-样本调度.md:4：带「延后」，同一行没写 C 号、拆分提案的节号与行首序号、文件）
noun-line-skipped: topic-red（rc=1，期望 1；缺 2 条 want：records/2026-10-22-样本调度.md:5：带「下一批」，同一行没写 C 号、拆分提案的节号与行首序号、文）
registry-table-ignored: registry-green（rc=1，期望 0；缺 1 条 want：records/ 下这次新增的带延后类词的行：判了 3 行，都写着指得到的登记（C 号、拆分提案的节号与序号、文件:行或）；registry-red（rc=1，期望 1；缺 1 条 want：records/ 下这次新增的带延后类词的行里 7 行没登记去向（判了 8 行））
registry-cell-unchecked: registry-red（rc=1，期望 1；缺 3 条 want：records/2026-10-22-样本调度.md:6：带「下轮开」，同一行没写 C 号、拆分提案的节号与行首序号、文）
registry-rows-unchecked: registry-red（rc=1，期望 1；缺 10 条 want：.claude/gate.d/deferral-registries.tsv:3：.claude/skills/gone）
跑了 78 趟
```
四个旧开关判错的样本没变；新开关判错的样本，与上表点名的逐个相同。stage-break-switch-replay 格经 stage-selftest 复跑的那一遍没跑（样本没暂存，钩子拒），归第六节。

## 四、真仓读数（主工作区仓根，不经门禁阶段）

```
bash -c 'source scripts/gate/changed-paths.sh && gate_added_lines HEAD "records/*.md"' > real/added-head-final.tsv            # rc=0，2641 行
python3 -B cell-from-stage.py real/added-head-final.tsv "延后 下轮开 下一轮开 下一批 留给 下一个题目" "" .claude/gate.d/deferral-registries.tsv   # rc=1
python3 -B cell-task41.py     real/added-head-final.tsv "延后 下轮开 下一轮开 下一批 留给 下一个题目" ""                                         # rc=1
```
汇总行原样（收窄后 / 任务 41 那一版）：
```
  ✗ records/ 下这次新增的带延后类词的行里 96 行没登记去向（判了 153 行）
  ✗ records/ 下这次新增的带延后类词的行里 109 行没登记去向（判了 165 行）
```
开工时第一次取的新增行是 2531 行，那时任务 41 那一版判红 107 行，和任务 41 报告里的数一样。后来别的会话往 `records/2026-10-18-里程碑三收尾与近期欠账.md` 加了几行，多出两条红：第 1580 行与第 1594 行，两条都是真延后，见下面的列表。

### 收窄后转绿的 13 行（逐行判过，都是该转的）

| 行 | 凭哪一条转绿 | 判 |
|---|---|---|
| 复用键与全仓改名.md:979 | 名词「延后项」 | 讲的是把延后项合成一批派出去，本行没在延后什么 |
| 子agent结果落地审计.md:48 | 名词（另一个「延后」在「」里） | 九条缺口表里的一行，讲的是话题 |
| 子agent结果落地审计.md:87、104、115 | 名词 | 主 agent 点名的第一类 |
| 子agent结果落地审计.md:94 | 名词加统计数 | 主 agent 点名的第一类（后半句「延后当时就登进……的 6/54」靠统计数那一条） |
| 子agent结果落地审计.md:127 | 名词 | 「延后项」是缺口的名字。这一行另有「进欠账」，不在词表里，原来那一版也只是碰巧因「延后项」判红 |
| 子agent结果落地审计.md:140 | 名词、标题 | 主 agent 点名的第一类（`### 延后项的登记`） |
| 子agent结果落地审计.md:170 | 名词加统计数 | 「延后项 8/54」是统计数。这一行另有「要等第 18 号任务加 `--until`」，不在词表里 |
| 子agent结果落地审计.md:212、264、323 | 登记处（去向格点名 staged-gate.md） | 主 agent 点名的第二类 |
| 门禁自证覆盖格修正.md:23 | 标题 | `## 延后与不归这一轮的`，延后的事写在节里的行上 |

收窄过头、已经修掉的两处（中间那一版判法把它们放过了，最终这一版照旧判红）：里程碑四预开发:838「记进延后项」、里程碑三收尾:1289「进延后项：」靠 `(?<!进)`；里程碑三收尾:1418 靠「登记处只认表格行的去向格」。

### 剩下的 96 行，按文件逐行判（文件:行、带哪几个词、判定、原行前 120 字）

#### records/2026-10-13-代码欠账批.md（2 行）

- `records/2026-10-13-代码欠账批.md:175` 「留给」：还是误判：「留给」作交给这一轮已排定的下一步（同一轮的攻方腿与判决），不是延后
  原行前 120 字：- 2026-10-17 m3-b44-code-r2 正推腿交回（`m3-b44-code-r2-sonnet-output.md` a4b74279…，373 行；腿报 cite-check 64 处对上 64、裸行号 192 处没判）
- `records/2026-10-13-代码欠账批.md:217` 「留给」：真延后，没登记
  原行前 120 字：- 2026-10-18 E179 跑前登记修订一交回（`e179-preregistration.md` 改为 696 行、c29bef1b…，修订一在第十二节；报告 `~/.local/share/singlefs/work/e179-

#### records/2026-10-13-复用键与全仓改名.md（1 行）

- `records/2026-10-13-复用键与全仓改名.md:989` 「下一批」：真延后，没登记
  原行前 120 字：批 1、批 2 结果（2026-10-18）：两条腿交回（`dyn-share-r1-sonnet-output.md` 300 行、`dyn-share-r1-opus-output.md` 295 行与模型目录 `dyn-share-r

#### records/2026-10-13-里程碑四预开发.md（26 行）

- `records/2026-10-13-里程碑四预开发.md:227` 「下一批」：真延后，没登记（写了「记进拆分提案」，没写节号与行首序号）
  原行前 120 字：- 合入验证会话 报：它那一趟的清场失败两次，一是本机卡被 `singlefs-crates-mutation-target-w2` 里的 `crash_judge_gpu_agrees…` 测试二进制（pid 305290）占着——是这边
- `records/2026-10-13-里程碑四预开发.md:277` 「留给」：真延后，没登记（去向是提交时的整表复跑，没点名登记处文件）
  原行前 120 字：- **写段写完**（照 `.claude/main-agent.md`「写与验分两段」三条）：① 依赖表这一段的条款（修复批一 F1–F10、二、三 H1–H5、四 J1–J6、五）都有实现，没有「下一批再补」的行；② 每条新行为都有测试
- `records/2026-10-13-里程碑四预开发.md:337` 「下轮开」：真延后，没登记
  原行前 120 字：| 截断#9 截庚、截辛、截癸 | 下轮开 | 等 合入验证会话 全量跑完的通知；派发提示先写好（`~/.local/share/singlefs/work/m4-trunc-batch9-prompts/`） |
- `records/2026-10-13-里程碑四预开发.md:706` 「下一批」：真延后，没登记（交别的会话的下一批；文件:行只写了文件名）
  原行前 120 字：- 里程碑三收尾会话 报（它报的，本轮没核）：修红那份在它副本里库单测（gpu）60 过；每状态对拍 243 条里 242 过、1 红，上次红的 15 条都转绿。红的那条是 `the_gpu_judge_agrees_with_the_cp
- `records/2026-10-13-里程碑四预开发.md:747` 「下一批」：真延后，没登记（交别的会话的下一批；文件:行只写了文件名）
  原行前 120 字：  - new-pool-file-creation：`crash_point_replay_quick_tier_matches_the_full_tally_shape` 断言「I-9.40 评估过的状态数」左 9 右 46（`cras
- `records/2026-10-13-里程碑四预开发.md:751` 「留给」：还是误判：「留给」作留给在跑的调查员交回时对照的线索，不是延后
  原行前 120 字：  - 里程碑三收尾会话 回（2026-10-20）：录入键它没动；`crash_amplification.rs` 已打进暂存区的只有批 5 GPU 那份补丁的 2 行（`SCAN_REBUILD_OUTCOME_NOT_REACHED`
- `records/2026-10-13-里程碑四预开发.md:838` 「延后」：真延后，没登记
  原行前 120 字：- 写段 `apply-writer-patch.py` 报的 mutation-tables 红查清：`crates/mutations.tsv` 第 4958–4965 行 8 条「E173 第一段 MR1–MR8」点名的 `crate
- `records/2026-10-13-里程碑四预开发.md:861` 「下一批」：真延后，没登记
  原行前 120 字：- 用户：「rocksdb 和 snappy 很重要 尽量优化复用和性能」。补给在跑的工具实现员（用户原话的要求，不等下一次派发）：两个工作区、各构建档与覆盖率目录链同一份预编库；照 release 级别优化编；给编一次、链接与判定库用例改
- `records/2026-10-13-里程碑四预开发.md:864` 「下一批」：真延后，没登记（这一行另有「下一批交回之后删」（副本清理延后）；前一处「派写段下一批实现员」是轮内续派）
  原行前 120 字：- m4-i315-r3 定案实现员交回（opus，1h27m；报告 `~/.local/share/singlefs/work/m4-i315-r3-writer/report.md`，sha256 40a9718e…；补丁 `patch
- `records/2026-10-13-里程碑四预开发.md:891` 「下一批」：真延后，没登记
  原行前 120 字：- 2026-10-22 弹窗，用户定：E187 岔路「落点甲，checker 只认记号 (Recommended)」；代码轮 T5 甲「丑：写者接得住这一形 (Recommended)」。两样都改格式或产品代码，并进下一批实现，连带 D1
- `records/2026-10-13-里程碑四预开发.md:896` 「下一批」：真延后，没登记（交别的会话的下一批）
  原行前 120 字：- 3c 提议 r3 的崩溃轻型用 `--item` 跳过 mkfs 那一项；主 agent 现查 `scripts/crash/crash-point-replay.sh` 文件头第 18 行「--quick [--item …] 轻型：
- `records/2026-10-13-里程碑四预开发.md:914` 「下一批」：真延后，没登记（第八十七节没写行首序号）
  原行前 120 字：- admission 自证红的来源：3c、b2、8c 都说不是自己的；8c 指出草稿 `work/round2-two-logs` 对得上「日志出两份」那批，登记在 `records/2026-10-21-doc门禁违规修复.md`，那个
- `records/2026-10-13-里程碑四预开发.md:917` 「下一批」：真延后，没登记
  原行前 120 字：- 它停下的五条，主 agent 判：① 区域扫描不认记号（重挂后带记号的记录归 Dead、只报告不搬）——安全一侧，记欠账；② 回退遇一块盘带记号照清照做——那种盘 I-3.17 第三句本就判红，记欠账；③ 内存里「已释放又带记号」写得出
- `records/2026-10-13-里程碑四预开发.md:932` 「留给」：真延后，没登记（去向是验证段的变异轻型（不在登记处表里））
  原行前 120 字：- 2026-10-22 执行器判据与 clippy 遮蔽实现员交回（报告 `~/.local/share/singlefs/work/m4-validate-executor-mark-and-shadows-2/report.md`，s
- `records/2026-10-13-里程碑四预开发.md:939` 「留给」：真延后，没登记（去向是验证段的变异轻型（不在登记处表里））
  原行前 120 字：- 2026-10-22 派实现员修本会话的 4 行变异（草稿 `~/.local/share/singlefs/work/m4-validate-mutation-rows-repair/`）：105 替换文补隔离记号字段；4234、50
- `records/2026-10-13-里程碑四预开发.md:940` 「留给」：真延后，没登记（去向是验证段的变异轻型（不在登记处表里））
  原行前 120 字：- 2026-10-22 变异行修法实现员交回（报告 `~/.local/share/singlefs/work/m4-validate-mutation-rows-repair/result.md`，sha256 4ac83a11…）：1
- `records/2026-10-13-里程碑四预开发.md:957` 「下一批」：还是误判：转述用户定的流程（「红了进写段下一批」），不是延后某一件
  原行前 120 字：- 2026-10-23 25 协调：用户在它那边定了合入后验证要加一步（主 agent 带 SINGLEFS_HEAVY_TESTS=user-request 只跑单元键对不上的 checker 档快档测试，红了进写段下一批），要改 sc
- `records/2026-10-13-里程碑四预开发.md:960` 「下一批」：真延后，没登记
  原行前 120 字：- 2026-10-23 用户要求「用变量拼拷贝目标」这个问题一定要解决、用脚本解决：派工具实现员 m4-variable-copy-target-script，新建备份原文件的脚本，Bash 检出钩子 ⑫ 对有写范围的子 agent 拒绝
- `records/2026-10-13-里程碑四预开发.md:962` 「下一批」：真延后，没登记（写了「拆分提案九十二节末尾」，没写「第」与行首序号）
  原行前 120 字：- 2026-10-23 ④（测试删自己的临时目录）交回：补丁 `~/.local/share/singlefs/work/m4-tests-delete-temp-dirs/patch/`（crates.patch sha256 1b00
- `records/2026-10-13-里程碑四预开发.md:967` 「下一批」：真延后，没登记（去向写的是两次派发名，不是登记）
  原行前 120 字：- 2026-10-23 ②（每趟自己的编译目录）交回（报告 `~/.local/share/singlefs/work/m4-per-run-build-dirs/result.md`，sha256 6172f5b9…）：七份入口加 pe
- `records/2026-10-13-里程碑四预开发.md:968` 「下一批」：真延后，没登记
  原行前 120 字：- 2026-10-23 用户弹窗定：变异门禁「这一趟现编一份外部包、拷给这一趟的工作进程、跑完全删」可以（趟内拷不算共用），不拆。ee 报 `scripts/experiment-apparatus/replay.sh` 还往固定的 `s
- `records/2026-10-13-里程碑四预开发.md:971` 「下一批」：真延后，没登记
  原行前 120 字：- 2026-10-23 清理钩子归属那件交回（报告 `~/.local/share/singlefs/work/m4-cleanup-hook-attribution/result.md`，sha256 21d3c306…）：运行目录只认
- `records/2026-10-13-里程碑四预开发.md:972` 「下一批」：真延后，没登记
  原行前 120 字：- 2026-10-23 临时目录与裸 cargo 那件交回（报告 `~/.local/share/singlefs/work/m4-temporary-files-and-bare-cargo/result.md`，sha256 c3d3
- `records/2026-10-13-里程碑四预开发.md:978` 「下一批」：真延后，没登记
  原行前 120 字：- 2026-10-23 主 agent 在主工作区复跑：crash-amplification.sh --selftest 退 0（开关轮与文件头相同）；crash-point-replay.sh --selftest 退 0（41 个开
- `records/2026-10-13-里程碑四预开发.md:986` 「下一批」：真延后，没登记
  原行前 120 字：- 2026-10-23 m4-break-switch-outside-selftest 交回（报告 sha256 f1d48c01…）：copy-originals.py、test-environment-check.py、broad-
- `records/2026-10-13-里程碑四预开发.md:993` 「下一批」：还是误判：把记下的下一批提前到这一轮做（撤回延后）
  原行前 120 字：- 2026-10-23 用户：「脚本 还是提前休比较好」（读作「提前修」）。把记下的下一批脚本小修提前到验证段里同时做，派两个 tooling-writer（opus，批次 #脚本小修）：① `m4-memory-cap-guard-an

#### records/2026-10-18-里程碑三收尾与近期欠账.md（42 行）

- `records/2026-10-18-里程碑三收尾与近期欠账.md:179` 「下一批」：真延后，没登记
  原行前 120 字：- 调查员交回 E181 f9 与 narrow（报告 `~/.local/share/singlefs/work/m3-close-w4-investigate-e181-f9/report.md`，sha256 c370599d…）：f
- `records/2026-10-18-里程碑三收尾与近期欠账.md:226` 「下一批」：真延后，没登记
  原行前 120 字：- 调查员交回 shard-run 真跑那一格（报告 `~/.local/share/singlefs/work/m3-close-w4-investigate-shard-run-real/report.md`，sha256 8848e6
- `records/2026-10-18-里程碑三收尾与近期欠账.md:696` 「下一批」：真延后，没登记
  原行前 120 字：  它报的四样进下一批：
- `records/2026-10-18-里程碑三收尾与近期欠账.md:708` 「下一批」：真延后，没登记
  原行前 120 字：  - 行名里「（io_min 8192 的池建不出表）」已不是今天的现象，进下一批，删行再加行改名。
- `records/2026-10-18-里程碑三收尾与近期欠账.md:722` 「下一批」：真延后，没登记
  原行前 120 字：  双机驱动自证与四道门禁的样本档在后台跑。三态数那 4 个进下一批，照闭式算再与跑出的数核，或者删掉。
- `records/2026-10-18-里程碑三收尾与近期欠账.md:742` 「下一批」：真延后，没登记
  原行前 120 字：    这 4 处进下一批工具员，只补文件头名单。
- `records/2026-10-18-里程碑三收尾与近期欠账.md:758` 「下一批」：还是误判：只说下一批清单写在这一节各条里，本行不延后什么
  原行前 120 字：  - 下一批清单写在这一节各条里。
- `records/2026-10-18-里程碑三收尾与近期欠账.md:771` 「下一批」：真延后，没登记
  原行前 120 字：- 工具批补丁只打了一部分（`git apply --include='scripts/*' --include='.claude/gate.d/stage-inputs.tsv'`）：quote-kb、看门狗门槛、双机驱动拷库拿锁、注释旧
- `records/2026-10-18-里程碑三收尾与近期欠账.md:774` 「下一批」：真延后，没登记（文件:行只写了文件名）
  原行前 120 字：- 里程碑四预开发会话 转来一条红，归这一会话：checker 档库单测 `crash_injection::tests::a_remount_refused_on_a_second_crash_image_where_device_zer
- `records/2026-10-18-里程碑三收尾与近期欠账.md:780` 「下一批」：真延后，没登记
  原行前 120 字：- 看门狗报第二台那一趟「在用 5.4 核（登记 31）……同时在算的线程约 1.0 条」，没有声明串行段。主 agent 现查第二台：`cargo test --offline --no-fail-fast -p singlefs-cor
- `records/2026-10-18-里程碑三收尾与近期欠账.md:842` 「下一批」：真延后，没登记
  原行前 120 字：  2. `crash_state_enumeration.rs` 的取样多线程每领一片照样现建一份表：不在这一批，轻型里撞了再并进下一批，没撞就记进下一批欠账。
- `records/2026-10-18-里程碑三收尾与近期欠账.md:843` 「下一批」：真延后，没登记
  原行前 120 字：  3. `judging_units_entries.rs` 文件头「表里的文字不进键」对 `observer_conclusion_composition` 不成立：进下一批欠账。
- `records/2026-10-18-里程碑三收尾与近期欠账.md:903` 「下一批」：真延后，没登记
  原行前 120 字：  它顺带看到的两处进下一批工具：门禁 checker-tier-qemu-device-streams 与它的样本、kb 里仍写 `14+1+1+1+4`；doc-registries 的 `STAGE_CELLS_BREAK=check
- `records/2026-10-18-里程碑三收尾与近期欠账.md:910` 「下一批」：真延后，没登记
  原行前 120 字：  - Q4 `transaction.rs` 两个名字与 GPU 失败点码常量的改名：下一批，碰 `transaction.rs` 前先告诉 GPU 垃圾回收会话。
- `records/2026-10-18-里程碑三收尾与近期欠账.md:911` 「下一批」：真延后，没登记（等别的会话下一批合入）
  原行前 120 字：  - Q5 GPU 恢复内核不判 deadlist 条目宽：与 deadlist I-1.10、`walk.rs` 那一批同批（等 里程碑四预开发会话 下一批合入）。
- `records/2026-10-18-里程碑三收尾与近期欠账.md:958` 「下一批」：真延后，没登记
  原行前 120 字：- 里程碑四预开发会话 指出崩溃轻型每一趟跑全部项、`--item` 只核项名，mkfs 用过的盘那一条撞内存上限挡着所有会话的崩溃轻型；派了调查员（`~/.local/share/singlefs/work/m3-close-b16-mk
- `records/2026-10-18-里程碑三收尾与近期欠账.md:978` 「下一批」：真延后，没登记
  原行前 120 字：  - 不派实现员。留两样进下一批清单：CPU 判段那份共享表还没有自己的单测与变异行（调查员指的）；合入后验证的崩溃轻型里看 mkfs 这一项的峰值，16G 下跑得过就不另量，报告 7.1 的重型量法不跑。
- `records/2026-10-18-里程碑三收尾与近期欠账.md:998` 「下一批」：真延后，没登记
  原行前 120 字：- 下一批（批 16 实现，G4 那一簇）：
- `records/2026-10-18-里程碑三收尾与近期欠账.md:1024` 「下一批」：真延后，没登记
  原行前 120 字：  - F6（GPU 判退取并集）：丙不取。它要改「配了 GPU 就一定要 GPU 判」，用户的现行要求是做不到就报错、不降级。先走乙：GPU 一簇的实现员在 checker 档写一条用例，在 GPU 判的崩溃态域上逐态比 CPU 三样读者
- `records/2026-10-18-里程碑三收尾与近期欠账.md:1056` 「留给」：还是误判：「留给」作把副本留给主 agent 跑轻型（交接物），不是延后
  原行前 120 字：  - 整工作区 `cargo build --all-targets` 在实现员那里四趟超时（负载 106），归验证段。它的副本 `~/.local/share/singlefs/work/m3-close-b16-gpu-recover
- `records/2026-10-18-里程碑三收尾与近期欠账.md:1069` 「留给」：真延后，没登记（去向是提交时的门禁，没点名登记处文件）
  原行前 120 字：  - 第 4–7 条做完。字段改名 `devices_with_fewer_than_two_self_verified_slots`。第 1 条只挂了四条流。第 2 条停下交主 agent。第 3 条的六行变异没证红，checker 档
- `records/2026-10-18-里程碑三收尾与近期欠账.md:1093` 「下一批」：真延后，没登记
  原行前 120 字：  - 同一个漏法第二次出现：实现员交回前、`apply-writer-patch.py` 打补丁时都不跑这条核法，新测试目标进了核心变异行而轻型登记表没跟上。记成工具欠账，下一批工具改动并进 `apply-writer-patch.py`
- `records/2026-10-18-里程碑三收尾与近期欠账.md:1192` 「下一批」：真延后，没登记
  原行前 120 字：  - 两个词的译名统一，并进下一批译文活：选一套译名，改那两组，再落术语表。
- `records/2026-10-18-里程碑三收尾与近期欠账.md:1193` 「下一批」：真延后，没登记
  原行前 120 字：  - 日语布局表本来就缺「合计 496 字节」与记录头偏移两段，原位置还留着两行写 487 的旧 format-const 标记，同进下一批译文活补。
- `records/2026-10-18-里程碑三收尾与近期欠账.md:1202` 「下一批」：真延后，没登记
  原行前 120 字：- 戊簇停下的四问，并进下一批 kb 文本活，主 agent 的定法：
- `records/2026-10-18-里程碑三收尾与近期欠账.md:1216` 「下一批」：真延后，没登记
  原行前 120 字：    - 这几处 kb 文本并进下一批文本活。
- `records/2026-10-18-里程碑三收尾与近期欠账.md:1237` 「下一批」：真延后，没登记
  原行前 120 字：  - 里程碑收口检查对真仓：里程碑三段表一写「结束」就红 249 行。判据按前置格任何点名都算，比「去向里程碑三」的 100 行宽；收窄成只认「去向：」之后，并进下一批工具。
- `records/2026-10-18-里程碑三收尾与近期欠账.md:1283` 「下一批」：真延后，没登记
  原行前 120 字：  - 派发闸 `.claude/hooks/runner-dispatch-guard.sh` 改了一处（这一轮的出口被它挡着，直接改）：TaskStop 的结果是「No task found with ID: <id>. Running
- `records/2026-10-18-里程碑三收尾与近期欠账.md:1289` 「延后」：真延后，没登记
  原行前 120 字：  - 另报三条，进延后项：
- `records/2026-10-18-里程碑三收尾与近期欠账.md:1300` 「留给」：真延后，没登记（去向是提交前整理（不在登记处表里））
  原行前 120 字：  - 留给提交前整理：三份变更史 D8 节 2026-10-22 块里两处与标题同日的日期，照变更史整理办、跑 `history-consolidate.py loss-check`。
- `records/2026-10-18-里程碑三收尾与近期欠账.md:1319` 「下一批」：真延后，没登记
  原行前 120 字：  - 它报的旁事：`sweep-term.py --check` 主工作区退 2，`.claude/term-rename-exempt` 三条按行豁免指向不存在的 `briefs/2026-09-1x/2x/10.zh.md`；swee
- `records/2026-10-18-里程碑三收尾与近期欠账.md:1326` 「延后」「下一批」：真延后，没登记
  原行前 120 字：  - 它报的岔路单之外的观测进延后项：可写挂载读阶段经 `ReadStageCache`（`mount.rs` 那一段）扫环，读到的非 0 记录字节整份留在内存里，也随盘数涨。立欠账行，并进下一批写回。
- `records/2026-10-18-里程碑三收尾与近期欠账.md:1338` 「下一批」：真延后，没登记
  原行前 120 字：- 乙簇 GPU 一半交回（报告 `~/.local/share/singlefs/work/m3-close-b16-cluster-yi-gpu/result.md`，cdcd3580…；`crates.patch` a4b44955…
- `records/2026-10-18-里程碑三收尾与近期欠账.md:1353` 「下一批」：真延后，没登记
  原行前 120 字：  - 自证基线 119 格 22 格红，都是 `reuse-*` 一族：判法单元指纹对三个入口拒算（`scripts/crash/crash-amplification-lib.sh`、`scripts/hosts/host-resour
- `records/2026-10-18-里程碑三收尾与近期欠账.md:1368` 「下一批」：真延后，没登记（条件跟改：别的会话下一批合入时）
  原行前 120 字：- 隔离记号那个会话回：`history.rs` 它此刻不改；它的实现员要把变异表第 4234 行（「m4 整理对拍：目的地写诞生代也判分配代不对」）的锚点改到 `history.rs` 的 `records_a_relocation_ba
- `records/2026-10-18-里程碑三收尾与近期欠账.md:1376` 「下一批」：真延后，没登记
  原行前 120 字：- 空出的一位派己六推上游的两份脚本（`~/.local/share/singlefs/work/m3-close-b16-tools-ji6/`）：待优化 51 gate-lint 只认行首的 ✗ / ✓、待优化 44 check.sh 
- `records/2026-10-18-里程碑三收尾与近期欠账.md:1418` 「下一批」：真延后，没登记（commit-stages.md 是被改的对象、不在表格去向格，收窄没放过它）
  原行前 120 字：  - 没改成：`.claude/skills/crash-test/commit-stages.md` 第 2b 步三行（不在工具员写范围，补丁 `patch/commit-stages.patch` d13a9e39…），并进下一批治理
- `records/2026-10-18-里程碑三收尾与近期欠账.md:1486` 「下一批」：真延后，没登记
  原行前 120 字：- 变异轻型撞顶 15 行重跑（每条 12G，外层 36G，日志 `mutation-oom.log`）退 1，计数「抓到 277 条、没红 0 条、无效 1 条、内存撞顶 0 条…（共 278 条）」。这一趟 ✓ 277 ✗ 1 本次未跑
- `records/2026-10-18-里程碑三收尾与近期欠账.md:1525` 「下一批」：真延后，没登记
  原行前 120 字：- e165 r7 续做执行员交回（`~/.local/share/singlefs/work/m3-close-b16-e165-r7-run2/result.md`，6651eefe…）：L1 产物那一半与 L2 够判不翻（P2、P3 
- `records/2026-10-18-里程碑三收尾与近期欠账.md:1546` 「下一批」：真延后，没登记
  原行前 120 字：  - 钩子（`~/.local/share/singlefs/work/m3-close-b17-heavy-guard-replay/`，`patch/hooks.patch` 96b18243…）：ee、f6 都同意后经 `hook-
- `records/2026-10-18-里程碑三收尾与近期欠账.md:1580` 「下一批」：真延后，没登记
  原行前 120 字：  - 另记：这份测试第 187 行读 `SINGLEFS_CRASH_POINT_REPLAY_THREADS`，与 `.claude/rules/verification.md` 写的 `SINGLEFS_CRASH_JUDGE_CPU
- `records/2026-10-18-里程碑三收尾与近期欠账.md:1594` 「下一批」：真延后，没登记（写了「登进收拢表」，收拢表不是认得的登记写法）
  原行前 120 字：- 派发类型的教训：kb 译文派 clerk 写不进 `.claude/kb/`（两次都被拒，靠交规格由主 agent 打），下一批治理文件里核 `.claude/main-agent.md` 派发表与 `.claude/skills/tr

#### records/2026-10-19-里程碑四GPU垃圾回收开工.md（11 行）

- `records/2026-10-19-里程碑四GPU垃圾回收开工.md:158` 「下一批」：真延后，没登记
  原行前 120 字：- 看门狗检出（处置）：攻方腿在自己拷贝上跑 `publish_lookup_by_identity` 的测试二进制（进程 734504），最近 722 秒平均只用 0.9 核。慢在用例自己：增长慢档用例在一个线程里依次量 512 与 20
- `records/2026-10-19-里程碑四GPU垃圾回收开工.md:159` 「下一批」：真延后，没登记
  原行前 120 字：- 七·6 代码轮第二轮两条腿交回，判决 `research/prompts/m4-publish-lookup-code-r2-main-verification.md`：T1 成立（另坐实一处七·6 五处之外的线性查找，`transac
- `records/2026-10-19-里程碑四GPU垃圾回收开工.md:160` 「下一批」：真延后，没登记
  原行前 120 字：- 用户定（弹窗，选「并进下一批」）：代码轮第二轮另坐实的线性查找并进七·6 下一批——`relocation_checked_against_the_version` 与 `mapping_locations_of_a_mapped_un
- `records/2026-10-19-里程碑四GPU垃圾回收开工.md:185` 「下一批」：真延后，没登记
  原行前 120 字：- 七·6 后续批合入后验证（第二台，日志 `~/.local/share/singlefs/work/merge-and-verify/2026-10-22-660698`）：fmt、clippy、build 过，测试红 1 条 `the
- `records/2026-10-19-里程碑四GPU垃圾回收开工.md:200` 「下一批」：真延后，没登记
  原行前 120 字：- 看门狗检出（处置）：本机崩溃轻型里 `crash_injection_campaign`（`--features verdict-store,gpu`、`--test-threads 1`，进程 3970279 那一支）最近 750 秒
- `records/2026-10-19-里程碑四GPU垃圾回收开工.md:273` 「下一批」：真延后，没登记（写了「工具欠账八十六」，不是认得的节号写法）
  原行前 120 字：- 看门狗报第 11 批崩溃轻型 3257846（`crash_injection_read_faulted_states_never_hold_the_witness_criterion` 双机驱动）在用 2.4 核、没声明串行：工具欠账
- `records/2026-10-19-里程碑四GPU垃圾回收开工.md:276` 「下一批」：真延后，没登记（交别的会话的下一批）
  原行前 120 字：- 里程碑三收尾会话回复认领这条 I-8.10 红：并进它下一批 crates，先判末条标志不唯一的组上新判法该不该让路，再改用例期望或判法，改完来告。
- `records/2026-10-19-里程碑四GPU垃圾回收开工.md:295` 「下一批」：真延后，没登记
  原行前 120 字：- 用户弹窗定：T3 两处「并进下一批」、T4 ①「并进下一批」。下一批（第 17 批）实现员关判决第三节 1–7 条加 T3 ①②、T4 ①。
- `records/2026-10-19-里程碑四GPU垃圾回收开工.md:304` 「下一批」：真延后，没登记
  原行前 120 字：- 预开发会话回分工：脚本全归它（实现员①双机驱动、②起 cargo 的脚本，名单外的收进它下一批），规则与 agent 定义归它的实现员③（含 `test-script-discipline.md` 那一句、`main-agent.md`
- `records/2026-10-19-里程碑四GPU垃圾回收开工.md:323` 「下一批」：真延后，没登记（交别的会话的下一批）
  原行前 120 字：- 预开发会话回：变异门禁外部包那一条用户在它那边弹窗选「趟内拷可以」——这一趟开头现编一份外部包、拷给这一趟的几个工作进程、跑完连同工作进程编译目录一起删，不跨趟；跨趟每台预编的那一种已经没有了，与本会话用户定的「算共用，不做」（问的是跨
- `records/2026-10-19-里程碑四GPU垃圾回收开工.md:347` 「下一批」：真延后，没登记
  原行前 120 字：- 看门狗报快档两格补跑（3775658）最近 758 秒平均 6.0 核、没声明串行：现查门禁 checker-tier-crash-point-replay 那一趟是一条 `cargo test --release --no-fail-

#### records/2026-10-21-doc门禁违规修复.md（10 行）

- `records/2026-10-21-doc门禁违规修复.md:65` 「下一批」：真延后，没登记
  原行前 120 字：- D 包主 agent 自己改（没人认领的几份，replace-once 定点）：两处额度句只留「中途停下没交回」（`records/2026-10-18-审查报告核实与修复.md`、`records/2026-10-19-文档普查报告核
- `records/2026-10-21-doc门禁违规修复.md:99` 「下一批」：真延后，没登记
  原行前 120 字：- archive-past-rounds：照 `GATE_BASE=1bb3a24dc` `--apply --keep-registered` 删了 471 份（里程碑二、三与更早实验的记录，`m4-` 起头的 0 份），22 份文件里
- `records/2026-10-21-doc门禁违规修复.md:148` 「下一批」：真延后，没登记
  原行前 120 字：- 批次 3 不归这一轮、照原样留着的红：译文（item-ref-status、settled-ref-says-open、cross-decision-status、kb-shape 的译文标题；admission-terms 与 tre
- `records/2026-10-21-doc门禁违规修复.md:160` 「下轮开」：真延后，没登记
  原行前 120 字：| archive-past-rounds | 本轮关（471 份）；脚本三处毛病下轮开 | 主 agent |
- `records/2026-10-21-doc门禁违规修复.md:166` 「下轮开」：真延后，没登记（第八十一节没写行首序号）
  原行前 120 字：| 门禁与脚本的问题：results-cited 不认 `.out.gz`；`lib-governance-refs.py` 的 `README_PATTERNS` 还认 `README.zh.md`；archive-past-rounds
- `records/2026-10-21-doc门禁违规修复.md:170` 「下一批」：真延后，没登记（交别的会话的下一批）
  原行前 120 字：- 里程碑四预开发会话回：m4-i315-r3 判决写完（`research/prompts/m4-i315-r3-main-verification.md`，用户已定案），快照不再挡；它下一批照判决补 05-空间记账.md 里 I-3.1
- `records/2026-10-21-doc门禁违规修复.md:172` 「下轮开」：真延后，没登记（第八十一节没写行首序号）
  原行前 120 字：- D22 已定项 8 的进度照用户定交里程碑三收尾会话，它回「并进我这边 kb 写回批：写回时现查 HEAD 加 mkfs 那批的实现判翻不翻」。这两轮撞上的治理缺口七处记进 `records/2026-09-16-subagent拆分提
- `records/2026-10-21-doc门禁违规修复.md:203` 「下轮开」：真延后，没登记
  原行前 120 字：| `baseline-diff.sh` 把带计数的汇总行当新红；检出器对「目标在变量里、而静态前缀已在写范围内」的写也记检出（回放 52 条，看门狗每条都被叫醒） | 下轮开，交 tooling-writer | — |
- `records/2026-10-21-doc门禁违规修复.md:204` 「下轮开」：真延后，没登记
  原行前 120 字：| `runner-dispatch-guard.sh` 里第三份读写范围表的代码没并进 `lib_write_scope.py`；检出器 ⑧ 对 heredoc 里的文字也当真写判；`cited-paths-exist` 格不认 `.ou
- `records/2026-10-21-doc门禁违规修复.md:207` 「下轮开」：还是误判：「下轮开的几处这一轮全做」（撤回延后）
  原行前 120 字：- 用户「你最好一次做完 不要给我说下一轮了」：收拢表里标下轮开的几处这一轮全做，截止批次改成 3。`.claude/agent-common.md`「执行前拒绝的写法」里检出器那一块补 ⑫（写范围）一项，和第 39 行一起在提交时交用户登

#### records/2026-10-21-仓外目录不进仓.md（1 行）

- `records/2026-10-21-仓外目录不进仓.md:41` 「下轮开」「下一批」：真延后，没登记
  原行前 120 字：| 不带 `~/` 的相对写法「`<家目录条目名>/<名>` 80G」不红（第三份补丁的工具实现员按代码推的，没有样本跑过） | 下轮开：下一批治理改动时补一份样本、定要不要判 | — |

#### records/2026-10-22-子agent结果落地审计.md（3 行）

- `records/2026-10-22-子agent结果落地审计.md:262` 「下轮开」「下一批」：真延后，没登记（「九十节各行」不点序号（派发提示定的照旧红））
  原行前 120 字：| ㉕ ㉖ ㉗ ㉙ | 下轮开 | 九十节各行，下一批工具改动 |
- `records/2026-10-22-子agent结果落地审计.md:278` 「留给」：真延后，没登记（去向是轮 landing-audit-r5，不是认得的登记写法）
  原行前 120 字：- 任务 32（定义与规则同步，`research/prompts/subagent-landing-audit-2026-10-22/b10-defs-sync.md`，sha256 `4fd6019d6bf3ba58afa53d3452
- `records/2026-10-22-子agent结果落地审计.md:284` 「下一批」：真延后，没登记（交别的会话的下一批）
  原行前 120 字：- 别的会话通知：`copy-originals.py`、`test-environment-check.py`、`broad-mutation.py`、`replay.sh`、`verify-citations.sh` 补了平常跑设开关就

合计 96 行：真延后没登记 89 行，还是误判 7 行。

## 五、还是误判的 7 行：为什么没再收窄

| 类 | 行 | 想过的文字规则 | 为什么不收 |
|---|---|---|---|
| 「留给」作轮内交接或保留给某人当场用 | 代码欠账批:175、里程碑四预开发:751、里程碑三收尾:1056 | 「留给主 agent」「留给<在跑的腿>」不算 | `records/` 里「实现员留给主 agent 判的」「它留给主 agent 的（写范围外）」这类写法有好几处（`grep -o '.\{0,30\}留给主 agent.\{0,30\}' records/*.md`：删除与并发事务、里程碑三形式化接入、欠账还五条、里程碑三开工第一轮），都是悬着没定的事，正是这一格要抓的；按「留给」后面接谁来放行，会把它们一起放过 |
| 撤回延后（提前做） | 里程碑四预开发:993、doc门禁违规修复:207 | 同一分句里有「提前」或「这一轮全做」的不算 | 「提前到下一批」这种写法，提前了照样是延后；「这一轮全做」只出现过一次，为一行立一条规则不划算，改写那一行更省事 |
| 转述流程或说清单在哪 | 里程碑四预开发:957、里程碑三收尾:758 | 无 | 读的人才分得出这一行是在讲规则还是在延后，文字上一模一样 |

这三类都写进了格的文件头「管不到的」（「照样判（文字分不出，改写那一行）」）。改法交主 agent：要么改写这 7 行（比如「留给攻方腿与判决」改成「交攻方腿与判决」），要么定一种行内豁免写法；后一种算新条款，这一批没加。

## 六、要主 agent 跑的命令（工作目录都是主工作区仓根）

新样本与登记处表都还没暂存，钩子拒子 agent 喂样本。拒绝原文开头是：「doc-process-records.sh 的样本与 /home/fy5090/code/singlefs 的 HEAD 加暂存区不同（18 处：deferred-items-registered-green/.gate-cells（暂存区里没有）…」。所以下面几条先暂存这一批的文件再跑：`.claude/gate.d/doc-process-records.sh`、`.claude/gate.d/deferral-registries.tsv`、`.claude/gate.d/fixtures/doc-process-records.sh/deferred-items-registered-*/`（六份，两份旧的也还没进暂存区）。

1. 样本档（这一道登记的是 samples-light）：
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=stage-selftest.sh 喂 doc-process-records.sh 的样本; 范围=1 道阶段的样本档; 为什么=核任务 47 收窄 deferred-items-registered 后的四份新样本与两份旧样本' bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh $PWD/.claude/gate.d --item doc-process-records.sh`
   看末行「判得都对」。deferred-items-registered 的六份都要判对。
2. 开关逐个打开，在同一条命令前面加 `DEFERRED_ITEMS_BREAK=<项>`。每一项判错的样本应当正好是第三节表里点名的那几份；四个旧开关照旧（pointers-unresolved-accepted → red；quotes-counted、registry-rows-judged、fenced-lines-judged → green）。共用库的两条：`STAGE_CELLS_BREAK=red-swallowed` 应当多出 topic-red、registry-red 两份；`=marker-ignored` 应当多出四份新样本。
3. 复跑格与结构格：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 stage-break-switch-replay、gate-structure、fixture-claims 三格; 范围=3 格; 为什么=核任务 47 新加的八个弄坏开关、格描述与新样本' bash .claude/gate.d/code-tooling.sh --check stage-break-switch-replay,gate-structure,fixture-claims`
4. 真仓单跑这一格看读数（会红，红行就是第四节那 96 行；基准换成 gate 取法之后，条数可能不一样）：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-process-records.sh 的 deferred-items-registered 格; 范围=1 格; 为什么=看收窄后真仓 records/ 里延后行登没登记' bash .claude/gate.d/doc-process-records.sh --check deferred-items-registered`
5. 改过的门禁归提交前那一趟的 gate-lint、shell-lint、preflight-lint、gate-overlap（`scripts/gate/gate-staged.sh`）。新建的 `.claude/gate.d/deferral-registries.tsv` 要不要登进 `.claude/batch-scope`，由 batch-scope 格判，归主 agent。

## 七、没做什么

- 没跑重型测试（checker-tier-crash-point-replay、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay 本身、checker-tier-broad-mutation 与 `scripts/mutation/broad-mutation.py` 起 cargo 的用法、`gate.sh`、全量 `cargo test`、`check.sh`）；没走定义三方；没提交。
- 没经 stage-selftest 喂样本，也没跑 code-tooling 的复跑格（钩子拒，见第六节）。草稿里只单跑了从阶段原样抽出的 python 与 shell 函数。
- 没跑命名检查：这一批没改 `.rs`。没改 `records/` 下任何文件，没碰别的格，没碰 `.claude/batch-scope` 与 `stage-owners.tsv`（那一行的归属不变）。
- 7 行误判没有再收窄（第五节）；89 行真延后的补登记与改写归主 agent 与各会话。
- 「推迟」「进欠账」「要等」这类不在词表里的延后照旧判不到（文件头「管不到的」），这一批没加词。

## 草稿目录

`~/.local/share/singlefs/work/landing-audit-b12-deferral-narrow/` 里有：`cell.py`（新判法，与阶段里嵌的那段逐字节相同，另一份是 `cell-from-stage.py`）、`cell-task41.py`（改前那一版）、`function-from-stage.sh` 与 `run-function.sh`、`run-cell.sh`、`judge-switches.py` 与输出 `judge-switches.txt`、`fixtures/`（六份样本的源）、`real/`（真仓新增行与各版读数）、`verdicts.py` 与 `verdicts.md`、`originals/`（开工时五份文件的底与清单：阶段、两份旧样本的 setup.sh 与 expect）、`gate-overlap-list.txt`、`list.txt`。没有仓副本、编译目录与库拷贝；跑样本建的临时树都在用完时删了。
