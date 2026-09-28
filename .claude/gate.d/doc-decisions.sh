#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: 决策文档与变更史（十七格：kb 形状；分项引用状态；状态别说两遍；说未定其实已定；已定分项自称未定；引用写已定紧跟说没定；未定项被别处定了；定了新东西没回看同文件未定项；冻结层归属登记表；决策分项清单与正文同步；未定项判过改不改新池新建文件的字节；动了用户定案的条款记了未还的账；决策索引结论列的宽度；同一结构的写频率只有一个量级；决策正文改了要留变更史条目；变更史的形状；变更史各节现状与索引表同步）
# gate-category: 文档类
# gate-similar: history-ordinal.sh 它也读变更史的条目标题、也只判这一轮新增的，但判的是「（其 N）」撞号，住在上游副本里由上游门禁管，项目本地改不了它
# gate-similar: doc-registries.sh 它从欠账表、收口表与总审核出发判表本身的形状与账的去向，判字段与布局登记表（字节表「指向」列点名的分项存不存在、字段投影与加和、认购表求和不超预留），它的 decision-refs 那一格判 kb 全文引用的 D 号有定义、不看分项号与状态；这里的 user-verdict-owed 那一格方向相反，从决策正文里的「待用户复核」出发去欠账表找一笔未还的账，item-ref-status 那一格判引用写的已定 / 未定与正文分项表相符，判的对象都是决策正文、索引页、冻结层归属登记表与变更史
# gate-similar: code-source-discipline.sh 它的 clause-enums 格也读 .claude/gate.d/ 下一张登记表、去决策分项里找逐字的串，但比对的另一边是代码里的枚举；这里格「同一结构的写频率只有一个量级」两边都在决策正文里（句子写的频率与登记的量级），不碰代码
#
# 决策文档与变更史：kb 里同一件事不许有两种写法，决策与分项的状态在哪一处说都要说得一样，
# 决策索引页与它的投影、决策相关的登记表都要与决策正文对得上；决策正文改了要在变更史里留条目，变更史自己的形状与各节现状也要对。
# 原是两道阶段（「kb 形状与决策文档」十四格、「变更史的条目与形状」三格，下面分别叫决策文档组、变更史组），2026-09-28 按门禁收缩判决
# research/prompts/gate-shrink-r1-main-verification.md「采纳的改法」第 2 条合成这一道；每一格的判据、出路、成功行报的数与射程照原样留在那一格的函数上方与函数里。
# 格名表（--list 打它，--check <格名>[,<格名>…] 只跑点名的格，不给就十七格按下面的次序全跑，格名写错退 2）：
# gate-cell: kb-shape kb 形状：分项只许一种叫法、kb 文件不链回自己、规则路径带 .claude/、决策标题与分项两节、索引页分项计数与正文相符
# gate-cell: item-ref-status 每一处「D<n>（简称） 已定项 k / 未定项 k」写的状态与正文分项表相符
# gate-cell: status-redundancy 状态别说两遍：小节标题不带「—— 已定」、条目不同时写破折号与「状态：」、分项索引行挂在对的节
# gate-cell: cross-decision-status 紧贴「D<n>（简称）」写「未定」，而那条决策的状态行不是待定
# gate-cell: settled-item-self-open 已定项区段里说它自己、或同一条决策里另一条已定分项还没定
# gate-cell: settled-ref-says-open 引用写着「已定项 k」、归属到的那一条也已定，紧跟着却说它没定
# gate-cell: stale-open-items 未定项点名的决策，状态行在这条未定项最近一次复核之后又变过
# gate-cell: settled-same-file 这次在一份决策里新增了「已定」小节，同文件开着的未定项与索引页的待议节一个字没动
# gate-cell: freeze-layer-membership 冻结层归属登记表：每层每个结构都有行、写「退出」的必是派生态、依据指得到已定分项
# gate-cell: decision-items-sync decisions.md 的分项清单生成块与索引表「状态」列，与生成器的输出逐字相同
# gate-cell: blocking-verdict 每条未定项都写了「改新池新建文件的字节」与「动不动格式」两把尺的判定
# gate-cell: user-verdict-owed 决策正文里标着「待用户复核」的，checks-owed.md 里要有一笔未还的账点名那条决策
# gate-cell: decision-summary-width 决策索引表每一行的「结论」列不超过 decisions.md 写的字数上限
# gate-cell: write-frequency 决策正文里落在登记结构上的频率断言，不与 write-frequency.tsv 登记的量级说反话
# gate-cell: entry-added 决策正文改了，变更史里要有新增条目
# gate-cell: shape 变更史的形状：日期与序号分属两层、条目住在自己的节、同节日期不重复
# gate-cell: status-sync 变更史各节顶上的现状与索引表同步
#
#   格名（--check 用）         格（原中文名）                   判什么
#   kb-shape                   kb 形状                          1 未定项 / 已定项只许一种叫法；3 kb 文件不许链回自己；4 上游规则路径带 .claude/；
#                                                               5 决策标题连号带状态、分项两节不串味不重号、标题未定条数与列表相符；7 索引页状态列的分项计数与正文相符
#   item-ref-status            分项引用状态                     每一处「D<n>（简称） 已定项 k / 未定项 k」写的状态与正文分项表相符（归属规则在 lib-item-ref-status.py）
#   status-redundancy          状态别说两遍                     小节标题不带「—— 已定」、条目不同时写破折号与「状态：」、分项索引行的「状态：」与所在节一致
#   cross-decision-status      说未定其实已定                   紧贴「D<n>（简称）」写「未定」，而那条决策的状态行不是待定
#   settled-item-self-open     已定分项自称未定                 已定项区段里说它自己、或同一条决策里另一条已定分项还没定
#   settled-ref-says-open      引用写已定紧跟说没定             引用写着「已定项 k」、归属到的那一条也已定，紧跟着却说它没定
#   stale-open-items           未定项被别处定了                 未定项点名的决策，状态行在这条未定项最近一次复核之后又变过（看 git 历史）
#   settled-same-file          定了新东西没回看同文件未定项     这次改动在一份决策里新增了「已定」小节，同文件还开着的未定项与索引页的待议节一个字没动（看 diff）
#   freeze-layer-membership    冻结层归属登记表                 每层每个结构都有行、写「退出」的必是派生态、依据指得到已定分项（依据的状态借 lib-item-ref-status.py）
#   decision-items-sync        决策分项清单与正文同步           decisions.md 的分项清单生成块与索引表「状态」列，与生成器 .claude/scripts/gen-decision-items.py 的输出逐字相同
#   blocking-verdict           未定项判过改不改新池新建文件的字节   每条未定项都写了「改新池新建文件的字节」与「动不动格式」两把尺的判定（未定项清单取自同一个生成器）
#   user-verdict-owed          动了用户定案的条款记了未还的账   决策正文里标着「待用户复核」的，checks-owed.md 里要有一笔未还的账点名那条决策
#   decision-summary-width     决策索引结论列的宽度             决策索引表每一行的「结论」列不超过 decisions.md 那一句写的字数上限
#   write-frequency            同一结构的写频率只有一个量级     决策正文里一处频率断言落在登记的结构上（叫法在句内、同一表格行或所属标题），与 .claude/gate.d/write-frequency.tsv 登记的说反话
#                                                               （不同级、否定了登记罩住的时机、「只在 / 仅」限定到别的量）
#   entry-added                决策正文改了要留条目             决策正文改了，decisions-history.md 里要有新增条目
#   shape                      变更史的形状                     日期与序号分属 `### ` / `#### ` 两层；两份变更史里的日期块住在自己的 `## D<n>（` / `## E<n>（` 节；同一节里日期不重复
#   status-sync                变更史现状与索引表同步           两份变更史每节顶上的「**现状**：」一行与索引表那一行一致（--write 只重写这一行）
#
# 参数解析、样本根的 .gate-cells、每格一个子 shell、逐格汇总与退出码都经共用库 lib/stage-cells.sh（写法与判据在它的文件头），
# 这里只登记格、解析自己的 --write 与项目根。格按上表的次序一格跑完再跑下一格（决策文档组十四格原是并行跑的，并进来之后照共用库逐格跑）。
#   有一格红（退出码既不是 0 也不是 77）⇒ 整道退 1，红格逐格接它登记的出路；跑到的格全退 77 ⇒ 整道退 77，不记通过；
#   退 77 的格逐格往 $GATE_NOT_RUN_FILE 报一行「本次未跑」。
#
# --write 重写两块投影，按格表次序：先 decision-items-sync（重新生成 decisions.md 的分项清单、改写索引表「状态」列），
# 再 status-sync（按索引表重写两份变更史每节顶上的现状行）——后一块抄的是前一块写好的索引表，次序反了会抄进旧状态。
# 给了项目根写那个仓，不给写本脚本所在的仓。--write 可以与 --check decision-items-sync / --check status-sync 一起用，只写点名的那一块；
# 与别的格一起用退 2。两块写完各自回读判一遍，汇总照同一套退出码。
#
# 判别力：fixtures/doc-decisions.sh/ 下每份样本根放 .gate-cells，只跑它点名的格：
#   <格名>-red                    决策文档组的十四份红样本，各对一格（kb-shape-red、item-ref-status-red……write-frequency-red），必须判红，
#                                 expect 除那一格自己的 want 外还要汇总里「<格名>：红（退出 1）」那一行
#   decision-documents-green      决策文档组十四格的绿样本并在一棵树里（setup.sh 改编号、建 lib.rs、造 git 历史与工作区改动），
#                                 十三格都判绿、整道退 0，写频率那一格没有登记表退 77
#   write-frequency-green         格「同一结构的写频率只有一个量级」的绿样本（一份决策、登记表与排除表，不是 git 仓），那一格判绿、整道退 0
#   decision-documents-some-cells-not-run  不是 git 仓、没有分项引用、没有 D15、没有一处「待用户复核」、没有写频率登记表：
#                                 决策文档组十四格里七格退 77，整道退 0 而那七格逐格报「本次未跑」
#   decision-history-red          决策首行状态翻了、已提交没推出去、变更史没新增条目；新建的一份 kb 页里有日期与序号粘在一起的标题；
#                                 D1 的现状行与索引表对不上（entry-added、shape、status-sync 三格都红）
#   shape-cells-marker-red        现场与 decision-history-red 同样三格都红，.gate-cells 只点名 shape：只跑 shape 那一格，汇总只列一格
#   decision-history-green        决策正文改了 7 行，D1 节里新增一个日期块带一条 `#### `，现状行与索引表一致（三格都绿）
#   shape-red                     这一轮新增的 a 两处、b 四处、c 两组；基准里另有 a、b、c 各一处存量，只计数
#   shape-green                   新增的日期块都住对了节、同节日期不重复、序号写在 `#### ` 里、围栏里的旧形态不算；基准里 a、b、c 各一处存量，只计数不判红
#   status-sync-red               一节现状文字对不上、一节缺现状行、一节在索引表里没有、索引表一行没有节、实验那一份的现状对不上；另有一节对得上
#   status-sync-green             决策两节、实验一节的现状行都与索引表一致
#   shape-red、status-sync-red 只点名自己那一格；shape-green、status-sync-green 照原样三格（entry-added、shape、status-sync）一起跑。
# 汇总与 --check 的弄坏开关归共用库（STAGE_CELLS_BREAK=red-swallowed 让各份 *-red 整道退 0；=marker-ignored 让 shape-cells-marker-red
# 十七格都跑、「跑了 1 格」那条 want 找不到；=check-ignored 让 --check 不起作用）。
# 格的弄坏开关 DOC_DECISIONS_BREAK=<项>（样本自检时设进环境），每一项都让某几份样本判错：
#   <格名>                                   那一格不跑、按判过了记：那一格的红样本出不来它的红行，绿样本出不来它的成功行
#   write-frequency-levels-merged            格「同一结构的写频率只有一个量级」把不同级一律判成同级，write-frequency-red 出不来「每次挂载」那一行
#   write-frequency-negation-asserts         那一格把否定形态也算断言，write-frequency-green 里「不是每个事务推进一次」判红
#   write-frequency-subject-in-sentence-only 那一格只认句内的叫法，write-frequency-red 出不来 C245 立账那一行（主语只在标题里）的红行
#   write-frequency-negation-adjacent-only   那一格只认紧挨频率词的否定词，write-frequency-red 出不来「不会在每次发布时写」「每次发布不写」「不再每次发布推进」三行、
#                                            write-frequency-green 里「不会在每次挂载时推进」「每个事务都不写」当成肯定断言判红
#   write-frequency-restriction-as-plain     那一格把「只在 / 仅」限定形态当普通断言，write-frequency-red 出不来「只在 fsync 时写」那一行
#   stock-judged                             shape 把存量也当新增判：shape-green 判红
#   fused-ignored                            shape 不判 a：shape-red、decision-history-red 里 a 那几条 want 找不到
#   no-home-ignored                          shape 不判 b：shape-red 里 b 那几条 want 找不到
#   duplicate-date-ignored                   shape 不判 c：shape-red 里 c 那几条 want 找不到
#   status-text-ignored                      status-sync 只看现状行在不在、不比文字：status-sync-red、decision-history-red 里对不上的那几条 want 找不到
#
# ── entry-added：决策正文改了，变更史里要有新增条目 ──
# 改了决策正文却没在 `.claude/kb/decisions-history.md` 留条目 ⇒ 判红。
#
# 为什么判红而不是提醒：`.claude/rules/format-evolution.md`「硬约束」已定决策变更**必须记进**决策变更史，含推翻依据；
# 而 kb-discipline 又要求正文只写现状 ⇒ **被推翻的那句话会被直接改掉**，
# 不留条目就等于它从未存在过，三个月后没人知道它为什么不算数了。
#
# ⚠️ 判据是「有没有新增条目」，不判「条目写得对不对」——后者只有人能判。
# 条目数按新增的 `#### ` 子标题数（一条改动一个）；一个 `#### ` 都没加的，按新增的 `### 日期` 标题数。
#
# 改动范围：基准取共用库 research/scripts/changed-paths.sh 的 gate 取法（与 doc-process-records.sh、这一道 settled-same-file 格「定了新东西没回看同文件未定项」同一份）：
# 这一轮已经提交、还没推出去的决策改动也算，不只看 HEAD 之后的——先提交决策改动再跑门禁，基准是 HEAD 就什么都看不见。
# 小改动：决策正文增删合计不超过 4 行、而且一行状态都没碰（首行「## D<n> 简称 —— 状态」、「状态：」标记、
# 已定项 / 未定项的标题），不要求变更史，这一格退 77 记「本次未判」，不记通过；碰了状态行的照常判。
#
# ⚠️ **这一格自己恒绿过一段时间**（2026-08-29 复跑复核轮查出）：两个路径被塞进一个变量再加引号，
# `git diff -- "$KB"` 于是拿一个「两条路径粘成的字符串」当单个 pathspec，**匹配不到任何文件**。
# 加上历史外置到 decisions-history.md 之后仍在决策正文里找 `### 2026-`，判据也早已错位。
# ⇒ 路径改成数组，历史条目改到正确的文件里数。
#
# ── shape：变更史的形状 ──
# 判三样：
#   a) `.claude/kb/` 下任何一份 .md 里，`### 20YY-MM-DD` 后面直接跟「（其…）」：日期与序号粘在同一个标题里的旧形态。
#      同一天的改动收进一个 `### 日期`，每条各占一个 `#### ` 子标题，「（其N）」只写在 `#### ` 里。
#   b) decisions-history.md 里每个 `### 日期` 的上一级标题是 `## D<n>（`，experiments-history.md 里是 `## E<n>（`：
#      落在节外（文件头下面、`## 不挂在某一条决策上的` 这类别的节里）是条目忘了放进决策 / 实验自己的节。
#      文件自己文末的「## 历史版本」留着写这一份文件自己的改动（kb-discipline 要求每份 kb 以它收尾），那里的日期块不算节外；
#      但块里 `### ` / `#### ` 标题点名了 `D<n>（` / `E<n>（` 的，是那条决策 / 实验的条目写错了地方，照样判红。
#   c) 同一个 `## D<n>（` / `## E<n>（` 节里，同一个日期的 `### ` 标题出现两次以上（按标题开头的日期比）。
# 只判这一轮新增的：基准与 entry-added 同一个（gate_diff_base gate）。a) 看那一行是不是新增的；
#   b) 看日期块的 `### ` 行或块里某个 `#### ` 行是不是新增的；c) 看重复的几行里有没有一行是新增的。
#   基准里就有的（规则改形态之前写下、等搬迁的存量）只计数写进输出，不判红——与上游 history-ordinal.sh 对存量撞号同一个口径。
#   新增行按行号取（`git diff -U0` 的 hunk 头，未跟踪的文件整份算新增）：b、c 要知道是哪一行，
#   共用库 changed-paths.sh 的 gate_added_lines 只给行文、不给行号，所以这里另解析一次。
#   不在 git 仓里没有基准：整棵树都按新增判，输出里写明。围栏代码块里的行不算。
# ⚠️ 判不了：摘要写得对不对、一条改动该点名的决策或实验点没点全、日期块排没排成倒序、
#   同一天只有一条改动时 `#### ` 那一层省没省——这几样靠人。
#
# ── status-sync：各节顶上的现状与索引表同步 ──
# decisions-history.md 每个 `## D<n>（简称）` 节顶上（标题下第一行非空行）写一行
#   **现状**：<decisions.md 索引表那一行的「状态」格>。<「结论（简报）」格>
# experiments.md 的索引表同样有「状态」「结论（简报）」两格，experiments-history.md 的 `## E<n>（简称）` 节照同一个写法。
# 判四样：节顶上没有这一行；这一行与索引表对不上；节在索引表里没有那一行；索引表里的一行在变更史里没有节。
# --write 只重写这一行（缺了就在标题下补上），不碰历史条目；节或索引行缺了，--write 补不了，照样判红。
# 两份变更史都不在、或一节都没有且索引表一行都没有，这一格退 77（本次无对象可判）。
# ⚠️ 判不了：索引表那一行本身写得对不对。
#
#   bash .claude/gate.d/doc-decisions.sh [项目根]                                     十七格都跑
#   bash .claude/gate.d/doc-decisions.sh --list                                       逐行打格名与判什么，不跑格
#   bash .claude/gate.d/doc-decisions.sh --check <格名>[,<格名>…] [项目根]            只跑点名的格
#   bash .claude/gate.d/doc-decisions.sh --write [--check <格名>[,<格名>…]] [项目根]   按格表次序重写两块投影（分项清单与索引表状态列、变更史现状行），再回读判一遍
set -uo pipefail
# 路径要在 cd 之前算：$0 是相对路径时，cd 进项目根之后就指不到了
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
STAGE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GATE_DIRECTORY="$STAGE_DIR"
REPO_ROOT="$(cd "$STAGE_DIR/../.." && pwd)"
STAGE_RELATIVE_PATH=".claude/gate.d/$(basename "${BASH_SOURCE[0]}")"
LIB_ITEM_REF_STATUS="$GATE_DIRECTORY/lib-item-ref-status.py"
LIB_CHANGED_PATHS="$REPO_ROOT/research/scripts/changed-paths.sh"
# 生成器按本脚本自己所在的仓取，不按项目根：样本在只放着样本 kb 的临时目录里跑，那里没有 .claude/scripts/
GENERATOR="$REPO_ROOT/.claude/scripts/gen-decision-items.py"
DECISION_ITEMS_WRITE=0
BREAK_SWITCH="${DOC_DECISIONS_BREAK:-}"
# gate.sh 与阶段之间的握手变量只留给共用库在汇总时写，不导出给各格起的子进程
# （`.claude/singlefs-ai-sop/rules/command-safety.md`「进程边界上的三种静默失效」）
export -n GATE_NOT_RUN_FILE 2>/dev/null || true
source "$STAGE_DIR/lib/stage-cells.sh"
bad() { printf '  ✗ %s\n' "$*"; }
ok()  { printf '  ✓ %s\n' "$*"; }
howto() { printf '     → %s\n' "$*"; }

stage_cell kb-shape judge_kb_shape "kb 形状：分项只许一种叫法、kb 文件不链回自己、规则路径带 .claude/、决策标题与分项两节、索引页分项计数与正文相符" \
  "照这一格各段红行下面的出路改（分项叫法、自指链接、规则路径、决策标题与分项两节、索引页状态列）；状态列的计数以正文两节为准"
stage_cell item-ref-status judge_item_ref_status "每一处「D<n>（简称） 已定项 k / 未定项 k」写的状态与正文分项表相符" \
  "照红行把引用处写的「已定项 / 未定项」改成与那条决策正文分项表一致；分项改了号的，引用处跟着改号（research/scripts/relabel-item.py）"
stage_cell status-redundancy judge_status_redundancy "状态别说两遍：小节标题不带「—— 已定」、条目不同时写破折号与「状态：」、分项索引行挂在对的节" \
  "状态只在一处说：删掉小节标题里的「—— 已定」与条目里多出的那一种写法，挂错节的分项挪到它状态对应的那一节"
stage_cell cross-decision-status judge_cross_decision_status "紧贴「D<n>（简称）」写「未定」，而那条决策的状态行不是待定" \
  "读那条决策现在定了什么，把这句改写成它的现状"
stage_cell settled-item-self-open judge_settled_item_self_open "已定项区段里说它自己、或同一条决策里另一条已定分项还没定" \
  "这是定案之后没清理的推导过程：把这句改写成定案后的现状；真没定就把分项挪回未定项"
stage_cell settled-ref-says-open judge_settled_ref_says_open "引用写着「已定项 k」、归属到的那一条也已定，紧跟着却说它没定" \
  "那条分项已经定了：把句子改成它定下来之后的说法（定成了什么、这一处按它重算了没有），别只改标签"
stage_cell stale-open-items judge_stale_open_items "未定项点名的决策，状态行在这条未定项最近一次复核之后又变过" \
  "逐条复核：已被别处定掉的改写成「已定，权威记录在 XX」，仍然开着的把点名改成不构成依赖的写法"
stage_cell settled-same-file judge_settled_same_file "这次在一份决策里新增了「已定」小节，同文件开着的未定项与索引页的待议节一个字没动" \
  "被顺带定掉的改写成「已定，权威记录在 XX」或收摊，仍然开着的在条目里补一行「YYYY-MM-DD 复核过，仍然开着，因为 …」；decisions.md 的待议节同样回看"
stage_cell freeze-layer-membership judge_freeze_layer_membership "冻结层归属登记表：每层每个结构都有行、写「退出」的必是派生态、依据指得到已定分项" \
  "照红行给缺的层与结构补一行（态别按 D21 已定项 10 的判据推，推不出写「判不动」）、写「退出」的行核态别、依据写成指得到的「D<n>（简称） 已定项 k」"
stage_cell decision-items-sync judge_decision_items_sync "decisions.md 的分项清单生成块与索引表「状态」列，与生成器的输出逐字相同" \
  "权威记录是各决策正文的「### 已定项」/「### 未定项」两节；改完正文跑 bash .claude/gate.d/doc-decisions.sh --write 重写分项清单与索引表状态列"
stage_cell blocking-verdict judge_blocking_verdict "每条未定项都写了「改新池新建文件的字节」与「动不动格式」两把尺的判定" \
  "在每条缺判定的未定项的登记行或列表条目里补「改新池新建文件的字节：是 / 否 / 无对象（日期，依据：…）」与「动不动格式」那一句"
stage_cell user-verdict-owed judge_user_verdict_owed "决策正文里标着「待用户复核」的，checks-owed.md 里要有一笔未还的账点名那条决策" \
  "在 .claude/kb/checks-owed.md 立一笔未还的账点名那条决策的哪一项、原定案哪天由谁拍、复核要回答什么；已经复核完就把「待用户复核」改成结论"
stage_cell decision-summary-width judge_decision_summary_width "决策索引表每一行的「结论」列不超过 decisions.md 写的字数上限" \
  "结论列只写那条决策定了什么，为什么定、还欠什么搬进 decisions/ 下的正文"
stage_cell write-frequency judge_write_frequency "决策正文里落在登记结构上的频率断言，不与 write-frequency.tsv 登记的量级说反话" \
  "按登记行点名的权威分项改这句的频率；权威分项自己改了先改 .claude/gate.d/write-frequency.tsv 的登记；这句不是在说这个结构多久写一次，加进 .claude/gate.d/write-frequency-exclude.tsv 并写理由"
stage_cell entry-added judge_entry_added "决策正文改了，变更史里要有新增条目" \
  "在 .claude/kb/decisions-history.md 那条决策的 \`## D<n>（简称）\` 节里加条目（.claude/rules/changelog-format.md）；纯排版改动拆成单独一个提交"
stage_cell shape judge_shape "变更史的形状：日期与序号分属两层、条目住在自己的节、同节日期不重复" \
  "日期写在 \`### \`、序号写在 \`#### \`，条目搬进自己的 \`## D<n>（\` / \`## E<n>（\` 节，同节同一天并成一个日期块（.claude/rules/changelog-format.md）"
stage_cell status-sync judge_status_sync "变更史各节顶上的现状与索引表同步" \
  "先改索引表，再跑 bash .claude/gate.d/doc-decisions.sh --write 重写各节顶上的现状行；节或索引行缺了先核编号"
stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}

write_mode=0; root_argument=""
while (($#)); do
  case "$1" in
    --write) write_mode=1; shift ;;
    -*)
      bad "认不出的选项 $1"
      howto "只认 --list、--check <格名>[,<格名>…]、--write 与一个项目根；格名用 --list 看。"
      exit 2 ;;
    *)
      if [[ -n "$root_argument" ]]; then
        bad "给了两个项目根：$root_argument 与 $1"
        howto "参数只认一个项目根；不给项目根就取这个脚本往上两级。"
        exit 2
      fi
      root_argument="$1"; shift ;;
  esac
