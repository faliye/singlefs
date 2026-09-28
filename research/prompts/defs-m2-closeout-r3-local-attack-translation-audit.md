# 转述核对表：defs-m2-closeout-r3-local-attack（2026-09-26）

逐句核对 `research/prompts/defs-m2-closeout-r3-local-attack.md`（本地攻方 K2 的 G7：
按事实表逐格核十道阶段各自读不读实验页或实验索引、在 `.claude/gate.d/stage-owners.tsv`
里登记给不给执行员，T1–T10 / SRC-1–SRC-10 / REG-1–REG-10 / Q1–Q13）里每一句英文转述
与中文原文的对照。行号现查：全部十道阶段脚本与 `.claude/gate.d/stage-owners.tsv` 都用
工作区当前文件（这一轮被判的正是这批文件今天的字面本身，不是历史快照）。

提示本身不引用任何文件名加行号（按共用约束「英文提示里的每一句转述」一节与本地攻方
定义「答复不写代码行号与文件行号」），只在这份核对表里写死来源文件与行号；核对表与
运行记录里，样本自带的行号（若模型自己写出行号）一律标「模型自给、未核」。

格式：英文项 / 原文文件:行 / 首稿缺的 / 定稿理由。核对粒度按「同一句中文」分组。

## 表一：项目背景与立场框定的转述（六句以内）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿理由 |
|---|---|---|---|
| 提示第三段第二句（registry 记录「哪个角色负责在自己活干完之后跑这个阶段」；experiment runner 是其中一个角色，可以与别的角色名同现一行，逗号分隔） | `.claude/gate.d/stage-owners.tsv:1`（「门禁阶段归属：每个项目本地阶段先由哪个 agent 在干完自己的活之后跑。提交前的整轮门禁（gate-triage）照旧跑全部阶段。」）与 `:2`（「三列用制表符分隔：阶段文件名、agent 名（逗号分隔，每个名字要有`.claude/agents/<名字>.md`），为什么归它。」） | 无遗漏：「先……在干完自己的活之后跑」译成「responsible for running that stage once their own other work is done」，「先」的时序义保留在「once ... is done」；「逗号分隔」译成「separated by commas」 | 「提交前的整轮门禁（gate-triage）照旧跑全部阶段」这半句与本轮攻击面（十道阶段各自登记给不给执行员）无关，不译入提示，只译与本题相关的半句；这不是摘句，是「这句话本身有两个分句，一句相关一句不相关」的取舍，且不改变相关分句的字面 |
| 提示第三段第三句（实验正文与索引文件都住在一个路径名含 experiments 的目录下） | `.claude/gate.d/34-experiment-index-sync.sh:4`（「`experiments.md` 的索引表是检索这批实验的入口，而编号的登记位在各正文的」）与 `.claude/gate.d/40-results-cited.sh:15`（「⚠️ 2026-08-29 起实验正文拆到 `kb/experiments/` 下，索引只剩导航表。」） | 无遗漏：「索引表是检索入口」译成「one index file that lists which experiment numbers exist」，「导航表」与「索引的入口」同义；「正文拆到 kb/experiments/ 下」译成「both the experiment pages and the index file live under a directory path that contains the word experiments」 | 两处原文分别只讲索引、只讲正文目录，提示合并成一句背景陈述，因为本地攻方这一格不判索引与正文两者的历史沿革（这属于云端腿的语义范围），只需要模型知道「两者都住在含 experiments 的路径下」这一个事实，供 SRC 分类时判断 ENUMERATES-PATHS / NAMES-PATH-ONLY 之类范畴 |
| 提示第五段（十道阶段的主张：都读了实验页或索引、且都登记给了执行员） | `research/prompts/_defs-m2-closeout-r3-body.md:36`（「按事实表逐格填：十道阶段各自读不读实验页或索引（修定义的 agent 报告第一节表下的逐道 grep 行号）、登记给不给执行员（`.claude/gate.d/stage-owners.tsv`）；每行写来源文件与行号，每张表不超过 6 行」）与 `/tmp/claude-1000/defs-closeout-r2-fixes/report.md:26`（「第 6 步「84 号除外」改成「读实验页或实验索引的那几道除外（27、34、40、69、75、84、85、86、88、99 号）」……」） | 无遗漏：「各自读不读」译成 T1–T10 逐条「claims that stage N ... reads an experiment page or the experiment index」；「登记给不给执行员」译成「is assigned ... to the experiment runner role in the registry file」 | 十道阶段号（27、34、40、69、75、84、85、86、88、99）原样抄进 T1–T10，一个不漏、顺序不变 |
| 提示第七段（本地这一格不判「归给执行员是不是好主意」，只判 exhibit B 的证据撑不撑得住 exhibit A 的主张） | `research/prompts/_defs-m2-closeout-r3-body.md:38`（「两条攻方腿不重叠：Opus 造情形攻语义，本地只逐格核 G7 那张名单的字面依据。」） | 无遗漏：「只逐格核……字面依据」译成「only whether the specific quoted evidence in exhibit B actually supports each half of exhibit A's claim, taken strictly on its own」 | 「Opus 造情形攻语义」半句不译入提示（本地腿不需要知道另一条腿在做什么，且禁读 sonnet/opus 那一轮的提示与产出），只把「本地只核字面依据、不判是不是好主意」这个立场译进去 |

## 表二：类别框架的说明（不是转述，是本地攻方自己设计的操作化定义）

