#!/usr/bin/env python3
"""治理文档、kb 与仓根 README（README.md、README.en.md、README.ja.md）里对门禁的指向还指不指得到：门禁 doc-registries 的 governance-refs 那一格调它，不单独成阶段。

治理文档：CLAUDE.md、.claude/main-agent.md、.claude/agent-common.md、.claude/agents/*.md、.claude/rules/*.md、.claude/skills/*/SKILL.md。
现状文字：治理文档加 .claude/kb/ 下全部 .md 与仓根的 README.md、README.en.md、README.ja.md（三语文件，在的都算），去掉「## 历史版本」那一节（到下一个「## 」标题或文件末尾）与
decisions-history.md、experiments-history.md、CHANGELOG.md 三种变更史；research/prompts/、records/、.claude/warnings/ 不在射程里（冻结的证据与记事）。
仓根的三份 README 只判 ④ ⑤（它们不是治理文档，① ② ③ 不扫它们）。

只扫治理文档的三类（原样）：
  ① 门禁号：「门禁 65 号」「54、55、57、59 号」这类两位数加「号」，门禁目录里要有 <号>-*.sh。
     「、」或「/」连着的每个号都判（「20 / 21 号」判 20 与 21）；前面紧挨「第」「月」「日」（可隔空白）的不算门禁号（「第 13 号」「9 月 23 号」），
     紧挨「-」「–」的也不算（「2026-09-26 号」；「54–59 号」这种区间写法不判，要判就逐个写出来）。
     门禁目录取本文件所在的目录，不取被扫的仓根：样本目录里没有门禁脚本。去编号之前的旧号不归这一条，归 ⑤。
  ② 反引号里的仓内路径：反引号里按空白切开的每个词（命令里的脚本路径也算），ASCII 写成、中间带 `/`、
     而且第一段在解析基准下现存、或末段带常见的文件扩展名（`origin/master`、`text/plain` 两样都不满足，不算仓内路径）。
     解析基准：仓根、所在文件的目录、`.claude/`、`.claude/kb/`，外加同一个反引号里 `cd 目录` 之后的那个目录；一处在就算指得到。
     跳过（计数报在末行）：被 .gitignore 挡着的（规范副本、构建目录）、`../` `/` `~` `http` 起头的、`refs/` 起头的 git 引用、带占位（`<` `*` `{`）的。
     已归档的写裸文件名（`.claude/agent-common.md`「找不到历史实验的数据」那一条），不带目录，不在射程里。
     指向去编号之前的门禁文件（`gate.d/<旧号>-…`）的不归这一条，归 ⑤。
     指不到的红行后面接后继结论（scripts/kb/path-successor.py 的 conclude，见下面 ② 扩到 kb 那一段的红行写法）。
     自动跳过与保留表和 ② 扩到 kb 那一段是同一套（判法共用 git_history_word_indexes、successor_named_on_line、read_kept_table）：
     git 历史命令里 `--` 之后的词、说搬迁的那一行不判，计进治理文档那一行的两类自动跳过数；指不到而（载体, 路径）登在保留表里的不红、
     计进保留数，保留表这一行核「此刻在了」按 ② 的四个解析基准解析。滞后表只收 kb 那一半的存量，治理文档的不在路径不登滞后表。
  ③ `文件「小节」`：文件在仓里、而「小节」里的字（去掉空白、反引号、星号、「」『』之后）在那份文件的全文里一处都找不到。
     判的是「全文任意位置出现」，不只看标题：定义常点正文里的一句；小节改了名而原名的字还散在正文里时抓不到。
     小节名里嵌套「」的（「「无效」那一栏」），只核到第一个」之前那一截。
     另认三种引用写法（判法 section_found，治理文档与 kb 共用）：嵌套的引号写成『』的（「看它能不能把『用错了』变成『挂不上』」，原文里是「」）；
     编号带简称的（「I-1（块自描述） 块自描述」，原文标题是「I-1 块自描述」：小节名与原文都把紧跟在 C、D、E、I 编号后面的（简称）去掉再比一次）；
     文件是脚本（.sh、.py）时，小节名里单独一个大写字母当占位，对上 PLACEHOLDER_SPAN 那么长的一段（「结果：N 条命中」对上 echo 里的「结果：$pass 条命中」）。
     红行接出路（nearest_text）：那份文件里与小节名最像的标题或句子（小节名的字至少 NEAREST_MINIMUM_SHARE 按次序落在里面，取比例最高的，
     同分先取标题、再取短的）；一条都够不上写「那份文件里没有，改写那一句或删去」。

扫现状文字的两类（2026-09-28 门禁去编号之后加，判决 research/prompts/gate-shrink-r1-main-verification.md 第 4 问「按全名指门禁从此没人判」）：
  ④ 按全名指门禁与格：门禁一律按门禁目录顶层的文件名去掉 .sh 称呼。
     「门禁 <名>」（名是小写字母、数字与连字符、至少带一个连字符，可带 .sh、可套反引号）里的名要是门禁目录顶层的一道门禁，
     或上游规范副本 .claude/singlefs-ai-sop/scripts/ 下的一份脚本（共享门禁的阶段，「门禁 doc-lint」这类）；
     「<门禁名> 的 <格名> 格」（「那一格」「这一格」同）与反引号里「gate.d/<门禁名>.sh … --check <格名>[,<格名>…]」里的格名，
     要在那道门禁文件头的格名表里（`# gate-cell:` 行，经 scripts/gate/gate-structure-check.py 的 header_cells 读）。
     「<名> 的 <x> 格」里 <名> 不是门禁目录里的门禁的不判（分不清是不是在说门禁）。
  ⑤ 用旧编号称呼门禁：「NN 号」（排除与 ① 相同）、「门禁 NN」、「gate.d/NN-…」（含 gate.d/fixtures/NN-…），NN 是下面 OLD_GATES 里的旧号。出路给那个旧号今天在哪道门禁、哪几格。
     「NN 号」「门禁 NN」两种写法先把带目录的文件路径与「日期-」起头的记录文件名遮掉再找（`records/2026-09-28-门禁59号提速与双机分片.md`
     这种文件名里的号不是在称呼门禁；① 同样遮）；「gate.d/NN-」那一种就是路径，不遮。
     OLD_GATES 照调度记录 records/2026-09-28-门禁59号提速与双机分片.md「旧编号与现在的门禁」那张表，外加表里漏了的 96 号
     （实验源码纪律，第一轮并进 33 号，今天是 code-source-discipline 的 reading-discipline 格）；
     表里点名的门禁与格本身也现判：指到门禁目录里不在的门禁或格，这一段自己判红（门禁再改名时这张表要跟着改）。

② 扩到 kb 的现状文字（不变量清单在登记的 kb 目录里，一起扫）：
  扫哪些：登记位 .claude/kb-directories 登记的目录下的 .md（读法只在 scripts/kb/quote-kb.py 的 kb_candidate_roots 一份），只扫中文那一份
     （.en.md、.ja.md 不扫，三语同步归门禁 doc-text 的 trilingual 格）；与 ④ ⑤ 同一套现状文字口径：「## 历史版本」一节、代码围栏、变更史整份不扫。
  判哪些：反引号里的词照 ② 的切法，只判第一段是仓根下的目录（ROOT_DIRECTORIES：crates/ scripts/ .claude/ records/ litmus/ briefs/ research/）的，
     只按仓根解析（不试 .claude/、.claude/kb/、所在目录这几个基准）；相对简写（`layout/01`、`tests/x.rs`、`src/lib.rs`）不判、计进跳过数；
     仓外或带占位的、被 .gitignore 挡着的、指旧门禁文件（归 ⑤）的同样不判、计数。
# gate-similar: doc-experiments.sh 它的 cited-paths-exist 格判 kb 全文（含历史版本节、变更史、英日页与代码围栏）里 research/results/*.out、scripts/mutation/tables/*.tsv、scripts/experiment-apparatus/*.rs 三类路径（反引号内外都认），树里或历史里有就放；这里 ② 只判 kb 中文现状文字反引号里以仓根目录起头的全部路径，不在就红、只 research/ 下进过 git 的放行。判的文字段与判法都不同，只在中文现状文字里那三类上重叠（research/ 那一类两边都是历史里有就放、点名 <名>.out 而只有 <名>.out.gz 存放的两边都认，另两类这里更严），不并
  指不到的：research/ 下的进过 git（git log --all -1 -- <路径> 有输出）放行、计数（归档规矩：.claude/agent-common.md「找不到历史实验的数据」那一条），
     从没进过 git 的判红；别的判红。research/results/ 下的 <名>.out 压成 <名>.out.gz 入库之后只剩后者：在不在、进没进过 git 都照两种存放名查（stored_forms），
     树里或历史里只有 <名>.out.gz 的也算指得到（口径与门禁 doc-experiments 的 cited-paths-exist 格相同，那边由 results-cited 格的 product_stored_forms 列存放名）。红行「<载体>:<行>: `<反引号原文>` 里的 <路径> 在仓里不存在：<后继结论>」，后继结论是 path-successor.py 的结论句：
     有后继的「改成 <后继>（依据）」，删了没后继的「删于 <提交>（标题）；现状句改写或删去，说那一次的事件句写成 `<提交>^:<路径>`」，找不到的「从没进过 git」。
  滞后表 LAG_TABLE（.claude/gate.d/dead-path-lag.tsv，被判的根下）：一行「<载体路径><TAB><不在的路径><TAB># <理由>」，按（载体, 路径）认，不按行号；
     登记了的不红、计进滞后数。列数不对、理由不到 4 个字、同一条登记两次判红；登记的路径此刻在了、或那份载体的现状文字里已经不引它
     （引它的那一处不再是指不到的红：改了、删了、挪进历史节，或是 research/ 下进过 git 的）判红「这一行该删」。
     只缩不涨：与 gate 基准那一版（--base <提交>，调用方给 scripts/gate/changed-paths.sh 的 gate_diff_base gate；取法是 .claude/gate.d/lib-lag-baseline.py 的 baseline_rows）
     按（载体, 路径）比，基准里没有的行判红；基准里还没有这张表的不比（初次登记），没给 --base（不在 git 仓里）的不比，成功行写明。
  保留表 KEPT_TABLE（.claude/gate.d/dead-path-kept.tsv，被判的根下，可以增长）：照规矩该留旧路径的不在路径，一行「<载体路径><TAB><不在的路径><TAB># <理由>」，
     理由写它属于哪种（决策或计划里定的将来落点、句子本身说它不在、抄下来的当时的命令或输出、相对仓外副本的路径），至少 MINIMUM_KEPT_REASON_CHARACTERS 个字；
     按（载体, 路径）认，登记了的不红、计进保留数；载体是治理文档的也认（② 治理文档那一半）。列数不对、理由太短、同一条登记两次判红；登记的路径此刻在了判红「建好了或回来了，删这一行」；
     那份载体的现状文字里已经不引它（口径与滞后表同）判红「这一行该删」；同一对滞后表也登了判红（两张表只登一张）。
     第二列带「的是小节行（③ 扩到 kb，写成红行里的 `<文件>「<小节>」`）：不判「此刻在了」，那份载体的现状文字里已经没有这一处找不到的判「这一行该删」，
     同一对小节滞后表也登了判红。
  两种自动跳过（照 .claude/rules/path-moves.md「怎么做」第 5 步留旧路径的写法，不判、计进跳过数，各报各的数）：
     git 历史命令：反引号里 git log、git show、git blame、git diff 命令中 `--` 之后的词（到 |、;、&&、|| 这类命令分隔为止）；`--` 之前的照判。
     说搬迁的那一行：路径不在、path-successor.py 给出了后继，而后继的文件名（最后一段）在同一行里另有一处（把旧路径本身从那一行里去掉之后还找得到，
     后继与旧路径同名时不会因为旧路径自己就算数）。这一判在查保留表、滞后表之前：跳过了的不进「载体还引着」的集合，登着它的行判「该删」。

③ 扩到 kb 的现状文字（载体与 ② 扩到 kb 同一批：kb_carriers 交的登记 kb 目录下中文页，历史版本节、代码围栏、变更史与英日页不扫）：
  判法与出路同 ③（section_found、nearest_text）；文件在四个解析基准下都找不到的不判、只计进「文件找不到」数，不逐条列（多是只写文件名的已归档提示词与仓外报告）。
  找不到的：（载体, `<文件>「<小节>」`）登在保留表 KEPT_TABLE 的不红、计进保留数（照规矩该留的：句子本身说那一句已改掉、抄下来的当时的原文）；
     登在小节滞后表 SECTION_LAG_TABLE（.claude/gate.d/dead-section-lag.tsv，被判的根下）的不红、计进滞后数；别的判红。
  小节滞后表：写法与滞后表相同，第二列写红行里的 `<文件>「<小节>」`；列数不对、理由不到 4 个字、同一条登记两次判红；那份载体的现状文字里已经没有这一处找不到的
     （小节找得到了，或那一句改了、删了、挪进历史节）判「这一行该删」；只缩不涨，与 gate 基准那一版按（载体, 第二列）比，取法与弄坏开关 lag-growth-unjudged 都与滞后表共用。

判不到、逐条列进「没判的」（不算红）：③ 里文件解析不到的（多半是只写了文件名、住在四个基准之外）；
② 里不全是路径字符（非 ASCII、引号、`$` 这类）、中间带 `/`、又指不到的记号（悬空的中文文件名路径与 `NN-简称.md` 这类占位分不开；指得到的照常算判过）。
够不着的：不带「门禁」二字也不带「的 … 格」的裸名（「跑 doc-kb」）；裸写的旧门禁文件名（「`47-research-script-selftests.sh`」，不带 gate.d/）。

输出：每处红一行「文件:行: 说明」；「没判的」逐条一行；末两行
「扫 N 份治理文档：门禁号 A 处、路径 B 处、小节 C 处；没判 D 处；跳过 E 处（被 .gitignore 挡着 F、不像仓内路径 G、仓外或带占位 H）；
git 历史命令里 -- 之后 X 处、说搬迁的行 Y 处、保留 Z 处」（E 只是括号里三类之和；X、Y 不算判过，Z 算在 B 里）与
「扫 M 份现状文字（治理文档、kb 与仓根 README.md、README.en.md、README.ja.md，历史版本节与变更史除外）：门禁全名 I 处、格名 J 处、旧编号 K 处」，
「kb 路径：扫 Q 份 kb 中文页，判了 P 处（research/ 进过 git 放行 R 处、滞后 L 处、保留 K 处）；跳过相对简写 S 处、被 .gitignore 挡着 U 处、仓外或带占位 V 处、
git 历史命令里 -- 之后 G 处、说搬迁的行 M 处；滞后表 W 行（与基准比的结局）；保留表 X 行」，
「kb 小节：扫 Q 份 kb 中文页，判了 O 处（滞后 L 处、保留 K 处）；文件找不到没判 U 处；小节滞后表 W 行（与基准比的结局）」，
再加最末一行「共查 T 项（门禁号 A + 路径 B + 小节 C + 门禁全名 I + 格名 J + kb 路径 P + kb 小节 O）」，T 是 A、B、C、I、J、P、O 七个数之和（旧编号 K 是红行，不算查过的）；调用方（门禁 doc-registries 的 governance-refs 格）
只从这一行取查了多少项，退 0 却找不到这一行、找到不止一行、或 T 为 0，这一格判红；成功句另抄「kb 路径：」「kb 小节：」两行。
参数：--base <提交>（可不给）：滞后表与之比只缩不涨的 gate 基准。
退出码：0 全指得到；1 有指不到的；3 一份治理文档都没扫到（没有对象，不许当通过）；4 本脚本自己出错（读不了某份文件这类，打出 traceback）；
python 起不来时退 2（打不开脚本），参数认不出也退 2。调用方把 0、1、3 之外的都当「没跑成」。

弄坏开关 GOVERNANCE_REFS_BREAK=<项>（doc-registries 的样本判错）：
  old-numbers-ignored   ⑤ 整条不判：governance-refs-red 里「用旧编号称呼门禁」那几行 want 找不到
  names-ignored         ④ 整条不判：governance-refs-red 里「不是门禁目录里的门禁」「格名表里没有」那几行 want 找不到
  history-scanned       「## 历史版本」一节与变更史也扫：governance-refs-green 判红
  fences-scanned        代码围栏里的行也当现状文字扫：governance-refs-green 里围栏内那行旧路径判红
  readme-skipped        仓根的三份 README 都不进现状文字：governance-refs-red 里「README.md:」「README.en.md:」「README.ja.md:」那几行 want 找不到，
                        governance-refs-green 的现状文字份数与全名、格名处数对不上
  readme-old-name       README_PATTERNS 退回改名之前的 README.zh.md（仓根现在没有这一份，README.en.md 扫不到）：governance-refs-red 里「README.en.md:」那一行 want 找不到，
                        governance-refs-green 的现状文字份数少一份
  count-line-missing    不打末行「共查 T 项」：governance-refs-green 里这一格判红（退 0 却没报出查了多少项）
  count-zero            末行把 T 打成 0：governance-refs-green 里这一格判红（扫到 0 项不算通过）
  kb-unscanned          ② 与 ③ 都不扫 kb（kb_carriers 一份都不交）：governance-refs-kb-red、governance-refs-kb-sections-red 里 kb 那几行红 want 找不到
  research-not-passed   research/ 下进过 git 的也判红：governance-refs-kb-green、governance-refs-kb-gz-green 判红
  gz-storage-ignored    kb 路径只查点名的原样（research/results/ 下的 <名>.out 不认 <名>.out.gz 存放）：governance-refs-kb-gz-green 判红
  successor-dropped     红行不带后继结论：governance-refs-kb-red 里「改成」「删于」那几行 want 找不到
  lag-unread            不读滞后表：governance-refs-kb-green 里登记了的那一处判红，governance-refs-kb-lag-red 里「这一行该删」与涨了的那几行 want 找不到
  lag-growth-unjudged   滞后表与小节滞后表都不与基准比：governance-refs-kb-lag-red、governance-refs-kb-sections-lag-red 里「比 gate 基准那一版多」那一行 want 找不到
  kept-unread           不读保留表：governance-refs-kb-kept-green、governance-refs-governance-skips-green、governance-refs-kb-sections-green 里登记了的那一处判红，
                        governance-refs-kb-kept-red、governance-refs-kb-sections-lag-red 里保留表那几行红 want 找不到，
                        governance-refs-green、governance-refs-kb-green 的「保留表 0 行（没有保留表）」找不到
  kept-reds-unjudged    保留表的三种红（登记的路径在了、载体不再引它、理由不到 8 个字；小节行的该删）不判：governance-refs-kb-kept-red、governance-refs-kb-sections-lag-red 里那几行 want 找不到
  git-history-judged    git 历史命令 `--` 之后的路径照判（治理文档与 kb 两半）：governance-refs-kb-kept-green、governance-refs-governance-skips-green 判红，governance-refs-kb-skips-red 的跳过数对不上
  move-line-judged      说搬迁的那一行照判（治理文档与 kb 两半）：governance-refs-kb-kept-green、governance-refs-governance-skips-green 判红，governance-refs-kb-skips-red 的跳过数对不上
  kb-sections-unscanned ③ 不扫 kb：governance-refs-kb-sections-red 判绿，governance-refs-kb-sections-green 里小节滞后表与保留表那两行成了该删，
                        governance-refs-green、governance-refs-kb-sections-lag-red 的「kb 小节：…判了 N 处」找不到
  nearest-dropped       ③ 红行不接出路：governance-refs-kb-sections-red 里「最像的是…」「那份文件里没有…」两行 want 找不到
  nested-quotes-kept    『』不当装饰去掉：governance-refs-kb-sections-green 里嵌套引号那一处判红
  short-names-kept      编号后面的（简称）不去掉：governance-refs-kb-sections-green 里「I-1（块自描述） 块自描述」判红
  placeholders-literal  脚本输出句里的占位字母照字面比：governance-refs-kb-sections-green 里「结果：N 条命中」判红
  section-lag-unread    不读小节滞后表：governance-refs-kb-sections-green 里登记了的那一处判红，governance-refs-kb-sections-lag-red 里小节滞后表那几行 want 找不到，
                        governance-refs-green、governance-refs-kb-sections-red 的「小节滞后表 0 行（没有小节滞后表）」找不到
  （doc-registries.sh 文件头的开关清单逐条点名每个开关下判错的样本，以那一份为准）
"""
import difflib
import glob
import importlib.util
import os
import re
import subprocess
import sys