done
ROOT="${root_argument:-$REPO_ROOT}"
cd "$ROOT" 2>/dev/null || { echo "  ✗ 进不去项目根 $ROOT"; echo "     → 怎么办：第一个参数给项目根（仓的顶层目录），不给就取这个脚本往上两级；路径写错或没有权限进去时这一道什么都没判。"; exit 2; }

# ════════════════ 决策文档组的十四格（kb 形状与决策文档） ════════════════

# ── 决策索引表的「状态」列 vs 各决策正文（格「kb 形状」第 5 段取它的 declared_open_count，第 7 段整份跑它的 main）──
# 只有格「kb 形状」用它，所以住在这里、不另立库文件；第 5 段把它读成模块，第 7 段 python3 -c 直接跑。
IFS= read -r -d '' INDEX_VS_BODY_SOURCE <<'PY' || true
"""决策索引表的「状态」列 vs 各决策正文（格「kb 形状」第 7 段用）。

索引表那一格写的是**分项计数**：`已定 6 项 / 未定 0 项`，没有分项的写
`无分项 · 整条已定`。它是各正文「### 已定项」/「### 未定项」两节的投影
（`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 4 条：同一个事实只许一处权威记录）。

⚠️ **本段自己去数正文，不经过 `.claude/scripts/gen-decision-items.py`**。
写回索引页的是那个生成器，格「决策分项清单与正文同步」拿它的输出与索引页逐字比对——
**审计与被审计用同一段代码时，那个比对什么也证明不了**（`.claude/rules/fs-design.md`
「记账是事务的副产品」那一格的同一条道理）。生成器自己数错时，那一格两边一起错，
本段才会红。

⚠️ 三态词（已定 / 半定 / 待定）**不在索引列里**，它的权威记录是正文首行
`## D<n> 简称 —— 状态`。所以本段只对没有分项的那种行核一次三态词——
那种行的索引格里确实写着它。
"""
import re, sys

# 标题里声明的未定条数：阿拉伯数字，或 1–99 的中文数字（「十一」「二十三」「两」）。
# 格「kb 形状」第 5 段读同一份（不各抄一份）：中文数字只认一个字时，「十一项未定」会被读成「一项未定」。
TITLE_OPEN_COUNT = re.compile(r'([0-9]+|[一二两三四五六七八九十]+)\s*[项条]未定')
CHINESE_DIGITS = {"一": 1, "二": 2, "两": 2, "三": 3, "四": 4, "五": 5, "六": 6, "七": 7, "八": 8, "九": 9}


def parse_count_numeral(text):
    """阿拉伯数字或 1–99 的中文数字换成整数；认不出的写法（「一二」「十十」）返回 None。"""
    if text.isdigit():
        return int(text)
    tens_text, has_ten, ones_text = text.partition("十")
    if not has_ten:
        return CHINESE_DIGITS.get(text)
    if "十" in ones_text:
        return None
    tens = 1 if tens_text == "" else CHINESE_DIGITS.get(tens_text)
    ones = 0 if ones_text == "" else CHINESE_DIGITS.get(ones_text)
    if tens is None or ones is None:
        return None
    return tens * 10 + ones


def declared_open_count(title):
    """标题里声明的未定条数：(条数，认不出的写法为 None, 原文片段)；标题没声明返回 None。"""
    match = TITLE_OPEN_COUNT.search(title)
    if not match:
        return None
    return parse_count_numeral(match.group(1)), match.group(0)


CELL = re.compile(r'^已定\s*(\d+)\s*项\s*/\s*未定\s*(\d+)\s*项$')
NOITEM = re.compile(r'^无分项\s*·\s*整条(已定|半定|待定)$')


def kind(s):
    for k in ("已定", "半定", "待定"):
        if s.lstrip("*").startswith(k):
            return k
    return "?"


def count_items(body):
    """数两节各自索引表里的顶格编号项，返回 (已定数, 未定数)。

    只看**第一个 `####` 之前**那一段：子标题之下是各分项各自的论证，
    里面另有编号列表与表格，那些不是分项。
    """
    got = []
    for head in ("已定项", "未定项"):
        m = re.search(r'^### %s\s*$(.*?)(?=^#{1,3} |\Z)' % head, body, flags=re.M | re.S)
        seg = m.group(1) if m else ""
        cut = re.search(r'^#{4}\s', seg, flags=re.M)
        top = seg[:cut.start()] if cut else seg
        # 表格行要有**闭合的第二根竖线**，与 `.claude/scripts/gen-decision-items.py` 的 harvest、
        # `.claude/gate.d/lib-item-ref-status.py` 的分项表逐字同口径：口径不一时，一行畸形表格
        # 会一边数得到、一边数不到，而两边都不报错。
        got.append(len(re.findall(r'^\|\s*(\d+)\s*\|[^|]*\|', top, flags=re.M))
                   or len(re.findall(r'^\d+\.\s', top, flags=re.M)))
    return got[0], got[1]


def main():
    index_path, *bodies = sys.argv[1:]
    idx = open(index_path, encoding="utf-8").read()
    fail = 0
    seen = 0
    for p in bodies:
        body = open(p, encoding="utf-8").read()
        t = body.split("\n", 1)[0]
        # 破折号前的空格是可选的（D14 的标题就没有）。老版本只认带空格的写法，
        # 于是那一条决策**整条被跳过**，而输出里看不出少核了一条 —— 现在读不出来就判红。
        m = re.match(r'## (D\d+) .+?\s*——\s*(.+)$', t)
        if not m:
            print(f"  ✗ {p} 首行读不出 `## D<n> 简称 —— 状态`：{t[:40]}")
            print("     → 怎么办：把首行写成 `## D3 空间分配 —— 半定（…）`；")
            print("       读不出来就是这一条没核过，不许当成通过。")
            fail = 1
            continue
        num, bs = m.group(1), m.group(2)
        row = re.search(r'^\| %s（[^|]*） *\| ([^|]+)\|' % num, idx, flags=re.M)
        if not row:
            print(f"  ✗ {num} 在 decisions.md 索引里找不到对应行")
            print("     → 怎么办：给它补一行 `| D<n>（简称） | 状态 | 结论（简报） | 正文 |`，")
            print("       状态列跑 bash .claude/gate.d/doc-decisions.sh --write 生成。")
            fail = 1
            continue
        seen += 1
        isx = row.group(1).strip()
        # 正文里实际有几条分项 —— 本段自己数的那一份
        n_settled, n_open = count_items(body.split("\n## 历史版本")[0])

        mc, mn = CELL.match(isx), NOITEM.match(isx)
        if not mc and not mn:
            print(f"  ✗ {num} 状态列写着「{isx}」，不是分项计数的写法")
            print("     → 怎么办：状态列一律写 `已定 N 项 / 未定 M 项`（没有分项的写 `无分项 · 整条已定`）；")
            print("       跑 bash .claude/gate.d/doc-decisions.sh --write 从正文重新生成。")
            fail = 1
            continue
        if mn:
            if n_settled or n_open:
                print(f"  ✗ {num} 状态列写「无分项」，而正文里数出 {n_settled + n_open} 条分项")
                print("     → 怎么办：跑 bash .claude/gate.d/doc-decisions.sh --write")
                fail = 1
            elif mn.group(1) != kind(bs):
                print(f"  ✗ {num} 状态列写「整条{mn.group(1)}」，正文标题是「{kind(bs)}」")
                print("     → 怎么办：三态词的权威记录是正文首行；改完正文跑 --write 重新生成索引列。")
                fail = 1
            continue
        if (int(mc.group(1)), int(mc.group(2))) != (n_settled, n_open):
            print(f"  ✗ {num} 状态列写「已定 {mc.group(1)} 项 / 未定 {mc.group(2)} 项」，"
                  f"而正文两节里数出「已定 {n_settled} 项 / 未定 {n_open} 项」")
            print("     → 怎么办：权威记录是正文的「### 已定项」/「### 未定项」两节。")
            print("       改完正文跑 bash .claude/gate.d/doc-decisions.sh --write")
            fail = 1
            continue
        # 正文标题若自己声明了未定条数，也要与实数相符（写在标题里的那句同样会被检索到）；没声明就是 0
        declared = declared_open_count(bs)
        if declared is not None and declared[0] is None:
            print(f"  ✗ {num} 正文标题写「{declared[1]}」，里面的数认不出来")
            print("     → 怎么办：条数写阿拉伯数字，或 1–99 的中文数字（「十一项未定」「两项未定」）。")
            fail = 1
        elif (declared_count := 0 if declared is None else declared[0]) != n_open:
            print(f"  ✗ {num} 正文标题说「{declared_count} 项未定」，而「### 未定项」一节里是 {n_open} 项")
            print("     → 怎么办：改正文标题里的那个数，或把分项挪到对的那一节去。")
            fail = 1

    if not seen:
        print("  ✗ 一条决策的索引行都没核到——本段等于没跑")
        print("     → 怎么办：确认 decisions.md 的索引行形如 `| D1（简称） | … |`，")
        print("       以及 decisions/*.md 首行形如 `## D1 简称 —— 已定`。")
        sys.exit(1)
    if not fail:
        print(f"  ✓ 决策索引表的状态列与正文相符（{seen} 条决策，分项数是本段自己数的）")
    sys.exit(1 if fail else 0)


if __name__ == "__main__":
    main()
PY

# ── 格「未定项被别处定了」的复核判据：一条未定项对它点名的决策 Dn，最近一次「复核」是什么时候 ──
# 只有那一格用它，所以住在这里、不另立库文件；那一格逐条 python3 -c 跑它。
IFS= read -r -d '' OPEN_ITEM_REVIEW_SOURCE <<'PY' || true
"""格「未定项被别处定了」的复核判据：一条未定项对它点名的决策 Dn，最近一次「复核」是什么时候。

复核 = 条目块历史里**最近一次让 Dn 的点名次数变多**的那次提交（条目诞生那次也算）；
工作区里条目块比 HEAD 多点了 Dn 一次，算刚复核过。改别的地方不算。

为什么不是「条目块最后一次改动」：那样任何一次改动都能把红消掉。实测（2026-09-11）：
D19（块指针的结构与宽度预算） 未定项 6 那一格因 D16 的状态变动被标红，同日往那一格补了一段性能数（与 D16 无关），
红就没了——D16 那两句还成不成立是事后手核的，门禁没逼。

条目块：从分项首行起，到下一条分项（编号列表项或「| k |」表格行）或下一个标题之前。
表格行一行就是一条分项，不再像改前那样一直延到下一个标题、把后面几行分项都算进来。

用法：python3 -c "$OPEN_ITEM_REVIEW_SOURCE" <文件> <工作区里条目首行的行号> <Dn:状态变动时间>…
打印没复核过的那几个 Dn，一行一个。

条目块的历史取不到（git log -L 退非 0，或一个提交都没交出来）退 3、原因打到 stderr：
取不到历史不是「复核过了」，调用方（那一格）按「判据没跑成」判红。
"""
import re
import subprocess
import sys


class BlockHistoryUnavailable(Exception):
    """git log -L 没交出条目块的历史：对象缺了、行范围超出了文件、仓坏了。"""

ITEM_BOUNDARY = re.compile(r'^\s*\d+\.\s|^#{2,6} |^\|\s*\d+\s*\|')


def block_end(lines, start_index):
    for index in range(start_index + 1, len(lines)):
        if ITEM_BOUNDARY.match(lines[index]):
            return index - 1
    return min(start_index + 20, len(lines) - 1)


def mention_count(decision, text):
    return len(re.findall(r'(?<![A-Za-z0-9])' + decision + r'(?!\d)', text))


def head_start_index(head_lines, work_first_line):
    for index, line in enumerate(head_lines):
        if line == work_first_line:
            return index
    # 首行在工作区里改过：按同一个分项号在 HEAD 的「### 未定项」那一节里找
    prefix = re.match(r'^(\|\s*\d+\s*\||\s*\d+\.\s)', work_first_line)
    if not prefix:
        return None
    inside = False
    for index, line in enumerate(head_lines):
        if re.match(r'^### 未定项\s*$', line):
            inside = True
            continue
        if re.match(r'^#{2,4} ', line):
            inside = False
        if inside and line.startswith(prefix.group(1)):
            return index
    return None


def history_of_block(path, first_line_number, last_line_number):
    """条目块的历史（HEAD 里第 first..last 行，从 1 数）：[(提交时间, 改前, 改后)]，新的在前。"""
    # --no-color 写死：使用者配了 color.ui / color.diff = always 时，每行前面多一段转义码，
    # 「@@」「+」「-」一个都认不出，复核过的条目被读成没复核（绿样本 setup.sh 配的就是这种）。
    result = subprocess.run(['git', 'log', '--no-color', '-L', f'{first_line_number},{last_line_number}:{path}', '--format=%x00%ct'],
                            capture_output=True, text=True)
    if result.returncode != 0:
        # 失败时 stdout 可能是空的，也可能是半截（-L 边走边打）：两种都不能当成历史用
        reason = (result.stderr.strip().splitlines() or ['（git 没说原因）'])[-1]
        raise BlockHistoryUnavailable(f'git log -L {first_line_number},{last_line_number}:{path} 退出码 {result.returncode}：{reason}')
    log = result.stdout
    history = []
    for chunk in log.split('\x00')[1:]:
        lines = chunk.split('\n')
        try:
            committed = int(lines[0].strip())
        except ValueError:
            continue
        before, after, inside_hunk = [], [], False
        for line in lines[1:]:
            if line.startswith('@@'):
                inside_hunk = True
                continue
            if not inside_hunk:
                continue
            if line.startswith('-'):
                before.append(line[1:])
            elif line.startswith('+'):
                after.append(line[1:])
            elif line.startswith(' '):
                before.append(line[1:])
                after.append(line[1:])
        history.append((committed, '\n'.join(before), '\n'.join(after)))
    return history


def main():
    path, work_line_number = sys.argv[1], int(sys.argv[2])
    dependencies = [(entry.split(':')[0], int(entry.split(':')[1])) for entry in sys.argv[3:]]
    head = subprocess.run(['git', 'show', f'HEAD:{path}'], capture_output=True, text=True)
    if head.returncode != 0:
        return   # 文件还没进过 HEAD：比任何决策都新，无从陈旧
    head_lines = head.stdout.split('\n')
    with open(path, encoding='utf-8') as handle:
        work_lines = handle.read().split('\n')
    work_start = work_line_number - 1
    head_start = head_start_index(head_lines, work_lines[work_start])
    if head_start is None:
        return   # HEAD 里没有这条分项 = 新加的，无从陈旧
    head_end = block_end(head_lines, head_start)
    work_end = block_end(work_lines, work_start)
    head_block = '\n'.join(head_lines[head_start:head_end + 1])
    work_block = '\n'.join(work_lines[work_start:work_end + 1])
    try:
        history = history_of_block(path, head_start + 1, head_end + 1)
    except BlockHistoryUnavailable as error:
        print(f'  取不到条目块的历史，这一条没比：{error}', file=sys.stderr)
        sys.exit(3)
    if not history:
        # 退 0 却一个提交都没交出来：条目块在 HEAD 里，就一定有引入它的那次提交；什么都没有是判不了，不是「复核过了」
        print(f'  git log -L 对 {path} 第 {head_start + 1}–{head_end + 1} 行一个提交都没交出来，这一条没比', file=sys.stderr)
        sys.exit(3)
    for decision, changed_at in dependencies:
        if mention_count(decision, work_block) > mention_count(decision, head_block):
            continue   # 工作区里刚补了一句点名它的复核
        reviewed_at = next((committed for committed, before, after in history
                            if mention_count(decision, after) > mention_count(decision, before)), 0)
        if changed_at > reviewed_at:
            print(decision)


if __name__ == '__main__':
    main()
PY

