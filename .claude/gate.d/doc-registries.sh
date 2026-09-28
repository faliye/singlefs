#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录；外部引用那一格调的 research/scripts/verify-citations.sh 要本机固定下来的外部文献树，缺了在那一格判红，不交给 gate.sh 预判
# gate-stage: 登记表与引用（二十一格：字段表指向分项；字段投影；字段表加和与结构总宽登记；新池新建文件三份挂钩；准入不等式各项有人维护；段序列登记表与 E142 产物；格式常量占位；覆盖写释放回退复用三处挂钩；树表预留认购；欠账表两张登记表的行形状；收口表收全开着的账；已还清行点名的测试还在；总审核矛盾都有去向；收口表第 27 行那几笔的前置；实验号引用有定义；决策号引用有定义；不变量声明条数与表对得上、欠账表数得出条数；治理文档里的门禁号、路径与「小节」指得到；不变量条数在 invariants.md 之外也要对；外部引用还核得动；新写的不变量点名它判的字段住在哪条已定分项）
# gate-category: 文档类
# gate-similar: link-targets.py 它只解析 ](相对路径) 链接与「第 N 节」，不看反引号里的路径、「门禁 N 号」与「小节」名；这三类与 experiment-refs、decision-refs 两格同是「引用了不存在的东西」，并在这里
# gate-similar: doc-decisions.sh 它判决策文档本身：索引表结论列的字数上限、「分项引用状态」（引用写的已定 / 未定与正文相符，前提是那条分项存在）、每条未定项写没写「改新池新建文件的字节」那句判定、动了用户定案的条款有没有在欠账表里记一笔（方向是决策改动到欠账表）；这一道 field-refs 那一格判字节表「指向」的分项存不存在（指向 D22 已定项 99 的字段在它眼里没有对象可比），first-txn-hooks 那一格拿判了「是」的那几条去核字节表与里程碑引没引，tree-table-reserve 那一格对认购表求和、比预留，欠账表各格从欠账表、收口表与总审核出发判表本身的形状与账的去向，decision-refs 那一格只判 D 号有定义、不看分项号与状态
# gate-similar: checker-independence-and-sync.sh 它的 layout-checker-sync 那一格判 layouts.tsv 登记的布局常量变了 checker 跟没跟、按 lib-owed.py 判滞后登记点名的欠账还开着，对象是布局清单与 checker 源码；这一道只对 kb 各表之间、kb 与 E142 产物和钉住的用例之间的账，判欠账表与收口表本身
# gate-similar: code-source-discipline.sh 它的 reading-discipline 那一格按 lib-owed.py 判读数纪律登记表点名的欠账号还开着，对象是实验二进制源码；mutation-tables 那一格数变异表每条原文在源码里恰好命中一次，锚点腐化就红，服务于变异复跑；format-constants、clause-enums、feature-bits 几格拿 kb 里 format-const 标记的登记值去核 crates/singlefs-format 里同名 const 的字面量与 stale= 旧串、条文列与 Rust 枚举逐个成员对上、feature bit 与代码对应，方向是 kb 到代码；这一道判欠账表与收口表本身，closeout-row27-preconditions 那一格数的是登记的今天命中次数（可以是 0 或 2），全对上那一格记本次未跑，对不上说明那笔欠账的前置进来了、要回去重核；这一道的 field-table-sums 那一格与它共用 lib-format-const.py 读标记，判的是 kb 里字段表的加和、表前说明句与结构总宽有没有登记，format-const-placeholders 那一格是代码注释到 kb 的单向编号核对，closeout-row27-preconditions 那一格读源码只数几段写死的原文命中几次，都不拿 kb 的值去比代码
# gate-similar: doc-experiments.sh 它的 multipath-registry 那一格判实验页「路径与结论登记」表的形状、源码落点与共用项，quoted-result-lines 那一格判 kb 正文里整行抄的 E7RESULT 产物行在 research/results/ 里逐字找得到（方向是 kb 到产物）；这一道 audit-contradictions 那一格判总审核处置列里「」引的原文在点名的 kb 文件今天的正文里找得到，方向是记录到 kb，另外要判欠账号还开着
#
# 登记表与引用：字段与布局登记表之间、kb 与 E142 产物和钉住的用例之间的账；欠账表 checks-owed.md 与里程碑收口表本身的形状与账的去向；
# 「一处改了、引用它的地方没跟着改」（实验号、决策号、治理文档里的门禁号与路径与小节、外部引用），外加不变量清单自己的条数与字段落点。
# 原是三道阶段（「字段与布局登记」九格、「欠账与收口」五格、「引用与不变量」七格，下面分别叫字段与布局组、欠账与收口组、引用与不变量组），
# 2026-09-28 按门禁收缩判决 research/prompts/gate-shrink-r1-main-verification.md「采纳的改法」第 2 条合成这一道；
# 每一格的判据、出路、成功行里报的数与射程照原样留在那一格的注释与函数里，下面两处照判决改了：
#   欠账与收口组的 row27-preconditions 改名 closeout-row27-preconditions（盯的是里程碑收口表第 27 行那几笔欠账的前置），判法不动（判决 3-B1：不删）；
#   table-shape 那一格删掉「同一个编号在两张表里各登记一次」那一半，它归上游 doc-lint 的 F 判（判决 3-B3）。
#   experiment-refs、decision-refs 两格留着（判决 3-B8）。
# 格名表（--list 打它，--check <格名>[,<格名>…] 只跑点名的格，不给就二十一格按下面的次序全跑，格名写错退 2）：
# gate-cell: field-refs 新池新建文件的字段表，每个「指向」都指到一条真实存在的分项
# gate-cell: field-projection 决策里定的字段有没有漏投影进新池新建文件的表
# gate-cell: field-table-sums 字段表加出来的数（表后合计、表前说明句、format-const 标记）与字节布局表里结构总宽的登记
# gate-cell: first-txn-hooks 新池新建文件的三份文件互相挂钩（字节表 / 里程碑 / 决策索引）
# gate-cell: admission-terms 准入不等式的每一项都有人维护（统计量，或写明的例外）
# gate-cell: segment-registry 段序列登记表与 E142 产物逐字比对
# gate-cell: format-const-placeholders 格式常量文件里的占位，每个都指得到一条真实存在的分项或欠账
# gate-cell: second-txn-hooks 覆盖写、释放、回退与复用的三处互相挂钩（layout/02 登记表 / 里程碑各步「写出的字节」/ layout/01 段序列登记表）
# gate-cell: tree-table-reserve 树表条目预留字节的认购合计（认购不许超过预留，每条要指得到一个编号）
# gate-cell: table-shape 欠账表两张登记表的行形状（格数对得上表头、欠着的那张里不留写着已还的行、表不被空行断开、两张表都在）
# gate-cell: closeout-collects-open 里程碑收口表收全了文件里点名、还开着的欠账号，行号只许是顺序号
# gate-cell: paid-cited-tests 已还清的欠账行里点名的测试名，仓里还在不在
# gate-cell: audit-contradictions 最近一次总审核登记的文档级矛盾，每一行都要有去向
# gate-cell: closeout-row27-preconditions 里程碑收口表第 27 行那几笔欠账的前置有没有进来（进来了就得补会红的用例）
# gate-cell: experiment-refs kb 与项目规则里引用的实验号都有定义
# gate-cell: decision-refs kb 与项目规则里引用的决策号都有定义
# gate-cell: declared-counts invariants.md 自己那句「现共 N 条在用」与表里条数对得上；欠账表数得出条数
# gate-cell: governance-refs 治理文档、kb 与仓根 README.md 里按全名指的门禁与格都在、不用旧编号称呼门禁，治理文档里「门禁 N 号」有对应阶段、反引号里的仓内路径存在、`文件「小节」` 在那份文件里找得到
# gate-cell: invariant-count-elsewhere invariants.md 之外的 kb 文件里写「N 条在用」的地方，N 等于实际在用条数
# gate-cell: citations kb 里承重的外部逐行引用还核得动（调 research/scripts/verify-citations.sh）
# gate-cell: invariant-anchors 这次改动新写或改写的不变量行，点名它判的字段住在哪条已定分项
#
# 字段与布局组九格都判 kb 里的字段表、字节布局表、段序列登记表与里程碑各步之间的对账。
# first-txn-hooks 与 second-txn-hooks 判法不同形，各成一格：前一格从决策里判「是」的未定项出发，判字节表的空白表、
# 每节标题下的步号、里程碑每步点名的节标题（逐字，只许省末尾括注）；后一格从 layout/02 登记表出发，判行与步互相点名
# （形态前缀只许对得上一行）、钉住的用例在仓里、「八、」段序列登记表标到的步与点名它的步两边相等。
# 欠账与收口组五格都判 .claude/kb/checks-owed.md 与里程碑收口表，其中两格共用 lib-owed.py。
# 引用与不变量组七格都是机械可判的；governance-refs 的判据与够不着的写法在 lib-governance-refs.py 文件头。
# declared-counts 与 invariant-count-elsewhere 逐字核过，判的不是同一件事：前者只判 invariants.md 自己
# <!-- invariant-count --> 下一行那句，后者只判 invariants.md 与变更史之外的 kb 文件；在用条数两格用同一份数法（count_invariant_rows）。
# 实验与决策之间的两件事不在这里判，都归 doc-experiments.sh 的 decision-links 格（三方判决 gate-fix-forks-r1、r2 的 T1）：
#   「实验改成已跑、引用它的决策有没有同批回看」归 ⑤——表里改过一行不够，正文引了它的每条决策都要回看；
#   「已跑的实验有没有对应的决策」归 ⑨——表里至少一行支撑、推翻或备料，标题写了作废或退役的不判。
# 每格的成功行都报查了多少项；本该有对象却一个都没扫到的，判红（.claude/singlefs-ai-sop/rules/show-me-test.md「扫到 0 项也不是通过」）。
#
# 参数解析、样本根的 .gate-cells、每格一个子 shell、逐格汇总与退出码都经共用库 lib/stage-cells.sh（写法与判据在它的文件头），
# 这里只登记格、解析项目根。格按上表的次序一格跑完再跑下一格（字段与布局组九格原是并行跑的、引用与不变量组的 citations 原是先在后台起的，
# 并进来之后照共用库逐格跑，citations 轮到它时才起 verify-citations.sh、等它跑完）。
#   有一格红（退出码既不是 0 也不是 77）⇒ 整道退 1，红格逐格接它登记的出路；跑到的格全退 77 ⇒ 整道退 77，不记通过；
#   退 77 的格逐格往 $GATE_NOT_RUN_FILE 报一行「本次未跑」（closeout-row27-preconditions 那一格前置没进来时就退 77）。
#
# 判别力：fixtures/doc-registries.sh/ 下每份样本根放 .gate-cells，只跑它点名的格：
#   <格名>-red                  字段与布局组的九份红样本各对一格（field-refs-red……tree-table-reserve-red），必须判红，
#                               expect 除那一格自己的 want 外还要汇总里「<格名>：红（退出 1）」那一行
#   field-registry-green        字段与布局组九格的绿样本并在一棵树里，九格都判绿、整道退 0，每一格的成功行各有一条 want
#   checks-owed-red             欠账与收口五格的红样本放在一棵树里，同一份欠账表同时喂五格，五格都要判红、各报出自己点名的行，整道退 1
#   checks-owed-green           五格的绿样本，四格判绿、closeout-row27-preconditions 那一格退 77，整道退 0 并报那一格本次未跑
#   references-red              引用与不变量七格各放一处该红的对象（悬空的实验号与决策号、invariants.md 丢了登记标记、欠账表一行都数不出、
#                               治理文档里悬空的门禁号、路径与小节、别处写错的「N 条在用」、没有 verify-citations.sh、未跟踪的清单里一行没点名分项），逐格点名判红
#   references-unreadable-red   读不了的 kb 文件（悬空的符号链接），experiment-refs、decision-refs 两格要报出 grep 退 2
#   references-green            七格都判绿（setup.sh 把它建成 git 仓：清单已跟踪、新加一行带分项引用又 git add，只许算 1 行；
#                               research/scripts/verify-citations.sh 是替身，退 0、打一行 ✓，判的是 citations 这一格自己那两件事：脚本在被判的仓里就去跑它、按它的退出码判）
#   governance-refs-red         只跑 governance-refs：kb 现状文字里用旧编号称呼门禁（「门禁 11 号」「门禁 91」「gate.d/47-…」三种写法、「54、55 号」里的 55），
#                               「门禁 doc-kb」不是门禁、「doc-registries 的 no-such-cell 格」与「--check no-such-cell」格名不在，
#                               仓根 README.md 里的「门禁 59 号」与「checker-tier-lkmm 的 no-such-cell 格」，逐条点名判红
#   governance-refs-green       只跑 governance-refs：按全名指的门禁与格都在、「门禁 doc-lint」是上游共享门禁、「54 号」照旧；
#                               「## 历史版本」一节、decisions-history.md 与 experiments-history.md 里的旧编号不判，
#                               记录文件名里带的号不判，仓根 README.md 里按全名指的门禁与格（含门禁 54-layer0-replay 与它的脚本路径）都在，
#                               判绿并报扫了几份、全名与格名几处
# 每一格的样本放了什么，写在那一格的注释里。
# governance-refs 另有 lib-governance-refs.py 的弄坏开关 GOVERNANCE_REFS_BREAK=old-numbers-ignored / names-ignored（governance-refs-red 里对应的 want 找不到）、
# =history-scanned（governance-refs-green 判红）、=readme-skipped（仓根 README.md 不进射程：governance-refs-red 里 README.md 那两行 want 找不到，
# governance-refs-green 的份数与处数对不上）。
# 汇总与 --check 的弄坏开关归共用库（STAGE_CELLS_BREAK=red-swallowed 让各份 *-red 整道退 0；=check-ignored 让 --check 不起作用）。
# 格的弄坏开关 DOC_REGISTRIES_BREAK=<格名>（样本自检时设进环境）让那一格不跑、按判过了记：那一格的红样本出不来它的红行、
# 绿样本出不来它的成功行，样本判错。research/scripts/changed-paths.sh 的 LIB_CHANGED_PATHS_BREAK=untracked 让 invariant-anchors
# 看不见未跟踪的清单，references-red 里那一格报不出来，样本判错。
#
#   bash .claude/gate.d/doc-registries.sh [项目根]                                   二十一格都跑
#   bash .claude/gate.d/doc-registries.sh --list                                     逐行打格名与判什么，不跑格
#   bash .claude/gate.d/doc-registries.sh --check <格名>[,<格名>…] [项目根]          只跑点名的格
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

# 阶段自己的位置在 cd 之前取：用相对路径调本阶段、又另给项目根时，cd 之后 $(dirname "$0") 就解析不到了
STAGE_NAME="$(basename "${BASH_SOURCE[0]}")"
STAGE_FILE_NAME="$STAGE_NAME"
STAGE_DIRECTORY="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GATE_DIRECTORY="$STAGE_DIRECTORY"
REPOSITORY="$(cd "$STAGE_DIRECTORY/../.." && pwd)"
OWED_LIBRARY="$STAGE_DIRECTORY/lib-owed.py"
FORMAT_CONST_LIBRARY="$STAGE_DIRECTORY/lib-format-const.py"
SEGMENT_REGISTRY_SCRIPT="$REPOSITORY/research/scripts/check-segment-registry.py"
BREAK_SWITCH="${DOC_REGISTRIES_BREAK:-}"
# gate.sh 与阶段之间的握手变量只留给共用库在汇总时写，不导出给各格起的子进程
# （`.claude/singlefs-ai-sop/rules/command-safety.md`「进程边界上的三种静默失效」）
export -n GATE_NOT_RUN_FILE 2>/dev/null || true
source "$STAGE_DIRECTORY/lib/stage-cells.sh"

stage_cell field-refs judge_field_refs "新池新建文件的字段表，每个「指向」都指到一条真实存在的分项" \
  "每一行的「指向」列要么写一条真实存在的分项（编号改过就跟着改），要么写「无分项」把格式级空白摆出来"
stage_cell field-projection judge_field_projection "决策里定的字段有没有漏投影进新池新建文件的表" \
  "把缺的那一行补进 .claude/kb/layout/01-first-txn.md 对应的那一节（字段、宽度、指向哪条分项、状态）"
stage_cell field-table-sums judge_field_table_sums "字段表加出来的数（表后合计、表前说明句、format-const 标记）与字节布局表里结构总宽的登记" \
  "加错了就改那个数，刚加了一行字段就把同一轮要一起改的合计、说明句与 format-const 标记一起改；没登记的结构总宽照红行下面的写法登记"
stage_cell first-txn-hooks judge_first_txn_hooks "新池新建文件的三份文件互相挂钩（字节表 / 里程碑 / 决策索引）" \
  "判「是」的未定项要同时进 layout/01-first-txn.md 的「还剩几处空白」表与 milestone/01-first-txn.md 某一步的「会碰到的决策点」；节标题下标步号，里程碑每步点名的节标题逐字抄"
stage_cell admission-terms judge_admission_terms "准入不等式的每一项都有人维护（统计量，或写明的例外）" \
  "给式子里多出来的那一项在对照表里补一行（统计量或写明的例外），副本与权威那一行逐字对齐"
stage_cell segment-registry judge_segment_registry "段序列登记表与 E142 产物逐字比对" \
  "照 research/scripts/check-segment-registry.py 打出的逐行差异改 kb 的「八、」登记表，或重跑 E142 让产物跟上；出处指的用例文件与函数要在"
stage_cell format-const-placeholders judge_format_const_placeholders "格式常量文件里的占位，每个都指得到一条真实存在的分项或欠账" \
  "占位那一行写成「// placeholder: D<n>（简称） 已定项 k —— 为什么还是预想」或「// placeholder: C<n>（简称） —— …」，编号要指得到"
stage_cell second-txn-hooks judge_second_txn_hooks "覆盖写、释放、回退与复用的三处互相挂钩（layout/02 登记表 / 里程碑各步「写出的字节」/ layout/01 段序列登记表）" \
  "layout/02 登记表每一行的「里程碑那一步」与那一步的「写出的字节」互相点名（形态原样引），钉住的用例文件与函数要在仓里"
stage_cell tree-table-reserve judge_tree_table_reserve "树表条目预留字节的认购合计（认购不许超过预留，每条要指得到一个编号）" \
  "认购表在 .claude/kb/decisions/08-核心索引结构.md 已定项 11：合计超了先去掉一个认购者或压小宽度，别直接改 total；出处补一个还开着的欠账号或已定分项"
stage_cell table-shape judge_table_shape "欠账表两张登记表的行形状（格数对得上表头、欠着的那张里不留写着已还的行、表不被空行断开、两张表都在）" \
  "六列的欠账行挪回欠着的那张表、表中间的空行删掉；真还清的按已还清那张的四列改写再挪过去，只还了一部分的把开头改成写明还欠什么"
stage_cell closeout-collects-open judge_closeout_collects_open "里程碑收口表收全了文件里点名、还开着的欠账号，行号只许是顺序号" \
  "漏收的开着编号在收口表里给一行，或在表后加一行「- 不收口 C<编号>（简称）：为什么不收」；已经还了就先把欠账表那一行挪进「### 已还清」；行号改成没被占用的顺序号，不加撇号或字母"
stage_cell paid-cited-tests judge_paid_cited_tests "已还清的欠账行里点名的测试名，仓里还在不在" \
  "先 grep 那个测试今天叫什么，用 research/scripts/replace-once.py 把行里的旧名字定点换成现在的名字；它被删掉了，那笔欠账没还清，整行挪回欠着的那张表"
stage_cell audit-contradictions judge_audit_contradictions "最近一次总审核登记的文档级矛盾，每一行都要有去向" \
  "给这一行一个去向：改了就把处置列写成「已改…」、用「」引一句今天正文里的原文并点名它住在哪份文件；还没改就在 checks-owed.md 开着的那张表里立一笔账、把编号写进处置列"
stage_cell closeout-row27-preconditions judge_closeout_row27_preconditions "里程碑收口表第 27 行那几笔欠账的前置有没有进来（进来了就得补会红的用例）" \
  "先回 .claude/kb/milestone/02-second-txn.md 收口表第 27 行重核这一笔今天可不可达：可达了就当场做成一条会红的用例、把这一笔与探针挪走；只是重构就把探针原文改成今天逐字存在的那一段并写明重核过，别只改期望值"
stage_cell experiment-refs judge_experiment_refs "kb 与项目规则里引用的实验号都有定义" \
  "在 experiments/ 下给它建正文（\`## E<n> <简称>\` 起头），或把引用改成真实存在的实验号；读不了的文件按 grep 的原话修"
stage_cell decision-refs judge_decision_refs "kb 与项目规则里引用的决策号都有定义" \
  "在 decisions/ 下给它建正文（\`## D<n> <简称>\` 起头），或把引用改成真实存在的决策号；读不了的文件按 grep 的原话修"
stage_cell declared-counts judge_declared_counts "invariants.md 自己那句「现共 N 条在用」与表里条数对得上；欠账表数得出条数" \
  "按红行把 invariants.md 那句条数改成表里现数，登记标记丢了就补回；欠账表数不出条数就照 lib-owed.py 的读法把表恢复"
stage_cell governance-refs judge_governance_refs "治理文档、kb 与仓根 README.md 里按全名指的门禁与格都在、不用旧编号称呼门禁，治理文档里「门禁 N 号」有对应阶段、反引号里的仓内路径存在、\`文件「小节」\` 在那份文件里找得到" \
  "按红行把门禁改成按全名称呼（旧编号照红行给的今天的名字）、格名照那道门禁的 --list、路径与「小节」名改成今天存在的（判据与够不着的写法在 .claude/gate.d/lib-governance-refs.py 文件头）"
stage_cell invariant-count-elsewhere judge_invariant_count_elsewhere "invariants.md 之外的 kb 文件里写「N 条在用」的地方，N 等于实际在用条数" \
  "把那几处的 N 改成 invariants.md 表里实际在用的条数，或者改成引用 invariants.md 而不抄数"
stage_cell citations judge_citations "kb 里承重的外部逐行引用还核得动（调 research/scripts/verify-citations.sh）" \
  "按 verify-citations.sh 自己打印的下一步处置；未命中不等于 kb 写错，也可能是源码固定点没了或版本变了；脚本不在就从 git 找回"
stage_cell invariant-anchors judge_invariant_anchors "这次改动新写或改写的不变量行，点名它判的字段住在哪条已定分项" \
  "在那一行正文里点名一条「D<n>（简称） 已定项 k」，那是这条不变量要判的字段的落点；点不出来先去补分项"
stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}

root_argument=""
while (($#)); do
  case "$1" in
    -*)
      echo "  ✗ 认不出的选项 $1"
      echo "     → 怎么办：只认 --list、--check <格名>[,<格名>…] 与一个项目根；格名用 --list 看。"
      exit 2 ;;
    *)
      if [[ -n "$root_argument" ]]; then
        echo "  ✗ 给了两个项目根：$root_argument 与 $1"
        echo "     → 怎么办：参数只认一个项目根；不给项目根就取这个脚本往上两级。"
        exit 2
      fi
      root_argument="$1"; shift ;;
  esac