sys.dont_write_bytecode = True

GATE_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_OF_GATES = os.path.dirname(os.path.dirname(GATE_DIRECTORY))
UPSTREAM_SCRIPTS = os.path.join(REPOSITORY_OF_GATES, ".claude", "singlefs-ai-sop", "scripts")
STRUCTURE_CHECK = os.path.join(REPOSITORY_OF_GATES, "scripts", "gate", "gate-structure-check.py")
BROKEN = os.environ.get("GOVERNANCE_REFS_BREAK", "")
CARRIER_PATTERNS = ["CLAUDE.md", ".claude/main-agent.md", ".claude/agent-common.md",
                    ".claude/agents/*.md", ".claude/rules/*.md", ".claude/skills/*/SKILL.md"]
KB_PATTERN = ".claude/kb/**/*.md"
# 仓根的三份 README（README.md 是中文，README.en.md 是英语，README.ja.md 是日语）：只进现状文字（④ ⑤），不当治理文档；在的才算
README_PATTERNS = ["README.md", "README.en.md", "README.ja.md"]
# 弄坏开关 readme-old-name：退回改名之前的文件名
if BROKEN == "readme-old-name":
    README_PATTERNS = ["README.md", "README.zh.md", "README.ja.md"]
HISTORY_FILES = (
    "decisions-history.md", "decisions-history.en.md", "decisions-history.ja.md",
    "experiments-history.md", "experiments-history.en.md", "experiments-history.ja.md",
    "CHANGELOG.md", "CHANGELOG.en.md", "CHANGELOG.ja.md",
)
HISTORY_HEADING = re.compile(r"^##\s*(?:历史版本|Historical Versions|履歴バージョン)")
LEVEL_TWO_HEADING = re.compile(r"^##\s")
FENCE_LINE = re.compile(r"^[ \t]*```")
GATE_NUMBER = re.compile(r"(?<![0-9A-Za-z.\-–])((?:[0-9]{2}\s*[、/]\s*)*[0-9]{2})\s*号")
NOT_A_GATE_BEFORE = re.compile(r"(第|月|日)\s*$")
GATE_WORD_NUMBER = re.compile(r"门禁\s*`?([0-9]{2})(?![0-9])")
OLD_GATE_PATH = re.compile(r"gate\.d/(?:fixtures/)?([0-9]{2})-")
NAME = r"[0-9a-z][a-z0-9]*(?:-[a-z0-9]+)+"
GATE_BY_NAME = re.compile(rf"门禁\s*`?({NAME})(?:\.sh)?`?(?![A-Za-z0-9_-])")
CELL_OF_GATE = re.compile(rf"(?<![A-Za-z0-9_-])({NAME})(?:\.sh)?`?\s*的\s*`?([a-z0-9][a-z0-9-]*)`?\s*(?:那一|这一)?格")
BACKTICK = re.compile(r"`([^`\n]+)`")
REPO_PATH = re.compile(r"^[A-Za-z0-9_.\-/]+$")
FILE_WITH_SECTIONS = re.compile(r"`?([^\s`「」]+\.(?:md|py|sh))`?\s*(?:的)?\s*((?:「[^」]+」(?:、|与|和)?)+)")
# 『』是「」里再套一层引号的写法（原文里多半是「」），与「」一样当装饰去掉；弄坏开关 nested-quotes-kept 下不去掉『』
DECORATION = re.compile(r"[\s`*「」]" if BROKEN == "nested-quotes-kept" else r"[\s`*「」『』]")
# kb 引编号要写成「编号（简称）」，引来的小节名因此比原文多一段（简称）：紧跟在 C、D、E、I 编号后面的这一段两边都去掉再比一次
SHORT_NAME_AFTER_NUMBER = re.compile(r"(?<![A-Za-z0-9])([CDEI]-?[0-9]+(?:\.[0-9]+)*)（[^（）]+）")
# 脚本（.sh、.py）的输出句里，数由变量拼出：小节名里单独一个大写字母当占位
SCRIPT_SUFFIXES = (".sh", ".py")
PLACEHOLDER_LETTER = re.compile(r"(?<![A-Za-z0-9_$\-])[A-Z](?![A-Za-z0-9_\-])")
PLACEHOLDER_SPAN = r".{1,40}?"
PLACEHOLDER_MINIMUM_LITERAL_CHARACTERS = 4
# 红行的出路：小节名的字至少这么多比例按次序落在一条标题或句子里，才算「最像的」
NEAREST_MINIMUM_SHARE = 0.6
NEAREST_SHOWN_CHARACTERS = 60
HEADING_LINE = re.compile(r"^#{1,6}\s+(.*)$")
SENTENCE_END = re.compile(r"[。！？；;!?]")
BASE_DIRECTORIES = ("", ".claude", ".claude/kb")
# ② 扩到 kb：只判第一段是这几个仓根目录的路径；后继结论、kb 目录登记、基准那一版的取法各只有一份读法，这里调它们
ROOT_DIRECTORIES = ("crates", "scripts", ".claude", "records", "litmus", "briefs", "research")
ARCHIVED_DIRECTORY = "research"
# research/results/ 下的 <名>.out 压成 <名>.out.gz 入库之后只剩后者：口径同门禁 doc-experiments 的 product_stored_forms（那是 bash 函数，python 这里调不到，照同一条规则写）
PRODUCT_DIRECTORY = "research/results/"
PRODUCT_SUFFIX = ".out"
COMPRESSED_SUFFIX = ".gz"
TRANSLATION_SUFFIXES = (".en.md", ".ja.md")
LAG_TABLE = ".claude/gate.d/dead-path-lag.tsv"
MINIMUM_LAG_REASON_CHARACTERS = 4
SECTION_LAG_TABLE = ".claude/gate.d/dead-section-lag.tsv"
KEPT_TABLE = ".claude/gate.d/dead-path-kept.tsv"
MINIMUM_KEPT_REASON_CHARACTERS = 8
KEPT_KINDS = "决策或计划里定的将来落点、句子本身说它不在、抄下来的当时的命令或输出、相对仓外副本的路径"
# 指 git 历史的命令：这几个子命令里 `--` 之后是 pathspec（path-moves.md「怎么做」第 5 步留旧路径）
GIT_HISTORY_SUBCOMMANDS = ("log", "show", "blame", "diff")
GIT_OPTIONS_WITH_VALUE = ("-C", "-c", "--git-dir", "--work-tree", "--namespace")
COMMAND_SEPARATORS = ("|", "||", "&&", ";", "&")
PATH_SUCCESSOR = os.path.join(REPOSITORY_OF_GATES, "scripts", "kb", "path-successor.py")
QUOTE_KB = os.path.join(REPOSITORY_OF_GATES, "scripts", "kb", "quote-kb.py")
LAG_BASELINE = os.path.join(GATE_DIRECTORY, "lib-lag-baseline.py")

