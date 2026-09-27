# defs-m2-closeout-r3 云端正推（Sonnet）：K2

判 K2：G2、G5、G7 改后字面是不是第二轮判决（`research/prompts/defs-m2-closeout-r2-main-verification.md` 第三节）要的；G7 从判决括注三道扩成十道，扩得有没有依据、有没有漏、有没有做过头。探针：`research/prompts/defs-m2-closeout-r3-sonnet-model/rerun.sh`，输出 `rerun-output-2026-09-26.txt`（`SHA256SUMS` 校验）。全部引文先 `grep -nF` 命中一次才写进本报告；命中行号见对应命令。

## 一、G2：撤回「逐字节一致就删新文件」

r2 判决第三节原文（`research/prompts/defs-m2-closeout-r2-main-verification.md:32`）：

> - **G2** 撤回 F7 的「逐字节一致就删新文件」：执行员不删任何产物；这一次的输出写新文件名，`research/scripts/replay.sh` 的登记行改指新文件，旧的留着。

改后字面（`.claude/agents/experiment-runner.md:31`，`grep -nF` 命中一次）：

> 这一次的新文件也不删：重跑已有实验时，第 5 步把 `research/scripts/replay.sh` 里这个实验的登记行改指新文件，旧的留着；拿新文件与已有的那份比，逐字节一致的在报告里写明，对不上的在实验页两份都点名、写明对不上，交主 agent 定哪份承重。

判定：**一致**。三项要求（不删、写新文件名、`replay.sh` 登记行改指新文件、旧的留着）逐字对上。多出的「拿新文件与已有的那份比……」不是 G2 新加的义务，是把 F7 原文里「重跑已有实验时拿新文件与已有的那份比……对不上就两份都留，实验页两份都点名、写明对不上，交主 agent 定哪份承重」这半句原样保留（`_defs-m2-closeout-r3-diff.md:41` 那一行 diff 可见，旧行只删了「逐字节一致就删掉这一次的新文件、不新存」这一小句）——改动是精确的最小改法，没有借机加码，也没有漏改。

**什么会推翻它**：如果能在 `.claude/agents/experiment-runner.md` 里找到一处仍写着「逐字节一致就删」或类似字样，或者 `research/scripts/replay.sh` 的登记行改法与「旧的留着」冲突（例如要求覆盖旧登记行），就说明字面没改干净。我 grep 了 `逐字节一致就删` 全文件零命中（见 rerun.sh 之外的补充检查，见下）。

补充检查（未写进 rerun.sh，这里补一遍）：

`.claude/agents/experiment-runner.md` 里「逐字节一致就删」零命中，确认改干净。

**G2 与 40 号 `*.r[0-9].out` 跳过规则的交叉**（K2 问题栏点名的子问）：`.claude/gate.d/40-results-cited.sh:19` 原文（`grep -n 'r\[0-9\]\.out' .claude/gate.d/40-results-cited.sh` 命中）：

> `case "$b" in *local*|*.r[0-9].out|*.round*|confirm*) continue;; esac`

而 `.claude/agents/experiment-runner.md:31` 明写新文件名「带今天的日期或 `rN`」。一个形如 `e57_foo.r3.out` 的新文件，会被 40 号当成「逐轮中间件」永久跳过点名检查——G2 之后这类文件不再被删，会一直留在 `research/results/` 里却永远不必被 `kb/experiments.md` 点名。这正是修定义的 agent 自己在报告「没做的与原因」一节承认的缺口（`/tmp/claude-1000/defs-closeout-r2-fixes/report.md:639`，原句「攻方 H2 附带的一格（40 号把 `*.r[0-9].out` 当逐轮中间件跳过，G2 之后新文件常写成 `rN`）不在 G1–G9 里，没动」）。**判定：一致（缺口属实，且被如实披露为『没做』，不是被掩盖的discrepancy）**。这是 K3/K1 之外的一个真实的、公开承认的遗留缺口，我判给它「一致」是指「报告对这个缺口的陈述与代码现状相符」，不是指「这个缺口不存在」——它仍然是个可以在后续轮次被攻的点。