# ════ 格「kb 形状」════
# 查「同一件事在 kb 里有两种写法」。指代（位置指代、自指称呼、时间指代）由上游 doc-lint.sh 判，这里不重复。
#
# 为什么这几条要判红而不是写成提醒：
#   kb 按「被单条取出」设计（.claude/singlefs-ai-sop/rules/kb-discipline.md）。
#   同一概念两个名字（未答项 / 未定项）会让按其中一个名字的检索漏掉另一半；
#   标题里写死的条数与列表对不上，检索到标题的人拿到的就是错的。
#
# 判别力：fixtures/doc-decisions.sh/kb-shape-red 每一段都至少犯一次，必须判红；decision-documents-green 必须判绿。
# 第 7 段「一条决策的索引行都没核到」与「状态列的分项计数与正文不符」互斥（后者要先核到一行），
# 红样本里放的是后者。
grid_kb_shape() (
  KB=.claude/kb
  fail=0
  bad() { printf '  ✗ %s\n' "$*"; fail=1; }
  ok()  { printf '  ✓ %s\n' "$*"; }
  howto() { printf '     → %s\n' "$*"; }

  # 扫的文件装进数组，不靠 $(find …) 的分词：文件名里有空格时会被切成两个不存在的路径，
  # 而 grep 打不开它们的报错原先丢进 /dev/null——那一份里写了什么都判不到。
  kb_md_files=()
  while IFS= read -r -d '' found_file; do kb_md_files+=("$found_file"); done \
    < <(find "$KB" -name '*.md' -print0 2>/dev/null)
  shopt -s nullglob
  rule_md_files=(.claude/rules/*.md)
  kb_top_md_files=("$KB"/*.md "$KB"/decisions/*.md)
  shopt -u nullglob
  kb_and_rule_md_files=(${kb_md_files[@]+"${kb_md_files[@]}"} ${rule_md_files[@]+"${rule_md_files[@]}"})
  grep_error_file="$(mktemp)"
  trap 'rm -f "${grep_error_file:?}"' EXIT
  # 用法：grep_files <这一段叫什么> <grep 参数…> -- <文件…>；命中写进 $grep_hits。
  # 退 0：都读到了。退 1：一份要扫的文件都没有（判红）。退 2：有文件读不了（判红；读到的那些照样交命中）——
  # 出错的那几份里写了什么都没判，不能读成「没命中」，所以调用方只在退 0 时报绿。
  grep_files() {
    local stage_label="$1" grep_exit_code=0; shift
    local -a grep_options=()
    while [[ $# -gt 0 && "$1" != "--" ]]; do grep_options+=("$1"); shift; done
    shift
    grep_hits=""
    if [[ $# -eq 0 ]]; then
      bad "$stage_label：一份要扫的文件都没有，这一段没判"
      howto "确认门禁是在仓库根上跑的、$KB 目录在；扫到 0 份不是通过（.claude/singlefs-ai-sop/rules/show-me-test.md「扫到 0 项也不是通过」）。"
      return 1
    fi
    grep_hits="$(grep -Hn "${grep_options[@]}" -- "$@" 2>"$grep_error_file")" || grep_exit_code=$?
    if [[ "$grep_exit_code" -ge 2 ]]; then
      bad "$stage_label：有文件读不了（grep 退 $grep_exit_code），那几份没判："; sed 's/^/     /' "$grep_error_file"
      howto "按上面 grep 的原话修：悬空的符号链接删掉或指回真文件，没有读权限的补上读权限；读不了的那几份不能当成没命中。"
      return 2
    fi
    return 0
  }

  echo "── 1. 同一概念只许一个名字 ──"
  grep_status=0
  grep_files "「未答项 / 已答项」" -e '未答项' -e '已答项' -- ${kb_and_rule_md_files[@]+"${kb_and_rule_md_files[@]}"} || grep_status=$?
  if [[ -n "$grep_hits" ]]; then
    bad "kb 里出现「未答项 / 已答项」"; printf '%s\n' "$grep_hits" | sed 's/^/     /'
    howto "统一写「未定项 / 已定项」。records/ 是当时的会话记录，不在本检查范围。"
  elif [[ "$grep_status" -eq 0 ]]; then ok "未定项 / 已定项 用词统一（扫 ${#kb_md_files[@]} 份 kb 文件、${#rule_md_files[@]} 份规则）"; fi

  grep_status=0
  grep_files "「### 未定」/「### 已定」小节标题" -e '^### 未定$' -e '^### 已定$' -- ${kb_top_md_files[@]+"${kb_top_md_files[@]}"} || grep_status=$?
  if [[ -n "$grep_hits" ]]; then
    bad "小节标题写作「### 未定」/「### 已定」"; printf '%s\n' "$grep_hits" | sed 's/^/     /'
    howto "统一写「### 未定项」/「### 已定项」——按标题检索时两种写法只能命中一种。"
  elif [[ "$grep_status" -eq 0 ]]; then ok "已定项 / 未定项 小节标题统一（扫 $KB 顶层与 decisions/ 下 ${#kb_top_md_files[@]} 份）"; fi

  echo
  echo "── 3. 文件内自指链接 ──"
  # 扫 kb 下每一份 .md：链接目标（去掉 #锚点）按这份文件自己的目录解析之后就是它自己，判红。
  # 历史类文件（`*-history.md`、`decisions-history/` 下的月份文件）整份跳过：它们逐字记着当时的原文，
  # 里面抄录的链接多半是当时的原文，这一道分不出哪一句换了会成假话，所以整份不判；这是这一道的射程，
  # 不是豁免：`.claude/rules/path-moves.md`「改一个全仓术语：正文之外还有五处会红」里「历史类文件同样换名」那一段要逐句判，历史类文件里的自链靠人看。
  self_link_report=$(python3 - "$KB" <<'PY'
import os, re, sys, glob
kb = sys.argv[1]
scanned = 0
for path in sorted(glob.glob(os.path.join(kb, "**", "*.md"), recursive=True)):
    if os.path.basename(path).endswith("-history.md") or "/decisions-history/" in path:
        continue
    scanned += 1
    own = os.path.normpath(path)
    with open(path, encoding="utf-8") as handle:
        for line_number, line in enumerate(handle, 1):
            for link in re.finditer(r'\[[^\]\n]*\]\(([^)\s]+)\)', line):
                target = link.group(1).split("#", 1)[0]
                if not target or "://" in target:
                    continue
                if os.path.normpath(os.path.join(os.path.dirname(path), target)) == own:
                    print(f"HIT {path}:{line_number}: {link.group(0)}")
print(f"SCANNED {scanned}")
PY
  )
  self_link_scanned=$(sed -n 's/^SCANNED //p' <<<"$self_link_report")
  hit=$(sed -n 's/^HIT //p' <<<"$self_link_report")
  if [[ -z "$self_link_scanned" ]]; then
    bad "扫自指链接的 python 没跑完（没报出扫了几份），这一段等于没判"
    howto "单独跑这一段看它报什么错（多半是某份文件不是 UTF-8）；没跑完不许当成没有自指链接。"
  elif [[ "$self_link_scanned" -eq 0 ]]; then
    bad "一份 kb 文件都没扫到（$KB 下的 .md，历史类文件除外）——这一段没有对象可判"
    howto "确认门禁是在仓库根上跑的、$KB 目录在；扫到 0 项不是通过（.claude/singlefs-ai-sop/rules/show-me-test.md）。"
  elif [[ -n "$hit" ]]; then
    bad "kb 文件里有链接指回它自己：$(grep -c . <<<"$hit") 处（扫 $self_link_scanned 份）"; printf '%s\n' "$hit" | sed 's/^/     /'
    howto "同一文件内直接写编号带简称（例：D8（核心索引结构））或小节标题，不要链回本文件。"
  else ok "没有文件内自指链接（扫 $self_link_scanned 份 kb 文件，历史类文件除外）"; fi

  echo
  echo "── 4. 上游规则的路径写法 ──"
  # 裸 singlefs-ai-sop/rules/… 从仓库根解析不到，副本在 .claude/ 下。行首的裸路径也算（前面没有字符可配）。
  grep_status=0
  grep_files "上游规则的裸路径" -E -e '(^|[^/.])singlefs-ai-sop/rules/' -- ${kb_and_rule_md_files[@]+"${kb_and_rule_md_files[@]}"} || grep_status=$?
  hit=""
  [[ -n "$grep_hits" ]] && hit="$(grep -v '\.claude/singlefs-ai-sop/rules/' <<<"$grep_hits")"
  if [[ -n "$hit" ]]; then
    bad "上游规则写成了裸路径"; printf '%s\n' "$hit" | sed 's/^/     /'
    howto "统一写 .claude/singlefs-ai-sop/rules/<文件>.md ——裸路径从仓库根打不开。"
  elif [[ "$grep_status" -eq 0 ]]; then ok "上游规则路径统一（扫 ${#kb_md_files[@]} 份 kb 文件、${#rule_md_files[@]} 份规则）"; fi

  echo
  echo "── 5. 决策标题：编号连续、带状态；分项两节严格分开、未定条数与列表相符 ──"
  # ⚠️ 2026-08-29：此前这一段只读 decisions.md，而决策正文早已拆到 decisions/ 下，
  # decisions.md 里一个 `## D<n>` 标题都没有 ⇒ **整段恒绿**，标题声明的未定项条数从没被核过。
  # 现在逐个读 decisions/*.md。
  INDEX_VS_BODY_SOURCE="$INDEX_VS_BODY_SOURCE" python3 - $KB/decisions/*.md <<'PY'
import os, re, sys, types
# 标题里未定条数的读法与第 7 段同一份（INDEX_VS_BODY_SOURCE 的 declared_open_count），不各抄一份
lib_index_vs_body = types.ModuleType("lib_index_vs_body")
exec(compile(os.environ["INDEX_VS_BODY_SOURCE"], "INDEX_VS_BODY_SOURCE", "exec"), lib_index_vs_body.__dict__)
body = ""
for p in sys.argv[1:]:
    body += open(p, encoding="utf-8").read().split("\n## 历史版本", 1)[0] + "\n"
fail = 0
index_rows_checked = 0
def bad(m, how=""):
    """每一条拒绝都要给下一步——`.claude/singlefs-ai-sop/scripts/gate-lint.sh` 管的是
    上游脚本里的 `bad "..."`，够不着这里的 python，所以这条纪律在本文件里只能自己守。"""
    global fail
    print(f"  ✗ {m}")
    if how:
        print(f"     → {how}")
    fail = 1

# 编号连续（每个文件一个标题，按文件名顺序读进来，所以直接比对）
nums = [int(m.group(1)) for m in re.finditer(r'^## D(\d+) ', body, flags=re.M)]
if not nums:
    bad("一个 `## D<n>` 标题都没读到——这一段又变成恒绿了，先修检查再谈别的",
        "确认 decisions/*.md 首行是 `## D<n> 简称 —— 状态`；读不到就是这段检查在空转。")
elif nums != list(range(1, len(nums) + 1)):
    bad(f"决策编号不连续或不从 D1 起：{nums}",
         "决策编号必须从 D1 起连号；废弃一条也要留占位并在变更史里写明。")

# 切成每条决策
starts = [m.start() for m in re.finditer(r'^## D\d+ ', body, flags=re.M)]
starts.append(len(body))
decision_count = len(starts) - 1
for a, b in zip(starts, starts[1:]):
    blk = body[a:b]
    title = blk.split("\n", 1)[0]
    if not re.search(r'——\s*(已定|半定|待定)', title):
        bad(f"标题缺状态（已定/半定/待定）：{title[:50]}",
            "首行写成 `## D<n> 简称 —— 已定/半定/待定（…）`，索引行要与它一致。")
    # 分项分住两节：「### 已定项」全是已定的，「### 未定项」全是未定的。
    # 只看每节的**索引**（第一个 `####` 之前那一段）——之后是各分项各自的论证，
    # 那里面另有编号列表，混进来会把论证的第 1/2/3 条当成分项。
    def index_of(head):
        sec = re.search(r'^### %s$(.*?)(?=^#{1,3} |\Z)' % head, blk, flags=re.M | re.S)
        if not sec:
            return None
        t = sec.group(1)
        cut = re.search(r'^#{4}\s', t, flags=re.M)
        return t[:cut.start()] if cut else t
    # ① + ② 严格区分：每条分项索引行必须**自带状态词**，且那个状态词要与它所在的小节一致。
    #    kb 按「被单条取出」设计（kb-discipline 第 1 条）：检索端出来的是那一行，
    #    不是它上面的小节标题 ⇒ 状态只挂在位置上，单条取出时状态就没了。
    #
    #    ⚠️ **判状态之前必须先剥掉自引用短语**（「正文见 D2（RAID 条带策略）「已定项 2」」）。
    #    第一版没剥，于是本仓通行的那句自引用**本身就含「已定项」三个字**，
    #    足以让一条状态词被整段删空的行照样通过——2026-08-30 的对抗验证用故障注入当场击穿。
    #    ⚠️ **也不许只认加粗写法**：第一版的正则是 `\*\*未定` 与 `——\s*已定`，
    #    把状态词写成不加粗的「未定」就逃过去了，同一轮验证一并击穿。
    #    ⚠️ **也不许靠「取行内第一个状态词」来猜**：一条合法的分项常常先提到**别处**的状态
    #    （D12 未定项 3 逐字是「判据已定（2026-08-27），但答案的前置未定」，
    #    D13 未定项 2 先提到「D8（核心索引结构） 已定的 write buffer」）——猜法当场两条假阳性。
    #    现在要求每行带一个**规范标记**「状态：已定」/「状态：未定」，判据因此不是猜的。
    #    ⚠️ **判不了的那一半**：行首状态词写对、而后半句自相矛盾（「未定：…这一条已定为…」），
    #    本检查看不见。落点 [checks-owed.md](../kb/checks-owed.md) C51（分项引用只验状态不验身份）。
    for head in ("已定项", "未定项"):
        seg = index_of(head)
        if not seg:
            continue
        for row in re.findall(r'^(?:\|\s*\d+\s*\||\d+\.\s).*$', seg, flags=re.M):
            index_rows_checked += 1
            marks = re.findall(r'状态：\s*\*{0,2}(已定|未定)', row)
            if not marks:
                bad(f"{title.split()[1]} {head}里有一条分项没写「状态：已定/未定」这个规范标记，"
                    f"状态只挂在小节位置上：{row[:56]}",
                    "在这一行末尾补上 **状态：已定。** 或 **状态：未定。**——检索端出来的是这一行，不是它上面的小节标题。")
            elif len(set(marks)) > 1:
                bad(f"{title.split()[1]} {head}里有一条分项写了两个互相冲突的状态标记："
                    f"{row[:56]}",
                    "一行只许有一个「状态：」标记；要提别处的状态就写成「D<n>（简称） 已定项 k」。")
            elif marks[0] + "项" != head:
                bad(f"{title.split()[1]} {head}里有一条分项的状态标记写着「{marks[0]}」："
                    f"{row[:56]}",
                    "要么把它挪到对应的小节去（编号不变），要么改正状态标记；两节的编号是同一套。")
    # ③ 已定项那一侧的正文不许自陈「还没定」。
    #    实测（2026-08-30 五轮验证）：D23 已定项 8 的正文末尾留着一句
    #    「只走了本地腿，两条云端腿欠着——所以它落成未定项，不是定案」，
    #    而同一分项的标题、以及下游三条分项都以它已定为前提。**状态一致性检查看不见这一类**：
    #    它比的是引用处与索引表，比不了同一分项正文内部自相矛盾。
    #    措辞是窄的、只认「说本项自己没定」那几句，避免误伤「别处仍未定」这种合法陈述
    #    （本仓当前 0 假阳性：同样的词在未定项小节里 0 命中）。
    SELF_UNDECIDED = r'不是定案|落成未定项|退回未定|本项未定|本项仍未定|本项还没定|该项未定|仍未定案|尚未定案'
    regions = []
    m0 = re.search(r'^### 已定项\s*$(.*?)(?=^### |\Z)', blk, flags=re.M | re.S)
    if m0:
        regions.append(m0.group(1))
    for hm in re.finditer(r'^(#{3,4}) 已定项 \d+[^\n]*$', blk, flags=re.M):
        rest = blk[hm.end():]
        nx = re.search(r'^#{1,%d} ' % len(hm.group(1)), rest, flags=re.M)
        regions.append(rest[:nx.start()] if nx else rest)
    for reg in regions:
        for hit in re.finditer(SELF_UNDECIDED, reg):
            a = max(0, hit.start() - 34)
            bad(f"{title.split()[1]} 已定项一侧的正文自陈还没定："
                f"…{reg[a:hit.end() + 10]}…",
                "正文只写现状：定了就删掉这句陈旧的证据等级；真没定就把这条分项挪回未定项小节。")
    # ④ 两节合起来，编号必须**唯一且从 1 连到 n**。
    #    一条决策的分项只有一套编号，分住两节；重号会让「D8 已定项 2」同时指两件事，
    #    断号会让读的人以为中间那条被删了。两者都不改任何状态词 ⇒ 前两条检查看不见。
    nums = []
    for head in ("已定项", "未定项"):
        seg = index_of(head)
        if not seg:
            continue
        nums += [int(x) for x in (re.findall(r'^\|\s*(\d+)\s*\|', seg, flags=re.M)
                                  or re.findall(r'^(\d+)\.\s', seg, flags=re.M))]
    if nums:
        dup = sorted({n for n in nums if nums.count(n) > 1})
        if dup:
            bad(f"{title.split()[1]} 分项编号重号：{dup}（一套编号分住两节，重号等于一个号指两件事）",
                "给后加的那条换一个没用过的号，并同步改全仓引用；改完跑 .claude/gate.d/doc-decisions.sh，看格「分项引用状态」")
        elif sorted(nums) != list(range(1, len(nums) + 1)):
            bad(f"{title.split()[1]} 分项编号不是从 1 连到 {len(nums)}：{sorted(nums)}",
                "断号说明有分项被删了却没交代。要么补回那一条，要么在变更史里写明它去哪了。")
    # ⑤ 标题声明的未定条数与「### 未定项」小节的分项数相符
    #    中文数字按 1–99 读（「十一项未定」是 11，不是 1），读法在 INDEX_VS_BODY_SOURCE
    declared = lib_index_vs_body.declared_open_count(title)
    if declared is None:
        continue
    want, declared_phrase = declared
    if want is None:
        bad(f"{title.split()[1]} 标题写「{declared_phrase}」，里面的数认不出来",
            "条数写阿拉伯数字，或 1–99 的中文数字（「十一项未定」「两项未定」）。")
        continue
    t = index_of("未定项")
    if t is None:
        bad(f"标题声明了「{declared_phrase}」却没有「### 未定项」小节：{title[:40]}",
                "补一个「### 未定项」小节，或把标题里的未定条数改成 0 并去掉那句。")
        continue
    got = len(re.findall(r'^\|\s*\d+\s*\|', t, flags=re.M)) or \
          len(re.findall(r'^\d+\.\s', t, flags=re.M))
    if not got:
        first = [l for l in t.strip().split("\n") if l.strip()]
        got = 1 if first else 0
        if not got:
            bad(f"「### 未定项」小节里既没有编号条目也没有表格：{title[:40]}",
                "分项要写成带编号的表格行或编号列表，生成器与门禁都按这两种形状抽。")
            continue
    if got != want:
        bad(f"{title.split()[1]} 标题写「{declared_phrase}」，「### 未定项」小节里实际 {got} 项",
                "改正文标题里的条数，并同步 decisions.md 索引行；改完跑 .claude/gate.d/doc-decisions.sh --write")
if not fail:
    print(f"  ✓ 决策标题与未定项列表相符，且两节没有互相串味（{decision_count} 条决策、核 {index_rows_checked} 行分项索引）")
sys.exit(1 if fail else 0)
PY
  [[ $? -ne 0 ]] && fail=1

  echo
  echo "── 7. 决策索引表的状态列（分项计数）vs 正文实际分项数 ──"
  # ⚠️ 2026-08-30 实测踩过：给 D21 加了两个未定项、改了正文标题（两项→四项），
  # 而 decisions.md 的索引行还写「两项未定」。第 5 段只比「正文标题 vs 正文列表」，
  # 索引页在它的视野之外 ⇒ 这类不一致此前无人拦。
  # ⚠️ 本段**自己数正文**，不经过 gen-decision-items.py：写回索引页的是那个生成器，
  # 格「决策分项清单与正文同步」拿它的输出与索引页逐字比对 ⇒ 生成器数错时两边一起错，只有本段会红。
  if python3 -c "$INDEX_VS_BODY_SOURCE" "$KB/decisions.md" $KB/decisions/*.md; then :; else fail=1; fi

  echo
  if [[ $fail -eq 0 ]]; then echo "  ✓ kb 形状检查通过（kb 文件 ${#kb_md_files[@]} 份、规则 ${#rule_md_files[@]} 份）"; else echo "  ✗ kb 形状检查未通过"; fi   # gate-lint:summary
  exit $fail
)

# ════ 格「分项引用状态」════
# 每一处「D<n>（简称） 已定项 k / 未定项 k」的前缀都在**断言那条分项的状态**。
# 写错了没有任何东西会发现：检索会把「已定项 5」当成已经定了的东西端出去，
# 而它可能还开着——这正是 kb-discipline 第 4 条「矛盾比空白更糟」说的那种坏法
# （检索不会把两条都端出来，它会挑一条，而且不告诉你它挑了）。
#
# 权威是各决策正文的「### 已定项」与「### 未定项」两张索引表；
# 引用处一律是投影。扫 kb / records / research / crates 全域，含 .rs 注释。
# 判据、出路与成功行都在 lib-item-ref-status.py 的 main（同一份库另有调用方：格「冻结层归属登记表」与 research/scripts/relabel-item.py）。
# 判别力：fixtures/doc-decisions.sh/item-ref-status-red 必须判红。
grid_item_ref_status() (
  python3 "$LIB_ITEM_REF_STATUS"
)

# ════ 格「状态别说两遍」════
# 分项已经按状态分住「### 已定项」与「### 未定项」两节 ⇒ **节本身就是状态**。
# 再在标题或条目里写一遍「—— 已定」，同一个词就说了两遍；
# 而重复的标注会各自漂移——检索取到其中一处时不知道另一处写的是什么
# （`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 4 条：矛盾比空白更糟）。
#
# 判三样：
#   1. 小节标题不许写成「已定项 N —— 已定…」（节名已经说过了；编号后面带括注的也算）
#   2. 同一个条目不许既有「—— 已定 / 未定」又有「状态：已定 / 未定」，两处的值相不相同都判
#   一份都没扫到、或一条分项索引行都没核到，退 77（不记通过）。
#   3. 条目的「状态：」必须与它所在的节一致（已定项节里不许有状态：未定）
#   2、3 只看每节的索引（第一个 `####` 之前），与格「kb 形状」第 5 段同一条边界；
#   `####` 之下的论证里另有编号列表，不是分项。
#
# 判别力：fixtures/doc-decisions.sh/status-redundancy-red 三样各犯一次，必须判红；
# decision-documents-green 里另放一份论证带「3. …状态：未定」的决策，必须判绿。
#
# ⚠️ 只认**显式的**「状态：X」与破折号形态，不认句中随口提到的「已定」——
#    一条未定项里出现「D18 已定项 3 已定」是正常引用，不是它自己的状态。
grid_status_redundancy() (
  DEC=.claude/kb/decisions
  [[ -d "$DEC" ]] || { echo "  ! 找不到 $DEC，这一格跳过"; exit 77; }

  # 扫的范围：决策正文 + 实验正文 + records/。
  # 后两处今天是干净的（2026-08-31 现查：三类毛病各 0 处），扫它们是**防复发**——
  # 一条只在出事之后才加的检查，等于承认那一次出事没人拦得住。
  python3 - "$DEC" .claude/kb/experiments records <<'PY'
import re, sys, glob, os
bad = []
files = []
rows_checked = 0
for d in sys.argv[1:]:
    if os.path.isdir(d):
        files += sorted(glob.glob(os.path.join(d, '**', '*.md'), recursive=True))

for f in files:
    body = open(f, encoding='utf-8').read().split('\n## 历史版本')[0]
    name = os.path.basename(f)

    # 1. 标题里重复
    for i, line in enumerate(body.split('\n'), 1):
        # 编号后面许跟一段括注（「已定项 3（2026-09-01）—— 已定」），不许的话带日期的标题整个漏过去
        if re.match(r'^#{3,4} (已定项|未定项) \d+ *(?:[（(][^）)]*[）)])? *—— *\*{0,2}(已定|未定)', line):
            bad.append((name, i, '标题重复', line.strip()[:60]))

    # 2 / 3. 条目层
    # 只看每节的**索引**：节到下一个一至三级标题为止，再在第一个 `####` 处收口——
    # `####` 之下是各分项自己的论证，里面另有编号列表（「3. …状态：未定」这类），那些不是分项。
    # 边界与格「kb 形状」第 5 段的 index_of、INDEX_VS_BODY_SOURCE 的 count_items 同一条。
    for sec_name, want in (('已定项', '已定'), ('未定项', '未定')):
        m = re.search(r'^### %s\s*$(.*?)(?=^#{1,3} |\Z)' % sec_name, body, re.M | re.S)
        if not m:
            continue
        section_text = m.group(1)
        first_subheading = re.search(r'^#{4}\s', section_text, re.M)
        index_text = section_text[:first_subheading.start()] if first_subheading else section_text
        base = body[:m.start(1)].count('\n') + 1
        for off, line in enumerate(index_text.split('\n')):
            if not re.match(r'^(?:\d+\.|\|\s*\d+\s*\|)', line):
                continue
            rows_checked += 1
            ln = base + off
            dash = re.search(r'——\s*\*{0,2}(已定|未定)', line)
            stat = re.search(r'状态：\s*\*{0,2}(已定|未定)', line)
            # 破折号与「状态：」同时出现就算说了两遍，两处写的值相不相同都一样：值不同时是两处已经漂开了
            if dash and stat:
                bad.append((name, ln, '条目说两遍', line.strip()[:60]))
            if stat and stat.group(1) != want:
                bad.append((name, ln, '挂错节', line.strip()[:60]))

if bad:
    print('  ✗ 状态被说了两遍，或分项挂错了节：')
    for n, ln, why, txt in bad[:20]:
        print(f'     {n}:{ln}  {why} —— {txt}')
    if len(bad) > 20:
        print(f'     …… 另有 {len(bad)-20} 处')
    print('     → 小节标题写成「### 已定项 N（日期）：结论」，别再写「—— 已定」；')
    print('     → 条目二选一：留「状态：已定」这一处机器可读的标注，把「—— 已定」去掉，')
    print('       日期与结论一个字都不要丢；')
    print('     → 状态与所在节不一致的，把条目搬到对的那一节，别就地改状态词。')
    sys.exit(1)

# 扫到 0 份、或一条分项索引行都没核到：条目层的 2、3 没有对象，退 77，不记通过
# （.claude/singlefs-ai-sop/rules/show-me-test.md「扫到 0 项也不是通过」）。
if not files or rows_checked == 0:
    print(f'  ! 本次无对象可判：扫了 {len(files)} 份、核到 {rows_checked} 条分项索引行（决策正文、实验正文、records/ 下的 .md 里没有分项索引）')
    sys.exit(77)
print(f'  ✓ 状态只说一遍，且分项都在对的节里（扫 {len(files)} 份、核 {rows_checked} 条分项索引行）')
PY
)

# ════ 格「说未定其实已定」════
# **判据**：kb 正文里凡是紧贴着「D<n>（简称）」写下「未定」的句子，
# 拿它与那条决策**状态行**上的实际状态比对；实际不是「待定」（含「待定（…）」带括注的写法）就判红。
# 历史节只认整行的「## 历史版本」：「### 历史版本对照」这类标题不是文末历史节，它后面的正文照判。
#
# ⚠️ **它与格「未定项被别处定了」不是一条。**
# 那一格只扫**未定项小节**，且靠「谁比谁新」这个时间判据；
# 这一格扫**全部正文**，靠「说的状态与写着的状态对不对得上」这个文本判据。
# 实测三处它抓得到而那一格抓不到：一处写在**已定项的论证里**（D22 轴② 的前置①
# 说「D25，取值未定」，而 D25 已定），一处写在**不变量清单的开篇**
# （说「D4/D6/D8/D9/D10 未定」，而其中四条都定了），一处写在**实验正文的局限里**。
# 三处都不在未定项小节，那一格一个字都看不见。
#
# ⚠️ **为什么必须机检**：`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 4 条
# 「矛盾比空白更糟」——检索不会把两条都端出来，它挑一条而且不告诉你挑了哪条。
# 一句陈旧的「X 还没定」会让读的人以为那一格还开着，从而**不去读那条定案**。
#
# ⚠️ **判据是收紧过的，收紧的理由是实测**：第一版允许「D<n> …… 未定」之间隔任意
# 40 字，在真实语料上报 32 处而只有 4 处是真的——分项索引表的「状态：未定」、
# 自动生成的分项清单、「从未定过」、以及**被引号括起来当历史陈述引用的**
# 「且 D6 未定」全被误伤。假红压倒真红的检查等于没有检查
# （`.claude/singlefs-ai-sop/rules/show-me-test.md`），所以只认**紧贴**的写法，
# 并跳过「」引文。代价是漏掉隔着名词短语的那种（例：「D2 的条带宽度可变粒度仍然未定」），
# 那一类留给人看——**宁可漏，不可假红**。
#
# 判别力：fixtures/doc-decisions.sh/cross-decision-status-red 里「D180 未定」而 D180 标着已定，必须红；
# decision-documents-green 里同样紧贴着写「未定」而被引的那条标着待定（含「待定（两项未定）」），必须绿。
grid_cross_decision_status() (
  KB="$PWD/.claude/kb"
  # 无对象可判退 77，门禁记「本次未跑」，不记通过（`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）
  [[ -d "$KB/decisions" ]] || { echo "  ! 没有 $KB/decisions，这一格无对象可判"; exit 77; }

  python3 - "$KB" <<'PY'
import re, sys, pathlib

kb = pathlib.Path(sys.argv[1])

# 1) 每条决策的实际状态：取正文标题行 `## D<n> 简称 —— 状态`
# ⚠️ **破折号前的空格是可选的**：实测 D14 写作「…持久临时）—— 半定」，
# 第一版的 `\s+——` 把整条 D14 静默漏掉了，而它恰恰是矛盾最密的一条。
# ⚠️ **解析不了的标题一律判红，不许当成跳过**
# （`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）——
# 一条没被扫到的决策与一条干净的决策，在门禁输出里长得一模一样。
status, name, unparsed = {}, {}, []
title_re = re.compile(r'^##\s+(D\d+)\s*(.+?)\s*——\s*(.+?)\s*$')
for f in sorted((kb / "decisions").glob("*.md")):
    lines = f.read_text(encoding="utf-8").splitlines()
    hit = next((m for m in (title_re.match(l) for l in lines) if m), None)
    if hit:
        status[hit.group(1)], name[hit.group(1)] = hit.group(3), hit.group(2)
    elif any(re.match(r'^##\s+D\d+', l) for l in lines):
        unparsed.append(f.name)
if unparsed:
    for fn in unparsed:
        print(f"  ✗ {fn} 的决策标题解析不了，这一格没有扫过它")
    print("     → 标题要写成 `## D<n> 简称 —— 状态`。解析不了就等于没扫，不许当成通过。")
    sys.exit(1)
if not status:
    print("  ! 没有决策正文，这一格无对象可判"); sys.exit(77)

# 2) 紧贴写法：D<n>（简称） 之后只允许空白 / 逗号 / 括号 / 「取值」「状态」
#    再接可选的「仍/仍然/尚/都/均」，然后就是「未定」，且不许是「未定项」。
ref_re = re.compile(r'D\d+(?:（[^）]*）)?')
tight_re = re.compile(r'^[\s，,、（(]*(?:取值|状态)?[\s：:]*(?:仍然|仍|尚|都|均)?未定(?!项)')

bad, in_hist = [], False
scanned_files, history_files, history_section_lines, quoted_references = 0, [], 0, 0
for f in sorted(kb.rglob("*.md")):
    rel = f.relative_to(kb.parent.parent)
    # 决策变更史整份都是历史：decisions-history.md 里每条条目的「改前 / 改后」写的是当时的状态，组织形态见 .claude/rules/changelog-format.md
    if f.name == "decisions-history.md" or f.parent.name == "decisions-history":
        history_files.append(str(rel))
        continue
    scanned_files += 1
    in_hist = False
    for i, line in enumerate(f.read_text(encoding="utf-8").splitlines(), 1):
        # 只认整行的「## 历史版本」（kb 文件文末那一节）：前缀匹配会让「### 历史版本对照」把后半份文件整段跳过
        if re.match(r'^##\s*历史版本\s*$', line):
            in_hist = True          # 文末历史节按定义写的是旧状态，不判
        if in_hist:
            history_section_lines += 1
            continue
        for m in ref_re.finditer(line):
            d = re.match(r'D\d+', m.group(0)).group(0)
            if d not in status:
                continue
            tm = tight_re.match(line[m.end():])
            if not tm:
                continue
            # 「」引文里的是被当作历史陈述引用的原话，不判
            head = line[:m.end()]
            if head.count('「') > head.count('」'):
                quoted_references += 1
                continue
            # 标题可以写成「待定（两项未定）」：逐字比「待定」会把合法的「D<n> 未定」判红
            if re.match(r'\*{0,2}待定', status[d]):
                continue
            bad.append((str(rel), i, d, status[d], line.strip()[:110]))
            break

if bad:
    for rel, ln, d, st, text in bad:
        print(f"  ✗ {rel}:{ln} 说 {d} 未定，而 {d}（{name[d]}）的状态行是「{st}」")
        print(f"     原文：{text}")
    print("     → 怎么办：读那条决策现在定了什么，把这句改写成它的现状；")
    print("               若指的是它下面某个还开着的分项，写成「D<n>（简称） 未定项 k」。")
    sys.exit(1)
print(f"  ✓ 没有把已定的决策说成未定（扫 {scanned_files} 份 kb 文件，比对 {len(status)} 条决策的状态行）")
print(f"     没查的：决策变更史 {len(history_files)} 份（整份是历史）；各文件「## 历史版本」一节里 {history_section_lines} 行；"
      f"「」引文里紧贴决策号的 {quoted_references} 处（被引用的原话）")
for history_file in history_files:
    print(f"       变更史：{history_file}")
PY
)

# ════ 格「已定分项自称未定」════
# **判据**：在一条决策的「已定项」小节（含各 `#### 已定项 N` 论证）里，
# 凡出现「X 仍未定 / X 还没定 / X 尚未定」这样的断言，就查 X 是谁：
#   a. X 是**自指**（取值 / 本项 / 该项 / 本分项；「`T_time` 的取值」这类打头是「的」的，去掉「的」再比）⇒ 这条分项标着已定却说自己没定，判红；
#   b. X 落在**同一个决策里另一条已定分项的标题**上 ⇒ 说一个已经定了的东西没定，判红。
#   同一行有几处断言就逐处看，直到判出一处为止。没有一份决策有已定项区段 ⇒ 无对象可判，退 77。
#
# ⚠️ **这一类只发生在同一个文件内部，而门禁此前对它整个是盲的。**
# 格「kb 形状」比的是索引行与标题行的状态，格「分项引用状态」比的是引用处
# 写的状态与索引表，两个都只看**状态标记**，不看正文说了什么。
# 实测三处：D16 已定项 5 定了 `T_time` = 5 s，同一项正文里留着
# 「取值仍未定」；D23 已定项 10 说「hash 算法仍未定」而已定项 11 定了 CRC32C；
# D23 已定项 8 自己的标题写着「宽度 32 位」而它的论证里留着「宽度仍未定」。
# 三处都是**定案之后没清理推导过程**留下的，而检索会把陈旧的那一条单独端出来
# （`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 8 条）。
#
# ⚠️ **不判「本笔仍然未定」这一类。** 一条已定分项里就某个**子问题**显式记
# 「这一笔还没定」是 kb-discipline 第 3 条要求的「显式记录不知道」，不是矛盾；
# 真要收拾它，该做的是把那个子问题升成独立分项，而不是把这句话删掉。
# 判据因此只认自指与兄弟分项两种，**宁可漏，不可假红**。
#
# 判别力：fixtures/doc-decisions.sh/settled-item-self-open-red 里已定项 1 的正文写「取值仍未定」（a 支），必须红；
# 另一份决策在已定项 2 的正文里说「校验算法选哪个仍未定」，而已定项 1 的标题就是它（b 支），也必须红，两支各有一条 want；
# decision-documents-green 里同一句的无害版本（「本分项取区间上端 5」）留在已定项论证里，必须绿。
grid_settled_item_self_open() (
  DEC="$PWD/.claude/kb/decisions"
  # 无对象可判退 77，门禁记「本次未跑」，不记通过（`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）
  [[ -d "$DEC" ]] || { echo "  ! 没有 $DEC，这一格无对象可判"; exit 77; }

  python3 - "$DEC" <<'PY'
import re, sys, pathlib

dec = pathlib.Path(sys.argv[1])
SELF = ("取值", "本项", "该项", "本分项", "本条")
OPEN = re.compile(r'(仍然未定|仍未定|尚未定(?!案)|还没定|均未定)')
# X：断言之前那一小段名词短语。停在标点 / markdown 记号 / 空白上。
PHRASE = re.compile(r'([^\s。，、；：（）()「」|*`＊>#⚠️⇒—]{2,12})$')

bad = []
files_with_settled_region, settled_region_lines = 0, 0
decision_files = sorted(dec.glob("*.md"))
if not decision_files:
    print(f"  ! {dec} 下没有决策正文，这一格无对象可判")
    sys.exit(77)
for f in decision_files:
    lines = f.read_text(encoding="utf-8").splitlines()

    # 已定项小节 = `### 已定项` 索引表 + 各 `#### 已定项 N` 论证
    inside, region, settled_lines = False, [], []
    for i, line in enumerate(lines, 1):
        if re.match(r'^###\s+已定项\s*$', line) or re.match(r'^####\s+已定项', line):
            inside = True
            settled_lines.append(line)
            continue
        if re.match(r'^###\s+未定项\s*$', line) or re.match(r'^####\s+未定项', line) or re.match(r'^##\s', line):
            inside = False
        if inside:
            region.append((i, line))
            settled_lines.append(line)

    if not region:
        continue
    files_with_settled_region += 1
    settled_region_lines += len(region)
    settled_text = "\n".join(settled_lines)

    for i, line in region:
        for m in OPEN.finditer(line):
            head = line[:m.start()].rstrip()
            pm = PHRASE.search(head)
            if not pm:
                continue
            x = pm.group(1).strip('*` ')
            if len(x) < 2:
                continue
            # 「`T_time` 的取值仍未定」：短语停在反引号与空格上，取到的是「的取值」；去掉打头的「的」再比自指词
            x_core = x[1:] if x.startswith('的') else x
            why = None
            if x_core in SELF:
                why = "自指——这条分项标着已定，正文却说它自己没定"
            else:
                # 同一文件里另一条已定分项的标题含 X？（排除本行自己）
                for other in settled_text.splitlines():
                    if other.strip() == line.strip():
                        continue
                    if re.match(r'^\s*(\|\s*)?\d+[.\s|]', other) and x in other and '已定' in other:
                        why = f"「{x}」在同一条决策的另一条**已定**分项里已经定了"
                        break
            # 一行只报一处；这一处没判出问题就接着看同一行后面的断言（「子问题仍未定，取值仍未定」）
            if why:
                bad.append((f.name, i, x, why, line.strip()[:110]))
                break

if bad:
    for fn, ln, x, why, text in bad:
        print(f"  ✗ {fn}:{ln} 已定项正文里断言「{x}」未定：{why}")
        print(f"     原文：{text}")
    print("     → 怎么办：这是定案之后没清理的推导过程。把这句改写成定案后的现状；")
    print("               若那个子问题真的还开着，把它升成一条独立的未定项，别留在已定项正文里。")
    sys.exit(1)
# 每份决策都没有已定项区段：一行都没判，退 77，不记通过
if files_with_settled_region == 0:
    print(f"  ! 本次无对象可判：{len(decision_files)} 份决策正文里没有一份有已定项区段（### 已定项 / #### 已定项 N）")
    sys.exit(77)
print(f"  ✓ 已定项的正文没有把已经定了的东西说成未定（扫 {len(decision_files)} 条决策，"
      f"其中 {files_with_settled_region} 条有已定项区段、共 {settled_region_lines} 行）")
PY
)

# ════ 格「引用写已定紧跟说没定」════
# 一条分项从未定翻成已定之后，全仓的引用要从「未定项 k」改写成「已定项 k」（格「分项引用状态」逼的）；
# 改完标签，**句子本身**常常还在说它没定：「D19 已定项 6 未定」「那个 key 是 D19 已定项 6、今天没定」。
# 格「分项引用状态」只比标签，格「说未定其实已定」只认紧贴「D<n>（简称）」的「未定」，格「已定分项自称未定」只扫已定项小节里的自指——三个都看不见这一型。
# 实测（2026-09-11，D19 已定项 6 定案）：改完标签之后这样的句子留了六处（三份实验源码的注释、D06、两份实验正文），
# 是逐句手找出来的。
#
# 判据：归属按格「分项引用状态」的规则（同一份库 `lib-item-ref-status.py`，不另抄）；归属到的那一条是已定、引用处也写着「已定项」，
# 而它后面紧跟着「未定 / 没定 / 定不下 / 待定 / 空着」（中间只许有一段括注、标点与「今天 / 仍 / 还」这类词）⇒ 判红。
# 两份变更史与 records/ 不扫：它们写的是当时的状态，按「今天的编号」改写之后本来就会读成这样。成功行报没扫的份数并逐个列名（现算）。
# 判别力：fixtures/doc-decisions.sh/settled-ref-says-open-red 必须判红；decision-documents-green 里 records/ 下同一种句子不扫、必须绿。
grid_settled_ref_says_open() (
  python3 - "$LIB_ITEM_REF_STATUS" <<'PY'
import importlib.util, sys
spec = importlib.util.spec_from_file_location('item_ref_status', sys.argv[1])
lib = importlib.util.module_from_spec(spec); spec.loader.exec_module(lib)
heads, unreadable = lib.decision_heads(strict=False)
item_map, names = lib.load_map(heads)
self_map = lib.self_decisions(heads)
def written_as_of_then(f):
    return (f.endswith(('decisions-history.md', 'experiments-history.md')) or '/decisions-history/' in f
            or f.startswith('records/'))
candidates = lib.scanned_files()
files = [f for f in candidates if not written_as_of_then(f)]
skipped = [f for f in candidates if written_as_of_then(f)]
bad = []; seen = 0
for path in files:
    for ln, line, m, want, owner, missing, _ in lib.references(path, item_map, self_map):
        if missing or owner is None or want != '已' or item_map[owner][int(m.group(2))] != '已':
            continue
        seen += 1
        if lib.says_open_after(line, m.end()):
            snippet = line[max(0, m.start() - 20):m.end() + 30].strip()
            bad.append(f"{path}:{ln} 「{owner}（{names[owner]}） {m.group(0)}」后面紧跟着说它没定：…{snippet}…")
if unreadable:
    lib.report_unreadable(unreadable)
if bad:
    print(f"  ✗ 引用写着已定项、紧跟着却说它没定 {len(bad)} 处")
    for b in bad[:40]: print("    ", b)
    print("     → 那条分项已经定了：把句子改成它定下来之后的说法（定成了什么、这一处按它重算了没有），别只改标签。")
if bad or unreadable:
    sys.exit(1)
# 一处归属到已定分项的「已定项」引用都没扫到，这一轮什么都没判过：退 77，不报绿
# （`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）。
if seen == 0:
    print(f"  ! 扫了 {len(files)} 个文件，没有一处归属到已定分项的「已定项」引用，这一格无对象可判")
    sys.exit(77)
print(f"  ✓ 扫了 {len(files)} 个文件、{seen} 处已定项引用，没有一处紧跟着说它没定")
print(f"    没扫 {len(skipped)} 个（两份变更史与 records/ 写的是当时的状态）" + ('：' + '、'.join(skipped) if skipped else ''))
PY
)

# ════ 格「未定项被别处定了」════
# 还 checks-owed.md C31（未定项被别处定了却没回收）。
#
# **判据**：一个还开着的未定项，如果它的正文点名了另一个决策，
# 而那个决策的**状态行**在它最后一次被改动之后又变过 —— 就要求复核。
# 「它的正文」是整个条目块：首行加它下面的续行，到下一条分项（编号列表项或「| k |」表格行）或下一个标题之前、至多 20 行，
# 与复核判据（OPEN_ITEM_REVIEW_SOURCE）的条目块同一个口径——列表式条目常把「依赖 Dn」写在续行里，只看首行就漏了。
#
# ⚠️ **纯文本的依赖图抓不到这一类**：实测 D22 的三个陈旧未定项里，
# 两个根本没有「前置是某某」这种标记（一个是「本事务内释放的块不得重用」
# 与 D16 新规则 2 撞了，一个是「D2 的口径要扩」而 D2 早就扩完了）。
# 能抓住它们的只有「谁比谁新」这个时间判据。
# ⚠️ **本检查对 2026-08-29 决策文档拆分之前的历史无效**：
# 拆分把一份 4830 行的 decisions.md 变成 25 个文件，逐行历史在那里断了，
# 两侧的时间戳都塌到拆分那一次提交。**它管的是今后**。
# 判别力在一次性合成仓里双向证过：D2 后定而 D22 的项没动 ⇒ rc=1；
# 把那条项改写成「已定」之后 ⇒ rc=0。
# git 取不到时间（对象缺了、仓坏了）判红，不当「这条决策从没定过」：那样点名它的未定项一条都不比，照样报绿。
# 样本：fixtures/doc-decisions.sh/stale-open-items-red 另造一个缺了旧版本对象的仓：一份决策的 git log -G 读不到，
# 一份未定项的条目块 git log -L 读不到，两处都要报出来。
grid_stale_open_items() (
  DEC=.claude/kb/decisions
  # 无对象可判退 77，门禁记「本次未跑」，不记通过（`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）
  [[ -d "$DEC" ]] || { echo "  ! 没有 $DEC，这一格无对象可判"; exit 77; }
  git rev-parse --git-dir >/dev/null 2>&1 || { echo "  ! 不在 git 仓库里，这一格跳过"; exit 77; }

  # 每个决策状态行最后一次变动的提交时间；取不到（git 失败）记 -1，点名它的未定项逐条报「没比过」
  declare -A st_time
  time_unavailable=()
  for f in "$DEC"/*.md; do
    n=$(grep -m1 -oE '^## (D[0-9]+)' "$f" | awk '{print $2}') || continue
    [[ -n "$n" ]] || continue
    # ⚠️ **不能拿标题行当「这个决策定了新东西」的信号**——标题行几乎不变，
    # 实测在已知有三处陈旧未定项的历史点上一条都不报。
    # 改用「这个文件里新增过『已定』标记」的那次提交时间。
    # ⚠️ **必须匹配「小节标题」，不能匹配正文里的「已定」二字。**
    # 用 -S'—— 已定' 时，正文里一句「D21 已定索引是派生态」也会命中，
    # 于是任何一次给决策补实验结论都会把引用它的未定项全报一遍（实测三处假阳性）。
    # -G 加行首锚定只认 `## …—— 已定` / `### …—— 已定` 这种标题行。
    # git 自己的退出码先判，再取第一行：写成「git log … | head -1」时管道只交 head 的退出码，git 失败被读成「从没定过」
    if ! t_out="$(git log -1 --format=%ct -G'^#{2,4} .*—— 已定' -- "$f")"; then
      st_time[$n]=-1
      time_unavailable+=("$n（$(basename "$f")）")
      continue
    fi
    t="$(head -n 1 <<<"$t_out")"
    # ⚠️ **没有兜底。** 第一版取不到时退化成「文件最后修改时间」，
    # 于是任何一次给决策补内容都会把引用它的未定项全报一遍。
    # 从没加过「—— 已定」小节标题 = 它没定过任何东西 = 不该触发任何复核。
    [[ -n "$t" ]] || t=0
    st_time[$n]=$t
  done

  flagged=0
  if ((${#time_unavailable[@]})); then
    flagged=1
    echo "  ✗ 这些决策的「—— 已定」小节最近一次变动时间取不到（git log 失败），点名它们的未定项这一轮没法比：${time_unavailable[*]}"
    echo "     → 怎么办：按上面 git 的报错修好仓库（git fsck 看缺了哪些对象，从远端或备份找回）再跑；取不到时间不是「这条决策从没定过」，不是通过。"
  fi
  open_items=0     # 扫到的未定项条数
  judged_items=0   # 其中点名了「定过东西的别的决策」、真拿去比过复核时间的条数
  for f in "$DEC"/*.md; do
    self=$(grep -m1 -oE '^## (D[0-9]+)' "$f" | awk '{print $2}')
    # 未定项行：编号开头且标着未定
    while IFS=: read -r ln text; do
      [[ -n "$ln" ]] || continue
      open_items=$((open_items + 1))
      # ⚠️ **要看整个条目块的最新改动，不是首行。**
      # 复核通常写在条目下面，首行的 blame 时间不动——只看首行会让
      # 复核过的条目永远红着，检查退化成噪声（实测踩过）。
      # ⚠️ **行号必须先映射回 HEAD 的版本，不能直接拿工作区的行号喂 `git log -L`。**
      # `-L` 的行范围是按**历史里的文件**解释的：工作区一旦有未提交改动，
      # 上方插几行就让整段偏移，`-L` 去历史里看的是另一段内容（实测：工作区 140 行、
      # HEAD 131 行，140 在 HEAD 里是空行）⇒ 取不到时间戳 ⇒ **把复核过的条目报成陈旧**。
      # 这是假红，而假红压倒真红的检查等于没有检查
      # （`.claude/singlefs-ai-sop/rules/show-me-test.md`）。
      # 判别力：改前在「工作区有未提交改动且条目上方插过行」时必红，改后转绿。
      # ⚠️ **「条目块动过」不等于「复核过」**（2026-09-12 改）：此前拿条目块最后一次改动的时间当复核时间，
      # 于是**任何**一次改动都能把红消掉——2026-09-11 往 D19 未定项 6 那一格补性能数（与 D16 无关），
      # 它对 D16 状态变动的那一红就这样没了，D16 那两句还成不成立是事后手核的，门禁没逼。
      # ⇒ 复核时间按「被点名的那条决策」逐条算：条目块历史里最近一次让它的点名次数变多的那次提交
      #   （条目诞生那次也算）；工作区里条目块比 HEAD 多点了它一次，算刚复核过。
      #   复核写一句「（YYYY-MM-DD 复核 Dn：……）」就满足；改别的地方不算。判据住在 OPEN_ITEM_REVIEW_SOURCE。
      deps=()
      # 点名按整个条目块认（首行加续行，到下一条分项或下一个标题之前、至多 20 行），与 OPEN_ITEM_REVIEW_SOURCE 的 block_end 同一个口径
      block_text="$(awk -v s="$ln" 'NR == s { print; next }
          NR > s && NR <= s + 20 { if (/^[[:space:]]*[0-9]+\. / || /^##+ / || /^\|[[:space:]]*[0-9]+[[:space:]]*\|/) exit; print }' "$f")"
      # ⚠️ D 编号前面要是非字母数字：「RAID5」里的「D5」不是在点名 D5（实测：D2 未定项 15 因此被报成依赖 D5，
      # 改判据之前那一红被任何一次改动顺手消掉，改判据之后永远复核不掉）。
      for d in $(grep -oE '(^|[^A-Za-z0-9])D[0-9]+' <<<"$block_text" | grep -oE 'D[0-9]+' | sort -u); do
        [[ "$d" == "$self" ]] && continue
        dt=${st_time[$d]:-0}
        if (( dt < 0 )); then
          echo "  ✗ $(basename "$f"):$ln 的未定项点名了 $d，而 $d 的变动时间取不到，这一条对 $d 没比过"   # gate-lint:detail
          continue
        fi
        (( dt > 0 )) && deps+=("$d:$dt")
      done
      (( ${#deps[@]} > 0 )) || continue
      judged_items=$((judged_items + 1))
      # 复核判据的输出先落到变量、判过退出码再读：接进 `< <(…)` 时它崩了只是少打几行，这一条被读成「复核过了」
      review_out="$(python3 -c "$OPEN_ITEM_REVIEW_SOURCE" "$f" "$ln" "${deps[@]}")" || {
        review_rc=$?
        echo "  ✗ $(basename "$f"):$ln 的复核判据没跑成（复核判据 OPEN_ITEM_REVIEW_SOURCE 退出码 $review_rc），这一条没比过"
        echo "     → 按上面的报错修 .claude/gate.d/doc-decisions.sh 里的 OPEN_ITEM_REVIEW_SOURCE 或这一份决策正文；判据没跑成不是通过。"
        exit 1
      }
      while IFS= read -r d; do
        [[ -n "$d" ]] || continue
        echo "  ✗ $(basename "$f"):$ln 的未定项点名了 $d，而 $d 的状态行在它之后变过"
        echo "     ⇒ 复核这一项是不是已经被 $d 定掉了；复核完在这一条里写一句点名 $d 的复核记录。原文：${text:0:60}"
        flagged=1
      done <<<"$review_out"
    # ⚠️ **列表式未定项的行内不含「未定」二字**——那两个字在小节标题上。
    # 第一版按行内关键字过滤，把 D22 那三条陈旧项全滤掉了，于是检查恒绿。
    # 改成：取「### 未定项」小节内的条目行，再排掉已经标了「已定」的。
    done < <(awk -F: '
        # ⚠️ 只认**光秃秃的**「### 未定项」标题。
        # 「### 未定项 3 —— 已定」也以它开头，第一版把已定案小节里的表格行
        # 全当成未定项抓了进来，报出两处假阳性。
        # ⚠️ **在第一个 `####` 处也收口**：小节里索引表之后是各分项各自的论证，
        # 那里面另有编号列表，不收口会把论证的第 1/2/3 条当成分项。
        /^### 未定项[[:space:]]*$/  { inside=1; next }
        /^### /              { inside=0 }
        /^#### /             { inside=0 }
        /^## /               { inside=0 }
        inside && /^[[:space:]]*[0-9]+\. |^\| [0-9]+ \|/ { print NR":"$0 }
      ' "$f")
    # ⚠️ **不再按行内关键字滤掉「已定」**：已定的分项现在住在「### 已定项」小节里，
    # 这一节按定义全是未定的。老版本那道 `grep -v 已定` 有个静默盲区——
    # 一条**未定**分项只要正文里提到别处的「已定」（例：D18 的「与 D16 已定的
    # checkpoint 序号怎么共存」）就会被滤掉，从此不被这一格看一眼。
  done

  if ((flagged)); then
    echo "     → 怎么办：逐条复核；已被别处定掉的就改写成「已定，权威记录在 XX」，"
    echo "               仍然开着的就把点名改成不构成依赖的写法。"
    exit 1
  fi
  # 没有一条未定项点名定过东西的别的决策，这一轮一次复核时间都没比：退 77，不报绿
  # （`.claude/singlefs-ai-sop/rules/show-me-test.md`「扫到 0 项也不是通过」）。
  if ((judged_items == 0)); then
    echo "  ! 查了 $open_items 条未定项，没有一条点名定过东西的别的决策，这一格无对象可判"
    exit 77
  fi
  echo "  ✓ 没有未定项被别处的更新甩在后面（查了 $open_items 条未定项，其中 $judged_items 条点名了定过东西的别的决策、逐条比过复核时间）"
)

# ════ 格「定了新东西没回看同文件未定项」════
# 还 checks-owed.md C36（未定项检查的三个盲区）的前两条。
#
# **它与格「未定项被别处定了」是两把不同的尺子，不是加强版**：
#   「未定项被别处定了」：**跨文件 + 看历史**——未定项点名了别的决策，而那个决策后来定过东西。
#   这一格：**同文件 + 看本次 diff**——本次 diff 在某个决策里新增了一个「已定」小节，
#       而同一个文件里还开着的未定项**这次一个都没碰**。
#
# ⚠️ **为什么必须看 diff 而不是看历史**：按历史判会在每个「有已定小节又有未定项」的
# 决策上恒红，而 D13（验证路线） 已定项 9（.claude/kb/decisions/13-验证路线.md）写过
# 「一个不可复现的夜间失败价值接近 0，甚至为负，它训练人忽略红灯」——一个恒红的检查比没有检查更坏。
# 看 diff 则只在**真正发生了定案的那一次提交**上说话，其余时候安静。
#
# **它抓的两个实测形态**（2026-08-29 审计轮，一轮之内各撞一次）：
#   ① D25 同一个文件里既写「已定：取粗粒度 8 叶 1 脊柱」，
#      又在「还要回答的」里留着「目标负载的两个数取什么值。这一步只能由人定」。
#   ② decisions.md 索引页末尾「待议：记账与反向索引的隔离纪律」自 2026-08-25 悬着，
#      而 D6 定案取「付 O(N) 次反向索引查找」已经实质选中了它要禁的那件事。
#      格「未定项被别处定了」看不见它，因为那一格只扫 decisions/ 下的正文文件，**不扫索引页**。
# ⚠️ **`git diff --name-only` 对非 ASCII 文件名默认做 C 转义**（`"\347\233\256…"`），
# 而本仓**每一个决策文件名都是中文** ⇒ 拿转义后的串再去 `git diff -- "$f"` 匹配不到任何文件
# ⇒ **检查恒绿**。必须带 `-c core.quotepath=false`。
# 这个 bug 在合成仓的双向验里当场暴露，是「新增的检查必须先证明它会红」抓到的第二个。
# 「碰没碰」按条目块判：表格行一行就是一条分项，列表式条目从首行到下一条分项或下一个标题之前（与 OPEN_ITEM_REVIEW_SOURCE 同一个口径）；
# 索引页的「待议」按那一节判（从 `## 待议` 到下一个 `## ` 之前），整份 decisions.md 别处动过不算碰了它。
# 这份文件跟踪没有，按 `git ls-files --error-unmatch` 的退出码判：1 是未跟踪（整份算碰过），别的非 0 是 git 自己出错，判红。
# 新增了已定小节的那几份里一条未定项都没有、索引页也没有待议节时，什么都没查，退 77。
# 样本：fixtures/doc-decisions.sh/settled-same-file-red 另放一份两行表格的决策（只改了第 2 行，第 1 行要报）与一节没动的待议（同一次只改了索引表那一行）。
grid_settled_same_file() (
  DEC=.claude/kb/decisions
  IDX=.claude/kb/decisions.md
  # 无对象可判退 77，门禁记「本次未跑」，不记通过（`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）
  [[ -d "$DEC" ]] || { echo "  ! 没有 $DEC，这一格无对象可判"; exit 77; }
  git rev-parse --git-dir >/dev/null 2>&1 || { echo "  ! 不在 git 仓库里，这一格跳过"; exit 77; }

  # 改动范围取共用脚本 research/scripts/changed-paths.sh（门禁 code-tooling 的 change-range-single-source 格判阶段里不另算一份）：
  # 基准是 gate_diff_base gate，名单带未跟踪文件——新写的决策文件在 git add 之前也算这次改动。
  # shellcheck source=../../research/scripts/changed-paths.sh
  source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"; echo "     → 怎么办：它随仓走（research/scripts/changed-paths.sh），被删了就从 git 找回来。"; exit 1; }
  BASE="$(gate_diff_base gate)"

  # 本次改动碰过的决策文件。名单先落到变量、判过退出码再读：git 失败时名单是空的，会被读成「本次没有新增已定小节」。
  all_changed="$(gate_changed_paths "$BASE" untracked)" || {
    echo "  ✗ 取不到这次改动碰了哪些路径（基准 $BASE），这次改了哪些决策文件没取到"
    echo "     → 按上面 git 的报错修好仓库状态（基准 $BASE 要存在；仓里还没有提交就先提交一次）再跑；git 失败时这一格什么都没比，不是通过。"
    exit 1
  }
  changed_names="$(grep "^\.claude/kb/decisions/" <<<"$all_changed" || true)"

  # 本次 diff 里**新增**了「已定」小节标题的决策文件
  settled_files=()
  while IFS= read -r f; do
    [[ -n "$f" ]] || continue
    # 只认新增行（+），且必须是小节标题：`## D1 简称 —— 已定` / `### 未定项 3 —— 已定` / `### 已定（…）`
    # ⚠️ **这个正则第一版写坏过，而且是「写坏了但恒绿」那种坏法**：
    # 写成 '^\+#{2,4} .*(—— 已定|^\+### 已定)' 时，第二个分支里的 ^ 在组内永远匹配不上，
    # 于是 `### 已定（…）` 这种最常见的定案小节标题一个都抓不到，检查恒绿。
    # 合成仓双向验的时候当场红——**这就是「新增的检查必须先证明它会红」拦下来的那一次**。
    # 同 doc-registries.sh 那条：pipefail + `grep -q` 提前退出 ⇒ 前段 SIGPIPE ⇒ 命中被读成没命中。
    # `git diff` 的输出可以很大，这里比那条更容易撞上。
    # 新增行按共用脚本取（未跟踪的文件整份算新增）；取不到就判红，不当「没有新增」
    added_out="$(gate_added_lines "$BASE" "$f")" || {
      echo "  ✗ 取不到 $f 这次新增了哪些行（基准 $BASE）"
      echo "     → 按上面 git 的报错修好仓库状态再跑；取不到新增行时这一格什么都没比，不是通过。"
      exit 1
    }
    added_text="$(cut -f2- <<<"$added_out")"
    if grep -qE '^#{2,4} .*—— 已定|^#{2,4} 已定[（(]' <<<"$added_text"; then
      settled_files+=("$f")
    fi
  done <<<"$changed_names"

  if ((${#settled_files[@]} == 0)); then
    # 退 77：门禁记「本次未跑」，不记通过（`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）
    echo "  ! 本次 diff 没有新增「已定」小节，这一格无对象可判"
    exit 77
  fi

  # 这次改动碰过的行号（新文件侧），一行一个。未跟踪的新文件整份都是这次写的，每一行都算碰过。
  # 跟踪没有按 ls-files --error-unmatch 的退出码判：0 跟踪、1 未跟踪，别的码是 git 自己出错——返回 2，调用方判红，
  # 不当「未跟踪」（那样整份每行都算碰过，这一份一条都不报）。
  touched_lines_of() {
    local file="$1" tracked_rc=0
    git -c core.quotepath=false ls-files --error-unmatch -- "$file" >/dev/null 2>&1 || tracked_rc=$?
    case "$tracked_rc" in
      0) git diff --no-color --no-ext-diff -U0 "$BASE" -- "$file" \
           | awk 'match($0,/^@@ .* \+([0-9]+)(,([0-9]+))? @@/,m){s=m[1]; n=(m[3]==""?1:m[3]); for(i=0;i<n;i++) print s+i}' ;;
      1) seq 1 "$(wc -l < "$file")" ;;
      *)
        echo "  判不了 $file 跟踪没有：git 退出码 $tracked_rc" >&2
        return 2 ;;
    esac
  }

  flagged=0
  open_checked=0      # 查过的未定项条数（新增了已定小节的那几份里）
  pending_checked=0   # 查过的索引页「待议」节数

  # ── ① 同文件里还开着、而本次一个都没碰的未定项 ──────────────────
  for f in "${settled_files[@]}"; do
    # 本次 diff 在这个文件里碰过的行号（新文件侧）；未跟踪的新文件整份都算碰过
    touched="$(touched_lines_of "$f")" || {
      echo "  ✗ 判不了 $f 有没有被 git 跟踪（上面是 git 的退出码），这一份的未定项这一轮没比"
      echo "     → 怎么办：按 git 的报错修好仓库状态（索引坏了就 git status 看、必要时重建索引）再跑；判不了不是「未跟踪、整份都碰过」。"
      exit 1
    }
    while IFS=: read -r ln text; do
      [[ -n "$ln" ]] || continue
      open_checked=$((open_checked + 1))
      # 条目块：表格行一行就是一条，列表项到下一条分项（编号列表项或「| k |」表格行）或任一级标题之前（与 OPEN_ITEM_REVIEW_SOURCE 同一个口径）
      end=$(awk -v s="$ln" 'NR>s && (/^[[:space:]]*[0-9]+\. /||/^\|[[:space:]]*[0-9]+[[:space:]]*\|/||/^##+ /){print NR-1; exit}' "$f")
      [[ -n "$end" ]] || end=$((ln+20))
      # 这个条目块里有没有任何一行在本次 diff 里被碰过
      hit=0
      for t in $touched; do (( t>=ln && t<=end )) && { hit=1; break; }; done
      (( hit )) && continue
      echo "  ✗ $(basename "$f"):$ln 本次新增了「已定」小节，而这条未定项一个字都没动"   # gate-lint:detail
      echo "     ⇒ 复核它是不是被这次定案顺带答掉了。原文：${text:0:60}"
      flagged=1
    done < <(awk '
        /^### 未定项[[:space:]]*$/    { inside=1; next }
        /^### 还要回答的[[:space:]]*$/ { inside=1; next }
        /^### /                      { inside=0 }
        /^#### /                     { inside=0 }
        /^## /                       { inside=0 }
        inside && /^[[:space:]]*[0-9]+\. |^\| [0-9]+ \|/ { print NR":"$0 }
      ' "$f")
    # ⚠️ 滤器已撤：已定的分项住「### 已定项」小节，这一节按定义全是未定的。
    # 老版本按行内关键字滤，会把「正文里提到别处已定」的未定分项一起滤掉。
  done

  # ── ② 索引页里还悬着、而本次一个都没碰的「待议」节 ──────────────
  if [[ -f "$IDX" ]]; then
    # 按这一节判「动没动」：索引页别处（索引表的一行）改了不算回头看过这一节待议
    idx_touched_lines=""
    if grep -qxF "$IDX" <<<"$all_changed"; then
      idx_touched_lines="$(touched_lines_of "$IDX")" || {
        echo "  ✗ 判不了 $IDX 有没有被 git 跟踪（上面是 git 的退出码），索引页的待议节这一轮没比"
        echo "     → 怎么办：按 git 的报错修好仓库状态再跑；判不了不是「整份都碰过」。"
        exit 1
      }
    fi
    while IFS=: read -r ln text; do
      [[ -n "$ln" ]] || continue
      grep -qE '已回收|已收摊|已并入' <<<"$text" && continue
      pending_checked=$((pending_checked + 1))
      section_end=$(awk -v s="$ln" 'NR>s && /^## /{print NR-1; exit}' "$IDX")
      [[ -n "$section_end" ]] || section_end=$(wc -l < "$IDX")
      section_touched=0
      for t in $idx_touched_lines; do (( t>=ln && t<=section_end )) && { section_touched=1; break; }; done
      (( section_touched )) && continue
      echo "  ✗ $(basename "$IDX"):$ln 本次有决策定案，而这一节「待议」一个字都没动"
      echo "     ⇒ 复核它是不是被这次定案实质回答/否决了。原文：${text:0:60}"
      flagged=1
    done < <(grep -nE '^## 待议' "$IDX")
  fi

  if ((flagged)); then
    echo "     → 怎么办：被顺带定掉的就改写成「已定，权威记录在 XX」或直接收摊；"
    echo "               仍然开着的就在条目里补一行「YYYY-MM-DD 复核过，仍然开着，因为 …」。"
    echo "               后一种做法本身就是这条检查要的东西——它要的是一次回头看，不是一次沉默。"
    exit 1
  fi
  if ((open_checked == 0 && pending_checked == 0)); then
    echo "  ! 本次无对象可判：新增了已定小节的 ${#settled_files[@]} 份决策里没有一条未定项，索引页也没有悬着的待议节，什么都没查"
    exit 77
  fi
  echo "  ✓ 本次定案之后，同文件的未定项与索引页的待议节都被回头看过（新增已定小节的决策 ${#settled_files[@]} 份，查了 $open_checked 条未定项、$pending_checked 节待议）"
)

# ════ 格「冻结层归属登记表」════
# 冻结层归属登记表：每层每个结构都有行、写「退出」的必是派生态、依据指得到已定分项。
#
# 还 checks-owed.md C45（四层图里每个结构的态别没有登记）可机检的那一半。
# 三条判据的权威原文在那一行的「怎么拦」列，清单从哪来写在
# `.claude/kb/freeze-layer-membership.md`「门禁判哪三条」，这一格照它判：
#   ① 表里要有行的对象，三份清单都现读、不在脚本里抄第二份：
#      - 层与独立冻结组件：D15 已定项 7 正文里 `第 N 层：` 的代码块与 `- ①/②/③ **…**` 的列表；
#        一棵树在其中两层各占一个结构——写着「key 编码」的那一层与写着「索引节点内部布局」的那一层，
#        两层的号从代码块里认，不写死；
#      - 树：`crates/singlefs-format/src/lib.rs` 的 `TREE_IDENTIFIER_*` 常量（`WATERMARK` 两个是水位、不是树）；
#      - 单元类：D18 已定项 11 登记表里首列是单个整数、类名不是「无效」「保留」的那几行。
#      每一层、每个组件至少一行；每棵树在 key 编码那层与节点内部布局那层各至少一行（结构列写着「树 ID <n>」）；
#      每个单元类码至少一行（结构列写着「码 <n>」）。漏一个判红。三份清单任一份读成 0 项也判红——扫到 0 项不是通过。
#   ② 「退不退出冻结」写「退出」的行，态别必须是「派生态」，别的都判红。
#   ③ 依据列点名的 `D<n>（简称） 已定项 k` 要在 `.claude/kb/decisions/` 里有那条决策、有第 k 条分项、
#      且索引表里是已定；归属与状态用格「分项引用状态」那份 `lib-item-ref-status.py`，不另抄一份。
#      依据列写「判不动」开头的行不判 ③。
#   表按 `<!-- freeze-layer-membership:table -->` 那一行定位（整行匹配，不用 in），表头五列逐字；
#   层列只认 D15 已定项 7 读出的那几个值加「没有条款」，态别列只认 权威态 / 派生态 / 判不动，
#   退出列只认 退出 / 不退出 / 判不动——写了别的字判红，不然一个错别字会让 ② 静默放行。
#   退不退出冻结写「判不动」的行 ② 判不到，成功那句逐个列出；态别写「判不动」而退不退出冻结已定的行另列一句，
#   两份清单都从表里现算，不无声跳过。
#
# 这一格管不了的：某一行的态别判得对不对（例：把记账树码 2 节点那行改成「派生态 + 退出」，三条都不红——
# ② 只判「退出 ⇒ 派生态」，这一行两格自洽，错在态别本身）、判不动的理由站不站得住、有没有一个仓里存在而
# 三份清单都没单列的结构（树表单元、实例表单元、容器索引三行不是任何一份清单里的独立一项，删掉哪一行这一格都不红）。
#
# 判别力：fixtures/doc-decisions.sh/freeze-layer-membership-red 的表漏了一棵树的 key 编码行、漏了组件 ② 与码 16 那两行、
# 一行写「权威态 + 退出」、依据指到不存在的分项（一处接在「已定项 5 / 98」后面）、指到未定项、指到不存在的决策
# （简称里套了一层括号）各有一处，必须判红；decision-documents-green 里 D13–D18 与 `.claude/kb/freeze-layer-membership.md` 是真表原样的一份（编号与分项号改成连号），必须判绿。
grid_freeze_layer_membership() (
  LIB="$LIB_ITEM_REF_STATUS"
  REGISTRY=.claude/kb/freeze-layer-membership.md
  TREES=crates/singlefs-format/src/lib.rs
  SELF="$STAGE_RELATIVE_PATH --check freeze-layer-membership"
  MARKER='<!-- freeze-layer-membership:table -->'

  layers_decision="$(compgen -G '.claude/kb/decisions/15-*.md' || true)"
  units_decision="$(compgen -G '.claude/kb/decisions/18-*.md' || true)"
  if [[ -z "$layers_decision" ]]; then
    echo "  ⊘ 本次未跑：.claude/kb/decisions/ 下没有 D15，没有四层图就无对象可判"
    exit 77
  fi

  # 2>&1 要写在这一行：放在 heredoc 结束符后面单独一行是一条空命令，会把 python 的退出码盖成 0。
  report="$(python3 - "$LIB" "$REGISTRY" "$layers_decision" "$units_decision" "$TREES" 2>&1 <<'PY'
import importlib.util, os, re, sys

lib_path, registry_path, layers_path, units_path, trees_path = sys.argv[1:6]
MARKER = "<!-- freeze-layer-membership:table -->"
HEADER = "| 结构 | 落哪一层 | 态别 | 退不退出冻结 | 依据 |"
CIRCLED = "①②③④⑤⑥⑦⑧⑨⑩⑪⑫⑬⑭⑮⑯⑰⑱⑲⑳"
STATES = ("权威态", "派生态", "判不动")
EXITS = ("退出", "不退出", "判不动")

def bad(kind, message):
    print("BAD", kind, message, sep="\t")

def section(path, heading_prefix):
    """从 `#### <heading_prefix>` 起到下一个任意级标题为止的正文；找不到返回 None。"""
    if not os.path.isfile(path):
        return None
    lines = open(path, encoding="utf-8").read().split("\n")
    start = next((index for index, line in enumerate(lines) if line.startswith(heading_prefix)), None)
    if start is None:
        return None
    body = []
    for line in lines[start + 1:]:
        if re.match(r"^#{1,4}\s", line):
            break
        body.append(line)
    return body

def strip_trailing_parenthetical(name):
    """去掉结构名末尾那一组配平的全角括号，只留给人认的那一段。"""
    if not name.endswith("）"):
        return name
    depth = 0
    for index in range(len(name) - 1, -1, -1):
        if name[index] == "）":
            depth += 1
        elif name[index] == "（":
            depth -= 1
            if depth == 0:
                return name[:index].strip() or name
    return name

# ── 清单一：D15 已定项 7 的层与组件 ────────────────────────
layers, components, key_layer, node_layer = [], [], None, None
body = section(layers_path, "#### 已定项 7")
if body is None:
    bad("清单", f"{layers_path} 里没有「#### 已定项 7」这一节，读不出四层图")
else:
    for line in body:
        hit = re.match(r"^第 (\d+) 层：(.*)$", line)
        if hit:
            number = int(hit.group(1))
            layers.append(number)
            if "key 编码" in hit.group(2):
                key_layer = number
            if "索引节点内部布局" in hit.group(2):
                node_layer = number
        hit = re.match(r"^- ([%s]) \*\*" % CIRCLED, line)
        if hit:
            components.append(hit.group(1))
    if not layers:
        bad("清单", f"{layers_path} 已定项 7 的代码块里一行「第 N 层：」都没读到")
    if not components:
        bad("清单", f"{layers_path} 已定项 7 里一行「- ① **…**」的独立冻结组件都没读到")
    if layers and key_layer is None:
        bad("清单", f"{layers_path} 已定项 7 的层图里找不到写着「key 编码」的那一层，一棵树该在哪层登记 key 编码认不出")
    if layers and node_layer is None:
        bad("清单", f"{layers_path} 已定项 7 的层图里找不到写着「索引节点内部布局」的那一层，一棵树该在哪层登记节点内部布局认不出")

# ── 清单二：lib.rs 的树 ID 常量 ────────────────────────
trees = []
if not os.path.isfile(trees_path):
    bad("清单", f"没有 {trees_path}，读不出树 ID 常量")
else:
    for line in open(trees_path, encoding="utf-8"):
        hit = re.match(r"^pub const TREE_IDENTIFIER_([A-Z_]+): u64 = (\d+);", line)
        if hit and not hit.group(1).startswith("WATERMARK"):
            trees.append((hit.group(1), int(hit.group(2))))
    if not trees:
        bad("清单", f"{trees_path} 里一个 TREE_IDENTIFIER_* 常量都没读到（水位常量不算树）")

# ── 清单三：D18 已定项 11 登记表的单元类码 ────────────────────────
unit_classes = {}
body = section(units_path, "#### 已定项 11") if units_path else None
if body is None:
    bad("清单", f"{units_path or '.claude/kb/decisions/18-*.md'} 里没有「#### 已定项 11」这一节，读不出单元类登记表")
else:
    # 表在第一个非 | 行就结束，空行也算结束：下一张表只隔一个空行时，不这样切会把它并进来
    # （样本实测：偏移表的「| 42 | 单元类型标签 |」被当成了单元类码 42）。
    table, seen_table = [], False
    for line in body:
        if line.startswith("|"):
            table.append(line)
            seen_table = True
        elif seen_table:
            break
    if not table or table[0].split("|")[1].strip() != "码":
        bad("清单", f"{units_path} 已定项 11 之下第一张表的首列不是「码」，读不出单元类登记表")
    else:
        for row in table[2:]:
            cells = [cell.strip() for cell in row.strip().strip("|").split("|")]
            if len(cells) < 2 or not cells[0].isdecimal():
                continue
            name = cells[1].replace("*", "").strip()
            if "无效" in name or "保留" in name:
                continue
            unit_classes[int(cells[0])] = name
        if not unit_classes:
            bad("清单", f"{units_path} 已定项 11 登记表里一个登记了名字的单元类都没读到")

# ── 登记表本身 ────────────────────────
rows = []
if not os.path.isfile(registry_path):
    bad("表", f"没有 {registry_path}")
else:
    lines = open(registry_path, encoding="utf-8").read().split("\n")
    # 整行匹配，不用 in：正文里提到这个标记的句子不是标记行。
    marker_at = next((index for index, line in enumerate(lines) if line.strip() == MARKER), None)
    if marker_at is None:
        bad("表", f"{registry_path} 里没有单独成行的 {MARKER}")
    else:
        cursor = marker_at + 1
        while cursor < len(lines) and not lines[cursor].strip():
            cursor += 1
        table = []
        while cursor < len(lines) and lines[cursor].startswith("|"):
            table.append((cursor + 1, lines[cursor]))
            cursor += 1
        if not table:
            bad("表", f"{registry_path} 的标记行之后没有表")
        elif table[0][1].strip() != HEADER:
            bad("表", f"{registry_path}:{table[0][0]} 表头不是 {HEADER}，实际是 {table[0][1].strip()[:60]}")
        else:
            for line_number, row in table[2:]:
                cells = [cell.strip() for cell in re.split(r"(?<!\\)\|", row.strip().strip("|"))]
                if len(cells) != 5:
                    bad("表", f"{registry_path}:{line_number} 不是五格：{row.strip()[:60]}")
                    continue
                rows.append((line_number, cells))

allowed_layers = {f"第 {number} 层" for number in layers} | {f"组件 {mark}" for mark in components} | {"没有条款"}
for line_number, (structure, layer, state, exits, _basis) in rows:
    short = strip_trailing_parenthetical(structure)
    if layers and layer not in allowed_layers:
        bad("表", f"{registry_path}:{line_number}「{short}」的层写成「{layer}」，不在 D15 已定项 7 读出的层与组件里，也不是「没有条款」")
    if state not in STATES:
        bad("表", f"{registry_path}:{line_number}「{short}」的态别写成「{state}」，只认 权威态 / 派生态 / 判不动")
    if exits not in EXITS:
        bad("表", f"{registry_path}:{line_number}「{short}」的退不退出冻结写成「{exits}」，只认 退出 / 不退出 / 判不动")

# ── ① 每层每个结构都有行 ────────────────────────
def rows_with(predicate):
    return [(line_number, cells) for line_number, cells in rows if predicate(cells)]

for number in layers:
    if not rows_with(lambda cells: cells[1] == f"第 {number} 层"):
        bad("漏行", f"第 {number} 层没有一行")
for mark in components:
    if not rows_with(lambda cells: cells[1] == f"组件 {mark}"):
        bad("漏行", f"组件 {mark} 没有一行")
for name, identifier in trees:
    mentions = rows_with(lambda cells: re.search(rf"树 ID {identifier}(?!\d)", cells[0]) is not None)
    if key_layer is not None and not any(cells[1] == f"第 {key_layer} 层" for _, cells in mentions):
        bad("漏行", f"TREE_IDENTIFIER_{name}（树 ID {identifier}）在第 {key_layer} 层（key 编码）没有一行")
    if node_layer is not None and not any(cells[1] == f"第 {node_layer} 层" for _, cells in mentions):
        bad("漏行", f"TREE_IDENTIFIER_{name}（树 ID {identifier}）在第 {node_layer} 层（索引节点内部布局）没有一行")
for code, name in sorted(unit_classes.items()):
    if not rows_with(lambda cells: code in {int(found) for found in re.findall(r"码 (\d+)", cells[0])}):
        bad("漏行", f"码 {code}（{name}）没有一行")

# ── ② 写「退出」的行必须是派生态 ────────────────────────
for line_number, (structure, _layer, state, exits, _basis) in rows:
    if exits == "退出" and state != "派生态":
        bad("退出", f"{registry_path}:{line_number}「{strip_trailing_parenthetical(structure)}」写着退出冻结，态别却是「{state}」")

# ── ③ 依据列点名的分项在且已定 ────────────────────────
spec = importlib.util.spec_from_file_location("item_ref_status", lib_path)
item_ref_status = importlib.util.module_from_spec(spec)
spec.loader.exec_module(item_ref_status)
item_map, _names = item_ref_status.load_map()
basis_checked = 0
for line_number, (structure, _layer, _state, _exits, basis) in rows:
    short = strip_trailing_parenthetical(structure)
    # 简称里可以再套一层全角括号（D14（双轨（大小文件 / 持久临时）） 那种），[^）]* 会在里层的 ） 处断开、整处引用漏抓；
    # 一处引用可以接着写几个项号（「已定项 3 / 4」「已定项 3 / 已定项 4」），只取第一个会让后面的无声漏判。
    references = []
    for hit in re.finditer(r"(D\d+)（(?:[^（）]|（[^（）]*）)*）\s*(已定项|未定项)\s*(\d+)((?:\s*[/、]\s*(?:(?:已定项|未定项)\s*)?\d+)*)", basis):
        decision, written, first, tail = hit.groups()
        references.append((decision, written, first))
        for kind, number in re.findall(r"[/、]\s*(?:(已定项|未定项)\s*)?(\d+)", tail):
            references.append((decision, kind or written, number))
    if not references:
        if not basis.startswith("判不动"):
            bad("依据", f"{registry_path}:{line_number}「{short}」的依据没点名任何「D<n>（简称） 已定项 k」，也不是「判不动」：{basis[:40]}")
        continue
    for decision, written, item in references:
        basis_checked += 1
        item = int(item)
        if decision not in item_map:
            bad("依据", f"{registry_path}:{line_number}「{short}」的依据点名 {decision}，而 .claude/kb/decisions/ 里没有这条决策")
        elif item not in item_map[decision]:
            bad("依据", f"{registry_path}:{line_number}「{short}」的依据点名 {decision} 第 {item} 条分项，而那条决策的索引表里没有第 {item} 条")
        elif item_map[decision][item] != "已":
            bad("依据", f"{registry_path}:{line_number}「{short}」的依据点名 {decision} 第 {item} 条分项，而它在索引表里是未定项——依据只能是已定项")
        elif written != "已定项":
            bad("依据", f"{registry_path}:{line_number}「{short}」的依据把 {decision} 第 {item} 条写成「{written}」，而它是已定项")

# ── 没判的：② 判不到的行，与态别没判出来而冻结归属已定的行 ────────────────────────
exit_undetermined = [strip_trailing_parenthetical(cells[0]) for _, cells in rows if cells[3] == "判不动"]
state_only_undetermined = [f"{strip_trailing_parenthetical(cells[0])}（{cells[3]}）" for _, cells in rows
                           if cells[2] == "判不动" and cells[3] != "判不动"]
for name in exit_undetermined:
    print("SKIP", name, sep="\t")
for name in state_only_undetermined:
    print("STATEONLY", name, sep="\t")
print("COUNT", len(rows), len(layers), len(components), len(trees), len(unit_classes), basis_checked,
      len(exit_undetermined), len(state_only_undetermined), sep="\t")
PY
  )"
  python_rc=$?

  if ((python_rc != 0)) || ! grep -q '^COUNT' <<<"$report"; then
    echo "  ✗ 扫描没跑完：内嵌 python 没报出 COUNT 那一行（退出码 ${python_rc}，多半是它自己崩了）"
    grep -v '^\(BAD\|SKIP\|COUNT\)' <<<"$report" | sed 's/^/      /'   # gate-lint:detail
    echo "    → 怎么办：直接跑 bash ${SELF} 看 python 的报错；⚠️ 别把这一步当通过——"
    echo "      没有 COUNT 就是一行都没判，而汇总行看着与判过了一模一样。"
    exit 1
  fi

  failed=0
  for kind in 清单 表 漏行 退出 依据; do
    mapfile -t hits < <(awk -F'\t' -v kind="$kind" '$1 == "BAD" && $2 == kind { print $3 }' <<<"$report")
    ((${#hits[@]})) || continue
    failed=1
    case "$kind" in
      清单)
        echo "  ✗ 判据要读的清单有 ${#hits[@]} 份读不出来，这一格什么也没在判："   # gate-lint:summary
        printf '      %s\n' "${hits[@]}"   # gate-lint:detail
        echo "    → 怎么办：三份清单是 D15 已定项 7 正文（代码块里「第 N 层：」与列表里「- ① **…**」）、"
        echo "      crates/singlefs-format/src/lib.rs 的 TREE_IDENTIFIER_* 常量、D18 已定项 11 之下首列是「码」的登记表。"
        echo "      哪一份的形态改了，就改这一格去跟上它，别在脚本里抄一份清单——抄了就与条文分叉。"
        ;;
      表)
        echo "  ✗ 登记表 ${REGISTRY} 有 ${#hits[@]} 处不合形态："   # gate-lint:summary
        printf '      %s\n' "${hits[@]}"   # gate-lint:detail
        echo "    → 怎么办：表前单独一行 ${MARKER}，表头逐字"
        echo "      | 结构 | 落哪一层 | 态别 | 退不退出冻结 | 依据 |；层写「第 N 层」「组件 ①」或「没有条款」，"
        echo "      态别写 权威态 / 派生态 / 判不动，退出列写 退出 / 不退出 / 判不动。形态见 .claude/kb/freeze-layer-membership.md 开头几段。"
        ;;
      漏行)
        echo "  ✗ ① 有 ${#hits[@]} 个层或结构在登记表里没有行："   # gate-lint:summary
        printf '      %s\n' "${hits[@]}"   # gate-lint:detail
        echo "    → 怎么办：给每一个补一行（结构 / 落哪一层 / 态别 / 退不退出冻结 / 依据），态别按 D21 已定项 10 的判据推，"
        echo "      推不出就写「判不动」并在表下「判不动的九行，各卡在哪一句」里写清卡在哪一句；"
        echo "      树的一行结构列要写「树 ID <n>」，单元类的一行要写「码 <n>」，这一格按这两个字样认行。"
        ;;
      退出)
        echo "  ✗ ② 有 ${#hits[@]} 行写着退出冻结而态别不是派生态："   # gate-lint:summary
        printf '      %s\n' "${hits[@]}"   # gate-lint:detail
        echo "    → 怎么办：D21 已定项 9 那张表说权威态「参与格式冻结，是永久契约」、派生态「不参与格式冻结」；"
        echo "      两列只能一起改：要么把态别改回派生态（依据要跟着指到判它派生态的那条分项），要么把退出列改成「不退出」。"
        ;;
      依据)
        echo "  ✗ ③ 有 ${#hits[@]} 处依据指不到一条已定分项："   # gate-lint:summary
        printf '      %s\n' "${hits[@]}"   # gate-lint:detail
        echo "    → 怎么办：依据写成「D<n>（简称） 已定项 k」，那条决策要在 .claude/kb/decisions/ 里、索引表里第 k 条要是已定；"
        echo "      分项还没定就不能拿它当依据——这一行的态别改成「判不动」，依据写「判不动，推导见「判不动的九行，各卡在哪一句」」。"
        ;;
    esac
  done
  ((failed)) && exit 1

  read -r _ row_count layer_count component_count tree_count unit_count basis_count skipped_count state_only_count < <(grep '^COUNT' <<<"$report")
  mapfile -t skipped < <(awk -F'\t' '$1 == "SKIP" { print $2 }' <<<"$report")
  mapfile -t state_only < <(awk -F'\t' '$1 == "STATEONLY" { print $2 }' <<<"$report")
  echo "  ✓ 冻结层归属登记表判过了（$((row_count)) 行：D15 已定项 7 的 ${layer_count} 层与 ${component_count} 个组件、lib.rs 的 ${tree_count} 棵树、D18 已定项 11 的 ${unit_count} 个单元类都有行；依据点名的 ${basis_count} 处分项都在且已定）"
  if ((skipped_count)); then
    echo "    ② 没判的 ${skipped_count} 行（退不退出冻结写「判不动」）：$(IFS='；'; echo "${skipped[*]}")"
  else
    echo "    ② 每一行都判了，没有退不退出冻结写「判不动」的行"
  fi
  if ((state_only_count)); then
    echo "    另有 ${state_only_count} 行态别写「判不动」而退不退出冻结已定（括号里是退出列）：$(IFS='；'; echo "${state_only[*]}")"
  fi
  exit 0
)

# ════ 格「决策分项清单与正文同步」════
# `decisions.md` 的分项清单生成块与索引表「状态」列，都要与生成器从各决策正文算出来的逐字相同。
#
# `decisions.md` 里的「分项清单」和索引表的「状态」列，都是各决策正文的**投影**，
# 不是第二处权威记录。手抄一份就会漂，而漂了没有任何东西会发现——
# 这一格拿生成器 `.claude/scripts/gen-decision-items.py` 的输出与索引页逐字比对，两处都比。
#
# 「状态」列写的是**分项计数**（`已定 6 项 / 未定 0 项`）：一条决策有几个分项、
# 其中几个还没定，看一眼就有数。三态词（已定 / 半定 / 待定）的权威记录是各正文首行
# `## D<n> 简称 —— 状态`，索引列不再抄它一遍；没有分项的决策那一格写「无分项 · 整条已定」。
#
# 为什么判红而不是提醒：`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 4 条
# 「矛盾比空白更糟」——检索不会把两条都端出来，它会挑一条，而且不告诉你它挑了。
#
# ⚠️ **计数对不对，另有一条不经过生成器的检查**：格「kb 形状」第 7 段
# （本文件的 INDEX_VS_BODY_SOURCE）直接去数正文两节里的分项，不调生成器——两格同住一个文件，判法仍是两段各自的代码。
# 只有这一格的话，生成器自己数错时索引页会跟着一起错，而两边仍然逐字相同。
#
# 带 --write 时先写这一块（格表次序在 status-sync 前面，status-sync 抄的是这里写好的索引表）：重新生成分项清单、改写索引表的「状态」列，写回 `decisions.md`（给了项目根写那个仓，不给写本脚本所在的仓）：
#   bash .claude/gate.d/doc-decisions.sh --write [项目根]
# 判别力：fixtures/doc-decisions.sh/decision-items-sync-red 状态列写错、索引漏登、索引多一行、生成块漂了各一处，必须判红。
grid_decision_items_sync() (
  GEN="$GENERATOR"
  WRITE="$DECISION_ITEMS_WRITE"
  IDX=.claude/kb/decisions.md
  S='<!-- gen:decision-items:start -->'
  E='<!-- gen:decision-items:end -->'
  [[ -f "$GEN" ]] || { echo "  ✗ 找不到生成器 $GEN"; echo "     → 生成器随门禁住在同一个仓的 .claude/scripts/ 下：它丢了这一格什么都判不了，从 git 里找回它"; exit 1; }
  [[ -f "$IDX" ]] || { echo "  ! 找不到 $IDX，这一格无对象可判"; exit 77; }
  grep -qF "$S" "$IDX" || { echo "  ✗ $IDX 里没有生成块标记 $S"; echo "     → 加回标记，或跑 --write 重建"; exit 1; }

  gen_err="$(mktemp)"
  cells_f="$(mktemp)"
  # 各分支手写的 rm 管不到半路被打断的那一次：退出时一律删
  trap 'rm -f "${gen_err:?}" "${cells_f:?}"' EXIT
  # 生成器自己会说清哪一节取不到编号项、下一步怎么办 —— 吞掉 stderr 等于把那条 howto 扔了
  want="$(python3 "$GEN" 2>"$gen_err")" || {
    echo "  ✗ 生成器跑不起来"
    sed 's/^/   /' "$gen_err"
    echo "     → 怎么办：按上面这段报错改 $GEN 或它读的那几份 kb；"
    echo "               生成器跑不起来时这一格什么都没查，不是通过。"
    rm -f "$gen_err" "$cells_f"; exit 1; }
  python3 "$GEN" --status-cells >"$cells_f" 2>"$gen_err" || {
    echo "  ✗ 生成器的 --status-cells 跑不起来"
    sed 's/^/   /' "$gen_err"
    echo "     → 怎么办：同上——状态列没算出来时，这一半什么都没查，不是通过。"
    rm -f "$gen_err" "$cells_f"; exit 1; }
  rm -f "$gen_err"
  got="$(awk -v s="$S" -v e="$E" 'index($0,s){f=1;next} index($0,e){f=0} f' "$IDX")"

  fail=0

  if [[ "$WRITE" == "1" ]]; then
    python3 - "$IDX" "$S" "$E" "$GEN" <<'PY'
import sys, subprocess
idx, s, e, gen = sys.argv[1:5]
body = open(idx, encoding='utf-8').read()
new = subprocess.run(['python3', gen], capture_output=True, text=True).stdout.rstrip()
a = body.index(s) + len(s)
b = body.index(e)
open(idx, 'w', encoding='utf-8').write(body[:a] + "\n" + new + "\n" + body[b:])
PY
  fi

  # ── 索引表「状态」列 ────────────────────────────────────────────
  # 表格的形状是 `| D<n>（简称） | 状态 | 结论（简报） | 正文 |`，本段只碰第 2 列。
  # 整行重写会把别的会话刚写进结论列的东西悄悄抹掉（并发会话共写一个仓）。
  python3 - "$IDX" "$([[ $WRITE == 1 ]] && echo write || echo check)" "$cells_f" <<'PY'
import re, sys

idx_path, mode, cells_path = sys.argv[1:4]
text = open(idx_path, encoding='utf-8').read()
want = dict(l.split('\t', 1) for l in
            open(cells_path, encoding='utf-8').read().splitlines() if l.strip())

bad, n, wrote = [], 0, 0
for num, cell in want.items():
    pat = re.compile(r'^(\|\s*%s（[^|]*）\s*\|)([^|]*)\|' % re.escape(num), re.M)
    m = pat.search(text)
    if not m:
        bad.append(f"{num} 有正文，而决策索引表里没有它的行")
        continue
    n += 1
    if m.group(2).strip() == cell:
        continue
    if mode == 'write':
        text = text[:m.start(2)] + f" {cell} " + text[m.end(2):]
        wrote += 1
    else:
        bad.append(f"{num} 状态列写着「{m.group(2).strip()}」，正文投影是「{cell}」")

# 反过来也要查：索引表里多出一行（决策正文已经删了/改名了），上面的循环够不着它。
for row in re.findall(r'^\|\s*(D\d+)（', text, re.M):
    if row not in want:
        bad.append(f"{row} 在决策索引表里有行，而 decisions/ 下没有它的正文")

if mode == 'write':
    open(idx_path, 'w', encoding='utf-8').write(text)

if bad:
    print(f"  ✗ 决策索引表的状态列与正文不同步（{len(bad)} 处）")   # gate-lint:summary
    for b in bad:
        print("     " + b)                                          # gate-lint:detail
    print("     → 怎么办：权威记录是各决策正文的「### 已定项」/「### 未定项」两节。")
    print("       改完正文跑： bash .claude/gate.d/doc-decisions.sh --write")
    sys.exit(1)

print(f"  ✓ 决策索引表状态列与正文同步（{n} 条决策"
      + (f"，写回 {wrote} 行）" if mode == 'write' else "）"))
PY
  [[ $? -ne 0 ]] && fail=1
  rm -f "$cells_f"

  if [[ "$WRITE" == "1" ]]; then
    echo "  ✓ 已重新生成并写回 $IDX"
    exit $fail
  fi

  if [[ "$want" == "$got" ]]; then
    n=$(printf '%s\n' "$want" | grep -c '^  - ' || true)
    decision_lines=$(printf '%s\n' "$want" | grep -c '^- ' || true)
    # 生成器一条决策都没产出、生成块也是空的：两个空串相等，什么都没比过，不记通过
    if [[ "$decision_lines" -eq 0 && "$fail" -eq 0 ]]; then
      echo "  ! 本次无对象可判：生成器一条决策都没产出（decisions/ 下没有读得出的决策正文），生成块也是空的"
      exit 77
    fi
    echo "  ✓ 决策分项清单与正文同步（$n 个分项）"
    exit $fail
  fi
  echo "  ✗ 决策分项清单与正文不同步"
  diff <(printf '%s\n' "$got") <(printf '%s\n' "$want") | head -20 | sed 's/^/     /'
  echo "     → 权威记录是各决策正文。改完正文跑： bash .claude/gate.d/doc-decisions.sh --write"
  exit 1
)

# ════ 格「未定项判过改不改新池新建文件的字节」════
# 每个未定项有没有判过改不改新池新建文件的字节（另一把尺：动不动格式）。
#
# 还 checks-owed.md C50（阻塞标记没人维护）的**覆盖性**那一半。
#
# **它拦的不是「判错」，是「没判过」。** 一条未定项挡不挡第一行代码，此前只记在正文
# 某一句 ⚠️ 里，措辞还各不相同（「不阻塞第一行代码」「不阻塞第一行事务层代码」
# 「可延后」），于是**没判过与判过是否**在 kb 里长得一模一样——2026-09-01 现查：
# 23 条未定项里只有 3 条写过明确判定，另外 20 条从来没按那把尺量过，而没有任何东西发现。
#
# **它与格「未定项被别处定了」「定了新东西没回看同文件未定项」是三把不同的尺子，不是加强版**：
#   「未定项被别处定了」：跨文件 + 看历史——未定项点名了别的决策，而那个决策后来定过东西。
#   「定了新东西没回看同文件未定项」：同文件 + 看本次 diff——本次新增了「已定」小节，而同文件的未定项一个都没碰。
#   这一格：**不看历史也不看 diff**——每条未定项当下有没有一条判定，缺就红。
# 那两格管的是「判定会不会过期」，这一格管的是「判定存不存在」。
# 少了这一格，一条**从未判过**的未定项在那两格眼里是干净的：它没点名别人，本次也没人定案。
#
# **判定的规范形态**（写在该分项的登记行或它那一条列表条目里）：
#
#   改新池新建文件的字节：否（2026-09-01，依据：…）
#
# 三个合法取值：**是** / **否** / **无对象**（前置已被推翻，这一项没有对象了）。
#
# **那把尺的可执行形式**（不是新发明，是 decisions-history.md 的 D5（快照 / 空间记账机制） 节 2026-08-29 那一条逐字用过的那个）：
# 那一轮的依据写作「攻方逐条给出**新池新建文件写不出的是哪些字节**」⇒ 尺子问的不是
# 「两个答案会不会给出不同的字节」，而是——
#
#   **不定它，新池新建文件写出的字节能不能由已经定了的条款唯一确定？**
#   （**mkfs 写出的字节算在内**——事务必须落在一个格式化好的盘上，改 mkfs 的布局同样要返工。
#    这一格是 2026-09-01 那轮的本地腿当成尺子的洞攻出来的，主 agent 当轮裁定算在内，
#    它当场改掉一条判定：D12 未定项 3 的一个候选只在 mkfs 写。）
#     能   ⇒ 否
#     不能 ⇒ 是
#
# 这一步不能省，省了两条结构一样的分项会判出相反的结果：
# D18 未定项 7（块头要不要第六个字段）与 D5 未定项 1（维度元组有哪几维）都带着
# 「先按最小的写、以后再加」，但 D18 已定项 3 是一条**正式已定**的五字段集合，
# 新池新建文件的块头由它唯一确定 ⇒ 否；而 D5 没有任何已定项钉住维度元组的内容，
# 「先按最小维度」是口头方向不是登记的已定项 ⇒ 第一条记账记录的 key 宽度写不出来 ⇒ 是。
#
# ⚠️ **这个名字是刻意不叫「阻塞」的**，而仓里此前三处判定都写作「不阻塞第一行代码」——
# 那个词被同时用作两个意思：①「不改新池新建文件写出的字节」（D21 未定项 1：共享单元数为 0
# ⇒ 两个答案逐字节相同）；②「改，但已定案接受先写后改」（D5 未定项 1、D18 未定项 7）。
# **两者在盘上的后果完全不同**：① 不返工，② 要重写记账树 / 重排块头。
# 一个词罩住两件事，正是 decisions-history.md 的 D5（快照 / 空间记账机制） 节 2026-08-29 那一条记下的那次
# 「判据用得不一致」的病因。⇒ **标记只回答那把尺**（改不改字节），
# **挡不挡开工是处置，写在依据里**。
# 日期不许省——判定会因为别处定案而过期，没有日期就没法判它是哪一轮的产物
# （`.claude/singlefs-ai-sop/rules/kb-discipline.md`「每条带出处与状态」）。
#
# ⚠️ **只认这一种写法，不认同义的散文。** 散文形态是这条检查诞生的原因：
# 三条已判过的分项用了三种措辞，机器分不开「判过否」与「随口提了一句阻塞」。
#
# ⚠️ **「哪些是未定项」不自己解析**：调 `.claude/scripts/gen-decision-items.py`，
# 与格「决策分项清单与正文同步」同一个权威解析器。这一格自己定位登记行，所以另加一道**条数比对**——
# 两侧对不上就说明定位漏了或多了，判红而不是安静地少查几条。「多了」＝「### 未定项」索引里定位到、
# 而生成器不认它是未定项的行（例：表格行缺了收尾的竖线，生成器不收）。对不上与缺判定各报各的，都红。
#
# 判别力：fixtures/doc-decisions.sh/blocking-verdict-red 两把尺各缺、外加一行表格缺收尾竖线（多定位出），必须判红。
grid_blocking_verdict() (
  DEC=.claude/kb/decisions
  # 生成器按**本脚本自身的位置**取，不按 cwd——判别力样本会把 cwd 换成一个只放着样本决策文件的临时目录，
  # 那里没有 `.claude/scripts/`。按 cwd 取会「找不到生成器 ⇒ 跳过」，红样本安静地绿掉。生成器自己 glob 的是 cwd 下的 kb，正合样本所需。
  GEN="$GENERATOR"
  # 无对象可判退 77，门禁记「本次未跑」，不记通过（`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）
  [[ -d "$DEC" ]] || { echo "  ! 没有 $DEC，这一格无对象可判"; exit 77; }
  # 生成器随仓走，不在就是被删了或挪了——退 1 不退 77，与格「决策分项清单与正文同步」依赖同一个生成器时的退法一致（三方判决 gate-fix-forks-r1 的 T7）
  [[ -f "$GEN" ]] || { echo "  ✗ 找不到生成器 $GEN"; echo "     → 生成器随门禁住在同一个仓的 .claude/scripts/ 下：它丢了这一格什么都判不了，从 git 里找回它"; exit 1; }

  python3 - "$DEC" "$GEN" <<'PY'
import re, sys, glob, subprocess, os

dec, gen = sys.argv[1], sys.argv[2]

# ── 权威清单：哪些 (决策号, 分项号) 是未定的 ────────────────────
r = subprocess.run(['python3', gen], capture_output=True, text=True)
if r.returncode != 0:
    print("  ✗ 生成器跑不起来，拿不到未定项清单")
    print('\n'.join('   ' + l for l in r.stderr.splitlines()))
    print("     → 怎么办：按上面这段报错修生成器或它读的那几份 kb。")
    print("               它跑不起来时这一格一条未定项都没查过，不是通过。")
    sys.exit(1)
want = set()
cur = None
for line in r.stdout.splitlines():
    m = re.match(r'- \*\*(D\d+)（', line)
    if m:
        cur = m.group(1); continue
    m = re.match(r'  - (\d+)\. .* —— (.+)$', line)
    if m and cur and '未定' in m.group(2):
        want.add((cur, m.group(1)))

# 两把尺，同一批登记行上各判各的。**不许互相代替，也不许拿一把的结论去填另一格**：
# 一条未定项可以在「改新池新建文件的字节」上判「否」而在「动不动格式」上判「动」——
# D2（RAID 条带策略） 未定项 21 就是：第一版两盘恒 w=2 不写第二种字节，
# 而它的答案会改 `w_max` 那个字节将来的取值。
RULERS = [
    ("改新池新建文件的字节",
     re.compile(r'改新池新建文件的字节：\*{0,2}(是|否|无对象)\*{0,2}[（(](\d{4}-\d{2}-\d{2})'),
     "判过改不改新池新建文件的字节"),
    ("动不动格式",
     re.compile(r'动不动格式：\*{0,2}(动|不动|界不定)\*{0,2}[（(](\d{4}-\d{2}-\d{2})'),
     "写了「动不动格式」的判定"),
]

seen, located, bad = set(), set(), {}
for f in sorted(glob.glob(os.path.join(dec, '*.md'))):
    s = open(f, encoding='utf-8').read()
    mm = re.match(r'## (D\d+) ', s.split('\n', 1)[0])
    if not mm:
        continue
    dnum = mm.group(1)
    body = s.split('\n## 历史版本')[0]
    m = re.search(r'^### 未定项\s*$(.*?)(?=^#{1,3} |\Z)', body, re.M | re.S)
    if not m:
        continue
    sec = m.group(1)
    # 索引表 / 编号列表都只取第一个 `####` 之前那一段，与生成器同一口径
    cut = re.search(r'^#{4}\s', sec, re.M)
    top = sec[:cut.start()] if cut else sec
    # 每条分项的「块」= 它的登记行起，到下一条登记行为止（表格形态就是一行）
    starts = [(mo.start(), mo.group(1))
              for mo in re.finditer(r'^(?:\|\s*(\d+)\s*\||(?=\d))(\d+)?\.?\s', top, re.M)]
    marks = [(mo.start(), mo.group(1) or mo.group(2))
             for mo in re.finditer(r'^\|\s*(\d+)\s*\||^(\d+)\.\s', top, re.M)]
    for i, (pos, num) in enumerate(marks):
        end = marks[i + 1][0] if i + 1 < len(marks) else len(top)
        block = top[pos:end]
        located.add((dnum, num))
        if (dnum, num) not in want:
            continue          # 生成器不认它是未定项：解析漂了，下面按「多定位出」报
        seen.add((dnum, num))
        for name, pattern, _ in RULERS:
            if not pattern.search(block):
                first = block.strip().splitlines()[0]
                bad.setdefault(name, []).append((os.path.basename(f), dnum, num, first[:70]))

missed = want - seen
# seen 只收 want 里的，seen - want 恒为空；多出来的要拿「定位到的」去比
extra = located - want
if missed or extra:
    print("  ✗ 登记行定位与生成器对不上，这一格这一轮查的不是全部未定项")
    for d, n in sorted(missed):
        print(f"     定位不到：{d} 未定项 {n}")
    for d, n in sorted(extra):
        print(f"     多定位出：{d} 未定项 {n}（生成器不认它是未定项）")
    print("     → 「### 未定项」一节要有索引表（`| # | 分项 | 状态 |`，每行要有收尾的竖线）或顶格编号列表，每条分项一行")

if bad:
    total = sum(len(v) for v in bad.values())
    print(f"  ✗ {total} 条未定项缺判定（两把尺分开数）：")          # gate-lint:summary
    for name, rows in bad.items():
        print(f"     ── 缺「{name}」：{len(rows)} 条")               # gate-lint:detail
        for fn, d, n, first in rows:
            print(f"        {fn}  {d} 未定项 {n}：{first}")          # gate-lint:detail
    if "改新池新建文件的字节" in bad:
        print("     → 缺「改新池新建文件的字节」怎么办：按「改不改变新池新建文件写出的字节」这把尺判一次，")
        print("               在该分项的登记行里写：改新池新建文件的字节：否（YYYY-MM-DD，依据：…）")
        print("               三个合法取值：是 / 否 / 无对象。**两侧要用同一把尺**——")
        print("               判「不阻塞」用这把尺、判「阻塞」换一把，量出来的集合不是包含关系")
        print("               （decisions-history.md 的 D5（快照 / 空间记账机制） 节 2026-08-29 那一条实测）。")
    if "动不动格式" in bad:
        print("     → 缺「动不动格式」怎么办：按 D15（格式冻结政策） 已定项 1 的尺判一次——它的答案")
        print("               会不会改动任何盘上字节，或改动已有字节的解释口径（宽度不变而含义变了）；")
        print("               在该分项的登记行里写：动不动格式：动（YYYY-MM-DD，依据：…）")
        print("               三个合法取值：动 / 不动 / 界不定（视同动）。⚠️ 别拿上面那把尺的结论来填：")
        print("               那把量「这一版写不写出不同字节」，这把量「将来动不动格式」，")
        print("               两把量的不是一个集合。")
if bad or missed or extra:
    sys.exit(1)

if not seen:
    print("  ! 没有一条未定项，这一格无对象可判")
    sys.exit(77)
print(f"  ✓ {len(seen)} 条未定项两把尺都判过："
      + "、".join(f"{name}" for name, _, _ in RULERS))
PY
)

# ════ 格「动了用户定案的条款记了未还的账」════
# 决策正文里标着「待用户复核」的，欠账表里要有一笔未还的账点名那条决策。
#
# 判据（`.claude/singlefs-ai-sop/skills/decide/SKILL.md` 硬要求第 6 条）：
# 决策正文里凡标着「待用户复核」的条款，`checks-owed.md` 里必须有一笔**未还**的账点名它。
#
# ⚠️ **这条是实测出来的**：2026-09-05 C113 定案顺手改了 D16 已定项 5 的丢失窗口口径，
# 而那一句是 2026-08-31 由用户拍板的。正文里写了「待用户复核」，然后就没有任何东西
# 保证它会被看到——唯一提到它的那笔账（C113）当天就标了「已还」。
# 拍板的人不知道自己的定案变了，而门禁全绿。
#
# 账点名决策按整个号认（D1 不算点名了 D16）。粒度是决策：一份决策正文里标了几处，都只要一笔未还的账点名这条决策。
# 判「未还」看账那一行有没有带日期的还清标记（`已还（2026-…`、`已还一半（…`）：
# 已还的账不会再被回看，等于没有账。要留着盯，就单立一笔。
# 不按「已还」两字判——那两个字出现在描述里就会把整笔账误判成还清（写这条时实测）。
#
# 判别力：fixtures/doc-decisions.sh/user-verdict-owed-red 里 D316 只有一笔已还的账、D31 只有一笔点名 D319 的未还账（按整号认不算点名 D31），必须判红。
grid_user_verdict_owed() (
  KB="$PWD/.claude/kb"
  OWED="$KB/checks-owed.md"
  MARK='待用户复核\|等用户复核'

  [[ -d "$KB/decisions" ]] || { echo "  ! 找不到 $KB/decisions，这一格跳过"; exit 77; }
  [[ -f "$OWED" ]] || { echo "  ✗ 缺 $OWED"
    echo "     → 怎么办： 待复核的条款要有账本盯着。先建 checks-owed.md，再把这一笔记进去。"; exit 1; }

  fail=0; checked=0; marked_lines=0
  while IFS= read -r f; do
    grep -q "$MARK" "$f" || continue
    marked_lines=$((marked_lines + $(grep -c "$MARK" "$f")))
    # 决策号取自文件名：`16-发布语义.md` → D16
    d="D$(basename "$f" | sed 's/^0*//; s/-.*//')"
    checked=$((checked+1))
    # 未还的账：同一行里点名了这条决策、带着复核标记，且没标「已还」
    # 末段不许是 `grep -q`：pipefail 下它一命中就退出，前段吃 SIGPIPE ⇒ 命中被读成没命中
    # （shell-lint 的 S7 判这一条：.claude/singlefs-ai-sop/scripts/shell-lint.sh）。先落到变量再判。
    # 决策号按整个号认：D1 不许靠子串命中 D10–D19 的账
    hits=$(grep "$MARK" "$OWED" | grep -E "${d}([^0-9]|\$)" || true)
    if [[ -n "$hits" ]] && grep -qv "已还[^ |]*（20" <<<"$hits"; then continue; fi
    echo "  ✗ $d 的正文标着待用户复核，checks-owed.md 里却没有一笔**未还**的账点名它"
    echo "        正文：$(basename "$f")"
    echo "     → 怎么办： 在 checks-owed.md 立一笔账，点名是 $d 的哪一项、原定案是哪天由谁拍的、"
    echo "                复核要回答什么问题；复核过了再标已还。"
    echo "                只在正文写一句「待用户复核」，没有任何东西保证它会被看到"
    echo "                （decide skill 硬要求第 6 条）。"
    fail=$((fail+1))
  done < <(find "$KB/decisions" -name '*.md' | sort)

  if [[ $fail -gt 0 ]]; then
    echo "  ✗ $fail 份决策正文标着待用户复核、却没有未还的账（共查 $checked 份）"
    echo "     → 怎么办：每一处「待用户复核」都要在 kb/checks-owed.md 里有一条未还的账盯着，"
    echo "               否则那句「待复核」没有任何人会回头看。补上那条账，或者"
    echo "               这一条已经复核完了就把「待用户复核」改成结论。"
    exit 1
  fi
  # 一处「待用户复核」都没有，这一轮什么都没判：退 77，门禁记「本次未跑」，不记通过
  # （`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）。
  if [[ $checked -eq 0 ]]; then
    echo "  ! 决策正文里没有一处标着待用户复核，这一格无对象可判"
    exit 77
  fi
  echo "  ✓ 待用户复核的条款都有未还的账盯着（$checked 份决策正文、$marked_lines 行标着待用户复核；按决策判，一份决策有一笔未还的账点名它就算有）"
)

# ════ 格「决策索引结论列的宽度」════
# 决策索引表每一行的「结论」列不超过 decisions.md 那一句写的字数上限。
#
# `decisions.md` 的索引表是检索这批决策的入口，「结论」列只写那条决策**定了什么**。
# 表头上写着一个字数上限，而这个上限此前没有任何东西在执行——实测（2026-09-07）
# 七行超标、最长 96 字，当时写的限是 50。一条只写在文档里、没人执行的规约，
# 下一个人会照着最长的那行写，索引就退化成第二份正文，导航价值归零。
#
# 上限**从 decisions.md 那一行读**，不写死在这里：同一个事实只许一处权威记录
# （`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 4 条）。改限只改文档那一行，
# 这一格跟着走；两边各写各的，就会一处改对、一处停在旧值。
#
# 判别力：fixtures/doc-decisions.sh/decision-summary-width-red 两行超限（一行结论里带转义竖线），必须判红。
grid_decision_summary_width() (
  IDX=.claude/kb/decisions.md
  [[ -f "$IDX" ]] || { echo "  ! 找不到 $IDX，这一格跳过"; exit 77; }

  python3 - "$IDX" <<'PY'
import re, sys

path = sys.argv[1]
text = open(path, encoding="utf-8").read()

m = re.search(r"「结论」列一律\s*(\d+)\s*字以内", text)
if not m:
    print("  ✗ decisions.md 的索引表说明里找不到「「结论」列一律 N 字以内」")   # gate-lint:summary
    print("  → 怎么办：上限的权威记录是那一句，这一格从它读。把它写回索引表的说明里，")
    print("    例：⚠️ **「结论」列一律 200 字以内**，写那条决策定了什么。")
    sys.exit(1)
limit = int(m.group(1))

rows, bad = [], []
for line in text.splitlines():
    if not re.match(r"^\|\s*D\d+（", line):
        continue
    # 按没转义的竖线切格：结论里写的 `\|` 是格子里的字，不是格界（与格「冻结层归属登记表」同一种切法）
    cells = [c.strip() for c in re.split(r"(?<!\\)\|", line.strip().strip("|"))]
    if len(cells) < 4:
        continue
    name = cells[0]
    concl = re.sub(r"\s+", " ", cells[2])
    rows.append(name)
    if not concl:
        bad.append(f"{name} 的结论列是空的")
    elif len(concl) > limit:
        bad.append(f"{name} 的结论列 {len(concl)} 字，超出上限 {limit} 字：{concl[:36]}…")

if not rows:
    print("  ✗ decisions.md 里一行 `| D<n>（简称） | 状态 | 结论 | 正文 |` 都没扫到")   # gate-lint:summary
    print("  → 怎么办：索引表的行首形态是 `| D1（数据可移动性 / 反向索引） | …`。")
    print("    改过表格形状就同步改这一格的匹配，别让它扫到 0 行还报绿。")
    sys.exit(1)

if bad:
    print(f"  ✗ 决策索引结论列有 {len(bad)} 行超出上限（限 {limit} 字）：")   # gate-lint:summary
    for b in bad:
        print("      " + b)                                                  # gate-lint:detail
    print("  → 怎么办：结论列只写那条决策**定了什么**，为什么定、还欠什么一律搬进 decisions/ 下的正文。")
    print("    删不动就说明这条决策的主结论本身没收敛——那要么改它的状态，要么把分项拆开写。")
    sys.exit(1)

print(f"  ✓ 决策索引结论列都不超 {limit} 字（共 {len(rows)} 行）")
PY
)

# ════ 格「同一结构的写频率只有一个量级」════
# C245（系统配置槽的写频率，两条条款说反话）的门禁形态：同一个结构的写频率在两处写成不同的量级，判红。
# 登记表 .claude/gate.d/write-frequency.tsv 给出封闭的频率词表（每个词归一个量级，可另记一个量）与每个结构的叫法、登记频率、权威分项；
# 排除表 .claude/gate.d/write-frequency-exclude.tsv 逐句登记确实该留的判红句子。两张表的写法在各自的文件头。
#
# 扫什么：.claude/kb/decisions/*.md 在「## 历史版本」之前的正文，代码围栏里的行不扫；去掉 ** 与反引号，
#   每行按没转义的竖线切成格，每格按「。；！？」切成句。
# 断言怎么认：句子里出现频率词表里的一个词，后面同一个小句里（不越过「，。；：、」、括号与引号）12 字以内跟着写动词
#   （写、更新、轮换、推进、回写、重写、改写、覆写、落盘、涨；「写放大」不算），或者紧跟「一次」。
#   只出现频率词、后面不跟写动词的（「每次发布四个序点」「每次挂载现取」）不是写频率断言，不判。
# 归到哪个结构：句子里出现的每个登记叫法都算（一句点了两个结构，两个都判）；句子里一个都没有的，
#   认同一表格行别的格里的叫法；还没有的，认所属最近一个标题（任意级）里的叫法。主语只在标题里的表格行（「tail 住超级块槽」下面那张表）就这样认到。
# 三种形态，各按一把尺判（量级与量见登记表文件头：量是同一量级里的一部分时机，每次 fsync 都有一次发布、发布不都是 fsync，fsync 那几个词量记 fsync）：
#   否定：频率词前面有「不是 / 并非 / 而非 / 没有 / 无需 / 无须 / 不必 / 不用 / 不 / 没 / 非」，中间只隔三个以内的虚字
#     （都、是、在、随、按、会、再、需、要、必、用、应、该、能、可、就、也、于）与空白（「不会在每次发布时写」「不再每次发布推进」）；
#     或者频率词与写动词之间有「不 / 没 / 无需 / 无须」（「每次发布不写」；「不仅、不止、不只、不但、不同、不变、不论、不管」不算否定）。
#     否定的是别的量级（「不是每个事务推进一次」），是在划界，不算断言、只计数；否定的时机被登记的罩住（同量级，且登记的是整个量级或正是这个量），与登记说反话，判红。
#   限定：频率词前面紧挨「只 / 仅 / 只有」（可带「在 / 于」，「只在 fsync 时写」「仅 fsync 路径写」）。写的量与登记的量不同就判红——
#     同级不够：「只在 fsync 时写」与登记的「每次发布」同级而不同量，判红。前面是「不只 / 不仅 / 不是只」的（「不是只在 fsync 时写」）不划界，按否定形态计数、不判。
#   普通：其余的断言按量级比，不同级判红（登记的是量级里的一部分时机的，还要同量）。
# 判红：上面三种形态说反话；登记表写坏（行认不出、理由不足 4 个字、频率词重复、叫法登记给两个结构、
#   登记频率不在词表里）；权威分项不是「D<n>（简称） 已定项 k」、决策不在、简称与正文首行不符、那一条不是已定项（归属借 lib-item-ref-status.py）；
#   排除表一行指的文件不在、那一段不足 8 个字、或一句判红的断言都没排到；登记了结构而全部正文里一处落在它们身上的断言都没扫到。
# 看不见的：频率词表外的说法（「每 10 个 fsync」「只在准入之后」，要认就把那一段登记成词）、叫法表外的称呼；
#   跨句的指代（「它每次挂载写一次」）：这一句自己没有叫法时只认同一表格行与所属标题，前一句的主语不带过来；
#   否定词与频率词之间隔了虚字表外的字（「不影响每次发布写」本来就不是否定，「不一定每次发布都写」是否定而认不到）、双重否定；
#   叫法取自同一表格行或标题时，那一格或那一节说的是不是这个结构；句子说的是不是这个结构的稳态写频率。后两样靠排除表与人。
#   成功行末尾逐个列出一处断言都没扫到的结构，它们这一轮没判。
# 登记表不在：这一格无对象可判，退 77（样本树里多数没有它）。
#
# 判别力：fixtures/doc-decisions.sh/write-frequency-red 里系统配置槽写成「每次挂载写一次」、tail 写成「不是每次发布都推进」、
# 「系统配置槽不会在每次发布时写」「系统配置槽每次发布不写」「tail 不再每次发布推进」三句隔字与动词前的否定、「系统配置槽只在 fsync 时写」、
# C245 立账时 D23 那一行原句（主语只在标题「tail 住超级块槽」里，「不是每 fsync 一次」），
# 一个结构的权威分项是未定项、一个简称写错、排除表一行指不存在的文件、一行没排到判红句子，必须判红；
# write-frequency-green 里「每个 checkpoint 一次」「每次 fsync 写一次」与登记的「每次发布」同级、「只在 checkpoint 持久之后写一次」与登记同量、
# 「不是每个事务推进一次」「不是每次挂载才写」「不会在每次挂载时推进」「每个事务都不写」否定的是别的量级、「不仅每次发布推进」「不是只在 fsync 时写」不划界、
# 表格行的主语在别的格、一句的主语只在标题里、一句「每次挂载回写」登记在排除表里，必须判绿。
grid_write_frequency() (
  REGISTRY=.claude/gate.d/write-frequency.tsv
  EXCLUDE=.claude/gate.d/write-frequency-exclude.tsv
  [[ -d .claude/kb/decisions ]] || { echo "  ! 找不到 .claude/kb/decisions，这一格跳过"; exit 77; }
  [[ -f "$REGISTRY" ]] || { echo "  ! 找不到 $REGISTRY（写频率登记表），这一格无对象可判"; exit 77; }

  python3 - "$REGISTRY" "$EXCLUDE" "$LIB_ITEM_REF_STATUS" "$BREAK_SWITCH" <<'PY'
import glob, importlib.util, os, re, sys

registry_path, exclude_path, lib_path, break_switch = sys.argv[1:5]
levels_merged = break_switch == "write-frequency-levels-merged"                        # 弄坏开关：不同级一律判成同级
negation_counted = break_switch == "write-frequency-negation-asserts"                  # 弄坏开关：否定形态也算断言
subject_in_sentence_only = break_switch == "write-frequency-subject-in-sentence-only"  # 弄坏开关：叫法只从句内认
negation_adjacent_only = break_switch == "write-frequency-negation-adjacent-only"      # 弄坏开关：否定词只认紧挨频率词的
restriction_as_plain = break_switch == "write-frequency-restriction-as-plain"          # 弄坏开关：「只在 / 仅」限定形态当普通断言

problems = []   # (类别, 明细)


def bad(kind, detail):
    problems.append((kind, detail))


def phrase_pattern(phrase):
    # 登记里的空格是「中英文之间可有可无的空白」：「每次 fsync」认「每次fsync」「每次 fsync」
    return r"\s*".join(re.escape(piece) for piece in phrase.split(" ") if piece)


# ── 读登记表 ────────────────────────
frequency_level = {}      # 频率词 → 量级
frequency_quantity = {}   # 频率词 → 量（频率词行没写量的，量就是量级本身）
structures = []           # (行号, 结构名, [叫法], 登记频率词, 权威分项)
with open(registry_path, encoding="utf-8") as handle:
    for line_number, raw in enumerate(handle, 1):
        line = raw.rstrip("\n")
        if not line.strip() or line.startswith("#"):
            continue
        cells = line.split("\t")
        reason = cells[-1] if cells[-1].startswith("#") else ""
        if len(reason.lstrip("#").strip()) < 4:
            bad("登记表", f"{registry_path}:{line_number} 末列不是「# 理由」或理由不足 4 个字：{line[:60]}")
            continue
        if cells[0] == "频率词" and len(cells) in (4, 5):
            word, level = cells[1].strip(), cells[2].strip()
            quantity = cells[3].strip() if len(cells) == 5 else level
            if not word or not level or not quantity:
                bad("登记表", f"{registry_path}:{line_number} 频率词行的词、量级或量是空的")
            elif word in frequency_level:
                bad("登记表", f"{registry_path}:{line_number} 频率词「{word}」登记了两次")
            else:
                frequency_level[word] = level
                frequency_quantity[word] = quantity
        elif cells[0] == "结构" and len(cells) == 6:
            name, aliases, registered, authority = (cell.strip() for cell in cells[1:5])
            alias_list = [alias.strip() for alias in aliases.split("|") if alias.strip()]
            if not name or not alias_list or not registered or not authority:
                bad("登记表", f"{registry_path}:{line_number} 结构行有空列：{line[:60]}")
            else:
                structures.append((line_number, name, alias_list, registered, authority))
        else:
            bad("登记表", f"{registry_path}:{line_number} 认不出的行（第一列写「频率词」带 4 或 5 列，或写「结构」带 6 列）：{line[:60]}")

for line_number, name, _aliases, registered, _authority in structures:
    if registered not in frequency_level:
        bad("登记表", f"{registry_path}:{line_number}「{name}」登记的频率「{registered}」不在频率词表里")
structures = [row for row in structures if row[3] in frequency_level]
alias_owner = {}
for line_number, name, alias_list, _registered, _authority in structures:
    for alias in alias_list:
        if alias in alias_owner and alias_owner[alias] != name:
            bad("登记表", f"{registry_path}:{line_number} 叫法「{alias}」同时登记给了「{alias_owner[alias]}」与「{name}」")
        alias_owner[alias] = name

# ── 权威分项在、已定、简称对得上（归属借 lib-item-ref-status.py）────────────────────────
spec = importlib.util.spec_from_file_location("item_ref_status", lib_path)
item_ref_status = importlib.util.module_from_spec(spec)
spec.loader.exec_module(item_ref_status)
item_map, short_names = item_ref_status.load_map()
AUTHORITY = re.compile(r"^(D\d+)（((?:[^（）]|（[^（）]*）)*)）\s*已定项\s*(\d+)$")
for line_number, name, _aliases, _registered, authority in structures:
    matched = AUTHORITY.match(authority)
    if not matched:
        bad("权威", f"{registry_path}:{line_number}「{name}」的权威分项「{authority}」不是「D<n>（简称） 已定项 k」的写法")
        continue
    decision, short_name, item = matched.group(1), matched.group(2), int(matched.group(3))
    if decision not in item_map:
        bad("权威", f"{registry_path}:{line_number}「{name}」的权威分项点名 {decision}，而 .claude/kb/decisions/ 里没有这条决策")
    elif short_names[decision] != short_name:
        bad("权威", f"{registry_path}:{line_number}「{name}」把 {decision} 的简称写成「{short_name}」，正文首行是「{short_names[decision]}」")
    elif item_map[decision].get(item) != "已":
        state = "没有这一条" if item not in item_map[decision] else "它是未定项"
        bad("权威", f"{registry_path}:{line_number}「{name}」的权威分项是 {decision} 第 {item} 条，而那条决策的分项表里{state}——权威只能是已定项")

# ── 读排除表 ────────────────────────
exclusions = []   # [行号, 决策文件名, 句中逐字的一段, 排到过没有]
if os.path.exists(exclude_path):
    with open(exclude_path, encoding="utf-8") as handle:
        for line_number, raw in enumerate(handle, 1):
            line = raw.rstrip("\n")
            if not line.strip() or line.startswith("#"):
                continue
            cells = line.split("\t")
            if len(cells) != 3 or not cells[2].startswith("#") or len(cells[2].lstrip("#").strip()) < 4:
                bad("排除表", f"{exclude_path}:{line_number} 不是「决策文件名<TAB>句中逐字的一段<TAB># 理由」：{line[:60]}")
                continue
            file_name, snippet = cells[0].strip(), cells[1].strip()
            if not os.path.exists(os.path.join(".claude/kb/decisions", file_name)):
                bad("排除表", f"{exclude_path}:{line_number} 指的 .claude/kb/decisions/{file_name} 不存在")
                continue
            if len(snippet) < 8:
                bad("排除表", f"{exclude_path}:{line_number} 句中那一段不足 8 个字，会排到不相干的句子：{snippet}")
                continue
            exclusions.append([line_number, file_name, snippet, False])

# ── 扫决策正文 ────────────────────────
ordered_words = sorted(frequency_level, key=len, reverse=True)
FREQUENCY = re.compile("|".join(f"(?:{phrase_pattern(word)})" for word in ordered_words)) if ordered_words else None
word_of = {re.sub(r"\s+", "", word): word for word in ordered_words}
# gap 是频率词与写动词之间那一段，留着看里面有没有否定词（「每次发布不写」）
WRITE_AFTER = re.compile(r"^(?P<gap>[^，。；：、,;:（）()「」“”|]{0,12}?)(?:写(?!放大)|更新|轮换|推进|回写|重写|改写|覆写|落盘|涨)|^\s*一次")
# 否定词在频率词前面，中间只许隔三个以内的虚字（「不会在每次发布时写」「不再每次发布推进」）
NEGATED_BEFORE = re.compile(r"(?:不是|并非|而非|没有|无需|无须|不必|不用|(?<!除)非|不|没)\s*[都是在随按会再需要必用应该能可就也于]{0,3}\s*$")
NEGATED_BEFORE_ADJACENT = re.compile(r"(?:不是|并非|而非|没有|无需|不必|不用|不|没|非)\s*[都是在随按]?\s*$")   # 弄坏开关换回的旧判法
NEGATED_BEFORE_VERB = re.compile(r"不(?![仅止只但同变论管])|没|无需|无须")
RESTRICTED_BEFORE = re.compile(r"(?:只有|只|仅)[在于]?\s*$")
RESTRICTION_NEGATED = re.compile(r"(?:不是|并非|而非|不)\s*(?:只有|只|仅)[在于]?\s*$")
alias_patterns = [(re.compile(phrase_pattern(alias)), alias_owner[alias]) for alias in sorted(alias_owner, key=len, reverse=True)]
registered_of = {name: (registered, authority) for _ln, name, _al, registered, authority in structures}
SOURCE_NOTE = {"sentence": "", "row": "（叫法在同一表格行的别的格）", "heading": "（叫法在所属标题）"}


def owners_in(text):
    return {owner for pattern, owner in alias_patterns if pattern.search(text)}


def judge(written, registered, form):
    """form 是 plain / negated / restricted；返回这一处与登记说不说反话。"""
    same_level = levels_merged or frequency_level[written] == frequency_level[registered]
    written_quantity, registered_quantity = frequency_quantity[written], frequency_quantity[registered]
    registered_is_whole_level = registered_quantity == frequency_level[registered]
    if form == "negated":     # 否定的时机被登记的那些时机罩住
        return same_level and (registered_is_whole_level or written_quantity == registered_quantity)
    if form == "restricted":  # 「只在 X 时」：X 与登记的必须是同一个量
        return written_quantity != registered_quantity
    return not same_level or (not registered_is_whole_level and written_quantity != registered_quantity)


files = sorted(glob.glob(".claude/kb/decisions/*.md"))
sentences_scanned = 0
assertion_sentences = 0
attributed_hits = 0
hits_by_source = {"sentence": 0, "row": 0, "heading": 0}
restricted_hits = 0
negations_ignored = 0
hit_structures = set()
red_hits = []   # (文件名, 行号, 结构, 写的词, 登记的词, 权威, 句子, 类别, 叫法从哪来)
for path in files:
    body = open(path, encoding="utf-8").read().split("\n## 历史版本")[0]
    in_fence = False
    heading_owners = set()
    for line_number, line in enumerate(body.split("\n"), 1):
        if line.lstrip().startswith("```"):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        plain = line.replace("**", "").replace("`", "")
        if plain.startswith("#"):
            heading_owners = owners_in(plain)
        cells = re.split(r"(?<!\\)\|", plain)
        cell_owners = [owners_in(cell) for cell in cells] if plain.lstrip().startswith("|") and len(cells) > 2 else []
        for index, cell in enumerate(cells):
            row_owners = set().union(*(owners for other, owners in enumerate(cell_owners) if other != index))
            for sentence in re.split(r"(?<=[。；！？])", cell):
                if not sentence.strip():
                    continue
                sentences_scanned += 1
                if FREQUENCY is None:
                    continue
                owners, source = owners_in(sentence), "sentence"
                if not owners and not subject_in_sentence_only:
                    if row_owners:
                        owners, source = row_owners, "row"
                    elif heading_owners:
                        owners, source = heading_owners, "heading"
                sentence_has_assertion = False
                for found in FREQUENCY.finditer(sentence):
                    after = WRITE_AFTER.search(sentence[found.end():])
                    if not after:
                        continue
                    sentence_has_assertion = True
                    written = word_of[re.sub(r"\s+", "", found.group(0))]
                    before = sentence[:found.start()]
                    if negation_adjacent_only:
                        negated = bool(NEGATED_BEFORE_ADJACENT.search(before))
                    else:
                        negated = bool(NEGATED_BEFORE.search(before) or NEGATED_BEFORE_VERB.search(after.group("gap") or ""))
                    if negation_counted:
                        form = "plain"
                    elif not restriction_as_plain and RESTRICTION_NEGATED.search(before):
                        form = "restriction-negated"   # 「不只在 X 时写」：还在别的时机写，不划界
                    elif negated:
                        form = "negated"
                    elif not restriction_as_plain and RESTRICTED_BEFORE.search(before):
                        form = "restricted"
                    else:
                        form = "plain"
                    for owner in sorted(owners):
                        attributed_hits += 1
                        hits_by_source[source] += 1
                        hit_structures.add(owner)
                        registered, authority = registered_of[owner]
                        if form == "restricted":
                            restricted_hits += 1
                        if form == "restriction-negated" or not judge(written, registered, form):
                            if form in ("negated", "restriction-negated"):
                                negations_ignored += 1
                            continue
                        red_hits.append((os.path.basename(path), line_number, owner, written, registered, authority, sentence.strip(), form, source))
                if sentence_has_assertion:
                    assertion_sentences += 1

for file_name, line_number, owner, written, registered, authority, sentence, form, source in red_hits:
    covering = [exclusion for exclusion in exclusions if exclusion[1] == file_name and exclusion[2] in sentence]
    for exclusion in covering:
        exclusion[3] = True
    if covering:
        continue
    where = f"{file_name}:{line_number}「{owner}」{SOURCE_NOTE[source]}"
    if form == "plain":
        bad("写频率", f"{where}写成「{written}」（{frequency_level[written]}级），登记的是「{registered}」（{frequency_level[registered]}级），权威：{authority}｜{sentence[:80]}")
    elif form == "negated":
        bad("写频率", f"{where}否定了「{written}」，而登记的「{registered}」与它同属{frequency_level[registered]}级，权威：{authority}｜{sentence[:80]}")
    else:
        bad("写频率", f"{where}用「只在 / 仅」把写的时机限定到「{written}」（量：{frequency_quantity[written]}），登记的「{registered}」量是{frequency_quantity[registered]}，"
                      f"不是同一个量，权威：{authority}｜{sentence[:80]}")
for line_number, file_name, snippet, used in exclusions:
    if not used:
        bad("排除表", f"{exclude_path}:{line_number}（{file_name}「{snippet}」）没排到任何一句判红的断言：句子改过了或已经不红，删掉这一行")

if not structures:
    bad("登记表", f"{registry_path} 一个结构都没登记（或登记的频率全不在词表里）")
elif attributed_hits == 0:
    bad("写频率", f"登记了 {len(structures)} 个结构，{len(files)} 份决策正文里一处落在它们身上的频率断言都没扫到——叫法对不上正文了，或者正文搬了地方")

if problems:
    kinds = sorted({kind for kind, _ in problems})
    print(f"  ✗ 写频率有 {len(problems)} 处不对（{'、'.join(kinds)}）：")   # gate-lint:summary
    for kind, detail in problems:
        print(f"      [{kind}] {detail}")                                  # gate-lint:detail
    print("  → 怎么办：[写频率] 按那一行点名的权威分项改这句的频率；权威分项自己改了，先改 .claude/gate.d/write-frequency.tsv 那个结构的登记，")
    print("    再把别处的说法一起改到同一个量级（C245（系统配置槽的写频率，两条条款说反话） 那一族：不许两处各留各的）。")
    print("    这句确实不是在说这个结构多久写一次，把「决策文件名<TAB>句中逐字的一段<TAB># 理由」加进 .claude/gate.d/write-frequency-exclude.tsv。")
    print("    [登记表] [权威] [排除表] 照明细改那张表的那一行；两张表的写法在各自的文件头。改完只重跑这一格：--check write-frequency")
    sys.exit(1)

missed = [name for _ln, name, _al, _r, _a in structures if name not in hit_structures]
levels = sorted(set(frequency_level.values()))
print(f"  ✓ 同一结构的写频率只有一个量级（扫 {len(files)} 份决策正文、{sentences_scanned} 句，{assertion_sentences} 句有频率断言、"
      f"{attributed_hits} 处落在登记的结构上（叫法在句内 {hits_by_source['sentence']}、同一表格行 {hits_by_source['row']}、所属标题 {hits_by_source['heading']}），"
      f"其中「只在 / 仅」限定形态 {restricted_hits} 处；登记 {len(structures)} 个结构、{len(levels)} 个量级、{len(set(frequency_quantity.values()))} 个量、"
      f"{len(frequency_level)} 个频率词；否定形态 {negations_ignored} 处不算断言；排除表 {len(exclusions)} 行）")
if missed:
    print(f"    没扫到断言的结构 {len(missed)} 个（叫法没和频率断言同句、同行或同标题出现，这几个没判）：{'；'.join(missed)}")
PY
)

# ════════════════ 变更史组的三格（变更史的条目与形状） ════════════════

# 在仓里时取 gate 基准写进 diff_base；不在仓里 diff_base 留空。source 失败、取不到基准就判红退出。
load_diff_base() {
  local LIB_CHANGED_PATHS
  diff_base=""
  git rev-parse --is-inside-work-tree >/dev/null 2>&1 || return 0
  LIB_CHANGED_PATHS="$REPO_ROOT/research/scripts/changed-paths.sh"
  # shellcheck source=../../research/scripts/changed-paths.sh
  source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用库 $LIB_CHANGED_PATHS"; echo "     → 怎么办：改动范围的取法只有那一份，恢复它，别在阶段里再抄一份。"; exit 1; }
  diff_base="$(gate_diff_base gate)" || { echo "  ✗ 共用库算不出 diff 基准"; echo "     → 怎么办：照上面 gate_diff_base 的报错修（多半是 GATE_BASE 写错了），再跑。"; exit 1; }
}

# ── entry-added ──
cell_entry_added() {
  local KB HIST diff_base diff_rc numstat changed decision_diff status_lines_touched added hist_diff
  KB=(.claude/kb/decisions.md .claude/kb/decisions)
  HIST=.claude/kb/decisions-history.md

  # 不在 git 仓里没有「改了什么」可比：退 77（本次无对象可判）。老写法把 git 的报错丢进 /dev/null，
  # 每一道 git 都失败、行数读成 0，于是「只改了 0 行，按小改动放行」报绿。
  if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "  ! 不在 git 仓库里，这一格无对象可判"
    exit 77
  fi
  # 在仓里时每一道 git 都要成功：失败了这一格什么都没比，判红，不许读成「没改」。
  git_failed() {
    bad "git $1 跑不起来（退出码 $2），决策正文改了多少、变更史加了几条都没数出来"
    howto "按上面 git 的报错修好仓库状态再跑（常见是还没有任何提交、HEAD 不存在：先提交一次）；"
    howto "  git 失败时这一格什么都没比，不是通过。"
    exit 1
  }
  load_diff_base
  diff_rc=0
  git diff --quiet "$diff_base" -- "${KB[@]}" || diff_rc=$?
  case "$diff_rc" in
    0) echo "  ! 决策正文与基准 $diff_base 无差异，这一格无对象可判"; exit 77 ;;
    1) ;;
    *) git_failed "diff --quiet $diff_base" "$diff_rc" ;;
  esac

  # 决策正文改了多少行（增 + 删，各文件相加）
  numstat="$(git diff "$diff_base" --numstat -- "${KB[@]}")" || git_failed "diff --numstat" "$?"
  changed=$(awk '{n+=$1+$2} END{print n+0}' <<<"$numstat")
  # 改动碰没碰状态行：碰了就不算小改动（把「—— 半定」改成「—— 已定」只有两行，而它正是最该留痕的那种）
  decision_diff="$(git diff "$diff_base" -U0 -- "${KB[@]}")" || git_failed "diff -U0" "$?"
  status_lines_touched=$(grep -cE '^[+-](## D[0-9]+ .*——|#{3,4} *(已定项|未定项)|.*状态：)' <<<"$decision_diff" || true)
  # 本次 diff 往变更史里加了几条：一条改动一个 `#### `；一个都没加的按 `### 日期` 数
  hist_diff="$(git diff "$diff_base" -- "$HIST")" || git_failed "diff（变更史）" "$?"
  added=$(grep -c '^+#### ' <<<"$hist_diff" || true)
  if ((added == 0)); then added=$(grep -c '^+### 20[0-9][0-9]-' <<<"$hist_diff" || true); fi

  if [[ "$added" -gt 0 ]]; then
    ok "决策正文改了 $changed 行，变更史新增 $added 条条目"
    exit 0
  fi
  if [[ "$changed" -le 4 && "$status_lines_touched" -eq 0 ]]; then
    echo "  ! 本次未判：决策正文只改了 $changed 行、没碰状态行，按小改动不要求变更史（不记通过）"
    exit 77
  fi

  if [[ "$status_lines_touched" -gt 0 ]]; then
    bad "决策正文改了 $changed 行（碰了 $status_lines_touched 行状态行，不按小改动放行），却没往变更史新增任何条目"
  else
    bad "决策正文改了 $changed 行，却没往变更史新增任何条目"
  fi
  howto "若这次改动推翻或定下了任何结论，在 $HIST 里那条决策的 \`## D<n>（简称）\` 节里加一条（.claude/rules/changelog-format.md）："
  howto "  这一天的 \`### $(TZ=Asia/Tokyo date +%F)\` 块已经有就在块里最下面加，没有就在现状行下面新开一块（新的日期在上），"
  howto "  每条改动一个 \`#### 摘要\`，正文写改前 / 改后 / 依据；一条改动点名了几条决策，就在几节里各写一份。"
  howto "已定 / 未定项数或一句话现状变了，改完 decisions.md 的索引表再跑 bash .claude/gate.d/doc-decisions.sh --write 同步节顶的现状行。"
  howto "纯排版改动可以拆成单独一个提交，那时这一格就无对象可判了。"
  exit 1
}