# 旧号 → (今天的门禁, 那一道里的哪几格；空元组表示整道或拆得说不清，只给门禁名)
OLD_GATES = {
    10: ("doc-registries", ("experiment-refs", "decision-refs", "declared-counts", "governance-refs")),
    11: ("doc-process-records", ("batch-scope",)),
    12: ("doc-text", ("prime-marks", "clock-times")),
    13: ("code-source-discipline", ("vague-names",)),
    15: ("research-build-and-replay", ("research-unit-tests",)),
    16: ("doc-decisions", ("freeze-layer-membership",)),
    19: ("doc-process-records", ("verdict-names-local-samples",)),
    20: ("doc-decisions", ()),
    21: ("doc-decisions", ("decision-items-sync",)),
    22: ("doc-decisions", ("item-ref-status",)),
    24: ("doc-decisions", ("status-redundancy",)),
    27: ("code-source-discipline", ("format-constants",)),
    28: ("doc-decisions", ("cross-decision-status",)),
    29: ("doc-decisions", ("settled-item-self-open",)),
    30: ("doc-decisions", ("entry-added", "shape")),
    31: ("doc-decisions", ("blocking-verdict",)),
    32: ("doc-registries", ("field-refs",)),
    33: ("code-source-discipline", ("mutation-tables",)),
    34: ("doc-experiments", ("index-sync",)),
    35: ("doc-decisions", ("user-verdict-owed",)),
    36: ("doc-registries", ("invariant-count-elsewhere",)),
    37: ("doc-decisions", ("decision-summary-width",)),
    38: ("doc-registries", ("field-projection",)),
    39: ("doc-registries", ("field-table-sums",)),
    40: ("doc-experiments", ("results-cited",)),
    42: ("doc-registries", ("first-txn-hooks",)),
    43: ("doc-registries", ("table-shape",)),
    44: ("doc-decisions", ("settled-ref-says-open",)),
    47: ("code-tooling", ("research-script-selftests", "research-script-selftest-coverage")),
    48: ("doc-decisions", ()),
    49: ("doc-decisions", ()),
    50: ("code-tooling", ("rules-manifest",)),
    51: ("doc-registries", ("admission-terms",)),
    52: ("doc-registries", ("segment-registry",)),
    53: ("doc-registries", ("format-const-placeholders",)),
    54: ("crash-point-replay", ()),
    55: ("qemu-device-streams", ("device-streams",)),
    56: ("doc-process-records", ("crates-adversarial-review",)),
    57: ("lkmm", ("litmus-verdicts",)),
    58: ("doc-process-records", ("implementation-premise",)),
    59: ("crates-mutation-replay", ("crates-mutation-rows",)),
    60: ("doc-decisions", ("stale-open-items",)),
    61: ("doc-decisions", ("settled-same-file",)),
    62: ("code-tooling", ("stage-owners",)),
    63: ("code-tooling", ("agent-write-scope",)),
    64: ("code-tooling", ("change-range-single-source",)),
    66: ("doc-process-records", ("abandoned-rounds",)),
    67: ("doc-registries", ("closeout-collects-open",)),
    68: ("doc-process-records", ("knowledge-sync",)),
    69: ("doc-experiments", ("evidence-in-repo",)),
    70: ("doc-registries", ("citations",)),
    72: ("doc-process-records", ("agent-def-adversarial-review",)),
    73: ("code-tooling", ("research-gate-lint",)),
    74: ("model-differential-and-scenarios", ("model-random-histories", "model-reuse-sampling", "model-rollback-sampling",
                                                       "model-allocation-record-wall", "model-unit-area-wall",
                                                       "model-unit-area-wall-admission-judged")),
    75: ("doc-experiments", ("decision-links",)),
    76: ("doc-registries", ("second-txn-hooks",)),
    77: ("harness-test-environment", ("machine-clean",)),
    78: ("doc-registries", ("paid-cited-tests",)),
    79: ("doc-registries", ("tree-table-reserve",)),
    80: ("code-source-discipline", ("absolute-assertions",)),
    81: ("doc-registries", ("audit-contradictions",)),
    82: ("code-source-discipline", ("clause-enums",)),
    84: ("doc-experiments", ("verdict-false-named",)),
    85: ("doc-experiments", ("repro-command",)),
    86: ("doc-experiments", ("experiment-orphans",)),
    87: ("research-build-and-replay", ("experiment-replay",)),
    88: ("doc-experiments", ("quoted-result-lines",)),
    89: ("doc-registries", ("closeout-row27-preconditions",)),
    90: ("doc-text", ("term-renames",)),
    91: ("doc-experiments", ("archive-past-rounds",)),
    92: ("checker-independence-and-sync", ("layout-checker-sync",)),
    93: ("code-source-discipline", ("feature-bits",)),
    94: ("checker-independence-and-sync", ("checker-implementation-disjoint",)),
    95: ("code-tooling", ("fixture-claims",)),
    96: ("code-source-discipline", ("reading-discipline",)),
    97: ("doc-registries", ("invariant-anchors",)),
    98: ("code-tooling", ("kb-registry",)),
    99: ("doc-experiments", ("multipath-registry",)),
}
# 14 号拆进了三道，单列
SPLIT_OLD_GATES = {
    14: "拆开了：「一条用例一个场景」是门禁 model-differential-and-scenarios 的 one-scenario 格，"
        "「测试文件不按里程碑起名」是门禁 code-source-discipline 的 test-file-names 格，"
        "「崩溃枚举用例住 验证包且登记」「测试文件声明模块」归门禁 crash-point-replay",
}


