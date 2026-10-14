#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: 决策文档与变更史（十八格：决策与分项的状态、索引与登记表对正文、未定项回看、写频率、候选代号复用、变更史条目与形状）
# gate-fixture: yes
# gate-category: 文档类
# gate-similar: history-ordinal.sh 它也读变更史的条目标题、也只判这一轮新增的，但判的是「（其 N）」撞号，住在上游副本里由上游门禁管，项目本地改不了它
# gate-similar: doc-registries.sh 它从欠账表、收口表与总审核出发判表本身的形状与账的去向，判字段与布局登记表（字节表「指向」列点名的分项存不存在、字段投影与加和、认购表求和不超预留），它的 decision-refs 那一格判 kb 全文引用的 D 号有定义、不看分项号与状态；这里的 user-verdict-owed 那一格方向相反，从决策正文里的「待用户复核」出发去欠账表找一笔未还的账，item-ref-status 那一格判引用写的已定 / 未定与正文分项表相符，判的对象都是决策正文、索引页、冻结层归属登记表与变更史
# gate-similar: code-source-discipline.sh 它的 clause-enums 格也读 .claude/gate.d/ 下一张登记表、去决策分项里找逐字的串，但比对的另一边是代码里的枚举；这里格「同一结构的写频率只有一个量级」两边都在决策正文里（句子写的频率与登记的量级），不碰代码
#
# kb 里同一件事只许一种写法：决策与分项的状态在哪一处说都一样，索引页与决策相关的登记表与正文对得上；决策正文改了要在变更史留条目，变更史的形状要对。
# 格名表（--list 打它；各格的中文名写在格函数上方的「格「…」」那一行）：
# gate-cell: kb-shape kb 形状：分项只许一种叫法、kb 文件不链回自己、规则路径带 .claude/、决策标题与分项两节、索引页分项计数与进度计数与正文相符
# gate-cell: item-ref-status 每一处「D<n>（简称） 已定项 k / 未定项 k」写的状态与正文分项表相符
# gate-cell: status-redundancy 状态别说两遍：小节标题不带「—— 已定」、条目不同时写破折号与「状态：」、分项索引行挂在对的节
# gate-cell: cross-decision-status 紧贴「D<n>（简称）」写「未定」，而那条决策的状态行不是待定
# gate-cell: settled-item-self-open 已定项区段里说它自己、或同一条决策里另一条已定分项还没定
# gate-cell: settled-ref-says-open 引用写着「已定项 k」、归属到的那一条也已定，紧跟着却说它没定
# gate-cell: stale-open-items 未定项点名的决策，状态行在这条未定项最近一次复核之后又变过
# gate-cell: settled-same-file 这次在一份决策里新增了「已定」小节（含「已定项 N：…」分项定案节），同文件开着的未定项与索引页的待议节一个字没动
# gate-cell: freeze-layer-membership 冻结层归属登记表：每层每个结构都有行、写「退出」的必是派生态、依据指得到已定分项
# gate-cell: decision-items-sync decisions.md 的分项清单生成块与索引表「状态」「进度」两列，与生成器的输出逐字相同
# gate-cell: blocking-verdict 每条未定项都写了「改新池新建文件的字节」与「动不动格式」两把尺的判定
# gate-cell: user-verdict-owed 决策正文里标着「待用户复核」的，checks-owed.md 里要有一笔未还的账点名那条决策
# gate-cell: decision-summary-width 决策索引表每一行的「结论」列不超过 decisions.md 写的字数上限
# gate-cell: write-frequency 决策正文里落在登记结构上的频率断言，不与 write-frequency.tsv 登记的量级说反话
# gate-cell: entry-added 决策正文改了，变更史里要有新增条目
# gate-cell: shape 变更史的形状：日期块住在自己的节、同节同一天只一块、块下是 - 变更项而不是 #### 子条目、不留快查与现状行
# gate-cell: history-loss 两份变更史与基准相比，基准里每个日期块的数、编号、产物名、引名在改后都找得到（提交前整理不许丢）
# gate-cell: candidate-codes 同一份决策里，同一个候选代号（甲乙丙丁… 或单个大写字母）不许出现在两个分项的候选表里
#
# 用法：
#   bash .claude/gate.d/doc-decisions.sh [项目根]                                       十八格都跑；项目根不给取本脚本往上两级
#   bash .claude/gate.d/doc-decisions.sh --list                                         逐行打格名与判什么，不跑格
#   bash .claude/gate.d/doc-decisions.sh --check <格名>[,<格名>…] [项目根]              只跑点名的格，按格名表的次序
#   bash .claude/gate.d/doc-decisions.sh --write [--check decision-items-sync] [项目根]  重写 decisions.md 的分项清单生成块与索引表「状态」「进度」两列，写完回读判一遍
# 退出码：0 绿；1 有格红（红格逐格接它登记的出路）；77 跑到的格全本次未跑（逐格报进 $GATE_NOT_RUN_FILE）；
#   2 参数错：认不出的选项、两个项目根、进不去项目根、格名写错、--write 配了 decision-items-sync 之外的格。
# --list、--check、样本根的 .gate-cells、每格一个子 shell 与逐格汇总都在共用库 lib/stage-cells.sh（写法在它的文件头）。
#
# 样本 fixtures/doc-decisions.sh/，每份样本根的 .gate-cells 点名只跑哪几格：
#   <格名>-red（前十四格各一份，kb-shape-red … write-frequency-red）  那一格判红、汇总有「<格名>：红（退出 1）」；各份犯了什么写在那一格函数上方
#   decision-documents-green                前十四格的绿样本并在一棵树里（setup.sh 造 git 历史与工作区改动）：十三格绿，write-frequency 没有登记表退 77，整道退 0
#   write-frequency-green                   write-frequency 的绿样本（不是 git 仓），那一格绿；判绿靠的几句写在那一格函数上方
#   candidate-codes-red / -green            candidate-codes 的样本（不是 git 仓）：红的那份两个分项的候选表都用丙（一张表格、一张列表）；
#                                           绿的那份两个分项代号不同，另一处写「已定项 1 的丙」，历史节里复用的代号不扫
#   decision-documents-some-cells-not-run   不是 git 仓，没有分项引用、D15、「待用户复核」与写频率登记表：七格退 77 逐格报本次未跑，整道退 0
#   decision-history-red                    决策首行状态翻了、已提交没推出去、变更史没新增条目，新 kb 页里日期与序号粘在一起：entry-added、shape 都红
#   shape-cells-marker-red                  现场同 decision-history-red，.gate-cells 只点名 shape：汇总只列一格
#   decision-history-green                  决策正文改了 7 行，D1 节新增一个带一条顶格 `- ` 项的日期块：entry-added、shape 都绿
#   shape-red                               shape 的 a 到 e 五样都有新增的，基准里另有存量只计数：shape 红
#   shape-green                             新增的日期块都住对节、同节日期不重复、围栏里的旧形态不算，存量只计数：shape 绿、entry-added 本次未跑
#   settled-same-file-item-heading-red / -green  settled-same-file 的分项定案标题样本：本次提交在未定项一节之前新增 `#### 已定项 2：丙问题`；
#                                           红的那份未定项一字未动、索引页待议节补了复核注，红在那条未定项上；绿的那份未定项与待议节都补了复核注，绿
#   history-loss-red / history-loss-green   基准里一块带一个数：整理时弄丢了判红；整理成一条结论、数都在判绿
# 弄坏开关（样本自检时设进环境）。逐项写开关名、弄坏哪一处，下一行起写打开后判错的样本（fixtures/doc-decisions.sh/ 下的目录名，
# 括号里是 stage-selftest.sh 报那份样本判错的第一行的句首）：
#   STAGE_CELLS_BREAK=red-swallowed  归共用库：红格按绿记、整道退 0
#       blocking-verdict-red、candidate-codes-red、cross-decision-status-red、decision-history-red、decision-items-sync-red、decision-summary-width-red、
#       freeze-layer-membership-red、history-loss-red、item-ref-status-red、kb-shape-red、settled-item-self-open-red、settled-ref-says-open-red、
#       settled-same-file-item-heading-red、settled-same-file-red、shape-cells-marker-red、shape-red、stale-open-items-red、status-redundancy-red、user-verdict-owed-red、
#       write-frequency-red（期望退出 1，实测 0）
#   STAGE_CELLS_BREAK=marker-ignored  归共用库：不认样本根的 .gate-cells，每份样本把登记的格都跑
#       blocking-verdict-red、candidate-codes-red、cross-decision-status-red、decision-items-sync-red、decision-summary-width-red、freeze-layer-membership-red、
#       history-loss-red、item-ref-status-red、kb-shape-red、settled-item-self-open-red、settled-ref-says-open-red、settled-same-file-item-heading-red、settled-same-file-red、shape-red、
#       stale-open-items-red、status-redundancy-red、user-verdict-owed-red、write-frequency-red（输出里找不到「doc-decisions.sh：1 格里 1 格红…」）；
#       candidate-codes-green、decision-documents-green、decision-history-green、history-loss-green、settled-same-file-item-heading-green、shape-green（期望退出 0，实测 1）；
#       decision-documents-some-cells-not-run（输出里找不到「doc-decisions.sh：14 格里 7 格绿…」）；decision-history-red（输出里找不到「doc-decisions.sh：2 格里 2 格红…」）；
#       shape-cells-marker-red（输出里找不到「样本只跑 .gate-cells 点名的格：shape」）；write-frequency-green（输出里找不到「doc-decisions.sh：1 格里 1 格绿…」）
#   STAGE_CELLS_BREAK=check-ignored  归共用库：点名选格不起作用（--check 与样本根的 .gate-cells 共用一段），照跑全部登记的格
#       blocking-verdict-red、candidate-codes-red、cross-decision-status-red、decision-items-sync-red、decision-summary-width-red、freeze-layer-membership-red、
#       history-loss-red、item-ref-status-red、kb-shape-red、settled-item-self-open-red、settled-ref-says-open-red、settled-same-file-item-heading-red、settled-same-file-red、shape-red、
#       stale-open-items-red、status-redundancy-red、user-verdict-owed-red、write-frequency-red（输出里找不到「doc-decisions.sh：1 格里 1 格红…」）；
#       candidate-codes-green、decision-documents-green、decision-history-green、history-loss-green、settled-same-file-item-heading-green、shape-green（期望退出 0，实测 1）；
#       decision-documents-some-cells-not-run（输出里找不到「doc-decisions.sh：14 格里 7 格绿…」）；decision-history-red（输出里找不到「doc-decisions.sh：2 格里 2 格红…」）；
#       shape-cells-marker-red（输出里找不到「样本只跑 .gate-cells 点名的格：shape」）；write-frequency-green（输出里找不到「doc-decisions.sh：1 格里 1 格绿…」）
#   DOC_DECISIONS_BREAK=kb-shape  kb-shape 格不跑、按判过了记
#       decision-documents-green（输出里找不到「决策标题与未定项列表相符，且两节没有互相串味…」）；kb-shape-red（期望退出 1，实测 0）
#   DOC_DECISIONS_BREAK=item-ref-status  item-ref-status 格不跑、按判过了记
#       decision-documents-green（输出里找不到「分项引用与正文状态一致…」）；decision-documents-some-cells-not-run（输出里找不到「  item-ref-status：本次未跑」）；
#       item-ref-status-red（期望退出 1，实测 0）
#   DOC_DECISIONS_BREAK=status-redundancy  status-redundancy 格不跑、按判过了记
#       decision-documents-green（输出里找不到「状态只说一遍，且分项都在对的节里…」）；status-redundancy-red（期望退出 1，实测 0）
#   DOC_DECISIONS_BREAK=cross-decision-status  cross-decision-status 格不跑、按判过了记
#       cross-decision-status-red（期望退出 1，实测 0）；decision-documents-green（输出里找不到「没有把已定的决策说成未定…」）
#   DOC_DECISIONS_BREAK=settled-item-self-open  settled-item-self-open 格不跑、按判过了记
#       decision-documents-green（输出里找不到「已定项的正文没有把已经定了的东西说成未定…」）；settled-item-self-open-red（期望退出 1，实测 0）
#   DOC_DECISIONS_BREAK=settled-ref-says-open  settled-ref-says-open 格不跑、按判过了记
#       decision-documents-green（输出里找不到「扫了 29 个文件、43 处已定项引用，没有一处紧跟着说它没…」）；decision-documents-some-cells-not-run（输出里找不到「  settled-ref-says-open：本次未跑」）；
#       settled-ref-says-open-red（期望退出 1，实测 0）
#   DOC_DECISIONS_BREAK=stale-open-items  stale-open-items 格不跑、按判过了记
#       decision-documents-green（输出里找不到「没有未定项被别处的更新甩在后面…」）；decision-documents-some-cells-not-run（输出里找不到「  stale-open-items：本次未跑」）；
#       stale-open-items-red（期望退出 1，实测 0）
#   DOC_DECISIONS_BREAK=settled-same-file  settled-same-file 格不跑、按判过了记
#       decision-documents-green（输出里找不到「同文件的未定项与索引页的待议节都被回头看过…」）；decision-documents-some-cells-not-run（输出里找不到「  settled-same-file：本次未跑」）；
#       settled-same-file-item-heading-green（输出里找不到「本次定案之后，同文件的未定项与索引页的待议节都被回头看过…」）；
#       settled-same-file-item-heading-red、settled-same-file-red（期望退出 1，实测 0）
#   DOC_DECISIONS_BREAK=settled-item-heading-unrecognised  settled-same-file 退回老的小节标题正则，不认 `#### 已定项 N：…` 与 `#### 已定项 N（…）`
#       settled-same-file-item-heading-red（期望退出 1，实测 77：本次 diff 没有新增「已定」小节）；settled-same-file-item-heading-green（期望退出 0，实测 77）
#   DOC_DECISIONS_BREAK=freeze-layer-membership  freeze-layer-membership 格不跑、按判过了记
#       decision-documents-green（输出里找不到「冻结层归属登记表判过了…」）；decision-documents-some-cells-not-run（输出里找不到「  freeze-layer-membership：本次未跑」）；
#       freeze-layer-membership-red（期望退出 1，实测 0）
#   DOC_DECISIONS_BREAK=decision-items-sync  decision-items-sync 格不跑、按判过了记
#       decision-documents-green（输出里找不到「决策索引表状态列与进度列与正文同步（21 条决策）」）；decision-items-sync-red（期望退出 1，实测 0）
#   DOC_DECISIONS_BREAK=blocking-verdict  blocking-verdict 格不跑、按判过了记
#       blocking-verdict-red（期望退出 1，实测 0）；decision-documents-green（输出里找不到「13 条未定项两把尺都判过」）
#   DOC_DECISIONS_BREAK=user-verdict-owed  user-verdict-owed 格不跑、按判过了记
#       decision-documents-green（输出里找不到「待用户复核的条款都有未还的账盯着」）；decision-documents-some-cells-not-run（输出里找不到「  user-verdict-owed：本次未跑」）；
#       user-verdict-owed-red（期望退出 1，实测 0）
#   DOC_DECISIONS_BREAK=decision-summary-width  decision-summary-width 格不跑、按判过了记
#       decision-documents-green（输出里找不到「决策索引结论列都不超 200 字（共 21 行）」）；decision-summary-width-red（期望退出 1，实测 0）
#   DOC_DECISIONS_BREAK=write-frequency  write-frequency 格不跑、按判过了记
#       decision-documents-green、decision-documents-some-cells-not-run（输出里找不到「  write-frequency：本次未跑」）；write-frequency-green（输出里找不到「同一结构的写频率只有一个量级…」）；
#       write-frequency-red（期望退出 1，实测 0）
#   DOC_DECISIONS_BREAK=entry-added  entry-added 格不跑、按判过了记
#       decision-history-green（输出里找不到「决策正文改了 7 行，变更史新增 1 条条目」）；decision-history-red（输出里找不到「决策正文改了 2 行（碰了 2 行状态行，不按小改动放行），…」）；
#       shape-green（输出里找不到「doc-decisions.sh：2 格里 1 格绿…」）
#   DOC_DECISIONS_BREAK=shape  shape 格不跑、按判过了记
#       decision-history-green（输出里找不到「查了 3 份 kb 文件、1 个 `### 日期` 标题…」）；decision-history-red（输出里找不到「.claude/kb/invariants.md:5  ##…」）；
#       shape-cells-marker-red、shape-red（期望退出 1，实测 0）；shape-green（输出里找不到「变更史的形状对：查了 7 份 kb 文件、11 个 `###…」）
#   DOC_DECISIONS_BREAK=history-loss  history-loss 格不跑、按判过了记
#       history-loss-green（输出里找不到「变更史整理没丢：查了 1 份，基准里每个日期块的数、编号、产…」）；history-loss-red（期望退出 1，实测 0）
#   DOC_DECISIONS_BREAK=write-frequency-levels-merged  write-frequency 不同级一律判同级
#       write-frequency-green（期望退出 0，实测 1）；write-frequency-red（输出里找不到「01-写频率样本甲.md:11「系统配置槽」写成「每次挂载」…」）
#   DOC_DECISIONS_BREAK=write-frequency-negation-asserts  write-frequency 否定形态也算断言
#       write-frequency-green（期望退出 0，实测 1）；write-frequency-red（输出里找不到「01-写频率样本甲.md:11「tail」否定了「每次发布」…」）
#   DOC_DECISIONS_BREAK=write-frequency-subject-in-sentence-only  write-frequency 叫法只从句内认
#       write-frequency-green（输出里找不到「同一结构的写频率只有一个量级…」）；write-frequency-red（输出里找不到「01-写频率样本甲.md:23「tail」（叫法在所属标题）…」）
#   DOC_DECISIONS_BREAK=write-frequency-negation-adjacent-only  write-frequency 否定词只认紧挨频率词的
#       write-frequency-green（期望退出 0，实测 1）；write-frequency-red（输出里找不到「01-写频率样本甲.md:12「系统配置槽」否定了「每次发布…」）
#   DOC_DECISIONS_BREAK=write-frequency-restriction-as-plain  write-frequency「只在 / 仅」当普通断言
#       write-frequency-green（输出里找不到「同一结构的写频率只有一个量级…」）；write-frequency-red（输出里找不到「01-写频率样本甲.md:15「系统配置槽」用「只在 / 仅…」）
#   DOC_DECISIONS_BREAK=stock-judged  shape 把存量也当新增判
#       shape-green（期望退出 0，实测 1）；shape-red（输出里找不到「2 处新写的标题把日期与「（其N）」粘在同一个 `### `…」）
#   DOC_DECISIONS_BREAK=fused-ignored  shape 不判 a（日期与序号粘在一起）
#       decision-history-red（输出里找不到「.claude/kb/invariants.md:5  ##…」）；shape-cells-marker-red（期望退出 1，实测 0）；
#       shape-green（输出里找不到「存量（基准里就有，等搬迁，只计数不判红）：日期与序号粘在一起…」）；shape-red（输出里找不到「2 处新写的标题把日期与「（其N）」粘在同一个 `### `…」）
#   DOC_DECISIONS_BREAK=no-home-ignored  shape 不判 b（日期块没住进自己的节）
#       shape-green（输出里找不到「存量（基准里就有，等搬迁，只计数不判红）：日期与序号粘在一起…」）；shape-red（输出里找不到「4 个新写的日期块没住进自己的节」）
#   DOC_DECISIONS_BREAK=duplicate-date-ignored  shape 不判 c（同节同一个日期开几块）
#       shape-green（输出里找不到「存量（基准里就有，等搬迁，只计数不判红）：日期与序号粘在一起…」）；shape-red（输出里找不到「2 组同一节里同一个日期开了几个 `### ` 块」）
#   DOC_DECISIONS_BREAK=wrapper-ignored  shape 不判 d（旧形态包装）
#       shape-green（输出里找不到「存量（基准里就有，等搬迁，只计数不判红）：日期与序号粘在一起…」）；shape-red（输出里找不到「3 行新写进变更史的旧形态包装…」）
#   DOC_DECISIONS_BREAK=empty-ignored  shape 不判 e（日期块下没有变更项）
#       shape-green（输出里找不到「存量（基准里就有，等搬迁，只计数不判红）：日期与序号粘在一起…」）；shape-red（输出里找不到「1 个新开的 `### 日期` 块下一条 `- ` 变更项都…」）
#   DOC_DECISIONS_BREAK=candidate-codes-unchecked  candidate-codes 复用的候选代号不判红，汇总照样记绿
#       candidate-codes-red（期望退出 1，实测 0）
set -uo pipefail
# 路径要在 cd 之前算：$0 是相对路径时，cd 进项目根之后就指不到了
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
STAGE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GATE_DIRECTORY="$STAGE_DIR"
REPO_ROOT="$(cd "$STAGE_DIR/../.." && pwd)"
STAGE_RELATIVE_PATH=".claude/gate.d/$(basename "${BASH_SOURCE[0]}")"
LIB_ITEM_REF_STATUS="$GATE_DIRECTORY/lib-item-ref-status.py"
LIB_CHANGED_PATHS="$REPO_ROOT/scripts/gate/changed-paths.sh"
# 生成器按本脚本所在的仓取、不按项目根：样本根里没有 .claude/scripts/（坑见格 blocking-verdict）
GENERATOR="$REPO_ROOT/.claude/scripts/gen-decision-items.py"
DECISION_ITEMS_WRITE=0
BREAK_SWITCH="${DOC_DECISIONS_BREAK:-}"
# 握手变量 GATE_NOT_RUN_FILE 只留给共用库汇总时写，不导出给各格的子进程（command-safety.md「进程边界上的三种静默失效」）
export -n GATE_NOT_RUN_FILE 2>/dev/null || true
source "$STAGE_DIR/lib/stage-cells.sh"
bad() { printf '  ✗ %s\n' "$*"; }
ok()  { printf '  ✓ %s\n' "$*"; }
howto() { printf '     → %s\n' "$*"; }