done
ROOT="${root_argument:-$REPOSITORY}"
cd "$ROOT" 2>/dev/null || { echo "  ✗ 进不去项目根 $ROOT"; echo "     → 怎么办：第一个参数给项目根（仓的顶层目录），不给就取这个脚本往上两级；路径写错或没有权限进去时这一道什么都没判。"; exit 2; }
ROOT="$(pwd)"

# ── 引用与不变量组七格共用的现场：kb 目录、放 grep 报错与 verify-citations.sh 输出的临时目录、判红记号与打印
KB=.claude/kb
work_directory="$(mktemp -d)" || { echo "  ✗ 建不了临时目录"; echo "     → 怎么办：看 \${TMPDIR:-/tmp} 能不能写、满没满；这一道什么都没判。"; exit 1; }
trap 'rm -rf "${work_directory:?}"' EXIT

cell_failed=0
say() { printf '  %s\n' "$*"; }
bad() { printf '  ✗ %s\n' "$*"; cell_failed=1; }
ok()  { printf '  ✓ %s\n' "$*"; }
howto() { printf '     → 怎么办： %s\n' "$1"; shift; for l in "$@"; do printf '                %s\n' "$l"; done; }

# ════════════════ 字段与布局组的九格（字段与布局登记） ════════════════

# ── 格 field-refs：新池新建文件的字段表，每个「指向」都指到一条真实存在的分项
#
# 判据：`kb/layout/01-first-txn.md` 每一行的「指向」列，凡是写了
# 「D<n>（简称） 已定项 <k>」或「…… 未定项 <k>」的（简称可以不写，也可以套一层括号），那条分项必须在
# `kb/decisions/` 那条决策「### 已定项」/「### 未定项」的索引里（第一个 `####` 之前）真实存在；指不到就判红。
#
# 那张表是 C66（格式级空白没有分项号）欠的东西，它的价值在于「每个字段都指得到一条分项」这个性质持续成立：
# 分项会改号、会从未定项挪进已定项、会被合并，某一行指向的分项不存在了，读表的人看到的仍然是一个格式正确的编号。
#
# ⚠️ 它和「分项引用的状态与正文相符」（`doc-decisions.sh` 的格「分项引用状态」）不是一条：
# 那一格判「说已定的是不是真已定」，前提是那条分项存在；这一格判分项存不存在。
# 一个指向 `D22 已定项 99` 的字段，在 doc-decisions 的格「分项引用状态」眼里没有对象可比，会安静地通过。
#
# ⚠️ 写了「无分项」的行是合法的，那正是这张表要暴露的格式级空白，不判红。
# 表或 decisions/ 不在、表里一处引用都没有：这一格无对象可判，退 77。
#
# 样本：field-refs-red 里一行指向 D322 已定项 7（索引里没有）、一行指向简称套了括号的 D323 已定项 3、
# 一行指向 D322 已定项 5（只在论证的编号列表里出现，不是分项）；field-registry-green 里表格式与列表式两种索引、未定项与「无分项」都放行。
cell_field_refs() {
  local KB=.claude/kb
  local TABLE="$KB/layout/01-first-txn.md"
  [[ -f "$TABLE" ]] || { echo "  ! 没有 $TABLE，这一格无对象可判"; exit 77; }
  [[ -d "$KB/decisions" ]] || { echo "  ! 没有 $KB/decisions，这一格无对象可判"; exit 77; }

  local bad=0 checked=0 lineno dnum kind item file section

  # 抽出表里每一处「D<n>（…） 已定项 <k>」/「未定项 <k>」引用，连同行号
  while IFS=$'\t' read -r lineno dnum kind item; do
    [[ -z "${dnum:-}" ]] && continue
    checked=$((checked + 1))

    # 找到那条决策的正文文件：文件名以两位编号打头
    file="$(ls "$KB/decisions/" 2>/dev/null | grep -E "^0*${dnum}-" | head -1)"
    if [[ -z "$file" ]]; then
      echo "  ✗ $TABLE:$lineno 指向 D$dnum，而 decisions/ 里没有这条决策"   # gate-lint:detail
      bad=$((bad + 1))
      continue
    fi

    # 那条分项要在对应小节的索引表里有一行，形如 `| <k> | …`
    # 已定项与未定项各有一张表，取「### 已定项」/「### 未定项」之后到下一个任意级标题之前：
    # 只看索引，`#### 已定项 N` 之下的论证里另有编号列表、「## 历史版本」里另有表格，那些不是分项
    # （与 doc-decisions 的格「kb 形状」第 5 段、格「状态别说两遍」同一条边界）
    section="$(awk -v want="$kind" '
      /^### 已定项[[:space:]]*$/ { cur="已定项"; next }
      /^### 未定项[[:space:]]*$/ { cur="未定项"; next }
      /^#+ / { cur="" }
      cur == want { print }
    ' "$KB/decisions/$file")"

    # 分项行有两种写法：表格行 `| k | …`（D18 / D22 一类），
    # 或编号列表 `k. **…**`（D16 / D19 / D23 一类）。两种都认。
    if ! grep -qE "^(\| *${item} *\||${item}\. )" <<<"$section"; then
      echo "  ✗ $TABLE:$lineno 指向 D$dnum（$file）的「$kind $item」，而那张表里没有这一条"   # gate-lint:detail
      bad=$((bad + 1))
    fi
  done < <(
    grep -n '指向\|已定项\|未定项' "$TABLE" 2>/dev/null \
      | grep -oE '^[0-9]+:.*' \
      | while IFS= read -r line; do
          n="${line%%:*}"
          body="${line#*:}"
          # 一行里可能有多处引用，逐个抠出来：简称可以不写（「D22 已定项 99」），也可以里面再套一层括号
          # （「D14（双轨（大小文件 / 持久临时）） 已定项 9」）；编号左边不许紧挨字母数字
          grep -oE '(^|[^A-Za-z0-9])D[0-9]+(（([^（）]|（[^（）]*）)*）)? *(已定项|未定项) *[0-9]+' <<<"$body" \
            | grep -oE 'D[0-9]+.*' \
            | while IFS= read -r ref; do
                d="$(grep -oE '^D[0-9]+' <<<"$ref" | tr -d 'D')"
                k="$(grep -oE '(已定项|未定项)' <<<"$ref" | head -1)"
                i="$(grep -oE '[0-9]+$' <<<"$ref")"
                printf '%s\t%s\t%s\t%s\n' "$n" "$d" "$k" "$i"
              done
        done
  )

  if ((bad)); then
    echo "  ✗ 新池新建文件的字段表里有 $bad 处指向了不存在的分项"
    echo "     → 怎么办： 每一行的「指向」列要么写一条真实存在的分项（编号改过就跟着改），"
    echo "                要么写「无分项」——后者是这张表要暴露的格式级空白，不判红。"
    echo "                分项现在叫什么，看 .claude/kb/decisions/ 里那条决策的两张索引表。"
    exit 1
  fi

  if ((checked == 0)); then
    echo "  ! $TABLE 里一处「D<n>（简称） 已定项 / 未定项 k」引用都没有，这一格无对象可判"
    exit 77
  fi
  echo "  ✓ 新池新建文件的字段表指向都成立（查了 $checked 处引用）"
}

# ── 格 field-projection：决策里定的字段有没有漏投影进新池新建文件的表
#
# field-refs 那一格查的是正向：[layout/01-first-txn.md] 表里每个字段都指得到一条真实分项。
# 这一格查反向：一条分项的字段表里新加了一行，而投影表没跟着加。
#
# ⚠️ 这条是实测出来的（2026-09-07）：D8（核心索引结构）已定项 8 ② 于 2026-09-06 把
# 树 ID 水位 8 字节加进根记录（D22 已定项 7 的字段表当天就加了这一行，合计 186 → 194），
# 而 [layout/01-first-txn.md] 第七节「发布（根记录与根槽）」那张表一个字没动，正向检查全程判绿。
#
# 射程（`.claude/singlefs-ai-sop/rules/show-me-test.md`「没实现的要明说」）：
#   1. 只认表头恰好是 `| 字段 | 宽 |` 或 `| 字段 | 宽度 |` 的表。散在段落里的字段定义抓不到。
#   2. 只检查已经被投影过的分项——[layout/01-first-txn.md] 里出现过 `D<n>（简称） 已定项 <k>` 的那些。
#      不是每条分项都属于新池新建文件（口径见 [layout/01-first-txn.md] 开头：不含快照、加密、目录树），
#      对没被投影的分项要求投影会假红。代价是：一条从头到尾就没进过投影表的分项，这一格一个字也不说。
#   3. 字段名按子串比对，只判「在不在」，不判宽度值对不对（宽度归 code-source-discipline 与 C94）。
#      比对范围是投影表里引用了这条分项的那几节（按 `## ` 标题切）里的表格行：字段名只在别的节、或只在正文段落里出现，不算投影了。
#   一张被投影的字段表都没核到：无对象可判，退 77（不记通过）。
#
# 样本：field-projection-red 里 D362 已定项 7 的字段表有三行，投影表只收了 magic，「块头版本」只在别的节的正文里提了一句；
# field-registry-green 里被投影的两行都在引用了那条分项的那一节的表里。
cell_field_projection() {
  local FTL=.claude/kb/layout/01-first-txn.md
  [[ -f "$FTL" && -d .claude/kb/decisions ]] || { echo "  ! 找不到 $FTL 或 decisions/，这一格无对象可判"; exit 77; }

  python3 - "$FTL" <<'PY'
import re, sys, glob, os

ftl_path = sys.argv[1]
ftl = open(ftl_path, encoding="utf-8").read().split("\n## 历史版本")[0]

# 投影表引用过哪些分项：D<n>（任意简称） 已定项 <k>
REFERENCE = r"D(\d+)（[^）]*）\s*已定项\s*(\d+)"
refs = {(int(a), int(b)) for a, b in re.findall(REFERENCE, ftl)}

# 按 `## ` 切节：每节记下它引用了哪些分项、它的表格行。字段名要落在引用了那条分项的节的表格行里，
# 在整份文件里按子串找会让别的节的正文顶替（「树 ID 水位」在别处提过一句，那一节的表里却没有这一行）
sections = []
for chunk in re.split(r"(?m)^(?=## )", ftl):
    chunk_refs = {(int(a), int(b)) for a, b in re.findall(REFERENCE, chunk)}
    table_rows = "\n".join(line for line in chunk.splitlines() if line.startswith("|"))
    sections.append((chunk_refs, table_rows))

def projected_table_rows(decision_item):
    return "\n".join(rows for chunk_refs, rows in sections if decision_item in chunk_refs)

checked_tables, checked_rows, bad = 0, 0, []
for p in sorted(glob.glob(".claude/kb/decisions/*.md")):
    dnum = int(os.path.basename(p).split("-")[0])
    lines = open(p, encoding="utf-8").read().split("\n## 历史版本")[0].splitlines()
    item = None      # 最近一个「已定项 N」标题的编号
    in_tbl = False
    for i, l in enumerate(lines, 1):
        m = re.match(r"#{3,6}\s*已定项\s*(\d+)", l)
        if m:
            item = int(m.group(1))
        if not l.startswith("|"):
            in_tbl = False
            continue
        c = [x.strip() for x in l.strip().strip("|").split("|")]
        if len(c) >= 2 and c[0] == "字段" and c[1] in ("宽", "宽度"):
            in_tbl = item is not None and (dnum, item) in refs
            if in_tbl:
                checked_tables += 1
            continue
        if in_tbl and len(c) >= 2 and re.fullmatch(r"\**\d+\**", c[1]):
            name = re.sub(r"[*`]", "", c[0]).strip()
            if not name:
                continue
            checked_rows += 1
            if name not in projected_table_rows((dnum, item)):
                bad.append(f"{p}:{i}  D{dnum} 已定项 {item} 的字段表有「{name}」（宽 {c[1]}），"
                           f"而 {ftl_path} 里引用了这条分项的那几节的表格行里找不到它")

if bad:
    print(f"  ✗ 分项定了的字段没投影进新池新建文件的表 {len(bad)} 处：")     # gate-lint:summary
    for b in bad:
        print("      " + b)                                              # gate-lint:detail
    print(f"  → 怎么办：把缺的那一行补进 {ftl_path} 对应的那一节（字段、宽度、指向哪条分项、状态），")
    print("    它是「新池新建文件写出哪些字节」的投影表，漏一行等于那几个字节没有任何人在看；")
    print("    若这个字段确实不属于新池新建文件（快照 / 加密 / 目录树），")
    print("    那就把它从这条分项的字段表里挪走，或把该分项从投影表的引用里去掉——两处口径必须一致。")
    sys.exit(1)

if checked_tables == 0:
    print(f"  ! 本次无对象可判：投影表引用了 {len(refs)} 条分项，其中没有一条的正文里有「| 字段 | 宽 |」字段表")
    sys.exit(77)
print(f"  ✓ 被投影的分项，字段表都投影全了（{checked_tables} 张字段表、{checked_rows} 行；"
      f"投影表引用了 {len(refs)} 条分项）")
PY
}