def existing_gate_numbers():
    numbers = set()
    for name in os.listdir(GATE_DIRECTORY):
        match = re.match(r"([0-9]+)-.*\.sh$", name)
        if match:
            numbers.add(int(match.group(1)))
    return numbers


def load_structure_check():
    specification = importlib.util.spec_from_file_location("gate_structure_check", STRUCTURE_CHECK)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


LOADED_MODULES = {}


def load_module(name, path):
    """按路径 import 一份带连字符名字的脚本（path-successor.py、quote-kb.py、lib-lag-baseline.py），同一个进程里只装一次。"""
    if name not in LOADED_MODULES:
        specification = importlib.util.spec_from_file_location(name, path)
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        LOADED_MODULES[name] = module
    return LOADED_MODULES[name]


CONCLUSIONS = {}


def path_conclusion(path):
    """path-successor.py 对这条路径（相对被判的根）的结论（Conclusion），同一个进程里同一条路径只问一次。"""
    if path not in CONCLUSIONS:
        CONCLUSIONS[path] = load_module("path_successor", PATH_SUCCESSOR).conclude(path, ".")
    return CONCLUSIONS[path]


def successor_sentence(path):
    """红行接的后继结论：path-successor.py 对这条路径的结论句；弄坏开关 successor-dropped 下交空串。"""
    if BROKEN == "successor-dropped":
        return ""
    return "：" + path_conclusion(path).sentence()


def git_history_word_indexes(words):
    """反引号里按空白切开的词中，git log / show / blame / diff 命令 `--` 之后的那几个词的下标（到命令分隔为止）。
    `$(git …)` 起头的照认；以 ; 或 ) 收尾的词算这条命令到头。弄坏开关 git-history-judged 下一个都不交。"""
    indexes = set()
    if BROKEN == "git-history-judged":
        return indexes
    state, value_pending = "outside", False   # outside → git（见到 git）→ history（子命令是那四个之一）→ after-dash（见到 --）
    for index, word in enumerate(words):
        if word in COMMAND_SEPARATORS:
            state = "outside"
            continue
        if state == "outside":
            if word.lstrip("$(") == "git":
                state, value_pending = "git", False
        elif state == "git":
            if value_pending:
                value_pending = False
            elif word in GIT_OPTIONS_WITH_VALUE:
                value_pending = True
            elif not word.startswith("-"):
                state = "history" if word.rstrip(";)") in GIT_HISTORY_SUBCOMMANDS else "outside"
        elif state == "history":
            if word == "--":
                state = "after-dash"
        else:
            indexes.add(index)
        if word.endswith((";", ")")):
            state = "outside"
    return indexes


NAME_CHARACTER_BEFORE = r"(?<![A-Za-z0-9_.\-])"
NAME_CHARACTER_AFTER = r"(?![A-Za-z0-9_\-])"


def successor_named_on_line(conclusion, path, line):
    """说搬迁本身的那一行：path-successor.py 给出了后继，而某个后继的文件名（最后一段）在这一行里另有一处
    （先把旧路径从行里去掉：后继与旧路径同名时，不因为旧路径自己就算数）。弄坏开关 move-line-judged 下一律交 False。"""
    if BROKEN == "move-line-judged" or not conclusion.successors:
        return False
    remainder = line.replace(path, " ")
    for successor, _why in conclusion.successors:
        name = os.path.basename(successor.rstrip("/"))
        if name and re.search(NAME_CHARACTER_BEFORE + re.escape(name) + NAME_CHARACTER_AFTER, remainder):
            return True
    return False


SECTION_TEXTS = {}


def without_short_names(text):
    """去掉紧跟在 C、D、E、I 编号后面的（简称）。弄坏开关 short-names-kept 下原样交回。"""
    if BROKEN == "short-names-kept":
        return text
    return SHORT_NAME_AFTER_NUMBER.sub(r"\1", text)