stage_cell kb-shape judge_kb_shape "kb 形状：分项只许一种叫法、kb 文件不链回自己、规则路径带 .claude/、决策标题与分项两节、索引页分项计数与进度计数与正文相符" \
  "照这一格各段红行下面的出路改（分项叫法、自指链接、规则路径、决策标题与分项两节、分项的进度标记、索引页状态列与进度列）；两列的计数以正文两节为准"
stage_cell item-ref-status judge_item_ref_status "每一处「D<n>（简称） 已定项 k / 未定项 k」写的状态与正文分项表相符" \
  "照红行把引用处写的「已定项 / 未定项」改成与那条决策正文分项表一致；分项改了号的，引用处跟着改号（scripts/kb/relabel-item.py）"
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
stage_cell settled-same-file judge_settled_same_file "这次在一份决策里新增了「已定」小节（含「已定项 N：…」分项定案节），同文件开着的未定项与索引页的待议节一个字没动" \
  "被顺带定掉的改写成「已定，权威记录在 XX」或收摊，仍然开着的在条目里补一行「YYYY-MM-DD 复核过，仍然开着，因为 …」；decisions.md 的待议节同样回看"
stage_cell freeze-layer-membership judge_freeze_layer_membership "冻结层归属登记表：每层每个结构都有行、写「退出」的必是派生态、依据指得到已定分项" \
  "照红行给缺的层与结构补一行（态别按 D21 已定项 10 的判据推，推不出写「判不动」）、写「退出」的行核态别、依据写成指得到的「D<n>（简称） 已定项 k」"