**什么会推翻这条**：如果 40 号的 skip 规则被悄悄改掉、或者 G2 改动其实把新文件名规则改成不会撞上 `.r[0-9].out` 的形状，这条判定就要重新核；目前两处原文都没有变。

## 二、G5：判决行点名对象改成「表示没过的字段」

r2 判决第三节原文（`defs-m2-closeout-r2-main-verification.md:35`）：

> - **G5** 执行员第 4c 步：点名的对象是判决行里表示「没过」的字段——布尔值为 false、取值为 `not_run`、以及数值计数里名字表示违例、不匹配、歧义、失败且值大于 0 的；布尔值为 true、取值为 `zero`、以及名字不表示「没过」的计数（例如 `journal_differing_states`）不点名。拿不准的点名并写「拿不准」。

改后字面（`.claude/agents/experiment-runner.md:33`，`grep -nF` 命中一次）：

> ……在报告里逐个点名其中表示「没过」的字段：取值是布尔 `false` 的；取值是 `not_run` 的；取值是大于 0 的整数、而名字表示违例、不匹配、歧义、失败的计数（例 `layer0_violations`、`width_mismatches`、`today_ambiguous`）。取值是布尔 `true` 的、取值是 `zero` 的（例 `control_violations_ok=true`、`layer0_violations=zero`）与名字不表示「没过」的计数（例 `journal_differing_states`）不点名；拿不准的点名，写「拿不准」。

判定：**一致**。三类点名、三类不点名、「拿不准」兜底，逐条对上；改后字面比 r2 原文多举了三个例字段（`layer0_violations`、`width_mismatches`、`today_ambiguous`），例子不改变判据本身。

**在今天全部判决行里各落哪一边**（K2 问题栏点名的子问，现查 `research/results/*.out` 里全部 26 行 `name=verdict`/`name=verdict_layer0`，命令与原样输出见 rerun.sh 第 6 节）：

| 字段（今天出现过的） | 判给哪一边 | 依据 |
|---|---|---|
| `layer0_states_ok=true` / `control_states_ok=true` / 等一批 `_ok=true` | 不点名 | 布尔 true |
| `layer0_states_ok=not_run`、`layer0_violations=not_run`、`journal_differing_states=not_run` | 点名 | 取值 not_run（这条不看名字，`journal_differing_states=not_run` 照样点名，`journal_differing_states=3` 不点名——同一个字段名两种取值两种判法，规则自洽） |
| `layer0_violations=0`、`width_mismatches=0` | 不点名 | 数值计数值不大于 0 |
| `layer0_violations=zero` | 不点名 | 取值字面是 `zero`，r2 原文自己举的例子 |
| `control_violations_ok=false`、`control_outcome_matrix_ok=false`、`g7_states_ok=false`、`g7_violations_zero=false`、`r18_g8_ok=false`、`r18_g9_ok=false` | 点名 | 布尔 false（`g7_violations_zero` 名字带「zero」但取值是布尔而非字面 `zero`，走布尔规则，不落入「取值为 zero 不点名」那一类——两条规则不冲突） |
| `control_violations_ok=true` | 不点名 | 布尔 true，r2 原文自己举的例子 |
| `today_ambiguous=0` | 不点名 | 名字含「歧义」但值不大于 0 |
| `journal_differing_states=3` | 不点名 | 名字不表示「没过」，r2 原文自己举的例子 |
| `trees_with_height_difference=0`、`layout_total=413` 等非违例/不匹配/歧义/失败命名的计数 | 不点名 | 名字不落入四个关键词，且今天取值都不大于会引发争议的门槛 |

**今天的数据里没有出现真正「拿不准」的格**：唯一名字接近歧义地带的 `trees_with_height_difference` 今天取值是 0，不触发判据的任一支；没有一个字段同时具备「计数、名字含糊、值大于 0」这三个条件。这与 H5 当初打中的两个具体缺口（漏点名 `journal_differing_states`、错点名 `*_violations_ok=true` / `layer0_violations=zero`）在今天的真实数据上都被正确修复：改后规则对 `journal_differing_states=3`、`control_violations_ok=true`、`layer0_violations=zero` 三个 H5 点名的例子，都判「不点名」，与 H5 的要求一致。