def section_haystacks(target):
    """③ 要比的那份文件：(原文, 去掉装饰的全文, 再去掉编号简称的全文)，同一个进程里同一份只读一次。"""
    if target not in SECTION_TEXTS:
        with open(target, encoding="utf-8", errors="replace") as handle:
            text = handle.read()
        bare = DECORATION.sub("", text)
        SECTION_TEXTS[target] = (text, bare, without_short_names(bare))
    return SECTION_TEXTS[target]


def section_found(target, section):
    """③ 的判法：小节名去掉装饰之后在那份文件的全文里找得到；或两边都去掉编号简称之后找得到；
    或那份文件是脚本、把小节名里单独的大写字母当占位之后找得到（弄坏开关 placeholders-literal 下这一条不认）。"""
    _text, bare, bare_without_short_names = section_haystacks(target)
    wanted = DECORATION.sub("", section)
    if wanted in bare:
        return True
    wanted = without_short_names(wanted)
    if wanted in bare_without_short_names:
        return True
    if not target.endswith(SCRIPT_SUFFIXES) or BROKEN == "placeholders-literal":
        return False
    literals = PLACEHOLDER_LETTER.split(wanted)
    if len(literals) < 2 or sum(len(literal) for literal in literals) < PLACEHOLDER_MINIMUM_LITERAL_CHARACTERS:
        return False
    return re.search(PLACEHOLDER_SPAN.join(re.escape(literal) for literal in literals), bare_without_short_names) is not None


def nearest_text(target, section):
    """③ 红行的出路：那份文件里与小节名最像的标题或句子，写成「：最像的是那份文件第 L 行的标题「…」…」；
    一条都够不上 NEAREST_MINIMUM_SHARE 的写「：那份文件里没有，改写那一句或删去」。弄坏开关 nearest-dropped 下交空串。"""
    if BROKEN == "nearest-dropped":
        return ""
    text = section_haystacks(target)[0]
    wanted = without_short_names(DECORATION.sub("", section))
    best = None
    for number, line in enumerate(text.split("\n") if wanted else [], 1):
        heading = HEADING_LINE.match(line)
        pieces = [("标题", heading.group(1))] if heading else [("句子", piece) for piece in SENTENCE_END.split(line)]
        for kind, piece in pieces:
            shown = piece.strip(" |>-").replace("**", "").strip()
            bare = without_short_names(DECORATION.sub("", shown))
            if len(bare) < 2:
                continue
            matcher = difflib.SequenceMatcher(None, wanted, bare, autojunk=False)
            if matcher.quick_ratio() * (len(wanted) + len(bare)) / 2 < NEAREST_MINIMUM_SHARE * len(wanted):
                continue
            share = sum(block.size for block in matcher.get_matching_blocks()) / len(wanted)
            key = (share, kind == "标题", -len(bare))
            if share >= NEAREST_MINIMUM_SHARE and (best is None or key > best[0]):
                best = (key, number, kind, shown)
    if best is None:
        return "：那份文件里没有，改写那一句或删去"
    _key, number, kind, shown = best
    if len(shown) > NEAREST_SHOWN_CHARACTERS:
        shown = shown[:NEAREST_SHOWN_CHARACTERS] + "…"
    return f"：最像的是那份文件第 {number} 行的{kind}「{shown}」，照它改小节名或改写那一句"


def is_section_key(written):
    """滞后表、保留表第二列是小节行（③ 扩到 kb，写成 `<文件>「<小节>」`）还是路径。"""
    return "「" in written


def local_gates(structure):
    """门禁目录顶层每道门禁的全名（文件名去掉 .sh）→ 它文件头格名表里的格名集合。"""
    gates = {}
    for name in sorted(os.listdir(GATE_DIRECTORY)):
        path = os.path.join(GATE_DIRECTORY, name)
        if name.endswith(".sh") and os.path.isfile(path):
            gates[name[:-len(".sh")]] = {cell for _number, cell, _title in structure.header_cells(path)}
    return gates


def upstream_stage_names():
    if not os.path.isdir(UPSTREAM_SCRIPTS):
        return set()
    return {os.path.splitext(name)[0] for name in os.listdir(UPSTREAM_SCRIPTS)
            if name.endswith((".sh", ".py")) and os.path.isfile(os.path.join(UPSTREAM_SCRIPTS, name))}


def today_name_of(number):
    if number in SPLIT_OLD_GATES:
        return SPLIT_OLD_GATES[number]
    gate, cells = OLD_GATES[number]
    if cells:
        return f"门禁 {gate} 的 {'、'.join(cells)} 格"
    return f"门禁 {gate}"


def mapping_problems(gates):
    problems = []
    for number, (gate, cells) in sorted(OLD_GATES.items()):
        if gate not in gates:
            problems.append(f"{os.path.basename(__file__)}: OLD_GATES 里 {number} 号指的门禁 {gate} 不在门禁目录里")
            continue
        for cell in cells:
            if cell not in gates[gate]:
                problems.append(f"{os.path.basename(__file__)}: OLD_GATES 里 {number} 号指的 {gate} 的 {cell} 格不在那道门禁的格名表里")
    return problems