stage_cell decision-items-sync judge_decision_items_sync "decisions.md 的分项清单生成块与索引表「状态」「进度」两列，与生成器的输出逐字相同" \
  "权威记录是各决策正文的「### 已定项」/「### 未定项」两节；改完正文跑 bash .claude/gate.d/doc-decisions.sh --write 重写分项清单与索引表状态列、进度列"
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
stage_cell shape judge_shape "变更史的形状：日期块住在自己的节、同节同一天只一块、块下是 - 变更项而不是 #### 子条目、不留快查与现状行" \
  "日期写在 \`### \`，块下每条改动一个顶格 \`- \` 项，条目搬进自己的 \`## D<n>（\` / \`## E<n>（\` 节，同节同一天并成一个日期块；\`#### \`、「（其N）」、快查两行与现状行删掉（.claude/rules/changelog-format.md）"
stage_cell history-loss judge_history_loss "两份变更史与基准相比，基准里每个日期块的数、编号、产物名、引名在改后都找得到（提交前整理不许丢）" \
  "红行列出的数或编号逐个补回那一块、改成指针或 git 引用；确认是有意删的过程数才可以不补，那就在同一块里写一行「去向」说明"
stage_cell candidate-codes judge_candidate_codes "同一份决策里，同一个候选代号（甲乙丙丁… 或单个大写字母）不许出现在两个分项的候选表里" \
  "红行点名的代号，在后写的那个分项里换一个这份决策里没用过的代号；引用别的分项的候选时写「已定项 7 的丙」带上分项，带了分项前缀的写法不算复用"
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

# ════════════════ 前十四格：决策正文、索引页与登记表 ════════════════

# 决策索引表「状态」列对各决策正文：只有格「kb 形状」用（第 5 段读成模块取 declared_open_count，第 7 段 python3 -c 跑 main），所以不另立库文件。
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

# 标题里声明的未定条数：阿拉伯数字或 1–99 的中文数字；第 5 段读同一份。只认一个字的中文数字会把「十一项未定」读成「一项未定」。
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
PROGRESS_MARK = re.compile(r'进度：\s*\*{0,2}(已实现|未实现|无对象)')


def count_progress(body):
    """数两节索引表里每条分项的进度标记，返回 (已实现, 未实现, 无对象, 缺标记)。行的取法与 count_items 同口径。"""
    done = todo = none = missing = 0
    for head in ("已定项", "未定项"):
        m = re.search(r'^### %s\s*$(.*?)(?=^#{1,3} |\Z)' % head, body, flags=re.M | re.S)
        seg = m.group(1) if m else ""
        cut = re.search(r'^#{4}\s', seg, flags=re.M)
        top = seg[:cut.start()] if cut else seg
        rows = re.findall(r'^\|\s*\d+\s*\|[^|]*\|.*$', top, flags=re.M) or re.findall(r'^\d+\.\s.*$', top, flags=re.M)
        for row in rows:
            marks = set(PROGRESS_MARK.findall(row))
            if len(marks) != 1:
                missing += 1
            elif "已实现" in marks:
                done += 1
            elif "未实现" in marks:
                todo += 1
            else:
                none += 1
    return done, todo, none, missing


def progress_cell_of(done, todo, none, has_items):
    if not has_items:
        return "无对象"
    return f"已实现 {done} 项 / 未实现 {todo} 项" + (f" / 无对象 {none} 项" if none else "")


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
        # 表格行要有闭合的第二根竖线，与 gen-decision-items.py 的 harvest、lib-item-ref-status.py 同口径：
        # 口径不一时一行畸形表格一边数得到、一边数不到，两边都不报错。
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
        # 破折号前的空格可选（D14 的标题就没有）；读不出首行判红，跳过会让少核一条在输出里看不出来。
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
            print("     → 怎么办：给它补一行 `| D<n>（简称） | 状态 | 结论（简报） | 正文 | 进度 |`，")
            print("       状态列与进度列跑 bash .claude/gate.d/doc-decisions.sh --write 生成。")
            fail = 1
            continue
        seen += 1
        isx = row.group(1).strip()
        n_settled, n_open = count_items(body.split("\n## 历史版本")[0])

        # 进度列（第 5 列）：本段自己数正文里的进度标记，不经生成器。转义竖线 `\|` 不算分隔。
        line = re.search(r'^\|\s*%s（[^|]*）.*$' % num, idx, flags=re.M).group(0)
        cells = [c.strip() for c in re.split(r'(?<!\\)\|', line.strip().strip("|"))]
        if len(cells) < 5:
            print(f"  ✗ {num} 索引行只有 {len(cells)} 列，缺「进度」列（第 5 列）")
            print("     → 怎么办：索引表写成 `| 决策 | 状态 | 结论（简报） | 正文 | 进度 |`，")
            print("       进度列跑 bash .claude/gate.d/doc-decisions.sh --write 生成。")
            fail = 1
        else:
            done, todo, none, missing = count_progress(body.split("\n## 历史版本")[0])
            want_cell = progress_cell_of(done, todo, none, n_settled + n_open > 0)
            if cells[4] != want_cell:
                print(f"  ✗ {num} 进度列写「{cells[4]}」，而正文两节里数出「{want_cell}」"
                      + (f"（另有 {missing} 条分项缺进度标记或写了两个）" if missing else ""))
                print("     → 怎么办：权威记录是各分项索引行末尾的 **进度：已实现 / 未实现 / 无对象。** 标记；")
                print("       改完正文跑 bash .claude/gate.d/doc-decisions.sh --write")
                fail = 1

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
        # 标题声明的未定条数也要与实数相符（标题同样会被检索到）；没声明按 0
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
        print(f"  ✓ 决策索引表的状态列与进度列与正文相符（{seen} 条决策，分项数与进度数是本段自己数的）")
    sys.exit(1 if fail else 0)


if __name__ == "__main__":
    main()
PY