READS-CONTENT / CHECKS-EXISTENCE / ENUMERATES-PATHS / NAMES-PATH-ONLY / OTHER
这五个类别名字与定义，不对应任何一句中文原文——它们是本地攻方为了把「读不读实验页
或索引」这个模糊问题拆成可逐格判的操作化定义而自己设计的框架，因此不进「转述对照」，
只在这里注明来源是本地攻方本轮的设计选择，不是翻译。

## 多出来的：英文比原文多的限定词或括注

| 多出来的英文 | 出现位置 | 为什么加 |
|---|---|---|
| 「even though each stage's real script file is much longer than the one line quoted here」 | 提示第五段末句 | 原文（report.md:29 与 body.md:36）都没有这句提醒，是本地攻方加的：防止模型因为常识推断「这么短的脚本不可能做实验相关的事」而误判，明确告知它评判范围只限于给定的那一两行，不代表整份脚本 |
| 「treat exhibit B as the complete truth about each stage for the purposes of this task」 | 提示第五段末句 | 同上，明确任务边界，防止模型去猜测脚本里没给出的其他部分 |

## 表三：事实表来源（提示里不写行号，行号只记在这里）——T1–T5

| 项 | 阶段 | SRC 来源文件:行（原样引文） | REG 来源文件:行（原样引文） |
|---|---|---|---|
| T1 | 27 | `.claude/gate.d/27-format-constants.sh:56`（`kb_all = sorted(glob.glob('.claude/kb/**/*.md', recursive=True))`） | `.claude/gate.d/stage-owners.tsv:13`（`27-format-constants.sh	kb-scribe,experiment-runner	kb 登记的格式常量与实验源码两边`） |
| T2 | 34 | `.claude/gate.d/34-experiment-index-sync.sh:30-31`（`IDX=.claude/kb/experiments.md` / `EXP=.claude/kb/experiments`） | `.claude/gate.d/stage-owners.tsv:20`（`34-experiment-index-sync.sh	experiment-runner	实验索引行与正文标题`） |
| T3 | 40 | `.claude/gate.d/40-results-cited.sh:17`（`EXP_DIR=.claude/kb/experiments`） | `.claude/gate.d/stage-owners.tsv:26`（`40-results-cited.sh	experiment-runner	实验产物写回`） |
| T4 | 69 | `.claude/gate.d/69-evidence-in-repo.sh:119`（`    directory = os.path.join(kb_dir, "experiments")`） | `.claude/gate.d/stage-owners.tsv:58`（`69-evidence-in-repo.sh	experiment-runner,kb-scribe,gate-triage	……`） |
| T5 | 75 | `.claude/gate.d/75-decision-experiment-links.sh:40`（`if [[ ! -d .claude/kb/experiments || ! -d .claude/kb/decisions ]]; then`） | `.claude/gate.d/stage-owners.tsv:61`（`75-decision-experiment-links.sh	kb-scribe,experiment-runner	……`） |

## 表四：事实表来源——T6–T10

| 项 | 阶段 | SRC 来源文件:行（原样引文） | REG 来源文件:行（原样引文） |
|---|---|---|---|
| T6 | 84 | `.claude/gate.d/84-verdict-false-named.sh:73`（`EXP_DIR = '.claude/kb/experiments'`） | `.claude/gate.d/stage-owners.tsv:48`（`84-verdict-false-named.sh	experiment-runner	……`） |
| T7 | 85 | `.claude/gate.d/85-repro-command.sh:14`（`EXP_DIR=.claude/kb/experiments`） | `.claude/gate.d/stage-owners.tsv:49`（`85-repro-command.sh	experiment-runner	实验复跑命令`） |
| T8 | 86 | `.claude/gate.d/86-experiment-orphans.sh:13`（`EXP_DIR=.claude/kb/experiments`） | `.claude/gate.d/stage-owners.tsv:50`（`86-experiment-orphans.sh	experiment-runner	实验号在 kb 里有正文`） |
| T9 | 88 | `.claude/gate.d/88-quoted-result-lines.sh:54`（`kb = sorted(f for f in glob.glob('.claude/kb/**/*.md', recursive=True)`，本行是多行语句的第一行，只抄这一整行） | `.claude/gate.d/stage-owners.tsv:52`（`88-quoted-result-lines.sh	experiment-runner	正文整行抄的产物行`） |
| T10 | 99 | `.claude/gate.d/99-multipath-registry.sh:33`（`EXPERIMENTS=.claude/kb/experiments`） | `.claude/gate.d/stage-owners.tsv:76`（`99-multipath-registry.sh	experiment-runner	多条路径互证的登记：路径、源码落点与共用项由跑实验的人写，写完就跑`） |

⚠️ 报告 `research/prompts/defs-closeout-r2-fixes-tmp-evidence/report.md:29` 给 84 号的分组引文是
「84、85（`:14` `EXP_DIR=.claude/kb/experiments`）」，把 84 与 85 并列只标一个行号；
现查（`grep -n` 现取）之后，84 号自己的 `EXP_DIR` 赋值行在 `84-verdict-false-named.sh:73`，
不在 `:14`——`:14` 是 85 号自己文件里的行号。T6/SRC-6 因此不沿用报告的分组引文，改用
84 号脚本自己现查到的 `:73` 那一行；这处偏差不影响本轮提示内容（提示不含行号），只记在
这里备核。