# ── status-sync ──
#   cell_status_sync check   比对
#   cell_status_sync write   按索引表重写（缺了补上）每节顶上的现状行，再回读判一遍
cell_status_sync() {
  python3 - "$1" <<'PY'
import os, re, sys

PAIRS = (('D', '决策', '.claude/kb/decisions.md', '.claude/kb/decisions-history.md'),
         ('E', '实验', '.claude/kb/experiments.md', '.claude/kb/experiments-history.md'))
PREFIX = '**现状**：'
FENCE = re.compile(r'^[ \t]*```')
BREAK = os.environ.get('DOC_DECISIONS_BREAK', '')
WRITE_COMMAND = 'bash .claude/gate.d/doc-decisions.sh --write'


def split_cells(line):
    return [cell.strip().replace('\\|', '|') for cell in re.split(r'(?<!\\)\|', line.strip())[1:-1]]


def index_rows(kind, path):
    """{编号: 该有的现状行}：索引表第一格 `D<n>（…）`，现状行取第二格（状态）与第三格（结论（简报））。"""
    rows = {}
    with open(path, encoding='utf-8') as handle:
        for line in handle:
            match = re.match(rf'^\| {kind}(\d+)（', line)
            if not match:
                continue
            cells = split_cells(line)
            if len(cells) >= 3:
                rows.setdefault(int(match.group(1)), f'{PREFIX}{cells[1]}。{cells[2]}')
    return rows


def sections(kind, lines):
    """[(编号, 标题行下标, 现状行下标或 None)]。现状行是标题下第一行非空行、以「**现状**：」开头。"""
    heading = re.compile(rf'^## {kind}(\d+)（')
    found, fenced = [], False
    for index, line in enumerate(lines):
        if FENCE.match(line):
            fenced = not fenced
            continue
        match = None if fenced else heading.match(line)
        if not match:
            continue
        status_index = None
        for later in range(index + 1, len(lines)):
            if lines[later].strip():
                status_index = later if lines[later].startswith(PREFIX) else None
                break
        found.append((int(match.group(1)), index, status_index))
    return found


def judge(kind, index_path, history_path, write):
    """返回（问题清单, 查了几节, 改写了几行）。write 时只改现状行，别的问题照报。"""
    if not os.path.isfile(index_path):
        return [f'{history_path} 在，索引表 {index_path} 却不在：现状行没处对'], 0, 0
    rows = index_rows(kind, index_path)
    with open(history_path, encoding='utf-8') as handle:
        lines = handle.read().split('\n')
    found = sections(kind, lines)
    problems, rewritten = [], 0
    # 从后往前改：补一行会挪动后面各节的下标
    for number, heading_index, status_index in reversed(found):
        expected = rows.get(number)
        if expected is None:
            problems.append(f'{history_path}:{heading_index + 1} {lines[heading_index]}  {index_path} 的索引表里没有 {kind}{number} 这一行')
            continue
        if status_index is None:
            if write:
                blank_follows = heading_index + 1 < len(lines) and not lines[heading_index + 1].strip()
                lines[heading_index + 1:heading_index + 1] = ['', expected] + ([] if blank_follows else [''])
                rewritten += 1
            else:
                problems.append(f'{history_path}:{heading_index + 1} {lines[heading_index]}  节顶上没有「{PREFIX}」一行；索引表那一行是「{expected}」')
            continue
        if BREAK == 'status-text-ignored' or lines[status_index] == expected:
            continue
        if write:
            lines[status_index] = expected
            rewritten += 1
        else:
            problems.append(f'{history_path}:{status_index + 1}  {kind}{number} 的现状行与索引表对不上：'
                            f'这里写「{lines[status_index][len(PREFIX):]}」，索引表是「{expected[len(PREFIX):]}」')
    problems.reverse()
    for number in sorted(set(rows) - {number for number, _, _ in found}):
        problems.append(f'{index_path} 的索引表有 {kind}{number}，{history_path} 里没有 `## {kind}{number}（简称）` 节')
    if write and rewritten:
        with open(history_path, 'w', encoding='utf-8') as handle:
            handle.write('\n'.join(lines))
    return problems, len(found), rewritten


def run(write):
    """返回（问题清单, {类: 查了几节}, 改写了几行, 不在的变更史）。"""
    problems, checked, rewritten, absent = [], {}, 0, []
    for kind, name, index_path, history_path in PAIRS:
        if not os.path.isfile(history_path):
            absent.append(history_path)
            continue
        pair_problems, pair_checked, pair_rewritten = judge(kind, index_path, history_path, write)
        problems += pair_problems
        checked[name] = pair_checked
        rewritten += pair_rewritten
    return problems, checked, rewritten, absent


write = sys.argv[1] == 'write'
if write:
    _, _, rewritten, _ = run(True)
    print(f'  · 按索引表重写了 {rewritten} 行现状（缺的补在标题下面），下面回读再判一遍')
problems, checked, _, absent = run(False)
if len(absent) == len(PAIRS):
    # 退 77：门禁记「本次未跑」，不记通过（`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）
    print(f'  ! 找不到 {"、".join(absent)}，这一格无对象可判')
    sys.exit(77)
if problems:
    print(f'  ✗ 变更史各节顶上的现状与索引表有 {len(problems)} 处对不上')  # gate-lint:summary
    for problem in problems[:40]:
        print(f'     {problem}')  # gate-lint:detail
    if len(problems) > 40:
        print(f'     ……另有 {len(problems) - 40} 处')  # gate-lint:detail
    print(f'     → 现状行照索引表那一行写：「{PREFIX}<状态格>。<结论（简报）格>」，放在 `## D<n>（简称）` / `## E<n>（简称）` 标题下第一行；')
    print(f'       先改索引表，再跑 {WRITE_COMMAND} 按索引表重写（缺了补上），它不碰历史条目。')
    print('     → 节在索引表里没有行、索引表的行在变更史里没有节，--write 补不了：先核编号——新立的决策 / 实验在变更史里加一节')
    print('       （标题照索引表第一格写），编号写错或并走了就改对，再跑 --write。规则：.claude/rules/changelog-format.md。')
    sys.exit(1)
total = sum(checked.values())
if total == 0:
    print(f'  ! 两份变更史里一节 `## D<n>（` / `## E<n>（` 都没有、索引表也一行都没有，这一格无对象可判')
    sys.exit(77)
skipped = f'；{"、".join(absent)} 不在，那一半没判' if absent else ''
print(f'  ✓ 现状行与索引表一致：查了{"、".join(f"{name} {count} 节" for name, count in checked.items())}{skipped}')
PY
}