# 格「未定项被别处定了」的复核判据：只有那一格用（逐条 python3 -c 跑），所以不另立库文件。
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
    # --no-color 写死：配了 color.ui = always 时行首多转义码，「@@」「+」「-」都认不出（绿样本 setup.sh 就这样配）。
    result = subprocess.run(['git', 'log', '--no-color', '-L', f'{first_line_number},{last_line_number}:{path}', '--format=%x00%ct'],
                            capture_output=True, text=True)
    if result.returncode != 0:
        # 失败时 stdout 可能是半截（-L 边走边打），不能当历史用
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
        # 条目块在 HEAD 里就一定有引入它的提交：退 0 却一个都没交出来是判不了，不是「复核过了」
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
# 同一件事在 kb 里只许一种写法（.claude/singlefs-ai-sop/rules/kb-discipline.md）；位置、自指、时间指代归上游 doc-lint.sh。
# 样本：kb-shape-red 每一段至少犯一次；第 7 段的两种红互斥，放的是「状态列的分项计数与正文不符」。
grid_kb_shape() (
  KB=.claude/kb
  fail=0
  bad() { printf '  ✗ %s\n' "$*"; fail=1; }
  ok()  { printf '  ✓ %s\n' "$*"; }
  howto() { printf '     → %s\n' "$*"; }

  # 文件装进数组，不靠 $(find …) 的分词：带空格的文件名会被切成两个不存在的路径
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
  # 用法：grep_files <段名> <grep 参数…> -- <文件…>；命中写进 $grep_hits。
  # 退 0 都读到了；退 1 没有要扫的文件；退 2 有文件读不了（读到的照样交命中）。非 0 都已判红，调用方只在退 0 时报绿。
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
  # 历史类文件（*-history.md、decisions-history/ 下）整份不判：里面是当时的原文，分不出哪一句换了会成假话；
  # 它们里的自链靠人看（.claude/rules/path-moves.md「改一个全仓术语：正文之外还有五处会红」）。
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
  INDEX_VS_BODY_SOURCE="$INDEX_VS_BODY_SOURCE" python3 - $KB/decisions/*.md <<'PY'
import os, re, sys, types
# 标题里未定条数的读法与第 7 段同一份（INDEX_VS_BODY_SOURCE 的 declared_open_count）
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

# 编号连续：每个文件一个标题、按文件名次序读入，所以直接比
nums = [int(m.group(1)) for m in re.finditer(r'^## D(\d+) ', body, flags=re.M)]
if not nums:
    bad("一个 `## D<n>` 标题都没读到——这一段又变成恒绿了，先修检查再谈别的",
        "确认 decisions/*.md 首行是 `## D<n> 简称 —— 状态`；读不到就是这段检查在空转。")
elif nums != list(range(1, len(nums) + 1)):
    bad(f"决策编号不连续或不从 D1 起：{nums}",
         "决策编号必须从 D1 起连号；废弃一条也要留占位并在变更史里写明。")

starts = [m.start() for m in re.finditer(r'^## D\d+ ', body, flags=re.M)]
starts.append(len(body))
decision_count = len(starts) - 1
for a, b in zip(starts, starts[1:]):
    blk = body[a:b]
    title = blk.split("\n", 1)[0]
    if not re.search(r'——\s*(已定|半定|待定)', title):
        bad(f"标题缺状态（已定/半定/待定）：{title[:50]}",
            "首行写成 `## D<n> 简称 —— 已定/半定/待定（…）`，索引行要与它一致。")
    # 只看每节第一个 `####` 之前的索引：之后是各分项的论证，里面另有编号列表
    def index_of(head):
        sec = re.search(r'^### %s$(.*?)(?=^#{1,3} |\Z)' % head, blk, flags=re.M | re.S)
        if not sec:
            return None
        t = sec.group(1)
        cut = re.search(r'^#{4}\s', t, flags=re.M)
        return t[:cut.start()] if cut else t
    # ①② 每条分项索引行要带规范标记「状态：已定 / 未定」且与所在节一致：单条取出时出来的是这一行，不是小节标题。
    #    不靠「取行内第一个状态词」猜：合法分项常先提别处的状态。加粗与不加粗都认。
    #    判不了：标记写对而后半句自相矛盾，欠账 C51（分项引用只验状态不验身份）。
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
            # 进度标记与状态标记并排：已实现 / 未实现 / 无对象 三选一，缺了与写了两个不同的值都红（判法在 decisions.md「进度」一段）。
            progress_marks = sorted(set(re.findall(r'进度：\s*\*{0,2}(已实现|未实现|无对象)', row)))
            if not progress_marks:
                bad(f"{title.split()[1]} {head}里有一条分项没写「进度：已实现 / 未实现 / 无对象」这个规范标记："
                    f"{row[:56]}",
                    "在这一行的 **状态：…。** 之后补上 **进度：已实现。**、**进度：未实现。** 或 **进度：无对象。**；判法在 decisions.md「进度」一段。")
            elif len(progress_marks) > 1:
                bad(f"{title.split()[1]} {head}里有一条分项写了两个不同的进度标记（{'、'.join(progress_marks)}）："
                    f"{row[:56]}",
                    "一行只许一个「进度：」标记，删掉多余的那个。")
    # ③ 已定项一侧的正文不许自陈本项没定；措辞只认「说本项自己没定」那几句，不误伤「别处仍未定」。
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
    # ④ 两节共用一套编号，合起来要唯一且从 1 连到 n：重号让一个号指两件事，断号像有分项被删
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
  # 本段自己数正文、不经生成器（理由在 INDEX_VS_BODY_SOURCE 的文档串）
  if python3 -c "$INDEX_VS_BODY_SOURCE" "$KB/decisions.md" $KB/decisions/*.md; then :; else fail=1; fi

  echo
  if [[ $fail -eq 0 ]]; then echo "  ✓ kb 形状检查通过（kb 文件 ${#kb_md_files[@]} 份、规则 ${#rule_md_files[@]} 份）"; else echo "  ✗ kb 形状检查未通过"; fi   # gate-lint:summary
  exit $fail
)

# ════ 格「分项引用状态」════
# 每一处「D<n>（简称） 已定项 k / 未定项 k」都在断言那条分项的状态，权威是正文两节的索引表；扫 kb / records / research / crates 全域，含 .rs 注释。
# 判据、出路与成功行在 lib-item-ref-status.py 的 main（格「冻结层归属登记表」与 scripts/kb/relabel-item.py 也用这份库）。
# 样本：item-ref-status-red 判红。
grid_item_ref_status() (
  python3 "$LIB_ITEM_REF_STATUS"
)

# ════ 格「状态别说两遍」════
# 分项按状态分住两节，节本身就是状态；再写一遍会各自漂移（.claude/singlefs-ai-sop/rules/kb-discipline.md 第 4 条）。判三样：
#   1. 小节标题不写成「已定项 N —— 已定…」（编号后带括注的也算）
#   2. 同一条目不同时写「—— 已定 / 未定」与「状态：已定 / 未定」，两处值同不同都判
#   3. 条目的「状态：」与所在节一致
#   2、3 只看每节第一个 `####` 之前的索引，与格「kb 形状」第 5 段同一条边界；只认显式的「状态：X」与破折号形态，句中的「D18 已定项 3 已定」是引用。
#   一份都没扫到或一条索引行都没核到退 77。决策正文之外也扫实验正文与 records/，防同样的写法在那里复发。
# 样本：status-redundancy-red 三样各犯一次判红；decision-documents-green 里论证带「3. …状态：未定」的决策判绿。
grid_status_redundancy() (
  DEC=.claude/kb/decisions
  [[ -d "$DEC" ]] || { echo "  ! 找不到 $DEC，这一格跳过"; exit 77; }

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
        # 编号后面许跟一段括注（「已定项 3（2026-09-01）—— 已定」），否则带日期的标题整个漏过去
        if re.match(r'^#{3,4} (已定项|未定项) \d+ *(?:[（(][^）)]*[）)])? *—— *\*{0,2}(已定|未定)', line):
            bad.append((name, i, '标题重复', line.strip()[:60]))

    # 2 / 3. 条目层：节到下一个一至三级标题，再在第一个 `####` 处收口（与 index_of、count_items 同一条边界）
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

if not files or rows_checked == 0:
    print(f'  ! 本次无对象可判：扫了 {len(files)} 份、核到 {rows_checked} 条分项索引行（决策正文、实验正文、records/ 下的 .md 里没有分项索引）')
    sys.exit(77)
print(f'  ✓ 状态只说一遍，且分项都在对的节里（扫 {len(files)} 份、核 {rows_checked} 条分项索引行）')
PY
)

# ════ 格「说未定其实已定」════
# kb 正文里紧贴「D<n>（简称）」写的「未定」，与那条决策状态行比：状态不是「待定」（含「待定（…）」）判红。
# 与格「未定项被别处定了」不是一条：那一格只扫未定项小节、按时间比；这一格扫全部正文、按文本比，写在已定项论证、不变量清单、实验正文里的它才抓得到。
# 只认紧贴的写法并跳过「」引文：放宽到隔 40 字时假红远多于真红。代价是隔着名词短语的（「D2 的条带宽度可变粒度仍然未定」）漏掉，靠人看。
# 样本：cross-decision-status-red 里「D180 未定」而 D180 已定判红；decision-documents-green 里被引的那条标着待定（含「待定（两项未定）」）判绿。
grid_cross_decision_status() (
  KB="$PWD/.claude/kb"
  [[ -d "$KB/decisions" ]] || { echo "  ! 没有 $KB/decisions，这一格无对象可判"; exit 77; }

  python3 - "$KB" <<'PY'
import re, sys, pathlib

kb = pathlib.Path(sys.argv[1])

# 1) 每条决策的状态取正文标题行 `## D<n> 简称 —— 状态`：破折号前的空格可选（D14 没有）；解析不了的标题判红，不当成跳过
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

# 2) 紧贴：D<n>（简称） 之后只许空白、逗号、括号、「取值」「状态」与「仍 / 仍然 / 尚 / 都 / 均」，然后是「未定」（不是「未定项」）
ref_re = re.compile(r'D\d+(?:（[^）]*）)?')
tight_re = re.compile(r'^[\s，,、（(]*(?:取值|状态)?[\s：:]*(?:仍然|仍|尚|都|均)?未定(?!项)')

bad, in_hist = [], False
scanned_files, history_files, history_section_lines, quoted_references = 0, [], 0, 0
for f in sorted(kb.rglob("*.md")):
    rel = f.relative_to(kb.parent.parent)
    # 决策变更史整份是历史，写的是当时的状态
    if f.name == "decisions-history.md" or f.parent.name == "decisions-history":
        history_files.append(str(rel))
        continue
    scanned_files += 1
    in_hist = False
    for i, line in enumerate(f.read_text(encoding="utf-8").splitlines(), 1):
        # 只认整行的「## 历史版本」：前缀匹配会让「### 历史版本对照」把后半份文件整段跳过
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
# 一条决策的已定项区段（「### 已定项」索引与各「#### 已定项 N」论证）里出现「X 仍未定 / 还没定 / 尚未定」，看 X：
#   a. 自指（取值 / 本项 / 该项 / 本分项 / 本条；打头的「的」去掉再比）：标着已定却说自己没定，判红；
#   b. 同一决策已定项区段里另一条带编号、写着「已定」的行里有 X（兄弟分项已经定了它）：判红。
#   同一行逐处看，判出一处为止。没有一份决策有已定项区段退 77。
# 格「kb 形状」「分项引用状态」只比状态标记，看不见正文；这一格抓定案之后没清理的推导过程。
# 不判「本笔仍然未定」这类子问题：那是 kb-discipline 第 3 条要的显式记录不知道，所以只认自指与兄弟分项，宁可漏不可假红。
# 样本：settled-item-self-open-red 里 a 支「取值仍未定」、b 支「校验算法选哪个仍未定」各一条判红；decision-documents-green 里无害的「本分项取区间上端 5」判绿。
grid_settled_item_self_open() (
  DEC="$PWD/.claude/kb/decisions"
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
            # 「`T_time` 的取值仍未定」取到的是「的取值」：去掉打头的「的」再比自指词
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
if files_with_settled_region == 0:
    print(f"  ! 本次无对象可判：{len(decision_files)} 份决策正文里没有一份有已定项区段（### 已定项 / #### 已定项 N）")
    sys.exit(77)
print(f"  ✓ 已定项的正文没有把已经定了的东西说成未定（扫 {len(decision_files)} 条决策，"
      f"其中 {files_with_settled_region} 条有已定项区段、共 {settled_region_lines} 行）")
PY
)

# ════ 格「引用写已定紧跟说没定」════
# 引用写着「已定项 k」、归属到的那条也已定，后面紧跟「未定 / 没定 / 定不下 / 待定 / 空着」（中间只许一段括注、标点与「今天 / 仍 / 还」这类词）：判红。
# 抓的是改完标签、句子还在说没定的那一型：格「分项引用状态」只比标签，「说未定其实已定」只认紧贴的「未定」，「已定分项自称未定」只扫已定项小节里的自指。
# 归属用 lib-item-ref-status.py（与格「分项引用状态」同一份）。两份变更史与 records/ 写的是当时的状态，不扫，成功行逐个列出。
# 样本：settled-ref-says-open-red 判红；decision-documents-green 里 records/ 下同一种句子不扫，判绿。
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
if seen == 0:
    print(f"  ! 扫了 {len(files)} 个文件，没有一处归属到已定分项的「已定项」引用，这一格无对象可判")
    sys.exit(77)
print(f"  ✓ 扫了 {len(files)} 个文件、{seen} 处已定项引用，没有一处紧跟着说它没定")
print(f"    没扫 {len(skipped)} 个（两份变更史与 records/ 写的是当时的状态）" + ('：' + '、'.join(skipped) if skipped else ''))
PY
)

# ════ 格「未定项被别处定了」════
# 还开着的未定项的条目块点名了别的决策 Dn，而 Dn 最近一次新增「—— 已定」小节标题晚于这条未定项对 Dn 的最近一次复核：判红，要求复核。
# 按时间比、不按文本依赖图：陈旧的未定项多半没写「前置是某某」这种标记。
# git 取不到时间或条目块历史判红，不当「从没定过 / 复核过了」。没有一条未定项点名定过东西的别的决策退 77。
# 判不了：2026-08-29 决策文档拆分之前的历史（逐行历史在拆分处断了），欠账 C36（未定项检查的第三个盲区）。
# 样本：stale-open-items-red 另造缺旧版本对象的仓：一份决策的 git log -G、一份条目块的 git log -L 读不到，两处都要报。
grid_stale_open_items() (
  DEC=.claude/kb/decisions
  [[ -d "$DEC" ]] || { echo "  ! 没有 $DEC，这一格无对象可判"; exit 77; }
  git rev-parse --git-dir >/dev/null 2>&1 || { echo "  ! 不在 git 仓库里，这一格跳过"; exit 77; }

  # 每个决策状态行最后一次变动的提交时间；取不到（git 失败）记 -1，点名它的未定项逐条报「没比过」
  declare -A st_time
  time_unavailable=()
  for f in "$DEC"/*.md; do
    n=$(grep -m1 -oE '^## (D[0-9]+)' "$f" | awk '{print $2}') || continue
    [[ -n "$n" ]] || continue
    # 「定了新东西」取这个文件最近一次新增「—— 已定」小节标题的提交：决策首行几乎不变；-G 加行首锚定只认标题行，正文里的「已定」二字会造成假阳性。
    # git 的退出码先判再取第一行：「git log … | head -1」只交 head 的退出码，git 失败会被读成「从没定过」
    if ! t_out="$(git log -1 --format=%ct -G'^#{2,4} .*—— 已定' -- "$f")"; then
      st_time[$n]=-1
      time_unavailable+=("$n（$(basename "$f")）")
      continue
    fi
    t="$(head -n 1 <<<"$t_out")"
    # 从没加过「—— 已定」标题记 0、不触发复核；不退化成文件修改时间（那样每次补内容都把引用它的未定项全报一遍）
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
    while IFS=: read -r ln text; do
      [[ -n "$ln" ]] || continue
      open_items=$((open_items + 1))
      # 复核时间按被点名的决策逐条算，判据在 OPEN_ITEM_REVIEW_SOURCE：条目块的任意改动不算复核（否则什么改动都能消红），
      # 工作区行号先映射回 HEAD 再喂 git log -L（-L 按历史里的文件解释行号，上方插过行会错位成假红）。
      # 复核写一句点名 Dn 的「（YYYY-MM-DD 复核 Dn：……）」就满足。
      deps=()
      # 点名按整个条目块认（首行加续行，到下一条分项或标题之前、至多 20 行），与 OPEN_ITEM_REVIEW_SOURCE 的 block_end 同口径：续行里的点名也算
      block_text="$(awk -v s="$ln" 'NR == s { print; next }
          NR > s && NR <= s + 20 { if (/^[[:space:]]*[0-9]+\. / || /^##+ / || /^\|[[:space:]]*[0-9]+[[:space:]]*\|/) exit; print }' "$f")"
      # D 编号前面要是非字母数字：「RAID5」里的「D5」不是在点名 D5
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
    # 未定项按所在小节取，不按行内关键字滤：列表式条目行内没有「未定」，未定项也常提到别处的「已定」
    done < <(awk -F: '
        # 只认光秃秃的「### 未定项」（「### 未定项 3 —— 已定」也以它开头）；在第一个 #### 处收口，之后是论证里的编号列表
        /^### 未定项[[:space:]]*$/  { inside=1; next }
        /^### /              { inside=0 }
        /^#### /             { inside=0 }
        /^## /               { inside=0 }
        inside && /^[[:space:]]*[0-9]+\. |^\| [0-9]+ \|/ { print NR":"$0 }
      ' "$f")
  done

  if ((flagged)); then
    echo "     → 怎么办：逐条复核；已被别处定掉的就改写成「已定，权威记录在 XX」，"
    echo "               仍然开着的就把点名改成不构成依赖的写法。"
    exit 1
  fi
  if ((judged_items == 0)); then
    echo "  ! 查了 $open_items 条未定项，没有一条点名定过东西的别的决策，这一格无对象可判"
    exit 77
  fi
  echo "  ✓ 没有未定项被别处的更新甩在后面（查了 $open_items 条未定项，其中 $judged_items 条点名了定过东西的别的决策、逐条比过复核时间）"
)

# ════ 格「定了新东西没回看同文件未定项」════
# 这一轮在一份决策里新增了「已定」小节标题（`—— 已定`、`已定（…）`，或分项定案的 `已定项 N：…` / `已定项 N（…）`），而同文件还开着的未定项（「### 未定项」与「### 还要回答的」两节）一条都没碰、
# 索引页 decisions.md 还悬着的「## 待议」节一个字没动：判红，要一次回头看。
# 与格「未定项被别处定了」是两把尺：那一格跨文件看历史，这一格同文件看这一轮的 diff。按历史判会在每个有已定小节又有未定项的决策上恒红。
# 「碰没碰」按条目块判（表格行一行一条，列表项到下一条分项或标题之前，与 OPEN_ITEM_REVIEW_SOURCE 同口径）；待议按那一节判（`## 待议` 到下一个 `## `），索引页别处动过不算。
# 新增了已定小节的几份里一条未定项都没有、索引页也没有待议节时退 77。
# 样本：settled-same-file-red 另放一份两行表格的决策（只改了第 2 行，第 1 行要报）与一节没动的待议（同一次只改了索引表那一行）。
# settled-same-file-item-heading-red / -green 新增的是现行写法 `#### 已定项 N：名字`：红的那份只有未定项没动，绿的那份未定项与待议节都补了复核注。
grid_settled_same_file() (
  DEC=.claude/kb/decisions
  IDX=.claude/kb/decisions.md
  [[ -d "$DEC" ]] || { echo "  ! 没有 $DEC，这一格无对象可判"; exit 77; }
  git rev-parse --git-dir >/dev/null 2>&1 || { echo "  ! 不在 git 仓库里，这一格跳过"; exit 77; }

  # 改动范围取共用脚本 changed-paths.sh，不另算一份（code-tooling 的 change-range-single-source 格判）；名单带未跟踪文件
  # shellcheck source=../../scripts/gate/changed-paths.sh
  source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"; echo "     → 怎么办：它随仓走（scripts/gate/changed-paths.sh），被删了就从 git 找回来。"; exit 1; }
  BASE="$(gate_diff_base gate)"

  # 名单先落到变量、判过退出码再读：git 失败时名单是空的，会被读成「本次没有新增已定小节」
  all_changed="$(gate_changed_paths "$BASE" untracked)" || {
    echo "  ✗ 取不到这次改动碰了哪些路径（基准 $BASE），这次改了哪些决策文件没取到"
    echo "     → 按上面 git 的报错修好仓库状态（基准 $BASE 要存在；仓里还没有提交就先提交一次）再跑；git 失败时这一格什么都没比，不是通过。"
    exit 1
  }
  changed_names="$(grep "^\.claude/kb/decisions/" <<<"$all_changed" || true)"

  settled_files=()
  while IFS= read -r f; do
    [[ -n "$f" ]] || continue
    # 只认新增行里的小节标题：`## D1 简称 —— 已定` / `### 未定项 3 —— 已定` / `### 已定（…）` / `#### 已定项 12：名字` / `#### 已定项 12（名字）`；未跟踪的文件整份算新增
    # 不接 `| grep -q`：pipefail 下前段吃 SIGPIPE，命中会被读成没命中
    added_out="$(gate_added_lines "$BASE" "$f")" || {
      echo "  ✗ 取不到 $f 这次新增了哪些行（基准 $BASE）"
      echo "     → 按上面 git 的报错修好仓库状态再跑；取不到新增行时这一格什么都没比，不是通过。"
      exit 1
    }
    added_text="$(cut -f2- <<<"$added_out")"
    # 新分项的定案标题写作 `#### 已定项 N：名字`（format-evolution.md「决策正文只写现状」）；老写法 `—— 已定` 与 `已定（` 照认
    # 开关 settled-item-heading-unrecognised 退回不认「已定项 N」的老正则（样本自检用）
    settled_heading_pattern='^#{2,4} .*—— 已定|^#{2,4} 已定[（(]|^#{2,4} 已定项[[:space:]]*[0-9]+[[:space:]]*(：|:|（|\()'
    if [[ "$BREAK_SWITCH" == settled-item-heading-unrecognised ]]; then
      settled_heading_pattern='^#{2,4} .*—— 已定|^#{2,4} 已定[（(]'
    fi
    if grep -qE "$settled_heading_pattern" <<<"$added_text"; then
      settled_files+=("$f")
    fi
  done <<<"$changed_names"

  if ((${#settled_files[@]} == 0)); then
    echo "  ! 本次 diff 没有新增「已定」小节，这一格无对象可判"
    exit 77
  fi

  # touched_lines_of <文件>：这一轮碰过的行号（新文件侧）一行一个，未跟踪的整份都算碰过。
  # ls-files --error-unmatch 退 0 跟踪、1 未跟踪，别的码返回 2 由调用方判红：当成未跟踪会整份算碰过、一条都不报。
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
    touched="$(touched_lines_of "$f")" || {
      echo "  ✗ 判不了 $f 有没有被 git 跟踪（上面是 git 的退出码），这一份的未定项这一轮没比"
      echo "     → 怎么办：按 git 的报错修好仓库状态（索引坏了就 git status 看、必要时重建索引）再跑；判不了不是「未跟踪、整份都碰过」。"
      exit 1
    }
    while IFS=: read -r ln text; do
      [[ -n "$ln" ]] || continue
      open_checked=$((open_checked + 1))
      end=$(awk -v s="$ln" 'NR>s && (/^[[:space:]]*[0-9]+\. /||/^\|[[:space:]]*[0-9]+[[:space:]]*\|/||/^##+ /){print NR-1; exit}' "$f")
      [[ -n "$end" ]] || end=$((ln+20))
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
  done

  # ── ② 索引页里还悬着、而本次一个都没碰的「待议」节 ──────────────
  if [[ -f "$IDX" ]]; then
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
# .claude/kb/freeze-layer-membership.md 那张表（按整行的 <!-- freeze-layer-membership:table --> 定位，表头五列逐字），清单从哪来见那份文件「门禁判哪三条」：
#   ① 每层、每个独立冻结组件至少一行；每棵树在 key 编码层与索引节点内部布局层各至少一行（结构列写「树 ID <n>」）；每个单元类码至少一行（结构列写「码 <n>」）。
#      三份清单都现读、不抄：D15 已定项 7 的「第 N 层：」与「- ① **…**」（两层的号按「key 编码」「索引节点内部布局」认）、
#      crates/singlefs-format/src/lib.rs 的 TREE_IDENTIFIER_*（WATERMARK 两个是水位）、D18 已定项 11 登记表里首列是整数、类名不是「无效」「保留」的行。任一份读成 0 项判红。
#   ② 写「退出」的行态别必须是派生态。
#   ③ 依据列点名的「D<n>（简称） 已定项 k」要存在且已定（归属借 lib-item-ref-status.py）；依据以「判不动」开头的不判。
#   层、态别、退出三列只认封闭的几个值，写别的判红（一个错别字会让 ② 静默放行）。② 判不到的「判不动」行、态别判不动而退出已定的行，成功行逐个列出。
# 判不了：某一行的态别对不对（改成自洽的「派生态 + 退出」三条都不红）、判不动的理由、三份清单都没单列的结构（树表单元、实例表单元、容器索引）。
# 样本：freeze-layer-membership-red 漏一棵树的 key 编码行、漏组件 ② 与码 16 两行、一行「权威态 + 退出」、依据指到不存在的分项（一处接在「已定项 5 / 98」后面）、
#   未定项、不存在的决策（简称里套括号），判红；decision-documents-green 里是真表原样的一份（编号与分项号改成连号），判绿。
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
    # 表在第一个非 | 行（含空行）处结束：下一张表只隔一个空行时，不这样切会被并进来
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
    # 简称里可以再套一层全角括号（D14（双轨（大小文件 / 持久临时））），一处引用可以接几个项号（「已定项 3 / 4」）：都要认全，否则无声漏判
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
        echo "      推不出就写「判不动」并在表下「判不动的十一行，各卡在哪一句」里写清卡在哪一句；"
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
        echo "      分项还没定就不能拿它当依据——这一行的态别改成「判不动」，依据写「判不动，推导见「判不动的十一行，各卡在哪一句」」。"
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
# decisions.md 的分项清单生成块与索引表「状态」「进度」两列是各决策正文的投影，与生成器 .claude/scripts/gen-decision-items.py 的输出逐字比。
# 状态列写分项计数（`已定 6 项 / 未定 0 项`，没有分项写「无分项 · 整条已定」）；三态词的权威在正文首行。
# 进度列（最后一列）写实现计数（`已实现 3 项 / 未实现 1 项`，有无对象的再带 ` / 无对象 2 项`；没有分项写「无对象」），取自各分项索引行末尾的 **进度：…。** 标记。
# 计数本身由格「kb 形状」第 7 段（INDEX_VS_BODY_SOURCE）不经生成器另数：只靠这一格时，生成器数错索引页会跟着错而仍逐字相同。
# --write 时把两块写回被判的仓的 decisions.md（用法在文件头）。
# 样本：decision-items-sync-red 状态列写错、索引漏登、索引多一行、生成块漂了各一处，判红。
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
  # 生成器的 stderr 里有它自己的出路，不吞
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

  # ── 索引表「状态」列：只改第 2 列，整行重写会抹掉别的会话刚写进结论列的东西 ──
  python3 - "$IDX" "$([[ $WRITE == 1 ]] && echo write || echo check)" "$cells_f" <<'PY'
import re, sys

idx_path, mode, cells_path = sys.argv[1:4]
text = open(idx_path, encoding='utf-8').read()
want = {}
for l in open(cells_path, encoding='utf-8').read().splitlines():
    if l.strip():
        num, status_cell, progress_cell = l.split('\t')
        want[num] = (status_cell, progress_cell)

bad, n, wrote = [], 0, 0
for num, (cell, progress) in want.items():
    pat = re.compile(r'^(\|\s*%s（[^|]*）\s*\|)([^|]*)\|' % re.escape(num), re.M)
    m = pat.search(text)
    if not m:
        bad.append(f"{num} 有正文，而决策索引表里没有它的行")
        continue
    n += 1
    if m.group(2).strip() != cell:
        if mode == 'write':
            text = text[:m.start(2)] + f" {cell} " + text[m.end(2):]
            wrote += 1
        else:
            bad.append(f"{num} 状态列写着「{m.group(2).strip()}」，正文投影是「{cell}」")
    # 进度列是最后一列：只改行尾那一格，前面几格一个字不碰（结论列里可能有转义竖线）。
    line_m = re.search(r'^\|\s*%s（[^|]*）.*$' % re.escape(num), text, re.M)
    cells = [c.strip() for c in re.split(r'(?<!\\)\|', line_m.group(0).strip().strip('|'))]
    if len(cells) < 5:
        bad.append(f"{num} 索引行只有 {len(cells)} 列，缺「进度」列（第 5 列，写成 `| 决策 | 状态 | 结论（简报） | 正文 | 进度 |`）")
        continue
    if cells[4] == progress:
        continue
    tail = re.search(r'\|([^|]*)\|\s*$', line_m.group(0))
    if mode == 'write':
        a, b = line_m.start() + tail.start(1), line_m.start() + tail.end(1)
        text = text[:a] + f" {progress} " + text[b:]
        wrote += 1
    else:
        bad.append(f"{num} 进度列写着「{cells[4]}」，正文投影是「{progress}」")

# 反过来查索引表多出的行（正文删了或改名了），上面的循环够不着
for row in re.findall(r'^\|\s*(D\d+)（', text, re.M):
    if row not in want:
        bad.append(f"{row} 在决策索引表里有行，而 decisions/ 下没有它的正文")

if mode == 'write':
    open(idx_path, 'w', encoding='utf-8').write(text)

if bad:
    print(f"  ✗ 决策索引表的状态列或进度列与正文不同步（{len(bad)} 处）")   # gate-lint:summary
    for b in bad:
        print("     " + b)                                          # gate-lint:detail
    print("     → 怎么办：权威记录是各决策正文的「### 已定项」/「### 未定项」两节。")
    print("       改完正文跑： bash .claude/gate.d/doc-decisions.sh --write")
    sys.exit(1)

print(f"  ✓ 决策索引表状态列与进度列与正文同步（{n} 条决策"
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
    # 生成器一条决策都没产出、生成块也空：两个空串相等不算比过
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
# 每条未定项（清单取自生成器 gen-decision-items.py，与格「决策分项清单与正文同步」同一个解析）的登记行或列表条目里要有两把尺的判定，缺就红：
#   改新池新建文件的字节：是 / 否 / 无对象（YYYY-MM-DD，依据：…）
#   动不动格式：动 / 不动 / 界不定（YYYY-MM-DD，依据：…）
# 只认这一种写法，散文措辞分不开「判过否」与「随口提了一句阻塞」；日期不许省，判定会因别处定案而过期。
# 第一把尺问：不定它，新池新建文件写出的字节（mkfs 写的也算）能不能由已定条款唯一确定？能为否，不能为是。
# 刻意不叫「阻塞」：那个词同时罩着「不改字节」与「改、但接受先写后改」，挡不挡开工是处置，写在依据里。
# 与格「未定项被别处定了」「定了新东西没回看同文件未定项」的分工：那两格管判定会不会过期，这一格管判定在不在。
# 登记行由这一格自己定位，所以另比条数：定位到的与生成器认的未定项对不上（漏了，或多了：例如表格行缺收尾竖线生成器不收）判红。
# 只判判定在不在，判得对不对（依赖对）还欠着：C50（阻塞标记没人维护）。
# 样本：blocking-verdict-red 两把尺各缺、外加一行表格缺收尾竖线（多定位出），判红。
grid_blocking_verdict() (
  DEC=.claude/kb/decisions
  # 生成器按本脚本所在的仓取（GENERATOR），不按 cwd：样本把 cwd 换成只放着样本 kb 的临时目录，按 cwd 取会找不到生成器而跳过，红样本悄悄变绿。
  # 生成器自己 glob 的是 cwd 下的 kb，正合样本所需。
  GEN="$GENERATOR"
  [[ -d "$DEC" ]] || { echo "  ! 没有 $DEC，这一格无对象可判"; exit 77; }
  # 生成器随仓走，不在是被删了或挪了：退 1 不退 77，与格「决策分项清单与正文同步」一致
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

# 两把尺在同一批登记行上各判各的，不拿一把的结论填另一把：一条未定项可以前一把判「否」而后一把判「动」
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
    # 只取第一个 `####` 之前那一段，与生成器同口径
    cut = re.search(r'^#{4}\s', sec, re.M)
    top = sec[:cut.start()] if cut else sec
    # 每条分项的块从它的登记行到下一条登记行为止（表格形态就是一行）
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
# seen 只收 want 里的，多出来的要拿定位到的去比
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
# 决策正文里标着「待用户复核 / 等用户复核」的，checks-owed.md 里要有一笔未还的账在同一行点名那条决策（.claude/singlefs-ai-sop/skills/decide/SKILL.md 硬要求第 6 条）。
# 粒度是决策：一份正文标了几处，有一笔未还的账点名这条决策就算有。
# 「未还」看那一行有没有带日期的还清标记（`已还（20…`、`已还一半（…`），不按「已还」两字判：描述里出现这两个字会被误判成还清。已还的账不再被回看，要盯就单立一笔。
# 样本：user-verdict-owed-red 里 D316 只有一笔已还的账、D31 只有一笔点名 D319 的未还账，判红。
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
    # 先落到变量再判，末段不接 `grep -q`：pipefail 下命中会被读成没命中（shell-lint S7）。决策号按整个号认：D1 不命中 D10–D19 的账
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
  if [[ $checked -eq 0 ]]; then
    echo "  ! 决策正文里没有一处标着待用户复核，这一格无对象可判"
    exit 77
  fi
  echo "  ✓ 待用户复核的条款都有未还的账盯着（$checked 份决策正文、$marked_lines 行标着待用户复核；按决策判，一份决策有一笔未还的账点名它就算有）"
)

# ════ 格「决策索引结论列的宽度」════
# 决策索引表每一行的「结论」列不超过 decisions.md 里「「结论」列一律 N 字以内」那一句的上限；上限从那一句读、不写死（只许一处权威）。
# 样本：decision-summary-width-red 两行超限（一行结论里带转义竖线），判红。
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
    # 按没转义的竖线切格：结论里的 `\|` 是格子里的字（与格「冻结层归属登记表」同一种切法）
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
# 决策正文里落在登记结构上的写频率断言，不许与 .claude/gate.d/write-frequency.tsv 登记的量级说反话；.claude/gate.d/write-frequency-exclude.tsv 逐句登记该留的判红句。两张表的写法与量级、量的口径在各自的文件头。
# 扫 .claude/kb/decisions/*.md「## 历史版本」之前、围栏之外的行：去掉 ** 与反引号，按没转义的竖线切格、按「。；！？」切句。
# 断言：频率词后面同一小句里 12 字以内跟写动词（「写放大」不算），或紧跟「一次」；只有频率词、不跟写动词的不判。
# 归属：句里的每个登记叫法都算；句里没有就认同一表格行别的格，还没有就认所属最近一个标题。
# 三种形态：
#   否定（频率词前隔至多三个虚字的否定词，或频率词与写动词之间的「不 / 没 / 无需 / 无须」）：否定别的量级是划界、只计数；否定的时机被登记的罩住，判红。
#   限定（前面紧挨「只 / 仅 / 只有」，可带「在 / 于」）：写的量与登记的量不同就判红，同级不同量也红；「不只 / 不仅 / 不是只」不划界，按否定计数。
#   普通：不同级判红；登记的只是量级里一部分时机的，还要同量。
# 另判红：登记表写坏；权威分项不是已定的「D<n>（简称） 已定项 k」、决策不在或简称与首行不符（归属借 lib-item-ref-status.py）；
#   排除表一行指不存在的文件、那一段不足 8 字或一句判红句都没排到；登记了结构而一处落在它们身上的断言都没扫到。
# 看不见：词表外的频率说法与叫法、跨句指代、否定词与频率词之间隔着虚字表外的字、双重否定、叫法取自表格行或标题时说的是不是这个结构、
#   是不是稳态写频率。成功行逐个列出一处断言都没扫到的结构。登记表不在退 77。
# 样本：write-frequency-red 里不同级、否定同级、隔字与动词前否定三句、「只在 fsync 时写」、主语只在标题里的一句、权威是未定项、简称写错、
#   排除表一行指不存在的文件、一行没排到判红句，判红；write-frequency-green 里同级不同词、限定与登记同量、否定别的量级、不划界的「不仅 / 不是只」、
#   主语在别的格或只在标题、排除表登记的一句，判绿。
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
    # 登记里的空格是中英文之间可有可无的空白：「每次 fsync」也认「每次fsync」
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

# ════════════════ 后三格：变更史 ════════════════

# 在仓里时取 gate 基准写进 diff_base；不在仓里 diff_base 留空。source 失败、取不到基准就判红退出。
load_diff_base() {
  local LIB_CHANGED_PATHS
  diff_base=""
  git rev-parse --is-inside-work-tree >/dev/null 2>&1 || return 0
  LIB_CHANGED_PATHS="$REPO_ROOT/scripts/gate/changed-paths.sh"
  # shellcheck source=../../scripts/gate/changed-paths.sh
  source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用库 $LIB_CHANGED_PATHS"; echo "     → 怎么办：改动范围的取法只有那一份，恢复它，别在阶段里再抄一份。"; exit 1; }
  diff_base="$(gate_diff_base gate)" || { echo "  ✗ 共用库算不出 diff 基准"; echo "     → 怎么办：照上面 gate_diff_base 的报错修（多半是 GATE_BASE 写错了），再跑。"; exit 1; }
}

# ── entry-added：决策正文改了，变更史里要有新增条目 ──
# 改了 decisions.md 或 decisions/ 却没在 decisions-history.md 新增条目，判红（.claude/rules/format-evolution.md「硬约束」：已定决策变更必须记进变更史）。
# 只数有没有新增：顶格 `- ` 变更项，一个都没有按 `### 日期` 标题数；条目写得对不对靠人。
# 基准取 changed-paths.sh 的 gate_diff_base gate：已提交没推出去的决策改动也算。
# 小改动（增删合计不超过 4 行、一行状态都没碰：首行「## D<n> 简称 —— 状态」、「状态：」、已定项 / 未定项标题）不要求变更史，退 77。
# 两条路径放数组：拼成一个字符串再加引号会变成单个 pathspec，什么都匹配不到。
cell_entry_added() {
  local KB HIST diff_base diff_rc numstat changed decision_diff status_lines_touched added hist_diff
  KB=(.claude/kb/decisions.md .claude/kb/decisions)
  HIST=.claude/kb/decisions-history.md

  if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "  ! 不在 git 仓库里，这一格无对象可判"
    exit 77
  fi
  # 在仓里时每一道 git 都要成功：失败了判红，不许读成「没改」
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

  numstat="$(git diff "$diff_base" --numstat -- "${KB[@]}")" || git_failed "diff --numstat" "$?"
  changed=$(awk '{n+=$1+$2} END{print n+0}' <<<"$numstat")
  # 碰了状态行就不算小改动：「—— 半定」改成「—— 已定」只有两行，却最该留痕
  decision_diff="$(git diff "$diff_base" -U0 -- "${KB[@]}")" || git_failed "diff -U0" "$?"
  status_lines_touched=$(grep -cE '^[+-](## D[0-9]+ .*——|#{3,4} *(已定项|未定项)|.*状态：)' <<<"$decision_diff" || true)
  hist_diff="$(git diff "$diff_base" -- "$HIST")" || git_failed "diff（变更史）" "$?"
  added=$(grep -c '^+- ' <<<"$hist_diff" || true)
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
  howto "若这次改动推翻或定下了任何结论，在 $HIST 里那条决策的 \`## D<n>（简称）\` 节里加一条变更（.claude/rules/changelog-format.md）："
  howto "  这一天的 \`### $(TZ=Asia/Tokyo date +%F)\` 块已经有就在块里最下面加一个顶格 \`- \` 项，没有就在节标题下面新开一块（新的日期在上）；"
  howto "  项里写结论、依据与改前（改前写一行 git show <提交>:<路径>，不抄原文）；一条改动点名了几条决策，只在标题里点名的第一条决策的节里写全文，别的节写一行「- 见 D<n>（简称） 节同日条目「摘要」」。"
  howto "已定 / 未定项数或一句话现状变了，改 decisions.md 的索引表再跑 bash .claude/gate.d/doc-decisions.sh --write 同步索引表的状态列。"
  howto "纯排版改动可以拆成单独一个提交，那时这一格就无对象可判了。"
  exit 1
}

# ── history-loss：变更史整理不许丢 ──
# 整理变更项（.claude/rules/changelog-format.md）不许丢数、编号、产物名、别的日期、引名。两份变更史各拿基准里那一版与现在的比：
# scripts/kb/history-consolidate.py loss-check（判法与出路在它的文件头），任一份退非 0 判红；都与基准无差异或基准里没有退 77。
cell_history_loss() {
  local diff_base checked=0 failed=0 hist base_copy rc
  if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "  ! 不在 git 仓库里，这一格无对象可判"
    exit 77
  fi
  load_diff_base
  for hist in .claude/kb/decisions-history.md .claude/kb/experiments-history.md; do
    [[ -f "$hist" ]] || continue
    if git diff --quiet "$diff_base" -- "$hist" 2>/dev/null; then continue; fi
    if ! git cat-file -e "$diff_base:$hist" 2>/dev/null; then continue; fi
    base_copy="$(mktemp "${TMPDIR:-/tmp}/history-loss-base.XXXXXX")"
    git show "$diff_base:$hist" > "$base_copy"
    rc=0
    python3 "$REPO_ROOT/scripts/kb/history-consolidate.py" loss-check "$base_copy" "$hist" --repo "$PWD" --force > "$base_copy.out" 2>&1 || rc=$?
    checked=$((checked + 1))
    if ((rc != 0)); then
      failed=$((failed + 1))
      echo "  ✗ $hist：整理之后基准里的这几块有数或编号找不到了"   # gate-lint:summary
      grep -E '^  ' "$base_copy.out" | sed 's/^/     /'   # gate-lint:detail
    fi
    rm -f "$base_copy" "$base_copy.out"
  done
  if ((failed)); then
    howto "红行列出的数或编号逐个补回那一块、改成指针或 git 引用；判法与出路在 scripts/kb/history-consolidate.py 文件头。"
    exit 1
  fi
  if ((checked == 0)); then
    echo "  ! 两份变更史与基准 $diff_base 无差异，这一格无对象可判"
    exit 77
  fi
  ok "变更史整理没丢：查了 $checked 份，基准里每个日期块的数、编号、产物名、引名在改后都找得到"
}

# ── shape：变更史的形状 ──
# 形态是「条目 → 日期 → 变更清单」：每条决策 / 实验一个 `## D<n>（` / `## E<n>（` 节，节里按 `### YYYY-MM-DD` 分块，块下每条改动一个顶格 `- ` 项（.claude/rules/changelog-format.md）。判五样：
#   a) .claude/kb/ 下任何 .md 里 `### 20YY-MM-DD` 后面直接跟「（其…）」
#   b) 两份变更史里日期块的上一级标题不是 `## D<n>（` / `## E<n>（`；文末「## 历史版本」里的块不算节外，但块里标题或 `- ` 项点名了 `D<n>（` / `E<n>（` 的照样判红
#   c) 同一节里同一个日期的 `### ` 出现两次以上
#   d) 两份变更史里的 `#### ` 子条目、「> 快查·」行、「**现状**：」行
#   e) 节里的 `### 日期` 块一条 `- ` 项都没有
# 只判这一轮新增的（基准同 entry-added）：a、d 看那一行，b 看块里任一行，c 看重复的几行里任一行，e 看 `### ` 行；基准里就有的只计数不判红。
# 新增行按 `git diff -U0` 的 hunk 头取行号（changed-paths.sh 的 gate_added_lines 只给行文），未跟踪的整份算新增；不在 git 仓里整棵树按新增判。围栏里的行不算。
# 判不了：变更项写得对不对、该点名的决策或实验点没点全、日期块是不是倒序。
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

fused_new, no_home_new, duplicate_new, wrapper_new, empty_new = [], [], [], [], []
fused_stock = no_home_stock = duplicate_stock = wrapper_stock = empty_stock = 0
WRAPPER = re.compile(r'^(?:####[ \t]|>[ \t]*快查·|\*\*现状\*\*：)')
ITEM = re.compile(r'^- ')
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
        if WRAPPER.match(line) and BREAK != 'wrapper-ignored':
            if is_new(changes, path, [number]):
                wrapper_new.append(f'{path}:{number}  {line[:70]}')
            else:
                wrapper_stock += 1
            continue
        match = DATE_HEADING.match(line)
        if not match:
            continue
        history_date_headings += 1
        block = [(number, line)]
        items = 0
        for later_number, later_line in lines[index + 1:]:
            if BLOCK_END.match(later_line):
                break
            block.append((later_number, later_line))
            if ITEM.match(later_line):
                items += 1
        in_section = bool(parent and section_heading.match(parent[1]))
        if in_section and items == 0 and BREAK != 'empty-ignored':
            if is_new(changes, path, [number]):
                empty_new.append(f'{path}:{number}  {line}')
            else:
                empty_stock += 1
        if in_section:
            by_section.setdefault(parent, {}).setdefault(match.group(1), []).append(number)
            continue
        if parent and OWN_HISTORY.match(parent[1]):
            named = sorted({found.rstrip('（') for _, text in block if text.startswith('#') or ITEM.match(text) for found in mention.findall(text)})
            if not named:
                continue
            reason = f'在文件自己的「## 历史版本」里，标题或变更项点名了 {"、".join(named)}，要写进那一节'
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
         f'条目住在节外 {no_home_stock} 处、同节日期重复 {duplicate_stock} 组、旧形态包装 {wrapper_stock} 行、空日期块 {empty_stock} 个')
failed = False
if fused_new:
    failed = True
    print(f'  ✗ {len(fused_new)} 处新写的标题把日期与「（其N）」粘在同一个 `### ` 里')  # gate-lint:summary
    for item in fused_new:
        print(f'     {item}')  # gate-lint:detail
    print('     → 同一天的改动收进一个 `### YYYY-MM-DD`（这一组里这一天已经有就并进去），块下每条改动一个顶格 `- ` 项，「（其N）」不再写。')
    print('       规则：.claude/rules/changelog-format.md。')
if no_home_new:
    failed = True
    print(f'  ✗ {len(no_home_new)} 个新写的日期块没住进自己的节')  # gate-lint:summary
    for item in no_home_new:
        print(f'     {item}')  # gate-lint:detail
    print('     → decisions-history.md 的变更写进它点名的那条决策的 `## D<n>（简称）` 节，experiments-history.md 的写进 `## E<n>（简称）` 节；')
    print('       点名了几条只在标题里点名的第一条的节里写全文，别的节写一行「- 见 … 节同日条目」。文件自己的「## 历史版本」只写这一份文件自己的改动。规则：.claude/rules/changelog-format.md。')
if duplicate_new:
    failed = True
    print(f'  ✗ {len(duplicate_new)} 组同一节里同一个日期开了几个 `### ` 块')  # gate-lint:summary
    for item in duplicate_new:
        print(f'     {item}')  # gate-lint:detail
    print('     → 同一天的改动并进一个 `### 日期` 块，块下每条改动一个顶格 `- ` 项（先写的在上面）；多出来的 `### 日期` 行删掉。')
if wrapper_new:
    failed = True
    print(f'  ✗ {len(wrapper_new)} 行新写进变更史的旧形态包装（`#### ` 子条目、「> 快查·」行或「**现状**：」行）')  # gate-lint:summary
    for item in wrapper_new:
        print(f'     {item}')  # gate-lint:detail
    print('     → 变更写成顶格 `- ` 项、结论写在项里；快查两行删掉；现状只在 decisions.md / experiments.md 的索引表里写。规则：.claude/rules/changelog-format.md。')
if empty_new:
    failed = True
    print(f'  ✗ {len(empty_new)} 个新开的 `### 日期` 块下一条 `- ` 变更项都没有')  # gate-lint:summary
    for item in empty_new:
        print(f'     {item}')  # gate-lint:detail
    print('     → 在块下写至少一条顶格 `- ` 项说明改了什么，或者删掉这个空块。')
if failed:
    print(f'     {scope}；{stock}')
    print(f'     → 改完只重跑这一格：{CHECK_COMMAND}')
    sys.exit(1)
if date_headings == 0:
    print(f'  ! {KB} 下 {len(files)} 份 .md 里一个 `### 日期` 标题都没有，这一格无对象可判')
    sys.exit(77)
print(f'  ✓ 变更史的形状对：查了 {len(files)} 份 kb 文件、{date_headings} 个 `### 日期` 标题'
      f'（两份变更史里 {history_date_headings} 个），{scope}，新增的没有违例；{stock}')

PY
}

# ════ 格「候选代号不跨分项复用」════
# 同一份决策里，同一个候选代号（甲乙丙丁戊己庚辛壬癸，或单个大写拉丁字母）不许出现在两个分项的候选表里：
# 检索到其中一条时分不出这个代号属于哪一格（C256：D19 已定项 7 的丙是「都不存」，已定项 8 的草案也有丙，指的是别的东西）。
# 扫 .claude/kb/decisions/*.md「## 历史版本」之前、围栏之外的行，按分项小节切块：`### / #### 已定项 N` 与 `未定项 N` 到下一个同级或更高级标题；
#   块外的行（分项索引、连锁一节）不扫。
# 候选表认两种形态：表格某一列整格是单字代号（去掉 ** 与反引号后），且这一列至少有两个不同的代号；
#   行首列表「- 甲：」「甲）」「（甲）」「- **乙**：」，相距不超过 6 行的一组里至少有两个不同的代号。判不出候选表的块不判。
# 同一个代号在两个不同分项块的候选表里都出现判红，点名文件、代号与两处行号；同一块里的几张表共用一个代号不算。
# 带了分项前缀的写法（「已定项 7 的丙」）不是表格整格也不在行首，本来就不算。看不见：「甲（采用）」这类带括注的格、正文里的「候选甲」「臂乙」。
# 弄坏开关 DOC_DECISIONS_BREAK=candidate-codes-unchecked：复用不判红。
# 样本：candidate-codes-red 两个分项的候选表都用丙（一张表格、一张列表），判红；candidate-codes-green 两个分项代号不同、另一处写「已定项 7 的丙」，判绿。
grid_candidate_codes() (
  [[ -d .claude/kb/decisions ]] || { echo "  ! 找不到 .claude/kb/decisions，这一格跳过"; exit 77; }

  python3 - "$BREAK_SWITCH" <<'PY'
import glob, re, sys

BREAK = sys.argv[1]
CHECK_COMMAND = 'bash .claude/gate.d/doc-decisions.sh --check candidate-codes'
DECISIONS = '.claude/kb/decisions'
FENCE = re.compile(r'^[ \t]*```')
HEADING = re.compile(r'^(#{1,6})[ \t]+(.*?)[ \t]*$')
ITEM_HEADING = re.compile(r'^(已定项|未定项)[ \t]*(\d+)')
OWN_HISTORY = re.compile(r'^## 历史版本[ \t]*$')
SEPARATOR_ROW = re.compile(r'^[\s:|\-]+$')
HEAVENLY = '甲乙丙丁戊己庚辛壬癸'
CODE_CELL = re.compile(rf'^[{HEAVENLY}A-Z]$')
LIST_CODE = re.compile(rf'^[ \t]*(?:[-*][ \t]+)?\**[（(]?([{HEAVENLY}A-Z])\**[）)：:]')
LIST_GAP = 6
UNESCAPED_PIPE = re.compile(r'(?<!\\)\|')

files = sorted(glob.glob(f'{DECISIONS}/*.md'))
if not files:
    print(f'  ! 找不到 {DECISIONS} 下的决策文件，这一格无对象可判')
    sys.exit(77)


def cell_text(raw):
    return re.sub(r'[*`\s]', '', raw)


def blocks_of(path):
    """[(块名, [(行号, 行)])]：分项小节到下一个同级或更高级标题；历史节之后和围栏里的行不算。"""
    blocks, current, fenced = [], None, False
    for number, line in enumerate(open(path, encoding='utf-8').read().splitlines(), 1):
        if OWN_HISTORY.match(line):
            break
        if FENCE.match(line):
            fenced = not fenced
            continue
        if fenced:
            continue
        heading = HEADING.match(line)
        if heading:
            level = len(heading.group(1))
            if current is not None and level <= current[0]:
                blocks.append(current[1:])
                current = None
            item = ITEM_HEADING.match(heading.group(2))
            if item and level in (3, 4) and current is None:
                current = (level, f'{item.group(1)} {item.group(2)}', [])
            continue
        if current is not None:
            current[2].append((number, line))
    if current is not None:
        blocks.append(current[1:])
    return blocks


def table_groups(lines):
    """候选表：表格某一列整格是单字代号、这一列至少两个不同的代号。[[(代号, 行号)]]，一张表的一列一组。"""
    found, table = [], []
    for number, line in lines + [(0, '')]:
        if line.lstrip().startswith('|'):
            table.append((number, line))
            continue
        if table:
            columns = {}
            for row_number, row in table:
                if SEPARATOR_ROW.match(row.strip()):
                    continue
                cells = [cell_text(c) for c in UNESCAPED_PIPE.split(row.strip().strip('|'))]
                for index, cell in enumerate(cells):
                    if CODE_CELL.match(cell):
                        columns.setdefault(index, []).append((cell, row_number))
            for hits in columns.values():
                if len({code for code, _ in hits}) >= 2:
                    found.append(hits)
            table = []
    return found


def list_groups(lines):
    """候选列表：行首「- 甲：」「甲）」这类，相距不超过 LIST_GAP 行的一组里至少两个不同的代号。[[(代号, 行号)]]。"""
    hits = []
    for number, line in lines:
        match = LIST_CODE.match(line)
        if match:
            hits.append((match.group(1), number))
    found, group = [], []
    for hit in hits + [None]:
        if hit is not None and (not group or hit[1] - group[-1][1] <= LIST_GAP):
            group.append(hit)
            continue
        if len({code for code, _ in group}) >= 2:
            found.append(group)
        group = [hit] if hit is not None else []
    return found


scanned, tables, reused, all_codes = 0, 0, [], set()
for path in files:
    scanned += 1
    where = {}
    for name, lines in blocks_of(path):
        groups = table_groups(lines) + list_groups(lines)
        tables += len(groups)
        first = {}
        for group in groups:
            for code, number in group:
                first.setdefault(code, number)
        for code, number in first.items():
            where.setdefault(code, []).append((name, number))
            all_codes.add(code)
    for code, places in sorted(where.items()):
        if len(places) >= 2:
            reused.append((path, code, places))

if reused and BREAK == 'candidate-codes-unchecked':
    print(f'  ✓ 弄坏开关 DOC_DECISIONS_BREAK=candidate-codes-unchecked：{len(reused)} 个复用的候选代号没判')
    sys.exit(0)
if reused:
    print(f'  ✗ {len(reused)} 个候选代号在同一份决策的两个分项候选表里复用')  # gate-lint:summary
    for path, code, places in reused:
        text = '、'.join(f'{name}（第 {number} 行）' for name, number in places)
        print(f'     {path} 的候选代号「{code}」同时在 {text}的候选表里，检索到其中一条时分不出它属于哪一格')  # gate-lint:detail
    sys.exit(1)
if tables == 0:
    print(f'  ! 扫了 {scanned} 份决策，一处候选表都没判出来，这一格无对象可判')
    sys.exit(77)
print(f'  ✓ 同一份决策里的候选代号没有跨分项复用：扫了 {scanned} 份决策、{tables} 处候选表、{len(all_codes)} 个代号')
PY
)

# ════ 登记的格函数：弄坏开关点名这一格就不跑、按判过了记，否则调这一格的判法 ════
# 前十四格的 stdin 接 /dev/null，与它们原先在后台跑时读到的一样。
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
judge_history_loss()            { break_skips history-loss;            cell_history_loss; }
judge_candidate_codes()         { grid_candidate_codes </dev/null; }

# ════ --write：在子 shell 里重写 decision-items-sync 的两块投影，写完回读判一遍 ════
if ((write_mode)); then
  write_cells=(decision-items-sync)
  if ((STAGE_CELLS_CHECK_GIVEN)); then
    for write_cell in "${STAGE_CELLS_SELECTED[@]}"; do
      if [[ "$write_cell" != decision-items-sync ]]; then
        bad "--write 只重写 decision-items-sync（decisions.md 的分项清单与索引表状态列）管的投影，不能与 --check $write_cell 一起用"
        howto "去掉 --check，或写成 --check decision-items-sync：bash .claude/gate.d/doc-decisions.sh --write"
        exit 2
      fi
    done
  fi
  echo "── decision-items-sync（--write）"
  if ( DECISION_ITEMS_WRITE=1; grid_decision_items_sync ); then write_exit=0; else write_exit=$?; fi
  if ((write_exit == 77)); then
    echo "  ! --write 写的 ${#write_cells[@]} 块都本次无对象可判（${write_cells[*]}），不记通过"
    exit 77
  fi
  if ((write_exit != 0)); then
    bad "--write 写了 ${#write_cells[@]} 块，写完回读仍红：${write_cells[*]}（退出 ${write_exit}）"
    howto "照上面那一块自己的红行改（节或索引行缺了 --write 补不了，先核编号），再跑 bash .claude/gate.d/doc-decisions.sh --write"
    exit 1
  fi
  ok "--write 写完 ${#write_cells[@]} 块、回读都对：${write_cells[*]}"
  exit 0
fi


stage_cells_run "$ROOT"