def ignored_by_git(path):
    result = subprocess.run(["git", "check-ignore", "-q", "--no-index", path],
                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return result.returncode == 0


def bases_for(carrier, changed_directories):
    return list(BASE_DIRECTORIES) + [os.path.dirname(carrier)] + list(changed_directories)


def resolve(path, carrier, changed_directories=()):
    bare = path.rstrip("/")
    for base in bases_for(carrier, changed_directories):
        candidate = os.path.normpath(os.path.join(base, bare))
        if os.path.exists(candidate):
            return candidate
    return None


FILE_EXTENSION = re.compile(r"\.(md|sh|py|rs|tsv|toml|json|txt|out|lock)$")


def looks_like_repo_path(path, carrier, changed_directories):
    """第一段在解析基准下现存，或末段带常见的文件扩展名，才当仓内路径：`origin/master`、`text/plain` 两样都不满足。"""
    first = path.split("/", 1)[0]
    if any(os.path.exists(os.path.join(base, first)) for base in bases_for(carrier, changed_directories)):
        return True
    return bool(FILE_EXTENSION.search(path.rstrip("/")))


OUTSIDE_TOKEN = "outside"


def path_token(token):
    """反引号里的一个词若形如一条仓内路径，返回去掉 `:行号`、`@标题`、`~正则` 之后的路径；
    中间不带 `/` 的返回 None（不是路径，不计数）；起头在仓外或带占位的返回 OUTSIDE_TOKEN（计进跳过）。"""
    path = re.split(r"[:@~]", token, maxsplit=1)[0]
    if "/" not in path.rstrip("/"):
        return None
    if path.startswith(("../", "/", "~", "refs/", "http")):
        return OUTSIDE_TOKEN
    if any(mark in path for mark in "<*{"):
        return OUTSIDE_TOKEN
    return path


def is_old_gate_path(path):
    """路径指到去编号之前的门禁文件（gate.d/<旧号>-…）：归 ⑤，② 不判。54 号与别的旧号一样（它的全名早已不是 54-layer0-replay）。"""
    match = OLD_GATE_PATH.search(path)
    return match is not None and int(match.group(1)) in OLD_GATES.keys() | SPLIT_OLD_GATES.keys()


# 带目录的文件路径与「日期-」起头的记录文件名整个遮掉再找号：
# `records/2026-09-28-门禁59号提速与双机分片.md` 这种文件名里的「门禁59号」不是在称呼门禁。
PATH_LIKE = re.compile(r"[^\s`「」（）()，。；：、,;]*(?:/[^\s`「」（）()，。；：、,;]*\.(?:md|sh|py|tsv|txt|out|rs)"
                       r"|[0-9]{4}-[0-9]{2}-[0-9]{2}-[^\s`「」（）()，。；：、,;]*\.md)")


def without_paths(line):
    return PATH_LIKE.sub(lambda match: " " * len(match.group(0)), line)


def gate_numbers_in(line):
    """一行里「NN 号」的每个号：[(起点, 号)]，前面紧挨「第」「月」「日」的不算，文件路径里的不算。"""
    found = []
    line = without_paths(line)
    for match in GATE_NUMBER.finditer(line):
        if NOT_A_GATE_BEFORE.search(line[:match.start()]):
            continue
        offset = match.start(1)
        for piece in re.finditer(r"[0-9]{2}", match.group(1)):
            found.append((offset + piece.start(), int(piece.group(0))))
    return found


def current_text_lines(carrier):
    """现状文字：跳过「## 历史版本」一节、代码围栏里的行；变更史整份跳过。返回 [(行号, 行)]。"""
    if os.path.basename(carrier) in HISTORY_FILES and BROKEN != "history-scanned":
        return []
    with open(carrier, encoding="utf-8", errors="replace") as handle:
        lines = handle.read().split("\n")
    kept, in_history, in_fence = [], False, False
    for number, line in enumerate(lines, 1):
        if FENCE_LINE.match(line):
            in_fence = not in_fence and BROKEN != "fences-scanned"
            continue
        if in_fence:
            continue   # 围栏里是整行抄的产物与命令，不是现状文字：旧路径照抄进去不算称呼门禁
        if LEVEL_TWO_HEADING.match(line):
            in_history = bool(HISTORY_HEADING.match(line)) and BROKEN != "history-scanned"
        if not in_history:
            kept.append((number, line))
    return kept


def judge_governance(carriers, gates_on_disk, kept_keys, problems, unjudged, counts, skipped, dead_references):
    """① ② ③：只扫治理文档。② 的两种自动跳过与保留表与 kb 那一半同一套：skipped 另记 git_history、move_line、kept；
    dead_references 收指不到、又不是说搬迁那一行的（载体, 路径）（保留表核「该删」用）。"""
    for carrier in carriers:
        with open(carrier, encoding="utf-8") as handle:
            lines = handle.read().split("\n")
        for number, line in enumerate(lines, 1):
            for _start, gate in gate_numbers_in(line):
                if gate in OLD_GATES or gate in SPLIT_OLD_GATES:
                    continue   # 旧号归 ⑤
                counts["gate"] += 1
                if gate not in gates_on_disk:
                    problems.append(f"{carrier}:{number}: 「{gate:02d} 号」在门禁目录里没有 {gate:02d}-*.sh")
            for match in BACKTICK.finditer(line):
                words = match.group(1).split()
                # 命令里 `cd 目录 && …` 之后的相对路径按那个目录解析
                changed_directories = [words[i + 1] for i in range(len(words) - 1) if words[i] == "cd"]
                history_indexes = git_history_word_indexes(words)
                for index, word in enumerate(words):
                    path = path_token(word)
                    if path is None:
                        continue
                    if index in history_indexes:
                        skipped["git_history"] += 1
                        continue
                    if path is OUTSIDE_TOKEN:
                        skipped["outside"] += 1
                        continue
                    if is_old_gate_path(path):
                        continue   # 旧门禁文件归 ⑤
                    if not REPO_PATH.match(path):
                        # 非 ASCII 的记号：指得到就算判过；指不到的与 `NN-简称.md` 这类占位分不开，列进没判的
                        if resolve(path, carrier, changed_directories) is not None:
                            counts["path"] += 1
                        else:
                            unjudged.append(f"{carrier}:{number}: `{word}` 不全是路径字符（非 ASCII、引号、`$` 这类）、指不到，分不清是占位还是悬空，路径没判")
                        continue
                    if not looks_like_repo_path(path, carrier, changed_directories):
                        skipped["not_repo_path"] += 1
                        continue
                    if ignored_by_git(path):
                        skipped["ignored"] += 1
                        continue
                    resolved = resolve(path, carrier, changed_directories)
                    if resolved is None and successor_named_on_line(path_conclusion(path), path, line):
                        skipped["move_line"] += 1
                        continue
                    counts["path"] += 1
                    if resolved is not None:
                        continue
                    key = (carrier, path.rstrip("/"))
                    dead_references.add(key)
                    if key in kept_keys:
                        skipped["kept"] += 1
                        continue
                    problems.append(f"{carrier}:{number}: `{match.group(1)}` 里的 {path} 在仓里不存在{successor_sentence(path)}")
            for match in FILE_WITH_SECTIONS.finditer(line):
                target = resolve(match.group(1), carrier)
                if target is None or not os.path.isfile(target):
                    unjudged.append(f"{carrier}:{number}: {match.group(1)}{match.group(2)} 的文件在四个解析基准下都找不到，小节没判")
                    continue
                for section in re.findall(r"「([^」]+)」", match.group(2)):
                    counts["section"] += 1
                    if not section_found(target, section):
                        problems.append(f"{carrier}:{number}: {match.group(1)}「{section}」在那份文件里一处都找不到{nearest_text(target, section)}")


def judge_current_text(carriers, gates, upstream, problems, counts):
    """④ ⑤：扫现状文字。"""
    known_names = set(gates) | upstream
    for carrier in carriers:
        for number, line in current_text_lines(carrier):
            if BROKEN != "old-numbers-ignored":
                reported = set()
                spans = [(start, gate) for start, gate in gate_numbers_in(line)]
                spans += [(match.start(1), int(match.group(1))) for match in GATE_WORD_NUMBER.finditer(without_paths(line))]
                spans += [(match.start(1), int(match.group(1))) for match in OLD_GATE_PATH.finditer(line)]
                for start, gate in sorted(spans):
                    if start in reported or (gate not in OLD_GATES and gate not in SPLIT_OLD_GATES):
                        continue
                    reported.add(start)
                    counts["old"] += 1
                    problems.append(f"{carrier}:{number}: 用旧编号称呼门禁（…{line[max(0, start - 12):start + 16].strip()}…）：{gate} 号今天是{today_name_of(gate)}")
            if BROKEN == "names-ignored":
                continue
            for match in GATE_BY_NAME.finditer(line):
                name = match.group(1)
                if re.match(r"[0-9]{2}-", name) and int(name[:2]) in OLD_GATES.keys() | SPLIT_OLD_GATES.keys():
                    continue   # 带旧号的文件名，⑤ 不判裸名，这里也不重复报
                counts["name"] += 1
                if name not in known_names:
                    problems.append(f"{carrier}:{number}: 「门禁 {name}」不是门禁目录里的门禁（也不是上游共享门禁的脚本）")
            for match in CELL_OF_GATE.finditer(line):
                gate, cell = match.group(1), match.group(2)
                if gate not in gates:
                    continue
                counts["cell"] += 1
                if cell not in gates[gate]:
                    problems.append(f"{carrier}:{number}: 「{gate} 的 {cell} 格」：{gate} 的格名表里没有 {cell}")
            for match in BACKTICK.finditer(line):
                words = match.group(1).split()
                gate = None
                for index, word in enumerate(words):
                    found = re.search(r"gate\.d/([^/\s]+)\.sh$", word)
                    if found:
                        gate = found.group(1)
                    elif word in ("--check",) and gate in gates and index + 1 < len(words):
                        for cell in words[index + 1].split(","):
                            if not cell or any(mark in cell for mark in "<*{…"):
                                continue   # 占位（`--check <格名>`）不判
                            counts["cell"] += 1
                            if cell not in gates[gate]:
                                problems.append(f"{carrier}:{number}: `{match.group(1)}`：{gate} 的格名表里没有 {cell}")


def kb_carriers():
    """② 扩到 kb 的载体：登记的 kb 目录下的中文 .md（译文与变更史不算），相对被判的根、排序。弄坏开关 kb-unscanned 下一份都不交。"""
    if BROKEN == "kb-unscanned":
        return []
    found = set()
    for directory in load_module("quote_kb", QUOTE_KB).kb_candidate_roots(os.getcwd()):
        for path in glob.glob(os.path.join(directory, "**", "*.md"), recursive=True):
            name = os.path.basename(path)
            if not name.endswith(TRANSLATION_SUFFIXES) and name not in HISTORY_FILES and os.path.isfile(path):
                found.add(os.path.normpath(path))
    return sorted(found)


def kb_path_kind(word):
    """kb 反引号里的一个词归哪一类：(类, 路径)。类是 None（不是路径）、outside、old-gate、relative、root 之一；root 的还要过 .gitignore 那一判。
    比治理文档的 ② 多去掉两样尾巴：`#锚` 与命令里进程替换收尾的 `)`（`<(sed … 文件)`）；`?`、`[` 这类通配与以 - 收尾的名字前缀（`…-s1-`）算占位。"""
    path = path_token(word)
    if path is None:
        return None, None
    if path is OUTSIDE_TOKEN:
        return "outside", None
    path = path.split("#", 1)[0]
    while path.endswith(")") and "(" not in path:
        path = path[:-1]
    if any(mark in path for mark in "?[") or path.rstrip("/").endswith("-"):
        return "outside", None
    if is_old_gate_path(path):
        return "old-gate", path   # 旧门禁文件归 ⑤
    if path.split("/", 1)[0] not in ROOT_DIRECTORIES:
        return "relative", path
    return "root", path


def ignored_among(paths):
    """一次 git check-ignore --stdin 判一批路径，交被 .gitignore 挡着的那几条（逐条起 git 太慢：真仓 kb 里六千多处）。"""
    if not paths:
        return set()
    result = subprocess.run(["git", "check-ignore", "--no-index", "--stdin", "-z"], input="\0".join(paths) + "\0",
                            capture_output=True, text=True)
    return {item for item in result.stdout.split("\0") if item} if result.returncode in (0, 1) else set()


def archived_in_git(path):
    """research/ 下这条路径进没进过 git：git log --all -1 -- <路径> 有输出。"""
    result = subprocess.run(["git", "--literal-pathspecs", "log", "--all", "-1", "--format=%h", "--", path],
                            capture_output=True, text=True)
    return result.returncode == 0 and bool(result.stdout.strip())


def stored_forms(path):
    """kb 点名的路径在树里、历史里可能的存放名：research/results/ 下的 <名>.out 另认 <名>.out.gz，别的只认原样。弄坏开关 gz-storage-ignored 下只认原样。"""
    if BROKEN != "gz-storage-ignored" and path.startswith(PRODUCT_DIRECTORY) and path.endswith(PRODUCT_SUFFIX):
        return [path, path + COMPRESSED_SUFFIX]
    return [path]


def judge_kb_paths(carriers, lagged_keys, kept_keys, problems, kb_counts, dead_references):
    """② 扩到 kb：kb_counts 累计判了、放行、滞后、保留与几类跳过；dead_references 收（载体, 路径）——指不到、
    又不是 research/ 下进过 git 的、也不是说搬迁那一行的那几处（滞后表与保留表核「该删」用）。"""
    found = []   # (载体, 行号, 整行, 反引号原文, 路径)：第一段是仓根目录的
    for carrier in carriers:
        for number, line in current_text_lines(carrier):
            for match in BACKTICK.finditer(line):
                words = match.group(1).split()
                history_indexes = git_history_word_indexes(words)
                for index, word in enumerate(words):
                    kind, path = kb_path_kind(word)
                    if kind is None or kind == "old-gate":
                        continue
                    if index in history_indexes:
                        kb_counts["git_history"] += 1
                        continue
                    if kind != "root":
                        kb_counts[kind] += 1
                        continue
                    found.append((carrier, number, line, match.group(1), path))
    ignored = ignored_among(sorted({path for _carrier, _number, _line, _quoted, path in found}))
    for carrier, number, line, quoted, path in found:
        if path in ignored:
            kb_counts["ignored"] += 1
            continue
        forms = stored_forms(path)
        if any(os.path.exists(form) for form in forms):
            kb_counts["judged"] += 1
            continue
        if path.split("/", 1)[0] == ARCHIVED_DIRECTORY and BROKEN != "research-not-passed" and any(archived_in_git(form) for form in forms):
            kb_counts["judged"] += 1
            kb_counts["archived"] += 1
            continue
        if successor_named_on_line(path_conclusion(path), path, line):
            kb_counts["move_line"] += 1
            continue
        kb_counts["judged"] += 1
        key = (carrier, path.rstrip("/"))
        dead_references.add(key)
        if key in kept_keys:
            kb_counts["kept"] += 1
            continue
        if key in lagged_keys:
            kb_counts["lagged"] += 1
            continue
        problems.append(f"{carrier}:{number}: `{quoted}` 里的 {path} 在仓里不存在{successor_sentence(path)}")


def judge_kb_sections(carriers, kept_keys, lagged_keys, problems, section_counts, dead_sections):
    """③ 扩到 kb：section_counts 累计判了、滞后、保留与文件找不到几处；dead_sections 收（载体, `<文件>「<小节>」`）——
    找不到的那几处（小节滞后表与保留表的小节行核「该删」用）。弄坏开关 kb-sections-unscanned 下一处都不判。"""
    if BROKEN == "kb-sections-unscanned":
        return
    for carrier in carriers:
        for number, line in current_text_lines(carrier):
            for match in FILE_WITH_SECTIONS.finditer(line):
                target = resolve(match.group(1), carrier)
                if target is None or not os.path.isfile(target):
                    section_counts["unresolved"] += 1
                    continue
                for section in re.findall(r"「([^」]+)」", match.group(2)):
                    section_counts["judged"] += 1
                    if section_found(target, section):
                        continue
                    key = (carrier, f"{match.group(1)}「{section}」")
                    dead_sections.add(key)
                    if key in kept_keys:
                        section_counts["kept"] += 1
                        continue
                    if key in lagged_keys:
                        section_counts["lagged"] += 1
                        continue
                    problems.append(f"{carrier}:{number}: {match.group(1)}「{section}」在那份文件里一处都找不到{nearest_text(target, section)}")


def read_pair_table(table, reason_rule, minimum_reason_characters, problems):
    """滞后表与保留表同一种写法：{(载体, 路径): 表的行号}；列数不对、理由不合 reason_rule、同一条登记两次的行报进 problems、不认。
    minimum_reason_characters 为 0 时理由长短不判（保留表的弄坏开关 kept-reds-unjudged 用）。"""
    rows = {}
    with open(table, encoding="utf-8", errors="replace") as handle:
        content = load_module("lib_lag_baseline", LAG_BASELINE).content_rows(handle.read())
    for row_number, columns in content:
        if len(columns) != 3:
            problems.append(f"{table}:{row_number}: 有 {len(columns)} 列，要三列：<载体路径><TAB><不在的路径><TAB># <理由>")
            continue
        carrier, path, reason = (column.strip() for column in columns)
        if minimum_reason_characters and (not reason.startswith("#") or len(reason.lstrip("#").strip()) < minimum_reason_characters):
            problems.append(f"{table}:{row_number}: 第三列要以 # 起头、{reason_rule}（至少 {minimum_reason_characters} 个字）")
            continue
        key = (carrier, path.rstrip("/"))
        if key in rows:
            problems.append(f"{table}:{row_number}: 与第 {rows[key]} 行登记的是同一条（载体与路径都相同）")
            continue
        rows[key] = row_number
    return rows


def read_lag_table(problems):
    """滞后表此刻的行：{(载体, 路径): 表的行号}；写得不对的行报进 problems、不认。弄坏开关 lag-unread 下不读、交 None。"""
    if BROKEN == "lag-unread" or not os.path.isfile(LAG_TABLE):
        return None
    return read_pair_table(LAG_TABLE, "写为什么还没改", MINIMUM_LAG_REASON_CHARACTERS, problems)


def read_section_lag_table(problems):
    """小节滞后表此刻的行：{(载体, `<文件>「<小节>」`): 表的行号}；写得不对的行报进 problems、不认。弄坏开关 section-lag-unread 下不读、交 None。"""
    if BROKEN == "section-lag-unread" or not os.path.isfile(SECTION_LAG_TABLE):
        return None
    return read_pair_table(SECTION_LAG_TABLE, "写为什么还没改", MINIMUM_LAG_REASON_CHARACTERS, problems)


def read_kept_table(problems):
    """保留表此刻的行：{(载体, 路径): 表的行号}；写得不对的行报进 problems、不认。弄坏开关 kept-unread 下不读、交 None。"""
    if BROKEN == "kept-unread" or not os.path.isfile(KEPT_TABLE):
        return None
    minimum = 0 if BROKEN == "kept-reds-unjudged" else MINIMUM_KEPT_REASON_CHARACTERS
    return read_pair_table(KEPT_TABLE, f"写它属于哪种（{KEPT_KINDS}）", minimum, problems)


def judge_kept_table(rows, lag_tables, dead_references, dead_sections, governance_carriers, problems):
    """保留表该删的行（路径在了、载体不再引它；小节行：载体的现状文字里已经没有这一处找不到的）与滞后表、小节滞后表也登了的行报进 problems；
    lag_tables 是 [(表的路径, 那张表此刻的行或 None)]。载体是治理文档的路径行按 ② 的四个解析基准核「此刻在了」。交一句表的情况。"""
    if rows is None:
        return "弄坏开关 kept-unread：没读保留表" if BROKEN == "kept-unread" else "没有保留表"
    for (carrier, path), row_number in sorted(rows.items(), key=lambda item: item[1]):
        for lag_table, lag_rows in lag_tables:
            if lag_rows and (carrier, path) in lag_rows:
                problems.append(f"{KEPT_TABLE}:{row_number}: 与滞后表 {lag_table} 第 {lag_rows[(carrier, path)]} 行登记的是同一对（{carrier} 里的 {path}）："
                                "两张表只登一张，照规矩该留的留在保留表、删滞后表那一行，要改的删这一行")
        if BROKEN == "kept-reds-unjudged":
            continue
        if is_section_key(path):
            if (carrier, path) not in dead_sections:
                problems.append(f"{KEPT_TABLE}:{row_number}: {carrier} 的现状文字里已经没有找不到的 {path}（小节找得到了，或那一句改了、删了、挪进历史节，"
                                "或那份载体不在、不在 kb 射程里），这一行该删")
            continue
        exists = resolve(path, carrier) is not None if carrier in governance_carriers else os.path.exists(path)
        if exists:
            problems.append(f"{KEPT_TABLE}:{row_number}: {carrier} 里的 {path} 此刻在了：建好了或回来了，删这一行")
        elif (carrier, path) not in dead_references:
            problems.append(f"{KEPT_TABLE}:{row_number}: {carrier} 的现状文字里已经不引 {path}（或那份载体不在、不在 kb 射程里、引它的那一行归了自动跳过），这一行该删")
    return "可以增长，不与基准比"


def judge_lag_table(table, rows, dead_keys, base, problems):
    """滞后表（table 是 LAG_TABLE）或小节滞后表（SECTION_LAG_TABLE）该删的行与比基准涨了的行报进 problems；交一句与基准比的结局。
    dead_keys 是此刻还指不到的（载体, 第二列）；路径行另核「此刻在了」，小节行只看还找不找得到。"""
    sections = table == SECTION_LAG_TABLE
    if rows is None:
        if sections:
            return "弄坏开关 section-lag-unread：没读小节滞后表" if BROKEN == "section-lag-unread" else "没有小节滞后表"
        return "弄坏开关 lag-unread：没读滞后表" if BROKEN == "lag-unread" else "没有滞后表"
    for (carrier, path), row_number in sorted(rows.items(), key=lambda item: item[1]):
        if not sections and os.path.exists(path):
            problems.append(f"{table}:{row_number}: {carrier} 里的 {path} 此刻在了，这一行该删（只缩不涨，删了不用补）")
        elif sections and (carrier, path) not in dead_keys:
            problems.append(f"{table}:{row_number}: {carrier} 的现状文字里已经没有找不到的 {path}（小节找得到了，或那一句改了、删了、挪进历史节，"
                            "或那份载体不在、不在 kb 射程里），这一行该删（只缩不涨，删了不用补）")
        elif (carrier, path) not in dead_keys:
            problems.append(f"{table}:{row_number}: {carrier} 的现状文字里已经不引 {path}（或那份载体不在、不在 kb 射程里），这一行该删（只缩不涨，删了不用补）")
    if BROKEN == "lag-growth-unjudged":
        return "弄坏开关 lag-growth-unjudged：没与 gate 基准比"
    if not base:
        return "没给 gate 基准（被判的根不在 git 仓里），只缩不涨没比"
    lag_baseline = load_module("lib_lag_baseline", LAG_BASELINE)
    base_rows, problem = lag_baseline.baseline_rows(base, table, cwd=os.getcwd())
    if problem is not None:
        problems.append(f"{table}: 与 gate 基准比不了：{problem.sentence()}")
        return "与 gate 基准比不了"
    if base_rows is None:
        return f"gate 基准 {base} 里还没有{'小节滞后表' if sections else '滞后表'}（初次登记），不比"
    base_keys = {(columns[0].strip(), columns[1].strip().rstrip("/")) for _number, columns in base_rows if len(columns) >= 2}
    grown = sorted((row_number, key) for key, row_number in rows.items() if key not in base_keys)
    for row_number, (carrier, path) in grown:
        problems.append(f"{table}:{row_number}: 比 gate 基准 {base} 那一版多这一行（{carrier} 里的 {path}）：只缩不涨，"
                        + ("新写的找不到的小节当场照红行的出路改，不往小节滞后表里加" if sections
                           else "新写的不在的路径当场改成后继或改写那一句，不往滞后表里加"))
    return f"与 gate 基准 {base} 那一版比没涨（基准 {len(base_rows)} 行，此刻 {len(rows)} 行）" if not grown else f"比 gate 基准 {base} 那一版多 {len(grown)} 行"


def main():
    arguments = sys.argv[1:]
    base = ""
    if arguments[:1] == ["--base"] and len(arguments) == 2:
        base = arguments[1]
    elif arguments:
        print(f"  ✗ 认不出的参数 {' '.join(arguments)}")
        print("     → 怎么办：只认可不给的 --base <提交>（滞后表与之比只缩不涨的 gate 基准）；在仓库根上跑。")
        return 2
    carriers = sorted({found for pattern in CARRIER_PATTERNS for found in glob.glob(pattern)})
    if not carriers:
        print("  ✗ 一份治理文档都没扫到（CLAUDE.md、.claude/main-agent.md、.claude/agent-common.md、agents、rules、skills）")
        print("     → 怎么办：在仓库根上跑；治理文档搬了家的话，改本文件的 CARRIER_PATTERNS。")
        return 3
    structure = load_structure_check()
    gates = local_gates(structure)
    problems, unjudged = [], []
    counts = {"gate": 0, "path": 0, "section": 0, "name": 0, "cell": 0, "old": 0}
    skipped = {"ignored": 0, "not_repo_path": 0, "outside": 0, "git_history": 0, "move_line": 0, "kept": 0}
    problems.extend(mapping_problems(gates))
    lag_rows = read_lag_table(problems)
    section_lag_rows = read_section_lag_table(problems)
    kept_rows = read_kept_table(problems)
    kept_keys = set(kept_rows or ())
    dead_references, dead_sections = set(), set()
    judge_governance(carriers, existing_gate_numbers(), kept_keys, problems, unjudged, counts, skipped, dead_references)
    readmes = set() if BROKEN == "readme-skipped" else {found for pattern in README_PATTERNS for found in glob.glob(pattern)}
    current_carriers = sorted(path for path in set(carriers) | set(glob.glob(KB_PATTERN, recursive=True)) | readmes
                              if BROKEN == "history-scanned" or os.path.basename(path) not in HISTORY_FILES)
    judge_current_text(current_carriers, gates, upstream_stage_names(), problems, counts)
    kb_counts = {"judged": 0, "archived": 0, "lagged": 0, "kept": 0, "relative": 0, "ignored": 0, "outside": 0,
                 "git_history": 0, "move_line": 0}
    section_counts = {"judged": 0, "lagged": 0, "kept": 0, "unresolved": 0}
    kb_pages = kb_carriers()
    judge_kb_paths(kb_pages, set(lag_rows or ()), kept_keys, problems, kb_counts, dead_references)
    judge_kb_sections(kb_pages, kept_keys, set(section_lag_rows or ()), problems, section_counts, dead_sections)
    lag_note = judge_lag_table(LAG_TABLE, lag_rows, dead_references, base, problems)
    section_lag_note = judge_lag_table(SECTION_LAG_TABLE, section_lag_rows, dead_sections, base, problems)
    kept_note = judge_kept_table(kept_rows, [(LAG_TABLE, lag_rows), (SECTION_LAG_TABLE, section_lag_rows)],
                                 dead_references, dead_sections, set(carriers), problems)
    for problem in problems:
        print(f"  ✗ {problem}")   # gate-lint:detail
    if problems:
        print("     → 怎么办：门禁一律按门禁目录顶层的文件名去掉 .sh 称呼（「门禁 doc-registries」「门禁 doc-registries 的 governance-refs 格」）；"
              "旧编号照上面每一行给的今天的名字改（对照表在 records/2026-09-28-门禁59号提速与双机分片.md「旧编号与现在的门禁」），"
              "格名用 bash .claude/gate.d/<门禁>.sh --list 看；门禁号改成现存的号或共享 gate.sh 里那一道的名字；"
              "路径改成现存的，已归档的写裸文件名；照规矩该留旧路径的（将来落点、句子说它不在、抄下来的命令或输出、仓外副本里的路径），"
              f"登进 {KEPT_TABLE}；小节照红行冒号后面给的那份文件里最像的标题或句子改，那份文件里没有的改写那一句或删去，"
              f"小节滞后表 {SECTION_LAG_TABLE} 只缩不涨、不往里加，照规矩该留的小节（句子本身说那一句已改掉、抄下来的当时的原文）登进 {KEPT_TABLE}，第二列写 <文件>「<小节>」。")
    for item in unjudged:
        print(f"  没判的：{item}")
    print(f"  扫 {len(carriers)} 份治理文档：门禁号 {counts['gate']} 处、路径 {counts['path']} 处、小节 {counts['section']} 处；没判 {len(unjudged)} 处；"
          f"跳过 {skipped['ignored'] + skipped['not_repo_path'] + skipped['outside']} 处（被 .gitignore 挡着 {skipped['ignored']}、不像仓内路径 {skipped['not_repo_path']}、仓外或带占位 {skipped['outside']}）；"
          f"git 历史命令里 -- 之后 {skipped['git_history']} 处、说搬迁的行 {skipped['move_line']} 处、保留 {skipped['kept']} 处")
    print(f"  扫 {len(current_carriers)} 份现状文字（治理文档、kb 与仓根 README.md、README.en.md、README.ja.md，历史版本节与变更史除外）：门禁全名 {counts['name']} 处、格名 {counts['cell']} 处、旧编号 {counts['old']} 处")
    print(f"  kb 路径：扫 {len(kb_pages)} 份 kb 中文页，判了 {kb_counts['judged']} 处（research/ 进过 git 放行 {kb_counts['archived']} 处、"
          f"滞后 {kb_counts['lagged']} 处、保留 {kb_counts['kept']} 处）；"
          f"跳过相对简写 {kb_counts['relative']} 处、被 .gitignore 挡着 {kb_counts['ignored']} 处、仓外或带占位 {kb_counts['outside']} 处、"
          f"git 历史命令里 -- 之后 {kb_counts['git_history']} 处、说搬迁的行 {kb_counts['move_line']} 处；"
          f"滞后表 {len(lag_rows or ())} 行（{lag_note}）；保留表 {len(kept_rows or ())} 行（{kept_note}）")
    print(f"  kb 小节：扫 {len(kb_pages)} 份 kb 中文页，判了 {section_counts['judged']} 处（滞后 {section_counts['lagged']} 处、保留 {section_counts['kept']} 处）；"
          f"文件找不到没判 {section_counts['unresolved']} 处；小节滞后表 {len(section_lag_rows or ())} 行（{section_lag_note}）")
    checked_total = 0 if BROKEN == "count-zero" else (sum(counts[kind] for kind in ("gate", "path", "section", "name", "cell"))
                                                      + kb_counts["judged"] + section_counts["judged"])
    if BROKEN != "count-line-missing":
        print(f"  共查 {checked_total} 项（门禁号 {counts['gate']} + 路径 {counts['path']} + 小节 {counts['section']} + 门禁全名 {counts['name']} + 格名 {counts['cell']}"
              f" + kb 路径 {kb_counts['judged']} + kb 小节 {section_counts['judged']}）")
    return 1 if problems else 0


if __name__ == "__main__":
    try:
        exit_code = main()
    except Exception:
        import traceback
        traceback.print_exc()
        print("  ✗ lib-governance-refs.py 自己出错（上面是 traceback）——这一段没判完")
        print("     → 怎么办：按 traceback 修本脚本或那份读不了的文件；没判完就是没判，不许当成判过了。")
        exit_code = 4
    sys.exit(exit_code)