cell_status_sync_check() { cell_status_sync check; }

# ── shape ──
cell_shape() {
  local diff_base
  load_diff_base
  python3 - "$diff_base" <<'PY'
import os, re, subprocess, sys

KB = '.claude/kb'
HISTORY_KIND = {'.claude/kb/decisions-history.md': 'D', '.claude/kb/experiments-history.md': 'E'}
KIND_NAME = {'D': '决策', 'E': '实验'}
FENCE = re.compile(r'^[ \t]*```')
FUSED = re.compile(r'^###[ \t]+20\d\d-\d\d-\d\d[ \t]*（其')
DATE_HEADING = re.compile(r'^###[ \t]+(20\d\d-\d\d-\d\d)')
UPPER_HEADING = re.compile(r'^#{1,2}[ \t]')
BLOCK_END = re.compile(r'^#{1,3}[ \t]')
SUB_HEADING = re.compile(r'^####[ \t]')
OWN_HISTORY = re.compile(r'^## 历史版本[ \t]*$')
HUNK = re.compile(r'^@@ -\S+ \+(\d+)(?:,(\d+))? @@')
BREAK = os.environ.get('DOC_DECISIONS_BREAK', '')
CHECK_COMMAND = 'bash .claude/gate.d/doc-decisions.sh --check shape'
base = sys.argv[1]


class DiffUnreadable(Exception):
    pass


def git(*arguments):
    result = subprocess.run(['git', '-c', 'core.quotepath=false', *arguments],
                            capture_output=True, encoding='utf-8', errors='replace')
    if result.returncode != 0:
        raise DiffUnreadable(f'git {arguments[0]} 退出 {result.returncode}：{result.stderr.strip()[:200]}')
    return result.stdout


def added_line_numbers():
    """（{路径: 新增的行号}, {整份算新增的路径}）；base 为空（不在 git 仓里）返回 None：整棵树都按新增判。"""
    if not base:
        return None
    added, path, in_header = {}, None, False
    diff = git('diff', '--no-renames', '--unified=0', '--no-color', '--no-ext-diff',
               '--src-prefix=a/', '--dst-prefix=b/', base, '--', KB)
    for line in diff.split('\n'):
        if line.startswith('diff --git '):
            in_header, path = True, None
        elif in_header and line.startswith('+++ '):
            target = line[4:].rstrip('\t')
            if target.startswith('"'):
                raise DiffUnreadable(f'git 给路径加了引号（{target}），还原不了是哪一份')
            path = None if target == '/dev/null' else (target[2:] if target.startswith('b/') else target)
        elif line.startswith('@@ '):
            in_header = False
            match = HUNK.match(line)
            if match and path:
                start, count = int(match.group(1)), int(match.group(2) or 1)
                added.setdefault(path, set()).update(range(start, start + count))
    whole = {item for item in git('ls-files', '-z', '--others', '--exclude-standard', '--', KB).split('\0') if item}
    return added, whole


def is_new(changes, path, numbers):
    if changes is None or BREAK == 'stock-judged':
        return True
    added, whole = changes
    return path in whole or bool(added.get(path, set()) & set(numbers))


def unfenced_lines(path):
    """[(行号, 行)]；围栏代码块里的行（含围栏行本身）不给。"""
    out, fenced = [], False
    with open(path, encoding='utf-8', errors='replace') as handle:
        for number, line in enumerate(handle, 1):
            line = line.rstrip('\n')
            if FENCE.match(line):
                fenced = not fenced
                continue
            if not fenced:
                out.append((number, line))
    return out


def kb_files():
    out = []
    for directory, subdirectories, names in os.walk(KB):
        subdirectories.sort()
        out += [os.path.join(directory, name) for name in sorted(names) if name.endswith('.md')]
    return out


if not os.path.isdir(KB):
    print(f'  ! 没有 {KB}，这一格无对象可判')
    sys.exit(77)
try:
    changes = added_line_numbers()
except DiffUnreadable as error:
    print(f'  ✗ 取不到这一轮新增了哪些行：{error}')  # gate-lint:summary
    print('     → 按上面 git 的报错修好仓库状态再跑；取不到新增行时这一格什么都没比，不是通过。')
    sys.exit(1)

fused_new, no_home_new, duplicate_new = [], [], []
fused_stock = no_home_stock = duplicate_stock = 0
files = kb_files()
date_headings = history_date_headings = 0
for path in files:
    lines = unfenced_lines(path)
    for number, line in lines:
        if DATE_HEADING.match(line):
            date_headings += 1
        if FUSED.match(line) and BREAK != 'fused-ignored':
            if is_new(changes, path, [number]):
                fused_new.append(f'{path}:{number}  {line}')
            else:
                fused_stock += 1
    kind = HISTORY_KIND.get(path)
    if kind is None:
        continue
    section_heading = re.compile(rf'^## {kind}\d+（')
    mention = re.compile(rf'(?<![\w-]){kind}\d+（')
    parent = None
    by_section = {}
    for index, (number, line) in enumerate(lines):
        if UPPER_HEADING.match(line):
            parent = (number, line)
            continue
        match = DATE_HEADING.match(line)
        if not match:
            continue
        history_date_headings += 1
        block = [(number, line)]
        for later_number, later_line in lines[index + 1:]:
            if BLOCK_END.match(later_line):
                break
            if SUB_HEADING.match(later_line):
                block.append((later_number, later_line))
        if parent and section_heading.match(parent[1]):
            by_section.setdefault(parent, {}).setdefault(match.group(1), []).append(number)
            continue
        if parent and OWN_HISTORY.match(parent[1]):
            named = sorted({found.rstrip('（') for _, text in block for found in mention.findall(text)})
            if not named:
                continue
            reason = f'在文件自己的「## 历史版本」里，标题点名了 {"、".join(named)}，要写进那一节'
        else:
            upper = parent[1] if parent else '（没有，在文件开头）'
            reason = f'不在任何 `## {kind}<n>（` 节里，上一级标题是「{upper}」'
        if BREAK == 'no-home-ignored':
            continue
        if is_new(changes, path, [item_number for item_number, _ in block]):
            no_home_new.append(f'{path}:{number}  {line}  {reason}')
        else:
            no_home_stock += 1
    for (section_number, section_line), dates in by_section.items():
        for date, numbers in dates.items():
            if len(numbers) < 2 or BREAK == 'duplicate-date-ignored':
                continue
            if is_new(changes, path, numbers):
                duplicate_new.append(f'{path}:{section_number} {section_line} 节里 ### {date} 出现 {len(numbers)} 次'
                                     f'（第 {"、".join(map(str, numbers))} 行）')
            else:
                duplicate_stock += 1

scope = '不在 git 仓里、没有基准：整棵树都按新增判' if changes is None else f'只判基准 {base[:12]} 之后新增的行'
stock = ('没有存量一说' if changes is None else
         f'存量（基准里就有，等搬迁，只计数不判红）：日期与序号粘在一起 {fused_stock} 处、'
         f'条目住在节外 {no_home_stock} 处、同节日期重复 {duplicate_stock} 组')
failed = False
if fused_new:
    failed = True
    print(f'  ✗ {len(fused_new)} 处新写的标题把日期与「（其N）」粘在同一个 `### ` 里')  # gate-lint:summary
    for item in fused_new:
        print(f'     {item}')  # gate-lint:detail
    print('     → 同一天的改动收进一个 `### YYYY-MM-DD`（这一组里这一天已经有就并进去），每条改动各占一个 `#### 摘要`；')
    print('       「（其N）」只在摘要撞了或没有摘要时写进 `#### `。规则：.claude/rules/changelog-format.md「一份 kb 文件自己的「历史版本」节」。')
if no_home_new:
    failed = True
    print(f'  ✗ {len(no_home_new)} 个新写的日期块没住进自己的节')  # gate-lint:summary
    for item in no_home_new:
        print(f'     {item}')  # gate-lint:detail
    print('     → decisions-history.md 的条目写进它点名的那条决策的 `## D<n>（简称）` 节，experiments-history.md 的写进 `## E<n>（简称）` 节；')
    print('       点名了几条就在几节里各写一份。文件自己的「## 历史版本」只写这一份文件自己的改动。规则：.claude/rules/changelog-format.md。')
if duplicate_new:
    failed = True
    print(f'  ✗ {len(duplicate_new)} 组同一节里同一个日期开了几个 `### ` 块')  # gate-lint:summary
    for item in duplicate_new:
        print(f'     {item}')  # gate-lint:detail
    print('     → 同一天的改动并进一个 `### 日期` 块，块下每条改动各一个 `#### `（先写的在上面）；多出来的 `### 日期` 行删掉。')
if failed:
    print(f'     {scope}；{stock}')
    print(f'     → 改完只重跑这一格：{CHECK_COMMAND}')
    sys.exit(1)
if date_headings == 0:
    # 退 77：门禁记「本次未跑」，不记通过（`.claude/singlefs-ai-sop/rules/show-me-test.md`「扫到 0 项也不是通过」）
    print(f'  ! {KB} 下 {len(files)} 份 .md 里一个 `### 日期` 标题都没有，这一格无对象可判')
    sys.exit(77)
print(f'  ✓ 变更史的形状对：查了 {len(files)} 份 kb 文件、{date_headings} 个 `### 日期` 标题'
      f'（两份变更史里 {history_date_headings} 个），{scope}，新增的没有违例；{stock}')
PY
}