# ── 格 field-table-sums：字段表加出来的数（表后合计、表前说明句、format-const 标记）与字节布局表里结构总宽的登记
#
# C94（登记的格式常量与后来的定案对不上）逐字要的两条：
# 「每个 `format-const` 标记的值，与同一份正文里点名同一个量的全部已定增量之和相等，不等即判红」；
# 「[layout/01-first-txn.md] 表里每个已定宽度都有登记标记，缺即判红」。
# 这一格落的是它们能机械判的那几样：字段表紧跟着的「合计 N 字节」、表前说明句里的总宽与增量、
# 一个分项里唯一的那个标记，以及字节布局表里写成认得出的形态的结构总宽（射程第 7、8 条）。
#
# ⚠️ 这条是实测出来的（2026-09-07）：D22（单元原子性怎么合成）已定项 7 的根记录字段表
# 2026-09-06 加了「树 ID 水位 8」，正文的合计跟着改成 194，而同文件索引表那一行还写着「合计 186」。
#
# 射程（`.claude/singlefs-ai-sop/rules/show-me-test.md`「没实现的要明说」）：
#   1. 只认表头里「字段」一列紧跟着「宽」或「宽度」一列的表（前面可以有「段」这类列：
#      字节布局表的系统配置字段表就是 `| 段 | 字段 | 宽 |`），表后合计要在表后 3 行内、写成「合计 N 字节」或「合计 **N** 字节」。
#   2. 宽度格按整数或只含 ×、+、− 的整数算式读（「8 × 3」「12 + 16 + 1」）；表里只要有一行读不出来
#      （「未定」「随实现定」「—」），整张表跳过并计入跳过数：加不出确定的和时判红只会是假红。
#   3. 只判总宽与各行之和、标记与分项之和、布局里的总宽有没有登记；各行宽度本身对不对不判。
#   4. 一条「已定项 N」里恰好一个 `format-const` 标记时，标记值要等于该分项下
#      全部字段表之和（`DATA_UNIT_HEADER_BYTES = 105` 对着 D18 已定项 7 那两张表 42 + 63）。一个分项里
#      有多个标记时无从对应，跳过并报出——跳过的那些这一格一个字也没验。只有一个标记、而这一节的字段表
#      有宽度不是数的行或根本加不出和的，同样进跳过清单点名，不静默放过。
#   5. 扫两处的正文（「## 历史版本」之前）：`.claude/kb/decisions/*.md` 与 `.claude/kb/layout/*.md`。
#   6. 标记按 `lib-format-const.py` 读（code-source-discipline、checker-independence-and-sync 用的是同一份）：以 `<!-- format-const` 开头而按文法读不出来的、
#      同一份文件里同一个名字登记了不止一次的，判红——前者会让「一个分项里恰好一个标记」数错，
#      后者两个值里哪个算数这一格说不清。
#   7. 表前说明句：表头之前最多 3 行非空行里、以 `**` 起头的标题行（D18（块里携带什么信息） 已定项 7 那两张表的
#      「**类身份段——数据单元（+63 字节 ⇒ 共 105 字节初值）**：」）。认两种写法：
#      增量式「+M 字节 ⇒ 共 N 字节」判 M = 这张表之和、N = 这一小节（上一个标题起）到这张表为止全部字段表之和；
#      绝对式「合计 / 共 N 字节」「N 字节初值」「（N 字节，」「定长 N」判 N = 这张表之和。增量式认上了就不再按绝对式判。
#      不是 `**` 起头的普通段落不认：「记录定长 140」说的多半不是紧跟着的那张表。
#   8. 字节布局表（`.claude/kb/layout/*.md`）里的结构总宽要有登记。「结构总宽」只认三种写法：
#      ① 非表格行的「合计 N 字节」；② 以 ⇒ 起头的行里、紧挨着一个求和等式的加粗整数（「**88** = 头部 50 + …」
#      「24 + 88 = **112**」；等式那一侧去掉括注后要有 +，只有 × 的「6 × 55 = **330**」是条数乘宽、不算）；
#      ③ 带「状态」列的表里、状态以「已定」开头的行、宽度格里加粗的整数。
#      登记 = 同一行（①还可以是它那张表的说明句）里有值等于 N 的 `<!-- format-const: 名字 = N -->`，
#      或有「`format-const: 名字`」引用、而那个名字在 kb 正文里登记的值等于 N。缺即判红。
#      ⚠️ 认不出的写法（不加粗的总宽、「107 + 29」这种算式格、各字段自己的宽度）这一格一个字也不说；
#      成功那句报出认出了几处，认出 0 处不是「都登记了」。表后合计、表前说明句、标记、结构总宽四样一处都没认出：
#      无对象可判，退 77（不记通过）。表前表后都没写认得出的总宽的表，成功行下面逐个列表头行。
#
# 样本：field-table-sums-red 放一张合计写错的决策字段表、一个与表和对不上的标记、
# 一张合计写错的布局字段表、一条多写了键的标记与一个同一份文件里登记两次的名字；C94 要的「登记 78 而正文写着
# 两笔增量共 13 字节」（367-样例）；三句写错的表前说明句（369-样例：绝对式、增量、累计各一句）；
# 以及布局里没登记的三种结构总宽、引用了值不对的名字与没登记的名字、`| 段 | 字段 | 宽 |` 带乘式的表合计写错（02-样例），
# 必须判红。field-registry-green 放同一个增量样本而登记值改成 91（368-样例）、三种写法各一处登记好的总宽、
# 未定行里的加粗宽度与只有 × 的等式（不算），必须判绿。
cell_field_table_sums() {
  [[ -d .claude/kb/decisions ]] || { echo "  ! 找不到 decisions/，这一格无对象可判"; exit 77; }

  python3 - "$FORMAT_CONST_LIBRARY" <<'PY'
import re, glob, importlib.util, os, sys

library_spec = importlib.util.spec_from_file_location("format_const", sys.argv[1])
format_const = importlib.util.module_from_spec(library_spec)
library_spec.loader.exec_module(format_const)

SUM = re.compile(r"合计\s*\**\s*(\d+)\s*\**\s*字节")
HEAD = re.compile(r"^#{2,6}\s")
ITEM = re.compile(r"^#{3,6}\s*已定项\s*(\d+)")
# 表前说明句的写法（射程第 7 条）。增量式先认，认上了就不再按绝对式判同一句。
CAPTION_INCREMENT = re.compile(r"\+\s*(\d+)\s*字节\s*⇒\s*共\s*\**\s*(\d+)\s*\**\s*字节")
CAPTION_ABSOLUTE_FORMS = (
    re.compile(r"(?:合计|共)\s*\**\s*(\d+)\s*\**\s*字节"),
    re.compile(r"(\d+)\s*字节初值"),
    re.compile(r"（\s*(\d+)\s*字节[，；）]"),
    re.compile(r"定长\s*(\d+)"),
)
WIDTH_EXPRESSION = re.compile(r"\d+(?:\s*[×+−-]\s*\d+)*")
# 射程第 8 条：字节布局表里认得出的结构总宽
BOLD_WIDTH = re.compile(r"\*\*([^*\d]{0,4})(\d+)\*\*")
BOLD_PURE_WIDTH = re.compile(r"\*\*(\d+)\*\*")
CLAUSE_BOUNDARY = re.compile(r"[；;，,。：:⇒]")
PARENTHETICAL = re.compile(r"（[^（）]*）|\([^()]*\)")
REFERENCE = re.compile(r"format-const:\s*`?([A-Z][A-Z0-9_]*)")


def table_cells(line):
    return [cell.strip() for cell in line.strip().strip("|").split("|")]


def is_separator_row(cells):
    return set("".join(cells)) <= set("-: ")


def width_value(cell):
    """宽度格按整数或只含 ×、+、− 的整数算式读；读不出来返回 None。"""
    text = re.sub(r"[*`]", "", cell).strip()
    if not WIDTH_EXPRESSION.fullmatch(text):
        return None
    expression_value = 0
    for sign, term in re.findall(r"([+−-]?)\s*(\d+(?:\s*×\s*\d+)*)", text):
        product = 1
        for factor in re.split(r"\s*×\s*", term):
            product *= int(factor)
        expression_value += -product if sign in ("−", "-") else product
    return expression_value


def field_table_width_column(cells):
    """表头里「字段」紧跟着「宽 / 宽度」时，返回宽度列的下标。"""
    for index in range(len(cells) - 1):
        if cells[index] == "字段" and cells[index + 1] in ("宽", "宽度"):
            return index + 1
    return None


def caption_lines(lines, header_index):
    """表头之前的说明句：往上最多 3 行非空行，碰到表格行或标题就停。"""
    collected = []
    index = header_index - 1
    while index >= 0 and len(collected) < 3:
        line = lines[index]
        if line.startswith("|") or HEAD.match(line):
            break
        if line.strip():
            collected.append(index)
        index -= 1
    return sorted(collected)


def without_parentheticals(text):
    """去掉括注（可以嵌套）：「头部 50 + 位置条目 14 × 2（`format-const: LOC_ENTRY`）+ 写序 10」只剩算式。"""
    previous = None
    while previous != text:
        previous, text = text, PARENTHETICAL.sub("", text)
    return text


def strip_bold_and_code(text):
    return re.sub(r"[*`]", "", text)


def note_bold_width_in_settled_row(header, cells, line_index, layout_widths):
    """带「状态」列的表里，状态以「已定」开头的行，宽度格里加粗的整数算一处结构总宽。"""
    if is_separator_row(cells) or "状态" not in header:
        return
    width_names = [name for name in ("宽", "宽度") if name in header]
    if not width_names:
        return
    status_index, width_index = header.index("状态"), header.index(width_names[0])
    status = strip_bold_and_code(cells[status_index]) if status_index < len(cells) else ""
    if not status.startswith("已定") or width_index >= len(cells):
        return
    for bold in BOLD_PURE_WIDTH.finditer(cells[width_index]):
        layout_widths.append((line_index + 1, int(bold.group(1)), [line_index], "已定行的加粗宽度"))


# 登记值：kb 正文里全部读得出来的标记（与 code-source-discipline 同一个取法：不含变更史）
kb_body_paths = [path for path in sorted(glob.glob(".claude/kb/**/*.md", recursive=True))
                 if not path.endswith("-history.md") and "/decisions-history/" not in path]
registered_value_by_name = {}
for path in kb_body_paths:
    body_text = open(path, encoding="utf-8").read().split("\n## 历史版本")[0]
    for mark in format_const.parse_marks(body_text).marks:
        registered_value_by_name.setdefault(mark.name, mark.value)


def registrations_in(scope_text):
    """一段文字里的登记：标记（带值）与 `format-const: 名字` 引用（值取它在 kb 里登记的那个）。"""
    found = [(mark.name, mark.value) for mark in format_const.parse_marks(scope_text).marks]
    for name in REFERENCE.findall(format_const.strip_marks(scope_text)):
        found.append((name, registered_value_by_name.get(name)))
    return found


checked, skipped, tables_without_total, bad = 0, [], [], []
caption_checked = 0
mark_checked, mark_skipped = 0, []
marker_problems = []
layout_widths_checked, layout_unregistered = 0, []
decision_paths = sorted(glob.glob(".claude/kb/decisions/*.md"))
layout_paths = sorted(glob.glob(".claude/kb/layout/*.md"))

for kb_path in decision_paths + layout_paths:
    body = open(kb_path, encoding="utf-8").read().split("\n## 历史版本")[0]
    lines = body.split("\n")
    is_layout = kb_path in layout_paths
    parsed_marks = format_const.parse_marks(body)
    marks_by_line = {}
    for mark in parsed_marks.marks:
        marks_by_line.setdefault(mark.line_number, []).append((mark.name, mark.value))
    for unparsable in parsed_marks.unparsable:
        marker_problems.append(f"{kb_path}:{unparsable.line_number}  标记按文法读不出来：「{unparsable.excerpt}」")
    for duplicate in parsed_marks.duplicates:
        marker_problems.append(f"{kb_path}  {duplicate.name} 在这一份里登记了 {len(duplicate.line_numbers)} 次"
                               f"（第 {'、'.join(str(line_number) for line_number in duplicate.line_numbers)} 行）")
    # 分项区间：从「已定项 N」标题到下一个任意标题
    item, item_sum, item_marks, item_ok, item_head = None, 0, [], True, 0
    # 小节区间：从任意标题到下一个任意标题，给「+M 字节 ⇒ 共 N 字节」的累计用
    section_sum, section_ok = 0, True
    # 字节布局表里最近一张字段表：「合计 N 字节」那句的登记可以写在它的说明句里
    last_table_end, last_table_caption = -10, []
    layout_widths = []   # (行号, 值, 登记范围的行下标, 怎么认出来的)

    def close_item():
        global item, item_sum, item_marks, item_ok, item_head, mark_checked
        # 分项里有标记而没比成的，一律进跳过清单点名：宽度不是数、或这一节没有字段表，都不许静默放过
        if item is not None and item_marks:
            if len(item_marks) != 1:
                mark_skipped.append(f"{os.path.basename(kb_path)} 已定项 {item} 里有 "
                                    f"{len(item_marks)} 个 format-const 标记，对不上哪张表")
            elif not item_ok:
                mark_skipped.append(f"{os.path.basename(kb_path)} 已定项 {item} 的 format-const 标记 {item_marks[0][0]} 没核："
                                    f"这一节有宽度不是数的字段表，加不出确定的和")
            elif not item_sum:
                mark_skipped.append(f"{os.path.basename(kb_path)} 已定项 {item} 的 format-const 标记 {item_marks[0][0]} 没核："
                                    f"这一节没有加得出和的字段表")
            else:
                name, value_registered = item_marks[0]
                mark_checked += 1
                if value_registered != item_sum:
                    bad.append(f"{kb_path}:{item_head}  已定项 {item} 登记 {name} = {value_registered}，"
                               f"而这一节的字段表加起来是 {item_sum}")
        item, item_sum, item_marks, item_ok, item_head = None, 0, [], True, 0

    header_of_current_table = None
    line_index = 0
    while line_index < len(lines):
        line = lines[line_index]
        item_match = ITEM.match(line)
        if item_match:
            close_item()
            item, item_head = int(item_match.group(1)), line_index + 1
        elif HEAD.match(line) and item is not None:
            close_item()
        if HEAD.match(line):
            section_sum, section_ok = 0, True
        if item is not None:
            item_marks.extend(marks_by_line.get(line_index + 1, []))
        if not line.startswith("|"):
            header_of_current_table = None
            if is_layout:
                for sum_match in SUM.finditer(line):
                    scope = [line_index] + (last_table_caption if line_index - last_table_end <= 3 else [])
                    layout_widths.append((line_index + 1, int(sum_match.group(1)), scope, "合计"))
                if line.lstrip().startswith("⇒"):
                    for bold in BOLD_WIDTH.finditer(line):
                        after = CLAUSE_BOUNDARY.split(without_parentheticals(line[bold.end():]), 1)[0]
                        before = CLAUSE_BOUNDARY.split(without_parentheticals(line[:bold.start()]))[-1]
                        defines_by_sum = (after.lstrip().startswith("=") and "+" in after) or \
                                         (before.rstrip().endswith("=") and "+" in before)
                        if defines_by_sum:
                            layout_widths.append((line_index + 1, int(bold.group(2)), [line_index], "⇒ 等式"))
            line_index += 1
            continue
        cells = table_cells(line)
        if header_of_current_table is None:
            header_of_current_table = cells
        elif is_layout:
            note_bold_width_in_settled_row(header_of_current_table, cells, line_index, layout_widths)
        width_index = field_table_width_column(cells)
        if width_index is None:
            line_index += 1
            continue
        header_line_number = line_index + 1
        caption = caption_lines(lines, line_index)
        line_index += 2
        table_sum, unreadable_width, summed_rows = 0, None, 0
        while line_index < len(lines) and lines[line_index].startswith("|"):
            row_cells = table_cells(lines[line_index])
            if is_layout:
                note_bold_width_in_settled_row(cells, row_cells, line_index, layout_widths)
            cell = row_cells[width_index] if width_index < len(row_cells) else ""
            value = width_value(cell)
            if value is not None:
                table_sum += value; summed_rows += 1
            elif cell and not is_separator_row([cell]):
                unreadable_width = unreadable_width or (f"{os.path.basename(kb_path)}:{line_index+1} 「{strip_bold_and_code(row_cells[width_index - 1])[:40]}」"
                                  f"宽度是「{strip_bold_and_code(cell)[:40]}」")
            line_index += 1
        header_of_current_table = None
        last_table_end, last_table_caption = line_index - 1, caption
        if item is not None:
            if unreadable_width:
                item_ok = False
            else:
                item_sum += table_sum
        if unreadable_width:
            section_ok = False
        else:
            section_sum += table_sum
        # 表前说明句只认加粗起头的标题行（「**类身份段——数据单元（+63 字节 ⇒ 共 105 字节）**：」）：
        # 普通段落里的「记录定长 140」说的多半不是紧跟着的这张表
        caption_text = format_const.strip_marks(" ".join(lines[index] for index in caption
                                                         if lines[index].lstrip().startswith("**")))
        caption_judged = False
        increment_match = CAPTION_INCREMENT.search(caption_text)
        absolute_match = None
        if not increment_match:
            for form in CAPTION_ABSOLUTE_FORMS:
                absolute_match = form.search(caption_text)
                if absolute_match:
                    break
        if (increment_match or absolute_match) and unreadable_width:
            skipped.append(unreadable_width)
            caption_judged = True
        elif increment_match:
            caption_judged = True
            caption_checked += 1
            increment, cumulative = int(increment_match.group(1)), int(increment_match.group(2))
            if increment != table_sum:
                bad.append(f"{kb_path}:{header_line_number}  表前写「+{increment} 字节」，而这张字段表 {summed_rows} 行加起来是 {table_sum}")
            if not section_ok:
                skipped.append(f"{os.path.basename(kb_path)}:{header_line_number} 表前写「共 {cumulative} 字节」，"
                               f"而同一节前面有宽度不是数的字段表，累计加不出来")
            elif cumulative != section_sum:
                bad.append(f"{kb_path}:{header_line_number}  表前写「共 {cumulative} 字节」，而这一节到这张表为止的字段表加起来是 {section_sum}")
        elif absolute_match:
            caption_judged = True
            caption_checked += 1
            if int(absolute_match.group(1)) != table_sum:
                bad.append(f"{kb_path}:{header_line_number}  表前写「{absolute_match.group(0)}」，而这张字段表 {summed_rows} 行加起来是 {table_sum}")
        sum_match_after, sum_line_number = None, 0
        for following_index in range(line_index, min(line_index + 3, len(lines))):
            sum_match_after = SUM.search(lines[following_index])
            if sum_match_after:
                sum_line_number = following_index + 1
                break
        if not sum_match_after:
            if not caption_judged:
                tables_without_total.append(f"{os.path.basename(kb_path)}:{header_line_number}")
            continue
        if unreadable_width:
            if unreadable_width not in skipped:
                skipped.append(unreadable_width)
            continue
        checked += 1
        if int(sum_match_after.group(1)) != table_sum:
            bad.append(f"{kb_path}:{sum_line_number}  写「合计 {sum_match_after.group(1)} 字节」，而它上面那张字段表 {summed_rows} 行加起来是 {table_sum}"
                       f"（表头在第 {header_line_number} 行）")
    close_item()

    for line_number, value, scope, form in layout_widths:
        layout_widths_checked += 1
        found = registrations_in("\n".join(lines[index] for index in scope))
        if any(registered == value for _name, registered in found):
            continue
        shown = "、".join(f"{name} = {registered if registered is not None else '（kb 里没登记）'}"
                         for name, registered in found) or "一个都没有"
        layout_unregistered.append(f"{kb_path}:{line_number}  {form}写出宽度 {value}，这一行的登记：{shown}")

if bad:
    print(f"  ✗ 字段表加起来的数与写下来的对不上 {len(bad)} 处：")        # gate-lint:summary
    for entry in bad:
        print("      " + entry)                                              # gate-lint:detail
    print("  → 怎么办：加错了就改那个数；若是刚加了一行字段，那么**同一轮要一起改的还有**——")
    print("    该决策索引表那一行里的合计、这一节的 format-const 标记、表前说明句里的「+M 字节 ⇒ 共 N 字节」、")
    print("    [layout/01-first-txn.md] 对应那一节，以及全仓引过这个数的地方")
    print("    （`.claude/singlefs-ai-sop/rules/evidence-discipline.md`：撤回一个数要当场回扫谁在引它）。")
if marker_problems:
    print(f"  ✗ format-const 标记读不出来或在同一份文件里重复登记 {len(marker_problems)} 处：")  # gate-lint:summary
    for problem in marker_problems:
        print("      " + problem)                                        # gate-lint:detail
    print("  → 怎么办：读不出来的照 <!-- format-const: 名字 = 整数 stale=旧串|旧串 --> 改写，stale= 之外不许有别的键；")
    print("    它只是正文里举的例子、不是登记，就去掉 <!--，写成「`format-const: 名字 = 值`」。")
    print("    重复的只留定这个值的那一处，别处要提它写成不带 <!-- 的文字（例如「`format-const: 名字`」）。")
    print("    标记的文法只有一份，在 .claude/gate.d/lib-format-const.py，code-source-discipline、checker-independence-and-sync 按同一份读。")
if layout_unregistered:
    print(f"  ✗ 字节布局表里写出的结构总宽没有登记 {len(layout_unregistered)} 处：")  # gate-lint:summary
    for entry in layout_unregistered:
        print("      " + entry)                                          # gate-lint:detail
    print("  → 怎么办：在同一行（「合计 N 字节」那句也可以写在它那张表的说明句里）登记这个宽度：")
    print("    这个量还没有登记位，就在定它的那一处加 <!-- format-const: 名字 = N -->；")
    print("    已经在某份决策里登记过，就在这一行写「`format-const: 名字`」引用它，门禁按登记值比。")
    print("    名字与 crates/singlefs-format 里的 const 同名时，code-source-discipline 会拿登记值去核那个 const，要写成整数字面量。")
    print("    这个数其实不是一个结构的总宽，就别用加粗或「合计 N 字节」写它（C94（登记的格式常量与后来的定案对不上））。")
if bad or marker_problems or layout_unregistered:
    sys.exit(1)

# 四样一样都没核到：什么都没比过，不记通过（射程第 8 条：认出 0 处不是「都登记了」）
if checked + caption_checked + mark_checked + layout_widths_checked == 0:
    print(f"  ! 本次无对象可判：扫了 {len(decision_paths)} 份决策、{len(layout_paths)} 份字节布局表，"
          f"表后合计、表前说明句、format-const 标记、结构总宽一处都没认出来")
    sys.exit(77)

message = (f"  ✓ 字段表加出来的数都对得上（扫了 {len(decision_paths)} 份决策、{len(layout_paths)} 份字节布局表；"
           f"表后合计 {checked} 张，表前说明句 {caption_checked} 张，format-const 标记 {mark_checked} 个；"
           f"字节布局表里的结构总宽 {layout_widths_checked} 处都有登记）")
if skipped:
    message += f"，跳过 {len(skipped)} 处（有非数字宽度）"
if tables_without_total:
    message += f"，另有 {len(tables_without_total)} 张表前表后都没写认得出的总宽、这一格没验它们（表头行逐个列在下面）"
print(message)
for entry in skipped + mark_skipped:
    print(f"     ! 跳过：{entry}")
for entry in tables_without_total:
    print(f"     没验：{entry} 那张字段表，表前表后都没写认得出的总宽")
PY
}

# ── 格 first-txn-hooks：新池新建文件的三份文件互相挂钩（字节表 / 里程碑 / 决策索引）
#
# 三份文件说的是同一件事的三面：
#   decisions/*.md 里每条未定项判过「改新池新建文件的字节：是 / 否」（`doc-decisions.sh` 的 blocking-verdict 格管「判过没有」）；
#   layout/01-first-txn.md 把新池新建文件的每个字节列成表，判「是」的未定项在它的「还剩几处空白」表里各占一行；
#   milestone/01-first-txn.md 把写这些字节的活拆成步，每步写「写出的字节」在字节表哪几节、「会碰到的决策点」有哪些。
#
# 它拦的是三面对不上（2026-09-10 实测三处）：字节表的空白表里躺着一条已定案的分项（D8 已定项 7）、
# 判「是」的 D2 未定项 15 与 D19 未定项 8 在字节表与里程碑里一处都没有、
# 里程碑说「六个根槽」而字节表按已定项 8 只有三份。三处当时门禁全绿——没有任何阶段同时看两份文件。
#
# 判据（每条都报绝对数，扫到 0 也报出来）：
#   1. 判「是」且仍未定的每一条 `D<n> 未定项 <k>`，字节表与里程碑各至少引一次。
#   2. 字节表「还剩几处空白」表里引到的未定项，必须都在第 1 条那个集合里（定了就要从表里拿走）。
#   3. 字节表每一节（历史版本与空白表除外）标题下紧跟一行 `**里程碑**：… 步 N`，N 是里程碑里存在的 `## 步 N`。
#   4. 里程碑每个 `## 步 N` 有一行 `**写出的字节**：`，要么写「无」，要么点名字节表里存在的节标题（「…」引起来的那段）：
#      引起来的那段要与某一节的标题逐字相同，只许省掉标题末尾那一段全角括注（「七、发布」认「七、发布（根记录与根槽）」）；
#      只是标题的一截（「根」「发布」）不算点名。被字节表某节标成归属的步，不许写「无」。
#
# 射程（`.claude/singlefs-ai-sop/rules/show-me-test.md`「没实现的要明说」）：
#   只认「改新池新建文件的字节：**是**（日期…」这一种写法（doc-decisions 的 blocking-verdict 格定的规范形态，括号是判据的一部分）；
#   判定写在未定项小节之外的散文抓不到，转述别的分项判过「是」的引号句不算。
#   判「是」的未定项、字节表的节、里程碑的步三样一个都没有时，什么都没比，退 77（本次无对象可判），不报绿。
#   引用形态只认 `D<n>（简称） 未定项 <k>`，与 field-refs 那一格同一形态。不判引用处说的内容对不对。
#
# 样本：first-txn-hooks-red 里判「是」的 D401 未定项 1 里程碑没引、空白表引着判「否」的未定项 2、一节没标步号、
# 被标成归属的步 1 写「无」、步 2 只点了标题的一截；field-registry-green 里三面都对得上。
cell_first_txn_hooks() {
  local FTL=.claude/kb/layout/01-first-txn.md
  local MS=.claude/kb/milestone/01-first-txn.md
  local DEC=.claude/kb/decisions
  [[ -f "$FTL" && -f "$MS" && -d "$DEC" ]] || { echo "  ! 找不到 $FTL / $MS / $DEC，这一格无对象可判"; exit 77; }

  python3 - "$FTL" "$MS" "$DEC" <<'PY'
import re, sys, glob, os

ftl_path, ms_path, dec_dir = sys.argv[1:4]
ftl = open(ftl_path, encoding="utf-8").read().split("\n## 历史版本")[0]
ms = open(ms_path, encoding="utf-8").read().split("\n## 历史版本")[0]
REF = re.compile(r"D(\d+)（[^）]*）\s*未定项\s*(\d+)")
# 只认带日期括号的规范形态；「……判过「改新池新建文件的字节：是」」这种转述引用不算（D26 未定项 5 实测误判）
YES = re.compile(r"改新池新建文件的字节：\**是\**（")

# 1. 判「是」且仍未定的分项集合
blocking = set()
for p in sorted(glob.glob(os.path.join(dec_dir, "*.md"))):
    m = re.match(r"(\d+)-", os.path.basename(p))
    if not m:
        continue
    dnum = int(m.group(1))
    body = open(p, encoding="utf-8").read().split("\n## 历史版本")[0]
    sec = re.search(r"^### 未定项.*?(?=^### |\Z)", body, re.M | re.S)
    if not sec:
        continue
    entries = re.split(r"^(?=\| *\d+ *\||\s*\d+\. )", sec.group(0), flags=re.M)
    for e in entries:
        m2 = re.match(r"\| *(\d+) *\||\s*(\d+)\. ", e)
        if not m2:
            continue
        k = int(m2.group(1) or m2.group(2))
        if YES.search(e):
            blocking.add((dnum, k))

in_ftl = {(int(a), int(b)) for a, b in REF.findall(ftl)}
in_ms = {(int(a), int(b)) for a, b in REF.findall(ms)}
bad = []
for d, k in sorted(blocking):
    if (d, k) not in in_ftl:
        bad.append(f"D{d} 未定项 {k} 判了「改新池新建文件的字节：是」，而 {ftl_path} 一处都没引它")
    if (d, k) not in in_ms:
        bad.append(f"D{d} 未定项 {k} 判了「改新池新建文件的字节：是」，而 {ms_path} 一处都没引它")

# 2. 空白表只许装第 1 条那个集合里的分项
gap = re.search(r"^## 还剩几处空白.*?(?=^## |\Z)", ftl, re.M | re.S)
gap_refs = set()
if gap:
    for l in gap.group(0).splitlines():
        if l.startswith("|"):
            gap_refs |= {(int(a), int(b)) for a, b in REF.findall(l)}
for d, k in sorted(gap_refs - blocking):
    bad.append(f"{ftl_path} 的「还剩几处空白」表引着 D{d} 未定项 {k}，而它今天不在「判是且未定」的集合里（定了，或判的不是「是」）")

# 3. 字节表每节标里程碑步号
steps = {int(x) for x in re.findall(r"^## 步 (\d+)", ms, re.M)}
ftl_lines = ftl.splitlines()
sections, tagged_steps = 0, set()
for i, l in enumerate(ftl_lines):
    if not re.match(r"^#{2,3} ", l) or l.startswith("## 还剩几处空白"):
        continue
    sections += 1
    nxt = next((x for x in ftl_lines[i + 1:] if x.strip()), "")
    m = re.match(r"\*\*里程碑\*\*：.*步 (\d+)", nxt)
    if not m:
        bad.append(f"{ftl_path}:{i + 1} 「{l.strip()}」标题下面没有一行「**里程碑**：… 步 N」")
        continue
    ns = {int(x) for x in re.findall(r"步 (\d+)", nxt)}
    for n in ns:
        if n not in steps:
            bad.append(f"{ftl_path}:{i + 2} 标着里程碑步 {n}，而 {ms_path} 里没有「## 步 {n}」")
    tagged_steps |= ns

# 4. 里程碑每步一行「写出的字节」
headings = [re.sub(r"^#{2,3} ", "", l).strip() for l in ftl_lines if re.match(r"^#{2,3} ", l)]
ms_lines = ms.splitlines()
step_lines = [(i, int(m.group(1))) for i, l in enumerate(ms_lines) if (m := re.match(r"^## 步 (\d+)", l))]
for idx, (i, n) in enumerate(step_lines):
    end = step_lines[idx + 1][0] if idx + 1 < len(step_lines) else len(ms_lines)
    block = ms_lines[i:end]
    row = next((x for x in block if x.startswith("**写出的字节**：")), None)
    if row is None:
        bad.append(f"{ms_path}:{i + 1} 「## 步 {n}」里没有「**写出的字节**：」这一行")
        continue
    if re.match(r"\*\*写出的字节\*\*：无", row):
        if n in tagged_steps:
            bad.append(f"{ms_path}:{i + 1} 步 {n} 写「写出的字节：无」，而 {ftl_path} 有节标着归它")
        continue
    quoted = re.findall(r"「([^」]+)」", row)
    if "layout/01-first-txn.md" not in row or not quoted:
        bad.append(f"{ms_path}:{i + 1} 步 {n} 的「写出的字节」既不是「无」，也没点名 layout/01-first-txn.md 里的节")
        continue
    for q in quoted:
        # 逐字等于某一节的标题，或只省掉了标题末尾的全角括注；标题的一截（「根」之于「七、发布（根记录与根槽）」）不算
        if not any(h == q or (h.startswith(q) and h[len(q):].startswith("（") and h.endswith("）")) for h in headings):
            bad.append(f"{ms_path}:{i + 1} 步 {n} 点名「{q}」，而 {ftl_path} 没有标题是它的节（只许省掉标题末尾的全角括注）")

if bad:
    print(f"  ✗ 新池新建文件的三份文件对不上 {len(bad)} 处：")                    # gate-lint:summary
    for b in bad:
        print("      " + b)                                                     # gate-lint:detail
    print(f"  → 怎么办：判「是」的未定项要同时进 {ftl_path} 的「还剩几处空白」表和 {ms_path} 某一步的「会碰到的决策点」；")
    print("    定了案的就从空白表里拿走；字节表每节标题下写「**里程碑**：… 步 N」，里程碑每步写「**写出的字节**：无」或点名字节表的节标题。")
    sys.exit(1)

if not blocking and not sections and not steps and not gap_refs:
    print(f"  ! 本次无对象可判：{dec_dir} 里没有判「是」的未定项，{ftl_path} 没有节，{ms_path} 没有步，三样一样都没比")
    sys.exit(77)
print(f"  ✓ 新池新建文件的三份文件互相挂钩（判「是」的未定项 {len(blocking)} 条、字节表 {sections} 节、里程碑 {len(steps)} 步）")
PY
}