**什么会推翻它**：往产物里加一个「计数、名字含糊指向没过、取值大于 0」的字段（比如 `trees_with_height_difference` 真的大于 0），或者在 `_zero=false` 与「取值字面 zero」之间发现规则实际实现（而不是文字）互相打架，就说明这条判定站不住；今天没有这样的字段出现。

## 三、G7：从判决括注三道扩成十道

r2 判决第三节原文（`defs-m2-closeout-r2-main-verification.md:37`）：

> - **G7** 执行员：登记给它的阶段里凡是读实验页的（84、40、86 号）都挪到第 7 步写完实验页之后跑；这一次不写实验页的在第 6 步最后跑，红了照写。

括注三道（84、40、86）来自 H7（同一份判决第二节，`defs-m2-closeout-r2-main-verification.md:22`：「打中：40、86 同形，只挪了 84 号」），是「H7 具体指出的例子」，而判据本身写的是「凡是读实验页的」——一条开放式规则，不是三个编号的封闭列表。

改后字面（`.claude/agents/experiment-runner.md:35`，`grep -nF` 命中一次）：

> 读实验页或实验索引的那几道除外（27、34、40、69、75、84、85、86、88、99 号）：它们放到第 7 步写完实验页之后跑；这一次不写实验页的，在这一步最后跑，红了照写，产物照第 4 步一个不删（86 号出路里「把 research 下那些文件删掉」那一句不照做，写进报告交主 agent）。

### 3.1 扩得有没有依据

修定义的 agent 自己在报告里写了现查方法（`/tmp/claude-1000/defs-closeout-r2-fixes/report.md:29`）：先用「共用约束『门禁』一节」的 `awk` 取法列出登记给 experiment-runner 的全部阶段，再逐道 `grep -n 'kb/experiments\|experiments\.md\|\.claude/kb\|KB=\|kb_dir'` 现查。我用同一条 `awk`（rerun.sh 第 3 节）独立复现，取到的 14 道与报告完全一致：

```
27 33 34 40 52 69 75 80 84 85 86 88 96 99
```

我再对这 14 道逐一 `grep`（rerun.sh 第 4 节，原样输出见 `rerun-output-2026-09-26.txt`），读实验页或索引的是 27、34、40、69、75、84、85、86、88、99（十道），不读的是 33、52（读的是 `.claude/kb/layout/01-first-txn.md`，不是实验页）、80（零命中）、96（读的是 `.claude/kb/checks-owed.md`）。与报告的分类逐道对上，**十道名单有依据、可独立复核**。

### 3.2 有没有漏一道

14 道是 `stage-owners.tsv` 里登记给 `experiment-runner` 的全集（rerun.sh 第 3 节现算，非手数），逐道扫过一遍没有遗漏对象；扫描范围与判据字面「登记给它的阶段」一致，没有把不归 experiment-runner 的阶段（如 15、87，定义里已另行写明「不归你」）算进来，也没有漏掉这 14 道里的任何一道。**没有漏道**。

### 3.3 有没有做过头——两处值得记一笔

**第一处：判据的措辞从「实验页」扩到「实验页或实验索引」。** r2 原文只写「读实验页的」，34 号读的主要是索引文件 `.claude/kb/experiments.md`（登记表，见 `.claude/gate.d/34-experiment-index-sync.sh:30`）而不是单篇实验正文；改后字面把「或实验索引」明写进了除外条件（第 6 步原文，见上）。这是把 r2 的一般原则（同一机理：главный agent 在第 7 步才写就绪的东西，之前跑就可能落空）套用到索引文件上，逻辑站得住——索引行与正文一样在第 7 步才写（定义第 7 步「索引行进 `.claude/kb/experiments.md`」），但**这一步解释超出了 r2 判决的字面**，是修定义的 agent 自己做的外推，没有再等一轮判决确认。这不是「改宽」（没有放松任何约束，只是把更多阶段挪到更晚跑，属于收严的方向），但按「臂的定义也在跑前写死之列」的精神，这类解释性扩张理应留痕——报告里确实写了「这是按判据字面扩的名单，不是判决括注的原样，第三轮可以攻」（`report.md:29` 末句），做到了留痕，我判**可接受，但明确留给第三轮攻**。