# ════ 登记的格函数：弄坏开关点名这一格就不跑、按判过了记，否则调这一格的判法 ════
# 决策文档组十四格原是在后台起的（stdin 是 /dev/null），这里照样把 stdin 接到 /dev/null。
break_skips() {
  if [[ "$BREAK_SWITCH" == "$1" ]]; then
    echo "  ✓ 弄坏开关 DOC_DECISIONS_BREAK=$1：这一格没判"
    exit 0
  fi
}
judge_kb_shape()                { break_skips kb-shape;                grid_kb_shape </dev/null; }
judge_item_ref_status()         { break_skips item-ref-status;         grid_item_ref_status </dev/null; }
judge_status_redundancy()       { break_skips status-redundancy;       grid_status_redundancy </dev/null; }
judge_cross_decision_status()   { break_skips cross-decision-status;   grid_cross_decision_status </dev/null; }
judge_settled_item_self_open()  { break_skips settled-item-self-open;  grid_settled_item_self_open </dev/null; }
judge_settled_ref_says_open()   { break_skips settled-ref-says-open;   grid_settled_ref_says_open </dev/null; }
judge_stale_open_items()        { break_skips stale-open-items;        grid_stale_open_items </dev/null; }
judge_settled_same_file()       { break_skips settled-same-file;       grid_settled_same_file </dev/null; }
judge_freeze_layer_membership() { break_skips freeze-layer-membership; grid_freeze_layer_membership </dev/null; }
judge_decision_items_sync()     { break_skips decision-items-sync;     grid_decision_items_sync </dev/null; }
judge_blocking_verdict()        { break_skips blocking-verdict;        grid_blocking_verdict </dev/null; }
judge_user_verdict_owed()       { break_skips user-verdict-owed;       grid_user_verdict_owed </dev/null; }
judge_decision_summary_width()  { break_skips decision-summary-width;  grid_decision_summary_width </dev/null; }
judge_write_frequency()         { break_skips write-frequency;         grid_write_frequency </dev/null; }
judge_entry_added()             { break_skips entry-added;             cell_entry_added; }
judge_shape()                   { break_skips shape;                   cell_shape; }
judge_status_sync()             { break_skips status-sync;             cell_status_sync_check; }