# ── 格 admission-terms：准入不等式的每一项都有人维护（统计量，或写明的例外）
#
# 判据（2026-09-13 用户定案，登记在 D5（快照 / 空间记账机制） 已定项 4）：
# 「式子里出现的每一项都必须是被维护的统计量，如果无法做到就写上例外是谁」。
#
# 做法：
#   1. 式子的权威正文前面有一行 `<!-- gate:admission-formula -->`，全仓只许一处；取它下面第一个代码块里 `可用 =` 那一行，
#      按 `−` 拆成项（去掉 `Σ设备(` 与括号）。
#   2. 对照表前面有一行 `<!-- gate:admission-terms -->`，全仓只许一处；取它下面第一张表的两列。
#   3. 两边的项必须逐一相等——式子加了一项而对照表没加、或对照表多出一项，都判红。
#   4. 对照表每一行的右列要么是「第 N 项（名字）」，N 在同一文件那张 `| # | 统计量 |` 表里、名字逐字相等、且不是已撤回的编号；
#      要么以 `**例外**：` 开头并写出理由（不少于 8 个字）。
#   5. `decisions/` 下凡是写着 `可用 = Σ设备` 的地方（索引行、挪走后留的指针），去掉空白与反引号之后必须与权威那一行逐字相等：
#      副本在反引号里的，取到收尾那个反引号；在表格行里、没有反引号的，取到那一格的竖线；都不是的取到行尾（末尾的句号、分号不算）。
#      多出一项、少一项都判红，不是只比开头。
#   两个锚不是各恰好一处（含一处都没有）判红，不退 77：锚没了就是式子与对照表脱了钩。decisions/ 不在时无对象可判，退 77。
#
# ⚠️ 这条是实测出来的：2026-09-13 立 D28（挂载期承诺量） 时往式子里加了第五项「挂载期承诺量」，
# 而 D5（快照 / 空间记账机制） 已定项 4 的完备性口径逐字是「式子里出现的每一项都必须是被维护的统计量」——
# 新项只住内存、不是统计量，那句话当场为假；同一轮回头看，式子里的「容量」也从来不是统计量，没有任何东西发现过。
#
# 射程（`.claude/singlefs-ai-sop/rules/show-me-test.md`「没实现的要明说」）：
#   只判「每一项都有归属、归属指得到」，不判归属对不对（某一项该不该是统计量、例外的理由成不成立，靠人）；
#   不判统计量表里多出来的项（那一半是「没出现在任何判定里的就不要」，判定不止准入一处，机器列不全）。
#
# 样本：admission-terms-red 的式子多了「挂载期承诺量」而对照表没有、副本在权威那一行后面多写一项；field-registry-green 三项各有归属、副本逐字一致。
cell_admission_terms() {
  [[ -d .claude/kb/decisions ]] || { echo "  ! 找不到 .claude/kb/decisions/，这一格无对象可判"; exit 77; }

  python3 - <<'PY'
import glob, re, sys

files = sorted(glob.glob('.claude/kb/decisions/*.md'))
texts = {path: open(path, encoding='utf-8').read() for path in files}

def anchors(marker):
    hits = []
    for path, text in texts.items():
        lines = text.split('\n')
        for number, line in enumerate(lines):
            if line.strip() == marker:
                hits.append((path, number, lines))
    return hits

problems = []

formula_hits = anchors('<!-- gate:admission-formula -->')
terms_hits = anchors('<!-- gate:admission-terms -->')
if len(formula_hits) != 1 or len(terms_hits) != 1:
    print(f'  ✗ 准入式子的锚 {len(formula_hits)} 处、对照表的锚 {len(terms_hits)} 处，各要恰好 1 处')
    print('     → 在准入不等式权威正文的代码块上一行写 `<!-- gate:admission-formula -->`，')
    print('       在逐项对照表上一行写 `<!-- gate:admission-terms -->`；两个锚全仓各一处，别处只留指针。')
    sys.exit(1)

formula_path, formula_line, formula_lines = formula_hits[0]
formula_text = None
inside_block = False
for line in formula_lines[formula_line + 1:]:
    if line.startswith('```'):
        if inside_block:
            break
        inside_block = True
        continue
    if inside_block and re.match(r'^可用(\(d\))?\s*=', line.strip()):
        formula_text = line.strip()
        break
if formula_text is None:
    print(f'  ✗ {formula_path}:{formula_line + 1} 的锚下面没有代码块，或代码块里没有 `可用 =` 那一行')
    print('     → 锚要紧挨着式子的代码块，式子写成一行：可用 = Σ设备( … ) − … 。')
    sys.exit(1)

right_hand_side = formula_text.split('=', 1)[1]
right_hand_side = right_hand_side.replace('(d)', '').replace('（d）', '')
right_hand_side = right_hand_side.replace('Σ设备(', ' ').replace('Σ设备（', ' ').replace('(', ' ').replace(')', ' ').replace('（', ' ').replace('）', ' ')
formula_terms = [re.sub(r'\s*÷\s*副本数$', '', term.strip()) for term in right_hand_side.split('−') if term.strip()]

terms_path, terms_line, terms_lines = terms_hits[0]
mapping = {}
table_started = False
for line in terms_lines[terms_line + 1:]:
    stripped = line.strip()
    if not stripped:
        if table_started:
            break
        continue
    if not stripped.startswith('|'):
        if table_started:
            break
        continue
    table_started = True
    cells = [cell.strip() for cell in stripped.strip('|').split('|')]
    if len(cells) < 2 or set(cells[0]) <= set('-: ') or cells[0] == '式子里的项':
        continue
    mapping[cells[0]] = cells[1]

statistics = {}
statistics_text = texts[terms_path].split('\n')
in_statistics_table = False
for line in statistics_text:
    if re.match(r'^\| # \| 统计量 \|', line):
        in_statistics_table = True
        continue
    if in_statistics_table:
        if not line.startswith('|'):
            in_statistics_table = False
            continue
        cells = [cell.strip() for cell in line.strip().strip('|').split('|')]
        if cells and cells[0].isdigit():
            statistics[int(cells[0])] = cells[1].replace('**', '')

if not statistics:
    problems.append(f'{terms_path}: 找不到表头是 `| # | 统计量 |` 的统计量表，对照表的「第 N 项」没处可指')

formula_set, mapping_set = set(formula_terms), set(mapping)
for term in formula_terms:
    if term not in mapping_set:
        problems.append(f'式子里有「{term}」，对照表（{terms_path}）里没有这一行')
for term in mapping:
    if term not in formula_set:
        problems.append(f'对照表（{terms_path}）里有「{term}」，式子（{formula_path}）里没有这一项')

exceptions = []
for term, owner in mapping.items():
    if owner.startswith('**例外**：'):
        reason = owner[len('**例外**：'):].strip()
        if len(reason) < 8:
            problems.append(f'「{term}」写了例外，但理由不到 8 个字：{owner}')
        exceptions.append(term)
        continue
    match = re.match(r'^第 (\d+) 项（(.+)）$', owner)
    if not match:
        problems.append(f'「{term}」的归属写成了「{owner}」，要么「第 N 项（名字）」、要么「**例外**：理由」')
        continue
    number, name = int(match.group(1)), match.group(2)
    if number not in statistics:
        problems.append(f'「{term}」指到第 {number} 项，统计量表里没有这一行')
    elif '已撤回' in statistics[number]:
        problems.append(f'「{term}」指到第 {number} 项，而那一项已撤回')
    elif statistics[number] != name:
        problems.append(f'「{term}」写的是第 {number} 项（{name}），统计量表里第 {number} 项叫「{statistics[number]}」')

canonical = re.sub(r'[\s`]', '', formula_text)
copies = 0
for path, text in texts.items():
    for number, line in enumerate(text.split('\n'), 1):
        position = line.find('可用(d) =')
        if position < 0:
            position = line.find('可用 = Σ设备')
        if position < 0:
            position = line.find('可用 =Σ设备')
        if position < 0:
            continue
        copies += 1
        fragment = line[position:]
        if '`' in line[:position]:
            fragment = fragment.split('`')[0]
        elif line.lstrip().startswith('|'):
            fragment = fragment.split('|')[0]
        fragment = re.sub(r'[\s`]', '', fragment).rstrip('。；;，,')
        # 逐字相等，不是只比开头：副本在权威那一行后面多写一项（「… − 多一项」）也要红
        if fragment != canonical:
            problems.append(f'{path}:{number} 的式子副本与权威那一行（{formula_path}）对不上')

if problems:
    print(f'  ✗ 准入不等式与统计量清单的逐项对照有 {len(problems)} 处不对')
    for problem in problems:
        print(f'     {problem}')  # gate-lint:detail
    print('     → 式子里每一项都要在对照表里有一行：被维护的统计量写「第 N 项（名字）」，做不到的写「**例外**：理由」')
    print('       （2026-09-13 用户定案，D5（快照 / 空间记账机制） 已定项 4）；式子改了，先改对照表，再改各处副本。')
    sys.exit(1)

print(f'  ✓ 准入不等式 {len(formula_terms)} 项逐项有归属：统计量 {len(formula_terms) - len(exceptions)} 项，写明的例外 {len(exceptions)} 项（{"、".join(exceptions)}）；式子副本 {copies} 处与权威一致')
PY
}

# ── 格 segment-registry：段序列登记表与 E142 产物逐字比对
#
# 判据（C316（提交步骤的登记位有四处且互不相同） 欠账的第①半）：
#   .claude/kb/layout/01-first-txn.md 「八、根槽写路径的段序列登记表」里每一行的段序列数字串
#   （`4+1+1+1+2` 这种）都是人从 E142（新池新建文件的干跑） 产物里抄过来的——抄错一位，
#   或者产物重跑之后表没跟着改，此前没有任何东西会报警。
#
# 做法：转发给 research/scripts/check-segment-registry.py，逻辑不写第二份
#   （表怎么解析、path 怎么从「出处」栏原样抠出来、第四条产物路径怎么按排除法认领，
#   都写在那个脚本自己的文档字符串与注释里）。它的退出码 1（对不上）与 2（结构认不出、文件不在）这一格都记红。
#
# 该脚本自己的 --selftest（改坏拷贝里的一个段序列数字，确认判红；未改动的拷贝确认判绿）
# 由 code-tooling 阶段（三方论证 research 脚本的自证）复跑，这里不重复跑一遍。
#
# ⚠️ 脚本按这一道阶段自己的位置取仓里的那一份（$SEGMENT_REGISTRY_SCRIPT），不按 cwd 取；`--root` 指被判的仓。
#   判别力样本会把 cwd 换成样本目录，按 cwd 取就成了样本里那份拷贝：仓里的脚本退化了（例如 main() 不再传退出码），
#   样本照样判对，这一格与 code-tooling 的 --selftest 一起放过（三方判决 research/prompts/gate-fix-forks-r1-main-verification.md 的 T6）。
#   同一个坑见 `doc-decisions.sh` 里 blocking-verdict 那一格的注释。脚本随仓走，不在就是被删了或挪了——退 1 不退 77，与 checker-tier-lkmm、code-tooling、harness-test-environment、这一道的 citations 那一格同一条
#   （三方判决 gate-fix-forks-r1 的 T7）。
#
# 样本：segment-registry-red 只放脚本读的三样合成输入——layout 表的「八、」一节（十行表、一句 path 声明、
#   一句整条流）、replay.sh 里 E142 那一行、产物里几行 name=segments。十行各埋一种错（段序列、种类、操作数、状态数、
#   产物里没有这条 path、没写段序列、产物这一行没有 kinds、钉住的用例文件不在、用例里没有那个函数、出处既无 path 也无用例），
#   另把整条流的状态数写错、第二条流的写数与数组与用例里的状态数各错一处，脚本每条逐行比对的分支都有一处会红，逐条点名；
#   field-registry-green 一致，判绿（三方判决 gate-fix-forks-r3 的 T6）。「声明的 path 集合剩不下恰好一条」走退 2 那一支，放不进这份红样本，没有样本格。
#   钉活代码的那几句与真产物的解析由脚本自己的 --selftest（code-tooling 跑）拿真文件测，样本不再测一遍。
cell_segment_registry() {
  if [[ ! -f "$SEGMENT_REGISTRY_SCRIPT" ]]; then
    echo "  ✗ 找不到 $SEGMENT_REGISTRY_SCRIPT"
    echo "     → 怎么办：它随仓走（research/scripts/ 下），不在就是被删了或挪了：从 git 找回来，挪了就改这个阶段里 SEGMENT_REGISTRY_SCRIPT 那一行的路径。"
    exit 1
  fi
  python3 "$SEGMENT_REGISTRY_SCRIPT" --root "$ROOT" || exit 1
}

# ── 格 format-const-placeholders：格式常量文件里的占位，每个都指得到一条真实存在的分项或欠账
#
# 里程碑「新池新建文件」步 0 要求：一个格式常量文件，新池新建文件要写的每个宽度都在里面，
# 占位的常量带占位标记与分项号；门禁数出常量文件里还有几个占位，每个占位都指得到分项号，指不到判红。
# 占位的写法（crates/singlefs-format/src/*.rs）：常量前一行
#   // placeholder: D22（单元原子性怎么合成） 已定项 2 —— 为什么还是预想
#   // placeholder: C323（镜像大小全仓没有条款） —— 为什么还是预想
# 文档注释写法（`/// placeholder:`、`//! placeholder:`）一样认作占位，一样要指得到。
# 判据：编号带简称；D<n> 的分项号在 .claude/kb/decisions/<n>-*.md 的「### 已定项 / ### 未定项」两张表里能找到那一行，
# C<n> 在 .claude/kb/checks-owed.md 里有登记行；简称与登记位一致由 doc-lint 管，这里只核编号与分项号。
# 成功那句报出检查了多少个占位；没有常量目录、或一个占位都没有时退 77（本次无对象可判），不报绿。
#
# 样本：format-const-placeholders-red 里一个占位指已定项 99、一个把只在已定项表里的第 2 行写成未定项、一个没写编号、
# 一个文档注释写法的指到不存在的 D99；field-registry-green 里一个 D 占位、一个 C 占位都指得到。
cell_format_const_placeholders() {
  [[ -d crates/singlefs-format/src ]] || { echo "  ! 没有 crates/singlefs-format/src，这一格无对象可判（步 0 之前没有常量文件）"; exit 77; }

  python3 - <<'PY'
import glob, os, re, sys
bad = []
placeholders = []
for path in sorted(glob.glob('crates/singlefs-format/src/**/*.rs', recursive=True)):
    for number, line in enumerate(open(path, encoding='utf-8'), 1):
        m = re.match(r'\s*//[/!]?\s*placeholder:\s*(.*)$', line)
        if not m:
            continue
        text = m.group(1).strip()
        placeholders.append((path, number, text))
        d = re.match(r'D(\d+)（[^）]+）\s*(已定项|未定项)\s*(\d+)', text)
        c = re.match(r'C(\d+)（[^）]+）', text)
        if d:
            n, kind, k = d.group(1), d.group(2), d.group(3)
            files = glob.glob(f'.claude/kb/decisions/{int(n):02d}-*.md')
            if not files:
                bad.append((path, number, f'D{n} 没有决策文件')); continue
            body = open(files[0], encoding='utf-8').read()
            # 索引行 `| k | **…` 或正文标题 `#### 已定项 k（`，两种形态任一命中即可。
            # 索引行只在同名那一节（`### 已定项` / `### 未定项`）第一个 #### 之前找：不分节时 `未定项 2` 会被已定项表的第 2 行放行
            section = re.search(rf'^### {kind}[ \t]*$(.*?)(?=^#{{2,4}} |\Z)', body, re.M | re.S)
            index_table = section.group(1) if section else ''
            hit = re.search(rf'^\|\s*{k}\s*\|', index_table, re.M) or re.search(rf'^#{{3,5}}\s*{kind}\s*{k}[（:：]', body, re.M)
            if not hit:
                bad.append((path, number, f'D{n} 里找不到 {kind} {k}'))
        elif c:
            n = c.group(1)
            owed = open('.claude/kb/checks-owed.md', encoding='utf-8').read()
            if not re.search(rf'^\|\s*C{n}\s*\|', owed, re.M):
                bad.append((path, number, f'checks-owed.md 里找不到 C{n} 的登记行'))
        else:
            bad.append((path, number, '占位没写成「D<n>（简称） 已定项/未定项 k」或「C<n>（简称）」'))
if bad:
    print(f'  ✗ 格式常量文件里有 {len(bad)} 个占位指不到分项或欠账：')
    for path, number, why in bad:
        print(f'     {path}:{number}  {why}')  # gate-lint:detail
    print('     → 怎么办：占位那一行写成「// placeholder: D<n>（简称） 已定项 k —— 为什么还是预想」或「// placeholder: C<n>（简称） —— …」，')
    print('       编号要在 .claude/kb/decisions/ 的索引表或 .claude/kb/checks-owed.md 里真的有那一行；分项定了就把占位行删掉。')
    sys.exit(1)
if not placeholders:
    print('  ! 本次无对象可判：crates/singlefs-format/src 下一个 `// placeholder:` 占位都没有，没有编号可核')
    sys.exit(77)
print(f'  ✓ 格式常量文件里的占位都指得到分项或欠账（{len(placeholders)} 个占位：' + '；'.join(t.split(' —— ')[0] for _, _, t in placeholders) + '）')
PY
}

# ── 格 second-txn-hooks：覆盖写、释放、回退与复用的三处互相挂钩
#
# 里程碑「覆盖写、释放、回退与复用」的字节登记在三处：`layout/02-second-txn.md` 那张表登记第一次出现的盘上形态，
# 每行指里程碑的一步与钉住它的用例；里程碑每步的「**写出的字节**：」点名 layout/01 的节与 layout/02 的行；
# `layout/01-first-txn.md`「八、根槽写路径的段序列登记表」的行标「里程碑「覆盖写、释放、回退与复用」步 N」。
# first-txn-hooks 那一格只管新池新建文件那三份（里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 31 行）。
#
# 判据（每条都报绝对数）：
#   0. layout/02 表每一行都要六格（形态 / 第一次出现在哪次发布 / 写成什么 / 决策分项 / 里程碑那一步 / 钉住它的用例），少一格判红，不静默丢掉。
#   1. layout/02 表每一行「里程碑那一步」点名的步 N 在里程碑里存在，且步 N 的「写出的字节」行用「」点名了这一行的形态
#      （前缀即可，但这个前缀只许对得上一行：同时是几行形态的前缀，就指不出点的是哪一行，不算点名，判红）。
#   2. 里程碑里每一行「写出的字节」用「」点名的东西，要么是 layout/01 的节标题（子串即可），要么是 layout/02 表某一行形态的前缀
#      （同样只许对得上一行）；点名 layout/02 某一行的步 N，那一行的「里程碑那一步」里要有步 N。
#   3. layout/02 表「钉住它的用例」里反引号的 `*.rs` 在 `crates/*/tests/` 下存在，反引号的函数名在同一格点名的文件里有 `fn 名字`。
#   4. 步 N 的「写出的字节」点名了 layout/01 的「八、」那一节，当且仅当「八、」那张表有一行写「里程碑「覆盖写、释放、回退与复用」步 N」。
#
# 射程：只认「」引起来的点名与反引号里的用例名；点名的内容与字节对不对，这里不判（字节由用例与层 0 钉）。
# 三份文件缺一份：这一格无对象可判，退 77。
#
# 样本：second-txn-hooks-red 里一行指的步没点名它、一步点名了不存在的节、用例文件不在、用例函数改了名、
# 「八、」表标着的步没点名那一节、点名「八、」的步表里没有、一行只有四格、点名写成两行的共同前缀；field-registry-green 三处互相点名。
cell_second_txn_hooks() {
  local L1=.claude/kb/layout/01-first-txn.md
  local L2=.claude/kb/layout/02-second-txn.md
  local MS=.claude/kb/milestone/02-second-txn.md
  [[ -f "$L1" && -f "$L2" && -f "$MS" ]] || { echo "  ! 找不到 $L1 / $L2 / $MS，这一格无对象可判"; exit 77; }

  python3 - "$L1" "$L2" "$MS" <<'PY'
import glob, re, sys

l1_path, l2_path, ms_path = sys.argv[1:4]
body = lambda path: open(path, encoding="utf-8").read().split("\n## 历史版本")[0]
l1, l2, ms = body(l1_path), body(l2_path), body(ms_path)
headings = [re.sub(r"^#{2,3} ", "", line).strip() for line in l1.splitlines() if re.match(r"^#{2,3} ", line)]
split_cells = lambda line: [cell.strip() for cell in re.split(r"(?<!\\)\|", line.strip().strip("|"))]
plain = lambda text: text.replace("`", "")

rows, in_table = [], False
for line in l2.splitlines():
    if line.startswith("| 形态 |"):
        in_table = True
        continue
    if in_table and line.startswith("|---"):
        continue
    if in_table and line.startswith("|"):
        rows.append(split_cells(line))
        continue
    in_table = False

bad = []
if not rows:
    bad.append(f"{l2_path} 里找不到表头是「| 形态 |」的登记表，或表里一行都没有")
short_rows = [row for row in rows if len(row) < 6]
for row in short_rows:
    bad.append(f"{l2_path}「{row[0][:30]}」这一行只有 {len(row)} 格，登记表每行要 6 格（形态 / 第一次出现在哪次发布 / 写成什么 / 决策分项 / 里程碑那一步 / 钉住它的用例）")
rows = [row for row in rows if len(row) >= 6]

ms_lines = ms.splitlines()
step_starts = [(index, int(match.group(1))) for index, line in enumerate(ms_lines) if (match := re.match(r"^## 步 (\d+)", line))]
step_bytes_line = {}
for position, (start, number) in enumerate(step_starts):
    end = step_starts[position + 1][0] if position + 1 < len(step_starts) else len(ms_lines)
    step_bytes_line[number] = next((line for line in ms_lines[start:end] if line.startswith("**写出的字节**：")), "")

def rows_named_by(quote):
    """「」里的串点名 layout/02 的哪一行：它是那一行形态的前缀，而且只对得上这一行。同时是几行的前缀就指不出是哪一行，不算点名。"""
    matched = [row for row in rows if plain(row[0]).startswith(plain(quote))]
    return matched if len(matched) == 1 else []

def ambiguous_prefix(quote):
    matched = [row for row in rows if plain(row[0]).startswith(plain(quote))]
    return matched if len(matched) > 1 else []

# 1. 登记表一行 → 里程碑那一步
row_links = 0
for row in rows:
    for number in [int(x) for x in re.findall(r"步 (\d+)", row[4])]:
        row_links += 1
        if number not in step_bytes_line:
            bad.append(f"{l2_path}「{row[0][:30]}」指里程碑步 {number}，而 {ms_path} 里没有「## 步 {number}」")
            continue
        quotes = re.findall(r"「([^」]+)」", step_bytes_line[number])
        if not any(row in rows_named_by(quote) for quote in quotes):
            bad.append(f"{l2_path}「{row[0][:30]}」指里程碑步 {number}，而步 {number} 的「写出的字节」没点名这一行")

# 2. 里程碑每行「写出的字节」的点名都落得到
quotes_checked = 0
for index, line in enumerate(ms_lines):
    if not line.startswith("**写出的字节**："):
        continue
    step_number = next((number for start, number in reversed(step_starts) if start < index), None)
    for quote in re.findall(r"「([^」]+)」", line):
        quotes_checked += 1
        named_rows = rows_named_by(quote)
        shared_prefix = ambiguous_prefix(quote)
        if shared_prefix:
            bad.append(f"{ms_path}:{index + 1} 点名「{quote[:40]}」，而它是 {l2_path} 里 {len(shared_prefix)} 行形态的共同前缀"
                       f"（{'、'.join(row[0][:20] for row in shared_prefix)}），指不出点的是哪一行，不算点名")
            continue
        if not named_rows and not any(quote in heading for heading in headings):
            bad.append(f"{ms_path}:{index + 1} 点名「{quote[:40]}」，而 {l1_path} 没有这一节、{l2_path} 表里也没有这一行")
            continue
        for row in named_rows:
            if step_number is not None and str(step_number) not in re.findall(r"步 (\d+)", row[4]):
                bad.append(f"{ms_path}:{index + 1} 步 {step_number} 点名 {l2_path}「{row[0][:30]}」，而那一行的「里程碑那一步」里没有步 {step_number}")

# 3. 钉住它的用例在仓里
tests_checked = 0
for row in rows:
    tokens = re.findall(r"`([^`]+)`", row[5])
    files = [token for token in tokens if token.endswith(".rs")]
    functions = [token for token in tokens if re.fullmatch(r"[a-z_][a-z0-9_]*", token)]
    paths = []
    for file_name in files:
        tests_checked += 1
        found = glob.glob(f"crates/*/tests/{file_name}")
        if not found:
            bad.append(f"{l2_path}「{row[0][:30]}」点名的用例文件 {file_name} 在 crates/*/tests/ 下不存在")
        paths += found
    sources = [open(path, encoding="utf-8").read() for path in paths]
    for function in functions:
        tests_checked += 1
        if not any(re.search(rf"\bfn {re.escape(function)}\b", source) for source in sources):
            bad.append(f"{l2_path}「{row[0][:30]}」点名的用例 {function} 在同一格点名的文件里找不到 fn")

# 4. 段序列登记表的行与里程碑各步对得上
registry = re.search(r"^## 八、.*?(?=^## |\Z)", l1, re.M | re.S)
registry_steps = set()
if registry:
    for line in registry.group(0).splitlines():
        if line.startswith("|"):
            registry_steps |= {int(x) for x in re.findall(r"里程碑「覆盖写、释放、回退与复用」\s*步 (\d+)", line)}
else:
    bad.append(f"{l1_path} 里没有「## 八、」那一节（段序列登记表）")
steps_naming_registry = {number for number, line in step_bytes_line.items() if re.search(r"「八、[^」]*段序列登记表", line)}
for number in sorted(registry_steps - steps_naming_registry):
    bad.append(f"{l1_path}「八、」有一行标着里程碑「覆盖写、释放、回退与复用」步 {number}，而步 {number} 的「写出的字节」没点名「八、」那一节")
for number in sorted(steps_naming_registry - registry_steps):
    bad.append(f"{ms_path} 步 {number} 的「写出的字节」点名了「八、」那一节，而那张表里没有一行标「里程碑「覆盖写、释放、回退与复用」步 {number}」")

if bad:
    print(f"  ✗ 覆盖写、释放、回退与复用的三处挂钩对不上 {len(bad)} 处：")    # gate-lint:summary
    for item in bad:
        print("      " + item)                                # gate-lint:detail
    print(f"  → 怎么办：{l2_path} 每一行的「里程碑那一步」与那一步的「**写出的字节**：」互相点名（「形态」原样引）；")
    print(f"    点名的节要在 {l1_path} 里存在，用例文件与函数名要在 crates/*/tests/ 里存在；")
    print(f"    某一步写了「八、」段序列那一节，那张表就要有一行标「里程碑「覆盖写、释放、回退与复用」步 N」，反过来也一样。")
    sys.exit(1)

print(f"  ✓ 覆盖写、释放、回退与复用的三处互相挂钩（登记表 {len(rows)} 行、行到步 {row_links} 处、「写出的字节」点名 {quotes_checked} 处、用例 {tests_checked} 个、段序列登记表标到的步 {len(registry_steps)} 个）")
PY
}