**第二处：十道里有三道（27、69、88）的「读」是通过一个覆盖全部 `.claude/kb/**/*.md` 的宽扫描碰到实验页，不是专门针对某一篇实验正文是否存在。** 我读了这三道的完整逻辑（不止 grep 命中行）：

- `27-format-constants.sh:56`：扫全部 kb 的 `format-const` 标记，与 `crates/`、`research/` 源码里的同名常量比对。新实验页不存在时，它就是简单地少扫到那一页——**不会因此判红**，因为它的判据是「已登记的标记要与源码一致」，没有登记（页不存在）就不参与比较（`27-format-constants.sh:88` `if declaration.name not in marks: continue`）。
- `69-evidence-in-repo.sh:119`：核对「产物旧时改没改对应实验页」，同样是遍历现存的 `.claude/kb/` 目录树（`69-evidence-in-repo.sh:220-221`），新页不存在时不在遍历范围内，不产生红。
- `88-quoted-result-lines.sh:54`：核对 kb 与 research 正文里整行抄的 `E7RESULT` 行是否能在产物里逐字找到，同样是扫现存的 `.md` 文件；新页不存在时没有可扫的引文，不产生红。

这三道与真正会「因为页不存在而判红」的 34（索引行存在但正文缺失会红，见 `34-experiment-index-sync.sh:67-69`）、40（产物文件存在但点不到名会红）、84（判决行有 false 却在（不存在的）实验页里找不到点名会红）、86（有实验号但正文缺失会红）、75（`.claude/kb/experiments` 目录必须存在，否则红，`75-decision-experiment-links.sh:40-41`）、85、99 不是同一种风险形状。报告把这七道一并归为「另外七道在页写出来之前跑是空判（新实验的页不在，判不到它）」（`report.md:29` 原句），这句话对 34/75/85/99 成立，对 27/69/88 不准确——27/69/88 的实际情况是「少扫一点，不会判红」，不是「空判」。

**判定：这一处不算做过头（挪到第 7 步之后跑对 27/69/88 无害，甚至能多覆盖新页的内容），但 report.md 给出的统一理由「空判」对这三道不成立，是一处不精确的归因**，值得在判决里指出、留给攻方核实是否需要更正说法（不必更正名单本身）。

**什么会推翻它**：若能找到 27、69、88 里任何一处逻辑，在新实验页不存在时会误判红（而不是仅仅少覆盖），那么「不算做过头」的判定要改判——需要重新读这三个脚本判红分支的每一个前置条件；我读到的分支（引用行号见上）都是「先收集现存文件 → 比对」，没有「要求某个特定新文件必须存在」的分支。

### 3.4 挪了之后第 6 步还剩什么

14 道减去挪走的十道，剩 33、52、80、96 四道留在第 6 步（rerun.sh 第 3、4 节可现算）：33（变异表复跑，不读任何 kb 路径）、52（读 `.claude/kb/layout/01-first-txn.md` 的段序列登记表，与实验页无关）、80（不读 kb 路径）、96（读 `.claude/kb/checks-owed.md` 欠账表，与实验页无关）。`.claude/agents/experiment-runner.md:35` 原文「这一次不写实验页的，在这一步最后跑」覆盖了「这一轮压根不写实验页」的分支（例如只补变异表锚点、或跑前登记里没安排写实验页），此时十道回到第 6 步最后跑，不会因为第 7 步永远不发生而被无限期搁置——这条分支写得完整。

### 3.5 一处引证不严谨（84、86 缺行号）