# ════ --write：按格表次序重写两块投影，每块在自己的子 shell 里写完回读判一遍 ════
if ((write_mode)); then
  write_cells=()
  if ((STAGE_CELLS_CHECK_GIVEN)); then
    for write_cell in "${STAGE_CELLS_SELECTED[@]}"; do
      case "$write_cell" in
        decision-items-sync|status-sync) write_cells+=("$write_cell") ;;
        *)
          bad "--write 只重写 decision-items-sync（decisions.md 的分项清单与索引表状态列）与 status-sync（两份变更史各节顶上的现状行）两格管的投影，不能与 --check $write_cell 一起用"
          howto "去掉 --check 两块一起写，或写成 --check decision-items-sync / --check status-sync：bash .claude/gate.d/doc-decisions.sh --write"
          exit 2 ;;
      esac
    done
  else
    write_cells=(decision-items-sync status-sync)
  fi
  write_green=(); write_red=(); write_not_run=()
  for write_cell in "${write_cells[@]}"; do
    echo "── ${write_cell}（--write）"
    case "$write_cell" in
      decision-items-sync)
        if ( DECISION_ITEMS_WRITE=1; grid_decision_items_sync ); then write_exit=0; else write_exit=$?; fi ;;
      status-sync)
        if ( cell_status_sync write ); then write_exit=0; else write_exit=$?; fi ;;
    esac
    case "$write_exit" in
      0) write_green+=("$write_cell") ;;
      77) write_not_run+=("$write_cell") ;;
      *) write_red+=("${write_cell}（退出 ${write_exit}）") ;;
    esac
  done
  if ((${#write_red[@]})); then
    bad "--write 写了 ${#write_cells[@]} 块，${#write_red[@]} 块写完回读仍红：${write_red[*]}"
    howto "照上面那一块自己的红行改（节或索引行缺了 --write 补不了，先核编号），再跑 bash .claude/gate.d/doc-decisions.sh --write"
    exit 1
  fi
  if ((${#write_green[@]} == 0)); then
    echo "  ! --write 写的 ${#write_cells[@]} 块都本次无对象可判（${write_not_run[*]}），不记通过"
    exit 77
  fi
  ok "--write 按格表次序写完 ${#write_cells[@]} 块、回读都对：${write_green[*]}；本次无对象 ${#write_not_run[@]} 块"
  exit 0
fi

stage_cells_run "$ROOT"