# ── 格 tree-table-reserve：树表条目预留字节的认购合计（认购不许超过预留，每条要指得到一个编号）
#
# C338（树表条目预留没有余量）欠的那一半：认购表立成正文之后，要有一道算和的检查。
# 2026-09-22 立表时当场抓到一处：正文列的是 8 + 8 + 8 + 16 + 8 = 48，而它自己写「共 56」——
# 那个 8 是 C143（inode 号水位在回退后会退回去重发）2026-09-16 定案「水位照旧住记账」之前的旧账，
# 定案之后没人回来改合计。靠人算迟早再漂，所以这个数由这一格来算。
#
# 表的形态（.claude/kb/decisions/08-核心索引结构.md 已定项 11）：
#   <!-- gate:tree-table-reserve total=76 -->
#   | 认购者 | 住哪条条目 | 宽度 | 出处 |
#   |---|---|---|---|
#   | … | 头条目 | 8 | C271（…） |
# 判据三条：① 宽度列全是正整数；② 合计不超过标记里的 total；③ 每行「出处」要带一个编号
#（C<n> 在 .claude/kb/checks-owed.md 有登记行，D<n> 在 .claude/kb/decisions/<n>-*.md 存在）。
# 带标记的表不是恰好一张（含一张都没有）判红；认购行不足四格（少了宽度或出处）判红，不静默丢掉；
# 标记下面一条认购都没有，本次无对象可判，退 77。
# 内嵌 python 的退出码要取，而且必须报出 CHECKED 那一行：它崩了（例如读不到 checks-owed.md）时不许走绿。
# 成功那句报出检查了几条认购、合计多少、余多少。
#
# 样本：tree-table-reserve-red 里认购合计 88 超过预留 76、一行只有三格；field-registry-green 三条认购合计 32、余 44。
cell_tree_table_reserve() {
  local report python_rc
  report="$(python3 - <<'PY'
import glob, re, sys

marker = re.compile(r"<!--\s*gate:tree-table-reserve\s+total=(\d+)\s*-->")
files = sorted(glob.glob(".claude/kb/decisions/*.md"))
found = []
for path in files:
    lines = open(path, encoding="utf-8").read().split("\n")
    for index, line in enumerate(lines):
        matched = marker.search(line)
        if matched:
            found.append((path, index, int(matched.group(1)), lines))

if len(found) != 1:
    print("BAD", f"带 gate:tree-table-reserve 标记的表应当恰好有一张，实际 {len(found)} 张", sep="\t")
    print("CHECKED", 0, 0, 0, sep="\t")
    sys.exit()

path, index, total, lines = found[0]
rows = []
for line in lines[index + 1:]:
    if not line.startswith("|"):
        if rows:
            break
        continue
    cells = [cell.strip() for cell in re.split(r"(?<!\\)\|", line.strip().strip("|"))]
    if cells[0] in ("认购者",) or cells[0].startswith("---"):
        continue
    if len(cells) < 4:
        print("BAD", f"「{cells[0]}」这一行只有 {len(cells)} 格，认购表每行要四格（认购者 / 住哪条条目 / 宽度 / 出处），它的宽度没进合计", sep="\t")
        continue
    rows.append(cells)

owed = open(".claude/kb/checks-owed.md", encoding="utf-8").read()
subtotal = 0
for cells in rows:
    name, width_text, source = cells[0], cells[2], cells[3]
    if not re.fullmatch(r"\d+", width_text):
        print("BAD", f"「{name}」的宽度列不是正整数：{width_text}", sep="\t")
        continue
    subtotal += int(width_text)
    numbers = re.findall(r"(C\d{1,3}|D\d{1,2})（", source)
    if not numbers:
        print("BAD", f"「{name}」的出处里没有带简称的编号", sep="\t")
        continue
    for number in numbers:
        if number.startswith("C"):
            if f"| {number} |" not in owed:
                print("BAD", f"「{name}」指的 {number} 在 checks-owed.md 里没有登记行", sep="\t")
        elif not (glob.glob(f".claude/kb/decisions/{int(number[1:])}-*.md")
                  or glob.glob(f".claude/kb/decisions/{int(number[1:]):02d}-*.md")):
            print("BAD", f"「{name}」指的 {number} 在 decisions/ 下没有正文", sep="\t")

if subtotal > total:
    print("BAD", f"认购合计 {subtotal} 超过预留 {total}", sep="\t")
print("CHECKED", len(rows), subtotal, total, sep="\t")
PY
)"
  python_rc=$?
  if ((python_rc != 0)) || ! grep -q '^CHECKED' <<<"$report"; then
    printf '%s\n' "$report" | tail -5 | sed 's/^/      /'   # gate-lint:detail
    echo "  ✗ 认购表没算完：内嵌 python 退出码 $python_rc，$(grep -q '^CHECKED' <<<"$report" && echo '报了' || echo '没报出') CHECKED 那一行"
    echo "    → 怎么办：单独跑一遍这一格（--check tree-table-reserve）看 python 的报错（多半是 .claude/kb/checks-owed.md 或决策文件读不到）；"
    echo "      没算完就是什么都没查，成功句里的数会是空的，而它看着与判过了一模一样。"
    exit 1
  fi

  local -a bad
  local rows subtotal total why
  mapfile -t bad < <(grep '^BAD' <<<"$report")
  read -r _ rows subtotal total < <(grep '^CHECKED' <<<"$report")

  if ((${#bad[@]})); then
    echo "  ✗ 树表条目预留的认购表不合规："
    while IFS=$'\t' read -r _ why; do
      printf '      %s\n' "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${bad[@]}")
    echo "    → 怎么办：认购表在 .claude/kb/decisions/08-核心索引结构.md 已定项 11，标记那一行写着 total=<预留字节数>。"
    echo "      合计超了就先去掉一个认购者（或把那一段的宽度压小），别直接改 total——预留多少由那条已定项定；"
    echo "      出处指不到编号的，补一个还开着的欠账号或一条已定分项，编号要带简称。"
    exit 1
  fi
  if [[ "$rows" == 0 ]]; then
    echo "  ⊘ 本次无对象可判：gate:tree-table-reserve 标记下面一条认购都没有（预留 ${total}）"
    exit 77
  fi
  echo "  ✓ 树表条目预留的认购表对得上（${rows} 条认购，合计 ${subtotal} 字节、预留 ${total}、余 $((total - subtotal))；每条的出处都指得到一个编号）"
}

# ════════════════ 欠账与收口组的五格（欠账表与里程碑收口表） ════════════════

# ── 格 table-shape：欠账表两张登记表的行形状（已还清那张不许混进六列的欠账行，欠着那张不许留写着已还的行）
#
# 判据：`.claude/kb/checks-owed.md` 有两张登记表——欠着的那张六列
# （编号 / 简称 / 要拦什么 / 怎么拦 / 前置 / 出处），「### 已还清」那张四列
# （编号 / 简称 / 怎么还的 / 还清日期）。一行的格数与它所在那张表的表头对不上就判红；
# 欠着的那张里，一行有哪一格（第 3 列起）开头写着
# 「已还」而后面不是「一半 / 这一半 / 大半」、或开头写着「销账」「作废」（三个词前面带 ⚠️ 与日期都算），也判红——还清了就挪走，
# 只还了一部分就把开头写成还欠什么（「已还一半」「决策已定、检查仍欠」「前置已还」）。
# 只看紧挨着 `doc-lint:registry` 标记的表，历史版本里别的表不管；带编号行而没判的表，成功行逐张列出（现算）。
# 「同一个编号在两张表里各登记一次」不在这一格判：两张表都挨着 `doc-lint:registry` 标记，每一行都是 kb 的登记位，
# 上游 doc-lint 的 F（一个编号只许有一处登记位）逐个编号判它；编号后面接全角空格的那一行 doc-lint 登记不上，
# 由它的 G 报成裸引用（门禁收缩判决 research/prompts/gate-shrink-r1-main-verification.md 3-B3，攻方 M7 V3）。
#
# ⚠️ 实测（2026-09-11）：C248–C273 这 26 行全是六列的欠账行，却一行接一行追加在
# 「已还清」那张四列表的末尾——几个会话往文件末尾追加时都没看它落在哪张表下面。
# 于是一条没还的欠账（C266、C272）按小节检索时挂在「已还清」标题下，
# 渲染时多出来的两格被截掉，而此前没有任何一道检查看这件事。
#
# ⚠️ 实测（2026-09-19）：反过来的形态。C273（小节清单不认一级标题） 2026-09-11 就还清了
# （自检接进门禁 code-tooling），行首写着「已还（2026-09-11……）」却一直留在欠着的那张表里，
# 门禁 doc-registries 照样把它算进「欠检查 N 条」；同一张表里另有九行开头写「已还（…）」，
# 核下来只还了决策、前置或一半，写法与已还清表同一个词，按行检索分不开。
# 2026-09-19 又查出三行（C137 / C150 / C151）第一格写「⚠️ 2026-09-07 作废……此编号不再使用」却留在欠着的那张表里：
# 同一族的 C135 / C138 当天就按同一理由挪进了已还清，这三行被算进「欠检查」的数里一个多月。
# 同一天还查出欠着的那张表中间夹着 6 个空行：Markdown 在空行处结束表格，第 87 行起约 270 行
# 渲染不成表，这一格也只把第一段当登记表判，后面几段一行都没判过。所以编号行前面没有表头也判红。
#
# 样本：checks-owed-red 的欠账表里欠着那张有一行写「已还（日期）」、一行「日期 作废」、一行「日期 销账」，一行被空行断开、前面没有表头，
# 已还清那张混进一行六格的；checks-owed-green 里欠着那张的「已还一半」「已还这一半」「前置已还」、已还清那张里写「销账」的都放行。
cell_table_shape() {
  local F="$ROOT/.claude/kb/checks-owed.md"
  if [[ ! -f "$F" ]]; then
    echo "  ✗ 找不到 $F"
    echo "  → 怎么办：欠账表挪了位置就同步改这个阶段里的路径。"
    exit 1
  fi
  python3 - "$F" <<'PY'
import re, sys
lines = open(sys.argv[1], encoding='utf-8').read().split('\n')
tables, cur, section, marked = [], None, '（开头）', False
headless, in_history = [], False
for i, l in enumerate(lines, 1):
    s = l.strip()
    if s.startswith('<!-- doc-lint:registry'):
        marked = True; cur = None; continue
    if l.startswith('#'):
        section = l.lstrip('#').strip(); cur = None; marked = False
        in_history = in_history or section.startswith('历史版本')
        continue
    if not s.startswith('|'):
        cur = None
        if s:
            marked = False
        continue
    cells = re.sub(r'\\\|', '', s).strip('|').split('|')
    if cur is None:
        # 一张表的第一行就是编号行，说明它前面没有表头：多半是被一个空行从上一张表断开的。
        # Markdown 在空行处结束表格，断开之后的行渲染不成表，也不在 registry 标记管的那张表里。
        if re.match(r'\|\s*C\d+\s*\|', s) and not in_history:
            headless.append(f'第 {i} 行 {s.split("|")[1].strip()}：前面没有表头（上一行是空行或别的正文）')
        cur = {'sec': section, 'width': len(cells), 'head': i, 'rows': [], 'reg': marked}
        tables.append(cur); marked = False; continue
    if re.fullmatch(r'\|(\s*:?-+:?\s*\|)+', s):
        continue
    m = re.match(r'\|\s*(C\d+)\s*\|', s)
    if m:
        cur['rows'].append((i, m.group(1), cells))
reg = [t for t in tables if t['reg'] and t['rows']]
if len(reg) < 2:
    print(f'  ✗ 只找到 {len(reg)} 张带编号行的登记表，欠着的与已还清的两张应当都在')
    print('  → 怎么办：表头或它上面的 doc-lint:registry 标记被改坏了；按文件开头的约定把两张表恢复出来。')
    sys.exit(1)
# 欠着的那张里，一格开头写「已还」（后面不是一半 / 这一半 / 大半）或「销账」，就是在说这一行已经不欠了。
paid_claim = re.compile(r'(?:20\d\d-\d\d-\d\d\s*)?(已还(?!一半|这一半|大半)|销账|作废)')
bad, claims, checked = [], [], 0
for t in reg:
    for ln, num, cells in t['rows']:
        checked += 1
        if len(cells) != t['width']:
            bad.append(f"第 {ln} 行 {num}：{len(cells)} 格，而它所在那张表（「{t['sec']}」下、表头在第 {t['head']} 行）是 {t['width']} 格")
        if t['sec'] == '已还清':
            continue
        for column_index, cell in enumerate(cells[2:], start=3):
            lead = re.sub(r'^[\s*⚠️]+', '', cell)
            if paid_claim.match(lead):
                claims.append(f"第 {ln} 行 {num}：第 {column_index} 列开头写着「{lead.replace('*', '')[:16]}…」")
if bad or claims or headless:
    print(f'  ✗ 欠账表有 {len(bad)} 行的格数与所在表的表头对不上、{len(claims)} 处还在欠着的表里却写着已还、{len(headless)} 处被空行断开：')  # gate-lint:summary
    for b in bad:
        print('      ' + b)  # gate-lint:detail
    for claim in claims:
        print('      ' + claim)  # gate-lint:detail
    for fragment in headless:
        print('      ' + fragment)  # gate-lint:detail
    print('  → 怎么办：六列的欠账行挪回欠着的那张表，接在它最后一行后面，不接在文件末尾；表中间的空行删掉，一张表从表头到最后一行不许断；')
    print('            真还清了的，按「已还清」那张表的四列改写（编号 / 简称 / 怎么还的 / 还清日期）再挪过去，欠着的那张里删掉；')
    print('            只还了一部分的，把那一格的开头改成写明还欠什么的说法（「已还一半」「决策已定、检查仍欠」「前置已还」）。')
    sys.exit(1)
print(f'  ✓ 欠账表两张登记表的行形状都对，欠着的那张里没有写着已还的行（检查了 {checked} 行，{len(reg)} 张表）')
# 没判的：带 C 编号行、却不紧挨着 registry 标记的表（历史版本里的、别处举例的），逐张列出，清单与被扫集合出自同一趟解析
unjudged_tables = [t for t in tables if not t['reg'] and t['rows']]
print(f"    没判 {len(unjudged_tables)} 张带编号行的表（没有紧挨着的 doc-lint:registry 标记）"
      + ('：' + '；'.join(f"「{t['sec']}」下表头在第 {t['head']} 行、{len(t['rows'])} 行" for t in unjudged_tables) if unjudged_tables else ''))
PY
}

# ── 格 closeout-collects-open：里程碑收口表收全了文件里点名、还开着的欠账号，行号只许是顺序号
#
# 实测（2026-09-17 核出）：.claude/kb/milestone/02-second-txn.md「增补 2」的收口表立表时 13 行，
# 同一文件别处点名、.claude/kb/checks-owed.md 里还开着的 C 编号（C374、C329、C330、C331、C287 等）表里一个字没提，
# 当天派四个只读核查才补进第 5–34 行；此前没有任何检查报警（那张表第 34 行）。
#
# 判据：
#   ① 收口表用一行标记指明：`<!-- milestone:closeout-table -->`，紧挨着写在表上方（中间只许空行），一份里程碑文件至多一处。
#   ② 还开着的欠账号：checks-owed.md 里「### 已还清」标题之前、表格首列是 C<编号> 的行
#      （按 `lib-owed.py` 读，checker-independence-and-sync、code-source-discipline 用的是同一份，不另抄）。
#   ③ 带标记的文件，全文（含历史版本节）出现的每个 C<编号>，若还开着，要么在收口表的某一行里出现，
#      要么在表后（表结束到下一个标题之前）有一行显式豁免，形态：`- 不收口 C<编号>（简称）：为什么不收`。
#   任一个开着的编号两处都没有 ⇒ 红。豁免行写坏（没有编号、冒号后面是空的）、同一个编号既豁免又进了表、
#   标记后面不是表、一份文件两处标记 ⇒ 也红。
#   ④ 收口表每一行的首列（表头 `#` 与分隔行除外）要么是纯十进制数字，要么是圆圈号 `①`–`⑳`
#      （那一档是这张表早先给「被打回的项」留的）；带撇号（U+2032、U+2033、ASCII 单引号）或紧跟字母的 ⇒ 红，指名那一行。
#      用户 2026-09-23 定：行号全改成顺序号，以后禁止后缀。当天现查，加撇号的那几行与它们的基号毫不相干，
#      撇号的实际含义是「这个号被占了」，读的人却当成从属关系，「跟第 N 行一起判」就指到一件不相干的事上；字母后缀同病。
#      全仓的 U+2032、U+2033 另由 doc-text 判；这一条多管的是 ASCII 单引号、字母与其余一切非顺序号的写法，只在收口表首列。
#
# 不判的：没有标记的里程碑文件，成功行逐个列出（现算）；一份带标记的文件都没有 ⇒ 这一格退 77，不记通过。
# ⚠️ 管不到的：表里那一行写的去向对不对、豁免的理由站不站得住（靠人）；编号只按字面 C<数字> 认，
# 写成「C 374」或只写简称的引用它看不见；已还清的编号、checks-owed.md 里没有的编号一律不管。
# 样本：checks-owed-red 放一个漏收的开着编号、一个既豁免又进表的编号、一行空理由的豁免、
# 一份标记后面没有表的文件、一行字母后缀与一行 ASCII 单引号后缀的行号，必须判红；checks-owed-green 放表里一个、豁免一个、已还清一个、
# 一份不带标记的文件，收口表里一个圆圈号、一个一位数与一个两位数的行号，必须判绿并报对数
# （checks-owed-green 里 closeout-row27-preconditions 那一格的 02-second-txn.md 也带标记，这一格一并判它）。
cell_closeout_collects_open() {
  python3 - "$OWED_LIBRARY" <<'PY'
import glob, importlib.util, os, re, sys

owed_library_spec = importlib.util.spec_from_file_location("owed", sys.argv[1])
owed_library = importlib.util.module_from_spec(owed_library_spec)
owed_library_spec.loader.exec_module(owed_library)

milestone_dir = ".claude/kb/milestone"
owed_path = ".claude/kb/checks-owed.md"
marker = "<!-- milestone:closeout-table -->"
owed_number = re.compile(r"(?<![A-Za-z0-9_])(C[0-9]+)(?![0-9])")
exemption_start = re.compile(r"^- 不收口 ")
exemption_form = re.compile(r"^- 不收口 (?P<number>C[0-9]+)(?:（.*?）)?：(?P<reason>.*\S.*)$")
circled_row_numbers = "①②③④⑤⑥⑦⑧⑨⑩⑪⑫⑬⑭⑮⑯⑰⑱⑲⑳"

milestone_files = sorted(glob.glob(os.path.join(milestone_dir, "*.md")))
if not milestone_files:
    print(f"  ! {milestone_dir} 下没有里程碑文件，本阶段无对象可判")
    sys.exit(77)

marked_files = []
unmarked_files = []
for path in milestone_files:
    lines = open(path, encoding="utf-8").read().split("\n")
    if any(line.strip() == marker for line in lines):
        marked_files.append((path, lines))
    else:
        unmarked_files.append(os.path.basename(path))

if not marked_files:
    print(f"  ! {milestone_dir} 下 {len(milestone_files)} 份里程碑文件没有一份带收口表标记 {marker}，本阶段无对象可判；没判：{'、'.join(unmarked_files)}")
    print(f"    要让本阶段判一份里程碑：在它的收口表上方单独写一行 {marker}，表后逐行写豁免「- 不收口 C<编号>（简称）：为什么不收」。")
    sys.exit(77)

if not os.path.isfile(owed_path):
    print(f"  ✗ 没有 {owed_path}，分不出哪些欠账号还开着")
    print(f"     → 怎么办：确认欠账表还在 {owed_path}；搬过家就照 .claude/rules/path-moves.md 改，并改本阶段的路径。")
    sys.exit(1)

owed_table = owed_library.read_owed_table(owed_path)
open_owed_names = owed_table.open_names
found_paid_heading = owed_table.paid_heading_found
if not found_paid_heading or not open_owed_names:
    print(f"  ✗ {owed_path} 里认不出还开着的欠账：找到「已还清」标题 {found_paid_heading}，标题之前首列是 C<编号> 的表格行 {len(open_owed_names)} 行")
    print(f"     → 怎么办：本阶段按「### 已还清 之前、表格首列是 C<编号>」认开着的账；欠账表改了形状，就同步改本阶段的解析，别让它对着一张认不出的表判绿。")
    sys.exit(1)

failed = False
per_file_counts = []
for path, lines in marked_files:
    name = os.path.basename(path)
    marker_line_indexes = [index for index, line in enumerate(lines) if line.strip() == marker]
    if len(marker_line_indexes) > 1:
        failed = True
        print(f"  ✗ {path} 有 {len(marker_line_indexes)} 处收口表标记（第 {', '.join(str(index + 1) for index in marker_line_indexes)} 行）")
        print(f"     → 怎么办：一份里程碑只留一张收口表、一处标记；另一张表要收的项并进收口表。")
        continue
    index = marker_line_indexes[0] + 1
    while index < len(lines) and not lines[index].strip():
        index += 1
    if index >= len(lines) or not lines[index].lstrip().startswith("|"):
        failed = True
        print(f"  ✗ {path} 第 {marker_line_indexes[0] + 1} 行的收口表标记后面第一处非空内容不是表格")
        print(f"     → 怎么办：把 {marker} 挪到收口表表头的正上方（中间只许空行）；还没有表就先删掉标记，本阶段对这份文件报未判。")
        continue
    table_start = index
    while index < len(lines) and lines[index].lstrip().startswith("|"):
        index += 1
    table_end = index
    table_text = "\n".join(lines[table_start:table_end])
    numbers_in_table = set(owed_number.findall(table_text))

    # ④ 行号：表头 `#` 与分隔行之外，每一行的首列只许是顺序号或圆圈号。
    suffixed_row_numbers = []
    judged_row_count = 0
    for row_index in range(table_start, table_end):
        first_cell = lines[row_index].split("|")[1].strip()
        if not first_cell or first_cell.startswith("-") or first_cell == "#":
            continue
        judged_row_count += 1
        if first_cell.isdecimal() or (len(first_cell) == 1 and first_cell in circled_row_numbers):
            continue
        suffixed_row_numbers.append(f"第 {row_index + 1} 行：行号写成「{first_cell}」")

    exempted_numbers = {}
    malformed_exemptions = []
    while index < len(lines) and not lines[index].startswith("#"):
        line = lines[index]
        if exemption_start.match(line):
            match = exemption_form.match(line)
            if match:
                exempted_numbers[match.group("number")] = index + 1
            else:
                malformed_exemptions.append(f"第 {index + 1} 行：{line}")
        index += 1

    first_line_by_number = {}
    for line_number, line in enumerate(lines, 1):
        for number in owed_number.findall(line):
            first_line_by_number.setdefault(number, line_number)
    open_numbers = sorted((number for number in first_line_by_number if number in open_owed_names), key=lambda number: int(number[1:]))
    missing_numbers = [number for number in open_numbers if number not in numbers_in_table and number not in exempted_numbers]
    contradicted_numbers = sorted((number for number in exempted_numbers if number in numbers_in_table), key=lambda number: int(number[1:]))

    if malformed_exemptions:
        failed = True
        print(f"  ✗ {path} 收口表后面这些豁免行写坏了（没有编号，或冒号后面没写为什么不收）：")  # gate-lint:summary
        for entry in malformed_exemptions:
            print(f"     {entry}")  # gate-lint:detail
        print("     → 怎么办：一行豁免一个编号，形态「- 不收口 C<编号>（简称）：为什么不收」，冒号后面写它归哪个里程碑、为什么不在这里还。")
    if contradicted_numbers:
        failed = True
        print(f"  ✗ {path} 这些编号豁免了，又写进了收口表——两处说反话：")  # gate-lint:summary
        for number in contradicted_numbers:
            print(f"     {number}（{open_owed_names.get(number, '已还清或欠账表里没有')}）：豁免在第 {exempted_numbers[number]} 行")  # gate-lint:detail
        print("     → 怎么办：这个里程碑要还它，就删掉豁免行；不还，就把它从收口表里拿掉，豁免行写清归哪。")
    if missing_numbers:
        failed = True
        print(f"  ✗ {path} 点名了这些还开着的欠账号，收口表里没有、表后也没有豁免：")  # gate-lint:summary
        for number in missing_numbers:
            print(f"     {number}（{open_owed_names[number]}）：第 {first_line_by_number[number]} 行起出现")  # gate-lint:detail
        print("     → 怎么办：在收口表里给它一行（或并进已有的一行，写清性质与去向），")
        print("               或在表后加一行「- 不收口 C<编号>（简称）：为什么不收」；它其实已经还了，就先把 checks-owed.md 那一行挪进「### 已还清」。")
    if suffixed_row_numbers:
        failed = True
        print(f"  ✗ {path} 收口表的行号带了后缀（撇号或字母），行号只许是顺序号或圆圈号：")  # gate-lint:summary
        for entry in suffixed_row_numbers:
            print(f"     {entry}")  # gate-lint:detail
        print("     → 怎么办：改成一个没被占用的顺序号（接着表里最大的那个往下取），不要在旧号上加撇号或字母——后缀会让读的人以为这一行从属于那个基号。")
        print("               ⚠️ 改号要连全仓引用一起改（「第 N 行」「跟第 N 行」这类），照 .claude/rules/path-moves.md「怎么做」那六步办。")
    per_file_counts.append(f"{name}：全文点名 {len(first_line_by_number)} 个 C 编号，其中还开着 {len(open_numbers)} 个：表里 {len([number for number in open_numbers if number in numbers_in_table])} 个、豁免 {len([number for number in open_numbers if number in exempted_numbers and number not in numbers_in_table])} 个；收口表 {judged_row_count} 行的行号都是顺序号或圆圈号")

if failed:
    sys.exit(1)
print(f"  ✓ 里程碑收口表收全了文件里点名、还开着的欠账号（判了 {len(marked_files)} 份；{'；'.join(per_file_counts)}）")
print(f"    没判 {len(unmarked_files)} 份（没有收口表标记）：{'、'.join(unmarked_files) if unmarked_files else '无'}")
PY
}

# ── 格 paid-cited-tests：已还清的欠账行里点名的测试名，仓里还在不在
#
# 判据：`.claude/kb/checks-owed.md` 的「### 已还清」那张表里，反引号括起来的 snake_case 标识符
# （形如 `a_b_c`，至少三段）必须在仓里的 `.rs` / `.py` / `.sh` 的代码里按整词找得到；找不到就红。
# 按整词（`a_b_c_v2` 不算 `a_b_c` 还在）、只认注释之外（`.rs` 的 `//` 之后、`.py` / `.sh` 的 `#` 之后不算：
# 旧名只剩在一句注释里，那个测试其实已经没了）。已还清表一行都没有、或行里一个这样的标识符都没有，这一格本次无对象可判，退 77。
# 一条已还清的行说「某某单测钉着这件事」，那个单测正是这条定案**唯一的记录位**——它被改名或删掉时，
# 今天没有任何东西会说话。
#
# 射程只到「已还清」那张表。**欠着的那张表不查**：那张表写的正是「知道要拦什么但还拦不了」，
# 里面点名一个还不存在的测试名是它应有的样子，拿这条判据去扫它一定是错的。
#
# 为什么只认「至少三段」的 snake_case：2026-09-20 在真语料上量过假阳性——已还清表 44 行、
# 命中 6 个标识符、仓里找不到的 1 个，而那 1 个正是要拦的那种（C313 行引的那条单测名，
# E142 第六次跑之后状态数变了、单测跟着改了名，这条已还清的行没跟上）。
# 放宽到两段会把 `log_append`、`write_cache` 这类词组也扫进来。
#
# ⚠️ 注释里不写被拦的那个标识符的整串字面量，搜索也排掉 `.claude/gate.d/`：第一版在注释里写了，
# 于是 grep 扫到这个阶段自己、把该红的那一条判成「还在」（与 `pgrep -f` 匹配到自己的命令行同形）。
# 门禁脚本里出现一个测试名，本来也不构成「这个测试存在」的证据——测试住在 crates/ 与 research/。
#
# 不扫 `research/prompts/`：当时原样发给模型的材料是冻结证据，里面引的旧名字本来就不该跟着改
# （`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「原样保存的证据不许事后改」）。
#
# 样本：checks-owed-red 的已还清表里一行引的单测改了名、一行引的旧名只剩在注释里、一行引的名字只剩加了后缀的，三个都要点名；
# 欠着那张里一行引一个不存在的名字，不许点名。checks-owed-green 里已还清表引的单测在、另有不带标识符的行进「没查的」。
cell_paid_cited_tests() {
  local OWED=".claude/kb/checks-owed.md"
  if [[ ! -f "$OWED" ]]; then
    echo "  ! 没有 $OWED，本阶段跳过"
    exit 77
  fi

  python3 - "$OWED" <<'PY'
import pathlib, re, subprocess, sys

owed = pathlib.Path(sys.argv[1])
lines = owed.read_text(encoding="utf-8").splitlines()

settled_heading = None
for index, line in enumerate(lines):
    if line.strip().startswith("### 已还清"):
        settled_heading = index
        break
if settled_heading is None:
    print("  ✗ 找不到「### 已还清」那一节，判不了已还清的行引的测试名还在不在")
    print("     → 怎么办：确认 .claude/kb/checks-owed.md 里那一节的标题没被改写；改了标题就把这一格的取法一起改。")
    raise SystemExit(1)

identifier = re.compile(r"`([a-z][a-z0-9]*(?:_[a-z0-9]+){2,})`")
row_number = re.compile(r"^\|\s*(C\d+)\s*\|")

rows = []
for line in lines[settled_heading + 1:]:
    stripped = line.strip()
    if stripped.startswith("## ") or (stripped.startswith("### ") and not stripped.startswith("### 已还清")):
        break
    # 只认首格是 `C<数字>` 的登记行。表头（真表写 `| # | 简称 | …`）与分隔行不是登记行：
    # 数进去会让成功句里报的行数比真实行数多，而那个数没有任何东西钉着
    # （`.claude/singlefs-ai-sop/rules/show-me-test.md`「成功那句里报的数也要有东西钉住」）。
    # 一行掉了编号算表的形状坏了，那是 table-shape 那一格判的事，这一格不重复判。
    if stripped.startswith("|") and stripped.count("|") >= 4 and row_number.match(stripped):
        rows.append(stripped)

cited = []
skipped = []
for row in rows:
    number = row_number.match(row).group(1)
    names = list(dict.fromkeys(identifier.findall(row)))  # 同一行里引两次算一个
    if names:
        for name in names:
            cited.append((number, name))
    else:
        skipped.append(number)

if not rows:
    print("  ⊘ 本次无对象可判：「### 已还清」那张表里一行首格是 C<编号> 的登记行都没有")
    raise SystemExit(77)
if not cited:
    print(f"  ⊘ 本次无对象可判：已还清的 {len(rows)} 行里没有一个至少三段的 snake_case 标识符："
          + "、".join(skipped))
    raise SystemExit(77)

def code_part(line, suffix):
    """去掉注释之后的那一截：.rs 砍掉 `//` 之后，.py / .sh 砍掉行首或空白之后的 `#` 之后。"""
    if suffix == ".rs":
        return line.split("//", 1)[0]
    return re.split(r"(?:^|\s)#", line, maxsplit=1)[0]

def is_in_the_repository(name):
    # grep 只用来缩小候选（按字面、按整词）；注释里的命中在下面逐行剔掉
    found = subprocess.run(
        ["grep", "-rnwF", "--include=*.rs", "--include=*.py", "--include=*.sh",
         "--exclude-dir=prompts", "--exclude-dir=target", "--exclude-dir=.git",
         "--exclude-dir=gate.d", "--", name, "."],
        capture_output=True, text=True)
    if found.returncode not in (0, 1):
        print(f"  ✗ grep 找 `{name}` 时出错（退 {found.returncode}）：{found.stderr.strip()[:120]}")
        print("     → 怎么办：按 grep 的报错修好（读不了的目录、权限），再跑；找的时候出错不等于没找到，也不等于找到了。")
        raise SystemExit(1)
    word = re.compile(r"(?<![A-Za-z0-9_])" + re.escape(name) + r"(?![A-Za-z0-9_])")
    for hit in found.stdout.splitlines():
        path, _, rest = hit.partition(":")
        _, _, text = rest.partition(":")
        if word.search(code_part(text, pathlib.Path(path).suffix)):
            return True
    return False

missing = [(number, name) for number, name in cited if not is_in_the_repository(name)]

if missing:
    for number, name in missing:
        print(f"  ✗ {number} 引的 `{name}` 在仓里的 .rs / .py / .sh 里一处都找不到")  # gate-lint:detail
    print(f"  ✗ 已还清的行里有 {len(missing)} 个点名的标识符已经不在仓里了")  # gate-lint:summary
    print("     → 怎么办：那条欠账是靠这个测试还清的。先 grep 它今天叫什么（多半是改了名），")
    print("               用 research/scripts/replace-once.py 把行里的旧名字定点换成现在的名字；")
    print("               它要是被删掉了，那这条欠账并没有还清，把整行挪回欠着的那张表。")
    raise SystemExit(1)

print(f"  ✓ 已还清的行引的测试名都还在（{len(rows)} 行，查了 {len(cited)} 个标识符："
      + "、".join(f"{number} {name}" for number, name in cited) + "）")
print(f"     没查的 {len(skipped)} 行（行里没有至少三段的 snake_case 标识符）："
      + "、".join(skipped))
PY
}

# ── 格 audit-contradictions：最近一次总审核登记的文档级矛盾，每一行都要有去向
#
# `records/` 里的总审核记录第五节登记文档级矛盾。这些矛盾**在 kb 里一个落点都没有**，
# 只活在那份记录里：没有任何检查会在它们腐化时变红，而其中几条按记录自陈会误导实现者
# （C358（总审核第五节的文档级矛盾没有回扫闸））。
#
# 这一格判：第五节表里每一行的处置列，要么以「已改」开头，要么点名一个 checks-owed.md
# **开着那张表**里的欠账号。只点名已还清的号不算去向——那笔账还了，这一行的矛盾却还在。
# 与 closeout-collects-open 那一格的分工：那一格的对象是里程碑文件点名的开着欠账号；这一格的对象是总审核记录第五节的矛盾行。
#
# 「已改」还要指得到今天的正文（C358（总审核第五节的文档级矛盾没有回扫闸） 的原话）：处置列要用「」引至少一句原文，每一句归一之后
# （去掉 ** 与反引号、空白压成一个）都要在这一行点名的 kb 文件的今天正文里找得到——点名指这一行四列里
# 出现的 D / E 编号（→ decisions/、experiments/ 下同号的那份）、C 号（→ checks-owed.md）、I- 号
# （→ invariants.md）与直接写出的 .md 文件名；四列里一份 kb 文件都没点名的，才在 kb 全部正文里找。
# 正文 = 「## 历史版本」之前，不含 decisions-history/ 与 *-history.md。
# 引号后面紧跟「0 命中」「不在」「没有了」的是在说那句话已经删掉，不算指向正文的引文；
# 这类「已经不在」的说法这一格**不核**（「0 命中」在哪个范围里 0 命中，处置列写不清，按整份文件判会误红）。
# 只判引文找不找得到，不判引文说的是不是那一行的矛盾——后者要人看。
# 引文里去掉编号（D / E / C / I- 号连同简称括注）与 .md 文件名之后，至少要剩两个字：只引一个编号（「D8」）
# 或一个字，在点名的那份文件里总找得到，指不出今天正文里的任何一句话。
#
# 第五节的标题认「## 五」后面跟标点、空白或行尾的写法（「## 五、」「## 五.」「## 五 文档级矛盾」都认）；
# 最近一次总审核里认不出第五节标题判红，认出了而表里一行矛盾都没有，这一格本次无对象可判，退 77。
# 欠账表开着与已还清的切法用共用读法 lib-owed.py（「已还清」整行标题为界，认不出标题判红），不在这里按子串切。
#
# 挂账表 .claude/audit-rows-pending 2026-09-23 已清空并关闭：立闸那天挂着的 26 行逐行现查过，
# 全部给了去向。表里再出现任何一行非注释内容就判红——新的矛盾行一律当场给去向，不许挂账。
#
# 样本：checks-owed-red 放一行「未改」且不给欠账号、一行只点名已还清的号、
# 一行挂进已关闭的挂账表、一行「已改」却不引原文、两行「已改」引的原文在点名的文件里找不到而只在别的文件里有
# （一行按 D 编号点名、一行按文件名点名）、一行引的原文哪里都没有、一行只引了一个编号「D9」，第五节标题写成
# 「## 五 文档级矛盾」（不带顿号），必须判红；checks-owed-green 放按 D 编号与按文件名点名、
# 原文都在点名的文件里各一行，一行没点名文件而原文在 kb 别处，一行带「「旧说法」0 命中」而 kb 里没有「旧说法」，
# 欠账表开着的那张里有一行正文提到「### 已还清」这几个字（按子串切会把它后面开着的账算成还清），必须判绿。
#
# 内嵌 python 崩了不许走绿：它的退出码要取，而且必须报出 COUNT 那一行，缺一样判红
# （2026-09-23 门禁审计那一轮查出：不取退出码时，python 崩了成功句照印、只是数变空白）。
cell_audit_contradictions() {
  local PENDING=".claude/audit-rows-pending"
  local latest report
  latest="$(ls -1 records/*-总审核.md 2>/dev/null | sort | tail -1)"
  if [[ -z "$latest" ]]; then
    echo "  ⊘ 本次未跑：records/ 下没有总审核记录，今天无对象可判"
    exit 77
  fi

  if ! report="$(LATEST="$latest" PENDING="$PENDING" OWED_LIBRARY="$OWED_LIBRARY" python3 - 2>&1 <<'PY'
import glob, importlib.util, os, re

latest = os.environ["LATEST"]
pending_path = os.environ["PENDING"]
owed_library_spec = importlib.util.spec_from_file_location("owed", os.environ["OWED_LIBRARY"])
owed_library = importlib.util.module_from_spec(owed_library_spec)
owed_library_spec.loader.exec_module(owed_library)

# 挂账表已关闭：任何一行非注释内容都是新挂的账。
if os.path.exists(pending_path):
    for number, line in enumerate(open(pending_path, encoding="utf-8"), 1):
        stripped = line.strip()
        if stripped and not stripped.startswith("#"):
            print("BAD", f"{pending_path}:{number}",
                  "挂账表 2026-09-23 已清空并关闭，不许再加行；这一行的矛盾要当场给去向", sep="\t")

# 只认开着那张表：「已还清」整行标题之后的号是还清了的，点名它们不算去向。切法用共用读法 lib-owed.py：
# 按子串「### 已还清」切，开着的那张表里有一行正文提到这几个字，它后面开着的账就全被算成还清。
owed_table = owed_library.read_owed_table(".claude/kb/checks-owed.md")
if not owed_table.file_found:
    print("BAD", ".claude/kb/checks-owed.md", "欠账表不在，分不出哪些欠账号还开着", sep="\t")
elif not owed_table.paid_heading_found:
    print("BAD", ".claude/kb/checks-owed.md", "认不出「已还清」那一行整行标题，分不出哪些欠账号还开着（不对着一张认不出的表判）", sep="\t")
open_owed = set(owed_table.open_names) if owed_table.paid_heading_found else set()
repaid_owed = set(owed_table.paid_names)

lines = open(latest, encoding="utf-8").read().split("\n")
in_section = False
section_heading = None
rows = []
for number, line in enumerate(lines, 1):
    if section_heading is None and re.match(r"^##\s*五(?:[、.．:：\s]|$)", line):
        in_section = True
        section_heading = (number, line.strip())
        continue
    if in_section and re.match(r"^##\s", line):
        break
    if in_section and line.startswith("| "):
        rows.append((number, line))

# 表头与分隔行不是矛盾行。
rows = [(number, line) for number, line in rows
        if not line.startswith("|---") and not re.match(r"^\|\s*#\s*\|", line)]
if section_heading is None:
    print("BAD", latest, "认不出第五节的标题（要写成「## 五」后面跟标点、空白或行尾），一行矛盾都没判", sep="\t")

# 「已改」要指得到今天的正文：处置列用「」引的原文，按空白与 ** ` 归一之后，要在今天的 kb 正文
# （「## 历史版本」之前，不含变更史）里找得到，而且要在这一行点名的文件里找得到（D / E 编号、C 号 → checks-owed.md、
# I- 号 → invariants.md、直接写出的 .md 文件名）；一个 kb 文件都没点名的行才在 kb 全部正文里找。
# 不退到全部正文里找：旧条款的原句常被实验页原样引着，改掉之后照样搜得到，会把已经不在的那句当成还在。
# 引号后面紧跟「0 命中」「不在」「没有了」的，说的是那句话已经删掉，不当成指向正文的引文。
QUOTE = re.compile(r"「([^」]+)」")
NEGATED_AFTER = re.compile(r"^\s*(?:字样|都|也|已经)?\s*(?:0\s*命中|不在|没有了)")

def normalized(text):
    return re.sub(r"\s+", " ", re.sub(r"[*`]", "", text)).strip()

REFERENCE = re.compile(r"(?<![A-Za-z0-9_])(?:[DEC]\d+|I-\d+(?:\.\d+)*)(?:（[^）]*）)?|[\w\-]+\.md")

def anchor_text(quote):
    """引文里去掉编号（连同简称括注）与 .md 文件名、标点与空白之后剩下的字：少于两个就指不出正文里的一句话。"""
    return re.sub(r"[\s\W_]+", "", REFERENCE.sub("", normalized(quote)))

kb_body_by_path = {}
for path in sorted(glob.glob(".claude/kb/**/*.md", recursive=True)):
    if path.endswith("-history.md") or "/decisions-history/" in path:
        continue
    kb_body_by_path[path] = normalized(open(path, encoding="utf-8").read().split("\n## 历史版本")[0])

def kb_paths_named_by(text):
    named = set()
    for number in re.findall(r"(?<![A-Za-z0-9_])D(\d+)(?!\d)", text):
        named |= set(glob.glob(f".claude/kb/decisions/{int(number):02d}-*.md"))
    for number in re.findall(r"(?<![A-Za-z0-9_])E(\d+)(?!\d)", text):
        named |= set(glob.glob(f".claude/kb/experiments/{int(number):02d}-*.md"))
    if re.search(r"(?<![A-Za-z0-9_])C\d+(?!\d)", text):
        named.add(".claude/kb/checks-owed.md")
    if re.search(r"(?<![A-Za-z0-9_])I-\d", text):
        named.add(".claude/kb/invariants.md")
    for file_name in re.findall(r"([\w\-]+\.md)\b", text):
        named |= {path for path in kb_body_by_path if os.path.basename(path) == file_name}
    return {path for path in named if path in kb_body_by_path}

checked = 0
fixed_rows, quote_count, unnamed_rows = 0, 0, 0
for number, line in rows:
    cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
    if len(cells) < 4:
        print("BAD", f"{latest}:{number}", "这一行不足四列，读不出处置列", sep="\t")
        continue
    checked += 1
    disposition = cells[3]
    if disposition.startswith("已改"):
        fixed_rows += 1
        affirmed_quotes = [quote.group(1) for quote in QUOTE.finditer(disposition)
                           if not NEGATED_AFTER.match(disposition[quote.end():])]
        if not affirmed_quotes:
            print("BAD", f"{latest}:{number}",
                  f"第 {cells[0]} 行处置列写「已改」，却没有用「」引一句今天正文里的原文（只写「0 命中」「不在了」的那种不算）",
                  sep="\t")
            continue
        named_paths = kb_paths_named_by(" ".join(cells[1:4]))
        if not named_paths:
            unnamed_rows += 1
        searched_paths = named_paths or set(kb_body_by_path)
        for quote in affirmed_quotes:
            quote_count += 1
            wanted = normalized(quote)
            if len(anchor_text(quote)) < 2:
                print("BAD", f"{latest}:{number}",
                      f"第 {cells[0]} 行处置列写「已改」，引的「{quote[:40]}」只是编号或文件名（去掉之后不足两个字），"
                      "在点名的那份文件里总找得到，指不出今天正文里的一句话", sep="\t")
                continue
            if any(wanted in kb_body_by_path[path] for path in searched_paths):
                continue
            elsewhere = sorted(os.path.relpath(path, ".claude/kb") for path, text in kb_body_by_path.items()
                               if path not in searched_paths and wanted in text)
            where_else = f"；只在 {'、'.join(elsewhere[:3])} 里有，那一行没点名它" if elsewhere else ""
            named_shown = "、".join(os.path.relpath(path, ".claude/kb") for path in sorted(named_paths)) or "kb 全部正文"
            print("BAD", f"{latest}:{number}",
                  f"第 {cells[0]} 行处置列写「已改」，引的「{quote[:40]}」在 {named_shown} 的今天正文里找不到{where_else}",
                  sep="\t")
        continue
    cited = set(re.findall(r"\bC\d+\b", disposition))
    if cited & open_owed:
        continue
    if cited & repaid_owed:
        print("BAD", f"{latest}:{number}",
              f"第 {cells[0]} 行处置列只点名了已还清的 {'、'.join(sorted(cited & repaid_owed))}——账还了，这一行的矛盾还在",
              sep="\t")
        continue
    if cited:
        print("BAD", f"{latest}:{number}",
              f"第 {cells[0]} 行处置列点名 {'、'.join(sorted(cited))}，而 checks-owed.md 里没有这个号",
              sep="\t")
        continue
    print("BAD", f"{latest}:{number}",
          f"第 {cells[0]} 行处置列写「{disposition[:40]}」，既不是「已改」、也没点名一个还开着的欠账号",
          sep="\t")

print("COUNT", latest, checked, len(rows), fixed_rows, quote_count, unnamed_rows,
      len(open_owed), len(repaid_owed), section_heading[0] if section_heading else 0,
      section_heading[1] if section_heading else "认不出", sep="\t")
PY
)"; then
    echo "  ✗ 扫描没跑完：内嵌 python 自己出错了，一行都没判"
    printf '%s\n' "$report" | tail -5 | sed 's/^/      /'   # gate-lint:detail
    echo "    → 怎么办：按上面的报错修 .claude/gate.d/doc-registries.sh 里 audit-contradictions 那一格的 python；"
    echo "      这一步没跑完就是什么都没查，不是通过。"
    exit 1
  fi

  if ! grep -q '^COUNT' <<<"$report"; then
    echo "  ✗ 扫描没跑完：内嵌 python 没报出 COUNT 那一行（多半是它自己崩了）"
    echo "    → 怎么办：单独跑一遍这一格（--check audit-contradictions）看 python 的报错；没有 COUNT 就是一行都没判，"
    echo "      而汇总行看着与判过了一模一样。"
    exit 1
  fi

  local -a bad
  local where checked total fixed quotes unnamed open_count repaid_count heading_line heading_text why
  mapfile -t bad < <(grep -E '^BAD' <<<"$report")
  IFS=$'\t' read -r _ where checked total fixed quotes unnamed open_count repaid_count heading_line heading_text < <(grep '^COUNT' <<<"$report")

  if ((${#bad[@]})); then
    echo "  ✗ 最近一次总审核第五节有行没有去向（第五节标题：${heading_text}）："
    while IFS=$'\t' read -r _ where why; do
      printf '      %s：%s\n' "$where" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${bad[@]}")
    echo "    → 怎么办：给这一行一个去向——改了就把处置列写成「已改…」，并用「」引一句今天正文里的原文、"
    echo "      点名它住在哪份文件（D / E 编号、C 号、I- 号或文件名）；引的那句今天找不到，就是那处又被改过，"
    echo "      回去现查这一行的矛盾还在不在：不在了就换成今天的原文，又回来了就当它没改。"
    echo "      还没改就在 .claude/kb/checks-owed.md 开着的那张表里立一笔账、把那个编号写进处置列。"
    echo "      只点名已还清的号不算；挂账表 .claude/audit-rows-pending 已关闭，不许往里加行。"
    exit 1
  fi

  if [[ "$total" == 0 ]]; then
    echo "  ⊘ 本次无对象可判：$where 第五节（第 ${heading_line} 行的标题）下面的表里一行矛盾都没有"
    exit 77
  fi
  echo "  ✓ 最近一次总审核第五节每一行都有去向（$where：第五节 ${total} 行，判了 ${checked} 行；写「已改」的 ${fixed} 行引了 ${quotes} 句原文，都在那一行点名的文件的今天正文里找得到；其中 ${unnamed} 行没点名 kb 文件，按 kb 全部正文找）"
  echo "    欠账表按共用读法 lib-owed.py 切：开着 ${open_count} 笔、已还清 ${repaid_count} 笔；第五节标题在第 ${heading_line} 行"
  exit 0
}

# ── 格 closeout-row27-preconditions：收口表第 27 行那几笔欠账的前置有没有进来（进来了就得补会红的用例）
#
# 里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 27 行点名了一批「今天不可达的欠账」：走不到那一格，
# 就写不出会红的用例。这一格不判那几笔本身，判的是它们各自的**前置**动没动：
# 每一笔登记一条逐字探针（文件、原文、今天的命中次数）。
#   全部对上 ⇒ 前置一个都没进来，今天无对象可判 ⇒ 这一格退 77（汇总往 $GATE_NOT_RUN_FILE 报「本次未跑」，不记通过、不算覆盖）。
#   任一条对不上 ⇒ 那一笔的前置进来了，或者它压着的那段代码改了形态 ⇒ 红，回去重核那一笔。
# 探针盯的是「这一笔的描述今天还成不成立」，所以重构那几处也会红：那时候要做的同样是回去重核，
# 不是把探针的期望值改成新数（改成新数等于把这一笔的描述悄悄换掉）。
# 与 closeout-collects-open 那一格的分工：那一格读收口表本身，判开着的欠账号收全了、行号只许顺序号；
# 这一格不判收口表，读 crates 源码，按写死的探针判第 27 行那几笔的前置动没动。
#
# 成功那句报出查了几条探针、覆盖几笔，以及**没做成探针的是哪几笔**（逐个列名；rules/show-me-test.md：
# 扫到 0 项不是通过，报了「查了多少」还要说「没查的是哪些」）。没做成探针的那张 UNPROBED 表是**手写的**：
# 收口表第 27 行只写「alloc-basis 第三轮转来的四条」、没有逐条列（四条的出处在 alloc-basis 第三轮判决第五节第 4 条），
# 现算不出来。能现算的那一半每轮现算：读收口表第 27 行第二格，按「、」「；」切成几笔，逐笔对探针与 UNPROBED 的笔名，
# 两边都没有的逐个列名；「alloc-basis 第三轮转来的 N 条」这一笔，拿 N 与这里名字以「alloc-basis 第三轮」起头的笔数比。
# 这一段只报不判：对不上的是这张表手写得不全，要人回去补，不是前置进来了。
#
# 样本：checks-owed-red 的 instance_table.rs 里多了一处 `rows.retain(`（C333 那一笔的前置进来了），必须判红并点名那一笔；
# checks-owed-green 的探针逐条对上今天的值，这一格退 77，并报第 27 行现算的笔数与 alloc-basis 那一笔的条数对照。
cell_closeout_row27_preconditions() {
  local PROBES UNPROBED report row27_report probe_count item_count unprobed_count name why line
  local -a bad
  # 四段：笔名、文件、原文（逐字）、今天的命中次数。同一笔可以有几条探针。
  PROBES="$(cat <<'TSV'
豁免集与隔离集按记录起点做 key	crates/singlefs-core/src/mount.rs	let key = (device.0, slot.0);	1
扣住位在空发布循环之后无条件放开	crates/singlefs-core/src/mount.rs	    allocator.release_reclaim_holds();	1
alloc-basis 第三轮 ①：挂载内回收	crates/singlefs-core/src/mount.rs	reclaim_released_up_to(	2
C333（删行那次发布被重放）：行回收	crates/singlefs-core/src/instance_table.rs	rows.retain(	0
C333（删行那次发布被重放）：行回收	crates/singlefs-core/src/instance_table.rs	rows.remove(	0
TSV
)"

  # 做不成逐字探针的那几笔，逐个列名——它们同样开着，只是这一格够不着。
  UNPROBED="$(cat <<'TSV'
alloc-basis 第三轮 ②：根槽写失败推进一格重发	这条路径今天不存在，没有一处逐字文本代表「它进来了」
alloc-basis 第三轮 ③：扣住配今天的 checker 放过「扣住位被撤掉」那份坏镜像	扣住位只住内存、盘上没有落点，没有可扫的对象
TSV
)"

  if ! report="$(PROBES="$PROBES" python3 - 2>&1 <<'PY'
import os

probes = []
for line in os.environ["PROBES"].split("\n"):
    if not line.strip():
        continue
    name, path, original, expected = line.split("\t")
    probes.append((name, path, original, int(expected)))

for name, path, original, expected in probes:
    try:
        source = open(path, encoding="utf-8").read()
    except FileNotFoundError:
        print("BAD", name, f"{path} 不在了（探针指着的文件没了）", sep="\t")
        continue
    hits = source.count(original)
    if hits != expected:
        print("BAD", name, f"{path} 里「{original.strip()}」命中 {hits} 次，登记的今天的值是 {expected} 次", sep="\t")
print("CHECKED", len(probes), len({name for name, _, _, _ in probes}), sep="\t")
PY
)"; then
    echo "  ✗ 探针没跑完：内嵌 python 自己出错了，一条都没核"
    printf '%s\n' "$report" | tail -5 | sed 's/^/      /'   # gate-lint:detail
    echo "    → 怎么办：按上面的报错修 .claude/gate.d/doc-registries.sh 里 closeout-row27-preconditions 那一格的 python；"
    echo "      没跑完不是「本次未跑」，是这一格自己坏了。"
    exit 1
  fi

  if ! grep -q '^CHECKED' <<<"$report"; then
    echo "  ✗ 探针没跑完：内嵌 python 没报出 CHECKED 那一行（多半是它自己崩了）"
    echo "    → 怎么办：单独跑一遍这一格（--check closeout-row27-preconditions）看 python 的报错；没有 CHECKED 就是一条都没核，"
    echo "      而「本次未跑」那句看着与核过了一模一样。"
    exit 1
  fi

  mapfile -t bad < <(grep '^BAD' <<<"$report")
  read -r _ probe_count item_count < <(grep '^CHECKED' <<<"$report")

  if ((${#bad[@]})); then
    echo "  ✗ 收口表第 27 行那几笔欠账的前置动了（或者它压着的那段代码改了形态）："
    while IFS=$'\t' read -r _ name why; do
      printf '      %s：%s\n' "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${bad[@]}")
    echo "    → 怎么办：先回 .claude/kb/milestone/02-second-txn.md 收口表第 27 行重核这一笔今天可不可达。"
    echo "      可达了就当场做成一条会红的用例（改坏一行、看着它红），把这一笔从第 27 行挪出去，探针那一行一起删；"
    echo "      只是重构、这一笔还不可达，就把探针的原文改成今天逐字存在的那一段，命中次数照实写，并在报告里写明重核过。"
    echo "      ⚠️ 别只把期望值改成新数：那等于把这一笔的描述换掉，而外面看不出换过。"
    exit 1
  fi

  unprobed_count="$(grep -c . <<<"$UNPROBED")"
  # 现算的那一半：收口表第 27 行点名的几笔里，探针与 UNPROBED 两边都没有的，逐个列名（只报不判）
  row27_report="$(PROBES="$PROBES" UNPROBED="$UNPROBED" python3 - 2>&1 <<'PY'
import os, re
path = ".claude/kb/milestone/02-second-txn.md"
names = [line.split("\t")[0] for key in ("PROBES", "UNPROBED") for line in os.environ[key].split("\n") if line.strip()]
if not os.path.isfile(path):
    print(f"读不到 {path}，第 27 行现算对照没做")
    raise SystemExit(0)
row = next((line for line in open(path, encoding="utf-8") if re.match(r"^\|\s*27\s*\|", line)), None)
if row is None:
    print(f"{path} 里找不到收口表第 27 行，现算对照没做")
    raise SystemExit(0)
cell = row.strip().strip("|").split("|")[1].strip()
cell = re.sub(r"^[^：]*：", "", cell, count=1)
items, depth, current = [], 0, ""
for character in cell:   # 只在括号外按「、」「；」切：括注里的「步 3、步 4」不是两笔
    depth += character in "（("
    depth -= character in "）)" and depth > 0
    if character in "、；" and depth == 0:
        items.append(current.strip()); current = ""
    else:
        current += character
items = [item for item in items + [current.strip()] if item]
declared_alloc_basis = None
uncovered = []
for item in items:
    counted = re.match(r"^alloc-basis 第三轮转来的([一二三四五六七八九十0-9]+)条$", item)
    if counted:
        digits = "一二三四五六七八九十"
        text = counted.group(1)
        declared_alloc_basis = int(text) if text.isdigit() else digits.index(text) + 1 if len(text) == 1 else None
        continue
    if not any(item.startswith(name.split("：")[0]) or name.startswith(item) for name in names):
        uncovered.append(item)
listed_alloc_basis = len({name.split("：")[0] for name in names if name.startswith("alloc-basis 第三轮")})
print(f"收口表第 27 行现算 {len(items)} 笔；探针与没做成探针的清单两边都没有的 {len(uncovered)} 笔：{'、'.join(uncovered) or '（没有）'}")
if declared_alloc_basis is not None:
    print(f"「alloc-basis 第三轮转来的」那一笔第 27 行写 {declared_alloc_basis} 条，这里探针与清单合计 {listed_alloc_basis} 条"
          + ("" if declared_alloc_basis == listed_alloc_basis else "，对不上：清单是手写的，回去按那一轮判决补齐"))
PY
)"
  echo "  ⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（${probe_count} 条逐字探针、覆盖 ${item_count} 笔，逐条对上今天的值）"
  echo "    没做成探针的 ${unprobed_count} 笔："
  while IFS=$'\t' read -r name why; do
    [[ -z "$name" ]] && continue
    printf '      %s：%s\n' "$name" "$why"
  done <<<"$UNPROBED"
  while IFS= read -r line; do
    [[ -n "$line" ]] && printf '    %s\n' "$line"
  done <<<"$row27_report"
  exit 77
}

# ════════════════ 引用与不变量组的七格（引用与不变量） ════════════════

# ── 1–2 格共用：扫的文件与编号引用的抽法 ─────────────────────────────────
# ⚠️ **必须递归扫**：2026-08-29 决策与实验都拆进了子目录，
# 而引用大半住在那里。用 $KB/*.md 单层通配会静默漏掉它们——
# 实测拆分之后这段检查有一段时间对决策文件里的引用完全失明。
# 扫的文件：kb 下全部 .md（递归）加项目规则。数组装、不靠 $(find …) 的分词，文件名里有空格也不散。
scanned_files_collected=0
collect_scanned_files() {
  ((scanned_files_collected)) && return 0
  scanned_files_collected=1
  kb_md_files=()
  while IFS= read -r -d '' found_file; do kb_md_files+=("$found_file"); done \
    < <(find "$KB" -name '*.md' -print0 2>/dev/null)
  shopt -s nullglob
  rule_md_files=(.claude/rules/*.md)
  shopt -u nullglob
  scanned_md_files=("${kb_md_files[@]}" "${rule_md_files[@]}")
  scanned_note="扫 ${#kb_md_files[@]} 份 kb 文件、${#rule_md_files[@]} 份规则"
}
# 引用抽取用 grep 一次读全部文件；某一份读不了（悬空的符号链接、没有读权限）时 grep 退 2，
# 那一份里的引用就从判据里静默消失，其余照判照绿。所以 grep 的错误不丢进 /dev/null：退 2 判红并列出它的原话。
grep_error_file="$work_directory/grep-errors"
# 用法：extract_numbered_references <字母> <数组名>；抽出全部不同的「字母+1–3 位数字」引用，读文件出错判红
extract_numbered_references() {
  local letter="$1" raw_matches grep_exit_code=0
  local -n extracted_references="$2"
  extracted_references=()
  [[ ${#scanned_md_files[@]} -gt 0 ]] || return 0
  raw_matches="$(grep -ohE "(^|[^A-Za-z0-9/-])${letter}[0-9]{1,3}([^A-Za-z0-9-]|\$)" "${scanned_md_files[@]}" 2>"$grep_error_file")" \
    || grep_exit_code=$?
  if [[ "$grep_exit_code" -ge 2 ]]; then
    bad "抽 ${letter} 号引用时有文件读不了（grep 退 $grep_exit_code），那几份里的引用没进判据："
    sed 's/^/      /' "$grep_error_file"
    howto "按上面 grep 的原话修：悬空的符号链接删掉或指回真文件，没有读权限的补上读权限；" \
          "读不了的文件里的引用一条都没判，不能当成都有定义。"
  fi
  mapfile -t extracted_references < <(printf '%s\n' "$raw_matches" | grep -oE "${letter}[0-9]{1,3}" | sort -u)
}

cell_experiment_refs() {
  cell_failed=0
  collect_scanned_files
  local missing=0 e
  local experiment_refs=()
  # ⚠️ 不能用 \bE[0-9]+\b —— 它会把 URL 里的 E19253-01 当成实验号（实测踩过）。
  extract_numbered_references E experiment_refs
  for e in "${experiment_refs[@]}"; do
    grep -rqE "^## $e " "$KB/experiments" 2>/dev/null || { bad "$e 被引用但 experiments/ 下没有它"
      howto "要么在 experiments/ 下给它建正文（\`## $e <简称>\` 起头），" \
            "要么把引用它的那处改成真实存在的实验号——编号引用悬空，检索到的人会自己补一个。"
      missing=1; }
  done
  if [[ ${#experiment_refs[@]} -eq 0 ]]; then
    bad "一个实验号引用都没扫到（$scanned_note）——这一段没有对象可判"
    howto "确认门禁是在仓库根上跑的（第一个参数是仓库根）、$KB 目录在；" \
          "扫到 0 项不是通过（.claude/singlefs-ai-sop/rules/show-me-test.md「扫到 0 项也不是通过」）。"
  elif [[ $missing -eq 0 ]]; then
    ok "实验号引用全部有定义：${#experiment_refs[@]} 个不同的实验号（$scanned_note）"
  fi
  return "$cell_failed"
}

cell_decision_refs() {
  cell_failed=0
  collect_scanned_files
  local missing=0 d
  local decision_refs=()
  extract_numbered_references D decision_refs
  for d in "${decision_refs[@]}"; do
    grep -rqE "^## $d " "$KB/decisions" 2>/dev/null || { bad "$d 被引用但 decisions/ 下没有它"
      howto "要么在 decisions/ 下给它建正文（\`## $d <简称>\` 起头），" \
            "要么把引用它的那处改成真实存在的决策号。"
      missing=1; }
  done
  if [[ ${#decision_refs[@]} -eq 0 ]]; then
    bad "一个决策号引用都没扫到（$scanned_note）——这一段没有对象可判"
    howto "确认门禁是在仓库根上跑的、$KB 目录在；" \
          "扫到 0 项不是通过（.claude/singlefs-ai-sop/rules/show-me-test.md「扫到 0 项也不是通过」）。"
  elif [[ $missing -eq 0 ]]; then
    ok "决策号引用全部有定义：${#decision_refs[@]} 个不同的决策号（$scanned_note）"
  fi
  return "$cell_failed"
}

# ── 3、5 格共用：不变量表的在用条数 ─────────────────────────────────────
# 表行形如 `| I-1.1 | 简称 | 陈述 | … |`；表里第 2 列是简称（singlefs-ai-sop/rules/kb-discipline.md 第 5 条），陈述在第 3 列。
# 设 inv_actual（总行）、inv_retired（退役）、inv_live（在用）。
count_invariant_rows() {
  inv_actual=$(grep -cE '^\| I-[0-9]+\.[0-9]+ ' "$KB/invariants.md")
  inv_retired=$(grep -cE '^\| I-[0-9]+\.[0-9]+ \| [^|]* \| \*\*此编号不再使用' "$KB/invariants.md")
  inv_live=$(( inv_actual - inv_retired ))
}

cell_declared_counts() {
  cell_failed=0
  if [[ ! -f "$KB/invariants.md" ]]; then
    bad "找不到 $KB/invariants.md，不变量条数无从核对"
    howto "确认门禁是在仓库根上跑的；文件真搬了家的话，按 .claude/rules/path-moves.md 把这里的路径一起改。"
  else
    count_invariant_rows
    # 当前条数的权威登记位是 <!-- invariant-count --> 下一行那句（2026-09-18 立）：历史版本里也有「现共 N 条在用」，
    # 那是当时的数、不跟着改，按文件序取第一处会取到历史里的那一句（实测：2026-09-18 取到 2026-09-14 那条的 66）。
    # 标记丢了、或下一行读不出那句，都判红：退回去取文件里第一处「N 条在用」，取到的正是那句历史。
    local marker_line_number claim_line inv_claim
    marker_line_number=$(grep -n -m1 -F '<!-- invariant-count -->' "$KB/invariants.md" | cut -d: -f1)
    if [[ "$inv_actual" -eq 0 ]]; then
      bad "invariants.md 里一行 \`| I-<章>.<号> \` 表行都没数到——不变量表的写法变了，或这份文件是空的"
      howto "表行形如 \`| I-1.1 | 简称 | 陈述 | … |\`；写法真改了的话，改这里的 count_invariant_rows（invariant-count-elsewhere 那一格用同一份）。"
    elif [[ -z "$marker_line_number" ]]; then
      bad "invariants.md 里没有 <!-- invariant-count --> 标记——当前条数的登记位丢了，读不出正文声称几条（表里在用 $inv_live 条）"
      howto "在「现共 N 条在用」那句的上一行补回 <!-- invariant-count -->；" \
            "不许让检查退回去取文件里第一处「N 条在用」：历史版本里那几句记的是当时的数。"
    else
      claim_line=$(sed -n "$((marker_line_number + 1))p" "$KB/invariants.md")
      inv_claim=$(grep -oE '现共 [0-9]+ 条在用' <<<"$claim_line" | grep -oE '[0-9]+' | head -1)
      if [[ -z "$inv_claim" ]]; then
        bad "invariants.md 第 $((marker_line_number + 1)) 行（<!-- invariant-count --> 的下一行）读不出「现共 N 条在用」：${claim_line:0:60}"
        howto "标记的下一行就写那句「现共 $inv_live 条在用（…）」，中间不空行；" \
              "读不出声明就是没核过，不许当成一致。"
      elif [[ "$inv_claim" != "$inv_live" ]]; then
        bad "invariants.md 正文声称在用 $inv_claim 条，实际 $inv_live 条（总行 $inv_actual，退役 $inv_retired）"
        howto "把正文那句「现共 N 条在用」改成 $inv_live，或者补回漏掉的那几条——" \
              "两个数对不上时，读的人不知道该信哪一个。"
      else
        ok "不变量条数一致：正文声称 $inv_claim 条在用，表里在用 $inv_live 条（总行 $inv_actual，退役 $inv_retired）"
      fi
    fi
  fi
  # 三段各数各的：欠着的那张表、「### 已还清」那张表、「## 历史版本」里的条目。
  # ⚠️ **分界线是「### 已还清」，不是「## 历史版本」**：已还清那张表住在历史版本**之前**，
  # 按历史版本切会把还清的全算进欠账里——2026-09-16 现查它报「欠 326、已还清 0」，
  # 真数是 297 / 29。一条报错数的检查与没有这条检查，在门禁输出里长得一模一样。
  # 数不出来（文件不在、awk 出错、两张表一行都没数到）判红：两个空白或两个 0 印在成功行里，看着就像数过了。
  if [[ ! -f "$KB/checks-owed.md" ]]; then
    bad "找不到 $KB/checks-owed.md，欠账条数取不到"
    howto "确认门禁是在仓库根上跑的；文件真搬了家的话，按 .claude/rules/path-moves.md 把这里的路径一起改。"
  else
    # 开着与已还清的切法用共用读法 lib-owed.py（doc-registries、checker-independence-and-sync、code-source-discipline 同一份），不在这里再抄一份 awk
    local chk_counts="" chk_actual chk_done chk_heading
    if chk_counts=$(python3 - "$GATE_DIRECTORY/lib-owed.py" "$KB/checks-owed.md" <<'PY_OWED'
import importlib.util, sys
spec = importlib.util.spec_from_file_location("lib_owed", sys.argv[1])
lib_owed = importlib.util.module_from_spec(spec); spec.loader.exec_module(lib_owed)
table = lib_owed.read_owed_table(sys.argv[2])
print(len(table.open_names), len(table.paid_names), int(table.paid_heading_found))
PY_OWED
    ); then :; else chk_counts=""; fi
    read -r chk_actual chk_done chk_heading <<<"$chk_counts"
    if [[ -z "${chk_actual:-}" || -z "${chk_done:-}" ]]; then
      bad "共用读法 lib-owed.py 没数出 checks-owed.md 的条数（输出「$chk_counts」）"
      howto "单独跑一遍上面那段 python 看它报什么错；数不出来就是没核过，不许当成数过了。"
    elif [[ $((chk_actual + chk_done)) -eq 0 ]]; then
      bad "checks-owed.md 里一行 \`| C<n> \` 都没数到（欠着 0、已还清 0）——欠账表的写法变了，或这份文件是空的"
      howto "欠账行形如 \`| C12 | 简称 | … |\`；写法真改了的话，改 .claude/gate.d/lib-owed.py 的正则，doc-registries、checker-independence-and-sync、code-source-discipline 与这里一起跟上。"
    elif [[ "${chk_heading:-0}" != 1 ]]; then
      bad "checks-owed.md 里认不出「已还清」标题——开着的与已还清的分不开，$chk_actual 条全被算成欠着"
      howto "已还清那张表上面要有一行「### 已还清」；标题改过名的话，改 .claude/gate.d/lib-owed.py 认标题的那条正则。"
    else
      ok "欠检查 $chk_actual 条、已还清 $chk_done 条（checks-owed.md）"
    fi
  fi
  return "$cell_failed"
}

cell_governance_refs() {
  cell_failed=0
  local governance_rc=0
  python3 "$GATE_DIRECTORY/lib-governance-refs.py" || governance_rc=$?
  if [[ $governance_rc -eq 0 ]]; then
    ok "治理文档与 kb 里的门禁号、门禁全名与格名、路径与「小节」都指得到，没有用旧编号称呼门禁"
  elif [[ $governance_rc -eq 3 ]]; then
    bad "一份治理文档都没扫到——这一段没有对象可判"
    howto "确认门禁是在仓库根上跑的；治理文档搬了家的话，改 lib-governance-refs.py 的 CARRIER_PATTERNS。"
  elif [[ $governance_rc -ne 1 ]]; then
    bad "lib-governance-refs.py 没跑成（退出码 $governance_rc）——这一段没判"
    howto "单独跑 python3 $GATE_DIRECTORY/lib-governance-refs.py 看它报什么错；没跑成就是没判，不许当成判过了。"
  else
    bad "治理文档与 kb 里有指不到的指向（上面逐条列出）"
    howto "旧编号与全名：门禁一律按门禁目录顶层的文件名去掉 .sh 称呼，旧编号照红行给的今天的名字改，格名用那道门禁的 --list 看；" \
          "门禁号：阶段被删或收归上游的，改成共享 gate.sh 里那一道的名字（「链接指向」这类）或现存的号；" \
          "路径：搬了家的改成新路径，已删的改指现存的做法，已归档的写裸文件名并指到取法（.claude/agent-common.md「找不到历史实验的数据」那一条）；" \
          "小节：按那份文件今天的标题改。改 agent 定义与共用约束照走门禁 doc-process-records 的 agent-def-adversarial-review 格那一条。"
  fi
  return "$cell_failed"
}

# 同一个量被别的文件抄过去之后，declared-counts 那一格只看 invariants.md 自己，就没有任何东西在对了。
# ⚠️ **这条是实测出来的**（2026-09-07）：`decisions/13-验证路线.md` 的「冲突 1」
# 写着「26 条不变量若既是 checker 源码又是形式验证的 spec，就成了单点」，
# 而 `invariants.md` 当时已是 63 条在用 —— 那个 26 是某轮的旧数，从写下起就没人回来改过。
# 坑在**跨文件的口径层**，不在某一段代码里（`.claude/singlefs-ai-sop/rules/show-me-test.md`「先问这个坑在哪一层」）。
# 查的是：`invariants.md` 之外的 kb 文件，正文里写「N 条在用」或「N 条不变量在用」的地方，N 必须等于实际在用条数。
# 一处这样的声明都没有：这一格本次无对象可判（77，不记通过）。
# ⚠️ **抓不到的那一半**：
#   1. 不带「在用」二字的写法（「66 条不变量」）抓不到 —— 那种句子多半在复述某一轮普查
#      当时的口径，是历史事实，照现值判会误判。要它被管住，就把话写成「N 条在用」。
#   2. `## 历史版本` 之后的内容整段跳过（旧数留在那里是 `kb-discipline.md` 第 8 条要求的）。
#   3. `*-history.md` 与 `decisions-history/` 下整份跳过 —— 那些文件通篇是历史。
cell_invariant_count_elsewhere() {
  cell_failed=0
  [[ -f "$KB/invariants.md" ]] || { echo "  ! 找不到 $KB/invariants.md，这一格本次无对象可判"; return 77; }
  count_invariant_rows
  local elsewhere_rc=0
  python3 - "$KB" "$inv_live" <<'PY' || elsewhere_rc=$?
import re, sys, os, glob

kb = sys.argv[1]
live = int(sys.argv[2])
inv = os.path.abspath(os.path.join(kb, "invariants.md"))

scanned, hits, bad = 0, [], []
for p in sorted(glob.glob(os.path.join(kb, "**", "*.md"), recursive=True)):
    if os.path.abspath(p) == inv or os.path.basename(p).endswith("-history.md") or "/decisions-history/" in p:
        continue
    scanned += 1
    body = open(p, encoding="utf-8").read().split("\n## 历史版本")[0]
    # 「条」与「在用」之间许夹「不变量」三个字（「现有 26 条不变量在用」）；别的名词不认，免得把别的量读成不变量条数
    for m in re.finditer(r"(\d+)\s*条(?:不变量)?\s*在用", body):
        n = int(m.group(1))
        hits.append((p, n))
        if n != live:
            line = body[:m.start()].count("\n") + 1
            bad.append(f"{p}:{line} 写「{m.group(0)}」，而 invariants.md 实际在用 {live} 条")

if bad:
    print(f"  ✗ 不变量条数跨文件对不上 {len(bad)} 处：")      # gate-lint:summary
    for b in bad:
        print("      " + b)                                   # gate-lint:detail
    print(f"  → 怎么办：以 .claude/kb/invariants.md 为准，把那句里的数改成 {live}；")
    print("    若那句说的其实是某一轮普查当时的口径（不是现值），就把「在用」二字去掉并写明是哪一轮，")
    print("    否则下一个检索到它的人会拿一个旧数当现状。")
    sys.exit(1)

if not hits:
    print(f"  ! 本次无对象可判：扫了 {scanned} 份 kb 文件（invariants.md 与变更史之外），没有一处写「N 条在用」的声明")
    sys.exit(77)
print(f"  ✓ 不变量条数跨文件一致：实际在用 {live} 条，扫了 {scanned} 份 kb 文件、命中 {len(hits)} 处声明")
PY
  if [[ $elsewhere_rc -ne 0 && $elsewhere_rc -ne 1 && $elsewhere_rc -ne 77 ]]; then
    bad "数「N 条在用」的那段 python 没跑成（退出码 $elsewhere_rc）——这一格没判"
    howto "按上面的 traceback 修；没跑成就是没判，不许当成判过了。"
    return 1
  fi
  return "$elsewhere_rc"
}

# 还 checks-owed.md C38（外部文献不可复核）的源码那一半。
# **它要拦的是证据蒸发，不是 kb 写错。** 2026-08-29 的 find 发现一批标着
# 「本机 PDF 逐字核实」的文献已经不在本机了，而引用它们的结论仍标着「已核实」——
# 那批蒸发得**无声无息**。源码这一侧会以同样的方式蒸发：路径变了、版本升了、树被删了。
# ⚠️ **源码树不在也判红，不许当跳过**——「跳过」正是让上一批文献无声蒸发的那个行为。
# 红了不等于 kb 写错，处置见 verify-citations.sh 自己打印的下一步。
# 真 verify-citations.sh 的判别力不在这里测：它读两个本机绝对路径下的源码树（FS_REFS、KERNEL_TREE 的默认值），装不进密封的样本目录；
# 它自己的 --selftest 合成两棵假树测，由 code-tooling 跑。它的判别力双向证过（2026-08-29）：
#   FS_REFS=/nonexistent ⇒ 固定点三棵树（refs-linux、refs-zfs、refs-docs）上的断言成批未命中、rc=1
#   （条数随断言表增减，这里不写死：现跑一次看末行「N 条未命中」）；
#   把 spa.h 的 SPA_BLKPTRSHIFT 从 7 改成 9 ⇒ 该条未命中、rc=1。
CITATIONS_SCRIPT=research/scripts/verify-citations.sh
citations_pid=""
start_citations() {
  [[ -f "$CITATIONS_SCRIPT" ]] || return 0
  bash "$CITATIONS_SCRIPT" > "$work_directory/citations.out" 2>&1 &
  citations_pid=$!
}
cell_citations() {
  cell_failed=0
  if [[ ! -f "$CITATIONS_SCRIPT" ]]; then
    echo "  ✗ 找不到 $CITATIONS_SCRIPT"
    echo "     → 怎么办：这一格靠它逐条复核引文。文件被挪走就改这里的路径，"
    echo "               还没写就先写它——缺了它，引文一条都没被验过。"
    return 1
  fi
  start_citations
  local citations_rc=0
  wait "$citations_pid" || citations_rc=$?
  cat "$work_directory/citations.out"
  if [[ $citations_rc -ne 0 ]]; then
    bad "外部引用复核没过：$CITATIONS_SCRIPT 退 $citations_rc"
    howto "按上面 verify-citations.sh 自己打印的下一步处置；未命中不等于 kb 写错，也可能是源码固定点没了或版本变了。"
  fi
  return "$cell_failed"
}

# 判据：这次改动在 `.claude/kb/invariants.md` 里新增或改写的每一行不变量（`| I-x.y | …`），
# 正文里必须至少点名一条 `D<n>（简称） 已定项 k`——那是这条不变量要判的字段的落点。
# 点不出来判红。已停用的行（正文写着「此编号不再使用」）不判。
# 为什么：I-1.2（块头写序已发布） 与 I-1.4（块头 fsid 一致） 写下来的时候，它们点名的 generation 与 fsid
# 在当时的字段表里根本不存在——两条已定不变量在第一版直接落空，而没有任何东西说话
# （C69（已定不变量没有字段可判））。一条判不了的不变量比没有这条更糟：checker 会老老实实跑它、
# 老老实实报绿（`show-me-test.md`「不变量本身就是错的，checker 会老老实实检查一条错规矩，而且全绿」）。
# ⚠️ 射程只到**这次改动的行**，与门禁 doc-experiments 的 results-cited、quoted-result-lines 两格同一条：存量里点不出分项的行今天有几十条，
# 一次全判会把每一次提交都挡住，而 `sop-first.md` 要的是抬地板、不是筑墙。存量那笔账仍然欠着
# （C69（已定不变量没有字段可判）），所以成功那句**每轮都把存量的条数现算着报出来**，不让它沉默。
# ⚠️ 判不了的：点名的那条分项里到底有没有这个字段、宽度对不对——那要人看。这一格只判「点没点名」。
# 改动范围：GATE_BASE 给了就与它比，否则与 @{upstream} 的 merge-base 比，都没有就与 HEAD 比；
# 工作区、暂存区与未跟踪文件都算（基准用共用库 research/scripts/changed-paths.sh 的 gate 取法，与 doc-process-records、doc-experiments 的 evidence-in-repo 格同一份代码）。
# 这次改动里 invariants.md 是新增的（相对基准新增、暂存新增或未跟踪），整份的每一行都算新写；
# 否则取「基准到工作区」与「HEAD 到暂存区」两份 diff 的 + 行，同一行两份都有只算一次；两份 diff 任一份 git 失败判红，
# 不当成「一行没改」。这次改动没有新写或改写任何不变量行，这一格本次无对象可判（77，存量条数照样报出来）。
cell_invariant_anchors() {
  cell_failed=0
  git rev-parse --is-inside-work-tree >/dev/null 2>&1 || { echo "  ! $PWD 不是 git 仓，这一格本次无对象可判"; return 77; }
  [[ -f "$KB/invariants.md" ]] || { echo "  ! 没有 $KB/invariants.md，这一格本次无对象可判"; return 77; }
  local LIB_CHANGED_PATHS base added added_path file_is_new=0 anchors_rc=0
  LIB_CHANGED_PATHS="$(cd "$GATE_DIRECTORY/../.." && pwd)/research/scripts/changed-paths.sh"
  # shellcheck source=../../research/scripts/changed-paths.sh
  source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用库 $LIB_CHANGED_PATHS"; echo "     → 怎么办：改动范围的取法只有那一份，恢复它，别在阶段里再抄一份。"; return 1; }
  base="$(gate_diff_base gate)"
  added="$(gate_changed_paths "$base" untracked A)" || {
    echo "  ✗ 取不到这次改动碰了哪些路径（基准 $base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到改动范围时这一格什么都没比，不是通过。"
    return 1
  }
  while IFS= read -r added_path; do
    if [[ "$added_path" == .claude/kb/invariants.md ]]; then file_is_new=1; fi
  done <<<"$added"
  python3 - "$base" "$file_is_new" <<'PY' || anchors_rc=$?
import re, subprocess, sys

base = sys.argv[1]
file_is_new = sys.argv[2] == "1"
path = ".claude/kb/invariants.md"
ROW = re.compile(r"^\|\s*(I-[\d.]+)\s*\|")
ITEM = re.compile(r"D\d+（[^）]+）\s*已定项\s*\d+")
RETIRED = "此编号不再使用"

def anchored(line):
    return ITEM.search(line) is not None

# 这次改动新增或改写的行：diff 的 `+` 侧。基准那一版没有这个文件时（新建，含未跟踪），整份都算新写。
# 两份 diff 会重叠（改动已暂存、工作区没再动时两份都有同一行），同一行只算一次，别让报出来的行数翻倍。
touched = []
if file_is_new:
    candidate_lines = open(path, encoding="utf-8").read().split("\n")
else:
    git_diff = ["git", "-c", "core.quotepath=false", "diff", "-U0"]
    diffs = []
    for arguments in ([base, "--", path], ["--cached", "--", path]):
        shown = subprocess.run([*git_diff, *arguments], capture_output=True, text=True)
        if shown.returncode != 0:
            # git 失败时 + 行是空的：当成「这次没改不变量」就会走到无对象可判，把新写的行整批放过去
            print(f"  ✗ 取不到 {path} 这次改动的 diff（git diff {' '.join(arguments[:-2]) or '基准'} 退 {shown.returncode}：{shown.stderr.strip()[:120]}）")
            print("     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、对象库与索引没坏）再跑；取不到 diff 时这一格什么都没比，不是通过。")
            sys.exit(1)
        diffs.append(shown.stdout)
    diff, staged = diffs
    candidate_lines = [line[1:] for chunk in (diff, staged) for line in chunk.split("\n")
                       if line.startswith("+") and not line.startswith("+++")]
for body in candidate_lines:
    if ROW.match(body) and RETIRED not in body and body not in touched:
        touched.append(body)

missing = [body for body in touched if not anchored(body)]

# 存量：整份文件里点不出分项的行，现算，每轮报出来
total, unanchored_all = 0, 0
for line in open(path, encoding="utf-8"):
    if ROW.match(line) and RETIRED not in line:
        total += 1
        if not anchored(line):
            unanchored_all += 1

if missing:
    print(f"  ✗ 这次改动新写或改写的 {len(missing)} 行不变量，没有点名它判的字段住在哪条已定分项（基准 {base}）：")  # gate-lint:summary
    for body in missing:
        identifier = ROW.match(body).group(1)
        print(f"      {identifier}：{body.strip()[:120]}")  # gate-lint:detail
    print("     → 怎么办：在那一行正文里写清这条不变量判的字段住哪，形态是「D<编号>（简称） 已定项 k」。")
    print("               写不出来，说明这条不变量今天判不了任何字节——那正是 C69（已定不变量没有字段可判） 记着的那一类：")
    print("               checker 会老老实实跑它、老老实实报绿。先补字段（在那条决策里定出落点与宽度），或者把这条不变量降级。")
    sys.exit(1)

if not touched:
    print(f"  ⊘ 本次无对象可判：这次改动没有新写或改写不变量行（基准 {base}）；存量 {total} 行里 {unanchored_all} 行还点不出已定分项，那笔账记在 C69（已定不变量没有字段可判）")
    sys.exit(77)
print(f"  ✓ 这次改动新写或改写的 {len(touched)} 行不变量都点名了已定分项（基准 {base}）；"
      f"存量 {total} 行里 {unanchored_all} 行还点不出，那笔账记在 C69（已定不变量没有字段可判）")
PY
  if [[ $anchors_rc -ne 0 && $anchors_rc -ne 1 && $anchors_rc -ne 77 ]]; then
    bad "判不变量字段落点的那段 python 没跑成（退出码 $anchors_rc）——这一格没判"
    howto "按上面的 traceback 修；没跑成就是没判，不许当成判过了。"
    return 1
  fi
  return "$anchors_rc"
}

# ════ 登记的格函数：弄坏开关点名这一格就不跑、按判过了记，否则调这一格的判法 ════
# 字段与布局组九格原是在后台起的（stdin 是 /dev/null），这里照样把 stdin 接到 /dev/null。
break_skips() {
  if [[ "$BREAK_SWITCH" == "$1" ]]; then
    echo "  ✓ 弄坏开关 DOC_REGISTRIES_BREAK=$1：这一格没判"
    exit 0
  fi
}
judge_field_refs()                   { break_skips field-refs;                   cell_field_refs </dev/null; }
judge_field_projection()             { break_skips field-projection;             cell_field_projection </dev/null; }
judge_field_table_sums()             { break_skips field-table-sums;             cell_field_table_sums </dev/null; }
judge_first_txn_hooks()              { break_skips first-txn-hooks;              cell_first_txn_hooks </dev/null; }
judge_admission_terms()              { break_skips admission-terms;              cell_admission_terms </dev/null; }
judge_segment_registry()             { break_skips segment-registry;             cell_segment_registry </dev/null; }
judge_format_const_placeholders()    { break_skips format-const-placeholders;    cell_format_const_placeholders </dev/null; }
judge_second_txn_hooks()             { break_skips second-txn-hooks;             cell_second_txn_hooks </dev/null; }
judge_tree_table_reserve()           { break_skips tree-table-reserve;           cell_tree_table_reserve </dev/null; }
judge_table_shape()                  { break_skips table-shape;                  cell_table_shape; }
judge_closeout_collects_open()       { break_skips closeout-collects-open;       cell_closeout_collects_open; }
judge_paid_cited_tests()             { break_skips paid-cited-tests;             cell_paid_cited_tests; }
judge_audit_contradictions()         { break_skips audit-contradictions;         cell_audit_contradictions; }
judge_closeout_row27_preconditions() { break_skips closeout-row27-preconditions; cell_closeout_row27_preconditions; }
judge_experiment_refs()              { break_skips experiment-refs;              cell_experiment_refs; }
judge_decision_refs()                { break_skips decision-refs;                cell_decision_refs; }
judge_declared_counts()              { break_skips declared-counts;              cell_declared_counts; }
judge_governance_refs()              { break_skips governance-refs;              cell_governance_refs; }
judge_invariant_count_elsewhere()    { break_skips invariant-count-elsewhere;    cell_invariant_count_elsewhere; }
judge_citations()                    { break_skips citations;                    cell_citations; }
judge_invariant_anchors()            { break_skips invariant-anchors;            cell_invariant_anchors; }

stage_cells_run "$ROOT"