修定义的 agent 的报告在列十道依据时写「84、85（`:14` `EXP_DIR=.claude/kb/experiments`）、86、88（`:54` `.claude/kb/**/*.md`）」（`report.md:29`），行号 `:14` 只属于 85（我核对 `.claude/gate.d/85-repro-command.sh:14` 确实是 `EXP_DIR=.claude/kb/experiments`），`:54` 只属于 88；84、86 在这句话里没有各自的行号。我独立核实：84 真正依赖的是 `.claude/gate.d/84-verdict-false-named.sh:73`（`EXP_DIR = '.claude/kb/experiments'`），86 是 `.claude/gate.d/86-experiment-orphans.sh:13`（`EXP_DIR=.claude/kb/experiments`）——两道确实都读实验页，实质结论不受影响（且 84、86 本来就是 r2 判决括注里明确点名的两道，H7 已经证实过），但报告这句话的行号标注方式容易让人误以为 84、86 没有独立证据、只是搭 85/88 的车。**这是一处可改进的证据展示问题，不影响判定，但值得记一笔。**

## 四、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| G2 字面 | 一致 | 撤回删文件、写新文件名、`replay.sh` 改指新文件、旧的留着，逐字对上 r2 第三节；多出的比较逻辑是保留 F7 原文里没被撤回的那半句，不是新加义务 |
| G2 与 40 号 `.r[0-9].out` 交叉 | 一致（缺口属实且已披露） | `rN` 命名的新文件会撞上 40 号「逐轮中间件」跳过规则，永远不必被点名；修定义的 agent 自己在「没做的与原因」里承认未处理，未被掩盖 |
| G5 字面 | 一致 | 三点名、三不点名、拿不准兜底逐条对上 r2 第三节；例字段是补充说明，不改判据 |
| G5 在今天全部判决行上的分类 | 一致，且 H5 打中的两个具体缺口在真实数据上确认已修复 | 26 行判决行逐格分类，`journal_differing_states=3`、`control_violations_ok=true`、`layer0_violations=zero` 均正确判「不点名」；今天数据里没有出现真正「拿不准」的格 |
| G7 名单十道：有没有依据 | 有依据，可独立复核 | 同一条 `awk` 取到同样 14 道，逐道 grep 分类与报告一致 |
| G7 名单十道：有没有漏道 | 没有漏道 | 14 道是 `stage-owners.tsv` 现算的全集，逐道核过 |
| G7 名单十道：有没有做过头 | 名单本身不算做过头；统一理由「空判」对 27/69/88 三道不准确 | 27/69/88 是宽扫描，页不存在时只是少扫、不会判红，与 34/40/75/84/85/86/99 那种「会真的判红」的风险不是同一种；挪后不会有害，但报告的归因需要更正说法 |
| G7「实验索引」扩到判据字面之外 | 可接受但留痕不足以自动过关，交第三轮 | r2 原文只写「实验页」，改法把「实验索引」也纳入除外名单，逻辑站得住（索引同样在第 7 步才写），但这是修定义的 agent 自己的外推，报告里写了「按判据字面扩，不是判决括注原样，第三轮可以攻」，算留了痕 |
| G7 第 6 步剩余对象 | 一致 | 33、52、80、96 四道留在第 6 步，均确认不读实验页或索引 |
| 84、86 的引证行号 | 有瑕疵，不影响结论 | 报告列证据时 84、86 没有各自的行号（借用了 85、88 的），我补了 `84-verdict-false-named.sh:73`、`86-experiment-orphans.sh:13` |

## 没做什么

- K1（G1、G9，74/15 号内存包装分界与阶段间退出码区分）、K3（G3、G4、G6、G8）不归 K2，没有判。
- 没有核 G1–G9 之外任何字面（例如 G8 本地辩方样本前缀、G6 退出码文件批号件号），那些在 K1/K3 范围。
- 没有跑重型测试、没有编译 `crates/`——这一轮不碰 `crates/`（背景材料二「实现今天的样子」已写明），K2 涉及的都是 agent 定义与门禁阶段脚本的静态核对，用 `grep`/`awk` 现查即可。
- 没有派本地攻方或云端攻方核对 G7 名单——按分工表那是本地攻方与 Opus 的任务（本地攻方专核 G7 事实表，Opus 造派发情形攻 K1/K2/K3 全部）。
- 3.3 节指出的「27/69/88 归因不准确」与 3.3 节的「实验索引扩到字面之外」两点，我只指出问题、没有替主 agent 定要不要因此打回重写——那是判决该做的事。
